//! XC_OPTXCORR — OPTX correction part only. **GGA-05.**
//!
//! # Source
//! - `xcfun-master/src/functionals/optxcorr.cpp:18-33`
//!
//! # Formula (sign flip relative to OPTX, no LDA part)
//! ```cpp
//! const parameter gamma = 0.006;
//! num g_xa2 = gamma * d.gaa * pow(d.a, -8.0/3.0);
//! num g_xb2 = gamma * d.gbb * pow(d.b, -8.0/3.0);
//! return  (d.a_43 * (pow(g_xa2, 2) * pow(1 + g_xa2, -2)))
//!       + (d.b_43 * (pow(g_xb2, 2) * pow(1 + g_xb2, -2)));
//! ```
//!
//! Note: optxcorr.cpp uses `g²/(1+g)²` directly (no `a2` multiplier — the
//! upstream comment explicitly says "without the weighting parameters"). So
//! we feed `a2 = 1.0` to the shared `optx_enhancement` helper.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::optx;

#[cube]
fn optxcorr_alpha<F: Float>(
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // g_xa2 = γ · grad² · ρ^(-8/3) (γ baked into shared::g_xa2).
    let mut gx = Array::<F>::new(size);
    optx::g_xa2::<F>(rho, grad2, &mut gx, n);

    // enh = 1.0 · g_xa2² / (1 + g_xa2)²  (a1 unused, a2 = 1.0 for correction).
    let mut enh = Array::<F>::new(size);
    optx::optx_enhancement::<F>(&gx, F::new(0.0), F::new(1.0), &mut enh, n);

    // out = a_43 · enh.
    ctaylor_mul::<F>(rho_43, &enh, out, n);
}

/// XC_OPTXCORR kernel. 1:1 port of `optxcorr.cpp:18-33`.
#[cube]
pub fn optxcorr_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_alpha = Array::<F>::new(size);
    optxcorr_alpha::<F>(&d.a_43, &d.a, &d.gaa, &mut e_alpha, n);

    let mut e_beta = Array::<F>::new(size);
    optxcorr_alpha::<F>(&d.b_43, &d.b, &d.gbb, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
