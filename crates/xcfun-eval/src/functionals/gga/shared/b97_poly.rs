//! B97-family power-series helpers — port target of
//! `xcfun-master/src/functionals/b97xc.hpp` + per-parameterisation `.cpp`.
//!
//! # Purpose
//! Shared `u = γ·s²/(1+γ·s²)` reduction and the 3-term enhancement polynomial
//! `c₀ + c₁·u + c₂·u²` used by B97X / B97-1X / B97-2X / B97C / B97-1C / B97-2C /
//! B97XC / etc.
//!
//! # Pitfall G6 (ACC-06)
//! `b97_enhancement` body MUST preserve the left-to-right `c₀ + c₁·u + c₂·(u·u)`
//! evaluation — do NOT reorder to Horner form `(c₂·u + c₁)·u + c₀`. Horner
//! minimises FLOPs but changes the rounding pattern vs. the C++ reference and
//! silently breaks 1e-12 parity.
//!
//! # Wave 1 status (03-01)
//! Both helpers are SKELETONS; full bodies land in 03-04 Task 1 Steps A/B.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_zero;

/// `u(γ, s²) = γ·s² / (1 + γ·s²)` — B97's bounded reduced-gradient variable.
///
/// SKELETON — full body lands in 03-04 Task 1 Step A (first B97 consumer).
#[cube]
pub fn ux_ab<F: Float>(
    gamma: F,
    s2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-04 Task 1 Step A.
    let _ = gamma;
    let _ = s2;
    ctaylor_zero::<F>(out, n);
}

/// `b97_enhancement(c₀, c₁, c₂, u) = c₀ + c₁·u + c₂·(u·u)` — three-term B97
/// enhancement polynomial.
///
/// SKELETON — full body lands in 03-04 Task 1 Step B. CRITICAL per Pitfall G6:
/// body MUST preserve `c₀ + c₁·u + c₂·(u·u)` (left-to-right); do NOT reorder
/// to Horner form.
#[cube]
pub fn b97_enhancement<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    u: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-04 Task 1 Step B.
    let _ = c0;
    let _ = c1;
    let _ = c2;
    let _ = u;
    ctaylor_zero::<F>(out, n);
}
