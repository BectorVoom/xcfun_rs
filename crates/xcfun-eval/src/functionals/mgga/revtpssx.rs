//! XC_REVTPSSX — revised TPSS exchange functional. MGGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/revtpssx.cpp:19-21`
//! - `xcfun-master/src/functionals/revtpssx_eps.hpp:58-64`
//!
//! # Formula (port of `revtpssx_eps`):
//! ```cpp
//! num Fxa = F_x(2*d.a, 4*d.gaa, 2*d.taua);
//! num epsxunif_a = epsx_unif(2*d.a);
//! num Fxb = F_x(2*d.b, 4*d.gbb, 2*d.taub);
//! num epsxunif_b = epsx_unif(2*d.b);
//! return (epsxunif_a * Fxa * d.a + epsxunif_b * Fxb * d.b);
//! ```
//!
//! Note: revTPSS returns `epsxunif * F_x * spin_density` (unlike TPSS which
//! uses 0.5 * (epsxunif * F_x) without explicit spin factor).
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::tpss_like;

/// Compute revTPSS exchange for one spin channel:
/// `epsx_unif(2*rho) * F_x(2*rho, 4*grad2, 2*tau) * rho`.
#[cube]
fn revtpss_exchange_spin<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    use xcfun_ad::ctaylor::ctaylor_scalar_mul;
    let size = comptime!((1_u32 << n) as usize);

    let mut two_rho = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho, F::cast_from(2.0_f64), &mut two_rho, n);

    let mut four_grad2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(grad2, F::cast_from(4.0_f64), &mut four_grad2, n);

    let mut two_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(tau, F::cast_from(2.0_f64), &mut two_tau, n);

    // F_x(2rho, 4grad2, 2tau)
    let mut fx = Array::<F>::new(size);
    tpss_like::revtpss_fx::<F>(&two_rho, &four_grad2, &mut two_tau, &mut fx, n);

    // epsx_unif(2*rho)
    let mut eps_unif = Array::<F>::new(size);
    tpss_like::revtpss_epsx_unif::<F>(&two_rho, &mut eps_unif, n);

    // eps_unif * F_x
    let mut eps_fx = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps_unif, &fx, &mut eps_fx, n);

    // * rho (spin density)
    ctaylor_mul::<F>(&eps_fx, rho, out, n);
}

#[cube]
pub fn revtpssx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut ea = Array::<F>::new(size);
    revtpss_exchange_spin::<F>(&d.a, &d.gaa, &d.taua, &mut ea, n);

    let mut eb = Array::<F>::new(size);
    revtpss_exchange_spin::<F>(&d.b, &d.gbb, &d.taub, &mut eb, n);

    ctaylor_add::<F>(&ea, &eb, out, n);
}
