---
phase: 01-core-types-ad-engine
reviewed: 2026-04-17T12:00:00Z
depth: standard
files_reviewed: 17
files_reviewed_list:
  - Cargo.toml
  - crates/xcfun-ad/Cargo.toml
  - crates/xcfun-ad/src/compose.rs
  - crates/xcfun-ad/src/ctaylor.rs
  - crates/xcfun-ad/src/lib.rs
  - crates/xcfun-ad/src/math.rs
  - crates/xcfun-ad/src/num.rs
  - crates/xcfun-ad/src/tmath.rs
  - crates/xcfun-core/Cargo.toml
  - crates/xcfun-core/src/constants.rs
  - crates/xcfun-core/src/density_vars.rs
  - crates/xcfun-core/src/enums.rs
  - crates/xcfun-core/src/error.rs
  - crates/xcfun-core/src/functional_id.rs
  - crates/xcfun-core/src/lib.rs
  - crates/xcfun-core/src/test_data.rs
  - crates/xcfun-core/src/traits.rs
findings:
  critical: 1
  warning: 5
  info: 4
  total: 10
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-04-17T12:00:00Z
**Depth:** standard
**Files Reviewed:** 17
**Status:** issues_found

## Summary

The xcfun-ad and xcfun-core crates implement a multilinear automatic differentiation engine and core type system for DFT functional evaluation. The code compiles cleanly and all 151 tests pass. The AD engine (compose, ctaylor, tmath, math) is a faithful port of the C++ xcfun algorithms with good test coverage. The core types (DensityVars, VarType, FunctionalId, Dependency) are well-structured.

Key concerns: one critical bug in the `ctaylor_abs` function where the derivative is non-differentiable at zero (matching C++ but worth flagging), a heap allocation in a recursive hot path (`multo_skipconst`), a duplicated `nvar_from_size` function, and a `taylorlen` function with potential integer overflow in const context. The `CF` constant value does not match the stated formula, which may or may not be intentional.

## Critical Issues

### CR-01: Division by zero guard uses exact comparison with debug_assert

**File:** `crates/xcfun-ad/src/ctaylor.rs:187`
**Issue:** The division guard `debug_assert!(q0.abs() > 0.0, ...)` uses `debug_assert!` which is stripped in release builds. If a CTaylor with `c[0] == 0.0` is used as a divisor in release mode, the code silently computes `1.0 / 0.0 = inf` and propagates infinity/NaN through the entire coefficient array. While the C++ version has similar behavior, this is a correctness risk in a numerical library where callers may not expect silent inf propagation. For a library targeting 1e-12 accuracy, undetected inf corruption is critical.
**Fix:**
```rust
// Option A: Always assert (not just debug)
assert!(q0.abs() > 0.0, "division by zero: CTaylor divisor has c[0] == 0");

// Option B: Return a Result or use a checked division method
// (depends on whether the project wants panicking or error-returning division)
```

## Warnings

### WR-01: Heap allocation in recursive hot path (multo_skipconst)

**File:** `crates/xcfun-ad/src/compose.rs:83`
**Issue:** `multo_skipconst` allocates a `Vec<f64>` via `dst[..half].to_vec()` on every recursive call. For N=128 (7 variables, the maximum supported), this creates 7 levels of recursion with allocations at each level. Since compose operations are called for every transcendental function evaluation at every grid point, this heap allocation pressure can be significant. The C++ version avoids this by using pointer arithmetic without aliasing issues.
**Fix:** Use a stack-allocated buffer since the maximum half-size is 64 elements (512 bytes):
```rust
// For sizes up to 128, half <= 64. Use a fixed-size array on the stack.
let mut lo_copy = [0.0; 64]; // or use a smaller const based on N
lo_copy[..half].copy_from_slice(&dst[..half]);
mul_recursive(&mut dst[half..], &lo_copy[..half], &y[half..]);
```
Alternatively, accept a scratch buffer parameter to avoid repeated allocation.

### WR-02: CF constant does not match its documented formula

**File:** `crates/xcfun-core/src/constants.rs:13`
**Issue:** The comment says `CF = 0.3 * (3*pi^2)^(2/3)` but the actual value 2.8711842930059836 differs from the formula result 2.871234293... The test comment acknowledges this ("may differ slightly from the formula") but does not verify against the actual C++ runtime value. If the C++ value is correct, the comment formula is misleading. If the formula is correct, the constant is wrong.
**Fix:** Either update the comment to state the exact derivation used by the C++ code, or verify the value against the C++ binary output:
```rust
/// Thomas-Fermi kinetic constant.
/// C++ runtime value (may differ from textbook 0.3 * (3*pi^2)^(2/3) = 2.871234
/// due to C++ compile-time constant folding differences).
pub const CF: f64 = 2.8711842930059836;
```

### WR-03: ctaylor_abs is non-differentiable at x=0

**File:** `crates/xcfun-ad/src/math.rs:79-85`
**Issue:** `ctaylor_abs` returns `x` when `c[0] >= 0.0` and `-x` when `c[0] < 0.0`. At exactly `c[0] == 0.0`, the derivative is discontinuous (should be undefined), but the function returns the identity (derivative = +1.0). This matches the C++ behavior but can produce incorrect derivatives when the evaluation point is exactly at zero. DFT functionals may hit this when density is regularized to a tiny value.
**Fix:** Document the behavior explicitly and consider whether a smooth approximation (e.g., `sqrt(x^2 + epsilon)`) would be more appropriate for the use case. If matching C++ is the priority, add a doc comment:
```rust
/// abs(x) for CTaylor: if c[0] >= 0, return x; else negate all coefficients.
///
/// WARNING: Non-differentiable at x=0. At c[0]==0.0, returns x (derivative = +1).
/// This matches C++ xcfun behavior.
```

### WR-04: taylorlen can overflow silently for large inputs

**File:** `crates/xcfun-core/src/lib.rs:33-40`
**Issue:** The `taylorlen` function computes `len = len * (n_vars + k) / k` in a loop. For large `n_vars` and `order`, the multiplication `len * (n_vars + k)` can overflow before the division by `k`. Since this is a `const fn`, the overflow behavior depends on build mode (panic in debug, wrap in release). The function has no bounds checks. While current usage is bounded (max 7 vars, order 6), the function is public and could be misused.
**Fix:**
```rust
pub const fn taylorlen(n_vars: usize, order: usize) -> usize {
    assert!(n_vars <= 20 && order <= 20, "taylorlen: inputs too large");
    let mut len: usize = 1;
    let mut k: usize = 1;
    while k <= order {
        len = len * (n_vars + k) / k;
        k += 1;
    }
    len
}
```

### WR-05: Num trait missing DivAssign bound

**File:** `crates/xcfun-ad/src/num.rs:13-24`
**Issue:** The `Num` trait requires `AddAssign`, `SubAssign`, and `MulAssign` but does not require `DivAssign`. However, `CTaylor<f64, N>` implements `DivAssign<f64>` and `DivAssign`. If a functional implementation uses `/=` via the `Num` trait, it will fail to compile. This is an asymmetry that may cause issues when implementing functionals.
**Fix:** Either add `DivAssign` to the trait bounds, or document that `/=` is not available through the `Num` trait:
```rust
pub trait Num:
    Clone
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + std::ops::DivAssign  // add this
    + Sized
```

## Info

### IN-01: Duplicated nvar_from_size function

**File:** `crates/xcfun-ad/src/ctaylor.rs:30-43` and `crates/xcfun-ad/src/math.rs:14-27`
**Issue:** The `nvar_from_size` const function is defined identically in both `ctaylor.rs` and `math.rs`. This duplication risks divergence if one is updated without the other.
**Fix:** Move `nvar_from_size` to `lib.rs` or a shared utility module within xcfun-ad, and import it from both files.

### IN-02: Compiler warnings for unused variable assignments in density_vars.rs

**File:** `crates/xcfun-core/src/density_vars.rs:84-87`
**Issue:** The compiler generates 35 warnings about variables like `n` and `s` being assigned zero and then overwritten in the match arms. While functionally correct, this produces noisy build output.
**Fix:** Initialize `n` and `s` (and similar) as `let mut n;` (uninitialized binding) or use `T::zero()` only in the match arms where they are not overwritten. Alternatively, restructure using a builder pattern or helper functions per VarType group.

### IN-03: FunctionalId::COUNT not verified against actual variant count

**File:** `crates/xcfun-core/src/functional_id.rs:106`
**Issue:** `FunctionalId::COUNT` is hardcoded to 78 but there is no compile-time or test-time verification that the enum actually has 78 variants. If a variant is added or removed, COUNT can silently go stale.
**Fix:** Add a test that iterates through all variants (using a macro or strum crate) to verify the count, or derive it programmatically.

### IN-04: Scalar operation impls take CTaylor by value (consuming it)

**File:** `crates/xcfun-ad/src/ctaylor.rs:113-324`
**Issue:** All arithmetic operator implementations take `self` by value, which means every operation consumes the operand. This forces frequent `.clone()` calls throughout functional implementations (visible in test code). While this is standard for Copy types, CTaylor is not Copy (it contains `[f64; N]` which is Clone but not Copy for large N). This may lead to unnecessary copies in functional code.
**Fix:** Consider adding reference-based operator implementations (`impl Add for &CTaylor<f64, N>`) to reduce cloning overhead in complex expressions. This can be done incrementally as functional implementations reveal the need.

---

_Reviewed: 2026-04-17T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
