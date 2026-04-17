//! VWN5 LDA correlation functional.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// VWN5 LDA correlation functional (Vosko-Wilk-Nusair, parameterization V).
pub struct Vwn5C;

impl Functional for Vwn5C {
    fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
        // Port of C++ vwn::vwn5_eps(d) * d.n
        let s = d.r_s.clone().sqrt();

        let eps_para = helpers::vwn_f(&s, &helpers::VWN5_PARA);
        let eps_ferro = helpers::vwn_f(&s, &helpers::VWN5_FERRO);
        let eps_inter = helpers::vwn_f(&s, &helpers::VWN5_INTER);

        // Constant is (2^(1/3)-1)^(-1) * (9/4) -- simplifies to 1.92366105093154
        let g = T::from_f64(1.92366105093154)
            * (helpers::ufunc(&d.zeta, 4.0 / 3.0) - T::from_f64(2.0));

        let zeta4 = d.zeta.clone().powi(4);

        // C++: g * ((vwn_f(s, ferro) - vwn_f(s, para)) * zeta4
        //        + vwn_f(s, inter) * (1 - zeta4) * (9/4*(2^(1/3)-1)))
        let one_minus_zeta4 = T::one() - zeta4.clone();
        // 9/4 * (2^(1/3) - 1) -- this is the inverse of 1.92366105093154 * 2
        let inter_factor = 9.0 / 4.0 * (2.0_f64.powf(1.0 / 3.0) - 1.0);

        let dd = g
            * ((eps_ferro - eps_para.clone()) * zeta4
                + eps_inter * one_minus_zeta4 * T::from_f64(inter_factor));

        let eps = eps_para + dd;
        d.n.clone() * eps
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn vwn5c_energy_matches_cpp() {
        let input: Vec<f64> = vec![39.0, 38.0];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        let energy = Vwn5C.energy(&dv);
        assert_relative_eq!(energy, -0.851077910672e+01, epsilon = 1e-11);
    }

    #[test]
    fn vwn5c_depends_density() {
        assert_eq!(Vwn5C.depends(), Dependency::DENSITY);
    }
}
