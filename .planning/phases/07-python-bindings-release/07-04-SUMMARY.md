---
phase: 07-python-bindings-release
plan: 04
subsystem: python-bindings

tags: [pyo3, abi3, intenum, functional, python, numpy, dft, xc]

# Dependency graph
requires:
  - phase: 07-python-bindings-release
    provides: "module skeleton + 11 free fns (07-02), XcfunError + xc_to_py mapping (07-03)"
provides:
  - "#[pyclass] Functional with eager-or-deferred construction (D-05) and 11 method delegates (set, get, is_gga, is_metagga, eval_setup, user_eval_setup, input_length, output_length, eval, eval_vec, configure)"
  - "#[pyclass(eq, eq_int, from_py_object)] Mode and Vars IntEnum mirrors with byte-matched discriminants vs xcfun-core"
  - "Re-exports (Functional, Mode, Vars) wired into python/xcfun_rs/__init__.py + .pyi"
  - "numpy_io.rs stub raising NotImplementedError for eval_vec — Plan 07-05 fills the body"
  - "pytest test_functional.py — 15 tests covering construction, configure, set/get, eval, IntEnum runtime values, error paths"
affects: [07-05-vectorised-numpy, 07-06-validation-and-wheels]

# Tech tracking
tech-stack:
  added:
    - "pyo3 0.28 from_py_object opt-in attribute (resolves automatic-FromPyObject deprecation for Clone-derived pyclass enums)"
  patterns:
    - "IntEnum mirror with #[pyclass(eq, eq_int, from_py_object)] + From<PyEnum> for RsEnum exhaustive match (T-7-04-03 mitigation)"
    - "const _: () = { assert!(...); } compile-time discriminant guard between Python enum and xcfun-core enum (T-7-04-01 mitigation)"
    - "Eager #[new] constructor with #[pyo3(signature = (name, *, vars=None, mode=None, order=None))] enabling both eager and deferred-configure styles (D-05)"
    - "Result<T, XcError> -> PyResult<T> via .map_err(crate::errors::xc_to_py) consistently across every fallible method (D-09 / D-12)"
    - "Stub-then-fill module pattern: numpy_io.rs ships a NotImplementedError stub in 07-04 so the surface compiles + types resolve, then 07-05 swaps the body without changing call sites"

key-files:
  created:
    - "crates/xcfun-py/src/numpy_io.rs"
    - "crates/xcfun-py/tests/test_functional.py"
  modified:
    - "crates/xcfun-py/src/functional.rs"
    - "crates/xcfun-py/src/lib.rs"
    - "crates/xcfun-py/python/xcfun_rs/__init__.py"
    - "crates/xcfun-py/python/xcfun_rs/__init__.pyi"

key-decisions:
  - "Added from_py_object opt-in to both #[pyclass] enums to migrate ahead of pyo3 0.28's deprecation of automatic FromPyObject for Clone-derived pyclass; the constructor's Option<Vars>/Option<Mode> kwargs need FromPyObject."
  - "Kept Backend / auto_backend out of the Python surface (D-04 lock); auto_backend remains a Rust-side concern reachable from Python only indirectly via eval_vec."
  - "eval releases the GIL via py.detach() (pyo3 0.28 rename of allow_threads); eval_vec dispatches into numpy_io::eval_vec_impl now to keep call site stable when 07-05 fills the body."
  - "Stub eval_vec raises NotImplementedError rather than silently returning an empty array — pytest can detect Plan 07-05 readiness by asserting it raises."

patterns-established:
  - "IntEnum mirror pattern: #[pyclass(eq, eq_int, from_py_object)] + exhaustive From + const _: () = { assert!(...) } guard."
  - "Constructor-time eval_setup pattern: kw-only optional triple (vars, mode, order); when all three are Some, run set + eval_setup atomically; when None, leave for configure()."

requirements-completed: [PY-02]

# Metrics
duration: 11min
completed: 2026-05-08
---

# Phase 07 Plan 04: PY-02 Functional pyclass + Mode/Vars IntEnum mirrors

**Python `Functional` class wrapping `xcfun_rs::Functional` 1:1 with eager-or-deferred construction, byte-matched Mode/Vars IntEnums, and pytest coverage of every method.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-05-08T06:43:16Z
- **Completed:** 2026-05-08T06:53:44Z
- **Tasks:** 3
- **Files modified:** 6 (2 created, 4 edited)

## Accomplishments
- `#[pyclass] Functional` with all 11 PY-02 methods reachable from Python, every fallible method routing `XcError` through `crate::errors::xc_to_py`.
- `Mode` (4 variants) and `Vars` (31 variants) `#[pyclass(eq, eq_int, from_py_object)]` IntEnum mirrors. Discriminants byte-matched to `xcfun-core::Mode` / `xcfun-core::Vars` and guarded by `const _: () = { assert!(...) };`.
- D-05 eager constructor: `Functional("pbe", vars=Vars.A_B_GAA_GAB_GBB, mode=Mode.PartialDerivatives, order=2)` runs `set` + `eval_setup` atomically; `Functional("pbe")` leaves the instance in a constructed-but-unconfigured state with `configure()` as the escape hatch.
- D-12 fail-fast: invalid `(vars, mode, order)` combos raise `XcfunError` from the constructor body, not later from `eval`.
- D-06 `set()` returns `None` (Pythonic in-place mutator); the alias-additive composition path in xcfun-rs is preserved end-to-end.
- `numpy_io.rs` stub provides a clean call site for `Functional.eval_vec`; Plan 07-05 swaps in the strict-zero-copy body without changing public surface.
- Type stubs in `__init__.pyi` cover the full Functional + Mode + Vars surface for PEP 561.

## Task Commits

Each task was committed atomically:

1. **Task 4.1: Mode + Vars IntEnums (#[pyclass(eq, eq_int)])** — `86d7005` (feat)
2. **Task 4.2: #[pyclass] Functional + 9 method delegates + #[new] eager constructor** — `6fa0ad1` (feat)
3. **Task 4.3: pytest test_functional.py — PY-02 method coverage** — `f368e3b` (test)

## Files Created/Modified

- `crates/xcfun-py/src/functional.rs` — Added `Mode` + `Vars` IntEnum types with `From` impls, then the `Functional` `#[pyclass]` with all 11 methods (PY-02).
- `crates/xcfun-py/src/lib.rs` — Wired `mod numpy_io;` and registered `Functional` / `Mode` / `Vars` via `m.add_class::<...>` in the `_native` `#[pymodule]` body. Removed Plan 07-02 placeholder comments.
- `crates/xcfun-py/src/numpy_io.rs` — New stub module exporting `eval_vec_impl(...) -> PyResult<Bound<'py, PyArray2<f64>>>` that returns `NotImplementedError`. Plan 07-05 will replace the body with the strict-zero-copy implementation.
- `crates/xcfun-py/python/xcfun_rs/__init__.py` — Added `Functional, Mode, Vars` to the `from ._native import (...)` block and `__all__`.
- `crates/xcfun-py/python/xcfun_rs/__init__.pyi` — Added Mode + Vars + Functional type stubs (31 Vars variants, 12 Functional methods including `__init__`).
- `crates/xcfun-py/tests/test_functional.py` — New pytest module with 15 tests covering construction, configure escape hatch, set/get, alias additive smoke, is_gga/is_metagga, IntEnum runtime values, eval (per-point), and input_length/output_length.

## Decisions Made

- **`from_py_object` opt-in on both IntEnums (deviation from plan's exact attr list).** PyO3 0.28.3 emits a deprecation warning when a `#[pyclass]` Clone-derived enum implicitly gets `FromPyObject`. Task 4.2's constructor accepts `vars: Option<Vars>` and `mode: Option<Mode>`, which require `FromPyObject`. Plan text used the older `#[pyclass(eq, eq_int, name = "...")]` form; we added `from_py_object` to opt in to the future-stable derive. Behavior is identical at runtime; only the deprecation warning is silenced.
- **Stub eval_vec body deliberately raises NotImplementedError**, not a panic — this keeps the import-time contract clean and lets Plan 07-05's pytest detect readiness by asserting `eval_vec` no longer raises.
- **Static AC verification for Task 4.3 pytest** because maturin/pytest are not installed in this worktree. The plan ACs are static (file exists, line count, ast.parse, grep); runtime test execution is gated by 07-VALIDATION CI.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Compatibility / Bug Avoidance] Added `from_py_object` to `#[pyclass]` IntEnum attributes**
- **Found during:** Task 4.1 (initial build emitted two `pyo3::impl_::deprecated::HasAutomaticFromPyObject` deprecation warnings).
- **Issue:** Plan specified `#[pyclass(eq, eq_int, name = "Mode", module = "xcfun_rs")]` and same for `Vars`. PyO3 0.28.3 deprecated the automatic `FromPyObject` derive for `#[pyclass]` enums that derive `Clone`; without `from_py_object` opt-in, every build emits a warning, and Task 4.2's constructor would inherit the deprecated derive.
- **Fix:** Added `from_py_object` to both `#[pyclass]` attributes. Behavior is identical at runtime; only the deprecation warning is silenced.
- **Files modified:** `crates/xcfun-py/src/functional.rs` (Mode + Vars `#[pyclass]` lines).
- **Verification:** Final `cargo build -p xcfun-py --release --no-default-features --features cpu` emits zero deprecation warnings on the xcfun-py crate (only legacy xcfun-kernels warnings remain, out of scope).
- **Committed in:** `86d7005` (Task 4.1 commit).

**2. [Documentation] Plan AC line-count mismatch — informational, no fix needed**
- **Found during:** Task 4.1 acceptance-criteria self-check.
- **Issue:** AC requires `grep -cE '^\s+[A-Z][A-Z_0-9]*\s*=\s*[0-9]+,$' >= 35`. The regex matches all-caps SCREAMING_SNAKE_CASE variant names but Mode variants are mixed-case (`Unset`, `PartialDerivatives`, `Potential`, `Contracted`) and don't match. Only the 31 SCREAMING_SNAKE_CASE Vars variants match. The intent (35 = 4 Mode + 31 Vars) and regex (all-caps only) are inconsistent in the plan text.
- **Disposition:** Did not modify variant names — they intentionally mirror `xcfun-core::Mode` (mixed-case) and `xcfun-core::Vars` (SCREAMING_SNAKE_CASE) byte-for-byte. The 31-line count from Vars satisfies the spirit of the AC (every Vars discriminant declared). Captured here as a follow-up note for plan-text correction; no code change.
- **Files modified:** None.
- **Verification:** All other ACs pass; the Vars-discriminant count (31) is correct per `crates/xcfun-core/src/enums.rs`.

---

**Total deviations:** 1 auto-fixed (1 compatibility/deprecation) + 1 documentation note.
**Impact on plan:** Auto-fix is forward-compatibility only; no behavioral change. No scope creep.

## Issues Encountered

- None blocking. The single pyo3 0.28.3 deprecation warning was anticipated by adding `from_py_object` (see deviation #1).

## Self-Check

Verifying claims in this SUMMARY against repo state:

**Files exist:**
- `crates/xcfun-py/src/functional.rs` — FOUND
- `crates/xcfun-py/src/lib.rs` — FOUND
- `crates/xcfun-py/src/numpy_io.rs` — FOUND
- `crates/xcfun-py/python/xcfun_rs/__init__.py` — FOUND
- `crates/xcfun-py/python/xcfun_rs/__init__.pyi` — FOUND
- `crates/xcfun-py/tests/test_functional.py` — FOUND

**Commits exist:**
- `86d7005` (Task 4.1) — FOUND
- `6fa0ad1` (Task 4.2) — FOUND
- `f368e3b` (Task 4.3) — FOUND

## Self-Check: PASSED

## User Setup Required

None — no external service configuration required for this plan.

## Next Phase Readiness

- **Plan 07-05 (PY-03 vectorised NumPy `eval_vec`):** Ready. The call site `crate::numpy_io::eval_vec_impl(py, &self.inner, densities)` is wired and stable; Plan 07-05 only needs to replace the body of `eval_vec_impl` in `crates/xcfun-py/src/numpy_io.rs`. No surface changes in `functional.rs` or `lib.rs` should be required.
- **Plan 07-06 (validation + wheels):** The Python `Functional`/`Mode`/`Vars`/`XcfunError` surface is complete enough for parity tests against the Rust facade once `eval_vec` lands.

---
*Phase: 07-python-bindings-release*
*Completed: 2026-05-08*
