//! MetaGGA (Meta-Generalised-Gradient-Approximation) functional bodies.
//!
//! Phase 4 ships **32 functional IDs** per D-01 (28 metaGGA + 4 carryovers
//! BRX/BRC/BRXC + CSC). LB94 (id=66) deferred to Phase 5 per D-13.
//!
//! # Layout
//!
//! - `shared::` — cross-family helpers (constants, tpss_like, scan_like,
//!   m0x_like, br_like, blocx, cs). Populated Wave 0 (this plan, 04-00).
//! - Family modules land in plans 04-01 (TPSS+BR+CSC), 04-02 (SCAN),
//!   04-03 (M0x+BLOCX). Plan 04-00 ONLY ships the `shared::` substrate; no
//!   per-functional kernels at this stage.
//!
//! # Wave breakdown (per CONTEXT D-01-A)
//!
//! - Wave 1 (04-01): TPSS family (5) + BR family (3) + CSC (1).
//! - Wave 2 (04-02): SCAN family (10) — heavy on `shared/scan_like.rs`.
//! - Wave 3 (04-03): M05 + M06 + BLOCX (13) — heavy on `shared/m0x_like.rs`
//!   and `shared/blocx.rs`.

pub mod shared;

// Wave 1 (04-01): TPSS family + BR + CSC
pub mod tpssx;
pub mod tpssc;
pub mod revtpssx;
pub mod revtpssc;
pub mod tpsslocc;
// Wave 1 (04-01): BR family (brx.rs contains BRX/BRC/BRXC kernels) + CSC.
pub mod brx;   // contains brx_kernel, brc_kernel, brxc_kernel
pub mod brc;   // thin re-export of brc_kernel from brx
pub mod brxc;  // thin re-export of brxc_kernel from brx
pub mod csc;

// Wave 2 (04-02): SCAN family — modules added in plan 04-02.
// Wave 3 (04-03): M0x + BLOCX — modules added in plan 04-03.
