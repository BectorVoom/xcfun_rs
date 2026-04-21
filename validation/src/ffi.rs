//! FFI shim — `unsafe extern "C"` declarations + RAII wrapper over
//! xcfun-master's C ABI.
//!
//! Source: `xcfun-master/api/xcfun.h:1-388` (every `XCFun_API` declaration
//! used by the tier-2 harness).
//!
//! This is the ONLY place in the validation crate where `unsafe` is used;
//! the rest of the codebase goes through the safe `CppXcfun` RAII wrapper.
//! Per Plan 02-06 CONTEXT D-14, the `validation` crate is the one place
//! `unsafe extern "C"` is permitted in the xcfun graph (ACC-01).

#![allow(unsafe_code)]

use std::ffi::{CString, c_void};

unsafe extern "C" {
    pub fn xcfun_new() -> *mut c_void;
    pub fn xcfun_delete(fun: *mut c_void);
    pub fn xcfun_set(fun: *mut c_void, name: *const i8, value: f64) -> i32;
    pub fn xcfun_eval_setup(fun: *mut c_void, vars: u32, mode: u32, order: i32) -> i32;
    pub fn xcfun_input_length(fun: *const c_void) -> i32;
    pub fn xcfun_output_length(fun: *const c_void) -> i32;
    pub fn xcfun_eval(fun: *const c_void, density: *const f64, result: *mut f64);
}

/// RAII wrapper around the C `xcfun_t *` opaque handle. `Drop` calls `xcfun_delete`.
///
/// NOT `Send`/`Sync` — xcfun's C++ reference is single-threaded per-handle; each
/// tier-2 driver loop iteration creates its own `CppXcfun`.
pub struct CppXcfun {
    handle: *mut c_void,
}

impl CppXcfun {
    /// Create a new C-side xcfun handle. Panics if `xcfun_new` returns null
    /// (would indicate a fundamental xcfun-master build failure).
    pub fn new() -> Self {
        let handle = unsafe { xcfun_new() };
        assert!(!handle.is_null(), "xcfun_new returned null");
        Self { handle }
    }

    /// `xcfun_set(name, value)` — activates the named functional with the given weight.
    /// Returns the i32 status from C++ (0 = success; non-zero = xcfun error code).
    pub fn set(&mut self, name: &str, value: f64) -> i32 {
        let cname = CString::new(name).expect("functional name contained NUL byte");
        unsafe { xcfun_set(self.handle, cname.as_ptr(), value) }
    }

    /// `xcfun_eval_setup(vars, mode, order)`.
    pub fn eval_setup(&mut self, vars: u32, mode: u32, order: i32) -> i32 {
        unsafe { xcfun_eval_setup(self.handle, vars, mode, order) }
    }

    /// `xcfun_input_length(fun)`.
    pub fn input_length(&self) -> usize {
        unsafe { xcfun_input_length(self.handle) as usize }
    }

    /// `xcfun_output_length(fun)`.
    pub fn output_length(&self) -> usize {
        unsafe { xcfun_output_length(self.handle) as usize }
    }

    /// `xcfun_eval(input, output)`. Panics on length mismatch to catch FFI
    /// drift before calling into unchecked C++.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) {
        let il = self.input_length();
        let ol = self.output_length();
        assert_eq!(
            input.len(),
            il,
            "FFI eval: input.len() = {} but xcfun_input_length = {}",
            input.len(),
            il
        );
        assert_eq!(
            output.len(),
            ol,
            "FFI eval: output.len() = {} but xcfun_output_length = {}",
            output.len(),
            ol
        );
        unsafe { xcfun_eval(self.handle, input.as_ptr(), output.as_mut_ptr()) };
    }
}

impl Drop for CppXcfun {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { xcfun_delete(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}
