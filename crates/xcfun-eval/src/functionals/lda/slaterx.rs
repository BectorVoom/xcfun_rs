//! Slater LDA exchange functional. **LDA-01.**
//!
//! # Source
//! - `xcfun-master/src/functionals/slaterx.cpp:18-37` (FUNCTIONAL macro + test data)
//! - `xcfun-master/src/functionals/slater.hpp:19-21` (formula)
//!
//! # Formula
//! $$ E_x = -c_{\text{slater}} \cdot (a^{4/3} + b^{4/3}) $$
//! where $c_{\text{slater}} = (81/(32\pi))^{1/3} \approx 0.93052574$.
//!
//! # Preconditions
//! - `d.a_43`, `d.b_43` populated by `build_densvars` (XC_A_B variant arm — Plan 02-03 Wave-1B-3).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};

use crate::density_vars::DensVarsDev;

/// `-(81 / (32 * π))^(1/3)` — negated Slater exchange constant, matching the
/// C++ expression `(-xcfun_constants::c_slater) * (d.a_43 + d.b_43)` in
/// `slater.hpp:19-21`. `xcfun-core::constants::C_SLATER = 0.9305257363491002`.
const NEG_C_SLATER_F32: f32 = -0.930_525_7_f32;

/// Slater LDA exchange kernel. 1:1 port of `slater.hpp:19-21`.
#[cube]
pub fn slaterx_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // C++: return (-c_slater) * (d.a_43 + d.b_43);
    //
    // Operation order (matches C++ left-to-right; ACC-06 forbids mul_add):
    //   1. tmp = a_43 + b_43         (ctaylor_add)
    //   2. out = (-c_slater) * tmp    (ctaylor_scalar_mul)
    let mut tmp = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_add::<F>(&d.a_43, &d.b_43, &mut tmp, n);
    ctaylor_scalar_mul::<F>(&tmp, F::new(NEG_C_SLATER_F32), out, n);
}
