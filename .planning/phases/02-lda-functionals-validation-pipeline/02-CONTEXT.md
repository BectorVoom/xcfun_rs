# Phase 2: LDA Functionals + Validation Pipeline - Context

**Gathered:** 2026-04-18 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the 5 LDA exchange-correlation functionals (SlaterX, Vwn3C, Vwn5C, Pz81C, Pw92C), the evaluation pipeline (XcFunctional lifecycle: new/set/eval_setup/eval), LDA alias expansion, the FunctionalImpl enum dispatch mechanism, and the automated validation test infrastructure comparing Rust output against C++ reference data. This phase delivers the first end-to-end evaluable functionals with verified accuracy.

</domain>

<decisions>
## Implementation Decisions

### Functional Module Structure
- **D-01:** One source file per functional in `xcfun-functionals/src/lda/` (e.g., `slaterx.rs`, `vwn3c.rs`, `vwn5c.rs`, `pz81c.rs`, `pw92c.rs`) with a shared `lda/mod.rs` re-exporting all types.
- **D-02:** Each functional is a unit struct (zero-sized type) implementing the `Functional` trait, following the pattern in `docs/design/02-traits.md`.
- **D-03:** Shared LDA helper functions (e.g., VWN parameterization, PW92 epsilon computation from `pw92eps.hpp`) live in `lda/helpers.rs` or `lda/pw92eps.rs` mirroring the C++ helper structure.

### FunctionalImpl Enum Dispatch
- **D-04:** Implement `FunctionalImpl` enum in `xcfun-functionals` with all 78 variants defined (matching `FunctionalId`), but only the 5 LDA variants have real implementations. Remaining 73 variants use `unimplemented!("Phase N")` in their `energy()` match arms.
- **D-05:** The `FunctionalImpl` enum provides `energy<T>()`, `depends()`, `id()`, and `from_id(FunctionalId) -> Self` methods via match dispatch. A `from_id()` constructor maps any `FunctionalId` to its `FunctionalImpl` variant.
- **D-06:** No proc macro for the enum dispatch — hand-write the match arms. The enum is stable and mechanical; a macro adds build complexity without proportional benefit at this scale.

### Reference Data Extraction
- **D-07:** Manually extract `test_in[]` / `test_out[]` arrays from the 5 LDA C++ source files (`slaterx.cpp`, `vwn3.cpp`, `vwn5c.cpp`, `pz81c.cpp`, `pw92c.cpp`) in `xcfun-master/src/functionals/`. Place as static arrays in `xcfun-core/src/test_data.rs`.
- **D-08:** Each `TestData` entry includes: `FunctionalId`, `VarType`, `EvalMode`, `order`, `threshold` (1e-11 to 1e-12), `input` slice, and `expected_output` slice. This matches the `TestData` struct already in `xcfun-core/src/traits.rs`.
- **D-09:** Additionally, each functional's `test_data()` trait method returns its own `TestData` for inline testing (as shown in the design doc example for SlaterX).

### Evaluation Pipeline
- **D-10:** Implement the full `XcFunctional` struct in `xcfun-eval` with the lifecycle: `new()` -> `set(name, weight)` -> `eval_setup(vars, mode, order)` -> `eval(input, output)`.
- **D-11:** `XcFunctional` stores active functionals as `Vec<(FunctionalImpl, f64)>` (functional + weight pairs). The `set()` method handles both direct functional names and alias expansion.
- **D-12:** All three evaluation modes (PartialDerivatives, Potential, Contracted) are implemented for LDA in this phase. LDA potential mode is straightforward (v_xc = dE/dn, no gradient divergence), making it a good first implementation of all three modes.
- **D-13:** `eval_setup()` validates that the chosen `VarType` provides the required `Dependency` flags, the order is <= MAX_ORDER, and the mode is compatible with the active functionals' dependencies.
- **D-14:** `eval()` implements the variable-pair loop from `docs/design/05-processing-flows.md`: for each variable pair (i,j) with i<=j, seed a CTaylor, evaluate all active functionals, accumulate weighted results, extract output coefficients.
- **D-15:** Batch evaluation (`evaluate_batch`) processes multiple density points sequentially in Phase 2. Parallel chunking (via `std::thread::scope`) is deferred to Phase 8 optimization unless trivially achievable.

### Alias Table
- **D-16:** Static alias definitions in `xcfun-functionals/src/aliases.rs` (or `xcfun-eval` if the eval crate owns composition). Each alias maps a name string to a list of `(FunctionalId, f64)` pairs plus optional parameters (e.g., `exx`).
- **D-17:** LDA aliases to implement: `lda` (SlaterX), `svwn5` (SlaterX + Vwn5C), `svwn3` (SlaterX + Vwn3C), `svwn` (= svwn5), `vwn` (= vwn5), `vwn5` (Vwn5C alone), `vwn3` (Vwn3C alone). Match C++ `aliases.cpp` definitions exactly.
- **D-18:** The `set()` method first checks the alias table, then falls back to direct `FunctionalId::from_name()` lookup. Alias expansion is non-recursive for LDA aliases (no alias references another alias).

### Validation Infrastructure
- **D-19:** A `test_all_lda_functionals()` integration test iterates over all 5 LDA test cases, evaluates each, and compares against C++ reference data using `approx::assert_relative_eq!` with the per-functional threshold.
- **D-20:** Accuracy reporting: on test failure, print max and mean relative error per functional alongside the specific failing output index. This supports diagnosing systematic vs. isolated errors.
- **D-21:** Individual per-functional unit tests in each functional's module (`#[cfg(test)]` in `slaterx.rs`, etc.) for faster iteration during development.

### Claude's Discretion
- Exact file placement of alias table (xcfun-functionals vs xcfun-eval) — whichever minimizes circular dependencies
- Whether `FunctionalImpl::from_id()` uses a match or an array lookup
- Internal structure of the VWN parameterization helper (single function vs struct with methods)
- Whether to put batch evaluation in XcFunctional directly or as a free function

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### LDA Functional Implementations
- `docs/design/02-traits.md` -- Functional trait definition, TestData struct, FunctionalImpl enum dispatch pattern, SlaterX example implementation
- `docs/design/01-data-structures.md` -- DensityVars<T> fields, XcFunctional struct, AliasEntry, Settings

### Evaluation Pipeline
- `docs/design/05-processing-flows.md` -- Complete evaluation flow (new->set->eval_setup->eval), variable-pair loop, potential mode, contracted mode, batch evaluation, alias composition
- `docs/design/07-error-handling.md` -- XcError variants, validation in eval_setup

### Testing
- `docs/design/08-testing.md` -- Reference data format (TestCase struct), per-functional tests, parametrized test_all, accuracy reporting, alias composition tests

### C++ Reference Sources
- `xcfun-master/src/functionals/slaterx.cpp` -- SlaterX implementation + test data
- `xcfun-master/src/functionals/vwn3.cpp` -- VWN3 correlation + test data
- `xcfun-master/src/functionals/vwn5c.cpp` -- VWN5 correlation + test data
- `xcfun-master/src/functionals/pz81c.cpp` -- Perdew-Zunger 1981 correlation + test data
- `xcfun-master/src/functionals/pw92c.cpp` -- Perdew-Wang 1992 correlation + test data
- `xcfun-master/src/functionals/pw92eps.hpp` -- PW92 epsilon helper (shared by PW92C and other functionals)
- `xcfun-master/src/functionals/vwn.hpp` -- VWN parameterization helper
- `xcfun-master/src/functionals/slater.hpp` -- Slater exchange helper
- `xcfun-master/src/functionals/pz81c.hpp` -- PZ81 helper
- `xcfun-master/src/functionals/aliases.cpp` -- Alias definitions (LDA subset: lda, svwn, svwn5, svwn3, vwn, vwn5, vwn3)
- `xcfun-master/src/functionals/list_of_functionals.hpp` -- Complete functional registry and enum mapping

### Architecture
- `docs/design/00-overview.md` -- Crate decomposition, dependency graph
- `docs/design/10-design-decisions.md` -- Locked design decisions

### Phase 1 Foundation
- `crates/xcfun-core/src/traits.rs` -- Functional trait, Dependency bitflags, TestData struct (already implemented)
- `crates/xcfun-core/src/density_vars.rs` -- DensityVars<T> with from_input() and regularization
- `crates/xcfun-core/src/functional_id.rs` -- FunctionalId enum (78 variants)
- `crates/xcfun-core/src/enums.rs` -- VarType, EvalMode enums
- `crates/xcfun-core/src/error.rs` -- XcError enum
- `crates/xcfun-core/src/constants.rs` -- C_SLATER, CF, TINY_DENSITY, MAX_ORDER
- `crates/xcfun-ad/src/lib.rs` -- CTaylor<T, N>, Num trait

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `xcfun-core::traits::Functional` trait -- ready for implementation on LDA structs
- `xcfun-core::traits::Dependency` bitflags -- used in depends() returns
- `xcfun-core::traits::TestData` struct -- ready for reference data population
- `xcfun-core::density_vars::DensityVars<T>` -- complete with from_input() and regularization
- `xcfun-core::functional_id::FunctionalId` -- 78-variant enum with from_name(), name(), depends()
- `xcfun-core::enums::{EvalMode, VarType}` -- evaluation mode and variable type enums
- `xcfun-core::error::XcError` -- error types for validation failures
- `xcfun-core::constants` -- C_SLATER, CF, TINY_DENSITY, MAX_ORDER
- `xcfun-ad::CTaylor<T, N>` -- automatic differentiation engine with all transcendentals
- `xcfun-ad::Num` trait -- unified interface for f64 and CTaylor

### Established Patterns
- Unit struct + Functional trait impl pattern (shown in design doc for SlaterX)
- Const generic `N` for derivative order throughout the AD engine
- Regularization via `set_constant()` on CTaylor (only modifies c[0])
- `from_input()` handles all 30 VarType variants for density variable construction

### Integration Points
- `xcfun-functionals` depends on `xcfun-core` (for Functional trait, types) and `xcfun-ad` (for Num trait bound)
- `xcfun-eval` depends on `xcfun-core` and `xcfun-functionals` (for FunctionalImpl dispatch)
- Test data lives in `xcfun-core/src/test_data.rs` (currently a stub awaiting population)
- Stub crates (`xcfun-eval`, `xcfun-functionals`) have Cargo.toml but only placeholder lib.rs

</code_context>

<specifics>
## Specific Ideas

- Match C++ implementations exactly for numerical equivalence -- port algorithm, not just mathematics
- The VWN parameterization (`vwn.hpp`) is shared between VWN3 and VWN5 -- extract as a shared helper
- PW92 epsilon calculation (`pw92eps.hpp`) is shared by PW92C and will be reused by GGA functionals in Phase 3 -- design the helper for reuse
- PZ81 correlation has separate expressions for r_s > 1 and r_s <= 1 (paramagnetic/ferromagnetic) -- match C++ branching exactly

</specifics>

<deferred>
## Deferred Ideas

None -- analysis stayed within phase scope

</deferred>

---

*Phase: 02-lda-functionals-validation-pipeline*
*Context gathered: 2026-04-18*
