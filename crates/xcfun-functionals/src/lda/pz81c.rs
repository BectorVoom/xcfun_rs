//! PZ81 LDA correlation functional.
//!
//! Placeholder -- will be fully implemented in Task 2.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// PZ81 LDA correlation functional (Perdew-Zunger 1981).
pub struct Pz81C;

impl Functional for Pz81C {
    fn energy<T: Num>(&self, _d: &DensityVars<T>) -> T {
        T::zero() // placeholder
    }

    fn depends(&self) -> Dependency {
        Dependency::DENSITY
    }

    fn id(&self) -> FunctionalId {
        FunctionalId::Pz81C
    }

    fn description(&self) -> &'static str {
        "PZ81 LDA correlation"
    }

    fn long_description(&self) -> &'static str {
        "PZ81 LDA correlation\n\
         Implemented by Ulf Ekstrom. Test from \
         http://www.cse.scitech.ac.uk/ccg/dft/data_pt_c_pz81.html\n"
    }

    fn test_data(&self) -> TestData {
        TestData {
            vars: VarType::A_B,
            mode: EvalMode::PartialDerivatives,
            order: 2,
            threshold: 1e-11,
            input: &[0.48e-01, 0.25e-01],
            expected_output: &[
                -0.358997585489e-02,
                -0.468661877874e-01,
                -0.731782746282e-01,
                0.218577885080e+00,
                -0.646538277526e+00,
                0.867717298846e+00,
            ],
        }
    }
}
