//! Thomas-Weizsäcker kinetic energy functional. **LDA-09 part 2 (kinetic-GGA).**
//!
//! # Source
//! - `xcfun-master/src/functionals/tw.cpp:20-30` (depends `XC_DENSITY | XC_GRADIENT`)
//!
//! # Formula
//! $$ T_W = \frac{1}{8} \cdot (\text{gaa} + \text{gbb})^2 / n $$
//!
//! # Preconditions (Pitfall PHASE2-D)
//! - `d.gaa`, `d.gbb` populated by `build_densvars` `XC_A_B_GAA_GAB_GBB` arm (Plan 02-05 Wave-1C-1).
//! - The pure-density `XC_A_B` arm leaves `gaa = gbb = 0` — TW would silently
//!   return 0 in that case. Must be driven through the GAA_GAB_GBB builder.
//! - `d.n > 0` (regularize ensures `>= TINY_DENSITY = 1e-14`; division by zero guarded).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_powi_2, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;

/// Thomas-Weizsäcker kinetic kernel. 1:1 port of `tw.cpp:20-22`:
///
/// ```cpp
/// return 1. / 8. * pow(d.gaa + d.gbb, 2.0) / d.n;
/// ```
///
/// Operation order (left-to-right, ACC-06 no mul_add):
///   1. sum = gaa + gbb                 (ctaylor_add)
///   2. sum2 = sum^2                    (ctaylor_powi_2 — fused x*x)
///   3. inv_n = 1/n                     (ctaylor_reciprocal)
///   4. tmp = sum2 * inv_n              (ctaylor_mul)
///   5. out = 0.125 * tmp               (ctaylor_scalar_mul; 1/8 = 0.125)
#[cube]
pub fn tw_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let mut sum = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut sum2 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut inv_n = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut tmp = Array::<F>::new(comptime!((1_u32 << n) as usize));

    ctaylor_add::<F>(&d.gaa, &d.gbb, &mut sum, n);
    ctaylor_powi_2::<F>(&sum, &mut sum2, n);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, n);
    ctaylor_mul::<F>(&sum2, &inv_n, &mut tmp, n);
    ctaylor_scalar_mul::<F>(&tmp, F::cast_from(0.125_f64), out, n);
}
