//! BRC kernel — thin re-export from `brx.rs`.
//!
//! All three Becke-Roussel kernels (BRX, BRC, BRXC) are implemented in
//! `brx.rs` (mirroring the single-file structure of `xcfun-master/src/functionals/brx.cpp`).
pub use crate::functionals::mgga::brx::brc_kernel;
