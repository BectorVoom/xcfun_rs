//! `ValidN<N, SIZE>` sealed trait — stable-Rust encoding of
//! "`SIZE == 1 << N` for `N ∈ 0..=7`".
//!
//! # Rule 1+2 deviation from plan (recorded in 01-01-SUMMARY.md)
//!
//! The plan's `<interfaces>` block specified `trait ValidN<const N: usize>`
//! bounding a struct `CTaylor<T, const N: usize>` with `pub c: [T; 1 << N]`.
//! That API is **not expressible on stable Rust 1.85** — the expression
//! `[T; 1 << N]` uses the const parameter `N` in a const operation, which
//! the stable compiler rejects with `E0799` ("generic parameters may not be
//! used in const operations"). The feature that enables it
//! (`generic_const_exprs`) is nightly-only, and the project's CLAUDE.md
//! pins the stable channel ("No nightly features reachable").
//!
//! Minimal viable stable encoding: **two const generics**, `N` (number of
//! variables) and `SIZE` (coefficient count), tied together by the sealed
//! trait `ValidN<N, SIZE>` which is implemented exactly 8 times — once per
//! `(N, 2^N)` pair. A user writes `CTaylor<f64, 3, 8>` directly, or uses
//! the `ct::N0..N7` type aliases for the common case.
//!
//! This preserves:
//! - CONTEXT.md D-01 (stack-only `[T; SIZE]` storage where `SIZE = 1 << N`)
//! - CONTEXT.md D-02 (`N ≤ 7` enforced at monomorphisation — a
//!   `ValidN<8, 256>` bound is not satisfied since no such impl exists)
//! - CONTEXT.md D-03 (`Copy + Clone + Debug + PartialEq` derives survive)
//! - Bit-flag indexing (`CNST = 0`, `VAR_k = 1 << k`) unchanged
//! - Sealed-trait guarantee (downstream crates cannot add new impls)
//!
//! The only surface-level change is that `CTaylor` now carries one more
//! const parameter that consumers must either supply explicitly or reach
//! via the `ct::Nk` aliases.
//!
//! # Example
//!
//! ```
//! use xcfun_ad::valid_n::{Bound, ValidN};
//!
//! fn needs_valid<const N: usize, const SIZE: usize>()
//! where Bound: ValidN<N, SIZE> {}
//!
//! needs_valid::<0, 1>();
//! needs_valid::<3, 8>();
//! needs_valid::<7, 128>();
//! // needs_valid::<3, 16>(); // compile error: no `ValidN<3, 16>` impl
//! // needs_valid::<8, 256>(); // compile error: no `ValidN<8, 256>` impl
//! ```

/// Sealed marker trait: `Bound: ValidN<N, SIZE>` holds only for
/// `(N, SIZE) ∈ {(0,1), (1,2), (2,4), (3,8), (4,16), (5,32), (6,64), (7,128)}`.
///
/// Use as `where Bound: ValidN<N, SIZE>` on any type or function generic
/// over both `const N: usize` (variable count) and `const SIZE: usize`
/// (coefficient-array length).
pub trait ValidN<const N: usize, const SIZE: usize>: sealed::Sealed {}

/// Uninhabited-style marker: the only type for which `ValidN<N, SIZE>` is
/// implemented. Downstream code never constructs a `Bound` value — the
/// type exists purely as a trait-bound handle.
pub struct Bound;

mod sealed {
    /// Sealed: outside this crate, no new types can satisfy `Sealed`, so
    /// no new `ValidN<N, SIZE>` implementations can be added for other
    /// types.
    pub trait Sealed {}
}

impl sealed::Sealed for Bound {}

impl ValidN<0, 1> for Bound {}
impl ValidN<1, 2> for Bound {}
impl ValidN<2, 4> for Bound {}
impl ValidN<3, 8> for Bound {}
impl ValidN<4, 16> for Bound {}
impl ValidN<5, 32> for Bound {}
impl ValidN<6, 64> for Bound {}
impl ValidN<7, 128> for Bound {}

#[cfg(test)]
mod tests {
    use super::*;

    fn require_valid_n<const N: usize, const SIZE: usize>()
    where
        Bound: ValidN<N, SIZE>,
    {
    }

    #[test]
    fn valid_n_0_through_7_compile() {
        require_valid_n::<0, 1>();
        require_valid_n::<1, 2>();
        require_valid_n::<2, 4>();
        require_valid_n::<3, 8>();
        require_valid_n::<4, 16>();
        require_valid_n::<5, 32>();
        require_valid_n::<6, 64>();
        require_valid_n::<7, 128>();
    }
}
