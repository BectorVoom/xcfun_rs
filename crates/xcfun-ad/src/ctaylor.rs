//! `CTaylor<T, const N: usize, const SIZE: usize>` — bit-flag-indexed
//! multilinear Taylor polynomial.
//!
//! Port target: `xcfun-master/external/upstream/taylor/ctaylor.hpp` (lines
//! 154–337 for struct header + constructors + elementwise ops).
//!
//! Design references:
//! - CONTEXT.md D-01: `[T; 1 << N]` storage, stack-only, `#[repr(C)]`
//! - CONTEXT.md D-02: `ValidN<N, SIZE>` sealed trait pins `N ∈ 0..=7` and
//!   `SIZE = 1 << N` (see `valid_n.rs` for Rule 1+2 stable-Rust deviation
//!   from the plan's single-const-generic API)
//! - CONTEXT.md D-03: `Copy + Clone + Debug + PartialEq` derived; no
//!   `Hash`, no `Eq`
//! - CONTEXT.md D-08: explicit `let` bindings preserve operation order —
//!   no fluent `.iter().zip()` chains when the C++ source uses an indexed
//!   `for` loop
//! - CONTEXT.md D-11: preconditions use `assert!` (active in release),
//!   not `debug_assert!` — catches the silent-NaN failure mode P10
//!
//! Note on `T` bounds: only `T: Copy` here. The richer `T: Num + Copy`
//! bound lives in `num.rs` (Plan 01-05). Keeping `T: Copy` here avoids a
//! module cycle and lets `valid_n` + `ctaylor` stand alone at Wave 0.

use crate::valid_n::{Bound, ValidN};
use core::ops::{Add, Div, Mul, Neg, Sub};

/// Tensored multilinear Taylor polynomial with `SIZE = 2^N` coefficients.
///
/// Coefficients live in `c: [T; SIZE]`, indexed by the bit-flag convention
/// from `docs/design/02-data-structures.md §1`: bit `k` set ⇔ the
/// coefficient carries one power of variable `k`. The "multilinear"
/// property — no variable appears twice in any monomial — is enforced by
/// the indexing (each index is a subset of `{0, …, N-1}` encoded as a
/// bitmask), never by explicit reduction.
///
/// `N` is the number of variables; `SIZE` is the coefficient count. The
/// `ValidN<N, SIZE>` bound on `Bound` pins the relationship
/// `SIZE = 1 << N` at monomorphisation: passing a mismatched pair (e.g.
/// `CTaylor<f64, 3, 16>`) fails to compile because no `ValidN<3, 16>`
/// impl exists.
///
/// For the common case, use the `ct::Nk` type aliases (e.g. `ct::N3<f64>`
/// for a 3-variable, 8-coefficient polynomial).
///
/// Layout mirrors `ctaylor<T, Nvar>` in `ctaylor.hpp:154-156` (`enum { size
/// = POW2(Nvar) }; T c[size];`) under `#[repr(C)]`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CTaylor<T: Copy, const N: usize, const SIZE: usize>
where
    Bound: ValidN<N, SIZE>,
{
    /// Bit-flag-indexed coefficient array. `c[0]` is the constant term
    /// (`CNST`); `c[1 << k]` is the first-order coefficient of variable
    /// `k`.
    pub c: [T; SIZE],
}

impl<const N: usize, const SIZE: usize> CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    /// All-zero coefficient array — useful for test scaffolding and as
    /// the additive identity. Const so downstream tests can compare
    /// against it without constructing a new array.
    pub const ZERO_ARRAY: [f64; SIZE] = [0.0_f64; SIZE];

    /// Constant (variable-free) polynomial. Mirrors `ctaylor(const T & c0)`
    /// at `ctaylor.hpp:179-187`.
    ///
    /// Places `c0` at `c[CNST]` and zero elsewhere.
    #[inline]
    pub fn from_scalar(c0: f64) -> Self {
        let mut c = [0.0_f64; SIZE];
        c[0] = c0;
        Self { c }
    }

    /// Polynomial seeded at variable slot `var`. Mirrors
    /// `ctaylor(const T & c0, int var)` at `ctaylor.hpp:188-198`.
    ///
    /// Places `c0` at `c[CNST]` and `1.0` at `c[var]`, zero elsewhere.
    ///
    /// # Preconditions (active in release — D-11)
    /// - `var` has exactly one bit set (`var.count_ones() == 1`)
    /// - that bit is `< SIZE` (so `var` is in range `[1, SIZE-1]`)
    ///
    /// Violating either precondition panics — both are `assert!` (not
    /// `debug_assert!`) so a release-mode caller passing e.g. `var = 0`
    /// (no bit set, `CNST`) or `var = SIZE` (out of range) fails loudly
    /// rather than silently corrupting coefficients.
    #[inline]
    pub fn from_variable(c0: f64, var: usize) -> Self {
        assert!(
            var.count_ones() == 1,
            "from_variable: var must have exactly one bit set (one variable only), got {var:#b}"
        );
        assert!(
            var < SIZE,
            "from_variable: var bit {var} out of range for SIZE={SIZE} (max index {})",
            SIZE - 1
        );
        let mut c = [0.0_f64; SIZE];
        c[0] = c0;
        c[var] = 1.0;
        Self { c }
    }
}

// ---------------------------------------------------------------------------
// Elementwise Add / Sub / Neg (ctaylor.hpp:263-311)
//
// Implemented only for `CTaylor<f64, N, SIZE>` (not generic over `T`) so the
// operation order is literally a `for i in 0..SIZE` indexed loop, matching
// the C++ `for (int i = 0; i < POW2(Nvar); i++)` form byte-for-byte. D-08
// mandates deterministic sequencing; fluent `.iter().zip()` chains are
// intentionally avoided here.
// ---------------------------------------------------------------------------

impl<const N: usize, const SIZE: usize> Add for CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        // ctaylor.hpp:308-309: `for (int i = 0; i < POW2(Nvar); i++) c[i] += t.c[i];`
        let mut c = [0.0_f64; SIZE];
        for i in 0..SIZE {
            c[i] = self.c[i] + rhs.c[i];
        }
        Self { c }
    }
}

impl<const N: usize, const SIZE: usize> Sub for CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        // ctaylor.hpp:291-292
        let mut c = [0.0_f64; SIZE];
        for i in 0..SIZE {
            c[i] = self.c[i] - rhs.c[i];
        }
        Self { c }
    }
}

impl<const N: usize, const SIZE: usize> Neg for CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        // ctaylor.hpp:273-274
        let mut c = [0.0_f64; SIZE];
        for i in 0..SIZE {
            c[i] = -self.c[i];
        }
        Self { c }
    }
}

// ---------------------------------------------------------------------------
// Scalar Mul / Div (ctaylor.hpp:314-337, 421-451)
//
// `CTaylor<f64, N, SIZE> * f64` and `CTaylor<f64, N, SIZE> / f64`. Division
// precomputes `let inv = 1.0 / rhs;` then multiplies — matches
// `ctaylor.hpp:326-337` where C++ also uses `*= 1/rhs` via
// `operator/=(const S & scale)`.
// ---------------------------------------------------------------------------

impl<const N: usize, const SIZE: usize> Mul<f64> for CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        // ctaylor.hpp:438-450: `operator*(const ctaylor &, const S &)` — elementwise.
        let mut c = [0.0_f64; SIZE];
        for i in 0..SIZE {
            c[i] = self.c[i] * rhs;
        }
        Self { c }
    }
}

impl<const N: usize, const SIZE: usize> Div<f64> for CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self {
        // ctaylor.hpp:326-337: `operator/=(const S & scale)` multiplies by 1/scale.
        // Explicit intermediate `inv` preserves D-08 (operation order).
        let inv = 1.0_f64 / rhs;
        let mut c = [0.0_f64; SIZE];
        for i in 0..SIZE {
            c[i] = self.c[i] * inv;
        }
        Self { c }
    }
}

/// Ergonomic type aliases for the 8 valid `(N, SIZE = 2^N)` pairs.
///
/// Usage: `ct::N3<f64>` is a 3-variable, 8-coefficient polynomial —
/// equivalent to `CTaylor<f64, 3, 8>`. Consumers who prefer the explicit
/// pair form can of course write `CTaylor<f64, 3, 8>` directly.
pub mod ct {
    use super::CTaylor;

    /// 0-variable (scalar-only) polynomial: `[T; 1]`.
    pub type N0<T> = CTaylor<T, 0, 1>;
    /// 1-variable polynomial: `[T; 2]`.
    pub type N1<T> = CTaylor<T, 1, 2>;
    /// 2-variable polynomial: `[T; 4]`.
    pub type N2<T> = CTaylor<T, 2, 4>;
    /// 3-variable polynomial: `[T; 8]`.
    pub type N3<T> = CTaylor<T, 3, 8>;
    /// 4-variable polynomial: `[T; 16]`.
    pub type N4<T> = CTaylor<T, 4, 16>;
    /// 5-variable polynomial: `[T; 32]`.
    pub type N5<T> = CTaylor<T, 5, 32>;
    /// 6-variable polynomial: `[T; 64]`.
    pub type N6<T> = CTaylor<T, 6, 64>;
    /// 7-variable polynomial: `[T; 128]`.
    pub type N7<T> = CTaylor<T, 7, 128>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CNST, VAR0, VAR1};

    // -----------------------------------------------------------------------
    // Layout tests: size_of<CTaylor<f64, N, SIZE>>() == 8 * SIZE for each N
    // -----------------------------------------------------------------------

    #[test]
    fn ctaylor_layout_n0() {
        let t: ct::N0<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 1);
        assert_eq!(core::mem::size_of_val(&t), 8 * 1);
    }

    #[test]
    fn ctaylor_layout_n1() {
        let t: ct::N1<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 2);
        assert_eq!(core::mem::size_of_val(&t), 8 * 2);
    }

    #[test]
    fn ctaylor_layout_n2() {
        let t: ct::N2<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 4);
        assert_eq!(core::mem::size_of_val(&t), 8 * 4);
    }

    #[test]
    fn ctaylor_layout_n3() {
        let t: ct::N3<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 8);
        assert_eq!(core::mem::size_of_val(&t), 8 * 8);
    }

    #[test]
    fn ctaylor_layout_n4() {
        let t: ct::N4<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 16);
        assert_eq!(core::mem::size_of_val(&t), 8 * 16);
    }

    #[test]
    fn ctaylor_layout_n5() {
        let t: ct::N5<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 32);
        assert_eq!(core::mem::size_of_val(&t), 8 * 32);
    }

    #[test]
    fn ctaylor_layout_n6() {
        let t: ct::N6<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 64);
        assert_eq!(core::mem::size_of_val(&t), 8 * 64);
    }

    #[test]
    fn ctaylor_layout_n7() {
        let t: ct::N7<f64> = CTaylor::from_scalar(0.0);
        assert_eq!(t.c.len(), 128);
        assert_eq!(core::mem::size_of_val(&t), 8 * 128);
    }

    // -----------------------------------------------------------------------
    // ZERO_ARRAY helper
    // -----------------------------------------------------------------------

    #[test]
    fn zero_array_helper_n2() {
        assert_eq!(ct::N2::<f64>::ZERO_ARRAY, [0.0, 0.0, 0.0, 0.0]);
    }

    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    #[test]
    fn from_scalar_sets_only_cnst() {
        let t = ct::N3::<f64>::from_scalar(5.0);
        assert_eq!(t.c, [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(t.c[CNST], 5.0);
    }

    #[test]
    fn from_variable_sets_cnst_and_var() {
        let t = ct::N2::<f64>::from_variable(3.0, VAR0);
        assert_eq!(t.c, [3.0, 1.0, 0.0, 0.0]);
        assert_eq!(t.c[CNST], 3.0);
        assert_eq!(t.c[VAR0], 1.0);
    }

    #[test]
    fn from_variable_seeds_var1() {
        let t = ct::N2::<f64>::from_variable(7.0, VAR1);
        assert_eq!(t.c, [7.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    #[should_panic(expected = "var must have exactly one bit set")]
    fn from_variable_rejects_cnst() {
        // var = 0 has zero bits set → CNST is not a variable slot.
        let _ = ct::N2::<f64>::from_variable(1.0, 0);
    }

    #[test]
    #[should_panic(expected = "var must have exactly one bit set")]
    fn from_variable_rejects_multi_bit() {
        // var = 0b11 = VAR0 | VAR1 has two bits set → not a single-variable seed.
        let _ = ct::N2::<f64>::from_variable(1.0, VAR0 | VAR1);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn from_variable_rejects_out_of_range() {
        // var = 1 << 2 = 4 is out of range for SIZE = 4 (max index = 3).
        let _ = ct::N2::<f64>::from_variable(1.0, 1 << 2);
    }

    // -----------------------------------------------------------------------
    // Elementwise Add / Sub / Neg
    // -----------------------------------------------------------------------

    #[test]
    fn add_is_elementwise() {
        let a = ct::N2::<f64> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let b = ct::N2::<f64> {
            c: [10.0, 20.0, 30.0, 40.0],
        };
        let s = a + b;
        for i in 0..4 {
            assert_eq!(s.c[i], a.c[i] + b.c[i], "add mismatch at i={i}");
        }
        assert_eq!(s.c, [11.0, 22.0, 33.0, 44.0]);
    }

    #[test]
    fn sub_is_elementwise() {
        let a = ct::N2::<f64> {
            c: [10.0, 20.0, 30.0, 40.0],
        };
        let b = ct::N2::<f64> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let d = a - b;
        for i in 0..4 {
            assert_eq!(d.c[i], a.c[i] - b.c[i], "sub mismatch at i={i}");
        }
        assert_eq!(d.c, [9.0, 18.0, 27.0, 36.0]);
    }

    #[test]
    fn neg_is_elementwise() {
        let a = ct::N2::<f64> {
            c: [1.0, -2.0, 3.0, -4.0],
        };
        let n = -a;
        for i in 0..4 {
            assert_eq!(n.c[i], -a.c[i], "neg mismatch at i={i}");
        }
        assert_eq!(n.c, [-1.0, 2.0, -3.0, 4.0]);
    }

    // -----------------------------------------------------------------------
    // Scalar Mul / Div
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_mul_is_elementwise() {
        let t = ct::N1::<f64> { c: [2.0, 3.0] };
        let r = t * 4.0;
        assert_eq!(r.c, [8.0, 12.0]);
    }

    #[test]
    fn scalar_div_is_elementwise() {
        let t = ct::N1::<f64> { c: [8.0, 12.0] };
        let r = t / 4.0;
        assert_eq!(r.c, [2.0, 3.0]);
    }

    // -----------------------------------------------------------------------
    // Copy semantics (D-03)
    // -----------------------------------------------------------------------

    #[test]
    fn copy_semantics() {
        let a = ct::N1::<f64>::from_scalar(1.0);
        let _b = a; // move-by-copy (CTaylor is Copy)
        let _c = a; // compiles because a wasn't actually consumed
        assert_eq!(a.c[0], 1.0);
    }

    // -----------------------------------------------------------------------
    // PartialEq / Debug derives work
    // -----------------------------------------------------------------------

    #[test]
    fn partial_eq_works() {
        let a = ct::N1::<f64>::from_scalar(5.0);
        let b = ct::N1::<f64>::from_scalar(5.0);
        let d = ct::N1::<f64>::from_scalar(6.0);
        assert_eq!(a, b);
        assert_ne!(a, d);
    }

    #[test]
    fn debug_impl_works() {
        // Just ensure Debug compiles and produces non-empty output.
        let t = ct::N1::<f64>::from_scalar(3.14);
        let s = format!("{t:?}");
        assert!(!s.is_empty());
    }

    // -----------------------------------------------------------------------
    // Explicit-pair form also works
    // -----------------------------------------------------------------------

    #[test]
    fn explicit_pair_form_compiles() {
        let t: CTaylor<f64, 3, 8> = CTaylor::from_scalar(2.71);
        assert_eq!(t.c[0], 2.71);
        assert_eq!(t.c.len(), 8);
    }
}
