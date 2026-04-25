---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "06"
type: execute
wave: 6
depends_on:
  - "04-05"
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
files_modified:
  - validation/report.html
  - validation/report.jsonl
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
autonomous: false
created: "2026-04-25"
goal: "Wave 6 — Full-matrix tier-2 validation (order 3, all 77 functionals), Contracted spot-checks at orders 5/6, Phase 4 sign-off; update REQUIREMENTS/ROADMAP/STATE"

must_haves:
  truths:
    - "All 15 metaGGA functionals pass tier-2 parity at 1e-12 on orders 0..=3 (or D-19 INCONCLUSIVE forwarded to Phase 6)"
    - "Mode::Contracted produces 1<<order outputs matching C++ DOEVAL on every legal (functional, vars, order) tuple"
    - "All 46 aliases resolve with correct weights (including negative-weight camcompx canary)"
    - "Functional::set('b3lyp', 1.0) + set('slaterx', 0.5) yields additive slaterx weight 1.30 matching C++ XCFunctional.cpp:389-402"
    - "XC_EXX/XC_RANGESEP_MU/XC_CAM_ALPHA/XC_CAM_BETA are settable via set() and readable via get() with correct defaults"
    - "cargo xtask validate --backend cpu --order 3 exits 0 across all 77 functionals (or with explicit D-19 INCONCLUSIVE entries signed off)"
  artifacts:
    - path: validation/report.html
      provides: tier-2 parity report for Phase 4
      contains: "Phase 4"
    - path: validation/report.jsonl
      provides: JSONL parity record for all 77 functionals at orders 0..=3
    - path: .planning/REQUIREMENTS.md
      provides: MGGA-01..05, MODE-03, ALIAS-01..06 marked Complete
      contains: "Complete"
    - path: .planning/STATE.md
      provides: Phase 4 completion record with D-19 forward entries
  key_links:
    - from: cargo xtask validate --backend cpu --order 3
      to: validation/report.jsonl
      via: tier-2 parity sweep producing JSONL output
      pattern: "order.*3"
---

<objective>
Phase 4 sign-off plan: run the full-matrix tier-2 validation (`xtask validate --backend cpu --order 3`) across all 77 functional IDs (78 - LB94), produce the committed report artifacts, forward any new D-19 INCONCLUSIVE entries to Phase 6, and update REQUIREMENTS/ROADMAP/STATE to mark Phase 4 complete.

This plan has a `checkpoint:human-verify` gate after the full-matrix run to confirm parity results before sign-off. The CI machine is the final arbiter — no manual exception applies.

Output: Updated validation artifacts, REQUIREMENTS/ROADMAP/STATE, Phase 4 complete.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/REQUIREMENTS.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md
</context>

<tasks>

<task id="6.1" type="auto">
  <name>Task 1: Full-matrix tier-2 run + Contracted spot-checks + report artifact commit</name>
  <files>
    validation/report.html,
    validation/report.jsonl
  </files>
  <read_first>
    - `validation/src/c_driver.rs` — current state including Contracted mode extension from Plan 04-05.
    - `validation/src/fixtures.rs` — current grid state (10k canonical + 1k metaGGA stratum from Plan 04-00).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-09 (tier-2 harness incremental extension), D-10 (13 Phase-3 D-19 forwards stay forwarded), D-11 (strict 1e-12 default)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md` — §"Wave 0 Requirements" (all checklist items) and §"Validation Sign-Off"
  </read_first>
  <action>
    Run the following in sequence:

    **Step 1: Pre-flight checks (must all pass before full-matrix run):**
    ```bash
    cargo xtask regen-registry --check   # ALIASES + PARAMETERS drift gate
    cargo xtask check-no-anyhow          # library boundary gate
    cargo xtask check-no-mul-add         # no FMA gate
    cargo test -p xcfun-eval --test self_tests --features testing  # tier-1 for all 77 ids
    cargo test -p xcfun-eval --test contracted_cross_mode --features testing  # cross-mode parity
    cargo test -p xcfun-eval  # alias canary + parameter default unit tests
    ```
    If any gate fails, stop and fix before proceeding.

    **Step 2: Full-matrix tier-2 validation (may take 10-20 min):**
    ```bash
    cargo xtask validate --backend cpu --order 3 --filter '.*'
    ```
    This runs all 77 functional ids (78 - LB94) at orders 0..=3 in Mode::PartialDerivatives on the 10k canonical grid + 1k metaGGA stratum at strict 1e-12.

    Expected: All 77 pass, OR known D-19 INCONCLUSIVE entries appear (the 13 inherited from Phase 3 per D-10, plus any new ones from metaGGA). Any NEW failures (not in the Phase-3 D-19 forward list) must be documented as new D-19 entries.

    **Step 3: Contracted spot-checks at orders 5/6:**
    ```bash
    cargo xtask validate --backend cpu --mode contracted --order 5 --filter 'slaterx,pbex,tpssx,m06x'
    cargo xtask validate --backend cpu --mode contracted --order 6 --filter 'slaterx,pbex,tpssx,m06x'
    ```
    Expected: all 4 functionals × orders 5/6 pass at strict 1e-12. If any fail, create D-19 INCONCLUSIVE entry.

    **Step 4: Commit report artifacts:**
    - `validation/report.html` — updated with Phase 4 capstone run (order 3, 77 functionals).
    - `validation/report.jsonl` — updated JSONL parity records.

    Git commit: `feat(phase-04): tier-2 capstone — order 3, 77 functionals; Contracted orders 5/6 spot-check`

    Record the final failure count (should be 0 + the inherited 13 D-19 forwards from Phase 3, + any new Phase-4 D-19 entries with structured documentation).
  </action>
  <acceptance_criteria>
    1. `cargo xtask validate --backend cpu --order 3 --filter '.*'` exits 0 OR the only failures are documented D-19 INCONCLUSIVE entries (inherited Phase-3 forwards + any new Phase-4 entries).
    2. `validation/report.html` file was modified in the past 1 hour (freshly committed).
    3. `grep -c '"status":"PASS"' validation/report.jsonl` is at least 77 * 4 = 308 records (77 functionals × 4 orders 0..=3), minus any D-19 exclusions.
    4. `cargo xtask validate --backend cpu --mode contracted --order 6 --filter 'slaterx'` exits 0 (at least 1 functional verified at order 6).
    5. `cargo test -p xcfun-eval --test self_tests --features testing` still exits 0 (all tier-1 GREEN).
  </acceptance_criteria>
  <done>Full-matrix tier-2 run complete. Report artifacts committed. Any new D-19 INCONCLUSIVE entries documented.</done>
</task>

<task id="6.2" type="checkpoint:human-verify" gate="blocking">
  <what-built>
    Full-matrix tier-2 validation completed:
    - `cargo xtask validate --backend cpu --order 3 --filter '.*'` ran on all 77 functional IDs.
    - `validation/report.html` updated with Phase 4 capstone run.
    - Contracted spot-checks at orders 5/6 ran for SLATERX, PBEX, TPSSX, M06X.
    - All 46 alias canary tests passed (b3lyp trace, camcompx -0.37 canary, additive accumulation, parameter overwrite, case-insensitive).
    - 4 parameter defaults verified: XC_RANGESEP_MU=0.4, XC_EXX=0.0, XC_CAM_ALPHA=0.19, XC_CAM_BETA=0.46.
  </what-built>
  <how-to-verify>
    1. Open `validation/report.html` in a browser. Verify that it shows a Phase 4 capstone run (order 3, 77 functionals listed). Confirm the overall failure count is 0 (not counting the inherited 13 Phase-3 D-19 forwards, which should appear as "INCONCLUSIVE" or "EXCLUDED" rows, not "FAIL").
    2. Inspect any NEW "FAIL" or "INCONCLUSIVE" rows that are NOT in the Phase-3 D-19 forward list (13 functionals: PW86X, APBEX, APBEC, P86C, PW91C, B97C, B97_1C, B97_2C, SPBEC, PBEINTC, PW91K, P86CORRC, BECKESRX). If new failures exist, confirm they have been documented as Phase-4 D-19 entries in STATE.md.
    3. Run: `cargo test -p xcfun-eval test_camcompx_negative_weight` — confirm passes with `beckecamx == -0.37`.
    4. Run: `cargo test -p xcfun-eval test_b3lyp_additive_accumulation` — confirm `get("slaterx") == 1.30`.
    5. Run: `cargo xtask validate --backend cpu --mode contracted --order 6 --filter 'tpssx'` — confirm exits 0.
  </how-to-verify>
  <resume-signal>Type "approved" if all 5 checks pass, or describe any issues found.</resume-signal>
</task>

<task id="6.3" type="auto">
  <name>Task 3: REQUIREMENTS/ROADMAP/STATE sign-off</name>
  <files>
    .planning/REQUIREMENTS.md,
    .planning/ROADMAP.md,
    .planning/STATE.md
  </files>
  <read_first>
    - `.planning/REQUIREMENTS.md` — READ FULLY. Find MGGA-01..05, MODE-03, ALIAS-01..06 entries to mark Complete.
    - `.planning/ROADMAP.md` — READ Phase 4 section. Mark phase complete. Update Plans field.
    - `.planning/STATE.md` — READ FULLY. Update current position to Phase 5 ready.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-10 (13 Phase-3 D-19 forwards stay forwarded unchanged), D-13 (LB94 stays in Phase 5).
  </read_first>
  <action>
    **1. `REQUIREMENTS.md` updates:**

    For each of the 12 Phase-4 requirement IDs, change `Pending` to `Complete` (or `Complete (with caveats)` if any D-19 entries were logged):
    - `MGGA-01: Complete` (TPSS/REVTPSS/TPSSLOCC tier-2 GREEN)
    - `MGGA-02: Complete` (SCAN family tier-2 GREEN)
    - `MGGA-03: Complete` (M05 family tier-2 GREEN)
    - `MGGA-04: Complete` (M06 family tier-2 GREEN, or Complete with caveats if D-19 forwarded per D-11 watch list)
    - `MGGA-05: Complete` (BLOCX tier-2 GREEN)
    - `MODE-03: Complete` (Mode::Contracted orders 0..=6 verified)
    - `ALIAS-01: Complete` (all 46 aliases resolve at strict weight trace)
    - `ALIAS-02: Complete` (XC_EXX=0.0 settable/readable)
    - `ALIAS-03: Complete` (XC_RANGESEP_MU=0.4 settable/readable)
    - `ALIAS-04: Complete` (XC_CAM_ALPHA=0.19 settable/readable)
    - `ALIAS-05: Complete` (XC_CAM_BETA=0.46 settable/readable)
    - `ALIAS-06: Complete` (Functional::set recursion with multiplicative weights + FIXME-EXX preserved)

    Also record: GGA-03 (BRX/BRC/BRXC carryover) Complete; GGA-10 (CSC portion) Complete.

    **2. `ROADMAP.md` updates:**

    Mark Phase 4 complete:
    ```markdown
    - [x] **Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases** - Complete (2026-04-25) [^d19p4] — 32 functional bodies (28 metaGGA + 4 carryovers BRX/BRC/BRXC + CSC); 46 aliases + 4 parameters; Mode::Contracted orders 0..=6; full-matrix tier-2 at order 3 GREEN (subject to D-19 INCONCLUSIVE entries forwarded to Phase 6).
    ```

    Update **Plans** count to "7 plans (04-00..04-06)".

    Add footnote `[^d19p4]` documenting any NEW Phase-4 D-19 INCONCLUSIVE entries (if any).

    **3. `STATE.md` updates:**

    Update:
    - `status: phase-4-complete`
    - `last_updated: "2026-04-25T23:xx:00.000Z"`
    - `progress.completed_phases: 4`
    - `progress.total_plans: 28` (21 + 7)
    - `progress.completed_plans: 28`
    - `progress.percent: 50` (4/8 phases)

    Current position: `Phase 04 complete; ready for Phase 5 (Rust facade + C ABI)`

    Record in Accumulated Context / Decisions:
    - **Phase 4 completion:** 32 functional bodies shipped (28 metaGGA + 4 carryovers). 46-alias engine wired. 4-parameter table. Mode::Contracted orders 0..=6. Strict 1e-12 tier-2 at order 3.
    - **LB94 stays Phase 5** per D-13 (confirmed not alias-feasible).
    - **Phase-3 D-19 forwards (13 entries) UNCHANGED** per D-10 — forwarded to Phase 6.
    - **Any new Phase-4 D-19 INCONCLUSIVE entries** (list them with functional name + max_rel_err + root cause hypothesis).
    - **BLOCX confirmed BRX-independent** per RESEARCH finding (CONTEXT D-01-A claim corrected — BLOCX is TPSS-shaped, no `BR(...)` call).

    Git commit: `docs(phase-04): sign-off — mark MGGA-01..05 MODE-03 ALIAS-01..06 Complete; advance STATE to Phase 5`
  </action>
  <acceptance_criteria>
    1. `grep -c "Complete" .planning/REQUIREMENTS.md` increased by at least 12 (12 Phase-4 IDs + 2 carryovers = 14 total new Complete entries).
    2. `grep -n "Phase 4.*Complete" .planning/ROADMAP.md` returns a match.
    3. `grep -n "completed_phases: 4" .planning/STATE.md` returns a match.
    4. `grep -n "ready for Phase 5" .planning/STATE.md` returns a match.
    5. `grep -n "BLOCX confirmed BRX-independent" .planning/STATE.md` returns a match (important correction documented).
    6. `git diff --name-only` after commit shows exactly: REQUIREMENTS.md, ROADMAP.md, STATE.md (+ report artifacts from Task 1).
  </acceptance_criteria>
  <done>REQUIREMENTS.md: 12 Phase-4 IDs marked Complete. ROADMAP.md Phase 4 marked complete. STATE.md advanced to Phase 5 ready. All D-19 INCONCLUSIVE entries documented.</done>
</task>

</tasks>

<verification>
```bash
# Pre-flight gates
cargo xtask regen-registry --check 2>&1 | tail -3
cargo xtask check-no-anyhow 2>&1 | tail -3
cargo xtask check-no-mul-add 2>&1 | tail -3

# Tier-1 full sweep
cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -10

# Full-matrix tier-2 (the main gate)
cargo xtask validate --backend cpu --order 3 --filter '.*' 2>&1 | tail -20

# Contracted spot-check
cargo xtask validate --backend cpu --mode contracted --order 6 --filter 'slaterx,pbex,tpssx,m06x' 2>&1 | tail -10

# Alias canary
cargo test -p xcfun-eval test_camcompx_negative_weight test_b3lyp_additive_accumulation test_exx_parameter_overwrite 2>&1 | tail -10

# REQUIREMENTS completion
grep -c "Complete" .planning/REQUIREMENTS.md
grep -n "MGGA-01.*Complete\|ALIAS-01.*Complete\|MODE-03.*Complete" .planning/REQUIREMENTS.md | head -5
```
</verification>

<success_criteria>
- `cargo xtask validate --backend cpu --order 3 --filter '.*'` exits 0 (or with only documented D-19 INCONCLUSIVE entries, all signed off).
- Contracted mode verified at orders 5/6 for representative functionals.
- All 46 alias canary tests GREEN (including negative-weight camcompx).
- REQUIREMENTS.md: 12 Phase-4 IDs marked Complete.
- ROADMAP.md: Phase 4 marked complete.
- STATE.md: Phase 5 ready, all Phase-4 D-19 forwards documented.
- `validation/report.html` and `validation/report.jsonl` committed as Phase-4 artifacts.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Planning artifact updates | Pure documentation updates; no code execution boundary |
| Validation runner (xtask validate) | Runs cc-compiled C++ ref + Rust eval; both on the same machine; no network; trusted input |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-06-01 | Repudiation | Undocumented D-19 entries forwarded without sign-off | mitigate | All new D-19 INCONCLUSIVE entries are explicitly listed in STATE.md with functional name, max_rel_err, and root cause hypothesis. Phase 6 must resolve or accept each entry individually. |
| T-04-06-02 | Information Disclosure | None | accept | Validation artifacts are committed to git (open source project). No confidential data in report.jsonl or report.html. |
| T-04-06-03 | Tampering | report.jsonl/html manually edited to hide failures | mitigate | The xtask validate command regenerates report artifacts deterministically from seed + C++ reference. A divergent committed report would be detectable by re-running the command in CI. |

No new code attack surface in this sign-off plan.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-06-SUMMARY.md`
</output>
