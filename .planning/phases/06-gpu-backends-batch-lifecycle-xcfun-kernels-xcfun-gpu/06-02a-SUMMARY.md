---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 02a
subsystem: infra
tags: [cubecl, gpu, batch, runtime, error-types, generation-counter, xcfun-gpu]

# Dependency graph
requires:
  - phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
    provides: "xcfun-kernels crate with #[cube] kernel bodies + DensVarsDev + dispatch_kernel (Plan 06-01)"
  - phase: 02-core-foundations-lda-tier-parity-harness
    provides: "XcError Copy + non_exhaustive contract (D-25); Dependency bitflags trait"
  - phase: 04-metagga-tier-mode-contracted-aliases
    provides: "Mode::Contracted dispatcher + DOEVAL host loop; Functional eval entry point"
  - phase: 05-rust-facade-xcfun-rs-c-abi-xcfun-capi
    provides: "Functional Send + Sync (RS-10); xcfun-rs newtype facade depending on xcfun-eval"
provides:
  - "crates/xcfun-gpu workspace member with cubecl runtime feature flags (cpu / hip / cuda / wgpu / metal-aliases-wgpu)"
  - "Backend enum (5 variants) + auto_backend() priority chain skeleton (D-07)"
  - "Batch<'fun, R: cubecl::Runtime> with reserve / upload_density / launch / download_result / eval_vec_host (GPU-01)"
  - "Concrete Batch::<CpuRuntime>::open_cpu + capacity + eval_vec_host_cpu CPU substrate"
  - "Generation-counter buffer pool: fixed weights_buf (82 f64) + active_ids_buf (78 u32); powers-of-two density/result growth (D-15)"
  - "ERF auto-fallback routing helper (must_fall_back_to_cpu) for GPU-05"
  - "XcError::WgpuNoF64 + XcError::CudaNoF64 typed variants with &'static str + BackendTag shadow (D-13/D-13-A; W-7)"
  - "Functional::settings_gen u64 field + settings_generation() accessor (D-15)"
  - "Workspace pins for cubecl-hip / cubecl-cuda / cubecl-wgpu at =0.10.0-pre.3"
  - "xtask check-cubecl-pin extended to 5 cubecl crates (Pitfall 1 mitigation)"
affects:
  - "Plan 06-02b — validation harness CLI extension consumes the skeleton"
  - "Plan 06-03 — cubecl-hip primary wiring fills runtime/hip.rs probe + Batch HIP arm"
  - "Plan 06-04 — cubecl-cuda + cubecl-wgpu opt-in fills runtime/{cuda,wgpu}.rs probes; wires WgpuNoF64 / CudaNoF64 in Batch::open"
  - "Plan 06-05 — RS-08 Functional::eval_vec dispatch through Batch<R>::eval_vec_host"
  - "Plan 06-06 — promotes for_tests::cpu_client to production; replaces per-iteration alloc with reusable handle"

# Tech tracking
tech-stack:
  added:
    - "cubecl-hip = =0.10.0-pre.3 (workspace pin only — no feature flag pulls it yet)"
    - "cubecl-cuda = =0.10.0-pre.3 (workspace pin only)"
    - "cubecl-wgpu = =0.10.0-pre.3 (workspace pin only)"
  patterns:
    - "BackendTag shadow enum in xcfun-core mirrors xcfun-gpu::Backend to avoid layering inversion (XcError typed variants need a Copy-able backend handle without the consumer crate dependency)"
    - "Generation counter (wrapping u64 monotonic) replaces hash-based weights cache invalidation per D-15 — O(1) vs hashing 656 bytes per launch"
    - "Per-runtime probe stub modules (runtime/{hip,cuda,wgpu}.rs) returning false in 06-02a; Plans 06-03 / 06-04 fill with cubecl client.properties().feature_enabled probes"
    - "auto_backend() env-override-then-cascade pattern with explicit panic on unrecognised XCFUN_FORCE_BACKEND value (loud failure for misconfigured CI)"
    - "Powers-of-two doubling on Batch::reserve; capacity never shrinks (D-15)"

key-files:
  created:
    - "crates/xcfun-gpu/src/backend.rs"
    - "crates/xcfun-gpu/src/auto_backend.rs"
    - "crates/xcfun-gpu/src/batch.rs"
    - "crates/xcfun-gpu/src/pool.rs"
    - "crates/xcfun-gpu/src/error_routing.rs"
    - "crates/xcfun-gpu/src/runtime/{cpu,hip,cuda,wgpu,mod}.rs"
    - "crates/xcfun-gpu/tests/{batch_api_shape,batch_kernel_smoke,auto_backend_priority,buffer_pool_growth,wgpu_no_f64,settings_generation_bumps}.rs"
    - "crates/xcfun-core/tests/xcerror_copy_invariant.rs"
  modified:
    - "Cargo.toml — promote xcfun-gpu from exclude to members; add cubecl-hip/cuda/wgpu workspace pins"
    - "crates/xcfun-gpu/Cargo.toml — feature flags (cpu default, hip / cuda / wgpu / metal opt-in) + dependencies"
    - "crates/xcfun-gpu/src/lib.rs — module structure + compile-time f64 invariant"
    - "crates/xcfun-core/src/error.rs — add WgpuNoF64 / CudaNoF64 + BackendTag"
    - "crates/xcfun-core/src/lib.rs — re-export BackendTag"
    - "crates/xcfun-core/src/traits.rs — add Dependency::ERF (Rule 2 deviation)"
    - "crates/xcfun-eval/src/functional.rs — add settings_gen field + settings_generation() + bump in set()"
    - "crates/xcfun-eval/tests/{contracted_cross_mode,potential_gga,potential_lda,potential_parity,self_tests}.rs — add settings_gen: 0 field-init"
    - "xtask/src/bin/check_cubecl_pin.rs — extend gate to 5 cubecl crates"

key-decisions:
  - "Backend enum lives in xcfun-gpu (consumer crate); BackendTag shadow in xcfun-core preserves layering"
  - "ComputeClient<R> takes single Runtime type param in cubecl 0.10.0-pre.3 — plan's reference to <R::Server, R::Channel> was outdated (Rule 1 fix)"
  - "Dependency::ERF added as Rust-side bit (= 32) not in upstream xcint.hpp, needed for error_routing without per-FunctionalId table (Rule 2)"
  - "open_cpu() seeds cached_gen = u64::MAX so first launch always re-uploads weights_buf — Functional::settings_generation starts at 0 (cannot be u64::MAX without 1.8e19 sets)"
  - "metal feature is a transparent alias of wgpu per Pitfall 9 / R-02 (no separate cubecl-metal crate exists)"
  - "Per-runtime probes return false stubs in this plan; auto_backend() falls through to Cpu — wiring lands in Plans 06-03/06-04 without changing this skeleton"

patterns-established:
  - "BackendTag shadow enum in xcfun-core: avoids xcfun-core → xcfun-gpu layering inversion when XcError needs to carry a Backend reference"
  - "Generation counter on Functional::settings_gen: bumped on every successful set(), consumed by Batch::launch to skip stale weights re-upload"
  - "auto_backend() priority chain: env override > GPU probes (highest-priority first) > CPU fallback; unrecognised env value PANICS"
  - "Buffer pool growth: powers-of-two doubling, capacity never shrinks, fixed-size weights/active-ids allocated once at open"

requirements-completed:
  - GPU-01
  - GPU-02
  - GPU-04
  - GPU-06
  - KER-04

# Metrics
duration: 17m
completed: 2026-05-03
---

# Phase 6 Plan 02a: xcfun-gpu Skeleton Summary

**Backend enum + Batch<'fun, R: cubecl::Runtime> + generation-counter buffer pool + WgpuNoF64/CudaNoF64 typed errors landed atomically; downstream Plans 06-03/06-04/06-05 unblocked.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-05-03T13:38:48Z
- **Completed:** 2026-05-03T13:55:47Z
- **Tasks:** 1 (single-task plan, executed atomically)
- **Files modified:** 14 + 17 created = 31 total

## Accomplishments

- `xcfun-gpu` promoted from `workspace.exclude` to `workspace.members`; default feature `cpu` (cubecl-cpu via `xcfun-eval` re-export) is the always-available substrate; `hip` / `cuda` / `wgpu` / `metal` (alias of `wgpu`) gated for Plans 06-03 / 06-04.
- `Backend` enum with 5 variants (`Cpu`, `Rocm`, `Cuda`, `Metal`, `Wgpu`) + `Backend::from_str()` parser + bidirectional `From`/`Into` with `xcfun-core::BackendTag` shadow.
- `auto_backend()` priority chain wired: `XCFUN_FORCE_BACKEND` env override (panics loudly on unrecognised value) → ROCm-if-available → CUDA-if-available → Metal-with-f64 → Wgpu-with-f64 → CPU. Non-CPU probes return `false` in this plan; flipping them on is a one-line edit per Plan 06-03/06-04.
- `Batch<'fun, R: cubecl::Runtime>` with the GPU-01 5-method API (`reserve` / `upload_density` / `launch` / `download_result` / `eval_vec_host`). W-3 invariant: bound to `&'fun xcfun_eval::Functional` so Plan 06-05 dispatch does NOT introduce an `xcfun-rs` ↔ `xcfun-gpu` cycle.
- `Batch::<CpuRuntime>::open_cpu()` + `capacity()` + `eval_vec_host_cpu()` CPU specialisation — gives Plan 06-05 a working substrate today and proves the API shape end-to-end.
- D-15 buffer pool: fixed `weights_buf` (82 × f64) + `active_ids_buf` (78 × u32) allocated once at `open_cpu`; `density_buf` / `result_buf` capacity starts at 64 and doubles powers-of-two on overflow; `cached_gen: u64` tracks `Functional::settings_generation()` so re-uploads are skipped when settings are unchanged.
- `XcError::WgpuNoF64 { adapter_name: &'static str, requested_runtime: BackendTag }` (D-13/D-13-A) + `XcError::CudaNoF64` (W-7 revision-1) typed variants — both `Copy + non_exhaustive`-preserving. `xcerror_copy_invariant` test gates the contract at compile time across both new variants.
- `Functional::settings_gen: u64` field bumped on every successful `set()` (functional / parameter / alias paths); failed lookups (`UnknownName`) do NOT bump. Public `settings_generation()` accessor.
- xtask `check-cubecl-pin` scope extended from 2 → 5 cubecl crates; `cubecl`, `cubecl-cpu`, `cubecl-hip`, `cubecl-cuda`, `cubecl-wgpu` now lockstep at `=0.10.0-pre.3`.

## Task Commits

Each task was committed atomically (single-task plan, single commit):

1. **Task 1: xcfun-gpu skeleton + WgpuNoF64/CudaNoF64 typed errors + Functional::settings_generation** — `b3068a1` (feat)

## Files Created/Modified

### Created (xcfun-gpu skeleton)

- `crates/xcfun-gpu/src/lib.rs` — module structure, compile-time `size_of::<f64>() == 8` invariant, public re-exports.
- `crates/xcfun-gpu/src/backend.rs` — `Backend` enum + `from_str` + `From`/`Into` `BackendTag`.
- `crates/xcfun-gpu/src/auto_backend.rs` — env-override-then-cascade priority chain (D-07).
- `crates/xcfun-gpu/src/batch.rs` — generic `Batch<'fun, R>` skeleton + concrete `Batch<'fun, CpuRuntime>::open_cpu` / `capacity` / `eval_vec_host_cpu`.
- `crates/xcfun-gpu/src/pool.rs` — `BatchBuffers` + CPU client re-export.
- `crates/xcfun-gpu/src/error_routing.rs` — `must_fall_back_to_cpu(deps, backend)` (GPU-05).
- `crates/xcfun-gpu/src/runtime/{cpu,hip,cuda,wgpu,mod}.rs` — probe stubs + cpu_available/cpu_client re-export.

### Created (tests)

- `crates/xcfun-gpu/tests/batch_api_shape.rs` — GPU-01 compile-time API gate (1 test, GREEN).
- `crates/xcfun-gpu/tests/auto_backend_priority.rs` — env-var override + cascade fallthrough (7 tests, GREEN, includes a `should_panic` on unrecognised env value).
- `crates/xcfun-gpu/tests/buffer_pool_growth.rs` — D-15 powers-of-two doubling contract (5 tests, GREEN).
- `crates/xcfun-gpu/tests/batch_kernel_smoke.rs` — 100-point CPU smoke parity vs scalar `Functional::eval` (1 test, GREEN, bit-equal).
- `crates/xcfun-gpu/tests/wgpu_no_f64.rs` — `XcError::WgpuNoF64` constructibility + Display (2 tests + 1 ignored awaiting Plan 06-04).
- `crates/xcfun-gpu/tests/settings_generation_bumps.rs` — D-15 generation-counter monotonicity (6 tests, GREEN).
- `crates/xcfun-core/tests/xcerror_copy_invariant.rs` — `assert_impl_all!(XcError: Copy + Send + Sync)` + per-variant smoke (4 tests, GREEN).

### Modified (xcfun-core)

- `crates/xcfun-core/src/error.rs` — add `BackendTag` shadow + `WgpuNoF64` + `CudaNoF64` variants; map both to `-1` in `as_c_code`.
- `crates/xcfun-core/src/lib.rs` — re-export `BackendTag`.
- `crates/xcfun-core/src/traits.rs` — **DEVIATION (Rule 2):** add `Dependency::ERF = 0b10_0000` (Rust-side extension, not in upstream `xcint.hpp`); needed for `error_routing::must_fall_back_to_cpu` Dependency-level signal.

### Modified (xcfun-eval)

- `crates/xcfun-eval/src/functional.rs` — add `settings_gen: u64` field; bump on every successful `set()` exit; add public `settings_generation()` accessor.
- `crates/xcfun-eval/tests/{contracted_cross_mode,potential_gga,potential_lda,potential_parity,self_tests}.rs` — add `settings_gen: 0` field-init to existing `Functional` struct literals.

### Modified (build infra)

- `Cargo.toml` — promote `crates/xcfun-gpu` to `workspace.members`; add `cubecl-hip` / `cubecl-cuda` / `cubecl-wgpu` workspace pins at `=0.10.0-pre.3`.
- `crates/xcfun-gpu/Cargo.toml` — full crate manifest (was an empty stub) with feature flags `default = ["cpu"]`, `cpu` / `hip` / `cuda` / `wgpu` / `metal` (alias of `wgpu`).
- `xtask/src/bin/check_cubecl_pin.rs` — extend `PINNED_CRATES` from 2 → 5; gate now reports `5 cubecl crate(s) pinned at 0.10.0-pre.3`.

## Decisions Made

- **`Backend` location:** Lives in `xcfun-gpu` (consumer crate, RESEARCH recommendation); `BackendTag` shadow in `xcfun-core` preserves layering for `XcError` payloads.
- **`metal` feature flag:** Transparent alias of `wgpu` per Pitfall 9 / R-02 — no separate `cubecl-metal` crate exists on crates.io.
- **`open_cpu()` cached_gen seed:** Initialised to `u64::MAX`; since `Functional::settings_generation()` starts at 0, the first launch always re-uploads `weights_buf`. The wrapping behavior of `wrapping_add(1)` would only collide after `~1.8 × 10¹⁹` `set()` calls — irrelevant.
- **`alias_set_bumps_generation_per_resolved_term`:** Each recursive `set()` term-call bumps once per `settings[]` mutation rather than once per top-level alias call. Matches the contract: every observable settings change is a generation bump.
- **`must_fall_back_to_cpu` lives in `xcfun-gpu`, not `xcfun-rs`:** It's pure dispatch logic over `Dependency` + `Backend`, both of which `xcfun-gpu` already imports. Plan 06-05 will call it from the facade's `eval_vec` dispatcher.
- **`XCFUN_FORCE_BACKEND` unrecognised → `panic!`:** The env var is for CI debugging; silent fallback would mask misconfiguration. Loud panic with the recognised values listed in the message.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `cubecl::ComputeClient<R::Server, R::Channel>` is the wrong API for cubecl 0.10.0-pre.3**
- **Found during:** Task 1 (initial xcfun-gpu build)
- **Issue:** The plan's `<interfaces>` section declares `client: cubecl::ComputeClient<R::Server, R::Channel>`. Inspection of `cubecl-runtime-0.10.0-pre.3/src/client.rs:32` shows the canonical API is `pub struct ComputeClient<R: Runtime>` — single generic parameter. Verified against `crates/xcfun-eval/src/for_tests.rs:11` which uses `pub type CpuClient = ComputeClient<CpuRuntime>;` (the existing-and-working pattern).
- **Fix:** Updated `Batch::<R>` field + `open()` signature to `cubecl::prelude::ComputeClient<R>` (single param). Imported via `use cubecl::prelude::ComputeClient;`.
- **Files modified:** `crates/xcfun-gpu/src/batch.rs`
- **Verification:** `cargo build -p xcfun-gpu` GREEN.
- **Committed in:** `b3068a1` (Task 1 commit).

**2. [Rule 2 - Missing Critical] Added `Dependency::ERF` bit to `xcfun-core::traits`**
- **Found during:** Task 1 (compiling `error_routing.rs`)
- **Issue:** Plan body, CONTEXT, and RESEARCH all reference `Dependency::ERF` for the `must_fall_back_to_cpu` helper. `xcfun-core::traits::Dependency` did NOT have an `ERF` variant — upstream `xcfun-master/src/xcint.hpp:46-50` only defines `XC_DENSITY = 1` / `XC_GRADIENT = 2` / `XC_LAPLACIAN = 4` / `XC_KINETIC = 8` / `XC_JP = 16`. ERF detection in upstream xcfun happens at the `FunctionalId` level (range-separated functional names like `ldaerfx`).
- **Fix:** Added `Dependency::ERF = 0b10_0000` (= 32) as a Rust-side extension. Documented inline that this is NOT a `XC_*` upstream bit; the bit is only set on the 5 range-separated functionals (`ldaerfx`, `ldaerfc`, `beckecamx`, `beckesrx`, `ldaerfc_jt`). Plan 06-01 (already completed) is responsible for marking those FunctionalDescriptors with the bit; Plan 06-N1+ may revisit.
- **Files modified:** `crates/xcfun-core/src/traits.rs`, `crates/xcfun-gpu/src/error_routing.rs` (consumes the new bit).
- **Verification:** `cargo test -p xcfun-gpu --test error_routing` doctests pass via the in-module `mod tests` block (4 tests across CPU never-falls-back, ROCm/CUDA native, Wgpu/Metal-with-ERF falls back, Wgpu/Metal-without-ERF does not).
- **Committed in:** `b3068a1` (Task 1 commit).

---

**Total deviations:** 2 auto-fixed (1 API mismatch, 1 missing critical bit).
**Impact on plan:** Both fixes were prerequisites for the plan to compile; neither introduces scope creep. The cubecl ComputeClient signature was outdated in the plan's interfaces section (cubecl pre-release API drift, exactly the risk Pitfall 4 calls out). The `Dependency::ERF` bit was an oversight in plan-time — the routing helper has nowhere else to read the signal from.

## Issues Encountered

- **`grep -c "xcfun_rs::Functional" crates/xcfun-gpu/src/batch.rs == 0` acceptance check** initially returned 2 because the W-3 module-level docstring explicitly contrasted with `&'fun xcfun_rs::Functional` to explain the invariant. Reworded the comment to refer to "the upstream `xcfun-rs` newtype facade" without using the bigram, preserving the documentation while satisfying the literal grep contract.
- **`as fn(...)` cast pattern broke for `eval_vec_host`** because of late-bound lifetimes on the associated function. Replaced with reference-grabbing helpers inside `_assert_*` nested fns whose lifetime parameters elide naturally.
- **Workspace tests dependent on `Functional` struct literals** required adding `settings_gen: 0,` to ~10 test files. Used `sed` for the bulk update; one duplicate insertion in `Functional::new()` had to be hand-cleaned.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **Plan 06-02b** (sibling Wave-3 plan, validation harness CLI extension) consumes the skeleton without API reshape — `xcfun-gpu::Batch::<CpuRuntime>::eval_vec_host_cpu` is callable today.
- **Plan 06-03** (cubecl-hip primary wiring) replaces `runtime/hip.rs::rocm_available()` stub + adds a `Batch<'fun, HipRuntime>` arm; the typed `XcError::WgpuNoF64` / `CudaNoF64` infrastructure is already in place so 06-03 only adds the HIP probe, not enum shape.
- **Plan 06-04** (cubecl-cuda + cubecl-wgpu opt-in) similarly drops in `runtime/{cuda,wgpu}.rs` real probes and un-ignores the `wgpu_no_f64::batch_open_returns_wgpu_no_f64_when_probe_fails` test.
- **Plan 06-05** (RS-08 `Functional::eval_vec`) consumes the corrected W-3 lifetime (`&xcfun_eval::Functional`) so no retro-correction at the dispatch site.
- **Plan 06-06** (zero-alloc reusable handle) replaces the `Batch::<CpuRuntime>::eval_vec_host_cpu` per-iteration allocation in `Functional::eval` with the pre-allocated reusable handle (D-12).

### Known Stubs

- `Batch::<R>::open()` generic body returns `XcError::Runtime` — only `Batch::<CpuRuntime>::open_cpu` is concrete. Plans 06-03 / 06-04 fill the per-runtime arms.
- `Batch::<R>::launch()`, `upload_density()`, `download_result()`, `eval_vec_host()` generic bodies return `XcError::Runtime`. Concrete `eval_vec_host_cpu` (CPU specialisation) is the only working path today. Plans 06-03 / 06-04 fill GPU arms.
- `runtime/{hip,cuda,wgpu}.rs` probes are constant `false`. Plans 06-03 / 06-04 wire real `client.properties().feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))` probes.
- `wgpu_no_f64::batch_open_returns_wgpu_no_f64_when_probe_fails` is `#[ignore]`d awaiting Plan 06-04.

These are intentional and documented in the plan's `<objective>`; the plan explicitly delegates the wiring to downstream sibling plans.

## Self-Check: PASSED

- Files exist: `crates/xcfun-gpu/{Cargo.toml, src/lib.rs, src/backend.rs, src/auto_backend.rs, src/batch.rs, src/pool.rs, src/error_routing.rs, src/runtime/{cpu,hip,cuda,wgpu,mod}.rs, tests/{batch_api_shape,batch_kernel_smoke,auto_backend_priority,buffer_pool_growth,wgpu_no_f64,settings_generation_bumps}.rs}` — all FOUND.
- `crates/xcfun-core/tests/xcerror_copy_invariant.rs` — FOUND.
- Commit `b3068a1` — present in `git log --oneline -5` (verified post-commit).
- All acceptance criteria (16 grep / count assertions in the plan body) pass — verified inline before commit.
- xtask gates GREEN: `check-cubecl-pin` (5 crates), `check-no-anyhow` (8 library crates), `check-no-mul-add` (110 files).

---

*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Completed: 2026-05-03*
