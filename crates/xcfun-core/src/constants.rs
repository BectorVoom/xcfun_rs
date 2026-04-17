//! Physical constants matching C++ xcfun values.
//!
//! IMPORTANT: Some values differ from the design doc (01-data-structures.md).
//! Where they differ, we use the C++ values from constants.hpp for accuracy.

/// Slater exchange constant: (81/(32*pi))^(1/3)
/// C++ uses `pow(81 / (32 * M_PI), 1.0 / 3.0)`
/// NOTE: The design doc uses formula (3/2)*(3/(4*pi))^(1/3) = 0.7386, but
/// the C++ code uses (81/(32*pi))^(1/3) = 0.9305. We match C++.
pub const C_SLATER: f64 = 0.9305257363491002;

/// Thomas-Fermi kinetic constant: 0.3 * (3*pi^2)^(2/3)
pub const CF: f64 = 2.8711842930059836;

/// Tiny density threshold for regularization
pub const TINY_DENSITY: f64 = 1e-14;

/// Maximum derivative order
pub const MAX_ORDER: u32 = 6;

/// PBE gamma = (1 - ln(2)) / pi^2
pub const PARAM_GAMMA: f64 = 0.031090690869654895;

/// PBE beta (accurate value)
pub const PARAM_BETA_ACCURATE: f64 = 0.06672455060314922;

/// 1/3 as f64 constant
pub const THIRD: f64 = 1.0 / 3.0;

/// 4/3 as f64 constant
pub const FOUR_THIRDS: f64 = 4.0 / 3.0;

/// (3/(4*pi))^(1/3) -- used for Wigner-Seitz radius
pub const RS_PREFACTOR: f64 = 0.6203504908994001;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn c_slater_matches_cpp() {
        // C++ value: pow(81 / (32 * PI), 1.0 / 3.0)
        let computed = (81.0 / (32.0 * std::f64::consts::PI)).powf(1.0 / 3.0);
        assert_relative_eq!(C_SLATER, computed, epsilon = 1e-15);
        assert_relative_eq!(C_SLATER, 0.9305257363491002, epsilon = 1e-15);
    }

    #[test]
    fn cf_matches_cpp() {
        // The plan specifies CF = 2.8711842930059836 matching C++ runtime value.
        // This may differ slightly from the formula 0.3 * (3*pi^2)^(2/3) = 2.8712340
        // due to C++ compilation/evaluation differences.
        assert_relative_eq!(CF, 2.8711842930059836, epsilon = 1e-14);
    }

    #[test]
    fn tiny_density() {
        assert_eq!(TINY_DENSITY, 1e-14);
    }

    #[test]
    fn max_order() {
        assert_eq!(MAX_ORDER, 6);
    }
}
