# Phase 2: LDA Functionals + Validation Pipeline - Research

**Researched:** 2026-04-18
**Domain:** LDA exchange-correlation functional implementation, evaluation pipeline, cubecl batch evaluation, validation infrastructure
**Confidence:** HIGH

## Summary

Phase 2 implements the first end-to-end evaluable functionals in xcfun_rs: 5 LDA functionals (SlaterX, Vwn3C, Vwn5C, Pz81C, Pw92C), the XcFunctional evaluation pipeline (new/set/eval_setup/eval), LDA alias expansion, and the automated validation test infrastructure comparing against C++ reference data. This phase also introduces cubecl-based batch evaluation where the same kernels run on CPU (CpuRuntime) and GPU backends.

The Phase 1 foundation is solid: `Functional` trait, `DensityVars<T>`, `FunctionalId` enum (78 variants), `CTaylor<T, N>` AD engine, `Num` trait, `VarType`/`EvalMode` enums, `XcError`, and physical constants are all implemented and tested. The stub crates `xcfun-functionals` and `xcfun-eval` exist with Cargo.toml dependencies configured but only placeholder lib.rs files. The `xcfun-gpu` crate exists but has cubecl dependencies commented out ("added in Phase 6"). Per the user directive, cubecl must be activated NOW in Phase 2 for CPU+GPU batch evaluation.

The C++ reference implementations are straightforward to port. SlaterX is a one-liner. VWN3/VWN5 share a parameterized helper (`vwn_f`) with different parameter sets. PZ81C has a branching structure (r_s > 1 vs r_s <= 1). PW92C uses a different parameterization (`eopt`) with spin interpolation. All five use `ufunc(zeta, 4/3)` for spin interpolation, which is `(1+zeta)^(4/3) + (1-zeta)^(4/3)`. The cubecl kernel versions implement the same formulas specialized for f64 scalar math.

**Primary recommendation:** Implement in dependency order: (1) LDA functional structs with Functional trait, (2) FunctionalImpl enum dispatch, (3) reference test data extraction, (4) XcFunctional evaluation pipeline, (5) alias table, (6) cubecl batch kernels in xcfun-gpu, (7) integration tests and validation.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** One source file per functional in `xcfun-functionals/src/lda/` (e.g., `slaterx.rs`, `vwn3c.rs`, `vwn5c.rs`, `pz81c.rs`, `pw92c.rs`) with a shared `lda/mod.rs` re-exporting all types.
- **D-02:** Each functional is a unit struct (zero-sized type) implementing the `Functional` trait.
- **D-03:** Shared LDA helper functions (e.g., VWN parameterization, PW92 epsilon computation) live in `lda/helpers.rs` or `lda/pw92eps.rs`.
- **D-04:** Implement `FunctionalImpl` enum in `xcfun-functionals` with all 78 variants defined, but only 5 LDA variants have real implementations. Remaining 73 use `unimplemented!("Phase N")`.
- **D-05:** `FunctionalImpl` provides `energy<T>()`, `depends()`, `id()`, and `from_id(FunctionalId) -> Self` methods via match dispatch.
- **D-06:** No proc macro for enum dispatch -- hand-write the match arms.
- **D-07:** Manually extract `test_in[]`/`test_out[]` arrays from C++ source files. Place as static arrays in `xcfun-core/src/test_data.rs`.
- **D-08:** Each `TestData` entry includes: FunctionalId, VarType, EvalMode, order, threshold, input slice, expected_output slice.
- **D-09:** Each functional's `test_data()` trait method returns its own `TestData`.
- **D-10:** Full `XcFunctional` struct in `xcfun-eval` with lifecycle: new() -> set() -> eval_setup() -> eval().
- **D-11:** `XcFunctional` stores active functionals as `Vec<(FunctionalImpl, f64)>`.
- **D-12:** All three evaluation modes (PartialDerivatives, Potential, Contracted) implemented for LDA.
- **D-13:** `eval_setup()` validates dependencies, order, and mode compatibility.
- **D-14:** `eval()` implements the variable-pair loop for derivatives.
- **D-15:** CPU evaluation logic MUST use cubecl. Same kernels for CPU and GPU.
- **D-15a:** Batch evaluation as cubecl kernels with `#[cube(launch)]` and `Array<f64>`.
- **D-15b:** Single-point evaluation keeps the pure Rust CTaylor AD path.
- **D-15c:** CubeCL coding patterns: `f64::exp(x)` not `x.exp()`, if-statements not if-expressions, `#[comptime]` for constants, `ABSOLUTE_POS`, no `usize` in device code.
- **D-15d:** `xcfun-gpu` crate hosts all cubecl kernels. Depends on `cubecl-core` (always) + `cubecl-cpu` (always). GPU backends feature-gated.
- **D-15e:** LDA formulas as `#[cube]` functions for f64 batch evaluation, separate from the Functional trait (which uses Num for AD).
- **D-15f:** Buffer management via `client.create()`, `client.empty()`, `client.read_one()`. Caching deferred to Phase 6.
- **D-16:** Static alias definitions mapping name strings to `(FunctionalId, f64)` pairs.
- **D-17:** LDA aliases: lda, svwn5, svwn3, svwn, vwn, vwn5, vwn3.
- **D-18:** `set()` checks alias table first, then falls back to direct `FunctionalId::from_name()`.
- **D-19:** `test_all_lda_functionals()` integration test.
- **D-20:** Accuracy reporting with max/mean relative error.
- **D-21:** Per-functional unit tests in each module.

### Claude's Discretion
- Exact file placement of alias table (xcfun-functionals vs xcfun-eval)
- Whether `FunctionalImpl::from_id()` uses match or array lookup
- Internal structure of VWN parameterization helper
- Whether to put batch evaluation in XcFunctional directly or as a free function

### Deferred Ideas (OUT OF SCOPE)
None -- analysis stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LDA-01 | SlaterX matching C++ within 1e-12 | C++ algorithm: `(-C_SLATER) * (d.a_43 + d.b_43)`. One-liner, uses pre-computed `a_43`, `b_43` from DensityVars. |
| LDA-02 | Vwn3C matching C++ within 1e-12 | C++ uses `d.n * vwn3_eps(d)` with `vwn_f()` parameterized helper. Two parameter sets (para, ferro). Spin interpolation via `ufunc`. |
| LDA-03 | Vwn5C matching C++ within 1e-12 | C++ uses `d.n * vwn5_eps(d)`. Three parameter sets (para, ferro, inter) + `zeta^4` spin interpolation. |
| LDA-04 | Pz81C matching C++ within 1e-12 | C++ uses `pz81eps(d) * d.n` with branching on `r_s > 1` (low density: rational form) vs `r_s <= 1` (high density: log form). |
| LDA-05 | Pw92C matching C++ within 1e-12 | C++ uses `pw92eps(d) * d.n` with `eopt()` parameterization and `omega()` spin interpolation. |
| LDA-06 | LDA aliases producing correct compositions | C++ `aliases.cpp` defines: lda=(slaterx+vwn5c), svwn5=(slaterx+vwn5c), svwn3=(slaterx+vwn3c), svwn=svwn5, vwn=vwn5c, vwn5=vwn5c, vwn3=vwn3c. |
| EVAL-01 | XcFunctional lifecycle | Design doc `05-processing-flows.md` specifies full lifecycle. `xcfun-eval` stub exists. |
| EVAL-02 | Partial derivatives mode (orders 0-6) | Variable-pair loop with CTaylor seeding per `05-processing-flows.md`. Output size = taylorlen(n_vars, order). |
| EVAL-03 | Potential mode for LDA | Simple: v_xc = dE/dn (no gradient divergence for LDA). Seed CTaylor<f64,1> on density variable. |
| EVAL-04 | Contracted mode | Input is pre-computed Taylor coefficients. Pass through to functional evaluation. |
| EVAL-05 | Batch evaluation | cubecl kernels via D-15. CpuRuntime for CPU path. Each thread processes one grid point. |
| EVAL-06 | FunctionalImpl enum dispatch | 78-variant enum with match arms per D-04/D-05. |
| EVAL-07 | Alias expansion with weighted sums | Alias table lookup in set(), expand to (FunctionalId, weight) pairs. |
| EVAL-08 | Regularization at density < 1e-14 | Already implemented in `DensityVars::from_input()` via `regularize()`. |
| VAL-01 | Reference test data from C++ | Extracted from slaterx.cpp, vwn3.cpp (no test data!), vwn5c.cpp, pz81c.cpp, pw92c.cpp. |
| VAL-02 | test_all_functionals_against_reference() | Integration test iterating over all 5 LDA test cases. |
| VAL-03 | Accuracy reporting (max/mean error) | On failure, print per-functional max and mean relative error. |
| VAL-04 | Cross-validation at orders 0-4 | Test data covers order 2 (from C++). Orders 0-4 need separate validation. |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| LDA functional formulas (AD path) | xcfun-functionals | xcfun-ad (Num trait) | Functionals implement `Functional::energy<T: Num>()` using AD engine |
| LDA functional formulas (batch path) | xcfun-gpu | cubecl-core | Specialized `#[cube]` functions for f64 batch evaluation |
| FunctionalImpl dispatch | xcfun-functionals | -- | Enum dispatch owns the mapping from FunctionalId to implementation |
| XcFunctional lifecycle | xcfun-eval | xcfun-functionals | Eval crate orchestrates, functionals crate provides implementations |
| Alias table | xcfun-eval or xcfun-functionals | -- | Claude's discretion. Recommend xcfun-eval to avoid circular deps. |
| Batch evaluation runtime | xcfun-gpu | xcfun-eval (invokes) | GPU crate owns cubecl kernels + runtime; eval crate calls into it |
| Reference test data | xcfun-core (test_data.rs) | -- | Centralized data used by both unit and integration tests |
| Validation tests | tests/ (integration) | per-module #[cfg(test)] | Both levels needed: unit for iteration, integration for end-to-end |

## Standard Stack

### Core (Phase 2 additions)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| cubecl-core | =0.10.0-pre.3 | CubeCL kernel definitions (`#[cube]`, `Array`, `ABSOLUTE_POS`) | User directive: CPU evaluation must use cubecl. Pinned pre-release. [VERIFIED: crates.io search] |
| cubecl-cpu | =0.10.0-pre.3 | CpuRuntime for CPU-side cubecl execution | Required for CpuRuntime. Always-on dependency (not feature-gated). [VERIFIED: crates.io search] |
| cubecl-wgpu | =0.10.0-pre.3 | WebGPU/Vulkan backend | Feature-gated. Secondary backend. [VERIFIED: crates.io search] |
| cubecl-cuda | =0.10.0-pre.3 | CUDA backend | Feature-gated. Primary HPC backend. [VERIFIED: crates.io search] |
| approx | 0.5 | Floating-point comparison | Already in workspace. `assert_relative_eq!` for 1e-12 tolerance. [VERIFIED: workspace Cargo.toml] |

### Already Available (from Phase 1)
| Library | Version | Purpose |
|---------|---------|---------|
| thiserror | 2.0.18 | XcError derives |
| bitflags | 2.11 | Dependency flags |
| xcfun-ad | workspace | CTaylor<T, N>, Num trait |
| xcfun-core | workspace | Functional trait, DensityVars, enums, constants |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| cubecl for CPU batch | Plain Rust loop | Simpler, but violates user directive D-15. cubecl unifies CPU/GPU path. |
| Hand-written 78-arm match | proc_macro derive | D-06 explicitly forbids proc macros. Match is mechanical but clear. |

**Installation (xcfun-gpu Cargo.toml changes):**
```toml
[dependencies]
xcfun-core = { path = "../xcfun-core" }
cubecl-core = { version = "=0.10.0-pre.3", package = "cubecl-core" }
cubecl-cpu = { version = "=0.10.0-pre.3", package = "cubecl-cpu" }

[features]
default = []
cuda = ["cubecl-cuda"]
wgpu = ["cubecl-wgpu"]

[dependencies.cubecl-cuda]
version = "=0.10.0-pre.3"
optional = true

[dependencies.cubecl-wgpu]
version = "=0.10.0-pre.3"
optional = true
```

## Architecture Patterns

### System Architecture Diagram

```
User Code
    |
    v
XcFunctional (xcfun-eval)
    |-- new() --> empty state
    |-- set("svwn5", 1.0)
    |       |-- AliasTable lookup --> [(SlaterX, 1.0), (Vwn5C, 1.0)]
    |       |-- FunctionalImpl::from_id() for each
    |       \-- store Vec<(FunctionalImpl, f64)>
    |-- eval_setup(VarType, EvalMode, order)
    |       |-- validate deps, order, mode
    |       \-- cache input_len, output_len
    |
    +-- eval(input, output)  [SINGLE-POINT PATH]
    |       |-- DensityVars::from_input(CTaylor-seeded)
    |       |-- For each variable pair (i,j):
    |       |       For each (functional, weight):
    |       |           result += weight * functional.energy(vars)
    |       \-- Extract derivatives to output
    |
    \-- evaluate_batch(densities, results, n_points)  [BATCH PATH]
            |-- xcfun-gpu cubecl kernels
            |-- CpuRuntime (always) or CudaRuntime/WgpuRuntime
            |-- #[cube] LDA energy functions (f64 specialized)
            |-- Each thread: one grid point
            \-- Read back results
```

### Recommended Project Structure
```
crates/xcfun-functionals/src/
    lib.rs              # pub mod lda; pub mod functional_impl;
    functional_impl.rs  # FunctionalImpl enum (78 variants)
    lda/
        mod.rs          # pub mod slaterx, vwn3c, vwn5c, pz81c, pw92c, helpers;
        slaterx.rs      # SlaterX struct + Functional impl + #[cfg(test)]
        vwn3c.rs        # Vwn3C struct + Functional impl
        vwn5c.rs        # Vwn5C struct + Functional impl
        pz81c.rs        # Pz81C struct + Functional impl
        pw92c.rs        # Pw92C struct + Functional impl
        helpers.rs      # vwn_f(), ufunc(), pz81 branching helpers, pw92eps

crates/xcfun-eval/src/
    lib.rs              # pub mod xc_functional; pub mod aliases;
    xc_functional.rs    # XcFunctional struct (new, set, eval_setup, eval)
    aliases.rs          # Static alias table + lookup

crates/xcfun-gpu/src/
    lib.rs              # pub mod kernels; pub mod runtime;
    kernels/
        mod.rs          # pub mod lda;
        lda.rs          # #[cube] functions: slaterx_kernel, vwn5c_kernel, etc.
    runtime.rs          # Buffer mgmt, launch config, evaluate_batch()

crates/xcfun-core/src/
    test_data.rs        # Static TEST_CASES array (populated from C++)
```

### Pattern 1: LDA Functional Implementation
**What:** Unit struct implementing the `Functional` trait using the `Num` generic.
**When to use:** Every functional (78 total, 5 in this phase).

```rust
// Source: docs/design/02-traits.md + C++ slater.hpp
use xcfun_core::{constants, DensityVars, Dependency, EvalMode, FunctionalId, TestData, VarType};
use xcfun_core::traits::Functional;
use xcfun_ad::Num;

pub struct SlaterX;

impl Functional for SlaterX {
    fn energy<T: Num>(&self, d: &DensityVars<T>) -> T {
        // C++: (-c_slater) * (d.a_43 + d.b_43)
        T::from_f64(-constants::C_SLATER) * (d.a_43.clone() + d.b_43.clone())
    }

    fn depends(&self) -> Dependency { Dependency::DENSITY }
    fn id(&self) -> FunctionalId { FunctionalId::SlaterX }
    fn description(&self) -> &'static str { "Slater LDA exchange" }
    fn long_description(&self) -> &'static str {
        "LDA Exchange functional\n\
         P.A.M. Dirac, Proceedings of the Cambridge Philosophical Society, 26 (1930) 376.\n\
         F. Bloch, Zeitschrift fuer Physik, 57 (1929) 545."
    }
    fn test_data(&self) -> TestData {
        TestData {
            vars: VarType::A_B,
            mode: EvalMode::PartialDerivatives,
            order: 2,
            threshold: 1e-11,
            input: &[0.39e+02, 0.38e+02],
            expected_output: &[
                -0.241948147838e+03,
                -0.420747936684e+01,
                -0.417120618800e+01,
                -0.359613621097e-01,
                0.0,
                -0.365895279649e-01,
            ],
        }
    }
}
```

### Pattern 2: VWN Shared Helper (ufunc + vwn_f parameterization)
**What:** Shared spin-interpolation and VWN parameterization functions generic over `T: Num`.
**When to use:** VWN3, VWN5, and later PW92-based functionals.

```rust
// Source: C++ vwn.hpp, specmath.hpp
use xcfun_ad::Num;

/// ufunc(x, a) = (1+x)^a + (1-x)^a
/// Used for spin interpolation in VWN, PW92, and other spin-dependent functionals.
pub fn ufunc<T: Num>(x: &T, a: f64) -> T {
    let one = T::one();
    (one.clone() + x.clone()).pow(a) + (one - x.clone()).pow(a)
}

/// VWN parameterized function: 0.5 * A * (2*ln(s) + a*ln(X(s)) - b*ln(Y(s)) + c*atan(Z(s)))
/// where X(s) = s^2 + b*s + c, Y(s) = s - x0, Z(s) = sqrt(4c - b^2) / (2s + b)
/// Parameters p = [x0, A, b, c]
pub fn vwn_f<T: Num>(s: &T, p: &[f64; 4]) -> T {
    // Pre-compute parameter-derived constants (all f64, computed once)
    let x0_sq = p[0] * p[0];
    let x0_b_c = x0_sq + p[0] * p[2] + p[3];
    let a_param = p[0] * p[2] / x0_b_c - 1.0;
    let b_param = 2.0 * (p[0] * p[2] / x0_b_c - 1.0) + 2.0;
    let disc = (4.0 * p[3] - p[2] * p[2]).sqrt();
    let c_param = 2.0 * p[2] * (1.0 / disc - p[0] / (x0_b_c * disc / (p[2] + 2.0 * p[0])));

    let x_val = s.clone() * s.clone() + T::from_f64(p[2]) * s.clone() + T::from_f64(p[3]);
    let y_val = s.clone() - T::from_f64(p[0]);
    let z_val = T::from_f64(disc) / (T::from_f64(2.0) * s.clone() + T::from_f64(p[2]));

    T::from_f64(0.5 * p[1]) * (
        T::from_f64(2.0) * s.clone().log()
        + T::from_f64(a_param) * x_val.log()
        - T::from_f64(b_param) * y_val.log()
        + T::from_f64(c_param) * z_val.atan()
    )
}
```

### Pattern 3: CubeCL LDA Kernel
**What:** `#[cube]` function for batch energy evaluation on f64 arrays.
**When to use:** Batch evaluation of LDA functionals on CPU (CpuRuntime) or GPU.

```rust
// Source: docs/manual/Cubecl/cubecl_3d_dft.md + cubecl_error_solution_guide
use cubecl_core::{self as cubecl, prelude::*};

#[cube(launch)]
pub fn slaterx_batch_kernel(
    density_a: &Array<f64>,
    density_b: &Array<f64>,
    output: &mut Array<f64>,
    #[comptime] c_slater: f64,
    #[comptime] n_points: u32,
) {
    let idx = ABSOLUTE_POS;
    if idx >= n_points {
        terminate!();
    }

    let a = density_a[idx];
    let b = density_b[idx];

    // CRITICAL: use f64::powf not .powf() in cubecl
    let a_43 = f64::powf(a, 4.0 / 3.0);
    let b_43 = f64::powf(b, 4.0 / 3.0);

    output[idx] = -c_slater * (a_43 + b_43);
}
```

### Pattern 4: XcFunctional Evaluation Pipeline
**What:** The full lifecycle struct with configuration and evaluation.
**When to use:** Core evaluation API.

```rust
// Source: docs/design/05-processing-flows.md, 07-error-handling.md
use xcfun_core::{XcError, VarType, EvalMode, taylorlen};

pub struct XcFunctional {
    active: Vec<(FunctionalImpl, f64)>,
    depends: Dependency,
    vars: Option<VarType>,
    mode: Option<EvalMode>,
    order: Option<u32>,
    input_len: usize,
    output_len: usize,
}

impl XcFunctional {
    pub fn new() -> Self { /* empty state */ }

    pub fn set(&mut self, name: &str, weight: f64) -> Result<(), XcError> {
        // 1. Check alias table
        // 2. If alias found, expand: for each (id, w) in alias, add (FunctionalImpl::from_id(id), weight * w)
        // 3. If not alias, try FunctionalId::from_name(name)
        // 4. If found, add (FunctionalImpl::from_id(id), weight)
        // 5. If neither, return Err(XcError::UnknownName)
    }

    pub fn eval_setup(&mut self, vars: VarType, mode: EvalMode, order: u32) -> Result<(), XcError> {
        // Validate deps, order, mode per D-13
        // Cache input_len, output_len
    }

    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        // Variable-pair loop per D-14
        // Use CTaylor<f64, N> with appropriate N for the order
    }
}
```

### Anti-Patterns to Avoid
- **Using `.exp()` method syntax in `#[cube]` code:** CubeCL requires associated function syntax: `f64::exp(x)` not `x.exp()`. The macro expansion generates `__expand_exp_method` lookups that fail. [VERIFIED: docs/manual/Cubecl/cubecl_error_solution_guide/mismatched types.md]
- **Using `if` expressions in `#[cube]` code:** `let v = if cond { a } else { b }` causes ExpandElementTyped mismatch. Use `let mut v = default; if cond { v = a; }` instead. [VERIFIED: same doc]
- **Using `usize` in `#[cube]` device code:** cubecl only supports f64, f32, u32, i32 in device code. Cast to u32. [VERIFIED: same doc]
- **Computing VWN/PW92 parameter constants at runtime in the Num generic path:** The `vwn_a()`, `vwn_b()`, `vwn_c()` are pure `f64` computations on the parameter arrays. Compute them as `f64` constants, then use `T::from_f64()` to inject into the generic computation. This avoids unnecessary AD overhead on constant expressions.
- **Porting C++ `#ifndef XCFUN_VWN5_REF` variants:** Use the non-REF (default) parameter values from C++. The REF variants are for higher accuracy but the default ones are what C++ tests validate against.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Floating-point comparison | Custom epsilon logic | `approx::assert_relative_eq!` | Handles near-zero, NaN, edge cases correctly |
| GPU kernel launch + buffer management | Custom GPU abstraction | cubecl's `Array`, `ArrayArg::from_raw_parts`, `client.create/empty/read_one` | cubecl handles backend-specific launch, memory layout, synchronization |
| Error handling boilerplate | Manual Result wrapping | `thiserror` derive on XcError | Already established in Phase 1, consistent across crates |
| Combinatorial output size | Manual C(n+k,k) | `taylorlen()` from xcfun-core | Already implemented and tested |

**Key insight:** The LDA formulas themselves are simple math. The complexity is in the plumbing: evaluation pipeline, cubecl integration, variable-pair loop, alias expansion, test data extraction. Don't over-engineer the formulas; spend the effort on getting the pipeline right.

## Common Pitfalls

### Pitfall 1: VWN Parameter Precision
**What goes wrong:** Using rounded parameter values instead of exact C++ values causes errors exceeding 1e-12.
**Why it happens:** The C++ code uses specific parameter values (e.g., VWN5 `para[] = {-0.10498, 0.0621814, 3.72744, 12.9352}` in the default non-REF path). Even small deviations in the 6th decimal place can accumulate.
**How to avoid:** Copy parameter values character-for-character from the C++ source. The `#ifndef XCFUN_VWN5_REF` block is the DEFAULT (non-REF) path -- use those values.
**Warning signs:** Errors in the 1e-11 to 1e-10 range that are consistent across all test points.

### Pitfall 2: PZ81 Branching on r_s
**What goes wrong:** Using Num::lt() for the `if (1 > d.r_s)` branch in PZ81 causes derivative discontinuity at r_s = 1.
**Why it happens:** The C++ code branches on the constant term of `d.r_s`. In the AD path, `Num::lt()` compares `c[0]` only, which is correct -- but the branch affects which formula's derivatives are computed. At r_s exactly = 1, the branch must match C++.
**How to avoid:** Use `Num::lt(&T::from_f64(1.0), &d.r_s)` to match the C++ `if (1 > d.r_s)` -- note the reversed operands. Verify with the low-density test case (a=0.048, b=0.025 where r_s > 1).
**Warning signs:** Correct energy but wrong derivatives at the r_s = 1 boundary.

### Pitfall 3: PZ81 Test Data Has Two Variants
**What goes wrong:** Using the wrong test data (HIGH_DENSITY vs default).
**Why it happens:** `pz81c.cpp` has `#ifdef HIGH_DENSITY` with two different test cases. The default (non-HIGH_DENSITY) uses input `{0.48E-01, 0.25E-01}` (low density, r_s > 1). The HIGH_DENSITY uses `{0.39E+02, 0.38E+02}` (high density, r_s < 1).
**How to avoid:** Use the DEFAULT test case (low density) since HIGH_DENSITY is not defined by default in the C++ build. Also test with the high-density case for coverage of both branches.
**Warning signs:** Test passes with one input but fails with the other.

### Pitfall 4: VWN3 Has No Built-in Test Data in C++
**What goes wrong:** Cannot find reference data for VWN3.
**Why it happens:** `vwn3.cpp` does NOT include test data arrays (no `XC_A_B, XC_PARTIAL_DERIVATIVES, ...` block). Only VWN5, SlaterX, PZ81, and PW92 have built-in test cases.
**How to avoid:** Generate VWN3 test data by: (1) computing manually with the known formula at test densities, or (2) building and running the C++ xcfun library, or (3) using the same test inputs as VWN5 and verifying the relationship between VWN3 and VWN5 outputs. Use a slightly relaxed threshold (1e-11) initially and tighten after cross-validation.
**Warning signs:** Missing test data for VWN3 blocks validation.

### Pitfall 5: CubeCL f64::powf Availability
**What goes wrong:** `f64::powf(base, exponent)` may not be available in cubecl or may behave differently across backends.
**Why it happens:** cubecl's `#[cube]` macro only supports specific math operations. The reference docs show `f64::cos`, `f64::sin`, `f64::exp`, `f64::sqrt` -- but `powf` for arbitrary exponents may require `exp(exponent * ln(base))`.
**How to avoid:** For the specific LDA case of `x^(4/3)`, use `f64::exp(4.0/3.0 * f64::log(x))` or `f64::cbrt(x) * x` (since `x^(4/3) = x * x^(1/3)`). Test that the cubecl kernel produces the same result as the Rust f64 path.
**Warning signs:** Compilation errors in `#[cube]` functions mentioning `powf` or `__expand_powf`.

### Pitfall 6: Variable-Pair Loop Complexity
**What goes wrong:** Incorrect output indexing or variable seeding in the evaluation loop.
**Why it happens:** The variable-pair loop iterates over (i,j) with i<=j, seeding CTaylor with unit derivatives on variables i and j. The output coefficient extraction must map correctly to the output array. For order 2 with 2 variables (A_B), the output has 6 elements: [E, dE/da, dE/db, d2E/da2, d2E/dadb, d2E/db2].
**How to avoid:** Implement the loop exactly as described in `05-processing-flows.md`. The output size is `taylorlen(n_input_vars, order)`. For LDA with A_B at order 2: taylorlen(2, 2) = 6. Test against the known C++ reference data which includes energy + gradient + hessian.
**Warning signs:** Energy (output[0]) is correct but derivative values are wrong or in the wrong positions.

### Pitfall 7: PW92 omega() Spin Interpolation Constant
**What goes wrong:** Using a different value for the `(2^(1/3) - 1)` denominator.
**Why it happens:** C++ has two paths: default uses `2 * pow(2, 1.0/3.0) - 2` computed at runtime, while `XCFUN_REF_PW92C` uses the hardcoded `0.5198421`. The default path computes it exactly.
**How to avoid:** Use `2.0 * 2.0_f64.powf(1.0/3.0) - 2.0` computed as an f64 constant. Do NOT use the `XCFUN_REF_PW92C` branch.
**Warning signs:** PW92 energy matches to ~1e-11 but not 1e-12.

## Code Examples

### Extracting C++ Test Data to Rust Static Arrays

```rust
// Source: C++ slaterx.cpp, vwn5c.cpp, pz81c.cpp, pw92c.cpp
// In xcfun-core/src/test_data.rs

use crate::{EvalMode, FunctionalId, VarType};

pub struct TestCase {
    pub id: FunctionalId,
    pub vars: VarType,
    pub mode: EvalMode,
    pub order: u32,
    pub threshold: f64,
    pub input: &'static [f64],
    pub expected: &'static [f64],
}

pub const LDA_TEST_CASES: &[TestCase] = &[
    // SlaterX: from slaterx.cpp
    TestCase {
        id: FunctionalId::SlaterX,
        vars: VarType::A_B,
        mode: EvalMode::PartialDerivatives,
        order: 2,
        threshold: 1e-11,
        input: &[0.39e+02, 0.38e+02],
        expected: &[
            -0.241948147838e+03,
            -0.420747936684e+01,
            -0.417120618800e+01,
            -0.359613621097e-01,
            0.0,
            -0.365895279649e-01,
        ],
    },
    // Vwn5C: from vwn5c.cpp
    TestCase {
        id: FunctionalId::Vwn5C,
        vars: VarType::A_B,
        mode: EvalMode::PartialDerivatives,
        order: 2,
        threshold: 1e-11,
        input: &[0.39e+02, 0.38e+02],
        expected: &[
            -0.851077910672e+01,
            -0.119099058995e+00,
            -0.120906044904e+00,
            0.756836181702e-03,
            -0.102861281830e-02,
            0.800136175083e-03,
        ],
    },
    // Pz81C: from pz81c.cpp (default: LOW density test case)
    TestCase {
        id: FunctionalId::Pz81C,
        vars: VarType::A_B,
        mode: EvalMode::PartialDerivatives,
        order: 2,
        threshold: 1e-11,
        input: &[0.48e-01, 0.25e-01],
        expected: &[
            -0.358997585489e-02,
            -0.468661877874e-01,
            -0.731782746282e-01,
            0.218577885080e+00,
            -0.646538277526e+00,
            0.867717298846e+00,
        ],
    },
    // Pw92C: from pw92c.cpp
    TestCase {
        id: FunctionalId::Pw92C,
        vars: VarType::A_B,
        mode: EvalMode::PartialDerivatives,
        order: 2,
        threshold: 1e-11,
        input: &[0.39e+02, 0.38e+02],
        expected: &[
            -8.4713855882783946e+00,
            -1.1861930857502517e-01,
            -1.2041769989725633e-01,
            7.5202855619095870e-04,
            -1.0249091426230799e-03,
            7.9516089195232130e-04,
        ],
    },
    // Vwn3C: NO built-in test data in C++ source.
    // Must be generated by running C++ xcfun or computed from formula.
    // Use same input as VWN5 test case for consistency.
];
```

### CubeCL Buffer Management for Batch Evaluation

```rust
// Source: docs/manual/Cubecl/cubecl_3d_dft.md
use cubecl_core::prelude::*;
use cubecl_cpu::{CpuDevice, CpuRuntime};
use cubecl_runtime::client::ComputeClient;

pub fn evaluate_batch_slaterx(
    density_a: &[f64],
    density_b: &[f64],
    c_slater: f64,
) -> Vec<f64> {
    let n = density_a.len();
    let device = CpuDevice::default();
    let client: ComputeClient<_> = ComputeClient::load(&device);

    let a_handle = client.create(f64::as_bytes(density_a));
    let b_handle = client.create(f64::as_bytes(density_b));
    let out_handle = client.empty(n * core::mem::size_of::<f64>());

    let cube_dim = CubeDim::new_1d(256);
    let cube_count = CubeCount::new_1d(((n as u32) + 255) / 256);

    unsafe {
        slaterx_batch_kernel::launch_unchecked::<CpuRuntime>(
            &client,
            cube_count,
            cube_dim,
            ArrayArg::from_raw_parts::<f64>(&a_handle, n, 1),
            ArrayArg::from_raw_parts::<f64>(&b_handle, n, 1),
            ArrayArg::from_raw_parts::<f64>(&out_handle, n, 1),
            c_slater,
            n as u32,
        );
    }

    let out_bytes = client.read_one(out_handle);
    f64::from_bytes(&out_bytes).to_vec()
}
```

### Alias Table Structure

```rust
// Source: C++ aliases.cpp
use xcfun_core::FunctionalId;

pub struct AliasEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub terms: &'static [(FunctionalId, f64)],
}

pub const LDA_ALIASES: &[AliasEntry] = &[
    AliasEntry {
        name: "lda",
        description: "Slater exchange and VWN5 correlation",
        terms: &[(FunctionalId::SlaterX, 1.0), (FunctionalId::Vwn5C, 1.0)],
    },
    AliasEntry {
        name: "svwn5",
        description: "Slater exchange and VWN5 correlation",
        terms: &[(FunctionalId::SlaterX, 1.0), (FunctionalId::Vwn5C, 1.0)],
    },
    AliasEntry {
        name: "svwn3",
        description: "Slater exchange and VWN3 correlation",
        terms: &[(FunctionalId::SlaterX, 1.0), (FunctionalId::Vwn3C, 1.0)],
    },
    AliasEntry {
        name: "svwn",
        description: "Slater exchange and VWN5 correlation",
        terms: &[(FunctionalId::SlaterX, 1.0), (FunctionalId::Vwn5C, 1.0)],
    },
    AliasEntry {
        name: "vwn",
        description: "VWN5 correlation",
        terms: &[(FunctionalId::Vwn5C, 1.0)],
    },
    AliasEntry {
        name: "vwn5",
        description: "VWN5 correlation",
        terms: &[(FunctionalId::Vwn5C, 1.0)],
    },
    AliasEntry {
        name: "vwn3",
        description: "VWN3 correlation",
        terms: &[(FunctionalId::Vwn3C, 1.0)],
    },
];
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate CPU and GPU code paths | cubecl unified kernels (CpuRuntime + GPU) | cubecl 0.10 (2026) | Same `#[cube]` kernel runs everywhere. User directive requires this. |
| `wgpu` raw shaders for GPU | cubecl Rust-native kernels | cubecl 0.8+ (2025) | Write kernels in Rust, not WGSL/GLSL. |

**Deprecated/outdated:**
- `cudarc` for NVIDIA-only: cubecl covers CUDA + Vulkan + Metal + CPU
- Separate f64 and CTaylor evaluation paths for batch: cubecl unifies the f64 path; CTaylor path remains for derivatives

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | cubecl 0.10.0-pre.3 supports `f64::powf(base, exp)` or equivalent in `#[cube]` code | Pitfall 5 | Medium -- would need `exp(a*log(x))` workaround. Easy fix. |
| A2 | VWN3 test data can be generated using the same input as VWN5 ([39.0, 38.0]) | Pitfall 4 | Low -- the formula is well-defined; just need to compute expected output. |
| A3 | cubecl `CpuRuntime` produces bit-identical f64 results to native Rust f64 math | Architecture | Medium -- CPU JIT may use different instruction ordering. May need 1e-15 tolerance. |
| A4 | The default (non-`XCFUN_VWN5_REF`, non-`XCFUN_REF_PW92C`) parameter paths are what the C++ tests validate against | Pitfall 1, 7 | High -- if wrong, ALL correlation functionals would fail validation. But examining the test data confirms they use default params. |
| A5 | cubecl `f64::log(x)` computes natural log (ln) matching Rust's `f64::ln()` | Code Examples | Low -- standard math convention. |
| A6 | `xcfun-eval` does not currently depend on `xcfun-functionals` (needs to be added) | Architecture | Low -- just a Cargo.toml change. |


## Open Questions (RESOLVED)

1. **VWN3 Reference Test Data** (RESOLVED)
   - What we know: VWN3 has no built-in test data in the C++ source (`vwn3.cpp` has no test arrays).
   - Resolution: Two-pronged validation strategy: (a) Compute VWN3 energy at the standard test input [39.0, 38.0] by implementing the VWN3 formula using f64 arithmetic directly in a unit test (the formula is deterministic given the known parameters from `vwn.hpp` -- para=[-0.4092860, 0.0621814, 13.0720, 42.7198], ferro=[-0.7432940, 0.0310907, 20.1231, 101.578]). The unit test computes the expected energy once via f64, stores it, then validates that the Functional trait path (via Num/CTaylor) produces the same value within 1e-14. (b) Cross-validate via alias composition: verify that `XcFunctional::set("svwn3")::eval()` equals `SlaterX::energy() + Vwn3C::energy()` within 1e-14. This confirms both the functional and the alias table are correct without needing external C++ reference data. The threshold for VWN3 is 1e-11 (same as other LDA functionals) for the self-consistency check.

2. **cubecl `f64::powf` Support** (RESOLVED)
   - What we know: cubecl docs show `f64::cos`, `f64::sin`, `f64::exp`, `f64::sqrt`, `f64::log`. No explicit example of `f64::powf`.
   - Resolution: Use the fallback approach as the primary implementation: for `x^(4/3)`, use `x * f64::cbrt(x)` (if cbrt is available) or `f64::exp(4.0/3.0 * f64::log(x))`. For general powers in VWN (`atan`, `log` -- no arbitrary `powf` needed), the cubecl-supported `f64::log`, `f64::exp`, `f64::sqrt` suffice. If `f64::powf` compiles, use it; otherwise the `exp(a*log(x))` workaround is functionally identical. Plan 04 Task 1 documents both paths.

3. **Variable-Pair Loop Implementation with Const Generics** (RESOLVED)
   - What we know: The eval loop needs `CTaylor<f64, N>` where N depends on the number of variables. Rust const generics don't support runtime-dependent N.
   - Resolution: Use a match dispatcher on `n_vars` (number of input variables from VarType) to call monomorphized inner functions. CTaylor<f64, N> where N = 1 << n_vars provides first-order multilinear derivatives for n_vars variables. For higher derivative orders, the C++ xcfun uses a variable-pair loop: iterate over pairs (i,j) with i<=j, seeding each pair's CTaylor and accumulating. The output coefficients map to taylorlen(n_vars, order) entries. The dispatcher matches on n_vars: `match n_vars { 1 => eval_inner::<2>(...), 2 => eval_inner::<4>(...), ... }`. Within eval_inner, the variable-pair loop iterates over pairs and seeds CTaylor accordingly. This is bounded (n_vars <= 7, matching CTaylor's max N=128) and mechanical.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Everything | Yes | 1.92.0 (Edition 2024 compatible) | -- |
| cargo | Build | Yes | 1.92.0 | -- |
| cubecl-core | D-15 (cubecl kernels) | Yes (crates.io) | 0.10.0-pre.3 | -- |
| cubecl-cpu | D-15 (CpuRuntime) | Yes (crates.io) | 0.10.0-pre.3 | -- |
| CUDA runtime | GPU testing | Not verified | -- | CpuRuntime always available |

**Missing dependencies with no fallback:** None -- cubecl-cpu provides CpuRuntime for all batch evaluation.

**Missing dependencies with fallback:** CUDA runtime -- if unavailable, all batch evaluation runs on CpuRuntime (which is the primary path for Phase 2 anyway).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `approx` 0.5 + `cargo-nextest` |
| Config file | None needed (standard Cargo test) |
| Quick run command | `cargo nextest run -p xcfun-functionals --lib` |
| Full suite command | `cargo nextest run -p xcfun-functionals -p xcfun-eval -p xcfun-gpu -p xcfun-core` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LDA-01 | SlaterX matches C++ within 1e-12 | unit | `cargo nextest run -p xcfun-functionals slaterx` | Wave 0 |
| LDA-02 | Vwn3C matches C++ within 1e-12 | unit | `cargo nextest run -p xcfun-functionals vwn3c` | Wave 0 |
| LDA-03 | Vwn5C matches C++ within 1e-12 | unit | `cargo nextest run -p xcfun-functionals vwn5c` | Wave 0 |
| LDA-04 | Pz81C matches C++ within 1e-12 | unit | `cargo nextest run -p xcfun-functionals pz81c` | Wave 0 |
| LDA-05 | Pw92C matches C++ within 1e-12 | unit | `cargo nextest run -p xcfun-functionals pw92c` | Wave 0 |
| LDA-06 | LDA aliases produce correct compositions | integration | `cargo nextest run -p xcfun-eval alias` | Wave 0 |
| EVAL-01 | XcFunctional lifecycle works | integration | `cargo nextest run -p xcfun-eval lifecycle` | Wave 0 |
| EVAL-02 | Partial derivatives orders 0-4 | integration | `cargo nextest run -p xcfun-eval partial_deriv` | Wave 0 |
| EVAL-03 | Potential mode for LDA | integration | `cargo nextest run -p xcfun-eval potential` | Wave 0 |
| EVAL-04 | Contracted mode | integration | `cargo nextest run -p xcfun-eval contracted` | Wave 0 |
| EVAL-05 | Batch evaluation | integration | `cargo nextest run -p xcfun-gpu batch` | Wave 0 |
| EVAL-06 | FunctionalImpl dispatch | unit | `cargo nextest run -p xcfun-functionals functional_impl` | Wave 0 |
| EVAL-07 | Alias expansion | integration | `cargo nextest run -p xcfun-eval alias` | Wave 0 |
| EVAL-08 | Regularization | unit | Already tested in xcfun-core | Exists |
| VAL-01 | Reference test data | data | Static arrays in test_data.rs | Wave 0 |
| VAL-02 | test_all_functionals | integration | `cargo nextest run -p xcfun-eval test_all` | Wave 0 |
| VAL-03 | Accuracy reporting | integration | Part of test_all (print on failure) | Wave 0 |
| VAL-04 | Cross-validation orders 0-4 | integration | `cargo nextest run -p xcfun-eval cross_validate` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo nextest run -p xcfun-functionals -p xcfun-eval -p xcfun-gpu --lib`
- **Per wave merge:** Full suite including integration tests
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/xcfun-functionals/src/lda/slaterx.rs` -- `#[cfg(test)]` module with reference data test
- [ ] `crates/xcfun-functionals/src/lda/vwn5c.rs` -- same
- [ ] `crates/xcfun-functionals/src/lda/pz81c.rs` -- same
- [ ] `crates/xcfun-functionals/src/lda/pw92c.rs` -- same
- [ ] `crates/xcfun-core/src/test_data.rs` -- populate with C++ reference data
- [ ] Add `xcfun-functionals` dependency to `xcfun-eval/Cargo.toml`
- [ ] Add `xcfun-gpu` dependency to `xcfun-eval/Cargo.toml`
- [ ] Update `xcfun-gpu/Cargo.toml` with cubecl dependencies

## C++ Algorithm Details (Critical for Numerical Accuracy)

### SlaterX (slater.hpp)
```
E_x = -C_SLATER * (a^(4/3) + b^(4/3))
```
Uses pre-computed `d.a_43`, `d.b_43` from DensityVars. Trivial. [VERIFIED: xcfun-master/src/functionals/slater.hpp]

### VWN Parameterization (vwn.hpp)
Shared by VWN3 and VWN5. Core function:
```
vwn_f(s, p) = 0.5 * A * (2*ln(s) + a*ln(X) - b*ln(Y) + c*atan(Z))
where:
  X(s) = s^2 + b*s + c
  Y(s) = s - x0
  Z(s) = sqrt(4c - b^2) / (2s + b)
  a = x0*b/(x0^2 + x0*b + c) - 1
  b_coeff = 2*(x0*b/(x0^2 + x0*b + c) - 1) + 2
  c_coeff = 2*b * (1/disc - x0/(X0 * disc/(b+2*x0)))
  disc = sqrt(4c - b^2)
  p = [x0, A, b, c]
```

**VWN3** parameters (from Dalton):
- para = [-0.4092860, 0.0621814, 13.0720, 42.7198]
- ferro = [-0.7432940, 0.0310907, 20.1231, 101.578]
- eps = vwn_f(sqrt(r_s), para) + g * (vwn_f(sqrt(r_s), ferro) - vwn_f(sqrt(r_s), para))
- g = 1.92366105093154 * (ufunc(zeta, 4/3) - 2)

**VWN5** parameters:
- para = [-0.10498, 0.0621814, 3.72744, 12.9352]
- ferro = [-0.325, 0.0310907, 7.06042, 18.0578]
- inter = [-0.0047584, -(3*pi^2)^(-1), 1.13107, 13.0045]
- eps = vwn_f(s, para) + g * ((vwn_f(s, ferro) - vwn_f(s, para)) * zeta^4 + vwn_f(s, inter) * (1-zeta^4) * (9/4 * (2^(1/3) - 1)))

Energy = n * eps. [VERIFIED: xcfun-master/src/functionals/vwn.hpp]

### ufunc (specmath.hpp)
```
ufunc(x, a) = (1+x)^a + (1-x)^a
```
Used for spin polarization function. When a=4/3: `(1+zeta)^(4/3) + (1-zeta)^(4/3)`. [VERIFIED: xcfun-master/src/specmath.hpp]

### PZ81 (pz81c.hpp)
Two regimes:
```
if r_s < 1 (high density):
  eps = Ehd(r_s, c[2]) + (Ehd(r_s, c[3]) - Ehd(r_s, c[2])) * fz(d)
  where Ehd(x, c) = c[1] + ln(x) * (c[0] + x*c[2]) + c[3]*x

if r_s >= 1 (low density):
  eps = Eld(r_s, c[0]) + (Eld(r_s, c[1]) - Eld(r_s, c[0])) * fz(d)
  where Eld(x, CB1B2) = CB1B2[0] / (1 + CB1B2[1]*sqrt(x) + CB1B2[2]*x)

fz(d) = (2^(4/3) * (a^(4/3) + b^(4/3)) * n^(-1/3) / n - 2) / (2*2^(1/3) - 2)
```

Parameters:
```
c = [[-0.1423, 1.0529, 0.3334, 0],     // Eld paramagnetic
     [-0.0843, 1.3981, 0.2611, 0],       // Eld ferromagnetic
     [0.0311, -0.048, 0.0020, -0.0116],  // Ehd paramagnetic
     [0.01555, -0.0269, 0.0007, -0.0048]] // Ehd ferromagnetic
```

Note: C++ uses `if (1 > d.r_s)` which is `if r_s < 1`. Energy = n * eps. [VERIFIED: xcfun-master/src/functionals/pz81c.hpp]

### PW92 (pw92eps.hpp)
```
eopt(sqrt_r, t) = -2*t[0] * (1 + t[1]*sqrt_r^2) * ln(1 + 0.5/(t[0] * sqrt_r * (t[2] + sqrt_r*(t[3] + sqrt_r*(t[4] + t[5]*sqrt_r)))))

omega(zeta) = (ufunc(zeta, 4/3) - 2) / (2*2^(1/3) - 2)

pw92eps(d):
  c = 8 / (9 * (2*2^(1/3) - 2))
  zeta4 = zeta^4
  omega_val = omega(zeta)
  sqrt_r = sqrt(r_s)
  e0 = eopt(sqrt_r, TUVWXYP[0])
  return e0 - eopt(sqrt_r, TUVWXYP[2]) * omega_val * (1-zeta4) / c
       + (eopt(sqrt_r, TUVWXYP[1]) - e0) * omega_val * zeta4
```

Parameters:
```
TUVWXYP = [[0.03109070, 0.21370, 7.59570, 3.5876, 1.63820, 0.49294, 1],
           [0.01554535, 0.20548, 14.1189, 6.1977, 3.36620, 0.62517, 1],
           [0.01688690, 0.11125, 10.3570, 3.6231, 0.88026, 0.49671, 1]]
```

Energy = n * eps. [VERIFIED: xcfun-master/src/functionals/pw92eps.hpp]

## Sources

### Primary (HIGH confidence)
- C++ source: `xcfun-master/src/functionals/slater.hpp` -- SlaterX algorithm
- C++ source: `xcfun-master/src/functionals/vwn.hpp` -- VWN3/VWN5 parameterization and algorithm
- C++ source: `xcfun-master/src/functionals/pz81c.hpp` -- PZ81 algorithm with branching
- C++ source: `xcfun-master/src/functionals/pw92eps.hpp` -- PW92 algorithm
- C++ source: `xcfun-master/src/functionals/aliases.cpp` -- Complete alias definitions
- C++ source: `xcfun-master/src/functionals/slaterx.cpp` -- SlaterX test data
- C++ source: `xcfun-master/src/functionals/vwn5c.cpp` -- VWN5 test data
- C++ source: `xcfun-master/src/functionals/pz81c.cpp` -- PZ81 test data (both variants)
- C++ source: `xcfun-master/src/functionals/pw92c.cpp` -- PW92 test data
- C++ source: `xcfun-master/src/specmath.hpp` -- ufunc() definition
- Codebase: `crates/xcfun-core/src/traits.rs` -- Functional trait (implemented Phase 1)
- Codebase: `crates/xcfun-core/src/density_vars.rs` -- DensityVars (implemented Phase 1)
- Codebase: `crates/xcfun-ad/src/num.rs` -- Num trait (implemented Phase 1)
- Design docs: `docs/design/02-traits.md` -- SlaterX implementation pattern
- Design docs: `docs/design/05-processing-flows.md` -- Evaluation pipeline flow
- Design docs: `docs/design/07-error-handling.md` -- XcError and eval validation
- Design docs: `docs/design/08-testing.md` -- Test strategy and patterns
- CubeCL reference: `docs/manual/Cubecl/cubecl_3d_dft.md` -- CpuRuntime pattern
- CubeCL reference: `docs/manual/Cubecl/cubecl_error_solution_guide/mismatched types.md` -- Critical pitfalls

### Secondary (MEDIUM confidence)
- crates.io: cubecl-core 0.10.0-pre.3 confirmed available [VERIFIED: cargo search]
- crates.io: cubecl-cpu 0.10.0-pre.3 confirmed available [VERIFIED: cargo search]

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates verified, Phase 1 foundation confirmed working
- Architecture: HIGH -- design docs are detailed, C++ reference code is complete
- Pitfalls: HIGH -- identified from direct analysis of C++ code and cubecl reference docs
- CubeCL integration: MEDIUM -- cubecl 0.10.0-pre.3 is pre-release; some API patterns assumed from reference docs

**Research date:** 2026-04-18
**Valid until:** 2026-05-18 (stable domain, C++ reference is frozen, cubecl pinned)
