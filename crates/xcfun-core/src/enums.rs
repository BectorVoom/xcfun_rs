//! Mode and Vars enums with metadata methods.

use crate::traits::Dependency;

/// Evaluation mode for functional derivatives. Discriminants match `xcfun.h::xcfun_mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Mode {
    /// Mode not yet selected via `eval_setup`. Matches `XC_MODE_UNSET = 0` (xcfun.h:36).
    Unset = 0,
    /// Compute partial derivatives up to configured order.
    PartialDerivatives = 1,
    /// Compute exchange-correlation potential.
    Potential = 2,
    /// Input contains pre-computed Taylor coefficients.
    Contracted = 3,
}

/// Input variable specification. Discriminants match `xcfun.h::xcfun_vars` exactly.
///
/// Per Phase 2 CONTEXT D-06: variant names are SCREAMING_SNAKE_CASE to match
/// the C identifiers `XC_<NAME>` minus the `XC_` prefix (e.g., `XC_A_B` → `A_B`).
/// `#[allow(non_camel_case_types)]` is required because Rust's default lint
/// rejects multi-underscore variants like `A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Vars {
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

impl Vars {
    /// Number of input values required for this variable type.
    pub const fn input_len(&self) -> usize {
        match self {
            // LDA
            Vars::A => 1,
            Vars::N => 1,
            Vars::A_B => 2,
            Vars::N_S => 2,

            // GGA (squared gradient)
            Vars::A_GAA => 2,
            Vars::N_GNN => 2,
            Vars::A_B_GAA_GAB_GBB => 5,
            Vars::N_S_GNN_GNS_GSS => 5,

            // metaGGA (laplacian variants)
            Vars::A_GAA_LAPA => 3,
            Vars::A_GAA_TAUA => 3,
            Vars::N_GNN_LAPN => 3,
            Vars::N_GNN_TAUN => 3,
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB => 7,
            Vars::A_B_GAA_GAB_GBB_TAUA_TAUB => 7,
            Vars::N_S_GNN_GNS_GSS_LAPN_LAPS => 7,
            Vars::N_S_GNN_GNS_GSS_TAUN_TAUS => 7,

            // metaGGA (laplacian + kinetic)
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB => 9,

            // metaGGA (with current density)
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => 11,

            // metaGGA (N/S with laplacian + kinetic)
            Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => 9,

            // GGA (gradient components)
            Vars::A_AX_AY_AZ => 4,
            Vars::A_B_AX_AY_AZ_BX_BY_BZ => 8,
            Vars::N_NX_NY_NZ => 4,
            Vars::N_S_NX_NY_NZ_SX_SY_SZ => 8,

            // metaGGA (gradient components + kinetic)
            Vars::A_AX_AY_AZ_TAUA => 5,
            Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB => 10,
            Vars::N_NX_NY_NZ_TAUN => 5,
            Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => 10,

            // 2nd order Taylor: taylorlen(n_vars, 2) input values
            // A_2ND_TAYLOR: 1 var, order 2 -> C(1+2,2) = 3 ... but wait
            // C++ comment says "1+3+6=10 numbers" for A_2ND_TAYLOR
            // That's actually the 3D spatial Taylor expansion of alpha density
            // taylorlen(3, 2) = C(5,2) = 10
            Vars::A_2ND_TAYLOR => 10,
            // "first alpha, then beta" -> 20 numbers
            Vars::A_B_2ND_TAYLOR => 20,
            Vars::N_2ND_TAYLOR => 10,
            Vars::N_S_2ND_TAYLOR => 20,
        }
    }

    /// Dependency flags provided by this variable type.
    pub const fn provides(&self) -> Dependency {
        match self {
            // LDA: density only
            Vars::A | Vars::N | Vars::A_B | Vars::N_S => Dependency::DENSITY,

            // GGA (squared gradient): density + gradient
            Vars::A_GAA
            | Vars::N_GNN
            | Vars::A_B_GAA_GAB_GBB
            | Vars::N_S_GNN_GNS_GSS => {
                Dependency::DENSITY.union(Dependency::GRADIENT)
            }

            // metaGGA with laplacian: density + gradient + laplacian
            Vars::A_GAA_LAPA
            | Vars::N_GNN_LAPN
            | Vars::A_B_GAA_GAB_GBB_LAPA_LAPB
            | Vars::N_S_GNN_GNS_GSS_LAPN_LAPS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
            }

            // metaGGA with kinetic: density + gradient + kinetic
            Vars::A_GAA_TAUA
            | Vars::N_GNN_TAUN
            | Vars::A_B_GAA_GAB_GBB_TAUA_TAUB
            | Vars::N_S_GNN_GNS_GSS_TAUN_TAUS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::KINETIC)
            }

            // metaGGA with laplacian + kinetic
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
            | Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
                    .union(Dependency::KINETIC)
            }

            // metaGGA with current density: density + gradient + laplacian + kinetic + JP
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
                    .union(Dependency::KINETIC)
                    .union(Dependency::JP)
            }

            // GGA (gradient components): density + gradient
            Vars::A_AX_AY_AZ
            | Vars::A_B_AX_AY_AZ_BX_BY_BZ
            | Vars::N_NX_NY_NZ
            | Vars::N_S_NX_NY_NZ_SX_SY_SZ => {
                Dependency::DENSITY.union(Dependency::GRADIENT)
            }

            // metaGGA (gradient components + kinetic): density + gradient + kinetic
            Vars::A_AX_AY_AZ_TAUA
            | Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB
            | Vars::N_NX_NY_NZ_TAUN
            | Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::KINETIC)
            }

            // 2nd order Taylor: density + gradient + laplacian
            Vars::A_2ND_TAYLOR
            | Vars::A_B_2ND_TAYLOR
            | Vars::N_2ND_TAYLOR
            | Vars::N_S_2ND_TAYLOR => {
                Dependency::DENSITY
                    .union(Dependency::GRADIENT)
                    .union(Dependency::LAPLACIAN)
            }
        }
    }

    /// Whether this is a spin-polarized variable type.
    pub const fn is_spin_polarized(&self) -> bool {
        match self {
            Vars::A
            | Vars::N
            | Vars::A_GAA
            | Vars::N_GNN
            | Vars::A_GAA_LAPA
            | Vars::A_GAA_TAUA
            | Vars::N_GNN_LAPN
            | Vars::N_GNN_TAUN
            | Vars::A_AX_AY_AZ
            | Vars::N_NX_NY_NZ
            | Vars::A_AX_AY_AZ_TAUA
            | Vars::N_NX_NY_NZ_TAUN
            | Vars::A_2ND_TAYLOR
            | Vars::N_2ND_TAYLOR => false,

            Vars::A_B
            | Vars::N_S
            | Vars::A_B_GAA_GAB_GBB
            | Vars::N_S_GNN_GNS_GSS
            | Vars::A_B_GAA_GAB_GBB_LAPA_LAPB
            | Vars::A_B_GAA_GAB_GBB_TAUA_TAUB
            | Vars::N_S_GNN_GNS_GSS_LAPN_LAPS
            | Vars::N_S_GNN_GNS_GSS_TAUN_TAUS
            | Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
            | Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB
            | Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS
            | Vars::A_B_AX_AY_AZ_BX_BY_BZ
            | Vars::N_S_NX_NY_NZ_SX_SY_SZ
            | Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB
            | Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS
            | Vars::A_B_2ND_TAYLOR
            | Vars::N_S_2ND_TAYLOR => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_has_4_variants() {
        // Verify all 4 variants exist (including Unset=0 per CORE-02 + D-07).
        let _a = Mode::Unset;
        let _b = Mode::PartialDerivatives;
        let _c = Mode::Potential;
        let _d = Mode::Contracted;
    }

    #[test]
    fn mode_unset_is_zero() {
        assert_eq!(Mode::Unset as u32, 0);
    }

    #[test]
    fn mode_repr_u32_round_trip() {
        assert_eq!(Mode::PartialDerivatives as u32, 1);
        assert_eq!(Mode::Potential as u32, 2);
        assert_eq!(Mode::Contracted as u32, 3);
    }

    #[test]
    fn vars_cpp_ordering() {
        assert_eq!(Vars::A as u32, 0);
        assert_eq!(Vars::N as u32, 1);
        assert_eq!(Vars::A_B as u32, 2);
        assert_eq!(Vars::A_AX_AY_AZ as u32, 19);
        assert_eq!(Vars::N_S_2ND_TAYLOR as u32, 30);
    }

    #[test]
    fn vars_input_len() {
        assert_eq!(Vars::A.input_len(), 1);
        assert_eq!(Vars::A_B.input_len(), 2);
        assert_eq!(Vars::A_B_GAA_GAB_GBB.input_len(), 5);
        assert_eq!(Vars::A_AX_AY_AZ.input_len(), 4);
        assert_eq!(Vars::A_B_AX_AY_AZ_BX_BY_BZ.input_len(), 8);
        assert_eq!(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB.input_len(), 11);
        assert_eq!(Vars::A_2ND_TAYLOR.input_len(), 10);
        assert_eq!(Vars::A_B_2ND_TAYLOR.input_len(), 20);
    }

    #[test]
    fn vars_provides() {
        assert_eq!(Vars::A.provides(), Dependency::DENSITY);
        assert_eq!(
            Vars::A_B_GAA_GAB_GBB.provides(),
            Dependency::DENSITY | Dependency::GRADIENT
        );
        assert_eq!(
            Vars::A_GAA_LAPA.provides(),
            Dependency::DENSITY | Dependency::GRADIENT | Dependency::LAPLACIAN
        );
        assert_eq!(
            Vars::A_GAA_TAUA.provides(),
            Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC
        );
        assert_eq!(
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB.provides(),
            Dependency::DENSITY
                | Dependency::GRADIENT
                | Dependency::LAPLACIAN
                | Dependency::KINETIC
                | Dependency::JP
        );
    }

    #[test]
    fn vars_spin_polarized() {
        assert!(!Vars::A.is_spin_polarized());
        assert!(!Vars::N.is_spin_polarized());
        assert!(Vars::A_B.is_spin_polarized());
        assert!(Vars::N_S.is_spin_polarized());
        assert!(!Vars::A_AX_AY_AZ.is_spin_polarized());
        assert!(Vars::A_B_AX_AY_AZ_BX_BY_BZ.is_spin_polarized());
    }
}
