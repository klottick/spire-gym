# Narrow Python Combat Behavior Implementation Plan

> **Approved sequencing amendment (2026-10-16):**
> [Source-first Rust development](../../scope.md#approved-source-first-sequencing-amendment-2026-10-16)
> supersedes the original-captures-before-Rust restriction in this completed plan's
> historical status and steps. Original fidelity promotion remains gated; the Python
> reference is not original-runtime validated. Rust requires its own reviewed design
> and plan/start approval; this completed Python plan does not authorize Rust code.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve the approved supplied-entry Ironclad A0 combat slice through player choices, enemy turns and terminal HP observations, preserving exact conditional enemy chances.

**Architecture:** Integrate the existing immutable project RNG into owned schema-2 combat state. A deterministic data-only work queue separates arithmetic/selection, card/pile effects, turn scheduling and detached public views. Author all behavior independently from the reviewed fact handoffs; neither Python self-tests nor Rust parity substitutes for original validation.

**Tech Stack:** Python 3.12 standard library and existing pytest 9.1.1; no dependency changes, NumPy/tensors/native calls in rules, Java execution or private test oracle.

**Spec:** [Approved combat specification](../specs/2026-10-16-python-combat-behavior-design.md), approved by the user on 2026-10-16.

**Status:** User approved this plan and execution on 2026-10-16. All seven tasks are complete under the selected Sol implementer/fresh-reviewer workflow, including independent whole-engine review and scoped correction review. Final controller verification: 735 tests passed; whitespace clean. This is a static-evidence-grounded Python reference, not original-runtime validated or authorized for shipping/Rust handoff. Exact execution reports, source snapshots, reviews and command substitutions are retained in `.superpowers/sdd/2026-10-16-python-combat-behavior-plans/progress.md`; the steps below retain their original planned wording rather than claiming every literal command ran.

## Global Constraints

- “Support complete combat **from a supplied first ready player decision through subsequent choices and terminal cleanup**.”
- “No implicit Ironclad HP, energy or hand-size defaults.”
- “The supplied deck has 1–999 cards, all owned exactly once across initial hand/draw.”
- “Preserve the probability of each next enemy selection conditional on the relevant gameplay state and selected move history.”
- “Use snapshot schema **2** and supported content identity `spire-lab-combat-reference-v2`.”
- “The standalone `RNG_CONTRACT_ID` does not change.”
- “At count 100,000, stop with an effect-guard fault before applying that dispatch's payload.”
- “Faults are not losses and do not imply automatic rollback.”
- “No product edits, new dependencies, original runtime access, staging/commit/push, cleanup or new subsystem are authorized by this document.” Product execution requires the user's subsequent plan/start approval; Git writes need their own explicit authorization.
- No original startup/UI input, HP sampling, new content, Ascensions, negative Strength, Rust, training, adapter work or original RNG parity. No changes to `rng.py`, its two test modules or `project_rng_v1.json` are planned.
- Use the existing isolated worktree. Preserve unrelated changes and all private accepted/failed evidence. Omit skill-default commit steps: retain exact task source snapshots and review reports locally instead.
- Sequential Sol task implementers and fresh independent review gates; controller verifies each handoff, then a fresh whole-engine review. A review finding is corrected and rechecked before advancing. No private producer/reviewer overlap or nested agents.

## Review Focus

1. Mutable input/view aliasing: changing a scenario, restored state or returned view must not change another owner; Task 1 ownership tests and Task 4 detached-view tests.
2. Bool/foreign/late-invalid action fields and mismatched targets: reject before any energy, pile, queue or RNG mutation; Task 4 exact invalid-action snapshot tests.
3. RNG exhaustion after partial rejection/shuffle/AI progress: retain the latest correct stream cursor, no fabricated choice or loss; Task 3 exhaustion tests and Task 5 recycle exhaustion.
4. Pending-work restoration and guard boundaries: resume between completed effect dispatches without resetting the same step's budget; Tasks 1/3 checkpoint tests.
5. Long-combat numeric boundaries: no hidden Python-only overflow support or turn-counter wrap; Task 2 arithmetic limits and Task 6 explicit out-of-surface counter fault.

---

## File map and shared contract

Paths below are relative to the repository root. Each task owns the listed changes
until its review finishes; later tasks consume the named interfaces, not guessed
alternatives.

| File | Responsibility |
|---|---|
| Modify `python/spire_lab/combat/model.py` | State/setup types, flags, ownership and validation; preserve existing action/error/fault types. |
| Modify `python/spire_lab/combat/content.py` | Content-v2 ID and immutable supported data. |
| Modify `python/spire_lab/combat/snapshot.py` | Schema 2, detached complete replay, compatibility/fault precedence. |
| Modify `python/spire_lab/combat/__init__.py` | Update package description only; no wildcard export framework. |
| Create `python/spire_lab/combat/rules.py` | Pure card statistics, bounded arithmetic and exact AI selection. |
| Create `python/spire_lab/combat/engine.py` | Entry, step/preflight, single-dispatch/resolution loop and fault boundary. |
| Create `python/spire_lab/combat/effects.py` | Damage/block/power/card-use/terminal effect application. |
| Create `python/spire_lab/combat/piles.py` | Draw/recycle/discard work and gameplay membership order. |
| Create `python/spire_lab/combat/turns.py` | Enemy execution/roll, round hooks and next-player scheduling. |
| Create `python/spire_lab/combat/view.py` | Frozen detached public values and legal actions. |
| Modify existing combat `conftest.py`, `test_model.py`, `test_snapshot.py`, `test_fault_state.py` | Explicit migration of structural assertions, not their deletion. |
| Create `tests/python/combat/test_entry.py`, `test_rules.py`, `test_enemy_ai.py`, `test_resolution.py`, `test_effects.py`, `test_actions.py`, `test_view.py`, `test_piles.py`, `test_turns.py`, `test_vertical.py` | Task-local tests below. |

### Exact model/API decisions

Keep `CardInstance`, `Piles`, semantic `PlayCard`/`EndTurn`, `Transition`, existing
exceptions, `mark_fault` and `require_steppable`. Keep `phase`/`status` as validated
strings, not incompatible new numeric action/phase protocols.

Introduce these ordinary values in `model.py`:

| Type/field | Exact contract |
|---|---|
| Mutable `PowerState(amount: int, skip_first: bool = False, monster_source_grace: bool = False)` | Power maps become `dict[str, PowerState]`. Preserve object identity and flags when stacking. Ritual alone uses skip-first; Bash-created Vulnerable has grace false. |
| Mutable `DamageEntry(base: int, output: int)` | Owned enemy entries; refresh output in place. |
| Frozen `RngState(contract_id: str, shuffle: RandomStreamState, enemy_ai: RandomStreamState, enemy_instance_id: str)` | Replaces opaque slots. Domains/scopes/owner validated; private AI scope suffix need not equal the public enemy handle. |
| `PlayerState.draw_per_turn: int` | Required field, no implicit scenario default; existing `max_energy` is the ordinary reset amount. |
| `EnemySetup` | Fields of the old `EnemyState`: `instance_id, enemy_id, hp, max_hp, block, powers, intent_id, move_history`; used only by `Scenario.enemies`. |
| Runtime `EnemyState` | Same setup fields plus `selected_move: str`, `damage_entries: dict[str, DamageEntry]`, `intent_damage: int | None`. |
| `Scenario` | Existing field names; `enemies: tuple[EnemySetup, ...]`, typed new `rng`. |
| Runtime state additions | `current: PendingEffect | None`, `in_play: CardInstance | None`, `resolved_effects: int`; phase and pending phase-work encode continuations, no Python stack/closure serialization. |
| `FaultKind.RNG_EXHAUSTED = "rng_exhausted"` | Additional distinct fault value; existing first-fault semantics unchanged. |

Keep `PendingEffect(kind, source_id, target_id, payload)` and exact builtin
primitive tuple payloads; `kind` remains a string. Recognized kinds have the exact
payload schemas below, not arbitrary executable callbacks. Source/target `"player"`
is reserved; an enemy ID cannot collide with it. An in-play card is owned exactly
once even when it is not yet in a pile.

Phases are `decision`, `player_action`, `enemy_turn`, `round_end`, `player_start`,
`terminal`; Transition statuses are `decision`, `terminal`, `fault`. A ready state
has empty pending work, no current/in-play card, a live player/enemy and consistent
published/selected move. Terminal may retain an in-play diagnostic card; do not
invent a pile destination merely to erase it.

`validate_state(state)` validates structural ownership/types including new fields.
`validate_runtime_state(state)` additionally checks supported content, numeric,
power, phase and derived-value invariants, without imposing first-entry conditions
on later turns. `validate_scenario(scenario)` enforces the spec's entry conditions.
Use `InvariantError` for malformed owned values and `UnsupportedMechanicError` for
well-formed excluded content. Snapshot compatibility is checked before fault/state
validation, as before. Diagnostic snapshot creation does not validate or repair.
Runtime validation permits zero HP while terminal work is pending; it must not
misclassify an ordinary lethal transition as an invariant fault. Busy phases may
have selected/published moves differing until publication; ready phase may not.

### Work catalog and dispatch ownership

Payload column lists the exact keys; tuples preserve that listed key order. Empty
payload means `()`. Integer damage `amount` and enemy damage `move` are mutually
exclusive alternatives. Scalar keys are exact builtin types, not bool-as-int.

| Kind | Payload | Owner |
|---|---|---|
| `damage` | `amount: int` OR `move: str` | `effects.py`; move resolves the source enemy's persistent entry at execution. |
| `gain_block` | `amount: int` | `effects.py` |
| `apply_power` | `power_id: str, amount: int` | `effects.py`; initialize only the power-specific flags for a newly inserted power. |
| `reduce_power` / `remove_power` | `power_id: str, amount: int` / `power_id: str` | `effects.py`; full reduction queues removal at top. |
| `finish_card` | empty | `effects.py`; source is the in-play card ID. |
| `decision` / `terminal` | empty / `outcome: str` | `effects.py` |
| `discard_hand` / `draw` / `recycle` | empty / `amount: int` / empty | `piles.py` |
| `end_player_turn` / `monster_start` / `enemy_move` / `roll_move` / `end_round` | empty | `turns.py` |

All owner dispatchers are `apply_effect(state: CombatState, effect: PendingEffect)
-> None`. `engine._apply_effect` routes the finite known kinds to their owner;
unknown kinds raise UnsupportedMechanicError. Task-stage unimplemented kinds fail
explicitly until their owning task lands—never silently no-op. Helpers use direct
list prepend/append; no generic runtime dispatcher registration.

### Fixed test inputs and evidence boundary

`conftest.py` supplies a `scenario_factory(**overrides)` fixture. Its explicit
default test values are player 40/50 HP, energy/reset 3, draw count 5, Cultist 48/48,
hand `[strike#1, defend#2, bash#3]`, empty other piles, first intent/history, and
zero blocks/powers. Overrides are **named Scenario fields**, applied before
validation; nested objects can be supplied explicitly. `minimal_state` becomes a
valid ready clone of that scenario; preserve its three-card identity assertions.

Use the existing RNG factory: root `bytes(32)`, partition `test`, episode
`combat-behavior-plan`, shuffle scope `("combat-plan-1",)`, AI scope
`("combat-plan-1", "enemy-scope-1")`, public owner `enemy-1`. Independently fixed
planning answers from the accepted standalone RNG are: first uniform-below-100
roll **29**, cursor **1**; shuffling storage `[1,2,3]` gives **`[2,3,1]`**, cursor
**2**, hence draw order **`[1,3,2]`**. These are project RNG answers, not original
combat captures. Do not regenerate expected values from the new engine.

The approved public spec's behavior/AI tables are normative. Local private
administrative bindings are in `.superpowers/sdd/2026-10-16-python-combat-behavior/`:
`semantic-review.md` (F01–F47 except quarantined F30),
`lifecycle-independent-review.md` (L01–L22, L15/L16 are prior reuse),
`lifecycle-guard-correction-review.md` (GC01–GC03),
`probability-fidelity-report.md`, and `planning-discard-order.md`.
They identify exact private provenance; they are **not runtime/test imports**.
Read only necessary independently worded accepted clauses for implementation;
original bodies must not be translated into product code. No new original study
or public raw-evidence fixture is part of this plan.

## Task 1: Typed owned entry and schema-2 replay

**Files:** Modify `model.py`, `content.py`, `snapshot.py`, combat `conftest.py`,
`test_model.py`, `test_snapshot.py`, `test_fault_state.py`; create `engine.py` and
`test_entry.py` under the paths mapped above.

**Interfaces:** Consumes existing `RandomStreamState`, `StreamId`, `RNG_CONTRACT_ID`.
Produces the shared types/validators above and
`engine.initial_combat(scenario: Scenario) -> CombatState`. Initial derivation is
content data only: create enemy damage entries with base/output 6 for Cultist,
11/7 for Jaw Worm Chomp/Thrash, and initial attack preview 11 or Incantation None.
Exports in `snapshot.py` retain existing signatures.

- [ ] **Step 1: Write failing schema/entry tests** and migrate old constructors,
  preserving every ownership/order/fault assertion. Replace the opaque-slot test
  with `test_rng_bundle_rejects_wrong_domains_scopes_and_owner`; replace old
  content/schema expectations with v2/2 and explicit v1 rejection.

  ```python
  def test_entry_is_supplied_ready_state(scenario_factory):
      state = initial_combat(scenario_factory())
      assert (state.phase, state.turn, state.player.hp) == ("decision", 1, 40)
      assert state.enemies[0].move_history == ("incantation",)
      assert state.pending == [] and state.current is None and state.in_play is None
      assert state.resolved_effects == 0 and state.player.draw_per_turn == 5
      assert snapshot(state).schema_version == 2
  ```

  Add `test_entry_rejects_inconsistent_setup_without_aliasing`: zero/1000 HP or
  reset energy, draw count 0/11, 0/1000 cards, overfull hand, bool integer, two
  enemies, mismatched encounter/owner, wrong first history, nonempty entry
  discard/exhaust/powers and malformed stream scopes all fail without changing
  the caller. `test_entry_and_restore_own_nested_values` independently mutates
  scenario, state and restored values and asserts the other two stay unchanged.
  `test_schema1_rejected_before_fault_validation` preserves existing precedence.

- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_entry.py tests/python/combat/test_model.py tests/python/combat/test_snapshot.py tests/python/combat/test_fault_state.py`.
  Expected RED: missing new types/entry or explicit new v2 assertions, not unrelated imports.
- [ ] **Step 3: Implement** the shared model/entry/compatibility contract. Deep-copy
  accepted inputs; validate first intent/ranges and owner without sampling. Include
  `current`, `in_play`, `resolved_effects` in detached comparison after `pending`
  and before `public`, preserving other existing component precedence. Keep
  diagnostic faulted snapshots possible and restoration of them forbidden.
- [ ] **Step 4: Run** the Step 2 command, then `.venv/bin/python -B -m pytest -q tests/python/combat`.
  Expected GREEN including unchanged standalone RNG tests.
- [ ] **Step 5: Retain an exact source/test handoff and obtain fresh review.** No
  staging/commit; review migration assertions and entry-vs-runtime validation.

## Task 2: Bounded arithmetic and exact conditional AI

**Files:** Create `rules.py`, `test_rules.py`, `test_enemy_ai.py`; extend immutable
data in `content.py`.

**Interfaces:** Consumes `CardInstance`, `RandomStreamState`; produces frozen
`CardStats(cost: int, damage: int, block: int, vulnerable: int)` and these `rules.py` functions:

- `card_stats(card: CardInstance) -> CardStats` with the exact six spec rows.
- `normal_damage(base: int, strength: int, vulnerable: bool) -> int`.
- `block_after_add(current: int, amount: int) -> int`.
- `hp_and_block_after_damage(hp: int, block: int, damage: int) -> tuple[int, int]`.
- `clamped_strength(current: int, delta: int) -> int` for nonnegative slice.
- `jaw_worm_fallback(history: tuple[str, ...], roll: int) -> tuple[int, int] | None`.
- `jaw_worm_move(history: tuple[str, ...], roll: int, fallback: bool | None) -> str`.
- `choose_enemy_move(enemy_id: str, history: tuple[str, ...], stream: RandomStreamState) -> tuple[str, RandomStreamState]`.

- [ ] **Step 1: Write failing arithmetic/AI tests.** Pin the exact card table and:

  ```python
  assert normal_damage(9, 0, True) == 13
  assert normal_damage(6, 999, False) == 1005
  assert normal_damage(11, 999, True) == 1515
  assert block_after_add(998, 8) == 999
  assert hp_and_block_after_damage(40, 5, 11) == (34, 0)
  assert hp_and_block_after_damage(5, 0, 11) == (0, 0)
  assert clamped_strength(998, 3) == 999
  ```

  `test_exact_history_laws` enumerates rolls 0–99 and both fallback booleans,
  weights leaves with `fractions.Fraction`, and asserts all five spec table rows
  exactly. Use histories `()`, `("chomp",)`, `("chomp","bellow")`,
  `("chomp","thrash")`, `("chomp","thrash","thrash")`; include boundaries
  24/25/54/55/99. Assert exact fallback arguments `9/16`, `2994733/8388608`,
  `3489661/8388608`, impossible repeats zero and every row sums to one.
  `test_forced_moves_consume_no_words` checks empty-history first moves and all
  later Cultist attacks return the unchanged stream. `test_only_blocked_bucket_draws_fallback`
  patches the module's sampling functions and checks 1 bucket call plus at most
  1 required Bernoulli, no aggregate weighted sampler. `test_real_project_first_roll`
  asserts Jaw Worm history `("chomp",)` chooses Thrash, ending at cursor 1.
  `test_malformed_selection_input_does_not_sample` covers bool/out-of-range rolls,
  foreign history values and excluded enemy IDs before any draw.
- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_rules.py tests/python/combat/test_enemy_ai.py`.
  Expected RED: missing rules/functions.
- [ ] **Step 3: Implement** the pure functions, using the existing `uniform_below`
  and `bernoulli` and the two-stage spec rule. Do not mutate history or streams.
  For this bounded slice `floor(1.5*n)` can be `(3*n)//2`: bases <=11 and Strength
  <=999 give half-integer results <=1515, exactly representable in binary32.
  Block addition is <=1007 before cap; HP subtraction is >=−1010 and healing <=1005
  before clamp. Check argument bounds rather than extending overflow semantics.
- [ ] **Step 4: Run** the Step 2 command and `.venv/bin/python -B -m pytest -q tests/python/combat`.
  Expected GREEN; rational enumeration, not statistical tolerance, is required.
- [ ] **Step 5: Retain exact handoff and fresh review.** Review finite masses versus
  rounded decimals and history selection versus execution. No Git writes.

## Task 3: Ordered single-dispatch resolution, powers and faults

**Files:** Extend `engine.py`; create `effects.py`, `test_resolution.py`,
`test_effects.py`; extend model runtime validation/snapshot tests as needed.

**Interfaces:** Consumes Tasks 1–2; produces `effects.apply_effect`,
`effects.refresh_enemy_values(state) -> None`,
`engine._apply_effect(state, effect) -> None`,
`engine._dispatch_one(state: CombatState) -> None`, and
`engine._resolve(state: CombatState) -> Transition`.
`_resolve` does not reset the persisted `resolved_effects` budget; external step
will reset it. Normal replay safe points are between completed payload dispatches,
with `current is None`; no arbitrary Python-function-stack snapshot is promised.

- [ ] **Step 1: Write failing ordered-effect/fault tests.** `test_top_bottom_do_not_replace_current`
  pins prepend/append and current-work ownership. `test_damage_payload_ownership`
  queues fixed card amount 8 and enemy move reference, changes only the enemy
  entry output to 9, and observes 8 versus 9 respectively. `test_power_stacking_preserves_flags`
  asserts Vulnerable 2+3=5 without replacing existing flags; Ritual retains
  skip-first on stacking. `test_reduce_then_remove_not_immediate_replacement`
  checks full reduction queues removal; Strength 998+3 caps 999 and refreshes
  enemy damage in place, preserving entry object identity.

  ```python
  # In a synthetic owned scheduler state, not a claimed game entry:
  _dispatch_one(state)
  saved = snapshot(state)
  assert state.resolved_effects == 1 and state.current is None
  assert compare(_resolve(restore(saved)).state, _resolve(state).state) is None
  ```

  `test_guard_99999_and_100000` uses a test-only patched dispatcher to enqueue
  repeated valid work: 99,999 dispatches can reach decision; attempt 100,000
  faults before its payload, with count 100000 and the failing current work
  retained. `test_partial_fault_is_first_and_not_loss` mutates HP, injects a
  failure, then verifies partial HP, the first FaultRecord, no fabricated outcome
  and rejection by the existing `require_steppable` gate. Task 4 additionally checks
  the completed `step` API's faulted-state rejection. `test_rejection_exhaustion_keeps_latest_cursor`
  forces a valid word rejection at the last available AI word, asserts that only
  AI reaches `2**64`, shuffle is unchanged and status is fault/RNG_EXHAUSTED.
  For this task inject the Task 2 sampler failure through a test-only patched
  dispatcher; Task 6 separately tests the real roll-move handler.
- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_resolution.py tests/python/combat/test_effects.py`.
  Expected RED on missing dispatch/effect semantics, not private evidence access.
- [ ] **Step 3: Implement** one-dispatch-at-a-time handling with explicit current,
  counter and first-fault preservation. Increment immediately before payload;
  guard at 100000 first. Catch ordinary handler exceptions into INVARIANT faults,
  distinguish UnsupportedMechanicError and RngExhaustedError, return status fault;
  do not catch BaseException/process termination. A failing current effect remains
  diagnostic and is never automatically retried. Implement core damage/block/power
  kinds and the quiescent `decision` marker; refresh persistent enemy entries in
  place. Empty work outside decision/terminal is an invariant fault, not an implicit
  fabricated decision. Leave not-yet-owned task kinds
  explicitly unsupported. Mid-operation RNG exhaustion installs `.state` in its
  identified owning stream before returning the fault.
- [ ] **Step 4: Run** Step 2 and `.venv/bin/python -B -m pytest -q tests/python/combat`.
  Expected GREEN, including snapshot resume with an unchanged same-step budget.
- [ ] **Step 5: Retain exact handoff and fresh review.** Review exceptions after
  partial mutation, double-counting at resume and preserving the failing work.

## Task 4: Player choices, public projection and bounded terminal completion

**Files:** Extend `engine.py`, `effects.py`; create `view.py`, `test_actions.py`,
`test_view.py`; extend `test_effects.py` for terminal cases.

**Interfaces:** Produces `engine.step(state: CombatState, action: Action) -> Transition`,
`view.legal_actions(state: CombatState) -> tuple[Action, ...]`, and
`view.public_view(state: CombatState) -> PublicView`. Frozen dataclasses in
`view.py` have these fields (all numeric fields below are `int`):

- `CardView(instance_id, card_id: str, upgraded: bool)`.
- `PlayerView(hp, max_hp, block, energy, max_energy, powers: tuple[tuple[str, int], ...])`.
- `EnemyView(instance_id: str, enemy_id: str, hp, max_hp, block, powers: tuple[tuple[str, int], ...], intent_id: str, intent_damage: int | None)`.
- `PublicView(content_id: str, turn, player: PlayerView, enemies: tuple[EnemyView, ...], hand: tuple[CardView, ...], draw_count, discard: tuple[CardView, ...], exhaust: tuple[CardView, ...], revealed_order: tuple[str, ...], outcome: str | None, hp_before_healing: int | None, hp_after_cleanup: int | None)`.

Sort public power pairs by power ID; preserve hand/discard/exhaust order. Expose
no lifecycle flags or mutable TerminalOutcome reference.
`engine._validate_action(state: CombatState, action: object) -> None`
checks exact types/targets before matching legal choices; view imports model/rules,
never engine, avoiding a cycle.

- [ ] **Step 1: Write failing choice/view/terminal tests.** Play base Bash then
  Strike against fresh Cultist: HP `48 -> 40 -> 31`, Vulnerable 2, energy `3 -> 1 -> 0`;
  new Bash Vulnerable must not affect its own first attack. Upgraded Strike under
  Vulnerable deals 13; Defend adds 5/8 and moves normally to discard.

  ```python
  before = snapshot(state)
  with pytest.raises(InvalidActionError):
      step(state, PlayCard(True, "enemy-1"))
  assert compare(state, before.state) is None
  ```

  Parameterize wrong/missing/already-played card ID, attack target None, Defend enemy
  target, bool target/card fields, unrelated target, insufficient energy, foreign
  action class, terminal and busy state; all pre-mutation. `test_public_hidden_state_noninterference`
  changes only hidden RNG/derivation scope and unrevealed draw order in equal-public
  states, asserting equal views and legal actions. Assert hand-order actions then
  EndTurn, no hidden RNG/AI work/episode fields, and view mutation cannot alias.
  `test_terminal_healing_and_no_invented_pile_reset` tests HP 40/50→46, 48/50→50,
  death→0, queues/current cleared and no choice; do not assert an unevidenced
  terminal card destination. Use a four-upgraded-Bash, energy-8 valid entry for
  victory here; test death via the core queued damage handler, not yet-unimplemented
  EndTurn. Faulted view/enumeration raises FaultedStateError;
  terminal enumeration is empty. `test_unneeded_exhausted_stream_does_not_fault`
  kills the sole enemy while AI is exhausted and asserts victory, no AI sampling.
- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_actions.py tests/python/combat/test_view.py tests/python/combat/test_effects.py`.
  Expected RED on missing step/view/card/terminal behavior.
- [ ] **Step 3: Implement** complete preflight before setting phase/resetting the
  step counter. Valid card use calculates damage first, removes hand membership
  into in-play ownership, spends cost, enqueues damage/block then Vulnerable,
  finish-card and decision work. For this no-extra-content slice, lethal damage
  schedules the explicit terminal checkpoint before another ordinary decision;
  that checkpoint synchronously records pre-heal HP, applies Burning Blood only
  on victory, records post-cleanup HP and clears work. No general terminal pile
  normalization or simultaneous-lethal policy. This is not frame/terminal-pile
  parity. Valid EndTurn routing is completed by Task 6, not a silent success here.
- [ ] **Step 4: Run** Step 2 and `.venv/bin/python -B -m pytest -q tests/python/combat`.
  Expected GREEN for the scoped task, including lethal attacks and visibility.
- [ ] **Step 5: Retain exact handoff and fresh review.** Check bool equality traps,
  in-play identity, no preflight sampling, terminal callback versus queue clearing.

## Task 5: Draw, reverse-top discard and recycle work

**Files:** Create `piles.py`, `test_piles.py`; route its three kinds in `engine.py`
and refactor `effects.py` finish-card membership to the shared primitive.

**Interfaces:** Produces `piles.apply_effect(state, effect) -> None`,
`piles.discard_card(state: CombatState, card_id: int, *, end_turn: bool) -> None`.
Card finish calls this same membership primitive; ordinary end-turn discard
suppresses manual-discard callbacks, not all discard bookkeeping. No callbacks
are needed for the three supported cards; no generic hook registry is introduced.
The primitive accepts a hand-owned or in-play card and removes its source before
appending to discard; it never duplicates membership. Synthetic pile test setups
must update the declared issued-card set and satisfy structural ownership.

- [ ] **Step 1: Write failing pile tests.** `test_draw_top_is_storage_end` draws IDs
  `[3,2]` from storage `[1,2,3]` without replacement. `test_discard_all_reverses_hand_storage`
  asserts hand `[1,2,3]`, existing discard `[9]` becomes hand `[]`, discard
  `[9,3,2,1]`. `test_recycle_fixed_project_order` asserts shuffle `[1,2,3]`→storage
  `[2,3,1]`, draw `[1,3,2]`, cursor 2. `test_split_draw_existing_then_recycle_then_residual`
  starts with draw `[4]`, discard `[1,2,3]`, asks 3 and asserts hand order
  `[4,1,3]`, remaining draw `[2]`, discard empty. `test_hand_cap_and_deck_exhaustion`
  starts at hand 9 asking 5 and draws only one; empty combined piles finish with
  no fabricated card. `test_singleton_recycle_consumes_no_words` preserves cursor.
  `test_recycle_exhaustion_preserves_piles_and_latest_stream` forces a later shuffle
  word failure: discard stays untransferred, latest shuffle cursor retained, AI
  unchanged, diagnostic fault, no partial fabricated permutation.
  `test_draw_records_public_reveal_order` asserts drawing from `[1,2,3]` twice
  appends `("3","2")` to the public revealed sequence, with remaining top hidden.
  Standalone resolution tests append a `decision` marker after their pile work;
  an empty busy queue is not implicitly a ready state.
- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_piles.py`.
  Expected RED on missing pile behavior.
- [ ] **Step 3: Implement** immediate gameplay membership with last-element draw
  and ordinary discard traversal. A split draw prepends existing draw, recycle,
  residual draw to remaining work in that execution order; recycle reads current
  discard and uses the existing pure shuffle, installing returned RNG. Transfer
  shuffled storage to draw only after shuffle succeeds. Recompute sizes/cap at
  execution; append each actually drawn card's decimal ID to public reveal order,
  never the unrevealed remainder. Do not promise arbitrary presentation/follow-up callbacks.
- [ ] **Step 4: Run** Step 2 and `.venv/bin/python -B -m pytest -q tests/python/combat`.
  Expected GREEN, with all card/identity/ownership regressions retained.
- [ ] **Step 5: Retain exact handoff and fresh review.** Check split child insertion,
  top direction, card uniqueness and failed-shuffle preservation.

## Task 6: Enemy turns, round hooks and next-player scheduling

**Files:** Create `turns.py`, `test_turns.py`; extend `engine.py`, `effects.py` and
runtime validation where these handlers require it.

**Interfaces:** Produces `turns.apply_effect(state, effect) -> None` and
`turns.schedule_next_player_turn(state: CombatState) -> None`. Consumes exact
`choose_enemy_move`, owned damage entries, core power and pile effects. Publishes
`selected_move` into `intent_id` without appending history again; next intent
damage is refreshed before returning a decision.

- [ ] **Step 1: Write failing turn tests.** `test_cultist_ritual_and_round_growth`
  ends three turns without attacks: HP 40→40→34→25; next attack previews 6,9,12;
  Ritual skip-first becomes false after the first round; Strength absent/3/6;
  AI cursor remains 0. `test_jaw_worm_first_turn_and_roll_publication` starts with
  upgraded Defend played (block 8), then EndTurn: HP 40→37, player block resets 0,
  next intent Thrash 7, history `("chomp","thrash")`, AI cursor 1. Assert no second
  enemy action executes in that turn.

  `test_roll_fallback_exhaustion_keeps_history_and_cursor` forces the real roll
  handler's bucket to 0 while advancing the last available AI word; its required
  fallback then exhausts. Assert history/selection are uncommitted, AI cursor
  `2**64`, shuffle unchanged, first RNG_EXHAUSTED fault and no terminal outcome.

  `test_bellow_then_thrash_block_and_strength_order` uses a test-only legal script
  selecting Bellow then Thrash: Bellow gives Strength 3 before block 6, next preview
  Thrash 10; that block persists through the player decision and resets on monster
  start, not arbitrary player start. `test_vulnerable_stack_and_round_expiry`
  covers Bash 2→end-round 1→next end-round removal and reapply without flag reset.
  `test_scheduling_side_effects_precede_queued_round_power` dispatches end-round
  in a controlled owned state and inspects the safe-point queue: energy already
  replaced, selected intent published, Ritual Strength pending before draw; after
  resolution preview includes Strength. `test_turn_counter_outside_numeric_surface_faults`
  sets the synthetic counter to `2**31-1`; attempted next-turn increment faults
  UNSUPPORTED without wrapping or inventing an outcome.
- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_turns.py`.
  Expected RED on missing EndTurn/round scheduling.
- [ ] **Step 3: Implement** the five turn kinds and EndTurn routing. Discard ordinary
  hand, reset enemy block/start hooks, dispatch the published move payload, then
  roll for the later move; append history only after successful selection. Preserve
  partial AI cursor on a failed fallback. Monster end-turn uses the monster-side
  hook argument, then player/monster round hooks; Ritual's round flag skips the
  first round and later queues Strength, Vulnerable queues reduction/removal.
  `schedule_next_player_turn` performs evidenced synchronous start/reset/recharge/
  publication construction effects and appends draw/decision work after pending
  round effects. Recharge replaces energy; player block resets at player start.
  Terminal handlers preempt remaining normal turn work. Refuse out-of-surface
  numeric increments instead of introducing Java-overflow emulation.
- [ ] **Step 4: Run** Step 2 and `.venv/bin/python -B -m pytest -q tests/python/combat`.
  Expected GREEN; all finite catalog kinds now have real handlers.
- [ ] **Step 5: Retain exact handoff and fresh review.** Check both internal turn
  phases, constructor-vs-resolution timing, skip-first and selection publication.

## Task 7: Whole-combat acceptance and honest status

**Files:** Create `test_vertical.py`; update package description and the approved
spec's implementation-status note only after verification. Product fixes found
here belong in their owning modules and require regression tests/fresh review.

**Interfaces:** Uses the existing public entry/step/view/snapshot functions; no new
public API, general trace format or fixture-loader subsystem.

- [ ] **Step 1: Write the following fixed complete-combat tests** before any final
  integration fixes. Test values are independently derived authored expectations,
  not original-runtime captures:

  | Test | Exact actions and expected checkpoints |
  |---|---|
  | `test_two_turn_cultist_victory` | Entry 40/50 player HP, Cultist 48, hand upgraded Bash#1/Strike#2/Defend#3, energy 3, draw count 3. Play #1,#2: enemy 25. EndTurn: player 40, Ritual 3, next attack 6, hand `[1,3,2]`, shuffle cursor 2, AI cursor 0. Play #1,#2: victory; pre-heal 40/post-cleanup 46. |
  | `test_two_turn_jaw_worm_victory` | Same entry/cards with Jaw Worm 40. First two plays leave enemy 17. EndTurn: player 29, next Thrash 7, history Chomp/Thrash, hand `[1,3,2]`, AI cursor 1. Next Bash leaves enemy 2, Strike kills; pre-heal 29/post-cleanup 35. |
  | `test_cultist_death_without_victory_heal` | Entry player 14/20, one Defend, draw 1; EndTurn three times without playing. HP 14→14→8→0; death, both HP checkpoints 0, no AI words. |
  | `test_jaw_worm_lethal_stops_future_rng` | Entry player 10/20, one Defend, draw 1, AI exhausted. EndTurn takes Chomp 11 and dies; no attempted roll/fault, HP checkpoints 0. |
  | `test_ready_snapshot_replay_matches_all_components` | Save after first Bash and after first EndTurn in each victory case; restore, replay remaining same semantic actions and assert `compare(...) is None`, identical views/outcomes/stream cursors. |

  Add `test_supported_paths_never_lose_or_duplicate_cards` across each intermediate
  decision, including owned in-play state in internal tests. `test_seed_variants_and_legal_walks`
  exercises both enemies with 20 distinct project episode identities and at most
  200 legal actions each; every returned value is a valid decision/terminal/fault,
  no unexpected fault, and replay reproduces it. Do not call these original seeds
  or a statistical chance proof. Use existing action order for a deterministic
  self-test policy, not a new RNG domain or training component.
- [ ] **Step 2: Run** `.venv/bin/python -B -m pytest -q tests/python/combat/test_vertical.py`.
  If earlier tasks already satisfy the new acceptance tests, an immediate GREEN is
  legitimate integration evidence, not a fabricated RED; any discovered defect
  first gets a minimal failing regression in its owning test module.
- [ ] **Step 3: Make only evidenced integration fixes** through the owning task's
  interfaces. No expected-value refresh from engine output, skipped private gates,
  provisional helper no-op, new scenario sampler or content expansion.
- [ ] **Step 4: Run** `.venv/bin/python -B -m pytest -q`, then `git diff --check`.
  Verify the unchanged RNG fixture SHA256 is
  `369d91e2ce3be38c191ef5ceab6fe6e1bf5c73f21504e1748074e9c35d962bad`,
  no private evidence/binaries/new dependencies are in product changes, all tests
  pass without private paths, and only planned files changed. Record actual test
  counts/output; do not assume the earlier 395-test count still applies.
- [ ] **Step 5: Obtain a fresh whole-engine review** against the spec and exact
  final source snapshot, independent of task implementers. Resolve findings with
  regression tests, rerun full verification, retain history and report remaining
  external gates. Mark implementation as static-evidence-grounded Python reference,
  **not original-runtime validated**. No Rust handoff, Git commit/push or original
  capture follows automatically.

## Final handoff and external gates

Human plan review/start confirmation was received on 2026-10-16; retain the chosen
Sol execution method. Execution is tracked in the plan-owned local SDD ledger.

Original validation remains incomplete: at least three original seeds for each
supported enemy, required interactions and two HP observation points, original
Windows/adapter/export/load/visibility/non-interference gates. RNG-001 permits
matched supplied semantic outcomes, not treating a shared seed or Python/Rust
agreement as original correctness. This plan neither reopens frozen capture work
nor grants original execution/desktop access. Report that status without deleting,
repairing or relabeling preceding evidence.
