# Architecture Research

**Domain:** Rust reimplementation of a numerical library (exchange–correlation functionals for DFT) with automatic differentiation and a unified CPU + CUDA + Wgpu kernel layer (`cubecl`).
**Researched:** 2026-04-18
**Overall confidence:** HIGH

This document answers "how are analogous Rust numerical/GPU libraries structured in 2026, and does the proposed xcfun_rs 7-crate workspace map cleanly to those precedents?" It evaluates the layout in `docs/design/01-source-tree.md` and `docs/design/05-module-responsibilities.md` against the way `burn`, `candle`, `cubecl` itself, `cubek`, `polars`, `scirs2`, and the PyO3 / cbindgen ecosystems organise their workspaces. It then gives a concrete build order for the milestones.

---

## 1. Reference layouts in the 2026 Rust numerical-GPU ecosystem

| Project | Role it plays | Workspace pattern | Crates (abridged) | Applicable lesson |
|---------|---------------|-------------------|-------------------|-------------------|
| `cubecl` (tracel-ai) | GPU kernel DSL the project uses as dependency | Layered core / IR / backends, umbrella crate re-exports | `cubecl-core`, `cubecl-ir`, `cubecl-opt`, `cubecl-runtime`, `cubecl-cuda`, `cubecl-wgpu`, `cubecl-cpu`, `cubecl-macros`, `cubecl-std`, umbrella `cubecl` | Kernel definitions are independent of the runtime that executes them (HIGH, Context7 + cubecl docs) |
| `burn` (tracel-ai) | Tensor / deep-learning framework on top of cubecl | Backend-per-crate with decorator backends; user-facing `burn` umbrella | `burn-core`, `burn-tensor`, `burn-autodiff`, `burn-fusion`, `burn-ndarray`, `burn-cuda`, `burn-wgpu`, `burn-tch`, `burn-train`, `burn` (umbrella) | AD lives in its own decorator crate, not inside the tensor core (HIGH, GitHub README) |
| `candle` (huggingface) | Minimalist tensor / LLM framework | Core + NN + kernels + hardware-specific kernel crates | `candle-core`, `candle-nn`, `candle-kernels` (CUDA), `candle-metal-kernels`, `candle-transformers`, `candle-pyo3`, `candle-examples` | Device-specific kernel source lives in a dedicated sibling crate, pulled in behind feature flags (HIGH, GitHub README) |
| `cubek` (tracel-ai) | Kernel library built *on top of* cubecl | One crate per operation class, blueprint/routine split | `cubek-matmul`, `cubek-convolution`, `cubek-reduce`, … | Downstream cubecl users separate kernels by operation class and compile-time blueprint, which xcfun_rs mirrors with "one `#[cube]` per (functional, order)" (HIGH, Context7 CubeK llms.txt) |
| `polars` (pola-rs) | Data-frame library, large workspace precedent | 20+ layered crates; user-facing `polars` umbrella that re-exports | `polars-core`, `polars-lazy`, `polars-ops`, `polars-io`, `polars-arrow`, … + `polars` | Layered crate graph where users depend on a single umbrella crate is the dominant 2026 Rust pattern (HIGH, DeepWiki polars package-structure) |
| `scirs2` | SciPy-compatible numerical library | 29 workspace crates, flat layout | `scirs2-core`, `scirs2-linalg`, `scirs2-stats`, `scirs2-autograd`, … | Even for millions of LOC, the flat workspace layout remains the dominant choice (MEDIUM, scirs2 README) |
| `rustls-ffi` | FFI/C ABI crate pattern | Core `rustls` + dedicated `-ffi` crate + generated C header | `rustls`, `rustls-ffi` | Separating the C ABI into an `-ffi` crate with `crate-type = ["cdylib", "staticlib"]` + cbindgen build script is the established pattern (HIGH, rustls-ffi README) |
| `pyo3` projects (tokenizers, polars-python, candle-pyo3) | Python bindings pattern | Separate binding crate with `cdylib` + maturin | Varies | Python bindings belong in their own crate depending on the Rust façade, never on internal crates (HIGH, PyO3 docs) |

The common shape across every one of these projects:

```
AD/math core  →  typed domain core  →  kernel definitions  →  runtime adapter  →  user-facing façade
                                                                                    │
                                                                                    ├── C ABI crate (cdylib)
                                                                                    └── Python bindings crate (cdylib)
```

This is precisely the shape of the proposed xcfun_rs workspace.

---

## 2. Verdict on the proposed 7-crate workspace

**The 7-crate layout maps cleanly onto the 2026 Rust numerical/GPU precedent.** The boundaries in `05-module-responsibilities.md` match or are stricter than the analogous boundaries in `burn`/`candle`/`cubecl`. Every major seam that those projects found it useful to expose exists in the proposal:

| Seam found in analogous projects | xcfun_rs equivalent | Present? |
|---|---|---|
| Pure AD engine, no domain types | `xcfun-ad` | ✅ |
| Domain core with static registry + scalar path | `xcfun-core` | ✅ |
| Kernel definitions independent of runtime | `xcfun-kernels` | ✅ |
| Runtime adapter / backend selection | `xcfun-gpu` | ✅ |
| User-facing façade that re-exports | `xcfun-rs` | ✅ |
| C ABI with `cdylib` + header generation | `xcfun-capi` | ✅ |
| Python bindings (PyO3 + maturin) | `xcfun-py` | ✅ |
| Binary-only validation harness separate from library graph | `validation/` | ✅ |
| `xtask` for codegen and release tasks | `xtask/` | ✅ |

One weak spot — addressed in §5 — is that the boundary between **host-side per-point evaluation** (in `xcfun-core`) and **device-side per-point evaluation** (in `xcfun-kernels`) is maintained via a "shared algebraic spec" mechanism that is asserted but not crate-enforced. That seam should be reified before the kernel crate is implemented.

---

## 3. System overview (confirmed)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                               User surface                                │
│   Rust app              C app               Python app                    │
│      │                    │                     │                         │
│      ▼                    ▼                     ▼                         │
│ ┌──────────┐       ┌────────────┐        ┌──────────┐                     │
│ │xcfun-rs  │       │xcfun-capi  │        │xcfun-py  │                     │
│ │ façade   │       │(cdylib /   │        │(cdylib + │                     │
│ │          │       │ staticlib) │        │ maturin) │                     │
│ └────┬─────┘       └──────┬─────┘        └────┬─────┘                     │
│      │   depends          │    re-exports     │    re-exports             │
│      │   on core+gpu      │    xcfun-rs       │    xcfun-rs               │
└──────┼────────────────────┴───────────────────┴───────────────────────────┘
       │
┌──────▼───────────────────────────────────────────────────────────────────┐
│                           Orchestration layer                             │
│   ┌────────────────────────────────────────────────────────────────┐     │
│   │                         xcfun-gpu                               │     │
│   │  Batch<'fun, R>, Backend enum, device buffer pool,              │     │
│   │  auto_backend(), tracing spans                                  │     │
│   │  ─ owns the only cubecl::Runtime instance in the workspace      │     │
│   └─────────────────────────┬─────────────┬──────────────────────────┘    │
│                             │             │                               │
└─────────────────────────────┼─────────────┼───────────────────────────────┘
                              │             │
                              ▼             ▼
         ┌─────────────────────┐     ┌─────────────────────┐
         │  xcfun-kernels      │     │  xcfun-core         │
         │  #[cube] per-point  │     │  Registry,          │
         │  evaluator;         │     │  DensVars, Vars,    │
         │  dispatch_table;    │     │  scalar CPU eval,   │
         │  DensVarsDev mirror │     │  78 functionals,    │
         │                     │     │  50+ aliases        │
         │  depends on         │     │                     │
         │  xcfun-core (data   │     │                     │
         │  + layout only)     │     │                     │
         │  and cubecl         │     │                     │
         └──────────┬──────────┘     └──────────┬──────────┘
                    │                           │
                    └──────────────┬────────────┘
                                   ▼
                       ┌──────────────────────┐
                       │     xcfun-ad         │
                       │  CTaylor<T, N>,      │
                       │  Num trait,          │
                       │  expand::{inv, exp,  │
                       │   log, pow, sqrt,    │
                       │   erf}               │
                       │                      │
                       │  (no_std-capable;    │
                       │  no domain types;    │
                       │  stack-only)         │
                       └──────────────────────┘

          ┌────────────────┐                ┌────────────────┐
          │  validation    │                │  xtask         │
          │  (bin, links   │                │  (bin, codegen │
          │  C++ xcfun via │                │  + release +   │
          │  cc / cxx)     │                │  boundary      │
          │                │                │  enforcement)  │
          └────────────────┘                └────────────────┘
```

The diagram matches `docs/design/05-module-responsibilities.md` section 10 ("Module boundary rules") precisely. Key dependency directions to notice:

1. Every arrow flows **downward** (façade → orchestration → kernel/core → AD). No cycles.
2. `xcfun-kernels` depends on `xcfun-core` for **data and layout only** (the `VARS_TABLE`, the `DensVars` field order, the `FunctionalId` enum). It does *not* depend on `xcfun-core`'s scalar evaluation paths — a seam worth enforcing with a `[features]` wall, since `xcfun-core` is `std` but kernels are `no_std` inside `#[cube]` bodies.
3. `xcfun-rs` has exactly two dependencies: `xcfun-core` and `xcfun-gpu`. All other crates depend on `xcfun-rs` (C ABI, Python, examples).
4. `validation/` and `xtask/` are leaves on the tree and may depend on `anyhow`; no library crate may. That invariant is enforced by `cargo xtask check-no-anyhow` per `05-module-responsibilities.md` §9.

---

## 4. Component responsibilities (confirmed, with one clarification)

| Component | Responsibility | Typical implementation (2026 Rust precedent) | Status |
|---|---|---|---|
| `xcfun-ad` | Pure AD primitives (`CTaylor<T, N>`, `Num`, scalar series expansions) | Same shape as `burn-autodiff`, `sophus_autodiff`, and `ad-trait` — a math-only crate with no knowledge of the domain | Keep as proposed |
| `xcfun-core` | Static registry + scalar CPU evaluation + `DensVars<T>` | Matches `candle-core`, `polars-core`, `burn-core` — the typed-domain core where all static data lives | Keep as proposed |
| `xcfun-kernels` | `#[cube]` per-point functional evaluators | Mirrors `candle-kernels` and `candle-metal-kernels`; also the pattern `cubek-matmul`/`cubek-convolution` use on top of cubecl | Keep; add §5 clarifications |
| `xcfun-gpu` | `Batch` lifecycle, `Backend` enum, buffer pool, `auto_backend()`, transfer minimisation | Analogous to `burn-wgpu` + `burn-cuda` joined under one orchestration crate; `candle`'s feature-gated backend split is the alternative | Keep as proposed (one crate is correct — see §6) |
| `xcfun-rs` | Thin façade, re-exports only | Matches the `polars`, `burn`, `candle-core` pattern | Keep as proposed |
| `xcfun-capi` | C ABI via `cdylib` + `staticlib`, cbindgen build step | Matches `rustls-ffi` | Keep as proposed |
| `xcfun-py` | PyO3 + maturin bindings | Matches `candle-pyo3`, `tokenizers`, `polars-python` | Keep as proposed |

The implicit lesson from `burn`, `candle`, and `polars` is that "one crate per responsibility" beats "one crate per feature". The proposal is already on the right side of that distinction.

---

## 5. Architectural patterns worth calling out

### 5.1 Sans-runtime kernel crate (from cubecl / cubek / candle-kernels)

**What:** `xcfun-kernels` defines kernels generic over `R: cubecl::Runtime` and `F: Float` but never **instantiates** a runtime. Instantiation, device selection, and buffer ownership are strictly the concern of `xcfun-gpu`. This pattern is explicit in cubecl's own architecture ("the generated code remains valid Rust code, allowing it to be bundled without any dependency on the specific runtime" — cubecl GitHub) and reflected in how `cubek` structures its matmul / reduce / convolution crates.

**When to use:** Always for any kernel-bearing crate. It is the prerequisite for swapping backends behind feature flags without touching kernel source.

**Trade-offs:** Creates a small amount of repetition at the launch site (every launch site picks the concrete `R` and passes buffers), but that repetition is the seam that lets CPU / CUDA / Wgpu coexist.

```rust
// xcfun-kernels: kernel is runtime-agnostic
#[cube(launch_unchecked)]
pub fn eval_batch_kernel<F: Float>(
    weights: &Array<F>, density: &Array<F>, result: &mut Array<F>,
    /* … */
    #[comptime] vars: u32, #[comptime] order: u32, #[comptime] mode: u32,
) { /* body calls only #[cube] fns */ }

// xcfun-gpu: runtime-specific launch site
fn launch_on<R: Runtime>(batch: &Batch<'_, R>, n: u32) {
    unsafe {
        eval_batch_kernel::launch_unchecked::<f64, R>(
            &batch.client,
            CubeCount::Static((n + 255) / 256, 1, 1),
            CubeDim::new(256, 1, 1),
            ArrayArg::from_raw_parts::<f64>(&batch.weights, /* … */),
            /* … */
        );
    }
}
```

### 5.2 Backend decorator (from burn-autodiff, burn-fusion)

**What:** Rather than bake AD into the tensor core, `burn` wraps *any* backend with `burn-autodiff`. xcfun_rs does the Rust analogue at the type level: every functional is `<T: Num>` so the same source compiles for `T = f64` (CPU scalar, no derivatives) and `T = CTaylor<f64, N>` (forward-mode AD). The `Num` trait is the decorator seam.

**When to use:** Any time you want the same algorithm compiled both with and without AD. This is *the* reason the 1e-12 parity target is achievable: the AD engine is not a separate implementation to keep in sync; it's the same source with a different `T`.

**Trade-offs:** Requires all functional code to be generic. Monomorphisation costs (code size grows by factor of 7 for seven `N` instantiations per functional) are real — the proposal accepts this explicitly.

### 5.3 Static const registry (from cubek blueprints, candle model definitions)

**What:** The functional / alias / vars tables are `static` in `.rodata` (not lazy, not heap-allocated). This matches the "blueprint" pattern `cubek` documents (`#[comptime]` blueprint struct drives kernel specialisation) and the `.rodata` static array pattern used throughout the Rust scientific ecosystem (`scirs2`, `ndarray-linalg`).

**When to use:** When the data is known at compile time and doesn't change at runtime. xcfun_rs's 78-entry `FUNCTIONAL_DESCRIPTORS` is a textbook case.

**Trade-offs:** Codegen required to keep the tables in sync with the C++ reference. The proposal handles this via `xtask codegen`, which is the standard pattern (c.f. `cubecl-macros`, `burn-import`).

### 5.4 Shared algebraic spec — reify as a crate or a macro

**What:** `docs/design/06-cubecl-strategy.md` §4.1 says each helper in `xcfun-core::functionals::shared` has two forms (`<name>_host<T: Num>` and `#[cube] <name>_dev<F: Float, N>`) "derived from a shared algebraic spec". The **shared spec is currently an aspiration, not an artifact**. This is the single most fragile seam in the proposal.

**Recommendation:** Either (a) make the `#[cube]` form of every helper *the* form and compile it to the CPU through `CpuRuntime` (the cubecl model), eliminating the host version entirely, or (b) introduce a proc-macro `#[functional_helper]` that takes a single body and emits both forms. Option (a) is closer to what cubecl's own examples do and what `cubek` recommends; option (b) is closer to the design doc's current wording. **Option (a) is simpler, is already implied by §6.2 ("Suitable for cubecl (on CPU via CpuRuntime) — exactly the same kernels"), and is how burn routes all backends through cubecl. Pick (a).**

If (a) is adopted, `xcfun-core` no longer needs to host scalar-CPU evaluation logic; it becomes registry + dispatch only, and all per-point evaluation moves into `xcfun-kernels`. This is a simplification the design doc should accept as a Phase 3 decision.

### 5.5 Thin façade (polars, burn, candle-core)

**What:** `xcfun-rs` exports only re-exports. Users write `use xcfun_rs::{Functional, Vars, Mode};`. This matches polars (umbrella crate for 20+ internal crates), burn (umbrella crate), and every maturing Rust library ecosystem.

**When to use:** Always, for any workspace with more than two library crates.

**Trade-offs:** A small amount of module duplication in `lib.rs` (one `pub use` line per export), but it decouples public API from internal file layout, exactly matching the Rust API Guidelines' facade advice.

### 5.6 C ABI as a dedicated cdylib crate (rustls-ffi, libsodium bindings)

**What:** `xcfun-capi` holds every `extern "C" fn`, every `#[repr(C)]` type the C ABI needs, and the `cbindgen` build script. No other crate contains C-visible symbols. This matches `rustls-ffi` exactly and the documented FFI best practice ("separate the FFI layer from the main library and move the unsafe code into a new crate").

**When to use:** Always when a C ABI is a deliverable. Mixing `extern "C"` into a library crate that also has a Rust API creates soundness risk (raw pointers escape the type system into safe code) and build friction (every downstream Rust user pays for `cbindgen`).

### 5.7 Python bindings as a dedicated cdylib crate (candle-pyo3, polars-python, tokenizers)

**What:** `xcfun-py` depends on `xcfun-rs` and exposes PyO3 bindings + maturin packaging. It is never pulled into another crate's dependency graph.

**Trade-offs:** Exactly the same as the C ABI — the crate exists precisely to keep its dependencies (PyO3, rust-numpy) out of the library graph.

---

## 6. Data flow (matches `04-control-flow.md`)

### 6.1 Setup flow

```
User --set("b3lyp", 1.0)--> xcfun-rs::Functional::set
                                     │
                                     ▼
                            xcfun-core::registry::lookup_alias
                                     │
                                     ▼
                    for each AliasTerm:
                        settings[id] += weight
                        active_ids.push(id)
                        depends |= descriptor.depends
                                     │
                                     ▼
                 User --eval_setup(vars, mode, order)--> validate depends
                                     │
                                     ▼
                                 Ready state
```

### 6.2 Per-point (scalar) evaluation flow — single grid point

```
input density slice &[f64]
        │
        ▼
for order O ∈ {0, 1, 2, 3, 4} (dispatcher branch):
    seed CTaylor<f64, ceil(log2(O+1))> inputs with VAR* slots
        │
        ▼
DensVars::build(vars, &inputs)        [xcfun-core or xcfun-kernels]
        │
        ▼
for each id in active_ids:
    out += settings[id] · FUNCTIONAL_DESCRIPTORS[id].fp{O}(&d)
        │
        ▼
output[k] = out.c[bit pattern for (i,j,…)]      [xcfun-core dispatcher]
        │
        ▼
caller-owned output slice &mut [f64]
```

### 6.3 Batch evaluation flow

```
User --eval_vec(host density, host out, n)--> xcfun-rs
        │
        ▼
xcfun-gpu::Batch<'fun, R>::reserve(n)   // allocate device buffers (once, or on growth)
        │
        ▼
Batch::upload_density(host slice)       // memcpy HtoD (40 MB @ 1M GGA points)
        │
        ▼
Batch::launch(n)
        │
        ▼
xcfun-kernels::eval_batch_kernel<R>::launch_unchecked(client, cube_count, cube_dim, weights, density, result, metadata)
        │
        ▼
per thread p:
    DensVarsDev::build(...)
    out = Σ weights[id] · call_functional(id, &d)
    result[p * pitch_out ...] = out.c[...]
        │
        ▼
Batch::download_result(host out slice) // memcpy DtoH (48 MB @ 1M GGA points)
        │
        ▼
User
```

### 6.4 Key data flows

1. **Registry lookup (cold path):** string → `FunctionalId` → `FunctionalDescriptor` slot. Happens at `set` / `eval_setup` time, never on the hot path. Strings and hash-lookups never leave the host.
2. **Per-point evaluation (hot, CPU):** stack-resident `DensVars<CTaylor<f64, N>>` (~14.5 KiB at order 6), no heap allocation, one function-pointer call per active functional per point.
3. **Batch evaluation (hot, GPU):** host-to-device memcpy of density, kernel launch with `#[comptime]` `(vars, order, mode)`, device-to-host memcpy of result. Weights + active IDs are uploaded once per `Batch` and persist across launches.
4. **Error path:** `XcError` bubbles up from `xcfun-core` through `xcfun-rs`. In `xcfun-capi` it is converted to an integer return code via `XcError::as_c_code`. In `xcfun-py` it is converted to a Python `XcfunError` exception. No panics cross the C ABI (caught by `catch_unwind` in every entry point).

---

## 7. Build order — the dependency DAG dictates phasing

The workspace is a strict DAG. Every downstream crate is unbuildable until its upstream is buildable *and* passes validation. This gives a deterministic phase order:

```
Phase      Crate                 Depends on                       Deliverable
─────────  ────────────────────  ───────────────────────────────  ──────────────────────────────────────
Phase 1    xcfun-ad              (nothing)                         CTaylor + Num + expansion tables, proptests
Phase 2    xcfun-core            xcfun-ad, bitflags, thiserror     Registry, DensVars, Vars, Mode, LDA functionals (5)
                                                                   scalar single-point eval, test_self passes for LDA
Phase 3    xcfun-kernels         xcfun-core, cubecl                #[cube] eval_batch_kernel on CpuRuntime,
                                                                   parity with xcfun-core scalar path at 5e-15
Phase 4    xcfun-gpu             xcfun-kernels, cubecl-cpu,        Batch<'fun, R> + buffer pool;
                                 cubecl-cuda (feat),               CPU batch round-trip green
                                 cubecl-wgpu (feat), tracing
Phase 5    xcfun-core (complete) —                                 All 78 functionals + 50+ aliases;
                                                                   validation harness reaches 1e-12 on CPU
Phase 6    xcfun-rs              xcfun-core, xcfun-gpu             Native Rust API surface (façade), examples
Phase 7    xcfun-capi            xcfun-rs, cbindgen                C ABI + generated xcfun.h diff-matches xcfun-master
Phase 8    xcfun-py              xcfun-rs, pyo3, numpy             Python wheel via maturin
```

Rationale for this ordering (matches `docs/design/05-module-responsibilities.md` §10 boundary rules and the published DAG):

- **Phase 1 must come first.** Nothing else compiles without `CTaylor` and `Num`. The AD engine also has the smallest API surface and the easiest property-test story (ring axioms, `exp`/`log` inverses), so it is a natural fast-feedback starting point.
- **Phase 2 before Phase 3.** The kernel crate needs `Vars`, `VARS_TABLE`, `DensVars` field order, and `FunctionalId` from `xcfun-core`. Reverse order is impossible.
- **Phase 3 before Phase 4.** Writing `Batch` without a kernel to launch is premature; `xcfun-gpu` can only be validated once `eval_batch_kernel` runs under `CpuRuntime`.
- **Phase 4 before Phase 5.** Wiring up the remaining 73 functionals is meaningful only when a batch path exists, since the validation harness drives batches, not single points.
- **Phase 5 before Phase 6.** `xcfun-rs` is a façade; re-exporting half-populated APIs wastes effort.
- **Phase 6 before Phase 7 & 8.** C ABI and Python both depend on the Rust façade. Inverting the order leads to ABI churn every time the façade changes.

The design document's §5 ("Workspace model with 7 library crates + validation/xtask") is compatible with this phasing; what it omits is the **within-crate** sequencing of "LDA functionals first, then GGA, then metaGGA" inside the registry. That sequencing belongs inside Phase 2/5.

### 7.1 Phase 0 (prerequisite, not a milestone)

Set up workspace skeleton (`Cargo.toml [workspace]`, `rust-toolchain.toml` pinned to 1.85, `deny.toml`, `.cargo/config.toml`, `cbindgen.toml` stub, `xtask` crate, `validation/` skeleton). This is zero-code scaffolding that unblocks Phase 1.

### 7.2 Phase 4.5 (implicit milestone)

Before Phase 5's long tail of 73 functionals, the validation harness (`validation/`) must be working end-to-end with the LDA functionals already implemented. This is the "first 1e-12 greenlight" moment. It requires `xcfun-core`'s scalar path, `xcfun-gpu`'s CPU batch path, and `validation/`'s C++ shim all working. Without this, the remaining functionals cannot be validated as they are added.

---

## 8. Scaling considerations

| Scale dimension | At 5 functionals (LDA MVP) | At 78 functionals | At + future additions |
|---|---|---|---|
| Compile time | ~5 s clean, sub-second incremental | ~45–90 s clean (monomorphisation of 7 × 78 AD instantiations is the bottleneck) | Pressure on rustc; may warrant `#[inline(never)]` on cold `fp{N}` entries, or splitting functionals into per-family `pub mod` boundaries |
| Binary size | ~2 MB cdylib | ~15–25 MB cdylib estimated (each functional × 7 orders × 2 PTX cubins) | Consider `profile.release.opt-level = "s"` for `xcfun-capi` release builds |
| PTX / WGSL compile time (cubecl) | One kernel, fast | 78 kernels × (CPU/CUDA/WGPU) = ~234 compile units | Stay on `launch_unchecked` to avoid per-launch check overhead; enable cubecl persistent cache |
| Stack frame per point | ≈ 8 KiB @ order 4 | ≈ 14.5 KiB @ order 6 | Already accommodated by CUDA 1 MiB per-thread stack; metaGGA at order ≥ 5 should be profiled on consumer GPUs where register pressure hurts occupancy |
| Points per batch | 100 k | 1 M | 10 M would require chunked streaming (not in v1 scope per `docs/design/06-cubecl-strategy.md` §8) |

### Scaling priorities

1. **First bottleneck — compile time.** With 78 functionals × 7 AD orders, `xcfun-core` becomes the slowest crate to recompile. Mitigation: split functionals into per-family files and keep each functional in its own `mod` (already in the design); consider `[profile.dev-fast]` with `opt-level=1` locally.
2. **Second bottleneck — PTX size.** 78 kernels × 7 orders × 3 runtimes is a large compile matrix. Mitigation: feature-gate `cuda` and `wgpu`; default to `cpu` only. Already designed-in per §6 feature flags.
3. **Third bottleneck — register pressure at order 6 on consumer GPUs.** `CTaylor<f64, 6>` is 512 B per variable; `DensVars<CTaylor<f64, 6>>` exhausts registers on GPUs with < 64 KiB register file per SM. Mitigation: document A100 as the minimum validated target for order 6; smaller GPUs fall back to order ≤ 4 automatically.

---

## 9. Anti-patterns to avoid (from the 2026 ecosystem's lessons)

### Anti-Pattern 1: "One crate for everything"

**What people do:** Put AD, registry, kernels, GPU, C ABI, Python in a single crate with big `[features]`.
**Why wrong:** Every feature flag multiplies compile time; no-std-ness is untenable when `pyo3` is in the same graph; C ABI symbols leak into pure-Rust downstream users; compile matrix explodes.
**Do this instead:** The proposed 7-crate split. Each crate has one set of dependencies. Feature flags within `xcfun-gpu` select **backends**, not unrelated features. (Exactly what the proposal does.)

### Anti-Pattern 2: Duplicating the scalar CPU path and the GPU kernel path

**What people do:** Write `fn pw92c_cpu(d: &DensVars<f64>) -> f64` and `#[cube] fn pw92c_gpu<F: Float>(d: &DensVarsDev<F>) -> F` as separate implementations.
**Why wrong:** Every algebraic change must be ported twice; parity drift is inevitable; 1e-12 becomes unprovable.
**Do this instead:** Write each functional once, generic over `T: Num`. Run the same source through both the scalar AD dispatcher and the `#[cube]` kernel. This is the reason `Num` is implemented for `f64`, `CTaylor<f64, N>`, and `cubecl`'s `F: Float`. The proposal's §5.4 correctly identifies this as the single most important design rule — it just needs to be reified (see §5.4 Recommendation above).

### Anti-Pattern 3: Let the C ABI crate re-export Rust types

**What people do:** Public Rust symbols in the `xcfun-capi` crate, so downstream Rust users pick it up "for convenience".
**Why wrong:** Downstream Rust users then transitively depend on `cbindgen` (a build-time tool pulling a large tree), pay for `cdylib` linking, and lose `catch_unwind` overhead on every call.
**Do this instead:** `xcfun-capi` is a leaf crate. Its `lib.rs` contains only `extern "C"` items. Downstream Rust users depend on `xcfun-rs`, never on `xcfun-capi`. (Already enforced by the proposal.)

### Anti-Pattern 4: Let `cubecl::Runtime` types leak into kernel source

**What people do:** Write kernels generic over `R: Runtime` and call runtime-specific client methods inside the kernel body.
**Why wrong:** Kernels must be pure `#[cube]` bodies; calling host-side runtime methods breaks the macro's purity check; lowering to WGSL fails.
**Do this instead:** Kernels are generic over `F: Float` only. The `R: Runtime` parameter lives on the launcher (`launch_unchecked::<F, R>(...)`) in `xcfun-gpu`, which is host code. (Already enforced by the proposal.)

### Anti-Pattern 5: Rely on lazy-static / `OnceCell` for the registry

**What people do:** Use `once_cell::sync::Lazy` for `FUNCTIONAL_DESCRIPTORS` because "it's easier".
**Why wrong:** Pulls in a dependency, adds synchronisation overhead, defeats `.rodata` placement, complicates `no_std` story.
**Do this instead:** `static FUNCTIONAL_DESCRIPTORS: [FunctionalDescriptor; NR_FUNCTIONALS] = […];` with all fields `const`-initialisable. This is what the proposal specifies (`02-data-structures.md` §6). Requires `EvalFn<const N>` to be a `fn` pointer (function item), not a closure — which the proposal respects.

---

## 10. Integration points

### External services / runtimes

| Integration | Pattern | Notes |
|---|---|---|
| `cubecl::CpuRuntime` | Always on; `#[cube]` lowered to native Rust | Default backend. Validated at 1e-12 against scalar `xcfun-core`. |
| `cubecl::CudaRuntime` | Feature `cuda`; pulls `cubecl-cuda 0.10.0-pre.3` | Primary HPC target. Requires NVIDIA driver + NVCC at build time. |
| `cubecl::WgpuRuntime` | Feature `wgpu`; pulls `cubecl-wgpu 0.10.0-pre.3` | Runs on Vulkan / Metal / DirectX via wgpu. `erf` variance forces `erf`-using functionals back to CPU on this backend, per `06-cubecl-strategy.md` §11. |
| C++ xcfun reference | `cc` build-script in `validation/` compiles `xcfun-master/` | Only `validation/` sees the C++ code; no production crate links against it. |
| Python runtime | `pyo3` + `maturin` | `xcfun-py` is a `cdylib`. Built via `maturin build --release` into a wheel. |

### Internal boundaries

| Boundary | Communication | Notes |
|---|---|---|
| `xcfun-ad` ↔ `xcfun-core` | `xcfun-core` imports `CTaylor`, `Num`, `VAR*`, `CNST` | Direct Rust types; no FFI; no trait objects |
| `xcfun-core` ↔ `xcfun-kernels` | `xcfun-kernels` imports `Vars`, `VARS_TABLE`, `FunctionalId`, `DensVars` field order | Data and layout only; no calls into `xcfun-core`'s scalar evaluator from kernel bodies |
| `xcfun-kernels` ↔ `xcfun-gpu` | `xcfun-gpu` calls `eval_batch_kernel::launch_unchecked::<F, R>` | Via cubecl's generated launcher; the only crossing of the host/device boundary |
| `xcfun-core` / `xcfun-gpu` ↔ `xcfun-rs` | `pub use` re-exports | No logic in the façade |
| `xcfun-rs` ↔ `xcfun-capi` | `xcfun-capi::xcfun_t = Functional` via `#[repr(transparent)]` | Opaque pointer on the C side; no duplicated state |
| `xcfun-rs` ↔ `xcfun-py` | PyO3 `#[pyclass]` wraps `Functional` | GIL-governed; numpy arrays zero-copy via `PyArray::as_slice` |
| `validation/` ↔ `xcfun-master/` | `cc` compiles C++ to staticlib, Rust `extern "C"` calls | FFI only in `validation/cpp_shim.rs` |

---

## 11. Missing seams that the design should clarify

From cross-referencing the 2026 Rust numerical-GPU ecosystem:

1. **Shared algebraic spec between CPU helper and `#[cube]` helper (§5.4 above).** Currently an aspiration. Either pick option (a) — kernel source is the only source — or commit to a macro (b). Recommendation: (a). This affects Phase 3's scope and should be decided before Phase 3 begins.

2. **Feature-flag enforcement between `xcfun-core` (std) and `xcfun-kernels` (no_std inside `#[cube]`).** The design implies kernels are no_std-callable, but `xcfun-core` defaults to `std`. Recommended: add `xcfun-core`'s `std` feature as `default`, and make `xcfun-kernels` depend on `xcfun-core` with `default-features = false`. Documented in `01-source-tree.md` §6 but not enforced in Cargo.toml templates.

3. **Kernel dispatch table as a build-script vs hand-written.** The design says `xcfun-kernels::dispatch_table` is "a compile-time-generated big switch" (§3.2 of `06-cubecl-strategy.md`). It is unclear whether this is generated by `xtask codegen` (like the registry) or by a proc-macro. Recommendation: same `xtask codegen` step as the registry; one source of truth for "which functionals exist".

4. **Boundary-enforcement CI check.** §10 of `05-module-responsibilities.md` mentions `cargo xtask check-boundaries`. Scope: (a) no `anyhow` in library crates, (b) no crate imports `xcfun-capi`, (c) `xcfun-kernels` imports `xcfun-core` only with `default-features = false`. Write this test before Phase 6; without it, façade-crate hygiene is aspirational.

5. **`Send` / `Sync` bounds on `Batch<'fun, R>`.** The design doesn't specify whether a `Batch` is movable across threads. `cubecl` `ComputeClient` is typically `Clone + Send + Sync`, but device buffers are often not. Recommendation: document `Batch: Send + !Sync` (a batch is owned by one thread but can move between threads), matching `burn`'s backend convention.

6. **`xcfun-kernels` test-seam under `CpuRuntime`.** `05-module-responsibilities.md` §3 mentions `feature = "cpu-testing"` for kernel unit tests. This needs a concrete Cargo setup: `cubecl-cpu` must be a dev-dep of `xcfun-kernels`, not a regular dep, to avoid pulling the CPU runtime into downstream users who only want CUDA.

---

## 12. Confidence assessment

| Claim | Confidence | Source |
|---|---|---|
| 7-crate layout is idiomatic for 2026 Rust numerical/GPU libraries | HIGH | cubecl, burn, candle, polars, cubek all share the same shape |
| AD belongs in its own crate, not inside the domain core | HIGH | `burn-autodiff`, `sophus_autodiff`, `scirs2-autograd` all do this |
| Kernel crate must be runtime-agnostic (generic over `R: Runtime`) | HIGH | cubecl's own docs state this explicitly; cubek follows the pattern |
| C ABI in a dedicated `-capi` / `-ffi` crate is the standard | HIGH | rustls-ffi, PyO3 docs, Rustonomicon FFI guidance |
| Python bindings in a dedicated `-py` crate is the standard | HIGH | candle-pyo3, polars-python, tokenizers |
| Build order AD → core → kernels → gpu → façade → capi/py is forced by the DAG | HIGH | Direct reading of `Cargo.toml` dependency declarations |
| Shared-spec seam between host helpers and kernel helpers is underspecified | HIGH | Explicit reading of `06-cubecl-strategy.md` §4.1 |
| `xcfun-gpu` as a single crate (not split per backend) is the right call | MEDIUM | Trade-off: `burn` splits (`burn-cuda`, `burn-wgpu`), `candle` merges behind feature flags. The proposal follows candle; both are defensible. Given xcfun's small Batch API surface, merging is simpler. |

---

## 13. Sources

### Authoritative (Context7 / official docs)

- [CubeCL — tracel-ai/cubecl (Context7 `/tracel-ai/cubecl`, 83 snippets, HIGH reputation)](https://github.com/tracel-ai/cubecl)
- [CubeCL Book — burn.dev (Context7 `/websites/burn_dev_books_cubecl_print`, 105 snippets)](https://burn.dev/books/cubecl/print.html)
- [CubeK — tracel-ai/cubek (Context7 `/tracel-ai/cubek`, 61 snippets)](https://github.com/tracel-ai/cubek)
- [Burn — tracel-ai/burn](https://github.com/tracel-ai/burn)
- [Candle — huggingface/candle](https://github.com/huggingface/candle)
- [Polars package structure — DeepWiki](https://deepwiki.com/pola-rs/polars/1.2-package-structure)
- [PyO3 ffi module](https://pyo3.rs/main/doc/pyo3_ffi/index.html)
- [Rustls FFI bindings (reference pattern)](https://github.com/rustls/rustls-ffi)

### Secondary (WebSearch-verified)

- [SciRS2 — cool-japan/scirs (29-crate scientific workspace precedent)](https://github.com/cool-japan/scirs)
- [cubecl-runtime crates.io](https://crates.io/crates/cubecl-runtime)
- [ad-trait paper (arxiv)](https://arxiv.org/html/2504.15976v1)
- [sophus_autodiff docs](https://docs.rs/sophus_autodiff)
- [burn-autodiff crates.io](https://crates.io/crates/burn-autodiff)
- [Large Rust Workspaces — matklad](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)
- [rust-bindgen](https://github.com/rust-lang/rust-bindgen)

### Internal (design documents under `docs/design/`)

- `docs/design/01-source-tree.md`
- `docs/design/02-data-structures.md`
- `docs/design/04-control-flow.md`
- `docs/design/05-module-responsibilities.md`
- `docs/design/06-cubecl-strategy.md`
- `.planning/PROJECT.md`

---

*Architecture research for: Rust reimplementation of xcfun with cubecl-backed unified CPU/GPU evaluation*
*Researched: 2026-04-18*
