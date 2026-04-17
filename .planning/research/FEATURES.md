# Feature Landscape

**Domain:** Exchange-correlation functional library for density functional theory (DFT)
**Researched:** 2026-04-17
**Competitive context:** libxc (600+ functionals, C, Maple codegen), ExchCXX (C++ wrapper, GPU-native), original C++ xcfun (78 functionals, AD-based)

## Table Stakes

Features users expect from an xcfun-compatible library. Missing any of these means the library cannot serve as a drop-in replacement and will not be adopted by existing codes (DALTON, PySCF, ADF, LSDalton).

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| All 78 functionals (LDA/GGA/mGGA/hybrid) | 1:1 parity with C++ xcfun; any missing functional breaks workflows that depend on it | High (volume) | 5 LDA + ~38 GGA + ~20 mGGA/kinetic + 15 hybrid. Each must match C++ output within 1e-12 |
| All 39+ aliases (B3LYP, PBE0, SCAN, etc.) | Users specify functionals by common names; aliases are the primary user interface | Medium | Weighted composition of base functionals. See `aliases.cpp` for complete list |
| Arbitrary-order derivatives (0-6) | xcfun's defining feature vs libxc (max order 4). Required for response property calculations (TDDFT, hyperpolarizabilities) | High | Via Taylor-expansion AD engine (`CTaylor<T, N>`). Order 6 means 2^6 = 64 Taylor coefficients |
| Three evaluation modes (partial derivatives, potential, contracted) | All three modes are used by downstream DFT codes; potential mode for SCF, partial for response, contracted for basis-set-aware evaluation | High | Potential mode requires GGA divergence handling. Contracted mode handles pre-expanded Taylor inputs |
| 30 variable type combinations (`xcfun_vars`) | DFT codes use different variable conventions (alpha/beta vs rho/spin, gamma-type vs explicit gradients, with/without Laplacian/tau/current) | High | Must support all VarType variants including 2nd-order Taylor input modes |
| Spin-polarized evaluation | Open-shell systems are a core DFT use case; every production code requires spin-polarized XC | Medium | Alpha/beta density variables, spin interpolation in correlation functionals |
| C FFI matching `xcfun.h` | Drop-in replacement for existing C/C++/Fortran codes that link against xcfun | Medium | ~20 C API functions. Must match signatures, error codes, and opaque handle pattern exactly |
| Python bindings | PySCF integration is xcfun's second-largest use case after Fortran codes | Medium | PyO3 with NumPy array I/O for batch evaluation |
| Numerical accuracy (1e-12 relative error) | Scientific computing demands reproducibility; any deviation from reference breaks validation pipelines | High | Requires careful regularization near zero density, matching C++ constants exactly |
| Batch evaluation (`xcfun_eval_vec`) | Grid-based DFT evaluates millions of points per SCF iteration; single-point API alone is insufficient | Medium | Pitched memory layout (density_pitch, result_pitch) for flexible array striding |
| Functional introspection (enumerate, describe, is_gga, is_metagga) | Codes need to query functional properties at runtime to set up correct variable types and grid requirements | Low | String-based enumeration of parameters and aliases, short/long descriptions |
| Regularization at low density | Division by zero in r_s, zeta, power-law terms causes NaN/Inf; every XC library must handle this | Medium | Threshold at density < 1e-14 matching C++ behavior |
| Exact exchange (EXX) parameter | Hybrid functionals specify HF exchange fraction; codes read this to mix XC with Fock exchange | Low | Single float parameter, set via alias composition (e.g., B3LYP sets exx=0.20) |
| Range-separation parameters (CAM) | CAM-B3LYP and other range-separated hybrids require alpha, beta, mu parameters | Low | Three parameters: cam_alpha, cam_beta, rangesep_mu |

## Differentiators

Features that set xcfun_rs apart from both the original C++ xcfun and competing libraries. Not expected by existing users but create competitive advantage.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| GPU batch evaluation via cubecl | Neither C++ xcfun nor libxc have production-quality cross-platform GPU support. ExchCXX exists but only covers ~13 functionals. xcfun_rs would cover all 78 on GPU | Very High | cubecl targets CUDA/Metal/Vulkan. AoS-to-SoA transposition, buffer caching, automatic CPU fallback. Only worthwhile for batch sizes > ~10k points |
| Memory safety (Rust) | C++ xcfun has raw pointer handling throughout; Rust eliminates entire classes of memory bugs in a library linked into large Fortran/C++ codes | Low (inherent) | FFI boundary requires `unsafe` but internal code is safe Rust |
| Const-generic derivative order | Compile-time specialization for each derivative order eliminates runtime branching in the hot path. C++ xcfun uses runtime polymorphism | Medium | `CTaylor<T, N>` where N is const. Monomorphized per order for zero-cost abstraction |
| Modular crate architecture | AD engine (`xcfun-ad`) is independently reusable for other automatic differentiation needs. GPU layer is feature-gated to avoid compile cost | Low | 7-crate workspace. Users who only need CPU pay zero cost for GPU dependencies |
| Thread-safe evaluation | Rust's ownership model makes concurrent batch evaluation safe by construction. C++ xcfun requires external synchronization | Low (inherent) | `XcFunctional` can be shared across threads with `&self` evaluation |
| Cross-platform GPU (CUDA + Metal + Vulkan) | ExchCXX supports only CUDA/HIP/SYCL. cubecl adds Metal (macOS) and Vulkan (portable) | High | cubecl 0.10 is pre-release; maturity is a risk. See PITFALLS.md |
| Better error handling | C++ xcfun returns integer error codes with no context. Rust version uses typed errors with descriptive messages | Low | `XcError` enum with thiserror. Propagates through FFI as error codes for compatibility |
| Criterion benchmarks with C++ comparison | No existing XC library ships structured, reproducible benchmarks against competitors | Medium | criterion.rs for statistical benchmarking. Validates "within 1.2x of C++" claim |

## Anti-Features

Features to explicitly NOT build. Each is a deliberate exclusion with rationale.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| New functionals beyond C++ xcfun's 78 | Scope explosion. This is a reimplementation, not a research project. Adding functionals is a rabbit hole (libxc has 600+) | Match C++ xcfun exactly. Users who need broader coverage use libxc |
| Machine-learned XC functionals (ML-XC) | Entirely different architecture (neural networks vs analytic functions). libnxc exists for this | Keep functional interface generic enough that an ML wrapper could be added later, but do not build it |
| Derivative orders > 6 | C++ xcfun caps at 6. Taylor coefficient count grows as 2^N; order 7 = 128 coefficients per variable pair. Diminishing returns and numerical instability | Match C++ xcfun maximum. Document the limit |
| Web API / REST interface | This is a computational kernel, not a service. XC evaluation happens in tight inner loops | Provide library API (Rust, C, Python) only |
| GUI or visualization | XC libraries are backend components; visualization belongs in the calling application | Export data in standard formats that visualization tools can consume |
| Fortran bindings (direct) | C FFI is sufficient; Fortran codes call C interfaces via `iso_c_binding`. Separate Fortran module adds maintenance burden | Provide C API that Fortran can call directly. The original xcfun also ships a thin `xcfun.f90` wrapper -- consider shipping an equivalent but not a native Fortran binding |
| Symbolic differentiation / Maple codegen | libxc's approach (Maple-generated C code). Conflicts with xcfun's AD philosophy. Would require maintaining two codegen pipelines | Use Taylor-expansion AD exclusively. It is xcfun's architectural identity |
| General-purpose AD library | xcfun-ad is domain-specialized (Taylor expansion, fixed order, specific transcendentals like `sqrtx_asinh_sqrtx`). Competing with general AD crates (enzyme, etc.) is out of scope | Keep xcfun-ad focused on XC evaluation needs. If others find it useful, great, but do not design for general use |
| Integration with specific DFT codes | Tight coupling with DALTON, PySCF, etc. would create maintenance burden and version lock | Provide stable C API and Python API. Let DFT codes integrate at their discretion |
| SIMD-explicit vectorization (initial) | Premature optimization. Get correct scalar code first; SIMD is a Phase 8 optimization target | Design data layouts (AoS for CPU) to be SIMD-friendly. Apply SIMD in optimization phase only |

## Feature Dependencies

```
Core Types (DensityVars, enums, traits) ──> AD Engine (CTaylor, Num trait)
     │                                           │
     v                                           v
LDA Functionals (5) ──────────────────> Evaluation Pipeline (XcFunctional, modes)
     │                                           │
     v                                           v
GGA Functionals (~38) ──────────────> GGA Potential Mode (divergence handling)
     │                                           │
     v                                           │
meta-GGA Functionals (~20) ─────────────────────┤
     │                                           │
     v                                           │
Hybrid Functionals (15) + All Aliases ──────────┤
     │                                           │
     v                                           v
C FFI ──────────────────────────────────> Python Bindings
                                                 │
GPU Batch Evaluation <── requires LDA + GGA ─────┤
                                                 │
                                                 v
                                    Benchmarks + Optimization
```

Key dependency chains:
- `AD Engine` -> `All Functionals`: Every functional is generic over `T: Num`, evaluated via `CTaylor`
- `LDA Functionals` -> `Evaluation Pipeline`: Pipeline must be built and validated with simplest functionals first
- `Evaluation Pipeline` -> `All Higher Functionals`: GGA/mGGA/hybrid reuse the same pipeline
- `GGA Functionals` -> `GPU Evaluation`: GPU testing needs at least GGA-level functionals to be meaningful
- `All Functionals + Aliases` -> `FFI + Python`: Bindings should expose complete functionality
- `Everything` -> `Benchmarks`: Optimization is the final phase

## MVP Recommendation

The minimum viable product for xcfun_rs that would be useful to downstream codes:

**Prioritize (in order):**
1. Core types + AD engine -- foundation for everything
2. 5 LDA functionals + full evaluation pipeline (all 3 modes) -- validates the entire architecture
3. ~38 GGA functionals + GGA potential mode -- covers the most commonly used functionals (PBE, BLYP, B88)
4. ~20 meta-GGA + kinetic functionals -- SCAN family is increasingly popular in materials science
5. 15 hybrid functionals + all 39+ aliases -- completes functional coverage (B3LYP alone justifies this)
6. C FFI -- enables integration with existing C/C++/Fortran codes

**Defer:**
- GPU batch evaluation: Useful but not blocking adoption. CPU batch evaluation is sufficient for correctness validation. GPU is a performance differentiator, not a functional requirement
- Python bindings: Important for PySCF users but secondary to C FFI for the broader DFT ecosystem
- Benchmarks and optimization: Correctness first, performance second. "Within 1.2x of C++" is a target, not a gate for initial release

**Minimum useful subset:** Phases 1-5 (core + all 78 functionals + aliases) + Phase 7 FFI = drop-in replacement for C++ xcfun on CPU. This alone justifies the project.

## Sources

- [C++ xcfun repository (dftlibs/xcfun)](https://github.com/dftlibs/xcfun) -- reference implementation, `api/xcfun.h`, aliases.cpp
- [libxc official site](https://libxc.gitlab.io/) -- 600+ functionals, up to 4th order derivatives, Maple codegen
- [libxc 7.0.0 release](https://gitlab.com/libxc/libxc/-/releases/7.0.0) -- 23 new functionals in latest release
- [ExchCXX (wavefunction91)](https://github.com/wavefunction91/ExchCXX) -- modern C++ XC library with GPU (CUDA/HIP/SYCL), ~13 functionals
- [GauXC](https://github.com/wavefunction91/GauXC) -- integrates ExchCXX for distributed GPU XC evaluation
- [XCFun documentation](https://xcfun.readthedocs.io/en/latest/) -- arbitrary-order derivatives via AD
- [Arbitrary-Order DFT Response Theory from AD](https://www.researchgate.net/publication/222102126_Arbitrary-Order_Density_Functional_Response_Theory_from_Automatic_Differentiation) -- theoretical basis for xcfun's AD approach
- [PySCF DFT documentation](https://pyscf.org/user/dft.html) -- xcfun/libxc integration patterns in practice
- [ADF xcfun TDDFT example](https://www.scm.com/doc/ADF/Examples/XCFUN_TDDFT_H2O.html) -- xcfun used for full (non-ALDA) TDDFT kernel
- [libnxc](https://github.com/semodi/libnxc) -- ML-learned XC functionals (anti-feature reference)
