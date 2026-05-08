//! XC_PBEINTX — PBEint Exchange Functional. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbeintx.cpp:18-41`
//!
//! # Formula
//! Like PBEX but with an s²-dependent interpolated μ:
//! ```cpp
//! st2 = S2(na, gaa);
//! mu = mu_GE + (mu_pbe - mu_GE) * alpha * st2 / (1 + alpha * st2);
//! t1 = 1 + mu*st2/kappa;
//! enh = 1 + kappa - kappa/t1;
//! lda = -c * na^(4/3);
//! return lda * enh;
//! ```
//! `alpha = 0.197`, `mu_pbe = 0.21951` (literal), `mu_GE = 10/81 ≈ 0.123456790123`,
//! `kappa = 0.804`.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{
    ALPHA_PBEINT_F64, MU_GE_F64, MU_PBEINT_PBE_F64, NEG_C_SLATER_F64, R_PBE_F64,
};
use crate::functionals::gga::shared::pw91_like;

#[cube]
fn pbeintx_channel<F: Float>(
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let kappa = F::cast_from(R_PBE_F64);
    let alpha = F::cast_from(ALPHA_PBEINT_F64);
    let mu_ge = F::cast_from(MU_GE_F64);
    let mu_pbe_minus_ge = F::cast_from(MU_PBEINT_PBE_F64 - MU_GE_F64);

    // s2_val = pw91_like::s2(rho, grad2).
    let mut s2_val = Array::<F>::new(size);
    pw91_like::s2::<F>(rho, grad2, &mut s2_val, n);

    // alpha_s2 = alpha * s2_val.
    let mut alpha_s2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2_val, alpha, &mut alpha_s2, n);

    // denom = 1 + alpha_s2.
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = alpha_s2[i];
    }
    denom[0] = denom[0] + F::new(1.0);

    // inv_denom = 1 / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // ratio = alpha_s2 · inv_denom.
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&alpha_s2, &inv_denom, &mut ratio, n);

    // mu_diff = (mu_pbe - mu_GE) · ratio.
    let mut mu_diff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ratio, mu_pbe_minus_ge, &mut mu_diff, n);

    // mu = mu_GE + mu_diff (CNST-bump).
    let mut mu_arr = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        mu_arr[i] = mu_diff[i];
    }
    mu_arr[0] = mu_arr[0] + mu_ge;

    // mu_s2_kappa = (mu · s2) / kappa.
    let mut mu_s2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&mu_arr, &s2_val, &mut mu_s2, n);
    let mut mu_s2_kappa = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&mu_s2, F::new(1.0) / kappa, &mut mu_s2_kappa, n);

    // t1 = 1 + mu_s2_kappa.
    let mut t1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        t1[i] = mu_s2_kappa[i];
    }
    t1[0] = t1[0] + F::new(1.0);

    // inv_t1 = 1 / t1.
    let mut inv_t1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&t1, &mut inv_t1, n);

    // rr = kappa · inv_t1.
    let mut rr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_t1, kappa, &mut rr, n);

    // enh = (1 + kappa) - rr.
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_rr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rr, neg_one, &mut neg_rr, n);
    let mut enh = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        enh[i] = neg_rr[i];
    }
    enh[0] = enh[0] + F::new(1.0) + kappa;

    // neg_pref = NEG_C_SLATER · rho_43.
    let mut neg_pref = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho_43, F::cast_from(NEG_C_SLATER_F64), &mut neg_pref, n);

    // out = neg_pref · enh.
    ctaylor_mul::<F>(&neg_pref, &enh, out, n);
}

#[cube]
pub fn pbeintx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut e_alpha = Array::<F>::new(size);
    pbeintx_channel::<F>(&d.a_43, &d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    pbeintx_channel::<F>(&d.b_43, &d.b, &d.gbb, &mut e_beta, n);
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
