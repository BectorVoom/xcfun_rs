//! XC_BECKEX — Becke 88 exchange. GGA-02.
//!
//! # Source
//! - `xcfun-master/src/functionals/beckex.cpp:17-25, 75-77`
//!
//! # Formula (port of `becke_alpha`):
//! ```cpp
//! c    = pow(81/(4π), 1/3) / 2;     // = -NEG_C_SLATER (analytically identical)
//! d    = 0.0042;
//! na43 = pow(na, 4/3);
//! lda  = -c * na43;                  // = NEG_C_SLATER · na43
//! chi2 = gaa * pow(na, -8/3);
//! b88  = -(d * na43 * chi2) / (1 + 6*d*sqrtx_asinh_sqrtx(chi2));
//! return lda + b88;
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_reciprocal, ctaylor_sqrtx_asinh_sqrtx};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{BECKE_6D_F64, BECKE_D_F64, NEG_C_SLATER_F64};
use crate::functionals::gga::shared::pw91_like;

#[cube]
fn becke_alpha<F: Float>(
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // chi2 = grad² · ρ^(-8/3)  (via shared helper).
    let mut chi2_a = Array::<F>::new(size);
    pw91_like::chi2::<F>(rho, grad2, &mut chi2_a, n);

    // sas = sqrtx_asinh_sqrtx(chi2).
    let mut sas = Array::<F>::new(size);
    ctaylor_sqrtx_asinh_sqrtx::<F>(&chi2_a, &mut sas, n);

    // 6d·sas.
    let mut six_d_sas = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sas, F::cast_from(BECKE_6D_F64), &mut six_d_sas, n);

    // denom = 1 + 6d·sas.
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = six_d_sas[i];
    }
    denom[0] = denom[0] + F::new(1.0);

    // inv_denom = 1 / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // num = d · chi².
    let mut num = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&chi2_a, F::cast_from(BECKE_D_F64), &mut num, n);

    // ratio = num · inv_denom.
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&num, &inv_denom, &mut ratio, n);

    // b88 = -(na43 · ratio).
    let mut na43_ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(rho_43, &ratio, &mut na43_ratio, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut b88 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&na43_ratio, neg_one, &mut b88, n);

    // lda = NEG_C_SLATER · rho_43.
    let mut lda = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho_43, F::cast_from(NEG_C_SLATER_F64), &mut lda, n);

    // out = lda + b88.
    ctaylor_add::<F>(&lda, &b88, out, n);
}

#[cube]
pub fn beckex_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut e_alpha = Array::<F>::new(size);
    becke_alpha::<F>(&d.a_43, &d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    becke_alpha::<F>(&d.b_43, &d.b, &d.gbb, &mut e_beta, n);
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
