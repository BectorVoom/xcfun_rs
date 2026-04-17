<!-- GSD:project-start source:PROJECT.md -->
## Project

**xcfun_rs**

A Rust reimplementation of the xcfun C++ exchange-correlation functional library for density functional theory (DFT). It provides 78 functionals, 39+ aliases, arbitrary-order derivatives (0-6) via automatic differentiation, and GPU batch evaluation via `cubecl`. Targets output compatibility with the C++ version (error <= 1e-12) with C FFI and Python bindings for drop-in replacement.

**Core Value:** Numerical accuracy: every functional must produce results matching C++ xcfun within 1e-12 relative error, across all evaluation modes and derivative orders.

### Constraints

- **Accuracy**: Output must match C++ xcfun within 1e-12 relative error
- **Compatibility**: C FFI must be a drop-in replacement for `api/xcfun.h`
- **Rust Edition**: 2024
- **GPU**: `cubecl` for GPU acceleration (CUDA/Metal/Vulkan backends)
- **Dependencies**: thiserror for library errors, anyhow for apps, bitflags for dependency flags
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Framework
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| Rust (Edition 2024) | 1.85+ | Language | Const generics for CTaylor<T, N>, stack-allocated fixed-size arrays, zero-cost abstractions. Edition 2024 required for latest language features. | HIGH |
| thiserror | 2.0.18 | Structured error types (`XcError`) | Derive macro for `std::error::Error`. v2 supports Edition 2024. De facto standard for library error types (609M+ downloads). | HIGH |
| anyhow | 1.0.x | App-boundary error handling | Used only in benchmarks, examples, and integration tests -- never in library crates. Ergonomic error propagation where structured types are unnecessary. | HIGH |
| bitflags | 2.10.0 | `Dependency` bitmask flags | Type-safe bitflag operations for functional dependency tracking. Zero-cost abstraction over integer bitmasks. 867M+ downloads, mature and stable. | HIGH |
| tracing | 0.1.44 | Structured logging | GPU fallback warnings and optional evaluation tracing. Zero overhead when no subscriber attached. Non-intrusive. | HIGH |
### Automatic Differentiation
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| In-house (`xcfun-ad` crate) | N/A | CTaylor<T, N> tensored polynomial AD | **No existing Rust crate implements xcfun's specific bit-flag indexed multilinear polynomial approach.** The AD engine is ~800 lines, zero dependencies, stack-allocated. Matching C++ xcfun to 1e-12 requires algorithmic identity, not just mathematical equivalence. | HIGH |
### GPU Acceleration
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| cubecl | =0.10.0-pre.3 (pinned) | GPU batch evaluation kernels | Rust-native GPU abstraction: write kernels in Rust, compile to CUDA/Metal/Vulkan/WebGPU. No separate shader language. Generic over `Float` trait (f32/f64). Designed for scientific computing. Only viable multi-backend Rust GPU crate. | MEDIUM |
| cubecl-cuda | 0.10.0-pre.3 | CUDA backend (feature-gated) | Primary HPC backend. NVIDIA GPUs dominant in computational chemistry. | MEDIUM |
| cubecl-wgpu | 0.10.0-pre.3 | WebGPU/Vulkan backend (feature-gated) | Cross-platform fallback. Vulkan on Linux, Metal on macOS via wgpu translation layer. | MEDIUM |
### C FFI
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| cbindgen | 0.29.2 | C header generation from Rust | Mozilla's tool for generating C/C++ headers from `#[no_mangle] extern "C"` functions. Build-time tool (not a runtime dependency). Generates `xcfun.h` matching the original C API. | HIGH |
### Python Bindings
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| pyo3 | 0.28.3 | Rust-Python FFI | De facto standard (no real alternatives). Mature, well-documented, supports Python 3.8+. v0.28 is latest stable (2026-04-02). | HIGH |
| numpy (rust-numpy) | 0.28.0 | NumPy array interop | Companion to PyO3 for zero-copy f64 array exchange. Version must match PyO3 major (0.28.x with 0.28.x). Critical for passing density grids without copying. | HIGH |
| maturin | >=1.0,<2.0 | Python package build tool | Standard build backend for PyO3 projects. Builds wheels, handles cross-compilation. Specified in `pyproject.toml`, not `Cargo.toml`. | HIGH |
### Testing and Benchmarking
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| criterion | 0.8.2 | Statistical benchmarking | Warm-up, outlier detection, regression comparison. Standard for Rust performance work. v0.8 is latest (132M+ downloads). | HIGH |
| approx | 0.5.x | Floating-point comparison | Provides `assert_relative_eq!` for 1e-12 tolerance testing. Essential for numerical accuracy validation against C++ xcfun. | HIGH |
### Development Tools
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| cargo-nextest | latest | Test runner | Parallel test execution, better output formatting. Recommended for large test suites (78 functionals x multiple derivative orders = thousands of tests). | HIGH |
| cargo-deny | latest | Dependency auditing | License compliance and vulnerability scanning. Important for scientific software that may be embedded in larger projects. | MEDIUM |
## Alternatives Considered and Rejected
| Category | Recommended | Alternative | Why Rejected |
|----------|-------------|-------------|--------------|
| AD engine | In-house CTaylor | `autodiff` (Enzyme) | Requires nightly Rust; source-to-source transformation doesn't match xcfun's runtime polynomial approach. Numerical equivalence harder to guarantee. |
| AD engine | In-house CTaylor | `hyperdual` | Only supports up to second derivatives. xcfun needs arbitrary order up to 6. |
| AD engine | In-house CTaylor | `ad` crate | Tape-based AD. Different algorithm means different rounding behavior, defeating the 1e-12 accuracy goal. |
| Numeric traits | Custom `Num` trait | `num-traits` (`Float`) | `Float` assumes IEEE 754 and includes methods meaningless for Taylor polynomials (`is_nan`, `classify`). Custom trait is more appropriate for the CTaylor domain. |
| GPU | cubecl | `wgpu` raw | Requires writing WGSL shaders separately from Rust code. No Rust-native kernel definition. |
| GPU | cubecl | `cudarc` | NVIDIA-only. Eliminates macOS/Metal and Vulkan support. |
| GPU | cubecl | OpenCL (`ocl`) | Declining ecosystem. cubecl covers the same backends with better Rust integration. |
| Parallelism | `std::thread::scope` | `rayon` | Batch evaluation is embarrassingly parallel with uniform work per point. Rayon's work-stealing overhead is unnecessary. Can add later if profiling shows benefit. |
| Linear algebra | None needed | `nalgebra` | Heavyweight dependency for a project that only needs scalar arithmetic. No matrix operations required. |
| Serialization | None needed | `serde` | No serialization requirements. Functionals are configured programmatically. |
| Python bindings | PyO3 | `cpython` | Unmaintained. PyO3 is the clear successor and standard. |
| C FFI | cbindgen | Manual headers | Error-prone. cbindgen auto-generates from Rust source, keeping headers in sync. |
| C FFI | cbindgen | `uniffi` | Designed for mobile/multi-language bindings, not drop-in C API replacement. Adds unnecessary abstraction. |
## Workspace Dependency Map
- `approx` 0.5 -- tolerance assertions
- `criterion` 0.8 -- benchmarks
- `anyhow` 1.0 -- error handling in tests/examples
## Installation
# Workspace Cargo.toml [workspace.dependencies]
# Build tools (not Cargo deps)
## Key Version Constraints
| Constraint | Reason |
|------------|--------|
| cubecl pinned to `=0.10.0-pre.3` | Pre-release API; avoid surprise breakage from semver-incompatible changes |
| pyo3 and numpy must share major version (0.28.x) | rust-numpy tracks PyO3 releases; mismatched versions cause compile errors |
| Rust Edition 2024 (MSRV ~1.85) | Required for latest const generic features and language ergonomics |
| thiserror 2.0 (not 1.x) | v2 supports Edition 2024; v1 does not |
## Risk Assessment
| Technology | Risk | Mitigation |
|------------|------|------------|
| cubecl (pre-release) | API instability, potential breakage on version bump | Pin exact version. Isolate behind feature gate. GPU crate has minimal surface area (batch eval only). |
| cubecl f64 on Vulkan/WebGPU | Some GPUs lack f64 support | CUDA is primary target for HPC. Vulkan/WebGPU are secondary. CPU fallback always available. |
| PyO3 0.28 | Relatively new release | PyO3 is heavily tested (Qiskit, Polars, etc. use it). Low risk. |
| In-house AD engine | Maintenance burden | Scope is small (~800 lines). Algorithm is well-understood from C++ reference. No external dependency risk. |
## Sources
- [PyO3 crates.io](https://crates.io/crates/pyo3) -- v0.28.3 confirmed
- [PyO3 user guide](https://pyo3.rs/v0.28.2/) -- maturin integration docs
- [rust-numpy GitHub](https://github.com/PyO3/rust-numpy) -- v0.28.0 for PyO3 0.28 compatibility
- [cubecl GitHub releases](https://github.com/tracel-ai/cubecl/releases) -- v0.10.0-pre.3 (2026-04-08)
- [CubeCL Book](https://burn.dev/books/cubecl/print.html) -- kernel API, Float trait, Runtime abstraction
- [cbindgen GitHub](https://github.com/mozilla/cbindgen) -- v0.29.2
- [criterion crates.io](https://crates.io/crates/criterion) -- v0.8.2 confirmed
- [thiserror crates.io](https://crates.io/crates/thiserror) -- v2.0.18 confirmed
- [bitflags crates.io](https://crates.io/crates/bitflags) -- v2.10.0 confirmed
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
