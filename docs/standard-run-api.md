# Standard Run API (M34)

This is the public contract of the native `Standard` Run profile in spire-env and its Python transport. Standard covers
every character (Ironclad, Silent, Defect, Watcher), Ascension 0–20, Acts 1–4 and the Heart, with all M26–M33 content.
The milestone record, coverage ledger and reconciliation are in [M34](milestones/m34-run-api.md). The Rust core
(`crates/spire-core`) owns every rule and value. spire-env (`crates/spire-env`) owns the session, encoder, records and
checkpoints. Python (`crates/spire-python`, `python/spire_lab/gymnasium/run.py`) only transports them.

Status: M34 source candidate, extended by M34s-a (public knowledge, the Standard combat view and `standard-combat-v1`
sessions; [M34s profile](milestones/m34s-search-scenario.md)). M34s (search and scenario API), M35 (batching) and M36
(delivery) build on this contract.
Until the roadmap finishes, no compatibility is promised (fingerprints may change with disclosure). M35 and M36 freeze it.

## Identities

| Identity | Value |
| --- | --- |
| Profile (native / Python) | `RunProfile::Standard` / `profile="standard"` |
| Observation schema | `spire-gym-standard-run-observation-v1` |
| Observation fingerprint | SHA-256 over the canonical descriptor (`observation_spec()["fingerprint"]`); since M34s-a `bf8e1d111bbf2171594231b99d8d6b701244249abfbb90a11e86c4b4a405f7ae` (M34b: `efbc1253544a80c5c4a5b3a63cb8169a215229aaffa4ac2af5402252e195311a`; see [Records](#records-and-semantic-replay)) |
| Standard combat view | `spire-gym-standard-combat-view-v1`, fingerprint `5d8147e13ded030c7faa501a8202eb2588915c2e1330b30cac5b06afc9f9916e` ([below](#standard-combat-view-and-sessions)) |
| Standard combat sessions | profile `standard-combat-v1`, catalog `spire-gym-standard-combat-catalog-v1`, reward `spire-gym-combat-outcome-reward-v1` |
| Record schema | `spire-gym-standard-run-replay-v1` |
| Rules / content | `spire-gym-native-standard-rules-v2` (v1 until M34s-a; bumped for the autoplay free-play fix G-a4 and the Secret Technique/Weapon candidate order R-a1) / `spire-gym-standard-content-v1` |
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
| `combat_cards` | 1024 | zone, zone_position, card_kind, upgrades, effective_cost, permanent_row, stasis_enemy_row, selected, base_damage, base_block, magic_number, damage_bonus, cost_kind, free_play, exhaust_on_finish, ethereal, innate, temporary_cost, misc, combat_cost, turn_cost, self_retain, retain, autoplay_free, free_play_once, rolled_cost, modified_combat_cost, hp_loss_discount, cost_zero_this_turn |
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
- **Draw pile (M34s-a).** Every draw-pile card that is not published as `known_draw` (or as another row, such as a Seek
  candidate) is a `combat_cards` row in zone `draw` with `zone_position` −1 (known 0). These rows come after every other
  row, sorted by the canonical card key: the published row without zone and zone_position, `permanent_row` last. Rows that
  tie are interchangeable for every card field the M34s-a census classified (published, non-distinguishing, or equal by
  invariant; the list is in the [M34s profile](milestones/m34s-search-scenario.md)), so their order carries no
  information; it is never the physical draw order. With Frozen Eye the order is public and the rows are `known_draw` references in order.
- **Card lifecycle (M34s-a).** The last six `combat_cards` columns are facts the player sees or derives from public history,
  each read by later play: `autoplay_free` (the card still holds an autoplay waiver; displayed cost 0), `free_play_once`
  (a one-use waiver from Forethought or Setup; displayed cost 0), `rolled_cost` (the cost Confusion or Snecko Oil rolled,
  shown when drawn; unknown when none), `modified_combat_cost` (a cost changed for this combat, e.g. Madness,
  Enlightenment, Force Field; unknown when none), `hp_loss_discount` (Blood for Blood's discount from HP lost this
  combat) and `cost_zero_this_turn` (Infernal Blade's cost 0 until the end of the turn). Descriptor semantics
  `card_lifecycle`.
- **Intent damage.** `intent_damage` is the damage per hit as the original `AbstractMonster.calculateDamage` displays it:
  attacker and defender modifiers and stance, Back Attack's ×1.5 truncated on the unfloored value, then the player's
  Intangible cap (1), then the floor. Damage dealt is unchanged (Back Attack truncates after the floor at the hit, as the
  original's `applyPowers`); only the display moved to the original's order (M34s-a).
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

**Re-recording (M34s-a, design §4.12).** The draw rows and card lifecycle columns change the Standard observation
fingerprint (`efbc1253…` → `bf8e1d11…`). A Standard record made between M34b (`d1825a96`) and the M34s-a merge carries
the old rules id (`spire-gym-native-standard-rules-v1`, checked first) and the old fingerprint, so `replay_run`, `RunSession::from_record` and Python `NativeRunSession.replay` reject it with a typed
`IncompatibleReplay` (Python `RunInputError`), never a silent mismatch. There is no legacy-fingerprint replay path:
re-record Standard runs from that window. Records of the captured profiles are unaffected by the fingerprint change (their
fingerprints and rules ids do not change), but the gameplay fix G-a4 (see the observation-vocabulary CHANGES) can make a
rare retained M10, M25 or Gym66 record diverge: one that reaches a card played by Havoc (or Omniscience) and later
returned by Exhume (or Tantrum) and played again now costs energy, so replay stops with a typed `IncompatibleReplay` at
that step, never silently. The retained frozen-replay inputs (323) are unaffected.

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

## Standard combat view and sessions

**Combat view.** `combat_observation_spec()` and `combat_view()` on a Standard session return the Standard combat view
`spire-gym-standard-combat-view-v1` (fingerprint domain `spire-gym/combat-view/standard/v1\0`). It is a projection of the
Standard observation: groups `combat_context`, `player`, `permanent_cards`, `combat_cards`, `enemies`, `powers`, `relics`,
`potions`, `orbs`, `selection`, `selection_counts`, `actions`, `combat_state`, `known_draw`, `stance_choices`,
`combat_card_offers` and `action_mask`, with Standard's columns, vocabularies, domains and capacities (each group equals the
Standard observation's at the same boundary). `combat_context` holds the Standard `context` columns character, ascension,
act and room_kind, plus `encounter` (the live combat's encounter, in the Standard encounter vocabulary) and `burning` (1 for
a burning elite); both are unknown outside a live combat. `actions` rows are the Run's native dispatch indices: step the Run
with the sidecar's token and row. Outside combat the mask is empty and `info.routing` is `noncombat`. The info carries
`combat_view_id`, `active`, `routing` (`combat`, `combat_selection`, `noncombat`), `selection_owner`, the phase, the visit,
entered/settled counts and a sticky `last_exit`; it is not a feature. The captured, policy-v3 and Gym66 view descriptors and fingerprints are unchanged; their
values change only as listed in the [observation vocabulary CHANGES](observation-vocabulary.md) (the Back Attack display
wherever Shield and Spear is admitted, including Gym66 `ironclad-combat-v1/v2`, and the G-a4 gameplay fix).

**Sessions (`standard-combat-v1`).** `CombatTrainingCatalog::standard_from_json` and `CombatTrainingSession::new_standard`
(Python `NativeStandardCombatSession` and the Gymnasium `StandardCombatEnv`) start one Standard combat per episode from
owned resources:
- **Catalog** `{"catalog_schema_id": "spire-gym-standard-combat-catalog-v1", "profile_id": "standard-combat-v1", "entries": [...]}`,
  ≤ 1 MiB, 1–256 entries, unknown fields rejected (hidden state such as `draw_order`, `seed`, `rng` or `hand` is never an
  input; `position` and `recipe` arrive in M34s-c). Each entry: `id`, `character`, `ascension` (0–20), `encounter` (any row
  of the Standard setup table; its act is the setup act), optional `burning` (`strength`, `max_hp`, `metallicize` or
  `regeneration`; elite encounters only), `hp`, `max_hp` (≤ 999), `gold`, `deck` (1–999 `{card, upgrades, misc}`, with
  entry-generated cards such as Mark of Pain's Wounds counted toward the engine's 999-card bound; an unordered multiset;
  only values the engine runs: Searing Blow up to 73 upgrades, `misc` 0 except Genetic Algorithm 1–999 and Ritual Dagger
  15–3003), `relics` (the
  character's complete relic union, in acquisition order), `relic_counters` and `potions` (the slot count is 3, or 2 from
  A11, plus 2 with Potion Belt). Resources are already owned: no acquisition effect is applied again. The normalised
  catalog is fingerprinted (`spire-gym/standard-combat-catalog/v1\0`) and returned by `catalog_descriptor()`.
- **Episode.** `reset(seed, entry_id)` builds the Run from the validated resources and enters through the shared
  `enter_owned`; steps use the shared resolver. The episode ends at the combat's own terminal, before Run settlement:
  combat-end hooks (Burning Blood, Black Blood, Repair) have applied; settlement effects (Meat on the Bone, Face of Cleric)
  and rewards never apply. `info.terminal_reason` is `victory` (an all-enemies-escaped victory sets
  `info.all_enemies_escaped`), `death` or `escaped` (Smoke Bomb), with `hp_before_healing` and `hp_after_cleanup`. The
  reward is +1, −1 or 0; `truncated` is always false. Faults freeze the session and are never retried.
- **Observation** is the Standard combat view; `info` adds `start_kind` (`fresh_start`), `entry_id` and `episode_ordinal`.
  No seed or hidden value appears in observations, info or specs; the effective seed is only in `diagnostic()`, an explicit
  oracle (a twin session with that seed replays the true future).
- The Ironclad Gym66 surfaces reject `standard-combat-v1` catalogs (and `new_standard` rejects Gym66 catalogs) with
  `UnsupportedBoundary` (Python `RunUnsupportedBoundaryError`).

**Public knowledge (core).** `Combat::draw_knowledge()` is the exact posterior over the draw pile's card-class sequence
given public history (`spire-gym-public-knowledge-v1`, design §4.5.1), or `Unrepresentable` until the next shuffle when an
operation leaves the exact model. `Combat::preroll_sets()` holds the values of hidden pre-rolled move numbers (Louse Bite,
Darkling Nip) consistent with every published intent display and execution. Neither is read by rules or published in an
observation; the M34s belief model (part b) consumes them.

**Measured costs (M34s-a; release build, ignored measurement tests, not gates).** On the supplied Standard routes (4
characters × {A0, A20}; the environment routes reach the Heart; in the core knowledge measurement both Watcher routes stop
at the 4,000-step cap), a Standard encode at a combat boundary took a median of about 22–25 µs without draw rows and 23–28 µs with
them (2–10 draw rows; `spire-env` `tests::m34s_measure`). `Combat::draw_knowledge()` took a median of 6–21 µs per combat
decision (p99 under 36 µs); every route stayed at one component and never became `Unrepresentable`
(`spire-core` `run::tests::m34s_measure`, `--features m34-diagnostics`). The bound is 65,536 components. Knowledge
maintenance (every draw-pile operation and pre-roll observation) and the core draw multiset run only in a combat entered
with public knowledge (the Standard Run and `standard-combat-v1`; `Combat::maintains_public_knowledge`): there it costs
8–19 µs per core step on supplied Standard A20 routes (14–28%). Other profiles do none of it; their core step time equals
base (Gym66-style Ironclad 13.6 vs 13.5 µs, M10 12.0/10.9 vs 11.8/10.7 µs; `m34s_knowledge_maintenance_cost_per_step`).

## Privacy and ownership

The encoder reads only `PublicDecision` and `PublicMap`. Public means what the player currently sees, or has been shown
and is entitled to remember (for example the known draw order, or event cards that were revealed and flipped back). Tests
cover Runic Dome, the A20 second boss (not named before its room), card-reward children (hidden until opened), undisplayed
event captures, and the absence of seed, RNG and cursor from the spec, info and descriptor. Observations, records,
checkpoints and Python arrays never alias live state, including across restore and fork.

## Base for M34s

M34s (spire-dxo.71) builds on this contract. Part a changed it as listed above (the Standard fingerprint, draw rows, card
lifecycle columns, the intent display, the Standard combat view and `standard-combat-v1`); parts b and c add belief
branching and tactical starts:
- **Roots.** M34s branch roots are public roots of live sessions: the public decision plus public knowledge. An
  `ExactCheckpoint` is never a source for public branching (it holds hidden state); `exact_*` tools stay exact-world tools.
- **Sessions.** `exact_fork` gives independent sessions with their own tokens, ownership and record.
- **Randomness.** The root-keyed HMAC domain `spire-lab-research-rng-v1` (core `RESERVED_RESEARCH_RNG_DOMAIN`) is reserved for
  research draws. It is disjoint from the Run (`spire-lab-run-rng-v1`) and combat stream labels, so research draws can never
  move a gameplay stream. M34 adds no draw API.
- **Hidden state.** Public is player knowledge (above). An M34s belief model resamples exactly the complement.
- **Names.** `exact_*` is reserved for exact-world tools. Public-information operations use other names.
- **Combat view.** The Standard combat view (above) is the observation of every Standard combat session and branch.

## Limits

- No strategy, policy or win-rate claim. The examples (`crates/spire-env/examples/standard_rollout.rs`,
  `examples/standard_run.py`) take random legal actions.
- Not covered here: batching, vector envs and autoreset (M35); throughput and memory gates and packaging (M36); checkpoint
  serialisation; public-information resampling and branch limits (M34s-b); synthetic positions and recipes (M34s-c).
- `standard-combat-v1` is not the original runtime: its terminal is the engine's combat terminal (settlement effects
  that the original applies at victory, such as Meat on the Bone, are Run settlement here and do not apply).
- Source acceptance is not Python delivery proof. Python results count only on a verified built extension (see
  [AGENTS.md](../AGENTS.md)).
