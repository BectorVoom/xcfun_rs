---
created: 2026-05-08T00:00:00.000Z
title: Regenerate xcfun-capi c_abi expected.json after 06-N7 substrate fixes
area: testing
priority: medium
files:
  - crates/xcfun-capi/tests/fixtures/expected.json
  - crates/xcfun-capi/tests/c_abi.rs
  - crates/xcfun-capi/examples/gen_expected.rs
  - crates/xcfun-ad/tests/expand_primary.rs:438
  - crates/xcfun-ad/tests/expand_primary.rs:453
---

## Problem

`cargo test -p xcfun-capi --test c_abi` fails on **fixture #4 (`bp86`, order 1, mode 1)** — 6 of 6 output slots out of tolerance. The energy slots (out[0..2]) drift at rel ≈ 1.6e-9 / 3.0e-9; the gradient-derivative slots (out[3..5]) drift at rel ≈ 7.4e-6 / 1.2e-5.

Representative tuples (from CPU/default-features run, master @ `6bcd502`):
- bp86 fix#4 out[3]: rel 7.40e-6, expected -6.5485870144203944e-03, actual -6.5486354441021324e-03
- bp86 fix#4 out[4]: rel 1.21e-5, expected  8.0029559962793689e-03, actual  8.0028591369158929e-03
- bp86 fix#4 out[0]: rel 1.62e-9, expected -8.0924471125415998e-01, actual -8.0924471256234409e-01

This is **not** a regression in computation — it's stale baseline. `expected.json` was baked at commit `849419d` (Phase 5, 2026-04-30) and never regenerated after the 06-N7 substrate fixes shifted P86C and BECKEX outputs:

- `b0e4409` "fix(06-N7/07-00): correct SPBEC + PW91C + P86 precomputed constants (GREEN)"
- `d204c69` "fix(06-N7/07-00): align becke SQRT_PI_F64 to 1.7724538509055159 (1-ULP)"
- `df57c90` "fix(06-N7/07-00): PW91C 1/1000 multiplier f32-truncation bug"

`bp86` = BECKEX + P86C, so both halves moved. P86C is also on the documented Phase 3/4 D-19 forward list (max_rel 9.16e-2 in STATE.md / 9.2e-2 in ROADMAP.md), so the 1.2e-5 here is well inside the existing relaxed envelope — the c_abi fixture just hasn't picked up the new ground truth.

PW91C is **not** part of this failure. The 541k→1.8k PW91C residual lives in the validation-harness order-3 sweep, not in `cargo test`.

## Solution

Regenerate `expected.json` against current master and re-baseline:

```bash
cargo run -p xcfun-capi --example gen_expected --release
cargo test -p xcfun-capi --test c_abi
```

The example writes to `crates/xcfun-capi/tests/fixtures/expected.json` (per `examples/gen_expected.rs:229`). After regen:

1. Diff the new fixture vs the committed one — confirm the only material drift is the bp86 entry; if other fixtures shifted unexpectedly, treat as a separate investigation rather than rubber-stamping.
2. Verify `cargo test -p xcfun-capi --test c_abi` passes.
3. Note in the commit message which substrate commits motivated the regen (cite `b0e4409`, `d204c69`, `df57c90`).
4. Cross-check the bp86 P86C drift against the D-19 forward list — if the new error stays inside the documented 9.16e-2 envelope (it should: 1.2e-5 ≪ 9.16e-2), no D-19 update needed.

Single commit, single concern. Should be a 5–10 minute job.

## Context

- Why this matters: `c_abi_drop_in_test` is the only ABI golden test we have for the C drop-in path; leaving it red masks future real regressions.
- Out of scope: PW91C 1.8k residual (validation harness, separate workstream); cbrt_expand f32-seed failures (separate todo, `2026-05-08-cbrt-expand-f32-seed-bug.md`).
- Discovered during `/gsd:explore resolve phase 07-00 test and cargo test failed` (2026-05-08) — phase 07-00 itself is COMPLETE per its STATE.md; this fixture rot is the actual surface failure.
