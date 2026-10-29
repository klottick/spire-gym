# M21 Watcher A0 authored combat design

Status: planning proposal, not implementation approval or milestone acceptance.
Planning Bead: `spire-dxo.43.1`; roadmap outcome: `spire-dxo.43`.
Read baseline: `d1fa706638a31a854fdcaeed9a9a4642abf716eb`.

## Outcome, approach, and boundary

Deliver the [M21 roadmap outcome](2026-10-22-narrow-gym-milestones-design.md#watcher-milestones2125-a0)
through finite native authored combats: Neutral/Calm/Wrath/Divinity, Mantra,
Scry, retained and generated instances, and representative power/item timing.
The user selected bounded parallel groundwork on the shared Rust engine.
The finite roster, interfaces and demonstrations below are the author's proposals;
they are not additional user approvals. Root schedules the independent planning
review and obtains actual primary-session approval before implementation.

Use existing character state, effects, powers, piles, card state, selection,
Run ownership, numeric encoding and native sessions. A focused Watcher card module
is appropriate; a Watcher engine, damage/cost/draw implementation, encounter table,
inventory, currency journal or Python facade loop is not. A complete ordinary
Watcher pool would absorb M22–25's character rollout; a stance-only toy would miss
the choice, mutation and continuation risks. The selected middle course is the
finite roster below, with explicit source checkpoints for its real compositions.

Canonical-resource authored entry bypasses Neow selection, map generation and Act1.
Supplied resources are explicitly diagnostic and were not earned. Normal Watcher
progression starts at M22; complete ordinary character/exclusive-item/event coverage
remains M25. No Act4/Heart, Python extension, ML, winning-policy, original-runtime,
original-seed parity or general full-content acceptance follows from M21.

## Accepted source and dependencies

| Fact at the read baseline | Owner and implication |
| --- | --- |
| Watcher starter resources and purple IDs already exist | `content/cards.rs`, `run/engine.rs`; reuse four StrikePurple, four DefendPurple, Eruption, Vigilance, 72/72HP, 99gold, PureWater, three empty potion slots and no keys |
| Neutral/Wrath/Calm and Calm-exit +2 already exist | `combat/characters.rs`; extend this owner, preserving M16's future accepted orb fields and operations |
| Purple starter/Empty card payloads and shared NORMAL damage exist | `combat/cards/mod.rs`, `rules.rs`, `effects.rs`; extend shared calculations, without an interim floor or separate Watcher damage path |
| Selection already suspends one resolver and stores ordered handles | `combat/model.rs`, `selection.rs`, `engine.rs`; Scry is missing, and existing hand-return preflight rejects overflow |
| Generated Miracle and PureWater creation already exist | `card_state.rs`, `piles.rs`, `items/relics.rs`; reuse creation/custody, but baseline dispatch is after opening deal and differs from qualified PureWater pre-draw timing |
| Standalone native session exists, currently Ironclad only | `spire-env/src/combat_training.rs`; add a typed profile branch to its catalog/preparation/encoding, preserving reset/step/token/fault ownership |
| All57 A0 setups are accepted in standalone v2 | `enemies::A0_ENCOUNTER_SETUPS`, `run/training.rs`; reference this inventory, never widen captured Run pools |

Task68/PR53 is accepted at the read baseline. Root reports native2043/0/0;
this planning task did not rerun it and it is not Python proof. M15 shared
cost/copy/history/draw/selection/obtain work is in progress and **unaccepted**.
Its retained facts can guide design; proposed implementation names cannot be
consumed as accepted APIs. Approved M16 design/plan extends the same character,
orb, mutation and encoding owners. Approved M18 and proposed M17/M19 remain
dependency context, not permission to implement M21. Roadmap delivery is after
accepted M20. Before implementation, bind the actual accepted M15–20 source
identities, reconcile affected seams, preserve accepted predecessors, and obtain
root's release under the user's actual M21 approval. Do not rewrite approved
M16/M18 documents or copy unaccepted predecessor code into this branch.

The [paired Watcher records](../../research/2026-10-19-act1-content-watcher.md)
provide local base/upgraded and static facts, with explicit W-CHOICE/W-END and
other composition frontiers. Their original context is off-color Ironclad
acquisition; that context does not define Watcher canonical starting resources.
The [data index](../../../data/README.md) records source grades. Owned archive
SHA-256 is `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`;
upstream build/channel is unverified. Use independently worded rules, never
publish private classes/disassembly, and retain source-grounded rather than
original-runtime-validated claims. RNG-001/TIME-001 remain in force.

## Exact proposed finite admission

Add `content/watcher.rs` with `M21_WATCHER_CARDS: &[CardId]` (28 permanent
kinds), `M21_WATCHER_GENERATED: &[CardId]` (three generated-only kinds),
`supports_permanent(CardId) -> bool` and `supports_generated(CardId) -> bool`.
These are admission facts, not a second rules registry or ordinary reward pool.
All rows support base and once-upgraded instances. Parent upgrade never silently
upgrades a child. Costs and nominal payloads below precede shared modifiers.

| Permanent kind | Cost base/+ | Exact producer witness |
| --- | --- | --- |
| StrikePurple, DefendPurple | 1 | Damage6/9; Block5/8; preserve `strike_p`/`defend_p` identity |
| Eruption | 2/1 | Damage9, then request Wrath |
| Vigilance | 2 | Block8/12, then request Calm |
| EmptyFist | 1 | Damage9/14, then request Neutral |
| FlurryOfBlows | 0 | Damage4/6; actual stance transition requests return of discard instance |
| Prostrate | 0 | Mantra2/3, then Block4 |
| Worship | 2 | Mantra5; upgrade adds inherent Retain |
| Brilliance | 1 | Damage12/16 plus combat Mantra-gained history |
| Devotion | 1 | Post-draw start Mantra2/3 per stack, including its no-existing-Mantra ≥10 branch |
| ThirdEye | 1 | Block7/9, Scry3/5 |
| CutThroughFate | 1 | Damage7/9, Scry2/3, then draw1 |
| JustLucky | 0 | Scry1/2, Block2/3, then damage3/4 |
| Weave | 0 | Damage4/6; qualified Scry notification returns discard instance |
| Foresight | 1 | Pre-draw Scry3/4; its own empty-draw shuffle precedes Scry |
| Nirvana | 1 | Scry notification requests Block3/4 |
| WindmillStrike | 2 | Inherent Retain; damage7/10; each actual retention adds4/5 |
| Perseverance | 1 | Inherent Retain; Block5/7; each actual retention adds2/3 |
| SandsOfTime | 4 | Inherent Retain; damage20/26; retention requests combat-cost reduction1 |
| Establishment | 1 | End-turn discount1 per stack on hand instances with a Retain flag; upgrade adds Innate |
| Meditate | 1 | Return1/2 discard instances with temporary Retain, request Calm, request end turn |
| CarveReality | 1 | Damage6/10, create base Smite to hand |
| DeceiveReality | 1 | Block4/7, create base Safety to hand |
| MasterReality | 1/0 | Upgrade only the qualified creation routes for these three generated kinds |
| BattleHymn | 1 | Start-turn create stacked count of base Smites; upgrade adds Innate |
| MentalFortress | 1 | Actual stance change requests Block4/6 per installed amount |
| Rushdown | 1/0 | Actual transition into Wrath requests draw2 per installation |
| LikeWater | 1 | Pre-end-turn-card hook in Calm requests Block5/7; stack clamp999 |

Generated-only: Miracle costs0, gains energy1/2; Smite costs1, damage12/16;
Safety costs1, Block12/16. All three have inherent Retain and Exhaust on use.
They cannot be supplied as permanent deck cards. Tests may construct their
qualified shared stat-equivalent copies; no general copy/generator pool is added.
The Watcher admission helpers constrain Watcher entry/producers, not shared
encounter output. Existing enemy-produced statuses/curses, Stasis and permanent
Parasite gain retain their accepted owners and encoding. Include those reachable
shared kinds in the new descriptor; they are not additional initial deck cards.

Append missing CardId variants with ordinary snake-case spellings, for example
`flurry_of_blows`, `cut_through_fate`, `just_lucky`, `sands_of_time`,
`master_reality`, `smite`, `safety`. Keep existing spellings and ordering stable.
This finite set excludes the other existing purple handlers from entry admission;
their existing behavior remains supported under its previous contracts.

Real relic admission is exactly PureWater, VioletLotus, GoldenEye,
TeardropLocket, SacredBark, Vajra and OddlySmoothStone. Real potion admission is
exactly StancePotion, Ambrosia, BottledMiracle, BlockPotion, EnergyPotion,
DexterityPotion and AncientPotion. The shared items are finite modifier witnesses,
not an expanded shop/potion pool. Append new native IDs with snake-case spellings.
VioletLotus changes Calm exit; GoldenEye changes Scry construction count;
TeardropLocket requests opening Calm; PureWater retains its existing opening
Miracle owner. Potions use real inventory consumption, choice and creation paths.
SacredBark potency follows the accepted stored-potency owner, not an extra cast.

No generated-to-draw insertion, ForeignInfluence/random offer, Expunger/X capture,
Omniscience/repeat, Vault/Blasphemy, Mark/Judgment/Wallop/fatal-permanent mutation,
Deva/Fasting, run event or exclusive acquisition selector is added. These explicit
cuts avoid claiming closure of their W-RNG/W-TERMINAL/W-CREATE frontiers. They do
not authorize removal or disablement of accepted predecessor content.

## State, timing and ownership

Append `Stance::Divinity` in `characters.rs`. Keep the existing stance as the
sole authority. Add `CharacterState.mantra_gained: i32`; current Mantra remains
the ordinary `FullPowerId("mantra")` amount. No parallel current-Mantra counter.
`CharacterEffect::Stance` remains the shared transition request. Introduce
`WatcherEffect` under `combat/cards/watcher.rs`, dispatched by the existing
EffectKind, for typed Mantra, Scry and return/retention continuation work.
The module constructs shared effects; it never drains a private queue.

An actual changed-stance request not blocked by Cannot Change Stance invokes
power then relic notifications, old exit, assignment, new entry, history and
discard-card notifications at their source checkpoints. Same stance is a no-op.
Calm exit queues2 energy; VioletLotus's old-Calm/new-not-Calm callback separately
queues1; Divinity entry queues3. Preserve those separate requests and ordering.
Calm→Divinity requests both, once. Divinity's start-turn request for Neutral
is queued; it is not an unconditional early synchronous assignment.
MentalFortress, Rushdown and Flurry use actual transitions, never every request.
MentalFortress, Nirvana and LikeWater enqueue direct installed-amount Block;
they do not run the card Dexterity/Frail calculation a second time.
Flurry/Weave move the exact existing instance and respect hand overflow, source
membership checks and queued return order; they do not construct replacement cards.

Extend the one binary32 NORMAL calculator with a stance-aware internal entry,
`full_normal_damage_received_for_stance(base: i32, strength: i32, weak: bool,
stance: Stance, receive: impl IntoIterator<Item=f32>) -> Result<i32, InputError>`.
The existing boolean-Wrath APIs adapt to this same calculation and keep their
contracts. Player outgoing Wrath ×2 / Divinity ×3 occurs at the existing stance
modifier position before receive/final hooks and one final floor. Incoming
NORMAL is doubled only in Wrath. Orb, THORNS and HP_LOSS do not gain these
multipliers. Preserve M16's typed orb damage and existing death/prevention owner.

`GainMantra` constructs the requested power/history at the source checkpoint;
history is not remaining Mantra, a turn count or Divinity-entry count. Existing
Mantra stacking checks threshold once, subtracts10 once and requests Divinity;
9+3 leaves2, 9+23 leaves22. First install does not impersonate that stacking path.
With no Mantra and Devotion amount≥10, request Divinity directly, with no Mantra
constructor history or stored excess. Same-Divinity threshold consumption grants
no further entry energy. Brilliance reads combat history before shared modifiers.
The qualified Mantra constructor uses signed int32 addition for history. Its
stack helper preserves the source amount−1 sentinel, then performs one signed
addition and one threshold check; it is not saturating or modulo arithmetic.

Scry captures only the top min(requested, available) draw instances in top-first
order, without moving them to a second pile. Ordinary Scry never reshuffles.
Unless enemies are already basically dead, notify Nirvana before the empty-pile
exit. GoldenEye contributes2 at Scry construction. Foresight explicitly queues
its own empty-pile shuffle. Reuse `PendingSelection` with
`SelectionOperation::Scry`, minimum0 and maximum revealed count, plus the existing
ordered selected vector. Keep the parent play/phase and queued later clauses
suspended; resumption never repays the parent or repeats its earlier effects.
CutThroughFate draws after Scry; JustLucky's Block/hit wait after it.
Keep revealed-order knowledge distinct from the hidden remainder. Weave callback
delivery must use the qualified source update/checkpoint law rather than assume
one callback per modal UI pass. The retained static pass establishes an opening
discard traversal after the reveal opens, followed by selected-list movement and
discard traversal on each resumed update. Weave bottom-queues a return every time;
delivery checks discard membership and hand<10, so a later request for an already
returned instance is a no-op. The number of resumed duration updates remains
unclosed. A semantic-collapse proof for this finite profile, or the exact missing
caller ruling, is mandatory before prescribing callback counts/order; native
tests alone cannot supply that source proof.

Reuse/extend M15's proposed `SelectionParent::Power { turn, id }` for Foresight
pre-draw selection, with an explicit retained resume phase. Existing ToolsOfTheTrade
post-draw validation is not a general power-selection entitlement. Scry and
Meditate each validate exact live parent, candidate zone, count and selected order.
Source selection appends chosen cards, removes deselected cards, and keeps cursor
navigation separate. Add `CombatAction::DeselectCard { instance: u64 }` only while
an M21 Scry/Meditate choice is live and uncommitted; remove that selected handle
without reordering survivors. Reselect appends at the end. No UI cursor is exposed.
Meditate's undersized auto-return flags every candidate before capacity checks;
selected return flags only candidates actually moved. Unlike generic strict
return, its overflow remains in discard. Do not weaken unrelated return preflight.
No Cancel action is offered for mandatory Meditate or StancePotion; Scry may
confirm zero. Invalid/cross-parent/stale selections mutate no state or RNG.

Add `self_retain: bool` and `retain: bool` to the accepted shared CombatCardData
owner, plus `base_block: Option<i32>` only if M16 has not already added it. Preserve
M11's EndTurn selection semantics by translating its accepted chosen identities
at the shared retain checkpoint, without a second independent retained set.
Ethereal still exhausts; merely remaining in hand does not imply `onRetained`.
Actual retention mutates Windmill/Perseverance and reduces Sands' cost on that
instance. Upgrading after two base retentions gives Windmill damage18 and future
growth5, Perseverance Block11 and future growth3; no retroactive growth or master
deck edit. Establishment is a separate queued current-hand flag scan. Reuse the
accepted mutable combat-cost / turn-cost fields and add the distinct source
`modifyCostForCombat` writer, preserving its positive-turn-cost and zero-turn-cost
branches rather than aliasing `updateCost` or free-play. New combat resets this
state. Source int32 field arithmetic, binary32 modifiers and actual local clamps
are distinct from administrative capacity checks; never invent a blanket999 cap
on Mantra history or retained base stats.

For fixed hand creation, extend the existing generated constructor and insertion
owner. Each child has a fresh combat-local identity, enters hand up to10 and then
discard on overflow. MasterReality's constructor-time check applies to all five
in-scope routes: CarveReality, DeceiveReality, BattleHymn, PureWater and
BottledMiracle use MakeTempCardInHandAction constructors. Capture the check when
that action is constructed, before later copying/insertion. Whether a
just-requested power has resolved before a deferred constructor remains a named
activation checkpoint; do not turn this into a universal insertion-time upgrade.
Generated and retained instances remain in the existing complete settlement
census, including held/play/power/autoplay/Stasis/inactive zones where applicable.
No new permanent-gain or mutation journal is needed for this roster.
Existing encounter gains still synchronize through the accepted Run gain owner.

StancePotion requires a genuine nonphysical stance choice, not a fake card offer.
Propose `CombatAction::ChooseStance { stance: Stance }` and
`StanceChoiceView { options: Vec<Stance> }`, with only offered Calm/Wrath legal.
The source option order is `[Wrath, Calm]`.
Use `PendingStanceChoice { parent: SelectionParent, options: Vec<Stance>,
resume_phase: Phase }`, entitled only to the actual StancePotion parent,
on the existing resolver;
at most one of pile selection, card offer or stance choice can be live. Consume
the accepted potion once at the qualified use boundary, retain the parent after
fault, and never consume again on choice/resumption. M15's provisional
ChooseCardOffer/SkipCardOffer remain card-specific. Ambrosia requests Divinity
without a selector; BottledMiracle uses the qualified base-Miracle creation route.
StancePotion potency is0 and stays0 with Bark. Ambrosia potency metadata is2/4
but combat use reads none of it and queues one transition. BottledMiracle creates
2/4 base Miracles; SacredBark acquisition reinitializes held potion potency.

PureWater is a concrete startup mismatch to reconcile, not an accepted-source
timing claim: baseline `engine.rs` deals the opening hand before CombatStarted,
and its item test expects overflow from an already-full hand. Static source
places PureWater before opening draw. Reuse any accepted upstream staged startup;
otherwise add shared `CombatStartupProfile::{Legacy, M21Watcher}` in the existing
item/entry context (default Legacy), `Hook::CombatStartedPreDraw` and
`EffectKind::OpeningHand { count: usize }` calling the extracted existing opening
deal/draw owner. Only M21 routes the existing PureWater effect through pre-draw
and suppresses its later duplicate dispatch. Legacy/default behavior and tests
remain frozen. There is one producer and one opening-hand implementation.

## Native entry, encoding and replay

Proposed core API in `run/training.rs`:

- `M21WatcherCombatStart::{Canonical, Supplied(AuthoredRunStart)}`.
- `Run::prepare_m21_watcher_combat(start: M21WatcherCombatStart,
  encounter: EncounterId, rng: RunRng) -> Result<Run, RunError>`.

Canonical obtains the existing `Run::new` Watcher A0 resources without choosing
Neow. Supplied validates HP1..maxHP≤999, deck1..128, unique nonzero permanent
instances, upgrades0..1, exactly three potion slots, finite card/item admission,
unique relics and valid equipped counters. Resources already include acquisition
effects; do not reapply them. No hidden hand/draw/stance/power/RNG import is allowed.
Validate encounters by referencing all57 shared A0 setups. Prepare through
`from_validated_combat_resources`, set the existing training encounter/act, return
the owner, then call existing `enter_training_combat` → `enter_owned` → setup →
`Combat::enter_with_item_context`. Returning preparation before effects lets the
session retain startup faults. Preserve existing authored constructors unchanged.

Propose a native typed `M21WatcherCombatEntry { id: String,
encounter: EncounterId, start: M21WatcherCombatStart }` and
`CombatTrainingCatalog::m21(entries: Vec<M21WatcherCombatEntry>)
-> Result<CombatTrainingCatalog, RunSessionError>`. IDs follow existing1..64 ASCII
rules, entries1..256; supplied input is canonicalized without granting draw order.
The internal catalog entry enum gains one M21 preparation branch. Use existing
`CombatTrainingSession::new(catalog, partition)` and its reset/step/close/token,
first-fault and outcome reward owner; no new session or facade loop. A typed-only
catalog avoids changing the old strict JSON contracts. Its canonical descriptor
records Canonical versus Supplied, normalized public resources and encounter.
Catalog schema is `spire-gym-m21-watcher-combat-catalog-v1`; fingerprint domain
is `spire-gym/m21-watcher-combat-catalog/v1\0` (one terminating NUL byte).
The fingerprint is SHA-256(domain bytes followed by existing `canonical_json`
bytes of the **entire** normalized descriptor, including every entry in catalog
order). It is not a selected-entry fingerprint. Reordering a supplied multiset
cannot prescribe draw order; retain the normalized permanent-instance mapping
needed for semantic replay.
The descriptor envelope has `catalog_schema_id`, `profile_id`, and `entries`;
its entries use this catalog owner's normalized entry representation. Add only
an internal bridge in `combat_training.rs`,
`CombatTrainingCatalog::m21_from_descriptor(descriptor: &serde_json::Value)
-> Result<CombatTrainingCatalog, RunSessionError>`. Its shared descriptor codec
strictly decodes all entries to `Vec<M21WatcherCombatEntry>`, calls the same
`m21(entries)` validation/canonicalization/fingerprint owner, and requires the
rebuilt descriptor to equal the supplied normalized descriptor. Keep catalog
entry order and normalized instance mappings. Do not copy catalog admission,
sorting or hashing into replay, or route M21 through the frozen legacy JSON parser.
For explicit M21 transport capacity tests/consumers, add
`CombatTrainingSession::new_m21(catalog: CombatTrainingCatalog,
partition: Partition, caps: RunCapacities) -> Result<Self, RunSessionError>`.
It requires the M21 catalog and delegates to the same initializer/lifecycle;
ordinary `new` uses `combat_policy_capacities()` for this profile. Capacity
configuration is transport metadata, never gameplay or hidden-state input.

Identities: profile `m21-watcher-combat-v1`, content
`spire-gym-m21-watcher-combat-content-v1`, rules
`spire-gym-native-m21-watcher-combat-rules-v1`, observation
`spire-gym-m21-watcher-combat-observation-v1`, replay
`spire-gym-m21-watcher-combat-replay-v1`. Keep the existing standalone outcome
reward and seed/RNG identities. Victory+1/Death−1, otherwise0; terminated only on
actual combat settlement, truncated=false. Budgets are unfinished diagnostics.

`RunEncodingProfile::M21Watcher` joins `encode_run_with_schema`.
`watcher_combat_observation_spec(caps: &RunCapacities)
-> Result<RunObservationSpec, RunSessionError>` and
`encode_watcher_combat(d: &PublicDecision, caps: &RunCapacities)
-> Result<RunObservation, RunSessionError>` live beside the common policy view;
`run_schema_m21.rs` owns the additive vocabulary/descriptor. Preserve Neutral0,
Wrath1, Calm2; append Divinity3 only here. Public combat state adds Mantra-gained
history. Visible combat-card columns add current base Block/damage, combat cost,
turn cost and Retain flags as needed beyond accepted predecessors. Selection rows
include reveal position and selected ordinal (0=unselected, otherwise1-based),
not just membership; stance options use a distinct small `stance_choices` group.
That group has two rows and one `stance` column, occupied only while the choice
is live, in Wrath/Calm order. M21 action rows append a `stance_choice_ref` column
with known1 and row0/1 only for ChooseStance; other verbs have known0 there.
Amounts use signed int32 inside native int64 transport with known/occupied masks.
Candidate row i remains native candidate i; action_mask has shape[caps.actions],
ones exactly on candidates, zero padding, and no filtering/reordering/paging.
Known kept Scry order may be exposed; unrevealed draw order, RNG, family IDs and
resolver cursors may not. Capacity failures retain the accepted owner and fault;
they never truncate legal content or retry to obtain a smaller observation.

Add `CombatTrainingSession::record() -> Result<Option<CombatEpisodeRecord>,
RunSessionError>` and `replay_m21_combat(record: &CombatEpisodeRecord,
caps: RunCapacities) -> Result<CombatReplayReport, RunSessionError>` beside existing
replay owners. The record contains the identities above, descriptor/catalog
fingerprints and the following exact catalog fields (JSON keys equal field names):

- `catalog_descriptor: serde_json::Value`: the full normalized object returned by
  the session's `CombatTrainingCatalog::descriptor()`, including all entries.
- `catalog_fingerprint: [u8; 32]`: the same catalog's `fingerprint()`, serialized
  as a 32-element byte array, matching the existing fingerprint JSON convention.
- `entry_id: String`: the actual resolved catalog ID at reset, including when
  reset used the default first entry; replay always passes the stored ID explicitly.
- `selected_entry: serde_json::Value`: the corresponding complete normalized
  `entries` object, including its ID, encounter, start kind, public resources and
  normalized permanent-instance mapping. This is a redundant selection receipt,
  not a second start authority or a singleton replacement for the catalog.

The record also contains partition/seed/ordinal, existing
RunRecordedStep/RunRecordedFault semantic records, final combat outcome and closed
status. Reuse existing semantic action/error serialization, adding `choose_stance`
only to the new schema (`combat.choose_stance`, `stance` = `calm` or `wrath`,
and `combat.deselect_card` with the public instance);
retain old schema rejection behavior. Give CombatEpisodeRecord `to_json(&self)
-> Result<Vec<u8>, RunSessionError>` and `from_json(bytes: &[u8])
-> Result<Self, RunSessionError>`. Both import/export validate these catalog fields
through the shared bridge. `replay_m21_combat(record, caps)` first validates record
identities and observation capacities/fingerprint, reconstructs the full catalog
through `m21_from_descriptor`, and compares its recomputed `fingerprint()` with
`catalog_fingerprint`. It then resolves `entry_id` explicitly, rejects unknown IDs,
and requires `selected_entry` to equal that catalog's complete normalized entry
object, including ID and instance mapping. Catalog/schema/selection mismatch is
`RunSessionError::IncompatibleReplay`, before session construction, episode
initialization, startup effects or gameplay steps. A changed unused entry with
the old fingerprint must fail; neither opaque trust nor singleton rehash is valid.
Only then construct the same session with the validated **full** catalog and
initialize the recorded seed/ordinal with that explicit `entry_id`, never the
default first entry. Match each recorded semantic action to its currently legal
row, and step that session. It does not replay numeric indices as identities or
implement combat rules. Verify reward/outcome and committed-fault prefix through
the same checkpoint; arbitrary injected delivery errors remain reported metadata,
not fabricated core faults. Extract the current reset preparation into one shared
internal episode initializer accepting the recorded seed/ordinal; replay must not
reset ordinal to0 or simulate prior episodes to reach it. Public reset semantics
stay unchanged. No normal Run replay start, new batch scheduler or
Python transport delivery is claimed.
Record export remains readable after fault/close, is None before first reset,
and returns None for legacy standalone profiles whose replay contract is unchanged.

## Qualification, evidence and acceptance

The retained bounded static report SHA-256 is
`150d6dc1420ce5ed05bbc39cf0e5ca8e8142ea1f6ca6ce52c644e7ade3bbed20`;
its manifest pins named members/disassemblies and reused M15 evidence. Qualified
local checkpoints are:

| Named members | Established fact and remaining boundary |
| --- | --- |
| ScryAction.update, GridCardSelectScreen, Weave.triggerOnScry, DiscardToHandAction.update | Top-first reveal; chosen-list append/remove order; opening and resumed discard notifications; membership/capacity-guarded returns. Duration-dependent update count remains open |
| MeditateAction.update | Auto-return group order/all-candidate flags differs from selected-list movement/only-moved flags; no unresolved click-versus-cursor order |
| GameActionManager.getNextAction, AbstractRoom.update, DivinityStance, ForesightPower, DevotionPower | Later turns call stance/start relics, cards/powers/orbs before appending normal draw, then post-draw callbacks. Initial combat appends opening draw before these start callbacks. Foresight queues its empty-deck shuffle at top, Scry at bottom; Devotion is post-draw. The later EnergyManager.recharge caller remains unlocated |
| DiscardAtEndOfTurnAction, RestoreRetainedCardsAction, EstablishmentPowerAction, LikeWaterPower | Retain moves through existing limbo/restore custody; restore invokes onRetained before hand return/temporary-flag clear; Establishment scans current hand at its action; LikeWater checks player-turn/Calm. Whole caller ordering with Ethereal remains open |
| MakeTempCardInHandAction constructors and the five fixed producers | All check MasterReality at construction, then make/insert copies later. Deferred application versus construction needs its exact supported activation schedule |
| VioletLotus, TeardropLocket, StancePotion, Ambrosia, BottledMiracle, SacredBark | Separate Lotus energy, queued opening Calm, exact Wrath/Calm order and potion/Bark laws above; PureWater's accepted-source startup mismatch is explicit |
| MantraPower constructor/stack and retained-card writers | Source int32 history addition, amount−1 stack sentinel, single threshold; instance-local growth and distinct modifyCostForCombat branches |

Before fixing remaining affected outcomes, qualify only the missing Scry
continuation-count/semantic-collapse ruling, later-turn recharge caller,
callEndOfTurnActions↔AbstractRoom.endTurn/queued Ethereal-retain-Establishment
relationship, and deferred MasterReality application/construction schedule.
Reuse retained M15/M16 facts rather than redo their broad owners. Each blocks
its affected producer test/implementation until ruled; no count/order is supplied
by convenience or inferred from native tests. This is a finite source-before-code
dependency, not a completed source gate or a whole-library audit. Public paired
records plus these qualifications do not close all W-* fields.

The implementation plan freezes a public-data caller and exact scenarios before
outcomes. It requires paired behavior-focused tests, invalid-input atomicity,
accepted-prefix fault retention and real settlement. Keep genuine Deaths, honest
unfinished budgets and separately labelled supplied diagnostics. No seed, policy,
resource or target search after outcomes without actual user authorization; no
automatic transfer of M14's supplied-start amendment.

One final stable preservation matrix follows focused TDD; capture exact accepted
predecessor outcomes and unchanged descriptors on the same source. Record a
workspace check once, revision-qualified failures intact, and meaningful native
entry/session/replay checks. A default Cargo build proves no Python delivery.
Then root schedules one fresh independent final milestone reviewer, with all five
Review Focus classes in the [implementation plan](../plans/2026-10-27-m21-watcher-combat.md).
No per-task review swarm or generic verification framework is introduced.

Retain brief, approved inputs, source facts, frozen candidate copies and SHA/mode
manifests outside Paseo at
`/home/klottick/.local/share/spire-gym/evidence/m21-planning-20261027`.
The managed worktree is not durable evidence. Planning checks only links, paths,
commands, source identities and whitespace including untracked documents; no
runtime/build/install/staging/commit/push/PR/cleanup/archive is authorized here.
