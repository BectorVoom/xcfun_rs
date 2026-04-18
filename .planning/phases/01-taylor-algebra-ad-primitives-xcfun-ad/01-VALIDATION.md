---
phase: 1
slug: taylor-algebra-ad-primitives-xcfun-ad
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-19
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Ground truth: 01-RESEARCH.md §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` + `proptest 1.11` + `cargo-nextest` (optional, faster) + `criterion 0.8.2` (bench only) |
| **Config file** | `crates/xcfun-ad/Cargo.toml` `[dev-dependencies]` + `.cargo/config.toml` (`-Cllvm-args=-fp-contract=off` in release) |
| **Quick run command** | `cargo test -p xcfun-ad --lib --tests` |
| **Full suite command** | `cargo nextest run -p xcfun-ad --all-features` (or `cargo test` fallback) |
| **Golden fixture command** | `cargo test -p xcfun-ad --test golden_fixtures` |
| **Property-test command** | `cargo test -p xcfun-ad --test proptest_algebra -- --test-threads=1` |
| **Estimated runtime** | ~8–12 seconds quick; ~30 seconds full (incl. 90k+ property iterations) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p xcfun-ad --lib` (≤ 5 s)
- **After every plan wave:** Run full suite (`cargo nextest run -p xcfun-ad`)
- **Before `/gsd-verify-work`:** Full suite + golden fixtures + property tests all green; `cargo asm` FMA-grep CI check green
- **Max feedback latency:** 12 seconds (quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 0 | AD-01 | — | stack-only storage; no heap | unit | `cargo test -p xcfun-ad --lib valid_n` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 0 | AD-01 | — | `CTaylor<f64, N>` compiles for N ∈ 0..=7 only | unit | `cargo test -p xcfun-ad --lib ctaylor_layout` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 0 | AD-01 | — | `.cargo/config.toml` pins `-fp-contract=off` | integration | `grep -q 'fp-contract=off' .cargo/config.toml` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | AD-04 | — | `inv_expand`, `exp_expand`, `log_expand` ports match tmath.hpp to_bits | unit | `cargo test -p xcfun-ad --lib expand::primary` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | AD-04 | — | `pow_expand`, `sqrt_expand`, `cbrt_expand` ports match | unit | `cargo test -p xcfun-ad --lib expand::power` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 1 | AD-04 | — | `erf_expand`, `gauss_expand` ports match | unit | `cargo test -p xcfun-ad --lib expand::trans` | ❌ W0 | ⬜ pending |
| 01-02-04 | 02 | 1 | AD-04 | — | all `*_expand` preconditions are `assert!` not `debug_assert!` | grep | `grep -c 'debug_assert' crates/xcfun-ad/src/expand/*.rs \| grep ':0$'` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 1 | AD-03 | — | `CTaylor::mul` recursion structure matches `ctaylor_rec::multo` byte-by-byte | golden | `cargo test -p xcfun-ad --test golden_fixtures ctaylor_mul` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 1 | AD-03 | — | `add`, `sub`, `neg`, `scalar_mul` pass f64::to_bits identity | golden | `cargo test -p xcfun-ad --test golden_fixtures ctaylor_arith` | ❌ W0 | ⬜ pending |
| 01-03-03 | 03 | 1 | AD-03 | — | CI `cargo asm -p xcfun-ad --release mul_into` shows no `vfmadd` / `fmadd` | integration | `cargo xtask check-no-fma` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 1 | AD-05 | — | xtask subcommand `regen-ad-fixtures` produces bincode + manifest | integration | `cargo xtask regen-ad-fixtures --dry-run` | ❌ W0 | ⬜ pending |
| 01-04-02 | 04 | 1 | AD-05 | — | fixture records count ≥ 1668 (168 expand + 1500 ctaylor) | integration | `cargo test -p xcfun-ad --test golden_fixtures count_records` | ❌ W0 | ⬜ pending |
| 01-04-03 | 04 | 1 | AD-05 | — | `fixtures.json` manifest pins xcfun-master content hash | integration | `cargo test -p xcfun-ad --test golden_fixtures manifest_hash` | ❌ W0 | ⬜ pending |
| 01-05-01 | 05 | 2 | AD-02 | — | `Num` trait for `f64` + `CTaylor<f64, N>` with all 14 ops | unit | `cargo test -p xcfun-ad --lib num::impls` | ❌ W0 | ⬜ pending |
| 01-05-02 | 05 | 2 | AD-02 | — | composed `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan` | unit | `cargo test -p xcfun-ad --lib math::composed` | ❌ W0 | ⬜ pending |
| 01-05-03 | 05 | 2 | AD-02 | — | composed ops match C++ intermediates via fixtures ≤ 1e-13 rel | golden | `cargo test -p xcfun-ad --test golden_fixtures composed` | ❌ W0 | ⬜ pending |
| 01-06-01 | 06 | 2 | AD-06 | — | proptest ring axioms pass ≥ 10 000 iterations | property | `cargo test -p xcfun-ad --test proptest_algebra ring_axioms` | ❌ W0 | ⬜ pending |
| 01-06-02 | 06 | 2 | AD-06 | — | proptest exp/log roundtrip + sqrt² + pow inverse | property | `cargo test -p xcfun-ad --test proptest_algebra roundtrips` | ❌ W0 | ⬜ pending |
| 01-06-03 | 06 | 2 | AD-06 | — | proptest Leibniz product rule on VAR0 coefficient | property | `cargo test -p xcfun-ad --test proptest_algebra leibniz` | ❌ W0 | ⬜ pending |
| 01-06-04 | 06 | 2 | AD-06 | — | 9 properties × ≥ 10k iters = ≥ 90k total without regression | property | `PROPTEST_CASES=10000 cargo test -p xcfun-ad --test proptest_algebra` | ❌ W0 | ⬜ pending |
| 01-07-01 | 07 | 2 | AD-03 | — | criterion baseline for `CTaylor::mul_assign` at N ∈ {2..6} published | bench | `cargo bench -p xcfun-ad --bench mul` | ❌ W0 | ⬜ pending |
| 01-07-02 | 07 | 2 | AD-03 | — | baseline for composed `exp`, `log`, `pow` at N=4 | bench | `cargo bench -p xcfun-ad --bench composed` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Task IDs are indicative; the planner may re-number to fit wave layout.*

---

## Wave 0 Requirements

- [ ] `crates/xcfun-ad/Cargo.toml` — declare crate + dev-deps (`proptest =1.11.0`, `rstest =0.26.1`, `rand_xoshiro =0.8.0`, `criterion =0.8.2`, `approx =0.5.1`, `bincode 1.3`, `serde`, `serde_json`)
- [ ] `crates/xcfun-ad/src/lib.rs` — crate root with `#![forbid(unsafe_code)]`, `pub const CNST`, `VAR0..VAR6`
- [ ] `.cargo/config.toml` — `[build] rustflags = ["-Cllvm-args=-fp-contract=off"]` in release profile
- [ ] `crates/xcfun-ad/tests/` directory with `golden_fixtures.rs`, `proptest_algebra.rs` stubs referencing AD-01..06
- [ ] `crates/xcfun-ad/benches/` directory with `mul.rs`, `composed.rs` stubs
- [ ] `crates/xcfun-ad/tests/fixtures/` directory (fixtures populated in Wave 1 task 04)
- [ ] `xtask/src/bin/regen_ad_fixtures.rs` + C++ driver skeleton at `xtask/cpp-driver/` (populated in task 04)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Code review: every `*_expand` body carries a comment with `tmath.hpp:L-L` line range | AD-04 | Comment-convention cannot be automated cheaply | Reviewer greps `rg 'tmath.hpp:\d+' crates/xcfun-ad/src/expand/` and verifies every file has one |
| Code review: every `CTaylor::mul` base case (N=0,1,2) carries `ctaylor.hpp:L-L` comment | AD-03 | Same | Reviewer greps `rg 'ctaylor.hpp:\d+' crates/xcfun-ad/src/ctaylor_rec/` |
| Upstream anomaly acknowledgement: `asin_expand` / `acos_expand` preserve the `t[0] = asinh(a)` typo from tmath.hpp:290, 313 verbatim and document it | AD-04 | Requires human judgment on upstream anomaly | Reviewer confirms RESEARCH.md A4 is reflected in code comments |

*Everything else has automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (verified above)
- [ ] Wave 0 covers all MISSING references (Cargo.toml, lib.rs, .cargo/config.toml, tests/, benches/, fixtures/, xtask/)
- [ ] No watch-mode flags in CI (`cargo watch` is local-only; CI uses single-shot `cargo test`)
- [ ] Feedback latency < 12s (quick run empirically ≤ 10 s on `cargo test -p xcfun-ad --lib`)
- [ ] `nyquist_compliant: true` set in frontmatter (flip when planner confirms every task in the final PLAN.md files has `<automated>` or is on the manual-only list above)

**Approval:** pending
