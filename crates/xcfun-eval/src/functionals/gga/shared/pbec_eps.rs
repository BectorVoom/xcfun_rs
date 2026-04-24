//! PBE correlation epsilon helpers — 1:1 port target of
//! `xcfun-master/src/functionals/pbec_eps.hpp` + consumer pieces of `pbec.cpp`.
//!
//! # Purpose
//! Inner helpers shared by PBEC / APBEC / SPBEC / PBEINTC / PBELOCC / ZVPBESOLC /
//! ZVPBEINTC / VWN_PBEC / PW91C — the β/γ correlation algebra around
//! `expm1(-ε/(γ·u³))` and the spin-polarisation factor φ(ζ).
//!
//! # Source
//! - `xcfun-master/src/functionals/pbec_eps.hpp:22-40` — `A`, `H` via expm1/log
//! - `xcfun-master/src/functionals/pbec_eps.hpp:44-60` — `phi(ζ) = ½·((1+ζ)^(2/3) + (1-ζ)^(2/3))`
//!
//! # Critical port rule (Known Hazard §PBEC β/γ)
//! Preserve operation order around `expm1`: compute `expm1(-ε/(γ·u³))` **first**
//! as a `ctaylor_expm1` on the scaled argument, then `ctaylor_reciprocal`, then
//! `scalar_mul` by `β_gamma`. Do NOT algebraically simplify to
//! `β_gamma / (exp(...) - 1)` — that loses the x → 0 stable-bracket from D-05.
//!
//! # Wave 1 status (03-01)
//! All three helpers are SKELETONS; full bodies land in 03-02 Task 1 Step A
//! (PBEC first consumer).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_zero;

/// `A_expm1_inner(ε, u³) = β_gamma / expm1(-ε / (γ·u³))` — the A term of the
/// PBEC H gradient correction.
///
/// SKELETON — full body lands in 03-02 Task 1 Step A (PBEC consumer).
#[cube]
pub fn a_expm1_inner<F: Float>(
    eps: &Array<F>,
    u3: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-02 Task 1 Step A.
    let _ = eps;
    let _ = u3;
    ctaylor_zero::<F>(out, n);
}

/// `h_gga(d², ε, u³)` — PBEC gradient correction H term from `pbec_eps.hpp:32-40`.
///
/// SKELETON — full body lands in 03-02 Task 1 Step A (PBEC consumer).
#[cube]
pub fn h_gga<F: Float>(
    d2: &Array<F>,
    eps: &Array<F>,
    u3: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-02 Task 1 Step A.
    let _ = d2;
    let _ = eps;
    let _ = u3;
    ctaylor_zero::<F>(out, n);
}

/// `phi(ζ) = ½ · ((1+ζ)^(2/3) + (1-ζ)^(2/3))` — PBEC spin-polarisation factor.
///
/// SKELETON — full body lands in 03-02 Task 1 Step A (PBEC consumer).
#[cube]
pub fn phi<F: Float>(
    zeta: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-02 Task 1 Step A.
    let _ = zeta;
    ctaylor_zero::<F>(out, n);
}
