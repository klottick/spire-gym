# M22 finite Watcher A0 Act1 design

Status: **planning proposal; independent review and actual primary approval pending**.
Planning Bead `spire-dxo.44.1`; roadmap outcome `spire-dxo.44`.
The producer choices below are the author's proposals under the standing planning
authorization. They are neither accepted runtime behavior nor implementation-start
approval. Root manages review and release; the author is not the final reviewer.

## Purpose and approach

Deliver canonical Watcher A0 creation, a real short-Neow choice, generated Act1
topology, actual ordinary/elite/boss fights, rewards and room decisions, boss relic
acquisition or confirmed skip, and a healthy nonterminal `Act2Ready` boundary.
Use one Rust Run, combat resolver, inventory, instance/mutation owners, encoder,
native session, semantic replay and batch scheduler. Runtime accuracy is the
objective; a genuine Death is a valid episode outcome.

| Approach considered | Tradeoff and proposed disposition |
| --- | --- |
| Finite Watcher profile on shared progression and M21 mechanics | Selected proposal. Explicit obtainable rows give a reviewable Act1 contract while preserving existing profiles. Requires new profile/schema branches and one small HolyWater extension. |
| Complete Watcher and standard random pools now | Would absorb M23–25 and parts of M26–28, including unqualified generators/events. Reject for M22; the final roadmap scope remains intact. |
| Core-only example with public delivery postponed | Would defer choice/encoding/custody defects and fail the evolving native caller contract. Reject; native session/replay/batch are included. |

A separate Watcher engine, copied Run loop, second encounter table, generation
framework or policy optimizer is outside every alternative. The small new module
holds finite producer data and delegates actual mechanics to existing owners.

## Evidence identity and prerequisites

Planning reads accepted M15 merge
`4d7e5be44fd1ce0167729ef1a4f8c9b17b9c5bdf`, retained independently with 765-file
accepted manifest
`0b7475fd1ffdec7c5919f532961d8ec37995603e1e1cad2a37024afcb268f6c0`.
Reuse that acceptance; this lane runs no builds, tests or episodes. Corrected M15
has 294 focused and 15 integration passes. Earlier full-matrix/full-history and
supplied 85-action Heart-win evidence is pre-correction; all six original Deaths
remain. Canonical normal completion is unproven; no Python or original-runtime
proof is inferred. Dated [work status](../../work-status.md) is orientation.

The exact approved M21 v2 design is SHA-256
`99dd3f7b34af439282fc65864ab7c4183e8dab53a1741d77d53bac8110b864fd`;
its plan is
`15432e23f5d1d4a0c73aaa3efe6fa64018f03f5580af1554c971320f69bc5167`.
They are independently retained in the approved-delivery-v2 package with manifest
`7a838c292eea58b610e3597d93cefc448f64917bc90172c5bd840622548bb99d`.
Their future public paths are `docs/superpowers/specs/2026-10-27-m21-watcher-combat-design.md`
and `docs/superpowers/plans/2026-10-27-m21-watcher-combat.md`; those files are not
present in this M15 checkout. Retained exact approved copies travel with this
candidate; unimplemented symbols in those documents are provisional.

Implementation Task0 must bind actual accepted M16–21 source, reconcile the
approved M21 seams against that source, and obtain root release under actual M22
primary approval. Approved plans or an open execution Bead do not satisfy this
gate. Planning can finish before those sources exist. Preserve M21's four named
source-before-code obligations: duration-dependent Scry semantic collapse/caller;
later recharge caller; Ethereal/retain/Establishment end-turn composition; deferred
MasterReality construction versus application. Consume accepted dispositions when
available, including M16 recharge facts at their exact grade. Native agreement or
a favorable run cannot close a missing source ruling.

Rules are source-grounded, not original-runtime validated. The owned archive
SHA-256 is `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`;
upstream channel/build remains unverified. Reuse M21's qualified static facts and
the [Watcher paired records](../../research/2026-10-19-act1-content-watcher.md),
with grades in the [data index](../../../data/README.md). This lane additionally
inspected only named HolyWater hooks and their replacement callers. Private raw
material stays outside public documents. RNG-001/TIME-001 remain applicable.

## Canonical creation and shared Act1 prefix

`RoomScenario::M22WatcherAct1` is proposed on
`Run::new_room_scenario(RunConfig, CreationProfile, RunRng, RoomScenario)`.
Only Watcher A0, fully unlocked, short Neow (`neow_eligible=false`) is admitted.
Derive resources from `Run::new`/Catalog: 72/72 HP, 99 gold, four StrikePurple,
four DefendPurple, Eruption, Vigilance; all base, ten distinct permanent instances;
PureWater, three empty potion slots and no keys. Do not copy a starter initializer
or import combat state. Normal entry cannot supply cards, HP, stance, powers,
draw order, keys, transition frames or later rooms.

The real short-Neow offers are max HP +7 (72 to79, through the ordinary max-HP/heal
owner) or NeowsLament with its actual counter. The map is absent until the choice
resolves. Full Neow, boss swap and Watcher-specific events are excluded.

Reuse `run/rooms/act1.rs` and the shared map/world owners unchanged in substance:

- Ordinary captures: Cultist, JawWorm, TwoLouse; then repeat LargeSlime,
  ThreeLouse, LotsOfSlimes by the shared ordinary encounter history.
- Elites: GremlinNob, Lagavulin, ThreeSentries with the accepted boss-dependent
  offset and actual elite history. Burning markers, buffs and Emerald use the
  shared eligibility/setup/key owner.
- Bosses: Guardian, Hexaghost, SlimeBoss at floor16, selected by initial unseen
  history before map generation. Initial history for Guardian is
  `[Hexaghost,SlimeBoss]`, Hexaghost `[Guardian,SlimeBoss]`, SlimeBoss
  `[Guardian,Hexaghost]`. Never patch the icon, encounter or map after construction.
- First two question-room visits are UpgradeShrine and Purifier; subsequent
  questions are captured Monster rooms. Shop/rest/treasure resolve real choices.
  Smith/upgrade, Purifier/purge and EmptyCage operate on permanent instances.
  Recall/Ruby, burning-elite Emerald and chest Sapphire remain real optional
  shared decisions; possessing keys does not unlock Act2 gameplay in M22.

These are captured encounters/content on generated topology, not a claim of
standard encounter distributions or original-seed parity. Setup stays in
`enemies::A0_ENCOUNTER_SETUPS`; no M22 copy or 57-encounter Run expansion.

## Exact obtainable card union and producer witnesses

Every acquisition below constructs a base permanent instance through
`run/mutations.rs`; legal Smith/shrine upgrades reach the once-upgraded pair.
The union is **28 permanent kinds: four Basic, six Common, fourteen Uncommon,
four Rare**, exactly the M21 capability set. Admission and producer outputs are
different concepts even though this proposed union deliberately covers that set.

| Producer / actual input | Ordered rows and rarity witnesses |
| --- | --- |
| Canonical starter | StrikePurple, DefendPurple, Eruption, Vigilance; Basic, with multiplicities above |
| Nonboss reward, visit mod6 =0 | EmptyFist (C), MentalFortress (U), CarveReality (U) |
| Nonboss reward, visit mod6 =1 | CutThroughFate (C), Rushdown (U), DeceiveReality (U) |
| Nonboss reward, visit mod6 =2 | ThirdEye (C), Foresight (U), Meditate (U) |
| Nonboss reward, visit mod6 =3 | JustLucky (C), Nirvana (U), Perseverance (U) |
| Nonboss reward, visit mod6 =4 | FlurryOfBlows (C), Weave (U), SandsOfTime (U) |
| Nonboss reward, visit mod6 =5 | Prostrate (C), Worship (U), WindmillStrike (U) |
| Boss card reward | Brilliance, Devotion, Establishment (all R) |
| First merchant card row | BattleHymn (U), LikeWater (U), MasterReality (R), prices75/75/150 |
| Later merchants | MentalFortress (U), Meditate (U), Establishment (R), prices75/75/150 |

Each reward has explicit Skip. `visit` means the complete zero-based
`CombatRecord.visit` captured at real settlement, including elites; pass that u64
to the M22 hook before applying mod6. Do not reuse the legacy `ordinal=visit%3`.
The first/later shop switch uses the existing destination's shop-visit counter
before increment, not current public floor or number of purchases. Rows are
fixed captures and prices, not a standard rarity roll or complete merchant.
All 24 nonbasics have a concrete real producer, including MasterReality only in
the first merchant row. Tests at visits0..11 pin rows3–5 and the second cycle;
no requirement to observe every row in a winning trajectory is introduced.

Generated-only **Miracle, Smite, Safety** are not permanent acquisition outputs.
Their source routes are PureWater/HolyWater/BottledMiracle;
CarveReality/BattleHymn; DeceiveReality, respectively. Parent upgrade does not
upgrade the child. M21's qualified MasterReality constructor-time check applies;
HolyWater uses the same MakeTempCardInHand construction family. Each child gets
a fresh combat identity, actual Retain/Exhaust and hand-capacity overflow to
discard. Generation is not draw, and no child survives as a master-deck gain.
Reachable encounter-produced Dazed, Burn and Slimed retain their shared owners
and vocabulary; they are neither permanent rewards nor new Watcher producers.
No random combat generator, draw-pile insertion, off-color/transform source,
ForeignInfluence or potion factory is admitted. Existing accepted behavior in
other profiles is preserved. Full ordinary Watcher closure remains M25.

## Exact relic and potion unions

The possible relic union is **18 kinds**, not M21's seven-kind authored admission:

| Producer | Exact ordered stock / source grade |
| --- | --- |
| Starter / Neow | PureWater (Starter); NeowsLament (Special, real short-Neow alternative) |
| Elite capture pool | Vajra (C), OddlySmoothStone (C), GoldenEye (R), TeardropLocket (U), Lantern (C), Pear (U), Mango (R), ToyOrnithopter (C) |
| Ordinary chest capture pool | Strawberry, Anchor, BloodVial (all C); one captured relic versus Sapphire, or Proceed |
| First / later shop relic | GoldenEye / TeardropLocket, price150 each, omitted when owned |
| Boss chest menu | HolyWater, VioletLotus, SacredBark, EmptyCage, SlaversCollar (all Boss), then Skip; five explicit captured offers, not the standard three-offer sampler |

Common/uncommon/rare tags are reference taxonomy, not tier-preserving drop
probabilities. Elite and ordinary-chest stock use this **proposed M22-only finite
capture law**: filter owned IDs from the ordered source pool before sampling;
empty gives no relic and no RNG request; otherwise call shared
`RunRng::bounded_scoped_checked` with the remaining length. Append only
`RunRngScope::M22RelicCapture`, spelling `run.m22.relic.capture`; do not modify
any old scope or counter. Key is
`[act, floor, node_id, visit, source_tag]`, where source_tag0=elite,1=ordinary
chest; elite visit is that reward's full CombatRecord.visit, chest visit is
`RunState.combat_progress.entered` at accepted chest opening (the existing next
combat-visit identity). Reconcile that owner at Task0; do not add a parallel counter.

At the inspected M15 source, this checked bounded operation **does make a
bounded request for length1**; its rejection loop may consume multiple tickets,
including rejection of u64::MAX. There is no generic no-draw singleton rule.
An orb sampler's shortcut does not change this Run sampler. Reconcile the
actual accepted implementation at Task0; any changed proposed capture law must
be recorded before outcomes. Empty never calls bound0. Prepare with a cloned
RNG and the actual detached combat inventory for elite rewards, then commit the
prepared capture/RNG once in existing reward preparation/settlement. Chest
prepares once at accepted OpenChest. Inspection, stale/rejected actions and
pre-accept failure consume no authoritative RNG. Captured screens survive
delivery/continuation faults; they never reroll when viewed, declined or resumed.
An owned filter is eligibility, not a postdraw duplicate retry.

This policy replaces the early draft's fixed eight-relic cursor: that sequence
would leave later stock without normal Act1 witnesses. Here every stock ID is
eligible at its first real producer; this is a domain/implementation guarantee,
not seed-search or an obligation to observe every relic in fixed episodes. The
tradeoff is a new explicit finite capture distribution, with no original relic
distribution/parity claim. Empty stock omits the relic and still offers Proceed;
it does not fabricate a relic, silently retry or terminate the episode.

All seven M21 potion kinds are obtainable: combat reward cycle is
`[BlockPotion,EnergyPotion,DexterityPotion,AncientPotion,StancePotion,Ambrosia,BottledMiracle]`
at full `CombatRecord.visit%7`; first/later shop offers StancePotion/BottledMiracle
for50. Exactly three slots, first-empty claim, actual use/discard, full-belt
disabled claim and later claim of the same captured item use existing owners.
No default Fire/Strength/Weak fallback is allowed. Stored potency follows the
shared owner; SacredBark acquisition reinitializes held potency once. StancePotion
uses the actual Wrath/Calm modal choice, Ambrosia one Divinity request,
BottledMiracle2/4 children. No generated potion pool or inventory clone is added.

## HolyWater extension and ongoing custody

HolyWater is the one explicit M22 item extension beyond M21 mechanics, appended
with native spelling `holy_water`. The named static members establish:
`HolyWater.atBattleStartPreDraw` constructs one fixed creation action for three
base Miracles; `canSpawn` requires PureWater. BossRelicSelectScreen's
`relicObtainLogic` routes HolyWater to `instantObtain(player,0,true)`;
`AbstractRelic.instantObtain` replaces an existing indexed slot and invokes
the new relic's equip hook when requested. This is replacement, not append-four
Miracles. No claim about unrelated obtain modes follows.

Extend `items/relics.rs` acquisition/replacement, using the accepted
RingOfTheSerpent/BlackBlood pattern and the actual M21 startup owner. Validate
Watcher, no active combat, PureWater in slot0, no HolyWater duplicate before
mutation. Replace slot0 once; remove only obsolete PureWater bookkeeping.
Boss-menu eligibility requires PureWater, and an inconsistent supplied diagnostic
rejects before replacement. The other four boss relics keep their accepted
acquisition/children: EmptyCage exact permanent selections/removals (including
EmptyDeck terminal), SlaversCollar's real combat context and SacredBark potency.
Skip has confirm/cancel and a retained parent; Proceed waits for child completion.

M22 opts into the **actual accepted M21 staged pre-draw startup** for every fight.
Legacy M15 PureWater post-opening-draw behavior is not source-correct evidence
for this profile. Do not duplicate PureWater or opening draw. Canonical PureWater
creates1 then draws5; HolyWater creates3 then draws5; requested opening10 with
HolyWater leaves3 generated+7 permanent in hand and3 permanent still in draw.
Subsequent actual draw callbacks remain distinct from creation. Teardrop's
opening Calm, VioletLotus's separate exit-energy callback, stance/Mantra history,
Scry suspension, retain/cost growth and cleanup are the accepted M21 shared rules.

Run owns permanent instances, HP/maxHP/gold, relics/counters, belt and keys outside
combat; CombatItemContext owns the detached inventory during combat. Settlement
returns it once, census-checks every original across all zones including held,
playing, powers and inactive ownership, applies genuine gains/mutations through
existing journals and discards generated-only children. New combat resets
stance, Mantra/history, temporary Retain and combat-cost/growth/powers without
resetting earned resources, card upgrades or existing persistent instance state.
No new mutation journal, source-character inference from purple producer color,
or Silent-generation fallback is permitted.

## Transition and reusable M23 prefix

Actual boss Victory settles a CombatRecord before the reward parent opens.
Proceed enters Act1 boss treasure at floor17, then the real captured boss menu,
its selection/child or confirmed skip, then shared `transition::begin/drive`.
Preconditions include live healthy player, nonempty deck, no combat or pending
selection, correct visited boss source and completed chest source. Reject wrong
act/floor/source or uncompleted child before mutation.

Keep the existing `ActTransition` flags `reset_applied`, `heal_applied`,
`initialized`: source reset, ordinary healing through actual modifiers and
successor map initialization happen once. Do not heal before boss acquisition or
again when publishing. Preserve unchanged Run identity, deck instances/upgrades,
gold, relic inventory and counters, belt, keys, selected Neow, prior combat/room
records and RNG; reset only the source owner's act-local state. Retain exact flags,
capture and first fault if healing/map/publication fails after acceptance.

`Act2Ready` is `(act,floor)=(2,17)`, terminal=None, no legal actions, valid zero
action mask, terminated=false and truncated=false. Publish the generated Act2 map
and boss icon with map-only encounter metadata; Act2 enemy play is unsupported.
Ordinary floor/key/ActAdvanced reward events remain; there is no readiness bonus
or synthetic win. New action returns
`UnsupportedBoundary("M22 stops at Act2Ready")` without changing or faulting the
healthy session; reset/record/reproduction/close remain available.

M23 must reuse this exact prefix and producer owner. Its future opt-in continuation
uses the same transition frame and the resulting resources; it must not replay
Neow, reconstruct a starter, regenerate Act1, reobtain/reheal or copy M22 tables.
Compare semantic/public prefix, actual resource/instance state and RNG at the
boundary, allowing intentional profile/schema labels to differ. M22 remains
frozen at its zero-action cutoff. No second generic prefix framework is needed.

## Native delivery and strict compatibility

Add `RunProfile::M22WatcherAct1` to the existing RunSession initializer;
`RunSession::new_m22(config: RunSessionConfig) -> Result<Self,RunSessionError>`
and `RunBatch::new_m22(configs: Vec<RunSessionConfig>, workers: usize)
-> Result<Self,RunBatchError>` delegate existing lifecycle/scheduler owners.
Accept only existing `RunBossHistory::Act1GuardianUnseen`,
`Act1HexaghostUnseen`, `Act1SlimeBossUnseen` for Watcher A0. Reject downstream
history/config pairs before creating Run; preserve M9BossHistory alias and all
old serialized spellings. There is no public supplied-run constructor.

| Identity | Proposed exact value |
| --- | --- |
| Profile | `M22WatcherAct1` |
| Content | `spire-gym-m22-watcher-a0-act1-captured-v1` |
| Rules | `spire-gym-native-m22-rules-v1` |
| Observation | `spire-gym-m22-watcher-act1-observation-v1` |
| Replay | `spire-gym-m22-watcher-act1-replay-v1` |
| Observation fingerprint domain | `spire-gym/run-schema/m22-watcher-act1/v1\0` (one terminating NUL) |

Keep shared Run reward identity `spire-gym-floor-key-terminal-reward-v1`,
seed adapter `spire-gym-run-u64-le-v1`, partition/ordinal and RNG identity.
`m22_run_observation_spec(&RunCapacities)` and
`encode_run_m22(&PublicDecision, Option<&PublicMap>, &RunCapacities)` produce the
existing Result types through `encode_run_with_schema`; `run_schema_m22.rs`
extends the accepted M21 Watcher groups/vocabulary with Run/map/room content.
Use one row encoder and descriptor/fingerprint owner, not a copy of M21 or M12.
The finite catalog is the producer union above plus generated/status kinds,
all actual reachable powers/intents, twelve playable encounters/sixteen enemy
kinds from shared Act1 metadata, and Champ/BronzeAutomaton/Collector as map-only
icons. Descriptor distinguishes capability admission from producer rows; it does
not claim M21's 57 encounters are playable in M22.

Expose stance Neutral0/Wrath1/Calm2/Divinity3, current Mantra and gained history,
public installed powers, current costs/base Block/damage/Retain flags, held and
played-power custody, Scry reveal position/selected ordinal, and real stance-choice
rows in Wrath/Calm order. Public current state is not a constructor's printed cost
or an installed power's original application amount. Candidate i stays row i;
mask shape is `[caps.actions]`, with exact zero padding/known/occupied conventions.
Expose only entitled kept/revealed order; hide unrevealed draws, AI/RNG, internal
instance family IDs and resolver/transition cursors. Raw handles in semantic
reproduction are privileged, never numeric policy features.

Capacity overflow rejects or retains the existing owner as appropriate to the
acceptance boundary. Never truncate offers/piles/candidates or retry a new draw
to fit. Stale/wrong-session tokens, negative/out-of-range indices, invalid nested
parents and wrong profile data are nonmutating before acceptance. After accepted
startup/core/publication work, keep the committed prefix, detached inventory,
publication serial and first structured fault. Observation/export failures are
not Death, empty-success observations, automatic reset or gameplay retries.

Extend existing `RunEpisodeRecord` / `replay_run` and strict JSON codec for M22,
including config/history, schema/rules/content/descriptor fingerprints,
seed/partition/ordinal, actual semantic selections, rewards, terminal and fault.
Reuse M21's semantic Scry/deselect/stance action helpers on the shared parser;
old schemas continue rejecting those verbs. Both typed replay and JSON import
validate exact profile/config/history and identities before Run construction.
Unknown/extra fields, altered rarity/stock descriptors, mismatched schema or
ordinal and modified rewards/outcomes reject. Replay starts the recorded ordinal
through the existing initializer, not earlier episodes or numeric candidate
indices. M21's full authored combat catalog/entry receipt remains its separate
contract; do not transplant or weaken it for normal Run. M22 adds no arbitrary
catalog/supplied-start JSON format. Reproduction/record remain readable after
fault and close; delivery-only errors remain diagnostics, not injected core work.

Batch reset/step is a full-batch operation. Validate every slot/token/index and
reserve result/fault storage before dispatch. A healthy cutoff/terminal slot's
invalid step rejects the whole call with no peer mutation; do not silently compact
or skip it. After dispatch, a failed slot retains its original fault; successful
peer rows whose result cannot be returned retain committed state and indexed
`delivery_lost` errors. Workers1/2 agree with singles in input-slot order.
Heterogeneous character coverage means separate homogeneous batches plus bounded
interleaved single-session M22/accepted-M15 prefixes; M22's constructor rejects
non-Watcher slots. Do not add mixed scheduling or partial-reset APIs to M22.

## Frozen callers, outcomes and review gate

Use accepted `examples/shared/public_routes.rs` production-only route extraction
required upstream by M19. If absent, bind that predecessor delivery before M22's
caller; do not import whole `cfg(test)` example modules or copy route evaluation.
The M22 pure caller consumes only PublicDecision/PublicMap and its own history of
accepted public decisions. Freeze its source/hash before outcomes: existing
public route evaluator with ordinary+elite witness preference; Neow maxHP;
first legal non-Skip card reward; shop buys one first affordable card then Leave
(track only the public accepted purchase for that act/floor); chest
relic before Sapphire; rest below halfHP otherwise Smith; first legal permanent
selection; HolyWater then VioletLotus then SacredBark then EmptyCage then
SlaversCollar at boss menu. Combat uses ascending potion slot, Calm for stance
choice, Scry confirm0, Meditate confirm when offered otherwise first legal
SelectCard (never Deselect), then first legal card play,
then EndTurn, preserving native target order. Fixed tie-breaks are proposal
choices, not a claim of good Watcher strategy. No seed/hidden trace lookups.

Run exactly six canonical cases: seeds7 and11 crossed with the three initial
Act1 unseen-boss histories. Limit32768 accepted actions each; no policy/resource/
seed search after results. Record every observed boss entry and matching actual
CombatRecord, enemy set, legal actions, callbacks, custody, reward and final reason.
Map icon alone is not boss entry. Genuine early Death remains valid and leaves
that boss entry/transition unproven. A nonattacking canonical Guardian/seed7 control
ends in Death if it reaches it within the same budget; budget0/1 and bad-case
controls remain explicit unfinished/input outcomes, not asserted Deaths.

This **proposed M22 evidence scope** permits separately labelled supplied native
diagnostics for missing boss/transition branches if normal cases die. It does not
amend M21 or transfer older supplied-start approvals. Use actual full-HP Guardian,
Hexaghost and SlimeBoss setups with72/72HP,99gold, PureWater,VioletLotus,Vajra,
OddlySmoothStone; deck four Eruption+, four Vigilance+, four EmptyFist+,
four CarveReality+, permanent IDs1..16 in that order; three empty slots. Freeze
the same caller/seed7/budget32768, run each once, retain actual Victory or Death.
This is supplied loadout evidence, never an earned prefix or guaranteed victory.
No strong SearingBlow/250HP fallback is inherited or requested for M22.

Separately, narrowly labelled transition fixtures may start at a valid captured
boss-reward/chest source with a genuine shared combat exit record, or a fixture
explicitly marked as constructed rather than played. Exercise each boss relic
child and confirmed skip through real shared transition code; never add that
fixture to the normal history. Fault injection and small hand/belt diagnostics
likewise disclose supplied state. No enemy weakening, forged gameplay victory or
fixture receipt is counted as a played boss outcome. Lack of normal completion
is reported as **unproven**, even when native APIs and supplied transition branches
pass. Missing actual boss-entry/record evidence is separately listed; no success
gate depends on optimizing the caller.

One final native preservation matrix follows focused meaningful TDD at actual
implementation, with all qualified predecessor failures retained. Native tests
prove native behavior only; default Cargo excludes Python extension delivery.
No install/sync/Python refresh/game execution is part of this plan. Then freeze
full source/docs/evidence with SHA-256 and mode manifests outside Paseo for one
fresh independent final milestone review. Root handles acceptance/publication.
This planning lane freezes the two documents, accepted-source binding, approved
inputs, qualified facts and prose-validation receipts for one independent senior
planning review before actual primary approval. No stage/commit/push/PR, cleanup,
archive, parent closure or runtime work is authorized by drafting this design.
