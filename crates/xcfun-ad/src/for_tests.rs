//! Testing-only seam. Feature-gated by `testing` (see Cargo.toml); never
//! exposed in default builds.
//!
//! Consumed by downstream crates via
//! `[dev-dependencies] xcfun-ad = { features = ["testing"] }` (CONTEXT.md
//! D-22). Keeps the public default-feature surface small while allowing
//! tests to poke at raw coefficient arrays.

use crate::{
    CTaylor,
    valid_n::{Bound, ValidN},
};

/// Borrow the raw coefficient array of a `CTaylor`. Generic over
/// `T: Copy`. Only available under `feature = "testing"`.
pub fn raw_coeffs<T: Copy, const N: usize, const SIZE: usize>(
    t: &CTaylor<T, N, SIZE>,
) -> &[T; SIZE]
where
    Bound: ValidN<N, SIZE>,
{
    &t.c
}

/// Construct a `CTaylor<f64, N, SIZE>` directly from a coefficient array.
/// Useful in fixture-driven tests where the coefficient layout is given.
/// Only available under `feature = "testing"`.
pub fn from_coeffs<const N: usize, const SIZE: usize>(
    c: [f64; SIZE],
) -> CTaylor<f64, N, SIZE>
where
    Bound: ValidN<N, SIZE>,
{
    CTaylor { c }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctaylor::ct;

    #[test]
    fn raw_coeffs_matches_field() {
        let t = ct::N2::<f64>::from_scalar(7.0);
        assert_eq!(raw_coeffs(&t), &[7.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn from_coeffs_roundtrip() {
        let arr = [1.0, 2.0, 3.0, 4.0];
        let t: ct::N2<f64> = from_coeffs(arr);
        assert_eq!(t.c, arr);
    }
}
