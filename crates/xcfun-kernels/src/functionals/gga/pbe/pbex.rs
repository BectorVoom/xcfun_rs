//! XC_PBEX — PBE Exchange Functional. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbex.cpp:20-24`
//!
//! # Formula (port of `pbex.cpp:20-23`):
//! ```cpp
//! return pbex::energy_pbe_ab(pbex::R_pbe, d.a, d.gaa)
//!      + pbex::energy_pbe_ab(pbex::R_pbe, d.b, d.gbb);
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::R_PBE_F64;
use crate::functionals::gga::shared::pbex as pbex_shared;

#[cube]
pub fn pbex_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let r = F::cast_from(R_PBE_F64);

    // alpha-channel: e_alpha = energy_pbe_ab(R_pbe, d.a, d.gaa).
    let mut e_alpha = Array::<F>::new(size);
    pbex_shared::energy_pbe_ab::<F>(r, &d.a_43, &d.a, &d.gaa, &mut e_alpha, n);

    // beta-channel: e_beta = energy_pbe_ab(R_pbe, d.b, d.gbb).
    let mut e_beta = Array::<F>::new(size);
    pbex_shared::energy_pbe_ab::<F>(r, &d.b_43, &d.b, &d.gbb, &mut e_beta, n);

    // out = e_alpha + e_beta.
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
