//! XC_M05X2C — M05-2X correlation functional. MGGA-04.
//!
//! # Source
//! - `xcfun-master/src/functionals/m05x2c.cpp:20-48`
//!
//! Same body shape as `m05c` with M05-2X-specific c-parameter sets.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::m0x_like;

// Anti-parallel param_c[5] from m05x2c.cpp:29-30.
const M05X2C_ANTI_C0_F64: f64 = 1.000_000e0_f64;
const M05X2C_ANTI_C1_F64: f64 = 1.092_970e0_f64;
const M05X2C_ANTI_C2_F64: f64 = -3.791_710e0_f64;
const M05X2C_ANTI_C3_F64: f64 = 2.828_100e0_f64;
const M05X2C_ANTI_C4_F64: f64 = -1.058_909e1_f64;

// Parallel param_c[5] from m05x2c.cpp:33-34.
const M05X2C_PARA_C0_F64: f64 = 1.000_000e0_f64;
const M05X2C_PARA_C1_F64: f64 = -3.054_300e0_f64;
const M05X2C_PARA_C2_F64: f64 = 7.618_540e0_f64;
const M05X2C_PARA_C3_F64: f64 = 1.476_650e0_f64;
const M05X2C_PARA_C4_F64: f64 = -1.192_365e1_f64;

#[cube]
pub fn m05x2c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut chi_a2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.a, &d.gaa, &mut chi_a2, n);
    let mut chi_b2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.b, &d.gbb, &mut chi_b2, n);

    let mut dsigma_a = Array::<F>::new(size);
    m0x_like::m0x_Dsigma::<F>(&d.a, &d.gaa, &d.taua, &mut dsigma_a, n);
    let mut dsigma_b = Array::<F>::new(size);
    m0x_like::m0x_Dsigma::<F>(&d.b, &d.gbb, &d.taub, &mut dsigma_b, n);

    let mut ueg_anti = Array::<F>::new(size);
    m0x_like::ueg_c_anti::<F>(d, &mut ueg_anti, n);
    let mut ueg_para_a = Array::<F>::new(size);
    m0x_like::ueg_c_para::<F>(&d.a, &mut ueg_para_a, n);
    let mut ueg_para_b = Array::<F>::new(size);
    m0x_like::ueg_c_para::<F>(&d.b, &mut ueg_para_b, n);

    let mut m05_anti = Array::<F>::new(size);
    m0x_like::m05_c_anti::<F>(
        F::cast_from(M05X2C_ANTI_C0_F64),
        F::cast_from(M05X2C_ANTI_C1_F64),
        F::cast_from(M05X2C_ANTI_C2_F64),
        F::cast_from(M05X2C_ANTI_C3_F64),
        F::cast_from(M05X2C_ANTI_C4_F64),
        &chi_a2,
        &chi_b2,
        &mut m05_anti,
        n,
    );

    let mut m05_para_a = Array::<F>::new(size);
    m0x_like::m05_c_para::<F>(
        F::cast_from(M05X2C_PARA_C0_F64),
        F::cast_from(M05X2C_PARA_C1_F64),
        F::cast_from(M05X2C_PARA_C2_F64),
        F::cast_from(M05X2C_PARA_C3_F64),
        F::cast_from(M05X2C_PARA_C4_F64),
        &chi_a2,
        &dsigma_a,
        &mut m05_para_a,
        n,
    );

    let mut m05_para_b = Array::<F>::new(size);
    m0x_like::m05_c_para::<F>(
        F::cast_from(M05X2C_PARA_C0_F64),
        F::cast_from(M05X2C_PARA_C1_F64),
        F::cast_from(M05X2C_PARA_C2_F64),
        F::cast_from(M05X2C_PARA_C3_F64),
        F::cast_from(M05X2C_PARA_C4_F64),
        &chi_b2,
        &dsigma_b,
        &mut m05_para_b,
        n,
    );

    let mut ec_ab = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_anti, &m05_anti, &mut ec_ab, n);
    let mut ec_aa = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_a, &m05_para_a, &mut ec_aa, n);
    let mut ec_bb = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_b, &m05_para_b, &mut ec_bb, n);

    let mut sum_ab_aa = Array::<F>::new(size);
    ctaylor_add::<F>(&ec_ab, &ec_aa, &mut sum_ab_aa, n);
    ctaylor_add::<F>(&sum_ab_aa, &ec_bb, out, n);
}
