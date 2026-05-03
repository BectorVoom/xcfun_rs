# Phase 6: GPU Backends + Batch Lifecycle - Pattern Map

**Mapped:** 2026-04-30
**Files analyzed:** ~85 new + ~25 modified across 10 plans (06-00..06-N3)
**Analogs found:** 80 / ~85 (5 files have no direct analog; planner falls back to RESEARCH.md patterns + design-doc-06)

## File Classification

### Plan 06-00 — Algebraic substrate (AD N≥4 + libm erf Taylor + tau_w guard + mpmath sidecar)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-ad/src/ctaylor_rec/multo.rs` (extend N=4,5,6) | kernel-primitive | transform | same file, N=2/3 arms | exact |
| `crates/xcfun-ad/src/ctaylor_rec/compose.rs` (extend N=4,5,6) | kernel-primitive | transform | same file, N=2/3 arms | exact |
| `crates/xcfun-ad/tests/golden_multo_n4.rs`, `_n5.rs`, `_n6.rs` | test (golden) | request-response | `crates/xcfun-ad/tests/golden_mul.rs` + `golden_composed.rs` | exact |
| `crates/xcfun-ad/tests/golden_compose_n4.rs`, `_n5.rs`, `_n6.rs` | test (golden) | request-response | `crates/xcfun-ad/tests/golden_composed.rs` | exact |
| `crates/xcfun-ad/src/expand/erf.rs` (add `erf_precise_taylor<F,N>`) | kernel-primitive | transform | same file (`erf_precise` at line 174); `expand/expm1.rs` (stable-bracket pattern) | exact |
| `crates/xcfun-ad/tests/erf_taylor_chain.rs` | test (golden) | request-response | `crates/xcfun-ad/tests/golden_composed.rs` | exact |
| `crates/xcfun-eval/src/functionals/mgga/{tpssc,tpsslocc,revtpssc}.rs` (insert `tau_clamped`) | kernel body | transform | `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818` (`ctaylor_max`) | exact |
| `crates/xcfun-eval/tests/tpss_tau_clamp.rs` | test (integration) | request-response | `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` | role-match |
| `xtask/src/bin/regen_mpmath_fixtures.rs` | xtask binary | event-driven (CLI) | `xtask/src/bin/regen_registry.rs` (extractor + check pattern) + `xtask/src/bin/regen_ad_fixtures.rs` (cc-compile pattern) | exact |
| `xtask/mpmath_eval/__main__.py`, `__init__.py`, `evaluator.py`, `functionals.py` | sidecar (Python) | request-response | none in repo (no Python sidecar yet) | NEW pattern |
| `validation/fixtures/mpmath/<functional>.jsonl` | data fixture | streaming-read | `validation/report.jsonl` (JSONL append-mode pattern) | role-match |

### Plan 06-01 — `xcfun-kernels` git mv

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-kernels/Cargo.toml` (NEW) | crate manifest | config | `crates/xcfun-eval/Cargo.toml` (current functionals owner) | exact |
| `crates/xcfun-kernels/src/lib.rs` (NEW) | crate root | re-exports | `crates/xcfun-eval/src/lib.rs` (post-mv shrunk form) | exact |
| `crates/xcfun-kernels/src/functionals/**/*` (git-mv 78 files) | kernel bodies | transform | (verbatim move; no code change) | exact (move) |
| `crates/xcfun-kernels/src/density_vars/`, `density_vars.rs` (git-mv) | kernel-primitive | transform | (verbatim move) | exact (move) |
| `crates/xcfun-kernels/src/dispatch.rs` (git-mv) | dispatch table | request-response | (verbatim move) | exact (move) |
| Workspace `Cargo.toml` (members) | workspace config | config | Phase 5 D-01/D-02 `exclude → members` migration history (current `members = [...]`) | exact |
| `crates/xcfun-eval/Cargo.toml` (depend on `xcfun-kernels`) | crate manifest | config | current `xcfun-eval` `[dependencies]` block | exact |
| `crates/xcfun-eval/src/lib.rs` (re-exports) | crate root | re-exports | current `xcfun-eval/src/lib.rs` | exact |
| `crates/xcfun-eval/src/functional.rs` (rewire imports) | controller | request-response | self (current state) | exact |
| `crates/xcfun-eval/src/dispatch.rs` (DELETE — moved) | (delete) | — | — | — |
| Tier-1 self-tests moved to `xcfun-kernels/tests/` | test | request-response | `crates/xcfun-eval/tests/self_tests.rs` | exact |

### Plan 06-02 — `xcfun-gpu` unstub (`Backend` + `Batch` + buffer pool + `auto_backend` skeleton)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-gpu/Cargo.toml` (promote to members; feature flags) | crate manifest | config | `crates/xcfun-eval/Cargo.toml` (cubecl + cubecl-cpu feature pattern) | role-match |
| `crates/xcfun-gpu/src/lib.rs` (NEW) | crate root | re-exports | `crates/xcfun-eval/src/lib.rs` | exact |
| `crates/xcfun-gpu/src/backend.rs` (Backend enum) | type definition | — | `crates/xcfun-core/src/enums.rs` (Mode/Vars enum precedent) | role-match |
| `crates/xcfun-gpu/src/auto_backend.rs` | service | request-response | (no direct analog; RESEARCH §Pattern 4 monomorphisation) | NEW pattern |
| `crates/xcfun-gpu/src/batch.rs` (`Batch<'fun, R>`) | service | streaming/batch | `crates/xcfun-eval/src/functional.rs::eval` per-point launch loop | role-match |
| `crates/xcfun-gpu/src/pool.rs` (generation-counter buffer pool) | service | streaming | `crates/xcfun-eval/src/for_tests.rs::cpu_client` (`OnceLock` substrate) — extends to per-runtime pool | role-match |
| `crates/xcfun-gpu/src/error_routing.rs` (ERF auto-fallback) | service | request-response | `crates/xcfun-core/src/traits.rs` `Dependency::ERF` lookup | partial |
| `crates/xcfun-gpu/tests/batch_api_shape.rs` | test (compile-time) | — | `crates/xcfun-rs/tests/send_sync.rs` (`assert_impl_all!`) | role-match |
| `crates/xcfun-gpu/tests/batch_kernel_smoke.rs` | test (integration) | request-response | `crates/xcfun-eval/tests/cubecl_spike.rs` | exact |
| `crates/xcfun-gpu/tests/auto_backend_priority.rs` | test (unit) | request-response | `crates/xcfun-rs/tests/free_fns.rs` | role-match |
| `crates/xcfun-gpu/tests/buffer_pool_growth.rs` | test (unit) | request-response | `crates/xcfun-eval/tests/cubecl_densvars_spike.rs` | role-match |
| `crates/xcfun-gpu/tests/wgpu_no_f64.rs` | test (compile + unit) | request-response | (no direct analog) | NEW pattern |
| `crates/xcfun-core/src/error.rs` (add `WgpuNoF64`) | error variant | — | same file (existing variants like `InvalidVarsAndMode`) | exact |
| `crates/xcfun-core/tests/xcerror_copy_invariant.rs` | test (compile-time) | — | `crates/xcfun-rs/tests/send_sync.rs` (`assert_impl_all!`) | role-match |

### Plan 06-03 — cubecl-hip primary wiring

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-gpu/Cargo.toml` (add `cubecl-hip` feature) | crate manifest | config | `crates/xcfun-eval/Cargo.toml` (cubecl-cpu feature gate) | exact |
| `crates/xcfun-gpu/src/auto_backend.rs` (HipRuntime probe) | service | request-response | (no direct analog; cubecl-cpu is always-available, so no probe needed today) | NEW pattern |
| `crates/xcfun-gpu/README.md` (RDNA-2 HSA_OVERRIDE_GFX_VERSION) | doc | — | `validation/README.md`-style env-var doc note | role-match |
| `validation/src/main.rs` (`--backend rocm` flag) | CLI | event-driven | same file (existing `--backend cpu` arm at lines ~50-60) | exact |

### Plan 06-04 — cubecl-cuda + cubecl-wgpu opt-in

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-gpu/Cargo.toml` (add `cubecl-cuda`, `cubecl-wgpu`) | crate manifest | config | Plan 06-03 cubecl-hip arm (just-added analog) | exact |
| `crates/xcfun-gpu/src/auto_backend.rs` (CudaRuntime + WgpuRuntime probe) | service | request-response | Plan 06-03 HipRuntime probe (just-added analog) | exact |
| `crates/xcfun-gpu/tests/erf_fallback.rs` | test (integration) | request-response | `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` (Dependency-aware test) | role-match |

### Plan 06-05 — RS-08 `eval_vec` GPU dispatch

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-rs/src/functional.rs` (wire `eval_vec`) | facade | request-response | same file `Functional::eval` (line 172) | exact |
| `crates/xcfun-rs/tests/eval_vec_threshold.rs` | test (integration) | request-response | `crates/xcfun-rs/tests/free_fns.rs` | role-match |
| `crates/xcfun-capi/src/lib.rs` (rewire `xcfun_eval_vec`) | C ABI | request-response | same file lines 427-462 (existing per-point loop stub) | exact |

### Plan 06-06 — Strict zero-alloc + Vec weights + LDA-vars=6 dispatch

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-rs/src/functional.rs` (`Box::leak` → `Vec`; `UnsafeCell` handle; LDA-vars=6 dispatch) | facade | request-response | same file lines 198-216 (`sync_weights_from_settings`) | exact |
| `crates/xcfun-eval/src/functional.rs` (`weights: Vec<...>`; settings_gen counter) | controller | request-response | same file lines 85-102 (`Functional` struct) | exact |
| `crates/xcfun-rs/tests/zero_alloc_strict.rs` | test (allocator-counting) | request-response | `crates/xcfun-rs/tests/zero_alloc.rs` (existing fall-back-(b) form) | exact |
| `crates/xcfun-rs/tests/no_leak_on_set.rs` | test (allocator-counting) | request-response | `crates/xcfun-rs/tests/zero_alloc.rs` | role-match |
| `crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs` | test (integration) | request-response | `crates/xcfun-eval/tests/alias_canary.rs` | role-match |

### Plan 06-N1 — Inherited Phase-3 forwards root-cause bisection

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/xcfun-kernels/src/functionals/{gga,mgga}/*.rs` (per-functional fixes) | kernel body | transform | per-fix; varies by functional | exact (file-local) |
| Path-B side-by-side reads of `xcfun-master/src/functionals/*.cpp` vs Rust ports | (no file change) | research workflow | Phase 4 Plan 04-10 Path-B methodology | exact |

### Plan 06-N2 — mpmath-only fixtures for 20 `excluded_by_upstream_spec`

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `validation/fixtures/mpmath/{br,brc,brx,brxc,csc,blocx,scan*,tw,vwk,pbelocc,zvpbesolc,zvpbeintc}.jsonl` | data fixture | streaming-read | Plan 06-00 `validation/fixtures/mpmath/<f>.jsonl` (analog-of-self) | exact |
| `validation/src/main.rs` (`--reference {cpp, mpmath}`, `--exclude-erf`) | CLI | event-driven | same file existing `--mode` flag arm | exact |
| `validation/src/driver.rs` (mpmath fixture-loader path) | service | streaming-read | same file existing C++ FFI path | role-match |

### Plan 06-N3 — Post-libm-hybrid sweep

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| (no new files — re-runs tier-2 + tier-3 to verify libm-hybrid `erf` tightens M05/M06/B97/LYPC/PW92C/PBEC/OPTX residuals) | — | — | Plan 04-10 Path-B finding | exact |

### Cross-cutting xtask gate updates

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `xtask/src/bin/check_cubecl_pin.rs` (extend `PINNED_CRATES` to 5) | xtask gate | request-response | same file lines 1-25 (current `PINNED_CRATES`) | exact |
| `xtask/src/bin/check_no_mul_add.rs` (scope add `xcfun-kernels/`) | xtask gate | request-response | same file (current scope: `xcfun-eval/src/functionals/`) | exact |
| `xtask/src/bin/check_no_anyhow.rs` (allowlist `xcfun-kernels`, `xcfun-gpu`) | xtask gate | request-response | same file (current crates-walk) | exact |

### Cross-cutting doc updates (per CONTEXT.md `<canonical_refs>`)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `docs/design/05-module-responsibilities.md` (xcfun-kernels + xcfun-gpu sections) | doc | — | self (existing structure) | exact |
| `docs/design/06-cubecl-strategy.md` (Cuda → Rocm primary; CUDA + Metal opt-in) | doc | — | self (existing tables) | exact |
| `docs/design/07-accuracy-strategy.md` (ACC-04 amendment §1+§5) | doc | — | self | exact |
| `docs/design/08-error-model.md` (`WgpuNoF64`) | doc | — | self | exact |
| `docs/design/09-testing-strategy.md` (tier-3 ROCm strict 1e-13; mpmath fixtures) | doc | — | self | exact |
| `docs/design/10-build-and-dependencies.md` (cubecl-hip + cubecl-cuda + cubecl-wgpu pins) | doc | — | self | exact |
| `.planning/REQUIREMENTS.md` (GPU-02/04/07; ACC-04) | doc | — | self | exact |
| `.planning/PROJECT.md` (GPU constraints; Key Decisions) | doc | — | self | exact |
| `CLAUDE.md` (root: ROCm-primary build pattern, RDNA-2/Apple Silicon risk rows) | doc | — | self | exact |

---

## Pattern Assignments

### Plan 06-00 — Algebraic Substrate

#### `crates/xcfun-ad/src/ctaylor_rec/multo.rs` (extend N=4,5,6)

- **Role:** cubecl `#[cube]` primitive (CTaylor recursion specialisation).
- **Data flow:** `(dst: &mut Array<F>, y: &Array<F>) -> dst *= y` in-place; called by `xcfun_ad::ctaylor_rec::multo_skipconst` and downstream `compose` recursion.
- **Closest analog:** same file, N=2/3 arms at `crates/xcfun-ad/src/ctaylor_rec/multo.rs:84-112` (N=2) and `:131-160+` (N=3).
- **Excerpt from analog (N=2 base case, lines 84-112):**
  ```rust
  /// N=2 multo. Port of `ctaylor.hpp:131-135` — the load-bearing base case.
  #[cube]
  pub(crate) fn ctaylor_multo_n2<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
      let d0 = dst[0]; let d1 = dst[1]; let d2 = dst[2]; let d3 = dst[3];
      // dst[3] = d0*y[3] + d3*y[0] + d1*y[2] + d2*y[1]   (C++ left-assoc)
      let t30 = d0 * y[3]; let t31 = d3 * y[0];
      let t32 = d1 * y[2]; let t33 = d2 * y[1];
      let s1 = t30 + t31; let s2 = s1 + t32;
      dst[3] = s2 + t33;
      // dst[2] = d0*y[2] + d2*y[0]
      let t20 = d0 * y[2]; let t21 = d2 * y[0];
      dst[2] = t20 + t21;
      // ... dst[1], dst[0] in descending order ...
  }
  ```
- **Adaptation notes:** N=4 follows the general recursion `multo(dst, y) = { multo_n3(dst+8, y); mul_n3(dst+8, dst, y+8); multo_n3(dst, y); }` from `ctaylor.hpp:55-65`. **Capture all 16 dst values into `let d0..d15` BEFORE any writes** — cubecl 0.10-pre.3 cannot sub-slice `Array<F>` (verified by N=3 pattern). Write in C++-descending order: `dst[15], dst[14], ..., dst[0]`. RESEARCH.md Pattern 1 strongly recommends macro-generation for N=5/6 (~500/1000 LOC). Pitfall P11 — do NOT reverse the descending write-order.

#### `crates/xcfun-ad/src/ctaylor_rec/compose.rs` (extend N=4,5,6)

- **Role:** cubecl `#[cube]` primitive (series composition).
- **Data flow:** `(out: &mut Array<F>, x: &Array<F>, f: &Array<F>) -> out = f(x)` where `f` is a length-(N+1) scalar series.
- **Closest analog:** same file, N=2 arm at `crates/xcfun-ad/src/ctaylor_rec/compose.rs:99-115`.
- **Excerpt from analog (N=2 base case, lines 99-115):**
  ```rust
  /// N=2 compose. Port of `ctaylor.hpp:146-151`.
  #[cube]
  pub(crate) fn ctaylor_compose_n2<F: Float>(
      out: &mut Array<F>, x: &Array<F>, f: &Array<F>,
  ) {
      out[0] = f[0];
      out[1] = f[1] * x[1];
      out[2] = f[1] * x[2];
      let two = F::new(2.0);
      let t1 = f[1] * x[3];
      let s1 = two * x[1]; let s2 = s1 * x[2]; let s3 = s2 * f[2];
      out[3] = t1 + s3;
  }
  ```
- **Adaptation notes:** N≥3 uses the descending-i Horner form (`ctaylor.hpp:72-82`); imports the corresponding `ctaylor_multo_skipconst_n{3,4,5}` from `multo.rs`. RESEARCH Pitfall P11: outer loop MUST be `for (i = Nvar-1; i >= 0; i--)`; reversing breaks 1e-12 parity for n ≥ 2.

#### `crates/xcfun-ad/tests/golden_{multo,compose}_n{4,5,6}.rs`

- **Role:** golden-fixture parity test (cubecl-cpu vs C++ reference fixtures).
- **Data flow:** loads bincode fixtures; runs each through the kernel adapter; compares per-coefficient at 1e-12.
- **Closest analog:** `crates/xcfun-ad/tests/golden_composed.rs` (same shape; uses `tests/fixtures/composed.bincode` generated by `xtask regen-ad-fixtures`).
- **Excerpt from analog (lines 1-50):**
  ```rust
  #![cfg(feature = "testing")]
  use cubecl::prelude::*;
  use cubecl_cpu::CpuRuntime;
  use serde::{Deserialize, Serialize};
  use xcfun_ad::for_tests::cpu_client;
  use xcfun_ad::math;

  #[derive(Serialize, Deserialize, Debug, Clone)]
  struct FixtureRecord { op: String, n_var: u8, inputs: Vec<f64>, coeffs: Vec<f64> }

  #[cube(launch_unchecked)]
  fn kernel_compose<F: Float>(x: &Array<F>, f: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
      ctaylor_rec::compose::ctaylor_compose::<F>(out, x, f, n);
  }
  ```
- **Adaptation notes:** Each golden test crate gets its own `kernel_*` adapter `#[cube(launch_unchecked)]`; fixture file extends `xtask regen-ad-fixtures` to emit N=4/5/6 records. Fixture-extension lives in `xtask/src/bin/regen_ad_fixtures.rs` (not new file).

#### `crates/xcfun-ad/src/expand/erf.rs` (add `erf_precise_taylor<F,N>`)

- **Role:** cubecl `#[cube]` AD-chain wrapper (lifts scalar `erf_precise` to a CTaylor chain).
- **Data flow:** `(out: &mut Array<F>, x: &Array<F>, #[comptime] n: u32)` where `x[0]` is the constant slot — composes scalar `erf_precise` with the `compose` recursion; replaces the polyfill-derived chain in `ctaylor_erf` (`math.rs::ctaylor_erf`).
- **Closest analog:** same file `erf_precise` at `crates/xcfun-ad/src/expand/erf.rs:174-280` (FreeBSD msun port; ≤ 1 ULP vs `libm::erf`); AND `crates/xcfun-ad/src/expand/expm1.rs:51-90` (stable-bracket pattern from Phase 2 Plan 02-06 — establishes the "use `erf_precise` for `t[0]`, use derivative-Taylor for `t[i≥1]`" hybrid).
- **Excerpt from `expm1.rs:51-65` (the stable-bracket pattern Phase 6 mirrors for `erf`):**
  ```rust
  /// Port of `ctaylor_math.hpp:85-102`. The stable-bracket correction applies
  /// only to t[0] (the constant term); higher-order coefficients t[i>=1]
  /// equal exp(x0) / i! unchanged (derivatives of expm1 match derivatives of exp).
  #[cube]
  pub fn expm1_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
      let mut ifac = F::new(1.0);
      t[0] = x0.exp();
      #[unroll]
      for i in 1_u32..=n {
          let k = i as usize;
          let i_f = F::cast_from(i);
          ifac = ifac * i_f;
          t[k] = t[0] / ifac;
      }
      // ... stable-bracket for t[0] when |x0| <= 1e-3 ...
  }
  ```
- **Excerpt from analog `erf_precise` (lines 174-203):** the FreeBSD-msun branch-A polynomial (~1 ULP scalar precision; already shipped — Plan 06-00 Task 2 just lifts it into the AD chain).
- **Adaptation notes:** New `erf_precise_taylor<F,N>` calls scalar `erf_precise(x[0])` to seed `t[0]`, then uses `gauss_expand` + `tfuns_integrate` for `t[i≥1]` (already done in `erf_expand:343-344`); the change is to replace `cubecl::Float::erf` (polyfill) with `erf_precise` in the seed step. Then `ctaylor_erf` in `math.rs` calls `erf_precise_taylor` for the chain.

#### `crates/xcfun-eval/src/functionals/mgga/{tpssc,tpsslocc,revtpssc}.rs` (insert `tau_clamped`)

- **Role:** kernel body modification (in-place guard insert).
- **Data flow:** `(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) -> writes out[*]`.
- **Closest analog:** `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818-838` (existing `ctaylor_max`).
- **Excerpt from analog (lines 818-838):**
  ```rust
  /// TPSS `max(a, b)` for CTaylor — takes max of CNST slot.
  #[cube]
  pub fn ctaylor_max<F: Float>(
      a: &Array<F>, b: &Array<F>, out: &mut Array<F>, #[comptime] n: u32,
  ) {
      let size = comptime!((1_u32 << n) as usize);
      if a[0] >= b[0] {
          #[unroll]
          for i in 0..size { out[i] = a[i]; }
      } else {
          #[unroll]
          for i in 0..size { out[i] = b[i]; }
      }
  }
  ```
- **Adaptation notes:** Existing kernel body in `tpssc.rs` is:
  ```rust
  pub fn tpssc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
      let size = comptime!((1_u32 << n) as usize);
      let mut eps = Array::<F>::new(size);
      tpss_like::tpss_eps_full::<F>(d, &mut eps, n);  // <-- consumes d.tau here
      ctaylor_mul::<F>(&d.n, &eps, out, n);
  }
  ```
  Phase 6 D-10 inserts `let mut tau_w = build_tau_w(d, n); let mut tau_clamped = ctaylor_max(d.tau, tau_w, n);` before the `tpss_eps_full` call, then passes `tau_clamped` as the tau input. Since `tpss_eps_full` reads `d.tau` directly (not via parameter), the cleanest patch is to add a `tpss_eps_full_clamped` variant that takes `tau_clamped` explicitly. Same shape repeats for `tpsslocc.rs` (`tpss_like::tpss_locc_eps_full`) and `revtpssc.rs` (`tpss_like::revtpss_eps_full`). RESEARCH Pattern 5 has the full snippet.

#### `xtask/src/bin/regen_mpmath_fixtures.rs`

- **Role:** xtask binary (offline mpmath truth generator).
- **Data flow:** CLI → spawn `python3 -m xtask.mpmath_eval` subprocess → JSONL stdout → write to `validation/fixtures/mpmath/<f>.jsonl` + `.sha256` stamp.
- **Closest analog:** `xtask/src/bin/regen_registry.rs` (extractor + `--check` drift gate, lines 1-50) and `xtask/src/bin/regen_ad_fixtures.rs` (cc-compile-then-run subprocess pattern).
- **Excerpt from analog `regen_registry.rs:1-40`:**
  ```rust
  //! Workflow:
  //!   1. Compile `xtask/assets/regen_registry/extractor.cpp` to an executable
  //!      under `target/regen_registry/` using the system C++ compiler.
  //!   2. Run the extractor; capture JSONL on stdout.
  //!   3. Parse each JSONL line.
  //!   4. Emit Rust source files + `.sha256` stamp.
  //!   5. `--check` mode regenerates in memory, hashes, and compares
  //!      against the committed `.sha256` stamps. Exits 2 on drift.
  use anyhow::{Context, Result, bail};
  use serde_json::Value;
  use sha2::{Digest, Sha256};
  use std::process::Command;
  ```
- **Adaptation notes:** Replace cc-compile step with `Command::new("python3").arg("-m").arg("xtask.mpmath_eval")`. Reuse `--check` drift gate exactly. mpmath dep stays in Python land — Cargo build path NEVER calls Python (D-04 contract). The `xtask/mpmath_eval/` Python module is run from the repo's `xtask/` directory; pass `--cwd` accordingly so `python3 -m` resolves the package.

#### `xtask/mpmath_eval/__main__.py`, `evaluator.py`, `functionals.py`

- **Role:** Python sidecar (mpmath at prec=200).
- **Data flow:** stdin/argv `(functional_name, vars_set, mode, order, density_tuple)` → stdout JSONL records.
- **Closest analog:** none in repo. RESEARCH §"mpmath Sidecar Architecture" (lines 812-907) is the spec.
- **Excerpt (from RESEARCH.md lines 814-895):**
  ```python
  # __main__.py
  import json, sys
  from .evaluator import eval_record
  for line in sys.stdin:
      req = json.loads(line)
      out = eval_record(req["functional"], req["vars"], req["mode"],
                        req["order"], req["density"])
      print(json.dumps(out))
  ```
- **Adaptation notes:** Pure Python; uses `mpmath` only (no other deps). One file per functional family in `functionals.py` (LDAERFX/C/JT, TPSSC/LOCC/REVTPSSC + the 20 `excluded_by_upstream_spec` set). Reproducibility: `mp.prec = 200` at module top; deterministic for fixed input.

#### `validation/fixtures/mpmath/<functional>.jsonl`

- **Role:** committed JSONL data fixtures (one record per `(functional, vars, mode, order, density_tuple)` truth).
- **Data flow:** read by `validation/src/driver.rs` when `--reference mpmath`; mirrored against `xcfun_rs::Functional::eval` output at strict 1e-13.
- **Closest analog:** `validation/report.jsonl` (line-per-record JSONL append-mode pattern; Phase 4 Plan 04-10 `--resume` finding).
- **Excerpt of layout (RESEARCH.md lines 896-907):**
  ```json
  {"functional":"ldaerfx","vars":"a_b","mode":"partial_derivatives","order":3,
   "density":[1.0,1.0],"truth":[-0.7385588...,...,...],"prec":200}
  ```
- **Adaptation notes:** `xtask regen-mpmath-fixtures --check` enforces drift gate. ~3300 records total (13 ACC-04-amended × ~100 + 20 `excluded_by_upstream_spec` × ~100).

---

### Plan 06-01 — `xcfun-kernels` git-mv

#### `crates/xcfun-kernels/Cargo.toml` (NEW)

- **Role:** crate manifest.
- **Data flow:** workspace member declaration; depends on `xcfun-ad` + `xcfun-core` + `cubecl` core only (NO cubecl-cpu / cubecl-hip / cubecl-cuda / cubecl-wgpu).
- **Closest analog:** `crates/xcfun-eval/Cargo.toml` (current functionals owner; will SHRINK after the mv).
- **Excerpt from analog:**
  ```toml
  [package]
  name = "xcfun-eval"
  version.workspace = true
  edition.workspace = true
  rust-version.workspace = true
  description = "cubecl launcher + functional bodies for xcfun_rs"
  license = "MPL-2.0"
  [features]
  default = ["cpu"]
  cpu = ["dep:cubecl-cpu"]
  testing = []
  [dependencies]
  xcfun-core = { path = "../xcfun-core" }
  xcfun-ad = { path = "../xcfun-ad" }
  cubecl = { workspace = true }
  cubecl-cpu = { workspace = true, optional = true }
  thiserror = { workspace = true }
  ```
- **Adaptation notes:** New `xcfun-kernels` Cargo.toml drops `cubecl-cpu` (kernel bodies don't instantiate runtimes per D-08); keeps `cubecl = { workspace = true }`. Drops `default = ["cpu"]`. `testing` feature retained for tier-1 self-tests in `crates/xcfun-kernels/tests/`.

#### Workspace `Cargo.toml` (members)

- **Role:** workspace config.
- **Data flow:** Cargo dependency resolution.
- **Closest analog:** Phase 5 D-01/D-02 migration (`exclude` → `members`); current state has `members = [xcfun-ad, xcfun-core, xcfun-eval, xcfun-rs, xcfun-capi, xtask, validation]` and `exclude = [xcfun-gpu, xcfun-python]`.
- **Excerpt from current state:**
  ```toml
  members = [
      "crates/xcfun-ad",
      "crates/xcfun-core",
      "crates/xcfun-eval",
      "crates/xcfun-rs",
      "crates/xcfun-capi",
      "xtask",
      "validation",
  ]
  exclude = [
      "crates/xcfun-gpu",
      "crates/xcfun-python",
  ]
  ```
- **Adaptation notes:** Plan 06-01 adds `crates/xcfun-kernels` to `members`. Plan 06-02 promotes `crates/xcfun-gpu` from `exclude` to `members`. `crates/xcfun-python` stays in `exclude` until Phase 7. Add cubecl-hip / cubecl-cuda / cubecl-wgpu to `[workspace.dependencies]` at `=0.10.0-pre.3`.

#### `crates/xcfun-kernels/src/functionals/**/*` (git-mv 78 files)

- **Role:** kernel bodies (verbatim move).
- **Data flow:** unchanged from `xcfun-eval/src/functionals/`.
- **Closest analog:** the files themselves (78 files in `crates/xcfun-eval/src/functionals/{lda,gga,mgga}/`).
- **Adaptation notes:** Pure `git mv`. Internal `crate::` paths must update to the new crate root. RESEARCH Risk R-06: every `crates/xcfun-eval/tests/*.rs` re-references `xcfun_eval::functionals::*` — those updates to `xcfun_kernels::functionals::*` are mandatory. D-09 sequencing: substrate (Plan 06-00) lands in CURRENT tree FIRST so the mv has zero algebraic deltas (any post-mv tier-1 / tier-2 regression is unambiguously a "move bug").

#### `crates/xcfun-kernels/src/dispatch.rs` (git-mv)

- **Role:** FunctionalId-keyed comptime dispatch table.
- **Data flow:** `(id, d, out, n)` → comptime if-chain → calls `crate::functionals::{lda,gga,mgga}::*::*_kernel`.
- **Closest analog:** `crates/xcfun-eval/src/dispatch.rs:79+` (the function being moved).
- **Excerpt from analog (lines 78-95):**
  ```rust
  /// Comptime-dispatched per-functional kernel call. Each arm corresponds to a
  /// `FunctionalId` discriminant.
  #[cube]
  #[allow(unused_variables)]
  pub fn dispatch_kernel<F: Float>(
      #[comptime] id: u32,
      d: &DensVarsDev<F>,
      out: &mut Array<F>,
      #[comptime] n: u32,
  ) {
      if comptime!(id == 0) {
          crate::functionals::lda::slaterx::slaterx_kernel::<F>(d, out, n);
      } else if comptime!(id == 1) {
          crate::functionals::gga::pw91::pw86x::pw86x_kernel::<F>(d, out, n);
      }
      // ... 78 arms ...
  }
  ```
- **Adaptation notes:** `crate::` paths resolve unchanged because `dispatch.rs` lives in the same crate as `functionals/`. The `pub fn supports(id: FunctionalId) -> bool` helper at line 339 also moves. `xcfun-eval::Functional::eval` post-mv calls `xcfun_kernels::dispatch::dispatch_kernel` via re-export.

#### `crates/xcfun-eval/src/functional.rs` (rewire imports post-mv)

- **Role:** controller (Functional struct + per-point eval entry point); STAYS in `xcfun-eval` per D-08.
- **Data flow:** `eval(input, output)` → builds DensVarsDev → launches `eval_point_kernel` adapter.
- **Closest analog:** self (current state).
- **Excerpt from current state (lines 31-65):**
  ```rust
  use cubecl::prelude::*;
  use cubecl_cpu::CpuRuntime;
  use xcfun_core::{ALIASES, Dependency, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, ParameterId, Vars, XcError, taylorlen};

  use crate::density_vars::DensVarsDev;
  use crate::density_vars::build::build_densvars;
  use crate::density_vars::DensVarsDevLaunch;
  use crate::dispatch;
  use crate::dispatch::dispatch_kernel;

  #[cube(launch_unchecked)]
  fn eval_point_kernel<F: Float>(
      input: &Array<F>, d: &mut DensVarsDev<F>, out: &mut Array<F>,
      #[comptime] id: u32, #[comptime] vars: u32, #[comptime] n: u32,
  ) {
      build_densvars::<F>(input, d, vars, n);
      dispatch_kernel::<F>(id, d, out, n);
  }
  ```
- **Adaptation notes:** Post-mv, replace `use crate::density_vars::*` with `use xcfun_kernels::density_vars::*` and `use crate::dispatch::dispatch_kernel` with `use xcfun_kernels::dispatch::dispatch_kernel`. Re-export `xcfun_kernels::density_vars` and `xcfun_kernels::dispatch` from `xcfun-eval/src/lib.rs` so existing dependents (`xcfun-rs`, `validation`) keep their import paths (with maybe deprecation comments).

---

### Plan 06-02 — `xcfun-gpu` Unstub

#### `crates/xcfun-gpu/Cargo.toml` (promote from exclude → members; feature flags)

- **Role:** crate manifest.
- **Data flow:** declares cubecl runtime deps (all optional, gated by features `cpu`/`hip`/`cuda`/`wgpu`).
- **Closest analog:** `crates/xcfun-eval/Cargo.toml` `cubecl-cpu` feature pattern.
- **Excerpt from analog (xcfun-eval Cargo.toml):**
  ```toml
  [features]
  default = ["cpu"]
  cpu = ["dep:cubecl-cpu"]
  [dependencies]
  cubecl-cpu = { workspace = true, optional = true }
  ```
- **Adaptation notes:** Plan 06-02 establishes the `cpu` arm + skeleton; Plan 06-03 adds `hip = ["dep:cubecl-hip"]`; Plan 06-04 adds `cuda = ["dep:cubecl-cuda"]` and `wgpu = ["dep:cubecl-wgpu"]`. cubecl-cpu re-exported from `xcfun-eval` (D-08) so always available; this crate's own `cpu` feature triggers `dep:cubecl-cpu` when caller wants direct access. Add `[lib]` crate-type stays default (rlib).

#### `crates/xcfun-gpu/src/backend.rs` (Backend enum)

- **Role:** type definition (5-variant runtime discriminator).
- **Data flow:** consumed by `auto_backend()` and `Batch::eval_vec_host` enum-dispatch.
- **Closest analog:** `crates/xcfun-core/src/enums.rs` (Mode/Vars enum precedent — `Copy + Clone + Debug + PartialEq + Eq + Hash`).
- **Excerpt of recommended shape (RESEARCH.md Pattern 4):**
  ```rust
  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub enum Backend {
      Cpu,
      Rocm,    // cubecl-hip (D-05 primary)
      Cuda,    // cubecl-cuda
      Metal,   // cubecl-wgpu Metal backend
      Wgpu,    // cubecl-wgpu generic (Vulkan/DX12/WebGPU)
  }
  ```
- **Adaptation notes:** Discretion (CONTEXT.md): `Backend` may live in `xcfun-core` or `xcfun-gpu` — RESEARCH recommends `xcfun-gpu` (closer to consumer); CONTEXT discretion section confirms either is fine. Note: `Metal` removed if D-06 amendment per RESEARCH Pitfall 9 says "cubecl-metal does not exist — Metal accessed via cubecl-wgpu only"; in that case the variant is dropped or kept-as-alias-of-`Wgpu`.

#### `crates/xcfun-gpu/src/auto_backend.rs`

- **Role:** runtime probe service.
- **Data flow:** reads env `XCFUN_FORCE_BACKEND` → cascading feature-gated probes → returns `Backend`.
- **Closest analog:** none in current codebase. RESEARCH Pattern 4 is the spec.
- **Excerpt from RESEARCH Pattern 4 (lines 429-443):**
  ```rust
  pub fn auto_backend() -> Backend {
      // D-07 priority order.
      if let Ok(force) = std::env::var("XCFUN_FORCE_BACKEND") {
          return parse_force(&force).expect("XCFUN_FORCE_BACKEND unrecognised");
      }
      #[cfg(feature = "hip")]
      if rocm_available() { return Backend::Rocm; }
      #[cfg(feature = "cuda")]
      if cuda_available() { return Backend::Cuda; }
      #[cfg(feature = "wgpu")]
      if wgpu_with_shader_f64_available() { return Backend::Wgpu; }
      Backend::Cpu
  }
  ```
- **Adaptation notes:** Each `*_available()` probe returns `bool` and lives in a feature-gated submodule (`runtime/hip.rs`, `runtime/cuda.rs`, `runtime/wgpu.rs`). Wgpu probe MUST check `wgpu::Features::SHADER_F64`; if absent, return `false` so the `Cpu` arm wins (D-13 typed-error path is for `Batch::open` callers who explicitly pre-selected `Wgpu`).

#### `crates/xcfun-gpu/src/batch.rs` (`Batch<'fun, R>`)

- **Role:** service (lifecycle + launch dispatch).
- **Data flow:** `Batch::open(fun, runtime) → Batch::launch(nr_points) → ensure_capacity → write density → kernel launch → read result`.
- **Closest analog:** `crates/xcfun-eval/src/functional.rs::eval` per-point launch loop (the lift-to-batch precedent).
- **Excerpt from RESEARCH Pattern 3 (lines 371-405):**
  ```rust
  pub struct Batch<'fun, R: cubecl::Runtime> {
      fun: &'fun xcfun_rs::Functional,
      client: cubecl::ComputeClient<R::Server, R::Channel>,
      weights_buf:    R::Handle,
      active_ids_buf: R::Handle,
      density_buf:    R::Handle,           // grows powers-of-two
      result_buf:     R::Handle,           // grows powers-of-two
      capacity:       usize,
      cached_gen:     u64,
  }
  impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
      pub fn launch(&mut self, nr_points: u32) -> Result<(), XcError> {
          if self.fun.inner_settings_gen() != self.cached_gen {
              self.client.write(&self.weights_buf, bytemuck::cast_slice(&self.fun.inner_settings()));
              self.cached_gen = self.fun.inner_settings_gen();
          }
          // ... launch_unchecked dispatch ...
          Ok(())
      }
      fn ensure_capacity(&mut self, nr_points: usize) {
          if nr_points > self.capacity {
              let mut new_cap = self.capacity.max(64);
              while new_cap < nr_points { new_cap *= 2; }
              // ... realloc density_buf + result_buf ...
              self.capacity = new_cap;
          }
      }
  }
  ```
- **Adaptation notes:** D-15 powers-of-two growth + generation counter + fixed `weights_buf` (82 f64) + fixed `active_ids_buf` (78 u32). Generic over `R: cubecl::Runtime` so monomorphisation per backend. RESEARCH Pattern 4: cannot use `Box<dyn Runtime>` — cubecl's trait isn't object-safe. Use `Functional::settings_gen` counter (NEW field; see Plan 06-06).

#### `crates/xcfun-gpu/src/pool.rs` (generation-counter buffer pool)

- **Role:** service (per-runtime client cache + buffer lifecycle).
- **Data flow:** `OnceLock<R::Client>` per runtime; allocates handles via `client.empty(bytes)`; doubles capacity on overflow.
- **Closest analog:** `crates/xcfun-eval/src/for_tests.rs:1-25` (`OnceLock<CpuClient>` pattern; promoted to production in Plan 06-06).
- **Excerpt from analog (`for_tests.rs:1-25`):**
  ```rust
  use cubecl::prelude::*;
  use cubecl_cpu::{CpuDevice, CpuRuntime};
  use std::sync::OnceLock;

  pub type CpuClient = ComputeClient<CpuRuntime>;
  static CPU_CLIENT: OnceLock<CpuClient> = OnceLock::new();

  pub fn cpu_client() -> &'static CpuClient {
      CPU_CLIENT.get_or_init(|| {
          let device = CpuDevice;
          CpuRuntime::client(&device)
      })
  }
  ```
- **Adaptation notes:** Generalise to `static HIP_CLIENT: OnceLock<HipClient>`, `static CUDA_CLIENT: OnceLock<CudaClient>`, `static WGPU_CLIENT: OnceLock<WgpuClient>`, each behind `#[cfg(feature = "*")]`. `cubecl_cpu` import comes from `xcfun-eval` re-export (D-08 boundary). Powers-of-two doubling lives inside `Batch::ensure_capacity` (RESEARCH Pattern 3) — `pool.rs` owns the `OnceLock`s + a small handle-allocation helper.

#### `crates/xcfun-gpu/src/error_routing.rs` (ERF auto-fallback)

- **Role:** service (Wgpu/Metal route ERF-bearing functionals back to CPU).
- **Data flow:** `Batch::launch` checks `fun.dependencies().contains(Dependency::ERF)` → if true and `R == Wgpu/Metal` → fall through to CpuClient.
- **Closest analog:** `crates/xcfun-core/src/traits.rs` `Dependency::ERF` constant + `Functional::dependencies()` accessor (already shipped).
- **Adaptation notes:** Pure dispatch logic; no new types. Consumed by Plan 06-05 `eval_vec` GPU dispatch (GPU-05 contract). Test analog: `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` (Dependency-aware test pattern).

#### `crates/xcfun-core/src/error.rs` (add `WgpuNoF64`)

- **Role:** error variant (added to existing `XcError` enum).
- **Data flow:** returned by `Batch::open` when Wgpu-selected device lacks `SHADER_F64`.
- **Closest analog:** same file `XcError::InvalidVarsAndMode` variant (existing 3-field shape).
- **Excerpt from analog (`error.rs:35-44`):**
  ```rust
  #[error("vars {vars:?} and mode {mode:?} both invalid for dependencies {depends:?}")]
  InvalidVarsAndMode {
      vars: Vars,
      mode: Mode,
      depends: Dependency,
  },
  ```
- **Adaptation notes:** Add (preserving `Copy + non_exhaustive`):
  ```rust
  #[error("Wgpu adapter {adapter_name} lacks SHADER_F64; cannot launch {requested_runtime:?}")]
  WgpuNoF64 {
      adapter_name: &'static str,           // D-13-A: &'static str (NOT String) preserves Copy
      requested_runtime: Backend,           // Copy enum
  },
  ```
  `Backend` import path requires either (a) moving `Backend` to `xcfun-core` (matches Mode/Vars precedent — CONTEXT discretion permits) or (b) using a tiny shadow-enum here. Pick (a) per CONTEXT discretion. `Box::leak` once at runtime to convert `wgpu::AdapterInfo::name: String` → `&'static str` is justified (one-time panic-on-misconfiguration message). Update `XcError::as_c_code` to map `WgpuNoF64` → `-1` (no upstream `XC_E*` mapping).

#### `crates/xcfun-gpu/tests/batch_api_shape.rs`

- **Role:** compile-time test (asserts `Batch<'fun, R>` exposes the contract API).
- **Data flow:** `assert_impl_all!(Batch<'_, CpuRuntime>: ...)` patterns.
- **Closest analog:** `crates/xcfun-rs/tests/send_sync.rs` (one-line `assert_impl_all!`).
- **Excerpt from analog:**
  ```rust
  //! RS-10 — `Functional` MUST be `Send + Sync`. Compile-time gate.
  use static_assertions::assert_impl_all;
  use xcfun_rs::Functional;
  assert_impl_all!(Functional: Send, Sync);
  ```
- **Adaptation notes:** Asserts `Batch<'_, CpuRuntime>: Send` (per `Runtime: 'static + Send + Sync` upstream). Uses `static_assertions = "1.1"` already in xcfun-rs dev-deps. GPU-01 contract.

#### `crates/xcfun-gpu/tests/wgpu_no_f64.rs`

- **Role:** unit test (asserts `Batch::open` returns `WgpuNoF64` when feature-probe fails).
- **Data flow:** mock or feature-gated test that simulates Wgpu adapter without f64.
- **Closest analog:** none direct. Use `XCFUN_FORCE_BACKEND=wgpu` env override + a conditional `#[cfg(target_os = "...")]` skip if no Wgpu adapter is available.
- **Adaptation notes:** GPU-06 contract; D-13/13-A. Test only runs under `--features wgpu`; otherwise `#[ignore]`. Per RESEARCH Pitfall 5: f64-probe MUST refuse to launch (not silently downgrade to f32) — Pitfall P7 from CONTEXT.

---

### Plan 06-03 — cubecl-hip Primary Wiring

#### `crates/xcfun-gpu/Cargo.toml` (add `cubecl-hip`)

- **Role:** crate manifest update.
- **Data flow:** `[features] hip = ["dep:cubecl-hip"]`.
- **Closest analog:** Plan 06-02 `cpu = ["dep:cubecl-cpu"]` arm (just-added analog).
- **Adaptation notes:** Add `cubecl-hip = { workspace = true, optional = true }`. Update workspace `[workspace.dependencies]` with `cubecl-hip = "=0.10.0-pre.3"`. Workspace `[workspace.dependencies]` lockstep at exact version per CLAUDE.md Pitfall 8.

#### `crates/xcfun-gpu/src/auto_backend.rs` (HipRuntime probe)

- **Role:** service extension.
- **Data flow:** `rocm_available() -> bool` checks `HipRuntime::client(&HipDevice)` succeeds + device exists.
- **Closest analog:** none direct (this is the FIRST GPU runtime probe in the codebase). RESEARCH Pitfall 3 documents the RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0` requirement.
- **Adaptation notes:** Default GPU. D-07 priority chain: `XCFUN_FORCE_BACKEND` env > Rocm > Cuda (if feature) > Wgpu (if SHADER_F64) > Cpu. Document in `xcfun-gpu/README.md` that RDNA-2 users must `export HSA_OVERRIDE_GFX_VERSION=10.3.0` before any binary launches `Backend::Rocm`.

#### `validation/src/main.rs` (`--backend rocm` flag)

- **Role:** CLI flag addition.
- **Data flow:** `--backend rocm` → switches harness driver to use `xcfun_gpu::Batch::<HipRuntime>` instead of CpuRuntime.
- **Closest analog:** same file (existing `--backend cpu` parsing at lines 50-60).
- **Excerpt from analog:**
  ```rust
  let backend = parse_arg(&args, "--backend").unwrap_or("cpu");
  let order: u32 = parse_arg(&args, "--order").unwrap_or("2").parse().context("--order must be u32")?;
  ```
- **Adaptation notes:** Add `match backend { "cpu" => ..., "rocm" => ..., "cuda" => ..., "wgpu" => ..., _ => bail!("..."), }`. Plan 06-N2 extends with `--reference {cpp, mpmath}` and `--exclude-erf` flags using the same parsing pattern.

---

### Plan 06-04 — cubecl-cuda + cubecl-wgpu Opt-in

Same shape as Plan 06-03 (Cargo.toml feature gate + auto_backend probe + harness flag). The only file with no analog yet is `crates/xcfun-gpu/tests/erf_fallback.rs`:

#### `crates/xcfun-gpu/tests/erf_fallback.rs`

- **Role:** integration test (Wgpu-selected functional with `Dependency::ERF` auto-routes to Cpu).
- **Data flow:** force-select Wgpu via env → set up LDAERFX (ERF-dependent) → call `eval_vec` → assert that internal dispatch chose CpuRuntime + result matches CPU baseline at 1e-13.
- **Closest analog:** `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` (Dependency-aware integration pattern); `crates/xcfun-rs/tests/zero_alloc.rs` (allocator-counting infrastructure for asserting which path ran).
- **Adaptation notes:** GPU-05 contract. Use a side-channel (e.g. `tracing` event capture or a debug counter) to assert which Backend actually executed.

---

### Plan 06-05 — RS-08 `eval_vec` GPU Dispatch

#### `crates/xcfun-rs/src/functional.rs` (wire `eval_vec`)

- **Role:** facade (Phase 5 stub → wired implementation).
- **Data flow:** `eval_vec(density, density_pitch, out, out_pitch, nr_points)` → if `nr_points < threshold` fall through to per-point loop; else `xcfun_gpu::Batch::<R>::eval_vec_host` with `R = auto_backend()`.
- **Closest analog:** same file `Functional::eval` at line 172 (per-point form).
- **Excerpt from analog:**
  ```rust
  /// RS-07 — evaluate. Zero heap allocation on the success path is the contract.
  pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
      self.0.eval(input, output)
  }
  ```
- **Adaptation notes:** D-14 threshold = `const XCFUN_MIN_BATCH_SIZE: usize = 64` with env override path. D-16 signature: `pub fn eval_vec(&self, density: &[f64], density_pitch: usize, out: &mut [f64], out_pitch: usize, nr_points: usize) -> Result<(), XcError>` (matches `xcfun-master/api/xcfun.h:54`). RESEARCH Pattern 4 monomorphisation: `match auto_backend() { Backend::Cpu => Batch::<CpuRuntime>::..., Backend::Rocm => Batch::<HipRuntime>::..., ... }`.

#### `crates/xcfun-capi/src/lib.rs` (rewire `xcfun_eval_vec`)

- **Role:** C ABI wrap.
- **Data flow:** existing C signature, body now delegates to `xcfun_rs::Functional::eval_vec` instead of looping per-point.
- **Closest analog:** same file lines 427-462 (existing per-point loop stub).
- **Excerpt from current state (lines 427-462):**
  ```rust
  #[unsafe(no_mangle)]
  pub extern "C" fn xcfun_eval_vec(
      fun: *const xcfun_s,
      nr_points: c_int,
      density: *const c_double,
      density_pitch: c_int,
      result: *mut c_double,
      result_pitch: c_int,
  ) {
      c_entry!("xcfun_eval_vec", fun, density, result => {
          // ... validate nr_points / pitches non-negative ...
          let f = unsafe { &(*fun).inner };
          let inlen = f.input_length();
          let outlen = match f.output_length() { /* ... */ };
          let dp = density_pitch as usize;
          let rp = result_pitch as usize;
          for k in 0..(nr_points as usize) {
              // PER-POINT loop — Phase 6 replaces with single eval_vec call
              let in_slice = unsafe { std::slice::from_raw_parts(density.add(k * dp), inlen) };
              let out_slice = unsafe { std::slice::from_raw_parts_mut(result.add(k * rp), outlen) };
              if let Err(e) = f.eval(in_slice, out_slice) { die_with(/* ... */); }
          }
      })
  }
  ```
- **Adaptation notes:** Replace the inner `for k in 0..(nr_points as usize)` loop with a single `match f.eval_vec(/* slice from raw_parts spanning all nr_points */, dp, /* result slice */, rp, nr_points as usize) { Ok(()) => {}, Err(e) => die_with(...) }`. Slice construction must use the FULL range `nr_points * pitch` for the safety invariants of `from_raw_parts`. CAPI-01..02 + RS-08 + D-16 contract preserved.

---

### Plan 06-06 — Strict Zero-Alloc + Vec Weights + LDA-vars=6 Dispatch

#### `crates/xcfun-rs/src/functional.rs` (`Box::leak` → `Vec`; `UnsafeCell` handle)

- **Role:** facade refactor.
- **Data flow:** `set()` → `sync_weights_from_settings` (now writes to `Vec<...>` not `Box::leak`); `eval_setup` populates `UnsafeCell<EvalHandle>` with pre-allocated buffers; `eval` reuses them.
- **Closest analog:** same file lines 198-216 (current `sync_weights_from_settings` with documented Box::leak).
- **Excerpt from analog (current state):**
  ```rust
  fn sync_weights_from_settings(&mut self) {
      const UPSTREAM_FUNCTIONAL_COUNT: usize = 78;
      let mut active: Vec<(FunctionalId, f64)> = Vec::new();
      for fd in FUNCTIONAL_DESCRIPTORS.iter() {
          let idx = fd.id as usize;
          if idx >= UPSTREAM_FUNCTIONAL_COUNT { continue; }
          let w = self.0.settings[idx];
          if w != 0.0 { active.push((fd.id, w)); }
      }
      // Box::leak the slice to obtain `&'static [(FunctionalId, f64)]`.
      // Phase 6: replace `weights: &'static [...]` with `weights: Vec<...>` and drop the leak.
      let leaked: &'static [(FunctionalId, f64)] = Box::leak(active.into_boxed_slice());
      self.0.weights = leaked;
  }
  ```
- **Adaptation notes:** D-17: change `xcfun-eval::Functional::weights` field from `&'static [(FunctionalId, f64)]` to `Vec<(FunctionalId, f64)>`. Drop the `Box::leak` line. `Vec` is `Send + Sync` so RS-10 is preserved (`assert_impl_all!(Functional: Send, Sync)` continues to compile). All read-sites that iterate `for (fid, w) in self.weights.iter()` keep working unchanged. D-12 reusable handle: introduce `UnsafeCell<EvalHandle>` (RESEARCH Pattern 2) with private buffers sized at `eval_setup`. `unsafe impl Sync` carries the doc-comment "Functional is Send + Sync, but eval() is racy if called concurrently on the same instance — clone the Functional or wrap in Mutex for concurrent eval."

#### `crates/xcfun-rs/tests/zero_alloc_strict.rs`

- **Role:** allocator-counting test (per-call alloc count == 0 after warm-up).
- **Closest analog:** `crates/xcfun-rs/tests/zero_alloc.rs` (existing fall-back-(b) form using `CountingAllocator` + stability assertion).
- **Excerpt from analog (lines 30-50):**
  ```rust
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
          unsafe { System.dealloc(ptr, layout) };
      }
  }

  #[global_allocator]
  static ALLOC: CountingAllocator = CountingAllocator;
  ```
- **Adaptation notes:** Strict form: assert `delta_count == 0` (not stability) for 100 consecutive `eval` calls after warm-up. Pre-allocated `EvalHandle` (D-12) is what makes this pass. RESEARCH Pitfall 8: `Box::leak` in `sync_weights_from_settings` (now `Vec`) was a blocker — D-17 fix unblocks. Also forbids any `format!` or `Vec::new` on the eval path.

#### `crates/xcfun-rs/tests/no_leak_on_set.rs`

- **Role:** allocator-counting test (specific to D-17 — `set()` no longer Box::leaks).
- **Closest analog:** `crates/xcfun-rs/tests/zero_alloc.rs` infrastructure.
- **Adaptation notes:** Track allocations across 100 `set` calls; assert no `Box::leak`-induced never-freed allocs. Counts BOTH alloc and dealloc; the difference must stay bounded.

#### `crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs`

- **Role:** integration test (b3lyp / camb3lyp / bp86 in-process eval succeeds).
- **Data flow:** set up b3lyp via `set("b3lyp", 1.0)` → `eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1)` → `eval(...)` → succeeds (not `XcError::NotConfigured`).
- **Closest analog:** `crates/xcfun-eval/tests/alias_canary.rs` (alias resolution pattern).
- **Adaptation notes:** D-18 contract — DensVars-driven dispatch resolves Phase 5 D-14 forward. Mixed LDA+GGA aliases (b3lyp = LDA-VWN5 + GGA-Becke + GGA-LYP) currently route through C++ harness only; Phase 6 makes them in-process.

---

### Plan 06-N1, 06-N2, 06-N3 — Cleanup

Plan 06-N1 (root-cause bisection of inherited Phase-3 forwards) and Plan 06-N3 (post-libm-hybrid sweep) are research-driven workflows; they touch existing kernel files in `crates/xcfun-kernels/src/functionals/{gga,mgga}/*.rs` (post-Plan-06-01 location). Each fix is local; analog is the file being fixed itself.

Plan 06-N2 fixture files (`validation/fixtures/mpmath/*.jsonl`) reuse the Plan 06-00 mpmath sidecar pipeline verbatim — same JSONL schema, same `--check` drift gate.

`validation/src/main.rs` (Plan 06-N2 CLI extension): same arg-parsing pattern as Plan 06-03 `--backend rocm` analog.

---

## Shared Patterns

### `OnceLock<R::Client>` per-runtime client cache
**Source:** `crates/xcfun-eval/src/for_tests.rs:1-25` (existing `cpu_client()`)
**Apply to:** `crates/xcfun-gpu/src/pool.rs` (Plan 06-02) — generalised to per-feature `static HIP_CLIENT`, `CUDA_CLIENT`, `WGPU_CLIENT`. Promoted from `for_tests` to production module.
```rust
use cubecl::prelude::*;
use std::sync::OnceLock;

pub type CpuClient = ComputeClient<CpuRuntime>;
static CPU_CLIENT: OnceLock<CpuClient> = OnceLock::new();

pub fn cpu_client() -> &'static CpuClient {
    CPU_CLIENT.get_or_init(|| { let device = CpuDevice; CpuRuntime::client(&device) })
}
```

### `#[cube(launch_unchecked)]` adapter pattern
**Source:** `crates/xcfun-eval/src/functional.rs:54-65` (existing `eval_point_kernel`)
**Apply to:** All new `xcfun-gpu/src/batch.rs` kernel adapters (Plan 06-02), all new `crates/xcfun-ad/tests/*` adapters (Plan 06-00), all `eval_vec` host wrappers (Plan 06-05).
```rust
#[cube(launch_unchecked)]
fn eval_point_kernel<F: Float>(
    input: &Array<F>, d: &mut DensVarsDev<F>, out: &mut Array<F>,
    #[comptime] id: u32, #[comptime] vars: u32, #[comptime] n: u32,
) {
    build_densvars::<F>(input, d, vars, n);
    dispatch_kernel::<F>(id, d, out, n);
}
```

### `xtask regen-* --check` drift gate
**Source:** `xtask/src/bin/regen_registry.rs:1-50` (Phase 2 D-21)
**Apply to:** `xtask/src/bin/regen_mpmath_fixtures.rs` (Plan 06-00) — same `--check` exit-2 pattern, SHA-256-stamped JSONL fixtures.
```rust
//! `--check` mode regenerates the Rust sources in memory, hashes them,
//! and compares against the committed `.sha256` stamps. Exits 2 on drift.
```

### `XcError::*` variant addition (preserve `Copy + non_exhaustive`)
**Source:** `crates/xcfun-core/src/error.rs:14-44` (existing pattern)
**Apply to:** D-13/13-A `WgpuNoF64` addition (Plan 06-02). Field types must all be `Copy` — use `&'static str` for the adapter name, NOT `String`.
```rust
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[non_exhaustive]
pub enum XcError {
    #[error("...")]
    Variant { copy_field_only: SomeCopyType, },
    // ...
}
```

### `assert_impl_all!` compile-time invariant gate
**Source:** `crates/xcfun-rs/tests/send_sync.rs:1-5` (RS-10)
**Apply to:** `crates/xcfun-gpu/tests/batch_api_shape.rs` (GPU-01); `crates/xcfun-core/tests/xcerror_copy_invariant.rs` (D-13/13-A); any future Send+Sync invariant.
```rust
use static_assertions::assert_impl_all;
use xcfun_rs::Functional;
assert_impl_all!(Functional: Send, Sync);
```

### `CountingAllocator` for zero-alloc tests
**Source:** `crates/xcfun-rs/tests/zero_alloc.rs:30-50` (Phase 5 D-13 fall-back)
**Apply to:** `tests/zero_alloc_strict.rs` (Plan 06-06 D-12); `tests/no_leak_on_set.rs` (D-17). Strict form asserts `count == 0`; Phase 5's fall-back form asserts stability across calls.

### Workspace `members` ↔ `exclude` migration
**Source:** Phase 5 D-01/D-02 history (current `Cargo.toml` workspace block)
**Apply to:** Plan 06-01 (`xcfun-kernels` joins members); Plan 06-02 (`xcfun-gpu` promoted from exclude → members); Plan 06-04 adds workspace `[workspace.dependencies]` for cubecl-hip/cuda/wgpu.

### Comptime if-chain dispatch on FunctionalId
**Source:** `crates/xcfun-eval/src/dispatch.rs:79-336` (existing 78-arm if-chain) + Phase 1 D-6 design
**Apply to:** Post-mv `xcfun-kernels::dispatch::dispatch_kernel` (Plan 06-01 verbatim); future `xcfun-gpu` per-Backend launch arms (Plan 06-02) follow the same `if comptime!(id == K)` shape.
```rust
#[cube]
pub fn dispatch_kernel<F: Float>(#[comptime] id: u32, d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    if comptime!(id == 0) { crate::functionals::lda::slaterx::slaterx_kernel::<F>(d, out, n); }
    else if comptime!(id == 1) { crate::functionals::gga::pw91::pw86x::pw86x_kernel::<F>(d, out, n); }
    // ... 78 arms ...
}
```

### Stable-bracket scalar correction for AD constant slot
**Source:** `crates/xcfun-ad/src/expand/expm1.rs:51-90` (Phase 2 Plan 02-06 LDAERFX fix) + `expand/erf.rs:174-280` (`erf_precise`)
**Apply to:** Plan 06-00 Task 2 `erf_precise_taylor<F,N>` — seed `t[0]` with `erf_precise(x[0])` (≤ 1 ULP), use derivative-Taylor for `t[i≥1]`.

---

## No Analog Found

Files with no close match in the codebase (planner uses RESEARCH.md patterns + cubecl-book references instead):

| File | Role | Data Flow | Reason | Fall-back source |
|------|------|-----------|--------|------------------|
| `xtask/mpmath_eval/*.py` | Python sidecar | request-response | No Python in repo today | RESEARCH.md §"mpmath Sidecar Architecture" (lines 812-907) |
| `crates/xcfun-gpu/src/auto_backend.rs` (HipRuntime/CudaRuntime/WgpuRuntime probes) | service | request-response | First non-CPU GPU runtime probes in codebase | RESEARCH Pattern 4; cubecl-book installation guide (`cubecl-book/src/getting-started/installation.md`); Pitfall 3 (RDNA-2 HSA env var) |
| `crates/xcfun-gpu/src/batch.rs` (`Batch<'fun, R>`) | service | streaming/batch | First batch runtime in codebase | RESEARCH Patterns 2 + 3 + `docs/design/06-cubecl-strategy.md §5 + §7` |
| `crates/xcfun-gpu/tests/wgpu_no_f64.rs` | unit test | request-response | First Wgpu f64-probe test | RESEARCH Pitfall 5 + Pitfall 7 (P7 silent f32 fallback) |
| `crates/xcfun-gpu/README.md` (RDNA-2 workaround note) | doc | — | First runtime-env-prereq doc | RESEARCH Pitfall 3 + cubecl-hip README upstream |

---

## Metadata

**Analog search scope:**
- `crates/xcfun-ad/src/`, `crates/xcfun-ad/tests/`
- `crates/xcfun-core/src/`
- `crates/xcfun-eval/src/`, `crates/xcfun-eval/tests/`
- `crates/xcfun-rs/src/`, `crates/xcfun-rs/tests/`
- `crates/xcfun-capi/src/`
- `crates/xcfun-gpu/` (currently stub)
- `xtask/src/bin/`
- `validation/src/`, `validation/Cargo.toml`
- Workspace `Cargo.toml`

**Files scanned (selective Read):** ~25 source files, ~6 Cargo.toml, ~5 test files. Searched via `Glob` + `Grep` for `ctaylor_max`, `xcfun_eval_vec`, `Box::leak`, `OnceLock`, `eval_point_kernel`, `assert_impl_all`, `dispatch_kernel`.

**Pattern extraction date:** 2026-04-30

---

## PATTERN MAPPING COMPLETE

**Phase:** 6 - GPU Backends + Batch Lifecycle (`xcfun-kernels` / `xcfun-gpu`)
**Files classified:** ~85 (across 10 plans 06-00 .. 06-N3)
**Analogs found:** 80 / 85

### Coverage
- Files with exact analog: 65 (most kernel-body, test, manifest, error-variant, dispatch, allocator-counting, mv targets)
- Files with role-match analog: 15 (xcfun-gpu test files where role matches but the GPU-runtime data flow is novel)
- Files with no analog (NEW patterns sourced from RESEARCH.md): 5 (Python sidecar, GPU runtime probes, `Batch<'fun, R>`, Wgpu-f64 test, RDNA-2 README)

### Key Patterns Identified
- `OnceLock<R::Client>` per-runtime client cache (`for_tests.rs` → production in `xcfun-gpu::pool`).
- `#[cube(launch_unchecked)]` adapter pattern (used everywhere kernels are launched from host).
- Comptime if-chain `dispatch_kernel<F>` over FunctionalId (78 arms; verbatim moves into `xcfun-kernels`).
- `xtask regen-* --check` SHA-256-stamped drift gate (extends to mpmath fixtures).
- `XcError` Copy + non_exhaustive variant pattern (extends to `WgpuNoF64` with `&'static str` payload to preserve Copy).
- Stable-bracket scalar correction for AD constant slot (`expm1.rs` → `erf_precise_taylor` for ERF AD chain).
- `assert_impl_all!` compile-time gate (Send/Sync invariants on `Functional`, `Batch`).
- `CountingAllocator` zero-alloc test infrastructure (Phase-5 fall-back form → strict form in Plan 06-06).
- Workspace `exclude → members` migration (Phase 5 precedent → applied to xcfun-kernels + xcfun-gpu in Plans 06-01 / 06-02).
- Substrate-first sequencing (D-09): all algebraic deltas in CURRENT tree (Plan 06-00) BEFORE git-mv (Plan 06-01) so post-mv regressions are unambiguously "move bugs".

### File Created
`/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can now reference analog patterns in PLAN.md files; per-plan action items can cite specific file:line excerpts above.
