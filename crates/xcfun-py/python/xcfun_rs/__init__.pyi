"""Type stubs for xcfun_rs (PEP 561).

Phase 7 Plan 07-02 — 11 free fns.
Phase 7 Plan 07-03 — XcfunError.
Phase 7 Plan 07-04 — Functional pyclass + Mode/Vars IntEnums.
"""
from typing import Optional

import numpy as np
from numpy.typing import NDArray

def version() -> str: ...
def splash() -> str: ...
def authors() -> str: ...
def is_compatible_library() -> bool: ...
def self_test() -> int: ...
def which_vars(
    func_type: int,
    dens_type: int,
    laplacian: int,
    kinetic: int,
    current: int,
    explicit_derivatives: int,
) -> Optional[int]: ...
def which_mode(mode_type: int) -> Optional[int]: ...
def enumerate_parameters(p: int) -> Optional[str]: ...
def enumerate_aliases(n: int) -> Optional[str]: ...
def describe_short(name: str) -> Optional[str]: ...
def describe_long(name: str) -> Optional[str]: ...

# Phase 7 Plan 07-03 — XcfunError stub.

class XcfunError(Exception):
    code: int
    kind: str
    def __init__(self, *args: object) -> None: ...


# Phase 7 Plan 07-04 — Mode / Vars IntEnums + Functional class.

class Mode:
    Unset: int
    PartialDerivatives: int
    Potential: int
    Contracted: int


class Vars:
    A: int
    N: int
    A_B: int
    N_S: int
    A_GAA: int
    N_GNN: int
    A_B_GAA_GAB_GBB: int
    N_S_GNN_GNS_GSS: int
    A_GAA_LAPA: int
    A_GAA_TAUA: int
    N_GNN_LAPN: int
    N_GNN_TAUN: int
    A_B_GAA_GAB_GBB_LAPA_LAPB: int
    A_B_GAA_GAB_GBB_TAUA_TAUB: int
    N_S_GNN_GNS_GSS_LAPN_LAPS: int
    N_S_GNN_GNS_GSS_TAUN_TAUS: int
    A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB: int
    A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB: int
    N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS: int
    A_AX_AY_AZ: int
    A_B_AX_AY_AZ_BX_BY_BZ: int
    N_NX_NY_NZ: int
    N_S_NX_NY_NZ_SX_SY_SZ: int
    A_AX_AY_AZ_TAUA: int
    A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB: int
    N_NX_NY_NZ_TAUN: int
    N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS: int
    A_2ND_TAYLOR: int
    A_B_2ND_TAYLOR: int
    N_2ND_TAYLOR: int
    N_S_2ND_TAYLOR: int


class Functional:
    def __init__(
        self,
        name: str,
        *,
        vars: Optional[Vars] = ...,
        mode: Optional[Mode] = ...,
        order: Optional[int] = ...,
    ) -> None: ...
    def configure(self, vars: Vars, mode: Mode, order: int) -> None: ...
    def set(self, name: str, value: float) -> None: ...
    def get(self, name: str) -> float: ...
    def is_gga(self) -> bool: ...
    def is_metagga(self) -> bool: ...
    def eval_setup(self, vars: Vars, mode: Mode, order: int) -> None: ...
    def user_eval_setup(
        self,
        order: int,
        func_type: int,
        dens_type: int,
        mode_type: int,
        laplacian: int,
        kinetic: int,
        current: int,
        explicit_derivatives: int,
    ) -> None: ...
    def input_length(self) -> int: ...
    def output_length(self) -> int: ...
    def eval(self, density: NDArray[np.float64], out: NDArray[np.float64]) -> None: ...
    def eval_vec(self, densities: NDArray[np.float64]) -> NDArray[np.float64]: ...


__version__: str
