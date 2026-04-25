//! 10 000-point stratified grid generator for tier-2 parity.
//!
//! Per Plan 02-06 CONTEXT D-15 + D-18 + RESEARCH §"Grid Generator Spec":
//! 70% bulk + 30% stress (regularize / polarised / gradient).
//!
//! **Determinism**: seeded `Xoshiro256PlusPlus::seed_from_u64(0x1234abcd)`.
//! The grid is regenerated on every harness invocation (no committed fixtures
//! — Phase 1 Plan 01-05 established `rand_xoshiro` 0.8 as deterministic
//! across platforms). Two calls produce bit-identical output.

use rand_xoshiro::Xoshiro256PlusPlus;
use rand_xoshiro::rand_core::{Rng, SeedableRng};

/// Fixed seed for the tier-2 harness grid. Must not change without a
/// corresponding bump of the registered fixture record count.
pub const GRID_SEED: u64 = 0x1234abcd;
pub const TOTAL_POINTS: usize = 10_000;
pub const N_BULK: usize = 7000;
pub const N_REGULARIZE: usize = 1000;
pub const N_POLARISED: usize = 1000;
pub const N_GRADIENT: usize = 1000;

/// Upper bound of the regularize-clamp stratum: `2 × TINY_DENSITY`.
///
/// D-22 (PHASE2-CONTEXT) defines `TINY_DENSITY = 1e-14` as the per-spin
/// density floor applied by `xcfun-eval::density_vars::regularize`. Grid
/// points with `min(a, b) ≤ 2 × TINY_DENSITY` land in the clamp regime
/// where the regularize function **deliberately saturates** density inputs
/// to 1e-14 — this is a precision sacrifice chosen by design (ensures
/// `pow(rho, 1/3)`, `log(rho)`, etc. produce finite outputs on numerically
/// trivial density).
///
/// **Testing AT the clamp is testing the clamp's own precision sacrifice,
/// not kernel correctness.** Tier-2 parity at these points is a
/// test of `regularize`'s design decision, not of the functional port.
/// Plan 02-06 Fix 2 excludes such points from the tier-2 verdict
/// (parallel to the existing `excluded_by_upstream_spec` marker for
/// TW/VWK which lack upstream `test_in` data).
///
/// The `2×` factor: the floor is 1e-14 per spin, but `min(a,b)` in
/// practice varies with the regularize stratum's log-uniform spread; 2e-14
/// catches points where EITHER spin component is clamped or within a
/// tight neighborhood of the clamp boundary.
pub const REGULARIZE_CLAMP_STRATUM_BOUND: f64 = 2e-14_f64;

/// One grid point — superset of all input slots used across LDA + GGA + MGGA tiers.
///
/// Phase 2 LDAs consume `n + s` (or `a + b` derived via `ab_from_ns`); the
/// kinetic-GGAs TW + VWK also use `gaa + gbb + gab`. Phase 3 GGAs reuse
/// the same struct without regenerating.
#[derive(Clone, Copy, Debug, Default)]
pub struct GridPoint {
    pub n: f64,
    pub s: f64,
    pub gnn: f64,
    pub gns: f64,
    pub gss: f64,
    pub gaa: f64,
    pub gab: f64,
    pub gbb: f64,
}

impl GridPoint {
    /// Convert `(n, s)` to `(a, b)`: `a = (n + s) / 2`, `b = (n - s) / 2`.
    /// Per `xcfun-master/src/densvars.hpp` XC_N_S → XC_A_B case arm.
    pub fn ab_from_ns(&self) -> (f64, f64) {
        ((self.n + self.s) * 0.5, (self.n - self.s) * 0.5)
    }
}

/// Generate the 10 000-point seeded grid. Deterministic across runs.
pub fn generate_grid() -> Vec<GridPoint> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(GRID_SEED);
    let mut out = Vec::with_capacity(TOTAL_POINTS);
    out.extend(generate_bulk(&mut rng, N_BULK));
    out.extend(generate_regularize_stress(&mut rng, N_REGULARIZE));
    out.extend(generate_polarised(&mut rng, N_POLARISED));
    out.extend(generate_gradient_stress(&mut rng, N_GRADIENT));
    out
}

/// Convert a u64 from Xoshiro256++ to a uniform `[0, 1)` f64 via the upper
/// 53 mantissa bits (standard IEEE-754 pattern).
fn next_uniform_01(rng: &mut Xoshiro256PlusPlus) -> f64 {
    let bits = rng.next_u64() >> 11;
    bits as f64 / (1_u64 << 53) as f64
}

/// Stratum 1 (7000 points): bulk density.
/// - `n` log-uniform on `[1e-5, 10.0]`
/// - `|s/n|` uniform on `[0, 0.95]` with random sign
/// - Gradient fields zero (LDA scope; strata 4 exercises gradients for TW/VWK).
fn generate_bulk(rng: &mut Xoshiro256PlusPlus, count: usize) -> Vec<GridPoint> {
    (0..count)
        .map(|_| {
            let u = next_uniform_01(rng);
            let n = 1e-5 * 10.0_f64.powf(6.0 * u); // log-uniform [1e-5, 1e1]
            let z_abs = 0.95 * next_uniform_01(rng);
            let z_sign = if next_uniform_01(rng) < 0.5 { -1.0 } else { 1.0 };
            let s = z_sign * z_abs * n;
            GridPoint {
                n,
                s,
                gnn: 0.0,
                gns: 0.0,
                gss: 0.0,
                gaa: 0.0,
                gab: 0.0,
                gbb: 0.0,
            }
        })
        .collect()
}

/// Stratum 2 (1000 points): regularize-stress.
/// `n` log-uniform on `[1e-14, 1e-5]`, `s = 0` (TINY_DENSITY floor tests).
fn generate_regularize_stress(rng: &mut Xoshiro256PlusPlus, count: usize) -> Vec<GridPoint> {
    (0..count)
        .map(|_| {
            let u = next_uniform_01(rng);
            let n = 1e-14 * 10.0_f64.powf(9.0 * u);
            GridPoint {
                n,
                s: 0.0,
                gnn: 0.0,
                gns: 0.0,
                gss: 0.0,
                gaa: 0.0,
                gab: 0.0,
                gbb: 0.0,
            }
        })
        .collect()
}

/// Stratum 3 (1000 points): polarised limit.
/// `n` log-uniform on `[1e-3, 10.0]`; `|zeta| = |s/n|` uniform on `[0.95, 1.0]`.
/// Tests cancellation forms like `(1 - zeta^4)` in `vwn5_eps`.
fn generate_polarised(rng: &mut Xoshiro256PlusPlus, count: usize) -> Vec<GridPoint> {
    (0..count)
        .map(|_| {
            let u = next_uniform_01(rng);
            let n = 1e-3 * 10.0_f64.powf(4.0 * u);
            let z_abs = 0.95 + 0.05 * next_uniform_01(rng);
            let z_sign = if next_uniform_01(rng) < 0.5 { -1.0 } else { 1.0 };
            let s = z_sign * z_abs * n;
            GridPoint {
                n,
                s,
                gnn: 0.0,
                gns: 0.0,
                gss: 0.0,
                gaa: 0.0,
                gab: 0.0,
                gbb: 0.0,
            }
        })
        .collect()
}

/// Phase 3 plan 03-06 — supplemental 400-point GGA-stratified grid.
///
/// Per PATTERNS.md J2: 4 strata × 100 points = 400 points, fixed seed `0xdeadbeef`
/// for cross-machine determinism. Strata exercise GGA-specific failure modes:
///
///   1. enhancement_sweep — Pade enhancement F(s²) regime (s ∈ [0, 5]).
///   2. low_density_high_gradient — small ρ + large |∇ρ| (clamp + grad stress).
///   3. high_polarisation — |zeta| ∈ [0.9, 0.999] cancellation regime.
///   4. rs_sweep — wide r_s coverage [1e-2, 1e6] for correlation kernels.
///
/// The supplemental grid is intended to be APPENDED to the standard 10k grid
/// via the validation CLI's `--grid supplemental` flag (Plan 03-06 Task 2).
pub const SUPPLEMENT_SEED: u64 = 0xdeadbeef;
pub const SUPPLEMENT_TOTAL: usize = 400;

/// 400-point supplemental GGA-stratified grid generator (PATTERNS.md J2).
pub fn gga_stratified_supplement() -> Vec<GridPoint> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(SUPPLEMENT_SEED);
    let mut out = Vec::with_capacity(SUPPLEMENT_TOTAL);
    out.extend(enhancement_sweep(&mut rng, 100));
    out.extend(low_density_high_gradient(&mut rng, 100));
    out.extend(high_polarisation(&mut rng, 100));
    out.extend(rs_sweep(&mut rng, 100));
    out
}

/// Stratum S1 — 100-point enhancement-factor sweep.
///
/// Sample `n_total ∈ [0.1, 10.0]` (log-uniform); `zeta ∈ [-0.95, 0.95]`.
/// Synthesise gradients such that `s = |∇ρ| / (2·(3π²)^(1/3)·ρ^(4/3))` lies
/// in `[0, 5]` — exercises the F(s²) Padé / exp branches of PBE/RPBE/PBESOL.
fn enhancement_sweep(rng: &mut Xoshiro256PlusPlus, n: usize) -> Vec<GridPoint> {
    // C_S = 2 · (3π²)^(1/3) ≈ 6.18733... — the s-normalisation prefactor.
    let c_s = 2.0_f64 * (3.0_f64 * std::f64::consts::PI.powi(2)).powf(1.0 / 3.0);
    (0..n)
        .map(|_| {
            let u = next_uniform_01(rng);
            let n_total = 0.1 * 10.0_f64.powf(2.0 * u); // log-uniform [0.1, 10]
            let z = (next_uniform_01(rng) * 2.0 - 1.0) * 0.95;
            let s = z * n_total;
            let s_target = 5.0 * next_uniform_01(rng); // s ∈ [0, 5]
            // |∇ρ|² so that s_target = |∇ρ| / (c_s · ρ^(4/3))
            let grad_mag = s_target * c_s * n_total.powf(4.0 / 3.0);
            let grad_sq = grad_mag * grad_mag;
            // Distribute between α and β proportional to |a|/|b|.
            let a = (n_total + s).max(1e-30) * 0.5;
            let b = (n_total - s).max(1e-30) * 0.5;
            let split = a / (a + b);
            let gaa = grad_sq * split * split;
            let gbb = grad_sq * (1.0 - split) * (1.0 - split);
            let gab = (gaa * gbb).sqrt() * (next_uniform_01(rng) * 2.0 - 1.0);
            let gnn = gaa + 2.0 * gab + gbb;
            let gss = gaa - 2.0 * gab + gbb;
            let gns = gaa - gbb;
            GridPoint {
                n: n_total,
                s,
                gnn,
                gns,
                gss,
                gaa,
                gab,
                gbb,
            }
        })
        .collect()
}

/// Stratum S2 — 100-point low-density / high-gradient regime.
///
/// `α/β ∈ [1e-10, 1e-4]` (log-uniform); `|∇ρ|²` log-uniform on `[1, 1e6]`.
/// Exercises the regularize-clamp boundary + gradient-large term cancellation
/// in correlation kernels (PBEC, LYPC, PW91C).
fn low_density_high_gradient(rng: &mut Xoshiro256PlusPlus, n: usize) -> Vec<GridPoint> {
    (0..n)
        .map(|_| {
            let u_a = next_uniform_01(rng);
            let u_b = next_uniform_01(rng);
            let a = 1e-10 * 10.0_f64.powf(6.0 * u_a); // [1e-10, 1e-4]
            let b = 1e-10 * 10.0_f64.powf(6.0 * u_b);
            let n_total = a + b;
            let s = a - b;
            let v = next_uniform_01(rng);
            let grad_sq = 10.0_f64.powf(6.0 * v); // [1, 1e6]
            let theta = next_uniform_01(rng) * std::f64::consts::PI;
            let gaa = grad_sq * theta.cos().powi(2);
            let gbb = grad_sq * theta.sin().powi(2);
            let gab = (gaa * gbb).sqrt() * (next_uniform_01(rng) * 2.0 - 1.0);
            let gnn = gaa + 2.0 * gab + gbb;
            let gss = gaa - 2.0 * gab + gbb;
            let gns = gaa - gbb;
            GridPoint {
                n: n_total,
                s,
                gnn,
                gns,
                gss,
                gaa,
                gab,
                gbb,
            }
        })
        .collect()
}

/// Stratum S3 — 100-point high-polarisation regime (|zeta| ∈ [0.9, 0.999]).
///
/// Total density `[0.1, 10]`; `|∇ρ|² ∈ [0.01, 1e4]`. Exercises the
/// `(1 - zeta^4)` + `pw92eps_polarized` FERRO-branch cancellation in B97
/// correlation kernels (the Wave-4 D-19 forward stratum).
fn high_polarisation(rng: &mut Xoshiro256PlusPlus, n: usize) -> Vec<GridPoint> {
    (0..n)
        .map(|_| {
            let u = next_uniform_01(rng);
            let n_total = 0.1 * 10.0_f64.powf(2.0 * u);
            let z_abs = 0.9 + 0.099 * next_uniform_01(rng); // [0.9, 0.999]
            let z_sign = if next_uniform_01(rng) < 0.5 { -1.0 } else { 1.0 };
            let s = z_sign * z_abs * n_total;
            let v = next_uniform_01(rng);
            let grad_sq = 0.01 * 10.0_f64.powf(6.0 * v); // [0.01, 1e4]
            let theta = next_uniform_01(rng) * std::f64::consts::PI;
            let gaa = grad_sq * theta.cos().powi(2);
            let gbb = grad_sq * theta.sin().powi(2);
            let gab = (gaa * gbb).sqrt() * (next_uniform_01(rng) * 2.0 - 1.0);
            let gnn = gaa + 2.0 * gab + gbb;
            let gss = gaa - 2.0 * gab + gbb;
            let gns = gaa - gbb;
            GridPoint {
                n: n_total,
                s,
                gnn,
                gns,
                gss,
                gaa,
                gab,
                gbb,
            }
        })
        .collect()
}

/// Stratum S4 — 100-point r_s sweep (wide Wigner-Seitz radius range).
///
/// `r_s ∈ [1e-2, 1e6]` (log-uniform), `zeta ∈ [-0.95, 0.95]`,
/// `|∇ρ|² ∈ [0, 100]`. Exercises the LSDA baseline (`pw92eps`, `vwn5_eps`,
/// `pz81_eps`) consumed by every GGA correlation kernel.
///
/// `n_total = 3 / (4π · r_s³)` from r_s definition.
fn rs_sweep(rng: &mut Xoshiro256PlusPlus, n: usize) -> Vec<GridPoint> {
    (0..n)
        .map(|_| {
            let u = next_uniform_01(rng);
            let r_s = 1e-2 * 10.0_f64.powf(8.0 * u); // [1e-2, 1e6]
            let n_total =
                3.0 / (4.0 * std::f64::consts::PI * r_s.powi(3));
            let z = (next_uniform_01(rng) * 2.0 - 1.0) * 0.95;
            let s = z * n_total;
            let v = next_uniform_01(rng);
            let grad_sq = 100.0 * v; // [0, 100], uniform
            let theta = next_uniform_01(rng) * std::f64::consts::PI;
            let gaa = grad_sq * theta.cos().powi(2);
            let gbb = grad_sq * theta.sin().powi(2);
            let gab = (gaa * gbb).sqrt() * (next_uniform_01(rng) * 2.0 - 1.0);
            let gnn = gaa + 2.0 * gab + gbb;
            let gss = gaa - 2.0 * gab + gbb;
            let gns = gaa - gbb;
            GridPoint {
                n: n_total,
                s,
                gnn,
                gns,
                gss,
                gaa,
                gab,
                gbb,
            }
        })
        .collect()
}

/// Stratum 4 (1000 points): gradient-stress.
/// `n` log-uniform on `[1e-3, 10.0]`; `|∇ρ|²` log-uniform on `[1, 1e6]`.
/// Phase 2 LDAs ignore gradient fields; TW + VWK consume `(gaa, gab, gbb)`.
fn generate_gradient_stress(rng: &mut Xoshiro256PlusPlus, count: usize) -> Vec<GridPoint> {
    (0..count)
        .map(|_| {
            let u = next_uniform_01(rng);
            let n = 1e-3 * 10.0_f64.powf(4.0 * u);
            let v = next_uniform_01(rng);
            let grad_sq = 10.0_f64.powf(6.0 * v); // log-uniform [1, 1e6]
            let theta = next_uniform_01(rng) * std::f64::consts::PI;
            let gaa = grad_sq * theta.cos().powi(2);
            let gbb = grad_sq * theta.sin().powi(2);
            let gab =
                (gaa * gbb).sqrt() * (next_uniform_01(rng) * 2.0 - 1.0); // |gab| ≤ sqrt(gaa*gbb)
            let z = (next_uniform_01(rng) * 2.0 - 1.0) * 0.5;
            let s = z * n;
            let gnn = gaa + 2.0 * gab + gbb;
            let gss = gaa - 2.0 * gab + gbb;
            let gns = gaa - gbb;
            GridPoint {
                n,
                s,
                gnn,
                gns,
                gss,
                gaa,
                gab,
                gbb,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_size_is_10000() {
        let grid = generate_grid();
        assert_eq!(grid.len(), TOTAL_POINTS);
    }

    #[test]
    fn grid_is_deterministic() {
        let g1 = generate_grid();
        let g2 = generate_grid();
        assert_eq!(g1.len(), g2.len());
        for (a, b) in g1.iter().zip(g2.iter()) {
            assert_eq!(a.n.to_bits(), b.n.to_bits(), "n bit-identity");
            assert_eq!(a.s.to_bits(), b.s.to_bits(), "s bit-identity");
            assert_eq!(a.gaa.to_bits(), b.gaa.to_bits(), "gaa bit-identity");
            assert_eq!(a.gbb.to_bits(), b.gbb.to_bits(), "gbb bit-identity");
        }
    }

    #[test]
    fn bulk_stratum_has_correct_n_range() {
        let grid = generate_grid();
        for gp in grid.iter().take(N_BULK) {
            assert!(
                gp.n >= 1e-5 && gp.n <= 10.0,
                "bulk n={} out of range",
                gp.n
            );
            assert!(
                gp.s.abs() <= 0.95 * gp.n + 1e-15,
                "bulk |s|={} exceeds 0.95*n={}",
                gp.s.abs(),
                gp.n
            );
        }
    }

    #[test]
    fn regularize_stratum_uses_low_density() {
        let grid = generate_grid();
        for gp in grid.iter().skip(N_BULK).take(N_REGULARIZE) {
            assert!(
                gp.n >= 1e-14 && gp.n <= 1e-5,
                "regularize n={} out of range",
                gp.n
            );
            assert_eq!(gp.s, 0.0, "regularize stratum: s should be 0");
        }
    }

    #[test]
    fn polarised_stratum_has_high_zeta() {
        let grid = generate_grid();
        for gp in grid
            .iter()
            .skip(N_BULK + N_REGULARIZE)
            .take(N_POLARISED)
        {
            let zeta_abs = (gp.s / gp.n).abs();
            assert!(
                zeta_abs >= 0.95 - 1e-12 && zeta_abs <= 1.0 + 1e-12,
                "polarised |zeta|={} out of range",
                zeta_abs
            );
        }
    }

    #[test]
    fn ab_from_ns_round_trips() {
        let gp = GridPoint {
            n: 2.0,
            s: 0.5,
            ..GridPoint::default()
        };
        let (a, b) = gp.ab_from_ns();
        assert!((a - 1.25).abs() < 1e-15);
        assert!((b - 0.75).abs() < 1e-15);
        assert!((a + b - gp.n).abs() < 1e-15);
        assert!((a - b - gp.s).abs() < 1e-15);
    }

    #[test]
    fn supplement_size_is_400() {
        let g = gga_stratified_supplement();
        assert_eq!(g.len(), SUPPLEMENT_TOTAL);
    }

    #[test]
    fn supplement_is_deterministic() {
        let g1 = gga_stratified_supplement();
        let g2 = gga_stratified_supplement();
        assert_eq!(g1.len(), g2.len());
        for (a, b) in g1.iter().zip(g2.iter()) {
            assert_eq!(a.n.to_bits(), b.n.to_bits(), "supplement n bit-identity");
            assert_eq!(a.s.to_bits(), b.s.to_bits(), "supplement s bit-identity");
            assert_eq!(a.gaa.to_bits(), b.gaa.to_bits(), "supplement gaa bit-identity");
            assert_eq!(a.gbb.to_bits(), b.gbb.to_bits(), "supplement gbb bit-identity");
        }
    }

    #[test]
    fn supplement_strata_split_evenly() {
        // 4 × 100 strata.
        let g = gga_stratified_supplement();
        // S1 enhancement_sweep — n_total ∈ [0.1, 10].
        for gp in g.iter().take(100) {
            assert!(
                gp.n >= 0.1 - 1e-12 && gp.n <= 10.0 + 1e-12,
                "S1 n={} out of range",
                gp.n
            );
        }
        // S3 high_polarisation — |zeta| ∈ [0.9, 0.999].
        for gp in g.iter().skip(200).take(100) {
            let zeta_abs = (gp.s / gp.n).abs();
            assert!(
                zeta_abs >= 0.9 - 1e-12 && zeta_abs <= 0.999 + 1e-12,
                "S3 |zeta|={} out of range",
                zeta_abs
            );
        }
    }
}
