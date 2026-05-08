//! von Weizsäcker kinetic energy functional. **LDA-10 (kinetic-GGA).**
//!
//! # Source
//! - `xcfun-master/src/functionals/vonw.cpp:17-30` (file is `vonw.cpp`,
//!   FUNCTIONAL macro is `XC_VWK`; depends `XC_DENSITY | XC_GRADIENT`).
//!
//! # Formula
//! $$ T_W = \frac{\text{gaa}}{8 \cdot a} + \frac{\text{gbb}}{8 \cdot b} $$
//!
//! # Preconditions (Pitfall PHASE2-D)
//! - `d.gaa`, `d.gbb` populated by `build_densvars` `XC_A_B_GAA_GAB_GBB` arm (Plan 02-05 Wave-1C-1).
//! - The pure-density `XC_A_B` arm leaves `gaa = gbb = 0` — VWK would silently
//!   return 0 in that case. Must be driven through the GAA_GAB_GBB builder.
//! - `d.a > 0`, `d.b > 0` (regularize ensures `>= TINY_DENSITY = 1e-14`;
//!   division by zero guarded).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;

/// Per-spin von Weizsäcker term. Port of `vW_alpha(na, gaa)` from
/// `xcfun-master/src/functionals/vonw.cpp:17-19`:
///
/// ```cpp
/// template <typename num> static num vW_alpha(const num & na, const num & gaa) {
///     return gaa / (8 * na);
/// }
/// ```
///
/// Rewritten as `out = 0.125 * gaa * (1/na)` (operation order left-to-right,
/// ACC-06 no mul_add):
///   1. inv_na = 1/na                   (ctaylor_reciprocal)
///   2. tmp = gaa * inv_na              (ctaylor_mul)
///   3. out = 0.125 * tmp               (ctaylor_scalar_mul; 1/8 = 0.125)
#[cube]
fn vw_alpha<F: Float>(na: &Array<F>, gaa: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let mut inv_na = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut tmp = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_reciprocal::<F>(na, &mut inv_na, n);
    ctaylor_mul::<F>(gaa, &inv_na, &mut tmp, n);
    ctaylor_scalar_mul::<F>(&tmp, F::cast_from(0.125_f64), out, n);
}

/// von Weizsäcker kinetic kernel. 1:1 port of `vonw.cpp:21-23`:
///
/// ```cpp
/// template <typename num> static num vW(const densvars<num> & d) {
///     return vW_alpha(d.a, d.gaa) + vW_alpha(d.b, d.gbb);
/// }
/// ```
#[cube]
pub fn vwk_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let mut va = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut vb = Array::<F>::new(comptime!((1_u32 << n) as usize));
    vw_alpha::<F>(&d.a, &d.gaa, &mut va, n);
    vw_alpha::<F>(&d.b, &d.gbb, &mut vb, n);
    ctaylor_add::<F>(&va, &vb, out, n);
}
