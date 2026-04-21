# `xcfun_rs` — design document set

This directory contains the complete design for `xcfun_rs`, a Rust-from-scratch reimplementation of the xcfun exchange–correlation functional library. The goal is 1e-12 relative-error parity with the C++ reference (`xcfun-master/`) across the full public API, plus a unified CPU/GPU batch evaluation engine built on `cubecl`.

These documents are the specification. A second engineer should be able to implement the library using only:
1. These 13 Markdown files.
2. The C++ reference at `xcfun-master/`.
3. The cubecl manual at `docs/manual/Cubecl/`.

No additional clarification required.

## Reading order

Documents are numbered in the order they should be read. Each relies only on documents with lower numbers.

| # | Document | Topic |
|---|----------|-------|
| [00](00-overview.md) | Overview | Scope, goals, non-goals, success criteria |
| [01](01-source-tree.md) | Source tree | Workspace, crate, module layout |
| [02](02-data-structures.md) | Data structures | `CTaylor`, `DensVars`, `Functional`, memory layout |
| [03](03-api-surface.md) | API surface | Every public API (Rust, C ABI, Python) with signatures, pre/postconditions |
| [04](04-control-flow.md) | Control flow | Mermaid diagrams for setup, evaluation, batch flows |
| [05](05-module-responsibilities.md) | Module responsibilities | Per-crate ownership, test seams, boundary rules |
| [06](06-cubecl-strategy.md) | CubeCL strategy | Unified CPU/GPU kernels, batch API, transfer minimisation |
| [07](07-accuracy-strategy.md) | Accuracy strategy | How 1e-12 parity is guaranteed |
| [08](08-error-model.md) | Error model | `thiserror` in libraries, `anyhow` at boundaries |
| [09](09-testing-strategy.md) | Testing strategy | Four test tiers, fixtures, CI matrix |
| [10](10-build-and-dependencies.md) | Build and dependencies | Workspace Cargo.toml, per-crate deps, features, MSRV |
| [11](11-process-and-milestones.md) | Process and milestones | Phased delivery plan with entry/exit criteria |
| [12](12-design-decisions.md) | Design decisions | Rationale and rejected alternatives |

## Fast paths for specific readers

- **Implementers starting work**: read 00 → 03 → 07 → 11 first. Then 02, 04, 05 as you build.
- **Reviewers**: 00, 03, 07, 12 cover scope, contract, correctness, and rationale.
- **GPU reviewers**: 06 and 09 §4.
- **API consumers (external)**: 03 alone is the interface contract.
- **CI engineers**: 09 and 10.

## Core invariants

1. **Accuracy**: output matches C++ xcfun within 1e-12 relative error for every `(functional, vars, mode, order)` tuple.
2. **Single kernel source**: every per-point evaluator is written once, as a `#[cube]` function, and dispatched to `CpuRuntime`, `CudaRuntime`, or `WgpuRuntime`.
3. **Zero hot-path allocation**: `Functional::eval` and `Batch::eval_vec_host` allocate no heap after construction/`reserve`.
4. **Library errors via `thiserror`**: no `anyhow` in any library crate; enforced by CI.
5. **Full API parity**: every symbol in `xcfun-master/api/xcfun.h` has a Rust counterpart and a C-ABI re-export.

Each invariant has an authoritative chapter (7, 6, 2, 8, 3 respectively) and a CI job guarding it.

## Reference material not in this set

- `xcfun-master/api/xcfun.h` — the C header we must reproduce.
- `xcfun-master/src/**/*.{hpp,cpp}` — the reference implementation to parity-test against.
- `docs/manual/Cubecl/*.md` — cubecl capability and idiom reference.
- `CLAUDE.md` (repo root) — pinned technology stack, versions, and constraints.
