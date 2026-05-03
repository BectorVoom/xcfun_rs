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
use xcfun_core::{
    ALIASES, Dependency, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, ParameterId, Vars, XcError,
    taylorlen,
};

// Phase 6 Plan 06-01 (D-08): kernel bodies + DensVarsDev + dispatch_kernel
// migrated to `xcfun-kernels`. Host-side `Functional::eval` keeps the same
// shape — only the import paths move.
use xcfun_kernels::density_vars::DensVarsDev;
use xcfun_kernels::density_vars::build::build_densvars;
use xcfun_kernels::density_vars::DensVarsDevLaunch;
use xcfun_kernels::dispatch;
use xcfun_kernels::dispatch::dispatch_kernel;

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

/// A weighted sum of functionals plus the full xcfun settings array.
///
/// Phase 2 minimal slice (D-21) extended in Phase 4 D-04/D-05 with the
/// 82-slot `settings: [f64; 82]` mirroring
/// `xcfun-master/src/XCFunctional.hpp:35` (`std::array<double, XC_NR_PARAMETERS_AND_FUNCTIONALS> settings`)
/// and the alias-resolution engine on top of it.
///
/// Layout of `settings`:
///   - indices 0..=77: per-`FunctionalId` weight; entries default to 0.0.
///     Updated *additively* by `set("functional_name", value)` per
///     `XCFunctional.cpp:373`.
///   - indices 78..=81: per-`ParameterId` value; seeded with the defaults
///     from `common_parameters.cpp:17-29`. Updated *destructively* by
///     `set("parameter_name", value)` per `XCFunctional.cpp:381`.
///
/// `weights` was Phase 2's static-slice form for the existing `eval`
/// launch loop, refactored to `Vec<(FunctionalId, f64)>` in Phase 6 Plan
/// 06-06 (D-17) to drop the Phase 5 `Box::leak` once-per-`set`.  `set/get`
/// operate on `settings` only — wiring the `set`-built state into
/// `weights` happens in `xcfun-rs::Functional::sync_weights_from_settings`.
pub struct Functional {
    /// (FunctionalId, weight) pairs. Weights sum to the active-functional set.
    ///
    /// Phase 6 Plan 06-06 (D-17): changed from `&'static [(FunctionalId, f64)]`
    /// to `Vec<(FunctionalId, f64)>` to drop the Phase 5 `Box::leak`-per-`set`
    /// in `xcfun-rs::Functional::sync_weights_from_settings`. `Vec<...>` is
    /// `Send + Sync` so RS-10 (`assert_impl_all!(Functional: Send, Sync)`) is
    /// preserved. All read-sites use `weights.iter()` / indexing which work
    /// unchanged via `Deref<Target = [_]>`.
    pub weights: Vec<(FunctionalId, f64)>,
    /// Input variable layout. Must match the actual `input.len()`.
    pub vars: Vars,
    /// Evaluation mode. Phase 2 supports `Mode::PartialDerivatives` only.
    pub mode: Mode,
    /// Derivative order. Phase 2 supported 0..=2 per D-23; Plan 03-06 Task 1
    /// extends to 0..=4 per MODE-01 D-16.
    pub order: u32,
    /// 82-slot xcfun-style settings array — the canonical state mutated by
    /// `Functional::set` and read by `Functional::get`. Plan 04-04 D-05.
    ///
    /// Indices 0..=77 are functional weights (FunctionalId discriminants).
    /// Indices 78..=81 are parameters (ParameterId discriminants), seeded
    /// with their defaults by `Functional::new()`.
    pub settings: [f64; 82],
    /// Monotonic generation counter bumped on every successful `set()` call.
    /// Phase 6 Plan 06-02a (D-15): consumed by `xcfun_gpu::Batch::launch` to
    /// decide whether the cached `weights_buf` is stale and needs re-upload.
    /// Hash-based comparison rejected (D-15) — 82 f64 = 656 bytes, simpler to
    /// track a counter than to hash on every launch.
    ///
    /// `wrapping_add(1)` is fine because (a) Plan 06-02a tests only assert
    /// strict inequality across a small number of `set()` calls, and (b) at
    /// the rate of 1 bump per `set` call, wrap-around requires ~1.8 × 10¹⁹
    /// settings updates — irrelevant for any realistic workload.
    pub settings_gen: u64,
}

/// Default `settings` array seeded by `Functional::new()` per
/// `XCFunctional.cpp:351-354`. Functional slots 0..=77 are zero; parameter
/// slots 78..=81 carry their defaults from `common_parameters.cpp:17-29`.
///
/// Replaces the Phase 3 `DEFAULT_PARAMETERS: [f64; 4]` constant; downstream
/// callers must update field name from `parameters` to `settings`.
pub const DEFAULT_SETTINGS: [f64; 82] = {
    let mut s = [0.0_f64; 82];
    s[ParameterId::XC_RANGESEP_MU as usize] = 0.4;
    s[ParameterId::XC_EXX as usize] = 0.0;
    s[ParameterId::XC_CAM_ALPHA as usize] = 0.19;
    s[ParameterId::XC_CAM_BETA as usize] = 0.46;
    s
};

impl Functional {
    /// Construct a fresh `Functional` with empty weights, `Mode::Unset`, and
    /// `settings` seeded by `DEFAULT_SETTINGS` (zero functional slots,
    /// parameter slots at their `common_parameters.cpp:17-29` defaults).
    ///
    /// Companion `Default` impl below delegates to `Self::new()`.
    ///
    /// Mirrors `XCFunctional::XCFunctional()` at `XCFunctional.cpp:350-355`:
    /// ```cpp
    /// for (int i = 0; i < XC_NR_FUNCTIONALS; ++i) settings[i] = 0;
    /// for (int i = XC_NR_FUNCTIONALS; i < XC_NR_PARAMETERS_AND_FUNCTIONALS; ++i)
    ///     settings[i] = xcint_params[i].default_value;
    /// ```
    pub const fn new() -> Self {
        Self {
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::Unset,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        }
    }

    /// Update `self.settings` for `name`. Three-case dispatch mirroring
    /// `xcfun_set` at `xcfun-master/src/XCFunctional.cpp:369-405`:
    ///
    /// 1. Functional name (`FunctionalId::from_name`):
    ///    `settings[id] += value`            (additive accumulation)
    /// 2. Parameter name (`ParameterId::from_name`):
    ///    `settings[id]  = value`            (overwrite)
    /// 3. Alias name (case-insensitive match in `ALIASES`):
    ///    for each `(term_name, weight)` recurse
    ///    `set(term_name, value * weight)`.
    ///    Per the C++ FIXME at L393 the multiplication by `value` applies
    ///    even to parameter terms (`exx`, `cam_alpha`, `cam_beta`,
    ///    `rangesep_mu`); this preserves bit-level parity with the C++
    ///    reference and is REQUIRED by the 1e-12 contract.
    ///
    /// Returns `Err(XcError::UnknownName)` when `name` matches no entry in
    /// any of the three tables (mirrors C++'s `return -1`).
    ///
    /// **Lookup priority** (functional → parameter → alias) follows the
    /// C++ ordering. As a consequence, names that are simultaneously a
    /// functional and an alias (e.g. `OPTX`, `PBEX`) route to the
    /// functional case. Names like `EXX` route to the parameter case
    /// before any alias check.
    ///
    /// **Recursion bound:** the static `ALIASES` table never refers to
    /// another alias as a term (verified across all 46 entries by the
    /// `aliases_all_terms_resolve_to_known_names` test). Maximum recursion
    /// depth is therefore 1 — no explicit depth counter is needed.
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        // Case 1 — Functional name (XCFunctional.cpp:372-385).
        if let Some(id) = FunctionalId::from_name(name) {
            self.settings[id as usize] += value;
            // Phase 6 Plan 06-02a (D-15): bump generation counter so a
            // subsequent xcfun_gpu::Batch::launch re-uploads weights_buf.
            self.settings_gen = self.settings_gen.wrapping_add(1);
            return Ok(());
        }
        // Case 2 — Parameter name (XCFunctional.cpp:386-388).
        if let Some(pid) = ParameterId::from_name(name) {
            self.settings[pid as usize] = value;
            self.settings_gen = self.settings_gen.wrapping_add(1);
            return Ok(());
        }
        // Case 3 — Alias name (XCFunctional.cpp:389-401).
        //
        // The recursive `self.set(...)` calls below each bump
        // `settings_gen` once per resolved term (one bump per `settings[]`
        // mutation). That matches the contract — every settings mutation
        // is observable downstream. We do NOT add an extra bump for the
        // alias call itself.
        if let Some(alias) = ALIASES
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(name))
        {
            for (term_name, term_weight) in alias.components.iter() {
                // Recurse — multiply by `value` per L397 (FIXME preserved).
                self.set(term_name, value * *term_weight)?;
            }
            return Ok(());
        }
        Err(XcError::UnknownName)
    }

    /// Phase 6 Plan 06-02a (D-15) — monotonic generation counter accessor.
    /// `xcfun_gpu::Batch::launch` reads this to decide whether the cached
    /// `weights_buf` upload is stale.
    #[inline]
    pub fn settings_generation(&self) -> u64 {
        self.settings_gen
    }

    /// Read a `settings[]` slot by name. Two-case dispatch mirroring
    /// `xcfun_get` at `xcfun-master/src/XCFunctional.cpp:407-419`.
    /// Aliases are NOT readable through `get` (the C++ implementation has
    /// no alias case; it returns -1 for any non-functional / non-parameter
    /// name).
    pub fn get(&self, name: &str) -> Result<f64, XcError> {
        if let Some(id) = FunctionalId::from_name(name) {
            return Ok(self.settings[id as usize]);
        }
        if let Some(pid) = ParameterId::from_name(name) {
            return Ok(self.settings[pid as usize]);
        }
        Err(XcError::UnknownName)
    }

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
    ///
    /// # Phase 3 plan 03-05 — Mode::Potential routing (D-13)
    ///
    /// `Mode::Potential` is now routed to `launch_potential` (line-for-line
    /// port of `XCFunctional.cpp:637-790`).
    ///
    /// # Phase 4 plan 04-05 — Mode::Contracted routing (D-06)
    ///
    /// `Mode::Contracted` is now routed to
    /// `crate::functionals::contracted::launch_contracted` — line-for-line
    /// port of `XCFunctional.cpp:619-635` `DOEVAL` macro across orders
    /// 0..=6 (D-06 + D-06-A). Input layout: `inlen × (1 << order)` flat
    /// f64 doubles; output layout: `(1 << order)` flat doubles per D-06-B.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        // 1. Validate mode.  Phase 4 plan 04-05 wires Mode::Contracted.
        match self.mode {
            Mode::Unset => return Err(XcError::NotConfigured),
            Mode::PartialDerivatives | Mode::Potential | Mode::Contracted => {}
        }
        // 2. Validate input length. Mode::Contracted at order N reads
        //    `inlen × (1 << order)` flat f64 doubles per D-06-A
        //    (XCFunctional.cpp:622-627). All other modes read `inlen` scalars.
        let expected_inlen = Self::input_length(self.vars);
        let expected_input_buf_len = match self.mode {
            Mode::Contracted => expected_inlen * (1_usize << self.order),
            _ => expected_inlen,
        };
        if input.len() != expected_input_buf_len {
            return Err(XcError::InputLengthMismatch {
                expected: expected_input_buf_len,
                got: input.len(),
            });
        }
        // 3. Validate order. Phase 3 plan 03-06 extends PartialDerivatives to
        //    0..=4 per MODE-01 D-16. Mode::Contracted accepts orders 0..=6
        //    per D-06 (XCFUN_MAX_ORDER = 6). Mode::Potential uses order 0
        //    by convention — the LDA loop runs at N=1, GGA at N=2.
        if self.mode == Mode::PartialDerivatives && self.order > 4 {
            return Err(XcError::InvalidOrder {
                order: self.order,
                mode: self.mode,
                n_vars: expected_inlen,
            });
        }
        if self.mode == Mode::Contracted && self.order > 6 {
            return Err(XcError::InvalidOrder {
                order: self.order,
                mode: self.mode,
                n_vars: expected_inlen,
            });
        }
        // 4. Validate output length per MODE-04 + RESEARCH + D-06-B.
        let expected_outlen = match self.mode {
            Mode::PartialDerivatives => taylorlen(expected_inlen, self.order as usize),
            Mode::Potential => Self::output_length(self.vars, self.mode, self.order)?,
            Mode::Contracted => Self::output_length(self.vars, self.mode, self.order)?,
            _ => unreachable!(),
        };
        if output.len() != expected_outlen {
            return Err(XcError::OutputLengthMismatch {
                expected: expected_outlen,
                got: output.len(),
            });
        }
        // 5. Validate every functional in `weights` is supported.
        // Plan 06-06 D-17: weights is now Vec<...>; iterate by reference.
        for (id, _w) in self.weights.iter() {
            if !dispatch::supports(*id) {
                return Err(XcError::NotConfigured);
            }
        }
        // 5b. Mode::Potential — defense-in-depth host gate (eval_setup re-run).
        if self.mode == Mode::Potential {
            self.eval_setup(self.vars, self.mode, self.order)?;
        }

        output.fill(0.0);

        // 6. Per-(FunctionalId, weight) launch loop.
        #[cfg(feature = "testing")]
        {
            match self.mode {
                Mode::PartialDerivatives => {
                    for &(id, weight) in self.weights.iter() {
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
                Mode::Potential => {
                    // D-13: line-for-line XCFunctional.cpp:637-790.
                    self.launch_potential(input, output)?;
                }
                Mode::Contracted => {
                    // D-06: line-for-line XCFunctional.cpp:619-635 DOEVAL.
                    crate::functionals::contracted::launch_contracted(
                        self, input, output,
                    )?;
                }
                Mode::Unset => unreachable!(),
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
    /// `Mode::Contracted` returns `1 << order` per D-06-B (Plan 04-05).
    /// `Mode::Unset` is rejected (`XcError::NotConfigured`).
    ///
    /// **D-06-B divergence from C++:** the C++ `xcfun_output_length`
    /// (`XCFunctional.cpp:488`) calls `xcfun::die("XC_CONTRACTED not implemented
    /// in xc_output_length()", 0)` — i.e., refuses to compute a Contracted
    /// output length. Rust takes the opposite stance and returns `1 << order`
    /// directly per the DOEVAL output-write loop at `XCFunctional.cpp:627-628`.
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
            Mode::Contracted => {
                // D-06-B: 1 << order for orders 0..=6 (XCFUN_MAX_ORDER).
                if order > 6 {
                    return Err(XcError::InvalidOrder {
                        order,
                        mode,
                        n_vars: Self::input_length(vars),
                    });
                }
                Ok(1_usize << order)
            }
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
    /// - `Mode::Contracted` + `order > 6` → `InvalidOrder` (Plan 04-05 D-06,
    ///   XCFUN_MAX_ORDER = 6); per D-06-A no Vars-specific rejection
    ///   (the DOEVAL macro at XCFunctional.cpp:619-635 contains no Vars guard).
    ///
    /// D-25 resolution: no new `XcError` variants; reuses `InvalidMode` +
    /// `InvalidVars` + `InvalidOrder` already present since Phase 2.
    pub fn eval_setup(
        &self,
        vars: Vars,
        mode: Mode,
        order: u32,
    ) -> Result<(), XcError> {
        let deps = self.dependencies();
        match mode {
            Mode::Unset => Err(XcError::NotConfigured),
            Mode::Contracted => {
                // Plan 04-05 D-06: accept any order in 0..=6 (XCFUN_MAX_ORDER).
                // Per D-06-A no Vars-specific rejection beyond the existing
                // depends-vs-vars check (the DOEVAL macro itself has no Vars guard).
                if order > 6 {
                    return Err(XcError::InvalidOrder {
                        order,
                        mode,
                        n_vars: Self::input_length(vars),
                    });
                }
                Ok(())
            }
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
                            // D-08-A — XCFunctional.cpp:441-443 returns
                            // XC_EVARS | XC_EMODE (= 6) for this combined case.
                            return Err(XcError::InvalidVarsAndMode {
                                vars,
                                mode,
                                depends: deps,
                            });
                        }
                    }
                }
                Ok(())
            }
        }
    }

    // -----------------------------------------------------------------------
    //  Mode::Potential host-side launchers (Phase 3 plan 03-05).
    //  Line-for-line port of `xcfun-master/src/XCFunctional.cpp:637-790`
    //  per D-13.
    //
    //  Two-pass structure (XCFunctional.cpp:671 — no `else` between the LDA
    //  block and the `if (fun->depends & XC_GRADIENT)` GGA block):
    //    Pass 1 (always)   — `launch_potential_lda` populates
    //                         `out[0] = energy`,
    //                         `out[j+1] = ∂E/∂ρ_{α/β}` (LDA-direct term).
    //    Pass 2 (GGA only) — `launch_potential_gga` subtracts
    //                         `Σ_dir Σ_id w_id · out[VAR0|VAR1]`
    //                         (= ∇·(∂E/∂g)) IN PLACE from `out[j+1]`.
    // -----------------------------------------------------------------------

    /// Mode::Potential entry point.  Routes the active functional set to the
    /// LDA + (optional) GGA divergence loops per `XCFunctional.cpp:637-790`.
    /// Defense-in-depth: rejects metaGGA-class deps even though `eval_setup`
    /// already gated them.
    #[cfg(feature = "testing")]
    pub fn launch_potential(&self, input: &[f64], out: &mut [f64]) -> Result<(), XcError> {
        let deps = self.dependencies();

        if deps.contains(Dependency::LAPLACIAN) || deps.contains(Dependency::KINETIC) {
            return Err(XcError::InvalidMode {
                mode: self.mode,
                depends: deps,
            });
        }

        // Pass 1 (XCFunctional.cpp:653-670): ALWAYS run the LDA N=1 loop
        // first.  Populates out[0] = energy, out[j+1] = ∂E/∂ρ (LDA-direct).
        self.launch_potential_lda(input, out)?;

        // Pass 2 (XCFunctional.cpp:671-791): if GRADIENT, subtract divergence
        // IN PLACE from out[j+1] (XCFunctional.cpp:720 / :785-787).
        if deps.contains(Dependency::GRADIENT) {
            self.launch_potential_gga(input, out)?;
        }

        Ok(())
    }

    /// Stub for non-testing builds (mirrors the `eval` non-testing guard).
    #[cfg(not(feature = "testing"))]
    pub fn launch_potential(&self, _input: &[f64], _out: &mut [f64]) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }

    /// Port of `XCFunctional.cpp:637-670` — LDA path at N=1.  ALWAYS runs
    /// (even for GGA functionals) to populate the LDA-direct potential term
    /// before the GGA block subtracts divergence.
    ///
    /// The C++ block:
    /// ```cpp
    /// int inlen = xcint_vars[fun->vars].len;
    /// int npot, inpos = 0;
    /// if (inlen == 1 || inlen == 10) npot = 1;
    /// else { npot = 2; if (inlen == 2) inpos = 1; else if (inlen == 20) inpos = 10; }
    /// typedef ctaylor<ireal_t, 1> ttype;
    /// ttype in[XC_MAX_INVARS], out = 0;
    /// for (int i = 0; i < inlen; i++) in[i] = input[i];
    /// for (int j = 0; j < npot; j++) {
    ///   in[j*inpos].set(VAR0, 1);
    ///   densvars<ttype> d(fun, in);
    ///   out = 0;
    ///   for (int i = 0; i < fun->nr_active_functionals; i++)
    ///     out += fun->settings[...] * fun->active_functionals[i]->fp1(d);
    ///   in[j*inpos] = input[j*inpos];        // reset seed
    ///   output[j+1] = out.get(VAR0);
    /// }
    /// output[0] = out.get(CNST);
    /// ```
    #[cfg(feature = "testing")]
    fn launch_potential_lda(&self, input: &[f64], out: &mut [f64]) -> Result<(), XcError> {
        // Mirrors XCFunctional.cpp:639-652.
        let inlen = Self::input_length(self.vars);
        let (npot, inpos) = match inlen {
            1 | 10 => (1_usize, 0_usize), // nspin = 1
            2 => (2, 1),
            20 => (2, 10),
            _ => {
                return Err(XcError::InvalidVars {
                    vars: self.vars,
                    required: Dependency::DENSITY,
                });
            }
        };

        // CTaylor<f64, 1> size = 1 << 1 = 2 coefficients per slot (CNST, VAR0).
        const SIZE_N1: usize = 2;
        let mut ct_in = vec![0.0_f64; inlen * SIZE_N1];
        let mut energy_accum = 0.0_f64;

        for j in 0..npot {
            // Re-pack the flat ct_in: every slot's CNST = input[l]; VAR0 = 0.
            for l in 0..inlen {
                ct_in[l * SIZE_N1] = input[l];
                ct_in[l * SIZE_N1 + 1] = 0.0;
            }
            // Seed VAR0 = 1 on density slot j*inpos (XCFunctional.cpp:659).
            ct_in[(j * inpos) * SIZE_N1 + 1] = 1.0;

            // Launch potential_lda_kernel for each active (id, weight) and
            // accumulate weighted out[CNST] (energy) + out[VAR0] (potential).
            let mut weighted_energy = 0.0_f64;
            let mut weighted_pot = 0.0_f64;
            let mut kernel_out = vec![0.0_f64; SIZE_N1];
            for &(id, w) in self.weights.iter() {
                self.launch_potential_kernel_n1(id as u32, &ct_in, &mut kernel_out)?;
                weighted_energy += w * kernel_out[0];
                weighted_pot += w * kernel_out[1];
            }

            // Output slot j+1 receives the LDA-direct potential
            // (XCFunctional.cpp:666). For GGA functionals,
            // launch_potential_gga subtracts divergence from this slot.
            out[j + 1] = weighted_pot;
            // Energy is the same across j for LDA (XCFunctional.cpp:669
            // takes it from the LAST out.get(CNST) — same value).
            energy_accum = weighted_energy;
        }

        out[0] = energy_accum;
        Ok(())
    }

    /// Port of `XCFunctional.cpp:671-791` — GGA path at N=2.  Subtracts the
    /// divergence `∇·(∂E/∂g)` IN PLACE from the LDA-direct potential term
    /// already written to `out[j+1]` by `launch_potential_lda`.
    ///
    /// XCFunctional.cpp:671 structural invariant: this fn does NOT
    /// re-compute the LDA-direct term — it only subtracts the divergence.
    #[cfg(feature = "testing")]
    fn launch_potential_gga(&self, input: &[f64], out: &mut [f64]) -> Result<(), XcError> {
        // CTaylor<f64, 2> size = 1 << 2 = 4 coefficients per slot.
        const SIZE_N2: usize = 4;

        // Per-direction Hessian-slot table — direct transcription of the
        // C++ assignments at XCFunctional.cpp:683-713 (single-spin) and
        // :736-784 (spin-resolved):
        //
        //   For the single-spin block (n gx gy gz xx xy xz yy yz zz):
        //     d/dx: in[0].VAR0 = input[1] (gx)
        //           in[1].VAR0 = input[4] (xx)   src=1 → 4
        //           in[2].VAR0 = input[5] (xy)   src=2 → 5
        //           in[3].VAR0 = input[6] (xz)   src=3 → 6
        //     d/dy: in[0].VAR0 = input[2] (gy)
        //           in[1].VAR0 = input[5] (xy)   src=1 → 5
        //           in[2].VAR0 = input[7] (yy)   src=2 → 7
        //           in[3].VAR0 = input[8] (yz)   src=3 → 8
        //     d/dz: in[0].VAR0 = input[3] (gz)
        //           in[1].VAR0 = input[6] (xz)   src=1 → 6
        //           in[2].VAR0 = input[8] (yz)   src=2 → 8
        //           in[3].VAR0 = input[9] (zz)   src=3 → 9
        //
        // HESS_SLOT[src - 1][dir] gives the input slot index for the
        // VAR0 coefficient of `in[src]` along direction dir ∈ {0, 1, 2}.
        const HESS_SLOT: [[usize; 3]; 3] = [
            // src=1 (gx): x → xx(4), y → xy(5), z → xz(6)
            [4, 5, 6],
            // src=2 (gy): x → xy(5), y → yy(7), z → yz(8)
            [5, 7, 8],
            // src=3 (gz): x → xz(6), y → yz(8), z → zz(9)
            [6, 8, 9],
        ];

        let inlen = Self::input_length(self.vars);
        let nspin = match inlen {
            10 => 1_usize,
            20 => 2,
            _ => {
                return Err(XcError::InvalidVars {
                    vars: self.vars,
                    required: Dependency::GRADIENT,
                });
            }
        };

        // Flat CTaylor<f64, 2> block: inlen slots × 4 coefficients each.
        let mut ct_in = vec![0.0_f64; inlen * SIZE_N2];

        for j in 0..nspin {
            let offset = if nspin == 2 { 10_usize } else { 0 };
            let active_offset = offset * j; // 0 for α, 10 for β

            // Per-j divergence accumulator — Σ_dir Σ_id w_id · out[VAR0|VAR1].
            // Mirrors the C++ accumulation `out += ... fp2(d)` over 3
            // direction blocks, then `output[j+1] -= out.get(VAR0|VAR1)`.
            let mut divergence_accum = 0.0_f64;

            for dir in 0..3_usize {
                // Zero ct_in completely; per XCFunctional.cpp:686-687/744-745
                // slots 4..9 (and β-side 14..19) are explicitly zeroed.
                for slot in 0..(inlen * SIZE_N2) {
                    ct_in[slot] = 0.0;
                }

                // Populate spin channels (always BOTH for spin-resolved —
                // only the VAR1=1 seed picks which channel the divergence
                // belongs to).
                let spin_offsets: &[usize] = if nspin == 2 { &[0, 10] } else { &[0] };
                for &off in spin_offsets {
                    // in[0 + off].CNST = input[0 + off] (density)
                    ct_in[(0 + off) * SIZE_N2] = input[off];
                    // in[0 + off].VAR0 = input[(1 + dir) + off] (1st-order density gradient)
                    ct_in[(0 + off) * SIZE_N2 + 1] = input[(1 + dir) + off];

                    // in[src + off] for src = 1..=3 (gx/gy/gz):
                    //   CNST = input[src + off]
                    //   VAR0 = input[HESS_SLOT[src-1][dir] + off]
                    for src in 1_usize..=3 {
                        ct_in[(src + off) * SIZE_N2] = input[src + off];
                        ct_in[(src + off) * SIZE_N2 + 1] =
                            input[HESS_SLOT[src - 1][dir] + off];
                    }
                    // Slots 4..9 (and 14..19) on this spin remain zero per
                    // XCFunctional.cpp:686-687 / :744-745
                    // (`for (int i = 4; i < 10; i++) in[i] = 0;`).
                    // verify bit-for-bit at integration test (Task 3)
                    // that A_B_2ND_TAYLOR parity holds for the β channel.
                }

                // Seed VAR1 = 1 on the gradient-direction slot
                // (XCFunctional.cpp:688/701/713 + :746/762/778):
                //   var1_slot = (1 + dir) + active_offset
                let var1_slot = (1 + dir) + active_offset;
                ct_in[var1_slot * SIZE_N2 + 2 /* VAR1 */] = 1.0;

                // Launch potential_gga_kernel for each active (id, weight)
                // and accumulate weighted out[VAR0|VAR1] (slot 3 = 0b11).
                let mut kernel_out = vec![0.0_f64; SIZE_N2];
                for &(id, w) in self.weights.iter() {
                    self.launch_potential_kernel_n2(id as u32, &ct_in, &mut kernel_out)?;
                    divergence_accum += w * kernel_out[3];
                }
            }

            // XCFunctional.cpp:720 (single-spin) / :785-787 (spin-resolved):
            //   output[j + 1] -= out.get(VAR0 | VAR1);
            //
            // out[j+1] was populated by launch_potential_lda with the
            // LDA-direct ∂E/∂ρ term; here we subtract the accumulated
            // divergence in place.
            out[j + 1] -= divergence_accum;
        }

        Ok(())
    }

    /// Stub for non-testing builds.
    #[cfg(not(feature = "testing"))]
    fn launch_potential_lda(&self, _input: &[f64], _out: &mut [f64]) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }

    /// Stub for non-testing builds.
    #[cfg(not(feature = "testing"))]
    fn launch_potential_gga(&self, _input: &[f64], _out: &mut [f64]) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }

    /// Build flat ct_in + launch the per-functional kernel at N=1.
    ///
    /// Body delegates to the existing `run_launch` infrastructure at
    /// `crates/xcfun-eval/src/functional.rs:288-484` specialised to N=1
    /// (out_len = 2 = 1 << 1).  The `(id, vars, n=1)` arms in `run_launch`'s
    /// match are extended in this plan to cover Mode::Potential dispatch.
    #[cfg(feature = "testing")]
    fn launch_potential_kernel_n1(
        &self,
        id: u32,
        ct_in: &[f64],
        kernel_out: &mut [f64],
    ) -> Result<(), XcError> {
        const OUT_LEN_N1: usize = 2; // 1 << 1
        let out_vec = run_launch(id, self.vars as u32, 1, ct_in, OUT_LEN_N1)?;
        debug_assert_eq!(out_vec.len(), OUT_LEN_N1);
        kernel_out.copy_from_slice(&out_vec);
        Ok(())
    }

    /// Stub for non-testing builds.
    #[cfg(not(feature = "testing"))]
    fn launch_potential_kernel_n1(
        &self,
        _id: u32,
        _ct_in: &[f64],
        _kernel_out: &mut [f64],
    ) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }

    /// Build flat ct_in + launch the per-functional kernel at N=2.
    ///
    /// Mirrors `launch_potential_kernel_n1` but at N=2 (out_len = 4).
    /// The `(id, vars=27..30, n=2)` arms in `run_launch` are extended in
    /// this plan to cover the GGA divergence path.
    #[cfg(feature = "testing")]
    fn launch_potential_kernel_n2(
        &self,
        id: u32,
        ct_in: &[f64],
        kernel_out: &mut [f64],
    ) -> Result<(), XcError> {
        const OUT_LEN_N2: usize = 4; // 1 << 2
        let out_vec = run_launch(id, self.vars as u32, 2, ct_in, OUT_LEN_N2)?;
        debug_assert_eq!(out_vec.len(), OUT_LEN_N2);
        kernel_out.copy_from_slice(&out_vec);
        Ok(())
    }

    /// Stub for non-testing builds.
    #[cfg(not(feature = "testing"))]
    fn launch_potential_kernel_n2(
        &self,
        _id: u32,
        _ct_in: &[f64],
        _kernel_out: &mut [f64],
    ) -> Result<(), XcError> {
        Err(XcError::Runtime)
    }
}

impl Default for Functional {
    /// `Functional::default()` is equivalent to `Functional::new()` — empty
    /// weights, `Mode::Unset`, and `settings` initialised to the
    /// `DEFAULT_SETTINGS` constant (parameter slots at their
    /// `common_parameters.cpp:17-29` defaults; functional slots zeroed).
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
//  W9 — Order 3 + 4 input-packing helpers (Plan 03-06 Task 1).
//
//  Each pack helper produces a flat `Vec<f64>` representing `inlen` slots of
//  CTaylor<f64, N> coefficients (size `1 << N` per slot). VAR0..VAR3 bit-flag
//  seeds are placed per the layout in `xcfun-master/src/XCFunctional.cpp:562-612`.
//
//  Bit-flag mapping (matches `crates/xcfun-ad/src/index.rs`):
//    CNST = 0  (0b0000)
//    VAR0 = 1  (0b0001)
//    VAR1 = 2  (0b0010)
//    VAR2 = 4  (0b0100)
//    VAR3 = 8  (0b1000)
//
//  These helpers are `pub` so they can be unit-tested from
//  `crates/xcfun-eval/tests/pack_ctaylor_inputs.rs`. They are also used by
//  `launch_and_accumulate` orders 3 and 4 below.
// ---------------------------------------------------------------------------

/// Pack a flat CTaylor<f64, 3> input block for order-3 seeding (W9).
///
/// Layout per XCFunctional.cpp:562-588:
///   - Each slot l ∈ 0..inlen occupies 8 consecutive f64s (size = 1 << 3 = 8).
///   - Coefficient[CNST=0]       = input[l]
///   - Coefficient[VAR0=1]       = 1.0 iff l == i
///   - Coefficient[VAR1=2]       = 1.0 iff l == j
///   - Coefficient[VAR2=4]       = 1.0 iff l == k
///   - All cross-terms (VAR0|VAR1 = 3, VAR0|VAR2 = 5, VAR1|VAR2 = 6,
///     VAR0|VAR1|VAR2 = 7) start at 0.0.
///
/// NOTE: Bit-flag semantics — VAR0 = 0b001, VAR1 = 0b010, VAR2 = 0b100.
/// So coefficient index 1 = VAR0-only, index 2 = VAR1-only, index 4 = VAR2-only.
/// Seeds at those three indices — NEVER at indices 3, 5, 6, or 7.
pub fn pack_ctaylor_inputs_order3(
    input: &[f64],
    inlen: usize,
    i: usize,
    j: usize,
    k: usize,
) -> Vec<f64> {
    const SIZE_N3: usize = 8; // 1 << 3
    const VAR0: usize = 1;
    const VAR1: usize = 2;
    const VAR2: usize = 4;

    debug_assert!(input.len() >= inlen);
    debug_assert!(i < inlen && j < inlen && k < inlen);

    let mut flat = vec![0.0_f64; inlen * SIZE_N3];
    for l in 0..inlen {
        flat[l * SIZE_N3 /* CNST */] = input[l];
    }
    flat[i * SIZE_N3 + VAR0] = 1.0;
    flat[j * SIZE_N3 + VAR1] = 1.0;
    flat[k * SIZE_N3 + VAR2] = 1.0;
    flat
}

/// Pack a flat CTaylor<f64, 4> input block for order-4 seeding (W9).
///
/// Layout per XCFunctional.cpp:600-612 (analogous to order 3):
///   - Each slot occupies 16 f64s (size = 1 << 4 = 16).
///   - Coefficient[VAR0=1]  = 1.0 iff l == i
///   - Coefficient[VAR1=2]  = 1.0 iff l == j
///   - Coefficient[VAR2=4]  = 1.0 iff l == k
///   - Coefficient[VAR3=8]  = 1.0 iff l == m
///
/// Readout: e.get(VAR0|VAR1|VAR2|VAR3) = flat_output[15].
pub fn pack_ctaylor_inputs_order4(
    input: &[f64],
    inlen: usize,
    i: usize,
    j: usize,
    k: usize,
    m: usize,
) -> Vec<f64> {
    const SIZE_N4: usize = 16; // 1 << 4
    const VAR0: usize = 1;
    const VAR1: usize = 2;
    const VAR2: usize = 4;
    const VAR3: usize = 8;

    debug_assert!(input.len() >= inlen);
    debug_assert!(i < inlen && j < inlen && k < inlen && m < inlen);

    let mut flat = vec![0.0_f64; inlen * SIZE_N4];
    for l in 0..inlen {
        flat[l * SIZE_N4 /* CNST */] = input[l];
    }
    flat[i * SIZE_N4 + VAR0] = 1.0;
    flat[j * SIZE_N4 + VAR1] = 1.0;
    flat[k * SIZE_N4 + VAR2] = 1.0;
    flat[m * SIZE_N4 + VAR3] = 1.0;
    flat
}

/// C(n, k) = n!/(k!(n-k)!). Helper for `inlen_triangle_count`.
fn binomial(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    (0..k).fold(1_usize, |acc, i| acc * (n - i) / (i + 1))
}

/// `taylor_len` up to and including order `max_order` for `inlen` variables.
/// For one slot + order N, the layout has `C(inlen + max_order, max_order)`
/// outputs per XCFunctional.cpp:501-612.
///
/// Used by `launch_and_accumulate` orders 3 + 4 to compute the starting
/// output slot index for each new derivative-order tier.
pub(crate) fn inlen_triangle_count(inlen: usize, max_order: usize) -> usize {
    (0..=max_order)
        .map(|k| binomial(inlen + k - 1, k))
        .sum()
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
        3 => {
            // Order 3 (Plan 03-06 Task 1, MODE-01 D-16). Triple-nested
            // (i ≤ j ≤ k) launch loop per XCFunctional.cpp:562-588.
            //
            // C++ XCFunctional.cpp `case 3:` falls through to `case 2:` (NO
            // break), populating outputs at orders 0/1/2 AND the new tier-3
            // slots. We mirror that fall-through behaviour by recursing into
            // launch_and_accumulate for order 2 first (which itself populates
            // orders 0/1/2), THEN appending the tier-3 outputs.
            //
            // Output slot offset starts after orders 0..=2 outputs:
            //   slot_start = inlen_triangle_count(inlen, 2)
            //              = 1 + inlen + inlen*(inlen+1)/2
            // Each (i, j, k) triple contributes one output: out[VAR0|VAR1|VAR2]
            // = out[7] of the kernel's CTaylor<F, 3> result.
            launch_and_accumulate(
                id_u32, vars_u32, 2, inlen, input, weight, output,
            )?;
            let mut slot = inlen_triangle_count(inlen, 2);
            for i in 0..inlen {
                for j in i..inlen {
                    for k in j..inlen {
                        let flat = pack_ctaylor_inputs_order3(input, inlen, i, j, k);
                        // CTaylor<f64, 3> coefficient block size = 1 << 3 = 8.
                        let out = run_launch(id_u32, vars_u32, 3, &flat, 8)?;
                        // VAR0 | VAR1 | VAR2 = 1 | 2 | 4 = 7.
                        output[slot] += weight * out[7];
                        slot += 1;
                    }
                }
            }
            Ok(())
        }
        4 => {
            // Order 4 (Plan 03-06 Task 1, MODE-01 D-16). Quadruple-nested
            // (i ≤ j ≤ k ≤ m) launch loop per XCFunctional.cpp:600-612.
            //
            // C++ does NOT support order 4 in xcfun_eval (XCFunctional.cpp
            // hits `xcfun::die` at the `default:` arm — no fall-through from
            // 4 → 3 → 2). Rust uniquely supports order 4 via its CTaylor<F,4>
            // generic kernel; tier-2 parity at order 4 is therefore
            // unattainable (no C++ reference). The driver caps at order 3.
            //
            // For Rust self-consistency we still mirror C++'s case-3 layout
            // pattern: populate orders 0/1/2/3 first, then the tier-4 slots.
            launch_and_accumulate(
                id_u32, vars_u32, 3, inlen, input, weight, output,
            )?;
            let mut slot = inlen_triangle_count(inlen, 3);
            for i in 0..inlen {
                for j in i..inlen {
                    for k in j..inlen {
                        for m in k..inlen {
                            let flat =
                                pack_ctaylor_inputs_order4(input, inlen, i, j, k, m);
                            // CTaylor<f64, 4> coefficient block size = 1 << 4 = 16.
                            let out = run_launch(id_u32, vars_u32, 4, &flat, 16)?;
                            // VAR0 | VAR1 | VAR2 | VAR3 = 1 | 2 | 4 | 8 = 15.
                            output[slot] += weight * out[15];
                            slot += 1;
                        }
                    }
                }
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
///
/// `pub(crate)` so the Mode::Contracted host-side dispatcher in
/// `crates/xcfun-eval/src/functionals/contracted.rs` can re-use the same
/// monomorphisation matrix (per-functional kernels are identical across
/// `Mode::PartialDerivatives` and `Mode::Contracted` per RESEARCH §"Mode::Contracted
/// Implementation"). Plan 04-05 D-06.
#[cfg(feature = "testing")]
pub(crate) fn run_launch(
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

            // ===== Phase 6 Plan 06-06 (D-18): LDA × vars=6 launch arms. =====
            //
            // Resolves the Phase 5 D-14 dispatch-table constraint forward.
            // A kernel's `Dependency` mask determines which Vars subset arms it
            // can launch into.  All 11 LDAs (Dependency::DENSITY only) → can
            // launch in any Vars where DENSITY ⊆ vars_dep_mask, including
            // A_B_GAA_GAB_GBB (vars=6).  `build_densvars` at vars=6 already
            // populates `d.n` / `d.s` correctly, so the LDA kernel bodies
            // (which only read those fields) work unchanged.  Mixed-LDA+GGA
            // aliases (b3lyp, camb3lyp) now eval in-process.
            //
            // Coverage: 11 LDA ids × n ∈ {0,1,2,3,4} = 55 arms.
            ( 0, 6, 0) => arm!( 0, 6, 0),  ( 0, 6, 1) => arm!( 0, 6, 1),
            ( 0, 6, 2) => arm!( 0, 6, 2),  ( 0, 6, 3) => arm!( 0, 6, 3),  ( 0, 6, 4) => arm!( 0, 6, 4),
            ( 2, 6, 0) => arm!( 2, 6, 0),  ( 2, 6, 1) => arm!( 2, 6, 1),
            ( 2, 6, 2) => arm!( 2, 6, 2),  ( 2, 6, 3) => arm!( 2, 6, 3),  ( 2, 6, 4) => arm!( 2, 6, 4),
            ( 3, 6, 0) => arm!( 3, 6, 0),  ( 3, 6, 1) => arm!( 3, 6, 1),
            ( 3, 6, 2) => arm!( 3, 6, 2),  ( 3, 6, 3) => arm!( 3, 6, 3),  ( 3, 6, 4) => arm!( 3, 6, 4),
            (13, 6, 0) => arm!(13, 6, 0),  (13, 6, 1) => arm!(13, 6, 1),
            (13, 6, 2) => arm!(13, 6, 2),  (13, 6, 3) => arm!(13, 6, 3),  (13, 6, 4) => arm!(13, 6, 4),
            (14, 6, 0) => arm!(14, 6, 0),  (14, 6, 1) => arm!(14, 6, 1),
            (14, 6, 2) => arm!(14, 6, 2),  (14, 6, 3) => arm!(14, 6, 3),  (14, 6, 4) => arm!(14, 6, 4),
            (15, 6, 0) => arm!(15, 6, 0),  (15, 6, 1) => arm!(15, 6, 1),
            (15, 6, 2) => arm!(15, 6, 2),  (15, 6, 3) => arm!(15, 6, 3),  (15, 6, 4) => arm!(15, 6, 4),
            (24, 6, 0) => arm!(24, 6, 0),  (24, 6, 1) => arm!(24, 6, 1),
            (24, 6, 2) => arm!(24, 6, 2),  (24, 6, 3) => arm!(24, 6, 3),  (24, 6, 4) => arm!(24, 6, 4),
            (25, 6, 0) => arm!(25, 6, 0),  (25, 6, 1) => arm!(25, 6, 1),
            (25, 6, 2) => arm!(25, 6, 2),  (25, 6, 3) => arm!(25, 6, 3),  (25, 6, 4) => arm!(25, 6, 4),
            (28, 6, 0) => arm!(28, 6, 0),  (28, 6, 1) => arm!(28, 6, 1),
            (28, 6, 2) => arm!(28, 6, 2),  (28, 6, 3) => arm!(28, 6, 3),  (28, 6, 4) => arm!(28, 6, 4),
            (55, 6, 0) => arm!(55, 6, 0),  (55, 6, 1) => arm!(55, 6, 1),
            (55, 6, 2) => arm!(55, 6, 2),  (55, 6, 3) => arm!(55, 6, 3),  (55, 6, 4) => arm!(55, 6, 4),
            (59, 6, 0) => arm!(59, 6, 0),  (59, 6, 1) => arm!(59, 6, 1),
            (59, 6, 2) => arm!(59, 6, 2),  (59, 6, 3) => arm!(59, 6, 3),  (59, 6, 4) => arm!(59, 6, 4),

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

            // ===== Phase 3 Wave-5 (Mode::Potential, plan 03-05) =====
            //
            // 38 supported ids × vars=28 (XC_A_B_2ND_TAYLOR) × {n=1, n=2}.
            //
            // Mode::Potential canonical Vars is A_B_2ND_TAYLOR (D-15 +
            // XCFunctional.cpp:482-490). The LDA loop runs at N=1; the GGA
            // divergence loop runs at N=2. For LDA functionals these arms
            // are also reachable when the harness drives Mode::Potential on
            // a pure-LDA functional with the spin-resolved 2ND_TAYLOR vars
            // (legal per `eval_setup`).
            //
            // 11 LDAs + 17 W2 GGAs + 10 W3 GGAs + 8 W4 GGAs = 46 total ids,
            // but only 38 are wired into dispatch_kernel (CSC + BRX/BRC/BRXC
            // + LB94 deferred per D-01-A and D-19).
            //
            // ----- 11 LDA + LDA-class ids -----
            ( 0, 28, 1) => arm!( 0, 28, 1),  ( 0, 28, 2) => arm!( 0, 28, 2),
            ( 2, 28, 1) => arm!( 2, 28, 1),  ( 2, 28, 2) => arm!( 2, 28, 2),
            ( 3, 28, 1) => arm!( 3, 28, 1),  ( 3, 28, 2) => arm!( 3, 28, 2),
            (13, 28, 1) => arm!(13, 28, 1),  (13, 28, 2) => arm!(13, 28, 2),
            (14, 28, 1) => arm!(14, 28, 1),  (14, 28, 2) => arm!(14, 28, 2),
            (15, 28, 1) => arm!(15, 28, 1),  (15, 28, 2) => arm!(15, 28, 2),
            (24, 28, 1) => arm!(24, 28, 1),  (24, 28, 2) => arm!(24, 28, 2),
            (25, 28, 1) => arm!(25, 28, 1),  (25, 28, 2) => arm!(25, 28, 2),
            (28, 28, 1) => arm!(28, 28, 1),  (28, 28, 2) => arm!(28, 28, 2),
            (55, 28, 1) => arm!(55, 28, 1),  (55, 28, 2) => arm!(55, 28, 2),
            (59, 28, 1) => arm!(59, 28, 1),  (59, 28, 2) => arm!(59, 28, 2),

            // ----- 17 Wave-2 GGAs -----
            ( 4, 28, 1) => arm!( 4, 28, 1),  ( 4, 28, 2) => arm!( 4, 28, 2),
            ( 5, 28, 1) => arm!( 5, 28, 1),  ( 5, 28, 2) => arm!( 5, 28, 2),
            ( 6, 28, 1) => arm!( 6, 28, 1),  ( 6, 28, 2) => arm!( 6, 28, 2),
            ( 7, 28, 1) => arm!( 7, 28, 1),  ( 7, 28, 2) => arm!( 7, 28, 2),
            ( 8, 28, 1) => arm!( 8, 28, 1),  ( 8, 28, 2) => arm!( 8, 28, 2),
            ( 9, 28, 1) => arm!( 9, 28, 1),  ( 9, 28, 2) => arm!( 9, 28, 2),
            (16, 28, 1) => arm!(16, 28, 1),  (16, 28, 2) => arm!(16, 28, 2),
            (19, 28, 1) => arm!(19, 28, 1),  (19, 28, 2) => arm!(19, 28, 2),
            (20, 28, 1) => arm!(20, 28, 1),  (20, 28, 2) => arm!(20, 28, 2),
            (21, 28, 1) => arm!(21, 28, 1),  (21, 28, 2) => arm!(21, 28, 2),
            (22, 28, 1) => arm!(22, 28, 1),  (22, 28, 2) => arm!(22, 28, 2),
            (69, 28, 1) => arm!(69, 28, 1),  (69, 28, 2) => arm!(69, 28, 2),
            (71, 28, 1) => arm!(71, 28, 1),  (71, 28, 2) => arm!(71, 28, 2),
            (72, 28, 1) => arm!(72, 28, 1),  (72, 28, 2) => arm!(72, 28, 2),
            (73, 28, 1) => arm!(73, 28, 1),  (73, 28, 2) => arm!(73, 28, 2),
            (74, 28, 1) => arm!(74, 28, 1),  (74, 28, 2) => arm!(74, 28, 2),
            (76, 28, 1) => arm!(76, 28, 1),  (76, 28, 2) => arm!(76, 28, 2),

            // ----- 10 Wave-3 GGAs -----
            ( 1, 28, 1) => arm!( 1, 28, 1),  ( 1, 28, 2) => arm!( 1, 28, 2),
            (17, 28, 1) => arm!(17, 28, 1),  (17, 28, 2) => arm!(17, 28, 2),
            (18, 28, 1) => arm!(18, 28, 1),  (18, 28, 2) => arm!(18, 28, 2),
            (26, 28, 1) => arm!(26, 28, 1),  (26, 28, 2) => arm!(26, 28, 2),
            (27, 28, 1) => arm!(27, 28, 1),  (27, 28, 2) => arm!(27, 28, 2),
            (56, 28, 1) => arm!(56, 28, 1),  (56, 28, 2) => arm!(56, 28, 2),
            (57, 28, 1) => arm!(57, 28, 1),  (57, 28, 2) => arm!(57, 28, 2),
            (67, 28, 1) => arm!(67, 28, 1),  (67, 28, 2) => arm!(67, 28, 2),
            (68, 28, 1) => arm!(68, 28, 1),  (68, 28, 2) => arm!(68, 28, 2),
            (77, 28, 1) => arm!(77, 28, 1),  (77, 28, 2) => arm!(77, 28, 2),

            // ----- 8 Wave-4 GGAs -----
            (23, 28, 1) => arm!(23, 28, 1),  (23, 28, 2) => arm!(23, 28, 2),
            (58, 28, 1) => arm!(58, 28, 1),  (58, 28, 2) => arm!(58, 28, 2),
            (60, 28, 1) => arm!(60, 28, 1),  (60, 28, 2) => arm!(60, 28, 2),
            (61, 28, 1) => arm!(61, 28, 1),  (61, 28, 2) => arm!(61, 28, 2),
            (62, 28, 1) => arm!(62, 28, 1),  (62, 28, 2) => arm!(62, 28, 2),
            (63, 28, 1) => arm!(63, 28, 1),  (63, 28, 2) => arm!(63, 28, 2),
            (64, 28, 1) => arm!(64, 28, 1),  (64, 28, 2) => arm!(64, 28, 2),
            (65, 28, 1) => arm!(65, 28, 1),  (65, 28, 2) => arm!(65, 28, 2),

            // ===== Phase 3 Wave-6 (Plan 03-06): orders 3 + 4 (MODE-01 D-16) =====
            //
            // 9 LDAs at vars=2 (XC_A_B, inlen=2) × {n=3, n=4} = 18 arms.
            // 35 GGAs at vars=6 (XC_A_B_GAA_GAB_GBB, inlen=5) × {n=3, n=4} = 70 arms.
            // Total: 88 new comptime monomorphisations (G10 budget validated at I2 capstone).
            //
            // ----- 9 LDAs at vars=2, n ∈ {3, 4} -----
            ( 0, 2, 3) => arm!( 0, 2, 3),  ( 0, 2, 4) => arm!( 0, 2, 4),
            ( 2, 2, 3) => arm!( 2, 2, 3),  ( 2, 2, 4) => arm!( 2, 2, 4),
            ( 3, 2, 3) => arm!( 3, 2, 3),  ( 3, 2, 4) => arm!( 3, 2, 4),
            (13, 2, 3) => arm!(13, 2, 3),  (13, 2, 4) => arm!(13, 2, 4),
            (14, 2, 3) => arm!(14, 2, 3),  (14, 2, 4) => arm!(14, 2, 4),
            (15, 2, 3) => arm!(15, 2, 3),  (15, 2, 4) => arm!(15, 2, 4),
            (24, 2, 3) => arm!(24, 2, 3),  (24, 2, 4) => arm!(24, 2, 4),
            (28, 2, 3) => arm!(28, 2, 3),  (28, 2, 4) => arm!(28, 2, 4),
            (55, 2, 3) => arm!(55, 2, 3),  (55, 2, 4) => arm!(55, 2, 4),

            // ----- 17 Wave-2 GGAs at vars=6, n ∈ {3, 4} -----
            ( 4, 6, 3) => arm!( 4, 6, 3),  ( 4, 6, 4) => arm!( 4, 6, 4),
            ( 5, 6, 3) => arm!( 5, 6, 3),  ( 5, 6, 4) => arm!( 5, 6, 4),
            ( 6, 6, 3) => arm!( 6, 6, 3),  ( 6, 6, 4) => arm!( 6, 6, 4),
            ( 7, 6, 3) => arm!( 7, 6, 3),  ( 7, 6, 4) => arm!( 7, 6, 4),
            ( 8, 6, 3) => arm!( 8, 6, 3),  ( 8, 6, 4) => arm!( 8, 6, 4),
            ( 9, 6, 3) => arm!( 9, 6, 3),  ( 9, 6, 4) => arm!( 9, 6, 4),
            (16, 6, 3) => arm!(16, 6, 3),  (16, 6, 4) => arm!(16, 6, 4),
            (19, 6, 3) => arm!(19, 6, 3),  (19, 6, 4) => arm!(19, 6, 4),
            (20, 6, 3) => arm!(20, 6, 3),  (20, 6, 4) => arm!(20, 6, 4),
            (21, 6, 3) => arm!(21, 6, 3),  (21, 6, 4) => arm!(21, 6, 4),
            (22, 6, 3) => arm!(22, 6, 3),  (22, 6, 4) => arm!(22, 6, 4),
            (69, 6, 3) => arm!(69, 6, 3),  (69, 6, 4) => arm!(69, 6, 4),
            (71, 6, 3) => arm!(71, 6, 3),  (71, 6, 4) => arm!(71, 6, 4),
            (72, 6, 3) => arm!(72, 6, 3),  (72, 6, 4) => arm!(72, 6, 4),
            (73, 6, 3) => arm!(73, 6, 3),  (73, 6, 4) => arm!(73, 6, 4),
            (74, 6, 3) => arm!(74, 6, 3),  (74, 6, 4) => arm!(74, 6, 4),
            (76, 6, 3) => arm!(76, 6, 3),  (76, 6, 4) => arm!(76, 6, 4),

            // ----- 10 Wave-3 GGAs at vars=6, n ∈ {3, 4} -----
            ( 1, 6, 3) => arm!( 1, 6, 3),  ( 1, 6, 4) => arm!( 1, 6, 4),
            (17, 6, 3) => arm!(17, 6, 3),  (17, 6, 4) => arm!(17, 6, 4),
            (18, 6, 3) => arm!(18, 6, 3),  (18, 6, 4) => arm!(18, 6, 4),
            (26, 6, 3) => arm!(26, 6, 3),  (26, 6, 4) => arm!(26, 6, 4),
            (27, 6, 3) => arm!(27, 6, 3),  (27, 6, 4) => arm!(27, 6, 4),
            (56, 6, 3) => arm!(56, 6, 3),  (56, 6, 4) => arm!(56, 6, 4),
            (57, 6, 3) => arm!(57, 6, 3),  (57, 6, 4) => arm!(57, 6, 4),
            (67, 6, 3) => arm!(67, 6, 3),  (67, 6, 4) => arm!(67, 6, 4),
            (68, 6, 3) => arm!(68, 6, 3),  (68, 6, 4) => arm!(68, 6, 4),
            (77, 6, 3) => arm!(77, 6, 3),  (77, 6, 4) => arm!(77, 6, 4),

            // ----- 8 Wave-4 GGAs at vars=6, n ∈ {3, 4} -----
            (23, 6, 3) => arm!(23, 6, 3),  (23, 6, 4) => arm!(23, 6, 4),
            (58, 6, 3) => arm!(58, 6, 3),  (58, 6, 4) => arm!(58, 6, 4),
            (60, 6, 3) => arm!(60, 6, 3),  (60, 6, 4) => arm!(60, 6, 4),
            (61, 6, 3) => arm!(61, 6, 3),  (61, 6, 4) => arm!(61, 6, 4),
            (62, 6, 3) => arm!(62, 6, 3),  (62, 6, 4) => arm!(62, 6, 4),
            (63, 6, 3) => arm!(63, 6, 3),  (63, 6, 4) => arm!(63, 6, 4),
            (64, 6, 3) => arm!(64, 6, 3),  (64, 6, 4) => arm!(64, 6, 4),
            (65, 6, 3) => arm!(65, 6, 3),  (65, 6, 4) => arm!(65, 6, 4),

            // ===== Phase 4 plan 04-07 (gap closure): metaGGA tier =====
            //
            // 26 metaGGA ids at vars=13 (XC_A_B_GAA_GAB_GBB_TAUA_TAUB, inlen=7)
            // × n ∈ {0,1,2,3} = 104 arms. Covers TPSS×5, BLOCX, SCAN×10,
            // M05×4, M06×8.
            //
            // ----- TPSS family + TPSSLOCC (5 ids) -----
            (41, 13, 0) => arm!(41, 13, 0),  (41, 13, 1) => arm!(41, 13, 1),
            (41, 13, 2) => arm!(41, 13, 2),  (41, 13, 3) => arm!(41, 13, 3),
            (42, 13, 0) => arm!(42, 13, 0),  (42, 13, 1) => arm!(42, 13, 1),
            (42, 13, 2) => arm!(42, 13, 2),  (42, 13, 3) => arm!(42, 13, 3),
            (43, 13, 0) => arm!(43, 13, 0),  (43, 13, 1) => arm!(43, 13, 1),
            (43, 13, 2) => arm!(43, 13, 2),  (43, 13, 3) => arm!(43, 13, 3),
            (44, 13, 0) => arm!(44, 13, 0),  (44, 13, 1) => arm!(44, 13, 1),
            (44, 13, 2) => arm!(44, 13, 2),  (44, 13, 3) => arm!(44, 13, 3),
            (75, 13, 0) => arm!(75, 13, 0),  (75, 13, 1) => arm!(75, 13, 1),
            (75, 13, 2) => arm!(75, 13, 2),  (75, 13, 3) => arm!(75, 13, 3),

            // ----- BLOCX (1 id, TAUA_TAUB only — no LAPLACIAN per descriptor) -----
            (70, 13, 0) => arm!(70, 13, 0),  (70, 13, 1) => arm!(70, 13, 1),
            (70, 13, 2) => arm!(70, 13, 2),  (70, 13, 3) => arm!(70, 13, 3),

            // ----- SCAN family (10 ids: 45..=54) -----
            (45, 13, 0) => arm!(45, 13, 0),  (45, 13, 1) => arm!(45, 13, 1),
            (45, 13, 2) => arm!(45, 13, 2),  (45, 13, 3) => arm!(45, 13, 3),
            (46, 13, 0) => arm!(46, 13, 0),  (46, 13, 1) => arm!(46, 13, 1),
            (46, 13, 2) => arm!(46, 13, 2),  (46, 13, 3) => arm!(46, 13, 3),
            (47, 13, 0) => arm!(47, 13, 0),  (47, 13, 1) => arm!(47, 13, 1),
            (47, 13, 2) => arm!(47, 13, 2),  (47, 13, 3) => arm!(47, 13, 3),
            (48, 13, 0) => arm!(48, 13, 0),  (48, 13, 1) => arm!(48, 13, 1),
            (48, 13, 2) => arm!(48, 13, 2),  (48, 13, 3) => arm!(48, 13, 3),
            (49, 13, 0) => arm!(49, 13, 0),  (49, 13, 1) => arm!(49, 13, 1),
            (49, 13, 2) => arm!(49, 13, 2),  (49, 13, 3) => arm!(49, 13, 3),
            (50, 13, 0) => arm!(50, 13, 0),  (50, 13, 1) => arm!(50, 13, 1),
            (50, 13, 2) => arm!(50, 13, 2),  (50, 13, 3) => arm!(50, 13, 3),
            (51, 13, 0) => arm!(51, 13, 0),  (51, 13, 1) => arm!(51, 13, 1),
            (51, 13, 2) => arm!(51, 13, 2),  (51, 13, 3) => arm!(51, 13, 3),
            (52, 13, 0) => arm!(52, 13, 0),  (52, 13, 1) => arm!(52, 13, 1),
            (52, 13, 2) => arm!(52, 13, 2),  (52, 13, 3) => arm!(52, 13, 3),
            (53, 13, 0) => arm!(53, 13, 0),  (53, 13, 1) => arm!(53, 13, 1),
            (53, 13, 2) => arm!(53, 13, 2),  (53, 13, 3) => arm!(53, 13, 3),
            (54, 13, 0) => arm!(54, 13, 0),  (54, 13, 1) => arm!(54, 13, 1),
            (54, 13, 2) => arm!(54, 13, 2),  (54, 13, 3) => arm!(54, 13, 3),

            // ----- M05 family (4 ids: 29, 30, 35, 36) -----
            (29, 13, 0) => arm!(29, 13, 0),  (29, 13, 1) => arm!(29, 13, 1),
            (29, 13, 2) => arm!(29, 13, 2),  (29, 13, 3) => arm!(29, 13, 3),
            (30, 13, 0) => arm!(30, 13, 0),  (30, 13, 1) => arm!(30, 13, 1),
            (30, 13, 2) => arm!(30, 13, 2),  (30, 13, 3) => arm!(30, 13, 3),
            (35, 13, 0) => arm!(35, 13, 0),  (35, 13, 1) => arm!(35, 13, 1),
            (35, 13, 2) => arm!(35, 13, 2),  (35, 13, 3) => arm!(35, 13, 3),
            (36, 13, 0) => arm!(36, 13, 0),  (36, 13, 1) => arm!(36, 13, 1),
            (36, 13, 2) => arm!(36, 13, 2),  (36, 13, 3) => arm!(36, 13, 3),

            // ----- M06 family (8 ids: 31..=34, 37..=40) -----
            (31, 13, 0) => arm!(31, 13, 0),  (31, 13, 1) => arm!(31, 13, 1),
            (31, 13, 2) => arm!(31, 13, 2),  (31, 13, 3) => arm!(31, 13, 3),
            (32, 13, 0) => arm!(32, 13, 0),  (32, 13, 1) => arm!(32, 13, 1),
            (32, 13, 2) => arm!(32, 13, 2),  (32, 13, 3) => arm!(32, 13, 3),
            (33, 13, 0) => arm!(33, 13, 0),  (33, 13, 1) => arm!(33, 13, 1),
            (33, 13, 2) => arm!(33, 13, 2),  (33, 13, 3) => arm!(33, 13, 3),
            (34, 13, 0) => arm!(34, 13, 0),  (34, 13, 1) => arm!(34, 13, 1),
            (34, 13, 2) => arm!(34, 13, 2),  (34, 13, 3) => arm!(34, 13, 3),
            (37, 13, 0) => arm!(37, 13, 0),  (37, 13, 1) => arm!(37, 13, 1),
            (37, 13, 2) => arm!(37, 13, 2),  (37, 13, 3) => arm!(37, 13, 3),
            (38, 13, 0) => arm!(38, 13, 0),  (38, 13, 1) => arm!(38, 13, 1),
            (38, 13, 2) => arm!(38, 13, 2),  (38, 13, 3) => arm!(38, 13, 3),
            (39, 13, 0) => arm!(39, 13, 0),  (39, 13, 1) => arm!(39, 13, 1),
            (39, 13, 2) => arm!(39, 13, 2),  (39, 13, 3) => arm!(39, 13, 3),
            (40, 13, 0) => arm!(40, 13, 0),  (40, 13, 1) => arm!(40, 13, 1),
            (40, 13, 2) => arm!(40, 13, 2),  (40, 13, 3) => arm!(40, 13, 3),

            // ===== Phase 4 plan 04-09 (gap closure): metaGGA cross-mode order 4 =====
            // Three exemplars (TPSSX, SCANX, M06X — one per family) at n=4 to
            // unblock the contracted_cross_mode test at orders 0..=4. Orders 5/6
            // for these ids remain forwarded to Phase 6 per Plan 04-05 D-19
            // (xcfun-ad ctaylor_compose/multo N=4..=6 specialisations).
            (42, 13, 4) => arm!(42, 13, 4),  // XC_TPSSX
            (46, 13, 4) => arm!(46, 13, 4),  // XC_SCANX
            (31, 13, 4) => arm!(31, 13, 4),  // XC_M06X

            // ----- BR family + CSC (4 ids at vars=17, full JP path) -----
            (10, 17, 0) => arm!(10, 17, 0),  (10, 17, 1) => arm!(10, 17, 1),
            (10, 17, 2) => arm!(10, 17, 2),  (10, 17, 3) => arm!(10, 17, 3),
            (11, 17, 0) => arm!(11, 17, 0),  (11, 17, 1) => arm!(11, 17, 1),
            (11, 17, 2) => arm!(11, 17, 2),  (11, 17, 3) => arm!(11, 17, 3),
            (12, 17, 0) => arm!(12, 17, 0),  (12, 17, 1) => arm!(12, 17, 1),
            (12, 17, 2) => arm!(12, 17, 2),  (12, 17, 3) => arm!(12, 17, 3),
            (66, 17, 0) => arm!(66, 17, 0),  (66, 17, 1) => arm!(66, 17, 1),
            (66, 17, 2) => arm!(66, 17, 2),  (66, 17, 3) => arm!(66, 17, 3),

            // ===== Phase 4 plan 04-05 (Mode::Contracted): orders 5 + 6 =====
            //
            // CTaylor<F, 5> = 32 coefficients per slot, CTaylor<F, 6> = 64
            // coefficients per slot. Stack budget at order 6 = 20 × 64 × 8 =
            // 10 KB per kernel invocation, well within cubecl-cpu budget per
            // RESEARCH §"CTaylor<F, 6> capacity check".
            //
            // Vars=2 SLATERX (id=0): inlen=2 — exercises orders 5/6 at
            // minimum cost (2 × 64 = 128 input doubles per launch).
            // Vars=6 PBEX (id=5): inlen=5 — exercises orders 5/6 at
            // representative GGA inlen (5 × 64 = 320 input doubles per launch).
            //
            // These two id/vars combinations are sufficient for the orders 5/6
            // cross-check vs the C++ DOEVAL macro (validation harness path).
            ( 0, 2, 5) => arm!( 0, 2, 5),  ( 0, 2, 6) => arm!( 0, 2, 6),
            ( 5, 6, 5) => arm!( 5, 6, 5),  ( 5, 6, 6) => arm!( 5, 6, 6),

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
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::Unset,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let mut out = vec![0.0; 1];
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::NotConfigured)
        ));
    }

    // The success path of `eval()` is only compiled under `feature = "testing"`
    // (the launch loop calls into `cpu_client()`, a test-only helper). Under
    // bare `cargo test -p xcfun-eval` (no features), `eval()` returns
    // `XcError::Runtime` from the `cfg(not(feature = "testing"))` else-branch.
    // Gate the happy-path acceptance test on the same feature so it only runs
    // when the launch loop is actually compiled in. Plan 04-06 Rule-1 fix.
    #[cfg(feature = "testing")]
    #[test]
    fn eval_contracted_mode_accepted_at_order_0() {
        // Plan 04-05 D-06: Mode::Contracted is wired. At order 0 the input
        // length is `inlen × (1 << 0) = inlen × 1 = inlen`, output length is 1.
        // With empty weights the output is zero-filled and Ok is returned.
        let f = Functional {
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::Contracted,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let mut out = vec![0.0; 1];
        assert!(f.eval(&[1.0, 0.5], &mut out).is_ok());
    }

    #[test]
    fn eval_rejects_contracted_order_above_6() {
        // Plan 04-05 D-06 + XCFUN_MAX_ORDER = 6: order 7 is rejected.
        let f = Functional {
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::Contracted,
            order: 7,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        // Output length validation runs first (output_length errors with
        // InvalidOrder when order > 6). The buffer size is irrelevant since
        // output_length rejects before any input parsing.
        let mut out = vec![0.0; 128];
        // Input would need 2 * (1 << 7) = 256 doubles to pass step 2; we don't
        // get that far — the order > 6 check at output_length step rejects first.
        let buf = vec![0.0_f64; 2 * (1_usize << 7)];
        assert!(matches!(
            f.eval(&buf, &mut out),
            Err(XcError::InvalidOrder { .. })
        ));
    }

    #[test]
    fn output_length_contracted_orders_0_to_6() {
        // D-06-B: output_length for Contracted = `1 << order`.
        for order in 0_u32..=6 {
            let expected = 1_usize << order;
            assert_eq!(
                Functional::output_length(Vars::A_B, Mode::Contracted, order).unwrap(),
                expected,
                "order {} should give 1 << {} = {}",
                order,
                order,
                expected,
            );
        }
    }

    #[test]
    fn output_length_contracted_rejects_order_above_6() {
        // D-06: order > 6 returns InvalidOrder.
        assert!(matches!(
            Functional::output_length(Vars::A_B, Mode::Contracted, 7),
            Err(XcError::InvalidOrder { .. })
        ));
    }

    #[test]
    fn eval_setup_contracted_accepts_orders_0_to_6() {
        // D-06: eval_setup must accept orders 0..=6 for Mode::Contracted.
        let f = Functional {
            weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Contracted,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        for order in 0_u32..=6 {
            assert!(
                f.eval_setup(Vars::A_B, Mode::Contracted, order).is_ok(),
                "Mode::Contracted at order {} must pass eval_setup",
                order
            );
        }
    }

    #[test]
    fn eval_setup_contracted_rejects_order_above_6() {
        // D-06 + XCFUN_MAX_ORDER = 6: eval_setup rejects order 7.
        let f = Functional {
            weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Contracted,
            order: 7,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        assert!(matches!(
            f.eval_setup(Vars::A_B, Mode::Contracted, 7),
            Err(XcError::InvalidOrder { .. })
        ));
    }

    #[test]
    fn eval_rejects_order_above_4() {
        // Plan 03-06 Task 1: order limit raised to 4 per MODE-01 D-16.
        let f = Functional {
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 5,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let mut out = vec![0.0; 21]; // taylorlen(2, 5) = 21
        assert!(matches!(
            f.eval(&[1.0, 0.5], &mut out),
            Err(XcError::InvalidOrder { .. })
        ));
    }

    #[test]
    fn eval_rejects_input_length_mismatch() {
        let f = Functional {
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
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
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 1,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
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
            weights: Vec::new(),
            vars: Vars::A_B,
            mode: Mode::PartialDerivatives,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
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
            weights: vec![(FunctionalId::XC_M05X, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
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
            weights: vec![(FunctionalId::XC_BRX, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
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
        // Plan 05-00 D-08-A: error variant is now InvalidVarsAndMode (combined),
        // mirroring XC_EVARS|XC_EMODE (=6) from XCFunctional.cpp:441-443.
        let f = Functional {
            weights: vec![(FunctionalId::XC_PBEX, 1.0)],
            vars: Vars::A_B_GAA_GAB_GBB,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let err = f
            .eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::Potential, 0)
            .unwrap_err();
        match err {
            XcError::InvalidVarsAndMode { vars: v, mode: m, depends: d } => {
                assert_eq!(v, Vars::A_B_GAA_GAB_GBB);
                assert_eq!(m, Mode::Potential);
                assert!(d.contains(Dependency::GRADIENT));
            }
            e => panic!("expected InvalidVarsAndMode, got {e:?}"),
        }
    }

    #[test]
    fn eval_setup_emits_combined_error_when_gga_potential_with_lda_vars() {
        // Plan 05-00 D-08-A — PBEX = pure GGA, no laplacian/kinetic.
        // Vars::A_B is LDA-shaped (no _2ND_TAYLOR). Must emit the combined
        // InvalidVarsAndMode (XC_EVARS|XC_EMODE = 6).
        let f = Functional {
            weights: vec![(FunctionalId::XC_PBEX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let err = f.eval_setup(Vars::A_B, Mode::Potential, 0).unwrap_err();
        match err {
            XcError::InvalidVarsAndMode { vars, mode, depends } => {
                assert_eq!(vars, Vars::A_B);
                assert_eq!(mode, Mode::Potential);
                assert!(depends.contains(Dependency::GRADIENT));
            }
            e => panic!("expected InvalidVarsAndMode, got {e:?}"),
        }
    }

    #[test]
    fn eval_setup_accepts_gga_with_2nd_taylor_potential() {
        // PBEX + A_B_2ND_TAYLOR is the valid combination — must pass.
        let f = Functional {
            weights: vec![(FunctionalId::XC_PBEX, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        assert!(f
            .eval_setup(Vars::A_B_2ND_TAYLOR, Mode::Potential, 0)
            .is_ok());
    }

    #[test]
    fn eval_setup_accepts_lda_with_any_vars_potential() {
        // SLATERX is DENSITY only — any non-metaGGA Vars should pass Mode::Potential.
        let f = Functional {
            weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        assert!(f.eval_setup(Vars::A_B, Mode::Potential, 0).is_ok());
    }

    #[test]
    fn eval_setup_rejects_unset_mode() {
        let f = Functional {
            weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
            vars: Vars::A_B,
            mode: Mode::Unset,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
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
            weights: vec![
                (FunctionalId::XC_SLATERX, 0.5),
                (FunctionalId::XC_PBEX, 0.5),
            ],
            vars: Vars::A_B_GAA_GAB_GBB,
            mode: Mode::PartialDerivatives,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let deps = f.dependencies();
        assert!(deps.contains(Dependency::DENSITY));
        assert!(deps.contains(Dependency::GRADIENT));
        assert!(!deps.contains(Dependency::KINETIC));
        assert!(!deps.contains(Dependency::LAPLACIAN));
    }
}
