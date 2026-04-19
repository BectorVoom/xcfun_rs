//! Singleton `CpuClient` wrapper for cubecl-cpu. Matches the pattern from
//! CONTEXT.md D-17: one client per test binary, shared across every test.
//!
//! # Cubecl 0.10-pre.3 API delta from plan <interfaces>
//!
//! The plan's <interfaces> block assumed `<CpuRuntime as Runtime>::Client`,
//! but cubecl 0.10-pre.3's `Runtime` trait has no `type Client`. The
//! concrete client is `cubecl::prelude::ComputeClient<R>`, so the alias
//! below is `ComputeClient<CpuRuntime>`. This keeps the downstream API
//! `&'static CpuClient` identical even though the underlying type
//! expression differs from the plan's assumed form.

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
/// the same underlying client (pointer equality). See CONTEXT.md D-17.
pub fn cpu_client() -> &'static CpuClient {
    CPU_CLIENT.get_or_init(|| {
        let device = CpuDevice;
        CpuRuntime::client(&device)
    })
}
