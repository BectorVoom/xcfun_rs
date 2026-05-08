//! Phase 6 Plan 06-06 (D-18) — DensVars-driven dispatch.
//!
//! Resolves the Phase 5 D-14 dispatch-table constraint forward: mixed-LDA+GGA
//! aliases (b3lyp = LDA-Slater + LDA-VWN5C + GGA-Becke + GGA-LYP + EXX param;
//! camb3lyp = range-separated b3lyp variant; bp86 = additive 2-GGA-term alias
//! with VWN3C LDA component) currently route through the C++ validation
//! harness only.  D-18 widens the dispatcher: a kernel's `Dependency` mask
//! determines which `Vars` subset arms it can launch into.  In particular,
//! LDA kernels (Dependency::DENSITY only) gain launch arms at the GGA
//! Vars subset where `DENSITY ⊆ vars_dep_mask`, so mixed aliases evaluate
//! in-process at `Vars::A_B_GAA_GAB_GBB`.
//!
//! These tests assert the dispatch SHAPE: `eval(...)` returns Ok and writes
//! a finite output for each alias.  Strict 1e-13 numerical parity vs the
//! C++ baseline lives in the validation tier-2 harness (Plan 06-N1/N3); this
//! integration test is the in-process compile-time-and-run gate.

use xcfun_core::{Mode, Vars};
use xcfun_rs::Functional;

#[test]
fn b3lyp_dispatches_in_process() {
    // b3lyp resolves to: 0.80 slaterx + 0.72 beckecorrx + 0.81 lypc
    //                   + 0.19 vwn5c + 0.20 exx (parameter, not in weights)
    // Active set: {SLATERX, BECKECORRX, LYPC, VWN5C}.  SLATERX + VWN5C are
    // LDAs (Dependency::DENSITY); BECKECORRX + LYPC are GGAs (DENSITY|GRADIENT).
    let mut f = Functional::new();
    f.set("b3lyp", 1.0).unwrap();
    f.eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1)
        .expect("b3lyp eval_setup at A_B_GAA_GAB_GBB / PartialDerivatives / order 1 must succeed");
    let inlen = f.input_length();
    let outlen = f
        .output_length()
        .expect("output_length must succeed for b3lyp");
    assert_eq!(inlen, 5, "Vars::A_B_GAA_GAB_GBB has 5 input doubles");
    // taylorlen(5, 1) = 6 (energy + 5 first derivatives)
    assert_eq!(
        outlen, 6,
        "PartialDerivatives order 1 with inlen=5 emits 6 outputs"
    );

    // Density point: positive ρα, ρβ, gradient invariants.
    let input: Vec<f64> = vec![0.5, 0.5, 0.1, 0.1, 0.1];
    let mut output: Vec<f64> = vec![0.0; outlen];
    let r = f.eval(&input, &mut output);
    assert!(r.is_ok(), "b3lyp Functional::eval failed: {:?}", r);
    // Energy must be finite + nonzero (b3lyp at this point yields a small
    // negative XC energy density).
    assert!(
        output[0].is_finite(),
        "b3lyp energy is not finite: {}",
        output[0]
    );
    assert!(
        output[0] != 0.0,
        "b3lyp energy is exactly zero — likely zeroed by mis-dispatch"
    );
}

#[test]
fn camb3lyp_dispatches_in_process() {
    // camb3lyp = camcompx + camb3lyp_corr (range-separated CAM functional).
    // Mixed LDA + GGA components per aliases.cpp.
    let mut f = Functional::new();
    f.set("camb3lyp", 1.0).unwrap();
    f.eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1)
        .expect("camb3lyp eval_setup at A_B_GAA_GAB_GBB / order 1 must succeed");
    let inlen = f.input_length();
    let outlen = f.output_length().expect("camb3lyp output_length");
    assert_eq!(inlen, 5);

    let input: Vec<f64> = vec![0.5, 0.5, 0.1, 0.1, 0.1];
    let mut output: Vec<f64> = vec![0.0; outlen];
    let r = f.eval(&input, &mut output);
    assert!(r.is_ok(), "camb3lyp Functional::eval failed: {:?}", r);
    assert!(
        output[0].is_finite(),
        "camb3lyp energy is not finite: {}",
        output[0]
    );
}

#[test]
fn bp86_dispatches_in_process() {
    // bp86 = beckex + p86c + p86corrc + vwn3c (additive 2-GGA-term alias
    // with VWN3C LDA component) per aliases.cpp.
    let mut f = Functional::new();
    f.set("bp86", 1.0).unwrap();
    f.eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1)
        .expect("bp86 eval_setup at A_B_GAA_GAB_GBB / order 1 must succeed");
    let inlen = f.input_length();
    let outlen = f.output_length().expect("bp86 output_length");
    assert_eq!(inlen, 5);

    let input: Vec<f64> = vec![0.5, 0.5, 0.1, 0.1, 0.1];
    let mut output: Vec<f64> = vec![0.0; outlen];
    let r = f.eval(&input, &mut output);
    assert!(r.is_ok(), "bp86 Functional::eval failed: {:?}", r);
    assert!(
        output[0].is_finite(),
        "bp86 energy is not finite: {}",
        output[0]
    );
    assert!(
        output[0] != 0.0,
        "bp86 energy is exactly zero — likely zeroed by mis-dispatch"
    );
}
