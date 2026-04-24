---
phase: 03
slug: gga-tier-mode-potential
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-24
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `03-RESEARCH.md §"Validation Architecture"` (lines 940+).
> **Authoritative truth:** `03-RESEARCH.md` — this file summarises the testable contract for sampling.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo-nextest` (CI) + `approx 0.5.1` (assertions) + custom tier-2 `validation/` binary (cc-linked C++ reference) |
| **Config file** | Cargo workspace (`Cargo.toml`); no pytest/jest-style config equivalent |
| **Quick run command** | `cargo test -p xcfun-eval --features testing --test self_tests` (tier-1, < 5 s gate) |
| **Mid run command** | `cargo nextest run -p xcfun-eval --features testing` (all xcfun-eval tests, parallel) |
| **Full suite command** | `cargo nextest run --workspace --all-features && cargo xtask validate --backend cpu --order 4 --filter gga` (tier-1 + tier-2 full matrix at orders 0..=4) |
| **Phase gate command** | `cargo nextest run --workspace --all-features && cargo xtask validate --backend cpu --order 4` (all 47 functionals tier-2 GREEN at 1e-12 strict; D-24 LDAERF 1e-7 override inherited — not extended to GGA) |
| **Estimated runtime** | tier-1: < 30 s · tier-2 per-family: 10–30 s · full phase gate: ~5 min (accepted per Phase-2 D-24 budget) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p xcfun-eval --features testing --test self_tests` (+ filter by family once tier-1 passes)
- **After every plan wave:** Run `cargo xtask validate --backend cpu --order 2 --filter <family>`
- **Before `/gsd-verify-work`:** Phase gate command must be green
- **Max feedback latency:** < 30 s for tier-1; < 30 s per-family tier-2; < 5 min phase gate
- **Tier-1 order coverage per family:** 0, 1, 2 until family tier-2 passes; bump to 3, 4 only in Wave 5
- **Tier-2 order coverage per family:** 0, 1 at each commit (quick); 2 at wave-end; 3, 4 in Wave 5 only

---

## Per-Task Verification Map

*Note: Task IDs `{N}-{P}-{T}` placeholder until planner materialises plans; Wave 0 prefixes shown. "File Exists" = ❌ W0 means created in a Wave 0 task.*

### Wave 0 — xcfun-ad substrate + DensVarsDev arms + GGA shared helpers

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-00-01 | 03-00 | 0 | D-05 fixture-gate | — | Clamp protects `expm1` x→0 branch | unit (golden) | `cargo test -p xcfun-ad golden_expand::test_expm1` | ❌ W0 | ⬜ pending |
| 03-00-02 | 03-00 | 0 | D-06 fixture-gate | — | `sqrtx_asinh_sqrtx` x→0 stable-bracket | unit (golden) | `cargo test -p xcfun-ad golden_composed::test_sqrtx_asinh_sqrtx` | ❌ W0 | ⬜ pending |
| 03-00-03 | 03-00 | 0 | CONTEXT D-10 (enum corrected to 27..30) | — | 7 new DensVarsDev Vars arms reject invalid inlen | unit | `cargo test -p xcfun-eval densvars_gga_arms` | ❌ W0 | ⬜ pending |
| 03-00-04 | 03-00 | 0 | D-08 GGA shared helpers scaffolded | — | No live kernels yet — compile gate only | compile-check | `cargo build -p xcfun-eval --features testing` | ❌ W0 | ⬜ pending |
| 03-00-05 | 03-00 | 0 | `regularize_2nd_taylor.rs` unit tests | — | `_2ND_TAYLOR` slot population correct (lapa = 0.5·(d[4]+d[7]+d[9])) | unit | `cargo test -p xcfun-eval regularize_2nd_taylor` | ❌ W0 | ⬜ pending |

### Waves 1–3 — GGA family ports (40 or 36 with BR/CSC deferral)

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command |
|---------|------|------|-------------|-----------|-------------------|
| 03-W1-* | 03-01..03-03 | 1 | GGA-01 (PBE ×12), GGA-02 (Becke ×4), GGA-04 (LYP ×1), GGA-03 (BR ×3, **deferral recommended — see Scope-Gate below**) | unit + tier-2 | `cargo xtask validate --backend cpu --order 2 --filter pbe`, `... --filter becke`, `... --filter lyp` |
| 03-W2-* | 03-04..03-06 | 2 | GGA-05 (OPTX ×2), GGA-06 (PW86/PW91 ×4), GGA-07 (P86 ×2), GGA-08 (APBE ×2) | unit + tier-2 | `cargo xtask validate --backend cpu --order 2 --filter optx`, `... --filter pw91x`, `... --filter p86`, `... --filter apbe` |
| 03-W3-* | 03-07..03-08 | 3 | GGA-09 (B97 ×6), GGA-10 (KT/BTK ×2, **CSC deferral recommended**) | unit + tier-2 | `cargo xtask validate --backend cpu --order 2 --filter b97`, `... --filter ktx` |

### Wave 4 — Mode::Potential

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command |
|---------|------|------|-------------|-----------|-------------------|
| 03-W4-01 | 03-09 | 4 | MODE-02 LDA path (11 LDAs) | tier-2 | `cargo xtask validate --backend cpu --mode potential --filter lda` |
| 03-W4-02 | 03-09 | 4 | MODE-02 GGA divergence (line-for-line port of XCFunctional.cpp:637-790) | tier-2 | `cargo xtask validate --backend cpu --mode potential --filter gga` |
| 03-W4-03 | 03-09 | 4 | MODE-05 `output_length ∈ {2, 3}` | unit | `cargo test -p xcfun-eval output_length_potential` |
| 03-W4-04 | 03-09 | 4 | `eval_setup` rejects non-`_2ND_TAYLOR` Vars with `XcError::InvalidVars` | unit | `cargo test -p xcfun-eval eval_setup_rejects_non_2nd_taylor` |

### Wave 5 — Orders 3..=4 + ACC-04 residual re-run

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command |
|---------|------|------|-------------|-----------|-------------------|
| 03-W5-01 | 03-10 | 5 | MODE-01 orders 3..=4 `PartialDerivatives` layout matches C++ | tier-2 | `cargo xtask validate --backend cpu --order 4 --filter gga` |
| 03-W5-02 | 03-10 | 5 | Phase-2 ACC-04 residual (VWN3C/VWN5C/PW92C/PZ81C near-clamp) at order 2 | tier-2 | `cargo xtask validate --backend cpu --order 2 --filter 'vwn3c\|vwn5c\|pw92c\|pz81c'` |
| 03-W5-03 | 03-10 | 5 | Supplemental 400-point grid covers new strata | tier-2 | `cargo xtask validate --backend cpu --order 2 --grid supplemental` |
| 03-W5-04 | 03-10 | 5 | Phase gate: all 47/51 functionals GREEN at order 4 | tier-2 | Phase gate command (above) |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Source: 03-RESEARCH.md §"Wave 0 Gaps".*

- [ ] `crates/xcfun-ad/src/expand/expm1.rs` — D-05 expand body (port of `ctaylor_math.hpp:85-102`)
- [ ] `crates/xcfun-ad/src/math.rs` — extend with `ctaylor_expm1` + `ctaylor_sqrtx_asinh_sqrtx`
- [ ] `crates/xcfun-ad/tests/golden_expand.rs` — add `expm1` test module
- [ ] `crates/xcfun-ad/tests/golden_composed.rs` — add `sqrtx_asinh_sqrtx` test module
- [ ] `xtask/src/bin/regen_ad_fixtures.rs` — emit 500 expm1 + 500 sqrtx_asinh_sqrtx fixtures (mpmath prec=200 ground truth)
- [ ] `crates/xcfun-eval/src/functionals/gga/shared/{pbex,pbec_eps,pw91_like,b97_poly,optx,constants}.rs` — 6 new shared helper modules (D-08)
- [ ] `crates/xcfun-eval/src/density_vars/build.rs` — 7 new `build_xc_<variant>` functions + 7 comptime if-chain arms (D-10 — correct discriminants 27..30 per research off-by-one finding)
- [ ] `crates/xcfun-eval/src/functional.rs` — 4 `_2ND_TAYLOR` vars match arms in `run_launch`; extend `(id, n)` match for Mode::Potential LDA path (N=1)
- [ ] `crates/xcfun-eval/tests/regularize_2nd_taylor.rs` — new test file (CONTEXT specifics)
- [ ] `crates/xcfun-eval/src/functionals/gga/mod.rs` — placeholder `pub mod shared; pub mod pbe;` etc.
- [ ] *(Conditional — if BRX/BRC/BRXC + CSC NOT deferred to Phase 4)* `crates/xcfun-ad/src/taylor.rs` + `crates/xcfun-ad/src/br_inverse.rs` — port `taylor.hpp` + `BR_taylor` Newton-inverse. **Research recommends deferral.**

**If BR/CSC deferred (recommended):** 10 Wave-0 gaps total.
**If not deferred:** ~14 Wave-0 gaps (~2× scope).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CONTEXT D-10 discriminant correction (27..30 vs stated 26..29) | — | Source of truth is `crates/xcfun-core/src/enums.rs:73-76`; manual diff required to confirm amendment lands before if-chain wiring | `grep -n "_2ND_TAYLOR" crates/xcfun-core/src/enums.rs` — confirm discriminants 27, 28, 29, 30 |
| BR/CSC deferral decision captured in CONTEXT.md amendments | — | Scope boundary decision; planner writes amendment to 03-CONTEXT.md `<decisions>` block | Read planner output for amendment; confirm `D-03` revised wave plan reflects deferral |

---

## Validation Sign-Off

- [ ] All tasks have automated verify commands or explicit Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (tier-1 gate after every commit — guaranteed by wave discipline)
- [ ] Wave 0 covers all MISSING references (10 gaps listed above; +4 if BR/CSC not deferred)
- [ ] No watch-mode flags (Rust test is one-shot per invocation)
- [ ] Feedback latency < 30 s tier-1; < 5 min phase gate
- [ ] `nyquist_compliant: true` set in frontmatter (after planner confirms all plan tasks map to this table)

**Approval:** pending — planner to confirm mapping completeness, then flip `nyquist_compliant: true`.
