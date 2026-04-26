//! XC_M05C — M05 correlation functional. MGGA-04.
//!
//! # Source
//! - `xcfun-master/src/functionals/m05c.cpp:20-48`
//!
//! # Formula (port of `m05c.cpp:20-48`):
//! ```cpp
//! chi_a2 = chi2(d.a, d.gaa); chi_b2 = chi2(d.b, d.gbb);
//! zet_a  = zet(d.a, d.taua); zet_b  = zet(d.b, d.taub);   // zet unused for m05_c_para
//! Dsigma_a = Dsigma(d.a, d.gaa, d.taua);
//! Dsigma_b = Dsigma(d.b, d.gbb, d.taub);
//!
//! Ec_ab = ueg_c_anti(d) * m05_c_anti(c_anti, chi_a2, chi_b2);
//! Ec_aa = ueg_c_para(d.a) * m05_c_para(c_para, chi_a2, zet_a, Dsigma_a);
//! Ec_bb = ueg_c_para(d.b) * m05_c_para(c_para, chi_b2, zet_b, Dsigma_b);
//! return Ec_ab + Ec_aa + Ec_bb;
//! ```
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::m0x_like;

// Anti-parallel param_c[5] from m05c.cpp:29-30.
const M05C_ANTI_C0_F64: f64 = 1.000_000e0_f64;
const M05C_ANTI_C1_F64: f64 = 3.785_690e0_f64;
const M05C_ANTI_C2_F64: f64 = -1.415_261e1_f64;
const M05C_ANTI_C3_F64: f64 = -7.465_890e0_f64;
const M05C_ANTI_C4_F64: f64 = 1.794_491e1_f64;

// Parallel param_c[5] from m05c.cpp:33-34.
const M05C_PARA_C0_F64: f64 = 1.000_000e0_f64;
const M05C_PARA_C1_F64: f64 = 3.773_440e0_f64;
const M05C_PARA_C2_F64: f64 = -2.604_463e1_f64;
const M05C_PARA_C3_F64: f64 = 3.069_913e1_f64;
const M05C_PARA_C4_F64: f64 = -9.226_950e0_f64;

#[cube]
pub fn m05c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // chi_a2, chi_b2
    let mut chi_a2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.a, &d.gaa, &mut chi_a2, n);
    let mut chi_b2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.b, &d.gbb, &mut chi_b2, n);

    // Dsigma_a, Dsigma_b
    let mut dsigma_a = Array::<F>::new(size);
    m0x_like::m0x_Dsigma::<F>(&d.a, &d.gaa, &d.taua, &mut dsigma_a, n);
    let mut dsigma_b = Array::<F>::new(size);
    m0x_like::m0x_Dsigma::<F>(&d.b, &d.gbb, &d.taub, &mut dsigma_b, n);

    // ueg_c_anti(d), ueg_c_para(d.a), ueg_c_para(d.b)
    let mut ueg_anti = Array::<F>::new(size);
    m0x_like::ueg_c_anti::<F>(d, &mut ueg_anti, n);
    let mut ueg_para_a = Array::<F>::new(size);
    m0x_like::ueg_c_para::<F>(&d.a, &mut ueg_para_a, n);
    let mut ueg_para_b = Array::<F>::new(size);
    m0x_like::ueg_c_para::<F>(&d.b, &mut ueg_para_b, n);

    // m05_c_anti(c_anti, chi_a2, chi_b2)
    let mut m05_anti = Array::<F>::new(size);
    m0x_like::m05_c_anti::<F>(
        F::cast_from(M05C_ANTI_C0_F64),
        F::cast_from(M05C_ANTI_C1_F64),
        F::cast_from(M05C_ANTI_C2_F64),
        F::cast_from(M05C_ANTI_C3_F64),
        F::cast_from(M05C_ANTI_C4_F64),
        &chi_a2,
        &chi_b2,
        &mut m05_anti,
        n,
    );

    // m05_c_para(c_para, chi_a2, Dsigma_a) — zet unused
    let mut m05_para_a = Array::<F>::new(size);
    m0x_like::m05_c_para::<F>(
        F::cast_from(M05C_PARA_C0_F64),
        F::cast_from(M05C_PARA_C1_F64),
        F::cast_from(M05C_PARA_C2_F64),
        F::cast_from(M05C_PARA_C3_F64),
        F::cast_from(M05C_PARA_C4_F64),
        &chi_a2,
        &dsigma_a,
        &mut m05_para_a,
        n,
    );

    let mut m05_para_b = Array::<F>::new(size);
    m0x_like::m05_c_para::<F>(
        F::cast_from(M05C_PARA_C0_F64),
        F::cast_from(M05C_PARA_C1_F64),
        F::cast_from(M05C_PARA_C2_F64),
        F::cast_from(M05C_PARA_C3_F64),
        F::cast_from(M05C_PARA_C4_F64),
        &chi_b2,
        &dsigma_b,
        &mut m05_para_b,
        n,
    );

    // Ec_ab = ueg_c_anti · m05_c_anti
    let mut ec_ab = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_anti, &m05_anti, &mut ec_ab, n);

    // Ec_aa = ueg_c_para(d.a) · m05_para_a
    let mut ec_aa = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_a, &m05_para_a, &mut ec_aa, n);

    // Ec_bb = ueg_c_para(d.b) · m05_para_b
    let mut ec_bb = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_b, &m05_para_b, &mut ec_bb, n);

    // out = Ec_ab + Ec_aa + Ec_bb (left-to-right per ACC-06)
    let mut sum_ab_aa = Array::<F>::new(size);
    ctaylor_add::<F>(&ec_ab, &ec_aa, &mut sum_ab_aa, n);
    ctaylor_add::<F>(&sum_ab_aa, &ec_bb, out, n);
}
