// Crate-internal `unsafe` audit: `xcfun-kernels` has no `unsafe` blocks
// today (kernel bodies are pure `#[cube]` Rust). Match the xcfun-eval
// pattern and forbid `unsafe_code` in non-test builds; the `testing` feature
// (when ever flipped here) downgrades to `deny` to leave room for
// `launch_unchecked` adapters in any tests that move into this crate later.
#![cfg_attr(not(feature = "testing"), forbid(unsafe_code))]
#![cfg_attr(feature = "testing", deny(unsafe_code))]

//! # xcfun-kernels
//!
//! Per-functional `#[cube] fn` kernel bodies + `DensVarsDev<F>` + `dispatch_kernel`.
//!
//! **Runtime-agnostic** — never instantiates a `cubecl::Runtime`; depends only
//! on `cubecl` core. Per Phase 6 CONTEXT D-08 (resurrects design-doc-05 §3
//! crate layout).
//!
//! Consumers:
//!   - `xcfun-eval` for per-point CPU validation (via cubecl-cpu).
//!   - `xcfun-gpu` for batch GPU dispatch (via cubecl-hip / cubecl-cuda / cubecl-wgpu).
//!
//! # Compile-time f64 invariant (Phase 6 Pitfall 2)
//!
//! cubecl-wgpu silently downgrades to f32 on devices without
//! `wgpu::Features::SHADER_F64` — see CLAUDE.md GPU constraint. This crate's
//! kernels are written against f64; a static assertion guards the type-size
//! invariant at compile time so an accidental f32 monomorphisation is caught
//! by the compiler rather than by a 1e-12 tier-3 parity regression.
const _: () = assert!(core::mem::size_of::<f64>() == 8);

pub mod density_vars;
pub mod dispatch;
pub mod functionals;
