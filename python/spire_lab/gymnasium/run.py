"""Single-Run transport only; all gameplay and legal actions belong to Rust."""

from __future__ import annotations

from operator import index
from typing import TYPE_CHECKING, Any, Literal, cast

import gymnasium as gym
import numpy as np
from numpy.typing import NDArray

from spire_lab import _native

from ._common import OperationGuard
from .spaces import spaces_from_spec

if TYPE_CHECKING:
    from spire_lab._native import CombatPolicyPayload, RunResetPayload, RunStepPayload

Observation = dict[str, NDArray[np.int64] | NDArray[np.int8]]
RunProfile = Literal[
    "m9-ironclad-act3", "m10-ironclad-heart", "m10-ironclad-policy", "standard"
]


class RunEnv(gym.Env[Observation, int]):
    """Native Run profile; callers develop their own strategies externally.

    Actions index the current legal mask, not stable game identities. The facade
    holds the matching native token; raw Gym integers alone cannot prove freshness.
    """

    def __init__(
        self,
        character: str,
        ascension: int,
        *,
        profile: RunProfile = "m9-ironclad-act3",
        partition: str = "train",
        boss_history: str | None = None,
        creation: dict[str, Any] | None = None,
        capacities: dict[str, int] | None = None,
        record_replay: bool = False,
    ) -> None:
        self._attach(
            _native.NativeRunSession(
                character,
                ascension,
                profile=profile,
                partition=partition,
                boss_history=boss_history,
                creation=creation,
                capacities=capacities,
                record_replay=record_replay,
            )
        )

    def _attach(self, session: _native.NativeRunSession) -> None:
        self.metadata = {"render_modes": []}
        self.render_mode = None
        self._guard = OperationGuard()
        self._session = session
        observations, actions = spaces_from_spec(self.observation_spec())
        # Gym's composite-space generics do not express its numeric Dict values.
        # Native descriptor/contains tests enforce these concrete boundary types.
        self.observation_space = cast(gym.spaces.Space[Observation], observations)
        self.action_space = cast(gym.spaces.Space[int], actions)
        self._token = (0, 0)

    def observation_spec(self) -> dict[str, Any]:
        """Owned native descriptor; Python defines no feature schema."""
        return self._session.observation_spec()

    def environment_spec(self) -> dict[str, Any]:
        """Public profile/rules/content/reward identities without a replay record."""
        with self._guard:
            return self._session.environment_spec()

    def combat_observation_spec(self) -> dict[str, Any]:
        """Reusable native combat-policy view; old Run schema stays unchanged."""
        with self._guard:
            return self._session.combat_observation_spec()

    def combat_view(self) -> CombatPolicyPayload:
        """Owned current combat features and visit/routing/exit metadata.

        Dispatch uses the original Run action index. Outside combat the view's
        action mask is empty. Snapshot resources on a settled-count change;
        last_exit is sticky while resource arrays always describe current state.
        """
        with self._guard:
            return self._session.combat_view()

    def _package_reset(
        self, raw: RunResetPayload
    ) -> tuple[Observation, dict[str, Any]]:
        result = (raw["obs"], dict(raw["info"]))
        super().reset(seed=raw["effective_seed"])
        self._token = raw["token"]
        return result

    def reset(
        self, *, seed: int | None = None, options: dict[str, Any] | None = None
    ) -> tuple[Observation, dict[str, Any]]:
        with self._guard:
            if options is not None and (not isinstance(options, dict) or options):
                raise _native.RunInputError(
                    "Run reset has no resource/scenario options"
                )
            return self._session._reset_deliver(self._package_reset, seed=seed)

    def _package_step(
        self, raw: RunStepPayload
    ) -> tuple[Observation, float, bool, bool, dict[str, Any]]:
        result = (
            raw["obs"],
            float(raw["reward"]),
            raw["terminated"],
            raw["truncated"],
            dict(raw["info"]),
        )
        self._token = raw["token"]
        return result

    def step(
        self, action: int
    ) -> tuple[Observation, float, bool, bool, dict[str, Any]]:
        with self._guard:
            if isinstance(action, (bool, np.bool_)) or not isinstance(
                action, (int, np.integer)
            ):
                raise _native.RunInputError(
                    "action must be an integer scalar, not bool"
                )
            return self._session._step_deliver(
                index(action), self._token, self._package_step
            )

    def record(self) -> bytes | None:
        """Privileged semantic record: never observation or ordinary info."""
        with self._guard:
            return self._session.record()

    def reproduction(self):
        """Privileged retained fault/record metadata, including after close."""
        with self._guard:
            return self._session.reproduction()

    def decision(self) -> tuple[Observation, dict[str, Any]]:
        """Owned current boundary; a forked or replayed env starts here."""
        with self._guard:
            raw = self._session.decision()
            self._token = raw["token"]
            return raw["obs"], dict(raw["info"])

    # Privileged exact-world tools (M34c). Not part of the Gymnasium API: reach
    # them through ``env.unwrapped``. A checkpoint replays the same hidden future,
    # so "step, look, restore" reveals it: an oracle, never planning input.

    def exact_checkpoint(self) -> _native.ExactCheckpoint:
        """Opaque in-memory exact copy of the current episode (oracle)."""
        with self._guard:
            return self._session.exact_checkpoint()

    def exact_restore(
        self, checkpoint: _native.ExactCheckpoint
    ) -> tuple[Observation, dict[str, Any]]:
        """Abandon this episode and continue at ``checkpoint``; old actions' tokens go stale."""
        with self._guard:
            raw = self._session.exact_restore(checkpoint)
            self._token = raw["token"]
            return raw["obs"], dict(raw["info"])

    def exact_fork(self, checkpoint: _native.ExactCheckpoint) -> RunEnv:
        """A new independent env at ``checkpoint``; read its boundary with ``decision()``."""
        return RunEnv._from_session(_native.NativeRunSession.exact_fork(checkpoint))

    @classmethod
    def from_record(
        cls,
        record: bytes,
        steps: int,
        *,
        capacities: dict[str, int] | None = None,
    ) -> RunEnv:
        """A live env after the first ``steps`` actions of ``record`` (checked as replay)."""
        return cls._from_session(
            _native.NativeRunSession.from_record(record, steps, capacities)
        )

    @classmethod
    def _from_session(cls, session: _native.NativeRunSession) -> RunEnv:
        env = cls.__new__(cls)
        env._attach(session)
        env._token = session.decision()["token"]
        return env

    def close(self) -> None:
        with self._guard:
            self._session.close()

    def render(self) -> None:
        return None
