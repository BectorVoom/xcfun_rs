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
use xcfun_core::{Dependency, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, Vars, XcError, taylorlen};

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

/// A weighted sum of functionals. Phase 2 minimal slice (D-21) extended in
/// Phase 3 plan 03-02 with B3 `parameters: [f64; 4]` for range-separation /
/// CAM functionals (BECKESRX + BECKECAMX).
pub struct Functional {
    /// (FunctionalId, weight) pairs. Weights sum to the active-functional set.
    pub weights: &'static [(FunctionalId, f64)],
    /// Input variable layout. Must match the actual `input.len()`.
    pub vars: Vars,
    /// Evaluation mode. Phase 2 supports `Mode::PartialDerivatives` only.
    pub mode: Mode,
    /// Derivative order. Phase 2 supports 0..=2 per D-23.
    pub order: u32,
    /// B3 — Range-separation / CAM parameters per
    /// `xcfun-master/src/functionals/common_parameters.cpp:17-29`.
    /// Indices:
    ///   0 = `XC_EXX`         default 0.0
    ///   1 = `XC_RANGESEP_MU` default 0.4
    ///   2 = `XC_CAM_ALPHA`   default 0.19
    ///   3 = `XC_CAM_BETA`    default 0.46
    ///
    /// BECKESRX reads index 1 (RANGESEP_MU); BECKECAMX reads indices 1..=3.
    /// The parameter buffer is launched as an extra cubecl `Array<F>` argument
    /// alongside `DensVarsDev` only by kernels that consume it.
    pub parameters: [f64; 4],
}

/// Default parameters per `common_parameters.cpp:17-29`. Use this when
/// constructing a `Functional` that doesn't override them explicitly.
pub const DEFAULT_PARAMETERS: [f64; 4] = [0.0, 0.4, 0.19, 0.46];

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

    /// Aggregate `Dependency` bitflags across every functional in `self.weights`.
    /// Used by `eval_setup` to decide whether `Mode::Potential` is applicable
    /// (metaGGA-class deps reject) and which `Vars` arms are acceptable.
    ///
    /// Port of `xcfun-master/src/XCFunctional.cpp:~430` `fun->depends` aggregation.
    pub fn dependencies(&self) -> Dependency {
        self.weights
            .iter()
            .map(|(id, _)| FUNCTIONAL_DESCRIPTORS[*id as usize].depends)
            .fold(Dependency::DENSITY, |acc, d| acc | d)
    }

    /// Number of `f64` values written to `output[]` for the given
    /// `(vars, mode, order)` tuple.
    ///
    /// Port of `xcfun-master/src/XCFunctional.cpp:482-490` per D-15:
    /// ```cpp
    /// if (mode == XC_POTENTIAL) {
    ///     if (vars == XC_A || vars == XC_A_2ND_TAYLOR) return 2;
    ///     return 3;  // all spin-resolved cases
    /// }
    /// ```
    ///
    /// `Mode::PartialDerivatives` returns `taylorlen(inlen, order)` per MODE-04.
    /// `Mode::Contracted` and `Mode::Unset` are rejected (`XcError::InvalidMode`
    /// / `XcError::NotConfigured`).
    pub fn output_length(
        vars: Vars,
        mode: Mode,
        order: u32,
    ) -> Result<usize, XcError> {
        match mode {
            Mode::PartialDerivatives => Ok(taylorlen(
                Self::input_length(vars),
                order as usize,
            )),
            Mode::Potential => {
                // D-15 + XCFunctional.cpp:482-490 — single-spin variants return 2,
                // every spin-resolved variant returns 3.
                match vars {
                    Vars::A | Vars::A_2ND_TAYLOR => Ok(2),
                    _ => Ok(3),
                }
            }
            Mode::Contracted => Err(XcError::InvalidMode {
                mode,
                depends: Dependency::DENSITY,
            }),
            Mode::Unset => Err(XcError::NotConfigured),
        }
    }

    /// Host-side setup validation. Port of `xcfun-master/src/XCFunctional.cpp:437-490`
    /// per D-13. Rejects invalid `(mode, vars, order, dependencies)` tuples
    /// BEFORE any kernel launch, so the kernel body can assume valid input.
    ///
    /// Rejection matrix:
    /// - `Mode::Potential` + `Dependency::{LAPLACIAN, KINETIC}` → `InvalidMode`
    ///   (metaGGA-class functionals cannot produce a potential at GGA tier).
    /// - `Mode::Potential` + `Dependency::GRADIENT` + non-`_2ND_TAYLOR` Vars →
    ///   `InvalidVars` (GGA potential requires the 2nd-Taylor-seeded density
    ///   input variants to compute ∇·(∂e/∂∇ρ)).
    /// - `Mode::Unset` → `NotConfigured`.
    /// - `Mode::Contracted` → `InvalidMode` (Phase 2 carryover — contract deferred
    ///   to later phases).
    ///
    /// D-25 resolution: no new `XcError` variants; reuses `InvalidMode` +
    /// `InvalidVars` already present since Phase 2.
    pub fn eval_setup(
        &self,
        vars: Vars,
        mode: Mode,
        _order: u32,
    ) -> Result<(), XcError> {
        let deps = self.dependencies();
        match mode {
            Mode::Unset => Err(XcError::NotConfigured),
            Mode::Contracted => Err(XcError::InvalidMode {
                mode,
                depends: deps,
            }),
            Mode::PartialDerivatives => Ok(()),
            Mode::Potential => {
                // metaGGA-class deps cannot produce a potential at GGA tier.
                if deps.contains(Dependency::LAPLACIAN)
                    || deps.contains(Dependency::KINETIC)
                {
                    return Err(XcError::InvalidMode {
                        mode,
                        depends: deps,
                    });
                }
                // GGA deps require 2nd-Taylor Vars for divergence construction.
                if deps.contains(Dependency::GRADIENT) {
                    match vars {
                        Vars::A_2ND_TAYLOR
                        | Vars::A_B_2ND_TAYLOR
                        | Vars::N_2ND_TAYLOR
                        | Vars::N_S_2ND_TAYLOR => {}
                        _ => {
                            return Err(XcError::InvalidVars {
                                vars,
                                required: Dependency::GRADIENT,
                            });
                        }
                    }
                }
                Ok(())
            }
        }
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
    // Phase 3 plan 03-03 — generalised inlen support: Phase 2 ships inlen=2
    // (LDA XC_A_B). Phase 3 GGAs use inlen=5 (XC_A_B_GAA_GAB_GBB). The launch
    // layout for arbitrary inlen mirrors `XCFunctional.cpp:515-612` exactly:
    //
    //   - Order 0: N=0 launch, input as inlen scalars; output[0] = out[CNST].
    //   - Order 1: 1 N=2 launch; in[i][VAR0]=1 for all i; output[i+1] = out[VAR_i].
    //     Wait — that's the `inlen <= 7` style. For our case the LDA path uses
    //     2 vars (in[0][VAR0]=1, in[1][VAR1]=1) but XCFunctional.cpp uses a
    //     loop where each i seeds in[i][VAR_i]=1. Either approach yields the
    //     same gradient information; we adopt the seeding-loop pattern from
    //     `XCFunctional.cpp:577-612` which generalises to any inlen.
    //   - Order 2: inlen·(inlen+1)/2 launches over (i,j) i≤j.
    //
    // The output layout for `taylorlen(inlen, order)` is the upper triangle:
    //   output[0]                               = energy (CNST)
    //   output[1..inlen+1]                      = ∂/∂x_i for i=0..inlen
    //   output[inlen+1..taylorlen(inlen,2)]     = ∂²/∂x_i∂x_j for i≤j (lex order)
    match order {
        0 => {
            // Order 0: N=0 launch, input length = inlen scalars.
            let flat_input: Vec<f64> = input.to_vec();
            let out = run_launch(id_u32, vars_u32, 0, &flat_input, 1)?;
            output[0] += weight * out[0];
            Ok(())
        }
        1 => {
            // Order 1 with arbitrary inlen ∈ {2, 5}: do `inlen` separate N=2
            // launches. For each i we seed in[i][VAR0]=1 and read out[VAR0]
            // for ∂/∂x_i. This is the per-VAR0-only seeding pattern of
            // XCFunctional.cpp:559-573 — single-direction directional derivative.
            //
            // OPTIMISATION OPPORTUNITY: For inlen=2 we still ship the
            // single-launch dual-seed pattern (in[0][VAR0]=1 + in[1][VAR1]=1)
            // for backwards compatibility with Phase-2 LDA tier-2.
            if inlen == 2 {
                let sz = 4_usize; // 1 << 2
                let mut flat = vec![0.0_f64; inlen * sz];
                flat[0] = input[0];
                flat[1] = 1.0;
                flat[sz] = input[1];
                flat[sz + 2] = 1.0;
                let out = run_launch(id_u32, vars_u32, 2, &flat, sz)?;
                output[0] += weight * out[0];
                output[1] += weight * out[1];
                output[2] += weight * out[2];
                return Ok(());
            }
            // General inlen: per-slot single-VAR0 seed.
            let sz = 4_usize;
            let mut energy_seen = 0.0_f64;
            for i in 0..inlen {
                let mut flat = vec![0.0_f64; inlen * sz];
                for k in 0..inlen {
                    flat[k * sz] = input[k];
                }
                flat[i * sz + 1] = 1.0; // in[i][VAR0] = 1
                let out = run_launch(id_u32, vars_u32, 2, &flat, sz)?;
                if i == inlen - 1 {
                    energy_seen = out[0];
                }
                output[i + 1] += weight * out[1]; // VAR0 → ∂/∂x_i
            }
            output[0] += weight * energy_seen;
            Ok(())
        }
        2 => {
            // Order 2: arbitrary inlen. Generalised version of Phase-2 inlen=2.
            // For inlen ∈ {2, 5}, do inlen·(inlen+1)/2 N=2 launches over (i,j) i≤j.
            //
            // For inlen=2 we keep the Phase-2-compatible dual-seeded path so the
            // LDA tier-2 baseline stays bit-identical.
            if inlen == 2 {
                return launch_and_accumulate_order2_inlen2(
                    id_u32, vars_u32, input, weight, output,
                );
            }
            launch_and_accumulate_order2_general(id_u32, vars_u32, inlen, input, weight, output)
        }
        _ => Err(XcError::InvalidOrder {
            order,
            mode: Mode::PartialDerivatives,
            n_vars: inlen,
        }),
    }
}

#[cfg(feature = "testing")]
fn launch_and_accumulate_order2_inlen2(
    id_u32: u32,
    vars_u32: u32,
    input: &[f64],
    weight: f64,
    output: &mut [f64],
) -> Result<(), XcError> {
    let inlen = 2_usize;
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

/// Order-2 launch loop for arbitrary inlen (Phase 3 plan 03-03 — Wave-2
/// INCONCLUSIVE absorption). Generalises the Phase-2 inlen=2 dual-seed
/// pattern to inlen=5 GGAs by performing inlen·(inlen+1)/2 N=2 launches
/// over the (i, j) i ≤ j upper triangle.
///
/// Output layout `[E, ∂/∂x_0, ..., ∂/∂x_{inlen-1}, ∂²/∂x_0∂x_0, ...]`
/// matches `XCFunctional.cpp:589-612` verbatim.
#[cfg(feature = "testing")]
fn launch_and_accumulate_order2_general(
    id_u32: u32,
    vars_u32: u32,
    inlen: usize,
    input: &[f64],
    weight: f64,
    output: &mut [f64],
) -> Result<(), XcError> {
    let sz = 4_usize;

    let mut last_out: Option<Vec<f64>> = None;
    let mut k = inlen + 1;
    let mut per_i_last_out: Vec<Option<Vec<f64>>> = vec![None; inlen];

    for i in 0..inlen {
        for j in i..inlen {
            let mut flat = vec![0.0_f64; inlen * sz];
            for kk in 0..inlen {
                flat[kk * sz] = input[kk];
            }
            // in[i][VAR0] = 1.
            flat[i * sz + 1] = 1.0;
            // in[j][VAR1] = 1.
            flat[j * sz + 2] = 1.0;

            let out = run_launch(id_u32, vars_u32, 2, &flat, sz)?;
            // VAR0|VAR1 = 3.
            output[k] += weight * out[3];
            k += 1;

            per_i_last_out[i] = Some(out.clone());
            last_out = Some(out);
        }
    }

    for (i, last_i) in per_i_last_out.iter().enumerate() {
        if let Some(out) = last_i {
            output[i + 1] += weight * out[1];
        }
    }
    if let Some(out) = last_out {
        output[0] += weight * out[0];
    }
    Ok(())
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

    // DensVarsDev scratch handles — 24 Array<F> fields, each of length (1 << n).
    // Phase 3 plan 03-01 B2: lapn + laps added after lapb (see density_vars.rs).
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
    let lapn_h = mk();
    let laps_h = mk();
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

    // Macro to compress the boilerplate of each launch arm. Each invocation
    // expands to a single `launch_eval_point::<ID, VARS, N>(...)` call with
    // the standard 24-handle DensVarsDev array.
    macro_rules! arm {
        ($id:literal, $vars:literal, $n:literal) => {
            launch_eval_point::<$id, $vars, $n>(
                client,
                in_h.clone(),
                &[
                    a_h.clone(), b_h.clone(), gaa_h.clone(), gab_h.clone(), gbb_h.clone(),
                    n_h.clone(), s_h.clone(), gnn_h.clone(), gns_h.clone(), gss_h.clone(),
                    tau_h.clone(), taua_h.clone(), taub_h.clone(), lapa_h.clone(),
                    lapb_h.clone(), lapn_h.clone(), laps_h.clone(), zeta_h.clone(),
                    rs_h.clone(), nm13_h.clone(), a43_h.clone(), b43_h.clone(),
                    jpaa_h.clone(), jpbb_h.clone(),
                ],
                out_h.clone(),
                flat_input.len(),
                arr_cnt,
                out_len,
            )
        };
    }
    #[allow(unsafe_code)]
    unsafe {
        // Dispatch on (id, vars, n) — cubecl monomorphises per comptime tuple.
        //
        // Phase 2 supports (id, vars=2, n) for 11 LDAs.
        // Phase 3 plan 03-03 absorbs the Wave-2 INCONCLUSIVE escalation by
        // adding (id, vars=6, n) arms for the 27 GGAs (17 Wave-2 + 10 Wave-3)
        // that consume Vars::A_B_GAA_GAB_GBB (inlen=5).
        match (id_u32, vars_u32, n) {
            // ===== Phase 2: 11 LDA ids × 3 orders, vars=2 (XC_A_B). =====
            (0,  2, 0) => arm!(0,  2, 0),  (0,  2, 1) => arm!(0,  2, 1),  (0,  2, 2) => arm!(0,  2, 2),
            (2,  2, 0) => arm!(2,  2, 0),  (2,  2, 1) => arm!(2,  2, 1),  (2,  2, 2) => arm!(2,  2, 2),
            (3,  2, 0) => arm!(3,  2, 0),  (3,  2, 1) => arm!(3,  2, 1),  (3,  2, 2) => arm!(3,  2, 2),
            (13, 2, 0) => arm!(13, 2, 0),  (13, 2, 1) => arm!(13, 2, 1),  (13, 2, 2) => arm!(13, 2, 2),
            (14, 2, 0) => arm!(14, 2, 0),  (14, 2, 1) => arm!(14, 2, 1),  (14, 2, 2) => arm!(14, 2, 2),
            (15, 2, 0) => arm!(15, 2, 0),  (15, 2, 1) => arm!(15, 2, 1),  (15, 2, 2) => arm!(15, 2, 2),
            (24, 2, 0) => arm!(24, 2, 0),  (24, 2, 1) => arm!(24, 2, 1),  (24, 2, 2) => arm!(24, 2, 2),
            (28, 2, 0) => arm!(28, 2, 0),  (28, 2, 1) => arm!(28, 2, 1),  (28, 2, 2) => arm!(28, 2, 2),
            (55, 2, 0) => arm!(55, 2, 0),  (55, 2, 1) => arm!(55, 2, 1),  (55, 2, 2) => arm!(55, 2, 2),

            // ===== Phase 3 Wave-2 GGAs: 17 ids × 3 orders, vars=6 (XC_A_B_GAA_GAB_GBB). =====
            // Plan 03-03 absorbs the Wave-2 INCONCLUSIVE escalation by wiring all 17 Wave-2
            // ids at vars=6 here.
            ( 4, 6, 0) => arm!( 4, 6, 0),  ( 4, 6, 1) => arm!( 4, 6, 1),  ( 4, 6, 2) => arm!( 4, 6, 2),
            ( 5, 6, 0) => arm!( 5, 6, 0),  ( 5, 6, 1) => arm!( 5, 6, 1),  ( 5, 6, 2) => arm!( 5, 6, 2),
            ( 6, 6, 0) => arm!( 6, 6, 0),  ( 6, 6, 1) => arm!( 6, 6, 1),  ( 6, 6, 2) => arm!( 6, 6, 2),
            ( 7, 6, 0) => arm!( 7, 6, 0),  ( 7, 6, 1) => arm!( 7, 6, 1),  ( 7, 6, 2) => arm!( 7, 6, 2),
            ( 8, 6, 0) => arm!( 8, 6, 0),  ( 8, 6, 1) => arm!( 8, 6, 1),  ( 8, 6, 2) => arm!( 8, 6, 2),
            ( 9, 6, 0) => arm!( 9, 6, 0),  ( 9, 6, 1) => arm!( 9, 6, 1),  ( 9, 6, 2) => arm!( 9, 6, 2),
            (16, 6, 0) => arm!(16, 6, 0),  (16, 6, 1) => arm!(16, 6, 1),  (16, 6, 2) => arm!(16, 6, 2),
            (19, 6, 0) => arm!(19, 6, 0),  (19, 6, 1) => arm!(19, 6, 1),  (19, 6, 2) => arm!(19, 6, 2),
            (20, 6, 0) => arm!(20, 6, 0),  (20, 6, 1) => arm!(20, 6, 1),  (20, 6, 2) => arm!(20, 6, 2),
            (21, 6, 0) => arm!(21, 6, 0),  (21, 6, 1) => arm!(21, 6, 1),  (21, 6, 2) => arm!(21, 6, 2),
            (22, 6, 0) => arm!(22, 6, 0),  (22, 6, 1) => arm!(22, 6, 1),  (22, 6, 2) => arm!(22, 6, 2),
            (69, 6, 0) => arm!(69, 6, 0),  (69, 6, 1) => arm!(69, 6, 1),  (69, 6, 2) => arm!(69, 6, 2),
            (71, 6, 0) => arm!(71, 6, 0),  (71, 6, 1) => arm!(71, 6, 1),  (71, 6, 2) => arm!(71, 6, 2),
            (72, 6, 0) => arm!(72, 6, 0),  (72, 6, 1) => arm!(72, 6, 1),  (72, 6, 2) => arm!(72, 6, 2),
            (73, 6, 0) => arm!(73, 6, 0),  (73, 6, 1) => arm!(73, 6, 1),  (73, 6, 2) => arm!(73, 6, 2),
            (74, 6, 0) => arm!(74, 6, 0),  (74, 6, 1) => arm!(74, 6, 1),  (74, 6, 2) => arm!(74, 6, 2),
            (76, 6, 0) => arm!(76, 6, 0),  (76, 6, 1) => arm!(76, 6, 1),  (76, 6, 2) => arm!(76, 6, 2),

            // ===== Phase 3 Wave-3 GGAs: 10 ids × 3 orders, vars=6. =====
            ( 1, 6, 0) => arm!( 1, 6, 0),  ( 1, 6, 1) => arm!( 1, 6, 1),  ( 1, 6, 2) => arm!( 1, 6, 2),
            (17, 6, 0) => arm!(17, 6, 0),  (17, 6, 1) => arm!(17, 6, 1),  (17, 6, 2) => arm!(17, 6, 2),
            (18, 6, 0) => arm!(18, 6, 0),  (18, 6, 1) => arm!(18, 6, 1),  (18, 6, 2) => arm!(18, 6, 2),
            (26, 6, 0) => arm!(26, 6, 0),  (26, 6, 1) => arm!(26, 6, 1),  (26, 6, 2) => arm!(26, 6, 2),
            (27, 6, 0) => arm!(27, 6, 0),  (27, 6, 1) => arm!(27, 6, 1),  (27, 6, 2) => arm!(27, 6, 2),
            (56, 6, 0) => arm!(56, 6, 0),  (56, 6, 1) => arm!(56, 6, 1),  (56, 6, 2) => arm!(56, 6, 2),
            (57, 6, 0) => arm!(57, 6, 0),  (57, 6, 1) => arm!(57, 6, 1),  (57, 6, 2) => arm!(57, 6, 2),
            (67, 6, 0) => arm!(67, 6, 0),  (67, 6, 1) => arm!(67, 6, 1),  (67, 6, 2) => arm!(67, 6, 2),
            (68, 6, 0) => arm!(68, 6, 0),  (68, 6, 1) => arm!(68, 6, 1),  (68, 6, 2) => arm!(68, 6, 2),
            (77, 6, 0) => arm!(77, 6, 0),  (77, 6, 1) => arm!(77, 6, 1),  (77, 6, 2) => arm!(77, 6, 2),

            // ===== Phase 3 Wave-4 GGAs: 8 ids × 3 orders, vars=6. =====
            // KTX (23), BTK (58), B97X (60), B97C (61), B97_1X (62), B97_1C (63),
            // B97_2X (64), B97_2C (65).
            (23, 6, 0) => arm!(23, 6, 0),  (23, 6, 1) => arm!(23, 6, 1),  (23, 6, 2) => arm!(23, 6, 2),
            (58, 6, 0) => arm!(58, 6, 0),  (58, 6, 1) => arm!(58, 6, 1),  (58, 6, 2) => arm!(58, 6, 2),
            (60, 6, 0) => arm!(60, 6, 0),  (60, 6, 1) => arm!(60, 6, 1),  (60, 6, 2) => arm!(60, 6, 2),
            (61, 6, 0) => arm!(61, 6, 0),  (61, 6, 1) => arm!(61, 6, 1),  (61, 6, 2) => arm!(61, 6, 2),
            (62, 6, 0) => arm!(62, 6, 0),  (62, 6, 1) => arm!(62, 6, 1),  (62, 6, 2) => arm!(62, 6, 2),
            (63, 6, 0) => arm!(63, 6, 0),  (63, 6, 1) => arm!(63, 6, 1),  (63, 6, 2) => arm!(63, 6, 2),
            (64, 6, 0) => arm!(64, 6, 0),  (64, 6, 1) => arm!(64, 6, 1),  (64, 6, 2) => arm!(64, 6, 2),
            (65, 6, 0) => arm!(65, 6, 0),  (65, 6, 1) => arm!(65, 6, 1),  (65, 6, 2) => arm!(65, 6, 2),

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
    densvar_handles: &[cubecl::server::Handle; 24],
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
                ArrayArg::from_raw_parts(densvar_handles[22].clone(), arr_cnt),
                ArrayArg::from_raw_parts(densvar_handles[23].clone(), arr_cnt),
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
            parameters: DEFAULT_PARAMETERS,
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
            parameters: DEFAULT_PARAMETERS,
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
            parameters: DEFAULT_PARAMETERS,
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
            parameters: DEFAULT_PARAMETERS,
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
            parameters: DEFAULT_PARAMETERS,
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
            parameters: DEFAULT_PARAMETERS,
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
            parameters: DEFAULT_PARAMETERS,
        };
        let mut out = vec![99.0_f64; 1];
        assert!(f.eval(&[1.0, 0.5], &mut out).is_ok());
        assert_eq!(out[0], 0.0, "eval must zero-fill the output on success");
    }

    // -----------------------------------------------------------------------
    //  Plan 03-01 Task 3 tests — `output_length` + `eval_setup` rejection paths
    //  for Mode::Potential (D-13 + D-15 + D-25).
    // -----------------------------------------------------------------------

    #[test]
    fn output_length_potential_nspin1() {
        // D-15: single-spin Mode::Potential returns 2.
        assert_eq!(
            Functional::output_length(Vars::A, Mode::Potential, 0).unwrap(),
            2
        );
        assert_eq!(
            Functional::output_length(Vars::A_2ND_TAYLOR, Mode::Potential, 0).unwrap(),
            2
        );
    }

    #[test]
    fn output_length_potential_nspin2() {
        // D-15: spin-resolved Mode::Potential returns 3.
        assert_eq!(
            Functional::output_length(Vars::A_B, Mode::Potential, 0).unwrap(),
            3
        );
        assert_eq!(
            Functional::output_length(Vars::N_S, Mode::Potential, 0).unwrap(),
            3
        );
        assert_eq!(
            Functional::output_length(Vars::N_S_2ND_TAYLOR, Mode::Potential, 0).unwrap(),
            3
        );
        assert_eq!(
            Functional::output_length(Vars::A_B_2ND_TAYLOR, Mode::Potential, 0).unwrap(),
            3
        );
    }

    #[test]
    fn output_length_partial_derivatives_matches_taylorlen() {
        // Sanity: PartialDerivatives branch unchanged from Phase 2.
        assert_eq!(
            Functional::output_length(Vars::A_B, Mode::PartialDerivatives, 0).unwrap(),
            taylorlen(2, 0)
        );
        assert_eq!(
            Functional::output_length(Vars::A_B, Mode::PartialDerivatives, 2).unwrap(),
            taylorlen(2, 2)
        );
    }

    #[test]
    fn eval_setup_rejects_metagga_potential() {
        // M05X carries Dependency::KINETIC — must reject Mode::Potential (D-13).
        let f = Functional {
            weights: &[(FunctionalId::XC_M05X, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        assert!(matches!(
            f.eval_setup(Vars::A_B_2ND_TAYLOR, Mode::Potential, 0),
            Err(XcError::InvalidMode { .. })
        ));
    }

    #[test]
    fn eval_setup_rejects_laplacian_potential() {
        // BRX carries Dependency::LAPLACIAN — must reject Mode::Potential.
        let f = Functional {
            weights: &[(FunctionalId::XC_BRX, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        assert!(matches!(
            f.eval_setup(Vars::A_B_2ND_TAYLOR, Mode::Potential, 0),
            Err(XcError::InvalidMode { .. })
        ));
    }

    #[test]
    fn eval_setup_rejects_gga_non_2nd_taylor_potential() {
        // PBEX carries GRADIENT only. For Mode::Potential we must require one of
        // the _2ND_TAYLOR Vars arms; using XC_A_B_GAA_GAB_GBB must reject.
        let f = Functional {
            weights: &[(FunctionalId::XC_PBEX, 1.0)],
            vars: Vars::A_B_GAA_GAB_GBB,
            mode: Mode::Potential,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        assert!(matches!(
            f.eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::Potential, 0),
            Err(XcError::InvalidVars { .. })
        ));
    }

    #[test]
    fn eval_setup_accepts_gga_with_2nd_taylor_potential() {
        // PBEX + A_B_2ND_TAYLOR is the valid combination — must pass.
        let f = Functional {
            weights: &[(FunctionalId::XC_PBEX, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        assert!(f
            .eval_setup(Vars::A_B_2ND_TAYLOR, Mode::Potential, 0)
            .is_ok());
    }

    #[test]
    fn eval_setup_accepts_lda_with_any_vars_potential() {
        // SLATERX is DENSITY only — any non-metaGGA Vars should pass Mode::Potential.
        let f = Functional {
            weights: &[(FunctionalId::XC_SLATERX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Potential,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        assert!(f.eval_setup(Vars::A_B, Mode::Potential, 0).is_ok());
    }

    #[test]
    fn eval_setup_rejects_unset_mode() {
        let f = Functional {
            weights: &[(FunctionalId::XC_SLATERX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Unset,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        assert!(matches!(
            f.eval_setup(Vars::A_B, Mode::Unset, 0),
            Err(XcError::NotConfigured)
        ));
    }

    #[test]
    fn dependencies_aggregates_across_weights() {
        // PBEX (GRADIENT) + SLATERX (DENSITY) — combined deps = DENSITY | GRADIENT.
        let f = Functional {
            weights: &[
                (FunctionalId::XC_SLATERX, 0.5),
                (FunctionalId::XC_PBEX, 0.5),
            ],
            vars: Vars::A_B_GAA_GAB_GBB,
            mode: Mode::PartialDerivatives,
            order: 0,
            parameters: DEFAULT_PARAMETERS,
        };
        let deps = f.dependencies();
        assert!(deps.contains(Dependency::DENSITY));
        assert!(deps.contains(Dependency::GRADIENT));
        assert!(!deps.contains(Dependency::KINETIC));
        assert!(!deps.contains(Dependency::LAPLACIAN));
    }
}
