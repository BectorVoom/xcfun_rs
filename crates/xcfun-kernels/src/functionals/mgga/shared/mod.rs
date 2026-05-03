//! Shared `#[cube] fn` helpers for metaGGA functional bodies (D-01-A).
//!
//! Mirrors the GGA pattern at `crates/xcfun-eval/src/functionals/gga/shared/`:
//! every helper lives here so Wave 1/2/3 family modules can import without
//! duplicating formulas. Scalar constants are centralised in `constants` as
//! `pub const <NAME>_F64: f64` — kernel bodies cast via
//! `F::cast_from(<NAME>_F64)` per S2 (PATTERNS §S.4).
//!
//! Helpers and their port sources:
//!
//! - `constants` — TPSS / SCAN / M0x / BLOCX / CSC scalar constants.
//! - `tpss_like` — `tpssx_eps.hpp` / `tpssc_eps.hpp` / `revtpssx_eps.hpp` /
//!   `revtpssc_eps.hpp` fused. F_x, fx_unif, tpssc_eps, revtpssx_eps,
//!   revtpssc_eps.
//! - `scan_like` — `SCAN_like_eps.hpp` (522 LOC); IDELEC comptime dispatch.
//!   Exports get_SCAN_Fx, r2SCAN_C, scan_ec0/ec1, lda_0, gcor2, get_lsda1,
//!   ufunc.
//! - `m0x_like` — `m0xy_fun.hpp` (262 LOC). M05 + M06 substrate.
//! - `br_like` — `brx.cpp:78-101` `BR(t)` ctaylor adapter + `polarized`
//!   helper. Composes `xcfun_ad::ctaylor_br_inverse` (Wave 0 Task 1).
//! - `blocx` — `blocx.cpp:18-46` (independent of BRX per RESEARCH).
//! - `cs` — `cs.cpp:17-27` CSC inline body.

pub mod constants;
pub mod tpss_like;
pub mod scan_like;
pub mod m0x_like;
pub mod br_like;
pub mod blocx;
pub mod cs;
