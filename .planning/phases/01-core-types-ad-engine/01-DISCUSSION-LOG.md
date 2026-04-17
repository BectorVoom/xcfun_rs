# Phase 1: Core Types + AD Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-17
**Phase:** 01-core-types-ad-engine
**Mode:** auto
**Areas discussed:** Workspace Setup, C++ Reference Data, Const Generic Strategy, Compose Implementation

---

## Workspace Setup

| Option | Description | Selected |
|--------|-------------|----------|
| Full workspace skeleton | Set up all 7 crates with stubs, active code in xcfun-core + xcfun-ad only | ✓ |
| Minimal (2 crates only) | Only create xcfun-core and xcfun-ad, add others later | |
| Single crate with modules | Start monolithic, split later | |

**User's choice:** Full workspace skeleton (auto-selected recommended default)
**Notes:** Establishes correct dependency graph from the start; stub crates prevent workspace restructuring later.

---

## C++ Reference Data

| Option | Description | Selected |
|--------|-------------|----------|
| Extract from source + compile | Parse C++ test arrays AND run C++ to generate additional data | ✓ |
| Extract from source only | Parse static arrays from C++ headers | |
| Generate fresh from mathematical truth | Use analytical derivatives only, no C++ data | |

**User's choice:** Extract from source + compile (auto-selected recommended default)
**Notes:** Belt-and-suspenders approach: mathematical truth validates AD engine correctness, C++ data validates numerical equivalence.

---

## Const Generic Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Truly generic with targeted tests | Implement for any const N, test at N=0..7 | ✓ |
| Specialize per N | Separate impls for common N values | |
| Generic with runtime N | Use Vec instead of arrays | |

**User's choice:** Truly generic with targeted tests (auto-selected recommended default)
**Notes:** Rust const generics are powerful enough; specialization adds complexity without benefit.

---

## Compose Implementation

| Option | Description | Selected |
|--------|-------------|----------|
| Match C++ exactly | Coefficient-by-coefficient replication of C++ compose() | ✓ |
| Faa di Bruno formula | Generate coefficients programmatically | |
| Hybrid | Match C++ for low orders, generate for high orders | |

**User's choice:** Match C++ exactly (auto-selected recommended default)
**Notes:** Numerical equivalence is the #1 project constraint. Matching the algorithm exactly simplifies verification.

---

## Claude's Discretion

- Test data format (inline vs external)
- FunctionalId metadata implementation (proc macro vs match arms)
- Error message wording
- Derive attribute choices

## Deferred Ideas

None
