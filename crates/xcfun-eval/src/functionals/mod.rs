//! Per-functional `#[cube] fn` bodies. Phase 2 ships LDA only (Plans 02-04, 02-05);
//! Phase 3 adds GGAs, Phase 4 adds metaGGAs.
//!
//! Phase 3 plan 03-05 adds `potential` for `Mode::Potential` kernels (LDA N=1
//! + GGA N=2 divergence) per D-13.

pub mod lda;
pub mod gga;
pub mod mgga;
pub mod potential;
