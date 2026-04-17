//! DensityVars<T> -- density variable container.
//!
//! Holds all density-derived quantities that functionals may consume.
//! The type parameter `T` is either `f64` (energy-only evaluation)
//! or `CTaylor<f64, N>` (automatic differentiation at order N).

use crate::constants;
use crate::enums::VarType;
use crate::error::XcError;
use xcfun_ad::Num;

/// Regularize a density value: clamp to TINY_DENSITY if below threshold.
///
/// For CTaylor, this modifies only c[0] (the constant term), preserving
/// all derivative coefficients -- matching C++ behavior in densvars.hpp.
fn regularize<T: Num>(x: &mut T) {
    if x.value_f64() < constants::TINY_DENSITY {
        x.set_constant(constants::TINY_DENSITY);
    }
}

/// All density-derived quantities at a single grid point.
///
/// Constructed from raw input arrays via `DensityVars::from_input()`.
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

impl<T: Num> DensityVars<T> {
    /// Convert a raw input array into canonical density variables.
    ///
    /// The interpretation of `input` depends on `var_type`.
    /// All derived quantities are computed automatically.
    /// Densities below TINY_DENSITY (1e-14) are regularized.
    ///
    /// # Errors
    /// Returns `XcError::InputLengthMismatch` if input is too short.
    pub fn from_input(input: &[T], var_type: VarType) -> Result<Self, XcError> {
        let expected = var_type.input_len();
        if input.len() < expected {
            return Err(XcError::InputLengthMismatch {
                expected,
                got: input.len(),
            });
        }

        let zero = T::zero();

        // Initialize all fields to zero
        let mut a = zero.clone();
        let mut b = zero.clone();
        let mut n = zero.clone();
        let mut s = zero.clone();
        let mut gaa = zero.clone();
        let mut gab = zero.clone();
        let mut gbb = zero.clone();
        let mut gnn = zero.clone();
        let mut gns = zero.clone();
        let mut gss = zero.clone();
        let mut taua = zero.clone();
        let mut taub = zero.clone();
        let mut tau = zero.clone();
        let mut lapa = zero.clone();
        let mut lapb = zero.clone();
        let mut jpaa = zero.clone();
        let mut jpbb = zero.clone();

        let two = T::from_f64(2.0);
        let half = T::from_f64(0.5);
        let quarter = T::from_f64(0.25);

        // Match the C++ switch-case logic from densvars.hpp.
        // C++ uses fallthrough; here we explicitly handle each case.
        match var_type {
            // === Alpha/beta density types (A-based) ===

            VarType::A => {
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_B => {
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::A_GAA => {
                // Fallthrough in C++: A_GAA -> sets gradient, then A -> sets density
                gaa = input[1].clone();
                gab = T::zero();
                gbb = T::zero();
                gnn = gaa.clone();
                gss = gaa.clone();
                gns = gaa.clone();
                // A part
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_B_GAA_GAB_GBB => {
                gaa = input[2].clone();
                gab = input[3].clone();
                gbb = input[4].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                // A_B part
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::A_B_GAA_GAB_GBB_TAUA_TAUB => {
                taua = input[5].clone();
                taub = input[6].clone();
                tau = taua.clone() + taub.clone();
                // A_B_GAA_GAB_GBB part
                gaa = input[2].clone();
                gab = input[3].clone();
                gbb = input[4].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                // A_B part
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::A_GAA_LAPA => {
                lapa = input[2].clone();
                lapb = T::zero();
                // A_GAA part
                gaa = input[1].clone();
                gab = T::zero();
                gbb = T::zero();
                gnn = gaa.clone();
                gss = gaa.clone();
                gns = gaa.clone();
                // A part
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_GAA_TAUA => {
                taua = input[2].clone();
                taub = T::zero();
                tau = taua.clone();
                // A_GAA part
                gaa = input[1].clone();
                gab = T::zero();
                gbb = T::zero();
                gnn = gaa.clone();
                gss = gaa.clone();
                gns = gaa.clone();
                // A part
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB => {
                lapa = input[5].clone();
                lapb = input[6].clone();
                // A_B_GAA_GAB_GBB part
                gaa = input[2].clone();
                gab = input[3].clone();
                gbb = input[4].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                // A_B part
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB => {
                // C++ densvars.hpp: lapa=d[5], lapb=d[6], taua=d[7], taub=d[8]
                lapa = input[5].clone();
                lapb = input[6].clone();
                taua = input[7].clone();
                taub = input[8].clone();
                tau = taua.clone() + taub.clone();
                // gradient
                gaa = input[2].clone();
                gab = input[3].clone();
                gbb = input[4].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                // density
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => {
                jpaa = input[9].clone();
                jpbb = input[10].clone();
                // Same as A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
                lapa = input[5].clone();
                lapb = input[6].clone();
                taua = input[7].clone();
                taub = input[8].clone();
                tau = taua.clone() + taub.clone();
                gaa = input[2].clone();
                gab = input[3].clone();
                gbb = input[4].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            // === Total/spin density types (N-based) ===

            VarType::N => {
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_S => {
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
            }

            VarType::N_GNN => {
                gnn = input[1].clone();
                gss = T::zero();
                gns = T::zero();
                gaa = quarter.clone() * gnn.clone();
                gab = gaa.clone();
                gbb = gaa.clone();
                // N part
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_S_GNN_GNS_GSS => {
                gnn = input[2].clone();
                gns = input[3].clone();
                gss = input[4].clone();
                gaa = quarter.clone()
                    * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone()
                    * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
                // N_S part
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
            }

            VarType::N_GNN_LAPN => {
                lapa = half.clone() * input[2].clone();
                lapb = lapa.clone();
                // N_GNN part
                gnn = input[1].clone();
                gss = T::zero();
                gns = T::zero();
                gaa = quarter.clone() * gnn.clone();
                gab = gaa.clone();
                gbb = gaa.clone();
                // N part
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_GNN_TAUN => {
                taua = input[2].clone() / two.clone();
                taub = taua.clone();
                tau = input[2].clone();
                // N_GNN part
                gnn = input[1].clone();
                gss = T::zero();
                gns = T::zero();
                gaa = quarter.clone() * gnn.clone();
                gab = gaa.clone();
                gbb = gaa.clone();
                // N part
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_S_GNN_GNS_GSS_LAPN_LAPS => {
                lapa = half.clone() * (input[5].clone() + input[6].clone());
                lapb = half.clone() * (input[5].clone() - input[6].clone());
                // N_S_GNN_GNS_GSS part
                gnn = input[2].clone();
                gns = input[3].clone();
                gss = input[4].clone();
                gaa = quarter.clone()
                    * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone()
                    * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
                // N_S part
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
            }

            VarType::N_S_GNN_GNS_GSS_TAUN_TAUS => {
                taua = half.clone() * (input[5].clone() + input[6].clone());
                taub = half.clone() * (input[5].clone() - input[6].clone());
                tau = taua.clone() + taub.clone();
                // N_S_GNN_GNS_GSS part
                gnn = input[2].clone();
                gns = input[3].clone();
                gss = input[4].clone();
                gaa = quarter.clone()
                    * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone()
                    * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
                // N_S part
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
            }

            VarType::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => {
                lapa = half.clone() * (input[5].clone() + input[6].clone());
                lapb = half.clone() * (input[5].clone() - input[6].clone());
                taua = half.clone() * (input[7].clone() + input[8].clone());
                taub = half.clone() * (input[7].clone() - input[8].clone());
                tau = taua.clone() + taub.clone();
                // N_S_GNN_GNS_GSS part
                gnn = input[2].clone();
                gns = input[3].clone();
                gss = input[4].clone();
                gaa = quarter.clone()
                    * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone()
                    * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
                // N_S part
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
            }

            // === Gradient component types ===

            VarType::A_AX_AY_AZ => {
                // [a, ax, ay, az]
                gaa = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gab = T::zero();
                gbb = T::zero();
                gnn = gaa.clone();
                gss = gaa.clone();
                gns = gaa.clone();
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_B_AX_AY_AZ_BX_BY_BZ => {
                // [a, b, ax, ay, az, bx, by, bz]
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                gaa = input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone()
                    + input[4].clone() * input[4].clone();
                gab = input[2].clone() * input[5].clone()
                    + input[3].clone() * input[6].clone()
                    + input[4].clone() * input[7].clone();
                gbb = input[5].clone() * input[5].clone()
                    + input[6].clone() * input[6].clone()
                    + input[7].clone() * input[7].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::N_NX_NY_NZ => {
                // [n, nx, ny, nz]
                gnn = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gss = T::zero();
                gns = T::zero();
                gaa = quarter.clone() * gnn.clone();
                gab = gaa.clone();
                gbb = gaa.clone();
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_S_NX_NY_NZ_SX_SY_SZ => {
                // [n, s, nx, ny, nz, sx, sy, sz]
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
                gnn = input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone()
                    + input[4].clone() * input[4].clone();
                gss = input[5].clone() * input[5].clone()
                    + input[6].clone() * input[6].clone()
                    + input[7].clone() * input[7].clone();
                gns = input[2].clone() * input[5].clone()
                    + input[3].clone() * input[6].clone()
                    + input[4].clone() * input[7].clone();
                gaa = quarter.clone() * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone() * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
            }

            // === Gradient component types with kinetic ===

            VarType::A_AX_AY_AZ_TAUA => {
                // [a, ax, ay, az, taua]
                taua = input[4].clone();
                taub = T::zero();
                tau = taua.clone();
                gaa = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gab = T::zero();
                gbb = T::zero();
                gnn = gaa.clone();
                gss = gaa.clone();
                gns = gaa.clone();
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB => {
                // [a, b, ax, ay, az, bx, by, bz, taua, taub]
                taua = input[8].clone();
                taub = input[9].clone();
                tau = taua.clone() + taub.clone();
                a = input[0].clone();
                regularize(&mut a);
                b = input[1].clone();
                regularize(&mut b);
                gaa = input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone()
                    + input[4].clone() * input[4].clone();
                gab = input[2].clone() * input[5].clone()
                    + input[3].clone() * input[6].clone()
                    + input[4].clone() * input[7].clone();
                gbb = input[5].clone() * input[5].clone()
                    + input[6].clone() * input[6].clone()
                    + input[7].clone() * input[7].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::N_NX_NY_NZ_TAUN => {
                // [n, nx, ny, nz, taun]
                taua = input[4].clone() / two.clone();
                taub = taua.clone();
                tau = input[4].clone();
                gnn = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gss = T::zero();
                gns = T::zero();
                gaa = quarter.clone() * gnn.clone();
                gab = gaa.clone();
                gbb = gaa.clone();
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => {
                // [n, s, nx, ny, nz, sx, sy, sz, taun, taus]
                taua = half.clone() * (input[8].clone() + input[9].clone());
                taub = half.clone() * (input[8].clone() - input[9].clone());
                tau = input[8].clone();
                n = input[0].clone();
                regularize(&mut n);
                s = input[1].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
                gnn = input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone()
                    + input[4].clone() * input[4].clone();
                gss = input[5].clone() * input[5].clone()
                    + input[6].clone() * input[6].clone()
                    + input[7].clone() * input[7].clone();
                gns = input[2].clone() * input[5].clone()
                    + input[3].clone() * input[6].clone()
                    + input[4].clone() * input[7].clone();
                gaa = quarter.clone() * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone() * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
            }

            // === 2nd order Taylor ===

            VarType::A_2ND_TAYLOR => {
                // C++ comment: "a gax gay gaz haxx haxy haxz hayy hayz hazz"
                // indices:       0  1   2   3   4    5    6    7    8    9
                lapa = input[4].clone() + input[7].clone() + input[9].clone();
                lapb = T::zero();
                gaa = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gab = T::zero();
                gbb = T::zero();
                gnn = gaa.clone();
                gss = gaa.clone();
                gns = gaa.clone();
                a = input[0].clone();
                regularize(&mut a);
                b = T::zero();
                n = a.clone();
                s = n.clone();
            }

            VarType::A_B_2ND_TAYLOR => {
                // C++: a gax gay gaz haxx haxy haxz hayy hayz hazz
                //      0 1   2   3   4    5    6    7    8    9
                //      b gbx gby gbz hbxx hbxy hbxz hbyy hbyz hbzz
                //      10 11  12  13  14   15   16   17   18   19
                lapa = input[4].clone() + input[7].clone() + input[9].clone();
                lapb = input[14].clone() + input[17].clone() + input[19].clone();
                a = input[0].clone();
                regularize(&mut a);
                b = input[10].clone();
                regularize(&mut b);
                gaa = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gab = input[1].clone() * input[11].clone()
                    + input[2].clone() * input[12].clone()
                    + input[3].clone() * input[13].clone();
                gbb = input[11].clone() * input[11].clone()
                    + input[12].clone() * input[12].clone()
                    + input[13].clone() * input[13].clone();
                gnn = gaa.clone() + two.clone() * gab.clone() + gbb.clone();
                gss = gaa.clone() - two.clone() * gab.clone() + gbb.clone();
                gns = gaa.clone() - gbb.clone();
                n = a.clone() + b.clone();
                s = a.clone() - b.clone();
            }

            VarType::N_2ND_TAYLOR => {
                // C++: N_2ND_TAYLOR falls through to N_NX_NY_NZ with lapa set
                // indices: n, nx, ny, nz, hxx, hxy, hxz, hyy, hyz, hzz
                //          0  1   2   3   4    5    6    7    8    9
                lapa = half.clone() * (input[4].clone() + input[7].clone() + input[9].clone());
                lapb = lapa.clone();
                gnn = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gss = T::zero();
                gns = T::zero();
                gaa = quarter.clone() * gnn.clone();
                gab = gaa.clone();
                gbb = gaa.clone();
                n = input[0].clone();
                regularize(&mut n);
                s = T::zero();
                a = half.clone() * n.clone();
                b = a.clone();
            }

            VarType::N_S_2ND_TAYLOR => {
                // Similar to N_2ND_TAYLOR but with spin
                // [n_coeffs(10), s_coeffs(10)]
                let n_lap = input[4].clone() + input[7].clone() + input[9].clone();
                let s_lap = input[14].clone() + input[17].clone() + input[19].clone();
                lapa = half.clone() * (n_lap.clone() + s_lap.clone());
                lapb = half.clone() * (n_lap - s_lap);
                // Gradient from n components
                gnn = input[1].clone() * input[1].clone()
                    + input[2].clone() * input[2].clone()
                    + input[3].clone() * input[3].clone();
                gss = input[11].clone() * input[11].clone()
                    + input[12].clone() * input[12].clone()
                    + input[13].clone() * input[13].clone();
                gns = input[1].clone() * input[11].clone()
                    + input[2].clone() * input[12].clone()
                    + input[3].clone() * input[13].clone();
                gaa = quarter.clone() * (gnn.clone() + two.clone() * gns.clone() + gss.clone());
                gab = quarter.clone() * (gnn.clone() - gss.clone());
                gbb = quarter.clone() * (gnn.clone() - two.clone() * gns.clone() + gss.clone());
                n = input[0].clone();
                regularize(&mut n);
                s = input[10].clone();
                a = half.clone() * (n.clone() + s.clone());
                regularize(&mut a);
                b = half.clone() * (n.clone() - s.clone());
                regularize(&mut b);
            }
        }

        // Compute derived quantities (after switch, matching C++)
        let zeta = s.clone() / n.clone();
        // r_s = (3/(4*pi*n))^(1/3) = (3/(4*pi))^(1/3) * n^(-1/3)
        let n_m13 = n.clone().pow(-1.0 / 3.0);
        let r_s = T::from_f64(constants::RS_PREFACTOR) * n_m13.clone();
        let a_43 = a.clone().pow(4.0 / 3.0);
        let b_43 = b.clone().pow(4.0 / 3.0);

        Ok(DensityVars {
            a,
            b,
            n,
            s,
            gaa,
            gab,
            gbb,
            gnn,
            gns,
            gss,
            taua,
            taub,
            tau,
            lapa,
            lapb,
            jpaa,
            jpbb,
            zeta,
            r_s,
            n_m13,
            a_43,
            b_43,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn from_a_basic() {
        let input = [1.0_f64];
        let dv = DensityVars::from_input(&input, VarType::A).unwrap();
        assert_relative_eq!(dv.a, 1.0, epsilon = 1e-14);
        assert_relative_eq!(dv.b, 0.0, epsilon = 1e-14);
        assert_relative_eq!(dv.n, 1.0, epsilon = 1e-14);
    }

    #[test]
    fn from_a_b() {
        let input = [0.5_f64, 0.3];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        assert_relative_eq!(dv.a, 0.5, epsilon = 1e-14);
        assert_relative_eq!(dv.b, 0.3, epsilon = 1e-14);
        assert_relative_eq!(dv.n, 0.8, epsilon = 1e-14);
    }

    #[test]
    fn from_n_s() {
        let input = [1.0_f64, 0.2];
        let dv = DensityVars::from_input(&input, VarType::N_S).unwrap();
        assert_relative_eq!(dv.n, 1.0, epsilon = 1e-14);
        assert_relative_eq!(dv.a, 0.6, epsilon = 1e-14);
        assert_relative_eq!(dv.b, 0.4, epsilon = 1e-14);
    }

    #[test]
    fn regularization_tiny_density() {
        let input = [1e-20_f64];
        let dv = DensityVars::from_input(&input, VarType::A).unwrap();
        assert_relative_eq!(dv.a, constants::TINY_DENSITY, epsilon = 1e-28);
    }

    #[test]
    fn derived_a_43() {
        let input = [2.0_f64, 1.0];
        let dv = DensityVars::from_input(&input, VarType::A_B).unwrap();
        assert_relative_eq!(dv.a_43, 2.0_f64.powf(4.0 / 3.0), epsilon = 1e-12);
    }

    #[test]
    fn derived_zeta() {
        let input = [1.0_f64, 0.2];
        let dv = DensityVars::from_input(&input, VarType::N_S).unwrap();
        // zeta = s/n = 0.2/1.0 = 0.2
        assert_relative_eq!(dv.zeta, 0.2, epsilon = 1e-14);
    }

    #[test]
    fn from_a_b_gaa_gab_gbb() {
        let input = [0.5_f64, 0.3, 0.1, 0.05, 0.08];
        let dv = DensityVars::from_input(&input, VarType::A_B_GAA_GAB_GBB).unwrap();
        assert_relative_eq!(dv.a, 0.5, epsilon = 1e-14);
        assert_relative_eq!(dv.b, 0.3, epsilon = 1e-14);
        assert_relative_eq!(dv.gaa, 0.1, epsilon = 1e-14);
        assert_relative_eq!(dv.gab, 0.05, epsilon = 1e-14);
        assert_relative_eq!(dv.gbb, 0.08, epsilon = 1e-14);
        // gnn = gaa + 2*gab + gbb = 0.1 + 0.1 + 0.08 = 0.28
        assert_relative_eq!(dv.gnn, 0.28, epsilon = 1e-14);
    }

    #[test]
    fn input_length_mismatch() {
        let input = [1.0_f64];
        let result = DensityVars::from_input(&input, VarType::A_B);
        assert!(result.is_err());
    }
}
