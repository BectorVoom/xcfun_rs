# Phase 7: Python Bindings (`xcfun-py`) + Release - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Phase:** 07-python-bindings-release
**Areas discussed:** Naming & GPU-feature exposure; Python API shape & NumPy ergonomics; Error mapping policy; Release ceremony scope & blockers
**Mode:** default (interactive, single-question turns; 4 areas pre-selected via multiSelect)

---

## Gray-area selection

**Question:** Which areas do you want to discuss for Phase 7 (Python Bindings + Release)?
**Options presented (multiSelect):**

| Option | Description | Selected |
|--------|-------------|----------|
| Naming & GPU-feature exposure | Locks Cargo crate name (xcfun-python vs xcfun-py), PyPI package name (xcfun_rs vs xcfun-rs), Python import name; AND how Phase 6 hip/cuda/wgpu/metal feature flags surface to Python. | ✓ |
| Python API shape & NumPy ergonomics | Functional class lifecycle (eager vs lazy eval_setup), set() mutation style, NumPy strictness, eval_vec out= kwarg. | ✓ |
| Error mapping policy | XcError → Python XcfunError shape; payload exposure for WgpuNoF64; panic policy. | ✓ |
| Release ceremony scope & blockers | v0.1 vs v1.0; Phase-6 HUMAN-UAT blocker subset; multi-crate publish topology; GitHub Release artifact strategy. | ✓ |

**User's choice:** All four areas selected.

---

## Area 1: Naming & GPU-feature exposure

### Question 1.1 — Cargo crate name

**Question:** Cargo crate name for the PyO3 crate — `crates/xcfun-python/` exists as a stub; design docs reference `xcfun-py`. Which name should be locked?

| Option | Description | Selected |
|--------|-------------|----------|
| Rename to xcfun-py | git mv + update package.name. Matches docs/design/05, ROADMAP, Phase 5 D-01 precedent (xcfun-ffi → xcfun-capi). | ✓ |
| Keep xcfun-python | Lower churn; breaks short-suffix convention. | |
| Both (alias) | Doubles maintenance; not justified for internal-graph crate. | |

**User's choice:** Rename to xcfun-py (Recommended).
**Notes:** Aligns the project on the short-suffix convention used by every other workspace crate.

### Question 1.2 — PyPI / import name

**Question:** PyPI package name + Python import name. PROJECT.md uses both `xcfun_rs` and `xcfun-rs`.

| Option | Description | Selected |
|--------|-------------|----------|
| pip name `xcfun_rs`, import `xcfun_rs` | Underscore both sides; modern PyO3 convention. | ✓ |
| pip name `xcfun-rs`, import `xcfun_rs` | Hyphenated PyPI distribution; pip normalization handles both. | |
| pip name `xcfun`, import `xcfun` | Drops _rs; collision risk. | |

**User's choice:** pip name `xcfun_rs`, import `xcfun_rs` (Recommended).
**Notes:** Resolves PROJECT.md inconsistency on the underscore side.

### Question 1.3 — GPU exposure in wheel

**Question:** How do GPU backends surface in the Python wheel? Phase 6 has cargo features hip/cuda/wgpu/metal.

| Option | Description | Selected |
|--------|-------------|----------|
| CPU-only default wheel + extras docs | Single PyPI wheel; users `maturin build --features hip` etc. for GPU. | ✓ |
| CPU default + multi-wheel for GPU | `xcfun_rs_cuda`, `xcfun_rs_rocm` separate distributions on PyPI. | |
| All-in-one GPU-enabled wheel | Single wheel with hip+cuda+wgpu features; non-portable. | |

**User's choice:** CPU-only default wheel + extras docs (Recommended).

### Question 1.4 — Python Backend exposure

**Question:** Should Python users see the Backend / auto_backend / XCFUN_FORCE_BACKEND knob?

| Option | Description | Selected |
|--------|-------------|----------|
| Hide — auto_backend only | Rust-side selection; env override still works. | ✓ |
| Optional backend kwarg | `eval_vec(densities, backend='auto')`. | |
| Expose Backend enum + auto_backend() | Full mirror of Rust API. | |

**User's choice:** Hide — auto_backend only (Recommended).

### Continuation prompt

**Question:** More questions on Naming & GPU exposure, or move to next?
**User's choice:** Next area.

---

## Area 2: Python API shape & NumPy ergonomics

### Question 2.1 — Constructor lifecycle

**Question:** Functional lifecycle — Rust requires eval_setup before eval. How does this surface in Python?

| Option | Description | Selected |
|--------|-------------|----------|
| Eager constructor | `Functional('pbe', vars=..., mode=..., order=...)` performs eval_setup at construction. | ✓ |
| Direct mirror of Rust | `f = Functional('pbe'); f.set(...); f.eval_setup(...); f.eval(...)`. | |
| Both styles | Eager + low-level setters retained. | |

**User's choice:** Eager constructor (Recommended).
**Notes:** Constructor + low-level configure() escape hatch (per CONTEXT D-05).

### Question 2.2 — set() style

**Question:** Mutation pattern for set(name, value).

| Option | Description | Selected |
|--------|-------------|----------|
| Mutating method, returns None | `f.set('exx', 0.25)` mutates and returns None. | ✓ |
| Mutating + returns self for chaining | Builder-pattern fluent. | |
| Constructor-only kwargs | Immutable Functional; no post-construction mutation. | |

**User's choice:** Mutating method, returns None (Recommended).

### Question 2.3 — NumPy strictness

**Question:** What happens if user passes f32 / Fortran-order / non-contiguous to eval_vec?

| Option | Description | Selected |
|--------|-------------|----------|
| Strict zero-copy or raise | Accept ONLY f64 + C_CONTIGUOUS; otherwise TypeError. | ✓ |
| Auto-convert with one-time DeprecationWarning | Internally np.ascontiguousarray + RuntimeWarning. | |
| Auto-convert silently | Most user-friendly; least transparent. | |

**User's choice:** Strict zero-copy or raise (Recommended).

### Question 2.4 — out= kwarg

**Question:** eval_vec output array layout.

| Option | Description | Selected |
|--------|-------------|----------|
| Allocate + return | `out = f.eval_vec(densities)` returns fresh ndarray. | ✓ |
| Allocate + return, with optional out= kwarg | NumPy-ufunc-convention; allows reuse across calls. | |
| out= kwarg required | Mirrors Rust &mut [f64]; unidiomatic. | |

**User's choice:** Allocate + return (Recommended).
**Notes:** out= kwarg deferred to v2 if perf-sensitive callers ask.

### Continuation prompt

**Question:** More questions on Python API shape, or move to next?
**User's choice:** Next area.

---

## Area 3: Error mapping policy

### Question 3.1 — Exception class shape

**Question:** Python exception class shape for PY-05.

| Option | Description | Selected |
|--------|-------------|----------|
| Single XcfunError + .code attribute | `.code: int` (matches Phase 5 D-08-A) + `.kind: str`. | ✓ |
| Hierarchy of exceptions | XcfunError base + XcfunInvalidVarsError / XcfunInvalidModeError / etc. | |
| Single class, .kind only (no .code) | Drops i32 code; loses C ABI parity. | |

**User's choice:** Single XcfunError + .code attribute (Recommended).
**Notes:** Mirrors stdlib `OSError.errno` pattern.

### Question 3.2 — WgpuNoF64 payload exposure

**Question:** Payload exposure for WgpuNoF64. (User intervention: "I do not need adapter name field" — adapter_name dropped from all subsequent options.)

| Option | Description | Selected |
|--------|-------------|----------|
| Drop adapter_name; keep only .kind='WgpuNoF64' | No payload at the Python boundary. | ✓ |
| Drop adapter_name; expose requested_runtime as string | Single field 'Wgpu' / 'Metal'. | |
| Drop both adapter_name and the variant entirely | Convert at boundary to kind='Runtime'. | |

**User's choice:** Drop adapter_name; keep only .kind='WgpuNoF64' (Recommended; user-driven simplification).
**Notes:** User explicitly does not want adapter_name. Rust XcError::WgpuNoF64 retains both fields server-side for tracing/debug; nothing crosses the Python boundary.

### Question 3.3 — Panic policy

**Question:** PyO3 default vs catch_unwind shim.

| Option | Description | Selected |
|--------|-------------|----------|
| Inherit PyO3 default | Rust panic → pyo3.PanicException (BaseException subclass). | ✓ |
| catch_unwind → raise XcfunError(kind='Panic') | Convert to regular exception. | |

**User's choice:** Inherit PyO3 default (Recommended).
**Notes:** Matches numpy / polars / pyarrow convention.

### Question 3.4 — Validation timing

**Question:** Where does eval_setup InvalidVars/Mode/Order error raise?

| Option | Description | Selected |
|--------|-------------|----------|
| Constructor raises XcfunError on bad (vars,mode,order) | Fail-fast at the bad call site. | ✓ |
| Constructor delays eval_setup to first eval() | Error raises far from the bad call site. | |

**User's choice:** Constructor raises XcfunError on bad (vars,mode,order) (Recommended).

### Continuation prompt

**Question:** More questions on Error mapping, or move to next?
**User's choice:** Next area.

---

## Area 4: Release ceremony scope & blockers

### Question 4.1 — Initial release version

**Question:** v0.1.0 vs v1.0.0?

| Option | Description | Selected |
|--------|-------------|----------|
| v0.1.0 — pre-1.0, API may evolve | Match workspace pin; cautious-first-release pattern. | ✓ |
| v1.0.0 — API stable, semver-bound | Strict; premature given cubecl pre-release dep. | |
| v0.1.0 + named release tag 'milestone-1' | Crate version 0.1.0 + separate human-readable tag. | |

**User's choice:** v0.1.0 — pre-1.0, API may evolve (Recommended).

### Question 4.2 — Phase-6 HUMAN-UAT blockers

**Question:** Block release on which of the 6 Phase-6 UAT items?

| Option | Description | Selected |
|--------|-------------|----------|
| Block on (3), (4), (5), (6); skip (1), (2) | Hardware-gated items deferred; offline items run. | ✓ |
| Block on all 6 | Strict; not actionable on current dev environment. | |
| Block on none — ship as v0.1, document caveats | Faster release; numerical contract softer. | |
| Block on (6) typo only | Minimal blocker. | |

**User's choice:** Block on (3), (4), (5), (6); skip (1), (2) (Recommended).
**Notes:** Pragmatic — what's runnable now gets run; what's hardware-gated is honestly deferred to v0.2 hardware-CI cycle.

### Question 4.3 — Crates.io publish order

**Question:** How is the multi-crate publish ordered?

| Option | Description | Selected |
|--------|-------------|----------|
| Topological cargo publish + automated script | xtask binary `release-publish` walks DAG with sleep + idempotent skip. | ✓ |
| Manual sequenced commands in CHANGELOG | Document the order; release manager runs each. | |
| Single mega-crate (collapse the workspace) | Loses the layering benefit. | |

**User's choice:** Topological cargo publish + automated script (Recommended).

### Question 4.4 — Release artifacts

**Question:** Beyond crates.io and PyPI, what ships?

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Releases: tarball + xcfun.h + per-platform libxcfun_capi | C/C++ users get pre-built binaries. | ✓ |
| GitHub Releases: tarball + xcfun.h only | Source build only; harder for C-only consumers. | |
| Crates.io + PyPI only | C users must build from source; misses positioning. | |

**User's choice:** GitHub Releases: tarball + xcfun.h + per-platform libxcfun_capi (Recommended).
**Notes:** Defends the "drop-in C ABI replacement for xcfun-master/api/xcfun.h" positioning.

### Continuation prompt

**Question:** More questions on Release ceremony, or wrap up?
**User's choice:** Wrap up — ready for context.

---

## Claude's Discretion

Items the user did not pin and where Claude has flexibility at planning time:

- Concrete `pyproject.toml` content (build-backend = `maturin`; ABI3 metadata; classifiers; URLs).
- Type-stub strategy: hand-written `.pyi` co-located with the wheel vs auto-generated. Recommend hand-written.
- Mode / Vars Python enum exposure: `#[pyclass(eq, eq_int)]` IntEnum vs `enum.StrEnum`. Recommend IntEnum.
- Free-function placement: module top-level vs `Functional` classmethods. Recommend module top-level.
- pytest harness shape: JSON-fixture-based vs vendored xcfun-master ctypes cross-check. Recommend JSON fixtures.
- CHANGELOG.md format: Keep-a-Changelog (https://keepachangelog.com/) is the standard.
- PyPI publish auth: PyPI Trusted Publisher (GitHub OIDC).
- Wheel matrix concrete shape: `manylinux_2_28_x86_64`, `macosx_11_0_arm64`, `win_amd64` (one wheel per platform with abi3-py310).
- Whether `xcfun_rs.Backend` / `xcfun_rs.auto_backend()` get a `_` prefix or are simply absent from `__all__`.
- Wheel filename suffix for GPU rebuilds (PEP 440 local-version-identifier, e.g., `+cuda12`).
- Linker stripping / debug-info stripping for the released `libxcfun_capi.{so,dylib}`.

---

## Deferred Ideas

(Synced from CONTEXT.md `<deferred>` for audit-trail completeness.)

- `out=` kwarg on `eval_vec` — v2 if perf-sensitive callers ask.
- Per-backend separate PyPI distributions (`xcfun_rs_cuda`, `xcfun_rs_rocm`) — rejected per D-03.
- All-in-one GPU-enabled wheel — rejected per D-03.
- ROCm tier-3 1e-13 hardware sweep — v0.2 hardware-CI cycle.
- Wgpu tier-3 1e-9 hardware sweep — v0.2 hardware-CI cycle.
- Linux aarch64 / macOS x86_64 / Windows aarch64 wheels — v0.2 wheel matrix expansion.
- Pre-built `libxcfun_capi` for additional triples — v0.2 GitHub Release artifact expansion.
- Python `Backend` enum exposure — v2+ if users ask.
- `adapter_name` payload on `WgpuNoF64` exception — explicitly dropped per user.
- `requested_runtime` payload on `WgpuNoF64` — dropped consequentially with Backend hiding.
- `catch_unwind` shim around PyO3 entries — v2+ only if panic-leakage reported.
- v1.0.0 hard semver lock — gated on cubecl stable + UAT clearance + downstream feedback.
- Patches to `xcfun-master/` C++ source — vendored content-hash invariant preserved.
- PyPI publish via long-lived API token — fallback only; OIDC is primary.
- Type stubs auto-generated from PyO3 — recommend hand-written for v0.1.
- Python `@dataclass`-style configuration objects for parameters — current set/get-by-name pattern is sufficient.
