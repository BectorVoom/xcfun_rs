//! xcfun-ad — automatic differentiation engine for xcfun_rs.
//!
//! Phase 1 scope (see `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/`):
//! - `CTaylor<T, const N: usize>` bit-flag-indexed multilinear polynomial
//!   (Plan 01-01 Wave 0: struct shape, elementwise ops; Plan 01-03 Wave 1:
//!   recursion-structured `mul`)
//! - `ValidN<N>` sealed trait bounding `N ∈ 0..=7` (Plan 01-01 Wave 0)
//! - `*_expand` scalar series ports mirroring
//!   `xcfun-master/external/upstream/taylor/tmath.hpp` (Plan 01-02 Wave 1)
//! - `Num` trait + composed elementary functions (`reciprocal`, `sqrt`,
//!   `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`) on `CTaylor<f64, N>`
//!   (Plan 01-05 Wave 2)
//!
//! # Bit-flag indexing
//!
//! `CTaylor<T, N>` stores `[T; 1 << N]` with each index a bitmask over `N`
//! variables. `CNST = 0` is the constant term; `VAR_k = 1 << k` is the
//! first-order coefficient of variable `k`. Mixed monomials (e.g.
//! `x_i * x_j`) live at the OR of their single-variable flags. See
//! `docs/design/02-data-structures.md §1` and CONTEXT.md D-01.
//!
//! # Safety / invariants
//!
//! - `#![forbid(unsafe_code)]` at crate root — no unsafe blocks anywhere.
//! - Stack-only storage: no `Box`, no `Vec`, no heap allocation (CONTEXT.md
//!   D-01).
//! - `assert!` — not `debug_assert!` — on every `*_expand` precondition
//!   (CONTEXT.md D-11), so release builds still catch silent-NaN regressions.
//! - Convention (D-13): prefer `powi` over `pow` whenever the exponent is a
//!   literal integer. Enforced at call sites by lint in Phase 2.

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod ctaylor;
pub mod valid_n;

// Wave 1 adds: pub mod ctaylor_rec; pub mod expand; pub mod tfuns;
// Wave 2 adds: pub mod num; pub mod math;

#[cfg(feature = "testing")]
pub mod for_tests;

pub use ctaylor::CTaylor;
pub use valid_n::{Bound, ValidN};

/// Index of the constant (non-derivative) coefficient in `CTaylor::c`.
pub const CNST: usize = 0;

/// Bit-flag index of variable 0 in `CTaylor::c`.
pub const VAR0: usize = 1 << 0;
/// Bit-flag index of variable 1 in `CTaylor::c`.
pub const VAR1: usize = 1 << 1;
/// Bit-flag index of variable 2 in `CTaylor::c`.
pub const VAR2: usize = 1 << 2;
/// Bit-flag index of variable 3 in `CTaylor::c`.
pub const VAR3: usize = 1 << 3;
/// Bit-flag index of variable 4 in `CTaylor::c`.
pub const VAR4: usize = 1 << 4;
/// Bit-flag index of variable 5 in `CTaylor::c`.
pub const VAR5: usize = 1 << 5;
/// Bit-flag index of variable 6 in `CTaylor::c`.
pub const VAR6: usize = 1 << 6;
/// Bit-flag index of variable 7 in `CTaylor::c` (contracted-mode slot;
/// storage upper bound per CONTEXT.md D-01).
pub const VAR7: usize = 1 << 7;
