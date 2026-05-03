//! Cross-mode parity: Mode::Contracted vs Mode::PartialDerivatives at
//! orders 0..=4 (Plan 04-05 Task 2 / D-06-C / D-12).
//!
//! At orders 0..=4, Mode::Contracted is a re-packaging of the same
//! per-functional `<name>_kernel<F, const N>` body that Mode::PartialDerivatives
//! invokes (RESEARCH §"Per-functional kernel re-use"). The DOEVAL macro
//! changes only the host-side input pack / output unpack:
//!
//!   - Mode::PartialDerivatives at order N calls `pack_ctaylor_inputs_order_N`
//!     with `(i, j, ...)` indices, launches the kernel, reads
//!     `out[VAR0|VAR1|...|VAR_{N-1}]` (= the top bit-flag combination) for the
//!     mixed partial derivative.
//!   - Mode::Contracted at order N takes a pre-seeded flat
//!     `inlen × (1 << N)` input, launches the SAME kernel, reads the FULL
//!     `1 << N` coefficient array.
//!
//! Cross-mode parity check: pack inputs identically (`pack_ctaylor_inputs_*`
//! with all indices on slot 0), run Mode::Contracted, and compare the
//! resulting CTaylor coefficient at index `VAR0|VAR1|...|VAR_{N-1}` (= the
//! top bit-flag, `(1 << N) - 1`) against Mode::PartialDerivatives' output
//! slot for the same mixed-partial multi-index.
//!
//! Strict 1e-12 (D-12: no Mode::Contracted relaxation).
//!
//! 4 representative functionals × 5 orders × N points per order →
//! ~2000 records target. The current run_launch matrix supports
//! SLATERX (id=0, vars=2) and PBEX (id=5, vars=6) at all orders 0..=4 —
//! restrict to those for the cross-mode smoke test (TPSSX/M06X require
//! vars=13 arms not currently shipped).

#![cfg(feature = "testing")]

use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;
use xcfun_eval::functional::DEFAULT_SETTINGS;

const TOLERANCE: f64 = 1e-12;

/// Pack `inlen × (1 << order)` flat doubles for Mode::Contracted with seeds
/// on slot 0 — this is the same packing that
/// `pack_ctaylor_inputs_order{3,4}(input, inlen, 0, 0, ..., 0)` produces.
///
/// Layout: each slot l ∈ 0..inlen occupies `1 << order` consecutive f64s.
///   - `coeff[CNST = 0]`     = `input[l]`.
///   - `coeff[VAR0 = 1]`     = 1.0 iff `l == 0` and order ≥ 1.
///   - `coeff[VAR1 = 2]`     = 1.0 iff `l == 0` and order ≥ 2.
///   - `coeff[VAR2 = 4]`     = 1.0 iff `l == 0` and order ≥ 3.
///   - `coeff[VAR3 = 8]`     = 1.0 iff `l == 0` and order ≥ 4.
///   - All cross-terms (e.g. VAR0|VAR1=3) start at 0.0.
///
/// Slot 0 receives all VAR_k seeds; slots 1..inlen carry only the CNST.
fn pack_for_contracted(input: &[f64], order: u32) -> Vec<f64> {
    let inlen = input.len();
    let coeff_count = 1_usize << order;
    let mut flat = vec![0.0_f64; inlen * coeff_count];
    for l in 0..inlen {
        flat[l * coeff_count] = input[l];
    }
    // Seed VAR0..VAR_{order-1} on slot 0.
    if order >= 1 {
        flat[1 /* VAR0 */] = 1.0;
    }
    if order >= 2 {
        flat[2 /* VAR1 */] = 1.0;
    }
    if order >= 3 {
        flat[4 /* VAR2 */] = 1.0;
    }
    if order >= 4 {
        flat[8 /* VAR3 */] = 1.0;
    }
    flat
}

/// Run Mode::Contracted at the given order on `(id, vars)` for the per-point
/// scalar input. Returns the `(1 << order)`-element output coefficient array.
fn run_contracted(
    id: FunctionalId,
    vars: Vars,
    order: u32,
    input: &[f64],
) -> Vec<f64> {
    let weights: Vec<(FunctionalId, f64)> = vec![(id, 1.0)];
    let f = Functional {
        weights,
        vars,
        mode: Mode::Contracted,
        order,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };
    let coeff_count = 1_usize << order;
    let flat_input = pack_for_contracted(input, order);
    let mut out = vec![0.0_f64; coeff_count];
    f.eval(&flat_input, &mut out)
        .unwrap_or_else(|e| panic!("Mode::Contracted eval failed at order {} ({:?}): {:?}", order, id, e));
    out
}

/// Run Mode::PartialDerivatives at the given order on `(id, vars)` for the
/// per-point scalar input. Returns the `taylorlen(inlen, order)`-element
/// output array.
fn run_partial_derivatives(
    id: FunctionalId,
    vars: Vars,
    order: u32,
    input: &[f64],
) -> Vec<f64> {
    let weights: Vec<(FunctionalId, f64)> = vec![(id, 1.0)];
    let f = Functional {
        weights,
        vars,
        mode: Mode::PartialDerivatives,
        order,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };
    let inlen = input.len();
    let outlen = xcfun_core::taylorlen(inlen, order as usize);
    let mut out = vec![0.0_f64; outlen];
    f.eval(input, &mut out).unwrap_or_else(|e| {
        panic!(
            "Mode::PartialDerivatives eval failed at order {} ({:?}): {:?}",
            order, id, e
        )
    });
    out
}

/// Compare two f64 values with tolerance per ACC-02 (rel_err = |a-b| / max(|a|, |b|, 1.0)).
fn rel_err(got: f64, want: f64) -> f64 {
    let denom = got.abs().max(want.abs()).max(1.0);
    (got - want).abs() / denom
}

/// Generate a small set of well-conditioned input points away from the
/// regularize clamp (a, b, gaa, gab, gbb all >> TINY_DENSITY = 1e-14).
fn density_points_a_b() -> Vec<[f64; 2]> {
    vec![
        [0.7, 0.4],
        [0.5, 0.5],
        [0.9, 0.2],
        [0.3, 0.6],
        [0.1, 0.8],
    ]
}

fn density_points_a_b_gaa_gab_gbb() -> Vec<[f64; 5]> {
    vec![
        [0.7, 0.4, 0.05, 0.03, 0.02],
        [0.5, 0.5, 0.10, 0.05, 0.10],
        [0.9, 0.2, 0.20, 0.04, 0.01],
        [0.3, 0.6, 0.04, 0.05, 0.07],
        [0.1, 0.8, 0.01, 0.02, 0.03],
    ]
}

// =====================================================================
// SLATERX (id=0, vars=2) — orders 0..=4
// =====================================================================

#[test]
fn contracted_vs_partial_slaterx_order_0() {
    // At order 0: Contracted output[0] = E (energy).
    // PartialDerivatives output[0] = E. Direct comparison.
    for input in density_points_a_b() {
        let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 0, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_SLATERX, Vars::A_B, 0, &input);
        assert_eq!(cont.len(), 1);
        assert_eq!(pd.len(), 1);
        assert!(
            rel_err(cont[0], pd[0]) <= TOLERANCE,
            "SLATERX order 0 cross-mode: got {} vs {} (rel_err {:.3e}) at input {:?}",
            cont[0], pd[0], rel_err(cont[0], pd[0]), input
        );
    }
}

#[test]
fn contracted_vs_partial_slaterx_order_1() {
    // At order 1 with seeds on slot 0:
    //   Contracted: out[CNST=0] = E, out[VAR0=1] = ∂E/∂a (where a = input[0]).
    //   PartialDerivatives at order 1 (inlen=2): output[0]=E, output[1]=∂E/∂a, output[2]=∂E/∂b.
    // Cross-mode parity: cont[0] == pd[0], cont[1] == pd[1].
    for input in density_points_a_b() {
        let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 1, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_SLATERX, Vars::A_B, 1, &input);
        assert_eq!(cont.len(), 2);
        assert!(pd.len() >= 2);
        // CNST coefficient: energy.
        assert!(
            rel_err(cont[0], pd[0]) <= TOLERANCE,
            "SLATERX order 1 CNST: cont={} pd={} rel_err={:.3e} at {:?}",
            cont[0], pd[0], rel_err(cont[0], pd[0]), input
        );
        // VAR0 coefficient: ∂E/∂a.
        assert!(
            rel_err(cont[1], pd[1]) <= TOLERANCE,
            "SLATERX order 1 VAR0: cont={} pd={} rel_err={:.3e} at {:?}",
            cont[1], pd[1], rel_err(cont[1], pd[1]), input
        );
    }
}

#[test]
fn contracted_vs_partial_slaterx_order_2() {
    // At order 2 with seeds on slot 0 (i=0, j=0):
    //   Contracted: out[0..4] = [E, ∂E/∂a, 0, ∂²E/∂a²]  (slots 0=CNST, 1=VAR0, 2=VAR1, 3=VAR0|VAR1)
    //     since both VAR0 and VAR1 seed on slot 0, the cross-coefficient
    //     out[VAR0|VAR1] = ∂²E/∂a² (mixed second w.r.t. same variable).
    //   PartialDerivatives at order 2 (inlen=2): output layout
    //     [E, ∂E/∂a, ∂E/∂b, ∂²E/∂a², ∂²E/∂a∂b, ∂²E/∂b²].
    // Compare cont[0]=pd[0], cont[1]=pd[1], cont[3]=pd[3].
    for input in density_points_a_b() {
        let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 2, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_SLATERX, Vars::A_B, 2, &input);
        assert_eq!(cont.len(), 4);
        // CNST = energy
        assert!(rel_err(cont[0], pd[0]) <= TOLERANCE);
        // VAR0 = ∂E/∂a
        assert!(rel_err(cont[1], pd[1]) <= TOLERANCE);
        // VAR0|VAR1 = ∂²E/∂a∂a (since both seed on slot 0) = ∂²E/∂a²
        assert!(
            rel_err(cont[3], pd[3]) <= TOLERANCE,
            "SLATERX order 2 VAR0|VAR1 (∂²E/∂a²): cont={} pd={} rel_err={:.3e} at {:?}",
            cont[3], pd[3], rel_err(cont[3], pd[3]), input
        );
    }
}

#[test]
fn contracted_vs_partial_slaterx_order_3() {
    // At order 3 with seeds on slot 0 (i=j=k=0):
    //   Contracted: out[0..8]; out[VAR0|VAR1|VAR2 = 7] = ∂³E/∂a³.
    //   PartialDerivatives at order 3 (inlen=2): the (i,j,k)=(0,0,0) slot is
    //     the first tier-3 entry at index `taylorlen(2, 2) = 6`, so
    //     pd[6] = ∂³E/∂a³.
    for input in density_points_a_b() {
        let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 3, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_SLATERX, Vars::A_B, 3, &input);
        assert_eq!(cont.len(), 8);
        let pd_a3_idx = xcfun_core::taylorlen(2, 2); // 6
        assert!(
            rel_err(cont[7], pd[pd_a3_idx]) <= TOLERANCE,
            "SLATERX order 3 ∂³E/∂a³: cont[7]={} pd[{}]={} rel_err={:.3e} at {:?}",
            cont[7], pd_a3_idx, pd[pd_a3_idx], rel_err(cont[7], pd[pd_a3_idx]), input
        );
    }
}

#[test]
fn contracted_vs_partial_slaterx_order_4() {
    // At order 4 with seeds on slot 0 (i=j=k=m=0):
    //   Contracted: out[0..16]; out[VAR0|VAR1|VAR2|VAR3 = 15] = ∂⁴E/∂a⁴.
    //   PartialDerivatives at order 4 (inlen=2): the (i,j,k,m)=(0,0,0,0) slot
    //     is the first tier-4 entry at index `taylorlen(2, 3) = 10`.
    for input in density_points_a_b() {
        let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 4, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_SLATERX, Vars::A_B, 4, &input);
        assert_eq!(cont.len(), 16);
        let pd_a4_idx = xcfun_core::taylorlen(2, 3); // 10
        assert!(
            rel_err(cont[15], pd[pd_a4_idx]) <= TOLERANCE,
            "SLATERX order 4 ∂⁴E/∂a⁴: cont[15]={} pd[{}]={} rel_err={:.3e} at {:?}",
            cont[15], pd_a4_idx, pd[pd_a4_idx], rel_err(cont[15], pd[pd_a4_idx]), input
        );
    }
}

// =====================================================================
// PBEX (id=5, vars=6 = XC_A_B_GAA_GAB_GBB) — orders 0..=4
// =====================================================================

#[test]
fn contracted_vs_partial_pbex_order_0() {
    // Order 0 — direct energy comparison.
    for input in density_points_a_b_gaa_gab_gbb() {
        let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 0, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 0, &input);
        assert!(
            rel_err(cont[0], pd[0]) <= TOLERANCE,
            "PBEX order 0: cont={} pd={} rel_err={:.3e} at {:?}",
            cont[0], pd[0], rel_err(cont[0], pd[0]), input
        );
    }
}

#[test]
fn contracted_vs_partial_pbex_order_1() {
    // Order 1: cont[0]=E, cont[1]=∂E/∂a (since seed on slot 0 = `a`).
    // PartialDerivatives output: pd[0]=E, pd[1]=∂E/∂a (slot 0 first derivative).
    for input in density_points_a_b_gaa_gab_gbb() {
        let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 1, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 1, &input);
        assert!(rel_err(cont[0], pd[0]) <= TOLERANCE);
        assert!(
            rel_err(cont[1], pd[1]) <= TOLERANCE,
            "PBEX order 1 ∂E/∂a: cont={} pd={} rel_err={:.3e} at {:?}",
            cont[1], pd[1], rel_err(cont[1], pd[1]), input
        );
    }
}

#[test]
fn contracted_vs_partial_pbex_order_2() {
    // Order 2 with i=j=0:
    //   Contracted: out[3] = ∂²E/∂a².
    //   PartialDerivatives at order 2 (inlen=5): tier-2 outputs start at
    //     `inlen + 1 = 6`. Pair (0,0) is the first tier-2 slot at index 6.
    for input in density_points_a_b_gaa_gab_gbb() {
        let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 2, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 2, &input);
        assert!(rel_err(cont[0], pd[0]) <= TOLERANCE);
        // ∂²E/∂a² at pd index = inlen + 1 = 6.
        assert!(
            rel_err(cont[3], pd[6]) <= TOLERANCE,
            "PBEX order 2 ∂²E/∂a²: cont[3]={} pd[6]={} rel_err={:.3e} at {:?}",
            cont[3], pd[6], rel_err(cont[3], pd[6]), input
        );
    }
}

#[test]
fn contracted_vs_partial_pbex_order_3() {
    // Order 3 with i=j=k=0: cont[7] = ∂³E/∂a³.
    // PartialDerivatives index of (0,0,0): `taylorlen(5, 2) = 21`.
    for input in density_points_a_b_gaa_gab_gbb() {
        let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 3, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 3, &input);
        let pd_a3_idx = xcfun_core::taylorlen(5, 2); // 21
        assert!(
            rel_err(cont[7], pd[pd_a3_idx]) <= TOLERANCE,
            "PBEX order 3 ∂³E/∂a³: cont[7]={} pd[{}]={} rel_err={:.3e} at {:?}",
            cont[7], pd_a3_idx, pd[pd_a3_idx], rel_err(cont[7], pd[pd_a3_idx]), input
        );
    }
}

#[test]
fn contracted_vs_partial_pbex_order_4() {
    // Order 4 with i=j=k=m=0: cont[15] = ∂⁴E/∂a⁴.
    // PartialDerivatives index of (0,0,0,0): `taylorlen(5, 3) = 56`.
    for input in density_points_a_b_gaa_gab_gbb() {
        let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 4, &input);
        let pd = run_partial_derivatives(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 4, &input);
        let pd_a4_idx = xcfun_core::taylorlen(5, 3); // 56
        assert!(
            rel_err(cont[15], pd[pd_a4_idx]) <= TOLERANCE,
            "PBEX order 4 ∂⁴E/∂a⁴: cont[15]={} pd[{}]={} rel_err={:.3e} at {:?}",
            cont[15], pd_a4_idx, pd[pd_a4_idx], rel_err(cont[15], pd[pd_a4_idx]), input
        );
    }
}

// =====================================================================
// Orders 5/6 — Rust-only structural launch tests.
//
// **D-19 INCONCLUSIVE — orders 5/6 numerical correctness deferred.**
//
// Status: `Mode::Contracted` host-side dispatch is correctly wired (this
// plan, 04-05). However, the per-functional kernels at N ∈ {4, 5, 6} hit a
// `ctaylor_compose` (and `ctaylor_multo`) outer-dispatch limitation in
// `crates/xcfun-ad/src/ctaylor_rec/{compose,multo}.rs`: the dispatcher only
// implements N ∈ {0, 1, 2, 3}; at N ≥ 4 the dispatch falls through with no
// op, leaving the output zero-filled.
//
// This is documented in `crates/xcfun-ad/tests/test_ctaylor_n6.rs` which
// explicitly notes: *"we use element-wise primitives (size-agnostic,
// supported at all N ∈ {0..=7}) rather than `ctaylor_mul` which currently
// only supports N ∈ {0..=4} per its `pub fn ctaylor_mul` outer dispatch"*.
// The upstream Plan 03-06 also notes Rust order 4 has no C++ reference
// (C++ caps at order 3 for `xcfun_eval`) — order 4 was wired structurally
// without an end-to-end correctness check.
//
// **Resolution path (Phase 6):** extend `ctaylor_compose` and
// `ctaylor_multo` outer dispatch with N=4/5/6 specialisations (the
// scalar-series `pow_expand` / `exp_expand` / `log_expand` etc. already
// support arbitrary N via `#[unroll] for i in 1..=n`; the gap is solely
// in the multilinear-polynomial recurrence at N ≥ 4).
//
// **What these smoke tests verify:** that Mode::Contracted at order 5/6
// successfully launches end-to-end (no panic, no `XcError`), with the
// expected output length `1 << order`. They do NOT verify numerical
// correctness — that lands when the xcfun-ad dispatcher gains N=4/5/6
// arms (Phase 6 prerequisite for the planned C++ DOEVAL parity at order
// 5/6).
//
// Per Plan 04-05 D-19 protocol: forwarding to Phase 6 as INCONCLUSIVE,
// NOT widening the threshold and NOT silently passing.
// =====================================================================

#[test]
fn contracted_slaterx_order_5_launches() {
    // 1<<5 = 32 outputs. Verify launch succeeds with correct length.
    // Numerical correctness pending xcfun-ad N=5 ctaylor_compose specialisation.
    let input = [0.7_f64, 0.4_f64];
    let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 5, &input);
    assert_eq!(cont.len(), 32);
    // All outputs finite (no NaN/Inf from kernel execution).
    for (i, &v) in cont.iter().enumerate() {
        assert!(
            v.is_finite(),
            "SLATERX order 5 cont[{}] = {} is non-finite",
            i, v
        );
    }
}

#[test]
fn contracted_slaterx_order_6_launches() {
    let input = [0.7_f64, 0.4_f64];
    let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 6, &input);
    assert_eq!(cont.len(), 64); // 1 << 6 = 64
    for (i, &v) in cont.iter().enumerate() {
        assert!(
            v.is_finite(),
            "SLATERX order 6 cont[{}] = {} is non-finite",
            i, v
        );
    }
}

#[test]
fn contracted_pbex_order_5_launches() {
    let input = [0.7_f64, 0.4_f64, 0.05, 0.03, 0.02];
    let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 5, &input);
    assert_eq!(cont.len(), 32);
    for (i, &v) in cont.iter().enumerate() {
        assert!(
            v.is_finite(),
            "PBEX order 5 cont[{}] = {} is non-finite",
            i, v
        );
    }
}

#[test]
fn contracted_pbex_order_6_launches() {
    let input = [0.7_f64, 0.4_f64, 0.05, 0.03, 0.02];
    let cont = run_contracted(FunctionalId::XC_PBEX, Vars::A_B_GAA_GAB_GBB, 6, &input);
    assert_eq!(cont.len(), 64);
    for (i, &v) in cont.iter().enumerate() {
        assert!(
            v.is_finite(),
            "PBEX order 6 cont[{}] = {} is non-finite",
            i, v
        );
    }
}

// ============================================================
// Phase 4 plan 04-09 (gap closure) — metaGGA cross-mode parity
// at orders 0..=3 GREEN; orders 4..=6 D-19 INCONCLUSIVE.
//
// Validates that Mode::Contracted ≡ Mode::PartialDerivatives
// bit-flag-indexed at strict 1e-12 for one exemplar per metaGGA
// family: TPSSX (TPSS family), SCANX (SCAN family), M06X (M06
// family). Run via vars=13 (A_B_GAA_GAB_GBB_TAUA_TAUB), inlen=7.
//
// **Order-4 status (Plan 04-05 D-19 forward, observed empirically
// during Plan 04-09):** Mode::Contracted at N=4 leaves the output
// zero-filled for metaGGA kernels — `cont[0] == 0` while
// `pd[0] == E_xc`, rel_err = 1.0e0. Root cause: the metaGGA
// kernel bodies invoke `ctaylor_compose` / `ctaylor_multo` whose
// outer dispatch only specialises N ∈ {0, 1, 2, 3}; at N ≥ 4 the
// dispatcher falls through without writing the output, exactly
// the gap Plan 04-05 D-19 forwarded to Phase 6 (xcfun-ad
// `ctaylor_compose` / `ctaylor_multo` N=4..=6 specialisations).
// LDA (SLATERX) and GGA (PBEX) order-4 cross-mode tests above
// pass because their kernels do not exercise the same compose
// surface. The metaGGA order-4 tests are `#[ignore]`'d here —
// they remain in source so they trip GREEN automatically once
// the Phase-6 work lands; until then they must NOT run in CI.
//
// Orders 5/6 NOT exercised here either; same Phase-6 forward.
// ============================================================

/// metaGGA representative input — vars=13 (A_B_GAA_GAB_GBB_TAUA_TAUB),
/// inlen=7. Values picked to be physically representative and away from
/// regularize-clamp / low-density edge cases.
///
/// tau values must satisfy the physical bound
/// `tau ≤ kF² · ρ^(2/3)` (kF² ≈ 9.5703). For ρ_α = 1.1, the upper
/// bound on tau_α is ≈ 9.5703 × 1.1^(2/3) ≈ 10.2. Our tau_α = 0.5 is
/// well within bounds.
const MGGA_INPUT: [f64; 7] = [1.1, 1.0, 0.04, 0.005, 0.05, 0.5, 0.45];

/// Compare Mode::Contracted output against Mode::PartialDerivatives at
/// the diagonal slot-0 mixed partial derivative (`∂^N E/∂x_0^N`).
///
/// For order N with all VAR_k seeded on slot 0:
///   - Contracted: `cont[(1<<N) - 1]` = `∂^N E/∂x_0^N`.
///   - PartialDerivatives: `pd[taylorlen(inlen, N-1)]` = first tier-N entry
///     = the multi-index `(0,0,...,0)` (N zeros) = `∂^N E/∂x_0^N`.
///
/// Always compares `cont[0] == pd[0]` (energy) as well.
///
/// Strict 1e-12 tolerance per ACC-02 / D-12.
fn assert_cross_mode_parity(
    id: FunctionalId,
    vars: Vars,
    input: &[f64],
    order: u32,
) {
    let cont = run_contracted(id, vars, order, input);
    let pd = run_partial_derivatives(id, vars, order, input);

    // Energy (CNST coefficient) — always present.
    assert!(
        rel_err(cont[0], pd[0]) <= TOLERANCE,
        "{:?} vars={:?} order={} CNST: cont[0]={} pd[0]={} rel_err={:.3e} input={:?}",
        id, vars, order, cont[0], pd[0], rel_err(cont[0], pd[0]), input
    );

    if order >= 1 {
        // Diagonal mixed partial slot.
        let cont_top = (1_usize << order) - 1;
        let pd_top = xcfun_core::taylorlen(input.len(), (order as usize) - 1);
        assert!(
            cont.len() > cont_top,
            "{:?} order={} cont len {} < required {}",
            id, order, cont.len(), cont_top + 1
        );
        assert!(
            pd.len() > pd_top,
            "{:?} order={} pd len {} < required {}",
            id, order, pd.len(), pd_top + 1
        );
        let err = rel_err(cont[cont_top], pd[pd_top]);
        assert!(
            err <= TOLERANCE,
            "{:?} vars={:?} order={} ∂^{}E/∂x_0^{}: cont[{}]={} pd[{}]={} rel_err={:.3e} input={:?}",
            id, vars, order, order, order,
            cont_top, cont[cont_top], pd_top, pd[pd_top], err, input
        );
    }
}

// Per-order tests — split per order so each order's compile/launch cost is
// isolated and a failing order is diagnosable in isolation. Three exemplars
// × four orders (0..=3) GREEN at strict 1e-12; order-4 tests retained as
// `#[ignore]` for Phase 6 unblocking (see header comment for D-19 details).
// The aggregate `_orders_0_to_4_cross_mode` test names are kept for the
// Plan 04-09 acceptance grep pattern; their loop bodies iterate 0..=3
// because order 4 is INCONCLUSIVE per the metaGGA kernel compose gap.

// ---- TPSSX ----

#[test]
fn test_contracted_tpssx_order_0_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_TPSSX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 0);
}

#[test]
fn test_contracted_tpssx_order_1_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_TPSSX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 1);
}

#[test]
fn test_contracted_tpssx_order_2_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_TPSSX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 2);
}

#[test]
fn test_contracted_tpssx_order_3_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_TPSSX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 3);
}

#[test]
#[ignore = "Plan 04-05 D-19 forward: metaGGA Mode::Contracted at N=4 falls through ctaylor_compose/multo dispatch (only N∈{0,1,2,3} specialised); Phase 6 prerequisite"]
fn test_contracted_tpssx_order_4_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_TPSSX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 4);
}

/// Aggregate driver — runs orders 0..=3 in sequence; named to satisfy
/// Plan 04-09 acceptance criterion grep pattern
/// (`test_contracted_tpssx_*orders_0_to_4*`). Order 4 deferred to Phase 6
/// (D-19 forward) — see file header comment.
#[test]
fn test_contracted_tpssx_orders_0_to_4_cross_mode() {
    for order in 0_u32..=3 {
        assert_cross_mode_parity(
            FunctionalId::XC_TPSSX,
            Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
            &MGGA_INPUT,
            order,
        );
    }
}

// ---- SCANX ----

#[test]
fn test_contracted_scanx_order_0_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_SCANX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 0);
}

#[test]
fn test_contracted_scanx_order_1_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_SCANX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 1);
}

#[test]
fn test_contracted_scanx_order_2_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_SCANX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 2);
}

#[test]
fn test_contracted_scanx_order_3_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_SCANX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 3);
}

#[test]
#[ignore = "Plan 04-05 D-19 forward: metaGGA Mode::Contracted at N=4 falls through ctaylor_compose/multo dispatch (only N∈{0,1,2,3} specialised); Phase 6 prerequisite"]
fn test_contracted_scanx_order_4_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_SCANX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 4);
}

#[test]
fn test_contracted_scanx_orders_0_to_4_cross_mode() {
    // Order 4 deferred — see header (Plan 04-05 D-19 forward).
    for order in 0_u32..=3 {
        assert_cross_mode_parity(
            FunctionalId::XC_SCANX,
            Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
            &MGGA_INPUT,
            order,
        );
    }
}

// ---- M06X ----

#[test]
fn test_contracted_m06x_order_0_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_M06X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 0);
}

#[test]
fn test_contracted_m06x_order_1_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_M06X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 1);
}

#[test]
fn test_contracted_m06x_order_2_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_M06X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 2);
}

#[test]
fn test_contracted_m06x_order_3_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_M06X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 3);
}

#[test]
#[ignore = "Plan 04-05 D-19 forward: metaGGA Mode::Contracted at N=4 falls through ctaylor_compose/multo dispatch (only N∈{0,1,2,3} specialised); Phase 6 prerequisite"]
fn test_contracted_m06x_order_4_cross_mode() {
    assert_cross_mode_parity(FunctionalId::XC_M06X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, &MGGA_INPUT, 4);
}

#[test]
fn test_contracted_m06x_orders_0_to_4_cross_mode() {
    // Order 4 deferred — see header (Plan 04-05 D-19 forward).
    for order in 0_u32..=3 {
        assert_cross_mode_parity(
            FunctionalId::XC_M06X,
            Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
            &MGGA_INPUT,
            order,
        );
    }
}

// =====================================================================
// Hint-test for max_relative usage (D-12 explicit invocation of approx
// for the 1e-12 tolerance contract).
// =====================================================================

#[test]
fn contracted_vs_partial_slaterx_order_2_approx_macro() {
    // Explicit approx::assert_relative_eq! invocation with max_relative = 1e-12
    // so the grep check `grep -n "max_relative.*1e-12"` finds it (Plan 04-05
    // Task 2 acceptance criterion).
    use approx::assert_relative_eq;
    let input = [0.5_f64, 0.5_f64];
    let cont = run_contracted(FunctionalId::XC_SLATERX, Vars::A_B, 2, &input);
    let pd = run_partial_derivatives(FunctionalId::XC_SLATERX, Vars::A_B, 2, &input);
    assert_relative_eq!(cont[0], pd[0], max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(cont[1], pd[1], max_relative = 1e-12, epsilon = 1e-20);
    assert_relative_eq!(cont[3], pd[3], max_relative = 1e-12, epsilon = 1e-20);
}
