//! Slater LDA exchange functional.

use xcfun_ad::Num;
use xcfun_core::constants;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

/// Slater LDA exchange functional (Dirac 1930, Bloch 1929).
pub struct SlaterX;

impl Functional for SlaterX {
    fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
        T::from_f64(-constants::C_SLATER) * (d.a_43.clone() + d.b_43.clone())
    }

    fn depends(&self) -> Dependency {
        Dependency::DENSITY
    }

    fn id(&self) -> FunctionalId {
        FunctionalId::SlaterX
    }

    fn description(&self) -> &'static str {
        "Slater LDA exchange"
    }

    fn long_description(&self) -> &'static str {
        "LDA Exchange functional\n\
         P.A.M. Dirac, Proceedings of the Cambridge Philosophical \
         Society, 26 (1930) 376.\n\
         F. Bloch, Zeitschrift fuer Physik, 57 (1929) 545.\n\n\
         Implemented by Ulf Ekstrom\n\
         Test case from http://www.cse.scitech.ac.uk/ccg/dft/data_pt_x_lda.html\n"
    }

    fn test_data(&self) -> TestData {
        TestData {
            vars: VarType::A_B,
            mode: EvalMode::PartialDerivatives,
            order: 2,
            threshold: 1e-11,
            input: &[0.39e+02, 0.38e+02],
            expected_output: &[
                -0.241948147838e+03,
                -0.420747936684e+01,
                -0.417120618800e+01,
                -0.359613621097e-01,
                0.0,
                -0.365895279649e-01,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn slaterx_energy_matches_cpp() {
        let input: Vec<f64> = vec![39.0, 38.0];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        let energy = SlaterX.energy(&dv);
        assert_relative_eq!(energy, -0.241948147838e+03, max_relative = 1e-11);
    }

    #[test]
    fn slaterx_depends_density() {
        assert_eq!(SlaterX.depends(), Dependency::DENSITY);
    }
}
