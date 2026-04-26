---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "10"
type: execute
wave: 3
depends_on:
  - "07"
  - "08"
  - "09"
files_modified:
  - validation/report.html
  - validation/report.jsonl
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
  - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
requirements:
  - MGGA-01
  - MGGA-02
  - MGGA-03
  - MGGA-04
  - MGGA-05
  - MODE-03
  - ALIAS-01
  - ALIAS-02
  - ALIAS-03
  - ALIAS-04
  - ALIAS-05
  - ALIAS-06
autonomous: false
gap_closure: true
created: "2026-04-26"
goal: "Wave 3 (gap closure) — re-execute the 6 must_haves from 04-06-validation-signoff-PLAN.md now that Plans 04-07/08/09 closed the 3 gaps; produce a new VERIFICATION.md with `must_haves_verified: 6 / 6` (or signoff_with_caveats: N/6 if some D-19 forwards remain unresolved); advance STATE.md / ROADMAP.md / REQUIREMENTS.md to mark Phase 4 complete"

must_haves:
  truths:
    - "All 15 metaGGA functionals (TPSS×5 + SCAN×10 + M05×4 + M06×8 + BLOCX = 28; minus excluded BR×3 + CSC = 24 active) pass tier-2 parity at 1e-12 on orders 0..=3 OR have explicit D-19 INCONCLUSIVE entries forwarded to Phase 6 (mirrors 04-06 must-have #1)"
    - "Mode::Contracted produces correct output at orders 0..=4 cross-checked against PartialDerivatives for at least 5 functionals (SLATERX, PBEX, TPSSX, SCANX, M06X — Plan 04-05 + 04-09); orders 5..=6 D-19 forwarded to Phase 6 (mirrors 04-06 must-have #2)"
    - "All 46 aliases resolve with correct weights; Functional::set('b3lyp', 1.0) + set('slaterx', 0.5) yields slaterx weight 1.30; XC_EXX/RANGESEP_MU/CAM_ALPHA/CAM_BETA defaults verified (mirrors 04-06 must-haves #3-5; PASS at unit-test level per VERIFICATION.md, just needed driver to not be the blocker)"
    - "`cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'` exits 0 OR produces only D-19-documented failures (mirrors 04-06 must-have #6, now actually testable thanks to Plan 04-07)"
  artifacts:
    - path: validation/report.html
      provides: "tier-2 parity report for Phase 4 sign-off — order 3, ≥76 functionals iterated"
      contains: "Phase 4"
    - path: validation/report.jsonl
      provides: "JSONL parity record for Phase 4 sign-off (gitignored, regeneratable)"
    - path: .planning/REQUIREMENTS.md
      provides: "MGGA-01..05, MODE-03, ALIAS-01..06 marked Complete (or Complete with caveats)"
      contains: "Complete"
    - path: .planning/STATE.md
      provides: "Phase 4 completion record + Phase-4 D-19 forwards integrated into the cross-phase ledger"
      contains: "phase-4-complete"
    - path: .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
      provides: "Updated VERIFICATION.md with `status: signed_off` (or `signed_off_with_caveats`) and `must_haves_verified: 6 / 6`"
      contains: "signed_off"
  key_links:
    - from: "04-06-validation-signoff-PLAN.md must_haves"
      to: "04-VERIFICATION.md (this plan rewrites it)"
      via: "Plan 04-10 task 4 regenerates the file with all 6 must_haves verified"
      pattern: "must_haves_verified: 6"
---

<objective>
This is the Phase-4 sign-off plan. It re-executes the same 6 must_haves listed in 04-06-validation-signoff-PLAN.md, now that the 3 gaps that prevented sign-off (driver coverage, ERF divergence, contracted metaGGA spot-checks) have been closed by Plans 04-07, 04-08, and 04-09 respectively.

**This plan does NOT:**
- Re-investigate ERF root causes (Plan 04-08 owns that, with Phase-6 forward).
- Add new metaGGA tests beyond what Plan 04-09 lands.
- Modify production code (validation/src/driver.rs and crates/xcfun-eval/ are frozen for the sign-off run).

**This plan DOES:**
- Run the full-matrix tier-2 sweep at order 3 (`cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'`).
- Run the alias canary tests + parameter-default tests (already GREEN per 04-VERIFICATION.md "What Was Verified Cleanly" — re-run for the sign-off ledger).
- Run the contracted_cross_mode test (now includes 3 new metaGGA exemplars from Plan 04-09).
- Reconcile any new D-19 INCONCLUSIVE entries against the per-fn-summary.json artifact from Plan 04-07 + the bisection log from Plan 04-08.
- Update `.planning/REQUIREMENTS.md` (12 IDs to Complete-or-Complete-with-caveats), `.planning/ROADMAP.md` (Phase 4 marked complete), `.planning/STATE.md` (advance to Phase 5 ready), and rewrite `04-VERIFICATION.md` to `signed_off` status with `must_haves_verified: 6 / 6`.

This plan has a `checkpoint:human-verify` gate after the full-matrix run (mirroring 04-06 task 6.2) so the developer reviews the report.html before the sign-off ledger is committed.

Output: Phase 4 signed off; ROADMAP advanced; STATE shows phase-4-complete; report artifacts committed.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/REQUIREMENTS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-06-validation-signoff-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
</context>

<tasks>

<task id="10.1" type="auto">
  <name>Task 1: Pre-flight gates + full-matrix tier-2 sweep at order 3 (the actual sign-off run)</name>
  <files>
    validation/report.html
    validation/report.jsonl
  </files>
  <read_first>
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-06-validation-signoff-PLAN.md` lines 80-137 — the original Task 6.1 actions; this plan inherits the same step structure.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-07-driver-extension-SUMMARY.md` (post-04-07) — the per-fn-summary.json artifact location and the order-2 partial-sweep verdict.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-08-erf-divergence-SUMMARY.md` (post-04-08) — the ERF + LDA-corr disposition table.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-09-contracted-metagga-SUMMARY.md` (post-04-09) — the metaGGA cross-mode pass/fail table.
  </read_first>
  <action>
    **Step 1 — pre-flight gates (must all pass):**
    ```bash
    cargo xtask regen-registry --check 2>&1 | tail -3
    cargo xtask check-no-anyhow 2>&1 | tail -3
    cargo xtask check-no-mul-add 2>&1 | tail -3
    cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -5
    cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | tail -5
    cargo test -p xcfun-eval 2>&1 | tail -5
    ```
    All six must exit 0.

    **Step 2 — full-matrix tier-2 sweep at order 3:**
    ```bash
    cargo run -p validation --release -- --backend cpu --order 3 --filter '.*' 2>&1 | tee /tmp/04-10-signoff-sweep.log | tail -30
    ```
    Expected runtime: ~5 hours per 04-VERIFICATION.md estimate. Run in foreground (NOT background) so the developer can observe progress; the contained `tee` captures the log for diagnostics.

    Expected output: `Tier-2 done: <N> records evaluated, <M> failed (<K> rust-unavailable, ...)`. M (= failed) should equal the sum of:
    - 13 inherited Phase-3 D-19 forwards (PW86X, APBEX, APBEC, P86C, PW91C, B97C, B97_1C, B97_2C, SPBEC, PBEINTC, PW91K, P86CORRC, BECKESRX)
    - 3 ERF entries from Plan 04-08 (LDAERFX, LDAERFC, LDAERFC_JT) — D-19 forward, status documented in REQUIREMENTS
    - Any new D-19 entries from Plan 04-08 Task 8.3 (LDA-correlation low-density)
    - Any new D-19 entries from Plan 04-07 task 7.3 per-fn-summary triage (metaGGA failures, if any — the order-2 sweep showed which families are clean vs problematic)

    **Step 3 — re-run contracted spot-checks at orders 5/6 for SLATERX + PBEX (NOT metaGGA — orders 5/6 metaGGA Contracted forwarded to Phase 6 by Plan 04-05):**
    ```bash
    cargo run -p validation --release -- --backend cpu --mode contracted --order 5 --filter 'slaterx,pbex' 2>&1 | tail -10
    cargo run -p validation --release -- --backend cpu --mode contracted --order 6 --filter 'slaterx,pbex' 2>&1 | tail -10
    ```
    Both must exit 0 (per Plan 04-05 D-19 — these emit "SKIP-WITH-RECORD" markers because C++ output_length die's; that's the expected behaviour).

    **Step 4 — alias canary tests:**
    ```bash
    cargo test -p xcfun-eval test_camcompx_negative_weight 2>&1 | tail -3
    cargo test -p xcfun-eval test_b3lyp_additive_accumulation 2>&1 | tail -3
    cargo test -p xcfun-eval test_exx_parameter_overwrite 2>&1 | tail -3
    cargo test -p xcfun-eval test_case_insensitive 2>&1 | tail -3
    ```
    All four must exit 0. (Per 04-VERIFICATION.md "What Was Verified Cleanly", these already pass; re-run is for the sign-off ledger.)

    **Step 5 — commit report artifacts:**

    `validation/report.html` is small and committed. `validation/report.jsonl` is gitignored at order 3 (~1.6 GB). DO NOT add the JSONL to git. Run:
    ```bash
    git add validation/report.html
    git commit -m "docs(04-10): tier-2 capstone — order 3 full-matrix sweep, post-gap-closure"
    ```

    Record the final failure count and the per-functional max_rel_err map; save to `/tmp/04-10-failure-ledger.txt` for Task 10.3 consumption.
  </action>
  <acceptance_criteria>
    1. `cargo xtask regen-registry --check 2>&1; echo $?` ends with 0.
    2. `cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | grep -cE "test result: ok\." | head -1` is at least 1.
    3. `cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | grep -cE "test result: ok\." | head -1` is at least 1.
    4. `tail -5 /tmp/04-10-signoff-sweep.log | grep -c "Tier-2 done"` is exactly 1 (sweep completed).
    5. `jq -r '.functional' validation/report.jsonl | sort -u | wc -l` is at least 76 (all metaGGAs iterated, post-04-07).
    6. `cargo run -p validation --release -- --backend cpu --mode contracted --order 6 --filter 'slaterx' 2>&1 | grep -c "SKIP-WITH-RECORD\|Tier-2 done"` is at least 1.
    7. `git log -1 --oneline | grep -c "04-10"` is exactly 1.
    8. `test -s /tmp/04-10-failure-ledger.txt` returns 0.
  </acceptance_criteria>
  <done>Pre-flight gates GREEN; full-matrix sweep at order 3 complete; contracted orders 5/6 verified for SLATERX+PBEX (metaGGA forwarded to Phase 6); alias canary tests GREEN; report.html committed; per-functional failure ledger captured.</done>
</task>

<task id="10.2" type="checkpoint:human-verify" gate="blocking">
  <what-built>
    Full-matrix tier-2 sweep at order 3 completed (Task 10.1):
    - `cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'` ran across all functional IDs.
    - report.html updated with Phase 4 capstone (post-gap-closure) entry.
    - Alias canary tests GREEN (4 tests: camcompx negative weight, b3lyp additive accumulation, exx parameter overwrite, alias case-insensitive).
    - Mode::Contracted spot-checks at orders 5/6 verified for SLATERX + PBEX (metaGGA orders 5/6 explicitly forwarded to Phase 6 per Plan 04-05).
    - 3 new metaGGA cross-mode tests (Plan 04-09) GREEN at strict 1e-12 across orders 0..=4 (or order ≤ 3 with documented N≥4 caveat).
    - Per-functional failure ledger captured at /tmp/04-10-failure-ledger.txt.
  </what-built>
  <how-to-verify>
    1. Open `validation/report.html` in a browser. Confirm:
       a. The capstone run timestamp matches today's date (file mtime within last 24 hours).
       b. The functional count in the matrix is at least 76 (was 46 in the gap_found state).
       c. Every NEW failing row that is NOT on the published Phase-3 D-19 forward list (13 entries: PW86X, APBEX, APBEC, P86C, PW91C, B97C, B97_1C, B97_2C, SPBEC, PBEINTC, PW91K, P86CORRC, BECKESRX) has been documented as a Plan 04-08 D-19 entry in REQUIREMENTS.md or STATE.md.
       d. The 3 ERF rows (LDAERFX, LDAERFC, LDAERFC_JT) display the order-3 max_rel_err with the threshold column showing 1e-7 (D-24 envelope) — the row appears as FAIL or INCONCLUSIVE per the Plan 04-08 verdict, NOT silently green.
       e. The metaGGA rows (TPSS family, SCAN family, M05/M06 families, BLOCX) — if any are FAIL with max_rel_err > 1e-7, they must be in the Plan-04-08-or-04-10 D-19 forward log; no silent failure.
       f. BR family + CSC rows display as `EXCLUDED (excluded_by_upstream_spec)` per Plan 04-07 task 7.2 skip-list extension — that's the expected verdict pending Phase-6 JP grid harness.

    2. Run: `jq -s 'group_by(.functional) | map({functional: .[0].functional, max_rel: (map(select(.excluded_by_regularize_clamp_design == false and .excluded_by_upstream_spec == false)) | map(.rel_err) | max)}) | sort_by(.max_rel) | reverse | .[0:10]' validation/report.jsonl` — confirm the top-10 worst max_rel_err matches the expected D-19 set (13 Phase-3 inherited + 3 ERF + any new from 04-07/08).

    3. Run: `cargo test -p xcfun-eval test_camcompx_negative_weight test_b3lyp_additive_accumulation test_exx_parameter_overwrite 2>&1 | grep -cE "test result: ok\." | head -1` — must be at least 1.

    4. Run: `cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | grep -cE "test result: ok\." | head -1` — must be at least 1.

    5. Visually confirm `/tmp/04-10-failure-ledger.txt` matches the report.html top-10 list (no surprise failures; all are documented forwards).
  </how-to-verify>
  <resume-signal>Type "approved" if all 5 verification steps pass and the report.html shows no undocumented failures, OR describe the specific row(s) that need additional D-19 entries before sign-off.</resume-signal>
</task>

<task id="10.3" type="auto">
  <name>Task 3: REQUIREMENTS.md / ROADMAP.md / STATE.md sign-off updates</name>
  <files>
    .planning/REQUIREMENTS.md
    .planning/ROADMAP.md
    .planning/STATE.md
  </files>
  <read_first>
    - `.planning/REQUIREMENTS.md` — find every Phase-4 requirement ID (MGGA-01..05, MODE-03, ALIAS-01..06).
    - `.planning/ROADMAP.md` — find the Phase 4 entry.
    - `.planning/STATE.md` — full file. Find the Accumulated Context section and the progress.completed_phases counter.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-10 (Phase-3 D-19 forwards stay forwarded), D-13 (LB94 stays in Phase 5).
    - `/tmp/04-10-failure-ledger.txt` — the post-sweep per-functional max_rel_err ledger.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-08-erf-divergence-SUMMARY.md` — Plan 04-08 D-19 entries.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-09-contracted-metagga-SUMMARY.md` — Plan 04-09 MODE-03 update.
  </read_first>
  <action>
    Apply the same sign-off updates as 04-06 task 6.3, with adjustments to reflect the Plan 04-07/08/09 work that closed the gaps.

    **REQUIREMENTS.md updates** — for each of the 12 Phase-4 IDs, change `[ ]` to `[x]` (Complete) or `[~]` (Complete with caveats):

    For each ID, examine `/tmp/04-10-failure-ledger.txt`. If the family had 0 failures at strict 1e-12 across all orders 0..=3, mark `[x] Complete`. If ≥1 failure exists but is in the D-19 forward list, mark `[~] Complete (with caveats)` and append `— D-19 forwards: <list>` to the line.

    Specific lines to update:
    - `MGGA-01` (TPSS family, line 60) — already `[x]` in current REQUIREMENTS; verify still GREEN at order 3 in the sweep, otherwise downgrade to `[~]`.
    - `MGGA-02` (SCAN family, line 61) — `[ ]` → `[x]` or `[~]` per sweep.
    - `MGGA-03` (M05 family, line 62) — `[ ]` → `[x]` or `[~]` per sweep.
    - `MGGA-04` (M06 family, line 63) — `[ ]` → `[x]` or `[~]` per sweep.
    - `MGGA-05` (BLOCX, line 64) — `[ ]` → `[x]` or `[~]` per sweep.
    - `MODE-03` (line 70) — already updated by Plan 04-09 task 9.3; CONFIRM the `[~]` marker is in place.
    - `ALIAS-01..06` (lines 76-81) — `[ ]` → `[x]` (per 04-VERIFICATION.md "PASS at unit-test level", which Task 10.1 step 4 re-confirmed).
    - **GGA-03 carryover** (line 49) — already `[~] DEFERRED to Phase 4`. Update to `[x]` (BR family ported via Plan 04-01) or `[~] Complete with caveats — BR family compiles and passes tier-1; tier-2 marked excluded_by_upstream_spec until Phase-6 JP grid harness lands`.
    - **GGA-10 carryover** (line 56) — currently `[~]`. Update to indicate XC_CSC ported (Plan 04-01) but tier-2 excluded_by_upstream_spec; LB94 still Phase 5.

    **ROADMAP.md updates:**

    Find the Phase 4 entry. Replace with:
    ```markdown
    - [x] **Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases** - Complete (2026-04-26) [^d19p4] — 32 functional bodies (28 metaGGA + 4 carryovers BRX/BRC/BRXC + CSC); 46-alias engine + 4 parameters; Mode::Contracted orders 0..=4 verified across 5 functionals (LDA + GGA + 3 metaGGA exemplars), orders 5..=6 D-19 forward to Phase 6; full-matrix tier-2 at order 3 GREEN subject to inherited Phase-3 D-19 forwards (13 entries) + Plan 04-08 ERF + LDA-corr forwards. Plans: 11 total (04-00..04-06 original + 04-07/08/09/10 gap closure).
    ```

    Update Plans count to "11 plans (04-00..04-10)".

    Add a footnote `[^d19p4]` if not present, listing every Phase-4-discovered D-19 entry verbatim from Plan 04-08-SUMMARY.md and the metaGGA per-fn-summary.

    **STATE.md updates:**

    In the YAML frontmatter:
    - `last_updated: "<ISO_TIMESTAMP_AT_RUN>"`
    - `progress.completed_phases: 4`
    - `progress.total_plans: 32` (pre-gap = 28; +4 gap-closure = 32)
    - `progress.completed_plans: 32`
    - `progress.percent: 50` (4/8 phases)

    Replace the `## Current Position` block with:
    ```markdown
    ## Current Position

    Phase: 04 (metagga-tier-mode-contracted-aliases) — **COMPLETE WITH GAP CLOSURE**
    Plans: 11 (04-00 ✓, 04-01 ✓, 04-02 ✓, 04-03 ✓, 04-04 ✓, 04-05 ✓, 04-06 partial→VERIFICATION gaps_found, 04-07 ✓ gap closure driver extension, 04-08 ✓ gap closure ERF investigation, 04-09 ✓ gap closure contracted metaGGA, 04-10 ✓ phase re-signoff)
    Scope: 32 functional bodies (28 metaGGA + 4 carryovers); 46 aliases + 4 parameters; Mode::Contracted orders 0..=4 verified.

    - **Milestone:** Initial v1 build-out
    - **Phase:** 04 (metagga-tier-mode-contracted-aliases) — **COMPLETE (2026-04-26)**
    - **Plan:** 04-10 complete. All 11 Phase-4 plans shipped.
    - **Status:** Phase-4 complete; ready for Phase 5 (Rust facade + C ABI)
    - **Progress:** [████████░░] 50%
    ```

    In the Accumulated Context > Decisions section, append:
    ```markdown
    ### Decisions added in Phase 4 (gap closure plans 04-07/08/09/10)

    - **Plan 04-07 driver extension:** validation/src/driver.rs::run iterates all 30 metaGGA tuples; run_launch wired with 120 new arms at vars=13/17 × n ∈ {0..3}. BR family + CSC pre-emptively excluded_by_upstream_spec (Phase-6 JP grid follow-up).
    - **Plan 04-08 ERF divergence forward:** XC_LDAERFX/LDAERFC/LDAERFC_JT order-3 catastrophic divergence confirmed as AD-chain amplification of the known erf bracket cancellation. No Phase-4 viable fix per bisection (Task 8.1). Forwarded to Phase 6 libm-hybrid.
    - **Plan 04-09 contracted metaGGA cross-mode:** Mode::Contracted orders 0..=4 verified for TPSSX, SCANX, M06X exemplars at strict 1e-12. Orders 5..=6 metaGGA Contracted reinforces Plan 04-05 D-19 forward (xcfun-ad ctaylor_compose/multo N=4..=6 specialisations — Phase 6 prerequisite).
    - **Phase 4 D-19 forward list (consolidated):**
      * 13 inherited Phase-3 forwards: PW86X, APBEX, APBEC, P86C, PW91C, B97C, B97_1C, B97_2C, SPBEC, PBEINTC, PW91K, P86CORRC, BECKESRX
      * 3 NEW Phase-4 ERF forwards: LDAERFX, LDAERFC, LDAERFC_JT
      * 0 to N NEW Phase-4 metaGGA forwards (per Plan 04-07 per-fn-summary triage; final count in 04-10-SUMMARY.md)
      * 0 to N NEW Phase-4 LDA-correlation forwards (per Plan 04-08 Task 8.3 triage)
      * BR family + CSC excluded_by_upstream_spec (Phase 6 JP grid)
      * Mode::Contracted orders 5..=6 metaGGA (Phase 6 xcfun-ad N≥4)
    - **BLOCX confirmed BRX-independent** per RESEARCH finding (CONTEXT D-01-A claim corrected — BLOCX is TPSS-shaped, no `BR(...)` call).
    - **LB94 stays in Phase 5** per D-13 (legacy `setup_lb94` pattern not in 78-entry FunctionalId enum).
    ```

    Commit:
    ```bash
    git add .planning/REQUIREMENTS.md .planning/ROADMAP.md .planning/STATE.md
    git commit -m "docs(04-10): Phase 4 sign-off — mark MGGA-01..05 MODE-03 ALIAS-01..06 Complete; advance STATE to Phase 5"
    ```
  </action>
  <acceptance_criteria>
    1. `grep -E "^- \[(x|~)\] \*\*MGGA-0[1-5]\*\*" .planning/REQUIREMENTS.md | wc -l` is exactly 5.
    2. `grep -E "^- \[(x|~)\] \*\*ALIAS-0[1-6]\*\*" .planning/REQUIREMENTS.md | wc -l` is exactly 6.
    3. `grep -E "^- \[(x|~)\] \*\*MODE-03\*\*" .planning/REQUIREMENTS.md | wc -l` is exactly 1.
    4. `grep -nE "Phase 4.*Complete \(2026-04-26\)" .planning/ROADMAP.md | wc -l` is at least 1.
    5. `grep -n "completed_phases: 4" .planning/STATE.md | wc -l` is exactly 1.
    6. `grep -n "ready for Phase 5" .planning/STATE.md | wc -l` is at least 1.
    7. `grep -n "BLOCX confirmed BRX-independent" .planning/STATE.md | wc -l` is exactly 1.
    8. `grep -nE "Plan 04-08.*ERF|XC_LDAERFX.*Phase 6" .planning/STATE.md | wc -l` is at least 1.
    9. `git log -1 --oneline | grep -c "04-10"` is exactly 1.
    10. `git diff HEAD~1 HEAD --name-only | sort -u | grep -E "REQUIREMENTS\.md|ROADMAP\.md|STATE\.md" | wc -l` is exactly 3.
  </acceptance_criteria>
  <done>REQUIREMENTS.md: 12 Phase-4 IDs marked Complete (or Complete with caveats); ROADMAP Phase 4 marked complete; STATE.md advanced to Phase 5 ready; all D-19 forwards documented in cross-phase ledger.</done>
</task>

<task id="10.4" type="auto">
  <name>Task 4: Rewrite 04-VERIFICATION.md to `signed_off` (or `signed_off_with_caveats`) with `must_haves_verified: 6 / 6`</name>
  <files>
    .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
  </files>
  <read_first>
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` — current `gaps_found` state (this file is being replaced).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-06-validation-signoff-PLAN.md` — the original 6 must_haves to be re-asserted in the new VERIFICATION.md.
    - `/tmp/04-10-failure-ledger.txt` — the post-sweep state.
  </read_first>
  <action>
    Replace the entire content of `04-VERIFICATION.md` with the following structure:

    ```markdown
    ---
    status: signed_off
    phase: 04-metagga-tier-mode-contracted-aliases
    generated: "<ISO_TIMESTAMP>"
    must_haves_total: 6
    must_haves_verified: 6
    must_haves_failed: 0
    gap_closure: complete
    superseded_verification: gaps_found (2026-04-26T12:15:00.000Z)
    ---

    # Phase 4 Verification — `signed_off`

    Phase 4 sign-off lands. The 3 gaps reported in the previous (`gaps_found`)
    VERIFICATION.md were closed by gap-closure plans 04-07, 04-08, 04-09;
    Plan 04-10 re-executed the 6 must_haves from the original 04-06 sign-off.

    ## Must-have ledger (post-gap-closure)

    | # | Must-have (from 04-06-validation-signoff-PLAN.md) | Status | Evidence |
    |---|---|---|---|
    | 1 | All 15 metaGGA functionals pass tier-2 parity at 1e-12 on orders 0..=3 (or D-19 forwarded) | VERIFIED | Plan 04-07 wired driver + run_launch; Plan 04-08 forwarded ERF + LDA-corr to Phase 6; report.jsonl iterates ≥76 functionals; Plan 04-10 sweep failure count = 13 inherited + 3 ERF + N new = X (see /tmp/04-10-failure-ledger.txt and STATE.md ledger) |
    | 2 | Mode::Contracted produces correct output cross-mode | VERIFIED | Plan 04-09 added 3 metaGGA exemplar tests at orders 0..=4 (TPSSX, SCANX, M06X), all GREEN at strict 1e-12. Orders 5..=6 forwarded to Phase 6 per Plan 04-05 D-19 (xcfun-ad N≥4 specialisations). |
    | 3 | All 46 aliases resolve with correct weights | VERIFIED | test_camcompx_negative_weight + test_b3lyp_additive_accumulation + test_exx_parameter_overwrite + test_case_insensitive all GREEN (Plan 04-04 + Task 10.1 step 4 re-confirmation). |
    | 4 | Functional::set('b3lyp', 1.0) + set('slaterx', 0.5) yields slaterx weight 1.30 | VERIFIED | test_b3lyp_additive_accumulation passes (Plan 04-04 + Task 10.1). |
    | 5 | XC_EXX/RANGESEP_MU/CAM_ALPHA/CAM_BETA defaults verified | VERIFIED | Parameter default tests pass (Plan 04-04 + Task 10.1). |
    | 6 | `cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'` exits 0 (or with documented D-19 entries) | VERIFIED | Task 10.1 step 2 captured in /tmp/04-10-signoff-sweep.log; failure count matches the consolidated D-19 ledger; no undocumented failures per Task 10.2 checkpoint. |

    ## D-19 INCONCLUSIVE forward summary (Phase 6 prerequisite)

    1. **Inherited from Phase 3 (13 entries — unchanged per CONTEXT D-10):**
       PW86X, APBEX, APBEC, P86C, PW91C, B97C, B97_1C, B97_2C, SPBEC, PBEINTC, PW91K, P86CORRC, BECKESRX.

    2. **NEW Phase-4 ERF entries (3, from Plan 04-08):**
       XC_LDAERFX (order-3 max_rel = 1.11e+1), XC_LDAERFC (order-3 max_rel = 5.10e+2), XC_LDAERFC_JT (order-3 max_rel = 1.07e-4).
       Root cause: AD-chain amplification of erf bracket cancellation (the same instability documented at orders 0..=2 by Phase 2 D-24, now visible at order 3+ once the Phase-3 order cap was lifted). Forwarded to Phase 6 libm-hybrid.

    3. **NEW Phase-4 metaGGA entries (0..N, from Plan 04-07 per-fn-summary triage):**
       Listed in 04-07-driver-extension-SUMMARY.md and reproduced in STATE.md.

    4. **NEW Phase-4 LDA-correlation entries (0..N, from Plan 04-08 Task 8.3 triage):**
       Listed in 04-08-erf-divergence-SUMMARY.md.

    5. **Excluded by upstream spec (BR family + CSC):**
       XC_BRX, XC_BRC, XC_BRXC, XC_CSC. Phase-6 JP grid harness required.

    6. **Mode::Contracted orders 5..=6 metaGGA (Plan 04-05 reinforcement):**
       Phase-6 xcfun-ad ctaylor_compose/multo N ∈ {4,5,6} specialisations required.

    ## Sign-off ledger

    - **Plan 04-07** (gap closure): driver + run_launch extension. Commits: <see git log>.
    - **Plan 04-08** (gap closure): ERF + LDA-corr triage. Commits: <see git log>.
    - **Plan 04-09** (gap closure): contracted metaGGA cross-mode. Commits: <see git log>.
    - **Plan 04-10** (re-signoff): full-matrix sweep + ledger updates. Commits: <see git log>.

    Phase 4 is **signed off**; Phase 5 (Rust facade + C ABI) is the next active phase.
    ```

    Replace `<ISO_TIMESTAMP>` with the actual run timestamp; replace the placeholder `X` and `0..N` counts with the actual counts from `/tmp/04-10-failure-ledger.txt`.

    Commit:
    ```bash
    git add .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
    git commit -m "docs(04-10): rewrite 04-VERIFICATION.md → signed_off (must_haves_verified: 6/6)"
    ```
  </action>
  <acceptance_criteria>
    1. `grep -c "^status: signed_off" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` is exactly 1.
    2. `grep -c "^must_haves_verified: 6" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` is exactly 1.
    3. `grep -c "^must_haves_failed: 0" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` is exactly 1.
    4. `grep -c "gap_closure: complete" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` is exactly 1.
    5. `grep -cE "Plan 04-(07|08|09|10)" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` is at least 4 (each gap-closure plan named).
    6. `git log -1 --oneline | grep -c "04-10"` is exactly 1.
  </acceptance_criteria>
  <done>04-VERIFICATION.md rewritten with `signed_off` status; 6/6 must_haves verified; D-19 ledger fully populated; commit landed.</done>
</task>

</tasks>

<verification>
```bash
# 10.1 sweep landed
test -s /tmp/04-10-signoff-sweep.log && grep -c "Tier-2 done" /tmp/04-10-signoff-sweep.log
jq -r '.functional' validation/report.jsonl | sort -u | wc -l   # ≥76

# 10.2 checkpoint approved (manual gate — no automated check)

# 10.3 Phase-4 IDs all marked Complete or Complete-with-caveats
grep -E "^- \[(x|~)\] \*\*MGGA-0[1-5]\*\*" .planning/REQUIREMENTS.md | wc -l   # 5
grep -E "^- \[(x|~)\] \*\*ALIAS-0[1-6]\*\*" .planning/REQUIREMENTS.md | wc -l  # 6
grep -E "^- \[(x|~)\] \*\*MODE-03\*\*" .planning/REQUIREMENTS.md | wc -l       # 1

# 10.4 VERIFICATION.md flipped
grep -c "status: signed_off" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md  # 1
grep -c "must_haves_verified: 6" .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md  # 1
```
</verification>

<success_criteria>
- Pre-flight gates GREEN; tier-1 GREEN; contracted_cross_mode GREEN (incl. 3 new metaGGA tests)
- Full-matrix tier-2 sweep at order 3 completes; failure count matches consolidated D-19 ledger; no undocumented failures
- Alias canary tests + parameter default tests GREEN
- 12 Phase-4 requirement IDs marked Complete (or Complete with caveats) in REQUIREMENTS.md
- ROADMAP.md Phase 4 marked complete
- STATE.md advanced to Phase 5 ready; cross-phase D-19 ledger updated
- 04-VERIFICATION.md rewritten to `signed_off` with `must_haves_verified: 6 / 6`
- All 4 commits landed (04-10 task 1 report.html; task 3 ledger updates; task 4 VERIFICATION rewrite)
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Sign-off ledger updates | Pure documentation updates; no code execution surface. |
| validation/report.jsonl (~1.6 GB at order 3) | Gitignored; read-only consumption by jq queries; never committed. |
| validation runner | cc-compiled C++ + Rust; same machine; no network; trusted input. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-10-01 | Repudiation | Sign-off recorded with hidden D-19 entries | mitigate | Task 10.2 checkpoint requires the developer to inspect each NEW failing row in report.html and confirm it appears in either Plan 04-08 SUMMARY or Plan 04-07 per-fn-summary.json. The ledger in STATE.md names every D-19 entry verbatim; CI re-running the sweep deterministically reproduces the result. |
| T-04-10-02 | Tampering | report.html / VERIFICATION.md manually edited to hide failures | mitigate | The xtask validate command regenerates report artifacts deterministically from seed + C++ reference. Any divergent committed report would be detectable by re-running the command in CI. The VERIFICATION.md d-19 ledger is cross-referenced in STATE.md and REQUIREMENTS.md; tampering with one would leave the other inconsistent. |
| T-04-10-03 | Denial of Service | report.jsonl growth (1.6 GB at order 3) blocks repo operations | mitigate | report.jsonl is in .gitignore and never committed. report.html is small and fits in git. Disk-pressure mitigation: developer can run on a tmpfs or pipe to /dev/null and rely on the in-memory Report aggregation for the failure ledger (Task 10.1 step 5 captures the ledger via tail of stdout, not by parsing report.jsonl). |
| T-04-10-04 | Information Disclosure | None | accept | All artefacts are deterministic numerics on synthetic densities; open-source project; no PII. |

No new code attack surface — this is a documentation + verification plan only.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-10-resignoff-SUMMARY.md`. Include:
- Final tier-2 sweep failure count and per-functional max_rel_err table.
- Confirmation that 04-VERIFICATION.md flipped to `signed_off`.
- Commit hashes for the four 04-10 commits.
- Reference to the consolidated Phase-4 D-19 forward list in STATE.md.
</output>
