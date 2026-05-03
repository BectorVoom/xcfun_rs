//! Mode::Potential — GGA path tests (Phase 3 plan 03-05 Task 2).
//!
//! Covers the full two-pass GGA-potential pipeline:
//!   - Pass 1: launch_potential_lda (N=1) populates `out[0]=energy`
//!     and `out[j+1] = ∂E/∂ρ` (LDA-direct term).
//!   - Pass 2: launch_potential_gga (N=2) subtracts the three-direction
//!     divergence `∇·(∂E/∂g)` from `out[j+1]` IN PLACE
//!     (XCFunctional.cpp:720 / :785-787).
//!
//! Numerical strategies used here:
//!   - PBEX with zero-gradient input → kernel evaluates `s² = 0`; the
//!     PBEX enhancement collapses to its LDA prefactor and we can compare
//!     the energy + α-potential against the analytic Slater-like value at
//!     `s = 0`. This exercises the full N=1 + N=2 launch chain end-to-end.
//!   - SLATERX driven through Vars::A_B_2ND_TAYLOR — pure-LDA functional
//!     so the GGA pass is short-circuited (Dependency does not include
//!     GRADIENT); covered already in `potential_lda.rs`.

#![cfg(feature = "testing")]

use approx::assert_relative_eq;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;
use xcfun_eval::functional::DEFAULT_SETTINGS;

/// Smoke test: PBEX at a non-zero density with ZERO gradient + ZERO
/// 2nd-order spatial Hessian. The divergence ∇·(∂E/∂g) at zero
/// gradient is well-defined for PBEX, so the GGA path should execute
/// without panic and produce a finite output.
///
/// PBEX at s=0 reduces to the LDA Slater-like exchange:
///     E = -c_slater · (a^(4/3) + b^(4/3))
///     pot_α = -(4/3) · c_slater · a^(1/3)
/// because the PBEX enhancement F(s) → 1 as s → 0.
///
/// We assert the energy at strict 1e-12.
/// The α/β potentials should agree with the Slater LDA baseline at
/// ZERO gradient too (the divergence subtract evaluates to zero
/// because ∂F/∂g vanishes when the second derivative of F w.r.t. g
/// is multiplied by zero density-Hessian — verified empirically).
#[test]
fn pbex_potential_zero_gradient_matches_slater_energy() {
    let f = Functional {
        weights: &[(FunctionalId::XC_PBEX, 1.0)],
        vars: Vars::A_B_2ND_TAYLOR,
        mode: Mode::Potential,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };

    let a = 0.4_f64;
    let b = 0.25_f64;
    // Vars::A_B_2ND_TAYLOR: 20 input slots (α: 0..9, β: 10..19).
    let mut input = vec![0.0_f64; 20];
    input[0] = a;
    input[10] = b;
    // All 9+9 derivative slots remain zero (zero-gradient + zero-Hessian).
    let mut out = vec![0.0_f64; 3];
    f.eval(&input, &mut out)
        .expect("Mode::Potential PBEX A_B_2ND_TAYLOR eval");

    // PBEX energy at s=0 reduces to the LDA Slater exchange formula
    // (PBEX enhancement F(s=0) = 1; xcfun-master/src/functionals/pbex.cpp).
    const C_SLATER: f64 = 0.930_525_736_349_100_2_f64;
    let want_energy = -C_SLATER * (a.powf(4.0 / 3.0) + b.powf(4.0 / 3.0));
    assert_relative_eq!(out[0], want_energy, max_relative = 1e-12, epsilon = 1e-20);

    // At zero gradient the GGA divergence term cleanly evaluates to zero
    // because ∂²E/∂ρ∂g vanishes when the gradient input is zero across
    // all three directions. The LDA-direct term equals the Slater LDA
    // potential.
    let want_pot_a = -(4.0 / 3.0) * C_SLATER * a.cbrt();
    let want_pot_b = -(4.0 / 3.0) * C_SLATER * b.cbrt();
    assert_relative_eq!(out[1], want_pot_a, max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(out[2], want_pot_b, max_relative = 1e-12, epsilon = 1e-20);
}

/// Sanity check that `eval_setup` rejects a GGA functional with non-2ND_TAYLOR
/// Vars (the InvalidVars rejection path required by D-13 + plan 03-01).
#[test]
fn pbex_potential_non_2nd_taylor_vars_rejects() {
    let f = Functional {
        weights: &[(FunctionalId::XC_PBEX, 1.0)],
        vars: Vars::A_B_GAA_GAB_GBB,
        mode: Mode::Potential,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };
    let mut out = vec![0.0_f64; 3];
    let err = f.eval(&[0.4, 0.25, 0.0, 0.0, 0.0], &mut out);
    assert!(
        matches!(err, Err(xcfun_core::XcError::InvalidVars { .. })),
        "expected InvalidVars, got {:?}",
        err
    );
}
