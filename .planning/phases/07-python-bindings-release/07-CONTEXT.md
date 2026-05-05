# Phase 7: Python Bindings (`xcfun-py`) + Release - Context

**Gathered:** 2026-05-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 7 is the public-distribution layer + the v0.1.0 release ceremony. Two deliverable axes stack on top of the already-shipped `xcfun-rs::Functional` (Phase 5) and `xcfun-gpu`-backed `eval_vec` (Phase 6 RS-08 + D-16 pitched signature):

1. **`xcfun-py` PyO3 0.28 extension module.** Wrap `xcfun-rs::Functional` + the 11 free functions as a Python module `xcfun_rs` (six requirements PY-01..06). Exposes `Functional` class with `set` / `get` / `is_gga` / `is_metagga` / `eval_setup` / `user_eval_setup` / `input_length` / `output_length` / `eval` / `eval_vec`; module-level free functions (`version`, `splash`, `describe_*`, `enumerate_*`, `which_*`, `self_test`, `is_compatible_library`); `XcfunError` exception class. NumPy 2-D `f64` C-order zero-copy on `eval_vec`. abi3-py310 wheel covers CPython ≥ 3.10. Build via `maturin >=1.12,<2.0`; CI ships wheels for {Linux x86_64, macOS arm64, Windows x86_64}.

2. **Release ceremony for v0.1.0.** Multi-crate `cargo publish` of the seven library crates in topological dependency order via an `xtask release-publish` binary; PyPI wheel publish via `maturin publish`; GitHub Release tag `v0.1.0` with source tarball + `xcfun.h` + pre-built `libxcfun_capi.{so, dylib, dll}` for the three primary platform triples; CHANGELOG.md aligned to the Keep-a-Changelog format; semver-flexible v0.x framing (cubecl is `=0.10.0-pre.3` — a stable 1.0 SLA on top of a pre-release dep is contradictory, plus four offline-runnable Phase-6 HUMAN-UAT items must clear first).

**In scope:** Cargo crate rename `crates/xcfun-python/` → `crates/xcfun-py/` + workspace member promotion; PyO3 module wiring (Functional class + 11 free fns + `XcfunError` exception + `Mode` / `Vars` enums exposed); eager Python constructor with `eval_setup` at construction; strict NumPy zero-copy contract on `eval_vec` (raise on non-f64 / non-C-contiguous); single `XcfunError` class with `.code` (matches Phase 5 D-08-A `as_c_code` i32) + `.kind` (variant-name str) attributes; CPU-only PyPI wheel (no GPU runtime libs); GPU rebuild via `maturin build --features hip|cuda|wgpu` documented in README; pyproject.toml at `crates/xcfun-py/pyproject.toml`; pytest harness against Rust-driver-computed expected values; CI wheel matrix on GitHub Actions; clearance of four Phase-6 HUMAN-UAT items (mpmath fixture regen, Plan 06-N1 verify, Plan 06-N3 verify, BR_Q_PREFACTOR_F64 typo) BEFORE tagging; `xtask release-publish` topological publish driver; PyPI trusted-publisher OIDC setup; GitHub Release artifact pipeline.

**Out of scope (deferred to v0.2 / v0.x patch / out-of-project):** v1.0.0 hard semver lock (cubecl pre-release dep blocks); ROCm hardware tier-3 1e-13 sweep + Wgpu hardware tier-3 1e-9 sweep (no AMD/SHADER_F64 hardware in dev environment — v0.2 hardware-CI cycle); per-backend separate PyPI distributions (`xcfun_rs_cuda` / `xcfun_rs_rocm` rejected); all-in-one GPU-enabled wheel (rejected — non-portable across machines without toolkits); `out=` kwarg on `eval_vec` (numpy-ufunc-style buffer reuse); Linux aarch64 / macOS x86_64 / Windows aarch64 wheels (v0.2 wheel matrix expansion); pre-built `libxcfun_capi` for additional triples; runtime panic catching via `catch_unwind` shim (PyO3 default `PanicException` retained); explicit Python `Backend` enum (hidden — `auto_backend()` is server-side only; `XCFUN_FORCE_BACKEND` env var still honored); `Backend` enum exposure in `WgpuNoF64` payload (dropped per user); `adapter_name` in `WgpuNoF64` exception payload (dropped per user — no payload at all on the Python side); patches to `xcfun-master/` C++ source (vendored content-hash invariant preserved).

</domain>

<decisions>
## Implementation Decisions

### Crate / package / module naming

- **D-01:** **Rename `crates/xcfun-python/` → `crates/xcfun-py/`.** `git mv` + update `package.name = "xcfun-py"` + add to workspace `members` (currently in `exclude`). Matches `docs/design/05-module-responsibilities.md`, `ROADMAP.md` Phase 7 success-criterion text, the project's existing short-suffix convention (`xcfun-ad`, `xcfun-core`, `xcfun-kernels`, `xcfun-eval`, `xcfun-gpu`, `xcfun-rs`, `xcfun-capi`), and the Phase 5 D-01 precedent (`xcfun-ffi → xcfun-capi`). The current stub `crates/xcfun-python/` is empty (`lib.rs` has only a comment) — rename cost is near-zero.
- **D-02:** **PyPI distribution name `xcfun_rs` (underscore).** Python module / import name is also `xcfun_rs`. Resolves the PROJECT.md inconsistency (`pip install xcfun_rs` AND `pip install xcfun-rs` appeared) on the underscore side. Avoids dash/underscore normalization confusion that PEP 503 introduces; matches the modern PyO3 wheel convention (`polars`, `pyarrow`, `tokenizers`).

### GPU-feature exposure

- **D-03:** **CPU-only default PyPI wheel.** maturin build flags: `--no-default-features --features cpu`. Smallest binary; broadest install compatibility (no CUDA / ROCm / Vulkan runtime required at install time). Document in `crates/xcfun-py/README.md` how users rebuild for GPU: `maturin build --release --features hip` (or `cuda` / `wgpu`). Matches PROJECT.md "no CUDA/Wgpu by default in the wheel — CPU only for out-of-the-box portability".
- **D-04:** **`xcfun-gpu::Backend` enum is HIDDEN from the Python surface.** `eval_vec` does not accept a `backend=` kwarg. Backend selection is via Rust-side `auto_backend()` (Phase 6 D-07 priority order); CPU-only default wheel only ever resolves to `Backend::Cpu` anyway. The `XCFUN_FORCE_BACKEND` env-var override (Phase 6 D-07) is still honored for power-user benchmarking — documented in README, not in a Python kwarg.

### Python API shape

- **D-05:** **Eager Python constructor.** `Functional('pbe', vars=Vars.A_B, mode=Mode.PartialDerivatives, order=2)` performs `eval_setup` at construction time (Pythonic — constructors fully initialize). For the rare case the caller wants a delayed configure, expose `Functional('pbe').configure(vars=..., mode=..., order=...)` as a low-level escape hatch. `Functional("pbe")` alone (no vars/mode/order kwargs) is also legal — leaves the Functional in its constructed-but-not-set-up state, requiring `configure(...)` before any `eval`/`eval_vec`.
- **D-06:** **`set()` mutates in place, returns `None`.** `f.set('exx', 0.25)` matches Rust `&mut self` semantics + Python `dict.update`-style. Errors raise `XcfunError` immediately. Aliases compose additively via repeated `set` calls (e.g., `f.set('b3lyp', 1.0); f.set('slaterx', 0.5)`) per Phase 4 alias-engine semantics.
- **D-07:** **NumPy strict zero-copy or raise.** `eval_vec(densities)` accepts ONLY `np.ndarray[np.float64]` with `flags['C_CONTIGUOUS']`. Otherwise raises `TypeError("xcfun_rs.eval_vec: densities must be float64 C-contiguous; got dtype={...}, strides={...}, flags={...}")`. Documents PY-03 zero-copy contract loud-and-clear. Users explicitly do `np.ascontiguousarray(d.astype(np.float64))` if needed — predictable performance, no silent copy.
- **D-08:** **`eval_vec` allocates and returns.** Returns a freshly-allocated 2-D `np.ndarray[float64]` shaped `(nr_points, output_length)`. The "zero-copy" in PY-03 refers to the OUTPUT being directly the Rust-side buffer exposed via numpy's array_interface (no data copy on the Python ← Rust path). No `out=` kwarg in v0.1.0 — see Deferred Ideas.

### Error mapping

- **D-09:** **Single `XcfunError` exception class** with attributes `.code: int` (matches Phase 5 D-08-A `as_c_code` mapping {Ok:0, InvalidOrder:1, InvalidVars:2, InvalidMode:4, both:6, UnknownName/other:-1}) + `.kind: str` (Rust variant name as string, e.g. `'InvalidVars'`, `'WgpuNoF64'`). Caller pattern: `if e.code == 2:` or `if e.kind == 'InvalidVars':`. Mirrors stdlib `OSError.errno` / `OSError.strerror`. Single-class shape minimizes API surface to version + maintain.
- **D-10:** **`WgpuNoF64` Python payload: NONE.** Surfaces as `XcfunError(code=-1, kind='WgpuNoF64')` with no extra Python-side payload. The Rust `XcError::WgpuNoF64 { adapter_name: &'static str, requested_runtime: Backend }` retains both fields server-side (used by Rust-side `tracing` / debug logs); they DO NOT cross the Python boundary. `adapter_name` was explicitly dropped per user preference. `requested_runtime` is dropped consequentially (Backend is hidden per D-04). The exception message string is fixed: `"GPU adapter lacks f64 support"`. CPU-only default wheel rarely hits this anyway.
- **D-11:** **PyO3 default panic policy.** Rust panics in `xcfun-py` surface as `pyo3.PanicException` (subclass of `BaseException`). No `catch_unwind` shim. Matches every other PyO3-based scientific Python library (numpy / polars / pyarrow). Production callers don't see panics from a correctly-used API; tests catch via `except BaseException`.
- **D-12:** **Constructor raises eagerly on invalid `(vars, mode, order)`.** Per D-05 the constructor calls `eval_setup` at construction; bad combinations raise `XcfunError` from the constructor itself (e.g., `Functional('pbe', vars=Vars.A_B, mode=Mode.Potential, order=2)` raises `XcfunError(code=4, kind='InvalidMode')` because PBE doesn't support Potential at order 2). Fail-fast at the bad call site; matches standard Python where constructors validate args.

### Release ceremony

- **D-13:** **Initial release version `v0.1.0`.** Matches the workspace `Cargo.toml` `version = "0.1.0"` already pinned. Pre-1.0 framing — "API may evolve, semver MAY have breaking changes 0.1 → 0.2". Justified by: (a) `cubecl =0.10.0-pre.3` is itself a pre-release — a stable 1.0 SLA on top of an unstable dep is contradictory; (b) the 6 Phase-6 HUMAN-UAT items are not all clear yet (see D-14); (c) ACC-04 mpmath amendment hasn't run a full sweep against newly-restored `xcfun-master/` HEAD `a89b783`. Standard cautious-first-release pattern (tokio / polars / cubecl all lived in 0.x for years).
- **D-14:** **HUMAN-UAT items 3 / 4 / 5 / 6 BLOCK v0.1.0; items 1 / 2 SKIP to v0.2.** From `06-HUMAN-UAT.md`:
  - **(3) MPMATH ground-truth fixture regen ~6h offline** — BLOCK. Runnable now; closes Plan 06-N2 26-functional manual lane. Subsequent `--reference mpmath` strict-1e-13 sweep must be GREEN for the 13 non-SCAN/non-BR functionals before tag.
  - **(4) Plan 06-N1 11-functional auto-tightening verify** — BLOCK. xcfun-master restored at HEAD `a89b783`; re-run `cargo run -p validation --release -- --backend cpu --order 3` and confirm strict 1e-12 GREEN for PBEINTC / BECKESRX / P86C / P86CORRC / PW91C / SPBEC / APBEC / B97C / B97_1C / B97_2C / PW91K.
  - **(5) Plan 06-N3 18-functional auto-tightening verify** — BLOCK. Same sweep covers M05/M06×10 + B97-X×3 + LYPC + VWN_PBEC + PW92C + PBEC + OPTX at strict 1e-13.
  - **(6) `BR_Q_PREFACTOR_F64` typo fix in `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs:37`** — BLOCK. Pre-existing correctness bug; `0.699_390_040_064_282_6` → `0.699_291_115_553_117_4` (verified `1/((2/3)·π^(2/3))` at f64 + mpmath@200). BRX/BRC/BRXC mpmath smoke must pass at strict 1e-13.
  - **(1) ROCm tier-3 1e-13 hardware sweep** — SKIP (no AMD/ROCm GPU on cloud-CI runner). Document as "best-effort, validated in v0.2 hardware-CI cycle".
  - **(2) Wgpu tier-3 1e-9 hardware sweep (excluding ERF)** — SKIP (no SHADER_F64 adapter). Same v0.2 deferral.
- **D-15:** **Topological `cargo publish` automated by `xtask release-publish`.** New xtask binary `cargo run -p xtask --bin release-publish` walks the dep DAG: `xcfun-ad` → `xcfun-core` → `xcfun-kernels` → `xcfun-eval` → `xcfun-gpu` → `xcfun-rs` → `xcfun-capi`, sleeping ~30 s between each `cargo publish` for crates.io index propagation; idempotent (`cargo search` + skip-if-already-published-at-this-version). Final step: `maturin publish --skip-existing` for the `xcfun-py` wheel matrix. Doesn't replace `RELEASING.md` documentation — it implements what the doc describes.
- **D-16:** **GitHub Release artifacts.** Tag `v0.1.0` triggers a GitHub Actions release-build workflow that produces:
  - Source tarball (auto-generated from the tag).
  - `xcfun.h` (the cbindgen-generated header from `crates/xcfun-capi/include/xcfun.h`).
  - Pre-built `libxcfun_capi.so` (Linux x86_64), `libxcfun_capi.dylib` (macOS arm64), `libxcfun_capi.dll` + import lib (Windows x86_64), each with `RUSTFLAGS=""` + `-fno-fast-math` discipline retained.

  Defends the "drop-in C ABI replacement for `xcfun-master/api/xcfun.h`" positioning (CAPI-* requirements) for C/C++ consumers without a Rust toolchain.

### Claude's Discretion

- Concrete `pyproject.toml` content (build-backend = `maturin`; ABI3 metadata; classifiers; URLs) — standard maturin patterns.
- Type-stub strategy: hand-written `.pyi` co-located with the wheel vs auto-generated. Recommend hand-written for v0.1 since `Functional` has well-defined signatures and the file is small.
- Mode / Vars Python enum exposure: pyo3 `#[pyclass(eq, eq_int)]` IntEnum vs `enum.StrEnum`. Recommend IntEnum (matches u32 representations the C ABI uses).
- Free-function placement: module top-level (`xcfun_rs.version()`) vs `Functional` classmethods. Recommend module top-level — matches Rust facade in `xcfun-rs/src/free_fns.rs`.
- pytest harness shape: pytest fixtures load Rust-driver-computed expected values from JSON committed to the repo (no C++ at Python test time) vs vendored xcfun-master ctypes cross-check. Recommend the JSON fixture path — keeps Python tests hermetic; the Rust + C++ cross-check is already enforced by `validation/` tier-2.
- CHANGELOG.md format: Keep-a-Changelog (https://keepachangelog.com/) is the standard.
- PyPI publish auth: PyPI Trusted Publisher (GitHub OIDC) — modern recommendation; rotates better than long-lived API tokens.
- Wheel matrix concrete shape: `manylinux_2_28_x86_64` (per cibuildwheel default), `macosx_11_0_arm64`, `win_amd64`. `abi3-py310` means one wheel covers CPython 3.10 / 3.11 / 3.12 / 3.13 per platform.
- Whether `xcfun_rs.Backend` / `xcfun_rs.auto_backend()` get a `# private` underscore prefix or are simply absent from `__all__` (D-04 says hidden — implementer picks the absence form).
- Wheel filename suffix for GPU rebuilds (e.g., `xcfun_rs-0.1.0+cuda12-cp310-abi3-linux_x86_64.whl`) — the `+cuda12` local-version-identifier is PEP 440 conformant.
- Linker stripping / debug-info stripping for the released `libxcfun_capi.{so,dylib}` to keep the GitHub Release artifacts small.

### Folded Todos

None — the one pending todo (`2026-04-26-phase-3-uat-test-1-order-3-full-matrix-tier-2-capstone-re-ru.md`) is a Phase-3 follow-up, out of Phase 7 scope.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner, executor) MUST read these before planning or implementing.**

### Phase 7 success criteria + requirements (the spec)

- `.planning/ROADMAP.md` §"Phase 7: Python Bindings (`xcfun-py`) + Release" — 5 success criteria. **THE SPEC for this phase.**
- `.planning/REQUIREMENTS.md` — PY-01..06 statements (verbatim wording for each requirement).
- `.planning/PROJECT.md` — `pyo3 0.28 + rust-numpy 0.28 with zero-copy f64 numpy arrays` active requirement; "no CUDA/Wgpu by default in the wheel — CPU only for out-of-the-box portability"; "separate `xcfun-rs-cuda` wheel can be built with the `cuda` feature enabled" (now resolved per D-03 = single CPU wheel + maturin-build instructions, no separate distribution).

### Rust facade being wrapped (the surface to expose)

- `crates/xcfun-rs/src/lib.rs` — `Functional`, `XCFUN_MIN_BATCH_SIZE`, `min_batch_size()` re-exports + 11 free fns + `Mode` / `Vars` / `XcError` / `ParameterId` / `FunctionalId` / `Dependency` re-exports from `xcfun-core`.
- `crates/xcfun-rs/src/functional.rs` — `Functional` newtype with `set` / `get` / `is_gga` / `is_metagga` / `eval_setup` / `user_eval_setup` / `input_length` / `input_buffer_length` / `output_length` / `eval` / `eval_vec` (line 188-471). PY-02 maps each to a Python method.
- `crates/xcfun-rs/src/free_fns.rs` — `version` / `splash` / `authors` / `is_compatible_library` / `self_test` / `which_vars` / `which_mode` / `enumerate_parameters` / `enumerate_aliases` / `describe_short` / `describe_long`. PY-04 maps each to a module-level Python function.
- `crates/xcfun-core/src/error.rs` (XcError) — 9 Copy variants + `as_c_code()` mapping (Phase 5 D-08-A); D-09 / D-10 here transcribe this to the Python `XcfunError` shape.
- `crates/xcfun-gpu/src/lib.rs` (Backend, auto_backend) — D-04 hides this from Python; **DO NOT** re-export to Python.

### C++ reference (algorithmic-identity source of truth — relevant only to test fixtures)

- `xcfun-master/api/xcfun.h` — drop-in target for the C ABI; relevant to Phase 7 only as the source of pre-built libxcfun_capi headers in the GitHub Release artifact (D-16). Phase 5's `xcfun-capi/include/xcfun.h` is already byte-matched to this.
- `xcfun-master/test/test.cpp` — the C++ reference's test fixtures; Python pytest may consume the same `(functional, vars, mode, order, density, expected)` tuples, computed once via the Rust driver at test-build time and committed as JSON.

### Design docs (project-internal contracts — UPDATE during Phase 7)

- `docs/design/03-api-surface.md` — RS / PY surface contract. **UPDATE:** add a "PY-API" section transcribing D-05..D-08 + D-09..D-12.
- `docs/design/05-module-responsibilities.md` — crate boundaries. **UPDATE:** §xcfun-py renamed from xcfun-python; clarify CPU-only PyPI wheel + GPU-feature-flag-rebuild model per D-03.
- `docs/design/08-error-model.md` — `XcError` variants + Python `XcfunError` mapping. **UPDATE:** add Python-side mapping table per D-09 / D-10 / D-11.
- `docs/design/10-build-and-dependencies.md` — pyo3 / rust-numpy / maturin pins. **UPDATE:** xcfun-py member promotion; CI wheel matrix; PyPI trusted-publisher OIDC setup; `xtask release-publish` topology.
- `docs/design/11-process-and-milestones.md` — milestone M9 (Python + release) is Phase 7. **UPDATE:** record the v0.1.0 framing per D-13 + the four blocking UAT items per D-14.

### Project-wide planning artefacts

- `.planning/STATE.md` — current position (`Phase 06 Complete with caveats`); Phase 7 will advance this to `Complete (released)` post-tag.
- `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` — D-12 (EvalHandle reusable handle), D-13 / D-13-A (`XcError::WgpuNoF64` payload Copy contract; D-10 here drops both fields at the Python boundary), D-14 (`XCFUN_MIN_BATCH_SIZE` env override; D-04 here keeps it Rust-side-only), D-16 (`eval_vec` pitched signature; D-07 / D-08 here align Python NumPy interface with it), D-17 (`weights: Vec`; nothing to do at Python layer), D-18 (LDA-vars=6 / DensVars-driven dispatch; nothing to do at Python layer).
- `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-HUMAN-UAT.md` — 6 follow-up items; D-14 here selects {3, 4, 5, 6} as v0.1.0 blockers and {1, 2} as v0.2 deferrals.
- `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md` — D-01 (ffi → capi rename precedent; D-01 here mirrors with python → py), D-02 (Functional newtype facade — what xcfun-py wraps), D-08-A (XcError::as_c_code i32 mapping; D-09 here transcribes), D-15 (cdylib + staticlib + rlib triple — relevant for the GitHub Release pre-built libxcfun_capi artifacts in D-16).

### CLAUDE.md (root) version pins

- `CLAUDE.md` "Python bindings" table: `pyo3 = "=0.28.3"` (features `["extension-module", "abi3-py310"]`), `numpy = "=0.28.0"`, `maturin >=1.12, <2.0`. **THESE ARE LOCKED — do not bump in Phase 7.**
- `CLAUDE.md` "Stack Patterns by Variant" §Python — describes `pip install xcfun-rs` (CPU only by default); D-03 keeps this exact behaviour.

### PyO3 / maturin / rust-numpy reference (verify during plan-phase research)

- `https://pyo3.rs/v0.28.3/` — PyO3 0.28.3 user guide; `#[pyclass]`, `#[pymethods]`, `#[pymodule]`, exception classes via `create_exception!`.
- `https://docs.rs/numpy/0.28.0/numpy/` — rust-numpy 0.28.0 docs; `PyArray2<f64>`, `PyReadonlyArray2`, `PyArrayMethods::as_slice` zero-copy access.
- `https://www.maturin.rs/` — maturin 1.13.x documentation; `pyproject.toml` shape; `[tool.maturin]` configuration; abi3 wheel layout.
- `https://github.com/PyO3/maturin-action` — recommended GitHub Actions wheel-build action; multi-platform matrix patterns.
- `https://docs.pypi.org/trusted-publishers/` — PyPI trusted publisher (OIDC) setup for GitHub Actions.
- `https://keepachangelog.com/en/1.1.0/` — CHANGELOG.md format.

### Crates.io publish reference

- `https://doc.rust-lang.org/cargo/commands/cargo-publish.html` — `cargo publish` semantics; `--dry-run`; index propagation timing.
- `https://semver.org/` — v0.x semver flexibility (D-13 framing).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`crates/xcfun-python/`** stub directory — `Cargo.toml` has `package.name = "xcfun-python"` + `description = "Python bindings for xcfun_rs via PyO3"` + only `xcfun-core` as a dep; `src/lib.rs` is a single-line comment "Python bindings for xcfun_rs via PyO3 (Phase 7+)". **Plan 07-00 D-01 renames the directory + package; Plan 07-01 starts the actual PyO3 module wiring atop this skeleton.**
- **`crates/xcfun-rs::Functional` + free_fns** — the entire surface to wrap. Already Send + Sync (Phase 5 D-17), strict zero-alloc plumbing scaffolded (Phase 6 D-12 EvalHandle), `eval_vec` GPU dispatch wired (Phase 6 D-14 + RS-08). Python layer is a thin pyo3 boilerplate over this.
- **`crates/xcfun-core::XcError`** — 9 Copy variants + `WgpuNoF64 { adapter_name: &'static str, requested_runtime: Backend }` + `as_c_code() -> i32` (Phase 5 D-08-A); D-09 here directly transcribes to Python `XcfunError(code, kind)`.
- **`xtask/`** — Phase 2 D-21 + Phase 5 D-09 establish the `regen-* --check` pattern; D-15 here adds `xtask release-publish` as a NEW xtask binary in the same idiom. `cc 1.2.60` for compiling C tests already in `validation/` Cargo deps.
- **`crates/xcfun-capi/`** — Phase 5 produces `cdylib` + `staticlib` + `rlib` triple at `target/release/libxcfun_capi.{so,a,rlib}`. D-16 here packages these (plus the `dylib` / `dll` cross-platform variants) for the GitHub Release.
- **`crates/xcfun-capi/include/xcfun.h`** — the cbindgen-generated, byte-matched-to-upstream-`xcfun.h` header. D-16 attaches this verbatim to the GitHub Release.
- **`validation/`** — tier-2 / tier-3 parity harness; the "block on (4)+(5) auto-tightening verify" UAT item from D-14 runs `cargo run -p validation --release -- --backend cpu --order 3` here. No new validation code in Phase 7; just running the existing harness.

### Established Patterns

- **`xtask regen-* --check` drift gate** (Phase 2 D-21, Phase 5 D-09) — applied to `xtask release-publish` for `--dry-run` semantics: print the publish plan, exit non-zero if a crate isn't ready (e.g., uncommitted changes, version not bumped). Only `--execute` actually publishes.
- **Workspace `members` / `exclude` migration** (Phase 5 D-01, Phase 6 Plan 06-02a) — Plan 07-00 D-01 follows: drop `crates/xcfun-python` from `exclude`, add `crates/xcfun-py` to `members` after the rename.
- **`#[non_exhaustive]` + Copy on `XcError`** (Phase 2 D-25) — preserved through the Python boundary; D-10 here drops the WgpuNoF64 payload at the Rust→Python conversion shim, NOT at the Rust source level.
- **Strict zero-allocation per-point `eval`** (Phase 5 D-13 + Phase 6 D-12) — Python `eval()` benefits transparently; PyO3 doesn't add allocations on the hot path beyond what the GIL release already requires.

### Integration Points

- **`xcfun-py` depends on:** `xcfun-rs` (Functional + free fns), `pyo3 =0.28.3`, `numpy =0.28.0`. Optional feature-flag deps: `xcfun-rs/cpu` (default), `xcfun-rs/hip`, `xcfun-rs/cuda`, `xcfun-rs/wgpu` — pulls the corresponding cubecl-runtime crate transitively. NO direct cubecl deps at the Python layer.
- **PyPI wheel does NOT depend on:** `xcfun-capi` (Python users go via `xcfun-rs` → `xcfun-eval` → ...; the C ABI is for non-Python C/C++ embedders). The `libxcfun_capi.{so,dylib,dll}` binaries are GitHub Release artifacts (D-16), NOT bundled with the Python wheel.
- **Phase 7 → External (PyPI / crates.io / GitHub Releases):** publish topology (D-15) walks the workspace dep DAG; PyPI trusted-publisher OIDC means the GitHub Action (no long-lived secrets) signs the wheels.
- **`xtask release-publish` reads:** workspace `Cargo.toml` to enumerate crates, runs `cargo metadata --format-version 1 | jq` to compute the topological order, and shells out to `cargo publish -p <crate> --dry-run` (or `--execute`) per crate.

### Pitfalls (from prior phases)

- **`anyhow` in any library crate** is CI-blocked (`xtask check-no-anyhow`). `xcfun-py` joins the enforced set when added to workspace members. PyO3-side error mapping uses `thiserror` + `pyo3::exceptions::PyException::new_err` patterns, NOT `anyhow`.
- **`-Cfast-math` / `RUSTFLAGS` reassociation** breaks 1e-12 / 1e-13 parity. The released `libxcfun_capi.{so,dylib,dll}` binaries (D-16) MUST be built with `RUSTFLAGS=""` + `-fno-fast-math` (clang/MSVC equivalents) — verify in the GitHub Actions release workflow.
- **cubecl pre-release version pin** (Phase 1+) — D-13 framing rests on this: cubecl `=0.10.0-pre.3` is a pre-release dep, so a 1.0.0 SLA on top of it is contradictory.
- **PyO3 0.28.0 / 0.28.1 yanked** (CLAUDE.md note) — `=0.28.3` pin is load-bearing; do NOT loosen to caret.
- **`numpy 0.28.x` MUST track `pyo3 0.28.x`** atomically (CLAUDE.md "Version Compatibility" matrix). A future `pyo3 0.29` upgrade requires a coordinated `numpy 0.29` upgrade + abi3 wheel rebuild — out of v0.1 scope; v0.2+ concern.
- **Python wheel filename includes platform tag** — `manylinux_2_28_x86_64` / `macosx_11_0_arm64` / `win_amd64`. Wrong tag = unusable wheel. CI matrix shape per Claude's discretion.
- **Crates.io 30-day download cap on yanks** — if a publish goes wrong, yank is irreversible-by-publish-of-same-version. `xtask release-publish` MUST `cargo publish --dry-run` first; `--execute` only after dry-run is GREEN.

</code_context>

<specifics>
## Specific Ideas

- **The Python module name is `xcfun_rs`** (underscore), not `xcfun_py` and not `xcfun`. `import xcfun_rs as xc` is the canonical form. The Cargo crate name is `xcfun-py` (hyphen, project's short-suffix convention).
- **`adapter_name` and `requested_runtime` payload on `WgpuNoF64` exception are dropped at the Python boundary** — the user explicitly does not want them. Rust-side `XcError::WgpuNoF64` retains both fields for `tracing` / debug logs.
- **Backend enum is hidden from Python** — `auto_backend()` runs Rust-side. CPU-only PyPI wheel never resolves to non-CPU anyway. `XCFUN_FORCE_BACKEND` env override remains documented as an escape hatch.
- **NumPy strictness is intentional** — silent `np.ascontiguousarray` copies hide perf footguns; the `TypeError` raise-with-helpful-message form makes the contract loud. PY-03's "zero-copy 2-D ndarray" is the contract; users meet it explicitly or get a TypeError.
- **v0.1.0, not v1.0.0.** Multiple reasons: cubecl pre-release dep, four offline-runnable HUMAN-UAT items not yet cleared, two hardware-gated UAT items deferred. Standard ecosystem cautious-first-release pattern.
- **The four blocking UAT items (3 / 4 / 5 / 6) are runnable on the dev environment** — no hardware gating. Plan order should sequence them BEFORE the publish ceremony (e.g., Plan 07-N0 = UAT clearance, Plan 07-NX = release).
- **MPMATH fixture regen ~6h offline** — schedule as an overnight run; the wall-clock cost is real but it only needs to succeed once before tag.
- **GitHub Release artifacts (D-16) cover {Linux x86_64, macOS arm64, Windows x86_64}** — three triples cover the bulk of the C-consumer audience. Linux aarch64 / macOS x86_64 / Windows aarch64 deferred to v0.2.

</specifics>

<deferred>
## Deferred Ideas

- **`out=` kwarg on `eval_vec`** — numpy-ufunc-style buffer reuse (`np.add(a, b, out=c)` precedent). v2 if perf-sensitive callers ask. Not in v0.1.
- **Per-backend separate PyPI distributions** (`xcfun_rs_cuda`, `xcfun_rs_rocm`) — rejected per D-03 in favor of single CPU wheel + maturin-rebuild docs.
- **All-in-one GPU-enabled wheel** — rejected per D-03 (wheel non-portable across machines without CUDA / ROCm runtime libs).
- **ROCm tier-3 1e-13 hardware sweep** — v0.2 hardware-CI cycle. No AMD/ROCm GPU on cloud-CI runner.
- **Wgpu tier-3 1e-9 hardware sweep (excluding ERF)** — v0.2 hardware-CI cycle. No SHADER_F64 adapter.
- **Linux aarch64 / macOS x86_64 / Windows aarch64 wheels** — v0.2 wheel matrix expansion.
- **Pre-built `libxcfun_capi` for additional triples** (linux-aarch64, macos-x86_64, win-aarch64) — v0.2 GitHub Release artifact expansion.
- **`Backend` enum exposure to Python** — D-04 hides it; v2+ if users specifically ask for runtime-side backend selection from Python (most won't given the CPU-only PyPI default wheel).
- **`adapter_name` payload on `WgpuNoF64` exception in Python** — explicitly dropped per user. Rust-side retains the field; not crossing the boundary.
- **`requested_runtime` payload on `WgpuNoF64`** — dropped consequentially with Backend hiding (D-04 / D-10).
- **`catch_unwind` shim around PyO3 entries** — D-11 inherits PyO3 default; no shim. v2+ if a downstream user reports panic-leakage causing Python-process crashes (unlikely; PyO3 already handles via PanicException).
- **v1.0.0 hard semver lock** — gated on cubecl shipping a stable 0.10 + all 6 HUMAN-UAT items clearing + a downstream-feedback cycle. Out of v0.1.0 scope.
- **Patches to `xcfun-master/` C++ source** — vendored content-hash invariant preserved (Phase 1 D-18).
- **PyPI publish via long-lived API token** — modern recommendation is OIDC trusted publisher; long-lived tokens are a fallback only.
- **Type stubs auto-generated from PyO3** — recommend hand-written `.pyi` for v0.1; revisit if the API surface grows.
- **Python `@dataclass`-style configuration objects** for parameters (EXX, RANGESEP_MU, CAM_ALPHA, CAM_BETA) — current set/get-by-name pattern is sufficient; v2+ if the parameter count grows.

### Reviewed Todos (not folded)

None — no pending todos surfaced in cross-reference scan that match Phase 7 scope. The one pending todo (`2026-04-26-phase-3-uat-test-1-order-3-full-matrix-tier-2-capstone-re-ru.md`) is Phase 3-specific and out of Phase 7 scope.

</deferred>

---

*Phase: 07-python-bindings-release*
*Context gathered: 2026-05-05*
