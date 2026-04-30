//! C ABI types mirroring `xcfun-master/api/xcfun.h:34-122, 252-262`.
//!
//! - `xcfun_s` — opaque handle wrapping `xcfun_rs::Functional`. C
//!   callers see `xcfun_t *` (== `*mut xcfun_s`).
//! - `xcfun_mode_t` and `xcfun_vars_t` — `#[repr]` mirrors of the
//!   upstream enums. C callers see plain enums; cbindgen (Plan 05-03)
//!   regenerates the C definitions from these Rust types.

#![allow(non_camel_case_types)]

use xcfun_rs::Functional;

/// cbindgen:no-export
///
/// Opaque handle (xcfun_t in C). Owns a heap-allocated `Functional`.
/// The generated `xcfun.h` declares this type via the prelude block in
/// cbindgen.toml (`struct xcfun_s; typedef struct xcfun_s xcfun_t;`) to
/// match upstream `xcfun-master/api/xcfun.h:252-262` byte-for-byte.
/// Rust-side the struct retains its `inner: Functional` field so the
/// FFI implementation in lib.rs can reach the wrapped facade.
#[repr(C)]
pub struct xcfun_s {
    pub(crate) inner: Functional,
}

/// cbindgen:no-export
///
/// `xcfun_mode_t` per xcfun-master/api/xcfun.h:35-41. Discriminants MUST
/// match `xcfun_core::Mode` (Plan 02 D-07: Mode has #[repr(u32)] with
/// Unset=0). The generated `xcfun.h` declares the upstream `xcfun_mode`
/// enum via the cbindgen.toml prelude (D-12). The Rust FFI surface uses
/// raw `c_int`, so cbindgen would not auto-emit this type; the prelude
/// declaration keeps drop-in source compatibility with downstream callers
/// using `XC_PARTIAL_DERIVATIVES` etc.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum xcfun_mode_t {
    XC_MODE_UNSET = 0,
    XC_PARTIAL_DERIVATIVES = 1,
    XC_POTENTIAL = 2,
    XC_CONTRACTED = 3,
    XC_NR_MODES = 4,
}

/// cbindgen:no-export
///
/// `xcfun_vars_t` per xcfun-master/api/xcfun.h:86-122. 31 active variants
/// + UNSET = -1 + NR_VARS = 31. Discriminants MUST match `xcfun_core::Vars`.
/// Verified by the smoke test. Same rationale as `xcfun_mode_t` above —
/// the upstream-equivalent `xcfun_vars` enum is emitted from the
/// cbindgen.toml prelude (D-12).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum xcfun_vars_t {
    XC_VARS_UNSET = -1,
    XC_A = 0,
    XC_N = 1,
    XC_A_B = 2,
    XC_N_S = 3,
    XC_A_GAA = 4,
    XC_N_GNN = 5,
    XC_A_B_GAA_GAB_GBB = 6,
    XC_N_S_GNN_GNS_GSS = 7,
    XC_A_GAA_LAPA = 8,
    XC_A_GAA_TAUA = 9,
    XC_N_GNN_LAPN = 10,
    XC_N_GNN_TAUN = 11,
    XC_A_B_GAA_GAB_GBB_LAPA_LAPB = 12,
    XC_A_B_GAA_GAB_GBB_TAUA_TAUB = 13,
    XC_N_S_GNN_GNS_GSS_LAPN_LAPS = 14,
    XC_N_S_GNN_GNS_GSS_TAUN_TAUS = 15,
    XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB = 16,
    XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB = 17,
    XC_N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS = 18,
    XC_A_AX_AY_AZ = 19,
    XC_A_B_AX_AY_AZ_BX_BY_BZ = 20,
    XC_N_NX_NY_NZ = 21,
    XC_N_S_NX_NY_NZ_SX_SY_SZ = 22,
    XC_A_AX_AY_AZ_TAUA = 23,
    XC_A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB = 24,
    XC_N_NX_NY_NZ_TAUN = 25,
    XC_N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS = 26,
    XC_A_2ND_TAYLOR = 27,
    XC_A_B_2ND_TAYLOR = 28,
    XC_N_2ND_TAYLOR = 29,
    XC_N_S_2ND_TAYLOR = 30,
    XC_NR_VARS = 31,
}
