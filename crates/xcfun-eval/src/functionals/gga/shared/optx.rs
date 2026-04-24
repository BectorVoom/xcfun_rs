//! OPTX family helpers — port target of
//! `xcfun-master/src/functionals/optx.cpp:18-40` + `optxcorr.cpp:18-35`.
//!
//! # Purpose
//! Handley-Cohen OPTX exchange enhancement factor and its correlation analog.
//!
//! # Wave 1 status (03-01)
//! Both helpers are SKELETONS; full bodies land in 03-03 Task 1 Step A.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_zero;

/// `g_xa2(ρ, |∇ρ|²) = γ · |∇ρ|² · ρ^(-8/3)` — OPTX's scaled reduced-gradient
/// analog (same structure as `pw91_like::chi2` with a γ prefactor).
///
/// SKELETON — full body lands in 03-03 Task 1 Step A (OPTX consumer).
#[cube]
pub fn g_xa2<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-03 Task 1 Step A.
    let _ = rho;
    let _ = grad2;
    ctaylor_zero::<F>(out, n);
}

/// `optx_enhancement(gx, a1, a2) = (gx / (1 + gx))² · (1 + a1·… + a2·…)`
/// — Handley-Cohen enhancement factor. Exact polynomial structure per
/// `optx.cpp:18-40`.
///
/// SKELETON — full body lands in 03-03 Task 1 Step A (OPTX consumer).
#[cube]
pub fn optx_enhancement<F: Float>(
    gx: &Array<F>,
    a1: F,
    a2: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-03 Task 1 Step A.
    let _ = gx;
    let _ = a1;
    let _ = a2;
    ctaylor_zero::<F>(out, n);
}
