//! PW91-like exchange helpers — 1:1 port of
//! `xcfun-master/src/functionals/pw9xx.hpp:25-95`.
//!
//! # Purpose
//! Cross-family helpers used by PW91X / PW91K / PBEX / BECKEX / PBEC and most
//! GGA exchange functionals whose enhancement factor is expressed in terms of
//! the reduced gradient `s` (or equivalently `s² = S²(ρ, |∇ρ|²)`).
//!
//! # Sources
//! - `xcfun-master/src/functionals/pw9xx.hpp:39-46`  — `chi2`, `S2`
//! - `xcfun-master/src/functionals/pw9xx.hpp:51-70`  — `prefactor`, `pw91k_prefactor`
//! - `xcfun-master/src/functionals/pw9xx.hpp:73-95`  — `pw91xk_enhancement`
//!
//! # Status
//! - `s2` — **FULL BODY** (Wave 1, plan 03-01).
//! - `chi2`, `prefactor` — **FULL BODY** (Wave 2, plan 03-02 — W3 conversion).
//! - `pw91k_prefactor`, `pw91xk_enhancement` — **FULL BODY** (Wave 3, plan 03-03 — W7 conversion).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_scalar_mul;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_exp, ctaylor_pow, ctaylor_powi_2, ctaylor_reciprocal, ctaylor_sqrtx_asinh_sqrtx,
};

use super::constants::{NEG_C_SLATER_F64, S2_PREFACTOR_F64};

/// `chi2(ρ, |∇ρ|²) = |∇ρ|² / ρ^(8/3)` — reduced gradient squared (Becke χ²).
///
/// **FULL BODY** (Wave 2, plan 03-02 — W3 conversion).
#[cube]
pub fn chi2<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // FULL BODY — port of `pw9xx.hpp:39-41`:
    //   chi2(ρ, |∇ρ|²) = grad2 / pow(rho, 8/3) = grad2 · pow(rho, -8/3)
    //
    // Operation order (no mul_add per ACC-06):
    //   1. rho_m83 = pow(rho, -8/3)
    //   2. out     = grad2 · rho_m83
    let size = comptime!((1_u32 << n) as usize);
    let mut rho_m83 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(-8.0_f64 / 3.0_f64), &mut rho_m83, n);
    ctaylor_mul::<F>(grad2, &rho_m83, out, n);
}

/// `S2(ρ, |∇ρ|²) = S²_PREFACTOR · |∇ρ|² / ρ^(8/3)` — Perdew-Wang reduced
/// gradient squared, differentiable at `grad = 0`.
///
/// FULL BODY (called by `pbex::enhancement` — shipped this plan).
///
/// Port of `pw9xx.hpp:43-46`:
/// ```cpp
/// return grad / pow(rho, 8.0/3.0)
///        * pow(pow(6.0, 2.0/3.0) / (12 * pow(M_PI, 2.0/3.0)), 2.0);
/// ```
///
/// Operation order (strict left-to-right, no `mul_add` per ACC-06):
///   1. `rho_m83 = pow(rho, -8/3)`                  (ctaylor_pow, f64 exponent)
///   2. `ratio  = grad2 * rho_m83`                  (ctaylor_mul)
///   3. `out    = S2_PREFACTOR · ratio`             (ctaylor_scalar_mul)
///
/// Preconditions: `rho[0] > 0` (post-regularize); `grad2[0] >= 0`.
#[cube]
pub fn s2<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // rho_m83 = rho^(-8/3). f64 exponent cast at kernel time per SP-2.
    let mut rho_m83 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(-8.0_f64 / 3.0_f64), &mut rho_m83, n);

    // ratio = grad2 * rho_m83.
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(grad2, &rho_m83, &mut ratio, n);

    // out = S2_PREFACTOR · ratio.
    ctaylor_scalar_mul::<F>(&ratio, F::cast_from(S2_PREFACTOR_F64), out, n);
}

/// `prefactor(ρ)` — exchange-LSDA prefactor for PW91-family kernels.
/// `pw9xx.hpp:51-63`:
/// ```cpp
/// return -0.75 · 2^(1/3) · (3π²)^(1/3) · ρ^(4/3) / π
/// ```
///
/// **FULL BODY** — analytically equal to `NEG_C_SLATER · ρ^(4/3)` to f64
/// precision (constants verified by Phase 2 SLATERX: same value 0.9305257…).
///
/// Operation order (no mul_add per ACC-06):
///   1. rho_43 = pow(rho, 4/3)
///   2. out    = NEG_C_SLATER · rho_43
#[cube]
pub fn prefactor<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut rho_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(4.0_f64 / 3.0_f64), &mut rho_43, n);
    ctaylor_scalar_mul::<F>(&rho_43, F::cast_from(NEG_C_SLATER_F64), out, n);
}

/// `pw91k_prefactor(ρ) = CF · 2^(2/3) · ρ^(5/3)` per `pw9xx.hpp:66-70`.
///
/// **FULL BODY** (Wave 3, plan 03-03 — W7 conversion). Port of:
/// ```cpp
/// using xcfun_constants::CF;
/// return CF * pow(2.0, 2.0/3.0) * pow(rho, 5.0/3.0);
/// ```
///
/// `CF` = `(3/10) · (3π²)^(2/3)` = `2.871234000188191` (LYP_CF_F64 from
/// `constants.rs`). Combined with `2^(2/3) ≈ 1.587401052…`, the precomputed
/// f64 product `CF · 2^(2/3) = 4.5577013615...`.
///
/// Operation order (no mul_add per ACC-06):
///   1. `rho_53 = pow(rho, 5/3)`
///   2. `out    = (CF · 2^(2/3)) · rho_53`
#[cube]
pub fn pw91k_prefactor<F: Float>(
    rho: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut rho_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(5.0_f64 / 3.0_f64), &mut rho_53, n);
    // CF · 2^(2/3) = 2.871234000188191 · 1.5874010519681994 = 4.557799872345596
    const CF_TIMES_2_23: f64 = 4.557_799_872_345_596_0_f64;
    ctaylor_scalar_mul::<F>(&rho_53, F::cast_from(CF_TIMES_2_23), out, n);
}

/// `pw91xk_enhancement(s², a1, a2, a3, a4, a5, b)` — PW91-style enhancement
/// polynomial. Uses `ctaylor_sqrtx_asinh_sqrtx` from Wave 0 plan 03-00 (D-06)
/// to remain differentiable at `s = 0`.
///
/// **FULL BODY** (Wave 3, plan 03-03 — W7 conversion). Port of `pw9xx.hpp:73-94`
/// line-by-line:
/// ```cpp
/// num st2 = S2(rho, grad);   // caller passes s2_arr
/// num t1 = 1 + a1 * sqrtx_asinh_sqrtx(a2*a2 * st2) / a2;
/// num t2 = st2 * (a3 - a4 * exp(-a5 * st2));
/// num numerator = t1 + t2;
/// num denominator = t1 + b * st2 * st2;
/// return numerator / denominator;
/// ```
///
/// Operation order (strict left-to-right, no mul_add per ACC-06; each C++
/// intermediate becomes a named Rust `Array<F>` per ACC-03):
///   1. `a2_sq      = a2 · a2`                       (scalar)
///   2. `a2_sq_st2  = a2² · st2`                     (scalar_mul)
///   3. `sas        = sqrtx_asinh_sqrtx(a2_sq_st2)`  (D-06)
///   4. `sas_a1     = a1 · sas`                      (scalar_mul)
///   5. `sas_part   = sas_a1 / a2`                   (scalar_mul by 1/a2)
///   6. `t1         = 1 + sas_part`                  (CNST-bump)
///   7. `neg_a5_st2 = -a5 · st2`                     (scalar_mul)
///   8. `e          = exp(neg_a5_st2)`               (ctaylor_exp)
///   9. `a4e        = a4 · e`                        (scalar_mul)
///   10. `a3_min_a4e = (a3·CNST) - a4e`              (scalar_mul -1 + CNST-bump)
///   11. `t2        = st2 · a3_min_a4e`              (mul)
///   12. `numer     = t1 + t2`                       (add)
///   13. `st2_sq    = st2²`                          (powi_2)
///   14. `b_st2_sq  = b · st2²`                      (scalar_mul)
///   15. `denom     = t1 + b_st2_sq`                 (add)
///   16. `inv_denom = 1 / denom`                     (reciprocal)
///   17. `out       = numer · inv_denom`             (mul)
#[cube]
pub fn pw91xk_enhancement<F: Float>(
    s2_arr: &Array<F>,
    a1: F,
    a2: F,
    a3: F,
    a4: F,
    a5: F,
    b: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: a2_sq = a2 · a2 (scalar).
    let a2_sq = a2 * a2;

    // Step 2: a2_sq_st2 = a2² · st2.
    let mut a2_sq_st2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(s2_arr, a2_sq, &mut a2_sq_st2, n);

    // Step 3: sas = sqrtx_asinh_sqrtx(a2_sq_st2)   (D-06 Padé-branched).
    let mut sas = Array::<F>::new(size);
    ctaylor_sqrtx_asinh_sqrtx::<F>(&a2_sq_st2, &mut sas, n);

    // Step 4: sas_a1 = a1 · sas.
    let mut sas_a1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sas, a1, &mut sas_a1, n);

    // Step 5: sas_part = sas_a1 / a2.
    let inv_a2 = F::new(1.0) / a2;
    let mut sas_part = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sas_a1, inv_a2, &mut sas_part, n);

    // Step 6: t1 = 1 + sas_part (copy + CNST-bump).
    let mut t1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        t1[i] = sas_part[i];
    }
    t1[0] = t1[0] + F::new(1.0);

    // Step 7: neg_a5_st2 = -a5 · st2.
    let neg_a5 = F::new(0.0) - a5;
    let mut neg_a5_st2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(s2_arr, neg_a5, &mut neg_a5_st2, n);

    // Step 8: e = exp(neg_a5_st2).
    let mut e = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_a5_st2, &mut e, n);

    // Step 9: a4e = a4 · e.
    let mut a4e = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&e, a4, &mut a4e, n);

    // Step 10: a3_min_a4e = a3 - a4e (negate a4e then bump CNST by a3).
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut a3_min_a4e = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a4e, neg_one, &mut a3_min_a4e, n);
    a3_min_a4e[0] = a3_min_a4e[0] + a3;

    // Step 11: t2 = st2 · a3_min_a4e.
    let mut t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(s2_arr, &a3_min_a4e, &mut t2, n);

    // Step 12: numer = t1 + t2.
    let mut numer = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        numer[i] = t1[i] + t2[i];
    }

    // Step 13: st2_sq = st2².
    let mut st2_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(s2_arr, &mut st2_sq, n);

    // Step 14: b_st2_sq = b · st2².
    let mut b_st2_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&st2_sq, b, &mut b_st2_sq, n);

    // Step 15: denom = t1 + b_st2_sq.
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = t1[i] + b_st2_sq[i];
    }

    // Step 16: inv_denom = 1 / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // Step 17: out = numer · inv_denom.
    ctaylor_mul::<F>(&numer, &inv_denom, out, n);
}
