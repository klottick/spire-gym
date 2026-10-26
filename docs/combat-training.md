# Ironclad combat training interface

Feature `.66` provides two modes over the authoritative Rust engine. Source is
based on accepted M11 plus the isolated `.66` delta; parent integration and final
feature acceptance are separate from this guide. No Learn installation/training,
complete standard pools, original-game RNG/runtime parity or strategy win claim.

```python
from spire_lab.gymnasium import IroncladCombatEnv, IroncladCombatVectorEnv, RunEnv

catalog = {"entries": [{
    "id": "sentries", "encounter": "three_sentries", "hp": 64, "max_hp": 80,
    "deck": [{"card": c, "upgrades": u} for c, u in [
        ("immolate", 1), ("armaments", 1), ("warcry", 1),
        ("strike", 0), ("defend", 0), ("whirlwind", 0)]],
    "relics": ["burning_blood", "anchor"], "potions": ["elixir", None, None]
}]}
env = IroncladCombatEnv(catalog, partition="train")
obs, info = env.reset(seed=11, options={"entry_id": "sentries"})
# Choose ONLY a current legal candidate; action index means a row in actions.
action = int(obs["action_mask"].nonzero()[0][0])
obs, reward, terminated, truncated, info = env.step(action)
identity = env.environment_spec()       # ordinary public metadata
support = identity["support"]          # exact finite input lists and limits
schema = env.observation_spec()        # fields, vocabulary, masks and references
normalized_catalog = env.catalog_descriptor()
env.close()
```

`IroncladCombatEnv(entry_catalog: dict, *, partition="train")` supports manual
`reset(*, seed=None, options=None)`, `step(action: int)`, `close()`, and
`diagnostic()`. Seed is a built-in integer in `0..2**64-1`; explicit seed resets
episode ordinal to zero, while `None` after reset advances the ordinal on the
retained seed. First `None` obtains entropy. Catalog entry selection is explicit,
defaulting to its first entry; no curriculum sampler. Partition is `train`,
`selection`, `final` or `test`. Single rewards: +1 Victory, -1 Death, otherwise0.
There is no built-in time-limit truncation; an external budget remains unfinished,
never converted into a loss or natural combat end. No semantic replay API is
promised for this new profile; consumers can record catalog fingerprint, seeds,
episode ordinals and current candidate indices themselves.

## Validated public construction

Catalog: strict `{"entries": [...]}` with 1..256 unique entries, at most1MiB JSON.
Entry IDs are 1..64 ASCII alphanumeric/underscore/hyphen. All fields in the example
are required; optional `relic_counters` accepts only equipped `neows_lament`,
values `1,2,3,-2`, default3 (spent=-2). Unknown fields, numeric booleans,
unsupported content/combos and hidden-state fields reject. HP:
`1 <= hp <= max_hp <= 999`; 1..128 cards; ordinary upgrades0..1 and SearingBlow0..10.
Deck and relic order are canonicalized; input cannot prescribe draw order.
Exactly3 potion slots; empty is `None`, repeated potion kinds are valid. Relics
are unique; BurningBlood+BlackBlood rejects. Resources are ALREADY OWNED: supplied
HP/maxHP/deck include past acquisition effects. Strawberry/Pear/Mango/EmptyCage
acquisition is not reapplied. Gym owns shuffle, startup, HP/intent setup and RNG.
No hand/draw/queue/seed-in-entry/instance-handle/private-snapshot import exists.

All75 accepted Red card IDs are supported. Obtain exact names and per-card upgrade
limits from `environment_spec()["support"]["cards"]`; registry/reference rows are
not executable-support promises. Generated statuses/curses can arise naturally
but cannot be supplied in this entry profile. Finite relics are the native
shared/Ironclad combat items (30), excluding the three foreign starter relics.
Potions (13): fire_potion, blood_potion, fruit_juice, block_potion, energy_potion,
strength_potion, swift_potion, weak_potion, fear_potion, dexterity_potion,
ancient_potion, elixir, heart_of_iron. Only public supported counters are exposed;
unknown/N/A is distinct from known zero. Internal listener bookkeeping is not
an actor feature.

The43 native A0 encounter setups fix act and ordinary/elite/boss context:

| Act | Accepted encounter IDs |
| --- | --- |
|1|cultist, jaw_worm, two_louse, small_slimes, large_slime, lots_of_slimes, three_louse, gremlin_nob, lagavulin, three_sentries, guardian, hexaghost, slime_boss|
|2|spheric_guardian, three_byrds, centurion_and_healer, snecko, chosen_and_byrds, three_cultists, gremlin_leader, slavers, book_of_stabbing, champ, collector, bronze_automaton|
|3|orb_walker, three_shapes, spire_growth, transient, four_shapes, maw, sphere_and_two_shapes, jaw_worm_horde, three_darklings, writhing_mass, giant_head, nemesis, reptomancer, awakened_one, time_eater, donu_and_deca|
|4|shield_and_spear, corrupt_heart|

These are finite accepted native setups, not complete standard encounter pools.
No burning-elite or arbitrary enemy override. The existing project RNG-001 and
TIME-001 qualifications remain. Runtime supported numeric/queue/RNG limits remain
finite and fail explicitly.

## Same combat view inside a normal Run

```python
run = RunEnv("ironclad", 0, profile="m10-ironclad-policy", partition="test")
full_obs, info = run.reset(seed=11)  # ordinary canonical Run, no resource override
view = run.combat_view()
assert run.combat_observation_spec()["fingerprint"] == schema["fingerprint"]
if view["info"]["routing"] in ("combat", "combat_selection"):
    action = int(view["obs"]["action_mask"].nonzero()[0][0])
else:
    action = int(full_obs["action_mask"].nonzero()[0][0])  # external fixed controller
full_obs, native_reward, run_terminated, run_truncated, info = run.step(action)
view = run.combat_view()  # original Run dispatch index, same current token
run.close()
```

The common view contains only descriptor-defined public combat/resource arrays.
No policy, prediction, reward shaping, encoder or noncombat controller is in Gym.
RunEnv's old reset/step outputs, reward, termination, profiles and fingerprints
remain unchanged. `environment_spec()` exposes normal Run provenance without
requiring a privileged record. Existing default remains M9. Use the explicit `m10-ironclad-policy` profile for
broad combat-policy collection: its `M10IroncladPolicy` identity uses
`spire-gym-run-observation-v3` and `spire-gym-run-replay-v3`. It shares original
M10 gameplay/content/reward/rules, adding the public `lose_strength` category for
Flex. Original `m10-ironclad-heart` remains v2 and rejects that unsupported public
payload. New/old replay schema/profile/fingerprint mixtures reject before play.

Sidecar `combat_view()` returns owned `obs`, a decision-local native `token`, and
metadata `info`: `active`, `routing` (`noncombat`, `combat`, `combat_selection`),
`selection_owner` (`card`, `potion`, or None for this Ironclad profile), active
visit, entered/settled counts and sticky `last_exit` with visit/encounter/outcome
and HP-before-healing/HP-after-cleanup. Combat potion selections belong to the
combat policy. Ordinary Run selectors belong to the noncombat controller; their
common-view action mask is empty. Keep tokens/visit/routing/seed provenance outside
actor features. Do not infer outcome from phase, HP, or reward.

A normal Run combat victory increments settled count without terminating or
resetting the Run. Learn owns any separate combat-task boundary. Snapshot current
numeric resources WHEN settled count changes: current/maxHP, potion slots,
relics/public supported counters, and permanent deck IDs/upgrades/occupancy.
`last_exit` stays sticky across later reward/map actions, but these arrays always
describe CURRENT resources. Do not combine stale exit metadata with later arrays
as a historical checkpoint. Reward collection can change inventory afterward.

## Common schema and dispatch

Schema/view ID `spire-gym-combat-policy-view-v1`; obtain the exact fingerprint
from `combat_observation_spec()` / `observation_spec()` (also recorded in the
delivery packet).
The shared Run encoder supplies the same meanings and vocabulary IDs, with
`lose_strength` appended in the new policy transport. Original Run v2 stays frozen.
Each values group has matching int8 `*_known` and row groups have int8
`*_occupied`; values are int64, native-endian owned C arrays. Padding is zero /
known0 / occupied0. An unavailable reference in an occupied row is -1 / known0.
Empty potion slots are known1/occupied0/kind0. Hidden draw cards have no rows;
only entitled known draws have references. Terminal actor groups may be empty
after settlement, while resources remain available.

| Values group | Shape | Meaning |
| --- | --- | --- |
|player|[12]|HP/maxHP/gold/Block/energy/maxEnergy/turn/draw-count/stance/focus/HP-before/HP-after|
|permanent_cards|[128,2]|card vocabulary and permanent upgrade count|
|combat_cards|[512,18]|visible zone/position/kind/upgrades/cost/permanent and Stasis refs/selection/current public card values|
|enemies|[16,12]|public species/HP/Block/intent/damage/actor flags/slot/facing/hit count|
|powers|[512,4]|owner group/row, power vocabulary, signed amount|
|relics|[32,2]|equipped kind/public supported counter|
|potions|[3,1]|belt kind by slot|
|selection|[128,4]|candidate source group/row/eligible/selected|
|selection_counts|[6]|required/selected/minimum/maximum/confirm-available/operation|
|actions|[1024,8]|verb/source-group/source-row/target-group/target-row/choice/scalar0/scalar1|
|combat_state|[4]|public facing/surrounded/Heart allowance/cap|
|known_draw|[512,3]|entitled end/offset/combat-card row|
|action_mask|[1024]|exact current legal dispatch indices|

Use descriptor `columns`, `reference_columns` and `vocabularies`, not enum
ordinals or guessed offsets. M10 row-group IDs remain stable: player1,
permanent_cards3, combat_cards4, enemies5, potions6, selection8, combat_state9, known_draw10. Powers/relics are feature groups;
they are not candidate source-group categories.
Combat verbs: use_potion6, play_card8, select_card9, end_turn10,
confirm_selection11. `actions` row IS the original native dispatch index, with no
filtering/reordering. Source/target references are known only where applicable;
scalar0 is potion slot for potion actions. Target group5 references enemy rows.
Never step an old candidate index against a new decision. Common capacities are
finite; overflow is a retained fault, never truncation/paging.

Standalone profile `ironclad-combat-v1`; content
`spire-gym-ironclad-combat-content-v1`; rules `spire-gym-native-m10-rules-v1`;
reward `spire-gym-combat-outcome-reward-v1`; seed adapter
`spire-gym-run-u64-le-v1`; RNG `spire-lab-rng-hmac-sha256-v1`.
`environment_spec()` also includes mode/partition, common-view ID and canonical
entry-catalog fingerprint. M9/M10 expose their own existing content/reward/rules
identities through the additive method; these are NONPRIVILEGED metadata.

## Native batch and final ownership

```python
import numpy as np
batch = IroncladCombatVectorEnv(2, catalog, partition="test")
obs, info = batch.reset(seed=[11, 12])
actions = np.array([np.flatnonzero(row)[0] for row in obs["action_mask"]], dtype=np.int64)
obs, rewards, terminated, truncated, info = batch.step(actions)
if terminated.any():
    obs, info = batch.reset(options={"reset_mask": terminated.copy()})
batch.close()
```

B=1..256. One GIL-released native serial batch operation; no Python loop of native
sessions, threaded scaling or Python full-Run vector claim. Numeric observations
are stacked `[B,...]`; rewards float32[B], bool terminal/truncation flags;
`info["slots"]` owns one lifecycle dictionary per row. Scalar seed expands as
`base+slot`, rejecting u64 overflow before dispatch; explicit length-B seed lists
are supported. `options` permits only length-B `entry_ids` and NumPy bool[B]
`reset_mask`. Initial reset must initialize every uninitialized row. Masked reset
preserves unselected state/token exactly. All-false reset is a read-only refresh.

Autoreset is DISABLED. Every row must be active/nonterminal before step; one
terminal row rejects the whole step before any dispatch. Reset terminal rows
explicitly before next batch step. Returned terminal obs/reward/info belong to
the OLD episode; there is no replacement row/final_obs alias. All arrays and
info are independently owned across calls, reset, close and caller mutation.

Invalid types/ranges/tokens/actions/reset requests reject before any dispatch and
are recoverable. Native runtime, capacity or accepted-output packaging failures
retain first fault and accepted counts, freeze the session/batch, and prohibit
reset/step/retry. A batch failure can follow accepted earlier rows; inspect
`diagnostic()` per-slot counts/committed flags, never replay uncertain actions.
Faults are exceptions, never Death/reward/truncation. A panic can leave an
uncertain partial attempt beyond the known completed prefix; counts are not
permission to replay that attempt. Close is idempotent and
retains diagnostics. Use a new owner after investigating a fault.

## Shared implementation and extension build

`Run::from_validated_combat_resources` is the private character-aware resource
builder after each profile validates. Entry/startup/item custody/settlement use
`Run::enter_owned`, `enemies::setup`, `Combat::enter_with_item_context`, shared
resolver and `Run::settle_combat`. Setup metadata lives once in
`enemies::A0_ENCOUNTER_SETUPS`; table order preserves existing RNG keys. Common
view selects existing Run encoding groups rather than copying the encoder.
PyO3 scalar/token/numeric/descriptor packaging helpers are shared in `run_env.rs`.
Later character progression must extend these owners/content/profile seams,
not copy Ironclad maps/rooms/rewards or mechanical transport loops. Current public
entry validation remains Ironclad A0; no Silent Gym/API or M12 start is implied.

Use existing locked dependencies, no dependency sync or installation required:

```sh
CARGO_TARGET_DIR="$PWD/target" TMPDIR="$PWD/.superpowers/gym66/scratch" \
CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true \
/path/to/existing/gym-venv/bin/maturin build --interpreter /path/to/existing/gym-venv/bin/python \
  --out target/gym66-wheels --locked --offline
```

Build output is a wheel; source acceptance alone does not rebuild an extension.
For isolated source verification, extract only its `spire_lab/_native*.so` into
this worktree's ignored `python/spire_lab/`, use this worktree's `PYTHONPATH` with
the existing interpreter, and check `_native.__file__`/`build_info()`. Do not alter
M10's environment/binary or install into Learn. Delivery packet records exact
artifact/source hashes, revision/dirty state and observed tests separately from
parent integration/acceptance. Build code is from an uncommitted M11+.66 delta;
HEAD alone does not identify those bytes.

Current descriptor fingerprints (exact current native defaults):

- Common view-v1: `02009686282c592f971eece3d5853eb54f2a482dde9dae1d0cf380ecdaf8fd78`.
- Opt-in policy Run-v3: `30905db1b7a520482d3ea8aa2e63ddb7f78b7c081de89d87268151f29cf4c7f8`.
- Frozen M10 Run-v2: `7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db`.
- Frozen M9 Run-v1: `0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999`.

All owners using RunDecisionToken share one checked owner-ID allocator. A token
from a different Run/standalone/batch owner rejects before dispatch, including
after reset. Tokens remain opaque transport metadata, unrelated to gameplay RNG.
Public fault diagnostics expose bounded error/checkpoint and accepted-count
metadata; they do not stringify retained private queues, selections or RNG
cursors. The same original error remains stored by its owner, and no reset or
retry is permitted after an accepted runtime/export fault. A real bounded
runtime-fault test uses Feed at the supported max-HP ceiling: its accepted fatal
hit cannot apply an overflowing permanent increase. This is an operational fault,
not Death. A batch test verifies an accepted first row and failing second row
both remain retained; no uncertain effects are replayed.
