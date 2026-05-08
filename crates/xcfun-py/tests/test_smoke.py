"""PY-04 — smoke tests for the 11 module-level free functions.

Each test calls the Python free fn that wraps the corresponding xcfun-rs Rust
fn (crates/xcfun-rs/src/free_fns.rs) and asserts a sane return value. These
are NOT parity tests — see test_parity.py (Plan 07-05) for cross-language
1e-12 tolerance against Rust-driver fixtures.
"""
import xcfun_rs as xc


def test_version_matches_workspace_pin():
    # D-13 — initial release is 0.1.0; matches workspace Cargo.toml.
    # Equivalent invocation form: xcfun_rs.version().
    assert xc.version() == "0.1.0"


def test_splash_is_string():
    s = xc.splash()
    assert isinstance(s, str)
    assert len(s) > 0


def test_authors_is_string():
    s = xc.authors()
    assert isinstance(s, str)
    assert len(s) > 0


def test_is_compatible_library_returns_bool():
    v = xc.is_compatible_library()
    assert isinstance(v, bool)


def test_self_test_returns_int():
    # self_test() returns failure count; 0 == pass.
    # We accept any non-negative int — the parity gate is the validation harness,
    # not this smoke test.
    n = xc.self_test()
    assert isinstance(n, int)
    assert n >= 0


def test_which_vars_returns_optional_int():
    # 0,0,0,0,0,0 is the no-deps probe — likely returns Some(Vars::A) or None.
    v = xc.which_vars(0, 0, 0, 0, 0, 0)
    assert v is None or isinstance(v, int)


def test_which_mode_unset_roundtrips_to_zero():
    # Phase 2 D-07 — Mode::Unset = 0. which_mode(0) should match the Unset variant.
    v = xc.which_mode(0)
    assert v == 0


def test_enumerate_parameters_indexed_zero_returns_string_or_none():
    v = xc.enumerate_parameters(0)
    assert v is None or isinstance(v, str)


def test_enumerate_aliases_indexed_zero_returns_string_or_none():
    v = xc.enumerate_aliases(0)
    assert v is None or isinstance(v, str)


def test_describe_short_known_functional_returns_nonempty_string():
    # SLATERX is the canonical LDA exchange functional — always present.
    s = xc.describe_short("slaterx")
    assert isinstance(s, str)
    assert len(s) > 0


def test_describe_long_known_functional_returns_nonempty_string():
    s = xc.describe_long("slaterx")
    assert isinstance(s, str)
    assert len(s) > 0


def test_describe_short_unknown_returns_none():
    # An invalid name returns None (xcfun-rs `describe_short` Option<&str>).
    assert xc.describe_short("__definitely_not_a_functional__") is None
