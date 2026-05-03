//! `cubecl-cpu` substrate — always-on per CONTEXT D-08.
//!
//! Re-exports the `OnceLock<CpuClient>` from `xcfun-eval::for_tests`. Plan
//! 06-06 (D-12) will promote that helper from `for_tests` to a production
//! module; until then we route through the existing accessor under the
//! `xcfun-eval/testing` feature (transitively pulled in by xcfun-gpu's
//! `cpu` feature flag).

pub use xcfun_eval::for_tests::{cpu_client, CpuClient};

/// CPU is always available — the substrate is statically linked once the
/// `cpu` feature flag is enabled (default).
pub fn cpu_available() -> bool {
    true
}
