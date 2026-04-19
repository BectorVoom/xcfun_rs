# Roadmap: xcfun_rs

**Created:** 2026-04-19
**Granularity:** standard (5-8 phases)
**Parallelization:** enabled
**Core Value:** Every functional must produce numerical output matching C++ xcfun within relative error <= 1.0e-12, across all evaluation modes and derivative orders.
**Requirements source:** `.planning/REQUIREMENTS.md` (103 v1 requirements across 14 categories)
**Research source:** `.planning/research/SUMMARY.md` section "Implications for Roadmap" (10-phase DAG collapsed into 7 standard-granularity phases)

## Phase Derivation Notes

The dependency DAG (per `ARCHITECTURE.md` section 7 and `SUMMARY.md` "Phase Ordering Rationale") admits exactly one topological order: `xcfun-ad` -> `xcfun-core` foundations -> LDA -> GGA -> metaGGA/aliases -> facade + capi -> kernels + CPU batch -> GPU backends -> python + release. Seven phases compress this naturally:

- **Phase 0 (scaffolding)** is a real phase: CI gates prevent pitfalls P1 (reassociation), P8 (cubecl drift), P12 (PW92C constants), P13 (registry drift). Not an afterthought.
- **Phase 2 (LDA)** carries the tier-2 parity harness: the 1e-12 gate must be exercised continuously, not after "all functionals ported". This is the LDA-first-for-validation decision.
- **Phase 5 (facade + capi together)** avoids splitting bikesheds on one API decision.
- **Phase 6 (kernels + CPU batch + GPU backends combined)** is a compressed unit because `CpuRuntime` is the runtime validatable at 1e-12 vs scalar, and GPU backends add tolerance tables and f64 gates on top; standard granularity folds them.
- **Phase 7 (python + release)** is last because it is the most visible surface but the least architectural.

## Phases

- [ ] **Phase 0: Workspace Scaffolding & CI Foundations** - Workspace, crate skeletons, CI gates blocking P1/P8/P12/P13 before any functional code exists
- [ ] **Phase 1: Taylor Algebra & AD Primitives (`xcfun-ad`)** - `CTaylor<T, N>`, `Num` trait, every `*_expand` function, bit-equivalence with C++ on orders 0..=3
- [ ] **Phase 2: Core Foundations + LDA Tier + Parity Harness** - `xcfun-core` scaffolding (vars/mode/error/densvars/registry) + 11 LDA functionals + tier-2 validation harness at 1e-12 on CPU
- [ ] **Phase 3: GGA Tier + `Mode::Potential`** - 45 GGA functionals + `Mode::Potential` via `CTaylor<f64, 2>` divergence construction
- [ ] **Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases** - 15 metaGGA functionals + orders 0..=6 for `Contracted` + 46 aliases with multiplicative weight composition
- [ ] **Phase 5: Rust Facade (`xcfun-rs`) + C ABI (`xcfun-capi`)** - Thin facade re-exports + full C ABI with cbindgen-generated `xcfun.h` byte-matched to reference
- [ ] **Phase 6: Kernels (`xcfun-kernels`) + CPU Batch + CUDA + Wgpu Backends** - Single `#[cube]` source per functional; `Batch<CpuRuntime>` at 1e-13; CUDA at 1e-13; Wgpu at 1e-9 with `erf` fallback
- [ ] **Phase 7: Python Bindings (`xcfun-py`) + Release** - PyO3 0.28 + rust-numpy 0.28 wheel passing `pytest`, crates published, release ceremony

## Phase Details

### Phase 0: Workspace Scaffolding & CI Foundations
**Goal**: CI gates and workspace skeleton exist that forbid the numerical pitfalls before any functional math is written.
**Depends on**: Nothing (first phase)
**Research flag**: No (standard patterns; skip `/gsd-research-phase`)
**Requirements**: QG-01, QG-02, QG-03, QG-04, QG-05, QG-06, QG-07, QG-08, CORE-10, ACC-05, ACC-06
**Success Criteria** (what must be TRUE):
  1. `cargo build --workspace` and `cargo test --workspace` succeed against empty crate skeletons (`xcfun-ad`, `xcfun-core`, `xcfun-kernels`, `xcfun-gpu`, `xcfun-rs`, `xcfun-capi`, `xcfun-py`) plus `xtask/` and `validation/` binaries.
  2. CI fails a PR that sets `RUSTFLAGS` or that fails `cargo xtask check-no-anyhow`, `cargo xtask check-boundaries`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo fmt --check`, `cargo deny check`, or `cargo metadata`-based cubecl `=0.10.0-pre.3` assertion.
  3. CI fails a PR that edits `xcfun-master/src/functionals/` without re-running `cargo xtask regen-registry --check` (registry content-hash gate).
  4. A lint rule rejects `mul_add` inside `xcfun-core/src/functionals/*.rs` and the release profile contains `-Cllvm-args=-fp-contract=off`.
  5. `config.hpp` has been read; the `pw92c-legacy-constants` Cargo feature is defined with the documented default matching the vendored `xcfun-master/`.
**Plans**: TBD

### Phase 1: Taylor Algebra & AD Primitives (`xcfun-ad`, cubecl-native)
**Goal**: A cubecl-native AD engine: `CTaylor<F, N>` as a pure `#[cube]` type backed by cubecl `Array<F>` storage, every arithmetic operation and every `*_expand` scalar series function written as `#[cube] fn` generic over `F: Float`, validated on `cubecl-cpu` (`CpuRuntime`) against the C++ xcfun reference at **1e-12 strict relative error**. Single source of truth — no parallel hand-Rust scalar implementation.
**Depends on**: Phase 0
**Research flag**: Yes (per SUMMARY.md "Research Flags" — `ctaylor.hpp`/`tmath.hpp` recursion patterns AND cubecl 0.10-pre.3 `#[cube]` type + `Array<F>` constraints, FMA suppression on cubecl-cpu's MLIR JIT, `OnceLock<CpuClient>` test pattern, batch-per-property kernel pattern for 10k-iter proptests)
**Requirements**: AD-01, AD-02, AD-03, AD-04, AD-05, AD-06
**Locked context**: `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` (cubecl pivot, 28 decisions, 2026-04-19 PM rewrite)
**Success Criteria** (what must be TRUE):
  1. `CTaylor<F: Float, const N: u32>` compiles as a pure `#[cube]` type for `N in 0..=7` with `Array<F>` storage of length `1 << N`, verified by passing `cargo test -p xcfun-ad --features cpu` exercising every `N` via `cubecl-cpu`.
  2. Every arithmetic operation (`+`, `-`, `*`, `/`, neg) and every composed elementary function (`reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`) is implemented as `#[cube] fn` generic over `F: Float`. For `F = f64`, every op produces coefficient arrays matching the C++ test driver at relative error ≤ 1e-12 on a fixed-seed input set at orders 0..=3.
  3. Every `*_expand` from `xcfun-master/external/upstream/taylor/tmath.hpp` (`inv_expand`, `exp_expand`, `log_expand`, `pow_expand`, `sqrt_expand`, `cbrt_expand`, `gauss_expand`, `erf_expand`) has a `#[cube] fn` port writing into a length-8 `Array<F>`, with golden-coefficient parity at ≤ 1e-12 vs the C++ driver across 3 inputs × 7 orders.
  4. Property tests (ring axioms, `exp`/`log` round-trip, `sqrt`-squared invariance, Leibniz product rule, ≥ 11 properties) run ≥ 10 000 iterations per property using the **batch-per-property kernel pattern** (proptest generates 10k inputs upfront, single kernel evaluates all, results aggregated host-side) with zero failures.
  5. `cargo bench -p xcfun-ad` publishes a baseline for the `CTaylor::mul`-equivalent `#[cube]` kernel at `N in {2,3,4,5,6}` and composed `exp`/`log`/`pow` at `N = 4`, measured at batch sizes {1, 64, 1024} so kernel-launch-amortized cost is visible.
  6. CI evidence (asm spot-check or equivalent) confirms cubecl-cpu's MLIR lowering does **not** introduce FMA or operation reordering inside `CTaylor::mul` on the f64 path. If reordering is detected and unavoidable, plan-phase MUST escalate via `PLANNING INCONCLUSIVE` rather than silently widen tolerance (per CONTEXT.md D-03).
**Plans**: 7 plans across 6 waves (granularity standard; parallelization enabled — Wave 2 runs plans 03 + 05 in parallel).

- [ ] 01-01-PLAN.md — Wave 0: revert pre-pivot commits + workspace/xtask scaffold + cubecl-cpu spike + for_tests harness (AD-01 substrate)
- [ ] 01-02-PLAN.md — Wave 1: CTaylor + ctaylor_rec{mul, multo, compose} — load-bearing recursion (AD-01, AD-03)
- [ ] 01-03-PLAN.md — Wave 2 (parallel with 01-05): expand/{inv, exp, log, pow, sqrt, cbrt} — primary scalar series (AD-04)
- [ ] 01-04-PLAN.md — Wave 3: tfuns helpers + expand/{atan, gauss, erf, asinh} — transcendentals (AD-04)
- [ ] 01-05-PLAN.md — Wave 2 (parallel with 01-03): xtask fixture generator + committed fixtures + golden_mul test (AD-03, AD-05)
- [ ] 01-06-PLAN.md — Wave 4: math.rs composed ops + extended fixtures + golden_expand/composed (AD-02, AD-05)
- [ ] 01-07-PLAN.md — Wave 5: proptest batch-per-property + criterion benchmarks + phase sign-off (AD-03, AD-06)

Pre-pivot plans (VOID — reverted by Wave 0 of the new plan, retained in git history):
- ~~pre-pivot 01-01 — Wave 0 hand-Rust scaffolding (commits f07611c, c7a3f46) [SUPERSEDED]~~
- ~~pre-pivot 01-02 — Wave 1 hand-Rust `*_expand` ports (commit 2db557c, partial) [SUPERSEDED]~~
- ~~pre-pivot 01-03 — Wave 1 hand-Rust `ctaylor_rec` mul/multo/compose port [SUPERSEDED]~~
- ~~pre-pivot 01-04 — Wave 1 fixture generator [INTENT RETAINED, replanned for cubecl validation]~~
- ~~pre-pivot 01-05 — Wave 2 `Num` trait + composed fns [SUPERSEDED — `Num` retired in favour of cubecl `Float`]~~
- ~~pre-pivot 01-06 — Wave 2 proptest 11 props × 10k iters [INTENT RETAINED, now batch-per-property kernel]~~
- ~~pre-pivot 01-07 — Wave 2 criterion bench baselines [INTENT RETAINED, now kernel-launch-amortized at batch sizes {1,64,1024}]~~

### Phase 2: Core Foundations + LDA Tier + Parity Harness
**Goal**: A user can run `cargo xtask validate --backend cpu --order 2 --filter 'lda|slaterx|vwn|pw92c|pz81c|ldaerf|tfk|tw|vonw'` and see zero failures at 1e-12 relative error against the C++ reference.
**Depends on**: Phase 1
**Research flag**: No (standard port pattern)
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, CORE-09, LDA-01, LDA-02, LDA-03, LDA-04, LDA-05, LDA-06, LDA-07, LDA-08, LDA-09, LDA-10, MODE-04, ACC-01, ACC-02, ACC-03, ACC-04
**Success Criteria** (what must be TRUE):
  1. `Vars` (31 variants), `Mode` (4 variants), `Dependency` bitflags, `XcError` enum compile with exact C header discriminants and `#[non_exhaustive]` on `XcError`; `input_length` matches reference for every Vars/order combination.
  2. `DensVars::build` populates every field for every one of the 31 Vars variants without C-style fallthrough bugs (helper-function-chain enforced; per-variant field-by-field parity test against C++ passes at bit equality on arithmetic-only fields and 1 ULP on `pow`/`regularize`-touched fields).
  3. `DensVars::regularize` modifies only `c[CNST]`; higher-order coefficients preserved (unit test passes).
  4. Tier-1 self-tests (`FUNCTIONAL_DESCRIPTORS[id].test_in/test_out`) run for all 11 LDA functionals in under 5 seconds and return zero failures on `cargo test`.
  5. Tier-2 parity harness `cargo xtask validate --backend cpu --order 2 --filter 'lda'` produces `validation/report.html` and `validation/report.jsonl` with max relative error <= 1e-12 across all 11 LDA functionals on a 10 000-point seeded grid, and any failing element blocks merge in CI.
**Plans**: TBD

### Phase 3: GGA Tier + `Mode::Potential`
**Goal**: All 45 GGA functionals ship in `xcfun-core` and `Mode::Potential` evaluates correctly for every `_2ND_TAYLOR`-capable Vars variant.
**Depends on**: Phase 2
**Research flag**: No (pattern established by LDA tier)
**Requirements**: GGA-01, GGA-02, GGA-03, GGA-04, GGA-05, GGA-06, GGA-07, GGA-08, GGA-09, GGA-10, MODE-01, MODE-02, MODE-05
**Success Criteria** (what must be TRUE):
  1. `cargo xtask validate --backend cpu --order 2 --filter 'gga'` reports zero failures at 1e-12 relative error across every GGA functional (PBE family, Becke family, Becke-Roussel, LYP, OPTX, PW86/PW91, P86, APBE, B97, KT/BTK/LB94/CSC).
  2. `Functional::is_gga()` returns `true` for each GGA functional and `eval_setup` rejects `Mode::Potential` with any Vars lacking `_2ND_TAYLOR` by returning `XcError::InvalidVars` (matching C++ `XCFunctional.cpp:438-447`).
  3. `output_length` returns `taylor_len(input_len, order)` for `Mode::PartialDerivatives` and 2 or 3 for `Mode::Potential`, matching the C++ reference on every configuration.
  4. `Mode::PartialDerivatives` produces output layout matching `XCFunctional.cpp:501-612` on orders 0..=4 for representative GGAs (verified in the parity harness).
  5. Range-separated GGA functionals (`beckecamx`, `beckesrx`) pass the 1e-12 gate on CPU; `erf`-drift warning signs from Pitfall P2 are resolved (no element within the tier-2 budget drifts above 5e-13 due to libm variance).
**Plans**: TBD

### Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases
**Goal**: All 78 functionals are ported and all 46 aliases resolve, with `Mode::Contracted` orders 0..=6 exercised; `cargo xtask validate --backend cpu --order 3` reports zero failures across the entire 78-functional set.
**Depends on**: Phase 3
**Research flag**: Yes (per SUMMARY.md - `Mode::Contracted` at orders 5-6, alias multiplicative semantics in `XCFunctional.cpp:370-405`, 46 aliases including negative-weight `camcompx`)
**Requirements**: MGGA-01, MGGA-02, MGGA-03, MGGA-04, MGGA-05, MODE-03, ALIAS-01, ALIAS-02, ALIAS-03, ALIAS-04, ALIAS-05, ALIAS-06
**Success Criteria** (what must be TRUE):
  1. All 15 metaGGA functionals (TPSS, SCAN family including rSCAN/rppSCAN/r2SCAN/r4SCAN, M05 family, M06 family, `blocx`) pass tier-1 self-tests and tier-2 parity at 1e-12 on orders 0..=3.
  2. `Mode::Contracted` at orders 0..=6 produces `1 << order` outputs matching the C++ `DOEVAL` macro expansion on every legal `(functional, vars, order)` tuple.
  3. For each of the 46 aliases, `Functional::new().set(alias, v).unwrap()` produces the exact same weight set as manual composition for `v in {1.0, 0.37}`, including the negative-weight `camcompx` canary (`beckecamx` weight is `-0.37` after `set("camcompx", 0.37)`).
  4. Setting `xcfun_set(fun, "b3lyp", 1.0)` followed by `xcfun_set(fun, "slaterx", 0.5)` yields an additive `slaterx` weight of (b3lyp's slaterx contribution + 0.5), matching C++ `XCFunctional.cpp:389-402`.
  5. Parameters `XC_EXX` (default 0.0), `XC_RANGESEP_MU` (default 0.4), `XC_CAM_ALPHA` (default 0.19), `XC_CAM_BETA` (default 0.46) are settable via `Functional::set` and readable via `Functional::get`.
**Plans**: TBD

### Phase 5: Rust Facade (`xcfun-rs`) + C ABI (`xcfun-capi`)
**Goal**: A C caller linking against `libxcfun_capi.so` with the cbindgen-generated `xcfun.h` produces byte-identical output to a Rust caller using `xcfun-rs::Functional` on a fixed fixture.
**Depends on**: Phase 4
**Research flag**: No (standard facade/FFI patterns)
**Requirements**: RS-01, RS-02, RS-03, RS-04, RS-05, RS-06, RS-07, RS-09, RS-10, CAPI-01, CAPI-02, CAPI-03, CAPI-04, CAPI-05, CAPI-06, CAPI-07
**Success Criteria** (what must be TRUE):
  1. The native Rust `Functional` API (`new`, `set`, `get`, `is_gga`, `is_metagga`, `eval_setup`, `user_eval_setup`, `eval`, plus free functions `version`, `splash`, `authors`, `self_test`, `is_compatible_library`, `which_vars`, `which_mode`, `enumerate_parameters`, `enumerate_aliases`, `describe_short`, `describe_long`) is reachable from `xcfun-rs` and behaves identically to the C++ reference on a fixed fixture; `Functional` is `Send + Sync`; `eval` performs zero heap allocation on the success path (verified by a `dhat` or `#[global_allocator]` fixture).
  2. Every symbol declared in `xcfun-master/api/xcfun.h` has a matching `#[no_mangle] extern "C"` export in `xcfun-capi`, wrapped in the `c_entry!` macro that calls `std::panic::catch_unwind` and aborts on panic; `xcfun_new` returns `Box<xcfun_t>` and `xcfun_delete` is NULL-safe.
  3. `cargo build -p xcfun-capi` emits both `libxcfun_capi.so` (cdylib) and `libxcfun_capi.a` (staticlib); the `headers-match` CI test asserts cbindgen output equals `xcfun-master/api/xcfun.h` modulo whitespace and comments.
  4. `cargo test -p xcfun-capi --test c_abi` compiles `tests/c_abi.c` against the staticlib + generated `xcfun.h` and produces matching output to the Rust reference driver on 10 random fixtures.
  5. `XcError::as_c_code` returns `1` (EORDER), `2` (EVARS), `4` (EMODE), `6` (EVARS|EMODE), `-1` (UnknownName/other), and `0` on success, verified by unit test.
**Plans**: TBD

### Phase 6: GPU Backends + Batch Lifecycle (`xcfun-kernels` / `xcfun-gpu`)
**Goal**: CUDA and Wgpu cubecl runtimes enabled; `Functional::eval_vec` auto-dispatches between `CpuRuntime`, `CudaRuntime`, and `WgpuRuntime` per `auto_backend()`; tier-3 parity at 1e-13 (CUDA vs CPU) and 1e-9 (Wgpu vs CPU with `erf` auto-fallback). Per-functional `#[cube]` kernel bodies already exist (landed in Phases 2–4 atop `xcfun-ad`'s cubecl-native `CTaylor`); Phase 6 adds the GPU runtimes, buffer pools, dispatch heuristic, and batch lifecycle on top.
**Depends on**: Phase 5
**Research flag**: Yes (per SUMMARY.md — `cubecl 0.10-pre.3` runtime-feature API for `auto_backend`, buffer-pool growth strategy, `Wgpu::Features::SHADER_F64` runtime probe, `erf` fallback matrix; per-functional `#[cube]` body design is no longer a Phase 6 concern — it's resolved in Phases 2–4)
**Requirements**: RS-08, KER-01, KER-02, KER-03, KER-04, KER-05, KER-06, GPU-01, GPU-02, GPU-03, GPU-04, GPU-05, GPU-06, GPU-07, GPU-08
**Note (post-cubecl-pivot)**: Pre-pivot, this phase was scoped as "port 78 functional bodies to `#[cube]` AND wire GPU runtimes". After the cubecl pivot (see Phase 1 CONTEXT.md D-23), per-functional `#[cube]` bodies move forward into Phases 2–4 (where each functional tier ships cubecl-native from day one). Phase 6's residual scope is the GPU-runtime + batch-lifecycle layer. The pre-pivot `CTaylorDev<F, N>` device type is eliminated — `xcfun-ad`'s `CTaylor<F, N>` already runs on any cubecl runtime.
**Success Criteria** (what must be TRUE):
  1. Every one of the 78 functionals has a `#[cube]` body generic over `F: Float` (already landed in Phases 2–4 atop `xcfun-ad`); Phase 6 verifies that the same source compiles unchanged for `CudaRuntime` and `WgpuRuntime`.
  2. Tier-3 parity: `Functional::eval_vec` on a 10 000-point grid via `Batch<CpuRuntime>` matches the scalar `Functional::eval` loop within 1e-13 relative error; CI asserts this on every PR.
  3. `Functional::eval_vec` dispatches to `Batch<CpuRuntime>` when `nr_points >= 64`; `Backend::Cpu` is always available; `Backend::Cuda` compiles behind `--features cuda` and `Backend::Wgpu` behind `--features wgpu`; `auto_backend()` selects CUDA when available, else Wgpu with `SHADER_F64`, else CPU.
  4. Tier-3 parity on CUDA: 10 000-point grid matches CPU within 1e-13 relative error; tier-3 parity on Wgpu: 10 000-point grid excluding functionals with `Dependency::ERF` matches CPU within 1e-9 relative error.
  5. On a Wgpu adapter without `wgpu::Features::SHADER_F64`, `Batch::open` returns `Err(XcError::Runtime)` at batch-open time (never silently downgrades to f32); compile-time `size_of::<Scalar>() == 8` assertion guards the kernel crate root; functionals with `Dependency::ERF` are auto-routed to `Backend::Cpu` when the active runtime is Wgpu.
**Plans**: TBD

### Phase 7: Python Bindings (`xcfun-py`) + Release
**Goal**: `pip install xcfun_rs` yields a wheel that passes `pytest` on Linux/macOS/Windows and reproduces C++ xcfun output on reference fixtures.
**Depends on**: Phase 6
**Research flag**: No (standard PyO3/maturin patterns)
**Requirements**: PY-01, PY-02, PY-03, PY-04, PY-05, PY-06
**Success Criteria** (what must be TRUE):
  1. `xcfun-py` builds as a PyO3 0.28 extension module with `abi3-py310`, producing a single wheel for CPython >= 3.10.
  2. The Python class `xcfun_rs.Functional` exposes `set`, `get`, `is_gga`, `is_metagga`, `eval_setup`, `user_eval_setup`, `input_length`, `output_length`, `eval`, and `eval_vec`; free module-level functions `version`, `splash`, `describe_*`, `enumerate_*`, `which_*`, `self_test`, `is_compatible_library` are reachable; raising an `XcError` on the Rust side surfaces as a Python `XcfunError` exception.
  3. `Functional.eval_vec` accepts a 2-D `numpy.ndarray[np.float64, order='C']` and returns a zero-copy 2-D `numpy.ndarray[np.float64]` verified against a copy-detection fixture (buffer pointers match when aliasing is legal).
  4. `pip install xcfun_rs` succeeds on Linux/macOS/Windows CI runners and `pytest tests/` passes all parity and smoke tests.
  5. Release artifacts are published: crates on crates.io (`xcfun-ad`, `xcfun-core`, `xcfun-kernels`, `xcfun-gpu`, `xcfun-rs`, `xcfun-capi`), wheels on PyPI via `maturin publish`, and `xcfun.h` checked into the release branch; CHANGELOG and semver tags in place.
**Plans**: TBD

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 0. Workspace Scaffolding & CI Foundations | 0/0 | Not started | - |
| 1. Taylor Algebra & AD Primitives | 0/7 | Not started (cubecl pivot) | - |
| 2. Core Foundations + LDA Tier + Parity Harness | 0/0 | Not started | - |
| 3. GGA Tier + `Mode::Potential` | 0/0 | Not started | - |
| 4. metaGGA Tier + `Mode::Contracted` + Aliases | 0/0 | Not started | - |
| 5. Rust Facade + C ABI | 0/0 | Not started | - |
| 6. Kernels + CPU Batch + CUDA + Wgpu Backends | 0/0 | Not started | - |
| 7. Python Bindings + Release | 0/0 | Not started | - |

## Coverage Notes

**Requirement categories mapped:**

| Category | Count | Phase(s) |
|----------|-------|----------|
| AD (AD-01..06) | 6 | Phase 1 |
| CORE (CORE-01..10) | 10 | Phase 0 (CORE-10 registry codegen gate), Phase 2 (CORE-01..09) |
| LDA (LDA-01..10) | 10 | Phase 2 |
| GGA (GGA-01..10) | 10 | Phase 3 |
| MGGA (MGGA-01..05) | 5 | Phase 4 |
| MODE (MODE-01..05) | 5 | Phase 2 (MODE-04), Phase 3 (MODE-01, MODE-02, MODE-05), Phase 4 (MODE-03) |
| ALIAS (ALIAS-01..06) | 6 | Phase 4 |
| RS (RS-01..10) | 10 | Phase 5 (RS-01..07, RS-09, RS-10), Phase 6 (RS-08 batch dispatch) |
| CAPI (CAPI-01..07) | 7 | Phase 5 |
| PY (PY-01..06) | 6 | Phase 7 |
| KER (KER-01..06) | 6 | Phase 6 |
| GPU (GPU-01..08) | 8 | Phase 6 |
| ACC (ACC-01..06) | 6 | Phase 0 (ACC-05, ACC-06 reassociation gates), Phase 2 (ACC-01..04 harness lands with LDA) |
| QG (QG-01..08) | 8 | Phase 0 |

**Coverage:** 103/103 requirements mapped, no orphans, no duplicates. Note: the initial instruction referenced 88 v1 requirements, but the authoritative `REQUIREMENTS.md` file lists 103 (14 categories with IDs AD, CORE, LDA, GGA, MGGA, MODE, ALIAS, RS, CAPI, PY, KER, GPU, ACC, QG). All 103 are mapped.

## Phase-to-Milestone Mapping (for reference)

The design brief's milestones (`docs/design/11-process-and-milestones.md`) map to GSD phases as:

| Design Milestone | GSD Phase |
|------------------|-----------|
| M0 Workspace scaffolding | Phase 0 |
| M1 `xcfun-ad` | Phase 1 |
| M2 `xcfun-core` foundations | Phase 2 (first half) |
| M3 LDA tier | Phase 2 (second half, with parity harness) |
| M4 GGA tier + `Mode::Potential` | Phase 3 |
| M5 metaGGA + `Mode::Contracted` + aliases | Phase 4 |
| M6 `xcfun-rs` + `xcfun-capi` | Phase 5 |
| M7 Kernels + CPU batch | Phase 6 (first half) |
| M8 CUDA + Wgpu | Phase 6 (second half) |
| M9 Python + release | Phase 7 |

Phase 2 intentionally fuses M2 + M3 and absorbs the tier-2 validation harness, per the LDA-first-for-validation directive: the 1e-12 gate must be exercised continuously, starting when the first functional ships.

Phase 6 fuses M7 + M8: CPU batch (`CpuRuntime`) is the only runtime validatable at 1e-12 vs scalar, and GPU backends add tolerance tables and f64 gates on top; standard granularity folds them into one phase.

---

*Roadmap created: 2026-04-19*
*Last updated: 2026-04-19 after initial creation*
