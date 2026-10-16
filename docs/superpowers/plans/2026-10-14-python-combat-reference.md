# Python Combat Reference Implementation Plan

> **Approved RNG policy amendment (2026-10-16):**
> [RNG-001](../../intentional-deviations.md#rng-001--decorrelated-reproducible-project-randomness)
> supersedes exact original RNG/seed/stream parity requirements in this historical
> plan. Intended probabilities and gameplay dependencies remain required. A revised
> project-RNG design/plan is still needed; this note does not unlock Tasks3–5.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an independently authored, readable Python reference for one supported Ironclad A0 combat without claiming support or validation absent reviewed original-game evidence.

**Architecture:** One owned combat state and explicit ordered scheduler cover the narrow approved combat boundary. Public view is separate from internal diagnostics; unsupported behavior and faults are explicit. Only structural state/fault/snapshot work is presently actionable; mechanics, exact RNG/arithmetic and externally grounded combat tests remain blocked pending reviewed evidence and a concrete plan revision.

**Tech Stack:** Python 3.12 standard library; pytest 9.1.1; dataclasses/enums; no new dependencies or native/tensor imports in the rules path.

**Spec:** `docs/superpowers/specs/2026-10-14-python-combat-reference-design.md`

## Readiness and authority

**The complete combat milestone is NOT EXECUTION READY.** The brief reports no accepted mechanics bodies, original RNG vectors, or full-combat captures. Frozen/unaccepted inspection and capture work is not reopened. Tasks 1–2 below are precise structural tasks, still requiring separate implementation-start approval. Tasks 3–5 are gated scope, not executable instructions: after separate applicable evidence-access approval and evidence review, revise this plan with concrete tests and assertions before authorizing implementation. Plan approval does not authorize original-game/private-artifact access. No Rust work is included.

## Global Constraints

- Support only Ironclad A0, base/upgraded Strike, Defend and Bash, Cultist or Jaw Worm, Burning Blood, one combat entry through terminal result.
- Author ordinary Python 3.12 rules independently; no copied/translated game code, training, optimization, full-game scope, or native/tensor imports on the rules path.
- Preserve ordered piles/effects, stable never-reused card-instance IDs, separate public/internal/evidence views, and explicit unsupported behavior.
- Invalid actions must be pre-mutation. Unsupported, invariant and guard faults retain diagnostic partial state and fault it; faults are not losses and cannot resume/clear. Recovery is valid nonfaulted snapshot restore or a new combat.
- Per external `step`, resolve no more than 100,000 effects; count at resolution, including a failing effect. Preserve the faulted partial state at exhaustion.
- Report HP immediately before combat-end healing and after terminal cleanup.
- No guessed facts, silent no-op, fabricated loss, or validation claim from invariants, self-authored cases, or Python/Rust parity.
- New combat tests must not import native/readiness tests or require GPU. Do not alter readiness, trace, old contracts, GPU setup, pyproject, README, prior plans, or SDD files.
- Tasks 1–2 are structural only; Tasks 3–5 remain blocked. Any future commit requires separate Git-write consent; this plan grants none.

## Review Focus

1. **Resolution guard preserves partial state and blocks further actions:** Task 4 `test_effect_guard_faults_after_100000_resolved_effects` and `test_faulted_state_rejects_action_after_guard`.
2. **Invalid action does not mutate state; actual transition faults retain diagnostic changes:** Task 4 `test_invalid_action_is_atomic` and `test_mid_resolution_invariant_fault_retains_partial_state`.
3. **Hidden RNG/unrevealed order does not leak:** Task 5 `test_public_view_ignores_hidden_rng_and_unrevealed_order`.
4. **Numeric/RNG/order errors are not masked by self-consistency:** Task 3 `test_evidenced_rng_vectors_and_consumption_order`, plus Task 4 reviewed vertical traces.
5. **Terminal precedence and both healing observations match original behavior:** Task 4 `test_terminal_precedence_matches_reviewed_trace` and `test_terminal_hp_reports_pre_heal_and_post_cleanup_values`.

## File map and shared type ownership

Create only within `python/spire_lab/combat/` and `tests/python/combat/`:

- `model.py` owns all state/action/error/result types and `validate_state(state: CombatState) -> None`. `Scenario` is an explicit semantic entry-input value; frozen `RngState(slots: tuple[int | str | bool, ...])` is an opaque, provisional in-memory slot (no asserted game algorithm or game-word shape); `CombatState` owns ordinary Python values only.
- `content.py` owns immutable identifiers for the approved content set; no executable rules.
- `snapshot.py` owns `CombatSnapshot`, `Divergence`, clone/restore/compare. Diagnostic snapshot/compare accepts faulted partial states even when structurally invalid; restore accepts only valid nonfaulted state.
- `engine.py` owns `initial_combat`, `legal_actions`, `step`, and scheduler/fault interactions.
- `rng.py` and `rules.py` own only evidence-backed RNG and numeric behavior.
- `view.py` owns `PublicView` and `public_view`.
- `tests/python/combat/conftest.py` owns the self-authored structural `minimal_state` fixture. Focused tests live under `tests/python/combat/`; reviewed cases/fixture wrappers live under `tests/python/fixtures/combat/`. A case wrapper carries provenance outside `CombatState`.

Structural `model.py` field/type inventory (types in parentheses): frozen `CardInstance(instance_id: int, card_id: str, upgraded: bool)`; `Piles(hand: list[CardInstance], draw: list[CardInstance], discard: list[CardInstance], exhaust: list[CardInstance])`; `PlayerState(hp: int, max_hp: int, block: int, energy: int, max_energy: int, powers: dict[str, int])`; `EnemyState(instance_id: str, enemy_id: str, hp: int, max_hp: int, block: int, powers: dict[str, int], intent_id: str | None, move_history: tuple[str, ...])`; `PendingEffect(kind: str, source_id: int | str | None, target_id: int | str | None, payload: tuple[tuple[str, int | str | bool], ...])`; `PublicKnowledge(revealed_order: tuple[str, ...])`; frozen opaque `RngState(slots: tuple[int | str | bool, ...])`; `Scenario(encounter_id: str, player: PlayerState, enemies: tuple[EnemyState, ...], piles: Piles, rng: RngState)`; `CombatState(content_id: str, phase: str, turn: int, player: PlayerState, enemies: list[EnemyState], piles: Piles, card_ids_issued: set[int], rng: RngState, pending: list[PendingEffect], public: PublicKnowledge, outcome: TerminalOutcome | None, fault: FaultRecord | None, hp_before_healing: int | None, hp_after_cleanup: int | None)`. This is a structural minimum only; Task 4 may extend `model.py` when reviewed mechanics require evidenced state. It does not guarantee a complete domain model.

Actions are `Action = PlayCard | EndTurn`; `PlayCard(card_instance_id: int, target_id: int | str | None)` and `EndTurn()`; target-required rules are not selected here. `TerminalOutcome(kind: str)` uses only `victory`/`death` as terminal categories, not to encode faults. `FaultKind` is an enum with values `UNSUPPORTED="unsupported"`, `INVARIANT="invariant"`, `EFFECT_GUARD="effect_guard"`; `FaultRecord(kind: FaultKind, message: str, resolved_effects: int)`; exceptions `InvalidActionError`, `UnsupportedMechanicError`, `InvariantError`, `ResolutionGuardError`, `FaultedStateError`, `SnapshotCompatibilityError`. `validate_state(state: CombatState) -> None` in `model.py` checks only that card instance IDs are unique across piles and included in `card_ids_issued`, and that owned values/payloads are the declared ordinary types; it raises `InvariantError` on failure. It does not assert game ranges/rules. `Transition(state: CombatState, status: str)` has status `decision | terminal | fault`. This inventory fixes structural ownership, not game facts or supported action availability.

## Task 1: Typed state and content boundary (structurally actionable)

**Files:** Create `python/spire_lab/combat/__init__.py`, `model.py`, `content.py`, `tests/python/combat/conftest.py`, `tests/python/combat/test_model.py`.

**Interfaces:** Use the exact field inventory above. `content.py` exports `SUPPORTED_CONTENT_ID: str` equal to the project-local version identifier `spire-lab-combat-reference-v1` (an internal project label, not game/version evidence), `SUPPORTED_CARD_IDS: frozenset[str]` containing exactly `strike`, `defend`, `bash`; `SUPPORTED_ENCOUNTER_IDS: frozenset[str]` containing exactly `cultist`, `jaw_worm`; `SUPPORTED_RELIC_IDS: frozenset[str]` containing exactly `burning_blood`. `RngState` is frozen and stores exactly `slots: tuple[int | str | bool, ...]`; this opaque in-memory representation is provisional and asserts nothing about original RNG.

**Test-only fixture:** `tests/python/combat/conftest.py` defines pytest fixture `minimal_state`, self-authored and explicitly not a valid game-entry claim: content ID above, phase `decision`, turn 0; player `(hp=17,max_hp=20,block=0,energy=3,max_energy=3,powers={})`; one enemy `(instance_id="enemy-1",enemy_id="cultist",hp=12,max_hp=12,block=0,powers={},intent_id=None,move_history=())`; card IDs 1/2/3 with `strike/defend/bash` in hand, other piles empty; issued IDs `{1,2,3}`; `RngState(())`; empty pending/public; no outcome/fault/HP observations. The arbitrary values are structural fixture values only, not mechanics or initialization.

- [ ] **Step 1: Write failing tests** in `test_model.py`:

```python
from spire_lab.combat.content import (
    SUPPORTED_CARD_IDS, SUPPORTED_CONTENT_ID, SUPPORTED_ENCOUNTER_IDS,
    SUPPORTED_RELIC_IDS,
)
import pytest
from spire_lab.combat.model import (
    CardInstance, EndTurn, FaultKind, FaultRecord, InvariantError,
    PendingEffect, Piles, PlayCard, RngState, TerminalOutcome, validate_state,
)

def test_minimal_state_owns_typed_values(minimal_state):
    assert isinstance(minimal_state.piles, Piles)
    assert [c.instance_id for c in minimal_state.piles.hand] == [1, 2, 3]
    assert minimal_state.player.powers == {}
    assert minimal_state.enemies[0].powers == {}
    assert minimal_state.content_id == SUPPORTED_CONTENT_ID
    validate_state(minimal_state)

def test_instance_ids_unique_across_all_piles(minimal_state):
    minimal_state.piles.draw.append(CardInstance(1, "strike", False))
    with pytest.raises(InvariantError):
        validate_state(minimal_state)

def test_piles_and_pending_keep_order(minimal_state):
    minimal_state.pending[:] = [PendingEffect("a", 1, "enemy-1", ()),
                                PendingEffect("b", "enemy-1", 2, ())]
    assert [effect.kind for effect in minimal_state.pending] == ["a", "b"]

def test_fault_is_not_terminal_outcome():
    assert FaultRecord(FaultKind.INVARIANT, "x", 0) != TerminalOutcome("death")

def test_action_union_is_semantic():
    assert isinstance(PlayCard(1, "enemy-1"), PlayCard)
    assert isinstance(EndTurn(), EndTurn)

def test_content_manifest_is_exact_scope():
    assert SUPPORTED_CONTENT_ID == "spire-lab-combat-reference-v1"
    assert SUPPORTED_CARD_IDS == {"strike", "defend", "bash"}
    assert SUPPORTED_ENCOUNTER_IDS == {"cultist", "jaw_worm"}
    assert SUPPORTED_RELIC_IDS == {"burning_blood"}

def test_model_rejects_foreign_mutable_objects(minimal_state):
    minimal_state.pending.append(PendingEffect("x", None, None, (("x", object()),)))
    with pytest.raises(InvariantError):
        validate_state(minimal_state)

def test_rng_slot_is_opaque_provisional_value():
    value = RngState((1, "x", True))
    assert value.slots == (1, "x", True)
```

The fixture values are structural only; this suite asserts no combat setup or game outcome. `validate_state` checks declared types, unique pile IDs and ID ownership, not game legality.
- [ ] **Step 2: Verify red** `.venv/bin/python -m pytest tests/python/combat/test_model.py -q`; expected collection/assertion failures because types/package are absent.
- [ ] **Step 3: Implement the listed datatypes and `validate_state(state: CombatState) -> None`** in the package files and `conftest.py`. `validate_state` verifies only declared dataclass/container/scalar types, unique IDs across all four piles, and that each pile card ID occurs in `card_ids_issued`; on any failure raise `InvariantError`. Restrict owned state/payloads to declared ordinary values; do not add rule behavior, generic schemas, plugins, or evidence validators.
- [ ] **Step 4: Verify green** with the same command; expected all eight tests pass without native/tensor imports.

## Task 2: Deep snapshots, compatibility and structural fault helpers (structurally actionable)

**Files:** Create `python/spire_lab/combat/snapshot.py`, `tests/python/combat/test_snapshot.py`, `tests/python/combat/test_fault_state.py`; modify `model.py` only to implement the exact inventory above.

**Interfaces:** `CombatSnapshot(content_id: str, schema_version: int, state: CombatState)`; `Divergence(component: str, left: object, right: object)` where component order is `content_id, phase, turn, player, enemies, piles, card_ids_issued, rng, pending, public, outcome, fault, hp_before_healing, hp_after_cleanup`; `snapshot(state: CombatState) -> CombatSnapshot`; `restore(value: CombatSnapshot, *, expected_content_id: str = SUPPORTED_CONTENT_ID) -> CombatState`; `compare(left: CombatState, right: CombatState) -> Divergence | None`; `mark_fault(state: CombatState, kind: FaultKind, message: str, resolved_effects: int = 0) -> None`; `require_steppable(state: CombatState) -> None`. `SnapshotCompatibilityError` is owned by `snapshot.py`. Schema version `1` is private in-memory compatibility only, not a wire format. Snapshot and compare deep-clone/inspect even faulted partial state without validating it. Restore checks snapshot content ID, state content ID, and schema version first (mismatch raises `SnapshotCompatibilityError`), then rejects a faulted snapshot with `FaultedStateError`, then runs `validate_state` (malformed/card-ownership-invalid state raises `InvariantError`); successful restore is a deep clone of valid nonfaulted state. `mark_fault` records only the first fault; later calls preserve that original record and all state fields. `require_steppable` checks only `fault is not None` and raises `FaultedStateError`; terminal-action policy belongs to Task 4. No `step`, action validation, transition, or engine stub belongs in this task.

- [ ] **Step 1: Write failing tests** in `test_snapshot.py` and `test_fault_state.py`; use the injected `minimal_state` fixture:

```python
import pytest
from spire_lab.combat.content import SUPPORTED_CONTENT_ID
from spire_lab.combat.model import (
    FaultKind, FaultRecord, FaultedStateError, InvariantError, PendingEffect,
    mark_fault, require_steppable,
)
from spire_lab.combat.snapshot import (
    CombatSnapshot, SnapshotCompatibilityError, compare, restore,
    snapshot,
)

def test_snapshot_is_deep_clone_and_mutation_does_not_alias(minimal_state):
    saved = snapshot(minimal_state)
    minimal_state.player.powers["self-test"] = 2
    minimal_state.piles.hand.clear()
    minimal_state.card_ids_issued.clear()
    minimal_state.enemies.clear()
    minimal_state.pending.append(PendingEffect("self-test", None, None, ()))
    assert saved.state.player.powers == {}
    assert [c.instance_id for c in saved.state.piles.hand] == [1, 2, 3]
    assert saved.state.card_ids_issued == {1, 2, 3}
    assert len(saved.state.enemies) == 1 and saved.state.pending == []

def test_restore_clone_does_not_alias_snapshot(minimal_state):
    saved = snapshot(minimal_state)
    restored = restore(saved)
    restored.piles.hand.pop()
    restored.player.powers["self-test"] = 1
    restored.enemies[0].powers["self-test"] = 1
    restored.card_ids_issued.clear()
    restored.pending.append(PendingEffect("self-test", None, None, ()))
    assert len(saved.state.piles.hand) == 3 and saved.state.player.powers == {}
    assert saved.state.enemies[0].powers == {}
    assert saved.state.card_ids_issued == {1, 2, 3} and saved.state.pending == []

def test_restore_rejects_content_or_schema_mismatch(minimal_state):
    saved = snapshot(minimal_state)
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot("other-content", 1, saved.state))
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(saved.content_id, 2, saved.state))
    wrong_state = snapshot(minimal_state).state
    wrong_state.content_id = "other-content"
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(saved.content_id, 1, wrong_state))

def test_compare_checks_content_then_declared_field_order(minimal_state):
    other = restore(snapshot(minimal_state))
    other.phase = "other"
    other.player.hp -= 1
    assert compare(minimal_state, other).component == "phase"
    other.content_id = "other-content"
    assert compare(minimal_state, other).component == "content_id"

def test_faulted_snapshot_is_diagnostic_but_cannot_be_restored(minimal_state):
    minimal_state.fault = FaultRecord(FaultKind.INVARIANT, "partial", 1)
    minimal_state.card_ids_issued.clear()  # deliberately invalid partial state
    saved = snapshot(minimal_state)
    assert saved.state.fault == minimal_state.fault
    assert saved.state.card_ids_issued == set()
    with pytest.raises(FaultedStateError):
        restore(saved)

def test_restore_rejects_invalid_card_ownership(minimal_state):
    saved = snapshot(minimal_state)
    saved.state.card_ids_issued.clear()
    with pytest.raises(InvariantError):
        restore(saved)

def test_mark_fault_preserves_partial_fields_and_first_record(minimal_state):
    mark_fault(minimal_state, FaultKind.UNSUPPORTED, "first", 3)
    partial = snapshot(minimal_state)
    mark_fault(minimal_state, FaultKind.INVARIANT, "second", 9)
    assert minimal_state.fault == FaultRecord(FaultKind.UNSUPPORTED, "first", 3)
    assert minimal_state.player == partial.state.player
    with pytest.raises(FaultedStateError):
        require_steppable(minimal_state)
```

Snapshot schema/content incompatibility uses `SnapshotCompatibilityError`; faulted and invalid states use their named exceptions.
- [ ] **Step 2: Verify red** `.venv/bin/python -m pytest tests/python/combat/test_snapshot.py tests/python/combat/test_fault_state.py -q`; expected missing-symbol failures.
- [ ] **Step 3: Implement deep clone/compatibility/ordered compare and fault helpers** in `snapshot.py` and `model.py`; use standard-library `copy.deepcopy` on the declared state graph, no pickle/serialization framework and no automatic rollback.
- [ ] **Step 4: Verify green** with the same command; expected all seven tests pass.

## Gated behavioral follow-on — NOT EXECUTION READY

Do not run Tasks 3–5 from this plan as written. They require reviewed evidence and a subsequent plan revision that supplies concrete test inputs and exact assertions. No placeholder oracle, guessed values, or “implement approved rules” task is authorized.

### Task 3: Evidence-backed RNG and exact arithmetic (blocked)

**Files:** `rng.py`, `rules.py`, `tests/python/combat/test_rng.py`, `test_rules.py`, reviewed fixtures under `tests/python/fixtures/combat/`.

**Interfaces after evidence review:** Starting from Task 1's provisional `RngState(slots: tuple[int | str | bool, ...])`, revise this plan if reviewed evidence requires a different representation; do not silently imply the provisional slots match a game word. Define `next_bounded(state: RngState, bound: int) -> tuple[int, RngState]` and `shuffle(items: Sequence[T], state: RngState) -> tuple[list[T], RngState]` only if those operations are established. Numeric helper names/signatures are selected from the evidenced operations; none are guessed now.

**Evidence prerequisite:** exact RNG algorithm/state representation, stream topology, bounded sampling, shuffle procedure, seed/state vectors, expected outputs and consumption order; arithmetic width, signedness, wrapping, truncation, rounding and floating-point semantics for supported operations. Preserve evidence class/identity/provenance with expected vectors. Unknown behavior stays explicitly unsupported.

- [ ] After reviewed evidence and approved plan revision, add concrete failing `test_evidenced_rng_vectors_and_consumption_order` assertions, including restored-state replay and exact output/state consumption; run that focused test and verify failure.
- [ ] Implement only evidenced behavior; run exact focused vectors and verify pass. No self-generated vector establishes correctness.

### Task 4: Complete supported combat engine (blocked)

**Files:** `engine.py`, extend `content.py`, `tests/python/combat/test_engine.py`, `test_vertical_combat.py`, reviewed fixture wrappers under `tests/python/fixtures/combat/`.

**Interfaces:** Task 4 owns `Scenario` (from model), `initial_combat(scenario: Scenario) -> CombatState`, `legal_actions(state: CombatState) -> tuple[Action, ...]`, `step(state: CombatState, action: Action) -> Transition`. Choose entry ownership narrowly: the engine initializes from one explicit semantic `Scenario` whose inputs are supplied by the caller. Controlled-entry and captured cases map to that same internal state only after separately authorized evidence work; provenance belongs in an external case wrapper, never `CombatState`. No capture parser or full-run emulator.

**Evidence prerequisite:** exact entry facts and ownership; card play/movement/draw/shuffle; Cultist/Jaw Worm intent/action/AI/history; Strength/Ritual/Vulnerable timing; energy/block/turn reset; effect order/evaluation timing; death/victory precedence; cleanup/Burning Blood healing order and both HP observation points; exact RNG/arithmetic facts from Task 3. Minimum original validation includes complete Cultist and Jaw Worm traces at no fewer than three seeds each, base/upgraded starter cards, block reset, Bash/Vulnerable, enemy buffs, shuffle, victory/healing, death, controlled edge cases, RNG vectors, and original-runtime capture/adapter/non-interference gates. Source/header claims or isolated probes alone do not establish whole-combat correctness.

- [ ] After evidence review/plan revision, write concrete red tests: `test_legal_action_sequences_preserve_structural_invariants_and_replay` generates deterministic reachable cases using supported setup and legal actions, asserting ID uniqueness, pile ownership, valid references, and identical replay transitions; `test_invalid_action_is_atomic` asserts state equals its pre-action snapshot; `test_unsupported_mechanic_fault_retains_partial_state`; `test_mid_resolution_invariant_fault_retains_partial_state`; `test_effect_guard_faults_after_100000_resolved_effects` asserts exact count/kind/retained partial fields; `test_faulted_state_rejects_action_after_guard`; the two terminal tests named in Review Focus; and evidence-backed complete vertical traces with exact observed expectations. Unsupported/invariant/guard tests must assert retained changes, fault kind, blocked next step and recovery only through valid restore/new initialization—not merely that an exception was raised.
- [ ] Run each concrete focused test before implementation and confirm the expected failing behavior.
- [ ] Implement `initial_combat`, ordered scheduler, card/enemy/power/turn/terminal mechanics from reviewed evidence only; resolve/count effects at the mandated point and preserve partial faults. Unknown mechanics fail explicitly.
- [ ] Run focused tests and `.venv/bin/python -m pytest tests/python/combat -q`; pass is necessary but not sufficient for validated status. Promote only after the full reviewed-original matrix and external gates pass.

### Task 5: Public projection and integrated diagnostics (blocked)

**Files:** `view.py`, extend `snapshot.py`, `tests/python/combat/test_view.py`, `test_vertical_combat.py`.

**Interfaces:** Task 5 owns `PublicView` and `public_view(state: CombatState) -> PublicView`; Task 4 owns `legal_actions`, returning stable ordered semantic actions. Task 2 owns `Divergence` and `compare`.

**Evidence prerequisite:** visibility/reveal facts for supported decision phases, reviewed decision states, candidate ordering, and Task 4 combat fixtures. Pair states with identical public histories only where evidence establishes those histories.

- [ ] After evidence review/plan revision, add concrete tests `test_public_view_ignores_hidden_rng_and_unrevealed_order`, `test_revealed_order_is_retained_in_public_history`, and integrated first-divergence test; assert exact fields, candidate descriptors/order, revealed history and divergence context from reviewed cases.
- [ ] Run concrete tests red; implement public projection without internal RNG, hidden AI or unrevealed order; run focused tests and full combat directory green.

## Evidence promotion and handoff gates

- Every supported mechanic must have tests traced to reviewed original evidence. Invariants and reachable-state properties establish consistency only.
- “Validated” requires the original-validation minimum and existing capture/adapter/non-interference requirements. Linux headless execution is not a substitute.
- Keep private source, bytecode, binaries, assets and private derived artifacts out of the repository. Ordinary fixture wrappers record evidence provenance outside game state.
- Rust is a later independent implementation. Parity and divergence reports supplement, never replace, original validation.

## Self-review

1. **Spec coverage:** Tasks 1–2 cover structural state, identity/order, faults, clone/restore/compare. Gated Task 3 covers exact arithmetic/RNG; Task 4 covers all mechanics, reachable invariants/replay, errors/guard, both full encounters, death/victory/healing; Task 5 covers visibility and integrated comparison. Original validation gates remain explicit. No training, full game, native optimization, or Rust implementation.
2. **Step scan:** Tasks 1–2 identify exact files, field owners, signatures, test names/assertions, red/green commands and minimal implementation boundaries. Gated tasks cannot execute until revised with evidence-grounded expected values.
3. **Type consistency:** `Scenario`, `RngState`, `CombatState`, `Action`, `Transition`, `CombatSnapshot`, `Divergence`, `FaultKind`, and `PublicView` each have a named owner and matching signatures. Task 4 owns stepping/action failures; Task 2 only owns direct fault helpers.
4. **Review Focus:** Guard/terminal tests map to Task 4; invalid/mid-resolution tests to Task 4; visibility to Task 5; numeric/RNG to Task 3 and vertical ordering to Task 4. Every focus item has a named owner test.
5. **Proportion:** Added only known structural constructor/type facts and test assertions. Exact mechanics, RNG algorithms, game values and behavioral oracles remain absent and explicitly block full execution readiness.
