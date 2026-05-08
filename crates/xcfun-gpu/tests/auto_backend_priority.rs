//! GPU-02 — `auto_backend()` priority chain.
//!
//! Phase 6 CONTEXT D-07 priority order:
//!   `XCFUN_FORCE_BACKEND` → ROCm → CUDA → Metal → Wgpu → Cpu.
//!
//! Plan 06-02a only wires the env-var override + the cascading shape;
//! all non-CPU probes return `false` until Plans 06-03 / 06-04 turn
//! them on. So with no env override, `auto_backend()` must fall through
//! to `Backend::Cpu`. Setting `XCFUN_FORCE_BACKEND` to a recognised
//! value short-circuits to that variant; an unrecognised value PANICS
//! (loud failure for misconfigured CI).
//!
//! ## Threading note
//!
//! `std::env::set_var` mutates a process-global; tests in a single
//! binary share that state. We serialise the env-touching tests via a
//! plain `Mutex` to avoid interleaving. `cargo nextest` runs each
//! integration test file in its own process by default — the lock here
//! is belt-and-braces.

use std::sync::Mutex;
use xcfun_gpu::{Backend, auto_backend};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const ENV_VAR: &str = "XCFUN_FORCE_BACKEND";

fn with_env<F: FnOnce()>(value: Option<&str>, f: F) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let prev = std::env::var(ENV_VAR).ok();
    // SAFETY: The Mutex above plus the per-process default of cargo
    // nextest serialises env-mutation across this test binary; no other
    // thread inside this test process is reading `XCFUN_FORCE_BACKEND`
    // concurrently while `f` runs.
    unsafe {
        match value {
            Some(v) => std::env::set_var(ENV_VAR, v),
            None => std::env::remove_var(ENV_VAR),
        }
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    // SAFETY: same justification as the set above; we restore the
    // pre-test env state regardless of whether `f` panicked.
    unsafe {
        match prev {
            Some(v) => std::env::set_var(ENV_VAR, v),
            None => std::env::remove_var(ENV_VAR),
        }
    }
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

/// Plan 06-02a behaviour test: when no GPU runtime feature is enabled,
/// the cascade bottoms out at `Backend::Cpu`. Gate on the absence of
/// every GPU feature so that builds with `--features hip` (Plan 06-03)
/// or `--features cuda` / `--features wgpu` (Plan 06-04) do NOT trigger
/// this assertion: those builds wire real probes that return the GPU
/// variant when the machine has the corresponding hardware available.
#[cfg(not(any(feature = "hip", feature = "cuda", feature = "wgpu")))]
#[test]
fn no_env_falls_through_to_cpu() {
    // With no GPU runtime probes compiled in, the cascade bottoms out
    // at `Backend::Cpu`.
    with_env(None, || {
        assert_eq!(auto_backend(), Backend::Cpu);
    });
}

/// Plan 06-03 behaviour test (feature `hip`): with the ROCm probe wired,
/// the no-env cascade returns `Backend::Rocm` when ROCm is installed
/// locally OR `Backend::Cpu` when the probe correctly reports
/// "ROCm unavailable" (no `/opt/rocm`, no AMD GPU, etc.). Either outcome
/// is acceptable per the priority chain — what we MUST NOT see is a
/// non-`Rocm`/`Cpu` variant (which would indicate the probe escaped its
/// `catch_unwind` / probe-failure return path and crashed into Cuda /
/// Wgpu / Metal arms despite no other GPU feature being enabled).
#[cfg(all(feature = "hip", not(any(feature = "cuda", feature = "wgpu"))))]
#[test]
fn no_env_with_hip_feature_resolves_to_rocm_or_cpu() {
    with_env(None, || {
        let b = auto_backend();
        assert!(
            b == Backend::Rocm || b == Backend::Cpu,
            "expected Rocm (probe succeeded) or Cpu (probe failed); got {:?}",
            b
        );
    });
}

#[test]
fn force_cpu_returns_cpu() {
    with_env(Some("cpu"), || {
        assert_eq!(auto_backend(), Backend::Cpu);
    });
    with_env(Some("CPU"), || {
        assert_eq!(auto_backend(), Backend::Cpu);
    });
}

#[test]
fn force_rocm_returns_rocm_even_when_probe_returns_false() {
    // The env-var override is the highest-priority signal — it must
    // bypass the runtime probe (else the override is useless on a CI
    // machine that lacks a ROCm GPU).
    with_env(Some("rocm"), || {
        assert_eq!(auto_backend(), Backend::Rocm);
    });
    // Alias: cubecl's crate is `cubecl-hip`, accept that spelling too.
    with_env(Some("hip"), || {
        assert_eq!(auto_backend(), Backend::Rocm);
    });
}

#[test]
fn force_cuda_returns_cuda() {
    with_env(Some("cuda"), || {
        assert_eq!(auto_backend(), Backend::Cuda);
    });
}

#[test]
fn force_metal_returns_metal() {
    with_env(Some("metal"), || {
        assert_eq!(auto_backend(), Backend::Metal);
    });
}

#[test]
fn force_wgpu_returns_wgpu() {
    with_env(Some("wgpu"), || {
        assert_eq!(auto_backend(), Backend::Wgpu);
    });
}

#[test]
#[should_panic(expected = "unrecognised")]
fn unrecognised_value_panics() {
    with_env(Some("turbomax9000"), || {
        let _ = auto_backend();
    });
}
