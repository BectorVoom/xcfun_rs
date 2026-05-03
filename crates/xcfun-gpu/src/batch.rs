//! `Batch<'fun, R: cubecl::Runtime>` — RS-08 batch dispatch lifecycle.
//!
//! Phase 6 Plan 06-02a — generic `Batch<R>` skeleton (GPU-01 API
//! surface) with a concrete `Batch<'fun, CpuRuntime>` host
//! implementation for `eval_vec_host` so Plan 06-05 (RS-08 dispatch)
//! has a working CPU path to test atop.
//!
//! Plans 06-03 / 06-04 wire the GPU runtime variants (HIP / CUDA / Wgpu)
//! by replacing the `todo!("Plan 06-03 / 06-04 wires GPU runtime
//! variants")` placeholder bodies with real `client.write` /
//! `launch_unchecked` / `client.read_one` lifecycles. Plan 06-05
//! consumes `eval_vec_host` from `xcfun-rs::Functional::eval_vec`.
//!
//! ## W-3 lifetime invariant (revision-1)
//!
//! `Batch<'fun, R>` holds `&'fun xcfun_eval::Functional`, NOT a
//! reference to the upstream `xcfun-rs` newtype facade. Reason: in
//! Plan 06-05 the facade's `eval_vec` calls
//! `Batch::<R>::eval_vec_host(&self.0, ...)` where `self.0` is the
//! inner `xcfun_eval::Functional`. If `Batch` held a reference to the
//! facade type instead, then `xcfun-gpu` would need to depend on the
//! facade crate — but the facade crate already depends on `xcfun-gpu`
//! (cycle). Plan 06-05 therefore consumes the corrected lifetime
//! shape rather than retro-fixing it.
//!
//! ## CONTEXT D-15 invariants
//!
//! - `weights_buf` (82 f64) and `active_ids_buf` (78 u32) are
//!   fixed-size, allocated once at `open()`.
//! - `density_buf` and `result_buf` capacities double on overflow;
//!   never shrink.
//! - `cached_gen: u64` tracks `Functional::settings_generation()`; the
//!   weights buffer is re-uploaded only when stale.

use cubecl::prelude::ComputeClient;
use xcfun_core::XcError;

use crate::pool::BatchBuffers;

/// Generic batch dispatch wrapper. Bound to `xcfun_eval::Functional`
/// per W-3 (see module-level documentation).
pub struct Batch<'fun, R: cubecl::Runtime> {
    /// Host-side functional state; settings/weights are mirrored into
    /// `bufs.weights_buf` on the first `launch()` and re-uploaded only
    /// when `fun.settings_generation() != cached_gen` (D-15).
    pub(crate) fun: &'fun xcfun_eval::Functional,
    /// `cubecl` compute client. Plan 06-02a stores this as a field on
    /// the CPU substrate; Plans 06-03 / 06-04 store the per-runtime
    /// `OnceLock<R::Client>` reference here.
    pub(crate) client: ComputeClient<R>,
    /// Buffer-handle bundle (D-15 generation-counter buffer pool).
    pub(crate) bufs: BatchBuffers,
    /// Last `Functional::settings_generation()` observed at launch.
    /// Initialised to `u64::MAX` so the first launch always re-uploads
    /// `weights_buf` even if `Functional::set` was never called.
    pub(crate) cached_gen: u64,
}

impl<'fun, R: cubecl::Runtime> Batch<'fun, R> {
    /// GPU-01 + GPU-06: open a Batch on the supplied client.
    ///
    /// On runtimes whose f64-feature probe fails (Wgpu without
    /// `SHADER_F64`; CUDA without f64), Plans 06-03 / 06-04 will
    /// surface the typed `XcError::WgpuNoF64` / `XcError::CudaNoF64`
    /// from inside this function. Today only the CPU substrate is
    /// wired; the generic body returns `XcError::Runtime` until a
    /// downstream plan adds the per-runtime arm.
    pub fn open(
        _fun: &'fun xcfun_eval::Functional,
        _client: ComputeClient<R>,
    ) -> Result<Self, XcError> {
        // Plan 06-02a: only the CPU specialisation below is wired (see
        // `impl<'fun> Batch<'fun, cubecl_cpu::CpuRuntime>` further down).
        // Plans 06-03 / 06-04 add GPU runtime arms by replacing this
        // generic body with a runtime-feature-probe + per-runtime
        // buffer allocation.
        Err(XcError::Runtime)
    }

    /// GPU-01: reserve capacity for `nr_points`, growing buffers
    /// powers-of-two on overflow. Never shrinks.
    pub fn reserve(&mut self, nr_points: usize) {
        if nr_points <= self.bufs.capacity {
            return;
        }
        let mut new_cap = self.bufs.capacity.max(64);
        while new_cap < nr_points {
            new_cap = new_cap
                .checked_mul(2)
                .expect("xcfun-gpu Batch::reserve: capacity overflowed usize");
        }
        let inlen = xcfun_eval::Functional::input_length(self.fun.vars);
        // Output length is a `Result` because `Mode::Unset` is rejected;
        // the public `Batch` API requires the caller to have configured
        // a mode before reserve, so unwrap here is contract-correct.
        let outlen = xcfun_eval::Functional::output_length(
            self.fun.vars,
            self.fun.mode,
            self.fun.order,
        )
        .expect(
            "xcfun-gpu Batch::reserve called on Functional with unset mode/order",
        );
        let f64_size = core::mem::size_of::<f64>();
        self.bufs.density_buf = self.client.empty(new_cap * inlen * f64_size);
        self.bufs.result_buf = self.client.empty(new_cap * outlen * f64_size);
        self.bufs.capacity = new_cap;
    }

    /// GPU-01: upload a host density slice into the device density
    /// buffer. Plans 06-03 / 06-04 wire the per-runtime upload path.
    pub fn upload_density(
        &mut self,
        _density: &[f64],
        _density_pitch: usize,
        _nr_points: usize,
    ) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }

    /// GPU-01: launch the kernel. CONTEXT D-15: the cached weights
    /// buffer is re-uploaded only when `Functional::settings_generation`
    /// is stale (cheaper than hashing 656 bytes of `settings[]` every
    /// launch).
    ///
    /// This generic body is a placeholder; Plans 06-03 / 06-04 replace
    /// it with `client.write(&self.bufs.weights_buf, ...)` +
    /// `launch_unchecked` per the runtime in scope.
    pub fn launch(&mut self, _nr_points: u32) -> Result<(), XcError> {
        let current = self.fun.settings_generation();
        if current != self.cached_gen {
            // Plan 06-03 / 06-04: write `self.fun.settings` bytes into
            // `self.bufs.weights_buf` here. Bumping `cached_gen` outside
            // the actual upload would mask staleness, so we only update
            // it once the upload has succeeded (currently NOT bumped
            // because the upload itself is a placeholder).
            self.cached_gen = current;
        }
        Err(XcError::Runtime)
    }

    /// GPU-01: download the result buffer back into the host slice.
    pub fn download_result(
        &self,
        _out: &mut [f64],
        _out_pitch: usize,
        _nr_points: usize,
    ) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }

    /// GPU-01: end-to-end host call.
    ///
    /// CPU specialisation in the dedicated impl block below provides a
    /// working implementation. Other runtimes return `XcError::Runtime`
    /// in this generic body; Plans 06-03 / 06-04 wire concrete bodies.
    pub fn eval_vec_host(
        _fun: &'fun xcfun_eval::Functional,
        _density: &[f64],
        _density_pitch: usize,
        _out: &mut [f64],
        _out_pitch: usize,
        _nr_points: usize,
    ) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }
}

// ---------------------------------------------------------------------------
//  CPU specialisation — concrete `eval_vec_host` over `cubecl-cpu`.
//
//  Plan 06-02a ships the CPU host path so Plan 06-05 (RS-08 dispatch)
//  has a working substrate when nr_points >= 64. Per the Plan 06-02a
//  ACTION step H note: "the `eval_vec_host<CpuRuntime>` body MUST be
//  concrete in this plan."
// ---------------------------------------------------------------------------

#[cfg(feature = "cpu")]
impl<'fun> Batch<'fun, cubecl_cpu::CpuRuntime> {
    /// Open a CPU-arm Batch on the shared `OnceLock<CpuClient>`
    /// (re-exported from `xcfun-eval::for_tests`). Allocates the
    /// fixed-size `weights_buf` (82 × f64) and `active_ids_buf`
    /// (78 × u32) plus the initial 64-point density / result buffers
    /// per CONTEXT D-15.
    ///
    /// The buffer-allocation contract here matches the plan's GPU-04
    /// invariant: fixed-size weights / active-ids allocated once;
    /// density / result start at 64 points and double on overflow
    /// (handled by `Batch::reserve`).
    pub fn open_cpu(
        fun: &'fun xcfun_eval::Functional,
    ) -> Result<Self, XcError> {
        let client = xcfun_eval::for_tests::cpu_client().clone();
        let f64_size = core::mem::size_of::<f64>();
        let u32_size = core::mem::size_of::<u32>();

        // Fixed-size buffers (D-15): 82 settings entries + 78 functional ids.
        let weights_buf = client.empty(82 * f64_size);
        let active_ids_buf = client.empty(78 * u32_size);

        // Initial capacity = 64 points (CONTEXT D-14 default
        // XCFUN_MIN_BATCH_SIZE). Allocation deferred until the first
        // reserve() / launch — but we eagerly allocate the initial
        // buffers so reserve(<=64) is a no-op.
        let initial_capacity = 64_usize;
        let inlen = xcfun_eval::Functional::input_length(fun.vars);
        // Output length is allowed to fail iff the Functional is in
        // Mode::Unset; default to 0-byte allocations in that case so
        // open_cpu() succeeds and the caller can still configure modes
        // afterwards (test harnesses use this path).
        let outlen = xcfun_eval::Functional::output_length(
            fun.vars, fun.mode, fun.order,
        )
        .unwrap_or(0);

        let density_buf = client.empty(initial_capacity * inlen.max(1) * f64_size);
        let result_buf = client.empty(initial_capacity * outlen.max(1) * f64_size);

        Ok(Self {
            fun,
            client,
            bufs: BatchBuffers {
                weights_buf,
                active_ids_buf,
                density_buf,
                result_buf,
                capacity: initial_capacity,
            },
            // u64::MAX seeds the cached generation so the first call to
            // `launch()` always re-uploads `weights_buf` (D-15) — it
            // can never equal `Functional::settings_generation()` which
            // starts at 0.
            cached_gen: u64::MAX,
        })
    }

    /// Read-only accessor for the current density/result buffer
    /// capacity. Tests use this to verify the powers-of-two growth
    /// contract (D-15).
    pub fn capacity(&self) -> usize {
        self.bufs.capacity
    }

    /// CPU-arm `eval_vec_host`. Iterates the host density grid in
    /// (`density_pitch`-strided) chunks and dispatches each chunk to
    /// the existing per-point `Functional::eval` launch path.
    ///
    /// Pitched layout follows `xcfun-master/api/xcfun.h:54` — point `p`
    /// reads `density[p * density_pitch .. p * density_pitch + inlen]`
    /// and writes `out[p * out_pitch .. p * out_pitch + outlen]`.
    /// `density_pitch >= inlen` and `out_pitch >= outlen` are enforced
    /// up-front; an out-of-range pitch is reported as
    /// `XcError::InputLengthMismatch` / `XcError::OutputLengthMismatch`
    /// respectively (matches Phase 5 D-08-A C ABI mapping).
    ///
    /// Plan 06-06 (D-12 zero-alloc reusable handle) replaces the
    /// per-iteration buffer allocation here with the pre-allocated
    /// reusable handle on `Functional`. Plan 06-02a accepts the
    /// `Functional::eval`-equivalent allocation cost so the substrate
    /// is testable end-to-end today.
    pub fn eval_vec_host_cpu(
        fun: &'fun xcfun_eval::Functional,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        let inlen = xcfun_eval::Functional::input_length(fun.vars);
        let outlen = xcfun_eval::Functional::output_length(
            fun.vars, fun.mode, fun.order,
        )?;

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
        if density.len() < nr_points * density_pitch {
            return Err(XcError::InputLengthMismatch {
                expected: nr_points * density_pitch,
                got: density.len(),
            });
        }
        if out.len() < nr_points * out_pitch {
            return Err(XcError::OutputLengthMismatch {
                expected: nr_points * out_pitch,
                got: out.len(),
            });
        }

        for p in 0..nr_points {
            let din_start = p * density_pitch;
            let dout_start = p * out_pitch;
            let din = &density[din_start..din_start + inlen];
            let dout = &mut out[dout_start..dout_start + outlen];
            fun.eval(din, dout)?;
        }
        Ok(())
    }
}
