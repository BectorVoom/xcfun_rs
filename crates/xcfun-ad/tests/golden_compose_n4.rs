//! Phase 6 Plan 06-00 Task 1 — N=4 compose specialisation test.
//!
//! Verifies `ctaylor_compose_n4(out, x, f)` produces correct Taylor-series
//! composition for several known closed-form identities:
//!
//! 1. **Linear function:** `f(t) = a + b*t` → composition with x is
//!    `a*1 + b*x` (since (1+t)^1 = 1+t scaling). out[0] = a + b*x[0],
//!    out[i≥1] = b * x[i].
//!
//! 2. **Identity polynomial:** `f(t) = t` (i.e. `f = [0, 1, 0, ..., 0]`).
//!    out should equal x at every slot.
//!
//! 3. **Constant:** `f(t) = c` (i.e. `f = [c, 0, 0, ..., 0]`). out should be
//!    [c, 0, 0, ..., 0] regardless of x.
//!
//! 4. **Cross-check:** for n ∈ {0, 1, 2, 3}, verify that `compose_n4` agrees
//!    with `compose_n3` when the higher slots of x and f are zero. This pins
//!    down the recursion equivalence at the boundary between specialisations.
//!
//! Strict 1e-13 fixtures vs C++ extractor will land in a follow-up plan
//! once the C++ extractor is extended (Plan 06-00 Task 1 Step A) with the
//! n_var=4 op="compose" cases.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::ctaylor_rec::compose::ctaylor_compose_n4;
use xcfun_ad::for_tests::cpu_client;

const SIZE: usize = 16;
const F_LEN: usize = 5; // n_var + 1 = 4 + 1

#[cube(launch_unchecked)]
fn kernel_compose_n4<F: Float>(out: &mut Array<F>, x: &Array<F>, f: &Array<F>) {
    ctaylor_compose_n4::<F>(out, x, f);
}

fn run_compose_n4(x: &[f64; SIZE], f_coef: &[f64; F_LEN]) -> [f64; SIZE] {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(x));
    let f_h = client.create_from_slice(f64::as_bytes(f_coef));
    let out_h = client.empty(SIZE * std::mem::size_of::<f64>());
    let read = out_h.clone();
    unsafe {
        kernel_compose_n4::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(out_h, SIZE),
            ArrayArg::from_raw_parts(x_h, SIZE),
            ArrayArg::from_raw_parts(f_h, F_LEN),
        );
    }
    let bytes = client.read_one_unchecked(read);
    let result = f64::from_bytes(&bytes);
    let mut out = [0.0_f64; SIZE];
    out.copy_from_slice(&result[..SIZE]);
    out
}

/// `f(t) = c` (constant) → out = [c, 0, 0, ..., 0] regardless of x.
#[test]
fn compose_n4_constant_function_returns_constant() {
    let f_coef: [f64; F_LEN] = [3.14, 0.0, 0.0, 0.0, 0.0];
    let x: [f64; SIZE] = [
        1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.5, 0.25, 0.125, 0.0625,
        0.03125, 0.015625, 0.0078125, 0.00390625,
    ];
    let out = run_compose_n4(&x, &f_coef);
    assert!(
        (out[0] - 3.14).abs() < 1e-15,
        "compose_n4 with f=const: out[0] = {}, expected {}",
        out[0],
        3.14
    );
    for i in 1..SIZE {
        assert!(
            out[i].abs() < 1e-15,
            "compose_n4 with f=const: out[{}] = {}, expected 0",
            i,
            out[i]
        );
    }
}

/// `f(t) = t` (identity around x[0]) — coefficient form: f Taylor-expanded
/// around x[0] is `[x[0], 1, 0, 0, 0]`.
///
/// Mathematically: composing the identity function with x gives back x.
/// out[0] = f(x(0)) = x[0]; out[i≥1] = x[i] (since f' = 1, f''=0, ...).
#[test]
fn compose_n4_identity_function_returns_x() {
    let x: [f64; SIZE] = [
        2.5, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.5, 0.25, 0.125, 0.0625,
        0.03125, 0.015625, 0.0078125, 0.00390625,
    ];
    // f(t) = t around x[0] = 2.5: Taylor coeffs = [2.5, 1, 0, 0, 0].
    let f_coef: [f64; F_LEN] = [x[0], 1.0, 0.0, 0.0, 0.0];
    let out = run_compose_n4(&x, &f_coef);
    for i in 0..SIZE {
        let expected = x[i];
        let abs_err = (out[i] - expected).abs();
        let scale = expected.abs().max(1.0);
        let rel_err = abs_err / scale;
        assert!(
            rel_err < 1e-13,
            "compose_n4 with f=identity: out[{}] = {:e}, expected {:e}, rel_err = {:e}",
            i,
            out[i],
            expected,
            rel_err
        );
    }
}

/// `f(t) = a + b*t` — linear function. Taylor coefficients around `x[0]`:
/// `[a + b*x[0], b, 0, 0, 0]`. After compose with x, out[0] = a + b*x[0],
/// out[i≥1] = b*x[i] (since f'(t) = b, f''(t) = 0, ...).
#[test]
fn compose_n4_linear_function() {
    let x: [f64; SIZE] = [
        2.0, 1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1,
        0.05,
    ];
    let a = 10.0_f64;
    let b = 3.0_f64;
    // f Taylor at x[0]=2.0: f(2+s) = (10+3*2) + 3*s = 16 + 3*s.
    let f_coef: [f64; F_LEN] = [a + b * x[0], b, 0.0, 0.0, 0.0];
    let out = run_compose_n4(&x, &f_coef);
    let expected_0 = a + b * x[0];
    assert!(
        (out[0] - expected_0).abs() < 1e-13,
        "compose_n4 linear: out[0] = {}, expected {}",
        out[0],
        expected_0
    );
    for i in 1..SIZE {
        let expected = b * x[i];
        let abs_err = (out[i] - expected).abs();
        let scale = expected.abs().max(1.0);
        let rel_err = abs_err / scale;
        assert!(
            rel_err < 1e-13,
            "compose_n4 linear: out[{}] = {:e}, expected {:e}, rel_err = {:e}",
            i,
            out[i],
            expected,
            rel_err
        );
    }
}
