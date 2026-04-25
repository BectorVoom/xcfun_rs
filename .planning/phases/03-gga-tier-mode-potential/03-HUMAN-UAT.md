---
status: partial
phase: 03-gga-tier-mode-potential
source: [03-VERIFICATION.md]
started: 2026-04-25T22:00:00Z
updated: 2026-04-25T22:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Order-3 Full-Matrix Tier-2 Capstone Re-run
expected: Either GREEN at strict 1e-12 across all 47 functionals, OR new D-19 entries documented with the same explicit-documentation rule (D-18); commit updated report.html + report.jsonl snapshot.
why_human: Order-3 capstone requires ~1h wall-clock + significant disk space (>500 MB report.jsonl); the executor previously hit usage-limit exhaustion mid-execution. A human-supervised run can monitor for similar issues and decide whether to commit the snapshot or split it.
result: [pending]

### 2. Verify BECKESRX D-18 Strict 1e-12 Violation Hypothesis (1.05e-1 max rel_err)
expected: Identify whether the failure mode is `erf_precise` cancellation (mirrors LDAERFX D-24 forensics) or kernel-level port-order drift; if the former, document a D-24-style upstream-sourced override; if the latter, file a Phase 6 fix path.
why_human: BECKESRX is a NEW D-19 entry (not inherited from prior phases). Per D-18, it must be explicitly documented (which it is, in 03-06-SUMMARY.md), but human review of the failure cluster would help calibrate whether Phase 6 mpmath-bridge or kernel-level rewrite is the appropriate fix path.
result: [pending]

### 3. Verify the Full 36-GGA Mode::Potential Sweep
expected: Either all 36 GGAs GREEN at strict 1e-12 in Mode::Potential, OR new D-19 entries documented per D-18.
why_human: Plan 03-05 directly verified only 8 of 36 GGAs (PBEX, BECKEX, LYPC, OPTX, KTX, BTK, B97X, B97C) — uniformly GREEN. The remaining 28 have not been run; the 13 functionals known-drifty in Mode::PartialDerivatives are likely also drifty in Mode::Potential. Full sweep ~14 minutes wall-clock.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
