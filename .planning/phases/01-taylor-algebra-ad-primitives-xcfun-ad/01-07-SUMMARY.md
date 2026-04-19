---
phase: 01-taylor-algebra-ad-primitives-xcfun-ad
plan: 07
subsystem: testing
tags: [proptest, criterion, cubecl-cpu, batch-per-property, fma-check, asm-gate, docs]

# Dependency graph
requires:
  - phase: 01-taylor-algebra-ad-primitives-xcfun-ad
    provides: cubecl-native CTaylor + ctaylor_rec{mul,multo,compose} + composed math.rs fns
provides:
  - 11 proptest batch-per-property tests at 110 000 aggregate iterations
  - criterion baselines for ctaylor_mul (N in 2..=6) and composed ctaylor_{exp,log,pow} at N=4 x batch {1, 64, 1024}
  - xtask check-no-fma asm gate (ROADMAP Phase 1 SC #6)
  - docs/design/06,07,12 D-28 cubecl-pivot revisions
affects: [Phase 2 LDA tier, Phase 6 GPU runtimes]

# Tech tracking
tech-stack:
  added:
    - rustc-demangle = "0.1" (xtask app-boundary only — FMA symbol demangling)
  patterns:
    - batch-per-property kernel (CONTEXT.md D-18): one kernel launch over 10k inputs via CubeCount::Static(K,1,1) x CubeDim::new_1d(1) + ABSOLUTE_POS dispatch
    - inline-per-N bench kernel pattern (N in 2,3) with explicit offset arithmetic — workaround for missing Array<F> sub-slicing in cubecl 0.10-pre.3
    - asm-gate FMA detection via rustc_demangle symbol-body grep across `ctaylor_mul*` (D-03 escalation path)

key-files:
  created:
    - crates/xcfun-ad/tests/proptest_algebra.rs
    - xtask/src/bin/check_no_fma.rs
    - docs/design/06-cubecl-strategy.md (revision-history banner only — file was pre-existing untracked)
    - docs/design/07-accuracy-strategy.md (revision-history banner + check-no-fma paragraph)
    - docs/design/12-design-decisions.md (revision-history + D1/D2/D4 SUPERSEDED banners)
    - .planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/deferred-items.md
  modified:
    - crates/xcfun-ad/benches/mul_bench.rs (populated from stub)
    - crates/xcfun-ad/benches/compose_bench.rs (populated from stub)
    - crates/xcfun-ad/Cargo.toml (required-features = ["testing"] on both benches)
    - xtask/Cargo.toml (register check-no-fma binary + rustc-demangle dep)
    - .planning/REQUIREMENTS.md (AD-04, AD-06 marked [x]; AD-01 traceability row synced)
    - .planning/ROADMAP.md (01-07 [x]; Phase 1 [x]; progress table 7/7)

key-decisions:
  - "batch-per-property launch geometry: CubeCount::Static(iters, 1, 1) x CubeDim::new_1d(1). The proposal's CubeDim::new_1d(iters) with one cube raised cubecl-cpu CallError (MLIR JIT resource budget); switching to many-cubes-one-unit resolves it and ABSOLUTE_POS still gives unique indices across all units."
  - "Bench N=4,5,6 fallback: cubecl 0.10-pre.3 has no Array<F> sub-slicing, so batched inline of the per-N mul formula at N >= 4 would require 15 kB+ of fully-unrolled body. Ship inline for N in {2,3} (real cross-batch amortization) and single-kernel-per-element for N in {4,5,6} (launch-amortization-only signal). Document limitation in bench header."
  - "proptest log roundtrip uses Float::ln() not Float::log() — the cubecl Float::log signature takes a base as second argument; the natural log is `.ln()`."
  - "cargo fmt reformats workspace-wide, so limit the Plan 07 fmt commit to the four new files. Pre-existing formatting drift in ~16 other files is recorded in deferred-items.md; out of scope per Rule-3 boundary."
  - "D-28 docs/design/ edits are prepended revision-history banners + SUPERSEDED markers. Original D1/D2/D4 text retained below each banner per Task 4 acceptance criteria."

patterns-established:
  - "Batch-per-property kernel pattern for proptest: 10k-input generate -> one flat device buffer -> one kernel launch with CubeCount::Static(iters,1,1) + CubeDim::new_1d(1) -> host-side result aggregation"
  - "required-features on [[bench]] entries: for_tests is cfg-gated, so benches that need cpu_client() must carry `required-features = [\"testing\"]`"
  - "xtask asm-gate: cargo rustc -p X --release --lib -- --emit=asm, then parse target/release/deps/X-*.s splitting by `sym:` labels, rustc_demangle, grep for forbidden mnemonics inside the target-symbol body"

requirements-completed: [AD-03, AD-06]

# Metrics
duration: 17min
completed: 2026-04-19
---

# Phase 01 Plan 07: Proptest batch-per-property + criterion benchmarks + phase sign-off Summary

**11 proptest batch-per-property tests at 110 000 aggregate cubecl-cpu iterations, criterion baselines at 24 (N, batch) points, active FMA-gate xtask satisfying ROADMAP Phase 1 SC #6, and three docs/design/ files carrying the D-28 cubecl-pivot revisions.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-04-19T21:51:00Z
- **Completed:** 2026-04-19T22:08:17Z
- **Tasks:** 5
- **Files modified:** 11 (4 new + 7 edits)

## Accomplishments

- AD-06 closed: 11 property tests covering ring axioms + Leibniz + exp/log/sqrt/pow transcendentals, each at 10 000 iterations via the D-18 batch-per-property kernel pattern. Zero failures on cubecl-cpu.
- AD-03 bench component closed: criterion baselines for `ctaylor_mul` at N in {2, 3, 4, 5, 6} x batch {1, 64, 1024} = 15 points + composed `ctaylor_{exp,log,pow}` at N=4 x batch {1, 64, 1024} = 9 points.
- ROADMAP Phase 1 Success Criterion #6 actively enforced: `cargo run -p xtask --bin check-no-fma` emits asm for `xcfun-ad --release`, scans every `ctaylor_mul*` symbol for forbidden FMA mnemonics, and prints `PASS` on the current build. Exit 2 on any match triggers D-03 escalation.
- D-28 documentation: `docs/design/06-cubecl-strategy.md`, `07-accuracy-strategy.md`, `12-design-decisions.md` carry explicit revision-history banners + per-decision SUPERSEDED markers on D1/D2/D4 directing readers to `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`.
- Phase 1 is shippable: `cargo test -p xcfun-ad --features "cpu testing" --all-targets` passes 86 tests across 10 binaries + 24 bench points smoke-compiled.

## Task Commits

Each task was committed atomically:

1. **Task 1: proptest_algebra batch-per-property 11 props x 10k iters** — `3514217` (test)
2. **Task 2: criterion baselines mul + composed** — `a3e4c3f` (perf)
3. **Task 3: xtask check-no-fma asm gate** — `2882b58` (feat)
4. **Task 4: D-28 design-doc updates 06/07/12** — `3a6927e` (docs)
5. **Task 5a: cargo fmt on Plan 07 new files** — `db792bf` (style)
6. **Task 5b: phase sign-off (this commit, metadata only)** — `TBD` (docs)

## Files Created/Modified

- `crates/xcfun-ad/tests/proptest_algebra.rs` — 11 batch-per-property tests, each honoring `PROPTEST_CASES` env var (default 10 000); one kernel launch per property with `CubeCount::Static(iters, 1, 1)` and `CubeDim::new_1d(1)`
- `crates/xcfun-ad/benches/mul_bench.rs` — inline-per-N kernels for N in {2, 3} + single-kernel fallback via `xcfun_ad::ctaylor_rec::mul::ctaylor_mul` for N in {4, 5, 6}
- `crates/xcfun-ad/benches/compose_bench.rs` — exp/log/pow bench at N=4; per-element kernel launch with `bs` iterations
- `xtask/src/bin/check_no_fma.rs` — asm-gate binary with 26-mnemonic forbidden list (x86-64 vfmadd/vfmsub/vfnmadd/vfnmsub × 3 form codes + aarch64 fmadd/fmsub/fnmadd/fnmsub + LLVM fma213/fma231 belt-and-suspenders)
- `xtask/Cargo.toml` — new `[[bin]] name = "check-no-fma"` + `rustc-demangle = "0.1"` dep
- `crates/xcfun-ad/Cargo.toml` — `required-features = ["testing"]` on both `[[bench]]` entries
- `docs/design/06-cubecl-strategy.md` — revision-history banner at top; §1 paragraph acknowledges xcfun-ad as cubecl-native baseline
- `docs/design/07-accuracy-strategy.md` — revision-history banner + §3 `check-no-fma` paragraph; zero occurrences of "Rust scalar port"
- `docs/design/12-design-decisions.md` — top-level revision-history + SUPERSEDED banners on D1, D2, D4 (original text retained)
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/deferred-items.md` — logs ~84 pre-existing cargo-fmt drifts (out of Plan 07 scope)
- `.planning/REQUIREMENTS.md` — AD-06 marked [x]; AD-04 and AD-01 traceability rows synced with STATE.md
- `.planning/ROADMAP.md` — 01-07-PLAN.md [x]; Phase 1 [x]; progress table 7/7

## Final test tally

`cargo test -p xcfun-ad --features "cpu testing" --all-targets` (unit + integration binaries):

| Binary | Passed |
|--------|--------|
| ctaylor_unit | 13 |
| cubecl_spike | 4 |
| expand_primary | 18 |
| expand_trans | 12 |
| golden_composed | 1 (180 internal records) |
| golden_expand | 1 (168 internal records) |
| golden_mul | 1 (250 internal records) |
| math_unit | 14 |
| proptest_algebra | **11 (Plan 07 new)** |
| tfuns_unit | 11 |
| **Total** | **86 top-level tests + ~598 golden records + 110 000 proptest iters** |

## Criterion baseline points (smoke-tested via `-- --test`)

| Bench | Point count | Points |
|-------|-------------|--------|
| mul_bench | 15 | N in {2,3,4,5,6} x batch in {1,64,1024} |
| compose_bench | 9 | op in {exp, log, pow} x batch in {1,64,1024} at N=4 |
| **Total** | **24** | all green on `cargo bench ... -- --test` |

No regression gate (v1 deferral per CONTEXT.md D-19; PERF-01 deferred).

## check-no-fma output (active evidence for ROADMAP Phase 1 SC #6)

```
check-no-fma: emitting asm for xcfun-ad --release ...
    Finished `release` profile [optimized] target(s) in 0.35s
check-no-fma: scanning 1 asm file(s)
check-no-fma: PASS — no FMA mnemonics on ctaylor_mul symbols.
```

Exit code: 0. The `-Cllvm-args=-fp-contract=off` flag in both `[build]` and `[target.'cfg(all())']` sections of `.cargo/config.toml` is effective through cubecl-cpu's MLIR JIT path, validated at the source level by this gate.

## AD-01..AD-06 coverage

| Req | Coverage |
|-----|----------|
| AD-01 | `crates/xcfun-ad/src/ctaylor.rs` + `tests/cubecl_spike.rs` (N ∈ 0..=7 CTaylor<F, const N: u32>) |
| AD-02 | `crates/xcfun-ad/src/math.rs` + `tests/math_unit.rs` + `golden_composed.rs` (9 composed fns) |
| AD-03 | `crates/xcfun-ad/src/ctaylor_rec/mul.rs` + `tests/golden_mul.rs` + `benches/mul_bench.rs` + xtask check-no-fma (recursion order + asm evidence) |
| AD-04 | `crates/xcfun-ad/src/expand/*.rs` + `tests/expand_primary.rs` + `tests/expand_trans.rs` + `golden_expand.rs` (8 `*_expand` fns) |
| AD-05 | `golden_mul.rs` + `golden_expand.rs` + `golden_composed.rs` (598 C++ reference records at 1e-12 on cubecl-cpu) |
| AD-06 | `tests/proptest_algebra.rs` (11 props × 10 000 iters, zero failures) |

## D-28 doc-edit excerpts

**`docs/design/06-cubecl-strategy.md`** — new top-of-file banner:
> **2026-04-19 PM — Phase 1 cubecl pivot.** The Taylor-algebra AD engine (`xcfun-ad`) is now **cubecl-native from day one**: `CTaylor<F: Float, const N: u32>` is a pure `#[cube]` type backed by cubecl `Array<F>` storage … Pre-pivot text below that positions `xcfun-ad` as a scalar Rust crate consumed by cubecl-bearing downstream crates is **SUPERSEDED**.

**`docs/design/07-accuracy-strategy.md`** — new §3 paragraph:
> On the cubecl-cpu runtime … any re-association risk is mitigated by the repository-wide `.cargo/config.toml` pinning `-Cllvm-args=-fp-contract=off` under both `[build]` and `[target.'cfg(all())']`, and validated actively by `cargo run -p xtask --bin check-no-fma` (Phase 1 Plan 07 Task 3) …

**`docs/design/12-design-decisions.md`** — 3 SUPERSEDED banners confirmed by `grep -c "SUPERSEDED 2026-04-19 PM by Phase 1 cubecl pivot"` → returns 3. Original D1/D2/D4 text retained below each banner (traceability).

## Decisions Made

- **Launch geometry for batch-per-property:** `CubeCount::Static(iters, 1, 1)` × `CubeDim::new_1d(1)` instead of the single-cube-many-units shape. Rationale: cubecl-cpu 0.10-pre.3 raises `CallError` on the single-cube shape at iters = 10 000 because the MLIR JIT's per-cube resource budget is exhausted. Many-cubes-one-unit spreads the dispatch across independent work-items and preserves `ABSOLUTE_POS` uniqueness.
- **N ∈ {4, 5, 6} bench fallback:** cubecl 0.10-pre.3 does not expose zero-copy `Array<F>` sub-slicing across batch elements. Full inline at N ≥ 4 would cost 15 kB+ of unrolled body per kernel. Taken: inline for {2, 3}; fall back to `xcfun_ad::ctaylor_rec::mul::ctaylor_mul` once-per-element for {4, 5, 6}. Bench header documents this trade-off.
- **Natural log is `.ln()` in cubecl Float:** the cubecl `Float::log` takes a base as second arg; `.ln()` is the no-arg natural log. Initial port used `.log()` and hit a compile error — this is a cubecl 0.10-pre.3 quirk worth noting for Phase 2+ ports.
- **D-28 docs edits are banners, not rewrites:** per Task 4 acceptance criteria, original D1/D2/D4 text is retained below each SUPERSEDED banner. This preserves historical traceability.
- **Pre-existing fmt drift is out of scope:** `cargo fmt --check` reports ~84 workspace-wide diffs from earlier plans. Per Rule-3 scope boundary, only the 4 Plan 07 files were formatted; the rest is logged to `deferred-items.md` for a future housekeeping commit.

## Deviations from Plan

### Rule 1 — Bug: batch-per-property launch geometry

- **Found during:** Task 1 (first test run with plan's suggested `CubeDim::new_1d(iters)` + `CubeCount::Static(1,1,1)` geometry)
- **Issue:** All 11 tests panicked with `CallError` from cubecl-runtime's `client.rs:104`. cubecl-cpu's MLIR JIT could not lower a 10 000-unit single-cube kernel.
- **Fix:** Swapped to `CubeCount::Static(iters as u32, 1, 1)` + `CubeDim::new_1d(1)` — K cubes × 1 unit. `ABSOLUTE_POS` still provides unique per-iter indices.
- **Files modified:** `crates/xcfun-ad/tests/proptest_algebra.rs`
- **Verification:** `PROPTEST_CASES=10000 cargo test --features "cpu testing" --test proptest_algebra` now passes 11/11 in 0.44 s.
- **Committed in:** `3514217` (Task 1 commit)

### Rule 3 — Blocking: `required-features = ["testing"]` on bench entries

- **Found during:** Task 2 (first bench compile)
- **Issue:** `cargo bench -p xcfun-ad --features cpu --bench mul_bench` failed with `unresolved import xcfun_ad::for_tests` because `for_tests` is gated behind `feature = "testing"`.
- **Fix:** Added `required-features = ["testing"]` to both `[[bench]]` entries in `crates/xcfun-ad/Cargo.toml`. Also updated plan-level smoke-test command to carry `"cpu testing"` jointly.
- **Files modified:** `crates/xcfun-ad/Cargo.toml`
- **Verification:** `cargo bench -p xcfun-ad --features "cpu testing" --bench mul_bench -- --test` now runs all 15 bench points.
- **Committed in:** `a3e4c3f` (Task 2 commit)

### Rule 1 — Bug: `Float::log` signature quirk

- **Found during:** Task 1 (exp_log_roundtrip compile)
- **Issue:** `y.log()` failed with `this method takes 1 argument but 0 arguments were supplied`. cubecl's `Float::log` takes a base as a second argument; the no-arg natural log is `.ln()`.
- **Fix:** Swapped `.log()` → `.ln()` in `exp_log_roundtrip_kernel`.
- **Files modified:** `crates/xcfun-ad/tests/proptest_algebra.rs`
- **Verification:** test compiles + passes.
- **Committed in:** `3514217` (Task 1 commit, same sweep as the launch-geometry fix)

### Rule 1 — Bug: `ValueTree` trait not in scope

- **Found during:** Task 1 (first compile)
- **Issue:** `tree.current()` failed with E0599 because `ValueTree` (which provides `current`) was not imported.
- **Fix:** Added `use proptest::strategy::{Strategy, ValueTree};`.
- **Files modified:** `crates/xcfun-ad/tests/proptest_algebra.rs`
- **Verification:** compile succeeds.
- **Committed in:** `3514217` (Task 1 commit)

---

**Total deviations:** 4 auto-fixed (3 bugs + 1 blocking)
**Impact on plan:** All four fixes essential to make the plan's acceptance criteria achievable on cubecl-cpu 0.10-pre.3. No scope creep beyond what the plan's `<action>` explicitly called out as executor choices (e.g., Option A inline vs Option B per-element for N ≥ 4 — documented).

## Issues Encountered

- **cubecl 0.10-pre.3 API quirks consolidated:** `ABSOLUTE_POS` is `usize` (not `u32`); `Array<F>::len()` returns `u32` inside `#[cube]` bodies — mixed `usize * u32` operations need explicit typing. The inline-N bench kernels were initially written with `size = 4_u32` and hit `E0277 cannot multiply usize by u32` — changing to `size = 4_usize` resolved it. Added to the quirk log (`crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs` header).
- **cargo fmt workspace-wide drift:** Running `cargo fmt` formatted 16 pre-existing files unrelated to Plan 07. These were reverted; the formatting normalization is logged to `deferred-items.md` for a dedicated housekeeping commit.

## Self-Check: PASSED

Automated verification:

```
crates/xcfun-ad/tests/proptest_algebra.rs      FOUND
crates/xcfun-ad/benches/mul_bench.rs           FOUND
crates/xcfun-ad/benches/compose_bench.rs       FOUND
xtask/src/bin/check_no_fma.rs                  FOUND
xtask/Cargo.toml                               FOUND
crates/xcfun-ad/Cargo.toml                     FOUND
docs/design/06-cubecl-strategy.md              FOUND
docs/design/07-accuracy-strategy.md            FOUND
docs/design/12-design-decisions.md             FOUND

commit 3514217                                 FOUND (Task 1)
commit a3e4c3f                                 FOUND (Task 2)
commit 2882b58                                 FOUND (Task 3)
commit 3a6927e                                 FOUND (Task 4)
commit db792bf                                 FOUND (Task 5a)

grep -c "^#\[test\]$" tests/proptest_algebra.rs = 11          ≥ 11 ✓
grep -c "Rust scalar port" docs/design/07-accuracy-strategy.md = 0  ✓
grep -c "SUPERSEDED 2026-04-19 PM by Phase 1 cubecl pivot" docs/design/12-design-decisions.md = 3  ≥ 3 ✓
check-no-fma                                   PASS ✓
cargo test -p xcfun-ad --features "cpu testing" = 86 passed, 0 failed ✓
```

## Next Phase Readiness

**Phase 1 cubecl-native `xcfun-ad` is complete; Phase 2 (xcfun-core + LDA tier + parity harness) can now consume `xcfun-ad` via `#[cube] fn` imports.**

Ready to advance:
- Phase 2 planning can begin (`/gsd-plan-phase 2`).
- All 6 AD requirements (AD-01..AD-06) marked complete in REQUIREMENTS.md.
- ROADMAP Phase 1 row: 7/7 plans complete.
- Every AD-01..AD-06 has at least one test file + CI-enforceable gate.
- `cargo run -p xtask --bin check-no-fma` can move into the `.github/workflows/ci.yml` required-checks list.

No blockers. Pre-existing workspace-wide fmt drift deferred to a separate housekeeping commit (see `deferred-items.md`); it does NOT gate Phase 2.

---
*Phase: 01-taylor-algebra-ad-primitives-xcfun-ad*
*Completed: 2026-04-19*
