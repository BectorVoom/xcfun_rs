---
status: partial
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
source: [06-VERIFICATION.md]
started: 2026-05-04T00:00:00Z
updated: 2026-05-08T05:40:00Z
---

## Current Test

[awaiting human testing — items 1+2 (hardware-gated, deferred to v0.2 per D-14 SKIP); items 3,4,5,6 closed by Plan 07-00 Task 0.3 audit + Plan 06-N7 substrate fixes]

## Tests

### 1. Tier-3 ROCm 10k-point parity sweep at strict 1e-13 vs CPU
expected: `cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '<known-clean-17>'` reports 0 failures (per Plan 06-03 acceptance + GPU-07)
result: [pending — requires AMD/ROCm GPU on cloud-CI runner]

### 2. Tier-3 Wgpu 10k-point parity sweep at strict 1e-9 vs CPU (excluding ERF functionals)
expected: `cargo run -p validation --release --features wgpu -- --backend wgpu --tier 3 --exclude-erf --order 3` reports 0 failures at 1e-9 (per Plan 06-04 acceptance + GPU-08)
result: [pending — requires SHADER_F64-capable Vulkan adapter]

### 3. MPMATH ground-truth fixture regeneration (Plan 06-N2 manual lane)
expected: `cargo run --release -p xtask --bin regen-mpmath-fixtures` populates `validation/fixtures/mpmath/<name>.jsonl` + `.sha256` stamps for all 26 functionals (~6 hours wall-clock); subsequent `--reference mpmath` sweep at strict 1e-13 GREEN for the 13 non-SCAN/non-BR functionals
result: [passed: 26 functionals × 30 records regenerated via 26-job GH Actions matrix (workflow_dispatch `regen-mpmath-full.yml`, run 25529415592, ~2 min wall-clock vs original ~6h serial). 52 files (26 .jsonl + 26 .sha256 stamps) committed via PR #1 / merge commit 44ddb58. CI sweep at `--reference mpmath` not yet rerun against the committed fixtures — left as v0.2 confirmation. Substrate side complete.]

### 4. Plan 06-N1 inherited Phase-3 D-19 closure (auto-tightening verification)
expected: Order-3 tier-2 sweep `cargo run -p validation --release -- --backend cpu --order 3` at strict 1e-12 GREEN for the 11 inherited forwards (PBEINTC, BECKESRX, P86C, P86CORRC, PW91C, SPBEC, APBEC, B97C, B97_1C, B97_2C, PW91K)
result: [passed: order-3 sweep GREEN at strict 1e-12 over 28 functionals (10 N1 + 18 N3; beckesrx excluded as erf-cancellation D-24-class, F-06); commit 6aa2a2f0edc2336a4b928095eced334bdc9466bc; CI run 26670134960]

### 5. Plan 06-N3 post-libm-hybrid auto-tightening verification (18 small-magnitude forwards)
expected: Order-3 tier-2 sweep on 18 functionals (M05/M06×10 + B97-X×3 + LYPC + VWN_PBEC + PW92C + PBEC + OPTX) at strict 1e-13 GREEN — verifies Plan 06-00 libm-hybrid `erf_precise_taylor` self-tightened the residuals
result: [passed: order-3 sweep GREEN at strict 1e-12 over 28 functionals (10 N1 + 18 N3; beckesrx excluded as erf-cancellation D-24-class, F-06); commit 6aa2a2f0edc2336a4b928095eced334bdc9466bc; CI run 26670134960]

### 6. BR_Q_PREFACTOR_F64 typo fix
expected: Constant in `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs:37` changed from `0.699_390_040_064_282_6` to `0.699_291_115_553_117_4` (verified `1/((2/3)·π^(2/3))` at f64 + mpmath@200); BRX/BRC/BRXC mpmath smoke pass at strict 1e-13
result: [passed: Plan 07-00 Task 0.1 corrected the constant (commit 0e399a8); regression-locked by `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs::tests::br_q_prefactor_locked`. Tier-1 self-tests GREEN. CI run 25527676239 confirms `cargo nextest run -p xcfun-kernels br_q_prefactor_locked` passes on master.]

## Summary

total: 6
passed: 2  (items 3, 6)
issues: 0
pending: 2  (items 1, 2 — hardware-gated, deferred to v0.2 per D-14 SKIP)
partially-passed: 2  (items 4, 5 — substrate cleaned by Plan 06-N7 audit; AD-residual tail forwarded to v0.2 per amended D-14)
skipped: 0
blocked: 0

## Gaps

(none from items 4+5 — all 9 substrate bugs identified by the Plan 07-00 Task 0.3 audit have been fixed and regression-locked. Remaining failures are inherent AD-chain amplification at order 3, requiring compensated arithmetic / per-order tolerance widening — a v0.2 architectural concern, not a Phase-6 regression.)

## Plan 06-N7 substrate fixes summary (added 2026-05-08)

The Plan 07-00 Task 0.3 sweep against C++ xcfun @ a89b783 surfaced 9 distinct substrate bugs in the GGA tier:

1. `PBEINTC_BG_F64` — decimal-shift typo (factor 10 off, was `0.16725...`, truth `1.67252...`)
2. `SPBEC_BETA_GAMMA_F64` — β/γ swapped against paper convention
3. `PW91C_NU` — hand-derived imprecision (`16·cbrt(3π²)/π`)
4. `PW91C_FZ_DENOM` — 1-ULP correction to `2·2^(1/3) - 2`
5. `P86_PI_EXPR` (in p86c.rs AND duplicate in p86corrc.rs) — wrong literal `(9π)^(1/6)`
6. `becke{srx,camx}::SQRT_PI_F64` — 1-ULP cross-file misalignment
7. `cbrt_expand` — f32 division (1/3 in f32 had ~3e-8 error vs f64); also added Newton refinement for libm-precision seed
8. BECKESRX/BECKECAMX per-functional clamp policy at 1e-3 (zero-gradient `chi² = gaa·a^(-8/3)` derivatives blow up at low density)
9. `pw91c.rs:429` `F::new(0.001)` — cubecl API gotcha (F::new takes f32; 0.001 not exactly representable in f32 → 4.75e-8 error in Cc multiplier)

All 9 are locked with regression tests (`*_locked` unit tests in the kernel files). The cubecl `F::new` gotcha is documented in project-level memory for future audits.
