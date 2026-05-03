"""Phase 6 D-04 — mpmath sidecar entry point.

Invoked from Rust via:
  python3 -m xtask.mpmath_eval --functional <name> --vars <Vars> --mode <Mode>
                                --order <u32> --input <comma-sep> --prec 200

Emits one JSONL record on stdout. Reproducibility: mp.prec=200 + mpmath>=1.4.

This module imports `mpmath` at runtime — it is invoked only by
`xtask/src/bin/regen_mpmath_fixtures.rs` and not by any library crate
build. `import xtask.mpmath_eval` (the package marker) does NOT load
`mpmath`; only `python3 -m xtask.mpmath_eval` does.
"""
import argparse
import json
import sys


def main() -> None:
    # Import mpmath inside main() so a bare `python3 -c "import xtask.mpmath_eval"`
    # smoke test (acceptance criterion) does not require mpmath to be installed.
    import mpmath
    from .evaluator import eval_record

    p = argparse.ArgumentParser(prog="xtask.mpmath_eval")
    p.add_argument("--functional", required=True)
    p.add_argument("--vars", required=True)
    p.add_argument("--mode", required=True)
    p.add_argument("--order", type=int, required=True)
    p.add_argument("--input", required=True)
    p.add_argument("--prec", type=int, default=200)
    args = p.parse_args()

    mpmath.mp.prec = args.prec
    inputs = [mpmath.mpf(x) for x in args.input.split(",")]
    record = eval_record(
        args.functional, args.vars, args.mode, args.order, inputs, args.prec
    )
    json.dump(record, sys.stdout)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
