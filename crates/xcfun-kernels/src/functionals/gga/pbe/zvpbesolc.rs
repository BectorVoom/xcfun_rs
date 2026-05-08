//! XC_ZVPBESOLC — zvPBEsol correlation. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/zvpbesolc.cpp:19-93`
//!
//! # Formula
//! Like PBELOCC but with a polynomial fit for |ζ|^ω (ω=4.5) and an
//! exponential cutoff:
//! ```cpp
//! beta = 0.046; alpha = 1.8;
//! bg = beta / γ;
//! tt = sqrt(d2);
//! v = tt · u · (r_s/3)^(-1/6);
//! v3 = v³;
//! zw = (0.462757 + 1.30129·ζ² - 1.59546·ζ⁴ + 1.19635·ζ⁶ - 0.36519·ζ⁸) · ζ⁴
//! ff = exp(-α · v³ · zw);
//! return n · (eps + ff · H);
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_exp, ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_2, ctaylor_powi_3,
    ctaylor_powi_4, ctaylor_powi_6, ctaylor_powi_8, ctaylor_reciprocal,
};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{PBEC_D2_PREFACTOR_F64, PBEC_GAMMA_F64};
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;

const ZVPBESOLC_BETA_F64: f64 = 0.046_f64;
const ZVPBESOLC_ALPHA_F64: f64 = 1.8_f64;
// β / γ = 0.046 / 0.0310906908696549.
const ZVPBESOLC_BG_F64: f64 = 1.479_546_797_054_858_f64;

// ζ-polynomial coefficients (zvpbesolc.cpp:81-86).
const POLY_C0_F64: f64 = 0.462_757_f64;
const POLY_C2_F64: f64 = 1.301_29_f64;
const POLY_C4_F64: f64 = -1.595_46_f64;
const POLY_C6_F64: f64 = 1.196_35_f64;
const POLY_C8_F64: f64 = -0.365_19_f64;

#[cube]
pub fn zvpbesolc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    zvpbe_common::<F>(
        d,
        F::cast_from(ZVPBESOLC_ALPHA_F64),
        F::cast_from(ZVPBESOLC_BG_F64),
        out,
        n,
    );
}

/// Shared body for ZVPBESOLC and ZVPBEINTC; only α and β/γ differ between them.
#[cube]
pub fn zvpbe_common<F: Float>(
    d: &DensVarsDev<F>,
    alpha: F,
    bg: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    let mut u = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut u, n);
    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);
    let mut denom_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut denom_d, n);
    let mut inv_denom_d = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_d, &mut inv_denom_d, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_denom_d, &mut g_over_d, n);
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    // tt = sqrt(d2).
    let mut tt = Array::<F>::new(size);
    ctaylor_pow::<F>(&d2, F::cast_from(0.5_f64), &mut tt, n);

    // v = tt · u · (r_s/3)^(-1/6) = tt · u · pow(r_s/3, -1/6).
    let mut rs_div_3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::new(1.0) / F::new(3.0), &mut rs_div_3, n);
    let mut rs3_pow = Array::<F>::new(size);
    ctaylor_pow::<F>(&rs_div_3, F::cast_from(-1.0_f64 / 6.0_f64), &mut rs3_pow, n);
    let mut tt_u = Array::<F>::new(size);
    ctaylor_mul::<F>(&tt, &u, &mut tt_u, n);
    let mut v = Array::<F>::new(size);
    ctaylor_mul::<F>(&tt_u, &rs3_pow, &mut v, n);
    let mut v3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&v, &mut v3, n);

    // zw = (c0 + c2·ζ² + c4·ζ⁴ + c6·ζ⁶ + c8·ζ⁸) · ζ⁴.
    let mut z2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.zeta, &mut z2, n);
    let mut z4 = Array::<F>::new(size);
    ctaylor_powi_4::<F>(&d.zeta, &mut z4, n);
    let mut z6 = Array::<F>::new(size);
    ctaylor_powi_6::<F>(&d.zeta, &mut z6, n);
    let mut z8 = Array::<F>::new(size);
    ctaylor_powi_8::<F>(&d.zeta, &mut z8, n);

    // poly = c0 + c2·z² + c4·z⁴ + c6·z⁶ + c8·z⁸.
    let mut t_c2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z2, F::cast_from(POLY_C2_F64), &mut t_c2, n);
    let mut t_c4 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z4, F::cast_from(POLY_C4_F64), &mut t_c4, n);
    let mut t_c6 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z6, F::cast_from(POLY_C6_F64), &mut t_c6, n);
    let mut t_c8 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z8, F::cast_from(POLY_C8_F64), &mut t_c8, n);

    // poly = c0 (CNST) + sum.
    let mut sum_c = Array::<F>::new(size);
    ctaylor_add::<F>(&t_c2, &t_c4, &mut sum_c, n);
    let mut sum_c2 = Array::<F>::new(size);
    ctaylor_add::<F>(&sum_c, &t_c6, &mut sum_c2, n);
    let mut poly = Array::<F>::new(size);
    ctaylor_add::<F>(&sum_c2, &t_c8, &mut poly, n);
    poly[0] = poly[0] + F::cast_from(POLY_C0_F64);

    let mut zw = Array::<F>::new(size);
    ctaylor_mul::<F>(&poly, &z4, &mut zw, n);

    // ff = exp(-α · v³ · zw).
    let mut v3_zw = Array::<F>::new(size);
    ctaylor_mul::<F>(&v3, &zw, &mut v3_zw, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_alpha_v3_zw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&v3_zw, neg_one * alpha, &mut neg_alpha_v3_zw, n);
    let mut ff = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_alpha_v3_zw, &mut ff, n);

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
    ctaylor_scalar_mul::<F>(&inv_em1, bg, &mut a, n);

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
    let mut num1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d2, bg, &mut num1, n);
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

    // ff_h = ff · H.
    let mut ff_h = Array::<F>::new(size);
    ctaylor_mul::<F>(&ff, &h, &mut ff_h, n);
    // sum = eps + ff_h.
    let mut sum_eps = Array::<F>::new(size);
    ctaylor_add::<F>(&eps, &ff_h, &mut sum_eps, n);
    // out = n · sum.
    ctaylor_mul::<F>(&d.n, &sum_eps, out, n);
    let _ = ZVPBESOLC_BETA_F64;
}
