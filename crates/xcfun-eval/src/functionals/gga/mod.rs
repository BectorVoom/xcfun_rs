//! Generalised-Gradient-Approximation (GGA) functional bodies.
//!
//! Phase 3 ships **36 GGA functional IDs** per D-01-A amendment:
//! BRX (10) / BRC (11) / BRXC (12) + CSC (66) defer to Phase 4 (metaGGA tier)
//! — they declare `Dependency::KINETIC|LAPLACIAN|JP` requiring an inlen=11
//! `Vars` arm not in D-10, plus a separate `BR_taylor` Newton-inverse algebra.
//!
//! # Layout
//!
//! - `shared::` — cross-family helpers (pbex, pw91_like, pbec_eps, b97_poly,
//!   optx). Populated Wave 1 (this plan) as signatures + full bodies for
//!   `pbex::enhancement`, `pbex::energy_pbe_ab`, `pw91_like::s2`; other helpers
//!   land as SKELETONS with `// SKELETON — full body lands in 03-XX Task Y`
//!   markers per W3 resolution.
//! - Family modules (`pbe`, `becke`, `lyp`, `optx`, `pw91`, `p86`, `apbe`,
//!   `b97`, `kt`) land in plans 03-02 through 03-04.

pub mod shared;

// Family modules — Wave 2 (03-02) ships PBE family (12), Becke (4), LYP (1).
pub mod pbe;
pub mod becke;
pub mod lyp;

// Wave 3 (03-03) ships OPTX (2), PW91 family (4), P86 (2), APBE (2).
pub mod optx;
pub mod pw91;
pub mod p86;
pub mod apbe;

// Other family modules — populated in later waves of Phase 3:
// pub mod b97;    // 03-04 (B97X, B97-1X, B97-2X, B97C, B97-1C, B97-2C, B97XC, …)
// pub mod kt;     // 03-03 (KTX, BTK)
