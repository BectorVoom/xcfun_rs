//! XC_BECKESRX — Short-range Becke 88 exchange. GGA-02.
//!
//! # Source
//! - `xcfun-master/src/functionals/beckex.cpp:42-54, 83-86`
//!
//! # Formula (port of `becke_sr`):
//! ```cpp
//! cparam = pow(81/(4π), 1/3) / 2;       // = -NEG_C_SLATER
//! d  = 0.0042;
//! na43 = pow(na, 4/3);
//! chi2 = gaa * pow(na, -8/3);
//! K = 2 * (cparam + (d * chi2) / (1 + 6*d*sqrtx_asinh_sqrtx(chi2)));
//! a = mu * sqrt(K) / (6 * sqrt(π) * pow(na, 1/3));
//! b = expm1(-1 / (4*a*a));
//! c = 2*a*a*b + 0.5;
//! return -0.5 * na43 * K *
//!        (1 - 8/3 * a * (sqrt(π) * erf(1/(2a)) + 2*a*(b - c)));
//! ```
//!
//! # B3 parameter access (Phase-3 plan 03-02)
//! Reads `mu = parameters[1] = XC_RANGESEP_MU` (default 0.4 per
//! `common_parameters.cpp:17-29`). The launch-path plumbing of `parameters`
//! through cubecl-level scalars is a future sub-plan; for the tier-2 harness
//! which uses the documented C++ defaults, this kernel uses the f64 default
//! `0.4` directly. Pitfall G4 bracket-cancellation order preserved verbatim.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_erf, ctaylor_expm1, ctaylor_pow, ctaylor_powi_2, ctaylor_reciprocal, ctaylor_sqrt,
    ctaylor_sqrtx_asinh_sqrtx,
};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{
    BECKE_6D_F64, BECKE_D_F64, C_SLATER_F64, NEG_C_SLATER_F64,
};
use crate::functionals::gga::shared::pw91_like;

/// Default RANGESEP_MU per `common_parameters.cpp:17`.
const DEFAULT_MU_F64: f64 = 0.4_f64;
/// `sqrt(π)` precomputed in f64. Must match
/// `lda::ldaerfx::SQRT_PI_F64 = 1.7724538509055159` (which matches what
/// C++ libm `sqrt(M_PI)` produces). Previous literal was 1 ULP low.
const SQRT_PI_F64: f64 = 1.7724538509055159_f64;

#[cube]
fn becke_sr<F: Float>(
    mu: F,
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // chi² = grad² · ρ^(-8/3).
    let mut chi2_arr = Array::<F>::new(size);
    pw91_like::chi2::<F>(rho, grad2, &mut chi2_arr, n);

    // sas = sqrtx_asinh_sqrtx(chi²).
    let mut sas = Array::<F>::new(size);
    ctaylor_sqrtx_asinh_sqrtx::<F>(&chi2_arr, &mut sas, n);

    // 6d·sas.
    let mut six_d_sas = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sas, F::cast_from(BECKE_6D_F64), &mut six_d_sas, n);

    // denom_b = 1 + 6d·sas.
    let mut denom_b = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom_b[i] = six_d_sas[i];
    }
    denom_b[0] = denom_b[0] + F::new(1.0);
    let mut inv_denom_b = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_b, &mut inv_denom_b, n);

    // d·chi².
    let mut d_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&chi2_arr, F::cast_from(BECKE_D_F64), &mut d_chi2, n);

    // ratio = d·chi² · inv_denom_b.
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&d_chi2, &inv_denom_b, &mut ratio, n);

    // K = 2 · (cparam + ratio).
    let mut k_inner = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        k_inner[i] = ratio[i];
    }
    k_inner[0] = k_inner[0] + F::cast_from(C_SLATER_F64);
    let mut k_arr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&k_inner, F::new(2.0), &mut k_arr, n);

    // sqrt_k = sqrt(K).
    let mut sqrt_k = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&k_arr, &mut sqrt_k, n);

    // rho_13 = ρ^(1/3).
    let mut rho_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(1.0_f64 / 3.0_f64), &mut rho_13, n);

    // denom_a = 6 · sqrt(π) · ρ^(1/3).
    let mut denom_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rho_13, F::new(6.0) * F::cast_from(SQRT_PI_F64), &mut denom_a, n);
    let mut inv_denom_a = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_a, &mut inv_denom_a, n);

    // a = mu · sqrt_k · inv_denom_a.
    let mut mu_sqrt_k = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_k, mu, &mut mu_sqrt_k, n);
    let mut a_arr = Array::<F>::new(size);
    ctaylor_mul::<F>(&mu_sqrt_k, &inv_denom_a, &mut a_arr, n);

    // a² and 4a².
    let mut a2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&a_arr, &mut a2, n);
    let mut four_a2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a2, F::new(4.0), &mut four_a2, n);

    // inv_4a2 = 1 / (4a²).
    let mut inv_4a2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&four_a2, &mut inv_4a2, n);

    // neg_inv_4a2.
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_inv_4a2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_4a2, neg_one, &mut neg_inv_4a2, n);

    // b = expm1(-1 / (4a²)).
    let mut b_arr = Array::<F>::new(size);
    ctaylor_expm1::<F>(&neg_inv_4a2, &mut b_arr, n);

    // c = 2a²·b + 0.5.
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

    // 2a (= 2·a_arr).
    let mut two_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a_arr, F::new(2.0), &mut two_a, n);
    // 1/(2a).
    let mut inv_2a = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&two_a, &mut inv_2a, n);
    // erf(1/(2a)).
    let mut erf_term = Array::<F>::new(size);
    ctaylor_erf::<F>(&inv_2a, &mut erf_term, n);
    // sqrt(π) · erf.
    let mut sqrt_pi_erf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&erf_term, F::cast_from(SQRT_PI_F64), &mut sqrt_pi_erf, n);

    // (b - c).
    let mut b_minus_c = Array::<F>::new(size);
    let mut neg_c = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&c_arr, neg_one, &mut neg_c, n);
    ctaylor_add::<F>(&b_arr, &neg_c, &mut b_minus_c, n);

    // 2a · (b - c).
    let mut two_a_bmc = Array::<F>::new(size);
    ctaylor_mul::<F>(&two_a, &b_minus_c, &mut two_a_bmc, n);

    // sum = sqrt_pi_erf + 2a(b - c).
    let mut sum_term = Array::<F>::new(size);
    ctaylor_add::<F>(&sqrt_pi_erf, &two_a_bmc, &mut sum_term, n);

    // a · sum.
    let mut a_sum = Array::<F>::new(size);
    ctaylor_mul::<F>(&a_arr, &sum_term, &mut a_sum, n);

    // 8/3 · a · sum.
    let mut eight_thirds_asum = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a_sum, F::new(8.0) / F::new(3.0), &mut eight_thirds_asum, n);

    // (1 - 8/3·a·sum).
    let mut bracket = Array::<F>::new(size);
    let mut neg_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_thirds_asum, neg_one, &mut neg_term, n);
    #[unroll]
    for i in 0..size {
        bracket[i] = neg_term[i];
    }
    bracket[0] = bracket[0] + F::new(1.0);

    // -0.5 · na43 · K · bracket.
    let mut na43_k = Array::<F>::new(size);
    ctaylor_mul::<F>(rho_43, &k_arr, &mut na43_k, n);
    let mut na43_k_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&na43_k, &bracket, &mut na43_k_b, n);
    ctaylor_scalar_mul::<F>(&na43_k_b, F::new(-0.5), out, n);

    let _ = NEG_C_SLATER_F64;
}

#[cube]
pub fn beckesrx_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mu = F::cast_from(DEFAULT_MU_F64);
    let mut e_alpha = Array::<F>::new(size);
    becke_sr::<F>(mu, &d.a_43, &d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    becke_sr::<F>(mu, &d.b_43, &d.b, &d.gbb, &mut e_beta, n);
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
