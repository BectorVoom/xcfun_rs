//! GPU-04 / D-15 — `Functional::settings_generation()` is monotonic.
//!
//! Phase 6 Plan 06-02a contract: every successful `set()` call bumps
//! the counter exactly once per `settings[]` mutation. The counter is
//! consumed by `Batch::launch` to decide whether the cached
//! `weights_buf` upload is stale.
//!
//! Lives in xcfun-gpu (rather than xcfun-eval) because the contract
//! exists for the GPU buffer-pool's benefit; xcfun-eval just provides
//! the accessor.

use xcfun_eval::Functional;

#[test]
fn fresh_functional_has_generation_zero() {
    let fun = Functional::new();
    assert_eq!(fun.settings_generation(), 0);
}

#[test]
fn functional_set_bumps_generation() {
    let mut fun = Functional::new();
    let g0 = fun.settings_generation();
    fun.set("slaterx", 1.0).expect("set slaterx");
    let g1 = fun.settings_generation();
    assert!(g1 > g0, "expected strict bump: g0={g0} g1={g1}");
}

#[test]
fn parameter_set_bumps_generation() {
    let mut fun = Functional::new();
    let g0 = fun.settings_generation();
    fun.set("rangesep_mu", 0.4).expect("set parameter");
    let g1 = fun.settings_generation();
    assert!(g1 > g0, "g0={g0} g1={g1}");
}

#[test]
fn alias_set_bumps_generation_per_resolved_term() {
    // `B3LYP` is an alias that resolves to multiple functional + parameter
    // terms. Each recursive set() bump is observable.
    let mut fun = Functional::new();
    let g0 = fun.settings_generation();
    fun.set("b3lyp", 1.0).expect("set alias");
    let g1 = fun.settings_generation();
    assert!(
        g1 > g0,
        "alias set must bump at least once: g0={g0} g1={g1}"
    );
    // Each component term in B3LYP triggers its own bump; we expect
    // strictly more than 1 bump for any multi-term alias.
    assert!(
        g1 - g0 >= 2,
        "B3LYP is a multi-term alias; expected >= 2 bumps, got {}",
        g1 - g0
    );
}

#[test]
fn unknown_name_does_not_bump_generation() {
    let mut fun = Functional::new();
    let g0 = fun.settings_generation();
    let _ = fun.set("not_a_real_functional_name", 1.0);
    let g1 = fun.settings_generation();
    assert_eq!(
        g0, g1,
        "failed set() must not advance the generation counter"
    );
}

#[test]
fn multiple_sets_strictly_increase_generation() {
    let mut fun = Functional::new();
    let g0 = fun.settings_generation();
    fun.set("slaterx", 1.0).unwrap();
    let g1 = fun.settings_generation();
    fun.set("slaterx", 0.5).unwrap();
    let g2 = fun.settings_generation();
    fun.set("rangesep_mu", 0.3).unwrap();
    let g3 = fun.settings_generation();
    assert!(g0 < g1, "g0={g0} g1={g1}");
    assert!(g1 < g2, "g1={g1} g2={g2}");
    assert!(g2 < g3, "g2={g2} g3={g3}");
}
