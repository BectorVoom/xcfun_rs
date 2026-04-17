# Phase 2: LDA Functionals + Validation Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-04-18
**Phase:** 02-lda-functionals-validation-pipeline
**Mode:** auto
**Areas discussed:** Functional module structure, FunctionalImpl dispatch bootstrapping, Reference data extraction, Evaluation pipeline scope, Alias table implementation

---

## Functional Module Structure

| Option | Description | Selected |
|--------|-------------|----------|
| One file per functional with shared LDA module | Each LDA functional gets its own file (slaterx.rs, vwn3c.rs, etc.) under lda/ directory | ✓ |
| Single file for all LDA functionals | All 5 LDA implementations in one lda.rs file | |
| Grouped by exchange/correlation | Separate exchange and correlation modules | |

**User's choice:** [auto] One file per functional with shared LDA utilities module (recommended default)
**Notes:** Mirrors C++ source organization. Each functional is small but distinct enough to warrant its own file. Shared helpers (VWN parameterization, PW92 epsilon) get their own files.

---

## FunctionalImpl Dispatch Bootstrapping

| Option | Description | Selected |
|--------|-------------|----------|
| All 78 variants with stubs | Define all variants now, unimplemented!() for non-LDA | ✓ |
| Only 5 LDA variants | Add variants incrementally per phase | |
| Trait object fallback | Use dyn Functional for unimplemented, enum for implemented | |

**User's choice:** [auto] All 78 variants defined, only 5 LDA implemented, rest use unimplemented!() (recommended default)
**Notes:** Keeps FunctionalImpl in sync with FunctionalId from the start. Later phases just replace unimplemented!() with real implementations. No proc macro -- hand-written match arms.

---

## Reference Data Extraction

| Option | Description | Selected |
|--------|-------------|----------|
| Manual extraction from C++ sources | Copy test_in/test_out arrays by hand into Rust static arrays | ✓ |
| Build script automation | Parse C++ files automatically during build | |
| External JSON files | Store reference data in JSON, load at test time | |

**User's choice:** [auto] Manual extraction into static Rust arrays in test_data module (recommended default)
**Notes:** Consistent with Phase 1 decision D-03. Only 5 functionals -- manual extraction is tractable and avoids build complexity.

---

## Evaluation Pipeline Scope

| Option | Description | Selected |
|--------|-------------|----------|
| All three modes for LDA | PartialDerivatives + Potential + Contracted | ✓ |
| PartialDerivatives only | Defer Potential and Contracted to later phases | |
| PartialDerivatives + Potential | Skip Contracted mode | |

**User's choice:** [auto] All three evaluation modes for LDA (recommended default)
**Notes:** LDA potential mode is simpler than GGA (no gradient divergence). Implementing all three modes now validates the full pipeline architecture before GGA adds complexity.

---

## Alias Table Implementation

| Option | Description | Selected |
|--------|-------------|----------|
| Static alias definitions with name-to-pairs mapping | Each alias maps to Vec<(FunctionalId, f64)> pairs, expanded at set() time | ✓ |
| Builder pattern | Fluent API for constructing aliases programmatically | |
| External config file | TOML/JSON alias definitions loaded at runtime | |

**User's choice:** [auto] Static alias definitions expanded at set() time (recommended default)
**Notes:** Matches C++ aliases.cpp approach. LDA aliases are simple (1-2 components each). Non-recursive expansion sufficient for LDA subset.

---

## Claude's Discretion

- File placement of alias table (xcfun-functionals vs xcfun-eval)
- FunctionalImpl::from_id() implementation strategy
- Internal VWN helper structure
- Batch evaluation placement

## Deferred Ideas

None -- all discussion stayed within phase scope.
