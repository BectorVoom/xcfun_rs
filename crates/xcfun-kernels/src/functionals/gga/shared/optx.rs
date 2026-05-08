//! OPTX family helpers — port target of
//! `xcfun-master/src/functionals/optx.cpp:18-26` + `optxcorr.cpp:18-33`.
//!
//! # Purpose
//! Handley-Cohen OPTX exchange enhancement factor and its correlation analog.
//!
//! # Wave 3 status (03-03)
//! Both helpers ship as **FULL BODIES** (Wave 3, plan 03-03 — W3 conversion
//! complete; SKELETON markers removed).
//!
//! # Sources
//! - `xcfun-master/src/functionals/optx.cpp:20`     — `g_xa = γ · gaa · pow(a, -8/3)`
//! - `xcfun-master/src/functionals/optx.cpp:22-24`  — full optx body
//! - `xcfun-master/src/functionals/optxcorr.cpp:30-32` — correction-only body

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_scalar_mul;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_powi_2, ctaylor_reciprocal};

use super::constants::OPTX_GAMMA_F64;

/// `g_xa2(ρ, |∇ρ|²) = γ · |∇ρ|² · ρ^(-8/3)` — OPTX's scaled reduced-gradient
/// analog (same structure as `pw91_like::chi2` with a γ prefactor).
///
/// **FULL BODY** (Wave 3, plan 03-03).
///
/// Port of `optx.cpp:20`:
/// ```cpp
/// num g_xa2 = gamma * d.gaa * pow(d.a, -8.0/3.0);
/// ```
///
/// Operation order (no mul_add per ACC-06):
///   1. `rho_m83 = pow(rho, -8/3)`           (ctaylor_pow)
///   2. `prod    = grad2 · rho_m83`           (ctaylor_mul)
///   3. `out     = γ · prod`                  (scalar_mul)
#[cube]
pub fn g_xa2<F: Float>(rho: &Array<F>, grad2: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: rho_m83 = pow(rho, -8/3).
    let mut rho_m83 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(-8.0_f64 / 3.0_f64), &mut rho_m83, n);

    // Step 2: prod = grad2 · rho_m83.
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(grad2, &rho_m83, &mut prod, n);

    // Step 3: out = γ · prod.
    ctaylor_scalar_mul::<F>(&prod, F::cast_from(OPTX_GAMMA_F64), out, n);
}

/// `optx_enhancement(g_xa2) = a2 · g_xa2² / (1 + g_xa2)²` — Handley-Cohen
/// OPTX correction factor (the second term inside the C++ bracket).
///
/// **FULL BODY** (Wave 3, plan 03-03).
///
/// Port of `optx.cpp:23-24` (the `a2 · pow(g_xa2, 2) · pow(1 + g_xa2, -2)` term):
/// ```cpp
/// a2 * pow(g_xa2, 2) * pow(1 + g_xa2, -2)
/// ```
///
/// `a1` is unused here (OPTX folds `a1 · c_slater` separately into the LDA
/// part) but is part of the signature for API symmetry — caller passes
/// `OPTX_A1_F64` and `OPTX_A2_F64` from `constants.rs`.
///
/// Operation order (no mul_add per ACC-06):
///   1. `g_sq    = g_xa2²`                          (ctaylor_powi_2)
///   2. `one_g   = 1 + g_xa2`                       (copy + CNST-bump)
///   3. `one_g2  = one_g²`                          (ctaylor_powi_2)
///   4. `inv_og2 = 1 / one_g²`                      (ctaylor_reciprocal)
///   5. `prod    = g_sq · inv_og2`                  (ctaylor_mul)
///   6. `out     = a2 · prod`                       (scalar_mul)
#[cube]
pub fn optx_enhancement<F: Float>(
    gx: &Array<F>,
    a1: F,
    a2: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a1; // a1 is consumed by the LDA branch of the OPTX kernel, not here.
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: g_sq = g_xa2².
    let mut g_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(gx, &mut g_sq, n);

    // Step 2: one_g = 1 + g_xa2 (copy + CNST-bump).
    let mut one_g = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_g[i] = gx[i];
    }
    one_g[0] = one_g[0] + F::new(1.0);

    // Step 3: one_g2 = one_g².
    let mut one_g2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&one_g, &mut one_g2, n);

    // Step 4: inv_og2 = 1 / one_g².
    let mut inv_og2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_g2, &mut inv_og2, n);

    // Step 5: prod = g_sq · inv_og2.
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&g_sq, &inv_og2, &mut prod, n);

    // Step 6: out = a2 · prod.
    ctaylor_scalar_mul::<F>(&prod, a2, out, n);
}
