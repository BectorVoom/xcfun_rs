//! EvalMode and VarType enums with metadata methods.

use crate::traits::Dependency;

/// Evaluation mode for functional derivatives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalMode {
    /// Compute partial derivatives up to configured order.
    PartialDerivatives,
    /// Compute exchange-correlation potential.
    Potential,
    /// Input contains pre-computed Taylor coefficients.
    Contracted,
}

/// Input variable specification.
///
/// Ordering matches C++ xcfun.h `xcfun_vars` enum exactly.
/// IMPORTANT: This uses the C++ ordering, NOT the design doc ordering
/// (which differs for gradient-component and meta-GGA variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VarType {
    // LDA
    A = 0,
    N = 1,
    A_B = 2,
    N_S = 3,

    // GGA (squared gradient)
    A_GAA = 4,
    N_GNN = 5,
    A_B_GAA_GAB_GBB = 6,
    N_S_GNN_GNS_GSS = 7,

    // metaGGA (laplacian)
    A_GAA_LAPA = 8,
    A_GAA_TAUA = 9,
    N_GNN_LAPN = 10,
    N_GNN_TAUN = 11,
    A_B_GAA_GAB_GBB_LAPA_LAPB = 12,
    A_B_GAA_GAB_GBB_TAUA_TAUB = 13,
    N_S_GNN_GNS_GSS_LAPN_LAPS = 14,
    N_S_GNN_GNS_GSS_TAUN_TAUS = 15,

    // metaGGA (laplacian + kinetic)
    A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB = 16,

    // metaGGA (with current density)
    A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB = 17,

    // metaGGA (N/S with laplacian + kinetic)
    N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS = 18,

    // GGA (gradient components)
    A_AX_AY_AZ = 19,
    A_B_AX_AY_AZ_BX_BY_BZ = 20,
    N_NX_NY_NZ = 21,
    N_S_NX_NY_NZ_SX_SY_SZ = 22,

    // metaGGA (gradient components + kinetic)
    A_AX_AY_AZ_TAUA = 23,
    A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB = 24,
    N_NX_NY_NZ_TAUN = 25,
    N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS = 26,

    // 2nd order Taylor
    A_2ND_TAYLOR = 27,
    A_B_2ND_TAYLOR = 28,
    N_2ND_TAYLOR = 29,
    N_S_2ND_TAYLOR = 30,
}

impl VarType {
    /// Number of input values required for this variable type.
    pub const fn input_len(&self) -> usize {
        match self {
            // LDA
            VarType::A => 1,
            VarType::N => 1,
            VarType::A_B => 2,
            VarType::N_S => 2,

            // GGA (squared gradient)
            VarType::A_GAA => 2,
            VarType::N_GNN => 2,
            VarType::A_B_GAA_GAB_GBB => 5,
            VarType::N_S_GNN_GNS_GSS => 5,

            // metaGGA (laplacian variants)
            VarType::A_GAA_LAPA => 3,
            VarType::A_GAA_TAUA => 3,
            VarType::N_GNN_LAPN => 3,
            VarType::N_GNN_TAUN => 3,
            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB => 7,
            VarType::A_B_GAA_GAB_GBB_TAUA_TAUB => 7,
            VarType::N_S_GNN_GNS_GSS_LAPN_LAPS => 7,
            VarType::N_S_GNN_GNS_GSS_TAUN_TAUS => 7,

            // metaGGA (laplacian + kinetic)
            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB => 9,

            // metaGGA (with current density)
            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => 11,

            // metaGGA (N/S with laplacian + kinetic)
            VarType::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => 9,

            // GGA (gradient components)
            VarType::A_AX_AY_AZ => 4,
            VarType::A_B_AX_AY_AZ_BX_BY_BZ => 8,
            VarType::N_NX_NY_NZ => 4,
            VarType::N_S_NX_NY_NZ_SX_SY_SZ => 8,

            // metaGGA (gradient components + kinetic)
            VarType::A_AX_AY_AZ_TAUA => 5,
            VarType::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB => 10,
            VarType::N_NX_NY_NZ_TAUN => 5,
            VarType::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => 10,

            // 2nd order Taylor: taylorlen(n_vars, 2) input values
            // A_2ND_TAYLOR: 1 var, order 2 -> C(1+2,2) = 3 ... but wait
            // C++ comment says "1+3+6=10 numbers" for A_2ND_TAYLOR
            // That's actually the 3D spatial Taylor expansion of alpha density
            // taylorlen(3, 2) = C(5,2) = 10
            VarType::A_2ND_TAYLOR => 10,
            // "first alpha, then beta" -> 20 numbers
            VarType::A_B_2ND_TAYLOR => 20,
            VarType::N_2ND_TAYLOR => 10,
            VarType::N_S_2ND_TAYLOR => 20,
        }
    }

    /// Dependency flags provided by this variable type.
    pub const fn provides(&self) -> Dependency {
        match self {
            // LDA: density only
            VarType::A | VarType::N | VarType::A_B | VarType::N_S => Dependency::DENSITY,

            // GGA (squared gradient): density + gradient
            VarType::A_GAA
            | VarType::N_GNN
            | VarType::A_B_GAA_GAB_GBB
            | VarType::N_S_GNN_GNS_GSS => {
                Dependency::DENSITY.union(Dependency::GRADIENT)
            }

            // metaGGA with laplacian: density + gradient + laplacian
            VarType::A_GAA_LAPA
            | VarType::N_GNN_LAPN
            | VarType::A_B_GAA_GAB_GBB_LAPA_LAPB
            | VarType::N_S_GNN_GNS_GSS_LAPN_LAPS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
            }

            // metaGGA with kinetic: density + gradient + kinetic
            VarType::A_GAA_TAUA
            | VarType::N_GNN_TAUN
            | VarType::A_B_GAA_GAB_GBB_TAUA_TAUB
            | VarType::N_S_GNN_GNS_GSS_TAUN_TAUS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::KINETIC)
            }

            // metaGGA with laplacian + kinetic
            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
            | VarType::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
                    .union(Dependency::KINETIC)
            }

            // metaGGA with current density: density + gradient + laplacian + kinetic + JP
            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
                    .union(Dependency::KINETIC)
                    .union(Dependency::JP)
            }

            // GGA (gradient components): density + gradient
            VarType::A_AX_AY_AZ
            | VarType::A_B_AX_AY_AZ_BX_BY_BZ
            | VarType::N_NX_NY_NZ
            | VarType::N_S_NX_NY_NZ_SX_SY_SZ => {
                Dependency::DENSITY.union(Dependency::GRADIENT)
            }

            // metaGGA (gradient components + kinetic): density + gradient + kinetic
            VarType::A_AX_AY_AZ_TAUA
            | VarType::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB
            | VarType::N_NX_NY_NZ_TAUN
            | VarType::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::KINETIC)
            }

            // 2nd order Taylor: density + gradient + laplacian
            VarType::A_2ND_TAYLOR
            | VarType::A_B_2ND_TAYLOR
            | VarType::N_2ND_TAYLOR
            | VarType::N_S_2ND_TAYLOR => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
            }
        }
    }

    /// Whether this is a spin-polarized variable type.
    pub const fn is_spin_polarized(&self) -> bool {
        match self {
            VarType::A
            | VarType::N
            | VarType::A_GAA
            | VarType::N_GNN
            | VarType::A_GAA_LAPA
            | VarType::A_GAA_TAUA
            | VarType::N_GNN_LAPN
            | VarType::N_GNN_TAUN
            | VarType::A_AX_AY_AZ
            | VarType::N_NX_NY_NZ
            | VarType::A_AX_AY_AZ_TAUA
            | VarType::N_NX_NY_NZ_TAUN
            | VarType::A_2ND_TAYLOR
            | VarType::N_2ND_TAYLOR => false,

            VarType::A_B
            | VarType::N_S
            | VarType::A_B_GAA_GAB_GBB
            | VarType::N_S_GNN_GNS_GSS
            | VarType::A_B_GAA_GAB_GBB_LAPA_LAPB
            | VarType::A_B_GAA_GAB_GBB_TAUA_TAUB
            | VarType::N_S_GNN_GNS_GSS_LAPN_LAPS
            | VarType::N_S_GNN_GNS_GSS_TAUN_TAUS
            | VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
            | VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB
            | VarType::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS
            | VarType::A_B_AX_AY_AZ_BX_BY_BZ
            | VarType::N_S_NX_NY_NZ_SX_SY_SZ
            | VarType::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB
            | VarType::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS
            | VarType::A_B_2ND_TAYLOR
            | VarType::N_S_2ND_TAYLOR => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_mode_has_3_variants() {
        // Verify all 3 variants exist
        let _a = EvalMode::PartialDerivatives;
        let _b = EvalMode::Potential;
        let _c = EvalMode::Contracted;
    }

    #[test]
    fn var_type_cpp_ordering() {
        assert_eq!(VarType::A as u32, 0);
        assert_eq!(VarType::N as u32, 1);
        assert_eq!(VarType::A_B as u32, 2);
        assert_eq!(VarType::A_AX_AY_AZ as u32, 19);
        assert_eq!(VarType::N_S_2ND_TAYLOR as u32, 30);
    }

    #[test]
    fn var_type_input_len() {
        assert_eq!(VarType::A.input_len(), 1);
        assert_eq!(VarType::A_B.input_len(), 2);
        assert_eq!(VarType::A_B_GAA_GAB_GBB.input_len(), 5);
        assert_eq!(VarType::A_AX_AY_AZ.input_len(), 4);
        assert_eq!(VarType::A_B_AX_AY_AZ_BX_BY_BZ.input_len(), 8);
        assert_eq!(VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB.input_len(), 11);
        assert_eq!(VarType::A_2ND_TAYLOR.input_len(), 10);
        assert_eq!(VarType::A_B_2ND_TAYLOR.input_len(), 20);
    }

    #[test]
    fn var_type_provides() {
        assert_eq!(VarType::A.provides(), Dependency::DENSITY);
        assert_eq!(
            VarType::A_B_GAA_GAB_GBB.provides(),
            Dependency::DENSITY | Dependency::GRADIENT
        );
        assert_eq!(
            VarType::A_GAA_LAPA.provides(),
            Dependency::DENSITY | Dependency::GRADIENT | Dependency::LAPLACIAN
        );
        assert_eq!(
            VarType::A_GAA_TAUA.provides(),
            Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC
        );
        assert_eq!(
            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB.provides(),
            Dependency::DENSITY
                | Dependency::GRADIENT
                | Dependency::LAPLACIAN
                | Dependency::KINETIC
                | Dependency::JP
        );
    }

    #[test]
    fn var_type_spin_polarized() {
        assert!(!VarType::A.is_spin_polarized());
        assert!(!VarType::N.is_spin_polarized());
        assert!(VarType::A_B.is_spin_polarized());
        assert!(VarType::N_S.is_spin_polarized());
        assert!(!VarType::A_AX_AY_AZ.is_spin_polarized());
        assert!(VarType::A_B_AX_AY_AZ_BX_BY_BZ.is_spin_polarized());
    }
}
