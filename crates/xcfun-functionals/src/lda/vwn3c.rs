//! VWN3 LDA correlation functional.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// VWN3 LDA correlation functional (Vosko-Wilk-Nusair, parameterization III).
pub struct Vwn3C;

impl Functional for Vwn3C {
    fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
        // Port of C++ vwn::vwn3_eps(d) * d.n
        let s = d.r_s.clone().sqrt();

        let eps_para = helpers::vwn_f(&s, &helpers::VWN3_PARA);
        let eps_ferro = helpers::vwn_f(&s, &helpers::VWN3_FERRO);

        // Constant is (2^(1/3)-1)^(-1) * (9/4) = 1.92366105093154
        let g = T::from_f64(1.92366105093154)
            * (helpers::ufunc(&d.zeta, 4.0 / 3.0) - T::from_f64(2.0));

        // VWN3 is simpler than VWN5: no inter term, no zeta4
        let dd = g * (eps_ferro - eps_para.clone());
        let eps = eps_para + dd;
        d.n.clone() * eps
    }

    fn depends(&self) -> Dependency {
        Dependency::DENSITY
    }

    fn id(&self) -> FunctionalId {
        FunctionalId::Vwn3C
    }

    fn description(&self) -> &'static str {
        "VWN3 LDA correlation"
    }

    fn long_description(&self) -> &'static str {
        "VWN3 LDA Correlation functional\n\
         S.H. Vosko, L. Wilk, and M. Nusair: Accurate spin-dependent\n\
         electron liquid correlation energies for local spin density\n\
         calculations: a critical analysis, Can. J. Phys. 58 (1980) 1200-1211.\n\
         Originally from Dalton, polished and converted by Ulf Ekstrom.\n"
    }

    fn test_data(&self) -> TestData {
        // VWN3 has no built-in C++ test data; use self-consistency reference
        TestData {
            vars: VarType::A_B,
            mode: EvalMode::PartialDerivatives,
            order: 0,
            threshold: 1e-14,
            input: &[39.0, 38.0],
            expected_output: &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Compute VWN3 energy using f64 arithmetic directly (self-consistency test).
    fn vwn3_energy_f64(input: &[f64]) -> f64 {
        let dv = DensityVars::from_input(input, VarType::A_B).unwrap();
        // Manually compute via the same helpers with f64
        let s = dv.r_s.sqrt();
        let eps_para = helpers::vwn_f(&s, &helpers::VWN3_PARA);
        let eps_ferro = helpers::vwn_f(&s, &helpers::VWN3_FERRO);
        let g = 1.92366105093154 * (helpers::ufunc(&dv.zeta, 4.0 / 3.0) - 2.0);
        let dd = g * (eps_ferro - eps_para);
        let eps = helpers::vwn_f(&s, &helpers::VWN3_PARA) + dd;
        dv.n * eps
    }

    #[test]
    fn vwn3c_energy_self_consistency() {
        let input = vec![39.0_f64, 38.0];
        let expected = vwn3_energy_f64(&input);
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        let energy = Vwn3C.energy(&dv);
        // f64 path and Num trait path must agree within 1e-14
        assert_relative_eq!(energy, expected, epsilon = 1e-14);
    }

    #[test]
    fn vwn3c_differs_from_vwn5c() {
        let input = vec![39.0_f64, 38.0];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        let vwn3_energy = Vwn3C.energy(&dv);
        let vwn5_energy = super::super::vwn5c::Vwn5C.energy(&dv);
        // VWN3 and VWN5 use different parameter sets, so they must differ
        assert!((vwn3_energy - vwn5_energy).abs() > 1e-6);
    }

    #[test]
    fn vwn3c_depends_density() {
        assert_eq!(Vwn3C.depends(), Dependency::DENSITY);
    }
}
