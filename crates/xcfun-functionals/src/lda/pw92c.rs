//! PW92 LDA correlation functional.
//!
//! Placeholder -- will be fully implemented in Task 2.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// PW92 LDA correlation functional (Perdew-Wang 1992).
pub struct Pw92C;

impl Functional for Pw92C {
    fn energy<T: Num>(&self, _d: &DensityVars<T>) -> T {
        T::zero() // placeholder
    }

    fn depends(&self) -> Dependency {
        Dependency::DENSITY
    }

    fn id(&self) -> FunctionalId {
        FunctionalId::Pw92C
    }

    fn description(&self) -> &'static str {
        "PW92 LDA correlation"
    }

    fn long_description(&self) -> &'static str {
        "Accurate and simple analytic representation of the \
         electron-gas correlation energy\n\
         J.P.Perdew, Y. Wang; Phys. Rev. B; 45, 13244, (1992)\n\
         Implemented by Ulf Ekstrom. Some parameters have higher\n\
         accuracy than given in the paper.\n"
    }

    fn test_data(&self) -> TestData {
        TestData {
            vars: VarType::A_B,
            mode: EvalMode::PartialDerivatives,
            order: 2,
            threshold: 1e-11,
            input: &[0.39e+02, 0.38e+02],
            expected_output: &[
                -8.4713855882783946e+00,
                -1.1861930857502517e-01,
                -1.2041769989725633e-01,
                7.5202855619095870e-04,
                -1.0249091426230799e-03,
                7.9516089195232130e-04,
            ],
        }
    }
}
