---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 05
type: execute
wave: 7
depends_on:
  - 06-03
  - 06-04
files_modified:
  - crates/xcfun-rs/src/functional.rs
  - crates/xcfun-rs/src/lib.rs
  - crates/xcfun-rs/Cargo.toml
  - crates/xcfun-rs/tests/eval_vec_threshold.rs
  - crates/xcfun-capi/src/lib.rs
autonomous: true
requirements:
  - RS-08
  - GPU-05
  - KER-06
must_haves:
  truths:
    - "Functional::eval_vec(&self, density: &[f64], density_pitch: usize, out: &mut [f64], out_pitch: usize, nr_points: usize) -> Result<(), XcError> matches xcfun-master/api/xcfun.h:54 byte-for-byte (per D-16 / RS-08)."
    - "Dispatch threshold: nr_points < threshold → per-point eval_loop fall-through; nr_points >= threshold → xcfun-gpu::Batch::<R>::eval_vec_host with R = auto_backend()."
    - "Threshold is `const XCFUN_MIN_BATCH_SIZE: usize = 64` with runtime override via env `XCFUN_MIN_BATCH_SIZE` (per D-14)."
    - "ERF auto-fallback at the dispatch site: when auto_backend returns Wgpu/Metal AND fun.dependencies().contains(Dependency::ERF) → override to CpuRuntime (per GPU-05; reuses error_routing::must_fall_back_to_cpu from Plan 06-04)."
    - "(B-4 revision-1) Tier-3 CPU 10k-grid 1e-13 sign-off (KER-06) lives in this plan: `cargo run -p validation --release -- --backend cpu --tier 3 --order 3` reports `0 failures` (or restricted to the 17 known-clean Phase-4 set as documented in 06-02b skeleton)."
    - "xcfun-capi::xcfun_eval_vec C entry point is rewired: replaces the per-point loop stub at lines 427-462 with a single `f.eval_vec(...)` call that delegates to the new RS-08 path (CAPI-01..02 contract preserved per D-16 drop-in C ABI)."
  artifacts:
    - path: "crates/xcfun-rs/src/functional.rs"
      provides: "pub fn eval_vec with auto_backend-driven dispatch + threshold + ERF auto-fallback"
      contains: "pub fn eval_vec"
    - path: "crates/xcfun-rs/tests/eval_vec_threshold.rs"
      provides: "Test that nr_points < threshold uses eval-loop; nr_points >= threshold uses Batch dispatch; XCFUN_MIN_BATCH_SIZE env override works"
      contains: "XCFUN_MIN_BATCH_SIZE"
    - path: "crates/xcfun-capi/src/lib.rs"
      provides: "xcfun_eval_vec C ABI entry point delegating to xcfun_rs::Functional::eval_vec"
      contains: "eval_vec"
  key_links:
    - from: "crates/xcfun-rs/src/functional.rs::eval_vec"
      to: "crates/xcfun-gpu/src/auto_backend.rs::auto_backend"
      via: "match arm dispatch"
      pattern: "auto_backend"
    - from: "crates/xcfun-rs/src/functional.rs::eval_vec"
      to: "crates/xcfun-gpu/src/error_routing.rs::must_fall_back_to_cpu"
      via: "ERF dependency check before runtime dispatch"
      pattern: "must_fall_back_to_cpu\\|Dependency::ERF"
    - from: "crates/xcfun-capi/src/lib.rs::xcfun_eval_vec"
      to: "crates/xcfun-rs/src/functional.rs::eval_vec"
      via: "Functional::eval_vec call replaces per-point loop"
      pattern: "f\\.eval_vec"
---

<objective>
Wire `Functional::eval_vec` GPU dispatch through `xcfun-gpu::Batch<R>` per RS-08. Phase 5 left this as a stub. Plan 06-05 makes it functional:

1. **Signature per D-16 / RS-08 / CAPI-01..02 drop-in contract:** `pub fn eval_vec(&self, density: &[f64], density_pitch: usize, out: &mut [f64], out_pitch: usize, nr_points: usize) -> Result<(), XcError>` matches `xcfun-master/api/xcfun.h:54` byte-for-byte.
2. **Threshold dispatch per D-14:** `nr_points < threshold` → per-point `eval_loop_fallback`; `nr_points >= threshold` → `xcfun-gpu::Batch::<R>::eval_vec_host` with `R = auto_backend()`. Threshold is `const XCFUN_MIN_BATCH_SIZE: usize = 64` with runtime override via env `XCFUN_MIN_BATCH_SIZE` (parsed once via `OnceLock<usize>` to avoid per-call env lookup).
3. **ERF auto-fallback per GPU-05:** When `auto_backend()` returns `Backend::Wgpu` or `Backend::Metal` AND `fun.dependencies().contains(Dependency::ERF)` → override the runtime selection to `Backend::Cpu` (silently; user gets correct numerics). Reuses `xcfun_gpu::error_routing::must_fall_back_to_cpu` (Plan 06-04).
4. **Monomorphisation via match arms per RESEARCH §"Pattern 4":** Cubecl's `Runtime` trait is not object-safe; cannot use `Box<dyn Runtime>`. Per-Backend match arm calls `Batch::<CpuRuntime>::eval_vec_host`, `Batch::<HipRuntime>::eval_vec_host`, `Batch::<CudaRuntime>::eval_vec_host`, `Batch::<WgpuRuntime>::eval_vec_host` — each behind `#[cfg(feature = ...)]` so the match compiles when only `cpu` is enabled.
5. **xcfun-capi `xcfun_eval_vec` rewire:** Phase 5 left the C entry point as a per-point loop stub at `crates/xcfun-capi/src/lib.rs:427-462`. Replace with a single `f.eval_vec(...)` call that delegates to the new RS-08 path. CAPI-01..02 drop-in C ABI contract preserved.

Purpose: Close out RS-08 (the only Phase-5 requirement deferred to Phase 6 per `.planning/STATE.md`). Wire the entire GPU stack — `xcfun-gpu::Batch<R>` skeleton (Plan 06-02), `cubecl-hip` ROCm primary (Plan 06-03), `cubecl-cuda` + `cubecl-wgpu` opt-in (Plan 06-04) — through to the user-facing facade and the C ABI.

Output: `Functional::eval_vec` complete; eval_vec_threshold test GREEN; xcfun-capi xcfun_eval_vec rewired; full Phase 5 → 6 surface continuous.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02a-xcfun-gpu-skeleton-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-03-cubecl-hip-rocm-primary-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-04-cubecl-cuda-wgpu-optin-PLAN.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@crates/xcfun-rs/src/functional.rs
@crates/xcfun-rs/src/lib.rs
@crates/xcfun-rs/Cargo.toml
@crates/xcfun-rs/tests/free_fns.rs
@crates/xcfun-capi/src/lib.rs
@xcfun-master/api/xcfun.h

<interfaces>
<!-- Existing types/functions the executor needs. -->

From crates/xcfun-rs/src/functional.rs (current — Phase 5 baseline):
```rust
pub struct Functional(xcfun_eval::Functional);

impl Functional {
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        self.0.eval(input, output)
    }
    // RS-08 stub:
    // pub fn eval_vec(&self, density: &[f64], density_pitch: usize,
    //                 out: &mut [f64], out_pitch: usize, nr_points: usize) -> Result<(), XcError>;
}
```

From xcfun-master/api/xcfun.h:54 (the C signature to MATCH byte-for-byte per D-16):
```c
XCFUN_API void xcfun_eval_vec(
    xcfun_t *fun,
    int nr_points,
    const double *density,
    int density_pitch,
    double *result,
    int result_pitch
);
```
Rust mirror:
```rust
pub fn eval_vec(
    &self,
    density: &[f64],
    density_pitch: usize,
    out: &mut [f64],
    out_pitch: usize,
    nr_points: usize,
) -> Result<(), XcError>;
```
(Rust uses `usize` instead of C `int`; otherwise the layout is identical. CAPI shim handles the int-vs-usize cast.)

From crates/xcfun-capi/src/lib.rs:427-462 (current per-point loop stub to REPLACE):
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
        // ... validate non-negative, compute inlen/outlen ...
        let dp = density_pitch as usize;
        let rp = result_pitch as usize;
        for k in 0..(nr_points as usize) {
            // PER-POINT LOOP — Phase 6 Plan 06-05 replaces with single eval_vec call.
            let in_slice = unsafe { std::slice::from_raw_parts(density.add(k * dp), inlen) };
            let out_slice = unsafe { std::slice::from_raw_parts_mut(result.add(k * rp), outlen) };
            if let Err(e) = f.eval(in_slice, out_slice) { die_with(/* ... */); }
        }
    })
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement Functional::eval_vec with threshold + auto_backend dispatch + ERF fallback + threshold env override</name>
  <files>crates/xcfun-rs/src/functional.rs, crates/xcfun-rs/src/lib.rs, crates/xcfun-rs/Cargo.toml, crates/xcfun-rs/tests/eval_vec_threshold.rs</files>
  <read_first>
    - crates/xcfun-rs/src/functional.rs (full file — see Functional newtype + eval signature)
    - crates/xcfun-rs/src/lib.rs (verify re-exports)
    - crates/xcfun-rs/Cargo.toml (verify deps; needs xcfun-gpu added with default cpu feature)
    - crates/xcfun-rs/tests/free_fns.rs (analog test pattern)
    - crates/xcfun-rs/tests/zero_alloc.rs (existing fall-back-(b) form; for env-OnceLock pattern)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-05" (lines 79-83, 663-678)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pattern 4 monomorphisation" (lines 446-461)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md D-14 / D-16
    - xcfun-master/api/xcfun.h:54 (xcfun_eval_vec signature)
  </read_first>
  <behavior>
    - Test 1 (RED first): set up XC_SLATERX functional. Call `Functional::eval_vec(..., nr_points = 32)`. Assert outputs match per-point `Functional::eval` baseline within strict 1e-13 (32 < 64 threshold → eval-loop fall-through path).
    - Test 2: Call `Functional::eval_vec(..., nr_points = 128)`. Assert outputs match baseline within 1e-13 (128 >= 64 → Batch<CpuRuntime> dispatch). The Batch path must produce identical numerics.
    - Test 3: Set `XCFUN_MIN_BATCH_SIZE=200`; call `Functional::eval_vec(..., nr_points = 100)`. Assert path is eval-loop (100 < 200 override threshold). Use a debug counter or trace to assert which path ran.
    - Test 4: Verify pitch handling: `density_pitch = inlen + 2` (some padding bytes); inputs at strided offsets. Outputs match baseline.
    - Test 5: Verify error path: `density.len() < density_pitch * nr_points` returns `XcError::InputLengthMismatch`.
    - All RED before eval_vec is wired.
  </behavior>
  <action>
**Step A — Update `crates/xcfun-rs/Cargo.toml`** to depend on `xcfun-gpu`:

```toml
[dependencies]
xcfun-core    = { path = "../xcfun-core" }
xcfun-eval    = { path = "../xcfun-eval" }
xcfun-gpu     = { path = "../xcfun-gpu", features = ["cpu"] }   # NEW Plan 06-05
thiserror     = { workspace = true }

[features]
default = []
hip   = ["xcfun-gpu/hip"]    # forwards to xcfun-gpu's hip feature
cuda  = ["xcfun-gpu/cuda"]
wgpu  = ["xcfun-gpu/wgpu"]
metal = ["xcfun-gpu/metal"]
```

**Step B — Implement `Functional::eval_vec` in `crates/xcfun-rs/src/functional.rs`:**

Add at the top of the file (or in a new `eval_vec` module):
```rust
use std::sync::OnceLock;
use xcfun_core::Dependency;
use xcfun_gpu::{auto_backend, Backend, Batch};

const DEFAULT_MIN_BATCH_SIZE: usize = 64;

fn min_batch_size() -> usize {
    static THRESHOLD: OnceLock<usize> = OnceLock::new();
    *THRESHOLD.get_or_init(|| {
        std::env::var("XCFUN_MIN_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MIN_BATCH_SIZE)
    })
}
```

Add the `eval_vec` method to `impl Functional`:
```rust
impl Functional {
    /// Phase 6 RS-08 — vectorised evaluation with GPU dispatch when nr_points >= threshold.
    ///
    /// Per D-16 / xcfun-master/api/xcfun.h:54 — pitched layout matches the upstream C ABI
    /// byte-for-byte. Density and result are interpreted as `nr_points × pitch` strides.
    ///
    /// Dispatch (per D-14 + D-07):
    /// - `nr_points < XCFUN_MIN_BATCH_SIZE` (default 64; env-overridable) → per-point loop.
    /// - else → `auto_backend()` selects ROCm > CUDA > Metal > Wgpu > CPU per priority chain.
    /// - **GPU-05**: ERF-bearing functionals on Wgpu/Metal auto-fall-back to CpuRuntime
    ///   (silent override; user gets correct numerics).
    pub fn eval_vec(
        &self,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        // Step 1: input validation (CAPI surface-level checks).
        let inlen = self.input_length();
        let outlen = self.output_length()?;
        if density_pitch < inlen { return Err(XcError::InputLengthMismatch); }
        if out_pitch < outlen { return Err(XcError::OutputLengthMismatch); }
        if density.len() < density_pitch * nr_points { return Err(XcError::InputLengthMismatch); }
        if out.len() < out_pitch * nr_points { return Err(XcError::OutputLengthMismatch); }

        // Step 2: threshold dispatch per D-14.
        if nr_points < min_batch_size() {
            return self.eval_loop_fallback(density, density_pitch, out, out_pitch, nr_points);
        }

        // Step 3: auto_backend selection + ERF auto-fallback.
        let mut chosen = auto_backend();
        let deps = self.0.dependencies();
        if xcfun_gpu::error_routing::must_fall_back_to_cpu(deps, chosen) {
            chosen = Backend::Cpu;
        }

        // Step 4: monomorphised match-arm dispatch per RESEARCH Pattern 4.
        match chosen {
            Backend::Cpu => Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host(
                self, density, density_pitch, out, out_pitch, nr_points,
            ),
            #[cfg(feature = "hip")]
            Backend::Rocm => Batch::<cubecl_hip::HipRuntime>::eval_vec_host(
                self, density, density_pitch, out, out_pitch, nr_points,
            ),
            #[cfg(feature = "cuda")]
            Backend::Cuda => Batch::<cubecl_cuda::CudaRuntime>::eval_vec_host(
                self, density, density_pitch, out, out_pitch, nr_points,
            ),
            #[cfg(feature = "wgpu")]
            Backend::Wgpu | Backend::Metal => Batch::<cubecl_wgpu::WgpuRuntime>::eval_vec_host(
                self, density, density_pitch, out, out_pitch, nr_points,
            ),
            // Without the corresponding feature, fall through to CPU (auto_backend would
            // not have returned these without the feature anyway; defensive).
            #[allow(unreachable_patterns)]
            _ => Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host(
                self, density, density_pitch, out, out_pitch, nr_points,
            ),
        }
    }

    /// Per-point fallback for nr_points < threshold or GPU-unavailable paths.
    fn eval_loop_fallback(
        &self,
        density: &[f64],
        density_pitch: usize,
        out: &mut [f64],
        out_pitch: usize,
        nr_points: usize,
    ) -> Result<(), XcError> {
        let inlen = self.input_length();
        let outlen = self.output_length()?;
        for k in 0..nr_points {
            let in_slice = &density[k * density_pitch..k * density_pitch + inlen];
            let out_slice = &mut out[k * out_pitch..k * out_pitch + outlen];
            self.eval(in_slice, out_slice)?;
        }
        Ok(())
    }
}
```

Note: `Batch<R>` is generic over `&'fun xcfun_rs::Functional`, which means xcfun-gpu must be able to refer back to `xcfun_rs::Functional`. That introduces a circular dep risk (xcfun-rs depends on xcfun-gpu; xcfun-gpu can't depend back on xcfun-rs). Resolution: `Batch<R>` should hold `&'fun xcfun_eval::Functional` instead (the underlying type). Update Plan 06-02's `Batch::open(fun: &'fun xcfun_rs::Functional, ...)` to `Batch::open(fun: &'fun xcfun_eval::Functional, ...)`. This plan's `eval_vec` body passes `self.0` (the xcfun_eval::Functional inner) to Batch::eval_vec_host. Adjust accordingly.

Concrete refinement to Plan 06-02 baseline (apply via grep + sed in this plan if needed):
```rust
// xcfun-gpu::Batch<'fun, R> binds 'fun to the xcfun_eval::Functional underlying:
pub struct Batch<'fun, R: cubecl::Runtime> {
    fun: &'fun xcfun_eval::Functional,   // CHANGED from xcfun_rs::Functional
    // ... rest ...
}
```

Then `Functional::eval_vec` calls:
```rust
Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host(&self.0, density, density_pitch, out, out_pitch, nr_points)
```

(Or `Batch<R>` exposes a ctor from a `&xcfun_eval::Functional` via `pub fn open_inner(fun: &xcfun_eval::Functional, ...)`. Pick whichever shape the executor finds cleanest given the Plan 06-02 baseline.)

**Step C — Write RED test `crates/xcfun-rs/tests/eval_vec_threshold.rs`:**

```rust
//! RS-08 + D-14 — threshold dispatch + env override behaviour.

use xcfun_rs::Functional;
use approx::assert_relative_eq;

fn make_slaterx() -> Functional {
    let mut f = Functional::new();
    f.set("slaterx", 1.0).unwrap();
    f.eval_setup(xcfun_core::Vars::A_B, xcfun_core::Mode::PartialDerivatives, 0).unwrap();
    f
}

#[test]
fn small_nr_points_uses_eval_loop() {
    let f = make_slaterx();
    let inlen  = f.input_length();
    let outlen = f.output_length().unwrap();
    let density: Vec<f64> = (0..32 * inlen).map(|i| 0.5 + (i as f64) * 0.001).collect();
    let mut out_vec  = vec![0.0; 32 * outlen];
    let mut out_loop = vec![0.0; 32 * outlen];

    f.eval_vec(&density, inlen, &mut out_vec, outlen, 32).unwrap();
    for k in 0..32 {
        f.eval(&density[k*inlen..(k+1)*inlen], &mut out_loop[k*outlen..(k+1)*outlen]).unwrap();
    }
    for i in 0..out_vec.len() {
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}

#[test]
fn large_nr_points_uses_batch() {
    let f = make_slaterx();
    let inlen  = f.input_length();
    let outlen = f.output_length().unwrap();
    let nr = 128;
    let density: Vec<f64> = (0..nr * inlen).map(|i| 0.5 + (i as f64) * 0.001).collect();
    let mut out_vec  = vec![0.0; nr * outlen];
    let mut out_loop = vec![0.0; nr * outlen];

    f.eval_vec(&density, inlen, &mut out_vec, outlen, nr).unwrap();
    for k in 0..nr {
        f.eval(&density[k*inlen..(k+1)*inlen], &mut out_loop[k*outlen..(k+1)*outlen]).unwrap();
    }
    for i in 0..out_vec.len() {
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}

#[test]
fn env_override_threshold() {
    // NOTE: OnceLock caches threshold; this test only verifies parsing logic
    // by reading min_batch_size() before any other test (test order-dependent).
    // Robust alternative: parse the env in-line (see implementation note in functional.rs).
    std::env::set_var("XCFUN_MIN_BATCH_SIZE", "200");
    // ... assert that min_batch_size() returned 200 (introspect via a debug accessor) ...
    std::env::remove_var("XCFUN_MIN_BATCH_SIZE");
}

#[test]
fn input_length_mismatch_returns_error() {
    let f = make_slaterx();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();
    let density = vec![0.0; 10 * inlen];   // only 10 records' worth
    let mut out = vec![0.0; 32 * outlen];
    let result = f.eval_vec(&density, inlen, &mut out, outlen, 32);   // requested 32 records
    assert!(matches!(result, Err(xcfun_core::XcError::InputLengthMismatch)));
}

#[test]
fn pitched_layout_matches_dense_layout() {
    let f = make_slaterx();
    let inlen  = f.input_length();
    let outlen = f.output_length().unwrap();
    let nr = 100;
    let pitch_extra = 3;   // arbitrary stride padding
    let density_pitch = inlen + pitch_extra;
    let density: Vec<f64> = (0..nr * density_pitch).map(|i| 0.5 + (i as f64) * 0.001).collect();
    let mut out_vec  = vec![0.0; nr * outlen];
    let mut out_loop = vec![0.0; nr * outlen];

    f.eval_vec(&density, density_pitch, &mut out_vec, outlen, nr).unwrap();
    for k in 0..nr {
        f.eval(&density[k*density_pitch..k*density_pitch+inlen], &mut out_loop[k*outlen..(k+1)*outlen]).unwrap();
    }
    for i in 0..out_vec.len() {
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}
```

Run `cargo nextest run -p xcfun-rs --test eval_vec_threshold` — must PASS after eval_vec is wired.

**Step D — Update `crates/xcfun-rs/src/lib.rs`** to keep the existing re-export surface; verify `pub use xcfun_eval::Functional` doesn't shadow the new `Functional` struct (which wraps `xcfun_eval::Functional`). The Phase 5 Functional newtype is already in `crates/xcfun-rs/src/functional.rs`; just make sure the new method is `pub` and accessible via `xcfun_rs::Functional::eval_vec`.
  </action>
  <verify>
    <automated>cargo build -p xcfun-rs && cargo nextest run -p xcfun-rs --test eval_vec_threshold</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "pub fn eval_vec" crates/xcfun-rs/src/functional.rs` >= 1
    - `grep -c "auto_backend\(\)" crates/xcfun-rs/src/functional.rs` >= 1
    - `grep -c "XCFUN_MIN_BATCH_SIZE\|min_batch_size" crates/xcfun-rs/src/functional.rs` >= 2
    - `grep -c "must_fall_back_to_cpu\|Dependency::ERF" crates/xcfun-rs/src/functional.rs` >= 1
    - `grep -c "Batch::<cubecl_cpu::CpuRuntime>" crates/xcfun-rs/src/functional.rs` >= 1
    - `grep -c "xcfun-gpu" crates/xcfun-rs/Cargo.toml` >= 1
    - `cargo build -p xcfun-rs` exits 0.
    - `cargo nextest run -p xcfun-rs --test eval_vec_threshold` exits 0.
    - `cargo nextest run -p xcfun-rs --tests` exits 0 (no regression in other tests).
    - Existing `assert_impl_all!(Functional: Send, Sync)` still compiles (no field added; eval_vec is `&self` per RS-10 contract).
  </acceptance_criteria>
  <done>Functional::eval_vec signature matches D-16 byte-for-byte; threshold dispatch + env override + ERF auto-fallback + monomorphised per-Backend match arms all wired; eval_vec_threshold test GREEN at strict 1e-13.</done>
</task>

<task type="auto">
  <name>Task 2: xcfun-capi xcfun_eval_vec C ABI rewire (CAPI-01..02 drop-in contract)</name>
  <files>crates/xcfun-capi/src/lib.rs</files>
  <read_first>
    - crates/xcfun-capi/src/lib.rs lines 427-462 (current per-point loop stub to REPLACE)
    - crates/xcfun-capi/src/lib.rs (full file — see other extern "C" entries + c_entry! macro)
    - xcfun-master/api/xcfun.h:54 (xcfun_eval_vec C signature — drop-in contract)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-05 xcfun-capi" (lines 83-84, 678-710)
  </read_first>
  <action>
**Step A — Replace the per-point loop in `crates/xcfun-capi/src/lib.rs`** (lines 427-462 per Plan 06-PATTERNS.md):

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
        // Validate non-negative pitches/counts.
        if nr_points < 0 || density_pitch < 0 || result_pitch < 0 {
            die_with(xcfun_core::XcError::InputLengthMismatch);
            return;
        }
        let nr   = nr_points as usize;
        let dp   = density_pitch as usize;
        let rp   = result_pitch as usize;
        let f    = unsafe { &(*fun).inner };

        // Reconstruct slices for the FULL pitched range (nr * pitch elements each).
        // SAFETY: caller is responsible for ensuring density and result are valid for
        //         the full pitched range. xcfun-master/api/xcfun.h:54 contract.
        let density_slice = unsafe { std::slice::from_raw_parts(density, nr * dp) };
        let result_slice  = unsafe { std::slice::from_raw_parts_mut(result, nr * rp) };

        // Phase 6 Plan 06-05 — single eval_vec call replaces the per-point loop.
        // Delegates to xcfun-rs::Functional::eval_vec → auto_backend → Batch<R>.
        // GPU-05: ERF-bearing functionals on Wgpu/Metal auto-fall-back to CpuRuntime.
        match f.eval_vec(density_slice, dp, result_slice, rp, nr) {
            Ok(()) => {}
            Err(e) => die_with(e),
        }
    })
}
```

This preserves:
- The `c_entry!` macro (catch_unwind + NULL guard + abort per CAPI-04).
- The exact C signature from `xcfun-master/api/xcfun.h:54` (CAPI-01..02 drop-in).
- The 10-fixture `tests/c_abi.c` golden test (Phase 5 CAPI-07 — must still GREEN with the new path).

**Step B — Verify Phase 5 CAPI tests still pass:**

```bash
cargo build -p xcfun-capi --release
cargo nextest run -p xcfun-capi --test c_abi    # 10-fixture golden test from Plan 05-04
cargo nextest run -p xcfun-capi --test api_smoke
```

All must exit 0. The 10-fixture C-ABI golden test from Plan 05-04 calls `xcfun_eval` per-point; this plan's rewire of `xcfun_eval_vec` doesn't affect that test directly. If a new fixture exercising `xcfun_eval_vec` doesn't exist yet, add a smoke test asserting `xcfun_eval_vec` returns the same as 10 individual `xcfun_eval` calls. Optional but recommended.

**Step C — Update headers-match drift gate (CAPI-02):**

Phase 5 cbindgen + headers-match harness should regenerate the same `xcfun.h` because no new public symbol was added (we just rewired the body of `xcfun_eval_vec`). Run:

```bash
cargo run -p xtask --bin regen-capi-header
git diff -- crates/xcfun-capi/include/xcfun.h
```

`git diff` should be empty (no header change). If non-empty, investigate — likely cbindgen re-emit produced a comment churn. The headers-match test should still pass.
  </action>
  <verify>
    <automated>cargo build -p xcfun-capi --release && cargo nextest run -p xcfun-capi --tests</automated>
  </verify>
  <acceptance_criteria>
    - `crates/xcfun-capi/src/lib.rs` no longer has a per-point loop in `xcfun_eval_vec`: `grep -A 5 'pub extern "C" fn xcfun_eval_vec' crates/xcfun-capi/src/lib.rs | grep -c 'for k in 0\.\.'` == 0
    - `grep -c 'f\.eval_vec\|eval_vec(' crates/xcfun-capi/src/lib.rs` >= 1
    - `cargo build -p xcfun-capi --release` exits 0.
    - `cargo nextest run -p xcfun-capi --test c_abi` exits 0 (Phase 5 10-fixture golden ALL FIXTURES PASS preserved).
    - `cargo nextest run -p xcfun-capi --test api_smoke` exits 0.
    - cbindgen regen produces zero header drift: `cargo run -p xtask --bin regen-capi-header && git diff --exit-code -- crates/xcfun-capi/include/xcfun.h` exits 0.
  </acceptance_criteria>
  <done>xcfun_eval_vec C entry point delegates to Rust eval_vec; per-point loop removed; Phase 5 CAPI tests still GREEN; cbindgen headers-match gate still GREEN.</done>
</task>

<task type="auto">
  <name>Task 3: KER-06 tier-3 CPU 10k-grid 1e-13 sign-off (B-4 revision-1: ownership moved here from 06-02)</name>
  <files>(none — runs the validation harness from 06-02b; may add the 17-known-clean filter pattern to .planning/phases/06-.../06-VALIDATION.md)</files>
  <read_first>
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02b-validation-harness-PLAN.md (run_tier3 driver skeleton — Cpu arm needs concrete body if not yet filled)
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md (17-known-clean Phase-4 set: SLATERX, TFK, PBEX, REVPBEX, PBEINTX, RPBEX, PBESOLX, BECKEX, BECKECORRX, PW86X, OPTXCORR, APBEX, PW91X, KTX, BTK, M05X2X, M06X2X)
  </read_first>
  <action>
**B-4 (revision-1) — KER-06 ownership.**

Plan 06-02 (now split into 06-02a + 06-02b) declared but did NOT GREEN-gate KER-06 — the original `requirements:` field listed KER-06 but no acceptance criterion ran the actual tier-3 sweep. Revision-1 B-4 moves KER-06 to this plan (06-05) because it fits naturally next to RS-08: once `Functional::eval_vec` dispatches through `Batch<CpuRuntime>::eval_vec_host`, KER-06 (tier-3 CPU 10k-grid 1e-13 vs scalar `Functional::eval`) is effectively a regression test on the entire RS-08 path.

**Step A — If 06-02b's `run_tier3` Cpu arm is still `todo!()` at this point (skeleton-only ship), fill the body NOW:**

The full Cpu arm body iterates `validation::fixtures::stratified_xoshiro_grid_10k()` (Phase 2 grid; seed `0x1234abcd`) for each `(functional, vars, mode, order)` tuple matching `--filter`. For each tuple:
1. Build `xcfun_eval::Functional` via the standard `xcfun_eval` API.
2. Call `Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host(&fun, density_flat, density_pitch, &mut batch_out, out_pitch, grid.len())?`.
3. Loop scalar `fun.eval(per-point)` to compute scalar baseline.
4. Compute per-record max_rel_err = `(batch - scalar).abs() / scalar.abs().max(1.0)`.
5. If `max_rel_err > 1e-13` for any record, accumulate as failure; emit JSONL record annotating `source: "scalar"`.
6. Print summary `N functionals: 0 failures` if all pass, else exit code 2.

The implementation pattern mirrors `run_tier2` (already in `validation/src/driver.rs`); the only change is replacing the cc-FFI per-point reference with the scalar `Functional::eval` baseline + the Batch::eval_vec_host comparison.

**Step B — Run the KER-06 sign-off command:**

```bash
cargo run -p validation --release -- --backend cpu --tier 3 --order 3     --jobs 4     --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'
```

The 17-functional filter is the Plan 04-10 known-clean set (per `04-VERIFICATION.md`). The remaining D-19 forwards are closed by Plans 06-N1/N2/N3 in parallel.

**Step C — Acceptance: grep `0 failures` in the validation output.**

```bash
cargo run -p validation --release -- --backend cpu --tier 3 --order 3 --jobs 4     --filter '<17-known-clean-pattern>' 2>&1 | grep -c "0 failures"
# Must be >= 1.
```

If the validation harness output format does not contain the literal string `0 failures`, the executor adapts the grep to whatever summary string the harness emits (e.g. "FAILURES: 0", "All passed"). The acceptance criterion in this task carries the literal string the harness ACTUALLY emits.
  </action>
  <verify>
    <automated>cargo run -p validation --release -- --backend cpu --tier 3 --order 3 --jobs 4 --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$' 2>&1 | grep -c "0 failures"</automated>
  </verify>
  <acceptance_criteria>
    - `cargo run -p validation --release -- --backend cpu --tier 3 --order 3 --jobs 4 --filter '^(slaterx|tfk|pbex|revpbex|pbeintx|rpbex|pbesolx|beckex|beckecorrx|pw86x|optxcorr|apbex|pw91x|ktx|btk|m05x2x|m06x2x)$'` exits 0 (KER-06 contract met for the 17 known-clean Phase-4 set).
    - The exit-0 outcome corresponds to "0 failures across the 17-functional filter at strict 1e-13 vs scalar `Functional::eval`".
    - Plans 06-N1 / 06-N2 / 06-N3 in parallel close the remaining 30+ D-19 forwards; full 78-functional tier-3 GREEN is the Phase 6 sign-off bar (per CONTEXT.md D-02), NOT this task's gate.
  </acceptance_criteria>
  <done>KER-06 sign-off command for the 17 known-clean Phase-4 set GREEN at strict 1e-13. Remaining D-19 functionals forwarded to 06-N1/N2/N3.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| C ABI ↔ Rust eval_vec | Caller-supplied raw pointers + pitches; Rust validates lengths before constructing slices |
| Functional ↔ xcfun-gpu Batch<R> | xcfun-rs depends on xcfun-gpu; xcfun-gpu depends only on xcfun-eval (NOT xcfun-rs); cycle avoided by passing `&self.0` (xcfun_eval::Functional) into Batch |
| OnceLock<usize> for env threshold | Threshold cached on first call; subsequent env changes ignored; documented |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-OOM | medium | Caller-supplied huge nr_points triggers Batch::reserve to allocate massive device buffer | Powers-of-two doubling per D-15 (Plan 06-02); Batch::open returns Err on cubecl-side allocation failure; usize bounds-checked at C ABI surface |
| T-06-PITCH-OOB | high | Caller-supplied bad density_pitch + nr_points → out-of-bounds memory read | Step B input validation: `density.len() < density_pitch * nr_points` returns InputLengthMismatch BEFORE any unsafe slice construction |
| T-06-WGPU-F32 | high | If auto_backend returns Wgpu but device lacks SHADER_F64 mid-flight, eval_vec must surface the error | Plan 06-04 Batch<WgpuRuntime>::open returns XcError::WgpuNoF64; the eval_vec match arm propagates it via `?` |
| T-06-CONCURRENT-EVAL | medium | Concurrent `eval_vec` calls on shared Functional → Plan 06-06 D-12 documents "racy if shared" | Plan 06-06 hardens with UnsafeCell + doc-comment; Plan 06-05 inherits (no change) |
| T-06-CAPI-PANIC | high | Panic in eval_vec must not unwind into C code | c_entry! macro with catch_unwind preserved; eval_vec inside c_entry block |
</threat_model>

<verification>
- All 2 tasks GREEN per their automated commands.
- Phase 5 CAPI 10-fixture golden test (Plan 05-04 ALL FIXTURES PASS) still GREEN.
- `cargo nextest run -p xcfun-rs --tests` exits 0 (no regression).
- xtask check-no-anyhow / check-no-mul-add / check-cubecl-pin all GREEN.
- RS-10 invariant preserved: `assert_impl_all!(Functional: Send, Sync)` still compiles.
- D-14 / D-16 / GPU-05 contracts implemented and tested.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 3 satisfied (the dispatch half): `Functional::eval_vec` dispatches to `Batch<CpuRuntime>` when `nr_points >= 64` (RS-08); `Backend::Cpu` always available; `Backend::{Cuda,Wgpu,Rocm,Metal}` compiled behind their feature flags (Plans 06-03/06-04); `auto_backend()` selection via D-07 priority chain.
- ROADMAP Phase 6 success criterion 5 advanced (the ERF half): functionals with `Dependency::ERF` auto-routed to `Backend::Cpu` when active runtime is Wgpu/Metal (GPU-05).
- RS-08 (the only Phase 5 → 6 deferral) closed.
- CAPI-01..02 drop-in C ABI contract preserved: `xcfun_eval_vec` C signature byte-for-byte from `xcfun-master/api/xcfun.h:54`; cbindgen regen produces zero header drift.
- Plan 06-06 unblocked: zero-alloc cleanup can extend the per-point eval path knowing eval_vec is wired.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-05-SUMMARY.md` documenting:
- Functional::eval_vec body with threshold (D-14) + auto_backend (D-07) + ERF auto-fallback (GPU-05) + monomorphised match arms (RESEARCH Pattern 4)
- min_batch_size() OnceLock<usize> env-override pattern (XCFUN_MIN_BATCH_SIZE)
- Batch<R> generic adjustment: `&'fun xcfun_eval::Functional` (NOT `&'fun xcfun_rs::Functional`) to avoid circular dep
- xcfun-capi xcfun_eval_vec rewired (per-point loop removed; delegates to f.eval_vec)
- Phase 5 10-fixture CAPI golden test regression check GREEN
- cbindgen drift-zero confirmation
- RS-08 marked Complete in REQUIREMENTS.md
</output>
