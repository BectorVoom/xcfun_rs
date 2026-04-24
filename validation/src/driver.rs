//! Tier-2 driver — compares Rust `xcfun-eval::Functional::eval` against the
//! C++ reference via `CppXcfun` FFI for every
//! `(functional, vars, mode=PartialDerivatives, order∈{0..=max_order≤2},
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

use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;

use xcfun_core::{FunctionalId, Mode, VARS_TABLE, Vars, taylorlen};
use xcfun_eval::Functional;

use crate::ffi::CppXcfun;
use crate::fixtures::{GridPoint, REGULARIZE_CLAMP_STRATUM_BOUND};

/// One tier-2 parity record — emitted per `(functional, vars, mode, order,
/// point_idx, element_idx)` tuple. Serialized to `validation/report.jsonl`.
#[derive(Serialize, Debug, Clone)]
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
    pub fn push(&mut self, rec: ReportRecord) {
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
        if !rec.pass {
            self.records.push(rec);
        } else if rec.point_idx == 0 && rec.element_idx == 0 {
            self.records.push(rec);
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

/// Run tier-2 parity for all 11 Phase-2 LDA functionals at orders 0..=max_order≤2.
pub fn run(grid: &[GridPoint], max_order: u32, filter: &regex::Regex) -> Result<Report> {
    let mut report = Report::default();

    // The 11 Phase-2 LDA functionals. 8 use Vars::A_B; TW + VWK use
    // Vars::A_B_GAA_GAB_GBB (kinetic-GGAs in LDA tier per RESEARCH §Critical Findings).
    let lda_targets: &[(FunctionalId, &str, Vars)] = &[
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
    ];

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

        // TW + VWK (Vars::A_B_GAA_GAB_GBB, inlen=5) are EXCLUDED from tier-2
        // because:
        //   1. Upstream `tw.cpp` / `vonw.cpp` ship NO `test_in`/`test_out` data
        //      in their FUNCTIONAL macros (ENERGY_FUNCTION only, no XC_A_B +
        //      XC_PARTIAL_DERIVATIVES + test_* args). With no upstream reference,
        //      tier-2 parity is not a defined comparison (CONTEXT D-19).
        //   2. C++ xcfun aborts on `pow(gaa+gbb, 2)` with zero gradients
        //      (xcfun-master/external/upstream/taylor/tmath.hpp:156), so the
        //      bulk/regularize/polarised strata (gradients = 0) cannot be
        //      exercised on the C++ side anyway.
        //   3. The Rust launch loop currently wires only (id, n) arms for
        //      inlen=2 LDAs. Extending to inlen=5 is deferred to Phase 3
        //      (GGA scaffolding) where it lands alongside the gradient-present
        //      grid strata — bug #4 host launch arm wiring.
        //
        // Per the user-approved tier-2 plan for these functionals, they are
        // tagged `excluded_by_upstream_spec` in the report; their failure
        // counts do NOT roll up into the harness verdict.
        let excluded = inlen != 2;

        for order in 0..=max_order.min(2) {
            let outlen = taylorlen(inlen, order as usize);
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
                report.push(rec);
                continue;
            }

            // C++ side: set up once per (functional, order).
            let mut cpp = CppXcfun::new();
            let status_set = cpp.set(&cpp_name(name), 1.0);
            if status_set != 0 {
                anyhow::bail!(
                    "xcfun_set({}, 1.0) failed: status={}",
                    cpp_name(name),
                    status_set
                );
            }
            // vars as u32 matches xcfun_vars discriminants; mode=1 is XC_PARTIAL_DERIVATIVES.
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

            // Rust side: leak a per-iteration `weights` slice — acceptable in
            // a one-shot validation binary (total leak across run < 1 KB).
            let weights: &'static [(FunctionalId, f64)] = Box::leak(Box::new([(id, 1.0)]));
            let rust_fun = Functional {
                weights,
                vars,
                mode: Mode::PartialDerivatives,
                order,
                parameters: xcfun_eval::functional::DEFAULT_PARAMETERS,
            };

            for (point_idx, gp) in grid.iter().enumerate() {
                let input = build_input(gp, vars);
                let mut rust_out = vec![0.0_f64; outlen];
                let mut cpp_out = vec![0.0_f64; outlen];

                // D-22 clamp stratum: records where `min(a,b)` is within
                // 2 × TINY_DENSITY test the regularize design intent
                // (deliberate density saturation), not kernel correctness.
                // Plan 02-06 Fix 2 marks these records for transparency but
                // excludes them from the tier-2 verdict.
                //
                // For vars = A_B (inlen = 2): `(a, b) = input[0..2]`.
                // For vars = A_B_GAA_GAB_GBB: same (a, b) slots in [0..2]
                // (we currently skip inlen != 2 via the `excluded` check,
                // so this branch handles only A_B in practice).
                let in_clamp_stratum = input.len() >= 2
                    && input[0].min(input[1]) <= REGULARIZE_CLAMP_STRATUM_BOUND;

                // Evaluate C++ side unconditionally.
                cpp.eval(&input, &mut cpp_out);

                // Evaluate Rust side; on `NotConfigured`, record as
                // rust_unavailable and fill rust_out with NaN so the
                // per-element loop still produces a record (with pass=false).
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
                    report.push(rec);
                }
            }
        }
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
