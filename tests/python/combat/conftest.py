import pytest

from spire_lab.combat.model import (
    CardInstance,
    EnemySetup,
    PlayerState,
    Piles,
    RngState,
    Scenario,
)
from spire_lab.combat.engine import initial_combat
from spire_lab.combat.rng import RNG_CONTRACT_ID, StreamId, make_stream


@pytest.fixture
def scenario_factory():
    def factory(**overrides):
        scenario = Scenario(
            encounter_id="cultist",
            player=PlayerState(40, 50, 0, 3, 3, 5, {}),
            enemies=(EnemySetup("enemy-1", "cultist", 48, 48, 0, {}, "incantation", ("incantation",)),),
            piles=Piles(
                hand=[CardInstance(1, "strike", False), CardInstance(2, "defend", False), CardInstance(3, "bash", False)],
                draw=[], discard=[], exhaust=[],
            ),
            rng=RngState(
                RNG_CONTRACT_ID,
                make_stream(bytes(32), "test", "combat-behavior-plan", StreamId("combat.shuffle", ("combat-plan-1",))),
                make_stream(bytes(32), "test", "combat-behavior-plan", StreamId("combat.enemy.ai", ("combat-plan-1", "enemy-scope-1"))),
                "enemy-1",
            ),
        )
        for name, value in overrides.items():
            if name not in Scenario.__dataclass_fields__:
                raise TypeError(f"unknown Scenario field: {name}")
            setattr(scenario, name, value)
        return scenario
    return factory


@pytest.fixture
def minimal_state(scenario_factory):
    return initial_combat(scenario_factory())
