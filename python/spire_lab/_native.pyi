"""Transport signatures for the compiled extension; no Python gameplay."""

from collections.abc import Callable, Sequence
from typing import Any, Literal, TypedDict

import numpy as np
from numpy.typing import NDArray

type RunObservation = dict[str, NDArray[np.int64] | NDArray[np.int8]]
type RunProfile = Literal[
    "m9-ironclad-act3", "m10-ironclad-heart", "m10-ironclad-policy", "standard"
]

class RunInfo(TypedDict, total=False):
    schema_id: str
    terminal_reason: str | None
    creation: dict[str, Any]
    reward_events: list[dict[str, Any]]
    episode_return: int
    heart_victory: bool

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
class RunBudgetError(RunStateError): ...
class RunFaultError(RuntimeError): ...
class RunUnsupportedBoundaryError(NotImplementedError): ...

class ExactCheckpoint:
    """Privileged opaque exact state: unpicklable, identity equality only."""

class NativeRunSession:
    def __init__(
        self,
        character: str,
        ascension: int,
        *,
        profile: RunProfile = ...,
        partition: str = ...,
        boss_history: str | None = ...,
        creation: dict[str, Any] | None = ...,
        capacities: dict[str, int] | None = ...,
        record_replay: bool = ...,
    ) -> None: ...
    def observation_spec(self) -> dict[str, Any]: ...
    def environment_spec(self) -> dict[str, Any]: ...
    def combat_observation_spec(self) -> dict[str, Any]: ...
    def combat_view(self) -> CombatPolicyPayload: ...
    def public_search(
        self,
        research_seed: int,
        *,
        max_live_branches: int,
        max_branches: int,
        max_steps_per_branch: int,
        max_total_steps: int,
    ) -> NativePublicSearch: ...
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
    def decision(self) -> RunResetPayload: ...
    def exact_checkpoint(self) -> ExactCheckpoint: ...
    def exact_restore(self, checkpoint: ExactCheckpoint) -> RunResetPayload: ...
    @staticmethod
    def exact_fork(checkpoint: ExactCheckpoint) -> NativeRunSession: ...
    @staticmethod
    def from_record(
        record: bytes, steps: int, capacities: dict[str, int] | None = ...
    ) -> NativeRunSession: ...

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
def card_metadata() -> dict[str, Any]: ...

class CombatPolicyPayload(TypedDict):
    obs: RunObservation
    token: tuple[int, int]
    info: dict[str, Any]

class CombatTrainingResetPayload(CombatPolicyPayload):
    effective_seed: int

class CombatTrainingStepPayload(CombatPolicyPayload):
    reward: int
    terminated: bool
    truncated: bool

class NativeIroncladCombatSession:
    def __init__(self, catalog_json: str | bytes, *, partition: str = ...) -> None: ...
    def observation_spec(self) -> dict[str, Any]: ...
    def environment_spec(self) -> dict[str, Any]: ...
    def catalog_descriptor(self) -> dict[str, Any]: ...
    def reset(
        self, seed: int | None = ..., entry_id: str | None = ...
    ) -> CombatTrainingResetPayload: ...
    def step(self, index: int, token: tuple[int, int]) -> CombatTrainingStepPayload: ...
    def decision(self) -> CombatPolicyPayload: ...
    def _reset_deliver[T](
        self,
        package: Callable[[CombatTrainingResetPayload], T],
        *,
        seed: int | None = ...,
        entry_id: str | None = ...,
    ) -> T: ...
    def _step_deliver[T](
        self,
        index: int,
        token: tuple[int, int],
        package: Callable[[CombatTrainingStepPayload], T],
    ) -> T: ...
    def diagnostic(self) -> dict[str, Any]: ...
    def close(self) -> None: ...

class NativeStandardCombatSession:
    def __init__(self, catalog_json: str | bytes, *, records: Sequence[bytes] | None = ..., partition: str = ...) -> None: ...
    def observation_spec(self) -> dict[str, Any]: ...
    def environment_spec(self) -> dict[str, Any]: ...
    def catalog_descriptor(self) -> dict[str, Any]: ...
    def reset(
        self, seed: int | None = ..., entry_id: str | None = ...
    ) -> CombatTrainingResetPayload: ...
    def step(self, index: int, token: tuple[int, int]) -> CombatTrainingStepPayload: ...
    def decision(self) -> CombatPolicyPayload: ...
    def public_search(
        self,
        research_seed: int,
        *,
        max_live_branches: int,
        max_branches: int,
        max_steps_per_branch: int,
        max_total_steps: int,
    ) -> NativePublicSearch: ...
    def _reset_deliver[T](
        self,
        package: Callable[[CombatTrainingResetPayload], T],
        *,
        seed: int | None = ...,
        entry_id: str | None = ...,
    ) -> T: ...
    def _step_deliver[T](
        self,
        index: int,
        token: tuple[int, int],
        package: Callable[[CombatTrainingStepPayload], T],
    ) -> T: ...
    def diagnostic(self) -> dict[str, Any]: ...
    def close(self) -> None: ...

class NativePublicSearch:
    def branch(self, sample: int | None = ...) -> int: ...
    def decision(self, branch: int) -> dict[str, Any]: ...
    def step(
        self,
        branch: int,
        token: tuple[int, int],
        action: int,
        view: Literal["full", "mask"] = ...,
    ) -> dict[str, Any]: ...
    def fork(self, branch: int) -> int: ...
    def actions(self, branch: int) -> list[int]: ...
    def public_search(
        self,
        branch: int,
        research_seed: int,
        *,
        max_live_branches: int,
        max_branches: int,
        max_steps_per_branch: int,
        max_total_steps: int,
    ) -> NativePublicSearch: ...
    def counters(self) -> dict[str, int]: ...
    def spec(self) -> dict[str, Any]: ...
    def close(self, branch: int | None = ...) -> None: ...

class NativeIroncladCombatBatch:
    def __init__(
        self, num_envs: int, catalog_json: str | bytes, *, partition: str = ...
    ) -> None: ...
    def observation_spec(self) -> dict[str, Any]: ...
    def environment_spec(self) -> dict[str, Any]: ...
    def catalog_descriptor(self) -> dict[str, Any]: ...
    def reset(
        self,
        *,
        seeds: list[int | None] | None = ...,
        entry_ids: list[str | None] | None = ...,
        reset_mask: object = ...,
    ) -> dict[str, Any]: ...
    def step(self, actions: object, tokens: object) -> dict[str, Any]: ...
    def _reset_deliver[T](
        self,
        package: Callable[[dict[str, Any]], T],
        *,
        seeds: list[int | None] | None = ...,
        entry_ids: list[str | None] | None = ...,
        reset_mask: object = ...,
    ) -> T: ...
    def _step_deliver[T](
        self, actions: object, tokens: object, package: Callable[[dict[str, Any]], T]
    ) -> T: ...
    def diagnostic(self) -> dict[str, Any]: ...
    def close(self) -> None: ...
