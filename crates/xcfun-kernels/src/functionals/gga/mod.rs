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
pub mod becke;
pub mod lyp;
pub mod pbe;

// Wave 3 (03-03) ships OPTX (2), PW91 family (4), P86 (2), APBE (2).
pub mod apbe;
pub mod optx;
pub mod p86;
pub mod pw91;

// Wave 4 (03-04) ships B97 family (6) + KT (2: KTX + BTK; CSC deferred per D-01-A).
pub mod b97;
pub mod kt;
