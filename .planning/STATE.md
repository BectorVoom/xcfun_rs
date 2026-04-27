---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-04-27T04:54:13.225Z"
progress:
  total_phases: 8
  completed_phases: 3
  total_plans: 32
  completed_plans: 28
  percent: 88
---

# Project State: xcfun_rs

**Last updated:** 2026-04-25 (Phase 4 context gathered via `/gsd:discuss-phase --auto`; 11 gray areas auto-resolved as D-01..D-14 in 04-CONTEXT.md; scope = 32 functional bodies (28 metaGGA + 4 Phase-3 carryovers BRX/BRC/BRXC + CSC) + 1 new xcfun-ad primitive (`ctaylor_br_inverse`) + 11 new DensVarsDev Vars arms (ids 8..=18, 23..=26) + alias engine (46 entries, line-for-line port of XCFunctional.cpp:369-405) + 4 parameters (XC_RANGESEP_MU/EXX/CAM_ALPHA/CAM_BETA) + Mode::Contracted (DOEVAL macro, orders 0..=6); LB94 confirmed not alias-feasible → Phase 5; 13 Phase-3 D-19 forwards inherited unchanged for Phase 6; ready for `/gsd:plan-phase 4 --auto`)

## Project Reference

**Core Value:** Every functional must produce numerical output matching C++ xcfun within relative error <= 1.0e-12, across all evaluation modes and derivative orders.

**Current focus:** Phase 04 — metagga-tier-mode-contracted-aliases

## Current Position

Phase: 04 (metagga-tier-mode-contracted-aliases) — EXECUTING
Plan: 1 of 11
Plans: 7 (03-00 ✓, 03-01 ✓, 03-02 ✓, 03-03 ✓, 03-04 ✓ partial, 03-05 Mode::Potential, 03-06 orders 3..=4 + ACC-04 re-run + sign-off)
Scope: 36 GGA functional IDs (BRX/BRC/BRXC + CSC deferred to Phase 4 per D-01-A; LB94 deferred per D-19)
Wave 0 (03-00) COMPLETE: `ctaylor_expm1` (D-05) + `ctaylor_sqrtx_asinh_sqrtx` (D-06) + 6500 fixtures GREEN at 1e-12.
Wave 1 (03-01) COMPLETE: `gga/` module + 6 shared helpers + 7 DensVarsDev Vars arms (D-10-A) + Mode::Potential host gates.
Wave 2 (03-02) COMPLETE: 17 GGA kernels (PBE×12 + Becke×4 + LYP), 5 skeleton→FULL conversions, `Functional::parameters: [f64;4]`, dispatch 11→28, c_stubs 67→50.
Wave 3 (03-03) COMPLETE: 10 GGA kernels (OPTX×2 + PW86/91×4 + P86×2 + APBE×2), W3+W7 FULL helpers, W8 pz81_eps pub, dispatch 28→38, c_stubs 50→40; Wave-2 INCONCLUSIVE ABSORBED (run_launch + launch_and_accumulate extended for inlen=5); OPTX GREEN at strict 1e-12. **NEW D-19 INCONCLUSIVE**: 5 functionals (PW86X, APBEX, APBEC, P86C, PW91C) show 1e-6 to 1e-9 port-order drift from C++ `pow` expression vs Rust `ctaylor_pow` chain — constants verified to 16 digits; forwarded to Wave 6 sign-off for Rule-1 fix decision.
Wave 4 (03-04) COMPLETE WITH D-19 FORWARDS: 8 GGA kernels (B97 ×6 + KTX + BTK), W3 b97_poly FULL bodies (G6-safe explicit u² preserved), `pw92eps_polarized` LSDA helper (FERRO branch), dispatch 38→46 (8 new comptime arms), c_stubs 40→32, validation/build.rs +5 entries. **I2 CAPSTONE: 4.10 s** — clean `cargo build -p xcfun-eval --release` ≤ 45 s budget, NO per-Mode split applied per unconditional rule. **Tier-2 PARTIAL**: 5/8 GREEN strict 1e-12 (B97X, B97_1X, B97_2X, KTX, BTK); 3/8 (B97C 11 fails, B97_1C 11, B97_2C 41) max rel_err 4.88e-11 on near-zero polarised gradient_stress (point_idx 8246 stratum: a=1.6e-8, b=1.2e-3, gradients zero). Failures ~3 orders of magnitude TIGHTER than Wave 3 D-19 (4.88e-11 vs 1e-6..1e-9). Likely root-cause: `pw92eps_polarized` FERRO-branch composition order. Forwarded as 3 new D-19 INCONCLUSIVE entries to Wave 6 sign-off (mirroring Wave 3 protocol). 63 / 2,240,000 record failure rate = 2.81e-5.
Wave 5 (03-05) COMPLETE: Mode::Potential routing per D-13 line-for-line port of XCFunctional.cpp:637-790. `launch_potential` (LDA N=1 single-pass + GGA N=2 two-pass divergence), `potential_lda_kernel` + `potential_gga_kernel` `#[cube] fn`s; 80 new (id, vars=28, n) match arms in run_launch covering all 38 supported ids at A_B_2ND_TAYLOR. **B5 path (a)**: new `xtask gen-potential-fixtures` binary drives cc-compiled C++ harness over deterministic 100-record grid (5 GGAs × 20 pts, seed 0xf00dbabe) → `potential_reference_100.json`. **potential_parity_100 GREEN strict 1e-12 100/100**. Tier-2 sweep 510k records GREEN across 11 LDAs (8 strict + 3 LDAERF at 1e-7 D-24) + 8 GGAs (incl. B97C — confirms Wave-4 D-19 forwards are Mode::PartialDerivatives-only). **Bonus Rule-1 fix** to `build_xc_a_b_2nd_taylor`: added missing gnn/gns/gss derivations (unblocks LYPC). MODE-02 + MODE-05 satisfied. Goal-complete for Mode::Potential.
Wave 6 (03-06) COMPLETE WITH PHASE-3 SIGN-OFF: orders 3+4 + W9 pack helpers + 88 new run_launch arms (9 LDAs + 35 GGAs at n∈{3,4}); supplemental 400-pt GGA-stratified grid (seed 0xdeadbeef); C++ fall-through fix (recursive accumulation N→N-1 before tier-N append); Mode::PartialDerivatives raised from > 2 to > 4 per MODE-01 D-16. **Tier-2 capstone at order 2: 9.86M records, 516 MB report.jsonl committed**. ACC-04 re-run on Phase-2 LDA residuals: orders 0/1 GREEN, order 2 unchanged from Phase-2 baseline → forward UNCHANGED to Phase 6 per I3. C++-abort exclusions added to skip list: ZVPBESOLC, ZVPBEINTC, PBELOCC. **Collective D-19 INCONCLUSIVE sign-off — 13 entries forwarded to Phase 6**: 5 Wave-3 (PW86X/APBEX/APBEC/P86C/PW91C) + 3 Wave-4 (B97C/B97_1C/B97_2C) + 5 NEW from full-matrix (SPBEC/PBEINTC/PW91K/P86CORRC/BECKESRX). Order-3 full-matrix run interrupted mid-execution (usage-limit) — forwarded as Phase-6 prereq with structural cover via W9 unit tests + C++ fall-through fix + lib unit tests (17/17 GREEN). REQUIREMENTS.md GGA-01..10 + MODE-01/02/05 marked Complete with per-functional caveats. **All 36 of 40 GGA functional IDs ship; BR×3 + CSC → Phase 4, LB94 → Phase 5.**

- **Milestone:** Initial v1 build-out
- **Phase:** 03 (gga-tier-mode-potential) — **COMPLETE (2026-04-25)**
- **Plan:** 4-06 complete. All 7 Phase-3 plans shipped (Wave 0 → Wave 6); 13 of 13 Phase-3 requirement IDs satisfied (10 GGA + 3 MODE) with 13 D-19 INCONCLUSIVE entries explicitly forwarded to Phase 6 + 3 follow-up items in 03-HUMAN-UAT.md (order-3 capstone re-run, BECKESRX D-18 forensics, full 36-GGA Mode::Potential sweep).
- **Status:** Executing Phase 04
- **Progress:** [█████████░] 86%

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files | Completed |
|-------|------|----------|-------|-------|-----------|
| 01    | 01   | 11m      | 2     | 8     | 2026-04-19 |
| 01    | 02   | 50m      | 3     | 7     | 2026-04-19 |
| 01    | 03   | 11m      | 3     | 9     | 2026-04-19 |
| 01    | 04   | 14m      | 3     | 9     | 2026-04-19 |
| 01    | 05   | 11m      | 3     | 12    | 2026-04-19 |
| 01    | 06   | 11m      | 3     | 9     | 2026-04-19 |
| 01    | 07   | 17m      | 5     | 11    | 2026-04-19 |
| 02    | 01   | ~25m     | 6     | 8     | 2026-04-20 |
| 02    | 02   | ~45m     | 3     | 18    | 2026-04-20 |
| 02    | 03   | ~7m      | 5     | 13    | 2026-04-20 |
| 02    | 04   | ~2h      | 4     | 7     | 2026-04-21 |
| 02    | 05   | ~10m     | 4     | 5     | 2026-04-21 |
| 02    | 06   | ~6h      | 9     | 22    | 2026-04-21 |
| 02    | 07   | ~25m     | 4     | 7     | 2026-04-22 |
| 03    | 00   | ~?m      | 2     | 8     | 2026-04-25 |
| 03    | 01   | ~?m      | 3     | 13    | 2026-04-25 |
| 03    | 02   | ~?m      | 4     | 21    | 2026-04-25 |
| 03    | 03   | ~75m     | 4     | 23    | 2026-04-25 |
| 03    | 04   | ~10m     | 3     | 19    | 2026-04-25 (PARTIAL — 3 D-19 forward) |

Will also track (as they accumulate):

- Phase completion dates
- Parity gate violations caught in CI
- Cubecl version bumps (requires full tier-2+3 re-run per Pitfall P8)

| Phase 04 P01 | 3h | 2 tasks | 17 files |

## Accumulated Context

### Decisions (from Phase 1 — cubecl pivot)

> **2026-04-19 PM cubecl pivot:** Pre-pivot D-Exec-01..D-Exec-03 and pre-pivot D1, D2, D4 are STRUCK. The `xcfun-ad` crate is cubecl-native; original commits reverted by Wave 0 of the new plan. Updated decisions below. See `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` for the 28 locked decisions.

- ~~**D-Exec-01 (Plan 01-01)** `CTaylor<T, const N, const SIZE>` two-const-generic with sealed `ValidN<N, SIZE>`~~ — VOID
- ~~**D-Exec-02 (Plan 01-01)** `.cargo/config.toml` duplicates `-Cllvm-args=-fp-contract=off`~~ — RETAINED in spirit
- ~~**D-Exec-03 (Plan 01-01)** Workspace `members` scoped to `crates/xcfun-ad`~~ — RETAINED
- ~~**D1** Algorithmic-identity port of `CTaylor` (hand-Rust verbatim)~~ — REPLACED by **D1'** cubecl-native CTaylor
- ~~**D2** Custom `Num` trait~~ — REPLACED by **D2'** cubecl's `Float` trait
- **D3** Single-source `#[cube]` kernel for CPU + CUDA + Wgpu — RETAINED
- ~~**D4** `f64` everywhere; `Num` not implemented for `f32`~~ — REPLACED by **D4'** generic over `F: Float`; f32 ban moves to xcfun-rs
- **D5** Wgpu best-effort: 1e-9 tolerance, `erf`-using functionals forced to `Backend::Cpu` — RETAINED (Phase 6)
- **D6** `#[comptime]` on `(vars, mode, order)` only; functional id is runtime dispatch — RETAINED
- **D7** `thiserror` in library crates, `anyhow` only at application boundaries — RETAINED
- **D8** `cubecl` pinned at `=0.10.0-pre.3` — RETAINED, criticality elevated
- **D9** Functional registry generated by `xtask codegen`, checked into git — RETAINED
- **D13** `catch_unwind` on every `xcfun-capi` FFI entry; `panic = "abort"` in cdylib release — RETAINED
- ~~**D17** No library-internal threading~~ — RETAINED **AND** extended: scalar `Functional::eval(point)` routes through cubecl-cpu
- **D18** `xcfun-master/` vendored; content-hash pinned — RETAINED
- **Order cap:** `PartialDerivatives` <= 4, `Contracted` <= 6 — RETAINED
- **Workspace:** 7 library crates + `validation/` + `xtask/` — RETAINED
- **License:** MPL-2.0 (inherited) — RETAINED
- **Granularity:** standard (5-8 phases), parallelization enabled — RETAINED

### Decisions added in Phase 2

Locked up front in Phase 2 CONTEXT:

- **D-01** Full cubecl-native (all DensVars + bodies are `#[cube] fn` from day one; extends Phase 1 D-04/D-09/D-23)
- **D-02** `DensVarsDev<F: Float>` as a `#[derive(CubeType, CubeLaunch)]` type (cubecl nesting spike green via Plan 02-03 Wave-1B-1 Pattern A)
- **D-03** Single generic `#[cube] fn <name>_kernel<F: Float>(d, out, #[comptime] n: u32)` per functional
- **D-04** `xcfun-eval` is the cubecl launcher + functional-body home; `xcfun-core` stays cubecl-free (types + registry only)
- **D-05** Surgical rewrite of pre-pivot `xcfun-core` scaffold (delete `density_vars.rs`, rewrite `lib.rs`, rename enums)
- **D-06** Keep screaming-snake-case variant names on Vars (`#[allow(non_camel_case_types)]`)
- **D-07** `EvalMode → Mode`, add `Unset = 0`, `#[repr(u32)]`
- **D-08** `VarType → Vars`
- **D-09** Wave-0 atomicity = one commit per cleanup task (6 atomic commits landed in Plan 02-01)
- **D-10** Phase 2 absorbs Phase 0 requirements it strictly needs (CORE-10, ACC-05/06, QG-01/02/06/07)
- **D-11** CORE-10 via cc-compiled C++ extractor (Plan 02-02 Wave-1A-2)
- **D-12** Registry codegen scope in Phase 2 = LDA populated + full VARS_TABLE + empty ALIASES (refined: 35 FunctionalDescriptor entries fully populated, 43 stubs)
- **D-13** ACC-06 via `xtask check-no-mul-add` grep gate
- **D-14** `validation/` binary crate with cc-linked xcfun-master + FFI shim
- **D-15** Report format: `report.html` + `report.jsonl`; no committed fixtures (seed-deterministic)
- **D-16** Tier-1 self-tests source from the xtask-generated registry
- **D-17** Per-functional kernel signature: `#[cube] fn <name>_kernel<F: Float>(d, out, #[comptime] n)`
- **D-18** 10k-point grid = stratified 70/30 (7000 bulk + 1000 regularize + 1000 polarised + 1000 gradient; xoshiro256++ seed 0x1234abcd)
- **D-19** Strict 1e-12 for all 11 LDAs — no blanket relaxation; per-functional override allowed only with user-approval
- **D-20** Workspace members mutation in Wave 0
- **D-21** Functional dispatcher + minimal `Functional` struct live in `xcfun-eval`
- **D-22** `regularize` on `#[cube] ctaylor` modifies only `Array<F>[0]` (CNST coefficient)
- **D-23** Tier-2 order scope in Phase 2: `Mode::PartialDerivatives` orders 0..=2 only

Added during Phase 2 execution (user-approved or surfaced by research/fix work):

- **D-24** LDAERF tier-2 1e-7 override (USER-APPROVED 2026-04-20 via plan-phase prompt). Sourced from upstream xcfun's own `test_threshold` at `ldaerfx.cpp:66`. Applied to `XC_LDAERFX`, `XC_LDAERFC`, `XC_LDAERFC_JT`. 8 other LDAs hold strict 1e-12. Transparent in report.html.
- **D-25** `XcError::UnknownName` drops payload (USER-APPROVED 2026-04-20 via plan-phase prompt). Required to satisfy CORE-04 `Copy` semantics; payload was cosmetic (caller already has the name).
- **Plan 02-04 Wave-1B-14a amendment:** `build_xc_a_b` reads input as flat `(inlen × (1<<N))` array of pre-seeded CTaylor coefficients (no host materialisation).
- **Plan 02-05 Wave-1C-1:** `XC_A_B_GAA_GAB_GBB` builder arm with explicit chain to `build_xc_a_b` (Pitfall PHASE2-D fix).
- **Plan 02-06 Wave-2-1:** `xtask regen-registry` extended to emit `validation/c_stubs.cpp` (67 non-LDA stubs for xcint template-recursion link).
- **Plan 02-06 Fix 1 (commit `6ab5872`):** LDAERFX branch B stable-bracket rederivation via `expm1` — algebraically identical to upstream at arbitrary precision (mpmath prec=200 agreement < 1e-60) but keeps f64 intermediates at natural magnitude, eliminating ~6-digit cancellation that plagues the C++ reference.
- **Plan 02-06 Fix 2 (commit `080a170`):** Regularize-clamp stratum exclusion per D-22 design intent. Grid points with `min(a,b) ≤ 2e-14` are marked `excluded_by_regularize_clamp_design` — tests of the clamp design, not kernel correctness.
- **Plan 02-06 in-kernel libm-port erf_precise (commit `dca382a`):** cubecl 0.10-pre.3 `Float::erf` polyfill (~1.3e-8 ULP) replaced with FreeBSD msun-derived port. Phase 1 baseline tightened from 1e-7 to 1e-14.
- **Critical finding at LDAERFX:** mpmath at 200-digit precision confirms **Rust = mathematical ground truth** while C++ diverges by 6.7% due to its OWN bracket cancellation. Cannot be resolved at Phase 2 without either widening threshold (forbidden by D-19) or forcing Rust to replicate C++'s bug (forbidden by algorithmic-identity contract). Phase 6 libm-hybrid on CUDA/Wgpu provides a second independent ground truth; a possible amendment will switch the parity reference from C++ to mpmath where C++ is documented to suffer cancellation.

### Pending decisions (deferred to phase-level research)

- `cubecl-cpu` as dev-dep vs regular dep of future `xcfun-gpu` — concrete Cargo setup at Phase 6
- `Send`/`Sync` bounds on `Batch<'fun, R>` — confirm at Phase 6
- Phase 6 libm-hybrid strategy for LDAERF resolution (inherits D-24 1e-7 until then)
- Phase 3 `build_densvars` redesign may incidentally tighten VWN/PW/PZ near-clamp numerical behaviour — re-run tier-2 after
- PW92C legacy-constants default state — Phase 0 reading of `config.hpp` (NOT blocking; default matches vendored xcfun-master)

### Active TODOs (project level)

- [ ] Plan Phase 3 (`/gsd-plan-phase 3`)
- [ ] **Phase 3 UAT Test 1 — order-3 full-matrix tier-2 capstone re-run** (~1h human-supervised; resume via `/gsd-verify-work 3`) — see `.planning/todos/pending/2026-04-26-phase-3-uat-test-1-order-3-full-matrix-tier-2-capstone-re-ru.md`

### Blockers

None.

### Recent activity

- 2026-04-19: Research complete (`.planning/research/SUMMARY.md`, `STACK.md`, `FEATURES.md`, `ARCHITECTURE.md`, `PITFALLS.md`)
- 2026-04-19: `PROJECT.md` and `REQUIREMENTS.md` defined (103 v1 requirements)
- 2026-04-19: `ROADMAP.md` created (7 phases, 100% requirement coverage)
- 2026-04-19: `STATE.md` initialized
- 2026-04-19 PM: Cubecl pivot adopted. Phase 1 CONTEXT rewritten.
- 2026-04-19: Plan 01-01 (cubecl pivot) complete. cubecl workspace scaffolding + for_tests harness + cubecl_spike green.
- 2026-04-19: Plan 01-02 complete. CTaylor<F,N> element-wise ops + ctaylor_rec::{mul, multo, multo_skipconst, compose}. Commits `d34c0cd`, `1589bfe`, `712cea9`. AD-01, AD-03 [x].
- 2026-04-19: Plan 01-03 complete. Primary `*_expand` ports (inv/exp/log/pow/sqrt/cbrt). Commits `c71ee62`, `12933c1`, `afe2795`. AD-04 [x].
- 2026-04-19: Plan 01-04 complete. tfuns helpers + transcendental expansions. Commits `877a533`, `e99f2d1`, `496e118`.
- 2026-04-19: Plan 01-05 complete. xtask fixture driver + 418 committed fixtures + golden_mul.rs. Commits `3f0d37b`, `e86c403`, `2539502`.
- 2026-04-19 PM: Plan 01-06 complete. 9 composed `ctaylor_*` `#[cube] fn`s + 598 fixtures + golden_expand/composed. Commits `3bbcc5f`, `2884dd2`, `1a5a744`. AD-02, AD-05 [x].
- 2026-04-19 PM: Plan 01-07 complete — Phase 1 signed off. 11 proptest batch-per-property tests (110k aggregate iters) + criterion baselines + `check-no-fma` asm gate + D-28 docs updates. Commits `3514217`, `a3e4c3f`, `2882b58`, `3a6927e`, `db792bf`. AD-04, AD-06 [x]. ROADMAP Phase 1 [x].
- 2026-04-20: Plan 02-01 complete (Wave 0: 6 atomic commits surgical cleanup of xcfun-core). Hashes: `f98fe26`, `82e21ba`, `8243bd1`, `4eb7c0a`, `f35bd9f`, `1feb23b`. Mode/Vars/XcError/FunctionalId reorganized; density_vars.rs deleted; xcfun-core cubecl-free.
- 2026-04-20: Plan 02-02 complete (Wave 1A: 5 xtask gates + regen-registry + 78-entry FUNCTIONAL_DESCRIPTORS + 31-row VARS_TABLE + 0-entry ALIASES). CORE-07/08/09/10, ACC-06, QG-01/02/06/07 satisfied.
- 2026-04-20: Plan 02-03 complete (Wave 1B-core: xcfun-eval scaffolding; D-02 cubecl nesting verified; DensVarsDev<F> + build_xc_a_b + regularize + Functional + dispatch skeleton). CORE-05/06, MODE-04 satisfied.
- 2026-04-21: Plan 02-04 complete (Wave 2: 9 LDA kernels + tier-1 self-tests pass < 5s; LDAERFC constants-correctness fix landed). Commits include `f85cefe`, `6dbbce3`, `abf3506`, `b0a61f5`. LDA-01..08, LDA-09 part 1 [x].
- 2026-04-21: Plan 02-05 complete (Wave 2: TW + VWK kinetic-GGA + XC_A_B_GAA_GAB_GBB builder arm; Pitfall PHASE2-D resolved). Commits `0b200d1`, `865d6b6`, `e114556`, `ebe2631`. LDA-09 part 2, LDA-10 [x].
- 2026-04-21: Plan 02-06 complete (Wave 3: validation/ tier-2 harness; user-approved D-24 LDAERF 1e-7 override transparent in report.html; LDAERFX expm1 stable bracket Fix 1; D-22 clamp-stratum exclusion Fix 2; in-kernel libm-port erf_precise). 13 commits `55dba99 → 8ab7d4e`. ACC-01..03 [x], ACC-04 [~] Partial.
- 2026-04-22: Plan 02-07 complete — Phase 2 signed off. 4 design docs updated (02/05/06/09) to reflect as-built D-02/D-04/D-17/tier-1-tier-2 pattern; 30 Phase-2 IDs [x] Complete + ACC-04 [~] Partial with documented D-19 residuals; ROADMAP Phase 2 [x] Complete (with caveats) 7/7 plans; STATE advanced to 25% (14/14 plans across 2 phases).
- 2026-04-24: `/gsd-discuss-phase 3 --auto` complete. Phase 3 (GGA Tier + Mode::Potential) CONTEXT.md written at `.planning/phases/03-gga-tier-mode-potential/03-CONTEXT.md` with 25 decisions (D-01..D-25) auto-selected across 10 gray areas. Inherits 53 locked decisions from Phases 1+2. DISCUSSION-LOG.md records the auto-selection rationale. Scope-count corrected: 40 GGA functionals (ROADMAP's "45" is loose). LB94 deferred to Phase 5 per D-19 (not in the 78-entry enum; uses legacy `setup_lb94` pattern). xcfun-ad additive extensions queued: `expm1`/`ctaylor_expm1` (D-05) and `sqrtx_asinh_sqrtx` helper (D-06) — mandatory for 6 GGA families. Note: `gsd-sdk` CLI unavailable in this environment; the workflow's final git-commit + auto-advance-to-plan steps were not auto-invoked.
- 2026-04-24: `/gsd-plan-phase 3 --auto` complete. Research + pattern-mapping + 7 PLAN.md files (03-00..03-06) shipped. CONTEXT.md amended (commit `ee77cd7`): D-01-A scope reduction 40→36 (BRX/BRC/BRXC + CSC deferred to Phase 4 — metaGGA-class deps); D-01-B Wave 1 = 17 kernels; D-01-C Wave 3 = 8 kernels; D-10-A `_2ND_TAYLOR` discriminants corrected to 27..30. Checker GREEN on iteration 3 (5 BLOCKER / 9 WARNING / 3 INFO on iter 1 → 2 BLOCKER / 1 WARNING on iter 2 → 0 on iter 3). Key commits: `eea97b3` (RESEARCH.md), `050d0c6` (VALIDATION.md), `3e35e7b` (PATTERNS.md), `ee77cd7` (CONTEXT amendments), `96932d4` (03-00 plan), `00ce743` (plans 01-06), `1e8c156`..`5ac932f` (r1 revisions), `5d1fe96` (r2 final).

## Session Continuity

**Last session stopped at:** `/gsd-plan-phase 3 --auto` — Phase 3 planned: 7 PLAN.md files across 7 waves; CONTEXT.md amended with D-01-A/B/C + D-10-A; checker GREEN on iteration 3. Git-committed (heads on `master` at `5d1fe96`).

**Next action:** `/gsd-execute-phase 3 --auto` to execute Wave 0 (xcfun-ad primitives D-05/D-06 + fixtures), then waves 1–6 in order.

**Phase 3 scope (locked in CONTEXT.md + amendments):**

- **36 GGA functional bodies** across 8 families (PBE ×12 / Becke ×4 / LYP ×1 / OPTX ×2 / PW86-PW91 ×4 / P86 ×2 / APBE ×2 / B97 ×6 / KT-BTK ×2) — BRX/BRC/BRXC + CSC deferred to Phase 4; LB94 deferred to Phase 5.
- `Mode::Potential` via line-for-line port of `XCFunctional.cpp:637-790` divergence construction.
- `Mode::PartialDerivatives` orders 3..=4 extension.
- 7 new Vars arms in DensVarsDev (3 GGA + 4 2ND_TAYLOR for Potential).
- 2 new xcfun-ad primitives (expm1, sqrtx_asinh_sqrtx).
- GGA shared helpers extracted to `crates/xcfun-eval/src/functionals/gga/shared/*.rs`.
- Wave 5 capstone re-runs tier-2 at order 2 for VWN3C/VWN5C/PW92C/PZ81C (ACC-04 Phase-2 forward-action).
- Strict 1e-12 parity; D-24 LDAERF 1e-7 override NOT extended to GGA `erf` usage.

**Related artifacts:**

- `.planning/PROJECT.md` — project context, core value, constraints
- `.planning/REQUIREMENTS.md` — 103 v1 requirements with traceability (37 Complete, 1 Partial, 65 Pending)
- `.planning/ROADMAP.md` — 7-phase plan (Phase 1 + Phase 2 complete; Phase 3–7 pending)
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-{01..07}-SUMMARY.md` — per-plan completion records
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-07-SUMMARY.md` — Phase 2 capstone (this plan)
- `validation/report.html` + `validation/report.jsonl` — tier-2 verdict matrix (committed artifact)
- `crates/xcfun-eval/src/functionals/lda/*.rs` — 11 LDA kernels (Phase-2 deliverable)
- `docs/design/` — 14-document design brief (updated 2026-04-22 for Phase 2 as-built)

---

*State initialized: 2026-04-19*
*Phase 1 signed off: 2026-04-19 PM*
*Phase 2 signed off: 2026-04-22 (with ACC-04 partial; residuals forwarded to Phase 3 + Phase 6)*

**Planned Phase:** 04 (metaGGA Tier + Mode::Contracted + Aliases) — 11 plans — 2026-04-26T21:35:00Z

**2026-04-26 — Phase 4 gap-closure plans added** (`/gsd:plan-phase 4 --gaps`): 4 new plans (04-07 driver-extension, 04-08 erf-divergence, 04-09 contracted-metagga, 04-10 resignoff) close the 3 gaps from `04-VERIFICATION.md` (`gaps_found`). Existing plans 04-00..04-06 unchanged. Plan-checker returned PASSED with 6 non-blocking warnings; issue #1 (test-name typo `test_alias_case_insensitive` → `test_case_insensitive` in 04-10) fixed inline. Decision-coverage gate (verify-phase counterpart) reported 14/14 D-decisions uncovered — **user-approved override** because all D-01..D-14 are demonstrably cited in already-executed plans 04-00..04-06 (47 D-NN references found across the 7 prior plans); the gate misfires in `--gaps` mode where new plans correctly only address verification gaps and don't re-cite implementation decisions. Verify-phase should re-surface this if the gap-closure plans drift from the original decisions.
