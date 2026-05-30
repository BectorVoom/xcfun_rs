---
quick_id: 260530-f06fix
slug: f06-doc-exception
date: 2026-05-30
status: complete
---

# Quick Task 260530-f06fix — Summary

## What was done
Resolved the two red F-06 CI validation sweeps (order-3 run `26668931715`,
Mode::Potential run `26668932207`) via the user-approved documented-exception
(D-19/D-24) pattern. The failures were ULP-accumulation drift (order-3 GGAs) +
erf_precise cancellation (beckesrx/beckecamx), not correctness bugs.

## Commits
| Task | Commit | Change |
|------|--------|--------|
| 1+2 | `2667a9a` | `threshold_for` per-functional D-19/D-24 overrides (28 functionals, 4 buckets + beckesrx 1e-6) + exclude XC_BECKESRX from PartialDerivatives `run` path |
| 3 | `1f705af` | Drop beckesrx from `validate-order3-sweep.yml` (29→28) |
| 4 | (docs) | F-06 resolution record appended to `03-VERIFICATION.md` |

## Verification
- `cargo build -p validation --release` — ✅ succeeds
- `cargo test -p validation` — ✅ 15 tests pass (default features)
- `cargo fmt -p validation --check` — ✅ clean
- `validate-order3-sweep.yml` YAML — ✅ valid, 28-functional matrix, beckesrx absent
- Full sweep re-verification — runs on CI (re-triggered post-commit per execution split;
  NOT run locally)

## Key finding
F-06 item #2 answered: **BECKESRX is an erf_precise cancellation breakdown** (rel_err
0.177 in PartialDerivatives even above the 1e-3 clamp; LDAERFX D-24 analog), **not**
port-order drift. Excluded from strict PartialDerivatives; covered in Mode::Potential at
1e-6.

## Notes
- All thresholds are tight (next decade above the measured verdict-counting max), per the
  D-18 "no blanket relaxation" rule. Default gate stays strict 1e-12.
- Pre-existing `cargo build --workspace --release` / clippy CI failures are unrelated to
  this task.
