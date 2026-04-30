# Phase 6: GPU Backends + Batch Lifecycle (`xcfun-kernels` / `xcfun-gpu`) - Research

**Researched:** 2026-04-30
**Domain:** cubecl 0.10-pre.3 multi-runtime kernel DSL (CPU / ROCm / CUDA / Wgpu) + AD substrate extension + Rust facade zero-alloc batch lifecycle + 30+ Phase-3/4 D-19 numerical-parity cleanup
**Confidence:** HIGH for cubecl API + version pins + crate split shape; MEDIUM for AD `N≥4` recursion port (algorithmic identity vs. f64-rounding); LOW for ROCm-on-this-host execution (no `/opt/rocm` directory found locally — see Step 2.6 audit).

## Summary

Phase 6 stacks three concurrent deliverables on top of an already-shipped cubecl-cpu single-source kernel substrate:

1. **AD algebra extension** — port `ctaylor_compose` / `ctaylor_multo` outer dispatch from `xcfun-master/external/upstream/taylor/ctaylor.hpp` to `N ∈ {4, 5, 6}` via the same recursion structure already used for `N ∈ {0, 1, 2, 3}` in `crates/xcfun-ad/src/ctaylor_rec/`. Add libm-hybrid `erf_precise_taylor<F, const N: u32>` (extending the FreeBSD-msun `erf_precise` already in `crates/xcfun-ad/src/expand/erf.rs:174`). Add `tau ≥ tau_w` clamp guard inside the existing `ctaylor_max` helper (`crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818`). Add an mpmath sidecar in `xtask` for offline ground-truth fixtures.
2. **Crate split + GPU runtimes** — git-mv `xcfun-eval/src/{functionals,density_vars,dispatch}` → new `crates/xcfun-kernels/`, leaving `xcfun-eval` with only `Functional` + `eval_point_kernel` + cubecl-cpu validation substrate. Unstub `crates/xcfun-gpu/` with `Backend` enum, `Batch<'fun, R: cubecl::Runtime>`, `auto_backend()`, generation-counter buffer pool, ERF-fallback routing. Wire `cubecl-hip` (PRIMARY ROCm/AMD), `cubecl-cuda` (opt-in), `cubecl-wgpu` (portable fallback covering Vulkan/Metal/DX12/WebGPU) behind feature flags.
3. **`Functional::eval_vec` + zero-alloc + cleanup** — RS-08 dispatch through `xcfun-gpu::Batch<R>` when `nr_points ≥ 64`. Strict zero-alloc per-point form via pre-allocated reusable handle in `Functional` (~287 → 0 allocs/eval after first call). 9 plans cover algebra, reorg, GPU runtimes, and the three D-19 cleanup tracks (06-N1 inherited Phase-3/4 forwards, 06-N2 mpmath-only fixtures for the 20 `excluded_by_upstream_spec` set, 06-N3 post-libm-hybrid sweep verification).

**Primary recommendation:** Land Plan 06-00 (substrate work) in the *current* `xcfun-eval` tree FIRST so the `git mv` in Plan 06-01 has zero algebraic-change overlap. Pre-pin all four cubecl runtime crates lock-step (`cubecl`, `cubecl-cpu`, `cubecl-hip`, `cubecl-cuda`, `cubecl-wgpu`) at the same exact version — never `cubecl-metal` (it does not exist as a separate crate; Metal is reached via cubecl-wgpu's Metal backend).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| AD algebra `ctaylor_compose/multo` for N ∈ {4,5,6} | `xcfun-ad` (cubecl-bearing math crate) | — | Already owns `N ∈ {0,1,2,3}` recursion; Phase 1 D-08 contract preserved |
| Libm-hybrid `erf_precise_taylor` | `xcfun-ad::expand::erf` | — | Phase 2 already landed `erf_precise` here; Phase 6 extends to AD-chain form |
| `tau ≥ tau_w` clamp guard | `xcfun-kernels::functionals::mgga::shared::tpss_like` (post 06-01 reorg) | `xcfun-ad` (`ctaylor_max` consumer only) | Lives at the per-functional kernel layer; `ctaylor_max` already exists |
| mpmath fixture generator | `xtask` (Python sidecar) | `validation/fixtures/mpmath/*.jsonl` | Offline tool; not in the runtime path; matches xtask `regen-*` pattern |
| Per-functional `#[cube] fn` bodies | `xcfun-kernels` (post 06-01) | — | Single source of truth; runtime-agnostic, only depends on `cubecl` core |
| `DensVarsDev<F>` + `build_densvars` + `regularize` + `dispatch_kernel` | `xcfun-kernels` (post 06-01) | — | Same boundary; designed-doc-05 §3 explicitly puts these in xcfun-kernels |
| `Functional` struct + `eval_point_kernel` (per-point launcher) | `xcfun-eval` (retains) | — | Phase 2 D-21; CPU per-point validation substrate; depends on cubecl-cpu |
| `Backend` enum + `Batch<'fun, R>` + `auto_backend()` + buffer pool | `xcfun-gpu` (newly unstubbed) | — | Design-doc-05 §4; only consumer of cubecl-cuda / -hip / -wgpu |
| ERF fallback routing (Wgpu/Metal SHADER_F64-or-no-erf) | `xcfun-gpu::Batch::open` | — | Inspects `Dependency::ERF` mask; routes to `Backend::Cpu` automatically |
| `Functional::eval_vec` GPU dispatch + threshold | `xcfun-rs::Functional` (facade) | `xcfun-gpu::Batch<R>` | RS-08; dispatches to `Batch` when `nr_points ≥ 64` |
| Zero-alloc reusable handle | `xcfun-rs::Functional` (interior mutability via `UnsafeCell`) | `xcfun-eval::Functional` (extends with private buffers) | Phase 5 D-13 forward; Send + Sync preserved with documented "racy if shared" |
| C ABI `xcfun_eval_vec` wiring | `xcfun-capi` (Phase 5 stub) | — | Caller-supplied pitches; Rust forwards to `Functional::eval_vec` |

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Phase scope structure (D-01/D-02):**
- D-01: Wide Phase 6 — single GSD phase, ~10–15 plans (decimal numbering = plan organisation, not sub-phases). Plan tree per CONTEXT.md: 06-00 (substrate), 06-01 (`xcfun-kernels` git-mv), 06-02 (xcfun-gpu unstub + Backend/Batch/buffer pool/auto_backend + WgpuNoF64 typed variant), 06-03 (cubecl-hip primary wiring), 06-04 (cubecl-cuda + cubecl-metal-via-wgpu opt-in + cubecl-wgpu portable + SHADER_F64 probe), 06-05 (RS-08 eval_vec + threshold + ERF auto-fallback), 06-06 (zero-alloc reusable handle + Phase-5 weights `Box::leak` refactor + LDA-vars=6 DensVars-driven dispatch), 06-N1/N2/N3 (D-19 cleanup).
- D-02: Strict 1e-13 across all 78 functionals at Phase 6 sign-off; tier-3 GREEN via `cargo run -p validation --release -- --backend rocm --order 3 --filter '.*'`.

**Ground-truth policy (D-03/D-04):**
- D-03: ACC-04 amendment — mpmath truth where C++ documents cancellation. Default ground truth stays C++ xcfun. For density points where the C++ source explicitly notes bracket cancellation (e.g., `xcfun-master/src/functionals/ldaerfx.cpp:66` `test_threshold` rationale), tier-2 / tier-3 reference switches to mpmath at 200-digit precision computed offline and committed as JSONL fixtures.
- D-04: mpmath fixture generator landed in `xtask` during Plan 06-00 via Python sidecar (`subprocess::Command::new("python3").arg("-m").arg("xtask.mpmath_eval")`). NOT a runtime/library dep.

**GPU backend strategy (D-05/D-06/D-07):**
- D-05: ROCm/HIP is the PRIMARY GPU backend. `cubecl-hip = "=0.10.0-pre.3"` (feature flag `hip`) carries the strict 1e-13 tier-3 contract. RDNA-2 GPUs need `HSA_OVERRIDE_GFX_VERSION=10.3.0` documented in `xcfun-gpu/README.md`.
- D-06: CUDA + Metal as opt-in best-effort feature flags. `cubecl-cuda = "=0.10.0-pre.3"` (feature `cuda`); **Metal is reached via `cubecl-wgpu`** (Apple Silicon caveat: Apple Silicon GPUs lack hardware f64; runtime probe and refuse).
- D-07: `auto_backend()` priority — env `XCFUN_FORCE_BACKEND` → ROCm-if-available → CUDA-if-available-and-`cuda`-feature → Metal-if-available-and-f64-and-`metal`-feature → Wgpu-if-`SHADER_F64`-and-`wgpu`-feature → CPU.

**Crate boundary (D-08/D-09):**
- D-08: Full `xcfun-kernels` + `xcfun-gpu` split. Migrate from `xcfun-eval/src/`: `functionals/`, `density_vars/`, `density_vars.rs`, `dispatch.rs` → new `crates/xcfun-kernels/`. `xcfun-kernels` exposes `#[cube] fn` kernel bodies, `DensVarsDev<F>`, `dispatch_kernel<F>`. Never instantiates a runtime; only depends on `cubecl` core.
- D-09: Migration order — substrate FIRST (Plan 06-00 in current `xcfun-eval` tree), MOVE SECOND (Plan 06-01).

**Numerical / kernel guards (D-10/D-11):**
- D-10: TPSS-correlation `tau ≥ tau_w` hard-clamp guard inside TPSSC/TPSSLOCC/REVTPSSC kernels.
- D-11: Libm-hybrid `erf` extension — `erf_precise_taylor<F: Float, const N: u32>` using stable-bracket reduction inside the AD chain.

**Batch + dispatch (D-12 through D-18):**
- D-12: Pre-allocated reusable handle in `Functional` (zero-alloc strategy). `eval(&self, ...)` keeps `&self`; interior mutability via `UnsafeCell` (or `RefCell`). Document "racy if shared".
- D-13 / D-13-A: Wgpu `SHADER_F64`-missing → typed `XcError::WgpuNoF64 { adapter_name: &'static str, requested_runtime: Backend }`. `Box::leak` once at runtime to obtain `&'static str` (preserves Phase 2 D-25 `Copy + non_exhaustive`).
- D-14: `eval_vec` dispatch threshold = 64 (default), env-overridable via `XCFUN_MIN_BATCH_SIZE`.
- D-15: Buffer pool grows powers-of-two doubling. `weights_buf` (82 f64) + `active_ids_buf` (78 u32) fixed-size, allocated once. Generation counter monotonic `u64` on `Functional::settings`.
- D-16: `xcfun-rs::Functional::eval_vec` matches upstream `xcfun_eval_vec` signature.
- D-17: `xcfun-eval::Functional::weights` refactor `&'static [(FunctionalId, f64)]` → `Vec<(FunctionalId, f64)>` (drops Phase 5 `Box::leak`).
- D-18: LDA-vars=6 launch arms via DensVars-driven dispatch — kernel's `Dependency` mask determines which Vars subset arms it can launch into.

### Claude's Discretion
- Exact form of `SyncUnsafeCell` vs `RefCell` — implementer picks based on RS-10 contract.
- Whether `Backend` enum lives in `xcfun-core` or `xcfun-gpu`.
- cubecl-feature-flag default in `xcfun-gpu` (recommend `default = ["cpu"]`; `hip`/`cuda`/`wgpu` opt-in).
- Layout of mpmath-fixture JSONL files (record-per-line vs nested vs separate files per functional).
- Threshold for "small-magnitude residual" in Plan 06-N3 sweep (anything between 1e-15 and 1e-9).
- Plan ordering of 06-N1/N2/N3 (parallel vs sequential).
- Which xcfun-ad N≥4 specialisations carry which `#[comptime]` constants.

### Deferred Ideas (OUT OF SCOPE)
- Stream-overlapped async GPU dispatch (PROJECT.md: < 20% throughput gain for 3× API complexity).
- `Line<F>` lane vectorisation (`docs/design/06 §8` revisits in v2).
- Shared-memory reductions.
- Stable `cubecl 0.10` release migration (when it ships).
- Patches to `xcfun-master/` C++.
- CUDA / Metal local validation (no hardware in dev environment; cloud-CI best-effort).
- `Backend::OpenCL` / `Backend::Vulkan` direct (Wgpu covers Vulkan).
- PyO3 + NumPy interop (Phase 7).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RS-08 | `Functional::eval_vec(&self, density, density_pitch, out, out_pitch, nr_points) -> Result<(), XcError>` dispatches to `Batch<R>` when `nr_points ≥ 64` | §"Backend dispatch shape" + §"Recommended plan tree" Plan 06-05 |
| KER-01 | Every functional has a `#[cube]` counterpart generic over `F: Float` whose body is the single source of truth | Already shipped Phases 2–4 (78 kernels); KER-01 verified by `xcfun-kernels` post-Plan-06-01 git-mv (no behavioural change) |
| KER-02 | `DensVarsDev<F, N>` mirrors `DensVars<T>` with identical field order and builder logic | Already shipped (Phase 2 D-02); migrates with bodies |
| KER-03 | `CTaylorDev<F, N>` uses the same bit-flag indexing as `CTaylor<T, N>` | `xcfun-ad::CTaylor<F, N>` already covers both — no separate `CTaylorDev` exists post-cubecl-pivot (Phase 1 design-doc-06 banner) |
| KER-04 | `eval_batch_kernel<F: Float>` is a `#[cube(launch_unchecked)]` entry point with `#[comptime]` specialisation on `(vars, mode, order)` and runtime dispatch on `FunctionalId` | §"Backend dispatch shape" — extend `eval_point_kernel` from `crates/xcfun-eval/src/functional.rs:54` to a multi-point batch entry |
| KER-05 | Inside any `#[cube]` body, only other `#[cube]` functions and cubecl intrinsics are callable | Already enforced by `#[cube]` macro at compile time |
| KER-06 | Tier-3 parity — 10k-point grid through `Batch<CpuRuntime>` matches scalar `Functional::eval` within 1e-13 relative error | §"Validation Architecture" — tier-3 sweep design |
| GPU-01 | `Batch<'fun, R: cubecl::Runtime>` exposes `reserve`, `upload_density`, `launch`, `download_result`, `eval_vec_host` | §"Buffer pool design" + §"Backend dispatch shape" |
| GPU-02 | `Backend` enum (`Cpu`, `Rocm`, `Cuda`, `Metal`, `Wgpu`); `auto_backend()` priority per D-07 | §"Backend dispatch shape" |
| GPU-03 | `cubecl-hip` enabled under feature `hip`; `cubecl-cuda` under `cuda`; `cubecl-wgpu` under `wgpu`; CPU always on | §"Recommended plan tree" Plans 06-03 / 06-04 |
| GPU-04 | Device-buffer pool grows with powers-of-two; weights uploaded once per batch (generation-counter guarded) | §"Buffer pool design" |
| GPU-05 | On Wgpu, functionals with `Dependency::ERF` are routed to `Backend::Cpu` automatically | §"Backend dispatch shape" |
| GPU-06 | Wgpu without `SHADER_F64` returns `Err(XcError::WgpuNoF64)` at batch open; compile-time `size_of::<Scalar>() == 8` assertion | §"Backend dispatch shape" + §"Risks & open questions" |
| GPU-07 | Tier-3 parity on ROCm — 10k-point grid within 1e-13 rel-err vs. CPU (CUDA/Metal best-effort) | §"Validation Architecture" |
| GPU-08 | Tier-3 parity on Wgpu — 10k-point grid (excluding range-separated functionals) within 1e-9 rel-err vs. CPU | §"Validation Architecture" |

## Standard Stack

### Core (already in workspace; verified against crates.io 2026-04-30)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cubecl` | `=0.10.0-pre.3` | Kernel DSL + `Runtime` trait | The only Rust-native kernel DSL with ROCm/CUDA/Wgpu/CPU lock-step. Pre-release pin is load-bearing (semver not respected between pre-releases) [VERIFIED: crates.io — newest 0.10.0-pre.4, max_stable 0.9.0; project pins one minor pre-release behind the head — see Risk] |
| `cubecl-cpu` | `=0.10.0-pre.3` | `CpuRuntime` (always-on validation substrate) | Already in workspace; same lockstep version |
| `cubecl-hip` | `=0.10.0-pre.3` | `HipRuntime` for AMD ROCm — PRIMARY backend | NEW addition Phase 6 D-05; lockstep pin [VERIFIED: crates.io] |
| `cubecl-cuda` | `=0.10.0-pre.3` | `CudaRuntime` opt-in feature `cuda` | NEW Phase 6 D-06; lockstep pin [VERIFIED: crates.io] |
| `cubecl-wgpu` | `=0.10.0-pre.3` | `WgpuRuntime` covers Vulkan/Metal/DX12/WebGPU | NEW Phase 6; lockstep pin. **Metal is reached HERE, not via a separate `cubecl-metal` crate** [VERIFIED: crates.io — `cubecl-metal` does NOT exist] |

### Supporting (already in workspace)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `thiserror` | `=2.0.18` | `XcError::WgpuNoF64` derive | Existing pattern — extending the enum with one variant |
| `bitflags` | `=2.11.1` | `Dependency` (already used) | `Backend` could be `bitflags`-based but enum is simpler — discretion call |
| `static_assertions` | (workspace dev-dep) | `assert_impl_all!(Functional: Send, Sync)` compile-time gate | Already used in `xcfun-rs/tests/send_sync.rs` |
| `serde` / `serde_json` | `=1.0` / `=1.0.149` | mpmath fixture JSONL serialisation | xtask layer only; library graph stays serde-free |
| `rand_xoshiro` | `=0.8.0` | Deterministic 10k-point grid for tier-3 | Already in validation/ harness |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Pin `=0.10.0-pre.3` | Bump to `=0.10.0-pre.4` | Available; would require running tier-3 across all 78 functionals to re-validate. Defer until cubecl 0.10 stable per CONTEXT.md "Deferred Ideas" |
| `cubecl-metal` (separate crate) | `cubecl-wgpu` + Metal backend | **No alternative — `cubecl-metal` does not exist on crates.io.** D-06 phrasing must be corrected at sign-off |
| `Box<dyn Runtime>` for `Backend` enum | Type-erased trait object | Cubecl's `Runtime` trait carries associated types (`Device`, `Server`, `Channel`) that make `dyn Runtime` non-trivial. Use enum + monomorphised dispatch (one `Batch<R>` instantiation per concrete runtime) |
| `UnsafeCell` for reusable handle | `RefCell` (Send + !Sync) | RefCell loses `Sync`; documented design says `Functional: Send + Sync` with "racy if shared" semantics — `UnsafeCell` matches |
| Hash-based weights cache invalidation | Generation counter | Hash 656 bytes per launch on every kernel invocation; counter is u64 inc on `set` only — D-15 explicit |

**Installation (Plan 06-04 — feature-gated cubecl runtimes):**

```toml
# crates/xcfun-gpu/Cargo.toml
[features]
default = ["cpu"]                # CpuRuntime always available
cpu     = ["cubecl-cpu"]
hip     = ["cubecl-hip"]         # ROCm/AMD — primary backend (D-05)
cuda    = ["cubecl-cuda"]        # NVIDIA — opt-in (D-06)
wgpu    = ["cubecl-wgpu"]        # portable: Metal / Vulkan / DX12 / WebGPU
metal   = ["cubecl-wgpu"]        # alias of wgpu — D-06 D-07 phrasing

[dependencies]
xcfun-kernels = { path = "../xcfun-kernels" }
xcfun-core    = { path = "../xcfun-core" }
xcfun-ad      = { path = "../xcfun-ad" }
cubecl        = { workspace = true }
cubecl-cpu    = { workspace = true, optional = true }
cubecl-hip    = { workspace = true, optional = true }
cubecl-cuda   = { workspace = true, optional = true }
cubecl-wgpu   = { workspace = true, optional = true }
thiserror     = { workspace = true }
```

**Version verification (run during Plan 06-02 — D-02 / D-04 cubecl-runtime-pin extension):**

```bash
cargo metadata --format-version=1 | jq -r '
  .packages[] | select(.name | startswith("cubecl"))
  | "\(.name) \(.version)"' | sort
```

Expected output: every `cubecl-*` line shows `0.10.0-pre.3`. Add this to `xtask check-cubecl-pin` (Plan 02-02 pattern) covering the new four crates (`cubecl-hip`, `cubecl-cuda`, `cubecl-wgpu` plus existing `cubecl`, `cubecl-cpu`).

## Architecture Patterns

### System Architecture Diagram

```
                                       ┌──────────────────────┐
                                       │   user (Rust caller) │
                                       └──────────┬───────────┘
                                                  │ Functional::eval_vec(...)
                                                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ xcfun-rs::Functional   (facade — RS-08 dispatch + zero-alloc handle D-12) │
│  ── nr_points < 64  → eval-loop fall-through (per-point `eval`)           │
│  ── nr_points ≥ 64  → xcfun-gpu::Batch<R>::eval_vec_host(...)             │
└────────────────┬─────────────────────────┬────────────────────────────────┘
                 │ per-point                │ batch
                 ▼                          ▼
┌──────────────────────┐        ┌─────────────────────────────────────┐
│ xcfun-eval           │        │ xcfun-gpu                           │
│  ── Functional       │        │  ── Backend enum                    │
│  ── eval_point_kernel│        │  ── Batch<'fun, R: cubecl::Runtime> │
│  ── for_tests::      │        │  ── auto_backend() priority chain   │
│       cpu_client     │        │  ── buffer pool (gen counter)       │
│     (CpuRuntime)     │        │  ── ERF auto-fallback to CPU        │
└─────────┬────────────┘        └────┬────────────────────────────────┘
          │                          │
          ▼                          ▼ (R = HipRuntime / CudaRuntime / WgpuRuntime / CpuRuntime)
┌────────────────────────────────────────────────────────────────────────┐
│ xcfun-kernels   (#[cube] fn — single source of truth)                   │
│  ── functionals/{lda,gga,mgga}/<name>.rs  (78 kernel bodies)            │
│  ── density_vars/{build, regularize}      (DensVarsDev<F> + builders)   │
│  ── dispatch::dispatch_kernel<F>          (FunctionalId match arms)     │
└─────────┬───────────────────────────────────────────────────────────────┘
          │ (calls only #[cube] fn / cubecl intrinsics)
          ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ xcfun-ad   (cubecl-bearing math primitives)                              │
│  ── CTaylor<F, N> (length 1<<N Array<F>)                                 │
│  ── ctaylor_rec::{mul, multo, compose}  ← Phase 6 extends to N ∈ {4,5,6} │
│  ── expand::{exp, log, pow, sqrt, erf, expm1, ...}                       │
│  ── math::ctaylor_{reciprocal,sqrt,exp,log,pow,erf,asinh,atan}           │
│  ── math::erf_precise_taylor<F, const N: u32>  ← Phase 6 D-11 NEW        │
└──────────────────────────────────────────────────────────────────────────┘

Validation paths (out-of-band):
  validation/  cc-links xcfun-master C++ — emits report.html + report.jsonl
  xtask/mpmath_eval (Python sidecar)  — emits validation/fixtures/mpmath/*.jsonl
                                        for points where C++ documents cancellation
```

### Recommended Project Structure (post Plan 06-01 git-mv)

```
crates/
├── xcfun-ad/              # cubecl-bearing AD primitives (extended N ≤ 6)
│   ├── src/
│   │   ├── ctaylor_rec/   # multo / compose recursion — extend to N=4,5,6
│   │   ├── expand/        # *_expand series ports
│   │   ├── math.rs        # composed CTaylor ops + erf_precise_taylor (NEW)
│   │   └── lib.rs
│   └── tests/             # property tests + golden_compose_n4.rs / multo_n4.rs (NEW)
├── xcfun-core/            # types + registry tables (cubecl-free; unchanged)
├── xcfun-kernels/         # NEW (Plan 06-01 git-mv from xcfun-eval)
│   ├── src/
│   │   ├── functionals/   # lda/, gga/, mgga/ — 78 kernel bodies (moved)
│   │   ├── density_vars/  # build.rs + regularize.rs (moved)
│   │   ├── dispatch.rs    # FunctionalId-keyed dispatch_kernel (moved)
│   │   └── lib.rs
│   └── tests/             # tier-1 self-tests (moved from xcfun-eval/tests)
├── xcfun-eval/            # SHRUNK: only Functional + eval_point_kernel + cpu_client
│   ├── src/
│   │   ├── functional.rs  # Functional struct + eval entry point
│   │   ├── for_tests.rs   # CpuClient OnceLock — promoted to production
│   │   └── lib.rs
│   └── tests/             # zero_alloc_strict.rs (NEW Plan 06-06)
├── xcfun-gpu/             # NEW (Plan 06-02 unstub)
│   ├── src/
│   │   ├── backend.rs     # Backend enum + auto_backend()
│   │   ├── batch.rs       # Batch<'fun, R: cubecl::Runtime>
│   │   ├── pool.rs        # generation-counter buffer pool
│   │   ├── runtime/       # one module per feature: hip/cuda/wgpu_with_metal
│   │   └── lib.rs
│   └── README.md          # ROCm RDNA-2 HSA_OVERRIDE_GFX_VERSION note
├── xcfun-rs/              # facade — gains eval_vec + zero-alloc handle
│   ├── src/
│   │   ├── functional.rs  # Functional newtype + eval_vec dispatch + UnsafeCell handle
│   │   └── lib.rs
│   └── tests/
│       ├── eval_vec_threshold.rs (NEW)
│       ├── zero_alloc_strict.rs  (NEW — counts == 0)
│       └── send_sync.rs (existing)
└── xcfun-capi/            # Phase 5; eval_vec C ABI wires through xcfun-rs

xtask/
├── src/bin/
│   ├── regen_mpmath_fixtures.rs  (NEW Plan 06-00 — Python sidecar driver)
│   └── ...
└── mpmath_eval/                  (NEW Plan 06-00)
    ├── __init__.py
    ├── __main__.py               # entry point: python3 -m xtask.mpmath_eval
    ├── functionals.py            # mpmath ports of LDAERFX/TPSSC/etc.
    └── README.md                 # mpmath prec=200 reproducibility
```

### Pattern 1: Per-N CTaylor recursion specialisation

**What:** The C++ general-recursion form at `ctaylor.hpp:55-65` (multo) and `:72-82` (compose) takes a template parameter `Nvar` and recurses to `Nvar-1`. The current Rust port (`crates/xcfun-ad/src/ctaylor_rec/{multo.rs, compose.rs}`) ships fully-flattened per-N specialisations for `N ∈ {0, 1, 2, 3}` and dispatches via `comptime!(n == K)` if-chains. Plan 06-00 Task 1 extends this pattern to `N ∈ {4, 5, 6}`.

**When to use:** Any `#[cube] fn` operating on `CTaylor<F, N>` where N is comptime — this is mandatory for cubecl 0.10-pre.3 since it does not support comptime for-loop unroll over a `#[comptime] u32`.

**Algorithmic recipe for N=4 (analogous to existing N=3 at `ctaylor_rec/multo.rs:114-119`):**

The C++ general recursion is:
```cpp
// ctaylor.hpp:55-65
static void multo(T * dst, const T * y) {
  ctaylor_rec<T, Nvar - 1>::multo(dst + POW2(Nvar - 1), y);
  ctaylor_rec<T, Nvar - 1>::mul  (dst + POW2(Nvar - 1), dst, y + POW2(Nvar - 1));
  ctaylor_rec<T, Nvar - 1>::multo(dst, y);
}
```

For N=4, `POW2(3) = 8`, so:
```rust
#[cube]
pub(crate) fn ctaylor_multo_n4<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    // Step A: multo on upper half [8..16] using y[0..8]
    //   (existing ctaylor_multo_n3 working on the upper-half slice)
    // Step B: mul-accumulate cross-term [8..16] += dst[0..8] * y[8..16]
    //   (existing ctaylor_mul_n3 acting on dst[0..8] · y[8..16] writes to [8..16])
    // Step C: multo on lower half [0..8] using y[0..8]
    //   (existing ctaylor_multo_n3 on the lower-half slice)
}
```

**Cubecl 0.10-pre.3 constraint:** cubecl `Array<F>` does not support sub-slicing in `#[cube]` bodies (verified by inspection of `crates/xcfun-ad/src/ctaylor_rec/multo.rs:74-112` N=2 case which captures all 4 dst values into local `let` bindings before any writes). The N=4 port must follow the same pattern: capture all 16 dst values into `let d0..d15`, then write all 16 in C++ descending order.

**Source citation:** `xcfun-master/external/upstream/taylor/ctaylor.hpp` general-recursion `:55-65` (multo) + `:72-82` (compose). Existing N=2 pattern at `crates/xcfun-ad/src/ctaylor_rec/multo.rs:74-112` is the template — N=4 is 4 such code blocks (one per "octant" pair), ~250 LOC; N=5/6 are ~500/~1000 LOC each. Macro-generation is **strongly recommended** (CONTEXT.md Discretion: "macro-generated if repetitive") to keep maintenance feasible.

### Pattern 2: Pre-allocated reusable handle in `Functional`

**What:** Phase 5 D-13 forwarded the cubecl-cpu `create_from_slice` per-launch cost (~287 allocs/eval) to Phase 6. The fix is a private mutable buffer trio inside `Functional` that's sized at `eval_setup` time and reused across all subsequent `eval` calls.

**When to use:** Always — RS-07 zero-alloc-on-eval is a strict contract; the Phase 5 form-(b) "head/tail mean stability" was a temporary hack.

**Example (Plan 06-06 Task 1):**

```rust
// crates/xcfun-rs/src/functional.rs
use std::cell::UnsafeCell;
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;

pub struct Functional {
    inner: xcfun_eval::Functional,
    /// Reusable per-launch buffers. Initialised at eval_setup time.
    /// `UnsafeCell` because eval(&self, ...) must not require &mut self
    /// (RS-07 + RS-10 contract). Concurrent eval on the same Functional
    /// is RACY — clone or wrap in Mutex for thread-shared use.
    handle: UnsafeCell<EvalHandle>,
}

struct EvalHandle {
    /// cubecl input buffer; capacity = max(input_buffer_length over all set/eval_setup calls)
    input_buf: Option<cubecl::Handle>,    // or `Box<[f64]>` mirror — implementer choice
    /// cubecl output buffer; same capacity policy
    out_buf: Option<cubecl::Handle>,
    /// Generation counter for D-15 weights re-upload skip
    gen: u64,
}

unsafe impl Send for Functional {}
unsafe impl Sync for Functional {}  // documented: racy if shared concurrently
```

**Source citation:** Phase 5 D-13 forward (`05-CONTEXT.md`); Phase 6 D-12. The `unsafe impl Sync` carries a doc-comment matching CONTEXT.md exact phrasing: "Functional is Send + Sync, but eval() is racy if called concurrently on the same instance — clone the Functional or wrap in Mutex for concurrent eval."

### Pattern 3: Generation-counter-guarded buffer pool

**What:** The `weights_buf` (82 f64) is a tiny but recurring upload. The hash-vs-counter trade-off: hashing 656 bytes per launch is cheap on CPU (~50ns) but represents 1-2% of GPU launch overhead at small batch sizes. A monotonic `u64` counter is O(1) and zero-bytes.

**When to use:** Inside `Batch::launch` — check `Functional.settings_gen != batch.cached_gen` and re-upload + bump only when stale.

**Example (Plan 06-02 Task 4):**

```rust
// xcfun-eval::Functional gains:
pub fn settings_generation(&self) -> u64 { self.settings_gen }

impl Functional {
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        // ... existing body ...
        self.settings_gen = self.settings_gen.wrapping_add(1);  // NEW
        Ok(())
    }
}

// xcfun-gpu::Batch:
pub struct Batch<'fun, R: cubecl::Runtime> {
    fun: &'fun xcfun_rs::Functional,
    client: cubecl::ComputeClient<R::Server, R::Channel>,
    weights_buf:    R::Handle,
    active_ids_buf: R::Handle,
    density_buf:    R::Handle,           // grows powers-of-two
    result_buf:     R::Handle,           // grows powers-of-two
    capacity:       usize,
    cached_gen:     u64,
}

impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
    pub fn launch(&mut self, nr_points: u32) -> Result<(), XcError> {
        // D-15: re-upload weights only when stale.
        if self.fun.inner_settings_gen() != self.cached_gen {
            self.client.write(&self.weights_buf, bytemuck::cast_slice(&self.fun.inner_settings()));
            self.cached_gen = self.fun.inner_settings_gen();
        }
        // ... launch_unchecked dispatch ...
        Ok(())
    }

    fn ensure_capacity(&mut self, nr_points: usize) {
        if nr_points > self.capacity {
            // Powers-of-two doubling per D-15.
            let mut new_cap = self.capacity.max(64);
            while new_cap < nr_points { new_cap *= 2; }
            let input_len  = self.fun.input_buffer_length();
            let output_len = self.fun.output_length().unwrap();
            self.density_buf = self.client.empty(new_cap * input_len  * 8);
            self.result_buf  = self.client.empty(new_cap * output_len * 8);
            self.capacity = new_cap;
        }
    }
}
```

**Source citation:** `docs/design/06-cubecl-strategy.md §5.1, §7`; D-15.

### Pattern 4: `auto_backend()` as runtime-typed dispatch with monomorphisation

**What:** Cubecl's `Runtime` trait carries associated types (`Device`, `Server`, `Channel`, `Compiler`, etc.) that make `Box<dyn Runtime>` impractical (`dyn` requires object-safety; cubecl's trait isn't designed for it). Instead: `Backend` enum is the *runtime-discriminator*; the actual `Batch<R>` instantiations are monomorphised via per-arm dispatch.

**When to use:** Always — any code that wants to "pick a runtime at runtime" must enum-dispatch and monomorphise the kernel launch path per arm.

**Example (Plan 06-02 Task 2):**

```rust
// crates/xcfun-gpu/src/backend.rs
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Backend {
    Cpu,
    Rocm,    // cubecl-hip (D-05 primary)
    Cuda,    // cubecl-cuda
    Metal,   // cubecl-wgpu Metal backend
    Wgpu,    // cubecl-wgpu generic (Vulkan/DX12/WebGPU)
}

pub fn auto_backend() -> Backend {
    // D-07 priority order.
    if let Ok(force) = std::env::var("XCFUN_FORCE_BACKEND") {
        return parse_force(&force).expect("XCFUN_FORCE_BACKEND unrecognised");
    }
    #[cfg(feature = "hip")]
    if rocm_available() { return Backend::Rocm; }
    #[cfg(feature = "cuda")]
    if cuda_available() { return Backend::Cuda; }
    #[cfg(feature = "metal")]
    if metal_with_f64_available() { return Backend::Metal; }
    #[cfg(feature = "wgpu")]
    if wgpu_with_shader_f64_available() { return Backend::Wgpu; }
    Backend::Cpu
}

// crates/xcfun-rs/src/functional.rs eval_vec dispatcher:
pub fn eval_vec(&self, density: &[f64], density_pitch: usize,
                out: &mut [f64], out_pitch: usize, nr_points: usize)
                -> Result<(), XcError> {
    if nr_points < self.threshold() { return self.eval_loop_fallback(...); }
    match xcfun_gpu::auto_backend() {
        Backend::Cpu => xcfun_gpu::Batch::<CpuRuntime>::eval_vec_host(
                            self, density, density_pitch, out, out_pitch, nr_points),
        #[cfg(feature = "hip")]
        Backend::Rocm => xcfun_gpu::Batch::<HipRuntime>::eval_vec_host(...),
        #[cfg(feature = "cuda")]
        Backend::Cuda => xcfun_gpu::Batch::<CudaRuntime>::eval_vec_host(...),
        #[cfg(feature = "wgpu")]
        Backend::Metal | Backend::Wgpu =>
            xcfun_gpu::Batch::<WgpuRuntime>::eval_vec_host(...),
    }
}
```

**Source citation:** Cubecl-book design — `Runtime: 'static + Send + Sync` with associated type members. Verified pattern in `cubecl_3d_dft.md` (project's vendored manual) using `R: Runtime` generic + per-runtime `client = R::client(&device)`. CubeCL examples consistently monomorphise: `cargo run --example gelu --features cpu`, `--features cuda`, `--features hip` — one binary per feature combination, not one binary dispatching at runtime.

### Pattern 5: Comptime branchless select for `tau ≥ tau_w` guard

**What:** D-10 calls for `tau_clamped = ctaylor_max(tau, tau_w)` inside TPSSC/TPSSLOCC/REVTPSSC kernel bodies. The `ctaylor_max` helper already exists at `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818`. Phase 6 just *uses* it at the top of each TPSS-correlation kernel.

**When to use:** TPSSC, TPSSLOCC, REVTPSSC kernel bodies (3 places). Possibly also a future hardening pass for any other `tau`-using metaGGA hitting the same regime.

**Example (Plan 06-00 Task 3):**

```rust
// crates/xcfun-eval/src/functionals/mgga/tpssc.rs (or post-06-01 location)
#[cube]
pub fn tpssc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Phase 6 D-10: hard-clamp tau to tau_w to guard the unphysical regime
    // where von Weizsäcker bound is violated by f64-rounding cancellation.
    // tau_w = |∇ρ|² / (8 ρ); both d.tau and tau_w are CTaylor — ctaylor_max
    // compares CNST slot only (Phase 4 Plan 04-10 Path-B finding) and outputs
    // the larger arg's full coefficient set. Bit-exact when tau ≥ tau_w
    // (the physical regime); diverges from C++ only in the unphysical regime
    // where ACC-04 mpmath amendment (D-03) substitutes ground truth.
    let size = comptime!((1_u32 << n) as usize);
    let mut tau_w = Array::<F>::new(size);
    build_tau_w::<F>(d, &mut tau_w, n);  // |∇ρ|² / (8 ρ)
    let mut tau_clamped = Array::<F>::new(size);
    ctaylor_max::<F>(&d.tau, &tau_w, &mut tau_clamped, n);
    // ... use `tau_clamped` everywhere `d.tau` was used downstream ...
}
```

**Source citation:** `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:814-838` — the existing `ctaylor_max` function. Phase 4 Plan 04-10 Path-B bisection note in `04-CONTEXT.md` D-19. **No new `ctaylor_max` work is needed** — the helper compares `a[0] >= b[0]` and copies the full coefficient set of the winner; this is exactly the C++ `operator>` on CNST-slot semantics.

### Pattern 6: mpmath sidecar via Python subprocess

**What:** D-04 + D-03. The mpmath fixture generator runs offline as a Python script invoked by `xtask`. It NEVER becomes a runtime/library dep — `cargo build` of any `xcfun-*` crate must NOT require Python.

**When to use:** Plan 06-00 Task 4 (initial fixture build); Plan 06-N2 (20 `excluded_by_upstream_spec` functionals); ad-hoc when a new ACC-04 amendment-eligible point is identified.

**Example layout:**

```
xtask/
├── src/bin/regen_mpmath_fixtures.rs  # Rust driver — invokes the Python sidecar
└── mpmath_eval/                       # Python package
    ├── __init__.py
    ├── __main__.py                    # `python3 -m xtask.mpmath_eval --fn ldaerfx ...`
    ├── functionals.py                 # mpmath-precision ports
    │   # def eval_ldaerfx(rho_a, rho_b, prec=200) -> dict
    │   # def eval_tpssc(...) -> dict
    └── README.md                      # how to install + reproducibility
```

**JSONL schema (one record per density point):**

```json
{
  "functional": "ldaerfx",
  "vars": "XC_A_B",
  "mode": "PartialDerivatives",
  "order": 3,
  "input": [0.5, 0.5],
  "output": [-0.0123, 0.456, ...],
  "mpmath_prec": 200,
  "source": "mpmath",
  "rationale": "C++ ldaerfx.cpp:66 documents bracket cancellation"
}
```

**Subprocess interface (Plan 06-00 Task 4 — `xtask` driver):**

```rust
// xtask/src/bin/regen_mpmath_fixtures.rs
let output = std::process::Command::new("python3")
    .arg("-m").arg("xtask.mpmath_eval")
    .arg("--fn").arg("ldaerfx")
    .arg("--input").arg("0.5,0.5")
    .arg("--order").arg("3")
    .arg("--prec").arg("200")
    .current_dir(workspace_root)
    .output()?;
let record: MpmathRecord = serde_json::from_slice(&output.stdout)?;
```

**Reproducibility:** mpmath at 200-digit precision is stable across decades. Document `python3 -V` (≥ 3.10) + `mpmath.__version__` (≥ 1.4) in `xtask/mpmath_eval/README.md`. The local env has `Python 3.14.4 + mpmath 1.4.1` (verified Step 2.6 audit) — adequate.

**Cost:** mpmath is ~1000× slower than f64. 10k records × 78 functionals = unreasonable. **Selective fixture set:** ~100 records per ACC-04-amended functional (so ~1000 total records); strata sampled to cover the cancellation regime explicitly. This is the right cost-vs-coverage trade-off per CONTEXT.md "Specific Ideas".

**Source citation:** D-03 / D-04. Phase 2 already validated mpmath-vs-Rust agreement at LDAERFX (2026-04-21 finding referenced in `02-CONTEXT.md`).

### Anti-Patterns to Avoid

- **`Box<dyn cubecl::Runtime>` for type erasure** — cubecl's `Runtime` trait is not object-safe (associated types `Device`, `Server`, `Channel`). Use enum-dispatch with monomorphised `Batch<R>`.
- **Per-launch `client.create_from_slice`** — re-introduces the ~287 allocs/eval cost Phase 5 D-13 was forwarded for. Always use `client.write(&handle, ...)` against pre-allocated handles.
- **`#[cube(fast_math = ...)]`** — explicitly forbidden by ACC-05 / `xtask check-no-fma`. CubeCL's `FastMath` flags trade precision for speed (CUDA `__frsqrt_rn` etc.). All Phase-6 kernel bodies and AD primitives must remain `fast_math`-free.
- **Hash-based weights cache invalidation** — rejected per D-15 (hashing 656 bytes adds 1-2% per launch overhead at small batch sizes). Use generation counter.
- **`cubecl-metal` as a Cargo dep** — does not exist on crates.io. Metal access is via `cubecl-wgpu` Metal backend.
- **`launch_async` / stream overlap** — out-of-scope per PROJECT.md (deferred to v2).
- **`Line<F>` vectorisation** — `docs/design/06 §8` defers to v2; per-thread one-grid-point keeps register pressure manageable.
- **f32 anywhere on the numerical path** — CLAUDE.md hard constraint.
- **`mul_add(...)` calls** — CI-blocked by `xtask check-no-mul-add`. Phase 6 ports must keep accumulation order matching C++.
- **`tau_w` clamp via `regularize` style on tau directly** — `regularize` per CORE-06 / D-22 mutates only `c[CNST]`. The TPSS guard wants the larger of two CTaylors, not a CNST-floor. Use `ctaylor_max`.
- **Adopting cubecl `0.10.0-pre.4`** — newer pre-release available, but D-02 / D-04 lockstep contract says all four runtime crates move together AND tier-3 must be re-validated. Defer.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Multi-runtime kernel dispatch | Custom CUDA + WGSL + ROCm shader sources | `cubecl 0.10-pre.3` `#[cube]` + `Runtime` trait | Single-source contract; already in workspace |
| f64 device-feature probing | Custom `wgpu::Features::SHADER_F64` parsing | `client.properties().feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))` | cubecl-book primary API; verified via Context7 |
| Buffer growth strategy | Custom doubling allocator | cubecl `client.empty(N * size_of::<f64>())` + tracking | Already idiomatic |
| AD `compose`/`multo` for N=4..6 | Generic recursive cubecl-`Array` indexing | Hand-written per-N specialisations (consistent with N=0..3) | Cubecl 0.10-pre.3 cannot unroll a comptime for-loop over a comptime u32 |
| `ctaylor_max` semantics | New "max on CNST then propagate" code | Existing `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818` | Already shipped Phase 4 |
| `erf` Taylor expansion | Per-N hand-written branches | `erf_expand` (existing `xcfun-ad/src/expand/erf.rs:321`) + new wrapper using stable bracket reduction | Phase 2 already shipped 1e-14 baseline |
| mpmath integration | Custom 200-digit float arithmetic in Rust | Python sidecar via `subprocess::Command` | mpmath is the de facto standard for arbitrary-precision real arithmetic; already verified at LDAERFX |
| C ABI signature for `xcfun_eval_vec` | Custom pitched-array layout | Match `xcfun-master/api/xcfun.h:54` byte-for-byte | CAPI-01 / CAPI-02 drop-in contract |
| Generation counter | Hash-based cache invalidation | `wrapping_add(1)` u64 monotonic counter | D-15 explicit |
| `Send + Sync` for the reusable handle | Custom lock-free buffer | `UnsafeCell<EvalHandle>` + documented "racy if shared" | Standard pattern; matches Phase 5 D-17 / Phase 6 D-12 |

**Key insight:** The Phase 6 surface is dominated by integration work — wiring already-shipped substrate (cubecl, `xcfun-ad`, `xcfun-eval`, `Functional` facade) through new crates and feature flags. The only genuinely new algorithmic work is (a) AD `N≥4` recursion specialisations, (b) `erf_precise_taylor` AD-chain wrapper, and (c) mpmath ground-truth fixtures. Everything else is plumbing.

## Runtime State Inventory (Plan 06-01 git-mv impact)

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — there is no runtime database / Mem0 / Redis use in this project | None |
| Live service config | None — pure Rust workspace, no n8n / Datadog / Cloudflare | None |
| OS-registered state | None — no Task Scheduler / launchd / systemd entries reference `xcfun-eval` paths | None |
| Secrets / env vars | `XCFUN_FORCE_BACKEND` (NEW Phase 6) + `XCFUN_MIN_BATCH_SIZE` (NEW Phase 6) — env-only, read at runtime; no SOPS/.env files | Document in `xcfun-gpu/README.md` |
| Build artifacts / installed packages | After Plan 06-01 `git mv`: existing `target/debug/build/xcfun-eval-*` cache directories will be stale; existing `target/doc/xcfun_eval/functionals/` symlinks → broken | `cargo clean -p xcfun-eval && cargo clean -p xcfun-kernels` after the rename. Document in 06-01-SUMMARY.md. |

**Plan 06-01 git-mv side effects requiring code edits:**
- Every test in `crates/xcfun-eval/tests/*.rs` that imports `xcfun_eval::functionals::lda::*_kernel` → switch to `xcfun_kernels::functionals::lda::*_kernel` (path rename only).
- `crates/xcfun-rs/src/functional.rs` and `crates/xcfun-eval/src/functional.rs` continue to depend on `xcfun-eval`; the path that used to read `crate::functionals::*` becomes `xcfun_kernels::functionals::*`.
- `validation/src/driver.rs` `run_launch` → `xcfun_kernels::dispatch::run_launch`.
- `xtask` regen-registry generated `descriptors.rs` does NOT contain function-pointer references (Phase 2 D-04 invariant — descriptors hold metadata only), so no xtask path edit is needed.
- `crates/xcfun-eval/src/for_tests.rs::cpu_client()` is consumed in production by Plan 06-06 (D-12 promotion); the path stays the same.

## Common Pitfalls

### Pitfall 1: cubecl-runtime version drift across the four runtime crates

**What goes wrong:** Cubecl pre-release crates cross-reference internal types (`cubecl_runtime::server::ComputeServer`, `cubecl_ir::Item`, etc.). A `cubecl-hip 0.10.0-pre.3` paired with `cubecl 0.10.0-pre.4` produces an opaque "type X1 is not the same as type X2" compiler error.
**Why it happens:** cargo's caret-resolver cheerfully picks the newest matching pre-release for any non-`=` constraint.
**How to avoid:** Pin all five (`cubecl`, `cubecl-cpu`, `cubecl-hip`, `cubecl-cuda`, `cubecl-wgpu`) at exactly `=0.10.0-pre.3`. Extend `xtask check-cubecl-pin` (Plan 02-02 pattern) to enforce all five lockstep. Run on every PR.
**Warning signs:** Build error mentioning two `Item`/`Element`/`ComputeServer` types from "different versions".

### Pitfall 2: Wgpu silently downgrading to f32 on Apple Silicon / WGSL

**What goes wrong:** `wgpu::Features::SHADER_F64` is reported `false` on M1/M2/M3, on most consumer Vulkan drivers without f64 ext, and on all WebGPU. cubecl-wgpu compiles the `Float` trait to f32 in those cases, silently breaking the 1e-12 contract.
**Why it happens:** No compile-time check; the trait monomorphises to whatever the device says.
**How to avoid:** D-13 typed `XcError::WgpuNoF64` + compile-time `const _: () = assert!(size_of::<Scalar>() == 8);` in `crates/xcfun-kernels/src/lib.rs`. Runtime probe via `client.properties().feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))` at `Batch::open`.
**Warning signs:** Wgpu adapter info string "Apple M1 Pro" / "Apple M2"; wgpu device with no `SHADER_F64` flag in `Features::SUPPORTED`.

### Pitfall 3: ROCm RDNA-2 silent failure without HSA_OVERRIDE_GFX_VERSION

**What goes wrong:** RDNA-2 GPUs (RX 6000-series) are not officially supported by ROCm. cubecl-hip lights up but kernels fail with cryptic "code object load failed" at first launch.
**Why it happens:** AMD's HIP runtime requires `HSA_OVERRIDE_GFX_VERSION=10.3.0` to coerce-match RDNA-2 to RDNA-3 PTX.
**How to avoid:** Document in `crates/xcfun-gpu/README.md` (D-05 explicit). CI scripts must `export HSA_OVERRIDE_GFX_VERSION=10.3.0` before any tier-3 sweep.
**Warning signs:** First-launch failure with "kernel module not found" or "code object load failed"; `rocminfo` showing `gfx1031` / `gfx1032` / `gfx1033` in the architecture line.
**Source citation:** [VERIFIED via Context7 + GitHub README] `crates/cubecl-hip/README.md` upstream RDNA-2 note.

### Pitfall 4: Cubecl `0.10-pre.X` API drift between pre-releases

**What goes wrong:** Pre-release semver is not respected. Internal IR types churn between e.g. `pre.3` and `pre.4` (verified: `cubecl 0.10.0-pre.4` is currently the head — one minor bump beyond the project's pin).
**Why it happens:** Pre-releases are explicitly unstable.
**How to avoid:** Hard pin (`=0.10.0-pre.3`); when a future bump is required, the bump itself is a sub-phase that re-runs full tier-2 + tier-3 across all 78 functionals (CLAUDE.md Pitfall P8).
**Warning signs:** A `cargo update` PR that touches `Cargo.lock` cubecl entries — block these unless the bump is explicitly planned.

### Pitfall 5: f64 expansion on cubecl-wgpu WGSL

**What goes wrong:** Even when `SHADER_F64` is reported by the device, WGSL has no f64 type — wgpu-WGSL emits 32-bit code regardless. Only the SPIR-V backend honours f64 properly.
**Why it happens:** WebGPU's WGSL spec deliberately omits f64.
**How to avoid:** `Wgpu` backend on the validation envelope is **1e-9 only, NEVER 1e-13** (already encoded). For the strict 1e-13 ROCm-primary contract, never route to Wgpu without explicit ERF-fallback already routing to CPU.
**Warning signs:** Documented Wgpu f64 row "?" in cubecl-book features.md (already cited in CLAUDE.md).

### Pitfall 6: `tau_w` arithmetic underflow in extreme regularize regime

**What goes wrong:** `tau_w = |∇ρ|² / (8 ρ)`; when ρ ~ 1e-14 (regularize floor), tau_w ~ 1e+27 if |∇ρ|² is sizable. f64 is fine, but Plan 06-00 needs to verify no intermediate `1.0 / ρ` overflows before the `ctaylor_max(tau, tau_w)` clamp.
**Why it happens:** Regularize stratum is by design the most numerically-fragile region.
**How to avoid:** Test the guard against the existing 1000-record "regularize stratum" in `validation/src/fixtures.rs`.
**Warning signs:** Plan 04-10 already saw `tauwtau3 ≈ 1e+27` — this is the known fingerprint.

### Pitfall 7: `ctaylor_compose_n4` recursion divergence vs. C++

**What goes wrong:** The C++ outer dispatch at `ctaylor.hpp:55-65` (multo) and `:72-82` (compose) descends `multo_skipconst` first, accumulates, recurses on the lower half. A subtle reordering of any of these three steps loses 1e-12 parity instantly.
**Why it happens:** N=4 is the first level where the recursion is no longer "tiny enough to inspect" — N=3 has 8 coefficients; N=4 has 16; N=5 has 32; N=6 has 64.
**How to avoid:** Generate the per-N body via a build.rs / proc-macro that emits the C++ recursion-equivalent structurally; OR write each N=4/5/6 body by hand and gate it against a fixture sweep at orders 3..6 where N is reachable. Plan 06-00 Task 1 should land golden fixtures FIRST (proptest at 1e-12 against a Rust-host reference, then a C++ extractor cross-check) BEFORE any kernel body uses the new specialisations.
**Warning signs:** Mode::Contracted orders 5..=6 metaGGA tests no longer return zeros (good) but produce non-zero residuals at >1e-12 (regression).

### Pitfall 8: Box::leak preventing the zero-alloc strict-form gate

**What goes wrong:** Phase 5 D-13 documented "fall-back form (b)" head/tail mean stability — meaning the test asserts mean delta == 0 over many calls, allowing one-time setup allocations. Plan 06-06's strict form (delta == 0 every call) breaks if any sub-path still calls `Box::leak` or `Vec::new`.
**Why it happens:** D-17 retires the `weights: &'static [...]` Box::leak in `xcfun-rs::sync_weights_from_settings` (line ~196). But there are likely still sub-paths in `xcfun-eval::Functional::eval` and the cubecl launch loop that do per-iteration `Vec` allocation.
**How to avoid:** Plan 06-06 must include a `dhat` or counting-allocator full-trace sweep, not just a delta-check; identify and remove every per-eval `Vec::new` / `Box::new`. This is the largest "unknown unknown" of the phase.
**Warning signs:** Counting allocator delta > 0 on the strict form test.

### Pitfall 9: `cubecl-metal` does not exist; D-06 phrasing needs correction

**What goes wrong:** CONTEXT.md D-06 references `cubecl-metal = "=0.10.0-pre.3"`. There is no such crate. Metal is reached via `cubecl-wgpu` Metal backend.
**Why it happens:** Reasonable assumption from the design-doc-06 §2 "Wgpu — Vulkan/Metal/WebGPU" phrasing; the table cell labelled "Metal" was treated as a separate runtime, but it's a backend within wgpu.
**How to avoid:** Plan 06-04 `Cargo.toml` declares `cubecl-wgpu` only (not `cubecl-metal`). The `metal` feature in `xcfun-gpu/Cargo.toml` is an alias of `wgpu` (or simply: drop the `metal` feature alias and let users opt into `wgpu` knowing Metal is one of its backends). At Phase 6 sign-off, file an erratum amending CONTEXT.md D-06 wording.
**Warning signs:** `cargo build -p xcfun-gpu --features metal` errors with "no crate cubecl-metal".

### Pitfall 10: `XcError::WgpuNoF64` payload `String` breaks `Copy`

**What goes wrong:** `XcError` is `Copy + non_exhaustive` per Phase 2 D-25. Adding a `String` payload makes it non-Copy.
**Why it happens:** `wgpu::AdapterInfo::name` is `String`. The intuitive variant signature carries that String through.
**How to avoid:** D-13-A — payload is `&'static str` filled via `Box::leak` once at runtime when the panic-on-misconfiguration is constructed. `requested_runtime: Backend` is already `Copy` (enum). Verify with `static_assertions::assert_impl_all!(XcError: Copy);` at compile time.
**Warning signs:** Phase 5 D-25 compile-time `Copy + non_exhaustive` gate fails.

## Code Examples

### Verified pattern: cubecl `client.read_one` + `client.create` lifecycle

```rust
// Source: docs/manual/Cubecl/cubecl_3d_dft.md (vendored)
use cubecl::prelude::*;
use cubecl_cpu::{CpuDevice, CpuRuntime};
use cubecl_runtime::client::ComputeClient;

let device = CpuDevice::default();
let client: ComputeClient<_> = ComputeClient::load(&device);

// Allocate handles
let in_re_handle = client.create(f64::as_bytes(&input_re));     // upload
let out_re_handle = client.empty(N * core::mem::size_of::<f64>()); // empty

// Launch
unsafe {
    fft_x::launch_unchecked::<CpuRuntime>(
        &client,
        cube_count.clone(),
        cube_dim,
        ArrayArg::from_raw_parts::<f64>(&in_re_handle, N, 1),
        ArrayArg::from_raw_parts::<f64>(&out_re_handle, N, 1),
        NX, NY, NZ,
    );
}

// Read back
let out_re_bytes = client.read_one(out_re_handle);
let out_re = f64::from_bytes(&out_re_bytes);
```

### Verified pattern: feature probing for a specific float type

```rust
// Source: Context7 fetch of /tracel-ai/cubecl 2026-04-30
use cubecl::{features::Plane, prelude::*};

pub fn print_features<R: Runtime>(device: &R::Device) {
    let client = R::client(device);
    if client.properties()
             .feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))
    {
        // f64 supported; safe to launch the numerical-path kernel
    } else {
        // device cannot meet the 1e-12 contract — refuse
        return Err(XcError::WgpuNoF64 {
            adapter_name: leak_static_str(adapter_info.name),
            requested_runtime: Backend::Wgpu,
        });
    }
}
```

### Verified pattern: comptime branching for ERF-bearing kernels

```rust
// Phase 6 Plan 06-05 D-13 — ERF auto-fallback wiring at host
use xcfun_core::Dependency;

impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
    pub fn open(fun: &'fun Functional, runtime: Backend) -> Result<Self, XcError> {
        // GPU-05: Wgpu/Metal SHADER_F64 + ERF-bearing functional → CPU fallback
        if matches!(runtime, Backend::Wgpu | Backend::Metal)
            && fun.dependencies().contains(Dependency::ERF)
        {
            // Open a CpuRuntime batch instead — caller's runtime choice is
            // overridden because the 1e-12 numerical contract is non-negotiable.
            return Self::open_cpu(fun);
        }
        // ... normal open path ...
    }
}
```

### Existing pattern: `ctaylor_max` (already shipped — used as-is in TPSS guard)

```rust
// Source: crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818
#[cube]
pub fn ctaylor_max<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    if a[0] >= b[0] {
        #[unroll]
        for i in 0..size { out[i] = a[i]; }
    } else {
        #[unroll]
        for i in 0..size { out[i] = b[i]; }
    }
}
```

### Existing pattern: `OnceLock<R::Client>` (Phase 1 → promoted to production in Plan 06-06)

```rust
// Source: crates/xcfun-eval/src/for_tests.rs (current; Plan 06-06 promotes to prod)
use cubecl::prelude::*;
use cubecl_cpu::{CpuDevice, CpuRuntime};
use std::sync::OnceLock;

pub type CpuClient = ComputeClient<CpuRuntime>;

static CPU_CLIENT: OnceLock<CpuClient> = OnceLock::new();

pub fn cpu_client() -> &'static CpuClient {
    CPU_CLIENT.get_or_init(|| {
        let device = CpuDevice;
        CpuRuntime::client(&device)
    })
}

// Plan 06-02 generalises to per-runtime OnceLocks:
//   pub fn hip_client() -> &'static HipClient { ... }
//   pub fn cuda_client(device_id: usize) -> &'static CudaClient { ... }
//   pub fn wgpu_client() -> &'static WgpuClient { ... }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `cubecl 0.9.0` stable | `cubecl 0.10.0-pre.3` (project pin) | 2026-02 | Pre-release; project hard-pinned, lockstep gate via xtask |
| Pre-cubecl-pivot dual scalar+kernel implementations | Single-source `#[cube] fn` for AD + functional bodies | 2026-04-19 PM | Phase 1 cubecl pivot; Phase 6 just inherits this contract |
| f32 polyfill `Float::erf` | In-kernel libm-port `erf_precise` (FreeBSD msun) | Phase 2 Plan 02-06 commit `dca382a` | 1e-14 baseline; Phase 6 D-11 extends to AD-chain `erf_precise_taylor` |
| `&'static [(FunctionalId, f64)]` weights via `Box::leak` | `Vec<(FunctionalId, f64)>` weights | Phase 6 Plan 06-06 D-17 | Eliminates per-`set` heap leak |
| `cubecl-metal` as a separate crate | `cubecl-wgpu` Metal backend | (never existed as separate crate) | Phase 6 Plan 06-04 D-06 phrasing must amend |

**Deprecated/outdated:**
- The `CTaylorDev<F, N>` separate type (design-doc-06 §3.2) — superseded by the cubecl-pivot using `xcfun-ad::CTaylor<F, N>` directly on any runtime.
- The "scalar host path" mentioned in pre-pivot 06-cubecl-strategy.md §6.4 — superseded by cubecl-cpu being the single per-point validation substrate.

## mpmath Sidecar Architecture

### Python Module Layout

```
xtask/mpmath_eval/
├── __init__.py
├── __main__.py        # python3 -m xtask.mpmath_eval ARGS
├── functionals/
│   ├── __init__.py
│   ├── ldaerfx.py    # mpmath-native port at prec=200
│   ├── ldaerfc.py
│   ├── ldaerfc_jt.py
│   ├── tpssc.py      # for D-10 boundary verification
│   ├── tpsslocc.py
│   ├── revtpssc.py
│   ├── br.py         # for the 20 excluded_by_upstream_spec set
│   ├── csc.py
│   ├── blocx.py
│   ├── scan.py
│   ├── tw.py
│   ├── vwk.py
│   ├── pbelocc.py
│   ├── zvpbesolc.py
│   └── zvpbeintc.py
├── ad_chain.py        # generic Taylor-series AD chain at prec=200
├── densvars.py        # DensVars equivalent at prec=200
└── README.md
```

### Subprocess Interface

**Driver (Rust):**

```rust
// xtask/src/bin/regen_mpmath_fixtures.rs
let output = std::process::Command::new("python3")
    .args(["-m", "xtask.mpmath_eval", "--functional", "ldaerfx",
           "--vars", "XC_A_B", "--mode", "PartialDerivatives",
           "--order", "3", "--input", "0.5,0.5", "--prec", "200"])
    .current_dir(workspace_root)
    .output()?;
let record: MpmathRecord = serde_json::from_slice(&output.stdout)?;
```

**Sidecar (Python `__main__.py`):**

```python
# xtask/mpmath_eval/__main__.py
import argparse, json, mpmath, sys
from .functionals import LOOKUP

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--functional', required=True)
    p.add_argument('--vars', required=True)
    p.add_argument('--mode', required=True)
    p.add_argument('--order', type=int, required=True)
    p.add_argument('--input', required=True)  # comma-separated
    p.add_argument('--prec', type=int, default=200)
    args = p.parse_args()

    mpmath.mp.prec = args.prec
    inputs = [mpmath.mpf(x) for x in args.input.split(',')]
    fn = LOOKUP[args.functional.lower()]
    output = fn(inputs, vars=args.vars, mode=args.mode, order=args.order)

    record = {
        'functional': args.functional,
        'vars': args.vars,
        'mode': args.mode,
        'order': args.order,
        'input': [float(x) for x in inputs],
        'output': [float(x) for x in output],
        'mpmath_prec': args.prec,
        'source': 'mpmath',
    }
    json.dump(record, sys.stdout)
    sys.stdout.write('\n')

if __name__ == '__main__':
    main()
```

### JSONL Schema (validation/fixtures/mpmath/<functional>.jsonl)

```json
{"functional":"ldaerfx","vars":"XC_A_B","mode":"PartialDerivatives","order":3,"input":[0.5,0.5],"output":[-0.0123456789,0.456,...],"mpmath_prec":200,"source":"mpmath","rationale":"C++ ldaerfx.cpp:66 documents bracket cancellation"}
```

### Reproducibility

- Python: ≥ 3.10 (project local: 3.14.4 — verified)
- mpmath: ≥ 1.4 (project local: 1.4.1 — verified)
- Document in `xtask/mpmath_eval/README.md`. Pin in CI install step (`pip install 'mpmath>=1.4,<2.0'`).
- mpmath at prec=200 is bit-stable across decades — validated regression. The fixtures are committed to git; regeneration via `cargo xtask regen-mpmath-fixtures --check` (mirrors Phase 2 D-21 pattern).

## D-19 Bisection Methodology

The 30+ Phase-3/4 D-19 forwards split into three categories, addressed in three plans:

### Plan 06-N1 — Inherited Phase-3 forwards still failing at order 3 (~11 entries)

Targets: PBEINTC `6.2e+1`, BECKESRX `2.3e+2`, P86C/P86CORRC `9.2e-2`, PW91C `1.7e-3`, SPBEC `5.3e-4`, APBEC `5.7e-9`, B97{,_1,_2}C `7.8e-11`, PW91K `1.4e-11`.

**Methodology:**

1. **Substrate-first hypothesis.** D-10 (TPSS guard), D-11 (libm-hybrid erf), and Plan 06-00 N≥4 specialisations may incidentally tighten unrelated functionals. Re-run tier-2 at order 3 AFTER Plan 06-00 lands but BEFORE 06-N1 starts; document which forwards self-resolved (Plan 04-10 already showed PW86X + APBEX self-resolved this way at order 3).

2. **Path-B side-by-side reads (per Plan 04-10 finding).** For each persistent forward, open `xcfun-master/src/functionals/<name>.cpp` and `crates/xcfun-eval/src/functionals/<tier>/<name>.rs` (or post-06-01 location) side-by-side; trace the first divergence:
   - Look for accidental re-parenthesisation (most common root cause per `docs/design/07 §7`).
   - Check for missing `mul`-vs-`multo` distinction (mul allocates, multo is in-place; algorithmic-identity requires the same form as C++).
   - Look for fp-contract / FMA risk (xtask check-no-fma should already block, but verify).
   - For correlation kernels: check the eps_pbe vs eps_pbe_polarized chain — Phase 4 Plan 04-10 found that summax decomposition is line-for-line correct but f64-rounding cancellation regions need stratum-specific handling.

3. **Root-cause pattern recognition.** Phase 3/4 history shows shared-helper port-order bugs propagate to multiple functionals (Plan 03-05's `build_xc_a_b_2nd_taylor` fix tightened LYPC + others). Expect that fixing one root cause tightens 3-5 functionals at once.

4. **Per-functional stratum exclusion as last resort.** Per D-02, the bar is strict 1e-13 across all 78 — but in extremis a CONTEXT.md amendment can authorise a per-functional D-24-style override (Phase 2 LDAERF 1e-7 precedent). User approval required.

### Plan 06-N2 — 20 `excluded_by_upstream_spec` (mpmath-only)

Targets: BR×3 (BRX/BRC/BRXC), CSC, BLOCX, SCAN×10, TW, VWK, PBELOCC, ZVPBESOLC, ZVPBEINTC.

**Methodology:**

1. **mpmath sidecar is the sole reference.** C++ harness aborts on these (`tmath::sqrt_expand`/`log_expand`/`pow_expand` aborts at low-density tail). Plan 06-N2 generates mpmath JSONL fixtures for each functional × stratum.

2. **Validation harness extension.** `validation/src/driver.rs` gains `--reference {cpp, mpmath}` CLI flag. For excluded_by_upstream_spec entries, the reference defaults to mpmath. Per-record source annotated in `report.html`.

3. **Test cost.** ~100 records per functional × 20 functionals = 2000 mpmath evaluations. At ~10s each at prec=200, this is ~6 hours of one-time fixture generation. Run once; commit; replay forever.

### Plan 06-N3 — Post-libm-hybrid sweep (~12 small-magnitude forwards)

Targets: M05/M06 family small-magnitude (1.5e-12 to 6.3e-11), B97{X,_1X,_2X} 9.5e-12, LYPC 1.3e-10, VWN_PBEC 6.9e-9, PW92C 9.0e-12, PBEC 1.8e-12, OPTX 1.2e-12, M06HFX 7.8e-12.

**Methodology:**

1. After Plan 06-00 lands D-11 `erf_precise_taylor`, run a full tier-2 sweep at order 3.
2. Bisect any that didn't tighten. Hypothesis (per CONTEXT.md "Specifics"): same erf-bracket-cancellation pattern as LDAERF, but lower-amplitude — the libm-hybrid wrapper should automatically fix these.
3. If a residual remains: Path-B side-by-side read, same as 06-N1.

## Risks & Open Questions

### R-01: Local environment lacks ROCm — tier-3 ROCm-primary cannot be validated locally

**Severity:** HIGH (user's primary backend per D-05)
**Detail:** Step 2.6 audit shows `/opt/rocm` does NOT exist on this host. `hipconfig` is present but resolves to a stub (`which hipconfig` succeeded but the install dir is missing). The user's CONTEXT.md memory file `project_gpu_target.md` says "user's dev env is AMD" — but ROCm runtime is not installed.
**Mitigation:** Plan 06-03 (cubecl-hip wiring) ships the code path; a separate "Quick Task" must install ROCm OR the validation runs on a remote/cloud-CI ROCm runner. Block Phase 6 sign-off on "ROCm tier-3 GREEN" — but acknowledge that "GREEN" may be observed via a non-local runner. Document the install path in `xcfun-gpu/README.md` (rocm.docs.amd.com Quick Start).

### R-02: cubecl-metal does not exist; D-06 phrasing is incorrect

**Severity:** MEDIUM (resolved by acknowledging Metal-via-Wgpu)
**Detail:** Verified via crates.io API — `cubecl-metal` returns 404. CONTEXT.md D-06 mentions `cubecl-metal = "=0.10.0-pre.3"` which is unbuildable.
**Mitigation:** Plan 06-04 declares `cubecl-wgpu` only; the `metal` feature in `xcfun-gpu` is either an alias of `wgpu` or dropped entirely. Amend CONTEXT.md D-06 at sign-off (note also flags this for the discuss-phase log).

### R-03: AD `N≥4` recursion — algorithmic identity vs. f64-rounding cancellation

**Severity:** MEDIUM
**Detail:** Plan 06-00 Task 1 ports the C++ recursion verbatim; the recursion structure is identical to N=2/3 (already shipping). But N=4 has 16 coefficients per dst, which means each output coefficient is the sum of up to 16 product terms. f64 rounding cancellation in this many-term accumulation could exceed 1e-12 — the typical `~ 50-200 ops × 0.5 ULP = ~ 1-2e-13` budget per design-doc-07 §6 still holds, BUT the operation count grows with N.
**Mitigation:** Tier-1 fixture-driven test BEFORE any kernel uses N=4: golden_compose_n4 / golden_multo_n4 with at least 100 test inputs at prec=200 mpmath ground truth, with rel-err threshold of 1e-13. Block Plan 06-01 on tier-1 GREEN. Failure here means rethinking the contract for orders 5..6 metaGGA.

### R-04: cubecl 0.10-pre.3 vs. 0.10-pre.4 drift

**Severity:** LOW (handled by hard pin)
**Detail:** The pre-release at HEAD is `0.10.0-pre.4`; project pinned at `0.10.0-pre.3`. Pre-release semver is unstable.
**Mitigation:** Hold the `=0.10.0-pre.3` pin until cubecl `0.10` ships stable, then bump as a sub-phase per CLAUDE.md Pitfall P8. Verified that all four runtime crates exist at `=0.10.0-pre.3` on crates.io.

### R-05: Apple Silicon adapter info Box::leak risk

**Severity:** LOW
**Detail:** D-13-A `Box::leak`s `wgpu::AdapterInfo::name` once per program lifetime when constructing the panic-on-misconfig variant. If a user creates many `Functional::eval_vec` calls on a Wgpu-no-f64 adapter, this leaks once and then is cached because the variant is `Copy + 'static`. Worst case: handful of bytes per process lifetime.
**Mitigation:** Explicitly document; consider a `static OnceLock<&'static str>` cache so leak happens at most once per (process, adapter) tuple.

### R-06: Plan 06-01 git-mv test path drift

**Severity:** LOW
**Detail:** `crates/xcfun-eval/tests/*.rs` will continue to test `xcfun_eval::Functional` (which still lives in xcfun-eval), but ANY test importing kernel internals (`use xcfun_eval::functionals::lda::slaterx::slaterx_kernel`) breaks at the import line.
**Mitigation:** Plan 06-01 runs a comprehensive `git grep -n "use xcfun_eval::functionals\|xcfun_eval::dispatch\|xcfun_eval::density_vars"` and updates each. Keep a list of all hits in 06-01-SUMMARY.md for post-merge verification.

### Open Question: 1. ROCm runtime install precondition

**What we know:** D-05 makes ROCm-primary; D-02 says strict 1e-13 across all 78 functionals. Local env has no ROCm.
**What's unclear:** Should Phase 6 ship code-only (with ROCm tier-3 deferred to a "Quick Task") or block on local ROCm install? The user's PROFILE / instructions don't say.
**Recommendation:** Treat Phase 6 plans 06-00..06-06 as "ROCm-ready" code; Phase 6 sign-off depends on a separate `gsd:quick` task that installs ROCm and runs `cargo run -p validation --release -- --backend rocm --order 3 --filter '.*'` GREEN. The "Validation Architecture" section below assumes this two-step structure.

### Open Question: 2. mpmath fixture cardinality

**What we know:** D-04 says fixtures live in `validation/fixtures/mpmath/`; ~100 records/functional × 20 functionals × ~10s/record = ~6 hours one-time generation.
**What's unclear:** Should the fixture set be reproducible from a fixed seed (mirroring Phase 2 `xoshiro 0x1234abcd`) or be hand-curated to specifically cover boundary cases?
**Recommendation:** Both — start with a 50-record xoshiro-stratified subset for each ACC-04-amended functional + 50 hand-curated boundary records. Document in 06-N2 plan.

### Open Question: 3. Generation counter wrap-around

**What we know:** D-15 monotonic u64 counter on `Functional::settings`; bumped on every `set` call.
**What's unclear:** `wrapping_add(1)` per `set` call yields ~5e8-year duration at 1µs/`set` before wrap. Worth-mentioning?
**Recommendation:** Document as "non-issue at any reasonable usage". `wrapping_add` ensures the counter never traps, even in long-running services that re-set weights at high frequency.

## Pitfalls

(See "Common Pitfalls" section above for the full catalogue. Cross-cutting ones for the planner:)

- cubecl-runtime version drift across the four runtime crates → extend `xtask check-cubecl-pin`
- no `-Cfast-math`, no `-ffast-math`, no `#[cube(fast_math)]` — extend `check-no-fma` to `xcfun-kernels` and `xcfun-gpu` post Plan 06-01
- `mul_add` ban — extend `xtask check-no-mul-add` scope to `xcfun-kernels/src/functionals/**/*.rs`
- `anyhow` ban — extend `xtask check-no-anyhow` allowlist to include the two new library crates
- Wgpu silent-f32 → typed `XcError::WgpuNoF64` + compile-time `size_of::<Scalar>() == 8` assertion
- ROCm RDNA-2 needs `HSA_OVERRIDE_GFX_VERSION=10.3.0`
- Apple Silicon Metal lacks hardware f64; refuse-or-fallback at Batch::open

## Recommended Plan Tree

CONTEXT.md D-01 lists 9 plans (06-00, 06-01, 06-02, 06-03, 06-04, 06-05, 06-06, 06-N1, 06-N2, 06-N3). Confirmed structure with refinements:

| Plan | Title | Phase 6 D-decisions | Wave / sequencing | Deliverable axis |
|------|-------|---------------------|-------------------|------------------|
| 06-00 | Algebraic substrate | D-10 (tau guard), D-11 (erf taylor), D-04 (mpmath sidecar) + AD N≥4 specialisations | Wave 1 — substrate FIRST in CURRENT xcfun-eval tree | Axis 1 (substrate) |
| 06-01 | `xcfun-kernels` git-mv | D-08 (full split), D-09 (move-after-substrate) | Wave 2 — pure structural; no algebraic changes | Axis 2 (reorg) |
| 06-02 | `xcfun-gpu` unstub | D-08 (xcfun-gpu boundary), D-13 + D-13-A (typed `WgpuNoF64`), D-15 (buffer pool + gen counter), Backend enum + auto_backend skeleton | Wave 3 — depends on Plan 06-01 | Axis 2 (GPU runtimes) |
| 06-03 | cubecl-hip primary wiring | D-05 (Rocm primary), tier-3 ROCm parity harness, RDNA-2 doc note | Wave 4a — depends on Plan 06-02 | Axis 2 (GPU runtimes) |
| 06-04 | cubecl-cuda + cubecl-wgpu (Metal-via-Wgpu) opt-in | D-06 (CUDA + Metal opt-in), SHADER_F64 probe | Wave 4b — parallel with 06-03; depends on Plan 06-02 | Axis 2 (GPU runtimes) |
| 06-05 | RS-08 `eval_vec` GPU dispatch | D-14 (threshold + `XCFUN_MIN_BATCH_SIZE`), D-16 (signature), GPU-05 (ERF auto-fallback) | Wave 5 — depends on Plans 06-03 + 06-04 | Axis 3 (eval_vec + cleanup) |
| 06-06 | Strict zero-alloc + weights `Vec` refactor + LDA-vars=6 dispatch | D-12 (reusable handle, `UnsafeCell`), D-17 (weights → `Vec`), D-18 (DensVars-driven dispatch), promotion of `cpu_client()` to production | Wave 6 — depends on Plan 06-05 | Axis 3 (eval_vec + cleanup) |
| 06-N1 | D-19 cleanup — root-cause bisection inherited Phase-3 forwards | Path-B per Plan 04-10 finding; substrate-first hypothesis | Wave 7a — parallel with 06-N2 + 06-N3 | Axis 3 (D-19 cleanup) |
| 06-N2 | mpmath-only fixtures for 20 `excluded_by_upstream_spec` | D-04 mpmath sidecar reuse | Wave 7b — parallel | Axis 3 (D-19 cleanup) |
| 06-N3 | Post-libm-hybrid sweep — verify ~12 small-magnitude residuals tighten | Tier-2 re-run at order 3 after Plan 06-00 | Wave 7c — parallel | Axis 3 (D-19 cleanup) |

**Parallelisation notes:**
- Wave 4 (06-03 + 06-04) parallelisable — independent feature flags.
- Wave 7 (N1 + N2 + N3) parallelisable per D-01 discretion ("parallel is fine because they touch independent functional sets").
- Wave 1 → 2 → 3 is strictly sequential (Plan 06-00 in *current* tree must land before Plan 06-01 git-mv per D-09).

**Plan ordering rationale:**
- Substrate before reorg (D-09): land all algebraic changes in the current `xcfun-eval/src/functionals/` tree first; the git-mv in 06-01 then has zero algebraic deltas, making any post-mv tier-1 / tier-2 regression unambiguously a "move bug" not a "substrate bug".
- GPU runtimes before `eval_vec` wiring: Plan 06-05 needs `Backend` enum + `Batch<R>` from Plan 06-02 + cubecl-hip / cubecl-cuda / cubecl-wgpu from 06-03 / 06-04.
- Cleanup last: Plans 06-N1/N2/N3 verify the new substrate against tier-2 / tier-3.

## Validation Architecture

> Per `.planning/config.json` — `workflow.nyquist_validation: true` (enabled).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` + `cargo nextest` (workspace already configured); validation harness binary `cargo run -p validation` |
| Config file | Workspace `Cargo.toml` (members include `validation`); per-crate `Cargo.toml` for feature gating |
| Quick run command | `cargo nextest run --workspace --tests -j 4` |
| Full suite command | `cargo run -p validation --release -- --backend cpu --order 3 --jobs 18 --filter '.*' && cargo nextest run --workspace --tests` |
| Phase tier-3 gate | `cargo run -p validation --release -- --backend rocm --order 3 --jobs 18 --filter '.*'` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RS-08 | `eval_vec` dispatches to `Batch<R>` when nr_points ≥ 64 | unit | `cargo nextest run -p xcfun-rs --test eval_vec_threshold` | ❌ Wave 0 (Plan 06-05) |
| KER-01 | 78 `#[cube] fn` bodies generic over F | integration (already shipped) | `cargo nextest run -p xcfun-kernels --tests` | ❌ Wave 0 (Plan 06-01) |
| KER-02 | `DensVarsDev<F>` field-order parity with C++ | unit (existing) | `cargo nextest run -p xcfun-kernels --test densvars_invariant` | ❌ Wave 0 (post 06-01) |
| KER-03 | `CTaylor<F, N>` bit-flag indexing parity | unit (existing) | `cargo nextest run -p xcfun-ad --tests` | ✅ |
| KER-04 | Batch entry `eval_batch_kernel<F>` with `#[comptime]` (vars, mode, order) | unit | `cargo nextest run -p xcfun-gpu --test batch_kernel_smoke` | ❌ Wave 0 (Plan 06-02) |
| KER-05 | `#[cube]` purity rule (compile-time enforced) | compile gate | `cargo build -p xcfun-kernels --tests` | ✅ (auto by macro) |
| KER-06 | Tier-3 CPU 10k-grid 1e-13 vs scalar | integration | `cargo run -p validation --release -- --backend cpu --tier 3 --order 3 --filter '.*'` | ❌ Wave 0 (Plan 06-02 driver extension) |
| GPU-01 | `Batch<'fun, R>` API surface | unit | `cargo nextest run -p xcfun-gpu --test batch_api_shape` | ❌ Wave 0 (Plan 06-02) |
| GPU-02 | `Backend` enum + `auto_backend()` priority chain | unit | `cargo nextest run -p xcfun-gpu --test auto_backend_priority` | ❌ Wave 0 (Plan 06-02) |
| GPU-03 | Feature flags `hip` / `cuda` / `wgpu` compile + link | compile gate | `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu` | ❌ Wave 0 (Plan 06-04) |
| GPU-04 | Buffer pool powers-of-two + generation counter | unit | `cargo nextest run -p xcfun-gpu --test buffer_pool_growth` | ❌ Wave 0 (Plan 06-02) |
| GPU-05 | ERF auto-fallback to Cpu on Wgpu | integration | `cargo nextest run -p xcfun-gpu --features wgpu --test erf_fallback` | ❌ Wave 0 (Plan 06-05) |
| GPU-06 | `WgpuNoF64` typed error at Batch::open | unit | `cargo nextest run -p xcfun-gpu --features wgpu --test wgpu_no_f64` | ❌ Wave 0 (Plan 06-02) |
| GPU-07 | Tier-3 ROCm 10k-grid 1e-13 vs CPU | integration | `cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '.*'` | ❌ Wave 0 (Plan 06-03) |
| GPU-08 | Tier-3 Wgpu 10k-grid (excluding ERF) 1e-9 vs CPU | integration | `cargo run -p validation --release --features wgpu -- --backend wgpu --tier 3 --order 3 --filter '.*' --exclude-erf` | ❌ Wave 0 (Plan 06-04) |
| AD `N≥4` | `ctaylor_compose_n{4,5,6}` + `ctaylor_multo_n{4,5,6}` golden parity | unit | `cargo nextest run -p xcfun-ad --test golden_compose_n4 --test golden_multo_n4` | ❌ Wave 0 (Plan 06-00 Task 1) |
| D-10 tau_w guard | TPSSC/TPSSLOCC/REVTPSSC bounded output in unphysical regime | unit | `cargo nextest run -p xcfun-kernels --test tpss_tau_clamp` | ❌ Wave 0 (Plan 06-00 Task 3) |
| D-11 erf_precise_taylor | LDAERFX order 3 strict 1e-13 vs mpmath | unit | `cargo nextest run -p xcfun-ad --test erf_taylor_chain` | ❌ Wave 0 (Plan 06-00 Task 2) |
| D-12 strict zero-alloc | `eval` after `eval_setup` allocates exactly 0 bytes | unit | `cargo nextest run -p xcfun-rs --test zero_alloc_strict` | ❌ Wave 0 (Plan 06-06) |
| D-13/13-A WgpuNoF64 | XcError::WgpuNoF64 carries `&'static str`; `Copy` preserved | compile-time + unit | `cargo nextest run -p xcfun-core --test xcerror_copy_invariant` | ❌ Wave 0 (Plan 06-02) |
| D-17 weights Vec | `Functional::set` no longer Box::leaks | unit | `cargo nextest run -p xcfun-rs --test no_leak_on_set` | ❌ Wave 0 (Plan 06-06) |
| D-18 LDA-vars=6 dispatch | b3lyp / camb3lyp / bp86 in-process eval | integration | `cargo nextest run -p xcfun-rs --test lda_gga_alias_dispatch` | ❌ Wave 0 (Plan 06-06) |

### Sampling Rate

- **Per task commit:** `cargo nextest run -p <crate-touched> --tests` (the crate immediately touched + `xcfun-core` invariants if XcError changed)
- **Per wave merge:** `cargo nextest run --workspace --tests && cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda|gga'` (~10 min — order-2 LDA+GGA quick sweep)
- **Phase gate (sign-off):** Full tier-2 + tier-3 sweep across all 78 functionals at order 3:
  - `cargo run -p validation --release -- --backend cpu --order 3 --jobs 18 --filter '.*'` (CPU: 1e-12 strict + ACC-04 mpmath where amended)
  - `cargo run -p validation --release --features hip -- --backend rocm --order 3 --jobs 18 --filter '.*'` (ROCm tier-3 at 1e-13 — D-02 sign-off bar)
  - `cargo run -p validation --release --features wgpu -- --backend wgpu --order 3 --jobs 18 --filter '.*' --exclude-erf` (Wgpu tier-3 at 1e-9)
  - `cargo run -p validation --release --features cuda -- --backend cuda --order 3 --jobs 18 --filter '.*'` (CUDA best-effort if cloud-CI runner available; not blocking sign-off)

### 10 000-point Grid (Plan 06-02 driver extension)

Reuses Phase 2 `validation/src/fixtures.rs` stratified xoshiro grid (seed `0x1234abcd`):
- 7000 bulk points (`n ∈ [1e-5, 10.0]`, `|s/n| ∈ [0, 0.95]`)
- 1000 regularize stratum (`ρ ∈ [1e-14, 1e-5]`)
- 1000 polarised stratum (`|ζ| ∈ [0.95, 1.0]`)
- 1000 gradient stratum (`|∇ρ|² ∈ [1, 1e6]` — Phase 3+ usage)

Per Phase 6, tier-3 cross-backend uses the SAME grid; the comparison reference is `Backend::Cpu` (always-available CpuRuntime), with the C++ reference reserved for tier-2.

### mpmath Fixture Cadence

- **One-time generation:** Plan 06-00 Task 4 generates ~100 records each for the 13 ACC-04-amended functionals (LDAERFX/C/JT, TPSSC, TPSSLOCC, REVTPSSC, plus Plan 06-N3 small-magnitude residuals if libm-hybrid alone doesn't tighten them).
- **One-time generation:** Plan 06-N2 generates ~100 records each for the 20 `excluded_by_upstream_spec` functionals.
- **Total:** ~3300 mpmath records committed under `validation/fixtures/mpmath/<functional>.jsonl`.
- **Drift gate:** `cargo xtask regen-mpmath-fixtures --check` (mirrors Phase 2 D-21 `regen-registry --check`).

### Wave 0 Gaps

- [ ] `crates/xcfun-ad/tests/golden_compose_n4.rs`, `golden_compose_n5.rs`, `golden_compose_n6.rs` — covers AD N≥4 (Plan 06-00 Task 1)
- [ ] `crates/xcfun-ad/tests/golden_multo_n4.rs`, `golden_multo_n5.rs`, `golden_multo_n6.rs` — covers AD N≥4 (Plan 06-00 Task 1)
- [ ] `crates/xcfun-ad/tests/erf_taylor_chain.rs` — covers D-11 erf_precise_taylor (Plan 06-00 Task 2)
- [ ] `crates/xcfun-eval/tests/tpss_tau_clamp.rs` (or post-06-01: `crates/xcfun-kernels/tests/`) — covers D-10 (Plan 06-00 Task 3)
- [ ] `crates/xcfun-gpu/tests/batch_api_shape.rs`, `batch_kernel_smoke.rs`, `auto_backend_priority.rs`, `buffer_pool_growth.rs`, `erf_fallback.rs`, `wgpu_no_f64.rs` — covers GPU-01..06 (Plan 06-02 + 06-03 + 06-04 + 06-05)
- [ ] `crates/xcfun-rs/tests/eval_vec_threshold.rs`, `zero_alloc_strict.rs`, `no_leak_on_set.rs`, `lda_gga_alias_dispatch.rs` — covers RS-08 + D-12/D-17/D-18 (Plan 06-05 + 06-06)
- [ ] `xtask/src/bin/regen_mpmath_fixtures.rs` + `xtask/mpmath_eval/__main__.py` + JSONL fixtures — Plan 06-00 Task 4
- [ ] Validation driver `--reference {cpp, mpmath}` flag + `--backend rocm` flag + `--exclude-erf` flag — Plan 06-02 driver extension
- [ ] xtask gates: `check-cubecl-pin` extension to 5 crates; `check-no-mul-add` scope add `xcfun-kernels/src/functionals/**/*.rs`; `check-no-anyhow` allowlist add `xcfun-kernels` + `xcfun-gpu`
- [ ] `crates/xcfun-gpu/README.md` — RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0` note + Apple Silicon caveat + env vars (`XCFUN_FORCE_BACKEND`, `XCFUN_MIN_BATCH_SIZE`)

## Project Constraints (from CLAUDE.md)

- **Rust Edition 2024, MSRV 1.85** — required for const-generics + Edition-2024 module resolution
- **f64 only on numerical path** — f32 ban; cubecl `Float` trait monomorphises but caller never instantiates with f32
- **No `-Cfast-math`, no `-ffast-math`, no `#[cube(fast_math)]`** — ACC-05/06 / `xtask check-no-fma` / `xtask check-no-mul-add`
- **`anyhow` allowed only in `validation`/`xtask`/dev-deps** — `xcfun-kernels` and `xcfun-gpu` join the enforced library-graph set; extend `xtask check-no-anyhow` allowlist
- **`mul_add(...)` ban** — `xtask check-no-mul-add` extends to `xcfun-kernels/src/functionals/**/*.rs` post Plan 06-01
- **cubecl pin `=0.10.0-pre.3`** — extend `xtask check-cubecl-pin` to 5 crates lockstep
- **No nightly features** — verified by `rust-toolchain.toml`
- **MPL-2.0 license inheritance** — applies to new crates `xcfun-kernels` + `xcfun-gpu`

## Sources

### Primary (HIGH confidence)
- Context7 `/tracel-ai/cubecl` — runtime API surface (`R::client(&device)`, `client.create`, `client.empty`, `client.read_one`, `client.write`, `client.properties().feature_enabled(...)`, `client.features()`)
- Context7 `/tracel-ai/cubecl` — feature probing example with `Feature::Type(Elem::Float(FloatKind::F64))`
- Context7 `/tracel-ai/cubecl` — RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0` note from `crates/cubecl-hip/README.md`
- Context7 `/tracel-ai/cubecl` — `#[cube(fast_math = ...)]` attribute is OPT-IN per-function, NOT default
- Context7 `/tracel-ai/cubecl` — `cubecl-wgpu` covers Vulkan/Metal/DX12/WebGPU/OpenGL platforms
- Crates.io REST API — verified `cubecl-{cpu,hip,cuda,wgpu}` exist at `=0.10.0-pre.3`; `cubecl-metal` does NOT exist (returns 404)
- Crates.io REST API — head pre-release is `0.10.0-pre.4` (project pin one minor pre-release behind)
- Vendored `docs/manual/Cubecl/cubecl_3d_dft.md` — full lifecycle: `device → ComputeClient::load(&device) → client.create / empty / launch_unchecked / read_one / from_bytes`
- Vendored `docs/manual/Cubecl/cubecl_macro_fanout_manual.md` — minimise launch surface; prefer free `#[cube]` helpers over trait expansion; comptime for constants
- Project source: `crates/xcfun-ad/src/ctaylor_rec/{multo.rs, compose.rs}` — existing N=0..3 specialisation pattern (template for N=4..6)
- Project source: `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818` — existing `ctaylor_max` (D-10 reuses)
- Project source: `crates/xcfun-ad/src/expand/erf.rs:174` — existing `erf_precise` libm-port (D-11 extends)
- Project source: `crates/xcfun-eval/src/for_tests.rs` — `OnceLock<CpuClient>` pattern (Plan 06-06 promotes to production)
- Project source: `crates/xcfun-rs/src/functional.rs:184-216` — Phase 5 `sync_weights_from_settings` `Box::leak` (D-17 retires)

### Secondary (MEDIUM confidence)
- `docs/design/06-cubecl-strategy.md §2 / §5 / §7 / §10` — buffer lifecycle, transfer budget, kernel-resident buffers, launch config
- `docs/design/05-module-responsibilities.md §3 / §4` — `xcfun-kernels` and `xcfun-gpu` responsibilities (D-08 source)
- `docs/design/07-accuracy-strategy.md §1 / §6` — 1e-12 invariant + tolerance budget breakdown
- `docs/design/08-error-model.md §2.3` — XcError `Copy + Send + Sync + 'static` constraint (D-13-A preserves)
- `docs/design/09-testing-strategy.md §4` — Tier-3 cross-backend parity 1e-13 envelope; nightly schedule
- `docs/design/10-build-and-dependencies.md §3.4 / §6` — `xcfun-gpu` Cargo.toml and feature flags template
- Phase 5 `05-CONTEXT.md` D-13 — zero-alloc fall-back form (b) → strict here
- Phase 5 `05-CONTEXT.md` D-14 — LDA-vars=6 dispatch table constraint (D-18 resolves)
- Phase 5 `05-CONTEXT.md` D-17 — `Functional` Send + Sync (preserved by D-12)
- Phase 4 `04-CONTEXT.md` D-19 — 30+ inherited forwards consolidated (Plans 06-N1/N2/N3)
- Plan 04-10 Path-B side-by-side bisection finding — TPSS algorithmic-identity confirmed; root cause f64-rounding cancellation in unphysical regime

### Tertiary (LOW confidence — flagged for validation)
- mpmath at prec=200 reproducibility — known stable across decades but project hasn't specifically pinned a CI version
- Apple Silicon Metal f64 absence — extrapolated from cubecl-book features.md "f64 not supported for all operations" but not specifically confirmed for M1/M2/M3 hardware locally (no Apple hardware available)
- Local ROCm install state — `which hipconfig` returns `/usr/bin/hipconfig` but `/opt/rocm` is missing; the actual ROCm runtime install state is uncertain (NEEDS USER VERIFICATION before Plan 06-03 sign-off)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `cubecl-wgpu` Metal backend is sufficient for Plan 06-04 D-06 (no separate `cubecl-metal` crate exists) | "Standard Stack" + "Pitfall 9" | LOW — verified via crates.io 404; only impact is CONTEXT.md D-06 wording amendment at sign-off |
| A2 | Local ROCm runtime is NOT installed (only `hipconfig` stub at `/usr/bin/hipconfig`; no `/opt/rocm`) | "R-01" + "Open Question 1" | MEDIUM — if ROCm IS actually installed, no plan change needed; if NOT installed, Phase 6 sign-off depends on a separate Quick Task that installs ROCm or uses cloud-CI |
| A3 | mpmath JSONL fixtures at ~100 records/functional is the right cost-coverage trade-off | "mpmath Sidecar Architecture" + "Validation Architecture" | LOW — adjustable per CONTEXT.md Discretion ("layout … pick whatever the validation harness reads cleanest") |
| A4 | Plan 06-00 N≥4 specialisations may be macro-generated to reduce ~1000-LOC hand-port for N=6 | "Pattern 1" + "R-03" | LOW — explicit Discretion in CONTEXT.md; impacts implementer choice only |
| A5 | Generation counter on `Functional::settings_gen` is bumped only on `set` (not `eval_setup`) | "Pattern 3" | LOW — `eval_setup` does not modify settings; if it did, the counter would need bumping there too |
| A6 | `xtask check-cubecl-pin` extends to 5 crates (cubecl + cpu + hip + cuda + wgpu) | "Pitfall 1" + "Validation Architecture" | LOW — straightforward Plan 02-02 pattern extension |
| A7 | Phase 5 `Box::leak` per `set` call is the ONLY remaining heap leak in `Functional::eval` | "Pitfall 8" | MEDIUM — there may be other per-eval allocations in cubecl-cpu launch path; Plan 06-06 needs full counting-allocator trace, not just delta-check |
| A8 | f64 device probing via `feature_enabled(Feature::Type(Elem::Float(FloatKind::F64)))` is the canonical cubecl 0.10-pre.3 API | "Backend dispatch shape" + Code Examples | LOW — verified via Context7 fetch; confirmed in cubecl-book features.md |
| A9 | `Backend` enum includes a `Metal` variant even though Metal is reached via cubecl-wgpu | "Pattern 4" | LOW — discretion; could collapse `Metal | Wgpu → WgpuRuntime`, but having a separate `Metal` variant gives the user diagnostic clarity (`auto_backend()` returning `Metal` vs `Wgpu` is helpful debug info) |
| A10 | mpmath sidecar's Python module path layout (`xtask/mpmath_eval/__main__.py`) follows Python conventions and `xtask` already runs Python via `subprocess` | "mpmath Sidecar Architecture" | LOW — established pattern, no project-specific risk |

**If any of these assumptions is wrong, the planner should escalate via PLANNING INCONCLUSIVE rather than guess.**

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All Phase 6 plans | ✓ (assumed) | 1.85+ per `rust-toolchain.toml` | — |
| `cargo` + `cargo nextest` | All testing | ✓ (assumed in CI / dev env) | latest stable | `cargo test` (slower) |
| `cubecl-cpu` runtime | Plans 06-00..06-06 (always-on) | ✓ (already in workspace) | `=0.10.0-pre.3` | — |
| `cubecl-hip` (ROCm) | Plan 06-03 (D-05 primary) | **PARTIAL** — `hipconfig` present but `/opt/rocm` directory missing | unknown | Cloud CI runner; defer Plan 06-03 sign-off to a separate Quick Task |
| `cubecl-cuda` | Plan 06-04 (opt-in) | ✗ (no NVIDIA hardware in dev env) | — | Best-effort cloud CI; non-blocking sign-off |
| `cubecl-wgpu` (Vulkan/Metal/DX12) | Plan 06-04 (portable fallback) | ✓ (assumed — wgpu has broad Linux Vulkan support) | `=0.10.0-pre.3` after install | CPU fallback automatic via `auto_backend()` |
| Python 3 | mpmath sidecar (Plan 06-00 Task 4 + Plan 06-N2) | ✓ | 3.14.4 | — |
| `mpmath` Python lib | mpmath sidecar | ✓ | 1.4.1 | `pip install 'mpmath>=1.4,<2.0'` if missing |
| `xcfun-master/` C++ source | `validation/` tier-2 harness | ✓ (vendored under `xcfun-master/`) | content-hash pinned | — |
| `cc` C++ toolchain | `validation/build.rs` (cc-compiles xcfun-master) | ✓ (assumed in CI / dev env per Phase 2 + 5 successes) | gcc/clang ≥ C++17 | — |

**Missing dependencies with no fallback:**
- ROCm runtime install — blocks Plan 06-03 final sign-off (tier-3 ROCm GREEN). Mitigation: separate Quick Task or cloud-CI runner.

**Missing dependencies with fallback:**
- CUDA hardware — Plan 06-04 ships code-only; cloud-CI best-effort tier-3.
- Apple hardware — Plan 06-04 ships code-only; cloud-CI / community-maintained tier-3 (Wgpu route on Linux Vulkan covers most of the code path).

## Security Domain

> Phase 6 has minimal security surface. The library does not expose any user-input parsing, network endpoints, or authentication paths. Per CLAUDE.md, `security_enforcement` is not explicitly disabled, so this section is included for completeness.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — (numerical library, no auth surface) |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Existing pattern: validate `(vars, mode, order)` in `eval_setup`; validate `density.len()` and `out.len()` in `eval`/`eval_vec`. RS-08 `eval_vec` adds pitched-buffer validation: `density.len() >= density_pitch * nr_points`, `out.len() >= out_pitch * nr_points`, `density_pitch >= input_length()`, `out_pitch >= output_length()`. Returns `XcError::InputLengthMismatch` / `XcError::OutputLengthMismatch` on violation. |
| V6 Cryptography | no | — |
| V7 Error Handling | yes | Existing `XcError` thiserror-derived enum; D-13/D-13-A adds `WgpuNoF64` variant preserving `Copy + non_exhaustive` |
| V12 File Handling | yes (mpmath sidecar) | `xtask` invokes Python via `Command::new("python3")`; argument list is hardcoded in the Rust driver, no user input pass-through. JSONL files written under `validation/fixtures/mpmath/` are content-hash-stamped per Phase 2 D-21 pattern (drift gate via `--check`). |
| V14 Configuration | yes | Env vars `XCFUN_FORCE_BACKEND` and `XCFUN_MIN_BATCH_SIZE` are read at runtime; both are validated (parse-or-panic for FORCE_BACKEND; integer parse with default for MIN_BATCH_SIZE). |

### Known Threat Patterns for Numerical Library + GPU Runtime

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Caller passes mismatched `density_pitch` vs actual buffer size → out-of-bounds read | Tampering | Validate `density.len() >= density_pitch * nr_points` before constructing slice; return `XcError::InputLengthMismatch` |
| GPU device returns to userspace without f64 → silent precision degradation | Repudiation (silent failure) | D-13/D-13-A: typed `XcError::WgpuNoF64` at `Batch::open`; compile-time `size_of::<Scalar>() == 8` assertion |
| Concurrent `eval()` on shared `Functional` → torn writes to reusable handle | Tampering | D-12: `unsafe impl Sync` carries a doc-comment "racy if shared concurrently"; users must clone or wrap in Mutex |
| Python sidecar shell injection via fixture regen | Tampering | All sidecar args are project-internal (functional name from FunctionalId enum, etc.); no user-controlled strings cross the `Command::new` boundary |
| ROCm RDNA-2 silent kernel-load failure | DoS (silent abort) | RUST-04 / RUST-05 wave gate: doc the `HSA_OVERRIDE_GFX_VERSION` requirement; `Batch::open` verifies kernel compiles before any user `launch` call |
| C ABI panic propagation | DoS (unwinding into C code is UB) | Phase 5 already established `c_entry!` macro with `catch_unwind` + `abort()`; Plan 06-05 inherits this for `xcfun_eval_vec` |

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every cubecl-* version verified via crates.io REST API; `cubecl-metal` non-existence is a fact, not a guess
- Architecture (xcfun-kernels + xcfun-gpu split): HIGH — design-doc-05 is explicit; existing tests already exercise the Plan 06-01 git-mv shape via the `xcfun-eval` substrate
- AD `N≥4` recursion: MEDIUM — the algorithm is verbatim port of C++; the f64-rounding cumulative-error budget is the open question
- ERF `erf_precise_taylor` AD-chain wrapper: MEDIUM — the libm-hybrid baseline already shipped at 1e-14; the AD-chain wrapper is novel work but follows the `expm1` LDAERFX pattern (Phase 2 Plan 02-06 Fix 1)
- TPSS `tau ≥ tau_w` guard: HIGH — `ctaylor_max` already exists; the guard is a 3-line insert at the top of three kernel bodies
- mpmath sidecar: HIGH — pattern is standard subprocess interface; mpmath is decades-stable
- D-19 cleanup methodology: MEDIUM — Path-B bisection has Phase 4 precedent (Plan 04-10) but each persistent forward may have its own root cause
- Pitfalls: HIGH — every pitfall is either documented in CLAUDE.md or surfaced by Phase 2/3/4 hindsight

**Research date:** 2026-04-30
**Valid until:** 2026-05-30 (30 days — cubecl 0.10 stable release would invalidate Standard Stack pins; AD/AC/CRATE-architecture sections stable for ~6 months)

## RESEARCH COMPLETE
