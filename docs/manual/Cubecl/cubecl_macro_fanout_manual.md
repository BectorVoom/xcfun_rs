# Manual: Reducing CubeCL Macro Fan-out

## Purpose

This manual describes practical techniques for reducing **macro fan-out** in CubeCL-based scientific computing projects.

In CubeCL, the `#[cube]` procedural macro parses Rust syntax, tracks scope and variables, and generates expansion code that constructs the intermediate representation (IR) used to define GPU/CPU kernels. Because the macro can be applied to **functions, traits, and impl blocks**, and because `#[cube(launch)]` additionally generates launch wrappers, careless usage can significantly increase generated code volume, compile time, and binary size.

This guide focuses on **code-organization patterns** that preserve performance while keeping compilation manageable.

---

## What “macro fan-out” means in CubeCL

In this manual, **macro fan-out** means the amount of generated code and compilation work triggered by one CubeCL macro annotation or derive.

Typical fan-out sources in CubeCL include:

- Applying `#[cube]` to large functions, trait hierarchies, or many impl blocks.
- Applying `#[cube(launch)]` to helpers that do not need a public launch wrapper.
- Deriving `CubeType` / `CubeLaunch` on many internal structs and enums.
- Splitting kernels by concrete numeric type (`f16`, `bf16`, `f32`, etc.) instead of using CubeCL’s generic expansion model.
- Encoding runtime specialization using repeated macro-generated APIs instead of compile-time or launch-time information.

The result is usually one or more of the following:

- slower `cargo build` / `cargo check`
- more proc-macro work
- larger generated code and binaries
- harder-to-debug compile-time failures

---

## Core design principle

**Keep the CubeCL expansion surface as small as possible.**

A good rule of thumb is:

> Only annotate code that truly needs CubeCL expansion, and keep the public launch surface narrower than the internal computation surface.

In practice, that means:

- expose **few launchable kernels**
- use **small internal `#[cube]` helpers**
- avoid broad trait/impl expansion unless it is necessary
- keep type specialization centralized rather than duplicated

---

## CubeCL features that already help reduce fan-out

Before changing architecture, it is important to use CubeCL’s existing mechanisms correctly.

### 1. Use `#[cube]` and `#[cube(launch)]` differently

`#[cube]` is the general expansion macro for CubeCL functions, traits, and implementations.

`#[cube(launch)]` (or `#[cube(launch_unchecked)]`) generates both the expansion function and a **launch wrapper**. Therefore, it should normally be used only on **entry kernels** that must be launched from the host.

### 2. Use `comptime!` and `#[comptime]`

CubeCL provides `comptime!` to mark code as compile-time and to **turn off expansion for that code**, using it verbatim. It also supports compile-time parameters through `#[comptime]`.

Use these for:

- tile sizes
- line sizes
- unroll factors
- constant formulas
- policy decisions

Do **not** use runtime IR construction for values that can stay compile-time only.

### 3. Use `#[define(T)]` to avoid type-specific entry-point explosion

CubeCL supports `#[define(T)]` parameters that map a runtime value to a generic type parameter. This allows a single kernel entry to cover multiple numeric type choices, instead of creating many concrete host-side wrappers.

### 4. Prefer generic numeric kernels over many concrete kernels

CubeCL maps `Float`, `Int`, and `Numeric` generic parameters into generic expansion types such as `FloatExpand`, `IntExpand`, and `NumericExpand`. This reduces compilation from “one expansion per concrete numeric type” toward “one expansion per generic position”.

This is one of the most important compile-time optimizations already present in CubeCL.

---

## Primary strategies to reduce fan-out

## Strategy 1: Restrict `#[cube(launch)]` to true entry kernels

### Why

Launch variants generate more surface than plain `#[cube]`. If helper functions are marked as `launch`, CubeCL will generate extra wrapper machinery that is unnecessary.

### Recommended pattern

- Mark only host-invoked kernels with `#[cube(launch)]` or `#[cube(launch_unchecked)]`.
- Mark internal reusable helper functions with plain `#[cube]`.

### Example

```rust
use cubecl::prelude::*;

#[cube(launch)]
fn saxpy_kernel<F: Float>(
    x: &Array<Line<F>>,
    y: &mut Array<Line<F>>,
    a: F,
) {
    if ABSOLUTE_POS < x.len() {
        y[ABSOLUTE_POS] = saxpy_inner(x[ABSOLUTE_POS], y[ABSOLUTE_POS], a);
    }
}

#[cube]
fn saxpy_inner<F: Float>(x: Line<F>, y: Line<F>, a: F) -> Line<F> {
    a * x + y
}
```

### Anti-pattern

```rust
#[cube(launch)]
fn helper_1<...>(...) { ... }

#[cube(launch)]
fn helper_2<...>(...) { ... }
```

If `helper_1` and `helper_2` are never launched directly from the host, they should not be launch kernels.

---

## Strategy 2: Prefer free `#[cube]` helper functions over broad trait/impl expansion

### Why

CubeCL supports `#[cube]` on traits and impl blocks, but that expansion is broader than free-function expansion.

When `#[cube]` is applied to traits, CubeCL generates expansion traits and related machinery.
When applied to impl blocks, CubeCL generates multiple expansion forms for methods.

This is convenient, but it can increase generated code quickly.

### Recommended pattern

- Keep host-side abstractions in traits if needed.
- Keep CubeCL kernel computation in **small free functions** with `#[cube]`.
- Use traits only where the abstraction pays for itself.

### Better

```rust
#[cube]
fn gelu_map<F: Float>(x: Line<F>) -> Line<F> {
    let sqrt2 = F::new(comptime!(2.0f32.sqrt()));
    let tmp = x / Line::new(sqrt2);
    x * (Line::erf(tmp) + 1.0) / 2.0
}
```

### More expensive pattern

```rust
#[cube]
trait Activation<F: Float> {
    fn map(x: Line<F>) -> Line<F>;
}

#[cube]
impl<F: Float> Activation<F> for Gelu {
    fn map(x: Line<F>) -> Line<F> { ... }
}
```

Use the trait version only when there is a genuine benefit that justifies the extra expansion surface.

---

## Strategy 3: Avoid per-type kernel duplication

### Why

Writing separate kernels for `f16`, `bf16`, `f32`, and `f64` multiplies the macro surface and the host API surface.

CubeCL already has generic expansion reduction for `Float`, `Int`, and `Numeric`. Use it.

### Recommended pattern

```rust
#[cube]
fn reduce_step<F: Float>(x: Line<F>, y: Line<F>) -> Line<F> {
    x + y
}
```

### Avoid

```rust
#[cube]
fn reduce_step_f16(x: Line<f16>, y: Line<f16>) -> Line<f16> { x + y }

#[cube]
fn reduce_step_bf16(x: Line<bf16>, y: Line<bf16>) -> Line<bf16> { x + y }

#[cube]
fn reduce_step_f32(x: Line<f32>, y: Line<f32>) -> Line<f32> { x + y }
```

### Rule

If the algorithm is structurally identical across numeric types, keep **one generic kernel path**.

---

## Strategy 4: Use `#[define(T)]` instead of multiplying public APIs

### Why

If you create separate launch APIs for each numeric type, memory layout, or data configuration, you often duplicate outer wrappers and dispatch code.

`#[define(T)]` allows launch-time information to determine a generic type parameter without generating many public entry points.

### Example

```rust
#[cube(launch)]
fn map_kernel<N: Numeric>(
    input: &Array<N>,
    output: &mut Array<N>,
    #[define(N)] _elem: ElemType,
) {
    if ABSOLUTE_POS < input.len() {
        output[ABSOLUTE_POS] = input[ABSOLUTE_POS];
    }
}
```

### Rule

If multiple host-visible functions differ only by type choice, try to collapse them into **one generic launch entry** with `#[define]`.

---

## Strategy 5: Push policy and constants into `comptime`

### Why

Compile-time-only values do not need full runtime IR expansion.

CubeCL marks compile-time variables differently and can keep them out of runtime expansion.

### Good candidates

- tile shapes
- reduction axis policies
- kernel scheduling flags
- loop unroll factors
- precomputed numeric constants

### Example

```rust
#[cube]
fn affine<F: Float>(x: Line<F>) -> Line<F> {
    let alpha = F::new(comptime!(1.25f32));
    let beta = F::new(comptime!(0.5f32));
    alpha * x + beta
}
```

### Practical rule

If a value is fixed at kernel compilation time and does not need to be represented in runtime IR, move it into `comptime!` or a `#[comptime]` parameter.

---

## Strategy 6: Limit `CubeType` / `CubeLaunch` derive usage to boundary types

### Why

`#[derive(CubeType)]` and `#[derive(CubeLaunch)]` are useful, but they also add generated code.

If they are applied indiscriminately to internal helper structs and enums, expansion surface grows without much benefit.

### Recommended pattern

Use derive macros only for:

- true kernel argument boundary types
- custom IR-visible types that must participate in CubeCL type handling

Avoid deriving them for:

- host-only helper structs
- temporary organizational enums
- internal pipeline state not passed to kernels

### Rule

Treat CubeCL derives like FFI boundary derives: apply them only where there is a real boundary requirement.

---

## Strategy 7: Split large kernels into small reusable `#[cube]` helpers—but keep those helpers function-based

### Why

A single huge `#[cube]` function often creates a large parse/generate surface and becomes hard to reason about.

Splitting a giant kernel into smaller `#[cube]` helpers can reduce complexity and improve reuse.

However, if you split it into many trait-based expansions or many launchable kernels, you only move the problem around.

### Recommended pattern

- one launch entry kernel
- several small free `#[cube]` helpers
- minimal trait/impl expansion

### Example structure

```rust
#[cube(launch)]
fn reduce_kernel<F: Float>(input: &Array<Line<F>>, output: &mut Array<Line<F>>) {
    if ABSOLUTE_POS < input.len() {
        let v = load_and_preprocess(input[ABSOLUTE_POS]);
        output[ABSOLUTE_POS] = reduce_local(v);
    }
}

#[cube]
fn load_and_preprocess<F: Float>(x: Line<F>) -> Line<F> {
    x * x
}

#[cube]
fn reduce_local<F: Float>(x: Line<F>) -> Line<F> {
    x
}
```

### Rule

Break apart **algorithmic stages**, not host-visible kernel entry points.

---

## Strategy 8: Reduce debug-only expansion in normal workflows

CubeCL supports debug symbol generation through `#[cube(debug_symbols)]` and debug-related configuration. This is useful when debugging IR generation or kernel source mapping, but it should not be left enabled all the time in normal development or CI if compile time is a concern.

### Recommended policy

- keep debug-oriented expansion off by default
- enable it only for focused debugging sessions
- use a dedicated debug profile or environment switch

---

## Strategy 9: Prefer compile-time information over heavy macro-based type switching

The Burn 0.20 / CubeCL ecosystem specifically reported that moving away from heavy macro-based patterns for generic numerical types toward **dynamic data types with compile-time information** produced:

- a cleaner codebase
- smaller binaries
- faster compilation times

### Implication for application code

If your current design encodes many type/policy combinations by duplicating CubeCL macro entry points, consider whether those combinations can instead be expressed using:

- generic kernels
- compile-time parameters
- launch-time specialization
- runtime type descriptors mapped with `#[define]`

---

## Refactoring checklist

Use this checklist when reviewing a CubeCL-heavy module.

### Launch surface

- [ ] Is every `#[cube(launch)]` function actually launched from the host?
- [ ] Can any `launch` kernel be downgraded to plain `#[cube]`?

### Trait/impl usage

- [ ] Are `#[cube]` traits used where a free helper function would work?
- [ ] Are impl blocks expanded only where the method-based abstraction is truly necessary?

### Numeric specialization

- [ ] Are there multiple kernels that differ only by `f16` / `bf16` / `f32` / `f64`?
- [ ] Can they become one `F: Float` kernel?
- [ ] Can host-side type selection move to `#[define(T)]`?

### Compile-time values

- [ ] Are fixed constants computed inside runtime-expanded code?
- [ ] Can those values move into `comptime!`?
- [ ] Are policy values represented as runtime IR when they could be compile-time only?

### Derives

- [ ] Is `CubeType` derived only on true CubeCL data boundary types?
- [ ] Is `CubeLaunch` derived only on launch argument boundary types?

### Debug

- [ ] Are debug symbols or debug-heavy expansion enabled by default?
- [ ] Can debugging be moved to an opt-in workflow?

---

## Measurement workflow

Do not optimize fan-out blindly. Measure first.

### 1. Use Cargo timings

```bash
cargo build --timings
```

Open the generated HTML report and identify:

- which kernel crates dominate compile time
- whether proc-macro crates serialize the build
- whether a small number of CubeCL-heavy crates are the critical path

### 2. Use macro statistics

```bash
cargo +nightly rustc -p your_kernel_crate -- -Zmacro-stats
```

This helps determine:

- which macros expand the most code
- whether CubeCL-heavy modules are structurally too large
- whether trait/impl expansion is contributing substantially

### 3. Compare before/after refactors

After each refactor, record:

- clean build time
- incremental build time
- proc-macro build time contribution
- binary size (if relevant)
- developer feedback on readability and maintainability

---

## Cargo-side support configuration

CubeCL projects often benefit from Cargo profile tuning, especially because proc macros and build dependencies can be configured separately.

### Helpful workspace settings

```toml
[profile.dev]
debug = "line-tables-only"
incremental = true

[profile.dev.build-override]
opt-level = 0
codegen-units = 256
debug = false
```

### Notes

- Cargo supports profile overrides for build dependencies, proc macros, and their dependencies.
- This can help keep proc-macro compilation lightweight during development.
- For normal dependency optimization experiments, be careful with generics-heavy crates because optimization location can affect monomorphization behavior.

---

## Recommended architecture pattern for CubeCL scientific kernels

A practical low-fan-out architecture often looks like this:

1. **One public launchable kernel per real algorithm entry point**
2. **A small set of free `#[cube]` helper functions for shared kernel logic**
3. **Generic numeric abstraction (`Float`, `Int`, `Numeric`) instead of concrete-type duplication**
4. **`#[define]` for launch-time specialization**
5. **`comptime!` for constants and policies**
6. **Derive macros only on boundary types**
7. **Optional debug expansion only when needed**

This structure tends to produce:

- lower macro fan-out
- smaller generated launch/API surfaces
- easier compile-time debugging
- better long-term maintainability

---

## Common anti-patterns

### Anti-pattern 1: Every helper is launchable

```rust
#[cube(launch)] fn helper_a(...) { ... }
#[cube(launch)] fn helper_b(...) { ... }
#[cube(launch)] fn helper_c(...) { ... }
```

**Fix:** only the true entry kernel should be launchable.

### Anti-pattern 2: Trait-driven CubeCL everywhere

```rust
#[cube]
trait Op<...> { ... }

#[cube]
impl<...> Op<...> for ... { ... }
```

**Fix:** prefer small free helper functions unless trait expansion clearly pays off.

### Anti-pattern 3: One kernel per numeric type

```rust
kernel_f16(...)
kernel_bf16(...)
kernel_f32(...)
kernel_f64(...)
```

**Fix:** use generic numeric kernels and `#[define]` where necessary.

### Anti-pattern 4: Runtime IR for compile-time policy

```rust
let tile = some_runtime_value_used_only_for_codegen_logic;
```

**Fix:** move compile-time-only decisions into `comptime!` / `#[comptime]`.

### Anti-pattern 5: Derive macros on all the things

```rust
#[derive(CubeType, CubeLaunch)]
struct InternalTemporaryState { ... }
```

**Fix:** derive only on true kernel boundary types.

---

## Decision guide

Use this short decision guide when adding new CubeCL code.

### Should this function be `#[cube(launch)]`?

Use `#[cube(launch)]` only if the function must be launched directly from the host.
Otherwise use `#[cube]`.

### Should this abstraction be a CubeCL trait?

Only if the trait-based abstraction is essential inside kernel code.
If a free function can express the same logic, prefer the free function.

### Should this be split by numeric type?

Only if the algorithm is materially different by type.
If the structure is the same, use a generic numeric kernel.

### Should this value be runtime or compile-time?

If it only affects code generation or specialization and is fixed at kernel compile time, keep it compile-time.

### Should this type derive `CubeType` or `CubeLaunch`?

Only if it is a true CubeCL-visible kernel boundary type.

---

## Final recommendation

For most CubeCL scientific computing codebases, the single highest-leverage rule is:

> **Minimize the number of launchable kernels and keep the rest of the kernel graph as small, generic, function-based `#[cube]` helpers.**

Then layer in:

- `comptime!` for constants/policies
- `#[define]` for launch-time type selection
- generic numeric kernels instead of concrete duplication
- boundary-only derive usage

If you apply those consistently, you can usually cut a large amount of unnecessary procedural macro work without losing runtime performance.

---

## References

- CubeCL procedural macro architecture and generation model
- CubeCL macro API (`cube`, `comptime!`, `intrinsic!`, `CubeType`, `CubeLaunch`)
- Burn 0.20 release notes on reducing heavy macro-based patterns
- Rust Performance Book (`--timings`, `-Zmacro-stats`)
- Cargo profile and build override documentation
