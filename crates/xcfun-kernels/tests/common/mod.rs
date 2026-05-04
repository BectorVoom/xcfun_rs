//! Phase 6 Plan 06-N1 (B-6 revision-1) — shared test helper for the 11
//! per-functional D-19 unit tests under `tests/d19_*.rs`.
//!
//! Each `tests/d19_<name>.rs` file:
//!   1. Defines a `#[cube(launch_unchecked)]` adapter that runs
//!      `build_densvars` + `<NAME>_kernel` at comptime VARS=6, N=0.
//!   2. Defines a thin `run(input)` Rust helper invoking that adapter via
//!      cubecl-cpu launch_unchecked.
//!   3. Reads `validation/fixtures/d19_n1/<name>_baseline.jsonl` records
//!      `{ "input": [a, b, gaa, gab, gbb], "expected_energy": <f64> }`.
//!   4. Asserts `assert_relative_eq!(actual, expected, max_relative = TOL)`.
//!
//! # Substrate gap (xcfun-master missing)
//!
//! The plan's intent is to lock each per-functional D-19 fix to a strict 1e-13
//! GREEN gate vs the upstream C++ reference. Without `xcfun-master/` in this
//! worktree, no C++ baseline is computable; the per-functional jsonl fixtures
//! ship with `expected_energy` set to the **current Rust kernel's output** at
//! commit time — a self-consistency regression detector.
//!
//! When `xcfun-master/` is restored, the fixtures should be regenerated with
//! C++ truth (or mpmath truth via the xtask sidecar shipped in Plan 06-00 D-04
//! when the per-functional `xtask/mpmath_eval/functionals/<name>.py` body is
//! filled in). The test bodies (this module + the 11 thin `d19_*.rs` files)
//! do not change — only the `expected_energy` field in the fixture flips.

#![cfg(feature = "testing")]
#![allow(dead_code)]

use serde::Deserialize;

/// Vars discriminant for `Vars::A_B_GAA_GAB_GBB` (inlen=5).
/// Mirrors `xcfun_core::Vars::A_B_GAA_GAB_GBB as u32`.
pub const VARS_A_B_GAA_GAB_GBB: u32 = 6;

/// Fixture record format (jsonl line). 5-element input matches the
/// `Vars::A_B_GAA_GAB_GBB` layout: `[a, b, gaa, gab, gbb]`.
#[derive(Deserialize, Debug, Clone)]
pub struct FixtureRecord {
    pub input: [f64; 5],
    pub expected_energy: f64,
}

/// Tolerance for the regression-detector assertion. Set to 1e-12 to
/// accommodate any sub-ULP drift between cubecl-cpu launches on the same
/// host (the kernel is deterministic; 1e-12 is a safety margin against
/// platform/arch-dependent libm differences).
///
/// When `xcfun-master/` is restored and the fixture is re-baselined to C++
/// truth, this constant should drop to the strict 1e-13 plan target.
pub const REL_TOL: f64 = 1.0e-12;

/// Path to the fixture jsonl. Cargo sets cwd to the package root for
/// integration tests, so we go two `..` up to reach the workspace root.
pub fn fixture_path(name: &str) -> String {
    format!("../../validation/fixtures/d19_n1/{}_baseline.jsonl", name)
}

/// Load fixture records from a jsonl file. Skips blank lines.
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
