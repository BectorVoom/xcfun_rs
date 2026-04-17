# Roadmap: xcfun_rs

## Overview

This roadmap delivers a complete Rust reimplementation of the C++ xcfun exchange-correlation functional library. The journey builds upward from core types and automatic differentiation, through progressively complex functional families (LDA, GGA, meta-GGA, hybrid), then adds GPU acceleration, foreign language bindings, and performance optimization. Each phase produces a verifiable capability: the AD engine can differentiate, LDA functionals match C++ output, GGA functionals extend coverage, and so on until all 78 functionals are accessible from C, Python, and GPU with benchmark-proven performance.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Core Types + AD Engine** - Type system and automatic differentiation foundation
- [ ] **Phase 2: LDA Functionals + Validation Pipeline** - First functionals, evaluation engine, and test infrastructure
- [ ] **Phase 3: GGA Functionals** - ~38 gradient-corrected functionals and GGA potential mode
- [ ] **Phase 4: Meta-GGA Functionals** - ~20 meta-GGA and kinetic energy functionals
- [ ] **Phase 5: Hybrid Functionals + Aliases** - M05/M06 family, range separation, full alias table
- [ ] **Phase 6: GPU Evaluation** - Batch GPU evaluation via cubecl with CPU fallback
- [ ] **Phase 7: FFI + Python Bindings** - C API drop-in replacement and PyO3 Python module
- [ ] **Phase 8: Benchmarks + Optimization** - Performance validation and targeted optimization

## Phase Details

### Phase 1: Core Types + AD Engine
**Goal**: Developers can define density variables, enumerate functionals, and compute arbitrary-order derivatives of composed mathematical functions
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, AD-01, AD-02, AD-03, AD-04, AD-05, AD-06, AD-07, AD-08, AD-09
**Success Criteria** (what must be TRUE):
  1. DensityVars can be constructed from raw input arrays for all 30 VarType variants
  2. CTaylor produces correct derivatives for exp, log, pow, sqrt, sin, cos, erf at orders 0-6
  3. Composed functions (chain rule) yield correct mixed partial derivatives verified against known analytical values
  4. AD engine handles edge cases (near-zero density, extreme coefficients) without NaN or panic
  5. FunctionalId enum covers all 78 variants with name lookup and dependency metadata
**Plans:** 3 plans

Plans:
- [ ] 01-01-PLAN.md -- Workspace setup + xcfun-ad core (CTaylor, arithmetic, Num trait, compose)
- [ ] 01-02-PLAN.md -- xcfun-core types (enums, DensityVars, error, constants, Functional trait, stub crates)
- [ ] 01-03-PLAN.md -- Transcendental functions, special functions, and numerical stability

### Phase 2: LDA Functionals + Validation Pipeline
**Goal**: Users can evaluate LDA exchange-correlation energies and potentials with verified accuracy against C++ reference data
**Depends on**: Phase 1
**Requirements**: LDA-01, LDA-02, LDA-03, LDA-04, LDA-05, LDA-06, EVAL-01, EVAL-02, EVAL-03, EVAL-04, EVAL-05, EVAL-06, EVAL-07, EVAL-08, VAL-01, VAL-02, VAL-03, VAL-04
**Success Criteria** (what must be TRUE):
  1. All 5 LDA functionals produce energy values matching C++ xcfun within 1e-12 relative error at orders 0-4
  2. XcFunctional lifecycle (new, set, eval_setup, eval) works for all three evaluation modes
  3. LDA aliases (svwn, svwn5, etc.) expand to correct weighted functional compositions
  4. Automated test suite compares every implemented functional against C++ reference data and reports max/mean error
  5. Batch evaluation processes multiple density points in a single call
**Plans**: TBD

Plans:
- [ ] 02-01: TBD
- [ ] 02-02: TBD
- [ ] 02-03: TBD

### Phase 3: GGA Functionals
**Goal**: Users can evaluate gradient-corrected exchange-correlation functionals covering the PBE, Becke, LYP, and B97 families
**Depends on**: Phase 2
**Requirements**: GGA-01, GGA-02, GGA-03, GGA-04, GGA-05
**Success Criteria** (what must be TRUE):
  1. All ~38 GGA functionals produce energy values matching C++ xcfun within 1e-12 relative error
  2. GGA potential mode correctly computes v_xc including gradient divergence terms
  3. GGA aliases (blyp, pbe, bp86, etc.) expand to correct functional compositions
  4. Shared helper modules (pw91_like, specmath) are reused across GGA implementations without duplication
**Plans**: TBD

Plans:
- [ ] 03-01: TBD
- [ ] 03-02: TBD
- [ ] 03-03: TBD

### Phase 4: Meta-GGA Functionals
**Goal**: Users can evaluate meta-GGA and kinetic energy functionals that depend on the kinetic energy density
**Depends on**: Phase 3
**Requirements**: MGGA-01, MGGA-02, MGGA-03, MGGA-04, MGGA-05, MGGA-06
**Success Criteria** (what must be TRUE):
  1. All TPSS and SCAN family functionals produce energy values matching C++ xcfun within 1e-12
  2. All kinetic energy functionals (TfK, Tw, VwK, Pw91K, BtK) match C++ output within 1e-12
  3. Potential mode correctly rejects meta-GGA functionals with an informative error
  4. Meta-GGA aliases (scan, r2scan, etc.) expand to correct compositions
**Plans**: TBD

Plans:
- [ ] 04-01: TBD
- [ ] 04-02: TBD
- [ ] 04-03: TBD

### Phase 5: Hybrid Functionals + Aliases
**Goal**: Full functional coverage -- all 78 functionals and 39+ aliases are implemented and validated
**Depends on**: Phase 4
**Requirements**: HYB-01, HYB-02, HYB-03, HYB-04, HYB-05, HYB-06
**Success Criteria** (what must be TRUE):
  1. All 12 M05/M06 family functionals match C++ xcfun within 1e-12
  2. Range-separated CAM-B3LYP evaluates correctly with proper CAM parameters
  3. All 39+ aliases produce correct compositions (verified against C++ alias definitions)
  4. HF alias sets exx=1.0 with no DFT functional components
  5. Property-based tests pass: spin symmetry for unpolarized inputs, zero density limit behavior
**Plans**: TBD

Plans:
- [ ] 05-01: TBD
- [ ] 05-02: TBD
- [ ] 05-03: TBD

### Phase 6: GPU Evaluation
**Goal**: Users can offload batch functional evaluation to GPU for large-scale DFT grid computations
**Depends on**: Phase 2, Phase 3
**Requirements**: GPU-01, GPU-02, GPU-03, GPU-04, GPU-05, GPU-06
**Success Criteria** (what must be TRUE):
  1. GpuEvaluator can evaluate all functionals on GPU at orders 0 and 1
  2. GPU and CPU paths produce identical results for all functionals (difference < 1e-12)
  3. Automatic fallback to CPU occurs seamlessly when GPU is unavailable or batch size is small
  4. Buffer cache reuses GPU memory across repeated evaluation calls without leaking
**Plans**: TBD

Plans:
- [ ] 06-01: TBD
- [ ] 06-02: TBD
- [ ] 06-03: TBD

### Phase 7: FFI + Python Bindings
**Goal**: External programs can use xcfun_rs as a drop-in replacement for C++ xcfun via C or Python
**Depends on**: Phase 5
**Requirements**: FFI-01, FFI-02, FFI-03, FFI-04, PY-01, PY-02, PY-03, PY-04
**Success Criteria** (what must be TRUE):
  1. C programs using xcfun.h can link against xcfun_rs without source changes
  2. All ~20 C API functions behave identically to C++ xcfun (verified by running C++ test suite)
  3. Python XcFun class can set functionals, evaluate, and return NumPy arrays
  4. No undefined behavior across FFI boundary (panics caught, null pointers handled)
**Plans**: TBD

Plans:
- [ ] 07-01: TBD
- [ ] 07-02: TBD
- [ ] 07-03: TBD

### Phase 8: Benchmarks + Optimization
**Goal**: Performance is validated against C++ xcfun and optimized to within acceptable bounds
**Depends on**: Phase 6, Phase 7
**Requirements**: PERF-01, PERF-02, PERF-03, PERF-04
**Success Criteria** (what must be TRUE):
  1. Criterion benchmarks exist for all functional categories (LDA, GGA, meta-GGA, hybrid)
  2. Rust implementation is no slower than 1.2x C++ xcfun for equivalent operations
  3. GPU path shows measurable speedup for batch sizes above 10k grid points
  4. All accuracy tests still pass after optimization (no regressions)
**Plans**: TBD

Plans:
- [ ] 08-01: TBD
- [ ] 08-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8
Note: Phase 6 (GPU) depends on Phases 2+3; Phase 8 depends on Phases 6+7.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Types + AD Engine | 0/3 | Planned | - |
| 2. LDA Functionals + Validation Pipeline | 0/3 | Not started | - |
| 3. GGA Functionals | 0/3 | Not started | - |
| 4. Meta-GGA Functionals | 0/3 | Not started | - |
| 5. Hybrid Functionals + Aliases | 0/3 | Not started | - |
| 6. GPU Evaluation | 0/3 | Not started | - |
| 7. FFI + Python Bindings | 0/3 | Not started | - |
| 8. Benchmarks + Optimization | 0/2 | Not started | - |
