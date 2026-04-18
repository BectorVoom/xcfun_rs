# Feature Research

**Domain:** Exchange–correlation (XC) functional library for Kohn–Sham density functional theory (DFT) — the dependency that quantum-chemistry host codes (Psi4, PySCF, NWChem, Dalton, ADF, Turbomole, Q-Chem, DIRAC, LSDalton, ERKALE, gpu4pyscf, …) link against to get `{E_xc, v_xc, f_xc, g_xc, …}` for a density grid.
**Researched:** 2026-04-18
**Confidence:** HIGH — reference C++ source, official xcfun docs, Libxc/ExchCXX/GauXC public docs, and multiple host-code integration pages (Psi4, PySCF, ADF, ORCA, VASP, gpu4pyscf) cross-verify the feature set. Numerical-accuracy requirement (1e-12) and the 14-document design brief pin the answer tightly; the only MEDIUM-confidence areas are competitor performance numbers.

## 1. Executive orientation

The xcfun_rs scope is defined by three fixed points:

1. It must reproduce every symbol and every numerical output of the C++ `xcfun-master/` reference to relative error ≤ 1e-12 (see [PROJECT.md](../../PROJECT.md), [docs/design/00-overview.md](../../../docs/design/00-overview.md)).
2. It competes in the *same market position* that xcfun occupies today: the **arbitrary-order-AD** XC library. Its value relative to Libxc (the 400-functional default) is **higher derivative orders** (0–6 vs Libxc's 0–4), not wider functional coverage.
3. "User" = a quantum-chemistry code author, not an end-user chemist. They evaluate the library on: correct output, drop-in linkability, language bindings, stability of `(functional, vars, mode, order)` across versions, speed on dense grids, and GPU capability.

All feature classification below is relative to that audience.

## Feature Landscape

### Table Stakes (Users Expect These)

These are non-negotiable. Missing any one drops xcfun_rs out of consideration for Libxc/xcfun replacement slots in host codes. All are in-scope in `docs/design/00-overview.md`; none are optional.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Full 78-functional coverage (LDA + GGA + meta-GGA, all xcfun IDs `XC_SLATERX`..`XC_PW91C`) | Host codes test against a *named* functional set (`b3lyp`, `pbe`, `blyp`, `scan`, `camb3lyp`, …); any missing ID is a regression from C++ xcfun | HIGH | Each functional is ~50–300 lines of Rust that must algebraically match the C++ source. Source of truth: `xcfun-master/src/functionals/*.cpp` |
| ≥ 46 named aliases (`lda`, `pbe`, `blyp`, `pbe0`, `b3lyp`, `camb3lyp`, `svwn`, `bpw91`, `scan`, `rscan`, `r2scan`, `r4scan`, `m06*`, `b97*`, `kt1..kt3`, `olyp`, `bp86`, `b88x`, …) | Host codes pass user-supplied functional strings verbatim to `xcfun_set`. Unrecognized alias = broken user workflow | LOW–MEDIUM | One registry table (`xcfun-master/src/functionals/aliases.cpp` → Rust static array). 46 exactly counted in the reference source; design doc uses "50+" loosely |
| LDA + GGA + meta-GGA support (τ-dependent *and* Laplacian-dependent) | ~ every DFT paper since 2015 uses meta-GGA; SCAN family is the state of the art for materials; laplacian-only meta-GGAs (BR, BLOC) exist in the 78-list | HIGH | Drives the 31 `Vars` combinations (α/β vs n/s × with/without Laplacian × with/without τ × with/without current × explicit vs γ-form) |
| 4 tunable parameters: `EXX`, `RANGESEP_MU`, `CAM_ALPHA`, `CAM_BETA` | Range-separated hybrids (`camb3lyp`, `ldaerf`) are dominant for charge-transfer excitations. Libxc and xcfun both expose them as named parameters | LOW | Stored in the same settings array as functional weights; encoded by `XC_RANGESEP_MU..XC_CAM_BETA` in `list_of_functionals.hpp` |
| 3 evaluation modes: `PartialDerivatives`, `Potential`, `Contracted` | Reference xcfun exposes these; removing any breaks downstream TDDFT / response-property workflows (contracted mode is the response-theory primitive) | HIGH | Each mode has distinct output layout; design doc §1.4.11 lays out the Taylor coefficient enumeration |
| Derivative orders 0–4 for `PartialDerivatives`, 0–6 for `Contracted` | Libxc tops out at order 4. xcfun's differentiator is order 5–6 for non-adiabatic TDDFT / cubic response. Users pick xcfun precisely because they need it | HIGH | `XCFUN_MAX_ORDER = 6` is a literal contract in `xcfun.h`. Must be enforced at setup time |
| C ABI drop-in replacement for `xcfun-master/api/xcfun.h` | Host codes currently `#include <xcfun.h>` and `link -lxcfun`. Any ABI deviation is a re-port, not a swap | MEDIUM | Covered by `xcfun-capi` + cbindgen header-diff CI (design §03 §2.3, §4) |
| Numerical parity within 1e-12 relative error | Host regression tests compare xc potentials bit-for-bit (within tolerance) across library upgrades. 1e-4 drift fails DFT convergence tests | **VERY HIGH** — this is the entire raison d'être | Drives stack choices (no `-Cfast-math`, no alternative AD crate, in-house `CTaylor<T, N>`). See [docs/design/07-accuracy-strategy.md] |
| Python bindings (NumPy-interoperable) | Psi4 / PySCF / custom scripts drive xcfun from Python far more than from C. The conda-forge `xcfun` wheel is how most non-C++ users actually consume the library | MEDIUM | PyO3 0.28 + rust-numpy 0.28, zero-copy `f64` arrays |
| Single-point `eval` API (one density vector → one derivative vector) | The *per-grid-point* contract every XC library exposes. Removing it breaks every integration we want to land | LOW | `Functional::eval(&[f64], &mut [f64])` |
| Vectorized `eval_vec` API (N-point batch, caller-owned slices with pitches) | Host codes iterate over 10k–1M grid points per SCF step. Per-point C-call overhead (`xcfun_eval` × 1e6) dominates cost. Libxc 7, ExchCXX, and C++ xcfun 2.1 all expose this | MEDIUM | Caller passes `density_pitch`, `out_pitch`, `nr_points`. On CPU, `std::thread::scope` parallelization threshold = 16384 elements |
| Zero hidden global state; thread-safe `Functional` handle | Host codes evaluate different functionals concurrently (e.g., per-fragment in linear-scaling DFT). Reference uses an opaque `xcfun_t*` per thread. Shared globals = race conditions | MEDIUM | Design: owned `Functional`, no `static mut`. `Send + Sync` provable |
| Spin-compensated *and* spin-resolved (α/β) evaluation | Every open-shell / unrestricted DFT calculation needs α/β. Closed-shell uses n, n/s, or α-only. Hosts expect *both* paths | MEDIUM | 31 `Vars` combinations express this: `A`, `N`, `A_B`, `N_S` × four gradient/kinetic/laplacian/current flavours |
| Explicit-gradient-component path (∂/∂x, ∂/∂y, ∂/∂z, not just γ) | Current-dependent functionals, non-abelian DFT, some TDDFT kernels need gradient components separately. Encoded by the `explicit_derivatives` flag in `xcfun_which_vars` | MEDIUM | Design includes `XC_A_AX_AY_AZ`, `XC_N_NX_NY_NZ`, `XC_A_2ND_TAYLOR`, … |
| Dependency introspection (`is_gga`, `is_metagga`, required `Vars`) | Host codes route `fun` to LDA / GGA / meta-GGA integration grids. Missing → wrong grid → wrong integral | LOW | Computed from `Dependency` bitflags union of active functionals |
| Input/output length introspection (`input_length`, `output_length`) | Host allocates buffers before calling `eval`. Mis-sized → buffer overrun / UB | LOW | Already in API surface §1.4.7–8 |
| Error model with stable error codes (1=EORDER, 2=EVARS, 4=EMODE, 6=EVARS\|EMODE, -1=EUNKNOWN) | Host codes switch on error code to report user-friendly messages. Bit-encoded values allow `code & XC_EVARS` tests | LOW | `XcError::as_c_code()` preserves the C-side values |
| Versioning (`xcfun_version`, `xcfun_is_compatible_library`, `API_VERSION=2`) | Host code links at build time; runtime ABI check catches .so/.dll mismatch | LOW | Compile-time constants; strict major-version equality |
| Self-test entry point (`xcfun_test` / `self_test()`) | Every host integration runs it post-install to confirm library health. Returning > 0 means "don't proceed with calculation" | MEDIUM | Iterates built-in test vectors; 0 failures ⇒ 0 return value |
| `enumerate_aliases` / `enumerate_parameters` / `describe_short` / `describe_long` | Host codes build UI dropdowns and help text from these. Missing → users can't discover functional names | LOW | Index-walk + static string tables |
| MPL-2.0 license compatibility | Host codes span GPL (NWChem fragments), BSD (PySCF), Apache (some plugins). MPL is compatible with all commonly; any license change would block adoption | LOW | Inherited from reference; cargo-deny CI gate confirms |
| `no_std`-compatible core crate / no heap allocation on per-point path | Embedded DFT (Turbomole inline, lightweight orbital-free DFT) and GPU kernels need allocation-free inner loops | MEDIUM | Core `eval` uses stack-allocated `[T; 1 << N]` Taylor arrays |
| Documentation of output memory layout (Taylor-coefficient enumeration) | Host code has to index into the result array; undefined layout = unusable | LOW | Design §1.4.11 specifies the canonical order; test fixtures from C++ reference verify |

### Differentiators (Competitive Advantage)

These are why a host code chooses xcfun_rs over Libxc or C++ xcfun. Each directly maps to a Core Value constraint (numerical parity, Rust safety, cubecl GPU path).

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Arbitrary-order derivatives (≤ 6) via algorithmic AD** | Libxc caps at order 4; ExchCXX at order 1–2. Orders 5–6 are required for non-adiabatic TDDFT kernels and cubic-response properties. This is *the* reason xcfun exists; xcfun_rs inherits the niche | **VERY HIGH** (CTaylor<T, N> = 800 lines of subtle polynomial algebra) | In-house AD engine, bit-flag-indexed multilinear polynomial. No Rust crate replicates the approach. See `docs/design/07-accuracy-strategy.md` |
| **Unified CPU/GPU batch evaluation via a single cubecl kernel source** | Libxc GPU support is "experimental", CPU-only in practice; ExchCXX ships distinct CUDA/HIP/SYCL backends with separate code paths. xcfun_rs ships one `#[cube]` per functional that targets `CpuRuntime`, `CudaRuntime`, `WgpuRuntime`. Host code picks backend at launch; maintenance cost is *one* implementation | VERY HIGH | Goal G3 in design §4. cubecl pinned `=0.10.0-pre.3`. f32 path intentionally absent (can't meet 1e-12) |
| **Rust memory safety for a library embedded in long-running HPC jobs** | xcfun C++ has had at least one buffer-indexing CVE historically (DFT libraries run untrusted basis-set inputs). Rust `&[f64]` + slice-length checks eliminate the class. Differentiator vs. all C/C++/Fortran competitors | LOW (comes free with the language) | Enforced by `#[deny(unsafe_op_in_unsafe_fn)]`, bounds-checked slice access, `Result<_, XcError>` on every fallible path |
| **Zero-copy NumPy interop via rust-numpy** | C++ xcfun Python bindings use `pybind11` with copy at the boundary. For a 1 M-point grid, that's 8 MB per call × 2 (in + out). Zero-copy wins a measurable fraction of wall time on large grids | LOW | rust-numpy 0.28 `PyArray2<f64>`, `readonly() / readwrite()` borrow tokens |
| **Typed Rust API (`Vars`, `Mode`, `XcError`, `Dependency`) for Rust callers** | Libxc and C++ xcfun expose only integer enums + `void*` + error codes. Rust callers get pattern-matchable errors, `Result`-based propagation, and compile-time `Vars`-`Dependency` mismatch checks | LOW | Fully native types, not repr(C) unless exported through `xcfun-capi` |
| **Hermetic build (no C++ toolchain required)** | Building Libxc needs autotools (or CMake), a C compiler, optionally NVCC. Building gpu4pyscf-libxc also needs GCC + CUDA headers matching runtime. xcfun_rs workspace is pure `cargo build` | MEDIUM | Functional registry is code-generated by `xtask codegen`, checked into git. Avoids embedding xcfun-master build in downstream consumer builds |
| **Drop-in replacement with *stronger* guarantees than the reference** | (1) Thread-safe by construction, (2) bounds-checked, (3) no signal-unsafe aborts (design replaces `xcfun::die` with `Result` on the Rust side; retains `die` only on the C-ABI path), (4) panic-safe. Host code can rely on these without adding their own wrappers | LOW | Policy-level, enforced by code review |
| **CI-enforced numerical parity gate** | Every PR runs the full (functional × vars × mode × order × density-point) harness and blocks merge on any element > 1e-12 relative error. Libxc/xcfun upstream relies on selected test vectors; xcfun_rs covers the full product grid | HIGH (harness itself) | Design §S2 and [docs/design/09-testing-strategy.md]. Harness links the C++ reference via `cc` crate in the xtask |
| **Feature-gated CUDA / WebGPU backends** | HPC users compile in `cuda` for Linux + NVIDIA; cross-platform users compile in `wgpu` for Metal / Vulkan. Libxc's CUDA support is all-or-nothing at configure time; cubecl gives us runtime dispatch between installed backends | MEDIUM | `cubecl-cuda` and `cubecl-wgpu` crates under `cuda` / `wgpu` features |
| **Deterministic, documented output layout** | The reference xcfun layout is implicit in C++ source; we document it explicitly (design §1.4.11) and test every element against it. Host-code authors can trust the layout across version upgrades | LOW | Side-effect of the validation harness |
| **Smaller binary footprint than Libxc** | Libxc ships ~400 functionals; xcfun_rs ships 78. For HPC nodes with many colocated MPI ranks, loader time + page-cache footprint matter. Not a headline win, but real for embedded / frontend use | LOW (emergent from scope) | Measured, not engineered |

### Anti-Features (Commonly Requested, Often Problematic)

Features that look attractive to add but either conflict with the 1e-12 contract, conflict with the "drop-in replacement for xcfun" scope, or push the project into a different product category. Every entry below has been explicitly rejected in `docs/design/00-overview.md` §2.2 or in PROJECT.md Out of Scope. Do not re-add without a design-level override.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Libxc-scale functional coverage (~400 functionals)** | Libxc has ~400; 78 looks "small" on paper | Scope explosion (~5× the code), test-harness explosion (5× tuples to validate), and the user picks xcfun_rs *because* they want orders 5–6 — for which only xcfun-style AD works. Libxc already covers the broad-coverage niche | Stay at 78. Document the overlap with Libxc so users know which to pick. Let Libxc own coverage; own the derivative order |
| **Bit-identical output with C++ xcfun** | Makes validation "trivial" (`memcmp`) | Infeasible across libm variants (glibc vs musl), CUDA math intrinsics, and f64 reassociation. Promising it either forces awful workarounds (ship our own libm) or silently breaks on cross-platform builds | Relative error ≤ 1e-12; documented and CI-enforced (PROJECT.md Out of Scope §1) |
| **User-supplied Rust functionals with AD (plugin API)** | "I want to add my custom functional" is a common feature request; Libxc has it via Maple code-gen | Opens the AD engine's internals as public API surface → every refactor becomes a breaking change. Also blocks optimizations like codegen-specialization per functional. Libxc solves this via its Maple pipeline; we don't have one | Keep the 78 built-in set. External functionals → fork or send a PR. Future v2 may add a Maple-like pipeline, but not v1 |
| **Third-party AD (Enzyme, hyperdual, `ad` crate)** | "Just use an existing AD library, don't roll your own" | None replicate xcfun's bit-flag multilinear polynomial algebra. Different algorithm → different rounding → 1e-12 parity unprovable. Enzyme requires nightly; hyperdual stops at order 2; `ad` is tape-based | In-house `CTaylor<T, N>` (PROJECT.md Active requirements) |
| **f32 ("mixed precision") numerical path** | GPUs are faster at f32; ML-DFT uses f32 | f32 unit roundoff ≈ 6e-8, cannot meet 1e-12. Any f32 code path silently invalidates the contract for the user who doesn't read the docs | f64 only. Document clearly. `Num` not implemented for `f32` (design §3, PROJECT.md) |
| **Differentiation order > 6** | TDDFT cubic response sometimes wants 7th | Reference caps at 6 (`XCFUN_MAX_ORDER`); raising requires re-verifying coefficient algebra against the C++ source, which also caps at 6. Out of scope | Raise ceiling in a future major version if an upstream reference exists |
| **f128 / extended precision** | "Numerical analysts always want more precision" | Reference uses f64. No host code consumes f128 xc potentials. Adds a whole parallel type hierarchy for zero validated users | Skip. If needed, a future crate `xcfun-ext` could add it |
| **Distributed (multi-node) evaluation** | GPUs are limited by memory; bigger grids across nodes | Density grids are *local* by construction — each grid point evaluates independently. The host code (Psi4 / PySCF / NWChem) already handles grid-point distribution via MPI at a higher level. Adding it at the XC library layer duplicates responsibility | Batch API exposes the parallel work; host handles distribution (PROJECT.md Out of Scope) |
| **Streamed / async GPU overlap in v1** | "Overlap H→D copy with compute for pipelining" | Design estimated < 20 % throughput gain for 3× API complexity (two kernels in flight, user-visible completion events). xcfun_rs is bandwidth-bound on transfer, not compute-bound in most cases | Push to v2; single-stream simplicity in v1 |
| **Symbolic / rational output** | "Show me the analytic derivative, not the number" | Use case is pedagogical (Jupyter); host codes never consume symbolic f_xc. Adds a whole parallel code path. Maple and SymPy solve it externally | Numerical f64 only |
| **GPU-resident stateful functional (persistent device weights)** | "Don't re-upload weights every launch" | Weights are ≤ 82 f64 values = 656 bytes. Per-launch upload cost is nanoseconds, not a bottleneck. Adding persistence adds a lifecycle problem (invalidation on re-setup) for no measurable gain | Upload per launch (design §5 non-goal G5) |
| **VV10 non-local correlation kernel** | Modern dispersion-corrected functionals (ωB97M-V) use it | VV10 involves double-grid integration — it's a higher-level numerical task the host code owns (Libxc itself doesn't evaluate it; it exposes parameters). xcfun-master doesn't implement it either | Out of scope; unchanged from reference. Document that `wb97m-v` is unavailable |
| **D3 / D4 empirical dispersion corrections** | Every modern DFT paper needs one; Psi4 ships `dftd3` / `dftd4` interfaces | Separate library concern. Psi4 / PySCF interface to `s-dftd3`/`dftd4` *parallel* to the XC library; xcfun isn't expected to provide it | Point users to `dftd4` / `s-dftd3`. Same behaviour as the C++ reference |
| **Machine-learned functionals (Skala, libNXC)** | Hot research area in 2025–2026 | Different numerical contract (f32 / neural network weights, non-analytic derivatives). Belongs in a separate crate with different precision guarantees | Stay XC-analytic; ML is a future sibling product, not a feature |
| **Arbitrary custom functional DSL at runtime** | "Let me define beckex variants at startup" | Would re-invent Libxc's Maple pipeline. Can't verify numerical identity with C++ xcfun → violates Core Value | The 78 built-ins cover all named aliases in the reference. Users who want more → fork |
| **Changing the public xcfun C API** | "We could simplify it" | The whole project exists to be ABI-compatible. Any deviation means callers must port. Design §2.2 explicitly locks the API | C ABI exactly matches `xcfun-master/api/xcfun.h`; header diffed byte-for-byte in CI |
| **Pre-`XCFUN_API_VERSION=2` header compatibility** | Hypothetical legacy-caller support | No modern host uses it; reference dropped it. Adds a dead code path | Target exactly API v2 (design §5 non-goal) |
| **Libxc compatibility shim** | "Also accept Libxc functional IDs so we can replace Libxc too" | Libxc's ID space, functional set, and semantics differ. A shim would lie about behaviour and fail silently on edge cases | Wrong product. Users wanting Libxc should link Libxc. xcfun_rs replaces *xcfun*, not Libxc |

### Scope-item labels (per docs/design/00-overview.md §2.1)

The downstream consumer asked for an explicit label per in-scope item:

| Scope item | Classification | Rationale |
|---|---|---|
| Full public C API parity | Table stakes | Drop-in replacement is the entire Core Value |
| All 78 functionals | Table stakes | Named-functional coverage is the primary user contract |
| All named aliases (46+) | Table stakes | Host codes pass user strings verbatim |
| All 4 tunable parameters | Table stakes | Range-separated hybrids are dominant |
| All 31 `xcfun_vars` combinations | Table stakes | Every host-code evaluation mode needs one of these |
| All 3 evaluation modes | Table stakes | Contracted mode is the response-theory primitive |
| Derivative orders 0..6 (PD≤4, Contracted≤6) | **Differentiator** | The reason to pick xcfun_rs over Libxc |
| Native Rust API | Differentiator | Rust-ecosystem adoption driver |
| C ABI (cbindgen-generated) | Table stakes | Without it, no existing C/C++/Fortran host links against xcfun_rs |
| Python bindings (PyO3 0.28) | Table stakes | Most users consume via PySCF/Jupyter |
| CPU/GPU batch evaluation (unified cubecl kernel) | **Differentiator** | No competitor has a single-source CPU+CUDA+WGPU kernel |
| Validation harness (Rust vs C++ reference, 1e-12) | Differentiator (as a *product feature*, because it's CI-publicly-visible) | Buyers trust outputs because they see the gate |

## Feature Dependencies

```
API_VERSION / is_compatible_library
    └──supports──> all C FFI entry points

Functional registry (78 functionals)
    ├──requires──> CTaylor<T, N> AD engine (order 0..6)
    ├──requires──> Num trait + Taylor polynomial algebra
    └──feeds────> Alias table (46 entries)
                     └──feeds────> Functional::set(name, value)

Vars (31 combinations)  Mode (3)  order (0..6)
    └─all three──> eval_setup ──> Dependency check (bitflags)
                                     └──drives──> input_length / output_length

eval(density, out)
    ├──requires──> eval_setup configured
    ├──requires──> CTaylor AD engine
    └──depends on──> bit-flag → index-triple dispatcher

eval_vec(density, pitch, ...)
    ├──requires──> eval
    ├──on CPU──> std::thread::scope parallelism
    └──on GPU──> batch_on<R> + cubecl #[cube] kernel

batch_on<R: Runtime>
    ├──requires──> cubecl runtime (CpuRuntime | CudaRuntime | WgpuRuntime)
    ├──feature-gated──> `cuda` / `wgpu` cargo features
    └──fallback──> CPU when GPU unavailable

Python bindings (pyo3)
    ├──wraps──> Functional (native Rust API)
    ├──requires──> rust-numpy for zero-copy arrays
    └──exposes──> Vars, Mode, XcfunError (Python shadow types)

C ABI (xcfun-capi)
    ├──wraps──> Functional
    ├──requires──> cbindgen build step (generates xcfun.h)
    └──CI gate──> header diff vs xcfun-master/api/xcfun.h
```

### Dependency Notes

- **All 78 functionals depend on CTaylor<T, N>**: Every functional is coded as arithmetic over `CTaylor` values. If the AD engine doesn't match the reference coefficient-algebra bit-for-bit, every functional silently fails the 1e-12 parity gate. Phase ordering: AD engine must land *before* any functional is implemented for real.
- **Aliases depend on Functional::set**: Aliases are syntactic sugar that expand into repeated `set(functional_name, weight * outer_weight)` calls. The recursion depth is ≤ 2 (no alias of an alias in the reference source). Aliases can ship in the same phase as `set` or one later.
- **eval_vec depends on eval**: The scalar path must be correct first; vectorization is a *layer*, not a parallel implementation.
- **batch_on depends on eval and on cubecl**: The CPU batch path already uses a cubecl `CpuRuntime` (design §1.4.10), so GPU is not a separate code path — it's a runtime swap. This is the architectural keystone of the unified-kernel differentiator.
- **Python bindings depend on native Rust API**: Python wraps the Rust `Functional`. If the Rust API changes, PyO3 glue changes mechanically. Do not duplicate logic in Python glue.
- **C ABI depends on native Rust API**: Same layering. The C-ABI crate is a thin `unsafe` adapter; business logic lives in the core.
- **Every mode/order combination must be validated**: The CI gate is (78 functionals) × (31 vars) × (3 modes) × (7 orders) ≈ 50,000 tuples. Not all are legal — eval_setup rejects illegal combinations — but the harness must enumerate and test all legal ones.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed for a host code to *replace* C++ xcfun with xcfun_rs without changing anything else. This aligns with the Active requirements in PROJECT.md.

- [ ] `CTaylor<T, N>` AD engine, orders 0..6, bit-flag multilinear polynomial layout — essential for parity
- [ ] All 78 functionals from `list_of_functionals.hpp` — removing any is a regression from the reference
- [ ] All 46 aliases from `aliases.cpp` — user-facing names must resolve
- [ ] 4 tunable parameters (`EXX`, `RANGESEP_MU`, `CAM_ALPHA`, `CAM_BETA`)
- [ ] All 31 `Vars` combinations
- [ ] All 3 evaluation modes with order caps (PartialDerivatives≤4, Contracted≤6)
- [ ] C ABI (`xcfun-capi`) with cbindgen-generated header matching `xcfun.h` byte-for-byte where possible
- [ ] Python bindings (`xcfun-py` via PyO3 0.28 + rust-numpy 0.28, zero-copy f64)
- [ ] Single `#[cube]` kernel per functional, dispatchable to `CpuRuntime` (minimum), plus `CudaRuntime` (primary HPC target)
- [ ] `eval` and `eval_vec` on CPU backend
- [ ] Validation harness covering the full legal `(functional, vars, mode, order)` grid at ≤ 1e-12
- [ ] Numerical-parity CI gate (no merge on any > 1e-12)
- [ ] Zero heap allocation on per-point hot path (benchmark-enforced)
- [ ] Self-test (`self_test()` / `xcfun_test`) returning # of failing functionals
- [ ] Documentation: every public Rust item has rustdoc; Python docstrings; C header generated with comments

### Add After Validation (v1.x)

Features to land after the core is shipping and running in at least one external host integration.

- [ ] `WgpuRuntime` backend (relaxed 1e-9 tolerance for `erf`-using functionals) — cross-platform GPU users
- [ ] Criterion-based performance benchmark suite vs C++ xcfun (Goal S4: within ±10% wall-clock)
- [ ] Optional `tracing` subscriber for evaluation-path debugging — useful once users report perf anomalies
- [ ] AArch64 native-build CI (Apple Silicon via wgpu+Metal, ARM HPC via CPU)
- [ ] Windows binary release (Rust toolchain makes it low-cost)

### Future Consideration (v2+)

- [ ] Stream-overlapped async GPU path — push for >20% throughput gain first (PROJECT.md Out of Scope)
- [ ] Higher-order derivatives (>6) — requires new upstream reference algorithm
- [ ] User-plugin functional API — only if a Maple-style codegen pipeline is adopted; changes numerical-parity story
- [ ] Libxc-style functional coverage — only if user demand and resource capacity support it (currently: no)
- [ ] ML-functional hosting (Skala, libNXC bridge) — distinct product; f32 OK; different numerical contract

## Feature Prioritization Matrix

Prioritization relative to v1 ship. HIGH cost means > 2 weeks of engineering for one developer.

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `CTaylor<T, N>` AD engine (orders 0..6) | HIGH | HIGH | P1 |
| 78 functionals, algebraic port from C++ | HIGH | HIGH | P1 |
| 46 aliases | HIGH | LOW | P1 |
| 4 tunable parameters | HIGH | LOW | P1 |
| 31 `Vars` combinations | HIGH | MEDIUM | P1 |
| 3 evaluation modes | HIGH | HIGH | P1 |
| Native Rust API (`Functional`, `Vars`, `Mode`, `XcError`) | HIGH | MEDIUM | P1 |
| C ABI (cbindgen + `xcfun-capi`) | HIGH | MEDIUM | P1 |
| Python bindings (PyO3 + rust-numpy) | HIGH | MEDIUM | P1 |
| `eval` (per-point) | HIGH | LOW (given AD) | P1 |
| `eval_vec` on CPU | HIGH | MEDIUM | P1 |
| `#[cube]` unified kernel per functional | HIGH | HIGH | P1 (differentiator) |
| CUDA backend (`cubecl-cuda`) | HIGH | HIGH | P1 (primary GPU target) |
| Validation harness (full tuple grid, 1e-12 gate) | HIGH | HIGH | P1 |
| Self-test / `enumerate_*` / `describe_*` | MEDIUM | LOW | P1 |
| Version/compatibility hooks | MEDIUM | LOW | P1 |
| `Wgpu` backend (cross-platform) | MEDIUM | MEDIUM | P2 |
| Perf benchmarks vs C++ | MEDIUM | MEDIUM | P2 |
| `tracing` instrumentation | LOW | LOW | P2 |
| `no_std` core confirmation + tests | MEDIUM | LOW | P2 |
| AArch64 / Metal CI | LOW | MEDIUM | P2 |
| D3/D4 dispersion integration | LOW (out of scope) | MEDIUM | P3 (reject) |
| VV10 non-local correlation | LOW (out of scope) | HIGH | P3 (reject) |
| Libxc-style functional coverage | LOW (out of scope) | HIGH | P3 (reject) |
| User-supplied functional API | LOW (out of scope) | HIGH | P3 (reject) |
| Symbolic output | LOW (out of scope) | HIGH | P3 (reject) |
| Distributed evaluation | LOW (out of scope) | HIGH | P3 (reject) |

**Priority key:** P1 = must for v1 ship; P2 = next iteration; P3 = explicitly rejected (keep the rejection documented).

## Competitor Feature Analysis

| Feature | Libxc 7 (default in most hosts) | C++ xcfun 2.1 (reference) | ExchCXX + GauXC (GPU-first) | Skala / libNXC (ML) | xcfun_rs (this project) |
|---|---|---|---|---|---|
| Language | C (Fortran/Python bindings) | C++ (Fortran/Python bindings) | C++ (CUDA/HIP/SYCL) | Python + PyTorch (Skala), Python (libNXC) | Rust (native + C FFI + Python) |
| Functional coverage | ~400 | 78 | 17 built-in + Libxc wrap | 1 neural net (Skala) / arbitrary ML | 78 (parity with xcfun) |
| Named aliases | extensive | 46 | Kernel::* enum | — | 46 (parity) |
| Max derivative order | 4 | 6 | 1–2 | 1 | 6 (matches xcfun) |
| GPU support | Experimental CUDA (configure-time) | None (CPU only) | CUDA, HIP, SYCL (separate backends) | PyTorch GPU | CUDA + WGPU via cubecl, single kernel source |
| Batch/vectorized API | Yes (since Libxc 6) | Yes (`xcfun_eval_vec`) | Yes (primary API) | N/A (tensor-level) | Yes (matches reference + GPU) |
| Zero-copy NumPy | Via wrappers (PyLibxc copies) | Via pybind11 (copies) | No | Native in PyTorch | Yes (rust-numpy) |
| Rust bindings | Unofficial, thin | None | None | None | Native first-class |
| Memory safety | None | None | None | Python-level | Rust-enforced |
| License | MPL-2.0 | MPL-2.0 | BSD-3 | MIT / open | MPL-2.0 (inherited) |
| Build system | autotools + CMake | CMake | CMake | Python/pip | cargo (hermetic) |
| f32 path | Yes (Libxc 7) | No | Yes | Yes | **No** (rejected) |
| User-defined functionals | Maple code-gen | No | No | Trivially (NN weights) | No (rejected) |
| Dispersion (D3/D4) | No (separate lib) | No | No | N/A | No (rejected) |
| Target user | Every DFT host | Hosts needing order 5–6 | GauXC users / GPU-heavy hosts | ML-DFT researchers | Same as C++ xcfun, plus Rust ecosystem |

### What users of Libxc / NWChem / Psi4 / PySCF expect that xcfun_rs *does* provide (parity confirmed)

- Named-functional string API (`"b3lyp"`, `"pbe"`, `"camb3lyp"`, `"scan"`, `"r2scan"`)
- Both α/β and n/s variable paths
- Meta-GGA with either τ or Laplacian
- Range-separated hybrids (`camb3lyp`, `ldaerf*`) with all four tunable parameters
- Spin-unpolarized and spin-polarized evaluation
- Explicit-gradient path (`XC_A_AX_AY_AZ` et al.) for current-dependent functionals
- C header generation (cbindgen is our Libxc-style `libxc.h` equivalent)
- Python bindings
- Self-test
- Version query

### What users *might* expect that xcfun_rs does *not* provide (and why)

These are **legitimate feature gaps vs Libxc**, not bugs. If a user lists one as a blocker, the right answer is "use Libxc for this, xcfun_rs for the rest" — same division of labour as the current C++ xcfun ↔ Libxc split.

| User expectation | Not in xcfun_rs because | User should use | Effort to add later |
|---|---|---|---|
| Functionals outside the 78-list (e.g., `ωB97X-V`, `M11`, `PW6B95`, `MN15`) | Not in C++ xcfun reference. Adding them breaks the algorithmic-parity contract (nothing to verify against) | Libxc | HIGH — new AD-grade ports |
| VV10 non-local correlation | Double-grid integration; host-code responsibility | Libxc (parameters only) + host grid code | HIGH |
| D3 / D4 empirical dispersion | Parallel library concern, not XC | `s-dftd3`, `dftd4` | N/A (different product) |
| libxc `XC_CORRELATION_ONLY` / `XC_EXCHANGE_ONLY` switches | xcfun has no equivalent; ID-level exchange/correlation split exists instead (`slaterx` vs `vwn5c`) | Compose via `set()` calls | LOW (already available) |
| Automatic MPI distribution | Out of scope (host owns grid distribution) | Host MPI layer | N/A |
| "Set functional by name" with Libxc IDs (numerical) | Libxc ID space, not xcfun's | Libxc | N/A |
| f32 GPU kernels | Cannot meet 1e-12 | Another library if f32 OK | N/A |
| Plugin / user-functional API | Cannot verify against C++ reference | Fork xcfun_rs and add functional by hand | HIGH + breaks parity story |

### Integrations xcfun_rs should target first

Based on who currently integrates C++ xcfun:

1. **PySCF** (`pyscf.dft.xcfun`) — already has a `libxc` ↔ `xcfun` switch; xcfun_rs can land as a conda-forge / pip replacement wheel with zero code changes on the PySCF side (assuming parity)
2. **Psi4** (`psi4.driver.procrouting.dft`) — uses xcfun for orders beyond Libxc's cap in response calculations; drop-in linkable
3. **Dalton / LSDalton** — authors of xcfun; highest-fidelity parity target
4. **ADF / AMS** — uses xcfun for CAMY-B3LYP and range-separated kernels where Libxc has gaps
5. **ERKALE** — small basis-set code historically tied to xcfun
6. **gpu4pyscf** — current Python-level path; xcfun_rs could offer a GPU-native alternative within PySCF

## Sources

- [xcfun-master C++ reference](../../../xcfun-master/) (vendored in-tree; authoritative for functional set, aliases, order cap, API surface)
- [xcfun-master/api/xcfun.h](../../../xcfun-master/api/xcfun.h) (21 functions, 3 enums, `XCFUN_MAX_ORDER=6`, `XCFUN_API_VERSION=2`)
- [xcfun-master/src/functionals/list_of_functionals.hpp](../../../xcfun-master/src/functionals/list_of_functionals.hpp) (78 functional IDs enumerated)
- [xcfun-master/src/functionals/aliases.cpp](../../../xcfun-master/src/functionals/aliases.cpp) (46 aliases counted via `grep -c "^    {\""`)
- [docs/design/00-overview.md](../../../docs/design/00-overview.md) (in-scope / out-of-scope catalogue, goals G1..G7, success S1..S9)
- [docs/design/03-api-surface.md](../../../docs/design/03-api-surface.md) (native Rust + C ABI + Python surfaces, error mapping, output layout)
- [.planning/PROJECT.md](../../PROJECT.md) (active requirements, constraints, out-of-scope reasoning)
- [Libxc homepage (libxc.gitlab.io)](https://libxc.gitlab.io/) — LDA/GGA/mGGA/GH/RSH support, derivative orders 1–4, Maple code-generation strategy, experimental CUDA
- [Libxc functionals catalogue](https://libxc.gitlab.io/functionals/) — confirms ~400 functionals, CAM-B3LYP support, ID numbering scheme
- [XCFun documentation (xcfun.readthedocs.io)](https://xcfun.readthedocs.io/en/latest/) — changelog, pybind11-based Python API, `xcfun_user_eval_setup` user-friendly entry
- [ExchCXX GitHub (wavefunction91/ExchCXX)](https://github.com/wavefunction91/ExchCXX) — CUDA / HIP / SYCL backends, 17 built-in kernels, Libxc wrap for the rest
- [GauXC GitHub (wavefunction91/GauXC)](https://github.com/wavefunction91/GauXC) — GPU-distributed XC integrator consuming ExchCXX
- [PySCF DFT documentation](https://pyscf.org/user/dft.html) — confirms runtime switch between Libxc and xcfun, both libraries first-class
- [PySCF xcfun bindings source](https://github.com/pyscf/pyscf/blob/master/pyscf/dft/xcfun.py) — real-world usage pattern for an xcfun consumer
- [gpu4pyscf releases (pyscf/gpu4pyscf)](https://github.com/pyscf/gpu4pyscf/releases) — third-order XC derivatives on GPU (Libxc 0.7), refactored libxc interface, removed rarely used functionals
- [SCM ADF 2025.1 documentation — Density Functionals (XC)](https://www.scm.com/doc/ADF/Input/Density_Functional.html) — confirms range-separated functionals require XCFun in ADF; CAM-B3LYP vs CAMY-B3LYP distinction; LibXC 7 adoption
- [ORCA 6.1 manual — DFT](https://orca-manual.mpi-muelheim.mpg.de/contents/modelchemistries/DensityFunctionalTheory.html) — meta-GGA (TPSS, SCAN) use patterns
- [VASP wiki — METAGGA / LIBXC1](https://www.vasp.at/wiki/index.php/METAGGA) — Laplacian- vs τ-dependent meta-GGA classification
- [Microsoft Skala (github.com/microsoft/skala)](https://github.com/microsoft/skala) — modern ML-functional comparator
- [libNXC (github.com/semodi/libnxc)](https://github.com/semodi/libnxc) — ML-functional library comparator
- [Psi4 DFT documentation](https://psicode.org/psi4manual/master/dft.html) — D3/D4 dispersion handled by `s-dftd3`/`dftd4`, not the XC library
- [arXiv 1203.1739 (original Libxc paper)](https://arxiv.org/abs/1203.1739) — baseline functional coverage
- [ScienceDirect — Recent developments in Libxc](https://www.sciencedirect.com/science/article/pii/S2352711017300602) — Maple code-gen, coverage growth over time
- [ChemRxiv 2025-k7zbn — GPU-accelerated XC evaluation algorithms](https://chemrxiv.org/doi/pdf/10.26434/chemrxiv-2025-k7zbn) — current-state (2025) GPU XC performance comparison
- [Q-Chem 5.0 manual — TDDFT Hessian section](https://manual.q-chem.com/5.0/sect-tddft.html) — confirms TDDFT analytical Hessian requires up to 4th-order XC derivatives (xcfun_rs comfortably supports 6th)

---
*Feature research for: DFT XC functional library (Rust reimplementation of xcfun)*
*Researched: 2026-04-18*
