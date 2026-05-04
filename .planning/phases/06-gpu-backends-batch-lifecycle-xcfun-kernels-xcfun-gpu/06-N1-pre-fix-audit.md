# Phase 6 Plan 06-N1 — Pre-fix audit (Task 1)

**Date:** 2026-05-04
**Plan:** 06-N1 (D-19 bisection campaign for 11 inherited Phase-3/4 forwards)
**Status:** Substrate-deferred — see "Inconclusive — substrate gap" below.

## Context

This plan inherits the 11 Phase-3 GGA correlation/exchange/kinetic D-19
forwards still failing strict 1e-12 at order 3 in the Phase-4 sign-off
sweep (`validation/report.html`, committed `db0f8ad`). Per
`.planning/STATE.md` Phase-4 sign-off summary:

| Functional | Order-3 max_rel_err (Phase-4 baseline) | Suspected cause |
|------------|----------------------------------------|-----------------|
| PBEINTC    | 6.17e+1                                 | shared `pbec_eps` helper |
| BECKESRX   | 2.27e+2                                 | erf bracket cancellation; expect Plan 06-00 D-11 `erf_precise_taylor` to self-resolve |
| P86C       | 9.16e-2                                 | `pbec_eps` shared port |
| P86CORRC   | 9.16e-2                                 | `pbec_eps` shared port |
| PW91C      | 1.7e-3                                  | `pw91c_helper` AD-chain residual |
| SPBEC      | 5.3e-4                                  | `pbec_eps` shared port (variant of PBEC) |
| APBEC      | 5.7e-9                                  | `pbex` substrate; mid-magnitude residual |
| B97C       | 7.8e-11                                 | `b97_poly` near-zero polarised gradient_stress |
| B97_1C     | 7.8e-11                                 | `b97_poly` (same) |
| B97_2C     | 7.8e-11                                 | `b97_poly` (same) |
| PW91K      | 1.4e-11                                 | `pw91_like` mid-magnitude AD-residual |

## Substrate self-resolution hypothesis (Step 1 — RESEARCH §"D-19 Bisection Methodology")

Plan 06-00 substrate landed:

- `erf_precise_taylor` libm-hybrid AD-chain wrapper — should self-resolve
  BECKESRX and tighten any `Dependency::ERF` consumer.
- AD `ctaylor_multo_n4` / `ctaylor_compose_n4` specialisations — Mode::Contracted
  order-5 unblock only; no impact expected on `Mode::PartialDerivatives` order-3
  for the 11 forwards (which use AD orders 0..=3 with the existing `_set` /
  `_skipconst` recursion bodies).
- TPSS tau ≥ tau_w hard-clamp guard — TPSS-only; no impact on the 11 GGA
  forwards.
- mpmath sidecar — fixture-format placeholder; no run-time path active.

Plan 06-06 substrate landed:

- `Functional::weights` → `Vec<...>` (D-17) — no numerical impact (Send/Sync
  preserved; sums over `iter()` unchanged).
- `UnsafeCell<EvalHandle>` reusable handle (D-12) — structural only, fast
  path deferred; no numerical impact.
- 55 LDA × vars=6 launch arms (D-18) — adds dispatch coverage; LDA kernels
  unchanged; no impact on the 11 forwards (all are GGA, all already had
  vars=6 arms).

**Conclusion:** the only Plan 06-00 substrate axis that could plausibly
have affected the 11 forwards is `erf_precise_taylor` for BECKESRX. None
of the substrate work touched the `pbec_eps` / `b97_poly` / `pw91_like`
helpers that the Path-B hypothesis flags as the most likely root cause
for the other 10 forwards.

## Inconclusive — substrate gap

The validation tier-2 sweep cannot run in this worktree:
`xcfun-master/` (vendored C++ reference, ~3 MB tree) is not present
(gitignored, not a submodule, not vendored in this checkout).
`validation/build.rs` reads from `../xcfun-master/src/**/*.cpp` to compile
the C++ reference into the validation harness; without it,
`cargo run -p validation --release -- --backend cpu --order 3 --filter ...`
fails at `cc` time before any kernel-launch comparison can happen.

Path-B side-by-side bisection (Step 2 of the methodology) requires
reading the upstream C++ source (`xcfun-master/src/functionals/<name>.cpp`)
side-by-side with the Rust port. Without `xcfun-master/`, no Path-B
bisection is possible in this worktree.

**Therefore:** Tasks 1 (substrate self-resolution sweep) and 2 (Path-B
fix campaign) cannot be completed in this worktree. The plan's success
criteria stipulates that any forward not closed at strict 1e-13 must be
escalated via PLANNING INCONCLUSIVE — that escalation is the substantive
output of this plan in the absence of `xcfun-master/`.

## Tier-1 self-tests still GREEN (substrate intact)

The substrate work landed in Plan 06-00 and Plan 06-06 has not regressed
the existing tier-1 self-test ledger:

```
$ cargo test -p xcfun-eval --features testing --test self_tests --release
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All upstream-test-data-bearing functionals (P86C @ 1e-11, PW91C @ 1e-11,
plus the LDA / GGA / metaGGA functionals with `test_in`/`test_out` data)
continue to pass at their declared `test_threshold`. This confirms the
algorithmic-identity contract is preserved at the upstream-test-vector
density points; the D-19 forwards manifest at *order 3* density-grid
strata that the upstream `test_in` records do not cover.

## Recommended next step (post-orchestrator merge)

1. Restore `xcfun-master/` (re-download from upstream tag, or unstash from
   prior Phase-4 capstone artifacts).
2. Re-run `cargo run -p validation --release -- --backend cpu --order 3
   --jobs 18 --filter '^(pbeintc|beckesrx|p86c|p86corrc|pw91c|spbec|apbec
   |b97c|b97_1c|b97_2c|pw91k)$'` to confirm which forwards self-resolved
   and which need Path-B.
3. For each persistent forward, perform Path-B side-by-side reads of
   `xcfun-master/src/functionals/<name>.cpp` against the Rust port at
   `crates/xcfun-kernels/src/functionals/gga/<tier>/<name>.rs`.
4. Apply the per-functional fix; the per-functional unit tests created in
   Task 2 (this plan) become the GREEN gate.
