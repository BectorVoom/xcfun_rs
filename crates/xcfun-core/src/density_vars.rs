//! DensityVars<T> -- density variable container.
//! Full implementation added in Task 2.

use xcfun_ad::Num;

/// All density-derived quantities at a single grid point.
pub struct DensityVars<T: Num> {
    // Primary spin densities
    pub a: T,
    pub b: T,

    // Derived total/spin densities
    pub n: T,
    pub s: T,

    // Gradient squared magnitudes
    pub gaa: T,
    pub gab: T,
    pub gbb: T,
    pub gnn: T,
    pub gns: T,
    pub gss: T,

    // Kinetic energy densities
    pub taua: T,
    pub taub: T,
    pub tau: T,

    // Laplacian
    pub lapa: T,
    pub lapb: T,

    // Current density
    pub jpaa: T,
    pub jpbb: T,

    // Pre-computed derived quantities
    pub zeta: T,
    pub r_s: T,
    pub n_m13: T,
    pub a_43: T,
    pub b_43: T,
}
