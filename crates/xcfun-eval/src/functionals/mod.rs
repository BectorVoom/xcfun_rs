//! Host-side launcher modules retained in `xcfun-eval` after Phase 6 Plan
//! 06-01 (D-08) migrated the per-functional `#[cube]` bodies (lda/, gga/,
//! mgga/) and the Mode::Potential adapter kernel (`potential.rs`) to
//! `xcfun-kernels::functionals`.
//!
//! `contracted.rs` STAYS here because it is a host-side launcher that
//! depends on `crate::functional::Functional` + `crate::functional::run_launch`
//! — both of which are per-point cubecl-cpu validation substrate that
//! belongs with `Functional` in `xcfun-eval` (per CONTEXT D-08: kernel
//! bodies live in xcfun-kernels; per-point validation substrate lives in
//! xcfun-eval).

pub mod contracted;
