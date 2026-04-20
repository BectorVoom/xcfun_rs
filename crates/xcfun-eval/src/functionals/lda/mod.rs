//! LDA tier — 11 functionals (Plans 02-04 + 02-05).
//!
//! Plan 02-04 ships the 9 pure-density bodies (use XC_A_B builder):
//!   slaterx, vwn3c, vwn5c, pw92c, pz81c, ldaerfx, ldaerfc, ldaerfc_jt, tfk
//!
//! Plan 02-05 ships the 2 kinetic-GGA bodies (use XC_A_B_GAA_GAB_GBB builder):
//!   tw, vwk

// Modules added by Plans 02-04 and 02-05.
pub mod slaterx; // 02-04
pub mod vwn3c; // 02-04
pub mod vwn5c; // 02-04
pub mod vwn_eps; // 02-04 (shared vwn3_eps/vwn5_eps helpers)
pub mod pw92c; // 02-04
pub mod pw92eps; // 02-04 (shared pw92_eps helper)
pub mod pz81c; // 02-04
pub mod ldaerfx; // 02-04
pub mod ldaerfc; // 02-04
pub mod ldaerfc_jt; // 02-04
pub mod tfk; // 02-04
// pub mod tw;          // 02-05
// pub mod vwk;         // 02-05
