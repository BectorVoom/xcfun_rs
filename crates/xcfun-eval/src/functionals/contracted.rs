//! Mode::Contracted host-side dispatcher — port of `xcfun-master/src/XCFunctional.cpp:619-635`
//! `DOEVAL` macro per Plan 04-05 D-06 + D-06-A + D-06-B.
//!
//! # Verbatim port target (`XCFunctional.cpp:619-635`)
//!
//! ```cpp
//! } else if (fun->mode == XC_CONTRACTED) {
//! #define DOEVAL(N, E)                                                                \
//!   if (fun->order == N) {                                                            \
//!     typedef ctaylor<ireal_t, N> ttype;                                              \
//!     int inlen = xcint_vars[fun->vars].len;                                          \
//!     ttype in[XC_MAX_INVARS], out = 0;                                               \
//!     int k = 0;                                                                      \
//!     for (int i = 0; i < inlen; i++)                                                 \
//!       for (int j = 0; j < (1 << fun->order); j++)                                   \
//!         in[i].set(j, input[k++]);                                                   \
//!     densvars<ttype> d(fun, in);                                                     \
//!     for (int i = 0; i < fun->nr_active_functionals; i++)                            \
//!       out += fun->settings[fun->active_functionals[i]->id] *                        \
//!              fun->active_functionals[i]->fp##N(d);                                  \
//!     for (int i = 0; i < (1 << fun->order); i++)                                     \
//!       output[i] = out.get(i);                                                       \
//!   } else
//!     FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )
//!     xcfun::die("bug! Order too high in XC_CONTRACTED", fun->order);
//! }
//! ```
//!
//! `FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )` expands to seven `if` arms (orders
//! 0..=6). The Rust port dispatches each as a comptime arm into the existing
//! `run_launch` infrastructure (which monomorphises the per-functional kernel
//! at the requested CTaylor order N).
//!
//! # Input layout (D-06-A, RESEARCH §"Mode::Contracted Implementation: Input Layout")
//!
//! For order N and `inlen = VARS_TABLE[vars].len`, the caller passes
//! `inlen × (1 << N)` flat f64 doubles laid out per Vars-element:
//!
//! ```text
//! input[0]                = in[0].coeff(CNST = 0)
//! input[1]                = in[0].coeff(VAR0 = 1)
//! input[2]                = in[0].coeff(VAR1 = 2)
//! ...
//! input[(1<<N) - 1]       = in[0].coeff((1<<N) - 1)   // top bit-flag combo
//! input[(1<<N) + 0]       = in[1].coeff(CNST)
//! ...
//! input[inlen*(1<<N) - 1] = in[inlen-1].coeff((1<<N) - 1)
//! ```
//!
//! This is **identical** to the flat layout consumed by `run_launch` /
//! `eval_point_kernel` / `build_densvars` for `Mode::PartialDerivatives` —
//! Mode::Contracted is a re-packaging, not a different kernel (RESEARCH
//! §"Mode::Contracted does NOT call a per-mode kernel"). Every per-functional
//! `<name>_kernel<F, const N>` body already used by `Mode::PartialDerivatives`
//! is re-invoked here at `N = order`.
//!
//! # Output layout (D-06-B)
//!
//! `output[i] = out.get(i)` for `i ∈ 0..(1 << order)`. The bit-flag indexing
//! means `output[0]` = energy (CNST coefficient), `output[1]` = ∂E/∂x₀,
//! `output[3]` = ∂²E/∂x₀∂x₁, etc. Output length is exactly `1 << order` per
//! D-06-B.
//!
//! # Order cap (D-06)
//!
//! Orders 0..=6 are valid (XCFUN_MAX_ORDER = 6). Order > 6 returns
//! `XcError::InvalidOrder` — matches the C++ `xcfun::die` at the FOR_EACH
//! fall-through. The host-side validation in `Functional::eval` rejects
//! `order > 6` before reaching this dispatcher.
//!
//! # Stack budget (T-04-05-01)
//!
//! At order 6 with XC_MAX_INVARS = 20 Vars-elements: 20 × 64 × 8 = 10,240
//! bytes per kernel invocation. Within cubecl-cpu's kernel-stack budget per
//! RESEARCH §"CTaylor<F, 6> capacity check". Verified at Wave 0 by the
//! `test_ctaylor_n6` smoke test on xcfun-ad.

use xcfun_core::{VARS_TABLE, XcError};

use crate::functional::Functional;

/// Mode::Contracted entry point — dispatches one of seven comptime order arms
/// (0..=6) per `XCFunctional.cpp:619-635` `DOEVAL` macro expansion.
///
/// # Behaviour
///
/// 1. Look up `inlen = VARS_TABLE[vars].len`.
/// 2. Compute `coeff_count = 1 << order`.
/// 3. Defense-in-depth re-validate `input.len() == inlen * coeff_count` and
///    `output.len() == coeff_count`. (`Functional::eval` already enforces
///    these — duplicating here protects against direct callers per
///    threat-model T-04-05-02 / T-04-05-03.)
/// 4. For each `(id, weight)` in `self.weights`, launch the per-functional
///    kernel at `N = order` via `run_launch` and accumulate
///    `output[i] += weight * out_coeffs[i]` for `i ∈ 0..coeff_count`.
///
/// Per RESEARCH §"Per-functional kernel re-use", the underlying
/// `<name>_kernel<F, const N>` body is the SAME one consumed by
/// `Mode::PartialDerivatives`. The DOEVAL macro merely changes the host-side
/// pack/unpack — no per-functional kernel changes are needed.
///
/// # Errors
///
/// - `XcError::InvalidOrder` if `order > 6`.
/// - `XcError::InputLengthMismatch` if `input.len() != inlen * (1 << order)`.
/// - `XcError::OutputLengthMismatch` if `output.len() != (1 << order)`.
/// - Any `XcError` propagated from the underlying `run_launch` (typically
///   `XcError::NotConfigured` when an `(id, vars, n)` tuple is missing
///   from the dispatch matrix).
#[cfg(feature = "testing")]
pub fn launch_contracted(
    functional: &Functional,
    input: &[f64],
    output: &mut [f64],
) -> Result<(), XcError> {
    let order = functional.order;
    let vars = functional.vars;
    let inlen = VARS_TABLE[vars as usize].len as usize;

    // Order cap (D-06 + XCFUN_MAX_ORDER = 6).
    if order > 6 {
        return Err(XcError::InvalidOrder {
            order,
            mode: functional.mode,
            n_vars: inlen,
        });
    }

    let coeff_count = 1_usize << order;

    // Defense-in-depth — Functional::eval already validates these (Plan 04-05
    // step 2 / 4). Duplicating per threat-model T-04-05-02 / T-04-05-03.
    if input.len() != inlen * coeff_count {
        return Err(XcError::InputLengthMismatch {
            expected: inlen * coeff_count,
            got: input.len(),
        });
    }
    if output.len() != coeff_count {
        return Err(XcError::OutputLengthMismatch {
            expected: coeff_count,
            got: output.len(),
        });
    }

    // DOEVAL inner loop (XCFunctional.cpp:631-633): accumulate weighted
    // outputs from each active functional. The CTaylor<f64, ORDER>
    // monomorphisation is folded into `run_launch`'s existing comptime arms.
    //
    // Note: input slice IS the flat CTaylor pack — `run_launch` passes it
    // directly to `eval_point_kernel`, which calls `build_densvars` to
    // populate `DensVarsDev<F>` from the flat layout (identical to the
    // PartialDerivatives input contract — see
    // `crates/xcfun-eval/src/density_vars/build.rs:154-189`).
    // Plan 06-06 D-17: weights is now Vec<...>; iterate by reference.
    for &(id, weight) in functional.weights.iter() {
        let id_u32 = id as u32;
        let out_vec = crate::functional::run_launch(
            id_u32,
            vars as u32,
            order,
            input,
            coeff_count,
        )?;
        debug_assert_eq!(out_vec.len(), coeff_count);

        // XCFunctional.cpp:633 `output[i] = out.get(i)` is a write — but
        // `out += weight * fp_N(d)` accumulates ACROSS active functionals at
        // L631. We accumulate weighted outputs into `output[i]` directly.
        for i in 0..coeff_count {
            output[i] += weight * out_vec[i];
        }
    }

    Ok(())
}

/// Stub for non-testing builds (mirrors the `eval` non-testing guard).
/// Production paths will land via the Phase 5 `xcfun-rs::Functional` facade.
#[cfg(not(feature = "testing"))]
pub fn launch_contracted(
    _functional: &Functional,
    _input: &[f64],
    _output: &mut [f64],
) -> Result<(), XcError> {
    Err(XcError::Runtime)
}
