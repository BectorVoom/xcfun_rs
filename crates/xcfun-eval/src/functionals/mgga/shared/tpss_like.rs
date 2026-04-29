//! TPSS / revTPSS exchange + correlation enhancement helpers.
//!
//! Phase 4 plan 04-01 Wave 1 — FULL BODIES replacing the Wave-0 skeletons.
//!
//! # Sources
//! - `tpssx_eps.hpp:1-60`     — `fx_unif`, `x`, `F_x`
//! - `tpssc_eps.hpp:1-62`     — `C`, `epsc_summax`, `epsc_revpkzb`, `tpssc_eps`
//! - `revtpssx_eps.hpp:1-65`  — `epsx_unif`, `x`, `F_x`, `revtpssx_eps`
//! - `revtpssc_eps.hpp:1-111` — `revtpssA`, `revtpssH`, `revtpss_beta`,
//!                               `revtpss_pbec_eps`, `revtpss_pbec_eps_polarized`,
//!                               `C`, `epsc_summax`, `epsc_revpkzb`, `revtpssc_eps`

#![allow(non_snake_case)]

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_exp, ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_2, ctaylor_powi_3,
    ctaylor_reciprocal, ctaylor_sqrt,
};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::PBEC_D2_PREFACTOR_F64;
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;
use crate::functionals::mgga::shared::constants::{
    REVTPSS_B_F64, REVTPSS_C_F64, REVTPSS_E_F64, REVTPSS_KAPPA_F64, REVTPSS_MU_F64,
    REVTPSS_SQRT_E_F64, TPSS_B_F64, TPSS_C_F64, TPSS_DD_F64, TPSS_E_F64, TPSS_KAPPA_F64,
    TPSS_MU_F64, TPSS_SQRT_E_F64,
};

// ---------------------------------------------------------------------------
//  Shared helper: ufunc(zeta, p) = (1+zeta)^p + (1-zeta)^p
//  Used by TPSS C factor; local to this module until scan_like Wave-2 fills it.
// ---------------------------------------------------------------------------

/// `ufunc(zeta, p) = (1+zeta)^p + (1-zeta)^p`.
/// Port of `specmath.hpp:35-37`.
#[cube]
fn ufunc_p<F: Float>(zeta: &Array<F>, p: F, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    // pz = 1 + zeta
    let mut pz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        pz[i] = zeta[i];
    }
    pz[0] = pz[0] + F::new(1.0);
    // mz = 1 - zeta
    let mut mz = Array::<F>::new(size);
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(zeta, neg_one, &mut mz, n);
    mz[0] = mz[0] + F::new(1.0);
    // pz^p
    let mut pzp = Array::<F>::new(size);
    ctaylor_pow::<F>(&pz, p, &mut pzp, n);
    // mz^p
    let mut mzp = Array::<F>::new(size);
    ctaylor_pow::<F>(&mz, p, &mut mzp, n);
    // out = pzp + mzp
    ctaylor_add::<F>(&pzp, &mzp, out, n);
}

// ---------------------------------------------------------------------------
//  TPSS exchange helpers (tpssx_eps.hpp)
// ---------------------------------------------------------------------------

/// `fx_unif(d) = (-3/4 * (3/π)^(1/3)) * d^(4/3)`.
/// Port of `tpssx_eps.hpp:23-25`.
/// Constant: `-0.75 * cbrt(3/π) = -0.7385587663820223`.
#[cube]
pub fn tpss_fx_unif<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // (-3/4) * (3/π)^(1/3) computed in f64.
    const COEFF: f64 = -0.738_558_766_382_022_3_f64;
    let size = comptime!((1_u32 << n) as usize);
    let mut rho_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(4.0_f64 / 3.0_f64), &mut rho_43, n);
    ctaylor_scalar_mul::<F>(&rho_43, F::cast_from(COEFF), out, n);
}

/// TPSS exchange `x` numerator-of-enhancement composite.
/// Port of `tpssx_eps.hpp:27-51`.
#[cube]
pub fn tpss_x<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // p0 = 1 / (4 * (3π²)^(2/3) * n^(8/3))
    // = 1/(4*(3π²)^(2/3)) * n^(-8/3)
    // (3π²)^(2/3) computed: 3*PI^2=29.608..., cbrt(29.608...)^2=9.5751...
    // 4*(3π²)^(2/3) = 38.2831...
    // 1/(4*(3π²)^(2/3)) = 0.026100...
    const FOUR_3PI2_23_F64: f64 = 38.283_120_002_509_214_f64; // 4*(3π²)^(2/3)
    let mut n_m83 = Array::<F>::new(size);
    ctaylor_pow::<F>(d_n, F::cast_from(-8.0_f64 / 3.0_f64), &mut n_m83, n);
    let mut p0 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n_m83, F::cast_from(1.0_f64 / FOUR_3PI2_23_F64), &mut p0, n);

    // p = gnn * p0
    let mut p = Array::<F>::new(size);
    ctaylor_mul::<F>(d_gnn, &p0, &mut p, n);

    // tauw = gnn / (8*n)
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(d_n, &mut inv_n, n);
    let mut tauw_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(d_gnn, &inv_n, &mut tauw_raw, n);
    let mut tauw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&tauw_raw, F::cast_from(1.0_f64 / 8.0_f64), &mut tauw, n);

    // z = tauw / tau
    let mut inv_tau = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(d_tau, &mut inv_tau, n);
    let mut z = Array::<F>::new(size);
    ctaylor_mul::<F>(&tauw, &inv_tau, &mut z, n);

    // z2 = z^2
    let mut z2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&z, &mut z2, n);

    // tau_unif = 0.3 * (3π²)^(2/3) * n^(5/3)
    // (3π²)^(2/3) = 9.5751... → 0.3 * 9.5751 = 2.8725...
    const COEFF_TAU_UNIF: f64 = 2.871_234_000_188_191_f64; // 0.3*(3π²)^(2/3) = CF from LYP
    let mut n_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(d_n, F::cast_from(5.0_f64 / 3.0_f64), &mut n_53, n);
    let mut tau_unif = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n_53, F::cast_from(COEFF_TAU_UNIF), &mut tau_unif, n);

    // alpha = (tau - tauw) / tau_unif
    let mut tau_m_tauw = Array::<F>::new(size);
    ctaylor_sub::<F>(d_tau, &tauw, &mut tau_m_tauw, n);
    let mut inv_tau_unif = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&tau_unif, &mut inv_tau_unif, n);
    let mut alpha = Array::<F>::new(size);
    ctaylor_mul::<F>(&tau_m_tauw, &inv_tau_unif, &mut alpha, n);

    // q_b = (9/20) * (alpha-1) / sqrt(1 + b*alpha*(alpha-1)) + 2*p/3
    // alpha - 1
    let mut alpha_m1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        alpha_m1[i] = alpha[i];
    }
    alpha_m1[0] = alpha_m1[0] - F::new(1.0);

    // b*alpha*(alpha-1)
    let mut b_alpha = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&alpha, F::cast_from(TPSS_B_F64), &mut b_alpha, n);
    let mut b_alpha_am1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b_alpha, &alpha_m1, &mut b_alpha_am1, n);

    // 1 + b*alpha*(alpha-1)
    let mut one_plus = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus[i] = b_alpha_am1[i];
    }
    one_plus[0] = one_plus[0] + F::new(1.0);

    // sqrt(...)
    let mut sq = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&one_plus, &mut sq, n);

    // inv_sq
    let mut inv_sq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&sq, &mut inv_sq, n);

    // (9/20) * (alpha-1) / sqrt(...)
    let mut frac1_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&alpha_m1, &inv_sq, &mut frac1_raw, n);
    let mut frac1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&frac1_raw, F::cast_from(9.0_f64 / 20.0_f64), &mut frac1, n);

    // 2*p/3
    let mut two_p_3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(2.0_f64 / 3.0_f64), &mut two_p_3, n);

    let mut q_b = Array::<F>::new(size);
    ctaylor_add::<F>(&frac1, &two_p_3, &mut q_b, n);

    // x_a = p * (10/81 + c*z2/(1+z2)^2)
    // (1+z2)
    let mut one_z2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_z2[i] = z2[i];
    }
    one_z2[0] = one_z2[0] + F::new(1.0);

    // (1+z2)^2
    let mut one_z2_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&one_z2, &mut one_z2_sq, n);

    // c*z2 / (1+z2)^2
    let mut cz2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z2, F::cast_from(TPSS_C_F64), &mut cz2, n);
    let mut inv_one_z2_sq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_z2_sq, &mut inv_one_z2_sq, n);
    let mut cz2_frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&cz2, &inv_one_z2_sq, &mut cz2_frac, n);

    // (10/81 + c*z2/(1+z2)^2) as CTaylor: add 10/81 to CNST slot
    let mut bracket = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket[i] = cz2_frac[i];
    }
    bracket[0] = bracket[0] + F::cast_from(10.0_f64 / 81.0_f64);

    // x_a = p * bracket
    let mut x_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&p, &bracket, &mut x_a, n);

    // += 146/2025 * q_b^2
    let mut q_b_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&q_b, &mut q_b_sq, n);
    let mut q_b_sq_coeff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&q_b_sq, F::cast_from(146.0_f64 / 2025.0_f64), &mut q_b_sq_coeff, n);
    let mut x_a2 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a, &q_b_sq_coeff, &mut x_a2, n);

    // -= 73/405 * q_b * gnn * sqrt(0.5*0.36*pow(8*n*tau,-2) + 0.5*p0*p0)
    // sqrt arg: 0.5*0.6^2*(8*n*tau)^(-2) + 0.5*p0^2
    // = 0.18*(8*n*tau)^(-2) + 0.5*p0^2
    // (8*n*tau)
    let mut eight_n_tau_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(d_n, d_tau, &mut eight_n_tau_raw, n);
    let mut eight_n_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_raw, F::cast_from(8.0_f64), &mut eight_n_tau, n);

    // (8*n*tau)^(-2)
    let mut eight_n_tau_m2_raw = Array::<F>::new(size);
    ctaylor_pow::<F>(&eight_n_tau, F::cast_from(-2.0_f64), &mut eight_n_tau_m2_raw, n);
    let mut eight_n_tau_m2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_m2_raw, F::cast_from(0.5_f64 * 0.36_f64), &mut eight_n_tau_m2, n);

    // p0^2
    let mut p0_sq_raw = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&p0, &mut p0_sq_raw, n);
    let mut p0_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p0_sq_raw, F::cast_from(0.5_f64), &mut p0_sq, n);

    // sqrt_arg = 0.18*(8nt)^-2 + 0.5*p0^2
    let mut sqrt_arg = Array::<F>::new(size);
    ctaylor_add::<F>(&eight_n_tau_m2, &p0_sq, &mut sqrt_arg, n);

    let mut sq2 = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&sqrt_arg, &mut sq2, n);

    // 73/405 * q_b * gnn * sq2
    let mut term3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&q_b, d_gnn, &mut term3, n);
    let mut term3b_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&term3, &sq2, &mut term3b_raw, n);
    let mut term3b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&term3b_raw, F::cast_from(73.0_f64 / 405.0_f64), &mut term3b, n);
    let mut x_a3 = Array::<F>::new(size);
    ctaylor_sub::<F>(&x_a2, &term3b, &mut x_a3, n);

    // += (p*10/81)^2 / kappa
    let mut p_10_81 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(10.0_f64 / 81.0_f64), &mut p_10_81, n);
    let mut p_10_81_sq_raw = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&p_10_81, &mut p_10_81_sq_raw, n);
    let mut p_10_81_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p_10_81_sq_raw, F::cast_from(1.0_f64 / TPSS_KAPPA_F64), &mut p_10_81_sq, n);
    let mut x_a4 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a3, &p_10_81_sq, &mut x_a4, n);

    // += 2*sqrt(e)*0.36*z2*10/81 + e*mu*p^3
    // 2*sqrt(e)*0.36*10/81 — constant folded at host, passed as F scalar
    let mut z2_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &z2,
        F::cast_from(2.0_f64 * TPSS_SQRT_E_F64 * 0.36_f64 * 10.0_f64 / 81.0_f64),
        &mut z2_term,
        n,
    );
    let mut x_a5 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a4, &z2_term, &mut x_a5, n);

    // e*mu*p^3
    let mut p3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&p, &mut p3, n);
    let mut p3_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p3, F::cast_from(TPSS_E_F64 * TPSS_MU_F64), &mut p3_term, n);
    let mut x_a6 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a5, &p3_term, &mut x_a6, n);

    // divide by (1 + sqrt(e)*p)^2
    let mut sp = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(TPSS_SQRT_E_F64), &mut sp, n);
    sp[0] = sp[0] + F::new(1.0);
    let mut sp_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&sp, &mut sp_sq, n);
    let mut inv_sp_sq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&sp_sq, &mut inv_sp_sq, n);

    ctaylor_mul::<F>(&x_a6, &inv_sp_sq, out, n);
}

/// TPSS `F_x = 1 + κ - κ/(1 + x/κ)`. Port of `tpssx_eps.hpp:54-59`.
#[cube]
pub fn tpss_F_x<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut xpz = Array::<F>::new(size);
    tpss_x::<F>(d_n, d_gnn, d_tau, &mut xpz, n);

    // 1 + x/kappa
    let mut x_over_k = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xpz, F::cast_from(1.0_f64 / TPSS_KAPPA_F64), &mut x_over_k, n);
    x_over_k[0] = x_over_k[0] + F::new(1.0);

    // kappa / (1 + x/kappa)
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&x_over_k, &mut inv_denom, n);
    let mut k_over_denom = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_denom, F::cast_from(TPSS_KAPPA_F64), &mut k_over_denom, n);

    // out = 1 + kappa - kappa/(1 + x/kappa)
    // = (1 + kappa) - k_over_denom
    let one_plus_k = 1.0_f64 + TPSS_KAPPA_F64;
    #[unroll]
    for i in 0..size {
        out[i] = k_over_denom[i];
    }
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&k_over_denom, neg_one, out, n);
    out[0] = out[0] + F::cast_from(one_plus_k);
}

// ---------------------------------------------------------------------------
//  revTPSS exchange helpers (revtpssx_eps.hpp)
// ---------------------------------------------------------------------------

/// `epsx_unif(n) = -3 * cbrt(3π²n) / (4π)`.
/// Port of `revtpssx_eps.hpp:22-24`.
/// = -(3/(4π)) * (3π²)^(1/3) * n^(1/3)
/// Constant: -(3/(4π)) * (3π²)^(1/3) = -0.9305257363491001 * 0.5... wait
/// Actually: -3/(4π) * (3π²)^(1/3) = -(3/(4π)) * cbrt(3π²)
/// 3π² ≈ 29.608; cbrt(29.608) ≈ 3.094; 3/(4π) ≈ 0.23873
/// → -0.23873 * 3.094 ≈ -0.7386... but wait, that's the TPSS coeff.
/// Let me recalculate: -3 * cbrt(3π²*n) / (4π) = -3/(4π) * (3π²)^(1/3) * n^(1/3)
/// -3/(4π) = -0.238732...; cbrt(3π²) = 3.09366...; product = -0.738558...
/// Hmm, that equals fx_unif(n^(4/3)) / n = -(3/4)*(3/π)^(1/3) * n^(1/3) different!
/// Actually: -3*cbrt(3π²*n)/(4π) = -(3/(4π))*(3π²)^(1/3)*n^(1/3)
/// but fx_unif(n) = -(3/4)*(3/π)^(1/3)*n^(4/3).
/// For revTPSS: epsxunif_a = epsx_unif(2a) = -3*cbrt(3π²*2a)/(4π)
/// Then: revtpssx = epsxunif_a*Fxa*d.a + epsxunif_b*Fxb*d.b
///                = -3/(4π)*(3π²)^(1/3)*cbrt(2a)*Fxa*a + ...
/// = -3*cbrt(3π²)/(4π) * cbrt(2)*cbrt(a)*a * Fxa + ...
/// = -3*cbrt(3π²)/(4π) * cbrt(2) * a^(4/3) * Fxa + ...
/// So epsx_unif(2a) = -3/(4π)*(3π²*2a)^(1/3) = -(3/(4π))*(3π²)^(1/3)*cbrt(2)*a^(1/3)
///
/// Coefficient: -(3/(4π)) * (3π²)^(1/3)
/// 3π² = 29.6088...; (3π²)^(1/3) = 3.09366...; 3/(4π) = 0.23873...
/// product = -0.73856... (same as TPSS! But used differently with n^(1/3) not n^(4/3))
#[cube]
pub fn revtpss_epsx_unif<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // -3 * cbrt(3π²*rho) / (4π) = C * rho^(1/3)
    // C = -3/(4π) * (3π²)^(1/3) = -0.738558766382...  (same magnitude as TPSS fx_unif coeff)
    // WAIT - tpss fx_unif coeff: -0.75*(3/π)^(1/3) = -0.75*0.98474... = -0.73856...
    // revTPSS: -3/(4π)*(3π²)^(1/3) = -0.23873*3.09366 = -0.73856...
    // They ARE the same constant! But different power: tpss uses n^(4/3), revTPSS uses n^(1/3).
    const COEFF: f64 = -0.738_558_766_382_022_3_f64;
    let size = comptime!((1_u32 << n) as usize);
    let mut rho_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(1.0_f64 / 3.0_f64), &mut rho_13, n);
    ctaylor_scalar_mul::<F>(&rho_13, F::cast_from(COEFF), out, n);
}

/// revTPSS exchange `x` numerator.
/// Port of `revtpssx_eps.hpp:27-48`.
#[cube]
pub fn revtpss_x<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // p = gnn / (4*(3π²)^(2/3) * n^(8/3))
    const FOUR_3PI2_23_F64: f64 = 38.283_120_002_509_214_f64;
    let mut n_m83 = Array::<F>::new(size);
    ctaylor_pow::<F>(d_n, F::cast_from(-8.0_f64 / 3.0_f64), &mut n_m83, n);
    let mut p_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(d_gnn, &n_m83, &mut p_raw, n);
    let mut p = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p_raw, F::cast_from(1.0_f64 / FOUR_3PI2_23_F64), &mut p, n);

    // tauw = gnn / (8*n)
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(d_n, &mut inv_n, n);
    let mut tauw_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(d_gnn, &inv_n, &mut tauw_raw, n);
    let mut tauw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&tauw_raw, F::cast_from(1.0_f64 / 8.0_f64), &mut tauw, n);

    // z = tauw / tau
    let mut inv_tau = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(d_tau, &mut inv_tau, n);
    let mut z = Array::<F>::new(size);
    ctaylor_mul::<F>(&tauw, &inv_tau, &mut z, n);

    // z2 = z^2
    let mut z2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&z, &mut z2, n);

    // tau_unif = 0.3*(3π²)^(2/3)*n^(5/3)
    const COEFF_TAU_UNIF: f64 = 2.871_234_000_188_191_f64;
    let mut n_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(d_n, F::cast_from(5.0_f64 / 3.0_f64), &mut n_53, n);
    let mut tau_unif = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n_53, F::cast_from(COEFF_TAU_UNIF), &mut tau_unif, n);

    // alpha = (tau - tauw) / tau_unif
    let mut tau_m_tauw = Array::<F>::new(size);
    ctaylor_sub::<F>(d_tau, &tauw, &mut tau_m_tauw, n);
    let mut inv_tau_unif = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&tau_unif, &mut inv_tau_unif, n);
    let mut alpha = Array::<F>::new(size);
    ctaylor_mul::<F>(&tau_m_tauw, &inv_tau_unif, &mut alpha, n);

    // q_b = 9*(alpha-1) / (20*sqrt(1+b*alpha*(alpha-1))) + 2*p/3
    let mut alpha_m1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        alpha_m1[i] = alpha[i];
    }
    alpha_m1[0] = alpha_m1[0] - F::new(1.0);

    let mut b_alpha = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&alpha, F::cast_from(REVTPSS_B_F64), &mut b_alpha, n);
    let mut b_alpha_am1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b_alpha, &alpha_m1, &mut b_alpha_am1, n);

    let mut one_plus = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus[i] = b_alpha_am1[i];
    }
    one_plus[0] = one_plus[0] + F::new(1.0);

    let mut sq = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&one_plus, &mut sq, n);

    // 20*sqrt(...)
    let mut twenty_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sq, F::cast_from(20.0_f64), &mut twenty_sq, n);
    let mut inv_twenty_sq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&twenty_sq, &mut inv_twenty_sq, n);

    // 9*(alpha-1) / (20*sqrt(...))
    let mut frac1_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&alpha_m1, &inv_twenty_sq, &mut frac1_raw, n);
    let mut frac1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&frac1_raw, F::cast_from(9.0_f64), &mut frac1, n);

    let mut two_p_3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(2.0_f64 / 3.0_f64), &mut two_p_3, n);

    let mut q_b = Array::<F>::new(size);
    ctaylor_add::<F>(&frac1, &two_p_3, &mut q_b, n);

    // x_a = p * (10/81 + c * z2 * z / (1+z2)^2)   ← NOTE: z2*z = z^3 in revTPSS
    // (1+z2)
    let mut one_z2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_z2[i] = z2[i];
    }
    one_z2[0] = one_z2[0] + F::new(1.0);
    let mut one_z2_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&one_z2, &mut one_z2_sq, n);
    let mut inv_one_z2_sq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_z2_sq, &mut inv_one_z2_sq, n);

    // z2 * z = z^3
    let mut z3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&z2, &z, &mut z3, n);

    let mut cz3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z3, F::cast_from(REVTPSS_C_F64), &mut cz3, n);
    let mut cz3_frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&cz3, &inv_one_z2_sq, &mut cz3_frac, n);

    let mut bracket = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket[i] = cz3_frac[i];
    }
    bracket[0] = bracket[0] + F::cast_from(10.0_f64 / 81.0_f64);

    let mut x_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&p, &bracket, &mut x_a, n);

    // += 146/2025 * q_b^2
    let mut q_b_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&q_b, &mut q_b_sq, n);
    let mut q_b_sq_coeff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&q_b_sq, F::cast_from(146.0_f64 / 2025.0_f64), &mut q_b_sq_coeff, n);
    let mut x_a2 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a, &q_b_sq_coeff, &mut x_a2, n);

    // -= 73/405 * q_b * gnn * sqrt(0.5*0.36*(8*n*tau)^-2 + 0.5*p*p)
    // Note: revTPSS uses p*p NOT p0*p0 (different from TPSS!)
    let mut eight_n_tau_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(d_n, d_tau, &mut eight_n_tau_raw, n);
    let mut eight_n_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_raw, F::cast_from(8.0_f64), &mut eight_n_tau, n);
    let mut eight_n_tau_m2_raw = Array::<F>::new(size);
    ctaylor_pow::<F>(&eight_n_tau, F::cast_from(-2.0_f64), &mut eight_n_tau_m2_raw, n);
    let mut eight_n_tau_m2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_m2_raw, F::cast_from(0.5_f64 * 0.36_f64), &mut eight_n_tau_m2, n);

    let mut p_sq_raw = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&p, &mut p_sq_raw, n);
    let mut p_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p_sq_raw, F::cast_from(0.5_f64), &mut p_sq, n);

    let mut sqrt_arg = Array::<F>::new(size);
    ctaylor_add::<F>(&eight_n_tau_m2, &p_sq, &mut sqrt_arg, n);

    let mut sq2 = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&sqrt_arg, &mut sq2, n);

    let mut term3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&q_b, d_gnn, &mut term3, n);
    let mut term3b_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&term3, &sq2, &mut term3b_raw, n);
    let mut term3b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&term3b_raw, F::cast_from(73.0_f64 / 405.0_f64), &mut term3b, n);
    let mut x_a3 = Array::<F>::new(size);
    ctaylor_sub::<F>(&x_a2, &term3b, &mut x_a3, n);

    // += (10*p/81)^2 / kapa
    let mut p_10_81 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(10.0_f64 / 81.0_f64), &mut p_10_81, n);
    let mut p_10_81_sq_raw = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&p_10_81, &mut p_10_81_sq_raw, n);
    let mut p_10_81_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p_10_81_sq_raw, F::cast_from(1.0_f64 / REVTPSS_KAPPA_F64), &mut p_10_81_sq, n);
    let mut x_a4 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a3, &p_10_81_sq, &mut x_a4, n);

    // += sqrt(e)*0.36*z2*20/81 + e*mu*p^3
    // constants folded at host, passed as F scalar via F::cast_from
    let mut z2_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &z2,
        F::cast_from(REVTPSS_SQRT_E_F64 * 0.36_f64 * 20.0_f64 / 81.0_f64),
        &mut z2_term,
        n,
    );
    let mut x_a5 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a4, &z2_term, &mut x_a5, n);

    let mut p3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&p, &mut p3, n);
    let mut p3_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &p3,
        F::cast_from(REVTPSS_E_F64 * REVTPSS_MU_F64),
        &mut p3_term,
        n,
    );
    let mut x_a6 = Array::<F>::new(size);
    ctaylor_add::<F>(&x_a5, &p3_term, &mut x_a6, n);

    // divide by (1 + sqrt(e)*p)  ← NOTE: revTPSS uses ^1 not ^2
    let mut sp = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(REVTPSS_SQRT_E_F64), &mut sp, n);
    sp[0] = sp[0] + F::new(1.0);
    let mut inv_sp = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&sp, &mut inv_sp, n);
    ctaylor_mul::<F>(&x_a6, &inv_sp, out, n);
}

/// revTPSS `F_x = 1 + kapa - kapa/(1 + x/kapa)`. Port of `revtpssx_eps.hpp:52-56`.
#[cube]
pub fn revtpss_fx<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut xpz = Array::<F>::new(size);
    revtpss_x::<F>(d_n, d_gnn, d_tau, &mut xpz, n);

    let mut x_over_k = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xpz, F::cast_from(1.0_f64 / REVTPSS_KAPPA_F64), &mut x_over_k, n);
    x_over_k[0] = x_over_k[0] + F::new(1.0);

    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&x_over_k, &mut inv_denom, n);
    let mut k_over_denom = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_denom, F::cast_from(REVTPSS_KAPPA_F64), &mut k_over_denom, n);

    let one_plus_k = 1.0_f64 + REVTPSS_KAPPA_F64;
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&k_over_denom, neg_one, out, n);
    out[0] = out[0] + F::cast_from(one_plus_k);
}

// ---------------------------------------------------------------------------
//  TPSS correlation helpers (tpssc_eps.hpp)
// ---------------------------------------------------------------------------

/// TPSS `C(d)` factor. Port of `tpssc_eps.hpp:22-30`.
/// C0 = 0.53 + 0.87*ζ² + 0.50*ζ⁴ + 2.26*ζ⁶
/// C = C0 * (1 + 0.5*xi2*ufunc(ζ,-4/3))^(-4)
#[cube]
fn tpss_C<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // gzeta2 = (n²*gss - 2*n*s*gns + s²*gnn) / n^4
    let mut n2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.n, &mut n2, n);
    let mut t1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&n2, &d.gss, &mut t1, n);

    let mut ns = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.s, &mut ns, n);
    let mut t2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ns, &d.gns, &mut t2_raw, n);
    let neg_two = F::new(0.0) - F::new(2.0);
    let mut t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&t2_raw, neg_two, &mut t2, n);

    let mut s2_arr = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.s, &mut s2_arr, n);
    let mut t3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&s2_arr, &d.gnn, &mut t3, n);

    let mut gzeta2_num1 = Array::<F>::new(size);
    ctaylor_add::<F>(&t1, &t2, &mut gzeta2_num1, n);
    let mut gzeta2_num = Array::<F>::new(size);
    ctaylor_add::<F>(&gzeta2_num1, &t3, &mut gzeta2_num, n);

    let mut n4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&n2, &mut n4, n);
    let mut inv_n4 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&n4, &mut inv_n4, n);
    let mut gzeta2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gzeta2_num, &inv_n4, &mut gzeta2, n);

    // xi2 = gzeta2 / (4 * (3π²)^(2/3) * n^(2/3))  ← wait, actually:
    // xi2 = gzeta2 / (4 * pow(3*pi^2*n, 2/3))  per C++ code
    // = gzeta2 / (4 * (3π²)^(2/3) * n^(2/3))
    // = gzeta2 * (1/(4*(3π²)^(2/3))) * n^(-2/3)
    const FOUR_3PI2_23: f64 = 38.283_120_002_509_214_f64;
    let mut n_m23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-2.0_f64 / 3.0_f64), &mut n_m23, n);
    let mut xi2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&gzeta2, &n_m23, &mut xi2_raw, n);
    let mut xi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xi2_raw, F::cast_from(1.0_f64 / FOUR_3PI2_23), &mut xi2, n);

    // C0 = 0.53 + 0.87*zeta^2 + 0.50*zeta^4 + 2.26*zeta^6
    let mut z2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.zeta, &mut z2, n);
    let mut z4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&z2, &mut z4, n);
    let mut z6 = Array::<F>::new(size);
    ctaylor_mul::<F>(&z4, &z2, &mut z6, n);

    let mut t87z2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z2, F::cast_from(0.87_f64), &mut t87z2, n);
    let mut t50z4 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z4, F::cast_from(0.50_f64), &mut t50z4, n);
    let mut t226z6 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z6, F::cast_from(2.26_f64), &mut t226z6, n);

    let mut C0_raw = Array::<F>::new(size);
    ctaylor_add::<F>(&t87z2, &t50z4, &mut C0_raw, n);
    let mut C0 = Array::<F>::new(size);
    ctaylor_add::<F>(&C0_raw, &t226z6, &mut C0, n);
    C0[0] = C0[0] + F::cast_from(0.53_f64);

    // ufunc(zeta, -4/3) = (1+zeta)^(-4/3) + (1-zeta)^(-4/3)
    let mut uf = Array::<F>::new(size);
    ufunc_p::<F>(&d.zeta, F::cast_from(-4.0_f64 / 3.0_f64), &mut uf, n);

    // 0.5 * xi2 * ufunc
    let mut xi2_uf_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&xi2, &uf, &mut xi2_uf_raw, n);
    let mut xi2_uf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xi2_uf_raw, F::cast_from(0.5_f64), &mut xi2_uf, n);

    // (1 + 0.5*xi2*ufunc)
    xi2_uf[0] = xi2_uf[0] + F::new(1.0);

    // (...)^(-4) = 1/(...)^4
    let mut pow4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&xi2_uf, &mut pow4, n);
    let mut pow4b = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&pow4, &mut pow4b, n);
    let mut inv_pow4 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&pow4b, &mut inv_pow4, n);

    // out = C0 * inv_pow4
    ctaylor_mul::<F>(&C0, &inv_pow4, out, n);
}

/// TPSS `pbec_eps(d)` — PBE correlation energy per particle.
/// Internally computes PBEC eps (not the full n*eps).
/// Port of `pbec_eps.hpp::pbec_eps(d)` = H(d2, pw92eps, u^3) + pw92eps.
#[cube]
fn tpss_pbec_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = pw92_eps(d)
    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    // u = phi_reorganised
    let mut u = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut u, n);

    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);

    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut denom, n);

    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_denom, &mut g_over_d, n);

    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    let mut h = Array::<F>::new(size);
    pbec_eps::h_gga::<F>(&d2, &eps, &u3, &mut h, n);

    ctaylor_add::<F>(&eps, &h, out, n);
}

/// TPSS `pbec_eps_polarized(a, gaa)` — spin-polarised PBE eps.
/// Port of `pbec_eps.hpp::pbec_eps_polarized(a, gaa)`.
#[cube]
fn tpss_pbec_eps_polarized<F: Float>(
    a: &Array<F>,
    gaa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = pw92eps_polarized(a)
    let mut eps = Array::<F>::new(size);
    pw92eps::pw92eps_polarized::<F>(a, &mut eps, n);

    // u = 2^(-1/3) (scalar, fully polarized phi)
    // phi for single-spin = pow(2.0, -1.0/3.0) = 0.793700525984...
    const PHI_POLAR: f64 = 0.793_700_525_984_099_8_f64;
    let mut u = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        u[i] = F::new(0.0);
    }
    u[0] = F::cast_from(PHI_POLAR);

    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    // d2 = PREFACTOR * gaa / (u² * a^(7/3))
    let mut a_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(a, F::cast_from(7.0_f64 / 3.0_f64), &mut a_73, n);

    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &a_73, &mut denom, n);

    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_denom, &mut g_over_d, n);

    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    let mut h = Array::<F>::new(size);
    pbec_eps::h_gga::<F>(&d2, &eps, &u3, &mut h, n);

    ctaylor_add::<F>(&eps, &h, out, n);
}

/// TPSS `max(a, b)` for CTaylor — takes max of CNST slot; for `tauwtau2 >= 0`
/// the epsc values are negative, so max means "less negative" i.e. the one
/// with greater value. Since both are scalar CTaylor, we branch on CNST slot.
/// Per T-04-01-01 threat: abs() on CNST only is safe here.
#[cube]
pub fn ctaylor_max<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // Select whichever has larger CNST slot.
    if a[0] >= b[0] {
        #[unroll]
        for i in 0..size {
            out[i] = a[i];
        }
    } else {
        #[unroll]
        for i in 0..size {
            out[i] = b[i];
        }
    }
}

/// TPSS `epsc_summax(d)` — `(a * max(eps_pbe, eps_pbe_a) + b * max(eps_pbe, eps_pbe_b)) / n`.
/// Port of `tpssc_eps.hpp:33-42`.
#[cube]
fn tpss_epsc_summax<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut epsc_pbe = Array::<F>::new(size);
    tpss_pbec_eps::<F>(d, &mut epsc_pbe, n);

    let mut epsc_pbe_a = Array::<F>::new(size);
    tpss_pbec_eps_polarized::<F>(&d.a, &d.gaa, &mut epsc_pbe_a, n);

    let mut epsc_pbe_b = Array::<F>::new(size);
    tpss_pbec_eps_polarized::<F>(&d.b, &d.gbb, &mut epsc_pbe_b, n);

    // max(epsc_pbe, epsc_pbe_a)
    let mut max_a = Array::<F>::new(size);
    ctaylor_max::<F>(&epsc_pbe, &epsc_pbe_a, &mut max_a, n);

    // max(epsc_pbe, epsc_pbe_b)
    let mut max_b = Array::<F>::new(size);
    ctaylor_max::<F>(&epsc_pbe, &epsc_pbe_b, &mut max_b, n);

    // a * max_a
    let mut a_max_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &max_a, &mut a_max_a, n);

    // b * max_b
    let mut b_max_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &max_b, &mut b_max_b, n);

    // sum = a*max_a + b*max_b
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&a_max_a, &b_max_b, &mut sum, n);

    // / n
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, n);
    ctaylor_mul::<F>(&sum, &inv_n, out, n);
}

/// TPSS correlation `tpssc_eps(d)`. Port of `tpssc_eps.hpp:56-61`.
#[cube]
pub fn tpss_eps<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // NOTE: This function signature kept for Wave-0 compat but is NOT used by tpssc_kernel.
    // tpssc_kernel uses tpss_eps_full which takes the full DensVarsDev.
    // These parameters are unused here — kept only for API compat.
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// TPSS correlation `tpssc_eps(d)` — full DensVarsDev variant.
/// Port of `tpssc_eps.hpp:56-61`.
#[cube]
pub fn tpss_eps_full<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // tauwtau2 = (gnn / (8*n*tau))^2
    let mut eight_n_tau_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.tau, &mut eight_n_tau_raw, n);
    let mut eight_n_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_raw, F::cast_from(8.0_f64), &mut eight_n_tau, n);
    let mut gnn_8nt = Array::<F>::new(size);
    let mut inv_8nt = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&eight_n_tau, &mut inv_8nt, n);
    ctaylor_mul::<F>(&d.gnn, &inv_8nt, &mut gnn_8nt, n);
    let mut tauwtau2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&gnn_8nt, &mut tauwtau2, n);

    // epsc_pbe = tpss_pbec_eps(d)
    let mut epsc_pbe = Array::<F>::new(size);
    tpss_pbec_eps::<F>(d, &mut epsc_pbe, n);

    // epsc_sum = tpss_epsc_summax(d)
    let mut epsc_sum = Array::<F>::new(size);
    tpss_epsc_summax::<F>(d, &mut epsc_sum, n);

    // C_zeta_xi = tpss_C(d)
    let mut C_zeta_xi = Array::<F>::new(size);
    tpss_C::<F>(d, &mut C_zeta_xi, n);

    // eps_pkzb = epsc_pbe * (1 + C*tauwtau2) - (1+C)*tauwtau2*epsc_sum
    // = epsc_pbe + epsc_pbe*C*tauwtau2 - (1+C)*tauwtau2*epsc_sum

    let mut C_tauwtau2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&C_zeta_xi, &tauwtau2, &mut C_tauwtau2, n);

    let mut pbe_C_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&epsc_pbe, &C_tauwtau2, &mut pbe_C_t2, n);

    let mut eps_pkzb = Array::<F>::new(size);
    ctaylor_add::<F>(&epsc_pbe, &pbe_C_t2, &mut eps_pkzb, n);

    // (1 + C) * tauwtau2 * epsc_sum
    let mut one_C = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_C[i] = C_zeta_xi[i];
    }
    one_C[0] = one_C[0] + F::new(1.0);

    let mut one_C_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_C, &tauwtau2, &mut one_C_t2, n);

    let mut rhs = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_C_t2, &epsc_sum, &mut rhs, n);

    let mut eps_pkzb2 = Array::<F>::new(size);
    ctaylor_sub::<F>(&eps_pkzb, &rhs, &mut eps_pkzb2, n);

    // tauwtau3 = (gnn / (8*n*tau))^3
    let mut tauwtau3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&gnn_8nt, &mut tauwtau3, n);

    // out = eps_pkzb * (1 + dd * eps_pkzb * tauwtau3)
    let mut dd_eps_t3_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps_pkzb2, &tauwtau3, &mut dd_eps_t3_raw, n);
    let mut dd_eps_t3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&dd_eps_t3_raw, F::cast_from(TPSS_DD_F64), &mut dd_eps_t3, n);
    dd_eps_t3[0] = dd_eps_t3[0] + F::new(1.0);
    ctaylor_mul::<F>(&eps_pkzb2, &dd_eps_t3, out, n);
}

// ---------------------------------------------------------------------------
//  revTPSS correlation helpers (revtpssc_eps.hpp)
// ---------------------------------------------------------------------------

/// revTPSS `beta(dens)` — density-dependent beta.
/// Port of `revtpssc_eps.hpp:43-47`.
/// beta = beta_pbe * (1 + 0.1*r_s) / (1 + 0.1778*r_s)
/// r_s = cbrt(3 / (4π * dens))
#[cube]
fn revtpss_beta<F: Float>(dens: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    // r_s = (3/(4π*dens))^(1/3) = (3/(4π))^(1/3) * dens^(-1/3)
    // = 0.6203504908994001 * dens^(-1/3)  (same as RS_PREFACTOR)
    const RS_PREF: f64 = 0.620_350_490_899_400_1_f64;
    let mut r_s_raw = Array::<F>::new(size);
    ctaylor_pow::<F>(dens, F::cast_from(-1.0_f64 / 3.0_f64), &mut r_s_raw, n);
    let mut r_s = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&r_s_raw, F::cast_from(RS_PREF), &mut r_s, n);

    // beta_pbe_paper = 0.066725
    const BETA_PBE: f64 = 0.066_725_f64;

    // (1 + 0.1*r_s)
    let mut numer = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&r_s, F::cast_from(0.1_f64), &mut numer, n);
    numer[0] = numer[0] + F::new(1.0);

    // (1 + 0.1778*r_s)
    let mut denom = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&r_s, F::cast_from(0.1778_f64), &mut denom, n);
    denom[0] = denom[0] + F::new(1.0);

    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&numer, &inv_denom, &mut frac, n);

    ctaylor_scalar_mul::<F>(&frac, F::cast_from(BETA_PBE), out, n);
}

/// revTPSS `A(eps, u3, beta_tpss)`. Port of `revtpssc_eps.hpp:25-29`.
#[cube]
fn revtpss_A<F: Float>(
    eps: &Array<F>,
    u3: &Array<F>,
    beta_tpss: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    use crate::functionals::gga::shared::constants::PBEC_GAMMA_F64;
    let size = comptime!((1_u32 << n) as usize);

    // beta_gamma = beta_tpss / param_gamma
    let mut beta_gamma = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(beta_tpss, F::cast_from(1.0_f64 / PBEC_GAMMA_F64), &mut beta_gamma, n);

    // expm1(-eps / (gamma * u3))
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(u3, F::cast_from(PBEC_GAMMA_F64), &mut gu3, n);
    let mut inv_gu3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gu3, &mut inv_gu3, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(eps, &inv_gu3, &mut prod, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&prod, neg_one, &mut arg, n);
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);

    // out = beta_gamma / expm1(...) = beta_gamma * inv_em1
    ctaylor_mul::<F>(&beta_gamma, &inv_em1, out, n);
}

/// revTPSS `H(d2, eps, u3, beta_tpss)`. Port of `revtpssc_eps.hpp:31-41`.
#[cube]
fn revtpss_H<F: Float>(
    d2: &Array<F>,
    eps: &Array<F>,
    u3: &Array<F>,
    beta_tpss: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    use crate::functionals::gga::shared::constants::PBEC_GAMMA_F64;
    let size = comptime!((1_u32 << n) as usize);

    // beta_gamma = beta_tpss / gamma
    let mut beta_gamma = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(beta_tpss, F::cast_from(1.0_f64 / PBEC_GAMMA_F64), &mut beta_gamma, n);

    // A = revtpss_A
    let mut A = Array::<F>::new(size);
    revtpss_A::<F>(eps, u3, beta_tpss, &mut A, n);

    // d2A = d2 * A
    let mut d2A = Array::<F>::new(size);
    ctaylor_mul::<F>(d2, &A, &mut d2A, n);

    // 1 + d2A
    let mut one_d2A = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2A[i] = d2A[i];
    }
    one_d2A[0] = one_d2A[0] + F::new(1.0);

    // d2A * (1 + d2A)
    let mut d2A_one_d2A = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2A, &one_d2A, &mut d2A_one_d2A, n);

    // 1 + d2A*(1+d2A)
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = d2A_one_d2A[i];
    }
    den[0] = den[0] + F::new(1.0);

    // beta_gamma * d2 * (1 + d2A)
    let mut bg_d2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&beta_gamma, d2, &mut bg_d2, n);
    let mut num = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg_d2, &one_d2A, &mut num, n);

    // num / den
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num, &inv_den, &mut frac, n);

    // log(1 + frac)
    frac[0] = frac[0] + F::new(1.0);
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&frac, &mut lg, n);

    // gamma * u3 * log(...)
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(u3, F::cast_from(PBEC_GAMMA_F64), &mut gu3, n);
    ctaylor_mul::<F>(&gu3, &lg, out, n);
}

/// revTPSS `pbec_eps(d)` — modified PBE correlation eps with density-dependent beta.
/// Port of `revtpssc_eps.hpp:49-57`.
#[cube]
fn revtpss_pbec_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut beta_tpss = Array::<F>::new(size);
    revtpss_beta::<F>(&d.n, &mut beta_tpss, n);

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    let mut u = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut u, n);

    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    // d2 = PREFACTOR * gnn / (u² * n^(7/3))
    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut denom, n);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_denom, &mut g_over_d, n);
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    let mut h = Array::<F>::new(size);
    revtpss_H::<F>(&d2, &eps, &u3, &mut beta_tpss, &mut h, n);

    ctaylor_add::<F>(&eps, &h, out, n);
}

/// revTPSS `pbec_eps_polarized(a, gaa)`. Port of `revtpssc_eps.hpp:59-68`.
#[cube]
fn revtpss_pbec_eps_polarized<F: Float>(
    a: &Array<F>,
    gaa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92eps_polarized::<F>(a, &mut eps, n);

    const PHI_POLAR: f64 = 0.793_700_525_984_099_8_f64;
    let mut u = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        u[i] = F::new(0.0);
    }
    u[0] = F::cast_from(PHI_POLAR);

    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    let mut beta_tpss = Array::<F>::new(size);
    revtpss_beta::<F>(a, &mut beta_tpss, n);

    let mut a_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(a, F::cast_from(7.0_f64 / 3.0_f64), &mut a_73, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &a_73, &mut denom, n);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_denom, &mut g_over_d, n);
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    let mut h = Array::<F>::new(size);
    revtpss_H::<F>(&d2, &eps, &u3, &mut beta_tpss, &mut h, n);

    ctaylor_add::<F>(&eps, &h, out, n);
}

/// revTPSS `C(d)`. Port of `revtpssc_eps.hpp:70-79`.
/// C0 = 0.59 + 0.9269*ζ² + 0.6225*ζ⁴ + 2.1540*ζ⁶
#[cube]
fn revtpss_C<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // gzeta2 = (n²*gss - 2*n*s*gns + s²*gnn) / n^4
    let mut n2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.n, &mut n2, n);
    let mut t1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&n2, &d.gss, &mut t1, n);
    let mut ns = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.s, &mut ns, n);
    let mut t2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ns, &d.gns, &mut t2_raw, n);
    let neg_two = F::new(0.0) - F::new(2.0);
    let mut t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&t2_raw, neg_two, &mut t2, n);
    let mut s2_arr = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.s, &mut s2_arr, n);
    let mut t3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&s2_arr, &d.gnn, &mut t3, n);
    let mut gzeta2_num1 = Array::<F>::new(size);
    ctaylor_add::<F>(&t1, &t2, &mut gzeta2_num1, n);
    let mut gzeta2_num = Array::<F>::new(size);
    ctaylor_add::<F>(&gzeta2_num1, &t3, &mut gzeta2_num, n);
    let mut n4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&n2, &mut n4, n);
    let mut inv_n4 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&n4, &mut inv_n4, n);
    let mut gzeta2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gzeta2_num, &inv_n4, &mut gzeta2, n);

    // xi2
    const FOUR_3PI2_23: f64 = 38.283_120_002_509_214_f64;
    let mut n_m23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-2.0_f64 / 3.0_f64), &mut n_m23, n);
    let mut xi2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&gzeta2, &n_m23, &mut xi2_raw, n);
    let mut xi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xi2_raw, F::cast_from(1.0_f64 / FOUR_3PI2_23), &mut xi2, n);

    // C0 with revTPSS coefficients
    let mut z2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.zeta, &mut z2, n);
    let mut z4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&z2, &mut z4, n);
    let mut z6 = Array::<F>::new(size);
    ctaylor_mul::<F>(&z4, &z2, &mut z6, n);
    let mut t1c = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z2, F::cast_from(0.9269_f64), &mut t1c, n);
    let mut t2c = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z4, F::cast_from(0.6225_f64), &mut t2c, n);
    let mut t3c = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z6, F::cast_from(2.1540_f64), &mut t3c, n);
    let mut C0_raw = Array::<F>::new(size);
    ctaylor_add::<F>(&t1c, &t2c, &mut C0_raw, n);
    let mut C0 = Array::<F>::new(size);
    ctaylor_add::<F>(&C0_raw, &t3c, &mut C0, n);
    C0[0] = C0[0] + F::cast_from(0.59_f64);

    // revTPSS uses (1 + zeta)^(-4/3) + (1 - zeta)^(-4/3) directly
    let mut pz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        pz[i] = d.zeta[i];
    }
    pz[0] = pz[0] + F::new(1.0);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut mz = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.zeta, neg_one, &mut mz, n);
    mz[0] = mz[0] + F::new(1.0);
    let mut pz_pow = Array::<F>::new(size);
    ctaylor_pow::<F>(&pz, F::cast_from(-4.0_f64 / 3.0_f64), &mut pz_pow, n);
    let mut mz_pow = Array::<F>::new(size);
    ctaylor_pow::<F>(&mz, F::cast_from(-4.0_f64 / 3.0_f64), &mut mz_pow, n);
    let mut uf = Array::<F>::new(size);
    ctaylor_add::<F>(&pz_pow, &mz_pow, &mut uf, n);

    // 0.5 * xi2 * uf
    let mut xi2_uf_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&xi2, &uf, &mut xi2_uf_raw, n);
    let mut xi2_uf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xi2_uf_raw, F::cast_from(0.5_f64), &mut xi2_uf, n);
    xi2_uf[0] = xi2_uf[0] + F::new(1.0);

    // (...)^(-4)
    let mut pow4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&xi2_uf, &mut pow4, n);
    let mut pow4b = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&pow4, &mut pow4b, n);
    let mut inv_pow4 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&pow4b, &mut inv_pow4, n);

    ctaylor_mul::<F>(&C0, &inv_pow4, out, n);
}

/// revTPSS `epsc_summax(d)`. Port of `revtpssc_eps.hpp:81-91`.
#[cube]
fn revtpss_epsc_summax<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut epsc_pbe = Array::<F>::new(size);
    revtpss_pbec_eps::<F>(d, &mut epsc_pbe, n);

    let mut epsc_pbe_a = Array::<F>::new(size);
    revtpss_pbec_eps_polarized::<F>(&d.a, &d.gaa, &mut epsc_pbe_a, n);

    let mut epsc_pbe_b = Array::<F>::new(size);
    revtpss_pbec_eps_polarized::<F>(&d.b, &d.gbb, &mut epsc_pbe_b, n);

    let mut max_a = Array::<F>::new(size);
    ctaylor_max::<F>(&epsc_pbe, &epsc_pbe_a, &mut max_a, n);

    let mut max_b = Array::<F>::new(size);
    ctaylor_max::<F>(&epsc_pbe, &epsc_pbe_b, &mut max_b, n);

    let mut a_max_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &max_a, &mut a_max_a, n);

    let mut b_max_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &max_b, &mut b_max_b, n);

    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&a_max_a, &b_max_b, &mut sum, n);

    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, n);
    ctaylor_mul::<F>(&sum, &inv_n, out, n);
}

/// revTPSS correlation `revtpssc_eps(d)`. Port of `revtpssc_eps.hpp:105-110`.
#[cube]
pub fn revtpss_eps<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // Unused — kept for Wave-0 API compat. Full version uses revtpss_eps_full.
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// revTPSS correlation `revtpssc_eps(d)` — full DensVarsDev version.
#[cube]
pub fn revtpss_eps_full<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // tauwtau2 = (gnn / (8*n*tau))^2
    let mut eight_n_tau_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.tau, &mut eight_n_tau_raw, n);
    let mut eight_n_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_raw, F::cast_from(8.0_f64), &mut eight_n_tau, n);
    let mut inv_8nt = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&eight_n_tau, &mut inv_8nt, n);
    let mut gnn_8nt = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_8nt, &mut gnn_8nt, n);
    let mut tauwtau2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&gnn_8nt, &mut tauwtau2, n);

    let mut epsc_pbe = Array::<F>::new(size);
    revtpss_pbec_eps::<F>(d, &mut epsc_pbe, n);

    let mut epsc_sum = Array::<F>::new(size);
    revtpss_epsc_summax::<F>(d, &mut epsc_sum, n);

    let mut C_zeta_xi = Array::<F>::new(size);
    revtpss_C::<F>(d, &mut C_zeta_xi, n);

    // eps_pkzb = epsc_pbe*(1 + C*t2) - (1+C)*t2*epsc_sum
    let mut C_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&C_zeta_xi, &tauwtau2, &mut C_t2, n);
    let mut pbe_C_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&epsc_pbe, &C_t2, &mut pbe_C_t2, n);
    let mut eps_pkzb = Array::<F>::new(size);
    ctaylor_add::<F>(&epsc_pbe, &pbe_C_t2, &mut eps_pkzb, n);

    let mut one_C = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_C[i] = C_zeta_xi[i];
    }
    one_C[0] = one_C[0] + F::new(1.0);
    let mut one_C_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_C, &tauwtau2, &mut one_C_t2, n);
    let mut rhs = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_C_t2, &epsc_sum, &mut rhs, n);
    let mut eps_pkzb2 = Array::<F>::new(size);
    ctaylor_sub::<F>(&eps_pkzb, &rhs, &mut eps_pkzb2, n);

    // out = eps_pkzb * (1 + 2.8 * eps_pkzb * tauwtau2)
    let mut dd_eps_t2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps_pkzb2, &tauwtau2, &mut dd_eps_t2_raw, n);
    let mut dd_eps_t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&dd_eps_t2_raw, F::cast_from(2.8_f64), &mut dd_eps_t2, n);
    dd_eps_t2[0] = dd_eps_t2[0] + F::new(1.0);
    ctaylor_mul::<F>(&eps_pkzb2, &dd_eps_t2, out, n);
}
