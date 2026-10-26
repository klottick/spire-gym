# Ironclad combat interface (.66)

The user authorizes implementation now in an isolated accepted-M10 worktree, in
parallel with M11. The senior owns this feature; the parent owns final integrated
acceptance. Routine API decisions below are the senior's choices. No Learn code,
training, dependency changes, commits, publication or frozen-root edits.

## Intent and approach

Provide varied independently resettable Ironclad A0 fights and a reusable public
combat policy view during real normal Runs. Use existing Run/Combat setup,
shuffle, startup, resolver, item custody and settlement owners. Keep combat-v1,
M9/M10 defaults, observations, replay identities and fingerprints unchanged.

Considered: branch the legacy CombatEnv (requires broad pool changes), add reset
overrides to RunEnv (conflates combat and Run lifecycle), or a distinct standalone
facade plus common public view. Choose the third. Common view and sidecars are
additive methods; existing reset/step payloads remain unchanged.

## Public surface

`IroncladCombatEnv(entry_catalog: dict, *, partition="train")`;
`IroncladCombatVectorEnv(num_envs, entry_catalog, *, partition="train")`.
Both support reset/step/close and descriptors. Single reset accepts `seed` and
`options={"entry_id": str}`. Catalog selection defaults to its first entry,
never curriculum RNG. Batch reset accepts explicit seeds and entry IDs, plus an
optional Boolean reset mask; native preflight precedes mutation. Batch step uses
int64 actions and native tokens in one GIL-released operation. Autoreset is
DISABLED: terminal rows retain old final observations, outcomes and reward;
explicit masked reset replaces them. No threaded throughput or Python full-Run
vector claim. Batch runtime faults freeze the batch without retry; diagnostics
identify accepted rows. No automatic truncation/time budget.

`RunEnv.environment_spec()`, `RunEnv.combat_observation_spec()` and
`RunEnv.combat_view()` expose an additive, owned public sidecar. The view contains
`obs`, current token, and lifecycle `info`; the same common numeric descriptor
applies to standalone observations. `combat_view` keeps original decision-local
Run dispatch indices. Outside combat its legal mask is empty and routing is
noncombat. Combat selections include card- and potion-parent ownership; ordinary
Run selectors are noncombat. Call the normal RunEnv.step(index) for dispatch.

## Entry configuration and bounds

Strict JSON-compatible catalog `{"entries": [{"id": str, "encounter": str,
"hp": int, "max_hp": int, "deck": [{"card": str, "upgrades": int}],
"relics": [str], "potions": [str|null, str|null, str|null],
"relic_counters": {str: int}}]}`. Unknown fields fail, bool is not an integer.
IDs are 1..64 ASCII alphanumeric/underscore/hyphen. Catalog 1..256 unique IDs.
HP: 1 <= hp <= max_hp <= 999. Deck: 1..128 cards, canonicalized unordered multiset;
ordinary upgrades 0..1, Searing Blow 0..10. Only the accepted 75 Red cards; no
foreign colors, debug/generated/status/curse inputs. Full public catalog method
lists exact IDs, upgrade limits, encounters and item IDs from the finite profile.
Encounter fixes act and ordinary/elite/boss context; no caller-supplied enemy
stats, intent, burning flag or hidden state. Expose the 43 accepted A0 native
setup encounters only after each passes actual entry/publication tests. A0 and
Ironclad are fixed. No complete standard pool or original-runtime parity claim.

Items are already-owned resources, not acquisitions: HP/maxHP/deck already
include prior Strawberry/Pear/Mango/EmptyCage effects. No acquisition hook repeats.
Allowed relics are finite native Ironclad/shared combat items, excluding the three
foreign starters; duplicate relics and Burning Blood + Black Blood reject.
Potions are the 13 native implemented kinds. Exactly three slots, duplicates
allowed. Fresh default Neow's Lament counter is 3; optional typed override for
that equipped relic only, values 1..3 or -2 (spent). Other public supported
counters derive from startup. No arbitrary snapshots/cursors/draw order/RNG imports.
Canonical card/relic ordering makes array order unable to prescribe draw order.
The independent seed/partition/episode ordinal enter Gym's existing RNG adapter.

## Common numeric view and identities

New `spire-gym-combat-policy-view-v1` uses M10 vocabulary/feature meanings for
player, permanent_cards, combat_cards, enemies, powers, relics, potions,
selection, selection_counts, actions, combat_state and known_draw. Exclude full
Run context/map/offers/orbs. Keep existing known and occupied masks, -1 unknown
references, native-endian owned C arrays. Row-group vocabulary retains its M10
IDs; action row equals original dispatch index. Actor input contains no seeds,
RNG, raw instance handles or future hidden draws. Only entitled known draws.
Finite common capacities published by the descriptor; capacity exhaustion is a
retained fault, never truncation/paging. Public resources survive settlement;
terminal actors may be absent after owner settlement, identified by lifecycle.

`environment_spec()` publishes profile/schema/fingerprint/content/rules/reward,
seed-adapter/RNG/combat-view IDs and entry-catalog fingerprint. Standalone profile
`ironclad-combat-v1`; content `spire-gym-ironclad-combat-content-v1`, rules
`spire-gym-native-m10-rules-v1`, reward `spire-gym-combat-outcome-reward-v1` (+1
Victory, -1 Death, 0 otherwise). Run reward remains floor/key/terminal unchanged.
These are ordinary metadata, never privileged replay output. Provenance includes
mode/profile/partition/catalog IDs; seed/episode counters stay metadata, not
features. No standalone semantic replay promise in v1; record effective seeds,
catalog fingerprint and action indices externally for reproducibility.

## Lifecycle and ownership

Standalone natural combat end is environment termination. Victory in a normal
Run is a combat boundary while the Run continues. Sidecar lifecycle reports
active, selection ownership, entered/settled counts and a sticky last exit keyed
by visit (consumer observes each increment once), outcome, HP before healing and
HP after cleanup. Numeric view supplies postsettlement current/max HP, potion
slots, equipped relics/public supported counters and permanent deck. Reward
selection afterward is not part of that checkpoint. A new combat has its own
visit; do not infer wins from phase changes or HP. Death remains distinct from
fault, unsupported input, close and explicit reset abandonment.

Inputs reject before dispatch; accepted runtime/export faults retain their owner
and first cause and disallow reset/step/retry. Invalid action/stale token is
recoverable. Every returned array owns its data independently of future steps,
resets, close and other outputs. Single and batch never implicitly replace a
final. Run sidecar cannot alter old observation fingerprints or replay records.

## Delivery and review

TDD through real owners: all supported entries, seeded multiset/shuffle, startup
hooks, varied cards/upgrades/HP/items, selectors/potions, victory/Death/cleanup,
Run routing/boundaries/resources, masks/references, stale tokens, capacities,
export failures, batch final/masked-reset/preflight and owned arrays. Preserve
accepted M10's seven historical failures in this baseline, then preserve M11's
one-failure result after parent integration. No duplicate Silent work.

Build source-matched extension offline with existing locked Gym dependencies in
this worktree only; report loaded artifact/revision/dirty state and exact tests.
One fresh independent Luna/MAX feature review on materialized/hashverified bytes;
parent accepts integrated source. No final Learn READY callback until accepted.

## Enduring shared ownership requirement

The user explicitly requires later Silent progression to reuse Ironclad's maps,
rooms, enemies, bosses, rewards and transitions, without copied implementations.
`.66` shares the internal character-aware validated resource builder, existing
Run entry/settlement, native enemy setup metadata/RNG keys, resolver/startup,
versioned Run public encoder, and Python scalar/token/numeric packaging helpers.
Only current profile validation/content/identity is Ironclad-specific. Future
profiles extend this validation/catalog seam and existing transport mechanics;
this ticket neither adds a Silent Gym profile nor starts M12 product work.

Batch clarification: every row must be nonterminal before a step. A terminal row
rejects the entire step before any dispatch. Masked reset replaces only selected
rows; ongoing rows keep their exact state/token. Last-exit HP/visit metadata is
sticky; numeric inventory/deck/maxHP is current. Consumers must snapshot those
arrays on the settled-count transition, before subsequent reward/map actions.
The sticky metadata and later current resources are not one historical snapshot.

## Observed vocabulary gap and opt-in normal-Run policy profile

Actual all75 base/upgraded publication exposed Flex's public `lose_strength`
power missing from frozen M10 v2. Preserve v2 exactly. New
`m10-ironclad-policy` / `M10IroncladPolicy` uses observation-v3/replay-v3 with that
category appended, the same M10 core creation/encoding/replay/settlement owners,
and unchanged M10 rules/content/reward. Common view-v1 includes the actual new
vocabulary. Old/new profile/schema/fingerprint mixtures reject. No broad claim
that legacy v2 can publish every Red power. All75 numeric plays and actual Flex
application/expiry/seeded action replay must pass; new policy-Run semantic record
roundtrip/preservation tests are separate.

Review fixes: all RunDecisionToken owners must allocate from the same checked
namespace, preventing cross-mode owner confusion. Public diagnostics reuse the
existing bounded Run fault projection and never Debug-stringify retained queues,
selection internals or RNG cursors. Both are verified through actual native
public methods, including cross-mode batch/reset cases and a real Feed/max-HP
runtime fault after an accepted prefix.
