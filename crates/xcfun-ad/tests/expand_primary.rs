//! Integration tests for Plan 01-03's six primary `*_expand` `#[cube] fn`s
//! (`inv`, `exp`, `log`, `pow`, `sqrt`, `cbrt`). Each expansion is launched
//! on `cubecl-cpu` at order `n = 3` across 3 representative inputs; the
//! result is compared against a hand-computed / host-libm reference at
//! relative error ≤ 1e-13.
//!
//! See Plan 01-03 `<behavior>` for the input grid and expected-value
//! derivations. Plan 01-05 is responsible for C++ golden-fixture parity
//! at tighter tolerances.
//!
//! # Oracle strategy
//!
//! - `inv_expand`: exact rationals `(-1)^i / a^(i+1)` computed host-side
//!   in `f64` as `f64::powi`; exact for small integer powers.
//! - `exp_expand`: `exp(x0) / i!` computed host-side via `f64::exp` and
//!   a direct factorial loop. This matches what the kernel does
//!   step-by-step, modulo one extra `*` and `/`.
//! - `log_expand`: matches the kernel recurrence host-side using
//!   `f64::ln(x0)` + the same integer sign factor.
//! - `pow_expand`, `sqrt_expand`, `cbrt_expand`: use the same recurrence
//!   the kernel uses (same operation order) on host `f64`. Kernel-vs-host
//!   delta should be ≤ 1 ULP on cubecl-cpu, well within 1e-13 relative.
//!
//! # Cubecl 0.10-pre.3 launch idiom
//!
//! Pattern identical to `tests/ctaylor_unit.rs`: one `#[cube(launch_unchecked)]`
//! adapter fn per library fn, with `#[comptime] n` as the trailing scalar
//! argument. `ArrayArg::from_raw_parts(handle, len)` — 2 args, owned
//! handle, no turbofish (cubecl 0.10-pre.3 delta).

#![cfg(feature = "testing")]
// Host-side reference fns intentionally use `for i in 1..=n` + index
// to mirror the kernel operation order 1:1. Switching to enumerate()
// would obscure the kernel/host parity — this is the point of the
// host references.
#![allow(clippy::needless_range_loop)]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::expand::{cbrt, exp, inv, log, pow, sqrt};
use xcfun_ad::for_tests::cpu_client;

// ---------------------------------------------------------------------------
//  Kernel adapters — one per fn. `n` passed as trailing comptime arg.
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn kernel_inv_expand<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    // scalars[0] = a
    inv::inv_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_exp_expand<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    // scalars[0] = x0
    exp::exp_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_log_expand<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    // scalars[0] = x0
    log::log_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_pow_expand<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    // scalars = [x0, a]
    pow::pow_expand::<F>(t, scalars[0], scalars[1], n);
}

#[cube(launch_unchecked)]
fn kernel_sqrt_expand<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    // scalars[0] = x0
    sqrt::sqrt_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_cbrt_expand<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    // scalars[0] = x0
    cbrt::cbrt_expand::<F>(t, scalars[0], n);
}

// ---------------------------------------------------------------------------
//  Generic launch helper for the above kernel shape (scalars in, t out).
// ---------------------------------------------------------------------------

fn run_expand<L>(scalars: &[f64], out_len: usize, launcher: L) -> Vec<f64>
where
    L: FnOnce(
        &cubecl::prelude::ComputeClient<CpuRuntime>,
        cubecl::server::Handle,
        cubecl::server::Handle,
    ),
{
    let client = cpu_client();
    let s_h = client.create_from_slice(f64::as_bytes(scalars));
    let t_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = t_h.clone();
    launcher(client, s_h, t_h);
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

// ---------------------------------------------------------------------------
//  Relative-error assertion. 1e-13 gate per Plan 01-03 success criteria.
// ---------------------------------------------------------------------------

const REL_TOL: f64 = 1.0e-13;

fn assert_close(got: &[f64], expected: &[f64], label: &str) {
    assert_eq!(
        got.len(),
        expected.len(),
        "{label}: length mismatch got={} expected={}",
        got.len(),
        expected.len()
    );
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
//  Host reference builders. The recurrences match the kernel recurrences
//  line-for-line so operation order is identical (cubecl-cpu lowers to
//  libm; host f64 goes to libm directly — kernel-vs-host drift ≤ 1 ULP).
// ---------------------------------------------------------------------------

fn host_inv(a: f64, n: usize) -> Vec<f64> {
    // t[i] = (-1)^i / a^(i+1), computed via the same recurrence as kernel.
    let mut t = vec![0.0_f64; n + 1];
    t[0] = 1.0 / a;
    for i in 1..=n {
        t[i] = -t[i - 1] * t[0];
    }
    t
}

fn host_exp(x0: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    let mut ifac = 1.0_f64;
    t[0] = x0.exp();
    for i in 1..=n {
        ifac *= i as f64;
        t[i] = t[0] / ifac;
    }
    t
}

fn host_log(x0: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    t[0] = x0.ln();
    let x0inv = 1.0 / x0;
    let mut xn = x0inv;
    for i in 1..=n {
        let i_f = i as f64;
        let sign = (2 * ((i as i32) & 1) - 1) as f64;
        t[i] = (xn / i_f) * sign;
        xn *= x0inv;
    }
    t
}

fn host_pow(x0: f64, a: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    t[0] = x0.powf(a);
    let x0inv = 1.0 / x0;
    for i in 1..=n {
        let i_f = i as f64;
        t[i] = t[i - 1] * x0inv * (a - i_f + 1.0) / i_f;
    }
    t
}

fn host_sqrt(x0: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    t[0] = x0.sqrt();
    let x0inv = 1.0 / x0;
    for i in 1..=n {
        let i_f = i as f64;
        let num = 3.0 * x0inv;
        let den = 2.0 * i_f;
        let quot = num / den;
        let factor = quot - x0inv;
        t[i] = t[i - 1] * factor;
    }
    t
}

fn host_cbrt(x0: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    // Match kernel exactly (Plan 07-00 Task 0.3, commits 92b1a4f + 1edb1b0):
    // seed `t[0]` from `powf(f64 1/3)` then refine with two Newton iterations
    //     y_{k+1} = (2·y_k + x0 / y_k²) / 3
    // which converges from the 1–2 ULP `powf` seed to ≤ 1 ULP correctly-
    // rounded cbrt — i.e. matches libm `cbrt` to within the f64 contract
    // while staying inside the cubecl `Float` trait surface (no `.cbrt()`).
    // Operation order is line-for-line identical to `cbrt_expand` in
    // `crates/xcfun-ad/src/expand/cbrt.rs:63-69` so kernel-vs-host drift
    // stays ≤ 1 ULP, well within the 1e-13 relative gate.
    let y0 = x0.powf(1.0_f64 / 3.0_f64);
    let y0_sq = y0 * y0;
    let y1 = (2.0 * y0 + x0 / y0_sq) / 3.0;
    let y1_sq = y1 * y1;
    t[0] = (2.0 * y1 + x0 / y1_sq) / 3.0;
    let x0inv = 1.0 / x0;
    for i in 1..=n {
        let i_f = i as f64;
        let num = 4.0 * x0inv;
        let den = 3.0 * i_f;
        let quot = num / den;
        let factor = quot - x0inv;
        t[i] = t[i - 1] * factor;
    }
    t
}

// ---------------------------------------------------------------------------
//  Launch + compare helper per expansion.
// ---------------------------------------------------------------------------

fn run_inv_expand(a: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_expand(&[a], out_len, |client, sh, oh| unsafe {
        kernel_inv_expand::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_exp_expand(x0: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_expand(&[x0], out_len, |client, sh, oh| unsafe {
        kernel_exp_expand::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_log_expand(x0: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_expand(&[x0], out_len, |client, sh, oh| unsafe {
        kernel_log_expand::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_pow_expand(x0: f64, a: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_expand(&[x0, a], out_len, |client, sh, oh| unsafe {
        kernel_pow_expand::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 2),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_sqrt_expand(x0: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_expand(&[x0], out_len, |client, sh, oh| unsafe {
        kernel_sqrt_expand::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_cbrt_expand(x0: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_expand(&[x0], out_len, |client, sh, oh| unsafe {
        kernel_cbrt_expand::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

// ===========================================================================
//  inv_expand tests
// ===========================================================================

#[test]
fn inv_expand_a_0_1() {
    let got = run_inv_expand(0.1, 3);
    let expected = host_inv(0.1, 3);
    assert_close(&got, &expected, "inv_expand a=0.1");
}

#[test]
fn inv_expand_a_1_0() {
    let got = run_inv_expand(1.0, 3);
    // Closed form: [+1, -1, +1, -1]
    let expected = [1.0_f64, -1.0, 1.0, -1.0];
    assert_close(&got, &expected, "inv_expand a=1.0");
}

#[test]
fn inv_expand_a_10_0() {
    let got = run_inv_expand(10.0, 3);
    let expected = host_inv(10.0, 3);
    assert_close(&got, &expected, "inv_expand a=10.0");
}

// ===========================================================================
//  exp_expand tests
// ===========================================================================

#[test]
fn exp_expand_x0_neg1() {
    let got = run_exp_expand(-1.0, 3);
    let expected = host_exp(-1.0, 3);
    assert_close(&got, &expected, "exp_expand x0=-1.0");
}

#[test]
fn exp_expand_x0_0() {
    let got = run_exp_expand(0.0, 3);
    let expected = [1.0_f64, 1.0, 0.5, 1.0 / 6.0];
    assert_close(&got, &expected, "exp_expand x0=0.0");
}

#[test]
fn exp_expand_x0_2() {
    let got = run_exp_expand(2.0, 3);
    let expected = host_exp(2.0, 3);
    assert_close(&got, &expected, "exp_expand x0=2.0");
}

// ===========================================================================
//  log_expand tests
// ===========================================================================

#[test]
fn log_expand_x0_0_1() {
    let got = run_log_expand(0.1, 3);
    let expected = host_log(0.1, 3);
    assert_close(&got, &expected, "log_expand x0=0.1");
}

#[test]
fn log_expand_x0_1() {
    let got = run_log_expand(1.0, 3);
    // Closed form: [0, 1, -1/2, 1/3]
    let expected = [0.0_f64, 1.0, -0.5, 1.0 / 3.0];
    assert_close(&got, &expected, "log_expand x0=1.0");
}

#[test]
fn log_expand_x0_10() {
    let got = run_log_expand(10.0, 3);
    let expected = host_log(10.0, 3);
    assert_close(&got, &expected, "log_expand x0=10.0");
}

// ===========================================================================
//  pow_expand tests
// ===========================================================================

#[test]
fn pow_expand_x0_1_a_half() {
    // (1 + x)^0.5 = 1 + x/2 - x^2/8 + x^3/16
    let got = run_pow_expand(1.0, 0.5, 3);
    let expected = [1.0_f64, 0.5, -0.125, 0.0625];
    assert_close(&got, &expected, "pow_expand x0=1.0 a=0.5");
}

#[test]
fn pow_expand_x0_2_a_1_5() {
    let got = run_pow_expand(2.0, 1.5, 3);
    let expected = host_pow(2.0, 1.5, 3);
    assert_close(&got, &expected, "pow_expand x0=2.0 a=1.5");
}

#[test]
fn pow_expand_x0_10_a_neg1() {
    // (10+x)^-1 = 1/(10+x) — matches inv_expand(a=10) coefficient-for-coefficient.
    let got = run_pow_expand(10.0, -1.0, 3);
    let expected = host_inv(10.0, 3);
    assert_close(&got, &expected, "pow_expand x0=10.0 a=-1.0");
}

// ===========================================================================
//  sqrt_expand tests
// ===========================================================================

#[test]
fn sqrt_expand_x0_0_1() {
    let got = run_sqrt_expand(0.1, 3);
    let expected = host_sqrt(0.1, 3);
    assert_close(&got, &expected, "sqrt_expand x0=0.1");
}

#[test]
fn sqrt_expand_x0_1() {
    // sqrt(1+x) = 1 + x/2 - x^2/8 + x^3/16
    let got = run_sqrt_expand(1.0, 3);
    let expected = [1.0_f64, 0.5, -0.125, 0.0625];
    assert_close(&got, &expected, "sqrt_expand x0=1.0");
}

#[test]
fn sqrt_expand_x0_10() {
    let got = run_sqrt_expand(10.0, 3);
    let expected = host_sqrt(10.0, 3);
    assert_close(&got, &expected, "sqrt_expand x0=10.0");
}

// ===========================================================================
//  cbrt_expand tests
// ===========================================================================

#[test]
fn cbrt_expand_x0_0_1() {
    let got = run_cbrt_expand(0.1, 3);
    let expected = host_cbrt(0.1, 3);
    assert_close(&got, &expected, "cbrt_expand x0=0.1");
}

#[test]
fn cbrt_expand_x0_1() {
    // cbrt(1+x) = 1 + x/3 - x^2/9 + 5x^3/81  (closed form)
    let got = run_cbrt_expand(1.0, 3);
    let expected = [1.0_f64, 1.0 / 3.0, -1.0 / 9.0, 5.0 / 81.0];
    assert_close(&got, &expected, "cbrt_expand x0=1.0");
}

#[test]
fn cbrt_expand_x0_10() {
    let got = run_cbrt_expand(10.0, 3);
    let expected = host_cbrt(10.0, 3);
    assert_close(&got, &expected, "cbrt_expand x0=10.0");
}
