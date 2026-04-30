# Phase 5: Rust Facade + C ABI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in 05-CONTEXT.md — this log preserves the discussion.

**Date:** 2026-04-30
**Phase:** 05-rust-facade-xcfun-rs-c-abi-xcfun-capi
**Mode:** discuss (interactive, all 4 areas selected)
**Areas discussed:** crate topology, C ABI panic + error contract, cbindgen workflow, hot-path zero-alloc verification

## Area Selection

**Question:** Which areas do you want to discuss for Phase 5 (Rust Facade + C ABI)?
- ☑ Crate topology
- ☑ C ABI panic + error contract
- ☑ cbindgen workflow
- ☑ Hot-path zero-alloc verification

User selected all four.

## Area 1 — Crate topology

### Q1.1 — `xcfun-capi` crate origin

**Options:**
- Rename xcfun-ffi → xcfun-capi (Recommended)
- Keep dir as xcfun-ffi, set package name = xcfun-capi
- Add new xcfun-capi crate, delete xcfun-ffi

**User chose:** Rename xcfun-ffi → xcfun-capi.

**→ D-01:** `git mv crates/xcfun-ffi crates/xcfun-capi`, update `package.name`, update workspace exclude list.

### Q1.2 — `xcfun-rs` shape

**Options:**
- Wrapper struct (Recommended) — `pub struct Functional(xcfun_eval::Functional)`, methods delegate, free functions live in xcfun-rs
- Thin `pub use` shim — `pub use xcfun_eval::Functional;`
- Inline RS-API onto xcfun-eval::Functional (no new crate)

**User chose:** Wrapper struct.

**→ D-02:** new `crates/xcfun-rs` with private inner field; methods delegate; free functions at crate root.

### Q1.3 — Static lookup data home

**Options:**
- Data in xcfun-core, fns in xcfun-rs (Recommended)
- Tables and fns both in xcfun-core, re-exported by xcfun-rs
- New xcfun-rs lookup module that re-implements

**User chose:** Data in xcfun-core, fns in xcfun-rs.

**→ D-03:** xcfun-core stays cubecl-free + data-only; xcfun-rs walks the tables for `enumerate_*`, `describe_*`, `which_vars`, `which_mode`.

### Q1.4 — `crates/xcfun-functionals/` stub disposition

**Options:**
- Delete it (Recommended)
- Leave it alone
- Repurpose for Phase 5

**User chose:** Delete it.

**→ D-04:** delete the directory; remove from workspace exclude list.

## Area 2 — C ABI panic + error contract

### Q2.1 — `c_entry!` panic policy

**Options:**
- stderr message + abort (Recommended) — `eprintln!("xcfun: died from panic: {msg}")` + `abort()`
- Silent abort
- Custom panic hook + abort (rejected — global state)

**User chose:** stderr message + abort.

**→ D-05:** `c_entry!` does `catch_unwind(AssertUnwindSafe(|| body))`, downcasts panic message, prints + aborts.

### Q2.2 — Void-returning C fn error policy

**Options:**
- Match C++: abort with diagnostic (Recommended) — mirror `xcfun::die`
- Silent no-op
- Add `eval_with_status` sibling fn (Rust-side only)
- Return error via thread-local + new accessor (rejected — adds non-upstream symbols)

**User chose:** Match C++: abort with diagnostic.

**→ D-06:** `xcfun_eval` / `xcfun_eval_vec` print stderr message and `abort()` on internal `Err`.

### Q2.3 — NULL pointer policy

**Options:**
- Defensive: abort with message (Recommended)
- Match C++ exactly (UB on NULL)
- Silent return: 0/-1 on null

**User chose:** Defensive: abort with message.

**→ D-07:** every `c_entry!` body checks `is_null()` first; `xcfun_delete` is the lone NULL-safe exception per CAPI-03.

### Q2.4 — Buffer length validation

**Options:**
- C ABI trusts, Rust validates (Recommended)
- Defensive size pre-check (no length param available — listed for completeness)
- Add length params (rejected — violates drop-in)

**User chose:** C ABI trusts, Rust validates.

**→ D-08:** C entry constructs slices via `from_raw_parts` from `fun.input_length()` / `fun.output_length()`; Rust eval validates internally.

## Area 3 — cbindgen workflow

### Q3.1 — When/how cbindgen runs

**Options:**
- xtask regen + checked-in header (Recommended)
- build.rs codegen at compile time
- build.rs + checked-in fallback

**User chose:** xtask regen + checked-in header.

**→ D-09:** `xtask regen-capi-header` produces `xcfun-capi/include/xcfun.h`; CI drift gate `--check`. Hermetic-build preserved.

### Q3.2 — `headers-match` test location

**Options:**
- `cargo test -p xcfun-capi --test headers_match` (Recommended)
- `xtask check-headers-match` (CI-only)
- Both

**User chose:** `cargo test -p xcfun-capi --test headers_match`.

**→ D-10:** integration test at `xcfun-capi/tests/headers_match.rs`; reads + normalizes both files; runs every PR.

### Q3.3 — Doxygen comment handling

**Options:**
- Strip comments before diff (Recommended)
- Generate matching Doxygen via Rust doc-comments
- Keep cbindgen Doxygen, accept divergence

**User chose:** Strip comments before diff.

**→ D-11:** cbindgen.toml `documentation = false`; both files normalized comment-free for diff.

### Q3.4 — `XCFun_API` export macro

**Options:**
- Emit XCFun_API verbatim via cbindgen prefix (Recommended)
- Strip the macro, ignore in diff
- Hand-author prelude + cbindgen body

**User chose:** Emit XCFun_API verbatim via cbindgen prefix.

**→ D-12:** cbindgen.toml `function.prefix = "XCFun_API"` + prelude block defining the macro inline (visibility/declspec). Drop-in: caller's existing `#include <xcfun.h>` compiles unchanged.

## Area 4 — Hot-path zero-allocation + golden tests

### Q4.1 — Zero-alloc verification approach

**Options:**
- Custom counting `#[global_allocator]` (Recommended)
- dhat-rs heap profiler
- Both: counter for pass/fail + dhat for diagnosis
- Skip dynamic check, rely on code review (rejected — RS-07 spec)

**User chose:** Custom counting `#[global_allocator]`.

**→ D-13:** test-only allocator wraps System; counts deltas around `eval` calls; asserts zero. Lives in `xcfun-rs/tests/zero_alloc.rs`. ~30 LOC, no extra deps.

### Q4.2 — `tests/c_abi.c` scope

**Options:**
- Reference-driven 10 random fixtures (Recommended)
- Full smoke surface (every C entry point)
- Both: 10 fixtures + symbol-coverage smoke

**User chose:** Reference-driven 10 random fixtures.

**→ D-14:** 10 hardcoded tuples spanning LDA/GGA/metaGGA/alias/Contracted/Potential at orders 0..=3; expected outputs computed once via Rust driver; C test asserts 1e-12 rel-err. Compiled by `cc` against the staticlib.

## Cross-Area Notes

- **No scope creep encountered.** Every area stayed within "HOW we expose Functional", not "what new behaviors should xcfun-rs add." Phase 7's Python bindings, Phase 6's GPU dispatch, and the Phase 4 D-19 numerical-parity forwards stayed deferred.
- **No SPEC.md was loaded** for this phase (none generated via `/gsd-spec-phase`). All requirement-shape constraints came from `.planning/REQUIREMENTS.md` RS-01..10 + CAPI-01..07 plus the ROADMAP success criteria. The discussion clarified implementation choices within those locked requirements.
- **Phase 4 `.continue-here.md`** anti-patterns (validation binary all-or-nothing flush, `gsd-executor` worktree stalls, WSL VM lifetime) noted but not directly applicable — Phase 5 has no long-running validation sweeps. The only carry-forward: do NOT delegate any cargo-build / test runs that exceed ~20 min to `gsd-executor` worktrees; for Phase 5 these are minutes-scale, not hours.
- **Claude's Discretion items** (in 05-CONTEXT.md `<decisions>`): macro-syntax details, panic-message format string, cbindgen.toml fine-grained options, integration-test file naming. Captured for transparency, not pinned.

## Deferred Ideas Captured

- Stream-overlapped async GPU dispatch (PROJECT.md out-of-scope).
- `xcfun_eval_checked` Rust-side sibling (would violate CAPI-01 drop-in).
- Doxygen comment fidelity in generated header (deferred until downstream asks).
- Mode::Contracted orders 5..=6 metaGGA parity (Phase 4 D-19 → Phase 6).
- 30+ Phase-4 + 13 Phase-3 D-19 parity drifts (Phase 6).
- ABI versioning beyond `XCFUN_API_VERSION = 2` (out of scope).
