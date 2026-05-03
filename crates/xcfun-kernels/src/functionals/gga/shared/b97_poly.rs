//! B97-family power-series helpers — port target of
//! `xcfun-master/src/functionals/b97xc.hpp` + per-parameterisation `.cpp`.
//!
//! # Purpose
//! Shared `u = γ·s²/(1+γ·s²)` reduction and the 3-term enhancement polynomial
//! `c₀ + c₁·u + c₂·u²` used by B97X / B97-1X / B97-2X / B97C / B97-1C / B97-2C.
//!
//! # Pitfall G6 (ACC-06)
//! `b97_enhancement` body MUST preserve the left-to-right `c₀ + c₁·u + c₂·(u·u)`
//! evaluation — do NOT reorder to Horner form `(c₂·u + c₁)·u + c₀`. Horner
//! minimises FLOPs but changes the rounding pattern vs. the C++ reference and
//! silently breaks 1e-12 parity. B97-2C's `c₂ = -7.44060` makes this the
//! largest |c₂| in the family — the conditioning stress.
//!
//! # Wave 4 status (03-04)
//! Both helpers ship as **FULL BODIES** (Wave 4, plan 03-04 — W3 conversion
//! complete; SKELETON markers removed).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

/// `u(γ, s²) = γ·s² / (1 + γ·s²)` — B97's bounded reduced-gradient variable.
///
/// **FULL BODY** (Wave 4, plan 03-04).
///
/// Port of `b97xc.hpp:28-31`:
/// ```cpp
/// num ux = Gamma * spin_dens_grad / (1.0 + Gamma * spin_dens_grad);
/// ```
///
/// Operation order (no mul_add per ACC-06):
///   1. `num_term = γ·s²`                  (scalar_mul)
///   2. `denom    = 1 + num_term`          (copy + CNST-bump)
///   3. `inv_denom = 1 / denom`            (ctaylor_reciprocal)
///   4. `out      = num_term · inv_denom`  (ctaylor_mul)
#[cube]
pub fn ux_ab<F: Float>(
    gamma: F,
    s2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: num_term = γ · s².
    let mut num_term = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(s2, gamma, &mut num_term, n);

    // Step 2: denom = 1 + num_term (copy + CNST-bump).
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = num_term[i];
    }
    denom[0] = denom[0] + F::new(1.0);

    // Step 3: inv_denom = 1 / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // Step 4: out = num_term · inv_denom.
    ctaylor_mul::<F>(&num_term, &inv_denom, out, n);
}

/// `b97_enhancement(c₀, c₁, c₂, u) = c₀ + c₁·u + c₂·(u·u)` — three-term B97
/// enhancement polynomial.
///
/// **FULL BODY** (Wave 4, plan 03-04). CRITICAL per Pitfall G6: body preserves
/// `c₀ + c₁·u + c₂·(u·u)` (left-to-right); does NOT reorder to Horner form.
///
/// Port of `b97xc.hpp:34-41`:
/// ```cpp
/// return c_params[0] + c_params[1] * ux + c_params[2] * ux * ux;
/// ```
///
/// Operation order (no mul_add per ACC-06; explicit `u² = u · u` per G6):
///   1. `u2     = u · u`           (ctaylor_mul — explicit, NOT fused)
///   2. `term1  = c₁ · u`          (scalar_mul)
///   3. `term2  = c₂ · u²`         (scalar_mul)
///   4. `sum    = term1 + term2`   (ctaylor_add)
///   5. `out    = c₀ + sum`        (copy + CNST-bump via add)
#[cube]
pub fn b97_enhancement<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    u: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: u² = u · u (EXPLICIT — Pitfall G6 forbids fusing into mul_add).
    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(u, u, &mut u2, n);

    // Step 2: term1 = c₁ · u.
    let mut term1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(u, c1, &mut term1, n);

    // Step 3: term2 = c₂ · u² (B97-2C stresses this with c₂ = -7.44060).
    let mut term2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&u2, c2, &mut term2, n);

    // Step 4: sum = term1 + term2.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&term1, &term2, &mut sum, n);

    // Step 5: out = c₀ + sum (copy sum into out, then bump CNST by c₀).
    #[unroll]
    for i in 0..size {
        out[i] = sum[i];
    }
    out[0] = out[0] + c0;
}
