//! B97 family kernels — Wave 4 (plan 03-04), GGA-09.
//!
//! Six functionals across three C++ source files:
//!
//! - `b97x` — XC_B97X (id=60): B97 exchange (b97xc.cpp:20-23, 36-43).
//! - `b97c` — XC_B97C (id=61): B97 correlation (b97xc.cpp:25-34, 45-52).
//! - `b97_1x` — XC_B97_1X (id=62): B97-1 exchange (b97-1xc.cpp:20-23, 36-43).
//! - `b97_1c` — XC_B97_1C (id=63): B97-1 correlation (b97-1xc.cpp:25-34, 44-50).
//! - `b97_2x` — XC_B97_2X (id=64): B97-2 exchange (b97-2xc.cpp:20-23, 36-43).
//! - `b97_2c` — XC_B97_2C (id=65): B97-2 correlation (b97-2xc.cpp:25-34, 45-52).
//!
//! All six bodies share the same algebraic structure differing only by
//! coefficient table — see `gga/shared/b97_poly.rs` for the common helpers
//! (`ux_ab`, `b97_enhancement`) and `gga/shared/constants.rs` for the
//! per-functional 3-coefficient arrays.
//!
//! # Pitfall G6
//! `b97_enhancement` preserves operator precedence `c₀ + c₁·u + c₂·(u·u)` —
//! NO Horner form. B97-2C is the conditioning-stress canary (`c₂ = -7.44060`).

pub mod b97_1c;
pub mod b97_1x;
pub mod b97_2c;
pub mod b97_2x;
pub mod b97c;
pub mod b97x;
