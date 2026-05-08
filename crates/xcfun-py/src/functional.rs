//! `Functional` `#[pyclass]` + `Mode`/`Vars` IntEnums + 11 module-level free fns.
//!
//! Phase 7 Plan 07-02 ships ONLY the `free_fns` submodule (PY-04). The
//! `Functional` class + IntEnums are added by Plan 07-04 (PY-02);
//! the `eval_vec` numpy zero-copy path is added by Plan 07-05 (PY-03).

pub mod free_fns {
    //! Module-level free fns. Each delegates 1:1 to `xcfun_rs::<fn>`.
    //!
    //! Source signatures: crates/xcfun-rs/src/free_fns.rs.

    use pyo3::prelude::*;
    use xcfun_rs as rs;

    /// xcfun_rs.version() -> str.
    #[pyfunction]
    pub fn version() -> &'static str { rs::version() }

    /// xcfun_rs.splash() -> str.
    #[pyfunction]
    pub fn splash() -> &'static str { rs::splash() }

    /// xcfun_rs.authors() -> str.
    #[pyfunction]
    pub fn authors() -> &'static str { rs::authors() }

    /// xcfun_rs.is_compatible_library() -> bool.
    #[pyfunction]
    pub fn is_compatible_library() -> bool { rs::is_compatible_library() }

    /// xcfun_rs.self_test() -> int — failure count (0 = pass).
    #[pyfunction]
    pub fn self_test() -> i32 { rs::self_test() }

    /// xcfun_rs.which_vars(...) -> Optional[int].
    #[pyfunction]
    pub fn which_vars(
        func_type: u32, dens_type: u32,
        laplacian: u32, kinetic: u32,
        current: u32, explicit_derivatives: u32,
    ) -> Option<u32> {
        rs::which_vars(func_type, dens_type, laplacian, kinetic, current,
                       explicit_derivatives).map(|v| v as u32)
    }

    /// xcfun_rs.which_mode(mode_type) -> Optional[int].
    #[pyfunction]
    pub fn which_mode(mode_type: u32) -> Option<u32> {
        rs::which_mode(mode_type).map(|m| m as u32)
    }

    #[pyfunction]
    pub fn enumerate_parameters(p: i32) -> Option<&'static str> {
        rs::enumerate_parameters(p)
    }

    #[pyfunction]
    pub fn enumerate_aliases(n: i32) -> Option<&'static str> {
        rs::enumerate_aliases(n)
    }

    #[pyfunction]
    pub fn describe_short(name: &str) -> Option<&'static str> {
        rs::describe_short(name)
    }

    #[pyfunction]
    pub fn describe_long(name: &str) -> Option<&'static str> {
        rs::describe_long(name)
    }
}
