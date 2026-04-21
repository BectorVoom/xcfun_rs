# 03 — API surface

This document specifies every public API of `xcfun_rs` in three surfaces, each mapped one-to-one with an entry in the reference C header `xcfun-master/api/xcfun.h`. Every row gives the exact signature, preconditions, postconditions, error conditions, and a minimal usage example.

1. Native Rust API (`xcfun-rs`), preferred for Rust callers.
2. C ABI (`xcfun-capi`), drop-in replacement for the C++ `libxcfun`.
3. Python API (`xcfun-py`), PyO3-backed.

All three share a single internal implementation in `xcfun-core` + `xcfun-gpu`.

The Rust API returns `Result<_, XcError>` from every fallible entry. The C ABI matches the reference's integer error return codes exactly. The Python API converts `XcError` to a Python `XcfunError` exception.

---

## 1. Native Rust API

Crate: `xcfun-rs`. Module path: crate root unless otherwise noted.

### 1.1 Compile-time constants

| Rust | C counterpart | Value |
|------|---------------|-------|
| `pub const API_VERSION: u32 = 2;` | `XCFUN_API_VERSION` | 2 |
| `pub const MAX_ORDER: usize = 6;` | `XCFUN_MAX_ORDER` | 6 |
| `pub const NR_FUNCTIONALS: usize = 78;` | `XC_NR_FUNCTIONALS` | 78 |
| `pub const NR_VARS: usize = 31;` | `XC_NR_VARS` | 31 |
| `pub const MAX_ALIASES: usize = 60;` | `XC_MAX_ALIASES` | 60 |
| `pub const MAX_ALIAS_TERMS: usize = 10;` | `MAX_ALIAS_TERMS` | 10 |

### 1.2 Enumerations

| Rust | C counterpart |
|------|---------------|
| `pub enum Mode { Unset=0, PartialDerivatives=1, Potential=2, Contracted=3 }` | `xcfun_mode` |
| `pub enum Vars { Unset=-1, A=0, N=1, AB=2, NS=3, … } (31 variants)` | `xcfun_vars` |

See [02-data-structures.md](02-data-structures.md) §3 and §4 for the full variant list and discriminant mapping.

### 1.3 Error type

```rust
// crate: xcfun-core  (re-exported by xcfun-rs)
#[derive(thiserror::Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum XcError {
    #[error("invalid differentiation order {order} for mode {mode:?}")]
    InvalidOrder { order: i32, mode: Mode },
    #[error("variables {vars:?} do not satisfy dependencies {required:?}")]
    InvalidVars { vars: Vars, required: Dependency },
    #[error("mode {mode:?} incompatible with functional set {depends:?}")]
    InvalidMode { mode: Mode, depends: Dependency },
    #[error("unknown functional / parameter / alias name: {0}")]
    UnknownName(alloc::string::String),
    #[error("input slice has length {got}, expected {expected}")]
    InputLengthMismatch { got: usize, expected: usize },
    #[error("output slice has length {got}, expected {expected}")]
    OutputLengthMismatch { got: usize, expected: usize },
    #[error("functional not configured: call eval_setup() first")]
    NotConfigured,
    #[error("invalid bitwise encoding in user_eval_setup ({0:#x})")]
    InvalidEncoding(u32),
}
```

Bitwise correspondence to the C constants (`XC_EORDER=1`, `XC_EVARS=2`, `XC_EMODE=4`):

```rust
impl XcError {
    pub fn as_c_code(&self) -> i32 {
        match self {
            XcError::InvalidOrder { .. } => 1,
            XcError::InvalidVars { .. } => 2,
            XcError::InvalidMode { .. } => 4,
            XcError::InvalidVars { .. } | XcError::InvalidMode { .. } => 6, // handled above
            _ => -1,
        }
    }
}
```

### 1.4 `Functional` — the primary handle

| Method | Signature |
|--------|-----------|
| `new` | `pub fn new() -> Self` |
| `set` | `pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError>` |
| `get` | `pub fn get(&self, name: &str) -> Result<f64, XcError>` |
| `is_gga` | `pub fn is_gga(&self) -> bool` |
| `is_metagga` | `pub fn is_metagga(&self) -> bool` |
| `eval_setup` | `pub fn eval_setup(&mut self, vars: Vars, mode: Mode, order: u32) -> Result<(), XcError>` |
| `user_eval_setup` | `pub fn user_eval_setup(&mut self, order: u32, func_type: u32, dens_type: u32, mode_type: u32, laplacian: bool, kinetic: bool, current: bool, explicit_derivatives: bool) -> Result<(), XcError>` |
| `input_length` | `pub fn input_length(&self) -> usize` |
| `output_length` | `pub fn output_length(&self) -> usize` |
| `eval` | `pub fn eval(&self, density: &[f64], out: &mut [f64]) -> Result<(), XcError>` |
| `eval_vec` | `pub fn eval_vec(&self, density: &[f64], density_pitch: usize, out: &mut [f64], out_pitch: usize, nr_points: usize) -> Result<(), XcError>` |
| `batch_on` | `pub fn batch_on<R: cubecl::Runtime>(&self, client: R::Client) -> Batch<'_, R>` |

#### 1.4.1 `Functional::new`

| Property | Value |
|----------|-------|
| Preconditions | None |
| Postconditions | Returned value has `vars = Unset`, `mode = Unset`, `order = -1`, no active functionals, all parameters initialised to their `default_value` |
| Errors | None |
| Allocation | One `Box<Functional>` (not exposed). After that, no heap allocations on the eval path |

#### 1.4.2 `Functional::set`

| Property | Value |
|----------|-------|
| Preconditions | `name` is (case-insensitively) one of: the 78 functional names, the 4 parameter names, or a registered alias name |
| Postconditions | If `name` is a functional, `settings[id] += value` and, if not already active, appends to `active_ids`. If a parameter, `settings[param_id] = value`. If an alias, recursively sets each term with weight multiplied by `value` |
| Errors | `XcError::UnknownName(name.into())` when no match; `XcError::InvalidEncoding` propagated if an alias recursion fails |
| Allocation | `UnknownName` branch allocates a `String`; success path is zero-alloc |

Usage:

```rust
use xcfun_rs::{Functional, Mode, Vars};
let mut fun = Functional::new();
fun.set("slaterx", 1.0)?;
fun.set("vwn5c", 1.0)?;                  // LDA (Slater + VWN5) manually
fun.set("b3lyp", 1.0)?;                  // or use the alias — same effect
fun.set("exx", 0.2)?;                    // parameter
```

#### 1.4.3 `Functional::get`

| Preconditions | `name` resolves to a functional or a parameter (not an alias) |
| Postconditions | Returns the current value from `settings[]` |
| Errors | `UnknownName` for aliases or unknown names |

#### 1.4.4 `Functional::is_gga` / `is_metagga`

| Postconditions | `is_gga` returns `depends.contains(Dependency::GRADIENT)`; `is_metagga` returns `depends.intersects(Dependency::LAPLACIAN | Dependency::KINETIC)` |
| Errors | None |

#### 1.4.5 `Functional::eval_setup`

| Preconditions | At least one functional has been activated via `set` (otherwise `input_length()` is still defined but `eval` will yield zero) |
| Postconditions | Stores `vars`, `mode`, `order`; recomputes and caches `input_len_cached`, `output_len_cached` |
| Errors | |
| — `XcError::InvalidOrder` | `order > MAX_ORDER`, or `mode == PartialDerivatives && order > 4` |
| — `XcError::InvalidVars` | `(fun.depends & vars_table[vars].provides) != fun.depends` |
| — `XcError::InvalidMode` | GGA-only potential requires `_2ND_TAYLOR` vars; no potential mode for metaGGA |

Matches the C++ logic in `xcfun-master/src/XCFunctional.cpp` lines 426-452 exactly.

#### 1.4.6 `Functional::user_eval_setup`

Thin wrapper that converts `(func_type, dens_type, …)` to `Vars` (via `which_vars`) and `mode_type` to `Mode` (via `which_mode`), then calls `eval_setup`. Error conditions inherit from both.

#### 1.4.7 `Functional::input_length`

| Preconditions | `vars != Unset` |
| Postconditions | Returns `VARS_TABLE[vars].len as usize` (1..=20) |
| Errors | None — panics in debug if called before `vars` is set; in release returns 0 |

#### 1.4.8 `Functional::output_length`

| Preconditions | `vars != Unset && mode != Unset && order >= 0` |
| Postconditions | |
| — For `Mode::PartialDerivatives` | `taylor_len(input_length, order)` = C(n+k, k) binomial coefficient |
| — For `Mode::Potential` | 2 if `input_length ∈ {1, 10}`, else 3 |
| — For `Mode::Contracted` | `1 << order` |
| Errors | `XcError::NotConfigured` if any of the three fields is unset |

#### 1.4.9 `Functional::eval`

```rust
pub fn eval(&self, density: &[f64], out: &mut [f64]) -> Result<(), XcError>;
```

| Preconditions | `eval_setup` has been called; `density.len() == self.input_length()`; `out.len() == self.output_length()` |
| Postconditions | `out[0..output_length()]` holds the weighted sum of active functional contributions in the layout described in §1.4.11 |
| Errors | `InputLengthMismatch`, `OutputLengthMismatch`, `NotConfigured` |
| Allocations | None |
| Per-point runtime | Proportional to `nr_active_functionals × output_length`; no branches inside the AD core beyond the functional id dispatch |

#### 1.4.10 `Functional::eval_vec`

Identical contract to `eval`, applied `nr_points` times with user-supplied pitches. Dispatches to a CPU kernel via `cubecl::CpuRuntime` by default; if the caller opts into a GPU backend via `batch_on`, uses that.

```rust
pub fn eval_vec(&self,
                density: &[f64], density_pitch: usize,
                out: &mut [f64], out_pitch: usize,
                nr_points: usize) -> Result<(), XcError>;
```

| Preconditions | `density.len() >= density_pitch * nr_points && density_pitch >= input_length()`; `out.len() >= out_pitch * nr_points && out_pitch >= output_length()` |
| Errors | `InputLengthMismatch`, `OutputLengthMismatch`, `NotConfigured` |
| Allocations | None (uses a CPU-resident cubecl runtime with zero-copy bindings into the caller's slices) |
| Parallelism | The CPU backend parallelises with `std::thread::scope` when `nr_points * input_length > 16384` |

#### 1.4.11 Output layout for `Mode::PartialDerivatives`

For `input_length = n` and `order = k`, the output is the full Taylor expansion in xcfun's canonical order. Mapping from a multi-index `(i_1, i_2, …, i_k)` with `i_1 ≤ i_2 ≤ … ≤ i_k` (sorted weakly) to an offset in `out[]` is defined by the natural Taylor-coefficient enumeration:

```
offset(i_1, …, i_k) = Σ_{j=0..k} taylor_len(n, j) +  <position within k-th block>
```

where the "position within k-th block" is the lexicographic rank of the weakly-increasing tuple. `taylor_len(n, k) = C(n+k, n) − C(n+k−1, n)`. This is identical to the `output[k++]` assignments at `xcfun-master/src/XCFunctional.cpp` lines 568-605.

| order | layout |
|-------|--------|
| 0 | `[E]` |
| 1 | `[E, ∂_1, ∂_2, …, ∂_n]` |
| 2 | `[E, ∂_1, …, ∂_n, ∂_{11}, ∂_{12}, …, ∂_{1n}, ∂_{22}, …, ∂_{nn}]` (upper triangular) |
| 3 | Order-2 prefix, then `∂_{111}, ∂_{112}, …, ∂_{nnn}` lexicographic |

Bit-flag → index-triple correspondence is computed once per input by the dispatcher; it is not exposed to users.

#### 1.4.12 `Functional::batch_on`

```rust
pub fn batch_on<R: cubecl::Runtime>(&self, client: R::Client) -> Batch<'_, R>;
```

Opens a GPU or CPU batch; see [06-cubecl-strategy.md](06-cubecl-strategy.md).

### 1.5 Free functions (text / registry)

| Rust | C counterpart | Notes |
|------|---------------|-------|
| `pub fn version() -> &'static str` | `xcfun_version` | Compile-time string |
| `pub fn splash() -> &'static str` | `xcfun_splash` | Static, 7 lines |
| `pub fn authors() -> &'static str` | `xcfun_authors` | Static |
| `pub fn self_test() -> u32` | `xcfun_test` | Returns number of failing functionals |
| `pub fn is_compatible_library() -> bool` | `xcfun_is_compatible_library` | Compares major version |
| `pub fn which_vars(func_type: u32, dens_type: u32, laplacian: bool, kinetic: bool, current: bool, explicit_derivatives: bool) -> Result<Vars, XcError>` | `xcfun_which_vars` | Returns `XcError::InvalidEncoding` for malformed bit patterns |
| `pub fn which_mode(mode_type: u32) -> Result<Mode, XcError>` | `xcfun_which_mode` | |
| `pub fn enumerate_parameters(param: i32) -> Option<&'static str>` | `xcfun_enumerate_parameters` | Range 0..=81 |
| `pub fn enumerate_aliases(n: i32) -> Option<&'static str>` | `xcfun_enumerate_aliases` | Range 0..MAX_ALIASES |
| `pub fn describe_short(name: &str) -> Option<&'static str>` | `xcfun_describe_short` | Case-insensitive |
| `pub fn describe_long(name: &str) -> Option<&'static str>` | `xcfun_describe_long` | Case-insensitive |

### 1.6 `Batch<'fun, R>` — GPU lifecycle

```rust
pub struct Batch<'fun, R: cubecl::Runtime> { /* see 02-data-structures.md §9 */ }

impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
    pub fn reserve(&mut self, nr_points: usize) -> Result<(), XcError>;
    pub fn upload_density(&mut self, host: &[f64], pitch: usize) -> Result<(), XcError>;
    pub fn launch(&mut self, nr_points: usize) -> Result<(), XcError>;
    pub fn download_result(&self, host: &mut [f64], pitch: usize) -> Result<(), XcError>;
    pub fn eval_vec_host(&mut self,
                          density: &[f64], density_pitch: usize,
                          out: &mut [f64], out_pitch: usize,
                          nr_points: usize) -> Result<(), XcError>;
}
```

See [06-cubecl-strategy.md](06-cubecl-strategy.md) for the detailed contract of the batch API.

### 1.7 Minimal usage example

```rust
use xcfun_rs::{Functional, Mode, Vars, XcError};

fn main() -> Result<(), XcError> {
    let mut fun = Functional::new();
    fun.set("blyp", 1.0)?;                        // alias: beckex + lypc
    fun.eval_setup(Vars::ABGaaGabGbb, Mode::PartialDerivatives, 1)?;

    let input = [0.39, 0.38, 0.1, 0.05, 0.1];     // alpha, beta, gaa, gab, gbb
    let mut output = vec![0.0; fun.output_length()];

    fun.eval(&input, &mut output)?;
    // output[0] = energy; output[1..=5] = first derivatives in canonical layout
    Ok(())
}
```

---

## 2. C ABI

Crate: `xcfun-capi`. All declarations in `include/xcfun.h` generated by `cbindgen`. Every symbol in `xcfun-master/api/xcfun.h` is implemented; the Rust side forwards to the native API.

### 2.1 Handle

```c
struct xcfun_s;
typedef struct xcfun_s xcfun_t;
```

Rust: `#[repr(transparent)] pub struct xcfun_t(xcfun_rs::Functional);`. Box-allocated by `xcfun_new`, freed by `xcfun_delete`.

### 2.2 Function table (bytewise compatible)

| C declaration | Rust implementation |
|---------------|---------------------|
| `const char *xcfun_version(void)` | returns `xcfun_rs::version().as_ptr() as *const c_char` (NUL-terminated static) |
| `const char *xcfun_splash(void)` | ditto |
| `const char *xcfun_authors(void)` | ditto |
| `int xcfun_test(void)` | `xcfun_rs::self_test() as i32` |
| `bool xcfun_is_compatible_library(void)` | direct |
| `xcfun_vars xcfun_which_vars(unsigned func_type, …)` | forwards; `InvalidEncoding` → aborts with a message (matches `xcfun::die` behavior in the reference) |
| `xcfun_mode xcfun_which_mode(unsigned mode_type)` | ditto |
| `const char *xcfun_enumerate_parameters(int param)` | returns NULL for out-of-range |
| `const char *xcfun_enumerate_aliases(int n)` | ditto |
| `const char *xcfun_describe_short(const char *name)` | returns NULL for unknown |
| `const char *xcfun_describe_long(const char *name)` | ditto |
| `xcfun_t *xcfun_new(void)` | `Box::into_raw(Box::new(xcfun_t(Functional::new())))` |
| `void xcfun_delete(xcfun_t *fun)` | `drop(unsafe { Box::from_raw(fun) })`; NULL-safe |
| `int xcfun_set(xcfun_t *fun, const char *name, double value)` | `0` on success, `-1` on `UnknownName`, matching reference |
| `int xcfun_get(const xcfun_t *fun, const char *name, double *value)` | same |
| `bool xcfun_is_gga(const xcfun_t *fun)` | direct |
| `bool xcfun_is_metagga(const xcfun_t *fun)` | direct |
| `int xcfun_eval_setup(xcfun_t *fun, xcfun_vars vars, xcfun_mode mode, int order)` | returns `XcError::as_c_code()` on failure, 0 on success |
| `int xcfun_user_eval_setup(xcfun_t *fun, int order, unsigned func_type, …)` | same |
| `int xcfun_input_length(const xcfun_t *fun)` | returns the cached length, or calls `xcfun::die` if unset |
| `int xcfun_output_length(const xcfun_t *fun)` | same |
| `void xcfun_eval(const xcfun_t *fun, const double density[], double result[])` | calls `Functional::eval`, aborts the process on length mismatch (matches reference's assumption that caller passes correctly sized arrays) |
| `void xcfun_eval_vec(const xcfun_t *fun, int nr_points, const double *density, int density_pitch, double *result, int result_pitch)` | calls `Functional::eval_vec` |

### 2.3 ABI guarantees

- All `enum` sizes match the reference. `xcfun_mode`: `unsigned` (4 bytes). `xcfun_vars`: `int` (4 bytes). Enforced by `#[repr(u32)]` and `#[repr(i32)]` in the Rust `Vars` and `Mode` newtypes in `xcfun-capi`.
- `xcfun_t` is opaque on the C side; the Rust side never exposes `Functional` internals through the header.
- The generated `xcfun.h` is diff-checked against `xcfun-master/api/xcfun.h` in CI; intentional deviations (e.g., attribute annotations) are stored in `xcfun-capi/cbindgen.toml` `after_includes`.

### 2.4 Error mapping

| Rust `XcError` | C return value |
|----------------|----------------|
| `InvalidOrder` | `1` (`XC_EORDER`) |
| `InvalidVars` | `2` (`XC_EVARS`) |
| `InvalidMode` | `4` (`XC_EMODE`) |
| `InvalidVars & InvalidMode` (GGA potential without `_2ND_TAYLOR`) | `6` (`XC_EVARS \| XC_EMODE`) |
| `UnknownName` in `xcfun_set` | `-1` |
| `NotConfigured` etc. | Aborts with `xcfun::die`-equivalent, matching reference runtime behavior |

### 2.5 Example (C)

```c
#include "xcfun.h"
int main() {
    xcfun_t *fun = xcfun_new();
    xcfun_set(fun, "BLYP", 1.0);
    xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB, XC_PARTIAL_DERIVATIVES, 1);

    int n = xcfun_output_length(fun);
    double in[] = {0.39, 0.38, 0.1, 0.05, 0.1};
    double *out = malloc(n * sizeof(double));
    xcfun_eval(fun, in, out);

    free(out);
    xcfun_delete(fun);
    return 0;
}
```

---

## 3. Python API

Crate: `xcfun-py`. Module: `xcfun_rs`. Built with `maturin develop`.

### 3.1 Classes

```python
class Functional:
    def __init__(self) -> None: ...
    def set(self, name: str, value: float) -> None: ...
    def get(self, name: str) -> float: ...
    def is_gga(self) -> bool: ...
    def is_metagga(self) -> bool: ...
    def eval_setup(self, vars: int, mode: int, order: int) -> None: ...
    def user_eval_setup(self, order: int, func_type: int, dens_type: int,
                        mode_type: int, laplacian: bool, kinetic: bool,
                        current: bool, explicit_derivatives: bool) -> None: ...
    def input_length(self) -> int: ...
    def output_length(self) -> int: ...
    def eval(self, density: np.ndarray) -> np.ndarray: ...           # 1-D f64
    def eval_vec(self, density: np.ndarray) -> np.ndarray: ...       # 2-D (N, input_length)

class XcfunError(Exception): ...
```

### 3.2 Free functions

```python
def version() -> str: ...
def splash() -> str: ...
def authors() -> str: ...
def self_test() -> int: ...
def is_compatible_library() -> bool: ...
def which_vars(func_type: int, dens_type: int,
               laplacian: bool, kinetic: bool,
               current: bool, explicit_derivatives: bool) -> int: ...
def which_mode(mode_type: int) -> int: ...
def enumerate_parameters(param: int) -> str | None: ...
def enumerate_aliases(n: int) -> str | None: ...
def describe_short(name: str) -> str | None: ...
def describe_long(name: str) -> str | None: ...
```

### 3.3 Zero-copy array contract

`eval_vec(density)` accepts a 2-D `numpy.ndarray[np.float64, order='C']` with shape `(n_points, input_length)` and returns a 2-D array with shape `(n_points, output_length)`. The returned array wraps a Rust-owned buffer using `rust-numpy`'s `PyArray2`. No copy is made on ingress or egress.

### 3.4 Minimal example (Python)

```python
import numpy as np
import xcfun_rs

fun = xcfun_rs.Functional()
fun.set("blyp", 1.0)
fun.eval_setup(xcfun_rs.Vars.ABGaaGabGbb, xcfun_rs.Mode.PartialDerivatives, 1)

n_points = 10_000
density = np.random.default_rng(0).uniform(0.1, 1.0, (n_points, fun.input_length()))
out = fun.eval_vec(density)  # shape (n_points, fun.output_length())
```

---

## 4. API coverage guarantee

CI runs `crates/xcfun-rs/tests/api_coverage.rs`, which invokes every public Rust function at least once, and `crates/xcfun-capi/tests/headers_match.rs`, which diffs the cbindgen-generated `xcfun.h` against `xcfun-master/api/xcfun.h`. Merge to `main` is blocked if either fails.
