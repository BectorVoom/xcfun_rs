//! VWN3 LDA correlation. **LDA-02.**
//!
//! # Source
//! - `xcfun-master/src/functionals/vwn3.cpp:18-20` (`d.n * vwn::vwn3_eps(d)`)
//! - `xcfun-master/src/functionals/vwn.hpp:80-90` (vwn3_eps formula)
//!
//! # Preconditions
//! - `d.n`, `d.r_s`, `d.zeta` populated by `build_densvars` (XC_A_B arm).

use cubecl::prelude::*;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use super::vwn_eps::vwn3_eps;
use crate::density_vars::DensVarsDev;

/// VWN3 correlation kernel. 1:1 port of `vwn3.cpp:18-20`.
#[cube]
pub fn vwn3c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // C++: return d.n * vwn::vwn3_eps(d);
    let mut eps = Array::<F>::new(comptime!((1_u32 << n) as usize));
    vwn3_eps::<F>(d, &mut eps, n);
    ctaylor_mul::<F>(&d.n, &eps, out, n);
}
