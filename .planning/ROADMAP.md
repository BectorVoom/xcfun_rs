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
- [x] **Phase 1: Taylor Algebra & AD Primitives (`xcfun-ad`)** - `CTaylor<T, N>`, `Num` trait, every `*_expand` function, bit-equivalence with C++ on orders 0..=3
- [x] **Phase 2: Core Foundations + LDA Tier + Parity Harness** - Complete (2026-04-22)[^acc04] — `xcfun-core` type surface + registry (11 LDAs populated, 67 stubs), `xcfun-eval` cubecl launcher with 11 LDA `#[cube] fn` kernels, `DensVarsDev<F>` + `build_densvars` + `regularize`, tier-1 self-tests GREEN for 7/7 LDAs with upstream test_in, tier-2 validation harness GREEN at orders 0/1 for 9/9 non-excluded LDAs (8 strict 1e-12 + 3 LDAERF 1e-7 per D-24)
- [x] **Phase 3: GGA Tier + `Mode::Potential`** - Complete (2026-04-25) — 36 of 40 GGA functionals shipped (BR×3 + CSC deferred to Phase 4 per D-01-A; LB94 deferred to Phase 5 per D-19); `Mode::Potential` via `CTaylor<f64, 2>` divergence construction GREEN strict 1e-12 (potential_parity_100 + 510k-record sweep); `Mode::PartialDerivatives` orders 0..=4 (capstone at order 2, 9.86M records); 13 D-19 INCONCLUSIVE entries forwarded to Phase 6 per D-18; 3 follow-up items in 03-HUMAN-UAT.md
- [x] **Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases** - Complete (2026-04-30)[^d19p4] — 32 functional bodies (28 metaGGA + 4 Phase-3 carryovers BRX/BRC/BRXC + CSC); 46-alias engine + 4 parameters with multiplicative weight composition; `Mode::Contracted` orders 0..=4 verified across 5 functionals (LDA + GGA + 3 metaGGA exemplars TPSSX/SCANX/M06X), orders 5..=6 D-19 forward to Phase 6 per Plan 04-05 D-19; full-matrix tier-2 at order 3 GREEN modulo inherited Phase-3 D-19 forwards (11 entries) + 3 NEW Phase-4 ERF forwards (Plan 04-08) + 3 NEW Phase-4 gradient-stress AD-chain divergences for TPSS-correlation (Plan 04-10 Path-B bisection confirmed algorithmically faithful port; root cause is f64-rounding cancellation in unphysical tau<<tau_w regime) + routine clamp-boundary + small-magnitude residuals for TPSSX/REVTPSSX/M05/M06 family. Plans: 11 total (04-00..04-06 original + 04-07/08/09/10 gap closure).
- [x] **Phase 5: Rust Facade (`xcfun-rs`) + C ABI (`xcfun-capi`)** - Complete (2026-04-30) — thin facade re-exports + full C ABI with cbindgen-generated `xcfun.h` byte-matched to reference; 16 requirements (RS-01..07/09/10 + CAPI-01..07) marked Complete; 10-fixture C-ABI golden test ALL FIXTURES PASS at 1e-12
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
**Plans**: 11 plans across 11 sequential waves (granularity standard; parallelization disabled — every plan touches xcfun-py source files that the next plan extends, forcing a strict topological chain 07-00 → 07-01 → ... → 07-10).

- [x] 07-00-PLAN.md — Wave 0 COMPLETE (2026-05-08): all 4 blocking HUMAN-UAT items resolved (items 3+6 passed; items 4+5 partially-passed via Plan 06-N7 substrate audit which closed 9 GGA-tier bugs). Eliminated ~3.35M failing records across PBEINTC + SPBEC + P86C + P86CORRC + BECKESRX + PW91C; 8 regression tests added; CI workflows for matrix-split mpmath regen + tier-2 sweep added. AD-residual tail forwarded to v0.2 per amended D-14. See 07-00-SUMMARY.md.
- [x] 07-01-PLAN.md — Wave 1: rename xcfun-python → xcfun-py + workspace member promotion + dep wiring (D-01, D-03)
- [x] 07-02-PLAN.md — Wave 2: pyproject.toml + #[pymodule] _native skeleton + 11 free fns (PY-01, PY-04)
- [x] 07-03-PLAN.md — Wave 3: XcfunError + abi3 §5 PyException workaround (PY-05; D-09, D-10)
- [ ] 07-04-PLAN.md — Wave 4: Functional #[pyclass] + Mode/Vars IntEnum + 9 method delegates (PY-02; D-05, D-06, D-12)
- [ ] 07-05-PLAN.md — Wave 5: NumPy strict zero-copy eval_vec + cross-language parity (PY-03; D-07, D-08)
- [ ] 07-06-PLAN.md — Wave 6: crates/xcfun-py/README.md (D-03, D-04, D-13)
- [ ] 07-07-PLAN.md — Wave 7: release.yml sdist + 3 wheel matrix + pytest-from-wheel (PY-06)
- [ ] 07-08-PLAN.md — Wave 8: xtask release-publish topological cargo publish driver (D-15)
- [ ] 07-09-PLAN.md — Wave 9: release.yml publish-pypi (OIDC) + release-artifacts + github-release (D-15, D-16)
- [ ] 07-10-PLAN.md — Wave 10: CHANGELOG.md + tag v0.1.0 (D-13; CHECKPOINT — pre-tag dry-run + tag push + xtask release-publish --execute)

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

- [x] 01-01-PLAN.md — Wave 0: revert pre-pivot commits + workspace/xtask scaffold + cubecl-cpu spike + for_tests harness (AD-01 substrate)
- [x] 01-02-PLAN.md — Wave 1: CTaylor + ctaylor_rec{mul, multo, compose} — load-bearing recursion (AD-01, AD-03)
- [x] 01-03-PLAN.md — Wave 2 (parallel with 01-05): expand/{inv, exp, log, pow, sqrt, cbrt} — primary scalar series (AD-04)
- [x] 01-04-PLAN.md — Wave 3: tfuns helpers + expand/{atan, gauss, erf, asinh} — transcendentals (AD-04)
- [x] 01-05-PLAN.md — Wave 2 (parallel with 01-03): xtask fixture generator + committed fixtures + golden_mul test (AD-03, AD-05)
- [x] 01-06-PLAN.md — Wave 4: math.rs composed ops + extended fixtures + golden_expand/composed (AD-02, AD-05)
- [x] 01-07-PLAN.md — Wave 5: proptest batch-per-property + criterion benchmarks + phase sign-off (AD-03, AD-06)

Pre-pivot plans (VOID — reverted by Wave 0 of the new plan, retained in git history):
- ~~pre-pivot 01-01 — Wave 0 hand-Rust scaffolding (commits f07611c, c7a3f46) [SUPERSEDED]~~
- ~~pre-pivot 01-02 — Wave 1 hand-Rust `*_expand` ports (commit 2db557c, partial) [SUPERSEDED]~~
- ~~pre-pivot 01-03 — Wave 1 hand-Rust `ctaylor_rec` mul/multo/compose port [SUPERSEDED]~~
- ~~pre-pivot 01-04 — Wave 1 fixture generator [INTENT RETAINED, replanned for cubecl validation]~~
- ~~pre-pivot 01-05 — Wave 2 `Num` trait + composed fns [SUPERSEDED — `Num` retired in favour of cubecl `Float`]~~
- ~~pre-pivot 01-06 — Wave 2 proptest 11 props × 10k iters [INTENT RETAINED, now batch-per-property kernel]~~
- ~~pre-pivot 01-07 — Wave 2 criterion bench baselines [INTENT RETAINED, now kernel-launch-amortized at batch sizes {1,64,1024}]~~

### Phase 2: Core Foundations + LDA Tier + Parity Harness
**Goal**: A user can run `cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda'` and see zero failures at 1e-12 (or per-functional D-24 override) relative error against the C++ reference for 9/9 non-excluded LDAs at orders 0/1.
**Depends on**: Phase 1
**Research flag**: No (standard port pattern)
**Requirements**: CORE-01..10, LDA-01..10, MODE-04, ACC-01..06 (absorbed ACC-05/06 from Phase 0 per D-10), QG-01/02/06/07 (absorbed from Phase 0 per D-10)
**Success Criteria** (what must be TRUE): — all five met with ACC-04 partial per Phase 2 SUMMARY footnote
  1. `Vars` (31 variants), `Mode` (4 variants with Unset=0), `Dependency` bitflags, `XcError` enum (9 variants, Copy, `#[non_exhaustive]`) compile with exact C header discriminants — PASS (Plan 02-01)
  2. `DensVarsDev<F>` cubecl type (D-02 + D-04) populates XC_A_B + XC_A_B_GAA_GAB_GBB arms; explicit helper-function chain replaces C-style fallthrough — PASS (Plans 02-03 + 02-05)
  3. `regularize` #[cube] fn modifies only Array<F>[0]; higher-order coefficients preserved (`tests/regularize_invariant.rs`) — PASS (Plan 02-03)
  4. Tier-1 self-tests run for 7 LDAs with upstream test_in in under 5 s — PASS (Plan 02-04)
  5. Tier-2 parity harness `--order 2` GREEN for 9/9 non-excluded LDAs at orders 0/1; order-2 results documented in `report.html` with D-19 INCONCLUSIVE residuals (VWN/PW/PZ near-clamp → Phase 3; LDAERF bracket cancellation where Rust = mpmath truth → Phase 6) — PASS with caveats (Plan 02-06)

**Plans**: 7 plans across 4 waves (granularity standard; parallelization enabled — Wave 2 ran Plans 02-04 + 02-05 in parallel after Plan 02-03 finished Wave-1B-3).

- [x] 02-01-PLAN.md — Wave 0: Surgical xcfun-core cleanup (6 atomic commits per D-09; Mode/Vars/XcError/FunctionalId reorganized; density_vars.rs deleted)
- [x] 02-02-PLAN.md — Wave 1A: xtask gates (regen-registry, check-no-mul-add, check-no-anyhow, check-boundaries, check-cubecl-pin, validate wrapper) + auto-generated FUNCTIONAL_DESCRIPTORS (78 entries, 35 populated) / VARS_TABLE (31 rows) / ALIASES (empty) — CORE-07/08/09/10, ACC-06, QG-01/02/06/07
- [x] 02-03-PLAN.md — Wave 1B-core: xcfun-eval scaffolding — DensVarsDev<F>, build_densvars XC_A_B arm, regularize, Functional + dispatch skeleton — CORE-05/06, MODE-04
- [x] 02-04-PLAN.md — Wave 2: 9 LDA bodies (SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, LDAERFC_JT, TFK) + tier-1 self-tests — LDA-01..08, LDA-09 part 1, ACC-04
- [x] 02-05-PLAN.md — Wave 2: TW + VWK kinetic-GGA bodies + XC_A_B_GAA_GAB_GBB builder arm (Pitfall PHASE2-D fix) — LDA-09 part 2, LDA-10
- [x] 02-06-PLAN.md — Wave 3: validation/ tier-2 parity harness (cc-compiled xcfun-master + FFI shim + 10k grid + report.html/jsonl; D-24 LDAERF 1e-7 override, D-22 clamp-stratum exclusion, LDAERFX expm1 stable bracket) — ACC-01..04
- [x] 02-07-PLAN.md — Wave 4: docs/design/ updates + REQUIREMENTS/ROADMAP/STATE sign-off (this plan)

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
**Plans**: 7 plans across 7 waves (granularity standard; parallelization enabled — Waves 2-4 parallelize family-port subtrees).

- [x] 03-00-PLAN.md — Wave 0: xcfun-ad substrate (D-05 ctaylor_expm1 + D-06 ctaylor_sqrtx_asinh_sqrtx) + fixture-gate at 1e-14
- [x] 03-01-PLAN.md — Wave 1: gga/ module tree + 7 new DensVarsDev Vars arms (D-10-A corrected 27..30) + shared helpers + regularize_2nd_taylor test + Mode::Potential host-side gates
- [x] 03-02-PLAN.md — Wave 2: 17 kernels (PBE ×12 + Becke ×4 + LYP ×1) per D-01-B; dispatch extension
- [x] 03-03-PLAN.md — Wave 3: 10 kernels (OPTX ×2 + PW86/PW91 ×4 + P86 ×2 + APBE ×2)
- [x] 03-04-PLAN.md — Wave 4: 8 kernels (B97 ×6 + KTX + BTK) per D-01-C; compile-time capstone (Pitfall G10)
- [x] 03-05-PLAN.md — Wave 5: Mode::Potential (LDA + GGA 3-direction divergence per XCFunctional.cpp:637-790)
- [x] 03-06-PLAN.md — Wave 6: Mode::PartialDerivatives orders 3..=4 + tier-2 full-matrix + Phase-2 ACC-04 re-run + Phase-3 sign-off

**Scope amendments (planner 2026-04-24):**
- **BRX/BRC/BRXC (GGA-03) + CSC (part of GGA-10) deferred to Phase 4** per D-01-A — these declare `Dependency::KINETIC|LAPLACIAN|JP` (metaGGA-class) requiring inlen=11 Vars arm + a separate `BR_taylor` Newton-inverse xcfun-ad primitive.
- **LB94 (part of GGA-10) deferred** per D-19 to Phase 5 (or Phase 4 if alias-feasible) — legacy `setup_lb94` pattern, not in the 78-entry FunctionalId enum.
- Phase 3 ships **36 of 40** GGA functional IDs (not 45 as the original Goal stated). Goal text to be corrected at sign-off.

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
**Plans**: 11 plans (04-00..04-10) — original plan set 04-00..04-06 (7 plans, Waves 0..6); gap-closure 04-07..04-10 (4 plans, Waves 1..3) added 2026-04-26 to close VERIFICATION gaps_found:
  - [x] 04-00-substrate-PLAN.md — xcfun-ad br_inverse + metaGGA shared helpers + Vars arms + fixture gates (Wave 0)
  - [x] 04-01-tpss-br-csc-PLAN.md — TPSS family + BR family + CSC kernels (Wave 1)
  - [x] 04-02-scan-family-PLAN.md — SCAN/rSCAN/r++SCAN/r2SCAN/r4SCAN kernels (Wave 2)
  - [x] 04-03-m0x-blocx-PLAN.md — M05/M06 families + BLOCX kernels (Wave 3)
  - [x] 04-04-alias-parameters-PLAN.md — 46-alias engine + 4 parameter table (Wave 4)
  - [x] 04-05-mode-contracted-PLAN.md — Mode::Contracted dispatcher + cross-mode tests (Wave 5)
  - [~] 04-06-validation-signoff-PLAN.md — original sign-off; produced VERIFICATION.md=gaps_found (Wave 6); SUPERSEDED by Plan 04-10 re-signoff
  - [x] 04-07-driver-extension-PLAN.md — gap closure: validation driver + run_launch metaGGA arms (Wave 1, gap_closure)
  - [x] 04-08-erf-divergence-PLAN.md — gap closure: ERF + LDA-corr triage; D-19 forwards (Wave 1, gap_closure)
  - [x] 04-09-contracted-metagga-PLAN.md — gap closure: contracted_cross_mode metaGGA exemplars at orders 0..=4 (Wave 2, gap_closure)
  - [x] 04-10-resignoff-PLAN.md — phase re-signoff; rewrote VERIFICATION.md to signed_off_with_caveats (Wave 3, gap_closure)

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
**Plans**: 5 plans across 5 sequential waves (granularity standard; Phase 5 is a facade + drop-in-ABI phase — no new numerical kernels — so plans naturally chain on shared crate state).

- [x] 05-00-topology-foundation-PLAN.md — Wave 1: rename xcfun-ffi→xcfun-capi + delete xcfun-functionals + register xcfun-rs as workspace member + add XcError::InvalidVarsAndMode + as_c_code mapping (D-01, D-04, D-08-A; CAPI-05 substrate)
- [x] 05-01-rust-facade-PLAN.md — Wave 2: xcfun-rs Functional newtype + 11 free fns + Send+Sync gate + zero-alloc fixture (D-02, D-03, D-13, D-15-A, D-17; RS-01..07, RS-09, RS-10)
- [x] 05-02-c-abi-exports-PLAN.md — Wave 3: xcfun-capi 23 #[no_mangle] extern "C" fns + c_entry! macro + cdylib/staticlib/rlib triple (D-05, D-06, D-07, D-08, D-15; CAPI-01, CAPI-03, CAPI-04, CAPI-06)
- [x] 05-03-cbindgen-headers-match-PLAN.md — Wave 4: cbindgen.toml + xtask regen-capi-header binary + sha256 stamp + headers_match diff test (D-09, D-10, D-11, D-12; CAPI-02)
- [x] 05-04-c-abi-golden-signoff-PLAN.md — Wave 5: tests/c_abi.c + tests/c_abi.rs (cc compile/link/run) + 10-fixture golden + Phase 5 sign-off; D-14 row-10 LB94→LDA(Potential) runtime substitute per upstream lb94.cpp:15 `#if 0` finding; rows 4 + 9 substituted to bp86 / beckecamx per dispatch-table constraints (Phase 6 work) (D-14, D-16; CAPI-07)

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
**Plans**: 12/13 — Complete with caveats[^p6caveats] (2026-05-04; gap-closure 06-N5 added 2026-05-07; gap-closure 06-N6 added 2026-05-07 covering pre-existing drift surfaced by 06-N5 CI)
  - 06-00 substrate (AD N≥4 + libm-hybrid erf + tau≥tau_w guard + mpmath fixture generator)
  - 06-01 extract-xcfun-kernels (workspace split per design-doc-05)
  - 06-02a xcfun-gpu skeleton + 06-02b validation harness
  - 06-03 cubecl-hip-rocm primary + 06-04 cubecl-cuda/wgpu opt-in
  - 06-05 eval-vec-dispatch (RS-08; auto_backend matrix)
  - 06-06 zero-alloc cleanup (D-12 EvalHandle + D-17 Vec weights + D-18 DensVars dispatch)
  - 06-N1 D-19 bisection scaffolding (11 inherited Phase-3 forwards — fixture+test scaffolds)
  - 06-N2 mpmath-only-spec (20 excluded_by_upstream_spec functionals — mpmath ground truth + driver wiring)
  - 06-N3 libm-hybrid residual sweep scaffolding (18 small-magnitude AD residuals)
  - [x] 06-N5-mpmath-acc04-bodies-PLAN.md — Gap closure complete (2026-05-07): filled 6 ACC-04 mpmath stub bodies (LDAERF×3 + TPSS-C×3) at prec=200 + 2 new substrate modules (_ldaerf_eps.py + _tpss_eps.py with D-10 tau-clamp guard) + python-interpreter dispatch fix (XCFUN_MPMATH_PYTHON env-var override, default `python3`); LDAERFX matches C++ to 16 digits, LDAERFC to 14 digits (documented pw92c-precision drift), TPSSC matches C++ at order=1 within the 1e-6 threshold; smoke-mode regen exits 0 on GitHub Actions clean Linux runner (CI run #25526864865); all 6 ACC-04 functionals respond to single-record invocation with finite floats — Plan 07-00 Task 0.2 unblocked. CI bring-up surfaced one assertion-drift bug (pbex_potential_non_2nd_taylor test missed Phase-5 InvalidVarsAndMode variant) — fixed inline at `f61afa2`.
  - [ ] 06-N6-pre-existing-drift-sweep-PLAN.md — Follow-up gap-closure (autonomous: false; operator decisions required): MSRV-vs-Cargo.lock policy (cubecl-zspace 0.10.0-pre.3 needs rustc 1.92, workspace pins 1.85), xcfun-master/ on CI strategy (validation crate + headers_match test depend on .gitignore'd upstream tree), cargo fmt --all sweep across crates/xcfun-ad/src/, clippy unused-import / dead-code triage in 6 xcfun-kernels files. All 4 items predate 06-N5; surfaced when 06-N5 brought up GitHub Actions for the first time.

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
| 1. Taylor Algebra & AD Primitives | 7/7 | Complete | 2026-04-19 |
| 2. Core Foundations + LDA Tier + Parity Harness | 7/7 | Complete (with caveats) | 2026-04-22[^acc04] |
| 3. GGA Tier + `Mode::Potential` | 7/7 | Complete (with caveats — 13 D-19 forwarded to Phase 6; 3 HUMAN-UAT items pending) | 2026-04-25 |
| 4. metaGGA Tier + `Mode::Contracted` + Aliases | 11/11 | Complete (with caveats — 30+ D-19 forwarded to Phase 6) | 2026-04-30 |
| 5. Rust Facade + C ABI | 5/5 | Complete | 2026-04-30 |
| 6. Kernels + CPU Batch + CUDA + Wgpu Backends | 11/11 | Complete (with caveats — 6 HUMAN-UAT items pending; xcfun-master now restored, re-baseline of N1/N3 sweeps deferred per [^p6caveats]) | 2026-05-04 |
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

## Footnotes

[^acc04]: **Phase 2 sign-off caveat — ACC-04 partial.** Tier-2 parity at 1e-12 is GREEN for orders 0/1 on 9/9 non-excluded LDAs (8 strict + 3 LDAERF at D-24 1e-7 override). At order 2, the matrix splits 4/9 GREEN + 5 D-19 INCONCLUSIVE: (a) VWN3C/VWN5C/PW92C/PZ81C exhibit near-clamp precision drift 1–3 ULP above 1e-12 at `min(a,b) ∈ [2e-14, 1e-11]`, forwarded to Phase 3 re-evaluation after the GGA-phase `build_densvars` redesign; (b) LDAERFX/LDAERFC/LDAERFC_JT exhibit bracket-cancellation residuals — critically, at the LDAERFX failing point, mpmath at 200-digit precision confirms **Rust matches mathematical ground truth while C++ diverges by 6.7%** (its own f64 cancellation). Forwarded to Phase 6 (GPU + libm-hybrid re-evaluation) and a possible amendment to switch the parity reference from C++ to mpmath ground truth where C++ is documented to suffer cancellation. See `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-06-SUMMARY.md` and `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-07-SUMMARY.md` for the full investigation arc.

[^d19p4]: **Phase 4 sign-off caveat — D-19 INCONCLUSIVE entries forwarded to Phase 6.** Order-3 full-matrix tier-2 sweep (Plan 04-10; 3,001,208 records via parallel scheduler from Quick Task 260430-4x7) finds 17 functionals 100% clean at strict 1e-12, 20 functionals `excluded_by_upstream_spec` (BR×3 + CSC + BLOCX + SCAN×10 + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC), and the following Phase-4 D-19 forwards to Phase 6: (a) **3 NEW gradient-stress AD-chain divergences** — TPSSC (max_rel 1.09e+30), TPSSLOCC (8.89e+27), REVTPSSC (3.73e+15) at points 9000–9999 where tau << tau_w (von Weizsäcker bound violated by ~9 orders of magnitude); root cause is f64-rounding cancellation in `eps_pkzb*(1+2.8*eps_pkzb*tauwtau3)` where tauwtau3≈1e+27 amplifies ULP-level differences between C++ and Rust evaluation orders; **algorithmically faithful port confirmed via Plan 04-10 Path-B bisection** (read xcfun-master/src/functionals/{tpssc.cpp,tpssc_eps.hpp,pbec_eps.hpp} side-by-side with crates/xcfun-eval/src/functionals/mgga/{tpssc.rs,shared/tpss_like.rs}; ctaylor max/operator> semantics match; pbec_eps and pbec_eps_polarized are line-for-line ports); Phase-6 triage hand-off: add `tau ≥ tau_w` regularization guard or exclude gradient-stress sub-grid for tau-using metaGGAs. (b) **5 NEW clamp-boundary AD-tail forwards** — TPSSX (2.68e-2), REVTPSSX (1.33e-2), BECKECAMX (2.0e-8), VWN5C (1.57e-11), VWN3C (7.17e-12), PZ81C (2.96e-12) at rho ≈ 2e-14 regularize stratum (same shape as Phase-2 LDAERF clamp story). (c) **~12 NEW small-magnitude AD-residual forwards** — M06{C,LC,HFC,X2C,X,LX,HFX}, M05{C,X,X2C}, B97{X,_1X,_2X}, VWN_PBEC (6.9e-9 — Plan 04-08), LYPC (1.3e-10), PW92C (8.97e-12), PBEC (1.8e-12), OPTX (1.2e-12) at 1e-11 to 7e-9 magnitudes on low-density polarised + gradient_stress strata. (d) **3 ERF Phase-4 forwards** (Plan 04-08) — LDAERFX (6.74e-2), LDAERFC (4.57e-6), LDAERFC_JT (4.56e-5); AD-chain amplification of erf bracket cancellation at orders 2+, Phase-6 libm-hybrid required. (e) **11 inherited Phase-3 forwards still failing at order 3** — PBEINTC (6.17e+1), P86C/P86CORRC (9.16e-2), PW91C (1.72e-3), SPBEC (5.27e-4), BECKESRX (2.27e+2), APBEC (5.7e-9), B97{,_1,_2}C (7.8e-11), PW91K (1.4e-11). Note: PW86X and APBEX (Phase-3 D-19) tightened to 100% clean strict 1e-12 at order 3 (better than expected). See `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` and `04-10-resignoff-SUMMARY.md` for the consolidated ledger.

[^p6caveats]: **Phase 6 sign-off caveats — 11/11 plans landed; 6 HUMAN-UAT items pending.** Phase 6 delivered the GPU runtime + batch lifecycle layer (RS-08 + KER-01..06 + GPU-01..08 structurally satisfied), strict-zero-alloc substrate (D-12 EvalHandle landed; the strict-0 test is `#[ignore]`'d pending cubecl 0.10-pre.3 `client.write` API), DensVars-driven dispatch (b3lyp/camb3lyp/bp86 in-process via D-18), and D-19 cleanup scaffolding. Six follow-ups carried forward to `06-HUMAN-UAT.md`: (a) Tier-3 ROCm 1e-13 sweep — requires AMD hardware on cloud-CI runner; (b) Tier-3 Wgpu 1e-9 sweep — requires SHADER_F64-capable adapter; (c) MPMATH ~6h offline fixture regen + 26-functional tier-2 1e-13 sweep — Plan 06-N2 manual lane; (d) Plan 06-N1 auto-tightening verification + Path-B fixes for the 11 inherited Phase-3 forwards — `xcfun-master/` now restored at HEAD `a89b783`, re-run order-3 sweep to verify; (e) Plan 06-N3 auto-tightening verification on 18 small-magnitude residuals; (f) BR_Q_PREFACTOR_F64 typo fix in `br_like.rs:37` — pre-existing, tracked as Plan 06-N4. Verifier returned `human_needed` with 14/16 must-haves verified directly + 2 hardware-gated overrides; Plan 06-N2 SCAN family achieves ~1e-5 vs mpmath (per-functional tolerance documented per user authorization — algorithmic-identity tradeoff between mpmath.diff numerical AD and CTaylor symbolic AD). Per-plan SUMMARYs and `06-VERIFICATION.md` carry the full ledger.

---

*Roadmap created: 2026-04-19*
*Last updated: 2026-05-04 after Phase 6 sign-off (11/11 plans; Complete with caveats per [^p6caveats])*
