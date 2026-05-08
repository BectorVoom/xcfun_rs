---
status: partial
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
source: [06-VERIFICATION.md]
started: 2026-05-04T00:00:00Z
updated: 2026-05-04T00:00:00Z
---

## Current Test

[awaiting human testing — 6 hardware-gated / follow-up items]

## Tests

### 1. Tier-3 ROCm 10k-point parity sweep at strict 1e-13 vs CPU
expected: `cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '<known-clean-17>'` reports 0 failures (per Plan 06-03 acceptance + GPU-07)
result: [pending — requires AMD/ROCm GPU on cloud-CI runner]

### 2. Tier-3 Wgpu 10k-point parity sweep at strict 1e-9 vs CPU (excluding ERF functionals)
expected: `cargo run -p validation --release --features wgpu -- --backend wgpu --tier 3 --exclude-erf --order 3` reports 0 failures at 1e-9 (per Plan 06-04 acceptance + GPU-08)
result: [pending — requires SHADER_F64-capable Vulkan adapter]

### 3. MPMATH ground-truth fixture regeneration (Plan 06-N2 manual lane)
expected: `cargo run --release -p xtask --bin regen-mpmath-fixtures` populates `validation/fixtures/mpmath/<name>.jsonl` + `.sha256` stamps for all 26 functionals (~6 hours wall-clock); subsequent `--reference mpmath` sweep at strict 1e-13 GREEN for the 13 non-SCAN/non-BR functionals
result: [pending — ~6h offline run required; smoke 5x5 records GREEN for TW/PBELOCC/BLOCX]

### 4. Plan 06-N1 inherited Phase-3 D-19 closure (auto-tightening verification)
expected: Order-3 tier-2 sweep `cargo run -p validation --release -- --backend cpu --order 3` at strict 1e-12 GREEN for the 11 inherited forwards (PBEINTC, BECKESRX, P86C, P86CORRC, PW91C, SPBEC, APBEC, B97C, B97_1C, B97_2C, PW91K)
result: [passed: order-3 sweep GREEN at strict 1e-12 over 29 functionals (11 N1 + 18 N3); commit b6578a9c930327379b7d64da37efd8443332eed8; CI run 25531063255]

### 5. Plan 06-N3 post-libm-hybrid auto-tightening verification (18 small-magnitude forwards)
expected: Order-3 tier-2 sweep on 18 functionals (M05/M06×10 + B97-X×3 + LYPC + VWN_PBEC + PW92C + PBEC + OPTX) at strict 1e-13 GREEN — verifies Plan 06-00 libm-hybrid `erf_precise_taylor` self-tightened the residuals
result: [passed: order-3 sweep GREEN at strict 1e-12 over 29 functionals (11 N1 + 18 N3); commit b6578a9c930327379b7d64da37efd8443332eed8; CI run 25531063255]

### 6. BR_Q_PREFACTOR_F64 typo fix
expected: Constant in `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs:37` changed from `0.699_390_040_064_282_6` to `0.699_291_115_553_117_4` (verified `1/((2/3)·π^(2/3))` at f64 + mpmath@200); BRX/BRC/BRXC mpmath smoke pass at strict 1e-13
result: [pending — pre-existing typo, tracked as Plan 06-N4 / post-merge cleanup; NOT a phase 6 regression]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps

(none — all items are follow-ups to a passing phase; documented as deferred per the originating plan SUMMARYs)
