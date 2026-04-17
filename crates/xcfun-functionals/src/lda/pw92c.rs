//! PW92 LDA correlation functional.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// PW92 LDA correlation functional (Perdew-Wang 1992).
pub struct Pw92C;

impl Functional for Pw92C {
    fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
        // Port of C++ pw92eps::pw92eps(d) * d.n
        // Non-XCFUN_REF_PW92C path (exact constants)
        let c: f64 = 8.0 / (9.0 * (2.0 * 2.0_f64.powf(1.0 / 3.0) - 2.0));

        let zeta4 = d.zeta.clone().powi(4);
        let omega_val = helpers::pw92_omega(&d.zeta);
        let sqrt_r = d.r_s.clone().sqrt();

        let e0 = helpers::pw92_eopt(&sqrt_r, &helpers::PW92_TUVWXYP[0]);
        let e1 = helpers::pw92_eopt(&sqrt_r, &helpers::PW92_TUVWXYP[1]);
        let e2 = helpers::pw92_eopt(&sqrt_r, &helpers::PW92_TUVWXYP[2]);

        // C++: e0 - e2 * omegaval * (1 - zeta4) / c + (e1 - e0) * omegaval * zeta4
        let one_minus_zeta4 = T::one() - zeta4.clone();
        let eps = e0.clone()
            - e2 * omega_val.clone() * one_minus_zeta4 / T::from_f64(c)
            + (e1 - e0) * omega_val * zeta4;

        d.n.clone() * eps
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn pw92c_energy_matches_cpp() {
        let input: Vec<f64> = vec![39.0, 38.0];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        let energy = Pw92C.energy(&dv);
        assert_relative_eq!(energy, -8.4713855882783946e+00, epsilon = 1e-11);
    }

    #[test]
    fn pw92c_depends_density() {
        assert_eq!(Pw92C.depends(), Dependency::DENSITY);
    }
}
