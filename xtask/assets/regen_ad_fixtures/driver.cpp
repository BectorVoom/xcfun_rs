// Fixture driver for xcfun-ad. Compiled by xtask/src/bin/regen_ad_fixtures.rs
// (via a direct c++ compiler invocation — see the binary for the build story)
// and emits deterministic records to stdout in a line-oriented text format.
//
// Record format — one per line, semicolon-separated fields:
//   <op>;<n_var>;<input_count>;<i0>,<i1>,...,<i{k-1}>;<coeff_count>;<c0>,<c1>,...,<c{m-1}>
//
// f64 values are written via printf("%.17g", v) which preserves round-trip
// identity on IEEE-754 double (std::stod / Rust's str::parse::<f64> accept
// "%.17g" output verbatim).
//
// Scope — Phase 1 Plan 05:
//   Expand records: 168 = 3 inputs × 7 orders × 8 fns
//                   (inv, exp, log, pow, sqrt, cbrt, gauss, erf)
//   Mul records:    250 = 50 seeds × 5 N values (n_var ∈ {0, 1, 2, 3, 4})
//   Total:          418
// Phase 1 Plan 06 extends to 1668 by adding add/sub/neg/mul_assign/div.

#include <cstdio>
#include <cstdlib>
#include <cstddef>
#include <cmath>
#include <random>
#include <vector>
#include "ctaylor.hpp"
#include "ctaylor_math.hpp"
#include "tmath.hpp"

static void emit_record(const char * op,
                        int n_var,
                        const std::vector<double> & inputs,
                        const std::vector<double> & coeffs) {
    printf("%s;%d;%zu;", op, n_var, inputs.size());
    for (size_t i = 0; i < inputs.size(); i++) {
        printf("%.17g", inputs[i]);
        if (i + 1 < inputs.size()) printf(",");
    }
    printf(";%zu;", coeffs.size());
    for (size_t i = 0; i < coeffs.size(); i++) {
        printf("%.17g", coeffs[i]);
        if (i + 1 < coeffs.size()) printf(",");
    }
    printf("\n");
}

// ---------------------------------------------------------------------------
//  *_expand record emitters — templated on expansion order N.
// ---------------------------------------------------------------------------

template <int N>
static void emit_inv_expand(double a) {
    double t[N + 1];
    inv_expand<double, N>(t, a);
    std::vector<double> inputs = {a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("inv_expand", N, inputs, coeffs);
}

template <int N>
static void emit_exp_expand(double x0) {
    double t[N + 1];
    exp_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("exp_expand", N, inputs, coeffs);
}

template <int N>
static void emit_log_expand(double x0) {
    double t[N + 1];
    log_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("log_expand", N, inputs, coeffs);
}

template <int N>
static void emit_pow_expand(double x0, double a) {
    double t[N + 1];
    pow_expand<double, N>(t, x0, a);
    std::vector<double> inputs = {x0, a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("pow_expand", N, inputs, coeffs);
}

template <int N>
static void emit_sqrt_expand(double x0) {
    double t[N + 1];
    sqrt_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("sqrt_expand", N, inputs, coeffs);
}

template <int N>
static void emit_cbrt_expand(double x0) {
    double t[N + 1];
    cbrt_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("cbrt_expand", N, inputs, coeffs);
}

template <int N>
static void emit_gauss_expand(double a) {
    double t[N + 1];
    gauss_expand<double, N>(t, a);
    std::vector<double> inputs = {a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("gauss_expand", N, inputs, coeffs);
}

template <int N>
static void emit_erf_expand(double a) {
    double t[N + 1];
    erf_expand<double, N>(t, a);
    std::vector<double> inputs = {a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("erf_expand", N, inputs, coeffs);
}

// ---------------------------------------------------------------------------
//  expm1_expand — inlined port of ctaylor_math.hpp:85-102 for fixture
//  generation. Upstream has no standalone `expm1_expand` template in
//  tmath.hpp (the stable-bracket correction lives inside ctaylor_math.hpp's
//  `expm1(const ctaylor&)`); we mirror the Rust `expm1_expand` body here so
//  the fixture records reflect exactly what the Rust kernel should produce.
// ---------------------------------------------------------------------------
template <int N>
static void emit_expm1_expand(double x0) {
    double t[N + 1];
    exp_expand<double, N>(t, x0);
    if (std::fabs(x0) > 1e-3) {
        t[0] -= 1.0;
    } else {
        t[0] = 2.0 * std::exp(x0 / 2.0) * std::sinh(x0 / 2.0);
    }
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("expm1_expand", N, inputs, coeffs);
}

// ---------------------------------------------------------------------------
//  ctaylor_mul record emitters — one per n_var so template instantiation
//  picks the right per-N base case (ctaylor.hpp:86-152 + :41-65 recursion).
// ---------------------------------------------------------------------------

template <int NVAR>
static void emit_mul_record(const double * a, const double * b) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> ta, tb;
    for (int i = 0; i < SIZE; i++) ta.c[i] = a[i];
    for (int i = 0; i < SIZE; i++) tb.c[i] = b[i];
    ctaylor<double, NVAR> tc = ta * tb;

    std::vector<double> inputs(2 * SIZE);
    for (int i = 0; i < SIZE; i++) inputs[i] = a[i];
    for (int i = 0; i < SIZE; i++) inputs[SIZE + i] = b[i];
    std::vector<double> coeffs(tc.c, tc.c + SIZE);
    emit_record("mul", NVAR, inputs, coeffs);
}

// ---------------------------------------------------------------------------
//  Composed CTaylor record emitters — one per op. Schema:
//    op = "ctaylor_<name>"
//    inputs[0..SIZE] = x.c[0..SIZE]
//    inputs[SIZE]    = extra_arg (optional; for pow, powi)
//    coeffs[0..SIZE] = y.c[0..SIZE]  where  y = op(x)
// ---------------------------------------------------------------------------

template <int NVAR>
static void emit_ctaylor_reciprocal(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    // ctaylor_math.hpp: operator/(S, ctaylor) with S = 1 is the reciprocal.
    ctaylor<double, NVAR> y = 1.0 / x;
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_reciprocal", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_sqrt(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = sqrt(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_sqrt", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_exp(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = exp(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_exp", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_log(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = log(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_log", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_pow(const double * x_in, double a) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    // Explicitly select the real-exponent overload (ctaylor_math.hpp:120).
    ctaylor<double, NVAR> y = pow(x, a);
    std::vector<double> inputs(x_in, x_in + SIZE);
    inputs.push_back(a);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_pow", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_erf(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = erf(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_erf", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_asinh(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = asinh(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_asinh", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_atan(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = atan(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_atan", NVAR, inputs, coeffs);
}

template <int NVAR>
static void emit_ctaylor_powi(const double * x_in, int ie) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    // Integer-exponent overload (ctaylor_math.hpp:165-178).
    ctaylor<double, NVAR> y = pow(x, ie);
    std::vector<double> inputs(x_in, x_in + SIZE);
    inputs.push_back(static_cast<double>(ie));
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_powi", NVAR, inputs, coeffs);
}

// ---------------------------------------------------------------------------
//  ctaylor_expm1 — calls ctaylor_math.hpp:85-102 `expm1` and emits the
//  ctaylor output coefficients (length 1<<NVAR) under the `ctaylor_expm1` op.
// ---------------------------------------------------------------------------
template <int NVAR>
static void emit_ctaylor_expm1(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = expm1(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_expm1", NVAR, inputs, coeffs);
}

// ---------------------------------------------------------------------------
//  ctaylor_sqrtx_asinh_sqrtx — calls ctaylor_math.hpp:275-325
//  `sqrtx_asinh_sqrtx` which chooses between direct and [8,8] Padé branch
//  based on |x.c[0]| vs 0.5. Precondition x.c[0] > -0.5 (upstream assert).
// ---------------------------------------------------------------------------
template <int NVAR>
static void emit_ctaylor_sqrtx_asinh_sqrtx(const double * x_in) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> x;
    for (int i = 0; i < SIZE; i++) x.c[i] = x_in[i];
    ctaylor<double, NVAR> y = sqrtx_asinh_sqrtx(x);
    std::vector<double> inputs(x_in, x_in + SIZE);
    std::vector<double> coeffs(y.c, y.c + SIZE);
    emit_record("ctaylor_sqrtx_asinh_sqrtx", NVAR, inputs, coeffs);
}

int main() {
    // -------------------------------------------------------------------
    //  *_expand records: 3 inputs × 7 orders × 8 fns = 168
    // -------------------------------------------------------------------

    const double inv_inputs[]   = {0.1, 1.0, 10.0};
    const double exp_inputs[]   = {-1.0, 0.0, 2.0};
    const double log_inputs[]   = {0.1, 1.0, 10.0};
    const double sqrt_inputs[]  = {0.1, 1.0, 10.0};
    const double cbrt_inputs[]  = {0.1, 1.0, 10.0};
    const double gauss_inputs[] = {-1.0, 0.0, 1.0};
    const double erf_inputs[]   = {-1.0, 0.0, 1.0};
    // pow: 3 (x0, a) pairs. Preserve x0 > 0 precondition.
    struct PowInput { double x0, a; };
    const PowInput pow_inputs[] = {{1.0, 0.5}, {2.0, 1.5}, {10.0, -1.0}};

    #define EMIT_EXPAND(FN_NAME, INPUTS_ARR)              \
        for (auto & x : INPUTS_ARR) {                      \
            emit_ ## FN_NAME ## _expand<0>(x);             \
            emit_ ## FN_NAME ## _expand<1>(x);             \
            emit_ ## FN_NAME ## _expand<2>(x);             \
            emit_ ## FN_NAME ## _expand<3>(x);             \
            emit_ ## FN_NAME ## _expand<4>(x);             \
            emit_ ## FN_NAME ## _expand<5>(x);             \
            emit_ ## FN_NAME ## _expand<6>(x);             \
        }

    EMIT_EXPAND(inv,   inv_inputs);
    EMIT_EXPAND(exp,   exp_inputs);
    EMIT_EXPAND(log,   log_inputs);
    EMIT_EXPAND(sqrt,  sqrt_inputs);
    EMIT_EXPAND(cbrt,  cbrt_inputs);
    EMIT_EXPAND(gauss, gauss_inputs);
    EMIT_EXPAND(erf,   erf_inputs);

    #undef EMIT_EXPAND

    for (auto & p : pow_inputs) {
        emit_pow_expand<0>(p.x0, p.a);
        emit_pow_expand<1>(p.x0, p.a);
        emit_pow_expand<2>(p.x0, p.a);
        emit_pow_expand<3>(p.x0, p.a);
        emit_pow_expand<4>(p.x0, p.a);
        emit_pow_expand<5>(p.x0, p.a);
        emit_pow_expand<6>(p.x0, p.a);
    }

    // -------------------------------------------------------------------
    //  ctaylor_mul records: 50 seeds × 5 N values (n_var ∈ {0..=4}) = 250.
    //  Deterministic mt19937_64 with a fixed seed so re-running the driver
    //  produces byte-identical output.
    // -------------------------------------------------------------------

    std::mt19937_64 rng(0x1234abcdULL);
    std::uniform_real_distribution<double> dist(-10.0, 10.0);

    for (int n_var = 0; n_var <= 4; n_var++) {
        const int SIZE = 1 << n_var;
        for (int seed_i = 0; seed_i < 50; seed_i++) {
            std::vector<double> a(SIZE), b(SIZE);
            for (int i = 0; i < SIZE; i++) a[i] = dist(rng);
            for (int i = 0; i < SIZE; i++) b[i] = dist(rng);

            switch (n_var) {
                case 0: emit_mul_record<0>(a.data(), b.data()); break;
                case 1: emit_mul_record<1>(a.data(), b.data()); break;
                case 2: emit_mul_record<2>(a.data(), b.data()); break;
                case 3: emit_mul_record<3>(a.data(), b.data()); break;
                case 4: emit_mul_record<4>(a.data(), b.data()); break;
            }
        }
    }

    // -------------------------------------------------------------------
    //  Composed CTaylor records (Plan 01-06).
    //
    //  Shape of every input x:
    //    x.c[0] = x_cnst  (must be > 0 to satisfy sqrt/log/pow preconditions)
    //    x.c[1] = x_var0  (only meaningful when NVAR >= 1)
    //    x.c[i] = 0       for i >= 2 — keeps the input pattern simple and
    //                     matches the Plan 01-06 PLAN.md input spec.
    //
    //  8 composed ops × 3 inputs × 4 n-values (0..=3) = 96 records.
    //  plus ctaylor_pow records: 3 (x, a) pairs × 4 n = 12 (rolled into the
    //  8-op loop below with pow_a taken from the extra_arg column).
    //
    //  7 exponents × 3 inputs × 4 n = 84 powi records.
    //
    //  Grand total composed = 96 + 84 = 180.
    //  Final fixture count = 418 + 180 = 598.
    // -------------------------------------------------------------------

    struct ComposedInput { double x_cnst; double x_var0; double pow_a; };
    const ComposedInput ci[] = {
        {1.0, 0.5,  0.5},
        {2.0, 1.0,  1.5},
        {5.0, -0.1, 2.5},
    };

    // For each (nvar, input), call every composed op. nvar ∈ {0..=3}.
    for (auto & input : ci) {
        // nvar = 0 — SIZE = 1. No x_var0 slot.
        {
            double x[1] = {input.x_cnst};
            emit_ctaylor_reciprocal<0>(x);
            emit_ctaylor_sqrt<0>(x);
            emit_ctaylor_exp<0>(x);
            emit_ctaylor_log<0>(x);
            emit_ctaylor_pow<0>(x, input.pow_a);
            emit_ctaylor_erf<0>(x);
            emit_ctaylor_asinh<0>(x);
            emit_ctaylor_atan<0>(x);
        }
        // nvar = 1 — SIZE = 2.
        {
            double x[2] = {input.x_cnst, input.x_var0};
            emit_ctaylor_reciprocal<1>(x);
            emit_ctaylor_sqrt<1>(x);
            emit_ctaylor_exp<1>(x);
            emit_ctaylor_log<1>(x);
            emit_ctaylor_pow<1>(x, input.pow_a);
            emit_ctaylor_erf<1>(x);
            emit_ctaylor_asinh<1>(x);
            emit_ctaylor_atan<1>(x);
        }
        // nvar = 2 — SIZE = 4.
        {
            double x[4] = {input.x_cnst, input.x_var0, 0.0, 0.0};
            emit_ctaylor_reciprocal<2>(x);
            emit_ctaylor_sqrt<2>(x);
            emit_ctaylor_exp<2>(x);
            emit_ctaylor_log<2>(x);
            emit_ctaylor_pow<2>(x, input.pow_a);
            emit_ctaylor_erf<2>(x);
            emit_ctaylor_asinh<2>(x);
            emit_ctaylor_atan<2>(x);
        }
        // nvar = 3 — SIZE = 8.
        {
            double x[8] = {input.x_cnst, input.x_var0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0};
            emit_ctaylor_reciprocal<3>(x);
            emit_ctaylor_sqrt<3>(x);
            emit_ctaylor_exp<3>(x);
            emit_ctaylor_log<3>(x);
            emit_ctaylor_pow<3>(x, input.pow_a);
            emit_ctaylor_erf<3>(x);
            emit_ctaylor_asinh<3>(x);
            emit_ctaylor_atan<3>(x);
        }
    }

    // -------------------------------------------------------------------
    //  ctaylor_powi records: 7 exponents × 3 inputs × 4 n = 84.
    //
    //  Exponents chosen to cover the fast-path (small positive), the
    //  zero case, and two negative cases that delegate to ctaylor_pow.
    //  x_cnst = 2 is non-zero so negative exponents are defined.
    // -------------------------------------------------------------------

    const int powi_exponents[] = {-2, -1, 0, 1, 2, 5, 10};
    for (int ie_idx = 0; ie_idx < 7; ie_idx++) {
        int ie = powi_exponents[ie_idx];
        for (auto & input : ci) {
            {
                double x[1] = {input.x_cnst};
                emit_ctaylor_powi<0>(x, ie);
            }
            {
                double x[2] = {input.x_cnst, input.x_var0};
                emit_ctaylor_powi<1>(x, ie);
            }
            {
                double x[4] = {input.x_cnst, input.x_var0, 0.0, 0.0};
                emit_ctaylor_powi<2>(x, ie);
            }
            {
                double x[8] = {input.x_cnst, input.x_var0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0};
                emit_ctaylor_powi<3>(x, ie);
            }
        }
    }

    // -------------------------------------------------------------------
    //  Plan 03-00 Task 3 — expm1_expand stratified supplement.
    //
    //  500 x0 points × 5 orders (N ∈ {0,1,2,3,4}) = 2500 records.
    //  Strata exercise the D-05 stable-bracket threshold (|x0| <= 1e-3)
    //  as well as mid and large magnitudes, both signs. Seed fixed for
    //  deterministic regeneration.
    // -------------------------------------------------------------------
    {
        std::mt19937_64 rng_expm1(0xcafebabeULL);
        std::vector<double> strata_x0;
        strata_x0.reserve(500);

        // Stratum 1 — near-zero |x0| in [1e-15, 1e-3], 100 samples
        //   (EXERCISES the |x0| <= 1e-3 stable-bracket path).
        {
            std::uniform_real_distribution<double> s1_log(-15.0, -3.0);
            for (int i = 0; i < 100; i++) {
                double mag = std::pow(10.0, s1_log(rng_expm1));
                double sign = (i % 2 == 0) ? 1.0 : -1.0;
                strata_x0.push_back(sign * mag);
            }
        }
        // Stratum 2 — small |x0| in [1e-3, 1], 100 samples.
        {
            std::uniform_real_distribution<double> s2_log(-3.0, 0.0);
            for (int i = 0; i < 100; i++) {
                double mag = std::pow(10.0, s2_log(rng_expm1));
                double sign = (i % 2 == 0) ? 1.0 : -1.0;
                strata_x0.push_back(sign * mag);
            }
        }
        // Stratum 3 — mid |x0| in [1, 10], 100 samples.
        {
            std::uniform_real_distribution<double> s3_lin(1.0, 10.0);
            for (int i = 0; i < 100; i++) {
                double mag = s3_lin(rng_expm1);
                double sign = (i % 2 == 0) ? 1.0 : -1.0;
                strata_x0.push_back(sign * mag);
            }
        }
        // Stratum 4 — large |x0| in [10, 50], 100 samples.
        {
            std::uniform_real_distribution<double> s4_lin(10.0, 50.0);
            for (int i = 0; i < 100; i++) {
                double mag = s4_lin(rng_expm1);
                double sign = (i % 2 == 0) ? 1.0 : -1.0;
                strata_x0.push_back(sign * mag);
            }
        }
        // Stratum 5 — edge-around-threshold |x0| in [9e-4, 1.1e-3], 100 samples
        //   (exercises the |x0| == 1e-3 bracket boundary transition).
        {
            std::uniform_real_distribution<double> s5_edge(9e-4, 1.1e-3);
            for (int i = 0; i < 100; i++) {
                double mag = s5_edge(rng_expm1);
                double sign = (i % 2 == 0) ? 1.0 : -1.0;
                strata_x0.push_back(sign * mag);
            }
        }

        // Emit 500 × 5 = 2500 scalar records.
        for (double x0 : strata_x0) {
            emit_expm1_expand<0>(x0);
            emit_expm1_expand<1>(x0);
            emit_expm1_expand<2>(x0);
            emit_expm1_expand<3>(x0);
            emit_expm1_expand<4>(x0);
        }

        // Emit 500 × 4 = 2000 ctaylor_expm1 records (NVAR ∈ {0,1,2,3})
        //   Composed-op fixture set: single-variable ctaylor with VAR0=1 and
        //   higher-index slots zeroed. Mirrors the emit_ctaylor_<unary>
        //   pattern used in Plan 01-06.
        for (double x0 : strata_x0) {
            {
                double x[1] = {x0};
                emit_ctaylor_expm1<0>(x);
            }
            {
                double x[2] = {x0, 1.0};
                emit_ctaylor_expm1<1>(x);
            }
            {
                double x[4] = {x0, 1.0, 0.0, 0.0};
                emit_ctaylor_expm1<2>(x);
            }
            {
                double x[8] = {x0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0};
                emit_ctaylor_expm1<3>(x);
            }
        }
    }

    // -------------------------------------------------------------------
    //  Plan 03-00 Task 3 — sqrtx_asinh_sqrtx stratified supplement.
    //
    //  Upstream asserts x.c[0] > -0.5 (ctaylor_math.hpp:277). Domain here is
    //  x0 > 0 to stay comfortably away from the assertion boundary and let
    //  the Padé branch be the stress-tested path for small magnitudes.
    //
    //  500 x0 points × 4 NVAR values (0,1,2,3) = 2000 composed records
    //  (NVAR=4 omitted because it would require extra SIZE=16 inputs and
    //  the Padé branch proof is already covered by NVAR ∈ {0..3}; fixture
    //  budget is 500 × 4 = 2000 records per plan's ≥ 2500 acceptance by
    //  including the expm1 records above, total new = 4500 + 2000).
    // -------------------------------------------------------------------
    {
        std::mt19937_64 rng_sas(0xb16b00b5ULL);
        std::vector<double> strata_x0;
        strata_x0.reserve(500);

        // Stratum 1 — near-zero x0 in [1e-10, 1e-3], 100 samples.
        //   Exercises Padé branch in its most precision-sensitive regime.
        {
            std::uniform_real_distribution<double> s1_log(-10.0, -3.0);
            for (int i = 0; i < 100; i++) {
                strata_x0.push_back(std::pow(10.0, s1_log(rng_sas)));
            }
        }
        // Stratum 2 — Padé domain x0 in [1e-3, 0.4999], 100 samples.
        {
            std::uniform_real_distribution<double> s2_log(-3.0, std::log10(0.4999));
            for (int i = 0; i < 100; i++) {
                strata_x0.push_back(std::pow(10.0, s2_log(rng_sas)));
            }
        }
        // Stratum 3 — Padé/unstable boundary x0 in [0.4, 0.6], 100 samples.
        //   Exercises BOTH branches + branch-transition continuity.
        {
            std::uniform_real_distribution<double> s3_lin(0.4, 0.6);
            for (int i = 0; i < 100; i++) {
                strata_x0.push_back(s3_lin(rng_sas));
            }
        }
        // Stratum 4 — unstable branch mid x0 in [0.6, 10], 100 samples.
        {
            std::uniform_real_distribution<double> s4_log(std::log10(0.6), 1.0);
            for (int i = 0; i < 100; i++) {
                strata_x0.push_back(std::pow(10.0, s4_log(rng_sas)));
            }
        }
        // Stratum 5 — unstable branch large x0 in [10, 100], 100 samples.
        {
            std::uniform_real_distribution<double> s5_log(1.0, 2.0);
            for (int i = 0; i < 100; i++) {
                strata_x0.push_back(std::pow(10.0, s5_log(rng_sas)));
            }
        }

        for (double x0 : strata_x0) {
            {
                double x[1] = {x0};
                emit_ctaylor_sqrtx_asinh_sqrtx<0>(x);
            }
            {
                double x[2] = {x0, 1.0};
                emit_ctaylor_sqrtx_asinh_sqrtx<1>(x);
            }
            {
                double x[4] = {x0, 1.0, 0.0, 0.0};
                emit_ctaylor_sqrtx_asinh_sqrtx<2>(x);
            }
            {
                double x[8] = {x0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0};
                emit_ctaylor_sqrtx_asinh_sqrtx<3>(x);
            }
        }
    }

    return 0;
}
