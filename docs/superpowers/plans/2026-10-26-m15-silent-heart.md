# M15 Silent A0 Heart Implementation Plan

> **For agentic workers:** Use the preserved senior/native integration method,
> with bounded Luna/MAX assistants in independent source materializations and
> unique build targets. Read the design and ledger before executing tasks.
> This planning task authorizes no implementation. One fresh independent plan
> review and actual primary-session user approval precede product work; one fresh
> final milestone review follows implementation. No per-task review swarm.

**Goal:** Complete ordinary Silent A0 mechanics and exclusive content on the
shared Run, with real normal-start keys, Act4/Heart and honest evidence limits.

**Architecture:** Extend existing card families, item custody and M10 Act4 owners;
add one explicit M15 profile and versioned vocabulary over the existing encoder,
session, replay and batch scheduler. Captured producers admit the complete green
roster; source-specific generators retain their complete actual-character supports.

**Tech Stack:** Existing Rust 2024 workspace, locked Cargo dependencies, native
tests and existing static-reference tooling. No new dependencies or framework.

**Spec:** [M15 design](../specs/2026-10-26-m15-silent-heart-design.md) and
[coverage ledger](../../milestones/m15-silent-coverage-ledger.md).

## Global Constraints

- Baseline: accepted M14 `a8261aa55e64d9c013f418b37711060003ca50b4`.
- Ordinary roster: **75 kinds = 4 Basic + 19 Common + 33 Uncommon + 19 Rare**;
  71 nonbasic ordinary offer candidates. Do not drop missing definitions.
- Start: 70/70 HP, 99 gold, five `StrikeGreen`, five `DefendGreen`, `Survivor`,
  `Neutralize`, `RingOfTheSnake`, three empty potion slots, no keys.
- Preserve all old identities, fingerprints, setup RNG keys, default profiles and
  frozen combat-v1/policy-v3 contracts. Add enum arms at the end; do not renumber.
- One authoritative engine, resolver, Run, encoder and scheduler; no copied loops.
- Default capacities remain `RunCapacities::m10_default()`; explicit capacity
  faults, no truncation/paging or dynamic spaces.
- Fixed normal evidence: seeds7/11, M14 `FIXED_CASES` six triples, 8192 actions.
  Freeze caller/source before play; genuine Death is valid; no tuning/seed search.
- No injected normal-start resources or outcomes; supplied mechanics/ending
  proof remains separately labelled. Unproven normal traversal prevents closure
  without an actual primary-user acceptance amendment.
- No installs, sync, existing Python artifact refresh, game execution, broad
  private archive extraction/audit or private-source publication.
- No commits/pushes under this planning authority. Later publication requires
  current primary-session authority, explicit path staging and cached diff check.
- Reconcile accepted encounter `.68` changes before implementation; do not
  duplicate that audit or assume it expands normal Run pools. Preserve `.69` ownership.

## Review Focus

1. Actual-character generators and full-belt/nested obtain: complete supports,
   source-specific sampling and real inventory lifetime; no finite replacement
   or invented full-belt prompt. Tasks1/8.
2. Per-instance costs, copy families, discard/draw/play history and suspended
   choices: source timing, exactly-once payment/movement/resume and reset. Tasks3/5.
3. Signed powers and per-hit/death callback compositions: independent Artifact,
   Poison, Specimen/Corpse, Intangible and actual Heart allowance/Beat order. Tasks4/6/7/9.
4. Key/Act4/ending provenance: same-visit earned source, once-only reward and
   real live inventory; missing-key/Death/fault cannot impersonate completion. Task9.
5. New public offer/selection/Act4 rows under old-profile isolation, hidden-data
   entitlement, stale tokens, capacity/first-fault and ordered replay/batch. Tasks8/10/11.

## File and integration map

Source paths below are relative to `crates/spire-core/src/` unless prefixed.
Use existing modules; the listed new files contain coherent missing families.

| Unit | Files and responsibility |
|---|---|
| IDs/registry | `content/ids/{cards,items}.rs`, `content/{cards,items}.rs`; append explicit identities and correct actual metadata, keep standard incomplete-pool contract |
| Card dispatch | `combat/cards/{mod,silent}.rs`; retain shared dispatcher and finite old behavior; new `silent/{direct,history,powers,state,costs,generation}.rs` only where the family needs them |
| Shared state/lifecycle | `combat/{model,card_state,piles,selection,engine,effects,hooks,powers,view,command}.rs`, existing `cards/silent/{discard,retain,copy,poison,shiv}.rs`; one queue/custody owner |
| Item work | `items/{mod,relics,potions}.rs`, new `items/silent.rs`, `items/potion_generation.rs`; item callbacks, complete potion factories/obtain |
| Run content | new `run/rooms/m15_content.rs`; extend `run/{mod,mutations,combat,rng}.rs`, `run/rooms/{mod,act1,act2,act3,act3_end,act4,rewards,shop,boss,rest,events}.rs` only at existing seams |
| Native transport | new `crates/spire-env/src/run_schema_m15.rs`; existing `lib.rs`, `run_{schema,observation,session,replay,batch}.rs`, `combat_policy.rs` only for necessary shared exhaustive dispatch/isolation |
| Evidence/docs | new `crates/spire-core/examples/m15_silent_heart.rs`, `examples/m15_support/mod.rs`, external/core/env M15 tests, new `docs/milestones/m15-silent-heart.md`; update this ledger with actual evidence |

Senior owns shared-module integration. Assistants may own disjoint new family
files in separate materializations; returned patches are integrated sequentially.
Do not use `.68`'s enemy setup files or `.69`'s README/AGENTS as M15 write scope.

## Task 1: Resolve named source dependencies and preserve the baseline

**Files:** Read design/ledger, accepted M10/M14 profiles, `data/README.md`,
`data/{cards,relics,powers}` records and their cited studies. Update only the M15
ledger/research addendum with independently worded facts; private outputs stay
under independently retained task evidence.

**Interfaces:** Consumes the ledger's exact S1–S8 and G1–G5 questions. Produces
source-qualified supports, deterministic native support ordering, callback rules and explicit unavailable facts for
Tasks2–10; it does not create a new dataset or executable mechanics.

- [ ] Confirm assigned checkout/Bead/actor/current authority and accepted `.68`
  integration; preserve preimages and exact task inputs. Use the existing
  verified M14 receipt; do not rerun a cold full baseline matrix.
- [ ] Resolve only named missing class/method facts in ledger S1–S8/G1–G5 with
  retained public records and authorized named static lookups. Record archive
  SHA/tool version and unverified upstream channel; never publish raw private code.
  Initialized Silent Skill/potion memberships/type/tags and rare/common counts must be actual
  supports, not Ironclad substitutions. Original Java cursor/HashMap iteration parity remains outside scope; native
  Distraction concatenates rarity groups and sorts eligible IDs by `as_str`
  within each group. Other generators use their declared native support order.
- [ ] Resolve G5's named `AttackPotion`/`SkillPotion`/`PowerPotion.use`,
  `DiscoveryAction` constructor/generateCardChoices/update, actual-character
  pool initializer and relevant green constructor/type/HEALING tags. Qualify
  exact Silent26/33/11 expected sets, source exclusions, no rarity roll, ordered
  without-replacement choices and typed skip; do not import Ironclad28/28/14.
  G5 fixes deterministic native ordering; Java incidental order remains excluded.
  Any unavailable rule keeps its selector explicitly blocked, never narrowed.
- [ ] Specify the native checkpoint rule for any composition not already fixed
  by accepted owners. If source contradicts frozen shared behavior, send the
  exact conflict to primary root; do not silently revise old rules IDs.
- [ ] Verify cardinalities, names and eligibility against cited input records;
  record unresolved rules with affected cards/items and owning task. An ordinary
  missing implementation is work for those tasks; unavailable exact evidence is
  a blocker, never permission to prune its pool.

**Check:** link/path/anchor and input-hash checks only. No gameplay suite proves
a source fact. Task8's selectors must remain unavailable until its full support
and behavior dependencies are executable.

## Task 2: Complete direct card rules and explicit registry

**Files:** new `combat/cards/silent/direct.rs`, extend existing dispatch,
`content/ids/cards.rs`, `content/cards.rs`, `combat/card_state.rs` and new
`combat/tests/m15_silent_direct.rs`; register in `combat/tests/mod.rs`.

**Interfaces:** Keep `silent::payload(&State,&LiveCard,Option<u64>) ->
Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>` and
`public_values(&LiveCard)->Option<CardPublicValues>`. Add
`silent::direct::payload` with the same signature. Later family modules use that
same payload signature and return `None` for unowned kinds. All new CardId
variants/tokens follow the ledger's proposed native column; source game IDs
remain a distinct catalog mapping. Do not mark missing handlers executable.

Own nine missing kinds: DaggerSpray, Slice, SuckerPunch, Backstab, Dash, LegSweep,
RiddleWithHoles, Adrenaline, DieDieDie. Append missing identities for all56 as
their family arrives, not placeholder success rows. Wire permanent/generated
constructors, upgrade flags, target legality, cost and public fields with behavior.

- [ ] Add base/+ behavior tests named `m15_direct_pairs_use_shared_damage_and_block`:
  Slice6/9 at0 energy cost; Dash10/13 Block before10/13 damage at2; DaggerSpray
  two separate AoE hits4/6; Riddle five selected hits3/4; Adrenaline1/2 energy
  then two draws, Exhaust; Backstab0 cost, Innate and Exhaust both variants.
  SuckerPunch costs1, selected damage7/9 then Weak1/2 as a separate application
  (Artifact can block Weak while damage still resolves); DieDieDie costs1,
  no selected target, separate per-enemy AoE damage13/17 and Exhaust exactly once.
  Use actual native entries, target/Block/Weak cases, not just metadata equality.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m15_direct` and
  retain a meaningful missing-behavior failure before implementing minimal dispatch.
- [ ] Add the paired local values from the cited card records, ordinary legality,
  fresh/generated flags and exact public cost values. Retain shared binary32
  damage/Block and per-hit effects; no direct HP writes.
- [ ] Re-run the focused command, then `cargo test -p spire-core --lib --locked --offline m12_silent_values`.
  PASS means actual play/flags/visible quantities, not complete Silent coverage.

## Task 3: Discard, draw-history and action-time counts

**Files:** new `silent/history.rs`, extend `silent/discard.rs`,
`combat/{model,piles,engine,effects,hooks}.rs`, new
`combat/tests/m15_silent_history.rs`.

**Interfaces:** Keep `manual_discard(&mut State,&LiveCard)->Result<bool,InputError>`.
Add source-specific typed `SilentEffect` in `silent/state.rs`, exported through
`cards/mod.rs` and `EffectKind::Silent(SilentEffect)`; shared dispatcher calls
`silent::apply(&mut State,&Effect,&SilentEffect)->Result<(),EffectError>` and
`silent::validate_work(&State,&Effect,&SilentEffect)->Result<(),InputError>`.
Use explicit per-turn play/discard history and per-draw-action history owners
in `State`; do not infer them from final piles. New state is private and resets
at the source-qualified checkpoint. Reuse `PendingSelection` for hand movement.

Own Acrobatics, Prepared, SneakyStrike, AllOutAttack, CalculatedGamble,
Concentrate, EscapePlan, Expertise, Finisher, Flechettes, GrandFinale, Unload.

- [ ] Write `m15_history_distinguishes_manual_cleanup_and_action_snapshots`:
  Prepared draws1/2 then discards1/2; Concentrate with0/1 remaining cards still
  grants2 energy; manual discard triggers Reflex2/3 and Tactician1/2, cleanup
  does not; Unload discards non-Attacks including Powers/Statuses/Reflex;
  reaction-drawn cards do not enlarge the original discard snapshot.
- [ ] Write `m15_history_distinguishes_absent_draw_from_draw_zero` with nonempty
  history: empty CalculatedGamble and Expertise at h>=6/7 enqueue no draw and
  do not clear history; actual zero-count draw clears at its qualified initial
  checkpoint. Pin nested EscapePlan's history replacement from Task1 S3.
- [ ] Write `m15_count_cards_not_hits_or_future_hand`: one two-hit Attack counts
  once for Finisher6/8 per prior Attack; Flechettes4/6 per live Skill includes
  unplayable Reflex/Tactician; GrandFinale50/60 is playable only with empty draw.
  Check captured count versus later reactions and target death.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m15_silent_history`; observe
  behavior RED, implement only the shared transactions/continuations and re-run
  GREEN. Also run the existing `m11_silent_discard` filter. No new selection loop.

## Task 4: Delayed resources, powers and trigger checkpoints

**Files:** new `silent/powers.rs`, extend `combat/{hooks,powers,effects,model}.rs`,
existing full-power registry/validation in `model.rs`/`engine.rs`,
new `combat/tests/m15_silent_powers.rs`.

**Interfaces:** New payload uses Task2 signature; typed continuation uses Task3
`SilentEffect`. Use existing `FullPowerId`, `FullPowerEntry`, signed `PowerState`,
shared `DamageResult` and hook dispatch. New strings are explicit in the M15
descriptor; old schemas reject new-only states. Validate private scheduled
resource/duration state alongside existing Nightmare/turn state.

Proposed new native power tokens are `energized`, `draw_card_next_turn`,
`next_turn_block`, `blur`, `choked`, `infinite_blades`, `after_image`,
`a_thousand_cuts`, `envenom`, `phantasmal`, `double_damage`, `intangible_player`,
`wraith_form`, `tools_of_the_trade`; Task5 adds `burst`, Task6 `corpse_explosion`.
Reuse existing `dexterity`, `thorns`, `no_draw`, `shackled`, `poison`, `weak`,
`vulnerable` and preserve enemy `intangible` separately. These are native names,
not claims about game constructor IDs. Signed Wraith downside and per-application
Artifact classification are explicit validation/descriptor entries, not inferred
from name or positive-only counters. Task8 adds only its required shared tokens:
`lose_dexterity`, `plated_armor`, `duplication`, `panache`, `sadistic_nature`,
`magnetism`, `mayhem`, `the_bomb`, reusing `regeneration`, `ritual`, `confused`.
Reconcile the accepted `.68` `plated_armor` owner instead of duplicating it.

Own DodgeAndRoll, FlyingKnee, Outmaneuver, Blur, Caltrops, Choke, Footwork,
InfiniteBlades, Predator, AThousandCuts, AfterImage, Envenom, PhantasmalKiller,
ToolsOfTheTrade, WraithForm.

- [ ] Write `m15_powers_separate_capture_recharge_predraw_and_postdraw`: Dodge
  and Roll captures calculated4/6 Block with current Dexterity; changing next
  turn Dexterity does not recalculate that capture. Outmaneuver2/3 delivers once
  at recharge; Predator adds2 next-turn draws. Tools stacked draw/discard suspends
  once after draw; InfiniteBlades creates ordinary Shivs rather than Shiv+.
- [ ] Write `m15_powers_separate_duration_and_signed_downside`: Wraith installs
  Intangible2/3 and independent -1 Dexterity-per-turn downside; Intangible expiry
  does not erase downside; negative Dexterity remains signed; Artifact can prevent
  the downside separately. Phantasmal doubles NORMAL once while duration controls
  lifetime; Blur blocks start-turn clearing, not incoming hits.
- [ ] Write `m15_powers_callbacks_use_source_checkpoints`: fully blocked NORMAL
  attack triggers Caltrops3/5 but not Envenom; positive NORMAL enemy HP loss
  triggers Envenom, THORNS/HP_LOSS do not. AfterImage on-use differs from
  AThousandCuts after-play and Choke HP_LOSS. Pin Task1 S4/S5 callback order with
  selection suspension and actual repeated plays, then reuse these assertions
  against Heart in Task9.
- [ ] RED/GREEN: `cargo test -p spire-core --lib --locked --offline m15_powers`;
  preserve existing `m11_silent_retain` and `m10_power_turns` filters.

## Task 5: Cost layers, copy identity and X/repeat work

**Files:** new `silent/costs.rs`, extend `silent/{copy,shiv}.rs`,
`combat/{card_state,piles,model,engine,effects,selection}.rs`, existing shared
autoplay owner; new `combat/tests/m15_silent_cost_copy.rs`.

**Interfaces:** Use existing `LiveCard`, `CombatCardData`, `CopyProvenance`,
`copy_family`, `PlayContext`/`PlayFrame`, Nightmare template ownership and typed
Task3 work. Add explicit combat/turn cost state for green cards rather than
reusing the BloodForBlood-only field as a generic price. Extend the existing
stat-equivalent/same-family copy functions once; no second copy engine.

Own EndlessAgony, Eviscerate, MasterfulStab, Setup, Skewer, BulletTime, Burst,
Doppelganger, GlassKnife, Malaise, StormOfSteel. Preserve existing Nightmare.

- [ ] Write `m15_cost_copy_keeps_instance_location_and_writer_order`: Masterful
  in hand/draw/discard rises by1 per positive loss event, exhausted/playing card
  misses it; a5HP hit is one callback. Eviscerate receives per-pile discard
  notifications and reconciles on draw; fresh copy versus stat-equivalent cost
  differs. Test the source's makeCopy source-side effect from the cited study.
- [ ] Write `m15_setup_and_bullet_time_have_distinct_cost_lifetimes`: Setup grants
  a free flag only for positive mutable combat cost, preserves both numeric
  costs, moves the same instance to draw top; X/unplayable/zero-combat-cost with
  positive turn cost get no new flag. BulletTime affects eligible nonnegative
  turn costs only, with NoDraw independent of generation. Parent resumes once.
- [ ] Write `m15_copy_generation_is_not_draw_or_alias`: EndlessAgony creates one
  stat-equivalent copy on actual draw, not creation; Storm snapshots n before
  discard reactions and supplies n ordinary/upgraded Shivs as appropriate;
  GlassKnife two8/12 hits precede -2 family mutation, floor0, fresh unrelated
  copy unaffected, master deck unchanged. Verify hand10 overflow is retained.
- [ ] Write `m15_burst_and_x_preserve_captured_energy_and_real_use_count`: Burst
  repeats next1/2 eligible Skills once each, purge child does not consume again;
  Skewer C hits7/10; Doppelganger/Malaise upgraded at zero captured energy still
  have amount1; captured C differs from live energy, free wrapper preserves C.
  ChemicalX's K=2 is required only for that reachable composition; do not invent
  its native support if the relic remains unadmitted. No second child payment.
- [ ] RED/GREEN `cargo test -p spire-core --lib --locked --offline m15_silent_cost_copy`;
  then existing `m11_silent_copy`, `m10_copy`, `m10_autoplay` filters. Resolve
  Task1 S1/S2 before asserting mixed writer/terminal totals.

## Task 6: Poison, independent debuffs and death-chain composition

**Files:** extend `silent/poison.rs`, `combat/{powers,effects,hooks,engine}.rs`,
existing death owners in `combat/enemies/act2`/`act3` only for shared integration;
new `combat/tests/m15_silent_poison_death.rs`.

**Interfaces:** Same payload/typed work seams; use existing `DamageResult`,
`ActorLife`, `PoisonTickHpLoss`/`PoisonTickComplete` and native death/half-dead
callbacks. Source target handles and live callback amounts stay owned by the
queued continuation; no precomputed universal chain result.

Own Bane, BouncingFlask, PiercingWail, CripplingCloud, HeelHook, Terror,
CorpseExplosion; extend Specimen death integration with Task7.

- [ ] Write `m15_poison_samples_before_each_qualified_application`: Flask3/4
  bounces each Poison3, next target selected before current application; singleton
  and no-live-support paths; stale/dead target does not reroll unconditionally.
  Bane captures7/10 twice at use and honors its distinct later Poison/HP gates.
  HeelHook conditional draw/energy follows the source-qualified Weak checkpoint.
- [ ] Write `m15_debuff_applications_have_independent_artifact_receipts`: Wail
  queues -6/-8 Strength for all then Shackled only for use-time-no-Artifact
  entries; CripplingCloud Poison4/7 then Weak2 per live target; Terror
  Vulnerable99 and its base/+ cost1/0. Corpse Poison6/9 and explosion installation
  can be prevented independently, and removing Poison leaves explosion.
- [ ] Write `m15_death_chains_use_callback_max_hp_and_live_owners`: supplied
  actual split, Darkling half-dead, Awakened rebirth and last-enemy cases; explosion
  uses callback maxHP × stacks, not remaining HP; pin Task1 S5 arbitration and
  Specimen timing before exact nested totals. Retain accepted prefix on fault.
- [ ] RED/GREEN `cargo test -p spire-core --lib --locked --offline m15_silent_poison_death`;
  then `m11_silent_poison`, `m14_silent_interactions`, `m10_heart` focused filters.

## Task 7: Exclusive relics and three Silent potion payloads

**Files:** `content/ids/items.rs`, `content/items.rs`, new `items/silent.rs`,
existing `items/{mod,relics,potions}.rs`, shared hooks/damage/copy owners,
new `items/m15_silent_items.rs` registered in `items/mod.rs`.

**Interfaces:** Keep `acquire_relic(&mut RunState,RelicId)->Result<(),RunError>`
and existing `CombatItemContext` custody. Add
`replace_ring_of_the_snake(&mut RunState)->Result<(),RunError>` next to the
existing BurningBlood replacement. Relic callbacks return ordinary
`RelicHookWork`/effects; potion payloads use existing `PotionUseEffects`.

- [ ] Add nine missing exclusive relic IDs and three potion IDs from the ledger.
  RingSnake/SneckoSkull are existing mechanics, not new implementations. Validate
  source tiers/upgrade prerequisite and actual callback facts from Task1 S6.
- [ ] Write `m15_silent_items_keep_custody_and_once_only_lifetimes`: Serpent
  replaces Snake (opening bonus2 removed, draw-per-turn+1); NinjaScroll creates
  three ordinary Shivs at the qualified startup checkpoint; TwistedFunnel
  applies Poison4 per live enemy; ToughBandages3 Block per manual discard,
  Tingsha3 random enemy damage per discard, Kite first qualifying discard energy1
  per turn; cleanup does not impersonate manual discard. Parent interruption
  keeps already accepted effects and owner counters.
- [ ] Write `m15_item_damage_and_poison_use_shared_callbacks`: PaperKrane weak
  multiplier0.6 at the source modifier checkpoint; WristBlade+4 only for eligible
  zero-turn-cost NORMAL Attack damage (not every zero-cost effect); Specimen
  transfers callback Poison to a source-selected living target, respecting
  prevention/Skull and Task1 source order. No target or half-dead case is guessed.
- [ ] Write `m15_exclusive_potion_pairs_use_real_belt`: Poison6/12, Cunning
  three/six Shiv+ (verify exact constructor potency in Task1), Ghost Intangible
  one/two; target/phase gates, hand overflow, consumed-once slot and Ornithopter
  hook order. SacredBark is a supplied interaction fixture, not automatic M15
  normal relic admission. Reject wrong-character factory eligibility separately.
- [ ] RED/GREEN `cargo test -p spire-core --lib --locked --offline m15_silent_items`;
  existing `m11_silent` and `m10_ironclad` item filters preserve their behavior.

## Task 8: Complete generation and potion obtain dependencies

**Files:** new `silent/generation.rs`, `items/potion_generation.rs`, extend
`items/{mod,potions}.rs`, `combat/{model,command,view,engine,effects,card_state}.rs`,
shared generated-card support, new `combat/tests/m15_generation.rs` and
`items/m15_potion_generation_tests.rs`. New `combat/cards/colorless.rs` dispatches
`colorless/{direct,choices,powers,generation}.rs` through the Task2 payload
signature, with `combat/tests/m15_colorless_{direct,choices,powers,generation}.rs`.
These exact transitive families cover ledger G3, not unrelated later content.

**Interfaces (new proposals):**

- `silent::generation::distraction_candidates(character:Character)->Result<&'static [CardId],InputError>`
  returns the complete source-qualified nonhealing Skill domain or error.
- `silent::generation::typed_potion_candidates(character:Character,kind:CardType)->Result<&'static [CardId],InputError>`
  exposes only G5's complete qualified actual-character Attack/Skill/Power
  domains in their declared native order; other types/incomplete domains reject.
  Its three-offer sampler uses the existing card-random stream and pending offer
  owner below, with G5 typed optionality and without-replacement identity law.
- `items::potion_generation::candidates(character:Character,limited:bool)->Result<&'static [PotionId],InputError>`
  and `sample(character:Character,limited:bool,stream:&combat::rng::StreamState)->Result<(PotionId,combat::rng::StreamState),EffectError>`
  use the existing card-random owner; outside-combat Brew uses a separately
  registered Run RNG scope with unchanged old keys. Exact rarity support comes
  from Task1. Never draw from only currently registered kinds.
- `CombatItemContext::obtain_potion(&mut self,id:PotionId)->Result<bool,InputError>`
  validates the proposed ID/current inventory before insertion into the first
  empty slot; full valid belt is `Ok(false)` with no mutation. Invalid accepted
  ID/inventory is `InputError::Invariant("generated potion obtain input invalid")`.
  `EffectKind::ObtainPotion` performs the source-specific Sozu check before
  insertion, converts helper errors through existing `From<InputError>` to
  `EffectError::Invalid`, and lets the shared resolver retain the original
  current effect/source and accepted prefix as first fault. No `RunError`
  conversion is added to combat `EffectError`.
  Outside combat, `items::obtain_run_potion(&mut RunState,PotionId)->Result<bool,RunError>`
  uses the same belt insertion rule under Run custody. Preaccept invalid input
  maps to `RunError::Unsupported("generated potion obtain input invalid")`;
  failure after an accepted outside obtain continuation retains
  `RunFaultKind::Invariant("accepted potion obtain failed")` at
  `run.m15.potion.obtain`, with no retry/rollback. Full is still `Ok(false)`.
- Generated card-offer choices are templates, not owned pile cards. Add
  `CombatCardOfferView{cards:Vec<CardId>,optional:bool}` and
  `CombatPublic.card_offer:Option<CombatCardOfferView>`, private pending offer
  with source/selected/copy-count cursor, and
  `CombatAction::ChooseCardOffer{index:u32}` / `SkipCardOffer`. They use the
  existing resolver suspension and parent, never fake hand-instance handles.
  Append corresponding semantic replay actions and M15 vocabulary. Old profiles
  cannot encounter these rows; old encoders reject them if supplied diagnostically.
  Preserve old public Debug bytes when `card_offer` is None: replace derived
  Debug with an explicit formatter reproducing the prior field order/shape and
  emit the extra field only when present. Do the same for any added field that
  reaches accepted transcript serialization; never rewrite preserved digests.

Own Distraction and Alchemize plus **all** actual generated dependencies:
30 shared + three Silent potion kinds, complete typed green potion supports,
34 eligible colorless kinds, their generated/status children and durable effects.
Named missing shared kinds are work, not source blockers. Group their handlers
by shared draw/stat/damage/selection/copy/death mechanics and reuse Tasks2–7;
do not create a card-by-card framework or silently close G3 on metadata alone.

- [ ] Write `m15_generation_supports_match_exact_actual_character_domains` with
  complete expected ID sets and native order from Task1: Distraction filters SKILL/HEALING,
  no rarity draw; limited Alchemize excludes FruitJuice and red exclusives,
  outside Brew permits FruitJuice; Colorless excludes BandageUp, basics/status/
  curse/special and admits HandOfGreed. Also assert G5's exact26 Attack/33
  Skill/11 Power ID sets, Alchemize HEALING exclusion, retained unplayable
  Reflex/Tactician, no basics/Shiv/status/curse/special, native rarity-group then
  `as_str` order, and three distinct ordered offers with uniform next-ID support
  `N`, `N-1`, `N-2` (not a rarity lottery). Check no registered-subset fallback.
  For each typed potion assert `optional == true`, legal skip, skip creates no
  child, choose resumes once and inspection/selection do not reroll. Keep
  Colorless/Discovery's separate qualified optionality rather than sharing a flag.
- [ ] Write `m15_generated_obtain_uses_live_capacity_and_source_phase`: full
  belt fails without prompt or replacement, Sozu blocks at execution after a
  generated proposal, no reroll; combat Brew creates live-slot-count proposals
  and its own slot vacates before obtains; outside Brew Sozu prevents generation.
  Obtain does not fire use/Ornithopter; actual used generated potion returns
  remaining belt on Victory and Death, accepted gain retained on later fault.
  Malformed accepted generated ID/inventory must yield the resolver-native
  invariant fault, preserve its original ObtainPotion effect/source and belt,
  and retain the first fault on repeated inspection/step; full valid belt is
  a successful no-obtain effect, not that fault. Outside preaccept/accepted
  failures exercise their distinct Run mappings above.
- [ ] Write `m15_card_offer_selection_is_not_pile_ownership`: three distinct
  ordered offers, source-specific skip, no reroll on inspection/select; chosen
  identity1/2 copies with guarded turn-cost zero, hand9 overflow, proper temporary
  upgrade checkpoint, one parent resume/payment; stale/foreign index atomic.
- [ ] Write `m15_transitive_potions_resolve_real_mechanics`: Fairy automatic
  rescue versus manual unusability, Smoke source restrictions/escape settlement,
  Duplication/DistilledChaos captured repeats/targets, Memories/Gambler selectors,
  Entropic nested obtain, colorless generated/permanent gain reconciliation.
  Assert actual callbacks/resources and retained prefix; no fixed aggregate
  terminal outcome until Task1 source chain is qualified.
- [ ] Complete the four shared colorless family cycles below, each RED/GREEN
  through `cargo test -p spire-core --lib --locked --offline m15_colorless`.
  Base/+ costs/scalars/flags come from existing paired records, and exact
  selection/turn/copy rules from the cited red/colorless study at its stated
  grade; Task1 S1–S5 qualify remaining compositions before oracle totals:

  | Family | Complete kind set | Discriminating assertions |
  |---|---|---|
  | direct (15) | Blind, DarkShackles, DeepBreath, DramaticEntrance, Finesse, FlashOfSteel, GoodInstincts, Impatience, MindBlast, Panacea, PanicButton, SwiftStrike, Trip, HandOfGreed, MasterOfStrategy | Finesse Block2/4 then draw1; Flash3/6 damage then draw1; DeepBreath moves real discard into shuffle then draw1/2, no duplicate ownership; Impatience checks remaining hand Attacks; Blind/Trip upgraded AoE; HandOfGreed actual eligible Fatal gold20/25 retained once through Run, minion/last-enemy cases use existing Fatal rules |
  | choices/cost (9) | Apotheosis, Enlightenment, Forethought, Madness, Purity, SecretTechnique, SecretWeapon, ThinkingAhead, Violence | actual all-pile temporary upgrade versus master deck; Forethought ordered draw-bottom/free flag; Madness permanent-combat cost mutation versus turn reset; Purity optional0 selection, actual Exhaust hooks; Secret type-filtered draw choice; ThinkingAhead post-draw choice; Violence random live Attack movement without drawing callbacks |
  | powers (5) | Magnetism, Mayhem, Panache, SadisticNature, TheBomb | actual start-turn generation/autoplay; Panache counts cards rather than hits and resets at turn; SadisticNature per accepted enemy debuff application, independent Artifact; Bomb3-turn delay and40/50 source payload, actual cancellation/stack ownership |
  | generation (5) | Discovery, JackOfAllTrades, Chrysalis, Metamorphosis, Transmutation | full source-qualified supports, typed offers/optionality; Jack1/2 colorless children; Chrysalis/Metamorphosis3/5 generated Skills/Attacks with combat cost mutation; Transmutation captured X, generated identities and turn costs; no unsupported recursive candidates |

- [ ] RED/GREEN `cargo test -p spire-core --lib --locked --offline m15_generation`
  and `cargo test -p spire-core --lib --locked --offline m15_potion_generation_tests`.
  If a named rule is unavailable, retain its exact blocker and do not enable an
  incomplete sampler or call the full character complete. Update G1–G5 with
  implemented paths/tests and source qualifications before Task9 publication.

## Task 9: M15 admission, captured resources, keys and shared Act4

**Files:** new `run/rooms/m15_content.rs`; existing Run/room files in map;
new `run/tests/m15_silent_heart.rs`, `run/tests/m15_content.rs`,
`combat/tests/m15_act4_interactions.rs`, register tests/exports.

**Interfaces:** Append `RoomScenario::M15SilentHeart`, add `is_m15(&RunState)->bool`
and `uses_act4(&RunState)->bool` (M10 or M15). Extend Silent/fullAct3 predicates
at existing owners; keep `is_m10` exact. New content API:
`green_candidates(CardRarity)->&'static [CardId]`,
`sample_capture_row(&mut RunRng,&[u64],&[CardRarity])->Result<Vec<CardId>,RunError>`,
M15 card/relic/potion admission constants, and explicit 19-kind stock capture.
Append `RunRngScope::M15CardOffer` / `"run.m15.card.offer"`; clone/preflight/
commit as M10 does, but use checked exhaustion for the new scope. Do not change
M10's RNG messages, enum discriminants, captured row order or stock.

- [ ] Write `m15_normal_creation_and_capture_support_are_character_complete`:
  exact canonical resources; 4/19/33/19 unique green IDs and base/+ obtainable/
  Smith/removable instances; 71 ordinary support, no basics/Parasite/Shiv in
  ordinary rewards; all new kinds executable before enabled rows. Existing
  captured event branches act on actual green basics and duplicate instances.
- [ ] Write `m15_stock_counts_offers_and_preserves_skip_custody`: 19 accepted
  chest/elite captures across all acts, Sapphire/Proceed consume opened offer,
  shop/boss/event sources cannot steal stock; cursor/choice overflow fails before
  accept; source exhaustion operational, no refill or route pruning. Verify
  all-exclusive acquisition tiers and boss replacement prerequisite. Reuse the
  accepted `.68` gold custody/earned-recovery API for generated HandOfGreed and
  escape interactions; no second journal or unconditional stolen-gold refund.
- [ ] Write `m15_shop_rng_commits_once_per_accepted_visit` with a compact
  act1/2/3/4 shop table using actual shared room-entry/preparation owners. Save
  original Run RNG, prepare on a detached clone, enter once and assert committed
  RNG equals that prepared snapshot, including exactly its accepted draw count.
  Repeated decision/public-map/offer inspection and legal buy/Leave do not
  recommit/resample; a stale repeated Move does not advance it. Force choice-ID
  exhaustion during detached shop capture: its stream remains uncommitted on
  failure in each act. Keep any already accepted floor/hook prefix and first
  fault; this assertion promises no rollback of earlier accepted work. Existing
  M10 shop transcript/RNG behavior and keys remain unchanged.
- [ ] Write `m15_earned_keys_and_actual_act4_chain_have_same_visit_sources`:
  real Recall/relic-vs-Sapphire/burning-elite Victory; each key+5 once, full-belt
  reward child then parent restore, stale key choice atomic; all eight key masks
  with actual boss record, no-key ending0, all-key fourth choice transitions
  once with ordinary heal and floors51/52/53/54. These internal supplied fixtures
  are lane1, even when all subsequent actions are public.
- [ ] Write `m15_act4_rewards_and_endings_reject_forged_or_partial_sources`:
  actual Shield/Spear Victory and closed reward child before Heart; no repeated
  visits/forged foreign record; real Heart Victory screen then Proceed55 +110
  once; Death -1, pending/budget/fault not terminal success. Inject capacity/ID/
  settlement faults at preaccept and postaccept frontiers and retain first owner.
- [ ] Write `m15_act4_interactions_use_real_enemy_and_item_owners`: actual
  Shield110/Spear160/Heart800 entry; Artifact and facing after poison survivor,
  Burst repeated Skill Beat once per actual use, AfterImage/Choke/AThousandCuts
  source order, Intangible vs Heart multi-hit, Poison/THORNS/NORMAL vs remaining
  allowance300 and reset. Corpse/Specimen transfer cannot bypass shared allowance.
- [ ] RED/GREEN `cargo test -p spire-core --lib --locked --offline m15_` then
  existing `m10_act4`, `m10_keys`, `m10_settlement`, `m14_silent_act3` filters.
  Finish only after all enabled generator dependencies are real or explicitly
  retain M15 incomplete at the exact blocker; no finite-profile substitution.

## Task 10: Versioned native schema, session, replay and batch

**Files:** native transport map above; new
`crates/spire-env/tests/run_m15_{schema,session,replay,batch}.rs`.

**Interfaces:** Add `RunSession::new_m15(RunSessionConfig)->Result<Self,RunSessionError>`,
`RunBatch::new_m15(Vec<RunSessionConfig>,usize)->Result<Self,RunBatchError>`,
`m15_run_observation_spec(&RunCapacities)->Result<RunObservationSpec,RunSessionError>`,
`encode_run_m15(&PublicDecision,Option<&PublicMap>,&RunCapacities)->Result<RunObservation,RunSessionError>`.
Append typed history arm with existing Act1/2/3 target enums; portable JSON and
all exact M15 IDs are fixed in the design. Reuse shared reset/step/token/record/
reproduction/replay and persistent ordered scheduler APIs unchanged.

- [ ] Write schema tests enumerating every admitted ordinary/generated/curse/
  potion/relic/enemy/offer/operation ID to descriptor name, signed values, known
  masks, facing and allowance. Use M10 groups plus a `combat_card_offers` group
  with rows bounded by `capacities.offers`, columns `[card_kind, offer_index]`,
  and append the two Task8 action verbs. Candidate actions reference that public
  offer index, never private template handles. Overflow explicit; no hidden draw
  rows, templates/cursors/future offers or unknown-as-zero leakage.
- [ ] Write session/replay tests for all27 triples, complete malformed JSON
  preflight, native Reset/step/token mismatch, fresh public offer choice,
  no-recording fault reproduction, first retained core/capacity fault, incompatible
  old profile/rules/content IDs and M15 semantic replay action decoding.
- [ ] Write `m15_native_profiles_keep_old_bytes_and_boundaries`: preserve old
  descriptor fingerprints, typed histories and accepted full Debug transcript
  digests, M12 Act2Ready, M13 Act3Ready,
  M14 healthy Act4Ready/step rejection, M10 no Silent sidecar change and old
  default constructor. Old schema rejects supplied new-only offer/state.
- [ ] RED/GREEN commands: `cargo test -p spire-env --locked --offline --test run_m15_schema`,
  likewise `run_m15_session`, `run_m15_replay`, `run_m15_batch`. Batch tests use
  workers1/2, heterogeneous slots, whole-input atomic preflight, accepted peer
  prefix/first indexed fault/quiescence, lost delivery, no retry/autoreset.
- [ ] Run existing `cargo test -p spire-env --locked --offline --test run_m14_schema`
  and affected M10/session/replay/batch targets once after final shared changes.
  No binding build/import or Python-delivery claim follows native success.

## Task 11: Frozen evidence, qualified profile and one final review

**Files:** M15 example/support (including new `examples/m15_support/caller_tests.rs`)
and external tests, new milestone profile, ledger; additive opt-in M15 dispatch
in existing `examples/m13_support/mod.rs`, preserving existing M13/M14 selector
bodies and accepted transcripts. All receipt/log files retained outside managed
sessions under assigned evidence.

**Interfaces:** Reuse the public caller's legal `PublicDecision` input and
`GameAction` output. Add the pure M15 key override
`key_action(decision:&PublicDecision,map:Option<&PublicMap>)->Option<GameAction>`
in `examples/m15_support/mod.rs`, revision `m15-key-aware-public-caller-v1`.
In the existing shared caller module, add `Caller::new_m15(limit:usize)->Result<Self,String>`
and `rollout_m15(run:&mut Run,caller:&mut Caller,key_action:fn(&PublicDecision,Option<&PublicMap>)->Option<GameAction>)->Result<M15CallerOutcome,String>`.
Define the separate public `M15CallerOutcome { HeartVictory, Death,
BudgetExhausted, Act3NoKeys, EmptyDeck }`, with exact success display `HeartVictory`.
Keep existing `CallerOutcome`, its Display and M13/M14 public return types and
exhaustive matches unchanged. Widen only the private shared-loop finish carrier
for M15 results; old wrappers explicitly reject a foreign M15 result instead of
reclassifying it. Do not copy the rollout loop.
Add `m15_stop_outcome(decision:&PublicDecision,last:Option<&TraceStep>)->Result<Option<M15CallerOutcome>,String>`.
Only actual final `RunTerminal::HeartVictory` after an accepted legal Proceed
can return `Some(M15CallerOutcome::HeartVictory)`: require current M15 profile,
act4/floor55/TrueVictory and the last accepted TraceStep's nonterminal floor54
HeartVictory screen, Offered Proceed present in legal candidates, matching
Choose action, terminal transition and committed Ended(HeartVictory) receipt.
A claimed Heart terminal without that accepted source is an operational error.
Heart entry or pending HeartVictory screen returns None; actual Death,
Act3NoKeys and EmptyDeck return their distinct M15 variants. Evaluate actual
terminal completion before the budget check; otherwise an exhausted budget
returns M15 BudgetExhausted. Leave `m14_stop_outcome` rejecting HeartVictory and
its supplied Act4Ready cutoff unchanged. This result labels settlement only;
supplied fixtures never become normal-start evidence by returning it.
Pass that pure override into the existing shared rollout at the M15 entrypoint;
old entrypoints pass no override. This is one callback parameter, not a stored
callback state or another rollout implementation. The key override reads
`decision.player.keys` in
Ruby/Sapphire/Emerald order and public map nodes/edges only. It returns no override
while a combat/selection parent is suspended. At ordinary room boundaries:

1. Ruby missing + offered legal Recall: choose Recall before Rest/Smith.
2. Sapphire missing: OpenChest if unopened, then offered legal Sapphire before
   any chest relic. With Sapphire owned, retain ordinary chest policy.
3. Emerald missing + offered legal Emerald reward: claim before other parent
   rewards. At Map, prioritize offered Move candidates with a public directed
   path through an unvisited `burning == Some(true)` Elite and onward to Boss.
   Rank by minimum directed edge count from candidate to that burning node,
   then smallest next NodeId; unknown/nonelite burning fields never qualify.
   If no candidate has such a path, return no override: preserve normal route
   fallback and record any unreached key. With Emerald owned, stop this objective.
4. At actual Act3Ending choose the offered legal stage4 continuation after the
   earlier dialog stages; HeartVictory chooses offered legal Proceed. Missing
   keys retain the real no-key ending rather than supplying them.

Every override must be in current legal candidates and its choice row Offered;
otherwise return no override. Wire an opt-in `CallerProfile::M15` through the
existing rollout/Caller dispatch and invoke this override before ordinary room
selection. Without an override reuse the existing M14 public selectors, with
first-offered-legal fallback for new offer/Act4 cases; do not copy its rollout
loop. Existing M13/M14 selector bodies and accepted transcript bytes remain
unchanged even though additive dispatch changes the shared file hash. Do not
consult private Run state for choices. This fixed key objective is specified
before outcomes; it does not filter seeds, avoid losses or promise a win. Reuse
M14's six fixed seed/triples (not its subsequent42-case coverage expansion).

- [ ] Write `m15_public_caller_pursues_only_missing_keys_with_deterministic_ties`
  in `examples/m15_support/caller_tests.rs`, registered by that support module:
  public-only fixtures pin missing/owned Ruby Recall, unopened/opened Sapphire
  priority vs relic, Emerald reward, visible reachable burning-route preference
  over ordinary weights, distance then NodeId ties, unreachable/unknown/visited
  burning fallback, stopped Emerald objective after acquisition, offered stage4
  and Heart Proceed, suspended parent, and disabled/absent/stale legal-candidate
  rejection. Freeze identical decisions in different presentation orders to
  confirm declared route ties; other first-legal fallback keeps candidate order.
  Run `cargo test -p spire-core --example m15_silent_heart --locked --offline m15_public_caller`
  before evidence play. These are legality/key-objective tests, not win tests.
- [ ] Write `m15_public_caller_heart_success_requires_final_proceed_terminal`
  in the same caller test module: actual Heart entry and the post-combat
  HeartVictory screen are pending; budget exhaustion before Proceed is
  BudgetExhausted; only the offered legal Proceed accepted by Run then yielding
  floor55/TrueVictory/RunTerminal::HeartVictory and its trace receipt returns
  M15 HeartVictory, including when the final action exactly uses the budget.
  Death, no-key ending and EmptyDeck remain distinct; a forged terminal without
  the accepted Proceed receipt errors. Use the existing supplied Task9 chain for
  mechanics only. Assert old CallerOutcome variants/Display/exhaustive callers
  still compile, old M14 mapper still returns None for a Heart terminal, and
  accepted M13/M14 transcripts/cutoffs remain unchanged. Run with the existing
  `m15_public_caller` command before the six normal attempts.
- [ ] Before any run, freeze source/modes, six input rows, caller revision,
  this tested key-aware adapter and8192 budget. Keep option validation/failure
  reporting checks. Make no result-driven policy changes after this freeze.
- [ ] Run all six normal rows once through core public actions. Persist complete
  attempts including errors, every real key decision/visit/record and exact
  endpoint. `cargo run -p spire-core --example m15_silent_heart --locked --offline -- --seed 7 --scenario all`
  and seed11 are proposed entrypoints. Budgets0/1 are unfinished errors;
  budget8193 and malformed scenario/history reject. Genuine Death succeeds as a
  valid gameplay outcome but does not establish an unreached integration gate.
- [ ] Feed the same semantic actions to native single sessions, portable replay
  and workers1/2 batch; compare complete observations, rewards, terminals and
  faults, not only final summaries. Process one case at a time; receipt identity
  distinguishes core and transport evidence. Do not rerun for improved outcomes.
- [ ] Preserve separately labelled supplied key/Act4/Heart/ending mechanics
  fixtures from Task9; no tuned stronger supplied-loadout play series. The
  completion oracle rejects normal labels with injected inputs, missing same-visit
  records, partial dialog, Death/budget/fault and missing earned keys.
- [ ] Run smallest affected focused lanes above during work; after final source
  stabilization run one `cargo test --workspace --locked --offline` and record
  exact results/failure identities against accepted M14 and accepted `.68`.
  Do not assume the old historical failure remains after its fix, or rewrite
  assertions to hit a pass count. Check changed links, commands, paths and
  untracked prose whitespace separately. Native suites establish native behavior.
- [ ] Update the ledger per row with exact implementation/test/source evidence.
  Publish profile boundaries, actual normal versus supplied results, generator
  supports, shared-event/standard-pool and source/runtime limits. If normal
  traversal or a dependency is unproved, leave M15 incomplete and send root the
  concrete gap/options. No automatic adoption of M14/M10 acceptance amendments.
- [ ] Preserve independently retained source/preimages/manifests and hashes,
  then ONE fresh independent final milestone review of the frozen source and
  all five Review Focus classes. Senior disposes every finding/declined judgment;
  same reviewer rechecks fixes. Root verifies source identity and accepted checks.
  Close Beads only after actual acceptance under current primary authority.

## Planning handoff

This plan follows the already supplied execution method. Root receives frozen
design/plan/ledger/source-fact hashes before dispatching the ONE fresh independent
planning reviewer, then presents the concrete package for actual user approval.
No additional execution-method question, builder-review swarm or product start
is part of this handoff.
