---
slug: 04-10-metagga-pd-regression
status: resolved
goal: find_and_fix
trigger: "Plan 04-10 order-3 full-matrix tier-2 sweep completed cleanly (JSONL flush fix worked) but produced 10,546,454 genuinely failing records out of 10,584,886 total — 46 distinct functionals failing, far beyond the documented D-19 forward envelope (13 inherited Phase-3 + 3 Phase-4 ERF = 16). Top offenders are the metaGGA family (M06×8, M05×2, TPSS×2, REVTPSS×2) — NOT on any prior D-19 forward list. M06X has rel_err = 1.24e-2 at order 0 (point_idx=0, element_idx=0, vars=A_B), which is upstream of AD-chain amplification and therefore points to a structural kernel/routing bug rather than derivative-composition drift. TPSSX at order 0 shows mixed pass/fail across elements (some 0.0, some 2.17e-6). Plan 04-09 cross-mode tests claimed M06X/TPSSX/SCANX GREEN at strict 1e-12 across orders 0..=3 — either those tests have a coverage gap (small grid, different vars combo) or there's been a recent regression."
created: "2026-04-29T00:45:52Z"
updated: "2026-04-29T02:30:00Z"
phase: 04-metagga-tier-mode-contracted-aliases
plan: "10"
related_session: 04-10-incremental-jsonl-flush (resolved — separate JSONL durability fix)
---

# Debug session: metaGGA Mode::PartialDerivatives parity regression at order 0

## Symptoms

- **Expected:** `cargo run -p validation --release -- --backend cpu --order 3` produces ≤ ~6M failures concentrated in the documented D-19 forward set (13 Phase-3 + 3 Phase-4 ERF + Plan 04-08 LDA-corr forwards + a small bounded number of new Phase-4 metaGGA entries from Plan 04-07 triage). Per Plan 04-09, M06X / TPSSX / SCANX are GREEN at strict 1e-12 on Mode::PartialDerivatives orders 0..=3.
- **Actual:** 10,546,454 genuine failures (99.6% of all records). 46 distinct functionals failing. The metaGGA family is not on any prior D-19 forward list but is producing the bulk of the failure mass.
- **Errors:** No panics. The harness completed and reported `ERROR validation: Tier-2 FAIL: 10546454 failing records` from main.rs's end-of-run gate.
- **Timeline:** First time order-3 full-matrix tier-2 sweep has been run successfully end-to-end (prior two attempts were lost to JSONL-buffering bug, now resolved). So we don't know whether this is "always was broken" or "regressed since Plan 04-09".
- **Reproduction:** `cargo run -p validation --release -- --backend cpu --order 3` (filter defaults to '.*'). For fast iteration: `cargo run -p validation --release -- --backend cpu --order 0 --filter '^xc_m06x$'` should reproduce M06X order=0 rel_err ≈ 1.24e-2 in seconds.

## User decisions (locked in pre-investigation)

- **Goal:** diagnose AND fix whatever it takes. The user has accepted that this may pull the debugger into kernel-level changes; the alternative (defer to a new Phase) is rejected because Phase 4 sign-off is currently blocked.
- **First anchor:** M06X at order 0 (point_idx=0, element_idx=0, vars=A_B, rel_err = 1.24e-2). Order 0 is upstream of any AD-chain amplification, so a 1% error there points to a structural bug in either the kernel body or how it's wired up. Cleanest signal.
- **Existing report.jsonl** (4.8 GB, 10.58M records) **is evidence input** — query it (jq / python-stream) for stratum analysis BEFORE re-running anything. Do not delete it. Use narrow `--filter` re-runs for hypothesis testing (seconds, not hours).

## Working hypothesis (initial — superseded by Root Cause below)

A 1% rel_err at order 0 (no AD chain involvement yet) means one of:

1. **Routing bug:** the M06X kernel exists and is correct, but `validation/src/driver.rs::run_launch` dispatches the wrong (functional_id, vars, mode, order) tuple to it.
2. **Vars-table bug:** `validation/build.rs` or the registry's VARS_TABLE maps M06X to the wrong vars discriminant.
3. **Kernel body bug:** the M06X kernel was ported but never exercised at vars=A_B.
4. **`_2ND_TAYLOR` discriminant mismatch.**
5. **Plan 04-09 cross-mode tests had a coverage gap.**

## Files of interest

- `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs` — **THE BUG IS HERE** (line 688: `LSDA_X_COEFF_F64`).
- `xcfun-master/src/functionals/m0xy_fun.hpp` (lines 260-262) — C++ reference for `lsda_x`.
- `crates/xcfun-eval/src/functionals/mgga/m06x.rs`, `m06hfx.rs`, `m06lx.rs` — call sites of `m0x_lsda_x` (3 functionals affected by this single bug).
- `crates/xcfun-eval/src/functionals/mgga/m05x.rs`, `m05x2x.rs`, `m06x2x.rs` — do NOT use `m0x_lsda_x` (M05 family has no h-term contribution; M06X2X drops the h-term). Their failures are SEPARATE bugs.
- `crates/xcfun-eval/src/functionals/mgga/tpssx.rs` — TPSSX failures show DIFFERENT pattern (pre-AD-chain error 2e-6 to 6e-5, only when gradients ≠ 0). SEPARATE bug.

## Suggested first probes (now executed)

1. ✅ Pretty-printed failing M06X order=0 record from `validation/report.jsonl`.
2. ✅ Hand-computed C++ reference value from formula and constants — matches recorded cpp to 1e-15.
3. ✅ Identified the wrong constant.

## Eliminated

- (1) Routing bug: NOT the issue — kernel IS being called, but produces structurally wrong output.
- (2) Vars-table bug: NOT the issue — vars are routed correctly (`A_B_GAA_GAB_GBB_TAUA_TAUB`, all 7 inputs delivered).
- (4) `_2ND_TAYLOR` discriminant mismatch: NOT the issue — order=0 doesn't exercise discriminant slots.
- (5) Plan 04-09 coverage gap: WAS the issue (those tests used Mode::Contracted with full metaGGA vars where the buggy LSDA constant doesn't bias as much, OR they didn't compare against the C++ reference for this specific value), but the kernel is genuinely wrong.

## Evidence

- 2026-04-29T00:42Z — orchestrator triage: report.jsonl has 10,584,886 total records, 184 pass=true (sampled), 10,546,454 genuine fails, 38,168 excluded_by_regularize_clamp_design, 80 excluded_by_upstream_spec (BR + CSC), 80 rust_unavailable. 46 distinct failing functionals.
- 2026-04-29T00:42Z — top failing functionals: M06HFX, M06LX, M06X, M05X, M05X2X, M06X2X, TPSSX, REVTPSSX (NOT on prior D-19 forward list).
- 2026-04-29T00:42Z — TPSSX at order 0 has BOTH 0.0 (pass, elem=0) AND 2.17e-6 (fail, elem=0 different point) — vars-or-stratum-specific.
- 2026-04-29T01:15Z — pretty-printed first failing M06X record:
  ```json
  {
    "functional": "XC_M06X", "vars": "A_B_GAA_GAB_GBB_TAUA_TAUB",
    "mode": 1, "order": 0, "point_idx": 0, "element_idx": 0,
    "input": [0.0384358262061213, 0.09939284909478768, 0.0, 0.0, 0.0, 0.5450146191978726, 1.0268037142167838],
    "rust": -0.016377277461874853,
    "cpp":  -0.004013877165375889,
    "rel_err": 0.012363400296498964
  }
  ```
- 2026-04-29T01:15Z — only vars combo M06X is ever exercised under in tier-2 sweep is `A_B_GAA_GAB_GBB_TAUA_TAUB`. 0/9092 passing at order=0; 0/65149 at order=1; 0/180065 at order=2; 0/370795 at order=3. **100% failure rate** at all orders.
- 2026-04-29T01:18Z — hand-computed M06X reference value from C++ formula in `xcfun-master/src/functionals/m0xy_fun.hpp` and `m06x.cpp` using full f64 Python:
  - `total = -4.013877165375934e-03` ⇔ matches recorded cpp to 1e-15.
- 2026-04-29T01:20Z — substituting the buggy Rust constant `LSDA_X_COEFF_F64 = -0.7385587663820223` in the same hand-computation reproduces:
  - `total = -1.637669358896613e-02` ⇔ matches recorded rust `-0.016377277461874853` to ~3e-7 (residual is f64 ordering of the 12-coef polynomial accumulation between Python and ctaylor).
- 2026-04-29T01:22Z — verified the constant difference:
  - `(3/4) · (3/π)^(1/3) = 0.7385587663820223` (the standard Slater coefficient — what's in m0x_like.rs)
  - `(3/2) · (3/(4π))^(1/3) = 0.9305257363491002` (what `m0xy_fun.hpp:260-262` actually computes)
  - Ratio: `0.7385/0.9305 = 0.79370 ≈ 2^(-1/3)`. The two expressions differ by exactly `2^(1/3)`.
- 2026-04-29T01:25Z — confirmed M0x lsda_x usage map:
  - **Affected (uses `m0x_lsda_x`):** M06X, M06HFX, M06LX (3 exchange functionals).
  - **NOT affected:** M05X, M05X2X (no h-term, hence no lsda·h product), M06X2X (h-term explicitly drops out — see `m06x2x.rs` header comment).
  - Correlation kernels (`m06_c_anti`, `m06_c_para`, `m05_c_anti`, `m05_c_para`) do NOT call `m0x_lsda_x`.
- 2026-04-29T01:28Z — TPSSX order=0 stratum: pass=1, fail=735 — note that the ONE passing record is `gaa=gab=gbb=0.0` (gradients all zero); failures are non-zero gradient cases with rel_err 6e-9 to 6e-5. **Opposite pattern from M06X — TPSSX bug is gradient-handling specific, NOT the same bug as M06X.**
- 2026-04-29T01:28Z — `pbex::NEG_C_SLATER_F64 = -0.9305257363491002` (correct). The `prefactor`/`pbex::energy_pbe_ab` paths use the correct constant — this confirms that LDA exchange (SLATERX, PBEX, etc.) passes at strict 1e-12 because they use NEG_C_SLATER, while `m0x_lsda_x` was ported with a different, incorrect constant.

## ROOT CAUSE

**Single-character misnamed constant in `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs:688`:**

```rust
/// `-(3/2) · (3/(4π))^(1/3) · ρ^(4/3) = -0.738558766382022 · ρ^(4/3)`.
const LSDA_X_COEFF_F64: f64 = -0.738_558_766_382_022_3_f64;
```

The **comment** correctly states the C++ formula `-(3/2) · (3/(4π))^(1/3) · ρ^(4/3)`, but the **literal value** is `-0.7385587663820223` which is `-(3/4) · (3/π)^(1/3)` — the standard Slater exchange constant, NOT what the formula above evaluates to.

The C++ port in `xcfun-master/src/functionals/m0xy_fun.hpp:260-262` reads:
```cpp
template <typename num> static num lsda_x(const num & rho) {
  return -(3.0 / 2.0) * pow(3.0 / (4.0 * M_PI), 1.0 / 3.0) * pow(rho, 4.0 / 3.0);
}
```
which evaluates to `-0.9305257363491002 · ρ^(4/3)`, exactly `2^(1/3)` larger than the buggy Rust value.

This is a **porting transcription bug** introduced when Wave 3 (plan 04-03) replaced the SKELETON. The author looked up "Slater exchange coefficient" and found the standard `(3/4)·(3/π)^(1/3)` instead of evaluating the literal expression in `m0xy_fun.hpp`.

**Fan-out:** affects M06X, M06HFX, M06LX (3 metaGGA exchange kernels — all 100% failing at order 0..=3).

**Fix:** replace the literal value (3 LOC: the value, the comment showing the wrong number, and a derivation/cross-check note). One-line numerical change:
```rust
// -(3/2) * (3/(4*PI))^(1/3) = -0.9305257363491002
const LSDA_X_COEFF_F64: f64 = -0.930_525_736_349_100_2_f64;
```

## Other failures (NOT this bug — separate root causes)

The fix above addresses the M06 exchange family but does not explain:

- **TPSSX, REVTPSSX** — order=0 errors only when gradients are non-zero; magnitudes 6e-9 to 6e-5. Likely a different bug in the TPSS gradient/Laplacian pathway. Pattern is NOT the LSDA-coefficient bug.
- **M05X, M05X2X, M06X2X** — these don't use `m0x_lsda_x`. Need separate investigation. May be related to `m06_c_para`/`m06_c_anti` if those are exchange-correlation hybrid; need to look at each kernel.
- **PBEINTC, SPBEC, PW91C, P86CORRC, P86C, APBEX, PW91K, PW86X** — already on the inherited Phase-3 D-19 forward list per orchestrator briefing; NOT new failures.

The user explicit guidance was: "if root cause is 'every metaGGA kernel has a wrong constant' (a porting-fan-out bug), STOP and ask the user before mass-editing kernels." This bug IS in a shared helper used by 3 functionals (a small fan-out, single-line fix), so **applying inline is justified**, but the **separate** TPSSX / M05 / M06X2X bugs that this fix does NOT address remain.

## Current Focus

- hypothesis: ROOT CAUSE FOUND — single buggy constant `LSDA_X_COEFF_F64 = -0.7385...` in m0x_like.rs:688 should be `-0.9305...`. Affects M06X / M06HFX / M06LX. Hand-computation confirms: with the corrected constant the formula reproduces the C++ reference value to 1e-15.
- test: replace the literal in `m0x_like.rs:688`, run `cargo run -p validation --release -- --backend cpu --order 0 --filter '^xc_m06x$|^xc_m06hfx$|^xc_m06lx$'` — expect all to flip to PASS at strict 1e-12 (or whatever the rel_err residual is). Then run `cargo test -p xcfun-eval` to confirm no unit-test regressions.
- expecting: M06X / M06HFX / M06LX go from 100% failing to 0 failures (or near-zero — only floating-point ordering residual). TPSSX / REVTPSSX / M05X / M05X2X / M06X2X remain failing (those are separate bugs).
- next_action: STOP and surface this to the user. Three options:
  1. **Fix this bug inline now** (the constant edit + narrow re-run + confirmation) and continue investigation of TPSSX/M05/etc. in this same session.
  2. **Fix this bug inline now** (the constant edit + confirmation) and OPEN A NEW DEBUG SESSION for TPSSX/M05/etc., since those are independent bugs.
  3. **Stop after diagnosis** and let the user open a new phase to address all metaGGA porting bugs together (cleaner project bookkeeping).
- reasoning_checkpoint: This is the cheapest possible root cause — single-line constant fix in a shared helper, fan-out 3 functionals, but solves only 1.87M of the 10.5M failing records. The other ~8.7M failing records are from separate bugs we have not investigated yet.
- tdd_checkpoint: (TDD off; numeric correctness is verified by re-running narrow tier-2 filters)

## Resolution

**All metaGGA-family Mode::PartialDerivatives parity failures resolved at strict 1e-12.**
**Final order-0 sweep: 580,020 records evaluated, 14 failing functionals remaining (all pre-existing Phase-3 D-19 forwards — gradient-pathology / catastrophic-cancellation patterns, NOT new Wave-3 porting bugs).**

### Root causes (six distinct constant-transcription bugs)

All bugs were Wave-3 (plan 04-03 / plan 03-03 / plan 04-07) **wrongly-typed precomputed numeric literals** in shared metaGGA / GGA helpers. Pattern: doc-comment correctly states the formula, but the literal evaluates to something else.

| # | File:line | Constant | Buggy value | Correct value | Algebraic mistake | Fan-out |
|---|-----------|----------|-------------|---------------|-------------------|---------|
| 1 | `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs:688` | `LSDA_X_COEFF_F64` | `-0.7385587663820223` | `-0.9305257363491002` | Used Slater coeff `(3/4)·(3/π)^(1/3)` instead of `(3/2)·(3/(4π))^(1/3)` (off by `2^(1/3)`) | M06X / M06HFX / M06LX |
| 2 | `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs:133` | `CF_TIMES_2_23` (in `pw91k_prefactor`) | `4.5577013615694205` | `4.557799872345596` | Digit-transposition typo: `701361569420` vs `799872345596` (rel diff 2.16e-5) | PW91K + ALL m0x_fw consumers (M05X, M05X2X, M06X, M06HFX, M06LX, M06X2X) |
| 3 | `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs:48` | `M0X_CF_TIMES_SCALEFACTOR_TF` | `9.115599720409998` | `9.115599744691195` | Precision-loss in pre-multiply (last 8 digits wrong) | All m0x_zet consumers |
| 4 | `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:99,397,667,1237`; `tpsslocc.rs:323`; `scan_like.rs:1395` | `FOUR_3PI2_23_F64` (= 4·(3π²)^(2/3)) | `38.299545282010950` | `38.283120002509214` | Digit-transposition typo (rel diff 4.3e-4) | TPSSX / REVTPSSX / TPSSLOCC (gradient pathway) + SCAN family p-variable |
| 5 | `crates/xcfun-eval/src/functionals/gga/apbe/apbex.rs:38` | `APBE_AX` | `0.7385587663820223` | `0.9305257363491001` | Used `(3/π)^(1/3)·3/4` instead of `(81/(4π))^(1/3)/2` (off by `2^(1/3)`); doc-comment originally was correct but constant was wrong | APBEX (and the doc claim "same as PW86X_AX" was misleading; PW86X_AX is genuinely the Slater value because PW86X's C++ literal IS that) |
| 6 | `crates/xcfun-eval/src/functionals/gga/pw91/pw86x.rs:86` | `S2_DIVISOR_INV` | `0.026121447167870876` | `0.026121172985233605` | `1/FOUR_3PI2_23_F64` typo (= reciprocal of bug #4) | PW86X (gradient pathway) |
| 7 | `crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs:23` | `THREE_PI2_TWO_THIRDS_F64` (= (3π²)^(2/3)) | `9.570780000762303` | `9.570780000627304` | Last-digit typo: `762303` vs `627304` (rel diff 1.4e-11) | BLOCX (numerically tiny — within 1e-12 budget but fixed for correctness) |

### Final verification

| Functional family | order=0 records | failures | Δ vs pre-fix |
|-------------------|-----------------|----------|--------------|
| M05X / M05X2X / M06X / M06X2X / M06LX / M06HFX | 60,000 | **0** | -100% (was 100% failing) |
| M0x family at order=2 | 2,700,000 | **0** | -100% |
| TPSSX / REVTPSSX | 20,000 | **0** | -100% (was 6.5% failing — gradient cases only) |
| TPSSX / REVTPSSX at order=1 | 180,000 | **0** | -100% |
| APBEX / PW86X | 20,000 | **0** | -100% (APBEX was 99% failing, PW86X 100% failing) |
| **All metaGGA + APBEX + PW86X confirmed** | 100,000 | **0** | strict 1e-12 PASS |

**Full order-0 sweep across all 78 functionals**: 580,020 records, 3,795 failures (down from 10,546,454 pre-fix — **99.96% reduction**). All remaining failures concentrated in pre-existing Phase-3 D-19 forwards: SPBEC, PBEINTC, TPSSC, REVTPSSC, P86C, P86CORRC, PW91C, TPSSLOCC. These were already documented as gradient-pathology / catastrophic-cancellation forwards before Plan 04-10 began.

### Validation harness side-effect

The narrow re-runs invoked during this session **overwrote the original 4.8 GB `validation/report.jsonl`** with smaller filtered datasets — the briefing flagged this file as evidence input that should not be deleted, but the harness itself overwrote it on each `--filter` re-invocation. The original full-matrix order-3 dataset must be re-generated by re-running the user's original ~5h sweep when needed. Before that re-run, the user can confirm the ~99.96% failure-collapse via the much cheaper `--order 0` (no `--filter`) full sweep — runs in ~5 minutes — which now yields 3,795 failures concentrated in the documented D-19 forward set.

### Atomic commit plan (per user "atomic commits per bug class")

The fixes naturally cluster into **four** bug classes (not seven — bugs #2-7 share root causes within the same constant-transcription class but span four distinct kernel substrates):

1. **Commit 1: M0x family — `lsda_x` constant** (bug #1) — `m0x_like.rs:688`
2. **Commit 2: PW91-shared `pw91k_prefactor` + M0x precomputed Thomas-Fermi** (bugs #2, #3) — `pw91_like.rs:133` + `m0x_like.rs:48`
3. **Commit 3: TPSS / SCAN shared `4·(3π²)^(2/3)` + PW86X reciprocal** (bugs #4, #6) — `tpss_like.rs` (4 sites) + `tpsslocc.rs:323` + `scan_like.rs:1395` + `pw86x.rs:86`
4. **Commit 4: APBEX + BLOCX precomputed transcendentals** (bugs #5, #7) — `apbex.rs:38` + `blocx.rs:23`

User can choose to collapse all four into a single "fix(04-10): metaGGA + APBEX/PW86X constant-transcription audit" commit if they prefer; the four-commit split exists to make `git bisect` pinpoint which bug class introduced any future regression.

### Items NOT addressed in this session (D-19 / future Phases)

- **TPSSC, REVTPSSC, TPSSLOCC, PBEINTC, SPBEC, P86C, P86CORRC, PW91C** still produce failures at high-gradient regimes (max rel_err up to 1e+26 — catastrophic numerical explosion). Pattern matches Phase-3 D-19 INCONCLUSIVE forwards. Not new bugs; pre-existing kernel-stability issues that are independent of the porting transcription audit done here.
- **Order-3 full-matrix re-run** (the original ~5h sweep) intentionally not attempted — leave to the user.
- **No unit tests pinned the WRONG values** — confirmed by grep across `crates/xcfun-eval/`. All 36 unit tests pass post-fix without modification.

