# Phase 5: Rust Facade (`xcfun-rs`) + C ABI (`xcfun-capi`) — Pattern Map

**Mapped:** 2026-04-30
**Files analyzed:** 14 new + 6 modified (per CONTEXT D-01..D-17)
**Analogs found:** 17 / 14 new (every new file maps to ≥1 analog; three originate fresh patterns: the `c_entry!` panic-trap macro, the cbindgen `build.rs`, and the headers-match diff harness)

Phase 5 is a **facade + drop-in-C-ABI** phase — no new numerical kernels, no new functional bodies, no GPU work. Every Rust-side analog already exists somewhere in `crates/xcfun-eval/{functional,dispatch,density_vars}.rs`, `crates/xcfun-core/{enums,error,traits}.rs`, and `xtask/src/bin/*.rs`. Only three patterns are wholly new to the project:

- the `c_entry!` macro (no precedent — design specified in CONTEXT D-05/D-07; `std::panic::catch_unwind` + stderr + `abort`)
- the cbindgen build/regen flow (no `build.rs` in any `crates/xcfun-*/`; only `validation/build.rs` exists, which is a `cc` driver — not cbindgen)
- the C-side golden test driving `cc::Build` from a `tests/` directory (validation/build.rs is closest, but it compiles a 100+ source library, not a 1-file driver)

The C++ reference for every C-ABI body is `xcfun-master/src/XCFunctional.cpp` lines 48-868, with `xcfun-master/api/xcfun.h` as the symbol-set spec and `xcfun-master/src/functional.hpp` as the panic-policy reference.

---

## File Classification

### New files (14)

#### A. `xcfun-rs` — native Rust facade (4 new files)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-rs/Cargo.toml` | crate-manifest | static | `crates/xcfun-eval/Cargo.toml` (multi-feature library; deps on xcfun-core + xcfun-ad) | exact |
| `crates/xcfun-rs/src/lib.rs` | facade-module-root + free fns | request-response (delegation) | `crates/xcfun-core/src/lib.rs` (re-export root with `pub use enums::{...}`) + `crates/xcfun-eval/src/lib.rs` (`pub mod` declarations) | role-match (Phase 5 D-02 wraps inner `Functional`; no existing wrapper-struct analog in repo — the Phase-2-D-21 `xcfun-eval::Functional` is the wrapped type) |
| `crates/xcfun-rs/tests/zero_alloc.rs` | invariant test (allocator counter on hot path) | request-response | `crates/xcfun-eval/tests/regularize_invariant.rs` (launch + readback + assert; gated behind `#[cfg(feature = "testing")]`) | role-match (counting `#[global_allocator]` is a NEW shape; closest analog is the snapshot-and-diff style of `regularize_invariant.rs`) |
| `crates/xcfun-rs/tests/send_sync.rs` | compile-time invariant test | static | `crates/xcfun-core/src/error.rs:55-59` (`assert_impl_all!(XcError: Copy, Clone, Send, Sync, ...)`) | exact |

#### B. `xcfun-capi` — C ABI drop-in (8 new files; replaces stub `crates/xcfun-ffi/`)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-capi/Cargo.toml` (rename of `xcfun-ffi/Cargo.toml`) | crate-manifest with `crate-type = ["cdylib", "staticlib", "rlib"]` | static | `crates/xcfun-ffi/Cargo.toml` (current 10-line stub — provides the rename target; no existing crate in repo declares `cdylib`/`staticlib`) | role-match (3-output crate-type is novel; existing `crates/*/Cargo.toml` are all rlib-only) |
| `crates/xcfun-capi/src/lib.rs` | C-ABI-entry-points + `#[no_mangle] extern "C"` exports | request-response (FFI gateway) | `validation/src/ffi.rs:14-24` (`unsafe extern "C"` declarations consuming xcfun's C ABI; **inverse direction** but same symbol set) | role-match (we're producing the symbols `validation/src/ffi.rs` consumes; symbol list is identical) |
| `crates/xcfun-capi/src/c_entry.rs` (or `src/macros.rs` per Claude's Discretion) | macro-defining-module | declarative | NONE in repo — `c_entry!` is a Phase-5-original pattern | no-analog (**source spec:** `xcfun-master/src/functional.hpp:1-27` `xcfun::die` + Rust idiom `std::panic::catch_unwind(AssertUnwindSafe(\|\| body))`) |
| `crates/xcfun-capi/build.rs` | build-time-codegen (cbindgen header emission — runs only via `xtask`, NOT as a `[build-dependency]`; this file is a no-op shim or absent — see D-09) | build-time | `validation/build.rs:1-226` (cc-driven C++ compile) | role-match for build-script structure; cbindgen invocation pattern is novel |
| `crates/xcfun-capi/cbindgen.toml` | config-file | declarative | NONE in repo | no-analog (**source spec:** cbindgen 0.29.2 quick-start; D-11/D-12 fix `documentation = false`, `function.prefix = "XCFun_API"`, prelude block defining `XCFun_API`) |
| `crates/xcfun-capi/include/xcfun.h` (auto-generated, committed) | generated-header | static | `xcfun-master/api/xcfun.h:1-390` (the diff target itself) | exact (this file IS the drop-in spec) |
| `crates/xcfun-capi/tests/headers_match.rs` | drift-gate test (diff generated vs upstream header) | request-response (file-read + normalize + diff) | `xtask/src/bin/regen_registry.rs:24-30` (`.sha256` stamp + `--check` drift gate — **structurally similar** but operates over committed Rust source files) | role-match |
| `crates/xcfun-capi/tests/c_abi.rs` | C-driver-compile-and-link integration test | request-response (cc compile + link staticlib + exec test binary) | `validation/build.rs:69-224` (cc::Build driving C++ compile against xcfun-master headers + linking) | role-match (the build.rs analog drives a 100+-file lib compile; ours is a 1-file driver linking against `libxcfun_capi.a`) |
| `crates/xcfun-capi/tests/c_abi.c` | C-driver source | static (data-tuples + assertions) | NONE in repo (this is the first hand-written C source in the project) | no-analog (**source spec:** D-14 fixture table — 10 reference-driven tuples; expected outputs computed once via Rust `Functional::eval`) |

#### C. `XcError::as_c_code` mapping (1 new method on existing enum)

| New Code | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| New `impl XcError { pub fn as_c_code(&self) -> i32 { ... } }` block in `crates/xcfun-core/src/error.rs` | enum-method-mapping | transform | `crates/xcfun-core/src/enums.rs:39-71` `impl ParameterId` (`from_name` + `default_value` matching by variant — same shape: enum → primitive lookup) | exact |

#### D. xtask additions (1 new binary)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `xtask/src/bin/regen_capi_header.rs` | xtask drift-gate binary (cbindgen invocation + `--check` mode) | request-response (read crate AST → emit header → compare via SHA stamp) | `xtask/src/bin/regen_registry.rs:1-80` (compile extractor + JSONL emit + write Rust source + sha256 stamp + `--check` drift) | exact (D-09: "mirrors Phase 2 D-21 `regen-registry --check` pattern") |

### Modified files (6)

| File | Change | Pattern Source |
|------|--------|----------------|
| `Cargo.toml` (workspace) | Remove `crates/xcfun-ffi` from `exclude`; remove `crates/xcfun-functionals` outright (D-04); add `crates/xcfun-rs` and `crates/xcfun-capi` to `members` | existing `members = ["crates/xcfun-ad", "crates/xcfun-core", "crates/xcfun-eval", "xtask", "validation"]` and `exclude = [...]` lines |
| `crates/xcfun-core/src/error.rs` | Add `impl XcError { pub fn as_c_code(&self) -> i32 { ... } }` per D-08-A; add unit tests for the 6 mappings | `crates/xcfun-core/src/error.rs:54-91` (existing test module pattern) |
| `crates/xcfun-eval/src/functional.rs` | Update `eval_setup` to detect-both `InvalidVars + InvalidMode` per D-08-A (currently returns one OR the other; needs to accumulate so `as_c_code` returns 6) | `xcfun-master/src/XCFunctional.cpp:442` `return xcfun::XC_EVARS \| xcfun::XC_EMODE` |
| `crates/xcfun-ffi/` directory | RENAMED via `git mv` to `crates/xcfun-capi/` per D-01; package name in `Cargo.toml` updated to `xcfun-capi` | D-01 mechanical rename |
| `crates/xcfun-functionals/` directory | DELETED per D-04 (dead post-cubecl-pivot stub) | D-04 outright deletion |
| `xtask/src/main.rs` | Add `regen-capi-header` to subcommand dispatch (currently only routes to `regen-ad-fixtures`) | `xtask/src/main.rs:7-22` existing match dispatch |

---

## Pattern Assignments

### A.1 — `crates/xcfun-rs/Cargo.toml` (NEW)

**Closest analog:** `crates/xcfun-eval/Cargo.toml:1-29` (multi-feature library crate with `xcfun-core` + cubecl deps).

**Imports / structure pattern (xcfun-eval/Cargo.toml:1-15):**

```toml
[package]
name = "xcfun-rs"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "Native Rust public API for xcfun_rs (Functional struct + free fns)"
license = "MPL-2.0"

[features]
default = []
# RS-08 GPU dispatch lives in Phase 6; Phase 5 ships a Functional::eval_vec stub.

[dependencies]
xcfun-core = { path = "../xcfun-core" }
xcfun-eval = { path = "../xcfun-eval" }
thiserror = { workspace = true }   # XcError re-export

[dev-dependencies]
static_assertions = "=1.1.0"       # send_sync.rs + zero_alloc.rs
```

**Differences planner must specify:**
- D-15-A: `[lib] crate-type = ["rlib"]` (default — facade is Rust-only).
- NO `cubecl` dep at facade level (CONTEXT "Integration Points": kernel substrate stays in `xcfun-eval`; xcfun-rs is pure delegation).
- Decline to depend on `xcfun-functionals` (D-04 deletes it).

**Cross-references:** CONTEXT D-02, D-15-A; ROADMAP Phase 5 success criterion 1.

---

### A.2 — `crates/xcfun-rs/src/lib.rs` (NEW)

**Closest analog (re-export module-root shape):** `crates/xcfun-core/src/lib.rs:1-29`.

**Imports + module-doc pattern (xcfun-core/src/lib.rs:1-29):**

```rust
//! xcfun-rs — native Rust public API for the xcfun exchange-correlation
//! functional library.
//!
//! Phase 5 facade per RS-01..10 (CONTEXT D-02). Wraps `xcfun_eval::Functional`
//! through a `pub struct Functional(xcfun_eval::Functional)` newtype so the
//! public-API surface is decoupled from internal cubecl details — `xcfun-eval`
//! can refactor its kernel layout without breaking RS-API doc/signature
//! contracts.
//!
//! # Module layout
//! - `Functional` — wrapper struct + methods (D-02).
//! - free functions — `version`, `splash`, `authors`, `self_test`,
//!   `is_compatible_library`, `which_vars`, `which_mode`,
//!   `enumerate_parameters`, `enumerate_aliases`, `describe_short`,
//!   `describe_long` (RS-04 + RS-05 + RS-06).
//!
//! # Send + Sync (RS-10, locked by D-17)
//! `Functional` is `Send + Sync` by structural design — no `static mut`,
//! `thread_local!`, or `Mutex`/`RwLock`. Compile-time gate in
//! `tests/send_sync.rs`.

#![forbid(unsafe_code)]

pub use xcfun_core::{Mode, ParameterId, Vars, XcError, FunctionalId};
```

**Wrapper-struct delegation pattern (D-02; no existing repo analog — propose):**

```rust
/// Public Rust facade over `xcfun_eval::Functional`. RS-01..10.
///
/// Decouples the public API from cubecl internals: `xcfun-eval` may refactor
/// its kernel layout without breaking the RS-API contracts here.
pub struct Functional(xcfun_eval::Functional);

impl Functional {
    /// Construct a fresh `Functional` with default settings (parameter slots
    /// 78..=81 seeded from `common_parameters.cpp:17-29`). Mirrors
    /// `XCFunctional::XCFunctional()` per RS-02.
    pub fn new() -> Self { Self(xcfun_eval::Functional::new()) }

    /// Set a functional weight, parameter, or alias by name. Three-case
    /// dispatch per `xcfun-master/src/XCFunctional.cpp:369-405`. Delegates to
    /// `xcfun_eval::Functional::set` (already implemented in Phase 4 D-04).
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        self.0.set(name, value)
    }

    pub fn get(&self, name: &str) -> Result<f64, XcError> { self.0.get(name) }

    /// `xcfun-master/src/XCFunctional.cpp:420` — `(depends & XC_GRADIENT)`.
    pub fn is_gga(&self) -> bool {
        self.0.dependencies().contains(xcfun_core::Dependency::GRADIENT)
    }
    /// `xcfun-master/src/XCFunctional.cpp:422-424`.
    pub fn is_metagga(&self) -> bool {
        let d = self.0.dependencies();
        d.contains(xcfun_core::Dependency::LAPLACIAN)
            || d.contains(xcfun_core::Dependency::KINETIC)
    }

    pub fn eval_setup(&mut self, vars: Vars, mode: Mode, order: u32)
        -> Result<(), XcError>
    {
        self.0.eval_setup(vars, mode, order)?;
        // Phase-5 wires self.0.{vars,mode,order} as a side-effect; current
        // xcfun_eval::Functional::eval_setup is read-only in Phase 4. The
        // facade is the place to mutate state; verify per CONTEXT-A2 plan.
        self.0.vars = vars; self.0.mode = mode; self.0.order = order;
        Ok(())
    }

    pub fn user_eval_setup(/* per xcfun.h:333-341 signature */) -> Result<(), XcError> {
        // Compose `which_vars` + `which_mode` then call `eval_setup`.
        // Port of XCFunctional.cpp:454-467.
        todo!()
    }

    /// Zero heap allocation on the success path (RS-07). Delegates to
    /// `xcfun_eval::Functional::eval`.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        self.0.eval(input, output)
    }
}
```

**Free-function pattern — pure delegation to `xcfun-core` registry tables (no analog to copy verbatim; design-specified):**

```rust
/// `xcfun-master/src/XCFunctional.cpp:48-51` — version string.
pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }

/// `xcfun-master/src/XCFunctional.cpp:53-62` — splash message.
pub fn splash() -> &'static str { include_str!("../assets/splash.txt") }

/// `xcfun-master/src/XCFunctional.cpp:64-72`.
pub fn authors() -> &'static str { include_str!("../assets/authors.txt") }

/// `xcfun-master/src/XCFunctional.cpp:74-124` — runs Tier-1 self-tests.
/// Phase 5 ships a small subset (~5 LDAs at order 0) per Claude's Discretion.
pub fn self_test() -> i32 { /* iterate FUNCTIONAL_DESCRIPTORS subset */ todo!() }

pub fn is_compatible_library() -> bool { true } // single-process build

/// `xcfun-master/src/XCFunctional.cpp:131-277` — bitwise dispatch.
pub fn which_vars(
    func_type: u32, dens_type: u32, laplacian: u32,
    kinetic: u32, current: u32, explicit_derivatives: u32,
) -> Option<Vars> { /* port the bitwise switch */ todo!() }

pub fn which_mode(mode_type: u32) -> Option<Mode> { /* 1..3 */ todo!() }

/// `xcfun-master/src/XCFunctional.cpp:302-311` — walk `FUNCTIONAL_DESCRIPTORS`
/// then `PARAMETERS`. Returns None if `param` exceeds 81.
pub fn enumerate_parameters(param: i32) -> Option<&'static str> {
    use xcfun_core::{FUNCTIONAL_DESCRIPTORS, PARAMETERS};
    if param < 0 { return None; }
    let i = param as usize;
    if i < FUNCTIONAL_DESCRIPTORS.len() {
        Some(FUNCTIONAL_DESCRIPTORS[i].name)
    } else if i < FUNCTIONAL_DESCRIPTORS.len() + PARAMETERS.len() {
        Some(PARAMETERS[i - FUNCTIONAL_DESCRIPTORS.len()].name)
    } else { None }
}

/// `xcfun-master/src/XCFunctional.cpp:313-320` — walk `ALIASES`.
pub fn enumerate_aliases(n: i32) -> Option<&'static str> {
    if n < 0 { return None; }
    xcfun_core::ALIASES.get(n as usize).map(|a| a.name)
}

/// `xcfun-master/src/XCFunctional.cpp:322-334` — case-insensitive name lookup
/// across functionals + parameters + aliases (3-table cascade, mirrors
/// Phase 4 D-04-B `eq_ignore_ascii_case`).
pub fn describe_short(name: &str) -> Option<&'static str> { /* ... */ todo!() }
pub fn describe_long(name: &str) -> Option<&'static str> { /* ... */ todo!() }
```

**Differences planner must specify:**
- `Functional::new()` and `Functional::eval_setup` may need to update internal `vars`/`mode`/`order` fields — currently `xcfun_eval::Functional::eval_setup` is read-only (functional.rs:423-474 returns `Result<(), XcError>` without mutating self). Phase 5 facade may either (a) wrap and mutate inner fields after a successful inner call (as sketched above), or (b) extend `xcfun_eval::Functional::eval_setup` to mutate self. Plan should pick (a) to avoid touching xcfun-eval's hot path.
- D-08-A combined `InvalidVars + InvalidMode` requires `eval_setup` to accumulate both errors when both apply; current Phase 4 implementation returns one or the other (functional.rs:446-470). Modification is a Phase 5 task in `crates/xcfun-eval/src/functional.rs`.

**Cross-references:** CONTEXT D-02, D-03, D-08-A, D-17; ROADMAP Phase 5 success criteria 1, 5.

---

### A.3 — `crates/xcfun-rs/tests/zero_alloc.rs` (NEW; D-13)

**Closest analog:** `crates/xcfun-eval/tests/regularize_invariant.rs:1-60` (launch + readback + assert pattern; gated behind `#[cfg(feature = "testing")]`).

**Pattern excerpt (regularize_invariant.rs:1-44 — copy structure, substitute counter logic):**

```rust
//! Verify `Functional::eval` performs zero heap allocation on the success
//! path (RS-07 + CONTEXT D-13). Test-only counting `#[global_allocator]`
//! wraps `std::alloc::System` and increments an `AtomicUsize` per
//! alloc/dealloc; we snapshot the counter, run `eval` 100 times across
//! varied densities, and assert delta is exactly zero.

#![cfg(test)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Don't decrement — we're counting allocs only (delta on alloc only).
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static A: CountingAllocator = CountingAllocator;

#[test]
fn eval_is_zero_alloc_on_hot_path() {
    use xcfun_rs::{Functional, Vars, Mode};

    let mut f = Functional::new();
    f.set("slaterx", 1.0).unwrap();
    f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0).unwrap();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();
    let mut input = vec![0.5_f64; inlen];
    let mut output = vec![0.0_f64; outlen];

    // Warm up — ensure any lazy statics are initialized.
    f.eval(&input, &mut output).unwrap();

    let baseline = ALLOC_COUNT.load(Ordering::SeqCst);
    for k in 0..100 {
        input[0] = 0.5 + (k as f64) * 0.001;
        f.eval(&input, &mut output).unwrap();
    }
    let after = ALLOC_COUNT.load(Ordering::SeqCst);

    assert_eq!(after - baseline, 0, "expected zero allocations on hot path");
}
```

**Differences planner must specify:**
- D-13 fixture: 100 iterations × varied densities; delta-zero assertion.
- The `cubecl-cpu` runtime allocates device buffers internally during `client.create_from_slice` — this means `xcfun-eval::Functional::eval` (which calls `cpu_client().create_from_slice(...)`) IS NOT zero-alloc today. Phase 5 has two options: (i) revise `xcfun-eval::Functional::eval` to use a pre-allocated reusable handle; (ii) verify zero-alloc only at the `xcfun-rs::Functional::eval` wrapper boundary EXCLUDING the cubecl substrate (instrument with a `noalloc` scope). Plan should escalate this as a research item via `PLANNING INCONCLUSIVE` if the cubecl-cpu allocator surface conflicts with RS-07.

**Cross-references:** CONTEXT D-13; ROADMAP Phase 5 success criterion 1 ("zero heap allocation on the success path").

---

### A.4 — `crates/xcfun-rs/tests/send_sync.rs` (NEW)

**Closest analog:** `crates/xcfun-core/src/error.rs:55-59`.

**Pattern (verbatim from error.rs:54-59):**

```rust
//! RS-10 — `Functional` MUST be `Send + Sync`. Compile-time gate.

use static_assertions::assert_impl_all;
use xcfun_rs::Functional;

assert_impl_all!(Functional: Send, Sync);
```

**Cross-references:** CONTEXT D-17; ROADMAP Phase 5 success criterion 1.

---

### B.1 — `crates/xcfun-capi/Cargo.toml` (RENAMED + EXTENDED from `xcfun-ffi/Cargo.toml`)

**Closest analog:** `crates/xcfun-ffi/Cargo.toml:1-9` (current 10-line stub) + `validation/Cargo.toml` (cc dev-dep pattern).

**Pattern (extending the renamed stub per D-01 + D-15):**

```toml
[package]
name = "xcfun-capi"
version.workspace = true
edition.workspace = true
description = "C ABI drop-in replacement for xcfun-master/api/xcfun.h"
license = "MPL-2.0"

[lib]
crate-type = ["cdylib", "staticlib", "rlib"]

[dependencies]
xcfun-rs   = { path = "../xcfun-rs" }
xcfun-core = { path = "../xcfun-core" }   # XcError::as_c_code direct reach

[dev-dependencies]
cc = { workspace = true }                  # tests/c_abi.rs builds tests/c_abi.c
```

**Differences planner must specify:**
- D-15: 3-output `crate-type` is novel in this workspace; must be added.
- D-09: NO cbindgen `[build-dependencies]` here — generation runs only via `xtask` and the result is committed.
- The current `crates/xcfun-ffi/Cargo.toml:8` line `xcfun-core = { path = "../xcfun-core" }` is preserved through the rename; xcfun-rs is added.

**Cross-references:** CONTEXT D-01, D-15; CLAUDE.md "Hermetic build" invariant.

---

### B.2 — `crates/xcfun-capi/src/lib.rs` (NEW; replaces stub `xcfun-ffi/src/lib.rs`)

**Closest analog (consumer side — same symbol set, opposite direction):** `validation/src/ffi.rs:14-95`.

**Pattern excerpt (validation/src/ffi.rs:14-24 — symbols we MUST re-export with `#[no_mangle]`):**

```rust
unsafe extern "C" {
    pub fn xcfun_new() -> *mut c_void;
    pub fn xcfun_delete(fun: *mut c_void);
    pub fn xcfun_set(fun: *mut c_void, name: *const i8, value: f64) -> i32;
    pub fn xcfun_eval_setup(fun: *mut c_void, vars: u32, mode: u32, order: i32) -> i32;
    pub fn xcfun_input_length(fun: *const c_void) -> i32;
    pub fn xcfun_output_length(fun: *const c_void) -> i32;
    pub fn xcfun_eval(fun: *const c_void, density: *const f64, result: *mut f64);
}
```

**Phase 5 target — flip these into `#[no_mangle] extern "C" fn` exports per D-05/D-07:**

```rust
//! C ABI drop-in replacement for `xcfun-master/api/xcfun.h`. CAPI-01..07.
//!
//! Every `XCFun_API` symbol declared in `xcfun.h` has a matching
//! `#[no_mangle] extern "C"` export here, wrapped in `c_entry!` for
//! panic + NULL-pointer + error-code policy (D-05 + D-06 + D-07).
//!
//! Layering: this crate depends on `xcfun-rs` (the public Rust API), NOT
//! directly on `xcfun-eval` or `xcfun-core` — keeps the dependency graph
//! one-way (CONTEXT "Integration Points").

#![allow(unsafe_code)]   // FFI boundary — explicit `extern "C"` permits unsafe.

mod c_entry;             // c_entry! macro lives here (D-05).

use std::ffi::{CStr, c_char, c_double, c_int, c_uint};
use xcfun_rs::{Functional, Mode, Vars};

/// Opaque handle exposed to C as `xcfun_t *`. Concrete type is
/// `xcfun_rs::Functional` boxed onto the heap.
pub struct xcfun_s(Functional);

#[unsafe(no_mangle)]
pub extern "C" fn xcfun_new() -> *mut xcfun_s {
    c_entry!("xcfun_new" => {
        Box::into_raw(Box::new(xcfun_s(Functional::new())))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcfun_delete(fun: *mut xcfun_s) {
    // CAPI-03: NULL-safe (matches `delete (T*)nullptr` C++ semantics).
    if fun.is_null() { return; }
    unsafe { drop(Box::from_raw(fun)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcfun_set(fun: *mut xcfun_s, name: *const c_char, value: c_double) -> c_int {
    c_entry!("xcfun_set", fun, name => {
        let name_str = unsafe { CStr::from_ptr(name) }.to_str()
            .unwrap_or_else(|_| die_with("xcfun_set: invalid UTF-8 in name"));
        match unsafe { &mut (*fun).0 }.set(name_str, value) {
            Ok(()) => 0,
            Err(e) => e.as_c_code(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcfun_eval(
    fun: *const xcfun_s,
    density: *const c_double,
    result: *mut c_double,
) {
    c_entry!("xcfun_eval", fun, density, result => {
        let f = unsafe { &(*fun).0 };
        let inlen  = f.input_length();        // RS-09
        let outlen = f.output_length().unwrap_or_else(
            |e| die_with(&format!("xcfun_eval: output_length failed: {}", e)));
        let input  = unsafe { std::slice::from_raw_parts(density, inlen) };
        let output = unsafe { std::slice::from_raw_parts_mut(result, outlen) };
        if let Err(e) = f.eval(input, output) {
            // D-06: void-returning C fns abort on Err with stderr message.
            die_with(&format!("xcfun_eval: {} — did you call xcfun_eval_setup?", e));
        }
    })
}

// ... 18 more entry points, each in a c_entry! body ...
```

**Differences planner must specify:**
- 25 entry points total per `xcfun-master/api/xcfun.h` (counted: version, splash, authors, test, is_compatible_library, which_vars, which_mode, enumerate_parameters, enumerate_aliases, describe_short, describe_long, new, delete, set, get, is_gga, is_metagga, eval_setup, user_eval_setup, input_length, output_length, eval, eval_vec — 23 functions + 2 typedefs `xcfun_s`/`xcfun_t`).
- D-08: caller-supplied buffer sizes are read via `f.input_length()` / `f.output_length()`; C signatures take no length params (matches xcfun.h byte-for-byte).
- D-06 abort policy applies to the void-returning fns: `xcfun_eval`, `xcfun_eval_vec`. The int-returning fns (`xcfun_set`, `xcfun_eval_setup`, `xcfun_user_eval_setup`, `xcfun_get`, `xcfun_test`) return `e.as_c_code()` ergonomically.

**Cross-references:** CONTEXT D-05, D-06, D-07, D-08, D-08-A; CAPI-01..07; ROADMAP Phase 5 success criterion 2.

---

### B.3 — `crates/xcfun-capi/src/c_entry.rs` (NEW — Phase 5 originates this pattern)

**No analog in the repo.** Closest reference is the C++ `xcfun::die` in `xcfun-master/src/functional.hpp` + idiomatic Rust `std::panic::catch_unwind(AssertUnwindSafe(\|\| body))`.

**Source spec — C++ `xcfun::die` reference (functional.hpp + xcint.cpp:55-85):**

```cpp
// In xcfun-master/src/functionals/.../xcint.cpp:
xcfun::die("Functional symbol does not start with XC_", FUN);   // prints + exits
```

**Phase 5 design — `c_entry!` macro skeleton (D-05 + D-07):**

```rust
//! `c_entry!` macro — the panic-trap + NULL-check + diagnostic-abort
//! envelope wrapping every `extern "C" fn` body in this crate.
//!
//! # Behaviour (per CONTEXT D-05/D-06/D-07)
//!
//! 1. Wrap `body` in `std::panic::catch_unwind(AssertUnwindSafe(\|\| ...))`.
//! 2. If any input pointer named in the macro args is NULL, print
//!    `"xcfun: null pointer to {fn_name}"` to stderr and `abort()` (D-07).
//! 3. On Ok(value) — return value to C.
//! 4. On panic — downcast `Box<dyn Any>` to `&str` / `String` for the message,
//!    print `"xcfun: died from panic in {fn_name}: {msg}"` to stderr, `abort()`.
//! 5. Exception: `xcfun_delete` is NULL-safe per CAPI-03 — does NOT use the
//!    macro's NULL-check arm; calls the macro with no pointer args.

use std::panic::{catch_unwind, AssertUnwindSafe};

/// Panic-trap envelope. `fn_name` is a string literal for diagnostic output.
/// Each `$ptr` arg is checked against null BEFORE running `body`.
///
/// Forms:
///   c_entry!("fn_name" => { body })                 — no NULL checks
///   c_entry!("fn_name", ptr1, ptr2, ... => { body }) — NULL-check each ptr
#[macro_export]
macro_rules! c_entry {
    ($fn_name:literal => { $($body:tt)* }) => {{
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $($body)*
        }));
        match result {
            Ok(v) => v,
            Err(payload) => $crate::c_entry::die_from_panic($fn_name, payload),
        }
    }};

    ($fn_name:literal, $($ptr:ident),+ => { $($body:tt)* }) => {{
        $(
            if $ptr.is_null() {
                eprintln!("xcfun: null pointer to {} (arg `{}`)", $fn_name, stringify!($ptr));
                std::process::abort();
            }
        )+
        $crate::c_entry!($fn_name => { $($body)* })
    }};
}

/// Helper invoked from inside an `extern "C"` body when an internal `Err`
/// reaches the void-returning C signature (D-06).
pub fn die_with(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::abort();
}

/// Panic downcast helper — extracts the panic message and aborts. Mirrors
/// xcfun::die's stderr noise.
pub fn die_from_panic(fn_name: &str, payload: Box<dyn std::any::Any + Send>) -> ! {
    let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        format!("(unknown panic payload type {:?})", std::any::TypeId::of::<()>())
    };
    eprintln!("xcfun: died from panic in {fn_name}: {msg}");
    std::process::abort();
}
```

**Differences planner must specify:**
- Decide between two macro shapes per Claude's Discretion: (a) positional args `c_entry!("name", p1, p2 => { ... })` as above, or (b) a separate `null_check!(p1, p2)` sub-macro followed by `c_entry!`. Recommend (a) — single call site, zero ceremony.
- `die_with` is the public escape hatch for the void-returning C fns (`xcfun_eval`, `xcfun_eval_vec`) when `Err` reaches them mid-body. Distinct from `die_from_panic` because the latter handles `catch_unwind` payloads.
- Module placement: `crates/xcfun-capi/src/c_entry.rs` (per D-05/D-07 implicit; explicit choice noted in Claude's Discretion).

**Cross-references:** CONTEXT D-05, D-06, D-07; CAPI-04 (panic safety); CAPI-03 (NULL-safe `xcfun_delete`).

---

### B.4 — `crates/xcfun-capi/build.rs` (OPTIONAL — Phase 5 prefers NO build.rs per D-09)

**Closest analog:** `validation/build.rs:1-226` (cc-driven build script — but for compiling C++ sources, NOT cbindgen).

**Per D-09: cbindgen runs ONLY via `xtask`, not as `[build-dependencies]`.** This means `crates/xcfun-capi/build.rs` SHOULD NOT exist for the cbindgen flow. The `cargo:rerun-if-changed=...` lines should live in the xtask binary instead.

**If a build.rs is needed** (e.g., for the `tests/c_abi.rs` cc::Build invocation — **but that's a test-time concern, NOT a build-time concern**), follow this validation/build.rs:1-21 pattern:

```rust
//! Build-time tasks for `xcfun-capi`. Phase 5: NONE — cbindgen runs only
//! via `cargo run -p xtask --bin regen-capi-header` per D-09. This file is
//! intentionally absent OR a no-op stub:
//!
//! ```rust
//! fn main() {
//!     // Hermetic build invariant (CLAUDE.md): cbindgen NOT invoked here.
//!     // See xtask/src/bin/regen_capi_header.rs.
//! }
//! ```
```

**Recommendation:** Plan should NOT add `crates/xcfun-capi/build.rs`. The `tests/c_abi.rs` integration test handles its own cc::Build at test-runtime (see B.8 below).

**Cross-references:** CONTEXT D-09; CLAUDE.md "Hermetic build" invariant.

---

### B.5 — `crates/xcfun-capi/cbindgen.toml` (NEW — Phase 5 originates)

**No analog in repo.** Source spec: cbindgen 0.29.2 quick-start + CONTEXT D-11/D-12.

**Phase 5 design (D-11 + D-12):**

```toml
# cbindgen.toml — config for `cargo run -p xtask --bin regen-capi-header`.
# Phase 5: drop-in replacement for `xcfun-master/api/xcfun.h`.

language     = "C"
header       = "/* THIS FILE IS GENERATED BY cbindgen. DO NOT EDIT. */"
include_guard = "XCFUN_CAPI_H"
documentation = false                 # D-11: strip Rust doc-comments
documentation_style = "doxy"          # ignored when documentation=false
cpp_compat   = true
style        = "type"
no_includes  = false
sys_includes = ["stdbool.h", "stddef.h"]
include_version = false

# D-12: prelude block defines `XCFun_API` inline so consumers don't need
# the cmake-generated companion `XCFun/XCFunExport.h` header.
after_includes = """
#define XCFUN_API_VERSION 2

#ifndef XCFun_API
# if defined(_WIN32) || defined(__CYGWIN__)
#   ifdef XCFUN_BUILD_SHARED
#     define XCFun_API __declspec(dllexport)
#   else
#     define XCFun_API __declspec(dllimport)
#   endif
# else
#   define XCFun_API __attribute__((visibility(\"default\")))
# endif
#endif
"""

[fn]
prefix = "XCFun_API"                  # D-12: every fn decl gets the visibility prefix.

[export]
prefix = "xcfun_"                     # only export `pub extern "C"` items prefixed `xcfun_`.

[parse]
parse_deps = false                    # only parse the xcfun-capi crate.
```

**Differences planner must specify:**
- D-11: `documentation = false` strips doc-comments; both files diff after comment-strip in `headers_match.rs`.
- D-12: `[fn] prefix = "XCFun_API"` matches upstream `xcfun.h:128` (`XCFun_API const char * xcfun_version();`).
- D-12: prelude inlines the `XCFun_API` macro definitions so the generated header is standalone (no companion `XCFunExport.h` required).
- The `XCFUN_API_VERSION 2` macro from upstream `xcfun.h:24` MUST be reproduced — included in `after_includes`.

**Cross-references:** CONTEXT D-09, D-11, D-12; ROADMAP Phase 5 success criterion 3 (`headers-match` CI test).

---

### B.6 — `crates/xcfun-capi/include/xcfun.h` (AUTO-GENERATED, COMMITTED)

This file IS the diff-target. Closest analog is the source-of-truth file itself: `xcfun-master/api/xcfun.h:1-390` (391 lines).

**Pattern (xcfun-master/api/xcfun.h:34-41 — the entire output should look like this after cbindgen+ comment-strip):**

```c
typedef enum {
  XC_MODE_UNSET = 0,
  XC_PARTIAL_DERIVATIVES,
  XC_POTENTIAL,
  XC_CONTRACTED,
  XC_NR_MODES
} xcfun_mode;
```

**Differences planner must specify:**
- Generated in commit by `cargo run -p xtask --bin regen-capi-header`. Drift gated by `crates/xcfun-capi/tests/headers_match.rs`.
- D-11 strips comments before diff; the generated and reference headers must agree on EVERY non-comment / non-whitespace token.

**Cross-references:** CONTEXT D-09, D-10, D-11; CAPI-02.

---

### B.7 — `crates/xcfun-capi/tests/headers_match.rs` (NEW)

**Closest analog (drift-gate shape):** `xtask/src/bin/regen_registry.rs:24-30` (sha256 stamp + `--check` mode for committed Rust source files). Same idea, different transformation function (whitespace + comment normalization vs SHA-256).

**Phase 5 design (D-10):**

```rust
//! Phase 5 D-10 — diff-test the generated `xcfun-capi/include/xcfun.h`
//! against `xcfun-master/api/xcfun.h` modulo whitespace and comments.
//! Drift produces a diff in test output for human review.

use std::fs;
use std::path::Path;

fn normalize(s: &str) -> String {
    // 1. Strip C `/* ... */` block comments.
    // 2. Strip C++ `//` line comments.
    // 3. Collapse whitespace runs to single space; strip leading/trailing.
    // 4. Drop blank lines.
    // (Implementation ~30 lines of regex/str scanning.)
    todo!()
}

#[test]
fn capi_header_matches_xcfun_master() {
    let generated = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("include/xcfun.h"))
        .expect("missing generated xcfun.h — run `cargo run -p xtask --bin regen-capi-header`");
    let reference = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("xcfun-master/api/xcfun.h"))
        .expect("missing xcfun-master/api/xcfun.h");

    let g = normalize(&generated);
    let r = normalize(&reference);

    if g != r {
        // Print a unified-diff snippet for human review.
        eprintln!("HEADERS DRIFT — generated vs reference (normalized):");
        eprintln!("--- generated ({} bytes)", g.len());
        eprintln!("+++ reference ({} bytes)", r.len());
        // ... line-by-line diff print ...
        panic!("headers_match: drift detected");
    }
}
```

**Differences planner must specify:**
- D-10 normalization: strip C-style `/* ... */`, C++ `//`, blank lines, collapse whitespace; CAPI-02 contract.
- Test runs as `cargo test -p xcfun-capi --test headers_match` (every PR via standard `cargo test`).

**Cross-references:** CONTEXT D-10, D-11; CAPI-02; ROADMAP Phase 5 success criterion 3.

---

### B.8 — `crates/xcfun-capi/tests/c_abi.rs` (NEW)

**Closest analog:** `validation/build.rs:69-224` (cc::Build chain — but for a 100+-file C++ lib compile, NOT a 1-file driver).

**Phase 5 design (D-14):**

```rust
//! Phase 5 D-14 — compile `tests/c_abi.c` against `libxcfun_capi.a`
//! (staticlib output of the cdylib/staticlib/rlib triple per D-15) and
//! exercise 10 reference-driven fixtures.
//!
//! Pattern: cc::Build (mirroring validation/build.rs:69-104) emits a
//! standalone test binary that links the `cargo build` staticlib output.
//! Test runner is `cargo test -p xcfun-capi --test c_abi`.

#[test]
fn c_abi_drop_in_test() {
    // 1. Locate `target/<profile>/libxcfun_capi.a` via env!("CARGO_TARGET_DIR")
    //    or default `target/`.
    // 2. cc::Build::new()
    //      .file("tests/c_abi.c")
    //      .include("include")                 // generated xcfun.h
    //      .flag("-fno-fast-math")
    //      .flag("-ffp-contract=off")          // CLAUDE.md ACC-05/06
    //      .compile("c_abi_test_bin");
    //    // -- BUT cc::Build is a build-time tool; for tests we must
    //    //    invoke the C compiler manually via Command::new("cc")
    //    //    and link against libxcfun_capi.a. See validation/tests/...
    //    //    for the test-runtime pattern.
    // 3. Run the resulting binary; capture exit status.
    // 4. Assert exit code is 0 (each fixture printed PASS).
    todo!()
}
```

**Differences planner must specify:**
- The test-runtime cc invocation is more complex than build-time `cc::Build` — you need to discover the staticlib path, compile + link manually, run the binary, and assert. May be simpler to ship `tests/c_abi.c` as a `[[bin]]` member of an auxiliary `tests/c_abi_runner/` crate that uses cc.rs at build-time (but that adds workspace mass). Plan should pick the simpler form; recommend a single `tests/c_abi.rs` test that shells out to `cc` directly.
- D-14 fixtures: 10 hardcoded tuples (LDA / GGA / metaGGA / alias / Mode::Contracted / Mode::Potential / LB94). Expected outputs computed once via Rust `Functional::eval` and stored as `static const double expected[]` blocks in `tests/c_abi.c`.
- CLAUDE.md: do NOT pass `-ffast-math` / `-Cfast-math` — the cc invocation must be empty-RUSTFLAGS-equivalent.

**Cross-references:** CONTEXT D-14, D-16 (LB94 fixture row 10); ROADMAP Phase 5 success criterion 4.

---

### B.9 — `crates/xcfun-capi/tests/c_abi.c` (NEW; first hand-written C source in repo)

**No analog in repo.** Closest reference: the upstream test pattern in `xcfun-master` (not vendored as test source) and the D-14 fixture spec.

**Phase 5 design (D-14):**

```c
/*
 * Phase 5 D-14 — drop-in C-side golden test.
 * Compiled by `crates/xcfun-capi/tests/c_abi.rs` against `libxcfun_capi.a`
 * + `crates/xcfun-capi/include/xcfun.h`.
 *
 * 10 reference-driven fixtures spanning the public surface:
 * LDA / GGA / metaGGA / B3LYP alias (5 terms) / PBE0 alias /
 * M06 metaGGA / M06X Mode::Contracted / SCAN family /
 * CAM-B3LYP range-separated alias / LB94 Mode::Potential.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include "xcfun.h"

static int run_lda_xc_a_b_partial0(void) {
    xcfun_t * fun = xcfun_new();
    if (!fun) return 1;

    if (xcfun_set(fun, "lda", 1.0) != 0) { xcfun_delete(fun); return 2; }
    if (xcfun_eval_setup(fun, XC_A_B, XC_PARTIAL_DERIVATIVES, 0) != 0) {
        xcfun_delete(fun); return 3;
    }

    const double density[2] = { 0.5, 0.5 };
    double result[1];
    static const double expected[1] = { /* computed via Rust driver */ -0.4576 };

    xcfun_eval(fun, density, result);

    for (int i = 0; i < 1; i++) {
        double rel = fabs((result[i] - expected[i]) / expected[i]);
        if (rel > 1e-12) {
            fprintf(stderr, "FAIL lda: result[%d] = %.16e expected %.16e rel %.2e\n",
                    i, result[i], expected[i], rel);
            xcfun_delete(fun);
            return 10 + i;
        }
    }
    xcfun_delete(fun);
    return 0;
}

/* ... 9 more fixture functions per D-14 table ... */

int main(void) {
    int rc;
    if ((rc = run_lda_xc_a_b_partial0()) != 0)            return rc;
    /* if ((rc = run_pbe_xc_a_b_partial1()) != 0)            return rc; */
    /* ... */
    printf("ALL FIXTURES PASS\n");
    return 0;
}
```

**Differences planner must specify:**
- 10 fixtures per D-14 table: LDA / PBE / BECKEX / B3LYP / PBE0 / M06 / M06X (Contracted) / SCANX (or TPSSX fallback per D-14 SCAN-family note) / CAMB3LYP / LB94 (Potential).
- Expected outputs computed ONCE during plan execution via `xcfun_rs::Functional::eval`; encoded as `static const double expected[]`.
- Fault-isolation: each fixture is a standalone fn returning int; `main` short-circuits on first failure with a numbered exit code.

**Cross-references:** CONTEXT D-14, D-16; CAPI-06; ROADMAP Phase 5 success criterion 4.

---

### C.1 — `XcError::as_c_code` (NEW METHOD on existing enum)

**Closest analog:** `crates/xcfun-core/src/enums.rs:39-71` `impl ParameterId` — same shape (enum match → primitive lookup with comments citing C++ source).

**Pattern excerpt (enums.rs:39-71 — pattern to mirror):**

```rust
impl ParameterId {
    pub const fn default_value(self) -> f64 {
        match self {
            Self::XC_RANGESEP_MU => 0.4,
            Self::XC_EXX => 0.0,
            Self::XC_CAM_ALPHA => 0.19,
            Self::XC_CAM_BETA => 0.46,
        }
    }
}
```

**Phase 5 target — extend `crates/xcfun-core/src/error.rs` (D-08-A):**

```rust
impl XcError {
    /// C ABI error code per CAPI-05 + Phase 5 D-08-A. Mirrors the
    /// `XC_E*` constants in `xcfun-master/src/XCFunctional.hpp:40-46`:
    /// ```cpp
    /// constexpr auto XC_EORDER = 1;
    /// constexpr auto XC_EVARS  = 2;
    /// constexpr auto XC_EMODE  = 4;
    /// ```
    /// `xcfun_eval_setup` may return the bitwise-or `XC_EVARS \| XC_EMODE`
    /// (= 6) when both conditions trigger (XCFunctional.cpp:442). The
    /// `ResolutionPair` variants below (added in Phase 5 if not present)
    /// surface the combination explicitly.
    ///
    /// All other variants (UnknownName, NotConfigured, Runtime,
    /// {Input,Output}LengthMismatch, InvalidEncoding) map to `-1`
    /// (mirroring C++ `xcfun::die`'s "exits cleanly with -1" pattern).
    pub fn as_c_code(&self) -> i32 {
        match self {
            Self::InvalidOrder { .. } => 1,                 // XC_EORDER
            Self::InvalidVars { .. } => 2,                  // XC_EVARS
            Self::InvalidMode { .. } => 4,                  // XC_EMODE
            // The combined 6 case is produced by `eval_setup` accumulating
            // the bitmask; a single XcError variant can't represent both.
            // SEE: D-08-A — `eval_setup` checks both vars-vs-depends and
            // mode-vs-vars BEFORE returning, returns the combined when
            // both apply. This requires an enum variant or a separate
            // wrapper; recommend adding `XcError::InvalidVarsAndMode { ... }`
            // OR returning a non-enum bitmask from `eval_setup`.
            _ => -1,                                         // UnknownName / other
        }
    }
}
```

**Differences planner must specify:**
- D-08-A specifies: `(InvalidVars + InvalidMode) → 6` (XC_EVARS \| XC_EMODE). Two implementation choices:
  - (i) Add a 10th variant `XcError::InvalidVarsAndMode { vars, mode, depends }` (breaks `non_exhaustive` Phase 4 D-25 contract — but `non_exhaustive` is already on `XcError` so additive is fine).
  - (ii) Have `eval_setup` return a bitmask (`u32` accumulated) and have the C ABI map directly without going through XcError.
  - Plan should pick (i) — keeps the Rust API typed and lets `as_c_code` do all the C-mapping.
- Test additions in `error.rs::tests`: 6 new tests asserting `as_c_code()` returns 0, 1, 2, 4, 6, -1 for the corresponding states.

**Cross-references:** CONTEXT D-08-A; CAPI-05; ROADMAP Phase 5 success criterion 5.

---

### D.1 — `xtask/src/bin/regen_capi_header.rs` (NEW)

**Closest analog:** `xtask/src/bin/regen_registry.rs:1-80` (compile extractor + emit + sha256 stamp + `--check` drift gate). EXACT pattern match per CONTEXT D-09 ("mirrors Phase 2 D-21 `regen-registry --check` pattern").

**Pattern excerpt (regen_registry.rs:1-30 — module-doc + workflow:):**

```rust
//! Regenerate `crates/xcfun-capi/include/xcfun.h` from cbindgen + matching
//! `.sha256` stamp file. Phase 5 D-09.
//!
//! Workflow:
//!   1. cbindgen::Builder::new()
//!        .with_crate(/* crates/xcfun-capi */)
//!        .with_config(/* cbindgen.toml */)
//!        .generate()?
//!        .write_to_file(/* include/xcfun.h */);
//!   2. Read written file; sha256 it; write `xcfun.h.sha256`.
//!   3. `--check` mode: regenerate in memory, sha256 it, compare to committed
//!      stamp; exit 2 on drift. Mirrors Phase 2 D-21 regen-registry --check.
//!
//! Invocation:
//!   - `cargo run -p xtask --bin regen-capi-header`               (write mode)
//!   - `cargo run -p xtask --bin regen-capi-header -- --check`    (CI drift gate)

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin regen-capi-header")?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir.parent().context("xtask has no parent directory")?.to_path_buf();
    Ok(root)
}

fn main() -> Result<()> {
    let check_mode = std::env::args().any(|a| a == "--check");
    let root = project_root()?;
    let crate_dir = root.join("crates/xcfun-capi");
    let cbindgen_toml = crate_dir.join("cbindgen.toml");
    let header_path = crate_dir.join("include/xcfun.h");
    let sha_path = crate_dir.join("include/xcfun.h.sha256");

    let cfg = cbindgen::Config::from_file(&cbindgen_toml)
        .context("failed to load cbindgen.toml")?;
    let header = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cfg)
        .generate()
        .context("cbindgen generate failed")?;

    let mut buf = Vec::<u8>::new();
    header.write(&mut buf);
    let hash = format!("{:x}", Sha256::digest(&buf));

    if check_mode {
        let committed = fs::read_to_string(&sha_path)?.trim().to_string();
        if committed != hash {
            bail!("header drift detected — run `cargo run -p xtask --bin regen-capi-header`");
        }
        eprintln!("regen-capi-header: OK");
    } else {
        fs::create_dir_all(crate_dir.join("include"))?;
        fs::write(&header_path, &buf)?;
        fs::write(&sha_path, format!("{hash}\n"))?;
        eprintln!("regen-capi-header: wrote {} ({} bytes; sha256 {})",
                  header_path.display(), buf.len(), hash);
    }
    Ok(())
}
```

**Differences planner must specify:**
- New `[dev-dependencies]` (in xtask/Cargo.toml): `cbindgen = "=0.29.2"`, `sha2 = ...` (already present from regen-registry).
- xtask main.rs dispatch needs a new `Some("regen-capi-header") => { ... }` arm pointing here.

**Cross-references:** CONTEXT D-09; CAPI-02; CLAUDE.md `cbindgen = "=0.29.2"` pin.

---

## Shared Patterns (cross-cutting; apply to multiple files)

### S.1 — Module-doc + canonical-source citation header

**Source:** `crates/xcfun-eval/src/functional.rs:1-30` + `crates/xcfun-core/src/functional_id.rs:1-7` + `crates/xcfun-core/src/error.rs:1-7`.

**Apply to:** every new Rust source file (lib.rs, c_entry.rs, headers_match.rs, c_abi.rs, regen_capi_header.rs, zero_alloc.rs, send_sync.rs).

**Required sections:** `//! <crate-purpose>` + `//! # <Phase 5 link>` + `//! Source: <xcfun-master/...>` for every wrapped C++ entry point + `//! # Phase 5 D-XX` for design-decision references.

### S.2 — `#[forbid(unsafe_code)]` for the Rust facade; `#![allow(unsafe_code)]` for the C ABI

**Source:** `crates/xcfun-core/src/lib.rs:11` (`#![forbid(unsafe_code)]`) + `validation/src/ffi.rs:12` (`#![allow(unsafe_code)]`).

**Apply to:**
- `crates/xcfun-rs/src/lib.rs` — `#![forbid(unsafe_code)]` (D-02 facade is pure delegation).
- `crates/xcfun-capi/src/lib.rs` — `#![allow(unsafe_code)]` (FFI boundary requires `extern "C"` + `from_raw_parts` + `Box::from_raw`).

### S.3 — Case-insensitive name lookup via `eq_ignore_ascii_case` / `to_ascii_uppercase`

**Source:** `crates/xcfun-core/src/functional_id.rs:100-104`:

```rust
pub fn from_name(name: &str) -> Option<Self> {
    let upper = name.to_ascii_uppercase();
    let trimmed = upper.strip_prefix("XC_").unwrap_or(&upper);
    match trimmed { /* ... */ }
}
```

Plus `crates/xcfun-eval/src/functional.rs:182-184`:

```rust
if let Some(alias) = ALIASES.iter().find(|a| a.name.eq_ignore_ascii_case(name)) { /* ... */ }
```

**Apply to:** `xcfun-rs::describe_short`, `describe_long`, and the `which_vars` / `which_mode` numeric-parameter validation (no case folding needed there, but the same idiom applies to anything string-based).

### S.4 — `pub use` re-export at crate root

**Source:** `crates/xcfun-core/src/lib.rs:20-28`:

```rust
pub use constants::*;
pub use enums::{Mode, ParameterId, Vars};
pub use error::XcError;
pub use functional_id::FunctionalId;
pub use registry::{
    ALIASES, Alias, FUNCTIONAL_DESCRIPTORS, FunctionalDescriptor, PARAMETERS,
    ParameterEntry, VARS_TABLE, VarsRow,
};
```

**Apply to:** `crates/xcfun-rs/src/lib.rs` — re-export `Mode`, `Vars`, `XcError`, `ParameterId`, `FunctionalId` (per RS-01 the public surface; consumers should not need `xcfun_core::...` paths).

### S.5 — `assert_impl_all!` compile-time invariants

**Source:** `crates/xcfun-core/src/error.rs:55-59`:

```rust
#[cfg(test)]
mod tests {
    use static_assertions::assert_impl_all;
    assert_impl_all!(XcError: Copy, Clone, Send, Sync, std::fmt::Debug);
}
```

**Apply to:** `crates/xcfun-rs/tests/send_sync.rs` (RS-10 — `Functional: Send + Sync`); `crates/xcfun-capi/src/lib.rs` if any opaque type needs Send-ness asserted (recommend yes for `xcfun_s` since handles cross threads in C callers).

### S.6 — xtask `--check` drift gate pattern (sha256 stamp)

**Source:** `xtask/src/bin/regen_registry.rs:24-30` + the committed `.sha256` files in `crates/xcfun-core/src/registry/generated/`.

**Apply to:** `xtask/src/bin/regen_capi_header.rs` for `crates/xcfun-capi/include/xcfun.h` (D-09; same stamp-and-compare logic).

### S.7 — No `anyhow` in library crates (CI-enforced)

**Source:** `xtask/src/bin/check_no_anyhow.rs` (already covers 7 library crates per CLAUDE.md gate).

**Apply to:** `crates/xcfun-rs/src/**/*.rs` and `crates/xcfun-capi/src/**/*.rs` use `xcfun_core::XcError` only; `anyhow` is permitted in `tests/c_abi.rs` (test code) and in `xtask/src/bin/regen_capi_header.rs` (xtask app boundary).

**CI gate:** existing `cargo run -p xtask --bin check-no-anyhow` adds `xcfun-rs` + `xcfun-capi` to its enforced set per CONTEXT "Pitfalls" line 173-174.

### S.8 — Algorithmic-identity rule (CLAUDE.md ACC-05/06)

**Source:** `crates/xcfun-eval/src/density_vars/build.rs:246-251` (no `mul_add`) + workspace-level RUSTFLAGS empty.

**Apply to:** `crates/xcfun-capi/tests/c_abi.rs` cc invocation MUST pass `-fno-fast-math` and `-ffp-contract=off` (mirroring `validation/build.rs:73-74`). The Rust facade has no float-arithmetic body; ACC-06 applies via the kernel substrate already.

---

## No Analog Found (3 patterns — Phase 5 originates)

| File / Pattern | Role | Reason | Source spec |
|----------------|------|--------|-------------|
| `crates/xcfun-capi/src/c_entry.rs` (`c_entry!` macro) | macro-trap-envelope | No prior Rust code in repo uses `std::panic::catch_unwind` over an FFI boundary; `validation/src/ffi.rs` is consumer-side and unwraps panics differently | CONTEXT D-05 + Rust idiom `catch_unwind(AssertUnwindSafe(\|\| body))` + `xcfun-master/src/functional.hpp:1-27` `xcfun::die` reference |
| `crates/xcfun-capi/cbindgen.toml` | config-file | No prior cbindgen usage in repo | cbindgen 0.29.2 quick-start + CONTEXT D-11/D-12 |
| `crates/xcfun-capi/tests/c_abi.c` | hand-written-C-source | First hand-written C source in the repo | CONTEXT D-14 fixture table; expected outputs computed once via Rust `Functional::eval` |

For these three, planner should treat CONTEXT D-05/D-07 (c_entry!), D-09/D-11/D-12 (cbindgen.toml), and D-14 (c_abi.c) as the **primary specification** rather than seeking a Rust analog.

---

## Strongest Analogs (5 most copyable)

1. `crates/xcfun-eval/src/functional.rs:170-208` (the existing `Functional::set/get` 3-case dispatch — directly delegated by `xcfun-rs::Functional::set/get`).
2. `crates/xcfun-core/src/error.rs:54-91` (`#[cfg(test)] mod tests { assert_impl_all!(...) }` pattern — copied for `send_sync.rs` and any compile-time invariant test).
3. `xtask/src/bin/regen_registry.rs:1-80` (xtask binary skeleton with `--check` drift gate — mirrored verbatim for `regen_capi_header.rs`).
4. `validation/src/ffi.rs:14-95` (the `extern "C"` symbol set + RAII wrapper — inverted from consumer to producer side in `crates/xcfun-capi/src/lib.rs`).
5. `crates/xcfun-eval/tests/regularize_invariant.rs:1-44` (`#[cfg(feature = "testing")]` + launch + readback + assert pattern — adapted for `tests/zero_alloc.rs`).

---

## Metadata

**Analog search scope:**
- `crates/xcfun-{ad,core,eval,ffi,functionals}/src/**/*.rs` — facade/error/registry/dispatch analogs
- `crates/xcfun-eval/tests/*.rs` — invariant test analogs
- `validation/src/{ffi,driver,fixtures,report}.rs` + `validation/build.rs` — FFI consumer + cc::Build analogs
- `xtask/src/bin/*.rs` — drift-gate / xtask binary analogs
- `xcfun-master/src/{XCFunctional.cpp,xcint.cpp,functional.hpp,XCFunctional.hpp}` — C++ source-of-truth for every entry point + `xcfun::die` panic-policy
- `xcfun-master/api/xcfun.h` — drop-in C ABI spec (391 lines, 23 functions + 2 typedefs)

**Files scanned:** ~95 (`*.rs` files in `crates/`, `validation/`, `xtask/`; `*.hpp`/`*.cpp` files in `xcfun-master/src/`; `xcfun-master/api/xcfun.h`).

**Pattern extraction date:** 2026-04-30.

**Phase 5 risk note (planner-relevant):** RS-07 zero-allocation success criterion may conflict with `cubecl-cpu` device-buffer allocation inside `xcfun-eval::Functional::eval` (functional.rs:296-328 calls `cpu_client().create_from_slice(...)` per-launch). Planner should escalate via `PLANNING INCONCLUSIVE` if the zero-alloc fixture cannot be satisfied by the wrapper boundary alone.

