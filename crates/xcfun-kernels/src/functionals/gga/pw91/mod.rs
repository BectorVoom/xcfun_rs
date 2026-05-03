//! PW91-family kernels — Wave 3 (plan 03-03), GGA-06.
//!
//! - `pw86x` — XC_PW86X (id=1): Perdew-Wang 1986 exchange.
//! - `pw91x` — XC_PW91X (id=26): Perdew-Wang 1991 exchange.
//! - `pw91c` — XC_PW91C (id=77): Perdew-Wang 1991 correlation (longest GGA body).
//! - `pw91k` — XC_PW91K (id=27): Perdew-Wang 1991 kinetic-energy GGA.

pub mod pw86x;
pub mod pw91x;
pub mod pw91c;
pub mod pw91k;
