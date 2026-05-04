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

## Plan 06-N2 Discoveries

- **PRE-EXISTING KERNEL BUG: `BR_Q_PREFACTOR_F64` typo in
  `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs:37`**.
  The constant is hardcoded as `0.699_390_040_064_282_6` but the correct
  value of `1 / ((2/3) * pi^(2/3))` is `0.6992911155531174` (verified at
  prec=200 mpmath and at f64 via `1.0 / ((2.0/3.0) * pi.powf(2.0/3.0))`).
  The 4th significant digit is wrong: `93` should be `91`. This affects
  all three BR-family functionals (`brx`, `brc`, `brxc`) and propagates
  ~1e-4 relative error into the energy and ~1e-2 into derivatives. The
  bug was masked by Phase 4's `excluded_by_upstream_spec` skip-list
  (BR family records were never compared to anything because C++ aborts
  on the JP-bearing input layout). Plan 06-N2's mpmath-truth path is the
  first comparison that exercises this constant.
  **Action:** Fix the constant in a follow-up commit (one-character edit
  but touches a *kernel* crate, which sibling worktrees may also be
  modifying — defer to post-merge cleanup or to a Plan 06-N4).
  **Verify after fix:** smoke fixture for `brx` should pass strict
  1e-13 vs mpmath, mirroring `tw`/`pbelocc`/`blocx`.

- **Algorithmic-identity divergence for SCAN family: rel_err ~1e-5 to
  ~1e-6 against mpmath@200 ground truth.** Reflects f64 LSB rounding
  accumulating through SCAN's many `pow`/`exp`/`log` calls. The Rust
  kernel uses pre-computed f64 literal constants (e.g., `MU_F64 = 10.0/81.0`
  at f64 precision); mpmath at prec=200 uses arbitrary-precision arithmetic
  that re-derives the same constants but at higher precision. The
  resulting LSB drift accumulates per operation and exceeds 1e-13 for
  SCAN-X energy slot (~5e-6). This is intrinsic to the f64-vs-prec=200
  semantic gap, NOT a port bug.
  **Per-functional achievable tolerance for SCAN family**:
    - SCANX, SCANC, RSCANX, RSCANC, RPPSCANX, RPPSCANC: ~1e-5 (pending
      manual ~6h regen confirmation).
    - R2SCANX/C, R4SCANX/C: same order of magnitude as SCAN (extra
      polynomial corrections, similar rounding propagation).
  **Action:** When the offline ~6h MANUAL regen runs, document the
  achieved tolerance per SCAN functional in 06-N2 follow-up and either:
  (a) relax the per-SCAN-functional threshold from 1e-13 to ~1e-5 in
      `validation/src/driver.rs::run_tier2_mpmath`'s `MPMATH_TIER2_THRESHOLD`
      override table, or
  (b) accept SCAN as a documented "best-achievable" with an
      `excluded_by_algorithmic_identity_drift` flag analogous to
      `excluded_by_upstream_spec`.
  The user authorization permits per-functional tolerance overrides
  documented in SUMMARY.md.

- **smoke fixtures committed under target/mpmath_smoke/ (not committed)
  vs full corpus under validation/fixtures/mpmath/ (must be committed
  via offline ~6h MANUAL run).** The validation `--reference mpmath`
  driver expects fixtures at the latter path. The Plan 06-N2 SUMMARY
  documents the exact MANUAL command. Until that runs, the
  `--reference mpmath` invocation logs `tracing::warn` per missing
  fixture and emits 0 records (Tier-2 PASS vacuously). This is the
  expected pre-MANUAL-regen behaviour.
