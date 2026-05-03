//! XC_ZVPBEINTC — zvPBEint correlation. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/zvpbeint.cpp:19-55`
//!
//! Same shape as ZVPBESOLC but with `beta = 0.052` and `alpha = 1.0`.

use cubecl::prelude::*;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::pbe::zvpbesolc::zvpbe_common;

const ZVPBEINTC_BETA_F64: f64 = 0.052_f64;
const ZVPBEINTC_ALPHA_F64: f64 = 1.0_f64;
// β / γ = 0.052 / 0.0310906908696549.
const ZVPBEINTC_BG_F64: f64 = 1.672_524_726_145_254_4_f64;

#[cube]
pub fn zvpbeintc_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = ZVPBEINTC_BETA_F64;
    zvpbe_common::<F>(
        d,
        F::cast_from(ZVPBEINTC_ALPHA_F64),
        F::cast_from(ZVPBEINTC_BG_F64),
        out,
        n,
    );
}
