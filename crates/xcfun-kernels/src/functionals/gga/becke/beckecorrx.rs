//! XC_BECKECORRX — Becke 88 exchange correction (no LDA part). GGA-02.
//!
//! # Source
//! - `xcfun-master/src/functionals/beckex.cpp:27-32, 79-81`
//!
//! # Formula
//! ```cpp
//! d = 0.0042;
//! na43 = pow(na, 4/3);
//! chi2 = gaa * pow(na, -8/3);
//! return -(d * na43 * chi2) / (1 + 6*d*sqrtx_asinh_sqrtx(chi2));
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_reciprocal, ctaylor_sqrtx_asinh_sqrtx};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{BECKE_6D_F64, BECKE_D_F64};
use crate::functionals::gga::shared::pw91_like;

#[cube]
fn becke_corr<F: Float>(
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut chi2_a = Array::<F>::new(size);
    pw91_like::chi2::<F>(rho, grad2, &mut chi2_a, n);
    let mut sas = Array::<F>::new(size);
    ctaylor_sqrtx_asinh_sqrtx::<F>(&chi2_a, &mut sas, n);
    let mut six_d_sas = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sas, F::cast_from(BECKE_6D_F64), &mut six_d_sas, n);
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = six_d_sas[i];
    }
    denom[0] = denom[0] + F::new(1.0);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    let mut num = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&chi2_a, F::cast_from(BECKE_D_F64), &mut num, n);
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&num, &inv_denom, &mut ratio, n);
    let mut na43_ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(rho_43, &ratio, &mut na43_ratio, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&na43_ratio, neg_one, out, n);
}

#[cube]
pub fn beckecorrx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut e_alpha = Array::<F>::new(size);
    becke_corr::<F>(&d.a_43, &d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    becke_corr::<F>(&d.b_43, &d.b, &d.gbb, &mut e_beta, n);
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
