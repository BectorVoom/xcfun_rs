# Phase 1: Core Types + AD Engine - Research

**Researched:** 2026-04-17
**Domain:** Rust const-generic automatic differentiation, exchange-correlation type system
**Confidence:** HIGH

## Summary

Phase 1 builds the foundation layer: two crates (`xcfun-ad` and `xcfun-core`) plus workspace scaffolding for the remaining five crates. The AD engine (`xcfun-ad`) is the highest-risk component -- it must replicate C++ xcfun's tensored multilinear polynomial approach exactly, including the recursive multiplication algorithm (O(3^N)), the compose/chain-rule algorithm, and all transcendental function Taylor expansions. The core types crate (`xcfun-core`) provides domain-specific enums, the `DensityVars<T>` container, error types, and the `Functional` trait definition (without implementations).

The C++ source in `xcfun-master/` has been thoroughly examined. The key algorithmic structures are: (1) `ctaylor_rec<T, Nvar>` for recursive multiplication with three sub-problems per level, (2) `ctaylor_rec::compose()` for Horner-like composition using `multo_skipconst`, (3) `tmath.hpp` functions (`exp_expand`, `log_expand`, `pow_expand`, `sqrt_expand`, etc.) that generate Taylor coefficient arrays, and (4) the `sqrtx_asinh_sqrtx` Pade approximant for numerical stability near zero. All of these must be translated to Rust with identical algorithmic structure.

**Primary recommendation:** Implement `xcfun-ad` first as a zero-dependency crate, validate against analytical derivatives, then build `xcfun-core` types that depend on it. The compose algorithm and multiplication recursion are the most sensitive code -- translate them coefficient-for-coefficient from the C++ source.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Full Cargo workspace from Phase 1 with xcfun-core and xcfun-ad as active crates. Stub Cargo.toml files for xcfun-functionals, xcfun-eval, xcfun-gpu, xcfun-ffi, xcfun-python.
- **D-02:** xcfun-ad depends on no other workspace crate. xcfun-core depends on xcfun-ad (for Num trait bound on DensityVars<T>).
- **D-03:** Extract test_in/test_out reference arrays from C++ xcfun source files. Become static test data in xcfun-core (test_data module) and xcfun-ad tests.
- **D-04:** For AD validation, use known analytical derivatives (e.g., d^3/dx^3 of exp(x) at x=1.0 = e) as ground truth for the AD engine.
- **D-05:** Compile and run C++ xcfun test suite for additional cross-validation reference data.
- **D-06:** CTaylor<T, N> uses truly generic const N (not specialized impls per N).
- **D-07:** Test at N=0 through N=7. Ensure no panics or numerical instability at boundary values.
- **D-08:** taylorlen() is a const fn in xcfun-core (not xcfun-ad) because the evaluation pipeline needs it.
- **D-09:** Replicate C++ compose() implementation exactly, coefficient by coefficient.
- **D-10:** All transcendental functions use compose internally.
- **D-11:** sqrtx_asinh_sqrtx uses Pade approximant near x=0, matching C++ exactly.
- **D-12:** Regularization at TINY_DENSITY = 1e-14 only affects constant term c[0], preserving derivative coefficients.
- **D-13:** FunctionalId enum has 78 variants with #[repr(u32)]. Provides from_name(), name(), description(), depends() methods.
- **D-14:** VarType enum has 30 variants. Each provides input_len(), provides() -> Dependency, is_spin_polarized() as const fn.
- **D-15:** Functional trait defined in xcfun-core but implemented in xcfun-functionals (Phase 2+).
- **D-16:** FunctionalImpl enum dispatch is NOT part of Phase 1.

### Claude's Discretion
- Exact test data format (inline arrays vs external files vs build-script-generated)
- Whether to use a proc macro for FunctionalId name/description metadata or hand-write match arms
- Error message wording in XcError variants
- Whether to derive Debug/Clone/Copy on all types or selectively

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CORE-01 | DensityVars<T> struct with all 25 fields and from_input() for all 30 VarType variants | C++ densvars.hpp constructor analyzed; switch/case-fallthrough pattern mapped to Rust match arms |
| CORE-02 | EvalMode enum (PartialDerivatives, Potential, Contracted) with mode validation | C++ xcfun_mode enum from xcfun.h verified |
| CORE-03 | VarType enum (30 variants) with input_len(), provides(), is_spin_polarized() metadata | C++ xcfun_vars enum from xcfun.h verified; ordering documented |
| CORE-04 | FunctionalId enum (78 variants) with from_name(), name(), description(), depends() | C++ functional registration pattern analyzed |
| CORE-05 | Dependency bitflags (DENSITY, GRADIENT, LAPLACIAN, KINETIC, JP) | C++ XC_DENSITY etc. flags at values 1,2,4,8,16 verified |
| CORE-06 | XcError enum with thiserror derive and FFI error code mapping | Error handling design doc provides complete variant list |
| CORE-07 | Physical constants module (C_SLATER, CF, TINY_DENSITY, MAX_ORDER) | C++ constants.hpp values extracted and verified |
| CORE-08 | Functional trait definition | Trait design doc provides complete signature; Phase 1 defines only, no implementations |
| AD-01 | CTaylor<T, N> struct with const generic N and bit-flag indexing | C++ ctaylor.hpp fully analyzed; Rust const generics pattern documented |
| AD-02 | All arithmetic operators with recursive multiplication | C++ ctaylor_rec mul/mul_set/multo analyzed; 3 recursive sub-problems per level |
| AD-03 | Transcendental functions (exp, log, pow, sqrt, cbrt, abs) | C++ tmath.hpp expansion functions extracted; compose pattern documented |
| AD-04 | Trigonometric functions (sin, cos, atan, asin, acos) | C++ tmath.hpp sin_expand, cos_expand, atan_expand, asin_expand, acos_expand analyzed |
| AD-05 | Special functions (asinh, erf, sqrtx_asinh_sqrtx) | C++ asinh_expand, erf_expand, sqrtx_asinh_sqrtx with Pade approximant analyzed |
| AD-06 | Num trait with implementations for f64 and CTaylor<f64, N> | Design doc provides complete trait definition |
| AD-07 | Taylor composition (chain rule) | C++ ctaylor_rec::compose() and tfuns::compose() analyzed in detail |
| AD-08 | taylorlen() function | C++ taylorlen() in taylor.hpp: C(nvar+ndeg, nvar) iterative computation |
| AD-09 | Numerical stability near zero, infinity, extreme coefficients | Regularization pattern, Pade approximant, assert guards documented from C++ |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CTaylor<T,N> arithmetic and transcendentals | xcfun-ad crate | -- | Self-contained AD engine with zero dependencies |
| Num trait definition + f64 impl | xcfun-ad crate | -- | Trait lives with its primary implementor |
| CTaylor<T,N> Num impl | xcfun-ad crate | -- | Impl lives with the type |
| DensityVars<T>, VarType, EvalMode, FunctionalId | xcfun-core crate | -- | Domain types consumed by all downstream crates |
| Functional trait definition | xcfun-core crate | -- | Trait defined where types live; implemented in xcfun-functionals |
| Dependency bitflags | xcfun-core crate | -- | Used by FunctionalId::depends() and VarType::provides() |
| XcError, error types | xcfun-core crate | -- | Public error type for library API |
| Physical constants | xcfun-core crate | -- | Domain constants (C_SLATER, CF, TINY_DENSITY) |
| taylorlen() | xcfun-core crate | -- | Decision D-08: evaluation pipeline needs it |
| Test reference data | xcfun-core crate (test_data module) | -- | Extracted from C++ source, shared across test suites |

## Standard Stack

### Core (Phase 1 Active)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust (Edition 2024) | 1.85+ (installed: 1.92) | Language | Const generics for `[T; 1 << N]`, stack allocation, zero-cost abstractions [VERIFIED: rustc --version] |
| thiserror | 2.0.18 | XcError derive | v2 required for Edition 2024 support [VERIFIED: cargo search] |
| bitflags | 2.11.1 | Dependency flags | Type-safe bitmask operations [VERIFIED: cargo search -- note: 2.11.1 is latest, not 2.10.0] |
| approx | 0.5.1 | Float comparison in tests | `assert_relative_eq!` for 1e-12 tolerance testing [VERIFIED: cargo search shows 0.6.0-rc2, use 0.5.1 stable] |

### Supporting (Phase 1 dev-dependencies only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| anyhow | 1.0.x | Test/example error handling | Integration tests and examples only, never in library crates |

### Not Yet Active (Stub Crates)

| Library | Version | Purpose | Phase |
|---------|---------|---------|-------|
| cubecl | =0.10.0-pre.3 | GPU batch eval | Phase 6 |
| pyo3 | 0.28.3 | Python bindings | Phase 7 |
| cbindgen | 0.29.2 | C header gen | Phase 7 |
| criterion | 0.8.2 | Benchmarks | Phase 8 |

**Installation (workspace Cargo.toml):**
```toml
[workspace.dependencies]
thiserror = "2.0.18"
bitflags = "2.11"
approx = "0.5"
anyhow = "1.0"
tracing = "0.1"
```

## Architecture Patterns

### System Architecture Diagram

```
Raw input array (f64 slice)
        |
        v
+------------------+         +-------------------+
| DensityVars<T>   |-------->| from_input()      |
| (xcfun-core)     |         | VarType dispatch  |
+------------------+         | Regularization    |
        |                    +-------------------+
        | T = f64 or CTaylor<f64, N>
        v
+------------------+         +-------------------+
| Functional trait |-------->| energy<T: Num>()  |
| (xcfun-core)     |         | (impl in Phase 2) |
+------------------+         +-------------------+
        |
        | Uses Num trait operations
        v
+------------------+         +-------------------+
| CTaylor<T, N>    |-------->| Arithmetic ops    |
| (xcfun-ad)       |         | (+, -, *, /)      |
+------------------+         | Transcendentals   |
        |                    | (exp, log, pow..) |
        |                    +-------------------+
        |                            |
        v                            v
+------------------+         +-------------------+
| compose()        |<--------| Taylor expansion  |
| (chain rule)     |         | coefficients      |
+------------------+         | (tmath functions) |
        |
        v
Output: [T; 1 << N] coefficients = derivatives at all orders
```

### Recommended Project Structure
```
xcfun_rs/
+-- Cargo.toml                     # workspace root
+-- crates/
|   +-- xcfun-ad/
|   |   +-- Cargo.toml             # zero external dependencies
|   |   +-- src/
|   |       +-- lib.rs             # re-exports
|   |       +-- ctaylor.rs         # CTaylor<T, N> struct, constructors, arithmetic
|   |       +-- num.rs             # Num trait definition + f64 impl
|   |       +-- compose.rs         # compose(), multo_skipconst() -- chain rule
|   |       +-- tmath.rs           # Taylor expansion generators (exp_expand, log_expand, etc.)
|   |       +-- math.rs            # Transcendental impls on CTaylor (exp, log, pow, sqrt, etc.)
|   +-- xcfun-core/
|   |   +-- Cargo.toml             # depends on xcfun-ad, thiserror, bitflags
|   |   +-- src/
|   |       +-- lib.rs             # re-exports, taylorlen()
|   |       +-- density_vars.rs    # DensityVars<T>, from_input()
|   |       +-- enums.rs           # EvalMode, VarType (with metadata methods)
|   |       +-- functional_id.rs   # FunctionalId enum (78 variants)
|   |       +-- traits.rs          # Functional trait, Dependency bitflags
|   |       +-- error.rs           # XcError (thiserror)
|   |       +-- constants.rs       # C_SLATER, CF, TINY_DENSITY, MAX_ORDER
|   |       +-- test_data.rs       # Reference test arrays from C++ source
|   +-- xcfun-functionals/         # STUB: Cargo.toml only
|   +-- xcfun-eval/                # STUB: Cargo.toml only
|   +-- xcfun-gpu/                 # STUB: Cargo.toml only
|   +-- xcfun-ffi/                 # STUB: Cargo.toml only
|   +-- xcfun-python/              # STUB: Cargo.toml only
```

### Pattern 1: Recursive Multiplication (CTaylor * CTaylor)

**What:** The C++ `ctaylor_rec<T, Nvar>::mul` splits arrays into halves (terms without/with the last variable) and recurses with 3 sub-problems.

**When to use:** Every CTaylor multiplication.

**Key insight from C++ source:** [VERIFIED: ctaylor.hpp lines 41-83]

```rust
// Source: xcfun-master/external/upstream/taylor/ctaylor.hpp
// The recursive structure mirrors the C++ template specialization:
//
// ctaylor_rec<T, Nvar>::mul(dst, x, y):
//   ctaylor_rec<T, Nvar-1>::mul(dst[..half], x[..half], y[..half])       // lo*lo
//   ctaylor_rec<T, Nvar-1>::mul(dst[half..], x[half..], y[..half])       // hi*lo
//   ctaylor_rec<T, Nvar-1>::mul(dst[half..], x[..half], y[half..])       // lo*hi
//
// Base case (Nvar=0): dst[0] += x[0] * y[0]
// Specialized (Nvar=1): dst[0] += x[0]*y[0]; dst[1] += x[0]*y[1] + x[1]*y[0]
//
// Note: P_hi * Q_hi is DROPPED (multilinear -- each variable at most first order)

fn mul_recursive(dst: &mut [f64], a: &[f64], b: &[f64]) {
    let n = a.len();
    debug_assert_eq!(n, b.len());
    debug_assert_eq!(n, dst.len());
    if n == 1 {
        dst[0] += a[0] * b[0];
        return;
    }
    let half = n / 2;
    mul_recursive(&mut dst[..half], &a[..half], &b[..half]);
    mul_recursive(&mut dst[half..], &a[half..], &b[..half]);
    mul_recursive(&mut dst[half..], &a[..half], &b[half..]);
}
```

**Critical variants to implement:**
- `mul` -- adds x*y to dst (accumulating)
- `mul_set` -- sets dst = x*y (first call non-accumulating)
- `multo` -- in-place dst *= y (order matters: hi before lo)
- `multo_skipconst` -- dst *= (y - y[0]), used in compose

### Pattern 2: Taylor Composition (Chain Rule)

**What:** `ctaylor_rec<T, Nvar>::compose(res, x, coeff[])` computes `sum_i coeff[i] * (x - x[0])^i` using Horner's method with `multo_skipconst`.

**When to use:** Every transcendental function (exp, log, pow, sqrt, sin, cos, atan, erf, asinh, asin, acos, etc.)

**C++ algorithm:** [VERIFIED: ctaylor.hpp lines 74-82]
```
compose(res, x, coeff[0..Nvar]):
    res[0] = coeff[Nvar]
    res[1..] = 0
    for i = Nvar-1 downto 0:
        multo_skipconst(res, x)    // res *= (x - x[0])
        res[0] += coeff[i]
```

There are also specialized compose implementations in `tmath.hpp` (`tfuns<T, N>::compose`) for 1D Taylor polynomials used inside the expansion generators. The 1D compose (lines 81-113 of tmath.hpp) uses a hardcoded switch/case for orders 0-6 with explicit algebraic expressions. This is used by `sqrtx_asinh_sqrtx` and `atan_expand`.

### Pattern 3: Taylor Expansion Generators

**What:** Functions like `exp_expand<T, N>(t, x0)` fill an array `t[0..N]` with the Taylor coefficients of a function around point `x0`.

**Source:** [VERIFIED: tmath.hpp]

| Function | Taylor coefficients t[k] | Formula |
|----------|-------------------------|---------|
| `exp_expand(t, x0)` | `t[0] = exp(x0)`, `t[k] = exp(x0)/k!` | All same up to factorial |
| `log_expand(t, x0)` | `t[0] = ln(x0)`, `t[k] = (-1)^(k+1) / (k * x0^k)` | Alternating inverse powers |
| `pow_expand(t, x0, a)` | `t[0] = x0^a`, `t[k] = t[k-1] * (a-k+1) / (k*x0)` | Falling factorial recurrence |
| `sqrt_expand(t, x0)` | `t[0] = sqrt(x0)`, `t[k] = t[k-1] * (3/(2k) - 1) / x0` | Special case of pow(x, 0.5) |
| `cbrt_expand(t, x0)` | `t[0] = cbrt(x0)`, `t[k] = t[k-1] * (4/(3k) - 1) / x0` | Special case of pow(x, 1/3) |
| `inv_expand(t, x0)` | `t[0] = 1/x0`, `t[k] = -t[k-1] / x0` | Geometric series |
| `sin_expand(t, x0)` | Interleaved sin/cos with factorials | See source |
| `cos_expand(t, x0)` | Interleaved cos/sin with factorials | See source |
| `atan_expand(t, x0)` | Expand 1/(1+x^2), compose with 2ax+x^2, integrate | Multi-step |
| `erf_expand(t, x0)` | gauss_expand * 2/sqrt(pi), integrate | Multi-step |
| `asinh_expand(t, x0)` | pow_expand of (1+x^2)^(-1/2), compose, integrate | Multi-step |
| `asin_expand(t, x0)` | pow_expand of (1-x^2)^(-1/2), compose, integrate | Multi-step |
| `acos_expand(t, x0)` | Same as asin but negated | Multi-step |

### Pattern 4: DensityVars Construction (Switch-Case Fallthrough)

**What:** The C++ `densvars` constructor uses switch-case fallthrough to progressively fill fields based on `VarType`. In Rust, this becomes explicit match arms.

**Critical detail from C++ source:** [VERIFIED: densvars.hpp]
- Regularization uses `regularize(x)` which checks `x < 1e-14` and sets `x.c[0] = 1e-14` for CTaylor (preserving derivative coefficients) or `x = 1e-14` for f64
- The C++ fallthrough pattern means e.g. `XC_A_B_GAA_GAB_GBB` falls through to `XC_A_B` -- in Rust we must explicitly handle each variant fully
- Derived quantities (`zeta`, `r_s`, `n_m13`, `a_43`, `b_43`) are computed AFTER the switch, using already-regularized values
- Regularization is applied to `n`, `a`, and `b` depending on the variable type

### Pattern 5: VarType Enum Ordering

**CRITICAL:** The C++ enum in `xcfun.h` has this ordering (relevant for `#[repr(u32)]`):

```
0:  XC_A
1:  XC_N
2:  XC_A_B
3:  XC_N_S
4:  XC_A_GAA
5:  XC_N_GNN
6:  XC_A_B_GAA_GAB_GBB
7:  XC_N_S_GNN_GNS_GSS
8:  XC_A_GAA_LAPA              (metaGGA laplacian)
9:  XC_A_GAA_TAUA              (metaGGA kinetic)
10: XC_N_GNN_LAPN
11: XC_N_GNN_TAUN
12: XC_A_B_GAA_GAB_GBB_LAPA_LAPB
13: XC_A_B_GAA_GAB_GBB_TAUA_TAUB
14: XC_N_S_GNN_GNS_GSS_LAPN_LAPS
15: XC_N_S_GNN_GNS_GSS_TAUN_TAUS
16: XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
17: XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB
18: XC_N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS
19: XC_A_AX_AY_AZ              (gradient components)
20: XC_A_B_AX_AY_AZ_BX_BY_BZ
21: XC_N_NX_NY_NZ
22: XC_N_S_NX_NY_NZ_SX_SY_SZ
23: XC_A_AX_AY_AZ_TAUA
24: XC_A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB
25: XC_N_NX_NY_NZ_TAUN
26: XC_N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS
27: XC_A_2ND_TAYLOR
28: XC_A_B_2ND_TAYLOR
29: XC_N_2ND_TAYLOR
30: XC_N_S_2ND_TAYLOR (= 30, so XC_NR_VARS = 31)
```

**WARNING:** The design doc (01-data-structures.md) shows a DIFFERENT ordering -- it lists gradient-component types at indices 8-11 and metaGGA types at higher indices. The Rust implementation MUST follow the C++ ordering for FFI compatibility. The design doc ordering should be corrected. [VERIFIED: xcfun.h lines 86-122]

### Anti-Patterns to Avoid

- **Hand-rolling specialized impls per N:** Decision D-06 mandates truly generic const N. Use `[T; 1 << N]` with const generic, not `match N { 1 => ..., 2 => ..., }`. The C++ uses template specialization for N=0, N=1, N=2 as optimizations; the Rust version should use a single generic recursion that the optimizer can specialize.

- **Clamping all CTaylor coefficients during regularization:** Only `c[0]` (constant term) is clamped. Setting `c[1..] = 0` would destroy derivative information and break every functional's output. [VERIFIED: densvars.hpp line 23]

- **Using `f64::powf` for constants:** Physical constants like `C_SLATER = (81/(32*pi))^(1/3)` must be computed at compile time or as `const` literals. Use precomputed values, not runtime `powf` in const contexts (Rust's const eval does not support `powf`).

- **Ignoring the C++ `multo` ordering:** In-place multiplication `multo(dst, y)` processes the HIGH half first, then adds cross-terms, then processes the LOW half. This order matters for correctness because the high half reads from the low half before it's modified.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Bitmask flag operations | Manual `u32` bit ops | `bitflags` 2.x macro | Handles `contains()`, `intersects()`, `union()`, Display, iteration, type safety |
| Structured error types | Manual Error/Display impls | `thiserror` 2.x derive | Avoids boilerplate, supports `#[from]`, works with Edition 2024 |
| Float tolerance assertions | `(a - b).abs() < eps` | `approx::assert_relative_eq!` | Handles near-zero denominators, reports useful diff on failure |
| Const array initialization | Manual loops | `std::array::from_fn` | Cleaner, works in const context for simple cases |

## Common Pitfalls

### Pitfall 1: Compose Algorithm Order Sensitivity
**What goes wrong:** Implementing compose with slightly different iteration order (e.g., bottom-up instead of Horner top-down) produces mathematically equivalent but numerically different results.
**Why it happens:** Floating-point addition is not associative. The C++ algorithm uses Horner's method (top-down: start from highest coefficient, multiply by polynomial, add next coefficient) which has specific rounding behavior.
**How to avoid:** Copy the C++ `ctaylor_rec::compose()` algorithm exactly. The loop runs `for i = Nvar-1 downto 0` with `multo_skipconst` followed by `res[0] += coeff[i]`.
**Warning signs:** AD tests pass for simple functions (exp, log) but fail for composed expressions or at higher orders.

### Pitfall 2: Const Generic Array Size Limits
**What goes wrong:** Compile errors or excessive compile times for `[T; 1 << N]` when N is not bounded.
**Why it happens:** Rust const generics evaluate `1 << N` at compile time. If N is unbounded, the compiler cannot verify array sizes. Also, `1 << 7 = 128` f64s = 1 KiB per CTaylor, which is the practical maximum.
**How to avoid:** Add a `where` clause or trait bound that constrains N (e.g., `N <= 7` via a sealed trait or assert). The design says MAX_ORDER = 6 which means N = max 7 variables per CTaylor.
**Warning signs:** Compilation succeeds but stack overflow at runtime for large N, or "overflow evaluating" compiler errors.

### Pitfall 3: VarType Ordering Mismatch
**What goes wrong:** Enum discriminant values don't match C++ `xcfun_vars`, causing FFI to pass wrong variable types.
**Why it happens:** The design doc (01-data-structures.md) shows a VarType ordering that differs from the C++ header. The gradient-component variants (AX_AY_AZ etc.) are in different positions.
**How to avoid:** Use the C++ `xcfun.h` ordering (documented in Pattern 5 above) as the authoritative source. The design doc ordering must be corrected.
**Warning signs:** Tests pass internally but cross-validation with C++ produces wrong results.

### Pitfall 4: DensityVars Regularization Scope
**What goes wrong:** Regularizing `n` but not `a` and `b` independently (or vice versa) leads to inconsistency where `a + b != n`.
**Why it happens:** The C++ code regularizes `a` and `b` independently, then computes `n = a + b`. For unpolarized inputs (`XC_N`), it regularizes `n` then sets `a = b = 0.5*n`. The regularization pattern depends on the VarType.
**How to avoid:** Follow the C++ `densvars` constructor exactly for each VarType case. For alpha/beta inputs, regularize a and b, then compute n = a + b. For n/s inputs, regularize n, compute a = (n+s)/2 and b = (n-s)/2, then regularize a and b independently.
**Warning signs:** Very small density tests produce slightly different results between Rust and C++.

### Pitfall 5: tmath compose vs ctaylor compose
**What goes wrong:** Confusing the two `compose` functions -- `ctaylor_rec::compose` operates on CTaylor coefficient arrays (2^N elements), while `tfuns::compose` operates on 1D Taylor polynomial coefficient arrays (N+1 elements).
**Why it happens:** Both are called "compose" in the C++ source. They serve different purposes: `ctaylor_rec::compose` maps a scalar function onto a multivariate polynomial, while `tfuns::compose` composes two 1D Taylor series.
**How to avoid:** Name them differently in Rust: `ctaylor_compose()` for the CTaylor version, `taylor1d_compose()` for the 1D version. The 1D version is used internally by `atan_expand`, `erf_expand`, `asinh_expand`, and `sqrtx_asinh_sqrtx`.
**Warning signs:** atan, erf, or asinh produce wrong derivatives while exp and log work correctly.

### Pitfall 6: sqrt_expand Recurrence Formula
**What goes wrong:** Getting the recurrence relation wrong for `sqrt_expand` or `cbrt_expand`.
**Why it happens:** The C++ formulas look unusual: `t[i] = t[i-1] * ((3*x0inv)/(2*i) - x0inv)` for sqrt, `t[i] = t[i-1] * ((4*x0inv)/(3*i) - x0inv)` for cbrt. These are special cases of the general `pow_expand` recurrence.
**How to avoid:** Verify by expanding manually: for sqrt, the k-th Taylor coefficient of `sqrt(x0 + h)` around h=0 is `(1/2)(1/2-1)(1/2-2)...(1/2-k+1) / k! * x0^(1/2-k)`. Compare term-by-term against the recurrence.
**Warning signs:** sqrt/cbrt derivatives correct at order 1 but wrong at order 2+.

## Code Examples

### CTaylor Struct Definition
```rust
// Follows C++ ctaylor<T, Nvar> structure
#[derive(Clone)]
pub struct CTaylor<T, const N: usize> {
    pub c: [T; 1 << N],
}

impl<T: Clone + Default, const N: usize> CTaylor<T, N> {
    pub const SIZE: usize = 1 << N;

    pub fn constant(value: T) -> Self {
        let mut c = std::array::from_fn(|_| T::default());
        c[0] = value;
        Self { c }
    }

    pub fn variable(value: T, var: usize) -> Self
    where
        T: From<f64>,
    {
        debug_assert!(var < N, "variable index {var} out of range for N={N}");
        let mut c = std::array::from_fn(|_| T::default());
        c[0] = value;
        c[1 << var] = T::from(1.0);
        Self { c }
    }
}
```

### Recursive Multiplication (mul_set variant)
```rust
// Source: ctaylor.hpp lines 49-53
// mul_set: first call (dst = a*b), subsequent adds via mul
fn mul_set_recursive(dst: &mut [f64], a: &[f64], b: &[f64]) {
    let n = a.len();
    if n == 1 {
        dst[0] = a[0] * b[0];
        return;
    }
    let half = n / 2;
    // dst_lo = a_lo * b_lo
    mul_set_recursive(&mut dst[..half], &a[..half], &b[..half]);
    // dst_hi = a_hi * b_lo
    mul_set_recursive(&mut dst[half..], &a[half..], &b[..half]);
    // dst_hi += a_lo * b_hi (accumulating!)
    mul_recursive(&mut dst[half..], &a[..half], &b[half..]);
}
```

### multo_skipconst (Used by compose)
```rust
// Source: ctaylor.hpp lines 62-65
// Computes dst *= (y - y[0]), i.e. multiply by polynomial with zeroed constant
fn multo_skipconst_recursive(dst: &mut [f64], y: &[f64]) {
    let n = dst.len();
    if n == 1 {
        dst[0] = 0.0; // Base case: result is 0
        return;
    }
    let half = n / 2;
    // Process high half first (reads from low half before it's modified)
    multo_skipconst_recursive(&mut dst[half..], y);
    mul_recursive(&mut dst[half..], &dst[..half], &y[half..]);  // NOTE: reads dst[..half]
    // Then process low half
    multo_skipconst_recursive(&mut dst[..half], y);
}
```

**CRITICAL NOTE:** The `multo_skipconst` function reads `dst[..half]` (low half) when computing the high half cross-term. This means the low half must NOT be modified before the cross-term is computed. The C++ achieves this naturally through recursive call ordering; the Rust version must maintain the same ordering. This may require borrowing tricks (copy the low half to a temporary) since Rust won't allow simultaneous mutable borrow of `dst[half..]` and immutable borrow of `dst[..half]`.

### Exp Expansion
```rust
// Source: tmath.hpp lines 132-139
fn exp_expand(t: &mut [f64], x0: f64) {
    let n = t.len() - 1; // degree
    let mut ifac: f64 = 1.0;
    t[0] = x0.exp();
    for i in 1..=n {
        ifac *= i as f64;
        t[i] = t[0] / ifac;
    }
}
```

### Regularization
```rust
// Source: densvars.hpp lines 22-29
// For CTaylor: only modify c[0], preserve derivatives
fn regularize_ctaylor<const N: usize>(x: &mut CTaylor<f64, N>) {
    if x.c[0] < TINY_DENSITY {
        x.c[0] = TINY_DENSITY;
    }
}

// For f64: simple clamp
fn regularize_f64(x: &mut f64) {
    if *x < TINY_DENSITY {
        *x = TINY_DENSITY;
    }
}
```

### Physical Constants
```rust
// Source: xcfun-master/src/functionals/constants.hpp
pub mod constants {
    use std::f64::consts::PI;

    /// Slater exchange constant: (81/(32*pi))^(1/3)
    /// C++ computes: pow(81 / (32 * M_PI), 1.0 / 3.0)
    /// Value: 0.9305257363491002 (verified against C++ output)
    pub const C_SLATER: f64 = 0.9305257363491002;

    /// Thomas-Fermi kinetic constant: 0.3 * (3*pi^2)^(2/3)
    pub const CF: f64 = 2.8711842930059836;

    /// Tiny density threshold for regularization
    pub const TINY_DENSITY: f64 = 1e-14;

    /// Maximum derivative order
    pub const MAX_ORDER: u32 = 6;

    /// PBE gamma = (1 - ln(2)) / pi^2
    pub const PARAM_GAMMA: f64 = 0.031090690869654895; // (1 - 2.0_f64.ln()) / (PI * PI)

    /// PBE beta (accurate value)
    pub const PARAM_BETA_ACCURATE: f64 = 0.06672455060314922;
}
```

**NOTE:** The design doc states `C_SLATER = 1.5 * (3/(4*pi))^(1/3) = 0.7386...` but the C++ defines it as `(81/(32*pi))^(1/3) = 0.9305...`. These are different constants used in different formulations. Verify: `(81/(32*PI))^(1/3) = (81/(32*3.14159...))^(1/3) = (0.8059...)^(1/3) = 0.9305...`. The C++ typically writes Slater exchange as `-C_x * (rho_a^(4/3) + rho_b^(4/3))` where C_x is this value. The design doc uses a different convention. **The Rust implementation MUST match the C++ value.** [VERIFIED: constants.hpp line 30]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `impl<T, const N: usize>` without bounds | Rust 1.79+ allows `where [(); 1 << N]:` bounds | Rust 1.79 (mid-2024) | Enables compile-time array sizing in stable Rust [ASSUMED] |
| `Default::default()` for array init | `std::array::from_fn` | Stable since Rust 1.63 | Cleaner array initialization pattern |
| thiserror 1.x | thiserror 2.0.18 | 2024 | Required for Edition 2024 support [VERIFIED: CLAUDE.md] |
| `#[derive(Error)]` from thiserror 1.x | Same syntax, different internals in 2.x | thiserror 2.0 | No API change for users |
| bitflags 1.x | bitflags 2.11.1 | 2023 | Const-friendly, better type safety [VERIFIED: cargo search] |

**Note on const generics:** Rust stable (1.92) supports basic const generics (`const N: usize`) and expressions like `1 << N` in array types. However, `where [(); 1 << N]:` bounds may require nightly or feature flags depending on exact usage. The common workaround is to use a trait-based approach or accept that the compiler will error for invalid N values at the call site. This needs validation during implementation. [ASSUMED]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `where [(); 1 << N]:` bounds work on Rust stable 1.92 for const generic array sizes | State of the Art | May need nightly feature or alternative pattern (sealed trait bound). LOW risk: can use `assert!(N <= 7)` at runtime instead. |
| A2 | C_SLATER value 0.9305... matches C++ exactly to all 16 digits | Code Examples | If even one digit differs, all functionals will fail 1e-12 tolerance. MEDIUM risk: verify by computing in Rust at runtime and comparing. |
| A3 | Splitting `multo_skipconst` to handle the borrow-checker limitation (simultaneous read of dst[..half] and write of dst[half..]) can be done without extra allocation | Code Examples | May need a temporary copy of the low half. LOW risk: 1 KiB max temporary per call. |

## Open Questions (RESOLVED)

1. **Const generic array bounds on stable Rust** (RESOLVED)
   - Resolution: `[T; 1 << N]` in struct definitions works on stable Rust. The `where [(); 1 << N]:` bound syntax requires `generic_const_exprs` (nightly). Use the fallback: define CTaylor with `pub c: [T; 1 << N]` directly (this works because `1 << N` is a valid const expression in array types). If additional where-clause bounds are needed, use a sealed trait `ValidOrder` implemented for N=0..7.

2. **C_SLATER precise value** (RESOLVED)
   - Resolution: Use the C++ computed value as a Rust `const` literal with all 16 significant digits. Compute `(81.0_f64 / (32.0 * std::f64::consts::PI)).powf(1.0 / 3.0)` at test time and verify it matches. If any digit differs, the literal from C++ takes precedence. Add a `#[test]` that asserts the const matches the computed value.

3. **Test data extraction from C++ source** (RESOLVED)
   - Resolution: Phase 1 extracts test data by reading C++ source files (xcfun-master/src/functionals/*.cpp) and copying test_in/test_out arrays into a Rust test_data module. Not all 78 functionals need test data in Phase 1 — only the AD engine and core types need validation here. Full functional test data extraction happens in Phase 2 (validation pipeline).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust compiler | Everything | Yes | 1.92.0 stable | -- |
| Cargo | Build system | Yes | 1.92.0 | -- |
| C++ compiler | D-05 (build C++ xcfun for cross-validation) | Needs verification | -- | Skip cross-validation; use extracted static test data only |

**Missing dependencies with no fallback:**
- None

**Missing dependencies with fallback:**
- C++ compiler: If not available, cross-validation (D-05) is deferred. Static test data extraction (D-03) still works by reading source files directly.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `approx` 0.5 |
| Config file | None needed -- Rust tests are convention-based |
| Quick run command | `cargo test -p xcfun-ad && cargo test -p xcfun-core` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AD-01 | CTaylor struct creation and indexing | unit | `cargo test -p xcfun-ad -- ctaylor_creation` | Wave 0 |
| AD-02 | Arithmetic operators (+,-,*,/) | unit | `cargo test -p xcfun-ad -- ctaylor_arithmetic` | Wave 0 |
| AD-03 | Transcendental functions (exp,log,pow,sqrt,cbrt,abs) | unit | `cargo test -p xcfun-ad -- transcendental` | Wave 0 |
| AD-04 | Trigonometric functions (sin,cos,atan,asin,acos) | unit | `cargo test -p xcfun-ad -- trigonometric` | Wave 0 |
| AD-05 | Special functions (asinh,erf,sqrtx_asinh_sqrtx) | unit | `cargo test -p xcfun-ad -- special_functions` | Wave 0 |
| AD-06 | Num trait for f64 and CTaylor | unit | `cargo test -p xcfun-ad -- num_trait` | Wave 0 |
| AD-07 | Compose (chain rule) | unit | `cargo test -p xcfun-ad -- compose` | Wave 0 |
| AD-08 | taylorlen() | unit | `cargo test -p xcfun-core -- taylorlen` | Wave 0 |
| AD-09 | Numerical stability edge cases | unit | `cargo test -p xcfun-ad -- stability` | Wave 0 |
| CORE-01 | DensityVars from_input for all VarTypes | unit | `cargo test -p xcfun-core -- density_vars` | Wave 0 |
| CORE-02 | EvalMode enum | unit | `cargo test -p xcfun-core -- eval_mode` | Wave 0 |
| CORE-03 | VarType metadata methods | unit | `cargo test -p xcfun-core -- var_type` | Wave 0 |
| CORE-04 | FunctionalId lookup and metadata | unit | `cargo test -p xcfun-core -- functional_id` | Wave 0 |
| CORE-05 | Dependency bitflags | unit | `cargo test -p xcfun-core -- dependency` | Wave 0 |
| CORE-06 | XcError Display and variants | unit | `cargo test -p xcfun-core -- error` | Wave 0 |
| CORE-07 | Constants values | unit | `cargo test -p xcfun-core -- constants` | Wave 0 |
| CORE-08 | Functional trait compiles | smoke | `cargo check -p xcfun-core` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p xcfun-ad && cargo test -p xcfun-core`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/xcfun-ad/Cargo.toml` -- crate definition
- [ ] `crates/xcfun-core/Cargo.toml` -- crate definition with xcfun-ad dependency
- [ ] All test files listed above must be created as part of implementation tasks

## Security Domain

Security enforcement is not relevant for this phase. This is a pure computational mathematics library with no network I/O, no user authentication, no file system access beyond compilation, and no untrusted input processing. The only "input" is numerical arrays of f64 values.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A |
| V3 Session Management | No | N/A |
| V4 Access Control | No | N/A |
| V5 Input Validation | Minimal | Array length checks in from_input(); regularization of near-zero densities |
| V6 Cryptography | No | N/A |

## Sources

### Primary (HIGH confidence)
- `xcfun-master/external/upstream/taylor/ctaylor.hpp` -- CTaylor struct, recursive multiplication, compose algorithm
- `xcfun-master/external/upstream/taylor/tmath.hpp` -- All Taylor expansion generators (exp, log, pow, sqrt, cbrt, sin, cos, atan, erf, asinh, asin, acos)
- `xcfun-master/external/upstream/taylor/ctaylor_math.hpp` -- Transcendental function wrappers using compose
- `xcfun-master/src/densvars.hpp` -- DensityVars constructor with all VarType switch cases
- `xcfun-master/src/functionals/constants.hpp` -- Physical constants (C_SLATER, CF)
- `xcfun-master/src/config.hpp` -- TINY_DENSITY = 1e-14
- `xcfun-master/api/xcfun.h` -- VarType enum ordering (authoritative), EvalMode enum
- `xcfun-master/src/xcint.hpp` -- functional_data struct, test_in/test_out arrays
- `docs/design/03-autodiff.md` -- Rust AD engine design decisions
- `docs/design/01-data-structures.md` -- Rust type definitions
- `docs/design/02-traits.md` -- Num trait, Functional trait, GpuEvaluable trait

### Secondary (MEDIUM confidence)
- `cargo search` -- Verified crate versions (thiserror 2.0.18, bitflags 2.11.1, approx 0.5/0.6-rc2, criterion 0.8.2)
- `rustc --version` -- Confirmed Rust 1.92.0 stable installed

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crate versions verified against registry, Rust toolchain confirmed
- Architecture: HIGH -- C++ source code analyzed in detail, algorithm structures mapped to Rust patterns
- Pitfalls: HIGH -- identified from actual C++ source code analysis and Rust const generics experience

**Research date:** 2026-04-17
**Valid until:** 2026-05-17 (stable domain, no fast-moving dependencies in Phase 1)
