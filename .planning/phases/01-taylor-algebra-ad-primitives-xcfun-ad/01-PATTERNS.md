# Phase 1: Taylor Algebra & AD Primitives (cubecl-native) — Pattern Map

**Mapped:** 2026-04-19
**Files analyzed:** 18 new/modified files (the complete cubecl-native `xcfun-ad` surface)
**Analogs found:** 18 / 18 (every file has either a C++ reference analog under `xcfun-master/external/upstream/taylor/` or a pre-pivot Rust file under `crates/xcfun-ad/src/` serving as an operation-order oracle)

**Important note on analogs:** Phase 1 is the **first cubecl file in the workspace**. There is no existing `#[cube] fn` Rust code anywhere in `crates/` (grep confirms zero matches for `cubecl::prelude` or `#[cube]`). Analogs fall into two categories:

| Analog kind | Role in the port |
|-------------|------------------|
| **C++ reference (`xcfun-master/external/upstream/taylor/*.hpp`)** | Algorithmic-identity source-of-truth. Every `#[cube] fn` body must preserve the C++ operation order line-for-line (CONTEXT.md D-08, D-10, D-14). |
| **Pre-pivot hand-Rust (`crates/xcfun-ad/src/**`)** | About to be reverted in Wave 0 per D-21. Useful **only** as a translation step — they show how the C++ was already adapted to Rust idioms (explicit `let` bindings for D-08, `assert!` over `debug_assert!` for D-11, stack-only scratch arrays). The planner should treat these as ephemeral scaffolding, not as templates. The final code wraps the same operation sequence in `#[cube] fn` + `Array<F>`. |
| **cubecl reference (external)** | No in-repo cubecl code. Planner should consult `docs/design/06-cubecl-strategy.md` §3 ("Kernel structure") and the cubecl-book `core-features/features.md` cited in RESEARCH.md. Per-site `#[cube]` idioms (e.g., `#[comptime]` const generics, `Array<F>` indexing, `F: Float` bound) are not yet embodied in any project file. |

---

## File Classification

### New `#[cube]` source files (created in Waves 0-2)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-ad/src/lib.rs` | crate-root / re-exports | config / host-visible consts | `crates/xcfun-ad/src/lib.rs` (pre-pivot, being rewritten) | role-match — same purpose, new surface |
| `crates/xcfun-ad/src/index.rs` | config (host-visible const flags) | static data | `xcfun-master/external/upstream/taylor/ctaylor.hpp:12-20` (`#define VAR0..VAR7`) | exact — verbatim constants |
| `crates/xcfun-ad/src/ctaylor.rs` | model / device type (`#[cube] CTaylor<F, N>`) | transform | `xcfun-master/external/upstream/taylor/ctaylor.hpp:154-337` (`struct ctaylor<T, Nvar>`) | exact — direct port to cubecl form |
| `crates/xcfun-ad/src/ctaylor_rec/mul.rs` | transform (`#[cube] fn ctaylor_mul`) | transform | `ctaylor.hpp:41-65` (general recursion) + `:86-152` (N=0,1,2 base cases) | exact — load-bearing verbatim port |
| `crates/xcfun-ad/src/ctaylor_rec/multo.rs` | transform (`#[cube] fn ctaylor_multo`, `*_skipconst`) | transform | `ctaylor.hpp:55-65` (general) + `:88, 103-110, 131-142` (base cases) | exact — verbatim port |
| `crates/xcfun-ad/src/ctaylor_rec/compose.rs` | transform (`#[cube] fn ctaylor_compose`) | transform | `ctaylor.hpp:72-82` (general `compose`) + `:91, 112-115, 146-151` (base cases) | exact — verbatim port |
| `crates/xcfun-ad/src/tfuns.rs` | transform (`#[cube] fn tfuns::{mul,multo,integrate,differentiate,shift,compose,stretch}`) | transform | `xcfun-master/external/upstream/taylor/tmath.hpp:36-121` (`template<T,N> struct tfuns`) | exact — verbatim port |
| `crates/xcfun-ad/src/expand/inv.rs` | transform (`#[cube] fn inv_expand<F>`) | transform | `tmath.hpp:124-129` + `crates/xcfun-ad/src/expand/inv.rs` (pre-pivot, reverted) | exact — C++ recurrence + explicit `let`-chain translation |
| `crates/xcfun-ad/src/expand/exp.rs` | transform (`#[cube] fn exp_expand<F>`) | transform | `tmath.hpp:132-139` + `crates/xcfun-ad/src/expand/exp.rs` (pre-pivot, reverted) | exact |
| `crates/xcfun-ad/src/expand/log.rs` | transform (`#[cube] fn log_expand<F>`) | transform | `tmath.hpp:142-151` + `crates/xcfun-ad/src/expand/log.rs` (pre-pivot, reverted) | exact |
| `crates/xcfun-ad/src/expand/pow.rs` | transform (`#[cube] fn pow_expand<F>`) | transform | `tmath.hpp:154-161` + `crates/xcfun-ad/src/expand/pow.rs` (pre-pivot, reverted) | exact |
| `crates/xcfun-ad/src/expand/sqrt.rs` | transform (`#[cube] fn sqrt_expand<F>`) | transform | `tmath.hpp:164-170` + `crates/xcfun-ad/src/expand/sqrt.rs` (pre-pivot, reverted) | exact |
| `crates/xcfun-ad/src/expand/cbrt.rs` | transform (`#[cube] fn cbrt_expand<F>`) | transform | `tmath.hpp:172-178` | exact — sqrt_expand structure with `4/(3i)` coefficient |
| `crates/xcfun-ad/src/expand/atan.rs` | transform (`#[cube] fn atan_expand<F>`) | transform + compose | `tmath.hpp:180-198` | exact |
| `crates/xcfun-ad/src/expand/gauss.rs` | transform (`#[cube] fn gauss_expand<F>`) | transform + compose | `tmath.hpp:200-215` | exact |
| `crates/xcfun-ad/src/expand/erf.rs` | transform (`#[cube] fn erf_expand<F>`) | transform + compose | `tmath.hpp:217-225` | exact |
| `crates/xcfun-ad/src/expand/asinh.rs` | transform (`#[cube] fn asinh_expand<F>`) | transform + compose | `tmath.hpp:259-274` | exact |
| `crates/xcfun-ad/src/math.rs` | service (composed elementary funcs on `CTaylor`) | pipeline (expand → compose) | `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:7-325` | exact — 1:1 function-per-function port |
| `crates/xcfun-ad/src/for_tests/mod.rs` | utility (test harness seam) | request-response | `crates/xcfun-ad/src/for_tests.rs` (pre-pivot, being rewritten) | role-match |
| `crates/xcfun-ad/src/for_tests/cpu_client.rs` | config / singleton (`OnceLock<CpuClient>`) | singleton init | NO IN-REPO ANALOG — cubecl-specific (see §No Analog Found) | none |
| `crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs` | utility (1-thread kernel launch wrapper) | request-response | NO IN-REPO ANALOG — cubecl-specific | none |

### Test files (created in Waves 1-2)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-ad/tests/fixtures/` (directory + `.gitkeep`) | config (committed oracle data) | static data | — | — |
| `crates/xcfun-ad/tests/golden_mul.rs` | test (golden to_bits parity) | request-response | (no pre-pivot analog in repo; design in RESEARCH.md §Code Examples Ex.5) | role-match |
| `crates/xcfun-ad/tests/golden_expand.rs` | test | request-response | (same) | role-match |
| `crates/xcfun-ad/tests/golden_composed.rs` | test | request-response | (same) | role-match |
| `crates/xcfun-ad/tests/props_ring.rs` | test (proptest batch-per-property) | batch | (no pre-pivot analog; D-18 batch-per-property is new to Phase 1) | none (use design from RESEARCH.md §Code Examples Ex.4 as starting point) |
| `crates/xcfun-ad/tests/props_leibniz.rs` | test | batch | (same) | none |
| `crates/xcfun-ad/tests/props_roundtrips.rs` | test | batch | (same) | none |

### Bench files (created in Wave 2 / Phase 1 gate)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-ad/benches/mul_bench.rs` | bench | batch throughput | `crates/xcfun-ad/benches/mul_bench.rs` (current stub, `fn main() {}`) | role-match — same file, populated |
| `crates/xcfun-ad/benches/compose_bench.rs` | bench | batch throughput | `crates/xcfun-ad/benches/compose_bench.rs` (current stub) | role-match |

### xtask & build config (Wave 0)

| New / Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------------|------|-----------|----------------|---------------|
| `crates/xcfun-ad/Cargo.toml` | config | static data | `crates/xcfun-ad/Cargo.toml` (pre-pivot — add cubecl deps) | role-match |
| `Cargo.toml` (workspace) | config | static data | `Cargo.toml` (workspace — add cubecl pins) | role-match |
| `.cargo/config.toml` | config | static data | `.cargo/config.toml` (existing — preserved) | exact — keep |
| `xtask/src/bin/regen_ad_fixtures.rs` | utility (fixture generation) | file-I/O + C++ FFI | NO IN-REPO ANALOG — design in RESEARCH.md §Golden-fixture Tooling | none |
| `xtask/assets/regen_ad_fixtures/driver.cpp` | build (C++ driver) | file-I/O | `xcfun-master/external/upstream/taylor/unittest_taylor.cpp` (shape reference) | role-match |

### Files to revert (Wave 0, D-21)

Modifications during revert task: `crates/xcfun-ad/src/{lib.rs, ctaylor.rs, valid_n.rs, for_tests.rs}` and `crates/xcfun-ad/src/expand/{mod.rs, inv.rs, exp.rs, log.rs, pow.rs, sqrt.rs}` are **removed** via `git revert` of commits `217af4d`, `f07611c`, `c7a3f46`, `1b95fe3`, `2db557c` plus cleanup of untracked WIP. No patterns apply — these files cease to exist.

---

## Pattern Assignments

### `crates/xcfun-ad/src/lib.rs` (crate-root, re-exports)

**Analog:** Pre-pivot `crates/xcfun-ad/src/lib.rs` (reverted; use ONLY for the `#![forbid(unsafe_code)]` pattern and the `pub const VAR0..VAR7` re-exports)

**Crate-root pattern** (pre-pivot lib.rs lines 32-47 — copy the attribute set and re-export shape):

```rust
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod ctaylor;        // #[cube] CTaylor<F, N> type
pub mod ctaylor_rec;    // #[cube] multiply / compose recursion
pub mod expand;         // #[cube] *_expand scalar series
pub mod tfuns;          // #[cube] tfuns helpers
pub mod math;           // #[cube] composed elementary funcs
pub mod index;          // host-visible CNST / VAR0..VAR7 consts

#[cfg(feature = "testing")]
pub mod for_tests;

pub use ctaylor::CTaylor;
pub use index::{CNST, VAR0, VAR1, VAR2, VAR3, VAR4, VAR5, VAR6, VAR7};
```

**Do NOT carry forward:** The pre-pivot `pub use valid_n::{Bound, ValidN};` surface and the `ValidN<N, SIZE>` two-const-generic scheme. Per D-05 the sealed trait is replaced by cubecl const-generic validation / `debug_assert!`-in-kernel. No `Bound` type.

---

### `crates/xcfun-ad/src/index.rs` (host-visible const flags)

**Analog:** `xcfun-master/external/upstream/taylor/ctaylor.hpp` lines 12-20 (verbatim C++ `#define`s)

**C++ source** (`ctaylor.hpp:12-20`):

```cpp
#define CNST 0 // avoid defining CONST
#define VAR0 1
#define VAR1 2
#define VAR2 4
#define VAR3 8
#define VAR4 16
#define VAR5 32
#define VAR6 64
#define VAR7 128
```

**Rust port** (copy-paste with `pub const` prefix; CONTEXT.md D-06 says these stay host-visible):

```rust
pub const CNST: u32  = 0;
pub const VAR0: u32  = 1;
pub const VAR1: u32  = 2;
pub const VAR2: u32  = 4;
pub const VAR3: u32  = 8;
pub const VAR4: u32  = 16;
pub const VAR5: u32  = 32;
pub const VAR6: u32  = 64;
pub const VAR7: u32  = 128;
```

**Note on type:** Pre-pivot used `usize`. Under cubecl, `#[comptime]` slots prefer `u32`. D-06 says "passed as `#[comptime]` values inside `#[cube]` scopes" — prefer `u32` unless planner discovers a cubecl 0.10-pre.3 constraint requiring `usize`.

---

### `crates/xcfun-ad/src/ctaylor.rs` (`#[cube]` CTaylor type, transform)

**Analog:** `xcfun-master/external/upstream/taylor/ctaylor.hpp` lines 154-337 (struct header + constructors + elementwise ops)

**C++ struct** (`ctaylor.hpp:154-170`):

```cpp
template <class T, int Nvar> struct ctaylor {
  enum { size = POW2(Nvar) };
  T c[size];
  ctaylor() { for (int i = 0; i < size; i++) c[i] = 0; }
  ctaylor(const T & c0) {
    c[0] = c0;
    for (int i = 1; i < size; i++) c[i] = 0;
  }
  // ...
};
```

**Elementwise Add** (`ctaylor.hpp:295-311, 468-474`):

```cpp
template <class T, int Nvar>
ctaylor<T, Nvar> operator+(const ctaylor<T, Nvar> & t1, const ctaylor<T, Nvar> & t2) {
  ctaylor<T, Nvar> res;
  for (int i = 0; i < POW2(Nvar); i++) res.c[i] = t1.c[i] + t2.c[i];
  return res;
}
```

**multo recursion, N=2** (`ctaylor.hpp:131-135` — load-bearing):

```cpp
static void multo(T * dst, const T * y) {
  dst[3] = dst[0] * y[3] + dst[3] * y[0] + dst[1] * y[2] + dst[2] * y[1];
  dst[2] = dst[0] * y[2] + dst[2] * y[0];
  dst[1] = dst[0] * y[1] + dst[1] * y[0];
  dst[0] = dst[0] * y[0];
}
```

**Cubecl port shape (no existing in-repo analog; skeleton only):**

```rust
// D-04: pure #[cube] type. NO host struct. NO #[derive(CubeType)].
// Storage = cubecl::prelude::Array<F> of length 1 << N allocated in kernel scope.

use cubecl::prelude::*;

// CTaylor is conceptually "an Array<F> of length 1 << N owned by some kernel scope".
// It's passed to all arithmetic #[cube] fns by `&Array<F>` / `&mut Array<F>`.
// Const generic N is threaded as `#[comptime] N: u32`.

#[cube]
pub fn ctaylor_add<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // ctaylor.hpp:308-309: for (int i = 0; i < POW2(Nvar); i++) c[i] += t.c[i];
    let size = 1_u32 << n;
    #[unroll]
    for i in 0..size {
        out[i] = a[i] + b[i];
    }
}
```

**Pre-pivot hand-Rust Add pattern** (`crates/xcfun-ad/src/ctaylor.rs:120-135` — for operation-order reference, NOT for direct copy):

```rust
fn add(self, rhs: Self) -> Self {
    // ctaylor.hpp:308-309: `for (int i = 0; i < POW2(Nvar); i++) c[i] += t.c[i];`
    let mut c = [0.0_f64; SIZE];
    for i in 0..SIZE {
        c[i] = self.c[i] + rhs.c[i];
    }
    Self { c }
}
```

Use this pre-pivot block **only** to confirm the D-08 intent (indexed `for i in 0..SIZE` loop, no `.iter().zip()`). Port that loop into the cubecl form above.

**Constructors (ctaylor.hpp:179-198 — `from_scalar`, `from_variable` equivalents):**
Inside cubecl these become `#[cube] fn ctaylor_from_scalar(c0: F, out: &mut Array<F>, #[comptime] n: u32)` etc. Body: `#[unroll] for i in 0..size { out[i] = F::new(0.0); } out[0] = c0;`.

---

### `crates/xcfun-ad/src/ctaylor_rec/mul.rs` (load-bearing recursion, transform)

**Analog:** `xcfun-master/external/upstream/taylor/ctaylor.hpp` lines 41-65 (general recursion) + 86-152 (N=0,1,2 base cases). This is the **single most load-bearing port in Phase 1** (CONTEXT.md D-08, RESEARCH.md P3).

**C++ general recursion** (`ctaylor.hpp:41-47, 55-59`):

```cpp
template <class T, int Nvar> struct ctaylor_rec {
  // Add x*y to dst
  static void mul(T * dst, const T * x, const T * y) {
    ctaylor_rec<T, Nvar - 1>::mul(dst, x, y);
    ctaylor_rec<T, Nvar - 1>::mul(dst + POW2(Nvar - 1), x + POW2(Nvar - 1), y);
    ctaylor_rec<T, Nvar - 1>::mul(dst + POW2(Nvar - 1), x, y + POW2(Nvar - 1));
  }
  // dst = dst * y
  static void multo(T * dst, const T * y) {
    ctaylor_rec<T, Nvar - 1>::multo(dst + POW2(Nvar - 1), y);
    ctaylor_rec<T, Nvar - 1>::mul(dst + POW2(Nvar - 1), dst, y + POW2(Nvar - 1));
    ctaylor_rec<T, Nvar - 1>::multo(dst, y);
  }
};
```

**C++ N=0 base case** (`ctaylor.hpp:86-92`):

```cpp
template <class T> struct ctaylor_rec<T, 0> {
  static void mul(T * dst, const T * x, const T * y)     { dst[0] += x[0] * y[0]; }
  static void mul_set(T * dst, const T * x, const T * y) { dst[0]  = x[0] * y[0]; }
  static void multo(T * dst, const T * y)                { dst[0] *= y[0]; }
  static void multo_skipconst(T * dst, const T * y)      { dst[0]  = 0; }
};
```

**C++ N=1 base case** (`ctaylor.hpp:94-116` — note `multo` writes `dst[1]` BEFORE `dst[0]`):

```cpp
template <class T> struct ctaylor_rec<T, 1> {
  static void mul(T * dst, const T * x, const T * y) {
    dst[0] += x[0] * y[0];
    dst[1] += x[0] * y[1] + x[1] * y[0];
  }
  static void multo(T * dst, const T * y) {
    dst[1] = dst[1] * y[0] + dst[0] * y[1];   // MUST precede dst[0] update
    dst[0] *= y[0];
  }
};
```

**C++ N=2 base case** (`ctaylor.hpp:118-142` — critical descending-index write order):

```cpp
template <class T> struct ctaylor_rec<T, 2> {
  static void multo(T * dst, const T * y) {
    dst[3] = dst[0] * y[3] + dst[3] * y[0] + dst[1] * y[2] + dst[2] * y[1];
    dst[2] = dst[0] * y[2] + dst[2] * y[0];
    dst[1] = dst[0] * y[1] + dst[1] * y[0];
    dst[0] = dst[0] * y[0];
  }
};
```

**Cubecl port shape (planner derives the exact form from cubecl 0.10-pre.3 idioms; there is no in-repo analog):**

Per-N specialization — a separate `#[cube] fn ctaylor_multo_n{k}` for each `k ∈ 0..=7` — is the pattern dictated by D-08 ("Per-order specialization for N ∈ 0..=7. No re-association, no parallel accumulation, no cubecl::reduce::* primitives"). General recursion via cubecl const-generic dispatch is the fallback only if the planner confirms cubecl 0.10-pre.3 supports const-generic recursion in `#[cube]` fns; otherwise macro-expand.

```rust
#[cube]
fn ctaylor_multo_n2<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    // ctaylor.hpp:131-135 — EXACT write order
    // Explicit let bindings preserve operation order per D-08.
    let t3 = dst[0] * y[3] + dst[3] * y[0] + dst[1] * y[2] + dst[2] * y[1];
    dst[3] = t3;
    let t2 = dst[0] * y[2] + dst[2] * y[0];
    dst[2] = t2;
    let t1 = dst[0] * y[1] + dst[1] * y[0];
    dst[1] = t1;
    dst[0] = dst[0] * y[0];
}
```

**Code-review anchors per D-10:**
1. Every `#[cube] fn ctaylor_multo_n{k}` has a header comment citing `ctaylor.hpp:<start>-<end>`.
2. Explicit `let` for every intermediate expression — no `a + b + c + d` collapse (D-08).
3. `#[cube]` intrinsic `F::add`, `F::mul` should NOT be hand-called; infix `+` / `*` on `F: Float` are the expected surface.

**Borrow-checker discipline** (from RESEARCH.md Pitfall "borrow-checker forcing heap"):
- No `.to_vec()` or heap scratch. Use cubecl's Array slicing primitives if Rust's borrow-checker refuses split reads while writing. Planner task: confirm cubecl 0.10-pre.3's `Array<F>` supports the required aliasing / slicing patterns; fall back to stack-allocated scratch if not.

---

### `crates/xcfun-ad/src/expand/exp.rs` (scalar Taylor series, transform)

**Analog (both):**
- `xcfun-master/external/upstream/taylor/tmath.hpp:132-139` (C++ authoritative)
- `crates/xcfun-ad/src/expand/exp.rs` (pre-pivot, reverted — shows the C++-to-Rust operation-order mapping that the cubecl port will further re-express)

**C++ source** (`tmath.hpp:132-139`):

```cpp
template <class T, int Ndeg> static void exp_expand(T * t, const T & x0) {
  T ifac = 1;
  t[0] = exp(x0);
  for (int i = 1; i <= Ndeg; i++) {
    ifac *= i;
    t[i] = t[0] / ifac;
  }
}
```

**Pre-pivot Rust pattern** (`crates/xcfun-ad/src/expand/exp.rs:28-42` — shows explicit `let` chain per D-08):

```rust
pub fn exp_expand(t: &mut [f64], x0: f64) {
    let mut ifac: f64 = 1.0;
    t[0] = exp_f64(x0);
    for i in 1..t.len() {
        let i_f = i as f64;
        ifac = ifac * i_f;
        t[i] = t[0] / ifac;
    }
}
```

**Cubecl port shape (skeleton):**

```rust
use cubecl::prelude::*;

/// Fill `t[0..=n]` with Taylor coefficients of `exp(x0+x)`.
///
/// tmath.hpp:132-139 verbatim.
/// Identity: `exp(x0+x) = exp(x0) * sum_{i>=0} x^i / i!`
/// Preconditions: none (exp analytic everywhere).
#[cube]
pub fn exp_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // tmath.hpp:133: T ifac = 1;
    let mut ifac = F::new(1.0);

    // tmath.hpp:134: t[0] = exp(x0);  (cubecl's F::exp intrinsic)
    t[0] = F::exp(x0);

    // tmath.hpp:135-138 — explicit let bindings per D-08
    #[unroll]
    for i in 1..=n {
        let i_f = F::cast_from(i);   // i as F
        ifac = ifac * i_f;
        t[i] = t[0] / ifac;
    }
}
```

**Shared pattern applied to ALL `*_expand` files:**

| Element | Source | Applied uniformly |
|---------|--------|-------------------|
| **Module header doc comment** | Pre-pivot `expand/exp.rs:1-27` | Every `#[cube] fn *_expand` opens with: `//!` comment citing `tmath.hpp:<lines>`, pasted C++ block, mathematical identity in LaTeX, explicit preconditions (D-13) |
| **Precondition asserts** | Pre-pivot `expand/inv.rs:38`, `log.rs:41`, `pow.rs:35`, `sqrt.rs:34` | `assert!(pred, "msg")` at fn start — D-11 says `assert!` not `debug_assert!` even in release |
| **Explicit `let` chain** | Pre-pivot `expand/sqrt.rs:45-52`, `log.rs:57-63` | Every C++ expression with > 2 operands gets one `let` binding per sub-expression (D-08) |
| **libm call (C++ `exp(x0)`) → F::exp intrinsic** | (no in-repo analog) | Replace `x.exp()` with cubecl's `F::exp(x)` intrinsic; similarly `F::log`, `F::sqrt`, `F::powf`, `F::cbrt`, `F::erf` |

Specific ports per-file:

| File | C++ analog | Pre-pivot analog (deprecated) |
|------|-----------|-------------------------------|
| `expand/inv.rs` | `tmath.hpp:124-129` | `crates/xcfun-ad/src/expand/inv.rs` (N=0 test, N=3 bit-exact test copy-over) |
| `expand/log.rs` | `tmath.hpp:142-151` | `crates/xcfun-ad/src/expand/log.rs` (`(2*(i&1)-1)` sign-factor expression order) |
| `expand/pow.rs` | `tmath.hpp:154-161` | `crates/xcfun-ad/src/expand/pow.rs` (note: precondition is `x0 > 0`, NOT `a > 0`) |
| `expand/sqrt.rs` | `tmath.hpp:164-170` | `crates/xcfun-ad/src/expand/sqrt.rs` (5-line expression split pattern) |
| `expand/cbrt.rs` | `tmath.hpp:172-178` | **no pre-pivot analog** — structurally identical to sqrt with `4/(3i)` coefficient |
| `expand/atan.rs` | `tmath.hpp:180-198` | none — composed from `inv_expand` + `tfuns::{compose, integrate}` |
| `expand/gauss.rs` | `tmath.hpp:200-215` | none — composed from `exp_expand` + `tfuns::{stretch, multo}` |
| `expand/erf.rs` | `tmath.hpp:217-225` | none — calls `gauss_expand` + `tfuns::integrate` |
| `expand/asinh.rs` | `tmath.hpp:259-274` | none — composed from `pow_expand` + `tfuns::{compose, integrate}` |

---

### `crates/xcfun-ad/src/tfuns.rs` (scalar helpers, transform)

**Analog:** `xcfun-master/external/upstream/taylor/tmath.hpp:36-121` (`template<T,N> struct tfuns`)

**C++ source extract** (`tmath.hpp:36-56` — subset; full range lines 36-121 is the port target):

```cpp
template <class T, int N> struct tfuns {
  static void mul(T * z, const T * x, const T * y) {
    for (int i = 0; i <= N; i++) {
      z[i] = x[0] * y[i];
      for (int j = 1; j <= i; j++)
        z[i] += x[j] * y[i - j];
    }
  }
  // z *= x -- write descending per D-08
  static void multo(T * z, const T * x) {
    for (int i = N; i >= 0; i--) {
      z[i] = x[0] * z[i];
      for (int j = 1; j <= i; j++)
        z[i] += x[j] * z[i - j];
    }
  }
  static void integrate(T * x) {
    for (int i = N; i >= 1; i--) x[i] = x[i - 1] / i;
  }
  // ... differentiate, shift, compose (switch cascade N=6..0), stretch
};
```

**Cubecl port:** Seven `#[cube] fn tfuns_<name><F: Float>(args..., #[comptime] n: u32)` — one per C++ static method. The `compose` switch cascade (tmath.hpp:80-113) is the trickiest: it uses C++ fallthrough so every case `case k:` executes cases `k, k-1, …, 0`. Port by writing separate `#[cube] fn tfuns_compose_n{k}<F>(f: &mut Array<F>, x: &Array<F>)` for each k ∈ 0..=6, and a top-level `tfuns_compose<F>` that dispatches on `#[comptime] n`.

**Pre-pivot analog:** NONE. Not yet ported in the pre-pivot code.

---

### `crates/xcfun-ad/src/math.rs` (composed elementary functions, service)

**Analog:** `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:7-325` (1:1 function-per-function port)

**C++ source — composed `exp`** (`ctaylor_math.hpp:71-81`):

```cpp
template <class T, int Nvar>
static ctaylor<T, Nvar> exp(const ctaylor<T, Nvar> & t) {
  T tmp[Nvar + 1];
  exp_expand<T, Nvar>(tmp, t.c[0]);
  ctaylor<T, Nvar> res;
  ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
  return res;
}
```

**Cubecl port pattern** (applied uniformly to all 9 composed functions: `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`):

```rust
#[cube]
pub fn ctaylor_exp<F: Float>(
    x: &Array<F>,                   // input polynomial, length 1 << n
    out: &mut Array<F>,             // output polynomial, length 1 << n
    scratch: &mut Array<F>,         // length n+1 (caller provides)
    #[comptime] n: u32,
) {
    // ctaylor_math.hpp:76: T tmp[Nvar + 1];
    //   Scratch provided by caller — D-01 stack-only, no heap.
    // ctaylor_math.hpp:77: exp_expand<T, Nvar>(tmp, t.c[0]);
    exp_expand::<F>(scratch, x[0], n);
    // ctaylor_math.hpp:78-79: ctaylor<T, Nvar> res; compose(res.c, t.c, tmp);
    ctaylor_compose::<F>(out, x, scratch, n);
}
```

**Function table** — apply pattern above to each row:

| Rust function | ctaylor_math.hpp lines | Scratch fill call | Precondition |
|---------------|------------------------|-------------------|--------------|
| `ctaylor_reciprocal<F>` | 7-28 | `inv_expand` | `x[0] != 0.0` |
| `ctaylor_sqrt<F>` | 133-145 | `sqrt_expand` | `x[0] > 0.0` |
| `ctaylor_exp<F>` | 71-81 | `exp_expand` | none |
| `ctaylor_log<F>` | 104-115 | `log_expand` | `x[0] > 0.0` |
| `ctaylor_pow<F>` | 117-131 | `pow_expand` | `x[0] > 0.0` |
| `ctaylor_powi<F>` | 165-178 | integer fast path (no expand) | none (incl. x[0]=0 for n>=0) |
| `ctaylor_erf<F>` | 194-206 | `erf_expand` | none |
| `ctaylor_asinh<F>` | 256-268 | `asinh_expand` | none |
| `ctaylor_atan<F>` | 180-192 | `atan_expand` | none |

**Pre-pivot analog:** `crates/xcfun-ad/src/math.rs` does NOT exist (Plan 01-05 never landed). The pre-pivot plan called for it in Wave 2, so there is no reference code. Planner derives the pattern from the ctaylor_math.hpp lines above + the `#[cube]` idiom established by the `*_expand` files.

---

### `crates/xcfun-ad/src/for_tests/cpu_client.rs` (singleton init, config)

**Analog:** NONE IN-REPO. See §No Analog Found below.

**Design (from CONTEXT.md D-17 + RESEARCH.md):**

```rust
use std::sync::OnceLock;
use cubecl_cpu::CpuClient;

static CPU_CLIENT: OnceLock<CpuClient> = OnceLock::new();

/// Shared cubecl-cpu client for all tests in the binary. Matches
/// cubecl-cpu's expected usage (single client, shared across tasks).
pub fn cpu_client() -> &'static CpuClient {
    CPU_CLIENT.get_or_init(|| {
        use cubecl_cpu::CpuRuntime;
        // Per cubecl 0.10-pre.3 idiom — exact init sequence is a planner
        // task (consult cubecl-book core-features/features.md; no in-repo
        // example exists).
        todo!("cubecl-cpu client init — planner fills from 0.10-pre.3 docs")
    })
}
```

**Rationale:** D-17 says "exposes a `OnceLock<CpuClient>` initialized on first call and shared across every test in the binary." The Rust stdlib pattern is idiomatic; the cubecl-specific `CpuClient` init ceremony is the only unknown and must be resolved at implementation time.

---

### `crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs` (request-response wrapper)

**Analog:** NONE IN-REPO.

**Design (from CONTEXT.md D-16):**

Generic helper: "launch a 1-thread kernel with these inputs, collect output array." Signature:

```rust
/// Launch `kernel_fn` as a 1-thread kernel on `cpu_client()` with the given
/// input buffer, returning the output buffer contents as a Vec.
///
/// Internal use — for test bodies and property-test fixture generation.
/// Not part of the public API (feature="testing" only).
pub fn raw_eval_scalar<F: Float, ...>(
    inputs: &[F],
    expected_output_len: usize,
    kernel_fn: impl Fn(/* kernel launch args */),
) -> Vec<F> {
    // Use cpu_client(); allocate device buffers sized to inputs/outputs;
    // launch 1-thread kernel; sync; download; return.
    todo!()
}
```

This is a piece of cubecl scaffolding that the planner should derive from the cubecl-book's "launching a kernel" example.

---

### `crates/xcfun-ad/tests/golden_mul.rs` (golden to_bits test, request-response)

**Analog:** Pre-pivot does NOT have this file. Closest reference: RESEARCH.md §Code Examples Example 5 (lines 889-934).

**Fixture record schema** (CONTEXT.md D-19, §specifics):

```rust
#[derive(serde::Deserialize)]
struct FixtureRecord {
    op: String,       // "mul" | "inv_expand" | "exp_expand" | ...
    n_var: u8,        // 0..=3 for the to_bits gate
    inputs: Vec<f64>, // flat layout
    coeffs: Vec<f64>, // expected
}
```

**Test skeleton** (RESEARCH.md Example 5, adapted for cubecl kernel launch):

```rust
#[test]
fn mul_matches_cpp_reference_to_bits_n_le_3() {
    let bytes = include_bytes!("fixtures/mul.bincode");
    let records: Vec<FixtureRecord> = bincode::deserialize(bytes).unwrap();
    for rec in records.iter().filter(|r| r.op == "mul" && r.n_var <= 3) {
        // 1. Pack rec.inputs into cubecl Array
        // 2. Launch ctaylor_mul kernel (via raw_eval_scalar or direct)
        // 3. Download output Array -> Vec<f64>
        // 4. Assert to_bits equality with rec.coeffs
        let got = /* kernel launch */ ;
        for (i, (g, e)) in got.iter().zip(&rec.coeffs).enumerate() {
            assert_eq!(g.to_bits(), e.to_bits(),
                "mul n_var={} coeff {i}: got {g}, expected {e}", rec.n_var);
        }
    }
}
```

---

### `crates/xcfun-ad/tests/props_ring.rs` (batch-per-property proptest, batch)

**Analog:** NONE IN-REPO (D-18 batch-per-property is new to Phase 1). Closest reference: RESEARCH.md §Code Examples Example 4 (per-test launch, NOT batched).

**Pattern (D-18):**

```rust
use proptest::prelude::*;
use proptest::strategy::Strategy;
use proptest::test_runner::TestRunner;

#[test]
fn associativity_batched_10k() {
    let mut runner = TestRunner::default();
    let strategy = (
        prop::collection::vec(-100.0_f64..100.0, 4),  // a coefficients, N=1 => [CNST, VAR0]
        prop::collection::vec(-100.0_f64..100.0, 4),
        prop::collection::vec(-100.0_f64..100.0, 4),
    );

    // 1. Generate 10k inputs upfront.
    let mut all_inputs: Vec<(Vec<f64>, Vec<f64>, Vec<f64>)> = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let tree = strategy.new_tree(&mut runner).unwrap();
        all_inputs.push(tree.current());
    }

    // 2. Pack into a single cubecl Array (flat layout: a0,b0,c0, a1,b1,c1, ...).
    // 3. Launch a single kernel that computes (a+b)+c and a+(b+c) across all 10k points.
    // 4. Aggregate diffs host-side and prop_assert! tolerances.
    // (Planner fills the cubecl launch details.)
}
```

**Shared with `props_leibniz.rs`, `props_roundtrips.rs`:** Same batch-pack-launch-aggregate shape; only the property predicate and the kernel body differ.

---

### `crates/xcfun-ad/Cargo.toml` (config, dependencies)

**Analog:** Current `crates/xcfun-ad/Cargo.toml` (preserve `[features]` structure; add cubecl deps).

**Diff pattern (add cubecl, remove `valid_n` / ValidN assumptions):**

```toml
[package]
name = "xcfun-ad"
version.workspace = true
edition.workspace = true

[features]
default = ["cpu"]
cpu = ["dep:cubecl-cpu"]        # D-26: only `cpu` feature at xcfun-ad level
testing = []                    # D-22: for_tests seam

[dependencies]
cubecl = { workspace = true }
cubecl-cpu = { workspace = true, optional = true }
# libm removed — cubecl F::exp / F::log intrinsics replace libm on the
# numerical path.

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

**Workspace `Cargo.toml` additions** (root — adds cubecl pins per CLAUDE.md):

```toml
[workspace.dependencies]
# existing:
# thiserror = "2.0.18"
# bitflags = "2.11"
# approx = "0.5"
# ...

# cubecl pivot additions:
cubecl = "=0.10.0-pre.3"
cubecl-cpu = "=0.10.0-pre.3"
# (cubecl-cuda, cubecl-wgpu NOT added at Phase 1 — Phase 6 scope per D-26)
```

---

## Shared Patterns

### SP-1: Doc-comment header citing C++ source (applied to every `#[cube] fn` port)

**Source:** Pre-pivot `crates/xcfun-ad/src/expand/exp.rs:1-22` (header comment shape)
**Mandate:** CONTEXT.md D-13 — every ported `*_expand` and every ported `ctaylor_rec` function carries the three-item header.
**Apply to:** every file under `src/ctaylor_rec/`, `src/expand/`, `src/math.rs`, `src/tfuns.rs`.

```rust
//! <Function name and short description>. Port of
//! `xcfun-master/external/upstream/taylor/<filename>.hpp:<start>-<end>`.
//!
//! # C++ recurrence (<filename>.hpp:<start>-<end>)
//!
//! ```cpp
//! <paste exact C++ block>
//! ```
//!
//! # Mathematical identity
//!
//! <LaTeX or plain-text statement of the identity>
//!
//! # Precondition
//!
//! <precondition, e.g. `x0 > 0`. `assert!` active in release (D-11).>
```

### SP-2: Explicit `let` bindings for every expression with > 2 operands

**Source:**
- CONTEXT.md D-08 (the mandate)
- Pre-pivot `crates/xcfun-ad/src/expand/sqrt.rs:45-52` (concrete example)
- Pre-pivot `crates/xcfun-ad/src/expand/log.rs:57-63` (concrete example)

**Example pattern (from `sqrt.rs:45-52`):**

```rust
// C++: t[i] = t[i - 1] * ((3 * x0inv) / (2 * i) - x0inv);
// Rust: decompose into single-op statements to defeat compiler reassociation.
for i in 1..t.len() {
    let i_f = i as f64;          // integer-to-float cast
    let num = 3.0 * x0inv;       // numerator
    let den = 2.0 * i_f;         // denominator
    let quot = num / den;        // division
    let factor = quot - x0inv;   // subtraction
    t[i] = t[i - 1] * factor;    // final multiplication
}
```

**Apply to:** every `#[cube] fn` body with a C++ expression containing 3+ floating-point operations.

### SP-3: `assert!` (not `debug_assert!`) for analyticity preconditions

**Source:**
- CONTEXT.md D-11 / RESEARCH.md P10
- Pre-pivot `crates/xcfun-ad/src/expand/inv.rs:38`, `log.rs:41`, `pow.rs:35`, `sqrt.rs:34`

**Example:**

```rust
// inv_expand: tmath.hpp:125: assert(a != 0 && "1/(a+x) not analytic at a = 0");
assert!(a != F::new(0.0), "1/(a+x) not analytic at a = 0");

// log_expand: tmath.hpp:143
assert!(x0 > F::new(0.0), "log(x) not real analytic at x <= 0");

// pow_expand: tmath.hpp:155-156
assert!(x0 > F::new(0.0), "pow(x,a) not real analytic at x <= 0");

// sqrt_expand: tmath.hpp:165
assert!(x0 > F::new(0.0), "sqrt(x) not real analytic at x <= 0");
```

**Apply to:** `expand/{inv,log,pow,sqrt,cbrt}.rs` — every `*_expand<F>` whose C++ version has an `assert`.

**Caveat for cubecl:** If cubecl 0.10-pre.3's `#[cube]` proc-macro rejects host-style `assert!` inside kernel bodies, fall back to `debug_assert!` + CI test that exercises the release build. D-05's fallback text acknowledges this: "fallback is a `debug_assert!` at kernel entry." Planner research step confirms which form compiles.

### SP-4: Stack-only scratch buffer (no heap)

**Source:** CONTEXT.md D-01; RESEARCH.md Pitfall "heap allocation in composed functions".
**Anti-pattern** (from RESEARCH.md — violates D-01):

```rust
let tmp = vec![0.0; Nvar + 1];   // WRONG — heap allocation on hot path
```

**Correct pattern (pre-pivot hand-Rust):**

```rust
let mut scratch = [0.0_f64; 8];            // N_MAX + 1 = 8 on stack
let slice = &mut scratch[..=N];            // use only 0..=N entries
```

**Cubecl equivalent:** `Array<F>` of length 8 allocated inside the `#[cube] fn` scope. cubecl's memory model keeps this in registers / local memory; no device global allocation.

**Apply to:** every composed `ctaylor_<op>` function in `src/math.rs` + every `*_expand` that internally needs a temporary (tmath.hpp `asinh_expand`, `gauss_expand`, `atan_expand` use a `T tmp[Ndeg+1]` VLA).

### SP-5: FMA suppression carry-forward

**Source:** `.cargo/config.toml` (current — preserved as-is). CONTEXT.md D-02 mandates.

```toml
[build]
rustflags = ["-Cllvm-args=-fp-contract=off"]

[target.'cfg(all())']
rustflags = ["-Cllvm-args=-fp-contract=off"]
```

**Apply to:** the Cargo.toml is already correct. Phase 1 adds a CI asm-spot-check task (per D-02) that verifies cubecl-cpu's JIT-lowered kernels for `ctaylor_mul` emit no `vfmadd`/`fmadd`. The asm-spot-check task is part of VALIDATION.md, not PATTERNS.md — flagged here because D-02 explicitly says this is non-negotiable.

### SP-6: `#[forbid(unsafe_code)]` at crate root

**Source:** Pre-pivot `crates/xcfun-ad/src/lib.rs:32`. Matches CONTEXT.md "No unsafe" constraint.

```rust
#![forbid(unsafe_code)]
```

**Apply to:** preserved verbatim in `src/lib.rs`. cubecl macros expand to tokens the compiler checks — no hand-written `unsafe` permitted.

### SP-7: Fixture record schema

**Source:** CONTEXT.md §specifics ("Fixture records: preserve ... schema so the C++ driver is unchanged"); RESEARCH.md Example 5.

**Shared struct used by every `tests/golden_*.rs` file AND the xtask C++ driver:**

```rust
#[derive(serde::Deserialize, serde::Serialize)]
struct FixtureRecord {
    op: String,
    n_var: u8,
    inputs: Vec<f64>,
    coeffs: Vec<f64>,
}
```

**Apply to:** `tests/golden_mul.rs`, `tests/golden_expand.rs`, `tests/golden_composed.rs`, and `xtask/src/bin/regen_ad_fixtures.rs` (define once in a shared crate, e.g., `xtask::fixtures::FixtureRecord`, and both read/write sides import).

---

## No Analog Found

Files with **no close match** anywhere in the workspace. The planner must consult external references (cubecl-book, cubecl 0.10-pre.3 docs fetched via Context7) to establish the pattern. These are explicitly called out so the planner knows to include a cubecl research task ahead of writing these files.

| File | Role | Data Flow | Reason no analog exists |
|------|------|-----------|-------------------------|
| `crates/xcfun-ad/src/for_tests/cpu_client.rs` | config / singleton | singleton init | Cubecl-cpu `CpuClient` init idiom is cubecl 0.10-pre.3 specific; no cubecl usage anywhere in `crates/` today. |
| `crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs` | utility wrapper | request-response | 1-thread cubecl kernel launch; no prior kernel launches in the repo. |
| All `#[cube] fn` bodies in `ctaylor.rs`, `ctaylor_rec/**`, `tfuns.rs`, `expand/**`, `math.rs` — the cubecl-specific surface | transform | transform | No `#[cube]` code exists in `crates/`. The **logic** (operation sequencing) has in-repo analogs (hand-Rust pre-pivot) and C++ analogs (xcfun-master); the **cubecl expression form** (`Array<F>`, `#[comptime]`, `F::exp`) does not. |
| `crates/xcfun-ad/tests/props_*.rs` batch-per-property pattern | test | batch | D-18 batch-per-property is new to Phase 1. Closest precedent is RESEARCH.md Example 4 (per-test launch, non-batched). |
| `xtask/src/bin/regen_ad_fixtures.rs` | utility / fixture generation | file-I/O + C++ FFI | No xtask crate exists in the repo today. Cargo workspace has no `xtask` member. |
| `xtask/assets/regen_ad_fixtures/driver.cpp` | build (C++ driver) | file-I/O | Closest shape: `xcfun-master/external/upstream/taylor/unittest_taylor.cpp` — but that's a unit test, not a fixture emitter. Driver must be written from scratch. |

### Resolution for "no analog" files

The planner must add the following tasks to the plan wave structure:

1. **Wave 0 — cubecl spike task:** Write `for_tests/cpu_client.rs` + a trivial "add two numbers on cubecl-cpu" smoke test BEFORE any `#[cube] fn ctaylor_*` is ported. This is the pattern source for all subsequent `#[cube]` fns.
2. **Wave 0 — xtask scaffold:** Create `xtask/` workspace member with `cc`-based C++ driver build.
3. **Wave 2 — proptest batching research:** Investigate proptest 1.11's `new_tree`/`current()` API under the batch-per-property pattern before writing `props_ring.rs`.

---

## Metadata

**Analog search scope:**
- `/home/chemtech/workspace/xcfun_rs/crates/xcfun-ad/src/**` (pre-pivot code, reverted in Wave 0 but useful for operation-order transfer)
- `/home/chemtech/workspace/xcfun_rs/crates/xcfun-ad/benches/**` (Phase-1 criterion stubs)
- `/home/chemtech/workspace/xcfun_rs/crates/xcfun-{core,eval,ffi,functionals,gpu,python}/**` (all currently skeletons — no applicable patterns)
- `/home/chemtech/workspace/xcfun_rs/xcfun-master/external/upstream/taylor/**` (authoritative C++ reference — primary pattern source)
- `/home/chemtech/workspace/xcfun_rs/docs/design/**` (design docs — **reference**, not a code pattern source)
- `/home/chemtech/workspace/xcfun_rs/.cargo/config.toml` (preserved verbatim per D-02)

**Files scanned:** ~40 (6 existing Rust files in xcfun-ad src, 2 bench stubs, 7 C++ headers in taylor/, 1 Cargo.toml, 1 workspace Cargo.toml, 1 .cargo/config.toml, 13 design docs, 3 other crate lib.rs's).

**Pattern extraction date:** 2026-04-19

**Phase:** 01-taylor-algebra-ad-primitives-xcfun-ad

**Key architectural decisions reflected:**
- D-04: `CTaylor<F, N>` is a pure `#[cube]` type, NO host struct → no `#[derive]` on a host struct, no `Copy` impl.
- D-05: sealed `ValidN` trait retired → no `valid_n.rs` port.
- D-08: algorithmic-identity C++ port → primary analog is always the C++ header, even when pre-pivot Rust exists.
- D-11: `assert!` not `debug_assert!` → shared pattern SP-3.
- D-14: composed functions use scratch array + compose → shared pattern SP-4.
- D-17: `OnceLock<CpuClient>` in `for_tests::cpu_client()` → no in-repo analog, flagged for cubecl research.
- D-21: pre-pivot code reverted first → pre-pivot Rust is an **operation-order reference**, not a template to evolve.
- D-26: only `cpu` feature at `xcfun-ad` level → no `cuda`/`wgpu` deps in `xcfun-ad/Cargo.toml`.
