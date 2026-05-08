//! MetaGGA (Meta-Generalised-Gradient-Approximation) functional bodies.
//!
//! Phase 4 ships **32 functional IDs** per D-01 (28 metaGGA + 4 carryovers
//! BRX/BRC/BRXC + CSC). Phase 5 D-16 added LB94 (id=78) as a registry
//! stub; eval returns XcError::Runtime since lb94.cpp is `#if 0`'d
//! upstream. (The earlier comment claiming "id=66" was incorrect —
//! id 66 is XC_CSC.)
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
pub mod revtpssc;
pub mod revtpssx;
pub mod tpssc;
pub mod tpsslocc;
pub mod tpssx;
// Wave 1 (04-01): BR family (brx.rs contains BRX/BRC/BRXC kernels) + CSC.
pub mod brc; // thin re-export of brc_kernel from brx
pub mod brx; // contains brx_kernel, brc_kernel, brxc_kernel
pub mod brxc; // thin re-export of brxc_kernel from brx
pub mod csc;

// Wave 2 (04-02): SCAN family (10 kernels — 5 exchange + 5 correlation).
pub mod r2scanc;
pub mod r2scanx;
pub mod r4scanc;
pub mod r4scanx;
pub mod rppscanc;
pub mod rppscanx;
pub mod rscanc;
pub mod rscanx;
pub mod scanc;
pub mod scanx;

// Wave 3 (04-03): M0x family (12) + BLOCX (1).
pub mod blocx;
pub mod m05c;
pub mod m05x;
pub mod m05x2c;
pub mod m05x2x;
pub mod m06c;
pub mod m06hfc;
pub mod m06hfx;
pub mod m06lc;
pub mod m06lx;
pub mod m06x;
pub mod m06x2c;
pub mod m06x2x;
