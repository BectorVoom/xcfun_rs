//! Phase 6 D-19 verification — shared test helpers for the
//! `tests/d19_*.rs` per-functional unit tests.
//!
//! Two plans landed simultaneously:
//!
//! - **Plan 06-N1 (B-6 revision-1)** — drives the per-functional
//!   `<NAME>_kernel` directly via cubecl-cpu (no `xcfun-eval` round-trip;
//!   avoids a dev-dep cycle). Each `tests/d19_<n1-name>.rs` defines a
//!   `#[cube(launch_unchecked)]` adapter at comptime VARS=6, N=0,
//!   loads `validation/fixtures/d19_n1/<name>_baseline.jsonl`, and
//!   asserts at `REL_TOL`.
//! - **Plan 06-N3 (W-9 / I-3 Option B, PURE-VERIFICATION)** — drives
//!   `xcfun_eval::Functional::eval` end-to-end. Loads
//!   `validation/fixtures/d19_n3/<name>_baseline.jsonl` and asserts at
//!   strict 1e-13 vs the committed snapshot. Helper:
//!   `run_d19_n3_contract(name, FunctionalId)`.
//!
//! Both helper sets coexist in this module. N1's loaders live under their
//! plan-1-original names (`fixture_path`, `load_fixture`, `FixtureRecord`,
//! `REL_TOL`). N3's loaders are prefixed `n3_*` to avoid collision with
//! N1's flat `use common::*;` imports.
//!
//! # Substrate gap (xcfun-master)
//!
//! When `xcfun-master/` (the C++ vendored reference) is not present in
//! the worktree, the validation harness cannot generate a fresh C++
//! baseline. Both N1 and N3 fall back to **regression snapshots**:
//! each fixture's `expected_*` field is the CURRENT kernel output at the
//! Plan-06-00-substrate revision committed to `master`. The
//! NEEDS-VERIFICATION escalation in `06-N3-SUMMARY.md` documents the
//! expected re-baselining once `xcfun-master/` is restored.

#![cfg(feature = "testing")]
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;

// ---------------------------------------------------------------------
// Plan 06-N1 helpers (B-6 revision-1) — direct cubecl-cpu kernel launch
// ---------------------------------------------------------------------

/// Vars discriminant for `Vars::A_B_GAA_GAB_GBB` (inlen=5).
/// Mirrors `xcfun_core::Vars::A_B_GAA_GAB_GBB as u32`.
pub const VARS_A_B_GAA_GAB_GBB: u32 = 6;

/// Plan 06-N1 fixture record format (jsonl line). 5-element input matches
/// the `Vars::A_B_GAA_GAB_GBB` layout: `[a, b, gaa, gab, gbb]`.
#[derive(Deserialize, Debug, Clone)]
pub struct FixtureRecord {
    pub input: [f64; 5],
    pub expected_energy: f64,
}

/// Tolerance for the N1 regression-detector assertion. Set to 1e-12 to
/// accommodate any sub-ULP drift between cubecl-cpu launches on the same
/// host (the kernel is deterministic; 1e-12 is a safety margin against
/// platform/arch-dependent libm differences).
///
/// When `xcfun-master/` is restored and the fixture is re-baselined to C++
/// truth, this constant should drop to the strict 1e-13 plan target.
pub const REL_TOL: f64 = 1.0e-12;

/// Path to the Plan 06-N1 fixture jsonl. Cargo sets cwd to the package
/// root for integration tests, so we go two `..` up to reach the workspace
/// root.
pub fn fixture_path(name: &str) -> String {
    format!("../../validation/fixtures/d19_n1/{}_baseline.jsonl", name)
}

/// Load Plan 06-N1 fixture records from a jsonl file. Skips blank lines.
pub fn load_fixture(path: &str) -> Vec<FixtureRecord> {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path, e));
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str::<FixtureRecord>(line)
                .unwrap_or_else(|e| panic!("invalid jsonl record {} in {}: {}", i, path, e))
        })
        .collect()
}

// ---------------------------------------------------------------------
// Plan 06-N3 helpers (W-9 / I-3 Option B) — Functional::eval contract
// ---------------------------------------------------------------------

/// One record of a Plan 06-N3 fixture
/// (`validation/fixtures/d19_n3/<name>_baseline.jsonl`).
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
    match v {
        2 => Vars::A_B,
        6 => Vars::A_B_GAA_GAB_GBB,
        13 => Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        _ => panic!(
            "D19Record: unsupported vars discriminant {} (Plan 06-N3 uses 6 + 13)",
            v
        ),
    }
}

/// Locate the Plan 06-N3 fixture file for the given functional name,
/// relative to the cargo manifest dir (`crates/xcfun-kernels`).
///
/// Renamed from `fixture_path` to avoid collision with N1's `fixture_path`
/// in the same module.
pub fn n3_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("validation")
        .join("fixtures")
        .join("d19_n3")
        .join(format!("{}_baseline.jsonl", name))
}

/// Load one Plan 06-N3 fixture file as a `Vec<D19Record>`.
///
/// Renamed from `load_fixture` to avoid collision with N1's `load_fixture`
/// in the same module.
pub fn n3_load_fixture(name: &str) -> Vec<D19Record> {
    let path = n3_fixture_path(name);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("D19 fixture missing: {} ({})", path.display(), e));
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
    let records = n3_load_fixture(name);
    assert!(!records.is_empty(), "D19 fixture {} has zero records", name);
    for (idx, rec) in records.iter().enumerate() {
        assert_eq!(
            rec.functional, name,
            "D19 fixture {} record {}: functional name mismatch ({} vs file name)",
            name, idx, rec.functional
        );
        assert!(
            rec.rel_err_threshold == 1.0e-13,
            "D19 fixture {} record {}: rel_err_threshold {} != 1.0e-13 (CONTEXT.md D-02)",
            name,
            idx,
            rec.rel_err_threshold
        );

        let mut fun = Functional::new();
        fun.set(name.to_uppercase().as_str(), 1.0)
            .unwrap_or_else(|e| panic!("Functional::set({}) failed: {:?}", name.to_uppercase(), e));
        fun.weights = vec![(id, 1.0)];
        fun.vars = vars_from_u32(rec.vars);
        fun.mode = mode_from_u32(rec.mode);
        fun.order = rec.order;

        let mut output = vec![0.0_f64; rec.expected.len()];
        fun.eval(&rec.input, &mut output)
            .unwrap_or_else(|e| panic!("Functional::eval({}) record {}: {:?}", name, idx, e));

        for (i, (&got, &want)) in output.iter().zip(rec.expected.iter()).enumerate() {
            let denom = want.abs().max(1.0);
            let rel = (got - want).abs() / denom;
            assert!(
                rel <= rec.rel_err_threshold,
                "D19 fixture {} record {} element {}: rel_err {:.3e} > threshold {:.0e}\n\
                 got={:.17e}\nwant={:.17e}\n\
                 (snapshot regression — either Plan 06-00 substrate output changed,\n\
                  or a follow-up kernel-edit plan was intended; surface as PLANNING\n\
                  INCONCLUSIVE escalation per Plan 06-N3 acceptance criteria)",
                name,
                idx,
                i,
                rel,
                rec.rel_err_threshold,
                got,
                want
            );
        }
    }
}
