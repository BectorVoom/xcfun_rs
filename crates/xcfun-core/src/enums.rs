//! Mode, Vars, and ParameterId enums with metadata methods.

use crate::traits::Dependency;

/// Parameter identifier for the xcfun common parameter table.
///
/// Discriminants match `xcfun-master/src/functionals/list_of_functionals.hpp:99-105`
/// EXACTLY:
/// ```cpp
/// enum xc_parameter {
///   XC_RANGESEP_MU = XC_NR_FUNCTIONALS,  // 78
///   XC_EXX,                              // 79
///   XC_CAM_ALPHA,                        // 80
///   XC_CAM_BETA,                         // 81
///   XC_NR_PARAMETERS_AND_FUNCTIONALS     // 82
/// };
/// ```
///
/// `as u32` indexes into the `Functional::settings[82]` array alongside
/// `FunctionalId` (discriminants 0..=77). Plan 04-04 + Phase 4 D-05.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ParameterId {
    /// Range separation inverse length [1/a0]; default 0.4.
    /// `xcfun-master/src/functionals/common_parameters.cpp:17`.
    XC_RANGESEP_MU = 78,
    /// Amount of exact (HF-like) exchange; default 0.0.
    /// `xcfun-master/src/functionals/common_parameters.cpp:19-21`.
    XC_EXX = 79,
    /// Amount of exact exchange within CAM-B3LYP; default 0.19.
    /// `xcfun-master/src/functionals/common_parameters.cpp:23-25`.
    XC_CAM_ALPHA = 80,
    /// Amount of long-range exchange within CAM-B3LYP; default 0.46.
    /// `xcfun-master/src/functionals/common_parameters.cpp:27-29`.
    XC_CAM_BETA = 81,
}

impl ParameterId {
    /// Total number of parameters (matches
    /// `XC_NR_PARAMETERS_AND_FUNCTIONALS - XC_NR_FUNCTIONALS = 4` in C++).
    pub const COUNT: usize = 4;

    /// Look up a parameter by string name (case-insensitive). Mirrors the
    /// C++ `xcint_lookup_parameter` (`xcint.cpp:38-43`) which uses
    /// `strcasecmp` against the name with the `XC_` prefix stripped.
    /// Accepts both prefixed (`"XC_RANGESEP_MU"`) and bare
    /// (`"rangesep_mu"`) forms per D-04-B (Phase 4 CONTEXT).
    pub fn from_name(name: &str) -> Option<Self> {
        let upper = name.to_ascii_uppercase();
        let trimmed = upper.strip_prefix("XC_").unwrap_or(&upper);
        match trimmed {
            "RANGESEP_MU" => Some(Self::XC_RANGESEP_MU),
            "EXX" => Some(Self::XC_EXX),
            "CAM_ALPHA" => Some(Self::XC_CAM_ALPHA),
            "CAM_BETA" => Some(Self::XC_CAM_BETA),
            _ => None,
        }
    }

    /// Default value for this parameter, copied verbatim from
    /// `xcfun-master/src/functionals/common_parameters.cpp:17-29`.
    /// Used by `Functional::new` to seed the parameter slots in
    /// `settings[78..=81]`.
    pub const fn default_value(self) -> f64 {
        match self {
            Self::XC_RANGESEP_MU => 0.4,
            Self::XC_EXX => 0.0,
            Self::XC_CAM_ALPHA => 0.19,
            Self::XC_CAM_BETA => 0.46,
        }
    }
}

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
            Vars::A_GAA | Vars::N_GNN | Vars::A_B_GAA_GAB_GBB | Vars::N_S_GNN_GNS_GSS => {
                Dependency::DENSITY.union(Dependency::GRADIENT)
            }

            // metaGGA with laplacian: density + gradient + laplacian
            Vars::A_GAA_LAPA
            | Vars::N_GNN_LAPN
            | Vars::A_B_GAA_GAB_GBB_LAPA_LAPB
            | Vars::N_S_GNN_GNS_GSS_LAPN_LAPS => Dependency::DENSITY
                .union(Dependency::GRADIENT)
                .union(Dependency::LAPLACIAN),

            // metaGGA with kinetic: density + gradient + kinetic
            Vars::A_GAA_TAUA
            | Vars::N_GNN_TAUN
            | Vars::A_B_GAA_GAB_GBB_TAUA_TAUB
            | Vars::N_S_GNN_GNS_GSS_TAUN_TAUS => Dependency::DENSITY
                .union(Dependency::GRADIENT)
                .union(Dependency::KINETIC),

            // metaGGA with laplacian + kinetic
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
            | Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => Dependency::DENSITY
                .union(Dependency::GRADIENT)
                .union(Dependency::LAPLACIAN)
                .union(Dependency::KINETIC),

            // metaGGA with current density: density + gradient + laplacian + kinetic + JP
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => Dependency::DENSITY
                .union(Dependency::GRADIENT)
                .union(Dependency::LAPLACIAN)
                .union(Dependency::KINETIC)
                .union(Dependency::JP),

            // GGA (gradient components): density + gradient
            Vars::A_AX_AY_AZ
            | Vars::A_B_AX_AY_AZ_BX_BY_BZ
            | Vars::N_NX_NY_NZ
            | Vars::N_S_NX_NY_NZ_SX_SY_SZ => Dependency::DENSITY.union(Dependency::GRADIENT),

            // metaGGA (gradient components + kinetic): density + gradient + kinetic
            Vars::A_AX_AY_AZ_TAUA
            | Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB
            | Vars::N_NX_NY_NZ_TAUN
            | Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => Dependency::DENSITY
                .union(Dependency::GRADIENT)
                .union(Dependency::KINETIC),

            // 2nd order Taylor: density + gradient + laplacian
            Vars::A_2ND_TAYLOR
            | Vars::A_B_2ND_TAYLOR
            | Vars::N_2ND_TAYLOR
            | Vars::N_S_2ND_TAYLOR => Dependency::DENSITY
                .union(Dependency::GRADIENT)
                .union(Dependency::LAPLACIAN),
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
        assert_eq!(
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB.input_len(),
            11
        );
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
    fn parameter_id_discriminants() {
        // list_of_functionals.hpp:99-105 — discriminants 78..=81 (immediately
        // after XC_NR_FUNCTIONALS = 78).
        assert_eq!(ParameterId::XC_RANGESEP_MU as u32, 78);
        assert_eq!(ParameterId::XC_EXX as u32, 79);
        assert_eq!(ParameterId::XC_CAM_ALPHA as u32, 80);
        assert_eq!(ParameterId::XC_CAM_BETA as u32, 81);
        assert_eq!(ParameterId::COUNT, 4);
    }

    #[test]
    fn parameter_id_from_name_case_insensitive() {
        // C++ xcint_lookup_parameter uses strcasecmp on the symbol minus "XC_".
        assert_eq!(
            ParameterId::from_name("rangesep_mu"),
            Some(ParameterId::XC_RANGESEP_MU)
        );
        assert_eq!(
            ParameterId::from_name("RANGESEP_MU"),
            Some(ParameterId::XC_RANGESEP_MU)
        );
        assert_eq!(
            ParameterId::from_name("XC_RANGESEP_MU"),
            Some(ParameterId::XC_RANGESEP_MU)
        );
        assert_eq!(
            ParameterId::from_name("Cam_Alpha"),
            Some(ParameterId::XC_CAM_ALPHA)
        );
        assert_eq!(
            ParameterId::from_name("xc_cam_beta"),
            Some(ParameterId::XC_CAM_BETA)
        );
        assert_eq!(ParameterId::from_name("not_a_param"), None);
        assert_eq!(ParameterId::from_name(""), None);
    }

    #[test]
    fn parameter_id_default_values_from_cpp() {
        // common_parameters.cpp:17, 19-21, 23-25, 27-29.
        assert_eq!(ParameterId::XC_RANGESEP_MU.default_value(), 0.4);
        assert_eq!(ParameterId::XC_EXX.default_value(), 0.0);
        assert_eq!(ParameterId::XC_CAM_ALPHA.default_value(), 0.19);
        assert_eq!(ParameterId::XC_CAM_BETA.default_value(), 0.46);
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
