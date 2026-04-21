# 01 — Source tree

## 1. Workspace layout

`xcfun_rs` is a Cargo workspace. The workspace root holds top-level configuration; each crate is a separate directory under `crates/`. Tests live inside each crate (unit tests `#[cfg(test)]`, integration tests in `tests/`); cross-crate integration tests and the C++-parity validation harness live under `xtask/` and `validation/`.

The top-level `src/` directory currently in the repository (with only `main.rs`) is removed; the project is a pure workspace.

```
xcfun_rs/
├── Cargo.toml                     # [workspace] declaration, [workspace.dependencies]
├── Cargo.lock
├── rust-toolchain.toml            # channel = "1.85", components = [rustfmt, clippy]
├── rustfmt.toml
├── clippy.toml
├── deny.toml                      # cargo-deny rules
├── CLAUDE.md
├── README.md
├── LICENSE                        # MPL-2.0 (inherited from xcfun)
│
├── .cargo/
│   └── config.toml                # sysroot, linker tweaks
│
├── crates/
│   ├── xcfun-ad/                  # Automatic differentiation (CTaylor<T,N>)
│   ├── xcfun-core/                # Functional registry, densvars, dispatcher
│   ├── xcfun-kernels/             # cubecl kernels (per-point evaluation, shared CPU/GPU)
│   ├── xcfun-gpu/                 # Batch orchestration, runtime selection, buffer management
│   ├── xcfun-rs/                  # Top-level native Rust API (re-exports + Functional)
│   ├── xcfun-capi/                # C ABI (cdylib) + cbindgen build script
│   └── xcfun-py/                  # Python bindings (pyo3 / maturin)
│
├── xtask/                         # cargo-xtask helper (codegen, release tasks)
│   └── src/main.rs
│
├── validation/                    # C++ parity harness (binary crate, uses anyhow)
│   ├── Cargo.toml
│   ├── build.rs                   # compiles xcfun-master/ to a static lib
│   ├── src/
│   │   ├── main.rs                # CLI entry
│   │   ├── cpp_shim.rs            # FFI to C++ xcfun
│   │   ├── fixtures.rs            # density grid generators
│   │   ├── compare.rs             # relative error reducer
│   │   └── report.rs              # HTML / JSON report writer
│   └── fixtures/                  # Recorded C++ reference data, gzipped .jsonl
│
├── benches/                       # Workspace-level benchmark runner
│   └── bench_driver.rs
│
├── docs/
│   ├── design/                    # This document set
│   ├── manual/                    # Cubecl reference (committed material)
│   └── rust_crate_test_guideline.md
│
├── xcfun-master/                  # Unmodified C++ reference (fetched submodule or vendored)
│
└── .planning/                     # GSD workflow artifacts
```

## 2. Crate responsibilities (summary)

| Crate | Kind | Depends on | Exports | Heap? |
|-------|------|-----------|---------|-------|
| `xcfun-ad` | `lib` | (none) | `CTaylor<T, const N: usize>`, `Num` trait, bit-flag helpers | no |
| `xcfun-core` | `lib` | `xcfun-ad`, `bitflags`, `thiserror` | Functional registry, `DensVars<T>`, `Vars`, `Mode`, dispatcher | no |
| `xcfun-kernels` | `lib` | `xcfun-core`, `cubecl` | `#[cube]` per-point evaluators, one per (functional, order) | no |
| `xcfun-gpu` | `lib` | `xcfun-kernels`, `cubecl-cpu`, `cubecl-cuda`, `cubecl-wgpu`, `tracing` | `Batch`, `Backend` enum, buffer pool | yes (device buffers) |
| `xcfun-rs` | `lib` | `xcfun-core`, `xcfun-gpu` | `Functional`, `XcError`, re-exports | no on hot path |
| `xcfun-capi` | `cdylib`+`staticlib` | `xcfun-rs`, `cbindgen` (build-dep) | C symbols + generated `xcfun.h` | no |
| `xcfun-py` | `cdylib` | `xcfun-rs`, `pyo3`, `numpy` | `xcfun_rs` Python module | GIL-governed |
| `validation` | `bin` | `xcfun-rs`, `anyhow`, `approx` | Binary `xcfun-validate` | yes |
| `xtask` | `bin` | `anyhow` | Codegen, release helpers | yes |

Detailed responsibilities, public vs. internal symbols, and test seams are in [05-module-responsibilities.md](05-module-responsibilities.md).

## 3. Module layout per crate

### 3.1 `crates/xcfun-ad/`

```
xcfun-ad/
├── Cargo.toml
├── src/
│   ├── lib.rs                     # pub use
│   ├── num.rs                     # Num trait (add, sub, mul, neg, reciprocal, pow, exp, log, sqrt, erf)
│   ├── ctaylor.rs                 # CTaylor<T, const N: usize> struct + ops
│   ├── ctaylor_mul.rs             # Recursive/unrolled multiplication per N
│   ├── ctaylor_compose.rs         # Series composition (divison, inversion, exp, log, pow, sqrt, erf)
│   ├── expand/                    # Scalar series expansion coefficients
│   │   ├── mod.rs
│   │   ├── inv.rs                 # inv_expand
│   │   ├── exp.rs                 # exp_expand
│   │   ├── log.rs                 # log_expand
│   │   ├── pow.rs                 # pow_expand
│   │   ├── sqrt.rs
│   │   └── erf.rs
│   ├── bits.rs                    # VAR0, VAR1, … VAR7 constants; CNST = 0; index helpers
│   └── tests/                     # in-module unit tests (#[cfg(test)])
│
└── tests/
    ├── cpp_taylor_parity.rs       # Against taylor.hpp / ctaylor.hpp generated fixtures
    └── proptest_algebra.rs        # Property tests for ring axioms
```

Test crate dependency on `proptest`, `approx`; no `anyhow`.

### 3.2 `crates/xcfun-core/`

```
xcfun-core/
├── Cargo.toml
├── build.rs                        # optional codegen for functional table; see §4
├── src/
│   ├── lib.rs
│   ├── error.rs                    # XcError (thiserror)
│   ├── vars.rs                     # Vars enum (31 variants), vars_data table
│   ├── mode.rs                     # Mode enum (Unset, PartialDerivatives, Potential, Contracted)
│   ├── depends.rs                  # bitflags: XC_DENSITY|XC_GRADIENT|XC_LAPLACIAN|XC_KINETIC|XC_JP
│   ├── parameter.rs                # XC_EXX, XC_RANGESEP_MU, XC_CAM_ALPHA, XC_CAM_BETA
│   ├── densvars.rs                 # DensVars<T: Num> with per-Vars constructor
│   ├── registry/
│   │   ├── mod.rs                  # FunctionalId enum + static tables
│   │   ├── descriptor.rs           # FunctionalDescriptor (name, depends, test_data, fp fn ptrs)
│   │   ├── lookup.rs               # case-insensitive name → id
│   │   └── aliases.rs              # Alias table (parsed from generated code)
│   ├── functionals/                # One module per functional (pure Rust, generic over T: Num)
│   │   ├── mod.rs
│   │   ├── lda/
│   │   │   ├── slaterx.rs
│   │   │   ├── vwn3c.rs
│   │   │   ├── vwn5c.rs
│   │   │   ├── pw92c.rs
│   │   │   ├── pz81c.rs
│   │   │   ├── ldaerfx.rs
│   │   │   ├── ldaerfc.rs
│   │   │   ├── ldaerfc_jt.rs
│   │   │   └── tfk.rs
│   │   ├── gga/                    # 45 GGA exchange / correlation
│   │   │   ├── pbex.rs / pbec.rs / revpbex.rs / rpbex.rs / pbesolx.rs / pbeintx.rs / pbeintc.rs / spbec.rs / pbelocc.rs / apbex.rs / apbec.rs / vwn_pbec.rs / zvpbeintc.rs / zvpbesolc.rs
│   │   │   ├── beckex.rs / beckecorrx.rs / beckesrx.rs / beckecamx.rs
│   │   │   ├── brx.rs / brc.rs / brxc.rs
│   │   │   ├── lypc.rs
│   │   │   ├── optx.rs / optxcorr.rs
│   │   │   ├── pw86x.rs / pw91x.rs / pw91c.rs / pw91k.rs
│   │   │   ├── p86c.rs / p86corrc.rs
│   │   │   ├── b97x.rs / b97c.rs / b97_1x.rs / b97_1c.rs / b97_2x.rs / b97_2c.rs
│   │   │   ├── ktx.rs / btk.rs / vwk.rs / lb94.rs
│   │   │   └── csc.rs / tw.rs
│   │   ├── metagga/                # 15 metaGGA
│   │   │   ├── tpssx.rs / tpssc.rs / revtpssx.rs / revtpssc.rs / tpsslocc.rs
│   │   │   ├── scanx.rs / scanc.rs / rscanx.rs / rscanc.rs / rppscanx.rs / rppscanc.rs / r2scanx.rs / r2scanc.rs / r4scanx.rs / r4scanc.rs
│   │   │   ├── m05x.rs / m05c.rs / m05x2x.rs / m05x2c.rs
│   │   │   ├── m06x.rs / m06c.rs / m06lx.rs / m06lc.rs / m06hfx.rs / m06hfc.rs / m06x2x.rs / m06x2c.rs
│   │   │   └── blocx.rs
│   │   └── shared/                 # Helpers ported from .hpp files
│   │       ├── constants.rs
│   │       ├── pbe_eps.rs          # pbec_eps.hpp
│   │       ├── pw92_eps.rs
│   │       ├── pz81.rs
│   │       ├── vwn.rs
│   │       ├── slater.rs
│   │       ├── tpssx_eps.rs / tpssc_eps.rs / revtpssx_eps.rs / revtpssc_eps.rs
│   │       ├── m0xy_fun.rs
│   │       ├── scan_eps.rs
│   │       ├── pbex_r.rs
│   │       ├── b97_common.rs
│   │       └── specmath.rs         # poly (Horner), pow2, pow3, ufunc, integer powers
│   ├── dispatch.rs                 # order→CTaylor<_, N> selection, partial-deriv layout
│   ├── potential.rs                # XC_POTENTIAL specialisation
│   ├── contracted.rs               # XC_CONTRACTED specialisation
│   └── setup.rs                    # Functional struct, set/get, eval_setup, user_eval_setup
│
└── tests/
    ├── densvars_parity.rs
    ├── functional_self_tests.rs    # Runs each functional's test_in/test_out vector
    ├── alias_composition.rs
    └── dispatch_layout.rs
```

### 3.3 `crates/xcfun-kernels/`

```
xcfun-kernels/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── eval_point.rs               # #[cube] eval_functional_at_point<F: Float, const ORDER: u32, const VARS: u32>
│   ├── eval_batch.rs               # #[cube(launch_unchecked)] eval_batch_kernel
│   ├── weights.rs                  # Upload of settings[] slice to device
│   └── dispatch_table.rs           # Compile-time (functional_id, order) → kernel function handle
│
└── tests/
    └── cpu_runtime_parity.rs       # Kernel output under CpuRuntime vs. xcfun-core scalar path
```

### 3.4 `crates/xcfun-gpu/`

```
xcfun-gpu/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── backend.rs                  # Backend: Cpu | Cuda | Wgpu
│   ├── select.rs                   # auto_backend() : probes devices + f64 support
│   ├── batch.rs                    # Batch<'a>, lifecycle: alloc → copy → launch → read
│   ├── buffers.rs                  # DeviceBuffer wrapper, pooling
│   └── metrics.rs                  # tracing spans for kernel wall-clock
│
└── tests/
    └── round_trip.rs               # 100k points: upload, eval, download
```

### 3.5 `crates/xcfun-rs/`

```
xcfun-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # re-exports + top-level doc
│   ├── functional.rs               # Functional<'ctx> user-facing handle
│   ├── prelude.rs
│   └── text.rs                     # version(), splash(), authors(), describe_*, enumerate_*
│
├── examples/
│   ├── minimal_lda.rs              # xcfun.h README example in Rust (uses anyhow)
│   ├── b3lyp_grid.rs               # 100k points, CPU
│   └── m06_cuda.rs                 # 1M points, CUDA (feature = "cuda")
│
└── tests/
    ├── api_coverage.rs             # every xcfun.h function is reachable
    └── smoke.rs
```

### 3.6 `crates/xcfun-capi/`

```
xcfun-capi/
├── Cargo.toml                      # [lib] crate-type = ["cdylib", "staticlib"]
├── cbindgen.toml
├── build.rs                        # runs cbindgen to emit include/xcfun.h
├── src/
│   ├── lib.rs                      # extern "C" { ... } matching xcfun.h symbol-for-symbol
│   └── handle.rs                   # xcfun_t → Functional bridging, aborts on UB
│
├── include/
│   └── xcfun.h                     # generated, checked into repo for release tags
│
└── tests/
    ├── headers_match.rs            # diff generated xcfun.h vs. xcfun-master/api/xcfun.h
    └── c_abi.c                     # C test, compiled via build.rs for nextest
```

### 3.7 `crates/xcfun-py/`

```
xcfun-py/
├── Cargo.toml
├── pyproject.toml                  # maturin build-system
├── src/
│   ├── lib.rs                      # #[pymodule] fn xcfun_rs(py, m)
│   ├── functional.rs               # PyFunctional wrapper
│   └── numpy_io.rs                 # zero-copy f64 array plumbing
│
└── tests/
    └── test_parity.py              # Against original python/xcfun bindings if available
```

## 4. Generated files

Some tables are large and must be kept in one-to-one correspondence with `xcfun-master`. These are produced by `xtask codegen`:

| Generated file | Source of truth | Trigger |
|----------------|-----------------|---------|
| `xcfun-core/src/registry/generated/functionals.rs` | `xcfun-master/src/functionals/list_of_functionals.hpp` | `cargo xtask regen-registry` |
| `xcfun-core/src/registry/generated/aliases.rs` | `xcfun-master/src/functionals/aliases.cpp` | same |
| `xcfun-core/src/registry/generated/vars_table.rs` | `xcfun-master/src/xcint.cpp` (the `xcint_vars[]` array) | same |
| `xcfun-core/src/registry/generated/test_vectors.rs` | Each functional's `FUNCTIONAL(...)` macro `test_in` / `test_out` | same |
| `xcfun-capi/include/xcfun.h` | `cargo build -p xcfun-capi` (cbindgen) | every build |

The codegen tool parses the C++ headers via `syn`-free text pattern-matching (header grammar is stable and narrow) and emits Rust source. Generated files are committed so `cargo build` with no feature flags does not require a C++ toolchain.

## 5. Naming conventions

- Rust crate names use `kebab-case`: `xcfun-core`, `xcfun-gpu`.
- Rust module names use `snake_case`: `dens_vars`, but preserving xcfun's established identifiers where they ease comparison to the C++ source (`pw92c`, `m06x`).
- Public types use `CamelCase`: `Functional`, `CTaylor`, `XcError`.
- Constants use `SCREAMING_SNAKE_CASE`: `MAX_ORDER`, `NR_FUNCTIONALS`, `XCFUN_TINY_DENSITY`.
- Functional identifiers keep the `XC_` prefix (`FunctionalId::XC_SLATERX`) so translation tables are grep-able across languages.

## 6. Feature flags (per crate)

| Crate | Feature | Effect |
|-------|---------|--------|
| `xcfun-gpu` | `cuda` | pulls `cubecl-cuda`, adds `Backend::Cuda` |
| `xcfun-gpu` | `wgpu` | pulls `cubecl-wgpu` |
| `xcfun-gpu` | default | `cpu` (always on) |
| `xcfun-rs` | `cuda`, `wgpu` | re-exports corresponding `xcfun-gpu` feature |
| `xcfun-rs` | `capi-inline` | enable `#[inline]` on thin shims for C-callers |
| `xcfun-capi` | default | cdylib; no external opts |
| `xcfun-py` | `abi3` | build a stable-abi wheel |
| `xcfun-core` | `std` | default on; `no_std` build available for kernel-internal use by `xcfun-kernels` |

Feature flags never change numerical behaviour; they only vary the set of available backends. Regression tests run all feature combinations on CI.
