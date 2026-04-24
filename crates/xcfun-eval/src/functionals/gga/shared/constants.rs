//! f64-precision scalar constants for all 36 Phase-3 GGA kernels (D-08 + SP-2).
//!
//! Convention (PATTERNS §D6 + §S2): every constant is `pub const <NAME>_F64: f64`,
//! and kernel bodies cast via `F::cast_from(<NAME>_F64)`. Never `F::new(<literal_f32>)`
//! on a numerical path — `F::new` takes an `f32` in cubecl 0.10-pre.3 and widens,
//! silently dropping the lower ~7 decimal digits of precision.
//!
//! Values below were computed or extracted verbatim from
//! `xcfun-master/src/functionals/*` (cited inline per constant). The few values
//! computed from C++ expressions (e.g. `0.066725 * M_PI * M_PI / 3.0`) are
//! materialised in f64 at planning-time to avoid rounding drift at load-time.
//!
//! # Double-check audit (planner cross-check 2026-04-24)
//! - `MU_PBE_F64` = `0.066725 * π² / 3` = **0.21951645122089580** (C++ default branch
//!   `pbex.hpp:34`). Note: the Daresbury branch `XCFUN_REF_PBEX_MU` (pbex.hpp:32)
//!   uses the literal `0.2195149727645171` — stored separately as
//!   `MU_PBE_RPBEX_F64` for the `enhancement_RPBE` helper (`pbex.hpp:44`) which
//!   uses that literal even in the default branch.
//! - `S2_PREFACTOR_F64` = `(6^(2/3) / (12·π^(2/3)))²` = **0.01645530784602056**
//!   (`pw9xx.hpp:44-46`). The plan specification carried `0.16162...` which was
//!   off by a factor of 10 — the correct value is used here (executor Rule-1 bug fix,
//!   documented in plan 03-01 SUMMARY.md §Deviations).
//! - `NEG_C_SLATER_F64` = `-(81/(32·π))^(1/3)` = **-0.9305257363491001** matches
//!   `xcfun_constants::c_slater` computed in C++ and the Phase-2 SLATERX constant.

// --- PBE family ----------------------------------------------------------

/// `0.066725 · π² / 3` — default branch of `pbex.hpp:34` (XCFUN_REF_PBEX_MU undefined).
pub const MU_PBE_F64: f64 = 0.219_516_451_220_895_8_f64;

/// PBE enhancement_RPBE literal — used by `pbex.hpp:44`'s `enhancement_RPBE`
/// regardless of `XCFUN_REF_PBEX_MU`, and as the Daresbury value for PBE when that
/// flag is set. Kept as a separate constant so the RPBE helper can cast from it.
pub const MU_PBE_RPBEX_F64: f64 = 0.219_514_972_764_517_1_f64;

/// `pbex.hpp:22` κ for PBE.
pub const R_PBE_F64: f64 = 0.804_f64;

/// `pbex.hpp:23` κ for REVPBE.
pub const R_REVPBE_F64: f64 = 1.245_f64;

/// `rpbex.cpp:20` κ for RPBE (same as PBE per Zhang & Yang 1998).
pub const R_RPBE_F64: f64 = 0.804_f64;

/// `pbesolx.cpp:21` μ for PBESOL = 10/81 (Perdew et al. PRL 100).
pub const MU_PBESOL_F64: f64 = 0.123_456_790_123_456_79_f64;

/// `apbex.cpp:21` μ for APBE (Constantin et al. PRB 83).
pub const MU_APBE_F64: f64 = 0.26_f64;

/// `pbeintx.cpp:24` α for PBEINT.
pub const ALPHA_PBEINT_F64: f64 = 0.197_f64;

/// `pbeintx.cpp:21` μ_pbe (literal, not computed).
pub const MU_PBEINT_PBE_F64: f64 = 0.21951_f64;

/// `pbeintx.cpp:22` + `pbesolx.cpp:21` μ_GE (gradient expansion).
pub const MU_GE_F64: f64 = 0.123_456_790_123_f64;

// --- PBE correlation (pbec.cpp / pbec_eps.hpp) --------------------------

/// `constants.hpp:34` PBE correlation γ = `(1 - log(2)) / π²`.
pub const PBEC_GAMMA_F64: f64 = 0.031_090_690_869_654_9_f64;

/// `constants.hpp:35` β (PBE paper value).
pub const PBEC_BETA_PBE_PAPER_F64: f64 = 0.066_725_f64;

/// `constants.hpp:36` β (accurate; used as the default `param_beta` everywhere
/// `param_beta_accurate` or `param_beta_gamma` are referenced).
pub const PBEC_BETA_ACCURATE_F64: f64 = 0.066_724_550_603_149_22_f64;

/// `constants.hpp:37` β / γ ratio (precomputed for expm1 denominator).
/// `PBEC_BETA_ACCURATE_F64 / PBEC_GAMMA_F64` evaluated in f64.
pub const PBEC_BETA_GAMMA_F64: f64 = 2.146_126_339_967_364_2_f64;

/// PBEC `d2` (= t²) prefactor: `(1/12 · 3^(5/6) / π^(-1/6))² = cbrt(π/3) / 16`.
/// Evaluated in f64 as 0.06346820609770369. Used by PBEC, PBEINTC, PBELOCC,
/// ZVPBESOLC, ZVPBEINTC, VWN_PBEC; also by SPBEC where it is written as
/// `cbrt(π/3) / 16`.
pub const PBEC_D2_PREFACTOR_F64: f64 = 0.063_468_206_097_703_69_f64;

// --- PW91-like family (pw9xx.hpp:39-94) --------------------------------

/// `pw9xx.hpp:44-46` `S2` prefactor = `(6^(2/3) / (12·π^(2/3)))²`.
pub const S2_PREFACTOR_F64: f64 = 0.016_455_307_846_020_56_f64;

// --- Becke (beckex.cpp:18-19) -------------------------------------------

pub const BECKE_D_F64: f64 = 0.004_2_f64;
pub const BECKE_6D_F64: f64 = 0.025_2_f64;

// --- LYP (lypc.cpp:19-22) -----------------------------------------------

pub const LYP_A_F64: f64 = 0.049_18_f64;
pub const LYP_B_F64: f64 = 0.132_f64;
pub const LYP_C_F64: f64 = 0.253_3_f64;
pub const LYP_D_F64: f64 = 0.349_f64;
/// `(3/10) · (3π²)^(2/3)` = `xcfun_constants::CF` from `constants.hpp:31`.
pub const LYP_CF_F64: f64 = 2.871_234_000_188_191_f64;

// --- OPTX (optx.cpp:19; optxcorr.cpp:28) --------------------------------

pub const OPTX_A1_F64: f64 = 1.051_51_f64;
pub const OPTX_A2_F64: f64 = 1.431_69_f64;
pub const OPTX_GAMMA_F64: f64 = 0.006_f64;

// --- KT/BTK (ktx.cpp:19; btk.cpp:18-20) ---------------------------------

/// KT DELTA — not the OPTX γ. Verbatim from `ktx.cpp:19`.
pub const KTX_DELTA_F64: f64 = 0.1_f64;
/// BTK qav (Blanco-Tarrago-Karakos parameter).
pub const BTK_QAV_F64: f64 = 0.343_412_5_f64;
pub const BTK_BETA_F64: f64 = 1.990_328_f64;
/// UPSTREAM fudge — `btk.cpp:20`. NOT `f64::EPSILON`; do not substitute.
pub const BTK_FUDGE_F64: f64 = 1e-24_f64;

// --- P86 (p86c.cpp — placeholder values; body lands in 03-04) ----------

/// P86 f parameter (`p86c.cpp`). Placeholder for plan 03-04 — revisit value
/// against `p86c.cpp:17` when that plan executes.
pub const P86_F_F64: f64 = 0.11_f64;

// --- B97 family --------------------------------------------------------
//
// Coefficient tables extracted from xcfun-master/src/functionals/{b97x.hpp,
// b97c.hpp, b97xc.hpp, b97-1xc.cpp, b97-2xc.cpp}. Values verbatim from
// 03-RESEARCH.md §Appendix "B97 coefficient tables" (lines 822-833) which are
// themselves verbatim copies of the upstream literal arrays.

pub const B97_GAMMA_X_F64: f64 = 0.004_f64;
pub const B97_GAMMA_C_PAR_F64: f64 = 0.2_f64;
pub const B97_GAMMA_C_ANTIPAR_F64: f64 = 0.006_f64;

pub const B97_X_COEF: [f64; 3] = [0.8094_f64, 0.5073_f64, 0.7481_f64];
pub const B97_1X_COEF: [f64; 3] = [0.789_518_f64, 0.573_805_f64, 0.660_975_f64];
pub const B97_2X_COEF: [f64; 3] = [0.827_642_f64, 0.047_840_f64, 1.761_25_f64];
pub const B97_C_PAR_COEF: [f64; 3] = [0.173_7_f64, 2.348_7_f64, -2.486_8_f64];
pub const B97_C_ANTIPAR_COEF: [f64; 3] = [0.945_4_f64, 0.747_1_f64, -4.596_1_f64];
pub const B97_1C_PAR_COEF: [f64; 3] = [0.082_001_1_f64, 2.716_81_f64, -2.871_03_f64];
pub const B97_1C_ANTIPAR_COEF: [f64; 3] = [0.955_689_f64, 0.788_552_f64, -5.478_69_f64];
pub const B97_2C_PAR_COEF: [f64; 3] = [0.585_808_f64, -0.691_682_f64, 0.394_796_f64];
pub const B97_2C_ANTIPAR_COEF: [f64; 3] = [0.999_849_f64, 1.406_26_f64, -7.440_60_f64];

// --- LSDA prefactors (cross-family reuse) ------------------------------

/// `-(81 / (32·π))^(1/3)` — negated Slater exchange constant. Mirrors Phase 2
/// `slaterx.rs::NEG_C_SLATER_F64`.
pub const NEG_C_SLATER_F64: f64 = -0.930_525_736_349_100_1_f64;

/// `(81 / (32·π))^(1/3)` — positive slaterx factor for kernels that want the
/// absolute value (e.g. Becke's pre-scaled exchange body).
pub const C_SLATER_F64: f64 = 0.930_525_736_349_100_1_f64;

/// PW91-style exchange LSDA prefactor coefficient
/// `-0.75 · 2^(1/3) · (3π²)^(1/3) / π` = `NEG_C_SLATER_F64` (identities agree
/// analytically to f64 precision per `pw9xx.hpp:51-63`).
pub const PREFACTOR_X_LSDA_F64: f64 = -0.930_525_736_349_099_9_f64;
