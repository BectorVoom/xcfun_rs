//! `xcfun-eval` — cubecl launcher + functional bodies for xcfun_rs.
//!
//! This crate hosts the `#[cube] fn` bodies for the 11 LDA functionals
//! (Phase 2 — Plans 02-04/02-05), the `DensVarsDev<F>` `#[derive(CubeType, CubeLaunch)]`
//! type, the `build_densvars` dispatcher, the `regularize` kernel, and the minimal
//! `Functional` struct + `eval` entry point used by tier-1 self-tests + the tier-2
//! parity harness (Plan 02-06).
//!
//! See `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md`
//! for the 25 locked design decisions (D-01..D-25) driving this crate.
#![forbid(unsafe_code)]

#[cfg(feature = "testing")]
pub mod for_tests;
