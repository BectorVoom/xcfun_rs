//! FunctionalId enum -- 78 exchange-correlation functional identifiers.

use crate::traits::Dependency;

/// Unique identifier for each exchange-correlation functional.
///
/// 78 variants matching the C++ xcfun functional set.
/// Ordering follows docs/design/01-data-structures.md section 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FunctionalId {
    // LDA
    SlaterX = 0,
    Vwn3C,
    Vwn5C,
    Pz81C,
    Pw92C,

    // GGA Exchange
    Pw86X,
    PbeX,
    BeckeX,
    BeckeCorrX,
    BeckeSrX,
    BeckeCamX,
    BrX,
    LdaErfX,
    LdaErfC,
    LdaErfC_JT,
    Pw91X,
    RevPbeX,
    RPbeX,
    OptX,
    OptXCorr,
    PbeSolX,
    PbeIntX,
    BlocX,
    KtX,
    B97X,
    B97_1X,
    B97_2X,

    // GGA Correlation
    PbeC,
    BrC,
    BrXC,
    LypC,
    P86C,
    P86CorrC,
    SPbeC,
    Vwn_PbeC,
    Pw91C,
    B97C,
    B97_1C,
    B97_2C,
    CsC,
    APbeC,
    ZvPbeSolC,
    PbeIntC,
    PbeLocC,
    ZvPbeIntC,

    // Kinetic
    TfK,
    Tw,
    VwK,
    Pw91K,

    // Hybrid meta-GGA (M05/M06 family)
    M05X,
    M05X2X,
    M06X,
    M06X2X,
    M06LX,
    M06HfX,
    M05C,
    M05X2C,
    M06C,
    M06HfC,
    M06LC,
    M06X2C,

    // meta-GGA (TPSS, SCAN families)
    TpssX,
    TpssC,
    RevTpssX,
    RevTpssC,
    TpssLocC,
    ScanX,
    ScanC,
    RScanX,
    RScanC,
    RppScanX,
    RppScanC,
    R2ScanX,
    R2ScanC,
    R4ScanX,
    R4ScanC,

    // Kinetic (meta-GGA)
    BtK,
}

impl FunctionalId {
    /// Total number of functionals.
    pub const COUNT: usize = 78;

    /// Look up a functional by its string name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            // LDA
            "slaterx" => Some(Self::SlaterX),
            "vwn3c" => Some(Self::Vwn3C),
            "vwn5c" => Some(Self::Vwn5C),
            "pz81c" => Some(Self::Pz81C),
            "pw92c" => Some(Self::Pw92C),

            // GGA Exchange
            "pw86x" => Some(Self::Pw86X),
            "pbex" => Some(Self::PbeX),
            "beckex" => Some(Self::BeckeX),
            "beckecorrx" => Some(Self::BeckeCorrX),
            "beckesrx" => Some(Self::BeckeSrX),
            "beckecamx" => Some(Self::BeckeCamX),
            "brx" => Some(Self::BrX),
            "ldaerfx" => Some(Self::LdaErfX),
            "ldaerfc" => Some(Self::LdaErfC),
            "ldaerfc_jt" => Some(Self::LdaErfC_JT),
            "pw91x" => Some(Self::Pw91X),
            "revpbex" => Some(Self::RevPbeX),
            "rpbex" => Some(Self::RPbeX),
            "optx" => Some(Self::OptX),
            "optxcorr" => Some(Self::OptXCorr),
            "pbesolx" => Some(Self::PbeSolX),
            "pbeintx" => Some(Self::PbeIntX),
            "blocx" => Some(Self::BlocX),
            "ktx" => Some(Self::KtX),
            "b97x" => Some(Self::B97X),
            "b97_1x" => Some(Self::B97_1X),
            "b97_2x" => Some(Self::B97_2X),

            // GGA Correlation
            "pbec" => Some(Self::PbeC),
            "brc" => Some(Self::BrC),
            "brxc" => Some(Self::BrXC),
            "lypc" => Some(Self::LypC),
            "p86c" => Some(Self::P86C),
            "p86corrc" => Some(Self::P86CorrC),
            "spbec" => Some(Self::SPbeC),
            "vwn_pbec" => Some(Self::Vwn_PbeC),
            "pw91c" => Some(Self::Pw91C),
            "b97c" => Some(Self::B97C),
            "b97_1c" => Some(Self::B97_1C),
            "b97_2c" => Some(Self::B97_2C),
            "csc" => Some(Self::CsC),
            "apbec" => Some(Self::APbeC),
            "zvpbesolc" => Some(Self::ZvPbeSolC),
            "pbeintc" => Some(Self::PbeIntC),
            "pbelocc" => Some(Self::PbeLocC),
            "zvpbeintc" => Some(Self::ZvPbeIntC),

            // Kinetic
            "tfk" => Some(Self::TfK),
            "tw" => Some(Self::Tw),
            "vwk" => Some(Self::VwK),
            "pw91k" => Some(Self::Pw91K),

            // Hybrid meta-GGA (M05/M06 family)
            "m05x" => Some(Self::M05X),
            "m05x2x" => Some(Self::M05X2X),
            "m06x" => Some(Self::M06X),
            "m06x2x" => Some(Self::M06X2X),
            "m06lx" => Some(Self::M06LX),
            "m06hfx" => Some(Self::M06HfX),
            "m05c" => Some(Self::M05C),
            "m05x2c" => Some(Self::M05X2C),
            "m06c" => Some(Self::M06C),
            "m06hfc" => Some(Self::M06HfC),
            "m06lc" => Some(Self::M06LC),
            "m06x2c" => Some(Self::M06X2C),

            // meta-GGA (TPSS, SCAN families)
            "tpssx" => Some(Self::TpssX),
            "tpssc" => Some(Self::TpssC),
            "revtpssx" => Some(Self::RevTpssX),
            "revtpssc" => Some(Self::RevTpssC),
            "tpsslocc" => Some(Self::TpssLocC),
            "scanx" => Some(Self::ScanX),
            "scanc" => Some(Self::ScanC),
            "rscanx" => Some(Self::RScanX),
            "rscanc" => Some(Self::RScanC),
            "rppscanx" => Some(Self::RppScanX),
            "rppscanc" => Some(Self::RppScanC),
            "r2scanx" => Some(Self::R2ScanX),
            "r2scanc" => Some(Self::R2ScanC),
            "r4scanx" => Some(Self::R4ScanX),
            "r4scanc" => Some(Self::R4ScanC),

            // Kinetic meta-GGA
            "btk" => Some(Self::BtK),

            _ => None,
        }
    }

    /// Canonical string name of this functional.
    pub fn name(&self) -> &'static str {
        match self {
            Self::SlaterX => "SlaterX",
            Self::Vwn3C => "Vwn3C",
            Self::Vwn5C => "Vwn5C",
            Self::Pz81C => "Pz81C",
            Self::Pw92C => "Pw92C",
            Self::Pw86X => "Pw86X",
            Self::PbeX => "PbeX",
            Self::BeckeX => "BeckeX",
            Self::BeckeCorrX => "BeckeCorrX",
            Self::BeckeSrX => "BeckeSrX",
            Self::BeckeCamX => "BeckeCamX",
            Self::BrX => "BrX",
            Self::LdaErfX => "LdaErfX",
            Self::LdaErfC => "LdaErfC",
            Self::LdaErfC_JT => "LdaErfC_JT",
            Self::Pw91X => "Pw91X",
            Self::RevPbeX => "RevPbeX",
            Self::RPbeX => "RPbeX",
            Self::OptX => "OptX",
            Self::OptXCorr => "OptXCorr",
            Self::PbeSolX => "PbeSolX",
            Self::PbeIntX => "PbeIntX",
            Self::BlocX => "BlocX",
            Self::KtX => "KtX",
            Self::B97X => "B97X",
            Self::B97_1X => "B97_1X",
            Self::B97_2X => "B97_2X",
            Self::PbeC => "PbeC",
            Self::BrC => "BrC",
            Self::BrXC => "BrXC",
            Self::LypC => "LypC",
            Self::P86C => "P86C",
            Self::P86CorrC => "P86CorrC",
            Self::SPbeC => "SPbeC",
            Self::Vwn_PbeC => "Vwn_PbeC",
            Self::Pw91C => "Pw91C",
            Self::B97C => "B97C",
            Self::B97_1C => "B97_1C",
            Self::B97_2C => "B97_2C",
            Self::CsC => "CsC",
            Self::APbeC => "APbeC",
            Self::ZvPbeSolC => "ZvPbeSolC",
            Self::PbeIntC => "PbeIntC",
            Self::PbeLocC => "PbeLocC",
            Self::ZvPbeIntC => "ZvPbeIntC",
            Self::TfK => "TfK",
            Self::Tw => "Tw",
            Self::VwK => "VwK",
            Self::Pw91K => "Pw91K",
            Self::M05X => "M05X",
            Self::M05X2X => "M05X2X",
            Self::M06X => "M06X",
            Self::M06X2X => "M06X2X",
            Self::M06LX => "M06LX",
            Self::M06HfX => "M06HfX",
            Self::M05C => "M05C",
            Self::M05X2C => "M05X2C",
            Self::M06C => "M06C",
            Self::M06HfC => "M06HfC",
            Self::M06LC => "M06LC",
            Self::M06X2C => "M06X2C",
            Self::TpssX => "TpssX",
            Self::TpssC => "TpssC",
            Self::RevTpssX => "RevTpssX",
            Self::RevTpssC => "RevTpssC",
            Self::TpssLocC => "TpssLocC",
            Self::ScanX => "ScanX",
            Self::ScanC => "ScanC",
            Self::RScanX => "RScanX",
            Self::RScanC => "RScanC",
            Self::RppScanX => "RppScanX",
            Self::RppScanC => "RppScanC",
            Self::R2ScanX => "R2ScanX",
            Self::R2ScanC => "R2ScanC",
            Self::R4ScanX => "R4ScanX",
            Self::R4ScanC => "R4ScanC",
            Self::BtK => "BtK",
        }
    }

    /// Short description of this functional.
    pub fn description(&self) -> &'static str {
        match self {
            // LDA
            Self::SlaterX => "Slater LDA exchange",
            Self::Vwn3C => "VWN3 LDA correlation",
            Self::Vwn5C => "VWN5 LDA correlation",
            Self::Pz81C => "Perdew-Zunger 1981 LDA correlation",
            Self::Pw92C => "Perdew-Wang 1992 LDA correlation",

            // GGA Exchange
            Self::Pw86X => "Perdew-Wang 1986 GGA exchange",
            Self::PbeX => "PBE GGA exchange",
            Self::BeckeX => "Becke 1988 GGA exchange",
            Self::BeckeCorrX => "Becke 1988 GGA exchange correction",
            Self::BeckeSrX => "Becke short-range exchange",
            Self::BeckeCamX => "Becke CAM exchange",
            Self::BrX => "Becke-Roussel exchange",
            Self::LdaErfX => "LDA short-range exchange (erf)",
            Self::LdaErfC => "LDA short-range correlation (erf)",
            Self::LdaErfC_JT => "LDA short-range correlation (erf, JT)",
            Self::Pw91X => "Perdew-Wang 1991 GGA exchange",
            Self::RevPbeX => "Revised PBE exchange",
            Self::RPbeX => "RPBE exchange",
            Self::OptX => "OptX exchange",
            Self::OptXCorr => "OptX exchange correction",
            Self::PbeSolX => "PBEsol exchange",
            Self::PbeIntX => "PBEint exchange",
            Self::BlocX => "BLOC exchange",
            Self::KtX => "KT exchange",
            Self::B97X => "B97 exchange",
            Self::B97_1X => "B97-1 exchange",
            Self::B97_2X => "B97-2 exchange",

            // GGA Correlation
            Self::PbeC => "PBE GGA correlation",
            Self::BrC => "Becke-Roussel correlation",
            Self::BrXC => "Becke-Roussel exchange-correlation",
            Self::LypC => "Lee-Yang-Parr correlation",
            Self::P86C => "Perdew 1986 correlation",
            Self::P86CorrC => "Perdew 1986 correlation correction",
            Self::SPbeC => "Simplified PBE correlation",
            Self::Vwn_PbeC => "VWN-PBE correlation",
            Self::Pw91C => "Perdew-Wang 1991 GGA correlation",
            Self::B97C => "B97 correlation",
            Self::B97_1C => "B97-1 correlation",
            Self::B97_2C => "B97-2 correlation",
            Self::CsC => "Colle-Salvetti correlation",
            Self::APbeC => "APBEc correlation",
            Self::ZvPbeSolC => "zvPBEsol correlation",
            Self::PbeIntC => "PBEint correlation",
            Self::PbeLocC => "PBEloc correlation",
            Self::ZvPbeIntC => "zvPBEint correlation",

            // Kinetic
            Self::TfK => "Thomas-Fermi kinetic energy",
            Self::Tw => "von Weizsacker kinetic energy",
            Self::VwK => "von Weizsacker kinetic energy functional",
            Self::Pw91K => "Perdew-Wang 1991 kinetic energy",

            // Hybrid meta-GGA (M05/M06 family)
            Self::M05X => "M05 exchange",
            Self::M05X2X => "M05-2X exchange",
            Self::M06X => "M06 exchange",
            Self::M06X2X => "M06-2X exchange",
            Self::M06LX => "M06-L exchange",
            Self::M06HfX => "M06-HF exchange",
            Self::M05C => "M05 correlation",
            Self::M05X2C => "M05-2X correlation",
            Self::M06C => "M06 correlation",
            Self::M06HfC => "M06-HF correlation",
            Self::M06LC => "M06-L correlation",
            Self::M06X2C => "M06-2X correlation",

            // meta-GGA
            Self::TpssX => "TPSS meta-GGA exchange",
            Self::TpssC => "TPSS meta-GGA correlation",
            Self::RevTpssX => "revTPSS meta-GGA exchange",
            Self::RevTpssC => "revTPSS meta-GGA correlation",
            Self::TpssLocC => "TPSSloc meta-GGA correlation",
            Self::ScanX => "SCAN meta-GGA exchange",
            Self::ScanC => "SCAN meta-GGA correlation",
            Self::RScanX => "rSCAN meta-GGA exchange",
            Self::RScanC => "rSCAN meta-GGA correlation",
            Self::RppScanX => "r++SCAN meta-GGA exchange",
            Self::RppScanC => "r++SCAN meta-GGA correlation",
            Self::R2ScanX => "r2SCAN meta-GGA exchange",
            Self::R2ScanC => "r2SCAN meta-GGA correlation",
            Self::R4ScanX => "r4SCAN meta-GGA exchange",
            Self::R4ScanC => "r4SCAN meta-GGA correlation",

            // Kinetic meta-GGA
            Self::BtK => "Becke-Tsuneda kinetic energy",
        }
    }

    /// Dependency flags for this functional.
    pub fn depends(&self) -> Dependency {
        match self {
            // LDA: density only
            Self::SlaterX | Self::Vwn3C | Self::Vwn5C | Self::Pz81C | Self::Pw92C => {
                Dependency::DENSITY
            }

            // GGA: density + gradient
            Self::Pw86X
            | Self::PbeX
            | Self::BeckeX
            | Self::BeckeCorrX
            | Self::BeckeSrX
            | Self::BeckeCamX
            | Self::BrX
            | Self::LdaErfX
            | Self::LdaErfC
            | Self::LdaErfC_JT
            | Self::Pw91X
            | Self::RevPbeX
            | Self::RPbeX
            | Self::OptX
            | Self::OptXCorr
            | Self::PbeSolX
            | Self::PbeIntX
            | Self::BlocX
            | Self::KtX
            | Self::B97X
            | Self::B97_1X
            | Self::B97_2X
            | Self::PbeC
            | Self::BrC
            | Self::BrXC
            | Self::LypC
            | Self::P86C
            | Self::P86CorrC
            | Self::SPbeC
            | Self::Vwn_PbeC
            | Self::Pw91C
            | Self::B97C
            | Self::B97_1C
            | Self::B97_2C
            | Self::CsC
            | Self::APbeC
            | Self::ZvPbeSolC
            | Self::PbeIntC
            | Self::PbeLocC
            | Self::ZvPbeIntC => Dependency::DENSITY | Dependency::GRADIENT,

            // Kinetic: density only (TfK, Tw)
            Self::TfK | Self::Tw => Dependency::DENSITY,

            // Kinetic with gradient: density + gradient + kinetic
            Self::VwK | Self::Pw91K | Self::BtK => {
                Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC
            }

            // Hybrid meta-GGA (M05/M06 family): density + gradient + kinetic
            Self::M05X
            | Self::M05X2X
            | Self::M06X
            | Self::M06X2X
            | Self::M06LX
            | Self::M06HfX
            | Self::M05C
            | Self::M05X2C
            | Self::M06C
            | Self::M06HfC
            | Self::M06LC
            | Self::M06X2C => Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC,

            // meta-GGA (TPSS, SCAN families): density + gradient + kinetic
            Self::TpssX
            | Self::TpssC
            | Self::RevTpssX
            | Self::RevTpssC
            | Self::TpssLocC
            | Self::ScanX
            | Self::ScanC
            | Self::RScanX
            | Self::RScanC
            | Self::RppScanX
            | Self::RppScanC
            | Self::R2ScanX
            | Self::R2ScanC
            | Self::R4ScanX
            | Self::R4ScanC => Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slaterx_is_zero() {
        assert_eq!(FunctionalId::SlaterX as u32, 0);
    }

    #[test]
    fn count_is_78() {
        assert_eq!(FunctionalId::COUNT, 78);
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(
            FunctionalId::from_name("slaterx"),
            Some(FunctionalId::SlaterX)
        );
        assert_eq!(
            FunctionalId::from_name("PBEX"),
            Some(FunctionalId::PbeX)
        );
        assert_eq!(
            FunctionalId::from_name("SlaterX"),
            Some(FunctionalId::SlaterX)
        );
    }

    #[test]
    fn from_name_nonexistent() {
        assert_eq!(FunctionalId::from_name("nonexistent"), None);
    }

    #[test]
    fn name_roundtrip() {
        assert_eq!(FunctionalId::SlaterX.name(), "SlaterX");
        assert_eq!(FunctionalId::PbeX.name(), "PbeX");
        assert_eq!(FunctionalId::BtK.name(), "BtK");
    }

    #[test]
    fn depends_lda() {
        assert_eq!(FunctionalId::SlaterX.depends(), Dependency::DENSITY);
    }

    #[test]
    fn depends_gga() {
        assert_eq!(
            FunctionalId::PbeX.depends(),
            Dependency::DENSITY | Dependency::GRADIENT
        );
    }

    #[test]
    fn depends_meta_gga() {
        assert_eq!(
            FunctionalId::ScanX.depends(),
            Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC
        );
    }

    #[test]
    fn depends_m06() {
        assert_eq!(
            FunctionalId::M06X.depends(),
            Dependency::DENSITY | Dependency::GRADIENT | Dependency::KINETIC
        );
    }
}
