//! XC_M06C — M06 correlation functional. MGGA-04. Largest M06 body (109 LOC).
//!
//! # Source
//! - `xcfun-master/src/functionals/m06c.cpp:20-63`
//!
//! # Formula (port of `m06c.cpp:20-63`):
//! ```cpp
//! chi_a2 = chi2(d.a, d.gaa); chi_b2 = chi2(d.b, d.gbb);
//! zet_a  = zet(d.a, d.taua); zet_b  = zet(d.b, d.taub);
//! Dsigma_a = Dsigma(d.a, d.gaa, d.taua);
//! Dsigma_b = Dsigma(d.b, d.gbb, d.taub);
//!
//! Ec_ab = ueg_c_anti(d) * m06_c_anti(c_anti, d_anti, chi_a2, zet_a, chi_b2, zet_b);
//! Ec_aa = ueg_c_para(d.a) * m06_c_para(c_para, d_para, chi_a2, zet_a, Dsigma_a);
//! Ec_bb = ueg_c_para(d.b) * m06_c_para(c_para, d_para, chi_b2, zet_b, Dsigma_b);
//! return Ec_ab + Ec_aa + Ec_bb;
//! ```
//!
//! ACCURACY NOTE: the 12-coefficient `fw` polynomial is NOT used here — that
//! lives in the exchange functional. The correlation uses 5-coef `g` + 6-coef
//! `h` per spin channel via `m06_c_anti` / `m06_c_para`.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::m0x_like;

// Anti-parallel param_c[5] from m06c.cpp:29-30.
const M06C_ANTI_C0_F64: f64 = 3.741_539e0_f64;
const M06C_ANTI_C1_F64: f64 = 2.187_098e2_f64;
const M06C_ANTI_C2_F64: f64 = -4.531_252e2_f64;
const M06C_ANTI_C3_F64: f64 = 2.936_479e2_f64;
const M06C_ANTI_C4_F64: f64 = -6.287_470e1_f64;

// Anti-parallel param_d[6] from m06c.cpp:31-36.
const M06C_ANTI_D0_F64: f64 = -2.741_539e0_f64;
const M06C_ANTI_D1_F64: f64 = -6.720_113e-1_f64;
const M06C_ANTI_D2_F64: f64 = -7.932_688e-2_f64;
const M06C_ANTI_D3_F64: f64 = 1.918_681e-3_f64;
const M06C_ANTI_D4_F64: f64 = -2.032_902e-3_f64;
const M06C_ANTI_D5_F64: f64 = 0.0_f64;

// Parallel param_c[5] from m06c.cpp:39-40.
const M06C_PARA_C0_F64: f64 = 5.094_055e-1_f64;
const M06C_PARA_C1_F64: f64 = -1.491_085e0_f64;
const M06C_PARA_C2_F64: f64 = 1.723_922e1_f64;
const M06C_PARA_C3_F64: f64 = -3.859_018e1_f64;
const M06C_PARA_C4_F64: f64 = 2.845_044e1_f64;

// Parallel param_d[6] from m06c.cpp:41-46.
const M06C_PARA_D0_F64: f64 = 4.905_945e-1_f64;
const M06C_PARA_D1_F64: f64 = -1.437_348e-1_f64;
const M06C_PARA_D2_F64: f64 = 2.357_824e-1_f64;
const M06C_PARA_D3_F64: f64 = 1.871_015e-3_f64;
const M06C_PARA_D4_F64: f64 = -3.788_963e-3_f64;
const M06C_PARA_D5_F64: f64 = 0.0_f64;

#[cube]
pub fn m06c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // chi_a2, chi_b2
    let mut chi_a2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.a, &d.gaa, &mut chi_a2, n);
    let mut chi_b2 = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(&d.b, &d.gbb, &mut chi_b2, n);

    // zet_a, zet_b
    let mut zet_a = Array::<F>::new(size);
    m0x_like::m0x_zet::<F>(&d.a, &d.taua, &mut zet_a, n);
    let mut zet_b = Array::<F>::new(size);
    m0x_like::m0x_zet::<F>(&d.b, &d.taub, &mut zet_b, n);

    // Dsigma_a, Dsigma_b
    let mut dsigma_a = Array::<F>::new(size);
    m0x_like::m0x_Dsigma::<F>(&d.a, &d.gaa, &d.taua, &mut dsigma_a, n);
    let mut dsigma_b = Array::<F>::new(size);
    m0x_like::m0x_Dsigma::<F>(&d.b, &d.gbb, &d.taub, &mut dsigma_b, n);

    // ueg_c_anti, ueg_c_para
    let mut ueg_anti = Array::<F>::new(size);
    m0x_like::ueg_c_anti::<F>(d, &mut ueg_anti, n);
    let mut ueg_para_a = Array::<F>::new(size);
    m0x_like::ueg_c_para::<F>(&d.a, &mut ueg_para_a, n);
    let mut ueg_para_b = Array::<F>::new(size);
    m0x_like::ueg_c_para::<F>(&d.b, &mut ueg_para_b, n);

    // m06_c_anti(c_anti, d_anti, chi_a2, zet_a, chi_b2, zet_b)
    let mut m06_anti = Array::<F>::new(size);
    m0x_like::m06_c_anti::<F>(
        F::cast_from(M06C_ANTI_C0_F64),
        F::cast_from(M06C_ANTI_C1_F64),
        F::cast_from(M06C_ANTI_C2_F64),
        F::cast_from(M06C_ANTI_C3_F64),
        F::cast_from(M06C_ANTI_C4_F64),
        F::cast_from(M06C_ANTI_D0_F64),
        F::cast_from(M06C_ANTI_D1_F64),
        F::cast_from(M06C_ANTI_D2_F64),
        F::cast_from(M06C_ANTI_D3_F64),
        F::cast_from(M06C_ANTI_D4_F64),
        F::cast_from(M06C_ANTI_D5_F64),
        &chi_a2,
        &zet_a,
        &chi_b2,
        &zet_b,
        &mut m06_anti,
        n,
    );

    // m06_c_para(c_para, d_para, chi_a2, zet_a, Dsigma_a)
    let mut m06_para_a = Array::<F>::new(size);
    m0x_like::m06_c_para::<F>(
        F::cast_from(M06C_PARA_C0_F64),
        F::cast_from(M06C_PARA_C1_F64),
        F::cast_from(M06C_PARA_C2_F64),
        F::cast_from(M06C_PARA_C3_F64),
        F::cast_from(M06C_PARA_C4_F64),
        F::cast_from(M06C_PARA_D0_F64),
        F::cast_from(M06C_PARA_D1_F64),
        F::cast_from(M06C_PARA_D2_F64),
        F::cast_from(M06C_PARA_D3_F64),
        F::cast_from(M06C_PARA_D4_F64),
        F::cast_from(M06C_PARA_D5_F64),
        &chi_a2,
        &zet_a,
        &dsigma_a,
        &mut m06_para_a,
        n,
    );

    let mut m06_para_b = Array::<F>::new(size);
    m0x_like::m06_c_para::<F>(
        F::cast_from(M06C_PARA_C0_F64),
        F::cast_from(M06C_PARA_C1_F64),
        F::cast_from(M06C_PARA_C2_F64),
        F::cast_from(M06C_PARA_C3_F64),
        F::cast_from(M06C_PARA_C4_F64),
        F::cast_from(M06C_PARA_D0_F64),
        F::cast_from(M06C_PARA_D1_F64),
        F::cast_from(M06C_PARA_D2_F64),
        F::cast_from(M06C_PARA_D3_F64),
        F::cast_from(M06C_PARA_D4_F64),
        F::cast_from(M06C_PARA_D5_F64),
        &chi_b2,
        &zet_b,
        &dsigma_b,
        &mut m06_para_b,
        n,
    );

    // Ec_ab = ueg_anti · m06_anti
    let mut ec_ab = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_anti, &m06_anti, &mut ec_ab, n);

    // Ec_aa = ueg_para_a · m06_para_a
    let mut ec_aa = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_a, &m06_para_a, &mut ec_aa, n);

    // Ec_bb = ueg_para_b · m06_para_b
    let mut ec_bb = Array::<F>::new(size);
    ctaylor_mul::<F>(&ueg_para_b, &m06_para_b, &mut ec_bb, n);

    // out = Ec_ab + Ec_aa + Ec_bb (left-to-right per ACC-06)
    let mut sum_ab_aa = Array::<F>::new(size);
    ctaylor_add::<F>(&ec_ab, &ec_aa, &mut sum_ab_aa, n);
    ctaylor_add::<F>(&sum_ab_aa, &ec_bb, out, n);
}
