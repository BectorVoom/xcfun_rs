//! Bit-flag index constants for `CTaylor<F, N>` coefficient arrays.
//!
//! Port of `xcfun-master/external/upstream/taylor/ctaylor.hpp:12-20`
//! (verbatim `#define` values). These are host-visible `pub const`
//! per CONTEXT.md D-06; inside `#[cube]` fns they are passed via
//! `#[comptime]` parameters.
//!
//! # C++ source (`ctaylor.hpp:12-20`)
//! ```cpp
//! #define CNST 0 // avoid defining CONST
//! #define VAR0 1
//! #define VAR1 2
//! #define VAR2 4
//! #define VAR3 8
//! #define VAR4 16
//! #define VAR5 32
//! #define VAR6 64
//! #define VAR7 128
//! ```

pub const CNST: u32 = 0;
pub const VAR0: u32 = 1;
pub const VAR1: u32 = 2;
pub const VAR2: u32 = 4;
pub const VAR3: u32 = 8;
pub const VAR4: u32 = 16;
pub const VAR5: u32 = 32;
pub const VAR6: u32 = 64;
pub const VAR7: u32 = 128;
