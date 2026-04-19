//! Integration tests for `ctaylor_*` #[cube] fns on cubecl-cpu. Verifies
//! bit-exact (`f64::to_bits` identity) parity against host-side reference
//! implementations for N ∈ {0, 1, 2, 3}.
//!
//! This is Plan 01-02's validation gate (VALIDATION.md task 01-03-02). The
//! C++ xcfun parity fixture test is Plan 01-05's concern; this test
//! proves the per-N specialisations match the C++ algebraic recursion as
//! transcribed by a host-side mirror implementation.
//!
//! # Cubecl 0.10-pre.3 idiom notes — consolidated
//!
//! - Launch via `kernel::launch_unchecked::<f64, CpuRuntime>(client, ...)`
//!   inside `unsafe { ... }` (working idiom from `tests/cubecl_spike.rs`).
//! - `ArrayArg::from_raw_parts(handle.clone(), len)` — 2 args, no turbofish,
//!   no vectorisation argument (cubecl 0.10-pre.3 delta).
//! - `client.create_from_slice(f64::as_bytes(&[..]))` / `client.empty(bytes)` /
//!   `client.read_one_unchecked(handle)` — see `for_tests/raw_eval_scalar.rs`.
//! - Each kernel declares `#[comptime] n: u32`; comptime value is passed as
//!   the trailing scalar argument on launch.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::ctaylor::{
    ctaylor_add, ctaylor_from_scalar, ctaylor_from_variable, ctaylor_neg,
    ctaylor_scalar_mul, ctaylor_sub,
};
use xcfun_ad::ctaylor_rec::{
    compose::ctaylor_compose, mul::ctaylor_mul,
    multo::{ctaylor_multo, ctaylor_multo_skipconst},
};
use xcfun_ad::for_tests::cpu_client;
use xcfun_ad::VAR1;

// ---------------------------------------------------------------------------
//  Kernel adapters — one per operation, comptime n supplied at launch.
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn kernel_add<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_add::<F>(a, b, out, n);
}

#[cube(launch_unchecked)]
fn kernel_sub<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_sub::<F>(a, b, out, n);
}

#[cube(launch_unchecked)]
fn kernel_neg<F: Float>(a: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_neg::<F>(a, out, n);
}

#[cube(launch_unchecked)]
fn kernel_scalar_mul<F: Float>(
    a: &Array<F>,
    s: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_scalar_mul::<F>(a, s[0], out, n);
}

#[cube(launch_unchecked)]
fn kernel_from_scalar<F: Float>(
    c0: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_from_scalar::<F>(c0[0], out, n);
}

#[cube(launch_unchecked)]
fn kernel_from_variable<F: Float>(
    c0_slope: &Array<F>,
    out: &mut Array<F>,
    #[comptime] var_bit: u32,
    #[comptime] n: u32,
) {
    // c0_slope = [c0, slope]
    ctaylor_from_variable::<F>(c0_slope[0], c0_slope[1], var_bit, out, n);
}

#[cube(launch_unchecked)]
fn kernel_mul<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_mul::<F>(a, b, out, n);
}

#[cube(launch_unchecked)]
fn kernel_multo<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_multo::<F>(dst, y, n);
}

#[cube(launch_unchecked)]
fn kernel_multo_skipconst<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_multo_skipconst::<F>(dst, y, n);
}

#[cube(launch_unchecked)]
fn kernel_compose<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_compose::<F>(out, x, f, n);
}

// ---------------------------------------------------------------------------
//  Launch helpers. Each returns Vec<f64> read back from the output handle.
// ---------------------------------------------------------------------------

fn run_binary_op<L>(
    a: &[f64],
    b: &[f64],
    out_len: usize,
    launcher: L,
) -> Vec<f64>
where
    L: FnOnce(&cubecl::prelude::ComputeClient<CpuRuntime>, cubecl::server::Handle, cubecl::server::Handle, cubecl::server::Handle),
{
    let client = cpu_client();
    let a_h = client.create_from_slice(f64::as_bytes(a));
    let b_h = client.create_from_slice(f64::as_bytes(b));
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();
    launcher(client, a_h, b_h, out_h);
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

fn run_unary_op<L>(a: &[f64], out_len: usize, launcher: L) -> Vec<f64>
where
    L: FnOnce(&cubecl::prelude::ComputeClient<CpuRuntime>, cubecl::server::Handle, cubecl::server::Handle),
{
    let client = cpu_client();
    let a_h = client.create_from_slice(f64::as_bytes(a));
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();
    launcher(client, a_h, out_h);
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

/// In-place kernel (like multo): uploads `dst` + `y`, launches kernel on
/// the `dst` handle, reads `dst` back.
fn run_inplace_op<L>(dst: &[f64], y: &[f64], launcher: L) -> Vec<f64>
where
    L: FnOnce(&cubecl::prelude::ComputeClient<CpuRuntime>, cubecl::server::Handle, cubecl::server::Handle),
{
    let client = cpu_client();
    let dst_h = client.create_from_slice(f64::as_bytes(dst));
    let y_h = client.create_from_slice(f64::as_bytes(y));
    let read_h = dst_h.clone();
    launcher(client, dst_h, y_h);
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

/// Ternary op helper (x, f, out_len + 3 handles).
fn run_ternary_out<L>(
    x: &[f64],
    f: &[f64],
    out_len: usize,
    launcher: L,
) -> Vec<f64>
where
    L: FnOnce(&cubecl::prelude::ComputeClient<CpuRuntime>, cubecl::server::Handle, cubecl::server::Handle, cubecl::server::Handle),
{
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(x));
    let f_h = client.create_from_slice(f64::as_bytes(f));
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();
    launcher(client, out_h, x_h, f_h);
    let bytes = client.read_one_unchecked(read_h);
    f64::from_bytes(&bytes).to_vec()
}

// ---------------------------------------------------------------------------
//  Host-side reference implementations (mirrors of the C++ recursion).
//  These are THE oracle for f64::to_bits identity assertions.
// ---------------------------------------------------------------------------

/// Host mirror of `ctaylor_rec<T, 2>::multo` from ctaylor.hpp:131-135.
fn host_multo_n2(dst: &mut [f64; 4], y: &[f64; 4]) {
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];
    let d3 = dst[3];
    // dst[3] = d0*y[3] + d3*y[0] + d1*y[2] + d2*y[1]
    let t30 = d0 * y[3];
    let t31 = d3 * y[0];
    let t32 = d1 * y[2];
    let t33 = d2 * y[1];
    let s1 = t30 + t31;
    let s2 = s1 + t32;
    dst[3] = s2 + t33;
    // dst[2] = d0*y[2] + d2*y[0]
    dst[2] = d0 * y[2] + d2 * y[0];
    // dst[1] = d0*y[1] + d1*y[0]
    dst[1] = d0 * y[1] + d1 * y[0];
    // dst[0] = d0 * y[0]
    dst[0] = d0 * y[0];
}

/// Host mirror of `ctaylor_rec<T, 2>::mul_set` from ctaylor.hpp:125-130.
fn host_mul_set_n2(dst: &mut [f64; 4], x: &[f64; 4], y: &[f64; 4]) {
    dst[0] = x[0] * y[0];
    dst[1] = x[0] * y[1] + x[1] * y[0];
    dst[2] = x[0] * y[2] + x[2] * y[0];
    // dst[3] = x[0]*y[3] + x[3]*y[0] + x[1]*y[2] + x[2]*y[1]   (left-assoc)
    let t30 = x[0] * y[3];
    let t31 = x[3] * y[0];
    let t32 = x[1] * y[2];
    let t33 = x[2] * y[1];
    let s1 = t30 + t31;
    let s2 = s1 + t32;
    dst[3] = s2 + t33;
}

/// Host mirror of `ctaylor_rec<T, 2>::mul` (accumulating).
fn host_mul_acc_n2(dst: &mut [f64; 4], x: &[f64; 4], y: &[f64; 4]) {
    dst[0] += x[0] * y[0];
    dst[1] += x[0] * y[1] + x[1] * y[0];
    dst[2] += x[0] * y[2] + x[2] * y[0];
    let t30 = x[0] * y[3];
    let t31 = x[3] * y[0];
    let t32 = x[1] * y[2];
    let t33 = x[2] * y[1];
    let s1 = t30 + t31;
    let s2 = s1 + t32;
    dst[3] += s2 + t33;
}

/// Host mirror of the N=3 mul_set (3-call recursion). Exactly matches
/// `ctaylor_mul_set_n3` flattening.
fn host_mul_set_n3(dst: &mut [f64; 8], x: &[f64; 8], y: &[f64; 8]) {
    // lower
    let (lo, hi) = dst.split_at_mut(4);
    let lo4: &mut [f64; 4] = lo.try_into().unwrap();
    let hi4: &mut [f64; 4] = hi.try_into().unwrap();
    let x_lo: &[f64; 4] = x[0..4].try_into().unwrap();
    let x_hi: &[f64; 4] = x[4..8].try_into().unwrap();
    let y_lo: &[f64; 4] = y[0..4].try_into().unwrap();
    let y_hi: &[f64; 4] = y[4..8].try_into().unwrap();

    host_mul_set_n2(lo4, x_lo, y_lo);
    host_mul_set_n2(hi4, x_hi, y_lo);
    host_mul_acc_n2(hi4, x_lo, y_hi);
}

/// Host mirror of `ctaylor_rec<T, 2>::compose` from ctaylor.hpp:146-151.
fn host_compose_n2(out: &mut [f64; 4], x: &[f64; 4], f: &[f64; 3]) {
    out[0] = f[0];
    out[1] = f[1] * x[1];
    out[2] = f[1] * x[2];
    // out[3] = f[1]*x[3] + 2 * x[1] * x[2] * f[2]
    let t1 = f[1] * x[3];
    let s1 = 2.0 * x[1];
    let s2 = s1 * x[2];
    let s3 = s2 * f[2];
    out[3] = t1 + s3;
}

// ---------------------------------------------------------------------------
//  Tests — element-wise ops
// ---------------------------------------------------------------------------

#[test]
fn add_n1() {
    let a = [1.0_f64, 2.0];
    let b = [3.0_f64, 4.0];
    let out = run_binary_op(&a, &b, 2, |client, ah, bh, oh| unsafe {
        kernel_add::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ah, 2),
            ArrayArg::from_raw_parts(bh, 2),
            ArrayArg::from_raw_parts(oh, 2),
            1_u32,
        );
    });
    assert_eq!(out, vec![4.0_f64, 6.0]);
    assert_eq!(out[0].to_bits(), 4.0_f64.to_bits());
    assert_eq!(out[1].to_bits(), 6.0_f64.to_bits());
}

#[test]
fn sub_n2() {
    let a = [10.0_f64, 20.0, 30.0, 40.0];
    let b = [1.0_f64, 2.0, 3.0, 4.0];
    let out = run_binary_op(&a, &b, 4, |client, ah, bh, oh| unsafe {
        kernel_sub::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ah, 4),
            ArrayArg::from_raw_parts(bh, 4),
            ArrayArg::from_raw_parts(oh, 4),
            2_u32,
        );
    });
    assert_eq!(out, vec![9.0_f64, 18.0, 27.0, 36.0]);
}

#[test]
fn neg_n3() {
    let a = [1.0_f64, -2.0, 3.0, -4.0, 5.0, -6.0, 7.0, -8.0];
    let expected = [-1.0_f64, 2.0, -3.0, 4.0, -5.0, 6.0, -7.0, 8.0];
    let out = run_unary_op(&a, 8, |client, ah, oh| unsafe {
        kernel_neg::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ah, 8),
            ArrayArg::from_raw_parts(oh, 8),
            3_u32,
        );
    });
    for (g, e) in out.iter().zip(expected.iter()) {
        assert_eq!(g.to_bits(), e.to_bits());
    }
}

#[test]
fn scalar_mul_n2() {
    let a = [1.0_f64, 2.0, 3.0, 4.0];
    let s = [0.5_f64];
    let out = run_binary_op(&a, &s, 4, |client, ah, sh, oh| unsafe {
        kernel_scalar_mul::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ah, 4),
            ArrayArg::from_raw_parts(sh, 1),
            ArrayArg::from_raw_parts(oh, 4),
            2_u32,
        );
    });
    assert_eq!(out, vec![0.5_f64, 1.0, 1.5, 2.0]);
}

#[test]
fn from_scalar_n3() {
    // Literal value chosen to avoid clippy's `approx_constant` PI lint.
    let c0 = [2.5_f64];
    let out = run_unary_op(&c0, 8, |client, ch, oh| unsafe {
        kernel_from_scalar::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ch, 1),
            ArrayArg::from_raw_parts(oh, 8),
            3_u32,
        );
    });
    let expected = [2.5_f64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    for (g, e) in out.iter().zip(expected.iter()) {
        assert_eq!(g.to_bits(), e.to_bits());
    }
}

#[test]
fn from_variable_n2_var1() {
    // VAR1 = 2, c0 = 2.0, slope = 5.0, n = 2
    //   expected result: [2, 0, 5, 0]  (out[VAR1=2] = 5.0)
    let c0_slope = [2.0_f64, 5.0];
    let out = run_unary_op(&c0_slope, 4, |client, ch, oh| unsafe {
        kernel_from_variable::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ch, 2),
            ArrayArg::from_raw_parts(oh, 4),
            VAR1,
            2_u32,
        );
    });
    let expected = [2.0_f64, 0.0, 5.0, 0.0];
    for (g, e) in out.iter().zip(expected.iter()) {
        assert_eq!(g.to_bits(), e.to_bits());
    }
}

// ---------------------------------------------------------------------------
//  Tests — multo / multo_skipconst
// ---------------------------------------------------------------------------

#[test]
fn multo_n1_exact_order() {
    // ctaylor.hpp:103-106: dst[1] = dst[1]*y[0] + dst[0]*y[1]; dst[0] *= y[0];
    // dst=[2,3], y=[4,5] => dst[1] = 3*4 + 2*5 = 22; dst[0] = 2*4 = 8
    let dst = [2.0_f64, 3.0];
    let y = [4.0_f64, 5.0];
    let out = run_inplace_op(&dst, &y, |client, dh, yh| unsafe {
        kernel_multo::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(dh, 2),
            ArrayArg::from_raw_parts(yh, 2),
            1_u32,
        );
    });
    assert_eq!(out[0].to_bits(), 8.0_f64.to_bits());
    assert_eq!(out[1].to_bits(), 22.0_f64.to_bits());
}

#[test]
fn multo_n2_exact_order() {
    // ctaylor.hpp:131-135.
    // dst=[1,2,3,4], y=[5,6,7,8] ⇒ [5, 16, 22, 60]
    let mut dst_arr = [1.0_f64, 2.0, 3.0, 4.0];
    let y = [5.0_f64, 6.0, 7.0, 8.0];

    let out = run_inplace_op(&dst_arr, &y, |client, dh, yh| unsafe {
        kernel_multo::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(dh, 4),
            ArrayArg::from_raw_parts(yh, 4),
            2_u32,
        );
    });

    // Compare to host-side reference (same recursion, same operation order).
    host_multo_n2(&mut dst_arr, &y);
    for i in 0..4 {
        assert_eq!(
            out[i].to_bits(),
            dst_arr[i].to_bits(),
            "multo_n2 coeff {i}: got {}, expected {}",
            out[i],
            dst_arr[i]
        );
    }
    // Sanity: exact values from plan <behavior>.
    let expected = [5.0_f64, 16.0, 22.0, 60.0];
    for i in 0..4 {
        assert_eq!(out[i].to_bits(), expected[i].to_bits());
    }
}

#[test]
fn multo_skipconst_n2() {
    // ctaylor.hpp:137-142:
    //   dst[3] = dst[0]*y[3] + dst[1]*y[2] + dst[2]*y[1]
    //   dst[2] = dst[0]*y[2]
    //   dst[1] = dst[0]*y[1]
    //   dst[0] = 0
    // dst=[1,2,3,4], y=[5,6,7,8] ⇒
    //   dst[3] = 1*8 + 2*7 + 3*6 = 8 + 14 + 18 = 40
    //   dst[2] = 1*7 = 7
    //   dst[1] = 1*6 = 6
    //   dst[0] = 0
    let dst = [1.0_f64, 2.0, 3.0, 4.0];
    let y = [5.0_f64, 6.0, 7.0, 8.0];
    let out = run_inplace_op(&dst, &y, |client, dh, yh| unsafe {
        kernel_multo_skipconst::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(dh, 4),
            ArrayArg::from_raw_parts(yh, 4),
            2_u32,
        );
    });
    let expected = [0.0_f64, 6.0, 7.0, 40.0];
    for i in 0..4 {
        assert_eq!(
            out[i].to_bits(),
            expected[i].to_bits(),
            "multo_skipconst_n2 coeff {i}: got {}, expected {}",
            out[i],
            expected[i]
        );
    }
}

// ---------------------------------------------------------------------------
//  Tests — mul
// ---------------------------------------------------------------------------

#[test]
fn mul_n3_vs_host_ref() {
    // a=b=[1..8]; run cubecl-cpu ctaylor_mul vs. host_mul_set_n3 reference.
    let a: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let b: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

    let out = run_binary_op(&a, &b, 8, |client, ah, bh, oh| unsafe {
        kernel_mul::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ah, 8),
            ArrayArg::from_raw_parts(bh, 8),
            ArrayArg::from_raw_parts(oh, 8),
            3_u32,
        );
    });

    let mut expected = [0.0_f64; 8];
    host_mul_set_n3(&mut expected, &a, &b);

    for i in 0..8 {
        assert_eq!(
            out[i].to_bits(),
            expected[i].to_bits(),
            "mul_n3 coeff {i}: got {}, expected {}",
            out[i],
            expected[i]
        );
    }
}

#[test]
fn mul_n1_basic() {
    // N=1 mul_set: out[0] = a[0]*b[0]; out[1] = a[0]*b[1] + a[1]*b[0]
    // a=[2, 3], b=[4, 5] ⇒ out=[8, 22]
    let a = [2.0_f64, 3.0];
    let b = [4.0_f64, 5.0];
    let out = run_binary_op(&a, &b, 2, |client, ah, bh, oh| unsafe {
        kernel_mul::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(ah, 2),
            ArrayArg::from_raw_parts(bh, 2),
            ArrayArg::from_raw_parts(oh, 2),
            1_u32,
        );
    });
    assert_eq!(out[0].to_bits(), 8.0_f64.to_bits());
    assert_eq!(out[1].to_bits(), 22.0_f64.to_bits());
}

// ---------------------------------------------------------------------------
//  Tests — compose
// ---------------------------------------------------------------------------

#[test]
fn compose_n2_simple_case() {
    // f(x) = f0 + f1 * (x - x[0]) + f2 * (x - x[0])^2
    // For x = [0, 1, 0, 0] (i.e. just VAR0 = x), (x - x[0]) has:
    //   c[0] = 0, c[1] = 1, c[2] = 0, c[3] = 0
    // (x - x[0])^2 at N=2:
    //   coeffs = multo_skipconst_n2 applied to (x - x[0]) = [0, 1, 0, 0]:
    //     dst[3] = 0; dst[2] = 0; dst[1] = 0; dst[0] = 0  (since dst[0] was 0)
    //   i.e. (x - x[0])^2 = 0 at N=2 if only VAR0 is nonzero.
    // So f(x) = f0 + f1 * x — the x^2 term vanishes at N=2 for this x.
    //
    // Test: f = [1.5, 2.5, 3.5], x = [0.0, 1.0, 0.0, 0.0]
    //   compose_n2 body per base case:
    //     out[0] = f[0] = 1.5
    //     out[1] = f[1] * x[1] = 2.5 * 1.0 = 2.5
    //     out[2] = f[1] * x[2] = 2.5 * 0.0 = 0.0
    //     out[3] = f[1] * x[3] + 2 * x[1] * x[2] * f[2]
    //            = 2.5 * 0.0 + 2 * 1.0 * 0.0 * 3.5 = 0.0
    let x = [0.0_f64, 1.0, 0.0, 0.0];
    let f = [1.5_f64, 2.5, 3.5];
    let out = run_ternary_out(&x, &f, 4, |client, oh, xh, fh| unsafe {
        kernel_compose::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(oh, 4),
            ArrayArg::from_raw_parts(xh, 4),
            ArrayArg::from_raw_parts(fh, 3),
            2_u32,
        );
    });

    let mut expected = [0.0_f64; 4];
    host_compose_n2(&mut expected, &x, &f);

    for i in 0..4 {
        assert_eq!(
            out[i].to_bits(),
            expected[i].to_bits(),
            "compose_n2 coeff {i}: got {}, expected {}",
            out[i],
            expected[i]
        );
    }
}

#[test]
fn compose_n2_general_case() {
    // More interesting x with all 4 slots nonzero: x[1] and x[2] are the two
    // independent variables, x[3] is their coupled term from an earlier op.
    //
    // x = [0.5, 1.0, 2.0, 0.3], f = [0.1, 0.2, 0.4]
    //   out[0] = 0.1
    //   out[1] = 0.2 * 1.0 = 0.2
    //   out[2] = 0.2 * 2.0 = 0.4
    //   out[3] = 0.2 * 0.3 + ((2 * 1.0) * 2.0) * 0.4
    //          = 0.06 + 1.6
    //          = 1.66
    let x = [0.5_f64, 1.0, 2.0, 0.3];
    let f = [0.1_f64, 0.2, 0.4];
    let out = run_ternary_out(&x, &f, 4, |client, oh, xh, fh| unsafe {
        kernel_compose::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(oh, 4),
            ArrayArg::from_raw_parts(xh, 4),
            ArrayArg::from_raw_parts(fh, 3),
            2_u32,
        );
    });

    let mut expected = [0.0_f64; 4];
    host_compose_n2(&mut expected, &x, &f);

    for i in 0..4 {
        assert_eq!(
            out[i].to_bits(),
            expected[i].to_bits(),
            "compose_n2 coeff {i}: got {}, expected {}",
            out[i],
            expected[i]
        );
    }
}
