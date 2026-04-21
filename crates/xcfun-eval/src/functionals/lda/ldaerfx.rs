//! Short-range LDA exchange (range-separated). **LDA-06.**
//!
//! # Source
//! - `xcfun-master/src/functionals/ldaerfx.cpp:24-73` (esrx_ldaerfspin + lda_erfx)
//!
//! # Formula (4-branch on a = mu/(2*akf))
//! - `a < 1e-9`:    `-3/8 * rhoa * pow(24*rhoa/PI, 1/3)`
//! - `1e-9 <= a < 100`: full short-range expression with erf(0.5/a) + exp(-1/(4a²))
//! - `100 <= a < 1e9`: `-(rhoa * pow(24*rhoa/PI, 1/3)) / (96*a²)`
//! - `a >= 1e9`:    `0`
//!
//! # D-24 Tier-2 Tolerance Override (USER-APPROVED 2026-04-20)
//!
//! Upstream `xcfun-master/src/functionals/ldaerfx.cpp:66` uses `test_threshold = 1e-7`.
//! cubecl 0.10-pre.3 `Float::erf` is a polyfill (~1.3e-8 ULP) that propagates to
//! ~2e-8 final-output rel-error vs C++ libm `erf` in the LDAERF chain (RESEARCH
//! §"D-19 LDAERF Tolerance Analysis"). Per CONTEXT D-24, Phase 2 tier-2 uses 1e-7
//! for this functional, MATCHING upstream's own self-test threshold. This is NOT
//! silent widening — report.html (Plan 02-06) annotates LDAERFX rows with
//! `1e-7 (D-24 override)` for full transparency.
//!
//! Phase 6 revisits with libm-call hybrid when CUDA/Wgpu erf drift also enters scope.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_erf, ctaylor_exp, ctaylor_pow, ctaylor_powi_3, ctaylor_reciprocal,
};

use crate::density_vars::DensVarsDev;

// ldaerfx.cpp line 26: const parameter ckf = 3.093667726280136;
// f64 storage + F::cast_from at kernel-time: f32 truncates to ~8 digits,
// which cascades into ~1e-7 rel-drift that tier-2 order-2 cancellation amplifies
// to catastrophic failure near branch B/C boundary (a ≈ 94–100).
const CKF_F64: f64 = 3.093667726280136_f64;

// Range-separation parameter — Phase 2 hard-codes the default 0.4 (matches xcfun's
// XC_RANGESEP_MU default). 0.4 is EXACTLY representable in f32/f64, so we keep
// a plain f32 for the scalar-mul fast path; no precision loss here.
const RANGESEP_MU_F32: f32 = 0.4_f32;

// 24/PI — C++ computes `24.0 / M_PI` at runtime; glibc M_PI is f64.
// 24 / pi_f64 = 7.639437268410976 (Python verified).
const TWENTY_FOUR_OVER_PI_F64: f64 = 7.639437268410976_f64;

// sqrt(PI) — C++ uses `sqrt(M_PI)`; libm value is 1.7724538509055159.
const SQRT_PI_F64: f64 = 1.7724538509055159_f64;

// -3/8 = -0.375 — EXACTLY representable in f32/f64; no drift.
const NEG_THREE_EIGHTHS: f32 = -0.375_f32;

// ---------------------------------------------------------------------------
//  esrx_ldaerfspin — per-spin short-range LDA exchange.
//  Port of `esrx_ldaerfspin(na, mu)` from ldaerfx.cpp:24-48.
//
//  Host convention: caller supplies `na` CTaylor (either d.a or d.b).
//  Range-separation mu is baked in as RANGESEP_MU_F32.
// ---------------------------------------------------------------------------

#[cube]
fn esrx_ldaerfspin<F: Float>(
    na: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // rhoa = 2 * na
    let mut rhoa = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(na, F::new(2.0), &mut rhoa, n);

    // akf = ckf * pow(rhoa, 1/3)
    // Use F::cast_from for the 1/3 exponent (f32 1/3 truncates) and the ckf scalar.
    let mut rhoa_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(&rhoa, F::cast_from(1.0_f64 / 3.0_f64), &mut rhoa_13, n);
    let mut akf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rhoa_13, F::cast_from(CKF_F64), &mut akf, n);

    // a = mu / (2 * akf)  →  a = mu * (1 / (2 * akf))
    let mut two_akf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&akf, F::new(2.0), &mut two_akf, n);
    let mut inv_two_akf = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&two_akf, &mut inv_two_akf, n);
    let mut a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_two_akf, F::new(RANGESEP_MU_F32), &mut a, n);

    // a2 = a * a; a3 = a2 * a  (used in multiple branches)
    let mut a2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&a, &a, &mut a2, n);
    let mut a3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&a2, &a, &mut a3, n);

    // Common factor: pow(24 * rhoa / PI, 1/3)
    //   step 1: twenty_four_rhoa_over_pi = (24/PI) * rhoa
    //   step 2: result = pow(..., 1/3)
    let mut twenty_four_rhoa_over_pi = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &rhoa,
        F::cast_from(TWENTY_FOUR_OVER_PI_F64),
        &mut twenty_four_rhoa_over_pi,
        n,
    );
    let mut pow_24_rhoa_pi_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(
        &twenty_four_rhoa_over_pi,
        F::cast_from(1.0_f64 / 3.0_f64),
        &mut pow_24_rhoa_pi_13,
        n,
    );

    // `rhoa * pow(24*rhoa/PI, 1/3)` — shared scale across branches A, B, C.
    let mut rhoa_pow = Array::<F>::new(size);
    ctaylor_mul::<F>(&rhoa, &pow_24_rhoa_pi_13, &mut rhoa_pow, n);

    // Runtime dispatch on scalar a[0]. Each branch writes `out` fully.
    let a_scalar = a[0];
    if a_scalar < F::new(1e-9_f32) {
        // Branch A: `-3/8 * rhoa * pow(24*rhoa/PI, 1/3)` (small-a limit).
        ctaylor_scalar_mul::<F>(&rhoa_pow, F::new(NEG_THREE_EIGHTHS), out, n);
    } else if a_scalar < F::new(100.0) {
        // Branch B: full expression (intermediate a). Operation order:
        //   inner = sqrt(PI) * erf(0.5/a)
        //           + (2*a - 4*a³) * exp(-0.25 / a²)
        //           - 3*a
        //           + 4*a³
        //   bracket = 3/8 - a * inner
        //   out = -(rhoa * pow(24*rhoa/PI, 1/3)) * bracket

        // inv_a = 1 / a; half_inv_a = 0.5 / a = 0.5 * inv_a
        let mut inv_a = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&a, &mut inv_a, n);
        let mut half_inv_a = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&inv_a, F::new(0.5), &mut half_inv_a, n);

        // erf(0.5/a) and sqrt(PI) * erf(0.5/a)
        let mut erf_val = Array::<F>::new(size);
        ctaylor_erf::<F>(&half_inv_a, &mut erf_val, n);
        let mut sqrt_pi_erf = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&erf_val, F::cast_from(SQRT_PI_F64), &mut sqrt_pi_erf, n);

        // 2*a - 4*a³
        let mut two_a = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&a, F::new(2.0), &mut two_a, n);
        let mut four_a3 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&a3, F::new(4.0), &mut four_a3, n);
        let mut two_a_m_four_a3 = Array::<F>::new(size);
        ctaylor_sub::<F>(&two_a, &four_a3, &mut two_a_m_four_a3, n);

        // -0.25 / a² = -0.25 * (1/a²) = -0.25 * inv_a * inv_a
        let mut inv_a2 = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&a2, &mut inv_a2, n);
        let mut neg_quarter_inv_a2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&inv_a2, F::new(-0.25), &mut neg_quarter_inv_a2, n);

        // exp(-0.25 / a²)
        let mut exp_val = Array::<F>::new(size);
        ctaylor_exp::<F>(&neg_quarter_inv_a2, &mut exp_val, n);

        // (2*a - 4*a³) * exp(...)
        let mut two_a_m_4a3_exp = Array::<F>::new(size);
        ctaylor_mul::<F>(&two_a_m_four_a3, &exp_val, &mut two_a_m_4a3_exp, n);

        // -3 * a
        let mut neg_three_a = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&a, F::new(-3.0), &mut neg_three_a, n);

        // inner = sqrt_pi_erf + two_a_m_4a3_exp - 3*a + 4*a³
        //   (C++ left-to-right: sqrt(PI)*erf(0.5/a) + (2a-4a³)*exp(-0.25/a²) - 3a + 4a³)
        //   Note in C++ it's `... - 3.0*a + 4.0*a3`, so `- 3a` then `+ 4a³`.
        //   We build: step1 = sqrt_pi_erf + two_a_m_4a3_exp
        //             step2 = step1 + neg_three_a      (== step1 - 3a)
        //             step3 = step2 + four_a3          (== step2 + 4a³)
        let mut step1 = Array::<F>::new(size);
        ctaylor_add::<F>(&sqrt_pi_erf, &two_a_m_4a3_exp, &mut step1, n);
        let mut step2 = Array::<F>::new(size);
        ctaylor_add::<F>(&step1, &neg_three_a, &mut step2, n);
        let mut inner = Array::<F>::new(size);
        ctaylor_add::<F>(&step2, &four_a3, &mut inner, n);

        // a * inner
        let mut a_inner = Array::<F>::new(size);
        ctaylor_mul::<F>(&a, &inner, &mut a_inner, n);

        // bracket = 3/8 - a*inner   (i.e. scalar 0.375 - a_inner)
        let mut three_eighths_const = Array::<F>::new(size);
        ctaylor_zero::<F>(&mut three_eighths_const, n);
        three_eighths_const[0] = F::new(0.375_f32);
        let mut bracket = Array::<F>::new(size);
        ctaylor_sub::<F>(&three_eighths_const, &a_inner, &mut bracket, n);

        // neg_rhoa_pow = -1 * rhoa_pow
        let mut neg_rhoa_pow = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&rhoa_pow, F::new(-1.0), &mut neg_rhoa_pow, n);

        // out = neg_rhoa_pow * bracket
        ctaylor_mul::<F>(&neg_rhoa_pow, &bracket, out, n);
    } else if a_scalar < F::new(1e9_f32) {
        // Branch C: `-(rhoa * pow(24*rhoa/PI, 1/3)) / (96 * a²)` (large-a expansion).
        let mut denom = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&a2, F::new(96.0), &mut denom, n);
        let mut inv_denom = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
        let mut tmp = Array::<F>::new(size);
        ctaylor_mul::<F>(&rhoa_pow, &inv_denom, &mut tmp, n);
        ctaylor_scalar_mul::<F>(&tmp, F::new(-1.0), out, n);
    } else {
        // Branch D: 0  (very-large-a limit).
        ctaylor_zero::<F>(out, n);
    }

    // Silence unused constants when some branches are not exercised.
    let _ = (a3, rhoa_13);
}

// Placate clippy — ctaylor_powi_3 is imported for forward-compat; branches above
// use explicit a2 * a via ctaylor_mul for strict algorithmic-identity with C++.
#[allow(dead_code)]
fn _force_powi_3_import<F: Float>(x: &Array<F>, out: &mut Array<F>, n: u32) {
    ctaylor_powi_3::<F>(x, out, n);
}

/// Short-range LDA exchange kernel. 1:1 port of `ldaerfx.cpp:49-52`:
/// `return 0.5 * (esrx_ldaerfspin(d.a, mu) + esrx_ldaerfspin(d.b, mu));`
#[cube]
pub fn ldaerfx_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut esrx_a = Array::<F>::new(size);
    let mut esrx_b = Array::<F>::new(size);
    esrx_ldaerfspin::<F>(&d.a, &mut esrx_a, n);
    esrx_ldaerfspin::<F>(&d.b, &mut esrx_b, n);
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&esrx_a, &esrx_b, &mut sum, n);
    ctaylor_scalar_mul::<F>(&sum, F::new(0.5), out, n);
}
