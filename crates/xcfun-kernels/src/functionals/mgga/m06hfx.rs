//! XC_M06HFX — M06-HF exchange functional. MGGA-03.
//!
//! # Source
//! - `xcfun-master/src/functionals/m06hfx.cpp:21-54`
//!
//! Same body shape as `m06x` with M06-HF-specific param_a/param_d.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pbex;
use crate::functionals::mgga::shared::constants::M0X_ALPHA_X_F64;
use crate::functionals::mgga::shared::m0x_like;

const R_PBE_F64: f64 = 0.804_f64;

// param_a[12] from m06hfx.cpp:28-39.
const M06HFX_A0_F64: f64 = 1.179_732e-1_f64;
const M06HFX_A1_F64: f64 = -1.066_708e0_f64;
const M06HFX_A2_F64: f64 = -1.462_405e-1_f64;
const M06HFX_A3_F64: f64 = 7.481_848e0_f64;
const M06HFX_A4_F64: f64 = 3.776_679e0_f64;
const M06HFX_A5_F64: f64 = -4.436_118e1_f64;
const M06HFX_A6_F64: f64 = -1.830_962e1_f64;
const M06HFX_A7_F64: f64 = 1.003_903e2_f64;
const M06HFX_A8_F64: f64 = 3.864_360e1_f64;
const M06HFX_A9_F64: f64 = -9.806_018e1_f64;
const M06HFX_A10_F64: f64 = -2.557_716e1_f64;
const M06HFX_A11_F64: f64 = 3.590_404e1_f64;

// param_d[6] from m06hfx.cpp:40-45.
const M06HFX_D0_F64: f64 = -1.179_732e-1_f64;
const M06HFX_D1_F64: f64 = -2.500_000e-3_f64;
const M06HFX_D2_F64: f64 = -1.180_056e-2_f64;
const M06HFX_D3_F64: f64 = 0.0_f64;
const M06HFX_D4_F64: f64 = 0.0_f64;
const M06HFX_D5_F64: f64 = 0.0_f64;

#[cube]
fn m06hfx_spin<F: Float>(
    rho: &Array<F>,
    rho_43: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut chi2_arr = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(rho, grad2, &mut chi2_arr, n);
    let mut zet_arr = Array::<F>::new(size);
    m0x_like::m0x_zet::<F>(rho, tau, &mut zet_arr, n);

    let mut pbex_part = Array::<F>::new(size);
    pbex::energy_pbe_ab::<F>(F::cast_from(R_PBE_F64), rho_43, rho, grad2, &mut pbex_part, n);

    let mut fw_part = Array::<F>::new(size);
    m0x_like::m0x_fw::<F>(
        F::cast_from(M06HFX_A0_F64),
        F::cast_from(M06HFX_A1_F64),
        F::cast_from(M06HFX_A2_F64),
        F::cast_from(M06HFX_A3_F64),
        F::cast_from(M06HFX_A4_F64),
        F::cast_from(M06HFX_A5_F64),
        F::cast_from(M06HFX_A6_F64),
        F::cast_from(M06HFX_A7_F64),
        F::cast_from(M06HFX_A8_F64),
        F::cast_from(M06HFX_A9_F64),
        F::cast_from(M06HFX_A10_F64),
        F::cast_from(M06HFX_A11_F64),
        rho,
        tau,
        &mut fw_part,
        n,
    );

    let mut pbex_term = Array::<F>::new(size);
    ctaylor_mul::<F>(&pbex_part, &fw_part, &mut pbex_term, n);

    let mut lsda = Array::<F>::new(size);
    m0x_like::m0x_lsda_x::<F>(rho, &mut lsda, n);

    let mut h_part = Array::<F>::new(size);
    m0x_like::m0x_h::<F>(
        F::cast_from(M06HFX_D0_F64),
        F::cast_from(M06HFX_D1_F64),
        F::cast_from(M06HFX_D2_F64),
        F::cast_from(M06HFX_D3_F64),
        F::cast_from(M06HFX_D4_F64),
        F::cast_from(M06HFX_D5_F64),
        F::cast_from(M0X_ALPHA_X_F64),
        &chi2_arr,
        &zet_arr,
        &mut h_part,
        n,
    );

    let mut lsda_term = Array::<F>::new(size);
    ctaylor_mul::<F>(&lsda, &h_part, &mut lsda_term, n);

    ctaylor_add::<F>(&pbex_term, &lsda_term, out, n);
}

#[cube]
pub fn m06hfx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut term_a = Array::<F>::new(size);
    m06hfx_spin::<F>(&d.a, &d.a_43, &d.gaa, &d.taua, &mut term_a, n);
    let mut term_b = Array::<F>::new(size);
    m06hfx_spin::<F>(&d.b, &d.b_43, &d.gbb, &d.taub, &mut term_b, n);
    ctaylor_add::<F>(&term_a, &term_b, out, n);
}
