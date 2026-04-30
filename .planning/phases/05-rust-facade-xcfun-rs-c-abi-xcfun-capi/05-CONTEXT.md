# Phase 5: Rust Facade (`xcfun-rs`) + C ABI (`xcfun-capi`) - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Public API surface for xcfun_rs. Two deliverables stacked on top of the already-shipped `xcfun-eval::Functional` core:

1. **`xcfun-rs`** — native Rust facade crate. Exposes `Functional` (constructor, setters/getters, `is_gga`/`is_metagga`, `eval_setup`/`user_eval_setup`, `eval`) plus the free functions `version`, `splash`, `authors`, `self_test`, `is_compatible_library`, `which_vars`, `which_mode`, `enumerate_parameters`, `enumerate_aliases`, `describe_short`, `describe_long`. `Functional` is `Send + Sync`; `eval` performs zero heap allocation on the success path.
2. **`xcfun-capi`** — C ABI drop-in replacement for `xcfun-master/api/xcfun.h`. Every declared symbol gets a matching `#[no_mangle] extern "C"` export wrapped in a `c_entry!` macro. cbindgen-generated `xcfun.h` diff-matched against the reference. Builds as both `cdylib` (`libxcfun_capi.so`) and `staticlib` (`libxcfun_capi.a`).

**In scope:** wiring the two crates, c_entry! macro, cbindgen flow + headers-match test, XcError::as_c_code mapping, NULL-pointer policy, hot-path zero-allocation verification, C-side golden test (`tests/c_abi.c`), LB94 inclusion (Phase 3 D-19 deferred this here for the alias-feasibility verdict re-evaluation in light of the now-shipped alias engine).

**Out of scope (Phase 6/7):** GPU runtime / batch lifecycle (`Functional::eval_vec` GPU dispatch for `nr_points >= 64` is RS-08 → Phase 6); Python bindings (`xcfun-py` → Phase 7); resolving the 30+ Phase-4 D-19 forwards (parity drift forwards); per-functional `#[cube]` body changes (those are already complete from Phases 2–4).
</domain>

<decisions>
## Implementation Decisions

### Crate topology

- **D-01:** **Rename `crates/xcfun-ffi` → `crates/xcfun-capi`.** `git mv`, update `package.name = "xcfun-capi"`, update workspace `exclude` list. Aligns with ROADMAP / REQUIREMENTS / PROJECT.md naming. Single source of truth.
- **D-02:** **`xcfun-rs` is a wrapper crate, not a re-export shim.** New `crates/xcfun-rs/` with `pub struct Functional(xcfun_eval::Functional)` (private inner field). Methods (`new`, `set`, `get`, `is_gga`, `is_metagga`, `eval_setup`, `user_eval_setup`, `eval`) delegate to the inner type. Free functions live at the `xcfun-rs` crate root. Decouples public API from `xcfun-eval` internals; `xcfun-eval` can refactor without breaking RS-API doc/signature contracts.
- **D-03:** **Static lookup tables stay in `xcfun-core`; free functions live in `xcfun-rs`.** Tables (`registry::generated::{aliases, parameters, descriptors}`) are already populated by Phase 2 D-21 + Phase 4 D-04-A + D-05 pipelines. `xcfun-rs::{enumerate_parameters, enumerate_aliases, describe_short, describe_long, which_vars, which_mode}` walk those tables. Preserves Phase 2 D-04: `xcfun-core` stays cubecl-free + data-only, no public-API string-shape coupling.
- **D-04:** **Delete `crates/xcfun-functionals/` stub directory.** Remove from workspace `exclude` list. Functional bodies live inline in `xcfun-eval/src/functionals/` post-cubecl-pivot; the stub is dead weight from the pre-pivot scaffold.

### C ABI panic + error contract

- **D-05:** **`c_entry!` macro: `catch_unwind` + abort with stderr diagnostic.** Wraps every `extern "C"` body in `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body))`. On panic: `eprintln!("xcfun: died from panic: {msg}")` (downcast `Box<dyn Any>` to `&str`/`String` for the message), then `std::process::abort()`. Mirrors `xcfun-master/src/functional.hpp` `xcfun::die`.
- **D-06:** **Void-returning C fns (`xcfun_eval`, `xcfun_eval_vec`) abort on internal `Err`.** When `Functional::eval` returns `Err(XcError::*)`, the C entry point prints `xcfun: {error message} — did you call xcfun_eval_setup?` to stderr and `abort()`s. Matches C++ reference (`functional.hpp` die()) bit-for-bit. Caller validates via `xcfun_eval_setup`'s int return BEFORE calling eval; an Err at eval time means the caller violated the contract. **Note:** the Rust facade `Functional::eval` returns `Result<(), XcError>` ergonomically — graceful failure is a Rust-side privilege, not a C-side one.
- **D-07:** **Defensive NULL pointer check at every `c_entry!` body.** Before constructing references from raw pointers, check `fun.is_null()` (and any other `*const c_char`/`*mut double` pointer the entry takes). On null: `eprintln!("xcfun: null pointer to {fn_name}")` and `abort()`. **Exception:** `xcfun_delete` is NULL-safe per CAPI-03 — silent no-op on null (matches `delete (T*)nullptr` C++ semantics).
- **D-08:** **C ABI trusts caller-supplied buffer sizes; Rust validates internally.** `xcfun_eval(fun, density, result)` reads `fun.input_length()` / `fun.output_length()`, constructs slices via `std::slice::from_raw_parts(_mut)`, then calls `Functional::eval`. The Rust eval performs internal bounds + state checks. C signatures are unchanged from upstream (no length params, drop-in xcfun.h).

### `XcError::as_c_code` (locked by spec — captured for reference)

- **D-08-A:** **`XcError::as_c_code` mapping** (per CAPI-05 success criterion):
  | Variant | Code |
  |---------|------|
  | `Ok` (success path) | `0` |
  | `XcError::InvalidOrder` | `1` (EORDER) |
  | `XcError::InvalidVars` | `2` (EVARS) |
  | `XcError::InvalidMode` | `4` (EMODE) |
  | `XcError::InvalidVars + InvalidMode` (combined when `eval_setup` rejects both) | `6` (EVARS\|EMODE) |
  | `XcError::UnknownName` and any other variant | `-1` |

  The combined `6` requires `eval_setup` to detect-both before returning. Implementation: `eval_setup` accumulates a bitmask, returns the combined error variant. Implementation lives in `xcfun-eval` (since `eval_setup` is there) — `xcfun-rs::Functional::eval_setup` forwards.

### cbindgen workflow + headers-match

- **D-09:** **cbindgen runs via `xtask`, output checked into git.** `cargo run -p xtask --bin regen-capi-header` generates `xcfun-capi/include/xcfun.h` from cbindgen. CI drift gate: `cargo run -p xtask --bin regen-capi-header --check` (mirrors Phase 2 D-21 `regen-registry --check` pattern). `cargo build -p xcfun-capi` does NOT invoke cbindgen — preserves the hermetic-build contract (CLAUDE.md).
- **D-10:** **`headers-match` test lives at `xcfun-capi/tests/headers_match.rs`.** Runs as `cargo test -p xcfun-capi --test headers_match`. Reads both `xcfun-capi/include/xcfun.h` and `xcfun-master/api/xcfun.h`, normalizes (strip C-style `/* ... */`, C++ `//` line comments, blank lines, leading/trailing whitespace per CAPI-02), asserts equality. Drift produces a diff in test output for human review. Runs on every PR via standard `cargo test`.
- **D-11:** **cbindgen.toml: `documentation = false`.** Both files' comments are stripped before diff. Generated `xcfun-capi/include/xcfun.h` is comment-free; downstream consumers read the upstream `xcfun-master/api/xcfun.h` for human-readable Doxygen. Lower drift surface (we don't have to maintain Rust doc-comments matching upstream Doxygen byte-for-byte).
- **D-12:** **cbindgen.toml: `function.prefix = "XCFun_API"`** plus a cbindgen prelude block defining `XCFun_API` inline. Upstream's `XCFun_API` is `#define XCFun_API XCFUN_EXPORT` from `xcfun-master/api/xcfun.h:22`, where `XCFUN_EXPORT` is generated by CMake into `XCFun/XCFunExport.h` (visibility/declspec). **Inline the macro definitions in the cbindgen prelude** so consumers don't need a cmake-generated companion header: `#define XCFun_API __attribute__((visibility("default")))` (GCC/Clang), `#define XCFun_API __declspec(dllexport)` (MSVC export side), with the standard `#ifdef XCFUN_BUILD_SHARED` / `_WIN32` guards. Drop-in: caller's existing `#include <xcfun.h>` compiles unchanged on every supported platform.

### Hot-path zero-allocation + golden tests

- **D-13:** **Zero-alloc verification via custom counting `#[global_allocator]`.** Test-only allocator wraps `std::alloc::System`; increments `AtomicUsize` on alloc/dealloc. Test fixture in `xcfun-rs/tests/zero_alloc.rs`:
  1. Construct `Functional`, run `set` + `eval_setup` (allocations OK during setup).
  2. Snapshot counter.
  3. Run `eval` N times (N=100, varied densities).
  4. Assert counter delta is exactly zero.
  ~30 LOC, zero new dependencies. Lives behind `cfg(test)` so the test allocator never ships in release. Mirrors RS-07 success criterion verbatim ("verified by a `dhat` or `#[global_allocator]` fixture" — picking the latter).
- **D-14:** **`tests/c_abi.c` exercises 10 reference-driven fixtures.** Hardcoded tuples spanning the public surface:
  | Functional | Vars | Mode | Order |
  |-----------|------|------|-------|
  | `LDA` | `XC_A_B` | PartialDerivatives | 0 |
  | `PBE` | `XC_A_B_AX_AY_AZ_BX_BY_BZ` | PartialDerivatives | 1 |
  | `BECKEX` | `XC_N_NX_NY_NZ` | PartialDerivatives | 2 |
  | `B3LYP` (alias → 5 terms) | `XC_A_B_AX_AY_AZ_BX_BY_BZ` | PartialDerivatives | 1 |
  | `PBE0` (alias) | `XC_N_NX_NY_NZ` | PartialDerivatives | 1 |
  | `M06` | metaGGA Vars | PartialDerivatives | 0 |
  | `M06X` | metaGGA Vars | Contracted | 3 |
  | `SCANX` | metaGGA Vars | PartialDerivatives | 0 (note: SCAN family is excluded_by_upstream_spec for full sweeps; specific point + order chosen to be in-domain — verify it passes Tier-1 self-tests; otherwise substitute `TPSSX`) |
  | `CAMB3LYP` (alias, range-separated) | `XC_A_B_AX_AY_AZ_BX_BY_BZ` | PartialDerivatives | 0 |
  | `LB94` (Phase-3 D-19 deferred surface) | LDA Vars | Potential | 0 |

  Each tuple has a single density point; expected outputs are computed once via the Rust driver (`Functional::eval`), stored as `static const double expected[]` blocks. C test asserts `fabs((actual[i] - expected[i]) / expected[i]) <= 1e-12` element-wise. **Compile flow:** `cc` crate compiles `tests/c_abi.c` against `libxcfun_capi.a` (staticlib) + generated `xcfun-capi/include/xcfun.h`. Test runner is `cargo test -p xcfun-capi --test c_abi`.

  **Why 10:** matches ROADMAP success criterion 4 wording ("matching output to the Rust reference driver on 10 random fixtures"). Selection is reference-driven, not random — covers LDA, GGA, metaGGA, alias (additive + range-separated), Mode::Contracted, Mode::Potential. Phase-3 D-19 forwards (e.g., PW86X drift at 1e-7) are NOT in the fixture set — those are tracked in the Phase 6 sweep, not the C-ABI golden.

### Crate metadata + build outputs

- **D-15:** **`xcfun-capi/Cargo.toml`: `[lib] crate-type = ["cdylib", "staticlib", "rlib"]`.** Three artefacts: `libxcfun_capi.so` (cdylib for dlopen / linker -lxcfun_capi), `libxcfun_capi.a` (staticlib for `tests/c_abi.c` linking + downstream embedders that prefer static), `libxcfun_capi.rlib` (rlib so other Rust crates in the workspace — e.g., a future integration test crate — can depend on `xcfun-capi` directly without going through the C ABI). Verified by `cargo build -p xcfun-capi --release && ls target/release/libxcfun_capi.{so,a,rlib}`.
- **D-15-A:** **`xcfun-rs/Cargo.toml`: `[lib] crate-type = ["rlib"]` (default).** `xcfun-rs` is the Rust-only public crate — no cdylib needed. Phase 7 (`xcfun-py`) consumes `xcfun-rs::Functional` + free functions through pyo3 bindings.

### LB94 inclusion (Phase-3 D-19 deferred to Phase 5)

- **D-16:** **LB94 surface — re-confirmed not alias-feasible (per Phase 4 D-13).** Phase 4 D-13 examined `xcfun-master/src/functionals/lb94.cpp:1-60` and confirmed LB94 cannot be expressed via the alias engine (it has a non-multiplicative dependency on density gradients in `Mode::Potential`). For Phase 5: include LB94 in the `tests/c_abi.c` Mode::Potential fixture (D-14 row 10) AND in the `xcfun-rs::enumerate_aliases` enumeration if it surfaces as an alias name. Since LB94 is a standalone functional (not an alias), `enumerate_aliases` does NOT enumerate it; `enumerate_parameters` does NOT enumerate it; it is reachable via `Functional::set("lb94", 1.0)` if it's registered as a functional id. **Verify:** check whether `crates/xcfun-core/src/registry/generated/descriptors.rs` already has the LB94 descriptor (Phase 4 had no LB94 work); if not, Phase 5 adds it as an additive descriptor entry. **Authoritative source:** `xcfun-master/src/functionals/lb94.cpp` (66 lines, `FUNCTIONAL` macro registration via `xcfun_register_functional("LB94", ...)`).

### Send + Sync / no global state (RS-10, locked)

- **D-17:** **`Functional` is `Send + Sync` by structural design.** All state is in the struct: `weights: Vec<(FunctionalId, f64)>` (or equivalent, already Send + Sync), `vars: Vars`, `mode: Mode`, `order: u32`, `settings: [f64; N]`. No `static mut`, no `thread_local!`, no `Mutex` / `RwLock` (no need — no shared mutable state). Compile-time gate: `static_assertions::assert_impl_all!(Functional: Send, Sync)` lives in `xcfun-rs/src/lib.rs` integration test or `const _: fn() = || { fn assert<T: Send + Sync>() {} assert::<Functional>(); };`.

### Claude's Discretion

- The exact `c_entry!` macro syntax (positional vs keyword args, whether NULL checks are inline or via a helper `null_check!` sub-macro) — pick what reads cleanest.
- Whether `xcfun-rs::Functional::eval_setup` returns `Result<(), XcError>` (Rust-style) or `Result<i32, XcError>` (preserving the C error code). Lean toward the former — Rust callers want typed errors; the C ABI calls `as_c_code` itself.
- Internal panic-message formatting (e.g., "xcfun: died from panic: panicked at 'foo' …") — match xcfun::die output reasonably; not pinned down.
- Whether `xcfun_test` (CAPI smoke) runs Tier-1 self-tests, a subset, or just returns 0 — choose a small fast subset (~5 functionals × order 0) so caller smoke-tests stay fast.
- cbindgen.toml options not covered by D-09..D-12 (e.g., `cpp_compat = true`, `language = "C"`, `style = "type"`) — pick whatever produces output closest to upstream xcfun.h.
- Choice of integration-test crate name conventions inside `xcfun-rs/tests/` (e.g., `zero_alloc.rs`, `send_sync.rs`, `free_fns.rs`) — by topic.
- Where `c_entry!` macro source lives inside `xcfun-capi` (`src/c_entry.rs` vs `src/macros/mod.rs`) — pick the simpler.

### Folded Todos

None — no pending todos matched Phase 5 scope.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner, executor) MUST read these before planning or implementing.**

### C++ reference (algorithmic-identity source of truth)

- `xcfun-master/api/xcfun.h` — drop-in target: every `XCFun_API` symbol must have a matching `#[no_mangle] extern "C"` export in `xcfun-capi`. 25 entry points. THIS IS THE SPEC for CAPI-01..02.
- `xcfun-master/src/XCFunctional.cpp` — bodies for `xcfun_version` / `xcfun_splash` / `xcfun_authors` / `xcfun_set` / `xcfun_get` / `xcfun_eval_setup` / `xcfun_user_eval_setup` / `xcfun_input_length` / `xcfun_output_length` / `xcfun_eval` / `xcfun_eval_vec` / `xcfun_new` / `xcfun_delete`. Source of truth for the body of every C entry point.
- `xcfun-master/src/xcint.cpp` — `xcfun_which_vars` / `xcfun_which_mode` / case-insensitive name-lookup pattern (`strcasecmp`-equivalent).
- `xcfun-master/src/functional.hpp` — `xcfun::die` definition (RELEVANT to D-05/D-06: panic + error abort policy mirrors this).
- `xcfun-master/src/functionals/aliases.cpp` — alias enumeration source (already consumed by Phase 4 D-04-A `xtask regen-registry`); re-read when implementing `xcfun_enumerate_aliases`.
- `xcfun-master/src/functionals/lb94.cpp` — LB94 functional body; D-16 verifies the descriptor lands in `xcfun-core` registry.

### Design docs (project-internal contracts)

- `docs/design/03-api-surface.md` — RS-01..10 surface contract, `Functional` method signatures, free-function semantics. **Authoritative for Rust facade shape.**
- `docs/design/05-module-responsibilities.md` — crate boundaries; verify `xcfun-rs` and `xcfun-capi` responsibilities post-pivot (the doc may pre-date the rename from `xcfun-ffi`; expect to update §xcfun-capi when D-01 lands).
- `docs/design/08-error-model.md` — `XcError` variants + `as_c_code` mapping; D-08-A is a transcription of this doc.

### Project-wide planning artefacts

- `.planning/ROADMAP.md` §"Phase 5: Rust Facade (`xcfun-rs`) + C ABI (`xcfun-capi`)" — 5 success criteria, 16 requirements (RS-01..07, RS-09, RS-10, CAPI-01..07).
- `.planning/REQUIREMENTS.md` lines 87-106 — full RS-01..10 + CAPI-01..07 requirement statements.
- `.planning/PROJECT.md` — Active requirements list; `xcfun-rs` + `xcfun-capi` named explicitly.
- `CLAUDE.md` (root) — pinned versions: `cbindgen = "=0.29.2"`, `cc = "^1.2.60"`, `thiserror = "=2.0.18"`. Hermetic-build invariant: `cargo build` does NOT require any C/C++ toolchain except for `validation/` and the `tests/c_abi.c` golden.

### Prior phase CONTEXT.md (locked decisions inheritance)

- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md` — D-04 (xcfun-core data-only), D-21 (Functional in xcfun-eval), D-25 (XcError 9 variants, Copy + non_exhaustive). **D-25 is the spec for the variant set referenced by D-08-A here.**
- `.planning/phases/03-gga-tier-mode-potential/03-CONTEXT.md` — D-13 (Mode::Potential variant rejection, metaGGA reject path), D-19 (LB94 + 13 forwards deferred to Phase 5/6), D-25 (XcError reuse, no new variants).
- `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-04 (alias engine line-for-line port), D-04-B (case-insensitive `eq_ignore_ascii_case`), D-13 (LB94 NOT alias-feasible, deferred to Phase 5 — this is the fact D-16 here cites), D-14 (XcError stays at 9 variants).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`xcfun-eval::Functional`** at `crates/xcfun-eval/src/functional.rs:85-878`. Already has `new`, `set` (line 170), `get` (line 200), `eval` (line 234), `dependencies` (line 346), `output_length` (line 373), `eval_setup` (line 423), `launch_potential` (line 496/521). Phase 5's `xcfun-rs::Functional` is a wrapper struct delegating to this surface (D-02).
- **`xcfun-core` registry tables**: `crates/xcfun-core/src/registry/generated/{aliases.rs, parameters.rs, descriptors.rs}`. Already populated by Phase 2 D-21 + Phase 4 D-04-A + D-05. Phase 5 free functions (`enumerate_aliases`, `enumerate_parameters`, `describe_short`, `describe_long`) walk these.
- **`xcfun-core::XcError`** — 9 variants, `Copy + Clone + Debug + non_exhaustive + thiserror::Error`. Phase 5 ADDS `as_c_code` method on this enum (per CAPI-05) — additive change, no new variants.
- **xtask infrastructure** at `xtask/`. Phase 2 D-21 established the `regen-registry` + `--check` pattern. D-09 here adds `regen-capi-header` + `--check` mirroring that pattern.
- **`crates/xcfun-ffi/`** stub directory (Cargo.toml + src) — to be renamed to `xcfun-capi` per D-01.
- **`crates/xcfun-functionals/`** stub directory — to be deleted per D-04.

### Established Patterns

- **`xtask regen-* --check` drift gate** (Phase 2 D-21) — applied to cbindgen header (D-09).
- **Case-insensitive name lookup via `eq_ignore_ascii_case`** (Phase 4 D-04-B) — re-used in `xcfun-rs::Functional::set` / `get` and any C-side name lookup.
- **`#[non_exhaustive]` on public enums** (`Mode`, `Vars`, `XcError`) — preserved at facade boundary.
- **Workspace exclude list as opt-in pattern** (`Cargo.toml`) — Phase 5 removes `xcfun-ffi`, adds `xcfun-rs` and `xcfun-capi` to `members`. Removes `xcfun-functionals` outright.

### Integration Points

- **`xcfun-rs` depends on:** `xcfun-eval` (Functional internals), `xcfun-core` (registry tables, error type, public Vars/Mode enums), `thiserror` (re-exported XcError). NO `cubecl` dependency at the facade level (the kernel substrate stays in `xcfun-eval`).
- **`xcfun-capi` depends on:** `xcfun-rs` (the public Rust API). **NOT directly on `xcfun-eval` or `xcfun-core`** — keeps the layering one-way. cbindgen sees only `xcfun-rs`-mediated types.
- **Phase 5 → Phase 6** (GPU + batch): RS-08 (`Functional::eval_vec`) is Phase 6; Phase 5 leaves the method either undefined or returning `XcError::Runtime("eval_vec requires Phase 6")` as a stub. **Recommendation:** define the signature in Phase 5 (so `xcfun-rs` API is "complete" from RS-01..10 surface POV minus RS-08) but make the body a stub — Phase 6 fills in the GPU dispatch.
- **Phase 5 → Phase 7** (Python): `xcfun-py` consumes `xcfun-rs::Functional` + free functions through pyo3. Decoupling the Rust facade from FFI internals (D-02 wrapper struct) is what enables this clean dependency.

### Pitfalls (from prior phases)

- **`anyhow` in any library crate** is CI-blocked (`xtask check-no-anyhow`, 7 library crates already enforced). **`xcfun-rs` and `xcfun-capi` join the enforced set** — they use `thiserror::XcError` only.
- **`-Cfast-math` / `RUSTFLAGS` reassociation** breaks 1e-12 parity. The `tests/c_abi.c` golden links the staticlib at the same RUSTFLAGS-empty contract; verify the `cc` invocation in the test does not add `-ffast-math`.
- **Symbol-name collision risk with `xcfun-master`** — if a downstream embedder links both `libxcfun_capi.so` (Rust) and `libxcfun.so` (C++ reference) into the same process, the `xcfun_*` symbols collide. **Mitigation:** document this as expected (the libraries are mutually exclusive by design); the validation harness (which DOES link both) namespaces the C++ reference via `extern "C"` block re-naming inside `validation/build.rs`. No Phase-5 action needed beyond a doc note.
</code_context>

<specifics>
## Specific Ideas

- "Drop-in replacement" is the load-bearing phrase: a caller's existing `#include <xcfun.h>` + linker `-lxcfun` setup must work against `libxcfun_capi.so` with **zero source code changes** on the C side. D-09..D-12 collectively defend this contract.
- D-06 + D-07 (abort with stderr message): align with the C++ reference's noisy-failure ethos. Computational chemistry users debug numerical code via stderr logs; silent failures hide bugs. Match the existing user-facing behaviour.
- D-14 fixture row for SCAN family: SCAN family is `excluded_by_upstream_spec` for full sweeps because its C++ harness aborts on `tmath::sqrt_expand` at low-density tail (Phase 4 sign-off ledger). The Phase-5 C-ABI golden is a single in-domain density point, not a sweep — should pass. If it doesn't, substitute `TPSSX` (also metaGGA, no SCAN substrate dependency).
- LB94 (D-16): the Phase 3 D-19 forward + Phase 4 D-13 reaffirmation closes the loop. Phase 5 includes LB94 only in the Mode::Potential C-ABI test row and (if missing) adds the descriptor to the registry — no new functional body work, no new alias logic.
</specifics>

<deferred>
## Deferred Ideas

- **Stream-overlapped async GPU dispatch** in `xcfun_eval_vec` C ABI — out of scope per PROJECT.md (`< 20% throughput gain for 3× API complexity`). Phase 6's `Functional::eval_vec` is synchronous from the C caller's POV.
- **`xcfun_eval_checked` Rust-side sibling fn** (recoverable error variant of `xcfun_eval`) — discussed and rejected during Area 2 (would violate CAPI-01 drop-in contract). The Rust facade `Functional::eval` already returns `Result<(), XcError>` for Rust callers; C callers stick with the abort-on-error contract.
- **Doxygen comment fidelity in cbindgen output** — D-11 strips comments. If a downstream consumer asks for inline Doxygen in the generated header, the work is to add Rust doc-comments matching upstream — non-trivial maintenance burden, deferred until requested.
- **Mode::Contracted orders 5..=6 metaGGA** — Phase 4 D-19 forwarded to Phase 6 (xcfun-ad ctaylor_compose/multo N≥4 specialisations). Phase 5 leaves `eval_setup` accepting orders 0..=6 for Contracted (per Phase 4 D-06-A); the failing combinations are a runtime-parity concern, not a facade-shape concern.
- **30+ Phase-4 D-19 + 13 Phase-3 D-19 parity drifts** — forwarded to Phase 6. Phase 5 is API-shape-only; numerical fidelity is enforced by the existing tier-2 sweep gates from prior phases.
- **`xcfun_input_length` / `xcfun_output_length` ABI versioning** — what happens if a future xcfun-master bumps `XCFUN_API_VERSION` past 2? Phase-5 just hard-pins to API_VERSION=2 (matches `xcfun-master/api/xcfun.h:24`). Future-version migration is out of scope.

### Reviewed Todos (not folded)

None — no pending todos surfaced in cross-reference scan.
</deferred>

---

*Phase: 05-rust-facade-xcfun-rs-c-abi-xcfun-capi*
*Context gathered: 2026-04-30*
