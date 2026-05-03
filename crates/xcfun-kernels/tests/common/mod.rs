//! Shared helpers for the Plan 06-N3 D-19 small-magnitude residual sweep.
//!
//! Plan 06-N3 (NEW per Phase-6 revision-1) is the post-libm-hybrid
//! verification sweep over the ~18 small-magnitude AD-residual D-19
//! forwards from Phase 4 (`04-VERIFICATION.md` D-19 ledger):
//!
//! - M05/M06 family (10 functionals): M05X, M05C, M05X2C, M06X, M06C,
//!   M06LX, M06LC, M06HFX, M06HFC, M06X2C
//! - B97 X-side (3): B97X, B97_1X, B97_2X
//! - Singletons (5): LYPC, VWN_PBEC, PW92C, PBEC, OPTX
//!
//! ## I-3 revision-2 — Option B (PURE-VERIFICATION)
//!
//! This plan creates per-functional fixtures + unit tests + runs them. It
//! makes ZERO kernel-source-file edits. Therefore the helpers in this
//! module ONLY load fixtures and re-run `Functional::eval` at the same
//! curated density points; they do not modify any state under
//! `crates/xcfun-kernels/src/functionals/`.
//!
//! ## Snapshot semantics
//!
//! In environments where `xcfun-master/` (the C++ vendored reference) is
//! not present, the validation harness cannot generate a fresh C++
//! baseline. Plan 06-N3 instead uses **regression snapshots**: each
//! fixture's `expected` field is the CURRENT `Functional::eval` output at
//! the Plan-06-00-substrate revision committed to `master`. The unit
//! tests assert strict 1e-13 vs the committed snapshot. This gives:
//!
//! - **Stability contract:** any future kernel-edit plan that changes
//!   `Functional::eval` output at these density points MUST either
//!   preserve the snapshot to strict 1e-13 OR explicitly re-emit the
//!   fixture in a commit citing the new ground truth (e.g. when the
//!   orchestrator dispatches a kernel-edit follow-up plan after this
//!   plan returns NEEDS-VERIFICATION).
//! - **Auto-tightening verification gap:** the hypothesis from
//!   `06-CONTEXT.md` "Specific Ideas" — that Plan 06-00 substrate
//!   self-tightens the small-magnitude D-19 residuals — is NOT
//!   confirmed by these tests alone. Confirmation requires re-running
//!   `cargo run -p validation --release -- --backend cpu --order 3
//!   --filter <names>` against the C++ baseline; this is documented as
//!   the NEEDS-VERIFICATION escalation in `06-N3-SUMMARY.md`.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;

/// One record of a Plan 06-N3 fixture (`validation/fixtures/d19_n3/<name>_baseline.jsonl`).
///
/// Each line of the JSONL file deserialises to one of these. The
/// per-functional unit test iterates the file and asserts each record's
/// `Functional::eval` output equals `expected` at strict
/// `rel_err_threshold` (1e-13).
#[derive(Debug, Deserialize)]
pub struct D19Record {
    /// Lowercase functional symbol, e.g. `"m05x"`. Must match the file name.
    pub functional: String,
    /// `Vars` discriminant as u32 (e.g. 6 for `Vars::A_B_GAA_GAB_GBB`,
    /// 13 for `Vars::A_B_GAA_GAB_GBB_TAUA_TAUB`). Decoded by
    /// `vars_from_u32` below.
    pub vars: u32,
    /// `Mode` discriminant as u32 (1 = PartialDerivatives, the only mode
    /// this plan exercises).
    pub mode: u32,
    /// Derivative order (0..=4). Plan 06-N3 fixtures all use order 3 —
    /// the order at which the Phase-4 D-19 sweep observed the residuals.
    pub order: u32,
    /// Flat density input. Length must equal `Vars::input_len(vars)` for
    /// `Mode::PartialDerivatives`. Hand-curated points from low-density
    /// polarised + gradient-stress strata per Phase-4 D-19.
    pub input: Vec<f64>,
    /// Snapshot of `Functional::eval` output at the substrate revision
    /// the fixture was generated at. Length = `taylorlen(inlen, order)`.
    pub expected: Vec<f64>,
    /// Per-record relative-error tolerance (1.0e-13 for all Plan 06-N3
    /// fixtures — the strict bar from CONTEXT.md D-02).
    pub rel_err_threshold: f64,
}

/// Decode `Mode` from its u32 discriminant. Plan 06-N3 fixtures only use
/// `Mode::PartialDerivatives = 1`.
pub fn mode_from_u32(m: u32) -> Mode {
    match m {
        1 => Mode::PartialDerivatives,
        2 => Mode::Potential,
        3 => Mode::Contracted,
        _ => panic!("D19Record: unsupported mode discriminant {}", m),
    }
}

/// Decode `Vars` from its u32 discriminant. Plan 06-N3 uses vars=6
/// (`A_B_GAA_GAB_GBB`) for the GGA forwards and vars=13
/// (`A_B_GAA_GAB_GBB_TAUA_TAUB`) for the metaGGA forwards.
pub fn vars_from_u32(v: u32) -> Vars {
    // Round-trip through Vars's discriminant. Vars is `#[repr(u32)]`.
    // Only the discriminants Plan 06-N3 exercises are wired.
    match v {
        2 => Vars::A_B,
        6 => Vars::A_B_GAA_GAB_GBB,
        13 => Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        _ => panic!("D19Record: unsupported vars discriminant {} (Plan 06-N3 uses 6 + 13)", v),
    }
}

/// Locate the fixture file for the given functional name, relative to the
/// cargo manifest dir (`crates/xcfun-kernels`).
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("validation")
        .join("fixtures")
        .join("d19_n3")
        .join(format!("{}_baseline.jsonl", name))
}

/// Load one Plan 06-N3 fixture file as a `Vec<D19Record>`.
pub fn load_fixture(name: &str) -> Vec<D19Record> {
    let path = fixture_path(name);
    let contents = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("D19 fixture missing: {} ({})", path.display(), e)
    });
    contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<D19Record>(l)
                .unwrap_or_else(|e| panic!("D19 fixture parse error in {}: {}", path.display(), e))
        })
        .collect()
}

/// Run the canonical Plan 06-N3 contract for one functional fixture: load
/// fixture, build `Functional` with `weight=1.0`, eval each record at
/// order 3, assert strict 1e-13 vs the snapshot.
///
/// Caller supplies the `FunctionalId` (which is duplicated from the file
/// name to keep the contract self-checking — a typo in either filename or
/// id surfaces immediately).
pub fn run_d19_n3_contract(name: &str, id: FunctionalId) {
    let records = load_fixture(name);
    assert!(
        !records.is_empty(),
        "D19 fixture {} has zero records",
        name
    );
    for (idx, rec) in records.iter().enumerate() {
        assert_eq!(
            rec.functional, name,
            "D19 fixture {} record {}: functional name mismatch ({} vs file name)",
            name, idx, rec.functional
        );
        assert!(
            rec.rel_err_threshold == 1.0e-13,
            "D19 fixture {} record {}: rel_err_threshold {} != 1.0e-13 (CONTEXT.md D-02)",
            name, idx, rec.rel_err_threshold
        );

        let mut fun = Functional::new();
        fun.set(name.to_uppercase().as_str(), 1.0).unwrap_or_else(|e| {
            panic!("Functional::set({}) failed: {:?}", name.to_uppercase(), e)
        });
        // Sanity: the `set` above wrote to settings; mirror into weights for eval.
        // xcfun-eval Functional uses `weights`; populate directly to match the
        // dispatch chain (the public `xcfun_rs::Functional` wraps this with
        // `sync_weights_from_settings`, but for tier-0 unit tests we set
        // weights directly — same shape as `crates/xcfun-eval/tests/self_tests.rs`).
        fun.weights = vec![(id, 1.0)];
        fun.vars = vars_from_u32(rec.vars);
        fun.mode = mode_from_u32(rec.mode);
        fun.order = rec.order;

        let mut output = vec![0.0_f64; rec.expected.len()];
        fun.eval(&rec.input, &mut output)
            .unwrap_or_else(|e| panic!("Functional::eval({}) record {}: {:?}", name, idx, e));

        for (i, (&got, &want)) in output.iter().zip(rec.expected.iter()).enumerate() {
            // Strict 1e-13 relative-error contract per CONTEXT.md D-02.
            // approx::assert_relative_eq! handles the `max(|want|, 1)` denominator.
            let denom = want.abs().max(1.0);
            let rel = (got - want).abs() / denom;
            assert!(
                rel <= rec.rel_err_threshold,
                "D19 fixture {} record {} element {}: rel_err {:.3e} > threshold {:.0e}\n\
                 got={:.17e}\nwant={:.17e}\n\
                 (snapshot regression — either Plan 06-00 substrate output changed,\n\
                  or a follow-up kernel-edit plan was intended; surface as PLANNING\n\
                  INCONCLUSIVE escalation per Plan 06-N3 acceptance criteria)",
                name, idx, i, rel, rec.rel_err_threshold, got, want
            );
        }
    }
}
