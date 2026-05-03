//! APBE family kernels — Wave 3 (plan 03-03), GGA-08.
//!
//! - `apbex` — XC_APBEX (id=68): APBE exchange (PBE-form with μ=0.26, κ=0.804).
//! - `apbec` — XC_APBEC (id=67): APBE correlation (PBE-correlation with β=0.07903).

pub mod apbex;
pub mod apbec;
