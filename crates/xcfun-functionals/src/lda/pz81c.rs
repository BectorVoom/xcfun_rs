//! PZ81 LDA correlation functional.

use xcfun_ad::Num;
use xcfun_core::density_vars::DensityVars;
use xcfun_core::enums::{EvalMode, VarType};
use xcfun_core::functional_id::FunctionalId;
use xcfun_core::traits::{Dependency, Functional, TestData};

use super::helpers;

/// PZ81 LDA correlation functional (Perdew-Zunger 1981).
pub struct Pz81C;

/// Spin polarization function f(zeta), matching C++ pz81eps::fz(d).
///
/// Formula: (2^(4/3) * (a^(4/3) + b^(4/3)) * n^(-1/3) / n - 2) / (2*2^(1/3) - 2)
fn fz<T: Num>(d: &DensityVars<T>) -> T {
    let p = 2.0_f64.powf(4.0 / 3.0);
    let q = 2.0 * 2.0_f64.powf(1.0 / 3.0) - 2.0;
    (T::from_f64(p) * (d.a_43.clone() + d.b_43.clone()) * d.n_m13.clone() / d.n.clone()
        - T::from_f64(2.0))
        / T::from_f64(q)
}

impl Functional for Pz81C {
    fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
        // Port of C++ pz81eps::pz81eps(d) * d.n
        let fz_val = fz(d);

        // C++: if (1 > d.r_s) -- meaning r_s < 1 (HIGH density)
        let eps = if T::from_f64(1.0).gt(&d.r_s) {
            // r_s < 1: high density path
            let e2 = helpers::pz81_ehd(&d.r_s, &helpers::PZ81_C2);
            let e3 = helpers::pz81_ehd(&d.r_s, &helpers::PZ81_C3);
            e2.clone() + (e3 - e2) * fz_val
        } else {
            // r_s >= 1: low density path
            let e0 = helpers::pz81_eld(&d.r_s, &helpers::PZ81_C0);
            let e1 = helpers::pz81_eld(&d.r_s, &helpers::PZ81_C1);
            e0.clone() + (e1 - e0) * fz_val
        };

        d.n.clone() * eps
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn pz81c_energy_matches_cpp() {
        let input: Vec<f64> = vec![0.048, 0.025];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        let energy = Pz81C.energy(&dv);
        assert_relative_eq!(energy, -0.358997585489e-02, epsilon = 1e-11);
    }

    #[test]
    fn pz81c_depends_density() {
        assert_eq!(Pz81C.depends(), Dependency::DENSITY);
    }
}
