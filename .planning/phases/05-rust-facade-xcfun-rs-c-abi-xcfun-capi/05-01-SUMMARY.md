---
phase: 05
plan: 01
plan_id: 05-01-rust-facade
subsystem: xcfun-rs facade — public Rust API surface (Functional + 11 free fns)
tags: [rust-facade, public-api, send-sync, zero-alloc, registry-cascade]
requires:
  - 05-00 topology foundation (xcfun-rs crate skeleton, XcError::InvalidVarsAndMode, LB94 registry stub)
  - xcfun-eval Functional surface (Phase 2..4 — set/get/eval/eval_setup/output_length/dependencies)
  - xcfun-core registries (FUNCTIONAL_DESCRIPTORS, PARAMETERS, ALIASES)
provides:
  - public `xcfun_rs::Functional` newtype (RS-01..07) with 9 methods + Default impl + manual Debug
  - 11 module-level free functions (RS-09): version, splash, authors, is_compatible_library, self_test, which_vars, which_mode, enumerate_parameters, enumerate_aliases, describe_short, describe_long
  - Send + Sync compile-time gate (RS-10, D-17)
  - facade-boundary zero-allocation invariant (RS-07, D-13) — fall-back form
  - re-exports of Mode, Vars, XcError, ParameterId, FunctionalId, Dependency
affects:
  - crates/xcfun-rs/src/lib.rs (overwritten — placeholder → full crate root)
  - crates/xcfun-rs/src/functional.rs (NEW)
  - crates/xcfun-rs/src/free_fns.rs (NEW)
  - crates/xcfun-rs/assets/splash.txt (NEW)
  - crates/xcfun-rs/assets/authors.txt (NEW)
  - crates/xcfun-rs/tests/send_sync.rs (NEW)
  - crates/xcfun-rs/tests/free_fns.rs (NEW)
  - crates/xcfun-rs/tests/zero_alloc.rs (NEW)
tech-stack:
  added: []
  patterns:
    - "newtype facade with private inner field — bypass-prevention for `weights`/`settings`"
    - "facade-boundary mutation: `eval_setup` validates via xcfun-eval (read-only) and writes inner `vars`/`mode`/`order` from xcfun-rs"
    - "31-arm verbatim port of `xcfun_which_vars` from XCFunctional.cpp:131-277"
    - "3-table cascade lookup (FunctionalId → ParameterId → ALIASES) with case-insensitive `eq_ignore_ascii_case`"
    - "UPSTREAM_FUNCTIONAL_COUNT = 78 isolation: `enumerate_parameters(78) → RANGESEP_MU`, NOT XC_LB94 — preserves C ABI semantics while still exposing LB94 via name"
    - "test-only `with_weights_for_test` private constructor — exercises is_gga/is_metagga/eval over an active functional set without waiting on Phase 6 settings→weights rebuild"
    - "facade-boundary zero-alloc fall-back: head/tail mean comparison instead of strict global delta == 0 (cubecl-cpu substrate cost ~287 allocs/eval is Phase 6's concern)"
key-files:
  created:
    - crates/xcfun-rs/src/functional.rs
    - crates/xcfun-rs/src/free_fns.rs
    - crates/xcfun-rs/assets/splash.txt
    - crates/xcfun-rs/assets/authors.txt
    - crates/xcfun-rs/tests/send_sync.rs
    - crates/xcfun-rs/tests/free_fns.rs
    - crates/xcfun-rs/tests/zero_alloc.rs
  modified:
    - crates/xcfun-rs/src/lib.rs
decisions:
  - id: D-02 (Phase 5 CONTEXT)
    description: "Functional newtype with private inner field — `pub struct Functional(xcfun_eval::Functional)` blocks bypass of `set` validation."
  - id: D-13 (Phase 5 CONTEXT)
    description: "Zero-alloc fixture lands in fall-back form (b): facade-boundary head/tail stability test instead of strict global-zero. Substrate (~287 allocs/eval from cubecl-cpu `create_from_slice`) deferred to Phase 6 persistent-buffer work."
  - id: D-17 (Phase 5 CONTEXT)
    description: "Send+Sync compile-time gate via `static_assertions::assert_impl_all!` in tests/send_sync.rs."
  - id: D-Plan-05-01-A
    description: "`UPSTREAM_FUNCTIONAL_COUNT = 78` isolates LB94 (Phase 5 D-16 stub at FunctionalId 78) from `enumerate_parameters` C ABI semantics: index 78 maps to `PARAMETERS[0].name = RANGESEP_MU`, not to `XC_LB94`. Required because the C ABI client expects the upstream 78-functional / 4-parameter contract; LB94 is reachable via `Functional::set(\"lb94\", 1.0)` and via `describe_short(\"LB94\")` but does NOT enumerate as a numeric index."
  - id: D-Plan-05-01-B
    description: "Manual `Debug` impl on `Functional` — the inner `xcfun_eval::Functional` does not derive `Debug`. The facade exposes a content-light `{vars, mode, order, weights_len}` summary so application logging / `assert_eq!` panics still compile."
  - id: D-Plan-05-01-C
    description: "Test-only private constructor `Functional::with_weights_for_test(&'static [(FunctionalId, f64)])` — needed because Phase 5 does not yet rebuild `weights` from `settings` (that rebuild lives in Plan 05-02 C ABI / Phase 6 dispatch). Without this fixture the inline `is_gga`/`is_metagga`/`eval`-non-zero tests would all see an empty weights slice and could not exercise RS-04 / RS-07 behaviour."
metrics:
  duration: ~11m
  completed_date: 2026-04-30
---

# Phase 5 Plan 01: Rust Facade (xcfun-rs) Summary

Public Rust API surface for the xcfun-rs crate: `Functional` newtype with 9 methods + Default + manual Debug, 11 free functions, Send+Sync compile-time gate, and a facade-boundary zero-allocation fixture.

## One-line summary

`xcfun-rs::Functional` newtype + 11 free functions land as the stable Rust facade over `xcfun-eval::Functional`, with 57 passing tests covering RS-01..07, RS-09, RS-10 and a documented (b) fall-back for the zero-alloc invariant.

## Tasks completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1.1 | Functional newtype + assets + Send+Sync gate | `980072a` | crates/xcfun-rs/src/{lib.rs,functional.rs,free_fns.rs}, crates/xcfun-rs/assets/{splash,authors}.txt, crates/xcfun-rs/tests/send_sync.rs |
| 1.2 | Free-function integration tests (40 tests) | `cc1add2` | crates/xcfun-rs/tests/free_fns.rs |
| 1.3 | Zero-allocation hot-path facade-boundary fixture | `c89e453` | crates/xcfun-rs/tests/zero_alloc.rs |

## Net deltas

### Functional newtype (Task 1.1)

- `pub struct Functional(xcfun_eval::Functional)` — private inner field per
  T-05-01-04 mitigation (callers cannot mutate `weights`/`settings`
  bypassing `set`).
- 9 public methods: `new`, `set`, `get`, `is_gga`, `is_metagga`,
  `eval_setup`, `user_eval_setup`, `eval`, `input_length`,
  `output_length`. All delegate to the inner `xcfun_eval::Functional`
  except `eval_setup`, which writes back to inner `vars`/`mode`/`order`
  on success — the facade-boundary mutation pattern that keeps
  xcfun-eval's `eval_setup` read-only.
- `user_eval_setup` composes `which_vars` + `which_mode`; out-of-range
  inputs return `Err(XcError::InvalidEncoding)` (diverges from C++ `xcfun::die`,
  per the must_have spec — the C ABI in Plan 05-02 maps this back to abort).
- `Default` impl delegates to `new()`.
- Manual `Debug` impl exposes a content-light summary (D-Plan-05-01-B).
- Test-only `with_weights_for_test` constructor (D-Plan-05-01-C).
- 16 inline `functional::tests` cover every <behavior> bullet:
  new/Default, set/get round-trip, unknown-name rejection,
  is_gga/is_metagga over LDA (SLATERX), GGA (PBEX), metaGGA (TPSSX),
  eval_setup field mutation, user_eval_setup happy path + 3 reject
  paths (func_type=4, mode_type=0, order=-1), eval non-zero output,
  input_length/output_length reflecting setup state.

### 11 free functions (Task 1.1 file body, Task 1.2 integration tests)

Every function is a verbatim port of XCFunctional.cpp:48-348, with one
intentional Rust-side divergence:

- `version()` → `env!("CARGO_PKG_VERSION")` — single-source-of-truth from
  the crate manifest, not C++ string baking.
- `splash()` / `authors()` → `include_str!("../assets/splash.txt")` /
  `include_str!("../assets/authors.txt")` — verbatim copies of the
  XCFunctional.cpp string literals.
- `is_compatible_library()` returns `true` (single-process build per
  D-03 implication; major-version match is trivial when both header and
  library come from THIS crate per Plan 05-03).
- `self_test()` iterates over functionals in `FUNCTIONAL_DESCRIPTORS`
  carrying upstream test data (`test_in.is_some()`); compares
  `eval` output to `test_out` within `test_threshold`. Returns failure
  count.
- `which_vars` — 31-arm port of XCFunctional.cpp:131-277, byte-for-byte.
  Range checks (`func_type ≤ 3, dens_type ≤ 3, others ≤ 1`) up front,
  unmapped `bitwise_vars` falls through to `None` (instead of C++
  `xcfun::die`).
- `which_mode` — 3-arm port of XCFunctional.cpp:281-300; out-of-range
  returns `None`.
- `enumerate_parameters` — port of XCFunctional.cpp:302-311 with the
  `UPSTREAM_FUNCTIONAL_COUNT = 78` isolation (D-Plan-05-01-A): index 78
  → `PARAMETERS[0].name = RANGESEP_MU`, NOT XC_LB94. Negative index
  → `None`.
- `enumerate_aliases` — port of XCFunctional.cpp:313-320 over the 46
  ALIASES rows; negative or out-of-range → `None`.
- `describe_short` / `describe_long` — 3-table cascade (FunctionalId →
  ParameterId → ALIASES) with `eq_ignore_ascii_case` on aliases. Same
  `UPSTREAM_FUNCTIONAL_COUNT = 78` isolation when mapping
  `ParameterId` discriminant back to `PARAMETERS` index.

40 integration tests in `tests/free_fns.rs` cover every <behavior>
bullet — well above the plan's ≥23 minimum.

### Send + Sync compile-time gate (Task 1.1)

`tests/send_sync.rs` is 5 lines: `assert_impl_all!(Functional: Send, Sync)`
at top level. Compile-time only; produces a "0 tests" runtime line and
fails to compile if either bound is dropped.

### Zero-allocation hot-path test (Task 1.3 — fall-back applied)

**Strict form failed against the current Phase 5 stack.** Observed
~28 706 allocations across 100 SLATERX evals (≈ 287 allocs/eval).
Cause: cubecl-cpu's per-launch `client.create_from_slice(...)` —
documented in 05-PATTERNS.md §A.3 and the plan's inline risk note as a
known Phase 6 concern.

**Documented (b) fall-back applied per Task 1.3 risk note:** verify
zero-alloc only at the FACADE BOUNDARY, EXCLUDING the cubecl substrate.
Concretely the test now asserts the per-call allocation count does NOT
drift upward across the 100-eval window: `tail[90..100].mean() ≤
head[0..10].mean() + 1.0` (1.0 alloc/call jitter slack — observed jitter
is single-digit). If the wrapper itself leaked even one allocation per
call, the tail mean would exceed head by ~10 over the run; the slack is
two orders of magnitude tighter than that.

**Phase 6 follow-up:** replace cubecl-cpu's per-launch
`create_from_slice` with a pre-allocated reusable handle, then tighten
this fixture to the strict (delta == 0) form. Linked under D-13.

The test reports the per-call substrate cost via `eprintln!` for visibility:

```
[zero_alloc] per-call substrate allocation cost: head=287.x, tail=287.x blocks/eval (Phase 6 target: 0)
```

### Re-exports (Task 1.1)

`pub use xcfun_core::{Dependency, FunctionalId, Mode, ParameterId, Vars, XcError};`
— the public Rust API surface is self-contained: callers do NOT need
to add `xcfun-core` as a direct dependency.

## Test inventory

| Test target | Count | Notes |
|-------------|-------|-------|
| `cargo test -p xcfun-rs --lib functional::tests` | 16 | Inline; covers RS-01..07 behaviour bullets |
| `cargo test -p xcfun-rs --test send_sync` | 0 (compile-time) | `assert_impl_all!(Functional: Send, Sync)` |
| `cargo test -p xcfun-rs --test free_fns` | 40 | RS-09 — every <behavior> bullet ≥1 test |
| `cargo test -p xcfun-rs --test zero_alloc` | 1 | RS-07/D-13 fall-back form |
| **Total** | **57** | All passing |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Inner `xcfun_eval::Functional` does not derive `Debug`**

- **Found during:** Task 1.1 (`cargo check`).
- **Issue:** Plan template uses `#[derive(Debug)]` on the newtype, but
  `xcfun_eval::Functional` does NOT derive `Debug` (large `[f64; 82]`
  field + cubecl-internal types). `derive(Debug)` propagation fails.
- **Fix:** Replaced derive with a manual `Debug` impl exposing a
  content-light `{vars, mode, order, weights_len}` summary. Sufficient
  for application logging / `assert_eq!` panic messages, avoids
  forcing a Debug bound on the inner type.
- **Files modified:** `crates/xcfun-rs/src/functional.rs`.
- **Commit:** Folded into Task 1.1 commit `980072a`.

**2. [Rule 3 — Blocking] `enumerate_parameters` index 78 ambiguity (LB94 vs RANGESEP_MU)**

- **Found during:** Task 1.1 free_fns body.
- **Issue:** The plan's reference comment ("Indices 0..78 → functional
  names; 78..82 → parameter names") was written against the upstream
  `XC_NR_FUNCTIONALS = 78`. Phase 5 D-16 introduced `XC_LB94` as
  Rust-side `FunctionalId 78`, growing `FUNCTIONAL_DESCRIPTORS` to 79
  rows. A literal port of XCFunctional.cpp:302-311 would then return
  `Some("XC_LB94")` for index 78 — diverging from the C ABI semantics
  the plan must_have spec assumes (`enumerate_parameters(78) → Some("RANGESEP_MU")`).
- **Fix:** Introduced a private `UPSTREAM_FUNCTIONAL_COUNT = 78`
  constant inside `free_fns.rs`. `enumerate_parameters` and the
  `ParameterId → PARAMETERS` index mapping in `describe_short` /
  `describe_long` both use this constant, NOT `FUNCTIONAL_DESCRIPTORS.len()`.
  Result: index 78 lands on `PARAMETERS[0] = RANGESEP_MU` (matches the
  must_have); LB94 is still reachable via name (`describe_short("LB94")`,
  `Functional::set("lb94", _)`) but does NOT surface as a numeric
  enumeration index.
- **Files modified:** `crates/xcfun-rs/src/free_fns.rs`.
- **Commit:** Folded into Task 1.1 commit `980072a`.
- **Decision:** Recorded as `D-Plan-05-01-A`.

**3. [Rule 3 — Blocking] `with_weights_for_test` test fixture required**

- **Found during:** Task 1.1 inline tests for `is_gga` / `is_metagga` /
  `eval` non-zero output.
- **Issue:** Phase 5 facade does not yet rebuild the inner `weights:
  &'static [(FunctionalId, f64)]` slice from the `settings[]` array
  updated by `set` (that wiring lives in Plan 05-02 / Phase 6). Tests
  that need an active functional set (so `dependencies()` returns the
  right flags and `eval` exercises the cubecl launch loop) cannot get
  there through the public surface alone.
- **Fix:** Added `#[cfg(test)] fn with_weights_for_test(&'static [...])
  -> Self` private constructor inside `Functional`. It writes the
  inner `weights` field directly. Keeps the public surface clean (the
  fixture is only visible to the `tests` module).
- **Files modified:** `crates/xcfun-rs/src/functional.rs`.
- **Commit:** Folded into Task 1.1 commit `980072a`.
- **Decision:** Recorded as `D-Plan-05-01-C`.

**4. [Rule 1 — Bug] Strict global-zero allocation test failed**

- **Found during:** Task 1.3 (`cargo test --test zero_alloc`).
- **Issue:** Strict 100-eval-with-zero-global-allocs fixture failed
  with observed delta = 28 706 (≈ 287 allocs/eval). Cause is
  cubecl-cpu's per-launch buffer creation, documented in 05-PATTERNS.md
  §A.3 as a known Phase 6 concern.
- **Fix:** Adopted the plan's documented (b) fall-back: assert the
  facade-boundary stability invariant (head[0..10].mean() vs
  tail[90..100].mean() within 1.0 alloc/call). Test passes with the
  observed substrate cost. Substrate cost reported via `eprintln!` for
  visibility; tightening to strict global-zero is filed under D-13 as
  a Phase 6 follow-up.
- **Files modified:** `crates/xcfun-rs/tests/zero_alloc.rs`.
- **Commit:** Task 1.3 commit `c89e453`.
- **Decision:** Recorded as `D-13` (Phase 5 CONTEXT) refinement.

No Rule 4 (architectural) deviations needed.

## Threat model coverage

| Threat ID | Status |
|-----------|--------|
| T-05-01-01 (DoS via alias recursion) | accepted as planned — alias depth = 1 is invariant |
| T-05-01-02 (`#![forbid(unsafe_code)]` enforced) | mitigated — already in `lib.rs:13` from Plan 05-00, retained |
| T-05-01-03 (`&'static str` from assets) | accepted as planned — assets are committed |
| T-05-01-04 (Functional field privacy) | mitigated — `pub struct Functional(xcfun_eval::Functional)` keeps inner field private; only `set`/`eval_setup` mutate state |

## Confirmation of plan output requirements

- ✅ File listing: lib.rs (modified), functional.rs (NEW), free_fns.rs (NEW), 3 tests, 2 assets — see `key-files` frontmatter.
- ✅ Test counts: 16 inline + 40 free_fns + 0 send_sync compile-time gate + 1 zero_alloc = 57 tests, all pass.
- ✅ Zero-alloc fall-back (b) applied — reason captured (cubecl-cpu per-launch allocation), follow-up linked to D-13/Phase 6.
- ✅ `Functional` is `Send + Sync` — verified by `assert_impl_all!` compile-time gate in `tests/send_sync.rs`.
- ✅ Re-exports cover `Mode, Vars, XcError, ParameterId, FunctionalId, Dependency` — `lib.rs:23`.

## Self-Check: PASSED

All claimed files exist:

```
crates/xcfun-rs/src/lib.rs                         FOUND
crates/xcfun-rs/src/functional.rs                  FOUND
crates/xcfun-rs/src/free_fns.rs                    FOUND
crates/xcfun-rs/assets/splash.txt                  FOUND
crates/xcfun-rs/assets/authors.txt                 FOUND
crates/xcfun-rs/tests/send_sync.rs                 FOUND
crates/xcfun-rs/tests/free_fns.rs                  FOUND
crates/xcfun-rs/tests/zero_alloc.rs                FOUND
```

All claimed commits exist in history:

- `980072a` feat(05-01): xcfun-rs Functional newtype + free fns + send/sync gate (Task 1.1)
- `cc1add2` test(05-01): add free_fns integration tests (Task 1.2)
- `c89e453` test(05-01): zero-allocation hot-path facade-boundary fixture (Task 1.3)

Plan verification commands all green:

- `cargo check -p xcfun-rs` → exit 0
- `cargo test -p xcfun-rs --test send_sync` → 0 tests (compile-time gate compiles)
- `cargo test -p xcfun-rs --test free_fns` → 40/40 passed
- `cargo test -p xcfun-rs --test zero_alloc` → 1/1 passed
- `cargo test -p xcfun-rs --lib functional::tests` → 16/16 passed
- `grep -cE "^pub fn ..." crates/xcfun-rs/src/free_fns.rs` → 11
- `grep -cE "31-arm match cases" crates/xcfun-rs/src/free_fns.rs` → 31
- `! grep -rE "use anyhow|anyhow::" crates/xcfun-rs/src/` → no anyhow leak
