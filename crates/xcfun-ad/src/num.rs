//! Num trait -- numeric abstraction for both f64 and CTaylor.
//!
//! Placeholder module; full implementation in Task 2.

/// Marker trait for numeric types used in xcfun.
/// Full definition will be added in Task 2.
pub trait Num:
    Clone
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + Sized
{
    fn from_f64(val: f64) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

impl Num for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}
