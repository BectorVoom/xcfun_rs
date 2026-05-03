//! `ctaylor_rec::compose` — series composition `out = f(x)` where `f` is
//! an `(Nvar+1)`-coefficient scalar series and `x` is a CTaylor. Per-N
//! specialization for N ∈ 0..=3.
//!
//! Port of `xcfun-master/external/upstream/taylor/ctaylor.hpp:72-82`
//! (general recursion) + `:91, 112-115, 146-151` (N=0, 1, 2 base cases).
//!
//! # C++ general recursion (ctaylor.hpp:72-82)
//!
//! ```cpp
//! /* Put sum_i coeff[i]*(x - x[0])^i in res,
//!    used when evaluating analytical functions of this */
//! static void compose(T * res, const T * x, const T coeff[]) {
//!   res[0] = coeff[Nvar];
//!   for (int i = 1; i < POW2(Nvar); i++) res[i] = 0;
//!   for (int i = Nvar - 1; i >= 0; i--) {
//!     ctaylor_rec<T, Nvar>::multo_skipconst(res, x);
//!     res[0] += coeff[i];
//!   }
//! }
//! ```
//!
//! # Pitfall P11 — descending loop order is MANDATORY.
//!
//! The outer loop at ctaylor.hpp:78 runs `for (i = Nvar-1; i >= 0; i--)`;
//! this is descending-order Horner. Reversing it to ascending breaks the
//! coefficient accumulation order for n ≥ 2, losing 1e-12 parity.
//!
//! # Base cases (ctaylor.hpp:91, :112-115, :146-151)
//!
//! ```cpp
//! // N=0
//! static void compose(T * res, const T * x, const T coeff[]) {
//!   res[0] = coeff[0];
//! }
//! // N=1
//! static void compose(T * res, const T * x, const T coeff[]) {
//!   res[0] = coeff[0];
//!   res[1] = coeff[1] * x[1];
//! }
//! // N=2
//! static void compose(T * res, const T * x, const T coeff[]) {
//!   res[0] = coeff[0];
//!   res[1] = coeff[1] * x[1];
//!   res[2] = coeff[1] * x[2];
//!   res[3] = coeff[1] * x[3] + 2 * x[1] * x[2] * coeff[2];
//! }
//! ```

use crate::ctaylor_rec::multo::{
    ctaylor_multo_skipconst_n1, ctaylor_multo_skipconst_n2,
    ctaylor_multo_skipconst_n3, ctaylor_multo_skipconst_n4,
};
use cubecl::prelude::*;

// ---------------------------------------------------------------------------
//  compose — out = f(x) given scalar-series f of length N+1
// ---------------------------------------------------------------------------

/// N=0 compose. Port of `ctaylor.hpp:91` — `res[0] = coeff[0]`.
///
/// `f` is expected to be length-1 (a single scalar constant).
#[cube]
pub(crate) fn ctaylor_compose_n0<F: Float>(
    out: &mut Array<F>,
    _x: &Array<F>,
    f: &Array<F>,
) {
    out[0] = f[0];
}

/// N=1 compose. Port of `ctaylor.hpp:112-115`.
///
/// ```cpp
/// res[0] = coeff[0];
/// res[1] = coeff[1] * x[1];
/// ```
#[cube]
pub(crate) fn ctaylor_compose_n1<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
) {
    out[0] = f[0];
    out[1] = f[1] * x[1];
}

/// N=2 compose. Port of `ctaylor.hpp:146-151`.
///
/// ```cpp
/// res[0] = coeff[0];
/// res[1] = coeff[1] * x[1];
/// res[2] = coeff[1] * x[2];
/// res[3] = coeff[1] * x[3] + 2 * x[1] * x[2] * coeff[2];
/// ```
///
/// The trailing term at `res[3]` reads `2 * x[1] * x[2] * coeff[2]`;
/// C++ left-to-right gives `((2 * x[1]) * x[2]) * coeff[2]`.
#[cube]
pub(crate) fn ctaylor_compose_n2<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
) {
    out[0] = f[0];
    out[1] = f[1] * x[1];
    out[2] = f[1] * x[2];

    let two = F::new(2.0);
    let t1 = f[1] * x[3];
    let s1 = two * x[1];
    let s2 = s1 * x[2];
    let s3 = s2 * f[2];
    out[3] = t1 + s3;
}

/// N=3 compose — implemented via the general recursion at `ctaylor.hpp:74-81`:
///
/// ```cpp
/// res[0] = coeff[3];
/// for (i = 1..7) res[i] = 0;
/// for (i = 2 downto 0) {
///   ctaylor_rec<T, 3>::multo_skipconst(res, x);
///   res[0] += coeff[i];
/// }
/// ```
///
/// Each step:
///   i=2: multo_skipconst_n3(out, x); out[0] += f[2];
///   i=1: multo_skipconst_n3(out, x); out[0] += f[1];
///   i=0: multo_skipconst_n3(out, x); out[0] += f[0];
#[cube]
pub(crate) fn ctaylor_compose_n3<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
) {
    let zero = F::new(0.0);

    // Seed with coeff[N] and zero out higher-order slots.
    out[0] = f[3];
    out[1] = zero;
    out[2] = zero;
    out[3] = zero;
    out[4] = zero;
    out[5] = zero;
    out[6] = zero;
    out[7] = zero;

    // Descending Horner: i = 2, 1, 0.
    ctaylor_multo_skipconst_n3::<F>(out, x);
    out[0] = out[0] + f[2];

    ctaylor_multo_skipconst_n3::<F>(out, x);
    out[0] = out[0] + f[1];

    ctaylor_multo_skipconst_n3::<F>(out, x);
    out[0] = out[0] + f[0];
}

// ---------------------------------------------------------------------------
//  N=1 and N=2 general-recursion forms (used for cross-checking in tests)
// ---------------------------------------------------------------------------
//
// The base-case bodies above are the C++ "ctaylor_rec<T, 1>::compose" and
// "ctaylor_rec<T, 2>::compose" specialisations, i.e. pre-unrolled forms.
// If any downstream test wants to exercise the general recursion for N=1
// or N=2, it can call `ctaylor_compose_rec_n{1,2}` below. These are NOT
// part of the public dispatch (the pre-unrolled base cases are faster
// and bit-identical to the C++ specialisations), but they document that
// the recursion/base-case equivalence is intentional.

#[cube]
#[allow(dead_code)]
pub(crate) fn ctaylor_compose_rec_n1<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
) {
    // Mirrors the general-recursion body for Nvar=1.
    out[0] = f[1];
    out[1] = F::new(0.0);
    ctaylor_multo_skipconst_n1::<F>(out, x);
    out[0] = out[0] + f[0];
}

#[cube]
#[allow(dead_code)]
pub(crate) fn ctaylor_compose_rec_n2<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
) {
    // Mirrors the general-recursion body for Nvar=2.
    let zero = F::new(0.0);
    out[0] = f[2];
    out[1] = zero;
    out[2] = zero;
    out[3] = zero;

    ctaylor_multo_skipconst_n2::<F>(out, x);
    out[0] = out[0] + f[1];

    ctaylor_multo_skipconst_n2::<F>(out, x);
    out[0] = out[0] + f[0];
}

/// N=4 compose — Phase 6 Plan 06-00 Task 1.
///
/// Implemented via the general recursion at `ctaylor.hpp:74-81`:
/// ```cpp
/// res[0] = coeff[4];
/// for (i = 1..15) res[i] = 0;
/// for (i = 3 downto 0) {
///   ctaylor_rec<T, 4>::multo_skipconst(res, x);
///   res[0] += coeff[i];
/// }
/// ```
///
/// Each step:
///   i=3: multo_skipconst_n4(out, x); out[0] += f[3];
///   i=2: multo_skipconst_n4(out, x); out[0] += f[2];
///   i=1: multo_skipconst_n4(out, x); out[0] += f[1];
///   i=0: multo_skipconst_n4(out, x); out[0] += f[0];
///
/// `f` is the (Nvar+1)-length scalar-series coefficient table; `x` and
/// `out` are length 1<<4 = 16.
#[cube]
pub fn ctaylor_compose_n4<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
) {
    let zero = F::new(0.0);

    // Seed with coeff[N=4] and zero out higher-order slots (size 16).
    out[0] = f[4];
    out[1] = zero;
    out[2] = zero;
    out[3] = zero;
    out[4] = zero;
    out[5] = zero;
    out[6] = zero;
    out[7] = zero;
    out[8] = zero;
    out[9] = zero;
    out[10] = zero;
    out[11] = zero;
    out[12] = zero;
    out[13] = zero;
    out[14] = zero;
    out[15] = zero;

    // Descending Horner: i = 3, 2, 1, 0.
    ctaylor_multo_skipconst_n4::<F>(out, x);
    out[0] = out[0] + f[3];

    ctaylor_multo_skipconst_n4::<F>(out, x);
    out[0] = out[0] + f[2];

    ctaylor_multo_skipconst_n4::<F>(out, x);
    out[0] = out[0] + f[1];

    ctaylor_multo_skipconst_n4::<F>(out, x);
    out[0] = out[0] + f[0];
}

// ---------------------------------------------------------------------------
//  Outer dispatch
// ---------------------------------------------------------------------------

/// Outer dispatch for `out = f(x)` across N ∈ {0, 1, 2, 3, 4}. `f` must have
/// length `n + 1` (scalar-series coefficients); `x` and `out` are length
/// `1 << n`.
///
/// **Phase 6 status:** N=4 lands in Plan 06-00 Task 1 (D-19 forward from
/// Phase-4 Plan 04-05 — Mode::Contracted order-5 metaGGA). N=5 / N=6 are
/// pending; a follow-up plan will land them.
#[cube]
pub fn ctaylor_compose<F: Float>(
    out: &mut Array<F>,
    x: &Array<F>,
    f: &Array<F>,
    #[comptime] n: u32,
) {
    if comptime!(n == 0) {
        ctaylor_compose_n0::<F>(out, x, f);
    } else if comptime!(n == 1) {
        ctaylor_compose_n1::<F>(out, x, f);
    } else if comptime!(n == 2) {
        ctaylor_compose_n2::<F>(out, x, f);
    } else if comptime!(n == 3) {
        ctaylor_compose_n3::<F>(out, x, f);
    } else if comptime!(n == 4) {
        ctaylor_compose_n4::<F>(out, x, f);
    }
}
