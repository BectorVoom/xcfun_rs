---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 02a
type: execute
wave: 3
depends_on:
  - 06-01
files_modified:
  - Cargo.toml
  - crates/xcfun-gpu/Cargo.toml
  - crates/xcfun-gpu/src/lib.rs
  - crates/xcfun-gpu/src/backend.rs
  - crates/xcfun-gpu/src/auto_backend.rs
  - crates/xcfun-gpu/src/batch.rs
  - crates/xcfun-gpu/src/pool.rs
  - crates/xcfun-gpu/src/error_routing.rs
  - crates/xcfun-gpu/src/runtime/cpu.rs
  - crates/xcfun-gpu/src/runtime/mod.rs
  - crates/xcfun-gpu/tests/batch_api_shape.rs
  - crates/xcfun-gpu/tests/batch_kernel_smoke.rs
  - crates/xcfun-gpu/tests/auto_backend_priority.rs
  - crates/xcfun-gpu/tests/buffer_pool_growth.rs
  - crates/xcfun-gpu/tests/wgpu_no_f64.rs
  - crates/xcfun-gpu/tests/settings_generation_bumps.rs
  - crates/xcfun-core/src/error.rs
  - crates/xcfun-core/tests/xcerror_copy_invariant.rs
  - crates/xcfun-eval/src/functional.rs
autonomous: true
requirements:
  - GPU-01
  - GPU-02
  - GPU-04
  - GPU-06
  - KER-04
must_haves:
  truths:
    - "Backend enum (Cpu, Rocm, Cuda, Metal, Wgpu) defined in xcfun-gpu::backend per D-07."
    - "auto_backend() skeleton present with priority chain XCFUN_FORCE_BACKEND > Rocm > Cuda > Metal-with-f64 > Wgpu-with-f64 > Cpu (cubecl-hip/cuda/wgpu probes return false until Plans 06-03/06-04 add the deps)."
    - "Batch<'fun, R: cubecl::Runtime> exposes reserve / upload_density / launch / download_result / eval_vec_host (GPU-01)."
    - "Batch's `fun` field is bound to `&'fun xcfun_eval::Functional` (W-3 from revision-1) — NOT `&'fun xcfun_rs::Functional` — to avoid circular dep (xcfun-rs → xcfun-gpu) at all downstream consumers (06-05 / 06-06)."
    - "Buffer pool grows powers-of-two; weights_buf (82 f64) + active_ids_buf (78 u32) fixed-size allocated once; generation counter monotonic u64 (GPU-04 / D-15)."
    - "XcError gains TWO typed variants: WgpuNoF64 { adapter_name: &'static str, requested_runtime: BackendTag } (D-13/D-13-A) AND CudaNoF64 { adapter_name: &'static str, requested_runtime: BackendTag } (W-7 from revision-1; symmetric typed error for the CUDA f64 probe wired in 06-04). Both preserve Phase 2 D-25 Copy + non_exhaustive."
    - "BackendTag shadow enum lives in xcfun-core (avoids xcfun-core ↔ xcfun-gpu layering inversion); xcfun-gpu::Backend ↔ BackendTag From/Into."
    - "Plan 06-02b (sibling Wave-3 plan) wires the validation-harness CLI extension (--tier 3, --reference, --exclude-erf) and consumes the skeleton; KER-06 tier-3 CPU sweep is owned by Plan 06-05 per revision-1 B-4."
  artifacts:
    - path: "crates/xcfun-gpu/Cargo.toml"
      provides: "Workspace-member crate manifest with feature flags default=[\"cpu\"]"
      contains: "name = \"xcfun-gpu\""
    - path: "crates/xcfun-gpu/src/backend.rs"
      provides: "Backend enum (Copy + Clone + Debug + PartialEq + Eq + Hash) — 5 variants Cpu/Rocm/Cuda/Metal/Wgpu"
      contains: "pub enum Backend"
    - path: "crates/xcfun-gpu/src/auto_backend.rs"
      provides: "auto_backend() priority chain skeleton"
      contains: "pub fn auto_backend"
    - path: "crates/xcfun-gpu/src/batch.rs"
      provides: "Batch<'fun, R: cubecl::Runtime> bound to &'fun xcfun_eval::Functional"
      contains: "pub struct Batch"
    - path: "crates/xcfun-gpu/src/pool.rs"
      provides: "OnceLock<R::Client> per-runtime + powers-of-two buffer pool"
      contains: "OnceLock"
    - path: "crates/xcfun-core/src/error.rs"
      provides: "XcError::WgpuNoF64 + XcError::CudaNoF64 typed variants + BackendTag shadow"
      contains: "WgpuNoF64"
  key_links:
    - from: "crates/xcfun-gpu/src/batch.rs::Batch::launch"
      to: "crates/xcfun-eval::Functional::settings_generation"
      via: "generation counter check before re-uploading weights_buf"
      pattern: "settings_generation\\|cached_gen"
    - from: "crates/xcfun-gpu/src/error_routing.rs"
      to: "crates/xcfun-core::Dependency::ERF"
      via: "Wgpu/Metal + ERF-bearing functional → fall back to Cpu"
      pattern: "Dependency::ERF"
    - from: "crates/xcfun-core/src/error.rs::WgpuNoF64"
      to: "crates/xcfun-core/src/error.rs::BackendTag"
      via: "preserves Copy + non_exhaustive via &'static str + enum shadow"
      pattern: "BackendTag"
---

<objective>
Unstub `crates/xcfun-gpu/`. Promote from `workspace.exclude` to `workspace.members` with the full skeleton: `Backend` enum, `Batch<'fun, R: cubecl::Runtime>` with the 5-method API per GPU-01, generation-counter buffer pool per GPU-04 / D-15, `auto_backend()` priority-chain skeleton per D-07, ERF auto-fallback routing per GPU-05 (skeleton; wired at dispatch site in Plan 06-05), the typed `XcError::WgpuNoF64` variant per GPU-06 / D-13 / D-13-A, AND a typed `XcError::CudaNoF64` variant (W-7 from revision-1) symmetric to WgpuNoF64 for the CUDA f64 probe Plan 06-04 wires.

This plan is the **skeleton half** of the original Plan 06-02. The validation-harness CLI extension (`--tier 3` / `--reference` / `--exclude-erf`) is split out to sibling Plan 06-02b (also Wave 3, depends on this plan). Per revision-1 B-4, KER-06 (the tier-3 CPU 10k-grid 1e-13 driver) is OWNED by Plan 06-05 — not this plan, not 06-02b — so this plan no longer carries `KER-06` in `requirements:`.

Per RESEARCH.md §"Recommended Plan Tree": Plan 06-02 establishes the GPU-backend SCAFFOLDING; cubecl-hip wiring lands in Plan 06-03 (ROCm primary); cubecl-cuda + cubecl-wgpu wiring + SHADER_F64 probes land in Plan 06-04. This plan ships the `Cpu` arm only as a working backend (cubecl-cpu via `xcfun-eval` re-export per D-08); ROCm/CUDA/Wgpu probes return `false` and `auto_backend()` falls through to `Cpu`.

Additional scope (carried over from original Plan 06-02 except the validation-harness extension):
- Add `XcError::WgpuNoF64 { adapter_name: &'static str, requested_runtime: BackendTag }` to xcfun-core::error per D-13/D-13-A.
- **(W-7 revision-1) Add `XcError::CudaNoF64 { adapter_name: &'static str, requested_runtime: BackendTag }`** symmetric to WgpuNoF64. Plan 06-04 wires the actual CUDA f64 probe + payload construction; this plan declares the variant so 06-04 only adds runtime probe code, not enum shape.
- Add `Functional::settings_generation()` accessor + `settings_gen: u64` field to xcfun-eval::Functional (bumped on `set` per D-15; consumed by `Batch::launch` for weights_buf re-upload skip).

**(W-3 revision-1) Batch<'fun, R> binds to `&'fun xcfun_eval::Functional`** — NOT `&'fun xcfun_rs::Functional`. Reason: in Plan 06-05 `xcfun-rs::Functional::eval_vec` calls `Batch::<R>::eval_vec_host(&self.0, ...)`, where `self.0: xcfun_eval::Functional`. If Batch held `&xcfun_rs::Functional`, then `xcfun-gpu` would need to depend on `xcfun-rs` — but `xcfun-rs` already depends on `xcfun-gpu` (cycle). The original Plan 06-02 deferred this lifetime fix to Plan 06-05; revision-1 W-3 moves it here so 06-05 consumes a corrected shape rather than retro-correcting it.

Note: This plan introduces NO `cubecl-hip`, `cubecl-cuda`, or `cubecl-wgpu` deps. The `Backend::Rocm`, `Backend::Cuda`, `Backend::Metal`, `Backend::Wgpu` variants are declared (Discretion: keep all 5 variants now to avoid enum-shape churn in 06-03/06-04). Their `*_available()` probes return `false`. Plans 06-03 / 06-04 add the deps + flip the probes.

Purpose: Establish the GPU dispatch skeleton that Plans 06-03 / 06-04 / 06-05 / 06-06 build atop. By landing `Batch<R>` generic + buffer pool + WgpuNoF64/CudaNoF64 typed errors + correct lifetime FIRST, downstream plans only need to wire individual cubecl runtime probes — they don't need to reshape the `Batch` API or re-do the lifetime.

Output: `xcfun-gpu` crate working in the `cpu` feature (default); 6 new test files (batch_api_shape, batch_kernel_smoke, auto_backend_priority, buffer_pool_growth, wgpu_no_f64-#[ignore]'d, settings_generation_bumps); xcerror_copy_invariant test; XcError extended with WgpuNoF64 + CudaNoF64 + BackendTag shadow; Functional gains `settings_gen` + `settings_generation()`.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@Cargo.toml
@crates/xcfun-core/src/error.rs
@crates/xcfun-eval/src/functional.rs
@crates/xcfun-eval/src/for_tests.rs
@crates/xcfun-rs/tests/send_sync.rs

<interfaces>
<!-- Existing types/exports the executor needs. -->

From crates/xcfun-core/src/error.rs (current — variants to MIRROR for two new variants):
```rust
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[non_exhaustive]
pub enum XcError {
    #[error("invalid order {order}: must be 0..={max_order}")]
    InvalidOrder { order: u32, max_order: u32 },
    // ... InvalidVars, InvalidMode, InvalidVarsAndMode, UnknownName, InputLengthMismatch,
    //     OutputLengthMismatch, NotConfigured, InvalidEncoding, Runtime ...
}
```

From RESEARCH.md §"Pattern 4" (Backend enum + auto_backend skeleton):
```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Backend { Cpu, Rocm, Cuda, Metal, Wgpu }
```

From RESEARCH.md §"Pattern 3" (Batch + buffer pool) — W-3 lifetime:
```rust
pub struct Batch<'fun, R: cubecl::Runtime> {
    fun: &'fun xcfun_eval::Functional,   // W-3: NOT xcfun_rs::Functional (cycle)
    client: cubecl::ComputeClient<R::Server, R::Channel>,
    weights_buf:    R::Handle,    // fixed 82 f64
    active_ids_buf: R::Handle,    // fixed 78 u32
    density_buf:    R::Handle,    // grows powers-of-two
    result_buf:     R::Handle,    // grows powers-of-two
    capacity:       usize,
    cached_gen:     u64,
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Promote xcfun-gpu to members + scaffold Backend / Batch / pool / error_routing + add XcError::WgpuNoF64 + XcError::CudaNoF64 + BackendTag + Functional::settings_generation</name>
  <files>Cargo.toml, crates/xcfun-gpu/Cargo.toml, crates/xcfun-gpu/src/lib.rs, crates/xcfun-gpu/src/backend.rs, crates/xcfun-gpu/src/auto_backend.rs, crates/xcfun-gpu/src/batch.rs, crates/xcfun-gpu/src/pool.rs, crates/xcfun-gpu/src/error_routing.rs, crates/xcfun-gpu/src/runtime/cpu.rs, crates/xcfun-gpu/src/runtime/mod.rs, crates/xcfun-gpu/tests/batch_api_shape.rs, crates/xcfun-gpu/tests/batch_kernel_smoke.rs, crates/xcfun-gpu/tests/auto_backend_priority.rs, crates/xcfun-gpu/tests/buffer_pool_growth.rs, crates/xcfun-gpu/tests/wgpu_no_f64.rs, crates/xcfun-gpu/tests/settings_generation_bumps.rs, crates/xcfun-core/src/error.rs, crates/xcfun-core/tests/xcerror_copy_invariant.rs, crates/xcfun-eval/src/functional.rs</files>
  <read_first>
    - Cargo.toml (workspace — verify exclude block; promote xcfun-gpu)
    - crates/xcfun-core/src/error.rs (full file — see InvalidVarsAndMode 3-field shape; current Copy+non_exhaustive contract)
    - crates/xcfun-eval/src/functional.rs (struct definition + `set()` body — add settings_gen field)
    - crates/xcfun-eval/src/for_tests.rs (OnceLock<CpuClient> pattern to generalise)
    - crates/xcfun-rs/tests/send_sync.rs (assert_impl_all! pattern)
    - crates/xcfun-eval/tests/cubecl_spike.rs (analog test for batch_kernel_smoke)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md §"Plan 06-02"
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pattern 3 + Pattern 4"
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md D-12 / D-13 / D-13-A / D-15
    - **(W-4 / W-10 revision-1)** Context7 query directive — fetch `/tracel-ai/cubecl` with the prompt `"Feature::Type Elem::Float FloatKind::F64 0.10.0-pre.3 API path"` to confirm the exact API path for the f64 probe BEFORE writing any `feature_enabled(...)` site. The probe argument MUST be `cubecl::Feature::Type(cubecl::ir::Elem::Float(cubecl::ir::FloatKind::F64))` (verified against cubecl 0.10.0-pre.3) — also accessible as `https://github.com/tracel-ai/cubecl/blob/main/cubecl-book/src/core-features/features.md` if Context7 is unavailable. Do NOT use a `feature_enabled(...)` placeholder; the executor must use the explicit `Feature::Type(Elem::Float(FloatKind::F64))` argument.
  </read_first>
  <behavior>
    - Test 1 (RED first): `tests/batch_api_shape.rs` uses `static_assertions::assert_impl_all!` to assert `Batch<'_, CpuRuntime>: Send` (per `cubecl::Runtime: 'static + Send + Sync`); asserts `Batch::<CpuRuntime>::reserve`, `upload_density`, `launch`, `download_result`, `eval_vec_host` exist with the GPU-01 signatures.
    - Test 2: `tests/auto_backend_priority.rs` — sets `XCFUN_FORCE_BACKEND=cpu`; asserts `auto_backend() == Backend::Cpu`. Sets `XCFUN_FORCE_BACKEND=rocm`; asserts equal to `Backend::Rocm`. Sets unrecognised value and asserts panic via `should_panic`.
    - Test 3: `tests/buffer_pool_growth.rs` — instantiates `Batch::<CpuRuntime>::open(&fun)`; calls `reserve(10)` then `reserve(50)` then `reserve(200)`; asserts `capacity == 64` after first, `capacity == 64` after second (still fits), `capacity == 256` after third (powers-of-two doubling per D-15).
    - Test 4: `tests/batch_kernel_smoke.rs` — sets up XC_SLATERX functional, runs `Batch::<CpuRuntime>::eval_vec_host` on 100 random density points; asserts each output matches scalar `Functional::eval` within 1e-13.
    - Test 5: `tests/wgpu_no_f64.rs` — `#[ignore]`d in this plan because the wgpu feature isn't compiled in until 06-04; placeholder `#[cfg(feature = "wgpu")]` test stub asserting `XcError::WgpuNoF64` is constructible.
    - Test 6: `crates/xcfun-core/tests/xcerror_copy_invariant.rs` — `assert_impl_all!(XcError: Copy);`. Compile-time gate: TWO new variants must preserve Copy.
    - Test 7: `tests/settings_generation_bumps.rs` — assert `Functional::settings_generation()` increments after `set()`.
    - All tests MUST FAIL before scaffolding lands (RED) and PASS after (GREEN), except wgpu_no_f64.rs which is `#[ignore]`d in this plan.
  </behavior>
  <action>
**Step A — Promote xcfun-gpu to workspace members (Cargo.toml):**

```toml
[workspace]
members = [
    "crates/xcfun-ad",
    "crates/xcfun-core",
    "crates/xcfun-kernels",
    "crates/xcfun-eval",
    "crates/xcfun-gpu",        # NEW Plan 06-02a (promoted from exclude)
    "crates/xcfun-rs",
    "crates/xcfun-capi",
    "xtask",
    "validation",
]
exclude = [
    "crates/xcfun-python",     # Phase 7
]
```

Add to `[workspace.dependencies]`:
```toml
cubecl-hip  = "=0.10.0-pre.3"
cubecl-cuda = "=0.10.0-pre.3"
cubecl-wgpu = "=0.10.0-pre.3"
```

**Step B — Create `crates/xcfun-gpu/Cargo.toml`:**

```toml
[package]
name = "xcfun-gpu"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "GPU batch lifecycle + auto_backend dispatch for xcfun_rs (cubecl-hip / cubecl-cuda / cubecl-wgpu opt-in via feature flags)"
license = "MPL-2.0"

[features]
default = ["cpu"]
cpu  = ["dep:cubecl-cpu"]
hip  = ["dep:cubecl-hip"]
cuda = ["dep:cubecl-cuda"]
wgpu = ["dep:cubecl-wgpu"]
metal = ["wgpu"]                  # alias of wgpu (RESEARCH §R-02 / Pitfall 9)

[dependencies]
xcfun-kernels = { path = "../xcfun-kernels" }
xcfun-core    = { path = "../xcfun-core" }
xcfun-ad      = { path = "../xcfun-ad" }
xcfun-eval    = { path = "../xcfun-eval" }
cubecl        = { workspace = true }
cubecl-cpu    = { workspace = true, optional = true }
cubecl-hip    = { workspace = true, optional = true }
cubecl-cuda   = { workspace = true, optional = true }
cubecl-wgpu   = { workspace = true, optional = true }
thiserror     = { workspace = true }

[dev-dependencies]
static_assertions = { workspace = true }
approx            = { workspace = true }
```

**Step C — Create `crates/xcfun-gpu/src/lib.rs`:**

```rust
//! # xcfun-gpu
//!
//! GPU batch lifecycle + auto_backend dispatch for xcfun_rs.
//!
//! - `Backend` (5 variants per D-07): Cpu, Rocm, Cuda, Metal, Wgpu.
//! - `Batch<'fun, R: cubecl::Runtime>` (GPU-01) — bound to `&'fun xcfun_eval::Functional`
//!   per W-3 revision-1 (avoids xcfun-rs ↔ xcfun-gpu cycle).
//! - `auto_backend()` priority chain: XCFUN_FORCE_BACKEND > Rocm > Cuda > Metal-w/-f64 > Wgpu-w/-f64 > Cpu.
//! - Generation-counter buffer pool (GPU-04 / D-15).
//! - ERF auto-fallback to Cpu on Wgpu/Metal (GPU-05).
//! - Compile-time f64 invariant.

#![deny(unsafe_op_in_unsafe_fn)]

const _: () = assert!(core::mem::size_of::<f64>() == 8);

pub mod backend;
pub mod auto_backend;
pub mod batch;
pub mod pool;
pub mod error_routing;
pub mod runtime;

pub use backend::Backend;
pub use auto_backend::auto_backend;
pub use batch::Batch;
```

**Step D — Create `crates/xcfun-gpu/src/backend.rs` (per RESEARCH Pattern 4):**

```rust
//! Backend enum — runtime discriminator. Per Phase 6 D-07 priority order.
//!
//! Note: cubecl-metal does not exist as a separate crate (RESEARCH Pitfall 9 / R-02).
//! The `Metal` variant is reached via `cubecl-wgpu`'s Metal backend.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Backend {
    Cpu, Rocm, Cuda, Metal, Wgpu,
}

impl Backend {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "cpu" => Some(Backend::Cpu),
            "rocm" | "hip" => Some(Backend::Rocm),
            "cuda" => Some(Backend::Cuda),
            "metal" => Some(Backend::Metal),
            "wgpu" => Some(Backend::Wgpu),
            _ => None,
        }
    }
}

// Bidirectional From/Into with the BackendTag shadow in xcfun-core (avoids layering inversion).
impl From<Backend> for xcfun_core::BackendTag {
    fn from(b: Backend) -> Self {
        match b {
            Backend::Cpu => xcfun_core::BackendTag::Cpu,
            Backend::Rocm => xcfun_core::BackendTag::Rocm,
            Backend::Cuda => xcfun_core::BackendTag::Cuda,
            Backend::Metal => xcfun_core::BackendTag::Metal,
            Backend::Wgpu => xcfun_core::BackendTag::Wgpu,
        }
    }
}
```

**Step E — Create `crates/xcfun-gpu/src/auto_backend.rs`:**

```rust
//! auto_backend() — D-07 priority chain. See module-level doc-comment for rationale.
use crate::Backend;

pub fn auto_backend() -> Backend {
    if let Ok(force) = std::env::var("XCFUN_FORCE_BACKEND") {
        return Backend::from_str(&force)
            .unwrap_or_else(|| panic!("XCFUN_FORCE_BACKEND={} unrecognised (expected: cpu|rocm|cuda|metal|wgpu)", force));
    }
    #[cfg(feature = "hip")]
    if crate::runtime::hip::rocm_available() { return Backend::Rocm; }
    #[cfg(feature = "cuda")]
    if crate::runtime::cuda::cuda_available() { return Backend::Cuda; }
    #[cfg(feature = "wgpu")]
    if crate::runtime::wgpu::metal_with_f64_available() { return Backend::Metal; }
    #[cfg(feature = "wgpu")]
    if crate::runtime::wgpu::wgpu_with_shader_f64_available() { return Backend::Wgpu; }
    Backend::Cpu
}
```

**Step F — Create `crates/xcfun-gpu/src/runtime/cpu.rs` + `runtime/mod.rs` (always-on substrate + stubs for hip/cuda/wgpu):**

```rust
// runtime/mod.rs
pub mod cpu;
#[cfg(feature = "hip")]   pub mod hip;
#[cfg(feature = "cuda")]  pub mod cuda;
#[cfg(feature = "wgpu")]  pub mod wgpu;
```

```rust
// runtime/cpu.rs — always-on substrate.
#[cfg(feature = "cpu")]
pub use xcfun_eval::for_tests::cpu_client;

#[cfg(feature = "cpu")]
pub fn cpu_available() -> bool { true }
```

For features `hip` / `cuda` / `wgpu`, create stub files inside `runtime/` returning `false` for their probes so `cargo build --features hip` etc don't immediately break — Plans 06-03 / 06-04 fill them with real implementations.

**Step G — Create `crates/xcfun-gpu/src/pool.rs` (powers-of-two buffer pool with generation counter):**

```rust
//! Per-runtime client cache (OnceLock) + buffer-handle allocation helper.
//! Powers-of-two doubling lives inside Batch::ensure_capacity.

use std::sync::OnceLock;

#[cfg(feature = "cpu")]
pub use xcfun_eval::for_tests::{cpu_client, CpuClient};

pub(crate) struct BatchBuffers<R: cubecl::Runtime> {
    pub weights_buf:    R::Handle,
    pub active_ids_buf: R::Handle,
    pub density_buf:    R::Handle,
    pub result_buf:     R::Handle,
    pub capacity:       usize,
}
```

**Step H — Create `crates/xcfun-gpu/src/batch.rs` (Batch<R> with the 5-method API; W-3 LIFETIME FIX):**

```rust
//! Batch<'fun, R: cubecl::Runtime> — RS-08 batch dispatch lifecycle.

use cubecl::prelude::*;
use xcfun_core::XcError;
use crate::Backend;
use crate::pool::BatchBuffers;

// W-3 (revision-1): Batch holds &'fun xcfun_eval::Functional, NOT &'fun xcfun_rs::Functional.
// xcfun-rs depends on xcfun-gpu (one-way); xcfun-gpu must NOT import xcfun-rs (would cycle).
// Plan 06-05 calls `Batch::<R>::eval_vec_host(&self.0, ...)` where self.0: xcfun_eval::Functional.
pub struct Batch<'fun, R: cubecl::Runtime> {
    fun: &'fun xcfun_eval::Functional,
    client: cubecl::ComputeClient<R::Server, R::Channel>,
    bufs: BatchBuffers<R>,
    cached_gen: u64,
}

impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
    /// GPU-01 + GPU-06: Open a Batch. Returns XcError::WgpuNoF64 on Wgpu without
    /// SHADER_F64 (Plan 06-04 fills the probe; this plan declares the API shape).
    pub fn open(fun: &'fun xcfun_eval::Functional, client: cubecl::ComputeClient<R::Server, R::Channel>)
        -> Result<Self, XcError>
    {
        // ... allocate fixed weights_buf (82 f64), active_ids_buf (78 u32) ...
        // ... initial capacity 64 → density_buf + result_buf ...
        // ... return Self with cached_gen = 0 (forces first launch to upload weights) ...
        todo!()
    }

    /// GPU-01: reserve capacity for `nr_points`; powers-of-two grow per D-15.
    pub fn reserve(&mut self, nr_points: usize) {
        if nr_points > self.bufs.capacity {
            let mut new_cap = self.bufs.capacity.max(64);
            while new_cap < nr_points { new_cap *= 2; }
            let inlen = self.fun.input_length();
            let outlen = self.fun.output_length().unwrap();
            self.bufs.density_buf = self.client.empty(new_cap * inlen  * 8);
            self.bufs.result_buf  = self.client.empty(new_cap * outlen * 8);
            self.bufs.capacity = new_cap;
        }
    }

    pub fn upload_density(&mut self, density: &[f64], density_pitch: usize, nr_points: usize) { todo!() }

    /// GPU-01: launch the kernel. D-15: re-upload weights only when stale.
    pub fn launch(&mut self, nr_points: u32) -> Result<(), XcError> {
        if self.fun.settings_generation() != self.cached_gen {
            // ... self.client.write(&self.bufs.weights_buf, ...) ...
            self.cached_gen = self.fun.settings_generation();
        }
        Ok(())
    }

    pub fn download_result(&self, out: &mut [f64], out_pitch: usize, nr_points: usize) { todo!() }

    /// GPU-01: end-to-end host call. CONCRETE for CpuRuntime in this plan;
    /// Plans 06-03 / 06-04 wire HIP / CUDA / Wgpu variants atop Self::open + reserve + upload + launch + download.
    pub fn eval_vec_host(
        fun: &'fun xcfun_eval::Functional,
        density: &[f64], density_pitch: usize,
        out: &mut [f64], out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> { todo!() }
}
```

**However, the `eval_vec_host<CpuRuntime>` body MUST be concrete in this plan** so Plan 06-05 (RS-08 dispatch) can call it AND so Plan 06-05 (which now owns KER-06 per B-4) has a working CPU path to test. Implement it in full in `batch.rs` for the CPU arm; leave generic `R != CpuRuntime` body as `todo!("Plan 06-03/06-04 wires GPU runtime variants")`.

**Step I — Create `crates/xcfun-gpu/src/error_routing.rs`:**

```rust
//! ERF auto-fallback routing. Per GPU-05 + RESEARCH "Verified pattern: comptime branching".

use xcfun_core::Dependency;
use crate::Backend;

pub fn must_fall_back_to_cpu(deps: Dependency, backend: Backend) -> bool {
    deps.contains(Dependency::ERF) && matches!(backend, Backend::Wgpu | Backend::Metal)
}
```

**Step J — Add TWO typed variants + BackendTag shadow to `crates/xcfun-core/src/error.rs`:**

```rust
/// Subset of xcfun-gpu::Backend used by XcError typed runtime errors to avoid
/// layering inversion. Kept in sync with xcfun-gpu::Backend.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BackendTag { Cpu, Rocm, Cuda, Metal, Wgpu }

// Add to XcError enum:

/// Phase 6 D-13/D-13-A — Wgpu device lacks SHADER_F64.
/// adapter_name is &'static str (NOT String) to preserve Phase 2 D-25 Copy + non_exhaustive.
#[error("Wgpu adapter '{adapter_name}' lacks SHADER_F64; cannot launch {requested_runtime:?} (D-13/D-13-A)")]
WgpuNoF64 {
    adapter_name: &'static str,
    requested_runtime: BackendTag,
},

/// (W-7 revision-1) Symmetric typed error for the CUDA f64 probe (Plan 06-04 wires the probe).
/// CUDA f64 is reportedly always supported on real CUDA devices, but the cubecl-book feature
/// matrix flags CUDA f64 as "?" — defensive typed error for devices that fail the probe at
/// runtime. Same Copy + non_exhaustive contract as WgpuNoF64.
#[error("CUDA adapter '{adapter_name}' lacks f64 device support; cannot launch {requested_runtime:?} (W-7)")]
CudaNoF64 {
    adapter_name: &'static str,
    requested_runtime: BackendTag,
},
```

Update `XcError::as_c_code` to map `WgpuNoF64 { .. }` and `CudaNoF64 { .. }` → `-1` (no upstream `XC_E*` mapping; matches the `UnknownName` precedent).

**Step K — Add `Functional::settings_generation` to `crates/xcfun-eval/src/functional.rs`:**

Add field `settings_gen: u64` (default 0). In `set()` body, append `self.settings_gen = self.settings_gen.wrapping_add(1);`. Add public accessor `pub fn settings_generation(&self) -> u64 { self.settings_gen }`.

**Step L — Create the 6 test files** per behaviors. Each test mirrors the patterns in RESEARCH §"Code Examples". `wgpu_no_f64.rs` is `#[ignore]`'d in this plan and Plan 06-04 un-ignores it.

```rust
// crates/xcfun-core/tests/xcerror_copy_invariant.rs
use static_assertions::assert_impl_all;
use xcfun_core::XcError;
assert_impl_all!(XcError: Copy, Send, Sync);
```

Run `cargo build --workspace` and `cargo nextest run -p xcfun-gpu --tests` — all expected to GREEN (some tests `#[ignore]`d).
  </action>
  <verify>
    <automated>cargo build --workspace && cargo nextest run -p xcfun-gpu --tests && cargo nextest run -p xcfun-core --test xcerror_copy_invariant</automated>
  </verify>
  <acceptance_criteria>
    - `crates/xcfun-gpu/Cargo.toml` exists with `default = ["cpu"]` and feature flags `hip`, `cuda`, `wgpu`, `metal`.
    - `grep -c '"crates/xcfun-gpu"' Cargo.toml` >= 1 in `members` block.
    - `grep -E '"crates/xcfun-gpu"' Cargo.toml | grep exclude | wc -l` == 0 (no longer in exclude).
    - `grep -c "cubecl-hip\s*=\s*\"=0.10.0-pre.3\"" Cargo.toml` >= 1
    - `grep -c "cubecl-cuda\s*=\s*\"=0.10.0-pre.3\"" Cargo.toml` >= 1
    - `grep -c "cubecl-wgpu\s*=\s*\"=0.10.0-pre.3\"" Cargo.toml` >= 1
    - `grep -c "pub enum Backend" crates/xcfun-gpu/src/backend.rs` >= 1
    - `grep -c "pub fn auto_backend" crates/xcfun-gpu/src/auto_backend.rs` >= 1
    - `grep -c "pub struct Batch" crates/xcfun-gpu/src/batch.rs` >= 1
    - `grep -c "fun: &'fun xcfun_eval::Functional" crates/xcfun-gpu/src/batch.rs` >= 1   # W-3 invariant
    - `grep -c "xcfun_rs::Functional" crates/xcfun-gpu/src/batch.rs` == 0                # W-3 invariant: NEVER xcfun_rs there
    - `grep -c "WgpuNoF64" crates/xcfun-core/src/error.rs` >= 1
    - `grep -c "CudaNoF64" crates/xcfun-core/src/error.rs` >= 1                          # W-7 invariant
    - `grep -c "BackendTag" crates/xcfun-core/src/error.rs` >= 1                         # shadow enum
    - `grep -c "&'static str" crates/xcfun-core/src/error.rs` >= 1                       # D-13-A invariant
    - `grep -c "settings_gen\|settings_generation" crates/xcfun-eval/src/functional.rs` >= 2
    - `cargo build --workspace` exits 0.
    - `cargo nextest run -p xcfun-gpu --tests` exits 0 (with #[ignore] tests skipped).
    - `cargo nextest run -p xcfun-core --test xcerror_copy_invariant` exits 0 (TWO new variants preserve Copy).
    - `cargo nextest run -p xcfun-gpu --test auto_backend_priority` exits 0.
    - `cargo nextest run -p xcfun-gpu --test buffer_pool_growth` exits 0.
    - `cargo nextest run -p xcfun-gpu --test settings_generation_bumps` exits 0.
  </acceptance_criteria>
  <done>xcfun-gpu crate scaffolded as workspace member; Backend enum + auto_backend() + Batch<'fun &xcfun_eval::Functional, R> + buffer pool + error_routing + WgpuNoF64 + CudaNoF64 typed errors + BackendTag shadow all landed; Functional gains settings_gen + accessor; 6 new tests GREEN; xcerror_copy_invariant verifies BOTH new variants preserve Copy.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| xcfun-gpu ↔ cubecl runtime crates | Feature-gated; Plan 06-02a ships only `cpu` feature default; runtime probes for hip/cuda/wgpu return `false` |
| xcfun-core ↔ xcfun-gpu (Backend reference) | Avoided via `BackendTag` shadow in xcfun-core (preserves layering) |
| xcfun-gpu ↔ xcfun-rs (Batch lifetime) | W-3: Batch holds `&xcfun_eval::Functional` (xcfun-gpu does NOT import xcfun-rs); xcfun-rs depends on xcfun-gpu (one-way) |
| Functional `&self` ↔ settings_gen update | settings_gen is bumped only via `&mut self` (set()); no race in this plan; Plan 06-06 D-12 hardens the eval path |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-OOM | medium | Buffer pool unbounded growth on `reserve(usize::MAX)` would OOM | Powers-of-two doubling; D-15 + Step H reserve() body computes `new_cap *= 2` until ≥ nr_points; document upper bound expected |
| T-06-WGPU-F32 | high | Silent wgpu f32 downgrade at runtime | D-13 typed `XcError::WgpuNoF64` declared in xcfun-core (Step J); compile-time `size_of::<f64>() == 8` assertion in xcfun-gpu/src/lib.rs (Step C) |
| T-06-CUDA-F64 | medium | (W-7 revision-1) Defensive: CUDA f64 reported as "?" in cubecl feature matrix | D-13-A symmetric typed `XcError::CudaNoF64` declared in xcfun-core (Step J); Plan 06-04 wires the probe |
| T-06-CUBECL-DRIFT | high | Adding cubecl-hip/cuda/wgpu workspace pins without lockstep enforcement would drift | Workspace pins use `=0.10.0-pre.3`; xtask check-cubecl-pin scope extends to 5 crates (Plans 06-03/06-04 add hip/cuda/wgpu probes; this plan extends the pin check) |
| T-06-LIFETIME-CYCLE | high | (W-3 revision-1) Batch holding `&xcfun_rs::Functional` would cycle xcfun-rs ↔ xcfun-gpu | Step H lifetime is `&'fun xcfun_eval::Functional`; acceptance criteria assert `xcfun_rs::Functional` does NOT appear in batch.rs |
| T-06-CIRC-DEP | medium | xcfun-core depending on xcfun-gpu (Backend) inverts layering | Step J introduces `BackendTag` shadow in xcfun-core; xcfun-gpu::Backend has From/Into BackendTag |
</threat_model>

<verification>
- All acceptance criteria GREEN per Task 1's automated command.
- xtask gates remain GREEN: `cargo run -p xtask --bin check-cubecl-pin && cargo run -p xtask --bin check-no-anyhow && cargo run -p xtask --bin check-no-mul-add` exits 0.
- Phase 5 `assert_impl_all!(Functional: Send, Sync)` continues to compile.
- xcfun-core::tests::xcerror_copy_invariant GREEN (BOTH new variants — WgpuNoF64 AND CudaNoF64 — preserve XcError Copy).
- W-3 invariant: `grep` confirms Batch lifetime is `&xcfun_eval::Functional`, NOT `&xcfun_rs::Functional`.
- W-7 invariant: `grep` confirms `XcError::CudaNoF64` exists.
- Plan 06-02b consumes the skeleton WITHOUT needing any Batch API reshape; Plans 06-03/06-04 same.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 3 advanced: `Backend` enum + `auto_backend()` + `Batch<R>` API skeleton present (per GPU-01/02).
- ROADMAP Phase 6 success criterion 5 advanced: `XcError::WgpuNoF64` AND `XcError::CudaNoF64` typed variants declared (GPU-06 / D-13/D-13-A; W-7); compile-time f64 invariant in xcfun-kernels + xcfun-gpu.
- W-3: Batch lifetime correctly bound BEFORE 06-05 consumes it (no retro-correction).
- Plans 06-02b / 06-03 / 06-04 / 06-05 / 06-06 unblocked.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02a-SUMMARY.md` documenting:
- xcfun-gpu promoted from exclude to members
- Backend enum + 5 variants (Cpu, Rocm, Cuda, Metal, Wgpu)
- Batch<'fun, R: cubecl::Runtime> with reserve/upload_density/launch/download_result/eval_vec_host AND `fun: &'fun xcfun_eval::Functional` (W-3)
- buffer pool generation counter (D-15) + powers-of-two doubling
- XcError::WgpuNoF64 + XcError::CudaNoF64 (W-7) + BackendTag shadow + xcerror_copy_invariant test GREEN
- Functional::settings_gen field + settings_generation() accessor (Phase 5 forward closed)
- Stub probes for cubecl-hip/cuda/wgpu (Plans 06-03/06-04 fill)
- Plan 06-02b consumes the skeleton; KER-06 tier-3 CPU sweep deferred to 06-05 (B-4 from revision-1)
</output>
</content>
</invoke>