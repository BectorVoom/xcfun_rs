"""Generic Taylor-series AD chain at mp.prec=200.

Phase 6 Plan 06-N2 substrate. Provides multivariate partial-derivative
extraction for arbitrary scalar f: tuple[mp.mpf, ...] -> mp.mpf at
arbitrary order. The mpmath sidecar uses these helpers to produce
ground-truth output vectors matching the C++ xcfun_eval output layout
(rev-gradlex per `xcfun-master/api/xcfun.h:117`).

# Output convention — RAW partial derivatives (NOT Taylor coefficients)

xcfun's `xcfun_eval` output at a multi-index `mi = (k_0, ..., k_{N-1})`
with `|mi| = sum k_v` is the RAW partial derivative
    output[mi] = D^{|mi|} f / prod_v ∂x_v^{k_v}
NOT the Taylor coefficient (which would carry 1/prod_v k_v!).

This convention follows from the C++ `XCFunctional.cpp:577-612`
per-variable seeding pattern. For a diagonal multi-index (k, 0, ..., 0)
the C++ harness seeds slot 0 with two unit infinitesimals (`in[0][VAR0]=1`
and `in[0][VAR1]=1`) and reads `out[VAR0|VAR1]`. For
    f(x_0 + ε_0 + ε_1) = f + f'·(ε_0+ε_1) + (1/2)f''·(ε_0+ε_1)^2 + ...
                       = f + ... + f''·ε_0·ε_1 + (1/2)f''·ε_0^2 + (1/2)f''·ε_1^2 + ...
the ε_0·ε_1 coefficient is `f''(x_0)` (raw second derivative, no 1/2).
For an off-diagonal multi-index the same seed pattern lives on different
slots, again giving the raw mixed partial derivative `f''_{ij}`.

`mp.diff(f, point, n=mi)` returns `D^{|mi|} f / prod_v ∂x_v^{k_v}`
directly — exactly what xcfun's output convention demands. No
multinomial-factorial division is performed here.

# Output ordering

    Order 0:  [f(p)]                             (length 1)
    Order 1:  [f(p), ∂_0 f, ..., ∂_{N-1} f]      (length 1 + N)
    Order 2:  ... + lex (i, j) with i <= j        (triangle of N*(N+1)/2)
    Order 3:  ... + lex (i, j, k) with i <= j <= k
    ...
    Total length = taylorlen(N, order) per
    `xcfun-core::taylorlen` (`crates/xcfun-core/src/lib.rs:34-42`).

Reference: `validation/src/driver.rs::run_one_tuple_pd` output layout
-- element 0 is energy, then first derivatives, then second-order lex
pairs (raw mixed partials), etc.
"""
from __future__ import annotations
import mpmath as mp


def taylor_coeffs(f, x0, order, prec=200):
    """Single-variable Taylor coefficients at x0, scaled by 1/k!.

    Returns [f(x0), f'(x0), f''(x0)/2!, ..., f^{(order)}(x0)/order!] as a
    list of mp.mpf values. Used by per-functional ports for internal
    scalar Taylor expansions (e.g., the BR-Newton root function expanded
    around a fixed reciprocal value, mirroring xcfun-master/src/functionals/brx.cpp:50-71).
    """
    mp.mp.prec = prec
    out = []
    for k in range(order + 1):
        deriv = mp.diff(f, x0, n=k)
        out.append(mp.mpf(deriv) / mp.factorial(k))
    return out


def _multi_indices(n_vars, order):
    """Yield all multi-indices for order `order` in xcfun rev-gradlex.

    For order m and n_vars N, the indices are sorted i_0 <= i_1 <= ...
    <= i_{m-1} (lex order over the chosen index tuple), each yielding
    a multi-index `mi` of length N with mi[i_k] += 1 per (i_k). This
    matches the i ≤ j ≤ k iteration order of the C++ XCFunctional.cpp
    seeding loops at `:577-612`.
    """
    if order < 1:
        return
    indices = [0] * order
    while True:
        mi = [0] * n_vars
        for v in indices:
            mi[v] += 1
        yield tuple(mi)
        # Increment the sorted tuple lex-style.
        for pos in range(order - 1, -1, -1):
            if indices[pos] < n_vars - 1:
                indices[pos] += 1
                for q in range(pos + 1, order):
                    indices[q] = indices[pos]
                break
        else:
            return


def multivariate_taylor(f, point, order, prec=200):
    """Multivariate raw-partial-derivative vector at `point`.

    Returns a list of mp.mpf entries in the canonical xcfun output
    order (rev-gradlex over orders 0..=order). The energy slot
    `output[0]` equals `f(point)`. Subsequent slots populate the
    upper-triangle of the multi-index space with RAW partial
    derivatives `D^{|mi|} f / prod_v ∂x_v^{mi[v]}` — NO multinomial
    factorial denominator (see module docstring for the rationale; this
    matches xcfun's per-VAR seeding output convention).

    Args:
        f: callable accepting `len(point)` positional mp.mpf arguments
           and returning mp.mpf.
        point: tuple of mp.mpf input values.
        order: maximum derivative order (0..=4 for current tier-2).
        prec: mpmath working precision in bits (default 200).
    """
    mp.mp.prec = prec
    pt = tuple(mp.mpf(p) for p in point)
    out = [mp.mpf(f(*pt))]
    n = len(point)
    for k in range(1, order + 1):
        for mi in _multi_indices(n, k):
            d = mp.diff(f, pt, n=mi)
            out.append(mp.mpf(d))
    return out
