//! `xcfun-ad` — cubecl-native Taylor-algebra AD engine for xcfun_rs.
//!
//! This crate implements `CTaylor<F, N>` as a pure `#[cube]` type backed
//! by `cubecl::prelude::Array<F>` of length `1 << N`. All arithmetic
//! operations and every `*_expand` scalar series function from
//! `xcfun-master/external/upstream/taylor/` are ported verbatim as
//! `#[cube] fn` generic over `F: Float` (cubecl's Float trait). See
//! `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`
//! for the 28 locked design decisions driving this crate.
#![forbid(unsafe_code)]

pub mod ctaylor;
pub mod ctaylor_rec;
pub mod expand;
pub mod index;
pub mod math;
pub mod tfuns;

#[cfg(feature = "testing")]
pub mod for_tests;

pub use index::{CNST, VAR0, VAR1, VAR2, VAR3, VAR4, VAR5, VAR6, VAR7};

// Phase 4 plan 04-00 Task 1 — BR Newton-inverse primitive (D-02).
pub use expand::br_inverse::{br_inverse_expand, br_scalar, br_z};
pub use math::ctaylor_br_inverse;
