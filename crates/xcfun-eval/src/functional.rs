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

use xcfun_core::{FunctionalId, Mode, Vars, XcError, taylorlen};

use crate::dispatch;

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
    ///
    /// The actual cubecl-cpu launch loop over `(FunctionalId, weight)` pairs is
    /// stubbed in Plan 02-03 — it just zeroes the output buffer once validation
    /// passes. Plan 02-04 replaces the `output.fill(0.0)` body with the per-order
    /// launch loop calling `dispatch::dispatch_kernel`.
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
        // 6. Per-functional cubecl-cpu launches accumulating into `output`.
        //
        // Phase 2 launch pattern (per RESEARCH §"Mode::PartialDerivatives Output Layout"):
        //   - Order 0: 1 launch with N=0; output[0] = sum_w * out[CNST]
        //   - Order 1, inlen=k: k launches with N=1, each setting one input's
        //     VAR0=1 in turn. Output: [energy, ∂_0, ∂_1, ..., ∂_{k-1}].
        //   - Order 2, inlen=k: k*(k+1)/2 launches with N=2, nested loop.
        //     Output: [energy, k first-derivs, k*(k+1)/2 second-derivs].
        //
        // The actual launch helpers + per-order output element packing are stubbed
        // here; Plan 02-04 implements the per-order loops + the launch helper that
        // wraps `dispatch::dispatch_kernel`. This task ships the validation logic
        // + a zero-fill body so the type surface compiles.
        output.fill(0.0);
        // TODO(Plan 02-04): implement per-order launch loop calling dispatch_kernel.
        Ok(())
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
