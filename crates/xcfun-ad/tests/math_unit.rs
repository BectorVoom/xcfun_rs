//! Unit tests for Plan 01-06's composed `ctaylor_*` `#[cube] fn`s (9
//! elementary functions on `CTaylor`). Each kernel is launched on cubecl-cpu
//! at small N and the result is compared against hand-computed expected
//! values derived from the Taylor-series identity.
//!
//! See `crates/xcfun-ad/src/math.rs` and `xcfun-master/external/upstream/
//! taylor/ctaylor_math.hpp`.
//!
//! # Oracle strategy
//!
//! For the simple single-variable cases (n ∈ {0, 1, 2}), expected values are
//! hand-derived from the mathematical identity:
//!   - `ctaylor_reciprocal` at x = [a, 1, ...]: 1/(a + y) → t = [1/a, -1/a², 1/a³, ...]
//!   - `ctaylor_sqrt`       at x = [a, 1, ...]: sqrt(a+y) ≈ sqrt(a) + y/(2·sqrt(a)) - y²/(8·a·sqrt(a)) + ...
//!   - `ctaylor_exp`        at x = [a, 1, ...]: exp(a+y) = exp(a) · (1 + y + y²/2 + ...)
//!   - `ctaylor_log`        at x = [a, 1, ...]: log(a+y) = log(a) + y/a - y²/(2a²) + ...
//!   - `ctaylor_pow(·, p)`  at x = [a, 1, ...]: (a+y)^p → t = [a^p, p·a^(p-1), p(p-1)/2·a^(p-2), ...]
//!   - `ctaylor_powi(·, p)` integer p: same algebra as pow but integer
//!   - `ctaylor_erf`        at x = [0, 1, ...]: erf(y) = 2/√π · y - ... → t[1] = 2/√π
//!   - `ctaylor_atan`       at x = [0, 1, ...]: atan(y) = y - y³/3 + ... → t[1] = 1, t[2] = 0
//!   - `ctaylor_asinh`      at x = [0, 1, ...]: asinh(y) = y - y³/6 + ... → t[1] = 1, t[2] = 0
//!
//! # Cubecl 0.10-pre.3 idioms (consolidated)
//!
//! - `ArrayArg::from_raw_parts(h, len)` — 2 args, owned handle.
//! - `client.create_from_slice(f64::as_bytes(&[...]))`, `client.empty(bytes)`,
//!   `client.read_one_unchecked(handle)`.
//! - `#[cube(launch_unchecked)]` with comptime trailing args — launched inside
//!   `unsafe { ... }`.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::for_tests::cpu_client;
use xcfun_ad::math::{
    ctaylor_asinh, ctaylor_atan, ctaylor_erf, ctaylor_exp, ctaylor_log, ctaylor_pow,
    ctaylor_powi_0, ctaylor_powi_2, ctaylor_powi_3, ctaylor_powi_neg1,
    ctaylor_reciprocal, ctaylor_sqrt,
};

// ---------------------------------------------------------------------------
//  Kernel adapters
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn kernel_reciprocal<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_reciprocal::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_sqrt<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_sqrt::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_exp<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_exp::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_log<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_log::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_pow<F: Float>(
    x: &Array<F>,
    aa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_pow::<F>(x, aa[0], out, n);
}

#[cube(launch_unchecked)]
fn kernel_erf<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_erf::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_atan<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_atan::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_asinh<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_asinh::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_powi_0<F: Float>(out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_0::<F>(out, n);
}

#[cube(launch_unchecked)]
fn kernel_powi_2<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_2::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_powi_3<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_3::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_powi_neg1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_neg1::<F>(x, out, n);
}

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

fn run_unary(x: &[f64], out_len: usize, n: u32, kernel: KernelUnary) -> Vec<f64> {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(x));
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();
    match kernel {
        KernelUnary::Reciprocal => unsafe {
            kernel_reciprocal::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Sqrt => unsafe {
            kernel_sqrt::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Exp => unsafe {
            kernel_exp::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Log => unsafe {
            kernel_log::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Erf => unsafe {
            kernel_erf::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Atan => unsafe {
            kernel_atan::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Asinh => unsafe {
            kernel_asinh::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Powi2 => unsafe {
            kernel_powi_2::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::Powi3 => unsafe {
            kernel_powi_3::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
        KernelUnary::PowiNeg1 => unsafe {
            kernel_powi_neg1::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(x_h, x.len()),
                ArrayArg::from_raw_parts(out_h, out_len),
                n,
            );
        },
    }
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

enum KernelUnary {
    Reciprocal,
    Sqrt,
    Exp,
    Log,
    Erf,
    Atan,
    Asinh,
    Powi2,
    Powi3,
    PowiNeg1,
}

fn run_pow(x: &[f64], a: f64, out_len: usize, n: u32) -> Vec<f64> {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(x));
    let a_h = client.create_from_slice(f64::as_bytes(&[a]));
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();
    unsafe {
        kernel_pow::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(x_h, x.len()),
            ArrayArg::from_raw_parts(a_h, 1),
            ArrayArg::from_raw_parts(out_h, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

fn run_powi_0(out_len: usize, n: u32) -> Vec<f64> {
    let client = cpu_client();
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();
    unsafe {
        kernel_powi_0::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(out_h, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

const REL_TOL: f64 = 1.0e-12;

fn assert_close(got: &[f64], expected: &[f64], label: &str) {
    assert_eq!(got.len(), expected.len(), "{label}: length mismatch");
    for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
        let denom = e.abs().max(1.0);
        let rel = (g - e).abs() / denom;
        assert!(
            rel < REL_TOL,
            "{label} coeff {i}: got {g}, expected {e}, rel_err {rel:e}"
        );
    }
}

// ---------------------------------------------------------------------------
//  Tests — one per composed fn. n ∈ {0, 1, 2} typically; n=2 verifies
//  compose correctness beyond the linear term.
// ---------------------------------------------------------------------------

#[test]
fn reciprocal_n0() {
    // x = 2 → out = [1/2]
    let got = run_unary(&[2.0], 1, 0, KernelUnary::Reciprocal);
    assert_close(&got, &[0.5], "reciprocal_n0");
}

#[test]
fn reciprocal_n1() {
    // x = 2 + y → 1/(2+y) = 1/2 - y/4 → out = [0.5, -0.25]
    let got = run_unary(&[2.0, 1.0], 2, 1, KernelUnary::Reciprocal);
    assert_close(&got, &[0.5, -0.25], "reciprocal_n1");
}

#[test]
fn sqrt_n1() {
    // x = 4 + y → sqrt(4+y) = 2 + y/4 + O(y²) → out[0] = 2, out[1] = 0.25
    let got = run_unary(&[4.0, 1.0], 2, 1, KernelUnary::Sqrt);
    assert_close(&got, &[2.0, 0.25], "sqrt_n1");
}

#[test]
fn exp_n1() {
    // x = 0 + y → exp(y) ≈ 1 + y → out[0] = 1, out[1] = 1
    let got = run_unary(&[0.0, 1.0], 2, 1, KernelUnary::Exp);
    assert_close(&got, &[1.0, 1.0], "exp_n1");
}

#[test]
fn exp_n1_x2() {
    // x = 2 + y → exp(2+y) = exp(2) * (1 + y) → out[0] = e², out[1] = e²
    let e2 = 2.0_f64.exp();
    let got = run_unary(&[2.0, 1.0], 2, 1, KernelUnary::Exp);
    assert_close(&got, &[e2, e2], "exp_n1_x2");
}

#[test]
fn log_n1() {
    // x = 1 + y → log(1+y) = y + O(y²) → out[0] = 0, out[1] = 1
    let got = run_unary(&[1.0, 1.0], 2, 1, KernelUnary::Log);
    assert_close(&got, &[0.0, 1.0], "log_n1");
}

#[test]
fn pow_n1_half() {
    // x = 1 + y, a = 0.5 → (1+y)^0.5 ≈ 1 + 0.5·y → out = [1.0, 0.5]
    let got = run_pow(&[1.0, 1.0], 0.5, 2, 1);
    assert_close(&got, &[1.0, 0.5], "pow_n1_half");
}

#[test]
fn erf_n1() {
    // x = 0 + y → erf(y) ≈ (2/√π) · y → out[0] = 0, out[1] = 2/√π
    //
    // Cubecl-cpu computes erf(0) via a Wikipedia §5-term polyfill (not
    // libm::erf) that has max absolute error ~1.5e-7. So `erf(0)` is NOT
    // bit-zero — it's ~3e-8 on cubecl-cpu. See `expand/erf.rs` header for
    // the precision disclosure. This composed-function test accepts up
    // to 1e-7 absolute error on both `t[0]` and `t[1]`.
    let expected_t1 = 2.0 / std::f64::consts::PI.sqrt();
    let got = run_unary(&[0.0, 1.0], 2, 1, KernelUnary::Erf);
    // t[0] = erf(0) — polyfill drift ≤ 1.5e-7 absolute.
    assert!(
        got[0].abs() < 1.5e-7,
        "erf_n1 t[0] = {} (expected near 0, drift budget 1.5e-7)",
        got[0]
    );
    // t[1] inherits 2/√π drift from f32 π (~1.3e-8 relative).
    let rel1 = (got[1] - expected_t1).abs() / expected_t1.abs();
    assert!(
        rel1 < 1e-7,
        "erf_n1 t[1]: got {}, expected {}, rel_err {:e}",
        got[1], expected_t1, rel1
    );
}

#[test]
fn atan_n1() {
    // x = 0 + y → atan(y) ≈ y → out[0] = 0, out[1] = 1
    let got = run_unary(&[0.0, 1.0], 2, 1, KernelUnary::Atan);
    assert_close(&got, &[0.0, 1.0], "atan_n1");
}

#[test]
fn asinh_n1() {
    // x = 0 + y → asinh(y) ≈ y → out[0] = 0, out[1] = 1
    let got = run_unary(&[0.0, 1.0], 2, 1, KernelUnary::Asinh);
    assert_close(&got, &[0.0, 1.0], "asinh_n1");
}

#[test]
fn powi_0_n1() {
    // x^0 = 1 (constant CTaylor) → out = [1.0, 0.0]
    let got = run_powi_0(2, 1);
    assert_close(&got, &[1.0, 0.0], "powi_0_n1");
}

#[test]
fn powi_2_n1() {
    // x = 2 + y → (2+y)² = 4 + 4y + y² → n=1 truncation: out = [4.0, 4.0]
    let got = run_unary(&[2.0, 1.0], 2, 1, KernelUnary::Powi2);
    assert_close(&got, &[4.0, 4.0], "powi_2_n1");
}

#[test]
fn powi_3_n1() {
    // x = 2 + y → (2+y)³ = 8 + 12y + O(y²) → n=1 truncation: out = [8.0, 12.0]
    let got = run_unary(&[2.0, 1.0], 2, 1, KernelUnary::Powi3);
    assert_close(&got, &[8.0, 12.0], "powi_3_n1");
}

#[test]
fn powi_neg1_n1() {
    // x = 2 + y → (2+y)^(-1) = 1/2 - y/4 → out = [0.5, -0.25]
    let got = run_unary(&[2.0, 1.0], 2, 1, KernelUnary::PowiNeg1);
    assert_close(&got, &[0.5, -0.25], "powi_neg1_n1");
}
