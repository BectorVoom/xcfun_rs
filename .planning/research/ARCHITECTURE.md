# Architecture Patterns

**Domain:** Exchange-correlation functional library (DFT scientific computing)
**Researched:** 2026-04-17

## Recommended Architecture

The architecture follows a 5-layer, 7-crate workspace design that mirrors the natural separation of concerns in XC functional evaluation. This is well-established in the existing design docs and aligns with how successful Rust scientific computing libraries (SciRS2 with 29 crates, Burn/cubecl ecosystem) structure multi-concern domains.

```
+-----------------------------------------------------------+
|  PUBLIC API LAYER                                          |
|  xcfun-ffi (C ABI cdylib)  |  xcfun-python (PyO3 cdylib) |
+-----------------------------------------------------------+
          |                           |
          v                           v
+-----------------------------------------------------------+
|  EVALUATION PIPELINE                                       |
|  xcfun-eval                                                |
|  - XcFunctional object (builder + state machine)           |
|  - Mode dispatch (partial derivatives, potential,          |
|    contracted)                                             |
|  - Variable pairing and derivative extraction              |
|  - Batch evaluation with chunking + parallelism            |
+-----------------------------------------------------------+
          |                           |
          v                           v
+---------------------------+  +----------------------------+
|  FUNCTIONAL LAYER         |  |  ACCELERATION LAYER        |
|  xcfun-functionals        |  |  xcfun-gpu                 |
|  - 78 energy functions    |  |  - cubecl kernels          |
|  - Enum dispatch          |  |  - Buffer caching          |
|  - Alias table            |  |  - AoS<->SoA transpose    |
|  - Helper modules         |  |  - CPU/GPU fallback        |
+---------------------------+  +----------------------------+
          |         |                   |        |
          v         v                   v        v
+---------------------------+  +----------------------------+
|  CORE TYPES               |  |  AD ENGINE                 |
|  xcfun-core               |  |  xcfun-ad                  |
|  - DensityVars<T>         |  |  - CTaylor<T, N>           |
|  - EvalMode, VarType      |  |  - Num trait               |
|  - FunctionalId enum      |  |  - Transcendental funcs    |
|  - Dependency bitflags    |  |  - Composition (chain rule)|
|  - Functional trait       |  |  - taylorlen()             |
|  - XcError, constants     |  |                            |
+---------------------------+  +----------------------------+
```

### Component Boundaries

| Component | Responsibility | Depends On | Depended On By |
|-----------|---------------|------------|----------------|
| `xcfun-ad` | Taylor-expansion automatic differentiation. Generic math library with no domain knowledge. Provides `CTaylor<T, N>`, `Num` trait, all transcendental functions. | None (leaf crate) | xcfun-core, xcfun-functionals, xcfun-gpu |
| `xcfun-core` | Domain types and trait definitions. `DensityVars<T>`, enums, the `Functional` trait, error types, physical constants. | xcfun-ad (for `Num` bound in trait) | xcfun-functionals, xcfun-eval, xcfun-gpu, xcfun-ffi, xcfun-python |
| `xcfun-functionals` | All 78 XC energy function implementations. Unit structs implementing `Functional` trait. `FunctionalImpl` enum dispatch. Alias table. | xcfun-core, xcfun-ad | xcfun-eval |
| `xcfun-eval` | Evaluation pipeline orchestration. `XcFunctional` user-facing object (new/set/eval_setup/eval). Mode dispatch, derivative extraction, batch evaluation with parallelism. | xcfun-functionals, xcfun-gpu (optional) | xcfun-ffi, xcfun-python |
| `xcfun-gpu` | GPU batch evaluation via cubecl. Kernel definitions, buffer caching, AoS/SoA transposition, fallback logic. Feature-gated. | xcfun-core, xcfun-ad | xcfun-eval (optional) |
| `xcfun-ffi` | C ABI matching `api/xcfun.h`. Thin wrapper translating C calls to `xcfun-eval` API. | xcfun-eval, xcfun-core | External C consumers |
| `xcfun-python` | PyO3 Python bindings. NumPy array I/O for batch evaluation. | xcfun-eval, xcfun-core | External Python consumers |

### Data Flow

**Single-point evaluation (the hot path):**

```
User input: &[f64] (5-11 values depending on VarType)
    |
    v
DensityVars::from_input() -- parse raw input into named fields,
    compute derived quantities (n, s, zeta, r_s, a_43, b_43)
    |
    v
For each variable pair (i, j) where i <= j:
    |
    Create CTaylor<f64, 2> seeded with VAR0=x_i, VAR1=x_j
    |
    Rebuild DensityVars<CTaylor<f64, 2>> from seeded input
    |
    For each (functional_id, weight) in active list:
        |
        FunctionalImpl::energy(vars) -> CTaylor<f64, 2>   [enum dispatch]
        |
        accumulate += weight * result
    |
    Extract output[i,j] = accumulated.get(VAR0|VAR1)
    |
    v
User output: &mut [f64] (taylorlen(n_vars, order) values)
```

**Key data flow properties:**
- All intermediate values are stack-allocated (CTaylor is `[f64; 1 << N]` fixed array)
- Zero heap allocations in the per-point hot loop
- Variable pairing means CTaylor<_, 2> (4 coefficients, 32 bytes) handles most cases
- Composition is linear: weighted sum of independent functional evaluations
- Derivatives propagate automatically through CTaylor arithmetic

**Batch evaluation data flow:**

```
User: density[n_points * input_len], result[n_points * output_len]
    |
    v
Decision: n_points < threshold OR order > 2 OR no GPU?
    |
    +-- CPU path: parallel_chunks(n_points, CHUNK_SIZE=1024)
    |       |
    |       For each point in chunk: single-point evaluation
    |       (L2-cache-friendly: 1024 * ~200B = 200KB per chunk)
    |
    +-- GPU path (xcfun-gpu):
            |
            AoS -> SoA transpose on host
            |
            H2D upload to GPU buffer (reused from cache)
            |
            Single kernel launch: transform + eval + sum + write
            |
            D2H download results
            |
            SoA -> AoS transpose on host
```

## Patterns to Follow

### Pattern 1: Generic-over-Num for Write-Once Functionals

**What:** Every functional is written once using `T: Num` and works for both `f64` (energy-only) and `CTaylor<f64, N>` (with derivatives).

**When:** Always -- this is the core architectural invariant.

**Why:** Eliminates code duplication. The same `energy()` implementation computes energy when `T=f64` and all derivatives up to order N when `T=CTaylor<f64, N>`.

```rust
// Write once:
fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
    let rs = d.r_s.clone();
    let log_rs = rs.clone().log();
    // ... same code works for f64 AND CTaylor
}
```

### Pattern 2: Enum Dispatch Instead of Trait Objects

**What:** Use a `FunctionalImpl` enum with 78 variants rather than `Box<dyn Functional>`.

**When:** For the evaluation hot path where `energy<T>()` must be called generically.

**Why:** `energy<T>()` is generic -- it cannot be called through a trait object (not object-safe). Enum dispatch enables compile-time monomorphization while keeping a single collection type.

```rust
pub enum FunctionalImpl {
    SlaterX(SlaterX),
    PbeX(PbeX),
    // ... 78 variants
}

impl FunctionalImpl {
    pub fn energy<T: Num>(&self, vars: &DensityVars<T>) -> T {
        match self {
            Self::SlaterX(f) => f.energy(vars),
            Self::PbeX(f) => f.energy(vars),
            // ...
        }
    }
}
```

**Implication:** A derive macro or build script should generate the 78-arm match blocks to avoid hand-maintenance.

### Pattern 3: Const Generic Stack Allocation

**What:** `CTaylor<T, N>` uses `[T; 1 << N]` -- a compile-time-fixed array on the stack.

**When:** All AD operations. No heap allocation in the hot path.

**Why:** Stack allocation is faster, cache-friendly, and avoids allocator contention in parallel batch evaluation. The maximum size (512 bytes at N=6) fits comfortably in any stack frame.

### Pattern 4: Feature-Gated Heavy Dependencies

**What:** `xcfun-gpu` is isolated in its own crate behind a feature gate. Users who don't need GPU never compile `cubecl`.

**When:** Any dependency that significantly increases compile time or has platform-specific requirements.

```toml
# In xcfun-eval/Cargo.toml
[dependencies]
xcfun-gpu = { path = "../xcfun-gpu", optional = true }

[features]
gpu = ["xcfun-gpu"]
```

### Pattern 5: AoS at Boundary, Internal Layout Freedom

**What:** Public API accepts AoS (matching C API), but internal batch paths can use SoA for SIMD/GPU.

**When:** Any time the internal representation benefits from a different layout than the API contract.

**Why:** C API compatibility requires AoS with stride (pitch). GPU coalesced access requires SoA. Transposition cost is negligible vs evaluation cost.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Trait Object Hot Path

**What:** Using `Box<dyn Functional>` or `&dyn Functional` for evaluation dispatch.

**Why bad:** `energy<T>()` is generic over `T: Num`, making it non-object-safe. Attempting to work around this (e.g., separate `energy_f64` and `energy_ctaylor` methods) duplicates the API surface and loses the write-once property.

**Instead:** Enum dispatch with `FunctionalImpl`.

### Anti-Pattern 2: Heap-Allocated Taylor Coefficients

**What:** Using `Vec<f64>` inside CTaylor instead of fixed arrays.

**Why bad:** Heap allocation per grid point in a loop processing millions of points creates allocator pressure and cache misses. The coefficient count is known at compile time.

**Instead:** `[T; 1 << N]` with const generics.

### Anti-Pattern 3: One Kernel Per Functional

**What:** Launching separate GPU kernels for each functional in a composite (e.g., 5 launches for B3LYP).

**Why bad:** Kernel launch overhead (~5-50us) per functional, plus intermediate buffer allocation between kernels. For simple LDA functionals that evaluate in ~100ns, the overhead dominates.

**Instead:** Single fused kernel that evaluates all active functionals and sums weighted contributions in one pass.

### Anti-Pattern 4: Fat LTO for Full Workspace

**What:** Using `lto = "fat"` across all 7 crates.

**Why bad:** 78 functionals x up to 7 derivative orders = 546 monomorphized function instantiations. Fat LTO link time becomes extremely long.

**Instead:** Thin LTO for workspace, `codegen-units = 1` per crate, `opt-level = 3` for xcfun-ad.

### Anti-Pattern 5: Premature SIMD

**What:** Writing explicit SIMD intrinsics before profiling.

**Why bad:** LLVM auto-vectorization with `target-cpu=native` and `-O3` handles most CTaylor array operations. Manual SIMD introduces platform-specific code and maintenance burden.

**Instead:** Profile first. Explicit SIMD only for batch evaluation as a Phase 8 optimization.

## Build Order (Dependency-Driven)

The crate dependency graph dictates a strict build order:

```
Phase 1 (Foundation):
  xcfun-ad      [leaf crate, no domain deps]
  xcfun-core    [depends on xcfun-ad for Num trait bound]

Phase 2 (First Functionals + Pipeline):
  xcfun-functionals  [LDA subset: 5 functionals]
  xcfun-eval         [full pipeline: partial/potential/contracted modes]

Phase 3-5 (Expand Functionals):
  xcfun-functionals  [GGA -> meta-GGA -> hybrid, additive]

Phase 6 (GPU, parallel to Phase 3-5):
  xcfun-gpu     [depends on xcfun-core + xcfun-ad only]

Phase 7 (Bindings, after all functionals):
  xcfun-ffi     [depends on xcfun-eval]
  xcfun-python  [depends on xcfun-eval]
```

**Critical path:** xcfun-ad -> xcfun-core -> xcfun-functionals -> xcfun-eval -> xcfun-ffi/xcfun-python

**Parallel opportunities:**
- xcfun-gpu can begin after Phase 1 (needs only core + ad), though meaningful testing requires Phase 2-3 functionals
- xcfun-ffi and xcfun-python can be built in parallel once xcfun-eval stabilizes
- Functional families (GGA, meta-GGA, hybrid) within xcfun-functionals are independent of each other

**Why xcfun-ad must come first:**
- It is the leaf of the dependency tree -- everything depends on `Num` and `CTaylor`
- It has zero domain knowledge, making it testable in isolation
- Getting AD right is the highest-risk item: if Taylor derivatives are wrong, every functional produces wrong results
- ~800 lines of core code, well-defined correctness criteria (derivative tables)

**Why xcfun-eval must come in Phase 2 (not later):**
- The evaluation pipeline is the validation infrastructure -- it converts raw inputs to DensityVars, runs functionals, extracts derivatives
- Without the pipeline, functional implementations can only be tested via manual CTaylor construction
- Building the pipeline with 5 simple LDA functionals is much easier than building it with 78
- Every subsequent phase reuses the pipeline unchanged, just adding functional variants to the enum

## Scalability Considerations

| Concern | At 5 functionals | At 78 functionals | At hypothetical 200+ |
|---------|-------------------|--------------------|-----------------------|
| Enum dispatch | 5-arm match, trivial | 78-arm match, needs code gen | Derive macro essential |
| Compile time | Fast | xcfun-functionals is large; thin LTO + parallel codegen help | Split into sub-crates (lda, gga, mgga) |
| Binary size | Minimal | All 78 monomorphized at each N used; ~2-5MB release | LTO dead-code elimination helps |
| GPU kernels | 1-2 test kernels | All functionals in one fused kernel | Kernel specialization per functional family |
| Test suite | Manual reference data | 78 * (multi-order, multi-vartype) test cases | Property-based testing + generated reference data |

## Workspace Cargo.toml Structure

```toml
[workspace]
members = [
    "crates/xcfun-ad",
    "crates/xcfun-core",
    "crates/xcfun-functionals",
    "crates/xcfun-eval",
    "crates/xcfun-gpu",
    "crates/xcfun-ffi",
    "crates/xcfun-python",
]
resolver = "2"

[workspace.dependencies]
# Shared dependency versions
thiserror = "2.0"
anyhow = "1.0"
tracing = "0.1"
bitflags = "2.0"
cubecl = { version = "=0.10.0-pre.3", optional = true }

# Internal crates
xcfun-ad = { path = "crates/xcfun-ad" }
xcfun-core = { path = "crates/xcfun-core" }
xcfun-functionals = { path = "crates/xcfun-functionals" }
xcfun-eval = { path = "crates/xcfun-eval" }
xcfun-gpu = { path = "crates/xcfun-gpu" }

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
```

## Sources

- Project design documents: `docs/design/00-overview.md` through `docs/design/11-milestones.md` (HIGH confidence -- project-specific, authoritative)
- SciRS2 workspace architecture with 29 crates as community validation of fine-grained Rust scientific workspace pattern (MEDIUM confidence -- analogous project, different domain)
- CubeCL documentation via Context7: kernel definition patterns using `#[cube]` macro, `Runtime` trait, `Tensor` types (HIGH confidence -- current docs)
- C++ xcfun source at `xcfun-master/src/` for 1:1 mapping verification (HIGH confidence -- reference implementation)
