//! XC_KTX — Keal-Tozer GGA exchange correction. **GGA-10 (KTX, id=23).**
//!
//! # Source
//! - `xcfun-master/src/functionals/ktx.cpp:18-24`
//!
//! # Formula
//! ```cpp
//! const parameter DELTA = 0.1;
//! num ea = d.gaa / (DELTA + pow(d.a, 4.0 / 3.0));
//! num eb = d.gbb / (DELTA + pow(d.b, 4.0 / 3.0));
//! return ea + eb;
//! ```
//!
//! Use `d.a_43` / `d.b_43` directly — both equal `pow(d.a, 4/3)` /
//! `pow(d.b, 4/3)` per `build_xc_a_b_gaa_gab_gbb` (Plan 02-05). The C++ source
//! recomputes `pow` inline; algorithmic-identity allows substituting the
//! precomputed value because the operation `pow(a, 4/3)` is identical.
//!
//! # Preconditions (XC_A_B_GAA_GAB_GBB Vars arm)
//! - `d.a`, `d.b`, `d.gaa`, `d.gbb`, `d.a_43`, `d.b_43` populated.
//! - `a, b > 0` (post-regularize); denominator `0.1 + a_43` always positive.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::KTX_DELTA_F64;

/// Per-spin KTX correction: `gaa / (DELTA + a^(4/3))`.
#[cube]
fn ktx_alpha<F: Float>(
    grad2: &Array<F>,
    rho_43: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // denom = DELTA + rho_43 (copy + CNST-bump).
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = rho_43[i];
    }
    denom[0] = denom[0] + F::cast_from(KTX_DELTA_F64);

    // inv_denom = 1 / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // out = gaa · inv_denom.
    ctaylor_mul::<F>(grad2, &inv_denom, out, n);
}

/// XC_KTX kernel. 1:1 port of `ktx.cpp:18-24`.
#[cube]
pub fn ktx_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut ea = Array::<F>::new(size);
    ktx_alpha::<F>(&d.gaa, &d.a_43, &mut ea, n);

    let mut eb = Array::<F>::new(size);
    ktx_alpha::<F>(&d.gbb, &d.b_43, &mut eb, n);

    ctaylor_add::<F>(&ea, &eb, out, n);
}
