//! `ctaylor_rec::multo` / `multo_skipconst` — in-place `dst *= y` and the
//! skip-constant variant. Per-N specialization for N ∈ 0..=3.
//!
//! Port of `xcfun-master/external/upstream/taylor/ctaylor.hpp:55-65`
//! (general recursion) + `:88-89, 103-110, 131-142` (N=0, 1, 2 base cases).
//!
//! # Algorithmic-identity mandate (CONTEXT.md D-08, RESEARCH.md Pitfall P3)
//!
//! The C++ recursion writes dst coefficients in **descending** order
//! (dst[3] before dst[0] at N=2). This is load-bearing: each `dst[i]`
//! assignment reads dst[j] for j < i, so overwriting dst[0] first would
//! feed stale values into dst[3]'s expression and break 1e-12 parity.
//!
//! # C++ general recursion (ctaylor.hpp:55-65)
//!
//! ```cpp
//! static void multo(T * dst, const T * y) {
//!   ctaylor_rec<T, Nvar - 1>::multo(dst + POW2(Nvar - 1), y);
//!   ctaylor_rec<T, Nvar - 1>::mul  (dst + POW2(Nvar - 1), dst, y + POW2(Nvar - 1));
//!   ctaylor_rec<T, Nvar - 1>::multo(dst, y);
//! }
//! static void multo_skipconst(T * dst, const T * y) {
//!   ctaylor_rec<T, Nvar - 1>::multo_skipconst(dst + POW2(Nvar - 1), y);
//!   ctaylor_rec<T, Nvar - 1>::mul  (dst + POW2(Nvar - 1), dst, y + POW2(Nvar - 1));
//!   ctaylor_rec<T, Nvar - 1>::multo_skipconst(dst, y);
//! }
//! ```
//!
//! # Porting strategy
//!
//! Each per-N specialization is fully flattened: every coefficient of
//! `dst[0..=2^N-1]` is read once into a local `let` (capturing its
//! pre-assignment value), then all coefficients are assigned in the
//! C++ descending-write order. This removes the borrow-checker headache
//! of aliasing `&mut Array<F>` sub-slices and makes the summation order
//! inspectable as pure straight-line code.

use cubecl::prelude::*;

// ---------------------------------------------------------------------------
//  multo — dst = dst * y
// ---------------------------------------------------------------------------

/// N=0 multo. Port of `ctaylor.hpp:88` — `dst[0] *= y[0]`.
#[cube]
pub(crate) fn ctaylor_multo_n0<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    dst[0] = dst[0] * y[0];
}

/// N=1 multo. Port of `ctaylor.hpp:103-106`.
///
/// ```cpp
/// static void multo(T * dst, const T * y) {
///   dst[1] = dst[1] * y[0] + dst[0] * y[1];   // BEFORE dst[0]
///   dst[0] *= y[0];
/// }
/// ```
#[cube]
pub(crate) fn ctaylor_multo_n1<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    // Capture dst[0] and dst[1] pre-update so the descending write-order
    // semantics survive even when the compiler reorders reads.
    let d0 = dst[0];
    let d1 = dst[1];

    // dst[1] = d1 * y[0] + d0 * y[1]   (C++ left-assoc; only 2 operands)
    let t1a = d1 * y[0];
    let t1b = d0 * y[1];
    dst[1] = t1a + t1b;

    // dst[0] = d0 * y[0]
    dst[0] = d0 * y[0];
}

/// N=2 multo. Port of `ctaylor.hpp:131-135` — the load-bearing base case.
///
/// ```cpp
/// static void multo(T * dst, const T * y) {
///   dst[3] = dst[0]*y[3] + dst[3]*y[0] + dst[1]*y[2] + dst[2]*y[1];
///   dst[2] = dst[0]*y[2] + dst[2]*y[0];
///   dst[1] = dst[0]*y[1] + dst[1]*y[0];
///   dst[0] = dst[0]*y[0];
/// }
/// ```
#[cube]
pub(crate) fn ctaylor_multo_n2<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];
    let d3 = dst[3];

    // dst[3] = d0*y[3] + d3*y[0] + d1*y[2] + d2*y[1]   (C++ left-assoc)
    let t30 = d0 * y[3];
    let t31 = d3 * y[0];
    let t32 = d1 * y[2];
    let t33 = d2 * y[1];
    let s1 = t30 + t31;
    let s2 = s1 + t32;
    dst[3] = s2 + t33;

    // dst[2] = d0*y[2] + d2*y[0]
    let t20 = d0 * y[2];
    let t21 = d2 * y[0];
    dst[2] = t20 + t21;

    // dst[1] = d0*y[1] + d1*y[0]
    let t10 = d0 * y[1];
    let t11 = d1 * y[0];
    dst[1] = t10 + t11;

    // dst[0] = d0 * y[0]
    dst[0] = d0 * y[0];
}

/// N=3 multo — expanded from `ctaylor.hpp:55-59`:
///
/// ```cpp
/// multo_n2(dst[4..8], y[0..4])               // step A — upper-half multo
/// mul_acc_n2(dst[4..8], dst[0..4], y[4..8])  // step B — cross-term
/// multo_n2(dst[0..4], y[0..4])               // step C — lower-half multo
/// ```
///
/// The lower half dst[0..=3] is read by step B (as the `x` argument of
/// `mul_acc`) before step C overwrites it. We capture all pre-update
/// values of dst[0..=7] into locals, then write coefficients in
/// C++-descending order: dst[7], dst[6], dst[5], dst[4], dst[3], dst[2],
/// dst[1], dst[0].
///
/// Each dst[k] expression fuses all summands contributed by steps A, B, C
/// at that index, using the original (pre-multo) dst values captured above.
#[cube]
pub(crate) fn ctaylor_multo_n3<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    // Snapshot original dst[0..=7] pre-multiplication.
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];
    let d3 = dst[3];
    let d4 = dst[4];
    let d5 = dst[5];
    let d6 = dst[6];
    let d7 = dst[7];

    // ----- Upper half dst[4..=7] -----
    //
    // Step A: multo_n2 on dst[4..8] with y[0..4] gives the "u" partial
    //   u7 = d4*y[3] + d7*y[0] + d5*y[2] + d6*y[1]
    //   u6 = d4*y[2] + d6*y[0]
    //   u5 = d4*y[1] + d5*y[0]
    //   u4 = d4*y[0]
    //
    // Step B: mul_acc_n2(dst[4..8], dst[0..4], y[4..8]) adds to dst[4..=7]
    //   the pattern mul_acc_n2 produces on input (d0..d3, y[4..8]):
    //   b7 = d0*y[7] + d3*y[4] + d1*y[6] + d2*y[5]
    //   b6 = d0*y[6] + d2*y[4]
    //   b5 = d0*y[5] + d1*y[4]
    //   b4 = d0*y[4]
    //
    // Descending write-order: dst[7] first.

    // dst[7]: C++ left-to-right accumulates the 4 u7 terms THEN the 4 b7 terms
    //   (multo_n2 writes dst[3] first; then mul_acc_n2 runs dst[0]+= .., dst[1]+= .., dst[2]+= .., dst[3]+= .. in i-ascending order).
    //   So C++ operation order for dst[7] is:
    //     u_sum = ((d4*y[3] + d7*y[0]) + d5*y[2]) + d6*y[1]
    //     final = (((u_sum + d0*y[7]) + d3*y[4]) + d1*y[6]) + d2*y[5]
    //   i.e. left-to-right fold of 8 products.
    let u7_a = d4 * y[3];
    let u7_b = d7 * y[0];
    let u7_c = d5 * y[2];
    let u7_d = d6 * y[1];
    let u7_s1 = u7_a + u7_b;
    let u7_s2 = u7_s1 + u7_c;
    let u7_s3 = u7_s2 + u7_d;
    let b7_a = d0 * y[7];
    let b7_b = d3 * y[4];
    let b7_c = d1 * y[6];
    let b7_d = d2 * y[5];
    let f7_s1 = u7_s3 + b7_a;
    let f7_s2 = f7_s1 + b7_b;
    let f7_s3 = f7_s2 + b7_c;
    dst[7] = f7_s3 + b7_d;

    // dst[6]: u6 = d4*y[2] + d6*y[0]; plus b6 = d0*y[6] + d2*y[4].
    //   C++ left-to-right: ((d4*y[2] + d6*y[0]) + d0*y[6]) + d2*y[4]
    let u6_a = d4 * y[2];
    let u6_b = d6 * y[0];
    let b6_a = d0 * y[6];
    let b6_b = d2 * y[4];
    let f6_s1 = u6_a + u6_b;
    let f6_s2 = f6_s1 + b6_a;
    dst[6] = f6_s2 + b6_b;

    // dst[5]: u5 = d4*y[1] + d5*y[0]; plus b5 = d0*y[5] + d1*y[4].
    let u5_a = d4 * y[1];
    let u5_b = d5 * y[0];
    let b5_a = d0 * y[5];
    let b5_b = d1 * y[4];
    let f5_s1 = u5_a + u5_b;
    let f5_s2 = f5_s1 + b5_a;
    dst[5] = f5_s2 + b5_b;

    // dst[4]: u4 = d4*y[0]; plus b4 = d0*y[4].
    let u4_a = d4 * y[0];
    let b4_a = d0 * y[4];
    dst[4] = u4_a + b4_a;

    // ----- Lower half dst[0..=3] -----
    //
    // Step C: multo_n2 on dst[0..4] with y[0..4]. Writes descending.
    //   dst[3] = d0*y[3] + d3*y[0] + d1*y[2] + d2*y[1]
    //   dst[2] = d0*y[2] + d2*y[0]
    //   dst[1] = d0*y[1] + d1*y[0]
    //   dst[0] = d0*y[0]
    let l3_a = d0 * y[3];
    let l3_b = d3 * y[0];
    let l3_c = d1 * y[2];
    let l3_d = d2 * y[1];
    let l3_s1 = l3_a + l3_b;
    let l3_s2 = l3_s1 + l3_c;
    dst[3] = l3_s2 + l3_d;

    let l2_a = d0 * y[2];
    let l2_b = d2 * y[0];
    dst[2] = l2_a + l2_b;

    let l1_a = d0 * y[1];
    let l1_b = d1 * y[0];
    dst[1] = l1_a + l1_b;

    dst[0] = d0 * y[0];
}

// ---------------------------------------------------------------------------
//  multo_skipconst — dst = dst * (y - y[0])   (treats y[0] as 0)
// ---------------------------------------------------------------------------

/// N=0 multo_skipconst. Port of `ctaylor.hpp:89` — `dst[0] = 0`.
#[cube]
pub(crate) fn ctaylor_multo_skipconst_n0<F: Float>(
    dst: &mut Array<F>,
    _y: &Array<F>,
) {
    dst[0] = F::new(0.0);
}

/// N=1 multo_skipconst. Port of `ctaylor.hpp:107-110`.
///
/// ```cpp
/// static void multo_skipconst(T * dst, const T * y) {
///   dst[1] = dst[0] * y[1];
///   dst[0] = 0;
/// }
/// ```
#[cube]
pub(crate) fn ctaylor_multo_skipconst_n1<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
) {
    let d0 = dst[0];
    dst[1] = d0 * y[1];
    dst[0] = F::new(0.0);
}

/// N=2 multo_skipconst. Port of `ctaylor.hpp:137-142`.
///
/// ```cpp
/// static void multo_skipconst(T * dst, const T * y) {
///   dst[3] = dst[0]*y[3] + dst[1]*y[2] + dst[2]*y[1];
///   dst[2] = dst[0]*y[2];
///   dst[1] = dst[0]*y[1];
///   dst[0] = 0;
/// }
/// ```
#[cube]
pub(crate) fn ctaylor_multo_skipconst_n2<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
) {
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];

    // dst[3] = d0*y[3] + d1*y[2] + d2*y[1]    (C++ left-assoc)
    let t30 = d0 * y[3];
    let t31 = d1 * y[2];
    let t32 = d2 * y[1];
    let s1 = t30 + t31;
    dst[3] = s1 + t32;

    dst[2] = d0 * y[2];
    dst[1] = d0 * y[1];
    dst[0] = F::new(0.0);
}

/// N=3 multo_skipconst — expanded from `ctaylor.hpp:61-64`:
///
/// ```cpp
/// multo_skipconst_n2(dst[4..8], y[0..4])     // step A — upper-half skip
/// mul_acc_n2(dst[4..8], dst[0..4], y[4..8])  // step B — cross-term
/// multo_skipconst_n2(dst[0..4], y[0..4])     // step C — lower-half skip
/// ```
///
/// "skipconst" semantics: treat y[0]=0. So step A's N=2 skipconst on
/// dst[4..=7] with y[0..=3] writes only y[1], y[2], y[3] products. Same
/// goes for step C on dst[0..=3].
///
/// Descending order, using pre-captured d0..d7.
#[cube]
pub(crate) fn ctaylor_multo_skipconst_n3<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
) {
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];
    let d3 = dst[3];
    let d4 = dst[4];
    let d5 = dst[5];
    let d6 = dst[6];

    // ----- Upper half dst[4..=7] -----
    //
    // Step A (multo_skipconst_n2 on dst[4..8], y[0..4]):
    //   u7 = d4*y[3] + d5*y[2] + d6*y[1]
    //   u6 = d4*y[2]
    //   u5 = d4*y[1]
    //   u4 = 0
    //
    // Step B (mul_acc_n2 on (dst[4..8], dst[0..4], y[4..8])):
    //   b7 = d0*y[7] + d3*y[4] + d1*y[6] + d2*y[5]
    //   b6 = d0*y[6] + d2*y[4]
    //   b5 = d0*y[5] + d1*y[4]
    //   b4 = d0*y[4]
    //
    // C++ left-to-right order: step A writes dst[7] before B modifies it,
    //   so final dst[k] = left_assoc_fold(step-A terms, then step-B terms).

    // dst[7]: u7 (3 terms) then b7 (4 terms), all left-to-right.
    let u7_a = d4 * y[3];
    let u7_b = d5 * y[2];
    let u7_c = d6 * y[1];
    let u7_s1 = u7_a + u7_b;
    let u7_s2 = u7_s1 + u7_c;
    let b7_a = d0 * y[7];
    let b7_b = d3 * y[4];
    let b7_c = d1 * y[6];
    let b7_d = d2 * y[5];
    let f7_s1 = u7_s2 + b7_a;
    let f7_s2 = f7_s1 + b7_b;
    let f7_s3 = f7_s2 + b7_c;
    dst[7] = f7_s3 + b7_d;

    // dst[6]: u6 (1 term) then b6 (2 terms).
    let u6_a = d4 * y[2];
    let b6_a = d0 * y[6];
    let b6_b = d2 * y[4];
    let f6_s1 = u6_a + b6_a;
    dst[6] = f6_s1 + b6_b;

    // dst[5]: u5 (1 term) then b5 (2 terms).
    let u5_a = d4 * y[1];
    let b5_a = d0 * y[5];
    let b5_b = d1 * y[4];
    let f5_s1 = u5_a + b5_a;
    dst[5] = f5_s1 + b5_b;

    // dst[4]: u4 = 0 (skipconst), then b4 = d0*y[4].
    //   C++ writes 0 then accumulates: final = 0 + d0*y[4] = d0*y[4].
    dst[4] = d0 * y[4];

    // ----- Lower half dst[0..=3] -----
    //
    // Step C (multo_skipconst_n2 on dst[0..4], y[0..4]):
    //   dst[3] = d0*y[3] + d1*y[2] + d2*y[1]
    //   dst[2] = d0*y[2]
    //   dst[1] = d0*y[1]
    //   dst[0] = 0
    let l3_a = d0 * y[3];
    let l3_b = d1 * y[2];
    let l3_c = d2 * y[1];
    let l3_s1 = l3_a + l3_b;
    dst[3] = l3_s1 + l3_c;

    dst[2] = d0 * y[2];
    dst[1] = d0 * y[1];
    dst[0] = F::new(0.0);
}

// ---------------------------------------------------------------------------
//  Outer dispatch
// ---------------------------------------------------------------------------

/// Outer dispatch for `dst *= y` across N ∈ {0, 1, 2, 3}.
#[cube]
pub fn ctaylor_multo<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
    #[comptime] n: u32,
) {
    if comptime!(n == 0) {
        ctaylor_multo_n0::<F>(dst, y);
    } else if comptime!(n == 1) {
        ctaylor_multo_n1::<F>(dst, y);
    } else if comptime!(n == 2) {
        ctaylor_multo_n2::<F>(dst, y);
    } else if comptime!(n == 3) {
        ctaylor_multo_n3::<F>(dst, y);
    }
}

/// Outer dispatch for `dst *= (y - y[0])` across N ∈ {0, 1, 2, 3}.
#[cube]
pub fn ctaylor_multo_skipconst<F: Float>(
    dst: &mut Array<F>,
    y: &Array<F>,
    #[comptime] n: u32,
) {
    if comptime!(n == 0) {
        ctaylor_multo_skipconst_n0::<F>(dst, y);
    } else if comptime!(n == 1) {
        ctaylor_multo_skipconst_n1::<F>(dst, y);
    } else if comptime!(n == 2) {
        ctaylor_multo_skipconst_n2::<F>(dst, y);
    } else if comptime!(n == 3) {
        ctaylor_multo_skipconst_n3::<F>(dst, y);
    }
}
