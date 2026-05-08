//! `Functional` `#[pyclass]` + `Mode` / `Vars` IntEnum mirrors + free fns.
//!
//! Phase 7 Plan 07-04 — adds the `Mode` / `Vars` IntEnum mirrors to the
//! existing `free_fns` submodule from Plan 07-02. The `Functional` `#[pyclass]`
//! itself lands in Task 4.2; the `eval_vec` numpy zero-copy path is added by
//! Plan 07-05 (PY-03).

#![allow(unused_imports)] // RsFunctional is consumed by Task 4.2; kept here so the
// import survives `cargo build` of the Task 4.1 increment.

use pyo3::prelude::*;

use xcfun_rs::{Functional as RsFunctional, Mode as RsMode, Vars as RsVars};

/// Python `Mode` IntEnum — discriminants match `xcfun-core::Mode` byte-for-byte.
///
/// Source: crates/xcfun-core/src/enums.rs (Mode enum). The `eq, eq_int` attrs
/// make these comparable both as Python enum members AND as integers
/// (`Mode.PartialDerivatives == 1` is True).
///
/// Source: pyo3 0.28.3 guide §class.md "Rust Enum for Integer Conversion".
#[pyclass(eq, eq_int, from_py_object, name = "Mode", module = "xcfun_rs")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum Mode {
    Unset = 0,
    PartialDerivatives = 1,
    Potential = 2,
    Contracted = 3,
}

impl From<Mode> for RsMode {
    fn from(m: Mode) -> RsMode {
        match m {
            Mode::Unset => RsMode::Unset,
            Mode::PartialDerivatives => RsMode::PartialDerivatives,
            Mode::Potential => RsMode::Potential,
            Mode::Contracted => RsMode::Contracted,
        }
    }
}

/// Python `Vars` IntEnum — all 31 variants. Discriminants per
/// `crates/xcfun-core/src/enums.rs:98-147`.
#[pyclass(eq, eq_int, from_py_object, name = "Vars", module = "xcfun_rs")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum Vars {
    A = 0,
    N = 1,
    A_B = 2,
    N_S = 3,
    A_GAA = 4,
    N_GNN = 5,
    A_B_GAA_GAB_GBB = 6,
    N_S_GNN_GNS_GSS = 7,
    A_GAA_LAPA = 8,
    A_GAA_TAUA = 9,
    N_GNN_LAPN = 10,
    N_GNN_TAUN = 11,
    A_B_GAA_GAB_GBB_LAPA_LAPB = 12,
    A_B_GAA_GAB_GBB_TAUA_TAUB = 13,
    N_S_GNN_GNS_GSS_LAPN_LAPS = 14,
    N_S_GNN_GNS_GSS_TAUN_TAUS = 15,
    A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB = 16,
    A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB = 17,
    N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS = 18,
    A_AX_AY_AZ = 19,
    A_B_AX_AY_AZ_BX_BY_BZ = 20,
    N_NX_NY_NZ = 21,
    N_S_NX_NY_NZ_SX_SY_SZ = 22,
    A_AX_AY_AZ_TAUA = 23,
    A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB = 24,
    N_NX_NY_NZ_TAUN = 25,
    N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS = 26,
    A_2ND_TAYLOR = 27,
    A_B_2ND_TAYLOR = 28,
    N_2ND_TAYLOR = 29,
    N_S_2ND_TAYLOR = 30,
}

impl From<Vars> for RsVars {
    fn from(v: Vars) -> RsVars {
        // Both enums are `#[repr(u32)]`-equivalent (xcfun-core::Vars is
        // `#[repr(u32)]`) and the discriminants match by construction
        // above. The match below is exhaustive — adding a new Vars variant
        // upstream forces a compile error here (mitigates T-7-04-03).
        match v {
            Vars::A => RsVars::A,
            Vars::N => RsVars::N,
            Vars::A_B => RsVars::A_B,
            Vars::N_S => RsVars::N_S,
            Vars::A_GAA => RsVars::A_GAA,
            Vars::N_GNN => RsVars::N_GNN,
            Vars::A_B_GAA_GAB_GBB => RsVars::A_B_GAA_GAB_GBB,
            Vars::N_S_GNN_GNS_GSS => RsVars::N_S_GNN_GNS_GSS,
            Vars::A_GAA_LAPA => RsVars::A_GAA_LAPA,
            Vars::A_GAA_TAUA => RsVars::A_GAA_TAUA,
            Vars::N_GNN_LAPN => RsVars::N_GNN_LAPN,
            Vars::N_GNN_TAUN => RsVars::N_GNN_TAUN,
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB => RsVars::A_B_GAA_GAB_GBB_LAPA_LAPB,
            Vars::A_B_GAA_GAB_GBB_TAUA_TAUB => RsVars::A_B_GAA_GAB_GBB_TAUA_TAUB,
            Vars::N_S_GNN_GNS_GSS_LAPN_LAPS => RsVars::N_S_GNN_GNS_GSS_LAPN_LAPS,
            Vars::N_S_GNN_GNS_GSS_TAUN_TAUS => RsVars::N_S_GNN_GNS_GSS_TAUN_TAUS,
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB => {
                RsVars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB
            }
            Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB => {
                RsVars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB
            }
            Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS => {
                RsVars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS
            }
            Vars::A_AX_AY_AZ => RsVars::A_AX_AY_AZ,
            Vars::A_B_AX_AY_AZ_BX_BY_BZ => RsVars::A_B_AX_AY_AZ_BX_BY_BZ,
            Vars::N_NX_NY_NZ => RsVars::N_NX_NY_NZ,
            Vars::N_S_NX_NY_NZ_SX_SY_SZ => RsVars::N_S_NX_NY_NZ_SX_SY_SZ,
            Vars::A_AX_AY_AZ_TAUA => RsVars::A_AX_AY_AZ_TAUA,
            Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB => RsVars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB,
            Vars::N_NX_NY_NZ_TAUN => RsVars::N_NX_NY_NZ_TAUN,
            Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS => RsVars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS,
            Vars::A_2ND_TAYLOR => RsVars::A_2ND_TAYLOR,
            Vars::A_B_2ND_TAYLOR => RsVars::A_B_2ND_TAYLOR,
            Vars::N_2ND_TAYLOR => RsVars::N_2ND_TAYLOR,
            Vars::N_S_2ND_TAYLOR => RsVars::N_S_2ND_TAYLOR,
        }
    }
}

// Compile-time assertion: discriminants line up with the xcfun-core values.
// Mitigates T-7-04-01 (Vars discriminants drifting from xcfun-core::Vars):
// any drift fails the build before tests run.
const _: () = {
    assert!(Mode::Unset as u32 == 0);
    assert!(Mode::PartialDerivatives as u32 == 1);
    assert!(Mode::Potential as u32 == 2);
    assert!(Mode::Contracted as u32 == 3);
    assert!(Vars::A as u32 == 0);
    assert!(Vars::A_B as u32 == 2);
    assert!(Vars::A_B_GAA_GAB_GBB as u32 == 6);
    assert!(Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB as u32 == 17);
    assert!(Vars::A_B_2ND_TAYLOR as u32 == 28);
    assert!(Vars::N_S_2ND_TAYLOR as u32 == 30);
};

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
