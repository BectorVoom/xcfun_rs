// xcfun-ad fixture driver — Phase 1 Plan 01 scaffolding.
//
// This driver links against xcfun-master/external/upstream/taylor/ and emits
// bincode-compatible records. Phase 1 Plan 05 fills in the emission logic.

#include <cstdio>
#include "ctaylor.hpp"
#include "ctaylor_math.hpp"
#include "tmath.hpp"

int main() {
    // Smoke test: construct a ctaylor<double, 1> and print its coefficient.
    ctaylor<double, 1> x;
    x.c[0] = 1.0;
    x.c[1] = 2.0;
    printf("driver ok: x.c[0]=%.17g x.c[1]=%.17g\n", x.c[0], x.c[1]);
    return 0;
}
