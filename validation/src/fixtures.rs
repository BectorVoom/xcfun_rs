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
}
