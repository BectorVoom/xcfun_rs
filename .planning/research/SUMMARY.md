# Project Research Summary

**Project:** xcfun_rs
**Domain:** Rust reimplementation of a C++ scientific library (DFT exchange–correlation functionals) with arbitrary-order automatic differentiation, unified CPU/GPU kernels via `cubecl`, C ABI drop-in replacement, and Python bindings. Primary contract: output parity with C++ xcfun at ≤ 1e-12 relative error.
**Researched:** 2026-04-19
**Confidence:** HIGH overall (HIGH for stack, features, architecture; MEDIUM for `cubecl` — the one load-bearing pre-release dependency)

---

## Executive Summary

xcfun_rs is a numerical-library port with a non-negotiable parity contract and a narrow, well-scoped deliverable shape. The research confirms that every decision already in the 14-document design brief (`docs/design/`) is consistent with 2026 Rust ecosystem practice: a 7-crate workspace layered AD → core → kernels → runtime → façade → C ABI / Python matches the shape of `cubecl`, `burn`, `candle`, `polars`, and `rustls-ffi`. The project's unique value (order 5–6 AD via algorithmic identity with `xcfun-master/src/taylor/ctaylor.hpp`, plus a single `#[cube]` source dispatching to CPU / CUDA / Wgpu) cannot be obtained from any existing Rust crate; an in-house `xcfun-ad` of ~800 LOC is both cheaper and provably identical to the C++ reference compared with bending `autodiff` / `hyperdual` / `ad` / `num-dual` into shape.

The recommended approach is to **execute the existing 8-phase DAG (M0–M8 from `docs/design/11-process-and-milestones.md`, plus an M9 Python/release phase) in strict order**, because the dependency graph forbids reordering: nothing compiles without `CTaylor`, no functional can be validated without `xcfun-core` registry + `DensVars`, the GPU path cannot exist before the kernel crate, and the C ABI and Python bindings must follow the façade. Along the way, lock the stack with six concrete version bumps (bitflags 2.11.1, cc 1.2.60, proptest 1.11.0, rstest 0.26.1, rand_xoshiro 0.8.0, tracing-subscriber 0.3.23), and hold `cubecl = "=0.10.0-pre.3"` as the single explicit pre-release risk — isolated to exactly two crates (`xcfun-kernels`, `xcfun-gpu`) and permanently gated by CI.

The dominant risk class is numerical, not architectural: floating-point reassociation by LLVM (Pitfall 1), transcendental drift across libm / CUDA libdevice / WGSL (Pitfall 2), CTaylor coefficient-layout and `*_expand` rounding divergence from C++ (Pitfalls 3, 9), `regularize` and densvars-fallthrough semantic errors (Pitfalls 4, 5), and Wgpu silently instantiating in f32 (Pitfall 7). All are preventable with discipline: empty `RUSTFLAGS`, textual AD port, `f64::to_bits`-level golden tests against C++ intermediates on the AD engine, helper-function-chain rewrites of fall-through switches, and runtime probing of `SHADER_F64`. Combined with the four-tier validation harness (self-tests / CPU-reference / cross-backend / regression grid), the 1e-12 contract is achievable and the project is ready for roadmap creation.

---

## Key Findings

### Recommended Stack

Confirmed from `.planning/research/STACK.md`: the stack pinned in CLAUDE.md is close to optimal. Six deps are stale and should bump atomically; `cubecl` is the only load-bearing pre-release.

**Core technologies (with version pins):**
- **Rust Edition 2024 / MSRV 1.85** — const generics for `CTaylor<T, N>`; mandated by `thiserror 2.0`.
- **`thiserror =2.0.18`** — library error derive.
- **`anyhow ^1.0.102`** — app-boundary only; CI-enforced by `cargo xtask check-no-anyhow`.
- **`bitflags =2.11.1`** (bump from 2.10.0) — improved `Flags` derive.
- **`tracing =0.1.44`** + **`tracing-subscriber =0.3.23`** (bump from unbounded 0.3) — 0.3.21 yanked.
- **`cubecl =0.10.0-pre.3` (hard pin)** + `cubecl-cpu`, `cubecl-cuda`, `cubecl-wgpu` at the same exact version — unified kernel DSL.
- **In-house `xcfun-ad` crate** — no existing Rust AD crate replicates xcfun's bit-flag multilinear polynomial layout.
- **`cbindgen =0.29.2`** + **`cc ^1.2.60`** (bump from 1.1).
- **`pyo3 =0.28.3`** + **`numpy =0.28.0`** — zero-copy f64 NumPy interop; 0.28.0/0.28.1 yanked.
- **`maturin >=1.12, <2.0`** (bump floor).
- **Testing:** `criterion =0.8.2`, `approx =0.5.1` (do NOT bump to 0.6.0-rc), `proptest =1.11.0` (bump), `rstest =0.26.1` (bump), `rand_xoshiro =0.8.0` (bump), `serde_json ^1.0.149`.

**Agreed version bumps (six):** `bitflags 2.10.0 → 2.11.1`; `cc 1.1 → 1.2.60`; `proptest 1.6 → 1.11.0`; `rstest 0.23 → 0.26.1`; `rand_xoshiro 0.7 → 0.8.0`; `tracing-subscriber 0.3 → 0.3.23` (plus floor bumps on `anyhow → ^1.0.102` and `maturin >=1.12`).

**What NOT to use (explicit rejections):** `-Cfast-math` / `-Ctarget-cpu=native` / any reassociation flag; `f32` on the numerical path; any third-party AD crate; `rayon` inside per-point evaluation; `num-traits::Float` in `CTaylor`; stable `cubecl 0.9.0`; `approx 0.6.0-rcN`; `cpython`; `serde` in library crates.

### Expected Features

Confirmed from `.planning/research/FEATURES.md`: xcfun_rs occupies the same market niche as C++ xcfun — the **arbitrary-order-AD** XC library, sibling to (not competitor of) Libxc. **Correction to PROJECT.md:** alias count is **46**, not "50+" — counted directly in `xcfun-master/src/functionals/aliases.cpp` under `aliases_array`.

**Must have (table stakes):**
- 78 functionals (full `list_of_functionals.hpp` coverage).
- 46 named aliases (`lda`, `pbe`, `b3lyp`, `pbe0`, `camb3lyp`, `scan`, `r2scan`, `m06*`, `b97*`, …).
- 4 tunable parameters (`EXX`, `RANGESEP_MU`, `CAM_ALPHA`, `CAM_BETA`).
- 31 `Vars` combinations.
- 3 evaluation modes: `PartialDerivatives` (≤ 4), `Potential`, `Contracted` (≤ 6).
- Numerical parity ≤ 1e-12 relative error — CI-gated.
- C ABI drop-in replacement for `xcfun-master/api/xcfun.h`, cbindgen-generated, byte-diffed.
- Python bindings via PyO3 0.28 + rust-numpy 0.28 (zero-copy).
- Per-point `eval` + batch `eval_vec` with caller-owned pitched slices.
- Thread-safe `Functional` handle (`Send + Sync`).
- Stable error codes (`EORDER=1, EVARS=2, EMODE=4`).
- Zero heap allocation on per-point hot path.
- MPL-2.0 license.

**Should have (differentiators):**
- Arbitrary-order AD (0..=6) — Libxc caps at 4; ExchCXX at 1–2.
- Unified CPU/GPU via single `#[cube]` source per functional.
- Rust memory safety for long-running HPC jobs.
- Zero-copy NumPy (vs. pybind11 copies upstream).
- Typed Rust API (`Vars`, `Mode`, `XcError`, `Dependency`).
- Hermetic `cargo build` (no C++ toolchain required).
- CI-enforced parity gate.

**Anti-features (explicit rejections):**
- Libxc-scale functional coverage (~400).
- Bit-identical output with C++.
- User-supplied Rust functionals / plugin API.
- Third-party AD frameworks.
- `f32` numerical path.
- Order > 6 derivatives.
- `f128` extended precision.
- Distributed multi-node.
- VV10, D3/D4, ML functionals.
- Libxc-compatibility shim.
- Pre-API-v2 C headers.

### Architecture Approach

Confirmed from `.planning/research/ARCHITECTURE.md`: the proposed **7-library-crate workspace + `validation/` + `xtask/`** is idiomatic for 2026 Rust numerical/GPU libraries. One seam (shared algebraic spec between host helpers and `#[cube]` helpers) should be reified by **making the `#[cube]` form the single source and routing CPU through `CpuRuntime`** — eliminating host/device drift by construction (recommendation: adopt option (a)).

**Major components (7 library crates + 2 leaf binaries):**

1. **`xcfun-ad`** — `CTaylor<T, N>`, `Num` trait, scalar series expansions (`reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`, `gauss_expand`, `cbrt_expand`). `no_std`-capable. Stack-only `[T; 1 << N]`.
2. **`xcfun-core`** — `Vars`, `Mode`, `Dependency`, `XcError`, `DensVars<T>`, static `FUNCTIONAL_DESCRIPTORS`, `ALIASES`, `VARS_TABLE`, scalar CPU dispatcher, `Functional::{...}`.
3. **`xcfun-kernels`** — runtime-agnostic `#[cube]` per-point evaluators, `DensVarsDev`, `eval_batch_kernel<F: Float>`. Never instantiates a runtime.
4. **`xcfun-gpu`** — `Batch<'fun, R: Runtime>`, `Backend` enum, device buffer pool, `auto_backend()`, `erf`-aware CPU fallback on Wgpu, tracing.
5. **`xcfun-rs`** — thin façade; re-exports only.
6. **`xcfun-capi`** — C ABI leaf; `crate-type = ["cdylib", "staticlib"]`; every `extern "C"` wrapped in `c_entry!` macro; cbindgen generates `xcfun.h`; `headers-match` CI job.
7. **`xcfun-py`** — Python binding leaf; `crate-type = ["cdylib"]`; PyO3 + rust-numpy; maturin.

Plus **`validation/`** (links `xcfun-master/**/*.cpp` via `cc`; parity harness) and **`xtask/`** (codegen, boundary enforcement, release).

**Strict 8-phase build DAG** (forced by dependency graph):

```
M0  xcfun-ad         ← (nothing)
M1  xcfun-core scaffold ← xcfun-ad
M2  LDA tier         ← M1
M3  GGA tier         ← M2
M4  metaGGA + modes + aliases ← M3
M5  xcfun-rs + capi  ← M4
M6  xcfun-kernels + CPU batch ← M5
M7  CUDA + Wgpu      ← M6
M8  xcfun-py + release ← M7
```

### Critical Pitfalls

Confirmed from `.planning/research/PITFALLS.md` (14 ranked pitfalls). Dominant class: **numerical divergence from C++ reference**.

**Top pitfalls (prioritized):**

1. **P3: CTaylor coefficient-layout bug on port** (Phase M0) — flat triple-loop rewrite breaks parity at order ≥ 2. **Prevent:** verbatim recursive port; `f64::to_bits` golden test vs. C++ at M0 exit.
2. **P1: LLVM floating-point reassociation** (Phase 0 + M2) — may emit FMA/contract without `-Cfast-math`. **Prevent:** empty `RUSTFLAGS` + `-Cllvm-args=-fp-contract=off`; lint-ban `mul_add` in functional bodies; `cargo asm` spot-check.
3. **P9: `*_expand` coefficient layout miscopied from `tmath.hpp`** (Phase M0) — stability-optimized recursions differ per-function. **Prevent:** body-for-body port; 12×3×7 golden-coefficient tests against C++.
4. **P5: `densvars` switch fallthrough lost in Rust `match`** (Phase M1) — every meta-GGA with τ returns garbage. **Prevent:** flatten into helper-function chain; per-variant field-by-field parity test.
5. **P4: `regularize()` applied to wrong CTaylor coefficient** (Phase M1) — a naive rewrite zeros derivatives. **Prevent:** in-place on `c[CNST]` only; unit test asserts higher-order coefficients preserved.
6. **P2: libm vs CUDA libdevice vs WGSL transcendental drift** (Phase M3 + M7) — `exp`/`log`/`pow` differ 1–4 ULP; WGSL `erf` differs ≈ 1.5e-7. **Prevent:** backend-qualified tolerance table (CPU 1e-12, CUDA-vs-CPU 1e-13, Wgpu-vs-CPU 1e-9); force CPU for `erf`-using functionals on Wgpu.
7. **P7: Wgpu silently running in f32** (Phase M7) — missing `SHADER_F64` → silent f32 at ~1e-7 rel-err. **Prevent:** top-level launcher takes concrete `f64`; runtime `Features::SHADER_F64` gate; compile-time `size_of::<Scalar>() == 8`.
8. **P6: Alias weight propagation — multiplicative, not additive** (Phase M4) — map-overwrite port breaks every hybrid. **Prevent:** port byte-for-byte; per-alias test at `value ∈ {1.0, 0.37}`; `camcompx` negative-weight canary.
9. **P8: `cubecl =0.10.0-pre.3` API drift between pre-releases** (Phase 0) — **Prevent:** commit `Cargo.lock`; CI `cargo metadata` asserts the exact line; cubecl isolated to 2 crates.
10. **P14: Panic-across-FFI from `xcfun-capi`** (Phase M5) — unwinding across `extern "C"` is UB. **Prevent:** `c_entry!` macro (catch_unwind + null-check + XcError-to-int); `[profile.release] panic = "abort"`; clippy lint rejects raw `#[no_mangle] extern "C"`.
11. **P10: `*_expand` asserts compiled out in release** (Phase M0 + M1) — silent NaN. **Prevent:** use `assert!` not `debug_assert!` on preconditions; `DensVars::build` returns `Result`.
12. **P11: `poly` Horner evaluation direction flipped** (Phase M2) — C++ uses descending Horner; Rust idiom defaults to ascending. **Prevent:** port verbatim `res = coeffs[N-1]; for i in (0..N-1).rev() { res = res * x + coeffs[i]; }`.
13. **P13: Functional registry codegen drift** (Phase 0) — **Prevent:** content-hash on `registry.rs`; `cargo xtask regen-registry --check` in CI; fixtures record `xcfun_version`.
14. **P12: PW92C / PBEX_MU legacy-constants toggle** (Phase 0 + M2) — **Prevent:** read `config.hpp` in Phase 0; port both tables behind `pw92c-legacy-constants` Cargo feature.

---

## Integrated Risk Assessment

| Risk | Probability | Severity | Primary mitigation | Owner phase |
|------|-------------|----------|---------------------|-------------|
| `cubecl 0.10` never stabilises; bumps break kernels | MEDIUM | HIGH | `=0.10.0-pre.3` pin; isolated to 2 crates (~1 kLoC); every bump triggers full tier-2+3 | Phase 0; M6/M7 re-validation |
| `cubecl f64` degrades on future CUDA driver | LOW | HIGH | Runtime probe `Feature::Type(f64)`; refuse launch + `tracing::error!` if absent | M7 |
| `cubecl-wgpu` WGSL lacks f64 entirely | CERTAINTY | HIGH (if misused) | Wgpu runs at 1e-9 tolerance; `erf` functionals forced to CPU; `SHADER_F64` gate | M7 |
| In-house `xcfun-ad` algorithmic bug | MEDIUM (until tier-2) | HIGH | Four-tier tests; `f64::to_bits` golden tests on `*_expand` and `CTaylor` multiply at M0 exit | M0 |
| LLVM reassociation drift | MEDIUM | MEDIUM | Empty `RUSTFLAGS`; `-fp-contract=off`; `mul_add` lint | Phase 0, M2 |
| `pyo3 0.28.4+` yanked | LOW | MEDIUM | Pin `=0.28.3`; `cargo audit` | Phase 0 |
| `cbindgen` changes header format | LOW | MEDIUM | `headers-match` CI diff blocks merge | M5 |
| Registry/fixture drift vs. vendored `xcfun-master/` | MEDIUM | MEDIUM | Content-hash; `xtask regen-registry --check`; fixtures record version | Phase 0 |
| Panic-across-FFI from missed `c_entry!` | LOW | HIGH | Clippy lint; `panic = "abort"`; `cargo fuzz` driver | M5 |
| Compile-time blowup at 78 functionals × 7 orders | MEDIUM | LOW | Per-family `pub mod`; feature-gate `cuda`/`wgpu`; default CPU-only | M4 onward |
| Register pressure at order 6 on consumer GPUs | MEDIUM | LOW | A100 minimum for order 6; smaller GPUs fall back | M7 |

---

## Key Decisions Already Made in the Design (Inputs, Not Open Questions)

- **D1:** Algorithmic-identity port of `CTaylor` — verbatim, not Rust-idiomatic rewrite.
- **D2:** Custom `Num` trait, **not** `num-traits::Float`.
- **D3:** Single-source `#[cube]` kernel for CPU + CUDA + Wgpu.
- **D4:** `f64` everywhere; `Num` intentionally not implemented for `f32`.
- **D5:** Wgpu is best-effort — 1e-9 tolerance; `erf` functionals force `Backend::Cpu`.
- **D6:** `#[comptime]` on `(vars, mode, order)` only; functional id is runtime dispatch (78-arm match).
- **D9:** Functional registry generated by `xtask codegen`, checked into git (hermetic `cargo build`).
- **D13:** `catch_unwind` on every `xcfun-capi` FFI entry; `panic = "abort"` in cdylib release.
- **D17:** No library-internal threading; parallelism via `cubecl::CpuRuntime`.
- **D18:** `xcfun-master/` vendored (not a submodule); content-hash pinned.
- **Order cap:** `PartialDerivatives` ≤ 4, `Contracted` ≤ 6.
- **Workspace:** 7 library crates + `validation/` + `xtask/`.
- **Error model:** `thiserror` in libs; `anyhow` app-boundary only, CI-enforced.
- **License:** MPL-2.0 inherited; cargo-deny allowlist `{MPL-2.0, MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode-DFS-2016}`.
- **Shared-spec clarification (new):** adopt option (a) — `#[cube]` is the single source; CPU runs it through `CpuRuntime`.

---

## Implications for Roadmap

The 8-phase DAG from the design spec maps directly to a standard-granularity GSD roadmap. Recommended ordering:

### Phase 0: Workspace Scaffolding & CI Foundations
**Rationale:** unblocks Phase 1; CI gates catch P1/P8/P12/P13 before code exists.
**Delivers:** workspace + per-crate skeletons; `rust-toolchain.toml (1.85)`; `deny.toml`; `cbindgen.toml`; `.cargo/config.toml` (empty `RUSTFLAGS`, `-fp-contract=off`); CI jobs (fmt, clippy, test, deny, no-anyhow, headers-match stub, cubecl-lockfile-guard, registry-hash-check, cargo audit); `xtask` and `validation/` skeletons; vendored `xcfun-master/` content-hash committed.

### Phase 1: `xcfun-ad` — Taylor Algebra & AD Primitives
**Rationale:** nothing else compiles without `CTaylor` and `Num`; smallest surface, cleanest feedback.
**Delivers:** `CTaylor<T, N>` + ops; `Num` trait; all `*_expand` functions; composed `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`; property tests; micro-benchmarks.
**Avoids:** P3, P9, P10.
**Exit:** property tests green; bit-equivalence vs. C++ at orders 0..=3; baseline published.

### Phase 2: `xcfun-core` Foundations — Registry, Vars, Setup
**Rationale:** `DensVars` + registry prerequisite to every functional body; isolates fallthrough/regularize hazards before functional math.
**Delivers:** `Vars`, `Mode`, `Dependency`, `XcError`; `VARS_TABLE` codegen; `DensVars<T>` builders for every variant; `Functional::{new, set, get, is_gga, is_metagga, eval_setup, user_eval_setup, input_length, output_length}`; text APIs; 78 functional stubs; 46 alias stubs; `poly` descending-Horner helper.
**Avoids:** P4, P5, P11, P12.
**Exit:** `eval_setup` matches reference error branches; `input_length`/`output_length` match reference on 31 × 7 × 3; per-variant DensVars parity test passes.

### Phase 3: Functional Ports — LDA Tier
**Rationale:** shortest bodies; first 1e-12 greenlight; if LDA drifts, structural not math bugs.
**Delivers:** 11 LDA functionals (`slaterx`, `vwn3c`, `vwn5c`, `pw92c`, `pz81c`, `ldaerfx`, `ldaerfc`, `ldaerfc_jt`, `tfk`, `tw`, `vonw`) + helpers; `Mode::PartialDerivatives` order 0..=4 dispatcher.
**Avoids:** P1, P6 (LDA-level).
**Exit:** tier-1 self-tests pass; tier-2 at order 0..=2 passes at 1e-12 on CPU.

### Phase 4: Functional Ports — GGA Tier + `Mode::Potential`
**Rationale:** 45 functionals — bulk of the port; unlocks `Mode::Potential` via `CTaylor<f64, 2>` divergence trick.
**Delivers:** 45 GGA functionals + helpers; `Mode::Potential` with `_2ND_TAYLOR` vars enforcement.
**Avoids:** P2 (range-separated functionals surface here), P11.
**Exit:** tier-1 + tier-2 at order 0..=2 pass for every GGA functional.

### Phase 5: Functional Ports — metaGGA + `Mode::Contracted` + Aliases
**Rationale:** completes the 78-functional set; `Contracted` 0..=6 is the differentiator mode; aliases are the user-facing surface.
**Delivers:** 15 metaGGA functionals + helpers; `Mode::Contracted` for orders 0..=6; 46 aliases with composition weights.
**Avoids:** P6 (alias weight multiplicative propagation; `camcompx` negative-weight canary).
**Exit:** tier-1 + tier-2 at orders 0..=3 pass for every functional; aliases match manual composition; `cargo xtask validate --backend cpu --order 3` reports zero failures across 78 functionals.

### Phase 6: `xcfun-rs` Façade + `xcfun-capi` C ABI
**Rationale:** façade + C ABI land together — one API decision.
**Delivers:** `xcfun-rs` re-exports; `xcfun-capi` with every `extern "C"` symbol, `c_entry!` macro, cbindgen-generated `xcfun.h`, `tests/c_abi.rs` golden test, `headers-match` CI.
**Avoids:** P14.
**Exit:** `cargo test -p xcfun-capi --test c_abi` passes; `headers-match` green; C driver against generated header matches C++ reference.

### Phase 7: `xcfun-kernels` + CPU Batch Path (`Batch<CpuRuntime>`)
**Rationale:** differentiator feature lands; `CpuRuntime` is the only runtime validatable at 1e-12 vs. scalar. Decision: adopt shared-spec option (a).
**Delivers:** `#[cube]` per-point evaluator per functional (78 ports); `DensVarsDev` mirror; `CTaylorDev<F, N>`; `eval_batch_kernel<F: Float>` with `#[comptime]` specialisation; `xcfun-gpu::Batch<'fun, CpuRuntime>` with device-buffer pool; `Functional::eval_vec` dispatches to `Batch<CpuRuntime>` when `nr_points ≥ 64`.
**Avoids:** P3 (CTaylorDev uses identical recursion).
**Exit:** tier-3 parity `eval_vec` on `Batch<CpuRuntime>` matches scalar `eval` within 1e-13 on 10k GGA grid; `eval_vec` on 100k GGA within 10 % of C++ wall-clock.

### Phase 8: CUDA + Wgpu Backends
**Rationale:** primary HPC target first; Wgpu second with `erf`-fallback + f64 gate.
**Delivers:** `Backend` enum; `auto_backend()` with `erf` fallback; device-buffer pool for Cuda / Wgpu; `cubecl-cuda` / `cubecl-wgpu` feature-gated; runtime `Features::SHADER_F64` probe.
**Avoids:** P2 (backend-qualified tolerances), P7 (compile-time scalar-size + runtime f64 gate), P8 (full tier-2+3 on cubecl bump).
**Exit:** tier-3 CUDA ≤ 1e-13 vs. CPU; tier-3 Wgpu (exc. range-separated) ≤ 1e-9 vs. CPU; Wgpu without `SHADER_F64` returns `Err(WgpuNoF64)`.

### Phase 9: Python Bindings + Release
**Rationale:** final consumer surface; release ceremony.
**Delivers:** `xcfun-py` crate (PyFunctional, free functions, numpy interop); `pyproject.toml` with maturin + abi3-py310; Python CI job; CHANGELOG; semver tags; `maturin publish`.
**Avoids:** P14 (PyO3 boundary catches panics).
**Exit:** `pip install xcfun_rs` yields a wheel that passes `pytest`; crates published; wheels to PyPI; `xcfun.h` in release branch.

### Phase Ordering Rationale

- **DAG-forced:** the dependency graph admits exactly one topological order for the library graph.
- **LDA → GGA → metaGGA within functional ports** follows body complexity and mode coverage.
- **Façade + C ABI together** (one API decision; splitting bikesheds).
- **CPU batch before GPU** (`CpuRuntime` is the only batch runtime validatable at 1e-12).
- **Python last** (wheel shape + release should not churn).
- **Scaffolding first** (forcing function; CI gates catch pitfalls before code).

### Research Flags

**Phases likely needing `/gsd-research-phase` during planning:**
- **Phase 1** — deep `ctaylor.hpp` / `tmath.hpp` recursion pattern research; `f64::to_bits` diffing plan against C++ test driver.
- **Phase 5** — `Mode::Contracted` at 5–6 + alias multiplicative semantics + `XCFunctional.cpp:370-405` + 46 aliases including negative-weight `camcompx`.
- **Phase 7** — "shared-spec option (a)" verification; `#[cube]` recursive `CTaylor` multiply on `CpuRuntime`; kernel dispatch table (codegen vs. proc-macro).
- **Phase 8** — cubecl 0.10-pre.3 runtime-feature API, buffer-pool growth, `auto_backend` fallback matrix.

**Phases with standard patterns (skip research-phase):** 0, 2, 3, 4, 6, 9.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Every pin verified on crates.io 2026-04-18; six staleness bumps identified; single pre-release (`cubecl`) has explicit CI isolation; yank history tracked. |
| Features | HIGH | Pinned by vendored C++ source (78 functionals, 46 aliases, 4 params, 31 vars, 3 modes, max order 6, API_VERSION=2); cross-verified against Libxc, ExchCXX, PySCF, Psi4, ADF, ORCA, VASP. |
| Architecture | HIGH | 7-crate layout matches 2026 precedent (cubecl, burn, candle, polars, cubek, rustls-ffi). Shared-spec clarification recommendation aligns with design. |
| Pitfalls | HIGH | 14 pitfalls grounded in `xcfun-master/src/` line references + `docs/design/06/07/12`. MEDIUM only on empirical Wgpu `erf` variance. |

**Overall: HIGH.** No architectural rework implied. Ready for roadmap creation.

### Gaps Deferred to Phase-Level Research

- Shared algebraic spec (option (a) recommended): record at Phase 7 planning.
- `cubecl-cpu` as dev-dep of `xcfun-kernels`: concrete Cargo setup at Phase 7.
- `Send`/`Sync` bounds on `Batch<'fun, R>`: confirm at Phase 7.
- `cargo xtask check-boundaries` scope: implement in Phase 0.
- Wgpu `erf` empirical variance: confirm during Phase 8 on CI adapter matrix.
- PW92C legacy-constants default state: Phase 0 reading of `config.hpp`.

---

## Sources

### Primary (HIGH confidence)

- **xcfun C++ reference (vendored):** `xcfun-master/api/xcfun.h`, `src/XCFunctional.cpp`, `src/functionals/list_of_functionals.hpp`, `src/functionals/aliases.cpp`, `src/densvars.hpp`, `src/specmath.hpp`, `src/config.hpp`, `src/xcint.cpp`, `src/taylor/ctaylor.hpp`, `src/taylor/tmath.hpp`.
- **`.planning/research/STACK.md`, FEATURES.md, ARCHITECTURE.md, PITFALLS.md.**
- **`docs/design/00-overview.md`..`12-design-decisions.md`** — 14-document design brief.
- **`.planning/PROJECT.md`, `CLAUDE.md`.**
- **crates.io** (verified 2026-04-18).
- **CubeCL Book, tracel-ai/cubecl, tracel-ai/cubek** (Context7).
- **PyO3 user guide v0.28.2, rust-numpy Cargo.toml.**

### Secondary (HIGH confidence)

- **burn-dev, huggingface/candle, pola-rs/polars, rustls/rustls-ffi, cool-japan/scirs** — architecture precedents.
- **Libxc, ExchCXX + GauXC, PySCF DFT, Psi4, ADF 2025.1, ORCA 6.1, VASP wiki, gpu4pyscf** — competitor matrix.

### Tertiary (MEDIUM confidence)

- Wgpu `erf` variance — confirm on CI in Phase 8.
- `cubecl` 0.10 stabilization timeline — re-validate at each planning checkpoint.
- A100 order-6 register pressure — validate in Phase 8.

---

*Research completed: 2026-04-19*
*Ready for roadmap: yes*
