# M16 Defect Combat Implementation Plan

> **For agentic workers:** Preserve the senior/native integration method with
> bounded Luna/MAX source-fact assistants as needed. This package authorizes
> planning only. ONE independent planning review returned READY; the same
> reviewer acknowledges this documentation correction pass before actual user
> approval and product work. ONE fresh final milestone review follows actual
> execution. No per-task review swarm, commits, builds or game execution now.

**Goal:** Deliver real finite Defect A0 combat with four orbs, ten-slot capacity,
power/item interactions and correctly owned persistent card mutations.

**Architecture:** Extend shared character/orb, card-instance, callback, inventory
and Run synchronization owners. Add a finite authored constructor and explicit
native Defect observation overlay over the existing encoder; copy no resolver,
Run progression, session, damage formula, encounter table or Python loop.

**Tech Stack:** Existing Rust workspace and locked Cargo dependencies, existing
native tests/examples and qualified static references; no new dependencies.

**Spec:** [M16 design](../specs/2026-10-27-m16-defect-combat-design.md) and
[coverage ledger](../../milestones/m16-defect-combat-coverage-ledger.md).

## Global Constraints

- Read baseline `e5b07da5609a3a2654b3bced3f8388ea96324a8f`; actual execution
  starts only after accepted M15/task68 source reconciliation and user approval.
- Exactly29 M16 card kinds and four relic/two potion witnesses as named in spec;
  both base/upgraded, real metadata/legality/values/payload/disposition/copies.
- Native authored Defect A0; canonical75HP/99gold/four StrikeBlue/four
  DefendBlue/Zap/Dualcast/CrackedCore/three empty potions/no keys. Supplied
  mechanics fixtures retain supplied labels. No normal-run Act1/2 claim.
- Orb capacity0..10; Focus [-999,999]; energy add cap999; hand cap10.
  Dark/misc/Claw/Steam fields have no invented999 gameplay cap.
- Reuse source int32 mutation and binary32 Lock-On conversion. Distinguish
  stored values, displayed calculated values, typed payload and HP actually lost.
- Keep existing IDs/defaults, combat-v1, M9/M10, policy-v3, M12–14 descriptors
  unchanged. New schema `spire-gym-m16-defect-combat-observation-v1` is opt-in.
- One slot owner, card-instance registry, master deck, inventory and encoder.
  Accepted-prefix faults retain state; no retry of committed mutation/potion.
- Focused meaningful TDD, one stable integrated check and one final milestone
  review. No count goal, seed search, win tuning, verifier framework or fake fights.
- Planning source reads/static lookups only now. No install/build/test/product
  execution/commit/push/PR/cleanup is authorized by this plan.

## Review Focus

1. Empty/sparse/max-capacity slots: zero-slot Focus/channel and Consume work;
   front means index0; Barrage counts occupied >3 through10. Task1 boundary tests.
2. Interrupted orb work and callback phases: Plasma/start energy, Dark target
   capture, lethal eviction/Fission and Discharge versus actual HP loss preserve
   source order and fault custody. Tasks1–3 paired ordering/fault tests.
3. Permanent identity under repeats/Stasis/Death/fault: GA learns once per
   accepted growth, fresh copies cannot mutate master and settlement cannot
   replay growth. Task4 identity/journal/exit tests.
4. Mutable costs and repeated Power use: zero turn-cost Streamline, worn Steam
   upgrade and purge repeats notify/pay/reset correctly without recursion or
   master pollution. Tasks3–4 cost/copy tests.
5. Public projection/compatibility: Dark payload, holes and learned misc are
   represented; hidden state stays hidden; small capacity faults explicitly and
   predecessor fingerprints remain exact. Task5 encoder/constructor tests.

## File and ownership map

| Files | Responsibility |
|---|---|
| `crates/spire-core/src/content/{mod.rs,cards.rs,items.rs,ids.rs,ids/cards.rs,ids/items.rs}`; new `content/defect.rs` | Stable named identities/definitions and finite admission only |
| `combat/characters.rs`; optional focused `characters/orbs.rs`; `combat/{model.rs,effects.rs,engine.rs,turns.rs,powers.rs,hooks.rs,view.rs}` | Owned orb state, typed work, validation, phase/callback integration and public projections |
| `combat/cards/mod.rs`; new `combat/cards/defect.rs` | Existing Defect adapter plus coherent blue payload/values/upgrade/callback dispatch |
| `combat/card_state.rs`, `combat/piles.rs`, existing play-frame/copy owners | Live misc, combat Block/cost, copy family, exactly-once mutation/repeat execution |
| `items/{mod.rs,relics.rs,potions.rs}` | Defect item admission, callback state, real use/custody and reset |
| `run/{model.rs,decision.rs,combat.rs,mutations.rs}` | Permanent misc, authored start, ordinal synchronization and exit census |
| new `spire-env/src/run_schema_m16.rs`; `run_observation.rs`, `combat_policy.rs`, `lib.rs` | Versioned public numeric projection over existing encoder |
| new focused test modules listed below and their `mod.rs` registrations | Behavior and failure tests beside current owner tests |
| new `spire-core/examples/m16_defect_combat.rs`; new `docs/milestones/m16-defect-combat.md` | Fixed native executable demonstration and exact accepted limits/results |

Paths above are relative to `crates/spire-core/src/` unless crate is named.
Do not split all shared modules as an unrelated refactor. New focused modules
join existing dispatch; their existence does not authorize another rule table.

## Before implementation: freeze actual accepted inputs

Current dependency context is recorded in the design: Task68 PR53 is accepted at
`d1fa706638a31a854fdcaeed9a9a4642abf716eb`; M15 v3 planning is user-approved and
execution started, but its source/API is unaccepted. M18 planning is user-approved;
its execution requires accepted M16/M17 and root release. None is M16 approval.

- [ ] Confirm assigned checkout/Bead/actor; read accepted M15/task68 outcomes and
  actual source SHAs. Record those plus this plan/design hashes and the qualified
  source-facts manifest. No active worker tree is an integration baseline.
- [ ] Reconcile accepted cost/history/repeat, encounter and fault owners against
  the proposed signatures. Reuse accepted changes. M17 remains unassigned;
  neither it nor the approved M18 plan supplies an accepted upstream API.
- [ ] Read Defect paired-record sections5–7 and named-static callback addendum.
  Freeze source call-site facts for start/end order, Cables/Chip, Static Discharge
  damage stage, GA matching set and signed-damage conversion. Resolve only a
  named residual before its affected implementation; do not guess from wiki.
- [ ] Capture one actual execution-baseline check if no source-identical accepted
  result exists. Separate build time from tests and retain named pre-existing
  failures. Then iterate focused commands, not repeated workspace baselines.

For all commands below, use one task-owned external `CARGO_TARGET_DIR` recorded
in execution evidence, an existing provisioned toolchain and `--locked --offline`.
Commands are future execution instructions; they have not run in planning.

### Task 1: Owned four-orb lifecycle, Focus and real capacity

**Files:** Modify `content/{ids/cards.rs,cards.rs,mod.rs}`, `combat/{characters.rs,
model.rs,effects.rs,engine.rs,view.rs,turns.rs,powers.rs}` and current
`combat/cards/mod.rs`. Create `content/defect.rs`, `combat/cards/defect.rs`,
`combat/tests/m16_orbs.rs`; register in existing module owners.

**Interfaces:** Keep current `cards::payload` and `cards::public_values` adapters.
New `defect::payload(&State,&LiveCard,Option<u64>)
-> Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>` and
`defect::public_values(&LiveCard)->Option<CardPublicValues>` join that dispatch.
Create `M16_DEFECT_CARDS`, `supports_permanent(CardId)->bool`,
`supports_generated(CardId)->bool`. `OrbState`, `OrbPassivePhase`, `channel`,
`invoke_passive` and `change_slots` have the signatures in the design. Existing
`CharacterEffect` variants adapt into these operations; newly queued work retains
`Effect`/`Source` ownership and validator checks.

- [ ] Add behavior tests named `m16_orbs_focus_empty_capacity_and_bounds`,
  `m16_orbs_dark_identity_and_target`, `m16_orbs_plasma_phase_and_focus`,
  `m16_orbs_capacity_and_sparse_barrage`, `m16_orbs_recursion_fission_terminal`.
  Use real card plays with base/upgraded pairs, not only direct state setters.
  Fixtures can set initial state to isolate a boundary; they must then run the
  real effect/continuation. Pin these exact assertions:

  ```text
  Focus with no occupied orb: accepted; zero-slot Consume -> F+2/+3, slots0
  Dark fresh D6 -> end passive D12 -> F=-10 -> D12; Recursion preserves12
  lowest HP [9,4,4] -> source-order first4; no RNG tie choice
  in m16_orbs_dark_identity_and_target, CompileDriver [DarkD6,DarkD12]
  draws1, not2; adding Frost draws2
  Plasma F=-9/+9: natural start +1, end +0, evoke +2
  Capacitor base3slots ->5; upgraded ->6; repeated ->10, never11
  Consume removes last occupied Plasma without evoke; last empty also removed
  sparse [None,Dark,None,Frost] front evoke no-op; Barrage base hits2
  reuse existing Barrage [0,1,3,5,10]/sparse-five-of-ten table at 4/6 each;
  add real Capacitor/channel-produced high-capacity Barrage, not duplicate tables
  Fission Frost+Plasma: base n2 resource gain without orb effects;
  upgrade adds individual Frost5Block and Plasma2energy to n2 gain
  ```

- [ ] Run `cargo test -p spire-core --locked --offline m16_orbs`; expect semantic
  reds for absent producers/state. A compile red from a new type is not behavior
  evidence: join minimal types, then capture the failing assertions.
- [ ] Implement definitions/use/cost/upgrades for Darkness1, Fusion2/1,
  Capacitor1, Consume2, Defragment1, Recursion1/0, Fission0 and the existing9.
  Use exact printed base/upgraded values from paired records; Darkness+ invokes
  present Dark passives after channel; Fission always exhausts. Positive Focus
  no longer requires occupied slots. Add producer-specific validation instead
  of relaxing all arbitrary Focus/orb payloads. Preserve Shield Bash's branch.
- [ ] Add retained-fault tests: invalid Dark payload ownership or >10 slots is
  rejected without a partial producer commit; forced RNG exhaustion during full
  slot eviction retains current effect, slot state and cursor; repeat observation
  cannot evoke twice. Lethal first evoke prunes later Fission/channel work via the
  existing terminal owner. Reuse existing RNG exhaustion helpers.
- [ ] Rerun the focused filter and existing Barrage/CompileDriver/Frost/ShieldBash
  test filters affected by the changed representation; require new tests green
  and existing semantic expectations retained. Do not rewrite old expectations
  to disguise a behavior change. Record any source-backed correction explicitly.

### Task 2: Orb damage, phase and item interactions

**Files:** Modify `combat/cards/defect.rs`, `combat/{characters.rs,effects.rs,
engine.rs,powers.rs,hooks.rs,turns.rs}`, `items/{mod.rs,relics.rs,potions.rs}`,
`content/{items.rs,ids/items.rs}`. Create `combat/tests/m16_orb_interactions.rs` and
`items/tests/m16_defect.rs`, register with existing tests.

**Interfaces:** Existing `DamagePayload::Orb`, shared damage processing,
`powers::apply_effect`, `Hook` dispatch and `CombatItemContext` remain owners.
Add typed passive-group causes to distinguish natural phase, Darkness+, Loop
and EmotionChip; Cables is wired at its qualified call sites. Add item-local
EmotionChip armed state inside `CombatItemContext`, not Run inventory duplication.
No post-damage `DamageResult.hp_lost` proxy for the earlier Discharge callback.

- [ ] Add paired tests `m16_orb_interactions_electro_lockon_defenses`,
  `m16_orb_interactions_bias_loop_cables_chip_order`,
  `m16_orb_interactions_discharge_damage_stage`,
  `m16_orb_interactions_item_use_and_reset`. Pin Lightning3/Evoke8 ×1.5 ->4/12
  at F0; Lock-On duration1 and3 give same multiplier; Artifact prevents duration;
  Strength/Weak/Vulnerable do not alter orb amount. Electro channels2/3 and hits
  each eligible enemy once per invocation. Loop1/2 repeats front passives without
  removal; Plasma supplies1 per Loop activation. Bias blocked by Artifact leaves
  +4/+5 Focus and no later loss. Natural next Bias1 reduces Focus by1.
  In this same named Bias/Loop/Cables/Chip test, Bias1 at Focus0 with no Artifact
  reduces Focus to-1 for both zero slots and three empty slots; orbs remain empty.
- [ ] Pin source-order tests with the independently established callback addendum:
  Cables adds only eligible front-group invocation, never doubles Loop; Chip
  arms only on actual HP loss. With front DarkD6/Focus2, Loop1, Cables,
  armed Chip and Bias1 at ordinary turn start, Loop grows8 immediately, queued
  Impulse grows twice8 before queued Bias loss: final D30/Focus1. Darkness+
  with existing frontD6/Focus0 and one free slot produces frontD18/newD12
  with Cables; Loop alone receives no Cables extra. Static Discharge1/2
  responds to its earlier
  qualifying normal-damage value and excludes self/THORNS/HP_LOSS. Pair fully
  blocked damage, nonzero onAttacked then later zero HP loss, and actual HP loss.
  Use a small existing shared loss reducer if accepted; otherwise a typed
  unit seam for the stage proves the distinction without inventing a new item.
- [ ] Run `cargo test -p spire-core --locked --offline m16_orb_interactions`;
  capture meaningful reds. Implement actual Bullseye8/11+LockOn2/3,
  Electrodynamics2cost, BiasedCognition1cost, Loop1cost, StaticDischarge1cost;
  use exact callback/power stacking/lifecycle, not global invocation multipliers.
- [ ] Implement DataDisk start Focus1 even before any orb; Cables/Chip callback
  state, FocusPotion+2 and PotionOfCapacity+2slots through active item context.
  Sacred Bark potency doubles these amounts. Test full cap saturation, no partial
  potion replay after an injected downstream fault, combat reset and character
  exclusivity. Extend `validate_combat_inventory`; definitions alone do not pass.
- [ ] Add `m16_orb_interactions_numeric_and_continuation_faults`: Focus clamp at
  ±999, energy998+Plasma evoke ->999, Dark stored D beyond1515 retained, source
  int32 addition boundaries and binary32 conversion cases, invalid producer or
  missing item context faults preserve current work. Verify source-defined
  signed-to-payload conversion separately from field wrap.
- [ ] Rerun this filter plus new item filter
  `cargo test -p spire-core --locked --offline m16_defect`; preserve existing
  defensive/Artifact/terminal tests affected by shared damage joins.

### Task 3: Power-use callbacks and bounded repeats

**Files:** Modify `combat/cards/defect.rs`, existing accepted play-frame/copy
owners, `combat/{hooks.rs,powers.rs,engine.rs,card_state.rs}`, content definitions.
Create `combat/tests/m16_power_callbacks.rs`; register once.

**Interfaces:** Extend the actual accepted play-frame API after reconciliation,
never a second autoplay queue. `defect::card_played(&mut State,&Source)
-> Result<(),EffectError>` and `defect::turn_started(&mut State)
-> Result<(),EffectError>` join existing hook dispatch. Repeat requests preserve
same-family identity, captured energy and autoplay/purge flags. Existing
`LiveCard::stat_equivalent_copy` / `same_instance_family_copy` remain canonical.

- [ ] Add `m16_power_callbacks_notify_repeat_and_expire` and
  `m16_power_callbacks_fault_custody`. Install Storm1 and Heatsinks1/2, then play
  Defragment: actual Power notification channels1 Lightning and draws1/2.
  Amplify1/2 repeats next eligible1/2 nonpurge Powers once each; base Echo Form
  is Ethereal, upgrade is not; source-position Echo guard resets next turn.
  Combined Echo+Amplify produce independently queued repeats and no recursive
  purge repeats. Assert real Focus/slots, play history, card disposition and
  energy; do not assert only callback counts.
- [ ] Run `cargo test -p spire-core --locked --offline m16_power_callbacks`;
  capture behavior reds on accepted shared repeat/cost infrastructure.
- [ ] Implement Storm1cost (upgraded Innate), Heatsinks1cost/intensity1/2,
  Amplify1cost/charges1/2 and EchoForm3cost/intensity1 with source predicates,
  hook insertion and expiry. Do not substitute active-power count for notified
  uses or scalar multiply the original effect. Preserve existing Silent repeat
  behavior and source exception flags.
- [ ] Exercise zero-energy autoplay, full-hand draw overflow, a repeated Power
  whose first effect ends combat, and a forced failure between original and
  repeat. Accepted prefix stays accepted; pending copy has a unique owner; a
  fault cannot replay original payment, callback or card cleanup.
- [ ] Rerun focused callbacks and only affected existing copy/autoplay filters;
  require semantic green without a task-specific reviewer dispatch.

### Task 4: Persistent card identity, learned misc and combat mutations

**Files:** Modify `combat/{card_state.rs,model.rs,engine.rs,view.rs,piles.rs}`,
`combat/cards/defect.rs`, `run/{model.rs,decision.rs,combat.rs,mutations.rs}` and
content rows. Create `combat/tests/m16_mutations.rs`,
`run/tests/m16_defect_mutations.rs`; register with existing owners.

**Interfaces:** Add `misc:i32` to permanent/live/public card state; constructor
GA1/others0. Retain existing permanent_live_card adapter, add
`permanent_live_card_with_misc(CardId,CardInstanceId,u32,i32)->LiveCard` for Run
entry. Create `PermanentCardMutation` with exact design fields and
`Combat::permanent_card_mutations(&self)->&[PermanentCardMutation]`; ActiveCombat
tracks applied-mutation ordinal. Add
`prepare_card_mutations(&RunState)->Result<Option<(Vec<(u64,i32)>,u64)>,RunError>`
in `run/combat.rs`, joined by existing `sync_permanent_gains`. Return absolute
misc values/ordinal only after preflight, then commit once.

- [ ] Add `m16_mutations_genetic_capture_and_copy_family` and
  `m16_defect_mutations_consecutive_death_fault_stasis`. GA misc1 gains old
  calculated1Block while permanent/live misc becomes3/4; next combat starts3/4.
  Upgrade preserves learned misc; same-name other family unchanged. Fresh copy
  learns locally without master change; same-family repeat grows original twice
  only if two growth actions actually execute. Temporary upgrades do not become
  permanent upgrades or reset misc.
- [ ] Run `cargo test -p spire-core --locked --offline m16_mutations` and
  `cargo test -p spire-core --locked --offline m16_defect_mutations`; join minimal
  fields before recording behavioral reds. Add exact later-fault case: accepted
  GA growth, then forced unrelated continuation fault -> Run misc already3/4;
  observation/retry/settlement does not grow again. Death keeps prior growth.
- [ ] Implement GeneticAlgorithm1cost/Exhaust/misc growth2/3; source matching set
  includes playing card, hand/draw/discard/exhaust/limbo with object deduplication.
  Preflight all matching owners/journal capacity before one growth commit; append
  journal only for matching permanent identity. Sync after attempted action
  before returning its fault; keep existing fault precedence/custody.
- [ ] Add forged/missing master identity, wrong card kind, ordinal gap/replay,
  duplicate physical owner and missing Stasis/queued-frame exit-census cases.
  Preparation failure must mutate no permanent owner; a settled visit cannot
  update twice. Reuse Stasis lifecycle, do not add another retained-card store.
- [ ] Add `m16_mutations_claw_wear_streamline_reset`: Claw3/5 deals then grows2
  in specified piles, fresh Claw does not inherit; Steam6/8 declines stored
  baseBlock through negative while calculated Block floors0, upgrade adds2 to
  current wear; Streamline15/20 initialcost2 preserves source zero-turn-price
  branch and changes same-family combat instances only. Next combat resets all
  three local mutations while GA misc remains learned. Include stat-copy versus
  same-family copy and Confused/temporary-zero price using accepted cost layers.
- [ ] Implement these three cards and blue `can_upgrade(CardId,u32)->bool` /
  `upgrade(&mut LiveCard)->Result<(),InputError>` dispatch from existing Run
  mutation/upgrade owners. Do not route blue upgrades through Ironclad fallbacks.
  Rerun new filters plus affected existing permanent-census/temporary-upgrade
  tests; no broad mutable-cost refactor beyond actual accepted seams.

### Task 5: Authored native profile and versioned public projection

**Files:** Modify `run/{model.rs,combat.rs,decision.rs,mod.rs}`, core public views;
create `spire-env/src/run_schema_m16.rs`; modify env
`{run_observation.rs,combat_policy.rs,lib.rs}`. Create
`spire-env/tests/m16_defect_observation.rs`, `run/tests/m16_defect_profile.rs`.

**Interfaces:** `M16DefectCombatStart::{Canonical,Supplied(AuthoredRunStart)}`;
`Run::from_m16_defect_combat(M16DefectCombatStart,RunRng)->Result<Run,RunError>`.
It validates before `from_validated_combat_resources` and does not create a
RoomScenario. Public projection functions/signatures/schema ID are in the design;
use `RunEncodingProfile::M16Defect` in the shared encoder. No new step/reset loop.

- [ ] Add `m16_defect_profile_canonical_and_supplied_admission`: exact canonical
  resources, finite supplied29 admissibility/base-upgrade limits, exclusive
  items, zero/duplicate identity rejection, unknown unsupported card rejection;
  validation failure leaves resources/RNG untouched. Canonical is authored
  combat starting resources, never a normally traversed map run.
- [ ] Add `m16_defect_observation_public_values_and_holes` and
  `m16_defect_observation_capacity_privacy_and_compatibility`. Assert ten orb
  rows encode sparse positions, kind0/1/2/3/4, Dark D12 known only for Dark,
  misc and mutated cost/Block exact; zero slots vs empty slots vs padding differ.
  In `m16_defect_observation_public_values_and_holes`, for both canonical and
  supplied decisions, assert actions row i represents `PublicDecision.candidates[i]`
  in original order through the shared verb/source/target row relationships.
  Assert descriptor action-space n, actions row capacity and action-mask length
  all equal `caps.actions`; mask is1 exactly at candidate rows and0 at padding.
  No filtering/reordering/paging or new dispatch/token format is introduced.
  Encode same public state with different hidden draws/RNG/queued enemy moves
  -> identical features; no copy-family IDs/journal ordinals. Retained outputs
  stay owned after next state change. Capacity9 on real ten-slot state returns
  required10/capacity9, not clipping. Existing descriptor bytes/fingerprints
  remain unchanged; wrong character/schema rejected.
- [ ] Run `cargo test -p spire-core --locked --offline m16_defect_profile` and
  `cargo test -p spire-env --locked --offline --test m16_defect_observation`;
  capture reds, then implement constructor and projection. Permanent misc and
  new orb detail columns occur only in new descriptors; old supported outputs
  remain byte-identical. Keep default Gym66 sidecar and M10 policy-v3 frozen.
- [ ] Rerun these tests and impacted existing schema/policy tests. A new native
  encoder is not a claim that a Python extension was built or imported.

### Task 6: Fixed executable evidence, integrated check and final review

**Files:** Create `crates/spire-core/examples/m16_defect_combat.rs` and
`docs/milestones/m16-defect-combat.md`; update only relevant public guide links.
Use existing example CLI/error conventions, not a new verifier/strategy layer.

**Interfaces:** Example consumes the Task5 constructor, `Run::enter_combat`,
`Run::decision`, `choose_combat` and Task5 native encoder. Semantically replay
fixed actions against a cloned identical Run and compare public states/resources;
no second rules implementation or native session claim.

- [ ] Freeze scenario table before first evidence execution: seed7/budget8192,
  starter canonical Cultist; labelled supplied four-orb/slot-cap scenario,
  callback scenario, two-fight GA scenario and Death control. Fixed semantic
  card/target choices exercise real mechanics and fail if choice is unavailable;
  no enemy/stat/outcome substitution or adaptive resource search. Unit tests
  own individual rule values; example owns coherent playable evidence.
- [ ] Add small example argument/label and outcome assertions; run focused test
  once red, implement minimum CLI and native reporting, then green. Print exact
  label/source identity, resource checkpoints and genuine outcomes, including
  budget/unsupported/fault distinctly from Death.
- [ ] Run `cargo run -p spire-core --locked --offline --example m16_defect_combat
  -- --seed 7 --scenario all --budget 8192` once on the stable candidate. Record
  all cases, actions and outcomes. Starter winning is not an acceptance promise;
  actual four-orb and two-fight effects must execute in supplied witnesses.
- [ ] Run ONE stable `cargo test --workspace --locked --offline`; retain complete
  exact results and pre-existing qualified failures. Run `git diff --check` and
  check task-owned untracked doc whitespace/relative links explicitly. No Python
  rebuild/test, timing benchmark or repeated full baseline is implied. Later
  code fixes justify affected focused tests and one updated stable result.
- [ ] Write profile with exact supported29 cards/items, APIs, source grades,
  authored/supplied labels, test commands/results and limitations. Record upstream
  accepted SHAs, task-owned changed inputs/modes/hash manifest and evidence.
- [ ] Root obtains ONE fresh independent final milestone review of frozen actual
  implementation/evidence. Address concrete findings with focused tests; no
  per-task review or automatic new human approval loop. Root decides acceptance
  and any separately authorized publication. Plan completion is not execution.

## Planning self-review and handoff

Each roadmap mechanic maps to Tasks1–4, ownership/public integration to Tasks4–5,
and executable evidence/final review to Task6. Five Review Focus classes map to
named semantic and fault tests above. Shared implementation signatures are named
where new; accepted M15 interfaces are explicitly reconciled, not fabricated.
Baseline native facts and bounded static facts are independently retained with
exact hashes; public docs contain only independently worded qualified rules.
The single independent planning review returned READY with three nonblocking
notes. This documentation-only pass addresses them for the same reviewer’s
bounded acknowledgment; the original freeze/report remain retained. Actual
primary-session M16 user approval remains pending.
