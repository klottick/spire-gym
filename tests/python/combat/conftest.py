import pytest

from spire_lab.combat.content import SUPPORTED_CONTENT_ID
from spire_lab.combat.model import (
    CardInstance,
    CombatState,
    EnemyState,
    PlayerState,
    Piles,
    PublicKnowledge,
    RngState,
)


@pytest.fixture
def minimal_state():
    """Structural fixture only; it makes no claim about game initialization."""
    return CombatState(
        content_id=SUPPORTED_CONTENT_ID,
        phase="decision",
        turn=0,
        player=PlayerState(17, 20, 0, 3, 3, {}),
        enemies=[EnemyState("enemy-1", "cultist", 12, 12, 0, {}, None, ())],
        piles=Piles(
            hand=[CardInstance(1, "strike", False), CardInstance(2, "defend", False), CardInstance(3, "bash", False)],
            draw=[], discard=[], exhaust=[],
        ),
        card_ids_issued={1, 2, 3},
        rng=RngState(()),
        pending=[],
        public=PublicKnowledge(()),
        outcome=None,
        fault=None,
        hp_before_healing=None,
        hp_after_cleanup=None,
    )
