//! Validation crate library surface — exposes fixtures, driver, ffi, report
//! modules so they can be tested independently AND used by the `validation`
//! binary.
//!
//! Per Plan 02-06 CONTEXT D-14: this crate is the ONE place in the
//! xcfun library graph where `anyhow` is permitted (app boundary) and
//! `unsafe extern "C"` is permitted (FFI to the vendored xcfun C ABI).

pub mod driver;
pub mod ffi;
pub mod fixtures;
pub mod report;
