# Requirements: xcfun_rs

**Defined:** 2026-04-17
**Core Value:** Numerical accuracy -- every functional must produce results matching C++ xcfun within 1e-12 relative error

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Core Types

- [ ] **CORE-01**: DensityVars<T> struct with all 25 density-derived fields and from_input() for all 30 VarType variants
- [ ] **CORE-02**: EvalMode enum (PartialDerivatives, Potential, Contracted) with mode validation
- [ ] **CORE-03**: VarType enum (30 variants) with input_len(), provides(), is_spin_polarized() metadata
- [ ] **CORE-04**: FunctionalId enum (78 variants) with from_name(), name(), description(), depends()
- [ ] **CORE-05**: Dependency bitflags (DENSITY, GRADIENT, LAPLACIAN, KINETIC, JP)
- [ ] **CORE-06**: XcError enum with thiserror derive and FFI error code mapping
- [ ] **CORE-07**: Physical constants module (C_SLATER, CF, TINY_DENSITY, MAX_ORDER)
- [ ] **CORE-08**: Functional trait definition (energy<T: Num>, depends, id, description, test_data)

### Automatic Differentiation

- [ ] **AD-01**: CTaylor<T, N> struct with const generic N and bit-flag indexing
- [ ] **AD-02**: All arithmetic operators (+, -, *, /) for CTaylor with recursive multiplication
- [ ] **AD-03**: Transcendental functions (exp, log, pow, sqrt, cbrt, abs) with correct derivatives at all orders
- [ ] **AD-04**: Trigonometric functions (sin, cos, atan, asin, acos) with correct derivatives
- [ ] **AD-05**: Special functions (asinh, erf, sqrtx_asinh_sqrtx) with correct derivatives
- [ ] **AD-06**: Num trait with implementations for f64 and CTaylor<f64, N>
- [ ] **AD-07**: Taylor composition (chain rule) implementation
- [ ] **AD-08**: taylorlen() function for output size calculation
- [ ] **AD-09**: Numerical stability near zero, infinity, and with extreme coefficients

### LDA Functionals

- [ ] **LDA-01**: SlaterX (Slater exchange) matching C++ output within 1e-12
- [ ] **LDA-02**: Vwn3C (VWN3 correlation) matching C++ output within 1e-12
- [ ] **LDA-03**: Vwn5C (VWN5 correlation) matching C++ output within 1e-12
- [ ] **LDA-04**: Pz81C (Perdew-Zunger correlation) matching C++ output within 1e-12
- [ ] **LDA-05**: Pw92C (Perdew-Wang 1992 correlation) matching C++ output within 1e-12
- [ ] **LDA-06**: LDA aliases (lda, svwn, svwn5, svwn3, vwn, vwn5, vwn3) produce correct compositions

### Evaluation Pipeline

- [ ] **EVAL-01**: XcFunctional object with new(), set(), eval_setup(), eval() lifecycle
- [ ] **EVAL-02**: Partial derivatives mode (orders 0-6) producing correct taylorlen output
- [ ] **EVAL-03**: Potential mode for LDA and GGA functionals (v_xc output)
- [ ] **EVAL-04**: Contracted mode handling pre-expanded Taylor inputs
- [ ] **EVAL-05**: Batch evaluation (evaluate_batch) for multi-point workloads
- [ ] **EVAL-06**: FunctionalImpl enum dispatch for all 78 functionals
- [ ] **EVAL-07**: Functional composition (alias expansion with weighted sums)
- [ ] **EVAL-08**: Regularization at density < 1e-14 preserving derivative coefficients

### GGA Functionals

- [ ] **GGA-01**: ~22 GGA exchange functionals (PbeX, BeckeX, BeckeCorrX, BeckeSrX, BeckeCamX, Pw86X, Pw91X, RevPbeX, RPbeX, OptX, OptXCorr, PbeSolX, PbeIntX, BlocX, KtX, B97X, B97_1X, B97_2X, BrX, LdaErfX, LdaErfC, LdaErfC_JT) all within 1e-12
- [ ] **GGA-02**: ~18 GGA correlation functionals (PbeC, LypC, P86C, P86CorrC, SPbeC, Vwn_PbeC, BrC, BrXC, Pw91C, B97C, B97_1C, B97_2C, CsC, APbeC, ZvPbeSolC, PbeIntC, PbeLocC, ZvPbeIntC) all within 1e-12
- [ ] **GGA-03**: Helper function modules (pw91_like, specmath) shared across GGA functionals
- [ ] **GGA-04**: GGA potential mode with gradient divergence handling
- [ ] **GGA-05**: GGA aliases (blyp, pbe, bp86, bpw91, olyp, lyp, kt1, kt2, kt3, ldaerf, becke, slater, B88X, LDAX, PBEX, KT3X, OPTX)

### Meta-GGA and Kinetic Functionals

- [ ] **MGGA-01**: TPSS family (TpssX, TpssC, RevTpssX, RevTpssC, TpssLocC) within 1e-12
- [ ] **MGGA-02**: SCAN family (ScanX, ScanC, RScanX, RScanC, RppScanX, RppScanC, R2ScanX, R2ScanC, R4ScanX, R4ScanC) within 1e-12
- [ ] **MGGA-03**: Kinetic energy functionals (TfK, Tw, VwK, Pw91K, BtK) within 1e-12
- [ ] **MGGA-04**: Meta-GGA helper functions (SCAN enhancement factors, alpha parameter)
- [ ] **MGGA-05**: Meta-GGA aliases (scan, rscan, rppscan, r2scan, r4scan, tfk, tw)
- [ ] **MGGA-06**: Potential mode correctly rejects meta-GGA functionals

### Hybrid Functionals and Aliases

- [ ] **HYB-01**: M05/M06 family (M05X, M05X2X, M06X, M06X2X, M06LX, M06HfX, M05C, M05X2C, M06C, M06HfC, M06LC, M06X2C) within 1e-12
- [ ] **HYB-02**: M06 helper functions (zet, gamma, h, fw, lsda_x)
- [ ] **HYB-03**: Range-separated functional support (BeckeCamX with CAM parameters)
- [ ] **HYB-04**: All 39+ aliases produce correct compositions (including b3lyp, pbe0, camb3lyp, m06, etc.)
- [ ] **HYB-05**: EXX parameter handling (HF alias sets exx=1.0 with no DFT functionals)
- [ ] **HYB-06**: Property-based tests pass (spin symmetry, zero density limit)

### GPU Evaluation

- [ ] **GPU-01**: GpuEvaluator struct with cubecl runtime management
- [ ] **GPU-02**: GPU kernels for energy evaluation (order 0) and first derivatives (order 1)
- [ ] **GPU-03**: AoS to SoA transposition for GPU-friendly memory layout
- [ ] **GPU-04**: GPU-resident buffer caching across repeated calls
- [ ] **GPU-05**: Automatic CPU/GPU fallback based on batch size and hardware
- [ ] **GPU-06**: GPU/CPU consistency for all functionals (< 1e-12 difference)

### C FFI

- [ ] **FFI-01**: Complete C API matching xcfun.h (~20 functions: new, delete, set, get, eval_setup, eval, eval_vec, etc.)
- [ ] **FFI-02**: Error code mapping (XC_EORDER, XC_EVARS, XC_EMODE, XC_EINTERNAL)
- [ ] **FFI-03**: Header file generation via cbindgen
- [ ] **FFI-04**: Memory safety (no UB across FFI boundary, proper panic catching)

### Python Bindings

- [ ] **PY-01**: XcFun Python class with set(), eval_setup(), eval() methods via PyO3
- [ ] **PY-02**: NumPy array input/output for batch evaluation
- [ ] **PY-03**: Enumeration of functionals and aliases from Python
- [ ] **PY-04**: __repr__ and __str__ for debugging

### Performance

- [ ] **PERF-01**: Criterion benchmarks for all functional categories
- [ ] **PERF-02**: Performance within 1.2x of C++ xcfun for equivalent operations
- [ ] **PERF-03**: GPU path shows measurable speedup for batch sizes > 10k points
- [ ] **PERF-04**: No accuracy regressions after optimization (all tests still pass)

### Validation Infrastructure

- [ ] **VAL-01**: Reference test data extracted from C++ xcfun sources
- [ ] **VAL-02**: test_all_functionals_against_reference() automated test
- [ ] **VAL-03**: Accuracy reporting (max/mean relative error per functional)
- [ ] **VAL-04**: Cross-validation against C++ xcfun output for all 78 functionals at orders 0-4

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Extended Testing

- **V2-TEST-01**: Property-based testing with proptest for AD engine edge cases
- **V2-TEST-02**: Fuzzing inputs to functional evaluation for robustness
- **V2-TEST-03**: Miri testing for FFI memory safety validation

### Extended Bindings

- **V2-BIND-01**: Thin Fortran wrapper (xcfun.f90 equivalent)
- **V2-BIND-02**: Wasm compilation target for browser-based DFT tools

### Extended GPU

- **V2-GPU-01**: GPU evaluation at derivative orders 2-4
- **V2-GPU-02**: Multi-device GPU evaluation
- **V2-GPU-03**: Metal and Vulkan backend validation

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| New functionals beyond C++ xcfun's 78 | Reimplementation, not extension. Users needing more use libxc |
| Machine-learned XC functionals (ML-XC) | Different architecture (neural networks). libnxc exists |
| Derivative orders > 6 | Matches C++ xcfun maximum. 2^7 = 128 coefficients per pair |
| Web API / REST interface | Computational kernel, not a service |
| GUI or visualization | Backend component; visualization belongs in calling app |
| Direct Fortran bindings | C FFI via iso_c_binding is sufficient |
| Symbolic differentiation / Maple codegen | Conflicts with xcfun's AD philosophy |
| General-purpose AD library | xcfun-ad is domain-specialized |
| Integration with specific DFT codes | Provide stable APIs; let codes integrate |
| SIMD-explicit vectorization (initial) | Deferred to optimization phase |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 through CORE-08 | Phase 1 | Pending |
| AD-01 through AD-09 | Phase 1 | Pending |
| LDA-01 through LDA-06 | Phase 2 | Pending |
| EVAL-01 through EVAL-08 | Phase 2 | Pending |
| VAL-01 through VAL-04 | Phase 2 | Pending |
| GGA-01 through GGA-05 | Phase 3 | Pending |
| MGGA-01 through MGGA-06 | Phase 4 | Pending |
| HYB-01 through HYB-06 | Phase 5 | Pending |
| GPU-01 through GPU-06 | Phase 6 | Pending |
| FFI-01 through FFI-04 | Phase 7 | Pending |
| PY-01 through PY-04 | Phase 7 | Pending |
| PERF-01 through PERF-04 | Phase 8 | Pending |

**Coverage:**
- v1 requirements: 70 total
- Mapped to phases: 70
- Unmapped: 0

---
*Requirements defined: 2026-04-17*
*Last updated: 2026-04-17 after roadmap creation*
