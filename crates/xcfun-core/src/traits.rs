//! Dependency bitflags. `Functional` trait + `TestData` struct moved out of
//! xcfun-core per Phase 2 D-04: the functional surface lives in `xcfun-eval`,
//! and per-functional test data lives in `FUNCTIONAL_DESCRIPTORS` (CORE-07 +
//! Phase 2 D-12, populated in Plan 02-02 Wave-1A).

bitflags::bitflags! {
    /// Dependency flags indicating which input quantities a functional requires.
    ///
    /// Bit values match `xcfun-master/src/xcint.hpp:46-50`:
    /// `XC_DENSITY = 1`, `XC_GRADIENT = 2`, `XC_LAPLACIAN = 4`, `XC_KINETIC = 8`,
    /// `XC_JP = 16`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Dependency: u32 {
        const DENSITY   = 0b0000_0001;
        const GRADIENT  = 0b0000_0010;
        const LAPLACIAN = 0b0000_0100;
        const KINETIC   = 0b0000_1000;
        const JP        = 0b0001_0000;
    }
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
