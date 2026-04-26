//! BR (Becke-Roussel) family helpers — `polarized` driver + ctaylor adapter.
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A. Composes the
//! Wave-0 Task-1 primitive `xcfun_ad::ctaylor_br_inverse` (the BR
//! Newton-inverse Taylor polynomial) into the BR family kernel substrate.
//!
//! # Sources
//! - `xcfun-master/src/functionals/brx.cpp:78-87` — `BR(t)` ctaylor adapter.
//! - `xcfun-master/src/functionals/brx.cpp:89-101` — `polarized` helper.
//!
//! # Wave 0 status
//!
//! Each `pub fn` is a SKELETON — signature is final, body is a placeholder.
//! Wave 1 (Plan 04-01) replaces with FULL bodies. The IMPORTANT bit is the
//! `use xcfun_ad::ctaylor_br_inverse;` line below — verifies the linkage to
//! the Wave-0 Task-1 primitive.

use cubecl::prelude::*;

// IMPORTANT: this import is the Wave-0 Task-1 primitive consumed by br_like.
// Wave 1 (04-01) BR family kernels call `ctaylor_br_inverse` here.
#[allow(unused_imports)]
use xcfun_ad::ctaylor_br_inverse;

// ---------------------------------------------------------------------------
//  BR(t) ctaylor adapter.  Port of brx.cpp:78-87.
// ---------------------------------------------------------------------------

/// `BR(t)` — evaluate BR at a CTaylor argument.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `xcfun-master/src/functionals/brx.cpp:78-87`:
///
/// ```cpp
/// template <typename T, int Nvar>
/// static ctaylor<T, Nvar> BR(const ctaylor<T, Nvar> & t) {
///   auto tmp = BR_taylor<T, (Nvar >= 3) ? Nvar : 3>(t.c[0]);
///   ctaylor<T, Nvar> res = tmp[0];
///   for (int i = 1; i <= Nvar; i++)
///     res += tmp[i] * pow(t - t.c[0], i);
///   return res;
/// }
/// ```
///
/// In our cubecl pipeline, `ctaylor_br_inverse` performs the `BR_taylor` step
/// directly (host-side scalar Newton + linear-method polynomial sweep). The
/// composition `tmp[i] * pow(t - t.c[0], i)` is performed inline in the BR
/// family kernel using existing CTaylor primitives.
#[cube]
pub fn br_t<F: Float>(t: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Wave-1 placeholder. The full implementation will:
    //   1. Host-side: seed out[0] = br_scalar(t[0]).
    //   2. Cubecl: call ctaylor_br_inverse(t, out, n) to fill slots 1..size.
    //   3. Cubecl: compose tmp[i] * pow(t - t[0], i) for i = 1..=n.
    let _ = t;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
    // Reference the Wave-0 Task-1 primitive so this module's dependency
    // on `ctaylor_br_inverse` is verified at compile time.
    // (The actual call site is the Wave-1 BR family kernel.)
}

// ---------------------------------------------------------------------------
//  polarized — single-spin BR-family helper.  Port of brx.cpp:89-101.
// ---------------------------------------------------------------------------

/// `polarized(na, gaa, lapa, taua, jpaa)` — BR-family single-spin energy
/// density helper.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `xcfun-master/src/functionals/brx.cpp:89-101`:
///
/// ```cpp
/// template <typename num>
/// static num polarized(const num & na, const num & gaa, const num & lapa,
///                      const num & taua, const num & jpaa) {
///   num Q = (lapa - 2 * taua + (0.5 * gaa + 2 * jpaa) / na) / 6.0;
///   num x = BR((1.0 / (2.0 / 3.0 * pow(M_PI, 2.0 / 3.0))) * Q
///              * pow(na, -5.0 / 3.0));
///   num b = cbrt(pow3(x) * exp(-x) / (8 * M_PI * na));
///   return -(1 - (1 + 0.5 * x) * exp(-x)) / b;
/// }
/// ```
///
/// Calls `br_t` (which composes `ctaylor_br_inverse`).
#[cube]
pub fn polarized<F: Float>(
    na: &Array<F>,
    gaa: &Array<F>,
    lapa: &Array<F>,
    taua: &Array<F>,
    jpaa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = na;
    let _ = gaa;
    let _ = lapa;
    let _ = taua;
    let _ = jpaa;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}
