//! `regularize` — clamp `x[CNST]` to `>= XCFUN_TINY_DENSITY = 1e-14`.
//! Higher-order coefficients (`x[VAR0..]`) preserved per CORE-06 + D-22.
//!
//! Source (1:1 port):
//!
//! ```cpp
//! // xcfun-master/src/densvars.hpp:22-25
//! template <typename T, int N> void regularize(ctaylor<T, N> & x) {
//!     if (x < xcfun::XCFUN_TINY_DENSITY)
//!         x.set(0, xcfun::XCFUN_TINY_DENSITY);   // mutates only c[0]
//! }
//! ```
//!
//! `XCFUN_TINY_DENSITY` is `1e-14` per `xcfun-master/src/config.hpp:22`.
//!
//! Verified by `crates/xcfun-eval/tests/regularize_invariant.rs` (CORE-06).

use cubecl::prelude::*;

/// Matches `xcfun-master/src/config.hpp:22` `XCFUN_TINY_DENSITY = 1e-14`.
///
/// cubecl 0.10-pre.3's `F::new` takes `f32`; `1e-14_f32` is exactly representable
/// as `1.0000000031710769e-14_f64` after the f32 → f64 widening (absolute delta
/// ~3.2e-23). Well within the 1e-12 accuracy contract.
pub(crate) const TINY_DENSITY_F32: f32 = 1e-14_f32;

/// Clamp `x[0]` (CNST coefficient) to `>= TINY_DENSITY`. Higher-order coefficients
/// `x[1..(1<<n)]` are LEFT UNCHANGED — that is the CORE-06 + D-22 contract.
///
/// `_n` is kept as a comptime parameter for signature consistency with the rest of
/// the xcfun-eval `#[cube] fn` surface, even though the body only touches `x[0]`.
#[cube]
pub fn regularize<F: Float>(x: &mut Array<F>, #[comptime] _n: u32) {
    // C++ comparison `x < XCFUN_TINY_DENSITY` reads x.c[0] (the constant term).
    // cubecl's Float comparison is a primitive; this lowers cleanly on cubecl-cpu.
    let tiny = F::new(TINY_DENSITY_F32);
    if x[0] < tiny {
        x[0] = tiny;
    }
    // Higher-order coefficients x[1..(1<<n)] are left unchanged. CORE-06 contract.
}
