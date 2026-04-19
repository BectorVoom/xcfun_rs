//! Scalar `*_expand` series ports mirroring
//! `xcfun-master/external/upstream/taylor/tmath.hpp` byte-for-byte.
//!
//! Every `*_expand` takes a caller-provided `&mut [f64]` and writes Taylor
//! coefficients in-place — no heap allocation, no internal buffers beyond
//! fixed-size stack scratch (`[f64; 8]`) where the C++ source uses a
//! VLA-style `T tmp[Ndeg + 1]`.
//!
//! Plan: `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-02-PLAN.md`.
//! Upstream header comment per module cites the exact tmath.hpp line range
//! per CONTEXT.md D-10.

pub mod inv;
pub mod exp;
pub mod log;
// Task 2 (pow, sqrt, cbrt) and Task 3 (gauss, erf, atan, asinh) land in
// later commits inside Plan 01-02.

pub use inv::inv_expand;
pub use exp::exp_expand;
pub use log::log_expand;
