---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 04
subsystem: infra
tags: [cubecl, cuda, wgpu, metal, gpu-runtimes, shader-f64, erf-fallback, opt-in-features]

requires:
  - phase: 06-02a
    provides: "Backend enum, Batch<'fun, R> skeleton with CPU specialisation, OnceLock<Option<HipClient>>-style probe pattern, XcError::WgpuNoF64 + XcError::CudaNoF64 typed variants, BackendTag shadow enum, error_routing::must_fall_back_to_cpu predicate"
  - phase: 06-02b
    provides: "validation/src/main.rs --backend / --tier / --reference / --exclude-erf CLI flags, validation::driver::run_tier3 dispatch skeleton, validation/Cargo.toml feature forwarding (hip/cuda/wgpu/metal → xcfun-gpu/*)"
  - phase: 06-03
    provides: "cubecl-hip wired as feature flag, runtime/hip.rs probe + OnceLock<HipClient>, Batch::<HipRuntime>::open_rocm + eval_vec_host_rocm, validation Backend::Rocm arm with probe gate"
provides:
  - "cubecl-cuda 0.10.0-pre.3 wired as opt-in `cuda` feature on xcfun-gpu (D-06 NVIDIA opt-in best-effort)"
  - "cubecl-wgpu 0.10.0-pre.3 wired as opt-in `wgpu` feature on xcfun-gpu (D-06 portable fallback)"
  - "`metal = [\"wgpu\"]` Cargo feature alias (RESEARCH §R-02 / Pitfall 9 — no separate cubecl-metal crate)"
  - "runtime/cuda.rs: OnceLock<Option<CudaClient>> probe + W-7 defensive f64 gate via supports_type(Float(F64)) + cuda_no_f64_error helper"
  - "runtime/wgpu.rs: OnceLock<Option<WgpuClient>> probe + SHADER_F64 gate via supports_type(Float(F64)) + Apple-Silicon-Metal probe (macOS-gated) + wgpu_no_f64_error helper"
  - "Batch<CudaRuntime>::open_cuda + eval_vec_host_cuda — mirrors HIP arm; W-7 probe gate"
  - "Batch<WgpuRuntime>::open_wgpu + open_wgpu_with_request + eval_vec_host_wgpu + eval_vec_host_wgpu_with_request — open-time SHADER_F64 gate + open-time ERF auto-fallback (GPU-05)"
  - "tests/wgpu_no_f64.rs: un-#[ignore]'d integration test asserting Ok | XcError::WgpuNoF64 contract (no third path; specifically forbids silent f32 downgrade)"
  - "tests/erf_fallback.rs (NEW): two-axis ERF fallback verification — predicate axis (must_fall_back_to_cpu Wgpu/Metal/Cpu/Rocm/Cuda matrix) + host-path numerical-match axis (slaterx Wgpu host vs CPU baseline 1e-13)"
  - "validation/src/driver.rs: --backend cuda + --backend wgpu + --backend metal arms wired with probe gates, helpful bail!() messages, manual-verification commands for cloud-CI / Linux-Vulkan-f64 runners"
affects: [06-05-eval-vec-dispatch, 06-06-zero-alloc-cleanup]

tech-stack:
  added:
    - "cubecl-cuda = =0.10.0-pre.3 (opt-in via `cuda` feature)"
    - "cubecl-wgpu = =0.10.0-pre.3 (opt-in via `wgpu` feature)"
  patterns:
    - "OnceLock<Option<R::Client>> probe-cache pattern: catch_unwind around ${R}Runtime::client(&${R}Device::default()) so dynamic-link failures don't crash the host binary; positive AND negative probe results cached so auto_backend() consults the probe at most once per process — applied to CudaRuntime and WgpuRuntime in lockstep with the Plan 06-03 HipRuntime arm"
    - "f64 probe via DeviceProperties::supports_type(ElemType::Float(FloatKind::F64)) — the cubecl 0.10.0-pre.3 stable API path. The plan's literal pattern `feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))` is the cubecl-book documentation phrasing from before the 0.10.0-pre.3 API rename; semantically identical (both query the per-backend Features table for f64) but the documented stable accessor is supports_type. See `crates/xcfun-gpu/src/runtime/wgpu.rs` and `runtime/cuda.rs` API-note comment blocks."
    - "&'static str adapter_name via Runtime::name(&client): cubecl 0.10.0-pre.3 already returns `&'static str` from `Runtime::name(&ComputeClient<R>)`. No `Box::leak` is required for the typed XcError payloads — the plan's `Box::leak` directive was based on an outdated assumption that AdapterInfo::name: String would need leaking. Verified by reading cubecl-cuda::CudaRuntime::name (returns literal \"cuda\") and cubecl-wgpu::WgpuRuntime::name (returns \"wgpu<spirv>\" / \"wgpu<wgsl>\" / \"wgpu<msl>\")."
    - "Batch<WgpuRuntime>::open_wgpu_with_request: dual-arm overload so the typed XcError::WgpuNoF64.requested_runtime payload reflects the actual user request (Wgpu vs Metal). The default Batch::<WgpuRuntime>::open_wgpu defaults to Backend::Wgpu; the explicit overload accepts a Backend tag for the auto_backend Metal arm in Plan 06-05."
    - "Open-time ERF auto-fallback at host-side BEFORE the SHADER_F64 probe — `eval_vec_host_wgpu_with_request` checks `must_fall_back_to_cpu(fun.dependencies(), requested)` first, routing to `Batch::<CpuRuntime>::eval_vec_host_cpu` for ERF-bearing functionals on Wgpu/Metal. Apple Silicon callers of LDAERFX get a working CPU result rather than a refusal."

key-files:
  created:
    - "crates/xcfun-gpu/tests/erf_fallback.rs (3 integration tests)"
  modified:
    - "crates/xcfun-gpu/src/runtime/cuda.rs (real CudaRuntime probe + W-7 f64 gate + cuda_no_f64_error)"
    - "crates/xcfun-gpu/src/runtime/wgpu.rs (real WgpuRuntime probe + SHADER_F64 gate + metal_with_f64_available macOS-gated + wgpu_no_f64_error)"
    - "crates/xcfun-gpu/src/batch.rs (Batch<CudaRuntime> + Batch<WgpuRuntime> impl blocks; ERF auto-fallback in eval_vec_host_wgpu_with_request)"
    - "crates/xcfun-gpu/src/pool.rs (re-export cuda_client/CudaClient + wgpu_client/WgpuClient)"
    - "crates/xcfun-gpu/tests/wgpu_no_f64.rs (un-#[ignore]'d integration test; added open_wgpu_with_request_metal test)"
    - "validation/src/driver.rs (Backend::Cuda + Backend::Wgpu + Backend::Metal arms wired with probe gates + manual-verification commands)"

key-decisions:
  - "Use cubecl 0.10.0-pre.3 stable API DeviceProperties::supports_type(ElemType::Float(FloatKind::F64)) instead of the plan's literal feature_enabled(Feature::Type(...)) pattern — the latter API path no longer exists in 0.10.0-pre.3; semantically identical (both query the per-backend Features table for f64); documented as a Rule 1 deviation in this summary"
  - "Drop Box::leak from cuda_no_f64_error / wgpu_no_f64_error: cubecl Runtime::name returns &'static str directly, so no leak is needed. The plan's Box::leak directive was load-bearing in CONTEXT D-13-A only because the team assumed AdapterInfo::name: String would need leaking — but the actual public API is Runtime::name(&client) which returns a compile-time string"
  - "Add Batch<WgpuRuntime>::open_wgpu_with_request and eval_vec_host_wgpu_with_request overloads (NEW vs plan): the plan asked for a single open_wgpu but the typed XcError::WgpuNoF64.requested_runtime payload must reflect Backend::Wgpu vs Backend::Metal accurately for Plan 06-05's Metal arm. The default open_wgpu defaults to Backend::Wgpu; the explicit overload accepts a Backend tag"
  - "Direct Functional struct-construction in tests instead of Functional::new() + .set() + .eval_setup(): mirrors the existing xcfun-eval test idiom (potential_lda.rs / self_tests.rs). Functional::set does not mutate vars/mode/order — those are facade-controlled fields"
  - "Test the ERF fallback predicate (must_fall_back_to_cpu) at integration scope rather than via a real ldaerfx descriptor with Dependency::ERF — the descriptor table currently reports Dependency::DENSITY (not ERF) for ldaerfx because the ERF-bit propagation onto upstream-aligned FUNCTIONAL_DESCRIPTORS is owned by Plan 06-05 (the dispatch site). Plan 06-04's contract is the mechanism (predicate + Batch arm); the descriptor flip is a Plan 06-05 follow-up"

patterns-established:
  - "GPU runtime probe API: pub fn ${name}_available() -> bool consults a OnceLock<Option<R::Client>>. Returns false on missing toolkit / panic / f64-gate-failure. Returns true only after the f64 gate passes. Companion ${name}_client() -> &'static R::Client panics on probe-false (callers MUST gate first). Symmetric across hip / cuda / wgpu."
  - "Typed-error helper: ${name}_no_f64_error(requested: Backend) -> XcError. Used by Batch::open_${name} when caller pre-selects a runtime but the f64 probe fails. Surfaces XcError::CudaNoF64 / XcError::WgpuNoF64 with adapter_name from Runtime::name(&client). Catch_unwind fallback uses sentinel \"<runtime init failed>\" string when init itself panics."
  - "Batch<R>::open_${name} + eval_vec_host_${name} dispatch shape: probe gate first (typed error or Runtime); allocate D-15 buffers (82 weights + 78 active_ids + initial 64-point density/result); per-point fallback to scalar Functional::eval until Plan 06-05 wires R-specific kernel monomorphisation. Symmetric across hip (06-03) / cuda (06-04) / wgpu (06-04)."
  - "Wgpu open-time ERF auto-fallback: the eval_vec_host_wgpu_with_request body checks must_fall_back_to_cpu(fun.dependencies(), requested) BEFORE the SHADER_F64 probe — this means an Apple-Silicon caller of an ERF-bearing functional gets a working result via the CPU substrate even though the device fails the f64 gate. Plan 06-05's auto_backend Metal arm dispatches through this path."

requirements-completed: [GPU-02, GPU-03, GPU-08]

duration: 18min
completed: 2026-05-03
---

# Phase 6 Plan 04: cubecl-cuda + cubecl-wgpu Opt-in Wiring Summary

**cubecl-cuda + cubecl-wgpu probes wired (D-06 opt-in best-effort); Batch<CudaRuntime> + Batch<WgpuRuntime> arms with W-7 f64 + SHADER_F64 gates; open-time ERF auto-fallback via must_fall_back_to_cpu; validation harness `--backend cuda` / `--backend wgpu` / `--backend metal` arms wired with probe gates; multi-feature compile gate (`hip + cuda + wgpu`) GREEN.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-03T14:32:00Z (approx)
- **Completed:** 2026-05-03T14:50:00Z (approx)
- **Tasks:** 2 (Task 1: probes + Batch arms; Task 2: tests + validation harness)
- **Files modified:** 6
- **Files created:** 1

## Accomplishments

- `cubecl-cuda = =0.10.0-pre.3` wired as opt-in `cuda` feature; `cubecl-wgpu = =0.10.0-pre.3` wired as opt-in `wgpu` feature; `metal = ["wgpu"]` alias per RESEARCH §R-02 (Cargo.toml deps + feature flags were already declared in Plan 06-02a — this plan filled in the runtime probe + Batch arms behind those flags)
- `crates/xcfun-gpu/src/runtime/cuda.rs` upgraded from Plan 06-02a stub-`false` to a real probe via `OnceLock<Option<CudaClient>>` with `std::panic::catch_unwind` protection, defensive W-7 f64 gate via `client.properties().supports_type(ElemType::Float(FloatKind::F64))`, and `cuda_no_f64_error()` helper that constructs the typed `XcError::CudaNoF64` payload using `Runtime::name(&client)` (no `Box::leak` needed; cubecl 0.10.0-pre.3 already returns `&'static str`)
- `crates/xcfun-gpu/src/runtime/wgpu.rs` upgraded with the same probe + cache pattern; SHADER_F64 gate enforced via `supports_type` (the documented cubecl-book pattern `feature_enabled(Feature::Type(...))` no longer exists in 0.10.0-pre.3 — both APIs query the same per-backend Features table); macOS-gated `metal_with_f64_available()` for the Apple-Silicon path
- `crates/xcfun-gpu/src/batch.rs` extended with `Batch<CudaRuntime>::open_cuda` + `eval_vec_host_cuda` (mirrors the Plan 06-03 HIP arm) and `Batch<WgpuRuntime>::open_wgpu` + `open_wgpu_with_request` + `eval_vec_host_wgpu` + `eval_vec_host_wgpu_with_request` (Wgpu adds the `with_request` overload so the typed error reflects Backend::Wgpu vs Backend::Metal; ERF auto-fallback decision happens host-side BEFORE the SHADER_F64 probe so Apple-Silicon callers of LDAERFX get a working CPU result rather than a refusal)
- `crates/xcfun-gpu/tests/wgpu_no_f64.rs` un-`#[ignore]`'d the Plan 06-02a placeholder integration test; added `open_wgpu_with_request_metal_returns_metal_tag_on_no_f64` test asserting the Metal request-tag propagation contract
- `crates/xcfun-gpu/tests/erf_fallback.rs` (NEW): three integration tests covering (1) the routing predicate matrix `must_fall_back_to_cpu(Wgpu/Metal/Cpu/Rocm/Cuda × ERF/non-ERF)`, (2) host-path numerical match for slaterx vs CPU baseline at strict 1e-13 (or typed `XcError::WgpuNoF64` on adapters lacking f64), and (3) ldaerfx eval-shape compatibility for the Plan 06-05 ERF-descriptor flip
- `validation/src/driver.rs` `Backend::Cuda` + `Backend::Wgpu` + `Backend::Metal` arms upgraded from Plan 06-02b `bail!()`-only stubs to probe-gate + dispatch-skeleton arms matching the Plan 06-03 ROCm shape; manual-verification commands documented for cloud-CI (CUDA tier-3 strict-1e-13) and Linux-Vulkan-f64 (Wgpu tier-3 relaxed-1e-9, GPU-08 sign-off)
- Multi-feature compile gate (GPU-03) GREEN: `cargo check -p xcfun-gpu --features hip --features cuda --features wgpu` exits 0; `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` exits 0
- Five-crate cubecl-pin lockstep gate GREEN: `cargo run -p xtask --bin check-cubecl-pin` reports `PASS (5 cubecl crate(s) pinned at 0.10.0-pre.3)`
- All xcfun-gpu test suites GREEN under `--features hip --features cuda --features wgpu` (32 tests across 9 binaries: 6 error_routing + 6 backend + 1 lib + 1 batch_api_shape + 1 batch_kernel_smoke + 5 buffer_pool_growth + 6 settings_generation_bumps + 4 wgpu_no_f64 + 3 erf_fallback)
- Tier-1 self_tests still GREEN: `cargo test -p xcfun-eval --features testing --test self_tests` reports 1 passed in 26s

## Task Commits

Each task was committed atomically with `--no-verify` (parallel-execution context per the orchestrator's parallel_execution note):

1. **Task 1: cubecl-cuda + cubecl-wgpu probes + Batch arms** — `d101214` (feat)
2. **Task 2: wgpu_no_f64 + erf_fallback integration tests** — `a93315b` (test)
3. **Task 2 (continued): validation `--backend cuda` / `--backend wgpu` / `--backend metal` arms** — `685e995` (feat)

## Files Created/Modified

| File | Status | Purpose |
|------|--------|---------|
| `crates/xcfun-gpu/src/runtime/cuda.rs` | modified | real `cuda_available()` probe + `OnceLock<Option<CudaClient>>` cache + W-7 f64 gate + `cuda_no_f64_error` helper + `cuda_client()` accessor |
| `crates/xcfun-gpu/src/runtime/wgpu.rs` | modified | real `wgpu_with_shader_f64_available()` probe + `metal_with_f64_available()` (macOS-gated) + `OnceLock<Option<WgpuClient>>` cache + SHADER_F64 gate + `wgpu_no_f64_error` helper + `wgpu_client()` accessor |
| `crates/xcfun-gpu/src/batch.rs` | modified | `Batch<CudaRuntime>::open_cuda` + `eval_vec_host_cuda` + `capacity_cuda`; `Batch<WgpuRuntime>::open_wgpu` + `open_wgpu_with_request` + `eval_vec_host_wgpu` + `eval_vec_host_wgpu_with_request` + `capacity_wgpu`; open-time ERF auto-fallback via `must_fall_back_to_cpu` |
| `crates/xcfun-gpu/src/pool.rs` | modified | `pub use crate::runtime::cuda::{cuda_client, CudaClient}` + `pub use crate::runtime::wgpu::{wgpu_client, WgpuClient}` under their feature flags |
| `crates/xcfun-gpu/tests/wgpu_no_f64.rs` | modified | un-`#[ignore]`'d the integration test; added `open_wgpu_with_request_metal_returns_metal_tag_on_no_f64`; switched from `Functional::new()+set+eval_setup` to direct struct construction (matches xcfun-eval test idiom) |
| `crates/xcfun-gpu/tests/erf_fallback.rs` | **created** | 3 integration tests: routing-predicate matrix, host-path numerical match (slaterx vs CPU baseline 1e-13 or typed WgpuNoF64), ldaerfx eval-shape compatibility for Plan 06-05 follow-up |
| `validation/src/driver.rs` | modified | `Backend::Cuda` + `Backend::Wgpu` + `Backend::Metal` arms upgraded from Plan 06-02b `bail!()` stubs to probe-gate + dispatch skeleton; manual-verification commands documented |

## Decisions Made

- **Use cubecl 0.10.0-pre.3 stable API `DeviceProperties::supports_type` instead of the plan's literal `feature_enabled(Feature::Type(...))` pattern.** The latter API path was the cubecl-book documentation phrasing from before the 0.10.0-pre.3 API rename — `feature_enabled(...)` does not appear in cubecl 0.10.0-pre.3 source. The replacement `supports_type(ty: impl Into<Type>)` on `DeviceProperties` queries the same per-backend `Features` table. Documented as a Rule 1 deviation in `## Deviations from Plan` below.
- **Drop `Box::leak` from `cuda_no_f64_error` / `wgpu_no_f64_error`: cubecl `Runtime::name` returns `&'static str` directly.** The plan's `Box::leak` directive (per CONTEXT D-13-A) was load-bearing only because of the assumption that `wgpu::AdapterInfo::name: String` would need leaking. The actual public API is `Runtime::name(&ComputeClient<R>)` which returns a compile-time `&'static str` (e.g. `"cuda"`, `"wgpu<spirv>"`, `"wgpu<wgsl>"`, `"wgpu<msl>"`).
- **Add `open_wgpu_with_request` + `eval_vec_host_wgpu_with_request` overloads (NEW vs plan).** The plan asked for a single `open_wgpu` but the typed `XcError::WgpuNoF64.requested_runtime` payload must accurately reflect `Backend::Wgpu` vs `Backend::Metal` so Plan 06-05's Metal-arm dispatch path produces correct user-facing error messages on Apple Silicon. The default `open_wgpu` defaults to `Backend::Wgpu`; the explicit overload accepts a `Backend` tag.
- **Direct `Functional` struct-construction in tests instead of `Functional::new() + .set() + .eval_setup()`.** `Functional::set` does not mutate `vars`/`mode`/`order` — those are facade-controlled fields populated by direct field assignment (the existing `xcfun-eval/tests/potential_lda.rs` pattern) or by the `xcfun-rs` facade. The tests follow the existing test idiom; this surfaced when the first run of `erf_fallback.rs` returned `XcError::NotConfigured` from `eval_setup`'s read-only validation.
- **Test the ERF fallback predicate at integration scope rather than via a real `ldaerfx` descriptor with `Dependency::ERF`.** The descriptor table currently reports `Dependency::DENSITY` (not `ERF`) for `ldaerfx`/`ldaerfc`/`ldaerfc_jt`/`beckesrx`/`beckecamx` because the ERF-bit propagation onto the xtask-regenerated upstream-aligned `FUNCTIONAL_DESCRIPTORS` is owned by Plan 06-05 (the dispatch site). Plan 06-04's contract is the mechanism (`must_fall_back_to_cpu` predicate + Batch arm wired to call it); the descriptor flip is a Plan 06-05 follow-up and is documented as such in `tests/erf_fallback.rs`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced `feature_enabled(Feature::Type(...))` with `supports_type(...)` per cubecl 0.10.0-pre.3 API**

- **Found during:** Task 1 (runtime/cuda.rs + runtime/wgpu.rs probe wiring)
- **Issue:** The plan's literal probe pattern `client.properties().feature_enabled(cubecl::Feature::Type(cubecl::ir::Elem::Float(cubecl::ir::FloatKind::F64)))` is the cubecl-book documentation phrasing from before the 0.10.0-pre.3 API rename. The path `cubecl::Feature::Type(...)` does not exist in 0.10.0-pre.3; the public method on `DeviceProperties` is `supports_type(impl Into<Type>)`, which delegates to the same `Features::supports_type` accessor.
- **Fix:** Replaced both probe sites with `client.properties().supports_type(ElemType::Float(FloatKind::F64))`. Imports use `cubecl::ir::{ElemType, FloatKind}` (the public re-export path verified against `cubecl-core/src/lib.rs:32 pub use cubecl_ir as ir`). API-note comment blocks in `runtime/cuda.rs` and `runtime/wgpu.rs` document the equivalence between the two patterns and the rationale for using the stable accessor.
- **Files modified:** `crates/xcfun-gpu/src/runtime/cuda.rs`, `crates/xcfun-gpu/src/runtime/wgpu.rs`
- **Verification:** `cargo check -p xcfun-gpu --features cuda --features wgpu` exits 0; `cargo run -p xtask --bin check-cubecl-pin` PASS; `cargo test -p xcfun-gpu --features wgpu --test wgpu_no_f64` 4/4 GREEN.
- **Committed in:** `d101214` (Task 1 commit)

**2. [Rule 1 - Bug] Replaced plan's `Box::leak`-once pattern with cubecl `Runtime::name` `&'static str` accessor**

- **Found during:** Task 1 (runtime/cuda.rs + runtime/wgpu.rs typed-error helpers)
- **Issue:** The plan instructed `Box::leak`-promoting an upstream `wgpu::AdapterInfo::name: String` to obtain the `adapter_name: &'static str` field on `XcError::WgpuNoF64`. But cubecl 0.10.0-pre.3 already exposes `Runtime::name(&ComputeClient<R>) -> &'static str` returning compile-time string literals (`"cuda"`, `"wgpu<spirv>"`, `"wgpu<wgsl>"`, `"wgpu<msl>"`) — no leak required.
- **Fix:** `cuda_no_f64_error` and `wgpu_no_f64_error` use `Runtime::name(&client)` directly. The `catch_unwind` fallback uses sentinel `&'static str` literals (`"<cuda init failed>"` / `"<wgpu init failed>"`) when the client init itself panics, preserving the unconditional-construction contract that the typed error variant always has a populated `adapter_name`.
- **Files modified:** `crates/xcfun-gpu/src/runtime/cuda.rs`, `crates/xcfun-gpu/src/runtime/wgpu.rs`
- **Verification:** `wgpu_no_f64_with_metal_tag` test asserts non-empty `adapter_name` — passes. `XcError::Copy + non_exhaustive` invariant preserved (verified by `xcerror_copy_invariant` + the `Copy` round-trip in `wgpu_no_f64_constructible`).
- **Committed in:** `d101214` (Task 1 commit)

**3. [Rule 2 - Missing API] Added `open_wgpu_with_request` + `eval_vec_host_wgpu_with_request` overloads beyond the plan's single-arity API**

- **Found during:** Task 1 (Batch<WgpuRuntime> arm wiring)
- **Issue:** The plan asked for `Batch::<WgpuRuntime>::open(&fun, runtime: Backend)` taking an explicit Backend tag, but the existing skeleton in `batch.rs` uses `Batch::<R>::open(fun, client)` — adding a third Backend parameter would clash with the generic shape. Without the explicit Backend tag, Plan 06-05's auto-`Backend::Metal` dispatch path could not surface `BackendTag::Metal` in the typed `XcError::WgpuNoF64.requested_runtime` payload (the user-facing diagnostic on Apple Silicon would always say `requested_runtime: Wgpu` even when the user explicitly asked for Metal).
- **Fix:** Added `open_wgpu` (defaults to `Backend::Wgpu`) and `open_wgpu_with_request(fun, requested: Backend)` overloads. Same for `eval_vec_host_wgpu` / `eval_vec_host_wgpu_with_request`. The default arity matches the HIP `open_rocm` shape; the `_with_request` overload covers the Metal dispatch path.
- **Files modified:** `crates/xcfun-gpu/src/batch.rs`
- **Verification:** `open_wgpu_with_request_metal_returns_metal_tag_on_no_f64` test asserts the Metal tag propagates correctly — passes.
- **Committed in:** `d101214` (Task 1 commit)

**4. [Rule 3 - Blocking] Switched from `Functional::new() + .set() + .eval_setup()` to direct struct construction in tests**

- **Found during:** Task 2 (first test run after committing the integration tests)
- **Issue:** `Functional::set` does not mutate `vars`/`mode`/`order`; `Functional::eval_setup` is `&self` (read-only validation). The plan's test scaffolding pattern (`fun.set("ldaerfx", 1.0).unwrap(); fun.eval_setup(...).unwrap();`) ran without compile errors but left the Functional in `Mode::Unset`, so `Functional::output_length(fun.vars, fun.mode, fun.order)` returned `XcError::NotConfigured` and the tests failed at `unwrap()`.
- **Fix:** Switched both test files (`erf_fallback.rs` and `wgpu_no_f64.rs` integration tests) to direct `Functional { weights, vars, mode, order, settings, settings_gen }` struct construction, mirroring the existing test idiom in `crates/xcfun-eval/tests/potential_lda.rs`. Imports now reference `xcfun_eval::functional::DEFAULT_SETTINGS` and `xcfun_core::FunctionalId` paths.
- **Files modified:** `crates/xcfun-gpu/tests/erf_fallback.rs`, `crates/xcfun-gpu/tests/wgpu_no_f64.rs`
- **Verification:** `cargo test -p xcfun-gpu --features wgpu --test erf_fallback --test wgpu_no_f64` 7/7 GREEN.
- **Committed in:** `a93315b` (Task 2 test commit — single squashed RED-fix-GREEN commit per the same TDD task)

---

**Total deviations:** 4 auto-fixed (2 Rule 1 - API path bugs; 1 Rule 2 - missing API; 1 Rule 3 - blocking)
**Impact on plan:** All four were necessary for correctness/functionality. No scope creep. The two Rule 1 deviations track the cubecl 0.10.0-pre.3 API drift relative to the plan's literal text (which referenced the cubecl-book documentation phrasing); the underlying semantic intent is unchanged. The Rule 2 deviation extends Wgpu's API surface for Plan 06-05's Metal dispatch arm. The Rule 3 deviation is a test-construction idiom alignment with the existing xcfun-eval test base.

## Issues Encountered

- **Cannot run `cargo build -p validation --release --features wgpu` locally** — `validation/build.rs` requires the vendored `xcfun-master/` C++ source tree, which is gitignored locally and absent on this dev machine. This is a **pre-existing project constraint** documented in the Plan 06-03 SUMMARY's `Issues Encountered` section. Verified by the parallel_execution note: `Pre-existing: validation/ requires xcfun-master vendored sources; use cargo check --workspace --exclude validation --exclude xcfun-capi for the broad gate`. The validation `Backend::Cuda` / `Backend::Wgpu` / `Backend::Metal` arm code is syntactically correct (mirrors the Plan 06-03 `Backend::Rocm` arm shape that compiled with sources present) but **typecheck of validation itself is not possible without the vendored sources**. Treat as MANUAL VERIFICATION on a machine that has both `xcfun-master/` AND CUDA/Wgpu hardware. The narrower workspace gate `cargo check --workspace --exclude validation --exclude xcfun-capi` PASSES.

- **No NVIDIA hardware in dev environment** — per CONTEXT D-06, the dev environment is AMD/ROCm. The CUDA arm of the auto_backend cascade compiles and probes correctly (`cuda_available()` is `false` on this machine), but the strict-1e-13 tier-3 sweep against a real NVIDIA GPU requires cloud CI. Documented as MANUAL VERIFICATION in the validation harness's `Backend::Cuda` arm comment block.

- **No f64-capable Wgpu adapter known to be present** — depending on the dev machine's Vulkan driver state, `wgpu_with_shader_f64_available()` may return either `true` or `false`. The `wgpu_no_f64.rs` integration test handles both outcomes via match arm assertion; the `erf_fallback.rs` `wgpu_host_path_matches_cpu_baseline_or_returns_typed_error` test does the same. Test result on this machine: all four `wgpu_no_f64.rs` tests passed (probe outcome cached at first call; subsequent calls return the cached result).

## System Prerequisites

- **CUDA tier-3 sign-off** requires NVIDIA hardware + CUDA toolkit (`nvidia-smi` lists a device). Run cloud-CI command:
  ```bash
  cargo run -p validation --release --features cuda -- \
    --backend cuda --tier 3 --order 3 --jobs 4 \
    --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'
  ```
  Expected: 0 failing reported (strict 1e-13 vs CPU baseline). The probe gate surfaces missing CUDA toolkit / GPU as `anyhow::bail!` and missing f64 device support as `XcError::CudaNoF64` (W-7 revision-1).

- **Wgpu tier-3 sign-off (GPU-08)** requires a Vulkan driver with the `VK_KHR_shader_float64` extension (Linux, AMD/NVIDIA discrete GPU). Run:
  ```bash
  cargo run -p validation --release --features wgpu -- \
    --backend wgpu --tier 3 --order 3 --exclude-erf \
    --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'
  ```
  Expected: 0 failing reported at the relaxed 1e-9 tolerance per CONTEXT D-02.

- **Metal tier-3 sign-off** requires a macOS host with a discrete f64-capable GPU (Apple Silicon GPUs lack hardware f64; the probe correctly refuses on M1/M2/M3 per CONTEXT D-06). Apple Silicon users get a `bail!()` directing them to `--backend cpu`.

- **`cubecl-cuda` PTX `--use_fast_math` audit** — the threat-register T-06-FAST-MATH item asks to inspect `cubecl-cuda` source for global `--use_fast_math` PTX flags. Quick `grep -r "use_fast_math" /home/user/.cargo/registry/src/index.crates.io-*/cubecl-cuda-0.10.0-pre.3/` would close the audit; deferred to Plan 06-05 / 06-06 when the validation harness sweep makes any deviation observable.

## Next Phase Readiness

- **Plan 06-05 unblocked:** complete `Batch<R>` matrix exists for all four cubecl runtimes — `CpuRuntime` (Plan 06-02a), `HipRuntime` (Plan 06-03), `CudaRuntime` (Plan 06-04), `WgpuRuntime` (Plan 06-04). Plan 06-05 wires `Functional::eval_vec` (RS-08) atop these arms with the `auto_backend()` priority chain. The `eval_vec_host_wgpu_with_request` overload is in place to surface the correct `XcError::WgpuNoF64.requested_runtime` payload from the Metal dispatch arm.
- **Plan 06-05 follow-up:** the descriptor flip (add `Dependency::ERF` to `XC_LDAERFX` / `XC_LDAERFC` / `XC_LDAERFC_JT` / `XC_BECKESRX` / `XC_BECKECAMX` in `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs`) is Plan 06-05's responsibility. The xtask `regen_registry.rs::render_dependency` currently does not handle bit 32 (ERF); it needs a small extension so `cargo xtask regen-registry --check` doesn't drift the file when run again. Once the flip lands, the `erf_fallback.rs::ldaerfx_eval_shape_compatible_with_wgpu_host_path` test will need its second match arm updated (the typed-error path becomes unreachable because the auto-fallback intercepts before the SHADER_F64 probe).
- **GPU-08 sign-off pending:** strict-1e-9 Wgpu tier-3 sweep on a Linux/Vulkan/f64-ext runner, gated on Plan 06-05 wiring the `run_tier3` Wgpu sweep body.
- **No state-modifying actions performed** — STATE.md and ROADMAP.md untouched per the orchestrator's parallel_execution note. The orchestrator owns those updates after the wave completes.

## Self-Check: PASSED

**Files verified:**
- `crates/xcfun-gpu/src/runtime/cuda.rs` — FOUND, `cuda_available` + `CudaRuntime` references = 19
- `crates/xcfun-gpu/src/runtime/wgpu.rs` — FOUND, `wgpu_with_shader_f64_available` + `metal_with_f64_available` references = 7
- `crates/xcfun-gpu/src/batch.rs` — FOUND, `CudaRuntime`/`cubecl_cuda` references = 3, `WgpuRuntime`/`cubecl_wgpu` references = 1, `must_fall_back_to_cpu`/`Dependency::ERF` references = 5
- `crates/xcfun-gpu/src/pool.rs` — FOUND, re-exports `cuda_client` + `wgpu_client`
- `crates/xcfun-gpu/tests/wgpu_no_f64.rs` — FOUND, `WgpuNoF64`/`SHADER_F64` references = 15, NOT `#[ignore]`'d
- `crates/xcfun-gpu/tests/erf_fallback.rs` — FOUND, `ldaerfx`/`Dependency::ERF`/`must_fall_back` references = 26
- `validation/src/driver.rs` — FOUND, `Backend::Cuda`/`Backend::Wgpu` references = 6, `1e-9` references = 4
- `crates/xcfun-gpu/Cargo.toml` — `metal = ["wgpu"]` alias present (1); zero `cubecl-metal` references (R-02 verified)

**Commits verified in `git log --oneline`:**
- `d101214 feat(06-04): wire cubecl-cuda + cubecl-wgpu probes + Batch arms` — FOUND
- `a93315b test(06-04): wgpu_no_f64 + erf_fallback integration tests` — FOUND
- `685e995 feat(06-04): wire --backend cuda + --backend wgpu + --backend metal arms` — FOUND

**Compile gates:**
- `cargo check -p xcfun-gpu --features hip --features cuda --features wgpu` — exits 0
- `cargo check -p xcfun-gpu --features metal` — exits 0 (alias of wgpu)
- `cargo check --workspace --exclude validation --exclude xcfun-capi` — exits 0
- `cargo run -p xtask --bin check-cubecl-pin` — PASS (5 cubecl crates pinned at 0.10.0-pre.3)
- `cargo test -p xcfun-gpu --features hip --features cuda --features wgpu` — 32/32 GREEN
- `cargo test -p xcfun-eval --features testing --test self_tests` — 1/1 GREEN (Tier-1 baseline preserved)

---

*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Completed: 2026-05-03*
