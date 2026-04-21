---
phase: 02
plan: 04
subsystem: xcfun-eval
tags: [lda, tier-1, ldaerfc, constants-fix, range-separated, pw92c, slater, tfk]
requires:
  - Plan 02-02 (FUNCTIONAL_DESCRIPTORS with test_in/test_out/test_threshold populated)
  - Plan 02-03 (DensVarsDev<F>, build_densvars, Functional::eval skeleton, dispatch_kernel stub)
  - Phase 1 xcfun-ad primitives (ctaylor_{add,mul,sub,scalar_mul,pow,log,exp,sqrt,reciprocal,zero})
provides:
  - 9 LDA kernels wired into dispatch_kernel (SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, LDAERFC_JT, TFK)
  - pw92eps + vwn_eps shared helper modules
  - tier-1 self-test harness (`crates/xcfun-eval/tests/self_tests.rs`)
  - 7 of 7 LDAs with upstream test_in pass at their upstream test_threshold
affects:
  - crates/xcfun-eval/src/functionals/lda/{slaterx,vwn3c,vwn5c,pw92c,pz81c,ldaerfx,ldaerfc,ldaerfc_jt,tfk}.rs
  - crates/xcfun-eval/src/functionals/lda/{pw92eps,vwn_eps}.rs
  - crates/xcfun-eval/src/density_vars/build.rs
  - crates/xcfun-eval/src/dispatch.rs
  - crates/xcfun-eval/src/functional.rs
tech-stack:
  added: []
  patterns:
    - "libm-consistent f64 constant derivation via Python"
    - "`F::cast_from(f64)` in `#[cube]` bodies instead of `F::new(f32)` for 1e-11 constant fidelity"
    - "parametric tier-1 harness driven by FUNCTIONAL_DESCRIPTORS"
    - "infer_order_from_outlen reconciles upstream macro `order=1` with observed outlen=6 for TFK"
key-files:
  created:
    - crates/xcfun-eval/tests/self_tests.rs
  modified:
    - crates/xcfun-eval/src/functionals/lda/ldaerfc.rs
    - crates/xcfun-eval/src/functionals/lda/slaterx.rs
    - crates/xcfun-eval/src/functionals/lda/pz81c.rs
    - crates/xcfun-eval/src/functionals/lda/tfk.rs
    - crates/xcfun-eval/src/functionals/lda/vwn_eps.rs
    - crates/xcfun-eval/src/functionals/lda/pw92eps.rs
    - crates/xcfun-eval/src/density_vars/build.rs
decisions:
  - "LDAERFC constants regenerated via libm-consistent Python+glibc verification; 0 rel-err vs upstream target"
  - "f32 constants replaced with f64 throughout (F::cast_from) to hit 1e-11 tier-1 threshold"
  - "infer_order_from_outlen(inlen, outlen) reconciles regen-registry literal-order extraction with observed outlen"
  - "Tier-1 harness gated behind --features testing (xcfun-core registry feature-flag convention)"
metrics:
  duration: ~2h (pickup + diagnose + fix + commit + summary)
  completed: 2026-04-21
---

# Phase 2 Plan 04: LDA Tier-1 Self-Test Parity Summary

**One-liner:** Wired 9 LDA kernels into dispatch + fixed the LDAERFC constants-correctness bug that was producing 6.3e-6 rel-error vs the 1e-7 upstream threshold. All 7 tier-1 self-tests now green.

## Commits (this plan)

| # | Hash | Subject |
|---|------|---------|
| 1 | `f85cefe` | `feat(02-04): Wave-1B-7 LDA-01/02/03 — slaterx + vwn3c + vwn5c kernels + shared vwn_eps helper module` |
| 2 | `6dbbce3` | `feat(02-04): Wave-1B-9/10 LDA-04/05 — pw92c + pz81c kernels + shared pw92eps helper module` |
| 3 | `abf3506` | `feat(02-04): Wave-1B-11/12/13 LDA-06/07/08 — ldaerfx + ldaerfc + ldaerfc_jt range-separated kernels` |
| 4 | `b0a61f5` | `feat(02-04): Wave-1B-13 LDA-09 part 1 — tfk Thomas-Fermi kinetic kernel` |
| 5 | `242cc91` | `feat(02-04): Wave-1B-14a wire dispatch_kernel arms (9 of 11 LDAs) + extend supports() + Functional::eval launch loop` |
| 6 | `ce243e5` | `refactor(02-04): Wave-1B-14c-1 polish LDA kernel constants f32->f64 for 1e-11 tier-1` |
| 7 | `2f322d4` | `fix(02-04): Wave-1B-14c-2 LDAERFC — regenerate constants via libm-consistent f64 (was 6.3e-6 rel-err vs 1e-7 target)` |
| 8 | `63f99d3` | `test(02-04): Wave-1B-14c-3 tier-1 self-tests over FUNCTIONAL_DESCRIPTORS (7 LDAs with upstream test_in)` |

## Tier-1 Test Results (all PASS)

Inputs & thresholds sourced from `FUNCTIONAL_DESCRIPTORS.test_{in,out,threshold,order,vars}` (Plan 02-02 output). All tests run `Mode::PartialDerivatives` on `Vars::A_B`.

| Functional   | Order | Threshold | Observed max rel-err | Verdict |
|--------------|-------|-----------|----------------------|---------|
| XC_SLATERX   | 2     | 1.0e-11   | < 1e-13 (passes)     | PASS    |
| XC_VWN5C     | 2     | 1.0e-11   | < 1e-12 (passes)     | PASS    |
| XC_PW92C     | 2     | 1.0e-11   | < 1e-12 (passes)     | PASS    |
| XC_PZ81C     | 2     | 1.0e-11   | < 1e-12 (passes)     | PASS    |
| XC_LDAERFX   | 2     | 1.0e-7  (D-24) | < 1e-8 (passes) | PASS    |
| XC_LDAERFC   | 2     | 1.0e-7  (D-24) | < 1e-8 (passes) | PASS    |
| XC_TFK       | 2     | 1.0e-5    | < 1e-7 (passes)      | PASS    |

Total test-binary runtime (debug, cold cubecl-cpu JIT): ~12 s. Per-LDA eval ≈ 1.5 s dominated by JIT; the numerical inner loop is sub-millisecond.

LDAs **intentionally skipped** in tier-1 (no upstream `test_in` per the macro source):

| Functional      | Why skipped                                           | Next gate     |
|-----------------|-------------------------------------------------------|---------------|
| XC_VWN3C        | `ENERGY_FUNCTION(...)` without a test block upstream  | Plan 02-06 tier-2 (synthetic grid vs C++ runtime eval) |
| XC_LDAERFC_JT   | `ldaerfc_jt.cpp:64` has no test_in                    | Plan 02-06 tier-2 |

## Deviations from Plan

### [Rule 1 — Bug] LDAERFC 6.3e-6 rel-err vs 1e-7 D-24 threshold

**Found during:** Wave-1B-14b tier-1 harness dry-run (after Wave-1B-14a dispatch wiring).

**Symptom:** All 6 elements of the LDAERFC `XC_A_B` order-2 output were off by 3e-7 to 6e-6 relative. Element [0] (energy) showed 6.31e-6 vs. the 1e-7 target — 60× over budget.

**Root cause:** Six of the precomputed f64 literals in
`crates/xcfun-eval/src/functionals/lda/ldaerfc.rs` were derived from an
incorrect symbolic expansion and drifted from what libm produces at
IEEE-754 f64 precision:

| Constant | Prior value | libm-consistent f64 | Rel-drift |
|----------|-------------|---------------------|-----------|
| `QRPA_ACOUL` (`2(log2-1)/π²`) | -0.06218141773725462 | -0.0621813817393098    | 5.8e-8 |
| `QRPA_B2` (via Acoul)         | 7.451754872610209    | 7.4495253826340555     | **3.0e-4** |
| `DPOL_CF` / `ECORRLR_CF`      | 1.919158889369867    | 1.9191582926775128     | 2.1e-7 |
| `DPOL_CF_SQ` / `ECORRLR_CF_SQ`| 3.683170919291158    | 3.683168552352866      | 6.4e-7 |
| `DPOL_LEAD_SCALE` (`2^{5/3}/5 · cf²`) | 2.3392794351087596 | 2.3386662538324523 | **2.6e-4** |
| `ECORRLR_ALPHA` (`pow(4/9π,1/3)`) | 0.5210486459438814 | 0.521061761197848 | 2.5e-5 |
| `coe5` prefactor `-9/(40·√{2π})` | -0.08976231703841775 | -0.08976201309032236 | 3.4e-6 |

`QRPA_B2` alone (3e-4 off) was the dominant contributor: it flows
through `Qrpa(x)` → `log(num/den)` → `· Acoul` → into the `ecorrlr`
numerator and then out through `n · (eps − ecorrlr)`. That single
constant accounts for the majority of the 6.3e-6 final rel-err.

**Fix (commit `2f322d4`):** Regenerated every literal using Python+libm
(verified bit-for-bit against glibc `pow`/`cbrt`/`log`/`sqrt` via
`ctypes.CDLL('libm.so.6')`) and annotated each literal with its
derivation comment. After fix, Python reference evaluation (identical
operation order to the Rust kernel) matches the upstream target
`-1.4579390272267870e-01` to **exactly 0 relative error**, confirming
the discrepancy is strictly in the constants layer — no algorithmic
drift in `Qrpa/dpol/g0f/ecorrlr` or any Phase 1 `ctaylor_*` primitive.

**CONTEXT D-19 (algorithmic-identity port) invariant holds**: every
`ctaylor_{add,mul,sub,scalar_mul,pow,log,exp,sqrt,reciprocal}` call
still mirrors the C++ source line-for-line. The bug was purely in
the value of floating-point literals.

**Verification:** Tier-1 test `tier1_self_tests_pass` passes after fix;
`check-no-mul-add` and `check-no-fma` both PASS.

### [Rule 1 — Bug] LDA kernel constants promoted f32 → f64

**Found during:** Tier-1 harness design. f32 constants `F::new(0.062_181_4_f32)`
introduce ~1.3e-7 per-constant rounding, which stacks across `eopt` /
`omega` / `pw92eps` compositions well above the 1e-11 tier-1 threshold
for SLATERX/VWN5C/PW92C/PZ81C.

**Fix (commit `ce243e5`):** All precomputed LDA-kernel constants now
stored as `const FOO: f64` and cast at kernel-time via `F::cast_from(FOO)`
(not `F::new`). This matches the `F::cast_from` pattern used elsewhere
in xcfun-eval and keeps the numerical precision at 1 ULP of f64 through
the whole chain. Also the `build_densvars` `n_m13`/`a_43`/`b_43`/`r_s`
arm was updated from `F::new(1.0f32/3.0f32)` → `F::cast_from(1.0f64/3.0f64)`
for the same reason.

Without this, SLATERX/VWN5C/PW92C/PZ81C would still fail tier-1 at
1e-11. With it, all four pass comfortably.

## Known Stubs / Gaps → Forward to Plan 02-05

Two LDAs remain as stubs in `dispatch_kernel`:

| Id | Name  | Upstream source | Plan 02-05 task |
|----|-------|-----------------|------------------|
| 29 | XC_TW | `xcfun-master/src/functionals/tw.cpp` | LDA-09 part 2 |
| 41 | XC_VWK| `xcfun-master/src/functionals/vwk.cpp`| LDA-10        |

Neither has upstream `test_in` data (confirmed by inspection of the
upstream FUNCTIONAL macros), so tier-1 coverage for TW/VWK will be
supplied via Plan 02-06 tier-2 (synthetic-grid parity against C++
runtime evaluation). That is a known, planned gap — **not** a Plan
02-04 defect.

Also deferred (logged for a future task, not for Plan 02-05):
- `regen_registry` macro extractor for `order`/`outlen` reconciliation
  (the self-test harness works around this with `infer_order_from_outlen`).
- `xcfun-core::constants::CF` pre-existing drift (~5e-5) noted in
  `tfk.rs`; does not affect Plan 02-04 because TFK kernel uses the
  correct libm-derived constant `2.871234000188192` directly.

## Threat Flags

None. The plan does not introduce new auth/network/filesystem surface.

## Self-Check

All commit hashes verified present in `git log --oneline`:
- `f85cefe`, `6dbbce3`, `abf3506`, `b0a61f5`, `242cc91` (pre-existing)
- `ce243e5`, `2f322d4`, `63f99d3` (new — this session)

Created file verified present:
- `crates/xcfun-eval/tests/self_tests.rs` — FOUND

Tier-1 test verified PASS via `cargo test -p xcfun-eval --test self_tests --features testing`.

## Self-Check: PASSED
