//! XC_B97C — B97 GGA correlation. **GGA-09 (B97C, id=61).**
//!
//! # Source
//! - `xcfun-master/src/functionals/b97xc.cpp:25-34`     (b97c_en aggregator)
//! - `xcfun-master/src/functionals/b97c.hpp:36-59`      (energy_b97c_par + antipar)
//! - `xcfun-master/src/functionals/b97xc.hpp:22-41`     (s², ux_ab, enhancement)
//!
//! # Formula (per spin par + antipar)
//! ```cpp
//! // Parallel-spin contribution per spin:
//! e_LSDA_a       = pw92eps_polarized(a) · a
//! s2_ab2_par     = abs(gaa) / a_43²
//! u              = Γ_par · s2 / (1 + Γ_par · s2)
//! enhancement    = c_par[0] + c_par[1]·u + c_par[2]·u²            // c_par = c_b97[1]
//! energy_par_a   = e_LSDA_a · enhancement
//! // Antiparallel-spin contribution:
//! e_LSDA_anti    = pw92eps(d) · d.n - e_LSDA_a - e_LSDA_b
//! s2_anti        = 0.5 · (abs(gaa)/a_43² + abs(gbb)/b_43²)
//! enhancement    = c_anti[0] + c_anti[1]·u + c_anti[2]·u²         // c_anti = c_b97[0]
//! energy_anti    = e_LSDA_anti · enhancement
//!
//! return energy_par_a + energy_par_b + energy_anti
//! ```
//!
//! # Cross-tier import (W3)
//! - `crate::functionals::lda::pw92eps::pw92_eps`         (per-cell ε for antipar)
//! - `crate::functionals::lda::pw92eps::pw92eps_polarized` (per-spin ε for par)

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::b97_poly;
use crate::functionals::gga::shared::constants::{
    B97_C_ANTIPAR_COEF, B97_C_PAR_COEF, B97_GAMMA_C_ANTIPAR_F64, B97_GAMMA_C_PAR_F64,
};
use crate::functionals::lda::pw92eps;

/// `s2_ab2 = (gaa / a_43) / a_43` — left-associative C++ div chain.
#[cube]
fn s2_ab2<F: Float>(gaa: &Array<F>, a_43: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut inv_a43 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(a_43, &mut inv_a43, n);
    let mut first_div = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_a43, &mut first_div, n);
    ctaylor_mul::<F>(&first_div, &inv_a43, out, n);
}

/// Per-spin parallel energy: `e_LSDA · enhancement(Γ_par, c_par, s²)`.
/// Returns both the energy and the e_LSDA (out-parameter) — needed for antipar.
#[cube]
fn energy_b97c_par<F: Float>(
    rho: &Array<F>,
    rho_43: &Array<F>,
    grad2: &Array<F>,
    e_lsda: &mut Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // e_LSDA = pw92eps_polarized(rho) · rho.
    let mut eps_pol = Array::<F>::new(size);
    pw92eps::pw92eps_polarized::<F>(rho, &mut eps_pol, n);
    ctaylor_mul::<F>(&eps_pol, rho, e_lsda, n);

    // s2_ab2 = gaa / a_43².
    let mut s2 = Array::<F>::new(size);
    s2_ab2::<F>(grad2, rho_43, &mut s2, n);

    // u = Γ_par · s² / (1 + Γ_par · s²).
    let mut u = Array::<F>::new(size);
    b97_poly::ux_ab::<F>(F::cast_from(B97_GAMMA_C_PAR_F64), &s2, &mut u, n);

    // enh = c_par[0] + c_par[1]·u + c_par[2]·u².
    let mut enh = Array::<F>::new(size);
    b97_poly::b97_enhancement::<F>(
        F::cast_from(B97_C_PAR_COEF[0]),
        F::cast_from(B97_C_PAR_COEF[1]),
        F::cast_from(B97_C_PAR_COEF[2]),
        &u,
        &mut enh,
        n,
    );

    // out = e_LSDA · enh.
    ctaylor_mul::<F>(e_lsda, &enh, out, n);
}

/// Antiparallel B97C energy contribution.
#[cube]
fn energy_b97c_antipar<F: Float>(
    d: &DensVarsDev<F>,
    e_lsda_a: &Array<F>,
    e_lsda_b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // e_LSDA_total = pw92eps(d) · d.n - e_LSDA_a - e_LSDA_b.
    let mut eps_total = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps_total, n);
    let mut eps_n = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps_total, &d.n, &mut eps_n, n);
    let mut after_a = Array::<F>::new(size);
    ctaylor_sub::<F>(&eps_n, e_lsda_a, &mut after_a, n);
    let mut e_lsda = Array::<F>::new(size);
    ctaylor_sub::<F>(&after_a, e_lsda_b, &mut e_lsda, n);

    // s2_a = gaa / a_43²;  s2_b = gbb / b_43²;  s2 = 0.5 · (s2_a + s2_b).
    let mut s2_a = Array::<F>::new(size);
    s2_ab2::<F>(&d.gaa, &d.a_43, &mut s2_a, n);
    let mut s2_b = Array::<F>::new(size);
    s2_ab2::<F>(&d.gbb, &d.b_43, &mut s2_b, n);
    let mut s2_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&s2_a, &s2_b, &mut s2_sum, n);
    let mut s2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2_sum, F::new(0.5), &mut s2, n);

    // u = Γ_antipar · s² / (1 + Γ_antipar · s²).
    let mut u = Array::<F>::new(size);
    b97_poly::ux_ab::<F>(F::cast_from(B97_GAMMA_C_ANTIPAR_F64), &s2, &mut u, n);

    // enh = c_anti[0] + c_anti[1]·u + c_anti[2]·u².
    let mut enh = Array::<F>::new(size);
    b97_poly::b97_enhancement::<F>(
        F::cast_from(B97_C_ANTIPAR_COEF[0]),
        F::cast_from(B97_C_ANTIPAR_COEF[1]),
        F::cast_from(B97_C_ANTIPAR_COEF[2]),
        &u,
        &mut enh,
        n,
    );

    // out = e_LSDA · enh.
    ctaylor_mul::<F>(&e_lsda, &enh, out, n);
}

/// XC_B97C kernel. 1:1 port of `b97xc.cpp:25-34`.
#[cube]
pub fn b97c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Parallel-spin contributions (per α, β).
    let mut e_lsda_a = Array::<F>::new(size);
    let mut e_par_a = Array::<F>::new(size);
    energy_b97c_par::<F>(&d.a, &d.a_43, &d.gaa, &mut e_lsda_a, &mut e_par_a, n);

    let mut e_lsda_b = Array::<F>::new(size);
    let mut e_par_b = Array::<F>::new(size);
    energy_b97c_par::<F>(&d.b, &d.b_43, &d.gbb, &mut e_lsda_b, &mut e_par_b, n);

    // tmp = e_par_a + e_par_b.
    let mut tmp = Array::<F>::new(size);
    ctaylor_add::<F>(&e_par_a, &e_par_b, &mut tmp, n);

    // Antiparallel contribution uses e_lsda_a + e_lsda_b.
    let mut e_anti = Array::<F>::new(size);
    energy_b97c_antipar::<F>(d, &e_lsda_a, &e_lsda_b, &mut e_anti, n);

    // out = tmp + e_anti.
    ctaylor_add::<F>(&tmp, &e_anti, out, n);
}
