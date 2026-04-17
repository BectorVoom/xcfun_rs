//! VWN5 LDA correlation functional.
//!
//! Placeholder -- will be fully implemented in Task 2.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// VWN5 LDA correlation functional (Vosko-Wilk-Nusair, parameterization V).
pub struct Vwn5C;

impl Functional for Vwn5C {
    fn energy<T: Num>(&self, _d: &DensityVars<T>) -> T {
        T::zero() // placeholder
    }

    fn depends(&self) -> Dependency {
        Dependency::DENSITY
    }

    fn id(&self) -> FunctionalId {
        FunctionalId::Vwn5C
    }

    fn description(&self) -> &'static str {
        "VWN5 LDA correlation"
    }

    fn long_description(&self) -> &'static str {
        "VWN5 LDA Correlation functional\n\
         S.H. Vosko, L. Wilk, and M. Nusair: Accurate spin-dependent\n\
         electron liquid correlation energies for local spin density\n\
         calculations: a critical analysis, Can. J. Phys. 58 (1980) 1200-1211.\n\
         Originally from Dalton, polished and converted by Ulf Ekstrom.\n\
         Test case from http://www.cse.scitech.ac.uk/ccg/dft/data_pt_c_vwn5.html\n"
    }

    fn test_data(&self) -> TestData {
        TestData {
            vars: VarType::A_B,
            mode: EvalMode::PartialDerivatives,
            order: 2,
            threshold: 1e-11,
            input: &[0.39e+02, 0.38e+02],
            expected_output: &[
                -0.851077910672e+01,
                -0.119099058995e+00,
                -0.120906044904e+00,
                0.756836181702e-03,
                -0.102861281830e-02,
                0.800136175083e-03,
            ],
        }
    }
}
