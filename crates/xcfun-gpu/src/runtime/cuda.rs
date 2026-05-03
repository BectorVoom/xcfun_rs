//! `cubecl-cuda` probe stub.
//!
//! Phase 6 Plan 06-02a ships the probe as a constant `false`; Plan 06-04
//! wires the actual runtime probe via
//! `client.properties().feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))`
//! and surfaces the symmetric `XcError::CudaNoF64` (W-7 revision-1) when
//! the device fails the f64 check.

/// Plan 06-02a stub — Plan 06-04 replaces with a real cubecl-cuda
/// `client.properties()` probe.
pub fn cuda_available() -> bool {
    false
}
