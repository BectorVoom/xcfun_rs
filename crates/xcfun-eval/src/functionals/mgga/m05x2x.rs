//! XC_M05X2X — M05-2X exchange functional. MGGA-03.
//!
//! # Source
//! - `xcfun-master/src/functionals/m05x2x.cpp:21-39`
//!
//! # Formula (port of `m05x2x.cpp:21-39`):
//! ```cpp
//! return (pbex::energy_pbe_ab(pbex::R_pbe, d.a, d.gaa) * fw(param_a, d.a, d.taua)
//!       + pbex::energy_pbe_ab(pbex::R_pbe, d.b, d.gbb) * fw(param_a, d.b, d.taub));
//! ```
//!
//! # Parameters (12-coefficient `param_a` from `m05x2x.cpp:24-35`).
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pbex;
use crate::functionals::mgga::shared::m0x_like;

const R_PBE_F64: f64 = 0.804_f64;

// M05-2X exchange `param_a[12]` (verbatim from m05x2x.cpp:24-35).
const M05X2X_A0_F64: f64 = 1.000_000e0_f64;
const M05X2X_A1_F64: f64 = -5.683_300e-1_f64;
const M05X2X_A2_F64: f64 = -1.300_570e0_f64;
const M05X2X_A3_F64: f64 = 5.500_700e0_f64;
const M05X2X_A4_F64: f64 = 9.064_020e0_f64;
const M05X2X_A5_F64: f64 = -3.221_075e1_f64;
const M05X2X_A6_F64: f64 = -2.373_298e1_f64;
const M05X2X_A7_F64: f64 = 7.022_996e1_f64;
const M05X2X_A8_F64: f64 = 2.988_614e1_f64;
const M05X2X_A9_F64: f64 = -6.025_778e1_f64;
const M05X2X_A10_F64: f64 = -1.322_205e1_f64;
const M05X2X_A11_F64: f64 = 1.523_694e1_f64;

#[cube]
pub fn m05x2x_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let r = F::cast_from(R_PBE_F64);

    let mut pbe_a = Array::<F>::new(size);
    pbex::energy_pbe_ab::<F>(r, &d.a_43, &d.a, &d.gaa, &mut pbe_a, n);

    let mut fw_a = Array::<F>::new(size);
    m0x_like::m0x_fw::<F>(
        F::cast_from(M05X2X_A0_F64),
        F::cast_from(M05X2X_A1_F64),
        F::cast_from(M05X2X_A2_F64),
        F::cast_from(M05X2X_A3_F64),
        F::cast_from(M05X2X_A4_F64),
        F::cast_from(M05X2X_A5_F64),
        F::cast_from(M05X2X_A6_F64),
        F::cast_from(M05X2X_A7_F64),
        F::cast_from(M05X2X_A8_F64),
        F::cast_from(M05X2X_A9_F64),
        F::cast_from(M05X2X_A10_F64),
        F::cast_from(M05X2X_A11_F64),
        &d.a,
        &d.taua,
        &mut fw_a,
        n,
    );

    let mut term_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&pbe_a, &fw_a, &mut term_a, n);

    let mut pbe_b = Array::<F>::new(size);
    pbex::energy_pbe_ab::<F>(r, &d.b_43, &d.b, &d.gbb, &mut pbe_b, n);

    let mut fw_b = Array::<F>::new(size);
    m0x_like::m0x_fw::<F>(
        F::cast_from(M05X2X_A0_F64),
        F::cast_from(M05X2X_A1_F64),
        F::cast_from(M05X2X_A2_F64),
        F::cast_from(M05X2X_A3_F64),
        F::cast_from(M05X2X_A4_F64),
        F::cast_from(M05X2X_A5_F64),
        F::cast_from(M05X2X_A6_F64),
        F::cast_from(M05X2X_A7_F64),
        F::cast_from(M05X2X_A8_F64),
        F::cast_from(M05X2X_A9_F64),
        F::cast_from(M05X2X_A10_F64),
        F::cast_from(M05X2X_A11_F64),
        &d.b,
        &d.taub,
        &mut fw_b,
        n,
    );

    let mut term_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&pbe_b, &fw_b, &mut term_b, n);

    ctaylor_add::<F>(&term_a, &term_b, out, n);
}
