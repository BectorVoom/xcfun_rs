//! `tfuns<T, N>` scalar Taylor-series helpers. Port of
//! `xcfun-master/external/upstream/taylor/tmath.hpp:36-121` (the
//! `template<T, N> struct tfuns` block).
//!
//! These helpers operate on length-(n+1) `Array<F>`s that represent
//! scalar Taylor coefficients (distinct from `CTaylor`'s bit-flag-indexed
//! multilinear polynomials). They are used internally by the
//! transcendental `*_expand` fns in `crates/xcfun-ad/src/expand/` (atan,
//! gauss, erf, asinh).
//!
//! # Port strategy
//!
//! - `tfuns_mul`, `tfuns_multo`, `tfuns_integrate`, `tfuns_differentiate`,
//!   `tfuns_stretch`, `tfuns_shift` are ported as single `#[cube] fn`s
//!   whose inner loop is unrolled over `#[comptime] n`.
//! - `tfuns_compose` is the trickiest C++ function: a `switch(N)` with
//!   **fallthrough** (every `case k:` executes `case k..0`). Per-N
//!   specialisations are flattened straight-line bodies for n ∈ 0..=6,
//!   dispatched by a `comptime!(n == k)` chain matching the pattern
//!   established in `ctaylor_rec::compose` (Plan 01-02).
//!
//! # Snapshot discipline for tfuns_compose
//!
//! Every `case k:` in `compose` writes `f[k]` using only pre-update values
//! of `f[1..=k]` plus constants from `x[1..=k]`. We snapshot all `f[i]` into
//! local `let fi = f[i]` bindings at the top of each per-N body, then do
//! the descending writes from `f[k]` down to `f[1]`. This removes any
//! read-after-write aliasing ambiguity on the `&mut Array<F>` and keeps
//! the bodies as pure straight-line code (D-08 inspection discipline).
//!
//! # Cubecl 0.10-pre.3 deviations carried over from Plan 01-03
//!
//! - In-kernel `assert!` / `debug_assert!` rejected — no preconditions in
//!   `#[cube]` bodies. `tfuns_integrate` says "leaves x[0] undefined" per
//!   the C++ contract; the caller sets `x[0]` afterwards.
//! - Method-form for intrinsics (`.exp()`, `.sqrt()`, `.ln()`, `.powf()`).
//! - `F::cast_from(i)` casts loop indices into the float type.

use cubecl::prelude::*;

// ---------------------------------------------------------------------------
//  tfuns_mul — truncated 1D convolution: z[i] = sum_{j=0..=i} x[j] * y[i-j]
//  Port of tmath.hpp:37-43.
// ---------------------------------------------------------------------------

/// `z[i] = sum_{j=0..=i} x[j] * y[i-j]` for `i ∈ 0..=n`.
///
/// Port of tmath.hpp:37-43.
///
/// ```cpp
/// static void mul(T * z, const T * x, const T * y) {
///   for (int i = 0; i <= N; i++) {
///     z[i] = x[0] * y[i];
///     for (int j = 1; j <= i; j++)
///       z[i] += x[j] * y[i - j];
///   }
/// }
/// ```
#[cube]
pub fn tfuns_mul<F: Float>(
    z: &mut Array<F>,
    x: &Array<F>,
    y: &Array<F>,
    #[comptime] n: u32,
) {
    #[unroll]
    for i in 0_u32..=n {
        let k = i as usize;
        // tmath.hpp:39 — z[i] = x[0] * y[i]
        let mut acc = x[0] * y[k];
        // tmath.hpp:40-41 — z[i] += x[j] * y[i - j]
        #[unroll]
        for j in 1_u32..=i {
            let jk = j as usize;
            let ikj = (i - j) as usize;
            acc += x[jk] * y[ikj];
        }
        z[k] = acc;
    }
}

// ---------------------------------------------------------------------------
//  tfuns_multo — in-place z *= x, writing in DESCENDING order.
//  Port of tmath.hpp:45-51.
// ---------------------------------------------------------------------------

/// `z *= x` where both are length-(n+1) scalar Taylor series.
///
/// Port of tmath.hpp:45-51.
///
/// ```cpp
/// static void multo(T * z, const T * x) {
///   for (int i = N; i >= 0; i--) {
///     z[i] = x[0] * z[i];
///     for (int j = 1; j <= i; j++)
///       z[i] += x[j] * z[i - j];
///   }
/// }
/// ```
///
/// The descending outer loop is mandatory: each `z[i]` reads `z[i-j]` for
/// `j ∈ 1..=i`, and those reads must see the pre-multiply value of
/// `z[i-j]`. Descending `i` guarantees that lower-index cells are still
/// unmodified when higher-index cells read them.
#[cube]
pub fn tfuns_multo<F: Float>(z: &mut Array<F>, x: &Array<F>, #[comptime] n: u32) {
    // Descending i: counter k ∈ 0..=n, then i = n - k runs n..0.
    #[unroll]
    for k in 0_u32..=n {
        let i = n - k;
        let ki = i as usize;
        // tmath.hpp:47 — z[i] = x[0] * z[i]
        let mut acc = x[0] * z[ki];
        // tmath.hpp:48-49 — z[i] += x[j] * z[i - j]
        #[unroll]
        for j in 1_u32..=i {
            let jk = j as usize;
            let ikj = (i - j) as usize;
            acc += x[jk] * z[ikj];
        }
        z[ki] = acc;
    }
}

// ---------------------------------------------------------------------------
//  tfuns_integrate — anti-derivative, leaves x[0] undefined.
//  Port of tmath.hpp:53-56.
// ---------------------------------------------------------------------------

/// Term-wise integration: `x[i] = x[i-1] / i` for `i ∈ N..=1` (descending).
/// `x[0]` is left untouched by this fn (caller sets it afterwards per the
/// C++ contract `// Integrates termwise, leaves x[0] undefined!`).
///
/// Port of tmath.hpp:53-56.
///
/// ```cpp
/// static void integrate(T * x) {
///   for (int i = N; i >= 1; i--)
///     x[i] = x[i - 1] / i;
/// }
/// ```
#[cube]
pub fn tfuns_integrate<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    // Descending i: counter k ∈ 0..n, then i = n - k runs n..1.
    #[unroll]
    for k in 0_u32..n {
        let i = n - k;
        let ki = i as usize;
        let i_f = F::cast_from(i);
        x[ki] = x[ki - 1] / i_f;
    }
}

// ---------------------------------------------------------------------------
//  tfuns_differentiate — x[i-1] = i * x[i] ascending, then x[N] = 0.
//  Port of tmath.hpp:57-61.
// ---------------------------------------------------------------------------

/// Term-wise differentiation: `x[i-1] = i * x[i]` ascending, then `x[n] = 0`.
///
/// Port of tmath.hpp:57-61.
///
/// ```cpp
/// static void differentiate(T * x) {
///   for (int i = 1; i <= N; i++)
///     x[i - 1] = i * x[i];
///   x[N] = 0;
/// }
/// ```
#[cube]
pub fn tfuns_differentiate<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    #[unroll]
    for i in 1_u32..=n {
        let ki = i as usize;
        let i_f = F::cast_from(i);
        x[ki - 1] = i_f * x[ki];
    }
    let kn = n as usize;
    x[kn] = F::new(0.0);
}

// ---------------------------------------------------------------------------
//  tfuns_shift — x_new(y) = x_old(y + d).  Port of tmath.hpp:63-78.
// ---------------------------------------------------------------------------

/// Shift expansion point: replace the scalar series in `x` with the series
/// of the same polynomial re-expanded around a point offset by `d`. Port of
/// tmath.hpp:63-78.
///
/// ```cpp
/// static void shift(T * x, T d) {
///   T dn[N + 1];
///   dn[0] = 1;
///   for (int i = 1; i <= N; i++) dn[i] = d * dn[i - 1];
///   for (int n = 0; n < N; n++) {
///     int fac = n + 1;
///     for (int m = n + 1; m < N; m++) {
///       x[n] += fac * dn[m - n] * x[m];
///       fac *= m + 1;
///       fac /= m - n + 1;
///     }
///     x[n] += fac * dn[N - n] * x[N];
///   }
/// }
/// ```
///
/// Not consumed by any of atan/gauss/erf/asinh in Plan 01-04, but included
/// to complete the `tfuns` port per AD-04 (truths entry #1). Future plans
/// may call it; if unused downstream it is still exercised by the Task 1
/// unit test.
#[cube]
pub fn tfuns_shift<F: Float>(x: &mut Array<F>, d: F, #[comptime] n: u32) {
    // tmath.hpp:64 — T dn[N + 1] (stack-local scratch, length n+1).
    // cubecl 0.10-pre.3 `Array::new` takes `#[comptime] length: usize`, so
    // cast via `comptime!`. See plan 01-04 `<interfaces>` note on
    // in-kernel scratch allocation.
    let dn_len = comptime!((n + 1) as usize);
    let mut dn = Array::<F>::new(dn_len);

    // tmath.hpp:65 — dn[0] = 1
    dn[0] = F::new(1.0);
    // tmath.hpp:66-67 — dn[i] = d * dn[i-1]
    #[unroll]
    for i in 1_u32..=n {
        let ki = i as usize;
        dn[ki] = d * dn[ki - 1];
    }

    // tmath.hpp:69-77 — accumulate into x[ni] (C++ loop var `n`, renamed
    // here to `ni` to avoid shadowing the comptime parameter `n`).
    #[unroll]
    for ni in 0_u32..n {
        let kni = ni as usize;
        let mut acc_x = x[kni];
        // fac starts at (ni + 1) — keep it as an f-typed running product.
        // C++ uses `int fac`, and the expression `fac * dn[...] * x[...]`
        // widens fac to T via the multiplication. We mirror that by keeping
        // fac as F throughout; the integer semantics are exact because the
        // running product values all fit in u32 for N ≤ 6 (the Plan 01-04
        // envelope for tfuns_compose).
        let mut fac = F::cast_from(ni + 1_u32);

        // Inner loop bound depends on outer `ni`: m ∈ ni+1..n.
        #[unroll]
        for m in 0_u32..n {
            // We want `for m in ni+1..n`; since both bounds are comptime,
            // guard the body with a comptime condition.
            if comptime!(m > ni) {
                let km = m as usize;
                // tmath.hpp:72 — x[n] += fac * dn[m - n] * x[m]
                //                         ^^^                       ^^^
                //                    running product           pre-update x[m]
                let dn_idx = (m - ni) as usize;
                let s1 = fac * dn[dn_idx];
                let s2 = s1 * x[km];
                acc_x += s2;
                // tmath.hpp:73-74 — fac *= (m + 1); fac /= (m - n + 1)
                let mp1 = F::cast_from(m + 1_u32);
                let mmn = F::cast_from(m - ni + 1_u32);
                fac *= mp1;
                fac /= mmn;
            }
        }

        // tmath.hpp:76 — x[n] += fac * dn[N - n] * x[N]
        let kn = n as usize;
        let dn_last = (n - ni) as usize;
        let tail1 = fac * dn[dn_last];
        let tail2 = tail1 * x[kn];
        acc_x += tail2;

        x[kni] = acc_x;
    }
}

// ---------------------------------------------------------------------------
//  tfuns_stretch — t[i] *= a^i for i ≥ 1.
//  Port of tmath.hpp:114-120.
// ---------------------------------------------------------------------------

/// Scale each coefficient `t[i]` by `a^i` for `i ∈ 1..=n`. `t[0]` unchanged.
///
/// Port of tmath.hpp:114-120.
///
/// ```cpp
/// static void stretch(T * t, T a) {
///   T an = a;
///   for (int i = 1; i <= N; i++) {
///     t[i] *= an;
///     an *= a;
///   }
/// }
/// ```
#[cube]
pub fn tfuns_stretch<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:115 — T an = a (an tracks a^i with an initial value of a,
    // so at iteration i=1 the factor is a^1).
    let mut an = a;
    // tmath.hpp:116-119 — t[i] *= an; an *= a.
    #[unroll]
    for i in 1_u32..=n {
        let ki = i as usize;
        t[ki] *= an;
        an *= a;
    }
}

// ---------------------------------------------------------------------------
//  tfuns_compose — f_new[k] = [x^k]( sum_i f[i] * x(y)^i ), x[0] = 0.
//  Port of tmath.hpp:80-113 — switch cascade (fallthrough). Per-N bodies.
// ---------------------------------------------------------------------------

// The C++ switch(N) cascade uses DOWNWARD fallthrough: `case 6:` executes
// the `f[6] = ...` assignment AND THEN all subsequent cases `5, 4, 3, 2, 1`,
// each of which assigns a single `f[k]`. Each `f[k]` expression uses only
// the ORIGINAL f[1..=k] values (which are still pre-update because the
// assignment order is f[N], f[N-1], ..., f[1] — strictly descending).
//
// We implement this as per-N flattened bodies, snapshotting all input
// f[0..=k] into locals at the top to make the D-08 operation-order claim
// mechanically inspectable.

/// N=0 compose: a no-op (case 0 in the C++ switch is `break;`).
///
/// tmath.hpp:108-109.
#[cube]
pub(crate) fn tfuns_compose_n0<F: Float>(_f: &mut Array<F>, _x: &Array<F>) {}

/// N=1 compose: `f[1] = f[1] * x[1]`.
///
/// tmath.hpp:106-107.
#[cube]
pub(crate) fn tfuns_compose_n1<F: Float>(f: &mut Array<F>, x: &Array<F>) {
    let f1 = f[1];
    f[1] = f1 * x[1];
}

/// N=2 compose. tmath.hpp:104-107 (cases 2, 1 fall through).
///
/// ```cpp
/// case 2: f[2] = f[1]*x[2] + f[2]*x[1]*x[1];
/// case 1: f[1] = f[1]*x[1];
/// ```
#[cube]
pub(crate) fn tfuns_compose_n2<F: Float>(f: &mut Array<F>, x: &Array<F>) {
    // Snapshot pre-update f[1..=2].
    let f1 = f[1];
    let f2 = f[2];

    // f[2] = f[1]*x[2] + f[2]*x[1]*x[1]  (C++ left-assoc on the triple product)
    let a1 = f1 * x[2];
    let b1 = f2 * x[1];
    let b2 = b1 * x[1];
    f[2] = a1 + b2;

    // f[1] = f[1]*x[1]
    f[1] = f1 * x[1];
}

/// N=3 compose. tmath.hpp:102-107 (cases 3, 2, 1 fall through).
///
/// ```cpp
/// case 3: f[3] = f[1]*x[3] + x[1]*(2*f[2]*x[2] + f[3]*x[1]*x[1]);
/// case 2: f[2] = f[1]*x[2] + f[2]*x[1]*x[1];
/// case 1: f[1] = f[1]*x[1];
/// ```
#[cube]
pub(crate) fn tfuns_compose_n3<F: Float>(f: &mut Array<F>, x: &Array<F>) {
    let f1 = f[1];
    let f2 = f[2];
    let f3 = f[3];
    let two = F::new(2.0);

    // f[3] = f[1]*x[3] + x[1] * (2*f[2]*x[2] + f[3]*x[1]*x[1])
    //      C++ left-to-right: (2 * f[2]) * x[2]; (f[3] * x[1]) * x[1]
    let t1 = f1 * x[3];
    let s2a = two * f2;
    let s2b = s2a * x[2];
    let s3a = f3 * x[1];
    let s3b = s3a * x[1];
    let inner = s2b + s3b;
    let tail = x[1] * inner;
    f[3] = t1 + tail;

    // f[2] = f[1]*x[2] + f[2]*x[1]*x[1]
    let a1 = f1 * x[2];
    let b1 = f2 * x[1];
    let b2 = b1 * x[1];
    f[2] = a1 + b2;

    // f[1] = f[1]*x[1]
    f[1] = f1 * x[1];
}

/// N=4 compose. tmath.hpp:97-107 (cases 4, 3, 2, 1 fall through).
///
/// ```cpp
/// case 4: f[4] = f[1]*x[4]
///              + x[1]*(2*f[2]*x[3] + x[1]*(3*f[3]*x[2] + f[4]*x[1]*x[1]))
///              + f[2]*x[2]*x[2];
/// case 3: f[3] = f[1]*x[3] + x[1]*(2*f[2]*x[2] + f[3]*x[1]*x[1]);
/// case 2: f[2] = f[1]*x[2] + f[2]*x[1]*x[1];
/// case 1: f[1] = f[1]*x[1];
/// ```
#[cube]
pub(crate) fn tfuns_compose_n4<F: Float>(f: &mut Array<F>, x: &Array<F>) {
    let f1 = f[1];
    let f2 = f[2];
    let f3 = f[3];
    let f4 = f[4];
    let two = F::new(2.0);
    let three = F::new(3.0);

    // f[4]: case 4.
    // term_A = f[1] * x[4]
    let term_a = f1 * x[4];
    // inner_inner = 3*f[3]*x[2] + f[4]*x[1]*x[1]
    let ia1 = three * f3;
    let ia2 = ia1 * x[2];
    let ib1 = f4 * x[1];
    let ib2 = ib1 * x[1];
    let inner_inner = ia2 + ib2;
    // inner = 2*f[2]*x[3] + x[1] * inner_inner
    let ja1 = two * f2;
    let ja2 = ja1 * x[3];
    let jb = x[1] * inner_inner;
    let inner = ja2 + jb;
    // term_B = x[1] * inner
    let term_b = x[1] * inner;
    // term_C = f[2] * x[2] * x[2]
    let tc1 = f2 * x[2];
    let tc2 = tc1 * x[2];
    // f[4] = term_A + term_B + term_C  (C++ left-assoc)
    let sum_ab = term_a + term_b;
    f[4] = sum_ab + tc2;

    // f[3]: same as n3 case 3.
    let t1 = f1 * x[3];
    let s2a = two * f2;
    let s2b = s2a * x[2];
    let s3a = f3 * x[1];
    let s3b = s3a * x[1];
    let inner3 = s2b + s3b;
    let tail3 = x[1] * inner3;
    f[3] = t1 + tail3;

    // f[2]: case 2.
    let a1 = f1 * x[2];
    let b1 = f2 * x[1];
    let b2 = b1 * x[1];
    f[2] = a1 + b2;

    // f[1]: case 1.
    f[1] = f1 * x[1];
}

/// N=5 compose. tmath.hpp:90-107 (cases 5, 4, 3, 2, 1 fall through).
///
/// ```cpp
/// case 5: f[5] = f[1]*x[5]
///              + x[1]*(2*f[2]*x[4]
///                      + x[1]*(3*f[3]*x[3]
///                              + x[1]*(4*f[4]*x[2] + f[5]*x[1]*x[1]))
///                      + 3*f[3]*x[2]*x[2])
///              + 2*f[2]*x[2]*x[3];
/// case 4: f[4] = ... ;
/// case 3: f[3] = ... ;
/// case 2: f[2] = ... ;
/// case 1: f[1] = ... ;
/// ```
#[cube]
pub(crate) fn tfuns_compose_n5<F: Float>(f: &mut Array<F>, x: &Array<F>) {
    let f1 = f[1];
    let f2 = f[2];
    let f3 = f[3];
    let f4 = f[4];
    let f5 = f[5];
    let two = F::new(2.0);
    let three = F::new(3.0);
    let four = F::new(4.0);

    // ---- case 5 ----
    // innermost3 = 4*f[4]*x[2] + f[5]*x[1]*x[1]
    let a1 = four * f4;
    let a2 = a1 * x[2];
    let b1 = f5 * x[1];
    let b2 = b1 * x[1];
    let innermost3 = a2 + b2;
    // mid2 = 3*f[3]*x[3] + x[1] * innermost3
    let c1 = three * f3;
    let c2 = c1 * x[3];
    let d1 = x[1] * innermost3;
    let mid2 = c2 + d1;
    // inner1 = 2*f[2]*x[4] + x[1]*mid2 + 3*f[3]*x[2]*x[2]
    let e1 = two * f2;
    let e2 = e1 * x[4];
    let g1 = x[1] * mid2;
    let h1 = three * f3;
    let h2 = h1 * x[2];
    let h3 = h2 * x[2];
    let inner1_ab = e2 + g1;
    let inner1 = inner1_ab + h3;
    // term_B = x[1] * inner1
    let term_b = x[1] * inner1;
    // term_A = f[1] * x[5]
    let term_a = f1 * x[5];
    // term_C = 2*f[2]*x[2]*x[3]
    let p1 = two * f2;
    let p2 = p1 * x[2];
    let p3 = p2 * x[3];
    // f[5] = term_A + term_B + term_C
    let sum_ab = term_a + term_b;
    f[5] = sum_ab + p3;

    // ---- case 4 ----
    let term_a4 = f1 * x[4];
    let ia1_4 = three * f3;
    let ia2_4 = ia1_4 * x[2];
    let ib1_4 = f4 * x[1];
    let ib2_4 = ib1_4 * x[1];
    let inner_inner_4 = ia2_4 + ib2_4;
    let ja1_4 = two * f2;
    let ja2_4 = ja1_4 * x[3];
    let jb_4 = x[1] * inner_inner_4;
    let inner_4 = ja2_4 + jb_4;
    let term_b4 = x[1] * inner_4;
    let tc1_4 = f2 * x[2];
    let tc2_4 = tc1_4 * x[2];
    let sum_ab4 = term_a4 + term_b4;
    f[4] = sum_ab4 + tc2_4;

    // ---- case 3 ----
    let t1 = f1 * x[3];
    let s2a = two * f2;
    let s2b = s2a * x[2];
    let s3a = f3 * x[1];
    let s3b = s3a * x[1];
    let inner3 = s2b + s3b;
    let tail3 = x[1] * inner3;
    f[3] = t1 + tail3;

    // ---- case 2 ----
    let a1_2 = f1 * x[2];
    let b1_2 = f2 * x[1];
    let b2_2 = b1_2 * x[1];
    f[2] = a1_2 + b2_2;

    // ---- case 1 ----
    f[1] = f1 * x[1];
}

/// N=6 compose. tmath.hpp:83-107 (cases 6, 5, 4, 3, 2, 1 fall through).
///
/// ```cpp
/// case 6: f[6] = f[1]*x[6] + 2*f[2]*x[5]*x[1] + 2*f[2]*x[4]*x[2]
///              + f[2]*x[3]*x[3]
///              + 3*f[3]*x[4]*x[1]*x[1]
///              + 6*f[3]*x[3]*x[2]*x[1]
///              + f[3]*x[2]*x[2]*x[2]
///              + 4*f[4]*x[3]*x[1]*x[1]*x[1]
///              + 6*f[4]*x[2]*x[2]*x[1]*x[1]
///              + 5*f[5]*x[2]*x[1]*x[1]*x[1]*x[1]
///              + f[6]*x[1]*x[1]*x[1]*x[1]*x[1]*x[1];
/// case 5: ...
/// ```
#[cube]
pub(crate) fn tfuns_compose_n6<F: Float>(f: &mut Array<F>, x: &Array<F>) {
    let f1 = f[1];
    let f2 = f[2];
    let f3 = f[3];
    let f4 = f[4];
    let f5 = f[5];
    let f6 = f[6];
    let two = F::new(2.0);
    let three = F::new(3.0);
    let four = F::new(4.0);
    let five = F::new(5.0);
    let six = F::new(6.0);

    // ---- case 6 ----
    // term_0 = f[1] * x[6]
    let t0 = f1 * x[6];
    // term_1 = 2 * f[2] * x[5] * x[1]
    let t1a = two * f2;
    let t1b = t1a * x[5];
    let t1c = t1b * x[1];
    // term_2 = 2 * f[2] * x[4] * x[2]
    let t2a = two * f2;
    let t2b = t2a * x[4];
    let t2c = t2b * x[2];
    // term_3 = f[2] * x[3] * x[3]
    let t3a = f2 * x[3];
    let t3b = t3a * x[3];
    // term_4 = 3 * f[3] * x[4] * x[1] * x[1]
    let t4a = three * f3;
    let t4b = t4a * x[4];
    let t4c = t4b * x[1];
    let t4d = t4c * x[1];
    // term_5 = 6 * f[3] * x[3] * x[2] * x[1]
    let t5a = six * f3;
    let t5b = t5a * x[3];
    let t5c = t5b * x[2];
    let t5d = t5c * x[1];
    // term_6 = f[3] * x[2] * x[2] * x[2]
    let t6a = f3 * x[2];
    let t6b = t6a * x[2];
    let t6c = t6b * x[2];
    // term_7 = 4 * f[4] * x[3] * x[1] * x[1] * x[1]
    let t7a = four * f4;
    let t7b = t7a * x[3];
    let t7c = t7b * x[1];
    let t7d = t7c * x[1];
    let t7e = t7d * x[1];
    // term_8 = 6 * f[4] * x[2] * x[2] * x[1] * x[1]
    let t8a = six * f4;
    let t8b = t8a * x[2];
    let t8c = t8b * x[2];
    let t8d = t8c * x[1];
    let t8e = t8d * x[1];
    // term_9 = 5 * f[5] * x[2] * x[1] * x[1] * x[1] * x[1]
    let t9a = five * f5;
    let t9b = t9a * x[2];
    let t9c = t9b * x[1];
    let t9d = t9c * x[1];
    let t9e = t9d * x[1];
    let t9f = t9e * x[1];
    // term_10 = f[6] * x[1]^6
    let ta_a = f6 * x[1];
    let ta_b = ta_a * x[1];
    let ta_c = ta_b * x[1];
    let ta_d = ta_c * x[1];
    let ta_e = ta_d * x[1];
    let ta_f = ta_e * x[1];

    // Left-to-right sum (C++ associates left).
    let s01 = t0 + t1c;
    let s012 = s01 + t2c;
    let s0123 = s012 + t3b;
    let s4 = s0123 + t4d;
    let s5 = s4 + t5d;
    let s6 = s5 + t6c;
    let s7 = s6 + t7e;
    let s8 = s7 + t8e;
    let s9 = s8 + t9f;
    f[6] = s9 + ta_f;

    // ---- case 5 ----
    // innermost3 = 4*f[4]*x[2] + f[5]*x[1]*x[1]
    let a1 = four * f4;
    let a2 = a1 * x[2];
    let b1 = f5 * x[1];
    let b2 = b1 * x[1];
    let innermost3 = a2 + b2;
    // mid2 = 3*f[3]*x[3] + x[1] * innermost3
    let c1 = three * f3;
    let c2 = c1 * x[3];
    let d1 = x[1] * innermost3;
    let mid2 = c2 + d1;
    // inner1 = 2*f[2]*x[4] + x[1]*mid2 + 3*f[3]*x[2]*x[2]
    let e1 = two * f2;
    let e2 = e1 * x[4];
    let g1 = x[1] * mid2;
    let h1 = three * f3;
    let h2 = h1 * x[2];
    let h3 = h2 * x[2];
    let inner1_ab = e2 + g1;
    let inner1 = inner1_ab + h3;
    let term_b5 = x[1] * inner1;
    let term_a5 = f1 * x[5];
    let p1 = two * f2;
    let p2 = p1 * x[2];
    let p3 = p2 * x[3];
    let sum_ab5 = term_a5 + term_b5;
    f[5] = sum_ab5 + p3;

    // ---- case 4 ----
    let term_a4 = f1 * x[4];
    let ia1_4 = three * f3;
    let ia2_4 = ia1_4 * x[2];
    let ib1_4 = f4 * x[1];
    let ib2_4 = ib1_4 * x[1];
    let inner_inner_4 = ia2_4 + ib2_4;
    let ja1_4 = two * f2;
    let ja2_4 = ja1_4 * x[3];
    let jb_4 = x[1] * inner_inner_4;
    let inner_4 = ja2_4 + jb_4;
    let term_b4 = x[1] * inner_4;
    let tc1_4 = f2 * x[2];
    let tc2_4 = tc1_4 * x[2];
    let sum_ab4 = term_a4 + term_b4;
    f[4] = sum_ab4 + tc2_4;

    // ---- case 3 ----
    let t1 = f1 * x[3];
    let s2a = two * f2;
    let s2b = s2a * x[2];
    let s3a = f3 * x[1];
    let s3b = s3a * x[1];
    let inner3 = s2b + s3b;
    let tail3 = x[1] * inner3;
    f[3] = t1 + tail3;

    // ---- case 2 ----
    let a1_2 = f1 * x[2];
    let b1_2 = f2 * x[1];
    let b2_2 = b1_2 * x[1];
    f[2] = a1_2 + b2_2;

    // ---- case 1 ----
    f[1] = f1 * x[1];
}

/// Outer `tfuns_compose` dispatch for n ∈ 0..=6. Comptime `if` chain (same
/// pattern as `ctaylor_rec::compose`).
///
/// `f` is a scalar-series array of length `n + 1`. `x[0]` is assumed to be
/// zero (the C++ docstring at tmath.hpp:79 says "assuming x[0] = 0").
/// Caller is responsible for that invariant.
#[cube]
pub fn tfuns_compose<F: Float>(f: &mut Array<F>, x: &Array<F>, #[comptime] n: u32) {
    if comptime!(n == 0) {
        tfuns_compose_n0::<F>(f, x);
    } else if comptime!(n == 1) {
        tfuns_compose_n1::<F>(f, x);
    } else if comptime!(n == 2) {
        tfuns_compose_n2::<F>(f, x);
    } else if comptime!(n == 3) {
        tfuns_compose_n3::<F>(f, x);
    } else if comptime!(n == 4) {
        tfuns_compose_n4::<F>(f, x);
    } else if comptime!(n == 5) {
        tfuns_compose_n5::<F>(f, x);
    } else if comptime!(n == 6) {
        tfuns_compose_n6::<F>(f, x);
    }
}
