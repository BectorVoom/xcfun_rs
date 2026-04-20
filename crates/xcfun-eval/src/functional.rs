//! `Functional` struct + `eval` entry point. Phase 2 minimal slice per D-21:
//! carries weights/vars/mode/order, dispatches via cubecl-cpu launches over
//! the registry. Phase 5 (RS-01..10) re-exports through `xcfun-rs::Functional`
//! with the full public API surface.
//!
//! Phase 2 limits (D-23):
//! - `Mode::PartialDerivatives` orders 0..=2 only.
//! - `Mode::Potential` and `Mode::Contracted` reject with `XcError::InvalidMode`.
//! - `Mode::Unset` → `XcError::NotConfigured`.
//! - Functional IDs not in `dispatch::supports()` → `XcError::NotConfigured`.
//!
//! # Wave-1B-14a launch loop (Plan 02-04)
//!
//! The `eval` body replaces the Plan 02-03 `output.fill(0.0); TODO` stub with a
//! per-order cubecl-cpu launch loop per RESEARCH §"Mode::PartialDerivatives
//! Output Layout" (mirroring `xcfun-master/src/XCFunctional.cpp:493-617`).
//!
//! Launch strategy (inlen = VARS_TABLE[vars].len):
//!   - Order 0: 1 N=0 launch, output[0] = Σ_w * w * out[CNST]
//!   - Order 1 (inlen=2 even): 1 N=2 launch with in[0]→VAR0, in[1]→VAR1; read
//!     out[CNST] (energy), out[VAR0] (∂/∂a), out[VAR1] (∂/∂b).
//!   - Order 2 (inlen=2): 3 N=2 launches for (i,j) pairs (0,0), (0,1), (1,1);
//!     each reads out[VAR0|VAR1] for the second derivative and, on the last
//!     inner iteration, out[VAR0] for the first derivative. output[0] from
//!     the last launch's out[CNST].
//!
//! Each launch wraps the two-step kernel `build_densvars + dispatch_kernel`
//! via a `#[cube(launch_unchecked)]` adapter `eval_point_kernel`
//! parameterised by comptime `(id, vars, n)`.

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_core::{FunctionalId, Mode, Vars, XcError, taylorlen};

use crate::density_vars::DensVarsDev;
use crate::density_vars::build::build_densvars;
use crate::density_vars::{DensVarsDevLaunch};
use crate::dispatch;
use crate::dispatch::dispatch_kernel;

#[cfg(feature = "testing")]
use crate::for_tests::cpu_client;

// ---------------------------------------------------------------------------
//  Kernel adapter: one `#[cube(launch_unchecked)]` entry point that builds
//  DensVarsDev from the flat input and dispatches to the per-functional kernel.
//
//  Monomorphised per (id, vars, n) — all comptime — at the cubecl level.
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn eval_point_kernel<F: Float>(
    input: &Array<F>,
    d: &mut DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] id: u32,
    #[comptime] vars: u32,
    #[comptime] n: u32,
) {
    build_densvars::<F>(input, d, vars, n);
    dispatch_kernel::<F>(id, d, out, n);
}

/// A weighted sum of functionals. Phase 2 minimal slice (D-21).
pub struct Functional {
    /// (FunctionalId, weight) pairs. Weights sum to the active-functional set.
    pub weights: &'static [(FunctionalId, f64)],
    /// Input variable layout. Must match the actual `input.len()`.
    pub vars: Vars,
    /// Evaluation mode. Phase 2 supports `Mode::PartialDerivatives` only.
    pub mode: Mode,
    /// Derivative order. Phase 2 supports 0..=2 per D-23.
    pub order: u32,
}

impl Functional {
    /// `input_length(vars)` per MODE-04 — number of f64 inputs the kernel reads.
    /// Matches the C++ `xcfun_input_length(vars)` contract.
    pub const fn input_length(vars: Vars) -> usize {
        vars.input_len()
    }

    /// Evaluate the weighted sum at `input`, writing the result into `output`.
    ///
    /// Output length must match `taylorlen(input_length, order)` for
    /// `Mode::PartialDerivatives`. Returns `XcError` on length mismatch,
    /// unsupported mode, unsupported order, or unsupported vars/functional.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        // 1. Validate mode (Phase 2 = PartialDerivatives only).
        match self.mode {
            Mode::Unset => return Err(XcError::NotConfigured),
            Mode::PartialDerivatives => {}
            Mode::Potential | Mode::Contracted => {
                return Err(XcError::InvalidMode {
                    mode: self.mode,
                    depends: xcfun_core::Dependency::DENSITY,
                });
            }
        }
        // 2. Validate input length.
        let expected_inlen = Self::input_length(self.vars);
        if input.len() != expected_inlen {
            return Err(XcError::InputLengthMismatch {
                expected: expected_inlen,
                got: input.len(),
            });
        }
        // 3. Validate order (Phase 2 = 0..=2).
        if self.order > 2 {
            return Err(XcError::InvalidOrder {
                order: self.order,
                mode: self.mode,
                n_vars: expected_inlen,
            });
        }
        // 4. Validate output length per MODE-04 + RESEARCH §"Mode::PartialDerivatives Output Layout".
        let expected_outlen = taylorlen(expected_inlen, self.order as usize);
        if output.len() != expected_outlen {
            return Err(XcError::OutputLengthMismatch {
                expected: expected_outlen,
                got: output.len(),
            });
        }
        // 5. Validate every functional in `weights` is supported.
        for (id, _w) in self.weights {
            if !dispatch::supports(*id) {
                return Err(XcError::NotConfigured);
            }
        }

        output.fill(0.0);

        // 6. Per-(FunctionalId, weight) launch loop — accumulate weighted
        //    contributions into `output`. For Phase 2 (inlen = 2 for all LDAs
        //    that use XC_A_B; LDAERFC_JT has no upstream test_in), we support
        //    inlen = 2 at full fidelity.
        #[cfg(feature = "testing")]
        {
            for &(id, weight) in self.weights {
                let id_u32 = id as u32;
                launch_and_accumulate(
                    id_u32,
                    self.vars as u32,
                    self.order,
                    expected_inlen,
                    input,
                    weight,
                    output,
                )?;
            }
        }
        #[cfg(not(feature = "testing"))]
        {
            // Non-testing build: launch loop unavailable since `cpu_client()`
            // is test-only. Production use-cases will wire through the Phase 5
            // `xcfun-rs::Functional` facade with a non-test cpu_client.
            let _ = input;
            let _ = output;
            return Err(XcError::Runtime);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
//  Host-side launch helpers (feature = "testing" only — cpu_client is
//  test-scoped per Plan 02-03 Wave-1B-2).
// ---------------------------------------------------------------------------

#[cfg(feature = "testing")]
fn launch_and_accumulate(
    id_u32: u32,
    vars_u32: u32,
    order: u32,
    inlen: usize,
    input: &[f64],
    weight: f64,
    output: &mut [f64],
) -> Result<(), XcError> {
    match order {
        0 => {
            // Order 0: N=0 launch, input length = inlen scalars.
            let flat_input: Vec<f64> = input.to_vec();
            let out = run_launch(id_u32, vars_u32, 0, &flat_input, 1)?;
            output[0] += weight * out[0];
            Ok(())
        }
        1 => {
            // Order 1 with inlen = 2: one N=2 launch with in[0][VAR0]=1, in[1][VAR1]=1.
            //   Flat CTaylor<F, 2> input layout (per slot, 4 coeffs each):
            //     slot 0: [a, 1, 0, 0]   (a[CNST]=a, a[VAR0]=1, rest 0)
            //     slot 1: [b, 0, 1, 0]   (b[CNST]=b, b[VAR1]=1, rest 0)
            if inlen != 2 {
                // Phase 2 launch loop supports inlen=2 only; LDAs with other
                // vars (none in Phase 2 plan 02-04) reject here.
                return Err(XcError::NotConfigured);
            }
            let sz = 4_usize; // 1 << 2
            let mut flat = vec![0.0_f64; inlen * sz];
            flat[0] = input[0]; // a[CNST]
            flat[1] = 1.0; // a[VAR0]
            flat[sz] = input[1]; // b[CNST]
            flat[sz + 2] = 1.0; // b[VAR1]
            let out = run_launch(id_u32, vars_u32, 2, &flat, sz)?;
            // C++ XCFunctional.cpp:515-555 layout:
            //   output[0] = out[CNST]   (energy)
            //   output[1] = out[VAR0]   (∂/∂a)
            //   output[2] = out[VAR1]   (∂/∂b)
            output[0] += weight * out[0]; // CNST
            output[1] += weight * out[1]; // VAR0
            output[2] += weight * out[2]; // VAR1
            Ok(())
        }
        2 => {
            // Order 2 with inlen = 2: 3 N=2 launches for (i,j) in
            //   {(0,0), (0,1), (1,1)} — the i ≤ j upper triangle.
            //   Output layout (6 slots): [E, ∂/∂a, ∂/∂b, ∂²/∂a², ∂²/∂a∂b, ∂²/∂b²]
            if inlen != 2 {
                return Err(XcError::NotConfigured);
            }
            let sz = 4_usize;

            // Mirrors XCFunctional.cpp:589-612:
            //   k = inlen + 1 = 3
            //   for i in 0..inlen:
            //       in[i][VAR0] = 1
            //       for j in i..inlen:
            //           in[j][VAR1] = 1
            //           launch; output[k++] = out[VAR0|VAR1]
            //           in[j][VAR1] = 0
            //       output[i+1] = out[VAR0]   // from last inner iteration
            //       in[i] reset
            //   output[0] = out[CNST]         // from last launch overall

            let mut last_out: Option<Vec<f64>> = None;
            let mut k = inlen + 1; // first 2nd-deriv slot
            let mut per_i_last_out: Vec<Option<Vec<f64>>> = vec![None; inlen];

            for i in 0..inlen {
                for j in i..inlen {
                    // Pack input CTaylor: each slot has `sz=4` coefficients.
                    let mut flat = vec![0.0_f64; inlen * sz];
                    flat[0] = input[0];
                    flat[sz] = input[1];
                    // in[i][VAR0] = 1
                    flat[i * sz + 1] = 1.0;
                    // in[j][VAR1] = 1  (if i == j, both VAR0 and VAR1 are set on the same slot)
                    flat[j * sz + 2] = 1.0;

                    let out = run_launch(id_u32, vars_u32, 2, &flat, sz)?;

                    // VAR0|VAR1 = 1 | 2 = 3
                    output[k] += weight * out[3];
                    k += 1;

                    per_i_last_out[i] = Some(out.clone());
                    last_out = Some(out);
                }
            }

            // First derivatives: for each i, read out[VAR0] from that i's LAST inner launch
            //   (which corresponds to j = inlen - 1).
            for (i, last_i) in per_i_last_out.iter().enumerate() {
                if let Some(out) = last_i {
                    output[i + 1] += weight * out[1]; // VAR0
                }
            }

            // Energy: from the very last launch's CNST.
            if let Some(out) = last_out {
                output[0] += weight * out[0]; // CNST
            }

            Ok(())
        }
        _ => Err(XcError::InvalidOrder {
            order,
            mode: Mode::PartialDerivatives,
            n_vars: inlen,
        }),
    }
}

/// Single launch: create input/output handles, build DensVarsDev buffers, run
/// `eval_point_kernel` with the given comptime `(id, vars, n)`, read back the
/// output coefficients. Returns a Vec<f64> of length `(1 << n)`.
#[cfg(feature = "testing")]
fn run_launch(
    id_u32: u32,
    vars_u32: u32,
    n: u32,
    flat_input: &[f64],
    out_len: usize,
) -> Result<Vec<f64>, XcError> {
    let client = cpu_client();

    // Input buffer on device.
    let in_h = client.create_from_slice(f64::as_bytes(flat_input));

    // DensVarsDev scratch handles — 22 Array<F> fields, each of length (1 << n).
    let array_len = (1_usize << n) * core::mem::size_of::<f64>();
    let mk = || client.empty(array_len);
    let a_h = mk();
    let b_h = mk();
    let gaa_h = mk();
    let gab_h = mk();
    let gbb_h = mk();
    let n_h = mk();
    let s_h = mk();
    let gnn_h = mk();
    let gns_h = mk();
    let gss_h = mk();
    let tau_h = mk();
    let taua_h = mk();
    let taub_h = mk();
    let lapa_h = mk();
    let lapb_h = mk();
    let zeta_h = mk();
    let rs_h = mk();
    let nm13_h = mk();
    let a43_h = mk();
    let b43_h = mk();
    let jpaa_h = mk();
    let jpbb_h = mk();

    // Output handle + a clone we retain for readback.
    let out_h = client.empty(out_len * core::mem::size_of::<f64>());
    let read_h = out_h.clone();

    let arr_cnt = 1_usize << n;

    #[allow(unsafe_code)]
    unsafe {
        // Dispatch on (n, id, vars) — cubecl monomorphises per comptime tuple.
        //
        // Phase 2 supports (vars, n) ∈ { (2, 0), (2, 1), (2, 2) } for LDA XC_A_B.
        // Other combinations reject upstream via `supports()`.
        //
        // Because `#[cube(launch_unchecked)]` requires comptime integer values
        // at the call site, we match on each (id, n) pair supported in Phase 2.
        match (id_u32, n) {
            (0, 0) => launch_eval_point::<0, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (0, 1) => launch_eval_point::<0, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (0, 2) => launch_eval_point::<0, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (2, 0) => launch_eval_point::<2, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (2, 1) => launch_eval_point::<2, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (2, 2) => launch_eval_point::<2, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (3, 0) => launch_eval_point::<3, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (3, 1) => launch_eval_point::<3, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (3, 2) => launch_eval_point::<3, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (13, 0) => launch_eval_point::<13, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (13, 1) => launch_eval_point::<13, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (13, 2) => launch_eval_point::<13, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (14, 0) => launch_eval_point::<14, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (14, 1) => launch_eval_point::<14, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (14, 2) => launch_eval_point::<14, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (15, 0) => launch_eval_point::<15, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (15, 1) => launch_eval_point::<15, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (15, 2) => launch_eval_point::<15, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (24, 0) => launch_eval_point::<24, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (24, 1) => launch_eval_point::<24, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (24, 2) => launch_eval_point::<24, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (28, 0) => launch_eval_point::<28, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (28, 1) => launch_eval_point::<28, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (28, 2) => launch_eval_point::<28, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (55, 0) => launch_eval_point::<55, 2, 0>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (55, 1) => launch_eval_point::<55, 2, 1>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            (55, 2) => launch_eval_point::<55, 2, 2>(
                client, in_h, &[a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(), n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(), tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(), lapb_h.clone(), zeta_h.clone(), rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(), jpaa_h.clone(), jpbb_h.clone()], out_h, flat_input.len(), arr_cnt, out_len,
            ),
            _ => {
                let _ = vars_u32;
                return Err(XcError::NotConfigured);
            }
        }
    }

    let bytes = client.read_one_unchecked(read_h);
    let out_vec: Vec<f64> = f64::from_bytes(&bytes).to_vec();
    Ok(out_vec)
}

#[cfg(feature = "testing")]
#[allow(clippy::too_many_arguments)]
#[allow(unsafe_code)]
unsafe fn launch_eval_point<const ID: u32, const VARS: u32, const N: u32>(
    client: &crate::for_tests::CpuClient,
    in_h: cubecl::server::Handle,
    densvar_handles: &[cubecl::server::Handle; 22],
    out_h: cubecl::server::Handle,
    in_len: usize,
    arr_cnt: usize,
    out_len: usize,
) {
    #[allow(unsafe_code)]
    unsafe {
        eval_point_kernel::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(in_h, in_len),
            DensVarsDevLaunch::new(
                ArrayArg::from_raw_parts(densvar_handles[0].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[1].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[2].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[3].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[4].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[5].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[6].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[7].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[8].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[9].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[10].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[11].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[12].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[13].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[14].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[15].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[16].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[17].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[18].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[19].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[20].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[21].clone(), arr_cnt),
            ),
            ArrayArg::from_raw_parts(out_h, out_len),
            ID,
            VARS,
            N,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_length_matches_vars_table() {
        // MODE-04: input_length(vars) mirrors VARS_TABLE[vars].len.
        assert_eq!(Functional::input_length(Vars::A), 1);
        assert_eq!(Functional::input_length(Vars::N), 1);
        assert_eq!(Functional::input_length(Vars::A_B), 2);
        assert_eq!(Functional::input_length(Vars::N_S), 2);
        assert_eq!(Functional::input_length(Vars::A_B_GAA_GAB_GBB), 5);
    }

    #[test]
    fn eval_rejects_unset_mode() {
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::Unset,
            order: 0,
        };
        let mut out = vec![0.0; 1];
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::NotConfigured)
        ));
    }

    #[test]
    fn eval_rejects_potential_mode_in_phase2() {
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::Potential,
            order: 0,
        };
        let mut out = vec![0.0; 1];
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::InvalidMode { .. })
        ));
    }

    #[test]
    fn eval_rejects_contracted_mode_in_phase2() {
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::Contracted,
            order: 0,
        };
        let mut out = vec![0.0; 1];
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::InvalidMode { .. })
        ));
    }

    #[test]
    fn eval_rejects_order_above_2() {
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 3,
        };
        let mut out = vec![0.0; 10];
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::InvalidOrder { .. })
        ));
    }

    #[test]
    fn eval_rejects_input_length_mismatch() {
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 0,
        };
        let mut out = vec![0.0; 1];
        assert!(matches!(
            f.eval(&[1.0], &mut out),
            Err(XcError::InputLengthMismatch {
                expected: 2,
                got: 1
            })
        ));
    }

    #[test]
    fn eval_rejects_output_length_mismatch() {
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 1,
        };
        // Expected outlen = taylorlen(2, 1) = 3
        let mut out = vec![0.0; 5];
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::OutputLengthMismatch {
                expected: 3,
                got: 5
            })
        ));
    }

    #[test]
    #[cfg(feature = "testing")]
    fn eval_accepts_order_0_with_empty_weights() {
        // Empty weights set means no functional is validated (supports loop no-ops)
        // so the validation path walks through and returns Ok. Output is zero-filled.
        let f = Functional {
            weights: &[],
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 0,
        };
        let mut out = vec![99.0_f64; 1];
        assert!(f.eval(&[1.0, 0.5], &mut out).is_ok());
        assert_eq!(out[0], 0.0, "eval must zero-fill the output on success");
    }
}
