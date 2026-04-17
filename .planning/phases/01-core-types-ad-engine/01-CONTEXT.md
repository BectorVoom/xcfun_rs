# Phase 1: Core Types + AD Engine - Context

**Gathered:** 2026-04-17 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the xcfun-core and xcfun-ad crates: all type definitions (DensityVars<T>, EvalMode, VarType, FunctionalId, Dependency, XcError, constants, Functional trait) and the complete automatic differentiation engine (CTaylor<T, N>, Num trait, arithmetic operators, transcendental functions, composition). This is the foundation everything else depends on.

</domain>

<decisions>
## Implementation Decisions

### Workspace Setup
- **D-01:** Set up full Cargo workspace from Phase 1 with xcfun-core and xcfun-ad as active crates. Create stub Cargo.toml files for xcfun-functionals, xcfun-eval, xcfun-gpu, xcfun-ffi, xcfun-python so the workspace structure is established. Only xcfun-core and xcfun-ad get source code in this phase.
- **D-02:** xcfun-ad depends on no other crate in the workspace. xcfun-core depends on xcfun-ad (for the Num trait bound on DensityVars<T>). This matches the design doc dependency graph.

### C++ Reference Data Extraction
- **D-03:** Extract test_in/test_out reference arrays from C++ xcfun source files in xcfun-master/. These become static test data in xcfun-core (test_data module) and xcfun-ad tests.
- **D-04:** For AD engine validation, compute known analytical derivatives (e.g., d^3/dx^3 of exp(x) at x=1.0 = e) and compare against CTaylor output. Do not rely solely on C++ extraction — use mathematical truth as ground truth for the AD engine itself.
- **D-05:** Compile and run the C++ xcfun test suite from xcfun-master/ to generate additional reference data at orders 0-6 for cross-validation in later phases.

### Const Generic Strategy
- **D-06:** CTaylor<T, N> uses truly generic const N (not specialized impls per N). All arithmetic and transcendental functions work for any N at compile time.
- **D-07:** Test at N=0 (energy only), N=1 (first derivatives), N=2 (second derivatives), up to N=7 (maximum). Ensure no panics or numerical instability at boundary values.
- **D-08:** The `taylorlen()` function is a const fn that computes binomial coefficients. It is placed in xcfun-core (not xcfun-ad) because the evaluation pipeline needs it.

### Compose Implementation
- **D-09:** Replicate the C++ compose() implementation exactly, coefficient by coefficient. This is the highest-risk code — numerical equivalence depends on matching the C++ algorithm's rounding behavior.
- **D-10:** The compose function handles Taylor composition (chain rule) for applying scalar functions to CTaylor polynomials. All transcendental functions (exp, log, sqrt, pow, sin, cos, atan, asin, acos, asinh, erf, sqrtx_asinh_sqrtx) use compose internally.
- **D-11:** The sqrtx_asinh_sqrtx function uses a Pade approximant near x=0, matching the C++ implementation exactly for numerical stability.

### Regularization
- **D-12:** Regularization at TINY_DENSITY = 1e-14 is implemented in DensityVars::from_input(). The clamp must only affect the constant term c[0] of CTaylor, preserving all derivative coefficients. This is critical — clamping all coefficients would break every functional's potential output.

### Enum and Type Design
- **D-13:** FunctionalId enum has 78 variants with #[repr(u32)]. Provides from_name() (case-insensitive), name(), description(), and depends() methods. from_name() and name() can be const fn where possible.
- **D-14:** VarType enum has 30 variants (including 2nd-order Taylor input types). Each provides input_len(), provides() -> Dependency, and is_spin_polarized() as const fn methods.
- **D-15:** The Functional trait is defined in xcfun-core but implemented on concrete types in xcfun-functionals (Phase 2+). Phase 1 only defines the trait, not implementations.
- **D-16:** FunctionalImpl enum dispatch (78 variants wrapping concrete types) is NOT part of Phase 1 — it belongs in Phase 2 when the first functionals are implemented.

### Claude's Discretion
- Exact test data format (inline arrays vs external files vs build-script-generated)
- Whether to use a proc macro for FunctionalId name/description metadata or hand-write match arms
- Error message wording in XcError variants
- Whether to derive Debug/Clone/Copy on all types or selectively

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core Types
- `docs/design/01-data-structures.md` — DensityVars<T>, XcFunctional, EvalMode, VarType, FunctionalId, Dependency, XcError, Settings, constants
- `docs/design/02-traits.md` — Functional trait, Num trait, GpuEvaluable trait, enum dispatch pattern, TestData struct

### Automatic Differentiation
- `docs/design/03-autodiff.md` — CTaylor<T, N> design, bit-flag indexing, arithmetic, transcendentals, compose(), memory layout, taylorlen()

### Architecture
- `docs/design/00-overview.md` — Crate decomposition, dependency graph, expected source tree
- `docs/design/10-design-decisions.md` — 9 major design decisions with options/rationale (all locked)

### Error Handling
- `docs/design/07-error-handling.md` — XcError variants, FFI error codes, layered policy

### Testing
- `docs/design/08-testing.md` — Test strategy, accuracy validation, reference data format

### Dependencies
- `docs/design/09-dependencies.md` — Crate selection with rationale, version pins

### C++ Reference
- `xcfun-master/` — C++ xcfun source for extracting test data and verifying algorithmic equivalence

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Cargo.toml` — Existing workspace root with dependencies (anyhow, cubecl, thiserror, tracing) already declared
- `xcfun-master/` — Complete C++ xcfun source for reference extraction

### Established Patterns
- Rust Edition 2024 — use current idioms (let chains, etc.)
- Dependencies already pinned: cubecl =0.10.0-pre.3 (for later phases), thiserror 2.0, anyhow 1.0, tracing 0.1

### Integration Points
- xcfun-core exports all types that downstream crates consume
- xcfun-ad exports CTaylor<T, N> and Num trait
- xcfun-core re-exports Num from xcfun-ad for convenience

</code_context>

<specifics>
## Specific Ideas

- Match C++ xcfun algorithm exactly for numerical equivalence — this is a reimplementation, not a redesign
- All design decisions in docs/design/10-design-decisions.md are locked — do not revisit
- Stack allocation only on the hot path (CTaylor fits in L1 cache up to N=7)
- The recursive multiplication algorithm (O(3^N)) must match C++ bit-for-bit

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-core-types-ad-engine*
*Context gathered: 2026-04-17*
