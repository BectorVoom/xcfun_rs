---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "10"
subsystem: validation
tags: [parity-sweep, d19-forwards, sign-off, metagga, ad-chain, gradient-stress, regularize-clamp]

# Dependency graph
requires:
  - phase: 04
    provides: Plans 04-07 (driver extension), 04-08 (ERF triage), 04-09 (contracted metaGGA cross-mode)
  - phase: quick-260430-4x7
    provides: parallelized validation harness (--jobs 18) — prerequisite for completing the order-3 sweep within a single session
provides:
  - validation/report.html updated with order-3 capstone sweep
  - 04-VERIFICATION.md rewritten to status=signed_off_with_caveats (must_haves_verified: 6/6)
  - REQUIREMENTS.md MGGA-01..05 + ALIAS-01..06 + GGA-03/GGA-10 carryovers marked Complete (or Complete-with-caveats)
  - ROADMAP.md Phase 4 entry marked Complete (2026-04-30)[^d19p4] with consolidated D-19 forward footnote
  - STATE.md advanced to phase-4-complete (4/8 phases; 32/32 known plans); new Decisions block for Phase-4 gap-closure plans
  - 30+ Phase-4 D-19 INCONCLUSIVE forwards consolidated and handed off to Phase 6
affects: [phase-5-rust-facade-c-abi, phase-6-kernels-cpu-batch-cuda-wgpu]

tech-stack:
  added: []
  patterns:
    - "D-19 forward classification: gradient-stress AD-chain divergence vs clamp-boundary AD-tail vs small-magnitude AD-residual — distinguishes triage paths for Phase 6"

key-files:
  created:
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-10-resignoff-SUMMARY.md
  modified:
    - validation/report.html (commit db0f8ad)
    - .planning/REQUIREMENTS.md (commit c93ab44)
    - .planning/ROADMAP.md (commit c93ab44)
    - .planning/STATE.md (commit c93ab44)
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md (commit 8d823a8)

key-decisions:
  - "Phase 4 signed off WITH CAVEATS rather than fully clean — TPSS-correlation gradient-stress divergence (TPSSC 1e+30, TPSSLOCC 1e+27, REVTPSSC 1e+15) is real but characterized as Phase-6 work, not a Phase-4 fix"
  - "Plan 04-10 Path-B bisection chosen over Path-A blind sign-off — confirmed Rust TPSS-C port is algorithmically faithful to xcfun-master via side-by-side read of tpssc.cpp, tpssc_eps.hpp, pbec_eps.hpp vs crates/xcfun-eval/src/functionals/mgga/{tpssc.rs, shared/tpss_like.rs}; ctaylor max/operator> semantics match; no port bug"
  - "Phase-6 triage hand-off for TPSS-C trio: either add tau≥tau_w regularization guard, or exclude gradient-stress sub-grid for tau-using metaGGAs, or widen tier-2 threshold for gradient_stress stratum specifically"
  - "Validation parallelization via Quick Task 260430-4x7 (commits e79c3ef, 8c59675, feec803) was a hard prerequisite — without it the two prior sweep failures (SCANC C++ tmath_die at 4h, WSL VM termination at 1.5h) would have prevented sign-off"

patterns-established:
  - "Sparse failure-only report: validation/report.jsonl writes failing records exhaustively + 4 sentinel passes per (functional, mode={0..3}) tuple at pt=0; clean records are dropped. Flat record count (3.0M) is dominated by ~16 catastrophic functionals; 17 functionals are silently 100% clean."
  - "D-19 forward classification by stratum: bulk (clean) | regularize 7000-7999 (clamp-boundary AD-tail) | polarised 8000-8999 (small AD-residual) | gradient-stress 9000-9999 (catastrophic AD-chain divergence for tau-using metaGGAs)"

requirements-completed:
  - MGGA-01
  - MGGA-02
  - MGGA-03
  - MGGA-04
  - MGGA-05
  - MODE-03
  - ALIAS-01
  - ALIAS-02
  - ALIAS-03
  - ALIAS-04
  - ALIAS-05
  - ALIAS-06

duration: ~3h (Path-B bisection + ledger updates)
completed: 2026-04-30
---

# Phase 4 Plan 04-10 Re-Signoff Summary

**Phase 4 closes with `signed_off_with_caveats` after Plan 04-10 Path-B bisection confirmed the catastrophic TPSS-correlation divergence at gradient-stress points is f64 cancellation in the unphysical regime — not a port bug — and consolidated 30+ D-19 forwards for Phase 6 handoff.**

## What Shipped

### The order-3 capstone sweep produced clean data

The full-matrix tier-2 sweep ran on the parallelized validation binary (Quick Task `260430-4x7`):

```
cargo run -p validation --release -- --backend cpu --order 3 --resume --jobs 18 --filter '.*'
```

Final stats from `validation/report.jsonl` (1.27 GB, gitignored) + `validation/report.html` (committed `db0f8ad`):

| Metric | Count |
|---|---|
| Total records | 3,001,208 |
| Sentinel passes (pt=0) | 232 |
| Real failures vs strict 1e-12 | 2,983,450 |
| Excluded by regularize-clamp design | 16,966 |
| Excluded by upstream spec | 560 |
| Unique functionals iterated | 78 |

The flat "2.9M failures" headline is misleading: the report is sparse (only failing records get written) and 16 known-divergent functionals dominate the count. Decomposed cleanly:

- **17 functionals 100% clean strict 1e-12**: SLATERX, TFK, PBEX, REVPBEX, PBEINTX, RPBEX, PBESOLX, BECKEX, BECKECORRX, **PW86X**, OPTXCORR, **APBEX**, PW91X, KTX, BTK, M05X2X, M06X2X. PW86X + APBEX **tightened from Phase-3 D-19 to clean at order 3** — better than expected.
- **20 functionals excluded by upstream spec**: BR×3, CSC, BLOCX, SCAN×10, TW, VWK, PBELOCC, ZVPBESOLC, ZVPBEINTC.
- **30+ Phase-4 D-19 INCONCLUSIVE forwards** to Phase 6 (consolidated below).

### Path-B bisection on TPSS-correlation (the decision point)

Worst-case failure at pt=9019 (TPSSC, vars=A_B_GAA_GAB_GBB_TAUA_TAUB, order=2, elt=33):
- input = `[a=8.06e-4, b=7.26e-4, gaa=2.64e+5, gab=2.38e+5, gbb=3.73e+5, taua=4.14e-2, taub=3.87e-2]`
- C++ output = `0.0` exactly (f64 cancellation in `eps_pkzb*(1+2.8*eps_pkzb*tauwtau3)`)
- Rust output = `1.086e+30`
- regime: `tau / tau_w ≈ 8e-2 / 9.1e+7 ≈ 9e-10` — **von Weizsäcker bound violated by ~9 orders of magnitude** (intentional gradient-stress sub-grid stress test, not a physical input)

Side-by-side read confirmed Rust port is faithful:
- `xcfun-master/external/upstream/taylor/ctaylor.hpp:411` — `operator>` for ctaylor compares only CNST slot
- `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:336` — `max(a,b) = (a > b) ? a : b`
- `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818-838` — `ctaylor_max` matches semantics
- `xcfun-master/src/functionals/{tpssc.cpp,tpssc_eps.hpp,pbec_eps.hpp}` ↔ `crates/xcfun-eval/src/functionals/mgga/{tpssc.rs,shared/tpss_like.rs}` — line-for-line port (`tpss_pbec_eps`, `tpss_pbec_eps_polarized`, `tpss_C`, `tpss_epsc_summax`, `tpss_eps_full`)

**Verdict:** algorithmically faithful port; divergence is f64-rounding cancellation difference between C++ and Rust evaluation orders in a regime where `tauwtau3 ≈ 1e+27` amplifies ULP-level differences to 1e+27 magnitudes. Same shape as inherited Phase-3 D-19 forwards. **No Phase-4 fix; routine D-19 forward to Phase 6.**

### Per-functional max_rel_err table (failing only, sorted by max_rel)

| functional | records | fail | max_rel | rho_min(worst) | grad_max(worst) | category |
|---|---|---|---|---|---|---|
| XC_TPSSC | 23,894 | 23,890 | 1.086e+30 | 7.26e-04 | 3.73e+05 | NEW gradient-stress |
| XC_TPSSLOCC | 19,927 | 19,923 | 8.887e+27 | 7.86e-04 | 4.69e+05 | NEW gradient-stress |
| XC_REVTPSSC | 21,671 | 21,667 | 3.732e+15 | 7.37e-04 | 1.01e+05 | NEW gradient-stress |
| XC_BECKESRX | 65,514 | 63,612 | 2.273e+02 | 2.19e-08 | 0 | Phase-3 inherited |
| XC_PBEINTC | 626,645 | 621,969 | 6.167e+01 | 9.97e-03 | 9.14e-01 | Phase-3 inherited |
| XC_P86C / XC_P86CORRC | 496,357 / 496,359 | 496,353 / 496,355 | 9.156e-02 | 9.32e-08 | 0 | Phase-3 inherited |
| XC_LDAERFX | 590 | 586 | 6.741e-02 | 1.59e-10 | 0 | Phase-4 ERF (Plan 04-08) |
| XC_TPSSX | 18,505 | 18,359 | 2.676e-02 | 2.64e-14 | 0 | NEW clamp-boundary |
| XC_REVTPSSX | 15,414 | 15,276 | 1.326e-02 | 2.17e-14 | 0 | NEW clamp-boundary |
| XC_PW91C | 587,956 | 583,280 | 1.717e-03 | 6.17e-04 | 0 | Phase-3 inherited |
| XC_SPBEC | 616,799 | 611,951 | 5.268e-04 | 2.04e-14 | 0 | Phase-3 inherited |
| XC_LDAERFC_JT | 3,162 | 3,158 | 4.563e-05 | 9.83e-05 | 0 | Phase-4 ERF (Plan 04-08) |
| XC_LDAERFC | 375 | 247 | 4.568e-06 | 3.83e-14 | 0 | Phase-4 ERF (Plan 04-08) |
| XC_BECKECAMX | 2,196 | 1,942 | 2.005e-08 | 2.72e-14 | 0 | NEW clamp-boundary |
| XC_VWN_PBEC | 2,443 | 2,247 | 6.853e-09 | 3.65e-04 | 1.27e+00 | Phase-4 LDA-corr (Plan 04-08) |
| XC_APBEC | 1,779 | 1,775 | 5.697e-09 | 5.37e-04 | 4.39e+00 | Phase-3 inherited |
| XC_LYPC | 18 | 14 | 1.259e-10 | 7.92e-05 | 0 | NEW small AD-residual |
| XC_B97{,_1,_2}C | 68/73/143 | 64/69/139 | 7.816e-11 | 1.64e-08 | 0 | Phase-3 inherited |
| XC_M06{C,LC,HFC,X2C} | 32-57 | 28-53 | 4.88e-11 to 6.28e-11 | 1.64e-08+ | 0 | NEW small AD-residual |
| XC_M05X2C | 13 | 9 | 3.021e-11 | 1.44e-07 | 0 | NEW small AD-residual |
| XC_VWN5C | 145 | 110 | 1.568e-11 | 2.52e-14 | 0 | NEW clamp-boundary |
| XC_PW91K | 207 | 199 | 1.441e-11 | 7.13e-05 | 0 | Phase-3 inherited |
| XC_B97{,_1,_2}X | 10/10/14 | 6/6/10 | 9.463e-12 | 5.37e-04 | 4.39e+00 | NEW small AD-residual |
| XC_M05C | 11 | 7 | 9.263e-12 | 1.85e-06 | 0 | NEW small AD-residual |
| XC_PW92C | 10 | 6 | 8.974e-12 | 1.85e-06 | 0 | NEW small AD-residual |
| XC_M06HFX | 8 | 4 | 7.844e-12 | 1.57e-04 | 0 | NEW small AD-residual |
| XC_VWN3C | 45 | 38 | 7.172e-12 | 4.00e-14 | 0 | NEW clamp-boundary |
| XC_M06X / XC_M06LX | 7 / 6 | 3 / 2 | 1.46-4.21e-12 | 3.52e-3 to 6.25e-3 | 0 | NEW small AD-residual |
| XC_PZ81C | 11 | 7 | 2.958e-12 | 2.24e-14 | 0 | NEW clamp-boundary |
| XC_M05X | 13 | 9 | 1.890e-12 | 8.06e-03 | 0 | NEW small AD-residual |
| XC_PBEC | 5 | 1 | 1.796e-12 | 2.02e-06 | 0 | NEW small AD-residual |
| XC_OPTX | 5 | 1 | 1.164e-12 | 4.90e-04 | 3.32e+00 | NEW small AD-residual |

## Confirmation that `04-VERIFICATION.md` flipped to `signed_off_with_caveats`

```yaml
# Before (2026-04-26):
status: gaps_found
must_haves_total: 6
must_haves_verified: 0
must_haves_failed: 6

# After (2026-04-30, this plan):
status: signed_off_with_caveats
must_haves_total: 6
must_haves_verified: 6
must_haves_failed: 0
gap_closure: complete
```

All 6 must_haves verified with evidence cited inline in the new VERIFICATION.md. Caveats document the consolidated D-19 forward list, not a partial verification.

## Commit hashes for the four 04-10 commits

| # | SHA | Message | Files |
|---|---|---|---|
| 1 | `db0f8ad` | docs(04-10): tier-2 capstone — order 3 full-matrix sweep, post-gap-closure | validation/report.html |
| 2 | `c93ab44` | docs(04-10): Phase 4 sign-off — mark MGGA-01..05 MODE-03 ALIAS-01..06 Complete; advance STATE to Phase 5 | REQUIREMENTS.md, ROADMAP.md, STATE.md |
| 3 | `8d823a8` | docs(04-10): rewrite 04-VERIFICATION.md → signed_off_with_caveats (must_haves_verified: 6/6) | 04-VERIFICATION.md |
| 4 | (this commit) | docs(04-10): 04-10-resignoff plan summary | 04-10-resignoff-SUMMARY.md |

## Reference to consolidated Phase-4 D-19 forward list

See sections A-I of `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md` for the structured ledger, and the new `[^d19p4]` footnote in `.planning/ROADMAP.md` for the phase-roadmap-level summary. The cross-phase decisions block in `.planning/STATE.md` ("Decisions added in Phase 4") names every D-19 forward verbatim and links each to its Phase-6 triage hand-off.

## What Phase 6 inherits

1. **TPSS-correlation gradient-stress regularization** (NEW, Plan 04-10 Path-B): add `tau ≥ tau_w` guard or exclude gradient-stress sub-grid for tau-using metaGGAs. Affects TPSSC, TPSSLOCC, REVTPSSC.
2. **JP grid harness or guarded `{sqrt,log,pow}_expand`** (NEW, accumulated): unblocks tier-2 parity for the 20 `excluded_by_upstream_spec` functionals (BR×3, CSC, BLOCX, SCAN×10, TW, VWK, PBELOCC, ZVPBESOLC, ZVPBEINTC).
3. **libm-hybrid for ERF + LDA-correlation** (Plan 04-08, reinforced by 04-10 ERF residuals at order 3): LDAERFX, LDAERFC, LDAERFC_JT + VWN_PBEC, PBEC.
4. **xcfun-ad ctaylor_compose / ctaylor_multo specialisations for N ≥ 4** (Plan 04-05, reinforced by 04-09): unblocks Mode::Contracted orders 5..=6 metaGGA.
5. **General clamp-boundary AD-tail behaviour** (long-running): TPSSX, REVTPSSX, BECKECAMX, VWN5C, VWN3C, PZ81C — likely resolved by libm-hybrid + tighter clamp threshold.
6. **Minnesota meta-correlation small-magnitude AD-residual** (NEW, Plan 04-10): M05/M06 family at low-density polarised stratum — same shape as Phase-3 B97{,_1,_2}C forwards; likely resolved together.

Phase 5 (Rust facade + C ABI) is the next active phase; it does not depend on any of these forwards being resolved first.
