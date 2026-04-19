---
phase: 01-taylor-algebra-ad-primitives-xcfun-ad
plan: 05
subsystem: ad-engine
tags: [cubecl, cubecl-cpu, xtask, fixtures, bincode, golden-test, ctaylor-mul, c++-driver, sha256-manifest, taylor-algebra]

# Dependency graph
requires:
  - phase: 01-taylor-algebra-ad-primitives-xcfun-ad
    provides: |
      Plan 01-01 shipped the xtask workspace member with the regen-ad-fixtures
      binary skeleton and C++ driver stub linking
      xcfun-master/external/upstream/taylor/.
      Plan 01-02 shipped `ctaylor_mul` on cubecl-cpu for N ∈ 0..=3 with
      f64::to_bits identity tests against host-side reference mirrors, and
      explicitly deferred N ∈ 4..=7 to this plan's golden-fixture gate.

provides:
  - "xtask/src/fixtures.rs: shared FixtureRecord + FixturesManifest schema for driver writer and test readers"
  - "xtask/src/bin/regen_ad_fixtures.rs: full fixture regenerator — compiles C++ driver (c++ -std=c++17 -O2 -fno-fast-math -ffp-contract=off), parses records, partitions by op family, serialises to bincode, writes sha256 manifest"
  - "xtask/assets/regen_ad_fixtures/driver.cpp: full C++ driver emitting 168 expand records (3 inputs × 7 orders × 8 fns) + 250 mul records (50 seeds × 5 N) = 418 total, deterministic mt19937_64 seed 0x1234abcd"
  - "crates/xcfun-ad/tests/fixtures/{mul.bincode, expand.bincode, fixtures.json} committed — CI does NOT regenerate (D-19); byte-identical across re-runs on the same xcfun-master tree"
  - "crates/xcfun-ad/tests/golden_mul.rs: integration test loading mul.bincode, running ctaylor_mul on cubecl-cpu, asserting f64::to_bits identity for n_var ∈ 0..=3 (200 records) and rel-err ≤ 1e-13 for n_var = 4 (50 records)"
  - "crates/xcfun-ad/src/ctaylor_rec/mul.rs: ctaylor_mul_set_n4 (flattened from the 3-call recursion, ~110 explicit `let` bindings) + extended outer dispatch to N ∈ {0..=4}"
  - "Build-input convention: xcfun-master vendored symlink from parent checkout (entries added to .gitignore)"

affects:
  - 01-06 (golden_expand.rs + golden_composed.rs consume the same expand.bincode + schema)
  - 01-07 (benches) — baselines can reuse golden_mul launch pattern
  - Phase 2 (xcfun-core) — golden_mul is the first C++↔Rust parity test binding; sets the pattern for DensVars parity
  - Phase 6 (xcfun-gpu) — CUDA/Wgpu validation re-runs the same fixture sets at their respective tolerance budgets

# Tech tracking
tech-stack:
  added:
    - "sha2 0.10 (xtask-local app-boundary dep — manifest hash over xcfun-master taylor headers)"
    - "chrono 0.4 default-features=false, std+clock (xtask-local — fixtures.json generated_at RFC 3339)"
  patterns:
    - "Schema-duplication-on-purpose: FixtureRecord re-defined in the xcfun-ad test file verbatim rather than via an xtask path-dep, to keep the xcfun-ad dev-dep closure narrow"
    - "Deterministic regen: fixed mt19937_64 seed 0x1234abcd in driver.cpp + post-hoc sort_by on records before bincode serialisation → byte-identical output across runs"
    - "Pre-exec xcfun-master symlink pattern for worktree agents: `ln -s <main>/xcfun-master ./xcfun-master` + gitignore entry — build input stays out of commits"

key-files:
  created:
    - "xtask/src/fixtures.rs"
    - "xtask/assets/regen_ad_fixtures/driver.cpp (rewritten from Plan 01-01 stub)"
    - "xtask/src/bin/regen_ad_fixtures.rs (rewritten from Plan 01-01 stub)"
    - "crates/xcfun-ad/tests/fixtures/.gitkeep"
    - "crates/xcfun-ad/tests/fixtures/mul.bincode"
    - "crates/xcfun-ad/tests/fixtures/expand.bincode"
    - "crates/xcfun-ad/tests/fixtures/fixtures.json"
    - "crates/xcfun-ad/tests/golden_mul.rs"
  modified:
    - "xtask/Cargo.toml (added sha2, chrono)"
    - ".gitignore (added xcfun-master vendored symlink entry)"
    - "crates/xcfun-ad/src/ctaylor_rec/mul.rs (added ctaylor_mul_set_n4 + extended dispatch)"
    - "Cargo.lock (regenerated against new xtask deps)"

key-decisions:
  - "Direct c++ compiler invocation over cc::Build for the standalone C++ executable (cc is oriented toward linking static libs into Rust binaries); keeps the regenerator simple and transparent"
  - "Schema duplicated between xtask::fixtures and crates/xcfun-ad/tests/golden_mul.rs — with a 'keep in sync with' comment — rather than introducing an xtask path-dep into xcfun-ad's dev-dep closure"
  - "Post-serialise sort of mul + expand records (by op, n_var, first-input) before bincode write — defends byte-identity of the committed files against any future change in driver emission order"
  - "Rule 2 auto-add of ctaylor_mul_set_n4 — plan 01-02 explicitly deferred N ∈ 4..=7 to plan 01-05's golden-fixture gate. Without n=4 dispatch, the plan's own success criterion (`rel_err <= 1e-13` at n_var=4) is unreachable"
  - "fixtures.json manifest includes both generated_at (non-stable) and xcfun_version_git_sha (stable) — the drift-detection SHA is over the three taylor .hpp files concatenated in fixed order (ctaylor, ctaylor_math, tmath)"

patterns-established:
  - "Pattern: xtask → fixtures/*.bincode → xcfun-ad/tests/golden_*.rs oracle flow. Plan 01-06 wires golden_expand.rs + golden_composed.rs on top of the same expand.bincode; Phase 2+ reuses the same pattern for functional parity."
  - "Pattern: each golden test duplicates the record schema (keep-in-sync comment + matching #[derive]) and deserialises from `include_bytes!`. Zero runtime file-I/O during `cargo test`."
  - "Pattern: per-N mul flattening extended to n=4 (~110 `let` bindings). Straight-line code mandate (D-08) preserved. Extension to n ∈ 5..=7 is mechanical — same template, ~250 / ~500 / ~1000 bindings respectively."

requirements-completed: [AD-03, AD-05]

# Metrics
duration: 11min
completed: 2026-04-19
---

# Phase 1 Plan 05: Golden-Fixture Tooling + `ctaylor_mul` C++ Parity Summary

**C++-driven `regen-ad-fixtures` xtask binary emits 418 deterministic bincode fixtures (168 `*_expand` + 250 `ctaylor_mul`); `golden_mul.rs` validates `ctaylor_mul` on cubecl-cpu at f64::to_bits identity for n_var ∈ 0..=3 (200/200) and rel-err ≤ 1e-13 for n_var = 4 (50/50).**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-04-19T11:25:47Z
- **Completed:** 2026-04-19T11:36:58Z
- **Tasks:** 3 / 3
- **Files created:** 8 (1 shared schema + 3 fixture artifacts + 1 golden test + 1 .gitkeep + 2 updates to pre-existing stubs)
- **Files modified:** 4 (xtask/Cargo.toml, .gitignore, ctaylor_rec/mul.rs, Cargo.lock)

## Accomplishments

- **xtask regen-ad-fixtures binary is fully wired:** compiles `xtask/assets/regen_ad_fixtures/driver.cpp` via a direct `c++ -std=c++17 -O2 -fno-fast-math -ffp-contract=off` invocation (preserving arithmetic parity — no FMA fusion, no reassociation), runs the driver, parses its text-format stdout into `FixtureRecord`s, partitions by op family, serialises to bincode, and writes a sha256 manifest.
- **C++ driver emits exactly 418 records** (the Plan 05 scope target): 168 `*_expand` records (3 inputs × 7 orders × 8 fns over inv/exp/log/pow/sqrt/cbrt/gauss/erf) + 250 `mul` records (50 mt19937_64 seeds × 5 N values at n_var ∈ {0..=4}). Driver is deterministic: re-running against the same `xcfun-master/` tree produces byte-identical `mul.bincode` and `expand.bincode`.
- **Fixtures committed to Git** at `crates/xcfun-ad/tests/fixtures/{mul.bincode, expand.bincode, fixtures.json}` — 57 KB total, well under the 1 MB budget (D-17). CI never regenerates them (D-19); drift is detected via the `xcfun_version_git_sha` field in `fixtures.json`, which pins a sha256 over the three vendored taylor .hpp files.
- **`golden_mul.rs` integration test is green end-to-end** — all 250 mul records pass (per-n tally: `n=0 50/50, n=1 50/50, n=2 50/50, n=3 50/50, n=4 50/50`). Assertions are f64::to_bits identity for n_var ≤ 3 and rel-err < 1e-13 for n_var = 4.
- **Auto-added `ctaylor_mul_set_n4`** to unblock the n_var=4 fixture gate (Rule 2 deviation — Plan 02 explicitly deferred N ∈ 4..=7 to this plan). Flattened from the 3-call C++ recursion at n=4: `mul_set_n3 + mul_set_n3 + mul_acc_n3`, ~110 explicit `let` bindings preserving C++ left-to-right associativity (D-08).
- **Plan 02 regression guard** (13 `ctaylor_unit.rs` tests + 4 `cubecl_spike.rs` tests) still green; clippy clean at `-D warnings` across the crate.

## Task Commits

Each task was committed atomically via `--no-verify` (parallel executor convention):

1. **Task 1: xtask fixtures module + populated C++ driver + cc build glue** — `3f0d37b` (feat)
2. **Task 2: run regen-ad-fixtures; commit fixtures; smoke-check manifest** — `e86c403` (feat)
3. **Task 3: golden_mul.rs + ctaylor_mul_set_n4 to unblock n_var=4 gate** — `2539502` (feat)

## Files Created/Modified

### Created

- `xtask/src/fixtures.rs` — `FixtureRecord` + `FixturesManifest` with serde derives. Shared between the regen binary (writer) and the xcfun-ad test suite (reader — via schema duplication with a keep-in-sync comment).
- `xtask/assets/regen_ad_fixtures/driver.cpp` — 218-line C++ driver (rewritten from Plan 01 stub). Templates for 8 `*_expand` families + 5 `mul` n_var buckets. Emits semicolon-separated `<op>;<n_var>;<inp_cnt>;<i0,...>;<coeff_cnt>;<c0,...>` records via `printf("%.17g", v)` for round-trip identity.
- `xtask/src/bin/regen_ad_fixtures.rs` — 300-line binary (rewritten from stub). Compiles driver via direct `c++` invocation, parses stdout, partitions + sorts deterministically, serialises via bincode 1.3, writes `fixtures.json` with sha256 over `{ctaylor.hpp, ctaylor_math.hpp, tmath.hpp}` + RFC 3339 timestamp + optional git HEAD.
- `crates/xcfun-ad/tests/fixtures/.gitkeep` — keeps the empty fixtures dir under Git when hypothetically cleared.
- `crates/xcfun-ad/tests/fixtures/mul.bincode` — 44,208 bytes; 250 mul records.
- `crates/xcfun-ad/tests/fixtures/expand.bincode` — 12,860 bytes; 168 expand records.
- `crates/xcfun-ad/tests/fixtures/fixtures.json` — 463-byte manifest with per-op counts, total_records=418, xcfun_version_git_sha=`8ec452fd8d40d11c5ce251f5efcf2c16c2e3605fa5de253edbdb080df613eeee`.
- `crates/xcfun-ad/tests/golden_mul.rs` — 160-line integration test; `#[cube(launch_unchecked)] kernel_mul` adapter + launch helper + per-n pass tracker.

### Modified

- `xtask/Cargo.toml` — added `sha2 = "0.10"` + `chrono = { version = "0.4", default-features = false, features = ["std", "clock"] }`. Both are xtask-local (app-boundary); no workspace or library-crate dep added.
- `.gitignore` — added `xcfun-master` entry so the vendored-symlink build input doesn't show up as untracked in the worktree.
- `crates/xcfun-ad/src/ctaylor_rec/mul.rs` — added `ctaylor_mul_set_n4` (~110 `let`-bindings straight-line code from the 3-call recursion) + extended outer `ctaylor_mul` dispatch from `n ∈ {0..=3}` to `n ∈ {0..=4}`.
- `Cargo.lock` — regenerated against the new xtask deps (sha2, chrono + transitives).

## Decisions Made

- **Direct `c++` compiler invocation over `cc::Build`** for producing the standalone driver executable. `cc` is oriented toward linking static libs into Rust binaries, not C++ executables; spawning `c++` directly (honouring `$CXX` env var) keeps the regenerator transparent and independent of `cc::Build`'s build-script assumptions.
- **Schema duplication rather than xtask path-dep.** `FixtureRecord` is defined in `xtask/src/fixtures.rs` for the writer and re-defined verbatim in `crates/xcfun-ad/tests/golden_mul.rs` for the reader, with a `// Keep in sync with xtask/src/fixtures.rs` comment. Reasoning: the xcfun-ad dev-dep graph stays narrow; the schema is 4 fields; drift would be caught immediately by a failing `bincode::deserialize`.
- **Deterministic post-hoc sort.** Records from the driver are partitioned into `mul_records` + `expand_records` Vec<T>, then sorted by `(op, n_var, inputs)` before bincode serialisation. Defends byte-identity against any future change in driver emission order (e.g., reordering EMIT_EXPAND macro args).
- **Text format for driver stdout (not binary).** Per the plan's "Alternative approach" note — text is simpler for debug (a reviewer can `./regen_ad_driver | head` and read actual numbers). `"%.17g"` round-trips f64 exactly; parser robustness is tested by empty-input-count edge case.
- **Rule 2 auto-add of `ctaylor_mul_set_n4`.** Plan 02's summary explicitly deferred N ∈ 4..=7 to "a follow-on plan with its own golden-fixture gate" — and that follow-on plan is 01-05 (this one). The fixture driver emits n_var=4 records, so the dispatch must handle n=4 or the plan's own success criterion is unreachable.
- **Generated `fixtures.json` timestamp is acceptable churn.** The plan's acceptance criterion for content-hash stability covers `mul.bincode` and `expand.bincode` only; `fixtures.json` carries `generated_at` and optional `driver_commit` which by design change per run. Verified: re-running the regenerator produces identical bincode SHAs (`mul=55c9efd3...`, `expand=14ba3413...`) and only diffs `fixtures.json` on those two fields.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] `xcfun-master/` not present in worktree**

- **Found during:** Task 2 (running `cargo run -p xtask --bin regen-ad-fixtures`).
- **Issue:** The parallel-executor worktree was branched off `c1e957d5` at a point where `xcfun-master/` was untracked in the main checkout. The symlink / directory simply wasn't in the worktree filesystem, so the regen binary correctly errored out with `"xcfun-master taylor headers not found at ..."`.
- **Fix:** Created a symlink `ln -s /home/chemtech/workspace/xcfun_rs/xcfun-master xcfun-master` from the worktree root to the main checkout's vendored copy, and added `xcfun-master` to `.gitignore` so the symlink doesn't pollute Git status. This matches the convention in the main checkout (xcfun-master is never committed — it's a vendored build input with MPL-2.0 licensing handled out-of-band).
- **Files modified:** `.gitignore` (added), worktree filesystem (symlink).
- **Verification:** `ls xcfun-master/external/upstream/taylor/ctaylor.hpp` resolves; `cargo run -p xtask --bin regen-ad-fixtures` succeeds.
- **Committed in:** `e86c403` (Task 2 commit — .gitignore alongside the fixture files).

**2. [Rule 2 — Missing Critical Functionality] Added `ctaylor_mul_set_n4` to match fixture-driver emission**

- **Found during:** Task 3 (first `cargo test --test golden_mul` run).
- **Issue:** Plan 02 scoped `ctaylor_mul` to N ∈ {0..=3} and deferred N ∈ {4..=7} to Plan 05. The fixture driver in Task 1 of this plan emits n_var=4 mul records (50 of them — it has to, per Plan 05's success criterion "rel-err ≤ 1e-13 for n_var = 4"). Running the golden test surfaced the gap: `ctaylor_mul` dispatch had no `n == 4` arm, so the kernel wrote zero to every output element, and the test asserted `rel_err=1.0` against the non-zero C++ reference coefficient. Without closing this gap the plan's own success criterion is unreachable.
- **Fix:** Added `ctaylor_mul_set_n4` as a fully-flattened straight-line body — three concatenated sub-bodies mirroring `mul_set_n3(dst[0..8], x[0..8], y[0..8])`, `mul_set_n3(dst[8..16], x[8..16], y[0..8])`, `mul_acc_n3(dst[8..16], x[0..8], y[8..16])` — ~110 explicit `let` bindings preserving the C++ left-to-right associative operation order per D-08. Extended `ctaylor_mul` dispatch to `n ∈ {0..=4}`; N ∈ 5..=7 still deferred (no fixtures at those N in this plan).
- **Files modified:** `crates/xcfun-ad/src/ctaylor_rec/mul.rs` (additive — no change to existing ctaylor_mul_set_n{0..3} primitives).
- **Verification:** `cargo test -p xcfun-ad --features "cpu testing" --test golden_mul` — 1 passed, 0 failed. Per-n pass rate stderr shows `n=4 50/50`. Plan 02 regression guard (13 ctaylor_unit tests) still green.
- **Committed in:** `2539502` (Task 3 commit).

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical).

**Impact on plan:** Both deviations were mechanical and did not expand scope. The xcfun-master symlink is infrastructure that any worktree agent operating on this plan would hit. The `ctaylor_mul_set_n4` addition is exactly the completion of a Plan-02 deferral that Plan 02's summary explicitly routed to this plan. No architectural change, no user-visible API change, no new dependency at the library-crate layer.

## C++ Driver Build Quirks

- Compiler used: `c++` (GCC 15.2.0-16ubuntu1) — picked up from `/usr/bin/c++`. `CXX` env var honoured if set.
- Flags that matter for parity: `-std=c++17 -O2 -fno-fast-math -ffp-contract=off`. The first two compile the driver; the latter two preserve IEEE-754 semantics end-to-end (no operation reassociation, no FMA fusion).
- One build quirk worth noting: the stub driver from Plan 01 used `ctaylor<double, 1>` with `.c[0]` / `.c[1]` direct access. The plan-05 driver uses the same access pattern plus the multiplication operator overload, all declared in `ctaylor.hpp`. GCC 15.2 compiled cleanly with no template-instantiation warnings.

## Content-Hash Stability (Determinism Evidence)

Direct verification on this host, following the plan's `<output>` mandate:

| File                                           | SHA256 (before re-run)                                             | SHA256 (after re-run)                                              | Identical |
| ---------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | --------- |
| `crates/xcfun-ad/tests/fixtures/mul.bincode`   | `55c9efd3f59eb58ce493bca815f9807a1176bcd542709fa55b23a352903bdf65` | `55c9efd3f59eb58ce493bca815f9807a1176bcd542709fa55b23a352903bdf65` | **yes**   |
| `crates/xcfun-ad/tests/fixtures/expand.bincode`| `14ba34138eeef9aaf9254c6d7cae2439ae79758f398d36f88411bb1c589d9ce3` | `14ba34138eeef9aaf9254c6d7cae2439ae79758f398d36f88411bb1c589d9ce3` | **yes**   |

`fixtures.json` legitimately diffs on `generated_at` + `driver_commit` each run — these are by-design non-stable fields; the drift-detection field `xcfun_version_git_sha` is stable (`8ec452fd8d40d11c5ce251f5efcf2c16c2e3605fa5de253edbdb080df613eeee`).

## Per-N `golden_mul` Pass Rate

From `cargo test -p xcfun-ad --features "cpu testing" --test golden_mul -- --nocapture`:

```
[golden_mul] per-n pass: n=0 50/50, n=1 50/50, n=2 50/50, n=3 50/50, n=4 50/50
```

- n_var ∈ {0, 1, 2, 3}: 200/200 at f64::to_bits identity (AD-05 bit-exact gate).
- n_var = 4: 50/50 at relative error ≤ 1e-13 (libm drift tolerance).

## Fixture Size

- Total `crates/xcfun-ad/tests/fixtures/` directory size: **57,531 bytes** (57 KB). Well under the 1 MB budget from CONTEXT D-17.
- Per file: `mul.bincode` 44,208 bytes, `expand.bincode` 12,860 bytes, `fixtures.json` 463 bytes, `.gitkeep` 0 bytes.

## Record Counts (matching `fixtures.json` `per_op_counts`)

| op             | count |
| -------------- | ----- |
| `mul`          | 250   |
| `inv_expand`   | 21    |
| `exp_expand`   | 21    |
| `log_expand`   | 21    |
| `pow_expand`   | 21    |
| `sqrt_expand`  | 21    |
| `cbrt_expand`  | 21    |
| `gauss_expand` | 21    |
| `erf_expand`   | 21    |
| **Total**      | 418   |

## Known Stubs

None. Every file shipped by this plan is fully implemented for its documented scope. `ctaylor_mul` dispatch now covers N ∈ {0..=4}; N ∈ 5..=7 remain deferred to a future plan when / if the fixture driver extends to those ranges. This is documented in the outer-dispatch doc-comment.

## Issues Encountered

- **Initial Write-tool path confusion.** The Write tool treats absolute paths literally; passing `/home/chemtech/workspace/xcfun_rs/xtask/...` wrote to the main checkout rather than the worktree. Resolved by using worktree-rooted paths (`/home/chemtech/workspace/xcfun_rs/.claude/worktrees/agent-a915624d/...`) for every subsequent file, and restoring the main checkout state via `git checkout --`. No commits landed on the main checkout; the worktree state is self-contained.
- **Symlink-in-gitignore idiosyncrasy.** Initial `.gitignore` entry used `xcfun-master/` (trailing slash) which git rejected with "pathspec beyond a symbolic link"; changed to `xcfun-master` (no slash) so the symlink itself is matched.

## User Setup Required

None — Plan 05 is 100% autonomous given a system C++ compiler (GCC or Clang) on `$PATH`. The `xcfun-master/` vendored source tree must be present at repo root; for worktree agents this is a symlink to the main checkout's untracked copy.

## Next Plan Readiness

**Plan 01-03 (expand)** now has:

- The bincode reader pattern from `golden_mul.rs` — reusable verbatim for `golden_expand.rs` (same deserialize-into-Vec<FixtureRecord>-filter-by-op pattern).
- The committed `expand.bincode` with 168 records covering all 8 `*_expand` families at orders 0..=6 × 3 inputs — ready-made oracle for the `#[cube] fn *_expand` ports once they exist.
- The per-n-pass-rate logging pattern (stderr `[golden_expand] per-order pass: ...`) for SUMMARY.md population.

**Plan 01-06 (composed math + golden_composed.rs)** reuses:

- The same `expand.bincode` — composed C++ functions call the `*_expand` internals, so once the Rust composed functions exist they can be driven against the same fixtures via a `ctaylor_<op>` launcher.

**No blockers.**

---

*Phase: 01-taylor-algebra-ad-primitives-xcfun-ad*
*Plan: 01-05*
*Completed: 2026-04-19*

## Self-Check: PASSED

File presence:

- [x] `xtask/src/fixtures.rs` exists
- [x] `xtask/src/bin/regen_ad_fixtures.rs` exists
- [x] `xtask/assets/regen_ad_fixtures/driver.cpp` exists
- [x] `crates/xcfun-ad/tests/fixtures/.gitkeep` exists
- [x] `crates/xcfun-ad/tests/fixtures/mul.bincode` exists
- [x] `crates/xcfun-ad/tests/fixtures/expand.bincode` exists
- [x] `crates/xcfun-ad/tests/fixtures/fixtures.json` exists
- [x] `crates/xcfun-ad/tests/golden_mul.rs` exists

Commit presence (`git log --oneline --all | grep <hash>`):

- [x] `3f0d37b` — Task 1 (populated driver + shared schema + xtask deps)
- [x] `e86c403` — Task 2 (committed fixtures + .gitignore xcfun-master)
- [x] `2539502` — Task 3 (golden_mul.rs + ctaylor_mul_set_n4 + extended dispatch)

Test runs:

- [x] `cargo test -p xcfun-ad --features "cpu testing" --test golden_mul` → 1 passed
- [x] `cargo test -p xcfun-ad --features "cpu testing" --test ctaylor_unit` → 13 passed (regression green)
- [x] `cargo test -p xcfun-ad --features "cpu testing" --test cubecl_spike` → 4 passed (regression green)

Build:

- [x] `cargo check -p xtask --all-targets` → exits 0
- [x] `cargo clippy -p xcfun-ad --features "cpu testing" --all-targets -- -D warnings` → exits 0
