"""Authored whole-combat checks, not original-runtime captures or seed parity."""

from copy import deepcopy
from dataclasses import replace

import pytest

from spire_lab.combat import engine
from spire_lab.combat.model import (
    CardInstance, EndTurn, EnemySetup, Piles, PlayCard, PlayerState, PowerState,
    RngState, Transition, validate_runtime_state,
)
from spire_lab.combat.rng import RNG_CONTRACT_ID, StreamId, make_stream
from spire_lab.combat.snapshot import compare, restore, snapshot
from spire_lab.combat.view import legal_actions, public_view


VICTORY_ACTIONS = (
    PlayCard(1, "enemy-1"), PlayCard(2, "enemy-1"), EndTurn(),
    PlayCard(1, "enemy-1"), PlayCard(2, "enemy-1"),
)


def combat(scenario_factory, enemy_id, *, death=False):
    hp, first = (48, "incantation") if enemy_id == "cultist" else (40, "chomp")
    return engine.initial_combat(scenario_factory(
        encounter_id=enemy_id,
        player=(PlayerState(14, 20, 0, 3, 3, 1, {}) if death
                else PlayerState(40, 50, 0, 3, 3, 3, {})),
        enemies=(EnemySetup("enemy-1", enemy_id, hp, hp, 0, {}, first, (first,)),),
        piles=Piles(
            [CardInstance(1, "defend", False)] if death else [
                CardInstance(1, "bash", True), CardInstance(2, "strike", True),
                CardInstance(3, "defend", True),
            ], [], [], [],
        ),
    ))


def assert_victory(state, hp_before, hp_after):
    assert state.phase == "terminal" and state.fault is None
    assert state.outcome.kind == "victory"
    assert (state.hp_before_healing, state.hp_after_cleanup) == (hp_before, hp_after)
    assert state.player.hp == hp_after and state.enemies[0].hp == 0
    assert not state.pending and state.current is None
    assert legal_actions(state) == ()
    view = public_view(state)
    assert (view.outcome, view.hp_before_healing, view.hp_after_cleanup) == (
        "victory", hp_before, hp_after,
    )


def test_two_turn_cultist_victory(scenario_factory):
    # Catches wrong Bash/Vulnerable order, Ritual timing, recycle order or healing.
    state = combat(scenario_factory, "cultist")
    for action in VICTORY_ACTIONS[:2]:
        assert engine.step(state, action).status == "decision"
    assert state.enemies[0].hp == 25
    assert engine.step(state, EndTurn()).status == "decision"
    enemy = state.enemies[0]
    assert state.player.hp == 40
    assert enemy.powers["ritual"] == PowerState(3, skip_first=False)
    assert (enemy.intent_id, enemy.intent_damage) == ("attack", 6)
    assert [c.instance_id for c in state.piles.hand] == [1, 3, 2]
    assert (state.rng.shuffle.word_index, state.rng.enemy_ai.word_index) == (2, 0)
    assert engine.step(state, VICTORY_ACTIONS[3]).status == "decision"
    assert state.enemies[0].hp == 10
    assert engine.step(state, VICTORY_ACTIONS[4]).status == "terminal"
    assert_victory(state, 40, 46)


def test_two_turn_jaw_worm_victory(scenario_factory):
    # Catches executing the newly rolled move instead of the published Chomp.
    state = combat(scenario_factory, "jaw_worm")
    for action in VICTORY_ACTIONS[:2]:
        assert engine.step(state, action).status == "decision"
    assert state.enemies[0].hp == 17
    assert engine.step(state, EndTurn()).status == "decision"
    enemy = state.enemies[0]
    assert state.player.hp == 29
    assert (enemy.intent_id, enemy.intent_damage) == ("thrash", 7)
    assert enemy.move_history == ("chomp", "thrash")
    assert [c.instance_id for c in state.piles.hand] == [1, 3, 2]
    assert (state.rng.shuffle.word_index, state.rng.enemy_ai.word_index) == (2, 1)
    assert engine.step(state, VICTORY_ACTIONS[3]).status == "decision"
    assert enemy.hp == 2
    assert engine.step(state, VICTORY_ACTIONS[4]).status == "terminal"
    assert_victory(state, 29, 35)


def assert_death(state):
    assert state.phase == "terminal" and state.fault is None
    assert state.outcome.kind == "death"
    assert state.player.hp == state.hp_before_healing == state.hp_after_cleanup == 0
    assert not state.pending and state.current is None
    assert legal_actions(state) == ()
    view = public_view(state)
    assert (view.outcome, view.hp_before_healing, view.hp_after_cleanup) == ("death", 0, 0)


def test_cultist_death_without_victory_heal(scenario_factory):
    # Catches off-by-one Ritual growth and accidental Burning Blood on death.
    state = combat(scenario_factory, "cultist", death=True)
    assert state.player.hp == 14
    for hp, status in [(14, "decision"), (8, "decision"), (0, "terminal")]:
        assert engine.step(state, EndTurn()).status == status
        assert state.player.hp == hp
        assert state.rng.enemy_ai.word_index == 0
    assert_death(state)


def test_jaw_worm_lethal_stops_future_rng(scenario_factory):
    # An exhausted AI stream makes an erroneous post-lethal roll observable.
    state = combat(scenario_factory, "jaw_worm", death=True)
    state.player.hp = 10
    state.rng = replace(state.rng, enemy_ai=replace(state.rng.enemy_ai, word_index=2**64))
    before_rng = state.rng
    assert engine.step(state, EndTurn()).status == "terminal"
    assert_death(state)
    assert state.rng == before_rng
    assert state.enemies[0].move_history == ("chomp",)


def assert_same_checkpoint(left, right):
    assert compare(left, right) is None
    assert public_view(left) == public_view(right)
    assert left.outcome == right.outcome
    assert (left.rng.shuffle.word_index, left.rng.enemy_ai.word_index) == (
        right.rng.shuffle.word_index, right.rng.enemy_ai.word_index,
    )
    assert legal_actions(left) == legal_actions(right)


@pytest.mark.parametrize("enemy_id,hp_before,hp_after", [
    ("cultist", 40, 46), ("jaw_worm", 29, 35),
])
@pytest.mark.parametrize("checkpoint", [1, 3], ids=["after-bash", "after-end-turn"])
def test_ready_snapshot_replay_matches_all_components(
    scenario_factory, enemy_id, hp_before, hp_after, checkpoint,
):
    # Catches dropped power flags/history/pile order/cursors during restoration.
    state = combat(scenario_factory, enemy_id)
    for action in VICTORY_ACTIONS[:checkpoint]:
        assert engine.step(state, action).status == "decision"
    saved = snapshot(state)
    saved_baseline = deepcopy(saved.state)
    replay = restore(saved)
    assert_same_checkpoint(state, replay)
    for action in VICTORY_ACTIONS[checkpoint:]:
        direct_result = engine.step(state, action)
        replay_result = engine.step(replay, action)
        assert direct_result.status == replay_result.status
        assert_same_checkpoint(state, replay)
    assert_victory(state, hp_before, hp_after)
    assert compare(saved.state, saved_baseline) is None


def owned_cards(state):
    cards = state.piles.hand + state.piles.draw + state.piles.discard + state.piles.exhaust
    return cards + ([state.in_play] if state.in_play is not None else [])


@pytest.mark.parametrize("enemy_id", ["cultist", "jaw_worm"])
@pytest.mark.parametrize("death", [False, True], ids=["victory", "death"])
def test_supported_paths_never_lose_or_duplicate_cards(scenario_factory, monkeypatch, enemy_id, death):
    # Catches loss/duplication even while a played lethal card remains diagnostic.
    state = combat(scenario_factory, enemy_id, death=death)
    expected = {c.instance_id: c for c in owned_cards(state)}
    saw_in_play = []

    def check(owned):
        cards = owned_cards(owned)
        assert len(cards) == len(expected)
        assert {c.instance_id: c for c in cards} == expected
        assert owned.card_ids_issued == set(expected)
        assert all(c is expected[c.instance_id] for c in cards)
        if owned.in_play is not None:
            saw_in_play.append(owned.in_play.instance_id)

    dispatch = engine._dispatch_one

    def checked_dispatch(owned):
        check(owned)
        dispatch(owned)
        check(owned)

    monkeypatch.setattr(engine, "_dispatch_one", checked_dispatch)
    check(state)
    actions = (EndTurn(),) * (3 if enemy_id == "cultist" else 2) if death else VICTORY_ACTIONS
    for action in actions:
        assert engine.step(state, action).status in {"decision", "terminal"}
        check(state)
    assert state.phase == "terminal" and state.fault is None
    assert state.outcome.kind == ("death" if death else "victory")
    if not death:
        assert set(saw_in_play) == {1, 2}


@pytest.mark.parametrize("enemy_id", ["cultist", "jaw_worm"])
@pytest.mark.parametrize("episode", range(20))
def test_seed_variants_and_legal_walks(scenario_factory, enemy_id, episode):
    # Project episode identities only: bounded deterministic self-tests, not a
    # statistical chance proof or original-game seeds. Replay checks every step.
    scenario = scenario_factory()
    identity = f"combat-vertical-{episode}"
    scenario.rng = RngState(
        RNG_CONTRACT_ID,
        make_stream(bytes(32), "test", identity, StreamId("combat.shuffle", ("combat-plan-1",))),
        make_stream(bytes(32), "test", identity,
                    StreamId("combat.enemy.ai", ("combat-plan-1", "enemy-scope-1"))),
        "enemy-1",
    )
    hp, first = (48, "incantation") if enemy_id == "cultist" else (40, "chomp")
    scenario.encounter_id = enemy_id
    scenario.enemies = (EnemySetup("enemy-1", enemy_id, hp, hp, 0, {}, first, (first,)),)
    cards = [CardInstance(i + 1, card_id, bool((i + episode) % 2))
             for i, card_id in enumerate(("bash", "strike", "defend") * 3)]
    scenario.piles = Piles(cards[:3], cards[3:], [], [])
    state = engine.initial_combat(scenario)
    replay = restore(snapshot(state))
    expected_cards = {c.instance_id: c for c in cards}
    assert_same_checkpoint(state, replay)
    for index in range(200):
        choices = legal_actions(state)
        assert choices
        # Select by the existing public action order; no new random domain/policy.
        action = choices[(episode + index) % len(choices)]
        result = engine.step(state, action)
        replay_result = engine.step(replay, action)
        assert type(result) is Transition and result.state is state
        assert result.status in {"decision", "terminal", "fault"}
        assert result.status == replay_result.status
        assert state.fault is None and replay.fault is None
        validate_runtime_state(state)
        assert_same_checkpoint(state, replay)
        for owned in (state, replay):
            current_cards = owned_cards(owned)
            assert len(current_cards) == len(expected_cards)
            assert {c.instance_id: c for c in current_cards} == expected_cards
            assert owned.card_ids_issued == set(expected_cards)
        if result.status == "terminal":
            assert state.outcome.kind in {"victory", "death"}
            break
        assert state.phase == "decision" and state.outcome is None
    else:
        pytest.fail("legal walk did not terminate within its 200-action budget")
