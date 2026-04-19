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
pub mod index;

// Cubecl-bearing modules populated in later plans:
//   Plan 01-03/04: expand, tfuns
//   Plan 01-06: math

#[cfg(feature = "testing")]
pub mod for_tests;

pub use index::{CNST, VAR0, VAR1, VAR2, VAR3, VAR4, VAR5, VAR6, VAR7};
