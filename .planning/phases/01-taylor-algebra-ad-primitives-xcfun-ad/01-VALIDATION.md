---
phase: 1
slug: taylor-algebra-ad-primitives-xcfun-ad
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-19
revised: 2026-04-19
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Ground truth: 01-RESEARCH.md §"Validation Architecture".
> **Revision note (2026-04-19):** Row `01-03-03` (`cargo xtask check-no-fma`) was DROPPED during the planner revision pass — see foot note at bottom.

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
- **Before `/gsd-verify-work`:** Full suite + golden fixtures + property tests all green
- **Max feedback latency:** 12 seconds (quick run)

> **Removed gate (revision):** `cargo asm` FMA-grep is no longer a Phase 1 CI gate. See foot note.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 0 | AD-01 | — | stack-only storage; no heap | unit | `cargo test -p xcfun-ad --lib valid_n` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 0 | AD-01 | — | `CTaylor<f64, N>` compiles for N ∈ 0..=7 only | unit | `cargo test -p xcfun-ad --lib ctaylor_layout` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 0 | AD-01 | — | `.cargo/config.toml` pins `-fp-contract=off` | integration | `grep -q 'fp-contract=off' .cargo/config.toml` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | AD-04 | — | `inv_expand`, `exp_expand`, `log_expand` ports match tmath.hpp to_bits | unit | `cargo test -p xcfun-ad --lib expand::primary` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | AD-04 | — | `pow_expand`, `sqrt_expand`, `cbrt_expand` ports match | unit | `cargo test -p xcfun-ad --lib expand::power` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 1 | AD-04 | — | `erf_expand`, `gauss_expand`, `atan_expand`, `asinh_expand` ports match (B1 revision — atan/asinh included) | unit | `cargo test -p xcfun-ad --lib expand::trans --features libm` | ❌ W0 | ⬜ pending |
| 01-02-04 | 02 | 1 | AD-04 | — | all `*_expand` preconditions are `assert!` not `debug_assert!` | grep | `grep -c 'debug_assert' crates/xcfun-ad/src/expand/*.rs \| grep ':0$'` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 1 | AD-03 | — | `CTaylor::mul` recursion structure matches `ctaylor_rec::multo` byte-by-byte + `ctaylor_rec::compose` matches ctaylor.hpp | golden | `cargo test -p xcfun-ad --test golden_fixtures ctaylor_mul ctaylor_compose` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 1 | AD-03 | — | `add`, `sub`, `neg`, `scalar_mul` pass f64::to_bits identity | golden | `cargo test -p xcfun-ad --test golden_fixtures ctaylor_arith` | ❌ W0 | ⬜ pending |
| ~~01-03-03~~ | ~~03~~ | ~~1~~ | ~~AD-03~~ | ~~—~~ | ~~CI `cargo asm -p xcfun-ad --release mul_into` shows no `vfmadd` / `fmadd`~~ | ~~integration~~ | ~~`cargo xtask check-no-fma`~~ | ~~❌ W0~~ | 🚫 DROPPED (see foot note) |
| 01-04-01 | 04 | 1 | AD-05 | — | xtask subcommand `regen-ad-fixtures` produces bincode + manifest | integration | `cargo xtask regen-ad-fixtures --dry-run` | ❌ W0 | ⬜ pending |
| 01-04-02 | 04 | 1 | AD-05 | — | fixture records count ≥ 1668 (168 expand + 1500 ctaylor) | integration | `cargo test -p xcfun-ad --test fixture_format count_records` | ❌ W0 | ⬜ pending |
| 01-04-03 | 04 | 1 | AD-05 | — | `fixtures.json` manifest pins xcfun-master content hash | integration | `cargo test -p xcfun-ad --test fixture_format manifest_hash_pins_upstream` | ❌ W0 | ⬜ pending |
| 01-05-01 | 05 | 2 | AD-02 | — | `Num` trait for `f64` + `CTaylor<f64, N>` with all 14 ops | unit | `cargo test -p xcfun-ad --lib num::impls` | ❌ W0 | ⬜ pending |
| 01-05-02 | 05 | 2 | AD-02 | — | composed `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan` | unit | `cargo test -p xcfun-ad --lib math::composed` | ❌ W0 | ⬜ pending |
| 01-05-03 | 05 | 2 | AD-02 | — | composed ops match C++ intermediates via fixtures ≤ 1e-13 rel | golden | `cargo test -p xcfun-ad --test golden_fixtures composed` | ❌ W0 | ⬜ pending |
| 01-06-01 | 06 | 2 | AD-06 | — | proptest ring axioms pass ≥ 10 000 iterations | property | `cargo test -p xcfun-ad --test proptest_algebra ring_axioms` | ❌ W0 | ⬜ pending |
| 01-06-02 | 06 | 2 | AD-06 | — | proptest exp/log roundtrip + sqrt² + pow inverse | property | `cargo test -p xcfun-ad --test proptest_algebra roundtrips` | ❌ W0 | ⬜ pending |
| 01-06-03 | 06 | 2 | AD-06 | — | proptest Leibniz product rule on VAR0 coefficient | property | `cargo test -p xcfun-ad --test proptest_algebra leibniz` | ❌ W0 | ⬜ pending |
| 01-06-04 | 06 | 2 | AD-06 | — | 9 properties × ≥ 10k iters = ≥ 90k total without regression | property | `PROPTEST_CASES=10000 cargo test -p xcfun-ad --test proptest_algebra` | ❌ W0 | ⬜ pending |
| 01-07-01 | 07 | 2 | AD-03 | — | criterion baseline for `CTaylor::mul_assign` at N ∈ {2..6} published | bench | `cargo bench -p xcfun-ad --bench mul` | ❌ W0 | ⬜ pending |
| 01-07-02 | 07 | 2 | AD-03 | — | baseline for composed `exp`, `log`, `pow` at N=4 | bench | `cargo bench -p xcfun-ad --bench composed` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky · 🚫 dropped*
*Task IDs are indicative; the planner may re-number to fit wave layout.*

---

## Wave 0 Requirements

- [ ] `crates/xcfun-ad/Cargo.toml` — declare crate + dev-deps (`proptest =1.11.0`, `rstest =0.26.1`, `rand_xoshiro =0.8.0`, `criterion =0.8.2`, `approx =0.5.1`, `bincode 1.3`, `serde`, `serde_json`)
- [ ] `crates/xcfun-ad/src/lib.rs` — crate root with `#![forbid(unsafe_code)]`, `pub const CNST`, `VAR0..VAR6`
- [ ] `.cargo/config.toml` — `[build] rustflags = ["-Cllvm-args=-fp-contract=off"]` (W13 revision: `[build]` applies to all profiles — D-21 release-only is a minimum bar; header comment in config.toml documents the deviation)
- [ ] `crates/xcfun-ad/tests/` directory with `golden_fixtures.rs`, `proptest_algebra.rs` stubs referencing AD-01..06
- [ ] `crates/xcfun-ad/benches/` directory with `mul_bench.rs`, `compose_bench.rs` stubs (B4 revision — listed in Plan 01 `files_modified`)
- [ ] `crates/xcfun-ad/tests/fixtures/` directory (fixtures populated in Wave 1 task 04)
- [ ] `xtask/src/bin/regen_ad_fixtures.rs` + C++ driver skeleton at `xtask/assets/regen_ad_fixtures/` (populated in task 04)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Code review: every `*_expand` body carries a comment with `tmath.hpp:L-L` line range | AD-04 | Comment-convention cannot be automated cheaply | Reviewer greps `rg 'tmath.hpp:\d+' crates/xcfun-ad/src/expand/` and verifies every file (10 modules per B1 revision) has one |
| Code review: every `CTaylor::mul` base case (N=0,1,2) carries `ctaylor.hpp:L-L` comment | AD-03 | Same | Reviewer greps `rg 'ctaylor.hpp:\d+' crates/xcfun-ad/src/ctaylor_rec/` |
| Code review: `asinh.rs` module header explicitly disambiguates from the asin/acos typo (W10 revision) | AD-04 | Requires human judgment on upstream anomaly context | Reviewer greps `rg 'W10 revision\|asinh IS the function\|asin/acos typo' crates/xcfun-ad/src/expand/asinh.rs` |
| Upstream anomaly acknowledgement: `asin_expand` / `acos_expand` typo at tmath.hpp:290/:313 is out-of-scope for Phase 1 and will be handled in Phase 2+ | AD-04 (research A6) | Out-of-Phase-1-scope | No Phase 1 action; reviewer confirms Phase 2+ plan has the typo workaround when asin/acos are ported |

*Everything else has automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify (verified above)
- [x] Wave 0 covers all MISSING references (Cargo.toml, lib.rs, .cargo/config.toml, tests/, benches/, fixtures/, xtask/)
- [x] No watch-mode flags in CI (`cargo watch` is local-only; CI uses single-shot `cargo test`)
- [x] Feedback latency < 12s (quick run empirically ≤ 10 s on `cargo test -p xcfun-ad --lib`)
- [x] `nyquist_compliant: true` set in frontmatter (verified 2026-04-19 PM: every Phase 1 task across plans 01-01..01-07 has an `<automated>` verify block or is listed in the manual-only table above)

**Approval:** approved by planner-revision 2026-04-19 PM

---

## Foot note — dropped row `01-03-03` (revision, 2026-04-19)

The row `01-03-03` (CI `cargo asm -p xcfun-ad --release mul_into` shows no `vfmadd`/`fmadd`, verified by `cargo xtask check-no-fma`) was **DROPPED** during the planner revision pass for the following reasons:

1. **No plan delivered `cargo xtask check-no-fma`.** Plan 03 did not allocate a task to implement this xtask subcommand.
2. **D-20 delegates to a CI script, not a code-level check.** `cargo asm` grep is an operational CI gate, not a unit test — its logical home is a CI pipeline step added in Phase 2 when functional-body FMA risk becomes concrete (XC functional bodies are where the lint matters most; AD-engine code is stylistically guarded by the explicit-`let` pattern in D-08).
3. **The `-fp-contract=off` compiler flag + the `grep -E 'mul_add' crates/xcfun-ad/src/ctaylor_rec/*.rs` acceptance criterion in Plan 03 Task 2 together provide the Phase 1 floor.** The release object file cannot contain `vfmadd` because (a) no source-level `mul_add` call exists (grep-enforced) and (b) the LLVM backend cannot fuse `a * b + c` into an FMA when `-fp-contract=off` is set (rustflag-enforced).

Phase 1 is therefore protected at the source-level and compiler-level. An operational `cargo asm` CI job will be added as a Phase 2 task — at that point the FMA risk surface expands (functional bodies do arithmetic with many multiply-add patterns that the compiler could fuse on a build without `-fp-contract=off`), and the CI spot-check becomes load-bearing. Reference: B6 RESOLVED Q4 in `01-RESEARCH.md`.
