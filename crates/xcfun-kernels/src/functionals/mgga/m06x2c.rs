//! XC_M06X2C — M06-2X correlation functional. MGGA-04.
//!
//! # Source
//! - `xcfun-master/src/functionals/m06x2c.cpp:20-63`
//!
//! Same body shape as `m06c` with M06-2X-specific c/d parameter sets.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::m0x_like;

// Anti-parallel param_c[5] from m06x2c.cpp:29-30.
const M06X2C_ANTI_C0_F64: f64 = 8.833_596e-1_f64;
const M06X2C_ANTI_C1_F64: f64 = 3.357_972e1_f64;
const M06X2C_ANTI_C2_F64: f64 = -7.043_548e1_f64;
const M06X2C_ANTI_C3_F64: f64 = 4.978_271e1_f64;
const M06X2C_ANTI_C4_F64: f64 = -1.852_891e1_f64;

// Anti-parallel param_d[6] from m06x2c.cpp:31-36.
const M06X2C_ANTI_D0_F64: f64 = 1.166_404e-1_f64;
const M06X2C_ANTI_D1_F64: f64 = -9.120_847e-2_f64;
const M06X2C_ANTI_D2_F64: f64 = -6.726_189e-2_f64;
const M06X2C_ANTI_D3_F64: f64 = 6.720_580e-5_f64;
const M06X2C_ANTI_D4_F64: f64 = 8.448_011e-4_f64;
const M06X2C_ANTI_D5_F64: f64 = 0.0_f64;

// Parallel param_c[5] from m06x2c.cpp:39-40.
const M06X2C_PARA_C0_F64: f64 = 3.097_855e-1_f64;
const M06X2C_PARA_C1_F64: f64 = -5.528_642e0_f64;
const M06X2C_PARA_C2_F64: f64 = 1.347_420e1_f64;
const M06X2C_PARA_C3_F64: f64 = -3.213_623e1_f64;
const M06X2C_PARA_C4_F64: f64 = 2.846_742e1_f64;

// Parallel param_d[6] from m06x2c.cpp:41-46.
const M06X2C_PARA_D0_F64: f64 = 6.902_145e-1_f64;
const M06X2C_PARA_D1_F64: f64 = 9.847_204e-2_f64;
const M06X2C_PARA_D2_F64: f64 = 2.214_797e-1_f64;
const M06X2C_PARA_D3_F64: f64 = -1.968_264e-3_f64;
const M06X2C_PARA_D4_F64: f64 = -6.775_479e-3_f64;
const M06X2C_PARA_D5_F64: f64 = 0.0_f64;

#[cube]
pub fn m06x2c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut chi_a2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.a, &d.gaa, &mut chi_a2, n);
    let mut chi_b2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.b, &d.gbb, &mut chi_b2, n);

    let mut zet_a = Array::<F>::new(size);
    m0x_like::m0x_zet::<F>(&d.a, &d.taua, &mut zet_a, n);
    let mut zet_b = Array::<F>::new(size);
    m0x_like::m0x_zet::<F>(&d.b, &d.taub, &mut zet_b, n);

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

    let mut m06_anti = Array::<F>::new(size);
    m0x_like::m06_c_anti::<F>(
        F::cast_from(M06X2C_ANTI_C0_F64),
        F::cast_from(M06X2C_ANTI_C1_F64),
        F::cast_from(M06X2C_ANTI_C2_F64),
        F::cast_from(M06X2C_ANTI_C3_F64),
        F::cast_from(M06X2C_ANTI_C4_F64),
        F::cast_from(M06X2C_ANTI_D0_F64),
        F::cast_from(M06X2C_ANTI_D1_F64),
        F::cast_from(M06X2C_ANTI_D2_F64),
        F::cast_from(M06X2C_ANTI_D3_F64),
        F::cast_from(M06X2C_ANTI_D4_F64),
        F::cast_from(M06X2C_ANTI_D5_F64),
        &chi_a2, &zet_a, &chi_b2, &zet_b,
        &mut m06_anti,
        n,
    );

    let mut m06_para_a = Array::<F>::new(size);
    m0x_like::m06_c_para::<F>(
        F::cast_from(M06X2C_PARA_C0_F64),
        F::cast_from(M06X2C_PARA_C1_F64),
        F::cast_from(M06X2C_PARA_C2_F64),
        F::cast_from(M06X2C_PARA_C3_F64),
        F::cast_from(M06X2C_PARA_C4_F64),
        F::cast_from(M06X2C_PARA_D0_F64),
        F::cast_from(M06X2C_PARA_D1_F64),
        F::cast_from(M06X2C_PARA_D2_F64),
        F::cast_from(M06X2C_PARA_D3_F64),
        F::cast_from(M06X2C_PARA_D4_F64),
        F::cast_from(M06X2C_PARA_D5_F64),
        &chi_a2, &zet_a, &dsigma_a,
        &mut m06_para_a,
        n,
    );

    let mut m06_para_b = Array::<F>::new(size);
    m0x_like::m06_c_para::<F>(
        F::cast_from(M06X2C_PARA_C0_F64),
        F::cast_from(M06X2C_PARA_C1_F64),
        F::cast_from(M06X2C_PARA_C2_F64),
        F::cast_from(M06X2C_PARA_C3_F64),
        F::cast_from(M06X2C_PARA_C4_F64),
        F::cast_from(M06X2C_PARA_D0_F64),
        F::cast_from(M06X2C_PARA_D1_F64),
        F::cast_from(M06X2C_PARA_D2_F64),
        F::cast_from(M06X2C_PARA_D3_F64),
        F::cast_from(M06X2C_PARA_D4_F64),
        F::cast_from(M06X2C_PARA_D5_F64),
        &chi_b2, &zet_b, &dsigma_b,
        &mut m06_para_b,
        n,
    );

    let mut ec_ab = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_anti, &m06_anti, &mut ec_ab, n);
    let mut ec_aa = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_a, &m06_para_a, &mut ec_aa, n);
    let mut ec_bb = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_b, &m06_para_b, &mut ec_bb, n);

    let mut sum_ab_aa = Array::<F>::new(size);
    ctaylor_add::<F>(&ec_ab, &ec_aa, &mut sum_ab_aa, n);
    ctaylor_add::<F>(&sum_ab_aa, &ec_bb, out, n);
}
