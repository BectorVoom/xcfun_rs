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
//! - `pw91k_prefactor`, `pw91xk_enhancement` — **SKELETON** (lands in 03-03 per W3).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_scalar_mul, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_pow;

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
/// SKELETON — full body lands in 03-03 Task 2 Step A (PW91K consumer).
#[cube]
pub fn pw91k_prefactor<F: Float>(
    rho: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-03 Task 2 Step A.
    let _ = rho;
    ctaylor_zero::<F>(out, n);
}

/// `pw91xk_enhancement(s², a1, a2, a3, a4, a5, b)` — PW91-style enhancement
/// polynomial. Uses `ctaylor_sqrtx_asinh_sqrtx` from Wave 0 plan 03-00 (D-06)
/// to remain differentiable at `s = 0`.
///
/// Port of `pw9xx.hpp:73-94`:
/// ```cpp
/// num t1 = 1 + a1 · sqrtx_asinh_sqrtx(a2² · S²) / a2;
/// num t2 = S² · (a3 - a4 · exp(-a5 · S²));
/// return (t1 + t2) / (t1 + b · S²²);
/// ```
///
/// SKELETON — full body lands in 03-03 Task 2 Step B (PW91X/PW91K consumer).
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
    // SKELETON — full body lands in 03-03 Task 2 Step B.
    let _ = s2_arr;
    let _ = a1;
    let _ = a2;
    let _ = a3;
    let _ = a4;
    let _ = a5;
    let _ = b;
    ctaylor_zero::<F>(out, n);
}
