//! XC_PW91X — Perdew-Wang 1991 GGA exchange. **GGA-06.**
//!
//! # Source
//! - `xcfun-master/src/functionals/pw91x.cpp:18-24`
//!
//! # Formula
//! ```cpp
//! const parameter param_AB[6] = {0.19645, 7.7956, 0.2743, 0.15084, 100.0, 0.004};
//! using pw91_like_x_internal::prefactor;
//! using pw91_like_x_internal::pw91xk_enhancement;
//! return prefactor(d.a) * pw91xk_enhancement(param_AB, d.a, d.gaa)
//!      + prefactor(d.b) * pw91xk_enhancement(param_AB, d.b, d.gbb);
//! ```
//!
//! Note: per `pw9xx.hpp:73-94`, `pw91xk_enhancement` accepts (rho, grad) and
//! computes `S²(rho, grad)` internally — but our shared helper `pw91xk_enhancement`
//! signature accepts the precomputed `S²` array. We compute `S²` here per-spin
//! and pass it.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pw91_like;

const PW91X_A1: f64 = 0.196_45_f64;
const PW91X_A2: f64 = 7.795_6_f64;
const PW91X_A3: f64 = 0.274_3_f64;
const PW91X_A4: f64 = 0.150_84_f64;
const PW91X_A5: f64 = 100.0_f64;
const PW91X_B: f64 = 0.004_f64;

#[cube]
fn pw91x_alpha<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // pre = prefactor(rho) = NEG_C_SLATER · rho^(4/3).
    let mut pre = Array::<F>::new(size);
    pw91_like::prefactor::<F>(rho, &mut pre, n);

    // s2v = S²(rho, grad²) = S2_PREFACTOR · grad²/rho^(8/3).
    let mut s2v = Array::<F>::new(size);
    pw91_like::s2::<F>(rho, grad2, &mut s2v, n);

    // enh = pw91xk_enhancement(s², a1, a2, a3, a4, a5, b).
    let mut enh = Array::<F>::new(size);
    pw91_like::pw91xk_enhancement::<F>(
        &s2v,
        F::cast_from(PW91X_A1),
        F::cast_from(PW91X_A2),
        F::cast_from(PW91X_A3),
        F::cast_from(PW91X_A4),
        F::cast_from(PW91X_A5),
        F::cast_from(PW91X_B),
        &mut enh,
        n,
    );

    // out = pre · enh.
    ctaylor_mul::<F>(&pre, &enh, out, n);
}

/// XC_PW91X kernel. 1:1 port of `pw91x.cpp:18-24`.
#[cube]
pub fn pw91x_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_alpha = Array::<F>::new(size);
    pw91x_alpha::<F>(&d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    pw91x_alpha::<F>(&d.b, &d.gbb, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
