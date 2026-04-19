# Deferred items from Phase 1 plans

## Pre-existing cargo fmt drift (noted during Plan 01-07 Task 5)

`cargo fmt --check` reports ~84 formatting diffs across source files
created in Plans 01-01 through 01-06 and 01-05's fixture tooling. None of
these drifts originate from Plan 01-07's four new files (proptest_algebra,
mul_bench, compose_bench, check_no_fma) — those four are formatted clean.

Impact: `cargo fmt --check` exits 1 workspace-wide. Plan 01-07's Task 5
acceptance check `cargo fmt --check` exits 0 is violated by this
pre-existing drift.

Scope decision (per `<scope_boundary>` in execute-plan workflow): do
NOT run workspace-wide `cargo fmt` as part of Plan 01-07 because:
  1. The drift was not caused by Plan 07 changes.
  2. Touching 16+ files from earlier plans would inflate the Plan 07
     commit footprint and obscure the plan's actual scope.
  3. A dedicated housekeeping pass should land the formatting normalization
     as its own commit.

Proposed resolution: a Phase 0 CI-tightening task (or a manual
`chore(fmt): workspace-wide cargo fmt` commit before Phase 2 starts)
runs `cargo fmt` once and commits the normalization. Until then, CI should
NOT gate on `cargo fmt --check` for the files that currently drift.

Files with drift (partial list):
- crates/xcfun-ad/src/{ctaylor.rs, ctaylor_rec/{compose,mul,multo}.rs,
  expand/{cbrt,sqrt}.rs, math.rs, tfuns.rs}
- crates/xcfun-ad/tests/{ctaylor_unit,cubecl_spike,golden_{composed,expand,mul},
  math_unit,tfuns_unit}.rs
- xtask/src/bin/regen_ad_fixtures.rs
