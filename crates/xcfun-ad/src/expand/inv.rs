//! Taylor series of `1/(a+x)`. Port of
//! `xcfun-master/external/upstream/taylor/tmath.hpp:124-129`.
//!
//! # C++ recurrence (tmath.hpp:124-129)
//!
//! ```cpp
//! // Taylor series of 1/(a+x)
//! template <class T, int N> static void inv_expand(T * t, const T & a) {
//!   assert(a != 0 && "1/(a+x) not analytic at a = 0");
//!   t[0] = 1 / a;
//!   for (int i = 1; i <= N; i++)
//!     t[i] = -t[i - 1] * t[0];
//! }
//! ```
//!
//! # Mathematical identity
//!
//! `1/(a+x) = (1/a) * sum_{k>=0} (-x/a)^k`, so the coefficient of `x^i` is
//! `(-1)^i / a^(i+1)`. The recurrence `t[i] = -t[i-1] * t[0]` with
//! `t[0] = 1/a` builds exactly that sequence.
//!
//! # Precondition
//!
//! `a != 0`. Violating this is a genuine user error — `1/(a+x)` is not
//! analytic at `a = 0`. Enforced with `assert!` (active in release per D-11)
//! to surface the failure loudly rather than silently producing
//! `t[0] = +inf`.

/// Fill `t[0..=N]` with Taylor coefficients of `1/(a+x)` where `N = t.len() - 1`.
///
/// Writes into caller-provided slice; no heap allocation.
///
/// # Panics
///
/// Panics if `a == 0.0`. `1/(a+x)` is not real analytic at `a = 0`.
pub fn inv_expand(t: &mut [f64], a: f64) {
    // tmath.hpp:125: assert(a != 0 && "1/(a+x) not analytic at a = 0");
    assert!(a != 0.0, "1/(a+x) not analytic at a = 0");

    // tmath.hpp:126: t[0] = 1 / a;
    t[0] = 1.0 / a;

    // tmath.hpp:127-128: for (int i = 1; i <= N; i++) t[i] = -t[i - 1] * t[0];
    // D-08: explicit intermediate `step` mirrors the C++ expression tree so
    // compiler traversal order is deterministic.
    for i in 1..t.len() {
        let step = -t[i - 1];
        t[i] = step * t[0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// N=0 degenerate case: slice len=1, `inv_expand(&mut t, 2.0)` yields `t == [0.5]`.
    #[test]
    fn inv_expand_n0() {
        let mut t = [0.0_f64; 1];
        inv_expand(&mut t, 2.0);
        assert_eq!(t[0].to_bits(), 0.5_f64.to_bits(), "t[0] must be 1/2 = 0.5");
    }

    /// N=3 bit-equal check: `inv_expand(&mut t, 2.0)` yields
    /// `[0.5, -0.25, 0.125, -0.0625]`. All powers of 2 are exact in f64.
    #[test]
    fn inv_expand_n3() {
        let mut t = [0.0_f64; 4];
        inv_expand(&mut t, 2.0);
        let expected = [0.5_f64, -0.25, 0.125, -0.0625];
        for i in 0..4 {
            assert_eq!(
                t[i].to_bits(),
                expected[i].to_bits(),
                "t[{i}] bit-mismatch: got {} expected {}",
                t[i],
                expected[i]
            );
        }
    }

    /// Precondition `a != 0` enforced via `assert!` (active in release, D-11).
    #[test]
    #[should_panic(expected = "1/(a+x) not analytic at a = 0")]
    fn inv_expand_panics_on_zero() {
        let mut t = [0.0_f64; 4];
        inv_expand(&mut t, 0.0);
    }

    /// Negative `a` is fine — `a != 0` is the only precondition.
    #[test]
    fn inv_expand_negative_a() {
        let mut t = [0.0_f64; 2];
        inv_expand(&mut t, -2.0);
        // t[0] = 1/-2 = -0.5; t[1] = -t[0] * t[0] = -(-0.5)*(-0.5) = -0.25
        assert_eq!(t[0].to_bits(), (-0.5_f64).to_bits());
        assert_eq!(t[1].to_bits(), (-0.25_f64).to_bits());
    }
}
