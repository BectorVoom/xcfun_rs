# Phase 6: GPU Backends + Batch Lifecycle (`xcfun-kernels` / `xcfun-gpu`) - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 6 lights up the GPU runtime + batch evaluation layer on top of the cubecl-native AD substrate (`xcfun-ad`) and per-functional `#[cube]` bodies (Phases 2–4) that are already in place. It also closes out the 30+ D-19 numerical-parity forwards from Phases 3/4 that were explicitly deferred here, and resurrects the original `docs/design/05-module-responsibilities.md` crate layout by extracting `xcfun-kernels` from `xcfun-eval`.

Three deliverable axes stack:

1. **Algebraic substrate** — `xcfun-ad` `ctaylor_compose` / `ctaylor_multo` specialisations for `N ∈ {4, 5, 6}` (unblocks `Mode::Contracted` orders 5..=6 metaGGA per Phase 4 D-19); in-kernel libm-hybrid `erf` at tightened precision (resolves LDAERF order-3 AD-chain amplification per Phase 2 + Phase 4 D-19); `tau ≥ tau_w` hard-clamp guard inside TPSS-correlation kernels (resolves TPSSC/TPSSLOCC/REVTPSSC unphysical-regime divergence per Phase 4 D-19); mpmath-truth fixture generator in `xtask` (offline tool emitting JSONL fixtures at 200-digit precision for documented-cancellation regimes — supports the ACC-04 amendment).

2. **Crate reorganisation + GPU runtimes** — Create `crates/xcfun-kernels/` and migrate the 78 functional bodies + `DensVarsDev` + `dispatch_kernel` from `xcfun-eval/src/functionals/`; `xcfun-eval` retains only `Functional` + per-point eval + cubecl-cpu validation substrate. Unstub `crates/xcfun-gpu/` with `Backend` enum, `Batch<'fun, R: cubecl::Runtime>`, `auto_backend()`, generation-counter-guarded buffer pool, ERF-fallback routing. Wire cubecl-hip (primary), cubecl-cuda + cubecl-metal (opt-in), cubecl-wgpu (portable fallback) behind feature flags.

3. **`Functional::eval_vec` + zero-alloc + cleanup** — RS-08 dispatch through `xcfun-gpu::Batch<R>` when `nr_points ≥ 64`. Strict zero-alloc per-point form via pre-allocated reusable handle in `Functional` (Phase 5 D-13 forward, ~287 → 0 allocs/eval). Three D-19 cleanup plans: root-cause bisection of inherited Phase-3 forwards (PBEINTC/BECKESRX/P86C/etc.), mpmath-only fixtures for the 20 `excluded_by_upstream_spec` functionals (BR×3 + SCAN×10 + CSC + BLOCX + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC), post-libm-hybrid sweep verifying small-magnitude AD-residuals tighten to 1e-13.

**In scope:** AD N≥4 specialisations, libm-hybrid `erf`, `tau ≥ tau_w` regularization guard, mpmath fixture pipeline, `xcfun-kernels` crate creation + functional-body migration, `xcfun-gpu` unstub + `Backend` + `Batch` + buffer pool + `auto_backend`, cubecl-hip wiring (primary), cubecl-cuda + cubecl-metal opt-in feature flags, cubecl-wgpu portable fallback with f64 gate, RS-08 (`Functional::eval_vec`) GPU dispatch + threshold, strict zero-alloc per-point form, Phase 5 weights `Box` leak refactor (`&'static [...]` → `Vec<...>`), LDA-vars=6 launch arms (or DensVars-driven dispatch) for mixed-LDA+GGA aliases (b3lyp / camb3lyp / bp86), tier-3 parity gate sweeping all 78 functionals at strict 1e-13, ACC-04 amendment for mpmath ground truth where C++ documents cancellation, ROADMAP / REQUIREMENTS / PROJECT.md / CLAUDE.md updates for ROCm-primary GPU strategy.

**Out of scope (Phase 7+):** Python bindings (`xcfun-py` → Phase 7); release ceremony / crates.io publish (Phase 7); stream-overlapped async GPU dispatch (PROJECT.md out-of-scope, < 20% throughput gain for 3× API complexity); `Line<F>` lane vectorisation (`docs/design/06 §8` — revisit in v2); shared-memory reductions (none in xcfun's API surface); CUDA tier-3 strict-1e-13 local validation (no NVIDIA hardware in dev environment; community-maintained best-effort via cloud CI when available); Metal tier-3 strict-1e-13 (Apple Silicon GPUs lack hardware f64; opt-in best-effort with f64-probe gate); patches to `xcfun-master/` C++ source (vendored content-hash invariant preserved).

</domain>

<decisions>
## Implementation Decisions

### Phase scope structure

- **D-01:** **Wide Phase 6 — single GSD phase, ~10–15 plans** (decimal numbering is plan organisation, not sub-phases). Plan tree:
  - Plan 06-00 = full algebraic substrate (AD N≥4 specialisations + libm-hybrid `erf` + `tau ≥ tau_w` guard + mpmath fixture generator) in CURRENT `xcfun-eval/src/functionals/` tree.
  - Plan 06-01 = create `crates/xcfun-kernels/` + `git mv` `functionals/` + `density_vars/` + `dispatch.rs` from `xcfun-eval` → `xcfun-kernels` + import-fixup across workspace + tier-1 self-test path updates.
  - Plan 06-02 = unstub `crates/xcfun-gpu/` + `Backend` enum + `Batch<'fun, R>` skeleton + buffer pool + `auto_backend()` + `XcError::WgpuNoF64` typed variant addition.
  - Plan 06-03 = cubecl-hip primary wiring (default GPU backend) + ROCm tier-3 parity harness extension + RDNA-2 `HSA_OVERRIDE_GFX_VERSION` doc note.
  - Plan 06-04 = cubecl-cuda opt-in feature + cubecl-metal opt-in feature + cubecl-wgpu portable fallback feature + `SHADER_F64` runtime probe.
  - Plan 06-05 = RS-08 `Functional::eval_vec` GPU dispatch + threshold (env `XCFUN_MIN_BATCH_SIZE`, default 64) + ERF auto-fallback routing on Wgpu/Metal.
  - Plan 06-06 = strict zero-alloc per-point form (pre-allocated reusable handle in `Functional` per D-08 below; ~287 → 0 allocs/eval) + Phase-5 weights `Box` leak refactor (`&'static` → `Vec<...>`) + LDA-vars=6 launch arms (or DensVars-driven dispatch) for mixed-LDA+GGA aliases.
  - Plan 06-N1 = D-19 cleanup — root-cause bisection of inherited Phase-3 forwards (PBEINTC `6.2e+1`, BECKESRX `2.3e+2`, P86C `9.2e-2`, P86CORRC, PW91C `1.7e-3`, SPBEC `5.3e-4`, APBEC `5.7e-9`, B97{,_1,_2}C `7.8e-11`, PW91K `1.4e-11`); Path-B style side-by-side reads of upstream C++ vs Rust ports.
  - Plan 06-N2 = mpmath-only fixture pipeline for the 20 `excluded_by_upstream_spec` functionals (BR×3 + CSC + BLOCX + SCAN×10 + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC). C++ harness aborts on these density strata; mpmath is the sole reference.
  - Plan 06-N3 = post-libm-hybrid sweep verifying ~12 small-magnitude AD-residuals (M05/M06 family, B97 family, LYPC, PW92C, PBEC, OPTX) tightened to 1e-13; bisects any that didn't tighten.

- **D-02:** **Strict 1e-13 across all 78 functionals at Phase 6 sign-off.** Unphysical regimes (TPSSC/TPSSLOCC/REVTPSSC where tau ≪ tau_w; LDAERF where C++ documentably suffers cancellation) get **kernel-level regularization guards or mpmath ground-truth substitution** — never stratum exclusion. The bar is enforced via `cargo run -p validation --release -- --backend rocm --order 3 --filter '.*'` + tier-3 parity GREEN.

### Ground-truth policy

- **D-03:** **Amend ACC-04: mpmath truth where C++ documents cancellation.** Default ground truth stays C++ xcfun. For density points where the C++ source explicitly notes bracket cancellation (e.g., `xcfun-master/src/functionals/ldaerfx.cpp:66` `test_threshold` rationale), tier-2 / tier-3 reference switches to mpmath at 200-digit precision computed offline and committed as JSONL fixtures. Strict 1e-13 = `|rust − mpmath| / max(|mpmath|, 1) ≤ 1e-13` for those points; `|rust − cpp|` elsewhere. Preserves algorithmic-identity contract (Rust does NOT replicate C++ bugs); resolves LDAERFX/LDAERFC/LDAERFC_JT order-3 catastrophic divergence without per-functional tolerance widening. Phase 0 ACC-04 wording, REQUIREMENTS.md ACC category, and `docs/design/07-accuracy-strategy.md §1 + §5` updates required.
- **D-04:** **mpmath fixture generator landed in `xtask` during Plan 06-00.** Offline tool: takes `(functional, vars, mode, order, density)` tuple lists, evaluates against mpmath at 200-digit precision, commits JSONL fixtures under `validation/fixtures/mpmath/`. Re-run via `cargo xtask regen-mpmath-fixtures`. mpmath dependency lives in `xtask` only (a Python sidecar via `subprocess::Command::new("python3").arg("-m").arg("xtask.mpmath_eval")`). NOT a runtime/library dep — `cargo build` of `xcfun-rs` / `xcfun-capi` / `xcfun-py` does NOT require Python.

### GPU backend strategy

- **D-05:** **ROCm/HIP is the PRIMARY GPU backend.** `cubecl-hip = "=0.10.0-pre.3"` (feature flag `hip`) carries the strict 1e-13 tier-3 contract. `Backend::Rocm` + `HipRuntime` are first-class. Dev/CI loops run on ROCm; RDNA-2 GPUs need `HSA_OVERRIDE_GFX_VERSION=10.3.0` documented in `xcfun-gpu/README.md`. Reason: project dev environment is AMD; no CUDA hardware available locally. ROADMAP / REQUIREMENTS / PROJECT.md / CLAUDE.md / `docs/design/06-cubecl-strategy.md §2` rename `Cuda` → `Rocm` in primary path.
- **D-06:** **CUDA + Metal as opt-in best-effort feature flags.** `cubecl-cuda = "=0.10.0-pre.3"` (feature `cuda`) ships for NVIDIA users; tier-3 only via cloud CI when available, no local validation. `cubecl-metal = "=0.10.0-pre.3"` (feature `metal`) ships for macOS / Apple Silicon. **Apple Silicon caveat:** Apple Silicon GPUs lack hardware f64; cubecl-metal must runtime-probe f64 support and refuse if absent (analogous to Wgpu `SHADER_F64` gate). Both are community-maintained; numerical contract relaxes to "best-effort" for these — never gate the primary v1 release on either.

### Amended on revision-1 (2026-04-30)

**D-06 correction:** `cubecl-metal` does NOT exist as a separate crate on crates.io (verified by RESEARCH §"Standard Stack" + Pitfall 9 + R-02). Metal is reached via `cubecl-wgpu`'s Metal adapter; the `metal` cargo feature in `xcfun-gpu` is an alias of `wgpu` (`metal = ["wgpu"]`). The original D-06 text above mentioning `cubecl-metal = "=0.10.0-pre.3"` is preserved for audit-trail but superseded by this amendment. All Plan 06-02a / 06-04 implementation work uses the `metal = ["wgpu"]` alias model. The `metal` cargo feature is therefore a transparent alias of `wgpu` and pulls the same `cubecl-wgpu` crate under the hood; the runtime probe distinguishes Metal-adapter f64 support via `wgpu::Features::SHADER_F64` exactly as for any other Wgpu adapter.

- **D-07:** **`auto_backend()` priority order:** env `XCFUN_FORCE_BACKEND` → ROCm-if-available → CUDA-if-available-and-`cuda`-feature → Metal-if-available-and-f64-and-`metal`-feature → Wgpu-if-`SHADER_F64`-and-`wgpu`-feature → CPU. Documented in `xcfun-gpu/src/auto_backend.rs` doc-comment.

### Crate boundary (resurrect design-doc-05 layout)

- **D-08:** **Full `xcfun-kernels` + `xcfun-gpu` split.** Create `crates/xcfun-kernels/` (currently absent). Migrate from `xcfun-eval/src/`:
  - `functionals/` (78 kernels in `lda/`, `gga/`, `mgga/`)
  - `density_vars/` and `density_vars.rs` (`DensVarsDev<F>` + `build_densvars` + `regularize`)
  - `dispatch.rs` (FunctionalId-keyed `dispatch_kernel`)

  `xcfun-kernels` exposes: `#[cube] fn` kernel bodies, `DensVarsDev<F>` `#[derive(CubeType, CubeLaunch)]`, `dispatch_kernel<F>`. **Never instantiates a runtime; never depends on cubecl-cpu / cubecl-hip / cubecl-cuda / cubecl-metal / cubecl-wgpu directly — only on `cubecl` core.**

  `xcfun-eval` keeps: `Functional` struct, per-point `eval` entry point, `eval_point_kernel` `#[cube(launch_unchecked)]` adapter, `for_tests::cpu_client()` (promoted to production). Depends on `xcfun-kernels` + `cubecl-cpu`. The CPU per-point validation substrate.

  `xcfun-gpu` (unstubbed): `Backend` enum, `Batch<'fun, R: cubecl::Runtime>`, buffer pool with generation counter, `auto_backend()`, ERF-fallback routing. Depends on `xcfun-kernels`. Feature flags `hip` / `cuda` / `metal` / `wgpu` pull cubecl-hip/cuda/metal/wgpu in turn. `cubecl-cpu` always available (re-exported from `xcfun-eval`).

  `xcfun-rs` updates: facade depends on `xcfun-eval` (per-point) AND `xcfun-gpu` (batch via RS-08). Re-exports both surfaces under the existing `Functional` newtype.

- **D-09:** **Migration order: substrate FIRST, move SECOND** (per D-01 plan tree). Plan 06-00 lands AD N≥4 + libm-hybrid + tau guards + mpmath fixtures in CURRENT `xcfun-eval/src/functionals/` tree. Plan 06-01 then performs the `git mv` + import-fixup with no concurrent algebraic changes. Avoids merge-conflicts between substrate work and structural reorg; reorg-induced regressions are bisectable as "move-bug" vs "substrate-bug".

### Numerical / kernel guards

- **D-10:** **TPSS-correlation `tau ≥ tau_w` hard-clamp guard.** Inside `crates/xcfun-eval/src/functionals/mgga/{tpssc.rs, tpssloc.rs, revtpssc.rs}` (or post-Plan-06-01 location in `xcfun-kernels`), insert at the top of each kernel body: `let tau_clamped = ctaylor_max(tau, tau_w);` then use `tau_clamped` everywhere `tau` was previously used. Resolves the unphysical regime (von Weizsäcker bound violation by ~9 orders of magnitude) where f64-rounding cancellation in `eps_pkzb * (1 + 2.8 * eps_pkzb * tauwtau3)` amplifies ULP differences to `1e+30`. Algorithmically faithful: TPSS correlation is undefined for tau < tau_w in the physical interpretation; the guard makes the kernel return the limiting value at the bound. C++ does not have this guard, so this is one of the points where Rust diverges intentionally; ACC-04 amendment (D-03) covers this with mpmath-truth verification at the boundary.
- **D-11:** **Libm-hybrid `erf` extension.** Phase 2 Plan 02-06 commit `dca382a` already landed an in-kernel FreeBSD msun-derived `erf_precise` (1e-14 baseline). Phase 6 extends this to handle order-3 AD-chain amplification: introduce `erf_precise_taylor<F: Float, const N: u32>(x: CTaylor<F, N>) -> CTaylor<F, N>` that uses the stable-bracket bracket-reduction technique (analogous to Plan 02-06 Fix 1 `expm1`-stable LDAERFX) inside the AD chain. Resolves LDAERFX `6.7e-2`, LDAERFC `4.6e-6`, LDAERFC_JT `4.6e-5` at order 3. Lands in `xcfun-ad::math` (cubecl-native, `#[cube] fn`).

### Batch + dispatch

- **D-12:** **Pre-allocated reusable handle in `Functional` (zero-alloc strategy).** `Functional` gains private mutable buffers (`input_buf: Array<F>`, `out_buf: Array<F>`, `dens_vars: DensVarsDev<F>`) sized at `eval_setup` time per `(vars, mode, order)`. `eval(&self, ...)` keeps the `&self` signature; interior mutability via `SyncUnsafeCell` (or `RefCell` if Send+Sync gate weakens). cubecl-cpu `CpuClient` stays in `for_tests`-promoted-to-production `OnceLock<CpuClient>`. **Trade-off:** one `Functional` per thread to avoid contention; document in RS-10 as "Functional is Send + Sync, but eval() is racy if called concurrently on the same instance — clone the Functional or wrap in Mutex for concurrent eval." `static_assertions::assert_impl_all!(Functional: Send, Sync)` retained. Reaches strict 0 allocs/eval after first call (RS-07 contract met).
- **D-13:** **Wgpu `SHADER_F64`-missing → typed `XcError::WgpuNoF64`.** Add to `XcError` enum (xcfun-core): `XcError::WgpuNoF64 { adapter_name: String, requested_runtime: Backend }` (the `String` payload breaks Phase 2 D-25 `Copy` constraint — D-13-A below resolves this). Returned by `Batch::open` when the selected runtime is Wgpu but device lacks `wgpu::Features::SHADER_F64`. Caller pattern-matches and decides (downgrade to CPU, log + skip, etc.). Compile-time `const _: () = assert!(size_of::<Scalar>() == 8);` in `xcfun-kernels` root. No silent f32 fallback.
- **D-13-A:** **`XcError::WgpuNoF64` payload as `&'static str` (not `String`).** To preserve Phase 2 D-25 `Copy + non_exhaustive`, the variant carries `adapter_name: &'static str` filled by `cubecl-wgpu`'s adapter info (`AdapterInfo::name` is `String` upstream — wrap via `Box::leak` once at runtime, justified by being a one-time panic-on-misconfiguration message). `requested_runtime: Backend` is `Copy` (enum). XcError stays `Copy`.
- **D-14:** **`eval_vec` dispatch threshold = 64 (default), env-overridable via `XCFUN_MIN_BATCH_SIZE`** (per `docs/design/06 §12`). Threshold is a compile-time `const` in `xcfun-rs` with a runtime override path; below threshold falls through to per-point `eval` loop, at-or-above dispatches to `xcfun-gpu::Batch<R>`. R = `auto_backend()` unless caller pre-selects.
- **D-15:** **Buffer pool grows powers-of-two doubling** (per `docs/design/06 §5.1` + §7). `density_buf` / `result_buf` capacities double on overflow; never shrink. `weights_buf` (82 f64) + `active_ids_buf` (78 u32) are fixed-size, allocated once. **Generation counter:** monotonic `u64` on `Functional::settings`; `Batch` re-uploads `weights_buf` only when its cached generation is stale. (Hash-based comparison rejected: 82 f64 = 656 bytes — simpler to track generation than to hash on every launch.)
- **D-16:** **`xcfun-rs::Functional::eval_vec` matches upstream `xcfun_eval_vec` signature.** Per RS-08 + the C ABI drop-in contract (CAPI-01..02): `eval_vec(&self, density: &[f64], density_pitch: usize, out: &mut [f64], out_pitch: usize, nr_points: usize) -> Result<(), XcError>`. Pitched layout matches `xcfun-master/api/xcfun.h:54` exactly. Phase 7 (`xcfun-py`) consumes via PyO3 with NumPy 2-D `f64` `ndarray[order='C']` (PY-03); Python adapter computes pitches from `ndarray.strides`.

### Phase 5 substrate forwards

- **D-17:** **`xcfun-eval::Functional::weights` refactor: `&'static [(FunctionalId, f64)]` → `Vec<(FunctionalId, f64)>`** (lands in Plan 06-06). Drops the documented Phase 5 `Box::leak` in `sync_weights_from_settings` (one leak per `set` call). Send + Sync preserved (`Vec<(FunctionalId, f64)>` is Send + Sync).
- **D-18:** **LDA-vars=6 launch arms via DensVars-driven dispatch.** Currently `xcfun-eval::dispatch::run_launch` only has LDA at `vars=2` and GGA at `vars=6`; mixed-LDA+GGA aliases (b3lyp = LDA-VWN5 + GGA-Becke + GGA-LYP; camb3lyp; bp86) currently route through the C++ validation harness only. Phase 6 adds DensVars-driven dispatch: a kernel's `Dependency` mask determines which Vars subset arms it can launch into. LDA kernel → Vars subset where `Dependency::DENSITY ⊆ vars_dep_mask`. GGA kernel → Vars subset where `Dependency::DENSITY | Dependency::GRADIENT ⊆ vars_dep_mask`. The dispatcher computes the subset at `eval_setup` time. Resolves the Phase-5 D-14 dispatch-table constraint forward.

### Claude's Discretion

- The exact form of `SyncUnsafeCell` vs `RefCell` for the per-point reusable handle (Send + Sync trade-off) — implementer picks based on the RS-10 contract that lands in Plan 06-06.
- Whether `Backend` enum lives in `xcfun-core` (matches `Mode` / `Vars` / `Dependency` precedent) or `xcfun-gpu` (closer to where it's consumed) — pick whichever reads cleaner.
- cubecl-feature-flag default in `xcfun-gpu` (recommend `default = ["cpu"]`; `hip` / `cuda` / `metal` / `wgpu` opt-in via Cargo features) but other defaults are fine.
- Layout of mpmath-fixture JSONL files (record-per-line vs nested vs separate files per functional) — pick whatever the validation harness reads cleanest.
- Threshold for "small-magnitude residual" in the Plan 06-N3 sweep — anything between 1e-15 and 1e-9 is reasonable; Phase 4 sign-off used 1e-12 as the strict bar so 1e-13 is the natural cleanup target.
- Plan ordering of 06-N1/N2/N3 (parallel vs sequential) — parallel is fine because they touch independent functional sets; sequential is fine for review tractability.
- Which xcfun-ad N≥4 specialisations carry which `#[comptime]` constants (recommend N=4, N=5, N=6 each get their own `#[cube] fn` instance to avoid runtime branching; macro-generated if repetitive).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner, executor) MUST read these before planning or implementing.**

### C++ reference (algorithmic-identity source of truth, except where ACC-04 amendment applies)

- `xcfun-master/src/XCFunctional.cpp:493-617` — `Mode::PartialDerivatives` output layout for orders 0..=4 (relevant when extending dispatch table for LDA-vars=6 arms per D-18).
- `xcfun-master/src/XCFunctional.cpp:614-617` — order 4 fall-through pattern (Phase 3 MODE-01 cap reasoning).
- `xcfun-master/src/XCFunctional.cpp:622-627` — `Mode::Contracted` `inlen × (1 << order)` input layout (referenced by Phase 5 D-15-A and reused for Mode::Contracted orders 5..=6 verification post Plan 06-00 AD N≥4 substrate).
- `xcfun-master/external/upstream/taylor/ctaylor.hpp` — `ctaylor_rec::{multo, multo_skipconst, compose}` recursion structure. **Phase 6 extends `compose` and `multo` outer dispatch to N ∈ {4, 5, 6} per Plan 06-00 Task 1 (D-19 Phase-4 forward; Plan 04-05 reinforcement).**
- `xcfun-master/external/upstream/taylor/tmath.hpp` — `*_expand` family. The 20 `excluded_by_upstream_spec` functionals trip `tmath::sqrt_expand` / `log_expand` / `pow_expand` aborts; Plan 06-N2 mpmath-only path bypasses C++ harness for these.
- `xcfun-master/src/functionals/ldaerfx.cpp:66` — `test_threshold` documenting bracket cancellation (justifies ACC-04 mpmath amendment per D-03).
- `xcfun-master/src/functionals/{tpssc.cpp, tpssc_eps.hpp, pbec_eps.hpp}` — line-for-line ports verified by Phase 4 Plan 04-10 Path-B bisection. Phase 6 D-10 `tau ≥ tau_w` guard inserted RUST-side; C++ has no equivalent guard (ACC-04 mpmath verification at boundary per D-03).
- `xcfun-master/src/functionals/{lb94.cpp, br*.cpp, csc.cpp, blocx.cpp, scan*.cpp}` — bodies for the 20 `excluded_by_upstream_spec` functionals. Plan 06-N2 mpmath fixture generator computes truth offline.
- `xcfun-master/api/xcfun.h:54` — `xcfun_eval_vec` C signature with density_pitch / out_pitch (D-16 RS-08 alignment).
- `xcfun-master/src/specmath.hpp` — `poly` descending Horner (`docs/design/07-accuracy-strategy.md §2` row).

### Design docs (project-internal contracts; UPDATE during Phase 6)

- `docs/design/05-module-responsibilities.md` — **PRIMARY ARCHITECTURE REFERENCE.** Phase 6 D-08 resurrects the `xcfun-kernels` + `xcfun-gpu` split this doc specifies. The doc may pre-date the cubecl pivot; verify the per-crate responsibilities match the cubecl-native architecture and update §xcfun-kernels / §xcfun-gpu sections at sign-off.
- `docs/design/06-cubecl-strategy.md` — Runtime matrix (§2; **UPDATE: replace `Cuda` primary with `Rocm`; list CUDA + Metal as opt-in**), batch lifecycle (§5; D-15 powers-of-two growth + generation counter), kernel-resident buffers (§7), launch config (§10), numerical parity across backends (§11; **UPDATE: Wgpu 1e-9 retained; ROCm 1e-13 primary; CUDA + Metal best-effort**), `eval_vec` threshold (§12).
- `docs/design/07-accuracy-strategy.md` — **§1 invariant + §5 fixtures get the ACC-04 amendment.** D-03 mpmath-truth substitution language goes here. §6 tolerance budget unchanged (still 1e-12 worst-case CPU; 1e-13 cross-backend; 1e-9 Wgpu range-separated).
- `docs/design/08-error-model.md` — `XcError` variants. D-13 / D-13-A add `WgpuNoF64 { adapter_name: &'static str, requested_runtime: Backend }`; doc update at sign-off.
- `docs/design/09-testing-strategy.md` — Tier-3 cross-backend parity expected behaviour. **UPDATE: ROCm tier-3 strict-1e-13; CUDA + Metal tier-3 best-effort with cloud-CI gate language; mpmath fixtures (Plan 06-00 / 06-N2) referenced as authoritative for the 20 + ERF cases.**
- `docs/design/10-build-and-dependencies.md` — Add cubecl-hip + cubecl-cuda + cubecl-metal at `=0.10.0-pre.3`; update cubecl-pin invariant (Plan 02-02 `xtask check-cubecl-pin`) to enforce all four cubecl-runtime crates lock-step at the same exact version.

### Project-wide planning artefacts

- `.planning/ROADMAP.md` §"Phase 6: GPU Backends + Batch Lifecycle" — 5 success criteria, 15 requirements (RS-08, KER-01..06, GPU-01..08). **UPDATE: success criterion 4 renames `CUDA` → `ROCm` for primary; CUDA + Metal listed as opt-in best-effort. Sign-off bar = strict 1e-13 across all 78 functionals (per D-02).**
- `.planning/REQUIREMENTS.md` — RS-08, KER-01..06, GPU-01..08 wording. **UPDATE: GPU-02 `auto_backend()` priority order per D-07; GPU-04 ROCm tier-3 1e-13; GPU-07 wording (Cuda → Rocm); GPU-08 Wgpu unchanged at 1e-9; ACC-04 amendment per D-03.**
- `.planning/PROJECT.md` Constraints §GPU — **UPDATE:** "`cubecl-hip` for AMD/ROCm primary; `cubecl-cuda` + `cubecl-metal` as opt-in feature flags for NVIDIA / Apple users; `cubecl-wgpu` portable fallback with f64 device gate; f32 never on numerical path".
- `.planning/PROJECT.md` Key Decisions table — **UPDATE:** "CUDA primary GPU target; Wgpu best-effort..." → "ROCm primary GPU target; CUDA + Metal opt-in; Wgpu best-effort..." (matches memory: `project_gpu_target.md`).
- `CLAUDE.md` (root) — Tech stack table. **UPDATE:** add `cubecl-hip = "=0.10.0-pre.3"` (primary), `cubecl-cuda = "=0.10.0-pre.3"` (opt-in), `cubecl-metal = "=0.10.0-pre.3"` (opt-in); CLAUDE.md "Risk Assessment" table gets two new rows for ROCm RDNA-2 driver requirement and Apple Silicon f64-absence; "Stack Patterns by Variant" gets a ROCm-primary build pattern.
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md` — D-04 (xcfun-core data-only stays); D-21 (Functional in xcfun-eval moves toward xcfun-eval keeping Functional + xcfun-kernels owning bodies per Phase 6 D-08); D-22 (regularize on `c[CNST]` only — referenced by D-10 tau guard which is a similar in-kernel guard pattern); D-25 (XcError 9 variants Copy + non_exhaustive — D-13-A preserves this).
- `.planning/phases/03-gga-tier-mode-potential/03-CONTEXT.md` — D-19 (LB94 + 13 forwards forwarded to Phase 5/6; the 11 still-failing-at-order-3 list flows into Plan 06-N1).
- `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-19 (30+ forwards consolidated; TPSS gradient-stress AD-chain divergence; 20 `excluded_by_upstream_spec`); Plan 04-05 D-19 (Mode::Contracted orders 5..=6 metaGGA xcfun-ad N≥4 specialisations forwarded — Plan 06-00 Task 1).
- `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md` — D-13 (zero-alloc fall-back form b → strict form here per D-12); D-14 (LB94→LDA Mode::Potential substitute; rows 4 + 9 vars-substitution per dispatch-table constraint — D-18 LDA-vars=6 / DensVars-driven dispatch resolves); D-17 (Functional Send + Sync — D-12 preserves).
- Memory: `.claude/projects/-home-chemtech-workspace-xcfun-rs/memory/project_gpu_target.md` (ROCm primary), `project_crate_layout.md` (xcfun-kernels + xcfun-gpu split).

### cubecl-specific references (verify during Plan 06-00 / 06-02 / 06-04 research)

- `https://github.com/tracel-ai/cubecl/blob/main/cubecl-book/src/getting-started/installation.md` — installation matrix; ROCm Linux Quick Start; Metal macOS 10.13+; CUDA toolkit verify via `nvidia-smi`.
- `https://github.com/tracel-ai/cubecl/blob/main/cubecl-book/src/core-features/features.md` — datatype / feature matrix; f64 row "support varies" across CUDA / ROCm / WGPU; Plane / CMMA / TensorAccelerator support.
- `https://github.com/tracel-ai/cubecl/blob/main/crates/cubecl-hip/README.md` — RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0` workaround; rocwmma compile-time concerns.
- `https://github.com/tracel-ai/cubecl/blob/main/crates/cubecl-wgpu/README.md` — wgpu Metal/Vulkan/OpenGL/WebGPU backend matrix.
- `docs/manual/Cubecl/` — vendored authoritative cubecl reference.

### mpmath references (Plan 06-00 + Plan 06-N2)

- mpmath at 200-digit precision invoked via Python sidecar in `xtask`. No URL pin needed (mpmath is API-stable over decades). Phase 2 already verified mpmath-vs-Rust agreement at LDAERFX (2026-04-21 finding referenced in `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md`).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`crates/xcfun-eval/src/for_tests.rs::cpu_client()`** — `OnceLock<CpuClient>` pattern wrapping `cubecl-cpu`'s `CpuRuntime::client(&CpuDevice)`. **Promoted from `for_tests` to production** in Plan 06-06; same pattern reused in `xcfun-gpu` for `HipClient` / `CudaClient` / `MetalClient` / `WgpuClient` `OnceLock`s.
- **`crates/xcfun-eval/src/functional.rs:54` `eval_point_kernel`** — existing `#[cube(launch_unchecked)]` adapter that builds DensVarsDev from flat input + dispatches to per-functional kernel. The CPU per-point launcher; survives the Phase 6 reorg in `xcfun-eval` (D-08).
- **`crates/xcfun-eval/src/dispatch.rs::dispatch_kernel`** — FunctionalId-keyed `match` arms calling `<name>_kernel<F>(d, out, n)`. Migrates to `xcfun-kernels::dispatch::dispatch_kernel` in Plan 06-01.
- **`crates/xcfun-eval/src/density_vars/`** — `DensVarsDev<F>` `#[derive(CubeType, CubeLaunch)]` + `build_densvars` + `regularize`. Migrates to `xcfun-kernels::density_vars/` in Plan 06-01.
- **`crates/xcfun-eval/src/functionals/`** — 78 functional kernel bodies in `lda/`, `gga/`, `mgga/`. Migrates to `xcfun-kernels::functionals/` in Plan 06-01.
- **`crates/xcfun-ad/src/`** — cubecl-native `CTaylor<F, N>` + `ctaylor_rec::{mul, multo, compose}` + `expand::*` + `math::*` (composed ops). **Plan 06-00 Task 1 extends `ctaylor_compose` / `ctaylor_multo` outer dispatch to N ∈ {4, 5, 6}.**
- **`crates/xcfun-rs/src/functional.rs`** — Phase 5 facade with documented `Box::leak` in `sync_weights_from_settings` (line ~196) — Plan 06-06 D-17 refactor target. RS-08 `eval_vec` stub awaits Plan 06-05.
- **`crates/xcfun-capi/`** — Phase 5 C ABI; cdylib + staticlib + rlib triple. `xcfun_eval_vec` C signature exists but stub-only; Plan 06-05 wires through `xcfun-rs::Functional::eval_vec` → `xcfun-gpu::Batch<R>`.
- **xtask infrastructure** — Phase 2 D-21 `regen-registry` + `--check` pattern. Plan 06-00 Task 4 adds `xtask regen-mpmath-fixtures` mirroring the same pattern; Plan 06-N2 reuses for the 20 `excluded_by_upstream_spec` set.
- **`validation/`** — tier-2 parity harness; cc-linked `xcfun-master`; per-functional override threshold dispatch (Phase 2 D-24 1e-7 LDAERF). Plan 06-N1/N2/N3 extend with `--reference {cpp, mpmath}` CLI flag selecting ground truth per record.

### Established Patterns

- **`xtask regen-* --check` drift gate** (Phase 2 D-21) — applied to mpmath fixtures (Plan 06-00 D-04).
- **Per-functional override threshold dispatch** (Phase 2 D-24) — extended in Plan 06-N1/N2 for the 30+ D-19 forwards if any prove genuinely intractable; default is "tighten to strict 1e-13".
- **`#[non_exhaustive]` + Copy on `XcError`** (Phase 2 D-25) — preserved by D-13-A `&'static str` payload.
- **`#[comptime]` on `(vars, mode, order)`; runtime dispatch on FunctionalId** (Phase 1 D-6) — preserved across all GPU runtimes.
- **`ctaylor_max(a, b)` semantics: `operator>` on CNST slot only** (Phase 4 Plan 04-10 Path-B finding) — D-10 tau guard uses this form.
- **Workspace exclude → members migration** (Phase 5 D-01 / D-02) — Plan 06-01 follows the same pattern: `exclude = ["crates/xcfun-gpu", "crates/xcfun-python"]` → drops `xcfun-gpu`, adds `xcfun-kernels` to members.
- **`OnceLock<CpuClient>` substrate** (Phase 1 + Phase 2 `for_tests`) — generalised to `OnceLock<R::Client>` per runtime in Plan 06-02.

### Integration Points

- **`xcfun-rs` depends on:** `xcfun-eval` (per-point eval), `xcfun-gpu` (batch dispatch via RS-08), `xcfun-core` (registry tables, types). NO direct cubecl deps.
- **`xcfun-capi` depends on:** `xcfun-rs` only (Phase 5 boundary preserved).
- **`xcfun-eval` depends on:** `xcfun-kernels` (kernel bodies), `xcfun-ad` (CTaylor primitives), `xcfun-core` (types, registry), `cubecl-cpu` (validation substrate).
- **`xcfun-gpu` depends on:** `xcfun-kernels`, `xcfun-ad`, `xcfun-core`, `cubecl` core. Feature-gated runtime deps: `hip` → `cubecl-hip`; `cuda` → `cubecl-cuda`; `metal` → `cubecl-metal`; `wgpu` → `cubecl-wgpu`. `cubecl-cpu` re-exported from `xcfun-eval` for the always-available CPU path.
- **`xcfun-kernels` depends on:** `xcfun-ad`, `xcfun-core`, `cubecl` core. NO runtime deps.
- **Plan 06-01 `git mv` impact:** every `crates/xcfun-eval/tests/*.rs` re-references `xcfun_eval::functionals::*` paths; updates to `xcfun_kernels::functionals::*`. Tier-1 self-tests move next to bodies (`xcfun-kernels/tests/`). `validation/` import path updates.
- **Phase 6 → Phase 7** (Python): `xcfun-py` consumes `xcfun-rs::Functional::eval_vec` through pyo3 + numpy. RS-08 + D-16 pitched signature aligns with NumPy 2-D `f64` `ndarray[order='C']` strides for PY-03 zero-copy.

### Pitfalls (from prior phases)

- **`anyhow` in any library crate** is CI-blocked (`xtask check-no-anyhow`). `xcfun-kernels` and `xcfun-gpu` join the enforced set when added to workspace.
- **`-Cfast-math` / `RUSTFLAGS` reassociation** breaks 1e-13 parity. `xcfun-gpu` Cargo.toml inherits `[profile.release] rustflags = []` from workspace; cubecl-cuda / cubecl-hip / cubecl-metal must NOT introduce `--use_fast_math` PTX flags (verify via cubecl source inspection in Plan 06-04 research).
- **`mul_add` ban** (Phase 2 D-13 / `xtask check-no-mul-add`) extends to `xcfun-kernels/src/functionals/**/*.rs` post Plan 06-01 `git mv`; xtask gate scope updates.
- **cubecl-runtime version drift** (Pitfall P8): all four cubecl runtime crates (cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-metal, cubecl-wgpu) MUST lock-step at `=0.10.0-pre.3`. `xtask check-cubecl-pin` extends to the new four; failure to lockstep is a hard build break (cubecl pre-release internal types cross-reference).
- **Wgpu silently running in f32** (Pitfall P7): D-13 `WgpuNoF64` typed error + compile-time `size_of::<Scalar>() == 8` assertion in `xcfun-kernels/src/lib.rs`.
- **Pitfall PHASE2-D pattern**: `XC_A_B_GAA_GAB_GBB` builder arm via explicit chain (no match-fallthrough). Extends to any new Vars arm Plan 06-00 substrate touches.
- **Phase 5 D-18 LDA-vars=6 / DensVars-driven dispatch**: Phase 6 D-18 closes this; the b3lyp / camb3lyp / bp86 alias dispatch must work in-process post Plan 06-06.

</code_context>

<specifics>
## Specific Ideas

- "Drop-in replacement" extends to `eval_vec`: a C caller's existing `xcfun_eval_vec(fun, density, density_pitch, result, result_pitch, nr_points)` continues to compile + link unchanged against `libxcfun_capi.so` — RS-08 + D-16 + CAPI-01..02 contract.
- ROCm RDNA-2 GPUs need `HSA_OVERRIDE_GFX_VERSION=10.3.0` documented in `xcfun-gpu/README.md` AND in CI scripts that run on RDNA-2 hardware. Tier-3 GREEN on RDNA-2 is acceptable evidence for "Backend::Rocm passes strict 1e-13" only after the override is set.
- mpmath ground truth at 200-digit precision was already validated against Rust at LDAERFX (Phase 2 finding: Rust = mpmath truth, C++ diverges by 6.7%). The mpmath dependency is "verified pattern", not "experimental new dependency".
- Apple Silicon Metal lacks hardware f64 — cubecl-metal on Apple Silicon will runtime-probe and refuse. Apple users with Apple Silicon get Wgpu fallback (also probably refuses without SHADER_F64) → CPU. This is acceptable for v1; document explicitly that Apple Silicon = CPU-only.
- TPSS `tau ≥ tau_w` guard is technically a divergence from C++ (which has no guard). Per D-03 ACC-04 amendment, mpmath-truth at the boundary is the verification reference; C++ is documented to suffer cancellation in this regime, so it is no longer the truth. This is the primary design tension to call out at Phase 6 sign-off.
- Plan 06-N1 root-cause bisection of inherited Phase-3 forwards may reveal common patterns (e.g., shared helper port-order issues across multiple GGA-correlation kernels). Expect that fixing one root cause tightens 3-5 functionals at once.
- The 20 `excluded_by_upstream_spec` functionals are NOT optional v1 deliverables — they are part of the 78-functional contract per ROADMAP. Plan 06-N2 mpmath-only validation is what closes the loop.
- Phase 6 sign-off requires `cargo run -p validation --release -- --backend rocm --order 3 --filter '.*'` GREEN (strict 1e-13 across 78 functionals on the primary backend). Cloud CI (GitHub Actions or equivalent) for cubecl-cuda + cubecl-metal: best-effort, non-blocking.

</specifics>

<deferred>
## Deferred Ideas

- **Stream-overlapped async GPU dispatch** — out of scope per PROJECT.md (`< 20% throughput gain for 3× API complexity`); preserved for v2.
- **`Line<F>` lane vectorisation** — `docs/design/06 §8` revisits in v2; per-thread one-grid-point keeps register pressure manageable on consumer GPUs.
- **Shared-memory reductions** — none in xcfun's API; v2 if a sum-of-energies-over-grid API is added.
- **Stable `cubecl 0.10` release** — when it ships, run a full tier-2 + tier-3 re-validation per CLAUDE.md Pitfall P8 mitigation. Out of scope until cubecl ships stable.
- **Patches to `xcfun-master/` C++** — vendored content-hash invariant preserved; mpmath amendment (D-03) substitutes for C++ where C++ is documented to suffer cancellation, instead of forking the vendored source.
- **CUDA / Metal local validation** — no hardware in dev environment; community-maintained best-effort via cloud CI or borrowed hardware. v2 may add a dedicated NVIDIA-CI runner if usage warrants.
- **`Backend::OpenCL` / `Backend::Vulkan` direct** — not in cubecl's runtime list; covered indirectly by Wgpu (Vulkan backend).
- **PyO3 + NumPy interop (PY-01..06)** — Phase 7 deliverable; Phase 6 only ensures the `eval_vec` signature shape (D-16) is compatible.

### Reviewed Todos (not folded)

None — no pending todos surfaced in cross-reference scan.

</deferred>

---

*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Context gathered: 2026-04-30*
