---
phase: 02-core-foundations-lda-tier-parity-harness
plan: 07
subsystem: phase sign-off (design-brief + REQUIREMENTS + ROADMAP + STATE alignment)
tags: [wave-4, phase-sign-off, documentation, design-brief-update, requirements-traceability, d-02, d-04, d-17, d-24, d-25, acc-04-partial]
requires:
  - phase: 02
    plans: [01, 02, 03, 04, 05, 06]
    provides: "All Phase-2 code artefacts (xcfun-core surgical cleanup, xtask gates + registry, xcfun-eval substrate + 11 LDA kernels, tier-2 parity harness with D-24 LDAERF 1e-7 override)"
provides:
  - "docs/design/02-data-structures.md §5 updated for DensVarsDev cubecl type (D-02 + D-04)"
  - "docs/design/05-module-responsibilities.md §2 + new §2.1 + §10 updated for xcfun-eval ownership (D-04)"
  - "docs/design/06-cubecl-strategy.md §3.1 updated confirming Phase-2 #[cube] fn convention landed"
  - "docs/design/09-testing-strategy.md §2 + §3 updated with tier-1 (xcfun-eval/tests/self_tests.rs) + tier-2 (validation/ crate) as landed"
  - ".planning/REQUIREMENTS.md: 30 Phase-2 IDs marked [x] Complete + ACC-04 [~] Partial with documented D-19 residuals"
  - ".planning/ROADMAP.md: Phase 2 marked [x] Complete (with caveats) 2026-04-22; 7-plan listing populated; Progress Table 7/7; ACC-04 footnote explaining Rust > C++ finding at LDAERFX"
  - ".planning/STATE.md: progress 25% (14/14 plans, 2/8 phases); Phase-2 decisions recorded; Recent activity extended; Session Continuity → Phase 3"
  - ".planning/phases/02-core-foundations-lda-tier-parity-harness/02-07-SUMMARY.md (this file) as Phase 2 capstone"
affects:
  - "Phase 3 planning can now begin via /gsd-plan-phase 3 — state + roadmap + requirements consistent"
  - "Phase 6 inherits the D-24 LDAERF 1e-7 override + the mpmath-ground-truth amendment path for LDAERF parity reference"

tech-stack:
  added: []
  patterns:
    - "docs-as-reality: banner-annotated SUPERSEDED + Updated-YYYY-MM-DD markers on revised sections"
    - "capstone SUMMARY with critical-finding documentation (Rust > C++ at LDAERFX verified by mpmath prec=200)"

key-files:
  created:
    - ".planning/phases/02-core-foundations-lda-tier-parity-harness/02-07-SUMMARY.md"
  modified:
    - "docs/design/02-data-structures.md"
    - "docs/design/05-module-responsibilities.md"
    - "docs/design/06-cubecl-strategy.md"
    - "docs/design/09-testing-strategy.md"
    - ".planning/REQUIREMENTS.md"
    - ".planning/ROADMAP.md"
    - ".planning/STATE.md"
  added_to_git:
    - "docs/design/00-overview.md (baseline import from prior planning)"
    - "docs/design/01-source-tree.md"
    - "docs/design/02-data-structures.md"
    - "docs/design/03-api-surface.md"
    - "docs/design/04-control-flow.md"
    - "docs/design/05-module-responsibilities.md"
    - "docs/design/08-error-model.md"
    - "docs/design/09-testing-strategy.md"
    - "docs/design/10-build-and-dependencies.md"
    - "docs/design/11-process-and-milestones.md"
    - "docs/design/README.md"

decisions:
  - "Pragmatic baseline-then-update: committed untracked design docs as a single baseline commit (751eaf7) before applying Phase-2 updates; gives clean diff visibility for each Phase-2 revision"
  - "Per-file atomic commits for each of the 4 updated design docs (D-09 atomicity inherited from Wave 0)"
  - "ACC-04 marked [~] Partial (not [x] Complete) per user-chosen 'sign-off with caveats' policy; documented D-19 INCONCLUSIVE residuals forwarded to Phase 3 + Phase 6"
  - "ROADMAP footnote documents the critical Rust > C++ finding at LDAERFX: at failing point, mpmath prec=200 confirms Rust matches mathematical ground truth while C++ diverges by 6.7% due to its own f64 cancellation. This documents a candidate amendment path: switch parity reference from C++ to mpmath ground truth where C++ is documented to suffer cancellation (to be decided at Phase 6)"
  - "REQUIREMENTS.md traceability table updated to reflect Phase-2 absorption of CORE-10, ACC-05/06, QG-01/02/06/07 (originally Phase 0 per D-10); phase column annotated 'Phase 2 (absorbed from Phase 0 per D-10)'"

requirements-completed:
  marked_complete: [CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, CORE-09, CORE-10, LDA-01, LDA-02, LDA-03, LDA-04, LDA-05, LDA-06, LDA-07, LDA-08, LDA-09, LDA-10, MODE-04, ACC-01, ACC-02, ACC-03, ACC-05, ACC-06, QG-01, QG-02, QG-06, QG-07]
  marked_partial: [ACC-04]

metrics:
  duration: "~25 min (5 atomic edits + baseline import commit + capstone SUMMARY)"
  tasks: 4
  commits: 6
  files_created: 12
  files_modified: 7
  completed: "2026-04-22"
---

# Phase 2 Plan 07: Phase 2 Sign-Off Summary

**Phase 2 (Core Foundations + LDA Tier + Parity Harness) signed off on 2026-04-22 with 30 of 31 requirement IDs complete and ACC-04 partial. The Phase-2 arc — 7 plans, 30+ code commits, 4 design-brief updates, 1 critical Rust > C++ finding at LDAERFX verified by mpmath prec=200 — lands the cubecl-native `xcfun-eval` substrate + 11 LDA kernels + tier-2 parity harness that the rest of the project will extend.**

## Commits (this plan)

| Wave | Commit | Subject |
|------|--------|---------|
| 4-0  | `751eaf7` | docs(02-07): import docs/design/ baseline (untracked from prior planning) |
| 4-1a | `c5dedd6` | docs(02-07): Wave-4-1a update docs/design/02-data-structures.md §5 for DensVarsDev (D-02 + D-04) |
| 4-1b | `3f6d7a9` | docs(02-07): Wave-4-1b update docs/design/05-module-responsibilities.md §2 + §2.1 + §10 for xcfun-eval ownership (D-04) |
| 4-1c | `dd44ad4` | docs(02-07): Wave-4-1c update docs/design/06-cubecl-strategy.md §3.1 confirming Phase 2 #[cube] fn landed |
| 4-1d | `61e59e3` | docs(02-07): Wave-4-1d update docs/design/09-testing-strategy.md tier-1 + tier-2 patterns as landed in Plan 02-04 + 02-06 |
| 4-2  | `4b04745` | docs(02-07): Wave-4-2 mark 30 Phase-2 requirement IDs complete + ACC-04 partial in REQUIREMENTS.md |
| 4-3  | `2b8e4dc` | docs(02-07): Wave-4-3 mark Phase 2 complete (with caveats) in ROADMAP.md; populate 7-plan listing |
| 4-4  | `64ff77e` | docs(02-07): Wave-4-4 STATE.md Phase 2 sign-off — 25% progress, D-24/D-25 + all Phase-2 decisions recorded, Session Continuity → Phase 3 |

A final `docs(02-07): close Phase 2 sign-off` commit lands this SUMMARY file alongside a STATE.md line referencing it.

## What the 4 design-brief updates changed

### 02-data-structures.md §5 (DensVarsDev)
- Replaced the pre-pivot `DensVars<T: Num>` host struct of 29 `T`-typed fields with the as-built `DensVarsDev<F: Float>` `#[derive(CubeType, CubeLaunch)]` type in `crates/xcfun-eval/src/density_vars.rs` (22 `Array<F>` fields)
- Documented `#[cube] fn build_densvars` comptime if-chain (XC_A_B + XC_A_B_GAA_GAB_GBB arms) + `regularize` `#[cube] fn` (CORE-06 + D-22 invariant)
- Cross-referenced `crates/xcfun-eval/src/density_vars/{build,regularize}.rs`
- Marked pre-Phase-2 host-struct design SUPERSEDED

### 05-module-responsibilities.md §2 + §2.1 + §10
- §2 (xcfun-core): narrowed to types + registry tables only (Mode, Vars, Dependency, XcError, FunctionalId, FUNCTIONAL_DESCRIPTORS, VARS_TABLE, ALIASES, taylorlen); deps narrowed to `bitflags` + `thiserror`
- §2.1 (xcfun-eval): NEW section for the cubecl-launcher crate; hosts per-functional `#[cube] fn <name>_kernel` bodies + `DensVarsDev<F>` + `dispatch_kernel` + Functional struct + eval entry
- §10 (boundary rules): rules 2, 3, 10 updated; rule 9 added (anyhow allowlist QG-01)

### 06-cubecl-strategy.md §3.1
- Per-functional `#[cube] fn` body convention LANDED for 11 LDAs (Plans 02-04 + 02-05); Phase 3 extends to 45 GGAs; Phase 4 to 15 metaGGAs
- Updated kernel signature: `&DensVarsDev<F>` + `&mut Array<F>` write-target + `#[comptime] n: u32` per D-17
- ACC-06 `check-no-mul-add` enforcement note

### 09-testing-strategy.md §2 + §3
- Tier-1 relocated from xcfun-core to `crates/xcfun-eval/tests/self_tests.rs` per D-04 boundary
- Tier-2 as-built architecture documented: `validation/` binary crate + cc-compiled xcfun-master LDA .cpp + auto-generated `c_stubs.cpp` (67 stubs) + 10k xoshiro grid + per-functional `threshold_for()` dispatch (D-24 LDAERF 1e-7 override) + two exclusion markers (`excluded_by_upstream_spec` + `excluded_by_regularize_clamp_design`)
- Final tier-2 verdict table: 9/9 GREEN at orders 0/1; order-2 D-19 INCONCLUSIVE residuals forwarded to Phase 3 + Phase 6

## Phase 2 retrospective

### What worked

- **Atomic-commits-per-task (D-09)** — made the Wave-0 surgical cleanup and every subsequent plan safely bisectable. Not a single regression required more than one commit to identify.
- **Wave-2 parallelism (Plans 02-04 + 02-05 after Plan 02-03)** — the dispatcher-first-then-bodies pattern let the kinetic-GGA work proceed independently of the pure-density LDAs.
- **mpmath prec=200 as third-party ground truth** — was the only way to resolve the LDAERFX cancellation question. Without it, the Fix 1 debate would have been Rust-vs-C++ with no tiebreaker.
- **Per-functional threshold dispatch (D-24)** — D-19's "escalate instead of silently widen" rule held up under scrutiny; user-approved override survived the close-out rewrite unchanged.
- **`xtask regen-registry` as a single source of truth** — extractor + `.sha256` stamps made the registry-drift gate (QG-07) self-policing.
- **Cubecl 0.10-pre.3 stayed stable** through all 7 plans, 30+ commits. The hard `=` pin was justified.

### What was harder than expected

- **LDAERFX bracket cancellation** — the single largest investigation of the phase. What looked like a Rust port bug turned out to be an intrinsic f64 cancellation in the upstream C++ algorithm. Took an mpmath prec=200 independent ground truth to diagnose. Resolution required Fix 1 (stable bracket rederivation via `expm1`) + documenting the finding for Phase 6 revisit.
- **Four cascading constants bugs** (commits `e67de81`, `e66af9d`, `5243c2c`, `8611915`) — each looked like a distinct root cause but all traced to identical pattern: `F::new(f32_literal)` truncates to 8 digits; manually-transcribed Python literals had off-by-digit typos. Established the rule that **every LDA constant in `crates/xcfun-eval/src/functionals/lda/**.rs` MUST be an f64 literal via `F::cast_from(f64)` with inline derivation comment**.
- **Cubecl 0.10-pre.3 `Float::erf` polyfill** carried ~1.3e-8 ULP error from its 5-term Wikipedia rational approximation, blocking the 1e-14 parity target. Required the in-kernel libm-port `erf_precise` (FreeBSD msun-derived port) landed in commit `dca382a`.
- **Regularize-clamp boundary effects (VWN/PW/PZ)** — near-clamp precision drift at `min(a,b) ∈ [2e-14, 1e-11]` produces 1–3 ULP residuals above the 1e-12 threshold. Fix 2 (D-22 clamp-stratum exclusion) handles the deliberate-clamp records, but a residual set just above the boundary remains (38 VWN3C, 110 VWN5C, 6 PZ81C records). Phase 3's `build_densvars` redesign may incidentally resolve these.
- **Pre-pivot scaffolding cleanup** (Plan 02-01 Wave 0) was larger than initially scoped — the `density_vars.rs` deletion triggered cascading updates to `traits.rs`, `Cargo.toml`, `lib.rs`. Rule 3 (auto-fix blocking) applied twice.

### What to carry into Phase 3

- **`xtask regen-registry` is the gate.** Phase 3 GGA work MUST re-run `regen-registry` to populate the 45 GGA `FunctionalDescriptor` entries. The extractor already handles nested `#ifdef` macros (Plan 02-02's p86c.cpp handling) so the GGA extension should be one invocation away.
- **`validation/build.rs` cc-compilation must extend to GGA C++ bodies.** Today Plan 02-06 compiles 14 LDA .cpp files + xcint.cpp + xcfunctional.cpp + auto-generated c_stubs.cpp. Phase 3 extends the source list; `c_stubs.cpp` shrinks as real entries move from stub to populated.
- **The `DensVarsDev<F>` field set extends in Phase 3** (add GGA-gradient-only scalars: `gnn`, `gns`, `gss` are already present; Phase 3 may add `n_m13`, `a_43`, `b_43` + `r_s`-derived fields depending on how PBE/Becke port). The `#[derive(CubeType, CubeLaunch)]` layer accepts new fields without breaking the launch ABI — verified by Plan 02-05's `XC_A_B_GAA_GAB_GBB` extension.
- **The D-24 per-functional threshold dispatch is the template** for any Phase 3 range-separated GGA (`beckecamx`, `beckesrx` — both use erf) that hits the same cancellation class as LDAERFX. Plan 02-06's `threshold_for()` function is the integration point.
- **Phase 6's libm-hybrid strategy inherits the LDAERF 1e-7 override** until CUDA/Wgpu provide a second independent ground truth against which the C++ cancellation can be quantified. At that point, a possible amendment will switch the parity reference from C++ to mpmath where C++ is documented to suffer cancellation.
- **The cubecl monomorphization budget has headroom.** 11 LDA `#[cube] fn`s × (f64 only, N in {2,3,4}) × (XC_A_B + XC_A_B_GAA_GAB_GBB variant arms) lowers cleanly on cubecl-cpu. Phase 3 adds 45 more bodies; if compile time balloons, the `#[comptime]` if-chain can be split into per-variant sub-dispatchers.

### Phase 2 architecture notes for future reference

- **Per-functional kernel signature (D-17):** `#[cube] fn <name>_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32)`. Writes into `out` (no return value). Matches Phase 1 `*_expand` fn convention.
- **Compose-only:** kernel bodies compose Phase-1 `xcfun_ad::ctaylor_*` + `xcfun_ad::math::ctaylor_{reciprocal,sqrt,exp,log,pow,powi_*,erf,asinh,atan}` + `xcfun_ad::expand::*`. No host Rust, no `.mul_add(`, no FMA-inducing intrinsics.
- **Shared helpers:** `crates/xcfun-eval/src/functionals/lda/{pw92eps,vwn_eps}.rs` carry the ε-functions that Phase 3 GGA work will likely extend (PBE + BLYP families use the same `pw92eps` in different combinations).
- **Tier-2 exclusion markers are composable:** `excluded_by_upstream_spec` (TW/VWK) and `excluded_by_regularize_clamp_design` (D-22) stack — a single record can carry both markers; CellSummary treats them as a boolean OR when computing the non-excluded verdict.

## Phase 3 prerequisites (summary for the next plan-phase invocation)

Plan Phase 3 (`/gsd-plan-phase 3`) should assume:

1. **`xcfun-eval::Functional` is the entry point.** Phase 3 extends it (not replaces it) with the GGA `eval_setup(Vars, Mode, order)` variants that carry gradient dependencies.
2. **`dispatch_kernel` comptime if-chain is the extension surface.** 11 arms today (one per LDA FunctionalId); Phase 3 adds 45 GGA arms. No structural change needed.
3. **`DensVarsDev<F>` field set may extend** — field additions are non-breaking per cubecl 0.10-pre.3 `#[derive(CubeLaunch)]`.
4. **`xtask regen-registry` extension** — rerun to populate the 45 GGA descriptors; existing 67 stubs shrink to 22 (metaGGA + alias infrastructure).
5. **`validation/build.rs`** — extend cc source list to include `xcfun-master/src/functionals/gga/*.cpp`; expected compile-time increase ~20s.
6. **MODE-01 orders 3..=4** land with Phase 3 (Phase 2 shipped 0..=2 per D-23 + SC #5).
7. **Mode::Potential (MODE-02)** via `CTaylor<f64, 2>` divergence construction — Phase 3 deliverable.
8. **Re-run tier-2 for VWN3C/VWN5C/PW92C/PZ81C order-2** after Phase-3 `build_densvars` redesign; the near-clamp precision drift flagged in ACC-04 may resolve incidentally.
9. **Range-separated GGA tolerance check** — `beckecamx` and `beckesrx` will likely hit the same erf-cancellation class as LDAERFX. The D-24 threshold_for() framework accepts per-functional overrides; anticipate escalation.

## Self-Check: PASSED

- [x] File `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-07-SUMMARY.md` exists (this file)
- [x] Commit `751eaf7` (design-docs baseline import) in git log
- [x] Commit `c5dedd6` (Wave-4-1a docs/design/02) in git log
- [x] Commit `3f6d7a9` (Wave-4-1b docs/design/05) in git log
- [x] Commit `dd44ad4` (Wave-4-1c docs/design/06) in git log
- [x] Commit `61e59e3` (Wave-4-1d docs/design/09) in git log
- [x] Commit `4b04745` (Wave-4-2 REQUIREMENTS.md) in git log
- [x] Commit `2b8e4dc` (Wave-4-3 ROADMAP.md) in git log
- [x] Commit `64ff77e` (Wave-4-4 STATE.md) in git log
- [x] `cargo build --workspace` → PASS (0.25s; no code changes)
- [x] 30 Phase-2 IDs marked [x] + ACC-04 [~] in REQUIREMENTS.md
- [x] Phase 2 marked [x] Complete (with caveats) in ROADMAP.md; Progress Table 7/7
- [x] STATE.md progress 25%, session continuity → Phase 3
- [x] All 4 design-doc updates reference the as-built source files (crates/xcfun-eval/...)

---

*Phase 2 signed off: 2026-04-22*
*Next: /gsd-plan-phase 3 (GGA Tier + Mode::Potential)*
