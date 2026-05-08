//! XC_APBEX — APBE Exchange Functional. **GGA-08.**
//!
//! # Source
//! - `xcfun-master/src/functionals/apbex.cpp:18-38`
//!
//! # Formula
//! ```cpp
//! const parameter mu = 0.26;
//! const parameter kappa = 0.804;
//! S² = pw91_like::S2(na, gaa);
//! t1 = 1 + μ·S²/κ;
//! enh = 1 + κ - κ/t1;
//! Ax = (81/(4π))^(1/3) / 2;
//! return -Ax · ρ^(4/3) · enh   per spin, summed.
//! ```
//!
//! Note: APBE-specific Ax = `(81/(4π))^(1/3) / 2 = 0.9305257363491001`
//! (= `c_slater`, NOT same as PW86X_AX which is `-(3/π)^(1/3) · 3/4` —
//! the two are algebraically distinct). APBE's exchange enhancement is the same
//! polynomial form as PBEX with different (μ, κ) — we therefore re-use the
//! shared `pbex::enhancement` helper, but with μ=MU_APBE_F64 (0.26) instead
//! of the default MU_PBE_F64 (0.066725·π²/3).
//!
//! Since the shared `pbex::enhancement` hardcodes MU_PBE_F64, we inline an
//! APBE-specific version here that takes (μ, κ) as scalar parameters.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{MU_APBE_F64, R_PBE_F64};
use crate::functionals::gga::shared::pw91_like;

/// `Ax_APBE = (81/(4π))^(1/3) / 2 = 0.9305257363491001`. This is the
/// `c_slater` constant (`pow(81/(32π), 1/3)`) — algebraically equal to
/// `pow(81/(4π), 1/3) / 2`, since `(1/8)^(1/3) = 1/2` ⇒ `81/(4π·8) = 81/(32π)`.
/// Cross-check: `xcfun-master/src/functionals/apbex.cpp:29` literal
/// `pow(81 / (4 * M_PI), 1.0 / 3.0) / 2` yields exactly this value in f64.
const APBE_AX: f64 = 0.930_525_736_349_100_1_f64;

#[cube]
fn apbe_enhancement<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // S² = pw91_like::s2(rho, grad²).
    let mut s2v = Array::<F>::new(size);
    pw91_like::s2::<F>(rho, grad2, &mut s2v, n);

    // t1 = 1 + (μ/κ) · S².
    let mu_over_kappa = F::cast_from(MU_APBE_F64 / R_PBE_F64);
    let mut scaled = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2v, mu_over_kappa, &mut scaled, n);
    let mut t1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        t1[i] = scaled[i];
    }
    t1[0] = t1[0] + F::new(1.0);

    // inv_t1 = 1 / t1.
    let mut inv_t1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&t1, &mut inv_t1, n);

    // enh = (1 + κ) - κ · inv_t1.
    let mut k_inv_t1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_t1, F::cast_from(R_PBE_F64), &mut k_inv_t1, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_kit1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&k_inv_t1, neg_one, &mut neg_kit1, n);
    #[unroll]
    for i in 0..size {
        out[i] = neg_kit1[i];
    }
    out[0] = out[0] + F::new(1.0) + F::cast_from(R_PBE_F64);
}

#[cube]
fn apbex_alpha<F: Float>(
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // enh = apbe_enhancement(rho, grad²).
    let mut enh = Array::<F>::new(size);
    apbe_enhancement::<F>(rho, grad2, &mut enh, n);

    // lda = -Ax · ρ^(4/3).
    let neg_ax = F::cast_from(-APBE_AX);
    let mut lda = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho_43, neg_ax, &mut lda, n);

    // out = lda · enh.
    ctaylor_mul::<F>(&lda, &enh, out, n);
}

/// XC_APBEX kernel. 1:1 port of `apbex.cpp:18-38`.
#[cube]
pub fn apbex_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_alpha = Array::<F>::new(size);
    apbex_alpha::<F>(&d.a_43, &d.a, &d.gaa, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    apbex_alpha::<F>(&d.b_43, &d.b, &d.gbb, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
