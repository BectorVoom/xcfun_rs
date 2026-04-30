//! Native Rust facade `Functional` (Phase 5 D-02).
//!
//! Newtype around `xcfun_eval::Functional`. Methods delegate. The
//! field is private so callers cannot bypass `set` validation by
//! mutating `weights` / `settings` directly.

use xcfun_core::{Dependency, Mode, Vars, XcError};

/// The exchange-correlation functional handle.
///
/// RS-01..10 surface. Construct via [`Self::new`], then configure
/// active functionals + parameters via [`Self::set`], then invoke
/// [`Self::eval_setup`] before [`Self::eval`].
pub struct Functional(xcfun_eval::Functional);

impl core::fmt::Debug for Functional {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The inner xcfun_eval::Functional does not yet derive Debug.
        // Surface a stable, content-light summary so application logging /
        // assertion macros still compile against `Functional`.
        f.debug_struct("Functional")
            .field("vars", &self.0.vars)
            .field("mode", &self.0.mode)
            .field("order", &self.0.order)
            .field("weights_len", &self.0.weights.len())
            .finish()
    }
}

impl Functional {
    /// RS-01 — fresh handle: no active functionals, parameters at
    /// their defaults (XCFunctional.cpp:350-355).
    pub const fn new() -> Self {
        Self(xcfun_eval::Functional::new())
    }

    /// RS-02 — case-insensitive name set. Three-case dispatch
    /// (functional / parameter / alias) per XCFunctional.cpp:369-405.
    ///
    /// **Phase 5 D-02 boundary note:** `xcfun_eval::Functional::set` updates the
    /// 82-slot `settings` array. The Phase 5 facade does NOT yet rebuild the
    /// `weights: &'static [(FunctionalId, f64)]` slice from `settings`; the
    /// existing `weights` slice (default empty for fresh handles) drives
    /// `dependencies()` and `eval()`. To exercise an active-functional path
    /// the caller must construct a `Functional` whose inner `weights` field is
    /// pre-populated — done internally by tests via the
    /// `with_weights_for_test` helper. End-to-end wiring of `set` →
    /// `weights` is captured by the C-ABI layer (Plan 05-02) and the Phase 6
    /// dispatch refactor.
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        self.0.set(name, value)
    }

    /// RS-03 — read functional weight or parameter value.
    /// Aliases NOT supported (mirror XCFunctional.cpp:407-419).
    pub fn get(&self, name: &str) -> Result<f64, XcError> {
        self.0.get(name)
    }

    /// RS-04 — `(depends & XC_GRADIENT)` per XCFunctional.cpp:420.
    pub fn is_gga(&self) -> bool {
        self.0.dependencies().contains(Dependency::GRADIENT)
    }

    /// RS-04 — `(depends & (XC_LAPLACIAN | XC_KINETIC))` per
    /// XCFunctional.cpp:422-424.
    pub fn is_metagga(&self) -> bool {
        let d = self.0.dependencies();
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
    pub fn eval_setup(
        &mut self,
        vars: Vars,
        mode: Mode,
        order: u32,
    ) -> Result<(), XcError> {
        // XCFunctional.cpp:438-441 — validate first; mutate only on success.
        self.0.eval_setup(vars, mode, order)?;
        self.0.vars = vars;
        self.0.mode = mode;
        self.0.order = order;
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
        xcfun_eval::Functional::input_length(self.0.vars)
    }

    /// MODE-05 / RS-09 — number of `f64` outputs `eval` writes.
    pub fn output_length(&self) -> Result<usize, XcError> {
        xcfun_eval::Functional::output_length(self.0.vars, self.0.mode, self.0.order)
    }

    /// RS-07 — evaluate. Zero heap allocation on the success path is
    /// the contract; see `tests/zero_alloc.rs` for the verifying fixture.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        self.0.eval(input, output)
    }

    // -- internal helper used by `user_eval_setup` for the
    //    `InvalidOrder.n_vars` field ------------------------------------
    #[inline]
    fn input_length_of(vars: Vars) -> usize {
        xcfun_eval::Functional::input_length(vars)
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
    fn with_weights_for_test(weights: &'static [(xcfun_core::FunctionalId, f64)]) -> Self {
        let mut inner = xcfun_eval::Functional::new();
        inner.weights = weights;
        Self(inner)
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
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0).unwrap();
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
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0).unwrap();
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
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 1).unwrap();
        // taylorlen(2, 1) = 3
        assert_eq!(f.output_length().unwrap(), 3);
    }
}
