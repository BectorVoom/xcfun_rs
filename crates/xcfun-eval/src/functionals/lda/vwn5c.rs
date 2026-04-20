//! VWN5 LDA correlation. **LDA-03.**
//!
//! # Source
//! - `xcfun-master/src/functionals/vwn5c.cpp:18-20` (`d.n * vwn::vwn5_eps(d)`)
//! - `xcfun-master/src/functionals/vwn.hpp:54-78` (vwn5_eps formula)
//!
//! # Preconditions
//! - `d.n`, `d.r_s`, `d.zeta` populated by `build_densvars` (XC_A_B arm).

use cubecl::prelude::*;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use super::vwn_eps::vwn5_eps;
use crate::density_vars::DensVarsDev;

/// VWN5 correlation kernel. 1:1 port of `vwn5c.cpp:18-20`.
#[cube]
pub fn vwn5c_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // C++: return d.n * vwn::vwn5_eps(d);
    let mut eps = Array::<F>::new(comptime!((1_u32 << n) as usize));
    vwn5_eps::<F>(d, &mut eps, n);
    ctaylor_mul::<F>(&d.n, &eps, out, n);
}
