//! OPTX family kernels — Wave 3 (plan 03-03), GGA-05.
//!
//! - `optx`     — XC_OPTX (id=17): full Handy-Cohen OPTX exchange.
//! - `optxcorr` — XC_OPTXCORR (id=18): correction-only part.
//!
//! Both port the inline body in `optx.cpp:18-26` and `optxcorr.cpp:18-33`.

pub mod optx;
pub mod optxcorr;
