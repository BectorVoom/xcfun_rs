//! Thomas-Fermi kinetic energy functional. **LDA-09 part 1 (pure density).**
//!
//! Plan 02-05 Wave-1C-2 ships LDA-09 part 2 (`tw` — kinetic-GGA via XC_A_B_GAA_GAB_GBB).
//!
//! # Source
//! - `xcfun-master/src/functionals/tfk.cpp:20-22`
//!
//! # Formula
//! $$ T_{TF} = C_F \cdot n^{5/3} $$
//! where $C_F = (3/10) \cdot (3 \pi^2)^{2/3} \approx 2.871184293$.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_scalar_mul;
use xcfun_ad::math::ctaylor_pow;

use crate::density_vars::DensVarsDev;

/// Thomas-Fermi prefactor — matches `xcfun-core::constants::CF = 2.8711842930059836`.
/// f32 rounding: 2.8711843_f32 (exact to 7 significant figures).
const CF_F32: f32 = 2.871_184_3_f32;

/// Thomas-Fermi kinetic kernel. 1:1 port of `tfk.cpp:20-22`:
/// `return CF * pow(d.n, 5.0 / 3.0);`
#[cube]
pub fn tfk_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let mut n_53 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_pow::<F>(&d.n, F::new(5.0_f32 / 3.0_f32), &mut n_53, n);
    ctaylor_scalar_mul::<F>(&n_53, F::new(CF_F32), out, n);
}
