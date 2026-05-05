# Phase 7: Python Bindings (`xcfun-py`) + Release — Research

**Researched:** 2026-05-05
**Domain:** PyO3 0.28 + rust-numpy 0.28 extension module + multi-crate workspace publish + GitHub Actions wheel matrix + GitHub Release artifact pipeline
**Confidence:** HIGH (all critical questions resolved against current upstream docs / crates.io / real-world workflows; one CRITICAL FINDING that mutates D-09 surfaced and documented below)

## Summary

Phase 7 stacks two deliverables atop the already-shipped Phase 5 (`xcfun-rs::Functional` + 11 free fns) and Phase 6 (`eval_vec` GPU dispatch + 4 Phase-6 HUMAN-UAT items pending): a **`xcfun-py` PyO3 0.28.3 extension module** producing a CPU-only `abi3-py310` wheel for {Linux x86_64, macOS arm64, Windows x86_64}, and a **v0.1.0 release ceremony** publishing to crates.io / PyPI / GitHub Releases via OIDC trusted publishers and a new `xtask release-publish` topological driver.

The technical approach is straightforward — every layer is a thin shim over an already-validated Rust surface. The four notable risks: (1) a **CRITICAL abi3-py310 vs `#[pyclass(extends=PyException)]` collision** that forces a workaround for D-09's `.code` / `.kind` exception attributes; (2) the cubecl-cpu per-launch allocation cost from Phase 6 D-12 propagates to Python `eval_vec` at small batch sizes; (3) the `xtask release-publish` driver must idempotently handle index propagation between dependent `cargo publish` calls; (4) the four blocking HUMAN-UAT items must clear BEFORE the v0.1.0 tag — they are not parallel work to the Python bindings, they are sequenced before the release ceremony.

**Primary recommendation:** Plan as five waves — (W0) UAT clearance + workspace promotion of `xcfun-py`, (W1) PyO3 module skeleton + `Functional` `#[pyclass]` + 11 free fns, (W2) NumPy strict zero-copy `eval_vec` + `XcfunError` (with the abi3 workaround), (W3) wheel matrix CI + pytest harness against Rust-driver JSON fixtures, (W4) `xtask release-publish` + GitHub Actions release workflow + CHANGELOG.md + tag.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Crate / package / module naming**
- **D-01:** Rename `crates/xcfun-python/` → `crates/xcfun-py/`. `git mv` + update `package.name = "xcfun-py"` + add to workspace `members` (currently in `exclude`).
- **D-02:** PyPI distribution name `xcfun_rs` (underscore). Python module / import name is also `xcfun_rs`.

**GPU-feature exposure**
- **D-03:** CPU-only default PyPI wheel. maturin build flags: `--no-default-features --features cpu`. GPU rebuild via `maturin build --release --features hip|cuda|wgpu` documented in README.
- **D-04:** `xcfun-gpu::Backend` enum is HIDDEN from the Python surface. `eval_vec` does not accept a `backend=` kwarg. The `XCFUN_FORCE_BACKEND` env-var override is still honored — documented in README, not in a Python kwarg.

**Python API shape**
- **D-05:** Eager Python constructor. `Functional('pbe', vars=Vars.A_B, mode=Mode.PartialDerivatives, order=2)` performs `eval_setup` at construction time. `Functional('pbe').configure(vars=..., mode=..., order=...)` is the low-level escape hatch. `Functional("pbe")` alone (no kwargs) is also legal.
- **D-06:** `set()` mutates in place, returns `None`. `f.set('exx', 0.25)`. Aliases compose additively via repeated `set` calls.
- **D-07:** NumPy strict zero-copy or raise. `eval_vec(densities)` accepts ONLY `np.ndarray[np.float64]` with `flags['C_CONTIGUOUS']`. Otherwise raises `TypeError("xcfun_rs.eval_vec: densities must be float64 C-contiguous; got dtype={...}, strides={...}, flags={...}")`.
- **D-08:** `eval_vec` allocates and returns. Returns a freshly-allocated 2-D `np.ndarray[float64]` shaped `(nr_points, output_length)`. No `out=` kwarg in v0.1.0.

**Error mapping**
- **D-09:** Single `XcfunError` exception class with attributes `.code: int` (matches Phase 5 D-08-A `as_c_code` mapping `{Ok:0, InvalidOrder:1, InvalidVars:2, InvalidMode:4, both:6, UnknownName/other:-1}`) + `.kind: str` (Rust variant name, e.g. `'InvalidVars'`, `'WgpuNoF64'`).
- **D-10:** `WgpuNoF64` Python payload: NONE. Surfaces as `XcfunError(code=-1, kind='WgpuNoF64')` with no extra Python-side payload. Fixed message `"GPU adapter lacks f64 support"`.
- **D-11:** PyO3 default panic policy. Rust panics in `xcfun-py` surface as `pyo3.PanicException`. No `catch_unwind` shim.
- **D-12:** Constructor raises eagerly on invalid `(vars, mode, order)`. Per D-05 the constructor calls `eval_setup` at construction; bad combinations raise `XcfunError` from the constructor itself.

**Release ceremony**
- **D-13:** Initial release version `v0.1.0`. Pre-1.0 framing — "API may evolve, semver MAY have breaking changes 0.1 → 0.2".
- **D-14:** HUMAN-UAT items 3 / 4 / 5 / 6 BLOCK v0.1.0; items 1 / 2 SKIP to v0.2.
  - (3) MPMATH ground-truth fixture regen ~6h offline — BLOCK.
  - (4) Plan 06-N1 11-functional auto-tightening verify — BLOCK.
  - (5) Plan 06-N3 18-functional auto-tightening verify — BLOCK.
  - (6) `BR_Q_PREFACTOR_F64` typo fix in `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs:37` — BLOCK.
  - (1) ROCm tier-3 1e-13 hardware sweep — SKIP.
  - (2) Wgpu tier-3 1e-9 hardware sweep — SKIP.
- **D-15:** Topological `cargo publish` automated by NEW `xtask release-publish` binary. Walks dep DAG `xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval → xcfun-gpu → xcfun-rs → xcfun-capi`, ~30s sleep between publishes, idempotent (`cargo search` skip-if-already-published), `--dry-run` vs `--execute`. Final step: `maturin publish --skip-existing`.
- **D-16:** GitHub Release tag `v0.1.0` triggers GitHub Actions release-build workflow producing source tarball + `xcfun.h` (cbindgen-generated) + pre-built `libxcfun_capi.{so,dylib,dll}` for {Linux x86_64, macOS arm64, Windows x86_64}, all with `RUSTFLAGS=""` + `-fno-fast-math` discipline retained.

### Claude's Discretion

- Concrete `pyproject.toml` content (build-backend = `maturin`; ABI3 metadata; classifiers; URLs).
- Type-stub strategy: hand-written `.pyi` co-located with the wheel.
- Mode / Vars Python enum exposure: pyo3 `#[pyclass(eq, eq_int)]` IntEnum.
- Free-function placement: module top-level (`xcfun_rs.version()`).
- pytest harness shape: pytest fixtures load Rust-driver-computed expected values from JSON committed to the repo.
- CHANGELOG.md format: Keep-a-Changelog (https://keepachangelog.com/).
- PyPI publish auth: PyPI Trusted Publisher (GitHub OIDC).
- Wheel matrix concrete shape: `manylinux_2_28_x86_64`, `macosx_11_0_arm64`, `win_amd64`. `abi3-py310` means one wheel per platform covers CPython 3.10 / 3.11 / 3.12 / 3.13.
- Whether `xcfun_rs.Backend` / `xcfun_rs.auto_backend()` get a `# private` underscore prefix or are simply absent from `__all__` (D-04 says hidden — implementer picks the absence form).
- Wheel filename suffix for GPU rebuilds (e.g., `xcfun_rs-0.1.0+cuda12-cp310-abi3-linux_x86_64.whl`) — the `+cuda12` local-version-identifier is PEP 440 conformant.
- Linker stripping / debug-info stripping for the released `libxcfun_capi.{so,dylib}`.

### Deferred Ideas (OUT OF SCOPE)

- `out=` kwarg on `eval_vec`.
- Per-backend separate PyPI distributions (`xcfun_rs_cuda`, `xcfun_rs_rocm`).
- All-in-one GPU-enabled wheel.
- ROCm tier-3 1e-13 hardware sweep — v0.2.
- Wgpu tier-3 1e-9 hardware sweep — v0.2.
- Linux aarch64 / macOS x86_64 / Windows aarch64 wheels — v0.2.
- Pre-built `libxcfun_capi` for additional triples — v0.2.
- `Backend` enum exposure to Python — v2+.
- `adapter_name` payload on `WgpuNoF64` exception in Python — explicitly dropped per user.
- `requested_runtime` payload on `WgpuNoF64` — dropped.
- `catch_unwind` shim around PyO3 entries.
- v1.0.0 hard semver lock.
- Patches to `xcfun-master/` C++ source.
- PyPI publish via long-lived API token — OIDC only.
- Type stubs auto-generated from PyO3.
- Python `@dataclass`-style configuration objects for parameters.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **PY-01** | `xcfun-py` builds as a PyO3 0.28 extension module with `abi3-py310`. | §2.1 (Cargo.toml shape), §2.2 (pyproject.toml), §3.1 (`#[pymodule]` skeleton). |
| **PY-02** | `xcfun_rs.Functional` Python class exposes `set`, `get`, `is_gga`, `is_metagga`, `eval_setup`, `user_eval_setup`, `input_length`, `output_length`, `eval`, `eval_vec`. | §3.2 (Functional `#[pyclass]` skeleton), §3.3 (eager constructor pattern). |
| **PY-03** | `eval_vec` accepts a 2-D `numpy.ndarray[np.float64, order='C']` and returns a zero-copy 2-D `PyArray2<f64>`. | §4 (rust-numpy strict zero-copy contract), §4.2 (raise-on-non-contig pattern). |
| **PY-04** | Free functions (`version`, `splash`, `describe_*`, `enumerate_*`, `which_*`, `self_test`, `is_compatible_library`) exposed at module level. | §3.5 (free-fn `#[pyfunction]` pattern). |
| **PY-05** | Rust `XcError` raises Python `XcfunError` exception. | §5 (CRITICAL FINDING — abi3-py310 + `extends=PyException` collision), §5.1 (workaround), §5.2 (`From<XcError> for PyErr`). |
| **PY-06** | `pip install xcfun_rs` wheel build succeeds on Linux/macOS/Windows and passes `pytest`. | §6 (maturin commands), §7 (CI wheel matrix), §8 (pytest harness). |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

The following directives carry the same authority as locked decisions and MUST NOT be violated:

- **`pyo3 = "=0.28.3"`** with features `["extension-module", "abi3-py310"]` — load-bearing pin (0.28.0 / 0.28.1 are yanked).
- **`numpy = "=0.28.0"`** — must track pyo3 0.28.x atomically.
- **`maturin >=1.12, <2.0`** (current 1.13.1) — declared in `pyproject.toml`, NOT `Cargo.toml`.
- **`cubecl =0.10.0-pre.3`** — pre-release dep that JUSTIFIES the v0.1.0 (NOT v1.0.0) framing.
- **No `anyhow` in any library crate** — `xcfun-py` joins the enforced set when promoted to `members`. Use `thiserror` + `pyo3::exceptions` patterns.
- **No `-Cfast-math` / no `RUSTFLAGS` reassociation** — the released `libxcfun_capi.{so,dylib,dll}` MUST be built with `RUSTFLAGS=""` + `-fno-fast-math`. CI must verify.
- **No `f32` on the numerical path** — Python wheel must reject any `eval_vec` density with `dtype != np.float64` (D-07).
- **No `rayon` in library crates** — Python's `eval_vec` releases the GIL via `py.detach()` (PyO3 0.28 renamed from `allow_threads`); the Rust-side parallelism is owned by `cubecl-cpu`'s scheduler, not by Python.
- **MSRV 1.85 / Rust 2024 / `RUSTFLAGS` empty in CI** — already enforced; xcfun-py inherits.
- **MPL-2.0 license** inherited.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `Functional` Python class — `set`/`get`/`eval_setup`/`is_gga`/etc. | `xcfun-py` (PyO3 shim) | `xcfun-rs::Functional` (Rust facade) | Thin wrapper — every method delegates 1:1 to the already-validated Phase 5 facade. |
| Module-level free fns (`version`, `splash`, etc.) | `xcfun-py` (`#[pyfunction]`s) | `xcfun-rs` free fns | Direct delegation to `xcfun_rs::version()` etc. |
| `eval_vec` zero-copy NumPy IO | `xcfun-py` (NumPy strict-validate gate) | `xcfun-rs::Functional::eval_vec` (Rust pitched-buffer dispatch) | NumPy contiguity check + dtype check at the Python boundary; Rust side already pitched-flat. |
| Backend selection (HIDDEN per D-04) | `xcfun-rs` (Rust-side `auto_backend()`) | — | Never crosses the Python boundary. CPU-only PyPI wheel never resolves to non-CPU. |
| `XcfunError` exception class | `xcfun-py` (`create_exception!` + Python-source `__init__` shim — see §5 CRITICAL FINDING) | `xcfun-core::XcError` (Rust source) | abi3-py310 forces a workaround. |
| Mode / Vars Python enums | `xcfun-py` (`#[pyclass(eq, eq_int)]` IntEnum) | `xcfun-core::{Mode, Vars}` | Re-export. |
| `xtask release-publish` topological driver | `xtask/` | — | New binary; walks `cargo metadata` DAG and shells out to `cargo publish`. App-boundary, anyhow OK. |
| GitHub Actions wheel matrix | `.github/workflows/release.yml` | `PyO3/maturin-action@v1` | One workflow on tag-push; matrix over {linux, macos, windows} + sdist + publish job. |
| GitHub Release artifact pipeline | `.github/workflows/release.yml` (release-artifacts job) | `cargo build -p xcfun-capi --release` × 3 platforms | Builds CPU-only `libxcfun_capi.{so,dylib,dll}` + cbindgen header, attaches to release. |
| pytest harness against Rust-driver JSON fixtures | `crates/xcfun-py/tests/` (Rust-side `examples/gen_py_fixtures.rs` + Python `tests/test_parity.py`) | `xcfun-rs::Functional` (the driver) | Hermetic — no C++ at Python test time; reuses Phase 5 D-08-A bit-pattern. |

## Standard Stack

### Core (already pinned in `Cargo.toml` workspace dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `pyo3` | `=0.28.3` (features `["extension-module", "abi3-py310"]`) | Python<->Rust FFI | Single maintained option; `=0.28.3` pin is load-bearing (0.28.0 / 0.28.1 yanked). [VERIFIED: `cargo search pyo3` 2026-05-05 → `pyo3 = "0.28.3"`]. |
| `numpy` (rust-numpy) | `=0.28.0` | Zero-copy NumPy `f64` array interop | Companion crate to PyO3 0.28; must move atomically. [VERIFIED: `cargo search numpy` 2026-05-05 → `numpy = "0.28.0"`]. |
| `xcfun-rs` (path) | workspace | Rust facade being wrapped | Already shipped Phase 5 (RS-01..10 complete). |
| `thiserror` | `=2.0.18` (workspace) | Library error type for `xcfun-py` internal helpers (rare; most paths return `PyResult<T>`) | Library-graph standard. |

### Build / wheel tooling (declared in `pyproject.toml` — NOT in `Cargo.toml`)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `maturin` | `>=1.12, <2.0` (current 1.13.1) | Build backend for the wheel; declared via PEP 517 `[build-system]` | Standard PyO3 build tool; abi3-py310 wheel layout. [VERIFIED: `cargo search maturin` 2026-05-05 → `maturin = "1.13.1"`]. |
| `pytest` | `>=7.0` (dev-only, in `[project.optional-dependencies]`) | Python test runner | De facto standard. |
| `numpy` (Python wheel, runtime dep) | `>=1.26` (broad) | Required at runtime by the Python module | Match Python ecosystem floor. |

### CI / release infrastructure

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `PyO3/maturin-action` | `v1` (pin to a specific tag for hardening; e.g. `v1.51.0`) | GitHub Action that runs `maturin build` / `maturin publish` cross-platform | The canonical wheel-builder action; supports `manylinux`, target triples, sdist. [CITED: github.com/PyO3/maturin-action]. |
| `pypa/gh-action-pypi-publish` | `release/v1` | OIDC trusted-publisher PyPI upload | Modern recommendation; rotates better than long-lived tokens. [CITED: docs.pypi.org/trusted-publishers]. |
| `actions/checkout` | `v6` (current) | Standard checkout | — |
| `actions/setup-python` | `v6` (current) | Python interpreter for sdist + wheel testing | — |
| `actions/upload-artifact` | `v7` (current) | Pass wheels between jobs | — |
| `actions/download-artifact` | `v8` (current) | Pull wheels for the publish job | — |

### Alternatives Considered

| Instead of | Could Use | Tradeoff (when alternative makes sense) |
|------------|-----------|-----------------------------------------|
| `PyO3/maturin-action@v1` | `cibuildwheel` (PyPA) | cibuildwheel is the C-extension community standard, but maturin-action handles Rust-aware container images, target triples, and `--features` cleanly. Stick with maturin-action — it's the canonical PyO3 path. |
| `pypa/gh-action-pypi-publish@release/v1` | `maturin publish --skip-existing` directly inside the workflow | maturin-publish-direct is documented [CITED: maturin.rs/distribution] but trusted-publisher OIDC is the modern recommendation. Use the gh-action; remove `MATURIN_PYPI_TOKEN` env. Still call `maturin publish` if dropping the gh-action. |
| `cargo-workspaces` / `cargo-publish-ordered` | Hand-rolled `xtask release-publish` | The user's D-15 explicitly mandates the new `xtask` binary. Don't import a third-party publisher. The xtask uses `cargo metadata --format-version 1` to compute topological order and shells out to `cargo publish -p <crate>`. |
| `pyo3-stub-gen` (auto stubs) | Hand-written `.pyi` | Phase 7 surface (Functional + 11 free fns + 2 enums + 1 exception) is small and hand-writable. Auto-stub-gen adds a build-time dep with a different release cadence. Defer to v0.2. |
| `pyo3` `#[pyclass(extends=PyException)]` | `create_exception!` + Python `__init__` shim | **CRITICAL** — abi3-py310 forbids subclassing `PyException` until Python 3.12. See §5. |

**Installation (during dev):**

```bash
# Install maturin (one-time)
pip install 'maturin>=1.12,<2.0'

# Activate a Python venv with Python ≥ 3.10
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip pytest numpy

# Build + install the extension into the venv (development mode)
cd crates/xcfun-py
maturin develop --release   # or `maturin develop` for a debug build
# Or for a hermetic wheel test:
maturin build --release --features cpu --out dist/
pip install dist/xcfun_rs-0.1.0-cp310-abi3-*.whl
```

**Version verification (run in plan-phase, not now):**

```bash
cargo search pyo3 numpy maturin   # confirms current versions on crates.io
pip index versions maturin pytest # confirms PyPI floors
```

[VERIFIED: 2026-05-05] `pyo3 = "0.28.3"`, `numpy = "0.28.0"`, `maturin = "1.13.1"`. All match the CLAUDE.md pins.

## Architecture Patterns

### System Architecture Diagram

```
                        Python user
                            │
                            │ import xcfun_rs as xc
                            │ f = xc.Functional("pbe", vars=xc.Vars.A_B,
                            │                   mode=xc.Mode.PartialDerivatives, order=2)
                            │ f.set("exx", 0.25)
                            │ y = f.eval_vec(np.zeros((1024, 2), dtype=np.float64))
                            ▼
            ┌─────────────────────────────────────────┐
            │ xcfun-py (PyO3 0.28.3 cdylib)           │
            │ ───────────────────────────────────────  │
            │ #[pymodule] mod xcfun_rs                │
            │   ├── #[pyclass] Functional ───────┐    │
            │   ├── #[pyclass(eq, eq_int)] Mode  │    │
            │   ├── #[pyclass(eq, eq_int)] Vars  │    │
            │   ├── 11× #[pyfunction] (free fns) │    │
            │   └── XcfunError (PyException +    │    │
            │       Python __init__ shim — §5)   │    │
            └────────────────────────────────────┼────┘
                                                 │
                    delegates 1:1                ▼
            ┌─────────────────────────────────────────┐
            │ xcfun-rs (Phase 5; already shipped)     │
            │ ───────────────────────────────────────  │
            │ pub struct Functional { inner, … }      │
            │ pub fn version()/splash()/…             │
            │ pub use xcfun_core::{Mode, Vars, XcError}│
            └─────────────────────────────────────────┘
                                                 │
                                                 ▼
            ┌─────────────────────────────────────────┐
            │ xcfun-eval / xcfun-gpu / xcfun-kernels  │
            │ (Phase 6; eval_vec dispatch)            │
            └─────────────────────────────────────────┘

────────────────────────────────────────────────────────
                    Build & Release Pipeline
────────────────────────────────────────────────────────

  source tree                                   crates.io
      │                                              ▲
      │ git tag v0.1.0                              │
      ▼                                              │
  GitHub Actions release.yml                         │
      │                                              │
      ├─► sdist job  (maturin sdist)                 │
      ├─► linux job (manylinux_2_28_x86_64)          │
      ├─► macos job (macosx_11_0_arm64)              │
      ├─► windows job (win_amd64)        ┌───────────┘
      │                                  │
      │      ┌───────────────────────────┘
      │      │
      ▼      ▼                                 ┌────────────► PyPI (OIDC trusted-publisher)
   publish-pypi job  (pypa/gh-action-pypi-publish@release/v1)
      │
      ▼
   release-artifacts job (parallel for {linux, macos, windows})
      │
      ├─► cargo build --release -p xcfun-capi
      ├─► attach libxcfun_capi.{so,dylib,dll}
      ├─► attach xcfun.h (cbindgen-generated)
      ├─► attach source tarball
      │
      ▼
   GitHub Release v0.1.0

   Parallel — driven by `xtask release-publish --execute` from a maintainer's machine
   (NOT from CI by default — D-15 documents but doesn't mandate CI):

      xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval →
        xcfun-gpu → xcfun-rs → xcfun-capi  (~30s sleep between)
      maturin publish --skip-existing  (if local PyPI publish desired;
                                         GH Actions does this by default)
```

### Recommended Project Structure

```
crates/xcfun-py/                        # renamed from xcfun-python (D-01)
├── Cargo.toml                           # PyO3 + numpy + xcfun-rs deps; cdylib name = xcfun_rs
├── pyproject.toml                       # maturin build-backend; abi3-py310; classifiers
├── README.md                            # User-facing install + GPU rebuild instructions
├── CHANGELOG.md                         # Symlink to repo-root CHANGELOG.md OR per-crate
├── src/
│   └── lib.rs                           # #[pymodule] xcfun_rs { Functional + 11 free fns + enums + XcfunError }
├── python/                              # Python source dir (maturin python-source = "python")
│   └── xcfun_rs/
│       ├── __init__.py                  # Re-exports + the XcfunError __init__ shim (§5.1)
│       ├── __init__.pyi                 # Hand-written type stubs
│       └── py.typed                     # Empty marker file (PEP 561)
└── tests/
    ├── conftest.py                      # pytest fixtures
    ├── fixtures/
    │   └── eval_parity.json             # Generated by examples/gen_py_fixtures.rs (committed)
    ├── test_smoke.py                    # version()/splash()/import sanity
    ├── test_functional.py               # Functional class methods (set/get/eval/eval_vec)
    ├── test_eval_vec_zero_copy.py       # D-07 strict raise-on-non-contig
    ├── test_xcfun_error.py              # D-09 .code/.kind attribute access
    └── test_parity.py                   # JSON-fixture-driven 1e-12 parity
```

```
.github/workflows/
├── ci.yml                               # Existing (or new) — fmt, clippy, test on push
└── release.yml                          # NEW — Phase 7 D-15/D-16 — tagged on v* push

xtask/src/bin/
└── release_publish.rs                   # NEW — D-15 topological cargo publish driver

CHANGELOG.md                             # NEW — Keep-a-Changelog format at repo root
```

### Pattern 1: `#[pymodule]` skeleton (PY-01, PY-02, PY-04)

```rust
// crates/xcfun-py/src/lib.rs
//
// PyO3 0.28.3 extension module — wraps xcfun-rs::Functional + 11 free fns.
// Note: PyO3 0.28 uses Python::attach (renamed from with_gil) and py.detach
// (renamed from py.allow_threads).

use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::exceptions::PyException;

mod functional;       // Functional pyclass + Mode/Vars enums + free fns
mod numpy_io;         // eval_vec strict zero-copy contract (D-07)
mod errors;           // XcError → PyErr conversion (§5)

// Source: https://pyo3.rs/v0.28.3/exception
// CRITICAL: under abi3-py310, subclassing PyException is forbidden until
// Python 3.12 — see §5 of this RESEARCH.md. We declare the exception class
// here via create_exception! and graft .code/.kind attributes via the
// Python-source __init__ shim in python/xcfun_rs/__init__.py.
create_exception!(xcfun_rs, XcfunError, PyException);

#[pymodule]
fn xcfun_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use functional::{Functional, Mode, Vars};
    use functional::free_fns::*;

    // PY-02 — Functional class
    m.add_class::<Functional>()?;

    // Claude's discretion — Mode / Vars exposed as IntEnum
    // Source: pyo3 guide §class.md "Rust Enum for Integer Conversion"
    m.add_class::<Mode>()?;
    m.add_class::<Vars>()?;

    // PY-05 — single XcfunError class (the .code/.kind shim is in __init__.py)
    m.add("XcfunError", m.py().get_type::<XcfunError>())?;

    // PY-04 — 11 module-level free functions (mirror xcfun-rs/src/free_fns.rs)
    m.add_function(wrap_pyfunction!(version,                 m)?)?;
    m.add_function(wrap_pyfunction!(splash,                  m)?)?;
    m.add_function(wrap_pyfunction!(authors,                 m)?)?;
    m.add_function(wrap_pyfunction!(self_test,               m)?)?;
    m.add_function(wrap_pyfunction!(is_compatible_library,   m)?)?;
    m.add_function(wrap_pyfunction!(which_vars,              m)?)?;
    m.add_function(wrap_pyfunction!(which_mode,              m)?)?;
    m.add_function(wrap_pyfunction!(enumerate_parameters,    m)?)?;
    m.add_function(wrap_pyfunction!(enumerate_aliases,       m)?)?;
    m.add_function(wrap_pyfunction!(describe_short,          m)?)?;
    m.add_function(wrap_pyfunction!(describe_long,           m)?)?;

    Ok(())
}
```

[CITED: github.com/PyO3/pyo3 guide/src/module.md, exception.md, class.md]

### Pattern 2: `Functional` `#[pyclass]` with eager constructor (D-05, D-06, D-12)

```rust
// crates/xcfun-py/src/functional.rs

use numpy::{IntoPyArray, PyArray2, PyReadonlyArray2, PyArrayMethods, PyUntypedArrayMethods};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use xcfun_rs::{Functional as RsFunctional, Mode as RsMode, Vars as RsVars};

#[pyclass(name = "Functional", module = "xcfun_rs")]
pub struct Functional {
    inner: RsFunctional,
}

#[pymethods]
impl Functional {
    /// D-05 — eager constructor. Calls `eval_setup` if vars/mode/order all
    /// supplied; raises `XcfunError` from the constructor on bad combos
    /// (D-12).
    ///
    /// Source: pyo3 guide §function/signature — `#[pyo3(signature = ...)]`
    /// supplies positional + kw-only + defaults.
    #[new]
    #[pyo3(signature = (name, *, vars=None, mode=None, order=None))]
    fn new(
        name: &str,
        vars: Option<Vars>,
        mode: Option<Mode>,
        order: Option<u32>,
    ) -> PyResult<Self> {
        let mut inner = RsFunctional::new();
        // RS-02 — set the named functional weight.
        inner.set(name, 1.0).map_err(crate::errors::xc_to_py)?;

        // D-05 — if all three eval_setup args supplied, run eval_setup eagerly.
        if let (Some(v), Some(m), Some(o)) = (vars, mode, order) {
            inner
                .eval_setup(v.into(), m.into(), o)
                .map_err(crate::errors::xc_to_py)?;
        }
        Ok(Self { inner })
    }

    /// D-05 — low-level escape hatch. `f.configure(vars=..., mode=..., order=...)`
    /// runs eval_setup later. Same XcfunError mapping on bad combos.
    fn configure(&mut self, vars: Vars, mode: Mode, order: u32) -> PyResult<()> {
        self.inner
            .eval_setup(vars.into(), mode.into(), order)
            .map_err(crate::errors::xc_to_py)
    }

    /// D-06 — set() mutates in place, returns None. Aliases compose additively.
    fn set(&mut self, name: &str, value: f64) -> PyResult<()> {
        self.inner.set(name, value).map_err(crate::errors::xc_to_py)
    }

    fn get(&self, name: &str) -> PyResult<f64> {
        self.inner.get(name).map_err(crate::errors::xc_to_py)
    }

    fn is_gga(&self) -> bool { self.inner.is_gga() }
    fn is_metagga(&self) -> bool { self.inner.is_metagga() }

    fn eval_setup(&mut self, vars: Vars, mode: Mode, order: u32) -> PyResult<()> {
        self.inner
            .eval_setup(vars.into(), mode.into(), order)
            .map_err(crate::errors::xc_to_py)
    }

    #[pyo3(signature = (order, func_type, dens_type, mode_type,
                       laplacian, kinetic, current, explicit_derivatives))]
    fn user_eval_setup(
        &mut self, order: i32, func_type: u32, dens_type: u32, mode_type: u32,
        laplacian: u32, kinetic: u32, current: u32, explicit_derivatives: u32,
    ) -> PyResult<()> {
        self.inner
            .user_eval_setup(order, func_type, dens_type, mode_type,
                             laplacian, kinetic, current, explicit_derivatives)
            .map_err(crate::errors::xc_to_py)
    }

    fn input_length(&self) -> usize { self.inner.input_length() }

    fn output_length(&self) -> PyResult<usize> {
        self.inner.output_length().map_err(crate::errors::xc_to_py)
    }

    /// Per-point eval. `density` and `out` are 1-D f64 arrays. Returns None;
    /// writes into `out` in place.
    ///
    /// Releases the GIL via py.detach() (PyO3 0.28; renamed from py.allow_threads).
    /// Source: pyo3 guide §parallelism.md.
    fn eval<'py>(
        &self,
        py: Python<'py>,
        density: PyReadonlyArray2<'py, f64>,    // see notes — we accept 1-D as 2-D[1,n]
        out:     numpy::PyReadwriteArray1<'py, f64>,
    ) -> PyResult<()> {
        // (Implementation detail — Plan can decide 1-D vs 2-D-row API.)
        // The simpler signature: 1-D PyReadonlyArray1 + PyReadwriteArray1.
        // The Plan picks a single shape and locks it in pyi stubs.
        unimplemented!()
    }

    /// PY-03 / D-07 / D-08 — strict-zero-copy eval_vec.
    /// See §4 for the full code excerpt.
    #[pyo3(signature = (densities))]
    fn eval_vec<'py>(
        &self,
        py: Python<'py>,
        densities: PyReadonlyArray2<'py, f64>,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        crate::numpy_io::eval_vec_impl(py, &self.inner, densities)
    }
}
```

[CITED: pyo3 guide §class.md, §function/signature.md; rust-numpy llms.txt]

### Pattern 3: `Mode` / `Vars` as `#[pyclass(eq, eq_int)]` IntEnum (Claude's discretion)

```rust
// crates/xcfun-py/src/functional.rs (continued)
//
// IntEnum exposure. Mode discriminants match the C ABI u32; Vars likewise.
// The eq + eq_int attrs make them comparable both as Python enum members
// AND as integers, e.g. `Mode.PartialDerivatives == 1` is True in Python.
//
// Source: pyo3 guide §class.md "Rust Enum with PyO3 Attributes for
// Comparison and Integer Conversion".

#[pyclass(eq, eq_int, name = "Mode", module = "xcfun_rs")]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Mode {
    Unset = 0,
    PartialDerivatives = 1,
    Potential = 2,
    Contracted = 3,
}

impl From<Mode> for RsMode {
    fn from(m: Mode) -> RsMode {
        match m {
            Mode::Unset => RsMode::Unset,
            Mode::PartialDerivatives => RsMode::PartialDerivatives,
            Mode::Potential => RsMode::Potential,
            Mode::Contracted => RsMode::Contracted,
        }
    }
}

// Vars: identical pattern, all 31 variants. Discriminants per
// xcfun-master/api/xcfun.h and xcfun-core::Vars (CORE-01).
#[pyclass(eq, eq_int, name = "Vars", module = "xcfun_rs")]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Vars {
    A = 0,
    N = 1,
    A_B = 2,
    // ... all 31 variants ...
}
```

**Caveat:** if any `Vars` variant has `#[non_exhaustive]` upstream, the IntEnum copy breaks. Plan-phase MUST verify by inspecting `xcfun-core::Vars` definition. Currently `xcfun_core::Vars` is `#[allow(non_camel_case_types)] pub enum Vars { … }` per Phase 2 D-08 — NO `non_exhaustive`. Safe to mirror discriminants.

[CITED: pyo3 guide §class.md "Rust Enum for Integer Conversion with PyO3"]

### Pattern 4: GIL release on the `eval_vec` hot path (PyO3 0.28 API rename)

PyO3 0.28 renames `py.allow_threads(...)` to `py.detach(...)`. The Rust hot path inside `eval_vec` does NOT touch Python objects — only `&[f64]` and `&mut [f64]` — so it is safe to release the GIL.

```rust
// crates/xcfun-py/src/numpy_io.rs (excerpt — see §4 for full impl)

py.detach(|| {
    inner.eval_vec(
        density_slice,
        density_pitch,
        out_slice,
        out_pitch,
        nr_points,
    )
}).map_err(crate::errors::xc_to_py)?;
```

[CITED: pyo3 guide §parallelism.md "Detaching the GIL for parallel execution"]
**Migration note:** older PyO3 examples in the project's research from prior phases may reference `py.allow_threads(...)` — that name is still present as a deprecated alias in 0.28 but the canonical is `py.detach`.

### Anti-Patterns to Avoid

- **`#[pyclass(extends=PyException)]` with the abi3-py310 feature** — this **fails at runtime on Python 3.10/3.11** even if it compiles (per `#[cfg(any(not(Py_LIMITED_API), Py_3_12))]` guard upstream). See §5 for the workaround.
- **Calling `f.eval()` without releasing the GIL** — wastes parallelism for any caller using a thread pool. Always wrap the Rust hot path in `py.detach`.
- **Returning `&mut [f64]` borrowed from a `PyReadwriteArray2` without holding the readwrite guard** — UB. Always go through `as_array_mut()` while the guard is alive.
- **Silently `np.ascontiguousarray` the input** — D-07 explicitly forbids this. Raise `TypeError` instead. Users who want auto-coercion can do it themselves at the call site.
- **`anyhow` anywhere in `xcfun-py`** — CI-blocked once promoted to workspace members.
- **Hard-coded version strings in Python source** — use `xcfun_rs::version()` (which returns `env!("CARGO_PKG_VERSION")`) so the Python `xcfun_rs.__version__` automatically tracks the Cargo version. Synchronize via `__init__.py: __version__ = _native.version()`.
- **`maturin build` without `--release` for distribution wheels** — debug builds violate the perf contract.
- **`xtask release-publish --execute` without a clean `cargo publish --dry-run`** — yanks are irreversible. The xtask MUST gate `--execute` on a successful dry-run pass per crate.
- **Passing GPU `--features cuda` / `wgpu` / `hip` to the default PyPI wheel** — D-03 forbids; CPU-only wheel only.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Wheel build for {linux x86_64, macos arm64, win amd64} | Hand-rolled `cargo build` + `auditwheel` step | `PyO3/maturin-action@v1` | Handles `manylinux_2_28` containers, target triples, and the abi3 wheel name/tag layout. |
| PyPI upload | Long-lived API token in CI secret | `pypa/gh-action-pypi-publish@release/v1` (OIDC) | Modern; rotates better; the maturin docs explicitly recommend the OIDC path. [CITED: maturin.rs/distribution] |
| NumPy contiguity / dtype check | `unsafe` raw pointer arithmetic | `PyUntypedArrayMethods::is_c_contiguous()` + `.dtype()` + `.ndim()` | rust-numpy 0.28.0 exposes these on the trait. [CITED: docs.rs/numpy/0.28.0/numpy/trait.PyUntypedArrayMethods] |
| Type stubs auto-generation | Build-time stub-gen pipeline (`pyo3-stub-gen`) | Hand-written `.pyi` (Functional + 11 free fns + 2 enums + 1 exception ≈ 80 LoC) | The surface is small; auto-gen adds a new dep with a different release cadence. Revisit at v0.2. |
| Topological cargo publish | `cargo-workspaces publish` (third-party) | New `xtask release-publish` (D-15) | Project policy: third-party publishers add a non-trivial dep + a permissions surface. The `xtask` walks `cargo metadata` and shells out to `cargo publish -p <crate>`. |
| Wheel name suffix for GPU rebuilds | Manually editing the wheel filename | PEP 440 local-version-identifier in `pyproject.toml` (`version = "0.1.0+cuda12"`) | PEP 440-conformant; pip resolves correctly. |
| Python exception with custom attributes | Subclass `PyException` from Rust under abi3-py310 | `create_exception!` + Python-source `__init__` shim (§5.1) | abi3-py310 forbids subclassing PyException until Python 3.12. |

**Key insight:** every "could we hand-roll this" question in Phase 7 has a maintained, single-purpose tool already in the ecosystem. The only NEW custom artifact is `xtask release-publish` — and that's mandated by D-15 specifically to keep the publish ceremony in-project (no third-party publisher dep on the release path).

## Common Pitfalls

### Pitfall 1: abi3-py310 + `#[pyclass(extends=PyException)]` collision (CRITICAL)

**What goes wrong:** Naive implementation of D-09 — `#[pyclass(extends=PyException)] struct XcfunError { code, kind }` — compiles fine but **fails at runtime on Python 3.10 / 3.11** under the `abi3-py310` feature. See §5 for full analysis.

**Why it happens:** Python 3.12 introduced `PyType_FromMetaclass` which is required to subclass exception types from the limited (abi3) API. Earlier Python versions don't have it; under `abi3-py310` PyO3's `#[cfg(any(not(Py_LIMITED_API), Py_3_12))]` guard refuses to instantiate the subclass.

**How to avoid:** Use `create_exception!(xcfun_rs, XcfunError, PyException)` and graft the `.code` / `.kind` attributes via a Python-source `__init__.py` shim. See §5.1 for the canonical pattern.

**Warning signs:** any Phase 7 plan that proposes `#[pyclass(extends=PyException)]` with a `#[derive(Clone)]` exception struct must be rejected by plan-checker.

### Pitfall 2: Per-launch cubecl-cpu allocation cost in `eval_vec`

**What goes wrong:** `Functional::eval_vec` for small `nr_points` (< 64) falls through to the per-point `eval` loop, which Phase 6 D-12 documented as ~287 allocs/eval cubecl-cpu substrate cost. Python users running 1024-point batches notice this when they expect linear scaling from CPU `eval_vec`.

**Why it happens:** Phase 6's `EvalHandle::cached_*` fields are structural-only — populated by the future cubecl substrate upgrade. The Phase 5 fall-back form is still active for the `nr_points < threshold` path.

**How to avoid:** Document the 64-point dispatch threshold in the README. Python-side `eval_vec` benchmark fixtures should test BOTH `nr_points = 32` (per-point loop) and `nr_points = 1024` (Batch dispatch) so users know which path their workload hits.

**Warning signs:** pytest test_eval_vec_zero_copy.py timing > 100 ms on a 64-point batch — likely hitting the per-point loop unintentionally.

### Pitfall 3: NumPy 2-D contiguity check tripping on row-major-vs-column-major edge cases

**What goes wrong:** Caller does `densities = densities.T` (transpose) — the transposed array is now Fortran-contiguous, NOT C-contiguous. `is_c_contiguous()` returns False; `eval_vec` raises `TypeError` with a confusing-looking message ("strides=[...]") because the user thinks "but it's still 2-D and f64".

**Why it happens:** NumPy transposes are O(1) views — they reverse the strides. The user's mental model ("2-D and f64 should be enough") doesn't match the physical layout requirement.

**How to avoid:** the `TypeError` message format from D-07 already prints `dtype`, `strides`, AND `flags` — so the user has full diagnostic info. README should include a one-liner `np.ascontiguousarray(densities).astype(np.float64)` recipe for the common case. Pytest fixture `test_eval_vec_zero_copy.py` should EXPLICITLY test the transposed-array case to lock in the error message format.

**Warning signs:** user reports "TypeError on a clearly-2D-f64 array" — first thing to check is `densities.flags['C_CONTIGUOUS']`.

### Pitfall 4: `cargo publish` index propagation race (D-15)

**What goes wrong:** `xtask release-publish --execute` calls `cargo publish -p xcfun-ad`, immediately followed by `cargo publish -p xcfun-core` (which depends on xcfun-ad). The second publish fails with "no matching package named `xcfun-ad` found" because crates.io's sparse index hasn't propagated the new version yet.

**Why it happens:** crates.io publish-to-search-index latency is typically 5-30 seconds. There's no API-side ack signal that the new version is searchable.

**How to avoid:** D-15 specifies "~30s sleep between publishes" — but a more reliable signal is **`cargo search` polling**. The `xtask release-publish` should:
1. `cargo publish -p <crate>` → wait for command success.
2. Loop with 5s backoff: `cargo search <crate> --limit 1` and parse the output for the just-published version. Break loop when it appears.
3. Cap polling at 5 minutes; abort with non-zero exit if not visible.

**Warning signs:** `cargo publish -p xcfun-core` returns "failed to select a version for the requirement `xcfun-ad = ...`" mid-script. The user must NOT manually re-run the entire script — instead, identify which crate is stuck and resume from there. The `--from <crate>` flag on `xtask release-publish` lets the operator skip already-published crates idempotently.

### Pitfall 5: PyPI yank-irreversibility on a botched `0.1.0`

**What goes wrong:** `maturin publish` uploads `xcfun_rs-0.1.0-cp310-abi3-linux_x86_64.whl`, then a downstream test discovers a packaging issue (missing `py.typed`, wrong classifiers, etc.). PyPI's yank is non-destructive — it merely marks the version unsearchable for new resolution; existing pinning installs still work. But you cannot re-publish `0.1.0`.

**Why it happens:** PyPI is content-addressed by `(name, version, filename)`. Deletion is a manual support-ticket affair.

**How to avoid:** **Test the wheel build in an entire dry-run pipeline** before tagging. The plan should add a CI matrix step that:
1. Runs `maturin build --release --features cpu --out dist/` on each platform.
2. Installs the wheel into a fresh venv.
3. Runs `pytest crates/xcfun-py/tests/` against the installed wheel.
4. Verifies `python -c "import xcfun_rs as xc; print(xc.version())"` prints `0.1.0`.

Tag only after the dry-run passes on all three platforms. If a botched `0.1.0` ships, the recovery path is `0.1.1` — not "yank and re-publish".

**Warning signs:** PyPI publish job returns 403 "this version already exists" — STOP. Don't escalate; investigate.

### Pitfall 6: GitHub Actions `id-token: write` permission not granted at job level

**What goes wrong:** OIDC trusted-publishing workflow runs but `pypa/gh-action-pypi-publish` reports "OIDC token not available" or "audience not allowed".

**Why it happens:** `permissions: id-token: write` was set at the workflow level but the job inherits a more restrictive default; or PyPI's trusted-publisher config doesn't match the GitHub repo / workflow file / environment combination.

**How to avoid:** PyPA recommends setting `permissions: id-token: write` at the **job** level (not just workflow level) AND configuring a GitHub Environment (e.g. `environment: pypi`) on PyPI's side under "Trusted Publisher Management". The polars / ast-grep release workflows confirm this pattern.

**Warning signs:** "OIDC token not available" in the publish job log. Solution: add `permissions: { id-token: write }` to the job, and verify the PyPI trusted-publisher setup matches `(github.com/<owner>/<repo>, .github/workflows/release.yml, <environment-name>)`.

### Pitfall 7: f64 precision regression in pip-installed wheel vs `cargo test`

**What goes wrong:** pytest in the developer's local maturin-develop'd module passes at strict 1e-12, but the pip-installed wheel from the same source produces 1e-9 differences.

**Why it happens:** maturin's wheel build uses `--release` profile by default which has `lto = "thin"` per the workspace `Cargo.toml`. If a Phase 7 contributor adds a `[profile.release-py]` override or accidentally enables `-Cfast-math` via a feature flag, the floating-point rounding semantics drift.

**How to avoid:** the wheel CI matrix MUST run the parity pytest **on the installed wheel** (not on the source tree). The Pitfall-5 dry-run pipeline above captures this. Additionally, the existing `xtask check-no-fma` gate (Plan 02-02) should be extended to verify `xcfun-py`'s release object code has no FMA mnemonics in `xcfun_eval_vec` paths.

**Warning signs:** test_parity.py fails ONLY in the wheel-install pipeline, not in `maturin develop`. Diff the cargo build flags.

### Pitfall 8: Phase 6 RS-08 fallback path not exercised by Python tests

**What goes wrong:** Python eval_vec tests run on small batches (e.g. 16-point smoke tests) and accidentally only exercise the `nr_points < 64` per-point fallback — never the `Batch<CpuRuntime>` GPU dispatch path. The wheel ships with a broken `Batch<CpuRuntime>` invocation that nobody noticed.

**Why it happens:** the threshold gate at `Functional::eval_vec` (Phase 6 plan 06-05) routes small batches to a different code path. Python tests that only use small batches give a green light without testing the production-path code.

**How to avoid:** test_eval_vec_zero_copy.py MUST include a 1024-point fixture (well above the 64-point threshold) AND a 32-point fixture (below threshold) — so both paths are covered. Mark each test with the path it's exercising via pytest parametrization.

**Warning signs:** test runtime < 50 ms for the largest test → likely hitting the wrong path. Compare with `XCFUN_MIN_BATCH_SIZE=4 pytest ...` (which forces all batches through the Batch dispatch).

## Code Examples

(Numbered patterns above already cover most of this section. Additional concrete excerpts:)

### Example A: rust-numpy strict zero-copy gate (D-07 + PY-03)

```rust
// crates/xcfun-py/src/numpy_io.rs

use numpy::{IntoPyArray, PyArray2, PyReadonlyArray2,
            PyArrayMethods, PyUntypedArrayMethods};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use xcfun_rs::Functional as RsFunctional;

/// PY-03 / D-07 / D-08 — strict-zero-copy eval_vec.
///
/// Contract:
/// - Input: 2-D `np.ndarray[np.float64]`, C-contiguous, shape `(nr_points, inlen)`.
/// - Output: freshly-allocated 2-D `np.ndarray[np.float64]` shape `(nr_points, outlen)`.
/// - Raises `TypeError` (NOT XcfunError) on any layout violation per D-07.
///
/// Source for `is_c_contiguous` / `dtype` / `strides` / `flags`:
/// https://docs.rs/numpy/0.28.0/numpy/trait.PyUntypedArrayMethods.html
pub fn eval_vec_impl<'py>(
    py: Python<'py>,
    inner: &RsFunctional,
    densities: PyReadonlyArray2<'py, f64>,
) -> PyResult<Bound<'py, PyArray2<f64>>> {
    // ----- Layout validation per D-07.
    // PyReadonlyArray2<f64> already enforces 2-D + f64 dtype at the type level.
    // The only remaining check is C-contiguity (the trait coerces strided arrays
    // through a copy by default — we explicitly REJECT instead).
    if !densities.is_c_contiguous() {
        let strides = densities.strides();
        let dtype = densities.dtype();
        return Err(PyTypeError::new_err(format!(
            "xcfun_rs.eval_vec: densities must be float64 C-contiguous; \
             got dtype={}, strides={:?}, flags=non-C-contiguous",
            dtype, strides,
        )));
    }

    let shape = densities.shape();
    let nr_points = shape[0];
    let inlen     = shape[1];
    let expected_inlen = inner.input_buffer_length();
    if inlen != expected_inlen {
        return Err(PyTypeError::new_err(format!(
            "xcfun_rs.eval_vec: densities axis-1 length {} does not match \
             functional input length {}",
            inlen, expected_inlen,
        )));
    }

    let outlen = inner.output_length().map_err(crate::errors::xc_to_py)?;

    // ----- D-08: allocate the output as a fresh 2-D PyArray2<f64>.
    // Source: rust-numpy llms.txt — `IntoPyArray` for owned ndarray buffers.
    let out_array = PyArray2::<f64>::zeros(py, [nr_points, outlen], false);

    // ----- Hot path: release the GIL during the Rust eval_vec call.
    {
        let dens_view = densities.as_slice()?;       // contiguous f64 slice
        // Get a mutable raw view of the freshly-allocated output. Safety:
        // `out_array` was just created with no aliases.
        let mut out_rw = out_array.readwrite();
        let out_view = out_rw.as_slice_mut()?;

        // The Rust eval_vec is Send+Sync — Phase 5 RS-10 — so detach is safe.
        // Source: pyo3 guide §parallelism.md.
        py.detach(|| {
            inner.eval_vec(
                dens_view,
                inlen,                          // density_pitch == inlen (C-contig)
                out_view,
                outlen,                         // out_pitch     == outlen (fresh alloc)
                nr_points,
            )
        }).map_err(crate::errors::xc_to_py)?;
    }

    Ok(out_array)
}
```

[CITED: rust-numpy llms.txt; pyo3 guide §parallelism.md]

### Example B: `XcError → PyErr` conversion (PY-05 — see §5 for the full discussion)

```rust
// crates/xcfun-py/src/errors.rs

use pyo3::PyErr;
use xcfun_core::XcError;

/// Convert a Rust XcError into a Python XcfunError instance with .code and
/// .kind attributes attached.
///
/// Note: the .code/.kind attribute attachment happens in Python source code
/// (python/xcfun_rs/__init__.py — see §5.1) because abi3-py310 forbids
/// subclassing PyException at the C level until Python 3.12.
///
/// Source: pyo3 guide §exception.md "Define a new exception with create_exception!"
pub fn xc_to_py(err: XcError) -> PyErr {
    let code = err.as_c_code();
    let kind = match err {
        XcError::InvalidOrder { .. }       => "InvalidOrder",
        XcError::InvalidVars  { .. }       => "InvalidVars",
        XcError::InvalidMode  { .. }       => "InvalidMode",
        XcError::InvalidVarsAndMode { .. } => "InvalidVarsAndMode",
        XcError::UnknownName               => "UnknownName",
        XcError::InputLengthMismatch  { .. } => "InputLengthMismatch",
        XcError::OutputLengthMismatch { .. } => "OutputLengthMismatch",
        XcError::NotConfigured             => "NotConfigured",
        XcError::InvalidEncoding           => "InvalidEncoding",
        XcError::Runtime                   => "Runtime",
        XcError::WgpuNoF64 { .. }          => "WgpuNoF64",
        XcError::CudaNoF64 { .. }          => "CudaNoF64",
        // _ unreachable — XcError is #[non_exhaustive] but every variant
        //   has a kind string; new variants must be added here.
    };

    // D-09 — message is the Rust Display impl unless this is the WgpuNoF64
    // variant where D-10 pins the message.
    let msg = match err {
        XcError::WgpuNoF64 { .. } => "GPU adapter lacks f64 support".to_string(),
        other => format!("{}", other),
    };

    // Build the exception with positional args (msg, code, kind). The Python
    // __init__ shim (python/xcfun_rs/__init__.py) unpacks these into
    // .args, .code, .kind on the exception instance.
    crate::XcfunError::new_err((msg, code, kind))
}
```

### Example C: Python `__init__.py` exception shim (D-09 attribute attachment, see §5.1)

```python
# crates/xcfun-py/python/xcfun_rs/__init__.py

"""xcfun_rs — Rust reimplementation of xcfun via PyO3."""

# Re-export everything from the native extension.
from ._native import (  # type: ignore[attr-defined]
    Functional, Mode, Vars,
    XcfunError as _XcfunErrorBase,
    version, splash, authors, self_test, is_compatible_library,
    which_vars, which_mode,
    enumerate_parameters, enumerate_aliases,
    describe_short, describe_long,
)

# D-09 — attach .code and .kind attributes via a thin Python wrapper.
# CRITICAL — this exists because abi3-py310 forbids subclassing PyException
# at the C level until Python 3.12 (see RESEARCH.md §5).

class XcfunError(_XcfunErrorBase):  # noqa: N818
    """Exception raised by xcfun_rs operations.

    Attributes:
        code:  C ABI error code per Phase 5 D-08-A
               {0: ok, 1: InvalidOrder, 2: InvalidVars, 4: InvalidMode,
                6: both, -1: UnknownName/other}.
        kind:  Rust XcError variant name as string
               (e.g. "InvalidVars", "WgpuNoF64").
    """

    code: int
    kind: str

    def __init__(self, *args):
        # Rust raises with positional args (msg, code, kind).
        if len(args) == 3:
            msg, self.code, self.kind = args
            super().__init__(msg)
        else:  # defensive — direct construction from Python
            super().__init__(*args)
            self.code = -1
            self.kind = "Unknown"

# Replace the bare native exception with the attribute-bearing subclass on
# the module so `raise xc.XcfunError(...)` and isinstance checks work.
# Note: Rust-side raises still use _XcfunErrorBase (the bare PyException
# subclass) — but Python catches up the chain via super().

__version__ = version()

__all__ = [
    "Functional", "Mode", "Vars", "XcfunError",
    "version", "splash", "authors", "self_test", "is_compatible_library",
    "which_vars", "which_mode",
    "enumerate_parameters", "enumerate_aliases",
    "describe_short", "describe_long",
    "__version__",
]
```

**Open question for plan-phase (LOW priority):** Python catches the bare `_XcfunErrorBase` via `except XcfunError` only if `XcfunError` is a strict subclass. The user attribute access (`e.code`, `e.kind`) requires the user to call `XcfunError(*args)` rather than the bare base. The cleanest workaround is to have the Rust side raise via Python's `__init__` — i.e., `crate::XcfunError::new_err((msg, code, kind))` constructs a `_XcfunErrorBase` instance in Rust, but Python's `except xcfun_rs.XcfunError as e` matches both base AND subclass identically (Python exception inheritance rules). The Plan should: (a) prototype both shapes, (b) lock in the one where `e.code` / `e.kind` are accessible from `except`. The pyclass-extends-PyException Python 3.12+ alternative (gated by feature flag `abi3-py312`) is documented as a v0.2 follow-up.

### Example D: pyproject.toml (PY-01)

```toml
# crates/xcfun-py/pyproject.toml

[build-system]
requires      = ["maturin>=1.12,<2.0"]
build-backend = "maturin"

[project]
name            = "xcfun_rs"
version         = "0.1.0"
description     = "Rust reimplementation of xcfun XC functional library; \
                   memory-safe, GPU-capable, bit-level parity with C++ xcfun"
readme          = "README.md"
license         = { text = "MPL-2.0" }
requires-python = ">=3.10"
authors = [
    { name = "xcfun_rs contributors" },
]
keywords = ["dft", "density-functional-theory", "computational-chemistry",
            "xc-functional", "rust", "gpu"]
classifiers = [
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Science/Research",
    "License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)",
    "Operating System :: POSIX :: Linux",
    "Operating System :: MacOS :: MacOS X",
    "Operating System :: Microsoft :: Windows",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Python :: 3.13",
    "Programming Language :: Rust",
    "Topic :: Scientific/Engineering :: Chemistry",
    "Topic :: Scientific/Engineering :: Physics",
]
dependencies = [
    "numpy>=1.26",
]

[project.optional-dependencies]
test = ["pytest>=7.0", "numpy>=1.26"]

[project.urls]
Homepage      = "https://github.com/<owner>/xcfun_rs"
Repository    = "https://github.com/<owner>/xcfun_rs"
Documentation = "https://github.com/<owner>/xcfun_rs#readme"
Changelog     = "https://github.com/<owner>/xcfun_rs/blob/master/CHANGELOG.md"
Issues        = "https://github.com/<owner>/xcfun_rs/issues"

[tool.maturin]
# Source: maturin guide §project_layout.md, §local_development.md.
# python-source places the Python pkg dir at crates/xcfun-py/python/xcfun_rs/.
# module-name names the cdylib produced by the Rust crate; placed at
# python/xcfun_rs/_native.<abi-tag>.so (auto-named by maturin from cdylib).
python-source = "python"
module-name   = "xcfun_rs._native"

# D-03 — CPU-only by default; GPU rebuilds via `maturin build --features hip|cuda|wgpu`.
features = ["pyo3/extension-module", "cpu"]

# Strip release binaries to keep wheel size down.
strip = true

# Per-feature wheel filename suffixes (PEP 440 local-version-identifier).
# These do NOT activate by default; users opt in by passing --features manually.
# Documented in the README; planner may add a `[tool.maturin.gpu-cuda]` profile
# block if maturin supports per-feature local-version-identifiers in 1.13.x
# (verify in plan-phase).
```

[CITED: maturin guide §project_layout.md "Mixed rust/python projects"]

### Example E: GitHub Actions release workflow shape (D-15 + D-16)

This is a real-world template adapted from the ast-grep wheel publish workflow. Plan-phase customizes per project specifics. Source — full YAML quoted in §7.

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  PACKAGE_NAME: xcfun_rs
  # abi3-py310 — ONE wheel covers Python 3.10, 3.11, 3.12, 3.13 per platform.
  PYTHON_VERSION: "3.10"

jobs:
  sdist:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: actions/setup-python@v6
        with: { python-version: "${{ env.PYTHON_VERSION }}" }
      - uses: PyO3/maturin-action@v1
        with:
          command: sdist
          args: --out dist --manifest-path crates/xcfun-py/Cargo.toml
      - uses: actions/upload-artifact@v7
        with: { name: wheels-sdist, path: dist }

  linux:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v6
      - uses: actions/setup-python@v6
        with: { python-version: "${{ env.PYTHON_VERSION }}" }
      - uses: PyO3/maturin-action@v1
        with:
          target: x86_64-unknown-linux-gnu
          manylinux: "2_28"     # explicit per maturin-action hardening guidance
          args: >-
            --release
            --out dist
            --manifest-path crates/xcfun-py/Cargo.toml
            --features cpu
      - run: |
          pip install dist/${{ env.PACKAGE_NAME }}-*.whl --force-reinstall
          pytest crates/xcfun-py/tests/ -q
      - uses: actions/upload-artifact@v7
        with: { name: wheels-linux, path: dist }

  macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v6
      - uses: actions/setup-python@v6
        with: { python-version: "${{ env.PYTHON_VERSION }}" }
      - uses: PyO3/maturin-action@v1
        with:
          target: aarch64-apple-darwin
          args: >-
            --release
            --out dist
            --manifest-path crates/xcfun-py/Cargo.toml
            --features cpu
      - uses: actions/upload-artifact@v7
        with: { name: wheels-macos, path: dist }

  windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v6
      - uses: actions/setup-python@v6
        with: { python-version: "${{ env.PYTHON_VERSION }}" }
      - uses: PyO3/maturin-action@v1
        with:
          target: x86_64-pc-windows-msvc
          args: >-
            --release
            --out dist
            --manifest-path crates/xcfun-py/Cargo.toml
            --features cpu
      - uses: actions/upload-artifact@v7
        with: { name: wheels-windows, path: dist }

  publish-pypi:
    name: Publish to PyPI (OIDC trusted publisher)
    needs: [sdist, linux, macos, windows]
    runs-on: ubuntu-latest
    environment: pypi
    permissions:
      id-token: write   # MANDATORY for OIDC trusted publishing
    steps:
      - uses: actions/download-artifact@v8
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist
      - uses: pypa/gh-action-pypi-publish@release/v1
        with:
          skip-existing: true
          packages-dir: dist
          verbose: true

  release-artifacts:
    name: Build libxcfun_capi for GitHub Release (D-16)
    needs: [linux, macos, windows]   # gate on wheel success
    strategy:
      matrix:
        include:
          - os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            artifact: libxcfun_capi.so
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: libxcfun_capi.dylib
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: xcfun_capi.dll
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v6
      - run: rustup target add ${{ matrix.target }}
      - run: cargo build --release -p xcfun-capi --target ${{ matrix.target }}
      - run: cargo run -p xtask --bin regen-capi-header --release
      - uses: actions/upload-artifact@v7
        with:
          name: libxcfun_capi-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/${{ matrix.artifact }}
            crates/xcfun-capi/include/xcfun.h

  github-release:
    name: Attach artifacts to GitHub Release
    needs: [release-artifacts, publish-pypi]
    runs-on: ubuntu-latest
    permissions:
      contents: write   # to create the GH release
    steps:
      - uses: actions/checkout@v6
      - uses: actions/download-artifact@v8
        with:
          pattern: libxcfun_capi-*
          merge-multiple: true
          path: artifacts
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            artifacts/libxcfun_capi.so
            artifacts/libxcfun_capi.dylib
            artifacts/xcfun_capi.dll
            artifacts/xcfun.h
          generate_release_notes: true
```

[CITED: ast-grep/ast-grep .github/workflows/pypi.yml; maturin-action README; docs.pypi.org/trusted-publishers/using-a-publisher]

### Example F: pytest harness against Rust-driver JSON fixtures

```rust
// crates/xcfun-py/examples/gen_py_fixtures.rs
//
// Runs the Phase 5 Functional facade against a fixed-seed grid and emits
// a JSON fixture file consumed by python tests. Committed to the repo.
//
// Run: cargo run -p xcfun-py --example gen_py_fixtures --release
// Output: crates/xcfun-py/tests/fixtures/eval_parity.json

use serde::Serialize;
use xcfun_rs::{Functional, Mode, Vars};

#[derive(Serialize)]
struct Fixture {
    functional: String,
    vars: u32,
    mode: u32,
    order: u32,
    density: Vec<f64>,
    expected: Vec<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixtures = Vec::new();
    // Use a small, hand-picked corpus — not the full 10k-grid (that's the
    // tier-2 validation harness's job). The Python pytest is a hermetic
    // smoke test for the binding layer, not a parity sweep.
    for &(name, vars, mode, order) in &[
        ("slaterx", Vars::A_B, Mode::PartialDerivatives, 0),
        ("pbex",    Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1),
        ("blyp",    Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 2),
        // ... ~10 functionals × 3 (vars, mode, order) tuples
    ] {
        let mut f = Functional::new();
        f.set(name, 1.0)?;
        f.eval_setup(vars, mode, order)?;
        let inlen  = f.input_buffer_length();
        let outlen = f.output_length()?;
        let density: Vec<f64> = (0..inlen).map(|i| 0.1 + (i as f64) * 0.05).collect();
        let mut expected = vec![0.0_f64; outlen];
        f.eval(&density, &mut expected)?;
        fixtures.push(Fixture {
            functional: name.to_string(),
            vars: vars as u32,
            mode: mode as u32,
            order,
            density, expected,
        });
    }
    let json = serde_json::to_string_pretty(&fixtures)?;
    std::fs::write("tests/fixtures/eval_parity.json", json)?;
    Ok(())
}
```

```python
# crates/xcfun-py/tests/test_parity.py
import json, pathlib, numpy as np, pytest
import xcfun_rs as xc

FIXTURE_PATH = pathlib.Path(__file__).parent / "fixtures" / "eval_parity.json"

@pytest.fixture(scope="module")
def fixtures():
    return json.loads(FIXTURE_PATH.read_text())

def test_parity(fixtures):
    for fx in fixtures:
        f = xc.Functional(
            fx["functional"],
            vars=xc.Vars(fx["vars"]),
            mode=xc.Mode(fx["mode"]),
            order=fx["order"],
        )
        density = np.asarray(fx["density"], dtype=np.float64).reshape(1, -1)
        out = f.eval_vec(density)
        np.testing.assert_allclose(
            out[0], fx["expected"], rtol=1e-12,
            err_msg=f"functional={fx['functional']} vars={fx['vars']} "
                    f"mode={fx['mode']} order={fx['order']}",
        )
```

The fixture generation is a one-time `cargo run` checked into the repo (no pytest-time C++ link). The pytest assert_allclose at `rtol=1e-12` matches the project's strict parity contract (Phase 5 D-08-A).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `PyO3 0.27` `Python::with_gil(...)` + `py.allow_threads(...)` | `PyO3 0.28` `Python::attach(...)` + `py.detach(...)` | 2026-01 (PyO3 0.28.0) | Source-level rename; `with_gil` and `allow_threads` retained as deprecated aliases. Plans should use the new names. |
| Long-lived PyPI API token in CI secret | OIDC trusted-publisher via `pypa/gh-action-pypi-publish@release/v1` | PyPI added trusted publisher 2023; maturin docs migrated 2024 | Better security; rotates without manual action. The maturin docs explicitly document the migration. |
| `maturin generate-ci github` produces a single workflow file | Per-project tailoring of the generated template (most projects in 2026 hand-roll release.yml) | Ongoing | The polars / ast-grep / pydantic-core release workflows are the de facto reference patterns; the canonical generate-ci output is a starting point only. |
| `cargo-workspaces publish --topological` (third-party) | First-party `cargo publish` workspace support landed in nightly 1.90+ | 2025-09 | Stable cargo doesn't support workspace publishing yet; v0.1.0 cannot rely on stable cargo's multi-package publish. The `xtask release-publish` driver is the right shape. |
| Auto-generated type stubs via `pyo3-stub-gen` | Hand-written `.pyi` for small surfaces; `pyo3-stub-gen` for large | 2025+ | xcfun-py's surface is small enough (< 100 LoC) to hand-write. |
| `#[pyclass(extends=PyException)]` works on all Pythons under abi3 | Forbidden under abi3 until Python 3.12 | PyO3 0.20+ | abi3-py310 forces the create_exception! + Python __init__ shim workaround. |

**Deprecated / outdated:**

- The maturin docs reference `maturin >=1.0,<2.0` widely; floor `1.12` in the project pin reflects abi3-py310 wheel-layout improvements that landed at 1.10 / 1.11. Pin floor at 1.12 (current 1.13.1) — older 1.x has known abi3 issues.
- `pyo3 0.28.0` and `pyo3 0.28.1` are yanked; the `=0.28.3` pin is load-bearing.
- `cargo publish --dry-run` does NOT validate that dependencies are publishable in topological order — the xtask must do that itself.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The Python source-side `class XcfunError(_XcfunErrorBase)` shim allows `except xcfun_rs.XcfunError as e: e.code` to work because Python exception catching matches by class identity OR ancestor. | §5.1 | If wrong, e.code access fails when the Rust side raises the bare base — Plan must prototype both shapes and lock the working one. Mitigated by including a pytest test for exactly this case. |
| A2 | `cargo search <crate> --limit 1` reliably reports the just-published version within ~30s of `cargo publish` returning success. | Pitfall 4 | If wrong, the xtask's polling-with-timeout falls back to a wall-clock sleep with retry — 5-minute cap. The user can resume from `--from <crate>` after manual investigation. |
| A3 | The polars / ast-grep wheel workflow's `manylinux: "2_28"` is the right default for xcfun_rs (avoids ancient glibc issues). | Example E | manylinux_2_28 covers RHEL 8+ / Ubuntu 20.04+ — ~99% of HPC + scientific Python users by 2026. If a downstream user reports "GLIBC_2.27 not found", bump to `manylinux: "2_24"` or `"auto"` in v0.2. |
| A4 | abi3-py310 wheels work on Python 3.14 (released 2026-04). | Example D classifiers list | abi3 forward-compatibility is the explicit Python ABI promise. PyO3 0.28.3 + maturin 1.13.x both target abi3 cleanly. Verified by Python 3.14 already installed in this dev env. |
| A5 | The `xcfun_rs/_native` cdylib name doesn't collide with anything else in the import path. | Example D `[tool.maturin] module-name` | Standard maturin pattern; pydantic-core / polars use `_core` / `_polars_rs`. Locked. |
| A6 | The 4 blocking HUMAN-UAT items (3, 4, 5, 6) can clear in serial within 1-2 days of human-supervised work. | Phase ordering | Item 3 is ~6h offline; items 4 + 5 are quick re-runs of the validation harness; item 6 is a one-line constant fix. Plan should sequence these as Wave 0 of Phase 7 (BEFORE the Python work starts). |
| A7 | GitHub's PyPI trusted-publisher OIDC config requires a dedicated `environment: pypi` on PyPI's side. | Pitfall 6 | The PyPI docs state "either with or without an environment". A bare workflow + repo + path-to-file triple works without environment. Including environment is recommended for defense-in-depth. |
| A8 | `xtask release-publish` driven from a maintainer's machine (NOT from CI) is acceptable. The `cargo publish` token comes from the maintainer's `~/.cargo/credentials.toml`. | D-15 | If the user wants CI-driven crates.io publish: add `CARGO_REGISTRY_TOKEN` secret + a separate "publish-crates" CI job. Documented as a v0.2 follow-up. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

(Table is non-empty — 8 documented assumptions. Most are LOW risk; A1 is the most important.)

## Open Questions

1. **`crates.io` vs `pypi` topological dependency on the publish day.** The Rust crates publish first (D-15 explicit DAG), then `maturin publish`. But maturin doesn't actually need the Rust crates to be on crates.io — it builds against the `path = "..."` workspace deps. Question: does the GitHub Actions release.yml workflow build wheels against the `path = "..."` deps (no crates.io required) or against the just-published crates.io versions?
   - Recommendation: build wheels against the path deps in the same workflow run (the wheel build doesn't depend on crates.io). This decouples the two pipelines and lets PyPI publish succeed even if crates.io is down. Plan-phase confirms this is what `maturin build` actually does (it should — workspace path deps take precedence).

2. **Should `xtask release-publish` be invoked from CI on tag push, or only manually from a maintainer's machine?**
   - CI-driven publishing is the modern pattern but requires a CI-side `CARGO_REGISTRY_TOKEN` secret. The user has not opted in to that. **Plan default:** maintainer-driven from local machine. Documented as a v0.2 enhancement.

3. **Is `python/xcfun_rs/__init__.pyi` enough, or do we need a separate `Functional.pyi` etc.?**
   - Rule: if the Python package is one module with one cdylib, ONE `.pyi` file is enough. xcfun_rs is exactly this shape. **Lock:** `python/xcfun_rs/__init__.pyi` only.

4. **Should `xcfun_rs.Backend` and `xcfun_rs.auto_backend()` be present-but-private (underscore prefix) or fully absent from Python?**
   - D-04 says "hidden". Implementer's pick. **Recommendation:** fully absent from `__all__` AND not exposed at the module level — even an underscore prefix invites users to depend on it. The READING side should explain that backend selection is via `XCFUN_FORCE_BACKEND` env var if required.

5. **Should `XcfunError(code=-1, kind='WgpuNoF64')` constructor accept `code=-1, kind="WgpuNoF64"` keyword args, or always go through the positional `(msg, code, kind)` shape?**
   - The Rust side raises with positional. Python users who construct one manually (rare; mostly tests) will go positional too. Document the public constructor as `XcfunError(message: str)`; `.code` and `.kind` are read-only attributes set during raise. Plan locks this in the `.pyi` stub.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (stable) | Cargo build | ✓ | 1.95.0 (≥ MSRV 1.85) | — |
| Python 3.10+ | maturin develop, pytest | ✓ | 3.14.4 (≥ 3.10 for abi3-py310) | — |
| `maturin` (CLI) | wheel build | ✗ | — | `pip install 'maturin>=1.12,<2.0'` (one-time) |
| `pip` | maturin install | ✓ | (system) | — |
| `cargo` | xtask release-publish dry-run | ✓ | 1.95.0 | — |
| `gh` (GitHub CLI) | release-tag verification | ✓ | (system) | — |
| Internet access for crates.io | xtask release-publish (`cargo publish`) | (assumed) | — | — |
| Internet access for PyPI | `maturin publish` / OIDC | (assumed) | — | — |
| GitHub Actions runners (Linux + macOS + Windows) | wheel matrix | (managed by GH; assumed available) | ubuntu-22.04, macos-latest, windows-latest | — |
| AMD/ROCm GPU | HUMAN-UAT items 1, 2 (D-14 SKIP) | ✗ | — | Skip to v0.2 per D-14 |
| SHADER_F64 Vulkan adapter | HUMAN-UAT items 1, 2 (D-14 SKIP) | ✗ | — | Skip to v0.2 per D-14 |

**Missing dependencies with no fallback:**
- None blocking — `maturin` is `pip install`-able; everything else is already on the dev machine or managed by GitHub Actions.

**Missing dependencies with fallback:**
- AMD/ROCm GPU — D-14 explicitly SKIPS items 1 + 2 to v0.2.

**Action items for plan-phase:**
- Add `pip install 'maturin>=1.12,<2.0'` to the developer setup docs / `crates/xcfun-py/README.md`.
- Add `crates/xcfun-py/.gitignore` entries for `dist/`, `target/`, `*.egg-info/`, `__pycache__/`.

## Validation Architecture

(Including this section — `workflow.nyquist_validation` is not explicitly false in `.planning/config.json`. Sampling Rate retained.)

### Test Framework

| Property | Value |
|----------|-------|
| Rust test framework | `cargo test` / `cargo nextest run` (existing project standard) |
| Python test framework | `pytest >= 7.0` (declared in `[project.optional-dependencies] test`) |
| Config files | `crates/xcfun-py/pyproject.toml` (build), `crates/xcfun-py/tests/conftest.py` (pytest fixtures) |
| Quick run command (Rust side) | `cargo test -p xcfun-py --no-default-features --features cpu` |
| Quick run command (Python side) | `cd crates/xcfun-py && maturin develop --release && pytest tests/ -q` |
| Full suite command | `bash scripts/phase-7-full-validation.sh` (NEW — runs Rust unit tests + builds wheel + installs into venv + runs pytest from the wheel) |
| Phase gate | All quick + full suite green BEFORE `git tag v0.1.0` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PY-01 | xcfun-py builds as PyO3 0.28 abi3-py310 extension | unit (build) | `cd crates/xcfun-py && maturin build --release --features cpu` | ❌ Wave 1 — `pyproject.toml` + Cargo.toml deps |
| PY-02 | Functional class methods (set/get/eval_setup/etc.) | unit | `pytest crates/xcfun-py/tests/test_functional.py -q` | ❌ Wave 2 |
| PY-03 | eval_vec strict zero-copy contract | unit | `pytest crates/xcfun-py/tests/test_eval_vec_zero_copy.py -q` | ❌ Wave 2 |
| PY-04 | 11 module-level free fns | unit | `pytest crates/xcfun-py/tests/test_smoke.py -q` | ❌ Wave 1 |
| PY-05 | XcError → XcfunError mapping with .code/.kind | unit | `pytest crates/xcfun-py/tests/test_xcfun_error.py -q` | ❌ Wave 2 |
| PY-06 | wheel build + install + pytest-from-wheel | integration | `.github/workflows/release.yml` matrix `linux \| macos \| windows` jobs | ❌ Wave 3 |
| (cross) | 1e-12 parity vs Rust facade | integration | `pytest crates/xcfun-py/tests/test_parity.py -q` | ❌ Wave 2 — depends on `examples/gen_py_fixtures.rs` running first |
| (D-14) | 4 blocking HUMAN-UAT items cleared | manual | (per-item per HUMAN-UAT.md) | — |
| (D-15) | xtask release-publish topological dry-run | unit | `cargo run -p xtask --bin release-publish -- --dry-run` | ❌ Wave 4 |
| (D-16) | GH Release artifacts attached | manual | inspect Release page after tag push | — |

### Sampling Rate

- **Per task commit:** `cargo nextest run -p xcfun-py` (Rust side) + `pytest crates/xcfun-py/tests/ -q` (Python side, after `maturin develop`).
- **Per wave merge:** add `cargo run -p xtask --bin release-publish -- --dry-run` to verify the topology hasn't drifted.
- **Phase gate:** Full suite green before tag push. Tag push triggers `release.yml`; the workflow itself is the final gate.

### Wave 0 Gaps

- [ ] `crates/xcfun-py/Cargo.toml` — add pyo3 + numpy + xcfun-rs deps (currently only xcfun-core)
- [ ] `crates/xcfun-py/pyproject.toml` — NEW; per Example D
- [ ] `crates/xcfun-py/python/xcfun_rs/__init__.py` — NEW; per Example C
- [ ] `crates/xcfun-py/python/xcfun_rs/__init__.pyi` — NEW; hand-written type stubs
- [ ] `crates/xcfun-py/python/xcfun_rs/py.typed` — NEW; empty marker file
- [ ] `crates/xcfun-py/src/lib.rs` — REWRITE; per Pattern 1
- [ ] `crates/xcfun-py/src/functional.rs` — NEW; per Pattern 2
- [ ] `crates/xcfun-py/src/numpy_io.rs` — NEW; per Example A
- [ ] `crates/xcfun-py/src/errors.rs` — NEW; per Example B
- [ ] `crates/xcfun-py/tests/conftest.py` — NEW; pytest fixtures
- [ ] `crates/xcfun-py/tests/{test_smoke,test_functional,test_eval_vec_zero_copy,test_xcfun_error,test_parity}.py` — NEW
- [ ] `crates/xcfun-py/tests/fixtures/eval_parity.json` — NEW; generated by `cargo run --example gen_py_fixtures`
- [ ] `crates/xcfun-py/examples/gen_py_fixtures.rs` — NEW; per Example F
- [ ] `crates/xcfun-py/README.md` — NEW; user-facing install + GPU rebuild docs
- [ ] `xtask/src/bin/release_publish.rs` — NEW; per D-15
- [ ] `xtask/Cargo.toml` — add a new `[[bin]] name = "release-publish"` entry
- [ ] `.github/workflows/release.yml` — NEW; per Example E
- [ ] `CHANGELOG.md` — NEW at repo root; Keep-a-Changelog format
- [ ] root `Cargo.toml` — drop `crates/xcfun-python` from `exclude`; add `crates/xcfun-py` to `members`; (optionally) add `numpy = "=0.28.0"` and `pyo3 = "=0.28.3"` to workspace `[workspace.dependencies]`
- [ ] `crates/xcfun-py/.gitignore` — NEW; standard Python + maturin artifacts

## Security Domain

Phase 7's security surface is the **publish chain**, not the runtime — the library doesn't accept network input or auth, parse user-supplied formats, etc. The relevant ASVS rows:

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | PyPI / crates.io OIDC trusted-publisher (no long-lived secrets in CI) |
| V3 Session Management | no | (no sessions; CLI / library) |
| V4 Access Control | yes | GitHub Actions `permissions: id-token: write` only on the publish job; `contents: write` only on the release-attach job |
| V5 Input Validation | yes | NumPy `is_c_contiguous` + `dtype` check at the eval_vec boundary (D-07) |
| V6 Cryptography | no | (no crypto; no auth secrets) |
| V14 (Configuration) | yes | `cargo audit` on every PR; `cargo deny check licenses advisories bans` on every push |

### Known Threat Patterns for {PyPI release pipeline}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Stolen long-lived API token uploads malicious wheel | Spoofing | OIDC trusted-publisher (D-15 + Pitfall 6) |
| Compromised dev machine pushes a tag | Tampering | OIDC publish only triggered by authenticated push to `refs/tags/v*` from the project repo; PyPI side restricts to that repo + workflow file path |
| Supply-chain attack via a transitive dep | Tampering | `cargo audit` blocks vulnerable versions; lockfile `Cargo.lock` checked in; `cargo deny check bans` enforces an allowlist |
| Wheel content tampering between build and upload | Tampering | GitHub Actions provides build provenance attestation (default for `actions/upload-artifact@v7+`) |
| Phony PyPI package with similar name | Repudiation | Dist name `xcfun_rs` registered atomically with the v0.1.0 publish; squat-resistant via underscore (D-02) |
| Yanked-but-recoverable bad release | Tampering | All wheels are content-addressed; PyPI yanks are non-destructive but not re-publishable (Pitfall 5 mitigation: dry-run before tag) |

## Sources

### Primary (HIGH confidence)

- [Context7 /pyo3/pyo3 — class.md, exception.md, function/signature.md, parallelism.md, module.md] — `#[pymodule]`, `#[pyclass]`, `create_exception!`, `py.detach`, `#[pyo3(signature=...)]` patterns; the abi3 + PyException limitation note.
- [Context7 /pyo3/rust-numpy — README.md, llms.txt] — `PyArray2<f64>` / `PyReadonlyArray2` / `PyArrayMethods` zero-copy patterns.
- [Context7 /pyo3/maturin — README.md, guide/src/distribution.md, guide/src/local_development.md, guide/src/project_layout.md] — `pyproject.toml` shape, `[tool.maturin]`, `python-source` + `module-name`, `maturin generate-ci github`, OIDC trusted-publisher migration notes.
- [https://docs.pypi.org/trusted-publishers/using-a-publisher/] — exact `permissions: id-token: write` + `pypa/gh-action-pypi-publish@release/v1` pattern.
- [https://keepachangelog.com/en/1.1.0/] — CHANGELOG.md section conventions (Added/Changed/Deprecated/Removed/Fixed/Security; YYYY-MM-DD).
- [https://docs.rs/numpy/0.28.0/numpy/trait.PyUntypedArrayMethods.html] — `is_c_contiguous` / `strides` / `dtype` / `shape` / `ndim` method signatures.
- [crates.io] — `pyo3 = "0.28.3"`, `numpy = "0.28.0"`, `maturin = "1.13.1"` as of 2026-05-05 (CLI verified).

### Secondary (MEDIUM confidence)

- [https://github.com/PyO3/maturin-action] — action inputs (`command`, `args`, `target`, `manylinux`, `maturin-version`, `working-directory`, `sccache`); hardening guidance ("set explicit manylinux version").
- [https://www.maturin.rs/distribution] — recommended `maturin generate-ci github` flow + the trusted-publishing modifications (remove `MATURIN_PYPI_TOKEN`, add `id-token: write`).
- [Real-world workflow templates] — ast-grep, polars, pydantic-core, astral-sh/uv release.yml shapes.

### Tertiary (LOW confidence — flagged for plan-phase verification)

- [Assumption A1] — `class XcfunError(_XcfunErrorBase)` Python-source shim correctly intercepts Rust-raised bare-base exceptions for `except XcfunError` matching. **Plan must prototype.**
- [Assumption A2] — `cargo search` polling reliably reports just-published crate versions within ~30s. May need wall-clock fallback.

## Metadata

**Confidence breakdown:**
- Standard stack (pyo3 / numpy / maturin pins): HIGH — versions verified on crates.io 2026-05-05.
- PyO3 0.28 API surface (pymodule / pyclass / create_exception / py.detach): HIGH — Context7 docs current.
- Architecture (PY-01..06 mapping): HIGH — every requirement has a concrete pattern.
- §5 abi3 + PyException CRITICAL FINDING: HIGH — confirmed via two independent sources (PyO3 guide §exception.md upstream + Context7 docs); the workaround is documented but a single Plan task should prototype both shapes (#[pyclass(extends)] under abi3-py312 vs create_exception! + Python shim under abi3-py310).
- Wheel matrix CI YAML: MEDIUM — derived from real-world ast-grep workflow; Plan-phase tailors to project specifics.
- `xtask release-publish` design: MEDIUM — D-15 specifies the topology; Pitfall 4 documents the polling shape but Plan-phase implements + tests.
- HUMAN-UAT clearance scheduling: HIGH — D-14 is unambiguous about which 4 items block.
- v0.1.0 framing rationale: HIGH — D-13 is unambiguous; cubecl-pre.3 + 4 UAT items justify.
- pytest harness shape: HIGH — Example F is concrete and self-contained.

**Research date:** 2026-05-05
**Valid until:** 2026-06-05 (rate of change in Python wheel ecosystem — maturin and PyO3 release every 4-8 weeks; if this RESEARCH.md is stale at execution time, re-verify pyo3 / numpy / maturin versions and re-fetch the PyO3 0.28.x release notes for any abi3 / panic-policy changes).

---

## §2. Wheel build & packaging concrete shape (PY-01 + PY-06)

### §2.1 `crates/xcfun-py/Cargo.toml` (after rename per D-01)

```toml
[package]
name = "xcfun-py"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "Python bindings for xcfun_rs (PyO3 0.28 + rust-numpy 0.28)"
license = "MPL-2.0"

[lib]
crate-type = ["cdylib"]
# This is the cdylib NAME (the resulting .so/.dylib/.pyd basename). maturin
# pulls this from `[tool.maturin] module-name = "xcfun_rs._native"` in
# pyproject.toml — NOT from this `[lib].name` field. Keeping `[lib].name`
# unset (defaulting to "xcfun_py") prevents accidental ambiguity.

[features]
# D-03 — CPU-only by default. GPU rebuilds opt-in via the corresponding
# xcfun-rs feature flag.
default = ["cpu"]
cpu  = ["xcfun-rs/cpu"]
hip  = ["xcfun-rs/hip"]
cuda = ["xcfun-rs/cuda"]
wgpu = ["xcfun-rs/wgpu"]
metal = ["xcfun-rs/metal"]   # transparent alias for wgpu

[dependencies]
xcfun-rs   = { path = "../xcfun-rs", default-features = false }
xcfun-core = { path = "../xcfun-core" }
pyo3  = { version = "=0.28.3", features = ["extension-module", "abi3-py310"] }
numpy = { version = "=0.28.0" }

[dev-dependencies]
serde      = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }

[[example]]
name = "gen_py_fixtures"
path = "examples/gen_py_fixtures.rs"
```

### §2.2 `crates/xcfun-py/pyproject.toml` (per Example D above)

See Example D for the full file.

## §3. PyO3 module skeleton (PY-01, PY-02, PY-04)

(See Patterns 1 / 2 / 3 above.)

### §3.5 Free-fn pattern (PY-04 — applied to all 11 free fns)

```rust
// crates/xcfun-py/src/functional.rs (continued)

pub mod free_fns {
    use pyo3::prelude::*;
    use xcfun_rs as rs;

    /// xcfun_rs.version() -> str
    #[pyfunction]
    pub fn version() -> &'static str { rs::version() }

    /// xcfun_rs.splash() -> str
    #[pyfunction]
    pub fn splash() -> &'static str { rs::splash() }

    /// xcfun_rs.authors() -> str
    #[pyfunction]
    pub fn authors() -> &'static str { rs::authors() }

    /// xcfun_rs.is_compatible_library() -> bool
    #[pyfunction]
    pub fn is_compatible_library() -> bool { rs::is_compatible_library() }

    /// xcfun_rs.self_test() -> int (failure count)
    #[pyfunction]
    pub fn self_test() -> i32 { rs::self_test() }

    #[pyfunction]
    pub fn which_vars(
        func_type: u32, dens_type: u32,
        laplacian: u32, kinetic: u32,
        current: u32, explicit_derivatives: u32,
    ) -> Option<u32> {
        rs::which_vars(func_type, dens_type, laplacian, kinetic, current,
                       explicit_derivatives).map(|v| v as u32)
    }

    #[pyfunction]
    pub fn which_mode(mode_type: u32) -> Option<u32> {
        rs::which_mode(mode_type).map(|m| m as u32)
    }

    #[pyfunction] pub fn enumerate_parameters(p: i32) -> Option<&'static str> { rs::enumerate_parameters(p) }
    #[pyfunction] pub fn enumerate_aliases   (n: i32) -> Option<&'static str> { rs::enumerate_aliases(n)    }
    #[pyfunction] pub fn describe_short      (name: &str) -> Option<&'static str> { rs::describe_short(name) }
    #[pyfunction] pub fn describe_long       (name: &str) -> Option<&'static str> { rs::describe_long(name)  }
}
```

[CITED: pyo3 guide §function/index.md "Function exposition"]

## §4. NumPy strict zero-copy `eval_vec` (PY-03 + D-07 + D-08)

(See Example A above for the full code excerpt.)

### §4.1 The contract in plain English

- INPUT: `np.ndarray[np.float64]`, shape `(nr_points, inlen)`, `flags['C_CONTIGUOUS'] == True`. Anything else → `TypeError`.
- OUTPUT: a fresh `np.ndarray[np.float64]`, shape `(nr_points, outlen)`, owning its data. Returned by value to Python.

### §4.2 Why we raise instead of silent-`ascontiguousarray`

Silent coercion masks a 2× memory cost from the user. `np.ascontiguousarray` allocates and copies the entire density buffer if it's strided; for a 100k-point GGA grid (5 doubles/point = 4 MB) that's a noticeable allocation hit on the hot path. The TypeError is loud-fail-fast — the user does the explicit coercion at the call site or restructures their data layout.

### §4.3 Why the OUTPUT is fresh-allocated, not zero-copy

D-08 pins this. The OUTPUT shape `(nr_points, outlen)` depends on the functional set + `(vars, mode, order)` — Python callers don't pre-allocate it. Returning a fresh array sidesteps the `out=` kwarg complexity entirely (deferred to v0.2 per Deferred Ideas). The returned array IS zero-copy in the sense that its data pointer IS the Rust-allocated buffer (via `IntoPyArray` / `PyArray2::zeros` + direct write). It's "allocate-then-fill" — not "copy-the-rust-result-into-numpy".

## §5. CRITICAL FINDING — abi3-py310 vs `#[pyclass(extends=PyException)]` (PY-05 + D-09)

### §5.1 The collision

D-09 specifies a single `XcfunError` exception class with `.code: int` and `.kind: str` attributes. The naïve PyO3 implementation is:

```rust
// THIS DOES NOT WORK UNDER abi3-py310 ON PYTHON 3.10/3.11.
#[pyclass(extends=PyException)]
struct XcfunError {
    #[pyo3(get)] code: i32,
    #[pyo3(get)] kind: String,
}
```

PyO3's upstream documentation explicitly says ([source: github.com/PyO3/pyo3 guide/src/exception.md]):

> "Note that when the `abi3` feature is enabled, subclassing `PyException` is only possible on Python 3.12 or greater."

The reason: `PyType_FromMetaclass` (the CPython API required to subclass exception types under the limited / abi3 ABI) was introduced in Python 3.12. Earlier Python versions don't have it. PyO3's macro expansion includes a `#[cfg(any(not(Py_LIMITED_API), Py_3_12))]` guard that excludes the subclass implementation from abi3-py310 builds.

CLAUDE.md pins `pyo3` features `["extension-module", "abi3-py310"]`. **The pin is not negotiable — it's an explicit project pin AND the wheel's compatibility guarantee.**

### §5.2 The workaround — `create_exception!` + Python `__init__` shim

The maintained pattern (used by `polars`, `pyarrow`, `cryptography`):

1. **Rust side:** declare the exception via `create_exception!(xcfun_rs, XcfunError, PyException)` — this creates a bare `PyException` subclass at the C level using only the abi3-stable `PyErr_NewException` API, which works on every Python version.

2. **Rust side:** the conversion `From<XcError> for PyErr` builds the exception with positional args `(message, code, kind)`:
   ```rust
   crate::XcfunError::new_err((msg, code, kind))   // tuple → exception args
   ```

3. **Python side (`python/xcfun_rs/__init__.py`):** subclass the bare exception in pure Python and override `__init__` to unpack `(message, code, kind)` into `.args`, `.code`, `.kind` attributes:
   ```python
   class XcfunError(_XcfunErrorBase):
       def __init__(self, *args):
           if len(args) == 3:
               msg, self.code, self.kind = args
               super().__init__(msg)
           else:
               super().__init__(*args)
               self.code = -1; self.kind = "Unknown"
   ```

4. **The user-facing `xcfun_rs.XcfunError` is the Python subclass.** Users do `except xcfun_rs.XcfunError as e: print(e.code)` — and Python's exception-matching rules (subclass also matches base) catch the Rust-raised `_XcfunErrorBase` correctly. The catch handler instance HAS `.code` and `.kind` attributes because it's built via the shim.

### §5.3 Open question A1 — does the catch path actually work?

**The risk:** if the Rust side raises `_XcfunErrorBase` (the bare native class) directly, Python's `except XcfunError` (the Python subclass) catches it via base-class matching — but the caught instance is `_XcfunErrorBase`, NOT `XcfunError`, so it has no `.code` / `.kind`.

**The mitigation:** the Rust side raises the **subclass type** by importing the Python shim. PyO3 supports this via `Py_None.get_type::<XcfunError>().call((msg, code, kind), None)`. Plan-phase MUST prototype both shapes:

- Shape (a): Rust raises `_XcfunErrorBase`; Python catches via subclass match; access `.code` requires re-raising as the subclass.
- Shape (b): Rust raises by importing the Python `xcfun_rs.XcfunError` and constructing it directly.
- Shape (c): the create_exception! macro accepts an `__init__` body via `#[pymethods]` on the bare class — verify in PyO3 0.28.3 docs whether attributes can be attached to a `create_exception!` class via `#[pymethods]` without subclassing PyException at compile time.

The Plan ships ONE shape with a pytest test that verifies `except XcfunError as e: assert e.code == 2; assert e.kind == 'InvalidVars'` works end-to-end.

### §5.4 Future migration — `abi3-py312`

When the project drops Python 3.10 + 3.11 support (likely v0.3 or v0.4), the pin becomes `abi3-py312`. At that point `#[pyclass(extends=PyException)]` works directly and the Python shim can be deleted. v0.1.0 sticks with abi3-py310 + the Python shim — explicit, documented in CHANGELOG.md.

[CITED: github.com/PyO3/pyo3 guide/src/exception.md "Subclassing PyException"; PEP 384 "Defining a Stable ABI"]

## §6. Maturin commands (PY-06)

```bash
# Local development — install into the active venv with debug symbols.
cd crates/xcfun-py
maturin develop                           # debug build
maturin develop --release                 # release build, MUCH faster

# Build a wheel for the current platform (no install).
maturin build --release --features cpu --out dist/

# Build for a specific Python interpreter (abi3 means one wheel covers all
# CPython >= 3.10, so this is mostly for non-abi3 builds).
maturin build --release --features cpu -i python3.10 --out dist/

# Cross-compile to a different target (CI uses this; rarely on a dev machine).
maturin build --release --features cpu --target x86_64-pc-windows-msvc \
              --out dist/

# GPU rebuild — user opts in to one of these.
maturin build --release --features hip  --out dist/   # AMD/ROCm
maturin build --release --features cuda --out dist/   # NVIDIA
maturin build --release --features wgpu --out dist/   # Vulkan/Metal/WebGPU

# Publish to PyPI (manual; CI uses pypa/gh-action-pypi-publish instead).
maturin publish --skip-existing
```

[CITED: maturin guide §local_development.md, §distribution.md]

## §7. GitHub Actions wheel matrix (D-15 wheel side; D-16 release-artifact side)

(See Example E above for the full workflow.)

### §7.1 Why one workflow file (not per-platform)

A single `release.yml` keeps the artifact-passing logic (sdist + 3 wheels → publish-pypi → release-artifacts → github-release) in one place. Splitting per-platform creates artifact orphan-risk where one job's wheels never reach the publish job.

### §7.2 Why `manylinux: "2_28"` explicitly (not `auto`)

[CITED: maturin-action README hardening section]: "set an explicit `manylinux:` version for each target to prevent silent regressions". `2_28` covers RHEL 8 / Ubuntu 20.04+ which is essentially every modern HPC + scientific Python user.

### §7.3 PyPI Trusted Publisher OIDC setup (one-time, on PyPI side)

Before the first release:
1. Visit https://pypi.org/manage/account/publishing/.
2. Add a "pending" trusted publisher with:
   - PyPI Project Name: `xcfun_rs`
   - Owner: `<github-owner>`
   - Repository name: `<github-repo>`
   - Workflow filename: `release.yml`
   - Environment name: `pypi` (must match `environment: pypi` in the workflow)
3. The first successful publish converts pending → confirmed.

[CITED: docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/]

## §8. pytest harness (PY-06 cross-validation gate)

(See Example F above for the harness shape.)

### §8.1 Why JSON-fixture not C++-cross-check

The Python pytest is a **binding-layer integrity test**, not a parity sweep. The parity sweep is the tier-2 / tier-3 validation harness in `validation/` (already shipped). At Python test time we want:

- Hermetic — no C++ compilation needed.
- Fast — < 30s for full Python suite.
- Independent of validation/ — Python test failures should not block validation/ runs.

The JSON fixture is a bridge: Rust driver computes expected output once, commits to the repo. Python tests assert `numpy.allclose(rust_result, expected, rtol=1e-12)`. If the Rust facade itself drifts, the parity sweep catches it; the Python pytest catches the binding layer.

### §8.2 Fixture corpus size

Small — ~10 (functional, vars, mode, order) tuples × 1 density each. Total fixture file < 100 KB. Rationale: the binding layer doesn't do math; it forwards calls. ~10 representative tuples + a strict 1e-12 tolerance is enough to catch a binding mismatch (e.g., wrong int conversion, wrong slice length).

## §9. `xtask release-publish` topology (D-15)

```
[Computed at runtime from `cargo metadata --format-version 1`]

xcfun-ad ─► xcfun-core ─► xcfun-kernels ─► xcfun-eval ─► xcfun-gpu ─► xcfun-rs ─► xcfun-capi
                                                                                         │
                                                                                         ▼
                                                                              maturin publish (xcfun-py)
```

**Algorithm (pseudocode):**

```rust
// xtask/src/bin/release_publish.rs
//
// Topological cargo publish driver. Reads `cargo metadata`, computes the
// publishable-crate dependency DAG (filtering out path-only / private
// crates), and shells out to `cargo publish -p <crate>` in topological
// order, polling `cargo search` between calls for index propagation.
//
// Modes:
//   --dry-run          — print the publish order; no side effects.
//   --execute          — actually publish. Idempotent: skips crates whose
//                        current Cargo.toml version is already on crates.io.
//   --from <crate>     — resume from a partially-published run.
//   --skip <crate>...  — skip these crates (debug-only).
//
// On any per-crate failure: print which crates have shipped, which haven't,
// and exit non-zero. The user re-runs with `--from <next-crate>`.

fn main() -> anyhow::Result<()> {
    let args = parse_cli();
    let metadata = run("cargo metadata --format-version 1")?;
    let crates = topological_order(&metadata);   // filters publish=false / no version field
    println!("Publish order: {:?}", crates.iter().map(|c| &c.name).collect::<Vec<_>>());
    if args.dry_run { return Ok(()); }

    for c in crates {
        if args.from.as_ref().map_or(false, |f| c.name != *f && !already_passed(f, c)) {
            continue;
        }
        if is_published_at_version(&c.name, &c.version)? {
            println!("[skip] {}@{} already on crates.io", c.name, c.version);
            continue;
        }
        run(&format!("cargo publish --dry-run -p {}", c.name))?;
        if !args.execute {
            println!("[dry-run] would publish {}@{}", c.name, c.version);
            continue;
        }
        run(&format!("cargo publish -p {}", c.name))?;
        wait_for_index_propagation(&c.name, &c.version)?;   // poll cargo search up to 5min
    }

    println!("All Rust crates published. Now run:");
    println!("  cd crates/xcfun-py && maturin publish --skip-existing");
    Ok(())
}
```

The `--execute` is mandatory for actual publishing; `--dry-run` is the default. CI on tag-push could optionally invoke this — but per Open Question 2 the v0.1.0 release uses maintainer-driven local invocation. Documented in `.planning/phases/07-python-bindings-release/RELEASING.md` (a new release-runbook the Plan can produce as part of Wave 4).

## §10. CHANGELOG.md (D-13)

The Plan creates a new `CHANGELOG.md` at the repo root in Keep-a-Changelog 1.1.0 format.

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

(empty after the v0.1.0 cut)

## [0.1.0] - 2026-MM-DD

### Added

- Initial public release of the Rust-from-scratch reimplementation of xcfun.
- 78 functionals across LDA / GGA / metaGGA tiers (Phases 2–4).
- 50+ aliases + 4 parameters (Phase 4).
- 31 `Vars` arms × 3 evaluation modes (`PartialDerivatives` orders 0..=4,
  `Potential`, `Contracted` orders 0..=4) per Phase 3 + Phase 4.
- `xcfun-rs` native Rust facade `Functional` + 11 free fns (Phase 5).
- `xcfun-capi` C ABI drop-in replacement for `xcfun-master/api/xcfun.h`
  (cdylib + staticlib + rlib triple-crate; Phase 5 CAPI-01..07).
- `xcfun-py` Python bindings via PyO3 0.28.3 + rust-numpy 0.28.0 (Phase 7).
- CPU/GPU `Batch` lifecycle for `nr_points >= 64` via `cubecl` 0.10-pre.3
  (Phase 6 RS-08 + GPU-01..08).
- ROCm/HIP primary GPU backend; CUDA / Metal opt-in feature flags;
  Wgpu portable fallback at relaxed 1e-9 tolerance.
- `xtask release-publish` topological publish driver (Phase 7 D-15).

### Notes

- This is a 0.x release — the API may evolve before 1.0. cubecl is at
  `=0.10.0-pre.3` (a pre-release); a stable 1.0 SLA is contradictory
  with that dep.
- 4 Phase-6 HUMAN-UAT items deferred to v0.2:
  - ROCm tier-3 1e-13 hardware sweep (no AMD/ROCm GPU on dev runner).
  - Wgpu tier-3 1e-9 hardware sweep (no SHADER_F64 adapter).
- Wheel matrix covers `linux x86_64`, `macos arm64`, `windows x86_64`.
  Linux aarch64 / macOS x86_64 / Windows aarch64 deferred to v0.2.

[Unreleased]: https://github.com/<owner>/xcfun_rs/compare/v0.1.0...HEAD
[0.1.0]:      https://github.com/<owner>/xcfun_rs/releases/tag/v0.1.0
```

[CITED: keepachangelog.com/en/1.1.0/]

---

## RESEARCH COMPLETE
