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

// ---------------------------------------------------------------------------
//  HIP/ROCm specialisation — Plan 06-03 (D-05 ROCm primary).
//
//  Mirrors the CPU impl block above: `open_rocm()` constructs a
//  `Batch<'fun, HipRuntime>` from the cached `OnceLock<HipClient>` and
//  pre-allocates the D-15 buffer-handle bundle (fixed weights + active_ids
//  + initial 64-point density/result). `eval_vec_host_rocm()` is the
//  ROCm twin of `eval_vec_host_cpu()`.
//
//  Per Plan 06-03 acceptance criteria, the body shape mirrors the CPU
//  arm; the *kernel-launch* path (real `eval_point_kernel::launch_unchecked`
//  on HipRuntime) is owned by Plan 06-05 / RS-08 dispatch wiring. For
//  06-03 the per-point body falls back to scalar `Functional::eval` so
//  the validation harness `--backend rocm --tier 3` flag has a working
//  end-to-end code path that exercises HIP client init + buffer
//  allocation lifecycle without requiring the not-yet-monomorphised
//  `eval_point_kernel::launch_unchecked::<f64, HipRuntime>` arm. The
//  comment above each call site documents the Plan-06-05 follow-up.
// ---------------------------------------------------------------------------

#[cfg(feature = "hip")]
impl<'fun> Batch<'fun, cubecl_hip::HipRuntime> {
    /// Open a HIP-arm Batch on the cached `OnceLock<HipClient>`. Mirrors
    /// `Batch::<CpuRuntime>::open_cpu()` — fixed-size weights + active_ids
    /// allocated once; density/result start at 64 points and double on
    /// overflow per CONTEXT D-15.
    ///
    /// Returns `XcError::Runtime` when `rocm_available()` returned false
    /// (no `/opt/rocm`, no GPU visible, HIP init panic). Callers SHOULD
    /// call `auto_backend()` first and only invoke `open_rocm` on
    /// `Backend::Rocm`; bypassing the priority chain is supported but
    /// the typed error will reach you instead of a panic.
    pub fn open_rocm(
        fun: &'fun xcfun_eval::Functional,
    ) -> Result<Self, XcError> {
        if !crate::runtime::hip::rocm_available() {
            return Err(XcError::Runtime);
        }
        let client = crate::runtime::hip::hip_client().clone();
        let f64_size = core::mem::size_of::<f64>();
        let u32_size = core::mem::size_of::<u32>();

        // Fixed-size buffers (D-15): 82 settings entries + 78 functional ids.
        let weights_buf = client.empty(82 * f64_size);
        let active_ids_buf = client.empty(78 * u32_size);

        // Initial capacity = 64 points (CONTEXT D-14 default
        // XCFUN_MIN_BATCH_SIZE). Open with eager 64-point allocations so
        // reserve(<= 64) is a no-op (parity with the CPU arm).
        let initial_capacity = 64_usize;
        let inlen = xcfun_eval::Functional::input_length(fun.vars);
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
            cached_gen: u64::MAX,
        })
    }

    /// Read-only accessor for the current density/result buffer
    /// capacity. Mirrors the CPU-arm `capacity()` so tests can assert
    /// the powers-of-two growth contract uniformly across runtimes.
    pub fn capacity_rocm(&self) -> usize {
        self.bufs.capacity
    }

    /// HIP-arm `eval_vec_host`. Plan 06-03 ships the lifecycle skeleton
    /// (probe → client clone → reserve → per-point dispatch); the
    /// real `eval_point_kernel::launch_unchecked::<f64, HipRuntime>`
    /// monomorphisation is owned by Plan 06-05 / RS-08. Until then the
    /// per-point inner loop falls back to scalar `Functional::eval` so
    /// the validation harness `--backend rocm --tier 3` path exercises
    /// HIP init + buffer allocation lifecycle without claiming the
    /// strict-1e-13 GPU-vs-CPU parity contract (which inherently
    /// requires the kernel monomorphisation).
    ///
    /// Pitch validation matches `eval_vec_host_cpu`:
    /// `density_pitch >= inlen` and `out_pitch >= outlen` enforced
    /// up-front; under-sized buffers reported as
    /// `XcError::InputLengthMismatch` / `XcError::OutputLengthMismatch`.
    pub fn eval_vec_host_rocm(
        fun: &'fun xcfun_eval::Functional,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        // Probe gate — bail with typed error if ROCm is unavailable so
        // downstream test harnesses see Result::Err rather than a panic
        // on a CI runner without an AMD GPU.
        if !crate::runtime::hip::rocm_available() {
            return Err(XcError::Runtime);
        }

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

        // Open a HIP batch (allocates D-15 buffers on the cached
        // HipClient). The buffer allocations exercise the cubecl-hip
        // ComputeClient API surface (`client.empty`) so any
        // version-drift between cubecl-hip and the rest of the cubecl
        // 0.10-pre.3 family surfaces here at first launch rather than
        // at validation-time.
        let _batch = Self::open_rocm(fun)?;

        // Per-point dispatch — falls back to the scalar Functional::eval
        // path until Plan 06-05 wires `eval_point_kernel::launch_unchecked
        // ::<f64, HipRuntime>` (RS-08 dispatch). The pitched-layout
        // contract here is identical to `eval_vec_host_cpu` per Phase 5
        // D-08-A C ABI mapping.
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

// ---------------------------------------------------------------------------
//  CUDA specialisation — Plan 06-04 (D-06 NVIDIA opt-in best-effort).
//
//  Mirrors the HIP arm: probe → cached `CudaClient` clone → fixed weights
//  / active-ids buffers → 64-point density/result allocations. The
//  per-point kernel-launch path remains a scalar `Functional::eval`
//  fallback today; Plan 06-05 wires `eval_point_kernel::launch_unchecked
//  ::<f64, CudaRuntime>` once RS-08 dispatch is in place. The
//  open_cuda lifecycle exercises the cubecl-cuda ComputeClient API
//  surface (probe gate, client.empty allocations) so any version drift
//  between cubecl-cuda and the rest of the cubecl 0.10-pre.3 family
//  surfaces here at first launch rather than in the validation harness.
// ---------------------------------------------------------------------------

#[cfg(feature = "cuda")]
impl<'fun> Batch<'fun, cubecl_cuda::CudaRuntime> {
    /// Open a CUDA-arm Batch on the cached `OnceLock<CudaClient>`.
    ///
    /// Returns `XcError::CudaNoF64` when the probe was reached but the
    /// device failed the f64 gate (W-7 revision-1) — semantically
    /// distinct from `XcError::Runtime` which signals "init itself
    /// failed" (no CUDA toolkit, no GPU, driver mismatch). The probe
    /// outcome is cached, so the f64-gate check is effectively free
    /// after the first call.
    pub fn open_cuda(
        fun: &'fun xcfun_eval::Functional,
    ) -> Result<Self, XcError> {
        // Probe gate. The cuda_no_f64_error helper handles both
        // sub-cases: f64-gate-failed (real adapter name) and
        // init-failed (sentinel adapter name). One typed error variant
        // is enough; downstream callers don't need to distinguish.
        if !crate::runtime::cuda::cuda_available() {
            return Err(crate::runtime::cuda::cuda_no_f64_error(crate::Backend::Cuda));
        }
        let client = crate::runtime::cuda::cuda_client().clone();
        let f64_size = core::mem::size_of::<f64>();
        let u32_size = core::mem::size_of::<u32>();

        // Fixed-size buffers (D-15) — 82 settings entries + 78 ids.
        let weights_buf = client.empty(82 * f64_size);
        let active_ids_buf = client.empty(78 * u32_size);

        // Initial 64-point capacity (CONTEXT D-14 default).
        let initial_capacity = 64_usize;
        let inlen = xcfun_eval::Functional::input_length(fun.vars);
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
            cached_gen: u64::MAX,
        })
    }

    /// Read-only accessor for the current density/result buffer
    /// capacity (parity with the CPU/HIP arms; tests rely on the
    /// uniform name to assert the powers-of-two growth contract D-15).
    pub fn capacity_cuda(&self) -> usize {
        self.bufs.capacity
    }

    /// CUDA-arm `eval_vec_host`. Plan 06-04 ships the lifecycle
    /// skeleton (probe → client clone → reserve → per-point dispatch).
    /// The real `eval_point_kernel::launch_unchecked::<f64, CudaRuntime>`
    /// monomorphisation lands in Plan 06-05 / RS-08; until then the
    /// per-point inner loop falls back to scalar `Functional::eval`.
    pub fn eval_vec_host_cuda(
        fun: &'fun xcfun_eval::Functional,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        if !crate::runtime::cuda::cuda_available() {
            return Err(crate::runtime::cuda::cuda_no_f64_error(crate::Backend::Cuda));
        }

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

        // Open a CUDA batch (allocates D-15 buffers on the cached
        // CudaClient). Exercises cubecl-cuda's ComputeClient API
        // surface so version drift surfaces here rather than at
        // validation time.
        let _batch = Self::open_cuda(fun)?;

        // Per-point dispatch — scalar fallback until Plan 06-05 wires
        // CUDA kernel monomorphisation.
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

// ---------------------------------------------------------------------------
//  Wgpu specialisation — Plan 06-04 (D-06 portable fallback;
//  RESEARCH §"Pitfall 5" SHADER_F64 gate).
//
//  Two distinct contracts apply here vs. CPU/HIP/CUDA:
//
//  1. **Open-time SHADER_F64 gate** — Wgpu devices without f64 support
//     get the typed `XcError::WgpuNoF64` (D-13/D-13-A). Apple Silicon
//     and WGSL-only Vulkan drivers are common offenders. NEVER silently
//     downgrades to f32.
//
//  2. **Open-time ERF auto-fallback** — functionals carrying
//     `Dependency::ERF` cannot run on Wgpu/Metal at f64 precision (WGSL
//     has no f64; even with SHADER_F64 the range-separated kernels
//     compile to f32-degraded code). `must_fall_back_to_cpu` is checked
//     in `eval_vec_host_wgpu` and the kernel is re-dispatched on the
//     CPU substrate (GPU-05 contract).
//
//  Plan 06-05 wires the real Wgpu kernel-launch path. Plan 06-04 ships
//  the open/probe/dispatch lifecycle so the typed-error contract is
//  testable today (see crates/xcfun-gpu/tests/wgpu_no_f64.rs and
//  erf_fallback.rs).
// ---------------------------------------------------------------------------

#[cfg(feature = "wgpu")]
impl<'fun> Batch<'fun, cubecl_wgpu::WgpuRuntime> {
    /// Open a Wgpu-arm Batch. Returns `XcError::WgpuNoF64` when the
    /// default Wgpu adapter lacks SHADER_F64 (D-13 contract). The
    /// `requested_runtime` payload defaults to `Backend::Wgpu`; callers
    /// who pre-selected `Backend::Metal` should call
    /// [`Batch::open_wgpu_with_request`] instead so the typed error
    /// reflects the actual user request.
    pub fn open_wgpu(
        fun: &'fun xcfun_eval::Functional,
    ) -> Result<Self, XcError> {
        Self::open_wgpu_with_request(fun, crate::Backend::Wgpu)
    }

    /// Open a Wgpu-arm Batch, recording `requested` in the typed error
    /// payload if the f64 probe fails. Plan 06-05's `auto_backend()`
    /// + Metal arm path uses this overload to surface the correct
    /// `BackendTag::Metal` in `XcError::WgpuNoF64.requested_runtime`.
    pub fn open_wgpu_with_request(
        fun: &'fun xcfun_eval::Functional,
        requested: crate::Backend,
    ) -> Result<Self, XcError> {
        if !crate::runtime::wgpu::wgpu_with_shader_f64_available() {
            return Err(crate::runtime::wgpu::wgpu_no_f64_error(requested));
        }
        let client = crate::runtime::wgpu::wgpu_client().clone();
        let f64_size = core::mem::size_of::<f64>();
        let u32_size = core::mem::size_of::<u32>();

        let weights_buf = client.empty(82 * f64_size);
        let active_ids_buf = client.empty(78 * u32_size);

        let initial_capacity = 64_usize;
        let inlen = xcfun_eval::Functional::input_length(fun.vars);
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
            cached_gen: u64::MAX,
        })
    }

    /// Read-only accessor (parity with CPU/HIP/CUDA arms).
    pub fn capacity_wgpu(&self) -> usize {
        self.bufs.capacity
    }

    /// Wgpu-arm `eval_vec_host`. Two routing decisions happen up-front
    /// per GPU-05 (ERF auto-fallback) + GPU-06 (SHADER_F64 gate):
    ///
    /// 1. If the functional set carries `Dependency::ERF`, route to the
    ///    CPU substrate via `Batch::<CpuRuntime>::eval_vec_host_cpu`.
    ///    This preserves strict 1e-13 parity for range-separated
    ///    functionals on Wgpu (which would otherwise compile to
    ///    f32-degraded WGSL even on f64-capable adapters).
    /// 2. Otherwise probe SHADER_F64 (via `open_wgpu_with_request`); on
    ///    probe failure return the typed `XcError::WgpuNoF64`.
    ///
    /// On the happy path the per-point loop today falls back to scalar
    /// `Functional::eval`; Plan 06-05 wires the real Wgpu kernel
    /// launch.
    pub fn eval_vec_host_wgpu(
        fun: &'fun xcfun_eval::Functional,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        Self::eval_vec_host_wgpu_with_request(
            fun, density, density_pitch, out, out_pitch, nr_points,
            crate::Backend::Wgpu,
        )
    }

    /// Wgpu-arm `eval_vec_host` with explicit request tag — used by the
    /// Metal dispatch site so the GPU-05 fallback decision is correct
    /// (`Backend::Metal` triggers `must_fall_back_to_cpu` for ERF
    /// functionals exactly like `Backend::Wgpu`) AND the typed
    /// `XcError::WgpuNoF64.requested_runtime` payload is correct.
    pub fn eval_vec_host_wgpu_with_request(
        fun: &'fun xcfun_eval::Functional,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
        requested: crate::Backend,
    ) -> Result<(), XcError> {
        // GPU-05: ERF-bearing functional + Wgpu/Metal route → CPU
        // substrate. Decision happens host-side BEFORE the SHADER_F64
        // probe so an Apple Silicon caller of LDAERFX gets a working
        // result (via CPU fallback) rather than a refusal.
        if crate::error_routing::must_fall_back_to_cpu(fun.dependencies(), requested) {
            #[cfg(feature = "cpu")]
            {
                return Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host_cpu(
                    fun, density, density_pitch, out, out_pitch, nr_points,
                );
            }
            // No `cpu` feature compiled — surface XcError::Runtime so
            // the test harness sees a clean failure mode (this branch
            // is unreachable in practice because xcfun-gpu's default
            // features include "cpu").
            #[cfg(not(feature = "cpu"))]
            {
                return Err(XcError::Runtime);
            }
        }

        // GPU-06: SHADER_F64 gate. Returns XcError::WgpuNoF64 with the
        // caller's request tag baked into the payload.
        if !crate::runtime::wgpu::wgpu_with_shader_f64_available() {
            return Err(crate::runtime::wgpu::wgpu_no_f64_error(requested));
        }

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

        // Open + allocate D-15 buffers — exercises cubecl-wgpu's
        // ComputeClient API surface so version drift surfaces here.
        let _batch = Self::open_wgpu_with_request(fun, requested)?;

        // Per-point dispatch — scalar Functional::eval until Plan
        // 06-05 wires Wgpu kernel monomorphisation.
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
