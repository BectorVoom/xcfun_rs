//! C ABI drop-in replacement for xcfun-master/api/xcfun.h.
//! Every `XCFun_API` symbol in the upstream header has a matching
//! `#[unsafe(no_mangle)] pub extern "C" fn` here, wrapped in `c_entry!`
//! (Plan 05-02 D-05 + D-06 + D-07).
//!
//! Layering: depends on `xcfun-rs` (the public Rust facade), NOT on
//! `xcfun-eval` or `xcfun-core` directly (CONTEXT "Integration Points").

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod c_entry;
pub mod types;

use std::ffi::{CStr, CString, c_char, c_double, c_int, c_uint};
use std::sync::{Mutex, OnceLock};

use xcfun_rs::{Functional, Mode, Vars};

pub use c_entry::{die_from_panic, die_with, run_caught};
pub use types::{xcfun_mode_t, xcfun_s, xcfun_vars_t};

// ---------------------------------------------------------------------
//  Internal helper — convert an i32 vars/mode code from the C side back
//  into the strongly-typed Rust enum. Returns None for out-of-range.
// ---------------------------------------------------------------------

#[inline]
fn vars_from_i32(v: c_int) -> Option<Vars> {
    match v {
        0 => Some(Vars::A),
        1 => Some(Vars::N),
        2 => Some(Vars::A_B),
        3 => Some(Vars::N_S),
        4 => Some(Vars::A_GAA),
        5 => Some(Vars::N_GNN),
        6 => Some(Vars::A_B_GAA_GAB_GBB),
        7 => Some(Vars::N_S_GNN_GNS_GSS),
        8 => Some(Vars::A_GAA_LAPA),
        9 => Some(Vars::A_GAA_TAUA),
        10 => Some(Vars::N_GNN_LAPN),
        11 => Some(Vars::N_GNN_TAUN),
        12 => Some(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB),
        13 => Some(Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
        14 => Some(Vars::N_S_GNN_GNS_GSS_LAPN_LAPS),
        15 => Some(Vars::N_S_GNN_GNS_GSS_TAUN_TAUS),
        16 => Some(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB),
        17 => Some(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB),
        18 => Some(Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS),
        19 => Some(Vars::A_AX_AY_AZ),
        20 => Some(Vars::A_B_AX_AY_AZ_BX_BY_BZ),
        21 => Some(Vars::N_NX_NY_NZ),
        22 => Some(Vars::N_S_NX_NY_NZ_SX_SY_SZ),
        23 => Some(Vars::A_AX_AY_AZ_TAUA),
        24 => Some(Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB),
        25 => Some(Vars::N_NX_NY_NZ_TAUN),
        26 => Some(Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS),
        27 => Some(Vars::A_2ND_TAYLOR),
        28 => Some(Vars::A_B_2ND_TAYLOR),
        29 => Some(Vars::N_2ND_TAYLOR),
        30 => Some(Vars::N_S_2ND_TAYLOR),
        _ => None,
    }
}

#[inline]
fn mode_from_i32(m: c_int) -> Option<Mode> {
    match m {
        0 => Some(Mode::Unset),
        1 => Some(Mode::PartialDerivatives),
        2 => Some(Mode::Potential),
        3 => Some(Mode::Contracted),
        _ => None,
    }
}

// ---------------------------------------------------------------------
//  Internal helper — convert Option<&'static str> to a NUL-terminated
//  C-string pointer or NULL.
//
//  Implementation: a process-wide `Mutex<Vec<CString>>` cache. For any
//  given `&'static str s`, repeated calls return the same `*const c_char`
//  pointer for the program lifetime, and that pointer is NUL-terminated.
//  This avoids per-call allocation on the read path; the cache is
//  populated lazily on first encounter of each name. The lookup tables
//  in xcfun-core have ~80 functional names + 4 parameters + 46 aliases
//  (plus `describe_short` / `describe_long` strings) so the cache is
//  bounded.
//
//  This helper is NEVER called from the eval hot path — eval does not
//  return strings to C.
// ---------------------------------------------------------------------

static C_NAMES: OnceLock<Mutex<Vec<CString>>> = OnceLock::new();

fn null_or_cstr(opt: Option<&'static str>) -> *const c_char {
    match opt {
        None => std::ptr::null(),
        Some(s) => {
            let m = C_NAMES.get_or_init(|| Mutex::new(Vec::new()));
            let mut v = m.lock().unwrap();
            if let Some(existing) = v.iter().find(|c| c.to_bytes() == s.as_bytes()) {
                return existing.as_ptr();
            }
            let c = CString::new(s).expect("name contains interior NUL");
            let ptr = c.as_ptr();
            v.push(c);
            ptr
        }
    }
}

// =====================================================================
//  Free-function exports (RS-09 / xcfun.h:128-250)
// =====================================================================

/// xcfun.h:128.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_version() -> *const c_char {
    c_entry!("xcfun_version" => {
        // env!("CARGO_PKG_VERSION") expands to a &'static str at compile
        // time; concat with NUL gives a static NUL-terminated C string.
        static VERSION_C: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
        VERSION_C.as_ptr() as *const c_char
    })
}

/// xcfun.h:138.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_splash() -> *const c_char {
    c_entry!("xcfun_splash" => {
        static SPLASH_C: &[u8] =
            concat!(include_str!("../../xcfun-rs/assets/splash.txt"), "\0").as_bytes();
        SPLASH_C.as_ptr() as *const c_char
    })
}

/// xcfun.h:143.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_authors() -> *const c_char {
    c_entry!("xcfun_authors" => {
        static AUTHORS_C: &[u8] =
            concat!(include_str!("../../xcfun-rs/assets/authors.txt"), "\0").as_bytes();
        AUTHORS_C.as_ptr() as *const c_char
    })
}

/// xcfun.h:150.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_test() -> c_int {
    c_entry!("xcfun_test" => {
        xcfun_rs::self_test() as c_int
    })
}

/// xcfun.h:159.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_is_compatible_library() -> bool {
    c_entry!("xcfun_is_compatible_library" => {
        xcfun_rs::is_compatible_library()
    })
}

/// xcfun.h:215. C++ dies on out-of-range; we mirror via die_with.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_which_vars(
    func_type: c_uint,
    dens_type: c_uint,
    laplacian: c_uint,
    kinetic: c_uint,
    current: c_uint,
    explicit_derivatives: c_uint,
) -> c_int {
    c_entry!("xcfun_which_vars" => {
        match xcfun_rs::which_vars(
            func_type, dens_type, laplacian, kinetic, current, explicit_derivatives,
        ) {
            Some(v) => v as c_int,
            None => die_with("xcfun_which_vars: invalid input"),
        }
    })
}

/// xcfun.h:226.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_which_mode(mode_type: c_uint) -> c_int {
    c_entry!("xcfun_which_mode" => {
        match xcfun_rs::which_mode(mode_type) {
            Some(m) => m as c_int,
            None => die_with("xcfun_which_mode: invalid input"),
        }
    })
}

/// xcfun.h:232.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_enumerate_parameters(param: c_int) -> *const c_char {
    c_entry!("xcfun_enumerate_parameters" => {
        null_or_cstr(xcfun_rs::enumerate_parameters(param))
    })
}

/// xcfun.h:238.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_enumerate_aliases(n: c_int) -> *const c_char {
    c_entry!("xcfun_enumerate_aliases" => {
        null_or_cstr(xcfun_rs::enumerate_aliases(n))
    })
}

/// xcfun.h:244.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_describe_short(name: *const c_char) -> *const c_char {
    c_entry!("xcfun_describe_short", name => {
        let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => die_with("xcfun_describe_short: invalid UTF-8 in name"),
        };
        null_or_cstr(xcfun_rs::describe_short(name_str))
    })
}

/// xcfun.h:250.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_describe_long(name: *const c_char) -> *const c_char {
    c_entry!("xcfun_describe_long", name => {
        let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => die_with("xcfun_describe_long: invalid UTF-8 in name"),
        };
        null_or_cstr(xcfun_rs::describe_long(name_str))
    })
}

// =====================================================================
//  Handle lifecycle (xcfun.h:271-276)
// =====================================================================

/// xcfun.h:271.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_new() -> *mut xcfun_s {
    c_entry!("xcfun_new" => {
        Box::into_raw(Box::new(xcfun_s { inner: Functional::new() }))
    })
}

/// xcfun.h:276. NULL-safe per CAPI-03 — does NOT use c_entry!'s NULL guard.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_delete(fun: *mut xcfun_s) {
    // CAPI-03: silent no-op on null. Mirrors C++ `delete (T*)nullptr`.
    if fun.is_null() { return; }
    // SAFETY: caller MUST have obtained `fun` from xcfun_new and not
    // already deleted it. Library cannot detect double-delete.
    unsafe { drop(Box::from_raw(fun)); }
}

// =====================================================================
//  Per-functional setters / getters (xcfun.h:284-306)
// =====================================================================

/// xcfun.h:284.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_set(
    fun: *mut xcfun_s,
    name: *const c_char,
    value: c_double,
) -> c_int {
    c_entry!("xcfun_set", fun, name => {
        let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => die_with("xcfun_set: invalid UTF-8 in name"),
        };
        match unsafe { &mut (*fun).inner }.set(name_str, value) {
            Ok(()) => 0,
            Err(e) => e.as_c_code(),
        }
    })
}

/// xcfun.h:294. Note `value: *mut c_double`.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_get(
    fun: *const xcfun_s,
    name: *const c_char,
    value: *mut c_double,
) -> c_int {
    c_entry!("xcfun_get", fun, name, value => {
        let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => die_with("xcfun_get: invalid UTF-8 in name"),
        };
        match unsafe { &(*fun).inner }.get(name_str) {
            Ok(v) => {
                unsafe { *value = v; }
                0
            }
            Err(e) => e.as_c_code(),
        }
    })
}

/// xcfun.h:300.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_is_gga(fun: *const xcfun_s) -> bool {
    c_entry!("xcfun_is_gga", fun => {
        unsafe { &(*fun).inner }.is_gga()
    })
}

/// xcfun.h:306.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_is_metagga(fun: *const xcfun_s) -> bool {
    c_entry!("xcfun_is_metagga", fun => {
        unsafe { &(*fun).inner }.is_metagga()
    })
}

// =====================================================================
//  Setup + length + eval (xcfun.h:315-388)
// =====================================================================

/// xcfun.h:315.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_eval_setup(
    fun: *mut xcfun_s,
    vars: c_int,
    mode: c_int,
    order: c_int,
) -> c_int {
    c_entry!("xcfun_eval_setup", fun => {
        let v = match vars_from_i32(vars) {
            Some(v) => v,
            None => die_with("xcfun_eval_setup: invalid vars"),
        };
        let m = match mode_from_i32(mode) {
            Some(m) => m,
            None => die_with("xcfun_eval_setup: invalid mode"),
        };
        if order < 0 {
            // C++ XCFunctional.cpp:443 returns XC_EORDER for negative.
            return 1;
        }
        match unsafe { &mut (*fun).inner }.eval_setup(v, m, order as u32) {
            Ok(()) => 0,
            Err(e) => e.as_c_code(),
        }
    })
}

/// xcfun.h:333.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_user_eval_setup(
    fun: *mut xcfun_s,
    order: c_int,
    func_type: c_uint,
    dens_type: c_uint,
    mode_type: c_uint,
    laplacian: c_uint,
    kinetic: c_uint,
    current: c_uint,
    explicit_derivatives: c_uint,
) -> c_int {
    c_entry!("xcfun_user_eval_setup", fun => {
        match unsafe { &mut (*fun).inner }.user_eval_setup(
            order, func_type, dens_type, mode_type,
            laplacian, kinetic, current, explicit_derivatives,
        ) {
            Ok(()) => 0,
            Err(e) => e.as_c_code(),
        }
    })
}

/// xcfun.h:347.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_input_length(fun: *const xcfun_s) -> c_int {
    c_entry!("xcfun_input_length", fun => {
        unsafe { &(*fun).inner }.input_length() as c_int
    })
}

/// xcfun.h:356.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_output_length(fun: *const xcfun_s) -> c_int {
    c_entry!("xcfun_output_length", fun => {
        match unsafe { &(*fun).inner }.output_length() {
            Ok(n) => n as c_int,
            Err(e) => die_with(&format!(
                "xcfun_output_length: {} -- did you call xcfun_eval_setup?", e
            )),
        }
    })
}

/// xcfun.h:366. VOID return — die_with on Err per D-06.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_eval(
    fun: *const xcfun_s,
    density: *const c_double,
    result: *mut c_double,
) {
    c_entry!("xcfun_eval", fun, density, result => {
        let f = unsafe { &(*fun).inner };
        // Plan 05-04 fix: input buffer length is `inlen × (1 << order)`
        // for Mode::Contracted (per D-06-A, mirroring xcfun-master/src/
        // XCFunctional.cpp:622-627 DOEVAL macro), and `inlen` for
        // Mode::PartialDerivatives / Mode::Potential. Without the
        // mode-aware length, Contracted mode evaluations fault with
        // an InputLengthMismatch from the inner Functional::eval.
        let in_buf_len = f.input_buffer_length();
        let outlen = match f.output_length() {
            Ok(n) => n,
            Err(e) => die_with(&format!("xcfun_eval: output_length failed: {}", e)),
        };
        let input = unsafe { std::slice::from_raw_parts(density, in_buf_len) };
        let output = unsafe { std::slice::from_raw_parts_mut(result, outlen) };
        if let Err(e) = f.eval(input, output) {
            die_with(&format!(
                "xcfun_eval: {} -- did you call xcfun_eval_setup?", e
            ));
        }
    })
}

/// xcfun.h:382. VOID return — die_with on Err per D-06.
#[unsafe(no_mangle)]
pub extern "C" fn xcfun_eval_vec(
    fun: *const xcfun_s,
    nr_points: c_int,
    density: *const c_double,
    density_pitch: c_int,
    result: *mut c_double,
    result_pitch: c_int,
) {
    c_entry!("xcfun_eval_vec", fun, density, result => {
        if nr_points < 0 {
            die_with("xcfun_eval_vec: nr_points must be non-negative");
        }
        if density_pitch < 0 || result_pitch < 0 {
            die_with("xcfun_eval_vec: pitches must be non-negative");
        }
        let f = unsafe { &(*fun).inner };
        let inlen = f.input_length();
        let outlen = match f.output_length() {
            Ok(n) => n,
            Err(e) => die_with(&format!("xcfun_eval_vec: output_length failed: {}", e)),
        };
        let dp = density_pitch as usize;
        let rp = result_pitch as usize;
        for k in 0..(nr_points as usize) {
            let in_ptr = unsafe { density.add(k * dp) };
            let out_ptr = unsafe { result.add(k * rp) };
            let in_slice = unsafe { std::slice::from_raw_parts(in_ptr, inlen) };
            let out_slice = unsafe { std::slice::from_raw_parts_mut(out_ptr, outlen) };
            if let Err(e) = f.eval(in_slice, out_slice) {
                die_with(&format!(
                    "xcfun_eval_vec: point {} eval failed: {} -- did you call xcfun_eval_setup?",
                    k, e
                ));
            }
        }
    })
}
