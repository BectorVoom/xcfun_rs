//! Runtime probe + client modules.
//!
//! Phase 6 Plan 06-02a — only the `cpu` arm is functional today. Plans
//! 06-03 / 06-04 fill in the `hip` / `cuda` / `wgpu` arms with actual
//! cubecl-runtime instantiation + f64 device probes (CONTEXT D-13/D-13-A
//! for Wgpu; W-7 revision-1 for CUDA).
//!
//! Each submodule exposes a `*_available()` probe returning `bool`.
//! `auto_backend()` consults each probe in CONTEXT D-07 priority order.

#[cfg(feature = "cpu")]
pub mod cpu;

#[cfg(feature = "hip")]
pub mod hip;

#[cfg(feature = "cuda")]
pub mod cuda;

#[cfg(feature = "wgpu")]
pub mod wgpu;
