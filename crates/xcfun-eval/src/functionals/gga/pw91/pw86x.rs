//! XC_PW86X — Perdew-Wang 1986 GGA exchange. **GGA-06.**
//!
//! # Source
//! - `xcfun-master/src/functionals/pw86x.cpp:17-32`
//!
//! # Formula (per-spin `pw86x(na, gaa)`):
//! ```cpp
//! const parameter a = 1.0, b = 1.296, c = 14.0, d = 0.20;
//! num rho = 2 * na, grad2 = 4 * gaa;
//! const num Ax = -pow(3.0/M_PI, 1.0/3.0) * 3.0/4.0;
//! const num kf = pow(3.0 * pow(M_PI, 2) * rho, 1.0/3.0);
//! num s2 = grad2 / pow(2.0 * kf * rho, 2);
//! num F = pow(a + s2*(b + s2*(c + d*s2)), 1.0/15.0);
//! return Ax * pow(rho, 4.0/3.0) * F;
//! ```
//! The full functional is `pw86xtot(d) = 0.5 * (pw86x(d.a, d.gaa) + pw86x(d.b, d.gbb))`.
//!
//! # Constants (precomputed in f64)
//! - `Ax = -(3/π)^(1/3) · 3/4` — same as `xcfun_constants::c_slater` with sign flipped:
//!   `-pow(3.0/M_PI, 1.0/3.0) * 3.0/4.0 = -0.738558766382022...`
//!
//! Per-spin `s²` uses the C++-specific normalisation
//! `s² = grad2 / (2 kF ρ)² = grad2 / (4 kF² ρ²)` where ρ=2·na, grad2=4·gaa.
//! Algebraic substitution:
//!   `kF² · ρ² = (3π² · ρ)^(2/3) · ρ² = (3π²)^(2/3) · ρ^(8/3)`.
//! So `s² = grad2 / (4 · (3π²)^(2/3) · ρ^(8/3))`.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_pow;

use crate::density_vars::DensVarsDev;

// pw86x_alpha computes pw86x(na, gaa). Per C++:
//   rho = 2·na, grad2 = 4·gaa
//   kF = (3π²·ρ)^(1/3)
//   s² = grad2 / (2·kF·ρ)² = grad2 / (4·kF²·ρ²)
//   F = (1 + s²·(b + s²·(c + d·s²)))^(1/15)
//   = pow_f64(a + s²·(b + s²·(c + d·s²)), 1/15)
//   return Ax · ρ^(4/3) · F
//
// Numerical constants:
//   Ax = -(3/π)^(1/3) · 3/4 = -0.7385587663820223
//   kF^2_pref = (3π²)^(2/3) ≈ 9.5708981...
//   1/(4·(3π²)^(2/3)) ≈ 0.026121...
//
// Substitution: ρ^(4/3) = (2na)^(4/3) = 2^(4/3) · na^(4/3); but we keep
// rho/ρ symbolically via ctaylor_scalar_mul + pow_43.

const PW86X_A: f64 = 1.0_f64;
const PW86X_B: f64 = 1.296_f64;
const PW86X_C: f64 = 14.0_f64;
const PW86X_D: f64 = 0.20_f64;
/// `-(3/π)^(1/3) · 3/4` — same magnitude as exchange Slater constant.
const PW86X_AX: f64 = -0.738_558_766_382_022_3_f64;

#[cube]
fn pw86x_alpha<F: Float>(
    na: &Array<F>,
    gaa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // ρ = 2·na (CTaylor).
    let mut rho = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(na, F::new(2.0), &mut rho, n);

    // grad2_g = 4·gaa.
    let mut grad2_g = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(gaa, F::new(4.0), &mut grad2_g, n);

    // ρ^(4/3).
    let mut rho_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(&rho, F::cast_from(4.0_f64 / 3.0_f64), &mut rho_43, n);

    // ρ^(8/3) = ρ^(4/3) · ρ^(4/3).
    let mut rho_83 = Array::<F>::new(size);
    ctaylor_mul::<F>(&rho_43, &rho_43, &mut rho_83, n);

    // s² = grad2 / (4·(3π²)^(2/3) · ρ^(8/3))
    //    = grad2 · (1 / (4·(3π²)^(2/3))) · ρ^(-8/3)
    // Precomputed: 1 / (4·(3π²)^(2/3)) = 0.026121172985233605
    const S2_DIVISOR_INV: f64 = 0.026_121_172_985_233_605_f64;
    let mut rho_neg83 = Array::<F>::new(size);
    ctaylor_pow::<F>(&rho, F::cast_from(-8.0_f64 / 3.0_f64), &mut rho_neg83, n);
    let mut s2_unscaled = Array::<F>::new(size);
    ctaylor_mul::<F>(&grad2_g, &rho_neg83, &mut s2_unscaled, n);
    let mut s2v = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2_unscaled, F::cast_from(S2_DIVISOR_INV), &mut s2v, n);

    // inner = c + d·s².
    let mut d_s2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2v, F::cast_from(PW86X_D), &mut d_s2, n);
    let mut inner = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner[i] = d_s2[i];
    }
    inner[0] = inner[0] + F::cast_from(PW86X_C);

    // mid = b + s²·inner.
    let mut s2_inner = Array::<F>::new(size);
    ctaylor_mul::<F>(&s2v, &inner, &mut s2_inner, n);
    let mut mid = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        mid[i] = s2_inner[i];
    }
    mid[0] = mid[0] + F::cast_from(PW86X_B);

    // poly = a + s²·mid.
    let mut s2_mid = Array::<F>::new(size);
    ctaylor_mul::<F>(&s2v, &mid, &mut s2_mid, n);
    let mut poly = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        poly[i] = s2_mid[i];
    }
    poly[0] = poly[0] + F::cast_from(PW86X_A);

    // f_enh = pow(poly, 1/15).
    let mut f_enh = Array::<F>::new(size);
    ctaylor_pow::<F>(&poly, F::cast_from(1.0_f64 / 15.0_f64), &mut f_enh, n);

    // out = Ax · ρ^(4/3) · f_enh.
    let mut ax_rho43 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rho_43, F::cast_from(PW86X_AX), &mut ax_rho43, n);
    ctaylor_mul::<F>(&ax_rho43, &f_enh, out, n);
}

/// XC_PW86X kernel. 1:1 port of `pw86x.cpp:30-32`.
/// `pw86xtot(d) = 0.5 · (pw86x(d.a, d.gaa) + pw86x(d.b, d.gbb))`.
#[cube]
pub fn pw86x_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_alpha = Array::<F>::new(size);
    pw86x_alpha::<F>(&d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    pw86x_alpha::<F>(&d.b, &d.gbb, &mut e_beta, n);

    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&e_alpha, &e_beta, &mut sum, n);
    ctaylor_scalar_mul::<F>(&sum, F::new(0.5), out, n);
}
