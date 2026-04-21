# 05 — Module responsibilities

Each crate has exactly one responsibility, stated as a single sentence at the top of its entry. The rest of the entry lists the public surface (items re-exported or listed in `pub use`), the internal surface (reachable only within the crate), the seams for testing, and the inbound/outbound dependencies.

"Public" means visible outside the crate. "Internal" means `pub(crate)` or private. "Test seam" means an item exposed (possibly `#[cfg(test)]` or behind the `testing` feature) solely so that downstream tests can exercise the module in isolation.

---

## 1. `xcfun-ad`

**Responsibility**: provide the multilinear bit-flag-indexed Taylor polynomial `CTaylor<T, N>` and the scalar series expansions required to compose elementary functions on it.

| Surface | Items |
|---------|-------|
| Public | `CTaylor<T, N>`, `Num`, `CNST`, `VAR0..VAR6`, `Assert`, `expand::{inv, exp, log, pow, sqrt, erf}` |
| Internal | `ctaylor_mul` recursion helpers; scratch buffer types; const-generic dispatch shims |
| Test seam | `pub mod for_tests { pub use crate::expand::*; pub fn raw_coeffs<T, const N: usize>(t: &CTaylor<T, N>) -> &[T; 1 << N]; }` under `feature = "testing"` |
| Depends on | `core` only (no `std` when the `std` feature is off) |
| Allocates on hot path? | Never |
| Unsafe? | None; bit-index arithmetic is checked in debug, dropped in release |

Key invariant: `CTaylor::set(idx, v)` and `CTaylor::get(idx)` are both constant time and contain no branches beyond an array bounds check that the optimiser elides in release builds.

Property-test candidates:
- Ring axioms: commutativity/associativity/distributivity of `+` and `*` on `CTaylor<f64, N>` for `N ∈ 0..=6`.
- `exp(x) * exp(-x) ≈ 1` within `1e-14` relative error for polynomials with random small coefficients.
- `log(exp(x)) ≈ x`, `sqrt(x)^2 ≈ x`, `pow(x, a) * pow(x, -a) ≈ 1`.
- Derivative-of-product rule: `d/dx₀ (f·g) = f'·g + f·g'` — verified by inspecting the bit-flag-0 coefficient.

---

## 2. `xcfun-core`

**Responsibility**: hold the static registry of functionals / parameters / aliases / vars, and execute single-point evaluation on the CPU using `xcfun-ad`.

| Surface | Items |
|---------|-------|
| Public | `Vars`, `Mode`, `Dependency`, `XcError`, `DensVars<T>`, `Functional`, `FunctionalId`, `FunctionalDescriptor`, `Alias`, `AliasTerm`, free functions `version`, `splash`, `authors`, `self_test`, `is_compatible_library`, `which_vars`, `which_mode`, `enumerate_*`, `describe_*`, `input_length`, `output_length` |
| Internal | `registry::lookup_*`, `registry::FUNCTIONAL_DESCRIPTORS`, `registry::ALIASES`, `registry::VARS_TABLE`, `dispatch::eval_partial`, `dispatch::eval_potential`, `dispatch::eval_contracted`, the per-order scalar loops |
| Test seam | `pub mod for_tests { pub use crate::densvars::DensVars; pub use crate::dispatch::eval_partial_n; pub fn set_descriptor(id: FunctionalId, desc: FunctionalDescriptor); }` under `feature = "testing"` (the `set_descriptor` is used by table-driven tests to inject mocks) |
| Depends on | `xcfun-ad`, `bitflags` 2, `thiserror` 2 |
| Allocates on hot path? | Never |
| Unsafe? | None |

Each functional file under `functionals/{lda,gga,metagga}` exposes a single generic `pub(crate) fn <name>_energy<T: Num>(d: &DensVars<T>) -> T`. A `#[functional(id = XC_FOO, depends = …, test_vars = …, …)]` attribute macro generates the seven `fp{N}: EvalFn<N>` fields, the `FunctionalDescriptor` entry, and the `XC_FOO → <name>_energy` mapping.

Property-test candidates:
- For any active-set composition, `input_length()` equals the reference table entry.
- `taylor_len(n, k)` matches `C(n+k, k)` for `n ≤ 20`, `k ≤ 6`.
- `is_gga(self) ⇔ depends.contains(GRADIENT)`, `is_metagga(self) ⇔ depends.intersects(LAPLACIAN | KINETIC)`.

---

## 3. `xcfun-kernels`

**Responsibility**: expose each per-point functional evaluation as a `#[cube]` function usable by `cubecl::CpuRuntime`, `CudaRuntime`, and `WgpuRuntime`.

| Surface | Items |
|---------|-------|
| Public | `eval_batch_kernel<R: cubecl::Runtime, const VARS: u32, const ORDER: u32>` (the single `#[cube(launch_unchecked)]` entry point), plus per-functional `#[cube]` helpers declared inside the crate |
| Internal | Dispatch table mapping `(FunctionalId, order)` → `#[cube] fn` pointer (implemented as a `match` inside the kernel to keep the host/device boundary closed); scratch buffer sizing helpers |
| Test seam | `pub mod for_tests { pub fn launch_on_cpu_runtime(...) -> Vec<f64>; }` — under `feature = "cpu-testing"`, launches the kernel with `CpuRuntime` and returns the host-side result |
| Depends on | `xcfun-core` (for `Vars`, layout tables, `DensVars` builder used on the scalar path), `cubecl` |
| Allocates on hot path? | Device buffers only; no host heap |
| Unsafe? | `launch_unchecked` uses `unsafe` at the entry point as per cubecl idiom; bounds-checked inside |

Purity rule: inside a `#[cube]` body, only other `#[cube]`-annotated functions and `cubecl`-provided math intrinsics may be called. Each functional is mirrored by a `#[cube]`-annotated version using the generic `F: Float` (cubecl) through a `Num` implementation for `F`; see [06-cubecl-strategy.md](06-cubecl-strategy.md).

Property-test candidates:
- Parity: for a fixed seed RNG density grid, `eval_batch_kernel<CpuRuntime, V, K>` result matches `xcfun-core::Functional::eval` to within `5e-15` relative error (sanity bound, not the headline 1e-12 parity against C++).

---

## 4. `xcfun-gpu`

**Responsibility**: own the `Batch` lifecycle — backend selection, device buffer pooling, host↔device transfers, and asynchronous launch.

| Surface | Items |
|---------|-------|
| Public | `Backend` (enum `Cpu`, `Cuda`, `Wgpu`), `Batch<'fun, R>`, `auto_backend()`, `cpu_client()`, `cuda_client(device_id)`, `wgpu_client(device)` |
| Internal | Buffer growth policy; retry logic for GPU OOM; `tracing` spans |
| Test seam | `pub mod for_tests { pub fn launch_with_metrics(...) -> BatchMetrics; }` — exposes timing and transfer counters |
| Depends on | `xcfun-core`, `xcfun-kernels`, `cubecl`, `cubecl-cpu`, `cubecl-cuda` (feature-gated), `cubecl-wgpu` (feature-gated), `tracing` |
| Allocates on hot path? | Only device buffers; host heap used during `reserve()` |
| Unsafe? | `launch_unchecked` calls and a single `Send`-bound escape hatch in the buffer pool, both documented |

Decision tree for `auto_backend()`:

```
if env XCFUN_FORCE_BACKEND is set → use it (panic if unsupported)
else if CudaRuntime::is_available() → Cuda
else if WgpuRuntime::is_available() && wgpu_supports_f64(device) → Wgpu
else → Cpu
```

Property-test candidates: backend-agnostic equivalence — fixed input batch, identical output across `Cpu`, `Cuda`, `Wgpu` (f64-capable) within 1e-12.

---

## 5. `xcfun-rs`

**Responsibility**: user-facing native Rust façade. Composes `xcfun-core` (scalar / single-point) and `xcfun-gpu` (batch). Exposes nothing new, only a clean API.

| Surface | Items |
|---------|-------|
| Public | Re-exports `Functional`, `Vars`, `Mode`, `XcError`, all free functions, plus `Batch`, `Backend`, `batch_on`, `prelude` |
| Internal | None (thin re-exports) |
| Test seam | `tests/api_coverage.rs` exercises every function at least once |
| Depends on | `xcfun-core`, `xcfun-gpu` |
| Allocates on hot path? | Never |
| Unsafe? | None |

The façade's job is to make sure no downstream user must import `xcfun-core` or `xcfun-gpu` directly. If a user needs the scalar AD type, they import `xcfun_rs::ad::CTaylor`; re-exports are chosen to discourage reaching for internal modules.

---

## 6. `xcfun-capi`

**Responsibility**: implement the C ABI declared in `xcfun-master/api/xcfun.h`, generate a matching `xcfun.h` via `cbindgen`, and package as `cdylib` + `staticlib`.

| Surface | Items |
|---------|-------|
| Public (to C) | Every declaration in `xcfun.h`; no Rust item is `pub` outside the `extern "C"` block |
| Public (to Rust) | Only the crate's build script helpers (for the `cbindgen` step) |
| Internal | `xcfun_t` wrapper; a `catch_unwind` shim around every entry point to convert Rust panics to a `xcfun::die`-equivalent `abort` |
| Test seam | A C file `tests/c_abi.c` compiled by `cc` in `build.rs`, linked against the staticlib, and run as a cargo test |
| Depends on | `xcfun-rs`, `cbindgen` (build-dep) |
| Allocates on hot path? | Never beyond `xcfun_new` / `xcfun_delete` |
| Unsafe? | Entire crate is `unsafe`-dense by necessity (`extern "C"` + raw pointers); every `unsafe` block documents its preconditions |

Panic policy: any panic inside a C entry point is caught and converted to `xcfun::die`-style behaviour (stderr message, `abort`). This matches the reference: the C++ library calls `std::abort()` through `xcfun::die` in pathological cases.

---

## 7. `xcfun-py`

**Responsibility**: provide CPython bindings via PyO3 and expose the same API surface through a `xcfun_rs` Python module.

| Surface | Items |
|---------|-------|
| Public (to Python) | Class `Functional`, exception `XcfunError`, free functions as listed in [03-api-surface.md §3](03-api-surface.md#3-python-api) |
| Internal | `PyFunctional` wrapper; `numpy_io` helpers |
| Test seam | `tests/test_parity.py` compares against the C++ xcfun Python wrapper when present (CI only) |
| Depends on | `xcfun-rs`, `pyo3` 0.28, `numpy` (rust-numpy) 0.28 |
| Allocates on hot path? | GIL-governed numpy arrays, never re-allocated inside `eval_vec` |
| Unsafe? | PyO3 macros |

---

## 8. `validation` (binary crate)

**Responsibility**: the C++-parity harness. Loads `xcfun-master` as a C++ dependency (via `cc` / `cxx`), runs every `(functional, vars, mode, order)` combination the C++ library supports, and reports discrepancies.

| Surface | Items |
|---------|-------|
| Public | None (binary) |
| Internal | `cpp_shim` (FFI to C++), `fixtures::grid_*` density generators, `compare::max_rel_error`, `report::write_html`, `report::write_jsonl` |
| Test seam | None — the validation run itself is the test |
| Depends on | `xcfun-rs`, `anyhow`, `approx`, `cc`, `serde_json`, `tracing-subscriber` |
| Allocates on hot path? | Yes (this is a report-generating tool, not a hot path) |
| Unsafe? | FFI to C++ only |

The harness is the only crate allowed to depend on `anyhow`, since it is an application, not a library. This is verified by a `cargo metadata | jq` filter in CI.

---

## 9. `xtask` (binary crate)

**Responsibility**: project-local CLI automation (codegen, release, lint).

| Subcommand | Effect |
|------------|--------|
| `cargo xtask regen-registry` | Re-generate `xcfun-core/src/registry/generated/*.rs` from `xcfun-master` sources |
| `cargo xtask regen-capi-header` | Re-run cbindgen for `xcfun-capi/include/xcfun.h` and diff against `xcfun-master/api/xcfun.h` |
| `cargo xtask check-no-anyhow` | `cargo metadata` filter: fails if any library crate depends on `anyhow` |
| `cargo xtask bench` | Wrapper around `cargo criterion` with canonical reporters |
| `cargo xtask validate --order N --backend cpu\|cuda\|wgpu` | Runs the validation harness with a tolerance of 1e-12 |

---

## 10. Module boundary rules

1. `xcfun-ad` is a pure math crate. It must never know about `Vars`, `Functional`, or `DensVars`.
2. `xcfun-core` owns the static registry. No other crate defines a `FunctionalDescriptor`.
3. `xcfun-kernels` only references `xcfun-core` for *data* (the `VARS_TABLE`, `DensVars` layout) and re-implements evaluation in `#[cube]` form. It never imports CPU-only pieces of `xcfun-core` into a kernel body.
4. `xcfun-gpu` is the only crate that owns a `cubecl::Runtime` instance. `xcfun-core::Functional::eval` goes through the scalar path; `xcfun-core::Functional::eval_vec` opens a CPU batch via `xcfun-gpu::cpu_client()` internally, so the scalar path is the only code that ever runs without cubecl.
5. `xcfun-rs` is a façade. No logic lives there.
6. `xcfun-capi` is a shim. No logic lives there.
7. `xcfun-py` is a shim. No logic lives there.
8. The C++ reference lives only in `validation/` (and in `xcfun-master/` for reference). No production crate calls into the C++ library.

These boundaries are enforced by a `cargo xtask check-boundaries` step in CI: the script greps each crate's `Cargo.toml` against an allowlist.
