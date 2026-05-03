//! Per-functional `#[cube] fn` kernel bodies — runtime-agnostic.
//!
//! Phase 6 Plan 06-01 (D-08) moved this tree out of `xcfun-eval` into
//! `xcfun-kernels` so kernel bodies can be consumed by both the per-point
//! cubecl-cpu validation substrate (in `xcfun-eval`) and the batch GPU
//! dispatch (in `xcfun-gpu`) without dragging a runtime dependency through
//! the kernel-source crate.
//!
//! Sub-modules:
//!   - `lda`, `gga`, `mgga` — 78 per-functional kernel bodies, organised by
//!     functional family (line-for-line algorithmic ports of `xcfun-master/
//!     src/functionals/<name>.cpp`).
//!   - `potential` — `#[cube]`-only Mode::Potential adapter kernels that
//!     wrap `dispatch::dispatch_kernel` at fixed N=1 / N=2 (LDA-direct +
//!     GGA-divergence integrand). Host-side launchers (`launch_potential_lda` /
//!     `launch_potential_gga`) live in `xcfun-eval::Functional`.
//!
//! The host-side `launch_contracted` (Mode::Contracted dispatcher) STAYS
//! in `xcfun-eval/src/functionals/contracted.rs` because it depends on
//! `xcfun_eval::functional::Functional` + `run_launch` — both of which
//! belong with the per-point substrate per D-08.

pub mod lda;
pub mod gga;
pub mod mgga;
pub mod potential;
