//! 11 free functions per RS-09 + Phase 5 D-03.
//!
//! Source-of-truth file references in each fn doc-comment.

use xcfun_core::{
    ALIASES, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, PARAMETERS, ParameterId, Vars,
};

/// RS-09 — version string. Mirrors xcfun_version (XCFunctional.cpp:48-51).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// RS-09 — splash. Mirrors xcfun_splash (XCFunctional.cpp:53-62).
pub fn splash() -> &'static str {
    include_str!("../assets/splash.txt")
}

/// RS-09 — authors. Mirrors xcfun_authors (XCFunctional.cpp:64-72).
pub fn authors() -> &'static str {
    include_str!("../assets/authors.txt")
}

/// RS-09 — single-process header/library compatibility.
/// Always `true` because the header is generated from THIS crate
/// (Phase 5 Plan 05-03). Mirrors xcfun_is_compatible_library
/// (XCFunctional.cpp:126-129).
pub fn is_compatible_library() -> bool {
    true
}

/// RS-09 — run Tier-1 self-tests over functionals carrying upstream
/// `test_in` data. Returns failure count. Mirrors xcfun_test
/// (XCFunctional.cpp:74-124) but limited to the populated subset
/// (Claude's Discretion per CONTEXT — keep smoke-test fast).
pub fn self_test() -> i32 {
    let mut nfail: i32 = 0;
    for fd in FUNCTIONAL_DESCRIPTORS.iter() {
        if let (
            Some(test_in),
            Some(test_out),
            Some(test_vars),
            Some(test_mode),
            Some(test_order),
            Some(test_threshold),
        ) = (
            fd.test_in,
            fd.test_out,
            fd.test_vars,
            fd.test_mode,
            fd.test_order,
            fd.test_threshold,
        ) {
            let mut fun = crate::Functional::new();
            if fun.set(fd.name, 1.0).is_err() {
                nfail += 1;
                continue;
            }
            if fun.eval_setup(test_vars, test_mode, test_order).is_err() {
                nfail += 1;
                continue;
            }
            let outlen = match fun.output_length() {
                Ok(n) => n,
                Err(_) => {
                    nfail += 1;
                    continue;
                }
            };
            if outlen != test_out.len() {
                nfail += 1;
                continue;
            }
            let mut out = vec![0.0_f64; outlen];
            if fun.eval(test_in, &mut out).is_err() {
                nfail += 1;
                continue;
            }
            for (computed, reference) in out.iter().zip(test_out.iter()) {
                if (computed - reference).abs() > reference.abs() * test_threshold {
                    nfail += 1;
                }
            }
        }
    }
    nfail
}

/// RS-09 — bitwise dispatch port of XCFunctional.cpp:131-277.
/// Out-of-range inputs return `None` (instead of C++ `xcfun::die`).
pub fn which_vars(
    func_type: u32,
    dens_type: u32,
    laplacian: u32,
    kinetic: u32,
    current: u32,
    explicit_derivatives: u32,
) -> Option<Vars> {
    if func_type > 3
        || dens_type > 3
        || laplacian > 1
        || kinetic > 1
        || current > 1
        || explicit_derivatives > 1
    {
        return None;
    }
    let bw = (func_type << 6)
        | (dens_type << 4)
        | (laplacian << 3)
        | (kinetic << 2)
        | (current << 1)
        | explicit_derivatives;
    Some(match bw {
        0 => Vars::A,
        16 => Vars::N,
        32 => Vars::A_B,
        48 => Vars::N_S,
        64 => Vars::A_GAA,
        65 => Vars::A_AX_AY_AZ,
        80 => Vars::N_GNN,
        81 => Vars::N_NX_NY_NZ,
        96 => Vars::A_B_GAA_GAB_GBB,
        97 => Vars::A_B_AX_AY_AZ_BX_BY_BZ,
        112 => Vars::N_S_GNN_GNS_GSS,
        113 => Vars::N_S_NX_NY_NZ_SX_SY_SZ,
        132 => Vars::A_GAA_TAUA,
        133 => Vars::A_AX_AY_AZ_TAUA,
        136 => Vars::A_GAA_LAPA,
        148 => Vars::N_GNN_TAUN,
        149 => Vars::N_NX_NY_NZ_TAUN,
        152 => Vars::N_GNN_LAPN,
        164 => Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        165 => Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB,
        168 => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB,
        172 => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB,
        174 => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB,
        180 => Vars::N_S_GNN_GNS_GSS_TAUN_TAUS,
        181 => Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS,
        184 => Vars::N_S_GNN_GNS_GSS_LAPN_LAPS,
        188 => Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS,
        192 => Vars::A_2ND_TAYLOR,
        208 => Vars::N_2ND_TAYLOR,
        224 => Vars::A_B_2ND_TAYLOR,
        240 => Vars::N_S_2ND_TAYLOR,
        _ => return None,
    })
}

/// RS-09 — port of XCFunctional.cpp:281-300.
pub fn which_mode(mode_type: u32) -> Option<Mode> {
    match mode_type {
        1 => Some(Mode::PartialDerivatives),
        2 => Some(Mode::Potential),
        3 => Some(Mode::Contracted),
        _ => None,
    }
}

/// RS-09 — port of XCFunctional.cpp:302-311.
/// Indices 0..78 → functional names; 78..82 → parameter names.
pub fn enumerate_parameters(param: i32) -> Option<&'static str> {
    if param < 0 {
        return None;
    }
    let i = param as usize;
    // Upstream C++ enumerate_parameters bounds against XC_NR_FUNCTIONALS = 78
    // and XC_NR_PARAMETERS_AND_FUNCTIONALS = 82. Our FUNCTIONAL_DESCRIPTORS
    // table has 79 rows (78 upstream + LB94 stub at index 78 per Phase 5 D-16),
    // but to keep the C ABI behavior we expose only the upstream 78 here:
    // index 78 lands on XC_RANGESEP_MU (PARAMETERS[0]), not on XC_LB94.
    const UPSTREAM_FUNCTIONAL_COUNT: usize = 78;
    if i < UPSTREAM_FUNCTIONAL_COUNT {
        Some(FUNCTIONAL_DESCRIPTORS[i].name)
    } else if i < UPSTREAM_FUNCTIONAL_COUNT + PARAMETERS.len() {
        Some(PARAMETERS[i - UPSTREAM_FUNCTIONAL_COUNT].name)
    } else {
        None
    }
}

/// RS-09 — port of XCFunctional.cpp:313-320.
pub fn enumerate_aliases(n: i32) -> Option<&'static str> {
    if n < 0 {
        return None;
    }
    ALIASES.get(n as usize).map(|a| a.name)
}

/// RS-09 — case-insensitive 3-table cascade. Port of
/// XCFunctional.cpp:322-334.
pub fn describe_short(name: &str) -> Option<&'static str> {
    if let Some(id) = FunctionalId::from_name(name) {
        return Some(FUNCTIONAL_DESCRIPTORS[id as usize].short_description);
    }
    if let Some(pid) = ParameterId::from_name(name) {
        // ParameterId discriminants are 78..81; map back to PARAMETERS index.
        // Use the upstream offset (78) regardless of FUNCTIONAL_DESCRIPTORS.len()
        // since ParameterId discriminants are anchored to XC_NR_FUNCTIONALS = 78.
        const UPSTREAM_FUNCTIONAL_COUNT: usize = 78;
        let off = (pid as usize) - UPSTREAM_FUNCTIONAL_COUNT;
        return Some(PARAMETERS[off].description);
    }
    if let Some(alias) = ALIASES
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
    {
        return Some(alias.description);
    }
    None
}

/// RS-09 — port of XCFunctional.cpp:336-348. Identical 3-table cascade
/// to `describe_short` except returns `long_description` for the
/// functional case.
pub fn describe_long(name: &str) -> Option<&'static str> {
    if let Some(id) = FunctionalId::from_name(name) {
        return Some(FUNCTIONAL_DESCRIPTORS[id as usize].long_description);
    }
    if let Some(pid) = ParameterId::from_name(name) {
        const UPSTREAM_FUNCTIONAL_COUNT: usize = 78;
        let off = (pid as usize) - UPSTREAM_FUNCTIONAL_COUNT;
        return Some(PARAMETERS[off].description);
    }
    if let Some(alias) = ALIASES
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
    {
        return Some(alias.description);
    }
    None
}
