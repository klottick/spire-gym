# Changelog

Changes from the versioned contract freeze onward. The earlier `## CHANGES`
entries are retained in [observation vocabulary history](docs/observation-vocabulary.md).

## Unreleased

No changes yet.

## 1.2.0 — effective at the commit tagged v1.2.0

See the [1.2.0 release notes](docs/releases/v1.2.0.md).

- Set the project and Python package version to `1.2.0`. The version is a value, not a
  frozen name: the bump leaves the `build_info()` keys, frozen identities, fingerprints and
  replay digests unchanged. `source_digest` changes, because `Cargo.toml`, `Cargo.lock` and
  `pyproject.toml` are digest inputs.
- M39b adds the opt-in `spire-gym-standard-combat-view-compact-v1` layout
  for standalone Standard combat sessions and batches and PublicSearch branch
  views. `observation="compact"` selects `standard-combat-compact-v1`;
  pass `capacity_preset="standard-combat-compact-play-v1"` for smaller arrays.
  Both presets support capacity overrides and report typed overflow faults.
  Dense remains the default. See the [Standard combat API](docs/standard-run-api.md#compact-standard-combat-view-m39b).
- The `dev` dependency group adds pytest-xdist 3.8.0 (with execnet 2.1.2) so
  the full Python suite can run across workers; see
  [development](docs/development.md#full-python-suite-in-parallel). Because
  `pyproject.toml` is a build-digest input, extensions built after this change
  report new `source_digest` and `file_digest` values. Runtime behavior and
  frozen identities are unchanged.
- spire-lez: the `a20_checks` example adds `S.step_limit_episodes` to its JSON
  and prints a stderr line for each single-session episode that reaches the
  20,000-action limit, giving its character, ascension, seed and step count.
  Diagnostic only; the episodes and other output are unchanged.
- M40a (spire-61l stage 1) admits more `PublicRoot` and branch cases; no gameplay,
  observation, action or descriptor layout, rules ID or frozen pin changes. A nested
  card play or queued repeat whose enemy target is the player's own choice (a repeat
  from Double Tap, Duplication, Burst, Amplify, Echo Form or Necronomicon of a hand
  play) is now a supported root; it was refused as "a nested card play holds an enemy
  target". Randomly targeted autoplay (Havoc, Mayhem, Distilled Chaos, Omniscience)
  and its repeats stay refused. A branch step that picks among identical unpublished
  draw-pile copies of a single-pick grid (Secret Weapon, Secret Technique, Omniscience,
  Seek) now removes a copy chosen by a label-free research draw instead of refusing with
  `HIDDEN_ORDER_PICK`, where the draw knowledge is exact for the class; two-pick Seek+,
  Frozen Eye, non-exchangeable copies and mixed published copies keep the refusal. See
  [standard-run-api](docs/standard-run-api.md) and [M34s](docs/milestones/m34s-search-scenario.md).
- M39a (spire-e0l) adds `spire_env::combat_entry_resources_descriptor()` and
  Python `spire_lab.combat_entry_resources.load()` / `_native.combat_entry_resources()`.
  The [versioned descriptor](docs/combat-entry-resources.md) publishes relic and
  potion admission, counter and companion rules, potion-slot counts and bounds
  for owned Standard combat entries.
- Standard combat catalog decoding and validation name known fields or
  resource classes in input errors, including `gold`, `hp`, `max_hp`, `deck`,
  `deck.upgrades`, `deck.misc`, `relics`, `relic_counters`, `potions` and
  `burning`. Catalog admission and error classes are unchanged.
- spire-d4h fixes copy and top-card autoplay order in the shared combat queue.
  On-use copies go ahead of previously queued cards. Havoc, Mayhem and Distilled
  Chaos append their revealed cards in order. A copied Havoc plays before the
  cards it and its original reveal. During a Distilled Chaos burst, an already
  queued potion card can therefore be Time Warp's twelfth play, and remaining
  queued cards retire unused. This can change selection boundaries, when Havoc
  rolls its next random target, and downstream gameplay.
- When multiple copy sources fire on the same use, their dispatch order reverses
  their enqueue order. Player powers enqueue in list order, then Necronomicon
  enqueues its copy, which plays first. This corrects M27 F9's dispatch order.
  Ordinary work such as Storm's Lightning resolves before those copies,
  correcting the dispatch-order assertion in the M16 Heatsinks regression.
- A card now completes its ordinary cleanup before the next queued card plays.
  A Havoc copy can reshuffle the finished original when the deck runs out.
  A card played by Havoc can also draw the original from discard. During a
  selection, the finished producer appears in its destination zone. A finished
  Blood for Blood in discard can receive its cost reduction when damage from
  playing a copy reaches the player.
- Two ordering limits remain. Omniscience appends both plays in the original,
  while the engine front-inserts each play and schedules the second after an
  eligibility check. Storm queues its channel ahead of Power cleanup in the
  original, and the engine can run the Power's finish checkpoint first.
  See [Delivery §7](docs/delivery.md#7-known-limits-by-surface).
- Rules IDs move to `spire-gym-native-standard-rules-v7`,
  `spire-gym-native-m10-rules-v4` (also Gym66) and
  `spire-gym-native-m9-rules-v5` under the shared-engine freeze. Earlier rules
  IDs fail closed. Observation, action and descriptor layouts are unchanged.
  The Watcher A0 supplied Heart digest changes because Omniscience now
  exhausts before the card it plays. Five boundaries report the corrected
  exhaust order. The route keeps its actions, rewards and HeartVictory.
  The other Heart pins, frozen replay digests, agreement digests and
  descriptor fingerprints match their existing values. Combat-v1 has no rules ID and its
  replay digest also matches.

## 1.1.1 — effective at the commit tagged v1.1.1

See the [1.1.1 release notes](docs/releases/v1.1.1.md).

- Set the project and Python package version to `1.1.1`. The version is a value, not a
  frozen name: the bump leaves the `build_info()` keys, frozen identities, fingerprints and
  replay digests unchanged. `source_digest` changes, because `Cargo.toml`, `Cargo.lock` and
  `pyproject.toml` are digest inputs.
- spire-d8g fixes in the shared engine, found by the 2026-11-16 stress campaign. Rules IDs move to
  `spire-gym-native-standard-rules-v6`, `spire-gym-native-m10-rules-v3` (also Gym66) and
  `spire-gym-native-m9-rules-v4`; the engine is shared, so every frozen profile is treated as
  affected. Records with the old IDs fail closed. No observation, action or descriptor layout
  changes; fingerprints are unchanged.
  - A Smoke Bomb escape runs the victory effects, as the original does: its escape ends through
    the end-of-battle routine, which runs Meat on the Bone and then the player's victory callbacks.
    Burning Blood heals 6, Black Blood 12, Meat on the Bone 12 at or below half HP, Face of Cleric
    grants 1 Max HP, and Self Repair heals (it already did). Rewards stay suppressed.
    **Disclosure:** the M27 rule "G9: victory relics do not fire on Escaped" was wrong; its tests
    (`m27_smoke_bomb_rules_and_settlement`, `m27_g9_escape_skips_victory_relics`, now
    `m27_g9_escape_runs_victory_relics`, `m27_meat_on_the_bone_boundary` and
    `m27_face_of_cleric_boundary`) are amended. The diagnostics combat validator already
    expected the heal and faulted (`Invariant("victory healing")`).
  - Meat on the Bone checks its half-HP threshold on the HP before the victory heals (Burning or
    Black Blood, Self Repair) and before Face of Cleric's Max HP, as the original runs it first.
    An Ironclad winning at 40/80 now ends at 58 (it ended at 46). Only the threshold input changes.
  - A card's base cost no longer depends on a target. It used to come from a target-validated
    payload, so with no enemy targetable (Awakened One's rebirth window) Infernal Blade faulted
    (`Unsupported("Infernal Blade generated attack cost unavailable")`). A differential over every
    card kind at upgrades 0 and 1 (Searing Blow at every level), each character's combat, and
    every base-cost read in the workspace and diagnostics test suites shows no value change
    wherever the old derivation priced a card.
  - Every effect that reads a card's cost now sees it in a no-target window, as the original does,
    where costs are static card fields. Such a window has no targetable enemy, for example a won
    combat's final boundary or Awakened One's rebirth. Observations show the cost there (the
    displayed `effective_cost` is known instead of unknown; the layout does not change), and the
    cost-reading effects act on it: Snecko Eye and Confusion, Snecko Oil, Mummified Hand,
    Forethought, Madness, Enlightenment, Bullet Time, Setup, Establishment, Sands of Time,
    Nightmare, Recycle, Liquid Memories, the end-of-turn reset of temporary costs, and the cost
    rebase on an in-combat upgrade (Apotheosis, Armaments+). The cost also orders same-card copies
    in a draw-pile choice grid opened in such a window. Draw-pile knowledge views of Watcher's
    Evaluate, Wish and Reach Heaven also now show their cost, with targetable enemies too; that is
    observation only. **Release note:** in no-target windows these cost-reading
    effects can now behave differently from v1.1.0, matching the original. A Confusion roll on a
    drawn card, for example, now changes its cost and advances the card RNG. The displayed cost is
    the only cause of the re-pins: the Heart pins Ironclad-A20, Silent-A0, Silent-A20 and
    Watcher-A20, whose steps, terminal and return are unchanged, and six of the eight M35 R4
    `standard-combat-v1` agreement digests (`tests/fixtures/m35_agreement_digests.json`; R1 is
    unchanged). With the old cost derivation swapped back in, every one of them reproduces its old
    pin. The frozen replay digests and the combat-v1 digest are unchanged.
  - Time Warp's 12th card can be one Distilled Chaos played, directly or through a card it played:
    a Double Tap, Burst, Echo Form, Amplify or Necronomicon copy, or Havoc's top card. The burst's
    other queued cards are retired unused, enemies gain 2 Strength and the turn ends once queued
    work drains, as with card-owned autoplay. It used to fault
    (`Invariant("Time Warp card decision continuation absent")`). The card order for a copy of
    Havoc and for Havoc under Distilled Chaos still differs from the original and is tracked as
    spire-d4h; those cases now complete under the Time Warp rule instead of faulting.
  - A cancelled queued card (Time Warp's early end: Distilled Chaos, Havoc, Mayhem, Omniscience,
    repeats) is retired through the played-card cleanup, as the original's use-card cleanup does:
    a Power leaves the piles instead of going to the discard or exhaust pile, and Strange Spoon
    rolls for an exhaust card. A queued card that cannot be used (for example Havoc's top card
    under Normality, or a Status or Curse) is retired the same way.
  - Exemption, an input-timing interleave that is not modelled: in the original, Distilled Chaos
    used while Time Warp's 12th card is still queued leaves top-card actions that survive the
    early end. Potions are offered only at decision boundaries here, so this order cannot occur
    (see [delivery](docs/delivery.md#7-known-limits-by-surface)).
  - The Heart's Invincible caps the final damage of a hit. The Boot, which raises an unblocked
    1–4 to 5, now applies before the cap, as in the original. It used to apply after the cap and
    overspend Invincible below 0, and the validator then faulted ("Invincible is Heart-owned and
    bounded by its A0 allowance"). The other defender caps keep their original order: a monster's
    Intangible applies before The Boot, and the player's Intangible applies first. Changes only a
    hit against the Heart with The Boot.
  - Diagnostics only (default-off `m34-diagnostics`): `run::diagnostics::supply_gold`, the stress
    harness's gold supply, used by the E5 replay test. No other API change.
  - Omamori can negate a transformed curse. An event transform (Living Wall "Change", Drug Dealer,
    Transmogrifier) removes the chosen card and obtains the output through the negation-capable
    owner that Neow and Astrolabe already use: a transformed curse spends an
    Omamori charge and is not added, and the chosen card stays removed. It used to fault
    (`Unsupported("a negated card obtain needs an *_or_negated owner")`). Only that previously
    faulting case changes.
  - An enemy dies once. As in the original's `die()` guard (`isDying`) and Poison's
    `currentHealth > 0` check, a Death callback for an enemy whose death already ran does
    nothing. A poison tick's completion no longer kills again an enemy that another hit killed
    first (Corpse Explosion's THORNS damage, for example). A Bronze Orb holding a Stasis card
    used to receive two Death hooks and fault (`Invariant("Stasis return death/custody")`); its
    card now returns once, to hand or to discard at 10 cards. The guard is global, and a census
    shows where it fires. In the whole workspace test run, the M28 traversal at A0–A20 (756 runs,
    uniform, reach and supplied) and the native fuzz smoke, it fired only in the two
    previously faulting Stasis replays and in its own unit test. The fuzz smoke summary hash and
    every frozen pin are unchanged.
  - Tomb of Lord Red Mask's 222 gold with Ectoplasm is blocked entirely, as the original's
    `gainGold` returns before changing gold; nothing else changes. The event's result check now
    expects the gain the shared gold owner actually makes. It used to fault
    (`Invariant("accepted Act3 event mutation frontier changed")`). Only that previously faulting
    case changes.
  - A queued repeat waits while an earlier card of the same parent has a selection open. In the
    original the action queue (the selection) drains before the card queue, so the choice resolves
    and then the copy plays, and it may open its own choice: Burst's copy of Survivor plays after
    Survivor's discard choice and then opens its own. Burst's copy of Havoc while Havoc's top card
    (Survivor) is choosing a discard, and Duplication Potion's copy of Havoc while Havoc's top card
    (Secret Technique) is choosing a Skill, used to fault at the selection boundary
    (`Invariant("Burst repeat child/parent ownership")`,
    `Invariant("Duplication repeat retains its actual card and captured child")`); they now run
    to completion. One ownership rule now covers the repeats of Burst, Echo Form, Amplify,
    Necronomicon and Duplication Potion (Double Tap's has no separate check). It is a validator
    change only: play order is unchanged, so only the previously faulting boundaries change. In
    those Havoc paths the order still differs from the original, which plays the copy of Havoc
    before Havoc's top card, so that choice is not yet open when the copy plays. This known
    difference is tracked as spire-d4h.
  - Nightmare's delivery can wait behind a start-of-turn selection. Foresight's Scry and Nightmare
    run in the same pre-draw power pass, in power-list order, both before the turn draw. With
    Foresight first, its Scry boundary used to fault
    (`Invariant("Nightmare template identity, owner, reset state, or delivery cursor invalid")`).
    The engine already ran them in list order. This is a validator change only.
  - The hand never exceeds 10 cards at end of turn. As in the original, retained cards are restored
    before the end-of-turn exhaust listeners' bottom-queued work runs: Dead Branch's card, Dark
    Embrace's draw and Feel No Pain's Block. That work therefore sees the restored hand and the
    10-card cap, so a card that does not fit goes to discard and a draw stops at 10. The work used
    to run first, and the restore could make an 11-card hand that faulted at the next draw
    (`Invariant("hand exceeds cap")`). A census of where the new ordering applies covered the
    workspace tests, the core diagnostics, the M28 traversal at A0–A20 (756 runs) and the native
    fuzz smoke. It applied in the two E10 tests (draw and card work) and in one Standard
    random-episode test, where the only work moved was Feel No Pain's Block, which now lands after
    the restore and still before the monster turn. It never applied in the traversal or the smoke.
    The smoke summary hash and every frozen pin are unchanged.
  - Documented limit (DELIVERY §7, "Maximum HP and counters"): relic and power counters such as
    Self-Forming Clay's pending Block are bounded at 999, like a Run's maximum HP. Past that the
    engine stops with a typed `Unsupported` fault, whereas the original has no cap. No engine
    change.

## 1.1.0 — effective at the commit tagged v1.1.0

See the [1.1.0 release notes](docs/releases/v1.1.0.md).

- Set the project and Python package version to `1.1.0`. The version is a value, not a
  frozen name: the bump leaves the `build_info()` keys, frozen identities, fingerprints and
  replay digests unchanged. `source_digest` changes, because `Cargo.toml`, `Cargo.lock` and
  `pyproject.toml` are digest inputs.
- Add M37a profiling diagnostics: the ignored `m37_phase_timing` test in
  `spire-env`, `examples/run_vector_phase_timing.py`, [profiling](docs/profiling.md) and
  a README Speed section. These are diagnostics and documentation only. No gameplay,
  identity, fingerprint, observation or API change.
- M37b internal performance work, with no contract change. Gameplay, RNG use, IDs, rules
  IDs, fingerprints, observations, masks, records and replay bytes are unchanged; the
  freeze tests, frozen replay, traversal and fuzz hashes match v1.0.0. See
  [performance](docs/performance.md) and [profiling](docs/profiling.md) for the numbers.
  - Card preparation clones the combat state once per legal-action enumeration, not once
    per (card, target) payload.
  - `RunSession` returns each observation by move and keeps no copy. `decision()`
    re-encodes it from the unchanged Run, so the bytes are identical.
  - The session step no longer re-enumerates legal actions to check an action it has
    just published.
  - Record headers reuse a per-session observation-spec fingerprint.
  - The Python batch binding stacks observations per slot on the batch's own workers,
    with the GIL released. Each call still returns new arrays.
  - Two internal, `#[doc(hidden)]` additive Rust methods support this: `Run::step_offered`
    and `SessionBatch::map_jobs`. They are not part of the documented API and add no
    Python names.
- Add M38a scaling diagnostics: the ignored `m38_component_scaling` test, `M37_CAPACITIES`,
  `M37_STACK=parallel|parallel-reuse` and `M37_EARLY_DROP` in `m37_phase_timing`, the
  `--loops` and `--drop-before-step` options and a release timer in
  `examples/run_vector_phase_timing.py`, and a
  [parallel scaling](docs/profiling.md#m38a-parallel-scaling) report. These are test and
  example diagnostics only. No engine, binding, gameplay, identity, fingerprint, observation
  or API change.
- M38b internal performance work, with no contract change. Gameplay, RNG use, IDs, rules
  IDs, fingerprints, observations, masks, records, replay bytes and the bytes, dtypes and
  shapes of every returned array are unchanged; the freeze tests, frozen replay, traversal and
  fuzz hashes match M38a, and a packaging differential compares the Python batch outputs with
  the previous binding call by call. See [performance](docs/performance.md) and
  [profiling](docs/profiling.md#after-m38b) for the numbers.
  - A Standard `NativeRunBatch.step` encodes each slot's observation straight into its rows
    of the returned stacked arrays on the batch's workers, instead of encoding per-slot rows
    and copying them. Fault rows, isolated slots and other profiles are filled as before.
  - The batch keeps a bounded pool of observation array buffers. When NumPy releases a
    returned `obs` array, its memory goes back to the pool and the next step reuses it, so a
    batch may hold up to two calls' observation memory between calls. Each call still
    returns new arrays that share no memory with any array still alive; `close()` empties the
    pool. Arrays released later, after `close()` or after the batch object is freed, still
    return to it within the same bound, and the pool is freed only when the last array from
    that batch has been released. The owner object behind those arrays (`a.base.base`) is now an internal
    `PooledColumn` instead of rust-numpy's `PySliceContainer`.
  - Run and combat-training batches give each worker a home range of slots, so a slot normally
    stays on the same worker thread between calls; idle workers then take unclaimed slots from
    other ranges ([batching](docs/batching.md#slots-seeds-and-workers)).
  - The Standard encoder looks vocabulary identities up in tables built once, builds its
    group layout once per capacities and worker thread, and pre-sizes card rows; Run RNG
    draws feed their HMAC input without building a message buffer (same bytes).
  - Two internal, `#[doc(hidden)]` additive Rust items support this:
    `spire_env::ObservationTarget` and `SessionBatch::step_into`. They are not part of the
    documented API and add no Python names.
  - The `a20_checks` example gains `--speed-check`, and `examples/speed_check.py` is new: short
    fixed diagnostic runs with JSON output for build records. They have no thresholds.
  - [Batching](docs/batching.md#slots-seeds-and-workers) now advises native callers to copy
    what they need and drop the previous rows before the next step.

- M38e: an opt-in compact Standard Run observation. Additive; the dense
  `spire-gym-standard-run-observation-v1` (`bf8e1d11…`), every default, frozen
  identity, replay digest and Heart pin are unchanged. See
  [Standard Run API](docs/standard-run-api.md#compact-observation-m38e).
  - New schema `spire-gym-standard-run-observation-compact-v1` with its own
    fingerprint domain, capacity preset `standard-compact-v1` and fingerprint
    `633b9424be5070a2890b01725949447badba919c5c8c5bd798922f8c2d2a2b2d` at that
    preset; added to the freeze identity table. Same groups, columns and
    semantics as dense; fewer rows and `int16`/`int32` values (136,842 bytes per
    slot of arrays instead of 763,720). Its descriptor's `semantics.capacity_preset`
    is `standard-compact-v1` and it adds two `semantics` keys, `layout` and
    `dense_schema`; a compact session's `environment_spec()` reports
    `capacity_preset` `standard-compact-v1` (or `custom`).
  - Python: keyword `observation="dense"|"compact"` (default `"dense"`) on
    `NativeRunSession`, `NativeRunBatch`, `RunEnv` and `RunVectorEnv`; methods
    `fault_cause()` (sessions, `RunEnv`) and `fault_cause(slot)` (batches,
    `RunVectorEnv`). The `run.py` `Observation` alias gains `int32` and `int16`
    arrays. No new module-level names.
  - Rust (`spire-env`): `RunObservationLayout`, `RunSession::new_standard_with_layout`,
    `RunSession::observation_layout`, `RunSession::fault_cause`,
    `RunCapacities::standard_compact`, `standard_compact_run_observation_spec`,
    `RUN_OBSERVATION_STANDARD_COMPACT_SCHEMA_ID`, `STANDARD_COMPACT_CAPACITY_PRESET_ID`,
    `record_observation_layout` and `encode_run_standard_decision_with_layout`;
    `RunRecordError::from_error` is now public. Internal, `#[doc(hidden)]`:
    `ObservationChunk` and `ObservationTarget::typed`.
  - A compact state that does not fit is a typed `Capacity` fault through the
    existing paths: a row overflow names the row group; a value outside its dtype
    names `"<group>.<column>"` with the dtype maximum as `capacity`. The record
    codec accepts such value-fault groups only in compact records.
  - `fault_cause()` exists for every layout and profile; it is `None` unless the
    session froze. Python redacts a core cause as other Python errors do.
  - Dense speed: native dense is unchanged within noise; the dense Python batch
    step is about 3–5% slower than after M38b (cause diffuse, likely the
    encoder's code generation; see [profiling](docs/profiling.md#after-m38e)).
  - Diagnostics: `M37_LAYOUT` and `M37_STACK=slab` in `m37_phase_timing`,
    `--observation` in `a20_checks`, `examples/run_vector_phase_timing.py` and
    `examples/speed_check.py`.
- spire-3xq combat fixes in the shared engine, found by the M38e capacity stress. Rules IDs
  move to `spire-gym-native-standard-rules-v4`, `spire-gym-native-m10-rules-v2` (also Gym66)
  and `spire-gym-native-m9-rules-v3`; the engine is shared, so every frozen profile is
  treated as affected. Records with the old IDs fail closed. Each fix changes only a path
  that used to fault, so the frozen replay digests, the combat-v1 digest and the Heart pins
  are unchanged. No observation, action or descriptor layout changes.
  - Berserk stacked to 5 or more grants its whole amount at the start of each turn, as the
    original does. The energy check used to reject gains above 4.
  - A queued Fire Breathing hit keeps the amount it was queued with, as the original's
    damage snapshot does. It used to fault when another Fire Breathing stacked first (for
    example through Mayhem).
  - A won combat opens no player choice. Reptomancer's cleanup queues its victory behind
    work already queued, so a Gambling Chip discard or a killing card's follow-up choice
    (Dagger Throw's discard) used to open with no living enemy and fault. One deviation
    remains: if a Wish choice came up in that window, the original could still pay Wish's
    gold when its screen opened before the battle ended (frame timing); the engine offers no
    choice there and is not known to reach that window.
  - A choice opened between a finished card and its queued repeat is accepted: Melange's
    Scry (after a reshuffle) resolves first, then Necronomicon's repeat plays, as in the
    original. It used to fault.
  - Reaching the 999 card identities a combat can issue is the typed `Unsupported` fault
    "generated card count/runtime boundary" on every generation path. Card creation through
    the CreateCard and CopyCard paths (for example enemy status cards or Mark of Pain's Wound)
    used to report it as `Invariant`. See [delivery](docs/delivery.md#7-known-limits-by-surface).
  - A step that stops at an inconsistent choice is a retained `Invariant` fault. It used
    to surface later as `InvalidAction` for an action the mask had offered.
- Correct the `standard-default-v1` capacity note: `offers` is bounded by purchases, not by
  the engine. With The Courier a shop keeps bought rows and appends a restock per purchase,
  so more than 512 rows needs at least 498 purchases in one visit. It fails closed with a
  typed `Capacity` fault ([Standard Run API](docs/standard-run-api.md)). No identity change.
- spire-0kr fixes in the shared shop and reward owners. The Standard rules ID moves to
  `spire-gym-native-standard-rules-v5`; records with v4 fail closed. M9, M10 (and Gym66) and
  the other legacy profiles are exempt: these paths need Complete item admission, which only
  the Standard Run and default-off diagnostics use. The m27 diagnostic Act 4 shop
  (`complete_act4_run`) also restocks in place now; it has no rules ID. Combat-v1 has no shop.
  No observation, action or descriptor layout changes; fingerprints are unchanged.
  - With The Courier, a bought card, relic or potion is restocked in its own row, as in the
    original's 13 fixed item slots. The restock takes a new choice ID at the bought row's
    index. The shop no longer keeps the bought row as `claimed` and appends the restock.
    Restock identities, prices and RNG draws are unchanged. After a Courier purchase, only
    the row order differs: the restock sits at the bought row's index, not at the end, and so
    do the later shop action candidates. The row count also differs. A shop now has at most
    15 rows, and `offers` needs at most 20 (the Library's card board). This replaces the
    Courier capacity note above. Without The Courier, a bought row stays `claimed` as before.
  - Circlet, the relic offered when the relic pools are exhausted, is priced from the
    original's Special-tier base of 400 in a shop, before the usual jitter and discounts.
    Shop entry and a Courier relic restock used to fault (`InvalidAction`) there. Only paths
    that used to fault change.
  - A Circlet reward row while Circlet is already owned stacks Circlet's counter, as the
    original does. The reward chooser (chests, combat and event rewards, Dig, Calling Bell)
    used to reject it with `InvalidAction`, although the mask offered it. Only that
    previously faulting choice changes.

## 1.0.0 — effective at the merge of PR #103

See the [1.0.0 release notes](docs/releases/v1.0.0.md).

- Freeze the documented Standard, card metadata, `build_info()`, batching,
  Python facade and selected legacy consumer contracts. Add native identity and
  semantic replay pins, Heart pins and Python public-name checks.
- Bound `PublicSearch` worker counts to `1..=256`; reject invalid values with
  `search workers must be 1..=256`. Use weak per-count pool handles and a single
  strong hot pool; a live search holds its selected pool until it drops or `with_workers`
  changes its count, reversing
  M35 P13's per-count residency.
- Fix the M34b diagnostics-only (`m34-diagnostics`) supplied-wound hook so its
  scripted hit cannot kill a target. This is test-only and does not change gameplay or a rules ID.
- Add the supplied from-start Heart acceptance for all four characters at A0
  and A20, plus the ignored A0–A20 supplied sweep.
- Add `a20_checks`, `run_vector_rollout.py`, `standard_combat_vector.py`, and a
  `standard_rollout --policy random|first_legal` option; add example and vector
  fault coverage. Isolate the readiness test's mocked CPU probe so it passes
  with the GPU hidden or visible. Repair the `test_build_metadata.py` probe
  crate, broken since M35 made `build.rs` digest the workspace through
  `src/identity.rs` (test-only).
- Retire Rust examples `m22_same_choices.rs`, `m23_same_choices.rs`, and
  `m24_same_choices.rs`; they depended on stale private receipts. The M21
  example remains historical support.
- Set the project and Python package version to `1.0.0`.
- Publish M36 contract, delivery, performance, profile and release documentation.
