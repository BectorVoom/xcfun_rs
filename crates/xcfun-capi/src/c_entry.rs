//! `c_entry!` macro — panic-trap + NULL-pointer guard envelope
//! wrapping every `extern "C" fn` body in this crate (Phase 5 D-05).
//!
//! # Behaviour (D-05 + D-06 + D-07)
//! 1. Each `$ptr` arg is checked against null BEFORE running `body`;
//!    on null the macro prints `"xcfun: null pointer to {fn_name}
//!    (arg `{ptr}`)"` to stderr and `abort()`s.
//! 2. Body is wrapped in `std::panic::catch_unwind(AssertUnwindSafe(...))`.
//! 3. On Ok(value) — value returned to C.
//! 4. On panic — payload downcast for the message; print
//!    `"xcfun: died from panic in {fn_name}: {msg}"` to stderr;
//!    `abort()`.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::abort;

/// Helper — invoked from a void-returning `extern "C" fn` when an
/// internal `Err` reaches the body (D-06). The C++ analog is
/// `xcfun::die(msg, 0)` at `xcfun-master/src/functional.hpp`.
pub fn die_with(msg: &str) -> ! {
    eprintln!("{msg}");
    abort();
}

/// Helper — invoked from `c_entry!` after a panic. Extracts the panic
/// message and aborts.
pub fn die_from_panic(fn_name: &str, payload: Box<dyn Any + Send>) -> ! {
    let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        String::from("(unknown panic payload)")
    };
    eprintln!("xcfun: died from panic in {fn_name}: {msg}");
    abort();
}

/// Run `body` inside `catch_unwind`, aborting on panic with a
/// diagnostic naming `fn_name`.
#[inline]
pub fn run_caught<R>(fn_name: &'static str, body: impl FnOnce() -> R) -> R {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(v) => v,
        Err(payload) => die_from_panic(fn_name, payload),
    }
}

/// `c_entry!` — top-level macro:
/// ```ignore
/// c_entry!("xcfun_new" => { Box::into_raw(...) });
/// c_entry!("xcfun_set", fun, name => { ... });
/// ```
#[macro_export]
macro_rules! c_entry {
    ($fn_name:literal => { $($body:tt)* }) => {{
        $crate::c_entry::run_caught($fn_name, || { $($body)* })
    }};

    ($fn_name:literal, $($ptr:ident),+ => { $($body:tt)* }) => {{
        $(
            if $ptr.is_null() {
                eprintln!(
                    "xcfun: null pointer to {} (arg `{}`)",
                    $fn_name, stringify!($ptr)
                );
                std::process::abort();
            }
        )+
        $crate::c_entry::run_caught($fn_name, || { $($body)* })
    }};
}
