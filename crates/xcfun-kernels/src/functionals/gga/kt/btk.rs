//! XC_BTK — Borgoo-Tozer kinetic energy functional. **GGA-10 (BTK, id=58).**
//!
//! # Source
//! - `xcfun-master/src/functionals/btk.cpp:17-27`
//!
//! # Formula
//! ```cpp
//! qav   = 0.3434125
//! beta  = 1.990328
//! fudge = 1e-24                                 // upstream-prescribed, NOT f64::EPSILON
//!
//! btk_alpha(na, gaa) =
//!   beta · pow(na, 5/3) · pow(fudge + gaa, 0.5·qav) / pow(na, 4/3·qav)
//!
//! btk(d) = 0.5 · (btk_alpha(2·d.a, 4·d.gaa) + btk_alpha(2·d.b, 4·d.gbb))
//! ```
//!
//! Note the explicit `na = 2·d.a` / `gaa = 4·d.gaa` rescaling; this is the
//! upstream "single-spin to total-density rescaling" pattern used by several
//! kinetic GGAs (TFK and TW use a similar scheme; see Phase-2 `tw.rs`).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{BTK_BETA_F64, BTK_FUDGE_F64, BTK_QAV_F64};

/// Per-spin BTK contribution. `na = 2·d.a` and `gaa_in = 4·d.gaa` are
/// caller-rescaled (see `btk.cpp:26`).
#[cube]
fn btk_alpha<F: Float>(
    na: &Array<F>,
    gaa_in: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // na53 = pow(na, 5/3).
    let mut na53 = Array::<F>::new(size);
    ctaylor_pow::<F>(na, F::cast_from(5.0_f64 / 3.0_f64), &mut na53, n);

    // beta_na53 = beta · na53.
    let mut beta_na53 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&na53, F::cast_from(BTK_BETA_F64), &mut beta_na53, n);

    // fudge_plus_gaa = fudge + gaa_in (CNST-bump on a copy).
    // BTK_FUDGE_F64 = 1e-24, NOT f64::EPSILON — upstream-prescribed (btk.cpp:20).
    let mut fudge_plus_gaa = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        fudge_plus_gaa[i] = gaa_in[i];
    }
    fudge_plus_gaa[0] = fudge_plus_gaa[0] + F::cast_from(BTK_FUDGE_F64);

    // pow_g = pow(fudge + gaa, 0.5·qav).
    let half_qav = 0.5_f64 * BTK_QAV_F64;
    let mut pow_g = Array::<F>::new(size);
    ctaylor_pow::<F>(&fudge_plus_gaa, F::cast_from(half_qav), &mut pow_g, n);

    // numer = beta · na53 · pow_g.
    let mut numer = Array::<F>::new(size);
    ctaylor_mul::<F>(&beta_na53, &pow_g, &mut numer, n);

    // denom = pow(na, 4/3 · qav).
    let denom_exp = (4.0_f64 / 3.0_f64) * BTK_QAV_F64;
    let mut denom = Array::<F>::new(size);
    ctaylor_pow::<F>(na, F::cast_from(denom_exp), &mut denom, n);

    // out = numer / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    ctaylor_mul::<F>(&numer, &inv_denom, out, n);
}

/// XC_BTK kernel. 1:1 port of `btk.cpp:25-27`.
#[cube]
pub fn btk_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // na_alpha = 2 · d.a;   gaa_in_alpha = 4 · d.gaa.
    let mut na_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.a, F::new(2.0), &mut na_a, n);
    let mut gaa_in_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.gaa, F::new(4.0), &mut gaa_in_a, n);

    let mut e_alpha = Array::<F>::new(size);
    btk_alpha::<F>(&na_a, &gaa_in_a, &mut e_alpha, n);

    // na_beta = 2 · d.b;   gaa_in_beta = 4 · d.gbb.
    let mut na_b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.b, F::new(2.0), &mut na_b, n);
    let mut gaa_in_b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.gbb, F::new(4.0), &mut gaa_in_b, n);

    let mut e_beta = Array::<F>::new(size);
    btk_alpha::<F>(&na_b, &gaa_in_b, &mut e_beta, n);

    // sum = e_alpha + e_beta;  out = 0.5 · sum.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&e_alpha, &e_beta, &mut sum, n);
    ctaylor_scalar_mul::<F>(&sum, F::new(0.5), out, n);
}
