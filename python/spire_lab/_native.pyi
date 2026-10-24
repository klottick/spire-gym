"""Transport signatures for the compiled extension; no Python gameplay."""

from collections.abc import Callable
from typing import Any, TypedDict

import numpy as np
from numpy.typing import NDArray

type RunObservation = dict[str, NDArray[np.int64] | NDArray[np.int8]]

class RunInfo(TypedDict):
    schema_id: str
    terminal_reason: str | None

class RunBoundaryPayload(TypedDict):
    obs: RunObservation
    token: tuple[int, int]
    info: RunInfo

class RunResetPayload(RunBoundaryPayload):
    effective_seed: int

class RunStepPayload(RunBoundaryPayload):
    reward: int
    terminated: bool
    truncated: bool

class RunReproduction(TypedDict):
    seed: int | None
    ordinal: int
    closed: bool
    fault: str | None
    last_accepted_action: str | None
    record: bytes | None

class RunReplay(TypedDict):
    obs: RunObservation
    terminal_reason: str | None
    rewards: list[int]
    terminals: list[str | None]
    accepted_count: int
    reported_fault: dict[str, object] | None
    final_decision_error: str | None

class RunInputError(ValueError): ...
class RunStateError(RuntimeError): ...
class RunFaultError(RuntimeError): ...
class RunUnsupportedBoundaryError(NotImplementedError): ...

class NativeRunSession:
    def __init__(
        self,
        character: str,
        ascension: int,
        *,
        partition: str = ...,
        boss_history: str = ...,
        capacities: dict[str, int] | None = ...,
        record_replay: bool = ...,
    ) -> None: ...
    def observation_spec(self) -> dict[str, Any]: ...
    def reset(self, seed: int | None = ...) -> RunResetPayload: ...
    def step(self, index: int, token: tuple[int, int]) -> RunStepPayload: ...
    def _reset_deliver[T](
        self, package: Callable[[RunResetPayload], T], *, seed: int | None = ...
    ) -> T: ...
    def _step_deliver[T](
        self, index: int, token: tuple[int, int], package: Callable[[RunStepPayload], T]
    ) -> T: ...
    def record(self) -> bytes | None: ...
    def reproduction(self) -> RunReproduction: ...
    def close(self) -> None: ...
    @staticmethod
    def replay(record: bytes, capacities: dict[str, int] | None = ...) -> RunReplay: ...

# The accepted legacy interface has dynamic catalog/info/record dictionaries.
class CombatInputError(ValueError):
    slot: int | None
    reason: str
    faults: list[dict[str, Any]]

class CombatStateError(RuntimeError):
    slot: int | None
    reason: str
    faults: list[dict[str, Any]]

class CombatSetupError(RuntimeError): ...
class CombatFaultError(RuntimeError): ...

class NativeCombatPool:
    def __init__(
        self,
        catalog: object = ...,
        *,
        num_envs: int = ...,
        workers: int = ...,
        partition: str = ...,
        autoreset: str = ...,
        record_replay: bool = ...,
    ) -> None: ...
    def observation_spec(self) -> dict[str, Any]: ...
    def catalog_descriptor(self) -> dict[str, Any]: ...
    def reset(
        self,
        indices: object,
        *,
        seed: object = ...,
        scenario_ids: object = ...,
        return_all: bool = ...,
    ) -> dict[str, Any]: ...
    def step_batch(self, actions: object, tokens: object) -> dict[str, Any]: ...
    def _reset_deliver[T](
        self,
        indices: object,
        package: Callable[[dict[str, Any]], T],
        *,
        seed: object = ...,
        scenario_ids: object = ...,
        return_all: bool = ...,
    ) -> T: ...
    def _step_deliver[T](
        self, actions: object, tokens: object, package: Callable[[dict[str, Any]], T]
    ) -> T: ...
    def close(self) -> None: ...
    def reproduction(self, slot: int) -> dict[str, Any] | None: ...
    def diagnostics(self) -> dict[str, Any]: ...
    def take_completed_records(self) -> list[dict[str, Any]]: ...
    def mark_export_fault(self, message: str) -> None: ...

def build_info() -> dict[str, str]: ...
