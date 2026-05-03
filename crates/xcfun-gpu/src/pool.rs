//! Per-runtime client cache + buffer-handle bundle.
//!
//! Phase 6 Plan 06-02a CONTEXT D-15 invariants:
//!
//! - `OnceLock<R::Client>` per runtime (CPU substrate today; HIP / CUDA /
//!   Wgpu added in Plans 06-03 / 06-04). The CPU `OnceLock` is provided
//!   by `xcfun-eval::for_tests::cpu_client()` (re-exported via
//!   `runtime::cpu`), so this module only declares the buffer-bundle
//!   type today.
//! - `weights_buf` (82 f64 = 656 bytes) and `active_ids_buf` (78 u32 =
//!   312 bytes) are FIXED-SIZE — allocated once at `Batch::open`.
//! - `density_buf` and `result_buf` grow powers-of-two on overflow;
//!   never shrink. Doubling lives inside `Batch::reserve` /
//!   `ensure_capacity` (`batch.rs`).
//!
//! Hash-based weights cache invalidation was rejected per D-15 — hashing
//! 656 bytes per launch is 1-2% of GPU launch overhead at small batch
//! sizes; a monotonic `u64` counter on `Functional::settings_gen` is
//! O(1) instead. See `xcfun_eval::Functional::settings_generation`.

#[cfg(feature = "cpu")]
pub use crate::runtime::cpu::{cpu_client, CpuClient};

/// Buffer-handle bundle owned by a [`crate::Batch<R>`]. Generic over the
/// runtime so each backend gets its own monomorphised set of handles.
///
/// `cubecl::server::Handle` is the universal handle type across cubecl
/// 0.10-pre.3 runtimes (verified against
/// `crates/xcfun-eval/src/functional.rs:1614-1660` `launch_eval_point`
/// signature; same handle type used by every cubecl backend).
// `weights_buf` / `active_ids_buf` are read once Plans 06-03 / 06-04 wire
// concrete `Batch::launch` bodies that upload them on stale-generation
// detection. Plan 06-02a only declares the fields; suppress the dead-code
// lint until then.
#[allow(dead_code)]
pub(crate) struct BatchBuffers {
    /// Fixed 82 × f64 mirror of `Functional::settings`.
    pub weights_buf: cubecl::server::Handle,
    /// Fixed 78 × u32 active-functional id list.
    pub active_ids_buf: cubecl::server::Handle,
    /// Density input buffer; capacity = `capacity * input_len * 8` bytes.
    pub density_buf: cubecl::server::Handle,
    /// Result output buffer; capacity = `capacity * output_len * 8` bytes.
    pub result_buf: cubecl::server::Handle,
    /// Current allocated capacity in points. Starts at 64 (CONTEXT D-14
    /// `XCFUN_MIN_BATCH_SIZE` default); doubles on overflow per D-15.
    pub capacity: usize,
}
