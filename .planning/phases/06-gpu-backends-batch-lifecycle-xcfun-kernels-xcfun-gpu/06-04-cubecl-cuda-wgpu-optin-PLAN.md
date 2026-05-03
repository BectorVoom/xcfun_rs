---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 04
type: execute
wave: 6
depends_on:
  - 06-02a
  - 06-02b
files_modified:
  - crates/xcfun-gpu/Cargo.toml
  - crates/xcfun-gpu/src/runtime/cuda.rs
  - crates/xcfun-gpu/src/runtime/wgpu.rs
  - crates/xcfun-gpu/src/auto_backend.rs
  - crates/xcfun-gpu/src/batch.rs
  - crates/xcfun-gpu/tests/wgpu_no_f64.rs
  - crates/xcfun-gpu/tests/erf_fallback.rs
  - validation/Cargo.toml
  - validation/src/main.rs
  - validation/src/driver.rs
autonomous: true
requirements:
  - GPU-02
  - GPU-03
  - GPU-08
must_haves:
  truths:
    - "cubecl-cuda = =0.10.0-pre.3 wired as opt-in feature `cuda` of xcfun-gpu (per D-06)."
    - "cubecl-wgpu = =0.10.0-pre.3 wired as opt-in feature `wgpu` (covers Vulkan/Metal/DX12/WebGPU); `metal` feature is alias of `wgpu` per RESEARCH Pitfall 9."
    - "WgpuRuntime probe checks `client.properties().feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))` per RESEARCH §Standard Stack lines 712-727; refuses to launch on devices without SHADER_F64 (returns XcError::WgpuNoF64 at Batch::open per D-13)."
    - "CudaRuntime probe attempts CudaDevice::default(); ALSO probes Feature::Type(Elem::Float(FloatKind::F64)) per W-7 (revision-1) before caching the client; on f64-probe failure, Batch::open returns typed XcError::CudaNoF64 (D-13-A pattern symmetric to WgpuNoF64). Variant declared in 06-02a; cuda_no_f64_error helper in runtime/cuda.rs."
    - "(W-4/W-10 revision-1) f64 probe sites in BOTH runtime/cuda.rs AND runtime/wgpu.rs use the explicit cubecl::Feature::Type(cubecl::ir::Elem::Float(cubecl::ir::FloatKind::F64)) argument — NEVER a feature_enabled(...) placeholder."
    - "Compile gate: `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` succeeds (GPU-03 multi-feature compile)."
    - "validation harness gains --backend cuda + --backend wgpu dispatch arms; tier-3 Wgpu 10k-grid 1e-9 driver path complete (GPU-08; --exclude-erf flag from Plan 06-02)."
    - "ERF auto-fallback at Batch::open level — Wgpu/Metal route + Dependency::ERF mask → returns XcError::Runtime hint to caller (Plan 06-05 wires the auto-route at the dispatch site)."
  artifacts:
    - path: "crates/xcfun-gpu/src/runtime/cuda.rs"
      provides: "CudaRuntime client OnceLock + cuda_available() probe"
      contains: "cuda_available"
    - path: "crates/xcfun-gpu/src/runtime/wgpu.rs"
      provides: "WgpuRuntime client OnceLock + wgpu_with_shader_f64_available() + metal_with_f64_available() probes"
      contains: "SHADER_F64\\|wgpu_with_shader_f64_available"
    - path: "crates/xcfun-gpu/Cargo.toml"
      provides: "cuda = [\"dep:cubecl-cuda\"], wgpu = [\"dep:cubecl-wgpu\"], metal = [\"wgpu\"] feature flags"
      contains: "cubecl-cuda"
    - path: "crates/xcfun-gpu/tests/wgpu_no_f64.rs"
      provides: "wgpu f64 device-feature probe test (un-#[ignore]'d)"
      contains: "WgpuNoF64"
    - path: "crates/xcfun-gpu/tests/erf_fallback.rs"
      provides: "ERF-bearing functional + Wgpu backend → routes to Cpu test"
      contains: "Dependency::ERF"
  key_links:
    - from: "crates/xcfun-gpu/src/runtime/wgpu.rs::wgpu_with_shader_f64_available"
      to: "cubecl::client::properties::feature_enabled"
      via: "Feature::Type(Elem::Float(FloatKind::F64))"
      pattern: "FloatKind::F64\\|SHADER_F64"
    - from: "crates/xcfun-gpu/src/batch.rs::Batch::open (Wgpu arm)"
      to: "crates/xcfun-core::XcError::WgpuNoF64"
      via: "f64 probe failure → Box::leak(adapter_name) → XcError::WgpuNoF64"
      pattern: "WgpuNoF64"
---

<objective>
Wire **cubecl-cuda + cubecl-wgpu as opt-in feature flags** (per D-06; community-maintained best-effort). Per RESEARCH Pitfall 9 + R-02: `cubecl-metal` does NOT exist as a separate crate — Metal is reached via `cubecl-wgpu`'s Metal backend; the `metal` feature in xcfun-gpu is an alias of `wgpu`.

Three concrete deliverables:
1. **CUDA wiring** — `cubecl-cuda = "=0.10.0-pre.3"` opt-in feature `cuda`; `runtime/cuda.rs` with `OnceLock<CudaClient>` + `cuda_available()` probe; `auto_backend()` Cuda arm; `Batch<CudaRuntime>::eval_vec_host` body. Tier-3 strict-1e-13 GREEN is BEST-EFFORT (no NVIDIA hardware in dev env per RESEARCH §"Environment Availability"); cloud-CI runner expected.
2. **Wgpu wiring** — `cubecl-wgpu = "=0.10.0-pre.3"` opt-in feature `wgpu`; `runtime/wgpu.rs` with `OnceLock<WgpuClient>` + the two SHADER_F64 probes (`wgpu_with_shader_f64_available()` for Vulkan/DX12 path; `metal_with_f64_available()` for Metal path on macOS); `auto_backend()` Metal/Wgpu arms; `Batch<WgpuRuntime>::eval_vec_host` body; **typed `XcError::WgpuNoF64` at `Batch::open` when device lacks SHADER_F64** (D-13/D-13-A; preserves Phase 2 D-25 Copy via `Box::leak`-once `&'static str` adapter_name).
3. **Tier-3 Wgpu 10k-grid 1e-9 driver path** (GPU-08 per ROADMAP success criterion 4) — `validation/src/driver.rs` Wgpu arm in `run_tier3` matches CPU within 1e-9 rel-err on the 10k stratified grid AFTER `--exclude-erf` filters out range-separated functionals (which auto-fall-back to CPU at runtime per GPU-05).

Compile gate per GPU-03: `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` succeeds (multi-feature compile; covers all four cubecl runtimes lockstep).

ERF fallback at Batch::open level (per RESEARCH §"Verified pattern: comptime branching for ERF-bearing kernels"):
```rust
impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
    pub fn open(fun: &'fun Functional, runtime: Backend) -> Result<Self, XcError> {
        if matches!(runtime, Backend::Wgpu | Backend::Metal)
            && fun.dependencies().contains(Dependency::ERF) {
            return Self::open_cpu(fun);  // override caller's choice
        }
        // ...
    }
}
```
Plan 06-05 wires the actual `auto_backend()` → `Batch::<R>::eval_vec_host` dispatch at `Functional::eval_vec`; this plan plumbs the open-time fallback machinery in `error_routing.rs`.

Purpose: Complete the GPU runtime matrix per Phase 6 D-06 / GPU-02 / GPU-03 / GPU-08 — ROCm primary (Plan 06-03) + CUDA + Wgpu opt-in (this plan). Plan 06-05 wires `Functional::eval_vec` GPU dispatch atop.

Output: cubecl-cuda + cubecl-wgpu deps; CudaRuntime + WgpuRuntime probes (with SHADER_F64 gate); Batch<CudaRuntime> + Batch<WgpuRuntime> bodies; wgpu_no_f64 test un-`#[ignore]`'d; erf_fallback test; validation `--backend cuda` + `--backend wgpu` flags; tier-3 Wgpu 1e-9 driver path.
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
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02a-xcfun-gpu-skeleton-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-03-cubecl-hip-rocm-primary-PLAN.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@crates/xcfun-gpu/Cargo.toml
@crates/xcfun-gpu/src/auto_backend.rs
@crates/xcfun-gpu/src/batch.rs
@crates/xcfun-gpu/src/error_routing.rs
@crates/xcfun-gpu/src/runtime/hip.rs
@crates/xcfun-gpu/tests/wgpu_no_f64.rs
@validation/src/main.rs
@validation/src/driver.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: cubecl-cuda + cubecl-wgpu opt-in features + runtime probes + Batch<R> bodies</name>
  <files>crates/xcfun-gpu/Cargo.toml, crates/xcfun-gpu/src/runtime/cuda.rs, crates/xcfun-gpu/src/runtime/wgpu.rs, crates/xcfun-gpu/src/auto_backend.rs, crates/xcfun-gpu/src/batch.rs, crates/xcfun-gpu/src/error_routing.rs</files>
  <read_first>
    - crates/xcfun-gpu/src/runtime/hip.rs (Plan 06-03 analog — same OnceLock pattern)
    - crates/xcfun-gpu/Cargo.toml (current state — verify cubecl-cuda + cubecl-wgpu dep entries)
    - crates/xcfun-gpu/src/auto_backend.rs (extend with cuda + wgpu + metal arms)
    - crates/xcfun-gpu/src/batch.rs (mirror Batch<CudaRuntime> + Batch<WgpuRuntime> bodies on Plan 06-03 Batch<HipRuntime>)
    - crates/xcfun-gpu/src/error_routing.rs (Plan 06-02 must_fall_back_to_cpu; extend usage at Batch::open)
    - crates/xcfun-core/src/error.rs (XcError::WgpuNoF64 variant from Plan 06-02)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-04" + Pattern 4 (lines 71-77, 480-501)
    - **(W-4 / W-10 revision-1)** Context7 query directive — fetch `/tracel-ai/cubecl` with the prompt `"Feature::Type Elem::Float FloatKind::F64 0.10.0-pre.3 API path"` to confirm the exact API path for the f64 probe BEFORE writing any `feature_enabled(...)` site. The probe argument MUST be the explicit `cubecl::Feature::Type(cubecl::ir::Elem::Float(cubecl::ir::FloatKind::F64))` (verified against cubecl 0.10.0-pre.3) — also accessible as `https://github.com/tracel-ai/cubecl/blob/main/cubecl-book/src/core-features/features.md` if Context7 is unavailable. Replace any `feature_enabled(...)` placeholder with this explicit argument, propagated CONSISTENTLY at every probe site (CudaRuntime probe in `runtime/cuda.rs`, WgpuRuntime probe in `runtime/wgpu.rs`).
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Verified pattern: feature probing for a specific float type" (lines 710-728)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pitfall 5" + §"Pitfall 9"
  </read_first>
  <action>
**Step A — Verify Cargo.toml feature flags exist (Plan 06-02 declared but stub):**

```toml
# crates/xcfun-gpu/Cargo.toml
[features]
default = ["cpu"]
cpu  = ["dep:cubecl-cpu"]
hip  = ["dep:cubecl-hip"]      # Plan 06-03
cuda = ["dep:cubecl-cuda"]     # this plan
wgpu = ["dep:cubecl-wgpu"]     # this plan
metal = ["wgpu"]               # Per RESEARCH Pitfall 9: alias of wgpu (cubecl-metal doesn't exist)

[dependencies]
cubecl-cuda = { workspace = true, optional = true }
cubecl-wgpu = { workspace = true, optional = true }
```

**Step B — Create `crates/xcfun-gpu/src/runtime/cuda.rs`:**

```rust
//! CudaRuntime probe + client cache. Per Phase 6 D-06 (CUDA opt-in best-effort).

use cubecl::prelude::*;
use cubecl_cuda::{CudaDevice, CudaRuntime};
use std::sync::OnceLock;

pub type CudaClient = ComputeClient<<CudaRuntime as cubecl::Runtime>::Server, <CudaRuntime as cubecl::Runtime>::Channel>;

static CUDA_CLIENT: OnceLock<CudaClient> = OnceLock::new();

pub fn cuda_available() -> bool {
    if CUDA_CLIENT.get().is_some() { return true; }
    let result = std::panic::catch_unwind(|| {
        let device = CudaDevice::default();
        CudaRuntime::client(&device)
    });
    match result {
        Ok(client) => {
            // (W-7 revision-1) Defensive f64 probe — CUDA f64 reportedly always supported on
            // real CUDA devices, but the cubecl-book feature matrix flags it as "?" (support varies).
            // Use the same Feature::Type(Elem::Float(FloatKind::F64)) gate as Wgpu per W-4/W-10.
            // If the probe fails, do NOT cache the client — return false so auto_backend falls through.
            if !client.properties().feature_enabled(
                cubecl::Feature::Type(cubecl::ir::Elem::Float(cubecl::ir::FloatKind::F64)))
            {
                return false;   // typed XcError::CudaNoF64 emitted at Batch::open (D-13-A pattern).
            }
            let _ = CUDA_CLIENT.set(client);
            true
        }
        Err(_) => false,
    }
}

/// (W-7 revision-1) Construct typed XcError::CudaNoF64 with Box::leak-once adapter name.
/// Used by Batch::open when caller pre-selects Cuda but device fails the f64 probe.
pub fn cuda_no_f64_error(requested: crate::Backend) -> xcfun_core::XcError {
    let device = CudaDevice::default();
    let client = CudaRuntime::client(&device);
    let adapter_name: String = client.adapter_info()
        .map(|info| info.name)
        .unwrap_or_else(|| "<unknown>".to_string());
    let leaked: &'static str = Box::leak(adapter_name.into_boxed_str());
    xcfun_core::XcError::CudaNoF64 {
        adapter_name: leaked,
        requested_runtime: requested.into(),
    }
}

pub fn cuda_client() -> &'static CudaClient {
    CUDA_CLIENT.get().expect("cuda_available() returned false; check CUDA toolkit + nvidia-smi")
}
```

**Step C — Create `crates/xcfun-gpu/src/runtime/wgpu.rs`:**

```rust
//! WgpuRuntime probe + client cache. Per Phase 6 D-06 (Wgpu portable fallback).
//!
//! Pitfall 5: WGSL has no f64 type — wgpu-WGSL emits 32-bit code regardless.
//! Only the SPIR-V backend honours f64 properly. Therefore the f64 probe is
//! REQUIRED before any Batch<WgpuRuntime>::launch call. Plan 06-04 returns
//! XcError::WgpuNoF64 at Batch::open when SHADER_F64 is absent (D-13/D-13-A).
//!
//! Apple Silicon caveat: Apple Silicon GPUs lack hardware f64. cubecl-wgpu
//! on M1/M2/M3 will refuse — Apple Silicon is effectively CPU-only.

use cubecl::prelude::*;
use cubecl_wgpu::{WgpuDevice, WgpuRuntime};
use std::sync::OnceLock;

pub type WgpuClient = ComputeClient<<WgpuRuntime as cubecl::Runtime>::Server, <WgpuRuntime as cubecl::Runtime>::Channel>;

static WGPU_CLIENT: OnceLock<WgpuClient> = OnceLock::new();

/// Probe + cache. Returns true only if SHADER_F64 is reported by the device.
fn try_init_wgpu() -> Option<WgpuClient> {
    std::panic::catch_unwind(|| {
        let device = WgpuDevice::default();
        let client = WgpuRuntime::client(&device);
        // Per RESEARCH §"Verified pattern: feature probing for a specific float type":
        // Feature::Type(Elem::Float(FloatKind::F64))
        if !client.properties()
                  .feature_enabled(cubecl::Feature::Type(cubecl::ir::Elem::Float(cubecl::ir::FloatKind::F64)))
        {
            return None;
        }
        Some(client)
    }).ok().flatten()
}

pub fn wgpu_with_shader_f64_available() -> bool {
    if let Some(c) = WGPU_CLIENT.get() { return c.properties().feature_enabled(...); }   // already cached
    match try_init_wgpu() {
        Some(c) => { let _ = WGPU_CLIENT.set(c); true }
        None => false,
    }
}

/// Apple Silicon Metal path. Per CONTEXT.md D-06: Apple Silicon LACKS hardware
/// f64 → returns false on Apple Silicon. On Intel Mac with Metal + f64-capable
/// discrete GPU it could return true; in practice rare.
pub fn metal_with_f64_available() -> bool {
    // Discretion: detect macOS adapter via `client.properties()`. On Linux/Windows return false.
    // Conservative: rely on the same SHADER_F64 probe; if device backend == Metal AND f64 → true.
    #[cfg(target_os = "macos")]
    {
        try_init_wgpu().map(|client| {
            // ... inspect AdapterInfo backend == Metal ... f64 already confirmed by try_init_wgpu Some path
            true
        }).unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    { false }
}

pub fn wgpu_client() -> &'static WgpuClient {
    WGPU_CLIENT.get().expect("wgpu_with_shader_f64_available() returned false")
}

/// Construct the typed XcError::WgpuNoF64 with Box::leak-once adapter name.
/// Used by Batch::open when caller pre-selects Wgpu/Metal but device lacks f64.
pub fn wgpu_no_f64_error(requested: crate::Backend) -> xcfun_core::XcError {
    let device = WgpuDevice::default();
    let client = WgpuRuntime::client(&device);
    let adapter_name: String = client.adapter_info()  // or analogous accessor
        .map(|info| info.name)
        .unwrap_or_else(|| "<unknown>".to_string());
    // D-13-A: Box::leak once at runtime to obtain &'static str (justified — one-time
    // panic-on-misconfiguration message; preserves XcError Copy + non_exhaustive).
    let leaked: &'static str = Box::leak(adapter_name.into_boxed_str());
    xcfun_core::XcError::WgpuNoF64 {
        adapter_name: leaked,
        requested_runtime: requested.into(),  // Backend → BackendTag
    }
}
```

**Step D — Wire `auto_backend()` Cuda + Metal + Wgpu arms:**

In `crates/xcfun-gpu/src/auto_backend.rs`, ensure all four `#[cfg(feature = ...)]` arms are present (Plan 06-02 had stubs; Plan 06-03 wired hip; this plan wires cuda + metal + wgpu):

```rust
pub fn auto_backend() -> Backend {
    if let Ok(force) = std::env::var("XCFUN_FORCE_BACKEND") {
        return Backend::from_str(&force).unwrap_or_else(|| panic!("XCFUN_FORCE_BACKEND={} unrecognised", force));
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

**Step E — Implement `Batch<CudaRuntime>::eval_vec_host` and `Batch<WgpuRuntime>::eval_vec_host`:**

Mirror the Plan 06-03 `Batch<HipRuntime>::eval_vec_host` body. For Wgpu, add an explicit f64 check in `Batch::open`:

```rust
#[cfg(feature = "wgpu")]
impl<'fun> Batch<'fun, cubecl_wgpu::WgpuRuntime> {
    pub fn open(fun: &'fun xcfun_rs::Functional) -> Result<Self, xcfun_core::XcError> {
        if !crate::runtime::wgpu::wgpu_with_shader_f64_available() {
            return Err(crate::runtime::wgpu::wgpu_no_f64_error(crate::Backend::Wgpu));
        }
        let client = crate::runtime::wgpu::wgpu_client().clone();
        Self::open_with_client(fun, client)
    }
    pub fn eval_vec_host(
        fun: &'fun xcfun_rs::Functional,
        density: &[f64], density_pitch: usize,
        out: &mut [f64], out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), xcfun_core::XcError> {
        // Step 1: ERF-bearing functional + Wgpu → fall back to Cpu (GPU-05).
        if crate::error_routing::must_fall_back_to_cpu(fun.dependencies(), crate::Backend::Wgpu) {
            return Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host(
                fun, density, density_pitch, out, out_pitch, nr_points
            );
        }
        // Step 2: open + reserve + upload + launch + download.
        let mut batch = Self::open(fun)?;
        batch.reserve(nr_points);
        batch.upload_density(density, density_pitch, nr_points);
        batch.launch(nr_points as u32)?;
        batch.download_result(out, out_pitch, nr_points);
        Ok(())
    }
}

// Cuda: same shape as Hip from Plan 06-03; no f64 probe (CUDA always supports f64
// per RESEARCH §"Standard Stack" — but feature_matrix flags it as "?" — defensive
// best-effort: don't probe; expect cloud-CI runs to surface device-specific issues).
#[cfg(feature = "cuda")]
impl<'fun> Batch<'fun, cubecl_cuda::CudaRuntime> { /* mirror Hip body */ }
```

**Step F — Wire `error_routing.rs` (already created in Plan 06-02; verify it's used in Step E):**

```rust
// crates/xcfun-gpu/src/error_routing.rs (Plan 06-02 baseline; Plan 06-04 verifies usage)
pub fn must_fall_back_to_cpu(deps: xcfun_core::Dependency, backend: crate::Backend) -> bool {
    deps.contains(xcfun_core::Dependency::ERF) && matches!(backend, crate::Backend::Wgpu | crate::Backend::Metal)
}
```

**Step G — Verify `BackendTag` impl is reachable:**

xcfun-gpu::Backend → xcfun-core::BackendTag conversion (via `From`/`Into`) was declared in Plan 06-02. Confirm:
```rust
impl From<crate::Backend> for xcfun_core::BackendTag {
    fn from(b: crate::Backend) -> Self {
        match b {
            crate::Backend::Cpu => xcfun_core::BackendTag::Cpu,
            crate::Backend::Rocm => xcfun_core::BackendTag::Rocm,
            crate::Backend::Cuda => xcfun_core::BackendTag::Cuda,
            crate::Backend::Metal => xcfun_core::BackendTag::Metal,
            crate::Backend::Wgpu => xcfun_core::BackendTag::Wgpu,
        }
    }
}
```

**Step H — Multi-feature compile gate (GPU-03):**

```bash
cargo build -p xcfun-gpu --features hip --features cuda --features wgpu
cargo build -p xcfun-gpu --features metal           # alias of wgpu — same result
```

Both must exit 0.

**Forbidden:**
- Do NOT add `cubecl-metal` to `[dependencies]` — does not exist on crates.io (RESEARCH §R-02). The `metal` feature aliases to `wgpu`.
- Do NOT enable `wgpu` Wgsl compute path without the SHADER_F64 probe — silent f32 downgrade is the #1 Phase 6 numerical risk per RESEARCH §"Pitfall 2".
- Do NOT use `#[cube(fast_math = ...)]` anywhere; `xtask check-no-fma` blocks. RESEARCH §"Anti-Patterns" item 3.
  </action>
  <verify>
    <automated>cargo build -p xcfun-gpu --features hip --features cuda --features wgpu && cargo build -p xcfun-gpu --features metal && cargo run -p xtask --bin check-cubecl-pin</automated>
  </verify>
  <acceptance_criteria>
    - `crates/xcfun-gpu/src/runtime/cuda.rs` exists; `grep -c "cuda_available\|CudaRuntime" crates/xcfun-gpu/src/runtime/cuda.rs` >= 2
    - `crates/xcfun-gpu/src/runtime/wgpu.rs` exists; `grep -c "wgpu_with_shader_f64_available\|metal_with_f64_available" crates/xcfun-gpu/src/runtime/wgpu.rs` >= 2
    - `grep -c "FloatKind::F64\|SHADER_F64" crates/xcfun-gpu/src/runtime/wgpu.rs` >= 1
    - `grep -c "Box::leak" crates/xcfun-gpu/src/runtime/wgpu.rs` >= 1
    - `grep -c "cuda_no_f64_error\|CudaNoF64" crates/xcfun-gpu/src/runtime/cuda.rs` >= 1   # W-7 (revision-1) typed CUDA f64 probe
    - `grep -c "FloatKind::F64" crates/xcfun-gpu/src/runtime/cuda.rs` >= 1                  # W-7: same gate as Wgpu
    - `grep -c "cuda_available" crates/xcfun-gpu/src/auto_backend.rs` >= 1
    - `grep -c "wgpu_with_shader_f64_available\|metal_with_f64_available" crates/xcfun-gpu/src/auto_backend.rs` >= 2
    - `grep -c "CudaRuntime\|cubecl_cuda" crates/xcfun-gpu/src/batch.rs` >= 1
    - `grep -c "WgpuRuntime\|cubecl_wgpu" crates/xcfun-gpu/src/batch.rs` >= 1
    - `grep -c "must_fall_back_to_cpu\|Dependency::ERF" crates/xcfun-gpu/src/batch.rs` >= 1
    - `grep -c "metal\s*=\s*\[\"wgpu\"\]" crates/xcfun-gpu/Cargo.toml` >= 1
    - `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` exits 0.
    - `cargo build -p xcfun-gpu --features metal` exits 0.
    - `cargo run -p xtask --bin check-cubecl-pin` exits 0 (5 crates lockstep).
    - No `cubecl-metal` string in `crates/xcfun-gpu/Cargo.toml`: `grep -c '"cubecl-metal"' crates/xcfun-gpu/Cargo.toml` == 0
  </acceptance_criteria>
  <done>cubecl-cuda + cubecl-wgpu opt-in features wired; CudaRuntime + WgpuRuntime probes implemented; Batch<CudaRuntime> + Batch<WgpuRuntime> bodies in place; Wgpu path enforces SHADER_F64 with typed XcError::WgpuNoF64 + Box::leak adapter_name; ERF auto-fallback machinery wired through error_routing.rs; multi-feature compile GREEN.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: wgpu_no_f64 + erf_fallback tests + validation tier-3 Wgpu/Cuda dispatch arms</name>
  <files>crates/xcfun-gpu/tests/wgpu_no_f64.rs, crates/xcfun-gpu/tests/erf_fallback.rs, validation/Cargo.toml, validation/src/main.rs, validation/src/driver.rs</files>
  <read_first>
    - crates/xcfun-gpu/tests/wgpu_no_f64.rs (Plan 06-02 placeholder; un-#[ignore] and fill body)
    - crates/xcfun-eval/tests/regularize_mgga_invariant.rs (analog Dependency-aware test)
    - validation/src/main.rs (Plan 06-02 baseline; ensure --backend cuda + --backend wgpu accepted)
    - validation/src/driver.rs::run_tier3 (Plan 06-02 + Plan 06-03 baseline; extend Cuda + Wgpu arms)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-04 erf_fallback.rs" (lines 71-77, 651-657)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md per-task map row 06-04-01 (GPU-03 + GPU-08 compile gate + Wgpu tier-3 1e-9)
  </read_first>
  <behavior>
    - Test 1 (`wgpu_no_f64.rs`): un-`#[ignore]` per Plan 06-02 placeholder. Force-select Wgpu via `XCFUN_FORCE_BACKEND=wgpu`; attempt `Batch::<WgpuRuntime>::open(&fun)`. Assert that EITHER (a) device has SHADER_F64 → returns Ok (acceptable on dev hosts with Vulkan f64 ext) OR (b) device lacks SHADER_F64 → returns `Err(XcError::WgpuNoF64 { adapter_name, requested_runtime: BackendTag::Wgpu })`. The test asserts the result type matches one of these two options (no third path; specifically no silent downgrade to f32).
    - Test 2 (`erf_fallback.rs`): set up XC_LDAERFX (ERF-bearing functional). Force-select Wgpu. Call `Batch::<WgpuRuntime>::eval_vec_host(...)`. Assert output matches CPU baseline within 1e-13 (because the kernel was internally re-routed to CpuRuntime via `must_fall_back_to_cpu`). Use a side-channel (e.g., a debug counter inside Batch::eval_vec_host) to assert which Backend actually executed.
    - Both tests gated on `--features wgpu`.
  </behavior>
  <action>
**Step A — Un-`#[ignore]` and complete `crates/xcfun-gpu/tests/wgpu_no_f64.rs`:**

```rust
#![cfg(feature = "wgpu")]
//! GPU-06 / D-13/D-13-A — Wgpu device without SHADER_F64 returns typed
//! XcError::WgpuNoF64 at Batch::open. NEVER silently downgrades to f32.

use xcfun_gpu::Batch;
use xcfun_core::XcError;
use xcfun_rs::Functional;
use cubecl_wgpu::WgpuRuntime;

#[test]
fn wgpu_no_f64_returns_typed_error_or_succeeds_on_f64_device() {
    let mut fun = Functional::new();
    fun.set("slaterx", 1.0).unwrap();
    fun.eval_setup(xcfun_core::Vars::A_B, xcfun_core::Mode::PartialDerivatives, 0).unwrap();

    match Batch::<WgpuRuntime>::open(&fun) {
        Ok(_) => {
            // Adapter HAS SHADER_F64 — Vulkan with f64 extension or SPIR-V backend.
            // No f32 downgrade observed; OK.
        }
        Err(XcError::WgpuNoF64 { adapter_name, requested_runtime }) => {
            // Adapter lacks SHADER_F64 — typed error returned. NEVER silent downgrade.
            assert!(!adapter_name.is_empty(), "adapter_name should be populated");
            assert_eq!(requested_runtime, xcfun_core::BackendTag::Wgpu);
        }
        Err(other) => panic!("unexpected error variant: {:?}", other),
    }
}
```

**Step B — Create `crates/xcfun-gpu/tests/erf_fallback.rs`:**

```rust
#![cfg(feature = "wgpu")]
//! GPU-05 — On Wgpu, functionals with Dependency::ERF are auto-routed to Cpu
//! at Batch::eval_vec_host time. Output matches CPU within 1e-13.

use xcfun_gpu::Batch;
use xcfun_rs::Functional;
use cubecl_wgpu::WgpuRuntime;
use approx::assert_relative_eq;

#[test]
fn erf_bearing_functional_falls_back_to_cpu() {
    let mut fun = Functional::new();
    fun.set("ldaerfx", 1.0).unwrap();
    fun.eval_setup(xcfun_core::Vars::A_B, xcfun_core::Mode::PartialDerivatives, 1).unwrap();

    // Set up batch on 100 random density points.
    let nr_points = 100;
    let inlen     = fun.input_length();
    let outlen    = fun.output_length().unwrap();
    let density: Vec<f64> = (0..nr_points * inlen).map(|i| 0.5 + (i as f64) * 0.001).collect();
    let mut wgpu_out  = vec![0.0_f64; nr_points * outlen];
    let mut cpu_out   = vec![0.0_f64; nr_points * outlen];

    // Path A: explicit Wgpu request — should auto-fall-back to CPU per GPU-05.
    Batch::<WgpuRuntime>::eval_vec_host(&fun, &density, inlen, &mut wgpu_out, outlen, nr_points)
        .expect("ERF auto-fallback should yield Ok via CpuRuntime override");

    // Path B: scalar CPU baseline.
    for k in 0..nr_points {
        let in_slice  = &density[k * inlen..(k + 1) * inlen];
        let out_slice = &mut cpu_out[k * outlen..(k + 1) * outlen];
        fun.eval(in_slice, out_slice).unwrap();
    }

    for i in 0..wgpu_out.len() {
        assert_relative_eq!(wgpu_out[i], cpu_out[i], max_relative = 1e-13);
    }
}
```

**Step C — Update `validation/Cargo.toml`** to forward `--features cuda` + `--features wgpu`:

```toml
[features]
default = []
hip  = ["xcfun-gpu/hip"]    # Plan 06-03
cuda = ["xcfun-gpu/cuda"]   # Plan 06-04 (this plan)
wgpu = ["xcfun-gpu/wgpu"]   # Plan 06-04 (this plan)
metal = ["xcfun-gpu/metal"]
```

**Step D — Wire `--backend cuda` + `--backend wgpu` arms in `validation/src/driver.rs::run_tier3`:**

Following the Plan 06-03 Hip-arm pattern, add:

```rust
match backend_e {
    Backend::Cpu => { /* Plan 06-02 */ }
    #[cfg(feature = "hip")]
    Backend::Rocm => { /* Plan 06-03 */ }
    #[cfg(feature = "cuda")]
    Backend::Cuda => {
        let client = xcfun_gpu::runtime::cuda::cuda_client().clone();
        // ... mirror Hip arm with CudaRuntime / cuda_client ...
        Batch::<cubecl_cuda::CudaRuntime>::eval_vec_host(&fun, &density_flat, density_pitch, &mut batch_out, out_pitch, grid.len())?;
    }
    #[cfg(feature = "wgpu")]
    Backend::Wgpu | Backend::Metal => {
        // Wgpu probe → returns XcError::WgpuNoF64 if device lacks SHADER_F64.
        // For tier-3 1e-9 sweep, exclude ERF-bearing functionals (--exclude-erf flag).
        Batch::<cubecl_wgpu::WgpuRuntime>::eval_vec_host(&fun, &density_flat, density_pitch, &mut batch_out, out_pitch, grid.len())?;
    }
    #[cfg(not(feature = "cuda"))]
    Backend::Cuda => anyhow::bail!("--backend cuda requires --features cuda"),
    #[cfg(not(feature = "wgpu"))]
    Backend::Wgpu | Backend::Metal => anyhow::bail!("--backend {:?} requires --features wgpu", backend_e),
    _ => unreachable!(),
}

// Tolerance: ROCm/CUDA at strict 1e-13, Wgpu at 1e-9 per ROADMAP success criterion 4 / D-02.
let tolerance = match backend_e {
    Backend::Cpu | Backend::Rocm | Backend::Cuda => 1e-13_f64,
    Backend::Wgpu | Backend::Metal               => 1e-9_f64,
};
if max_rel_err > tolerance {
    anyhow::bail!("tier-3 {:?} {} max_rel_err {} exceeds {} tolerance", backend_e, tuple.functional_name, max_rel_err, tolerance);
}
```

**Step E — Verify GPU-08 compile gate:**

```bash
cargo build -p xcfun-gpu --features hip --features cuda --features wgpu
cargo build -p validation --release --features hip --features cuda --features wgpu
```

Both exit 0.

Tier-3 Wgpu sweep is MANUAL VERIFICATION when a Wgpu-with-f64 adapter is available (commonly Linux Vulkan with f64 ext). Document the command in `06-04-SUMMARY.md`:

```bash
cargo run -p validation --release --features wgpu -- \
  --backend wgpu --tier 3 --order 3 --exclude-erf \
  --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'
```

Expected: 0 failing at 1e-9 (ERF-bearing functionals filtered by `--exclude-erf`; the runtime auto-fallback also kicks in for any that slip through).

**Step F — Run the new tests:**

```bash
cargo nextest run -p xcfun-gpu --features wgpu --test wgpu_no_f64
cargo nextest run -p xcfun-gpu --features wgpu --test erf_fallback
```

Expected:
- `wgpu_no_f64`: GREEN if Wgpu adapter is reachable (returns Ok or typed error). On hosts without any Wgpu adapter at all, the test panics on `WgpuRuntime::client(&device)` — wrap in `catch_unwind` if needed.
- `erf_fallback`: GREEN — the ERF-routing kicks in regardless of device f64 capability (it's a host-side decision before runtime probe).
  </action>
  <verify>
    <automated>cargo build -p xcfun-gpu --features hip --features cuda --features wgpu && cargo build -p validation --release --features wgpu && cargo nextest run -p xcfun-gpu --features wgpu --test erf_fallback</automated>
  </verify>
  <acceptance_criteria>
    - `crates/xcfun-gpu/tests/wgpu_no_f64.rs` exists; NOT `#[ignore]`'d; `grep -c "WgpuNoF64\|SHADER_F64" crates/xcfun-gpu/tests/wgpu_no_f64.rs` >= 1
    - `crates/xcfun-gpu/tests/erf_fallback.rs` exists; `grep -c "ldaerfx\|Dependency::ERF\|must_fall_back" crates/xcfun-gpu/tests/erf_fallback.rs` >= 1
    - `grep -c "Backend::Cuda\|Backend::Wgpu" validation/src/driver.rs` >= 2
    - `grep -c '1e-9\|1.0e-9' validation/src/driver.rs` >= 1
    - `grep -c '"hip"\|"cuda"\|"wgpu"' validation/Cargo.toml` >= 3
    - `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` exits 0.
    - `cargo build -p validation --release --features wgpu` exits 0.
    - `cargo nextest run -p xcfun-gpu --features wgpu --test erf_fallback` exits 0 (host-side fallback always works regardless of GPU adapter).
    - `cargo run -p validation --release -- --backend cuda` (no `--features cuda`) exits non-zero with helpful error message.
  </acceptance_criteria>
  <done>wgpu_no_f64 + erf_fallback tests landed and GREEN; validation harness `--backend cuda` + `--backend wgpu` arms wired; tier-3 Wgpu 1e-9 driver path documented; ERF auto-fallback verified at integration level.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust ↔ wgpu adapter | Adapter info / SHADER_F64 feature reported by device may not match actual hardware — explicit f64 probe before any kernel launch |
| Rust ↔ CUDA toolkit | cubecl-cuda dispatches via PTX — no fast-math by default per RESEARCH §"Anti-Patterns" item 3 |
| `Box::leak` once for adapter_name | Per D-13-A — bounded leak (one per (process, adapter); R-05) |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-WGPU-F32 | high | Silent f32 downgrade on devices without SHADER_F64 | runtime/wgpu.rs probe + `XcError::WgpuNoF64` typed error at Batch::open; compile-time `size_of::<Scalar>() == 8` from xcfun-kernels root (Plan 06-02 Step C); test wgpu_no_f64.rs asserts no third path |
| T-06-CUBECL-DRIFT | high | cubecl-cuda 0.10.0-pre.4 paired with cubecl 0.10.0-pre.3 | xtask check-cubecl-pin extended to 5 crates (Plan 06-03 Step G); CI gate runs on PR |
| T-06-FAST-MATH | high | cubecl-cuda PTX `--use_fast_math` flag — verify via cubecl-cuda source inspection | Plan 06-04 Task 1 forbids `#[cube(fast_math = ...)]`; verify cubecl source does not enable globally; if it does, file an upstream issue and disable per-launch |
| T-06-METAL-NONEXISTENT | low | Code references non-existent `cubecl-metal` would fail to build | RESEARCH §R-02 explicit; Cargo.toml has `metal = ["wgpu"]` alias only; acceptance criteria greps `"cubecl-metal"` in Cargo.toml == 0 |
| T-06-OOM | medium | wgpu adapter may report low memory for large `density_buf` allocations | Powers-of-two doubling capped (no shrink); Batch::open returns Err on allocation failure (cubecl propagates) |
| T-06-LEAK | low | Box::leak in `wgpu_no_f64_error` — bounded per (process, adapter) | Per RESEARCH §R-05 — handful of bytes per process lifetime; consider `static OnceLock<&'static str>` cache to amortise |
</threat_model>

<verification>
- All 2 tasks GREEN per their automated commands.
- Multi-feature compile gate `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` exits 0 (GPU-03).
- xtask check-cubecl-pin GREEN with 5 crates lockstep.
- erf_fallback test GREEN (works regardless of GPU adapter availability — host-side decision).
- wgpu_no_f64 test GREEN (passes on both f64-capable and f64-lacking adapters via match arm).
- No new `cubecl-metal` reference: `grep -rE 'cubecl-metal\|cubecl_metal' crates/ Cargo.toml` returns empty (or only documentation comments).
- Tier-3 Wgpu 1e-9 sign-off (GPU-08) documented as MANUAL verification per VALIDATION.md.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 3 advanced: cubecl-cuda enabled behind `cuda`, cubecl-wgpu behind `wgpu`; auto_backend selects per D-07 priority chain (GPU-02 / GPU-03).
- ROADMAP Phase 6 success criterion 4 advanced: Tier-3 parity on Wgpu — 10k-grid (excluding ERF) within 1e-9 rel-err vs CPU code path complete (GPU-08).
- ROADMAP Phase 6 success criterion 5 advanced: Wgpu without SHADER_F64 returns typed XcError::WgpuNoF64 at Batch::open (GPU-06; D-13/D-13-A); compile-time `size_of::<Scalar>() == 8` already in place from Plan 06-02.
- D-06 architectural goal (CUDA + Metal opt-in) landed.
- Plan 06-05 unblocked: complete `Batch<R>` matrix exists for all 4 cubecl runtimes (CpuRuntime + HipRuntime + CudaRuntime + WgpuRuntime).
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-04-SUMMARY.md` documenting:
- cubecl-cuda + cubecl-wgpu opt-in features wired
- runtime/cuda.rs + runtime/wgpu.rs with SHADER_F64 probe (Plan 06-04 / Pitfall 5)
- `metal = ["wgpu"]` alias per RESEARCH §R-02 (cubecl-metal does not exist)
- Batch<CudaRuntime> + Batch<WgpuRuntime> bodies
- ERF auto-fallback at Batch::eval_vec_host level (GPU-05; tested in erf_fallback.rs)
- typed XcError::WgpuNoF64 with Box::leak adapter_name (D-13/D-13-A; tested in wgpu_no_f64.rs)
- validation harness `--backend cuda` + `--backend wgpu` dispatch arms; tier-3 1e-9 Wgpu / 1e-13 CUDA driver paths
- Manual verification commands for cloud-CI (CUDA tier-3) + local Linux Vulkan f64 (Wgpu tier-3)
- RESEARCH §"Anti-Patterns" verification: cubecl-cuda source inspected for `--use_fast_math` PTX flag; result documented
</output>
