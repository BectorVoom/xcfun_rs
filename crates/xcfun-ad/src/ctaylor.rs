//! `CTaylor<F, N>` — multilinear Taylor polynomial as a cubecl kernel-scope
//! abstraction. Storage = `Array<F>` of length `1 << N`. Port of
//! `xcfun-master/external/upstream/taylor/ctaylor.hpp:154-337` (struct body
//! + element-wise operators). See CONTEXT.md D-04 for why this is **not** a
//! `#[derive(CubeType)]` host struct — no `#[repr(C)]`, no `Copy`, no host
//! `[F; 1 << N]` field.
//!
//! # Cubecl 0.10-pre.3 idiom notes
//!
//! - `F::new(val: f32)` is the scalar-literal constructor; `F::new(0.0)`
//!   produces the zero of the target float type.
//! - Unary minus on `F: Float` is provided via `core::ops::Neg` (trait bound
//!   on `Float` itself), so `-a[i]` expands correctly inside `#[cube]` fns
//!   — no `F::new(0.0) - a[i]` workaround required.
//! - `#[unroll]` on a `for i in 0..N` loop where `N` is a `#[comptime] u32`
//!   parameter is the pattern exercised in `cubecl-core/runtime_tests/unroll.rs`
//!   and in `tests/cubecl_spike.rs`.
//! - `Array<F>` indexing expects `usize`; the unrolled loop counter is `u32`,
//!   so every index site casts `i as usize`. This matches the working idiom
//!   from `tests/cubecl_spike.rs::copy_kernel` (Plan 01-01).

use cubecl::prelude::*;

/// Fill `out[0..1<<n]` with zero. Port of `ctaylor.hpp:179-182` (default
/// constructor).
///
/// ```cpp
/// ctaylor() { for (int i = 0; i < size; i++) c[i] = 0; }
/// ```
#[cube]
pub fn ctaylor_zero<F: Float>(out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!(1_u32 << n);
    let zero = F::new(0.0);
    #[unroll]
    for i in 0..size {
        out[i as usize] = zero;
    }
}

/// Build `out = c0` as a constant polynomial (only `out[CNST]` is non-zero).
/// Port of `ctaylor.hpp:179-186` (scalar constructor).
///
/// ```cpp
/// ctaylor(const T & c0) {
///   c[0] = c0;
///   for (int i = 1; i < POW2(Nvar); i++) c[i] = 0;
/// }
/// ```
#[cube]
pub fn ctaylor_from_scalar<F: Float>(c0: F, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!(1_u32 << n);
    let zero = F::new(0.0);
    out[0] = c0;
    #[unroll]
    for i in 1..size {
        out[i as usize] = zero;
    }
}

/// Build `out = c0 + slope * x_k` where `var_bit` = `VAR{k}` (1, 2, 4, 8, ...).
/// Port of `ctaylor.hpp:199-209` (variable-slot constructor).
///
/// ```cpp
/// ctaylor(const T & c0, int var, const T & varval) {
///   c[0] = c0;
///   for (int i = 1; i < POW2(Nvar); i++) c[i] = 0;
///   assert(var >= 0); assert(Nvar > var);
///   c[var] = varval;
/// }
/// ```
///
/// `var_bit` is a single-bit slot (1 = VAR0, 2 = VAR1, 4 = VAR2, ...).
/// Precondition `var_bit.count_ones() == 1` and `var_bit < (1 << n)` are
/// checked by callers before launch (D-05 — the cubecl 0.10-pre.3 `#[cube]`
/// body does not admit host-style `assert!` on comptime values reliably, so
/// the check moves to the host-side launcher).
#[cube]
pub fn ctaylor_from_variable<F: Float>(
    c0: F,
    slope: F,
    #[comptime] var_bit: u32,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!(1_u32 << n);
    let zero = F::new(0.0);
    out[0] = c0;
    #[unroll]
    for i in 1..size {
        out[i as usize] = zero;
    }
    out[var_bit as usize] = slope;
}

/// Elementwise `out = a + b`. Port of `ctaylor.hpp:295-311` + `:468-474`
/// (operator+=, operator+).
///
/// ```cpp
/// template <class T, int Nvar>
/// ctaylor<T, Nvar> operator+(const ctaylor<T, Nvar> & t1,
///                            const ctaylor<T, Nvar> & t2) {
///   ctaylor<T, Nvar> tmp = t1;
///   tmp += t2;   // operator+= loops c[i] += t.c[i]
///   return tmp;
/// }
/// ```
#[cube]
pub fn ctaylor_add<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!(1_u32 << n);
    #[unroll]
    for i in 0..size {
        let k = i as usize;
        out[k] = a[k] + b[k];
    }
}

/// Elementwise `out = a - b`. Port of `ctaylor.hpp:278-294` + `:504-510`
/// (operator-=, binary operator-).
///
/// ```cpp
/// template <class T, int Nvar>
/// ctaylor<T, Nvar> operator-(const ctaylor<T, Nvar> & t1,
///                            const ctaylor<T, Nvar> & t2) {
///   ctaylor<T, Nvar> tmp = t1;
///   tmp -= t2;   // operator-= loops c[i] -= t.c[i]
///   return tmp;
/// }
/// ```
#[cube]
pub fn ctaylor_sub<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!(1_u32 << n);
    #[unroll]
    for i in 0..size {
        let k = i as usize;
        out[k] = a[k] - b[k];
    }
}

/// Elementwise `out = -a`. Port of `ctaylor.hpp:263-277` (unary operator-).
///
/// ```cpp
/// ctaylor<T, Nvar> operator-(void) const {
///   ctaylor<T, Nvar> res;
///   for (int i = 0; i < POW2(Nvar); i++) res.c[i] = -c[i];
///   return res;
/// }
/// ```
#[cube]
pub fn ctaylor_neg<F: Float>(a: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!(1_u32 << n);
    #[unroll]
    for i in 0..size {
        let k = i as usize;
        out[k] = -a[k];
    }
}

/// Elementwise `out = a * s` where `s` is a scalar. Port of
/// `ctaylor.hpp:421-451` (operator* with scalar on either side).
///
/// ```cpp
/// template <class T, int Nvar, class S>
/// ctaylor<T, Nvar> operator*(const ctaylor<T, Nvar> & t, const S & x) {
///   ctaylor<T, Nvar> tmp;
///   for (int i = 0; i < POW2(Nvar); i++) tmp.c[i] = x * t.c[i];
///   return tmp;
/// }
/// ```
#[cube]
pub fn ctaylor_scalar_mul<F: Float>(
    a: &Array<F>,
    s: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!(1_u32 << n);
    #[unroll]
    for i in 0..size {
        let k = i as usize;
        out[k] = a[k] * s;
    }
}
