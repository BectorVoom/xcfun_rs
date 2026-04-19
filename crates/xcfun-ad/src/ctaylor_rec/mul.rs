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

/// N=4 mul_set — expanded from `ctaylor.hpp:49-52` applied one more level:
///   mul_set_n3(dst[0..8],  x[0..8],  y[0..8])   → sets   dst[0..=7]
///   mul_set_n3(dst[8..16], x[8..16], y[0..8])   → sets   dst[8..=15]
///   mul_acc_n3(dst[8..16], x[0..8],  y[8..16])  → adds to dst[8..=15]
///
/// Each sub-call is in turn the flattened n=3 body (itself a flattening of
/// the n=2 recursion). The three sub-bodies are concatenated here in the
/// C++ left-to-right traversal order: first the `mul_set_n3` on the lower
/// half, then the `mul_set_n3` on the upper half (dst indices 8..=15),
/// then the `mul_acc_n3` adding onto dst indices 8..=15. Preserves the
/// C++ operation order at the 1e-12 parity gate (D-08). Added in Plan
/// 01-05 to unblock the n_var=4 golden-fixture gate (relative-error
/// tolerance 1e-13, not bit-exact).
#[cube]
pub(crate) fn ctaylor_mul_set_n4<F: Float>(
    dst: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
) {
    // =========================================================================
    //  Part 1: mul_set_n3(dst[0..8], x[0..8], y[0..8]) — coeffs dst[0..=7]
    //    (verbatim copy of ctaylor_mul_set_n3 body)
    // =========================================================================

    // mul_set_n2 on lower-quarter (x[0..4] * y[0..4]) → dst[0..=3]
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

    // mul_set_n2 on next-quarter (x[4..8] * y[0..4]) → dst[4..=7]
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

    // =========================================================================
    //  Part 2: mul_set_n3(dst[8..16], x[8..16], y[0..8]) — coeffs dst[8..=15]
    //    (mul_set_n3 body with x → x[8..16], y → y[0..8], dst → dst[8..16])
    // =========================================================================

    // mul_set_n2 on dst[8..=11] (x[8..=11] * y[0..=3])
    dst[8] = x[8] * y[0];

    let d10 = x[8] * y[1];
    let d11 = x[9] * y[0];
    dst[9] = d10 + d11;

    let d20 = x[8] * y[2];
    let d21 = x[10] * y[0];
    dst[10] = d20 + d21;

    let d30 = x[8] * y[3];
    let d31 = x[11] * y[0];
    let d32 = x[9] * y[2];
    let d33 = x[10] * y[1];
    let ds1 = d30 + d31;
    let ds2 = ds1 + d32;
    dst[11] = ds2 + d33;

    // mul_set_n2 on dst[12..=15] (x[12..=15] * y[0..=3])
    dst[12] = x[12] * y[0];

    let e10 = x[12] * y[1];
    let e11 = x[13] * y[0];
    dst[13] = e10 + e11;

    let e20 = x[12] * y[2];
    let e21 = x[14] * y[0];
    dst[14] = e20 + e21;

    let e30 = x[12] * y[3];
    let e31 = x[15] * y[0];
    let e32 = x[13] * y[2];
    let e33 = x[14] * y[1];
    let es1 = e30 + e31;
    let es2 = es1 + e32;
    dst[15] = es2 + e33;

    // mul_acc_n2 (x[8..=11] * y[4..=7]) onto dst[12..=15]
    let f0 = x[8] * y[4];
    dst[12] = dst[12] + f0;

    let f10 = x[8] * y[5];
    let f11 = x[9] * y[4];
    let f1 = f10 + f11;
    dst[13] = dst[13] + f1;

    let f20 = x[8] * y[6];
    let f21 = x[10] * y[4];
    let f2 = f20 + f21;
    dst[14] = dst[14] + f2;

    let f30 = x[8] * y[7];
    let f31 = x[11] * y[4];
    let f32 = x[9] * y[6];
    let f33 = x[10] * y[5];
    let fs1 = f30 + f31;
    let fs2 = fs1 + f32;
    let f3 = fs2 + f33;
    dst[15] = dst[15] + f3;

    // =========================================================================
    //  Part 3: mul_acc_n3(dst[8..16], x[0..8], y[8..16]) — adds to dst[8..=15]
    //    (mul_acc_n3 body with x → x[0..8], y → y[8..16], dst → dst[8..16])
    // =========================================================================

    // mul_acc_n2(dst[8..=11], x[0..=3], y[8..=11])
    // dst[8] += x[0] * y[8]
    let g0 = x[0] * y[8];
    dst[8] = dst[8] + g0;
    // dst[9] += x[0]*y[9] + x[1]*y[8]
    let g10 = x[0] * y[9];
    let g11 = x[1] * y[8];
    let g1 = g10 + g11;
    dst[9] = dst[9] + g1;
    // dst[10] += x[0]*y[10] + x[2]*y[8]
    let g20 = x[0] * y[10];
    let g21 = x[2] * y[8];
    let g2 = g20 + g21;
    dst[10] = dst[10] + g2;
    // dst[11] += x[0]*y[11] + x[3]*y[8] + x[1]*y[10] + x[2]*y[9]
    let g30 = x[0] * y[11];
    let g31 = x[3] * y[8];
    let g32 = x[1] * y[10];
    let g33 = x[2] * y[9];
    let gs1 = g30 + g31;
    let gs2 = gs1 + g32;
    let g3 = gs2 + g33;
    dst[11] = dst[11] + g3;

    // mul_acc_n2(dst[12..=15], x[4..=7], y[8..=11])
    let h0 = x[4] * y[8];
    dst[12] = dst[12] + h0;

    let h10 = x[4] * y[9];
    let h11 = x[5] * y[8];
    let h1 = h10 + h11;
    dst[13] = dst[13] + h1;

    let h20 = x[4] * y[10];
    let h21 = x[6] * y[8];
    let h2 = h20 + h21;
    dst[14] = dst[14] + h2;

    let h30 = x[4] * y[11];
    let h31 = x[7] * y[8];
    let h32 = x[5] * y[10];
    let h33 = x[6] * y[9];
    let hs1 = h30 + h31;
    let hs2 = hs1 + h32;
    let h3 = hs2 + h33;
    dst[15] = dst[15] + h3;

    // mul_acc_n2(dst[12..=15], x[0..=3], y[12..=15])
    let i0 = x[0] * y[12];
    dst[12] = dst[12] + i0;

    let i10 = x[0] * y[13];
    let i11 = x[1] * y[12];
    let i1 = i10 + i11;
    dst[13] = dst[13] + i1;

    let i20 = x[0] * y[14];
    let i21 = x[2] * y[12];
    let i2 = i20 + i21;
    dst[14] = dst[14] + i2;

    let i30 = x[0] * y[15];
    let i31 = x[3] * y[12];
    let i32 = x[1] * y[14];
    let i33 = x[2] * y[13];
    let is1 = i30 + i31;
    let is2 = is1 + i32;
    let i3 = is2 + i33;
    dst[15] = dst[15] + i3;
}

// ---------------------------------------------------------------------------
//  Outer dispatch
// ---------------------------------------------------------------------------

/// Outer dispatch for `dst = x * y` across N ∈ {0, 1, 2, 3, 4}.
///
/// Plan 01-02 shipped N ≤ 3 per its validation mandate (f64::to_bits
/// identity for N ∈ 0..=3). Plan 01-05 extends to N = 4 (relative-error
/// tolerance 1e-13, not bit-exact) so the mul golden-fixture gate covers
/// the full range emitted by the fixture driver. N ∈ 5..=7 still deferred.
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
    } else if comptime!(n == 4) {
        ctaylor_mul_set_n4::<F>(out, a, b);
    }
}
