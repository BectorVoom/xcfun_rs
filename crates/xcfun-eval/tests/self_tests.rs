//! Tier-1 self-tests — validate each LDA functional against its upstream
//! `test_in` / `test_out` data at `desc.test_threshold`. Source: CONTEXT D-16 +
//! REQUIREMENTS ACC-04.
//!
//! # Coverage
//! Loops over `FUNCTIONAL_DESCRIPTORS.iter().filter(|d| d.test_in.is_some())`.
//! For Phase 2 Plan 02-04's 9 LDAs, the following have upstream test data:
//!   - XC_SLATERX  (order 2, threshold 1e-11)
//!   - XC_VWN5C    (order 2, threshold 1e-11)
//!   - XC_LDAERFX  (order 2, threshold 1e-7, D-24 override)
//!   - XC_LDAERFC  (order 2, threshold 1e-7, D-24 override)
//!   - XC_TFK      (order 1, threshold 1e-5)   — inlen=2, outlen=3, not 6
//!   - XC_PW92C    (order 2, threshold 1e-11)
//!   - XC_PZ81C    (order 2, threshold 1e-11)
//!
//! Upstream-no-test-data (skipped via `test_in.is_some()`):
//!   - XC_VWN3C (LDA-02)       — ENERGY_FUNCTION(...) w/o test_in block
//!   - XC_LDAERFC_JT (LDA-08)  — ENERGY_FUNCTION(ldaerfc_jt) on one line
//!   - XC_TW (LDA-09 part 2)   — tw.cpp no upstream test_in (Plan 02-05)
//!   - XC_VWK (LDA-10)         — vwk.cpp no upstream test_in (Plan 02-05)
//!
//! Tier-2 (Plan 02-06) covers all 11 LDAs against a C++ runtime evaluation grid.
//!
//! # Upstream test_out layout (verified by manual inspection of the upstream
//! FUNCTIONAL macros — see e.g. slaterx.cpp:31-37):
//!   - All upstream LDA test cases use `XC_A_B` variant, inlen = 2.
//!   - Order 1 → outlen = 3 (1 + 2 = energy + 2 first-derivs).
//!   - Order 2 → outlen = 6 (1 + 2 + 3 = energy + 2 first + 3 second).
//! The registry's `test_outlen` field captures the observed length.

#![cfg(feature = "testing")]

use xcfun_core::{FUNCTIONAL_DESCRIPTORS, Mode, taylorlen};
use xcfun_eval::Functional;

/// Reconcile upstream `test_order` against observed `test_out.len()` — some
/// upstream macros (e.g. tfk.cpp) declare `order=1` in the macro but provide
/// order-2 reference data (6 values for inlen=2). We use `test_out.len()` as
/// the authoritative signal, finding the order k such that
/// `taylorlen(inlen, k) == test_out.len()`.
///
/// This compensates for the regen-registry extractor's literal reading of the
/// macro's order field; fixing the extractor is deferred to a future task.
fn infer_order_from_outlen(inlen: usize, outlen: usize) -> Option<u32> {
    for k in 0_u32..=2 {
        if taylorlen(inlen, k as usize) == outlen {
            return Some(k);
        }
    }
    None
}

#[test]
fn tier1_self_tests_pass() {
    let mut tested = 0_usize;
    let mut failures: Vec<String> = Vec::new();

    for desc in FUNCTIONAL_DESCRIPTORS.iter() {
        let Some(test_in) = desc.test_in else { continue };
        let Some(test_out) = desc.test_out else { continue };
        let Some(test_threshold) = desc.test_threshold else {
            continue;
        };
        let Some(test_vars) = desc.test_vars else { continue };
        let Some(_declared_order) = desc.test_order else { continue };

        // Prefer the observed order from test_out.len() (see infer_order_from_outlen
        // for the TFK reconciliation rationale).
        let inlen = test_vars.input_len();
        let Some(test_order) = infer_order_from_outlen(inlen, test_out.len()) else {
            continue;
        };

        // Phase 2 supports orders 0..=2 only (CONTEXT D-23).
        if test_order > 2 {
            continue;
        }

        // Phase 2 dispatch only wires 9 LDAs; skip functionals whose kernel isn't wired yet.
        // (Stubs are filtered out by `test_in.is_some()` since the extractor leaves stubs
        // with test_in=None, but other populated entries like XC_PBEC don't yet have a kernel.)
        use xcfun_eval::dispatch::supports;
        if !supports(desc.id) {
            continue;
        }

        // Leak a static slice for the per-descriptor weights — the Functional struct
        // (D-21 minimal slice) uses `&'static [(FunctionalId, f64)]`. In a test
        // context this is acceptable (one leak per LDA ≈ 16 bytes × 9 = 144 bytes
        // of test-binary bloat).
        let weights: &'static [(_, _)] = Box::leak(Box::new([(desc.id, 1.0_f64)]));
        let fun = Functional {
            weights,
            vars: test_vars,
            mode: Mode::PartialDerivatives,
            order: test_order,
        };

        let mut output = vec![0.0_f64; test_out.len()];
        if let Err(e) = fun.eval(test_in, &mut output) {
            failures.push(format!("{:?}: eval failed with {:?}", desc.id, e));
            continue;
        }

        let mut any_mismatch = false;
        for (i, (got, want)) in output.iter().zip(test_out.iter()).enumerate() {
            let denom = want.abs().max(1.0);
            let rel = (got - want).abs() / denom;
            if rel > test_threshold {
                if !any_mismatch {
                    failures.push(format!(
                        "{:?} (threshold {:.0e}) — element mismatches:",
                        desc.id, test_threshold
                    ));
                    any_mismatch = true;
                }
                failures.push(format!(
                    "  [{}] got={:.12e}, want={:.12e}, rel={:.2e}",
                    i, got, want, rel
                ));
            }
        }
        tested += 1;
    }

    if !failures.is_empty() {
        panic!(
            "tier-1 self-tests FAILED ({} functional(s) with issues; {} passed):\n{}",
            failures.iter().filter(|l| !l.starts_with("  ")).count(),
            tested.saturating_sub(
                failures
                    .iter()
                    .filter(|l| !l.starts_with("  "))
                    .count()
            ),
            failures.join("\n")
        );
    }

    // Sanity check: Phase 2 Plan 02-04 wires 9 LDAs; 7 have upstream test data
    // (SLATERX, VWN5C, LDAERFX, LDAERFC, TFK, PW92C, PZ81C). VWN3C and LDAERFC_JT
    // are upstream-no-test-data.
    assert!(
        tested >= 7,
        "Expected at least 7 LDA tier-1 tests wired; got {} — check Plan 02-04 dispatch arms",
        tested
    );
}
