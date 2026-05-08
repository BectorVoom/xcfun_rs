"""Phase 7 pytest fixtures shared across tests/*.py."""
import pytest


@pytest.fixture(scope="session", autouse=True)
def _xcfun_rs_importable():
    """Sanity check: importing xcfun_rs must succeed before any test runs.

    If `maturin develop --release` has not been run, this fails fast with a
    helpful message instead of cryptic ImportError trickle-down.
    """
    try:
        import xcfun_rs  # noqa: F401
    except ImportError as e:
        pytest.exit(
            f"xcfun_rs not importable: {e}\n"
            "Run: cd crates/xcfun-py && maturin develop --release",
            returncode=4,
        )
