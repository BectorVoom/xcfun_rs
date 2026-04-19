//! `ctaylor_rec<T, Nvar>` recursion ported from
//! `xcfun-master/external/upstream/taylor/ctaylor.hpp:41-152`. Per-N
//! specialization (N ∈ 0..=7) via comptime-matched outer-dispatch fns
//! (`ctaylor_mul`, `ctaylor_multo`, `ctaylor_multo_skipconst`,
//! `ctaylor_compose`). Inner per-N primitives are named
//! `ctaylor_mul_set_n{k}`, `ctaylor_mul_acc_n{k}`, `ctaylor_multo_n{k}`,
//! `ctaylor_multo_skipconst_n{k}`, `ctaylor_compose_n{k}`.
//!
//! ## General recursion shape (ctaylor.hpp:41-65)
//!
//! ```cpp
//! template <class T, int Nvar> struct ctaylor_rec {
//!   static void mul     (T * d, const T * x, const T * y);  // d += x*y
//!   static void mul_set (T * d, const T * x, const T * y);  // d  = x*y
//!   static void multo   (T * d,                const T * y);  // d *= y
//!   static void multo_skipconst(T * d,         const T * y);
//!   static void compose (T * r, const T * x, const T f[]);
//! };
//! ```
//!
//! ## Algorithmic-identity mandate (CONTEXT.md D-08)
//!
//! The C++ recursion's **write order** *is* the summation order for
//! `multo`; any deviation breaks the 1e-12 parity contract (RESEARCH.md
//! Pitfall P3). The per-N specializations in `multo.rs` preserve the
//! descending write order verbatim. All `> 2`-operand sums use explicit
//! `let`-chain bindings to defeat compiler re-association.

pub mod compose;
pub mod mul;
pub mod multo;
