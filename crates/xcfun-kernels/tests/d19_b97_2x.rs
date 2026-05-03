//! Plan 06-N3 D-19 small-magnitude residual sweep — `b97_2x` per-functional
//! unit test (B-6 pattern from Plan 06-N1; pure-verification per I-3 Option B).
//!
//! Loads `validation/fixtures/d19_n3/b97_2x_baseline.jsonl` and re-runs
//! `Functional::eval` at order 3 across the curated density-strata grid.
//! Asserts strict 1e-13 vs the regression snapshot.
//!
//! - **PASSES** if the snapshot bit-stable holds (auto-tightening
//!   hypothesis is preserved by future kernel-edit plans, OR the
//!   functional was never tightened — both are valid stable states for
//!   this regression test).
//! - **FAILS** if a future kernel-edit plan changes the output at any
//!   curated point — the failure surfaces the drift; the orchestrator
//!   then either updates the fixture (citing the new ground truth in the
//!   commit message) or reverts the kernel edit.
//!
//! See `tests/common/mod.rs` for snapshot semantics + the
//! NEEDS-VERIFICATION verdict in `06-N3-SUMMARY.md`.

#![cfg(feature = "testing")]

mod common;

use xcfun_core::FunctionalId;

#[test]
fn d19_b97_2x_strict_1e_13_at_failing_strata() {
    common::run_d19_n3_contract("b97_2x", FunctionalId::XC_B97_2X);
}
