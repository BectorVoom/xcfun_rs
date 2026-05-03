//! `xcfun-eval` — per-point cubecl-cpu launcher + `Functional` struct.
//!
//! After Phase 6 Plan 06-01 (D-08), this crate retains:
//!   - `Functional` struct + `eval` / `eval_setup` per-point entry point
//!     (`functional.rs`, including the `eval_point_kernel`
//!     `#[cube(launch_unchecked)]` adapter and the `run_launch` substrate).
//!   - `for_tests::cpu_client()` cubecl-cpu `OnceLock` substrate (Phase 6
//!     Plan 06-06 will promote this from `for_tests` to production).
//!   - `functionals::contracted::launch_contracted` — host-side
//!     `Mode::Contracted` dispatcher (depends on `Functional` + `run_launch`).
//!
//! All per-functional `#[cube]` kernel bodies (lda/, gga/, mgga/, plus the
//! `potential` adapter kernel), `DensVarsDev<F>`, `build_densvars`,
//! `regularize`, and `dispatch_kernel` migrated to the new
//! `xcfun-kernels` crate. The 11 test files in `crates/xcfun-eval/tests/`
//! that previously imported `xcfun_eval::functionals::*` /
//! `xcfun_eval::density_vars::*` / `xcfun_eval::dispatch::*` import via
//! `xcfun_kernels::*` directly — no re-export shim is provided here
//! (the test count is small enough that a clean import migration is
//! preferred over a long-lived re-export per W-8 revision-1).
//!
//! See `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md`
//! for the original D-01..D-25 design decisions and
//! `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md`
//! for the D-08 split rationale.
// `forbid(unsafe_code)` in non-test builds; tests/launch paths need `unsafe`
// to call cubecl's `launch_unchecked`, so we downgrade to `deny` and locally
// `#[allow(unsafe_code)]` the specific launch call sites.
#![cfg_attr(not(feature = "testing"), forbid(unsafe_code))]
#![cfg_attr(feature = "testing", deny(unsafe_code))]

pub mod functional;
pub mod functionals;

#[cfg(feature = "testing")]
pub mod for_tests;

pub use functional::Functional;
