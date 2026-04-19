//! `ctaylor_rec::mul` / `mul_set` — multiply-set and multiply-accumulate
//! for `CTaylor` polynomials. Per-N specialization for N ∈ 0..=7.
//!
//! Port of `xcfun-master/external/upstream/taylor/ctaylor.hpp:41`
//! (general recursion) + `:86-141` (N=0,1,2 base cases).
//!
//! # C++ general recursion (ctaylor.hpp:41-52)
//!
//! ```cpp
//! template <class T, int Nvar> struct ctaylor_rec {
//!   // Add x*y to dst (accumulating)
//!   static void mul(T * dst, const T * x, const T * y) {
//!     ctaylor_rec<T, Nvar - 1>::mul(dst, x, y);
//!     ctaylor_rec<T, Nvar - 1>::mul(dst + POW2(Nvar - 1),
//!                                   x + POW2(Nvar - 1), y);
//!     ctaylor_rec<T, Nvar - 1>::mul(dst + POW2(Nvar - 1),
//!                                   x,                  y + POW2(Nvar - 1));
//!   }
//!   // Set dst = x*y
//!   static void mul_set(T * dst, const T * x, const T * y) {
//!     ctaylor_rec<T, Nvar - 1>::mul_set(dst, x, y);
//!     ctaylor_rec<T, Nvar - 1>::mul_set(dst + POW2(Nvar - 1),
//!                                       x + POW2(Nvar - 1), y);
//!     ctaylor_rec<T, Nvar - 1>::mul(dst + POW2(Nvar - 1),
//!                                   x,                  y + POW2(Nvar - 1));
//!   }
//! };
//! ```
//!
//! # Porting strategy
//!
//! Rather than recurse through `#[cube] fn`s (which would require
//! sub-slicing `&mut Array<F>` and fighting the borrow-checker), each
//! per-N specialization is **fully flattened**: every coefficient of
//! `dst[0..=2^N-1]` is written by a single explicit statement. This
//! matches what a C++ compiler produces after template instantiation
//! and guarantees the C++ summation order survives into the cubecl
//! lowering (D-08).
//!
//! All `> 2`-operand sums use left-to-right `let`-chain bindings, e.g.
//! `let s1 = a + b; let s2 = s1 + c; let d = s2 + d;`, matching the
//! C++ left-to-right associativity of `a + b + c + d`.

use cubecl::prelude::*;

// ---------------------------------------------------------------------------
//  mul_acc — dst += x * y (accumulating)
// ---------------------------------------------------------------------------

/// N=0 mul_acc. Port of `ctaylor.hpp:86` — `dst[0] += x[0] * y[0]`.
#[cube]
pub(crate) fn ctaylor_mul_acc_n0<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    let d0 = x[0] * y[0];
    dst[0] = dst[0] + d0;
}

/// N=1 mul_acc. Port of `ctaylor.hpp:94-98`.
///
/// ```cpp
/// static void mul(T * dst, const T * x, const T * y) {
///   dst[0] += x[0] * y[0];
///   dst[1] += x[0] * y[1] + x[1] * y[0];
/// }
/// ```
#[cube]
pub(crate) fn ctaylor_mul_acc_n1<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    let d0 = x[0] * y[0];
    dst[0] = dst[0] + d0;

    let t10 = x[0] * y[1];
    let t11 = x[1] * y[0];
    let d1 = t10 + t11;
    dst[1] = dst[1] + d1;
}

/// N=2 mul_acc. Port of `ctaylor.hpp:119-124`.
///
/// ```cpp
/// static void mul(T * dst, const T * x, const T * y) {
///   dst[0] += x[0] * y[0];
///   dst[1] += x[0] * y[1] + x[1] * y[0];
///   dst[2] += x[0] * y[2] + x[2] * y[0];
///   dst[3] += x[0] * y[3] + x[3] * y[0] + x[1] * y[2] + x[2] * y[1];
/// }
/// ```
#[cube]
pub(crate) fn ctaylor_mul_acc_n2<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    // dst[0]
    let d0 = x[0] * y[0];
    dst[0] = dst[0] + d0;

    // dst[1] = x[0]*y[1] + x[1]*y[0]
    let t10 = x[0] * y[1];
    let t11 = x[1] * y[0];
    let d1 = t10 + t11;
    dst[1] = dst[1] + d1;

    // dst[2] = x[0]*y[2] + x[2]*y[0]
    let t20 = x[0] * y[2];
    let t21 = x[2] * y[0];
    let d2 = t20 + t21;
    dst[2] = dst[2] + d2;

    // dst[3] = x[0]*y[3] + x[3]*y[0] + x[1]*y[2] + x[2]*y[1]
    //   C++ is left-to-right: ((a + b) + c) + d
    let t30 = x[0] * y[3];
    let t31 = x[3] * y[0];
    let t32 = x[1] * y[2];
    let t33 = x[2] * y[1];
    let s1 = t30 + t31;
    let s2 = s1 + t32;
    let d3 = s2 + t33;
    dst[3] = dst[3] + d3;
}

/// N=3 mul_acc — expanded from the 3-call recursion at `ctaylor.hpp:42-46`.
///
/// The general recursion for N=3 expands to:
/// - `mul_acc_n2(dst[0..4], x[0..4], y[0..4])`              → covers dst[0..=3]
/// - `mul_acc_n2(dst[4..8], x[4..8], y[0..4])`              → covers dst[4..=7]
/// - `mul_acc_n2(dst[4..8], x[0..4], y[4..8])`              → adds to dst[4..=7]
///
/// We flatten the two contributions to dst[4..=7] into a single expression
/// per coefficient, preserving the C++ write order: the first call writes
/// `(x[4..8] * y[0..4])[i]`, the second ADDS `(x[0..4] * y[4..8])[i]`.
/// Both calls write dst[i] = dst[i] + (all its summands, C++ left-assoc).
///
/// Each of `dst[0..=3]` receives the N=2 mul_acc of `x[0..4] * y[0..4]`.
/// Each of `dst[4..=7]` receives:
///   (a) the N=2 mul_acc result of `x[4..8] * y[0..4]`   (first call)
///   (b) PLUS the N=2 mul_acc result of `x[0..4] * y[4..8]` (second call)
///
/// Because mul_acc is ADDITIVE on dst and both (a) and (b) are
/// independent, the aggregate contribution is the sum of the two
/// mul_acc_n2 bodies. Operation order preserved: (a) written first then
/// (b) added, which C++-side is exactly two passes of `dst[i] += ...`
/// statements in the same left-to-right order.
#[cube]
pub(crate) fn ctaylor_mul_acc_n3<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    // First call: mul_acc_n2(dst[0..4], x[0..4], y[0..4]) — coeffs dst[0..=3]
    // dst[0] += x[0]*y[0]
    let a0 = x[0] * y[0];
    dst[0] = dst[0] + a0;
    // dst[1] += x[0]*y[1] + x[1]*y[0]
    let a10 = x[0] * y[1];
    let a11 = x[1] * y[0];
    let a1 = a10 + a11;
    dst[1] = dst[1] + a1;
    // dst[2] += x[0]*y[2] + x[2]*y[0]
    let a20 = x[0] * y[2];
    let a21 = x[2] * y[0];
    let a2 = a20 + a21;
    dst[2] = dst[2] + a2;
    // dst[3] += x[0]*y[3] + x[3]*y[0] + x[1]*y[2] + x[2]*y[1]
    let a30 = x[0] * y[3];
    let a31 = x[3] * y[0];
    let a32 = x[1] * y[2];
    let a33 = x[2] * y[1];
    let as1 = a30 + a31;
    let as2 = as1 + a32;
    let a3 = as2 + a33;
    dst[3] = dst[3] + a3;

    // Second call: mul_acc_n2(dst[4..8], x[4..8], y[0..4]) — coeffs dst[4..=7]
    // dst[4] += x[4]*y[0]
    let b0 = x[4] * y[0];
    dst[4] = dst[4] + b0;
    // dst[5] += x[4]*y[1] + x[5]*y[0]
    let b10 = x[4] * y[1];
    let b11 = x[5] * y[0];
    let b1 = b10 + b11;
    dst[5] = dst[5] + b1;
    // dst[6] += x[4]*y[2] + x[6]*y[0]
    let b20 = x[4] * y[2];
    let b21 = x[6] * y[0];
    let b2 = b20 + b21;
    dst[6] = dst[6] + b2;
    // dst[7] += x[4]*y[3] + x[7]*y[0] + x[5]*y[2] + x[6]*y[1]
    let b30 = x[4] * y[3];
    let b31 = x[7] * y[0];
    let b32 = x[5] * y[2];
    let b33 = x[6] * y[1];
    let bs1 = b30 + b31;
    let bs2 = bs1 + b32;
    let b3 = bs2 + b33;
    dst[7] = dst[7] + b3;

    // Third call: mul_acc_n2(dst[4..8], x[0..4], y[4..8]) — coeffs dst[4..=7]
    // dst[4] += x[0]*y[4]
    let c0 = x[0] * y[4];
    dst[4] = dst[4] + c0;
    // dst[5] += x[0]*y[5] + x[1]*y[4]
    let c10 = x[0] * y[5];
    let c11 = x[1] * y[4];
    let c1 = c10 + c11;
    dst[5] = dst[5] + c1;
    // dst[6] += x[0]*y[6] + x[2]*y[4]
    let c20 = x[0] * y[6];
    let c21 = x[2] * y[4];
    let c2 = c20 + c21;
    dst[6] = dst[6] + c2;
    // dst[7] += x[0]*y[7] + x[2]*y[5] + x[1]*y[6] + x[3]*y[4]
    // (This is mul_acc_n2's dst[3] pattern: x[0]*y[3] + x[2]*y[1] + x[1]*y[2] + x[3]*y[0]
    //  applied with x = x[0..4], y = y[4..8]: x[0]*y[4+3] + x[2]*y[4+1] + x[1]*y[4+2] + x[3]*y[4+0].)
    let c30 = x[0] * y[7];
    let c31 = x[3] * y[4];
    let c32 = x[1] * y[6];
    let c33 = x[2] * y[5];
    let cs1 = c30 + c31;
    let cs2 = cs1 + c32;
    let c3 = cs2 + c33;
    dst[7] = dst[7] + c3;
}

// ---------------------------------------------------------------------------
//  mul_set — dst = x * y (overwriting)
// ---------------------------------------------------------------------------

/// N=0 mul_set. Port of `ctaylor.hpp:87` — `dst[0] = x[0] * y[0]`.
#[cube]
pub(crate) fn ctaylor_mul_set_n0<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    dst[0] = x[0] * y[0];
}

/// N=1 mul_set. Port of `ctaylor.hpp:99-102`.
#[cube]
pub(crate) fn ctaylor_mul_set_n1<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    dst[0] = x[0] * y[0];
    let t10 = x[0] * y[1];
    let t11 = x[1] * y[0];
    dst[1] = t10 + t11;
}

/// N=2 mul_set. Port of `ctaylor.hpp:125-130`.
#[cube]
pub(crate) fn ctaylor_mul_set_n2<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    dst[0] = x[0] * y[0];

    let t10 = x[0] * y[1];
    let t11 = x[1] * y[0];
    dst[1] = t10 + t11;

    let t20 = x[0] * y[2];
    let t21 = x[2] * y[0];
    dst[2] = t20 + t21;

    let t30 = x[0] * y[3];
    let t31 = x[3] * y[0];
    let t32 = x[1] * y[2];
    let t33 = x[2] * y[1];
    let s1 = t30 + t31;
    let s2 = s1 + t32;
    dst[3] = s2 + t33;
}

/// N=3 mul_set — expanded from `ctaylor.hpp:49-52`:
///   mul_set_n2(dst[0..4], x[0..4], y[0..4])
///   mul_set_n2(dst[4..8], x[4..8], y[0..4])
///   mul_acc_n2(dst[4..8], x[0..4], y[4..8])
#[cube]
pub(crate) fn ctaylor_mul_set_n3<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    // mul_set_n2 on lower half (x[0..4] * y[0..4]) → dst[0..=3]
    dst[0] = x[0] * y[0];

    let a10 = x[0] * y[1];
    let a11 = x[1] * y[0];
    dst[1] = a10 + a11;

    let a20 = x[0] * y[2];
    let a21 = x[2] * y[0];
    dst[2] = a20 + a21;

    let a30 = x[0] * y[3];
    let a31 = x[3] * y[0];
    let a32 = x[1] * y[2];
    let a33 = x[2] * y[1];
    let as1 = a30 + a31;
    let as2 = as1 + a32;
    dst[3] = as2 + a33;

    // mul_set_n2 on upper half (x[4..8] * y[0..4]) → dst[4..=7]
    dst[4] = x[4] * y[0];

    let b10 = x[4] * y[1];
    let b11 = x[5] * y[0];
    dst[5] = b10 + b11;

    let b20 = x[4] * y[2];
    let b21 = x[6] * y[0];
    dst[6] = b20 + b21;

    let b30 = x[4] * y[3];
    let b31 = x[7] * y[0];
    let b32 = x[5] * y[2];
    let b33 = x[6] * y[1];
    let bs1 = b30 + b31;
    let bs2 = bs1 + b32;
    dst[7] = bs2 + b33;

    // mul_acc_n2 (x[0..4] * y[4..8]) added onto dst[4..=7]
    let c0 = x[0] * y[4];
    dst[4] = dst[4] + c0;

    let c10 = x[0] * y[5];
    let c11 = x[1] * y[4];
    let c1 = c10 + c11;
    dst[5] = dst[5] + c1;

    let c20 = x[0] * y[6];
    let c21 = x[2] * y[4];
    let c2 = c20 + c21;
    dst[6] = dst[6] + c2;

    let c30 = x[0] * y[7];
    let c31 = x[3] * y[4];
    let c32 = x[1] * y[6];
    let c33 = x[2] * y[5];
    let cs1 = c30 + c31;
    let cs2 = cs1 + c32;
    let c3 = cs2 + c33;
    dst[7] = dst[7] + c3;
}

// ---------------------------------------------------------------------------
//  Outer dispatch
// ---------------------------------------------------------------------------

/// Outer dispatch for `dst = x * y` across N ∈ {0, 1, 2, 3}.
///
/// Plan 01-02 ships N ≤ 3 per the validation mandate (f64::to_bits identity
/// for N ∈ 0..=3); N ∈ 4..=7 are reserved for a follow-on plan with its
/// own golden-fixture gate. The `#[comptime] n` match form with only 4
/// arms is deliberate — it keeps the N ≤ 3 contract testable and the
/// generated code free of unreachable arms.
#[cube]
pub fn ctaylor_mul<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    if comptime!(n == 0) {
        ctaylor_mul_set_n0::<F>(out, a, b);
    } else if comptime!(n == 1) {
        ctaylor_mul_set_n1::<F>(out, a, b);
    } else if comptime!(n == 2) {
        ctaylor_mul_set_n2::<F>(out, a, b);
    } else if comptime!(n == 3) {
        ctaylor_mul_set_n3::<F>(out, a, b);
    }
}
