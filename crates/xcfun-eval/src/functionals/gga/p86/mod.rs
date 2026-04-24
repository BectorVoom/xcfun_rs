//! P86 family kernels — Wave 3 (plan 03-03), GGA-07.
//!
//! - `p86c`     — XC_P86C (id=56): full P86 correlation (LSDA + gradient).
//! - `p86corrc` — XC_P86CORRC (id=57): P86 gradient correction only.

pub mod p86c;
pub mod p86corrc;
