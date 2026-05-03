---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 03
type: execute
wave: 5
depends_on:
  - 06-02a
  - 06-02b
files_modified:
  - crates/xcfun-gpu/Cargo.toml
  - crates/xcfun-gpu/src/runtime/hip.rs
  - crates/xcfun-gpu/src/auto_backend.rs
  - crates/xcfun-gpu/src/batch.rs
  - crates/xcfun-gpu/src/pool.rs
  - crates/xcfun-gpu/README.md
  - validation/Cargo.toml
  - validation/src/main.rs
  - validation/src/driver.rs
  - xtask/src/bin/check_cubecl_pin.rs
autonomous: true
requirements:
  - GPU-02
  - GPU-07
must_haves:
  truths:
    - "cubecl-hip = =0.10.0-pre.3 wired as opt-in feature `hip` of xcfun-gpu (per D-05 ROCm primary)."
    - "auto_backend() priority chain returns Backend::Rocm when feature `hip` is enabled AND `rocm_available()` (HipRuntime::client succeeds + device exists)."
    - "Batch<HipRuntime>::eval_vec_host body wired (mirrors Batch<CpuRuntime> from Plan 06-02)."
    - "validation harness `--backend rocm` flag dispatches to Batch::<HipRuntime>; tier-3 ROCm 10k-grid 1e-13 driver path complete."
    - "xcfun-gpu/README.md documents RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0` requirement (per RESEARCH Pitfall 3)."
    - "xtask check-cubecl-pin scope extended to 5 crates: cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu (lockstep at =0.10.0-pre.3)."
    - "Tier-3 ROCm sign-off bar (per D-02): `cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '<known-clean-17>'` GREEN at strict 1e-13 vs CPU. **Documented as MANUAL verification per VALIDATION.md** (no local ROCm runtime in dev env per RESEARCH §R-01); blocked on Quick Task / cloud-CI runner."
  artifacts:
    - path: "crates/xcfun-gpu/src/runtime/hip.rs"
      provides: "HipRuntime client OnceLock + rocm_available() probe"
      contains: "rocm_available"
    - path: "crates/xcfun-gpu/Cargo.toml"
      provides: "hip = [\"dep:cubecl-hip\"] feature flag"
      contains: "cubecl-hip"
    - path: "crates/xcfun-gpu/README.md"
      provides: "RDNA-2 HSA_OVERRIDE_GFX_VERSION=10.3.0 + Apple Silicon caveat + env vars"
      contains: "HSA_OVERRIDE_GFX_VERSION"
    - path: "validation/src/main.rs"
      provides: "--backend rocm CLI dispatch"
      contains: "rocm"
    - path: "xtask/src/bin/check_cubecl_pin.rs"
      provides: "PINNED_CRATES = 5 entries"
      contains: "cubecl-hip"
  key_links:
    - from: "crates/xcfun-gpu/src/auto_backend.rs"
      to: "crates/xcfun-gpu/src/runtime/hip.rs::rocm_available"
      via: "#[cfg(feature = \"hip\")] gate"
      pattern: "rocm_available"
    - from: "validation/src/driver.rs::run_tier3"
      to: "crates/xcfun-gpu/src/batch.rs::Batch<HipRuntime>"
      via: "match Backend::Rocm arm"
      pattern: "HipRuntime"
---

<objective>
Wire **ROCm/HIP as the PRIMARY GPU backend** (per D-05 / GPU-07) by adding the `cubecl-hip = "=0.10.0-pre.3"` opt-in feature dep, implementing the `HipRuntime` probe + `OnceLock<HipClient>`, monomorphising `Batch<HipRuntime>::eval_vec_host`, and extending the validation harness `--backend rocm` flag.

Per Phase 6 D-05 / RESEARCH §R-01: project dev environment is AMD with ROCm-primary intent BUT local `/opt/rocm` directory was missing at audit time (see RESEARCH §R-01 + §"Open Question 1"). Plan 06-03 ships **the code path** ROCm-ready; tier-3 ROCm GREEN sign-off (per D-02) is gated on either (a) a separate `gsd:quick` task installing ROCm locally, or (b) cloud-CI runner. **This plan's success metric is "code compiles + auto_backend probes correct + manual verification documented in 06-03-SUMMARY.md as the precondition for Phase 6 sign-off"**, NOT requiring local tier-3 GREEN.

Per RESEARCH Pitfall 3: RDNA-2 GPUs (RX 6000-series, gfx1031/1032/1033) need `HSA_OVERRIDE_GFX_VERSION=10.3.0` to coerce-match RDNA-2 to RDNA-3 PTX. Without this env var, kernels fail with "code object load failed" on first launch. Doc this in `xcfun-gpu/README.md` (D-05 explicit).

Per RESEARCH Pitfall 1: cubecl pre-release crates cross-reference internal types — `cubecl-hip 0.10.0-pre.3` paired with `cubecl 0.10.0-pre.4` produces opaque "type X1 is not the same as type X2" errors. Extend `xtask check-cubecl-pin` to enforce all 5 crates lockstep at `=0.10.0-pre.3` (cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu).

Purpose: ROCm/HIP is the primary GPU target per the user's locked decision (D-05). This plan turns the Plan 06-02 skeleton (`auto_backend()` returns Cpu fallback) into a working ROCm-aware dispatch. Plans 06-04 (CUDA + Wgpu) and 06-05 (RS-08 wiring) build atop.

Output: cubecl-hip dep wired; HipRuntime probe + OnceLock; Batch<HipRuntime>::eval_vec_host body; validation `--backend rocm` flag; README doc note; xtask check-cubecl-pin scope extension (5 crates).
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
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@crates/xcfun-gpu/Cargo.toml
@crates/xcfun-gpu/src/auto_backend.rs
@crates/xcfun-gpu/src/batch.rs
@crates/xcfun-gpu/src/pool.rs
@crates/xcfun-gpu/src/runtime/cpu.rs
@xtask/src/bin/check_cubecl_pin.rs
@validation/src/main.rs
@validation/src/driver.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Wire cubecl-hip as opt-in feature + HipRuntime probe + Batch<HipRuntime> + xtask 5-crate pin gate</name>
  <files>crates/xcfun-gpu/Cargo.toml, crates/xcfun-gpu/src/runtime/hip.rs, crates/xcfun-gpu/src/auto_backend.rs, crates/xcfun-gpu/src/batch.rs, crates/xcfun-gpu/src/pool.rs, crates/xcfun-gpu/README.md, xtask/src/bin/check_cubecl_pin.rs</files>
  <read_first>
    - crates/xcfun-gpu/Cargo.toml (current Plan 06-02 state — `hip = ["dep:cubecl-hip"]` already declared but cubecl-hip dep itself stub-only; verify and ensure dep entry exists)
    - crates/xcfun-gpu/src/runtime/cpu.rs (analog probe pattern from Plan 06-02)
    - crates/xcfun-gpu/src/auto_backend.rs (full file — extend the `#[cfg(feature = "hip")]` arm)
    - crates/xcfun-gpu/src/batch.rs (full file — extend Batch<HipRuntime> body using the Cpu arm as analog)
    - crates/xcfun-gpu/src/pool.rs (extend with `static HIP_CLIENT: OnceLock<HipClient>`)
    - xtask/src/bin/check_cubecl_pin.rs (full file — see PINNED_CRATES constant)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-03" (lines 60-67, 619-644)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pitfall 3" (RDNA-2) + §"Pitfall 1" (cubecl drift)
    - https://github.com/tracel-ai/cubecl/blob/main/crates/cubecl-hip/README.md (RDNA-2 note — fetch fresh via Context7 if available)
  </read_first>
  <action>
**Step A — Verify / fix `crates/xcfun-gpu/Cargo.toml` `cubecl-hip` dep:**

Plan 06-02 declared the feature flag `hip = ["dep:cubecl-hip"]` but stub-only. Confirm the optional dep line exists:
```toml
[dependencies]
cubecl-hip = { workspace = true, optional = true }
```
And the workspace pin in root `Cargo.toml` `[workspace.dependencies]` is `cubecl-hip = "=0.10.0-pre.3"` (added in Plan 06-02 Step A).

Verify `cargo build -p xcfun-gpu --features hip` compiles (will succeed if cubecl-hip is published — verified at crates.io 2026-04-30 per RESEARCH §"Standard Stack").

**Step B — Create `crates/xcfun-gpu/src/runtime/hip.rs`:**

```rust
//! HipRuntime probe + client cache. Per Phase 6 D-05 (ROCm primary).
//!
//! RDNA-2 caveat (RESEARCH Pitfall 3): RX 6000-series GPUs need
//!     export HSA_OVERRIDE_GFX_VERSION=10.3.0
//! before any binary launches Backend::Rocm. Documented in xcfun-gpu/README.md.

use cubecl::prelude::*;
use cubecl_hip::{HipDevice, HipRuntime};
use std::sync::OnceLock;

pub type HipClient = ComputeClient<<HipRuntime as cubecl::Runtime>::Server, <HipRuntime as cubecl::Runtime>::Channel>;

static HIP_CLIENT: OnceLock<HipClient> = OnceLock::new();

/// Probe whether ROCm is available — attempts to construct a HipClient.
/// Returns false on missing /opt/rocm runtime, missing GPU, or HipRuntime init error.
pub fn rocm_available() -> bool {
    // Conservative probe: attempt client init; cache result via OnceLock.
    HIP_CLIENT.get_or_init(|| {
        let device = HipDevice::default();
        HipRuntime::client(&device)
    });
    HIP_CLIENT.get().is_some()
}

/// Returns the cached HipClient. Panics if `rocm_available()` returned false.
pub fn hip_client() -> &'static HipClient {
    HIP_CLIENT.get().expect("rocm_available() returned false; check ROCm install + HSA_OVERRIDE_GFX_VERSION")
}
```

Note: the `OnceLock::get_or_init` model panics propagate naturally on a real HipRuntime init failure. If a more graceful "probe-without-panic" pattern is desired, wrap the init in `std::panic::catch_unwind` and return `None`. RESEARCH Open Question 1 documents this trade-off; pick the catch_unwind approach for production-grade probe behaviour.

Refined version:
```rust
pub fn rocm_available() -> bool {
    HIP_CLIENT.get_or_try_init(|| {
        std::panic::catch_unwind(|| {
            let device = HipDevice::default();
            HipRuntime::client(&device)
        }).map_err(|_| ())
    }).is_ok()
}
```

(Uses unstable `get_or_try_init` — fall back to a manual `OnceLock::set` pattern if MSRV 1.85 doesn't have it stable.)

**Step C — Wire `auto_backend()` Rocm probe in `crates/xcfun-gpu/src/auto_backend.rs`:**

Plan 06-02 already has the `#[cfg(feature = "hip")]` gate stub. Make sure the implementation calls the new `crate::runtime::hip::rocm_available()`:

```rust
pub fn auto_backend() -> Backend {
    if let Ok(force) = std::env::var("XCFUN_FORCE_BACKEND") {
        return Backend::from_str(&force).unwrap_or_else(|| panic!(...));
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

**Step D — Implement `Batch<HipRuntime>::eval_vec_host` in `crates/xcfun-gpu/src/batch.rs`:**

The `Batch<R>` struct is generic over `R: cubecl::Runtime`, so most logic is shared. The `eval_vec_host` static method specialised for `HipRuntime` looks like:

```rust
#[cfg(feature = "hip")]
impl<'fun> Batch<'fun, cubecl_hip::HipRuntime> {
    pub fn eval_vec_host(
        fun: &'fun xcfun_rs::Functional,
        density: &[f64], density_pitch: usize,
        out: &mut [f64], out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), xcfun_core::XcError> {
        let client = crate::runtime::hip::hip_client().clone();
        let mut batch = Self::open(fun, client)?;
        batch.reserve(nr_points);
        batch.upload_density(density, density_pitch, nr_points);
        batch.launch(nr_points as u32)?;
        batch.download_result(out, out_pitch, nr_points);
        Ok(())
    }
}
```

(In practice the impl is shared via `Batch<R>::eval_vec_host_with_client(client)` and per-runtime dispatch wraps with `runtime::hip::hip_client()`. The shape above is sufficient for clarity.)

**Step E — Extend `crates/xcfun-gpu/src/pool.rs`:**

Add `#[cfg(feature = "hip")]` re-export of `hip_client` from `runtime::hip` so callers can `xcfun_gpu::pool::hip_client()`. Keep the OnceLock itself in `runtime/hip.rs` (per Step B).

**Step F — Create `crates/xcfun-gpu/README.md`:**

```markdown
# xcfun-gpu

GPU batch lifecycle + auto_backend dispatch for `xcfun_rs`.

## Backend Priority (per Phase 6 D-07)

`auto_backend()` selects the runtime in this order:

1. `XCFUN_FORCE_BACKEND` env var (cpu | rocm | cuda | metal | wgpu) — overrides everything
2. ROCm via `cubecl-hip` (PRIMARY per Phase 6 D-05) — feature `hip`
3. CUDA via `cubecl-cuda` — feature `cuda` (community-maintained best-effort)
4. Metal via `cubecl-wgpu` Metal backend — feature `metal` (alias of `wgpu`); requires hardware f64 (Apple Silicon LACKS f64 — falls through)
5. Wgpu (Vulkan / DX12 / WebGPU) via `cubecl-wgpu` — feature `wgpu`; requires `wgpu::Features::SHADER_F64`
6. Cpu via `cubecl-cpu` — always available (validation substrate)

## Environment Variables

| Variable | Required by | Effect |
|----------|-------------|--------|
| `HSA_OVERRIDE_GFX_VERSION=10.3.0` | RDNA-2 GPUs (RX 6000-series, gfx1031/1032/1033) | Coerces RDNA-2 to RDNA-3 PTX. Without this, kernel launches fail with "code object load failed". MANDATORY before any `Backend::Rocm` use on RX 6000-series. |
| `XCFUN_FORCE_BACKEND=<name>` | optional | Forces `auto_backend()` selection. Recognised: `cpu`, `rocm`, `cuda`, `metal`, `wgpu`. |
| `XCFUN_MIN_BATCH_SIZE=<usize>` | optional | Overrides default `eval_vec` dispatch threshold (default 64). |

## Apple Silicon Caveat

Apple Silicon GPUs lack hardware f64. `cubecl-wgpu` on Apple Silicon will runtime-probe and refuse to instantiate `Batch::<WgpuRuntime>` (returns `XcError::WgpuNoF64`). Fall-through: Cpu via cubecl-cpu. Document explicitly: **Apple Silicon = CPU-only.**

## Numerical Tolerance Envelope (per Phase 6 D-02)

| Backend | Strict bar | Per-functional override |
|---------|-----------|-------------------------|
| Cpu (validation substrate) | 1e-13 | none |
| Rocm (PRIMARY) | 1e-13 | none |
| Cuda (opt-in) | 1e-13 (best-effort; cloud-CI) | none |
| Metal (opt-in via Wgpu) | best-effort | range-separated functionals (Dependency::ERF) auto-fall-back to Cpu |
| Wgpu (portable) | 1e-9 | range-separated functionals auto-fall-back to Cpu |

## ROCm Install (Linux)

```bash
# rocm.docs.amd.com Quick Start
sudo apt install rocm-hip-runtime
# RDNA-2 users:
export HSA_OVERRIDE_GFX_VERSION=10.3.0
```

Verify:
```bash
rocminfo                                  # should list a gfx target
cargo build -p xcfun-gpu --features hip   # should compile
cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '.*'
```
```

**Step G — Extend `xtask/src/bin/check_cubecl_pin.rs`:**

Find the existing `PINNED_CRATES` constant (Phase 2 Plan 02-02; currently lists `cubecl` + `cubecl-cpu`). Extend:

```rust
const PINNED_CRATES: &[&str] = &[
    "cubecl",
    "cubecl-cpu",
    "cubecl-hip",   // NEW Plan 06-03 (D-05 ROCm primary)
    "cubecl-cuda",  // NEW Plan 06-04
    "cubecl-wgpu",  // NEW Plan 06-04
];
const REQUIRED_VERSION: &str = "0.10.0-pre.3";
```

The gate logic walks `cargo metadata` for each pinned crate name and asserts the resolved version equals `REQUIRED_VERSION`. If any cubecl-* crate is absent (e.g., not enabled by any feature in any consumer), the gate may either skip or fail — choose per existing behaviour. For Plan 06-03 the recommended behaviour is "skip if absent, fail if version mismatch" (so users not building with `--features hip --features cuda --features wgpu` don't trip the gate).

**Step H — Verification:**

```bash
cargo build --workspace                                # default features (cpu only)
cargo build -p xcfun-gpu --features hip                # explicit hip feature compile
cargo run -p xtask --bin check-cubecl-pin             # 5-crate gate GREEN
cargo nextest run -p xcfun-gpu --features hip --tests # gpu tests with hip feature compiled in
```

`cargo build -p xcfun-gpu --features hip` MUST exit 0 (assuming cubecl-hip is published — verified per RESEARCH §"Standard Stack" 2026-04-30). The `auto_backend_priority` test under `--features hip` may show `auto_backend() → Backend::Cpu` if no ROCm device is locally available; that's acceptable (probe returned false). The `should_panic` Force test still passes.
  </action>
  <verify>
    <automated>cargo build --workspace && cargo build -p xcfun-gpu --features hip && cargo run -p xtask --bin check-cubecl-pin && cargo nextest run -p xcfun-gpu --features hip --tests</automated>
  </verify>
  <acceptance_criteria>
    - `crates/xcfun-gpu/src/runtime/hip.rs` exists; `grep -c "rocm_available\|HipRuntime" crates/xcfun-gpu/src/runtime/hip.rs` >= 2
    - `grep -c "rocm_available" crates/xcfun-gpu/src/auto_backend.rs` >= 1
    - `grep -c "HipRuntime\|cubecl_hip" crates/xcfun-gpu/src/batch.rs` >= 1
    - `crates/xcfun-gpu/README.md` exists; `grep -c "HSA_OVERRIDE_GFX_VERSION" crates/xcfun-gpu/README.md` >= 1; `grep -c "XCFUN_FORCE_BACKEND\|XCFUN_MIN_BATCH_SIZE" crates/xcfun-gpu/README.md` >= 2; `grep -c "Apple Silicon" crates/xcfun-gpu/README.md` >= 1
    - `grep -c '"cubecl-hip"' xtask/src/bin/check_cubecl_pin.rs` >= 1
    - `grep -c '"cubecl-cuda"' xtask/src/bin/check_cubecl_pin.rs` >= 1
    - `grep -c '"cubecl-wgpu"' xtask/src/bin/check_cubecl_pin.rs` >= 1
    - `cargo build --workspace` exits 0.
    - `cargo build -p xcfun-gpu --features hip` exits 0.
    - `cargo run -p xtask --bin check-cubecl-pin` exits 0.
    - `cargo nextest run -p xcfun-gpu --features hip --tests` exits 0.
  </acceptance_criteria>
  <done>cubecl-hip wired as opt-in feature; HipRuntime probe + OnceLock<HipClient> in `runtime/hip.rs`; Batch<HipRuntime> body shares the generic Batch<R> with HIP-specific eval_vec_host adapter; xtask check-cubecl-pin extended to 5 crates; README documents RDNA-2 + Apple Silicon caveats + env vars.</done>
</task>

<task type="auto">
  <name>Task 2: validation harness --backend rocm dispatch + tier-3 ROCm path</name>
  <files>validation/Cargo.toml, validation/src/main.rs, validation/src/driver.rs</files>
  <read_first>
    - validation/Cargo.toml (Plan 06-02 added `xcfun-gpu` dep with `cpu` feature)
    - validation/src/main.rs (Plan 06-02 added --tier / --reference / --exclude-erf)
    - validation/src/driver.rs (Plan 06-02 added run_tier3 with Cpu arm; bailout for Rocm/Cuda/Wgpu)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-03" CLI lines (lines 65-69, 634-644)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md per-task map row 06-03-01
  </read_first>
  <action>
**Step A — Update `validation/Cargo.toml` to forward feature `hip`:**

```toml
[features]
default = []
hip  = ["xcfun-gpu/hip"]    # Plan 06-03
cuda = ["xcfun-gpu/cuda"]   # Plan 06-04
wgpu = ["xcfun-gpu/wgpu"]   # Plan 06-04
```

**Step B — Wire `--backend rocm` in `validation/src/driver.rs::run_tier3`:**

The Plan 06-02b `run_tier3` (renamed from original 06-02 per revision-1 W-1 split) had `bail!("--backend rocm requires --features hip (Plan 06-03)", ...)` for non-Cpu. Replace the `Backend::Rocm` arm with:

```rust
match backend_e {
    Backend::Cpu => { /* ... existing Cpu arm from Plan 06-02 ... */ }
    #[cfg(feature = "hip")]
    Backend::Rocm => {
        let client = xcfun_gpu::runtime::hip::hip_client().clone();
        let mut batch = Batch::<cubecl_hip::HipRuntime>::open(&fun, client)?;
        let density_pitch = fun.input_length();
        let out_pitch     = fun.output_length().unwrap();
        Batch::<cubecl_hip::HipRuntime>::eval_vec_host(
            &fun, &grid_flat(&grid, density_pitch), density_pitch,
            &mut batch_out, out_pitch, grid.len(),
        )?;
    }
    #[cfg(not(feature = "hip"))]
    Backend::Rocm => anyhow::bail!("--backend rocm requires --features hip (Plan 06-03)"),
    _ => anyhow::bail!("--backend {:?} requires Plan 06-04 (cubecl-cuda / cubecl-wgpu)", backend_e),
}
```

**Step C — Update help text + arg parser in `validation/src/main.rs`:**

If main.rs prints help on `--help`, ensure the `--backend` line lists `cpu | rocm | cuda | wgpu | metal`. Otherwise no main.rs changes needed.

**Step D — Add a documented manual verification command:**

In `06-03-SUMMARY.md` document the two-step ROCm tier-3 sign-off command:

```bash
# Precondition (RDNA-2): export HSA_OVERRIDE_GFX_VERSION=10.3.0
# Strict 1e-13 tier-3 sweep across the 17-known-clean Phase-4 set
# (full 78-functional set requires Plan 06-N1 root-cause closure of inherited Phase-3 forwards).
cargo run -p validation --release --features hip -- \
  --backend rocm --tier 3 --order 3 --jobs 4 \
  --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'
```

Expected: `0 failing` reported. If a tier-3 ROCm CI runner is available, this command runs there. Local sign-off blocked on RESEARCH §R-01 ROCm install.

**Step E — Verification (no-feature compile):**

```bash
cargo build -p validation --release                     # default features → backend rocm bails out
cargo build -p validation --release --features hip      # hip feature → cubecl-hip dep + Backend::Rocm arm compiles
```

Both must exit 0. Tier-3 ROCm sweep is MANUAL VERIFICATION per VALIDATION.md (no local ROCm); document in 06-03-SUMMARY.md.
  </action>
  <verify>
    <automated>cargo build -p validation --release && cargo build -p validation --release --features hip</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c 'hip\s*=\s*\["xcfun-gpu/hip"\]' validation/Cargo.toml` >= 1
    - `grep -c "Backend::Rocm" validation/src/driver.rs` >= 1
    - `grep -c "HipRuntime\|cubecl_hip" validation/src/driver.rs` >= 1
    - `cargo build -p validation --release` exits 0 (default features; --backend rocm bails with helpful message).
    - `cargo build -p validation --release --features hip` exits 0 (hip feature wires cubecl-hip).
    - `cargo run -p validation --release -- --backend rocm` exits non-zero (without `--features hip`) WITH error message mentioning "requires --features hip" or "Plan 06-03".
    - `06-03-SUMMARY.md` (post-execution) documents the manual ROCm tier-3 sign-off command and the precondition `HSA_OVERRIDE_GFX_VERSION=10.3.0`.
  </acceptance_criteria>
  <done>validation harness `--backend rocm` flag dispatches to Batch::<HipRuntime>; default features (no `hip`) build still succeeds with helpful error on `--backend rocm`; `--features hip` build wires cubecl-hip; manual ROCm tier-3 sign-off command documented for VERIFICATION phase.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust ↔ ROCm runtime | cubecl-hip dispatches HIP runtime; loading kernel-object binaries; RDNA-2 quirk requires HSA_OVERRIDE_GFX_VERSION |
| OnceLock<HipClient> ↔ multi-thread | OnceLock is thread-safe; `HipRuntime: 'static + Send + Sync` per cubecl::Runtime contract |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-ROCM-DRIFT | medium | ROCm/HIP intrinsic divergence beyond 1e-13 tolerance vs CPU baseline | tier-3 ROCm gate at strict 1e-13 (manually verified in this plan; CI gate post-Phase-6 sign-off) |
| T-06-CUBECL-DRIFT | high | cubecl-hip not lockstep with cubecl/cubecl-cpu version | xtask check-cubecl-pin extended to 5 crates in Step G; CI gate runs on every PR |
| T-06-FAST-MATH | high | cubecl-hip MAY emit `--use_fast_math` PTX flag silently | Plan 06-04 research task verifies via cubecl-hip source inspection; if fast-math is on by default, Plan 06-03 patches the launch path to disable; for Plan 06-03 grep `--use_fast_math` in cubecl-hip vendored source as part of this task verification |
| T-06-RDNA2-SILENT-FAIL | medium | Silent kernel-load failure on RDNA-2 without HSA env var | Documented in xcfun-gpu/README.md (Step F); Batch::open verifies kernel compiles before any user `launch` call (Plan 06-02 contract) |
| T-06-NO-LOCAL-ROCM | medium | Dev env lacks ROCm; cannot validate tier-3 locally per RESEARCH §R-01 | Plan ships code-only; Phase 6 sign-off depends on a separate `gsd:quick` task or cloud-CI runner; documented in VALIDATION.md "Manual-Only Verifications" |
</threat_model>

<verification>
- All 2 tasks GREEN per their automated commands.
- xtask check-cubecl-pin GREEN with 5 crates.
- xcfun-gpu compiles with `--features hip`.
- validation compiles with and without `--features hip`.
- xcfun-gpu/README.md committed with RDNA-2 + Apple Silicon + env-var documentation.
- Plan 06-03 establishes the precondition for D-02 sign-off (strict 1e-13 across all 78 functionals on `--backend rocm`); actual tier-3 GREEN sign-off blocked on ROCm-install Quick Task per RESEARCH §R-01 / Open Question 1.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 3 advanced: cubecl-hip enabled behind feature `hip` (GPU-03); auto_backend selects Rocm when available (D-07).
- ROADMAP Phase 6 success criterion 4 advanced: Tier-3 parity on ROCm — code path complete; 10k-grid 1e-13 driver wired (GPU-07). Manual verification deferred to Phase 6 sign-off Quick Task.
- D-05 ROCm-primary architectural goal landed in code; Plans 06-04 / 06-05 / 06-06 all proceed atop this skeleton.
- Phase 6 invariant T-06-CUBECL-DRIFT mitigated: 5-crate lockstep gate.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-03-SUMMARY.md` documenting:
- cubecl-hip = =0.10.0-pre.3 wired as opt-in feature `hip`
- HipRuntime probe + OnceLock<HipClient> in runtime/hip.rs
- Batch<HipRuntime>::eval_vec_host body
- xcfun-gpu/README.md: RDNA-2 HSA_OVERRIDE_GFX_VERSION + Apple Silicon caveat + env vars
- xtask check-cubecl-pin extended to 5 crates (cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu)
- validation harness `--backend rocm` dispatch
- Manual verification command for D-02 sign-off (strict 1e-13 ROCm tier-3 across 17-known-clean set)
- RESEARCH §R-01 / Open Question 1: tier-3 ROCm GREEN sign-off blocked on local ROCm install OR cloud-CI runner; flagged for Phase 6 sign-off Quick Task
</output>
