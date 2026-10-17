from dataclasses import dataclass
from types import MappingProxyType


SUPPORTED_CONTENT_ID: str = "spire-lab-combat-reference-v2"
SUPPORTED_CARD_IDS: frozenset[str] = frozenset({"strike", "defend", "bash"})
SUPPORTED_ENCOUNTER_IDS: frozenset[str] = frozenset({"cultist", "jaw_worm"})
SUPPORTED_RELIC_IDS: frozenset[str] = frozenset({"burning_blood"})

ENEMY_DAMAGE = MappingProxyType({
    "cultist": (("attack", 6),),
    "jaw_worm": (("chomp", 11), ("thrash", 7)),
})
ENEMY_FIRST_MOVE = MappingProxyType({"cultist": "incantation", "jaw_worm": "chomp"})
ENEMY_HP_RANGE = MappingProxyType({"cultist": (48, 54), "jaw_worm": (40, 44)})


@dataclass(frozen=True)
class CardStats:
    cost: int
    damage: int
    block: int
    vulnerable: int


CARD_STATS = MappingProxyType({
    ("strike", False): CardStats(1, 6, 0, 0),
    ("strike", True): CardStats(1, 9, 0, 0),
    ("defend", False): CardStats(1, 0, 5, 0),
    ("defend", True): CardStats(1, 0, 8, 0),
    ("bash", False): CardStats(2, 8, 0, 2),
    ("bash", True): CardStats(2, 10, 0, 3),
})
