---
created: 2026-04-26T02:09:26.920Z
title: Phase 3 UAT Test 1 — order-3 full-matrix tier-2 capstone re-run
area: planning
files:
  - .planning/phases/03-gga-tier-mode-potential/03-HUMAN-UAT.md:22-25
  - .planning/phases/03-gga-tier-mode-potential/03-VERIFICATION.md:168-172
  - .planning/phases/03-gga-tier-mode-potential/03-06-SUMMARY.md:139-145
  - validation/src/driver.rs:255-518
  - target/release/validation
---

## Problem

Phase 3 UAT (`.planning/phases/03-gga-tier-mode-potential/03-HUMAN-UAT.md`) has Test 1 outstanding: the order-3 full-matrix tier-2 capstone re-run that the Wave-6 executor started but aborted on usage-limit exhaustion (per `03-06-SUMMARY.md` §"Decisions Made" item 5 + §"Documented deviations" item 1). Without it, MODE-01's order-3 cell remains unverified at the parity level — only structural cover exists (W9 unit tests at inlen=2 exhaustive + C++ fall-through fix in commit 09b6831 + lib unit tests).

This is the last open test in the partial UAT (Tests 2 + 3 already resolved as `issue` with diagnoses committed in `29fe026`, `707b37c`; driver fixes in `6435aab`; D-26 classification in `1436413`).

Why human-supervised: ~1 hour wall-clock, ~500 MB `report.jsonl` output that overwrites the existing committed reference snapshot. The prior aborted attempt left the worktree with a partial 14-line increment + 1.2M-line truncation that the orchestrator had to manually discard.

## Solution

Resume via `/gsd-verify-work 3` — it will pick up Test 1 from the partial UAT and the `present_test` step renders Test 1's checkpoint.

Steps the session should take:

1. **Run** `./target/release/validation --backend cpu --order 3` (NOT `cargo run` — direct binary avoids cargo overhead; binary is at `target/release/validation` after `cargo build -p validation --release`). Wall-clock ~1 hour. The harness writes incrementally to `validation/report.jsonl` so a mid-run abort still leaves partial data; `report.html` only writes at the end.

2. **If the run completes** with `Tier-2 done: N records, M failed`:
   - Update Test 1 result in `03-HUMAN-UAT.md`: `result: pass` if `M == 0` (modulo the 14 known D-19 entries from PartialDerivatives + Mode::Potential), or `result: issue` with new D-19 entries cataloged
   - Commit refreshed `validation/report.html` + `validation/report.jsonl` (overrides current order-2 snapshot — that's the intended behavior; order-3 strictly subsumes order-2 since the harness runs orders 0..=min(max_order, 3))
   - Update UAT status: `partial` → `complete`
   - Catalog any new D-19 entries against the existing 14-entry collective sign-off in `03-06-SUMMARY.md` §"D-19 Collective Sign-Off"
   - Classify any new entries per the **D-26 decision-tree** in `03-CONTEXT.md` (port-order drift / cubecl polyfill / Ekström-FIXME)
   - If any new Ekström-FIXME-class entries surface, extend the D-26 entry's "Affected functionals" list

3. **If the run aborts mid-way** (usage-limit, OOM, signal): leave UAT at `partial`, document the abort point + last-completed functional, and re-attempt with the Wave-6 mitigation pattern (split by functional via `--filter` regex over disjoint subsets).

4. **Don't forget the documentation patch** queued in `03-HUMAN-UAT.md` Gaps §2 missing[3]: `03-VERIFICATION.md:170` and `03-06-SUMMARY.md:196` reference `cargo xtask validate` which is not a real cargo subcommand here. Real invocations: `./target/release/validation`, `cargo run -p validation --release --`, or `cargo run -p xtask --bin validate --release --`. Fold into the same commit as the Test 1 result.

## Context for the future session

- 14 known D-19 entries (13 from Phase 3 PartialDerivatives + XC_BECKECAMX from Mode::Potential): expected to also fail the order-3 capstone if they fail at order 2; new order-3-only failures would add to the list.
- C++ harness order cap: `min(max_order, 3)` per `validation/src/driver.rs:360`; case 4 hits `xcfun::die`. The Rust side supports orders 0..=4 self-consistently per MODE-01 D-16.
- Recent commits this UAT session: `29fe026 707b37c 6435aab 1436413` — read those if context is needed.
