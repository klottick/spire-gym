# Standard Run API (M34)

This is the public contract of the native `Standard` Run profile in spire-env and its Python transport. Standard covers
every character (Ironclad, Silent, Defect, Watcher), Ascension 0–20, Acts 1–4 and the Heart, with all M26–M33 content.
The milestone record, coverage ledger and reconciliation are in [M34](milestones/m34-run-api.md). The Rust core
(`crates/spire-core`) owns every rule and value. spire-env (`crates/spire-env`) owns the session, encoder, records and
checkpoints. Python (`crates/spire-python`, `python/spire_lab/gymnasium/run.py`) only transports them.

Status: M34 source candidate. M34s (search and scenario API), M35 (batching) and M36 (delivery) build on this contract.
Until the roadmap finishes, no compatibility is promised (fingerprints may change with disclosure). M35 and M36 freeze it.

## Identities

| Identity | Value |
| --- | --- |
| Profile (native / Python) | `RunProfile::Standard` / `profile="standard"` |
| Observation schema | `spire-gym-standard-run-observation-v1` |
| Observation fingerprint | SHA-256 over the canonical descriptor (`observation_spec()["fingerprint"]`); at M34b `efbc1253544a80c5c4a5b3a63cb8169a215229aaffa4ac2af5402252e195311a` |
| Record schema | `spire-gym-standard-run-replay-v1` |
| Rules / content | `spire-gym-native-standard-rules-v1` / `spire-gym-standard-content-v1` |
| Reward / seed adapter / RNG | `spire-gym-floor-key-terminal-reward-v1` / `spire-gym-run-u64-le-v1` / `spire-lab-rng-hmac-sha256-v1` |
| Capacity preset / vocabulary manifest | `standard-default-v1` / `standard-vocab-v1` |

A record replays only under the same identities. A gameplay change bumps the rules id in the same change.

## Creation and configuration

Native: `RunSession::new_standard(StandardRunConfig { character, ascension, partition, creation, capacities, record_replay })`.
`StandardRunConfig::new(character, ascension, partition)` uses the experienced creation profile, `RunCapacities::standard_default()`
and no recording. Python: `NativeRunSession(character, ascension, profile="standard", partition=..., creation=None,
capacities=None, record_replay=False)` or `RunEnv(..., profile="standard", creation=...)`.

`StandardCreation` has three fields:
- `neow`: `Full` or `Short`.
- `seen_bosses`: for each act, which of that act's three bosses have been seen.
- `stored_note`: the Note For Yourself card and its upgrades.

The default `StandardCreation::experienced()` is a full Neow, all bosses seen and Iron Wave+0. With all Act 3 bosses seen,
A20 fights two Act 3 bosses; with any unseen, it fights one. The Python `creation` dict is strict
(`{"neow", "seen_bosses", "note"}`), and `boss_history` is rejected on Standard. Invalid input (an unknown or wrong-act boss,
an illegal Note, an ascension above 20) raises `Input` before any Run is built.

The creation profile is configuration and player knowledge, so Standard publishes it in `environment_spec()`, the reset
info (`info["creation"]`) and the record. It is never a per-step observation column. Each reset builds a fresh Run from
the configured creation. Nothing one episode does (bosses becoming seen, the Note) reaches another episode or session.

## Observation

The Standard descriptor (`observation_spec()`) is the only feature schema. Python derives its spaces from it and defines
no features of its own. Groups and columns:

| Group | Rows | Columns |
| --- | --- | --- |
| `context` | 1 | character, ascension, act, floor, room_kind, event_kind, terminal_kind, reward_total, ruby, sapphire, emerald, map_boss, winged_moves, combats_entered, combats_settled, last_encounter, last_outcome, content_profile |
| `player` | 1 | hp, max_hp, gold, block, energy, max_energy, turn, draw_count, stance, focus, hp_before_healing, hp_after_cleanup, mantra_gained |
| `map_nodes` / `map_edges` / `current_node` | 107 / 742 / 1 | floor, x, y, kind, resolved_room, burning, visited, revealed / source_row, target_row / row |
| `permanent_cards` | 1024 | card_kind, upgrades, misc |
| `combat_cards` | 1024 | zone, zone_position, card_kind, upgrades, effective_cost, permanent_row, stasis_enemy_row, selected, base_damage, base_block, magic_number, damage_bonus, cost_kind, free_play, exhaust_on_finish, ethereal, innate, temporary_cost, misc, combat_cost, turn_cost, self_retain, retain |
| `enemies` | 12 | enemy_kind, hp, max_hp, block, intent_kind, intent_damage, replaced, escaped, half_dead, logical_slot, side, intent_hits, intent_category |
| `powers` | 2048 | owner_kind, owner_row, power_kind, amount |
| `relics` / `potions` / `orbs` | 256 / 16 / 10 | relic_kind, visible_counter / potion_kind / kind, dark_damage |
| `offers` | 512 | choice_kind, status, price, amount, card_kind, upgrades, relic_kind, potion_kind, card_type, permanent_row, owner_wins, neow_category, neow_reward, neow_drawback, required_count, selected_count, event_stage, event_option, param0, param1, param2 |
| `selection` / `selection_counts` | 1024 / 1 | source_group, source_row, eligible, selected, reveal_position, selected_ordinal / required, selected_count, minimum, maximum, confirm_available, operation |
| `actions` | 2048 | verb, source_group, source_row, target_group, target_row, choice_kind, scalar0, scalar1, stance_choice_ref |
| `combat_state` | 1 | facing, surrounded, heart_damage_remaining, heart_cap, previous_card_type, phase, selection_resume_phase |
| `known_draw` / `stance_choices` / `combat_card_offers` | 1024 / 2 / 512 | from_top, offset, combat_card_row / stance / card_kind_or_wish_choice, offer_index |
| `room_params` | 1 | stage0, stage1, stage2, stage3 |
| `action_mask` | 2048 | 1 for each legal candidate |

Each group has a `_known` mask and, for row groups, an `_occupied` mask. Padding is value 0 / known 0 / occupied 0. An
unknown reference inside an occupied row is −1 with known 0. Arrays are owned, C-contiguous, native-endian and row-major.
The descriptor's `semantics` section states the meaning of each rule below and is part of the fingerprint.

- **Vocabularies.** The open families (cards, relics, potions, events, encounters, actors, rooms, choice kinds, selection
  operations, intent categories) come from the checked-in manifest `crates/spire-env/src/standard_vocab_v1.json`. It was seeded
  once in lexicographic order and is append-only: an index never moves when an id is added. Powers use the shared
  `PUBLIC_POWERS` order. Intents use the shared `PUBLIC_INTENTS` order followed by `EVENT_INTENTS`. A shared id added later is
  appended after them, so Standard indices stay fixed (see [observation vocabulary](observation-vocabulary.md)).
- **Enemies.** Only present actors are published (active with HP above 0, or half-dead), in engine order. `logical_slot`
  tracks position. `intent_category` is the source intent class per (monster, move). `intent_hits` is the move's planned
  damage applications, including AI memory such as Book of Stabbing. Both are read from the engine when the move is
  selected; they are unknown for non-attacks (hits) and while intents are hidden (Runic Dome). A Stasis card whose holder
  has retired keeps zone Stasis with `stasis_enemy_row` unknown.
- **Heart cap.** `combat_state.heart_cap` is the present Corrupt Heart's live Invincible cap (300 at A0–A18, 200 from A19).
- **Event and room values.** Event rows carry `event_stage`, `event_option` and up to three displayed numbers
  (`param0..2`). `room_params` carries up to four stage numbers (Treasure `stage0` is the chest size). Their meanings per
  (event, stage, option, index) are the descriptor's `param_semantics`. Fixed identities and numbers of the six
  accepted-handler events are in `event_constants`.
- **Capacities.** The preset `standard-default-v1` covers every engine bound (deck ≤ 999, selection ≤ 1,000, map 105 + boss +
  the A20 second-boss node, present enemies ≤ 9). Custom capacities may be larger. Exceeding a capacity is a typed
  `Capacity` fault, never a loss.

## Actions

The action space is `Discrete(2048)` with the `action_mask`. An index is the core candidate index of the current boundary,
and each legal candidate has one row in `actions`. Indices are not stable game identities; always step with the current
mask and token. Every Standard candidate kind is encodable: full and short Neow, map moves (including Winged Boots), combat
plays, potions (`use_potion`, `discard_potion`), card offers, rewards (including Singing Bowl), shop (including purge), campfire
Rest/Smith/Recall/Lift/Toke/Dig, every event row, treasure, boss relics, Act 3 end, the A20 second boss, Act 4, the Heart and
TrueVictory. No choice kind is encoded as padding. A nonterminal boundary always has at least one legal action.

## Steps, rewards and terminals

`step(token, index)` returns the next boundary, `reward`, `terminated`, `truncated` (always false) and `reward_events`.
`reward_events` itemises the reward and sums to it:

| Item | Amount |
| --- | --- |
| `NewFloor{act, floor}`: first entry to a floor of Acts 1–3 (including the A20 floor 51) | +1 |
| `NewAct4Floor{floor}` | +10 |
| `FirstKey(key)`: first Ruby, Sapphire or Emerald key | +5 |
| `HeartVictory` | +100 |
| `Death`, `EmptyDeck` | −1 |

Revisits, repeated keys, reads, resets, invalid input, faults and `Act3NoKeys` add nothing. Python step info carries
`reward_events`, `episode_return` and `heart_victory`. The terminal reasons are `heart_victory`, `death`, `empty_deck` and
`act3_no_keys`; any of them sets `terminated`. A combat `Escaped` outcome continues the Run. TrueVictory is the final view
after the HeartVictory terminal.

## Errors and faults

Invalid input (`Input`), a stale token (`StaleToken`), a step before reset (`NotReset`), a step at a terminal (`Terminal`) and
a closed session (`Closed`) change nothing. An operational problem (a core fault, `Capacity`, a delivery fault) freezes the
session: the error is retained, the record notes whether the step committed and was recorded, no reward is added and no
terminal is set. Nothing is retried. A frozen session cannot step, reset or restore. Python maps these errors to
`RunInputError`, `RunStateError`, `RunFaultError` and `RunUnsupportedBoundaryError`.

## Seeding

`reset(seed)` starts ordinal 0 of `seed`. `reset(None)` continues the same seed at ordinal + 1 (the first `reset(None)` uses
seed 0). Run randomness is HMAC-derived per (partition, ordinal, scope), and combat streams per (ordinal, act, encounter,
visit). Reads (`decision`, specs, `record`, `reproduction`, `exact_checkpoint`) never draw. The spec, info and descriptor
carry no seed, RNG state or cursor.

## Records and semantic replay

With `record_replay=True`, `record()` returns an owned `RunEpisodeRecord`. It holds the identities, config (including
creation), seed, ordinal, the accepted actions with their rewards and terminals, and any fault. `to_json`/`from_json` is the
portable form (exact shape; unknown keys are rejected). `replay_run(&record, capacities)` (Python
`NativeRunSession.replay(bytes)`) re-runs the actions from the record's start. It checks the identities, the fingerprint, that
each action was offered, each reward and terminal, the reward items and the recorded fault. Replay checks gameplay, not
every configuration value. A field that changes neither legality nor rewards is not bound by replay, and records are not
signed.

### Replay to a chosen decision

`RunSession::from_record(&record, steps, capacities)` (Python `NativeRunSession.from_record(bytes, steps)`,
`RunEnv.from_record(bytes, steps)`) returns a live session at decision `steps`, after the record's first `steps` actions. It
validates the record as replay does, starts at the record's seed and ordinal under its config with `capacities`, and drives
each recorded action through the session's own `step`. The live publication, capacity, reward and fault checks therefore
apply unchanged.
- `steps` past the end raises `Input`.
- A prefix that diverges raises `IncompatibleReplay`.
- A prefix that reaches a recorded fault that recurs (a core, capacity or reward-item fault) reproduces it and returns that
  error.
- A recorded fault that cannot recur (a transport or batch delivery fault, or a capacity fault under larger capacities) does
  not stop the prefix, as in replay.

The new session's record is the prefix and continues from it. `reset(None)` then continues at the record's ordinal + 1. The
cost is about 150–180 µs per accepted step (release build, A0 rollouts), against about 80–105 µs per step for `replay_run`.

## Exact checkpoints (privileged)

> **Oracle warning.** An exact checkpoint copies the whole hidden state: the deck order, RNG counters, future card rewards
> and enemy AI. Restoring it replays the same hidden future, so "step, look, restore" reveals information the player does
> not have. Use exact checkpoints for exact-world reproduction: paired or repeated starts, debugging, tests. Do not use them
> as planning input. Public-information branching is a separate M34s feature.

- `exact_checkpoint() -> ExactCheckpoint` works on any reset, unfrozen, unclosed session, terminal included. Taking a
  checkpoint changes nothing.
- `exact_restore(&checkpoint) -> RunBoundary` needs the same profile and config (capacities, creation, recording). It
  abandons the current episode, restores seed, ordinal, record prefix and last step, and republishes the boundary under a new
  serial, so every earlier token is stale. A later `reset(None)` continues at the checkpoint's ordinal + 1. A frozen or closed
  session cannot restore, and a rejected restore changes nothing.
- `RunSession::exact_fork(&checkpoint) -> RunSession` creates a new session with a new identity and its own tokens and
  record. It works even after the source session has frozen or closed: the fork does not inherit a later fault. That is an
  explicit caller choice, not a retry.

`ExactCheckpoint` is opaque and immutable. It has no accessor, no `Clone`, `PartialEq` or `Hash` (state equality would be a
hidden-state side channel), and its `Debug` output is `ExactCheckpoint { .. }`. In Python it cannot be pickled or copied
(`TypeError`), its repr is redacted, and equality and hashing are by identity. A checkpoint can be restored or forked any number
of times. It is in memory only; the record is the portable form. At the first Act 3 boss, a checkpoint holds about 0.85 MB
and takes about 30 µs to take or fork (release build).

On the Gymnasium facade, `exact_checkpoint`, `exact_restore`, `exact_fork` and `from_record` are methods of the base `RunEnv`
(reach them through `env.unwrapped`), not part of the Gym API. `RunEnv.decision()` returns the current `(obs, info)` of a
forked or replayed env.

## Privacy and ownership

The encoder reads only `PublicDecision` and `PublicMap`. Public means what the player currently sees, or has been shown
and is entitled to remember (for example the known draw order, or event cards that were revealed and flipped back). Tests
cover Runic Dome, the A20 second boss (not named before its room), card-reward children (hidden until opened), undisplayed
event captures, and the absence of seed, RNG and cursor from the spec, info and descriptor. Observations, records,
checkpoints and Python arrays never alias live state, including across restore and fork.

## Base for M34s

M34s (spire-dxo.71) adds search and scenario features on top of this contract without changing it:
- **Branch roots.** An `ExactCheckpoint` or a `from_record` session. Both carry the full config, creation, seed and ordinal.
  A branch is reproducible from the start only when the root session records (`record_replay=True`): then the root also
  carries the record prefix. A checkpoint of a session without recording has no action history, so M34s branch roots
  must come from recording sessions. A `from_record` session records whenever its source record did.
- **Sessions.** `exact_fork` already gives independent sessions with their own tokens, ownership and record.
- **Randomness.** The root-keyed HMAC domain `spire-lab-research-rng-v1` (core `RESERVED_RESEARCH_RNG_DOMAIN`) is reserved for
  research draws. It is disjoint from the Run (`spire-lab-run-rng-v1`) and combat stream labels, so research draws can never
  move a gameplay stream. M34 adds no draw API.
- **Hidden state.** Public is player knowledge (above). An M34s belief model resamples exactly the complement.
- **Names.** `exact_*` is reserved for exact-world tools. Public-information operations use other names.
- **Combat sidecar.** `combat_view()` on Standard returns `UnsupportedBoundary` naming M34s. The Standard observation's
  combat groups are the source for the M34s Standard combat sidecar.

## Limits

- No strategy, policy or win-rate claim. The examples (`crates/spire-env/examples/standard_rollout.rs`,
  `examples/standard_run.py`) take random legal actions.
- Not covered here: batching, vector envs and autoreset (M35); throughput and memory gates and packaging (M36); checkpoint
  serialisation; public-information resampling, branch limits, synthetic setups and Gym66/standalone starts (M34s).
- Source acceptance is not Python delivery proof. Python results count only on a verified built extension (see
  [AGENTS.md](../AGENTS.md)).
