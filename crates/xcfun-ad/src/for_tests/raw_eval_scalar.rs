//! `raw_eval_scalar` — launch a 1-thread cubecl-cpu kernel with a prepared
//! input buffer, read back the output buffer, return its contents as a Vec.
//!
//! Primitive helper consumed by `tests/golden_*.rs` and `tests/props_*.rs`.
//! Not part of the public API (feature = "testing" only; D-16).
//!
//! # Cubecl 0.10-pre.3 API deltas vs plan <interfaces>
//!
//! - `client.create(bytes)` → `client.create_from_slice(bytes)`
//! - `client.read_one(handle.binding()) -> Vec<u8>` →
//!   `client.read_one_unchecked(handle) -> Bytes` (takes owned `Handle`,
//!   returns `cubecl_common` `Bytes` which derefs to `[u8]`).
//! - `ArrayArg::from_raw_parts::<F>(&handle, len, vec)` →
//!   `ArrayArg::from_raw_parts(handle.clone(), len)` (unsafe, 2 args,
//!   owns the handle — no turbofish, no vectorization argument).
//!
//! Consequently the `launcher` closure receives an OWNED pair of
//! `cubecl::server::Handle`s and the `&CpuClient`. The launcher is
//! expected to call `.clone()` on each handle inside `ArrayArg::from_raw_parts`
//! (the raw-part APIs in cubecl 0.10-pre.3 all use `.clone()`
//! — see `cubecl-core/src/runtime_tests/launch.rs`).

use crate::for_tests::cpu_client::{CpuClient, cpu_client};
use cubecl::prelude::*;
use cubecl::server::Handle;

/// Copy `inputs` into a fresh device buffer, allocate `out_len` f64s of
/// output, run `launcher(client, in_handle, out_handle)` once, then download
/// and return the output as a `Vec<f64>`.
///
/// `launcher` is the kernel-launch closure (caller writes the
/// unsafe kernel-launch block in the test body itself). The closure
/// receives OWNED `Handle`s so it can `.clone()` them into `ArrayArg`s.
/// After the closure returns, `raw_eval_scalar` reads the output handle
/// back and returns its f64 contents.
///
/// This is intentionally low-level: each downstream test supplies its own
/// `launcher` closure so kernel-specific `#[comptime]` args can be threaded
/// in without a generic soup.
pub fn raw_eval_scalar(
    inputs: &[f64],
    out_len: usize,
    launcher: impl FnOnce(&CpuClient, Handle, Handle),
) -> Vec<f64> {
    let client = cpu_client();
    let in_handle = client.create_from_slice(f64::as_bytes(inputs));
    let out_handle = client.empty(out_len * core::mem::size_of::<f64>());

    // Clone the output handle before handing ownership to the launcher so
    // we can still read it back afterwards.
    let read_handle = out_handle.clone();
    launcher(client, in_handle, out_handle);

    let bytes = client.read_one_unchecked(read_handle);
    f64::from_bytes(&bytes).to_vec()
}
