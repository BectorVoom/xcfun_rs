// App-boundary crate (CLAUDE.md: validation is allowed to use anyhow + unsafe FFI).
// The lints below are stylistic suggestions that don't pay rent for an
// already-tightly-scoped harness:
//   - too_many_arguments / many_single_char_names: parity-test plumbing
//   - module_name_repetitions: mirrors C++ namespace structure
//   - should_implement_trait: from_str helpers are not FromStr (no error type)
//   - new_without_default / branches_sharing_code / manual_range_contains:
//     load-bearing comments make the explicit forms more readable here
#![allow(
    clippy::should_implement_trait,
    clippy::new_without_default,
    clippy::branches_sharing_code,
    clippy::manual_range_contains,
    clippy::doc_overindented_list_items,
    clippy::if_same_then_else,
    clippy::drop_non_drop,
    clippy::redundant_closure
)]

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
