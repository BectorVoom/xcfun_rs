---
phase: 02
plan: 05
subsystem: xcfun-eval
tags: [lda, kinetic-gga, tw, vwk, xc-a-b-gaa-gab-gbb, phase2-d-fix]
requires:
  - Plan 02-03 (DensVarsDev, build_densvars skeleton, dispatch_kernel stub)
  - Plan 02-04 (9 of 11 LDA dispatch arms + tier-1 self-tests + pre-seeded CTaylor input convention)
  - Phase 1 xcfun-ad primitives (ctaylor_{add, sub, mul, scalar_mul, reciprocal, powi_2})
provides:
  - build_xc_a_b_gaa_gab_gbb — 2nd variant arm for build_densvars (CORE-05 part 3)
  - 2 kinetic-GGA LDA kernels (tw, vwk) composing Phase 1 ctaylor primitives
  - dispatch_kernel covers all 11 Phase-2 LDAs (XC_TW id=25, XC_VWK id=59 wired)
  - supports() allowlist extended to 11 IDs — Phase-2 LDA substrate feature-complete
affects:
  - crates/xcfun-eval/src/density_vars/build.rs
  - crates/xcfun-eval/src/functionals/lda/mod.rs
  - crates/xcfun-eval/src/functionals/lda/tw.rs (new)
  - crates/xcfun-eval/src/functionals/lda/vwk.rs (new)
  - crates/xcfun-eval/src/dispatch.rs
tech-stack:
  added: []
  patterns:
    - "Pre-seeded CTaylor slot layout extended to inlen=5 (a, b, gaa, gab, gbb)"
    - "Explicit helper-function chaining replaces C-style fallthrough (Pitfall P5)"
    - "ctaylor_powi_2 fused x*x for pow(x, 2.0) parity"
key-files:
  created:
    - crates/xcfun-eval/src/functionals/lda/tw.rs
    - crates/xcfun-eval/src/functionals/lda/vwk.rs
  modified:
    - crates/xcfun-eval/src/density_vars/build.rs
    - crates/xcfun-eval/src/functionals/lda/mod.rs
    - crates/xcfun-eval/src/dispatch.rs
decisions:
  - "ctaylor_powi_2 used (not ctaylor_pow with exp=2.0) for TW `(gaa+gbb)^2` — fused x*x, algorithmically identical to C++ `pow(sum, 2.0)` via the ctaylor_multo chain"
  - "Constants 1/8 and 2 derived via F::cast_from(f64) rather than F::new(f32) — preserves 1e-12 precision (LDAERFC-style libm mismatch avoided)"
  - "Kernel signatures use #[comptime] n: u32 (lowercase), matching Plan 02-04 convention; not const N: u32 as draft plan suggested"
  - "TW + VWK deferred to Plan 02-06 tier-2 parity harness — no upstream test_in data; Plan 02-05 verification is build + check-no-mul-add + no-regression on Plan 02-04 tier-1"
metrics:
  duration: ~10 minutes
  completed: 2026-04-21
  tasks: 4
  commits: 4
---

# Phase 2 Plan 05: TW + VWK Kinetic-GGA LDAs Summary

**One-liner:** Landed the final 2 LDAs (TW + VWK) that require the `XC_A_B_GAA_GAB_GBB` builder arm, fixing Pitfall PHASE2-D. xcfun-eval now ships all 11 Phase-2 LDA dispatch arms with the `supports()` allowlist complete.

## Commits (4 atomic)

| Wave | Hash | Subject |
|------|------|---------|
| 1C-1 | `0b200d1` | `feat(02-05): Wave-1C-1 build_xc_a_b_gaa_gab_gbb arm + dispatcher extension (CORE-05 part 3 — Pitfall PHASE2-D fix)` |
| 1C-2 | `865d6b6` | `feat(02-05): Wave-1C-2 LDA-09 part 2 — tw kinetic-GGA kernel (1/8 * (gaa+gbb)^2 / n via XC_A_B_GAA_GAB_GBB builder)` |
| 1C-3 | `e114556` | `feat(02-05): Wave-1C-3 LDA-10 — vwk kernel (gaa/(8*a) + gbb/(8*b); file is vonw.cpp, FUNCTIONAL is XC_VWK)` |
| 1C-4 | `ebe2631` | `feat(02-05): Wave-1C-4 dispatch_kernel TW (id=25) + VWK (id=59) arms; supports() allowlist all 11 Phase-2 LDAs` |

## Pitfall PHASE2-D — RESOLVED

Per RESEARCH §"Critical Findings", `tw.cpp:28` and `vonw.cpp:28` declare
`XC_DENSITY | XC_GRADIENT` and name `XC_A_B_GAA_GAB_GBB` as their variant.
Without this plan's 2nd builder arm, TW and VWK kernels would read
`d.gaa = d.gbb = 0` (defensive zero-init from Plan 02-03 Wave-1B-3) and
silently return zero.

**Fix (Wave-1C-1):** `build_xc_a_b_gaa_gab_gbb<F>` populates gaa/gab/gbb
from input slots 2/3/4 (pre-seeded CTaylor layout per Plan 02-04
Wave-1B-14a), derives gnn/gss/gns, then explicitly chains to
`build_xc_a_b` for a/b/n/s (replacing C-style fallthrough per Pitfall
P5). The top-level `build_densvars` dispatcher gains an
`else if comptime!(vars == 6)` arm.

**Doc-header notes in TW + VWK kernels:** Both `tw.rs` and `vwk.rs`
include a `# Preconditions (Pitfall PHASE2-D)` section warning callers
that these kernels require the `XC_A_B_GAA_GAB_GBB` builder.

## Kernel Formulas (1:1 port)

**TW — `xcfun-master/src/functionals/tw.cpp:20-22`:**
```cpp
return 1. / 8. * pow(d.gaa + d.gbb, 2.0) / d.n;
```
Rust operation order (left-to-right, ACC-06 no mul_add):
1. `sum = gaa + gbb` (ctaylor_add)
2. `sum2 = sum * sum` via `ctaylor_powi_2` (fused x*x — algorithmically
   identical to `pow(sum, 2.0)` via the ctaylor_multo chain)
3. `inv_n = 1/n` (ctaylor_reciprocal)
4. `tmp = sum2 * inv_n` (ctaylor_mul)
5. `out = 0.125 * tmp` (ctaylor_scalar_mul; 1/8 = 0.125)

**VWK — `xcfun-master/src/functionals/vonw.cpp:17-23`:**
```cpp
template <typename num> static num vW_alpha(const num & na, const num & gaa) {
    return gaa / (8 * na);
}
template <typename num> static num vW(const densvars<num> & d) {
    return vW_alpha(d.a, d.gaa) + vW_alpha(d.b, d.gbb);
}
```
Rust `vw_alpha` order: `inv_na = 1/na`; `tmp = gaa * inv_na`;
`out = 0.125 * tmp`. `vwk_kernel` calls `vw_alpha` twice and sums.
Note: file is `vonw.cpp` but the `FUNCTIONAL(XC_VWK)` macro drives the
Rust module name `vwk`.

## Verification Results

| Gate | Command | Result |
|------|---------|--------|
| xcfun-eval compiles | `cargo build -p xcfun-eval` | PASS (3 pre-existing warnings, no new) |
| Workspace compiles | `cargo build --workspace` | PASS |
| No mul_add | `cargo run -p xtask --bin check-no-mul-add` | PASS (15 files scanned, up from 14 — vwk.rs included) |
| Tier-1 self-tests (no regression) | `cargo test -p xcfun-eval --test self_tests --features testing` | PASS (1 test, 7 LDAs covered, 12.01s) |

**Tier-1 self-test coverage unchanged from Plan 02-04:**
- 7 LDAs PASS: SLATERX, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, TFK.
- 2 LDAs intentionally skipped in tier-1 (no upstream `test_in`):
  VWN3C, LDAERFC_JT (Plan 02-04 decision).
- TW + VWK (this plan): no upstream `test_in` → tier-2 (Plan 02-06)
  covers them via synthetic-grid parity against the C++ runtime.

## Deviations from Plan

### Plan-spec nuances noted (not bug fixes)

**1. Kernel signature convention**
- **Planned:** `#[cube] pub fn tw_kernel<F: Float, const N: u32>(d, out)`
  (with `const N: u32` as generic parameter).
- **Actual:** `#[cube] pub fn tw_kernel<F: Float>(d, out, #[comptime] n: u32)`
  (with `#[comptime] n: u32` as runtime-comptime parameter).
- **Rationale:** Matches Plan 02-04 established convention used by all 9
  existing LDA kernels (SLATERX, VWN3C, ..., TFK) and the `dispatch_kernel`
  call sites. The `const N` form would require touching every call site in
  `dispatch.rs`. No functional difference — both resolve to the same cubecl
  monomorphisation.

**2. `build_xc_a_b_gaa_gab_gbb` input-reading convention**
- **Planned:** Draft plan hinted at `ctaylor_from_scalar::<F>(input[2], ...)`
  while also noting the Plan 02-04 amendment to pre-seeded coefficients.
- **Actual:** Used the pre-seeded-coefficient copy loop matching
  `build_xc_a_b` (copies `input[slot*size..(slot+1)*size]` into the
  destination field), since Plan 02-04 Wave-1B-14a committed the
  pre-seeded layout.
- **Rationale:** Stays algorithm-identical with `build_xc_a_b`'s
  Plan 02-04 amendment; the draft plan's `ctaylor_from_scalar` was a
  historical placeholder.

### Auto-fixed issues

None. All 4 tasks executed without auto-fix triggers.

## Hand-off to Plan 02-06 (Tier-2 Parity Harness)

xcfun-eval is now feature-complete for Phase-2 LDA:
- 11 of 11 dispatch arms wired (SLATERX, VWN3C, VWN5C, LDAERFX, LDAERFC,
  LDAERFC_JT, TFK, TW, PW92C, PZ81C, VWK).
- 2 builder variant arms (XC_A_B, XC_A_B_GAA_GAB_GBB).
- `supports()` allowlist = `{ 0, 2, 3, 13, 14, 15, 24, 25, 28, 55, 59 }`.

Plan 02-06 should extend `run_launch` / `launch_and_accumulate` in
`crates/xcfun-eval/src/functional.rs` to handle `inlen=5` (XC_A_B_GAA_GAB_GBB)
input packing. The current Plan 02-04 launch path rejects
`inlen != 2` with `XcError::NotConfigured`; Plan 02-06 must add an inlen=5
branch before it can exercise TW/VWK at runtime.

## Threat Flags

None. No new network/FFI/file-access surface. The `mitigate`-disposition
STRIDE entries T-02-05-01 (PHASE2-D regression) and T-02-05-02 (dispatch
arm spoofing) are addressed by:
- T-02-05-01: The `build_xc_a_b_gaa_gab_gbb` arm exists and is wired;
  kernel doc-headers note the builder requirement. Plan 02-06 tier-2
  catches any runtime regression.
- T-02-05-02: Dispatch arm `id == 25` explicitly comments `XC_TW` and
  `id == 59` explicitly comments `XC_VWK`; Plan 02-02 `registry_tables`
  test asserts the FunctionalId discriminants.

## Self-Check

All commit hashes verified present in `git log --oneline`:
- `0b200d1` (Wave-1C-1) — FOUND
- `865d6b6` (Wave-1C-2) — FOUND
- `e114556` (Wave-1C-3) — FOUND
- `ebe2631` (Wave-1C-4) — FOUND

Created files verified present:
- `crates/xcfun-eval/src/functionals/lda/tw.rs` — FOUND
- `crates/xcfun-eval/src/functionals/lda/vwk.rs` — FOUND

Build green, no-mul-add green, tier-1 self-tests green (no regression vs
Plan 02-04 baseline — 7 LDAs PASS, 1 test case).

## Self-Check: PASSED
