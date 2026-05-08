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
pub(crate) fn ctaylor_multo_skipconst_n0<F: Float>(dst: &mut Array<F>, _y: &Array<F>) {
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
pub(crate) fn ctaylor_multo_skipconst_n1<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
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
pub(crate) fn ctaylor_multo_skipconst_n2<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
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
pub(crate) fn ctaylor_multo_skipconst_n3<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
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

/// N=4 multo — Phase 6 Plan 06-00 Task 1 (D-19 Phase-4 forward).
///
/// Expanded from `ctaylor.hpp:55-65` recursion at one more level:
///   multo_n3(dst[8..16], y[0..8])               // step A — upper-half multo
///   mul_acc_n3(dst[8..16], dst[0..8], y[8..16]) // step B — cross-term
///   multo_n3(dst[0..8], y[0..8])                // step C — lower-half multo
///
/// **Aliasing strategy:** capture all 16 dst values into pre-update locals
/// `d0..d15` BEFORE any writes, mirroring the N=3 multo and `mul_set_n4`
/// patterns. Once captured, the new dst values can be written in any order
/// since the source data is fully snapshotted into locals — but we preserve
/// the C++-descending write order (dst[15] first, dst[0] last) to keep the
/// per-coefficient accumulation order matching the recursion's left-to-right
/// fold pattern (D-08 algorithmic-identity contract).
///
/// **Bit-mask convolution semantics:** for each output coefficient
/// `new_dst[i]`, the multilinear-polynomial product gives
///   new_dst[i] = Σ over (j, k) with j|k = i and j&k = 0 of d_j * y_k
/// All 16 outputs follow this pattern; the per-coefficient summation order
/// is left-to-right per C++.
#[cube]
pub fn ctaylor_multo_n4<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    // Snapshot all 16 dst values pre-multiplication.
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];
    let d3 = dst[3];
    let d4 = dst[4];
    let d5 = dst[5];
    let d6 = dst[6];
    let d7 = dst[7];
    let d8 = dst[8];
    let d9 = dst[9];
    let d10 = dst[10];
    let d11 = dst[11];
    let d12 = dst[12];
    let d13 = dst[13];
    let d14 = dst[14];
    let d15 = dst[15];

    // ============================================================
    // Upper half dst[8..=15] — step A (multo_n3 on dst[8..16] *= y[0..8])
    //                       + step B (mul_acc_n3 on dst[8..16] += d[0..8] * y[8..16])
    // ============================================================
    //
    // Each new_dst[i] (i ∈ 8..=15) is the C++ left-to-right accumulation of:
    //   step-A terms (multo_n3 sub-pattern with d[8..16] × y[0..8]),
    //   then step-B terms (mul_acc_n3 sub-pattern adding d[0..8] × y[8..16]).
    //
    // The step-A multo_n3 body for dst[i] for i in upper half is the
    // 4-bit-popcount sum (8 terms for i=15; 4 for i ∈ {14,13,11};
    // 2 for i ∈ {12,10,9}; 1 for i=8 — restricted to k where k & 8 = 0).
    //
    // The step-B mul_acc_n3 body for dst[i] is the dual: same shape, but
    // (j, k) split as j ∈ [0..8], k ∈ [8..16] with j+k=i.

    // dst[15] — 16 cross-terms total; popcount(15) = 4 → 16 (j,k) pairs.
    // Step-A (multo_n3 of dst[8..16], y[0..8]): contributes (j,k) with
    //   j ∈ {8..15}, k ∈ {0..7}, j&k=0, j|k=15. That's:
    //     (15,0), (14,1), (13,2), (12,3), (11,4), (10,5), (9,6), (8,7).
    //   The N=3 multo "u7" pattern in dst[7] (the upper-half N=2 multo
    //   from N=3): d4*y[3] + d7*y[0] + d5*y[2] + d6*y[1]
    //   Translated to N=4 upper half: d12*y[3] + d15*y[0] + d13*y[2] + d14*y[1]
    //   ALSO from the N=3 mul_acc applied during step-A's inner cross-term:
    //   d8*y[7] + d11*y[4] + d9*y[6] + d10*y[5]
    //   Combined: 8 terms in C++ left-to-right multo_n3 order.
    //
    //   N=3 multo's dst[7] sequence (step A then B, see multo_n3 body):
    //     u7  = d4*y[3] + d7*y[0] + d5*y[2] + d6*y[1]   (4 terms)
    //     b7  = d0*y[7] + d3*y[4] + d1*y[6] + d2*y[5]   (4 terms)
    //   Promoted to N=4 (multo_n3 inside upper half: indices shift by 8 on
    //   one side; the "x"≡captured dst becomes d[8..16] in step A's lower
    //   recursion):
    //     u15 = d12*y[3] + d15*y[0] + d13*y[2] + d14*y[1]
    //     b15 = d8 *y[7] + d11*y[4] + d9 *y[6] + d10*y[5]
    //
    // Step-B (mul_acc_n3 of d[0..8], y[8..16] onto dst[8..16]) for dst[15]:
    //     mul_acc_n3 body's dst[7]-equivalent with d→d[0..8], y→y[8..16]:
    //     N=3 mul_acc body for dst[7] (= step A's u7 shape but additive,
    //     plus mul_acc_n2 cross-terms at d[0..4]*y[12..16]):
    //
    //   mul_acc_n3 dst[7] = d4*y[3] + d7*y[0] + d5*y[2] + d6*y[1]    (1st pass)
    //                     + d0*y[7] + d3*y[4] + d1*y[6] + d2*y[5]    (2nd/3rd pass)
    //   With y → y[8..16], d → d[0..8] (step B's argument):
    //     c15 = d4 *y[11] + d7 *y[8]  + d5 *y[10] + d6 *y[9]
    //         + d0 *y[15] + d3 *y[12] + d1 *y[14] + d2 *y[13]
    //
    // Final dst[15] = u15 +ASSOC b15 +ASSOC c15  (left-to-right, 16 terms).
    let u15_a = d12 * y[3];
    let u15_b = d15 * y[0];
    let u15_c = d13 * y[2];
    let u15_d = d14 * y[1];
    let u15_s1 = u15_a + u15_b;
    let u15_s2 = u15_s1 + u15_c;
    let u15_s3 = u15_s2 + u15_d;
    let b15_a = d8 * y[7];
    let b15_b = d11 * y[4];
    let b15_c = d9 * y[6];
    let b15_d = d10 * y[5];
    let f15_s1 = u15_s3 + b15_a;
    let f15_s2 = f15_s1 + b15_b;
    let f15_s3 = f15_s2 + b15_c;
    let f15_s4 = f15_s3 + b15_d;
    // Step-B pass: mul_acc_n3 contribution.
    let c15_a = d4 * y[11];
    let c15_b = d7 * y[8];
    let c15_c = d5 * y[10];
    let c15_d = d6 * y[9];
    let g15_s1 = f15_s4 + c15_a;
    let g15_s2 = g15_s1 + c15_b;
    let g15_s3 = g15_s2 + c15_c;
    let g15_s4 = g15_s3 + c15_d;
    let c15_e = d0 * y[15];
    let c15_f = d3 * y[12];
    let c15_g = d1 * y[14];
    let c15_h = d2 * y[13];
    let h15_s1 = g15_s4 + c15_e;
    let h15_s2 = h15_s1 + c15_f;
    let h15_s3 = h15_s2 + c15_g;
    dst[15] = h15_s3 + c15_h;

    // dst[14]: popcount=3 → 8 terms. (j|k)=14, (j&k)=0.
    //   step-A (d[8..16] × y[0..8]): pairs (14,0), (12,2), (10,4), (8,6).
    //     N=3 multo's dst[6]: u6 = d4*y[2] + d6*y[0]; b6 = d0*y[6] + d2*y[4]
    //     Promoted: u14 = d12*y[2] + d14*y[0], b14 = d8*y[6] + d10*y[4]
    //   step-B (d[0..8] × y[8..16]): pairs (6,8), (4,10), (2,12), (0,14).
    //     mul_acc_n3 dst[6] with shifted: c14 = d4*y[10] + d6*y[8] + d0*y[14] + d2*y[12]
    let u14_a = d12 * y[2];
    let u14_b = d14 * y[0];
    let b14_a = d8 * y[6];
    let b14_b = d10 * y[4];
    let f14_s1 = u14_a + u14_b;
    let f14_s2 = f14_s1 + b14_a;
    let f14_s3 = f14_s2 + b14_b;
    let c14_a = d4 * y[10];
    let c14_b = d6 * y[8];
    let c14_c = d0 * y[14];
    let c14_d = d2 * y[12];
    let g14_s1 = f14_s3 + c14_a;
    let g14_s2 = g14_s1 + c14_b;
    let g14_s3 = g14_s2 + c14_c;
    dst[14] = g14_s3 + c14_d;

    // dst[13]: popcount=3 → 8 terms. (j|k)=13, (j&k)=0.
    //   step-A pairs (13,0),(12,1),(9,4),(8,5).
    //     u13 = d12*y[1] + d13*y[0]; b13 = d8*y[5] + d9*y[4]
    //   step-B pairs (5,8),(4,9),(1,12),(0,13).
    //     c13 = d4*y[9] + d5*y[8] + d0*y[13] + d1*y[12]
    let u13_a = d12 * y[1];
    let u13_b = d13 * y[0];
    let b13_a = d8 * y[5];
    let b13_b = d9 * y[4];
    let f13_s1 = u13_a + u13_b;
    let f13_s2 = f13_s1 + b13_a;
    let f13_s3 = f13_s2 + b13_b;
    let c13_a = d4 * y[9];
    let c13_b = d5 * y[8];
    let c13_c = d0 * y[13];
    let c13_d = d1 * y[12];
    let g13_s1 = f13_s3 + c13_a;
    let g13_s2 = g13_s1 + c13_b;
    let g13_s3 = g13_s2 + c13_c;
    dst[13] = g13_s3 + c13_d;

    // dst[12]: popcount=2 → 4 terms.
    //   step-A pairs (12,0),(8,4); u12 = d12*y[0]; b12 = d8*y[4].
    //   step-B pairs (4,8),(0,12); c12 = d4*y[8] + d0*y[12].
    let u12_a = d12 * y[0];
    let b12_a = d8 * y[4];
    let f12_s1 = u12_a + b12_a;
    let c12_a = d4 * y[8];
    let c12_b = d0 * y[12];
    let g12_s1 = f12_s1 + c12_a;
    dst[12] = g12_s1 + c12_b;

    // dst[11]: popcount=3 → 8 terms.
    //   step-A pairs (11,0),(10,1),(9,2),(8,3).
    //     N=3 multo's dst[3] inside upper half (lower-N=2 multo of d[8..12], y[0..4]):
    //     u11 = d8*y[3] + d11*y[0] + d9*y[2] + d10*y[1]   (left-to-right, 4 terms)
    //   step-B pairs (3,8),(2,9),(1,10),(0,11).
    //     c11 = d0*y[11] + d3*y[8] + d1*y[10] + d2*y[9]
    let u11_a = d8 * y[3];
    let u11_b = d11 * y[0];
    let u11_c = d9 * y[2];
    let u11_d = d10 * y[1];
    let f11_s1 = u11_a + u11_b;
    let f11_s2 = f11_s1 + u11_c;
    let f11_s3 = f11_s2 + u11_d;
    let c11_a = d0 * y[11];
    let c11_b = d3 * y[8];
    let c11_c = d1 * y[10];
    let c11_d = d2 * y[9];
    let g11_s1 = f11_s3 + c11_a;
    let g11_s2 = g11_s1 + c11_b;
    let g11_s3 = g11_s2 + c11_c;
    dst[11] = g11_s3 + c11_d;

    // dst[10]: popcount=2 → 4 terms.
    //   step-A pairs (10,0),(8,2). u10 = d8*y[2] + d10*y[0]
    //   step-B pairs (2,8),(0,10). c10 = d0*y[10] + d2*y[8]
    let u10_a = d8 * y[2];
    let u10_b = d10 * y[0];
    let f10_s1 = u10_a + u10_b;
    let c10_a = d0 * y[10];
    let c10_b = d2 * y[8];
    let g10_s1 = f10_s1 + c10_a;
    dst[10] = g10_s1 + c10_b;

    // dst[9]: popcount=2 → 4 terms.
    //   step-A pairs (9,0),(8,1). u9 = d8*y[1] + d9*y[0]
    //   step-B pairs (1,8),(0,9). c9 = d0*y[9] + d1*y[8]
    let u9_a = d8 * y[1];
    let u9_b = d9 * y[0];
    let f9_s1 = u9_a + u9_b;
    let c9_a = d0 * y[9];
    let c9_b = d1 * y[8];
    let g9_s1 = f9_s1 + c9_a;
    dst[9] = g9_s1 + c9_b;

    // dst[8]: popcount=1 → 2 terms.
    //   step-A pair (8,0). u8 = d8*y[0]
    //   step-B pair (0,8). c8 = d0*y[8]
    let u8_a = d8 * y[0];
    let c8_a = d0 * y[8];
    dst[8] = u8_a + c8_a;

    // ============================================================
    // Lower half dst[0..=7] — step C (multo_n3 on dst[0..8] *= y[0..8])
    // ============================================================
    //
    // Each new_dst[i] for i in 0..=7 has only step-C contributions (the
    // step-A and step-B did not touch the lower half).
    //
    // Body is N=3 multo's body with d[0..8] / y[0..8] — verbatim.

    // dst[7] = u7 + b7 (8 terms, identical to multo_n3 dst[7]).
    let l7_u_a = d4 * y[3];
    let l7_u_b = d7 * y[0];
    let l7_u_c = d5 * y[2];
    let l7_u_d = d6 * y[1];
    let l7_us1 = l7_u_a + l7_u_b;
    let l7_us2 = l7_us1 + l7_u_c;
    let l7_us3 = l7_us2 + l7_u_d;
    let l7_b_a = d0 * y[7];
    let l7_b_b = d3 * y[4];
    let l7_b_c = d1 * y[6];
    let l7_b_d = d2 * y[5];
    let l7_fs1 = l7_us3 + l7_b_a;
    let l7_fs2 = l7_fs1 + l7_b_b;
    let l7_fs3 = l7_fs2 + l7_b_c;
    dst[7] = l7_fs3 + l7_b_d;

    // dst[6]: u6 + b6.
    let l6_u_a = d4 * y[2];
    let l6_u_b = d6 * y[0];
    let l6_b_a = d0 * y[6];
    let l6_b_b = d2 * y[4];
    let l6_fs1 = l6_u_a + l6_u_b;
    let l6_fs2 = l6_fs1 + l6_b_a;
    dst[6] = l6_fs2 + l6_b_b;

    // dst[5]: u5 + b5.
    let l5_u_a = d4 * y[1];
    let l5_u_b = d5 * y[0];
    let l5_b_a = d0 * y[5];
    let l5_b_b = d1 * y[4];
    let l5_fs1 = l5_u_a + l5_u_b;
    let l5_fs2 = l5_fs1 + l5_b_a;
    dst[5] = l5_fs2 + l5_b_b;

    // dst[4]: u4 + b4 = d4*y[0] + d0*y[4].
    let l4_u_a = d4 * y[0];
    let l4_b_a = d0 * y[4];
    dst[4] = l4_u_a + l4_b_a;

    // dst[3..=0]: from N=2 multo body on d[0..4], y[0..4].
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

/// N=4 multo_skipconst — analogous to N=4 multo but treating y[0]=0
/// (zeros out the constant term and skips contributions where k=0).
///
/// Bit-mask semantics: new_dst[i] = Σ over (j, k) with j|k=i, j&k=0,
/// k > 0 of d_j * y_k. This means new_dst[0] = 0 (no valid k=0 terms);
/// for other i, drop the (i, 0) term that multo_n4 would include.
#[cube]
pub fn ctaylor_multo_skipconst_n4<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    // Snapshot pre-update values d0..d15 (d15 unused — skipconst on dst[15]
    // never reads it; suppress with explicit `_ = d15` is unnecessary as we
    // simply don't bind it).
    let d0 = dst[0];
    let d1 = dst[1];
    let d2 = dst[2];
    let d3 = dst[3];
    let d4 = dst[4];
    let d5 = dst[5];
    let d6 = dst[6];
    let d7 = dst[7];
    let d8 = dst[8];
    let d9 = dst[9];
    let d10 = dst[10];
    let d11 = dst[11];
    let d12 = dst[12];
    let d13 = dst[13];
    let d14 = dst[14];
    // d15 is intentionally NOT captured: skipconst's new_dst[15] expression
    // contains no `d15 * y[0]` term (since k=0 is dropped). Any `d15 * y[k]`
    // for k>0 is also absent because (15, k) with k>0 has k & 15 != 0
    // (j&k=0 fails).

    // ============================================================
    // Upper half dst[8..=15] — step A (multo_skipconst_n3) + step B (mul_acc_n3)
    // ============================================================

    // dst[15]: drop (15, 0) term from u15 → 7 step-A terms; step-B unchanged (8 terms).
    //   u15_skip = d12*y[3] + d13*y[2] + d14*y[1]    (3 terms; d15*y[0] dropped)
    //            + d8*y[7] + d11*y[4] + d9*y[6] + d10*y[5]   (4 terms — k>0 here, all valid)
    //   c15_skip = same as multo_n4 dst[15] step-B, all 8 cross-terms (k ∈ 8..16, all >0).
    let u15_a = d12 * y[3];
    let u15_b = d13 * y[2];
    let u15_c = d14 * y[1];
    let u15_s1 = u15_a + u15_b;
    let u15_s2 = u15_s1 + u15_c;
    let b15_a = d8 * y[7];
    let b15_b = d11 * y[4];
    let b15_c = d9 * y[6];
    let b15_d = d10 * y[5];
    let f15_s1 = u15_s2 + b15_a;
    let f15_s2 = f15_s1 + b15_b;
    let f15_s3 = f15_s2 + b15_c;
    let f15_s4 = f15_s3 + b15_d;
    let c15_a = d4 * y[11];
    let c15_b = d7 * y[8];
    let c15_c = d5 * y[10];
    let c15_d = d6 * y[9];
    let g15_s1 = f15_s4 + c15_a;
    let g15_s2 = g15_s1 + c15_b;
    let g15_s3 = g15_s2 + c15_c;
    let g15_s4 = g15_s3 + c15_d;
    let c15_e = d0 * y[15];
    let c15_f = d3 * y[12];
    let c15_g = d1 * y[14];
    let c15_h = d2 * y[13];
    let h15_s1 = g15_s4 + c15_e;
    let h15_s2 = h15_s1 + c15_f;
    let h15_s3 = h15_s2 + c15_g;
    dst[15] = h15_s3 + c15_h;

    // dst[14]: drop (14, 0) from u14 → 1 step-A term; step-B unchanged.
    //   u14_skip = d12*y[2] + d8*y[6] + d10*y[4]   (3 terms; d14*y[0] dropped)
    //   c14_skip = d4*y[10] + d6*y[8] + d0*y[14] + d2*y[12]
    let u14_a = d12 * y[2];
    let b14_a = d8 * y[6];
    let b14_b = d10 * y[4];
    let f14_s1 = u14_a + b14_a;
    let f14_s2 = f14_s1 + b14_b;
    let c14_a = d4 * y[10];
    let c14_b = d6 * y[8];
    let c14_c = d0 * y[14];
    let c14_d = d2 * y[12];
    let g14_s1 = f14_s2 + c14_a;
    let g14_s2 = g14_s1 + c14_b;
    let g14_s3 = g14_s2 + c14_c;
    dst[14] = g14_s3 + c14_d;

    // dst[13]: drop (13, 0) from u13 → 1 step-A term; step-B unchanged.
    //   u13_skip = d12*y[1] + d8*y[5] + d9*y[4]
    //   c13 = d4*y[9] + d5*y[8] + d0*y[13] + d1*y[12]
    let u13_a = d12 * y[1];
    let b13_a = d8 * y[5];
    let b13_b = d9 * y[4];
    let f13_s1 = u13_a + b13_a;
    let f13_s2 = f13_s1 + b13_b;
    let c13_a = d4 * y[9];
    let c13_b = d5 * y[8];
    let c13_c = d0 * y[13];
    let c13_d = d1 * y[12];
    let g13_s1 = f13_s2 + c13_a;
    let g13_s2 = g13_s1 + c13_b;
    let g13_s3 = g13_s2 + c13_c;
    dst[13] = g13_s3 + c13_d;

    // dst[12]: drop (12, 0) → step-A becomes just d8*y[4].
    //   u12_skip = d8*y[4]
    //   c12 = d4*y[8] + d0*y[12]
    let b12_a = d8 * y[4];
    let c12_a = d4 * y[8];
    let c12_b = d0 * y[12];
    let g12_s1 = b12_a + c12_a;
    dst[12] = g12_s1 + c12_b;

    // dst[11]: drop (11, 0) from u11 → 3 step-A terms.
    //   u11_skip = d8*y[3] + d9*y[2] + d10*y[1]
    //   c11 = d0*y[11] + d3*y[8] + d1*y[10] + d2*y[9]
    let u11_a = d8 * y[3];
    let u11_c = d9 * y[2];
    let u11_d = d10 * y[1];
    let f11_s1 = u11_a + u11_c;
    let f11_s2 = f11_s1 + u11_d;
    let c11_a = d0 * y[11];
    let c11_b = d3 * y[8];
    let c11_c = d1 * y[10];
    let c11_d = d2 * y[9];
    let g11_s1 = f11_s2 + c11_a;
    let g11_s2 = g11_s1 + c11_b;
    let g11_s3 = g11_s2 + c11_c;
    dst[11] = g11_s3 + c11_d;

    // dst[10]: drop (10, 0) → u10_skip = d8*y[2]; c10 = d0*y[10] + d2*y[8].
    let u10_a = d8 * y[2];
    let c10_a = d0 * y[10];
    let c10_b = d2 * y[8];
    let g10_s1 = u10_a + c10_a;
    dst[10] = g10_s1 + c10_b;

    // dst[9]: drop (9, 0) → u9_skip = d8*y[1]; c9 = d0*y[9] + d1*y[8].
    let u9_a = d8 * y[1];
    let c9_a = d0 * y[9];
    let c9_b = d1 * y[8];
    let g9_s1 = u9_a + c9_a;
    dst[9] = g9_s1 + c9_b;

    // dst[8]: drop (8, 0) → u8_skip = 0 (only term was d8*y[0]).
    //                         step-B leaves c8 = d0*y[8].
    dst[8] = d0 * y[8];

    // ============================================================
    // Lower half dst[0..=7] — step C (multo_skipconst_n3 of dst[0..8], y[0..8])
    //   Body identical to multo_skipconst_n3.
    // ============================================================

    // dst[7]: skip (7, 0) → u7_skip = d4*y[3] + d5*y[2] + d6*y[1]; b7 same.
    let l7_u_a = d4 * y[3];
    let l7_u_b = d5 * y[2];
    let l7_u_c = d6 * y[1];
    let l7_us1 = l7_u_a + l7_u_b;
    let l7_us2 = l7_us1 + l7_u_c;
    let l7_b_a = d0 * y[7];
    let l7_b_b = d3 * y[4];
    let l7_b_c = d1 * y[6];
    let l7_b_d = d2 * y[5];
    let l7_fs1 = l7_us2 + l7_b_a;
    let l7_fs2 = l7_fs1 + l7_b_b;
    let l7_fs3 = l7_fs2 + l7_b_c;
    dst[7] = l7_fs3 + l7_b_d;

    // dst[6]: u6_skip = d4*y[2]; b6 = d0*y[6] + d2*y[4].
    let l6_u_a = d4 * y[2];
    let l6_b_a = d0 * y[6];
    let l6_b_b = d2 * y[4];
    let l6_fs1 = l6_u_a + l6_b_a;
    dst[6] = l6_fs1 + l6_b_b;

    // dst[5]: u5_skip = d4*y[1]; b5 = d0*y[5] + d1*y[4].
    let l5_u_a = d4 * y[1];
    let l5_b_a = d0 * y[5];
    let l5_b_b = d1 * y[4];
    let l5_fs1 = l5_u_a + l5_b_a;
    dst[5] = l5_fs1 + l5_b_b;

    // dst[4]: u4_skip = 0; b4 = d0*y[4].
    dst[4] = d0 * y[4];

    // dst[3..=0]: N=2 multo_skipconst on d[0..4], y[0..4].
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

/// Outer dispatch for `dst *= y` across N ∈ {0, 1, 2, 3, 4}.
///
/// **Phase 6 status:** N=4 lands in Plan 06-00 Task 1 (D-19 forward
/// from Phase-4 Plan 04-05 — Mode::Contracted order-5 metaGGA). N=5 / N=6
/// remain pending — they would unblock Mode::Contracted order 6 metaGGA
/// (the projected upper bound at xcfun-master/api/xcfun.h:21 `XC_MAX_ORDER=4`,
/// which Mode::Contracted exceeds via `inlen × (1 << order)` packing).
/// A follow-up plan in Phase 6 will land N=5/6 once the demand is concrete.
#[cube]
pub fn ctaylor_multo<F: Float>(dst: &mut Array<F>, y: &Array<F>, #[comptime] n: u32) {
    if comptime!(n == 0) {
        ctaylor_multo_n0::<F>(dst, y);
    } else if comptime!(n == 1) {
        ctaylor_multo_n1::<F>(dst, y);
    } else if comptime!(n == 2) {
        ctaylor_multo_n2::<F>(dst, y);
    } else if comptime!(n == 3) {
        ctaylor_multo_n3::<F>(dst, y);
    } else if comptime!(n == 4) {
        ctaylor_multo_n4::<F>(dst, y);
    }
}

/// Outer dispatch for `dst *= (y - y[0])` across N ∈ {0, 1, 2, 3, 4}.
#[cube]
pub fn ctaylor_multo_skipconst<F: Float>(dst: &mut Array<F>, y: &Array<F>, #[comptime] n: u32) {
    if comptime!(n == 0) {
        ctaylor_multo_skipconst_n0::<F>(dst, y);
    } else if comptime!(n == 1) {
        ctaylor_multo_skipconst_n1::<F>(dst, y);
    } else if comptime!(n == 2) {
        ctaylor_multo_skipconst_n2::<F>(dst, y);
    } else if comptime!(n == 3) {
        ctaylor_multo_skipconst_n3::<F>(dst, y);
    } else if comptime!(n == 4) {
        ctaylor_multo_skipconst_n4::<F>(dst, y);
    }
}
