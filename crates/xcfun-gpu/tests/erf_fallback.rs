//! GPU-05 — ERF auto-fallback at `Batch::eval_vec_host` level.
//!
//! On `Backend::Wgpu` and `Backend::Metal`, functionals whose
//! aggregated `Dependency` mask contains `Dependency::ERF` are routed
//! to the CPU substrate at host level — BEFORE the SHADER_F64 probe so
//! the routing decision is independent of device f64 capability. This
//! preserves strict 1e-13 parity for range-separated functionals
//! (`ldaerfx`, `ldaerfc`, `beckecamx`, `beckesrx`, `ldaerfc_jt`) on
//! Apple Silicon and WGSL-only Vulkan drivers (Pitfall 5).
//!
//! ## Two-axis verification
//!
//! 1. **Routing decision (`error_routing::must_fall_back_to_cpu`)** —
//!    the predicate consumed by `Batch::<WgpuRuntime>::
//!    eval_vec_host_wgpu_with_request`. Unit-covered in
//!    `crates/xcfun-gpu/src/error_routing.rs`; replicated here at
//!    integration scope so a regression in the predicate surfaces in
//!    the GPU test suite.
//! 2. **Host-path numerical match** — for non-ERF functionals
//!    (`slaterx`), the Wgpu host path must match the scalar
//!    `Functional::eval` baseline at strict 1e-13 (when SHADER_F64 is
//!    available) or return the typed `XcError::WgpuNoF64` (when not)
//!    — exercising the same dispatch shape that range-separated
//!    functionals would take on the fallback path.
//!
//! ## Why ldaerfx isn't called directly
//!
//! `Dependency::ERF` is a Rust-side extension introduced in Plan
//! 06-02a but the propagation onto the upstream-aligned
//! `FUNCTIONAL_DESCRIPTORS` table is owned by Plan 06-05 (the
//! dispatch site). Today the descriptors mark `ldaerfx` /
//! `ldaerfc` / `ldaerfc_jt` / `beckesrx` / `beckecamx` with
//! `Dependency::DENSITY` or `Dependency::DENSITY | GRADIENT`, NOT
//! with `Dependency::ERF`. Plan 06-04's contract is to wire the
//! mechanism (`must_fall_back_to_cpu` + Batch arm); the descriptor
//! flip is a Plan 06-05 follow-up.

use xcfun_core::traits::Dependency;
use xcfun_gpu::{error_routing::must_fall_back_to_cpu, Backend};

/// Routing-predicate axis: the `Dependency::ERF` bit triggers the CPU
/// fallback on Wgpu and Metal but not on the f64-native backends. The
/// predicate is the load-bearing decision wired into
/// `Batch::<WgpuRuntime>::eval_vec_host_wgpu_with_request`.
#[test]
fn dependency_erf_triggers_cpu_fallback_on_wgpu_and_metal_only() {
    let with_erf = Dependency::DENSITY | Dependency::ERF;

    assert!(must_fall_back_to_cpu(with_erf, Backend::Wgpu));
    assert!(must_fall_back_to_cpu(with_erf, Backend::Metal));
    assert!(!must_fall_back_to_cpu(with_erf, Backend::Cpu));
    assert!(!must_fall_back_to_cpu(with_erf, Backend::Rocm));
    assert!(!must_fall_back_to_cpu(with_erf, Backend::Cuda));

    // Inverse: same backends with NO ERF bit — never fall back.
    let no_erf = Dependency::DENSITY | Dependency::GRADIENT;
    assert!(!must_fall_back_to_cpu(no_erf, Backend::Wgpu));
    assert!(!must_fall_back_to_cpu(no_erf, Backend::Metal));
}

/// Host-path numerical-match axis: for a non-ERF functional, the Wgpu
/// arm of `eval_vec_host_wgpu` must either match the scalar
/// `Functional::eval` baseline at strict 1e-13 (when SHADER_F64 is
/// reachable on the host adapter) or surface the typed
/// `XcError::WgpuNoF64` — never any third path. This exercises the
/// per-point dispatch shape that the ERF auto-fallback path *also*
/// uses (after re-routing to CpuRuntime).
#[cfg(feature = "wgpu")]
#[test]
fn wgpu_host_path_matches_cpu_baseline_or_returns_typed_error() {
    use approx::assert_relative_eq;
    use cubecl_wgpu::WgpuRuntime;
    use xcfun_core::{FunctionalId, Mode, Vars, XcError};
    use xcfun_eval::functional::DEFAULT_SETTINGS;
    use xcfun_eval::Functional;
    use xcfun_gpu::Batch;

    // Direct-struct construction matches the existing xcfun-eval
    // test idiom (potential_lda.rs / self_tests.rs) — `Functional::set`
    // does not mutate vars/mode/order; those are facade-controlled.
    let fun = Functional {
        weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
        vars: Vars::A_B,
        mode: Mode::PartialDerivatives,
        order: 1,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };

    let nr_points = 16_usize;
    let inlen = Functional::input_length(fun.vars);
    let outlen = Functional::output_length(fun.vars, fun.mode, fun.order).unwrap();

    // Synthetic density grid — values stay safely positive so the
    // scalar fallback path never emits NaN.
    let density: Vec<f64> = (0..nr_points * inlen)
        .map(|i| 0.5 + (i as f64) * 0.001)
        .collect();
    let mut wgpu_out = vec![0.0_f64; nr_points * outlen];
    let mut cpu_out = vec![0.0_f64; nr_points * outlen];

    // CPU baseline via per-point Functional::eval.
    for k in 0..nr_points {
        let in_slice = &density[k * inlen..k * inlen + inlen];
        let out_slice = &mut cpu_out[k * outlen..k * outlen + outlen];
        fun.eval(in_slice, out_slice).unwrap();
    }

    // Wgpu host path — exercises the same per-point dispatch the ERF
    // fallback would take after redirecting to CpuRuntime.
    let wgpu_result = Batch::<WgpuRuntime>::eval_vec_host_wgpu(
        &fun,
        &density,
        inlen,
        &mut wgpu_out,
        outlen,
        nr_points,
    );

    match wgpu_result {
        Ok(()) => {
            for i in 0..wgpu_out.len() {
                assert_relative_eq!(wgpu_out[i], cpu_out[i], max_relative = 1e-13);
            }
        }
        Err(XcError::WgpuNoF64 { .. }) => {
            // Adapter lacks SHADER_F64 — typed error. This is the
            // contract: NEVER silent f32 downgrade, NEVER any third
            // outcome.
        }
        Err(other) => panic!(
            "GPU-05/06 contract violated: eval_vec_host_wgpu must return \
             Ok(_) or XcError::WgpuNoF64 — got {other:?}"
        ),
    }
}

/// ERF-fallback dispatch shape: when `must_fall_back_to_cpu` returns
/// true (synthesised via `Dependency::ERF`), the Batch arm code path
/// MUST take the CPU substrate route at the head of
/// `eval_vec_host_wgpu_with_request`. We don't need a real ERF
/// functional in the descriptor table for this — a successful Ok(())
/// from the Wgpu arm with a non-ERF functional that happens to land on
/// a non-f64 adapter would be a contract violation; today the
/// non-fallback path is verified by
/// `wgpu_host_path_matches_cpu_baseline_or_returns_typed_error` above.
///
/// This test specifically exercises the ldaerfx eval shape (input
/// shape, output shape) that a Plan-06-05 ERF descriptor flip would
/// hand to `eval_vec_host_wgpu`. Independent of GPU adapter
/// availability — host-side decision before the runtime probe.
#[cfg(feature = "wgpu")]
#[test]
fn ldaerfx_eval_shape_compatible_with_wgpu_host_path() {
    use cubecl_wgpu::WgpuRuntime;
    use xcfun_core::{Mode, Vars, XcError};
    use xcfun_eval::Functional;
    use xcfun_gpu::Batch;

    // ldaerfx: range-separated LDA exchange. The descriptor reports
    // Dependency::DENSITY today (Plan 06-05 will flip to include ERF).
    // The Wgpu host path is well-defined regardless: an Ok result on a
    // SHADER_F64-capable adapter, or a typed WgpuNoF64 otherwise.
    use xcfun_core::FunctionalId;
    use xcfun_eval::functional::DEFAULT_SETTINGS;
    let fun = Functional {
        weights: vec![(FunctionalId::XC_LDAERFX, 1.0)],
        vars: Vars::A_B,
        mode: Mode::PartialDerivatives,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };

    let nr_points = 8_usize;
    let inlen = Functional::input_length(fun.vars);
    let outlen = Functional::output_length(fun.vars, fun.mode, fun.order).unwrap();
    let density: Vec<f64> = (0..nr_points * inlen).map(|i| 1.0 + (i as f64) * 0.1).collect();
    let mut out = vec![0.0_f64; nr_points * outlen];

    let result = Batch::<WgpuRuntime>::eval_vec_host_wgpu(
        &fun, &density, inlen, &mut out, outlen, nr_points,
    );

    match result {
        Ok(()) => {
            // Wgpu adapter is f64-capable AND ldaerfx ran end-to-end.
            // Output should be finite.
            assert!(
                out.iter().all(|v| v.is_finite()),
                "ldaerfx produced non-finite output values: {out:?}"
            );
        }
        Err(XcError::WgpuNoF64 { .. }) => {
            // No f64-capable Wgpu adapter; expected on Apple Silicon
            // and WGSL-only drivers. Once Plan 06-05 propagates the
            // ERF bit to ldaerfx's descriptor, this branch becomes
            // unreachable (the auto-fallback intercepts before the
            // probe), and the test will then need to assert the CPU
            // baseline match instead.
        }
        Err(other) => panic!("unexpected error variant: {other:?}"),
    }
}
