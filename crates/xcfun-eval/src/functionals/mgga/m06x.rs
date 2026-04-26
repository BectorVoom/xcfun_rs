//! XC_M06X — M06 exchange functional. MGGA-03.
//!
//! # Source
//! - `xcfun-master/src/functionals/m06x.cpp:21-54`
//!
//! # Formula (port of `m06x.cpp:21-54`):
//! ```cpp
//! chia2 = chi2(d.a, d.gaa); chib2 = chi2(d.b, d.gbb);
//! return ((pbex::energy_pbe_ab(R_pbe, d.a, d.gaa) * fw(param_a, d.a, d.taua) +
//!          lsda_x(d.a) * h(param_d, alpha_x, chia2, zet(d.a, d.taua))) +
//!         (pbex::energy_pbe_ab(R_pbe, d.b, d.gbb) * fw(param_a, d.b, d.taub) +
//!          lsda_x(d.b) * h(param_d, alpha_x, chib2, zet(d.b, d.taub))));
//! ```
//!
//! `param_a[12]` and `param_d[6]` from `m06x.cpp:28-45`.
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

// param_a[12] from m06x.cpp:28-39.
const M06X_A0_F64: f64 = 5.877_943e-1_f64;
const M06X_A1_F64: f64 = -1.371_776e-1_f64;
const M06X_A2_F64: f64 = 2.682_367e-1_f64;
const M06X_A3_F64: f64 = -2.515_898e0_f64;
const M06X_A4_F64: f64 = -2.978_892e0_f64;
const M06X_A5_F64: f64 = 8.710_679e0_f64;
const M06X_A6_F64: f64 = 1.688_195e1_f64;
const M06X_A7_F64: f64 = -4.489_724e0_f64;
const M06X_A8_F64: f64 = -3.299_983e1_f64;
const M06X_A9_F64: f64 = -1.449_050e1_f64;
const M06X_A10_F64: f64 = 2.043_747e1_f64;
const M06X_A11_F64: f64 = 1.256_504e1_f64;

// param_d[6] from m06x.cpp:40-45.
const M06X_D0_F64: f64 = 1.422_057e-1_f64;
const M06X_D1_F64: f64 = 7.370_319e-4_f64;
const M06X_D2_F64: f64 = -1.601_373e-2_f64;
const M06X_D3_F64: f64 = 0.0_f64;
const M06X_D4_F64: f64 = 0.0_f64;
const M06X_D5_F64: f64 = 0.0_f64;

/// Compute one spin contribution: `pbex_term + lsda·h_term`.
#[cube]
fn m06x_spin<F: Float>(
    rho: &Array<F>,
    rho_43: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // chi² = pw91_like::chi2(rho, grad2)
    let mut chi2_arr = Array::<F>::new(size);
    m0x_like::m0x_chi2::<F>(rho, grad2, &mut chi2_arr, n);

    // zet = m0x_zet(rho, tau)
    let mut zet_arr = Array::<F>::new(size);
    m0x_like::m0x_zet::<F>(rho, tau, &mut zet_arr, n);

    // pbex_part = pbex::energy_pbe_ab(R_pbe, rho_43, rho, grad2)
    let mut pbex_part = Array::<F>::new(size);
    pbex::energy_pbe_ab::<F>(F::cast_from(R_PBE_F64), rho_43, rho, grad2, &mut pbex_part, n);

    // fw_part = m0x_fw(param_a, rho, tau)
    let mut fw_part = Array::<F>::new(size);
    m0x_like::m0x_fw::<F>(
        F::cast_from(M06X_A0_F64),
        F::cast_from(M06X_A1_F64),
        F::cast_from(M06X_A2_F64),
        F::cast_from(M06X_A3_F64),
        F::cast_from(M06X_A4_F64),
        F::cast_from(M06X_A5_F64),
        F::cast_from(M06X_A6_F64),
        F::cast_from(M06X_A7_F64),
        F::cast_from(M06X_A8_F64),
        F::cast_from(M06X_A9_F64),
        F::cast_from(M06X_A10_F64),
        F::cast_from(M06X_A11_F64),
        rho,
        tau,
        &mut fw_part,
        n,
    );

    // pbex_term = pbex_part · fw_part
    let mut pbex_term = Array::<F>::new(size);
    ctaylor_mul::<F>(&pbex_part, &fw_part, &mut pbex_term, n);

    // lsda = lsda_x(rho)
    let mut lsda = Array::<F>::new(size);
    m0x_like::m0x_lsda_x::<F>(rho, &mut lsda, n);

    // h_part = h(param_d, alpha_x, chi2, zet)
    let mut h_part = Array::<F>::new(size);
    m0x_like::m0x_h::<F>(
        F::cast_from(M06X_D0_F64),
        F::cast_from(M06X_D1_F64),
        F::cast_from(M06X_D2_F64),
        F::cast_from(M06X_D3_F64),
        F::cast_from(M06X_D4_F64),
        F::cast_from(M06X_D5_F64),
        F::cast_from(M0X_ALPHA_X_F64),
        &chi2_arr,
        &zet_arr,
        &mut h_part,
        n,
    );

    // lsda_term = lsda · h_part
    let mut lsda_term = Array::<F>::new(size);
    ctaylor_mul::<F>(&lsda, &h_part, &mut lsda_term, n);

    // out = pbex_term + lsda_term
    ctaylor_add::<F>(&pbex_term, &lsda_term, out, n);
}

#[cube]
pub fn m06x_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut term_a = Array::<F>::new(size);
    m06x_spin::<F>(&d.a, &d.a_43, &d.gaa, &d.taua, &mut term_a, n);

    let mut term_b = Array::<F>::new(size);
    m06x_spin::<F>(&d.b, &d.b_43, &d.gbb, &d.taub, &mut term_b, n);

    ctaylor_add::<F>(&term_a, &term_b, out, n);
}
