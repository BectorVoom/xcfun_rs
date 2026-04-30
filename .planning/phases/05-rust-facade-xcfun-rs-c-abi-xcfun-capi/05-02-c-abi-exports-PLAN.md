---
plan_id: 05-02-c-abi-exports
phase: 05
wave: 3
depends_on:
  - 05-00-topology-foundation
  - 05-01-rust-facade
files_modified:
  - crates/xcfun-capi/Cargo.toml
  - crates/xcfun-capi/src/lib.rs
  - crates/xcfun-capi/src/c_entry.rs
  - crates/xcfun-capi/src/types.rs
  - crates/xcfun-capi/tests/api_smoke.rs
requirements:
  - CAPI-01
  - CAPI-03
  - CAPI-04
  - CAPI-06
autonomous: true
---

## objective
<objective>
Phase 5 Wave 3 — fill the `xcfun-capi` C ABI crate (renamed from `xcfun-ffi`
in Plan 05-00) with **the complete `extern "C"` symbol set declared in
`xcfun-master/api/xcfun.h`**:

1. **23 `#[no_mangle] pub extern "C" fn` exports** matching every prototype
   in `xcfun-master/api/xcfun.h:128-388`. (CAPI-01)
2. **C-side enum mirrors** `xcfun_mode` and `xcfun_vars` exposed via
   `#[repr(i32 / u32)]` types or via `pub const`s with the exact upstream
   discriminants. The opaque handle `xcfun_t` is a typedef of
   `xcfun_s` which boxes `xcfun_rs::Functional`. (CAPI-01)
3. **`c_entry!` macro** that wraps every body in `std::panic::catch_unwind`
   + NULL-pointer guard + diagnostic abort. (D-05, D-07, CAPI-04)
4. **`die_with(msg)`** helper for void-returning C fns when an internal
   `Err` reaches them mid-body. (D-06)
5. **`xcfun_new` returns `Box<xcfun_s>::into_raw`; `xcfun_delete` is
   NULL-safe** (CAPI-03 — silent no-op on null pointer per
   `delete (T*)nullptr` C++ semantics).
6. **`Cargo.toml` `[lib] crate-type = ["cdylib", "staticlib", "rlib"]`** so
   `cargo build -p xcfun-capi --release` produces both
   `target/release/libxcfun_capi.so` and `target/release/libxcfun_capi.a`.
   (CAPI-06, D-15)

**Out of scope for this plan** (Plan 05-03 / Plan 05-04):
- cbindgen header generation (Plan 05-03).
- `tests/c_abi.c` golden test (Plan 05-04).

Output: a fully-populated `xcfun-capi` crate that builds as cdylib +
staticlib, exports the 23 C symbols, and passes a Rust-side smoke test
that exercises every entry point through its FFI signature.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-PATTERNS.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-00-SUMMARY.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-01-SUMMARY.md

# C ABI source-of-truth — this plan's spec
@xcfun-master/api/xcfun.h
@xcfun-master/src/XCFunctional.cpp
@xcfun-master/src/functional.hpp

# Rust crates this plan calls into
@crates/xcfun-rs/src/lib.rs
@crates/xcfun-rs/src/functional.rs
@crates/xcfun-core/src/error.rs

# Renamed crate state from Plan 05-00
@crates/xcfun-capi/Cargo.toml
@crates/xcfun-capi/src/lib.rs

<interfaces>
<!-- Every C declaration that this plan must mirror -->

The 23 functions to export, from xcfun-master/api/xcfun.h (line numbers cited):

```c
// L128:  XCFun_API const char * xcfun_version();
// L138:  XCFun_API const char * xcfun_splash();
// L143:  XCFun_API const char * xcfun_authors();
// L150:  XCFun_API int xcfun_test();
// L159:  XCFun_API bool xcfun_is_compatible_library();
// L215:  XCFun_API xcfun_vars xcfun_which_vars(unsigned int func_type, unsigned int dens_type, unsigned int laplacian, unsigned int kinetic, unsigned int current, unsigned int explicit_derivatives);
// L226:  XCFun_API xcfun_mode xcfun_which_mode(unsigned int mode_type);
// L232:  XCFun_API const char * xcfun_enumerate_parameters(int param);
// L238:  XCFun_API const char * xcfun_enumerate_aliases(int n);
// L244:  XCFun_API const char * xcfun_describe_short(const char * name);
// L250:  XCFun_API const char * xcfun_describe_long(const char * name);
// L271:  XCFun_API xcfun_t * xcfun_new();
// L276:  XCFun_API void xcfun_delete(xcfun_t * fun);
// L284:  XCFun_API int xcfun_set(xcfun_t * fun, const char * name, double value);
// L294:  XCFun_API int xcfun_get(const xcfun_t * fun, const char * name, double * value);
// L300:  XCFun_API bool xcfun_is_gga(const xcfun_t * fun);
// L306:  XCFun_API bool xcfun_is_metagga(const xcfun_t * fun);
// L315:  XCFun_API int xcfun_eval_setup(xcfun_t * fun, xcfun_vars vars, xcfun_mode mode, int order);
// L333:  XCFun_API int xcfun_user_eval_setup(xcfun_t * fun, const int order, const unsigned int func_type, const unsigned int dens_type, const unsigned int mode_type, const unsigned int laplacian, const unsigned int kinetic, const unsigned int current, const unsigned int explicit_derivatives);
// L347:  XCFun_API int xcfun_input_length(const xcfun_t * fun);
// L356:  XCFun_API int xcfun_output_length(const xcfun_t * fun);
// L366:  XCFun_API void xcfun_eval(const xcfun_t * fun, const double density[], double result[]);
// L382:  XCFun_API void xcfun_eval_vec(const xcfun_t * fun, int nr_points, const double * density, int density_pitch, double * result, int result_pitch);
```

The two C-side enums to mirror (xcfun-master/api/xcfun.h:35-122):

```c
typedef enum {
  XC_MODE_UNSET = 0,
  XC_PARTIAL_DERIVATIVES,    // 1
  XC_POTENTIAL,              // 2
  XC_CONTRACTED,             // 3
  XC_NR_MODES                // 4
} xcfun_mode;

typedef enum {
  XC_VARS_UNSET = -1,
  XC_A,                      // 0
  XC_N,                      // 1
  XC_A_B,                    // 2
  XC_N_S,                    // 3
  // ... 27 more variants ...
  XC_NR_VARS                 // 31
} xcfun_vars;
```

The opaque handle (xcfun-master/api/xcfun.h:255-262):

```c
struct xcfun_s;
typedef struct xcfun_s xcfun_t;
```

xcfun-rs surface (already shipped in Plan 05-01):
```rust
// crates/xcfun-rs/src/lib.rs:
pub use xcfun_core::{Dependency, FunctionalId, Mode, ParameterId, Vars, XcError};
pub use functional::Functional;
pub use free_fns::{
    authors, describe_long, describe_short, enumerate_aliases,
    enumerate_parameters, is_compatible_library, self_test, splash,
    version, which_mode, which_vars,
};

// crates/xcfun-rs/src/functional.rs:
pub struct Functional(/* private */);
impl Functional {
    pub const fn new() -> Self;
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError>;
    pub fn get(&self, name: &str) -> Result<f64, XcError>;
    pub fn is_gga(&self) -> bool;
    pub fn is_metagga(&self) -> bool;
    pub fn eval_setup(&mut self, vars: Vars, mode: Mode, order: u32) -> Result<(), XcError>;
    pub fn user_eval_setup(&mut self, order: i32, func_type: u32, dens_type: u32, mode_type: u32, laplacian: u32, kinetic: u32, current: u32, explicit_derivatives: u32) -> Result<(), XcError>;
    pub fn input_length(&self) -> usize;
    pub fn output_length(&self) -> Result<usize, XcError>;
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError>;
}
```

XcError mapping (from Plan 05-00):
```rust
impl XcError {
    pub fn as_c_code(&self) -> i32; // -> 1, 2, 4, 6, or -1
}
```

`Mode` and `Vars` discriminants (already in xcfun-core, must align with C-side):
- `Mode::Unset = 0, PartialDerivatives = 1, Potential = 2, Contracted = 3` (`#[repr(u32)]`)
- `Vars::A = 0, N = 1, A_B = 2, ...` (matches xcfun.h enum order)
</interfaces>
</context>

## must_haves
<must_haves>
truths:
  - "`cargo build -p xcfun-capi --release` produces both `target/release/libxcfun_capi.so` (cdylib) and `target/release/libxcfun_capi.a` (staticlib). (CAPI-06, D-15)"
  - "`crates/xcfun-capi/Cargo.toml` declares `[lib] crate-type = [\"cdylib\", \"staticlib\", \"rlib\"]`. (D-15)"
  - "Every one of the 23 functions in xcfun-master/api/xcfun.h has a corresponding `#[no_mangle] pub extern \"C\" fn` definition in `crates/xcfun-capi/src/lib.rs`. (CAPI-01)"
  - "Every C entry-point body is wrapped in the `c_entry!` macro that calls `std::panic::catch_unwind(AssertUnwindSafe(|| ...))` and `std::process::abort()` on panic. (D-05, CAPI-04)"
  - "`xcfun_new` returns `Box::into_raw(Box::new(xcfun_s(Functional::new())))`. (CAPI-03)"
  - "`xcfun_delete(NULL)` is a silent no-op (does NOT abort, does NOT panic). (CAPI-03)"
  - "Pointer args other than `xcfun_delete`'s `fun` are NULL-checked by `c_entry!` before being dereferenced; on NULL the entry prints `xcfun: null pointer to <fn_name> (arg `<arg>`)` to stderr and aborts. (D-07)"
  - "`xcfun_set` returns the value of `XcError::as_c_code()` on `Err` and `0` on `Ok`. (CAPI-05 indirectly — implementation lives here, mapping verified in Plan 05-00 tests)"
  - "`xcfun_eval_setup` returns `0` on Ok, `1` for InvalidOrder, `2` for InvalidVars, `4` for InvalidMode, `6` for InvalidVarsAndMode, `-1` for any other Err — by virtue of calling `as_c_code()`."
  - "`xcfun_eval` (void return) calls `die_with` on Err; the C ABI signature is unchanged from upstream (no length params). (D-06, D-08)"
  - "`xcfun_eval_vec` is implemented as a simple loop calling `Functional::eval` per point with the given pitches; (CONTEXT 'Phase 5 → Phase 6': Phase 6 may replace with cubecl-cpu Batch dispatch when nr_points >= 64; Phase 5 ships the synchronous correct loop)."
  - "`xcfun_input_length` and `xcfun_output_length` return `i32` matching upstream signatures (line 347 + 356)."
artifacts:
  - path: "crates/xcfun-capi/Cargo.toml"
    provides: "Triple crate-type, dep on xcfun-rs + xcfun-core"
    contains: "crate-type = [\"cdylib\""
  - path: "crates/xcfun-capi/src/lib.rs"
    provides: "23 #[no_mangle] extern \"C\" fn exports + opaque handle"
    contains: "pub extern \"C\" fn xcfun_new"
  - path: "crates/xcfun-capi/src/c_entry.rs"
    provides: "c_entry! macro + die_with + die_from_panic helpers"
    contains: "macro_rules! c_entry"
  - path: "crates/xcfun-capi/src/types.rs"
    provides: "xcfun_s opaque, plus xcfun_mode_t / xcfun_vars_t #[repr] mirrors"
    contains: "pub struct xcfun_s"
  - path: "crates/xcfun-capi/tests/api_smoke.rs"
    provides: "Rust-side integration test exercising every C entry point through its FFI signature"
    contains: "extern \"C\""
key_links:
  - from: "crates/xcfun-capi/src/lib.rs"
    to: "crates/xcfun-rs::Functional"
    via: "xcfun_s newtype boxes Functional; entry-point bodies delegate"
    pattern: "Functional::new|self\\.0\\.set|self\\.0\\.eval"
  - from: "every #[no_mangle] extern \"C\" body"
    to: "c_entry! macro"
    via: "macro invocation wrapping the body"
    pattern: "c_entry!"
  - from: "void-returning xcfun_eval / xcfun_eval_vec"
    to: "die_with on Err"
    via: "if let Err(e) = ... die_with(...)"
    pattern: "die_with"
</must_haves>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| **C caller → Rust extern "C" fn** | All FFI args are unvalidated raw pointers + UTF-8 strings. Every entry must NULL-check, length-bound, and trap UB before crossing into safe Rust. |
| **Rust panic ↔ C frame** | A panic that unwinds across an `extern "C"` boundary is **undefined behavior** on most ABIs. `c_entry!` macro MUST `catch_unwind` + abort — never propagate. |
| **Heap allocation across FFI** | `xcfun_new` returns `Box::into_raw`. The matching `xcfun_delete` MUST `Box::from_raw` to recover ownership and run Drop. Without this, the Functional's internal `weights` slice + `settings` array leak. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-05-02-01 | **Tampering / DoS — Memory safety** | NULL pointer deref in any entry except `xcfun_delete` | mitigate | `c_entry!` macro accepts a list of pointer-arg identifiers; checks each `is_null()` BEFORE constructing references. On null: stderr message + abort. (D-07) |
| T-05-02-02 | **Tampering — Buffer overrun** | `xcfun_eval` writes through `*mut double result` | mitigate | Body reads `f.input_length()` / `f.output_length()` from the validated `Functional` state, then constructs `slice::from_raw_parts_mut` with EXACTLY that length. Caller is trusted to allocate at least that many doubles per upstream contract. (D-08) |
| T-05-02-03 | **DoS — Buffer overrun (eval_vec)** | `xcfun_eval_vec` reads `nr_points` from a signed C int | mitigate | Cast `nr_points` to `usize` only after `nr_points >= 0` check. Negative → die_with. Positive but enormous (`> isize::MAX`) → still safe because the inner per-point eval re-validates lengths. |
| T-05-02-04 | **DoS — Panic propagation across FFI** | Any panic in a `Functional` method (e.g. index OOB inside cubecl) | mitigate | EVERY `extern "C"` body wrapped in `catch_unwind`; on panic, `die_from_panic` prints diagnostic and `abort()`s — never unwinds across the C frame. (D-05) |
| T-05-02-05 | **Information disclosure — Panic message** | `panic!` text leaking internal symbols / line numbers | accept | Panic messages go to stderr only; no return-value channel. Information disclosure to stderr is the established noisy-failure ethos (CONTEXT specifics line 182 — "computational chemistry users debug numerical code via stderr logs"). |
| T-05-02-06 | **DoS — UTF-8 invalid name** | `xcfun_set(fun, name, value)` with non-UTF-8 bytes | mitigate | `CStr::from_ptr(name).to_str()` returns `Err` on invalid UTF-8 → `die_with("xcfun_set: invalid UTF-8 in name")` + abort. Matches noisy-failure ethos. |
| T-05-02-07 | **Spoofing — Symbol collision** | Linking `libxcfun_capi.so` AND upstream `libxcfun.so` in the same process | accept | Documented in CONTEXT.md "code_context" line 175-176 as expected: the libraries are mutually exclusive by design. No mitigation at the Rust layer. |
| T-05-02-08 | **Tampering — Use-after-free** | C caller invokes `xcfun_eval` on a deleted `xcfun_t *` | accept | Caller bug; no library-side detection (no MagicNumber). The standard C-API contract per upstream `xcfun_delete` documentation. |
</threat_model>

## tasks
<tasks>

<task type="auto">
  <name>Task 2.1: Cargo.toml triple crate-type + c_entry! macro + types.rs (foundation for FFI exports)</name>
  <files>crates/xcfun-capi/Cargo.toml, crates/xcfun-capi/src/c_entry.rs, crates/xcfun-capi/src/types.rs</files>
  <read_first>
    - crates/xcfun-capi/Cargo.toml (post-Plan-05-00 state — name = "xcfun-capi", deps on xcfun-core only)
    - crates/xcfun-rs/Cargo.toml (Plan 05-01 — to confirm xcfun-rs path)
    - xcfun-master/api/xcfun.h:34-122 (xcfun_mode + xcfun_vars C enum discriminants)
    - xcfun-master/src/functional.hpp (`xcfun::die` reference behaviour for matching the noisy-failure pattern)
  </read_first>
  <action>
    1. **Replace `crates/xcfun-capi/Cargo.toml`** entirely with:
       ```toml
       [package]
       name = "xcfun-capi"
       version.workspace = true
       edition.workspace = true
       rust-version.workspace = true
       description = "C ABI drop-in replacement for xcfun-master/api/xcfun.h"
       license = "MPL-2.0"

       [lib]
       crate-type = ["cdylib", "staticlib", "rlib"]

       [dependencies]
       xcfun-rs   = { path = "../xcfun-rs" }
       xcfun-core = { path = "../xcfun-core" }
       ```

    2. **Create `crates/xcfun-capi/src/c_entry.rs`** with EXACTLY:
       ```rust
       //! `c_entry!` macro — panic-trap + NULL-pointer guard envelope
       //! wrapping every `extern "C" fn` body in this crate (Phase 5 D-05).
       //!
       //! # Behaviour (D-05 + D-06 + D-07)
       //! 1. Each `$ptr` arg is checked against null BEFORE running `body`;
       //!    on null the macro prints `"xcfun: null pointer to {fn_name}
       //!    (arg `{ptr}`)"` to stderr and `abort()`s.
       //! 2. Body is wrapped in `std::panic::catch_unwind(AssertUnwindSafe(...))`.
       //! 3. On Ok(value) — value returned to C.
       //! 4. On panic — payload downcast for the message; print
       //!    `"xcfun: died from panic in {fn_name}: {msg}"` to stderr;
       //!    `abort()`.

       use std::any::Any;
       use std::panic::{catch_unwind, AssertUnwindSafe};
       use std::process::abort;

       /// Helper — invoked from a void-returning `extern "C" fn` when an
       /// internal `Err` reaches the body (D-06).
       pub fn die_with(msg: &str) -> ! {
           eprintln!("{msg}");
           abort();
       }

       /// Helper — invoked from `c_entry!` after a panic. Extracts the panic
       /// message and aborts.
       pub fn die_from_panic(fn_name: &str, payload: Box<dyn Any + Send>) -> ! {
           let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
               (*s).to_string()
           } else if let Some(s) = payload.downcast_ref::<String>() {
               s.clone()
           } else {
               String::from("(unknown panic payload)")
           };
           eprintln!("xcfun: died from panic in {fn_name}: {msg}");
           abort();
       }

       /// Run `body` inside `catch_unwind`, aborting on panic with a
       /// diagnostic naming `fn_name`.
       #[inline]
       pub fn run_caught<R>(fn_name: &'static str, body: impl FnOnce() -> R) -> R {
           match catch_unwind(AssertUnwindSafe(body)) {
               Ok(v) => v,
               Err(payload) => die_from_panic(fn_name, payload),
           }
       }

       /// `c_entry!` — top-level macro:
       /// ```ignore
       /// c_entry!("xcfun_new" => { Box::into_raw(...) });
       /// c_entry!("xcfun_set", fun, name => { ... });
       /// ```
       #[macro_export]
       macro_rules! c_entry {
           ($fn_name:literal => { $($body:tt)* }) => {{
               $crate::c_entry::run_caught($fn_name, || { $($body)* })
           }};

           ($fn_name:literal, $($ptr:ident),+ => { $($body:tt)* }) => {{
               $(
                   if $ptr.is_null() {
                       eprintln!(
                           "xcfun: null pointer to {} (arg `{}`)",
                           $fn_name, stringify!($ptr)
                       );
                       std::process::abort();
                   }
               )+
               $crate::c_entry::run_caught($fn_name, || { $($body)* })
           }};
       }
       ```

    3. **Create `crates/xcfun-capi/src/types.rs`** with EXACTLY:
       ```rust
       //! C ABI types mirroring `xcfun-master/api/xcfun.h:34-122, 252-262`.
       //!
       //! - `xcfun_s` — opaque handle wrapping `xcfun_rs::Functional`. C
       //!   callers see `xcfun_t *` (== `*mut xcfun_s`).
       //! - `xcfun_mode_t` and `xcfun_vars_t` — `#[repr]` mirrors of the
       //!   upstream enums. C callers see plain enums; cbindgen (Plan 05-03)
       //!   regenerates the C definitions from these Rust types.

       #![allow(non_camel_case_types)]

       use xcfun_rs::Functional;

       /// Opaque handle (xcfun_t in C). Owns a heap-allocated `Functional`.
       #[repr(C)]
       pub struct xcfun_s {
           pub(crate) inner: Functional,
       }

       /// `xcfun_mode_t` per xcfun-master/api/xcfun.h:35-41.
       /// Discriminants MUST match `xcfun_core::Mode` (Plan 02 D-07: Mode
       /// has #[repr(u32)] with Unset=0).
       #[repr(i32)]
       #[derive(Debug, Clone, Copy, PartialEq, Eq)]
       pub enum xcfun_mode_t {
           XC_MODE_UNSET           = 0,
           XC_PARTIAL_DERIVATIVES  = 1,
           XC_POTENTIAL            = 2,
           XC_CONTRACTED           = 3,
           XC_NR_MODES             = 4,
       }

       /// `xcfun_vars_t` per xcfun-master/api/xcfun.h:86-122.
       /// 31 active variants + UNSET = -1 + NR_VARS = 31. Discriminants
       /// MUST match `xcfun_core::Vars`. Verified by the smoke test.
       #[repr(i32)]
       #[derive(Debug, Clone, Copy, PartialEq, Eq)]
       pub enum xcfun_vars_t {
           XC_VARS_UNSET                                    = -1,
           XC_A                                             =  0,
           XC_N                                             =  1,
           XC_A_B                                           =  2,
           XC_N_S                                           =  3,
           XC_A_GAA                                         =  4,
           XC_N_GNN                                         =  5,
           XC_A_B_GAA_GAB_GBB                               =  6,
           XC_N_S_GNN_GNS_GSS                               =  7,
           XC_A_GAA_LAPA                                    =  8,
           XC_A_GAA_TAUA                                    =  9,
           XC_N_GNN_LAPN                                    = 10,
           XC_N_GNN_TAUN                                    = 11,
           XC_A_B_GAA_GAB_GBB_LAPA_LAPB                     = 12,
           XC_A_B_GAA_GAB_GBB_TAUA_TAUB                     = 13,
           XC_N_S_GNN_GNS_GSS_LAPN_LAPS                     = 14,
           XC_N_S_GNN_GNS_GSS_TAUN_TAUS                     = 15,
           XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB           = 16,
           XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB = 17,
           XC_N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS           = 18,
           XC_A_AX_AY_AZ                                    = 19,
           XC_A_B_AX_AY_AZ_BX_BY_BZ                         = 20,
           XC_N_NX_NY_NZ                                    = 21,
           XC_N_S_NX_NY_NZ_SX_SY_SZ                         = 22,
           XC_A_AX_AY_AZ_TAUA                               = 23,
           XC_A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB               = 24,
           XC_N_NX_NY_NZ_TAUN                               = 25,
           XC_N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS               = 26,
           XC_A_2ND_TAYLOR                                  = 27,
           XC_A_B_2ND_TAYLOR                                = 28,
           XC_N_2ND_TAYLOR                                  = 29,
           XC_N_S_2ND_TAYLOR                                = 30,
           XC_NR_VARS                                       = 31,
       }
       ```

    Verify discriminant alignment with `xcfun-core::Vars` by reading
    `crates/xcfun-core/src/enums.rs` BEFORE writing — if any discriminant
    does NOT match, post a BLOCKER in 05-02-SUMMARY.md (the Vars enum
    in xcfun-core was generated in Phase 2 from the same xcfun.h source,
    so they should match by construction; this is defensive).
  </action>
  <verify>
    <automated>cargo check -p xcfun-capi 2>&1 | tee /tmp/check_05_02a.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>grep -nF 'crate-type = ["cdylib", "staticlib", "rlib"]' crates/xcfun-capi/Cargo.toml</automated>
    <automated>grep -nF 'macro_rules! c_entry' crates/xcfun-capi/src/c_entry.rs</automated>
    <automated>grep -nF 'pub fn die_with' crates/xcfun-capi/src/c_entry.rs</automated>
    <automated>grep -nF 'pub struct xcfun_s' crates/xcfun-capi/src/types.rs</automated>
    <automated>grep -cE "XC_(VARS_UNSET|A|N|A_B|N_S|A_GAA|N_GNN|A_B_GAA_GAB_GBB|N_S_GNN_GNS_GSS|A_GAA_LAPA|A_GAA_TAUA|N_GNN_LAPN|N_GNN_TAUN|A_B_GAA_GAB_GBB_LAPA_LAPB|A_B_GAA_GAB_GBB_TAUA_TAUB|N_S_GNN_GNS_GSS_LAPN_LAPS|N_S_GNN_GNS_GSS_TAUN_TAUS|A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB|A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB|N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS|A_AX_AY_AZ|A_B_AX_AY_AZ_BX_BY_BZ|N_NX_NY_NZ|N_S_NX_NY_NZ_SX_SY_SZ|A_AX_AY_AZ_TAUA|A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB|N_NX_NY_NZ_TAUN|N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS|A_2ND_TAYLOR|A_B_2ND_TAYLOR|N_2ND_TAYLOR|N_S_2ND_TAYLOR|NR_VARS)\s*=" crates/xcfun-capi/src/types.rs | grep -E "^(32|33)$"</automated>
  </verify>
  <done>
    - `crates/xcfun-capi/Cargo.toml` declares the three crate-types.
    - `c_entry!` macro exists with both forms (no-pointer / multi-pointer).
    - `die_with`, `die_from_panic`, `run_caught` helpers exist.
    - `xcfun_s` opaque type wraps `Functional`.
    - `xcfun_mode_t` has 5 variants; `xcfun_vars_t` has 33 entries (UNSET=-1, 31 active variants 0..30, NR_VARS=31).
    - `cargo check -p xcfun-capi` exits 0.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2.2: 23 #[no_mangle] extern "C" fn exports in src/lib.rs</name>
  <files>crates/xcfun-capi/src/lib.rs</files>
  <read_first>
    - crates/xcfun-capi/src/c_entry.rs (Task 2.1 — `c_entry!` macro signature)
    - crates/xcfun-capi/src/types.rs (Task 2.1 — `xcfun_s`, `xcfun_mode_t`, `xcfun_vars_t`)
    - crates/xcfun-rs/src/functional.rs (delegation target — every C entry wraps a method on this)
    - crates/xcfun-rs/src/free_fns.rs (delegation target for the 11 free fns)
    - xcfun-master/api/xcfun.h:128-388 (the 23 prototypes — copy each signature byte-for-byte)
    - xcfun-master/src/XCFunctional.cpp:367-617 (C++ bodies — semantic reference for delegation)
  </read_first>
  <behavior>
    - All 23 functions exist and link; an external Rust integration test that
      `extern "C"` declares each prototype can call every one.
    - `xcfun_new()` returns a non-null `*mut xcfun_s`; `xcfun_delete(p)` runs Drop.
    - `xcfun_delete(NULL)` is a silent no-op.
    - `xcfun_version()` returns a NUL-terminated `*const c_char` whose first
      char is an ASCII digit (delegates to `xcfun_rs::version()`).
    - `xcfun_splash()` and `xcfun_authors()` return non-null `*const c_char`.
    - `xcfun_test()` returns a non-negative `c_int`.
    - `xcfun_is_compatible_library()` returns `true` (`bool` ABI is `1` byte).
    - `xcfun_which_vars(0, 2, 0, 0, 0, 0)` returns the integer code for `XC_A_B` (== 2).
    - `xcfun_which_vars(4, 0, 0, 0, 0, 0)` aborts (the C++ reference dies; we follow D-07 noisy-failure pattern via `die_with`).
    - `xcfun_which_mode(2)` returns the integer code for `XC_POTENTIAL` (== 2).
    - `xcfun_which_mode(0)` aborts (C++ dies on this).
    - `xcfun_enumerate_parameters(0)` returns a non-null `*const c_char`; `xcfun_enumerate_parameters(82)` returns `NULL`.
    - `xcfun_enumerate_aliases(0)` returns a non-null `*const c_char`; `xcfun_enumerate_aliases(46)` returns `NULL`.
    - `xcfun_describe_short("SLATERX")` returns a non-null `*const c_char`; `xcfun_describe_short("not_a_thing")` returns `NULL`.
    - `xcfun_set(fun, "slaterx", 1.0)` returns `0`; `xcfun_set(fun, "not_a_known_name", 1.0)` returns `-1`.
    - After `xcfun_set(fun, "slaterx", 1.0)`, `xcfun_get(fun, "slaterx", &value)` writes `1.0` to value and returns `0`.
    - `xcfun_get(fun, "not_a_known_name", &value)` returns `-1` and does NOT write to value.
    - `xcfun_is_gga(fun)` returns `false` for SLATERX-only set, `true` for `set("pbex", 1.0)`.
    - `xcfun_is_metagga(fun)` returns `false` for SLATERX-only set, `true` for `set("tpssx", 1.0)`.
    - `xcfun_eval_setup(fun, XC_A_B, XC_PARTIAL_DERIVATIVES, 0)` returns `0`.
    - `xcfun_user_eval_setup(fun, 0, 0, 2, 1, 0, 0, 0, 0)` returns `0` (LDA / Alpha+Beta / PartialDerivatives / order 0).
    - `xcfun_input_length(fun)` after eval_setup returns `2` for `XC_A_B`.
    - `xcfun_output_length(fun)` after PartialDerivatives order 0 returns `1`.
    - `xcfun_eval(fun, density, result)` writes a non-zero double to `result[0]` for SLATERX at density `[0.5, 0.5]`.
    - `xcfun_eval_vec(fun, 4, density, 2, result, 1)` is callable and writes 4 doubles to `result`.
    - Every `*fun` pointer arg traps NULL via `c_entry!` (verifying this requires #[ignore]'d test that aborts; unit test asserts the pointer-pattern is present in source).
  </behavior>
  <action>
    Replace `crates/xcfun-capi/src/lib.rs` ENTIRELY with this content
    (do NOT preserve the placeholder doc-line from Plan 05-00). The file is
    long but every entry follows one of three structural patterns shown
    in the first three examples — copy them mechanically:

    ```rust
    //! C ABI drop-in replacement for xcfun-master/api/xcfun.h.
    //! Every `XCFun_API` symbol in the upstream header has a matching
    //! `#[no_mangle] pub extern "C" fn` here, wrapped in `c_entry!`
    //! (Plan 05-02 D-05 + D-06 + D-07).
    //!
    //! Layering: depends on `xcfun-rs` (the public Rust facade), NOT on
    //! `xcfun-eval` or `xcfun-core` directly (CONTEXT "Integration Points").

    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    pub mod c_entry;
    pub mod types;

    use std::ffi::{c_char, c_double, c_int, c_uint, CStr};

    use xcfun_rs::{Functional, Mode, Vars};

    pub use c_entry::{die_from_panic, die_with, run_caught};
    pub use types::{xcfun_mode_t, xcfun_s, xcfun_vars_t};

    // ---------------------------------------------------------------------
    //  Internal helper — convert an i32 vars/mode code from the C side back
    //  into the strongly-typed Rust enum. Returns None for out-of-range.
    // ---------------------------------------------------------------------

    #[inline]
    fn vars_from_i32(v: c_int) -> Option<Vars> {
        // Vars discriminant order matches xcfun_vars_t (verified by Task 2.1
        // discriminant audit). Use unchecked cast via match for safety.
        match v {
            0  => Some(Vars::A),
            1  => Some(Vars::N),
            2  => Some(Vars::A_B),
            3  => Some(Vars::N_S),
            4  => Some(Vars::A_GAA),
            5  => Some(Vars::N_GNN),
            6  => Some(Vars::A_B_GAA_GAB_GBB),
            7  => Some(Vars::N_S_GNN_GNS_GSS),
            8  => Some(Vars::A_GAA_LAPA),
            9  => Some(Vars::A_GAA_TAUA),
            10 => Some(Vars::N_GNN_LAPN),
            11 => Some(Vars::N_GNN_TAUN),
            12 => Some(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB),
            13 => Some(Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
            14 => Some(Vars::N_S_GNN_GNS_GSS_LAPN_LAPS),
            15 => Some(Vars::N_S_GNN_GNS_GSS_TAUN_TAUS),
            16 => Some(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB),
            17 => Some(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
            18 => Some(Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS),
            19 => Some(Vars::A_AX_AY_AZ),
            20 => Some(Vars::A_B_AX_AY_AZ_BX_BY_BZ),
            21 => Some(Vars::N_NX_NY_NZ),
            22 => Some(Vars::N_S_NX_NY_NZ_SX_SY_SZ),
            23 => Some(Vars::A_AX_AY_AZ_TAUA),
            24 => Some(Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB),
            25 => Some(Vars::N_NX_NY_NZ_TAUN),
            26 => Some(Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS),
            27 => Some(Vars::A_2ND_TAYLOR),
            28 => Some(Vars::A_B_2ND_TAYLOR),
            29 => Some(Vars::N_2ND_TAYLOR),
            30 => Some(Vars::N_S_2ND_TAYLOR),
            _  => None,
        }
    }

    #[inline]
    fn mode_from_i32(m: c_int) -> Option<Mode> {
        match m {
            0 => Some(Mode::Unset),
            1 => Some(Mode::PartialDerivatives),
            2 => Some(Mode::Potential),
            3 => Some(Mode::Contracted),
            _ => None,
        }
    }

    // =====================================================================
    //  Free-function exports (RS-09 / xcfun.h:128-250)
    // =====================================================================

    /// xcfun.h:128.
    #[no_mangle]
    pub extern "C" fn xcfun_version() -> *const c_char {
        c_entry!("xcfun_version" => {
            // version() is &'static str — must be NUL-terminated for C.
            // Phase 5: rely on a const NUL-terminated table for the
            // crate version. include_bytes! ensures the literal is
            // both static and NUL-terminated at compile time.
            static VERSION_C: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
            VERSION_C.as_ptr() as *const c_char
        })
    }

    /// xcfun.h:138.
    #[no_mangle]
    pub extern "C" fn xcfun_splash() -> *const c_char {
        c_entry!("xcfun_splash" => {
            static SPLASH_C: &[u8] = concat!(include_str!("../../xcfun-rs/assets/splash.txt"), "\0").as_bytes();
            SPLASH_C.as_ptr() as *const c_char
        })
    }

    /// xcfun.h:143.
    #[no_mangle]
    pub extern "C" fn xcfun_authors() -> *const c_char {
        c_entry!("xcfun_authors" => {
            static AUTHORS_C: &[u8] = concat!(include_str!("../../xcfun-rs/assets/authors.txt"), "\0").as_bytes();
            AUTHORS_C.as_ptr() as *const c_char
        })
    }

    /// xcfun.h:150.
    #[no_mangle]
    pub extern "C" fn xcfun_test() -> c_int {
        c_entry!("xcfun_test" => {
            xcfun_rs::self_test() as c_int
        })
    }

    /// xcfun.h:159.
    #[no_mangle]
    pub extern "C" fn xcfun_is_compatible_library() -> bool {
        c_entry!("xcfun_is_compatible_library" => {
            xcfun_rs::is_compatible_library()
        })
    }

    /// xcfun.h:215. C++ dies on out-of-range; we mirror via die_with.
    #[no_mangle]
    pub extern "C" fn xcfun_which_vars(
        func_type: c_uint, dens_type: c_uint, laplacian: c_uint,
        kinetic: c_uint, current: c_uint, explicit_derivatives: c_uint,
    ) -> c_int {
        c_entry!("xcfun_which_vars" => {
            match xcfun_rs::which_vars(func_type, dens_type, laplacian, kinetic, current, explicit_derivatives) {
                Some(v) => v as c_int,
                None => die_with("xcfun_which_vars: invalid input"),
            }
        })
    }

    /// xcfun.h:226.
    #[no_mangle]
    pub extern "C" fn xcfun_which_mode(mode_type: c_uint) -> c_int {
        c_entry!("xcfun_which_mode" => {
            match xcfun_rs::which_mode(mode_type) {
                Some(m) => m as c_int,
                None => die_with("xcfun_which_mode: invalid input"),
            }
        })
    }

    /// xcfun.h:232.
    #[no_mangle]
    pub extern "C" fn xcfun_enumerate_parameters(param: c_int) -> *const c_char {
        c_entry!("xcfun_enumerate_parameters" => {
            // Static lookup tables in xcfun-core/registry/generated/* contain
            // string literals that ARE NUL-terminated (Rust string literals
            // are not NUL-terminated by themselves, but every name in the
            // tables happens to be matched by a static CStr reference -- if
            // not, see xcfun_describe_short for the lookup-table CStr pattern).
            //
            // For Phase 5: stash NUL-terminated copies in a thread-local or
            // static cache. SIMPLER approach: use a small static slice of
            // CStr literals indexed by `param`. Implementation:
            null_or_cstr(xcfun_rs::enumerate_parameters(param))
        })
    }

    /// xcfun.h:238.
    #[no_mangle]
    pub extern "C" fn xcfun_enumerate_aliases(n: c_int) -> *const c_char {
        c_entry!("xcfun_enumerate_aliases" => {
            null_or_cstr(xcfun_rs::enumerate_aliases(n))
        })
    }

    /// xcfun.h:244.
    #[no_mangle]
    pub extern "C" fn xcfun_describe_short(name: *const c_char) -> *const c_char {
        c_entry!("xcfun_describe_short", name => {
            let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
                Ok(s) => s,
                Err(_) => die_with("xcfun_describe_short: invalid UTF-8 in name"),
            };
            null_or_cstr(xcfun_rs::describe_short(name_str))
        })
    }

    /// xcfun.h:250.
    #[no_mangle]
    pub extern "C" fn xcfun_describe_long(name: *const c_char) -> *const c_char {
        c_entry!("xcfun_describe_long", name => {
            let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
                Ok(s) => s,
                Err(_) => die_with("xcfun_describe_long: invalid UTF-8 in name"),
            };
            null_or_cstr(xcfun_rs::describe_long(name_str))
        })
    }

    // ---------------------------------------------------------------------
    //  Internal helper — convert Option<&'static str> to a NUL-terminated
    //  C-string pointer or NULL. Phase-5 implementation: lazy interning
    //  via a static-keyed cache so callers receive stable, NUL-terminated
    //  pointers without per-call allocation.
    //
    //  For the Phase-5 facade simplicity, we accept that the upstream
    //  string literals (in FUNCTIONAL_DESCRIPTORS / PARAMETERS / ALIASES)
    //  are NOT NUL-terminated by Rust's literal semantics. We therefore
    //  pre-build a parallel `&[CStr]` table at first call via OnceLock;
    //  this is one-time, NOT on the hot path.
    // ---------------------------------------------------------------------

    use std::sync::OnceLock;
    use std::ffi::CString;

    static C_NAMES: OnceLock<Vec<CString>> = OnceLock::new();

    fn null_or_cstr(opt: Option<&'static str>) -> *const c_char {
        match opt {
            None => std::ptr::null(),
            Some(s) => {
                // Intern the string in the OnceLock-backed cache so the
                // returned pointer remains stable for the program lifetime.
                let cache = C_NAMES.get_or_init(Vec::new);
                if let Some(existing) = cache.iter().find(|c| c.to_bytes() == s.as_bytes()) {
                    return existing.as_ptr();
                }
                // Slow path: extend the cache. SAFETY: the OnceLock is set
                // once with an empty Vec; subsequent additions race-safe via
                // an external mutex would be needed. For Phase 5 simplicity,
                // we accept the race at first call: under contention some
                // strings may be interned more than once, but each pointer
                // remains stable. Phase 6 may revisit with a Mutex<Vec>.
                //
                // For correctness in the test suite (no contention), use
                // a synchronized backing via std::sync::Mutex.
                static EXTEND_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
                let _g = EXTEND_LOCK.lock().unwrap();
                // Re-check after locking
                let cache = C_NAMES.get_or_init(Vec::new);
                if let Some(existing) = cache.iter().find(|c| c.to_bytes() == s.as_bytes()) {
                    return existing.as_ptr();
                }
                // Manual extend via a leak: since OnceLock cannot be mutated
                // post-init, switch to a Mutex<Vec<CString>> instead. Refactor:
                static C_NAMES_MUTEX: std::sync::OnceLock<std::sync::Mutex<Vec<CString>>>
                    = std::sync::OnceLock::new();
                let m = C_NAMES_MUTEX.get_or_init(|| std::sync::Mutex::new(Vec::new()));
                let mut v = m.lock().unwrap();
                if let Some(existing) = v.iter().find(|c| c.to_bytes() == s.as_bytes()) {
                    return existing.as_ptr();
                }
                let c = CString::new(s).expect("name contains interior NUL");
                let ptr = c.as_ptr();
                v.push(c);
                ptr
            }
        }
    }

    // =====================================================================
    //  Handle lifecycle (xcfun.h:271-276)
    // =====================================================================

    /// xcfun.h:271.
    #[no_mangle]
    pub extern "C" fn xcfun_new() -> *mut xcfun_s {
        c_entry!("xcfun_new" => {
            Box::into_raw(Box::new(xcfun_s { inner: Functional::new() }))
        })
    }

    /// xcfun.h:276. NULL-safe per CAPI-03 — does NOT use c_entry!'s NULL guard.
    #[no_mangle]
    pub extern "C" fn xcfun_delete(fun: *mut xcfun_s) {
        // CAPI-03: silent no-op on null. Mirrors C++ `delete (T*)nullptr`.
        if fun.is_null() { return; }
        // SAFETY: caller MUST have obtained `fun` from xcfun_new and not
        // already deleted it. Library cannot detect double-delete.
        unsafe { drop(Box::from_raw(fun)); }
    }

    // =====================================================================
    //  Per-functional setters / getters (xcfun.h:284-306)
    // =====================================================================

    /// xcfun.h:284.
    #[no_mangle]
    pub extern "C" fn xcfun_set(fun: *mut xcfun_s, name: *const c_char, value: c_double) -> c_int {
        c_entry!("xcfun_set", fun, name => {
            let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
                Ok(s) => s,
                Err(_) => die_with("xcfun_set: invalid UTF-8 in name"),
            };
            match unsafe { &mut (*fun).inner }.set(name_str, value) {
                Ok(()) => 0,
                Err(e) => e.as_c_code(),
            }
        })
    }

    /// xcfun.h:294. Note `value: *mut c_double`.
    #[no_mangle]
    pub extern "C" fn xcfun_get(fun: *const xcfun_s, name: *const c_char, value: *mut c_double) -> c_int {
        c_entry!("xcfun_get", fun, name, value => {
            let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
                Ok(s) => s,
                Err(_) => die_with("xcfun_get: invalid UTF-8 in name"),
            };
            match unsafe { &(*fun).inner }.get(name_str) {
                Ok(v) => { unsafe { *value = v; } 0 }
                Err(e) => e.as_c_code(),
            }
        })
    }

    /// xcfun.h:300.
    #[no_mangle]
    pub extern "C" fn xcfun_is_gga(fun: *const xcfun_s) -> bool {
        c_entry!("xcfun_is_gga", fun => {
            unsafe { &(*fun).inner }.is_gga()
        })
    }

    /// xcfun.h:306.
    #[no_mangle]
    pub extern "C" fn xcfun_is_metagga(fun: *const xcfun_s) -> bool {
        c_entry!("xcfun_is_metagga", fun => {
            unsafe { &(*fun).inner }.is_metagga()
        })
    }

    // =====================================================================
    //  Setup + length + eval (xcfun.h:315-388)
    // =====================================================================

    /// xcfun.h:315.
    #[no_mangle]
    pub extern "C" fn xcfun_eval_setup(
        fun: *mut xcfun_s, vars: c_int, mode: c_int, order: c_int,
    ) -> c_int {
        c_entry!("xcfun_eval_setup", fun => {
            let v = match vars_from_i32(vars) {
                Some(v) => v,
                None => die_with("xcfun_eval_setup: invalid vars"),
            };
            let m = match mode_from_i32(mode) {
                Some(m) => m,
                None => die_with("xcfun_eval_setup: invalid mode"),
            };
            if order < 0 {
                // C++ XCFunctional.cpp:443 returns XC_EORDER for negative.
                return 1;
            }
            match unsafe { &mut (*fun).inner }.eval_setup(v, m, order as u32) {
                Ok(()) => 0,
                Err(e) => e.as_c_code(),
            }
        })
    }

    /// xcfun.h:333.
    #[no_mangle]
    pub extern "C" fn xcfun_user_eval_setup(
        fun: *mut xcfun_s,
        order: c_int,
        func_type: c_uint, dens_type: c_uint, mode_type: c_uint,
        laplacian: c_uint, kinetic: c_uint, current: c_uint,
        explicit_derivatives: c_uint,
    ) -> c_int {
        c_entry!("xcfun_user_eval_setup", fun => {
            match unsafe { &mut (*fun).inner }.user_eval_setup(
                order, func_type, dens_type, mode_type,
                laplacian, kinetic, current, explicit_derivatives,
            ) {
                Ok(()) => 0,
                Err(e) => e.as_c_code(),
            }
        })
    }

    /// xcfun.h:347.
    #[no_mangle]
    pub extern "C" fn xcfun_input_length(fun: *const xcfun_s) -> c_int {
        c_entry!("xcfun_input_length", fun => {
            unsafe { &(*fun).inner }.input_length() as c_int
        })
    }

    /// xcfun.h:356.
    #[no_mangle]
    pub extern "C" fn xcfun_output_length(fun: *const xcfun_s) -> c_int {
        c_entry!("xcfun_output_length", fun => {
            match unsafe { &(*fun).inner }.output_length() {
                Ok(n) => n as c_int,
                Err(e) => die_with(&format!(
                    "xcfun_output_length: {} -- did you call xcfun_eval_setup?", e
                )),
            }
        })
    }

    /// xcfun.h:366. VOID return — die_with on Err per D-06.
    #[no_mangle]
    pub extern "C" fn xcfun_eval(
        fun: *const xcfun_s, density: *const c_double, result: *mut c_double,
    ) {
        c_entry!("xcfun_eval", fun, density, result => {
            let f = unsafe { &(*fun).inner };
            let inlen = f.input_length();
            let outlen = match f.output_length() {
                Ok(n) => n,
                Err(e) => die_with(&format!("xcfun_eval: output_length failed: {}", e)),
            };
            let input  = unsafe { std::slice::from_raw_parts(density, inlen) };
            let output = unsafe { std::slice::from_raw_parts_mut(result, outlen) };
            if let Err(e) = f.eval(input, output) {
                die_with(&format!(
                    "xcfun_eval: {} -- did you call xcfun_eval_setup?", e
                ));
            }
        })
    }

    /// xcfun.h:382. VOID return — die_with on Err per D-06.
    #[no_mangle]
    pub extern "C" fn xcfun_eval_vec(
        fun: *const xcfun_s,
        nr_points: c_int,
        density: *const c_double, density_pitch: c_int,
        result: *mut c_double, result_pitch: c_int,
    ) {
        c_entry!("xcfun_eval_vec", fun, density, result => {
            if nr_points < 0 {
                die_with("xcfun_eval_vec: nr_points must be non-negative");
            }
            if density_pitch < 0 || result_pitch < 0 {
                die_with("xcfun_eval_vec: pitches must be non-negative");
            }
            let f = unsafe { &(*fun).inner };
            let inlen = f.input_length();
            let outlen = match f.output_length() {
                Ok(n) => n,
                Err(e) => die_with(&format!("xcfun_eval_vec: output_length failed: {}", e)),
            };
            let dp = density_pitch as usize;
            let rp = result_pitch as usize;
            for k in 0..(nr_points as usize) {
                let in_ptr  = unsafe { density.add(k * dp) };
                let out_ptr = unsafe { result.add(k * rp) };
                let in_slice  = unsafe { std::slice::from_raw_parts(in_ptr, inlen) };
                let out_slice = unsafe { std::slice::from_raw_parts_mut(out_ptr, outlen) };
                if let Err(e) = f.eval(in_slice, out_slice) {
                    die_with(&format!(
                        "xcfun_eval_vec: point {} eval failed: {} -- did you call xcfun_eval_setup?",
                        k, e
                    ));
                }
            }
        })
    }
    ```

    NOTE on `null_or_cstr` simplicity: the implementation above has a redundant
    branch sequence (legacy from drafting). The executor MAY simplify to a
    single `Mutex<Vec<CString>>` cache if it does so equivalently — the only
    invariant that matters is: for any given `&'static str s`, repeated calls
    return the same `*const c_char` pointer for the program lifetime, and
    that pointer is NUL-terminated. Implementation MUST NOT cause a hot-path
    allocation on the eval path (eval doesn't call this fn).
  </action>
  <verify>
    <automated>cargo build -p xcfun-capi --release 2>&1 | tee /tmp/build_05_02b.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>test -f target/release/libxcfun_capi.so && test -f target/release/libxcfun_capi.a</automated>
    <automated>nm --defined-only target/release/libxcfun_capi.a 2>/dev/null | grep -cE " T xcfun_(version|splash|authors|test|is_compatible_library|which_vars|which_mode|enumerate_parameters|enumerate_aliases|describe_short|describe_long|new|delete|set|get|is_gga|is_metagga|eval_setup|user_eval_setup|input_length|output_length|eval|eval_vec)$" | grep -E "^23$"</automated>
    <automated>grep -cE '#\[no_mangle\]' crates/xcfun-capi/src/lib.rs | grep -E "^23$"</automated>
    <automated>grep -cE 'pub extern "C" fn xcfun_' crates/xcfun-capi/src/lib.rs | grep -E "^23$"</automated>
    <automated>cargo check -p xcfun-capi 2>&1 | tee /tmp/check_05_02b.log; test ${PIPESTATUS[0]} -eq 0</automated>
  </verify>
  <done>
    - All 23 `extern "C"` exports defined.
    - `cargo build -p xcfun-capi --release` produces both `.so` and `.a`.
    - `nm` confirms 23 `T xcfun_*` symbols in the static archive.
    - `cargo check -p xcfun-capi` exits 0.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2.3: api_smoke.rs — Rust integration test exercising every C entry through its FFI signature</name>
  <files>crates/xcfun-capi/tests/api_smoke.rs</files>
  <read_first>
    - crates/xcfun-capi/src/lib.rs (Task 2.2 — the 23 entries to call)
    - crates/xcfun-capi/src/types.rs (xcfun_s, xcfun_mode_t, xcfun_vars_t)
    - xcfun-master/api/xcfun.h (signatures to mirror in the test's `extern "C"` block)
  </read_first>
  <behavior>
    - Test: `xcfun_new` returns non-null; `xcfun_delete(non_null)` does not abort; `xcfun_delete(null)` does not abort.
    - Test: `xcfun_version` returns non-null + ASCII digit prefix when read via `CStr::from_ptr`.
    - Test: `xcfun_splash` and `xcfun_authors` return non-null; first byte is non-NUL.
    - Test: `xcfun_test` returns non-negative.
    - Test: `xcfun_is_compatible_library` returns true.
    - Test: `xcfun_which_vars(0, 2, 0, 0, 0, 0)` returns 2 (`Vars::A_B`).
    - Test: `xcfun_which_mode(2)` returns 2 (`Mode::Potential`).
    - Test: `xcfun_enumerate_parameters(0)` returns non-null; `xcfun_enumerate_parameters(82)` returns null.
    - Test: `xcfun_enumerate_aliases(0)` returns non-null; `xcfun_enumerate_aliases(46)` returns null.
    - Test: `xcfun_describe_short("SLATERX")` returns non-null; `xcfun_describe_short("not_a_thing")` returns null.
    - Test: `xcfun_describe_long("SLATERX")` returns non-null.
    - Test: full happy-path flow — `xcfun_new` → `xcfun_set("slaterx", 1.0)` (returns 0) → `xcfun_eval_setup(XC_A_B, XC_PARTIAL_DERIVATIVES, 0)` (returns 0) → `xcfun_input_length` returns 2 → `xcfun_output_length` returns 1 → `xcfun_eval` writes a non-zero double → `xcfun_get("slaterx", &v)` returns 0 + v == 1.0 → `xcfun_delete`.
    - Test: `xcfun_set(fun, "not_known", 1.0)` returns -1.
    - Test: `xcfun_eval_setup(fun, 99, 1, 0)` aborts (use `#[ignore]` — runs only via explicit invocation `cargo test -- --ignored xcfun_invalid_vars_aborts`).
    - Test: `xcfun_user_eval_setup(fun, 0, 0, 2, 1, 0, 0, 0, 0)` returns 0 (LDA, A_B, PartialDerivatives, order 0).
    - Test: `xcfun_eval_vec(fun, 4, density, 2, result, 1)` writes 4 doubles when set up for a 2-input 1-output configuration.
    - Test: `xcfun_is_gga(fun_with_pbex)` returns true; `xcfun_is_metagga(fun_with_tpssx)` returns true.
  </behavior>
  <action>
    Create `crates/xcfun-capi/tests/api_smoke.rs` as a Rust integration test
    that links the rlib output of xcfun-capi (the test crate sees
    `xcfun_capi::*` natively via the `rlib` part of the triple crate-type).
    Use the upstream `xcfun.h` signatures verbatim:
    ```rust
    //! Phase 5 D-08 — Rust-side smoke test exercising every C entry point
    //! through its FFI signature. Plan 05-04 adds the actual C-source
    //! golden test (`tests/c_abi.c`).

    use xcfun_capi::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_double, c_int, c_uint};

    fn cstr(s: &str) -> CString { CString::new(s).unwrap() }

    #[test]
    fn xcfun_new_and_delete_null_safe() {
        let fun = xcfun_new();
        assert!(!fun.is_null());
        xcfun_delete(fun);
        // NULL-safe — must not abort.
        xcfun_delete(std::ptr::null_mut());
    }

    #[test]
    fn xcfun_version_returns_digit_string() {
        let p = xcfun_version();
        assert!(!p.is_null());
        let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
        assert!(!s.is_empty());
        assert!(s.chars().next().unwrap().is_ascii_digit(), "got {s:?}");
    }

    #[test]
    fn xcfun_splash_and_authors_non_null() {
        let p = xcfun_splash();
        assert!(!p.is_null());
        let p2 = xcfun_authors();
        assert!(!p2.is_null());
    }

    #[test]
    fn xcfun_test_returns_non_negative() {
        let n = xcfun_test();
        assert!(n >= 0, "expected non-negative, got {n}");
    }

    #[test]
    fn xcfun_is_compatible_library_returns_true() {
        assert!(xcfun_is_compatible_library());
    }

    #[test]
    fn xcfun_which_vars_a_b_returns_two() {
        let v = xcfun_which_vars(0, 2, 0, 0, 0, 0);
        assert_eq!(v, 2);
    }

    #[test]
    fn xcfun_which_mode_potential_returns_two() {
        let m = xcfun_which_mode(2);
        assert_eq!(m, 2);
    }

    #[test]
    fn xcfun_enumerate_parameters_in_range_non_null_out_of_range_null() {
        let p0 = xcfun_enumerate_parameters(0);
        assert!(!p0.is_null());
        let p82 = xcfun_enumerate_parameters(82);
        assert!(p82.is_null());
        let pneg = xcfun_enumerate_parameters(-1);
        assert!(pneg.is_null());
    }

    #[test]
    fn xcfun_enumerate_aliases_in_range_non_null_out_of_range_null() {
        let p0 = xcfun_enumerate_aliases(0);
        assert!(!p0.is_null());
        let p46 = xcfun_enumerate_aliases(46);
        assert!(p46.is_null());
        let pneg = xcfun_enumerate_aliases(-1);
        assert!(pneg.is_null());
    }

    #[test]
    fn xcfun_describe_short_known_and_unknown() {
        let known = cstr("SLATERX");
        let p = xcfun_describe_short(known.as_ptr());
        assert!(!p.is_null());
        let unknown = cstr("not_a_known_thing_at_all_xyz");
        let p2 = xcfun_describe_short(unknown.as_ptr());
        assert!(p2.is_null());
    }

    #[test]
    fn xcfun_describe_long_known() {
        let known = cstr("SLATERX");
        let p = xcfun_describe_long(known.as_ptr());
        assert!(!p.is_null());
    }

    #[test]
    fn xcfun_full_happy_path_lda() {
        let fun = xcfun_new();
        let name = cstr("slaterx");
        assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);

        // Vars::A_B = 2; Mode::PartialDerivatives = 1; order = 0.
        assert_eq!(xcfun_eval_setup(fun, 2, 1, 0), 0);
        assert_eq!(xcfun_input_length(fun), 2);
        assert_eq!(xcfun_output_length(fun), 1);

        let density: [f64; 2] = [0.5, 0.5];
        let mut result: [f64; 1] = [0.0];
        xcfun_eval(fun, density.as_ptr(), result.as_mut_ptr());
        assert!(result[0].abs() > 1e-9, "expected non-zero result, got {}", result[0]);

        let mut got: f64 = 0.0;
        assert_eq!(xcfun_get(fun, name.as_ptr(), &mut got as *mut f64), 0);
        assert_eq!(got, 1.0);

        xcfun_delete(fun);
    }

    #[test]
    fn xcfun_set_unknown_returns_minus_one() {
        let fun = xcfun_new();
        let bad = cstr("not_a_known_name");
        assert_eq!(xcfun_set(fun, bad.as_ptr(), 1.0), -1);
        xcfun_delete(fun);
    }

    #[test]
    fn xcfun_user_eval_setup_lda_a_b() {
        let fun = xcfun_new();
        let name = cstr("slaterx");
        assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
        // order=0, func_type=0 (LDA), dens_type=2 (A_B), mode_type=1
        // (PartialDerivatives), all flags 0.
        assert_eq!(xcfun_user_eval_setup(fun, 0, 0, 2, 1, 0, 0, 0, 0), 0);
        xcfun_delete(fun);
    }

    #[test]
    fn xcfun_eval_vec_writes_all_points() {
        let fun = xcfun_new();
        let name = cstr("slaterx");
        assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
        assert_eq!(xcfun_eval_setup(fun, 2, 1, 0), 0);

        // 4 points × 2 inputs = 8 doubles, density_pitch = 2.
        let density: [f64; 8] = [0.5, 0.5, 0.6, 0.6, 0.7, 0.7, 0.8, 0.8];
        // 4 points × 1 output = 4 doubles, result_pitch = 1.
        let mut result: [f64; 4] = [0.0; 4];
        xcfun_eval_vec(fun, 4, density.as_ptr(), 2, result.as_mut_ptr(), 1);
        for v in result.iter() {
            assert!(v.abs() > 1e-9, "expected non-zero, got {v}");
        }
        xcfun_delete(fun);
    }

    #[test]
    fn xcfun_is_gga_pbex_true() {
        let fun = xcfun_new();
        let name = cstr("pbex");
        assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
        assert!(xcfun_is_gga(fun));
        assert!(!xcfun_is_metagga(fun));
        xcfun_delete(fun);
    }

    #[test]
    fn xcfun_is_metagga_tpssx_true() {
        let fun = xcfun_new();
        let name = cstr("tpssx");
        assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
        assert!(xcfun_is_metagga(fun));
        xcfun_delete(fun);
    }

    // -- abort tests, gated --
    #[test]
    #[ignore = "calls die_with -> abort; runs via `cargo test -- --ignored`"]
    fn xcfun_eval_setup_invalid_vars_aborts() {
        let fun = xcfun_new();
        // 99 is out of range for vars; should die_with.
        let _ = xcfun_eval_setup(fun, 99, 1, 0);
        unreachable!("xcfun_eval_setup with vars=99 must abort");
    }
    ```

    The test file `extern "C"`s through the rlib wrapper of xcfun-capi; the
    type names `xcfun_s`, `xcfun_mode_t`, `xcfun_vars_t` are accessible via
    `use xcfun_capi::*`. Function names are imported the same way.
  </action>
  <verify>
    <automated>cargo test -p xcfun-capi --test api_smoke 2>&1 | tee /tmp/test_05_02c.log; grep -E "test result: ok\." /tmp/test_05_02c.log</automated>
    <automated>cargo test -p xcfun-capi --test api_smoke -- xcfun_full_happy_path_lda 2>&1 | grep -F "test xcfun_full_happy_path_lda ... ok"</automated>
    <automated>cargo test -p xcfun-capi --test api_smoke -- xcfun_eval_vec_writes_all_points 2>&1 | grep -F "test xcfun_eval_vec_writes_all_points ... ok"</automated>
    <automated>grep -cE "^#\[test\]" crates/xcfun-capi/tests/api_smoke.rs | grep -E "^[1-9][0-9]+$"</automated>
  </verify>
  <done>
    - `crates/xcfun-capi/tests/api_smoke.rs` contains ≥ 16 tests covering every <behavior> bullet.
    - `cargo test -p xcfun-capi --test api_smoke` exits 0.
    - All happy-path / null-safety / out-of-range tests pass.
  </done>
</task>

</tasks>

<verification>
Run after all tasks complete:

```bash
# Build artifacts
cargo build -p xcfun-capi --release
test -f target/release/libxcfun_capi.so
test -f target/release/libxcfun_capi.a

# 23 symbols
nm --defined-only target/release/libxcfun_capi.a 2>/dev/null \
  | grep -E " T xcfun_(version|splash|authors|test|is_compatible_library|which_vars|which_mode|enumerate_parameters|enumerate_aliases|describe_short|describe_long|new|delete|set|get|is_gga|is_metagga|eval_setup|user_eval_setup|input_length|output_length|eval|eval_vec)$" \
  | wc -l   # expect 23

# 23 #[no_mangle] in source
grep -cE '#\[no_mangle\]' crates/xcfun-capi/src/lib.rs   # expect 23

# Tests
cargo test -p xcfun-capi --test api_smoke

# No anyhow
! grep -rE "use anyhow|anyhow::" crates/xcfun-capi/src/

# c_entry! used in every body
grep -cE 'c_entry!\(' crates/xcfun-capi/src/lib.rs   # expect >= 23

# NULL-safe delete confirmed
grep -nF 'if fun.is_null() { return; }' crates/xcfun-capi/src/lib.rs
```
</verification>

<success_criteria>
- 23 `#[no_mangle] pub extern "C" fn xcfun_*` exports in `crates/xcfun-capi/src/lib.rs`.
- All 23 wrapped via `c_entry!` (catch_unwind + NULL guard).
- `xcfun_delete` is NULL-safe.
- `cargo build -p xcfun-capi --release` produces cdylib + staticlib + rlib.
- `cargo test -p xcfun-capi --test api_smoke` exits 0 with ≥ 16 tests passing.
- No `anyhow` import in `crates/xcfun-capi/src/`.
- CAPI-01, CAPI-03, CAPI-04, CAPI-06 satisfied.
</success_criteria>

<output>
After completion, create `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-02-SUMMARY.md` documenting:
- 23 entry points landed; nm symbol count.
- c_entry! macro shape (no-pointer, multi-pointer forms).
- File sizes and crate-type triple confirmation.
- Any deviation from the planned `null_or_cstr` simplification noted.
</output>
