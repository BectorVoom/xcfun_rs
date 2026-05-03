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
    ///
    /// `ERF` (= 32) is a Rust-side extension introduced in Phase 6 Plan 06-02a
    /// (Rule 2 deviation). It is NOT a `XC_*` bit in the upstream C++ header
    /// — upstream xcfun encodes ERF-bearing functionals at the FunctionalId
    /// level (`ldaerfx`, `ldaerfc`, ...) — but Plans 06-02a and 06-05 need
    /// a Dependency-level signal so `xcfun-gpu::error_routing::
    /// must_fall_back_to_cpu` can decide whether a given functional set
    /// requires the CPU substrate on Wgpu/Metal (where WGSL has no f64
    /// `erf` support — see RESEARCH §"Pitfall 5"). The bit is only set
    /// for the range-separated functionals (`ldaerfx`, `ldaerfc`,
    /// `beckecamx`, `beckesrx`, `ldaerfc_jt`); all other functionals
    /// continue to report the upstream-aligned 5-bit set.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Dependency: u32 {
        const DENSITY   = 0b0000_0001;
        const GRADIENT  = 0b0000_0010;
        const LAPLACIAN = 0b0000_0100;
        const KINETIC   = 0b0000_1000;
        const JP        = 0b0001_0000;
        const ERF       = 0b0010_0000;
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
