//! XC_M06HFC — M06-HF correlation functional. MGGA-04.
//!
//! # Source
//! - `xcfun-master/src/functionals/m06hfc.cpp:20-63`
//!
//! Same body shape as `m06c` with M06-HF-specific c/d parameter sets.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::m0x_like;

// Anti-parallel param_c[5] from m06hfc.cpp:29-30.
const M06HFC_ANTI_C0_F64: f64 = 1.674_634e0_f64;
const M06HFC_ANTI_C1_F64: f64 = 5.732_017e1_f64;
const M06HFC_ANTI_C2_F64: f64 = 5.955_416e1_f64;
const M06HFC_ANTI_C3_F64: f64 = -2.311_007e2_f64;
const M06HFC_ANTI_C4_F64: f64 = 1.255_199e2_f64;

// Anti-parallel param_d[6] from m06hfc.cpp:31-36.
const M06HFC_ANTI_D0_F64: f64 = -6.746_338e-1_f64;
const M06HFC_ANTI_D1_F64: f64 = -1.534_002e-1_f64;
const M06HFC_ANTI_D2_F64: f64 = -9.021_521e-2_f64;
const M06HFC_ANTI_D3_F64: f64 = -1.292_037e-3_f64;
const M06HFC_ANTI_D4_F64: f64 = -2.352_983e-4_f64;
const M06HFC_ANTI_D5_F64: f64 = 0.0_f64;

// Parallel param_c[5] from m06hfc.cpp:39-40.
const M06HFC_PARA_C0_F64: f64 = 1.023_254e-1_f64;
const M06HFC_PARA_C1_F64: f64 = -2.453_783e0_f64;
const M06HFC_PARA_C2_F64: f64 = 2.913_180e1_f64;
const M06HFC_PARA_C3_F64: f64 = -3.494_358e1_f64;
const M06HFC_PARA_C4_F64: f64 = 2.315_955e1_f64;

// Parallel param_d[6] from m06hfc.cpp:41-46.
const M06HFC_PARA_D0_F64: f64 = 8.976_746e-1_f64;
const M06HFC_PARA_D1_F64: f64 = -2.345_830e-1_f64;
const M06HFC_PARA_D2_F64: f64 = 2.368_173e-1_f64;
const M06HFC_PARA_D3_F64: f64 = -9.913_890e-4_f64;
const M06HFC_PARA_D4_F64: f64 = -1.146_165e-2_f64;
const M06HFC_PARA_D5_F64: f64 = 0.0_f64;

#[cube]
pub fn m06hfc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
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
        F::cast_from(M06HFC_ANTI_C0_F64),
        F::cast_from(M06HFC_ANTI_C1_F64),
        F::cast_from(M06HFC_ANTI_C2_F64),
        F::cast_from(M06HFC_ANTI_C3_F64),
        F::cast_from(M06HFC_ANTI_C4_F64),
        F::cast_from(M06HFC_ANTI_D0_F64),
        F::cast_from(M06HFC_ANTI_D1_F64),
        F::cast_from(M06HFC_ANTI_D2_F64),
        F::cast_from(M06HFC_ANTI_D3_F64),
        F::cast_from(M06HFC_ANTI_D4_F64),
        F::cast_from(M06HFC_ANTI_D5_F64),
        &chi_a2, &zet_a, &chi_b2, &zet_b,
        &mut m06_anti,
        n,
    );

    let mut m06_para_a = Array::<F>::new(size);
    m0x_like::m06_c_para::<F>(
        F::cast_from(M06HFC_PARA_C0_F64),
        F::cast_from(M06HFC_PARA_C1_F64),
        F::cast_from(M06HFC_PARA_C2_F64),
        F::cast_from(M06HFC_PARA_C3_F64),
        F::cast_from(M06HFC_PARA_C4_F64),
        F::cast_from(M06HFC_PARA_D0_F64),
        F::cast_from(M06HFC_PARA_D1_F64),
        F::cast_from(M06HFC_PARA_D2_F64),
        F::cast_from(M06HFC_PARA_D3_F64),
        F::cast_from(M06HFC_PARA_D4_F64),
        F::cast_from(M06HFC_PARA_D5_F64),
        &chi_a2, &zet_a, &dsigma_a,
        &mut m06_para_a,
        n,
    );

    let mut m06_para_b = Array::<F>::new(size);
    m0x_like::m06_c_para::<F>(
        F::cast_from(M06HFC_PARA_C0_F64),
        F::cast_from(M06HFC_PARA_C1_F64),
        F::cast_from(M06HFC_PARA_C2_F64),
        F::cast_from(M06HFC_PARA_C3_F64),
        F::cast_from(M06HFC_PARA_C4_F64),
        F::cast_from(M06HFC_PARA_D0_F64),
        F::cast_from(M06HFC_PARA_D1_F64),
        F::cast_from(M06HFC_PARA_D2_F64),
        F::cast_from(M06HFC_PARA_D3_F64),
        F::cast_from(M06HFC_PARA_D4_F64),
        F::cast_from(M06HFC_PARA_D5_F64),
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
