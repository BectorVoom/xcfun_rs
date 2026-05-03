# Phase 6 — Deferred Items

Out-of-scope discoveries surfaced during plan execution. Tracked here for
future plans (Plans 06-N1/N2/N3 cleanup, or earlier follow-ups).

## Plan 06-06 Discoveries

- **Pre-existing test failure: `pbex_potential_non_2nd_taylor_vars_rejects`**
  in `crates/xcfun-eval/tests/potential_gga.rs`. The test asserts that
  `Functional::eval` returns `Err(XcError::InvalidVars { .. })` when
  `Mode::Potential` is invoked with a non-`*_2ND_TAYLOR` `Vars` arm
  (A_B_GAA_GAB_GBB), but the current `eval_setup` returns
  `Err(XcError::InvalidVarsAndMode { .. })` instead (combined-error
  variant added in Phase 5 D-08-A). The test predates Plan 06-06 and was
  failing on master before any 06-06 change — verified via `git stash`.
  **Action:** out-of-scope for Plan 06-06; track for Plan 06-N1 or
  earlier (test must update its `matches!` pattern from `InvalidVars` to
  `InvalidVars | InvalidVarsAndMode`, or `eval_setup` must be re-aligned
  with the original Phase 3 D-13 contract).
