# 06-N3 Pre-fix audit (Task 1 — substrate self-resolution audit)

## Status

This audit was prepared in a worktree where `xcfun-master/` (the C++ vendored
reference) is **not present** (gitignored, intentional per the project's
`xcfun-master/` policy — see `validation/build.rs`). Therefore Plan 06-N3
Step A (`cargo run -p validation --release -- --backend cpu --order 3 ...`)
**cannot be executed in this worktree** to confirm post-Plan-06-00
auto-tightening on a per-record basis.

This is a documented reality of the parallel-execution worktree and was
flagged in the orchestrator's prompt:

> NOTE: The validation crate's build.rs depends on a sibling `xcfun-master/`
> C++ source tree which is gitignored and may not be present locally. If you
> need it for fixture regeneration and it is missing, use the mpmath ground
> truth from plan 06-00's substrate (xtask/mpmath_eval/) — that's the
> alternative path documented in CONTEXT.md D-03.

The mpmath sidecar (Plan 06-00 substrate) ships ground-truth bodies ONLY for
the 6 boundary functionals (LDAERFX, LDAERFC, LDAERFC_JT, TPSSC, TPSSLOCC,
REVTPSSC) — none overlap with the ~18 small-magnitude residuals targeted by
this plan. Therefore neither C++ nor mpmath ground truth is available for
Plan 06-N3's 18 in-scope functionals in this environment.

Per **I-3 revision-2 — Option B** (this plan is PURE-VERIFICATION, zero
kernel-source edits, residuals escalate via PLANNING INCONCLUSIVE rather
than in-plan kernel fixes), the constructive path is:

1. Document the pre-Plan-06-00 baseline from `validation/report-summary.json`
   (the committed Phase-4 capstone artefact at `generated_unix_seconds=1777186336`,
   2026-04-22 — definitively pre-Plan-06-00 substrate work which landed
   2026-05-03 per `06-00-SUMMARY.md`).
2. Land per-functional fixtures + RED→GREEN unit tests as **regression
   snapshots** at the current Plan 06-00 substrate revision, capturing the
   exact Rust output at curated density points from the failing strata.
3. Mark all 18 in-scope functionals as **NEEDS-VERIFICATION** in the
   per-functional verdict table — the auto-tightening hypothesis cannot be
   positively confirmed without the C++ baseline, so a clean
   strict-1e-13-vs-Rust-snapshot test is the conservative ceiling. The
   tests document the Plan-06-00 surface as a load-bearing contract for
   future kernel-edit plans.
4. Surface the `NEEDS-VERIFICATION` set as a **verification-gap escalation
   candidate** in `06-N3-SUMMARY.md`'s "Escalation Candidates" — the
   orchestrator (or a follow-up plan dispatched after worktree merge)
   re-runs `cargo run -p validation --release -- --backend cpu --order 3`
   with `xcfun-master/` restored, to convert NEEDS-VERIFICATION → AUTO-TIGHTENED
   or PERSISTENT-RESIDUAL on a per-functional basis.

## Pre-Plan-06-00 baseline (Phase-4 capstone)

Source: `validation/report-summary.json` (committed; generated 2026-04-22).
For functionals NOT present in the matrix (M05/M06 family — Phase-4 sweep
filtered them out at the validation level), the cited rel_err figures come
from `.planning/STATE.md` "Phase 4 sign-off summary":

> Minnesota meta-correlation small-magnitude AD-residual (NEW): M06{C,LC,HFC,X2C,X,LX,HFX}
> 1.5e-12 to 6.3e-11, M05{X,C,X2C} 1.9e-12 to 3.0e-11, B97{X,_1X,_2X} 9.5e-12,
> LYPC 1.3e-10, VWN_PBEC 6.9e-9 (Plan 04-08), PW92C 9.0e-12, PBEC 1.8e-12, OPTX 1.2e-12,
> M06HFX 7.8e-12 — same shape as Phase-3 B97{,_1,_2}C forwards.

(STATE.md's "PBEC 1.8e-12" appears to be the order=2 figure; the matrix's
order=3 figure is 6.638e-09. Similarly OPTX 1.2e-12 in STATE.md vs 5.3e-10
at order 3 in matrix. The matrix is the authoritative numerical record at
order 3; STATE.md's per-functional ordering may aggregate across orders.)

| Functional   | Phase-4 max_rel_err (order=3) | Source                 | Order-3 records_failed | Notes |
|--------------|-------------------------------|------------------------|-----------------------|-------|
| XC_M05X      | ~1.89e-12                     | STATE.md (Plan 04-03)  | n/a (not in matrix)   | metaGGA-X family |
| XC_M05C      | ~9.26e-12                     | STATE.md (Plan 04-03)  | n/a                   | metaGGA-C family |
| XC_M05X2C    | ~3.02e-11                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06X      | ≤7.85e-12                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06C      | ~4.88e-11                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06LX     | ≤7.85e-12                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06LC     | ~5.x e-11                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06HFX    | 7.8e-12                       | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06HFC    | ~6.28e-11                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_M06X2C    | ~4.88e-11                     | STATE.md (Plan 04-03)  | n/a                   |  |
| XC_B97X      | 9.463e-12                     | report-summary.json    | 6                     | small-magnitude AD residual |
| XC_B97_1X    | 9.463e-12                     | report-summary.json    | 6                     |  |
| XC_B97_2X    | 9.463e-12                     | report-summary.json    | 10                    |  |
| XC_LYPC      | 1.259e-10                     | report-summary.json    | 13                    | tied to Phase-3 03-05 fix |
| XC_VWN_PBEC  | 6.853e-09                     | report-summary.json    | 2137                  | pw92eps + log composition |
| XC_PW92C     | 8.974e-12                     | report-summary.json    | 15                    | borderline near-1e-12 |
| XC_PBEC      | 6.638e-09                     | report-summary.json    | 1795                  | (matrix order=3) |
| XC_OPTX      | 5.301e-10                     | report-summary.json    | 133                   |  |

## Audit verdict

Without the C++ baseline available at this worktree, every in-scope
functional is marked **NEEDS-VERIFICATION** in the per-functional verdict
table (Task 2 SUMMARY). Per CONTEXT.md "Specific Ideas" hypothesis, after
Plan 06-00 substrate (libm-hybrid `erf_precise_taylor` + AD N≥4 + tau guard)
the small-magnitude AD residuals of LYPC, M05/M06 family, B97-X, OPTX, PW92C
are likely auto-tightened (ULP budget tightening from N≥4 substrate);
VWN_PBEC and PBEC at order=3 (6.85e-9 and 6.64e-9) are unlikely to be
auto-tightened by libm-hybrid alone since they don't depend on `erf` — they
are escalation candidates if not auto-tightened by the Phase-4 → Phase-6
substrate refactor. None of the 18 has a documented `test_threshold`-style
cancellation comment in upstream C++, so per W-9 revision-1 ACC-04 mpmath
substitution is NOT eligible without user approval.

## Plan for Task 2

For each of the 18 functionals:

1. Hand-curate 5-10 density points per Phase-4 strata (low-density polarised
   `min(a,b) ∈ [1e-7, 1e-3]`, gradient_stress, low-tau for metaGGAs).
2. Run Functional::eval at order 3 on those points; record the Rust output
   as the **regression snapshot** (`expected` field in the JSONL fixture).
3. Land a per-functional unit test under `crates/xcfun-kernels/tests/d19_<name>.rs`
   that re-runs the same eval and asserts strict 1e-13 vs the snapshot.
4. The snapshot is the Plan 06-00 substrate's CURRENT output. Future
   kernel-edit plans (whether or not auto-tightening is confirmed) MUST
   either preserve this output or update the fixture explicitly with a
   commit citing the new ground truth.

**Trade-off:** the test is regression-only at this revision (Rust-vs-Rust
strict 1e-13). It does NOT yet confirm auto-tightening vs C++ truth — that
gating happens when the orchestrator (or follow-up plan) restores
`xcfun-master/` and runs `cargo run -p validation --release -- --backend cpu
--order 3 --filter <names>`. The fixture format is stable and machine-read
by the test, so converting NEEDS-VERIFICATION → AUTO-TIGHTENED is a
mechanical follow-up: re-emit the `expected` field from a C++ baseline run,
re-run the test (must still pass at strict 1e-13), commit.
