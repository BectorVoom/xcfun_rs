//! XC_OPTX — Handy-Cohen OPTX GGA exchange. **GGA-05.**
//!
//! # Source
//! - `xcfun-master/src/functionals/optx.cpp:18-26`
//!
//! # Formula
//! ```cpp
//! const parameter a1 = 1.05151, a2 = 1.43169, gamma = 0.006;
//! num g_xa2 = gamma * d.gaa * pow(d.a, -8.0/3.0);
//! num g_xb2 = gamma * d.gbb * pow(d.b, -8.0/3.0);
//! return -(d.a_43 * (a1 * c_slater + a2 * pow(g_xa2, 2) * pow(1 + g_xa2, -2)))
//!      - (d.b_43 * (a1 * c_slater + a2 * pow(g_xb2, 2) * pow(1 + g_xb2, -2)));
//! ```
//!
//! Decomposes into `−(a_43 · BR_a) − (b_43 · BR_b)` where each bracket is
//! `a1 · c_slater + a2 · g²/(1+g)²` per spin channel. The `a1·c_slater` term
//! is constant (CNST-only), so we add it via CNST-bump after computing the
//! `optx_enhancement` part.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{C_SLATER_F64, OPTX_A1_F64, OPTX_A2_F64};
use crate::functionals::gga::shared::optx;

#[cube]
fn optx_alpha<F: Float>(
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // g_xa2 = γ · grad² · ρ^(-8/3).
    let mut gx = Array::<F>::new(size);
    optx::g_xa2::<F>(rho, grad2, &mut gx, n);

    // enh = a2 · g_xa2² / (1 + g_xa2)².
    let mut enh = Array::<F>::new(size);
    optx::optx_enhancement::<F>(
        &gx,
        F::cast_from(OPTX_A1_F64),
        F::cast_from(OPTX_A2_F64),
        &mut enh,
        n,
    );

    // bracket = a1 · c_slater + enh. Add (a1·c_slater) to CNST.
    let mut bracket = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket[i] = enh[i];
    }
    bracket[0] = bracket[0] + F::cast_from(OPTX_A1_F64 * C_SLATER_F64);

    // a43_bracket = a_43 · bracket.
    let mut a43_bracket = Array::<F>::new(size);
    ctaylor_mul::<F>(rho_43, &bracket, &mut a43_bracket, n);

    // out = -(a43_bracket).
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&a43_bracket, neg_one, out, n);
}

/// XC_OPTX kernel. 1:1 port of `optx.cpp:18-26`.
#[cube]
pub fn optx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_alpha = Array::<F>::new(size);
    optx_alpha::<F>(&d.a_43, &d.a, &d.gaa, &mut e_alpha, n);

    let mut e_beta = Array::<F>::new(size);
    optx_alpha::<F>(&d.b_43, &d.b, &d.gbb, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
