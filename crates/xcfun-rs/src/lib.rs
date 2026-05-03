//! xcfun-rs — native Rust public API for xcfun_rs (Phase 5).
//!
//! Stable Rust facade over `xcfun-eval::Functional`. Decouples the
//! public surface from cubecl internals (Phase 5 D-02).
//!
//! # Public surface (RS-01..10)
//! - `Functional` newtype + 8 methods.
//! - 11 free functions: see [`free_fns`].
//! - Re-exports of public types: [`Mode`], [`Vars`], [`XcError`],
//!   [`ParameterId`], [`FunctionalId`], [`Dependency`].

#![forbid(unsafe_code)]

mod functional;
mod free_fns;

pub use functional::{Functional, XCFUN_MIN_BATCH_SIZE, min_batch_size};
pub use free_fns::{
    authors, describe_long, describe_short, enumerate_aliases,
    enumerate_parameters, is_compatible_library, self_test, splash,
    version, which_mode, which_vars,
};
pub use xcfun_core::{Dependency, FunctionalId, Mode, ParameterId, Vars, XcError};
