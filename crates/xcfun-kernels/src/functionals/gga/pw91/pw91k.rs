//! XC_PW91K — Perdew-Wang 1991 GGA Kinetic Energy Functional. **GGA-06.**
//!
//! # Source
//! - `xcfun-master/src/functionals/pw91k.cpp:21-28`
//!
//! # Formula
//! ```cpp
//! const parameter param_AB[6] = {
//!     0.093907, 76.320, 0.26608, 0.0809615, 100.0, 0.57767e-4};
//! using pw91_like_x_internal::pw91k_prefactor;
//! using pw91_like_x_internal::pw91xk_enhancement;
//! return pw91k_prefactor(d.a) * pw91xk_enhancement(param_AB, d.a, d.gaa)
//!      + pw91k_prefactor(d.b) * pw91xk_enhancement(param_AB, d.b, d.gbb);
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pw91_like;

const PW91K_A1: f64 = 0.093_907_f64;
const PW91K_A2: f64 = 76.320_f64;
const PW91K_A3: f64 = 0.266_08_f64;
const PW91K_A4: f64 = 0.080_961_5_f64;
const PW91K_A5: f64 = 100.0_f64;
const PW91K_B: f64 = 0.577_67e-4_f64;

#[cube]
fn pw91k_alpha<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // pre = pw91k_prefactor(rho).
    let mut pre = Array::<F>::new(size);
    pw91_like::pw91k_prefactor::<F>(rho, &mut pre, n);

    // s2v = S²(rho, grad²).
    let mut s2v = Array::<F>::new(size);
    pw91_like::s2::<F>(rho, grad2, &mut s2v, n);

    // enh = pw91xk_enhancement(s², ...).
    let mut enh = Array::<F>::new(size);
    pw91_like::pw91xk_enhancement::<F>(
        &s2v,
        F::cast_from(PW91K_A1),
        F::cast_from(PW91K_A2),
        F::cast_from(PW91K_A3),
        F::cast_from(PW91K_A4),
        F::cast_from(PW91K_A5),
        F::cast_from(PW91K_B),
        &mut enh,
        n,
    );

    // out = pre · enh.
    ctaylor_mul::<F>(&pre, &enh, out, n);
}

/// XC_PW91K kernel. 1:1 port of `pw91k.cpp:21-28`.
#[cube]
pub fn pw91k_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_alpha = Array::<F>::new(size);
    pw91k_alpha::<F>(&d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    pw91k_alpha::<F>(&d.b, &d.gbb, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
