---
status: partial
phase: 03-gga-tier-mode-potential
source: [03-VERIFICATION.md]
started: 2026-04-25T22:00:00Z
updated: 2026-04-26T01:50:00Z
---

## Current Test

number: 1
name: Order-3 Full-Matrix Tier-2 Capstone Re-run
expected: |
  Either GREEN at strict 1e-12 across all 47 functionals, OR new D-19
  entries documented with the same explicit-documentation rule (D-18);
  commit updated report.html + report.jsonl snapshot.
awaiting: human-supervised order-3 run (deferred — ~1h wall-clock + risk of usage-limit exhaustion mid-execution; not safe to attempt within current session)

## Tests

### 1. Order-3 Full-Matrix Tier-2 Capstone Re-run
expected: Either GREEN at strict 1e-12 across all 47 functionals, OR new D-19 entries documented with the same explicit-documentation rule (D-18); commit updated report.html + report.jsonl snapshot.
why_human: Order-3 capstone requires ~1h wall-clock + significant disk space (>500 MB report.jsonl); the executor previously hit usage-limit exhaustion mid-execution. A human-supervised run can monitor for similar issues and decide whether to commit the snapshot or split it.
result: [pending]

### 2. Verify BECKESRX D-18 Strict 1e-12 Violation Hypothesis (1.05e-1 max rel_err)
expected: Identify whether the failure mode is `erf_precise` cancellation (mirrors LDAERFX D-24 forensics) or kernel-level port-order drift; if the former, document a D-24-style upstream-sourced override; if the latter, file a Phase 6 fix path.
why_human: BECKESRX is a NEW D-19 entry (not inherited from prior phases). Per D-18, it must be explicitly documented (which it is, in 03-06-SUMMARY.md), but human review of the failure cluster would help calibrate whether Phase 6 mpmath-bridge or kernel-level rewrite is the appropriate fix path.
result: issue
reported: |
  Failure mode is NEITHER `erf_precise` cancellation (D-24 pattern) NOR
  kernel-level port-order drift. Root cause is algorithmic instability in
  the C++ formula itself, flagged by the original author Ulf Ekström with
  a `// FIXME` comment at xcfun-master/src/functionals/beckex.cpp:38-44:
  "The erf + something is basically the erf minus its asymptotic expansion.
  This is horribel for numerics, will have to code a special function."
  The same author DID implement that special function for LDAERFX
  (ldaerfx.cpp:35-50, three-branch piecewise: a<1e-9 / a<100 / a<1e9 / else)
  but NEVER applied it to becke_sr. Both Rust and C++ use the unstable
  intermediate-only formula; their disagreement is rounding noise inside
  catastrophic cancellation. Recommended fix: NOT a D-24-style 1e-7 override
  (5,754 records would still fail and the underlying numerics is broken in
  both implementations); instead, Phase 6 implements the asymptotic-expansion
  bridge mirroring the LDAERFX C++ piecewise pattern. Once the bridge ships,
  Rust will be MORE accurate than C++ in this regime — same precedent as
  LDAERFX D-24 (Rust = mpmath truth; C++ itself diverges).
severity: major

### 3. Verify the Full 36-GGA Mode::Potential Sweep
expected: Either all 36 GGAs GREEN at strict 1e-12 in Mode::Potential, OR new D-19 entries documented per D-18.
why_human: Plan 03-05 directly verified only 8 of 36 GGAs (PBEX, BECKEX, LYPC, OPTX, KTX, BTK, B97X, B97C) — uniformly GREEN. The remaining 28 have not been run; the 13 functionals known-drifty in Mode::PartialDerivatives are likely also drifty in Mode::Potential. Full sweep ~14 minutes wall-clock.
result: issue
reported: |
  Human ran ./target/release/validation --backend cpu --mode potential --order 2
  on 2026-04-26 (~17 min wall-clock). Pre-requisite driver fix landed first
  (commit 6435aab — `run_potential` was missing the upstream-spec skip-list
  from `run`, causing hard abort on XC_TW; same commit added a false-green
  filter-mismatch warning). Final harness verdict: 1,230,005 records evaluated,
  17,375 failed across 10 functionals (5 upstream-spec markers excluded;
  51,634 clamp-stratum records excluded per D-22).

  Per-functional breakdown (failures, max_rel_err, status vs PartialDerivatives D-19):
    XC_PBEINTC      2,185   3.757e-01   inherited (was 1.9 in PD)
    XC_APBEX        3,000   2.063e-01   inherited
    XC_P86C         1,070   5.527e-04   inherited
    XC_P86CORRC     1,070   5.527e-04   inherited
    XC_PW91K        2,813   2.161e-05   inherited
    XC_SPBEC        3,000   1.271e-05   inherited
    XC_PW86X        2,999   1.033e-05   inherited
    XC_BECKESRX        76   4.693e-07   inherited (severity drops vs PD's 1.05e-1)
    XC_PW91C        1,128   3.333e-07   inherited
    XC_BECKECAMX       34   7.881e-09   *** NEW D-19 *** (PD: GREEN at 6.97e-12)

  Cross-mode pattern: 9 of 13 PartialDerivatives D-19 entries also fail in
  Mode::Potential (with broadly reduced max_rel_err since Potential output is
  2-3 elements vs ~20 for order-2 PD); 4 of 13 PD D-19 entries (APBEC, B97C,
  B97_1C, B97_2C) are GREEN in Potential — they only drift in higher-order
  PD output slots and don't surface here. Plan 03-05's 8-GGA sample (PBEX,
  BECKEX, LYPC, OPTX, KTX, BTK, B97X, B97C) all hold GREEN in this run too,
  confirming the sample wasn't biased.

  Net new finding: XC_BECKECAMX must be added to the D-19 collective sign-off
  per D-18 (no blanket relaxation). Total D-19 count goes from 13 → 14.
severity: major

## Summary

total: 3
passed: 0
issues: 2
pending: 1
skipped: 0
blocked: 0

## Gaps

- truth: "BECKESRX holds strict 1e-12 vs C++ across the 9.86M-record tier-2 capstone"
  status: failed
  reason: |
    User reported: tier-2 order-2 run shows 22,970/180k BECKESRX records failing
    strict 1e-12; max rel_err = 1.053e-01 at point_idx=7421 (a=b=6.95e-10,
    gaa=gab=gbb=0); 84 records fail >= 1e-3, all at density ∈ [2.47e-12, 2.65e-8]
    with zero gradient. Failure distribution: 7,463 in 1e-12..1e-11 (rounding
    noise from cancellation), 11,596 in 1e-11..1e-6 (asymptotic-regime drift),
    3,827 in 1e-6..1e-3 (deep cancellation), 84 ≥ 1e-3 (catastrophic).
    Order distribution: 0 at order=0, 4,559 at order=1, 18,411 at order=2 —
    cancellation amplifies with derivative order. Threshold sensitivity: relaxing
    to 1e-7 leaves 5,754 failures; relaxing to 1e-4 still leaves 515 failures.
  severity: major
  test: 2
  root_cause: |
    Algorithmic catastrophic cancellation in becke_sr formula
    `(sqrt(π)·erf(1/(2a)) + 2a·(b - c))` when `a → ∞` (i.e., when na^(1/3) → 0
    via low density and zero gradient). Original C++ author Ulf Ekström flagged
    this with `// FIXME: The erf + something is basically the erf minus its
    asymptotic expansion. This is horribel for numerics, will have to code a
    special function.` (xcfun-master/src/functionals/beckex.cpp:38-44). The
    "special function" was implemented for LDAERFX (ldaerfx.cpp:35-50: piecewise
    branches at a<1e-9, a<100, a<1e9, else) but never backported to becke_sr.
    Rust port is bit-faithful to the unstable C++ formula; the Rust↔C++
    disagreement is rounding noise inside the cancellation, not a port defect.
  artifacts:
    - path: "xcfun-master/src/functionals/beckex.cpp"
      issue: "FIXME at lines 38-44; becke_sr lacks asymptotic-expansion branch"
    - path: "xcfun-master/src/functionals/ldaerfx.cpp"
      issue: "lines 35-50 — reference implementation of the piecewise pattern that becke_sr needs"
    - path: "crates/xcfun-eval/src/functionals/gga/becke/beckesrx.rs"
      issue: "bit-faithful port of unstable formula; needs asymptotic branch when 1/(2a) << 1"
    - path: "validation/report.jsonl"
      issue: "22,970 failing BECKESRX records; max rel_err 1.053e-01 at point_idx=7421"
  missing:
    - "Phase 6: implement piecewise asymptotic-expansion bridge in `becke_sr` mirroring LDAERFX three-branch pattern"
    - "Phase 6 (or now): commit a CONTEXT.md entry (suggested: D-26) capturing the 'Ekström FIXME' classification — distinct from D-24 (cubecl polyfill divergence) and D-19 (port-order drift)"
    - "Phase 6: extend mpmath-bridge ground truth to cover BECKESRX so 'Rust > C++' accuracy claim is provable"
    - "REJECT D-24-style 1e-7 override for BECKESRX: would still leave 5,754 records failing AND would mask the algorithmic-instability finding"
  debug_session: ""

- truth: "All 36 GGA functionals hold strict 1e-12 vs C++ in Mode::Potential across the full validation grid"
  status: failed
  reason: |
    Human ran ./target/release/validation --backend cpu --mode potential --order 2
    on 2026-04-26 with the patched driver (commit 6435aab); 1,230,005 records
    evaluated, 17,375 failed across 10 functionals (5 upstream-spec markers
    excluded; 51,634 clamp-stratum records excluded per D-22). Pre-fix the run
    aborted at XC_TW (see Bug 1 below). 9 of 10 failing functionals are
    inherited from the PartialDerivatives 13-entry D-19 list with broadly
    reduced max_rel_err (Potential output is 2-3 elements vs ~20 for order-2
    PD). One NEW D-19 entry: XC_BECKECAMX (max_rel_err 7.881e-09; was GREEN
    at 6.97e-12 in PartialDerivatives — the Potential code path exposes drift
    invisible in PD).
  severity: major
  test: 3
  root_cause: |
    Three distinct failures bundled in this gap:

    (a) NEW D-19: XC_BECKECAMX — drift mode is mode-specific (PD GREEN at
        6.97e-12, Potential RED at 7.88e-9). beckecamx is the range-separated
        Becke variant carrying a different parameter set (RANGESEP_MU + CAM_ALPHA
        + CAM_BETA) vs. beckesrx; the Potential code path's CTaylor<f64,2>
        divergence accumulator likely amplifies a small kernel-side rounding
        difference that the PD output's 20-element layout averages out. The
        absolute magnitude (7.88e-9) is below the LDAERFX D-24 threshold (1e-7)
        but above the strict 1e-12 contract.

    (b) 9 inherited PD D-19 entries failing in Potential too — same kernel
        bodies as in PartialDerivatives, so same root causes (port-order drift
        for most; Ekström-FIXME cancellation for BECKESRX).

    (c) Driver-side bugs (now fixed in commit 6435aab): `run_potential` was
        missing the upstream-spec skip-list — caused hard abort on XC_TW
        before this Test 3 could ever complete; both `run` and `run_potential`
        silently produced "PASS" over 0 records when the --filter regex
        case-mismatched (lowercased name comparison vs uppercase regex).
  artifacts:
    - path: "validation/src/driver.rs"
      issue: "FIXED in commit 6435aab: run_potential skip-list parity + false-green guard"
    - path: "crates/xcfun-eval/src/functionals/gga/becke/beckecamx.rs"
      issue: "NEW D-19: 7.88e-9 max_rel_err in Mode::Potential vs strict 1e-12 (PD: GREEN at 6.97e-12)"
    - path: "validation/report.jsonl"
      issue: "1,230,005 records / 17,375 real failures (Mode::Potential, 2026-04-26 run)"
    - path: ".planning/phases/03-gga-tier-mode-potential/03-VERIFICATION.md"
      issue: "lines 153 + 184 — Mode::Potential SATISFIED claim was based on 8-GGA hand-picked sample; full sweep adds 1 NEW D-19 entry. Update verification report after this UAT lands."
  missing:
    - "Phase 6: investigate XC_BECKECAMX Potential-only drift (mode-specific failure mode requires kernel-side analysis distinct from the existing 13 entries)"
    - "Phase 6: D-19 collective sign-off list grows from 13 to 14 entries; update 03-06-SUMMARY.md §'D-19 Collective Sign-Off' or carry-forward in the receiving Phase 6 CONTEXT.md"
    - "Phase 3 retroactive: optional update to 03-VERIFICATION.md acknowledging that the original 'Mode::Potential SATISFIED' claim was on the 8-GGA sample only; the 36-GGA sweep is now complete and reveals 1 net-new D-19"
    - "Documentation: 03-VERIFICATION.md:170 + 03-06-SUMMARY.md:196 reference `cargo xtask validate` which is not a real cargo subcommand here; the actual invocations are `./target/release/validation` or `cargo run -p validation --release --` or `cargo run -p xtask --bin validate --release --`"
  debug_session: ""
