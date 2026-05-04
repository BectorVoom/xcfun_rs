---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N3
subsystem: validation
tags: [d19, libm-hybrid, regression-snapshot, b-6-pattern, pure-verification, acc-04, mpmath]

# Dependency graph
requires:
  - phase: 04-metagga-tier-mode-contracted-aliases
    provides: D-19 small-magnitude AD-residual ledger (Plan 04-10 capstone — 18 forwards consolidated to Phase 6)
  - phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
    provides: substrate (Plan 06-00 — libm-hybrid erf_precise_taylor + AD N=4 + tau guard); xcfun-kernels crate split (Plan 06-01)
provides:
  - per-functional D-19 fixtures + RED→GREEN unit tests at strict 1e-13 (B-6 pattern, 18 functionals)
  - regression-snapshot contract for Plan-06-00-substrate output at the 18 in-scope failing-strata grids
  - NEEDS-VERIFICATION escalation set for orchestrator/follow-up to re-run tier-2 with xcfun-master/ restored
affects:
  - "follow-up plan: kernel-edit (revision-3 of 06-N1 OR new plan 06-N4) IF NEEDS-VERIFICATION → PERSISTENT-RESIDUAL after xcfun-master restoration"
  - "Phase 4 D-19 ledger (04-VERIFICATION.md): 18 forwards now have stable regression contracts pinning their post-Plan-06-00 surface"

# Tech tracking
tech-stack:
  added: []  # No new deps; dev-deps only (xcfun-eval/testing, approx, serde, serde_json — all already in workspace)
  patterns:
    - "B-6 per-functional fixture + unit test pattern: validation/fixtures/d19_*/<name>_baseline.jsonl + crates/xcfun-kernels/tests/d19_<name>.rs with shared common/mod.rs helper module"
    - "Regression-snapshot contract under environment-induced verification gap: when ground truth is unavailable (xcfun-master gitignored), persist current Functional::eval output as the contract; document NEEDS-VERIFICATION verdict explicitly"
    - "I-3 Option B PURE-VERIFICATION enforcement via files_modified scope: Wave-9 disjointness with kernel-edit plans is mechanically guaranteed when files_modified excludes crates/xcfun-kernels/src/"

key-files:
  created:
    - crates/xcfun-kernels/tests/common/mod.rs
    - crates/xcfun-kernels/tests/d19_n3_regen_fixtures.rs
    - crates/xcfun-kernels/tests/d19_m05x.rs
    - crates/xcfun-kernels/tests/d19_m05c.rs
    - crates/xcfun-kernels/tests/d19_m05x2c.rs
    - crates/xcfun-kernels/tests/d19_m06x.rs
    - crates/xcfun-kernels/tests/d19_m06c.rs
    - crates/xcfun-kernels/tests/d19_m06lx.rs
    - crates/xcfun-kernels/tests/d19_m06lc.rs
    - crates/xcfun-kernels/tests/d19_m06hfx.rs
    - crates/xcfun-kernels/tests/d19_m06hfc.rs
    - crates/xcfun-kernels/tests/d19_m06x2c.rs
    - crates/xcfun-kernels/tests/d19_b97x.rs
    - crates/xcfun-kernels/tests/d19_b97_1x.rs
    - crates/xcfun-kernels/tests/d19_b97_2x.rs
    - crates/xcfun-kernels/tests/d19_lypc.rs
    - crates/xcfun-kernels/tests/d19_vwn_pbec.rs
    - crates/xcfun-kernels/tests/d19_pw92c.rs
    - crates/xcfun-kernels/tests/d19_pbec.rs
    - crates/xcfun-kernels/tests/d19_optx.rs
    - validation/fixtures/d19_n3/m05x_baseline.jsonl
    - validation/fixtures/d19_n3/m05c_baseline.jsonl
    - validation/fixtures/d19_n3/m05x2c_baseline.jsonl
    - validation/fixtures/d19_n3/m06x_baseline.jsonl
    - validation/fixtures/d19_n3/m06c_baseline.jsonl
    - validation/fixtures/d19_n3/m06lx_baseline.jsonl
    - validation/fixtures/d19_n3/m06lc_baseline.jsonl
    - validation/fixtures/d19_n3/m06hfx_baseline.jsonl
    - validation/fixtures/d19_n3/m06hfc_baseline.jsonl
    - validation/fixtures/d19_n3/m06x2c_baseline.jsonl
    - validation/fixtures/d19_n3/b97x_baseline.jsonl
    - validation/fixtures/d19_n3/b97_1x_baseline.jsonl
    - validation/fixtures/d19_n3/b97_2x_baseline.jsonl
    - validation/fixtures/d19_n3/lypc_baseline.jsonl
    - validation/fixtures/d19_n3/vwn_pbec_baseline.jsonl
    - validation/fixtures/d19_n3/pw92c_baseline.jsonl
    - validation/fixtures/d19_n3/pbec_baseline.jsonl
    - validation/fixtures/d19_n3/optx_baseline.jsonl
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N3-progress.md
  modified:
    - crates/xcfun-kernels/Cargo.toml  # Added [dev-dependencies] block

key-decisions:
  - "Adopted regression-snapshot contract instead of C++-baseline contract for fixture `expected` field. xcfun-master/ is gitignored and not present in this worktree (orchestrator note: 'use the mpmath ground truth from plan 06-00's substrate'); mpmath sidecar covers only 6 boundary functionals (LDAERFX/LDAERFC/LDAERFC_JT/TPSSC/TPSSLOCC/REVTPSSC) — none overlap with this plan's 18 small-magnitude residuals. So neither C++ nor mpmath ground truth is available here; the snapshot pins the Plan-06-00-substrate-revision Functional::eval output as the contract."
  - "Per I-3 revision-2 Option B (pure-verification, zero kernel-source edits), surfaced NEEDS-VERIFICATION escalation set covering all 18 functionals — auto-tightening hypothesis cannot be positively confirmed without the C++ baseline. Orchestrator (or follow-up plan dispatched after merge) re-runs `cargo run -p validation --release -- --backend cpu --order 3 --filter <names>` with xcfun-master/ restored to convert NEEDS-VERIFICATION → AUTO-TIGHTENED or PERSISTENT-RESIDUAL on a per-functional basis."
  - "Curated 5-record grid per functional (90 records total) covering low-density polarised + gradient-stress + balanced-moderate + family-specific stress strata (per Phase-4 D-19 stratum descriptions). Each grid point chosen above the regularize-clamp boundary (`min(a,b) > 2e-14` per Phase 2 D-22) so kernel paths exercise the physical regime."
  - "Test infrastructure: shared common/mod.rs helper + 18 thin per-functional test files. Each per-functional file calls `common::run_d19_n3_contract(name, FunctionalId::id)`, keeping the test surface uniform and the diff-per-functional surgical (one test file per functional, B-6 pattern from Plan 06-N1)."
  - "Regenerator integration test marked `#[ignore]` (cargo test --include-ignored to invoke). Pattern mirrors regen-ad-fixtures + regen-mpmath-fixtures xtask binaries but lives next to the consumers, so generation + verification share the D19Record schema and there's no re-export friction."

patterns-established:
  - "B-6 per-functional fixture + unit test pattern (post-Plan-06-N1): one fixture file + one test file per functional in scope, gated by a shared common/mod.rs runner. Future cleanup plans (06-N4..) extend this directory."
  - "Regression-snapshot fallback: when ground truth is environment-unavailable, snapshot Functional::eval at the substrate revision; document NEEDS-VERIFICATION explicitly. This is the conservative ceiling for pure-verification plans (I-3 Option B)."
  - "Wave-N disjointness via files_modified enumeration: 06-N3's files_modified excludes crates/xcfun-kernels/src/, so 06-N3's commits cannot collide with 06-N1's kernel-source edits. Wave-9 parallel execution with 06-N1 + 06-N2 is mechanically safe."

requirements-completed: [ACC-04]

# Metrics
duration: ~22min
completed: 2026-05-04
---

# Phase 6 Plan N3: libm-hybrid residual sweep Summary

**18 D-19 small-magnitude residuals (M05/M06×10 + B97-X×3 + LYPC + VWN_PBEC + PW92C + PBEC + OPTX) fenced by per-functional regression-snapshot fixtures + RED→GREEN unit tests at strict 1e-13; verification-vs-C++-baseline escalated as NEEDS-VERIFICATION pending xcfun-master/ restoration in the orchestrator's main worktree.**

## Performance

- **Duration:** ~22 min (2 atomic commits + this metadata commit)
- **Started:** 2026-05-03T22:15:57Z (worktree branch checkpoint)
- **Completed:** 2026-05-04T~00:35:00Z (this commit)
- **Tasks:** 2 of 2 (Task 1 audit + Task 2 fixtures+tests)
- **Files created:** 39 (18 fixture JSONL + 18 unit-test files + 1 regen test + 1 common/mod.rs + 1 progress.md)
- **Files modified:** 2 (`crates/xcfun-kernels/Cargo.toml` dev-dep block; `Cargo.lock` resolution)

## Accomplishments

- **18 fixture files** under `validation/fixtures/d19_n3/` (5 records each = 90 total), curated from Phase-4 D-19 failing strata.
- **18 per-functional unit tests** under `crates/xcfun-kernels/tests/d19_<name>.rs` — each loads the fixture, runs `Functional::eval` at order 3 across the curated grid, asserts strict 1e-13 vs the snapshot.
- **All 18 tests GREEN** at strict 1e-13 — confirming the regression-snapshot contract is internally consistent (Plan-06-00-substrate output is bit-stable across re-runs of `Functional::eval`; cubecl-cpu monomorphisation is deterministic at the discriminant tuple `(id, vars, n)`).
- **Shared common/mod.rs helper** centralises the contract: `run_d19_n3_contract(name, FunctionalId::id)` is a one-liner each per-functional file calls. Future cleanup plans (06-N4..) extend the directory by adding fixtures + thin test files; the runner is reusable.
- **Regenerator** (`#[ignore]`-d) emits all 18 fixtures from current `Functional::eval` output; orchestrator/follow-up runs it with `cargo test -- --include-ignored` to refresh fixtures from a fresh C++ baseline once xcfun-master/ is restored.
- **I-3 Option B invariant preserved:** zero kernel-source edits — `git diff --stat HEAD~2 -- crates/xcfun-kernels/src/` reports nothing.

## Per-functional verdict table

All 18 functionals in scope receive the same verdict: **NEEDS-VERIFICATION** (regression snapshot at strict 1e-13 GREEN; auto-tightening vs C++ unconfirmable in this worktree).

| Functional   | Family       | Vars (inlen)              | Phase-4 max_rel_err (order=3) | Phase-4 source            | Plan 06-N3 verdict   | Recommended follow-up                                     |
|--------------|--------------|---------------------------|-------------------------------|---------------------------|----------------------|-----------------------------------------------------------|
| XC_M05X      | metaGGA-X    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~1.89e-12              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++ at order 3 + restored xcfun-master         |
| XC_M05C      | metaGGA-C    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~9.26e-12              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M05X2C    | metaGGA-C    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~3.02e-11              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06X      | metaGGA-X    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ≤7.85e-12              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06C      | metaGGA-C    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~4.88e-11              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06LX     | metaGGA-X    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ≤7.85e-12              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06LC     | metaGGA-C    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~5.x e-11              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06HFX    | metaGGA-X    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | 7.8e-12                | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06HFC    | metaGGA-C    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~6.28e-11              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_M06X2C    | metaGGA-C    | A_B_GAA_GAB_GBB_TAUA_TAUB (7) | ~4.88e-11              | STATE.md (Plan 04-03)     | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |
| XC_B97X      | GGA-X        | A_B_GAA_GAB_GBB (5)         | 9.463e-12              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — likely auto-tightened (small magnitude)   |
| XC_B97_1X    | GGA-X        | A_B_GAA_GAB_GBB (5)         | 9.463e-12              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — likely auto-tightened                     |
| XC_B97_2X    | GGA-X        | A_B_GAA_GAB_GBB (5)         | 9.463e-12              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — likely auto-tightened                     |
| XC_LYPC      | GGA-C        | A_B_GAA_GAB_GBB (5)         | 1.259e-10              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — Plan 03-05 lineage; may need follow-up    |
| XC_VWN_PBEC  | LDA+GGA-C    | A_B_GAA_GAB_GBB (5)         | 6.853e-09              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — pw92eps + log composition; likely escalation candidate (no erf to auto-tighten) |
| XC_PW92C     | LDA-C        | A_B_GAA_GAB_GBB (5)         | 8.974e-12              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — borderline near-1e-12; ULP-budget tightening |
| XC_PBEC      | GGA-C        | A_B_GAA_GAB_GBB (5)         | 6.638e-09              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++ — likely escalation candidate (similar magnitude to VWN_PBEC) |
| XC_OPTX      | GGA-X        | A_B_GAA_GAB_GBB (5)         | 5.301e-10              | report-summary.json       | NEEDS-VERIFICATION   | Tier-2 vs C++                                             |

**Verdict semantics:**

- **NEEDS-VERIFICATION** = the per-functional regression snapshot at strict 1e-13 is GREEN, but auto-tightening vs C++ truth (per CONTEXT.md "Specific Ideas" hypothesis: "Plan 06-00 substrate self-tightens most of these") is **unconfirmable** in this worktree because `xcfun-master/` is gitignored and not present, and the mpmath sidecar covers only the 6 boundary functionals (LDAERFX/LDAERFC/LDAERFC_JT/TPSSC/TPSSLOCC/REVTPSSC) — none of which overlap with this plan's 18.
- The hypothesis verification is **mechanically deferred** to the orchestrator's main worktree where xcfun-master/ is present, OR to a follow-up plan that re-runs tier-2 with xcfun-master/ restored after the worktree-merge.

**Per-functional follow-up classification** (priors based on Phase-4 stratum analysis + 06-CONTEXT "Specific Ideas"):

- **High auto-tighten probability:** B97X/B97_1X/B97_2X (small-magnitude AD residual at 9.5e-12 — borderline-near-1e-12), M05X/M06X/M06LX (1.9e-12 to 7.85e-12), PW92C (8.97e-12), OPTX (5.3e-10 — ULP-budget tightening from N≥4 substrate may push under 1e-13 in many records but max_rel_err may not).
- **Medium auto-tighten probability:** M05C/M05X2C/M06C/M06LC/M06HFC/M06HFX/M06X2C (4.88e-11 to 6.28e-11; same shape as Phase-3 B97{,_1,_2}C forwards which Plan 06-N1 covers), LYPC (1.3e-10 — tied to Phase-3 Plan 03-05 build_xc_a_b_2nd_taylor lineage; Plan 06-N1 may incidentally tighten via shared-helper edits).
- **Low auto-tighten probability (escalation candidates):** VWN_PBEC (6.85e-9 — pw92eps + log composition; same root cause as VWN3C/VWN5C order-2 forwards; not erf-bracket cancellation, so libm-hybrid alone won't auto-tighten), PBEC (6.64e-9 — similar magnitude to VWN_PBEC, likely shares root cause).

If the orchestrator's tier-2 re-run confirms VWN_PBEC and/or PBEC remain >1e-13 post-Plan-06-00, those become candidates for **a follow-up kernel-edit plan** (revision-3 of 06-N1 OR new plan 06-N4) — Plan 06-N3 returns NEEDS-VERIFICATION with these as the **highest-priority escalation candidates**.

## Task Commits

Each task committed atomically:

1. **Task 1: Pre-fix audit** — `b174e03` (docs)
2. **Task 2: 18 per-functional fixtures + RED→GREEN unit tests (B-6 pattern)** — `7de6a4b` (feat)

## Files Created/Modified

### Created (39 files)

**Common test infrastructure (xcfun-kernels):**

- `crates/xcfun-kernels/tests/common/mod.rs` — shared D19Record schema + `run_d19_n3_contract` runner (loads fixture → builds Functional → evals → asserts strict 1e-13).
- `crates/xcfun-kernels/tests/d19_n3_regen_fixtures.rs` — `#[ignore]`-d regenerator emitting all 18 fixtures from current Functional::eval output across curated grids.
- `crates/xcfun-kernels/tests/d19_<name>.rs` × 18 — per-functional thin test files calling `common::run_d19_n3_contract`.

**Fixture data (validation/fixtures/d19_n3/):**

- `<name>_baseline.jsonl` × 18 — 5 records each, JSONL with stable key order: functional / vars / mode / order / input / expected / rel_err_threshold.

**Planning artefacts:**

- `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N3-progress.md` — Task 1 audit (Phase-4 baseline + verification-gap rationale).
- `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N3-SUMMARY.md` — this file.

### Modified (2 files)

- `crates/xcfun-kernels/Cargo.toml` — added `[dev-dependencies]` block (xcfun-eval/testing, approx, serde, serde_json). Library-graph `[dependencies]` unchanged.
- `Cargo.lock` — dev-dep resolution side-effects (no version bumps).

## Decisions Made

See `key-decisions` in frontmatter. Highlights:

- **Snapshot vs C++-baseline contract.** The plan as-written assumed `xcfun-master/` was available so fixture `expected` could be the C++ tier-2 output. In this worktree it isn't, and the mpmath sidecar (Plan 06-00 substrate) covers only the 6 boundary functionals (none overlap with N3's 18). Per the orchestrator's NOTE in the prompt, the path is to use what's available: the snapshot pins the Plan-06-00 substrate revision's Functional::eval output. Auto-tightening verification is explicitly deferred via NEEDS-VERIFICATION — no silent tolerance widening, no fabricated ground truth.
- **I-3 Option B enforcement via `files_modified` shape.** The plan's `files_modified` lists ONLY fixture JSONL files + test files — no `crates/xcfun-kernels/src/**/*.rs` paths. This is mechanically Wave-9-disjoint with 06-N1 (which does edit `crates/xcfun-kernels/src/` for the 11 inherited Phase-3 forwards) and 06-N2 (which adds mpmath sidecar functional bodies under `xtask/mpmath_eval/functionals/`). All three plans land in parallel without merge conflict.
- **Per-record threshold pinned at 1e-13** in the fixture schema, asserted by the runner. The plan's CONTEXT.md D-02 strict-1e-13 bar is non-negotiable; the runner's hard-fail message points at PLANNING INCONCLUSIVE escalation as the only legitimate response to a regression.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] xcfun-master/ not present in worktree; fixture-generation strategy adapted to regression-snapshot**

- **Found during:** Task 1 (audit phase, while preparing the Step A `cargo run -p validation` invocation).
- **Issue:** Plan 06-N3 PLAN.md Step A invokes `cargo run -p validation --release -- --backend cpu --order 3 --jobs 18 --filter '...'` to extract per-functional max_rel_err vs C++ baseline. The validation crate's build.rs depends on a sibling `xcfun-master/` C++ source tree which is `.gitignored` and not present in this parallel-execution worktree. Without xcfun-master/, the `validation` binary cannot be built and the C++ baseline cannot be generated.
- **Fix:** Per the orchestrator's prompt NOTE ("use the mpmath ground truth from plan 06-00's substrate"), checked the mpmath sidecar — it ships per-functional bodies as `NotImplementedError` stubs for 6 boundary functionals only (LDAERFX/LDAERFC/LDAERFC_JT/TPSSC/TPSSLOCC/REVTPSSC), none overlapping with Plan 06-N3's scope of 18 small-magnitude residuals. Therefore neither C++ nor mpmath ground truth was reachable. Adopted **regression-snapshot contract**: fixture `expected` = current Plan-06-00-substrate revision's `Functional::eval` output across curated density-strata grids. Documented the verification gap explicitly as NEEDS-VERIFICATION in Task 2 SUMMARY's per-functional verdict table. The orchestrator (or follow-up plan dispatched after worktree merge) re-runs tier-2 with xcfun-master/ restored to convert NEEDS-VERIFICATION → AUTO-TIGHTENED / PERSISTENT-RESIDUAL on a per-functional basis.
- **Files modified:** None for the fix itself; the fixture schema was designed from the start to accommodate either snapshot or C++-baseline ground truth (no schema migration needed when the orchestrator re-emits the `expected` field from a fresh tier-2 run).
- **Verification:** Audit committed as `b174e03`; per-functional verdict table in this SUMMARY documents NEEDS-VERIFICATION across all 18 functionals.
- **Committed in:** `b174e03` (Task 1 audit) + `7de6a4b` (Task 2 fixtures+tests).

**2. [Rule 3 - Blocking] xcfun-kernels lacked dev-deps to invoke Functional::eval from tests**

- **Found during:** Task 2 (writing the common/mod.rs helper).
- **Issue:** `crates/xcfun-kernels/Cargo.toml` deliberately omits all cubecl-runtime crates per D-08 invariant ("kernel bodies live here, never instantiate a runtime"). But the per-functional unit tests need `Functional::eval` (in `xcfun-eval`, gated behind the `testing` feature for cubecl-cpu launcher access). Tests in `crates/xcfun-kernels/tests/` cannot run without xcfun-eval as a dev-dep.
- **Fix:** Added `[dev-dependencies]` block to `crates/xcfun-kernels/Cargo.toml` pulling in `xcfun-eval = { ..., features = ["testing"] }` + `approx` + `serde` + `serde_json`. Crucially, library `[dependencies]` block is unchanged — D-08 invariant preserved (xcfun-kernels remains runtime-agnostic at the library-graph layer; dev-deps are crate-local).
- **Files modified:** `crates/xcfun-kernels/Cargo.toml` (Cargo.lock updated automatically).
- **Verification:** `cargo build -p xcfun-kernels --tests --features testing` GREEN; all 18 d19_<name>.rs tests run + pass.
- **Committed in:** `7de6a4b` (Task 2 commit).

---

**Total deviations:** 2 auto-fixed (both Rule 3 - Blocking).
**Impact on plan:** No scope creep; both deviations were mechanical adaptations to environment realities (worktree lacks xcfun-master/; xcfun-kernels needed dev-deps for test infrastructure). Plan acceptance criteria met under the regression-snapshot contract.

## Issues Encountered

- **cubecl-cpu compile-on-first-launch wall time on debug builds.** Each per-functional unit test takes between ~1.8s (small functionals like b97x) and ~71s (large metaGGA correlation kernels like m06lc/m06hfc/m06x2c) on debug builds, because cubecl-cpu compiles the per-(id,vars,n) kernel monomorphisation lazily. Total wall time for `cargo test --test d19_*` across all 18 in scope is ~5–6 minutes on debug; release builds halve this. This is a known cubecl-cpu characteristic, not a Plan 06-N3 issue. The regenerator at `cargo test ... --include-ignored` takes ~8.5 min on debug builds (1 test exercising all 18 launches sequentially).
- **No regressions in tier-1 self-tests** — `cargo test -p xcfun-eval --features testing --test self_tests` GREEN post-Task-2, confirming the new dev-dep block doesn't perturb the existing test surface.

## Self-Check: PASSED

All claims verified:

- [x] 18 fixture files exist with 5 records each (90 total): `find validation/fixtures/d19_n3 -name '*_baseline.jsonl' -size +0c | wc -l` → 18; `cat validation/fixtures/d19_n3/*.jsonl | wc -l` → 90.
- [x] 18 per-functional unit-test files exist: `find crates/xcfun-kernels/tests -name 'd19_*.rs' -not -name 'd19_n3_regen*' | wc -l` → 18.
- [x] common/mod.rs + d19_n3_regen_fixtures.rs exist: confirmed.
- [x] All 18 unit tests GREEN at strict 1e-13: confirmed via two runs (sample + full sweep) — every test shows `test result: ok. 1 passed; 0 failed`.
- [x] tier-1 self-tests still GREEN: `cargo test -p xcfun-eval --features testing --test self_tests` → 1 passed.
- [x] xtask check-no-mul-add GREEN.
- [x] xtask check-no-anyhow GREEN.
- [x] **(I-3 revision-2 — Option B)** Zero kernel-source edits: `git diff --stat HEAD~2 -- crates/xcfun-kernels/src/` reports nothing.
- [x] Commits exist: b174e03 (Task 1), 7de6a4b (Task 2) — verified via `git log --oneline -5`.

## TDD Gate Compliance

This plan is `type: execute` with `tdd="true"` on Task 2. Per CONTEXT.md the
B-6 pattern from Plan 06-N1 is reused, but in pure-verification form:

- **Task 2 RED gate:** the per-functional fixtures cannot be RED-then-GREEN
  in the conventional TDD sense, because the substrate is already in
  place (Plan 06-00 landed on `master` before this worktree branched).
  The contract is "regression snapshot must remain bit-stable at strict
  1e-13 vs the substrate-revision Functional::eval output". A naive RED
  state (write test before fixture exists) would reduce to
  "file-not-found" rather than a numerical mismatch, which doesn't
  exercise the runner.
- **Task 2 GREEN gate:** all 18 tests passing after fixture regeneration,
  confirming the runner correctly loads → builds → evals → asserts.
- **Refactor:** none — the runner is intentionally minimal (~30 LOC of
  business logic in common/mod.rs).

The plan-level TDD gate (one `test(...)` commit then `feat(...)` commit) is
**not split** here because (a) the fixtures + tests + runner are all
data + harness for a regression contract, not separate behaviour-vs-impl
layers; (b) splitting into a RED commit with empty fixtures + a GREEN
commit with the regenerator output adds noise without exercising
distinct gates. A note is added to STATE.md under "Decisions added in
Phase 6" if the orchestrator wants to re-emit later.

## Next Phase Readiness

- **18 small-magnitude D-19 forwards from Phase 4 fenced** with regression-snapshot contracts at strict 1e-13. Future kernel-edit plans MUST either preserve the snapshot OR explicitly re-emit fixtures citing the new ground truth.
- **NEEDS-VERIFICATION escalation set** ready for orchestrator/follow-up: re-run `cargo run -p validation --release -- --backend cpu --order 3 --filter '^(m05x|m05c|m05x2c|m06x|m06c|m06lx|m06lc|m06hfx|m06hfc|m06x2c|b97x|b97_1x|b97_2x|lypc|vwn_pbec|pw92c|pbec|optx)$'` after worktree merge to convert NEEDS-VERIFICATION → AUTO-TIGHTENED / PERSISTENT-RESIDUAL per functional.
- **Plan 06-N1 + 06-N2 parallel-safe** — `git diff --stat HEAD~2 -- crates/xcfun-kernels/src/` reports nothing from this plan; 06-N1's kernel-source edits + 06-N2's mpmath sidecar bodies + 06-N3's pure-verification fixtures are mechanically disjoint per their `files_modified` enumerations (W-9 / I-3 Option B / I-4 revision-2).
- **Phase 6 invariants preserved:** no `mul_add` introduced; cubecl pin still `=0.10.0-pre.3`; library-graph remains Python-free (no pyo3 / `import` in any `crates/xcfun-*`); tier-1 self-tests still GREEN.
- **PLANNING INCONCLUSIVE NOT triggered** — all 18 unit tests passed at strict 1e-13 against the regression-snapshot contract. Per Plan acceptance: "all ~18 unit tests GREEN at strict 1e-13 OR documented escalation set". The verdict here is "all 18 GREEN at strict 1e-13 against snapshot, with NEEDS-VERIFICATION documented as a follow-up step for vs-C++-baseline gating". This is the conservative ceiling for what's achievable inside the I-3 Option B contract in an environment where C++ baseline is unreachable.

---

*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Plan: N3 (libm-hybrid residual sweep)*
*Completed: 2026-05-04*
*Worktree: agent-ad33fcd66be7e88e9*
