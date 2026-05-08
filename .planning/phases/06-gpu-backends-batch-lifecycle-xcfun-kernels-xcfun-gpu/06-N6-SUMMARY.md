---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N6
subsystem: ci-and-toolchain
tags: [drift, ci, fmt, clippy, msrv, gap-closure, follow-up-from-06-N5]

# Dependency graph
requires:
  - plan: 06-N5
    provides: "First strict CI pipeline with mpmath-smoke job; surfaced 4 pre-existing repo-drift items"
provides:
  - "Strict CI pipeline (fmt + clippy + build + test + mpmath-smoke) with no continue-on-error or --exclude validation"
  - "MSRV bumped from 1.85 to 1.92 (forced by cubecl-zspace 0.10.0 transitive requirement)"
  - "xcfun-master deterministic resolution on CI via clone-step at HEAD a89b783 (Path B)"
  - "Workspace-wide cargo fmt + clippy clean on stable Rust 1.95"
affects: [07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Algorithm-faithful constant preservation via `#![allow(clippy::excessive_precision, clippy::approx_constant)]` at numerical-crate root (xcfun-kernels) + module level (xcfun-ad/expand/erf.rs) — constants from FreeBSD msun s_erf.c and xcfun-master must match the reference verbatim"
    - "App-boundary crate exemption pattern: stylistic clippy lints silenced at `validation/`, `xcfun-rs/` test files, examples — these crates have explicit harness style or FFI shims that the lints fight against"
    - "Rust 2024 if-let chains used to collapse nested `if let` / `if` blocks (clippy::collapsible_if + manual_find)"

key-files:
  created:
    - ".planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N6-SUMMARY.md"
  modified:
    # Task 1 — MSRV bump (commit 1d28f99)
    - "Cargo.toml (rust-version = 1.92)"
    - "CLAUDE.md (Tech Stack section documents the bump rationale)"
    # Task 2 — xcfun-master clone step (commit bb13b74)
    - ".github/workflows/ci.yml (clone xcfun-master at a89b783 in clippy/build/test jobs; drop --exclude validation; drop continue-on-error)"
    # Task 3 — cargo fmt --all (commit fb517d2)
    - "crates/xcfun-ad/src/* (mass-fmt; ctaylor.rs, ctaylor_rec/*, expand/*, etc.)"
    # Task 4 — clippy unused-import/dead-code triage (commit 0ea5eb0)
    - "crates/xcfun-kernels/src/functionals/mgga/{tpss_like,m0x_like,tpsslocc,scan_like}.rs"
    - "crates/xcfun-kernels/src/functionals/gga/{br_like,brx}.rs"
    # Task 5 — Rust 1.95 clippy lint sweep (commit af484bb)
    - "crates/xcfun-ad/src/expand/{erf,br_inverse}.rs (allow excessive_precision crate-wide; FRAC_2_SQRT_PI; redundant cast)"
    - "crates/xcfun-ad/tests/{golden_compose_n4,golden_multo_n4,expand_trans}.rs (approx_constant allow + iter rewrite + named const)"
    - "crates/xcfun-kernels/src/lib.rs (allow excessive_precision + approx_constant crate-wide)"
    - "crates/xcfun-kernels/src/dispatch.rs (allow manual_range_patterns on supports() to keep wave-grouping comments aligned)"
    - "crates/xcfun-kernels/src/functionals/{lda/{pw92c,pz81c},mgga/shared/{constants,tpss_like,br_like,m0x_like}}.rs (doc indentation, x = x + step → x +=, mut/find/range cleanups)"
    - "crates/xcfun-kernels/src/functionals/mgga/{tpssx,revtpssx}.rs (drop unnecessary &mut on tpss_F_x / revtpss_fx)"
    - "crates/xcfun-kernels/src/functionals/gga/optx/mod.rs (allow module_inception)"
    - "crates/xcfun-eval/src/functional.rs (drop identity_op (0+off); rewrite zeroing loop with iter_mut)"
    - "crates/xcfun-eval/tests/{pack_ctaylor_inputs,potential_parity,self_tests}.rs (test pedagogy allows; collapse if; rewrite manual_find)"
    - "crates/xcfun-gpu/src/backend.rs (allow should_implement_trait on from_str)"
    - "crates/xcfun-gpu/tests/batch_api_shape.rs (elide 'fun lifetime)"
    - "crates/xcfun-rs/{src/lib.rs,tests/{eval_vec_threshold,zero_alloc_strict}.rs} (too_many_arguments + type_complexity + doc indentation)"
    - "crates/xcfun-capi/{src/{lib,types}.rs,examples/gen_expected.rs} (not_unsafe_ptr_arg_deref for FFI; doc indentation; vec_init_then_push)"
    - "crates/xcfun-py/examples/gen_py_fixtures.rs (vec_init_then_push)"
    - "validation/{src/{lib,main}.rs,tests/parallelism_parity.rs,build.rs} (app-boundary lint suite + doc indentation)"
    - "xtask/src/bin/{check_no_anyhow,check_no_fma,check_boundaries,regen_capi_header,regen_registry}.rs (collapse if-chains, drop Ok+?, doc indentation, is_some+unwrap → if let)"

# Decision provenance
decisions:
  - id: D-N6-1
    text: "Hold xcfun-master via CI clone step (Path B), not vendored snapshot (Path A) or feature-gate (Path C)"
    rationale: "Path A adds ~10MB of upstream sources to the repo and a manual update workflow when xcfun-master moves. Path C loses the cbindgen drift gate on CI (the whole point of headers_match is to catch upstream xcfun.h drift). Path B is ~2-5 sec extra per CI run and gives single-source-of-truth tracking against upstream HEAD a89b783."

  - id: D-N6-2
    text: "Bump MSRV from 1.85 to 1.92 (Path B), not downgrade transitive deps (Path A)"
    rationale: "cubecl-zspace 0.10.0 declares rust-version = 1.92 transitively via the =0.10.0 cubecl pin. Downgrading via cargo update --precise would have to drop cubecl-zspace, which breaks the cubecl 0.10.0 lockstep contract pinned in CLAUDE.md. The MSRV-1.85 promise was never load-bearing for downstream consumers (no published crate yet); 1.92 is mechanically forced by the dependency tree."

  - id: D-N6-3
    text: "Algorithm-faithful constants preserved via crate-level allow attributes, not by replacing literals with std::f64::consts where they happen to coincide"
    rationale: "Phase 2 D-22 + ACC-04: the validation-crate parity contract is bit-for-bit identity with the C++ reference. Replacing FreeBSD msun coefficients (erf.rs ERX, EFX8, PP*, QQ*, etc.) or xcfun-master constants (M0X_CF_F64, INV_PI in pw91c, etc.) with named std constants would change the audit trail even when the value matches at f64 precision. The allow attribute pattern records intent and lets `git blame` find the source file in xcfun-master / libm. The ONE exception was the `2/√π` constant in erf.rs lines 336/384 — that was originally written as a literal because an earlier libm pull-up; replacing it with std::f64::consts::FRAC_2_SQRT_PI is bit-equal and reads cleaner."

# Verification
self-check:
  cargo-fmt: PASSED
  cargo-clippy: PASSED
  cargo-build: PASSED
  cargo-test-lib: PASSED (all library unit tests; cbrt_expand_x0_{0_1,10} pre-existing f32-truncation failure documented in project memory)
  ci-strict-jobs: TBD on next push to master (the clippy fixes in af484bb need a green CI run to confirm Task 5 closure)

deviations:
  - description: "Task 5 acceptance criterion 'one green CI run on master with all 5 jobs strict' was originally written assuming the local toolchain matched what CI uses. The 4 commits 1d28f99/bb13b74/fb517d2/0ea5eb0 covered the original Tasks 1-4 scope on Rust 1.92. The local environment has since moved to stable Rust 1.95 (released 2026-04-14), which introduced ~25 new clippy lints across the workspace that weren't surfaced when 06-N6 was first committed."
    resolution: "Added a 5th commit (af484bb) extending Task 5 scope to address the new 1.95 lints. All fixes are either (a) algorithm-faithful preservation via `#![allow(...)]` (numerical constants), (b) mechanical clippy-suggested rewrites (assign_op, manual_find, collapsible_if, redundant cast), or (c) app-boundary exemptions for stylistic lints that fight against intentional harness structure (validation crate, FFI signatures, test pedagogy). No production logic changed."
---

# Plan 06-N6 Summary — pre-existing CI drift sweep + Rust 1.95 clippy follow-up

## Context

Plan 06-N5's first GitHub Actions CI run (workflow run #25526864865) was the
first time master was exercised against a clean Linux runner. It surfaced
four pre-existing repo-drift items, none of which were caused by 06-N5
itself:

  1. **MSRV vs Cargo.lock mismatch (HIGH).** `Cargo.toml` pinned `rust-version = "1.85"`
     but the lockfile transitively required rustc ≥ 1.92 (`cubecl-zspace 0.10.0`,
     `darling 0.23`, `icu_*@2.2.0`, `sysinfo 0.38.4`, `tracel-llvm`).
  2. **`xcfun-master/` not available on CI (HIGH).** The `validation/` crate's
     `build.rs` and the `xcfun-capi::headers_match` test both depend on
     xcfun-master files at HEAD a89b783; CI had no way to access them.
  3. **fmt drift (MEDIUM).** `cargo fmt --check` reported diffs across dozens
     of files in `crates/xcfun-ad/src/`.
  4. **clippy unused-import / dead-code (MEDIUM).** Seven warning sites
     flagged by VS Code rustc and confirmed by `cargo clippy`.

Plan 06-N6 closed all four. The plan was `autonomous: false` because Tasks 1
and 2 were design decisions that only the operator could make.

## Decisions

**Task 1 (MSRV) — Path B: bump to 1.92.**
Path A (downgrade transitive deps) was infeasible: `cubecl-zspace 0.10.0` is
load-bearing per the `=0.10.0` lockstep pin in CLAUDE.md, and dropping it
would unwind the entire cubecl-* family. CLAUDE.md updated to document
the bump.

**Task 2 (xcfun-master) — Path B: CI clone step.**
The clone is `git clone https://github.com/dftlibs/xcfun.git --depth 1 xcfun-master &&
git -C xcfun-master fetch --depth 1 origin a89b783 && git -C xcfun-master checkout a89b783`.
Adds ~2-5 sec per CI run; preserves the cbindgen drift gate (the whole point
of `headers_match`). Path A (vendored subset) rejected as repo bloat;
Path C (feature gate) rejected as it would defeat the drift gate.

**Tasks 3 + 4 — mechanical.**
`cargo fmt --all` in one chunk (commit `fb517d2`); clippy site-by-site triage
(commit `0ea5eb0`) — three sites annotated with `#[allow]` + comment, four
sites genuinely stale and deleted.

**Task 5 — strict CI tightening.**
The first 4 commits were intended to land Task 5's "drop continue-on-error +
--locked + drop --exclude validation" together with Tasks 1-2 in commit
`bb13b74`. That commit was structurally complete, but a subsequent local
build on Rust 1.95 (released 2026-04-14, after the original 06-N6 work)
surfaced ~25 new clippy lints introduced in 1.95 that weren't visible at
commit time. Commit `af484bb` extends Task 5 to address these, completing
the strict-CI promise.

## Self-check

```
$ cargo fmt --all -- --check       # clean (exit 0)
$ cargo clippy --workspace --all-targets --locked -- -D warnings   # clean (exit 0)
$ cargo build --workspace --locked  # OK
$ cargo test --workspace --locked --release --lib   # all library unit tests pass
```

The two `cbrt_expand_x0_{0_1,10}` failures in `xcfun-ad`'s integration tests
(`tests/expand_primary.rs`) are pre-existing — bit-for-bit reproducible
both before and after this plan's commits. They're documented in project
memory as a `cubecl::Float::new(value: f32)` truncation gotcha; they are
out of scope for 06-N6 and tracked for a future fix.

## Commits in this plan

```
1d28f99 chore(06-N6): bump MSRV from 1.85 to 1.92 (cubecl-zspace 0.10.0 requirement)
bb13b74 ci(06-N6): add xcfun-master clone step; drop --exclude and continue-on-error
fb517d2 style(06-N6): cargo fmt --all (sweep pre-existing drift surfaced by 06-N5 CI)
0ea5eb0 fix(06-N6): triage 7 xcfun-kernels clippy unused-import/dead-code sites
af484bb fix(06-N6): address Rust 1.95 clippy lints surfaced by stable bump
```

## What this unblocks

Phase 7 Wave 1 (Plan 07-01) and beyond, including any future PR that
depends on a green strict CI baseline. The Rust 1.95 clippy sweep also
sets a precedent for how to handle future stable-Rust toolchain bumps:
prefer `#![allow]` for algorithm-faithful and explicit-harness style;
accept mechanical rewrites everywhere else.
