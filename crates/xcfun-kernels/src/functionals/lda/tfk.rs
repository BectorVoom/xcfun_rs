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

/// Thomas-Fermi prefactor — computed at Rust const-time from the same formula
/// as `xcfun-master/src/functionals/constants.hpp:31`: `CF = 0.3 * (3*π²)^(2/3)`.
///
/// Evaluated in f64: `2.871234000188192`. NOTE: `xcfun-core::constants::CF`
/// ships the value `2.8711842930059836` which differs by ~5e-5 — a pre-existing
/// xcfun-core discrepancy (see [issue tracker]; scope boundary — Plan 02-04
/// doesn't modify xcfun-core). Kernel uses the correct C++ runtime value so
/// TFK tier-1 parity passes at the 1e-5 threshold.
const CF_F64: f64 = 2.871_234_000_188_192_f64;

/// Thomas-Fermi kinetic kernel. 1:1 port of `tfk.cpp:20-22`:
/// `return CF * pow(d.n, 5.0 / 3.0);`
#[cube]
pub fn tfk_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let mut n_53 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_pow::<F>(&d.n, F::cast_from(5.0_f64 / 3.0_f64), &mut n_53, n);
    ctaylor_scalar_mul::<F>(&n_53, F::cast_from(CF_F64), out, n);
}
