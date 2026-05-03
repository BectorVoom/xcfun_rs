---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N3
type: execute
wave: 9
depends_on:
  - 06-00
  - 06-01
files_modified:
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
autonomous: true
requirements:
  - ACC-04
acc04_eligible:
  # (W-9 revision-1) Pre-enumerated list of which post-libm-hybrid forwards
  # are eligible for ACC-04 mpmath substitution. Per CONTEXT.md D-03, ACC-04
  # is reserved for points where C++ source EXPLICITLY documents bracket
  # cancellation. Of the ~18 small-magnitude forwards in this plan's scope,
  # NONE has a documented `test_threshold`-style cancellation comment.
  # The hypothesis (per CONTEXT.md "Specific Ideas") is that Plan 06-00 D-11
  # libm-hybrid `erf_precise_taylor` SELF-TIGHTENS most or all of these — so
  # this plan FIRST runs the post-substrate sweep to verify auto-tightening,
  # THEN escalates any persistent residuals via PLANNING INCONCLUSIVE
  # (this plan is PURE-VERIFICATION per I-3 Option B; no in-plan kernel edits).
  - "0 — auto-tighten verification only; escalate residuals via PLANNING INCONCLUSIVE"
must_haves:
  truths:
    - "Plan 06-N3 (NEW per B-1 revision-1) is the post-libm-hybrid residual sweep over the ~18 small-magnitude AD-residuals from Phase 4 D-19 forwards: M05/M06 family small-magnitude (M05X 1.89e-12, M05C 9.26e-12, M05X2C 3.02e-11, M06X ≤7.85e-12, M06LX ≤7.85e-12, M06HFX 7.8e-12, M06C/M06LC/M06HFC/M06X2C 4.88e-11..6.28e-11), B97 family X-side (B97X 9.5e-12, B97_1X 9.5e-12, B97_2X 9.5e-12), LYPC 1.3e-10, VWN_PBEC 6.9e-9, PW92C 9.0e-12, PBEC 1.8e-12, OPTX 1.2e-12."
    - "(I-3 revision-2 — Option B) PURE-VERIFICATION PLAN. This plan ONLY creates per-functional fixtures + unit tests + runs them; it makes ZERO kernel-source-file edits. Therefore `files_modified` lists ONLY `validation/fixtures/d19_n3/*.jsonl` and `crates/xcfun-kernels/tests/d19_*.rs` — no `crates/xcfun-kernels/src/**` paths. If any per-functional unit test FAILS post-Plan-06-00 (i.e., the auto-tightening hypothesis does not hold for that functional), the plan HALTS and the executor returns `PLANNING INCONCLUSIVE` to the orchestrator with the failing functional name + post-fix max_rel_err — NOT an in-plan kernel edit. This makes Wave-9 disjointness from 06-N1 trivially provable: 06-N3 cannot edit anything 06-N1 edits."
    - "Step 1 (per RESEARCH §"D-19 Bisection Methodology" Plan 06-N3): re-run tier-2 at order 3 AFTER Plan 06-00 substrate (libm-hybrid erf_precise_taylor + AD N≥4 + tau guard) lands. Document which forwards self-resolved (rel_err < 1e-13)."
    - "Step 2: for any forward that did NOT self-resolve, the plan does NOT apply a Path-B fix in-place. Instead, it captures the failing fixture + unit test (B-6 pattern), writes the per-functional verdict into 06-N3-SUMMARY.md, and escalates via PLANNING INCONCLUSIVE so the orchestrator can dispatch a follow-up plan (or extend 06-N1 in a separate revision) to apply the kernel edit. Acceptance criterion = '~18 unit tests GREEN OR documented escalation', NOT '~18 unit tests GREEN with in-plan kernel edits'."
    - "Step 3: per-functional unit test in `crates/xcfun-kernels/tests/d19_<name>.rs` (B-6 pattern from 06-N1) with RED-then-GREEN gating against expected values from C++ baseline (no ACC-04 substitution unless escalated). Each test runs in single-digit seconds on cubecl-cpu."
    - "Parallel-safe with 06-N1 + 06-N2 per CONTEXT.md D-01 discretion ('parallel is fine because they touch independent functional sets'). 06-N3 covers ~18 small-magnitude forwards; 06-N1 covers the 11 inherited-Phase-3 catastrophic forwards; 06-N2 covers the 20 excluded_by_upstream_spec set — three disjoint sets. Wave-9 disjointness is automatic given the pure-verification status of 06-N3 (I-3 Option B)."
    - "(B-1 revision-1) THIS PLAN IS NEW — added during revision-1 to close the post-libm-hybrid sweep that was implicit in CONTEXT.md D-01 plan tree but did not exist as a standalone PLAN.md. Original Phase 6 had only 06-N1 + 06-N2 in Wave 7."
  artifacts:
    - path: "validation/fixtures/d19_n3/<name>_baseline.jsonl"
      provides: "5-10 records per functional at the failing density strata + expected values from C++ baseline"
      contains: "expected"
    - path: "crates/xcfun-kernels/tests/d19_<name>.rs"
      provides: "Per-functional RED→GREEN gated unit test at strict 1e-13"
      contains: "assert_relative_eq"
  key_links:
    - from: "Plan 06-00 (substrate work — libm-hybrid erf_precise_taylor + AD N≥4 + tau guard)"
      to: "06-N3 verification sweep"
      via: "post-substrate tier-2 re-run identifies which forwards auto-tightened"
      pattern: "erf_precise_taylor"
    - from: "Plan 06-N1 (B-6 per-functional fixture pattern)"
      to: "Plan 06-N3 (same B-6 pattern reused, but pure-verification)"
      via: "RED→GREEN gating with 5-10 records per functional; residuals → PLANNING INCONCLUSIVE escalation, not in-plan kernel edits"
      pattern: "validation/fixtures/d19_n[13]/.*_baseline.jsonl"
---

<objective>
**This plan is NEW per B-1 revision-1.** The original Phase 6 had only 06-N1 + 06-N2 in Wave 7; CONTEXT.md D-01 plan tree referenced 06-N3 but no PLAN.md existed. Revision-1 closes that gap.

**(I-3 revision-2 — Option B) This is a PURE-VERIFICATION plan.** It creates per-functional fixtures + unit tests + runs them. It makes ZERO kernel-source-file edits. If a unit test FAILS, the plan halts and the executor returns `PLANNING INCONCLUSIVE` so the orchestrator can dispatch a separate kernel-edit plan. This guarantees Wave-9 disjointness with 06-N1: nothing this plan modifies can collide with anything 06-N1 modifies.

Scope: post-libm-hybrid verification sweep over the ~18 small-magnitude AD-residuals from Phase 4 D-19 forwards. The hypothesis (per CONTEXT.md "Specific Ideas") is that Plan 06-00 substrate work (D-11 `erf_precise_taylor` + AD N≥4 + tau guard) **incidentally tightens most of these to strict 1e-13**, leaving only a handful (if any) that need a follow-up Path-B kernel edit — which goes via PLANNING INCONCLUSIVE escalation, not in this plan.

Targets (~18 total, grouped by family):

**M05 / M06 family (10 functionals, all small-magnitude):**
- M05X 1.89e-12, M05C 9.26e-12, M05X2C 3.02e-11
- M06X ≤7.85e-12, M06LX ≤7.85e-12, M06HFX 7.8e-12
- M06C 4.88e-11, M06LC 5.x-11, M06HFC 6.28e-11, M06X2C 4.88e-11

**B97 family X-side (3 functionals):**
- B97X 9.5e-12, B97_1X 9.5e-12, B97_2X 9.5e-12

**Singletons (5 functionals):**
- LYPC 1.3e-10, VWN_PBEC 6.9e-9, PW92C 9.0e-12, PBEC 1.8e-12, OPTX 1.2e-12

(Note: TPSSX / REVTPSSX clamp-boundary AD-tail at 2.7e-2 / 1.3e-2 are explicitly NOT in this plan's scope — they're handled by 06-00 D-10 tau guard for the C-side and may need a separate clamp-tail fix if X-side residuals persist.)

Per CONTEXT.md "Specific Ideas": "expect that fixing one root cause tightens 3-5 functionals at once." Plan 06-N1 history (Phase 3 Plan 03-05's `build_xc_a_b_2nd_taylor` fix tightened LYPC + others) is the precedent — but any such fix lives in 06-N1 (or a follow-up plan), NOT here.

**Methodology** (pure-verification per I-3 Option B):
1. **Auto-tighten verification.** Re-run tier-2 at order 3 AFTER Plan 06-00 substrate; document which forwards self-resolved (rel_err < 1e-13). Per RESEARCH §"D-19 Bisection Methodology" Plan 06-N3, this is the bulk of the plan — most should pass.
2. **Per-functional fixture + unit test (B-6 pattern).** For each functional in scope, create `validation/fixtures/d19_n3/<name>_baseline.jsonl` with 5-10 records at the failing density strata + expected values from C++ baseline; create `crates/xcfun-kernels/tests/d19_<name>.rs` with strict-1e-13 gating.
3. **Residual handling = ESCALATION ONLY.** If a unit test fails post-Plan-06-00, do NOT apply a kernel edit. Halt and return `PLANNING INCONCLUSIVE` to the orchestrator with the functional name + max_rel_err. The orchestrator dispatches a follow-up plan (revision of 06-N1 or new) to do the kernel edit; that plan's `files_modified` lists the kernel paths and Wave-9 ordering is re-evaluated.

**Parallel-safe with 06-N1 + 06-N2** per CONTEXT.md D-01 discretion. Touches independent functional sets:
- 06-N1: 11 inherited Phase-3 catastrophic forwards (PBEINTC 6.2e+1 etc.)
- 06-N2: 20 excluded_by_upstream_spec (BR / SCAN / CSC / BLOCX / TW / VWK / PBE-corr variants)
- 06-N3 (this plan): ~18 small-magnitude post-libm-hybrid residuals (M05/M06 / B97-X / LYPC / VWN_PBEC / PW92C / PBEC / OPTX) — pure-verification

Output: ~18 per-functional fixtures + ~18 per-functional unit tests, RED→GREEN gated; tier-2 sweep over the ~18 small-magnitude set GREEN at strict 1e-13 OR documented PLANNING INCONCLUSIVE escalations for residuals that don't auto-tighten.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-00-substrate-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N1-d19-bisection-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@validation/src/driver.rs
@xcfun-master/src/functionals/m05x.cpp
@xcfun-master/src/functionals/m05c.cpp
@xcfun-master/src/functionals/m06x.cpp
@xcfun-master/src/functionals/m06c.cpp
@xcfun-master/src/functionals/b97x.cpp
@xcfun-master/src/functionals/lypc.cpp
@xcfun-master/src/functionals/optx.cpp
</context>

<tasks>

<task type="auto">
  <name>Task 1: Post-substrate auto-tighten verification + per-family triage</name>
  <files>(none — research + audit; emits 06-N3-progress.md scratch file)</files>
  <read_first>
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md (Plan 04-10 D-19 ledger — small-magnitude residuals classification)
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md §"D-19" (post-libm-hybrid hypothesis source)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"D-19 Bisection Methodology" Plan 06-N3 (auto-tighten hypothesis)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md "Specific Ideas" (~12 small-magnitude residual mention; this plan covers ~18)
    - validation/report.html (Plan 04-10 capstone — per-functional verdict matrix)
  </read_first>
  <action>
**Step A — Run order-3 tier-2 sweep AFTER Plan 06-00 substrate (no fixes yet):**

```bash
cargo run -p validation --release -- --backend cpu --order 3 --jobs 18 \
    --filter '^(m05x|m05c|m05x2c|m06x|m06c|m06lx|m06lc|m06hfx|m06hfc|m06x2c|b97x|b97_1x|b97_2x|lypc|vwn_pbec|pw92c|pbec|optx)$' \
    > /tmp/06-N3-pre-fix-sweep.log 2>&1
```

**Step B — Read output + classify:**

For each functional in scope, record:
- pre-Plan-06-00 max_rel_err (from `04-VERIFICATION.md`)
- post-Plan-06-00 max_rel_err (this run)
- self-resolved? (post < 1e-13)
- if not self-resolved: this plan does NOT diagnose root cause (per I-3 Option B); diagnosis is deferred to the follow-up plan dispatched after PLANNING INCONCLUSIVE escalation.

Hypothesis (per CONTEXT.md "Specific Ideas"):
- M05/M06 family: residuals are at low-density polarised stratum + small gradient_stress; same shape as B97_C 4.88e-11 (which 06-N1 covers). Likely auto-tightened by Plan 06-00 N≥4 substrate.
- B97 X-side: shares the B97 ε function chain with the C-side; Plan 06-00 substrate may auto-tighten.
- LYPC 1.3e-10: residual is tied to the Phase-3 Plan 03-05 `build_xc_a_b_2nd_taylor` fix; may need a follow-up fix to the same helper (which lives in 06-N1's scope, not here).
- VWN_PBEC 6.9e-9: pw92eps + log composition; same root cause as VWN3C/VWN5C order-2 forwards (Phase 2 D-19). Plan 06-00 substrate may NOT auto-tighten this one — it's not erf-bracket cancellation. Likely escalation candidate.
- PW92C 9.0e-12, PBEC 1.8e-12, OPTX 1.2e-12: borderline near-1e-12; small ULP-budget tightening from N≥4 substrate may push them under 1e-13.

**Step C — Document the audit in `06-N3-progress.md`** (a planning-time scratch file, absorbed into 06-N3-SUMMARY.md):

```markdown
# 06-N3 Pre-fix audit (after Plan 06-00 substrate)

| Functional | Phase-4 max_rel_err | post-06-00 max_rel_err | Self-resolved? | Escalation candidate? |
|------------|---------------------|------------------------|----------------|------------------------|
| M05X       | 1.89e-12            | (run sweep)            | likely YES     | no                     |
| M05C       | 9.26e-12            | (run sweep)            | partial        | maybe — escalate if test fails post-06-00 |
| ...        | ...                 | ...                    | ...            | ...                    |
```

**Step D — Identify escalation candidates** (no in-plan kernel edits per I-3 Option B):

If any functional in scope shows post-Plan-06-00 max_rel_err > 1e-13, mark as escalation candidate in 06-N3-SUMMARY.md. Do NOT identify or attempt shared-helper edits. Diagnosis (e.g., "M05/M06 share a helper; fix tightens all 10") happens in the follow-up plan dispatched by the orchestrator after this plan returns PLANNING INCONCLUSIVE.
  </action>
  <verify>
    <automated>cargo run -p validation --release -- --backend cpu --order 3 --jobs 4 --filter '^(m05x|m05c|m06x|b97x|lypc|optx)$'</automated>
  </verify>
  <acceptance_criteria>
    - Sweep completes without crash; logs show per-functional max_rel_err for the ~18 small-magnitude forwards.
    - Audit table populated in 06-N3-SUMMARY.md classifying which forwards self-resolved vs are escalation candidates.
    - No in-plan kernel-source diagnosis attempted (I-3 Option B invariant).
  </acceptance_criteria>
  <done>Post-substrate verification sweep complete; per-functional triage list documented; Task 2 work plan derived (which functionals self-tightened vs need PLANNING INCONCLUSIVE escalation).</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Per-functional fixtures + RED→GREEN unit tests (B-6 pattern); residuals escalate via PLANNING INCONCLUSIVE</name>
  <files>validation/fixtures/d19_n3/m05x_baseline.jsonl, validation/fixtures/d19_n3/m05c_baseline.jsonl, validation/fixtures/d19_n3/m05x2c_baseline.jsonl, validation/fixtures/d19_n3/m06x_baseline.jsonl, validation/fixtures/d19_n3/m06c_baseline.jsonl, validation/fixtures/d19_n3/m06lx_baseline.jsonl, validation/fixtures/d19_n3/m06lc_baseline.jsonl, validation/fixtures/d19_n3/m06hfx_baseline.jsonl, validation/fixtures/d19_n3/m06hfc_baseline.jsonl, validation/fixtures/d19_n3/m06x2c_baseline.jsonl, validation/fixtures/d19_n3/b97x_baseline.jsonl, validation/fixtures/d19_n3/b97_1x_baseline.jsonl, validation/fixtures/d19_n3/b97_2x_baseline.jsonl, validation/fixtures/d19_n3/lypc_baseline.jsonl, validation/fixtures/d19_n3/vwn_pbec_baseline.jsonl, validation/fixtures/d19_n3/pw92c_baseline.jsonl, validation/fixtures/d19_n3/pbec_baseline.jsonl, validation/fixtures/d19_n3/optx_baseline.jsonl, crates/xcfun-kernels/tests/d19_m05x.rs, crates/xcfun-kernels/tests/d19_m05c.rs, crates/xcfun-kernels/tests/d19_m05x2c.rs, crates/xcfun-kernels/tests/d19_m06x.rs, crates/xcfun-kernels/tests/d19_m06c.rs, crates/xcfun-kernels/tests/d19_m06lx.rs, crates/xcfun-kernels/tests/d19_m06lc.rs, crates/xcfun-kernels/tests/d19_m06hfx.rs, crates/xcfun-kernels/tests/d19_m06hfc.rs, crates/xcfun-kernels/tests/d19_m06x2c.rs, crates/xcfun-kernels/tests/d19_b97x.rs, crates/xcfun-kernels/tests/d19_b97_1x.rs, crates/xcfun-kernels/tests/d19_b97_2x.rs, crates/xcfun-kernels/tests/d19_lypc.rs, crates/xcfun-kernels/tests/d19_vwn_pbec.rs, crates/xcfun-kernels/tests/d19_pw92c.rs, crates/xcfun-kernels/tests/d19_pbec.rs, crates/xcfun-kernels/tests/d19_optx.rs</files>
  <read_first>
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N1-d19-bisection-PLAN.md (B-6 per-functional fixture + unit test pattern reference)
    - For each in-scope functional: `xcfun-master/src/functionals/<name>.cpp` (READ-ONLY — to understand the body for fixture-record curation; this plan does NOT edit any Rust kernel)
    - Phase 4 Plan 04-10 SUMMARY (Path-B methodology reference — for context only; this plan does NOT apply Path-B fixes)
  </read_first>
  <behavior>
    - For each of the ~18 in-scope functionals (M05/M06×10 + B97-X×3 + LYPC + VWN_PBEC + PW92C + PBEC + OPTX), produce:
      1. A `validation/fixtures/d19_n3/<name>_baseline.jsonl` file with 5-10 records at the failing density strata. Expected values come from C++ baseline at order 3 (per W-9 acc04_eligible empty list — no ACC-04 substitution unless escalated).
      2. A `crates/xcfun-kernels/tests/d19_<name>.rs` per-functional unit test (B-6 pattern from Plan 06-N1) with strict-1e-13 gating:
         - PASSES if Plan 06-00 substrate self-resolves the residual (auto-tighten hypothesis confirmed).
         - FAILS if the residual persists — the executor halts the plan and returns PLANNING INCONCLUSIVE with the functional name + max_rel_err.
    - Each unit test runs in single-digit seconds via cubecl-cpu.
    - Acceptance: all ~18 unit tests GREEN OR documented escalation set (NOT in-plan kernel edits).
  </behavior>
  <action>
**Step A — For each functional in scope:**

1. **Generate fixture file** `validation/fixtures/d19_n3/<name>_baseline.jsonl`:
   ```bash
   # Use validation harness in --emit-fixture mode (or hand-curate from existing report.jsonl):
   # Pull 5-10 records from validation/report.jsonl where (functional == <name> AND order == 3 AND rel_err > 1e-13)
   #   OR for the auto-tightened ones, sample 5-10 records at the strata that previously failed.
   # JSONL format mirrors existing fixture files:
   #   {"functional":"m05c","vars":"A_B_GAA_GAB_GBB","mode":"PartialDerivatives","order":3,
   #    "input":[0.5,0.5,0.1,0.05,0.1],"expected":[<C++ output>],"rel_err_threshold":1e-13}
   ```

2. **Write per-functional unit test** `crates/xcfun-kernels/tests/d19_<name>.rs`:
   ```rust
   #![cfg(feature = "testing")]
   use cubecl::prelude::*;
   use cubecl_cpu::CpuRuntime;
   use serde::Deserialize;
   use xcfun_eval::for_tests::cpu_client;
   use approx::assert_relative_eq;

   #[derive(Deserialize, Debug)]
   struct D19Record { functional: String, vars: String, mode: String, order: u32,
                      input: Vec<f64>, expected: Vec<f64>, rel_err_threshold: f64 }

   #[test]
   fn d19_<name>_strict_1e_13_at_failing_strata() {
       let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../validation/fixtures/d19_n3/", "<name>", "_baseline.jsonl");
       let lines = std::fs::read_to_string(path).unwrap();
       for line in lines.lines() {
           let rec: D19Record = serde_json::from_str(line).unwrap();
           // Build DensVarsDev, dispatch to <name>_kernel via cubecl-cpu launcher,
           // read_one back, compare element-by-element at strict 1e-13.
           let mut rust_out = vec![0.0; rec.expected.len()];
           // ... cubecl-cpu launcher invocation ...
           for i in 0..rec.expected.len() {
               assert_relative_eq!(rust_out[i], rec.expected[i],
                                   max_relative = rec.rel_err_threshold);
           }
       }
   }
   ```

3. **If the test FAILS post-Plan-06-00 (i.e., NOT auto-tightened):**
   - Per I-3 Option B: do NOT open `xcfun-master/src/functionals/<name>.cpp` for diagnosis.
   - Do NOT trace divergence.
   - Do NOT apply any kernel edit.
   - Instead, the executor RECORDS the failure (functional name + max_rel_err + the failing record indices) into `06-N3-SUMMARY.md` under "Escalation Candidates" and HALTS this plan with a `PLANNING INCONCLUSIVE` return marker. The orchestrator handles dispatch of a follow-up kernel-edit plan (revision of 06-N1 or new) that will edit the kernel source.

4. **If the test PASSES post-Plan-06-00 (auto-tightened):**
   - Document in 06-N3-SUMMARY.md as "self-resolved by Plan 06-00 substrate".

**Step B — Cluster diagnosis is OUT OF SCOPE here:**

Per I-3 Option B: this plan does NOT diagnose shared-helper candidates and does NOT apply cluster fixes. Any cluster-fix logic (e.g., "M05/M06 family shares a helper") lives in the follow-up plan after escalation.

**Step C — PLANNING INCONCLUSIVE escalation contract:**

If any unit test fails, the executor returns the following structured result to the orchestrator (per I-3 Option B):

```
## PLANNING INCONCLUSIVE — Plan 06-N3 escalation

Plan: 06-N3 (pure-verification per I-3 Option B)
Trigger: <N> functional(s) failed post-Plan-06-00 unit test
Failing functionals + max_rel_err:
  - <name>: <rel_err>
  - <name>: <rel_err>
Recommended follow-up:
  - Dispatch a kernel-edit plan (extend 06-N1 in revision-3 OR new plan 06-N4)
  - Failing-functional fixtures + tests (already created here) become RED-state input for the follow-up plan
  - Wave-9 disjointness re-evaluation: the follow-up plan's `files_modified` MUST be checked against 06-N1's
```

**Step D — Run all per-functional unit tests:**

```bash
cargo nextest run -p xcfun-kernels --test d19_m05x --test d19_m05c --test d19_m05x2c \
    --test d19_m06x --test d19_m06c --test d19_m06lx --test d19_m06lc --test d19_m06hfx \
    --test d19_m06hfc --test d19_m06x2c --test d19_b97x --test d19_b97_1x --test d19_b97_2x \
    --test d19_lypc --test d19_vwn_pbec --test d19_pw92c --test d19_pbec --test d19_optx
```

Must exit 0 (auto-tighten hypothesis confirmed) OR plan halts with PLANNING INCONCLUSIVE escalation per Step C.

**Forbidden:**
- Do NOT introduce `mul_add(...)` (xtask check-no-mul-add blocks; would also be moot since this plan doesn't edit kernel sources).
- Do NOT silently widen tolerance — the rel_err_threshold per fixture record is locked at 1e-13.
- Do NOT add ACC-04 mpmath substitution unless the functional is added to `acc04_eligible:` via PLANNING INCONCLUSIVE escalation + user approval (per W-9 revision-1 contract).
- **(I-3 revision-2)** Do NOT edit any file under `crates/xcfun-kernels/src/**` from this plan. Doing so violates the pure-verification contract and breaks Wave-9 disjointness.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-kernels --test d19_m05x --test d19_m05c --test d19_m06x --test d19_b97x --test d19_lypc --test d19_optx --test d19_pbec --test d19_pw92c</automated>
  </verify>
  <acceptance_criteria>
    - All ~18 per-functional unit tests GREEN at strict 1e-13: `cargo nextest run -p xcfun-kernels --test d19_m05x --test d19_m05c --test d19_m05x2c --test d19_m06x --test d19_m06c --test d19_m06lx --test d19_m06lc --test d19_m06hfx --test d19_m06hfc --test d19_m06x2c --test d19_b97x --test d19_b97_1x --test d19_b97_2x --test d19_lypc --test d19_vwn_pbec --test d19_pw92c --test d19_pbec --test d19_optx` exits 0 — OR — plan returns PLANNING INCONCLUSIVE with documented escalation set per Step C.
    - Each `validation/fixtures/d19_n3/<name>_baseline.jsonl` exists with 5-10 records: `find validation/fixtures/d19_n3 -name '*_baseline.jsonl' -size +0c | wc -l` >= 18.
    - Each `crates/xcfun-kernels/tests/d19_<name>.rs` exists: `find crates/xcfun-kernels/tests -name 'd19_*.rs' | wc -l` >= 18 (per-N3 set; 06-N1 contributes another 11).
    - Per-functional self-resolution / escalation verdict documented in 06-N3-SUMMARY.md.
    - **(I-3 revision-2 — Option B)** No kernel-source-file edits: `git diff --stat HEAD~1 -- crates/xcfun-kernels/src/` reports zero changes from this plan's commits.
    - tier-2 LDA + GGA quick sweep at order 2 still GREEN (sanity check; this plan cannot regress because it doesn't edit kernels, but verify nothing else broke).
  </acceptance_criteria>
  <done>~18 small-magnitude D-19 forwards from Phase 4 verified at strict 1e-13 (auto-tightened) OR escalated via PLANNING INCONCLUSIVE; per-functional fixtures + unit tests landed; no kernel-source edits in this plan (I-3 Option B invariant preserved).</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust port ↔ C++ source | Algorithmic-identity contract per CLAUDE.md Core Value; deviations only via D-03 ACC-04 amendment (W-9: empty acc04_eligible list = no auto-substitution) |
| Plan boundary ↔ kernel-source edits | (I-3 Option B) This plan never edits `crates/xcfun-kernels/src/**`. Any kernel edit triggered by a residual goes via PLANNING INCONCLUSIVE escalation to a follow-up plan. |
| Plan 06-00 substrate ↔ post-substrate auto-tightening | Hypothesis: most ~18 small-magnitude residuals self-resolve from libm-hybrid + N≥4 substrate; verified per-functional |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-WIDEN-TOLERANCE | medium | Implementer tempted to widen tolerance for stubborn residuals | Forbidden per CONTEXT.md D-02 + W-9 acc04_eligible empty list; escalate via PLANNING INCONCLUSIVE |
| T-06-IN-PLAN-KERNEL-EDIT | medium | (I-3 Option B) Implementer tempted to apply Path-B fix in-place when test fails — would violate Wave-9 disjointness | Forbidden in Step A.3 + Forbidden bullet; acceptance criterion `git diff --stat HEAD~1 -- crates/xcfun-kernels/src/` == 0 enforces |
| T-06-AUTO-TIGHTEN-FALSE-POSITIVE | low | A functional reports < 1e-13 post-Plan-06-00 but only at the single sample point checked | Per-functional fixtures carry 5-10 records covering the failing strata, not a single point |
</threat_model>

<verification>
- All ~18 per-functional unit tests GREEN at strict 1e-13 OR documented escalation set returned via PLANNING INCONCLUSIVE.
- xtask check-no-mul-add GREEN (vacuous: no kernel edits this plan).
- xtask check-no-anyhow GREEN.
- tier-2 LDA + GGA quick sweep at order 2 still GREEN.
- **(I-3 revision-2 — Option B)** Zero kernel-source-file edits from this plan: `git diff --stat HEAD~1 -- crates/xcfun-kernels/src/` reports nothing.
- Phase 4 D-19 ledger (`04-VERIFICATION.md`) updated to reflect Plan 06-N3 closures (auto-tightened vs escalated).
- 06-N3-SUMMARY.md includes a per-functional verdict table (auto-tightened vs PLANNING INCONCLUSIVE escalation).
</verification>

<success_criteria>
- ~18 small-magnitude D-19 forwards from Phase 4 verified at strict 1e-13 (auto-tightened) OR escalated for follow-up plan.
- ROADMAP Phase 6 success criterion 2 advanced (in concert with 06-N1 + 06-N2): full 78-functional tier-2 GREEN at strict 1e-13 on the primary backend (CPU/ROCm).
- B-1 from revision-1 closed (this plan exists; was missing from original Phase 6).
- B-6 / W-9 from revision-1 applied (per-functional fixtures + tests; explicit acc04_eligible list).
- I-3 from revision-2 closed (Option B chosen — pure-verification; Wave-9 disjointness with 06-N1 trivially provable).
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N3-SUMMARY.md` documenting:
- Per-functional verdict table (auto-tightened vs PLANNING INCONCLUSIVE escalation)
- For escalations: functional name + post-fix max_rel_err + failing record indices, prepared as input for the orchestrator's follow-up plan
- Updated Phase 4 D-19 ledger (`04-VERIFICATION.md` updates referencing N3 closures)
- Per-functional fixture record count + max_rel_err post-substrate
- Any escalations (W-9 acc04_eligible list amendments) authorised by user
</output>
</content>
</invoke>