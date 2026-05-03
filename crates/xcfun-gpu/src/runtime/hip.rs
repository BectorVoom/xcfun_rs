//! `cubecl-hip` (ROCm/AMD) probe stub.
//!
//! Phase 6 Plan 06-02a ships the probe as a constant `false`; Plan 06-03
//! wires the actual runtime probe + tier-3 strict-1e-13 validation
//! harness extension. Until then, `auto_backend()` falls through to
//! `Cuda` / `Metal` / `Wgpu` / `Cpu`.

/// Plan 06-02a stub — Plan 06-03 replaces with a real cubecl-hip
/// `client.properties()` probe + RDNA-2 `HSA_OVERRIDE_GFX_VERSION`
/// documentation note.
pub fn rocm_available() -> bool {
    false
}
