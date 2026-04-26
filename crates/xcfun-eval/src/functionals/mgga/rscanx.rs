//! XC_RSCANX — rSCAN exchange functional. MGGA-02.
//!
//! Port of `xcfun-master/src/functionals/rSCANx.cpp`.
//! Uses `get_SCAN_Fx(2*d.a, 4*d.gaa, 2*d.taua, IALPHA=1, IINTERP=1, IDELFX=0)`.
//! Vars: XC_A_B_GAA_GAB_GBB_TAUA_TAUB (id=13, inlen=7).

#![allow(non_snake_case)]

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::scan_like;

#[cube]
fn rscan_exchange_spin<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut two_rho = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho, F::cast_from(2.0_f64), &mut two_rho, n);
    let mut four_grad2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(grad2, F::cast_from(4.0_f64), &mut four_grad2, n);
    let mut two_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(tau, F::cast_from(2.0_f64), &mut two_tau, n);

    let mut fx = Array::<F>::new(size);
    scan_like::get_SCAN_Fx::<F>(&two_rho, &four_grad2, &two_tau, &mut fx, 1_u32, 1_u32, 0_u32, n);

    let mut eps_unif = Array::<F>::new(size);
    scan_like::scan_fx_unif::<F>(&two_rho, &mut eps_unif, n);

    ctaylor_mul::<F>(&eps_unif, &fx, out, n);
}

#[cube(launch_unchecked)]
pub fn rscanx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut ea = Array::<F>::new(size);
    rscan_exchange_spin::<F>(&d.a, &d.gaa, &d.taua, &mut ea, n);

    let mut eb = Array::<F>::new(size);
    rscan_exchange_spin::<F>(&d.b, &d.gbb, &d.taub, &mut eb, n);

    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&ea, &eb, &mut sum, n);
    ctaylor_scalar_mul::<F>(&sum, F::cast_from(0.5_f64), out, n);
}
