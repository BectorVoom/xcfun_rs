# 08 — Error model

`xcfun_rs` library crates use `thiserror` 2.x to define structured, typed errors. Application-boundary crates (`validation`, `benches`, `examples`, `xtask`) use `anyhow`. No library crate ever depends on `anyhow`; this is enforced by a `cargo xtask check-no-anyhow` rule in CI.

---

## 1. Dependency rules

| Crate kind | Allowed error crate |
|------------|---------------------|
| `xcfun-ad`, `xcfun-core`, `xcfun-kernels`, `xcfun-gpu`, `xcfun-rs`, `xcfun-capi`, `xcfun-py` (library crates) | `thiserror` 2.x |
| `validation`, `benches/*`, `xtask`, `examples/*` (application crates) | `anyhow` 1.x |

`cargo xtask check-no-anyhow` runs `cargo metadata --format-version=1` and walks the dependency tree, asserting no `anyhow` edge leads into a library crate. This rule is also encoded in `deny.toml`'s banned list for the library crates.

---

## 2. The library error type: `XcError`

`XcError` lives in `xcfun-core::error` and is re-exported by `xcfun-rs`. It is the return-error type for every fallible public function in all library crates. The C ABI converts it to integer codes (see §4).

```rust
// crate: xcfun-core
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum XcError {
    /// `order` is out of range, or incompatible with `mode`.
    /// Matches reference's `XC_EORDER = 1`.
    #[error("invalid differentiation order {order} for mode {mode:?}")]
    InvalidOrder { order: i32, mode: Mode },

    /// The selected `Vars` variant does not provide all ingredients the
    /// active functional set depends on. Matches `XC_EVARS = 2`.
    #[error("variables {vars:?} do not provide {missing:?} required by active functionals")]
    InvalidVars { vars: Vars, missing: Dependency },

    /// `mode` is not compatible with the active functional set
    /// (e.g. Potential mode for a metaGGA). Matches `XC_EMODE = 4`.
    #[error("mode {mode:?} incompatible with functional dependencies {depends:?}")]
    InvalidMode { mode: Mode, depends: Dependency },

    /// `name` does not resolve to any functional, parameter, or alias.
    #[error("unknown functional / parameter / alias name: {name}")]
    UnknownName { name: SmallString },

    /// The `density` slice has the wrong length.
    #[error("input slice has length {got}, expected {expected}")]
    InputLengthMismatch { got: usize, expected: usize },

    /// The `result` slice has the wrong length.
    #[error("output slice has length {got}, expected {expected}")]
    OutputLengthMismatch { got: usize, expected: usize },

    /// `eval_setup` (or `user_eval_setup`) has not been called.
    #[error("functional has not been configured: call eval_setup() first")]
    NotConfigured,

    /// `user_eval_setup` was called with a bit pattern that is out of range
    /// or does not map to a known `Vars`/`Mode`.
    #[error("invalid bitwise encoding {raw:#x} in user_eval_setup")]
    InvalidEncoding { raw: u32 },

    /// A GPU-specific runtime error (allocation, launch failure).
    /// Only surfaced by the `Batch` API.
    #[error("GPU runtime error in {context}: {detail}")]
    Runtime { context: &'static str, detail: &'static str },
}
```

### 2.1 `SmallString`

`SmallString` is a fixed-capacity (≤ 32 bytes, equal to the longest functional / alias name) stack-allocated string. `UnknownName` carries the offending name for diagnostic purposes without heap allocation in the common case. Implementation reuses `smartstring` if the `std` feature is enabled, otherwise uses a custom inline `[u8; 32]` type. This keeps `XcError` `Copy`.

### 2.2 `#[non_exhaustive]`

Added so we can extend the enum in future releases without breaking semver.

### 2.3 Constraints

- `XcError` is `Copy` (cheap to return by value).
- `XcError` implements `Send + Sync + 'static` (required for FFI catch-panic shims).
- No `source: Box<dyn Error>` chain — errors in this library have no nested cause; adding one would cost a heap allocation.

---

## 3. Error construction conventions

- Errors are constructed at the point of detection. No `From` impls convert unrelated errors into `XcError` (nothing should masquerade).
- Every variant is only constructed in one or two places; e.g. `InvalidOrder` only in `Functional::eval_setup`.
- Public entry points validate inputs first and return `Err` before any computation begins.
- Internal helpers return primitive types; no `Result<_, XcError>` propagates through the hot path except at the top level of `eval_setup`, `set`, `eval`, `eval_vec`.

---

## 4. Mapping to the C ABI

`xcfun-capi` exposes integer error codes identical to the reference (`xcfun-master/src/XCFunctional.hpp` lines 40-46):

```
XC_EORDER = 1
XC_EVARS  = 2
XC_EMODE  = 4
```

Mapping:

| `XcError` variant | C return code |
|-------------------|---------------|
| `InvalidOrder` | `1` |
| `InvalidVars` | `2` |
| `InvalidMode` | `4` |
| `InvalidVars | InvalidMode` (two-way incompatibility detected together) | `6` |
| `UnknownName` in `xcfun_set` | `-1` |
| `InputLengthMismatch`, `OutputLengthMismatch`, `NotConfigured`, `InvalidEncoding`, `Runtime` | Printed to stderr and `abort()` — matches the reference `xcfun::die` behaviour for these unrecoverable misuse cases |

The `as_c_code` helper in `xcfun-core`:

```rust
impl XcError {
    pub fn as_c_code(&self) -> i32 {
        match self {
            XcError::InvalidOrder { .. }                => 1,
            XcError::InvalidVars { .. }                 => 2,
            XcError::InvalidMode { .. }                 => 4,
            XcError::UnknownName { .. }                 => -1,
            XcError::InvalidEncoding { .. }
                | XcError::InputLengthMismatch { .. }
                | XcError::OutputLengthMismatch { .. }
                | XcError::NotConfigured
                | XcError::Runtime { .. }               => -1,
        }
    }
}
```

Two-way failures (C code `6 = 2 | 4`) are surfaced by a dedicated helper in `Functional::eval_setup`:

```rust
fn check_vars_and_mode(&self, vars: Vars, mode: Mode) -> Result<(), XcError> {
    let var_err = ...;   // Option<XcError::InvalidVars>
    let mode_err = ...;  // Option<XcError::InvalidMode>
    match (var_err, mode_err) {
        (Some(_), Some(_)) => Err(XcError::InvalidMode { mode, depends: self.depends }),
        // ... (a combined variant if we want bit 2|4; alternative: emit InvalidMode with
        //      the variant carrying both, per as_c_code docs; or use a BitFlags code directly)
    }
}
```

A pragmatic solution: `as_c_code` inspects the active-set's `depends` and the detected failure to emit `6` when both conditions apply, mirroring the reference.

---

## 5. Panic policy

| Location | Rule |
|----------|------|
| Library crates, public APIs | Never panic on any input the public API can construct. Return `Err(XcError)` or a well-typed value. |
| Library crates, internal helpers | May use `debug_assert!` for invariants (released builds remove the assertion). |
| Library crates, `unsafe` blocks | Must document the invariant that forbids a panic. |
| `xcfun-capi` | `catch_unwind` shim around every `extern "C"` function; a panic is printed to stderr and `abort()` is called, mirroring `xcfun::die`. |
| `xcfun-py` | PyO3's default `catch_unwind`; panics become Python `PanicException`. |
| `validation`, `xtask` | Panics allowed (application crates). |

### 5.1 `xcfun-capi` panic shim

```rust
macro_rules! c_entry {
    ($body:block) => {{
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || $body)) {
            Ok(result) => result,
            Err(_)     => {
                eprintln!("xcfun: panic in C ABI, aborting");
                std::process::abort();
            }
        }
    }};
}

#[no_mangle]
pub unsafe extern "C" fn xcfun_eval_setup(fun: *mut xcfun_t,
                                          vars: xcfun_vars,
                                          mode: xcfun_mode,
                                          order: i32) -> i32 {
    c_entry!({
        let fun = &mut (*fun).0;
        match fun.eval_setup(vars.into(), mode.into(), order as u32) {
            Ok(())  => 0,
            Err(e)  => e.as_c_code(),
        }
    })
}
```

Every `extern "C"` function uses `c_entry!`. There is no way for a Rust panic to unwind into C code.

---

## 6. Application-boundary error handling (`validation` etc.)

`anyhow` is used for:
- Top-level `main` in `validation`, `xtask`, examples.
- Internal helpers where the caller does not need to pattern-match on error variants (report writing, CLI parsing, temp-file handling).

Conversion from `XcError` to `anyhow::Error` is automatic via `std::error::Error` bound (supplied by `thiserror`). Example:

```rust
// validation/src/main.rs
use anyhow::{Context, Result};
use xcfun_rs::{Functional, Vars, Mode};

fn main() -> Result<()> {
    let mut fun = Functional::new();
    fun.set("blyp", 1.0).context("configure blyp")?;
    fun.eval_setup(Vars::ABGaaGabGbb, Mode::PartialDerivatives, 2)
        .context("eval_setup blyp")?;
    // ...
    Ok(())
}
```

No `anyhow::Error` crosses a library-crate boundary.

---

## 7. Logging and diagnostics

- Production code uses `tracing` (feature-gated behind `feature = "tracing"`) for structured events at Info/Warn/Error levels. Events are emitted at batch boundaries, not on the per-point hot path.
- Debug diagnostics (dumping `CTaylor` coefficients during parity debugging) live in `validation/` and emit at the `trace` level.
- `eprintln!` is used only by the `c_entry!` panic shim.

---

## 8. Summary

- `thiserror` for every library error; `XcError` is `Copy`, `Send`, `Sync`, `'static`, and carries no heap.
- `anyhow` for applications only; enforced by CI.
- Panic policy: impossible through the public API; `catch_unwind`-aborted at the FFI boundary.
- C ABI integer codes mirror the reference (1, 2, 4, 6, -1).
- Errors are constructed where detected; no `From` chains obscure origin.
