//! CSC kernel — Colle-Salvetti correlation.
//!
//! Port of `xcfun-master/src/functionals/cs.cpp:17-27`.
//! Dispatch ID: XC_CSC = 66. Vars id=17 (same arm as BR family).
//!
//! Body delegates entirely to `shared::cs::csc_energy`.

use cubecl::prelude::*;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::cs::csc_energy;

/// CSC correlation energy kernel.
///
/// Port of `xcfun-master/src/functionals/cs.cpp:17-27` (the `csc(d)` function).
#[cube(launch_unchecked)]
pub fn csc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    csc_energy::<F>(d, out, n);
}
