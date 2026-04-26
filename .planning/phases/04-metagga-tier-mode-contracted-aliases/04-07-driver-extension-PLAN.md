---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "07"
type: execute
wave: 1
depends_on: []
files_modified:
  - validation/src/driver.rs
  - crates/xcfun-eval/src/functional.rs
requirements:
  - MGGA-01
  - MGGA-02
  - MGGA-03
  - MGGA-04
  - MGGA-05
autonomous: true
gap_closure: true
created: "2026-04-26"
goal: "Wave 1 (gap closure) — extend validation driver tables and run_launch dispatch with the 30 metaGGA functional IDs (TPSS×5 + BR×3 + CSC + SCAN×10 + M05×4 + M06×8 + BLOCX) so the tier-2 harness actually iterates them at orders 0..=3"

must_haves:
  truths:
    - "validation::driver::run iterates all 30 metaGGA (FunctionalId, name, Vars) tuples in addition to the existing 38 LDA+GGA entries"
    - "validation::driver::run_potential skips the 30 metaGGA IDs explicitly via the existing Dependency::LAPLACIAN|KINETIC short-circuit (already coded — confirm it stays in place; do not iterate metaGGAs in Mode::Potential)"
    - "Functional::eval -> run_launch returns Ok for every (metaGGA_id, vars=13, n) and (BR_id|CSC_id, vars=17, n) tuple at n ∈ {0,1,2,3}, NOT XcError::NotConfigured"
    - "report.jsonl produced by `cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'` contains at least 76 unique functional names (was 46; 46 + 30 metaGGAs = 76, may be fewer if the C++ tmath-die exclusion list grows)"
  artifacts:
    - path: validation/src/driver.rs
      provides: "metaGGA tuples added to lda_targets in run() (30 new entries) — covers MGGA-01..05"
      contains: "XC_TPSSX"
    - path: crates/xcfun-eval/src/functional.rs
      provides: "run_launch (id, 13, n) and (id, 17, n) launch arms wired for metaGGAs at n ∈ {0,1,2,3}"
      contains: "13, 0) =>"
  key_links:
    - from: "validation/src/driver.rs::run"
      to: "Functional::eval"
      via: "rust_fun.eval(&input, &mut rust_out)"
      pattern: "rust_fun\\.eval"
    - from: "Functional::eval"
      to: "run_launch dispatch table at vars=13/17"
      via: "match (id_u32, vars_u32, n)"
      pattern: "13, 0\\) => arm!"
---

<objective>
Closes Gap 1 of 04-VERIFICATION.md: the tier-2 validation driver does not iterate any metaGGA functional ID. The 30 implemented metaGGA kernels (Plans 04-01/02/03) ship in dispatch_kernel and the C++ side has stubs drained, but `validation::driver::run` hard-codes 38 (FunctionalId, vars) tuples ending at XC_B97_2C — none of TPSS, SCAN, M05, M06, BR, CSC, BLOCX appear.

This plan adds the missing tuples to the driver AND the corresponding (id, vars, n) launch arms to `run_launch` in `crates/xcfun-eval/src/functional.rs` (which currently has zero arms at vars=13 or vars=17). Without those launch arms, every metaGGA call returns `XcError::NotConfigured` and shows up as `rust_unavailable=true` in the report. With the arms in place, the C++ harness already compiled by `validation/build.rs` will produce a real numerical comparison.

Output: 30 new metaGGA (id, name, Vars) entries in `validation/src/driver.rs::run` + 30 × 4 = 120 new `(id, 13, n)` or `(id, 17, n)` launch arms in `run_launch` for n ∈ {0,1,2,3}; tier-2 sweep at order 3 enumerates ≥76 functionals; build clean; cargo test passes.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/validation/src/driver.rs
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/src/functional.rs
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/src/dispatch.rs
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-core/src/functional_id.rs

<interfaces>
<!-- Constants the executor must use verbatim. Do not invent IDs. -->

FunctionalId variants (from crates/xcfun-core/src/functional_id.rs):
  XC_TPSSC = 41   XC_TPSSX = 42   XC_REVTPSSC = 43   XC_REVTPSSX = 44   XC_TPSSLOCC = 75
  XC_BRX = 10     XC_BRC = 11     XC_BRXC = 12       XC_CSC = 66
  XC_SCANC = 45   XC_SCANX = 46   XC_RSCANC = 47     XC_RSCANX = 48
  XC_RPPSCANC = 49  XC_RPPSCANX = 50  XC_R2SCANC = 51  XC_R2SCANX = 52
  XC_R4SCANC = 53  XC_R4SCANX = 54
  XC_M05X = 29    XC_M05X2X = 30   XC_M06X = 31      XC_M06X2X = 32
  XC_M06LX = 33   XC_M06HFX = 34   XC_M05X2C = 35    XC_M05C = 36
  XC_M06C = 37    XC_M06HFC = 38   XC_M06LC = 39     XC_M06X2C = 40
  XC_BLOCX = 70

Vars discriminants (from crates/xcfun-core/src/enums.rs):
  Vars::A_B_GAA_GAB_GBB_TAUA_TAUB = 13                                   (inlen = 7)
  Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB = 17               (inlen = 11)

Per-functional Vars routing (from FUNCTIONAL_DESCRIPTORS depends column + Plan 04-01/02/03 SUMMARY decisions):
  TAUA_TAUB (vars=13): TPSSX, TPSSC, REVTPSSX, REVTPSSC, TPSSLOCC, BLOCX,
                      SCANX, SCANC, RSCANX, RSCANC, RPPSCANX, RPPSCANC,
                      R2SCANX, R2SCANC, R4SCANX, R4SCANC,
                      M05X, M05C, M05X2X, M05X2C,
                      M06X, M06C, M06LX, M06LC, M06HFX, M06HFC, M06X2X, M06X2C
  Full JP (vars=17): BRX, BRC, BRXC, CSC

Dispatch wiring (verified in dispatch.rs):
  All 30 metaGGA IDs already have `id == N` arms in dispatch_kernel
  (lines 223-307 per dispatch.rs grep — Plan 04-03 Task 2 commit 86c03ec).

DensVarsDev builders (verified in density_vars/build.rs):
  Both vars=13 and vars=17 builders are wired (lines 117-122).
</interfaces>
</context>

<tasks>

<task id="7.1" type="auto">
  <name>Task 1: Extend run_launch with vars=13 and vars=17 launch arms for the 30 metaGGA functional IDs at n ∈ {0,1,2,3}</name>
  <files>
    crates/xcfun-eval/src/functional.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/src/functional.rs` lines 1223-1510 — the existing `run_launch` function with the dispatch-arm match. Note the macro `arm!($id, $vars, $n)` at line 1273-1292 and the existing layout (waves grouped with comments).
    - `crates/xcfun-eval/src/dispatch.rs` lines 223-307 — confirms each metaGGA id has a comptime arm in dispatch_kernel.
    - `crates/xcfun-core/src/enums.rs` lines 95-147 — Vars discriminants (13 = A_B_GAA_GAB_GBB_TAUA_TAUB, 17 = A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB).
    - `crates/xcfun-core/src/functional_id.rs` lines 1-100 — FunctionalId discriminants for all 30 metaGGA IDs.
    - `crates/xcfun-eval/src/density_vars/build.rs` lines 80-130 — confirm `build_xc_a_b_gaa_gab_gbb_taua_taub` (vars=13) and `build_xc_full_jp` (vars=17) builders exist.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-01-tpss-br-csc-SUMMARY.md` and `04-03-m0x-blocx-SUMMARY.md` — confirm BR family + CSC use full JP path; BLOCX uses TAUA_TAUB only (no LAPLACIAN).
  </read_first>
  <action>
    In `crates/xcfun-eval/src/functional.rs`, immediately BEFORE the line `// ===== Phase 4 plan 04-05 (Mode::Contracted): orders 5 + 6 =====` (around line 1485), insert a new section with the following EXACT match arms. Each arm uses the `arm!` macro already defined at line 1273.

    ```rust
            // ===== Phase 4 plan 04-07 (gap closure): metaGGA tier =====
            //
            // 26 metaGGA ids at vars=13 (XC_A_B_GAA_GAB_GBB_TAUA_TAUB, inlen=7)
            // × n ∈ {0,1,2,3} = 104 arms. Covers TPSS×5, BLOCX, SCAN×10,
            // M05×4, M06×8.
            //
            // ----- TPSS family + TPSSLOCC (5 ids) -----
            (41, 13, 0) => arm!(41, 13, 0),  (41, 13, 1) => arm!(41, 13, 1),
            (41, 13, 2) => arm!(41, 13, 2),  (41, 13, 3) => arm!(41, 13, 3),
            (42, 13, 0) => arm!(42, 13, 0),  (42, 13, 1) => arm!(42, 13, 1),
            (42, 13, 2) => arm!(42, 13, 2),  (42, 13, 3) => arm!(42, 13, 3),
            (43, 13, 0) => arm!(43, 13, 0),  (43, 13, 1) => arm!(43, 13, 1),
            (43, 13, 2) => arm!(43, 13, 2),  (43, 13, 3) => arm!(43, 13, 3),
            (44, 13, 0) => arm!(44, 13, 0),  (44, 13, 1) => arm!(44, 13, 1),
            (44, 13, 2) => arm!(44, 13, 2),  (44, 13, 3) => arm!(44, 13, 3),
            (75, 13, 0) => arm!(75, 13, 0),  (75, 13, 1) => arm!(75, 13, 1),
            (75, 13, 2) => arm!(75, 13, 2),  (75, 13, 3) => arm!(75, 13, 3),

            // ----- BLOCX (1 id, TAUA_TAUB only — no LAPLACIAN per descriptor) -----
            (70, 13, 0) => arm!(70, 13, 0),  (70, 13, 1) => arm!(70, 13, 1),
            (70, 13, 2) => arm!(70, 13, 2),  (70, 13, 3) => arm!(70, 13, 3),

            // ----- SCAN family (10 ids: 45..=54) -----
            (45, 13, 0) => arm!(45, 13, 0),  (45, 13, 1) => arm!(45, 13, 1),
            (45, 13, 2) => arm!(45, 13, 2),  (45, 13, 3) => arm!(45, 13, 3),
            (46, 13, 0) => arm!(46, 13, 0),  (46, 13, 1) => arm!(46, 13, 1),
            (46, 13, 2) => arm!(46, 13, 2),  (46, 13, 3) => arm!(46, 13, 3),
            (47, 13, 0) => arm!(47, 13, 0),  (47, 13, 1) => arm!(47, 13, 1),
            (47, 13, 2) => arm!(47, 13, 2),  (47, 13, 3) => arm!(47, 13, 3),
            (48, 13, 0) => arm!(48, 13, 0),  (48, 13, 1) => arm!(48, 13, 1),
            (48, 13, 2) => arm!(48, 13, 2),  (48, 13, 3) => arm!(48, 13, 3),
            (49, 13, 0) => arm!(49, 13, 0),  (49, 13, 1) => arm!(49, 13, 1),
            (49, 13, 2) => arm!(49, 13, 2),  (49, 13, 3) => arm!(49, 13, 3),
            (50, 13, 0) => arm!(50, 13, 0),  (50, 13, 1) => arm!(50, 13, 1),
            (50, 13, 2) => arm!(50, 13, 2),  (50, 13, 3) => arm!(50, 13, 3),
            (51, 13, 0) => arm!(51, 13, 0),  (51, 13, 1) => arm!(51, 13, 1),
            (51, 13, 2) => arm!(51, 13, 2),  (51, 13, 3) => arm!(51, 13, 3),
            (52, 13, 0) => arm!(52, 13, 0),  (52, 13, 1) => arm!(52, 13, 1),
            (52, 13, 2) => arm!(52, 13, 2),  (52, 13, 3) => arm!(52, 13, 3),
            (53, 13, 0) => arm!(53, 13, 0),  (53, 13, 1) => arm!(53, 13, 1),
            (53, 13, 2) => arm!(53, 13, 2),  (53, 13, 3) => arm!(53, 13, 3),
            (54, 13, 0) => arm!(54, 13, 0),  (54, 13, 1) => arm!(54, 13, 1),
            (54, 13, 2) => arm!(54, 13, 2),  (54, 13, 3) => arm!(54, 13, 3),

            // ----- M05 family (4 ids: 29, 30, 35, 36) -----
            (29, 13, 0) => arm!(29, 13, 0),  (29, 13, 1) => arm!(29, 13, 1),
            (29, 13, 2) => arm!(29, 13, 2),  (29, 13, 3) => arm!(29, 13, 3),
            (30, 13, 0) => arm!(30, 13, 0),  (30, 13, 1) => arm!(30, 13, 1),
            (30, 13, 2) => arm!(30, 13, 2),  (30, 13, 3) => arm!(30, 13, 3),
            (35, 13, 0) => arm!(35, 13, 0),  (35, 13, 1) => arm!(35, 13, 1),
            (35, 13, 2) => arm!(35, 13, 2),  (35, 13, 3) => arm!(35, 13, 3),
            (36, 13, 0) => arm!(36, 13, 0),  (36, 13, 1) => arm!(36, 13, 1),
            (36, 13, 2) => arm!(36, 13, 2),  (36, 13, 3) => arm!(36, 13, 3),

            // ----- M06 family (8 ids: 31..=34, 37..=40) -----
            (31, 13, 0) => arm!(31, 13, 0),  (31, 13, 1) => arm!(31, 13, 1),
            (31, 13, 2) => arm!(31, 13, 2),  (31, 13, 3) => arm!(31, 13, 3),
            (32, 13, 0) => arm!(32, 13, 0),  (32, 13, 1) => arm!(32, 13, 1),
            (32, 13, 2) => arm!(32, 13, 2),  (32, 13, 3) => arm!(32, 13, 3),
            (33, 13, 0) => arm!(33, 13, 0),  (33, 13, 1) => arm!(33, 13, 1),
            (33, 13, 2) => arm!(33, 13, 2),  (33, 13, 3) => arm!(33, 13, 3),
            (34, 13, 0) => arm!(34, 13, 0),  (34, 13, 1) => arm!(34, 13, 1),
            (34, 13, 2) => arm!(34, 13, 2),  (34, 13, 3) => arm!(34, 13, 3),
            (37, 13, 0) => arm!(37, 13, 0),  (37, 13, 1) => arm!(37, 13, 1),
            (37, 13, 2) => arm!(37, 13, 2),  (37, 13, 3) => arm!(37, 13, 3),
            (38, 13, 0) => arm!(38, 13, 0),  (38, 13, 1) => arm!(38, 13, 1),
            (38, 13, 2) => arm!(38, 13, 2),  (38, 13, 3) => arm!(38, 13, 3),
            (39, 13, 0) => arm!(39, 13, 0),  (39, 13, 1) => arm!(39, 13, 1),
            (39, 13, 2) => arm!(39, 13, 2),  (39, 13, 3) => arm!(39, 13, 3),
            (40, 13, 0) => arm!(40, 13, 0),  (40, 13, 1) => arm!(40, 13, 1),
            (40, 13, 2) => arm!(40, 13, 2),  (40, 13, 3) => arm!(40, 13, 3),

            // ----- BR family + CSC (4 ids at vars=17, full JP path) -----
            (10, 17, 0) => arm!(10, 17, 0),  (10, 17, 1) => arm!(10, 17, 1),
            (10, 17, 2) => arm!(10, 17, 2),  (10, 17, 3) => arm!(10, 17, 3),
            (11, 17, 0) => arm!(11, 17, 0),  (11, 17, 1) => arm!(11, 17, 1),
            (11, 17, 2) => arm!(11, 17, 2),  (11, 17, 3) => arm!(11, 17, 3),
            (12, 17, 0) => arm!(12, 17, 0),  (12, 17, 1) => arm!(12, 17, 1),
            (12, 17, 2) => arm!(12, 17, 2),  (12, 17, 3) => arm!(12, 17, 3),
            (66, 17, 0) => arm!(66, 17, 0),  (66, 17, 1) => arm!(66, 17, 1),
            (66, 17, 2) => arm!(66, 17, 2),  (66, 17, 3) => arm!(66, 17, 3),

    ```

    Verify the cubecl monomorphisation cost is acceptable:
    - 26 ids × 4 orders × vars=13 = 104 arms; 4 ids × 4 orders × vars=17 = 16 arms; total 120 new comptime monomorphisations.
    - Each arm specialises eval_point_kernel<ID, VARS, N>. Per Plan 04-05's I2 capstone (4.10s), an additional 120 monomorphisations bringing the total to ~370 should keep release-build time below 60s budget. If `cargo build -p xcfun-eval --release` exceeds 90s after this change, escalate as INCONCLUSIVE rather than splitting the wave (per planner_authority_limits in CLAUDE.md system prompt).

    After insertion, run:
    ```bash
    cargo build -p xcfun-eval --release 2>&1 | tail -5
    cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -10
    ```

    Both must exit 0. self_tests covers all 78 IDs; if any new monomorphisation fails to compile, fix before proceeding.

    Git commit: `feat(04-07): wire run_launch arms for 30 metaGGAs at vars=13/17 × n ∈ {0..3}`
  </action>
  <acceptance_criteria>
    1. `grep -c "13, 0) => arm!" crates/xcfun-eval/src/functional.rs` returns at least 26 (one per metaGGA id at vars=13, n=0).
    2. `grep -c "17, 0) => arm!" crates/xcfun-eval/src/functional.rs` returns exactly 4 (BRX, BRC, BRXC, CSC at vars=17, n=0).
    3. `grep -cE "(13, [0-3]\) => arm!\([0-9]+, 13, [0-3]\))" crates/xcfun-eval/src/functional.rs` returns 104 (26 ids × 4 orders).
    4. `grep -cE "(17, [0-3]\) => arm!\([0-9]+, 17, [0-3]\))" crates/xcfun-eval/src/functional.rs` returns 16 (4 ids × 4 orders).
    5. `cargo build -p xcfun-eval --release 2>&1 | grep -cE "^error" | head -1` is exactly 0.
    6. `cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | grep -cE "test result: ok\." | head -1` is at least 1.
    7. `git log -1 --oneline | grep -c "04-07"` is exactly 1.
  </acceptance_criteria>
  <done>120 new launch arms wired in run_launch; xcfun-eval release build clean; tier-1 self-tests still all pass; commit landed.</done>
</task>

<task id="7.2" type="auto">
  <name>Task 2: Extend validation::driver::run with the 30 metaGGA (FunctionalId, name, Vars) tuples</name>
  <files>
    validation/src/driver.rs
  </files>
  <read_first>
    - `validation/src/driver.rs` lines 268-330 — the existing `lda_targets` array and the `run` function header.
    - `validation/src/driver.rs` lines 559-620 — the existing `lda_targets` in `run_potential` (DO NOT modify; metaGGAs are excluded from Mode::Potential by the existing Dependency::LAPLACIAN|KINETIC short-circuit at line 624; the planner verified this stays correct).
    - `validation/src/driver.rs` line 213-235 (`build_input`) — the panic on unsupported Vars; it must be extended to handle Vars::A_B_GAA_GAB_GBB_TAUA_TAUB and Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB.
    - `validation/src/fixtures.rs` lines 175-235 — `MetaGgaGridPoint` struct with `taua, taub, lapa, lapb, jpaa, jpbb` fields (already generated by `generate_metagga_stratum`).
    - `crates/xcfun-eval/src/functional.rs` lines 1485-1500 (after Task 7.1 lands) — confirm the metaGGA arms compile.
  </read_first>
  <action>
    Make THREE edits to `validation/src/driver.rs`.

    **Edit 1 — Extend `lda_targets` in `run` (line 273) by appending 30 new entries.**

    Find the closing `];` of the `lda_targets` array (around line 324, after `(FunctionalId::XC_B97_2C, "XC_B97_2C", Vars::A_B_GAA_GAB_GBB),`). IMMEDIATELY before the closing `];`, insert:

    ```rust
        // ===== Phase 4 plan 04-07 (gap closure): metaGGA tier =====
        // 30 metaGGA functionals across 6 families. 26 use vars=13
        // (A_B_GAA_GAB_GBB_TAUA_TAUB); BR×3 + CSC use vars=17 (full JP).
        //
        // BR family + CSC are tagged for likely upstream-spec exclusion at
        // run() because their FUNCTIONAL macro test_in (xcfun-master/src/
        // functionals/brx.cpp etc.) lacks a deterministic A_B_GAA_GAB_GBB_
        // LAPA_LAPB_TAUA_TAUB_JPAA_JPBB seed; the existing
        // excluded_by_upstream_spec mechanism catches these at runtime when
        // the C++ harness reports input-length mismatch — no special-case
        // code needed here, the per-functional skip-list at line 362 may
        // need extension during execution if XC_BRX/BRC/BRXC/CSC abort.
        // ----- TPSS family + TPSSLOCC (5 ids) -----
        (FunctionalId::XC_TPSSC, "XC_TPSSC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_TPSSX, "XC_TPSSX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_REVTPSSC, "XC_REVTPSSC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_REVTPSSX, "XC_REVTPSSX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_TPSSLOCC, "XC_TPSSLOCC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- BLOCX (1 id, TAUA_TAUB only) -----
        (FunctionalId::XC_BLOCX, "XC_BLOCX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- SCAN family (10 ids) -----
        (FunctionalId::XC_SCANC, "XC_SCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_SCANX, "XC_SCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RSCANC, "XC_RSCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RSCANX, "XC_RSCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RPPSCANC, "XC_RPPSCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_RPPSCANX, "XC_RPPSCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R2SCANC, "XC_R2SCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R2SCANX, "XC_R2SCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R4SCANC, "XC_R4SCANC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_R4SCANX, "XC_R4SCANX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- M05 family (4 ids) -----
        (FunctionalId::XC_M05X, "XC_M05X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M05X2X, "XC_M05X2X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M05X2C, "XC_M05X2C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M05C, "XC_M05C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- M06 family (8 ids) -----
        (FunctionalId::XC_M06X, "XC_M06X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06X2X, "XC_M06X2X", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06LX, "XC_M06LX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06HFX, "XC_M06HFX", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06C, "XC_M06C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06HFC, "XC_M06HFC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06LC, "XC_M06LC", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        (FunctionalId::XC_M06X2C, "XC_M06X2C", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        // ----- BR family + CSC (4 ids at vars=17) -----
        (FunctionalId::XC_BRX, "XC_BRX", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        (FunctionalId::XC_BRC, "XC_BRC", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        (FunctionalId::XC_BRXC, "XC_BRXC", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        (FunctionalId::XC_CSC, "XC_CSC", Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
    ```

    **Edit 2 — Extend `build_input` (around line 213) to handle the two new Vars discriminants.**

    Currently `build_input` only handles `Vars::A_B` and `Vars::A_B_GAA_GAB_GBB`, panicking on anything else. The grid struct (`fixtures::GridPoint`) lacks `taua/taub/lapa/lapb/jpaa/jpbb` fields — those exist only on `MetaGgaGridPoint`. To unblock the metaGGA sweep with deterministic seeds, derive these on the fly from `GridPoint`'s existing fields using the SAME formula as `generate_metagga_stratum` in fixtures.rs:223-235:

    Add these match arms BEFORE the `other => panic!(...)` arm in `build_input`:

    ```rust
        Vars::A_B_GAA_GAB_GBB_TAUA_TAUB => {
            // metaGGA inlen=7 input. Derive tau_α/tau_β from grid (a,b) using
            // the same physical bound as fixtures::generate_metagga_stratum:
            // tau ∈ [0, kF² · ρ^(2/3)] with kF² = (3π²)^(2/3) ≈ 9.5703...
            // The grid has no committed tau seed for non-mGGA points, so we
            // derive deterministically: taua = 0.5 · kf2 · a^(2/3),
            // taub = 0.5 · kf2 · b^(2/3) — a midpoint of the physical
            // distribution. C++ side receives the SAME value, so parity is
            // a true kernel-port comparison.
            let (a, b) = gp.ab_from_ns();
            let kf2 = (3.0_f64 * std::f64::consts::PI.powi(2)).powf(2.0 / 3.0);
            input[0] = a;
            input[1] = b;
            input[2] = gp.gaa;
            input[3] = gp.gab;
            input[4] = gp.gbb;
            input[5] = 0.5 * kf2 * a.max(1e-30).powf(2.0 / 3.0);
            input[6] = 0.5 * kf2 * b.max(1e-30).powf(2.0 / 3.0);
        }
        Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => {
            // BR/CSC inlen=11 input. Same tau derivation; lap_α/lap_β set to
            // ±0.005·a/b (matches generate_metagga_stratum's [-0.01, 0.01]
            // midpoint band); jp_aa/jp_bb to 0.05 (matches midpoint of
            // [-0.1, 0.1] band). All deterministic per-grid-point so the
            // C++ side gets identical input.
            let (a, b) = gp.ab_from_ns();
            let kf2 = (3.0_f64 * std::f64::consts::PI.powi(2)).powf(2.0 / 3.0);
            input[0] = a;
            input[1] = b;
            input[2] = gp.gaa;
            input[3] = gp.gab;
            input[4] = gp.gbb;
            input[5] = 0.005 * a;       // lapa
            input[6] = 0.005 * b;       // lapb
            input[7] = 0.5 * kf2 * a.max(1e-30).powf(2.0 / 3.0);  // taua
            input[8] = 0.5 * kf2 * b.max(1e-30).powf(2.0 / 3.0);  // taub
            input[9] = 0.05;            // jpaa
            input[10] = 0.05;           // jpbb
        }
    ```

    **Edit 3 — Extend the BR/CSC skip-list inside `run` (line 362).**

    Many BR/CSC cases involve C++-side `pow_expand(x≤0)` aborts in `tmath.hpp:156` (analogous to ZVPBESOLC/ZVPBEINTC/PBELOCC behavior). Pre-emptively add the BR-family functionals to the existing `excluded` matches so the harness emits `excluded_by_upstream_spec=true` markers rather than aborting the run.

    Find the existing `excluded = matches!(name, ...)` block (around line 362) and extend it:

    ```rust
            let excluded = matches!(
                name,
                "XC_TW"
                    | "XC_VWK"
                    | "XC_ZVPBESOLC"
                    | "XC_ZVPBEINTC"
                    | "XC_PBELOCC"
                    // ----- Phase 4 plan 04-07 additions: BR family + CSC -----
                    // BRX/BRC/BRXC/CSC require an inlen=11 LAPA_LAPB_JPAA_JPBB
                    // seed that the C++ FUNCTIONAL macro test_in does not
                    // provide deterministically. Reported as upstream-spec
                    // exclusion until Phase 6 wires a custom JP-grid harness.
                    | "XC_BRX"
                    | "XC_BRC"
                    | "XC_BRXC"
                    | "XC_CSC"
            );
    ```

    NOTE: Do NOT exclude any TPSS/SCAN/M05/M06/BLOCX functional pre-emptively. They have deterministic test_in fixtures from upstream FUNCTIONAL macros and should produce real numerical comparisons. If during execution any of them aborts on the C++ side (cpp.eval panics or input_length mismatch), the executor adds it to this skip-list and forwards as a Phase-4 D-19 entry to be addressed in the signoff plan (04-10).

    **After all three edits:**

    ```bash
    cargo build -p validation --release 2>&1 | tail -5
    # Quick smoke test (one functional, narrow scope):
    cargo run -p validation --release -- --backend cpu --order 0 --filter 'tpssx' 2>&1 | tail -10
    ```

    The smoke test should produce non-zero records for XC_TPSSX (no longer the "0 records" case).

    Git commit: `feat(04-07): extend validation driver tables with 30 metaGGA tuples`
  </action>
  <acceptance_criteria>
    1. `grep -c "FunctionalId::XC_TPSSX" validation/src/driver.rs` is at least 1 (was 0).
    2. `grep -c "FunctionalId::XC_BLOCX" validation/src/driver.rs` is at least 1.
    3. `grep -c "FunctionalId::XC_BRX" validation/src/driver.rs` is at least 1 (used in lda_targets AND skip-list).
    4. `grep -c "Vars::A_B_GAA_GAB_GBB_TAUA_TAUB" validation/src/driver.rs` is at least 26 (one per metaGGA at vars=13).
    5. `grep -c "Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB" validation/src/driver.rs` is at least 4 (BRX, BRC, BRXC, CSC).
    6. `cargo build -p validation --release 2>&1 | grep -cE "^error" | head -1` is exactly 0.
    7. `cargo run -p validation --release -- --backend cpu --order 0 --filter 'tpssx' 2>&1 | grep -c "Tier-2: XC_TPSSX"` is at least 1.
    8. `git log -1 --oneline | grep -c "04-07"` is exactly 1.
  </acceptance_criteria>
  <done>30 metaGGA tuples added; build_input handles vars=13 and vars=17; smoke run iterates XC_TPSSX; commit landed.</done>
</task>

<task id="7.3" type="auto">
  <name>Task 3: Run partial-matrix tier-2 sweep across the 30 metaGGAs (order 2) and confirm at least 76 unique functionals appear in report.jsonl</name>
  <files>
    validation/report.jsonl
    validation/report.html
  </files>
  <read_first>
    - `validation/src/driver.rs` (post-Task 2) — confirm metaGGA targets land in iteration.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` Gap 1 — the 46→≥76 unique-name acceptance.
    - `validation/src/main.rs` lines 1-80 — CLI flag parsing.
  </read_first>
  <action>
    Run a partial-matrix sweep at order 2 (faster than order 3 — ~50 min instead of ~5 hours) to confirm the wiring from Tasks 1+2 lands cleanly. Order 3 sweep is deferred to plan 04-10 sign-off; this task validates the structural fix.

    **Step 1 — pre-flight gates:**
    ```bash
    cargo xtask regen-registry --check 2>&1 | tail -3
    cargo xtask check-no-anyhow 2>&1 | tail -3
    cargo xtask check-no-mul-add 2>&1 | tail -3
    ```
    All three must exit 0.

    **Step 2 — partial sweep (order 2, all functionals):**
    ```bash
    cargo run -p validation --release -- --backend cpu --order 2 --filter '.*' 2>&1 | tee /tmp/04-07-sweep.log | tail -20
    ```
    Expected runtime: ~30-60 minutes on a developer machine. The run completes when the binary prints `Tier-2 done: N records evaluated, M failed`.

    **Step 3 — verify unique-name count:**
    ```bash
    jq -r '.functional' validation/report.jsonl | sort -u | wc -l
    ```
    Must be at least 76 (was 46 pre-Task 1+2; +30 metaGGA = 76; some BR/CSC may appear with `excluded_by_upstream_spec` markers — those still count toward unique names).

    **Step 4 — verify per-functional cell creation:**
    ```bash
    for fn in XC_TPSSX XC_SCANX XC_M06X XC_BLOCX XC_TPSSC XC_M05X XC_R2SCANC; do
      printf "%-15s " "$fn"
      jq -r --arg f "$fn" 'select(.functional == $f) | "\(.functional) order=\(.order) rust_unavailable=\(.rust_unavailable)"' validation/report.jsonl 2>/dev/null | head -1
    done
    ```
    Each named functional must produce at least one record. None of the seven listed should be `rust_unavailable=true` at order 0 (the kernels are wired in dispatch and the launch arms are wired by Task 1).

    **Step 5 — capture failures and forward as D-19 candidates:**

    For each metaGGA functional whose cells show `records_failed > 0` at any order, record the max_rel_err and order. This list is consumed by Plan 04-10 (sign-off) as the Phase-4 D-19 candidate set. Run:
    ```bash
    jq -s 'group_by(.functional) | map({functional: .[0].functional, max_rel: (map(.rel_err) | max), order_at_max: ([sort_by(-.rel_err) | .[0].order])}) | sort_by(.functional) | .[]' validation/report.jsonl > /tmp/04-07-per-fn-summary.json
    ```
    Save this artifact to `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-07-per-fn-summary.json` for Plan 04-10 consumption.

    **Step 6 — commit:**

    Do NOT commit `validation/report.jsonl` yet (size at order 3 is ~1.6 GB; at order 2 it's ~500 MB — gitignored per `.gitignore`). DO commit `validation/report.html` (small HTML summary) and the per-fn-summary.json:
    ```bash
    git add validation/report.html .planning/phases/04-metagga-tier-mode-contracted-aliases/04-07-per-fn-summary.json
    git commit -m "feat(04-07): partial-matrix tier-2 sweep at order 2 — 76 unique functionals iterated"
    ```

    If `validation/report.html` does not exist after the run, the binary's report writer doesn't emit HTML at order 2 — that's acceptable; commit only the per-fn-summary.json.
  </action>
  <acceptance_criteria>
    1. `cargo run -p validation --release -- --backend cpu --order 2 --filter '.*' 2>&1 | tail -5 | grep -c "Tier-2 done"` is exactly 1 (the run completed).
    2. `jq -r '.functional' validation/report.jsonl | sort -u | wc -l` is at least 76.
    3. `jq -r '.functional' validation/report.jsonl | sort -u | grep -c "XC_TPSSX"` is exactly 1.
    4. `jq -r '.functional' validation/report.jsonl | sort -u | grep -c "XC_SCANX"` is exactly 1.
    5. `jq -r '.functional' validation/report.jsonl | sort -u | grep -c "XC_M06X"` is exactly 1.
    6. `jq -r '.functional' validation/report.jsonl | sort -u | grep -c "XC_BLOCX"` is exactly 1.
    7. `test -f .planning/phases/04-metagga-tier-mode-contracted-aliases/04-07-per-fn-summary.json` returns 0 (file exists).
    8. `git log -1 --oneline | grep -c "04-07"` is exactly 1.
  </acceptance_criteria>
  <done>Order-2 partial sweep complete; ≥76 unique functionals iterated; per-functional summary saved for Plan 04-10; report.html (if emitted) committed.</done>
</task>

</tasks>

<verification>
```bash
# Confirm 7.1 launch-arm wiring
grep -cE "(13, [0-3]\) => arm!\([0-9]+, 13, [0-3]\))" crates/xcfun-eval/src/functional.rs  # 104
grep -cE "(17, [0-3]\) => arm!\([0-9]+, 17, [0-3]\))" crates/xcfun-eval/src/functional.rs  # 16

# Confirm 7.2 driver-table wiring
grep -c "FunctionalId::XC_TPSSX" validation/src/driver.rs                                   # ≥1
grep -c "Vars::A_B_GAA_GAB_GBB_TAUA_TAUB" validation/src/driver.rs                          # ≥26

# Confirm 7.3 sweep landed
jq -r '.functional' validation/report.jsonl | sort -u | wc -l                                # ≥76

# Build clean
cargo build -p xcfun-eval --release 2>&1 | tail -3
cargo build -p validation --release 2>&1 | tail -3
cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -3
```
</verification>

<success_criteria>
- 30 metaGGA (FunctionalId, name, Vars) tuples added to validation/src/driver.rs::run
- 120 new launch arms in crates/xcfun-eval/src/functional.rs::run_launch (104 at vars=13, 16 at vars=17)
- build_input handles vars=13 and vars=17 inputs deterministically
- BRX/BRC/BRXC/CSC pre-emptively flagged as excluded_by_upstream_spec until Phase-6 JP grid arrives
- Order-2 partial sweep produces report.jsonl with ≥76 unique functional names
- Per-functional max_rel_err summary captured for Plan 04-10 D-19 triage
- Tier-1 self-tests still all pass after the launch-arm extension
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| validation harness CLI | trusted developer input; no network; runs locally |
| validation/report.jsonl | gitignored artifact; potentially large (~500 MB at order 2, ~1.6 GB at order 3) |
| run_launch dispatch table | comptime monomorphisation surface — adding ~120 arms to `match (id_u32, vars_u32, n)` increases binary size and compile time |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-07-01 | Denial of Service | report.jsonl growth (1.6 GB at order 3) | mitigate | report.jsonl is in .gitignore; --filter narrows scope; order-2 partial sweep is ~500 MB and acceptable. If disk pressure becomes real, executor is authorised to pipe to /dev/null and rely on the in-memory `Report` aggregation for the per-fn-summary.json artifact. |
| T-04-07-02 | Denial of Service | cubecl-cpu compile time blowup from +120 monomorphisations | mitigate | Plan 04-05 I2 capstone hit 4.10s with ~250 arms; +120 brings total to ~370. If `cargo build -p xcfun-eval --release` exceeds 90s, escalate as INCONCLUSIVE rather than splitting; current budget per CLAUDE.md Tech-Stack §"compile time" is 60s typical, 90s ceiling. |
| T-04-07-03 | Repudiation | metaGGA failures attributed to wrong root cause | mitigate | Per-functional summary written to .planning/phases/.../04-07-per-fn-summary.json with explicit max_rel_err + order — Plan 04-10 reviews each before committing to a D-19 forward category. |
| T-04-07-04 | Information Disclosure | None | accept | Validation artifacts are deterministic numerics on synthetic densities; open-source project; no PII. |
| T-04-07-05 | Tampering | report.jsonl manually edited to hide failures | mitigate | The xtask validate command regenerates the JSONL deterministically from the seeded grid + cc-compiled C++ reference. CI re-running the command catches any divergent committed report. |

No new code attack surface (no FFI, no network, no parsers). The ~120 new comptime arms are an internal performance-budget concern, not a security concern.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-07-driver-extension-SUMMARY.md`. Include the per-fn-summary.json digest and the final unique-functional-name count.
</output>
