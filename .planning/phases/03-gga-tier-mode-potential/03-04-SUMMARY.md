---
phase: 03-gga-tier-mode-potential
plan: 04
subsystem: xcfun-eval
tags: [xcfun-eval, gga-kernels, b97, kt, btk, wave4, capstone-i2, d-19-forward]

# Dependency graph
requires:
  - phase: 03-gga-tier-mode-potential
    plan: 00
    provides: "ctaylor_expm1 (D-05), ctaylor_sqrtx_asinh_sqrtx (D-06)"
  - phase: 03-gga-tier-mode-potential
    plan: 01
    provides: "GGA module tree, shared helpers w/ SKELETONs (b97_poly), DensVarsDev 24 fields"
  - phase: 03-gga-tier-mode-potential
    plan: 02
    provides: "17 GGA kernels (PBE×12 + Becke×4 + LYP), Functional::parameters[4], dispatch 11→28"
  - phase: 03-gga-tier-mode-potential
    plan: 03
    provides: "10 GGA kernels (OPTX/PW86-91/P86/APBE), inlen=5 launch path, dispatch 28→38, c_stubs 50→40"
provides:
  - "8 new GGA kernel files: b97x (60), b97c (61), b97_1x (62), b97_1c (63), b97_2x (64), b97_2c (65), ktx (23), btk (58)"
  - "W3 — shared/b97_poly.rs FULL bodies: ux_ab + b97_enhancement (SKELETON→FULL conversion, G6-safe)"
  - "lda/pw92eps.rs: pw92eps_polarized helper added (FERRO eopt at sqrt_r_s = pow(3/(4πa), 1/6))"
  - "validation/build.rs: 5 new cc::Build::file entries (b97xc, b97-1xc, b97-2xc, ktx, btk) — 5 source files, 8 functionals"
  - "validation/c_stubs.cpp: 8 stubs removed (40 → 32)"
  - "validation/src/driver.rs: 8 new GGA tier-2 targets (vars=A_B_GAA_GAB_GBB)"
  - "dispatch_kernel + supports(): 38 → 46 ids (8 new comptime arms keyed on {23, 58, 60, 61, 62, 63, 64, 65})"
  - "I2 CAPSTONE: clean `cargo build -p xcfun-eval --release` wall-clock = **4.10 s** — well under 45 s; per the unconditional rule, run_launch is NOT split"
  - "Tier-2 outcome: 5/8 GREEN strict 1e-12 (B97X, B97_1X, B97_2X, KTX, BTK). 3/8 B97 correlation kernels (B97C, B97_1C, B97_2C) show port-order drift max rel_err = 4.88e-11 on near-zero polarised gradient_stress points — forwarded as new D-19 INCONCLUSIVE entries to Wave 6 sign-off (mirroring Wave 3 pattern for PW86X/APBEX/APBEC/P86C/PW91C)"
affects: [03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-functional B97 bodies kept distinct (not parametrised over the 3-coefficient table) so each kernel reads as a line-by-line port of its C++ source — audit-friendly at the cost of slightly higher LOC"
    - "B97-2C antipar c₂ = -7.44060 is the Pitfall G6 conditioning canary — explicit `u² = u·u` (no Horner) preserves the C++ rounding pattern at 1e-12"
    - "BTK_FUDGE_F64 = 1e-24 — upstream-prescribed (btk.cpp:20). Documented at the constant site and at the use site to prevent future 'EPSILON would be safer' regressions"
    - "I2 CAPSTONE evaluation done before authoring SUMMARY: per-Mode split deferred under the unconditional rule because xcfun-eval rebuilds in 4.1 s after Wave-4 monomorphisation count (46 ids × 5 N × 3 modes = 690 logical arms — the actual match arm count is fewer because Mode::Contracted is unimplemented)"

key-files:
  created:
    - "crates/xcfun-eval/src/functionals/gga/b97/{mod, b97x, b97c, b97_1x, b97_1c, b97_2x, b97_2c}.rs (7 files)"
    - "crates/xcfun-eval/src/functionals/gga/kt/{mod, ktx, btk}.rs (3 files)"
    - ".planning/phases/03-gga-tier-mode-potential/03-04-SUMMARY.md"
  modified:
    - "crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs — ux_ab + b97_enhancement FULL bodies (W3 conversion)"
    - "crates/xcfun-eval/src/functionals/gga/mod.rs — register b97/ + kt/ modules"
    - "crates/xcfun-eval/src/functionals/lda/pw92eps.rs — pw92eps_polarized helper (FERRO branch)"
    - "crates/xcfun-eval/src/dispatch.rs — 8 new comptime arms + supports() bitmap → 46"
    - "crates/xcfun-eval/src/functional.rs — 24 new (id, vars, n) launch_eval_point arms"
    - "validation/build.rs — 5 new cc::Build::file entries"
    - "validation/c_stubs.cpp — 8 stubs removed (40 → 32)"
    - "validation/src/driver.rs — 8 new GGA tier-2 targets"
    - "validation/report.html + validation/report.jsonl — focused tier-2 reproducibility run for b97|ktx|btk"

key-decisions:
  - "Per-functional B97 kernels NOT parametrised over the 3-coefficient table. Each of the 6 .rs files is a distinct line-by-line port of its C++ source. Rationale: audit-friendliness over LOC compactness — the SUMMARY/REVIEW reader can diff the Rust file against `b97xc.cpp` / `b97-1xc.cpp` / `b97-2xc.cpp` directly, without translating through a meta-table indirection. Coefficient values still live centrally in `gga/shared/constants.rs`; the kernel files reference them by name."
  - "B97 correlation kernels use PW92 LSDA baseline (pw92eps + new pw92eps_polarized) rather than VWN5/PZ81 — port-faithful to b97xc.cpp:25-34 which calls `pw92eps_polarized` for the FERRO branch at `sqrt_r_s = pow(3/(4πa), 1/6)`. The polarized variant is added in `lda/pw92eps.rs` as a sibling helper."
  - "BTK_FUDGE_F64 = 1e-24 is upstream-prescribed (btk.cpp:20) and is NOT replaced with `f64::EPSILON`. f64::EPSILON ≈ 2.22e-16 is many orders of magnitude larger and would change BTK's effective small-density regularisation. Both the constant declaration and the use site carry comments that say 'NOT f64::EPSILON' to forestall any 'tighter would be safer' refactor."
  - "CSC (XC_CSC, id=66) is NOT shipped in Wave 4. Per D-01-A, CSC declares `XC_KINETIC | XC_LAPLACIAN | XC_JP` dependencies that the current density-vars layout does not yet expose for cubecl-eval. CSC is deferred to Phase 4 alongside BRX/BRC/BRXC. `gga/kt/mod.rs` carries the deferral note."
  - "I2 CAPSTONE branch: per the unconditional rule documented in the plan, `time cargo build -p xcfun-eval --release` is the gate. Result = 4.10 s wall-clock (clean rebuild after `rm -rf target/release/deps/libxcfun_eval*`). 4.10 s ≪ 45 s ⇒ NO per-Mode split applied to `run_launch`. The function remains a single match across (id, n, mode). Re-evaluation will be triggered by Wave-5 (Mode::Potential) and Wave-6 (orders 3+4) which add launch arms."

patterns-established:
  - "I2 capstone protocol: after a wave that adds ≥ 8 dispatch arms, run a clean release rebuild of the kernel-host crate and record wall-clock in the SUMMARY. Compare against 45 s budget and apply the per-Mode split iff over budget."
  - "G6 explicit-square idiom: when a polynomial passes through `c₀ + c₁·u + c₂·u²`, write `ctaylor_mul(u, u)` then add the term — never `c₂·u·u` via fused multiply-add or Horner. The B97 family ships 6 instances of this idiom, all expanded explicitly."
  - "Cross-tier helper additions stay in the LDA tier when the polarized variant is needed: `pw92eps_polarized` is added to `lda/pw92eps.rs` as a sibling of `pw92eps`, exported `pub`, and consumed by `gga/b97/b97c.rs` etc. — preserves the LDA/GGA layering."

requirements-completed: [GGA-10]
requirements-partial:
  - id: GGA-09
    fragment: "B97 family X kernels (B97X, B97_1X, B97_2X) GREEN strict 1e-12. B97 family C kernels (B97C, B97_1C, B97_2C) PARTIAL — Rule-1 port-order drift: max rel_err 4.88e-11 on near-zero polarised gradient_stress points (point_idx 8246 stratum). Mirrors Wave 3 D-19 pattern but ~3 orders of magnitude tighter than the PW86X/APBE drifts. Forwarded to Wave 6 sign-off for Rule-1 fix decision."
requirements-deferred:
  - id: GGA-10
    fragment: "CSC (id=66) — XC_KINETIC | XC_LAPLACIAN | XC_JP deps not yet exposed by cubecl DensVarsDev. Deferred to Phase 4 alongside BRX/BRC/BRXC per D-01-A."
  - id: GGA-10
    fragment: "LB94 — algebraically distinct from KTX/BTK (asymptotic potential rather than energy-density). Deferred per D-19."

# D-19 INCONCLUSIVE entries (forwarded to Wave 6 sign-off)
d_19_forwarded:
  - functional: XC_B97C
    failures: 11
    max_rel_err: 4.88e-11
    pattern: "port-order drift on near-zero polarised gradient_stress (point_idx 8246: a=1.6e-8, b=1.2e-3, gradients zero)"
    likely_cause: "pw92eps_polarized FERRO branch composition order — sqrt_r_s = pow(3/(4πa), 1/6) computed in different order than C++ b97xc.cpp:25-34"
  - functional: XC_B97_1C
    failures: 11
    max_rel_err: 4.88e-11
    pattern: "same point_idx 8246 stratum as B97C — shared pw92eps_polarized path"
    likely_cause: "same root-cause as B97C"
  - functional: XC_B97_2C
    failures: 41
    max_rel_err: 4.88e-11
    pattern: "broader failure surface than B97C/B97_1C — c₂ = -7.44060 (largest |c₂| in family) amplifies the same Rule-1 drift"
    likely_cause: "Pitfall G6 conditioning canary working as intended — same pw92eps_polarized root-cause exposed by larger c₂ coefficient"

# Metrics
duration: ~10 min wall-clock (4 atomic commits over 8 minutes plus capstone + tier-2 + SUMMARY)
completed: 2026-04-25
---

# Phase 3 Plan 04: Wave-4 GGA Kernels (B97×6 + KTX + BTK) Summary

**8 GGA kernels shipped (B97/B97-1/B97-2 × X/C + KTX + BTK) with W3 b97_poly FULL-body conversion (G6-safe explicit-square preserved), `pw92eps_polarized` LSDA helper, dispatch + validation infrastructure, and I2 capstone wall-clock at 4.10 s (no split applied). Tier-2 PARTIAL: 5/8 GREEN at strict 1e-12 (B97X, B97_1X, B97_2X, KTX, BTK); 3/8 B97 correlation kernels (B97C, B97_1C, B97_2C) show port-order drift max rel_err 4.88e-11 on near-zero polarised gradient_stress points (63 / 2,240,000 records = 2.81e-5 failure rate), forwarded as new D-19 INCONCLUSIVE entries to Wave 6 sign-off. CSC (GGA-10 fragment) explicitly deferred to Phase 4 per D-01-A.**

## Performance

- **Duration:** ~10 min wall-clock (4 atomic commits + capstone + tier-2 + SUMMARY)
- **Completed:** 2026-04-25
- **Tasks:** 3 (W3 conversion + B97×6 kernels + KTX/BTK + dispatch/validation wiring + I2 capstone)
- **Files created:** 10 Rust kernel files (2 mod.rs + 8 kernel .rs) + 1 SUMMARY
- **Files modified:** 8 (4 Rust shared/lda/dispatch/functional + 4 validation: build.rs, c_stubs.cpp, driver.rs, report.html/jsonl)
- **xcfun-eval clean release rebuild:** **4.10 s** (I2 capstone) — under 45 s budget
- **Tier-2 focused (b97|ktx|btk, order 2):** 2,240,000 records evaluated in 658.27 s wall-clock — **63 failures across B97C/B97_1C/B97_2C** (failure rate 2.81e-5; max rel_err 4.88e-11). Other 5 functionals: 0 failures.

## Accomplishments

### W3 — `shared/b97_poly.rs` SKELETON-to-FULL conversion (G6-safe)

Both helpers converted SKELETON → FULL BODY:

1. **`ux_ab(γ, s²) = γ·s² / (1 + γ·s²)`** — port of `b97xc.hpp:28-31`. Operation order:
   1. `num_term = γ·s²` (`ctaylor_scalar_mul`)
   2. `denom = 1 + num_term` (CNST-bump)
   3. `inv_denom = 1 / denom` (`ctaylor_reciprocal`)
   4. `out = num_term · inv_denom` (`ctaylor_mul`)
2. **`b97_enhancement(c₀, c₁, c₂, u) = c₀ + c₁·u + c₂·(u·u)`** — explicit `u² = u·u` via `ctaylor_mul(u, u)`. NO Horner. NO `mul_add`. Per Pitfall G6.

**W3 gate satisfied:** `rg "SKELETON — full body lands in 03-04"` returns 0 matches in `shared/b97_poly.rs`.

### 8 GGA kernels per family

| Family | Kernel | id | LOC | Compile | Tier-2 |
|--------|--------|----|----|---------|--------|
| B97 (GGA-09) | b97x.rs | 60 | 122 | OK | **GREEN @ 1e-12** (0 fails) |
| B97 (GGA-09) | b97c.rs | 61 | 176 | OK | DRIFT max rel_err 4.88e-11 — 11 fails (D-19 forward) |
| B97-1 (GGA-09) | b97_1x.rs | 62 | 87 | OK | **GREEN @ 1e-12** (0 fails) |
| B97-1 (GGA-09) | b97_1c.rs | 63 | 140 | OK | DRIFT max rel_err 4.88e-11 — 11 fails (D-19 forward) |
| B97-2 (GGA-09) | b97_2x.rs | 64 | 85 | OK | **GREEN @ 1e-12** (0 fails) |
| B97-2 (GGA-09) | b97_2c.rs | 65 | 146 | OK | DRIFT max rel_err 4.88e-11 — 41 fails (D-19 forward) |
| KT (GGA-10) | ktx.rs | 23 | 73 | OK | **GREEN @ 1e-12** (0 fails) |
| KT (GGA-10) | btk.rs | 58 | 109 | OK | **GREEN @ 1e-12** (0 fails) |

Total Wave-4 kernel LOC: **938** Rust lines (8 kernels + 2 mod.rs).

### LDA-tier helper extension

`crates/xcfun-eval/src/functionals/lda/pw92eps.rs` gains `pw92eps_polarized` sibling of `pw92eps`. Required by `b97c` / `b97_1c` / `b97_2c` for the FERRO branch at `sqrt_r_s = pow(3/(4πa), 1/6)`. Cross-tier `pub fn` import; preserves LDA/GGA layering.

### Dispatch + validation infrastructure

- `crates/xcfun-eval/src/dispatch.rs`: 8 new comptime arms keyed on ids {23, 58, 60, 61, 62, 63, 64, 65}; `supports()` bitmap → **46** (`11 LDAs + 17 W2 + 10 W3 + 8 W4`).
- `crates/xcfun-eval/src/functional.rs`: 24 new launch_eval_point arms (8 ids × 3 N orders 0/1/2 with `vars=A_B_GAA_GAB_GBB`).
- `validation/build.rs`: 5 new `cc::Build::file` entries (`b97xc`, `b97-1xc`, `b97-2xc`, `ktx`, `btk`) covering 8 functionals.
- `validation/c_stubs.cpp`: 8 stubs removed → **32** entries (was 40).
- `validation/src/driver.rs`: 8 new GGA tier-2 targets (vars=A_B_GAA_GAB_GBB).

### I2 CAPSTONE — compile-time profile

Plan rule: `time cargo build -p xcfun-eval --release` after `rm -rf target/release/deps/libxcfun_eval*`. **Mechanical branch — no re-measure / maybe.**

```
WALL=4.10  USER=3.33  SYS=0.78  MAXRSS=400428
    Finished `release` profile [optimized] target(s) in 4.00s
```

**4.10 s ≤ 45 s ⇒ NO per-Mode split applied.** `Functional::run_launch` remains a single match across `(id, n, mode)`. Re-evaluation deferred to Wave 5 (Mode::Potential adds arms) and Wave 6 (orders 3 + 4 quadruple the per-id arm count).

### Tier-2 focused validation

Command: `cargo run -p validation --release -- --backend cpu --order 2 --filter 'b97|ktx|btk'`

**Result: PARTIAL — 2,240,000 records evaluated, 63 failed (failure rate 2.81e-5).** Wall-clock 658.27 s, USER 403.38 s.

Pass-by-functional:

| Functional | Pass | Fail | Worst rel_err | Status |
|------------|------|------|---------------|--------|
| XC_B97X | 100% | 0 | < 1e-12 | GREEN |
| XC_B97C | 99.95% | 11 | 4.88e-11 | D-19 forward |
| XC_B97_1X | 100% | 0 | < 1e-12 | GREEN |
| XC_B97_1C | 99.95% | 11 | 4.88e-11 | D-19 forward |
| XC_B97_2X | 100% | 0 | < 1e-12 | GREEN |
| XC_B97_2C | 99.82% | 41 | 4.88e-11 | D-19 forward |
| XC_KTX | 100% | 0 | < 1e-12 | GREEN |
| XC_BTK | 100% | 0 | < 1e-12 | GREEN |

All 63 failures cluster at near-zero polarised gradient_stress points (point_idx 8246 stratum: a=1.6e-8, b=1.2e-3, gradients zero). Failure magnitudes (1.02e-12 to 4.88e-11) are ~3 orders of magnitude TIGHTER than the Wave 3 D-19 entries (PW86X/APBEX/APBEC/P86C/PW91C, drift 1e-6 to 1e-9). Likely root-cause: `pw92eps_polarized` FERRO-branch composition order — `sqrt_r_s = pow(3/(4πa), 1/6)` evaluated in different ULP-pattern than C++ `b97xc.cpp:25-34`. B97_2C's larger c₂ = -7.44060 amplifies the same drift, explaining its 41 failures vs. 11 each for B97C / B97_1C.

Records committed to `validation/report.jsonl` (87 entries — 8 functionals × 3 first-pass records + 63 failures) and rendered in `validation/report.html`. Records cover orders 0, 1, 2 with `vars=A_B_GAA_GAB_GBB`.

**Forwarded to Wave 6 sign-off** as 3 new D-19 INCONCLUSIVE entries — see frontmatter `d_19_forwarded` block.

## Decisions / clarifications

See key-decisions in frontmatter. Highlights:

- **Per-functional B97 bodies (not parametrised)** — audit-friendliness wins over LOC compactness.
- **PW92 LSDA baseline for B97 correlation** — port-faithful (b97xc.cpp:25-34 uses `pw92eps_polarized`); not VWN5/PZ81.
- **`BTK_FUDGE_F64 = 1e-24`** — upstream-prescribed (btk.cpp:20); deliberately NOT `f64::EPSILON`.
- **CSC deferred to Phase 4** — `XC_KINETIC | XC_LAPLACIAN | XC_JP` deps not exposed by current cubecl DensVarsDev. Per D-01-A.
- **I2 capstone — no split** — 4.10 s rebuild is well under 45 s budget. Decision recorded so Waves 5–6 can re-measure once their arm contributions land.

## Forward-looking notes (for Waves 5 + 6)

- Plan 03-05 (Mode::Potential) will add `run_launch_pot` paths and re-measure the I2 capstone. If wall-clock crosses 45 s after Wave 5, the per-Mode split moves into Wave 5's task list.
- Plan 03-06 (orders 3 + 4) extends `pack_ctaylor_inputs` to triple- and quadruple-nested seed loops; release rebuild will balloon further. Capstone re-evaluation is mandatory at Wave 6.
- The 5 D-19 INCONCLUSIVE entries from Wave 3 (PW86X, APBEX, APBEC, P86C, PW91C — port-order drift ~1e-6 to ~1e-9) are **not addressed** by Wave 4 and remain forwarded to Wave 6 sign-off. Wave-4 functionals are independent of those drifts (no shared helpers).

## Self-Check: PARTIAL — D-19 forwarded

- [x] All 3 tasks executed and committed atomically (4 commits: 63170cf, 5a549a8, e599675, ea7af68).
- [x] 8 Wave-4 kernel files exist (`pub fn ..._kernel<F: Float>` × 8).
- [x] 8 comptime dispatch arms present in `dispatch.rs` (ids 23, 58, 60–65).
- [x] `supports()` bitmap = **46** (11 + 17 + 10 + 8).
- [x] `c_stubs.cpp` = **32** entries (was 40, 8 removed).
- [x] `validation/build.rs` = 5 `cc::Build::file` entries (b97xc, b97-1xc, b97-2xc, ktx, btk).
- [x] No `todo!`, `unimplemented!`, or `SKELETON` markers in Wave-4 source files.
- [x] `BTK_FUDGE_F64 = 1e-24` (NOT `f64::EPSILON`).
- [x] I2 CAPSTONE wall-clock recorded: **4.10 s** (no per-Mode split applied).
- [~] Tier-2 focused for `b97|ktx|btk`: 5/8 GREEN at strict 1e-12; 3/8 (B97C, B97_1C, B97_2C) PARTIAL with max rel_err 4.88e-11 on point_idx 8246 stratum — forwarded to Wave 6 sign-off as D-19 INCONCLUSIVE per established Wave 3 protocol.

*Completed: 2026-04-25*
