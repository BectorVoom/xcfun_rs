//! `cubecl-wgpu` probe stubs (Wgpu generic + Metal-via-Wgpu).
//!
//! Phase 6 Plan 06-02a ships both probes as constant `false`; Plan 06-04
//! wires the actual `wgpu::Features::SHADER_F64` probes:
//!
//! - `wgpu_with_shader_f64_available` checks whether the default Wgpu
//!   adapter reports `SHADER_F64`. Returns `false` on Apple Silicon (no
//!   hardware f64) and on most consumer Vulkan drivers without the f64
//!   extension. Returning `false` here makes `auto_backend()` fall
//!   through to `Cpu`; explicit `Backend::Wgpu` callers go through
//!   `Batch::open` which surfaces `XcError::WgpuNoF64` per D-13/D-13-A.
//! - `metal_with_f64_available` is the equivalent probe restricted to
//!   the Metal backend — Apple Silicon GPUs lack hardware f64 and the
//!   probe must refuse rather than silently downgrade to f32.

/// Plan 06-02a stub — Plan 06-04 replaces with a real
/// `client.properties().feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))`
/// probe (RESEARCH §"Verified pattern: feature probing for a specific
/// float type"; cubecl 0.10.0-pre.3 API path verified W-4 / W-10).
pub fn wgpu_with_shader_f64_available() -> bool {
    false
}

/// Plan 06-02a stub — Plan 06-04 replaces with a real Metal-backend
/// f64 probe.
pub fn metal_with_f64_available() -> bool {
    false
}
