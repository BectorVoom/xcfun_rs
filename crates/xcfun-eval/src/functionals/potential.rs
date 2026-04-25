//! Mode::Potential kernels. Line-for-line port of
//! `xcfun-master/src/XCFunctional.cpp:637-790` (D-13).
//!
//! # LDA path (XCFunctional.cpp:637-670)
//!
//! Launch the active functional kernel at N=1 with VAR0 seeded on the
//! density slot.  The kernel writes `out[CNST]` (energy) and `out[VAR0]`
//! (∂E/∂ρ-component for the seeded direction).  For `nspin=2` the launcher
//! invokes the kernel twice (j=0 → α, j=1 → β) so that `out[VAR0]` carries
//! ∂E/∂ρ_α and ∂E/∂ρ_β respectively.
//!
//! # GGA path (XCFunctional.cpp:671-790)
//!
//! See `potential_gga_kernel` — three-direction N=2 accumulation that reads
//! `out[VAR0|VAR1]` (the divergence integrand `∇·(∂E/∂g)`).  The host-side
//! `launch_potential_gga` then subtracts `Σ_dir Σ_id w_id · out[VAR0|VAR1]`
//! from the LDA-direct potential term already populated by
//! `launch_potential_lda` (XCFunctional.cpp:671 structural invariant — the
//! GGA block runs AFTER the LDA loop, with no `else`, so `output[j+1]` is
//! already set when the divergence subtract runs).
//!
//! # Output layout (D-15)
//!
//!   - `nspin=1` → `[energy, pot_α]`        (Vars::A / Vars::A_2ND_TAYLOR)
//!   - `nspin=2` → `[energy, pot_α, pot_β]` (all spin-resolved Vars)

use cubecl::prelude::*;

use crate::density_vars::DensVarsDev;
use crate::dispatch::dispatch_kernel;

/// Mode::Potential LDA-path kernel (N=1).  Calls `dispatch_kernel` at N=1 so
/// the host can read `out[CNST]` (energy) and `out[VAR0]` (LDA-direct
/// ∂E/∂ρ on the seeded density slot).
///
/// 1:1 port of `XCFunctional.cpp:653-670`:
///
/// ```cpp
/// typedef ctaylor<ireal_t, 1> ttype;
/// // ... seed in[j*inpos].set(VAR0, 1) ...
/// densvars<ttype> d(fun, in);
/// for (int i = 0; i < fun->nr_active_functionals; i++)
///   out += fun->settings[...] * fun->active_functionals[i]->fp1(d);
/// output[j + 1] = out.get(VAR0);
/// output[0]    = out.get(CNST);
/// ```
///
/// `_n` is kept for kernel-signature consistency; the body always dispatches
/// at the literal `1` so cubecl monomorphises the inner kernel at N=1.
#[cube]
pub fn potential_lda_kernel<F: Float>(
    #[comptime] id: u32,
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] _n: u32,
) {
    dispatch_kernel::<F>(id, d, out, 1_u32);
}

/// Mode::Potential GGA-path kernel (N=2).  Called 3× per spin channel by
/// `launch_potential_gga` (one launch per spatial direction x/y/z) — the
/// host reads `out[VAR0|VAR1]` (slot 3, the divergence integrand
/// `∂²E/∂ρ∂g_dir`) and accumulates it as the divergence to subtract.
///
/// 1:1 port of the inner `densvars<ttype> d(fun, in); for ... fp2(d)` block
/// at `XCFunctional.cpp:689-694, 702-707, 714-719` (single-spin) and
/// `:747-752, :763-768, :779-784` (spin-resolved).
///
/// Output coefficient layout at N=2 (size=4, bit-flag indexed):
///   - `out[CNST]        (0)` energy           — NOT consumed (LDA path writes)
///   - `out[VAR0]        (1)` ∂E/∂ρ           — NOT consumed (LDA path writes)
///   - `out[VAR1]        (2)` ∂E/∂g_{dir}     — internal Taylor coefficient
///   - `out[VAR0 | VAR1] (3)` ∂²E/∂ρ∂g_{dir}  — CONSUMED as divergence integrand
#[cube]
pub fn potential_gga_kernel<F: Float>(
    #[comptime] id: u32,
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] _n: u32,
) {
    dispatch_kernel::<F>(id, d, out, 2_u32);
}
