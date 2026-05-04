"""Generic Taylor-series AD chain at mp.prec=200.

Phase 6 Plan 06-N2 substrate. Provides multivariate Taylor coefficient
extraction for arbitrary scalar f: tuple[mp.mpf, ...] -> mp.mpf at
arbitrary order. The mpmath sidecar uses these helpers to produce
ground-truth output vectors that mirror the C++ xcfun output layout
(rev-gradlex per `xcfun-master/api/xcfun.h:117`).

Algorithm:
    `mp.diff(f, point, n=multi_index)` produces the unscaled
    multivariate partial derivative
        \\partial^{|multi_index|} f / \\prod_i \\partial x_i^{multi_index[i]}
    at `point`. xcfun's `xcfun_eval` output convention divides each entry
    by the multinomial factorial \\prod_i (multi_index[i])! per
    `xcfun-master/src/XCFunctional.cpp:577-612` (the per-variable
    Taylor-coefficient seed pattern in[i][VAR_i] = 1 already encodes
    1/k! through CTaylor's seeding). This module returns multinomial
    Taylor coefficients (with the 1/k! factor folded in) so the output
    can be compared directly to `xcfun_eval` output.

Output ordering:
    Order 0:  [f(p)]                             (length 1)
    Order 1:  [f(p), \\partial_0 f, ..., \\partial_{N-1} f]
                                                  (length 1 + N)
    Order 2:  ... + lex (i, j) with i <= j        (triangle of N*(N+1)/2)
    Order 3:  ... + lex (i, j, k) with i <= j <= k
    ...
    Total length = taylorlen(N, order) per the
    `xcfun-core::taylorlen` (`crates/xcfun-core/src/lib.rs:34-42`).

Reference: cross-check against `validation/src/driver.rs::run_one_tuple_pd`
output layout — element 0 is energy, then first derivatives, then second-
order lex pairs, etc.
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
    """Yield all multi-indices for orders 1..=`order` in xcfun rev-gradlex.

    For order m and n_vars N, the indices are sorted i_0 <= i_1 <= ...
    <= i_{m-1} (lex order over the chosen index tuple), each yielding
    a multi-index `mi` of length N with mi[i_k]+=1 per (i_k).

    Yields (multi_index_tuple, factorial_factor) pairs where
    factorial_factor = \\prod_v (mi[v])! .
    """
    if order < 1:
        return
    indices = [0] * order
    while True:
        # Build the multi-index from the current sorted tuple.
        mi = [0] * n_vars
        for v in indices:
            mi[v] += 1
        fact = 1
        for k in mi:
            fact *= mp.factorial(k)
        yield tuple(mi), fact
        # Increment the sorted tuple lex-style with i_0 <= i_1 <= ...
        # Find rightmost index that can be incremented while keeping the
        # sorted invariant.
        for pos in range(order - 1, -1, -1):
            if indices[pos] < n_vars - 1:
                indices[pos] += 1
                # Reset trailing positions to indices[pos] (preserve sorted).
                for q in range(pos + 1, order):
                    indices[q] = indices[pos]
                break
        else:
            return


def multivariate_taylor(f, point, order, prec=200):
    """Multivariate Taylor expansion at `point`.

    Returns a list of mp.mpf coefficients in the canonical xcfun output
    order (rev-gradlex over orders 0..=order). The energy slot
    [`output[0]`] equals `f(point)`. Subsequent slots populate the
    upper-triangle of the multi-index space.

    Args:
        f: callable accepting `len(point)` positional mp.mpf arguments
           and returning mp.mpf.
        point: tuple of mp.mpf input values.
        order: maximum derivative order (0..=4 for current tier-2).
        prec: mpmath working precision in bits (default 200).

    The unscaled multivariate derivative
        D^{mi} f = \\partial^{|mi|} f / \\prod_v \\partial x_v^{mi[v]}
    is computed via mpmath's
        mp.diff(f, point, n=mi)
    and divided by \\prod_v (mi[v])! to yield the Taylor coefficient.

    Multivariate `mp.diff` requires Python 3 unpacking of the point
    tuple; the helper invokes `f(*args)`.
    """
    mp.mp.prec = prec
    n = len(point)
    pt = tuple(mp.mpf(p) for p in point)
    out = []
    # Order 0 — energy.
    out.append(mp.mpf(f(*pt)))
    # Orders 1..=order — multinomial Taylor coefficients.
    for k in range(1, order + 1):
        for mi, fact in _multi_indices(n, k):
            d = mp.diff(f, pt, n=mi)
            out.append(mp.mpf(d) / fact)
    return out
