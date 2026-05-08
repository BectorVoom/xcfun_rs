<!-- GSD:project-start source:PROJECT.md -->
## Project

**xcfun_rs**

A Rust-from-scratch reimplementation of the xcfun exchange–correlation functional library for density functional theory (DFT). It reproduces the full public C API of the C++ reference (`xcfun-master/`) — 78 functionals, 50+ aliases, 31 variable combinations, 3 evaluation modes, arbitrary derivative orders 0–6 — and extends it with a native Rust API, a C ABI drop-in replacement, Python bindings (pyo3), and a unified CPU/GPU batch evaluation engine implemented in `cubecl` so a single kernel source covers `CpuRuntime`, `CudaRuntime`, and `WgpuRuntime`. The intended audience: computational-chemistry code authors who need a memory-safe, GPU-capable XC library without giving up bit-level parity with the C++ reference.

**Core Value:** Every functional must produce numerical output matching C++ xcfun within relative error ≤ 1.0 × 10⁻¹², across all evaluation modes and derivative orders.

### Constraints

- **Accuracy**: Output must match C++ xcfun within relative error ≤ 1.0 × 10⁻¹² on every `(functional, vars, mode, order, density point)` tuple — the primary contract, non-negotiable
- **Compatibility**: C FFI must be a drop-in replacement for `xcfun-master/api/xcfun.h` — every declared symbol present with matching signature
- **Rust Edition**: 2024, MSRV 1.92 — bumped from 1.85 because `cubecl-zspace 0.10.0` (transitive via `=0.10.0` lockstep pin) declares `rust-version = "1.92"` and cannot be downgraded without breaking the cubecl pin
- **Compiler flags**: no `-Cfast-math`, no reassociation flags; `RUSTFLAGS` empty in CI — fast-math would break the accuracy contract
- **Tech stack**: `thiserror` 2.0.18 (library errors), `anyhow` 1.0 (app boundaries only — no library depends on it), `bitflags` 2.10.0, `cubecl` pinned at `=0.10.0`, `pyo3` 0.28.3 + `numpy` 0.28.0, `cbindgen` 0.29.2, `criterion` 0.8.2, `approx` 0.5 — pinned in CLAUDE.md
- **GPU**: `cubecl` for CPU / CUDA / Wgpu backends; f32 never on the numerical path; Wgpu validated at relaxed 1e-9 tolerance (device `erf` variance), CUDA and CPU at 1e-12
- **Memory**: zero heap allocation on the per-point hot path; batch lifecycle allocates device buffers with powers-of-two growth
- **Licensing**: MPL-2.0 (inherited from xcfun reference)
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## TL;DR — verdict on the existing CLAUDE.md pins
| Pin in CLAUDE.md | Verdict | Note |
|------------------|---------|------|
| Rust Edition 2024, MSRV 1.92 | BUMPED | Was 1.85; bumped to 1.92 because `cubecl-zspace 0.10.0` (locked by `=0.10.0` pin) requires rustc 1.92. Our own code still compiles on 1.85; the bump is forced by the transitive dep. |
| `thiserror 2.0.18` | CONFIRM (pin exact) | 2.0.18 is the current 2.0.x (2026-01-18), bumps MSRV to 1.68. No newer release. |
| `anyhow 1.0.x` | CONFIRM — but adjust floor to `1.0.102` | Latest is 1.0.102 (2026-02-20). Keep as app-boundary only. Do not pin `1.0` loosely; use `>=1.0.98` so CI dedupes. |
| `bitflags 2.10.0` | **CHALLENGE — bump to `2.11.1`** | 2.11.1 (2026-04-14) is the current release; 2.11 brought improved `Flags` trait derive. Move pin. |
| `tracing 0.1.44` | CONFIRM | Current (2025-12-18). Keep `default-features = false` — correct already. |
| `tracing-subscriber 0.3` (validation only) | CONFIRM — bump pin floor to `0.3.23` | 0.3.21 was yanked; lock `>=0.3.22`. |
| `cubecl =0.10.0` | CONFIRM — stable 0.10.0 shipped 2026-05-07 | All five crates (`cubecl`, `cubecl-cpu`, `cubecl-hip`, `cubecl-cuda`, `cubecl-wgpu`) published 0.10.0 on 2026-05-07 (not yanked). The `f64` cell in the official CubeCL feature matrix is marked **"?" (support may vary)** for CUDA and WGPU-SPIR-V, and **"Not supported"** for WGPU-WGSL. Runtime probing and CI gating remain required (see §Risk Assessment). |
| `pyo3 0.28.3` + `numpy 0.28.0` | CONFIRM | 0.28.3 (2026-04-02) is current PyO3; `numpy 0.28.0` (2026-02-08) tracks PyO3 0.28.x. Yanked versions on both sides (pyo3 0.28.0/0.28.1 yanked) — the `=0.28.3` / `=0.28.0` pins already avoid them. |
| `cbindgen 0.29.2` | CONFIRM | 0.29.2 (2025-10-21) is current. Keep. |
| `criterion 0.8.2` | CONFIRM | 0.8.2 (2026-02-04) is current. Keep. |
| `approx 0.5.x` | CONFIRM for now; DO NOT adopt `0.6.0-rc2` yet | 0.5.1 from 2022 is the current **stable**. 0.6.0-rc2 is a release candidate (2026-02-05); do not gate the 1e-12 parity contract on an RC. |
| `cc 1.1` (validation build-dep) | **CHALLENGE — bump to `1.2` (current 1.2.60)** | `cc 1.1` is two minor versions stale. `cc 1.2.60` (2026-04-10) includes C++20 handling improvements and faster parallel compile — relevant because the validation crate compiles the full `xcfun-master/src/**/*.cpp` tree. |
| `proptest 1.6` | **CHALLENGE — bump to `1.11`** | 1.11.0 (2026-03-24). Non-breaking improvements; 1.6 dates to mid-2025. |
| `rstest 0.23` | **CHALLENGE — bump to `0.26.1`** | 0.26.1 (2025-07-27). 0.23 → 0.26 is non-breaking at the `#[rstest]` macro surface we need. |
| `rand_xoshiro 0.7` | **CHALLENGE — bump to `0.8`** | 0.8.0 (2026-02-02). We want the rand 0.9 compat story in 0.8. |
| `maturin >=1.0,<2.0` | CONFIRM — tighten floor to `>=1.12` | 1.13.1 is current (2026-04-09); older 1.x lacks abi3-py310 wheel improvements. |
| `serde_json 1.0` | CONFIRM | 1.0.149 current. Loose `1.0` caret is fine. |
## Recommended Stack
### Core language and error model
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust (Edition 2024) | MSRV 1.92, stable channel | Language | Const generics `[T; 1 << N]` for `CTaylor<T, N>`; Edition 2024 for the 2024 module-resolution + `Gen`/let-else + 2024-specific trait resolution. MSRV bumped from 1.85 → 1.92 by `cubecl-zspace 0.10.0` transitive requirement (see Plan 06-N6). No nightly features needed. |
| `thiserror` | `=2.0.18` | Library-crate error derive | Zero-runtime-cost `std::error::Error` derive; de facto standard for library errors (907M+ downloads). v2 is Edition-2024-compatible; v1 is not. Used in every lib crate, especially `xcfun-core::XcError`. |
| `anyhow` | `^1.0.102` | App-boundary error handling | Used in `validation/`, `xtask/`, `benches/`, `examples/` **only** — never by any crate in the `xcfun-*` library graph (CI-enforced via `cargo xtask check-no-anyhow`). Ergonomic context-attachment for test harnesses where structured errors buy nothing. |
| `bitflags` | `=2.11.1` | `Dependency` bitmask flags | Replaces the C++ `Dependency` enum-class bitmask used in `xcfun-master/src/functional.hpp`. v2.11 (2026-02/04) improves the `Flags` trait derive and keeps zero-cost integer representation. |
| `tracing` | `=0.1.44` with `default-features = false` | Optional structured logging on batch boundaries | Zero overhead when no subscriber is attached (our default). Wanted in `xcfun-gpu` and `xcfun-capi` (behind feature) for Wgpu-fallback warnings and `validation --diagnose` intermediate dumps. Never on the per-point hot path. |
### Automatic differentiation (in-house — no crate picked)
| Component | Decision | Why |
|-----------|----------|-----|
| `xcfun-ad` crate (internal) | **Build; reject every existing Rust AD crate** | The 1e-12 contract demands _algorithmic identity_ with xcfun's bit-flag-indexed multilinear-polynomial representation (the `ctaylor` in `xcfun-master/src/taylor/ctaylor.hpp`). None of `autodiff` / `hyperdual` / `ad` / `num-dual` reproduces that structure. Re-implementing it in ~800 LOC with zero runtime dependencies is cheaper — and provably identical — than bending any third-party engine into shape. Design doc §3 of `07-accuracy-strategy.md` is explicit about this. |
| Internal `Num` trait | Custom, defined in `xcfun-ad` | `num-traits::Float` carries IEEE-754 semantics (`is_nan`, `classify`) that are meaningless for a Taylor polynomial type. Custom trait is narrower and safer. |
| Crate | Current ver | Why rejected |
|-------|-------------|--------------|
| `autodiff` (Enzyme-based) | 0.7.x | Source-to-source transformation; can't match xcfun's runtime polynomial accumulation order. Requires nightly Rust for Enzyme backend. |
| `hyperdual` | 1.3.x | 1st/2nd order only; we need order 0..6. |
| `ad` | 0.x | Tape-based reverse-mode; completely different algorithm, different rounding pattern, 1e-12 parity unprovable. |
| `num-dual` | 0.11.x | Dual/hyperdual only; same order-limitation problem. |
| `rust_ad` | 0.x (unmaintained) | Unmaintained; unsuitable. |
### GPU / unified kernel layer
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `cubecl` | **`=0.10.0`** (hard pin) | Kernel DSL + `Runtime` abstraction | The only Rust-native kernel DSL that (a) compiles one source to CPU, CUDA, ROCm, Metal, Vulkan, WebGPU; (b) is generic over a `Float` trait so the same `#[cube]` body works on f32 and f64; (c) reached stable 0.10.0 on 2026-05-07. Published by `tracel-ai` (Burn ML maintainers). |
| `cubecl-cpu` | `=0.10.0` | `CpuRuntime` backend | Executes the same kernel on the host for CI, development, and the always-on fallback. Critical: this is what lets us say "one kernel source, tested on CPU first, then re-run on GPU." |
| `cubecl-cuda` | `=0.10.0` (feature `cuda`) | `CudaRuntime` — primary HPC target | NVIDIA is the dominant HPC GPU in computational chemistry. PTX math.h intrinsics align with libm in the last 1–4 ULPs, tight enough for 1e-12 with the budget in `07-accuracy-strategy.md §6`. |
| `cubecl-wgpu` | `=0.10.0` (feature `wgpu`) | `WgpuRuntime` — portability fallback | Provides Vulkan/Metal/WebGPU coverage. **f64 is explicitly "?" (conditional) on SPIR-V backends and "Not supported" on WGSL** (from the official CubeCL feature matrix). Design already reflects this: `erf`-bearing functionals are CPU-only on Wgpu, Wgpu parity relaxed to 1e-9. |
| Alternative | Current ver | Why rejected for this project |
|-------------|-------------|-------------------------------|
| `wgpu` (raw) + WGSL | 23.x | Need to hand-author WGSL shaders in addition to Rust; two languages, two sources, no guarantee of algorithmic parity between them. Defeats "single source" goal (G3). |
| `cudarc` | 0.14.x | CUDA-only; eliminates Wgpu/Metal fallback and the `CpuRuntime` unification trick. Fine library, wrong scope. |
| `rust-gpu` | 0.10.x (unmaintained post-fork) | Compiles Rust to SPIR-V, but limited to SPIR-V targets; no CUDA; f64 support is worse than CubeCL's. |
| `krnl` / `ocl` | — | OpenCL-flavoured; declining ecosystem; cubecl already covers the same backends in Rust-native DSL. |
| `burn` tensor ops | — | Full ML tensor framework; dependency blast radius is 200+ crates; we only need the kernel DSL. Using `cubecl` directly keeps the dependency closure ~20 crates. |
### C FFI and header generation
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `cbindgen` | `=0.29.2` | Generate `xcfun.h` from `#[no_mangle] extern "C"` in `xcfun-capi` | Mozilla-maintained, build-time only, produces C/C++-11 headers matching the exact symbol set in `xcfun-master/api/xcfun.h`. The `headers-match` CI job diffs the generated header against the reference. |
| `cc` | **bump to `^1.2.60`** | Compile `xcfun-master/src/**/*.cpp` for the validation harness | Native C/C++ build driver. `1.2.x` added parallelisation improvements and better C++20 handling that matter for the full functionals tree. `parallel` feature already enabled in `validation/Cargo.toml`. |
### Python bindings
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `pyo3` | `=0.28.3` with features `["extension-module", "abi3-py310"]` | Python<->Rust FFI | Single maintained option; PyO3 0.28.3 is current stable (2026-04-02); 0.28.0 and 0.28.1 were yanked, so the `=0.28.3` pin is load-bearing. `abi3-py310` gives us one wheel for CPython ≥ 3.10 (matches the `requires-python = ">=3.10"` in `pyproject.toml`). |
| `numpy` (rust-numpy) | `=0.28.0` | Zero-copy NumPy `f64` array interop | Companion crate; release cadence locks to PyO3 major-minor. 0.28.0 explicitly depends on `pyo3 = "0.28.0"` (confirmed in its `Cargo.toml`); compatible with our pyo3 `=0.28.3` since pyo3 0.28 is caret-compatible internally. `PyArray1<f64>`/`PyArrayDyn<f64>` are the exact types we need for density grids and result buffers. |
| `maturin` | **bump floor to `>=1.12, <2.0`** (current `1.13.1`) | Build backend, declared in `pyproject.toml` | Standard PyO3 build tool; 1.12+ has improved abi3-py310 wheel layout and cross-compilation. Declared in `pyproject.toml`, not `Cargo.toml` — correct. |
### Testing and benchmarking
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `approx` | `=0.5.1` | `assert_relative_eq!(a, b, max_relative = 1e-12)` for the parity tests | The assertion macro under which the 1e-12 contract is checked at the unit-test level. **Hold at 0.5 — 0.6.0-rc2 is a release candidate, not stable.** |
| `criterion` | `=0.8.2` with `default-features = false, features = ["html_reports"]` | Statistical benchmarks for `eval_vec` throughput, CPU vs GPU batch size curves | Standard Rust benchmarking; outlier detection matters because GPU timings are noisy. The `html_reports` feature gives us `target/criterion/*/report.html` for the PR comment bot. |
| `proptest` | **bump to `=1.11.0`** | Ring-axiom and derivative-identity property tests on `CTaylor` | Hypothesis-style property tests. 1.11 improves shrinking for nested structures, useful for `CTaylor<T, N>` shrinking. |
| `rstest` | **bump to `=0.26.1`** | Parameterised tests across `(vars, mode, order)` tuples | Fixture/parameterised testing framework. 0.26.1 improves `#[case]` fixture syntax around async/generics. |
| `rand_xoshiro` | **bump to `=0.8.0`** | Deterministic RNG for fixture generation (fixed seed) | Xoshiro256++; required for reproducible 10 000-point regression grids (fixture `seed: 0x1234abcd` pattern from `07-accuracy-strategy.md §5`). 0.8 aligns with `rand 0.9`. |
| `serde_json` | `^1.0.149` | JSONL validation reports (`validation/report.jsonl`) | `report.jsonl` format in §5 of the accuracy doc. No alternative considered. |
### Development tools (not Cargo deps)
| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo-nextest` | Test runner | Thousands of parametric tests (78 functionals × up to 31 vars × 3 modes × 5 orders). Nextest's parallelism and retry UX beat `cargo test` materially. Install via `cargo install cargo-nextest --locked` in CI. |
| `cargo-deny` | License + advisory gate | Allowlist = `MPL-2.0, MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode-DFS-2016`. GPL rejected (MPL-2.0 compat matters for xcfun's downstream embedders). |
| `cargo-criterion` | Criterion driver wrapper | Enables `--baseline pr` for PR-time regression comparison. |
| `cargo-udeps` | Unused-dependency detection | Weekly CI job. |
| `pre-commit` | `fmt`, `clippy`, `check-no-anyhow` | Enforces the library/app boundary invariant from PROJECT.md constraints. |
| `cargo audit` | Vulnerability scan | Runs on every PR. |
| `rustup` toolchain pinned in `rust-toolchain.toml` | `channel = "1.85"`, `components = ["rustfmt", "clippy"]`, `profile = "minimal"` | No nightly features reachable. |
## Installation
# Library-graph deps (the only deps allowed inside xcfun-ad / xcfun-core / xcfun-kernels / xcfun-gpu / xcfun-rs / xcfun-capi / xcfun-py)
# App-boundary deps (validation / xtask / benches / examples / dev-deps only)
# Python
# Rust developer tooling
## Alternatives Considered
| Category | Recommended | Alternative | When (if ever) to use the alternative |
|----------|-------------|-------------|---------------------------------------|
| AD engine | In-house `xcfun-ad` (`CTaylor<T, N>`) | `autodiff` (Enzyme) | Never for this project. Valid choice for a _new_ functional library where we can re-derive tolerances; impossible when the spec is "match xcfun to 1e-12." |
| AD engine | In-house | `hyperdual` | If the project ever caps at 2nd derivatives. xcfun does not. |
| AD engine | In-house | `num-dual` | Same limitation. |
| Numeric trait | Custom `Num` (in `xcfun-ad`) | `num-traits::Float` | Never; `Float` carries irrelevant IEEE-754 semantics. |
| GPU kernel DSL | `cubecl` | `wgpu` raw + WGSL | Only if cubecl is abandoned upstream — at that point we would need dual-source kernels with a parity check between the two. Do not start here. |
| GPU kernel DSL | `cubecl` | `cudarc` (CUDA-only) | If the project is ever scoped CUDA-only (drop Wgpu fallback). Saves one dependency, loses portability. |
| GPU kernel DSL | `cubecl` | `burn` | If the caller already ships Burn tensors. We don't; pulling Burn adds 150+ transitive crates for tensor features we don't use. |
| Parallelism (host) | `std::thread::scope` (when we need it at all; currently CPU parallelism comes from `CpuRuntime`) | `rayon` | If profiling shows cubecl-cpu's built-in scheduling under-uses cores for small batches (< 64 points). Re-evaluate post-MVP. |
| Linear algebra | None | `nalgebra`/`ndarray` | No matrix operations in xcfun — every inner op is scalar Taylor algebra. Don't add a heavyweight dep for zero benefit. |
| Serialization | None in library; `serde_json` in validation | `serde` + `ciborium` | If fixture size ever becomes a problem; CBOR would compress better than JSON. Not needed in v1. |
| Python | `pyo3` + rust-numpy | `cpython` | Never — unmaintained. |
| Python wheels | `maturin` | `setuptools-rust` | Only if we need simultaneous multi-backend (pyo3 + cffi + uniffi) wheels, which we don't. |
| C FFI headers | `cbindgen` | Hand-written `xcfun.h` | Only if cbindgen breaks on an `extern "C"` pattern we need — unlikely, every pattern in `xcfun-master/api/xcfun.h` is representable in cbindgen. |
| C FFI headers | `cbindgen` | `uniffi` | uniffi targets mobile/multi-language generated bindings, not drop-in replacement of an existing C header. Wrong tool. |
| Test runner | `cargo-nextest` | `cargo test` | OK for solo-dev local loops; nextest mandatory in CI for throughput. |
| Property tests | `proptest` | `quickcheck` | Proptest's shrinking is better for nested structures (`CTaylor<CTaylor<f64, 2>, 3>`). |
| Parameterized tests | `rstest` | `test-case` | Rstest composes better with property tests; large ecosystem overlap. Pick one — `rstest`. |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `-Cfast-math`, `-ffast-math`, `-funsafe-math-optimizations` | Compiler re-associates floating-point operations; the 1e-12 parity contract fails silently | Empty `RUSTFLAGS` in CI + a `check-rustflags` xtask step |
| `-Ctarget-cpu=native` in release/CI | Non-reproducible builds; FMA emission changes rounding | Explicit target triples per platform |
| `f32` anywhere on the numerical path | f32 unit-round-off ≈ 6e-8 — cannot meet 1e-12 | `f64` throughout; `Num` intentionally NOT implemented for `f32` in `xcfun-ad` |
| `rayon` inside per-point evaluation | Rayon's work-stealing overhead per cell wastes cycles on a branch-light kernel; GPU batch already handles parallelism | `cubecl-cpu` for multicore batch eval; fall back to `std::thread::scope` only for coarse orchestration if profiling demands |
| `num-traits::Float` in `CTaylor` | Carries `is_nan`/`classify`/`FloatCore` semantics meaningless for a polynomial | Custom `Num` trait owned by `xcfun-ad` |
| Any AD crate (autodiff, hyperdual, ad, num-dual) | Different rounding pattern → loses 1e-12 parity | In-house `xcfun-ad` |
| `cpython` (not PyO3) | Unmaintained; last release pre-2023 | `pyo3` |
| `serde` in `xcfun-ad`/`xcfun-core` | No serialization requirement in the library graph; keeps `no_std` story clean | `serde_json` in `validation/` and `xtask/` only |
| `log` crate | Duplicates `tracing`'s role; `tracing` is strictly richer | `tracing` + `tracing-subscriber` (subscriber only in validation) |
| `once_cell` | Not needed with Rust 1.85 — `std::sync::LazyLock` is stable | `std::sync::LazyLock` |
| `lazy_static` | Same as above | `std::sync::LazyLock` |
| `anyhow` in any `xcfun-*` crate | Violates the library/app error model; CI must block | `thiserror::Error` + `XcError`; use `anyhow` only in `validation/`, `xtask/`, `benches/`, `examples/` |
| `bincode` for compilation caches | Less stable across versions than CBOR; CubeCL itself switched away | `ciborium` (already pulled in via `cubecl`) if we ever need a cache format |
| `cubecl` 0.9.0 (older API) | Missing the 0.10 features around arena allocation, staging buffers, scalar/metadata refactor, and the `Validate` execution mode — all of which we rely on in `06-cubecl-strategy.md` | `=0.10.0` |
| `approx 0.6.0-rc2` | Release candidate; one yanked predecessor (0.6.0-rc1, yanked 2026-02-06). Do not gate an accuracy contract on an RC. | Stay on `approx 0.5.1` until `0.6.0` ships stable. |
## Stack Patterns by Variant
- Feature flags: default (`cpu` on `xcfun-gpu`), no `cuda`, no `wgpu`.
- Result: `cubecl-cuda` / `cubecl-wgpu` not in the dependency graph; no CUDA toolkit required to build.
- This is the recommended default; CI's `fmt`/`clippy`/`test` jobs should run this.
- `cargo build --features xcfun-rs/cuda` (or `--all-features`).
- CUDA toolkit present; `cubecl-cuda` compiles kernels to PTX at first launch.
- Accuracy envelope: CPU and CUDA both validated at 1e-12 vs. the C++ reference.
- This is the stress-tested path for scientific deployments.
- `cargo build --features xcfun-rs/wgpu`.
- Wgpu backend active. **Accuracy envelope: 1e-9 vs. C++ reference**, not 1e-12.
- Range-separated functionals using `erf` (`ldaerfx`, `ldaerfc`, `beckecamx`, `beckesrx`, `ldaerfc_jt`) automatically fall back to CPU — `xcfun-gpu::auto_backend` inspects `Dependency::ERF` and routes.
- Runtime probe must also reject Wgpu devices without `wgpu::Features::SHADER_F64`. If the device lacks f64, accept CPU fallback rather than downgrading to f32 (which would violate the numerical contract).
- `pip install xcfun-rs` (via maturin-built wheel).
- No CUDA/Wgpu by default in the wheel — CPU only for out-of-the-box portability.
- A separate `xcfun-rs-cuda` wheel can be built with the `cuda` feature enabled; document the extra install step.
- `cargo build -p xcfun-capi --release`.
- Output: `target/release/libxcfun_capi.so` + regenerated `xcfun.h` (diff-matched to the reference).
- ABI compatibility verified by the `headers-match` CI job and the `cc`-based golden tests in `xcfun-capi/tests/c_abi.rs`.
## Version Compatibility
| Package A | Compatible with | Notes |
|-----------|-----------------|-------|
| `pyo3 =0.28.3` | `numpy =0.28.0` | rust-numpy's `Cargo.toml` depends on `pyo3 = "0.28"`; caret-compatible with 0.28.3. Mismatched majors (e.g. pyo3 0.27 + numpy 0.28) fail to compile. **Treat as a single-atomic version bump.** |
| `pyo3 =0.28.3` | `maturin >=1.12, <2.0` | Maturin 1.12+ ships the abi3-py310 wheel layout compatible with PyO3 0.28. Older maturin (< 1.10) has known abi3 issues with PyO3 0.28. |
| `pyo3 0.28.x` | CPython ≥ 3.10 | `abi3-py310` feature: one wheel covers 3.10, 3.11, 3.12, 3.13. |
| `cubecl =0.10.0` | `cubecl-cpu =0.10.0`, `cubecl-cuda =0.10.0`, `cubecl-wgpu =0.10.0`, `cubecl-hip =0.10.0` | **All five must move in lockstep** or compile fails. `=` equality pins are mandatory, not merely recommended. |
| `cubecl =0.10.0` | Rust 1.85+ | Uses Edition 2024 types. |
| `cubecl-cuda =0.10.0` | CUDA toolkit ≥ 12.0 (recommended 12.4) | Driver requirement: driver ≥ 535 for CUDA 12.4. |
| `cubecl-wgpu =0.10.0` | Vulkan 1.2+ / Metal 2+ / WebGPU (Chrome 113+, Firefox 121+) | **f64 only where device reports `Features::SHADER_F64`**. On machines without that feature, the backend is unsafe for the numerical path. |
| `thiserror =2.0.18` | Rust ≥ 1.68 | MSRV bump from 2.0.17's 1.61 → 2.0.18's 1.68. Fine for us (MSRV 1.85). |
| `bitflags =2.11.1` | Rust ≥ 1.56 | Comfortable margin. |
| `cbindgen =0.29.2` | Rust ≥ 1.74 | Comfortable margin. |
| `cc ^1.2.60` | Any Rust (build-time only); benefits from parallel jobs (needs `parallel` feature) | `parallel = true` speeds up compiling the whole `xcfun-master/src/**/*.cpp` tree by 4–8×. |
| `tracing =0.1.44` | `tracing-subscriber 0.3.22+` | `tracing-subscriber 0.3.21` is yanked — lock the subscriber to `>=0.3.22` (we recommend `=0.3.23`). |
| `criterion =0.8.2` | Rust ≥ 1.74 | Comfortable margin. |
| `approx =0.5.1` | Any Rust 1.x | No MSRV pressure. |
## Key Version Constraints (why each `=` pin exists)
| Constraint | Why it's exact (`=x.y.z`) rather than caret (`^x.y.z`) |
|------------|------------------------------------------------------|
| `cubecl =0.10.0` | Exact pin ensures `cargo update` never silently bumps to 0.10.1 or 0.11.x before we validate the accuracy contract across the new version. |
| `cubecl-{cpu,hip,cuda,wgpu} =0.10.0` | All five cubecl crates cross-reference internal types; any mismatch is a hard compile error. |
| `pyo3 =0.28.3` | pyo3 0.28.0 and 0.28.1 are yanked. Without `=`, Cargo could regenerate `Cargo.lock` onto a yanked version (yanked versions remain resolvable for existing lockfiles). |
| `numpy =0.28.0` | Must match PyO3 0.28.x; a future numpy 0.29.x would demand pyo3 0.29.x, and we want the bump to be explicit. |
| `thiserror =2.0.18` | MSRV bumped at this version; later 2.0.x patches may bump again. Lock for reproducibility. |
| `cbindgen =0.29.2` | Generated header layout is an ABI surface under test by `headers-match`. Lock to keep the diff stable. |
| `criterion =0.8.2` | Benchmark HTML report format changed in 0.8 → want deterministic report layout for CI. |
| `approx =0.5.1` | Assertion macros are load-bearing in accuracy tests. Lock to avoid any surprise in tolerance semantics. |
| `rand_xoshiro =0.8.0` | RNG algorithm is an input to fixture generation. A minor-version drift in 0.8.x could change sequence; lock and regenerate fixtures explicitly if bumped. |
| `rstest =0.26.1` | Macro-surface crate; lock to avoid `#[case]` signature drift between patch versions. |
| `proptest =1.11.0` | Shrinking algorithm differences can surface different failing inputs. Lock for reproducibility of any CI failure. |
## Risk Assessment
| Risk | Probability | Severity | Mitigation |
|------|-------------|----------|------------|
| **cubecl 0.10.1 or 0.11.x breaks the accuracy contract** | LOW | HIGH | 1. `=0.10.0` hard pin prevents silent bumps. 2. The entire cubecl surface is contained in `xcfun-kernels` and `xcfun-gpu` (two crates, ~1 kLoC). 3. Validation harness is CPU-first: a cubecl regression breaks GPU CI but not CPU CI, so the numerical contract is still enforced while we investigate. 4. Budget a "cubecl bump" sub-phase for any future version bump; re-run Tier 2 + Tier 3 harnesses; block merge if rel-error > 1e-12 anywhere. |
| **cubecl f64 on CUDA degrades on a future hardware/driver combination** | LOW | HIGH | The CubeCL feature matrix flags CUDA f64 as "?". Mitigate by: (a) probe `client.properties().feature_enabled(Feature::Type(f64))` at runtime before launching; (b) refuse to launch (and emit a `tracing::error!`) if unavailable; (c) the validation harness is the final gate — a silent accuracy drop cannot reach production. |
| **cubecl-wgpu on WGSL lacks f64 entirely** | CERTAINTY | HIGH only if someone tries to use it on the numerical path | Documented already; Wgpu path is explicitly 1e-9-tolerance and CPU-forced for range-separated functionals. Design accounts for this. |
| **pyo3 0.28.4+ yanked like 0.28.0/0.28.1 were** | LOW | MEDIUM | Pin `=0.28.3`; monitor yanks via `cargo audit` on every PR. |
| **rust-numpy lags behind pyo3 on a future release** | MEDIUM | LOW | Already an explicit single-atomic-bump rule. Cargo resolution will fail fast if mismatched. |
| **cbindgen changes header output format** | LOW | MEDIUM | `headers-match` CI diffs against `xcfun-master/api/xcfun.h`. Any cbindgen-side change causing a diff blocks merge. |
| **thiserror 2.0.19+ bumps MSRV beyond 1.85** | LOW | LOW | Revisit MSRV at that point; we have headroom. Pin is `=2.0.18` anyway. |
| **`approx 0.6.0` changes tolerance semantics when it stabilises** | LOW | MEDIUM | Stay on `=0.5.1` until 0.6.0 is published stable + reviewed. |
| **`cc 1.2.x` ABI change in the generated object files** | VERY LOW | MEDIUM | `cc` produces object code, not a library-surface API. Still, pin to `^1.2.60` (not `1.x`) so bumps are minor and observable. |
| **In-house `xcfun-ad` has an algorithmic bug vs. the C++ reference** | MEDIUM until Tier 2 harness passes | HIGH | Four-tier test strategy in `07-accuracy-strategy.md`. Tier 1 self-tests catch gross errors, Tier 2 parity harness catches per-element drift vs. C++. Numerical-parity CI gate is non-negotiable. |
## Sources
- [crates.io — pyo3](https://crates.io/crates/pyo3) — v0.28.3 confirmed (2026-04-02); 0.28.0/0.28.1 yanked, 0.28.2 (2026-02-18) superseded.
- [crates.io — numpy (rust-numpy)](https://crates.io/crates/numpy) — v0.28.0 confirmed (2026-02-08).
- [rust-numpy Cargo.toml on main](https://github.com/PyO3/rust-numpy/blob/main/Cargo.toml) — verifies `pyo3 = "0.28.0"` dependency declaration; caret-compatible with our `=0.28.3`.
- [crates.io — cubecl](https://crates.io/crates/cubecl) — v0.10.0 stable confirmed (2026-05-07); all five crates (cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu) published in lockstep, not yanked.
- [crates.io — cubecl-cuda](https://crates.io/crates/cubecl-cuda), [cubecl-cpu](https://crates.io/crates/cubecl-cpu), [cubecl-wgpu](https://crates.io/crates/cubecl-wgpu), [cubecl-hip](https://crates.io/crates/cubecl-hip) — all 0.10.0 confirmed 2026-05-07.
- [CubeCL feature matrix (cubecl-book/core-features/features.md)](https://github.com/tracel-ai/cubecl/blob/main/cubecl-book/src/core-features/features.md) via Context7 `/tracel-ai/cubecl` — authoritative source for the f64 "?" / "Not supported" flags per backend.
- [CubeCL release notes](https://github.com/tracel-ai/cubecl/releases) — 0.10.0 stable (Validate mode, arena memory, staging buffers, scalar/metadata refactor, ciborium cache).
- [crates.io — thiserror](https://crates.io/crates/thiserror) — v2.0.18 confirmed (2026-01-18); MSRV 1.68.
- [crates.io — bitflags](https://crates.io/crates/bitflags) — v2.11.1 current (2026-04-14); 2.10.0 is one minor stale.
- [crates.io — cbindgen](https://crates.io/crates/cbindgen) — v0.29.2 confirmed (2025-10-21).
- [crates.io — criterion](https://crates.io/crates/criterion) — v0.8.2 confirmed (2026-02-04).
- [crates.io — approx](https://crates.io/crates/approx) — v0.5.1 latest stable (2022-01-23); 0.6.0-rc2 (2026-02-05) is an RC, not stable; 0.6.0-rc1 yanked.
- [crates.io — cc](https://crates.io/crates/cc) — v1.2.60 latest (2026-04-10); `1.1` pin is stale.
- [crates.io — anyhow](https://crates.io/crates/anyhow) — v1.0.102 current (2026-02-20).
- [crates.io — tracing](https://crates.io/crates/tracing) — v0.1.44 current (2025-12-18); 0.1.42 yanked.
- [crates.io — tracing-subscriber](https://crates.io/crates/tracing-subscriber) — v0.3.23 current (2026-03-13); 0.3.21 yanked.
- [crates.io — proptest](https://crates.io/crates/proptest) — v1.11.0 current (2026-03-24).
- [crates.io — rstest](https://crates.io/crates/rstest) — v0.26.1 current (2025-07-27).
- [crates.io — rand_xoshiro](https://crates.io/crates/rand_xoshiro) — v0.8.0 current (2026-02-02).
- [crates.io — maturin](https://crates.io/crates/maturin) — v1.13.1 current (2026-04-09).
- [crates.io — serde_json](https://crates.io/crates/serde_json) — v1.0.149 current (2026-01-06).
- Internal: `docs/design/06-cubecl-strategy.md`, `docs/design/07-accuracy-strategy.md`, `docs/design/10-build-and-dependencies.md`, root `CLAUDE.md`, `.planning/PROJECT.md`.
## Confidence
| Area | Confidence | Reason |
|------|------------|--------|
| Core error/language (`thiserror`, `anyhow`, `bitflags`, Rust 2024) | HIGH | Verified on crates.io 2026-04; de facto standards with 100M+ downloads each. |
| AD engine (in-house) | HIGH | Exhaustively searched crates.io 2026-04; no crate replicates the bit-flag multilinear polynomial structure. Algorithmic identity with `xcfun-master/src/taylor/ctaylor.hpp` is the only route to 1e-12 parity. |
| GPU layer (`cubecl` family) | MEDIUM | Only viable multi-backend Rust kernel DSL; stable 0.10.0 pinned; f64 is conditionally supported per backend — design docs already handle the limitation. CI must permanently guard the accuracy contract across any future version bumps. |
| C FFI (`cbindgen`, `cc`) | HIGH | `cbindgen` is Mozilla-maintained and directly targets our drop-in-header use case; `cc` is Rust's standard native-compile driver. |
| Python (`pyo3`, `numpy`, `maturin`) | HIGH | Single dominant option with clear version-lock rules; zero-copy `f64` array interop is exactly the numpy crate's strength. |
| Testing (`criterion`, `approx`, `proptest`, `rstest`) | HIGH | Standard Rust scientific-testing stack; all currently maintained with regular 2025–2026 releases. |
| Tooling (`cargo-nextest`, `cargo-deny`, `maturin`) | HIGH | Widely used in production Rust projects; no realistic alternatives at the tier we need. |
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
