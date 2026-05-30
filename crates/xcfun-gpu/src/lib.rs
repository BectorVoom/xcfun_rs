//! # xcfun-gpu
//!
//! GPU batch lifecycle + `auto_backend` dispatch for `xcfun_rs`.
//!
//! Phase 6 Plan 06-02a — skeleton only. Plans 06-03 / 06-04 wire the
//! `cubecl-hip` / `cubecl-cuda` / `cubecl-wgpu` runtimes behind their
//! respective feature flags. This plan ships the always-on `cpu` arm
//! (cubecl-cpu re-exported from `xcfun-eval` per CONTEXT D-08) and the
//! type / dispatch skeleton that downstream plans build atop.
//!
//! ## Public surface
//!
//! - [`Backend`] — runtime discriminator (5 variants per CONTEXT D-07).
//! - [`Batch`] — lifecycle wrapper for `cubecl::Runtime`-bound dispatch.
//!   Bound to `&'fun xcfun_eval::Functional` per W-3 revision-1 (avoids
//!   `xcfun-rs` ↔ `xcfun-gpu` cycle when Plan 06-05 wires `eval_vec`).
//! - [`auto_backend`] — priority-chain selector (D-07): env
//!   `XCFUN_FORCE_BACKEND` → ROCm → CUDA → Metal-with-f64 → Wgpu-with-f64
//!   → CPU.
//! - [`error_routing::must_fall_back_to_cpu`] — ERF-bearing functionals on
//!   Wgpu/Metal route to CPU per GPU-05.
//!
//! ## Compile-time invariants
//!
//! - f64 size is 8 bytes (the cubecl-wgpu silent-f32-downgrade pitfall —
//!   Pitfall 2 in 06-RESEARCH.md). The static assertion below catches an
//!   accidental f32 monomorphisation at compile time.
//!
//! ## Feature flags
//!
//! | Feature | Pulls in | Default? |
//! |---------|----------|----------|
//! | `cpu`   | `cubecl-cpu` (via `xcfun-eval`'s `testing` feature) | yes |
//! | `hip`   | `cubecl-hip` — Plan 06-03 wires the probe + launch arms | no |
//! | `cuda`  | `cubecl-cuda` — Plan 06-04 wires the probe + launch arms | no |
//! | `wgpu`  | `cubecl-wgpu` — Plan 06-04 wires the probe + launch arms | no |
//! | `metal` | alias for `wgpu` (no separate cubecl-metal crate exists, see RESEARCH §"Pitfall 9") | no |

#![cfg_attr(not(feature = "cpu"), forbid(unsafe_code))]
#![cfg_attr(feature = "cpu", deny(unsafe_code))]
#![deny(unsafe_op_in_unsafe_fn)]

// f64 invariant — see Pitfall 2 in 06-RESEARCH.md.
const _: () = assert!(core::mem::size_of::<f64>() == 8);

pub mod auto_backend;
pub mod backend;
pub mod batch;
pub mod error_routing;
pub mod pool;
pub mod runtime;

pub use auto_backend::auto_backend;
pub use backend::Backend;
pub use batch::Batch;

// Phase 6 Plan 06-05 (audit-fix F-01..F-04): re-export the cubecl runtime
// marker types behind their feature flags so downstream crates (notably the
// `validation` tier-3 driver) can name `Batch::<R>::eval_vec_host_*` without
// taking a direct dependency on the individual `cubecl-{hip,cuda,wgpu}`
// crates — they reach the runtime types through `xcfun-gpu`'s feature
// forwarding instead.
#[cfg(feature = "cuda")]
pub use cubecl_cuda::CudaRuntime;
#[cfg(feature = "hip")]
pub use cubecl_hip::HipRuntime;
#[cfg(feature = "wgpu")]
pub use cubecl_wgpu::WgpuRuntime;
