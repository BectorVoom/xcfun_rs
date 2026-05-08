//! KT/BTK kernels (GGA-10 minus LB94 per D-19, minus CSC per D-01-A).
//!
//! Wave 4 (plan 03-04) ships:
//! - `ktx`  — XC_KTX (id=23): Keal-Tozer GGA exchange correction (ktx.cpp:18-24).
//! - `btk`  — XC_BTK (id=58): Borgoo-Tozer kinetic functional (btk.cpp:17-27).
//!
//! # Deferral notes
//! - **CSC (XC_CSC, id=66)** declares `XC_KINETIC | XC_LAPLACIAN | XC_JP`
//!   dependencies — metaGGA-class. Deferred to Phase 4 alongside BRX/BRC/BRXC.
//!   See `.planning/phases/03-gga-tier-mode-potential/03-CONTEXT.md` D-01-A.
//! - **LB94 (legacy `setup_lb94` pattern, not `FUNCTIONAL`)** is not in the
//!   78-entry FunctionalId enum; deferred to Phase 5 per D-19.

pub mod btk;
pub mod ktx;
