# Phase 1: Taylor Algebra & AD Primitives (`xcfun-ad`) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `01-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 01-taylor-algebra-ad-primitives-xcfun-ad
**Mode:** auto (all gray areas resolved with recommended defaults)
**Areas discussed:** Storage & const-generic bounds, Num trait & numeric backbone, CTaylor::mul recursion, *_expand ports, Composed elementary functions, no_std story, Golden fixture workflow, Property tests & bench, FP hygiene, Testability seams

---

## Storage & const-generic bounds

| Option | Description | Selected |
|--------|-------------|----------|
| `Assert<{N<=7}>: True` (nightly `generic_const_exprs`) | Compile-time bound via const-generic assertion trick | |
| Sealed `ValidN<N>` marker trait, impls for N ∈ 0..=7 | Stable Rust, no nightly features; explicit trait bound | ✓ |
| Runtime `debug_assert!` | Only catches bad N at test time; no compile-time guarantee | |

**Auto-selected:** Sealed trait — stable-Rust compatible, keeps the crate off nightly.
**Notes:** `Assert<{N<=7}>: True` is ergonomically similar but requires `#![feature(generic_const_exprs)]`. Not worth the nightly dependency for a stable library crate.

---

## Num trait & numeric backbone

| Option | Description | Selected |
|--------|-------------|----------|
| Custom `Num` trait | Narrow surface matching what functionals actually need | ✓ |
| `num-traits::Float` blanket | Brings `is_nan`, `classify`, `floor`, etc. — meaningless on polynomials | |
| `simba::scalar::Field` | Overkill, adds algebraic-structure deps | |

**Auto-selected:** Custom `Num`. Locked by design D2 in `docs/design/12-design-decisions.md`.

---

## CTaylor::mul recursion structure

| Option | Description | Selected |
|--------|-------------|----------|
| Verbatim port of `ctaylor_rec::multo` | Line-for-line recursion on highest bit; preserves accumulation order for 1e-12 parity | ✓ |
| Flat triple-loop | Idiomatic Rust but breaks parity at order ≥ 2 (Pitfall P3) | |
| SIMD-aware reassociated form | Faster but catastrophic for parity | |

**Auto-selected:** Verbatim port. Non-negotiable per PITFALLS.md P3 and design D1.

---

## *_expand scalar series ports

| Option | Description | Selected |
|--------|-------------|----------|
| Line-by-line port with upstream line-range comments | Cross-reference target for code review; body = textual port | ✓ |
| Re-expressed via closed-form identities | Avoids the recursion; introduces new rounding ordering — violates algorithmic identity | |
| Generated from a DSL | Too much meta, not enough parity safety | |

**Auto-selected:** Line-by-line port. Matches design D1/12 and PITFALLS.md P9.

---

## Preconditions: `assert!` vs `debug_assert!`

| Option | Description | Selected |
|--------|-------------|----------|
| `assert!` (always-on) | Runs in release, catches silent-NaN on malformed inputs (P10) | ✓ |
| `debug_assert!` (dev-only) | Cheap in release but silent NaN possible — PITFALLS.md P10 | |

**Auto-selected:** `assert!`. Explicit recommendation in PITFALLS.md P10.

---

## no_std story

| Option | Description | Selected |
|--------|-------------|----------|
| Features: `std` (default) + `libm` (alternative backend) | no_std-capable without blocking Phase 1; kernel crate can opt out later | ✓ |
| `std` only | Simpler but locks `xcfun-ad` to std forever | |
| Always `libm` | Tiny ULP differences vs `std::f64` — complicates fixture matching | |

**Auto-selected:** `std` default + `libm` optional. Gets `no_std` on the table without forcing it in Phase 1.

---

## Golden fixture generation

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-generated `.bincode` fixtures + `xtask regen-ad-fixtures` | Hermetic `cargo build`; maintainers regenerate on xcfun-master bumps | ✓ |
| `build.rs` compiles C++ at build time | Breaks hermetic `cargo build`; forces every consumer to have a C++ toolchain | |
| Hand-coded fixtures | Too brittle; drifts from upstream | |
| Property tests only, no fixtures | Doesn't catch ULP-level regressions that fixed-input fixtures expose | |

**Auto-selected:** Pre-generated fixtures. Lock in hermetic builds.

---

## Fixture format

| Option | Description | Selected |
|--------|-------------|----------|
| `bincode` records + JSON manifest | Small, fast to load, JSON debuggable | ✓ |
| JSON-only | Human-readable but bloated (~3–5× size) | |
| `.hex.txt` of `f64::to_bits` | Max debuggability, largest size | |

**Auto-selected:** `bincode` + JSON manifest. Size budget ≤ 1 MB total.

---

## Property tests & bench baseline

| Option | Description | Selected |
|--------|-------------|----------|
| `proptest` 1.11, ≥ 10 000 iters per property; criterion bench baseline (no gate) | Matches REQUIREMENTS.md AD-06 and the stack pin | ✓ |
| `quickcheck` | Smaller ecosystem, less rich shrinking | |
| Manual `#[test]` matrix | Insufficient for ring-axiom coverage | |

**Auto-selected:** `proptest` 1.11 with 10k+ iterations.

---

## Scratch buffer for expansions

| Option | Description | Selected |
|--------|-------------|----------|
| `[T; 8]` stack array (N_MAX + 1 = 8) | Fixed size, stack-only, correct for every N ≤ 7 | ✓ |
| `[T; N+1]` per-call sized | Requires `generic_const_exprs` (nightly) | |
| `Vec<T>` allocation | Violates no-heap rule | |

**Auto-selected:** `[T; 8]`. Matches the `N ≤ 7` invariant.

---

## Testability seam exposure

| Option | Description | Selected |
|--------|-------------|----------|
| `pub mod for_tests` behind `feature = "testing"` | Downstream dev-deps opt-in; no surface in default build | ✓ |
| Public from day one | Surface creep | |
| Private, tests inside crate only | Downstream functional parity tests can't reach low-level helpers | |

**Auto-selected:** `feature = "testing"`.

---

## FP hygiene enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| `-fp-contract=off` in `.cargo/config.toml` (Phase 0) + `cargo asm` CI spot-check of `CTaylor::mul` | Belt-and-suspenders against unsanctioned FMA emission | ✓ |
| Empty `RUSTFLAGS` only | Doesn't prevent LLVM contracting inside optimisation passes | |
| Function-level `#[target_feature(enable = "nofma")]` | Non-portable, not actually a target feature | |

**Auto-selected:** `-fp-contract=off` + `cargo asm` grep.

---

## Claude's Discretion

No areas left fully open — every gray area in the agenda was resolved with a recommended default in auto mode. Planning agents inherit the full decision set and should not re-ask.

## Deferred Ideas

- SIMD vectorisation of `CTaylor::mul` — v2.
- Nested `CTaylor<CTaylor<f64, M>, N>` — no downstream need.
- Pure-polynomial no_std (without libm) — feasible but complex; no consumer.
- Per-op bench regression gate — v2 (PERF-01).

---

*Auto-mode completion: 12 decisions, 0 free-form corrections.*
