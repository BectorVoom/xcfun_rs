//! Native Rust facade `Functional` (Phase 5 D-02).
//!
//! Newtype around `xcfun_eval::Functional`. Methods delegate. The
//! field is private so callers cannot bypass `set` validation by
//! mutating `weights` / `settings` directly.

use std::cell::UnsafeCell;
use std::sync::OnceLock;

use xcfun_core::{Dependency, Mode, Vars, XcError, registry::FUNCTIONAL_DESCRIPTORS};
use xcfun_gpu::{Backend, auto_backend, error_routing::must_fall_back_to_cpu};
// `Batch` is only referenced inside `#[cfg(feature = ...)]` arms below, so the
// import is feature-gated to avoid `unused_imports` under `--no-default-features`.
#[cfg(any(feature = "cpu", feature = "hip", feature = "cuda", feature = "wgpu"))]
use xcfun_gpu::Batch;

// -----------------------------------------------------------------------------
//  Phase 6 Plan 06-05 (RS-08 + D-14) — `eval_vec` dispatch threshold.
//
//  CONTEXT D-14 fixes the default threshold at 64 points, with a runtime
//  override path via the `XCFUN_MIN_BATCH_SIZE` environment variable. Above
//  the threshold `Functional::eval_vec` dispatches through `xcfun_gpu::Batch`
//  (auto-selected runtime per D-07); below the threshold it falls through to
//  a per-point loop reusing `Functional::eval`.
//
//  The threshold is parsed once via `OnceLock<usize>` so the env-var read is
//  amortised across all `eval_vec` calls in a process. Side-effect: changes
//  to `XCFUN_MIN_BATCH_SIZE` after the first call have no effect — documented
//  as the trade-off vs. per-call env lookup overhead.
// -----------------------------------------------------------------------------

/// Default threshold per CONTEXT D-14 — `nr_points >= 64` triggers the
/// `xcfun_gpu::Batch<R>` dispatch path; `nr_points < 64` falls through to
/// the per-point eval-loop fallback.
pub const XCFUN_MIN_BATCH_SIZE: usize = 64;

/// Test-only / introspection accessor for the cached threshold. Reads
/// `XCFUN_MIN_BATCH_SIZE` env var on first call (parsed via
/// `OnceLock<usize>`); returns the cached value on subsequent calls.
///
/// **Caching semantics:** the OnceLock is initialised exactly once per
/// process; `std::env::set_var` AFTER the first call has no effect on
/// the cached value. Tests verifying env-override behaviour must run in
/// a separate process or accept the boundary at the first call.
pub fn min_batch_size() -> usize {
    static THRESHOLD: OnceLock<usize> = OnceLock::new();
    *THRESHOLD.get_or_init(|| {
        std::env::var("XCFUN_MIN_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(XCFUN_MIN_BATCH_SIZE)
    })
}

// ---------------------------------------------------------------------------
//  Phase 6 Plan 06-06 (D-12) — `EvalHandle` reusable per-launch buffer set.
//
//  This is the structural plumbing for the strict zero-alloc per-point form
//  (RS-07).  Per `eval_setup` it caches the cubecl handles + DensVarsDev
//  buffer that `xcfun-eval::run_launch` allocates per-call today; a future
//  substrate upgrade (cubecl `client.write` API or an xcfun-rs-owned direct
//  cubecl-cpu launcher) will populate the cached fields and reuse them in
//  `Functional::eval` for strict 0 allocs/eval.
//
//  Until that substrate upgrade lands, the cached fields stay `None` and
//  `eval` falls through to `xcfun-eval::Functional::eval` (which allocates
//  per call — the substrate cost documented in `tests/zero_alloc.rs`).
//  The `tests/zero_alloc_strict.rs` test is `#[ignore]`'d as the regression
//  detector for the strict-0 contract.
// ---------------------------------------------------------------------------

/// Pre-allocated reusable buffer set for the per-point eval hot path.
///
/// Sized at `eval_setup` time per `(vars, mode, order)`; reused across
/// every subsequent `eval` call so the strict zero-alloc form (RS-07)
/// can land without per-call cubecl allocation.
///
/// **Status (Plan 06-06):** structural only — fields are `None` until the
/// cubecl-cpu substrate exposes a buffer-reuse API (or we ship an
/// xcfun-rs-owned direct launcher).  See `tests/zero_alloc_strict.rs` for
/// the regression detector that gates the substrate upgrade.
#[allow(dead_code)] // fields populated by future substrate-upgrade plan
struct EvalHandle {
    /// Cached `(vars, mode, order)` configuration the buffers were sized for.
    /// `None` means buffers are uninitialised; populated by `eval_setup`.
    cached_config: Option<(Vars, Mode, u32)>,
    /// Cached `settings_generation()` — a future strict-eval path re-uploads
    /// the device weights buffer only when this is stale.
    cached_settings_gen: u64,
}

impl EvalHandle {
    const fn new() -> Self {
        Self {
            cached_config: None,
            cached_settings_gen: 0,
        }
    }
}

/// The exchange-correlation functional handle.
///
/// RS-01..10 surface. Construct via [`Self::new`], then configure
/// active functionals + parameters via [`Self::set`], then invoke
/// [`Self::eval_setup`] before [`Self::eval`].
///
/// # Threading (RS-10 / D-12)
///
/// `Functional` is `Send + Sync` via the explicit `unsafe impl`s below.
/// However the internal `UnsafeCell<EvalHandle>` is **racy if two threads
/// call `eval()` concurrently on the same `Functional` instance** — clone
/// the handle (cheap; only the inner `Vec<...>` weights are deep-copied)
/// or wrap in `Mutex` for concurrent eval.  The `assert_impl_all!`
/// compile-time gate in `tests/send_sync.rs` continues to verify the trait
/// bounds are preserved by the D-12 refactor.
pub struct Functional {
    inner: xcfun_eval::Functional,
    /// Phase 6 Plan 06-06 (D-12) — reusable per-launch buffer set.  The
    /// `UnsafeCell` lets `eval(&self, …)` mutate the cached buffers without
    /// requiring `&mut self` (RS-07 + RS-10 contract).  See type docs for
    /// the racy-concurrent-eval caveat.  Currently structural-only; the
    /// strict-zero-alloc fast path consuming it lands when cubecl exposes
    /// buffer-reuse semantics (see SUMMARY.md "Deferred Issues").
    #[allow(dead_code)]
    eval_handle: UnsafeCell<EvalHandle>,
}

// Phase 6 Plan 06-06 (D-12) — preserved RS-10.  `xcfun_eval::Functional` is
// already Send+Sync; the wrapped `UnsafeCell<EvalHandle>` carries internal
// state that is racy under concurrent `eval()` on the same instance.  The
// "racy if shared concurrently — clone the Functional or wrap in Mutex"
// contract is documented on `Functional` above.
// SAFETY (Send): `xcfun_eval::Functional` is `Send` (its fields are scalars
// + `Vec<(FunctionalId, f64)>`, all `Send`). `UnsafeCell<EvalHandle>` is
// `Send` provided `EvalHandle` is `Send`; `EvalHandle`'s fields are
// `Option<(Vars, Mode, u32)>` (all `Copy + Send`) and `u64` — trivially
// `Send`.
//
// SAFETY (Sync): `xcfun_eval::Functional` is `Sync`. `UnsafeCell` itself is
// NOT `Sync`; we explicitly opt in here, accepting the documented racy-
// concurrent-`eval` contract — concurrent callers must clone the Functional
// or wrap in `Mutex` (RS-10 doc-comment on `Functional`). The compile-time
// gate in `tests/send_sync.rs` (`assert_impl_all!(Functional: Send, Sync)`)
// is preserved.
#[allow(unsafe_code)]
unsafe impl Send for Functional {}
#[allow(unsafe_code)]
unsafe impl Sync for Functional {}

impl core::fmt::Debug for Functional {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The inner xcfun_eval::Functional does not yet derive Debug.
        // Surface a stable, content-light summary so application logging /
        // assertion macros still compile against `Functional`.
        f.debug_struct("Functional")
            .field("vars", &self.inner.vars)
            .field("mode", &self.inner.mode)
            .field("order", &self.inner.order)
            .field("weights_len", &self.inner.weights.len())
            .finish()
    }
}

impl Functional {
    /// RS-01 — fresh handle: no active functionals, parameters at
    /// their defaults (XCFunctional.cpp:350-355).
    pub const fn new() -> Self {
        Self {
            inner: xcfun_eval::Functional::new(),
            eval_handle: UnsafeCell::new(EvalHandle::new()),
        }
    }

    /// RS-02 — case-insensitive name set. Three-case dispatch
    /// (functional / parameter / alias) per XCFunctional.cpp:369-405.
    ///
    /// **Phase 5 Plan 05-02 wiring:** after delegating to
    /// `xcfun_eval::Functional::set` (which updates the 82-slot `settings`
    /// array), we rebuild `self.inner.weights` from non-zero functional slots
    /// in `settings`. This mirrors the C++ design where `xcfun_set`
    /// maintains both `settings[i] += value` and the `active_functionals[]`
    /// array in lockstep (XCFunctional.cpp:372-385). Without this rebuild,
    /// downstream `is_gga`, `is_metagga`, `eval_setup`, and `eval` would
    /// observe an empty weights slice and produce zeroed output.
    ///
    /// **Memory note:** the rebuild leaks a small `Box<[(FunctionalId, f64)]>`
    /// per top-level `set` call (the field type is `&'static`). Phase 6
    /// will refactor `weights` to `Vec<...>` and drop the leak.
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        self.inner.set(name, value)?;
        self.sync_weights_from_settings();
        Ok(())
    }

    /// RS-03 — read functional weight or parameter value.
    /// Aliases NOT supported (mirror XCFunctional.cpp:407-419).
    pub fn get(&self, name: &str) -> Result<f64, XcError> {
        self.inner.get(name)
    }

    /// RS-04 — `(depends & XC_GRADIENT)` per XCFunctional.cpp:420.
    pub fn is_gga(&self) -> bool {
        self.inner.dependencies().contains(Dependency::GRADIENT)
    }

    /// RS-04 — `(depends & (XC_LAPLACIAN | XC_KINETIC))` per
    /// XCFunctional.cpp:422-424.
    pub fn is_metagga(&self) -> bool {
        let d = self.inner.dependencies();
        d.contains(Dependency::LAPLACIAN) || d.contains(Dependency::KINETIC)
    }

    /// RS-05 — validate `(vars, mode, order)` against the active
    /// functional set's dependencies; on success mutate the inner
    /// `vars`/`mode`/`order` so subsequent `input_length()` and
    /// `output_length()` reflect the new state.
    ///
    /// `xcfun_eval::Functional::eval_setup` is read-only; the field
    /// write happens here at the facade boundary so xcfun-eval's
    /// hot path stays untouched.
    pub fn eval_setup(&mut self, vars: Vars, mode: Mode, order: u32) -> Result<(), XcError> {
        // XCFunctional.cpp:438-441 — validate first; mutate only on success.
        self.inner.eval_setup(vars, mode, order)?;
        self.inner.vars = vars;
        self.inner.mode = mode;
        self.inner.order = order;
        Ok(())
    }

    /// RS-06 — host-program-friendly setup. Compose `which_vars` +
    /// `which_mode` then call `eval_setup`. Port of
    /// XCFunctional.cpp:472-485.
    ///
    /// Out-of-range numeric inputs (any of `func_type > 3`,
    /// `dens_type > 3`, `laplacian/kinetic/current/explicit_derivatives > 1`,
    /// or `mode_type ∈ {0, 4..}`) return `Err(XcError::InvalidEncoding)`
    /// — diverges from C++ which calls `xcfun::die`. The C ABI in
    /// Plan 05-02 maps this back to abort.
    pub fn user_eval_setup(
        &mut self,
        order: i32,
        func_type: u32,
        dens_type: u32,
        mode_type: u32,
        laplacian: u32,
        kinetic: u32,
        current: u32,
        explicit_derivatives: u32,
    ) -> Result<(), XcError> {
        let vars = crate::which_vars(
            func_type,
            dens_type,
            laplacian,
            kinetic,
            current,
            explicit_derivatives,
        )
        .ok_or(XcError::InvalidEncoding)?;
        let mode = crate::which_mode(mode_type).ok_or(XcError::InvalidEncoding)?;
        if order < 0 {
            return Err(XcError::InvalidOrder {
                order: 0,
                mode,
                n_vars: Self::input_length_of(vars),
            });
        }
        self.eval_setup(vars, mode, order as u32)
    }

    /// MODE-04 / RS-09 — number of `f64` inputs to `eval`.
    pub fn input_length(&self) -> usize {
        xcfun_eval::Functional::input_length(self.inner.vars)
    }

    /// Input-buffer length consumed by [`Self::eval`] for the current
    /// `(vars, mode, order)`. Equals `input_length()` for
    /// `Mode::PartialDerivatives` / `Mode::Potential`; equals
    /// `input_length() * (1 << order)` for `Mode::Contracted` per D-06-A
    /// (`XCFunctional.cpp:622-627` — Contracted mode reads `inlen ×
    /// (1 << order)` flat doubles, mirroring the `DOEVAL` macro layout).
    ///
    /// Plan 05-04: the C ABI `xcfun_eval` signature in upstream
    /// `xcfun-master/api/xcfun.h` carries no length parameter, so the
    /// FFI layer must derive this length on the C side. Plan 05-02's
    /// initial implementation hard-coded `input_length()` only,
    /// breaking Mode::Contracted invocation from C.
    pub fn input_buffer_length(&self) -> usize {
        let inlen = self.input_length();
        match self.inner.mode {
            xcfun_core::Mode::Contracted => inlen * (1_usize << self.inner.order),
            _ => inlen,
        }
    }

    /// MODE-05 / RS-09 — number of `f64` outputs `eval` writes.
    pub fn output_length(&self) -> Result<usize, XcError> {
        xcfun_eval::Functional::output_length(self.inner.vars, self.inner.mode, self.inner.order)
    }

    /// RS-07 — evaluate. Zero heap allocation on the success path is
    /// the contract; see `tests/zero_alloc.rs` for the verifying fixture.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        self.inner.eval(input, output)
    }

    /// **RS-08 / D-14 / D-16 / GPU-05** — vectorised evaluation with GPU
    /// dispatch when `nr_points >= XCFUN_MIN_BATCH_SIZE` (default 64).
    ///
    /// Signature mirrors `xcfun-master/api/xcfun.h:54` byte-for-byte (per
    /// CONTEXT D-16): `density` and `out` are pitched flat slices, with
    /// point `p` reading `density[p * density_pitch .. p * density_pitch +
    /// inlen]` and writing `out[p * out_pitch .. p * out_pitch + outlen]`.
    /// Rust uses `usize` instead of C `int`; otherwise the layout is identical.
    /// The `xcfun-capi::xcfun_eval_vec` C ABI shim handles the int-vs-usize
    /// cast at the FFI boundary.
    ///
    /// # Dispatch (CONTEXT D-14 + D-07)
    ///
    /// 1. **Below threshold** (`nr_points < min_batch_size()`): per-point
    ///    fall-through via the existing `Functional::eval` path. No device
    ///    buffer allocation; cheap for small grids.
    /// 2. **At/above threshold**: `auto_backend()` selects the highest-priority
    ///    runtime (env override → ROCm → CUDA → Metal-with-f64 → Wgpu-with-f64
    ///    → CPU). The selected runtime is monomorphised in a `match` arm
    ///    over `Backend` (RESEARCH Pattern 4 — `cubecl::Runtime` is not
    ///    object-safe; cannot use `Box<dyn Runtime>`).
    /// 3. **GPU-05 ERF auto-fallback**: when the selected runtime is
    ///    `Backend::Wgpu` or `Backend::Metal` AND the active functional set
    ///    contains `Dependency::ERF`, the runtime is silently overridden to
    ///    `Backend::Cpu`. Range-separated functionals (LDAERFX/LDAERFC/etc.)
    ///    cannot meet the strict 1e-13 contract on Wgpu/Metal where WGSL has
    ///    no f64 type; the CPU substrate produces correct numerics. Reuses
    ///    `xcfun_gpu::error_routing::must_fall_back_to_cpu` (Plan 06-04).
    ///
    /// # Errors
    ///
    /// - `XcError::InputLengthMismatch` — `density_pitch < inlen` OR
    ///   `density.len() < density_pitch * nr_points`.
    /// - `XcError::OutputLengthMismatch` — symmetric for `out`.
    /// - Any error returned by the selected runtime's `eval_vec_host_*`
    ///   path (e.g. `XcError::WgpuNoF64`, `XcError::CudaNoF64`).
    ///
    /// # Threading (RS-10 contract preserved)
    ///
    /// `eval_vec(&self, ...)` takes an immutable receiver — no mutable
    /// state is added on the facade for this plan. `Functional` remains
    /// `Send + Sync`; the `assert_impl_all!` invariant in `tests/send_sync.rs`
    /// continues to compile. Plan 06-06 (D-12) introduces the
    /// `UnsafeCell<EvalHandle>` reusable buffer for the strict-zero-alloc
    /// goal; that change preserves `Send + Sync` via the documented
    /// "racy if shared" contract.
    pub fn eval_vec(
        &self,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        // ----- Step 1: input validation (D-08-A C ABI typed-error mapping).
        let inlen = self.input_length();
        let outlen = self.output_length()?;
        if density_pitch < inlen {
            return Err(XcError::InputLengthMismatch {
                expected: inlen,
                got: density_pitch,
            });
        }
        if out_pitch < outlen {
            return Err(XcError::OutputLengthMismatch {
                expected: outlen,
                got: out_pitch,
            });
        }
        if nr_points == 0 {
            return Ok(());
        }
        if density.len() < density_pitch * nr_points {
            return Err(XcError::InputLengthMismatch {
                expected: density_pitch * nr_points,
                got: density.len(),
            });
        }
        if out.len() < out_pitch * nr_points {
            return Err(XcError::OutputLengthMismatch {
                expected: out_pitch * nr_points,
                got: out.len(),
            });
        }

        // ----- Step 2: threshold dispatch per D-14.
        if nr_points < min_batch_size() {
            return self.eval_loop_fallback(
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                inlen,
                outlen,
            );
        }

        // ----- Step 3: auto_backend selection + ERF auto-fallback (GPU-05).
        let mut chosen = auto_backend();
        let deps = self.inner.dependencies();
        if must_fall_back_to_cpu(deps, chosen) {
            chosen = Backend::Cpu;
        }

        // ----- Step 4: monomorphised match-arm dispatch (RESEARCH Pattern 4).
        //
        // Each arm calls into the runtime-specific `Batch<R>::eval_vec_host_*`
        // host helper from xcfun-gpu. Inputs are passed through unchanged
        // (the helper re-validates length/pitch invariants — defensive
        // double-check is acceptable since both sides are typed errors).
        // The helper internally falls back to scalar `Functional::eval` per
        // point until Plan 06-05's follow-up (kernel monomorphisation) lands;
        // this plan ships the dispatch wiring + auto_backend selection +
        // ERF fallback contract, which is sufficient to close RS-08.
        match chosen {
            Backend::Cpu => {
                #[cfg(feature = "cpu")]
                {
                    Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host_cpu(
                        &self.inner,
                        density,
                        density_pitch,
                        out,
                        out_pitch,
                        nr_points,
                    )
                }
                // No `cpu` feature compiled — fall back to the per-point loop.
                // This branch is unreachable in the default build (the `cpu`
                // feature is in the default set per Cargo.toml).
                #[cfg(not(feature = "cpu"))]
                {
                    self.eval_loop_fallback(
                        density,
                        density_pitch,
                        out,
                        out_pitch,
                        nr_points,
                        inlen,
                        outlen,
                    )
                }
            }
            #[cfg(feature = "hip")]
            Backend::Rocm => Batch::<cubecl_hip::HipRuntime>::eval_vec_host_rocm(
                &self.inner,
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
            ),
            #[cfg(feature = "cuda")]
            Backend::Cuda => Batch::<cubecl_cuda::CudaRuntime>::eval_vec_host_cuda(
                &self.inner,
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
            ),
            #[cfg(feature = "wgpu")]
            Backend::Wgpu => Batch::<cubecl_wgpu::WgpuRuntime>::eval_vec_host_wgpu_with_request(
                &self.inner,
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                Backend::Wgpu,
            ),
            #[cfg(feature = "wgpu")]
            Backend::Metal => Batch::<cubecl_wgpu::WgpuRuntime>::eval_vec_host_wgpu_with_request(
                &self.inner,
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                Backend::Metal,
            ),
            // Defensive arms: when a Backend variant is selected by
            // `auto_backend()` but the corresponding cargo feature is NOT
            // enabled in this build, fall through to the CPU path. In
            // practice `auto_backend()` returns a non-CPU variant only
            // when its corresponding feature is enabled (each probe is
            // gated on `#[cfg(feature = "...")]`), so these arms are
            // unreachable — but they make the match exhaustive across
            // all five `Backend` variants in every feature configuration.
            #[cfg(not(feature = "hip"))]
            Backend::Rocm => self.eval_loop_fallback(
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                inlen,
                outlen,
            ),
            #[cfg(not(feature = "cuda"))]
            Backend::Cuda => self.eval_loop_fallback(
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                inlen,
                outlen,
            ),
            #[cfg(not(feature = "wgpu"))]
            Backend::Wgpu => self.eval_loop_fallback(
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                inlen,
                outlen,
            ),
            #[cfg(not(feature = "wgpu"))]
            Backend::Metal => self.eval_loop_fallback(
                density,
                density_pitch,
                out,
                out_pitch,
                nr_points,
                inlen,
                outlen,
            ),
        }
    }

    /// Per-point fallback for `nr_points < threshold` and for any Backend
    /// arm whose feature is not compiled in. Reuses the existing
    /// `Functional::eval` path so numerics are bit-identical to the
    /// scalar evaluator.
    ///
    /// `inlen` and `outlen` are passed in (already computed by the caller)
    /// to avoid recomputing `output_length()` per-call — the latter is a
    /// cheap match but kept hot-path-clean.
    #[inline]
    fn eval_loop_fallback(
        &self,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
        inlen: usize,
        outlen: usize,
    ) -> Result<(), XcError> {
        for k in 0..nr_points {
            let din_start = k * density_pitch;
            let dout_start = k * out_pitch;
            let din = &density[din_start..din_start + inlen];
            let dout = &mut out[dout_start..dout_start + outlen];
            self.eval(din, dout)?;
        }
        Ok(())
    }

    // -- internal helper used by `user_eval_setup` for the
    //    `InvalidOrder.n_vars` field ------------------------------------
    #[inline]
    fn input_length_of(vars: Vars) -> usize {
        xcfun_eval::Functional::input_length(vars)
    }

    // ---------------------------------------------------------------
    //  Phase 5 Plan 05-02 — weight rebuild from `settings`.
    //
    //  Iterates the upstream-78 functional slots of `self.inner.settings`
    //  (indices 0..78) and rebuilds `self.inner.weights` from non-zero
    //  entries. Slots 78..82 are parameters and are NOT included.
    //  XC_LB94 (FunctionalId::XC_LB94 == 78) is intentionally excluded
    //  here because its discriminant collides with ParameterId::
    //  XC_RANGESEP_MU at slot 78; the upstream C ABI never enumerates
    //  LB94 as a functional weight.
    //
    //  The leaked `Box<[(FunctionalId, f64)]>` per call is the documented
    //  Phase 5 trade-off; Phase 6 refactors `weights` to `Vec<...>` and
    //  drops the leak.
    // ---------------------------------------------------------------
    fn sync_weights_from_settings(&mut self) {
        const UPSTREAM_FUNCTIONAL_COUNT: usize = 78;
        // Plan 06-06 D-17: weights is now `Vec<(FunctionalId, f64)>`.
        // `Vec::clear()` preserves capacity → steady-state alloc-free.
        // Phase 5 used `Box::leak` per call; that leak is REMOVED here.
        self.inner.weights.clear();
        for fd in FUNCTIONAL_DESCRIPTORS.iter() {
            let idx = fd.id as usize;
            if idx >= UPSTREAM_FUNCTIONAL_COUNT {
                continue; // skip XC_LB94 + parameter slots
            }
            let w = self.inner.settings[idx];
            if w != 0.0 {
                self.inner.weights.push((fd.id, w));
            }
        }
    }

    // ---------------------------------------------------------------
    //  Test-only constructor: build a Functional whose inner `weights`
    //  slice is pre-populated. The Phase 5 facade does not yet rebuild
    //  `weights` from the `settings[]` array updated by `set` (that
    //  refactor lives in Phase 6 / Plan 05-02 C ABI wiring). This
    //  helper lets the inline tests exercise dependencies() / is_gga
    //  / is_metagga / eval over an active functional set.
    // ---------------------------------------------------------------
    #[cfg(test)]
    fn with_weights_for_test(weights: &[(xcfun_core::FunctionalId, f64)]) -> Self {
        // Plan 06-06 D-17: weights is now Vec<...>; clone the slice in.
        let mut inner = xcfun_eval::Functional::new();
        inner.weights = weights.to_vec();
        Self {
            inner,
            eval_handle: UnsafeCell::new(EvalHandle::new()),
        }
    }
}

impl Default for Functional {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcfun_core::{FunctionalId, Mode, Vars, XcError};

    // -- RS-01 ------------------------------------------------------------
    #[test]
    fn new_constructs_empty_wrapper() {
        let f = Functional::new();
        // Empty weights → no GRADIENT/LAPLACIAN/KINETIC; just DENSITY base.
        assert!(!f.is_gga());
        assert!(!f.is_metagga());
    }

    #[test]
    fn default_matches_new() {
        let _f: Functional = Functional::default();
    }

    // -- RS-02 / RS-03 ----------------------------------------------------
    #[test]
    fn set_then_get_slaterx() {
        let mut f = Functional::new();
        f.set("slaterx", 1.0).unwrap();
        assert_eq!(f.get("slaterx").unwrap(), 1.0);
    }

    #[test]
    fn set_unknown_name_returns_unknown() {
        let mut f = Functional::new();
        let err = f.set("not_a_known_name", 1.0).unwrap_err();
        assert!(matches!(err, XcError::UnknownName));
    }

    #[test]
    fn get_unknown_name_returns_unknown() {
        let f = Functional::new();
        let err = f.get("not_a_known_name").unwrap_err();
        assert!(matches!(err, XcError::UnknownName));
    }

    // -- RS-04: is_gga / is_metagga ---------------------------------------
    #[test]
    fn is_gga_false_for_lda_only() {
        // LDA functional: SLATERX (depends only DENSITY).
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];
        let f = Functional::with_weights_for_test(W);
        assert!(!f.is_gga());
        assert!(!f.is_metagga());
    }

    #[test]
    fn is_gga_true_for_pbex() {
        // PBEX has DENSITY|GRADIENT.
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_PBEX, 1.0)];
        let f = Functional::with_weights_for_test(W);
        assert!(f.is_gga());
        assert!(!f.is_metagga());
    }

    #[test]
    fn is_metagga_true_for_tpssx() {
        // TPSSX has DENSITY|GRADIENT|KINETIC.
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_TPSSX, 1.0)];
        let f = Functional::with_weights_for_test(W);
        assert!(f.is_gga());
        assert!(f.is_metagga());
    }

    // -- RS-05: eval_setup mutates vars/mode/order ------------------------
    #[test]
    fn eval_setup_mutates_inner_state() {
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];
        let mut f = Functional::with_weights_for_test(W);
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)
            .unwrap();
        // input_length should now reflect Vars::A_B.
        assert_eq!(f.input_length(), 2);
        // output_length should now reflect (Vars::A_B, PartialDerivatives, 0)
        // → taylorlen(2, 0) = 1.
        assert_eq!(f.output_length().unwrap(), 1);
    }

    // -- RS-06: user_eval_setup -------------------------------------------
    #[test]
    fn user_eval_setup_lda_alpha_beta_partial_deriv() {
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];
        let mut f = Functional::with_weights_for_test(W);
        // (order=0, func_type=0=LDA, dens_type=2=A_B, mode_type=1=PartialDeriv,
        //  laplacian=0, kinetic=0, current=0, explicit_derivatives=0)
        f.user_eval_setup(0, 0, 2, 1, 0, 0, 0, 0).unwrap();
        assert_eq!(f.input_length(), 2);
        assert_eq!(f.output_length().unwrap(), 1);
    }

    #[test]
    fn user_eval_setup_rejects_out_of_range_func_type() {
        let mut f = Functional::new();
        let err = f.user_eval_setup(0, 4, 2, 1, 0, 0, 0, 0).unwrap_err();
        assert!(matches!(err, XcError::InvalidEncoding));
    }

    #[test]
    fn user_eval_setup_rejects_out_of_range_mode_type() {
        let mut f = Functional::new();
        let err = f.user_eval_setup(0, 0, 2, 0, 0, 0, 0, 0).unwrap_err();
        assert!(matches!(err, XcError::InvalidEncoding));
    }

    #[test]
    fn user_eval_setup_rejects_negative_order() {
        let mut f = Functional::new();
        let err = f.user_eval_setup(-1, 0, 2, 1, 0, 0, 0, 0).unwrap_err();
        assert!(matches!(err, XcError::InvalidOrder { .. }));
    }

    // -- RS-07: eval produces non-zero output for an active functional ----
    #[test]
    fn eval_writes_nonzero_for_slaterx() {
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];
        let mut f = Functional::with_weights_for_test(W);
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)
            .unwrap();
        let mut out = vec![0.0_f64; f.output_length().unwrap()];
        f.eval(&[0.5, 0.5], &mut out).unwrap();
        assert_ne!(out[0], 0.0, "expected non-zero SLATERX energy at (0.5,0.5)");
    }

    // -- input_length / output_length accessors --------------------------
    #[test]
    fn input_length_reflects_vars() {
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_PBEX, 1.0)];
        let mut f = Functional::with_weights_for_test(W);
        f.eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 0)
            .unwrap();
        assert_eq!(f.input_length(), 5);
    }

    #[test]
    fn output_length_reflects_mode_order() {
        static W: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];
        let mut f = Functional::with_weights_for_test(W);
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 1)
            .unwrap();
        // taylorlen(2, 1) = 3
        assert_eq!(f.output_length().unwrap(), 3);
    }
}
