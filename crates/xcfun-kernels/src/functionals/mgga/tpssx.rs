//! XC_TPSSX — TPSS exchange functional. MGGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/tpssx.cpp:21-27`
//! - `xcfun-master/src/functionals/tpssx_eps.hpp:1-60`
//!
//! # Formula (port of `tpssx`):
//! ```cpp
//! num Fxa = F_x(2*d.a, 4*d.gaa, 2*d.taua);
//! num epsxunif_a = fx_unif(2*d.a);
//! num Fxb = F_x(2*d.b, 4*d.gbb, 2*d.taub);
//! num epsxunif_b = fx_unif(2*d.b);
//! return 0.5 * (epsxunif_a * Fxa + epsxunif_b * Fxb);
//! ```
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::tpss_like;

/// Compute TPSS exchange for one spin channel: `eps_unif(2*rho) * F_x(2*rho, 4*grad2, 2*tau)`.
#[cube]
fn tpss_exchange_spin<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // 2*rho, 4*grad2, 2*tau (spin-scaling)
    let mut two_rho = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho, F::cast_from(2.0_f64), &mut two_rho, n);

    let mut four_grad2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(grad2, F::cast_from(4.0_f64), &mut four_grad2, n);

    let mut two_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(tau, F::cast_from(2.0_f64), &mut two_tau, n);

    // F_x(2rho, 4grad2, 2tau)
    let mut fx = Array::<F>::new(size);
    tpss_like::tpss_F_x::<F>(&two_rho, &four_grad2, &two_tau, &mut fx, n);

    // fx_unif(2*rho)
    let mut eps_unif = Array::<F>::new(size);
    tpss_like::tpss_fx_unif::<F>(&two_rho, &mut eps_unif, n);

    // eps_unif * F_x
    ctaylor_mul::<F>(&eps_unif, &fx, out, n);
}

#[cube]
pub fn tpssx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut ea = Array::<F>::new(size);
    tpss_exchange_spin::<F>(&d.a, &d.gaa, &d.taua, &mut ea, n);

    let mut eb = Array::<F>::new(size);
    tpss_exchange_spin::<F>(&d.b, &d.gbb, &d.taub, &mut eb, n);

    // 0.5 * (ea + eb)
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&ea, &eb, &mut sum, n);
    ctaylor_scalar_mul::<F>(&sum, F::cast_from(0.5_f64), out, n);
}
