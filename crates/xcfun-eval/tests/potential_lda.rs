//! Mode::Potential — LDA path tests (Phase 3 plan 03-05 Task 1).
//!
//! Covers the LDA-only divergence-free potential path
//! (`launch_potential_lda`, N=1):
//!   - SLATERX with Vars::A_B (open-shell)
//!   - SLATERX with Vars::A_B_2ND_TAYLOR (open-shell, GGA-vars input layout)
//!
//! For SLATERX the analytic potential is
//!   `pot_α = -(4/3) · c_slater · a^(1/3)`,
//!   `pot_β = -(4/3) · c_slater · b^(1/3)`,
//! with `c_slater = (81/(32π))^(1/3) ≈ 0.9305257363491002`.

#![cfg(feature = "testing")]

use approx::assert_relative_eq;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;
use xcfun_eval::functional::DEFAULT_SETTINGS;

const C_SLATER: f64 = 0.930_525_736_349_100_2_f64;

#[test]
fn slaterx_potential_a_b_matches_analytic() {
    // Vars::A_B (inlen = 2; nspin = 2; output = [energy, pot_α, pot_β]).
    let f = Functional {
        weights: &[(FunctionalId::XC_SLATERX, 1.0)],
        vars: Vars::A_B,
        mode: Mode::Potential,
        order: 0,
        settings: DEFAULT_SETTINGS,
    };

    let a = 0.7_f64;
    let b = 0.4_f64;
    let mut out = vec![0.0_f64; 3];
    f.eval(&[a, b], &mut out).expect("Mode::Potential LDA eval");

    // SLATERX energy:  E = -c_slater · (a^(4/3) + b^(4/3))
    // SLATERX potential (∂E/∂a):  -(4/3) · c_slater · a^(1/3)
    let want_energy =
        -C_SLATER * (a.powf(4.0 / 3.0) + b.powf(4.0 / 3.0));
    let want_pot_a = -(4.0 / 3.0) * C_SLATER * a.cbrt();
    let want_pot_b = -(4.0 / 3.0) * C_SLATER * b.cbrt();

    assert_relative_eq!(out[0], want_energy, max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(out[1], want_pot_a, max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(out[2], want_pot_b, max_relative = 1e-12, epsilon = 1e-20);
}

#[test]
fn slaterx_potential_a_b_2nd_taylor_matches_analytic() {
    // Vars::A_B_2ND_TAYLOR (inlen = 20; nspin = 2). All 18 derivative slots
    // (1..9, 11..19) zero — pure scalar-density input on the canonical
    // 2ND_TAYLOR Vars used by Mode::Potential GGA dispatch. SLATERX (LDA)
    // ignores the gradient slots.
    let f = Functional {
        weights: &[(FunctionalId::XC_SLATERX, 1.0)],
        vars: Vars::A_B_2ND_TAYLOR,
        mode: Mode::Potential,
        order: 0,
        settings: DEFAULT_SETTINGS,
    };

    let a = 0.5_f64;
    let b = 0.3_f64;
    let mut input = vec![0.0_f64; 20];
    input[0] = a;
    input[10] = b;
    let mut out = vec![0.0_f64; 3];
    f.eval(&input, &mut out)
        .expect("Mode::Potential SLATERX A_B_2ND_TAYLOR eval");

    let want_energy =
        -C_SLATER * (a.powf(4.0 / 3.0) + b.powf(4.0 / 3.0));
    let want_pot_a = -(4.0 / 3.0) * C_SLATER * a.cbrt();
    let want_pot_b = -(4.0 / 3.0) * C_SLATER * b.cbrt();

    assert_relative_eq!(out[0], want_energy, max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(out[1], want_pot_a, max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(out[2], want_pot_b, max_relative = 1e-12, epsilon = 1e-20);
}
