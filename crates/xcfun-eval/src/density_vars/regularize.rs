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
/// f64-precision literal. Using `F::new(1e-14_f32)` would widen to
/// `9.9999998245167e-15` (rel-drift ~1.75e-8 from the C++ f64 value) — this was
/// the root cause of SLATERX/PZ81C/TFK/LDAERFC/LDAERFC_JT order-2 tier-2 failures
/// on regularize-stratum inputs (abs density < 1e-14). See 02-06 INCONCLUSIVE
/// Bug #1 diagnosis.
pub(crate) const TINY_DENSITY_F64: f64 = 1e-14_f64;

/// Clamp `x[0]` (CNST coefficient) to `>= TINY_DENSITY`. Higher-order coefficients
/// `x[1..(1<<n)]` are LEFT UNCHANGED — that is the CORE-06 + D-22 contract.
///
/// `_n` is kept as a comptime parameter for signature consistency with the rest of
/// the xcfun-eval `#[cube] fn` surface, even though the body only touches `x[0]`.
#[cube]
pub fn regularize<F: Float>(x: &mut Array<F>, #[comptime] _n: u32) {
    // C++ comparison `x < XCFUN_TINY_DENSITY` reads x.c[0] (the constant term).
    // cubecl's Float comparison is a primitive; this lowers cleanly on cubecl-cpu.
    //
    // `F::cast_from(1e-14_f64)` preserves full f64 precision when F=f64. Using
    // `F::new(f32)` would widen `1e-14_f32` to 9.9999998245167e-15, drifting the
    // clamped density by ~1.75e-8 rel-err and cascading through pow/log-composed
    // LDA kernels as tier-2 order-2 failures.
    let tiny = F::cast_from(TINY_DENSITY_F64);
    if x[0] < tiny {
        x[0] = tiny;
    }
    // Higher-order coefficients x[1..(1<<n)] are left unchanged. CORE-06 contract.
}
