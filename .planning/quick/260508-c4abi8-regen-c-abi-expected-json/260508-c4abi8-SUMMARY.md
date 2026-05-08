---
quick_id: 260508-c4abi8
status: complete
date: 2026-05-08
---

# Quick Task 260508-c4abi8: Regenerate xcfun-capi c_abi expected.json

## What was done

Regenerated `crates/xcfun-capi/tests/fixtures/expected.json` against current master via
`cargo run -p xcfun-capi --example gen_expected --release`. The fixture had been baked at
commit `849419d` (Phase 5, 2026-04-30) and never refreshed after the 06-N7 substrate fixes
(`b0e4409`, `d204c69`, `df57c90`) shifted P86C and BECKEX outputs.

**Discovery during execution:** the gen_expected example regenerates only the JSON; the
C-side test (`crates/xcfun-capi/tests/c_abi.c`) carries hand-baked literal copies of each
fixture's `expected_<n>[]` array (per the file's own header comment lines 30–32). After the
JSON regen, `c_abi.c` still failed because its `expected_4[6]` block still held the
pre-06-N7 values. Synced the 6 hardcoded literals in fixture 4 (bp86) by hand to match the
new JSON. Other fixtures unchanged — diff stat: 6 insertions / 6 deletions in the JSON; 6
literal updates on a single block in `c_abi.c`.

## Files changed

- `crates/xcfun-capi/tests/fixtures/expected.json` — bp86 fixture #4 expected outputs updated
  (energy slots out[0..2] shifted by ~3e-9, gradient slots out[3..5] by ~1.2e-5)
- `crates/xcfun-capi/tests/c_abi.c` — `expected_4[6]` literals (lines 154–161) synced to
  match the regenerated JSON
- `.planning/todos/pending/2026-05-08-regenerate-c-abi-expected-json-after-06-n7-substrate-fixes.md`
  — deleted (resolved by this task)

## Verification results

- `cargo run -p xcfun-capi --example gen_expected --release` → wrote 10 fixtures
- `git diff --stat -- crates/xcfun-capi/tests/fixtures/expected.json` → 6 / 6 (only
  fixture #4, all 6 numeric slots; no other fixture moved)
- `cargo test -p xcfun-capi --test c_abi --release` → **1 passed; 0 failed** (31.58s)

The drift magnitudes match the documented Phase-3 D-19 envelope for P86C (max_rel 9.16e-2
in STATE.md / 9.2e-2 in ROADMAP.md) — the observed 1.2e-5 is well inside it. No D-19
update required.

## Out-of-scope, not addressed

- `xcfun-ad :: cbrt_expand_x0_0_1` and `cbrt_expand_x0_10` test failures — independent
  f32-seed bug, captured in `.planning/todos/pending/2026-05-08-cbrt-expand-f32-seed-bug.md`.
  Run `/gsd:debug` against that todo when ready.
- PW91C 1.8k validation-harness residual (CI workflow_dispatch only) — unchanged by this
  task; lives in the Phase-7+ validation workstream.

## Follow-up suggestion

The header comment on `c_abi.c` says fixtures are "generated once by `cargo run -p
xcfun-capi --example gen_expected`". That sentence implies the example writes both the JSON
*and* the C literals, but the example only writes the JSON — the C-side block is a manual
copy. Worth either (a) extending the example to also emit the C-array literals next to the
JSON, or (b) tightening the header comment to explicitly state the C literals are
hand-synced. Not in scope for this regen, but a 5-minute fix that would prevent the same
trip-up next time.
