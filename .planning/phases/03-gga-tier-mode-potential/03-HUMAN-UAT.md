---
status: partial
phase: 03-gga-tier-mode-potential
source: [03-VERIFICATION.md]
started: 2026-04-25T22:00:00Z
updated: 2026-04-26T00:00:00Z
---

## Current Test

number: 2
name: Verify BECKESRX D-18 Strict 1e-12 Violation Hypothesis (1.05e-1 max rel_err)
expected: |
  Identify whether the failure mode is `erf_precise` cancellation (mirrors
  LDAERFX D-24 forensics) or kernel-level port-order drift; if the former,
  document a D-24-style upstream-sourced override; if the latter, file a
  Phase 6 fix path.
awaiting: user response (forensic analysis complete, see Tests §2 result + Gaps §1)

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
result: [pending]

## Summary

total: 3
passed: 0
issues: 1
pending: 2
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
