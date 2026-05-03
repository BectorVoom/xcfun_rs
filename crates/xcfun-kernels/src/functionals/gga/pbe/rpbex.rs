//! XC_RPBEX — RPBE Exchange Functional. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/rpbex.cpp:18-23`
//!
//! # Formula
//! ```cpp
//! return prefactor(d.a) * enhancement_RPBE(d.a, d.gaa)
//!      + prefactor(d.b) * enhancement_RPBE(d.b, d.gbb);
//! ```
//!
//! Uses `pw91_like::prefactor` (FULL body — Wave 2) and
//! `pbex::enhancement_rpbe` (FULL body — Wave 2).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::{pbex as pbex_shared, pw91_like};

#[cube]
pub fn rpbex_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // alpha-channel:
    let mut pre_a = Array::<F>::new(size);
    pw91_like::prefactor::<F>(&d.a, &mut pre_a, n);
    let mut enh_a = Array::<F>::new(size);
    pbex_shared::enhancement_rpbe::<F>(&d.a, &d.gaa, &mut enh_a, n);
    let mut e_alpha = Array::<F>::new(size);
    ctaylor_mul::<F>(&pre_a, &enh_a, &mut e_alpha, n);

    // beta-channel:
    let mut pre_b = Array::<F>::new(size);
    pw91_like::prefactor::<F>(&d.b, &mut pre_b, n);
    let mut enh_b = Array::<F>::new(size);
    pbex_shared::enhancement_rpbe::<F>(&d.b, &d.gbb, &mut enh_b, n);
    let mut e_beta = Array::<F>::new(size);
    ctaylor_mul::<F>(&pre_b, &enh_b, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
