//! Shared `#[cube] fn` helpers for GGA functional bodies (D-08).
//!
//! Every helper lives here so that PBE / Becke / LYP / OPTX / PW91 / B97 /
//! KT / BTK / P86 / APBE family members can `use crate::functionals::gga::shared::{...}`
//! without duplicating formulas. Scalar constants are centralised in
//! `constants` as `pub const <NAME>_F64: f64` — kernel bodies cast via
//! `F::cast_from(<NAME>_F64)` per S2 (PATTERNS §D6).

pub mod b97_poly;
pub mod constants;
pub mod optx;
pub mod pbec_eps;
pub mod pbex;
pub mod pw91_like;
