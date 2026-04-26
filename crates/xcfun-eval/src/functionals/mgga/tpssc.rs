//! XC_TPSSC — TPSS correlation functional. MGGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/tpssc.cpp:19-21`
//! - `xcfun-master/src/functionals/tpssc_eps.hpp`
//!
//! # Formula:
//! ```cpp
//! num eps = tpssc_eps::tpssc_eps(d);
//! return d.n * eps;
//! ```
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::tpss_like;

#[cube]
pub fn tpssc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut eps = Array::<F>::new(size);
    tpss_like::tpss_eps_full::<F>(d, &mut eps, n);

    // out = d.n * eps
    ctaylor_mul::<F>(&d.n, &eps, out, n);
}
