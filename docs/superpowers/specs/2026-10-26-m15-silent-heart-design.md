# M15 Silent A0 through the Heart: character coverage design

Status: planning proposal, awaiting one independent planning review and actual
primary-session user approval. No implementation or acceptance is implied.
Baseline: accepted M14 `a8261aa55e64d9c013f418b37711060003ca50b4`.
Planning Bead: `spire-dxo.37.1`; outcome: `spire-dxo.37`.

## Outcome and evidence boundary

Deliver the [approved Silent M11–15 outcome](2026-10-22-narrow-gym-milestones-design.md#silent-milestones1115-a0):
ordinary Silent card, exclusive relic/potion and character-event coverage at A0,
real key acquisition, Shield/Spear and Heart through the same normal-start Run.
This is a character-complete composition of shared mechanics, not another
19-card profile or a promise that a particular caller wins. The accompanying
[coverage ledger](../../milestones/m15-silent-coverage-ledger.md) is part of this
design; every missing entry remains a completion obligation or an explicit
completion blocker. Reference records are inputs, not executable support.

The [M14 acceptance](../../milestones/m14-silent-act3.md) establishes 42 normal
Deaths with all three Act3 boss entries, six separately supplied full-HP boss
wins/endings, and an inspectable all-key Act4Ready cutoff. Canonical normal
completion is unproven. Neither M14's supplied amendment nor M10's captured
acceptance changes the M15 roadmap gate.

Keep three evidence lanes separate:

1. **Mechanics and ownership:** paired card/item tests, actual encounter entries,
   selections, effects, gains and settlement. Supplied resources are labelled
   fixtures, never normal runs.
2. **Normal integration:** canonical Silent start; actual Neow, generated maps,
   earned resources and key decisions; actual same-visit boss/elite/Heart records.
   A complete trace must demonstrate Ruby/Recall, Sapphire/relic tradeoff,
   Emerald/burning-elite Victory, Act3 ending, fixed Act4 route and Heart settlement.
   No injected deck/keys/floor/HP, enemy weakening or scripted result qualifies.
3. **Tools:** source-identical core/session/replay/batch agreement, including
   unfinished and failure states. Agreement cannot upgrade lane 1 into lane 2.

Genuine Death is valid. A budget error, capacity error, unsupported source or
fault is not Death, truncation or completion. Freeze six normal input rows at
seeds 7/11 and the six boss triples in M14 `FIXED_CASES`, the chosen public caller
and 8192-action budget before the first M15 evidence run. Add a fixed public
key-acquisition objective before that freeze: with Ruby missing choose offered
Recall; with Sapphire missing choose offered Sapphire ahead of a chest relic
(open the unopened chest first); with Emerald missing choose its offered reward
and prioritize a legal public map route through an unvisited burning elite.
Rank such routes by shortest directed edge distance to a public
`burning == Some(true)` elite that can still reach the boss, then smallest next
NodeId; use public edges only. If no such route is reachable, retain the normal
route fallback and record the missing key/reach. After actual all-key boss
settlement choose the offered fourth ending choice; HeartVictory chooses Proceed.
Selection/combat suspension retains its parent; every chosen action must occur
in current legal candidates. Once each key is owned, stop its preference. Other
choices retain existing public policy and first-offered-legal fallback for new
cases, preserving M14's selector bodies and accepted transcript bytes. Freeze
`m15-key-aware-public-caller-v1` and its choice/tie tests before the same six
attempts. This objective is not a winning-player benchmark.
No result-driven policy changes,
seed search, input-case filtering, stronger diagnostic resource search or repeated
plays to force a win. If no fixed run reaches the normal key/Act4/Heart boundary,
report exactly which gate is unproven. Do not close M15 as complete; root may
present a concrete acceptance amendment to the user. No such amendment is proposed
as already accepted, and supplied proof never acquires a normal-start label.

## Approaches and recommendation

The user has already selected one Rust engine and the M10/M14 progression,
encoder, session, replay and scheduler owners. The planner recommends:

| Approach | Benefit | Tradeoff / disposition |
|---|---|---|
| Complete Silent mechanics plus explicit M15 captured producers on the existing Run | Satisfies character breadth and real Act4 integration while keeping published acquisition rules inspectable | Requires full transitive generator behavior; captured reward rarity/shops remain a qualified profile. Recommended. |
| Implement full standard shared generators/events alongside Silent | Establishes wider ordinary-run reachability earlier | Advances unrelated M26–28 work and greatly expands the review surface. Retain for those milestones except dependencies actually reached by Silent mechanics. |
| Extend only M14's 19 cards to Act4 | Small implementation | Fails the explicit M15 character-coverage requirement; rejected. |

This recommendation is a planner decision for review, not an invented user
selection. All actual-character supports used by Distraction, Alchemize and
their descendants must be complete; the captured-producer choice does not permit
a smaller generator pool. Exact unresolved source/runtime dependencies in the
ledger explicitly prevent character-complete acceptance.

## Contract and eligibility

M15 is explicit opt-in `RoomScenario::M15SilentHeart`, Silent A0 only. Start:
70/70 HP, 99 gold, five `StrikeGreen`, five `DefendGreen`, `Survivor`, `Neutralize`,
`RingOfTheSnake`, three empty potion slots, no keys. Use existing short Neow,
fully unlocked creation and Act4 enabled. The public constructor has no resource
overrides. All 27 typed boss-history triples are installed before construction.

The ordinary roster is **75 kinds = 4 Basic + 19 Common + 33 Uncommon + 19 Rare**;
71 nonbasic ordinary offer candidates. Every kind has base and once-upgraded
behavior, legality, visible values and copy/disposition support. Native enum IDs
must be explicit; no `Unknown` fallback. Basics are starter/upgrade/removal
resources, not nonbasic ordinary reward rows. `Shiv` is a generated colorless
Attack, not a green reward. `Parasite` is a permanent curse from existing Writhing
Mass; `Dazed`, `Burn`, `Slimed`, `Wound`, `Void` are encounter-created kinds.
Keep the distinction between permanent admission and temporary creation.

All **11 Silent-exclusive relics** and **three exclusive potions** need native
mechanics, acquisition/use and public representation. There is no established
Silent-only event ID: verify character eligibility of existing event branches,
green basic recognition and gain/removal/upgrades rather than inventing an event.
Full shared event generation remains M26–28. Shared content reached transitively
by a Silent generator is a dependency, not an exemption.

Proposed M15 captured offers reuse the M10 preparation/commit structure:

| Producer | Declared M15 law |
|---|---|
| Ordinary combat, including Act4 elite | One Common and two distinct Uncommons, uniform eligible IDs within rarity |
| Act1/2 boss card reward | Three distinct Rares; actual child then boss chest |
| Shop cards | One Common/Uncommon/Rare at 50/75/150 gold; no duplicate row IDs |
| Ordinary potion reward | Cycle Fire, Strength, Weak, Poison, Cunning, GhostInAJar by local settled ordinal; actual belt checks |
| Shop potion | Same six-kind cycle by captured shop visit, 50 gold |
| Shop relic | Unowned TwistedFunnel at 150; no normal-stock relics sold |
| Boss relic capture | EmptyCage, SlaversCollar, RingOfTheSerpent, HoveringKite, WristBlade; disabled owned/ineligible rows, real children and skip confirmation |

These are proposed capture rules, not standard rarity/pity/price/potion-drop
algorithms. Every nonbasic green kind is eligible by its real rarity; no
unsupported candidate is removed. Distraction and actual potion generation use
their own source-qualified laws instead of these rows. Shops in every act
commit their detached RNG exactly once; existing Act1/2/3 `is_m10` commit gates
must include only the new M15 producer where appropriate.

Use one finite normal relic cursor across chests/elites. The proposed 19-kind
stock is Lantern, Pear, Mango, ToyOrnithopter, Strawberry, Anchor, BloodVial,
Vajra, OddlySmoothStone, BagOfPreparation, BagOfMarbles, BronzeScales, Orichalcum,
SneckoSkull, NinjaScroll, PaperKrane, TheSpecimen, Tingsha, ToughBandages.
This is 13 existing shared normal kinds plus six Silent normal kinds. At most
15 Act1–3 elite offers + three opened chests + one Act4 elite = 19 accepted offers.
Opening consumes the capture even if Sapphire/Proceed skips it. Starting, boss,
shop and RedMask event sources are outside this stock; M15 shops do not consume
future stock via separate acquisitions. Owned entries are skipped without
duplication/refill; source exhaustion stays an explicit operational fault.
Tests must prove that the proposed exclusive relic source tiers and the
19-offer bound fit qualified source facts before enabling this stock. The union
is 28 admitted relic kinds (19 stock + starter, NeowsLament, EmptyCage,
SlaversCollar, RedMask, TwistedFunnel and the three exclusive boss kinds).

## Shared mechanics and interfaces

Extend existing owners; split new family code only where it holds a coherent
responsibility. Do not copy Run, the enemy setup table, damage calculation,
Python facade loops, scheduler or effect draining.

| Owner | Existing seam and proposed change |
|---|---|
| `content/ids/cards.rs`, `content/cards.rs` | Append missing named CardId variants/as_str and actual `CardDefinition` rows; retain old spellings, enum order and metadata. Existing `Catalog::selectable_pool` stays fail-closed for incomplete standard pools. |
| `combat/cards/silent.rs` and family modules | Keep `payload(&State,&LiveCard,Option<u64>) -> Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>` and `public_values(&LiveCard)->Option<CardPublicValues>`; route new families through these owners. |
| `combat/model.rs`, `engine.rs`, `effects.rs`, `hooks.rs`, `powers.rs` | Add only typed work/state needed by missing behaviors; use existing `Effect`, `Source`, `DamagePayload`, full-power registry, selection suspension and accepted-prefix fault handling. |
| `combat/card_state.rs`, `piles.rs`, selection modules | Complete green permanent/generated/stat-equivalent/same-family construction, cost layers, ordered movement and one-resume selectors. Master-deck state remains separate. |
| `items/mod.rs`, `relics.rs`, `potions.rs` | One `CombatItemContext` takes Run inventory into combat and returns actual remaining inventory; relic callbacks and potion obtain/use use the same resolver. |
| `run/mutations.rs`, `rooms/m15_content.rs` (new) | M15 permanent admission, full green rarity supports and detached captures; shared obtain/remove/upgrade transaction, not direct deck edits. |
| `run/rooms/{mod,act3_end,act4,rewards,shop,boss,rest}.rs`, `run/combat.rs` | Add profile checks at existing seams. New `uses_act4(state:&RunState)->bool` means M10 or M15; leave `is_m10` an exact identity test. |
| `spire-env` versioned Run files | Add M15 identity/dispatch/schema tables while retaining one `encode_run_with_schema` and session/replay/batch implementation. |

The implementation plan fixes family interfaces and assertion examples. New
native internal APIs are proposals, not statements that those APIs exist today.

Required mechanics include action-time counts (Finisher/Flechettes/Expertise),
per-instance MasterfulStab/Eviscerate costs, distinct discard/counter/history
checkpoints, automatic all/undersized and optional selectors, delayed Block/
energy/draw/Phantasmal/Blur/Intangible lifetimes, Burst's same-family repeat,
GlassKnife mutation, generated Shiv upgrades and no-draw semantics. Use the
[paired Silent study](../../research/2026-10-19-act1-content-silent.md) at its
qualified static grade; its eight unclosed composition fields remain live
dependencies. Never translate printed text order directly into a scheduler.

Damage and Poison still use the shared resolver. Preserve separate NORMAL,
THORNS and HP_LOSS, Artifact per application, signed debuff state, binary32
rounding and per-hit receipts. Test CorpseExplosion's maxHP callback, Specimen
transfer, split/half-dead/rebirth and last-enemy interruption against actual
shared lifecycle owners. Any new source fact contradicting accepted behavior
must be reconciled explicitly with root; do not silently change M10 rules/IDs.

## Generator and obtain completion blockers

The ledger names exact source and native gaps. In particular:

* Distraction concatenates actual-character nonbasic Skill supports, excludes
  HEALING, then samples one index; no rarity roll. Existing research describes
  an Ironclad run, so derive Silent membership/type/tags from the named
  initialization/tag owners. Use deterministic native order within source rarity
  groups; matching a live Java HashMap order is not required by RNG-001. Do not use the finite M14 registry as its support.
* Alchemize samples an actual-character limited potion proposal before obtain.
  Factory identity is 30 shared plus three Silent kinds; limited generation
  excludes FruitJuice, while a later out-of-combat EntropicBrew can produce it.
  The existing 17/9/7 factory arithmetic is Ironclad-qualified, not a Silent
  rarity assertion. The planning lookup now confirms the Silent additions as
  common Poison, uncommon Cunning and rare Ghost, establishing 17/9/7
  (limited 17/9/6); see ledger G2 for the exact static grade.
* A generated potion is not automatically acquired. Add a queued native
  obtain effect for first-empty-slot insertion, full-belt failure without a
  replacement prompt, Sozu at execution and live belt mutations across
  Victory/Death/fault. The combat belt helper returns `Result<bool,InputError>`;
  the resolver uses the existing `EffectError::Invalid` conversion and retains
  the accepted effect as first fault. Full capacity is `Ok(false)`, not an error.
  The outside-Run adapter keeps its Run error/accepted-fault owner. Reward claims
  keep their separate offer-retention rule. No new inventory cache or delayed
  victory-only gain ledger.
* Attack/Skill/Power potions need complete actual-character filtered supports.
  Ledger G5 assigns named potion/Discovery/initializer source methods and exact
  expected Silent sets:26 nonbasic Attacks,33 nonhealing Skills (Alchemize excluded),
  11 Powers, canonical rarity-group/ID native order and typed optional skip.
  These catalog projections require the named constructor/type/tag/selector
  qualification before enabling the samplers; unavailable exact rules block
  completion. Ironclad28/28/14 counts cannot substitute for Silent sets.
  ColorlessPotion needs the existing study's 34 nonhealing ordinary colorless
  identities and real selectable offers. EntropicBrew, Fairy/Smoke and nested
  generation introduce actual use/ending dependencies. Missing kinds or an
  unavailable exact source rule prevent closure; no finite substitute.

Existing public studies/data are reused. Implementation first resolves only
named static class/method questions using already provisioned tools and an
authorized archive; no broad extraction, game execution or private publication.
If source is unavailable, retain the affected ledger rows as blocked and report
the precise absent rule. An implementer must not fabricate a passing oracle.

## Act4 and native public contract

Use M10's actual fourth ending choice and `PreparedAct4` transition, ordinary
maxHP heal, fixed Rest51 → Shop52 → Shield/Spear53 → Heart54 route and final
Proceed55. Reuse setup keys42/43, existing facing/Surrounded, Heart Invincible,
Beat of Death and reward ownership. Key order remains Ruby/Sapphire/Emerald;
each key awards +5 once. Missing keys/disabled Act4 ends Act3NoKeys with zero
ending bonus. Act4 floor entry awards10; final Proceed awards110 (floor55 +
HeartVictory100), once. Death/EmptyDeck remain distinct -1 terminals.
The M15 caller returns a separate `M15CallerOutcome::HeartVictory` only after
that legal final Proceed produces actual `RunTerminal::HeartVictory` and its
accepted trace receipt. Heart entry/pending victory, no-key ending, Death and
unfinished budget are not that result. Preserve the old CallerOutcome type and
M13/M14 mappings, including M14's rejection of a Heart terminal.

Test poison/Flask/Corpse/Specimen against Shield/Spear Artifact and survivor
facing; Wraith/Ghost Intangible and Dexterity loss against multi-hit Heart;
AfterImage/AThousandCuts/Choke/Burst callbacks against Beat; Poison/THORNS/
NORMAL and Corpse damage against the shared Heart allowance. Keep M10's explicit
headless terminal pruning ruling; unresolved new compositions require a named
source/owner ruling rather than an animation/clock subsystem.

Proposed native identities:

| Contract | New M15 value |
|---|---|
| Profile | `M15SilentHeart` |
| Observation | `spire-gym-m15-silent-heart-observation-v1` |
| Replay | `spire-gym-m15-silent-heart-replay-v1` |
| Rules | `spire-gym-native-m15-rules-v1` |
| Content | `spire-gym-m15-silent-a0-heart-captured-v1` |
| Fingerprint domain | `spire-gym/run-schema/m15-silent-heart/v1\0` |
| Seed/reward | Existing `spire-gym-run-u64-le-v1` / `spire-gym-floor-key-terminal-reward-v1` |

Add `RunSession::new_m15`, `RunBatch::new_m15`, `m15_run_observation_spec`,
`encode_run_m15` and `RunBossHistory::M15SilentHeart{act1,act2,act3}`. Portable
history JSON is `{"m15-silent-heart":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}`
inside existing `boss_history`. Reject wrong profile/partial/extra/unknown history
before construction. Preserve all old identities, fingerprints, setup RNG keys,
default profiles and frozen combat-v1/policy-v3 contracts. M15 does not advertise
the Ironclad combat-policy sidecar (`combat_view_id` remains null).

Use M10 numeric row groups, including `combat_state` facing/Heart allowance,
and append M15-specific vocabulary/selection operations. Publish every admitted
kind and operation, including generated offers, potion-use decisions and
signed power values. Hidden draw contents, pending templates, RNG and future
offers remain private. Occupied/known masks and entitled known-draw aliases keep
existing meanings. Default capacities remain `RunCapacities::m10_default()`;
fail explicitly at capacity, with no paging/truncation/dynamic space.

Native-only delivery is the planner recommendation, matching M12–14. No existing
Python extension is rebuilt or claimed current; Python binding delivery requires
separate authority and source/artifact receipt. Native additions must not break
binding source exhaustive matches, but do not add a Python M15 keyword here.

## Ownership, review and completion

One senior integration owner performs the cohesive plan using bounded Luna/MAX
facts or implementation assistants in independent source materializations and
unique build targets. No overlapping writers or per-task review swarms. Task
`.68` owns encounter setup/catalog audit; its expanded assignment owns all57
defined encounter setups in additive profile-v2, preserving old43 setup keys and
frozen Run selection contracts.
Reconcile its accepted PR before execution; its catalog closure does not itself
change the M15 normal sampler. Do not edit its files in this planning task. `.69` owns README/AGENTS navigation.

The plan has five Review Focus classes: source-dependent generation/obtain,
instance/cost/history ownership, damage/death callback order, key/ending source
provenance, and public/native fault/replay/batch isolation. Each has named tests.
At implementation end use one fresh independent milestone review and explicit
disposition of all findings/declined judgments, plus source identity and smallest
meaningful preservation checks. No count target or cold repeated baseline suite.

Completion requires every ordinary/exclusive ledger row resolved and honest
normal evidence. Unclosed exact source/runtime blockers keep M15 incomplete.
The [implementation plan](../plans/2026-10-26-m15-silent-heart.md) is reviewable
now, but product work waits for root's independent planning review and actual
user approval. No commits, pushes, builds, installs or product changes occur in
this planning task. Retain frozen documents, source-fact inputs and hashes under
independently retained evidence before root dispatches that one review.
