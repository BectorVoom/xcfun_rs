//! XC_PBELOCC — PBEloc correlation functional. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbelocc.cpp:19-41`
//!
//! # Formula
//! Like PBEC but with a position-dependent β:
//! ```cpp
//! beta0 = 0.0375; aa = 0.08;
//! ff = 1 - exp(-r_s²);
//! beta = beta0 + aa · d2 · ff;
//! bg = beta / γ;
//! H = γ·u³·log(1 + bg·d2·(1+d2A)/(1+d2A·(1+d2A)))
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_exp, ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_2, ctaylor_powi_3,
    ctaylor_reciprocal,
};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{PBEC_D2_PREFACTOR_F64, PBEC_GAMMA_F64};
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;

const PBELOCC_BETA0_F64: f64 = 0.0375_f64;
const PBELOCC_AA_F64: f64 = 0.08_f64;

#[cube]
pub fn pbelocc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

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

    // ff = 1 - exp(-r_s²).
    let mut rs2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.r_s, &mut rs2, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_rs2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs2, neg_one, &mut neg_rs2, n);
    let mut exp_term = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_rs2, &mut exp_term, n);
    let mut ff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&exp_term, neg_one, &mut ff, n);
    ff[0] = ff[0] + F::new(1.0);

    // beta = beta0 + aa · d2 · ff.
    let mut d2_ff = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &ff, &mut d2_ff, n);
    let mut aa_d2_ff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d2_ff, F::cast_from(PBELOCC_AA_F64), &mut aa_d2_ff, n);
    let mut beta_arr = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        beta_arr[i] = aa_d2_ff[i];
    }
    beta_arr[0] = beta_arr[0] + F::cast_from(PBELOCC_BETA0_F64);

    // bg = beta / γ.
    let mut bg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &beta_arr,
        F::new(1.0) / F::cast_from(PBEC_GAMMA_F64),
        &mut bg,
        n,
    );

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    // A = bg / expm1(-eps / (γ · u³)).
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&u3, F::cast_from(PBEC_GAMMA_F64), &mut gu3, n);
    let mut inv_gu3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gu3, &mut inv_gu3, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps, &inv_gu3, &mut prod, n);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&prod, neg_one, &mut arg, n);
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);
    let mut a = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg, &inv_em1, &mut a, n);

    // d2A = d2 · a.
    let mut d2a = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &a, &mut d2a, n);
    let mut one_d2a = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2a[i] = d2a[i];
    }
    one_d2a[0] = one_d2a[0] + F::new(1.0);
    let mut inner = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2a, &one_d2a, &mut inner, n);
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = inner[i];
    }
    den[0] = den[0] + F::new(1.0);
    // num1 = bg · d2.
    let mut num1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg, &d2, &mut num1, n);
    let mut num2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&num1, &one_d2a, &mut num2, n);
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num2, &inv_den, &mut frac, n);
    let mut lg_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        lg_arg[i] = frac[i];
    }
    lg_arg[0] = lg_arg[0] + F::new(1.0);
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&lg_arg, &mut lg, n);
    let mut h = Array::<F>::new(size);
    ctaylor_mul::<F>(&gu3, &lg, &mut h, n);

    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&eps, &h, &mut sum, n);
    ctaylor_mul::<F>(&d.n, &sum, out, n);
}
