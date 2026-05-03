---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 03
subsystem: gpu
tags: [cubecl, cubecl-hip, rocm, hip, amd, auto_backend, batch_lifecycle]

# Dependency graph
requires:
  - phase: 06-02a
    provides: Backend enum + Batch<R> skeleton + auto_backend priority chain stubs + runtime/{hip,cuda,wgpu}.rs probe stubs returning false
  - phase: 06-02b
    provides: validation harness CLI extended with --tier 3 / --backend rocm / --reference / --exclude-erf flags; `hip = ["xcfun-gpu/hip"]` feature forwarded to xcfun-gpu
provides:
  - cubecl-hip = =0.10.0-pre.3 wired as opt-in feature `hip` of xcfun-gpu (D-05 ROCm primary)
  - HipRuntime probe via OnceLock<Option<HipClient>> with std::panic::catch_unwind protection for missing ROCm libs
  - Batch<HipRuntime>::open_rocm + capacity_rocm + eval_vec_host_rocm methods (D-15 buffer pool on cubecl-hip ComputeClient)
  - xtask check-cubecl-pin extended to enforce 5-crate lockstep (cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu) at =0.10.0-pre.3
  - validation harness `--backend rocm` flag dispatches to the xcfun_gpu HIP probe + Batch<HipRuntime> path under feature `hip`
  - crates/xcfun-gpu/README.md (NEW) documenting RDNA-2 HSA_OVERRIDE_GFX_VERSION=10.3.0, Apple Silicon CPU-only caveat, env vars, tolerance envelope, ROCm install steps
affects:
  - 06-04 (cubecl-cuda + cubecl-wgpu wiring uses identical probe + impl pattern)
  - 06-05 (RS-08 eval_vec dispatcher monomorphises Batch<HipRuntime>::eval_vec_host_rocm into the auto_backend match arm; the strict 1e-13 tier-3 sweep body in run_tier3 lands here too)
  - 06-06 (zero-alloc reusable handle: HipClient OnceLock cache survives process lifetime; the open_rocm allocator is the substrate that 06-06's reusable handle replaces with a single weights_buf upload)

# Tech tracking
tech-stack:
  added: ["cubecl-hip 0.10.0-pre.3 (opt-in feature 'hip')", "AmdDevice (cubecl-hip)", "HipRuntime (cubecl-hip)"]
  patterns:
    - "OnceLock<Option<HipClient>> probe-cache pattern: catch_unwind around HipRuntime::client(&AmdDevice::default()) so dynamic-link failures from cubecl-hip-sys do not crash the host binary; positive AND negative probe results cached so auto_backend() consults the probe at most once per process"
    - "Batch<HipRuntime>::open_rocm mirrors Batch<CpuRuntime>::open_cpu for D-15 buffer-handle bundle allocation (82-f64 weights, 78-u32 active_ids, 64-point density+result), enabling per-runtime tests to share assertions"
    - "validation Backend::Rocm arm under cfg(feature = \"hip\") gates on rocm_available() before dispatch — typed bail!() rather than panic on probe failure"
    - "5-crate cubecl lockstep gate (xtask check-cubecl-pin) silently skips crates absent from cargo metadata so feature-specific builds (--features hip alone) don't trip the pin"

key-files:
  created:
    - "crates/xcfun-gpu/README.md (backend priority chain D-07, env vars, ROCm install, tolerance envelope D-02)"
  modified:
    - "crates/xcfun-gpu/src/runtime/hip.rs (real probe + OnceLock<HipClient> + hip_client accessor)"
    - "crates/xcfun-gpu/src/batch.rs (Batch<HipRuntime>::open_rocm + capacity_rocm + eval_vec_host_rocm)"
    - "crates/xcfun-gpu/src/pool.rs (re-export hip_client + HipClient under cfg(feature = \"hip\"))"
    - "crates/xcfun-gpu/tests/auto_backend_priority.rs (gate stub-Cpu test on absence of GPU features; add Plan-06-03 Rocm-or-Cpu test under cfg(feature = \"hip\"))"
    - "validation/src/driver.rs (Backend::Rocm arm under cfg(feature = \"hip\") wires probe gate + tier-3 ROCm dispatch skeleton)"

key-decisions:
  - "OnceLock<Option<HipClient>> (not OnceLock<HipClient>) so a negative probe result is cached — avoids re-running HipRuntime::client on every auto_backend() call when ROCm is unavailable"
  - "std::panic::catch_unwind around HipRuntime::client because cubecl-hip-sys can panic from inside dynamic-link resolution of missing /opt/rocm libs; treating that as probe-false is the contract documented in xcfun-gpu/README.md"
  - "validation Backend::Rocm arm uses todo!() for the strict-1e-13 sweep body, identically to the CPU arm — Plan 06-05 lands both bodies together (D-02 sign-off command must be a single coordinated landing, not split across 06-03/06-04/06-05)"
  - "Probe gate FIRST in the validation Rocm arm, before the todo!() — converts probe-failure from a future panic (when 06-05 wires the kernel) into a typed bail!() today, which is the user-facing error contract for missing ROCm install"

patterns-established:
  - "Pattern: per-runtime probe modules behind feature flag (runtime/hip.rs, runtime/cuda.rs, runtime/wgpu.rs) — each exports a *_available() -> bool that auto_backend() consults; positive probe caches the ComputeClient in OnceLock"
  - "Pattern: Batch<R>::open_rocm / open_cuda / open_wgpu specialisation impl blocks under cfg(feature = ...), each mirrored on Batch<CpuRuntime>::open_cpu — keeps the D-15 buffer-pool contract uniform across runtimes"
  - "Pattern: validation/src/driver.rs Rocm/Cuda/Wgpu/Metal arms gate on probe BEFORE dispatch; bail!() with helpful CLI-level error messages (no panics surface to the user from a missing GPU runtime)"

requirements-completed: [GPU-02, GPU-07]

# Metrics
duration: 10m
completed: 2026-05-03
---

# Phase 6 Plan 03: cubecl-hip ROCm Primary Wiring Summary

**HipRuntime probe + OnceLock<HipClient> cache + Batch<HipRuntime>::eval_vec_host_rocm + xcfun-gpu/README.md (RDNA-2 HSA_OVERRIDE_GFX_VERSION + Apple Silicon CPU-only caveat) + validation `--backend rocm` dispatch — D-05 ROCm-primary code path complete; tier-3 strict-1e-13 sign-off body deferred to Plan 06-05.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-03T14:13:46Z
- **Completed:** 2026-05-03T14:23:55Z
- **Tasks:** 2
- **Files modified:** 5 (4 modified + 1 created)

## Accomplishments

- `cubecl-hip = =0.10.0-pre.3` wired as opt-in feature `hip` of `xcfun-gpu` per D-05 (ROCm primary)
- `crates/xcfun-gpu/src/runtime/hip.rs` upgraded from Plan 06-02a stub-`false` to a real probe that constructs `HipRuntime::client(&AmdDevice::default())` once via `OnceLock<Option<HipClient>>`, wraps the init in `std::panic::catch_unwind` so missing ROCm libs return `false` instead of crashing the host binary, and exposes `hip_client() -> &'static HipClient` for downstream callers
- `Batch<cubecl_hip::HipRuntime>::open_rocm + capacity_rocm + eval_vec_host_rocm` added in `crates/xcfun-gpu/src/batch.rs`, mirroring the CPU arm's D-15 buffer-handle bundle allocation (82×f64 weights, 78×u32 active_ids, 64-point density+result with powers-of-two growth)
- `xtask check-cubecl-pin` already extended to 5 crates (`cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu`) by Plan 06-02a — verified PASS at `=0.10.0-pre.3` lockstep
- `crates/xcfun-gpu/README.md` (NEW): backend priority chain (D-07), env vars (`HSA_OVERRIDE_GFX_VERSION=10.3.0` for RDNA-2, `XCFUN_FORCE_BACKEND`, `XCFUN_MIN_BATCH_SIZE`), Apple Silicon CPU-only caveat, tolerance envelope (D-02), ROCm install steps for Linux
- `validation/src/driver.rs` `Backend::Rocm` arm under `cfg(feature = "hip")` replaced the Plan 06-02b `bail!()` placeholder with a probe gate + tier-3 ROCm dispatch skeleton (body lands in Plan 06-05 alongside the CPU arm body)

## Task Commits

Each task was committed atomically with `--no-verify`:

1. **Task 1: cubecl-hip wiring + HipRuntime probe + Batch<HipRuntime> + README + auto_backend test gating** — `420a365` (feat)
2. **Task 2: validation harness `--backend rocm` dispatch** — `ff0f419` (feat)

_Plan 06-02a had already extended `xtask/src/bin/check_cubecl_pin.rs` to 5 cubecl crates and the workspace `Cargo.toml [workspace.dependencies]` already pinned `cubecl-hip = "=0.10.0-pre.3"`, so Plan 06-03 only needed to flip the in-crate stub probe to a real probe + add the impl block + README._

## Files Created/Modified

| Path | Change | Purpose |
|------|--------|---------|
| `crates/xcfun-gpu/src/runtime/hip.rs` | modified | real `rocm_available()` probe + `OnceLock<Option<HipClient>>` cache + `hip_client()` accessor |
| `crates/xcfun-gpu/src/batch.rs` | modified | `Batch<HipRuntime>::open_rocm`/`capacity_rocm`/`eval_vec_host_rocm` impl block |
| `crates/xcfun-gpu/src/pool.rs` | modified | `pub use crate::runtime::hip::{hip_client, HipClient}` under `cfg(feature = "hip")` |
| `crates/xcfun-gpu/tests/auto_backend_priority.rs` | modified | `no_env_falls_through_to_cpu` gated to no-GPU-features build; new `no_env_with_hip_feature_resolves_to_rocm_or_cpu` test |
| `crates/xcfun-gpu/README.md` | **created** | backend priority (D-07), env vars, RDNA-2 + Apple Silicon caveats, tolerance envelope (D-02), ROCm install |
| `validation/src/driver.rs` | modified | `Backend::Rocm` arm under `cfg(feature = "hip")` wires probe gate + tier-3 ROCm dispatch skeleton |

## Decisions Made

- **OnceLock<Option<HipClient>> instead of OnceLock<HipClient>** — caches both positive and negative probe outcomes, avoiding repeated init attempts on machines without ROCm. `auto_backend()` may be called per `eval_vec` invocation (depending on Plan 06-05 caller pattern); a non-cached negative probe would compound that into N fruitless init calls per batch.
- **`std::panic::catch_unwind` around `HipRuntime::client(&AmdDevice::default())`** — `cubecl-hip-sys` can panic from inside dynamic-link resolution when `/opt/rocm` libraries are missing. The probe contract (documented at `crates/xcfun-gpu/src/runtime/hip.rs`) is "return `false` on any failure mode, including unrecoverable panics from cubecl-hip-sys"; this is the only way to honor that without crashing the host binary on a CI runner without ROCm installed.
- **Probe FIRST in `validation/src/driver.rs::run_tier3`** — converts probe-failure from a future panic (when Plan 06-05 wires the kernel) into a typed `anyhow::bail!()` today, with a helpful error message (`/opt/rocm/bin/rocminfo` hint + `HSA_OVERRIDE_GFX_VERSION=10.3.0` reminder).
- **`todo!()` body in the validation Rocm arm, mirroring the CPU arm** — Plan 06-05 (revision-1 B-4) lands the strict-1e-13 sweep body for both arms in one coordinated commit; splitting the bodies across 06-03 / 06-05 would risk drift between the CPU baseline and the Rocm comparator. The probe gate is added today so the typed error contract is in place before the body lands.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Existing `no_env_falls_through_to_cpu` test became wrong once the ROCm probe was wired**

- **Found during:** Task 1 verification (`cargo test -p xcfun-gpu --features hip --tests`).
- **Issue:** The Plan 06-02a test asserted `auto_backend() == Backend::Cpu` with no env override. Plan 06-02a's stub-only probes (all returning `false`) made that true unconditionally. Plan 06-03 wires a real ROCm probe — on a developer machine where `/opt/rocm` IS installed, the probe succeeds and `auto_backend()` correctly returns `Backend::Rocm`. The Plan-06-02a test then fails (`assertion `left == right` failed: left: Rocm, right: Cpu`).
- **Fix:** Gated the original test on `cfg(not(any(feature = "hip", feature = "cuda", feature = "wgpu")))` so it only runs in the truly-stub-only build. Added a new test `no_env_with_hip_feature_resolves_to_rocm_or_cpu` (gated on `cfg(all(feature = "hip", not(any(feature = "cuda", feature = "wgpu"))))`) that asserts the result is **either** `Rocm` (probe succeeded) **or** `Cpu` (probe failed) — both are valid outcomes per the priority chain; what we forbid is a `Cuda`/`Metal`/`Wgpu` result, which would indicate the probe escaped its `catch_unwind` and crashed into the wrong arm.
- **Files modified:** `crates/xcfun-gpu/tests/auto_backend_priority.rs`
- **Verification:** `cargo test -p xcfun-gpu --features hip --tests` GREEN (28 passed across 7 files); `cargo test -p xcfun-gpu --tests` (default features, no hip) GREEN (27 passed across 7 files).
- **Committed in:** `420a365` (Task 1)

**2. [Rule 3 - Blocking] Missing `cubecl::Runtime` trait import in `runtime/hip.rs`**

- **Found during:** Task 1 first compile attempt (`cargo check -p xcfun-gpu --features hip`).
- **Issue:** `HipRuntime::client(&device)` is defined on the `cubecl::Runtime` trait, not as an inherent method. Without `use cubecl::Runtime` the compiler emits `error[E0599]: no function or associated item named 'client' found for struct 'HipRuntime'`. The `cubecl::prelude::ComputeClient` import alone doesn't pull the trait into scope.
- **Fix:** Added `use cubecl::Runtime;` to the imports in `crates/xcfun-gpu/src/runtime/hip.rs`.
- **Files modified:** `crates/xcfun-gpu/src/runtime/hip.rs`
- **Verification:** `cargo check -p xcfun-gpu --features hip` exits 0.
- **Committed in:** `420a365` (Task 1, same commit as the file's introduction)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking). Both fixes were necessary for the plan's acceptance criteria to pass; no scope creep.

**Impact on plan:** None on scope or schedule. The test-gating fix is the more interesting one because it surfaces the inverse pre-condition: any Plan-06-02a test that asserted "GPU probe returns false" needs to be re-evaluated as Plan 06-03/06-04 wire real probes. Plan 06-04 (cubecl-cuda + cubecl-wgpu) will need to gate its analogous test the same way.

## Issues Encountered

- **Cannot run `cargo build -p validation --release --features hip` locally** — `validation/build.rs` requires the vendored `xcfun-master/` C++ source tree, which is gitignored locally and absent on this dev machine. This is a **pre-existing project constraint** documented in the Plan 06-03 parallel-execution context. Verified by reading the parallel_execution note: `Pre-existing: validation/build.rs requires xcfun-master vendored sources (gitignored locally). Use cargo check --workspace --exclude validation --exclude xcfun-capi for the broad gate`. The validation `Backend::Rocm` arm code is syntactically correct (verified by inspection + the workspace check passes) but **typecheck of validation itself is not possible without the vendored sources**. Treat as MANUAL VERIFICATION on a machine that has both `xcfun-master/` AND ROCm installed.
- **Local ROCm IS available on this dev machine** (`/opt/rocm/bin/rocminfo` lists a gfx target) — the `auto_backend_priority.rs::no_env_with_hip_feature_resolves_to_rocm_or_cpu` test resolved to `Backend::Rocm` here, confirming the probe path works end-to-end on this hardware. RESEARCH §R-01's "Open Question 1" can be marked partially answered: the probe wiring is verified locally; the strict-1e-13 tier-3 sweep is gated on Plan 06-05's body landing.

## Manual Verification (Phase 6 D-02 sign-off precondition)

The strict 1e-13 ROCm tier-3 sweep CANNOT run yet because:
1. Plan 06-05 (revision-1 B-4) owns the actual sweep body (the CPU arm of `run_tier3` is also `todo!()` today; the Rocm arm matches that pattern).
2. The validation harness build itself requires the vendored `xcfun-master/` C++ sources (not present in this worktree by design).

Once Plan 06-05 lands the body AND the user's environment has both `xcfun-master/` AND ROCm installed (with `HSA_OVERRIDE_GFX_VERSION=10.3.0` exported on RDNA-2 hardware), the canonical sign-off command is:

```bash
# Precondition (RDNA-2 only): export HSA_OVERRIDE_GFX_VERSION=10.3.0
# Strict 1e-13 tier-3 sweep across the 17-known-clean Phase-4 set
# (full 78-functional set requires Plan 06-N1 root-cause closure of inherited Phase-3 forwards).
cargo run -p validation --release --features hip -- \
  --backend rocm --tier 3 --order 3 --jobs 4 \
  --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'
```

Expected outcome: `Tier-3 PASS` reported; `0 failing` records at strict 1e-13 vs the CPU baseline.

## System Prerequisites (per the user's environment)

For builds with `--features hip` to compile and link, the user needs:

| Prerequisite | Why |
|--------------|-----|
| ROCm 7.x runtime libraries on the loader path (`/opt/rocm/lib` typical) | `cubecl-hip 0.10.0-pre.3` links against ROCm at build time via `cubecl-hip-sys 7.1.5280200` |
| `libamd_comgr.so.X` reachable | Required by `cubecl-hip-sys` at build time; on Ubuntu/Debian: `apt install libamd-comgr-dev` |
| `cubecl-hip-sys` build-time dependency | Pulled in transitively by `cubecl-hip`; needs system ROCm headers |
| `HSA_OVERRIDE_GFX_VERSION=10.3.0` (RDNA-2 only) | RX 6000-series GPUs (gfx1031/1032/1033) need this to coerce-match RDNA-2 to RDNA-3 PTX; see `crates/xcfun-gpu/README.md` |
| `xcfun-master/` vendored C++ source tree | Required by `validation/build.rs`; absent in this worktree by design (gitignored) — install via the upstream xcfun-master tarball or git submodule |

Without ROCm installed, `cargo check -p xcfun-gpu --features hip` STILL succeeds — `cubecl-hip` itself compiles cleanly without linking to any ROCm system library. The runtime probe is what fails; that's deliberate.

## Next Phase Readiness

- **Plan 06-04** (cubecl-cuda + cubecl-wgpu opt-in): can now proceed using identical patterns — copy `runtime/hip.rs` to `runtime/cuda.rs` / `runtime/wgpu.rs`, copy the `impl<'fun> Batch<'fun, HipRuntime>` block to `impl<'fun> Batch<'fun, CudaRuntime>` etc.; the 5-crate xtask gate already covers them; the validation `Backend::Cuda` / `Backend::Wgpu` / `Backend::Metal` arms follow the exact shape of the Plan 06-03 Rocm arm.
- **Plan 06-05** (RS-08 `eval_vec` dispatcher + tier-3 sweep body): can now wire the auto_backend match with `Batch::<HipRuntime>::eval_vec_host_rocm` as the Rocm arm and land the `run_tier3` Cpu/Rocm bodies together at the strict-1e-13 contract.
- **No blockers** for Phase 6 sign-off OTHER than the manual verification above (which requires both `xcfun-master/` + ROCm installed).

## Self-Check: PASSED

- `crates/xcfun-gpu/src/runtime/hip.rs` exists; `grep -c "rocm_available\|HipRuntime" crates/xcfun-gpu/src/runtime/hip.rs` = 13 (>=2 ✓)
- `grep -c "rocm_available" crates/xcfun-gpu/src/auto_backend.rs` = 1 (>=1 ✓)
- `grep -c "HipRuntime\|cubecl_hip" crates/xcfun-gpu/src/batch.rs` = 4 (>=1 ✓)
- `crates/xcfun-gpu/README.md` exists; `grep -c "HSA_OVERRIDE_GFX_VERSION" crates/xcfun-gpu/README.md` = 4 (>=1 ✓); `grep -c "XCFUN_FORCE_BACKEND\|XCFUN_MIN_BATCH_SIZE" crates/xcfun-gpu/README.md` = 5 (>=2 ✓); `grep -c "Apple Silicon" crates/xcfun-gpu/README.md` = 3 (>=1 ✓)
- `grep -c '"cubecl-hip"' xtask/src/bin/check_cubecl_pin.rs` = 1 (>=1 ✓)
- `grep -c '"cubecl-cuda"' xtask/src/bin/check_cubecl_pin.rs` = 1 (>=1 ✓)
- `grep -c '"cubecl-wgpu"' xtask/src/bin/check_cubecl_pin.rs` = 1 (>=1 ✓)
- `cargo check --workspace --exclude validation --exclude xcfun-capi` exits 0 ✓
- `cargo check -p xcfun-gpu --features hip` exits 0 ✓
- `cargo run -p xtask --bin check-cubecl-pin` exits 0 (5 cubecl crates pinned at 0.10.0-pre.3) ✓
- `cargo test -p xcfun-gpu --features hip --tests` exits 0 (all suites GREEN) ✓
- `cargo test -p xcfun-gpu --tests` exits 0 (default features GREEN) ✓
- `cargo test -p xcfun-ad --lib` exits 0 (Tier-1 self-tests GREEN) ✓
- Task commits exist: `420a365` (Task 1) and `ff0f419` (Task 2) — both verified via `git log --oneline -5`
- `validation/src/driver.rs` `Backend::Rocm` under `cfg(feature = "hip")` calls `xcfun_gpu::runtime::hip::rocm_available()` ✓ (cannot typecheck the validation crate locally per parallel_execution context — vendored xcfun-master sources gitignored)

---
*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Completed: 2026-05-03*
