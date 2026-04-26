//! XC_RSCANC — rSCAN correlation functional. MGGA-02.
//!
//! Port of `xcfun-master/src/functionals/rSCANc.cpp`.
//! Uses `SCAN_C(d, IALPHA=1, IINTERP=1, IDELEC=0)`.
//! Vars: XC_A_B_GAA_GAB_GBB_TAUA_TAUB (id=13, inlen=7).

#![allow(non_snake_case)]

use cubecl::prelude::*;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::scan_like;

#[cube(launch_unchecked)]
pub fn rscanc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    scan_like::r2SCAN_C::<F>(d, out, 1_u32, 1_u32, 0_u32, n);
}
