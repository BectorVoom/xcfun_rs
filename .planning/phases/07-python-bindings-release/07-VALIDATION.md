---
phase: 7
slug: python-bindings-release
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-05
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source-of-truth: `07-RESEARCH.md` §"Validation Architecture" (lines 1283–1342).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Rust test framework** | `cargo nextest run` (project standard) — covers Rust-side unit + the parity-fixture generator |
| **Python test framework** | `pytest >= 7.0` (declared in `crates/xcfun-py/pyproject.toml::[project.optional-dependencies].test`) |
| **Build / packaging tool** | `maturin >= 1.12, < 2.0` (current 1.13.1) — `[build-system] build-backend = "maturin"` |
| **Config files** | `crates/xcfun-py/pyproject.toml` (build + pytest config), `crates/xcfun-py/tests/conftest.py` (shared fixtures + JSON-fixture loader), `crates/xcfun-py/Cargo.toml` (deps + abi3 features) |
| **Quick run command (Rust side)** | `cargo nextest run -p xcfun-py --no-default-features --features cpu` |
| **Quick run command (Python side)** | `cd crates/xcfun-py && maturin develop --release && pytest tests/ -q` |
| **Full suite command** | `bash scripts/phase-7-full-validation.sh` (NEW — Rust unit tests, then `maturin build --release`, then `pip install dist/*.whl` into a fresh venv, then `pytest crates/xcfun-py/tests/ -q` from the wheel install) |
| **Estimated runtime** | ~8 s Rust unit + ~12 s pytest (post-`maturin develop`); ~2 min full wheel-rebuild suite locally; ~12 min CI matrix end-to-end |
| **Phase gate** | Quick + full suite GREEN on all three platforms (Linux x86_64 / macOS arm64 / Windows x86_64) BEFORE `git tag v0.1.0` is pushed |

---

## Sampling Rate

- **After every task commit:** Rust side `cargo nextest run -p xcfun-py` + Python side `pytest crates/xcfun-py/tests/ -q` (after `maturin develop --release` if pyo3 surface changed).
- **After every plan wave:** add `cargo run -p xtask --bin release-publish -- --dry-run` once Wave 4 lands — verifies the publish topology hasn't drifted.
- **Before `/gsd-verify-work 7`:** Full suite GREEN locally on dev box; CI matrix `release.yml` GREEN on at least one branch push (dry-run mode).
- **Phase gate:** Tag push triggers `.github/workflows/release.yml`; the workflow's wheel-build + `pytest`-from-wheel matrix on {Linux x86_64, macOS arm64, Windows x86_64} is the final gate before PyPI publish.
- **Max feedback latency:** ~20 s for the per-task quick run (Rust + Python combined, GIL release in `eval_vec` keeps Python pytest fast).

---

## Per-Task Verification Map

This map is provisional — Plan-phase will refine task IDs once plans are written. Each phase requirement is mapped to the wave/test-type/automated-command pair that proves it.

| Task ID (TBD) | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 7-01-XX | 07-00 | 0 | (D-14 #6 typo fix) | — | BR_Q_PREFACTOR_F64 = 0.699_291_115_553_117_4 (mpmath@200 verified) | unit | `cargo nextest run -p xcfun-kernels br_q_prefactor` + `cargo run -p validation --release -- --backend cpu --filter '^BR.*' --order 2` | ❌ W0 | ⬜ |
| 7-02-XX | 07-00 | 0 | (D-14 #4 + #5 verify) | — | strict 1e-12 GREEN on 11+18 functionals on restored xcfun-master HEAD a89b783 | integration | `cargo run -p validation --release -- --backend cpu --order 3 --filter '<list>'` | ❌ W0 | ⬜ |
| 7-03-XX | 07-00 | 0 | (D-14 #3 mpmath regen) | — | mpmath ground-truth fixtures regenerated on 26 functionals (~6h offline) | manual+integration | `python validation/mpmath_sidecar/regen_all.py` then `cargo run -p validation -- --reference mpmath` | ❌ W0 | ⬜ |
| 7-04-XX | 07-01 | 0 | (D-01) | — | crate rename `xcfun-python → xcfun-py` + workspace member promotion | unit | `cargo build -p xcfun-py --no-default-features --features cpu` + `grep -F 'crates/xcfun-py' Cargo.toml` | ❌ W0 | ⬜ |
| 7-05-XX | 07-02 | 1 | PY-01 | — | xcfun-py builds as PyO3 0.28 abi3-py310 extension | unit (build) | `cd crates/xcfun-py && maturin build --release --features cpu --quiet && python -c "import xcfun_rs; print(xcfun_rs.__file__)"` | ❌ W1 | ⬜ |
| 7-06-XX | 07-02 | 1 | PY-04 | — | 11 module-level free fns reachable | unit | `pytest crates/xcfun-py/tests/test_smoke.py -q` | ❌ W1 | ⬜ |
| 7-07-XX | 07-03 | 2 | PY-02 | — | Functional class methods (set/get/eval_setup/etc.) | unit | `pytest crates/xcfun-py/tests/test_functional.py -q` | ❌ W2 | ⬜ |
| 7-08-XX | 07-03 | 2 | PY-05 | — | Single XcfunError class with `.code: int` + `.kind: str` (abi3 §5 workaround) | unit | `pytest crates/xcfun-py/tests/test_xcfun_error.py -q` (must run on Python 3.10 AND 3.11 to lock the workaround) | ❌ W2 | ⬜ |
| 7-09-XX | 07-04 | 2 | PY-03 | T-7-V5 | eval_vec strict zero-copy contract; raise TypeError on non-f64/non-C-contig | unit | `pytest crates/xcfun-py/tests/test_eval_vec_zero_copy.py -q` | ❌ W2 | ⬜ |
| 7-10-XX | 07-05 | 2 | (cross-validation) | — | 1e-12 parity vs Rust facade on a stratified fixture set | integration | `cargo run --example gen_py_fixtures` + `pytest crates/xcfun-py/tests/test_parity.py -q` | ❌ W2 | ⬜ |
| 7-11-XX | 07-06 | 3 | PY-06 | T-7-V14 | wheel build + install + pytest-from-wheel passes on Linux/macOS/Windows | integration (CI) | `.github/workflows/release.yml` matrix `linux \| macos \| windows` `build-and-test-wheels` jobs (push to feature branch) | ❌ W3 | ⬜ |
| 7-12-XX | 07-07 | 4 | (D-15) | T-7-V14 | xtask release-publish topological dry-run idempotent | unit | `cargo run -p xtask --bin release-publish -- --dry-run` (must list crates in topological order, exit 0) | ❌ W4 | ⬜ |
| 7-13-XX | 07-08 | 4 | (D-16) | T-7-V4 | GH Release artifacts attached on tag push | manual | inspect Release page after `release.yml` runs; verify `xcfun.h` + 3× `libxcfun_capi.{so,dylib,dll}` attached | ❌ W4 | ⬜ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

The 18 file/code gaps below come straight from `07-RESEARCH.md` §"Wave 0 Gaps" (lines 1322–1342) and are mandatory for the test infrastructure to compile + run:

- [ ] `crates/xcfun-py/Cargo.toml` — add pyo3 = "=0.28.3" (features `["extension-module", "abi3-py310"]`) + numpy = "=0.28.0" + xcfun-rs (path) deps; currently only xcfun-core stub-dep
- [ ] `crates/xcfun-py/pyproject.toml` — NEW; per Example D in 07-RESEARCH.md (build-backend = "maturin"; abi3 metadata; classifiers; `requires-python = ">=3.10"`; `[project.optional-dependencies] test = ["pytest>=7", "numpy>=2"]`)
- [ ] `crates/xcfun-py/python/xcfun_rs/__init__.py` — NEW; per Example C — implements the abi3 §5 workaround Python-side shim that grafts `.code`/`.kind` onto the bare native `_XcfunError` exception class (pre-3.12 compatibility)
- [ ] `crates/xcfun-py/python/xcfun_rs/__init__.pyi` — NEW; hand-written type stubs covering `Functional`, `XcfunError`, `Mode`, `Vars`, 11 free fns
- [ ] `crates/xcfun-py/python/xcfun_rs/py.typed` — NEW; empty marker file (PEP 561 compliance)
- [ ] `crates/xcfun-py/src/lib.rs` — REWRITE; per Pattern 1 (`#[pymodule]` skeleton; registers `Functional`, `XcfunError`, `Mode`, `Vars`, all free fns)
- [ ] `crates/xcfun-py/src/functional.rs` — NEW; per Pattern 2 (`#[pyclass]` wrapping `xcfun_rs::Functional`)
- [ ] `crates/xcfun-py/src/numpy_io.rs` — NEW; per Example A (PyReadonlyArray2 + is_c_contiguous strict check)
- [ ] `crates/xcfun-py/src/errors.rs` — NEW; per Example B (`create_exception!` + `XcError` → `_XcfunError` mapping; `as_c_code` re-uses Phase 5 D-08-A)
- [ ] `crates/xcfun-py/tests/conftest.py` — NEW; pytest fixtures (eval_parity.json loader; per-platform skips)
- [ ] `crates/xcfun-py/tests/test_smoke.py` — NEW; PY-04 (11 free-fn smoke calls)
- [ ] `crates/xcfun-py/tests/test_functional.py` — NEW; PY-02 (constructor + set/get/eval/eval_setup/user_eval_setup/input_length/output_length/is_gga/is_metagga)
- [ ] `crates/xcfun-py/tests/test_eval_vec_zero_copy.py` — NEW; PY-03 (zero-copy round-trip + TypeError-on-non-C-contig + TypeError-on-non-f64)
- [ ] `crates/xcfun-py/tests/test_xcfun_error.py` — NEW; PY-05 + abi3 §5 workaround (.code/.kind access path; must pass on 3.10, 3.11, 3.12)
- [ ] `crates/xcfun-py/tests/test_parity.py` — NEW; cross-language parity (1e-12 rtol vs Rust driver fixture)
- [ ] `crates/xcfun-py/tests/fixtures/eval_parity.json` — NEW; generated by `cargo run -p xcfun-py --example gen_py_fixtures`
- [ ] `crates/xcfun-py/examples/gen_py_fixtures.rs` — NEW; per Example F (Rust binary that emits `(functional, vars, mode, order, density, expected)` JSON tuples)
- [ ] `crates/xcfun-py/README.md` — NEW; install + GPU rebuild docs
- [ ] `xtask/src/bin/release_publish.rs` — NEW; per D-15 topology
- [ ] `xtask/Cargo.toml` — add `[[bin]] name = "release-publish"` entry
- [ ] `.github/workflows/release.yml` — NEW; per Example E (sdist + linux/macos/windows wheel matrix + publish-pypi OIDC + release-artifacts + github-release)
- [ ] `CHANGELOG.md` — NEW at repo root; Keep-a-Changelog 1.1.0 format
- [ ] root `Cargo.toml` — drop `crates/xcfun-python` from `exclude`; add `crates/xcfun-py` to `members`; (optional) add `numpy` and `pyo3` to `[workspace.dependencies]`
- [ ] `crates/xcfun-py/.gitignore` — NEW; ignore `target/wheels`, `*.so`, `*.dylib`, `*.pyd`, `__pycache__`, `.pytest_cache`, `*.egg-info`

*Wave 0 also clears the 4 blocking HUMAN-UAT items (06-HUMAN-UAT items 3, 4, 5, 6) per D-14 — these gate v0.1.0 but are not new file scaffolding.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| MPMATH ground-truth fixture regen on 26 functionals | D-14 #3 | ~6h offline mpmath@200 sweep; not a CI workload | `python validation/mpmath_sidecar/regen_all.py` overnight; commit regenerated `validation/mpmath_fixtures/*.json` |
| 4 HUMAN-UAT items 3/4/5/6 cleared on restored xcfun-master HEAD a89b783 | D-14 | requires running tier-2/tier-3 sweeps on dev workstation; multi-hour wall time | per-item per `06-HUMAN-UAT.md`; record outcomes back into `06-HUMAN-UAT.md` |
| GitHub Release page artifacts visible after tag push | D-16 | requires inspecting GitHub UI after `release.yml` completes | open `https://github.com/<owner>/xcfun_rs/releases/tag/v0.1.0`; verify {source tarball, `xcfun.h`, 3× `libxcfun_capi.{so,dylib,dll}`} attached |
| PyPI page renders correctly with classifiers + README | PY-06 (post-publish) | PyPI rendering requires real upload; sandbox testpypi acceptable for a dry-run | `maturin publish --repository testpypi --skip-existing` first; visually inspect `https://test.pypi.org/project/xcfun_rs/` before real publish |
| `pip install xcfun_rs` from PyPI works on a clean machine | PY-06 | post-publish smoke test on a clean Python venv outside CI | `python -m venv /tmp/xcfun-smoke && /tmp/xcfun-smoke/bin/pip install xcfun_rs && /tmp/xcfun-smoke/bin/python -c "import xcfun_rs; print(xcfun_rs.version())"` |
| ROCm tier-3 1e-13 hardware sweep | (D-14 #1 — DEFERRED to v0.2) | no AMD/ROCm GPU on cloud-CI runner; v0.2 hardware-CI cycle | not in v0.1.0 scope; documented in `06-HUMAN-UAT.md` with v0.2 deferral |
| Wgpu tier-3 1e-9 hardware sweep (excluding ERF) | (D-14 #2 — DEFERRED to v0.2) | no SHADER_F64 adapter; v0.2 hardware-CI cycle | not in v0.1.0 scope |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (24 file/scaffold gaps + 4 HUMAN-UAT clearances)
- [ ] No watch-mode flags (CI matrix runs once-and-out per push)
- [ ] Feedback latency < 30 s for per-task quick run
- [ ] `nyquist_compliant: true` set in frontmatter (flip after Plan-phase populates final task IDs)

**Approval:** pending
