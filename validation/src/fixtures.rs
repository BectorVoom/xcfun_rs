//! 10 000-point stratified grid generator stub — replaced in Wave-2-4 with
//! the xoshiro-seeded implementation.

/// Placeholder grid point — Wave-2-4 fills in the 8 `f64` fields
/// (`n, s, gnn, gns, gss, gaa, gab, gbb`) and helpers.
#[derive(Clone, Copy, Debug, Default)]
pub struct GridPoint;

/// Placeholder grid generator — Wave-2-4 returns 10 000 points.
pub fn generate_grid() -> Vec<GridPoint> {
    Vec::new()
}
