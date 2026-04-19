# Phase 1: Taylor Algebra & AD Primitives (`xcfun-ad`, cubecl-native) - Context

**Gathered:** 2026-04-19 (rewritten for cubecl pivot)
**Status:** Ready for planning
**Supersedes:** Previous 01-CONTEXT.md (2026-04-19 AM, hand-Rust port) — rendered invalid by the cubecl pivot. Previous decisions D-01 through D-22 are VOID. New decisions below.

<domain>
## Phase Boundary

Build the `xcfun-ad` crate as a **cubecl-native** AD engine: `CTaylor<F, N>` expressed as a pure `#[cube]` type backed by cubecl `Array<F>` storage, every arithmetic operation and every `*_expand` scalar series function written as a `#[cube] fn` generic over `F: Float`, validated on `cubecl-cpu` (`CpuRuntime`) against the C++ xcfun reference at **1e-12 strict relative error** (equivalent to the prior hand-Rust tolerance contract).

The single source of truth is the `#[cube]` code. There is no parallel hand-Rust scalar CTaylor implementation — `cubecl-cpu` is the scalar validation runtime. CUDA / Wgpu validation is explicitly out of scope for this phase and is Phase 6's domain.

Out of scope for this phase: functional ports, `DensVars`, registry, dispatch, per-functional `#[cube]` kernel bodies, CUDA enablement, Wgpu enablement, batch lifecycle — all downstream.

</domain>

<decisions>
## Implementation Decisions

### Numerical parity contract

- **D-01:** Relative-error contract against the C++ xcfun reference on `cubecl-cpu` is **1e-12**, matching the pre-pivot tolerance. No relaxation to 1e-9 or hybrid. If cubecl 0.10-pre.3's MLIR lowering cannot preserve this on the CPU runtime, the pivot fails at the fixture-gate and we revert to the hand-Rust plan rather than silently widen the tolerance.
- **D-02:** No `mul_add` / FMA in the AD layer. The crate-wide FMA suppression mechanism from the previous plan (`-Cllvm-args=-fp-contract=off` in `.cargo/config.toml`) is preserved. cubecl's emission of FMA or operation reordering MUST be disproven by CI evidence before Phase 2 consumes `xcfun-ad`. Researcher to investigate and the planner to include an asm-spot-check CI task per Phase 0's `-fp-contract=off` pattern.
- **D-03:** If cubecl-cpu is observed to fuse or reorder operations inside `CTaylor::mul`, the pivot halts — the failure is routed to the planner as a `PLANNING INCONCLUSIVE` with an explicit instruction to re-open the hand-Rust alternative rather than loosen D-01.

### `CTaylor<F, N>` type and storage

- **D-04:** `CTaylor<F: Float, const N: u32>` is a **pure `#[cube]` type**. No `#[derive(CubeType)]` host struct, no host-side `[F; 1 << N]` array, no `Copy` impl. Storage is `cubecl::prelude::Array<F>` of length `1 << N` allocated inside the kernel scope.
- **D-05:** Valid `N` range remains `0..=7` (storage `[F; 1..=128]`). Enforcement moves from the prior sealed `ValidN<N, SIZE>` trait to a compile-time `#[cube]`-compatible assertion via cubecl const-generic validation (researcher to confirm cubecl 0.10-pre.3's exact idiom; fallback is a `debug_assert!` at kernel entry).
- **D-06:** Bit-flag index constants (`CNST = 0`, `VAR0 = 1`, `VAR1 = 2`, …, `VAR6 = 64`) stay as host-visible `pub const` items in `xcfun_ad::index` so that downstream callers and Phase 6's batch API can reference them without entering a `#[cube]` scope. Inside `#[cube]` fns, they are passed as `#[comptime]` values.
- **D-07:** `#[repr(C)]` on host-side `CTaylor` is not applicable (no host struct). Downstream consumers that need byte layout (`xcfun-capi` FFI) receive coefficient arrays from `eval_vec` as plain `&[f64]` slices after kernel execution.
- **D-08:** Algorithmic-identity port of `ctaylor_rec<T, Nvar>::multo` from `xcfun-master/external/upstream/taylor/ctaylor.hpp` is preserved **inside** `#[cube]` fns. Every intermediate binding from the C++ recursion has a corresponding `let` in the `#[cube] fn ctaylor_mul`, per-order specialization for `N ∈ 0..=7`. No re-association, no parallel accumulation, no `cubecl::reduce::*` primitives on the critical path.

### `Num` trait and `F: Float` generic bound

- **D-09:** The custom `Num` trait from the pre-pivot design is **retired**. cubecl's `Float` trait (from `cubecl_core::prelude::Float`) replaces it. All `#[cube] fn` operating on `CTaylor` are generic: `#[cube] fn ctaylor_mul<F: Float, const N: u32>(a: &CTaylor<F, N>, b: &CTaylor<F, N>, out: &mut CTaylor<F, N>)`.
- **D-10:** `f32` is not forbidden at the `xcfun-ad` layer (cubecl's `Float` admits `f16`, `f32`, `f64`). The ban on `f32` on the numerical path is enforced **upstream** by `xcfun-rs::Functional` refusing to instantiate with any `F != f64`. `xcfun-ad` itself exposes a generic API so Phase 6's Wgpu research spikes (feature-gated, non-numerical-contract) can compile the same source.
- **D-11:** `cargo test -p xcfun-ad` and all fixtures run with `F = f64` only. Generic kernels that don't have f64 available on a given runtime return `XcError::Runtime` at instantiation — contract still lives at the facade, not the AD crate.

### `*_expand` scalar series ports

- **D-12:** Every `*_expand` from `xcfun-master/external/upstream/taylor/tmath.hpp` — `inv_expand`, `exp_expand`, `log_expand`, `pow_expand`, `sqrt_expand`, `cbrt_expand`, `gauss_expand`, `erf_expand` — is ported as a `#[cube] fn name_expand<F: Float>(args..., out: &mut Array<F>)` where `out` is a length-8 cubecl `Array`.
- **D-13:** Each ported expansion carries a doc-comment header with three items: (1) upstream `tmath.hpp` line range; (2) mathematical identity in LaTeX; (3) preconditions (e.g., `x0 > 0` for `log_expand`). Preconditions become `assert!`-equivalent `#[cube]` guards active in release builds (matches prior design D-11 intent — silent NaN is still the enemy).
- **D-14:** Composed elementary functions on `CTaylor` (`reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`) are implemented as `#[cube] fn ctaylor_<op><F, N>(x: &CTaylor<F, N>, out: &mut CTaylor<F, N>)` that internally call the corresponding scalar `*_expand` and a `#[cube] fn ctaylor_compose` equivalent to the C++ `ctaylor_rec::compose`. Verbatim operation order is preserved.

### Scalar evaluation call pattern

- **D-15:** Scalar `Functional::eval(point)` (pre-pivot: direct scalar Rust loop) becomes **a 1-thread kernel launch** that calls `eval_vec(&[point])` internally. ~10 μs overhead per call is accepted as the cost of the single-source-of-truth pivot. There is **no** parallel Rust scalar fallback.
- **D-16:** `xcfun-ad::for_tests::raw_eval_scalar<F: Float>(...)` helper wraps the common "launch a 1-thread kernel with these inputs, collect output array" pattern. Internal to the crate, used by test bodies and property tests — not part of the public API surface.

### Test infrastructure

- **D-17:** `xcfun-ad::for_tests::cpu_client()` exposes a `OnceLock<CpuClient>` initialized on first call and shared across every test in the binary. Matches cubecl-cpu's expected usage. No per-test client construction.
- **D-18:** Property tests (`proptest 1.11`) use the **batch-per-property** pattern: `proptest` generates 10,000 inputs upfront per property (ring axioms, exp/log round-trip, sqrt-squared invariance, Leibniz product rule, etc.), the inputs are packed into one cubecl buffer, a single kernel evaluates the property across all 10,000 points, and pass/fail is aggregated host-side. This preserves the prior design's ≥10 000 iterations without paying 10,000× kernel-launch overhead.
- **D-19:** Golden-coefficient fixtures (from the C++ driver in `xcfun-master/external/upstream/taylor/`) remain the 1e-12 parity oracle. Fixture generation via `cargo xtask regen-ad-fixtures` is unchanged (C++ driver writes `bincode` records to `xcfun-ad/tests/fixtures/*.bincode`); test-side loading now runs the expected coefficients through a cubecl kernel and compares.
- **D-20:** Criterion baselines (`CTaylor::mul`-equivalent kernel for `N ∈ {2, 3, 4, 5, 6}`, composed `exp`/`log`/`pow` at `N = 4`) measure **kernel-launch-amortized** throughput at batches of 1, 64, 1024 points. Baseline for Phase 6 tier-3 to compare GPU backends against.

### Existing work disposition

- **D-21:** Git commits `217af4d` (initial 7-plan set), `f07611c` (crate scaffold), `c7a3f46` (hand-Rust `CTaylor + ValidN`), `1b95fe3` (Wave 0 plan docs), `2db557c` (hand-Rust inv/exp/log expansions) and the untracked WIP in `crates/xcfun-ad/src/expand/{pow,sqrt}.rs` are **reverted** as the first wave of the new plan. The planner's Wave 0 MUST include a `git revert` task (producing new revert commits, not destructive rewrite) and a cleanup task that deletes `crates/xcfun-ad/src/` entirely before the cubecl port begins. No zombie hand-Rust code lurks alongside the cubecl version.
- **D-22:** Revert is non-destructive (original commits remain in history). Plan 01-01-SUMMARY.md is preserved as historical record — not deleted, marked "SUPERSEDED BY CUBECL PIVOT" in its header by the revert task.

### Phase 6 scope change

- **D-23:** Phase 6 is **retained**, scope narrowed. Per-functional `#[cube]` kernel bodies move forward into Phases 2–4 (they're natural extensions of the cubecl-native functionals). Phase 6's remaining scope:
  - Batch lifecycle (`Batch::open`, buffer pools, power-of-two growth)
  - `auto_backend` dispatch (CUDA when available, else Wgpu with `SHADER_F64`, else CPU)
  - CUDA enablement (tier-3 parity at 1e-13 vs CPU)
  - Wgpu enablement (tier-3 parity at 1e-9 + `erf` auto-fallback)
  - `Functional::eval_vec` dispatch heuristic (`nr_points >= 64` → batch)
- **D-24:** ROADMAP.md Phase 6 "Goal" and "Success Criteria" require an editorial update as part of Decision D-28 (below). Requirements KER-01..KER-06 and GPU-01..GPU-08 remain in Phase 6 unchanged.

### GPU validation timing

- **D-25:** Phase 1 is **cubecl-cpu only**. No CUDA smoke test. No Wgpu feature gating. CUDA and Wgpu tier-3 parity gates land in Phase 6 per the current roadmap.
- **D-26:** `xcfun-ad` features: `cpu` (default, activates `cubecl-cpu`), no `cuda`/`wgpu` features at the `xcfun-ad` level. Phase 6's backend crates will feature-gate their own runtime deps.

### Documentation updates (pre-planning)

- **D-27:** Before `/gsd-plan-phase 1` runs, the following planning artifacts are rewritten by the discuss-phase commit so the planner doesn't read conflicting locked decisions:
  - `.planning/STATE.md` — decisions D1, D2, D3, D8, D9, D14, D17 from Accumulated Context updated to reflect the pivot; D-Exec-01..D-Exec-03 struck (the underlying CTaylor struct shape is obsolete).
  - `.planning/ROADMAP.md` — Phase 1 "Goal" and "Plans:" list rewritten; Phase 6 "Goal" and "Success Criteria" narrowed; Phase 2-4 Success Criteria annotated that per-functional `#[cube]` kernel bodies land in those phases.
  - `.planning/REQUIREMENTS.md` — AD-01..AD-06 rewritten in-place to specify cubecl-native implementation; AD-01 un-ticked (the prior `[x]` is void).
- **D-28:** Follow-on docs to update (planner will include as dedicated plan tasks):
  - `docs/design/06-cubecl-strategy.md` — shared-spec section reflects cubecl-native AD as the baseline, not a downstream consumer.
  - `docs/design/07-accuracy-strategy.md` §2, §3 — algorithmic-identity language extended from "Rust scalar port" to "cubecl-cpu lowering".
  - `docs/design/12-design-decisions.md` D1, D2, D4 — record pivot; mark original D1 "in-house port, reject cubecl" as superseded with pointer to this CONTEXT.md.

### Claude's Discretion

- Exact cubecl-cpu runtime init module path within `xcfun-ad::for_tests` (`cpu_client()` naming is fine).
- Whether `*_expand` fns live in one module (`xcfun_ad::expand`) or one module per expansion (`xcfun_ad::expand::inv`, etc.) — planner picks based on cubecl 0.10-pre.3's `#[cube]` mod-nesting constraints.
- Criterion bench directory layout.
- Exact error variant names the test helpers return on kernel-launch failure (may reuse `XcError::Runtime` from `xcfun-core` if available; else a crate-local variant).

### Folded Todos

None surfaced by `gsd-tools todo match-phase` at init time.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### C++ reference (algorithmic-identity source of truth)

- `xcfun-master/external/upstream/taylor/ctaylor.hpp` — `ctaylor<T, Nvar>` struct; `ctaylor_rec<T, Nvar>::multo` recursion (the `#[cube] fn ctaylor_mul` port target); `operator+`, `operator-`, `operator*`, `operator<<`
- `xcfun-master/external/upstream/taylor/ctaylor_math.hpp` — Composed `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `erf` via `ctaylor_rec::compose`; target for `#[cube] fn ctaylor_<op>` ports
- `xcfun-master/external/upstream/taylor/tmath.hpp` — Every `*_expand`; target for `#[cube] fn <name>_expand<F: Float>` ports
- `xcfun-master/src/specmath.hpp` §1–3 — `pow2`, `pow3`, polynomial helpers (cross-reference)

### cubecl reference (pivot substrate)

- `docs/design/06-cubecl-strategy.md` — Project's cubecl plan (MUST be updated per D-28; read current state to understand pre-pivot intent vs new direction)
- cubecl-book `core-features/features.md` (`tracel-ai/cubecl`) — f64 support matrix per backend; relevant for Phase 6, read here for context
- `cubecl_core::prelude::{Float, Array, CUBE}` — trait/type signatures the new `xcfun-ad` builds on
- `cubecl_cpu::CpuRuntime`, `CpuClient` — test harness substrate per D-17

### Design brief (decisions and constraints)

- `docs/design/02-data-structures.md` §1 — `CTaylor<T, N>` layout and `Num` trait definition (pre-pivot; MUST be read to understand what's being replaced)
- `docs/design/07-accuracy-strategy.md` §2, §3 — Algorithmic-identity rule, tolerance budget (D-28 edits pending)
- `docs/design/10-build-and-dependencies.md` §3.1 — `xcfun-ad/Cargo.toml` pre-pivot deps (cubecl crates replace the `num-traits`/custom-`Num` backbone)
- `docs/design/12-design-decisions.md` D1, D2, D4 — Pre-pivot decisions; D-28 marks these superseded
- `docs/design/11-process-and-milestones.md` §M1 — Phase entry/exit criteria; planner updates these

### Research (pitfalls and phase mapping)

- `.planning/research/SUMMARY.md` "Implications for Roadmap" → Phase 1
- `.planning/research/PITFALLS.md` P1 (reassociation), P3 (CTaylor layout), P8 (cubecl drift), P9 (`*_expand` miscopy), P10 (silent NaN)
- `.planning/research/STACK.md` — cubecl pins (`=0.10.0-pre.3`)

### Project-level

- `.planning/PROJECT.md` — Core Value (1e-12 parity)
- `.planning/REQUIREMENTS.md` AD-01..06, ACC-05, ACC-06 — acceptance bar (AD-01..06 rewritten per D-27)
- `.planning/STATE.md` — decisions (updated per D-27)
- `CLAUDE.md` — tech-stack pins, `cubecl =0.10.0-pre.3` hard pin rationale and f64-on-Wgpu caveat

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`xcfun-master/external/upstream/taylor/`** — C++ source remains the port target. Read line-for-line, port into `#[cube] fn`.
- **`.cargo/config.toml` `-Cllvm-args=-fp-contract=off`** — carry forward; verify it also disables FMA in cubecl-cpu's JIT-compiled kernels (researcher).
- **`rust-toolchain.toml`, `deny.toml`** — Phase 0 scaffolding; unaffected by pivot.
- **`xtask` skeleton** — `regen-ad-fixtures` subcommand still drives the C++ fixture generator; fixture format unchanged.

### Established Patterns

- **One-source rule**: algorithmic-identity port; cubecl idioms are secondary to operation-order identity with the C++ reference.
- **No `unsafe`**: crate root `#![forbid(unsafe_code)]` preserved. cubecl macros expand to tokens the compiler checks; no hand-written unsafe inside `xcfun-ad`.

### Integration Points

- **`xcfun-core` (Phase 2)** consumes `xcfun-ad` via kernel launches, not via scalar `CTaylor<f64, N>` values. `DensVars::build` in Phase 2 becomes a `#[cube] fn` that internally constructs `CTaylor`s.
- **`xcfun-kernels` / `xcfun-gpu` (Phase 6)** reuse the exact `#[cube]` source that Phase 1 ships. No device-side `CTaylorDev<F, N>` — that type goes away; `CTaylor` already runs on any cubecl runtime.

### Pre-pivot code to remove (per D-21)

- `crates/xcfun-ad/src/{ctaylor.rs, valid_n.rs, for_tests.rs, lib.rs}`
- `crates/xcfun-ad/src/expand/{mod.rs, inv.rs, exp.rs, log.rs, pow.rs, sqrt.rs}` (last two are untracked WIP)

</code_context>

<specifics>
## Specific Ideas

- Kernel-name prefix: `xcfun_ad_` (e.g., `xcfun_ad_ctaylor_mul`) so cubecl-compiler error messages mention the right crate.
- Keep `VAR0..VAR6`, `CNST` identifiers byte-identical to C++ headers; passed as `#[comptime]` into `#[cube]` fns.
- Every ported expansion keeps the three-item header comment (upstream line range, LaTeX identity, preconditions). Reviewer's checklist.
- Fixture records: preserve `{ op: String, n_var: u8, inputs: Vec<f64>, coeffs: Vec<f64> }` schema so the C++ driver is unchanged.

</specifics>

<deferred>
## Deferred Ideas

- **`f32` on the cubecl path** — generic `F: Float` keeps the door open; actual `f32` instantiation explicitly deferred to Phase 6's Wgpu research spike. No `f32` kernel compiled by `xcfun-ad` tests.
- **SIMD inside `CTaylor::mul`** — deferred to v2; cubecl already vectorizes on cubecl-cpu via MLIR; revisit if criterion baselines show headroom.
- **Nested `CTaylor<CTaylor<F, M>, N>`** — not supported in the cubecl-native form; not required by any downstream phase.
- **Hand-Rust CTaylor as a redundant oracle** — rejected per D-21. If pivot tolerance ever drifts above 1e-12, the escape hatch is to restore via `git revert` of the revert commits, not to maintain a parallel implementation.
- **Criterion bench regression gate** — deferred to v2 (PERF-01 in REQUIREMENTS.md).

### Reviewed Todos (not folded)

None reviewed at this session.

</deferred>

---

*Phase: 01-taylor-algebra-ad-primitives-xcfun-ad*
*Context rewritten for cubecl pivot: 2026-04-19 (discuss mode, 11 interactive decisions logged — all user-selected)*
