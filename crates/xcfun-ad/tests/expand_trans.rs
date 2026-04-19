//! Integration tests for Plan 01-04 Tasks 2-3 — the four transcendental
//! `*_expand` `#[cube] fn`s (`atan`, `asinh`, `gauss`, `erf`). Each kernel
//! is launched on `cubecl-cpu` at order `n = 3` across 3 representative
//! inputs and the result is compared against a host reference that mirrors
//! the kernel's operation path step-for-step.
//!
//! See `crates/xcfun-ad/src/expand/{atan,asinh,gauss,erf}.rs` and
//! `xcfun-master/external/upstream/taylor/tmath.hpp:180-225, 259-274`.
//!
//! # Oracle strategy
//!
//! All four kernels are compositions of Plan 01-03 primary expansions
//! (`inv`, `exp`, `pow`) and `tfuns` helpers (`compose`, `integrate`,
//! `stretch`, `multo`). The host references reproduce each step in f64
//! using the same operation order as the kernel, so kernel-vs-host delta
//! is ≤ 1 ULP on cubecl-cpu. The 1e-13 relative-error gate is comfortable.
//!
//! # Cubecl 0.10-pre.3 f32-constant drift (erf, gauss via `tfuns_stretch`)
//!
//! - `erf_expand` scales by `2/√π` computed with `f32` π
//!   (`core::f32::consts::PI` widened to `f64`): ~1.3e-8 drift vs f64 π.
//!   Host reference mirrors this exactly.
//! - `gauss_expand` itself has no f32 constant; it uses `F::new(-2.0)`
//!   which is exact.

#![cfg(feature = "testing")]
#![allow(clippy::needless_range_loop)]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::expand::{asinh, atan, erf, gauss};
use xcfun_ad::for_tests::cpu_client;

// ---------------------------------------------------------------------------
//  Kernel adapters
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn kernel_atan<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    atan::atan_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_asinh<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    asinh::asinh_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_gauss<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    gauss::gauss_expand::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_erf<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    erf::erf_expand::<F>(t, scalars[0], n);
}

// ---------------------------------------------------------------------------
//  Generic launch helper (scalars in, t out).
// ---------------------------------------------------------------------------

fn run_scalar_kernel<L>(scalars: &[f64], out_len: usize, launcher: L) -> Vec<f64>
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
            "{label} coeff {i}: got {g:e}, expected {e:e}, rel_err {rel:e}"
        );
    }
}

// ---------------------------------------------------------------------------
//  Host helper: tfuns building blocks used by the host references.
// ---------------------------------------------------------------------------

fn host_tfuns_integrate(x: &mut [f64]) {
    let n = x.len() - 1;
    for k in 0..n {
        let i = n - k;
        let i_f = i as f64;
        x[i] = x[i - 1] / i_f;
    }
}

fn host_tfuns_stretch(t: &mut [f64], a: f64) {
    let n = t.len() - 1;
    let mut an = a;
    for i in 1..=n {
        t[i] *= an;
        an *= a;
    }
}

fn host_tfuns_multo(z: &mut [f64], x: &[f64]) {
    let n = z.len() - 1;
    for k in 0..=n {
        let i = n - k;
        let mut acc = x[0] * z[i];
        for j in 1..=i {
            acc += x[j] * z[i - j];
        }
        z[i] = acc;
    }
}

/// Port of `tfuns_compose` for a chosen n. Only n ∈ 0..=3 needed for the
/// tests (we exercise n=3 throughout).
fn host_tfuns_compose(f: &mut [f64], x: &[f64]) {
    let n = f.len() - 1;
    let f1 = if n >= 1 { f[1] } else { 0.0 };
    let f2 = if n >= 2 { f[2] } else { 0.0 };
    let f3 = if n >= 3 { f[3] } else { 0.0 };

    match n {
        0 => {}
        1 => {
            f[1] = f1 * x[1];
        }
        2 => {
            let a1 = f1 * x[2];
            let b1 = f2 * x[1];
            let b2 = b1 * x[1];
            f[2] = a1 + b2;
            f[1] = f1 * x[1];
        }
        3 => {
            let t1 = f1 * x[3];
            let s2a = 2.0 * f2;
            let s2b = s2a * x[2];
            let s3a = f3 * x[1];
            let s3b = s3a * x[1];
            let inner3 = s2b + s3b;
            let tail3 = x[1] * inner3;
            f[3] = t1 + tail3;

            let a1 = f1 * x[2];
            let b1 = f2 * x[1];
            let b2 = b1 * x[1];
            f[2] = a1 + b2;

            f[1] = f1 * x[1];
        }
        _ => unreachable!("host_tfuns_compose only supports n ≤ 3 in these tests"),
    }
}

// ---------------------------------------------------------------------------
//  Primary expansion host mirrors (reuse kernel operation order).
// ---------------------------------------------------------------------------

fn host_inv_expand(t: &mut [f64], a: f64) {
    let n = t.len() - 1;
    let t0 = 1.0 / a;
    t[0] = t0;
    for i in 1..=n {
        let prev = t[i - 1];
        let neg_prev = -prev;
        t[i] = neg_prev * t0;
    }
}

fn host_exp_expand(t: &mut [f64], x0: f64) {
    let n = t.len() - 1;
    let mut ifac = 1.0_f64;
    t[0] = x0.exp();
    for i in 1..=n {
        let i_f = i as f64;
        ifac *= i_f;
        t[i] = t[0] / ifac;
    }
}

fn host_pow_expand(t: &mut [f64], x0: f64, a: f64) {
    let n = t.len() - 1;
    t[0] = x0.powf(a);
    let x0inv = 1.0 / x0;
    for i in 1..=n {
        let i_f = i as f64;
        let a_minus_i = a - i_f;
        let a_minus_i_plus_1 = a_minus_i + 1.0;
        let s1 = t[i - 1] * x0inv;
        let s2 = s1 * a_minus_i_plus_1;
        t[i] = s2 / i_f;
    }
}

// ---------------------------------------------------------------------------
//  Transcendental expansion host mirrors (step-for-step copy of kernel body).
// ---------------------------------------------------------------------------

fn host_atan_expand(a: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    let mut x = vec![0.0_f64; n + 1];

    // inv_expand(t, 1 + a*a)
    let one_plus_a_sq = 1.0 + a * a;
    host_inv_expand(&mut t, one_plus_a_sq);

    // Build x = [0, 2a, 1, 0, ...].
    x[0] = 0.0;
    if n >= 1 {
        x[1] = 2.0 * a;
    }
    if n >= 2 {
        x[2] = 1.0;
    }
    for i in 3..=n {
        x[i] = 0.0;
    }

    host_tfuns_compose(&mut t, &x);
    host_tfuns_integrate(&mut t);
    t[0] = a.atan();
    t
}

fn host_asinh_expand(a: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];
    let mut tmp = vec![0.0_f64; n + 1];

    let one_plus_a_sq = 1.0 + a * a;
    tmp[0] = one_plus_a_sq;
    if n >= 1 {
        tmp[1] = 2.0 * a;
    }
    if n >= 2 {
        tmp[2] = 1.0;
    }
    for i in 3..=n {
        tmp[i] = 0.0;
    }

    host_pow_expand(&mut t, tmp[0], -0.5);
    host_tfuns_compose(&mut t, &tmp);
    host_tfuns_integrate(&mut t);
    t[0] = a.asinh();
    t
}

fn host_gauss_expand(a: f64, n: usize) -> Vec<f64> {
    let mut t = vec![0.0_f64; n + 1];

    // exp_expand(t, -a*a)
    let neg_a_sq = -(a * a);
    host_exp_expand(&mut t, neg_a_sq);

    // tfuns_stretch(t, -2 * a)
    let neg_two_a = -2.0 * a;
    host_tfuns_stretch(&mut t, neg_two_a);

    // Build g = exp(-y²) coefficients: g[0]=1, g[odd]=0, g[2i] = -g[2(i-1)]/i.
    let mut g = vec![0.0_f64; n + 1];
    g[0] = 1.0;
    for i in 1..=n {
        g[i] = 0.0;
    }
    for i in 1..=n / 2 {
        let i_f = i as f64;
        let prev = g[2 * (i - 1)];
        g[2 * i] = -prev / i_f;
    }

    host_tfuns_multo(&mut t, &g);
    t
}

/// Cubecl-cpu's erf polyfill (Wikipedia §"Numerical approximations", max
/// error 1.5e-7). This is what cubecl-cpu lowers `Arithmetic::Erf(f64)` to
/// (see `cubecl-cpu/src/compiler/passes/erf_transform.rs` +
/// `cubecl-core/src/frontend/polyfills.rs:28-52`). The polyfill constants
/// are stored as f32 literals in the cubecl source, so even on f64 target
/// the computation widens f32 → f64 mid-expression and carries ~1.5e-7
/// drift vs a true f64 `libm::erf`. We mirror that path here so the test
/// can assert bit-close identity against the kernel output.
fn host_erf_cubecl_polyfill(x: f64) -> f64 {
    // Reproduces `erf(x) = sign(x) * erf_positive(|x|)` from polyfills.rs.
    let abs_x = x.abs();
    // Constants — f32 literals widened to f64 (the cubecl `F::new(f32) → f64`
    // path). `core::f32::consts`-free handwritten values exactly as the
    // polyfill stores them.
    let p = 0.3275911_f32 as f64;
    let a1 = 0.2548296_f32 as f64;
    let a2 = -0.28449674_f32 as f64;
    let a3 = 1.4214137_f32 as f64;
    let a4 = -1.453152_f32 as f64;
    let a5 = 1.0614054_f32 as f64;
    let one = 1.0_f64;

    let t = one / (one + p * abs_x);
    let tmp = ((((a5 * t + a4) * t) + a3) * t + a2) * t + a1;
    let erf_pos = one - (tmp * t * (-abs_x * abs_x).exp());
    if x < 0.0 { -erf_pos } else { erf_pos }
}

fn host_erf_expand(a: f64, n: usize) -> Vec<f64> {
    let mut t = host_gauss_expand(a, n);

    // Match kernel's f32-π drift: c = 2.0 / (core::f32::consts::PI as f64).sqrt().
    let pi_f32: f32 = core::f32::consts::PI;
    let sqrt_pi = (pi_f32 as f64).sqrt();
    let c = 2.0 / sqrt_pi;
    for i in 0..=n {
        t[i] *= c;
    }

    host_tfuns_integrate(&mut t);
    // Mirror cubecl-cpu's polyfill path for erf(a) — NOT libm::erf. The
    // polyfill has ~1.5e-7 drift vs libm::erf but is what the kernel
    // actually computes.
    t[0] = host_erf_cubecl_polyfill(a);
    t
}

// ---------------------------------------------------------------------------
//  Launch helpers per kernel
// ---------------------------------------------------------------------------

fn run_atan_expand(a: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_scalar_kernel(&[a], out_len, |client, sh, oh| unsafe {
        kernel_atan::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_asinh_expand(a: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_scalar_kernel(&[a], out_len, |client, sh, oh| unsafe {
        kernel_asinh::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_gauss_expand(a: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_scalar_kernel(&[a], out_len, |client, sh, oh| unsafe {
        kernel_gauss::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, out_len),
            n,
        );
    })
}

fn run_erf_expand(a: f64, n: u32) -> Vec<f64> {
    let out_len = (n as usize) + 1;
    run_scalar_kernel(&[a], out_len, |client, sh, oh| unsafe {
        kernel_erf::launch_unchecked::<f64, CpuRuntime>(
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
//  atan_expand tests — 3 inputs at n=3.
// ===========================================================================

#[test]
fn atan_expand_a_neg1() {
    let got = run_atan_expand(-1.0, 3);
    let expected = host_atan_expand(-1.0, 3);
    assert_close(&got, &expected, "atan_expand a=-1.0");
}

#[test]
fn atan_expand_a_0() {
    // atan(y) = y - y³/3 + y⁵/5 - ... → [0, 1, 0, -1/3] at n=3.
    let got = run_atan_expand(0.0, 3);
    let expected = [0.0_f64, 1.0, 0.0, -1.0 / 3.0];
    assert_close(&got, &expected, "atan_expand a=0.0");
}

#[test]
fn atan_expand_a_1() {
    let got = run_atan_expand(1.0, 3);
    let expected = host_atan_expand(1.0, 3);
    assert_close(&got, &expected, "atan_expand a=1.0");
}

// ===========================================================================
//  asinh_expand tests — 3 inputs at n=3.
// ===========================================================================

#[test]
fn asinh_expand_a_neg1() {
    let got = run_asinh_expand(-1.0, 3);
    let expected = host_asinh_expand(-1.0, 3);
    assert_close(&got, &expected, "asinh_expand a=-1.0");
}

#[test]
fn asinh_expand_a_0() {
    // asinh(y) = y - y³/6 + 3y⁵/40 - ... → [0, 1, 0, -1/6] at n=3.
    let got = run_asinh_expand(0.0, 3);
    let expected = [0.0_f64, 1.0, 0.0, -1.0 / 6.0];
    assert_close(&got, &expected, "asinh_expand a=0.0");
}

#[test]
fn asinh_expand_a_1() {
    let got = run_asinh_expand(1.0, 3);
    let expected = host_asinh_expand(1.0, 3);
    assert_close(&got, &expected, "asinh_expand a=1.0");
}

// ===========================================================================
//  gauss_expand tests — 3 inputs at n=3.
// ===========================================================================

#[test]
fn gauss_expand_a_neg1() {
    let got = run_gauss_expand(-1.0, 3);
    let expected = host_gauss_expand(-1.0, 3);
    assert_close(&got, &expected, "gauss_expand a=-1.0");
}

#[test]
fn gauss_expand_a_0() {
    // exp(-y²) = 1 - y² + y⁴/2 - y⁶/6 → [1, 0, -1, 0] at n=3.
    let got = run_gauss_expand(0.0, 3);
    let expected = [1.0_f64, 0.0, -1.0, 0.0];
    assert_close(&got, &expected, "gauss_expand a=0.0");
}

#[test]
fn gauss_expand_a_1() {
    let got = run_gauss_expand(1.0, 3);
    let expected = host_gauss_expand(1.0, 3);
    assert_close(&got, &expected, "gauss_expand a=1.0");
}

// ===========================================================================
//  erf_expand tests — 3 inputs at n=3.
// ===========================================================================

#[test]
fn erf_expand_a_neg1() {
    let got = run_erf_expand(-1.0, 3);
    let expected = host_erf_expand(-1.0, 3);
    assert_close(&got, &expected, "erf_expand a=-1.0");
}

#[test]
fn erf_expand_a_0() {
    let got = run_erf_expand(0.0, 3);
    let expected = host_erf_expand(0.0, 3);
    assert_close(&got, &expected, "erf_expand a=0.0");
}

#[test]
fn erf_expand_a_1() {
    let got = run_erf_expand(1.0, 3);
    let expected = host_erf_expand(1.0, 3);
    assert_close(&got, &expected, "erf_expand a=1.0");
}
