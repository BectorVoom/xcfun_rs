---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "09"
type: execute
wave: 2
depends_on:
  - "07"
files_modified:
  - crates/xcfun-eval/src/functional.rs
  - crates/xcfun-eval/tests/contracted_cross_mode.rs
  - .planning/REQUIREMENTS.md
requirements:
  - MODE-03
autonomous: true
gap_closure: true
created: "2026-04-26"
goal: "Wave 2 (gap closure) — extend the existing contracted_cross_mode integration test to cover at least one metaGGA exemplar per family (TPSSX, SCANX, M06X) at orders 0..=4 cross-checked against Mode::PartialDerivatives; explicitly forward orders 5/6 metaGGA Contracted to Phase 6 (already a Plan 04-05 D-19 forward — reinforce, do not re-investigate)"

must_haves:
  truths:
    - "contracted_cross_mode integration test (`crates/xcfun-eval/tests/contracted_cross_mode.rs`) covers at least 3 metaGGA exemplars (TPSSX from TPSS family, SCANX or R2SCANX from SCAN family, M06X from M06 family) at orders 0..=4, all GREEN at strict 1e-12"
    - "Plan 04-05's D-19 forward for ctaylor_compose/multo at N≥4 is explicitly cited as the reason metaGGA orders 5/6 are NOT in scope here (no new investigation)"
    - "MODE-03 in REQUIREMENTS.md is updated to reflect Phase-4-actual coverage: orders 0..=4 verified for SLATERX (LDA), PBEX (GGA), and at least 3 metaGGAs; orders 5/6 forwarded to Phase 6"
  artifacts:
    - path: crates/xcfun-eval/tests/contracted_cross_mode.rs
      provides: "metaGGA cross-mode tests at orders 0..=4 for TPSSX, SCANX, M06X (and family siblings if budget allows)"
      contains: "tpssx"
    - path: .planning/REQUIREMENTS.md
      provides: "MODE-03 line updated to reflect actual orders/families covered + explicit Phase-6 forward for orders 5/6"
      contains: "MODE-03"
  key_links:
    - from: "contracted_cross_mode test"
      to: "run_launch (id, vars=13, n) arms wired by Plan 04-07"
      via: "Functional::eval at Mode::Contracted forwards through run_launch"
      pattern: "Mode::Contracted"
---

<objective>
Closes Gap 3 of 04-VERIFICATION.md: contracted spot-checks at orders 5/6 for metaGGA were unrunnable because the validation driver did not iterate metaGGAs (Plan 04-07 fixes this) AND because xcfun-ad's `ctaylor_compose`/`ctaylor_multo` only specialise N ∈ {0,1,2,3} (Plan 04-05 already documented this as a Phase-6 forward).

This plan does NOT re-investigate the orders-5/6 issue — that work belongs to Phase 6 (`xcfun-ad ctaylor_compose/multo N=4..=6 specialisations` — Plan 04-05 D-19 forward, verbatim quote from 04-05-mode-contracted-SUMMARY.md). Instead, this plan EXTENDS the existing `contracted_cross_mode` integration test (lines for SLATERX + PBEX) with metaGGA exemplars at orders 0..=4, where `Mode::Contracted` IS algorithmically equivalent to `Mode::PartialDerivatives` (per Plan 04-05's pack_for_contracted equivalence).

If Plan 04-07's run_launch arm extension to vars=13 succeeds, this test becomes runnable: same `pack_for_contracted` helper, same comparison, just at vars=13 + n ∈ {0,1,2,3,4} instead of vars=2/6.

Output: 3 new metaGGA cross-mode test cases at orders 0..=4 (12 test cells); MODE-03 in REQUIREMENTS.md updated with Phase-4-actual coverage + Phase-6 forward citation.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/REQUIREMENTS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-05-mode-contracted-SUMMARY.md
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/tests/contracted_cross_mode.rs
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/src/functional.rs

<interfaces>
<!-- Cross-mode parity at orders 0..=4 is structurally a re-packaging exercise. -->

From Plan 04-05 (commit reference: see SUMMARY):
- `pack_for_contracted(input: &[f64], order: u32) -> Vec<f64>` packs CTaylor seeds: slot l, coefficient[CNST] = input[l]; VARi seeds = 1.0 on slot 0.
- `Mode::Contracted` output at order N = `1 << N` doubles, indexed by bit-flag.
- For SLATERX (vars=2) + PBEX (vars=6), orders 0..=4 cross-check is trivially GREEN — coefficient at bit-pattern P equals the corresponding multi-index PartialDerivatives slot.
- Orders 5/6 hit the xcfun-ad N≥4 specialisation gap and are explicitly forwarded to Phase 6 (Plan 04-05 D-19 forward).

From Plan 04-07 (this gap-closure phase):
- run_launch arms wired for (id, vars=13, n) at n ∈ {0,1,2,3} + (id, vars=17, n) for BR/CSC.
- We need to ALSO add (TPSSX, 13, 4), (SCANX, 13, 4), (M06X, 13, 4) for the cross-mode test at order 4 — Plan 04-07 stops at n=3.
- Vars=17 (BR/CSC) is NOT scoped to this plan's cross-mode exemplars; BR family Contracted is a niche use-case not on the Phase-4 critical path.
</interfaces>
</context>

<tasks>

<task id="9.1" type="auto">
  <name>Task 1: Extend run_launch with (TPSSX, 13, 4), (SCANX, 13, 4), (M06X, 13, 4) launch arms — needed for the cross-mode test at order 4</name>
  <files>
    crates/xcfun-eval/src/functional.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/src/functional.rs` (POST-Plan-04-07) — the metaGGA section added by Plan 04-07 stops at n=3. Inspect lines around the comment `// ===== Phase 4 plan 04-07 (gap closure): metaGGA tier =====`.
    - `crates/xcfun-eval/src/functional.rs` lines 1485-1500 — the existing Plan 04-05 Contracted arms at (0, 2, 5/6) and (5, 6, 5/6).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-05-mode-contracted-SUMMARY.md` — confirms the orders-5/6 metaGGA xcfun-ad specialisation gap is a Phase-6 forward.
  </read_first>
  <action>
    Plan 04-07 added arms for `(metaGGA_id, 13, n)` at n ∈ {0,1,2,3}. Order-4 PartialDerivatives is supported per MODE-01 D-16 (Phase-3 plan 03-06 raised the cap to ≤ 4). The cross-mode test at order 4 requires the launch arm to exist.

    Add 3 new arms IMMEDIATELY AFTER the `(40, 13, 3)` line (the last M06 family arm at n=3 inserted by Plan 04-07):

    ```rust
            // ===== Phase 4 plan 04-09 (gap closure): metaGGA cross-mode order 4 =====
            // Three exemplars (TPSSX, SCANX, M06X — one per family) at n=4 to
            // unblock the contracted_cross_mode test at orders 0..=4. Orders 5/6
            // for these ids remain forwarded to Phase 6 per Plan 04-05 D-19
            // (xcfun-ad ctaylor_compose/multo N=4..=6 specialisations).
            (42, 13, 4) => arm!(42, 13, 4),  // XC_TPSSX
            (46, 13, 4) => arm!(46, 13, 4),  // XC_SCANX
            (31, 13, 4) => arm!(31, 13, 4),  // XC_M06X
    ```

    Build + verify:
    ```bash
    cargo build -p xcfun-eval --release 2>&1 | tail -3
    cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -3
    ```

    Both must exit 0.

    Commit:
    ```bash
    git commit -m "feat(04-09): wire (TPSSX, SCANX, M06X) × n=4 launch arms for cross-mode test"
    ```
  </action>
  <acceptance_criteria>
    1. `grep -cE "(42|46|31), 13, 4\) => arm!" crates/xcfun-eval/src/functional.rs` is exactly 3.
    2. `cargo build -p xcfun-eval --release 2>&1 | grep -cE "^error" | head -1` is exactly 0.
    3. `cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | grep -cE "test result: ok\." | head -1` is at least 1.
    4. `git log -1 --oneline | grep -c "04-09"` is exactly 1.
  </acceptance_criteria>
  <done>3 new launch arms at n=4 wired; build clean; tier-1 GREEN; commit landed.</done>
</task>

<task id="9.2" type="auto">
  <name>Task 2: Extend `contracted_cross_mode.rs` with metaGGA exemplars at orders 0..=4</name>
  <files>
    crates/xcfun-eval/tests/contracted_cross_mode.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/tests/contracted_cross_mode.rs` — full file. Extract: (a) the test fn naming convention; (b) the `pack_for_contracted` helper; (c) the per-element comparison loop; (d) the existing SLATERX + PBEX test bodies; (e) the assertion macro and tolerance.
    - `crates/xcfun-eval/src/functional.rs` (post-Task-9.1) — confirm (42, 13, 4), (46, 13, 4), (31, 13, 4) arms wired.
    - `crates/xcfun-eval/src/dispatch.rs` — confirm TPSSX (42), SCANX (46), M06X (31) all have dispatch_kernel arms.
  </read_first>
  <action>
    Add three new test functions to `crates/xcfun-eval/tests/contracted_cross_mode.rs` following the pattern established for SLATERX/PBEX. The test must:

    1. Pick a representative density input for vars=13: `[a=1.1, b=1.0, gaa=0.04, gab=0.005, gbb=0.05, taua=0.5, taub=0.45]` (matches the metaGGA grid order; the exact values don't matter as long as both modes get the same input).
    2. Run `Mode::PartialDerivatives` at each order in 0..=4 to get the multi-index Taylor coefficients.
    3. Run `Mode::Contracted` at the same order, with `pack_for_contracted` over the same input.
    4. Verify each contracted-output bit-flag-indexed coefficient equals the corresponding PartialDerivatives multi-index coefficient at strict 1e-12.

    Append three test functions:

    ```rust
    // ============================================================
    // Phase 4 plan 04-09 (gap closure) — metaGGA cross-mode parity
    // at orders 0..=4. Validates that Mode::Contracted ≡
    // Mode::PartialDerivatives bit-flag-indexed for at least one
    // exemplar per metaGGA family (TPSSX, SCANX, M06X).
    //
    // Orders 5/6 NOT covered here — Plan 04-05 D-19 forward
    // (xcfun-ad ctaylor_compose/multo N=4..=6 specialisations
    // required, Phase 6 prerequisite).
    // ============================================================

    /// metaGGA representative input — vars=13 (A_B_GAA_GAB_GBB_TAUA_TAUB),
    /// inlen=7. Values picked to be physically representative and away from
    /// regularize-clamp / low-density edge cases.
    const MGGA_INPUT: [f64; 7] = [1.1, 1.0, 0.04, 0.005, 0.05, 0.5, 0.45];

    #[test]
    fn test_contracted_tpssx_orders_0_to_4_cross_mode() {
        for order in 0_u32..=4 {
            let id = FunctionalId::XC_TPSSX;
            let weights: &'static [(FunctionalId, f64)] = Box::leak(Box::new([(id, 1.0)]));
            assert_cross_mode_parity(weights, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, order);
        }
    }

    #[test]
    fn test_contracted_scanx_orders_0_to_4_cross_mode() {
        for order in 0_u32..=4 {
            let id = FunctionalId::XC_SCANX;
            let weights: &'static [(FunctionalId, f64)] = Box::leak(Box::new([(id, 1.0)]));
            assert_cross_mode_parity(weights, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, order);
        }
    }

    #[test]
    fn test_contracted_m06x_orders_0_to_4_cross_mode() {
        for order in 0_u32..=4 {
            let id = FunctionalId::XC_M06X;
            let weights: &'static [(FunctionalId, f64)] = Box::leak(Box::new([(id, 1.0)]));
            assert_cross_mode_parity(weights, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, order);
        }
    }
    ```

    The `assert_cross_mode_parity` helper is the shared comparison fn already in this test file (per Plan 04-05). If it doesn't exist with that exact name, REFACTOR the existing per-order code in SLATERX/PBEX into a helper named `assert_cross_mode_parity(weights, vars, input, order)` BEFORE adding the metaGGA tests, so the new tests have a clean call surface.

    NOTE on input values for metaGGA: tau values must satisfy the physical bound `tau ≤ kF² · ρ^(2/3)` (where kF² ≈ 9.5703). For ρ_α = 1.1, the upper bound on tau_α is ≈ 9.5703 × 1.1^(2/3) ≈ 10.2. Our tau_α = 0.5 is well within bounds.

    Build + run:
    ```bash
    cargo test -p xcfun-eval --test contracted_cross_mode --features testing -- --nocapture 2>&1 | tail -20
    ```

    All 3 new tests must pass. If they fail, the most likely root cause is the metaGGA kernel's compose/multo behavior at order 4 hitting the same N≥4 specialisation issue that Plan 04-05 documented — in which case the test must be GATED at order ≤ 3 with an explicit `#[cfg(feature = "phase6_n4_supported")]` or a per-test scoped order range, AND a Phase-6 D-19 entry must be added to Plan 04-04's MODE-03 line for orders 4 metaGGA Contracted.

    Commit:
    ```bash
    git commit -m "test(04-09): metaGGA cross-mode parity at orders 0..=4 (TPSSX, SCANX, M06X)"
    ```
  </action>
  <acceptance_criteria>
    1. `grep -c "test_contracted_tpssx" crates/xcfun-eval/tests/contracted_cross_mode.rs` is exactly 1.
    2. `grep -c "test_contracted_scanx" crates/xcfun-eval/tests/contracted_cross_mode.rs` is exactly 1.
    3. `grep -c "test_contracted_m06x" crates/xcfun-eval/tests/contracted_cross_mode.rs` is exactly 1.
    4. `cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | grep -E "test_contracted_(tpssx|scanx|m06x)_orders_0_to_4_cross_mode .* ok" | wc -l` is exactly 3 (all three new tests pass).
    5. `cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | grep -cE "test result: ok\." | head -1` is at least 1.
    6. `git log -1 --oneline | grep -c "04-09"` is exactly 1.
  </acceptance_criteria>
  <done>3 new metaGGA cross-mode tests added; all GREEN at strict 1e-12 across orders 0..=4; commit landed. (If order-4 fails for one or more, scope test to order ≤ 3 + add Phase-6 D-19 entry — handled in Task 9.3.)</done>
</task>

<task id="9.3" type="auto">
  <name>Task 3: Update MODE-03 in REQUIREMENTS.md to reflect Phase-4-actual orders/families coverage + explicit Phase-6 forward citation</name>
  <files>
    .planning/REQUIREMENTS.md
  </files>
  <read_first>
    - `.planning/REQUIREMENTS.md` line 70 — current `MODE-03` entry (status: `[ ] **MODE-03**: Mode::Contracted supports orders 0..=6 with output layout matching the DOEVAL macro expansion in the C++ reference`).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-05-mode-contracted-SUMMARY.md` — Plan 04-05 D-19 forward citation.
    - The status of Task 9.2 — did all three tests pass at orders 0..=4? If yes, the MODE-03 entry can claim orders 0..=4 metaGGA verified. If no (order 4 fails), the entry must be scoped to orders 0..=3 metaGGA verified + new Phase-6 forward.
  </read_first>
  <action>
    Update the MODE-03 entry in `.planning/REQUIREMENTS.md` to reflect actual Phase-4 coverage AFTER Tasks 9.1 + 9.2. The entry transitions from `[ ] Pending` to either `[~] Partial` or `[x] Complete (with caveats)`.

    **If Task 9.2's three tests all GREEN at orders 0..=4:**

    Replace line 70 with:
    ```
    - [~] **MODE-03**: `Mode::Contracted` supports orders 0..=4 verified across LDA (XC_SLATERX), GGA (XC_PBEX), and metaGGA exemplars (XC_TPSSX, XC_SCANX, XC_M06X) at strict 1e-12 cross-mode parity (Plan 04-05 + Plan 04-09). Orders 5..=6 D-19 INCONCLUSIVE forwarded to Phase 6 — root cause: `xcfun-ad ctaylor_compose`/`ctaylor_multo` only specialise N ∈ {0,1,2,3}; N ≥ 4 falls through with no op. Phase 6 owns the libm-hybrid + AD-N4..=N6 work that lifts the limit. Cross-mode parity for orders 0..=4 IS algorithmically equivalent to Mode::PartialDerivatives by construction (pack_for_contracted ↔ multi-index re-packaging) and verified by 11 + 3 = 14 integration tests.
    ```

    **If Task 9.2 fails at order 4 (one or more tests fall back to order ≤ 3):**

    Replace line 70 with:
    ```
    - [~] **MODE-03**: `Mode::Contracted` supports orders 0..=3 verified across LDA (XC_SLATERX), GGA (XC_PBEX), and metaGGA exemplars (XC_TPSSX, XC_SCANX, XC_M06X) at strict 1e-12 cross-mode parity (Plan 04-05 + Plan 04-09). Order 4 cross-mode for metaGGAs hits the same xcfun-ad N≥4 specialisation gap (Plan 04-05 D-19 forward). Orders 5..=6 D-19 INCONCLUSIVE forwarded to Phase 6 — root cause: `xcfun-ad ctaylor_compose`/`ctaylor_multo` only specialise N ∈ {0,1,2,3}; N ≥ 4 falls through with no op. Phase 6 owns the libm-hybrid + AD-N4..=N6 work that lifts the limit.
    ```

    Either way, append a Phase-4 sub-bullet under the requirement (or as a footnote) recording the Plan 04-09 commit hash that lands the metaGGA exemplar tests.

    Commit:
    ```bash
    git add .planning/REQUIREMENTS.md
    git commit -m "docs(04-09): update MODE-03 to reflect Plan 04-09 metaGGA cross-mode coverage"
    ```
  </action>
  <acceptance_criteria>
    1. `grep -E "^- \[(x|~)\] \*\*MODE-03\*\*" .planning/REQUIREMENTS.md | head -1 | grep -c "Plan 04-09"` is exactly 1.
    2. `grep "MODE-03" .planning/REQUIREMENTS.md | grep -c "Phase 6"` is at least 1 (Phase-6 forward for orders 5/6 explicitly cited).
    3. `grep -E "MODE-03.*XC_TPSSX|MODE-03.*XC_SCANX|MODE-03.*XC_M06X" .planning/REQUIREMENTS.md | wc -l` is at least 1 (at least one metaGGA exemplar named in the MODE-03 line).
    4. `git log -1 --oneline | grep -c "04-09"` is exactly 1.
  </acceptance_criteria>
  <done>MODE-03 updated to reflect Phase-4-actual coverage; Phase-6 forward explicitly cited; commit landed.</done>
</task>

</tasks>

<verification>
```bash
# 9.1 launch arms
grep -cE "(42|46|31), 13, 4\) => arm!" crates/xcfun-eval/src/functional.rs   # 3

# 9.2 metaGGA cross-mode tests pass
cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | grep -E "test_contracted_(tpssx|scanx|m06x)_orders_0_to_4_cross_mode .* ok" | wc -l  # 3

# 9.3 MODE-03 updated
grep "MODE-03" .planning/REQUIREMENTS.md | grep -c "Plan 04-09"  # 1
grep "MODE-03" .planning/REQUIREMENTS.md | grep -c "Phase 6"     # ≥1

# Tier-1 still GREEN
cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -3
```
</verification>

<success_criteria>
- 3 new launch arms wired in run_launch for (TPSSX, SCANX, M06X) at vars=13 × n=4
- 3 new test functions in contracted_cross_mode.rs covering one exemplar per metaGGA family at orders 0..=4
- All 3 metaGGA cross-mode tests GREEN at strict 1e-12 (or scoped to ≤3 with Phase-6 entry if order 4 hits N≥4 limit)
- MODE-03 entry in REQUIREMENTS.md transitioned from Pending to Partial/Complete with caveats — explicit citation of Plan 04-05 + 04-09 + Phase-6 forward
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Test additions | New code in tests/ directory; no production-path changes apart from 3 new launch arms (n=4 for 3 ids — already structurally identical to Plan 04-07's n=3 work). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-09-01 | Repudiation | Test passes "trivially" because Mode::Contracted falls back to PartialDerivatives without exercising the contracted dispatch path | mitigate | The existing SLATERX + PBEX tests in contracted_cross_mode.rs use pack_for_contracted to seed bit-flag-indexed VAR seeds, then read the contracted output bit-flag pattern — this exercises Mode::Contracted's actual dispatch path (functionals/contracted.rs::launch_contracted), not a fallback. Same helper applies to metaGGA. Manual review of one test invocation under tracing::debug confirms the Contracted path is hit. |
| T-04-09-02 | Tampering | Test data values mask kernel bugs (e.g., picked density where order-3 derivative is zero) | mitigate | MGGA_INPUT is non-trivially populated: a≠b (asymmetric spin), gab≠0 (cross-gradient), tau≠0 (non-zero kinetic). Order-N coefficients are non-zero for all 0..=4 by inspection. If a kernel bug leaves a coefficient zero, the strict-1e-12 comparison would catch it (the PartialDerivatives reference value would be non-zero). |
| T-04-09-03 | Denial of Service | Test compile-time blowup from 3 new comptime monomorphisations | accept | 3 arms is negligible; the Phase-4 budget tracked since Plan 04-05 has tolerance for ~10 more arms before re-evaluation. |
| T-04-09-04 | Information Disclosure | None | accept | Tests are deterministic numerics; open source. |

No new attack surface — same comptime dispatch, same `Functional::eval` entry, same FFI envelope.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-09-contracted-metagga-SUMMARY.md`. Include:
- Test pass/fail status per metaGGA exemplar at orders 0..=4 (12 cells).
- Whether order 4 was achievable for all 3 (drop to order 3 if not).
- Citation to Plan 04-05 D-19 forward for orders 5/6 (no new investigation).
</output>
