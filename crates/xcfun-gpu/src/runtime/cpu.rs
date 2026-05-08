//! `cubecl-cpu` substrate — always-on per CONTEXT D-08.
//!
//! Re-exports the `OnceLock<CpuClient>` from `xcfun-eval::for_tests`.
//!
//! Plan 06-06 (D-12) PROMOTED that helper from a `testing`-gated test
//! module to a `cpu`-gated production module — `for_tests::cpu_client()`
//! is now part of the production CPU substrate, no longer test-only.
//! The module name remains `for_tests` (rename deferred to avoid touching
//! ~30 workspace-wide import sites).

pub use xcfun_eval::for_tests::{CpuClient, cpu_client};

/// CPU is always available — the substrate is statically linked once the
/// `cpu` feature flag is enabled (default).
pub fn cpu_available() -> bool {
    true
}
