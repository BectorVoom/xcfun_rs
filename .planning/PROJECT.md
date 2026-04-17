# xcfun_rs

## What This Is

A Rust reimplementation of the xcfun C++ exchange-correlation functional library for density functional theory (DFT). It provides 78 functionals, 39+ aliases, arbitrary-order derivatives (0-6) via automatic differentiation, and GPU batch evaluation via `cubecl`. Targets output compatibility with the C++ version (error <= 1e-12) with C FFI and Python bindings for drop-in replacement.

## Core Value

Numerical accuracy: every functional must produce results matching C++ xcfun within 1e-12 relative error, across all evaluation modes and derivative orders.

## Requirements

### Validated

- ✓ Core type system (`DensityVars<T>`, `EvalMode`, `VarType`, `FunctionalId`, `Dependency`, `XcError`) — Phase 1
- ✓ Automatic differentiation engine (`CTaylor<T, N>`, `Num` trait, transcendentals, composition) — Phase 1

### Active
- [ ] 5 LDA functionals (SlaterX, Vwn3C, Vwn5C, Pz81C, Pw92C)
- [ ] ~38 GGA functionals (exchange + correlation, PBE/Becke/LYP families)
- [ ] ~20 meta-GGA + kinetic functionals (TPSS, SCAN families, TfK, Tw, etc.)
- [ ] 15 hybrid functionals (M05/M06 family) + all 39+ aliases
- [ ] Evaluation pipeline (partial derivatives, potential, contracted modes)
- [ ] GPU batch evaluation via `cubecl` (kernels, buffer caching, CPU/GPU fallback)
- [ ] C FFI matching xcfun's public C API (`xcfun-ffi`)
- [ ] Python bindings via PyO3 (`xcfun-python`)
- [ ] Benchmarks and performance optimization (within 1.2x of C++)

### Out of Scope

- New functionals not in C++ xcfun -- this is a reimplementation, not an extension
- Web API or REST interface -- library-only
- GUI or visualization tools -- this is a computational kernel
- Support for derivative orders > 6 -- matches C++ xcfun maximum

## Context

- Reimplementing the well-established C++ xcfun library (https://github.com/dftlibs/xcfun)
- 7-crate Rust workspace: xcfun-core, xcfun-ad, xcfun-functionals, xcfun-eval, xcfun-gpu, xcfun-ffi, xcfun-python
- Architecture: 5 layers (Public API, Evaluation Pipeline, Functional, Core, Acceleration)
- Automatic differentiation via Taylor expansion with const generic order (`CTaylor<T, N>`)
- Enum dispatch for 78 functionals (not trait objects -- `energy<T>()` is generic, not object-safe)
- GPU acceleration isolated behind feature gate via `cubecl`
- Existing Cargo.toml with dependencies: anyhow, cubecl 0.10.0-pre.3, thiserror, tracing
- Rust Edition 2024
- Comprehensive design documents in `docs/design/` (12 documents covering architecture, data structures, traits, AD engine, GPU strategy, processing flows, performance, error handling, testing, dependencies, design decisions, milestones)

## Constraints

- **Accuracy**: Output must match C++ xcfun within 1e-12 relative error
- **Compatibility**: C FFI must be a drop-in replacement for `api/xcfun.h`
- **Rust Edition**: 2024
- **GPU**: `cubecl` for GPU acceleration (CUDA/Metal/Vulkan backends)
- **Dependencies**: thiserror for library errors, anyhow for apps, bitflags for dependency flags

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Fine-grained workspace (7 crates) | AD engine is reusable, GPU is heavy dep behind feature gate, functionals are large by volume | -- Pending |
| Enum dispatch over trait objects | `energy<T>()` is generic -- not object-safe; enum allows compile-time monomorphization | -- Pending |
| Custom `Num` trait over num-traits | CTaylor needs domain-specific functions (erf, sqrtx_asinh_sqrtx); Float semantics don't fit | -- Pending |
| Flat `DensityVars<T>` struct | Matches C++ layout 1:1; functionals mix fields across categories; zero-cost access | -- Pending |
| Vec of active pairs for XcFunctional | Typical 2-5 components; faster than scanning 78-element array | -- Pending |
| Taylor expansion AD (not dual numbers) | Arbitrary-order derivatives (0-6) from single evaluation; matches C++ approach | -- Pending |
| GPU only for batch evaluation | Individual functionals too lightweight for kernel launch overhead | -- Pending |
| AoS layout for CPU, SoA for GPU | CPU benefits from spatial locality per point; GPU benefits from coalesced memory access | -- Pending |
| Regularization at density < 1e-14 | Prevents division-by-zero in r_s, zeta, power-law terms | -- Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check -- still the right priority?
3. Audit Out of Scope -- reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-18 after Phase 1 completion*
