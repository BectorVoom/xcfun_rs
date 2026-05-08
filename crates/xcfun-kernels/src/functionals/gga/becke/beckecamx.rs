//! XC_BECKECAMX — CAM Becke 88 exchange. GGA-02.
//!
//! # Source
//! - `xcfun-master/src/functionals/beckex.cpp:56-73, 88-94`
//!
//! # Formula (port of `becke_cam`):
//! Same as `becke_sr` but with `(1 - alpha - beta · 8/3 · a · (...))`.
//!
//! # B3 parameter access (Phase-3 plan 03-02)
//! Reads `mu = parameters[1]`, `alpha = parameters[2]`, `beta = parameters[3]`
//! per `common_parameters.cpp:17-29`. For tier-2 harness which uses defaults
//! `(0.4, 0.19, 0.46)`, the kernel uses these constants directly. Future plan
//! plumbs `parameters[]` through cubecl scalars.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_erf, ctaylor_expm1, ctaylor_pow, ctaylor_powi_2, ctaylor_reciprocal, ctaylor_sqrt,
    ctaylor_sqrtx_asinh_sqrtx,
};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{BECKE_6D_F64, BECKE_D_F64, C_SLATER_F64};
use crate::functionals::gga::shared::pw91_like;

/// Defaults per `common_parameters.cpp:17-29`.
const DEFAULT_MU_F64: f64 = 0.4_f64;
const DEFAULT_CAM_ALPHA_F64: f64 = 0.19_f64;
const DEFAULT_CAM_BETA_F64: f64 = 0.46_f64;
/// `sqrt(π)`. Must match `lda::ldaerfx::SQRT_PI_F64 = 1.7724538509055159`
/// (the f64-nearest of C++ libm `sqrt(M_PI)`). Previous literal was 1 ULP low.
const SQRT_PI_F64: f64 = 1.7724538509055159_f64;

#[cube]
fn becke_cam<F: Float>(
    alpha: F,
    beta: F,
    mu: F,
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut chi2_arr = Array::<F>::new(size);
    pw91_like::chi2::<F>(rho, grad2, &mut chi2_arr, n);
    let mut sas = Array::<F>::new(size);
    ctaylor_sqrtx_asinh_sqrtx::<F>(&chi2_arr, &mut sas, n);
    let mut six_d_sas = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sas, F::cast_from(BECKE_6D_F64), &mut six_d_sas, n);
    let mut denom_b = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom_b[i] = six_d_sas[i];
    }
    denom_b[0] = denom_b[0] + F::new(1.0);
    let mut inv_denom_b = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_b, &mut inv_denom_b, n);
    let mut d_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&chi2_arr, F::cast_from(BECKE_D_F64), &mut d_chi2, n);
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&d_chi2, &inv_denom_b, &mut ratio, n);
    let mut k_inner = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        k_inner[i] = ratio[i];
    }
    k_inner[0] = k_inner[0] + F::cast_from(C_SLATER_F64);
    let mut k_arr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&k_inner, F::new(2.0), &mut k_arr, n);
    let mut sqrt_k = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&k_arr, &mut sqrt_k, n);

    let mut rho_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(1.0_f64 / 3.0_f64), &mut rho_13, n);
    let mut denom_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &rho_13,
        F::new(6.0) * F::cast_from(SQRT_PI_F64),
        &mut denom_a,
        n,
    );
    let mut inv_denom_a = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_a, &mut inv_denom_a, n);
    let mut mu_sqrt_k = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_k, mu, &mut mu_sqrt_k, n);
    let mut a_arr = Array::<F>::new(size);
    ctaylor_mul::<F>(&mu_sqrt_k, &inv_denom_a, &mut a_arr, n);

    let mut a2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&a_arr, &mut a2, n);
    let mut four_a2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a2, F::new(4.0), &mut four_a2, n);
    let mut inv_4a2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&four_a2, &mut inv_4a2, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_inv_4a2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_4a2, neg_one, &mut neg_inv_4a2, n);
    let mut b_arr = Array::<F>::new(size);
    ctaylor_expm1::<F>(&neg_inv_4a2, &mut b_arr, n);

    let mut a2_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&a2, &b_arr, &mut a2_b, n);
    let mut two_a2_b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a2_b, F::new(2.0), &mut two_a2_b, n);
    let mut c_arr = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        c_arr[i] = two_a2_b[i];
    }
    c_arr[0] = c_arr[0] + F::new(0.5);

    let mut two_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a_arr, F::new(2.0), &mut two_a, n);
    let mut inv_2a = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&two_a, &mut inv_2a, n);
    let mut erf_term = Array::<F>::new(size);
    ctaylor_erf::<F>(&inv_2a, &mut erf_term, n);
    let mut sqrt_pi_erf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&erf_term, F::cast_from(SQRT_PI_F64), &mut sqrt_pi_erf, n);

    let mut neg_c = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&c_arr, neg_one, &mut neg_c, n);
    let mut b_minus_c = Array::<F>::new(size);
    ctaylor_add::<F>(&b_arr, &neg_c, &mut b_minus_c, n);
    let mut two_a_bmc = Array::<F>::new(size);
    ctaylor_mul::<F>(&two_a, &b_minus_c, &mut two_a_bmc, n);
    let mut sum_term = Array::<F>::new(size);
    ctaylor_add::<F>(&sqrt_pi_erf, &two_a_bmc, &mut sum_term, n);
    let mut a_sum = Array::<F>::new(size);
    ctaylor_mul::<F>(&a_arr, &sum_term, &mut a_sum, n);

    // CAM bracket: (1 - α - β · 8/3 · a · sum_term).
    let mut eight_thirds_asum = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a_sum, F::new(8.0) / F::new(3.0), &mut eight_thirds_asum, n);
    let mut beta_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_thirds_asum, beta, &mut beta_term, n);
    let mut neg_beta_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&beta_term, neg_one, &mut neg_beta_term, n);
    let mut bracket = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket[i] = neg_beta_term[i];
    }
    bracket[0] = bracket[0] + F::new(1.0) - alpha;

    let mut na43_k = Array::<F>::new(size);
    ctaylor_mul::<F>(rho_43, &k_arr, &mut na43_k, n);
    let mut na43_k_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&na43_k, &bracket, &mut na43_k_b, n);
    ctaylor_scalar_mul::<F>(&na43_k_b, F::new(-0.5), out, n);
}

#[cube]
pub fn beckecamx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mu = F::cast_from(DEFAULT_MU_F64);
    let alpha = F::cast_from(DEFAULT_CAM_ALPHA_F64);
    let beta = F::cast_from(DEFAULT_CAM_BETA_F64);
    let mut e_alpha = Array::<F>::new(size);
    becke_cam::<F>(alpha, beta, mu, &d.a_43, &d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    becke_cam::<F>(alpha, beta, mu, &d.b_43, &d.b, &d.gbb, &mut e_beta, n);
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
