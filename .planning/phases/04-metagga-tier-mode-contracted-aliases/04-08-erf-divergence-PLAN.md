---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "08"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/xcfun-eval/src/functionals/lda/ldaerfx.rs
  - crates/xcfun-eval/src/functionals/lda/ldaerfc.rs
  - crates/xcfun-eval/src/functionals/lda/ldaerfc_jt.rs
  - .planning/REQUIREMENTS.md
  - .planning/STATE.md
requirements: []
autonomous: true
gap_closure: true
created: "2026-04-26"
goal: "Wave 1 (gap closure) — investigate the catastrophic high-order divergences in XC_LDAERFX (o3 = 1.11e+1), XC_LDAERFC (o3 = 5.10e+2), XC_LDAERFC_JT (o3 = 1.07e-4 with red o2/o3) reported in 04-VERIFICATION.md Gap 2; either land a kernel fix that brings orders 0..=3 within 1e-12 (or upstream's 1e-7 D-24 envelope), OR document a Phase-6 D-19 INCONCLUSIVE entry per functional with explicit root-cause hypothesis"

must_haves:
  truths:
    - "Each of XC_LDAERFX, XC_LDAERFC, XC_LDAERFC_JT has either (a) a committed kernel fix that lands order-0..=3 max_rel_err ≤ 1e-7 (D-24 envelope) on the canonical 10k-point grid, OR (b) a documented D-19 INCONCLUSIVE entry in REQUIREMENTS.md with root-cause hypothesis + Phase-6 follow-up assignment"
    - "Plan 04-10 (sign-off) sees a deterministic verdict on the three ERF functionals: PASS-after-fix, or DEFERRED-with-rationale — no silent failures"
    - "The ~1e-12 to 1e-11 low-density rounding observed on VWN3C/VWN5C/VWN_PBEC/PBEC/PZ81C/PW92C is also captured (either by raising D-19 entries that already exist in REQUIREMENTS, or adding new ones for the GGA correlation cases)"
  artifacts:
    - path: .planning/REQUIREMENTS.md
      provides: "Updated D-19 INCONCLUSIVE forward list with the ERF + LDA-correlation entries discovered by the order-3 sweep"
      contains: "LDAERFX"
    - path: .planning/STATE.md
      provides: "Phase-4-discovered D-19 entries appended to Accumulated Context"
      contains: "Phase-4 D-19"
  key_links:
    - from: "validation/report.jsonl @ order=3"
      to: ".planning/REQUIREMENTS.md D-19 forward list"
      via: "per-functional max_rel_err extracted from sweep + manual triage"
      pattern: "LDAERFX|LDAERFC|LDAERFC_JT"
---

<objective>
Closes Gap 2 of 04-VERIFICATION.md: three range-separated LDAs (XC_LDAERFX, XC_LDAERFC, XC_LDAERFC_JT) exhibit catastrophic divergence at order 3 (1.11e+1, 5.10e+2, 1.07e-4 respectively) that is NOT on the existing Phase-3 D-19 forward list. Phase-2 already documented these at orders 0..=2 under D-24 (1e-7 override) — but the order-3 amplification through the AD chain wasn't visible at the Phase-2 order cap of 2.

Two acceptable paths per functional:

**Path A — fix the kernel.** Compare the Rust kernel against the C++ reference (`xcfun-master/src/functionals/ldaerfx.cpp`, `ldaerfc.cpp`, `ldaerfc_jt.cpp`) at the order-3 Taylor-coefficient level; identify the algebraic divergence (typically: a missing term in an erf expansion, a wrong sign, or a numerically-unstable bracket that amplifies cancellation between order 2 and order 3); land the fix; re-run order-3 spot check; confirm 1e-7 (D-24 envelope) on the canonical grid.

**Path B — document and forward.** If root-cause exceeds budget (>4 hours) or requires xcfun-ad-level surgery (the Phase-6 libm-hybrid is already on the roadmap for the underlying erf precision issue), append a structured D-19 INCONCLUSIVE entry per functional in REQUIREMENTS.md and STATE.md with: max_rel_err observed, hypothesis, Phase-6 follow-up assignment.

Either path leaves Plan 04-10 with a deterministic outcome to record in the sign-off ledger. There is no third option — silent failure is forbidden.

Output: 3 ERF kernels either fixed or explicitly forwarded with full documentation; STATE.md updated; new D-19 INCONCLUSIVE entries explicit (if any) so Plan 04-10 can ledger them.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/REQUIREMENTS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/src/functionals/lda/ldaerfx.rs
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/src/functionals/lda/ldaerfc.rs
@/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/src/functionals/lda/ldaerfc_jt.rs
@/home/chemtech/workspace/xcfun_rs/xcfun-master/src/functionals/ldaerfx.cpp
@/home/chemtech/workspace/xcfun_rs/xcfun-master/src/functionals/ldaerfc.cpp
@/home/chemtech/workspace/xcfun_rs/xcfun-master/src/functionals/ldaerfc_jt.cpp

<interfaces>
<!-- Phase-2 fixes already shipped (do not regress). -->

From Plan 02-06 commit 6ab5872 (LDAERFX expm1 stable bracket):
- The `branch B` of LDAERFX uses `expm1` rather than `exp - 1` to avoid 6-digit cancellation.
- mpmath at 200-digit precision confirms Rust = mathematical ground truth at orders 0..=2.
- C++ itself loses precision in this branch — the 1e-7 D-24 override (USER-APPROVED 2026-04-20) is permanent.

From Plan 02-06 commit dca382a (libm-port erf_precise):
- cubecl 0.10-pre.3's Float::erf polyfill (~1.3e-8 ULP) was replaced with FreeBSD msun-derived port.
- Phase 1 baseline tightened from 1e-7 to 1e-14.

D-24 (CONTEXT.md decision):
- Strict 1e-12 NOT applied to XC_LDAERFX, XC_LDAERFC, XC_LDAERFC_JT.
- Per-functional override = 1e-7. Set in `validation/src/driver.rs::threshold_for` line 202-208.

Order-2 vs order-3 amplification model:
- LDAERFX order 2 max_rel_err = 6.7e-2 (already FAIL at 1e-7).
- LDAERFX order 3 max_rel_err = 1.11e+1 (catastrophic — 8 orders of magnitude worse).
- The AD chain amplifies an underlying ~1 ULP error: each derivative step typically grows error by 10× to 100× when the input is near a cancellation point.
- Phase 6 libm-hybrid is the architecturally-correct fix; this plan's job is to verify the diagnosis and document, OR find a cheap kernel-level fix.

Per-record exclusion mask (Plan 02-06 Fix 2):
- Records with `min(a,b) ≤ 2e-14` (REGULARIZE_CLAMP_STRATUM_BOUND) are flagged `excluded_by_regularize_clamp_design`.
- They do NOT count against the tier-2 verdict.
- The reported 1.11e+1 max_rel_err is on NON-CLAMP records — a real failure, not a regularize edge case.
</interfaces>
</context>

<tasks>

<task id="8.1" type="auto">
  <name>Task 1: Order-by-order bisection of XC_LDAERFX divergence — identify whether the order-3 failure is a kernel bug or an AD-chain amplification</name>
  <files>
    crates/xcfun-eval/src/functionals/lda/ldaerfx.rs
    /tmp/04-08-ldaerfx-bisection.txt
  </files>
  <read_first>
    - `crates/xcfun-eval/src/functionals/lda/ldaerfx.rs` — full file. Read every line; the kernel is ~150 lines and includes the Phase-2 expm1 stable-bracket fix.
    - `xcfun-master/src/functionals/ldaerfx.cpp` — full file. Read every line; the C++ reference is the algorithmic-identity target.
    - `xcfun-master/external/upstream/taylor/tmath.hpp` — `pow_expand`, `exp_expand`, `erf_expand` definitions (the AD primitives whose composition produces the order-3 amplification).
    - `crates/xcfun-ad/src/expand/erf.rs` — the Rust `ctaylor_erf` implementation. Confirm it accepts orders ≥ 3 and inspect the recursion truncation point.
    - `validation/src/driver.rs` lines 200-210 — `threshold_for` confirms 1e-7 envelope.
    - `.planning/STATE.md` Accumulated Context — the Phase-2 D-24 reasoning trail.
  </read_first>
  <action>
    Bisect the order-3 divergence with three concrete probes. Budget: 90 minutes total. If exceeded, switch to Path B (Task 8.4 — document as D-19 forward).

    **Probe 1 — Order-by-order single-point comparison.**

    Pick a low-magnitude representative density point: `(a=1.1, b=1.0)` (matches the upstream test fixture in xcfun-master/src/functionals/ldaerfx.cpp:65 — the only point with a known-good reference value). Run:

    ```bash
    cargo run -p validation --release -- --backend cpu --order 3 --filter 'ldaerfx' 2>&1 | tee /tmp/04-08-ldaerfx-order3.log
    ```

    Then extract the failing point from report.jsonl:
    ```bash
    jq 'select(.functional == "XC_LDAERFX" and .order == 3 and .pass == false and .excluded_by_regularize_clamp_design == false)' validation/report.jsonl | head -3 > /tmp/04-08-ldaerfx-failures.json
    ```

    Examine `/tmp/04-08-ldaerfx-failures.json`: does the failure cluster at low density (a < 0.1, b < 0.1)? At high spin polarisation (|a-b|/n > 0.5)? At specific element_idx (i.e., specific Taylor coefficients)? Record the cluster pattern in `/tmp/04-08-ldaerfx-bisection.txt`.

    **Probe 2 — Branch-A vs Branch-B isolation.**

    Read `crates/xcfun-eval/src/functionals/lda/ldaerfx.rs`. Find the branch selector (typically a runtime predicate like `if mu/k_F < threshold`). The kernel has TWO mathematically-equivalent branches; the Phase-2 fix (commit 6ab5872) put `expm1` into Branch B. The order-3 failure may be exclusive to ONE branch.

    Add a temporary `tracing::debug!` to log which branch each failing point hits. Or, easier: deterministically force the kernel into one branch by setting `XC_RANGESEP_MU` to extremes:

    ```bash
    # Force long-range branch (mu small, expansion in mu/k_F)
    XCFUN_RANGESEP_MU_OVERRIDE=0.01 cargo run -p validation --release -- --backend cpu --order 3 --filter 'ldaerfx' 2>&1 | tail -5
    # Force short-range branch (mu large, expansion in k_F/mu)
    XCFUN_RANGESEP_MU_OVERRIDE=10.0 cargo run -p validation --release -- --backend cpu --order 3 --filter 'ldaerfx' 2>&1 | tail -5
    ```

    NOTE: If `XCFUN_RANGESEP_MU_OVERRIDE` is not a real env var (the codebase uses parameter slots, not env), instead modify `validation/src/driver.rs::run` temporarily to set the parameter via `Functional::set_parameter` before `eval`. Do NOT commit this temporary instrumentation.

    Record in `/tmp/04-08-ldaerfx-bisection.txt`:
    - Branch A failure rate at order 3
    - Branch B failure rate at order 3
    - Whether the failure is branch-specific or universal

    **Probe 3 — Compare ctaylor_erf order-3 coefficient against finite-difference reference.**

    Write a short standalone Rust test (NOT committed; under `/tmp/`):
    ```rust
    // /tmp/04-08-ad-probe.rs — paste into a temporary test file inside crates/xcfun-ad/tests/
    // and run: cargo test -p xcfun-ad --features cpu test_erf_order3_finite_diff -- --nocapture
    // DELETE after analysis.
    use xcfun_ad::*;
    #[test]
    fn test_erf_order3_finite_diff() {
        let x0 = 0.5_f64;
        let h = 1e-5;
        // 5-point stencil for d³erf/dx³ at x0
        let fd = (erf_scalar(x0 + 2.0*h) - 2.0*erf_scalar(x0 + h) + 2.0*erf_scalar(x0 - h) - erf_scalar(x0 - 2.0*h)) / (2.0 * h.powi(3));
        // ctaylor_erf at order 3 (4 coefficients): [erf(x0), erf'(x0), erf''(x0)/2!, erf'''(x0)/3!]
        let ad = ctaylor_erf_at_order_3(x0); // <- adapt to actual API
        let ad_d3 = ad[3] * 6.0;  // erf'''(x0) = 3! * coeff[3]
        println!("FD erf'''({}) = {:.16e}", x0, fd);
        println!("AD erf'''({}) = {:.16e}", x0, ad_d3);
        println!("rel_err = {:.6e}", (fd - ad_d3).abs() / fd.abs());
    }
    fn erf_scalar(x: f64) -> f64 { libm::erf(x) }
    ```
    Run it and record the AD-vs-FD relative error in `/tmp/04-08-ldaerfx-bisection.txt`. If `rel_err < 1e-9`, `ctaylor_erf` is correct and the bug is downstream in the LDAERFX bracket. If `rel_err > 1e-7`, the bug is in `ctaylor_erf` itself (a Phase-6 libm-hybrid concern, NOT a Phase-4 fix).

    **Verdict:**

    - If Probe 1+2 isolate the failure to one branch AND Probe 3 shows ctaylor_erf is correct → there is a localisable bug in `ldaerfx.rs` (Path A — go to Task 8.2).
    - If Probe 3 shows ctaylor_erf itself diverges → the failure is rooted in xcfun-ad's erf expansion at order ≥ 3 (Path B — go to Task 8.4 and document).
    - If the failure is universal across branches AND ctaylor_erf is fine → there is a structural issue in how LDAERFX composes erf with the bracket polynomial (likely Path B, but Task 8.2 should attempt a quick spot-check of the bracket code anyway).

    Save `/tmp/04-08-ldaerfx-bisection.txt` with the verdict and route decision.
  </action>
  <acceptance_criteria>
    1. `test -s /tmp/04-08-ldaerfx-bisection.txt` returns 0 (the bisection report exists and is non-empty).
    2. `grep -cE "Branch A failure rate|Branch B failure rate|FD erf|AD erf|rel_err" /tmp/04-08-ldaerfx-bisection.txt` is at least 3 (all three probes recorded).
    3. `grep -cE "Verdict:.*Path [AB]" /tmp/04-08-ldaerfx-bisection.txt` is exactly 1.
    4. `git status --porcelain crates/xcfun-eval/src/functionals/lda/ldaerfx.rs | wc -l` is 0 OR 1 (this task does NOT modify ldaerfx.rs unless Verdict is Path A AND the fix is included; Task 8.2 owns the modification commit).
  </acceptance_criteria>
  <done>Bisection complete; verdict recorded; route to Task 8.2 (kernel fix) or Task 8.4 (D-19 forward) determined.</done>
</task>

<task id="8.2" type="auto">
  <name>Task 2: If Verdict = Path A, attempt kernel fix on ldaerfx.rs (and parallel checks on ldaerfc.rs / ldaerfc_jt.rs); if Verdict = Path B or budget exceeded, skip and proceed to Task 8.4</name>
  <files>
    crates/xcfun-eval/src/functionals/lda/ldaerfx.rs
    crates/xcfun-eval/src/functionals/lda/ldaerfc.rs
    crates/xcfun-eval/src/functionals/lda/ldaerfc_jt.rs
  </files>
  <read_first>
    - `/tmp/04-08-ldaerfx-bisection.txt` — Task 8.1 verdict and route.
    - `crates/xcfun-eval/src/functionals/lda/ldaerfx.rs` — the file under repair.
    - `xcfun-master/src/functionals/ldaerfx.cpp` — the algorithmic-identity reference.
    - `crates/xcfun-eval/src/functionals/lda/ldaerfc.rs` and `ldaerfc.cpp` — sibling.
    - `crates/xcfun-eval/src/functionals/lda/ldaerfc_jt.rs` and `ldaerfc_jt.cpp` — sibling.
  </read_first>
  <action>
    **GATE: only execute this task if Task 8.1 verdict is Path A. If Path B, skip directly to Task 8.4.**

    For each of the three ERF functionals, the typical kernel-level repair is one of:

    1. **Missing higher-order term in a polynomial bracket.** When a bracket like `1 - exp(-mu²·z)` is expanded, low orders converge fast; high orders may need explicit truncation or stabilisation. Compare the Rust kernel's bracket ordering line-for-line against the C++ source.

    2. **Wrong sign in a Taylor coefficient.** Sign errors produce small-error at orders 0..=2 but explode at order 3+ as the AD chain rule amplifies. Spot-check by computing the order-3 coefficient symbolically (e.g., with sympy) for the canonical fixture (a=1.1, b=1.0) and diffing against `rust_out[2..6]` from the failing record.

    3. **Branch threshold mis-tuned.** Phase-2 left the branch threshold at the upstream default. If the bisection revealed branch-specific failures at the threshold boundary, adjusting the predicate (e.g., switching to a smoother sigmoid blend) may resolve it — but ONLY if the C++ reference uses the same blend. Algorithmic identity is non-negotiable.

    **Concrete repair process per functional:**

    a) Identify the suspected line in the Rust kernel from Task 8.1's bisection.
    b) Read the corresponding C++ line in xcfun-master.
    c) Diff the two; if the Rust deviates from C++, repair the Rust to match C++ verbatim.
    d) `cargo build -p xcfun-eval --release && cargo test -p xcfun-eval --test self_tests --features testing -- --nocapture | grep -E "ldaerf"` — confirm tier-1 still passes.
    e) `cargo run -p validation --release -- --backend cpu --order 3 --filter 'ldaerfx' 2>&1 | tail -10` — confirm order-3 max_rel_err drops to ≤ 1e-7.

    If after 90 minutes no fix lands for a given functional, document in `/tmp/04-08-fix-attempt.txt`:
    - What was tried
    - What was observed
    - Why the fix failed
    Then forward that functional via Task 8.4 (Path B).

    **Per-functional fix (or skip):**

    - **XC_LDAERFX:** if branch isolation showed Branch A failing → inspect Branch A's bracket (the cubecl-friendly expansion). Repair to match `xcfun-master/src/functionals/ldaerfx.cpp` lines 30-60 (the long-range branch).
    - **XC_LDAERFC:** order-3 failure is 5.10e+2 — even more extreme than LDAERFX. This is more likely a Path B (Phase-6) candidate, but spot-check the spin-decomposition: `ldaerfc.cpp:50-90` constructs the polarised correlation as `(ldaerfc(2a,0) + ldaerfc(0,2b))/2` for unrestricted spin. Confirm the Rust kernel preserves this exact composition.
    - **XC_LDAERFC_JT:** order-3 = 1.07e-4, much closer to the 1e-7 envelope. Most likely root cause: the `JT` (Jacob-Tritsch) parameter set has a single coefficient that drifts. Spot-check `ldaerfc_jt.cpp` parameter table line-by-line vs the Rust constants.

    **Commit (only for fixed functionals):**

    Per fix, commit individually:
    ```
    fix(04-08): land order-3 parity for XC_LDAERFX (Branch A bracket repair)
    fix(04-08): land order-3 parity for XC_LDAERFC (spin-decomposition fix)
    fix(04-08): land order-3 parity for XC_LDAERFC_JT (parameter table correction)
    ```

    Each commit message must reference the C++ source line range that was matched.
  </action>
  <acceptance_criteria>
    1. `test -f /tmp/04-08-ldaerfx-bisection.txt && grep -cE "Verdict:.*Path A" /tmp/04-08-ldaerfx-bisection.txt` is exactly 1 (gate condition; if 0, task is a no-op).
    2. For any functional fixed: `cargo run -p validation --release -- --backend cpu --order 3 --filter '<lda_name>' 2>&1 | grep -oE "max_rel_err=[0-9.e+-]+" | head -1` reports a numeric value ≤ 1e-7.
    3. For any functional NOT fixed (gave up): `test -f /tmp/04-08-fix-attempt.txt && grep -c "<lda_name>" /tmp/04-08-fix-attempt.txt` is at least 1 (documented giving up).
    4. `cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | grep -cE "test result: ok\." | head -1` is at least 1 (tier-1 still passes).
    5. `git log --oneline -5 | grep -cE "fix\(04-08\)" | head -1` is between 0 and 3 inclusive.
  </acceptance_criteria>
  <done>For each ERF functional: either fixed (commit landed) or documented as exceeded-budget (entry in /tmp/04-08-fix-attempt.txt) — Task 8.4 picks up the documented ones.</done>
</task>

<task id="8.3" type="auto">
  <name>Task 3: Triage low-density rounding (~1e-12 to 1e-11) on VWN3C/VWN5C/VWN_PBEC/PBEC/PZ81C/PW92C — confirm these are the same edge case already documented in Phase-2/3 D-19 (no new entries needed) OR escalate</name>
  <files>
    /tmp/04-08-lda-corr-triage.txt
  </files>
  <read_first>
    - `.planning/REQUIREMENTS.md` lines 35-44 — confirm LDA-02..05 already document order-2 D-19 entries.
    - `.planning/STATE.md` Accumulated Context — Phase-2 D-19 reasoning.
    - `validation/report.jsonl` (post-Task-8.1 sweep of order 3) — extract VWN3C/VWN5C/VWN_PBEC/PBEC/PZ81C/PW92C order-3 max_rel_err.
  </read_first>
  <action>
    The 04-VERIFICATION.md notes that VWN3C/VWN5C/VWN_PBEC/PBEC/PZ81C/PW92C show 1e-12..1e-11 failures at orders 2/3. Phase 2 ALREADY forwarded these as D-19 entries (LDA-02..05 in REQUIREMENTS.md explicitly note "order-2 near-clamp residuals documented D-19 INCONCLUSIVE — Phase 3"). Phase 3's order-3 sweep was interrupted (per STATE.md "order-3 capstone re-run forwarded to Phase 6"), so the order-3 failure is the SAME entry, just observed at the next derivative order.

    **Determine whether this needs new D-19 entries or is covered by existing forwards.**

    Extract per-functional max_rel_err from the order-3 sweep:
    ```bash
    for fn in XC_VWN3C XC_VWN5C XC_VWN_PBEC XC_PBEC XC_PZ81C XC_PW92C; do
      printf "%-15s " "$fn"
      jq -s --arg f "$fn" 'map(select(.functional == $f and .excluded_by_regularize_clamp_design == false)) | (map(.rel_err) | max)' validation/report.jsonl
    done > /tmp/04-08-lda-corr-triage.txt
    ```

    For each functional:
    - If max_rel_err < 1e-10 → covered by existing Phase-2/3 D-19 forwards; no new entry needed. Annotate `/tmp/04-08-lda-corr-triage.txt`: `<fn>: COVERED (max_rel=<value> < 1e-10, matches Phase-2 forward)`.
    - If max_rel_err ≥ 1e-10 → NEW Phase-4-discovered failure mode; record as new D-19 entry candidate. Annotate `/tmp/04-08-lda-corr-triage.txt`: `<fn>: NEW (max_rel=<value> ≥ 1e-10, escalate to Plan 04-10)`.

    For NEW entries, additionally identify whether the failure is at low density (a < 0.01 OR b < 0.01) by sampling the worst point:
    ```bash
    jq -s --arg f "$fn" 'map(select(.functional == $f and .excluded_by_regularize_clamp_design == false)) | sort_by(.rel_err) | reverse | .[0]' validation/report.jsonl | jq '{rel_err, input, order}'
    ```
    Append the input vector to `/tmp/04-08-lda-corr-triage.txt`.

    No source-code changes in this task. The triage feeds Plan 04-10's D-19 ledger.
  </action>
  <acceptance_criteria>
    1. `test -f /tmp/04-08-lda-corr-triage.txt` returns 0 (file exists).
    2. `grep -cE "VWN3C|VWN5C|VWN_PBEC|PBEC|PZ81C|PW92C" /tmp/04-08-lda-corr-triage.txt` is at least 6.
    3. `grep -cE "COVERED|NEW" /tmp/04-08-lda-corr-triage.txt` is at least 6.
    4. `git status --porcelain crates/xcfun-eval/ | wc -l` is exactly 0 (no source-code changes in this task).
  </acceptance_criteria>
  <done>Per-functional disposition recorded — covered by prior D-19 forwards or escalated to 04-10 as new entries.</done>
</task>

<task id="8.4" type="auto">
  <name>Task 4: For any ERF functional NOT fixed by Task 8.2, write structured D-19 INCONCLUSIVE entries to REQUIREMENTS.md and STATE.md; capture the bisection rationale verbatim</name>
  <files>
    .planning/REQUIREMENTS.md
    .planning/STATE.md
  </files>
  <read_first>
    - `.planning/REQUIREMENTS.md` lines 35-44 — the existing LDA-06/07/08 entries (D-19 INCONCLUSIVE - Phase 6 already cited).
    - `.planning/STATE.md` Accumulated Context section — the existing D-19 forward log format used by Phase-3 Wave-6 sign-off.
    - `/tmp/04-08-ldaerfx-bisection.txt` — Task 8.1 verdict.
    - `/tmp/04-08-fix-attempt.txt` — Task 8.2 give-up log (if any).
    - `/tmp/04-08-lda-corr-triage.txt` — Task 8.3 LDA-correlation triage.
  </read_first>
  <action>
    Append D-19 INCONCLUSIVE entries to REQUIREMENTS.md and STATE.md for any ERF functional NOT fixed in Task 8.2 plus any LDA-correlation functional flagged NEW in Task 8.3.

    **REQUIREMENTS.md updates:**

    For each ERF functional whose `LDA-06`/`LDA-07`/`LDA-08` entry exists (lines 39-41), AMEND the existing line with the order-3 finding (do not duplicate the entry):

    Example AMENDED line for LDA-06 (XC_LDAERFX):
    ```
    - [x] **LDA-06**: `XC_LDAERFX` ported, registered, self-test passes (Plan 02-04; tier-2 GREEN at orders 0/1; order-2 Rust = mpmath truth, C++ itself unstable — D-19 INCONCLUSIVE — Phase 6; **Phase 4 plan 04-08 confirms order-3 max_rel_err = 1.11e+1 amplifies the same erf-bracket cancellation across the AD chain — same root cause, no new fix viable without Phase-6 libm-hybrid; D-19 entry remains, Phase 6 prerequisite reinforced**)
    ```

    Apply identical pattern to LDA-07 (XC_LDAERFC) and LDA-08 (XC_LDAERFC_JT) with their respective max_rel_err values from Task 8.1's bisection.

    For NEW LDA-correlation entries from Task 8.3 (if any), insert AT END of the LDA section (before the GGA section):

    Example NEW line:
    ```
    - **D-19 (Phase 4 plan 04-08):** `XC_<NAME>` order-3 tier-2 max_rel_err = <VALUE> at low-density grid points (a < 0.01 OR b < 0.01). Inherits from Phase-2/3 D-19 LDA-correlation low-density rounding. Forwarded to Phase 6 libm-hybrid resolution. Hypothesis: <FROM TASK 8.3 INPUT VECTOR>.
    ```

    **STATE.md updates:**

    In the Accumulated Context > Decisions section (find "Plan 02-06 in-kernel libm-port erf_precise"-style entries), append a new entry:

    ```markdown
    - **Plan 04-08 D-19 forward (commit `<HASH>`):** Confirmed Phase-3 D-19 forward list now includes:
      * XC_LDAERFX order-3 max_rel_err = 1.11e+1 (was order-2 6.7e-2 — AD-chain amplification of the known erf bracket cancellation)
      * XC_LDAERFC order-3 max_rel_err = 5.10e+2 (spin-decomposition path inherits the same instability)
      * XC_LDAERFC_JT order-3 max_rel_err = 1.07e-4 (closest to 1e-7 envelope; least severe)
      * Plus <N> NEW low-density LDA-correlation entries from Task 8.3 (functional names + max_rel_err in 04-08-SUMMARY.md).
      * All forwarded to Phase 6 libm-hybrid resolution (no Phase-4 viable fix per Task 8.1 bisection).
      * Plan 04-10 ledgers these explicitly in the sign-off VERIFICATION.md update.
    ```

    Replace `<HASH>` with the actual commit hash AFTER committing.

    **Commit:**

    ```bash
    git add .planning/REQUIREMENTS.md .planning/STATE.md
    git commit -m "docs(04-08): forward 3 ERF + N LDA-corr D-19 INCONCLUSIVE entries to Phase 6"
    ```
  </action>
  <acceptance_criteria>
    1. `grep -c "Phase 4 plan 04-08" .planning/REQUIREMENTS.md` is at least 1.
    2. `grep -c "Plan 04-08 D-19 forward" .planning/STATE.md` is exactly 1.
    3. `grep -cE "LDAERFX order-3|LDAERFC order-3|LDAERFC_JT order-3" .planning/STATE.md` is at least 1 (at least one ERF functional explicitly named in STATE forward log).
    4. `git log -1 --oneline | grep -c "04-08"` is exactly 1.
    5. `git diff --name-only HEAD~1 HEAD` lists exactly: `.planning/REQUIREMENTS.md`, `.planning/STATE.md`.
  </acceptance_criteria>
  <done>D-19 forward ledger updated for all ERF + LDA-correlation residuals; commit landed; Plan 04-10 has the data needed for sign-off.</done>
</task>

</tasks>

<verification>
```bash
# 8.1 bisection report exists and routes
test -s /tmp/04-08-ldaerfx-bisection.txt && grep -cE "Verdict:.*Path [AB]" /tmp/04-08-ldaerfx-bisection.txt

# 8.2 fix landed (if Path A) — order-3 envelope met
cargo run -p validation --release -- --backend cpu --order 3 --filter 'ldaerfx' 2>&1 | grep "max_rel_err"

# 8.3 LDA-corr triage complete
grep -cE "COVERED|NEW" /tmp/04-08-lda-corr-triage.txt

# 8.4 D-19 ledger updated
grep -c "Plan 04-08 D-19 forward" .planning/STATE.md
grep -c "Phase 4 plan 04-08" .planning/REQUIREMENTS.md

# Tier-1 still GREEN
cargo test -p xcfun-eval --test self_tests --features testing 2>&1 | tail -3
```
</verification>

<success_criteria>
- Each of XC_LDAERFX, XC_LDAERFC, XC_LDAERFC_JT has a deterministic verdict: fixed (commit + 1e-7 envelope), or documented D-19 forward (REQUIREMENTS + STATE)
- LDA-correlation residuals (VWN3C/VWN5C/VWN_PBEC/PBEC/PZ81C/PW92C) categorised as "covered by Phase-2/3 forwards" or "new D-19 entry"
- /tmp/04-08-ldaerfx-bisection.txt contains the order-by-order analysis (artefact for Plan 04-10 review)
- Plan 04-10 ledger has all data needed to sign off the must_haves
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Per-functional kernel surgery (ldaerfx.rs) | Local source modifications; no external surface. |
| validation/report.jsonl | Gitignored; bisection reads it but does not commit it. |
| /tmp/04-08-*.txt artefacts | Local-only analysis; not committed; transient. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-08-01 | Tampering | Kernel "fix" that shifts error rather than removes it (local minimum) | mitigate | Every fix candidate must be verified by re-running `cargo run -p validation --release -- --backend cpu --order 3 --filter '<fn>' 2>&1 | grep max_rel_err` AND tier-1 self-tests must still pass at the upstream test_threshold. A fix that improves order 3 but regresses order 0/1/2 is not accepted. |
| T-04-08-02 | Repudiation | Silent forwarding without root-cause analysis | mitigate | Task 8.4 mandates STATE.md entries name each functional with its max_rel_err and the bisection verdict. A reader can trace from STATE → /tmp/04-08-bisection-archive (the bisection log is also referenced by SUMMARY.md). |
| T-04-08-03 | Denial of Service | Unbounded debugging time | mitigate | 90-minute budget per functional in Task 8.2 with explicit Path-B fallback. Total task budget = 4 hours (1h Task 8.1 + 3h × Task 8.2 ÷ at most 2 ERF retries → cap). |
| T-04-08-04 | Information Disclosure | None | accept | All artefacts are open-source numerics. No PII. |

No new code attack surface — the surgery is internal to the existing kernels and does not change the FFI signature.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-08-erf-divergence-SUMMARY.md`. Include:
- Per-ERF-functional verdict (fixed vs forwarded) and citation to commits / D-19 entries.
- LDA-correlation triage table.
- Reference to /tmp/04-08-*.txt artifacts (note: these are NOT committed; the SUMMARY.md captures their key conclusions for the historical record).
</output>
