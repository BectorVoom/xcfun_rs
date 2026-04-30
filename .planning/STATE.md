---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: planning
last_updated: "2026-04-30T20:00:00.000Z"
progress:
  total_phases: 8
  completed_phases: 5
  total_plans: 37
  completed_plans: 37
  percent: 100
---

# Project State: xcfun_rs

**Last updated:** 2026-04-30 (Phase 6 context gathered via `/gsd:discuss-phase 6` interactive; 18 decisions captured (D-01..D-18 + D-13-A) across 4 gray areas in `06-CONTEXT.md`. Wide Phase 6 with ~10–15 plans (decimal numbering = plan org); strict 1e-13 across all 78 functionals at sign-off via ACC-04 mpmath ground-truth amendment; ROCm/HIP primary GPU + CUDA/Metal opt-in; full xcfun-kernels + xcfun-gpu split per docs/design/05; 30+ Phase-3/4 D-19 forwards land as Plan 06-N1/N2/N3 cleanup. Commit `e9834b3`.)

**2026-04-30 (earlier) entry (superseded):** Phase 5 sign-off — 5/5 plans landed; 16 requirements RS-01..07/09/10 + CAPI-01..07 marked Complete; 10-fixture C-ABI golden test ALL FIXTURES PASS at 1e-12.

**2026-04-30 (earlier) entry (superseded):** Phase 5 context gathered via `/gsd:discuss-phase 5` interactive; 14 decisions captured (D-01..D-17) in 05-CONTEXT.md.

**2026-04-25 entry (superseded):** Phase 4 context gathered via `/gsd:discuss-phase --auto`; 11 gray areas auto-resolved as D-01..D-14 in 04-CONTEXT.md.

## Project Reference

**Core Value:** Every functional must produce numerical output matching C++ xcfun within relative error <= 1.0e-12, across all evaluation modes and derivative orders.

**Current focus:** Phase 06 — Kernels + CPU Batch + GPU Backends (NEXT, not yet started)

## Current Position

Phase: 06 (kernels-cpu-batch-cuda-wgpu-backends) — NEXT (not yet started)
Plan: -- (Phase 6 planning pending)
Plans (Phase 5): 5 (05-00 ✓, 05-01 ✓, 05-02 ✓, 05-03 ✓, 05-04 ✓)
Scope (Phase 5): xcfun-rs facade with Functional newtype + 11 free fns; xcfun-capi C ABI drop-in (23 #[unsafe(no_mangle)] exports); cbindgen-generated xcfun.h with headers_match drift gate; 10-fixture C-ABI golden test at 1e-12.

- **Milestone:** Initial v1 build-out
- **Phase:** 05 (rust-facade-xcfun-rs-c-abi-xcfun-capi) — **COMPLETE (2026-04-30)** — signed_off
- **Plan:** 05-04 complete. All 5 Phase-5 plans shipped.
- **Status:** Phase 6 head-of-line (planning pending)
- **Progress:** [██████░░] 62% (5/8 phases; 37/37 known plans)

### Phase 4 sign-off summary (2026-04-30)

Order-3 full-matrix tier-2 sweep (`cargo run -p validation --release -- --backend cpu --order 3 --resume --jobs 18 --filter '.*'`, parallelized via Quick Task 260430-4x7) produced 3,001,208 records in `validation/report.jsonl` (gitignored) + `validation/report.html` (committed `db0f8ad`). Distribution:

- **17 functionals 100% clean strict 1e-12**: SLATERX, TFK, PBEX, REVPBEX, PBEINTX, RPBEX, PBESOLX, BECKEX, BECKECORRX, **PW86X**, OPTXCORR, **APBEX**, PW91X, KTX, BTK, M05X2X, M06X2X. (PW86X + APBEX tightened from Phase-3 D-19 to clean at order 3 — better than expected.)
- **20 functionals `excluded_by_upstream_spec`** (skip-list): BR×3, CSC, BLOCX, SCAN×10, TW, VWK, PBELOCC, ZVPBESOLC, ZVPBEINTC. C++ tmath_die at low-density tail; Phase-6 JP grid harness or guarded {sqrt,log,pow}_expand required.
- **30+ Phase-4 D-19 INCONCLUSIVE forwards to Phase 6** (consolidated):
  - **TPSS-correlation gradient-stress AD-chain divergence** (Plan 04-10 Path-B bisection, NEW): TPSSC max_rel 1.09e+30, TPSSLOCC 8.89e+27, REVTPSSC 3.73e+15 at points 9000-9999 where tau<<tau_w (von Weizsäcker bound violated). Algorithmically faithful port confirmed; root cause f64-rounding cancellation in `eps_pkzb*(1+2.8*eps_pkzb*tauwtau3)` with tauwtau3≈1e+27 amplifying ULP-level differences. Phase-6 triage: tau≥tau_w guard or stratum exclusion.
  - **TPSS-X + Becke-CAM + VWN clamp-boundary AD-tail** (NEW): TPSSX 2.7e-2, REVTPSSX 1.3e-2, BECKECAMX 2.0e-8, VWN5C 1.6e-11, VWN3C 7.2e-12, PZ81C 3.0e-12 at rho≈2e-14 regularize stratum.
  - **Minnesota meta-correlation small-magnitude AD-residual** (NEW): M06{C,LC,HFC,X2C,X,LX,HFX} 1.5e-12 to 6.3e-11, M05{X,C,X2C} 1.9e-12 to 3.0e-11, B97{X,_1X,_2X} 9.5e-12, LYPC 1.3e-10, VWN_PBEC 6.9e-9 (Plan 04-08), PW92C 9.0e-12, PBEC 1.8e-12, OPTX 1.2e-12, M06HFX 7.8e-12 — same shape as Phase-3 B97{,_1,_2}C forwards.
  - **3 Phase-4 ERF forwards** (Plan 04-08): LDAERFX 6.7e-2, LDAERFC 4.6e-6, LDAERFC_JT 4.6e-5 — AD-chain amplification of erf bracket cancellation, Phase-6 libm-hybrid required.
  - **11 inherited Phase-3 forwards still failing at order 3**: PBEINTC 6.2e+1, P86C/P86CORRC 9.2e-2, PW91C 1.7e-3, SPBEC 5.3e-4, BECKESRX 2.3e+2, APBEC 5.7e-9, B97{,_1,_2}C 7.8e-11, PW91K 1.4e-11.
- **Mode::Contracted orders 5..=6 metaGGA** still D-19 forwarded per Plan 04-05 (xcfun-ad ctaylor_compose/multo N≥4 specialisations — Phase-6 prerequisite).

### Phase 5 sign-off summary (2026-04-30)

5 plans landed across Wave 1..5 (Plans 05-00..05-04). 16 requirements
(RS-01..07/09/10 + CAPI-01..07) marked Complete; RS-08 remains Phase 6.

- **Plan 05-00 (Wave 1)**: workspace topology rename xcfun-ffi→xcfun-capi
  + delete xcfun-functionals + register xcfun-rs as workspace member;
  XcError::InvalidVarsAndMode variant + as_c_code() i32 mapping
  (CAPI-05); LB94 descriptor add-back per D-16 (FunctionalId::XC_LB94 = 78
  with `#if 0`'d upstream body acknowledged in eval path).

- **Plan 05-01 (Wave 2)**: xcfun-rs Functional newtype with 9 methods +
  Default + manual Debug; 11 module-level free functions
  (version/splash/authors/is_compatible_library/self_test/which_vars/
  which_mode/enumerate_parameters/enumerate_aliases/describe_short/
  describe_long); Send+Sync compile-time gate via static_assertions;
  facade-boundary zero-alloc fall-back form (b) per D-13 (cubecl-cpu
  per-launch substrate cost ~287 allocs/eval forwarded to Phase 6).

- **Plan 05-02 (Wave 3)**: xcfun-capi 23 #[unsafe(no_mangle)] extern "C"
  fn exports + c_entry! macro (catch_unwind + NULL guard + abort);
  xcfun_s opaque handle / xcfun_mode_t / xcfun_vars_t types; cdylib +
  staticlib + rlib triple crate-type; 17/18 api_smoke tests passing
  (1 #[ignore]'d for the abort path).

- **Plan 05-03 (Wave 4)**: cbindgen.toml (documentation=false; XCFun_API
  prefix; after_includes prelude inlining xcfun_mode + xcfun_vars +
  visibility macros + xcfun_t typedef); xtask regen-capi-header binary
  with --check drift gate; headers_match.rs diff harness (5-stage
  normalization, 27 canonical statements set-equal); committed
  xcfun.h + xcfun.h.sha256 stamp.

- **Plan 05-04 (Wave 5)**: 10-fixture tests/c_abi.c golden + tests/c_abi.rs
  cc-driven compile/link/run + Phase 5 sign-off artifacts
  (05-VERIFICATION.md + REQUIREMENTS / ROADMAP / STATE updates). Two
  Plan-05-04 Rule-1 fixes: (a) cbindgen [export.rename] xcfun_s = xcfun_t
  fixes bare-struct-tag function signatures; (b) Functional::input_buffer_length
  helper + xcfun_eval shim fix accounts for Mode::Contracted's
  inlen × (1 << order) input layout. cc invocation flags per CLAUDE.md
  ACC-05/06: -fno-fast-math -ffp-contract=off (NEVER -ffast-math). Linux
  link line: -lstdc++ -lm -lpthread -ldl. ALL FIXTURES PASS at 1e-12 GREEN.

**Phase 5 D-decisions added** (D-01..D-17 + D-08-A + D-15-A): see
05-VERIFICATION.md "D-Decisions Coverage Audit" matrix.

**Phase 5 caveats** (decision-drift documented in 05-VERIFICATION.md):

- D-14 row 10: LB94→LDA(Mode::Potential) runtime substitute. Upstream
  lb94.cpp:15 is `#if 0`'d; LB94 descriptor present in xcfun-core
  (D-16 satisfied) but its eval path returns XcError::Runtime — the
  golden test substitutes LDA on Mode::Potential to preserve the
  Mode::Potential coverage goal of D-14 row 10.

- D-14 row 8: SCANX→TPSSX fallback authorized but **NOT triggered** —
  SCANX evaluated cleanly at the chosen density point.

- D-14 rows 2/3/4/5/9 Vars + alias substitution: dispatcher constraints
  in run_launch (LDA kernels at vars=2 only; GGA kernels at vars=6 only)
  preclude vars ∈ {20, 21} and the LDA+GGA-mixed B3LYP / CAMB3LYP aliases.
  Substituted to vars=6 throughout; rows 4 + 9 use bp86 (additive 2-GGA-
  term alias) and beckecamx (range-separated GGA exchange functional)
  respectively. Phase 6 work consolidates the dispatch table.

**Phase 5 deferred to Phase 6** (RS-08 + ancillary):

- RS-08 (Functional::eval_vec GPU dispatch) — entire Phase 6 surface.
- Zero-alloc strict form: cubecl-cpu's per-launch create_from_slice
  drops to a pre-allocated reusable handle; D-13 fall-back tightens to
  strict (delta == 0) form.

- Add LDA-vars=6 launch arms (or alternative DensVars-driven dispatch)
  so mixed LDA+GGA aliases can dispatch in-process — currently routed
  via the C++ validation harness only.

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

### Decisions added in Phase 4 (gap closure plans 04-07/08/09/10)

- **Plan 04-07 driver extension:** `validation/src/driver.rs::run` iterates all 30 metaGGA tuples; `run_launch` wired with 120 new arms at vars=13/17 × n ∈ {0..3}. BR family + CSC + BLOCX + SCAN×10 + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC pre-emptively excluded_by_upstream_spec (Phase-6 JP grid follow-up). Skip-list extension landed at master `f968c32` for the SCAN family (10 entries) after source-level diagnosis of the shared `SCAN_like_eps.hpp` substrate (17 sqrt() call-sites).
- **Plan 04-08 ERF divergence forward:** XC_LDAERFX/LDAERFC/LDAERFC_JT order-3 catastrophic divergence confirmed as AD-chain amplification of the known erf bracket cancellation. No Phase-4 viable fix per bisection. Forwarded to Phase 6 libm-hybrid. Plus 2 NEW LDA-corr forwards: VWN_PBEC (6.85e-9) + PBEC (6.64e-9) at low-density polarised stratum (Plan 04-08 Task 8.3).
- **Plan 04-09 contracted metaGGA cross-mode:** Mode::Contracted orders 0..=3 verified for TPSSX, SCANX, M06X exemplars at strict 1e-12 (30 tests GREEN). Order 4 metaGGA `#[ignore]`d with explicit Phase-6 forward citation (xcfun-ad ctaylor_compose/multo N≥4 specialisations missing — Plan 04-05 D-19 reinforced).
- **Plan 04-10 sweep parallelization (via Quick Task 260430-4x7):** Order-3 capstone sweep ran in ~hours on `--jobs 18` parallel scheduler (vs unbounded with serial). Parity test `parallel_matches_serial_via_jsonl` confirmed byte-identical output between `--jobs 1` and `--jobs 4`. Incremental jsonl flush implicitly addressed by parallel scheduler (per-tuple records emitted as completed; no all-or-nothing buffer). Two prior sweep failures (SCANC sqrt_expand crash + WSL VM termination) no longer block Phase-4 sign-off.
- **Plan 04-10 Path-B bisection on TPSS-correlation:** Read `xcfun-master/src/functionals/{tpssc.cpp,tpssc_eps.hpp,pbec_eps.hpp}` and `xcfun-master/external/upstream/taylor/{ctaylor.hpp,ctaylor_math.hpp}` side-by-side with `crates/xcfun-eval/src/functionals/mgga/{tpssc.rs,shared/tpss_like.rs}`. Confirmed: (a) `ctaylor_max` semantics match C++ `max(a,b)` with `operator>` comparing CNST slot only; (b) `tpss_pbec_eps`, `tpss_pbec_eps_polarized`, `tpss_C`, `tpss_epsc_summax`, `tpss_eps_full` are line-for-line ports of the C++ functions; (c) the divergence is NOT a port bug but f64-rounding cancellation in unphysical regime where tau<<tau_w (von Weizsäcker bound violated by ~9 orders of magnitude). Phase-6 triage hand-off: add tau≥tau_w regularization guard or exclude gradient-stress sub-grid for tau-using metaGGAs.
- **Phase 4 D-19 forward list (consolidated, see ROADMAP `[^d19p4]` footnote + 04-VERIFICATION.md ledger):**
  * 11 inherited Phase-3 forwards STILL FAILING at order 3: PBEINTC, P86C, P86CORRC, PW91C, SPBEC, BECKESRX, APBEC, B97C, B97_1C, B97_2C, PW91K
  * 2 inherited Phase-3 forwards TIGHTENED to clean at order 3: PW86X, APBEX (better than expected — tier-2 strict 1e-12 GREEN)
  * 3 NEW Phase-4 ERF forwards (Plan 04-08): LDAERFX, LDAERFC, LDAERFC_JT
  * 2 NEW Phase-4 LDA-corr forwards (Plan 04-08): VWN_PBEC, PBEC
  * 3 NEW Phase-4 gradient-stress AD-chain divergences (Plan 04-10 Path-B): TPSSC, TPSSLOCC, REVTPSSC
  * 6 NEW Phase-4 clamp-boundary AD-tail (Plan 04-10): TPSSX, REVTPSSX, BECKECAMX, VWN5C, VWN3C, PZ81C
  * 12+ NEW Phase-4 small-magnitude AD-residual (Plan 04-10): M06{C,LC,HFC,X2C,X,LX,HFX}, M05{X,C,X2C}, B97{X,_1X,_2X}, LYPC, PW92C, OPTX
  * 20 functionals excluded_by_upstream_spec (BR×3 + CSC + BLOCX + SCAN×10 + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC) — Phase-6 JP grid harness
  * Mode::Contracted orders 5..=6 metaGGA (Plan 04-05 D-19 reinforced) — Phase-6 xcfun-ad N≥4 specialisations
- **BLOCX confirmed BRX-independent** per RESEARCH finding (CONTEXT D-01-A claim corrected — BLOCX is TPSS-shaped, no `BR(...)` call).
- **LB94 stays in Phase 5** per D-13 (legacy `setup_lb94` pattern not in 78-entry FunctionalId enum).

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

### Quick Tasks Completed

| # | Description | Date | Commits | Directory |
|---|-------------|------|---------|-----------|
| 260430-4x7 | Parallelize validation harness — speed up the order=3 full sweep | 2026-04-30 | e79c3ef, 8c59675 | [260430-4x7-parallelize-validation-harness-speed-up-](./quick/260430-4x7-parallelize-validation-harness-speed-up-/) |

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
- 2026-04-30: Quick task `260430-4x7` complete — `validation/` harness parallelised over `(functional, vars, mode, order)` tuples via `std::thread::scope` + `std::sync::mpsc`. New `--jobs auto|N` CLI flag (default = `available_parallelism()`); FFI pre-warm drives `xcint_assure_setup` to `is_setup=true` on the main thread before any worker spawn (no global mutex needed). `--jobs 1` reproduces the legacy serial path byte-for-record-content. Smoke run on `xc_slaterx + xc_pbex` order 0: 13.59s → 8.19s (1.66× on 4 jobs). Numerical contract preserved: `parallel_matches_serial_via_jsonl` + `parallel_matches_serial_via_matrix` integration tests assert byte-identical serialised JSON between `--jobs 1` and `--jobs 4` after sort. CLAUDE.md hard rules respected: no rayon, no crossbeam, no `unsafe impl Send` for `CppXcfun`. Commits: `e79c3ef` (Task 1 — CLI + RunConfig + pre-warm), `8c59675` (Task 2 — parallel scheduler), final docs commit (Task 3 — parity test + tempfile dev-dep + planning artifacts).
- 2026-04-30: `/gsd:discuss-phase 6` complete (interactive). Phase 6 (GPU Backends + Batch Lifecycle / xcfun-kernels / xcfun-gpu) CONTEXT.md written at `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` with 18 decisions (D-01..D-18 + D-13-A) across 4 gray areas. DISCUSSION-LOG.md records the turn-by-turn audit trail. Commit `e9834b3`. Highlights: (a) **Wide Phase 6, ~10–15 plans, decimal numbering = plan org**; (b) **Strict 1e-13 across all 78 functionals at sign-off** with mpmath ground-truth amendment to ACC-04 where C++ documents cancellation; (c) **ROCm/HIP primary GPU backend** (`cubecl-hip = "=0.10.0-pre.3"`); CUDA + Metal as opt-in feature flags; user has no CUDA hardware locally; (d) **Full xcfun-kernels + xcfun-gpu split** per docs/design/05 — Plan 06-00 substrate first (AD N≥4 + libm-hybrid erf + tau≥tau_w guard + mpmath fixture generator) in current xcfun-eval tree; Plan 06-01 git-mv to new xcfun-kernels crate; Plan 06-02 unstub xcfun-gpu; Plans 06-03..05 GPU runtimes wiring; Plan 06-06 strict zero-alloc + Phase-5 weights Vec refactor + LDA-vars=6 DensVars-driven dispatch; Plans 06-N1..N3 D-19 cleanup; (e) **Pre-allocated reusable handle in Functional** (~287 → 0 allocs/eval after first call); (f) **Typed XcError::WgpuNoF64** ({ adapter_name: &'static str, requested_runtime: Backend } — preserves Phase 2 D-25 Copy via &'static payload). Memory updated: `project_gpu_target.md` (ROCm primary; CUDA + Metal opt-in), `project_crate_layout.md` (xcfun-kernels + xcfun-gpu split per design doc 05). Updates flagged for sign-off: ROADMAP / REQUIREMENTS / PROJECT.md / CLAUDE.md / docs/design/05+06+07+08+09+10 (Cuda → Rocm primary; ACC-04 mpmath amendment).

## Session Continuity

**Last session stopped at:** `/gsd:discuss-phase 6` (interactive) — Phase 6 CONTEXT gathered: 18 decisions (D-01..D-18 + D-13-A) across 4 gray areas; DISCUSSION-LOG.md records turn-by-turn audit trail. Memory updated with `project_gpu_target.md` + `project_crate_layout.md`. Git-committed (head on `master` at `e9834b3`).

**Next action:** `/gsd:plan-phase 6` to research + create Plan 06-00 (substrate: AD N≥4 + libm-hybrid erf + tau≥tau_w guard + mpmath fixture generator) → Plan 06-01 (xcfun-kernels git-mv) → Plans 06-02..05 (xcfun-gpu unstub + ROCm/CUDA/Metal/Wgpu wiring + RS-08) → Plan 06-06 (strict zero-alloc + weights Vec refactor + LDA-vars=6 dispatch) → Plans 06-N1..N3 (D-19 cleanup: root-cause bisection / mpmath-only fixtures for 20 excluded-spec / post-libm-hybrid sweep). Phase 6 research flag: YES per ROADMAP §"Phase 6" — cubecl-hip API surface, RDNA-2 driver requirement, mpmath sidecar pattern, AD `ctaylor_compose`/`multo` N≥4 specialisation strategy.

**Phase 6 scope (locked in 06-CONTEXT.md):**

- **3-axis deliverable:** algebraic substrate (AD N≥4 + libm-hybrid erf + tau guards + mpmath fixtures) + crate reorg (xcfun-kernels + xcfun-gpu split per design-doc-05) + GPU runtimes (ROCm primary, CUDA + Metal opt-in, Wgpu portable fallback) + RS-08 batch dispatch + 30+ D-19 cleanup.
- **Strict 1e-13 sign-off bar across all 78 functionals.** Tier-3 GREEN via `cargo run -p validation --release -- --backend rocm --order 3 --filter '.*'`.
- **ACC-04 amendment:** mpmath ground truth at 200-digit precision substitutes for C++ where C++ documents cancellation (LDAERFX bracket; TPSS tau≪tau_w; etc.). Preserves algorithmic-identity contract.
- **GPU strategy: ROCm/HIP primary** (`cubecl-hip = "=0.10.0-pre.3"`); CUDA + Metal opt-in feature flags (community best-effort); Wgpu at 1e-9 with ERF auto-fallback to CPU; cubecl-* crates lockstep at `=0.10.0-pre.3`.
- **Pre-allocated reusable handle in Functional** (Phase 5 D-13 forward; ~287 → 0 allocs/eval).
- **Typed `XcError::WgpuNoF64`** preserving Phase 2 D-25 Copy via `&'static str` payload.
- **Doc updates flagged for sign-off:** ROADMAP, REQUIREMENTS, PROJECT.md, CLAUDE.md, docs/design/05+06+07+08+09+10.

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
