---
status: gaps_found
phase: 04-metagga-tier-mode-contracted-aliases
generated: "2026-04-26T12:15:00.000Z"
must_haves_total: 6
must_haves_verified: 0
must_haves_failed: 6
---

# Phase 4 Verification — `gaps_found`

Phase 4 sign-off (Plan 04-06) cannot proceed. The full-matrix tier-2 validation
run completed but exposed two structural gaps that block the must_haves.

## Gap 1 — Validation driver does NOT iterate metaGGA functional IDs (BLOCKING)

**Evidence:** `validation/src/driver.rs:275-323` (`run_with_mode`) hard-codes a
list of 46 (FunctionalId, vars) tuples ending at `XC_B97_2C` (line 323). The 28+
metaGGA functional IDs implemented in Plans 04-00..04-04 (TPSS×2, REVTPSS×2,
TPSSLOCC, BR×3, CSC, SCAN family ×10, M05×4, M06×8, BLOCX) are **not in the
table**. The C++ harness extension shipped in Plan 04-01/04-02/04-03
`validation/c_stubs.cpp` exists, but the Rust-side driver never invokes it for
any metaGGA ID.

**Confirmed by `validation/report.jsonl` (3.83M records, 1.6 GB):** unique
functional names = 46. None of `XC_TPSSX`, `XC_SCANX`, `XC_M06X`, `XC_BRX`,
`XC_BLOCX`, etc. appear in the output.

**Impact on must_haves:**
- `MGGA-01..05` — TPSS/SCAN/M05/M06/BLOCX tier-2 parity at 1e-12 — UNVERIFIED
- `MODE-03` — Mode::Contracted at orders 0..=6 — partially verified (cross-mode
  test ships only for SLATERX + PBEX); metaGGA Contracted parity UNVERIFIED
- Plan 04-06 acceptance criterion #1 (`exits 0 across all 77 functionals`) is
  not testable — driver only iterates 46.

**Fix:** extend `run_with_mode` (and `run_potential` at line 565+) with the 28+
missing metaGGA tuples. Each entry is one line:
```rust
(FunctionalId::XC_TPSSX, "XC_TPSSX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
```
plus matching `c_stubs.cpp` entries (these may already exist — verify per id).
Then re-run `cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'`.
Expected runtime: ~3.5 min/functional × 28 metaGGAs = ~100 min for the metaGGA
extension; full matrix re-run ~5 hours.

## Gap 2 — LDA/GGA edge-case failures beyond Phase-3 D-19 forwards

**Evidence:** `validation/report.html` (recovered from worktree, committed).
Headline: 28,680,020 records evaluated, 3,806,228 failed (13.27%).

**Per-functional max rel_err at order 3 (highlights):**

| Functional | o0 | o1 | o2 | o3 | Status |
|---|---|---|---|---|---|
| XC_SLATERX | 2.3e-16 | 2.3e-16 | 2.4e-16 | 2.4e-16 | ✓ green all orders |
| XC_PBEX | 5.3e-16 | 9.3e-16 | 5.8e-15 | 5.0e-14 | ✓ green all orders |
| XC_REVPBEX | 5.9e-16 | 1.0e-15 | 1.2e-14 | 6.4e-14 | ✓ green all orders |
| XC_BTK | 0 | 0 | 0 | 4.8e-15 | ✓ green |
| XC_KTX | 0 | 0 | 0 | 5.5e-16 | ✓ green |
| XC_TFK | 4.6e-16 | 4.6e-16 | 4.6e-16 | 4.6e-16 | ✓ green |
| XC_VWN3C / VWN5C / VWN_PBEC | green o0/o1 | red o2 (~1e-11) | red o3 (~1e-9 to 1e-11) | red | ⚠ edge-case at low density |
| XC_PBEC / XC_PZ81C / XC_PW92C | green o0/o1 | red o2 (~1e-12 to 1e-11) | red o3 | ⚠ edge-case at low density |
| XC_LDAERFX | 4.2e-9 | 4.2e-9 | red 6.7e-2 | **red 1.11e+1** | ⚠ catastrophic high-order divergence (NEW) |
| XC_LDAERFC | 4.4e-16 | 5.5e-16 | red 7.5e-6 | **red 5.10e+2** | ⚠ catastrophic high-order divergence (NEW) |
| XC_LDAERFC_JT | 1.6e-9 | 2.5e-9 | red 8.4e-7 | red 1.07e-4 | ⚠ catastrophic high-order divergence (NEW) |
| XC_BECKESRX | 9.1e-15 | red 1.6e-3 | red 1.05e-1 | **red 2.27e+2** | known D-19 forward (Phase 3) |
| XC_PBEINTC | red 4.5e-1 | red 9.0e-1 | red 1.92 | **red 6.17e+1** | known D-19 forward (Phase 3) |
| XC_APBEX | red 2.06e-1 (all orders) | known D-19 forward (Phase 3) |
| XC_PW86X / XC_PW91C / XC_PW91K / XC_P86C / XC_P86CORRC / XC_SPBEC / XC_B97C / XC_B97_1C / XC_B97_2C | red multiple orders | all known D-19 forwards (Phase 3) |

**Analysis:**
- Six functionals are fully green at strict 1e-12 across orders 0–3:
  XC_SLATERX, XC_TFK, XC_KTX, XC_BTK, XC_PBEX, XC_REVPBEX, plus partial passes
  on XC_PBESOLX, XC_RPBEX, XC_PW91X, XC_BECKEX, XC_BECKECORRX, XC_B97X,
  XC_B97_1X, XC_B97_2X, XC_OPTX (all green o0–o2).
- Most "failures" on otherwise-correct functionals (VWN3C, PZ81C, PBEC) are
  rel_err in the 1e-12 to 1e-11 range — edge-case rounding at low-density grid
  points (e.g. ρ ≈ 1e-13). These are candidates for D-19 INCONCLUSIVE entries.
- **Three NEW catastrophic divergences** not on the Phase-3 D-19 forward list:
  XC_LDAERFX (o3 = 1.11e+1), XC_LDAERFC (o3 = 5.10e+2), XC_LDAERFC_JT. These
  are the range-separated LDAs and exceed any tolerance — likely a real
  numerical bug introduced or amplified in Phase 3 that wasn't caught at the
  time. Need investigation, not just a D-19 forward.

**Impact on must_haves:**
- Plan 04-06 must_have "All 77 pass OR documented D-19 INCONCLUSIVE entries"
  cannot be checked against unmaintained data.

## Gap 3 — Contracted spot-checks at orders 5/6 not exercised end-to-end

The plan calls for `cargo run -p validation -- --backend cpu --mode contracted
--order 6 --filter 'tpssx'`. The `Mode::Contracted` driver branch ships
(Plan 04-05), but invoking it on `tpssx` exercises Gap 1's missing-driver-entry
problem first — the filter resolves no functional, so the run is a no-op.

## Aggregate Verdict

| Must-have | Status |
|---|---|
| MGGA-01 (TPSS family tier-2 GREEN) | UNVERIFIED — driver gap |
| MGGA-02 (SCAN family tier-2 GREEN) | UNVERIFIED — driver gap |
| MGGA-03 (M05 family tier-2 GREEN) | UNVERIFIED — driver gap |
| MGGA-04 (M06 family tier-2 GREEN) | UNVERIFIED — driver gap |
| MGGA-05 (BLOCX tier-2 GREEN) | UNVERIFIED — driver gap |
| MODE-03 (Mode::Contracted orders 0–6) | PARTIAL — only SLATERX/PBEX/PBEC cross-mode unit tests pass; metaGGA Contracted unverified |
| ALIAS-01..06 (alias engine + parameters) | PASS at the unit-test level (Plan 04-04); full-matrix invocation pending Gap 1 |

**Phase 4 is not signed off. ROADMAP/STATE/REQUIREMENTS were NOT advanced.**

## What Was Verified Cleanly

- ✓ Pre-flight gates (regen-registry --check, check-no-anyhow, check-no-mul-add)
- ✓ Tier-1 self-tests for all 77 functional IDs (`cargo test -p xcfun-eval --test self_tests`)
- ✓ Cross-mode parity tests (`contracted_cross_mode`) — 15/15 GREEN at strict 1e-12 for SLATERX (LDA) and PBEX (GGA)
- ✓ Full xcfun-eval workspace test suite (alias canary + parameter defaults + 22 lib tests)
- ✓ The Mode::Contracted dispatch layer itself (Plan 04-05) is structurally correct

## Recommended Path

**Option A — close the gap properly (recommended):**
```bash
/clear
/gsd-plan-phase 4 --gaps
```
This reads this VERIFICATION.md and creates a gap-closure plan (e.g. 04-07-driver-extension)
that:
1. Extends `validation/src/driver.rs::run_with_mode` (and `run_potential`) with
   28+ metaGGA tuples + matching c_stubs.cpp entries.
2. Re-runs the full-matrix validation.
3. Investigates the LDAERFX/LDAERFC/LDAERFC_JT catastrophic divergences (NEW
   D-19 candidates that need root-cause analysis, not just forwarding).
4. Updates report artifacts, REQUIREMENTS, ROADMAP, STATE on a clean run.

**Option B — extend driver inline now (faster, riskier):**
Edit `validation/src/driver.rs` to append the missing tuples, re-run validate
(~5 hours), then re-run Plan 04-06 task 6.1+. This skips the planning hop but
loses the explicit gap-closure trail that GSD prefers.

**Option C — partial sign-off with explicit deferral:**
Document the gaps in STATE.md as Phase-4 deferred items, mark Phase 4 as
"complete with caveats", and address in a Phase 4.1. NOT recommended — the
metaGGA work is the core deliverable of Phase 4 and shipping it without
parity validation defeats the purpose.

## Key-files (verified to exist on disk)

- `validation/report.html` (recovered from worktree `agent-a8306586435ea59ec`, committed)
- `validation/report.jsonl` — 1.6 GB in worktree (gitignored, will not be committed)
- `crates/xcfun-eval/src/functionals/contracted.rs` (Plan 04-05, GREEN)
- `crates/xcfun-eval/src/functionals/mgga/` (32 functional kernels, all build, all pass tier-1)
- `validation/src/driver.rs:275-323` ← the file that needs extension
