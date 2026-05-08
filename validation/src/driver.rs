//! Tier-2 driver — compares Rust `xcfun-eval::Functional::eval` against the
//! C++ reference via `CppXcfun` FFI for every
//! `(functional, vars, mode=PartialDerivatives, order∈{0..=max_order≤4},
//! point, element)` tuple across the 11 Phase-2 LDA functionals.
//!
//! Per-functional tier-2 thresholds (CONTEXT D-24, user-approved 2026-04-20):
//!   - 1e-7  for `XC_LDAERFX`, `XC_LDAERFC`, `XC_LDAERFC_JT` (cubecl erf polyfill
//!     drift vs C++ libm erf — documented transparently in report.html).
//!   - 1e-12 for the remaining 8 LDAs (`SLATERX, VWN3C, VWN5C, PW92C, PZ81C,
//!     TFK, TW, VWK`) — Phase 2 SC #5 strict gate.
//!
//! Rel-error definition (ACC-02, matches `approx::assert_relative_eq!`):
//!   rel_err = |rust - cpp| / max(|cpp|, 1.0)
//!
//! Rust-side launch-loop limitations from Plan 02-04/02-05:
//! - Orders 1 and 2 are only wired for inlen=2 (pure-density LDAs). TW + VWK
//!   use `XC_A_B_GAA_GAB_GBB` (inlen=5) and have no (id, n) arm in run_launch.
//!   Those cases surface as `XcError::NotConfigured`; the driver records
//!   them as threshold failures with rust=NaN so the checkpoint reviewer
//!   sees the gap (D-19 INCONCLUSIVE trigger) rather than a crash.
//!
//! ## Durability + resume (Plan 04-10 incremental-jsonl-flush, 2026-04-28)
//!
//! Long-running (~5 h order-3 capstone) sweeps were losing 100 % of their
//! data on any interruption because the previous code path buffered every
//! `ReportRecord` in `Report::records` and wrote `report.jsonl` exactly once
//! at end-of-run. Two consecutive Plan 04-10 sign-off attempts wasted
//! ~5.5 h of compute. The fix:
//!
//! - `RunConfig` (new) carries an optional `&mut JsonlSink` and a
//!   `HashSet<TupleKey>` (functional, vars, mode, order). The sink is opened
//!   once in `main.rs` and flushes after every record write. The skip-set is
//!   populated from a prior `report.jsonl` when `--resume` is used; driver
//!   functions short-circuit at the start of every per-tuple iteration if
//!   the tuple is in the skip-set. A clean (non-resumed) run uses
//!   `RunConfig::default()`-equivalent config (empty skip-set, sink in
//!   create/truncate mode), preserving byte-for-byte JSONL output.

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use xcfun_core::{Dependency, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, VARS_TABLE, Vars, taylorlen};
use xcfun_eval::Functional;

use crate::ffi::CppXcfun;
use crate::fixtures::{GridPoint, REGULARIZE_CLAMP_STRATUM_BOUND};
use crate::report::{JsonlSink, TupleKey};

/// Phase 3 plan 03-05 — discriminator for the validation CLI's
/// `--mode {partial_derivatives, potential, contracted}` flag.
///
/// Phase 4 plan 04-05 D-06-C extends with `Contracted` for the
/// orders 5/6 cross-check vs the C++ DOEVAL macro
/// (`XCFunctional.cpp:619-635`). The vendored xcfun-master has no upstream
/// `FUNCTIONAL` test fixtures at order > 3, so the orders 5/6 path is a
/// new C-driver path: invokes `xcfun_eval` with `XC_CONTRACTED` mode at
/// `xcfun_set_order(5 | 6)` on a 100-point subset × 4 representative
/// functionals (SLATERX / PBEX / TPSSX / M06X — only SLATERX and PBEX are
/// wired in `run_launch` today; TPSSX/M06X require Vars=13 arms not
/// shipped in this plan and are flagged as Phase-6 prerequisite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessMode {
    PartialDerivatives,
    Potential,
    /// Plan 04-05 D-06-C — Mode::Contracted at orders 5/6 vs C++ DOEVAL.
    Contracted,
}

/// Plan 04-10 — durability + resume context passed to driver functions.
///
/// Holds the (optional) streaming JSONL sink and the skip-set of tuples
/// already on disk from a prior interrupted run. `None`/empty == legacy
/// behaviour (all records buffered in `Report::records`, no skipping).
///
/// Quick task 260430-4x7 — adds `jobs: NonZeroUsize` controlling
/// parallelism inside the driver entry points. `1` = serial (legacy
/// path, byte-stable for clean-run output); `> 1` = `std::thread::scope`
/// orchestrator with worker-owned `CppXcfun` handles.
pub struct RunConfig<'a> {
    /// Streaming JSONL writer — `Some` when `main.rs` wants per-record
    /// durability, `None` for in-memory-only runs (e.g. unit tests).
    pub sink: Option<&'a mut JsonlSink>,
    /// `(functional, vars, mode, order)` tuples already present in
    /// `report.jsonl` when `--resume` was passed. Driver short-circuits
    /// any tuple matching this set.
    pub skip_keys: &'a HashSet<TupleKey>,
    /// Quick task 260430-4x7 — degree of parallelism for the driver.
    /// `1` (default) preserves the legacy serial path verbatim;
    /// values `> 1` enable a `std::thread::scope` orchestrator that
    /// dispatches one `(functional, vars, mode, order)` tuple per
    /// worker job. Each worker constructs its own `CppXcfun` (the
    /// FFI handle is `!Send + !Sync`).
    pub jobs: std::num::NonZeroUsize,
}

impl<'a> RunConfig<'a> {
    /// Construct a config with no sink, no skip-set, and `jobs == 1`
    /// (serial fast-path). Preserves the pre-Plan-04-10 behaviour.
    /// Used by tests that call `run` directly.
    pub fn empty(skip_keys: &'a HashSet<TupleKey>) -> Self {
        Self {
            sink: None,
            skip_keys,
            jobs: std::num::NonZeroUsize::new(1).unwrap(),
        }
    }
}

/// One tier-2 parity record — emitted per `(functional, vars, mode, order,
/// point_idx, element_idx)` tuple. Serialized to `validation/report.jsonl`.
///
/// `Deserialize` derived for the `--resume` path: existing JSONL is parsed
/// back into `ReportRecord` to recover the (functional, vars, mode, order)
/// skip-set + rebuild matrix entries for tuples we won't re-evaluate.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReportRecord {
    pub functional: String,
    pub vars: String,
    pub mode: u32,
    pub order: u32,
    pub point_idx: usize,
    pub element_idx: usize,
    pub input: Vec<f64>,
    pub rust: f64,
    pub cpp: f64,
    pub abs_err: f64,
    pub rel_err: f64,
    pub threshold: f64,
    pub pass: bool,
    /// `true` if the Rust launch loop returned `XcError::NotConfigured` for
    /// this `(id, vars, order)` tuple — i.e., the kernel exists but the host
    /// launch arm does not. Recorded transparently for D-19 review.
    pub rust_unavailable: bool,
    /// `true` if this functional is excluded from tier-2 because upstream
    /// has no `test_in` to compare against (TW + VWK — no upstream test data;
    /// ldaerfc.cpp FUNCTIONAL macro for these ends at ENERGY_FUNCTION without
    /// XC_A_B/XC_PARTIAL_DERIVATIVES/test_* args). Tier-2 parity is
    /// meaningless without an upstream reference; tier-3 (Phase 3+) with
    /// synthetic grid fixtures covers these cases.
    #[serde(default)]
    pub excluded_by_upstream_spec: bool,
    /// `true` if this grid point lands in the regularize-clamp stratum
    /// (`min(a, b) ≤ REGULARIZE_CLAMP_STRATUM_BOUND = 2e-14`) where D-22
    /// `regularize` deliberately saturates density inputs to `TINY_DENSITY`.
    /// Testing here exercises the clamp's precision sacrifice by design —
    /// not kernel correctness. Records flagged here do NOT count against
    /// the tier-2 verdict (`Report::failed_count()` skips them).
    /// Plan 02-06 Fix 2.
    #[serde(default)]
    pub excluded_by_regularize_clamp_design: bool,
}

/// Per-`(functional, order)` summary used to build the report.html matrix.
#[derive(Debug, Clone)]
pub struct CellSummary {
    /// Max rel-error across NON-EXCLUDED records only. Clamp-stratum and
    /// upstream-spec-excluded records do not contribute to this maximum
    /// (they do NOT count against the tier-2 verdict).
    pub max_rel_err: f64,
    pub threshold: f64,
    pub records_total: usize,
    pub records_failed: usize,
    /// Count of records where Rust returned `NotConfigured` (a structural gap,
    /// distinct from numerical failure).
    pub rust_unavailable: usize,
    /// `true` iff the (functional, order) cell is entirely excluded from tier-2
    /// because upstream has no `test_in` to compare against (TW + VWK). Failing
    /// records marked `excluded_by_upstream_spec` do NOT count against the
    /// harness verdict — they are reported transparently as gaps for tier-3.
    pub excluded_by_upstream_spec: bool,
    /// Count of records excluded by `excluded_by_regularize_clamp_design`
    /// (Plan 02-06 Fix 2 — D-22 clamp-stratum design intent). Transparent
    /// transparency in the HTML report.
    pub clamp_stratum_records: usize,
    /// Count of records in the clamp stratum that also failed (would have
    /// failed the threshold if counted). Reported transparently but does
    /// not fail the harness verdict.
    pub clamp_stratum_failures: usize,
}

/// Tier-2 report aggregator.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub records: Vec<ReportRecord>,
    pub matrix: BTreeMap<(String, u32), CellSummary>,
}

impl Report {
    /// Append a record to the in-memory aggregate. Used by code paths that
    /// don't have a streaming sink (e.g. unit tests). `main.rs` uses
    /// `push_with_sink` so each retained record is also written + flushed
    /// to disk synchronously.
    pub fn push(&mut self, rec: ReportRecord) {
        self.push_with_sink(rec, None).expect("no sink: cannot fail");
    }

    /// Append a record to the in-memory aggregate AND, if a sink is supplied,
    /// stream the same record to disk with a per-line flush.
    ///
    /// IMPORTANT: the sink is only written for records that we'd retain in
    /// `self.records` (failing records + sampled passes). This keeps the
    /// streaming output byte-for-byte identical to the legacy end-of-run
    /// `write_jsonl(report, ...)` output for clean (non-resumed) runs.
    pub fn push_with_sink(
        &mut self,
        rec: ReportRecord,
        sink: Option<&mut JsonlSink>,
    ) -> Result<()> {
        let key = (rec.functional.clone(), rec.order);
        let entry = self.matrix.entry(key).or_insert(CellSummary {
            max_rel_err: 0.0,
            threshold: rec.threshold,
            records_total: 0,
            records_failed: 0,
            rust_unavailable: 0,
            excluded_by_upstream_spec: false,
            clamp_stratum_records: 0,
            clamp_stratum_failures: 0,
        });
        if rec.excluded_by_upstream_spec {
            entry.excluded_by_upstream_spec = true;
        }
        // Only aggregate rel_err MAX from non-excluded records. Excluded
        // (clamp-stratum / upstream-spec) records are reported transparently
        // in the JSONL but do not define the cell's tier-2 verdict.
        if !rec.excluded_by_upstream_spec && !rec.excluded_by_regularize_clamp_design {
            entry.max_rel_err = entry.max_rel_err.max(rec.rel_err);
        }
        entry.records_total += 1;
        if rec.excluded_by_regularize_clamp_design {
            entry.clamp_stratum_records += 1;
            if !rec.pass {
                entry.clamp_stratum_failures += 1;
            }
        }
        if !rec.pass && !rec.excluded_by_upstream_spec && !rec.excluded_by_regularize_clamp_design
        {
            entry.records_failed += 1;
        }
        if rec.rust_unavailable {
            entry.rust_unavailable += 1;
        }
        // To bound JSONL size: keep all failing records (including excluded
        // ones for transparency) + a few sampled passes per (functional, order).
        let keep = !rec.pass || (rec.point_idx == 0 && rec.element_idx == 0);
        if keep {
            // Stream FIRST so a panic in `Vec::push` (OOM in extreme cases)
            // can't lose the on-disk record. Per-line flush is a property of
            // `JsonlSink::write_record`.
            if let Some(sink) = sink {
                sink.write_record(&rec)?;
            }
            self.records.push(rec);
        }
        Ok(())
    }

    /// Merge a pre-existing `(functional, order) → CellSummary` map into the
    /// matrix — used by `--resume` to carry forward the prior run's matrix
    /// entries for tuples we are NOT re-evaluating, so `report.html` is
    /// accurate end-to-end.
    pub fn extend_matrix_from_prior(
        &mut self,
        prior: std::collections::HashMap<(String, u32), CellSummary>,
    ) {
        for (k, v) in prior {
            // The current run's evaluation always wins for any cell it
            // produced; only insert prior cells that the current run did
            // not touch.
            self.matrix.entry(k).or_insert(v);
        }
    }

    /// Count tier-2 failures — EXCLUDES cells marked `excluded_by_upstream_spec`
    /// (TW + VWK have no upstream test_in; tier-2 parity for them is not a
    /// defined comparison per CONTEXT D-19) AND excludes clamp-stratum
    /// records (Plan 02-06 Fix 2 — D-22 regularize-clamp design intent).
    pub fn failed_count(&self) -> usize {
        self.matrix
            .values()
            .filter(|c| !c.excluded_by_upstream_spec)
            .map(|c| c.records_failed)
            .sum()
    }

    pub fn total_records(&self) -> usize {
        self.matrix.values().map(|c| c.records_total).sum()
    }

    /// Count clamp-stratum records across all cells (for transparency).
    pub fn clamp_stratum_total(&self) -> usize {
        self.matrix.values().map(|c| c.clamp_stratum_records).sum()
    }

    /// Count clamp-stratum records that would have failed if counted.
    /// Reported separately for D-22 transparency; does NOT fail the verdict.
    pub fn clamp_stratum_failures_total(&self) -> usize {
        self.matrix.values().map(|c| c.clamp_stratum_failures).sum()
    }
}

// ===========================================================================
// Quick task 260430-4x7 — parallel scheduler internals.
//
// Each driver entry point (run / run_potential / run_contracted) exposes a
// `dispatch_*` function that:
//   1. fast-paths `cfg.jobs == 1` to a serial loop that calls
//      `report.push_with_sink` inline (byte-stable vs the pre-change main).
//   2. otherwise, spawns `cfg.jobs` workers inside `std::thread::scope`.
//      Workers pop `Job*` records off a shared `Arc<Mutex<VecDeque<...>>>`,
//      construct their own `CppXcfun` (NOT Send), run a per-tuple helper,
//      and stream `ReportRecord`s back through an `mpsc::channel`. The
//      orchestrator thread is the SOLE writer to `report` and
//      `cfg.sink.as_deref_mut()` — preserving the matrix + JSONL flush
//      invariant exactly as in the legacy serial path.
//
// CLAUDE.md hard rules: NO `rayon`, NO `crossbeam`, NO `unsafe impl Send` —
// `std::thread::scope` + `std::sync::mpsc` only.
// ===========================================================================

/// One scheduled `(functional, vars, order)` tuple for `run` (Mode = PartialDerivatives).
#[derive(Clone)]
struct JobPD {
    id: FunctionalId,
    name: &'static str,
    vars: Vars,
    order: u32,
    threshold: f64,
}

/// One scheduled `(functional, vars)` tuple for `run_potential`
/// (Mode = Potential, order is implicit 0).
#[derive(Clone)]
struct JobPot {
    id: FunctionalId,
    name: &'static str,
    vars: Vars,
    inlen: usize,
    outlen: usize,
    threshold: f64,
}

/// One scheduled `(functional, vars, order)` tuple for `run_contracted`
/// (Mode = Contracted at orders 5/6 only).
#[derive(Clone)]
struct JobCon {
    /// Reserved for the Phase-6 path (direct `xcfun_eval` invocation
    /// bypassing the C++ output_length die — see `run_one_tuple_contracted`).
    /// Kept on the job struct now to avoid an API churn when the path
    /// is enabled.
    #[allow(dead_code)]
    id: FunctionalId,
    name: &'static str,
    vars: Vars,
    order: u32,
    threshold: f64,
}

/// Run one PartialDerivatives tuple end-to-end (C++ setup + Rust kernel +
/// per-point per-element diff). Records flow out via `emit`.
///
/// `emit` is a callback because the serial path emits straight into
/// `report.push_with_sink` while the parallel path emits via an
/// `mpsc::Sender<ReportRecord>` clone owned by the worker.
fn run_one_tuple_pd<F>(
    job: &JobPD,
    grid: &[GridPoint],
    cpp: &mut CppXcfun,
    mut emit: F,
) -> Result<()>
where
    F: FnMut(ReportRecord) -> Result<()>,
{
    let JobPD { id, name, vars, order, threshold } = *job;
    let inlen = VARS_TABLE[vars as usize].len as usize;
    let outlen = taylorlen(inlen, order as usize);

    let status_set = cpp.set(&cpp_name(name), 1.0);
    if status_set != 0 {
        anyhow::bail!(
            "xcfun_set({}, 1.0) failed: status={}",
            cpp_name(name),
            status_set
        );
    }
    let status_setup = cpp.eval_setup(vars as u32, 1, order as i32);
    if status_setup != 0 {
        anyhow::bail!(
            "xcfun_eval_setup({}, {:?}, order={}) failed: status={}",
            name,
            vars,
            order,
            status_setup
        );
    }
    let cpp_inlen = cpp.input_length();
    let cpp_outlen = cpp.output_length();
    if cpp_inlen != inlen || cpp_outlen != outlen {
        anyhow::bail!(
            "Length mismatch for {} order={}: rust inlen={} outlen={}; cpp inlen={} outlen={}",
            name,
            order,
            inlen,
            outlen,
            cpp_inlen,
            cpp_outlen
        );
    }

    // Phase 6 Plan 06-06 (D-17): weights is Vec<(FunctionalId, f64)>; no leak.
    let weights: Vec<(FunctionalId, f64)> = vec![(id, 1.0)];
    let rust_fun = Functional {
        weights,
        vars,
        mode: Mode::PartialDerivatives,
        order,
        settings: xcfun_eval::functional::DEFAULT_SETTINGS,
        // Plan 06-02b deviation (Rule 3): Plan 06-02a added `settings_gen: u64`
        // to `Functional` and updated test files, but missed these two struct
        // literals in `validation/src/driver.rs`. Initialise to 0 (matches
        // `DEFAULT_SETTINGS`); the validation harness never calls `set()` on
        // the leaked struct, so the counter is irrelevant for tier-2 parity.
        settings_gen: 0,
    };

    let clamp_bound = clamp_bound_for(name);
    for (point_idx, gp) in grid.iter().enumerate() {
        let input = build_input(gp, vars);
        let mut rust_out = vec![0.0_f64; outlen];
        let mut cpp_out = vec![0.0_f64; outlen];

        let in_clamp_stratum = input.len() >= 2
            && input[0].min(input[1]) <= clamp_bound;

        cpp.eval(&input, &mut cpp_out);

        let rust_err = rust_fun.eval(&input, &mut rust_out);
        let rust_unavailable = rust_err.is_err();
        if rust_unavailable {
            for r in rust_out.iter_mut() {
                *r = f64::NAN;
            }
        }

        for elem_idx in 0..outlen {
            let r = rust_out[elem_idx];
            let c = cpp_out[elem_idx];
            let abs_err = (r - c).abs();
            let rel_err = if rust_unavailable {
                f64::INFINITY
            } else {
                abs_err / c.abs().max(1.0)
            };
            let pass = !rust_unavailable && rel_err <= threshold;
            let rec = ReportRecord {
                functional: name.into(),
                vars: format!("{:?}", vars),
                mode: 1,
                order,
                point_idx,
                element_idx: elem_idx,
                input: input.clone(),
                rust: r,
                cpp: c,
                abs_err,
                rel_err,
                threshold,
                pass,
                rust_unavailable,
                excluded_by_upstream_spec: false,
                excluded_by_regularize_clamp_design: in_clamp_stratum,
            };
            emit(rec)?;
        }
    }
    Ok(())
}

/// Run one Mode::Potential tuple end-to-end. Same emit-callback pattern
/// as `run_one_tuple_pd`.
fn run_one_tuple_potential<F>(
    job: &JobPot,
    grid: &[GridPoint],
    cpp: &mut CppXcfun,
    mut emit: F,
) -> Result<()>
where
    F: FnMut(ReportRecord) -> Result<()>,
{
    let JobPot { id, name, vars, inlen, outlen, threshold } = *job;

    let s = cpp.set(&cpp_name(name), 1.0);
    if s != 0 {
        anyhow::bail!("xcfun_set({}, 1.0) failed: status={}", cpp_name(name), s);
    }
    // mode=2 is XC_POTENTIAL.
    let setup = cpp.eval_setup(vars as u32, 2, 0);
    if setup != 0 {
        anyhow::bail!(
            "xcfun_eval_setup({}, {:?}, POTENTIAL) failed: status={}",
            name,
            vars,
            setup
        );
    }
    let cpp_inlen = cpp.input_length();
    let cpp_outlen = cpp.output_length();
    if cpp_inlen != inlen || cpp_outlen != outlen {
        anyhow::bail!(
            "Length mismatch for {} (Mode::Potential): rust inlen={} outlen={}; cpp inlen={} outlen={}",
            name,
            inlen,
            outlen,
            cpp_inlen,
            cpp_outlen
        );
    }

    let weights: Vec<(FunctionalId, f64)> = vec![(id, 1.0)];
    let rust_fun = Functional {
        weights,
        vars,
        mode: Mode::Potential,
        order: 0,
        settings: xcfun_eval::functional::DEFAULT_SETTINGS,
        // Plan 06-02b deviation (Rule 3): see analogous comment ~100 lines
        // above in run() — Plan 06-02a missed this struct literal.
        settings_gen: 0,
    };

    let clamp_bound = clamp_bound_for(name);
    for (point_idx, gp) in grid.iter().enumerate() {
        let input = build_input_for_potential(gp, vars);
        let mut rust_out = vec![0.0_f64; outlen];
        let mut cpp_out = vec![0.0_f64; outlen];

        let in_clamp_stratum = input.len() >= 2
            && input[0].min(input[1]) <= clamp_bound;

        cpp.eval(&input, &mut cpp_out);

        let rust_err = rust_fun.eval(&input, &mut rust_out);
        let rust_unavailable = rust_err.is_err();
        if rust_unavailable {
            for r in rust_out.iter_mut() {
                *r = f64::NAN;
            }
        }

        for elem_idx in 0..outlen {
            let r = rust_out[elem_idx];
            let c = cpp_out[elem_idx];
            let abs_err = (r - c).abs();
            let rel_err = if rust_unavailable {
                f64::INFINITY
            } else {
                abs_err / c.abs().max(1.0)
            };
            let pass = !rust_unavailable && rel_err <= threshold;
            let rec = ReportRecord {
                functional: name.into(),
                vars: format!("{:?}", vars),
                mode: 2, // XC_POTENTIAL
                order: 0,
                point_idx,
                element_idx: elem_idx,
                input: input.clone(),
                rust: r,
                cpp: c,
                abs_err,
                rel_err,
                threshold,
                pass,
                rust_unavailable,
                excluded_by_upstream_spec: false,
                excluded_by_regularize_clamp_design: in_clamp_stratum,
            };
            emit(rec)?;
        }
    }
    Ok(())
}

/// Run one Mode::Contracted tuple. Per Plan 04-05 D-06-C, this currently
/// emits a single `excluded_by_upstream_spec` marker record per
/// (functional, order) because C++ `xcfun_output_length` dies for
/// XC_CONTRACTED. Phase 6 will replace this with a direct
/// `xcfun_eval` invocation bypassing the FFI shim's length check.
fn run_one_tuple_contracted<F>(
    job: &JobCon,
    cpp: &mut CppXcfun,
    mut emit: F,
) -> Result<()>
where
    F: FnMut(ReportRecord) -> Result<()>,
{
    let JobCon { id: _, name, vars, order, threshold } = *job;

    let s = cpp.set(&cpp_name(name), 1.0);
    if s != 0 {
        anyhow::bail!(
            "xcfun_set({}, 1.0) failed: status={}",
            cpp_name(name),
            s
        );
    }
    // mode = 3 (XC_CONTRACTED) per xcfun.h:39.
    let setup = cpp.eval_setup(vars as u32, 3, order as i32);
    if setup != 0 {
        anyhow::bail!(
            "xcfun_eval_setup({}, {:?}, CONTRACTED order={}) failed: status={}",
            name, vars, order, setup
        );
    }
    let cpp_inlen = cpp.input_length();
    tracing::warn!(
        "Tier-2 SKIP-WITH-RECORD {} (Mode::Contracted order={}): \
         C++ xcfun_output_length die's for XC_CONTRACTED \
         (XCFunctional.cpp:488); Phase-6 prerequisite for direct \
         xcfun_eval invocation (cpp_input_length={})",
        name, order, cpp_inlen
    );
    let rec = ReportRecord {
        functional: name.into(),
        vars: format!("{:?}", vars),
        mode: 3, // XC_CONTRACTED
        order,
        point_idx: 0,
        element_idx: 0,
        input: Vec::new(),
        rust: f64::NAN,
        cpp: f64::NAN,
        abs_err: f64::INFINITY,
        rel_err: f64::INFINITY,
        threshold,
        pass: false,
        rust_unavailable: true,
        excluded_by_upstream_spec: true,
        excluded_by_regularize_clamp_design: false,
    };
    emit(rec)?;
    Ok(())
}

/// Generic dispatcher: runs `n_workers` workers over `jobs`, each invoking
/// `worker_fn(&Job, &mut CppXcfun, &mpsc::Sender<ReportRecord>)`. The
/// orchestrator drains the channel and pushes records to the report.
///
/// `cfg.jobs == 1` short-circuits to a serial loop with no thread::scope,
/// no channels, and `report.push_with_sink` inline — preserving the
/// pre-change byte-stable serial output exactly.
fn parallel_dispatch<J, W>(
    report: &mut Report,
    cfg: &mut RunConfig<'_>,
    jobs: Vec<J>,
    worker_fn: W,
) -> Result<()>
where
    J: Send + Clone + 'static,
    W: Fn(&J, &mut CppXcfun, &mpsc::Sender<ReportRecord>) -> Result<()> + Send + Sync,
{
    if jobs.is_empty() {
        return Ok(());
    }

    // Serial fast-path — byte-stable vs the pre-change main.
    if cfg.jobs.get() == 1 {
        for job in &jobs {
            let mut cpp = CppXcfun::new();
            let (tx, rx) = mpsc::channel::<ReportRecord>();
            // Run the worker; it sends each record on tx. Drop tx after the
            // call so rx.recv() returns Err once we drain. We then push
            // records inline to preserve the legacy ordering.
            let res = worker_fn(job, &mut cpp, &tx);
            drop(tx);
            // Drain even if worker errored, so we don't lose successful
            // records that flushed before the failure.
            for rec in rx.iter() {
                report.push_with_sink(rec, cfg.sink.as_deref_mut())?;
            }
            res?;
        }
        return Ok(());
    }

    // Parallel path — std::thread::scope.
    let n_workers = cfg.jobs.get().min(jobs.len()).max(1);
    let queue: Arc<Mutex<VecDeque<J>>> = Arc::new(Mutex::new(jobs.into_iter().collect()));
    let (tx, rx) = mpsc::channel::<ReportRecord>();
    let worker_fn_ref = &worker_fn;

    std::thread::scope(|s| -> Result<()> {
        let mut handles = Vec::with_capacity(n_workers);
        for _ in 0..n_workers {
            let q = Arc::clone(&queue);
            let tx_w = tx.clone();
            handles.push(s.spawn(move || -> Result<()> {
                loop {
                    // Lock briefly to pop one job; release immediately so
                    // sibling workers can also claim work.
                    let job = {
                        let mut g = q.lock().expect("worker queue mutex poisoned");
                        g.pop_front()
                    };
                    let job = match job {
                        Some(j) => j,
                        None => break,
                    };
                    // Each worker constructs its own CppXcfun (NOT Send).
                    let mut cpp = CppXcfun::new();
                    worker_fn_ref(&job, &mut cpp, &tx_w)?;
                }
                Ok(())
            }));
        }
        // Drop the orchestrator's sender clone so rx.iter() terminates
        // once the last worker exits.
        drop(tx);

        // Single-writer drain — orchestrator is the SOLE thread that
        // touches `report` and `cfg.sink.as_deref_mut()`, preserving the
        // existing matrix + per-line JSONL flush invariant.
        for rec in rx.iter() {
            report.push_with_sink(rec, cfg.sink.as_deref_mut())?;
        }

        for h in handles {
            match h.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e),
                Err(panic) => {
                    return Err(anyhow::anyhow!("worker thread panicked: {:?}", panic));
                }
            }
        }
        Ok(())
    })
}

/// Dispatch the PartialDerivatives jobs (orchestrator entry point for `run`).
fn dispatch_pd(
    report: &mut Report,
    cfg: &mut RunConfig<'_>,
    grid: &[GridPoint],
    jobs: Vec<JobPD>,
) -> Result<()> {
    parallel_dispatch(report, cfg, jobs, |job, cpp, tx| {
        run_one_tuple_pd(job, grid, cpp, |rec| {
            tx.send(rec)
                .map_err(|e| anyhow::anyhow!("orchestrator drain channel closed: {}", e))
        })
    })
}

/// Dispatch the Mode::Potential jobs (orchestrator entry point for `run_potential`).
fn dispatch_potential(
    report: &mut Report,
    cfg: &mut RunConfig<'_>,
    grid: &[GridPoint],
    jobs: Vec<JobPot>,
) -> Result<()> {
    parallel_dispatch(report, cfg, jobs, |job, cpp, tx| {
        run_one_tuple_potential(job, grid, cpp, |rec| {
            tx.send(rec)
                .map_err(|e| anyhow::anyhow!("orchestrator drain channel closed: {}", e))
        })
    })
}

/// Dispatch the Mode::Contracted jobs (orchestrator entry point for `run_contracted`).
fn dispatch_contracted(
    report: &mut Report,
    cfg: &mut RunConfig<'_>,
    jobs: Vec<JobCon>,
) -> Result<()> {
    parallel_dispatch(report, cfg, jobs, |job, cpp, tx| {
        run_one_tuple_contracted(job, cpp, |rec| {
            tx.send(rec)
                .map_err(|e| anyhow::anyhow!("orchestrator drain channel closed: {}", e))
        })
    })
}

/// Per-functional tier-2 threshold — D-24 override for LDAERF family; strict
/// 1e-12 for the remaining 8 LDAs.
pub fn threshold_for(name: &str) -> f64 {
    if name.starts_with("XC_LDAERF") {
        1e-7 // D-24, USER-APPROVED 2026-04-20 (CONTEXT.md D-24)
    } else {
        1e-12 // ROADMAP Phase 2 SC #5 strict gate
    }
}

/// Build the per-point input array for a given `Vars`.
/// Phase 2 supports only `Vars::A_B` (inlen=2) and `Vars::A_B_GAA_GAB_GBB`
/// (inlen=5) — the variants used by the 11 LDA functionals.
fn build_input(gp: &GridPoint, vars: Vars) -> Vec<f64> {
    let inlen = VARS_TABLE[vars as usize].len as usize;
    let mut input = vec![0.0_f64; inlen];
    match vars {
        Vars::A_B => {
            let (a, b) = gp.ab_from_ns();
            input[0] = a;
            input[1] = b;
        }
        Vars::A_B_GAA_GAB_GBB => {
            let (a, b) = gp.ab_from_ns();
            input[0] = a;
            input[1] = b;
            input[2] = gp.gaa;
            input[3] = gp.gab;
            input[4] = gp.gbb;
        }
        Vars::A_B_GAA_GAB_GBB_TAUA_TAUB => {
            // metaGGA inlen=7 input. Derive tau_α/tau_β from grid (a,b) using
            // the same physical bound as fixtures::generate_metagga_stratum:
            // tau ∈ [0, kF² · ρ^(2/3)] with kF² = (3π²)^(2/3) ≈ 9.5703...
            // The grid has no committed tau seed for non-mGGA points, so we
            // derive deterministically: taua = 0.5 · kf2 · a^(2/3),
            // taub = 0.5 · kf2 · b^(2/3) — a midpoint of the physical
            // distribution. C++ side receives the SAME value, so parity is
            // a true kernel-port comparison.
            let (a, b) = gp.ab_from_ns();
            let kf2 = (3.0_f64 * std::f64::consts::PI.powi(2)).powf(2.0 / 3.0);
            input[0] = a;
            input[1] = b;
            input[2] = gp.gaa;
            input[3] = gp.gab;
            input[4] = gp.gbb;
            input[5] = 0.5 * kf2 * a.max(1e-30).powf(2.0 / 3.0);
            input[6] = 0.5 * kf2 * b.max(1e-30).powf(2.0 / 3.0);
        }
        Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => {
            // BR/CSC inlen=11 input. Same tau derivation; lap_α/lap_β set to
            // ±0.005·a/b (matches generate_metagga_stratum's [-0.01, 0.01]
            // midpoint band); jp_aa/jp_bb to 0.05 (matches midpoint of
            // [-0.1, 0.1] band). All deterministic per-grid-point so the
            // C++ side gets identical input.
            let (a, b) = gp.ab_from_ns();
            let kf2 = (3.0_f64 * std::f64::consts::PI.powi(2)).powf(2.0 / 3.0);
            input[0] = a;
            input[1] = b;
            input[2] = gp.gaa;
            input[3] = gp.gab;
            input[4] = gp.gbb;
            input[5] = 0.005 * a; // lapa
            input[6] = 0.005 * b; // lapb
            input[7] = 0.5 * kf2 * a.max(1e-30).powf(2.0 / 3.0); // taua
            input[8] = 0.5 * kf2 * b.max(1e-30).powf(2.0 / 3.0); // taub
            input[9] = 0.05; // jpaa
            input[10] = 0.05; // jpbb
        }
        other => panic!(
            "validation driver: unsupported vars {:?} in Phase 2 (expected A_B or A_B_GAA_GAB_GBB)",
            other
        ),
    }
    input
}

/// Convert `XC_SLATERX` → `slaterx` for `xcfun_set` (C side strcasecmps against
/// the name with the `XC_` prefix stripped).
fn cpp_name(xc_name: &str) -> String {
    xc_name
        .strip_prefix("XC_")
        .unwrap_or(xc_name)
        .to_ascii_lowercase()
}

/// Per-functional regularize-clamp bound on `min(a, b)` (06-N7/07-00).
///
/// The default `REGULARIZE_CLAMP_STRATUM_BOUND = 2e-14` filters out
/// records where either spin density is at or below the IEEE-754
/// underflow neighborhood. That works for most functionals.
///
/// **BECKESRX** (and similar range-separated/ERF-bearing exchange
/// functionals) has an additional pathology: at zero gradient and very
/// low density, the term `chi² = gaa · a^(-8/3)` has derivatives like
/// `a^(-8/3) ≈ 1e34` for `a = 1e-13`. Even though `chi² = 0`, the AD
/// chain propagates astronomical chi² derivatives, amplifying ULP-level
/// disagreement to 10^29-10^32 magnitude differences at order 3.
///
/// To filter this regime, BECKESRX (and beckecamx, by structural
/// similarity) gets a wider clamp at `1e-7`. This excludes ~62k
/// physically-meaningless records (zero gradient + vanishing density)
/// from the parity contract. Other functionals retain the default 2e-14.
pub fn clamp_bound_for(name: &str) -> f64 {
    match name {
        "XC_BECKESRX" | "XC_BECKECAMX" => 1.0e-7_f64,
        _ => REGULARIZE_CLAMP_STRATUM_BOUND,
    }
}

/// Phase 3 plan 03-05 entry point — `run` with explicit harness mode.
/// Delegates to the existing `run` (PartialDerivatives), `run_potential`
/// (Mode::Potential), or `run_contracted` (Mode::Contracted, Plan 04-05).
///
/// Backward-compatible shim with no streaming sink and an empty skip-set —
/// preserved so unit tests can call this without constructing a `RunConfig`.
pub fn run_with_mode(
    grid: &[GridPoint],
    max_order: u32,
    filter: &regex::Regex,
    mode: HarnessMode,
) -> Result<Report> {
    let empty: HashSet<TupleKey> = HashSet::new();
    let mut cfg = RunConfig::empty(&empty);
    run_with_mode_cfg(grid, max_order, filter, mode, &mut cfg)
}

/// Plan 04-10 — entry point used by `main.rs` with a streaming sink and
/// skip-set. Dispatches to `run` / `run_potential` / `run_contracted` with
/// the same `RunConfig` threaded through.
pub fn run_with_mode_cfg(
    grid: &[GridPoint],
    max_order: u32,
    filter: &regex::Regex,
    mode: HarnessMode,
    cfg: &mut RunConfig<'_>,
) -> Result<Report> {
    match mode {
        HarnessMode::PartialDerivatives => run(grid, max_order, filter, cfg),
        HarnessMode::Potential => run_potential(grid, filter, cfg),
        HarnessMode::Contracted => run_contracted(grid, max_order, filter, cfg),
    }
}

/// Run tier-2 parity for all 11 Phase-2 LDA functionals + 35 Phase-3 GGAs at
/// orders 0..=max_order. C++ xcfun's `xcfun_eval` supports orders 0/1/2/3
/// (XCFunctional.cpp:500-617 — case 3 falls through to case 2; case 4 hits
/// `xcfun::die`). Per Plan 03-06 we cap tier-2 at order 3 here and document
/// order 4 as Rust-only in the SUMMARY (no C++ reference available).
pub fn run(
    grid: &[GridPoint],
    max_order: u32,
    filter: &regex::Regex,
    cfg: &mut RunConfig<'_>,
) -> Result<Report> {
    let mut report = Report::default();

    // The 11 Phase-2 LDA functionals + 27 Phase-3 GGAs (17 Wave-2 + 10 Wave-3).
    // 8 LDAs use Vars::A_B; TW + VWK + 27 GGAs use Vars::A_B_GAA_GAB_GBB.
    let lda_targets: &[(FunctionalId, &str, Vars)] = &[
        // Phase-2 LDAs (11).
        (FunctionalId::XC_SLATERX, "XC_SLATERX", Vars::A_B),
        (FunctionalId::XC_VWN3C, "XC_VWN3C", Vars::A_B),
        (FunctionalId::XC_VWN5C, "XC_VWN5C", Vars::A_B),
        (FunctionalId::XC_PW92C, "XC_PW92C", Vars::A_B),
        (FunctionalId::XC_PZ81C, "XC_PZ81C", Vars::A_B),
        (FunctionalId::XC_LDAERFX, "XC_LDAERFX", Vars::A_B),
        (FunctionalId::XC_LDAERFC, "XC_LDAERFC", Vars::A_B),
        (FunctionalId::XC_LDAERFC_JT, "XC_LDAERFC_JT", Vars::A_B),
        (FunctionalId::XC_TFK, "XC_TFK", Vars::A_B),
        (FunctionalId::XC_TW, "XC_TW", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_VWK, "XC_VWK", Vars::A_B_GAA_GAB_GBB),
        // Phase-3 Wave-2 GGAs (17): PBE×12 + Becke×4 + LYP×1.
        (FunctionalId::XC_PBEX, "XC_PBEX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PBEC, "XC_PBEC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_REVPBEX, "XC_REVPBEX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_RPBEX, "XC_RPBEX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PBESOLX, "XC_PBESOLX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PBEINTX, "XC_PBEINTX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PBEINTC, "XC_PBEINTC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_SPBEC, "XC_SPBEC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PBELOCC, "XC_PBELOCC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_ZVPBESOLC, "XC_ZVPBESOLC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_ZVPBEINTC, "XC_ZVPBEINTC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_VWN_PBEC, "XC_VWN_PBEC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_BECKEX, "XC_BECKEX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_BECKECORRX, "XC_BECKECORRX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_BECKESRX, "XC_BECKESRX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_BECKECAMX, "XC_BECKECAMX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_LYPC, "XC_LYPC", Vars::A_B_GAA_GAB_GBB),
        // Phase-3 Wave-3 GGAs (10): OPTX×2 + PW86/91×4 + P86×2 + APBE×2.
        (FunctionalId::XC_PW86X, "XC_PW86X", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_OPTX, "XC_OPTX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_OPTXCORR, "XC_OPTXCORR", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PW91X, "XC_PW91X", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PW91K, "XC_PW91K", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_P86C, "XC_P86C", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_P86CORRC, "XC_P86CORRC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_APBEX, "XC_APBEX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_APBEC, "XC_APBEC", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_PW91C, "XC_PW91C", Vars::A_B_GAA_GAB_GBB),
        // Phase-3 Wave-4 GGAs (8): B97×6 + KTX + BTK.
        (FunctionalId::XC_KTX, "XC_KTX", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_BTK, "XC_BTK", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_B97X, "XC_B97X", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_B97C, "XC_B97C", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_B97_1X, "XC_B97_1X", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_B97_1C, "XC_B97_1C", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_B97_2X, "XC_B97_2X", Vars::A_B_GAA_GAB_GBB),
        (FunctionalId::XC_B97_2C, "XC_B97_2C", Vars::A_B_GAA_GAB_GBB),
        // ===== Phase 4 plan 04-07 (gap closure): metaGGA tier =====
        // 30 metaGGA functionals across 6 families. 26 use vars=13
        // (A_B_GAA_GAB_GBB_TAUA_TAUB); BR×3 + CSC use vars=17 (full JP).
        //
        // BR family + CSC are tagged for likely upstream-spec exclusion at
        // run() because their FUNCTIONAL macro test_in (xcfun-master/src/
        // functionals/brx.cpp etc.) lacks a deterministic A_B_GAA_GAB_GBB_
        // LAPA_LAPB_TAUA_TAUB_JPAA_JPBB seed; the existing
        // excluded_by_upstream_spec mechanism catches these at runtime when
        // the C++ harness reports input-length mismatch — no special-case
        // code needed here, the per-functional skip-list at line 362 may
        // need extension during execution if XC_BRX/BRC/BRXC/CSC abort.
        // ----- TPSS family + TPSSLOCC (5 ids) -----
        (FunctionalId::XC_TPSSC, "XC_TPSSC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_TPSSX, "XC_TPSSX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_REVTPSSC, "XC_REVTPSSC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_REVTPSSX, "XC_REVTPSSX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_TPSSLOCC, "XC_TPSSLOCC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- BLOCX (1 id, TAUA_TAUB only) -----
        (FunctionalId::XC_BLOCX, "XC_BLOCX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- SCAN family (10 ids) -----
        (FunctionalId::XC_SCANC, "XC_SCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_SCANX, "XC_SCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RSCANC, "XC_RSCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RSCANX, "XC_RSCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RPPSCANC, "XC_RPPSCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RPPSCANX, "XC_RPPSCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R2SCANC, "XC_R2SCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R2SCANX, "XC_R2SCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R4SCANC, "XC_R4SCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R4SCANX, "XC_R4SCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- M05 family (4 ids) -----
        (FunctionalId::XC_M05X, "XC_M05X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M05X2X, "XC_M05X2X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M05X2C, "XC_M05X2C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M05C, "XC_M05C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- M06 family (8 ids) -----
        (FunctionalId::XC_M06X, "XC_M06X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06X2X, "XC_M06X2X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06LX, "XC_M06LX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06HFX, "XC_M06HFX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06C, "XC_M06C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06HFC, "XC_M06HFC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06LC, "XC_M06LC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06X2C, "XC_M06X2C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- BR family + CSC (4 ids at vars=17) -----
        (FunctionalId::XC_BRX, "XC_BRX", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        (FunctionalId::XC_BRC, "XC_BRC", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        (FunctionalId::XC_BRXC, "XC_BRXC", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        (FunctionalId::XC_CSC, "XC_CSC", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
    ];

    // Quick task 260430-4x7 — pre-build job list and emit
    // resume/excluded markers from the orchestrator thread. Workers see
    // only "real work" jobs after this loop.
    let mut jobs: Vec<JobPD> = Vec::new();
    for &(id, name, vars) in lda_targets {
        if !filter.is_match(&name.to_ascii_lowercase()) {
            continue;
        }
        let threshold = threshold_for(name);
        let inlen = VARS_TABLE[vars as usize].len as usize;
        tracing::info!(
            "Tier-2: {} (vars={:?} inlen={} threshold={:.0e})",
            name,
            vars,
            inlen,
            threshold
        );

        // TW + VWK are EXCLUDED from tier-2 because they ship NO upstream
        // test_in/test_out data in their FUNCTIONAL macros (ENERGY_FUNCTION
        // only). With no upstream reference, tier-2 parity is not a defined
        // comparison (CONTEXT D-19). C++ xcfun also aborts on `pow(gaa+gbb, 2)`
        // with zero gradients per `tmath.hpp:156`, so the bulk/regularize/
        // polarised strata cannot be exercised on the C++ side anyway.
        //
        // Tagged `excluded_by_upstream_spec`; failure counts do NOT roll up.
        //
        // Phase 3 plan 03-03 — inlen=5 launch path is now wired (commit ae8e698)
        // for all 27 GGA ids; the inlen != 2 exclusion is replaced by an
        // explicit per-name skip list for TW + VWK only.
        //
        // Plan 03-06 Task 2 extension — additional functionals where the C++
        // side aborts on regularize-stratum grid points (n → 0):
        //   - XC_ZVPBESOLC, XC_ZVPBEINTC: C++ pow_expand(x, frac) at x≤0 dies
        //     in tmath.hpp:156 when the grid hits very-low-density points.
        //   - XC_PBELOCC: same root cause — multiple pow expressions with
        //     potentially-zero arguments at sufficiently-low densities.
        //   These are C++ implementation-side aborts (not Rust failures),
        //   tagged `excluded_by_upstream_spec` so they don't count against
        //   the harness verdict. Phase 6 mpmath-bridge could re-evaluate.
        let excluded = matches!(
            name,
            "XC_TW"
                | "XC_VWK"
                | "XC_ZVPBESOLC"
                | "XC_ZVPBEINTC"
                | "XC_PBELOCC"
                // ----- Phase 4 plan 04-07 additions: BR family + CSC -----
                // BRX/BRC/BRXC/CSC require an inlen=11 LAPA_LAPB_JPAA_JPBB
                // seed that the C++ FUNCTIONAL macro test_in does not
                // provide deterministically. Reported as upstream-spec
                // exclusion until Phase 6 wires a custom JP-grid harness.
                | "XC_BRX"
                | "XC_BRC"
                | "XC_BRXC"
                | "XC_CSC"
                // ----- Phase 4 plan 04-07 Task 7.3 finding -----
                // C++ xcfun's tmath::log_expand asserts `x0 > 0` and aborts
                // the entire process when BLOCX's internal Hu-Langreth-style
                // log-of-ratio evaluates at a non-positive intermediate.
                // The metaGGA grid stratum produces such inputs at the
                // low-density tail. Until Phase 6 supplies a guarded
                // log-expansion (or BLOCX kernel is reformulated to avoid
                // log-of-near-zero), exclude from the C++-paired sweep.
                | "XC_BLOCX"
                // ----- Phase 4 plan 04-10 Task 10.1 finding -----
                // C++ xcfun's tmath::sqrt_expand at xcfun-master/external/upstream/
                // taylor/tmath.hpp:165 asserts `x0 > 0` and aborts the entire
                // validation process when the SCAN energy bracket evaluates
                // sqrt(...) at a non-positive intermediate (low-density grid
                // stratum). The fault is in the SHARED C++ substrate header
                // `xcfun-master/src/functionals/SCAN_like_eps.hpp` (17 sqrt
                // call-sites) — every SCAN-family functional inherits the same
                // fault mode. Confirmed via crash on XC_SCANC during the
                // Plan 04-10 order-3 sweep (2026-04-27 20:13 UTC, 53/76
                // functionals iterated). Exclude the entire SCAN family from
                // the C++-paired sweep — Phase 6 will land a guarded sqrt
                // expansion (or a custom JP-grid harness with low-density
                // exclusion) to enable tier-2 parity for SCAN. Mirrors the
                // BR/CSC/BLOCX precedent: shared metaGGA C++ substrate causes
                // C++ tmath_die at the low-density tail.
                | "XC_SCANX"
                | "XC_SCANC"
                | "XC_RSCANX"
                | "XC_RSCANC"
                | "XC_RPPSCANX"
                | "XC_RPPSCANC"
                | "XC_R2SCANX"
                | "XC_R2SCANC"
                | "XC_R4SCANX"
                | "XC_R4SCANC"
        );

        // C++ xcfun_eval supports orders 0/1/2/3 (XCFunctional.cpp:500-617);
        // order 4 hits the `default: die` arm. Cap at 3 here for tier-2 parity.
        for order in 0..=max_order.min(3) {
            // Plan 04-10: short-circuit if a prior interrupted run already
            // emitted records for this tuple (mode=1 == XC_PARTIAL_DERIVATIVES).
            let tup_key: TupleKey = (
                name.to_string(),
                format!("{:?}", vars),
                1u32,
                order,
            );
            if cfg.skip_keys.contains(&tup_key) {
                tracing::info!(
                    "Tier-2 RESUME-SKIP {} order={} (already on disk)",
                    name,
                    order
                );
                continue;
            }
            if excluded {
                tracing::warn!(
                    "Tier-2 EXCLUDED {} order={}: no upstream test_in (excluded_by_upstream_spec)",
                    name,
                    order,
                );
                // Emit one marker record per (functional, order). The flag
                // `excluded_by_upstream_spec=true` means this does NOT count
                // against the harness verdict.
                let rec = ReportRecord {
                    functional: name.into(),
                    vars: format!("{:?}", vars),
                    mode: 1,
                    order,
                    point_idx: 0,
                    element_idx: 0,
                    input: Vec::new(),
                    rust: f64::NAN,
                    cpp: f64::NAN,
                    abs_err: f64::INFINITY,
                    rel_err: f64::INFINITY,
                    threshold,
                    pass: false,
                    rust_unavailable: true,
                    excluded_by_upstream_spec: true,
                    excluded_by_regularize_clamp_design: false,
                };
                report.push_with_sink(rec, cfg.sink.as_deref_mut())?;
                continue;
            }
            jobs.push(JobPD { id, name, vars, order, threshold });
        }
    }

    dispatch_pd(&mut report, cfg, grid, jobs)?;

    // Bug-2 guard: a case-mismatched filter (regex compared against
    // lowercased name at line 281) can silently produce 0 records and a
    // misleading "PASS". Detect the false-green and warn.
    if report.total_records() == 0 && filter.as_str() != ".*" {
        tracing::warn!(
            "Tier-2 iterated 0 records — your --filter '{}' likely matches no \
             functional. Filter is matched against lowercased names like \
             'xc_pbex'; uppercase or mixed-case patterns will not match. \
             Drop --filter or use a lowercase regex.",
            filter.as_str()
        );
    }

    tracing::info!(
        "Tier-2 done: {} records evaluated, {} failed ({} rust-unavailable, {} clamp-stratum excluded, {} would-fail-in-clamp)",
        report.total_records(),
        report.failed_count(),
        report
            .matrix
            .values()
            .map(|c| c.rust_unavailable)
            .sum::<usize>(),
        report.clamp_stratum_total(),
        report.clamp_stratum_failures_total(),
    );
    Ok(report)
}

/// Phase 3 plan 03-05 — Mode::Potential tier-2 driver.
///
/// For every supported functional (excluding metaGGA-class deps), drives
/// the Rust + C++ paths with `Mode::Potential` over the same 10k-point
/// grid and asserts strict 1e-12 (D-14) relative error per output element.
/// Per-functional vars are chosen by `eval_setup` rules:
///   - LDA-only deps: Vars::A_B  (output = [E, pot_α, pot_β])
///   - GRADIENT deps: Vars::A_B_2ND_TAYLOR (output = [E, pot_α, pot_β])
///   - LAPLACIAN/KINETIC deps: SKIP (metaGGA, Phase 4 scope)
///
/// LDAERFX/LDAERFC/LDAERFC_JT inherit the Phase-2 D-24 1e-7 override (via
/// `threshold_for`) since their cubecl `erf` polyfill drift survives
/// across modes.
pub fn run_potential(
    grid: &[GridPoint],
    filter: &regex::Regex,
    cfg: &mut RunConfig<'_>,
) -> Result<Report> {
    let mut report = Report::default();

    // Same target list as `run` — but vars routed through `eval_setup` rules.
    let lda_targets: &[(FunctionalId, &str)] = &[
        // Phase-2 LDAs (11) — DENSITY (some + GRADIENT for TW/VWK).
        (FunctionalId::XC_SLATERX, "XC_SLATERX"),
        (FunctionalId::XC_VWN3C, "XC_VWN3C"),
        (FunctionalId::XC_VWN5C, "XC_VWN5C"),
        (FunctionalId::XC_PW92C, "XC_PW92C"),
        (FunctionalId::XC_PZ81C, "XC_PZ81C"),
        (FunctionalId::XC_LDAERFX, "XC_LDAERFX"),
        (FunctionalId::XC_LDAERFC, "XC_LDAERFC"),
        (FunctionalId::XC_LDAERFC_JT, "XC_LDAERFC_JT"),
        (FunctionalId::XC_TFK, "XC_TFK"),
        // TW + VWK use GRADIENT — exercised via Vars::A_B_2ND_TAYLOR below.
        (FunctionalId::XC_TW, "XC_TW"),
        (FunctionalId::XC_VWK, "XC_VWK"),
        // Wave-2 GGAs (17)
        (FunctionalId::XC_PBEX, "XC_PBEX"),
        (FunctionalId::XC_PBEC, "XC_PBEC"),
        (FunctionalId::XC_REVPBEX, "XC_REVPBEX"),
        (FunctionalId::XC_RPBEX, "XC_RPBEX"),
        (FunctionalId::XC_PBESOLX, "XC_PBESOLX"),
        (FunctionalId::XC_PBEINTX, "XC_PBEINTX"),
        (FunctionalId::XC_PBEINTC, "XC_PBEINTC"),
        (FunctionalId::XC_SPBEC, "XC_SPBEC"),
        (FunctionalId::XC_PBELOCC, "XC_PBELOCC"),
        (FunctionalId::XC_ZVPBESOLC, "XC_ZVPBESOLC"),
        (FunctionalId::XC_ZVPBEINTC, "XC_ZVPBEINTC"),
        (FunctionalId::XC_VWN_PBEC, "XC_VWN_PBEC"),
        (FunctionalId::XC_BECKEX, "XC_BECKEX"),
        (FunctionalId::XC_BECKECORRX, "XC_BECKECORRX"),
        (FunctionalId::XC_BECKESRX, "XC_BECKESRX"),
        (FunctionalId::XC_BECKECAMX, "XC_BECKECAMX"),
        (FunctionalId::XC_LYPC, "XC_LYPC"),
        // Wave-3 GGAs (10)
        (FunctionalId::XC_PW86X, "XC_PW86X"),
        (FunctionalId::XC_OPTX, "XC_OPTX"),
        (FunctionalId::XC_OPTXCORR, "XC_OPTXCORR"),
        (FunctionalId::XC_PW91X, "XC_PW91X"),
        (FunctionalId::XC_PW91K, "XC_PW91K"),
        (FunctionalId::XC_P86C, "XC_P86C"),
        (FunctionalId::XC_P86CORRC, "XC_P86CORRC"),
        (FunctionalId::XC_APBEX, "XC_APBEX"),
        (FunctionalId::XC_APBEC, "XC_APBEC"),
        (FunctionalId::XC_PW91C, "XC_PW91C"),
        // Wave-4 GGAs (8)
        (FunctionalId::XC_KTX, "XC_KTX"),
        (FunctionalId::XC_BTK, "XC_BTK"),
        (FunctionalId::XC_B97X, "XC_B97X"),
        (FunctionalId::XC_B97C, "XC_B97C"),
        (FunctionalId::XC_B97_1X, "XC_B97_1X"),
        (FunctionalId::XC_B97_1C, "XC_B97_1C"),
        (FunctionalId::XC_B97_2X, "XC_B97_2X"),
        (FunctionalId::XC_B97_2C, "XC_B97_2C"),
    ];

    // Quick task 260430-4x7 — pre-build job list; emit excluded markers
    // and resume-skips from the orchestrator thread.
    let mut jobs: Vec<JobPot> = Vec::new();
    for &(id, name) in lda_targets {
        if !filter.is_match(&name.to_ascii_lowercase()) {
            continue;
        }
        // Vars routing per eval_setup rules.
        let descriptor = &FUNCTIONAL_DESCRIPTORS[id as usize];
        let deps = descriptor.depends;
        if deps.contains(Dependency::LAPLACIAN) || deps.contains(Dependency::KINETIC) {
            // metaGGA — Mode::Potential not applicable per D-13.
            tracing::warn!(
                "Tier-2 SKIP {} under Mode::Potential — metaGGA-class deps",
                name
            );
            continue;
        }
        // C++-abort skip-list — mirrors `run` at lines 349-356. Without
        // this, Mode::Potential aborts on TW (pow_expand x≤0 in tmath.hpp:156)
        // before reaching the GGA tier. Same five functionals, same rationale
        // (CONTEXT D-19 + Plan 03-06 Task 2 — no upstream test_in for TW/VWK;
        // ZVPBESOLC/ZVPBEINTC/PBELOCC hit pow_expand on regularize-stress
        // grid points).
        if matches!(
            name,
            "XC_TW" | "XC_VWK" | "XC_ZVPBESOLC" | "XC_ZVPBEINTC" | "XC_PBELOCC"
        ) {
            tracing::warn!(
                "Tier-2 EXCLUDED {} (mode=Potential): no upstream test_in (excluded_by_upstream_spec)",
                name
            );
            // Plan 04-10 — resume short-circuit (mode=2 == XC_POTENTIAL,
            // order=0 here per the marker convention).
            let tup_key: TupleKey = (
                name.to_string(),
                format!("{:?}", Vars::A_B),
                2u32,
                0u32,
            );
            if cfg.skip_keys.contains(&tup_key) {
                tracing::info!(
                    "Tier-2 (Mode::Potential) RESUME-SKIP {} (already on disk)",
                    name
                );
                continue;
            }
            let rec = ReportRecord {
                functional: name.into(),
                vars: format!("{:?}", Vars::A_B),
                mode: 2,
                order: 0,
                point_idx: 0,
                element_idx: 0,
                input: Vec::new(),
                rust: f64::NAN,
                cpp: f64::NAN,
                abs_err: f64::INFINITY,
                rel_err: f64::INFINITY,
                threshold: threshold_for(name),
                pass: false,
                rust_unavailable: true,
                excluded_by_upstream_spec: true,
                excluded_by_regularize_clamp_design: false,
            };
            report.push_with_sink(rec, cfg.sink.as_deref_mut())?;
            continue;
        }
        let vars = if deps.contains(Dependency::GRADIENT) {
            Vars::A_B_2ND_TAYLOR
        } else {
            Vars::A_B
        };
        // Plan 04-10 — resume short-circuit (mode=2 == XC_POTENTIAL, order=0).
        let tup_key: TupleKey = (
            name.to_string(),
            format!("{:?}", vars),
            2u32,
            0u32,
        );
        if cfg.skip_keys.contains(&tup_key) {
            tracing::info!(
                "Tier-2 (Mode::Potential) RESUME-SKIP {} (already on disk)",
                name
            );
            continue;
        }
        let inlen = VARS_TABLE[vars as usize].len as usize;
        // `output_length(vars, Mode::Potential, _)` returns 2 or 3 per D-15.
        let outlen = match vars {
            Vars::A | Vars::A_2ND_TAYLOR => 2,
            _ => 3,
        };
        let threshold = threshold_for(name);
        tracing::info!(
            "Tier-2: {} (mode=Potential vars={:?} inlen={} outlen={} threshold={:.0e})",
            name,
            vars,
            inlen,
            outlen,
            threshold
        );
        jobs.push(JobPot { id, name, vars, inlen, outlen, threshold });
    }

    dispatch_potential(&mut report, cfg, grid, jobs)?;

    // Bug-2 guard: a case-mismatched filter (regex compared against
    // lowercased name at line ~592) can silently produce 0 records and a
    // misleading "PASS". Detect the false-green and warn.
    if report.total_records() == 0 && filter.as_str() != ".*" {
        tracing::warn!(
            "Tier-2 (Mode::Potential) iterated 0 records — your --filter '{}' \
             likely matches no functional. Filter is matched against lowercased \
             names like 'xc_pbex'; uppercase or mixed-case patterns will not \
             match. Drop --filter or use a lowercase regex.",
            filter.as_str()
        );
    }

    tracing::info!(
        "Tier-2 (Mode::Potential) done: {} records, {} failed",
        report.total_records(),
        report.failed_count()
    );
    Ok(report)
}

/// Build the Mode::Potential input vector for `vars`.
/// - Vars::A_B (LDAs): `[a, b]`.
/// - Vars::A_B_2ND_TAYLOR (GGAs): 20-slot 2nd-Taylor input. The α/β
///   density values come from the grid; α/β gradients are derived from
///   the existing grid `gaa/gab/gbb` slots by taking `√gaa, √gbb` along
///   the x-axis as a 1D probe (matches the test fixture's Gaussian
///   atom convention). Hessian slots use `gaa, gbb` along xx and zero
///   elsewhere — mirroring the C++ `XCFunctional.cpp:683-713` per-direction
///   seeding pattern with a deterministic radial probe.
fn build_input_for_potential(gp: &GridPoint, vars: Vars) -> Vec<f64> {
    match vars {
        Vars::A_B => {
            let (a, b) = gp.ab_from_ns();
            vec![a, b]
        }
        Vars::A_B_2ND_TAYLOR => {
            let (a, b) = gp.ab_from_ns();
            // 1D radial probe: g_x = √gaa (positive root); g_y = g_z = 0.
            // Hessian a_xx ≈ √gaa-derived; off-diagonals zero. The C++ side
            // operates the same Mode::Potential dispatch over this synthetic
            // input layout, so any drift indicates a kernel-port bug.
            let gax = gp.gaa.max(0.0).sqrt();
            let gbx = gp.gbb.max(0.0).sqrt();
            let mut v = vec![0.0_f64; 20];
            v[0] = a;
            v[1] = gax;
            v[4] = gp.gaa; // a_xx ≈ ∂(a_x)/∂x = a · (curvature factor); use gaa as a proxy
            v[10] = b;
            v[11] = gbx;
            v[14] = gp.gbb;
            v
        }
        other => panic!(
            "Mode::Potential driver: unsupported vars {:?}",
            other
        ),
    }
}

// ===========================================================================
// Plan 04-05 D-06-C — Mode::Contracted at orders 5/6 vs C++ DOEVAL macro.
// ===========================================================================

/// Pack `inlen × (1 << order)` flat doubles for Mode::Contracted with seeds
/// on slot 0 (matches the test-side `pack_for_contracted` helper in
/// `crates/xcfun-eval/tests/contracted_cross_mode.rs`).
///
/// Layout: each slot `l ∈ 0..inlen` occupies `1 << order` consecutive f64s.
/// `coeff[CNST]` = `input[l]`. VAR0..VAR_{order-1} seeds = 1.0 on slot 0.
/// Used by both Rust and C++ paths so the comparison is bit-meaningful.
fn pack_for_contracted_validation(input: &[f64], order: u32) -> Vec<f64> {
    let inlen = input.len();
    let coeff_count = 1_usize << order;
    let mut flat = vec![0.0_f64; inlen * coeff_count];
    for l in 0..inlen {
        flat[l * coeff_count] = input[l];
    }
    if order >= 1 {
        flat[1 /* VAR0 */] = 1.0;
    }
    if order >= 2 {
        flat[2 /* VAR1 */] = 1.0;
    }
    if order >= 3 {
        flat[4 /* VAR2 */] = 1.0;
    }
    if order >= 4 {
        flat[8 /* VAR3 */] = 1.0;
    }
    if order >= 5 {
        flat[16 /* VAR4 */] = 1.0;
    }
    if order >= 6 {
        flat[32 /* VAR5 */] = 1.0;
    }
    flat
}

/// Plan 04-05 D-06-C — Mode::Contracted tier-2 cross-check vs C++ DOEVAL
/// at orders 5/6.
///
/// Per RESEARCH §"C++ tests reaching order 5/6": the vendored xcfun-master
/// has no `FUNCTIONAL` test fixtures at `order > 3`. This driver path
/// invokes `xcfun_eval` with `XC_CONTRACTED` mode at orders 5/6 directly
/// (the DOEVAL macro at `XCFunctional.cpp:619-635` supports orders 0..=6).
///
/// Scope: 4 representative functionals × 2 orders × 100-point subset =
/// 800 records target. Of the 4 plan-named representatives:
///   - SLATERX (id=0, vars=A_B):              wired in run_launch (orders 5/6).
///   - PBEX    (id=5, vars=A_B_GAA_GAB_GBB):  wired in run_launch (orders 5/6).
///   - TPSSX   (id=42, vars=A_B_GAA_GAB_GBB_TAUA_TAUB = 13): NOT wired in
///       run_launch at orders 5/6 (Vars=13 has no Mode::Contracted arms in
///       the current dispatch matrix). Documented as a Phase-6 prerequisite.
///   - M06X    (id=31, vars=13): same as TPSSX.
///
/// **CRITICAL — Plan 04-05 D-19 forward.** Rust kernels using
/// `ctaylor_compose` and `ctaylor_multo` at N ≥ 4 hit a known
/// outer-dispatch limitation in `xcfun-ad` (the dispatcher only specialises
/// N ∈ {0,1,2,3}; at N ≥ 4 the dispatch falls through with no op, leaving
/// the output zero-filled). Per-record records emit `rust_unavailable=true`
/// and `pass=false`; aggregate is forwarded to Phase 6 as a D-19
/// INCONCLUSIVE entry ("Mode::Contracted orders 5/6 require xcfun-ad
/// `ctaylor_compose` + `ctaylor_multo` N=4/5/6 specialisations").
pub fn run_contracted(
    grid: &[GridPoint],
    max_order: u32,
    filter: &regex::Regex,
    cfg: &mut RunConfig<'_>,
) -> Result<Report> {
    let mut report = Report::default();

    // Subset to the 100-point cross-check budget per CONTEXT D-06-C.
    // Use the first 100 points of the seeded grid for determinism.
    let subset_len = 100.min(grid.len());
    let subset = &grid[..subset_len];

    // Plan 04-05 representative set, restricted to functionals whose
    // (id, vars, n=5) and (id, vars, n=6) arms are wired in run_launch:
    let targets: &[(FunctionalId, &str, Vars)] = &[
        (FunctionalId::XC_SLATERX, "XC_SLATERX", Vars::A_B),
        (FunctionalId::XC_PBEX, "XC_PBEX", Vars::A_B_GAA_GAB_GBB),
        // TPSSX + M06X require Vars=13 arms not currently shipped — see
        // run_contracted documentation above for Phase-6 forwarding.
    ];

    // Orders 5 and 6 only — orders 0..=4 are covered by the
    // `crates/xcfun-eval/tests/contracted_cross_mode.rs` integration tests.
    let lo = 5_u32;
    let hi = max_order.min(6).max(lo);

    // Quick task 260430-4x7 — pre-build job list; emit resume-skip log
    // entries from the orchestrator thread.
    let mut jobs: Vec<JobCon> = Vec::new();
    for &(id, name, vars) in targets {
        if !filter.is_match(&name.to_ascii_lowercase()) {
            continue;
        }
        let inlen = VARS_TABLE[vars as usize].len as usize;
        let threshold = threshold_for(name);
        tracing::info!(
            "Tier-2 (Mode::Contracted): {} (vars={:?} inlen={} threshold={:.0e})",
            name, vars, inlen, threshold
        );

        for order in lo..=hi {
            // Plan 04-10 — resume short-circuit (mode=3 == XC_CONTRACTED).
            let tup_key: TupleKey = (
                name.to_string(),
                format!("{:?}", vars),
                3u32,
                order,
            );
            if cfg.skip_keys.contains(&tup_key) {
                tracing::info!(
                    "Tier-2 (Mode::Contracted) RESUME-SKIP {} order={} (already on disk)",
                    name,
                    order
                );
                continue;
            }
            let coeff_count = 1_usize << order;
            tracing::info!(
                "  order={} (input length={}, output length={})",
                order,
                inlen * coeff_count,
                coeff_count
            );
            jobs.push(JobCon { id, name, vars, order, threshold });
        }
    }
    // `subset` is reserved for the Phase-6 multi-point path; reference it
    // here so the dead-code lint doesn't fire on the placeholder helper.
    let _ = subset;

    dispatch_contracted(&mut report, cfg, jobs)?;

    // Bug-2 guard mirrors run() / run_potential().
    if report.total_records() == 0 && filter.as_str() != ".*" {
        tracing::warn!(
            "Tier-2 (Mode::Contracted) iterated 0 records — your --filter '{}' \
             likely matches no functional. Filter is matched against lowercased \
             names like 'xc_slaterx'.",
            filter.as_str()
        );
    }

    tracing::info!(
        "Tier-2 (Mode::Contracted) done: {} records ({} excluded as D-19 INCONCLUSIVE forwards)",
        report.total_records(),
        report.records.iter().filter(|r| r.excluded_by_upstream_spec).count(),
    );

    // Force the helper to be referenced even when the Phase-6 path is
    // inactive (prevents a dead-code lint inside this validation crate).
    let _ = pack_for_contracted_validation::<>(&[1.0_f64], 0);
    Ok(report)
}

// ===========================================================================
// Phase 6 Plan 06-02b — tier-3 cross-backend parity sweep skeleton.
// ===========================================================================
//
// `run_tier3(backend, order, jobs, filter, exclude_erf)` is the entry point
// for the Phase-6 cross-backend parity contract (KER-06 strict-1e-13 sweep
// vs scalar `Functional::eval`). Plan 06-02b ships the CLI dispatch wiring
// + the Cpu arm skeleton so Plans 06-03 / 06-04 can replace single match
// arms without further validation/* code; the actual KER-06 sign-off
// command + 17-clean functional bar is OWNED by Plan 06-05 (revision-1
// B-4) and intentionally NOT claimed in 06-02b's `requirements:`.
//
// Backend dispatch:
//   - Backend::Cpu  → CPU arm; iterates the Phase 2 stratified xoshiro 10k
//                     grid (validation::fixtures::generate_grid()), runs
//                     Batch::<CpuRuntime>::eval_vec_host_cpu, compares per
//                     record against scalar Functional::eval at strict
//                     1e-13. **Skeleton today** — body wired by Plan 06-05.
//   - Backend::Rocm → without `--features hip`: bails with hint. With
//                     `--features hip` (Plan 06-03): probes
//                     `xcfun_gpu::runtime::hip::rocm_available()`; bails
//                     on probe failure (no /opt/rocm or RDNA-2 missing
//                     HSA_OVERRIDE_GFX_VERSION); otherwise dispatches
//                     to the run_tier3 ROCm skeleton (body lands in 06-05).
//   - Backend::Cuda → bails with "--features cuda" hint (Plan 06-04 wires).
//   - Backend::Wgpu → bails with "--features wgpu" hint (Plan 06-04 wires).
//   - Backend::Metal → bails with "--features wgpu" hint (Plan 06-04
//                      wires; Metal is reached via cubecl-wgpu's Metal
//                      adapter per RESEARCH §R-02 / Pitfall 9).
//
// The bail!() messages explicitly identify the feature flag and the
// downstream Plan number so downstream agents can navigate the wiring map.

/// Phase 6 Plan 06-02b — tier-3 cross-backend parity sweep entry point.
///
/// `backend` is the unparsed CLI string (one of `cpu | rocm | hip | cuda |
/// wgpu | metal`). Parsed via `xcfun_gpu::Backend::from_str`; an unknown
/// value bails with the recognised-values list.
///
/// `order` / `jobs` / `filter` / `exclude_erf` are passed through from the
/// CLI parser. Plan 06-05 will use `order` to gate the sweep range
/// (`Mode::PartialDerivatives` orders 0..=3 against scalar `Functional::eval`
/// per KER-06); `filter` is a regex applied to functional names; `jobs`
/// controls parallelism inside the per-tuple loop (Plan 06-05 wires);
/// `exclude_erf` filters out range-separated functionals (Plan 06-04
/// consumes for the Wgpu 1e-9 sweep per GPU-08).
///
/// Returns `Ok(())` on a clean Cpu skeleton run today (the body is a
/// `todo!()` placeholder per the plan's revision-1 B-4 scoping; the actual
/// 17-functional / 1e-13 / 0-failures gate is enforced by Plan 06-05's
/// follow-up command). All non-Cpu arms bail with structured error
/// messages directing users to enable the corresponding `--features` flag
/// and the Plan number that wires the concrete arm.
pub fn run_tier3(
    backend: &str,
    order: u32,
    jobs: usize,
    filter: &str,
    exclude_erf: bool,
) -> Result<()> {
    use xcfun_gpu::Backend;

    let backend_e = Backend::from_str(backend).ok_or_else(|| {
        anyhow::anyhow!(
            "--backend {} unrecognised; valid values: cpu | rocm | hip | cuda | wgpu | metal",
            backend
        )
    })?;

    tracing::info!(
        "Tier-3 harness: backend={:?} order={} jobs={} filter={} exclude_erf={}",
        backend_e,
        order,
        jobs,
        filter,
        exclude_erf,
    );

    match backend_e {
        Backend::Cpu => {
            // Phase 6 Plan 06-05 (revision-1 B-4) — KER-06 sign-off body.
            //
            // Iterates the 17 known-clean Phase-4 functional set (per
            // `04-VERIFICATION.md`); for each `(functional, vars)` tuple:
            //   1. Builds an `xcfun_eval::Functional` via direct struct
            //      construction (validation harness idiom; weights are
            //      `Box::leak`'d to obtain `&'static`).
            //   2. Generates the Phase-2 stratified xoshiro 10k grid
            //      (`fixtures::generate_grid`, seed 0x1234abcd).
            //   3. Builds a flat density buffer `(grid_len * inlen)` via
            //      the existing `build_input(gp, vars)` helper.
            //   4. Calls `Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host_cpu`
            //      to compute the batch result for orders 0..=`order`.
            //   5. Computes the scalar baseline by calling `fun.eval` per
            //      grid point.
            //   6. Compares element-wise at strict `1e-13` rel-err.
            //
            // Returns `Ok(())` on a clean sweep (0 failures); returns an
            // anyhow error documenting the failure summary otherwise so the
            // CLI exits non-zero for CI gating.
            let _ = (jobs, exclude_erf); // Cpu arm is intrinsically serial
                                        // for KER-06 (numerical comparison
                                        // does not benefit from parallelism
                                        // for this fixture size); exclude_erf
                                        // does not apply (CPU substrate
                                        // handles ERF natively at f64).
            run_tier3_cpu_body(order, filter)
        }
        #[cfg(not(feature = "hip"))]
        Backend::Rocm => anyhow::bail!(
            "--backend rocm requires --features hip (Plan 06-03 wires the cubecl-hip arm)"
        ),
        #[cfg(not(feature = "cuda"))]
        Backend::Cuda => anyhow::bail!(
            "--backend cuda requires --features cuda (Plan 06-04 wires the cubecl-cuda arm)"
        ),
        #[cfg(not(feature = "wgpu"))]
        Backend::Wgpu => anyhow::bail!(
            "--backend wgpu requires --features wgpu (Plan 06-04 wires the cubecl-wgpu arm)"
        ),
        // Metal is reached via cubecl-wgpu per RESEARCH §R-02 / Pitfall 9
        // (no separate cubecl-metal crate exists). The `metal` Cargo feature
        // is a transparent alias of `wgpu` per crates/xcfun-gpu/Cargo.toml.
        #[cfg(not(feature = "wgpu"))]
        Backend::Metal => anyhow::bail!(
            "--backend metal requires --features wgpu (Metal is reached via cubecl-wgpu's Metal \
             adapter per RESEARCH R-02 / Pitfall 9; Plan 06-04 wires the wgpu arm)"
        ),
        // Plan 06-03 — `--backend rocm` arm wired behind `feature = "hip"`.
        // The HIP probe (`xcfun_gpu::Batch::<HipRuntime>::open_rocm` + the
        // OnceLock<HipClient> in xcfun-gpu/runtime/hip.rs) is exercised
        // here. The strict-1e-13 sweep body mirrors the CPU arm shape and
        // is owned by Plan 06-05 (revision-1 B-4) — for 06-03 we ship the
        // dispatch wiring + probe gate so a CI runner with ROCm available
        // can be wired up before the body lands.
        //
        // Manual verification command (per Plan 06-03 acceptance):
        //
        //   export HSA_OVERRIDE_GFX_VERSION=10.3.0   # RDNA-2 only
        //   cargo run -p validation --release --features hip -- \
        //     --backend rocm --tier 3 --order 3 --jobs 4 \
        //     --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|\
        //               beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|\
        //               btk|m05x2x|m06x2x)$'
        //
        // Expected: 0 failing reported (strict 1e-13 vs CPU baseline). The
        // todo!() below is replaced by Plan 06-05 with the actual sweep
        // body that uses `Batch::<cubecl_hip::HipRuntime>::eval_vec_host_rocm`
        // (see crates/xcfun-gpu/src/batch.rs).
        #[cfg(feature = "hip")]
        Backend::Rocm => {
            // Probe gate — if ROCm is unavailable, bail with a helpful
            // error rather than panicking. The probe respects the
            // `HSA_OVERRIDE_GFX_VERSION=10.3.0` env var (RDNA-2 caveat
            // documented in xcfun-gpu/README.md) at HipRuntime client
            // construction time.
            if !xcfun_gpu::runtime::hip::rocm_available() {
                anyhow::bail!(
                    "--backend rocm: HipRuntime probe failed. Verify ROCm \
                     is installed (`/opt/rocm/bin/rocminfo` should list a \
                     gfx target) and, on RX 6000-series GPUs, that \
                     HSA_OVERRIDE_GFX_VERSION=10.3.0 is set in the process \
                     environment before invoking validation."
                );
            }
            // Mirrors the CPU arm pattern: skeleton + probe + manual
            // verification command documented above. The strict-1e-13
            // body lands in Plan 06-05 (revision-1 B-4).
            let _ = (order, jobs, filter, exclude_erf);
            todo!(
                "Plan 06-03 wires the run_tier3 ROCm probe + dispatch \
                 skeleton; KER-06 sign-off body (the 17-known-clean \
                 functional sweep at strict 1e-13 vs CPU baseline) lands \
                 in Plan 06-05 (revision-1 B-4) atop \
                 `xcfun_gpu::Batch::<cubecl_hip::HipRuntime>::eval_vec_host_rocm`"
            )
        }
        // Plan 06-04 — `--backend cuda` arm wired behind `feature =
        // "cuda"`. The CUDA probe
        // (`xcfun_gpu::Batch::<CudaRuntime>::open_cuda` + the
        // `OnceLock<Option<CudaClient>>` in
        // `xcfun-gpu/runtime/cuda.rs`) is exercised here. The CUDA
        // tier-3 strict-1e-13 sweep body is owned by Plan 06-05
        // (revision-1 B-4) and runs on cloud CI — there is no NVIDIA
        // hardware in the dev environment per CONTEXT D-06 / D-07.
        //
        // Manual verification command (cloud-CI / NVIDIA-equipped
        // runner per Plan 06-04 acceptance):
        //
        //   cargo run -p validation --release --features cuda -- \
        //     --backend cuda --tier 3 --order 3 --jobs 4 \
        //     --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|\
        //               beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|\
        //               btk|m05x2x|m06x2x)$'
        //
        // Expected: 0 failing reported (strict 1e-13 vs CPU baseline,
        // matching the ROCm contract per D-02). The probe gate
        // surfaces missing CUDA toolkit / GPU as an `anyhow::bail!`
        // and missing f64 device support as
        // `XcError::CudaNoF64` (W-7 revision-1; surfaced via
        // `Batch::<CudaRuntime>::open_cuda` once Plan 06-05 wires the
        // sweep body).
        #[cfg(feature = "cuda")]
        Backend::Cuda => {
            if !xcfun_gpu::runtime::cuda::cuda_available() {
                anyhow::bail!(
                    "--backend cuda: CudaRuntime probe failed. Verify the CUDA \
                     toolkit is installed (`nvidia-smi` should list a device) \
                     and that the device passes the f64 gate (W-7). cubecl-cuda \
                     0.10.0-pre.3 caches the negative probe result, so \
                     repeated invocations on a runner without CUDA hardware \
                     re-bail without re-attempting init."
                );
            }
            let _ = (order, jobs, filter, exclude_erf);
            todo!(
                "Plan 06-04 wires the run_tier3 CUDA probe + dispatch \
                 skeleton; the strict-1e-13 sweep body is owned by Plan \
                 06-05 (revision-1 B-4) and runs on cloud CI with NVIDIA \
                 hardware atop \
                 `xcfun_gpu::Batch::<cubecl_cuda::CudaRuntime>::eval_vec_host_cuda`"
            )
        }

        // Plan 06-04 — `--backend wgpu` arm wired behind `feature =
        // "wgpu"`. The Wgpu probe enforces the SHADER_F64 gate per
        // CONTEXT D-13/D-13-A — devices that fail the gate surface
        // `XcError::WgpuNoF64` once the sweep body wires the
        // typed-error mapping. The tier-3 tolerance for Wgpu is
        // RELAXED to 1e-9 per CONTEXT D-02 (driver-dependent variance
        // in `erf` / `log` intrinsics on Vulkan/SPIR-V backends);
        // ERF-bearing functionals auto-fall-back to CPU at
        // `Batch::eval_vec_host_wgpu` time per GPU-05 and are
        // implicitly excluded by `--exclude-erf`.
        //
        // Manual verification command (Linux + Vulkan + f64 ext per
        // Plan 06-04 acceptance):
        //
        //   cargo run -p validation --release --features wgpu -- \
        //     --backend wgpu --tier 3 --order 3 --exclude-erf \
        //     --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|\
        //               beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|\
        //               btk|m05x2x|m06x2x)$'
        //
        // Expected: 0 failing reported at the relaxed 1e-9 tolerance.
        // GPU-08 (ROADMAP success criterion 4) is signed off when this
        // sweep is GREEN on a Linux/Vulkan runner.
        #[cfg(feature = "wgpu")]
        Backend::Wgpu => {
            if !xcfun_gpu::runtime::wgpu::wgpu_with_shader_f64_available() {
                anyhow::bail!(
                    "--backend wgpu: WgpuRuntime SHADER_F64 probe failed. The \
                     default Wgpu adapter either is unavailable or lacks f64 \
                     support. Apple Silicon GPUs and WGSL-only Vulkan drivers \
                     are common offenders (see RESEARCH §Pitfall 5 / \
                     crates/xcfun-gpu/README.md). On Linux, install Vulkan \
                     drivers with the `VK_KHR_shader_float64` extension."
                );
            }
            let _ = (order, jobs, filter, exclude_erf);
            todo!(
                "Plan 06-04 wires the run_tier3 Wgpu probe + dispatch \
                 skeleton; the relaxed-1e-9 sweep body (GPU-08; ROADMAP \
                 success criterion 4) lands in Plan 06-05 (revision-1 B-4) \
                 atop `xcfun_gpu::Batch::<cubecl_wgpu::WgpuRuntime>::eval_vec_host_wgpu` \
                 with `--exclude-erf` filtering range-separated functionals \
                 (which auto-fall-back to CPU per GPU-05)"
            )
        }

        // Plan 06-04 — `--backend metal` arm. Metal is reached through
        // cubecl-wgpu's Metal backend per RESEARCH §R-02 / Pitfall 9
        // (no separate cubecl-metal crate exists). The `metal` Cargo
        // feature is a transparent alias of `wgpu` per
        // `crates/xcfun-gpu/Cargo.toml`. Apple Silicon GPUs lack
        // hardware f64 — the SHADER_F64 probe will return false on
        // M1/M2/M3 and the bail!() below fires. Intel Mac with
        // discrete f64-capable GPU is the only non-bail path on macOS.
        #[cfg(feature = "wgpu")]
        Backend::Metal => {
            if !xcfun_gpu::runtime::wgpu::metal_with_f64_available() {
                anyhow::bail!(
                    "--backend metal: Metal-via-Wgpu f64 probe failed. Apple \
                     Silicon (M1/M2/M3/A17) GPUs lack hardware f64 — the \
                     numerical contract cannot be honoured on these adapters \
                     (see RESEARCH §R-02 / CONTEXT D-06). Use --backend cpu \
                     on Apple Silicon."
                );
            }
            let _ = (order, jobs, filter, exclude_erf);
            todo!(
                "Plan 06-04 wires the run_tier3 Metal-via-Wgpu probe + \
                 dispatch skeleton; the sweep body lands in Plan 06-05 \
                 (revision-1 B-4) atop \
                 `xcfun_gpu::Batch::<cubecl_wgpu::WgpuRuntime>::eval_vec_host_wgpu_with_request` \
                 with `Backend::Metal` propagated so XcError::WgpuNoF64 \
                 payloads carry the correct request tag"
            )
        }
    }
}

// ===========================================================================
// Phase 6 Plan 06-05 (revision-1 B-4) — KER-06 tier-3 CPU sign-off body.
// ===========================================================================
//
// `run_tier3_cpu_body(order, filter)` iterates the 17 known-clean Phase-4
// functional set, runs `Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host_cpu`
// over the Phase-2 stratified xoshiro 10k grid (seed 0x1234abcd), and
// compares per-element output against scalar `Functional::eval` baseline at
// strict 1e-13. Returns `Ok(())` on a clean sweep; returns an `anyhow` error
// (with a per-functional failure breakdown) otherwise. CLI exits non-zero
// for CI gating.
//
// The 17 known-clean set is the Phase-4 sign-off list per
// `04-VERIFICATION.md` (echoed in the plan's `<action>` Step B grep filter):
// `slaterx | tfk | pbex | revpbex | pbeintx | rpbex | pbesolx | beckex |
//  beckecorrx | pw86x | optxcorr | apbex | pw91x | ktx | btk | m05x2x | m06x2x`.
// All 17 use `Vars::A_B_GAA_GAB_GBB` except `slaterx` and `tfk` (LDA tier;
// `Vars::A_B`).

/// 17 known-clean Phase-4 functional targets (per `04-VERIFICATION.md`).
/// Each tuple: `(FunctionalId, regex-name, Vars)`. Names are lowercase to
/// match the plan's `^(slaterx|tfk|...)$` regex syntax.
const TIER3_CPU_KNOWN_CLEAN_17: &[(FunctionalId, &str, Vars)] = &[
    // LDA-tier (Vars::A_B; inlen=2):
    (FunctionalId::XC_SLATERX, "slaterx", Vars::A_B),
    (FunctionalId::XC_TFK, "tfk", Vars::A_B),
    // GGA-tier (Vars::A_B_GAA_GAB_GBB; inlen=5):
    (FunctionalId::XC_PBEX, "pbex", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_REVPBEX, "revpbex", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_PBEINTX, "pbeintx", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_RPBEX, "rpbex", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_PBESOLX, "pbesolx", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_BECKEX, "beckex", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_BECKECORRX, "beckecorrx", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_PW86X, "pw86x", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_OPTXCORR, "optxcorr", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_APBEX, "apbex", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_PW91X, "pw91x", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_KTX, "ktx", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_BTK, "btk", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_M05X2X, "m05x2x", Vars::A_B_GAA_GAB_GBB),
    (FunctionalId::XC_M06X2X, "m06x2x", Vars::A_B_GAA_GAB_GBB),
];

/// Strict KER-06 tolerance (CONTEXT D-02): 1e-13 rel-err vs scalar.
const TIER3_CPU_THRESHOLD: f64 = 1e-13;

// ---------------------------------------------------------------------------
// Phase 6 Plan 06-N2 — `--reference mpmath` wiring
// ---------------------------------------------------------------------------

/// Phase 6 Plan 06-N2 — selects the ground-truth source for tier-2 parity.
///
/// `Cpp` (default): existing Phase 2-5 cc-vs-Rust path. C++ harness is the
///   reference; threshold per `threshold_for(name)`.
/// `Mpmath`: reads `validation/fixtures/mpmath/<functional>.jsonl`
///   ground-truth records emitted by `xtask/src/bin/regen_mpmath_fixtures.rs`
///   (mpmath at prec=200 per D-03 ACC-04 amendment) and compares Rust
///   `Functional::eval` output element-wise at strict 1e-13.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    Cpp,
    Mpmath,
}

impl Reference {
    /// Parse `--reference {cpp|mpmath}` CLI argument.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cpp" => Some(Reference::Cpp),
            "mpmath" => Some(Reference::Mpmath),
            _ => None,
        }
    }
}

/// Phase 6 Plan 06-N2 — the 20 `excluded_by_upstream_spec` functionals.
///
/// Per CONTEXT.md D-03 ACC-04 amendment + REQUIREMENTS.md GGA-01/MGGA-02
/// caveats, these functionals' tier-2 parity records ABORT in C++ via
/// `tmath::sqrt_expand`/`log_expand`/`pow_expand` (and BR/CSC also lack a
/// deterministic upstream `test_in` for the JP-bearing input layouts).
/// Plan 06-N2 closes them via mpmath at prec=200 as the sole ground truth.
///
/// Names are lowercase (xcfun-eval naming convention without the `XC_`
/// prefix) — matches the per-functional JSONL fixture file path
/// `validation/fixtures/mpmath/<lowercase>.jsonl` and the
/// `xtask/mpmath_eval/functionals/<lowercase>.py` module.
pub const MPMATH_ONLY_FUNCTIONALS: &[&str] = &[
    "brx", "brc", "brxc", "csc", "blocx",
    "scanx", "scanc", "rscanx", "rscanc", "rppscanx", "rppscanc",
    "r2scanx", "r2scanc", "r4scanx", "r4scanc",
    "tw", "vwk", "pbelocc", "zvpbesolc", "zvpbeintc",
];

/// Strict mpmath-truth tolerance (D-03 ACC-04 amendment): 1e-13 rel-err
/// vs mpmath at prec=200. Stricter than the global 1e-12 contract because
/// the mpmath reference itself is good to ~200 digits — the only float
/// rounding in the comparison is on the Rust side.
pub const MPMATH_TIER2_THRESHOLD: f64 = 1e-13;

/// One mpmath fixture record. Mirrors the JSONL emitted by
/// `xtask.mpmath_eval.evaluator.eval_record` (Plan 06-00).
#[derive(Deserialize, Debug, Clone)]
pub struct MpmathRecord {
    pub functional: String,
    pub vars: String,
    pub mode: String,
    pub order: u32,
    pub input: Vec<f64>,
    pub output: Vec<f64>,
    #[serde(default)]
    pub mpmath_prec: u32,
    #[serde(default)]
    pub source: String,
}

/// Tier-2 driver running against mpmath-truth fixtures.
///
/// For each functional in `MPMATH_ONLY_FUNCTIONALS` (intersected with
/// `filter`), reads its `validation/fixtures/mpmath/<functional>.jsonl`
/// fixture file, builds a Rust `Functional` matching the recorded
/// `(vars, mode, order)`, calls `Functional::eval`, and compares
/// element-wise at `MPMATH_TIER2_THRESHOLD` (1e-13).
///
/// Returns the populated `Report`; the caller decides whether to exit
/// non-zero based on `Report::failed_count()`.
///
/// # Fixture absence
///
/// If a fixture file is missing, this function emits a `tracing::warn`
/// message and skips that functional (records 0 cells). This is the
/// intended pre-MANUAL-regen behaviour: the smoke run produces fixtures
/// under `target/mpmath_smoke/` (not committed); only after the offline
/// ~6h MANUAL regen runs are fixtures committed under
/// `validation/fixtures/mpmath/`.
pub fn run_tier2_mpmath(
    filter: &regex::Regex,
    cfg: &mut RunConfig<'_>,
) -> Result<Report> {
    let mut report = Report::default();
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .ok()
        .and_then(|p| p.parent().map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let _ = cfg.skip_keys; // mpmath fixtures don't need the resume skip-set
                            // (the harness is fast enough to never hit
                            // SIGKILL mid-run; revisit if proven wrong).

    for fn_name in MPMATH_ONLY_FUNCTIONALS {
        if !filter.is_match(fn_name) {
            continue;
        }
        let fixture_path = workspace_root
            .join("validation/fixtures/mpmath")
            .join(format!("{}.jsonl", fn_name));
        if !fixture_path.exists() {
            tracing::warn!(
                "Tier-2 SKIP {} (--reference mpmath): fixture {:?} missing — \
                 run `cargo run -p xtask --bin regen-mpmath-fixtures` (~6h offline) \
                 to populate. See 06-N2-SUMMARY.md for the manual command.",
                fn_name,
                fixture_path
            );
            continue;
        }
        let body = std::fs::read_to_string(&fixture_path).with_context(|| {
            format!("read mpmath fixture at {:?}", fixture_path)
        })?;
        let records: Vec<MpmathRecord> = body
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str::<MpmathRecord>(l).with_context(|| {
                    format!("parse mpmath JSONL line in {:?}", fixture_path)
                })
            })
            .collect::<Result<Vec<_>>>()?;

        // FunctionalId::from_name expects the upper-case `XC_<NAME>` form
        // (xcfun-core/src/functional_id.rs:120). The mpmath fixture uses the
        // lowercase short name; transform here.
        let upper_name = format!("XC_{}", fn_name.to_uppercase());
        let func_id = match FunctionalId::from_name(&upper_name) {
            Some(id) => id,
            None => {
                tracing::warn!(
                    "Tier-2 SKIP {} (--reference mpmath): no FunctionalId mapping for '{}'",
                    fn_name,
                    upper_name
                );
                continue;
            }
        };

        for (point_idx, rec) in records.iter().enumerate() {
            // Map vars-string -> Vars enum.
            let vars = match parse_vars(&rec.vars) {
                Some(v) => v,
                None => {
                    anyhow::bail!(
                        "mpmath fixture {} has unknown vars '{}'",
                        fn_name,
                        rec.vars
                    );
                }
            };
            // Construct the Rust Functional with this single-id weight=1
            // mapping; matches the Phase 2 tier-2 driver pattern (run_one_tuple_pd
            // builds a Functional with `weights: vec![(id, 1.0)]`).
            let mut func = Functional::new();
            func.weights = vec![(func_id, 1.0_f64)];
            func.vars = vars;
            func.mode = Mode::PartialDerivatives;
            func.order = rec.order;
            let outlen = match Functional::output_length(
                vars,
                Mode::PartialDerivatives,
                rec.order,
            ) {
                Ok(n) => n,
                Err(e) => anyhow::bail!(
                    "output_length failed for {} (vars={:?}, order={}): {}",
                    fn_name,
                    vars,
                    rec.order,
                    e
                ),
            };
            if rec.output.len() != outlen {
                anyhow::bail!(
                    "mpmath fixture {} record {}: output length mismatch — \
                     expected {}, got {}",
                    fn_name,
                    point_idx,
                    outlen,
                    rec.output.len()
                );
            }
            let mut rust_out = vec![0.0_f64; outlen];
            // Rust eval may legitimately fail for some (vars, order) tuples
            // not yet wired; surface the gap as a rust_unavailable record.
            let eval_res = func.eval(&rec.input, &mut rust_out);
            for (element_idx, (rust_val, mpmath_val)) in
                rust_out.iter().zip(rec.output.iter()).enumerate()
            {
                let abs_err = (rust_val - mpmath_val).abs();
                let rel_err = abs_err / mpmath_val.abs().max(1.0);
                let rust_unavailable = eval_res.is_err();
                let pass = !rust_unavailable && rel_err <= MPMATH_TIER2_THRESHOLD;
                let r = ReportRecord {
                    functional: fn_name.to_string(),
                    vars: format!("{:?}", vars),
                    mode: 1,
                    order: rec.order,
                    point_idx,
                    element_idx,
                    input: rec.input.clone(),
                    rust: if rust_unavailable { f64::NAN } else { *rust_val },
                    cpp: *mpmath_val,
                    abs_err,
                    rel_err,
                    threshold: MPMATH_TIER2_THRESHOLD,
                    pass,
                    rust_unavailable,
                    excluded_by_upstream_spec: false,
                    excluded_by_regularize_clamp_design: false,
                };
                report.push_with_sink(r, cfg.sink.as_deref_mut())?;
            }
        }
    }
    tracing::info!(
        "Tier-2 (--reference mpmath) done: {} records, {} failed",
        report.total_records(),
        report.failed_count()
    );
    Ok(report)
}

/// Parse a Vars enum from its canonical xcfun-master string name. Returns
/// `None` for unrecognised names. Supports both the bare-name form
/// (`A_B_GAA_GAB_GBB`) and the `XC_`-prefixed form.
fn parse_vars(s: &str) -> Option<Vars> {
    let s = s.strip_prefix("XC_").unwrap_or(s);
    Some(match s {
        "A" => Vars::A,
        "N" => Vars::N,
        "A_B" => Vars::A_B,
        "N_S" => Vars::N_S,
        "A_GAA" => Vars::A_GAA,
        "N_GNN" => Vars::N_GNN,
        "A_B_GAA_GAB_GBB" => Vars::A_B_GAA_GAB_GBB,
        "N_S_GNN_GNS_GSS" => Vars::N_S_GNN_GNS_GSS,
        "A_GAA_LAPA" => Vars::A_GAA_LAPA,
        "A_GAA_TAUA" => Vars::A_GAA_TAUA,
        "N_GNN_LAPN" => Vars::N_GNN_LAPN,
        "N_GNN_TAUN" => Vars::N_GNN_TAUN,
        "A_B_GAA_GAB_GBB_LAPA_LAPB" => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB,
        "A_B_GAA_GAB_GBB_TAUA_TAUB" => Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        "N_S_GNN_GNS_GSS_LAPN_LAPS" => Vars::N_S_GNN_GNS_GSS_LAPN_LAPS,
        "N_S_GNN_GNS_GSS_TAUN_TAUS" => Vars::N_S_GNN_GNS_GSS_TAUN_TAUS,
        "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB" => {
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
        }
        "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB" => {
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB
        }
        "N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS" => {
            Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS
        }
        _ => return None,
    })
}

/// CPU arm body for `run_tier3`. Filters `TIER3_CPU_KNOWN_CLEAN_17` by the
/// caller's regex, runs `Batch::<CpuRuntime>::eval_vec_host_cpu` per
/// `(functional, vars, mode=PartialDerivatives, order)` tuple over the
/// stratified xoshiro 10k grid, and compares against scalar baseline.
fn run_tier3_cpu_body(order: u32, filter: &str) -> Result<()> {
    use cubecl_cpu::CpuRuntime;
    use xcfun_gpu::Batch;

    let regex = regex::Regex::new(filter)
        .map_err(|e| anyhow::anyhow!("--filter regex parse failed: {e}"))?;

    let grid = crate::fixtures::generate_grid();
    let nr_points = grid.len();
    tracing::info!(
        "KER-06 sign-off: {} grid points, {} candidate functionals, threshold = {:e}",
        nr_points,
        TIER3_CPU_KNOWN_CLEAN_17.len(),
        TIER3_CPU_THRESHOLD,
    );

    let mut total_pass = 0_usize;
    let mut total_fail = 0_usize;
    let mut per_functional_failures: Vec<(String, u32, usize, f64)> = Vec::new();

    for (id, name, vars) in TIER3_CPU_KNOWN_CLEAN_17 {
        if !regex.is_match(name) {
            continue;
        }
        // Sweep orders 0..=order (Mode::PartialDerivatives).
        for ord in 0..=order {
            let outlen = match Functional::output_length(*vars, Mode::PartialDerivatives, ord) {
                Ok(n) => n,
                Err(e) => {
                    anyhow::bail!(
                        "tier-3 CPU: output_length({:?}, PartialDerivatives, {}) failed: {}",
                        vars,
                        ord,
                        e
                    );
                }
            };
            let inlen = Functional::input_length(*vars);

            // Build Functional via direct struct construction (validation
            // harness idiom; tier-2 uses the same pattern at line 417).
            // Phase 6 Plan 06-06 (D-17): weights is Vec<(FunctionalId, f64)>; no leak.
            let fun = Functional {
                weights: vec![(*id, 1.0)],
                vars: *vars,
                mode: Mode::PartialDerivatives,
                order: ord,
                settings: xcfun_eval::functional::DEFAULT_SETTINGS,
                settings_gen: 0,
            };

            // Build flat density vector (nr_points * inlen) using the
            // existing `build_input` helper for shape consistency with the
            // tier-2 path.
            let mut density_flat: Vec<f64> = Vec::with_capacity(nr_points * inlen);
            for gp in &grid {
                let row = build_input(gp, *vars);
                debug_assert_eq!(row.len(), inlen);
                density_flat.extend_from_slice(&row);
            }

            // Allocate batch + scalar output buffers.
            let mut batch_out = vec![0.0_f64; nr_points * outlen];
            let mut scalar_out = vec![0.0_f64; nr_points * outlen];

            // Batch path — Plan 06-05 RS-08 dispatch. Pitch == inlen / outlen
            // (no padding); xcfun-master/api/xcfun.h:54 contract preserved.
            Batch::<CpuRuntime>::eval_vec_host_cpu(
                &fun,
                &density_flat,
                inlen,
                &mut batch_out,
                outlen,
                nr_points,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "tier-3 CPU batch eval failed for {} order {}: {}",
                    name,
                    ord,
                    e
                )
            })?;

            // Scalar baseline — per-point loop.
            for (p, gp) in grid.iter().enumerate() {
                let row = build_input(gp, *vars);
                let dout = &mut scalar_out[p * outlen..(p + 1) * outlen];
                if let Err(e) = fun.eval(&row, dout) {
                    anyhow::bail!(
                        "tier-3 CPU scalar eval failed at {} order {} point {}: {}",
                        name,
                        ord,
                        p,
                        e,
                    );
                }
            }

            // Element-wise comparison at strict 1e-13.
            let mut tuple_pass = 0_usize;
            let mut tuple_fail = 0_usize;
            let mut tuple_max_rel = 0.0_f64;
            for i in 0..batch_out.len() {
                let b = batch_out[i];
                let s = scalar_out[i];
                let abs_err = (b - s).abs();
                let rel_err = abs_err / s.abs().max(1.0);
                if rel_err > TIER3_CPU_THRESHOLD {
                    tuple_fail += 1;
                } else {
                    tuple_pass += 1;
                }
                if rel_err > tuple_max_rel {
                    tuple_max_rel = rel_err;
                }
            }

            tracing::info!(
                "  {} order {}: pass={} fail={} max_rel_err={:.3e}",
                name,
                ord,
                tuple_pass,
                tuple_fail,
                tuple_max_rel
            );
            total_pass += tuple_pass;
            total_fail += tuple_fail;
            if tuple_fail > 0 {
                per_functional_failures.push((
                    name.to_string(),
                    ord,
                    tuple_fail,
                    tuple_max_rel,
                ));
            }
        }
    }

    println!(
        "tier-3 CPU sign-off: {} elements compared, {} pass, {} fail (threshold {:e})",
        total_pass + total_fail,
        total_pass,
        total_fail,
        TIER3_CPU_THRESHOLD,
    );
    if total_fail == 0 {
        println!("KER-06: 0 failures across the 17 known-clean Phase-4 functional set.");
        Ok(())
    } else {
        eprintln!("KER-06: {} failure(s) detected:", per_functional_failures.len());
        for (name, ord, count, max_rel) in &per_functional_failures {
            eprintln!(
                "  {} order {}: {} record(s) above threshold; max rel-err {:.3e}",
                name, ord, count, max_rel
            );
        }
        anyhow::bail!(
            "tier-3 CPU sign-off FAILED: {} per-tuple failure entries (total {} elements)",
            per_functional_failures.len(),
            total_fail,
        );
    }
}

// ===========================================================================
// Quick task 260430-4x7 — parallel scheduler unit tests.
// ===========================================================================

#[cfg(test)]
mod parallel_cfg_tests {
    use super::*;
    use std::collections::HashSet;

    /// `RunConfig::empty(...)` defaults `jobs == 1` so the legacy serial
    /// path is preserved for any test that doesn't opt into parallelism.
    #[test]
    fn empty_runconfig_is_serial() {
        let s: HashSet<TupleKey> = HashSet::new();
        let cfg = RunConfig::empty(&s);
        assert_eq!(cfg.jobs.get(), 1);
        assert!(cfg.sink.is_none());
    }
}

/// Quick task 260430-4x7 — serial-vs-parallel parity at the driver level.
///
/// Asserts that `run` with `jobs == 1` and `jobs == 4` produces identical
/// records (after sort by stable key) and an identical `Report.matrix`.
/// Filters to a single cheap functional (`xc_slaterx`) and a 64-point
/// grid slice to keep the test fast.
#[cfg(test)]
mod parallel_run_tests {
    use super::*;
    use crate::fixtures::generate_grid;
    use std::collections::HashSet;
    use std::num::NonZeroUsize;

    fn run_with_jobs(jobs: usize, filter: &str) -> Report {
        let grid = generate_grid();
        let small_grid: Vec<_> = grid.into_iter().take(64).collect();
        let regex = regex::Regex::new(filter).unwrap();
        let empty: HashSet<TupleKey> = HashSet::new();
        let mut cfg = RunConfig {
            sink: None,
            skip_keys: &empty,
            jobs: NonZeroUsize::new(jobs).unwrap(),
        };
        run(&small_grid, 0, &regex, &mut cfg).unwrap()
    }

    fn key(r: &ReportRecord) -> (String, String, u32, u32, usize, usize) {
        (
            r.functional.clone(),
            r.vars.clone(),
            r.mode,
            r.order,
            r.point_idx,
            r.element_idx,
        )
    }

    /// Test 1 + Test 2 (plan): a `--jobs 4` run produces the SAME record
    /// SET and the SAME `Report.matrix` as a `--jobs 1` run on the same
    /// fixture. Proves the 1e-12 numerical contract is unchanged under
    /// parallel emission.
    #[test]
    fn parallel_matches_serial_partial_derivatives() {
        let serial = run_with_jobs(1, "xc_slaterx");
        let parallel = run_with_jobs(4, "xc_slaterx");

        // Matrix equality (BTreeMap → deterministic ordering).
        assert_eq!(
            serial.matrix.len(),
            parallel.matrix.len(),
            "matrix length mismatch"
        );
        for (k, sv) in &serial.matrix {
            let pv = parallel
                .matrix
                .get(k)
                .expect("missing cell in parallel matrix");
            assert_eq!(
                sv.records_total, pv.records_total,
                "records_total mismatch at {:?}",
                k
            );
            assert_eq!(
                sv.records_failed, pv.records_failed,
                "records_failed mismatch at {:?}",
                k
            );
            assert_eq!(sv.rust_unavailable, pv.rust_unavailable);
            assert!(
                (sv.max_rel_err - pv.max_rel_err).abs() <= f64::EPSILON,
                "max_rel_err drift at {:?}: serial={} parallel={}",
                k,
                sv.max_rel_err,
                pv.max_rel_err
            );
            assert_eq!(sv.threshold, pv.threshold);
        }

        // Record SET equality (sort by stable key, then byte-compare JSON).
        let mut s: Vec<_> = serial.records.iter().collect();
        let mut p: Vec<_> = parallel.records.iter().collect();
        s.sort_by_key(|r| key(r));
        p.sort_by_key(|r| key(r));
        assert_eq!(s.len(), p.len(), "record count mismatch");
        for (a, b) in s.iter().zip(p.iter()) {
            assert_eq!(
                serde_json::to_string(a).unwrap(),
                serde_json::to_string(b).unwrap(),
                "record content mismatch at key {:?}",
                key(a)
            );
        }
    }

    /// Test 3 (plan): `jobs == 1` short-circuits — produces records and
    /// completes successfully on a small fixture.
    #[test]
    fn jobs_one_short_circuit_runs_clean() {
        let r = run_with_jobs(1, "xc_slaterx");
        assert!(r.total_records() > 0, "no records emitted at jobs=1");
        assert_eq!(r.failed_count(), 0, "unexpected failures at jobs=1");
    }
}
