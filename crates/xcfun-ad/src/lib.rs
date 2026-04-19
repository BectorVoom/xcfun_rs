//! xcfun-ad — automatic differentiation engine for xcfun_rs.
//!
//! Phase 1 scope (see `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/`):
//! - `CTaylor<T, const N: usize>` bit-flag-indexed multilinear polynomial
//! - `ValidN<N>` sealed trait bounding `N ∈ 0..=7`
//! - `Num` trait + `*_expand` scalar series ports (Wave 1/2)
//!
//! Plan 01-01 (Wave 0) lands manifest/config/bench scaffolding only.
//! Plan 01-02/03/04 (Wave 1) adds `expand`, `ctaylor_rec`, `tfuns`.
//! Plan 01-05/06/07 (Wave 2) adds `num`, `math`, tests, benches.

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
