//! xcfun-core: Core types and traits for xcfun_rs.
//!
//! Provides the type system that all downstream crates depend on:
//! - `DensityVars<T>` -- density variable container
//! - `EvalMode`, `VarType` -- evaluation mode and variable specification
//! - `FunctionalId` -- functional identifiers
//! - `Dependency` -- dependency bitflags
//! - `XcError` -- error types
//! - `Functional` trait -- core functional interface

pub mod constants;
pub mod enums;
pub mod error;
pub mod functional_id;
pub mod test_data;
pub mod traits;

pub use constants::*;
pub use enums::{EvalMode, VarType};
pub use error::XcError;
pub use functional_id::FunctionalId;
pub use traits::{Dependency, Functional, TestData};

// Re-export Num from xcfun-ad for convenience (per D-02)
pub use xcfun_ad::Num;

/// Number of elements in a multivariate Taylor expansion.
/// Computes C(n_vars + order, order) iteratively.
///
/// This matches the C++ `taylorlen()` function.
pub const fn taylorlen(n_vars: usize, order: usize) -> usize {
    let mut len: usize = 1;
    let mut k: usize = 1;
    while k <= order {
        len = len * (n_vars + k) / k;
        k += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taylorlen_basic() {
        assert_eq!(taylorlen(1, 0), 1);
        assert_eq!(taylorlen(1, 1), 2);
        assert_eq!(taylorlen(2, 2), 6);
        assert_eq!(taylorlen(5, 1), 6);
    }

    #[test]
    fn taylorlen_larger() {
        assert_eq!(taylorlen(7, 4), 330);
    }
}
