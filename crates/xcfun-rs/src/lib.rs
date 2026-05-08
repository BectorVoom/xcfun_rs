// Public facade with `Functional::eval(...)` that mirrors xcfun-eval's
// many-arg signature; reducing them would require a builder layer outside
// the Phase 5 D-02 contract. Allow the linted argument count.
#![allow(clippy::too_many_arguments)]

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

// Phase 6 Plan 06-06 (D-12): downgrade `forbid(unsafe_code)` → `deny`. The
// `Functional` newtype now carries an `UnsafeCell<EvalHandle>` reusable
// buffer set (RS-07 strict zero-alloc plumbing) which requires explicit
// `unsafe impl Send/Sync` markers — `forbid` rejects them outright. With
// `deny`, local `#[allow(unsafe_code)]` on the marker impls (in
// `functional.rs`) is permitted; every `unsafe` block / impl carries a
// SAFETY comment documenting the invariants.
#![deny(unsafe_code)]

mod free_fns;
mod functional;

pub use free_fns::{
    authors, describe_long, describe_short, enumerate_aliases, enumerate_parameters,
    is_compatible_library, self_test, splash, version, which_mode, which_vars,
};
pub use functional::{Functional, XCFUN_MIN_BATCH_SIZE, min_batch_size};
pub use xcfun_core::{Dependency, FunctionalId, Mode, ParameterId, Vars, XcError};
