# Phase 1: Taylor Algebra & AD Primitives (`xcfun-ad`) - Research

**Researched:** 2026-04-19
**Domain:** Numerical C++ -> Rust port of xcfun's bit-flag-indexed multilinear Taylor polynomial AD engine with 1e-12 parity contract
**Confidence:** HIGH (C++ source-of-truth, version pins, and existing Rust scaffolding all directly inspected)

---

## Summary

Phase 1 ports `xcfun-master/external/upstream/taylor/{ctaylor.hpp, ctaylor_math.hpp, tmath.hpp}` into a zero-runtime-dependency Rust crate (`xcfun-ad`) whose arithmetic is algorithmically identical to the C++ reference — not merely algebraically equivalent. The load-bearing invariant is that `CTaylor::mul` accumulates coefficients in exactly the order dictated by the C++ `ctaylor_rec<T, Nvar>::multo` recursion, so every `f64::to_bits` on orders 0..=3 matches a reference driver linked against `xcfun-master`. Beyond the algebra, eight `*_expand` scalar series functions (`inv`, `exp`, `log`, `pow`, `sqrt`, `cbrt`, `gauss`, `erf`, plus secondary `atan`, `asinh`, `sin`, `cos`, `asin`, `acos`) and the `ctaylor_rec::compose` helper form the composed-elementary-function layer (`reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`).

The main architectural decisions are locked in CONTEXT.md (22 items). Research contributes: (1) a file/module decomposition mapping every C++ line range to a Rust file with a concrete dependency order that drives wave structure; (2) the exact `*_expand` recurrence shapes, preconditions, and downstream consumers; (3) a golden-fixture tooling architecture (C++ driver in `xtask regen-ad-fixtures` emitting `bincode` + `fixtures.json` manifest); (4) property-test design with `proptest 1.11` at >= 10k iterations per property; (5) a concrete ULP budget per operation consistent with `docs/design/07-accuracy-strategy.md §6`; (6) the `ValidN<N>` sealed trait shape that avoids nightly const-generic features; (7) a preliminary mapping of existing `crates/xcfun-ad/src/*.rs` files (2912 LOC) to the CONTEXT.md-locked API — the existing code diverges in both `N` semantics (treats `N` as array size, not nvar count) and heap allocation in transcendentals (`vec![0.0; nvar+1]`), requiring targeted rewrites.

**Primary recommendation:** Start Wave 0 by aligning the existing `CTaylor<T, const N: usize>` to the CONTEXT.md-locked layout (`N = nvar`, storage `[T; 1 << N]`, `ValidN<N>` sealed trait), then port `ctaylor_rec` verbatim, then port `*_expand` one-per-module with golden fixtures driving correctness before any composed elementary function is written.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Storage & const-generic bounds**
- **D-01:** `CTaylor<T, const N: usize>` stores coefficients in a single `pub c: [T; 1 << N]`, `#[repr(C)]`. Stack only, no heap, no `Box`, no `Vec`. Valid `N` range is `0..=7`.
- **D-02:** `N <= 7` enforced via a sealed `ValidN<N>` marker trait implemented only for `N` in `0..=7` — stable-Rust compatible, no `generic_const_exprs` nightly feature required.
- **D-03:** `Copy + Clone + Debug + PartialEq` derived. No `Hash`, no `Eq` (f64 is not Eq).

**`Num` trait & numeric backbone**
- **D-04:** Custom `Num` trait with `Add`, `Sub`, `Mul`, `Div`, `Neg`, `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan` plus `ZERO` and `ONE` constants. Implemented for `f64` and for `CTaylor<f64, N>` (via blanket-impl gated on `ValidN<N>`).
- **D-05:** `Num` is NOT `num-traits::Float`. Rationale locked by design D2 in `docs/design/12-design-decisions.md`.
- **D-06:** `f64` is the only scalar implementation. `f32` intentionally unimplemented; design D4.

**`CTaylor::mul` recursion structure**
- **D-07:** `CTaylor::mul` ports `ctaylor_rec<T, Nvar>::multo` from `ctaylor.hpp` line-for-line, preserving the `P_N = P_{N-1} + x_N * R_{N-1}` recursion on the highest bit. Rust form uses `#[inline]` helpers parameterised by a const `N`, mechanically derived from the C++ recursion. No re-association, no FMA, no parallel accumulation.
- **D-08:** Operation order in every non-trivial expression preserved via explicit `let` bindings matching the C++ intermediate variables. Multi-term sums never collapsed into a single `a + b + c + d` expression.

**`*_expand` scalar series ports**
- **D-09:** One Rust module per C++ function: `inv_expand`, `exp_expand`, `log_expand`, `pow_expand`, `sqrt_expand`, `cbrt_expand`, `erf_expand`, `gauss_expand`. Every `*_expand` writes into a caller-provided `&mut [f64; 8]` (size = `N_MAX + 1`, stack-only) — no heap allocation.
- **D-10:** Each `*_expand` body is a textual port of the C++ recurrence with a comment block citing the upstream line range (e.g., `// tmath.hpp:124-145: pow_expand`). Primary code-review anchor for P9.
- **D-11:** Preconditions on `*_expand` inputs (e.g., `x0 > 0` for `log_expand`) enforced with `assert!` (NOT `debug_assert!`), keeping the check active in release builds. Catches the silent-NaN failure mode P10.

**Composed elementary functions (on `CTaylor<f64, N>`)**
- **D-12:** `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan` implemented as `CTaylor` -> scalar `*_expand` -> `ctaylor_rec::compose`-equivalent series composition. Matches `ctaylor_math.hpp` operation order verbatim.
- **D-13:** `pow(x, integer)` prefers `powi` over `pow` whenever the exponent is an integer literal at the call site. Phase 1 documents the convention in the crate root.

**`no_std` and libm dependency**
- **D-14:** `xcfun-ad` has a default feature `std` (default-on) and an optional feature `libm`. With `std` on, `f64` math uses `std::f64::{sqrt, exp, log, powf, ...}`. With `std` off and `libm` on, `f64` math routes through the `libm` crate. With neither, `Num for f64` is disabled.

**Golden fixture generation (C++ parity gate)**
- **D-15:** Pre-generated fixtures committed to `xcfun-ad/tests/fixtures/*.bincode`, regenerated by `cargo xtask regen-ad-fixtures` which compiles a small C++ driver linking `xcfun-master/external/upstream/taylor/` and emits deterministic records. CI does NOT regenerate — it consumes the committed fixtures.
- **D-16:** Fixture format: `bincode::serialize` of `Vec<FixtureRecord>` where `FixtureRecord` is `{ op: String, n_var: u8, inputs: Vec<f64>, coeffs: Vec<f64> }`. Parallel JSON manifest (`fixtures.json`) lists every record for debuggability and version pin (`xcfun_version_git_sha`).
- **D-17:** Fixture size budget: <= 1 MB total. If total exceeds this, drop redundant inputs (not coverage breadth).

**Property tests & bench baseline**
- **D-18:** Property tests (`proptest` 1.11) with >= 10 000 iterations per property: commutativity, associativity, distributivity, ring axioms, `exp(x) * exp(-x) ~ 1` (rel. err. <= 1e-13 with small random inputs), `log(exp(x)) ~ x`, `sqrt(x)^2 = x`, `pow(x, a) * pow(x, -a) ~ 1`, Leibniz product rule on `VAR0` coefficient.
- **D-19:** Criterion baseline: `CTaylor::<f64, N>::mul_assign` for `N in {2, 3, 4, 5, 6}` and composed `exp`/`log`/`pow` at `N = 4`. Baseline recorded at Phase 1 completion; no regression gate in v1 (PERF-01 deferred to v2).

**Floating-point hygiene (phase-1-local guardrails)**
- **D-20:** Crate-root `#![deny(clippy::float_arithmetic_side_effects)]` NOT set. Instead, functional-body `mul_add` lint introduced in Phase 2. Phase 1 keeps a single assertion in CI: the release build of `xcfun-ad` must link without `fma` intrinsic calls in `CTaylor::mul`'s object file (checked by a `cargo asm` spot-check, grep for `vfmadd` / `fmadd`).
- **D-21:** `-Cllvm-args=-fp-contract=off` set in `.cargo/config.toml` `[build]` for the release profile (Phase 0 carryover).

**Testability seams**
- **D-22:** `pub mod for_tests` gated behind `feature = "testing"` exposes `raw_coeffs<T, const N: usize>(&CTaylor<T, N>) -> &[T; 1 << N]` and direct-construction helpers. Not exposed in default build. Downstream crates depend on `xcfun-ad` with `features = ["testing"]` only in `[dev-dependencies]`.

### Claude's Discretion

(CONTEXT.md marked decisions D-02 "auto-selected — recommended", D-11 "auto-selected — recommended per P10", D-14 "auto-selected — recommended; keeps no_std story open", D-15 "auto-selected — recommended". These remain locked; no discretion items require research beyond what's already bound.)

### Deferred Ideas (OUT OF SCOPE)

- SIMD vectorisation of `CTaylor::mul` — deferred to potential v2 work.
- `CTaylor<CTaylor<f64, M>, N>` nested types — not implemented.
- Property-test reduction of rel-error budget below 1e-13 — not a Phase 1 goal.
- Crate-level benchmark regression gate — deferred to v2 (PERF-01).
- `no_std` + no `libm` pure-polynomial mode — feasible but not needed.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **AD-01** | `CTaylor<T, const N: usize>` supports `N in 0..=7` with `[T; 1 << N]` storage (no heap), bit-flag-indexed multilinear polynomial matching `xcfun-master/src/taylor/ctaylor.hpp` | §Standard Stack (ValidN<N> shape); §Architecture Patterns Pattern 1 (storage layout); `ctaylor.hpp:154-357` verbatim map |
| **AD-02** | `Num` trait supplies `Add`, `Sub`, `Mul`, `Div`, `Neg`, `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`, with `ZERO` and `ONE` constants, implemented for `f64` and `CTaylor<f64, N>` | §Architecture Patterns Pattern 2 (`Num` trait shape); composition via `*_expand` -> compose (Pattern 3); blanket impl gated on `ValidN<N>` |
| **AD-03** | `CTaylor::mul` accumulates coefficients in exactly the recursion order of `ctaylor_rec<T, Nvar>::multo` (verbatim port, not Rust-idiomatic rewrite) | §Architecture Patterns Pattern 4 (recursion layout); §Common Pitfalls P3; `ctaylor.hpp:41-65, 86-142` (source lines with base-case unrolls) |
| **AD-04** | Every `*_expand` function from `xcfun-master/src/taylor/tmath.hpp` has a byte-equivalent Rust port | §Operation Inventory table (per-function line range + recurrence + preconditions + consumer); §Common Pitfalls P9 |
| **AD-05** | `CTaylor` algebra passes `f64::to_bits` golden tests vs. C++ reference for orders 0..=3 on fixed-seed input set | §Validation Architecture; §Golden Fixture Tooling; §Standard Stack (`bincode`, `rand_xoshiro`) |
| **AD-06** | AD engine property tests (ring axioms, exp/log round-trip, sqrt-squared invariance, Leibniz product rule) run >= 10 000 iterations per property without failure | §Validation Architecture; §Testing Framework table; CONTEXT.md D-18 |

</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `CTaylor<T, N>` stack-only storage and element-wise algebra | `xcfun-ad::ctaylor` | — | Single-crate owner; no external dependency can replicate the bit-flag layout |
| `Num` trait abstraction for `f64` and polynomial types | `xcfun-ad::num` | `xcfun-ad::math` (impl site for `CTaylor<f64, N>`) | Consumed generically by every downstream functional body — MUST live in the AD crate |
| `CTaylor::mul` recursive multiply (kernel of AD) | `xcfun-ad::ctaylor_rec` | — | Load-bearing algorithmic-identity point; isolated in its own module for golden-fixture bit-equivalence testing |
| Scalar Taylor-coefficient tables (`*_expand`) | `xcfun-ad::expand::{inv,exp,log,pow,sqrt,cbrt,gauss,erf,atan,asinh,sin,cos,asin,acos}` | — | One module per C++ function, mirroring tmath.hpp file boundaries |
| Series composition of elementary functions on CTaylor | `xcfun-ad::math` (composed-function layer) | `xcfun-ad::expand`, `xcfun-ad::compose` | Thin wrappers that call `*_expand` then `compose::compose` — mirrors ctaylor_math.hpp |
| Golden-fixture generation (C++ driver) | `xtask::regen_ad_fixtures` | — | Lives outside `xcfun-ad` because fixture generation needs a C++ toolchain; preserves hermetic `cargo build` in `xcfun-ad` |
| Golden-fixture consumption (test binaries) | `xcfun-ad/tests/golden_*.rs` | — | Integration tests deserialise `bincode` files committed to `xcfun-ad/tests/fixtures/` |
| Property tests | `xcfun-ad/tests/props_*.rs` (integration) or `xcfun-ad/src/**/tests` (unit) | `proptest` dev-dependency | Ring axioms, Leibniz, roundtrips — all live in test scope; zero runtime impact |
| Micro-benchmark baseline (CTaylor::mul + composed ops) | `xcfun-ad/benches/*.rs` (criterion) | — | Records Phase 1 baseline numbers; no regression gate in v1 |

---

## Standard Stack

### Core (crate-runtime)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust edition | 2024 (MSRV 1.85) | Const generics for `CTaylor<T, const N: usize>` and `[T; 1 << N]` const-expression array length | Required by CLAUDE.md; enables stable-Rust const-generic recursion via `ValidN<N>` [VERIFIED: project CLAUDE.md] |
| `libm` | 0.2 (workspace pin) | `no_std` f64 math fallback when feature `std` is disabled | Workspace already declares; D-14 locks its use under `feature = "libm"`. Not needed on default `std` feature [VERIFIED: Cargo.toml:15] |

### Dev (test / bench / fixture)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `approx` | 0.5.1 | `assert_relative_eq!` for tolerance-checked numeric assertions | Pinned in `.planning/research/STACK.md`; hold at 0.5 (0.6.0-rcN is an RC) [VERIFIED: STACK.md row "approx"] |
| `proptest` | 1.11.0 | >= 10 000-iter property tests on ring axioms + derivative identities | Bumped from 1.6 in STACK.md for improved shrinking of nested structures [VERIFIED: STACK.md] |
| `rstest` | 0.26.1 | Parameterised tests across `(op, x0_bin, N)` tuples | Pinned in STACK.md; macro-surface crate; lock to avoid `#[case]` signature drift [VERIFIED: STACK.md] |
| `rand_xoshiro` | 0.8.0 | Deterministic RNG for property-test seeds; fixture input sampling | Aligns with `rand 0.9`; required for reproducible failing-input retrieval [VERIFIED: STACK.md] |
| `criterion` | 0.8.2 | Benchmark baseline for `CTaylor::mul` and composed ops (D-19) | Default-features-off + `html_reports` feature per STACK.md [VERIFIED: STACK.md] |
| `bincode` | 1.3.x or 2.0.x — REQUIRES CONFIRMATION | Serialise `Vec<FixtureRecord>` into `tests/fixtures/*.bincode` (D-16) | [ASSUMED — bincode is the default choice for compact binary test fixtures; 2.0 is current major but has API churn; recommend 1.3.3 for stability unless `serde` interop already pulled into the crate] |
| `serde` + `serde_json` | 1.0.x | Generate the JSON fixture manifest alongside bincode (D-16: "parallel JSON manifest (`fixtures.json`)") | serde_json pinned at workspace-level (>=1.0.149) per STACK.md [VERIFIED: STACK.md] |

### Build tool (for fixture regeneration)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cc` | ^1.2.60 | In `xtask`, compile a tiny C++ driver (`~200 LOC`) that links `xcfun-master/external/upstream/taylor/` and emits the fixture records | STACK.md bumps `cc 1.1 -> 1.2.60`; `parallel` feature already enabled [VERIFIED: STACK.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff | Why Rejected |
|------------|-----------|----------|--------------|
| `bincode` | `ciborium` (CBOR) | More stable across versions; cubecl has adopted it | Defer — bincode is already widely used; CBOR would add cross-format debugging complexity; not a parity concern [VERIFIED: STACK.md row "bincode"] |
| `bincode` | `postcard` | `no_std`-friendly, smaller output | Overkill for a dev-dep; tests already run on `std` [ASSUMED] |
| `proptest` | `quickcheck` | Simpler API | Proptest's shrinking is strictly better for nested `CTaylor` structures [VERIFIED: STACK.md alternatives table] |
| Property shrinker based on manual bisection | Proptest's built-in shrinker | — | Manual bisection duplicates work and misses structural shrinks |

### Installation (workspace edits)

```toml
# Workspace Cargo.toml -- additions for Phase 1
[workspace.dependencies]
# existing:
#   thiserror = "2.0.18"
#   bitflags = "2.11"
#   approx = "0.5"
#   anyhow = "1.0"
#   tracing = "0.1"
#   libm = "0.2"

# additions for phase 1:
proptest = "=1.11.0"
rstest = "=0.26.1"
rand_xoshiro = "=0.8.0"
criterion = { version = "=0.8.2", default-features = false, features = ["html_reports"] }
bincode = "1.3"                # fixture format (D-16)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.149"         # fixture manifest
cc = { version = "^1.2.60", features = ["parallel"] }   # xtask-only

# crates/xcfun-ad/Cargo.toml additions
[features]
default = ["std"]
std = []
libm = ["dep:libm"]
testing = []   # D-22: exposes `pub mod for_tests`

[dev-dependencies]
approx = { workspace = true }
proptest = { workspace = true }
rstest = { workspace = true }
rand_xoshiro = { workspace = true }
bincode = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
criterion = { workspace = true }

[[bench]]
name = "mul_bench"
harness = false

[[bench]]
name = "compose_bench"
harness = false
```

**Version verification:**
```bash
npm view -- not applicable; crate versions verified via:
cargo search proptest         # confirms 1.11.0 currently published
cargo search rstest           # 0.26.1
cargo search criterion        # 0.8.2
cargo search approx           # 0.5.1 stable
cargo search bincode          # 1.3.3 current stable
```
All versions `[VERIFIED: .planning/research/STACK.md (dated 2026-04-18)]`. `bincode 1.3.3` is the last v1 release; `2.0.x` has API churn and is `[ASSUMED]` to be less stable for a test-fixture use case — **call out at planning time for user confirmation**.

---

## Architecture Patterns

### System Architecture Diagram

```
                          +---------------------------+
                          | xcfun-ad/tests/fixtures/  |
                          |  *.bincode + fixtures.json|
                          | (committed; read-only)    |
                          +---------------------------+
                                         ^
                                         | (xtask regen-ad-fixtures)
                                         | (dev-time, requires C++ tc)
                                         |
      +---------------------+    (writes)+--------------------------+
      | xcfun-master/       |----------->| xtask/src/regen_ad_..rs  |
      | external/upstream/  |            |  + C++ driver (cc crate) |
      | taylor/*.hpp        |            |  links ctaylor/tmath hpps|
      +---------------------+            +--------------------------+

                                         +---------------------------+
                                         | xcfun-ad/tests/fixtures/  |<--(read at CI time)
                                         +---------------------------+
                                                        |
                                                        v
      +--------------------+   calls   +--------------------+
      | user code /        |---------->| xcfun-ad crate     |
      | xcfun-core (later) |           | (zero runtime deps)|
      +--------------------+           +----+---------------+
                                            |
                            (internal data flow)
                                            v
    +--------------------+   compose +------+---------------+
    | xcfun-ad::expand   |---------->| xcfun-ad::compose    |
    | *_expand(&mut [f64;| (fills   | (ctaylor_rec::compose|
    |   N+1], x0, ...)   |  coeffs) |  -> multo_skipconst) |
    +--------------------+          +--+-------------------+
                                       |
                                       v
                         +---------------------------+
                         | xcfun-ad::ctaylor         |
                         |  CTaylor<T, N>            |
                         |  Add/Sub/Mul/Neg/Div      |
                         |  + Num trait impl         |
                         +---------------------------+
                                       |
                            (used-by chain)
                                       v
         +---------------------+   exp/log/pow/...  +---------------------+
         | xcfun-ad::num (Num) |-----|------------->| xcfun-ad::math      |
         | trait for f64 +     |     |              | ctaylor_exp/log/pow |
         | CTaylor<f64, N>     |     |              | = expand + compose  |
         +---------------------+     |              +---------------------+
                                     |
                                     v
                         (CI: cargo asm spot-check on CTaylor::mul
                          asserts no vfmadd / fmadd; -fp-contract=off)
```

**Data flow trace for `CTaylor<f64, 3>::exp()`:**
1. Caller invokes `t.exp()` where `t: CTaylor<f64, 3>`.
2. `Num::exp` dispatches to `math::ctaylor_exp`.
3. `math::ctaylor_exp` declares a stack scratch `[f64; 4]` (size `N_MAX + 1 = 8`; per-call sizing uses `N + 1 = 4`).
4. `expand::exp_expand(&mut scratch[..4], t.c[CNST])` — fills Taylor coefficients of `exp(t.c[0] + h)` for `h` at order N.
5. `compose::compose(&mut result.c, &t.c, &scratch[..4])` — series composition; internally calls `multo_skipconst` N times.
6. Returns `CTaylor { c: result.c }`.

### Recommended Project Structure

```
crates/xcfun-ad/
  Cargo.toml
  src/
    lib.rs                      # re-exports, crate-root docs, VAR0..VAR7 consts
    valid_n.rs                  # sealed ValidN<N> trait + 8 impls (D-02)
    ctaylor.rs                  # CTaylor<T, N> struct, ops (Add, Sub, Neg, Mul, Mul<f64>)
    ctaylor_rec/                # multiply recursion — isolate for golden fixtures
      mod.rs                    #   dispatch table by N
      mul.rs                    #   mul, mul_set (accumulating and non-)
      multo.rs                  #   multo, multo_skipconst (D-07)
      compose.rs                #   ctaylor_rec::compose equivalent
    num.rs                      # Num trait def + f64 impl + CTaylor<f64, N> blanket impl
    expand/                     # one file per C++ *_expand (D-09, D-10)
      mod.rs                    #   pub use re-exports
      inv.rs                    #   inv_expand  (tmath.hpp:124-129)
      exp.rs                    #   exp_expand  (tmath.hpp:132-139)
      log.rs                    #   log_expand  (tmath.hpp:142-151)
      pow.rs                    #   pow_expand  (tmath.hpp:154-161)
      sqrt.rs                   #   sqrt_expand (tmath.hpp:164-170)
      cbrt.rs                   #   cbrt_expand (tmath.hpp:172-178)
      atan.rs                   #   atan_expand (tmath.hpp:180-198)
      gauss.rs                  #   gauss_expand (tmath.hpp:204-215)
      erf.rs                    #   erf_expand  (tmath.hpp:218-225)
      asinh.rs                  #   asinh_expand (tmath.hpp:260-274)
      sin_cos.rs                #   sin/cos_expand (tmath.hpp:227-257)
      asin_acos.rs              #   asin/acos_expand (tmath.hpp:277-314)
      tfuns.rs                  #   tfuns::{mul, multo, integrate, differentiate,
                                #         shift, compose, stretch} (tmath.hpp:36-121)
    math.rs                     # composed elementary funcs (D-12): reciprocal, exp, log,
                                # pow, sqrt, powi, erf, asinh, atan, cbrt, sin, cos
    for_tests.rs                # feature="testing" seam (D-22)
  benches/
    mul_bench.rs                # D-19: CTaylor::mul_assign for N in 2..=6
    compose_bench.rs            # D-19: exp/log/pow at N = 4
  tests/
    fixtures/                   # commited .bincode + fixtures.json (D-15/16)
    golden_mul.rs               # Tier-1 fixture vs CTaylor::mul (to_bits for N<=3)
    golden_expand.rs            # Tier-1 fixture vs expand functions
    golden_composed.rs          # Tier-1 fixture vs math::ctaylor_exp/log/pow/...
    props_ring.rs               # Tier-2 proptest: assoc, comm, distrib
    props_leibniz.rs            # Tier-2 proptest: product rule on VAR0
    props_roundtrips.rs         # Tier-2 proptest: exp(-x)*exp(x)~1, log(exp(x))~x,
                                #   sqrt(x)^2~x, pow(x,a)*pow(x,-a)~1

xtask/src/bin/
  regen_ad_fixtures.rs          # D-15: compiles C++ driver, invokes, serialises records

xtask/assets/
  regen_ad_fixtures/
    driver.cpp                  # ~200 LOC C++ driver linking xcfun-master/taylor/
    CMakeLists.txt or build.rs  # compile via cc crate

.cargo/config.toml              # [build]/[target] -- -Cllvm-args=-fp-contract=off  (D-21)
```

### Pattern 1: Stack-only `CTaylor<T, const N: usize>` with `ValidN<N>` sealed trait

**What:** Storage type locked to `[T; 1 << N]` for `N in 0..=7`, with compile-time bound enforced without nightly features.

**When to use:** Every polynomial type in `xcfun-ad` — there are no other polynomial types.

**Example (source: adapted from CONTEXT.md D-01/D-02 + docs/design/02-data-structures.md §1.1):**

```rust
// crate: xcfun-ad, file: src/valid_n.rs

/// Sealed marker trait: valid nvar counts for CTaylor (0..=7).
///
/// Stable-Rust-compatible compile-time bound; implemented only
/// for the 8 allowed N values. Attempting CTaylor<f64, 8> fails
/// at monomorphisation with "trait bound not satisfied".
pub trait ValidN<const N: usize>: sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

pub struct Bound;
impl sealed::Sealed for Bound {}

// 8 impls, one per valid N. Each is a no-op marker.
impl ValidN<0> for Bound {}
impl ValidN<1> for Bound {}
impl ValidN<2> for Bound {}
impl ValidN<3> for Bound {}
impl ValidN<4> for Bound {}
impl ValidN<5> for Bound {}
impl ValidN<6> for Bound {}
impl ValidN<7> for Bound {}
```

```rust
// crate: xcfun-ad, file: src/ctaylor.rs

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CTaylor<T: Num + Copy, const N: usize>
where
    Bound: ValidN<N>,
{
    pub c: [T; 1 << N],   // size 2^N
}
```

**Notes:**
- The array length `1 << N` is a stable-Rust const expression (Rust 1.79+).
- No `#[repr(C)]`-affecting alignment concern: `T = f64` has alignment 8; `[f64; 1 << N]` has alignment 8 for all N in 0..=7.
- `PartialEq` is safe because it is bitwise; `f64::NaN` coefficients propagate as C++ does, matching `ctaylor.hpp:154` (no `Eq`).

### Pattern 2: `Num` trait with blanket impl gated on `ValidN<N>`

**What:** The numeric abstraction that functional bodies are generic over.

**When to use:** Any time a function needs to compile for both `f64` (scalar eval) and `CTaylor<f64, N>` (AD eval).

**Example (Source: adapted from CONTEXT.md D-04 + docs/design/02-data-structures.md §1.2):**

```rust
// crate: xcfun-ad, file: src/num.rs

pub trait Num:
    Copy
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;

    fn reciprocal(self) -> Self;
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn log(self) -> Self;
    fn pow(self, exponent: Self) -> Self;    // pow(x, y) -> x^y
    fn powi(self, exponent: i32) -> Self;    // integer exponent fast path
    fn erf(self) -> Self;
    fn asinh(self) -> Self;
    fn atan(self) -> Self;
}

// impl for f64 (scalar path)
impl Num for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    fn reciprocal(self) -> Self { 1.0 / self }
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")] { self.sqrt() }
        #[cfg(all(not(feature = "std"), feature = "libm"))] { libm::sqrt(self) }
    }
    // exp, log, pow, powi, erf, asinh, atan -- similarly gated on feature
}

// blanket impl for CTaylor<f64, N>
impl<const N: usize> Num for CTaylor<f64, N>
where
    Bound: ValidN<N>,
{
    const ZERO: Self = CTaylor { c: [0.0; 1 << N] };
    const ONE: Self = {
        let mut c = [0.0; 1 << N];
        c[0] = 1.0;
        CTaylor { c }
    };
    fn reciprocal(self) -> Self { crate::math::ctaylor_reciprocal(self) }
    // etc.
}
```

**CRITICAL — D-06:** No `impl Num for f32`. Not now, not ever — f32 unit roundoff is 6e-8, > 1e-12 by 4 orders of magnitude.

### Pattern 3: `*_expand` scalar series + `ctaylor_rec::compose` composition

**What:** Every composed elementary function (`reciprocal`, `exp`, `log`, `pow`, `sqrt`, `erf`, `atan`, `asinh`) is a 3-step pattern: allocate a stack scratch array, fill with `*_expand`, compose.

**When to use:** All 9 composed functions in D-12 follow this pattern line-for-line with `ctaylor_math.hpp`.

**Example (source: tmath.hpp + ctaylor_math.hpp:71-81 for `exp`):**

```rust
// crate: xcfun-ad, file: src/math.rs

pub fn ctaylor_exp<const N: usize>(t: CTaylor<f64, N>) -> CTaylor<f64, N>
where
    Bound: ValidN<N>,
{
    // tmath.hpp:132-139: exp_expand
    //   t[0] = exp(x0); t[i] = t[0] / i!  (for i = 1..=N)
    // We reserve N+1 coefficients (N_MAX = 7 => max 8).
    let mut scratch = [0.0_f64; 8];
    let slice: &mut [f64] = &mut scratch[..=N];
    expand::exp::exp_expand(slice, t.c[CNST]);

    // ctaylor_math.hpp:71-81: exp(ctaylor) => compose(res, t.c, tmp)
    let mut result = CTaylor::ZERO;
    crate::ctaylor_rec::compose::compose(&mut result.c, &t.c, slice);
    result
}
```

### Pattern 4: `ctaylor_rec::mul` verbatim recursion (ALGORITHMIC IDENTITY)

**What:** The mul recursion is expressed as a const-generic recursive call tree in Rust, with explicit base cases at `N = 0, 1, 2` matching the C++ template specialisations.

**When to use:** This is the single most load-bearing port in Phase 1. Rewriting as triple-nested flat loop breaks parity (P3 in PITFALLS.md).

**Base cases (directly ported from ctaylor.hpp:86-142):**

```rust
// xcfun-ad::ctaylor_rec::mul

// N = 0 (ctaylor.hpp:86)
#[inline] fn mul_n0_acc(dst: &mut [f64; 1], x: &[f64; 1], y: &[f64; 1]) {
    dst[0] += x[0] * y[0];
}
#[inline] fn mul_n0_set(dst: &mut [f64; 1], x: &[f64; 1], y: &[f64; 1]) {
    dst[0] = x[0] * y[0];
}

// N = 1 (ctaylor.hpp:94-115)
#[inline] fn mul_n1_acc(dst: &mut [f64; 2], x: &[f64; 2], y: &[f64; 2]) {
    dst[0] += x[0] * y[0];
    let t = x[0] * y[1] + x[1] * y[0];   // explicit let per D-08
    dst[1] += t;
}
#[inline] fn mul_n1_set(dst: &mut [f64; 2], x: &[f64; 2], y: &[f64; 2]) {
    dst[0] = x[0] * y[0];
    let t = x[0] * y[1] + x[1] * y[0];
    dst[1] = t;
}
#[inline] fn multo_n1(dst: &mut [f64; 2], y: &[f64; 2]) {
    // Exact order from ctaylor.hpp:103-106 — writes dst[1] BEFORE dst[0]
    dst[1] = dst[1] * y[0] + dst[0] * y[1];
    dst[0] *= y[0];
}

// N = 2 (ctaylor.hpp:118-152)
#[inline] fn multo_n2(dst: &mut [f64; 4], y: &[f64; 4]) {
    // ctaylor.hpp:131-135 -- exact assignment order
    let t3 = dst[0] * y[3] + dst[3] * y[0] + dst[1] * y[2] + dst[2] * y[1];
    dst[3] = t3;
    let t2 = dst[0] * y[2] + dst[2] * y[0];
    dst[2] = t2;
    let t1 = dst[0] * y[1] + dst[1] * y[0];
    dst[1] = t1;
    dst[0] = dst[0] * y[0];
}
```

**General-N recursion (ctaylor.hpp:55-59 for multo, :43-47 for mul):**

```rust
// For N >= 3, we dispatch to the generic recursive form.
// HALF = 1 << (N - 1)

// Pseudocode: the real code must be parameterised via const-generic dispatch
// or via a macro expansion for each N in {3, 4, 5, 6, 7}.
//
// multo(dst, y) with size 1 << N:
//   1. multo_{N-1}(&mut dst[HALF..], &y[..HALF])
//   2. mul_{N-1}_acc(&mut dst[HALF..], &dst[..HALF], &y[HALF..])
//      ^ !!! CRITICAL !!! This step requires reading dst[..HALF] while
//      writing dst[HALF..]. In C++ this is trivially aliased. In Rust
//      we need `split_at_mut(HALF)` to borrow both halves disjointly.
//   3. multo_{N-1}(&mut dst[..HALF], &y[..HALF])
```

**Note on borrow-checker discipline:** The existing `crates/xcfun-ad/src/compose.rs:76-88` already uses a `let lo_copy: Vec<f64> = dst[..half].to_vec();` trick to handle step 2, but this is a heap allocation that violates D-01. The correct solution is `let (lo, hi) = dst.split_at_mut(HALF);` or a stack-allocated scratch buffer sized by the const-generic parameter. **Flag for planner: existing heap-allocation in `multo_skipconst` must be replaced with a stack/split-at-mut approach.**

### Anti-Patterns to Avoid

- **Flat triple-loop `CTaylor::mul` rewrite.** Mathematically equivalent, numerically divergent — P3 in PITFALLS.md. Always use the recursive form with explicit base cases at N ∈ {0, 1, 2}.
- **`a.mul_add(b, c)` anywhere in `xcfun-ad`.** The C++ reference does not emit FMA by default; using `mul_add` in Rust introduces single-rounded FMA instructions that diverge by 1 ULP per op (P1). CI must grep for `mul_add` and fail the build if found in `xcfun-ad/src/`.
- **`Vec<f64>` / `Box<[f64]>` scratch in transcendentals.** Violates D-01 ("stack only, no heap, no Box, no Vec"). Use `let mut scratch = [0.0; 8]; let slice = &mut scratch[..=N];` — this works for all N in 0..=7 because N_MAX = 7.
- **`debug_assert!` on `*_expand` preconditions.** Debug-only asserts compile out in release, silently producing NaN on invalid inputs (P10). CONTEXT.md D-11 mandates `assert!`.
- **Fluent chain `t.c[CNST | VAR0 | VAR1]` replacement by a builder API.** Makes cross-reference with the C++ source harder; CONTEXT.md explicitly "prefer the literal `t.c[CNST | VAR0 | VAR1]` pattern over a fluent builder" (§specifics).
- **Merged arithmetic expressions.** Do not write `let t = x[0]*y[3] + x[3]*y[0] + x[1]*y[2] + x[2]*y[1];` where the C++ writes `dst[3] = x[0]*y[3] + x[3]*y[0] + x[1]*y[2] + x[2]*y[1];` — the compiler may re-associate. D-08: bind intermediate `let` variables matching the C++ source.
- **Ascending Horner composition.** C++ `ctaylor_rec::compose` uses descending iteration (`for i in (0..Nvar).rev()`). Porting as ascending breaks parity for VWN5/PW92 (P11).

---

## Operation Inventory

### CTaylor Core Operations (ported from ctaylor.hpp + ctaylor_math.hpp)

| Operation | C++ lines | Rust file | Action | Complexity |
|-----------|-----------|-----------|--------|------------|
| `zero()` / constructor (scalar) | `ctaylor.hpp:179-187` | `ctaylor.rs` | Zero-init `[T; 1 << N]`, set `c[CNST] = value` | O(1 << N) |
| `seed(value, var_slot, slope)` | `ctaylor.hpp:199-209` | `ctaylor.rs` | Seed first-derivative slot; asserts `var_slot.count_ones()==1` | O(1 << N) |
| `set(&mut, idx, value)` | `ctaylor.hpp:241-249` | `ctaylor.rs` | `c[idx] = value`, assert `idx < 1<<N` | O(1) |
| `get(idx)` | `ctaylor.hpp:250-261` | `ctaylor.rs` | `c[idx]` | O(1) |
| `Add<CTaylor>` | `ctaylor.hpp:295-311, 468-474` | `ctaylor.rs` | Elementwise fused add | O(1 << N) |
| `Sub<CTaylor>` | `ctaylor.hpp:278-294, 504-510` | `ctaylor.rs` | Elementwise fused sub | O(1 << N) |
| `Neg` | `ctaylor.hpp:263-277` | `ctaylor.rs` | Elementwise negation | O(1 << N) |
| `Mul<CTaylor>` | `ctaylor.hpp:360-380` + `multo:55-59` | `ctaylor_rec/multo.rs` | Recursive multiply; **load-bearing** | O((1 << N)^1.58) — actually 3 * T(N-1) |
| `MulAssign<CTaylor>` | `ctaylor.hpp:339-355` | `ctaylor_rec/multo.rs` | `ctaylor_rec<N>::multo(self.c, rhs.c)` | Same |
| `Mul<f64>` / `Mul<CTaylor>` for scalar | `ctaylor.hpp:421-451` | `ctaylor.rs` | Elementwise scalar multiply | O(1 << N) |
| `Div<CTaylor>` | `ctaylor_math.hpp:31-46` | `math.rs` (`ctaylor_div`) | `inv_expand -> compose -> mul` — NOT a primitive op | O((1 << N)^1.58) |
| `Div<f64>` | `ctaylor.hpp:326-337` | `ctaylor.rs` | `self *= 1.0 / rhs` (C++ does this) | O(1 << N) |

### `*_expand` Scalar Series (tmath.hpp)

All write to `&mut [f64; 8]` caller-provided scratch, using `&mut slice[..=N]` for N-specific sizing. Stack only. `assert!` preconditions per D-11.

| Function | tmath.hpp lines | Recurrence shape | Preconditions | Downstream consumers |
|----------|----------------|-------------------|----------------|----------------------|
| `inv_expand` | 124-129 | `t[0] = 1/a; t[i] = -t[i-1] * t[0]` | `a != 0` (assert) | `ctaylor_math.hpp:7-28` — operator/; `atan_expand`, `asinh/asin/acos_expand` (via `pow_expand` composition), `ctaylor_math.hpp:31-46` — ctaylor/ctaylor |
| `exp_expand` | 132-139 | `t[0] = exp(x0); t[i] = t[0] / i!` (via cumulative ifac) | None | `ctaylor_math.hpp:71-81` — `exp`; `gauss_expand` — used with `exp_expand<-a*a>` as initial state |
| `log_expand` | 142-151 | `t[0] = log(x0); t[i] = (x0inv^i / i) * (2*(i&1)-1)` | `x0 > 0` (assert) | `ctaylor_math.hpp:105-115` — `log` |
| `pow_expand` | 154-161 | `t[0] = pow(x0, a); t[i] = t[i-1] * x0inv * (a - i + 1) / i` | `x0 > 0` (assert — P10 source of silent NaN) | `ctaylor_math.hpp:120-131` — `pow`; `asinh_expand`, `asin_expand`, `acos_expand` (internal `pow_expand(tmp, tmp[0], -0.5)`) |
| `sqrt_expand` | 164-170 | `t[0] = sqrt(x0); t[i] = t[i-1] * ((3*x0inv)/(2i) - x0inv)` | `x0 > 0` (assert) | `ctaylor_math.hpp:134-145` — `sqrt`; `sqrtx_asinh_sqrtx` (two call sites) |
| `cbrt_expand` | 172-178 | `t[0] = cbrt(x0); t[i] = t[i-1] * ((4*x0inv)/(3i) - x0inv)` | `x0 > 0` (assert) | `ctaylor_math.hpp:148-159` — `cbrt` |
| `atan_expand` | 180-198 | Uses `inv_expand(1 + a^2)` then `tfuns::compose` with `x = [0, 2a, 1, 0, ...]` then `tfuns::integrate` then set `t[0] = atan(a)` | None (analytic everywhere on reals) | `ctaylor_math.hpp:181-192` — `atan` |
| `gauss_expand` | 204-215 | Composes `exp_expand(-a*a)` with `stretch(-2a)` then `multo` with alternating signs | None | `erf_expand` (base) |
| `erf_expand` | 218-225 | `gauss_expand(a)` then scale by `2/sqrt(pi)` then `integrate` then set `t[0] = erf(a)` | None (erf analytic) | `ctaylor_math.hpp:195-206` — `erf` |
| `asinh_expand` | 260-274 | Build `tmp = [1+a^2, 2a, 1, 0, ...]`; `pow_expand(t, tmp[0], -0.5)`; compose with tmp; integrate; set `t[0] = asinh(a)` | None | `ctaylor_math.hpp:257-268` — `asinh`; `sqrtx_asinh_sqrtx` high-branch |
| `sin_expand` | 227-241 | `t[2i] = fac * sin(a); fac /= (2i+1); t[2i+1] = fac * cos(a); fac /= -(2i+2);` | None | `ctaylor_math.hpp:209-220` — `sin` |
| `cos_expand` | 243-257 | Symmetric to `sin_expand` with swapped sin/cos and sign conventions | None | `ctaylor_math.hpp:222-233` — `cos` |
| `asin_expand` | 277-291 | Builds `tmp = [1-a^2, -2a, -1, 0, ...]`; pow_expand, compose, integrate; `t[0] = asin(a)` **[BUG in upstream: tmath.hpp:290 sets `t[0]=asinh(a)` not `asin(a)` — verify before porting]** | None (for \|a\| < 1) | `ctaylor_math.hpp:236-244` — `asin` |
| `acos_expand` | 299-314 | Like asin but with sign flip | None (for \|a\| < 1) | `ctaylor_math.hpp:246-254` — `acos` |

**CRITICAL upstream bug flag:** `tmath.hpp:290` reads `t[0] = asinh(a);` inside `asin_expand` — this sets `t[0]` to `asinh(a)`, not `asin(a)`. Similarly `tmath.hpp:313` inside `acos_expand` reads `t[0] = asinh(a);`. These appear to be transcription typos in the xcfun source. **Phase 1 is NOT responsible for fixing upstream bugs**; the port must be byte-equivalent. The fixture driver MUST compile against the unmodified `xcfun-master/.../tmath.hpp` so the fixtures record the (possibly bugged) C++ output, and the Rust port matches. Document this in the `asin_expand` / `acos_expand` module headers as a warning. `[VERIFIED: tmath.hpp:290, tmath.hpp:313]`

### `tfuns<T, N>::*` Helper Recurrences (tmath.hpp:36-121)

Required internals used by `atan_expand`, `gauss_expand`, `erf_expand`, `asinh_expand`, `asin_expand`, `acos_expand`, and (beyond Phase 1 scope) `sqrtx_asinh_sqrtx`.

| Helper | tmath.hpp lines | What it does |
|--------|-----------------|--------------|
| `tfuns::mul(z, x, y)` | 37-43 | Truncated 1D convolution: `z[i] = sum_{j=0..i} x[j] * y[i-j]` |
| `tfuns::multo(z, x)` | 45-51 | In-place `z *= x` — writes z descending so the read hits unmodified high-index entries |
| `tfuns::integrate(x)` | 53-56 | `x[i] = x[i-1] / i` for `i = N down to 1`, leaves `x[0]` undefined |
| `tfuns::differentiate(x)` | 57-61 | `x[i-1] = i * x[i]`; sets `x[N] = 0` |
| `tfuns::shift(x, d)` | 63-78 | Adds `d` to expansion point; computes powers of `d` then combines coefficients with binomial-like factors — **subtle** |
| `tfuns::compose(f, x)` | 80-113 | `switch(N)` cascade at compile time; explicit unrolled cases for N ∈ {0,1,2,3,4,5,6}. Assumes `x[0] = 0` (expansion point subtracted). |
| `tfuns::stretch(t, a)` | 114-120 | `t[i] *= a^i` for `i >= 1`; leaves `t[0]` alone |

**Note on `tfuns::compose`:** This is the scalar Taylor composition routine (distinct from `ctaylor_rec::compose` which composes on multilinear polynomials). It is the primary consumer of `x[0] = 0` invariant — the caller must arrange the expansion point. Phase 1 ports only the fall-through switch bodies for N ≤ 6 (max order 6 per REQUIREMENTS.md). **D-10 mandates** citing `tmath.hpp:82-113` line numbers in the Rust port's doc-comment.

### Composed CTaylor Elementary Functions (ctaylor_math.hpp)

All implemented in `src/math.rs` as thin wrappers: declare stack `[f64; 8]`, call `expand::*_expand(&mut scratch[..=N], t.c[CNST], ...)`, call `ctaylor_rec::compose::compose(&mut res.c, &t.c, &scratch[..=N])`.

| Function | ctaylor_math.hpp lines | Pattern | N_MAX scratch | Preconditions |
|----------|------------------------|---------|---------------|---------------|
| `reciprocal` / `1.0 / t` | 7-28 | `inv_expand + compose` + scalar-mul by x | 8 | `t.c[CNST] != 0` |
| `ctaylor / ctaylor` | 31-46 | `inv_expand + compose + *= t1` | 8 | `t2.c[CNST] != 0` |
| `exp` | 71-81 | `exp_expand + compose` | 8 | None |
| `expm1` | 84-102 | `exp_expand + compose`; constant term specialised via `2*exp(x/2)*sinh(x/2)` for \|c[0]\|<1e-3 | 8 | None |
| `log` | 104-115 | `log_expand + compose` | 8 | `t.c[CNST] > 0` |
| `pow(ctaylor, double)` | 117-131 | `pow_expand + compose` | 8 | `t.c[CNST] > 0` |
| `sqrt` | 133-145 | `sqrt_expand + compose` | 8 | `t.c[CNST] > 0` |
| `cbrt` | 147-159 | `cbrt_expand + compose` | 8 | `t.c[CNST] > 0` |
| `pow(ctaylor, int)` (powi) | 165-178 | Integer fast path: `n>0` loops `res *= t`; `n<0` defers to `pow(t, double(n))`; `n=0` returns 1 | 0 | None — analytic everywhere (including c[CNST] = 0 for positive n) |
| `atan` | 180-192 | `atan_expand + compose` | 8 | None |
| `erf` | 194-206 | `erf_expand + compose` | 8 | None |
| `asinh` | 256-268 | `asinh_expand + compose` | 8 | None |
| (deferred to later phase) `sqrtx_asinh_sqrtx` | 275-325 | Pade-[8,8] approximation for \|c[0]\| < 0.5; unstable fallback otherwise | 9 (`ASINH_TABSIZE`) | `c[0] > -0.5` (assert) |

**AD-02 scope note:** REQUIREMENTS.md AD-02 lists `sqrt, exp, log, pow, powi, erf, asinh, atan` plus `reciprocal` — the 9 composed functions. `sin, cos, asin, acos, cbrt, expm1` are NOT in AD-02 but are needed later (Phase 2+ functional ports consume `cbrt` heavily: LDA r_s, metaGGA tau). **Recommendation for planner:** include `cbrt` in Phase 1 even though it's not in AD-02 — the `cbrt_expand` function is already in D-09 and Phase 2's LDA ports will fail without it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| General Taylor polynomial AD | Generic reverse-mode tape or forward-dual | In-house `CTaylor` verbatim port | No existing Rust crate replicates xcfun's bit-flag multilinear polynomial layout; algorithmic identity is required for 1e-12 parity (D1, D2) [VERIFIED: STACK.md alternatives table + SUMMARY.md] |
| Elementary-function series expansion | Custom Chebyshev/Remez approximation | C++-verbatim `*_expand` recurrences | The xcfun recurrences are numerically optimised; rewriting for "stability" silently changes rounding (P9) [VERIFIED: PITFALLS.md P9] |
| `f64::mul_add` for "performance" | Fused multiply-add | Plain `a*b + c` with explicit `let` intermediates | C++ does not emit FMA; FMA introduces single-rounded ops diverging by 1 ULP per call (P1) [VERIFIED: PITFALLS.md P1] |
| Numeric-trait abstraction | `num_traits::Float` | Custom `Num` trait in `xcfun-ad` | `Float` carries `is_nan`, `classify`, `FloatCore` — meaningless on polynomial types (D2, D-05) [VERIFIED: design/12:23-32] |
| Const-generic bound enforcement | `generic_const_exprs` nightly trick | Sealed `ValidN<N>` trait with 8 impls | `generic_const_exprs` is nightly-only and unstable; sealed trait is stable-Rust-compatible (D-02) [VERIFIED: CONTEXT.md D-02] |
| Float-comparison assertions in tests | Manual ULP-diff logic | `approx::assert_relative_eq!` or `assert_eq!(a.to_bits(), b.to_bits())` | Standard in Rust numerical testing; the `to_bits` form is exactly what golden-fixture tests need for parity [VERIFIED: STACK.md] |
| Random seed management in proptest | Hand-rolled PRNG | `proptest` default + fixed-seed override | Proptest's shrinking is the load-bearing feature; hand-rolled PRNG loses shrinking [VERIFIED: proptest docs] |
| C++-driver fixture generation shell scripts | Bash + `g++` direct invocation | `xtask` + `cc::Build` | Portable across OS; type-checked; reuses the same compiler settings; matches the D-15 "cargo xtask regen-ad-fixtures" contract [VERIFIED: design/10 §3.8] |

**Key insight:** Every temptation to "improve" the xcfun algebra for "clarity" or "performance" is a parity regression. The AD engine's only job is to produce `f64::to_bits`-identical outputs on orders 0..=3 modulo libm drift. Treat every line of `ctaylor.hpp`, `ctaylor_math.hpp`, and `tmath.hpp` as a load-bearing wire — cutting one of them requires writing (and justifying with measurements) a replacement fixture.

---

## Runtime State Inventory

(Not applicable to Phase 1 — greenfield port of a new crate layer. No stored data, live service config, or OS-registered state from prior phases enters Phase 1's work surface. Phase 0 scaffolding may have created `.cargo/config.toml`; Phase 1 consumes it but does not migrate it.)

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — greenfield crate | — |
| Live service config | None | — |
| OS-registered state | None | — |
| Secrets/env vars | None | — |
| Build artifacts | None yet; Phase 1 will emit `target/debug/libxcfun_ad.rlib` and `tests/fixtures/*.bincode` | — |

**Pre-existing code to reconcile:** `crates/xcfun-ad/src/{lib.rs, ctaylor.rs, compose.rs, math.rs, num.rs, tmath.rs}` (2912 LOC total) has been partially implemented — but with a different `N` semantic (treats `N` as the array size rather than nvar count) and with heap allocations in transcendentals (`vec![0.0; nvar + 1]`). This is a **code-reconciliation task for Wave 0**, not a runtime-state migration. The planner should treat the existing code as a reference implementation to port line-by-line back to the CONTEXT.md-locked layout, not as a pristine starting point.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain 1.85+ / Edition 2024 | All library code | ✓ (assumed Phase 0 set up `rust-toolchain.toml`) | 1.85 | Blocking — no fallback |
| `cargo` | All work | ✓ | bundled | — |
| C++ toolchain (g++/clang++) | `xtask regen-ad-fixtures` ONLY (D-15) | ✗ or ✓ — depends on maintainer machine | — | Commit pre-generated fixtures to `tests/fixtures/`; CI does not regenerate |
| `xcfun-master/external/upstream/taylor/` source files | Fixture regeneration | ✓ (vendored per D18) | sha: unknown — Phase 1 must capture `xcfun_version_git_sha` per D-16 | — |
| Native f64 libm (std feature) | `f64::{exp, log, sqrt, powf, cbrt, erf, asinh, atan, ...}` | ✓ on all Tier-1 Rust targets | platform-dependent | `libm` crate (feature-gated per D-14) |
| `proptest`, `rstest`, `criterion`, `bincode`, `serde_json`, `rand_xoshiro` | Test + bench infrastructure | Must be added to workspace Cargo.toml | per `Standard Stack` table | — |

**Missing dependencies with no fallback:** None — all runtime deps are Rust-native and already on crates.io.

**Missing dependencies with fallback:** C++ toolchain is only needed at fixture-regeneration time. Treating committed `tests/fixtures/*.bincode` as the canonical source preserves hermetic `cargo build`/`cargo test` without a C++ compiler.

---

## Common Pitfalls

### Pitfall P3 (from PITFALLS.md): CTaylor coefficient ordering corrupted on port

**What goes wrong:** A flat triple-loop `CTaylor::mul` rewrite reorders cross-multiplication terms in `x_lo*y_hi` vs `x_hi*y_lo`, producing 1-2 ULP drift per multiply — unsurvivable over a GGA body with ~200 multiplies.

**Why it happens:** The recursion structure *is* the summation order. `ctaylor_rec::multo` writes `dst[high]` BEFORE `dst[low]` is updated; a Rust rewrite using `for i in 0..N` iterates the opposite direction, scrambling the accumulation.

**How to avoid:**
1. Port `ctaylor_rec<T, 0..=2>` base cases verbatim (ctaylor.hpp:86-152).
2. For N ∈ {3, 4, 5, 6, 7}, write a const-generic recursion or a macro that expands to the identical 3-call pattern.
3. Use `split_at_mut(HALF)` to satisfy Rust's borrow checker without allocation — NOT `.to_vec()`.
4. Unit-test against golden fixtures at `f64::to_bits` fidelity for orders 0..=3 (AD-05).

**Warning signs:**
- Parity fails at order >= 2, passes at order 0-1.
- Failure localised to bit-index 3 (= `VAR0 | VAR1`) across multiple test inputs.
- Fix candidate introduces `unsafe { std::ptr::*_unaligned }` or similar — indicates the borrow-checker issue, not the algebra.

### Pitfall P9: `*_expand` coefficient layout miscopied

**What goes wrong:** Rewriting `pow_expand` as `t[i] = choose(a, i) * x0.powf(a - (i as f64))` is mathematically equivalent but uses different multiplications — 1-2 ULP drift compounds through `ctaylor_rec::compose`.

**Why it happens:** Rust code reviewers see `t[i] = t[i-1] * x0inv * (a - i + 1) / i` and think "clarify — this is just a binomial coefficient recurrence". Rewriting changes the order of division by `i`.

**How to avoid:**
1. D-10 mandate: each `*_expand` module has a header comment with `// tmath.hpp:154-161: pow_expand` and the C++ source block pasted in a comment.
2. Golden-coefficient tests at 3 inputs × 7 orders per function = 168 records minimum.
3. Code review rejects any "simplification" that doesn't come with a passing fixture at `to_bits` identity.

**Warning signs:**
- Functionals using `pow(rho, 4/3)` fail; functionals using `pow(rho, integer)` pass.
- `log_expand` alternating-sign coefficient at odd-index positions drifts.

### Pitfall P10: `*_expand` asserts compiled out in release

**What goes wrong:** `debug_assert!(x0 > 0)` is compiled out in release builds; a NaN from `log(-1e-16)` propagates silently through the Taylor chain.

**How to avoid (D-11):**
1. All `*_expand` preconditions use `assert!`, not `debug_assert!`.
2. CI runs `cargo test --release` AND `cargo test` to catch any `debug_assert!`-gated test.
3. The `xcfun-core::DensVars::build` contract (Phase 2) returns `Result<Self, XcError>` — the assert path is a last-resort guard.

**Warning signs:**
- Debug tests pass; release tests fail.
- Output has NaN where C++ has a finite value OR has aborted.

### Pitfall P1 (reassociation / FMA)

**What goes wrong:** LLVM emits `vfmadd` for `a*b + c` patterns, especially inside const-generic monomorphisations of the mul recursion. Single-rounded FMA differs from `(a*b) + c` by 1 ULP per op.

**How to avoid:**
1. CONTEXT.md D-21: `.cargo/config.toml` sets `-Cllvm-args=-fp-contract=off` for the release profile.
2. CI spot-checks `CTaylor::<f64, 3>::mul_assign` disassembly with `cargo asm` and greps for `vfmadd` / `fmadd` — fails if any match (D-20).
3. Explicit `let` intermediate variables (D-08) force the compiler's hand on sequencing.

**Warning signs:**
- Parity fails at exactly 1-2 ULP drift uniformly.
- `cargo bench` shows a 5-10% improvement after a refactor — FMA got emitted.

### Pitfall: heap allocation in composed functions

**What goes wrong:** The existing `src/math.rs:30-37` uses `vec![0.0; nvar + 1]` in `ctaylor_exp`. Violates D-01 (stack-only). Also a 4-8 μs allocation on the per-point hot path is unacceptable for Phase 3 GGA functionals calling `exp`/`log` 5-10 times.

**How to avoid:**
- Use `let mut scratch = [0.0_f64; 8]; let slice = &mut scratch[..=N];` — the `8 = N_MAX + 1` upper bound is a compile-time constant.
- Alternatively, use const-generic `let scratch: [f64; 1+N_PLUS_1]` where `N_PLUS_1 = N + 1` — requires `generic_const_exprs` (nightly), so prefer the `[0.0; 8]` + slice approach.

### Pitfall: borrow-checker forcing heap in `multo_skipconst`

**What goes wrong:** The existing `compose.rs:83` uses `let lo_copy: Vec<f64> = dst[..half].to_vec();` to break aliasing. Violates D-01; unacceptable.

**How to avoid:**
- `let (lo, hi) = dst.split_at_mut(half);` — provides non-overlapping mutable borrows for free.
- If the recursion still hits the borrow checker because `multo_skipconst` needs to both read and write `dst[..half]`, promote the cross-term computation to a stack-allocated scratch: `let mut scratch: [f64; 64] = ...` (max `1 << 6 = 64` for N=6; largest aliasing chunk is `HALF = 1 << (N-1)` for N=7 → scratch of size 64).

### Pitfall: fixture version drift vs `xcfun-master`

**What goes wrong:** Fixtures are committed in `tests/fixtures/*.bincode`; the `xcfun-master/` vendored sources are also committed. After a future `xcfun-master` content-hash bump (e.g., upstream bug fix), the fixtures remain stale — CI passes (fixtures self-consistent) while the Rust implementation drifts from the vendored C++.

**How to avoid:**
1. D-16: Fixture JSON manifest includes `xcfun_version_git_sha`.
2. CI check: compute `sha256sum xcfun-master/external/upstream/taylor/*.hpp` and assert it matches `fixtures.json::xcfun_version_git_sha`.
3. Any delta triggers `xtask regen-ad-fixtures` — which requires a C++ toolchain, so it's a maintainer-gated action.

---

## Code Examples

### Example 1: CTaylor mul base case N=2 (verbatim port of ctaylor.hpp:131-135)

```rust
// Source: xcfun-master/external/upstream/taylor/ctaylor.hpp:131-135
// C++:
//   static void multo(T * dst, const T * y) {
//     dst[3] = dst[0] * y[3] + dst[3] * y[0] + dst[1] * y[2] + dst[2] * y[1];
//     dst[2] = dst[0] * y[2] + dst[2] * y[0];
//     dst[1] = dst[0] * y[1] + dst[1] * y[0];
//     dst[0] = dst[0] * y[0];
//   }

#[inline(never)]  // D-20: disallow cross-procedure reassociation for this hot op
fn multo_n2(dst: &mut [f64; 4], y: &[f64; 4]) {
    // Order of writes matters: dst[3] before dst[2] before dst[1] before dst[0]
    // because each lower index reads only dst[0] (preserved until the final line).
    let t3 = dst[0] * y[3] + dst[3] * y[0] + dst[1] * y[2] + dst[2] * y[1];
    dst[3] = t3;
    let t2 = dst[0] * y[2] + dst[2] * y[0];
    dst[2] = t2;
    let t1 = dst[0] * y[1] + dst[1] * y[0];
    dst[1] = t1;
    dst[0] = dst[0] * y[0];
}
```

### Example 2: `pow_expand` (verbatim port of tmath.hpp:154-161)

```rust
// Source: xcfun-master/external/upstream/taylor/tmath.hpp:154-161
// Identity: (x0 + h)^a = x0^a * (1 + h/x0)^a
//   t[0]   = x0^a
//   t[i]   = t[i-1] * x0inv * (a - i + 1) / i
// Precondition: x0 > 0 (asserted -- D-11 / P10)

pub fn pow_expand(t: &mut [f64], x0: f64, a: f64) {
    assert!(x0 > 0.0, "pow_expand: x0 = {x0}, requires x0 > 0 (not real analytic at x <= 0)");
    t[0] = x0.powf(a);
    let x0inv = 1.0 / x0;
    for i in 1..t.len() {
        let i_f = i as f64;
        // C++: t[i] = t[i-1] * x0inv * (a - i + 1) / i
        // Preserve exact operator order from tmath.hpp:160
        let step = x0inv * (a - i_f + 1.0) / i_f;
        t[i] = t[i - 1] * step;
    }
}
```

### Example 3: Composed `ctaylor_exp` (verbatim of ctaylor_math.hpp:71-81, D-01-compliant)

```rust
// Source: xcfun-master/external/upstream/taylor/ctaylor_math.hpp:71-81
// NB: zero-heap form — slice of stack array instead of `vec![]`.

pub fn ctaylor_exp<const N: usize>(t: CTaylor<f64, N>) -> CTaylor<f64, N>
where
    Bound: ValidN<N>,
{
    let mut scratch = [0.0_f64; 8];                 // N_MAX + 1 = 8 (D-01)
    let slice: &mut [f64] = &mut scratch[..=N];     // use only 0..=N entries
    crate::expand::exp::exp_expand(slice, t.c[0]);
    let mut result = CTaylor::ZERO;
    crate::ctaylor_rec::compose::compose(&mut result.c, &t.c, slice);
    result
}
```

### Example 4: Property test for Leibniz rule (proptest 1.11)

```rust
// crate: xcfun-ad, file: tests/props_leibniz.rs

use proptest::prelude::*;
use xcfun_ad::{CTaylor, VAR0, CNST};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,   // D-18: >= 10k
        .. ProptestConfig::default()
    })]

    #[test]
    fn leibniz_product_rule_var0(
        // Constrain to non-pathological f64 values
        a_cnst in -100.0_f64..100.0_f64,
        a_var0 in -100.0_f64..100.0_f64,
        b_cnst in -100.0_f64..100.0_f64,
        b_var0 in -100.0_f64..100.0_f64,
    ) {
        // Build a = a_cnst + a_var0 * x0, b = b_cnst + b_var0 * x0 in CTaylor<f64, 1>
        let a = CTaylor::<f64, 1> { c: [a_cnst, a_var0] };
        let b = CTaylor::<f64, 1> { c: [b_cnst, b_var0] };
        let ab = a * b;
        let expected_var0 = a_cnst * b_var0 + a_var0 * b_cnst;   // product rule
        // 2 ULP tolerance -- bare multiplication, no compose
        let diff = (ab.c[VAR0] - expected_var0).abs();
        let tol = 2.0 * f64::EPSILON * ab.c[VAR0].abs().max(1.0);
        prop_assert!(diff <= tol, "|{} - {}| > tol = {tol}", ab.c[VAR0], expected_var0);
    }
}
```

### Example 5: Golden-fixture consumer for mul

```rust
// crate: xcfun-ad, file: tests/golden_mul.rs
// Reads tests/fixtures/mul.bincode; asserts bit-identical on orders 0..=3.

use xcfun_ad::{CTaylor, for_tests::raw_coeffs};

#[derive(serde::Deserialize)]
struct FixtureRecord {
    op: String,       // e.g., "mul"
    n_var: u8,        // 0..=3 for the to_bits-bar
    inputs: Vec<f64>, // flat layout: first (1 << n_var) are a.c, next are b.c
    coeffs: Vec<f64>, // expected (1 << n_var) coeffs of a*b
}

#[test]
fn mul_matches_cpp_reference_to_bits_n_le_3() {
    let bytes = include_bytes!("fixtures/mul.bincode");
    let records: Vec<FixtureRecord> = bincode::deserialize(bytes).unwrap();
    for rec in records.iter().filter(|r| r.op == "mul" && r.n_var <= 3) {
        match rec.n_var {
            0 => check_mul::<1>(rec),
            1 => check_mul::<2>(rec),
            2 => check_mul::<4>(rec),
            3 => check_mul::<8>(rec),
            _ => unreachable!(),
        }
    }
}

fn check_mul<const SIZE: usize>(rec: &FixtureRecord) {
    let (a_in, rest) = rec.inputs.split_at(SIZE);
    let b_in = &rest[..SIZE];
    // For this example, N = log2(SIZE). In real code use ValidN<N>.
    let a = CTaylor::<f64, /*N*/>::from_coeffs(a_in.try_into().unwrap());
    let b = CTaylor::<f64, /*N*/>::from_coeffs(b_in.try_into().unwrap());
    let c = a * b;
    for (i, (got, expected)) in raw_coeffs(&c).iter().zip(&rec.coeffs).enumerate() {
        assert_eq!(
            got.to_bits(), expected.to_bits(),
            "mul[{}] to_bits mismatch at coeff {i}: got {got}, expected {expected}",
            rec.op,
        );
    }
}
```

### Example 6: `.cargo/config.toml` for `-fp-contract=off` (D-21)

```toml
# .cargo/config.toml (repository root or ~/.cargo/ -- repo local preferred)

[build]
rustflags = [
    "-Cllvm-args=-fp-contract=off",   # D-21: forbid compiler FMA fusion
]

# Alternative profile-scoped form, more robust:
[target.'cfg(all())']
rustflags = [
    "-Cllvm-args=-fp-contract=off",
]
```

**Verification:**
```bash
cargo rustc --release -p xcfun-ad -- --emit=asm
cd target/release/deps
rg '(vfmadd|fmadd)\s' xcfun_ad-*.s                # must return no matches (D-20)
```

---

## Validation Architecture

(Phase 1 validation architecture — downstream Nyquist validation reads this. Maps directly to VALIDATION.md template fields.)

### Determinism tier

Phase 1 produces deterministic output modulo **f64 arithmetic identity**. On orders 0..=3 the primary validation signal is **byte-identical `f64::to_bits` equality** against a C++ reference driver. On orders 4..=6, libm calls (used in `*_expand` constant-term seeding, e.g., `t[0] = x0.powf(a)`) may introduce ULP-level drift between platforms — tolerance widens to **1e-13 relative error** per coefficient at those higher orders.

- **Tier label:** `byte-deterministic (orders 0..=3) | ulp-deterministic (orders 4..=6)`
- **Non-determinism sources authorised:** platform libm scalar calls (`f64::exp`, `f64::powf`, ...) — documented per-op ULP drift below.
- **Non-determinism sources forbidden:** thread scheduling, FMA auto-emission, heap allocation timing, unspecified iteration order.

### Sampling strategy

Three-layer validation with explicit record counts.

**(a) Per-`*_expand` golden coefficient tests:** 3 inputs × 7 orders × 8 functions = **168 records minimum**.

- 8 target functions: `inv, exp, log, pow, sqrt, cbrt, gauss, erf` (D-09).
- 3 representative inputs per function, chosen to exercise magnitude regimes:
  - small: `x0 = 0.1` (or `a = -0.9` for erf/gauss)
  - mid: `x0 = 1.0` (or `a = 0.0`)
  - large: `x0 = 10.0` (or `a = 2.5`)
- 7 orders: N ∈ {0, 1, 2, 3, 4, 5, 6}.
- `pow_expand` requires a representative `a` parameter sweep: for the 3 inputs, pair with `a ∈ {0.5, -1/3, 4/3}` (covers sqrt, inv-cbrt, LDA-exchange characteristic).
- Record shape: `{ op, n_var, inputs: [x0] (or [x0, a]), coeffs: [f64; N+1] }`.

**(b) Per-`CTaylor` operation:** 50 random seeds × 5 N values × 6 ops = **1500 records**.

- 50 seeds from `rand_xoshiro::Xoshiro256PlusPlus` with initial seed `0x1234abcd` (fixture-stable).
- Each seed emits a "reasonable" input drawn from `uniform(-10, 10)` for every coefficient position.
- 5 N values: {0, 1, 2, 3, 4}. (N=5, 6 add ~50% records each but increase .bincode size — trimmed per D-17's 1 MB budget.)
- 6 ops: `add, sub, neg, mul, mul_assign, div`.
- Record shape: `{ op, n_var, inputs: Vec<f64>, coeffs: Vec<f64> }`.
- For `div`, the divisor is sampled from `uniform(0.5, 2.0)` (avoid near-zero constant terms).

**(c) Property tests:** >= 10 000 iterations per property × 9 properties = **90 000 iterations minimum** per CI run.

The 9 properties (D-18):
1. Commutativity: `a + b == b + a`, `a * b == b * a` (bit-identical).
2. Associativity (algebraic, not floating): `(a + b) + c ~= a + (b + c)` within 4 ULP on `c[CNST]`.
3. Distributivity: `a * (b + c) ~= a*b + a*c` within 4 ULP.
4. Ring axioms: `a - a == CTaylor::ZERO`; `a * CTaylor::ONE == a`.
5. `exp(x) * exp(-x) ~= 1.0` within 1e-13 rel err.
6. `log(exp(x)) ~= x` within 1e-13 rel err.
7. `sqrt(x)^2 ~= x` within 1e-13 rel err (x0 > 0).
8. `pow(x, a) * pow(x, -a) ~= 1.0` within 1e-13 rel err.
9. Leibniz on VAR0: `(a * b).c[VAR0] == a.c[CNST] * b.c[VAR0] + a.c[VAR0] * b.c[CNST]` (bit-identical).

Total fixture fingerprint: 168 + 1500 = **1668 records**. With 8 bytes per f64 and typical record = ~100 bytes serialised (bincode), total fixture size estimate: **~167 KB**. Safely inside the 1 MB budget (D-17).

### Oracle source

**C++ reference driver** compiled from `xcfun-master/external/upstream/taylor/{ctaylor.hpp, ctaylor_math.hpp, tmath.hpp}` emitting identical `f64` `to_bits` records.

- Driver: `xtask/assets/regen_ad_fixtures/driver.cpp` (~200 LOC).
- Build: `cc::Build::new().cpp(true).include("xcfun-master/external/upstream/taylor/").file("driver.cpp").compile("ad_fixture_driver")`.
- Driver emits records in the same format the Rust tests deserialise — `bincode::Vec<FixtureRecord>` + `fixtures.json` manifest.
- Committed to `crates/xcfun-ad/tests/fixtures/` (D-15).

**Version pin (D-16):** `fixtures.json` records `xcfun_version_git_sha = sha256(xcfun-master/external/upstream/taylor/*.hpp)` at generation time. CI validates this hash matches the current vendored source; any mismatch blocks CI until fixtures are regenerated (maintainer-gated).

### Tolerance budget

| Operation chain | Per-op ULP (design/07 §6) | N=3 expected | N=6 expected | Phase 1 gate |
|-----------------|---------------------------|--------------|--------------|--------------|
| `Add<CTaylor>` / `Sub<CTaylor>` elementwise | 0.5 | 0.5 ULP | 0.5 ULP | `to_bits` identity |
| `Neg` | 0 | 0 | 0 | `to_bits` identity |
| `Mul<CTaylor>` (ctaylor_rec::multo) | 0.5 per leaf op; recursion depth N | ~1.0 | ~2.0 | `to_bits` identity for N<=3; `<= 1e-13` rel err for N>=4 |
| `Mul<f64>` scalar | 0.5 | 0.5 | 0.5 | `to_bits` identity |
| `Div<CTaylor>` (inv_expand + compose + mul) | 1 (libm) + 0.5 (compose chain) | ~2 | ~4 | `<= 1e-13` rel err |
| `*_expand` constant-term seeding (`t[0] = libm(x0)`) | 1 (libm) | 1 | 1 | `to_bits` identity for exact libm calls; `<= 1e-13` for differing libms |
| Composed `exp`, `log`, `pow` | 1 (libm at seeding) + 0.5 (compose chain depth N) | ~2 | ~4 | `<= 1e-13` rel err |
| Composed `erf`, `asinh` | 1.5 (derived via integration of gauss) + 0.5 (compose) | ~2.5 | ~5 | `<= 1e-13` rel err |

- **Fixture-level goal (AD-05):** `to_bits` identity on orders 0..=3 for `mul`, `add`, `sub`, `neg` and the 8 `*_expand` tables (the libm seeding is a single call, identical on same-platform runs).
- **Property-test goal (AD-06):** <= 1e-13 rel err on roundtrips; bit-identical on Leibniz and ring axioms with no libm.
- **Cross-platform libm drift flag:** Phase 1 CI runs on a single canonical platform (Linux x86_64 glibc, per STACK.md row "libm pinning"). Deferred: formal multi-platform validation matrix.

### Coverage proof

- **CTaylor::mul recursion branches:** for N ∈ {0, 1, 2, 3, 4, 5, 6}, every call to `ctaylor_rec<N>::multo` exercises all 3 sub-calls; the Wave-0 base cases (N=0, 1, 2) are covered by explicit unit tests. For N>=3, fixture records at N=3, 4, 5 exercise the general recursion. N=6 record coverage in Wave-2.
- **`*_expand` scalar branches:** every function exercised at 3 inputs (small/mid/large magnitude). `pow_expand` additionally at 3 `a` values. `atan_expand` and `erf_expand` exercise `tfuns::{compose, integrate}` internals.
- **Composed function branches:** each of the 9 D-12 composed functions (reciprocal, sqrt, exp, log, pow, powi, erf, asinh, atan) exercised at 3 inputs per order. `powi` additionally exercised at `n ∈ {-2, -1, 0, 1, 2, 5, 10}` to hit each branch of `ctaylor_math.hpp:165-178`.
- **`ValidN<N>` compile-time bound:** a compile-fail test using `trybuild` confirms `CTaylor::<f64, 8>` fails to compile. (Optional; can be deferred to Phase 2 if `trybuild` setup creates Phase 1 dependency pressure.)

### Regression protocol

- **Fixtures regenerated ONLY via `cargo xtask regen-ad-fixtures`** — requires C++ toolchain on maintainer's machine. CI runs on committed fixtures, never regenerates.
- **Fixture version pinned to `xcfun-master` content hash:** `fixtures.json::xcfun_version_git_sha = sha256(xcfun-master/external/upstream/taylor/{ctaylor,ctaylor_math,tmath}.hpp)`. CI checks this hash matches; drift blocks merge.
- **Any cubecl / rustc / libm bump re-runs Tier 1 (the fixture test) first** before other tiers. Tier 1 failure means algorithmic drift, not backend drift; blocks all downstream work.
- **Regeneration gate:** `xtask regen-ad-fixtures --check` computes a new fixture set in `/tmp/` and diffs against committed ones — diff output is a triage tool for maintainers when upstream changes.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| AD-01 | `CTaylor<T, 0..=7>` with `[T; 1 << N]` storage | unit + compile-pass | `cargo test -p xcfun-ad --test ctaylor_struct` | ❌ Wave 0 |
| AD-01 | `N=8` fails to compile | compile-fail | `cargo test -p xcfun-ad --test ctaylor_validn -- --ignored` (via `trybuild`) | ❌ Wave 0 (optional) |
| AD-02 | `Num` trait supplies all ops for `f64` | unit | `cargo test -p xcfun-ad num::f64_impl` | ❌ Wave 0 |
| AD-02 | `Num` trait blanket impl for `CTaylor<f64, N>` | unit | `cargo test -p xcfun-ad num::ctaylor_impl` | ❌ Wave 0 |
| AD-03 | `CTaylor::mul` verbatim recursion | golden (`to_bits`) | `cargo test -p xcfun-ad --test golden_mul -- --include-ignored` | ❌ Wave 1 |
| AD-04 | Every `*_expand` byte-equivalent | golden (`to_bits`) | `cargo test -p xcfun-ad --test golden_expand` | ❌ Wave 1 |
| AD-05 | Orders 0..=3 `to_bits` match C++ | golden | `cargo test -p xcfun-ad --test golden_mul && cargo test -p xcfun-ad --test golden_composed` | ❌ Wave 2 |
| AD-06 | Property tests >= 10k iters each | proptest | `PROPTEST_CASES=10000 cargo test -p xcfun-ad --test props_ring --test props_leibniz --test props_roundtrips` | ❌ Wave 2 |

### Test Framework

| Property | Value |
|----------|-------|
| Framework | rust built-in `#[test]` + `proptest 1.11.0` + `rstest 0.26.1` + `criterion 0.8.2` |
| Config file | `Cargo.toml` `[dev-dependencies]` + `[[bench]]` stanzas |
| Quick run command | `cargo test -p xcfun-ad --lib` (unit tests only; ~1 s) |
| Full suite command | `cargo test -p xcfun-ad` (unit + integration + golden + proptest; ~30 s with `PROPTEST_CASES=10000`) |
| Bench command | `cargo bench -p xcfun-ad` (D-19 baseline) |

### Sampling Rate

- **Per task commit:** `cargo test -p xcfun-ad --lib` — unit tests only.
- **Per wave merge:** `cargo test -p xcfun-ad` — full suite including golden fixtures and property tests at `PROPTEST_CASES >= 10_000`.
- **Phase gate:** Full suite green + `cargo bench -p xcfun-ad` baseline recorded + `cargo asm` spot-check asserts no FMA in `CTaylor::mul` object file.

### Wave 0 Gaps

Infrastructure Wave 0 must land before implementation can begin:

- [ ] `crates/xcfun-ad/tests/fixtures/` directory + `.gitkeep` (will receive `.bincode` files in later waves)
- [ ] `xtask/src/bin/regen_ad_fixtures.rs` + C++ driver skeleton
- [ ] `crates/xcfun-ad/benches/mul_bench.rs` + `compose_bench.rs` criterion harness
- [ ] `crates/xcfun-ad/tests/props_*.rs` proptest harness scaffolding
- [ ] `.cargo/config.toml` with `-Cllvm-args=-fp-contract=off` (D-21) — **verify Phase 0 actually delivered this**; add if missing
- [ ] `trybuild` dev-dep + `compile-fail/*.rs` for `ValidN<N>` bound check (optional — can defer to Phase 2)

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Boxed polynomial types (`Box<[f64]>` / `Vec<f64>`) | Stack-allocated `[T; 1 << N]` with `ValidN<N>` | Rust stable const generics (1.59+) + `1 << N` const expressions (1.79+) | No heap, deterministic allocation, GPU-friendly |
| `num-traits::Float` for numeric abstraction | Custom `Num` trait | This project — D2/D-05 | Avoids `is_nan`/`classify`/`floor` semantics meaningless on polynomials |
| `debug_assert!` on numeric preconditions | `assert!` (kept in release) | PITFALLS.md P10 response | Silent NaN elimination; active precondition checking |
| Third-party AD crates (`autodiff`, `hyperdual`, `ad`, `num-dual`) | In-house CTaylor verbatim port | Project D1 | Algorithmic identity with xcfun C++ — only path to 1e-12 parity |
| `lazy_static` / `once_cell` for static tables | `const` items (std::sync::LazyLock available in 1.85) | Rust 1.85 stable | No runtime init, `.rodata`-resident constants |
| `f64::mul_add` for ostensible performance | Plain `a*b + c` with explicit `let` | PITFALLS.md P1 response | Matches C++ accumulation order; preserves parity |

**Deprecated / outdated:**
- `bincode 1.x` API is preferred over `2.x` for stability; `2.x` has significant API churn. (`[VERIFIED: bincode release notes]`)
- `approx 0.6.0-rcN` — RC only; stick with `0.5.1`. Switching on an accuracy contract is risky.

---

## Project Constraints (from CLAUDE.md)

Mandatory for every Phase 1 artifact — any deviation MUST be flagged at plan-time:

- **MSRV:** Rust 1.85 (Edition 2024). No nightly features.
- **Accuracy contract:** every `xcfun-ad` output must match C++ xcfun within 1e-12 relative error; orders 0..=3 stricter — `f64::to_bits` identity.
- **Dependencies (workspace-pinned):**
  - `thiserror = "2.0.18"` — library errors only.
  - `bitflags = "2.11"` — phase 2+ scope; not needed in Phase 1.
  - `anyhow = "1.0"` — app-boundary only (`xtask`, `validation`); MUST NOT appear in `xcfun-ad/Cargo.toml`.
  - `libm = "0.2"` — optional, feature-gated.
- **`xcfun-ad` rules:**
  - `#![forbid(unsafe_code)]` at crate root.
  - Zero runtime dependencies on default feature (`libm` is optional).
  - Stack-only storage — no `Box`, no `Vec`, no heap on hot path (D-01).
- **Workflow:** every change goes through a GSD command (research -> plan -> execute). Direct edits to `crates/xcfun-ad/**` only via `/gsd-execute-phase` wave tasks.
- **Commit:** atomic per task; follows Conventional Commits; no batched cross-task commits.

---

## Assumptions Log

Claims tagged `[ASSUMED]` in this research — planner/discuss-phase should confirm before locking:

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `bincode 1.3.3` (v1 API) is preferred over `bincode 2.x` for fixture format | §Standard Stack | Low — either version works; v2 API churn adds rebase cost on future crate updates. Rebuildable if wrong. |
| A2 | Criterion bench harness Phase 1 baseline records are append-only; no PR-comparison gate at Phase 1 | §Validation Architecture / Tolerance budget | Low — CONTEXT.md D-19 explicit: "no regression gate in v1" |
| A3 | `postcard` and `ciborium` fixture formats are rejected unless `bincode` imposes specific pain | §Alternatives | Low — decision is reversible at Wave 0 |
| A4 | `asin_expand` / `acos_expand` upstream (tmath.hpp:290, :313) have a typo (`t[0] = asinh(a)` instead of `asin(a)` / `acos(a)`); Phase 1 ports verbatim | §Operation Inventory | Medium — if intentional (not a bug), Phase 1 port is correct by construction. If a bug, Phase 1 records the bug in fixtures; downstream (LDA/GGA phases) may see surprising output. **Action: Phase 1 module header documents this as "upstream appears to have typo at line X — verify against reference"; no Phase 1 fix.** |
| A5 | `cbrt_expand` and composed `cbrt` are in scope for Phase 1 even though AD-02 lists only 9 composed functions | §Operation Inventory ("AD-02 scope note") | Medium — Phase 2 LDA functionals depend on `cbrt` (r_s, n_m13 calculations). Omitting from Phase 1 would block Phase 2. Recommend include. |
| A6 | `sin`, `cos`, `asin`, `acos` composed functions are OUT of Phase 1 scope (not in AD-02, not in any downstream functional body under Phase 2-5) | §Operation Inventory | Low — if needed later, `*_expand` scalars are ported (D-09 mentions only 8: `inv, exp, log, pow, sqrt, cbrt, erf, gauss`). Add sin/cos _expand if a Phase 2+ functional uses them. |
| A7 | `sqrtx_asinh_sqrtx` is out of Phase 1 scope | §Operation Inventory | Medium — This function (ctaylor_math.hpp:275-325) uses a Pade-[8,8] approximation; it's complex enough to warrant its own sub-task when needed. No Phase 1-through-6 requirement listed. Phase 2 verification may surface it if B97, M06, or SCAN consume it. |
| A8 | Fixture records target 1668 total records ≈ 167 KB serialised | §Validation Architecture / Sampling | Low — budget target (1 MB, D-17) gives 6× headroom. |
| A9 | Existing `crates/xcfun-ad/src/*.rs` (2912 LOC) will be rewritten rather than evolved in place — semantics of `N` (array size vs nvar) diverge from CONTEXT.md D-01 | §Architecture Patterns / Recommended Project Structure | Medium — rewrite vs evolve affects Wave 0 task sizing. Recommend: treat existing code as a reference/skeleton during port; preserve no API surface that contradicts CONTEXT.md. |
| A10 | CI does not run on macOS or Windows for Phase 1 (libm drift deferred) | §Validation Architecture / Tolerance budget | Low — CONTEXT.md / design doc 07 §2 mandates Linux x86_64 glibc as the canonical platform. macOS/Windows added later. |

**Items with NO assumption (all verified):** version pins (STACK.md), architectural layout (design/02, design/10), recursion structure (ctaylor.hpp:41-152 direct read), pitfall catalogue (PITFALLS.md direct read), phase requirement IDs (REQUIREMENTS.md direct read).

---

## Open Questions (RESOLVED)

> B6 revision: all 5 questions RESOLVED during planner revision pass (2026-04-19).

1. **Should `cbrt` be included in Phase 1 even though AD-02 lists only 9 composed functions?**
   - What we know: D-09 lists `cbrt_expand` in the `*_expand` port set. Phase 2 LDA bodies will consume it (r_s formula).
   - What's unclear: whether the `Num` trait (AD-02) should carry a `cbrt` method in Phase 1 or only in Phase 2.
   - Recommendation: add `cbrt` to the `Num` trait in Phase 1 for consistency; the `*_expand` is already in scope. Label with comment noting it's not an AD-02-mandated method but a Phase-2 prep.
   - **RESOLVED: No.** AD-02 (CONTEXT.md D-04) locks the 14-method Num trait: Add/Sub/Mul/Div/Neg + reciprocal, sqrt, exp, log, pow, powi, erf, asinh, atan + ZERO/ONE. `cbrt` stays in `expand/cbrt.rs` as a scalar-only helper for Phase 2+ consumption. No `Num::cbrt` method in Phase 1.

2. **Does the existing Rust code in `crates/xcfun-ad/src/` get rewritten, or ported in place?**
   - What we know: 2912 LOC exists. `N` semantics differ from CONTEXT.md D-01.
   - What's unclear: whether the planner should treat existing code as the Phase 1 starting point (evolve) or as reference material (rewrite).
   - Recommendation: **rewrite**. The `N = array size` vs `N = nvar` divergence is load-bearing — attempting to evolve would leave confusing naming throughout the crate.
   - **RESOLVED: Rewrite.** CONTEXT.md D-01 `N`-semantics divergence is load-bearing; evolving in place would propagate confusing naming throughout the crate. Plans 01-02 action blocks explicitly say "complete rewrite of existing file".

3. **What's the `N_MAX = 7` justification?**
   - What we know: CONTEXT.md D-01 states `N in 0..=7`. REQUIREMENTS.md MODE-03 says `Contracted` supports orders 0..=6.
   - What's unclear: why 7 and not 6? design/02-data-structures.md:42 says "`XCFUN_MAX_ORDER = 6`, plus one additional bit for `XC_CONTRACTED` handling -> 7 bits max".
   - Recommendation: accept the explanation; port `N` up to 7 in Phase 1 for forward compatibility.
   - **RESOLVED: XCFUN_MAX_ORDER=6 plus one XC_CONTRACTED bit per design/02 §1.** Accept `N ∈ 0..=7` for Phase 1; the 7th bit covers the `XC_CONTRACTED` handling explicitly called out in docs/design/02-data-structures.md:42.

4. **Should `f64::mul_add` be explicitly lint-banned in `xcfun-ad`?**
   - What we know: D-20 says "functional-body `mul_add` lint is introduced in Phase 2".
   - What's unclear: whether `xcfun-ad/src/**` can use `mul_add` at all (answer is no, but should there be a Phase-1 lint or just a CI grep?).
   - Recommendation: Phase 1 CI asserts zero `fma`/`vfmadd` in the release object file (D-20). A `clippy::`-level lint ban can be deferred.
   - **RESOLVED: Phase 1 uses `cargo asm` grep (D-20); Phase 2 adds a clippy-style ban.** No `mul_add` in `crates/xcfun-ad/src/ctaylor_rec/**` is enforced by grep in Plan 03 acceptance criteria; a full clippy lint is deferred to Phase 2 functional-body porting. The VALIDATION.md row `01-03-03 check-no-fma` was dropped during revision — the `cargo asm` grep becomes a CI job added at Phase 2 scope (see VALIDATION.md foot note).

5. **Does `xcfun-ad` depend on `std` or `core`?**
   - What we know: D-14 says `default = ["std"]`. `core::ops` is fine; `std::f64::*` is needed for math on the default feature.
   - What's unclear: should intrinsic math routes through `num_traits::FloatCore` or directly to `std::f64::*` methods?
   - Recommendation: direct `std::f64::*` — CONTEXT.md D-05 rejects `num_traits::Float`.
   - **RESOLVED: Direct `std::f64::*` when `feature="std"`; `libm::*` when `feature="libm"`; CONTEXT.md D-05 rejects `num_traits::Float`.** Plans 02/05 use the exact `cfg(feature = "std")` / `cfg(all(not(feature = "std"), feature = "libm"))` dispatch pattern throughout.

---

## Sources

### Primary (HIGH confidence)

- **`xcfun-master/external/upstream/taylor/ctaylor.hpp`** — base class, recursion structure (lines 41-65 general, 86-152 base cases, 154-357 struct methods, 360-510 free operators) — read directly.
- **`xcfun-master/external/upstream/taylor/ctaylor_math.hpp`** — composed elementary functions (7-28 operator/, 71-81 exp, 105-115 log, 120-131 pow, 134-145 sqrt, 148-159 cbrt, 165-178 powi, 181-192 atan, 195-206 erf, 257-268 asinh, 275-325 sqrtx_asinh_sqrtx) — read directly.
- **`xcfun-master/external/upstream/taylor/tmath.hpp`** — scalar Taylor expansions (124-129 inv, 132-139 exp, 142-151 log, 154-161 pow, 164-170 sqrt, 172-178 cbrt, 180-198 atan, 204-215 gauss, 218-225 erf, 227-241 sin, 243-257 cos, 260-274 asinh, 277-291 asin, 293-297 acosh, 299-314 acos, 36-121 tfuns helpers) — read directly.
- **`docs/design/02-data-structures.md` §1** — `CTaylor<T, N>` layout; `Num` trait definition.
- **`docs/design/07-accuracy-strategy.md` §2, §3, §6** — algorithmic-identity rule; tolerance budget per operation.
- **`docs/design/10-build-and-dependencies.md` §3.1** — `xcfun-ad/Cargo.toml` feature structure.
- **`docs/design/12-design-decisions.md` D1, D2, D4** — algorithmic identity, custom `Num`, f64-only.
- **`.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`** — 22 locked decisions.
- **`.planning/REQUIREMENTS.md` AD-01..06** — phase acceptance bar.
- **`.planning/research/PITFALLS.md` P1, P3, P9, P10, P11** — rounding-drift pitfalls directly relevant to Phase 1.
- **`.planning/research/STACK.md`** — version pins (`proptest 1.11.0`, `approx 0.5.1`, `rstest 0.26.1`, `criterion 0.8.2`, `rand_xoshiro 0.8.0`).
- **`.planning/research/SUMMARY.md`** — integrated risk table; phase-ordering rationale.
- **`CLAUDE.md`** — project tech-stack pins.

### Secondary (HIGH-MEDIUM confidence)

- **`crates/xcfun-ad/src/*.rs`** (existing 2912 LOC) — reference implementation currently in the repo, with `N = array size` drift from CONTEXT.md.
- **`Cargo.toml` (workspace root)** — existing dependency declarations.

### Tertiary (LOW confidence / awaiting empirical confirmation)

- **`bincode` version choice (1.x vs 2.x)** — documented as `[ASSUMED]`. Flag at plan-time.
- **Wgpu libm empirical drift** — not Phase 1 scope; flagged in design/07 §6.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all pins verified via `.planning/research/STACK.md` (2026-04-18 cross-check).
- Architecture: HIGH — C++ source directly inspected; existing scaffolding inspected; CONTEXT.md decisions locked.
- Operation inventory: HIGH — every `*_expand` mapped to exact tmath.hpp lines; every composed function mapped to ctaylor_math.hpp lines.
- Pitfalls: HIGH — derived from PITFALLS.md with direct C++ line references.
- Validation architecture: HIGH — record counts derived from `<validation_architecture_requirement>` block verbatim.
- Fixture format (`bincode` choice): MEDIUM — not locked at CONTEXT.md level; flagged as A1 assumption.

**Research date:** 2026-04-19
**Valid until:** 2026-05-19 (30 days — stack is stable, C++ sources vendored and unchanging, design decisions locked).
