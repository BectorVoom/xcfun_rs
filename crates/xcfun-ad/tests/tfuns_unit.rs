//! Unit tests for Plan 01-04 Task 1 — the `tfuns` scalar Taylor-series
//! helpers (`mul`, `multo`, `integrate`, `differentiate`, `shift`, `stretch`,
//! `compose`). Each test mirrors the `<behavior>` block of Plan 01-04 Task 1
//! and cross-checks the cubecl-cpu output against a host reference computed
//! from the C++ tmath.hpp formulas.
//!
//! See `crates/xcfun-ad/src/tfuns.rs` and
//! `xcfun-master/external/upstream/taylor/tmath.hpp:36-121`.

#![cfg(feature = "testing")]
// Host-reference fns use indexed loops to mirror kernel operation order 1:1.
#![allow(clippy::needless_range_loop)]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::for_tests::cpu_client;
use xcfun_ad::tfuns::{
    tfuns_compose, tfuns_differentiate, tfuns_integrate, tfuns_mul,
    tfuns_multo, tfuns_shift, tfuns_stretch,
};

// ---------------------------------------------------------------------------
//  Kernel adapters
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn kernel_mul<F: Float>(
    x: &Array<F>,
    y: &Array<F>,
    z: &mut Array<F>,
    #[comptime] n: u32,
) {
    tfuns_mul::<F>(z, x, y, n);
}

#[cube(launch_unchecked)]
fn kernel_multo<F: Float>(
    x: &Array<F>,
    z: &mut Array<F>,
    #[comptime] n: u32,
) {
    tfuns_multo::<F>(z, x, n);
}

#[cube(launch_unchecked)]
fn kernel_integrate<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    tfuns_integrate::<F>(x, n);
}

#[cube(launch_unchecked)]
fn kernel_differentiate<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    tfuns_differentiate::<F>(x, n);
}

#[cube(launch_unchecked)]
fn kernel_stretch<F: Float>(
    scalars: &Array<F>,
    t: &mut Array<F>,
    #[comptime] n: u32,
) {
    // scalars[0] = a
    tfuns_stretch::<F>(t, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_shift<F: Float>(
    scalars: &Array<F>,
    x: &mut Array<F>,
    #[comptime] n: u32,
) {
    // scalars[0] = d
    tfuns_shift::<F>(x, scalars[0], n);
}

#[cube(launch_unchecked)]
fn kernel_compose<F: Float>(
    x: &Array<F>,
    f: &mut Array<F>,
    #[comptime] n: u32,
) {
    tfuns_compose::<F>(f, x, n);
}

// ---------------------------------------------------------------------------
//  Common close-compare assertion (exact float equality for rational inputs,
//  rel 1e-13 for libm-touching inputs).
// ---------------------------------------------------------------------------

fn assert_close_bits(got: &[f64], expected: &[f64], label: &str) {
    assert_eq!(
        got.len(),
        expected.len(),
        "{label}: length mismatch got={} expected={}",
        got.len(),
        expected.len()
    );
    for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            g.to_bits(),
            e.to_bits(),
            "{label} coeff {i}: got {g:e} (bits {:#018x}), expected {e:e} (bits {:#018x})",
            g.to_bits(),
            e.to_bits()
        );
    }
}

// ---------------------------------------------------------------------------
//  Test: tfuns_mul (n=2)  — straight convolution.
// ---------------------------------------------------------------------------

#[test]
fn tfuns_mul_n2_polynomial_product() {
    // Behavior sample (Plan 01-04 Task 1 <behavior>):
    // x = [1, 2, 3], y = [4, 5, 6]. Expected: z[i] = sum_{j=0..=i} x[j]*y[i-j]
    //   z[0] = 1*4 = 4
    //   z[1] = 1*5 + 2*4 = 13
    //   z[2] = 1*6 + 2*5 + 3*4 = 28
    let x = [1.0_f64, 2.0, 3.0];
    let y = [4.0_f64, 5.0, 6.0];
    let n = 2_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x));
    let yh = client.create_from_slice(f64::as_bytes(&y));
    let zh = client.empty(out_len * core::mem::size_of::<f64>());
    let read_zh = zh.clone();
    unsafe {
        kernel_mul::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            ArrayArg::from_raw_parts(yh, out_len),
            ArrayArg::from_raw_parts(zh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_zh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [4.0_f64, 13.0, 28.0];
    assert_close_bits(got, &expected, "tfuns_mul n=2");
}

// ---------------------------------------------------------------------------
//  Test: tfuns_multo (n=2) — in-place descending-write z *= x.
// ---------------------------------------------------------------------------

#[test]
fn tfuns_multo_n2_in_place() {
    // Plan 01-04 <behavior>: z = [1,2,3], x = [4,5,6].
    // Descending write:
    //   z[2] = x[0]*z[2] + x[1]*z[1] + x[2]*z[0] = 4*3 + 5*2 + 6*1 = 28
    //   z[1] = x[0]*z[1] + x[1]*z[0]             = 4*2 + 5*1 = 13
    //   z[0] = x[0]*z[0]                         = 4*1 = 4
    let z_in = [1.0_f64, 2.0, 3.0];
    let x = [4.0_f64, 5.0, 6.0];
    let n = 2_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x));
    let zh = client.create_from_slice(f64::as_bytes(&z_in));
    let read_zh = zh.clone();
    unsafe {
        kernel_multo::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            ArrayArg::from_raw_parts(zh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_zh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [4.0_f64, 13.0, 28.0];
    assert_close_bits(got, &expected, "tfuns_multo n=2");
}

// ---------------------------------------------------------------------------
//  Test: tfuns_integrate (n=3) — x[i] = x[i-1] / i descending.
// ---------------------------------------------------------------------------

#[test]
fn tfuns_integrate_n3_descending() {
    // Plan <behavior>: Starting with x = [0, 1, 2, 3], after integrate:
    //   x[3] = x[2]/3 = 2/3
    //   x[2] = x[1]/2 = 1/2
    //   x[1] = x[0]/1 = 0
    //   x[0] unchanged = 0
    let x_in = [0.0_f64, 1.0, 2.0, 3.0];
    let n = 3_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x_in));
    let read_xh = xh.clone();
    unsafe {
        kernel_integrate::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_xh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [0.0_f64, 0.0, 0.5, 2.0 / 3.0];
    assert_close_bits(got, &expected, "tfuns_integrate n=3");
}

// ---------------------------------------------------------------------------
//  Test: tfuns_differentiate (n=3) — x[i-1] = i*x[i], x[n]=0.
// ---------------------------------------------------------------------------

#[test]
fn tfuns_differentiate_n3() {
    // Plan <behavior>: Starting with x = [10, 20, 30, 40]:
    //   x[0] = 1 * x[1] = 20
    //   x[1] = 2 * x[2] = 60
    //   x[2] = 3 * x[3] = 120
    //   x[3] = 0
    //
    // Note: The C++ loop is:
    //   for (i = 1..=N) x[i-1] = i * x[i];
    //   x[N] = 0;
    // With N=3, iteration order: i=1 writes x[0]; i=2 writes x[1]; i=3 writes x[2].
    // Reads of x[1..=3] happen BEFORE any write that would shadow them for the
    // current iteration — the loop is strictly "read x[i], write x[i-1]" so no
    // aliasing. After loop, x[3] is set to 0.
    let x_in = [10.0_f64, 20.0, 30.0, 40.0];
    let n = 3_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x_in));
    let read_xh = xh.clone();
    unsafe {
        kernel_differentiate::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_xh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [20.0_f64, 60.0, 120.0, 0.0];
    assert_close_bits(got, &expected, "tfuns_differentiate n=3");
}

// ---------------------------------------------------------------------------
//  Test: tfuns_stretch (n=3) — t[i] *= a^i, t[0] unchanged.
// ---------------------------------------------------------------------------

#[test]
fn tfuns_stretch_n3() {
    // Plan <behavior>: t = [5, 1, 2, 3], a = 0.5.
    //   t[0] = 5 (unchanged)
    //   t[1] = 1 * 0.5 = 0.5
    //   t[2] = 2 * 0.25 = 0.5
    //   t[3] = 3 * 0.125 = 0.375
    let t_in = [5.0_f64, 1.0, 2.0, 3.0];
    let a = [0.5_f64];
    let n = 3_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let sh = client.create_from_slice(f64::as_bytes(&a));
    let th = client.create_from_slice(f64::as_bytes(&t_in));
    let read_th = th.clone();
    unsafe {
        kernel_stretch::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(th, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_th);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [5.0_f64, 0.5, 0.5, 0.375];
    assert_close_bits(got, &expected, "tfuns_stretch n=3");
}

// ---------------------------------------------------------------------------
//  Test: tfuns_shift (n=2, d=0) — no-op at zero shift.
// ---------------------------------------------------------------------------

fn host_shift(x_in: &[f64], d: f64) -> Vec<f64> {
    // Host mirror of tmath.hpp:63-78.
    let n = x_in.len() - 1;
    let mut x = x_in.to_vec();
    let mut dn = vec![0.0_f64; n + 1];
    dn[0] = 1.0;
    for i in 1..=n {
        dn[i] = d * dn[i - 1];
    }
    let xn_full = x.clone();
    for ni in 0..n {
        let mut acc = xn_full[ni];
        let mut fac = (ni + 1) as f64;
        for m in (ni + 1)..n {
            let s1 = fac * dn[m - ni];
            let s2 = s1 * xn_full[m];
            acc += s2;
            let mp1 = (m + 1) as f64;
            let mmn = (m - ni + 1) as f64;
            fac *= mp1;
            fac /= mmn;
        }
        let tail1 = fac * dn[n - ni];
        let tail2 = tail1 * xn_full[n];
        acc += tail2;
        x[ni] = acc;
    }
    x
}

#[test]
fn tfuns_shift_n2_zero_shift_is_noop() {
    // d = 0 — the shift contribution is all zero (dn[i]=0 for i>=1),
    // so only the tail term adds `fac * dn[N-n] * x[N]`. But dn[N-n] = 0
    // for N-n >= 1, and only dn[0]=1 when N-n=0 which only happens at
    // ni=N (outside the loop). So x should be unchanged.
    let x_in = [7.0_f64, 3.0, 5.0];
    let d = [0.0_f64];
    let n = 2_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let sh = client.create_from_slice(f64::as_bytes(&d));
    let xh = client.create_from_slice(f64::as_bytes(&x_in));
    let read_xh = xh.clone();
    unsafe {
        kernel_shift::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_xh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = host_shift(&x_in, 0.0);
    assert_close_bits(got, &expected, "tfuns_shift n=2 d=0");
}

#[test]
fn tfuns_shift_n3_d_half() {
    let x_in = [1.0_f64, 2.0, 3.0, 4.0];
    let d = [0.5_f64];
    let n = 3_u32;
    let out_len = (n as usize) + 1;

    let client = cpu_client();
    let sh = client.create_from_slice(f64::as_bytes(&d));
    let xh = client.create_from_slice(f64::as_bytes(&x_in));
    let read_xh = xh.clone();
    unsafe {
        kernel_shift::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_xh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = host_shift(&x_in, 0.5);
    assert_close_bits(got, &expected, "tfuns_shift n=3 d=0.5");
}

// ---------------------------------------------------------------------------
//  Test: tfuns_compose per-N specialisations.
//  At x[0]=0, x[1]=y the compose fills f in place; the new f[i] is the
//  coefficient of `x_inner^i` in sum_k f_old[k] * x(y)^k. Small-N cases
//  reduce to simple polynomial substitutions.
// ---------------------------------------------------------------------------

#[test]
fn tfuns_compose_n0_is_identity() {
    // n=0 case: fn body is empty — f stays as-is.
    let f_in = [7.0_f64];
    let x = [0.0_f64];
    let n = 0_u32;
    let out_len = 1_usize;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x));
    let fh = client.create_from_slice(f64::as_bytes(&f_in));
    let read_fh = fh.clone();
    unsafe {
        kernel_compose::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            ArrayArg::from_raw_parts(fh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_fh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [7.0_f64];
    assert_close_bits(got, &expected, "tfuns_compose n=0");
}

#[test]
fn tfuns_compose_n1_linear() {
    // n=1: f[1] = f[1] * x[1]. f[0] unchanged.
    let f_in = [3.0_f64, 5.0];
    let x = [0.0_f64, 2.0];
    let n = 1_u32;
    let out_len = 2_usize;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x));
    let fh = client.create_from_slice(f64::as_bytes(&f_in));
    let read_fh = fh.clone();
    unsafe {
        kernel_compose::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            ArrayArg::from_raw_parts(fh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_fh);
    let got: &[f64] = f64::from_bytes(&bytes);

    // Expected: f[0]=3 (unchanged), f[1] = 5*2 = 10.
    let expected = [3.0_f64, 10.0];
    assert_close_bits(got, &expected, "tfuns_compose n=1");
}

#[test]
fn tfuns_compose_n2_matches_cpp() {
    // n=2: per tmath.hpp:104-107 cases 2, 1:
    //   f[2] = f[1]*x[2] + f[2]*x[1]*x[1]
    //   f[1] = f[1]*x[1]
    // Pick f=[1, 2, 3], x=[0, 4, 5]:
    //   f[2] = 2*5 + 3*4*4 = 10 + 48 = 58
    //   f[1] = 2*4 = 8
    let f_in = [1.0_f64, 2.0, 3.0];
    let x = [0.0_f64, 4.0, 5.0];
    let n = 2_u32;
    let out_len = 3_usize;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x));
    let fh = client.create_from_slice(f64::as_bytes(&f_in));
    let read_fh = fh.clone();
    unsafe {
        kernel_compose::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            ArrayArg::from_raw_parts(fh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_fh);
    let got: &[f64] = f64::from_bytes(&bytes);

    // f[0] unchanged.
    let expected = [1.0_f64, 8.0, 58.0];
    assert_close_bits(got, &expected, "tfuns_compose n=2");
}

#[test]
fn tfuns_compose_n3_matches_cpp() {
    // n=3: per tmath.hpp:102-107 cases 3, 2, 1:
    //   f[3] = f[1]*x[3] + x[1]*(2*f[2]*x[2] + f[3]*x[1]*x[1])
    //   f[2] = f[1]*x[2] + f[2]*x[1]*x[1]
    //   f[1] = f[1]*x[1]
    // Pick f=[10, 1, 2, 3], x=[0, 4, 5, 6]:
    //   f[3]: inner = 2*2*5 + 3*4*4 = 20 + 48 = 68; x[1]*inner = 4*68 = 272
    //         f[1]*x[3] = 1*6 = 6; sum = 278
    //   f[2] = 1*5 + 2*4*4 = 5 + 32 = 37
    //   f[1] = 1*4 = 4
    //   f[0] unchanged = 10
    let f_in = [10.0_f64, 1.0, 2.0, 3.0];
    let x = [0.0_f64, 4.0, 5.0, 6.0];
    let n = 3_u32;
    let out_len = 4_usize;

    let client = cpu_client();
    let xh = client.create_from_slice(f64::as_bytes(&x));
    let fh = client.create_from_slice(f64::as_bytes(&f_in));
    let read_fh = fh.clone();
    unsafe {
        kernel_compose::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(xh, out_len),
            ArrayArg::from_raw_parts(fh, out_len),
            n,
        );
    }
    let bytes = client.read_one_unchecked(read_fh);
    let got: &[f64] = f64::from_bytes(&bytes);

    let expected = [10.0_f64, 4.0, 37.0, 278.0];
    assert_close_bits(got, &expected, "tfuns_compose n=3");
}
