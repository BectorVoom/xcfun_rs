//! Production CPU substrate. Phase 6 Plan 06-06 (D-12) promoted this
//! module from `testing`-gated to `cpu`-gated — `xcfun-rs::Functional`'s
//! reusable handle and `xcfun-gpu::runtime::cpu` both depend on
//! `cpu_client()` at eval time, not just test time.  The module name
//! remains `for_tests` to keep import paths stable across the workspace
//! (rename is deferred to a later plan to avoid touching ~30 import sites).
//!
//! Mirrors `xcfun-ad::for_tests::cpu_client` — independent OnceLock<CpuClient>
//! per crate (cubecl-cpu allows this; both point at the same physical
//! CpuDevice).

use cubecl::prelude::*;
use cubecl_cpu::{CpuDevice, CpuRuntime};
use std::sync::OnceLock;

/// Concrete cubecl-cpu compute client. Exported as a type alias so the
/// crate's re-export stays stable across minor cubecl 0.10-pre.N drift.
pub type CpuClient = ComputeClient<CpuRuntime>;

static CPU_CLIENT: OnceLock<CpuClient> = OnceLock::new();

/// Shared cubecl-cpu client, initialised on first call.
///
/// Returns a `&'static CpuClient`; every caller in the test binary receives
/// the same underlying client (pointer equality). Mirrors Phase 1
/// `xcfun-ad::for_tests::cpu_client` verbatim.
pub fn cpu_client() -> &'static CpuClient {
    CPU_CLIENT.get_or_init(|| {
        let device = CpuDevice;
        CpuRuntime::client(&device)
    })
}
