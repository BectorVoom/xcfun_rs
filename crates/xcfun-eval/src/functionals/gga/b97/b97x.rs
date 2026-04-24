//! XC_B97X — B97 GGA exchange. **GGA-09 (B97X, id=60).**
//!
//! # Source
//! - `xcfun-master/src/functionals/b97xc.cpp:20-23`     (b97x_en aggregator)
//! - `xcfun-master/src/functionals/b97x.hpp:23-43`      (e_x_LSDA_ab + energy_b97x_ab)
//! - `xcfun-master/src/functionals/b97xc.hpp:22-41`     (spin_dens_gradient_ab2 + ux_ab + enhancement)
//!
//! # Formula (per spin, summed)
//! ```cpp
//! s2_ab          = abs(gaa) / a_43 / a_43
//! u              = Gamma · s2_ab / (1 + Gamma · s2_ab)
//! enhancement    = c[0] + c[1]·u + c[2]·u·u             // G6: preserve op order
//! e_x_LSDA_ab    = -PREFACTOR · a_43                    // PREFACTOR = 0.9305257363491002
//! energy_b97x_ab = e_x_LSDA_ab · enhancement
//! ```
//!
//! Total: `energy = energy_b97x_ab(α) + energy_b97x_ab(β)`.
//!
//! # Preconditions (XC_A_B_GAA_GAB_GBB Vars arm)
//! - `d.a`, `d.b`, `d.gaa`, `d.gbb`, `d.a_43`, `d.b_43` populated.
//! - `gaa, gbb ≥ 0` by construction (∇ρ·∇ρ); `abs()` is a no-op.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::b97_poly;
use crate::functionals::gga::shared::constants::{B97_GAMMA_X_F64, B97_X_COEF};

/// `s2_ab(gaa, a_43) = (gaa / a_43) / a_43` — C++ left-associative div chain.
///
/// Op order matches `b97xc.hpp:22-26` verbatim (port of `abs(gaa)/a_43/a_43`).
/// `abs(gaa)` is a no-op since `gaa = ∇ρ·∇ρ ≥ 0`.
#[cube]
fn s2_ab<F: Float>(
    gaa: &Array<F>,
    a_43: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // inv_a43 = 1 / a_43.
    let mut inv_a43 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(a_43, &mut inv_a43, n);

    // first_div = gaa / a_43 = gaa · (1/a_43).
    let mut first_div = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_a43, &mut first_div, n);

    // out = first_div / a_43 = first_div · (1/a_43).
    ctaylor_mul::<F>(&first_div, &inv_a43, out, n);
}

/// Per-spin B97 exchange energy: `e_x_LSDA · enhancement(Γ, c, s²)`.
///
/// `e_x_LSDA = -PREFACTOR · a_43` where `PREFACTOR = 0.9305257363491002`
/// (b97x.hpp:18 — equals `-NEG_C_SLATER_F64 = C_SLATER_F64`, but the C++ source
/// hard-codes the precomputed literal so we mirror that for byte-for-byte parity).
#[cube]
fn energy_b97x_ab<F: Float>(
    gamma: F,
    c0: F,
    c1: F,
    c2: F,
    rho_43: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // s2 = (gaa / a_43) / a_43.
    let mut s2 = Array::<F>::new(size);
    s2_ab::<F>(grad2, rho_43, &mut s2, n);

    // u = Γ · s² / (1 + Γ · s²).
    let mut u = Array::<F>::new(size);
    b97_poly::ux_ab::<F>(gamma, &s2, &mut u, n);

    // enh = c₀ + c₁·u + c₂·(u·u) — Pitfall G6 preserved by b97_enhancement.
    let mut enh = Array::<F>::new(size);
    b97_poly::b97_enhancement::<F>(c0, c1, c2, &u, &mut enh, n);

    // lsda = -PREFACTOR · a_43.
    // PREFACTOR_F64 = 0.9305257363491002 (b97x.hpp:18). The minus is part of the
    // scalar — combine into a single scalar_mul to mirror C++ `-PREFACTOR * a_43`.
    const NEG_PREFACTOR_F64: f64 = -0.930_525_736_349_100_2_f64;
    let mut lsda = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho_43, F::cast_from(NEG_PREFACTOR_F64), &mut lsda, n);

    // out = lsda · enh.
    ctaylor_mul::<F>(&lsda, &enh, out, n);
}

/// XC_B97X kernel. 1:1 port of `b97xc.cpp:20-23`.
#[cube]
pub fn b97x_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let gamma = F::cast_from(B97_GAMMA_X_F64);
    let c0 = F::cast_from(B97_X_COEF[0]);
    let c1 = F::cast_from(B97_X_COEF[1]);
    let c2 = F::cast_from(B97_X_COEF[2]);

    // e_alpha = energy_b97x_ab(Γ, c_b97, a_43, gaa).
    let mut e_alpha = Array::<F>::new(size);
    energy_b97x_ab::<F>(gamma, c0, c1, c2, &d.a_43, &d.gaa, &mut e_alpha, n);

    // e_beta = energy_b97x_ab(Γ, c_b97, b_43, gbb).
    let mut e_beta = Array::<F>::new(size);
    energy_b97x_ab::<F>(gamma, c0, c1, c2, &d.b_43, &d.gbb, &mut e_beta, n);

    // out = e_alpha + e_beta.
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
