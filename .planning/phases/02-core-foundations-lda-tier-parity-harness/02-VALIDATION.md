---
phase: 2
slug: core-foundations-lda-tier-parity-harness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-20
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Sourced from `02-RESEARCH.md` § "Validation Architecture" (lines 1025-1126).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` + `proptest =1.11.0` + `approx =0.5.1` (already pinned in workspace) + `cc =1.2.60` (Wave-2 build-dep) |
| **Config files** | `crates/xcfun-eval/Cargo.toml` (Wave-1B) + `validation/Cargo.toml` (Wave-2) |
| **Quick run command** | `cargo test -p xcfun-eval --lib` (tier-1 self-tests, < 5s per ACC-04) |
| **Full suite command** | `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter 'lda'` (tier-2 parity, ~5-15s including C++ build amortisation) |
| **Estimated runtime** | ~5s tier-1 / ~15-30s tier-2 (cold) / ~5-8s tier-2 (warm) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p <crate-name> --lib` (≈ 1-5s per crate; the modified crate's unit tests).
- **After every plan wave:** `cargo test --workspace` (≈ 30s including all unit + integration tests across 4 crates).
- **Phase gate (Wave-2 SC #5):** `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` reports zero failures across all 11 LDA × 10k grid × 3 orders.
- **Before `/gsd-verify-work`:** Tier-2 full suite must be green, with documented per-functional overrides for D-24 LDAERF tier (1e-7 threshold).
- **Max feedback latency:** ≤ 30 seconds for the full workspace suite.

---

## Per-Task Verification Map

| Req ID | Behavior | Test Type | Automated Command | File Exists | Status |
|--------|----------|-----------|-------------------|-------------|--------|
| CORE-01 | `Vars` enum 31 variants matching xcfun.h discriminants | unit | `cargo test -p xcfun-core --lib enums::tests::var_type_cpp_ordering` | ✅ W0d (rename) | ⬜ pending |
| CORE-02 | `Mode` enum with `Unset = 0` `#[repr(u32)]` | unit | `cargo test -p xcfun-core --lib enums::tests::eval_mode_has_4_variants` | ❌ W0d | ⬜ pending |
| CORE-03 | `Dependency` bitflags 5 entries | unit | `cargo test -p xcfun-core --lib traits::tests::dependency_bits` | ✅ existing | ⬜ pending |
| CORE-04 | `XcError` 9-variant `Copy + Send + Sync` `#[non_exhaustive]` (D-25 drops `UnknownName` payload) | unit + compile-test | `cargo test -p xcfun-core --lib error::tests::*` + `static_assertions::assert_impl_all!(XcError: Copy, Send, Sync)` | ❌ W0e | ⬜ pending |
| CORE-05 | `DensVarsDev` populates 22 fields per 5 Vars arms (helper-function chain, no fallthrough) | integration | `cargo test -p xcfun-eval --test densvars_field_parity` | ❌ W1B | ⬜ pending |
| CORE-06 | `regularize` modifies only `c[CNST]` | unit | `cargo test -p xcfun-eval --lib density_vars::tests::regularize_preserves_derivatives` | ❌ W1B | ⬜ pending |
| CORE-07 | `FUNCTIONAL_DESCRIPTORS` 78 entries in `.rodata` | unit | `cargo test -p xcfun-core --test descriptors_count` | ❌ W1A (post-codegen) | ⬜ pending |
| CORE-08 | `ALIASES` slice (empty for Phase 2) | unit | `cargo test -p xcfun-core --test aliases_empty_for_phase2` | ❌ W1A | ⬜ pending |
| CORE-09 | `VARS_TABLE` 31 entries, len + provides match xcint.cpp | unit | `cargo test -p xcfun-core --test vars_table_parity` | ❌ W1A (post-codegen) | ⬜ pending |
| CORE-10 (absorbed) | `xtask regen-registry` + `--check` SHA-256 drift gate | xtask | `cargo run -p xtask --bin regen-registry -- --check` | ❌ W1A | ⬜ pending |
| LDA-01 SLATERX | `#[cube] fn slaterx_kernel`; tier-1 + tier-2 at 1e-12 | integration | `cargo test -p xcfun-eval --test self_tests slaterx` + tier-2 grid | ❌ W1B | ⬜ pending |
| LDA-02 VWN3C | `#[cube] fn vwn3c_kernel`; tier-1 + tier-2 at 1e-12 | integration | `cargo test -p xcfun-eval --test self_tests vwn3c` + tier-2 grid | ❌ W1B | ⬜ pending |
| LDA-03 VWN5C | `#[cube] fn vwn5c_kernel`; tier-1 + tier-2 at 1e-12 | integration | `cargo test -p xcfun-eval --test self_tests vwn5c` + tier-2 grid | ❌ W1B | ⬜ pending |
| LDA-04 PW92C | `#[cube] fn pw92c_kernel` matching vendored xcfun-master defaults; tier-1 + tier-2 at 1e-12 (escalate if measured rel-error > 1e-12 per Pitfall P12) | integration | `cargo test -p xcfun-eval --test self_tests pw92c` + tier-2 grid | ❌ W1B | ⬜ pending |
| LDA-05 PZ81C | `#[cube] fn pz81c_kernel`; tier-1 + tier-2 at 1e-12 | integration | `cargo test -p xcfun-eval --test self_tests pz81c` + tier-2 grid | ❌ W1B | ⬜ pending |
| LDA-06 LDAERFX | `#[cube] fn ldaerfx_kernel`; tier-1 at upstream 1e-7; **tier-2 at 1e-7 per D-24** (per-functional override, user-approved 2026-04-20) | integration | `cargo test -p xcfun-eval --test self_tests ldaerfx` + tier-2 grid (1e-7 threshold) | ❌ W1B | ⬜ pending |
| LDA-07 LDAERFC | `#[cube] fn ldaerfc_kernel`; tier-1 + tier-2 at 1e-7 per D-24 | integration | `cargo test -p xcfun-eval --test self_tests ldaerfc` + tier-2 grid (1e-7 threshold) | ❌ W1B | ⬜ pending |
| LDA-08 LDAERFC_JT | `#[cube] fn ldaerfc_jt_kernel` (no upstream test_in; tier-2 covers); tier-2 at 1e-7 per D-24 | integration | tier-2 grid only (1e-7 threshold) | ❌ W1B | ⬜ pending |
| LDA-09 TFK + TW | TFK `#[cube] fn tfk_kernel` (pure density); TW `#[cube] fn tw_kernel` (kinetic GGA via XC_A_B_GAA_GAB_GBB) | integration | `cargo test -p xcfun-eval --test self_tests tfk tw` + tier-2 grid | ❌ W1B (TFK) + W1C (TW) | ⬜ pending |
| LDA-10 VWK | `#[cube] fn vwk_kernel` (vonW formula `gaa/(8a) + gbb/(8b)`; kinetic GGA); no upstream test_in — tier-2 covers | integration | tier-2 grid only | ❌ W1C | ⬜ pending |
| MODE-04 | `input_length(vars)` matches VARS_TABLE row | unit | `cargo test -p xcfun-eval --lib functional::tests::input_length` | ❌ W1B | ⬜ pending |
| ACC-01 | Output element rel-error ≤ 1e-12 vs C++ on 8 strict-1e-12 LDAs (XC_SLATERX, XC_VWN3C, XC_VWN5C, XC_PW92C, XC_PZ81C, XC_TFK, XC_TW, XC_VWK) | full system | `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` (full report green) | ❌ W2 | ⬜ pending |
| ACC-02 | Tier-2 covers all (functional, vars, mode=PartialDerivatives, order∈{0,1,2}, point∈10k) tuples | full system | report.jsonl row count check | ❌ W2 | ⬜ pending |
| ACC-03 | `report.html` + `report.jsonl` written to `validation/` | full system | file existence + schema check | ❌ W2 | ⬜ pending |
| ACC-04 | Tier-1 self-tests run < 5s | unit | `time cargo test -p xcfun-eval --test self_tests` < 5000ms | ❌ W1B | ⬜ pending |
| ACC-05 (absorbed) | `RUSTFLAGS` empty + `-Cllvm-args=-fp-contract=off` in release; xtask check-no-fma extends to xcfun-eval | xtask | `cargo run -p xtask --bin check-no-fma` (existing, scope-extended) | ✅ Phase 1 + W1A extension | ⬜ pending |
| ACC-06 (absorbed) | `mul_add` ban in `crates/xcfun-eval/src/functionals/**/*.rs` | xtask | `cargo run -p xtask --bin check-no-mul-add` | ❌ W1A | ⬜ pending |
| QG-01 (absorbed) | `xtask check-no-anyhow` blocks anyhow in library graph | xtask | `cargo run -p xtask --bin check-no-anyhow` | ❌ W1A | ⬜ pending |
| QG-02 (absorbed) | `xtask check-boundaries` (basic — library/app boundary) | xtask | `cargo run -p xtask --bin check-boundaries` | ❌ W1A | ⬜ pending |
| QG-06 (absorbed) | `cargo metadata` cubecl `=0.10.0-pre.3` pin assertion | xtask | `cargo run -p xtask --bin check-cubecl-pin` | ❌ W1A | ⬜ pending |
| QG-07 (absorbed) | Registry content-hash drift detection (rides on CORE-10) | xtask | `cargo run -p xtask --bin regen-registry -- --check` | ❌ W1A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements (Test Infrastructure Bootstrap)

- [ ] `crates/xcfun-core/tests/` — directory exists for integration tests added by Wave-1A
- [ ] `crates/xcfun-eval/tests/` — directory created in Wave-1B for `self_tests.rs`, `densvars_field_parity.rs`
- [ ] `validation/` crate skeleton (Wave-2) — `Cargo.toml` + `build.rs` + `src/main.rs` + `src/fixtures.rs` + `src/ffi.rs`
- [ ] `static_assertions = "1.1"` added to xcfun-core dev-deps for the CORE-04 Copy compile-test
- [ ] `xtask` binaries: `check-no-mul-add`, `check-no-anyhow`, `check-boundaries`, `check-cubecl-pin`, `regen-registry`, `validate` (added Wave-1A)

---

## What the Harness CAN Catch

1. **Algorithmic divergence** between cubecl-cpu and C++ — caught at every grid point per output element. ANY 1e-12 violation in any (functional, vars, mode, order, point, element) tuple fails the merge (1e-7 for the three LDAERF functionals per D-24).
2. **MLIR FMA injection** — `xtask check-no-fma` asm-grep gate (Phase 1 inheritance, scope extended to xcfun-eval in Wave-1A).
3. **Registry drift** — `xtask regen-registry --check` SHA-256 stamp comparison.
4. **PW92C constant mismatch** — IF Rust port uses wrong constant table, 7000-point bulk grid catches at ~1e-6 to 1e-4 (well above 1e-12).
5. **Regularize off-path** — 1000-point regularize-stress stratum exercises ρ ∈ [1e-14, 1e-5] explicitly; Pitfall P10 NaN canary.
6. **Densvars fallthrough bug (Pitfall P5)** — per-variant CORE-05 unit test (compares each of 22 fields populated against C++ densvars trace).
7. **`mul_add` introduction** — `xtask check-no-mul-add` grep gate.
8. **anyhow leakage into library graph** — `xtask check-no-anyhow` grep gate.

## What the Harness CANNOT Catch

1. **Rare grid corners** the seeded grid doesn't sample. Mitigation: seed `0x1234abcd` is fixed; if a corner case is found in the wild, add it as a deterministic seed alongside the random grid.
2. **CUDA / Wgpu drift** — Phase 6's responsibility (KER-06, GPU-07, GPU-08). Phase 2 is cubecl-cpu only.
3. **Order > 2 on PartialDerivatives** — Phase 3 (MODE-01 extends to orders 3..=4).
4. **Mode::Potential / Mode::Contracted** — Phase 3/4. Phase 2 eval_setup rejects them with `XcError::InvalidMode`.
5. **Cross-platform libm divergence** (Pitfall P2, MEDIUM impact) — Linux x86_64 glibc is the canonical CI runner. macOS/Windows runs accepted at the same 1e-12 bar but documented as known-flake risk.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tier-2 LDAERF override sign-off | LDA-06, LDA-07, LDA-08 (D-24) | User must approve the per-functional 1e-7 threshold against measured Wave-2 magnitudes | After Wave-2-7 lands, review `validation/report.html` LDAERF rows; confirm max-rel-error < 1e-7; confirm divergence note rendered |
| Cross-platform seed reproducibility | Grid generator (research §"Grid Generator Spec" / A8) | rand_xoshiro algorithm correctness across Linux/macOS/Windows requires multi-platform smoke run | Run `cargo run -p validation -- --emit-grid-checksum` on each platform; SHA-256 of grid file MUST match across all three |

---

## CI Integration

- **Per-commit (every PR):** `cargo test --workspace` runs tier-1 self-tests (~5s) + xtask gates (~10s).
- **Pre-merge (manual/required):** `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` (~15-30s including C++ cc-build amortised).
- **Nightly:** Full validate run + `cargo run -p xtask --bin regen-registry -- --check` + `cargo run -p xtask --bin check-no-fma`.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] D-24 LDAERF override documented in `report.html` rendering
- [ ] D-25 `XcError::UnknownName` Copy compile-test exists
- [ ] `nyquist_compliant: true` set in frontmatter (after Wave-2-7 sign-off)

**Approval:** pending (set to `approved YYYY-MM-DD` after Wave-2-7 user sign-off + cross-platform seed-reproducibility check)
