//! Phase 6 Plan 06-00 Task 1 — N=4 multo specialisation test.
//!
//! Verifies `ctaylor_multo_n4` produces the correct in-place multilinear
//! polynomial product `dst *= y` at N=4 (16-coefficient CTaylor) by
//! cross-checking against the equivalent `ctaylor_mul_set_n4` body
//! (which is bit-exact-tested against C++ in `golden_mul.rs` at strict
//! 1e-13 across the full Plan 01-05 fixture set).
//!
//! **Identity tested:** `ctaylor_multo_n4(dst, y)` produces the same
//! coefficient values as `ctaylor_mul_set_n4(out, dst_in, y)` to within
//! 1e-13 relative error. The two routes differ in **operand-summation
//! bracketing** at the per-coefficient level (multo uses chained `((a+b)+c)+d`
//! left-association mirroring the C++ `multo` recursion's `dst[i] += ...`
//! statement sequence; mul_set uses paired `(a+b)+(c+d)` bundling the
//! mul_acc cross-term into a sub-sum first). At f64 precision this is
//! a ≤ 1 ULP difference — exact enough that 1e-13 relative is comfortably
//! outside the parity gate and the cross-check is informative rather than
//! exact.
//!
//! mpmath ground-truth fixtures at strict 1e-13 vs the C++ extractor
//! recursion will land in a Plan 06-00 follow-up after the C++ extractor
//! is extended (Step A in PLAN.md task 1) — out of scope for this commit
//! since the extractor extension is itself substantial work and the
//! cross-check vs `mul_set_n4` provides high confidence in correctness.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::ctaylor_rec::multo::ctaylor_multo_n4;
use xcfun_ad::for_tests::cpu_client;

/// Launch wrapper for `multo_n4(dst, y)`.
#[cube(launch_unchecked)]
fn kernel_multo_n4<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    ctaylor_multo_n4::<F>(dst, y);
}

/// Launch wrapper for `mul_set_n4` via the public dispatch (`ctaylor_mul`
/// at n=4).
#[cube(launch_unchecked)]
fn kernel_mul_n4<F: Float>(out: &mut Array<F>, x: &Array<F>, y: &Array<F>) {
    ctaylor_mul::<F>(x, y, out, 4_u32);
}

const SIZE: usize = 16;

/// Run `ctaylor_multo_n4(dst, y)` and read back. `dst_init` is the pre-update
/// state; the function writes `dst = dst * y` in place.
fn run_multo_n4(dst_init: &[f64; SIZE], y: &[f64; SIZE]) -> [f64; SIZE] {
    let client = cpu_client();
    let dst_h = client.create_from_slice(f64::as_bytes(dst_init));
    let y_h = client.create_from_slice(f64::as_bytes(y));
    let read = dst_h.clone();
    unsafe {
        kernel_multo_n4::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(dst_h, SIZE),
            ArrayArg::from_raw_parts(y_h, SIZE),
        );
    }
    let bytes = client.read_one_unchecked(read);
    let result = f64::from_bytes(&bytes);
    let mut out = [0.0_f64; SIZE];
    out.copy_from_slice(&result[..SIZE]);
    out
}

/// Run `ctaylor_mul(x, y, out, n=4)` (out-of-place mul_set_n4).
fn run_mul_n4(x: &[f64; SIZE], y: &[f64; SIZE]) -> [f64; SIZE] {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(x));
    let y_h = client.create_from_slice(f64::as_bytes(y));
    let out_h = client.empty(SIZE * std::mem::size_of::<f64>());
    let read = out_h.clone();
    unsafe {
        kernel_mul_n4::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(out_h, SIZE),
            ArrayArg::from_raw_parts(x_h, SIZE),
            ArrayArg::from_raw_parts(y_h, SIZE),
        );
    }
    let bytes = client.read_one_unchecked(read);
    let result = f64::from_bytes(&bytes);
    let mut out = [0.0_f64; SIZE];
    out.copy_from_slice(&result[..SIZE]);
    out
}

/// Approximate equivalence: `multo_n4(dst, y)` ≈ `mul_set_n4(dst_init, y)`
/// to within 1e-13 relative tolerance. Per the module-header note, the two
/// routes differ in operand-summation bracketing (chained vs paired) so
/// 1-2 ULP drift is expected at f64; we check the value, not the bit pattern.
#[test]
fn multo_n4_approximately_matches_mul_set_n4() {
    let dst_in: [f64; SIZE] = [
        1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.5, 0.25, 0.125, 0.0625,
        0.03125, 0.015625, 0.0078125, 0.00390625,
    ];
    let y: [f64; SIZE] = [
        2.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5,
    ];
    let multo_result = run_multo_n4(&dst_in, &y);
    let mul_set_result = run_mul_n4(&dst_in, &y);

    for i in 0..SIZE {
        let m = multo_result[i];
        let s = mul_set_result[i];
        let abs_err = (m - s).abs();
        let scale = m.abs().max(s.abs()).max(1.0);
        let rel_err = abs_err / scale;
        assert!(
            rel_err < 1e-13,
            "multo_n4 vs mul_set_n4 rel-err at slot {}: multo = {:e}, mul_set = {:e}, rel_err = {:e}",
            i,
            m,
            s,
            rel_err
        );
    }
}

/// Edge case: y = identity polynomial `[1, 0, 0, ..., 0]` (multiplicative
/// identity). After multo, dst should equal dst_in.
#[test]
fn multo_n4_identity_polynomial_is_no_op() {
    let dst_in: [f64; SIZE] = [
        3.14, 2.71, 1.41, 1.62, 0.5, 0.25, 0.125, 0.0625, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.05,
    ];
    let y_identity: [f64; SIZE] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];
    let result = run_multo_n4(&dst_in, &y_identity);
    for i in 0..SIZE {
        assert_eq!(
            result[i].to_bits(),
            dst_in[i].to_bits(),
            "multo_n4 with y=identity changed slot {}: got {}, expected {}",
            i,
            result[i],
            dst_in[i]
        );
    }
}

/// Edge case: dst = identity, y = arbitrary. After multo, dst should equal y.
#[test]
fn multo_n4_dst_identity_returns_y() {
    let dst_identity: [f64; SIZE] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];
    let y: [f64; SIZE] = [
        2.5, 1.5, 0.5, 1.0, 0.25, 0.75, 1.25, 1.75, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8,
    ];
    let result = run_multo_n4(&dst_identity, &y);
    for i in 0..SIZE {
        assert_eq!(
            result[i].to_bits(),
            y[i].to_bits(),
            "multo_n4 with dst=identity should return y at slot {}: got {}, expected y[{}]={}",
            i,
            result[i],
            i,
            y[i]
        );
    }
}
