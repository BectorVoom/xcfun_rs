//! Scalar constants used by metaGGA functional bodies.
//!
//! Centralised here so kernel/helper bodies cast via `F::cast_from(NAME_F64)`
//! to preserve f64 precision (Phase 2 ACC-04 lesson — `F::new(f32)` carries
//! ≥ 6e-9 absolute error, breaking the strict 1e-12 parity contract).
//!
//! Sources:
//! - TPSS:    `xcfun-master/src/functionals/tpssx_eps.hpp:29-33`,
//!            `xcfun-master/src/functionals/tpssc_eps.hpp:59`
//! - revTPSS: `xcfun-master/src/functionals/revtpssx_eps.hpp:28-32`
//! - SCAN:    `xcfun-master/src/functionals/SCAN_like_eps.hpp:82-84` + family
//!            constants (multiple variants).
//! - M0x:     `xcfun-master/src/functionals/m0xy_fun.hpp:35-46`
//! - BLOCX:   `xcfun-master/src/functionals/blocx.cpp:20-24`
//! - CSC:     `xcfun-master/src/functionals/cs.cpp:18-21`

// ---------------------------------------------------------------------------
//  TPSS family constants (tpssx_eps.hpp + tpssc_eps.hpp).
// ---------------------------------------------------------------------------

/// TPSS exchange `kappa` parameter — `tpssx_eps.hpp:29`.
pub const TPSS_KAPPA_F64: f64 = 0.804_f64;
/// TPSS exchange `mu` parameter — `tpssx_eps.hpp:30`.
pub const TPSS_MU_F64: f64 = 0.21951_f64;
/// TPSS exchange `b` parameter — `tpssx_eps.hpp:31`.
pub const TPSS_B_F64: f64 = 0.40_f64;
/// TPSS exchange `e` parameter — `tpssx_eps.hpp:32`.
pub const TPSS_E_F64: f64 = 1.537_f64;
/// TPSS exchange `c` parameter — `tpssx_eps.hpp:33`.
pub const TPSS_C_F64: f64 = 1.59096_f64;
/// `sqrt(TPSS_E_F64)` — pre-computed because `f64::sqrt` is not `const fn`.
pub const TPSS_SQRT_E_F64: f64 = 1.239_758_040_909_596_f64;

/// TPSS correlation `dd` parameter — `tpssc_eps.hpp:59`.
pub const TPSS_DD_F64: f64 = 2.8_f64;

// ---------------------------------------------------------------------------
//  revTPSS family constants (revtpssx_eps.hpp + revtpssc_eps.hpp).
// ---------------------------------------------------------------------------

/// revTPSS exchange `kapa` parameter — `revtpssx_eps.hpp:28`.
pub const REVTPSS_KAPPA_F64: f64 = 0.804_f64;
/// revTPSS exchange `mu` parameter — `revtpssx_eps.hpp:29`.
pub const REVTPSS_MU_F64: f64 = 0.14_f64;
/// revTPSS exchange `b` parameter — `revtpssx_eps.hpp:30`.
pub const REVTPSS_B_F64: f64 = 0.40_f64;
/// revTPSS exchange `e` parameter — `revtpssx_eps.hpp:31`.
pub const REVTPSS_E_F64: f64 = 2.1677_f64;
/// revTPSS exchange `c` parameter — `revtpssx_eps.hpp:32`.
pub const REVTPSS_C_F64: f64 = 2.35204_f64;
/// `sqrt(REVTPSS_E_F64)` — pre-computed because `f64::sqrt` is not `const fn`.
pub const REVTPSS_SQRT_E_F64: f64 = 1.472_311_108_427_834_9_f64;

// ---------------------------------------------------------------------------
//  SCAN family constants (SCAN_like_eps.hpp).
//
//  Per CONTEXT D-01-A, scan_like.rs `get_SCAN_Fx` takes a comptime IDELEC
//  selector (0=SCAN, 1=rSCAN, 2=r++SCAN, 3=r2SCAN, 4=r4SCAN). The constants
//  below are SCAN's defaults; per-variant overrides live inside scan_like.rs
//  (gated by `if comptime!(idelec == k)` arms).
// ---------------------------------------------------------------------------

/// SCAN-family `ETA` regularisation — `SCAN_like_eps.hpp:82`.
pub const SCAN_ETA_F64: f64 = 1.0e-3_f64;
/// SCAN-family `TAU_R` — `SCAN_like_eps.hpp:83`.
pub const SCAN_TAU_R_F64: f64 = 1.0e-4_f64;
/// SCAN-family `A_REG` — `SCAN_like_eps.hpp:84`.
pub const SCAN_A_REG_F64: f64 = 1.0e-3_f64;

/// SCAN exchange `kappa` parameter (Sun-Ruzsinszky-Perdew 2015).
pub const SCAN_KAPPA_F64: f64 = 0.804_f64;
/// SCAN gradient enhancement coefficient `MUK` = `10/81` (Sun-Ruzsinszky-Perdew 2015).
pub const SCAN_MUK_F64: f64 = 10.0_f64 / 81.0_f64;

// ---------------------------------------------------------------------------
//  M05/M06 family constants (m0xy_fun.hpp).
// ---------------------------------------------------------------------------

/// M0x exchange `alpha` — `m0xy_fun.hpp:35`.
pub const M0X_ALPHA_X_F64: f64 = 0.001_867_26_f64;
/// M0x correlation parallel `alpha` — `m0xy_fun.hpp:36`.
pub const M0X_ALPHA_C_PARALLEL_F64: f64 = 0.005_150_88_f64;
/// M0x correlation antiparallel `alpha` — `m0xy_fun.hpp:37`.
pub const M0X_ALPHA_C_ANTIPARALLEL_F64: f64 = 0.003_049_66_f64;
/// M0x scale-factor TF constant — `m0xy_fun.hpp:46`.
///
/// `(2^(5/2))^(2/3) = 2^(5/3) = 3.174_802_103_936_40`. Used as
/// multiplicative correction to the standard Thomas-Fermi `C_F`.
pub const M0X_SCALEFACTOR_TF_F64: f64 = 3.174_802_103_936_40_f64;

// ---------------------------------------------------------------------------
//  BLOCX constants (blocx.cpp:20-24). Reuses TPSS values per-port.
// ---------------------------------------------------------------------------

/// BLOCX `kappa` parameter — `blocx.cpp:20` (== TPSS).
pub const BLOCX_KAPPA_F64: f64 = 0.804_f64;
/// BLOCX `mu` parameter — `blocx.cpp:21` (== TPSS).
pub const BLOCX_MU_F64: f64 = 0.21951_f64;
/// BLOCX `b` parameter — `blocx.cpp:22` (== TPSS).
pub const BLOCX_B_F64: f64 = 0.40_f64;
/// BLOCX `e` parameter — `blocx.cpp:23` (== TPSS).
pub const BLOCX_E_F64: f64 = 1.537_f64;
/// BLOCX `c` parameter — `blocx.cpp:24` (== TPSS).
pub const BLOCX_C_F64: f64 = 1.59096_f64;

// ---------------------------------------------------------------------------
//  CSC constants (cs.cpp:18-21). All four are 1.0 in upstream.
// ---------------------------------------------------------------------------

/// CSC `a` coefficient — `cs.cpp:18`.
pub const CSC_A_F64: f64 = 1.0_f64;
/// CSC `b` coefficient — `cs.cpp:19`.
pub const CSC_B_F64: f64 = 1.0_f64;
/// CSC `c` coefficient — `cs.cpp:20`.
pub const CSC_C_F64: f64 = 1.0_f64;
/// CSC `d` coefficient (named `dpar` upstream to avoid collision with
/// densvars `d`) — `cs.cpp:21`.
pub const CSC_DPAR_F64: f64 = 1.0_f64;
