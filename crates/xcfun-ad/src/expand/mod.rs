//! Scalar Taylor-series expansions ported from
//! `xcfun-master/external/upstream/taylor/tmath.hpp`. One file per C++
//! function (D-09). Each fn writes coefficients into a caller-provided
//! `Array<F>` of length `n + 1`.
//!
//! # Plan 01-03 scope
//!
//! This plan lands the six "primary" expansions whose recurrence is a
//! straight loop: `inv`, `exp`, `log`, `pow`, `sqrt`, `cbrt`.
//! The transcendental expansions (`atan`, `gauss`, `erf`, `asinh`) that
//! depend on the `tfuns` helpers land in Plan 01-04.
//!
//! # Preconditions
//!
//! Every expansion with an analyticity precondition (see per-file
//! headers) documents it textually — the C++ reference enforces it via
//! `assert!` (tmath.hpp). Cubecl 0.10-pre.3's `#[cube]` macro rejects
//! host-style assertion macros inside kernel bodies ("Unsupported
//! macro"), so D-11's "active in release" check is fulfilled via
//! CONTEXT.md D-05's fallback: host-side caller verification before the
//! kernel launch. Silent-NaN propagation remains the primary correctness
//! risk (Pitfall P10).
//!
//! # Cubecl 0.10-pre.3 idioms used here
//!
//! - Scalar float literal: `F::new(val: f32)`; exact for small integer
//!   constants. `F::cast_from(i)` casts a `u32` loop counter into `F`.
//! - Intrinsics are method-form: `x.exp()`, `x.sqrt()`, `x.powf(a)`,
//!   `x.ln()` (see `cubecl-core::frontend::operation::unary` for the
//!   trait surface).
//! - There is no `F::cbrt` — cbrt is implemented as `x.powf(1/3)` with
//!   a documented 1–2 ULP drift vs. C++ `std::cbrt` on cubecl-cpu.

pub mod exp;
pub mod inv;
pub mod log;
// Task 2 adds: `pow`, `sqrt`, `cbrt`.
// Plan 01-04 adds: `atan`, `gauss`, `erf`, `asinh` (alongside `tfuns`).
