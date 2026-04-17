//! Functional trait, Dependency bitflags, and TestData.

use crate::density_vars::DensityVars;
use crate::enums::{EvalMode, VarType};
use crate::functional_id::FunctionalId;

bitflags::bitflags! {
    /// Dependency flags indicating which input quantities a functional requires.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Dependency: u32 {
        const DENSITY   = 0b0000_0001;
        const GRADIENT  = 0b0000_0010;
        const LAPLACIAN = 0b0000_0100;
        const KINETIC   = 0b0000_1000;
        const JP        = 0b0001_0000;
    }
}

/// Reference data for validating a functional implementation against xcfun C++.
pub struct TestData {
    pub vars: VarType,
    pub mode: EvalMode,
    pub order: u32,
    pub threshold: f64,
    pub input: &'static [f64],
    pub expected_output: &'static [f64],
}

/// A single exchange-correlation energy functional.
///
/// Implementors compute E_xc as a function of density variables.
/// The generic parameter `T: Num` enables automatic differentiation:
/// when `T = f64`, only the energy is computed; when `T = CTaylor<f64, N>`,
/// all partial derivatives up to order N are computed simultaneously.
pub trait Functional: Send + Sync {
    /// Compute the exchange-correlation energy density.
    fn energy<T: xcfun_ad::Num>(&self, vars: &DensityVars<T>) -> T;

    /// Dependency flags indicating which input quantities this functional requires.
    fn depends(&self) -> Dependency;

    /// Unique identifier for this functional.
    fn id(&self) -> FunctionalId;

    /// Short human-readable description.
    fn description(&self) -> &'static str;

    /// Long description with references.
    fn long_description(&self) -> &'static str;

    /// Test data for accuracy validation.
    fn test_data(&self) -> TestData;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_bits() {
        assert_eq!(Dependency::DENSITY.bits(), 1);
        assert_eq!(Dependency::GRADIENT.bits(), 2);
        assert_eq!(Dependency::LAPLACIAN.bits(), 4);
        assert_eq!(Dependency::KINETIC.bits(), 8);
        assert_eq!(Dependency::JP.bits(), 16);
    }

    #[test]
    fn dependency_bitwise_operations() {
        let combined = Dependency::DENSITY | Dependency::GRADIENT;
        assert!(combined.contains(Dependency::DENSITY));
        assert!(combined.contains(Dependency::GRADIENT));
        assert!(!combined.contains(Dependency::LAPLACIAN));
    }
}
