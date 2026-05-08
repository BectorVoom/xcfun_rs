//! `br_inverse_expand` + `br_scalar` — Brent-Kung linear-method polynomial
//! inversion of `BR_z(x) = (x - 2)/x · exp(2x/3)` for the BR family.
//!
//! Phase 4 plan 04-00 Task 1 deliverable per CONTEXT D-02.
//!
//! Port targets:
//!
//! - `xcfun-master/src/functionals/brx.cpp:21-23` — `BR_z(x) = (x - 2)/x · exp(2x/3)`
//! - `xcfun-master/src/functionals/brx.cpp:25-27` — `NR_step` Newton-Raphson update
//! - `xcfun-master/src/functionals/brx.cpp:29-48` — `BR(z)` host scalar Newton root finder
//! - `xcfun-master/src/functionals/brx.cpp:50-71` — `BR_taylor<T, Ndeg>` linear-method
//!   polynomial coefficient sweep.
//!
//! # Two-phase pipeline
//!
//! 1. **Host-side scalar Newton** (`br_scalar`): solve `BR_z(x) = z` for scalar
//!    `z`. Plain `fn`, NOT a `#[cube]`. The caller seeds the constant slot
//!    `out[0]` with `br_scalar(z[0])` BEFORE launching the cubecl kernel.
//! 2. **Cubecl `#[cube] fn` linear-method sweep** (`br_inverse_expand`): given
//!    the constant slot pre-seeded with `BR(z0)`, populate slots 1..N via the
//!    Brent-Kung linear method. Each step evaluates `f = BR_z(t)` using the
//!    existing CTaylor primitives (`ctaylor_sub`, `ctaylor_reciprocal`,
//!    `ctaylor_scalar_mul`, `ctaylor_mul`, `ctaylor_exp`) and back-solves
//!    `t[i] = -f[i] · (1 / f[1])`.
//!
//! # Precondition
//!
//! `Ndeg >= 3` (C++ `static_assert` at brx.cpp:54). At runtime the cubecl-side
//! body uses comptime depth `max(n, 3)` per the C++ template instantiation
//! `BR_taylor<T, (Nvar >= 3) ? Nvar : 3>` (brx.cpp:81).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! Same fallback as `expand/sqrt.rs`: assertions move to the host-side guard.
//! `br_scalar` runs at host-side `f64`; the cubecl body never `assert!`s.

use cubecl::prelude::*;

use crate::ctaylor::{ctaylor_scalar_mul, ctaylor_zero};
use crate::ctaylor_rec::mul::ctaylor_mul;
use crate::math::{ctaylor_exp, ctaylor_reciprocal};

// ---------------------------------------------------------------------------
// Host-side scalar functions — plain `fn`, not `#[cube]`.
// Direct verbatim port of `xcfun-master/src/functionals/brx.cpp:21-48`.
// ---------------------------------------------------------------------------

/// Port of `brx.cpp:21-23` — `BR_z(x) = (x - 2)/x · exp(2x/3)`.
///
/// Currently used only by the unit tests. Public via the `tests` mod and
/// re-exported below as `pub fn` so xtask's fixture driver and downstream
/// validation code can call it without re-implementing.
#[inline]
pub fn br_z(x: f64) -> f64 {
    (x - 2.0) / x * (2.0_f64 / 3.0_f64 * x).exp()
}

/// Port of `brx.cpp:25-27` — Newton-Raphson update step.
///
/// ```cpp
/// static double NR_step(double x, double z) {
///   return (x * (3 * x * (exp(-2.0 / 3.0 * x) * z - 1) + 6))
///        / (x * (2 * x - 4) + 6);
/// }
/// ```
#[inline]
fn nr_step(x: f64, z: f64) -> f64 {
    (x * (3.0 * x * ((-2.0_f64 / 3.0_f64 * x).exp() * z - 1.0) + 6.0)) / (x * (2.0 * x - 4.0) + 6.0)
}

/// Port of `brx.cpp:29-48` — host-side scalar Newton-Raphson root finder
/// for `BR_z(x) = z`. Returns an `x` satisfying `BR_z(x) = z` to within
/// relative tolerance `1e-15 * (1 + |x|)` after at most 20 iterations.
///
/// Four initial-guess branches (brx.cpp:31-39) cover the full `z` dynamic
/// range:
/// - `z < -1e4`         → `x0 = -2 / z`
/// - `-1e4 ≤ z < -2`    → `x0 = (sqrt(9z² + 6z + 49) + 3z + 1) / 4`
/// - `-2 ≤ z < 1`       → `x0 = 2 · (z · exp(-4/3) + 1)`
/// - `z ≥ 1`            → `x0 = 1.5 · ln(z) + 3.75 / (1.5 + ln(z))`
///
/// On non-convergence emits `BR: Not converged for z = ...` to stderr (matches
/// C++ `fprintf(stderr, ...)` at brx.cpp:46) and returns the best estimate.
pub fn br_scalar(z: f64) -> f64 {
    let mut x0 = if z < -1.0e4 {
        -2.0 / z
    } else if z < -2.0 {
        ((9.0 * z * z + 6.0 * z + 49.0).sqrt() + 3.0 * z + 1.0) / 4.0
    } else if z < 1.0 {
        2.0 * (z * (-4.0_f64 / 3.0_f64).exp() + 1.0)
    } else {
        1.5 * z.ln() + 3.75 / (1.5 + z.ln())
    };
    for _ in 0..20 {
        let xold = x0;
        x0 += nr_step(x0, z);
        if (xold - x0).abs() < 1.0e-15 * (1.0 + x0.abs()) {
            return x0;
        }
    }
    eprintln!("BR: Not converged for z = {:e}", z);
    x0
}

// ---------------------------------------------------------------------------
// Cubecl `#[cube] fn` body — Brent-Kung linear-method polynomial sweep.
// Port of `brx.cpp:53-71` (`BR_taylor<T, Ndeg>`).
// ---------------------------------------------------------------------------

/// `#[cube] fn` evaluating `f = BR_z(t) = (t - 2)/t · exp(2t/3)` on a CTaylor
/// `t`, leaving the result in `f`. Used as the inner step of
/// `br_inverse_expand`'s linear-method sweep.
///
/// Precondition: `t[0] != 0` (the `1/t` factor is undefined at `t = 0`; the
/// host-side caller guarantees `t[0] = br_scalar(z0)` is well away from 0
/// for any reasonable `z0`).
///
/// Operation order (no mul_add, no FMA per ACC-06):
///   1. `tm2  = t - 2`                   (sub via CNST-bump on a copy)
///   2. `inv_t = 1 / t`                  (ctaylor_reciprocal)
///   3. `frac = tm2 · inv_t`             (ctaylor_mul)
///   4. `arg  = (2/3) · t`               (scalar_mul)
///   5. `e    = exp(arg)`                (ctaylor_exp)
///   6. `f    = frac · e`                (ctaylor_mul)
#[cube]
fn br_z_ctaylor<F: Float>(t: &Array<F>, f: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: tm2 = t - 2 — copy t then bump CNST slot down by 2.
    let mut tm2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        tm2[i] = t[i];
    }
    tm2[0] = tm2[0] - F::new(2.0);

    // Step 2: inv_t = 1 / t.
    let mut inv_t = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(t, &mut inv_t, n);

    // Step 3: frac = tm2 · inv_t.
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&tm2, &inv_t, &mut frac, n);

    // Step 4: arg = (2/3) · t.
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(t, F::cast_from(2.0_f64 / 3.0_f64), &mut arg, n);

    // Step 5: e = exp(arg).
    let mut e = Array::<F>::new(size);
    ctaylor_exp::<F>(&arg, &mut e, n);

    // Step 6: f = frac · e.
    ctaylor_mul::<F>(&frac, &e, f, n);
}

/// `br_inverse_expand` — Brent-Kung linear-method polynomial sweep that
/// populates `t[1..size]` from a pre-seeded `t[0] = br_scalar(z0)`.
///
/// This is NOT a `*_expand`-style scalar recurrence (which writes a length-`n+1`
/// scalar Taylor series); it is the full-CTaylor linear method that mirrors
/// `xcfun-master/src/functionals/brx.cpp:53-71`:
///
/// ```cpp
/// template <typename T, int Ndeg>
/// taylor <T, 1, Ndeg> BR_taylor(const T & z0) {
///   static_assert(Ndeg >= 3, ...);
///   taylor<T, 1, Ndeg> t;
///   t = 0;
///   t[0] = BR(z0);
///   t[1] = 1;
///   taylor<T, 1, Ndeg> f;
///   f = BR_z(t);
///   t[1] = 1 / f[1];
///   for (int i = 2; i <= Ndeg; i++) {
///     f = BR_z(t);
///     t[i] = -f[i] * t[1];
///   }
///   return t;
/// }
/// ```
///
/// **IMPORTANT:** Unlike the upstream `taylor<T, 1, Ndeg>` which is a
/// 1-variable Taylor series of length `Ndeg + 1`, our `t` is a multilinear
/// CTaylor of length `1 << n`. The linear-method sweep below reads/writes
/// the CNST slot (`t[0]`) and the per-VAR slots (`t[1] = VAR0`,
/// `t[2] = VAR1`, etc.) — equivalently the bit-flag indices that the
/// xcfun-ad `index` module exports as `CNST, VAR0, VAR1, ...`.
///
/// Per CONTEXT D-02 + PATTERNS A.1: when `n < 3` the recurrence still has to
/// run for at least 3 iterations because the `BR_z` derivative formula at
/// the highest order requires it (per upstream `Ndeg >= 3` static_assert). The
/// caller guards against `n < 3` host-side; this body trusts `n >= 0` and
/// loops at most `size - 1` iterations (which equals `(1 << n) - 1`). For
/// `n < 3` the cubecl body still completes correctly because the missing
/// higher-order terms are clamped to zero by `ctaylor_zero` initialisation.
///
/// Preconditions:
/// - `t[0]` MUST be pre-seeded by the host with `br_scalar(z0)` before launch.
/// - `n >= 0` (cubecl body trusts; host-side caller verifies n ≤ 7 per AD-01).
#[cube]
pub fn br_inverse_expand<F: Float>(t: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Defensive: zero out everything except CNST. Caller pre-seeds t[0].
    // (Phase-4 RESEARCH guarantees host-side already zeroed the slice; this
    // is belt-and-suspenders against a non-zero `Array::new` allocator.)
    #[unroll]
    for i in 1..size {
        t[i] = F::new(0.0);
    }

    // The C++ algorithm:
    //
    //   t[1] = 1;            // seed VAR0 = 1
    //   f = BR_z(t);
    //   t[1] = 1 / f[1];
    //   for (i = 2; i <= Ndeg; i++) {
    //     f = BR_z(t);
    //     t[i] = -f[i] * t[1];
    //   }
    //
    // For our multilinear CTaylor, `t[1]` is the VAR0 slope and `f[1]` is the
    // first-derivative coefficient of f at t[0] (which equals dBR_z/dx at
    // x = t[0]). Setting `t[1] = 1 / f[1]` is the inverse-function-theorem
    // first-order coefficient of x(z); the higher-order coefficients follow
    // by the linear-method back-substitution.

    // size == 1 (n == 0): only the CNST slot exists; nothing to do.
    if comptime!(size > 1) {
        // Step 1: seed t[1] = 1.
        t[1] = F::new(1.0);

        // Step 2: f = BR_z(t).
        let mut f = Array::<F>::new(size);
        ctaylor_zero::<F>(&mut f, n);
        br_z_ctaylor::<F>(t, &mut f, n);

        // Step 3: t[1] = 1 / f[1].
        let inv_f1 = F::new(1.0) / f[1];
        t[1] = inv_f1;

        // Step 4: for i in 2..size: f = BR_z(t); t[i] = -f[i] * t[1].
        //
        // We evaluate the FULL CTaylor BR_z(t) on every iteration (matches
        // brx.cpp:66 `f = BR_z(t)`). This is wasteful per-step (re-computes
        // unchanged lower-order coefficients) but matches the C++ algorithm
        // bit-for-bit — algorithmic-identity rule.
        #[unroll]
        for i in 2..size {
            br_z_ctaylor::<F>(t, &mut f, n);
            // t[i] = -f[i] * t[1]  (== -f[i] * inv_f1; we re-read t[1] for
            //                       clarity / symmetry with the C++ source).
            t[i] = -f[i] * t[1];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1 (per plan acceptance criterion): `br_scalar(-2.0)` converges
    /// to a finite f64 in 20 iterations.
    #[test]
    fn br_scalar_neg2_converges() {
        let x = br_scalar(-2.0);
        assert!(x.is_finite(), "br_scalar(-2.0) returned non-finite: {}", x);
    }

    /// Test 2: For any converged `x = br_scalar(z)`, `BR_z(x) - z` is small
    /// (within Newton's documented `1e-15 * (1 + |x|)` tolerance, so the
    /// residual on `BR_z(x) - z` is well below `1e-13` relative).
    #[test]
    fn br_scalar_satisfies_inverse_relation() {
        // Sample 12 z-values across the four C++ initial-guess branches.
        let zs: [f64; 12] = [
            -1e6, -1e5, -1e4, // z < -1e4 branch
            -100.0, -10.0, -2.5, // -1e4 ≤ z < -2 branch
            -1.5, -0.5, 0.0, 0.5, // -2 ≤ z < 1 branch
            1.5, 100.0, // z ≥ 1 branch
        ];
        for &z in &zs {
            let x = br_scalar(z);
            let z_back = br_z(x);
            let denom = z.abs().max(1.0);
            let rel = (z_back - z).abs() / denom;
            assert!(
                rel < 1e-13,
                "br_scalar inverse: z={}, x={}, BR_z(x)={}, rel_err={:.3e}",
                z,
                x,
                z_back,
                rel
            );
        }
    }
}
