# M10 Ironclad Act4 and Heart Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:executing-plans for native single-owner implementation, useful isolated factual assistance and ONE fresh independent final milestone review. Steps use checkbox (`- [ ]`) syntax; record actual execution in this plan's ledger. User approved this actual written plan on2026-10-24 ("I approve"); native implementation `.32.2` owns execution.

**Goal:** Deliver the Ironclad A0 character mechanics/content required by M10, genuine key-gated Act4/Shield–Spear/Heart progression, and corresponding numeric Gym/reset/replay/small-native-batch tools without developing strategy.

**Architecture:** Add `RoomScenario::M10IroncladHeart` over the existing Run, combat resolver, owned inventories and captured Acts1–3. Complete the remaining red-card and exclusive-item handlers in coherent native batches, then connect the real Act4 room/settlement owners. Keep M9's profile, default constructor, observation-v1 and replay-v1 stable; make M10 an immutable opt-in with a versioned richer numeric descriptor and the same session/delivery owner.

**Tech Stack:** Existing Rust workspace/Rayon, Python3.12/PyO3/maturin/NumPy/Gymnasium, uv-managed existing Gym-only environment, on-demand prek/Ruff/ty. Existing test frameworks only; no learner, verifier platform, dependency installation or browser/toolchain provisioning.

**Spec:** Approved [36-milestone roadmap](../specs/2026-10-22-narrow-gym-milestones-design.md), §M10 **and the Ironclad A0 character-coverage paragraph after M25**; approved [full-run design](../specs/2026-10-22-full-run-gym-poc-design.md), §§3–7; [accepted M9 profile](../../milestones/m9-act3.md). These are already reviewed design authorities; this plan does not restart their approval cycle.

**Planning baseline:** merged PR45 `ad40ee70947027fcdd5cdf93578f060bb1a2384f`, tree identical to the twice-reviewed publication candidate `5ff1f02`. Planning `.32.1` only, branch `plan/m10-ironclad-heart-20261024`. No product edits/builds/tests/private archive inspection have been performed while writing this plan. Prior counts are historical until execution reruns them.

## Global Constraints

- User: **"strategy is not in scope. We're building the tools to make the strategy"**. No policy tuning, winning-seed search, injected resources/keys, changed enemy values or scripted combat results. Real normal-start losses remain valid outcomes. A supplied edge fixture is not normal-start victory evidence. Report unobserved paths explicitly; do not impose an in-repo win-rate gate.
- M10 includes ordinary Ironclad A0 **75 red card kinds**, all **11 Ironclad-exclusive relics** and **3 exclusive potions**, with actual acquisition/upgrade/selection/event interactions. A few Heart-capable cards or an inert catalog are not character completion. Full shared standard-profile/pool closure remains M26–28; other characters/A1–20 remain later milestones, not silently accepted here.
- Standard start stays80/80HP,99gold, five Strikes/four Defends/Bash, Burning Blood, three empty potion slots, no keys. Keep immutable prior boss histories distinct from starting resources. Preserve M9 seed7/11 unsigned64 little-endian roots and actual old traces.
- One Rust Run, one combat resolver, one inventory/census/reward owner. No Python gameplay, second approximate simulator, universal snapshot codec, callback DSL, source-census prerequisite or custom verification platform.
- Preserve M5 Act2Ready17, M6 source-Proceed chest26, M7 Act3Ready34, M8 source-Proceed chest43, M9 no-key ending and all-key **pre-door3/50** cutoff. M10 alone continues the actual fourth all-key Heart choice into Act4. No M9 heal/door/floor/bonus leak.
- Existing reward policy is **+1** for each new highest Acts1–3 floor, **+10** for each new highest Act4 floor, **+5** for each first distinct actual key, **−1** for Death/settled EmptyDeck, **0** extra for Act3NoKeys and **+100 once** for actual HeartVictory. No door/transition/readiness bonus or operational-error penalty. The existing `run/reward.rs` already encodes Act4 +10.
- Invalid/stale actions are preflight-atomic. Accepted failures retain source, original/pending work, gains, census, latest RNG and first fault; never rollback/reset/retry or manufacture Death/success. Earned Reaper healing is not discarded merely because the last enemy died; no revived frame/clock proposal.
- Public observations expose only entitled public state. No hidden draw rows/order/count leakage, private enemy history, future offers, RNG/seed/cursors or raw resolver handles. Fixed numeric spaces and masks, explicit transport overflow, owned exports and native delivery ownership through final Python tuple remain mandatory.
- Manual reset; no base artificial truncation/time limit. Native batch inputs preflight as a whole, all workers quiesce after accepted faults, and every slot retains its actual result/evidence. No Python loop of environments pretending to be a native batch.
- Reference rows/citations retain their grades, null/provisional IDs, conflicts, RNG-001/TIME-001 and attribution. Task-local read-only existing-tool static lookup is for concrete named blockers only, not a whole-archive audit, new tooling, original execution or fidelity promotion. No source inspection during planning.
- Existing configured environment is `target/m9-gym-dev-venv`, Python3.12.15/NumPy2.5.4/Gymnasium1.4.0/pytest9.1.1/maturin1.15.0; Torch intentionally absent. Before Task27, obtain explicit M10 local-extension refresh permission if the prior task's grant does not cover it. Never run dependency-expanding ordinary `maturin develop`, sync/install or broad formatting as an implicit prerequisite.
- Execution needs actual written-plan user approval. Planning approval is not implementation approval. No new commits/push/merge/cleanup or shared hooks are authorized by this plan; task completion uses ledger/source manifests instead of commit/deletion defaults. Preserve main, every branch/worktree/stash/evidence.
- Native controller integrates all shared roots. Any factual assistant/reviewer gets a dedicated exact-source worktree and unique absolute target/jobs2. ONE fresh final milestone review, not a review swarm per card/task. The two M9 publication reviewers are not preallocated M10 reviewers.
- Controller native execution uses ignored `CARGO_TARGET_DIR=target/m10-native-20261024`, `CARGO_BUILD_JOBS=2`, locked/offline dependencies and an existing worktree-local ignored TMPDIR if needed. Preserve previous targets/binaries; no `/tmp` quota cleanup. Unique worker targets remain separate.
- Fast deterministic semantic fixtures, literal game values and bounded traces; no sleeps/network or incidental Debug goldens. Diagnose warm focused >5s/workspace >30s with build time separate. Preserve the exact seven historical Rust assertion bodies unless a demonstrated scoped defect legitimately changes them; do not repair unrelated legacy Python verifiers or call the whole workspace green.

## Review Focus

1. A selected/autoplayed/repeated/exhausted card that spawns further draw, HP-loss, heal or selection work must keep its real parent/card/payment and accepted gains through terminal/Time-Warp/retaliation boundaries (Tasks2–13,25,27).
2. Feed growth, mutable card state, relic thresholds/counters and optional potion selection must have exactly one owner; invalid choices or later faults cannot duplicate gains, lose an instance or desynchronize Run/combat resources (Tasks2,5–9,14–18).
3. All-key progression must use actual earlier-room key actions, the fourth real ending choice and every Act4 source; no key flag/near-Heart fixture, stale Proceed, dead partner or inherited ordinary reward may fabricate a complete run (Tasks18–23).
4. Facing/Back Attack, Beat of Death, Invincible allowance/reset and Heart buff history must use qualified actor/action checkpoints, not UI labels, flat probabilities or a global simultaneous-lethal shortcut (Tasks19–22,24).
5. New M10 public values, tokens, codecs and batch/facade delivery must agree with native execution without silently changing M9's ABI, leaking hidden state, dropping actions or retrying accepted work after output failure (Tasks24–29).

---

## Observed implementation gap and selected boundaries

Static inspection finds20 red typed identities: Strike, Defend, Bash; Anger, IronWave, Cleave, PommelStrike, ShrugItOff, TwinStrike, Clothesline, HeavyBlade, PerfectedStrike, Thunderclap; Bloodletting, BurningPact, Disarm; Bludgeon, Impervious, Offering; Inflame. Actual payloads exist, although Bash's older `definition_only` metadata still needs reconciliation against its real handler. The **55 remaining identities** are assigned exactly once in Tasks2–13 and17 below:54 card bodies first, then Infernal Blade with its complete28-Attack generator support. The reference roster is3basic/20common/36uncommon/16rare, not75 ordinary reward candidates: the nonbasic red pool is72.

Burning Blood, Black Blood and Magic Flower are the three existing exclusive relics. Missing eight: Red Skull, Paper Phrog (`relic.paper_frog`), Champion Belt, Charon's Ashes, Self-Forming Clay, Brimstone, Mark of Pain, Runic Cube. Blood Potion exists; Elixir and Heart of Iron do not. No new Ironclad-exclusive event is invented: Task17 must close the character-specific branches and eligibility of the declared shared event captures and explicitly identify any actual character-only producer found in the existing references. Broader shared event/pool closure is not inferred.

`combat/enemies/act4.rs` is a placeholder; no typed Shield/Spear/Heart bodies exist. Recall, Sapphire and burning-Emerald acquisition already exist. `act3_end.rs` deliberately stops M9 before door transition. Current Run observation-v1 lacks mutable card statistics, optional-selection details and meaningful facing/cap fields. Small native batching is new, not a pre-existing RunPool.

**Selected:** separate M10 captured profile, real full-red mechanics/eligible producer supports, a narrow owned Act4 route, opt-in numeric-v2 and small manual native batch. Rejected: widen M9 in place; implement only a handpicked winning deck; build a generic standard-pool/Act4/batch framework; or tune a caller until it wins. Captured source offers remain explicitly captured—not falsely labelled complete standard random generation.

## Fact prerequisite: qualify before fixing a new oracle

Reuse `data/{cards,relics,enemies,powers}/README.md`, exact records/citations, `docs/research/2026-10-19-act1-content-red-colorless.md` §§2,4–6, the existing relic report, and `2026-10-19-act1-content-economy-potions.md`. Do not repeat their complete audits. Named unresolved chains to close in Task1/owning task:

| Concrete uncertainty | Load-bearing called members only | Candidate reference values, not yet new native oracles |
|---|---|---|
| Fourth ending choice → TheEnding and rooms | `SpireHeart.goToFinalAct`, `DoorUnlockScreen.exit`, actual dungeon transition/`TheEnding.generateSpecialMap`/entry | after floor50, Rest51→Shop52→Elite53→Heart54; A0 transition healing and its order need closure |
| Shield/Spear roles/facing | actual constructors/`getMove`/`takeTurn`, Surrounded/BackAttack callbacks, Shield Smash helper | HP110/160, Artifact1 each; Bash12/Strength−1, Fortify30/group, Smash34/calculated-output Block; Spear5×2+Burn2 discard,10×3,group Strength2; pair permutations, not IID |
| Heart timing/cap/buffs/publicness | constructor/AI/payload, BeatOfDeath and Invincible callbacks, actual damage/terminal/public projection call sites | HP750; Beat1; cap300; Debilitate2 Weak/Vulnerable/Frail +one of five statuses;2×12/40 attacks; qualified buff-count schedule; private allowance fields are not automatically observations |
| Act4 elite and final reward/end sources | actual room reward producer, Heart death/settlement/Proceed owners | do NOT inherit25–35/95–105 gold, ordinary card/potion/relic screens or an extra chest without source closure |
| Card composition gaps | only called upgrade/copy/selection/autoplay/Fatal/hook members needed by the owning batch | Brutality printed HP-loss-first vs draft queued draw-first; Havoc/DoubleTap origins, X payment, exhaust callbacks, Feed/Reaper HP attribution |

Record semantic findings and explicit boundaries in `docs/research/2026-10-24-m10-act4-and-ironclad.md`, not raw game code/bytecode. Resolve provisional enemy/power identities only when a named executable handler needs them; update affected existing reference rows/citations together if warranted. If a candidate literal is contradicted, rule and amend its dependent test/plan **before** implementation, not by quietly weakening a failing oracle. If a fact remains unqualified, leave that task explicitly incomplete; never a successful stub.

## File responsibilities and pinned interfaces

| Owner | Files and responsibility |
|---|---|
| Card rules/state | `combat/card_state.rs`, new `combat/cards/ironclad/rules.rs`; actual upgrade law, local growth/cost/disposition; no second pile owner |
| Card batches | keep `combat/cards/ironclad.rs` as dispatch; add focused `ironclad/{upgrade,direct,support,choices,copies,autoplay,growth,fatal,powers,exhaust,reactive}.rs`; real payload/continuation owners |
| Shared resolver/selection/knowledge | `combat/{model,selection,engine,piles,hooks,effects,powers,view}.rs`, new `cards/ironclad/state.rs`; finite typed effects, public entitled draw knowledge and owned suspended parents, not a DSL |
| Exclusive items | new `items/ironclad.rs`, existing item/potion/registry owners; acquisition, live hooks, counters and real potion selection |
| Captured progression | new `run/rooms/{m10_content,act4}.rs`, existing profile/key/ending/reward/settlement paths; M10-only admission and exact sources |
| Act4 actors | `combat/enemies/act4.rs` dispatch plus `act4/{shield_spear,heart}.rs`, existing formation/canonical damage/power owners |
| Native tools | existing `spire-env/src/run_*`, new `run_schema_m10.rs`, `run_batch.rs`; same session, versioned descriptor/codec and native batch |
| Bindings/report | existing `spire-python/src/run_env.rs`, `python/.../run.py`, `_native.pyi`; existing report public snapshot/action/observer seams; no Python/browser gameplay |
| Evidence | `combat/tests/m10_*.rs`, `run/tests/m10_*.rs`, public `tests/m10_heart.rs`, env `tests/run_m10*.rs`, Python `test_run_m10*.py`; examples/profile/ledger |

All unqualified core paths above are under `crates/spire-core/src/`; env/binding/report paths are under their existing crates. Do not reorganize unrelated owners.

Pinned cross-task interfaces (add only in their owning task):

- Tasks2–13: keep existing `payload(&State,&LiveCard,Option<u64>)->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>`. Focused child modules use this signature. `EffectKind::Ironclad(IroncladEffect)` dispatches to `ironclad::apply(&mut State,&Effect,&IroncladEffect)->Result<(),EffectError>`; each task adds only its named finite variants.
- Task2: `rules::upgrade(&mut LiveCard)->Result<(),RunError>` and `rules::can_upgrade(CardId,u32)->bool`; `CardPublicValues` has `base_damage,base_block,magic_number,damage_bonus:Option<i32>`, `cost_kind:CardCostKind`, `free_play,exhaust_on_finish,ethereal,innate:bool`, `temporary_cost:i32`. `CardCostKind::{Fixed,X,Unplayable}` gets explicit v2 codes1/2/3 (0padding), not discriminants. Add `LiveCardView.values:Option<CardPublicValues>`; no values inferred in Python. Persistent vs temporary upgrade identity stays distinct.
- Tasks2/5: typed `SelectionOperation::{Move(SelectionDisposition),Upgrade,Topdeck,CopyToHand{copies:u8},ReturnFromExhaust}` separates operation from movement disposition. `SelectionParent::{Card(CardHandle),Potion{slot:usize,id:PotionId}}` retains actual ownership. Task5 adds min/max selection and `CombatAction::ConfirmSelection`; old required-one/multi selections remain equivalent and never gain spurious cancel/confirm actions. Extend the existing native knowledge owner with entitled top/bottom prefixes and expose `CombatPublic.known_draw:Vec<KnownDrawCardView>` (`card:LiveCardView,from_top:bool,offset:usize`), not the hidden draw pile.
- Task17: `RoomScenario::M10IroncladHeart`, `is_m10(&RunState)->bool`; captured predicates explicitly admit it without changing older-specific cutoffs. New `m10_content` owns finite source rows/eligible supports; public reset cannot override their contents.
- Tasks19/20: native `EnemyId::{SpireShield,SpireSpear,CorruptHeart}`, `EncounterId::{ShieldAndSpear,CorruptHeart}` and typed AI memory; actual original constructor identities remain separately qualified.
- Task21: `Act4Progress` owns the fixed route and accepted transition phase; `act4::{prepare_transition(&Run)->Result<PreparedAct4,RunError>,commit_transition(&mut Run,PreparedAct4),enter(&mut Run,NodeId)->Result<RunTransition,RunError>,finish_room(&mut Run)->Result<RunTransition,RunError>}`. Internal prepared type is defined by Task21, not an undefined later dependency.
- Tasks24/25: retain `RunSessionConfig`'s existing fields and `RunSession::new` as M9. Add `RunSession::new_m10(RunSessionConfig)->Result<Self,RunSessionError>` and `observation_spec(&self)->Result<RunObservationSpec,RunSessionError>`; private profile mode selects the same owner/encoder/replay path. `run_observation_spec`/`encode_run` stay v1; add `m10_run_observation_spec`/`encode_run_m10` with corresponding existing parameter/result types.
- Task26: `RunBatch::new_m10(Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>`, `reset(&mut self,&[Option<u64>])->Result<Vec<RunBoundary>,RunBatchError>`, `step(&mut self,&[RunDecisionToken],&[RunActionIndex])->Result<Vec<RunStepResult>,RunBatchError>`, `record(&self,slot:usize)->Result<Option<RunEpisodeRecord>,RunBatchError>`, `reproduction(&self,slot:usize)->Result<RunReproduction,RunBatchError>`, `close(&mut self)->Result<(),RunBatchError>` idempotently. `RunBatchError` distinguishes whole-input rejection from retained indexed slot faults. No autoreset or Python vector facade in this milestone.

## Tasks

### Task 1: Execution baseline and load-bearing fact closure

**Files:** new research note above; this plan's ignored ledger; affected existing reference rows only if a real named finding changes them.
**Interfaces:** consumes approved specs and existing evidence; produces a cited fact table used by Tasks2–22, not a new verifier/manifest prerequisite.

- [ ] After actual user approval, normally close planning `.32.1`, create/claim one native implementation child and record source baseline. No force/reclaim. Retain the existing isolated worktree and all artifacts.
- [ ] Run one pre-change matrix: workspace/core, public M1–M9, native M6–M9, env/binding/report consumers, other core integrations, examples and configured fresh-extension environment tests where refresh is authorized. Historical publication evidence was core690/7/public59/env57/binding5/report48/Python195; compare actual failures, not target counts.
- [ ] Close the named fact table, especially original-ID/AI/callback/room/reward uncertainties. Reuse established facts; inspect only an actually blocking called member with existing tools. No private source copied into the repo.
- [ ] Expected: exact old underlying failure identities documented and qualified literal/ordering inputs for each next batch. No test/body may substitute a guessed callback or nominal AI law. No implementation continues behind a load-bearing unresolved fact.

**Execution fact ruling:** [M10 named-member findings](../../research/2026-10-24-m10-act4-and-ironclad.md) qualify the real final TrueVictoryRoom after Heart Proceed. Preserve actual floor55 entry before terminal, not the draft's premature54/205 ending. Tasks21/22/28 below incorporate this source-required correction. Remaining source-specific compositions close at the owning task before its new oracle, as the fact-prerequisite section permits; Task1 is not a new whole-roster audit gate.

**Task3 compatibility amendment:** Qualified random-spot draw insertion excludes
the top gap: nonempty size n uses indices0..n−1, not n+1 gaps. Correcting the
existing shared helper affects earlier-profile producers too. Preserve the v1
observation/action tables, descriptor fingerprint, record schema and old artifacts,
but corrected M9-session records must advertise `spire-gym-native-m9-rules-v2`
instead of `spire-gym-native-m9-rules-v1`. Strict replay must reject the old rules
identity before executing actions; no hidden legacy simulator or record rewrite.
This is an explicit semantic compatibility change permitted by Task2's shared-rule
clause, not an RNG-algorithm change or unchanged old-trace/parity claim. New M10
session/replay-v2 remains separately owned by Tasks24/25. Existing callers' actual
source routes and focused producer regressions must qualify the helper change.

### Task 2: Upgrade/state ownership — Armaments and Searing Blow

**Files:** `card_state.rs`, new `ironclad/{rules,upgrade,state}.rs`; existing cards dispatch/model/view/selection/Run mutations; new `combat/tests/m10_upgrade.rs`.
**Interfaces:** produces the rules/public-values/typed-effect/required-one selection contracts above. Adds just these two card identities and actual handlers; later batches add theirs alongside bodies.

- [ ] Write `m10_armaments_changes_owned_combat_cards_only`, `m10_searing_blow_upgrade_and_copy_are_distinct`, `m10_upgrade_fault_retains_selected_parent`. Literal Searing Blow damage after0..4 upgrades is12/16/21/27/34; one real upgrade adds4+old count using checked source-width arithmetic. Armaments costs1, grants5 Block both variants, upgrades one eligible hand instance/base or all/upgraded, no master-deck upgrade.
- [ ] Run `cargo test -p spire-core --lib m10_upgrade --locked --offline`. Expected runtime RED on the missing handler/upgrade law after minimal ID scaffolding, not only enum compilation.
- [ ] Implement the pinned rule/selection/value interfaces. Preserve copy-vs-reconstruction, per-instance upgrades, true repeat upgrades, actual eligibility and current parent/queue. Route Smith/Ancient Writing through the same upgrade law; do not keep the old universal `upgrades==0` gate for Searing Blow.
- [ ] Migrate M9's typed vocabulary boundary **now** to explicitly reject new out-of-profile enum values without appending them to its v1 tables. Test that the exact old descriptor/fingerprint stays unchanged and a new card passed to v1 rejects. Every later enum addition must leave the whole workspace compiling; do not defer exhaustive consumer matches until Task24 or use ID-only waits. New M10 v2 support still belongs to Task24.
- [ ] A genuinely required shared-rule correction that changes M9 behavior must be reported as an explicit compatibility/identity amendment, with old artifacts retained; do not silently claim old record/rules compatibility or weaken an oracle to preserve a failure count. No second legacy simulator is authorized to hide that conflict.
- [ ] Rerun GREEN plus existing selection/copy/room Smith tests; public values show actual native stats without hidden piles or mutable buffer aliases. Log evidence, no commit.

### Task 3: Direct attacks, constraints and generated statuses

**Files:** new `ironclad/direct.rs`, registry/card flags/piles as needed; new `combat/tests/m10_direct.rs`.
**Interfaces:** consumes Task2 payload/rules; adds BodySlam, Carnage, Clash, Hemokinesis, Pummel, SwordBoomerang, Uppercut, WildStrike, Immolate.

- [ ] Table-test real base/upgraded effects: Body Slam cost1/0/current Block; Carnage20/28+Ethereal; Clash14/18 only all-Attack hand; Hemokinesis HP-loss2 then15/20; Pummel2×4/5/exhaust; Sword Boomerang3×3/4 with action-time living targets; Uppercut13 plus Weak/Vulnerable1/2; Wild Strike12/17+Wound shuffled draw; Immolate21/28 all enemies+Burn discard. Pin zero targets, mid-multihit death and per-hit retaliation.
- [ ] Run `cargo test -p spire-core --lib m10_direct --locked --offline` to runtime RED, then implement actual native payloads/flags and scoped random selection using existing RNG owner. Never pre-sample later random targets or silently prune non-damage disposition work.
- [ ] GREEN plus legacy numeric/THORNS/M8 revival tests. Every added identity executes both variants, not metadata-only registration.

### Task 4: Block, energy, debuff and conditional support

**Files:** new `ironclad/support.rs`; scoped power/flag helpers; new `combat/tests/m10_support.rs`.
**Interfaces:** adds Entrench, GhostlyArmor, Intimidate, PowerThrough, RecklessCharge, SeeingRed, Shockwave, SpotWeakness over Task2 state/payload contracts.

- [ ] Literal tests: Entrench cost2/1 doubles current Block once; Ghostly Armor10/13+Ethereal; Intimidate0cost/all Weak1/2/exhaust; Power Through Wounds2 into hand then Block15/20; Reckless Charge7/10 then shuffled Dazed; Seeing Red cost1/0 energy2/exhaust; Shockwave cost2 Weak+Vulnerable3/5/all/exhaust; Spot Weakness cost1/action-time intent attack-base≥0 grantsStrength3/4, including mixed attack and base0.
- [ ] Run `cargo test -p spire-core --lib m10_support --locked --offline` RED→minimal native handlers→GREEN. Read-only candidate generation may not reveal a hidden conditional result. Preserve full-hand overflow and actual status identities.
- [ ] Recheck Artifact, Block rounding, Ethereal and existing item/Time Warp consumers; no generic end-turn shortcut or duplicate modifier application.

### Task 5: Owned zone selection and optional confirmation

**Files:** new `ironclad/choices.rs`, `selection.rs`, full-selection engine/piles/public/observer seams; new `combat/tests/m10_selection.rs`.
**Interfaces:** adds TrueGrit, Headbutt, Warcry, BattleTrance, Exhume; produces typed source-zone/parent/min/max selection and `ConfirmSelection` needed by Task16.

- [ ] Tests: True Grit Block7/9, random eligible other hand card/base vs chosen/upgraded; Headbutt9/12 then **existing** discard instance to draw top; Warcry draws1/2 before choosing current hand topdeck, exhaust; Battle Trance draws3/4 before NoDraw for rest of turn; Exhume cost1/0 returns one existing non-Exhume exhaust instance, respects full hand/empty support, no constructor/catch-up callbacks.
- [ ] Add `m10_optional_selection_can_confirm_zero_and_retains_consumed_parent`: min0/maxhand has an initial confirm, selected cards distinct, invalid/stale selection leaves state unchanged, and no gameplay action slips in before the child ends. Required selections retain old automatic completion semantics.
- [ ] Run `cargo test -p spire-core --lib m10_selection --locked --offline` RED→typed same-resolver implementation→GREEN. A potion child must not have a fabricated CardHandle. Migrate old fixtures explicitly to `SelectionParent::Card`, preserving fault contents.
- [ ] Existing selection/Time Warp/Stasis/observer tests GREEN. Complete old action serializer/formatter matches explicitly; M9-v1 rejects impossible new confirmation instead of aliasing it to EndTurn or Run Confirm.
- [ ] Headbutt/Warcry also test public **known** top-deck identity/order until consumed or legitimately invalidated: update the existing knowledge owner on topdeck/draw/shuffle/hidden insertion, never expose the unknown tail. Known draw-card stats are limited to entitled facts, not private recomputation. This public projection feeds Task24; M9-v1 still exports no draw rows.

### Task 6: Dual Wield copying with actual child state

**Files:** new `ironclad/copies.rs`; rules/generated-card constructors; new `combat/tests/m10_copy.rs`.
**Interfaces:** adds DualWield; consumes Task5 selectors and actual card state. Produces valid independent red temporary copies for Task7. Infernal Blade waits until Task17, when all28 eligible Attack handlers actually exist.

- [ ] Literal tests: Dual Wield cost1, eligible other Attack/Power in hand,1/2 independent copies; no Skill eligibility or master-deck copies. Pin source-specific copy state, hand overflow and full-hand/no-eligible behavior; don't turn an upgraded parent into automatically upgraded children.
- [ ] Run `cargo test -p spire-core --lib m10_copy --locked --offline` RED→native selection/copy→GREEN. Preserve Searing Blow upgrades, independent instance allocation and retained partial allocation faults. Add typed stat-equivalent vs same-instance-family copy provenance now (private identity, not a learned handle), initialized through existing constructors. Test it on currently implemented cards; Task8 adds Rampage-specific assertions after that ID exists. Keep the existing public copy adapter compatible with its qualified route.
- [ ] Do not introduce Infernal Blade with a reduced implemented-only pool or allow it to generate unsupported cards as a completed deliverable. Task17 owns that handler and complete source support after its dependencies exist.

**Execution source/custody amendment:** Dual Wield's sole-eligible route leaves
that source active and creates1/2 children. Its multi-eligible choice instead
removes the selected source into selector custody and creates2/3 stat-equivalent
children; only held-aside ineligible cards are returned. No discard/exhaust
callback occurs at this replacement. Retain the selected original in typed,
inactive combat custody for validation/Run permanent census, not any active pile
or a master-deck removal. The existing single resolver/census owns this record;
partial-allocation faults retain it and any accepted children. Test this distinction
and actual exit agreement, not merely final hand count. See the M10 source note.

### Task 7: X payment, Havoc and Double Tap owned continuations

**Files:** new `ironclad/autoplay.rs`, typed play-frame state, payment/piles/engine/Time Warp seams; new `combat/tests/m10_autoplay.rs`.
**Interfaces:** adds Havoc, DoubleTap, Whirlwind. `IroncladEffect` gains named autoplay-top/repeat/X-round operations; owned play frames keep parent, actual child, origin, captured energy, payment/disposition/purge state.

- [ ] Tests: Havoc cost1/0, top-draw play with no child payment and imposed exhaust for an actually played child, empty-draw recycle, ordinary unplayable Status/Curse returning to its qualified non-play destination without an end-turn payload, nested selection; Double Tap cost1/next1/2 qualifying Attacks, one free purge repeat each, no recursive charge consumption; Whirlwind5/8 per enemy per captured X, X0/free-play/current-energy changes distinguished. Ethereal is not automatic exhaust merely because a card was autoplayed.
- [ ] Add `m10_nested_play_time_warp_and_terminal_keep_owned_parent`: selection during a repeat,12th-card barrier, lethal retaliation and later faults cannot lose parent/child, double-pay, publish a premature decision or erase earned effects. Use the Task1 origin/order ruling, not a new universal simultaneous-lethal law.
- [ ] Run `cargo test -p spire-core --lib m10_autoplay --locked --offline` to runtime RED; implement finite frames on the existing resolver/queue, not a second dispatcher. GREEN plus original Barrage0/1/5/10, selected-card and Time Warp tests.
- [ ] A suspended publicly started parent/child card remains identifiable in `CombatPublic.playing_cards:Vec<LiveCardView>`; export only already-revealed playing cards, never queued future autoplay identities/counts or private continuation cursors. Include those owned instances in exit/census and v2 rows without fabricating persistent repeat copies.

**Execution source amendment:** Havoc arms exhaust-on-use-once at extraction,
before child canUse. An ordinary unplayable Status/Curse therefore still exhausts
through suppressed-use cleanup; no card-use/end-turn payload or use/after-use hook
is emitted, but actual exhaust callbacks remain. Do not apply the older generic
non-Havoc autoplay-to-discard conclusion to this route. At the12th-card Time Warp
checkpoint, the actual early-end owner clears queued autoplay card entries and
queues their suppressed-use cleanup: Double Tap repeat purges, Havoc child
exhausts, neither executes its payload. Preserve this owned cleanup and current
accepted work before the native ordinary turn end; this is not a broad unrelated-
effects wipe or animation/frame-clock parity claim. See the M10 source supplement.

### Task 8: Temporary stats and same-instance growth

**Files:** new `ironclad/growth.rs`, scoped damage/HP hooks/rules; new `combat/tests/m10_growth.rs`.
**Interfaces:** adds Flex, LimitBreak, BloodForBlood, Rampage, Dropkick; Task2 public values carry actual local state.

- [ ] Tests: Flex Strength2/4 and actual end-turn loss/Artifact interaction; Limit Break doubles signed current Strength, base exhaust/upgraded not; Blood for Blood starting cost4/3, damage18/22, reductions on actual HP-loss notifications in source-qualified active zones with excluded-zone controls; Rampage8 then same-family combat growth+5/8 with no master mutation; Dropkick5/8 and action-time Vulnerable condition before hit, energy1/draw1 only when qualified.
- [ ] Run `cargo test -p spire-core --lib m10_growth --locked --offline` RED→native handlers/hooks→GREEN. Pin copy/recycle/Exhume differences and failed/blocked-to-zero HP damage, not a turn-counter approximation.
- [ ] Recheck displayed cost/damage, Artifact and signed stat consumers; new state resets at the actual combat boundary, not merely when observation is read.

**Execution source amendment:** Blood for Blood notifications visit hand/draw/
discard, not exhaust/limbo/queued autoplay or inactive census custody. Fresh-kind
creation separately catches up from positive damage-event history; stat-equivalent
copies restore actual source cost. Rampage's same-family traversal additionally
includes exhaust/limbo/card-in-use and native started frames, but not inactive
replacements/master deck. Dropkick always attempts its captured hit on a valid
target; only energy/draw require action-time Vulnerable, resolving Hit→Energy→Draw.
Reject a missing target atomically rather than model the source null dereference.
These named-source rules supersede the draft's generic all-owned notification
wording and older conditional-hit summary; they do not add another census owner.

### Task 9: Feed and Reaper durable gains/healing

**Files:** new `ironclad/fatal.rs`, existing DamageResult/HP mutation/Run combat census and exit owners; new `combat/tests/m10_fatal.rs`, `run/tests/m10_gains.rs`.
**Interfaces:** adds Feed/Reaper; reuses actual `CombatExit.max_hp` and extends existing permanent-gain custody only where durable deltas need publication before a later fault.

- [ ] Tests: Feed cost1, damage10/12, eligible Fatal maxHP+3/4 then ordinary heal, no Minion/half-dead gain; Reaper cost2, damage4/5/all/exhaust, heal sums actual HP removed excluding Block/overkill. Source says no lost-heal clock requirement: retain already earned heal when last enemy dies, but do not pre-credit a heal to rescue an earlier lethal retaliation.
- [ ] Pin Magic Flower/Mark of Bloom, maxHP ceiling, simultaneous-source sequential cases, gains before later fault, true Death exit and exact Run combat→persistent maxHP agreement. No direct write to a second Run deck/inventory or a fake victory.
- [ ] Run both `cargo test -p spire-core --lib m10_fatal --locked --offline` and `... --lib m10_gains ...` to RED, then implement shared source-qualified attribution/ordinary heal and retained durable ownership. GREEN plus existing gain-journal/Stasis/item-return tests.

### Task 10: Real player power lifecycle checkpoints

**Files:** new `ironclad/powers.rs`, native hooks/turns/powers; new `combat/tests/m10_power_turns.rs`.
**Interfaces:** adds Barricade, Berserk, Brutality, DemonForm, Metallicize, Combust and their actual native powers; does not infer classification from beneficial/harmful effect.

- [ ] Tests: Barricade cost3/2 retains Block at player start; Berserk cost0 applies Vulnerable2/1 and energy1 at its qualified start hook; Brutality post-normal-draw extra draw1 then HP-loss1 (resolve the recorded text/order conflict first), upgraded Innate; Demon Form post-normal-draw Strength2/3; Metallicize pre-end-card Block3/4; Combust end-turn HP-loss1 and all damage5/7 at the qualified order.
- [ ] Run `cargo test -p spire-core --lib m10_power_turns --locked --offline` RED→owned ordered hook producers→GREEN. No double normal draw, first-turn phantom trigger, blanket buff cleanse or duplicate Power disposition.
- [ ] Move only the demonstrated player Metallicize checkpoint if necessary; preserve separately qualified enemy behavior. Recheck Awakened curiosity, original played-Power census and terminal healing boundaries.

**Execution source closure:** Brutality's post-normal-draw action order is extra
Draw then HP_LOSS; Demon Form shares post-draw, Berserk ordinary start. Metallicize
is player pre-end-card work, not generic TurnEnded. Combust queues HP_LOSS before
null-owner THORNS all-enemy damage; damage sums per-copy amounts while HP loss
counts copies. Keep that state and damage attribution separate. These facts close
the named ordering conflict without changing qualified enemy checkpoints.

### Task 11: Exhaust callbacks and Skill cost/disposition

**Files:** new `ironclad/exhaust.rs`; actual exhaust/power/disposition hooks; new `combat/tests/m10_exhaust_hooks.rs`.
**Interfaces:** adds Corruption, DarkEmbrace, FeelNoPain, Sentinel; exposes one real-exhaust operation used by Task12/Elixir.

**Execution clarification:** Exhaust movement, per-use disposition, notification
and retained ownership are character-neutral shared combat mechanics. Specific
cards/powers supply their listeners and recipes. Verify the shared path with all
four character setups; do not implement an Ironclad-only exhaust engine.

- [ ] Tests: Corruption cost3/2, Skills pay0 and actually exhaust after use; Dark Embrace cost2/1 draws1 per actual exhaust; Feel No Pain cost1 grantsBlock3/4 per exhaust; Sentinel played Block5/8 and exhausted energy2/3. Power removal from play is not exhaust. Exhausted source instance and triggered children remain owned through suspended draw/selection/fault.
- [ ] Run `cargo test -p spire-core --lib m10_exhaust_hooks --locked --offline` RED→same resolver callback path→GREEN. Reuse cost/finish/disposition rather than writing special per-card cleanup; each actual exhaust notifies exactly once.
- [ ] Recheck hand cap, no-draw, generated status/Ethereal, played Powers, Time Warp and Run permanent census.

### Task 12: Batched exhaust actions with per-instance effects

**Files:** `ironclad/exhaust.rs`; new `combat/tests/m10_exhaust_actions.rs`.
**Interfaces:** adds SecondWind, SeverSoul, FiendFire over Task11 real exhaust and Task7 parent custody.

**Qualified execution contract:** Second Wind and Sever Soul capture non-Attack
references at action resolution; repeated front insertion executes them in
reverse hand order. Fiend Fire captures a fixed count, not frozen identities:
each random exhaust selects from the live group. Exhaust listener invocation
occurs during transfer; its queued Draw/Block/Energy children wait behind already
front-queued batch work. These are distinct finite contracts on the shared,
character-neutral exhaust operation, not one universal identity snapshot.

- [ ] Tests: Second Wind cost1 exhausts current non-Attacks and grantsBlock5/7 for each actually exhausted; Sever Soul cost2 exhausts non-Attacks before16/22 damage; Fiend Fire cost2 exhausts eligible hand instances,7/10 per actually exhausted card then source exhaust. Include Sentinel/Dark Embrace/FNP children, zero candidates, full hand/refills and intervening terminal/fault.
- [ ] Run `cargo test -p spire-core --lib m10_exhaust_actions --locked --offline` RED→finite snapshot/continuation rules qualified in Task1→GREEN. Do not recompute a declared snapshot after draw callbacks or count intended-but-uncommitted exhausts.

### Task 13: Draw, HP-loss, attack and Block reactions

**Files:** new `ironclad/reactive.rs`, scoped native hooks/damage/Block/turn expiry; new `combat/tests/m10_reactive.rs`.
**Interfaces:** adds Evolve, FireBreathing, FlameBarrier, Juggernaut, Rage, Rupture; completes54 of55 missing red identities; Infernal Blade is completed in Task17.

**Qualified callback semantics:** Rupture requires positive player-owned damage
loss, not exclusively a card-labelled source; actual Brutality/Combust/Burn
qualify under the named source predicate. Fire Breathing/Juggernaut/Flame Barrier
reaction packets use their qualified THORNS ownership. Juggernaut observes a
positive Block notification even at cap, chooses its target at deferred dispatch,
and completes queued work before the native ready/Decision boundary. Rage expires
at PlayerEndTurn; Flame Barrier expires at the next pre-draw player start.

- [ ] Literal tests: Evolve cost1/Status actual draw→draw1/2; Fire Breathing cost1/Status-or-Curse draw→all6/10; Flame Barrier cost2 Block12/16+per-attacked return4/6 this turn; Juggernaut cost2/actual Block gain→random living enemy5/7; Rage cost0/Attack use→Block3/5 this turn; Rupture cost1/card-attributed actual HP loss→Strength1/2.
- [ ] Run `cargo test -p spire-core --lib m10_reactive --locked --offline` RED→native source-attributed callbacks→GREEN. Distinguish blocked damage, HP_LOSS, THORNS, attack play vs hits, Status creation vs draw, zero/no-target Block gains and turn expiry. No recursion/double draw or per-card bespoke rules duplicated in the facade.
- [ ] All20 existing red handlers plus54 new have real base/upgraded behavior tests; Infernal Blade/full roster equality/production acquisition is Task17, not inferred from this count.

### Task 14: Five missing passive/reactive Ironclad relics

**Files:** new `items/ironclad.rs`, relic registry and specific shared damage/power/exhaust hooks; new item/combat `m10_ironclad_relics` tests.
**Interfaces:** adds RedSkull, PaperPhrog, ChampionsBelt, CharonsAshes, SelfFormingClay; real acquisition and combat ownership, no injected permanent stats.

**Qualified exceptions/placement:** Red Skull initially stacks Strength directly;
later queued negative Strength may be Artifact-blocked after its latch clears,
so the ordinary no-drift check does not suppress that source-defined exception.
Charon's Ashes is null-owner THORNS at top priority, preserving its target vector
before the next batch exhaust; other listeners retain their qualified placement.
Clay's next-turn payout is pre-draw, not the later post-draw checkpoint.

- [ ] Tests: Red Skull Strength3 while HP×2≤maxHP with threshold crossings/heal/Feed maxHP changes and no stacking drift; Paper Phrog Vulnerable multiplier1.75 (raw8→14), not doubled1.5 modifier; Champion Belt Weak1 only at qualified successful Vulnerable application/stack checkpoints; Charon's Ashes fixed3/all on actual exhaust; Clay pendingBlock3 next turn per actual HP-loss event, including multi-hit and self-card sources as qualified.
- [ ] Run `cargo test -p spire-core --lib m10_ironclad_relics --locked --offline` RED→owned real hooks→GREEN. Pin Artifact/zero damage/type exclusions/expired targets and pending gain faults. Each relic is acquired through the actual native item owner in tests.

### Task 15: Brimstone, Mark of Pain and Runic Cube

**Files:** `items/ironclad.rs`, startup/turn/HP-loss items and acquisition registry; new `m10_ironclad_relic_turns` tests.
**Interfaces:** completes all11 exclusive relics; consumes Task13 HP/draw lifecycle and existing startup inventory transfer.

**Qualified opening checkpoint:** Mark of Pain adds its two separate temporary
Wounds to the draw pile after the actual initial opening hand is dealt, before
later start work/selections. This refines the approximate before-opening wording
below; it does not alter the opening hand or master deck, or shuffle the whole
pile. Brimstone's top insertions execute reverse current-formation order, then
the player; Cube's later top-queued draw precedes Rupture's queued Strength.

- [ ] Tests: Brimstone each actual player-start Strength2/player and1/current enemies at its qualified order; Mark of Pain real energy+1 and shuffled Wounds2 before the qualified opening-draw checkpoint; Runic Cube draw1 per positive HP-loss notification, no blocked-to-zero draw. Include later summons, multi-hit, NoDraw/full hand and lethal self-loss.
- [ ] Run `cargo test -p spire-core --lib m10_ironclad_relic_turns --locked --offline` RED→actual hooks→GREEN. Retain current inventory/counters/last accepted work through failed setup/exit; no constructor-only or flag-only acceptance.

### Task 16: Elixir and Heart of Iron

**Files:** potion IDs/registry/handlers and actual combat potion dispatch, Task5 selection ownership; new `m10_ironclad_potions` tests.
**Interfaces:** adds PotionId::Elixir/HeartOfIron; completes three exclusive potions including existing BloodPotion.

- [ ] Tests: Elixir consumes the actual belt slot, permits0..eligible-hand exhaust then explicit ConfirmSelection, emits each real exhaust once; no fake source card, cancellation/refund or second potion use while child pending. Heart of Iron givesMetallicize6/12 with Sacred Bark through Task10's actual end-turn checkpoint. Pin unoffered/outside-combat use, full hand and post-consumption selection fault.
- [ ] Run `cargo test -p spire-core --lib m10_ironclad_potions --locked --offline` RED→native handlers→GREEN; Blood/Bark/Flower/Cube/Clay and export custody consumers remain valid.

### Task 17: Separate M10 profile and complete character reachability

**Files:** new `run/rooms/m10_content.rs`, Infernal Blade in `cards/ironclad/copies.rs`, existing `content/{cards,items,events}.rs`, M10 profile predicates/room captures/shop/rewards/boss/selection; new `run/tests/m10_content.rs`, `combat/tests/m10_generation.rs`.
**Interfaces:** produces `M10IroncladHeart`, `is_m10`, complete red source-support tables and M10-only captures. `m10_content::{red_candidates(CardRarity)->&'static [CardId],infernal_candidates()->&'static [CardId]}` are explicit native tables of implemented eligible IDs, not data-file interpretation at runtime.

- [ ] Write `m10_all_red_and_exclusive_items_have_real_producers`: exact red roster75/rarities3+20+36+16, ordinary nonbasics72, generator-specific exclusions, all11 exclusive relics/three potions, no unknown/definition-only executable placeholder. Assert literal representative **effects** from every earlier batch in addition to coverage equality; reconcile Bash's stale metadata with its real handler. Do not modify unsupported Silent/Defect/Watcher rows to make a catalog-wide test green.
- [ ] Write `m10_infernal_blade_uses_complete_qualified_attack_support`: cost1/0/exhaust, actual28 eligible red nonbasic Attacks excluding Feed/Reaper, temporary0-cost-this-turn restoration, hand overflow and source-local RNG/copy/upgrade rules. Observe real missing-handler RED, then implement using the now-complete Attack bodies/rules; execute generated outcomes rather than reducing the pool to earlier batches. This supplies the55th missing card.
- [ ] Selected bounded capture policy: M10 ordinary card rewards reveal one Common and two distinct Uncommons, boss rewards three distinct Rares; M10 shop reveals one Common/Uncommon/Rare. Select uniformly without replacement within each **declared capture row's** complete eligible red support using owned scoped native RNG. This is explicitly a new captured-offer workload, **not** the standard rarity/rare-pity/upgrade/shop generator; never call it full standard-profile generation. Infernal Blade separately uses its actual qualified generation support/law. Old M5–M9 captures/RNG scopes remain unchanged.
- [ ] M10-only item source extension: add the five Task14 drop relics to the13 existing M9 drop kinds for an18-kind finite sequence; make it available through all three acts and the qualified Act4 elite source. Brimstone occupies M10's shop-only relic slot (no shop Anchor draining the drop sequence); Mark/Cube/eligible Black Blood join captured boss choices with existing Cage/Collar filling unowned eligible rows. Add real exclusive-potion drop/shop captures. Fix exact valid captured prices from the qualified price facts before assertions, not by copying an unrelated item's price.
- [ ] Test native obtain/upgrade/removal/copy eligibility and producer coverage for all red kinds/exclusive items, duplicate/owned disabling and mandatory acquisition children. Existing Shrine/AncientWriting/Falling/Tomb captures must accept the new card types/upgrade laws correctly; identify any actual Ironclad-only event branch from existing qualified evidence and implement it in this task rather than inventing one or silently excluding it. Shared events/standard pools not implemented remain listed as such, not claimed complete by roster coverage.
- [ ] Run `cargo test -p spire-core --lib m10_content --locked --offline` runtime RED→profile/captures/source supports→GREEN, then all accepted public M1–M9 tests. Do not require an up-front full-game manifest before handlers, or edit old examples/policies to accept different outcomes.
- [ ] Keep the existing M9 profile encoder explicit-error-on-M10, with no new v1 vocabulary ID/fingerprint. Reconcile exhaustive profile/confirmation/report/codec matches in this task as needed; no deferred compile blockers.

### Task 18: Earlier-act key choices, finite stock and normal resources

**Execution amendment — accepted offers versus acquired relics:** the original
18-item arithmetic below counted acquisitions, but accepted chest captures consume
their finite entry at OpenChest even when Sapphire or Proceed skips that relic.
The actual upper bound is therefore 15 elite offers + 3 chest offers + 1 Act4 elite
offer = **19**, including key-complete paths. Preserve accepted-offer consumption;
extend the M10-only finite stock with one actual source-qualified normal-drop relic
and its real mechanics, as permitted by this task's larger-bound clause. Do not
recycle skipped entries, refill stock, substitute boss/shop/event relics or prune
routes. The original18 arithmetic is retained below as the superseded premise,
not current proof. M1–M9 and Task17's historical verification remain unchanged.

**Files:** existing rest/chest/burning/combat/reward owners with M10 gates; new `run/tests/m10_keys.rs`.
**Interfaces:** consumes M10 profile, existing real key mutations and capture rows; produces honest playable key/non-key branches for Task21, not a constructor key option.

- [ ] Write `m10_recall_sapphire_and_emerald_are_real_once_only`: Recall consumes its actual campfire option rather than also resting/Smithing; Sapphire excludes that chest's relic but preserves optional Proceed; Emerald requires actual burning-elite Victory/source, not just its marker. Each first key+5, duplicate keys Disabled/no second reward, real potion/full-belt and selection children retained.
- [ ] Pin the finite bound from actual generated topology: at most5 elites per Acts1–3 path, at most15 before Act4. No-key paths can take at most3 ordinary chest relics:15+3≤18. An Act4 path must sacrifice at least one chest relic to Sapphire, so at most15+2+one Act4 elite relic≤18. Brimstone/boss/RedMask/short-Neow sources are outside this drop sequence. Test maximal valid source preparations/skip-owned cases; if qualification finds another draining source or larger topology bound, expand the **real** finite stock or revise the captured profile before continuing—no duplicate fallback, refills or route pruning.
- [ ] Run `cargo test -p spire-core --lib m10_keys --locked --offline` RED→minimal owner/predicate fixes→GREEN. Small supplied room-edge cases prove producer law, not a normal-start key-complete win. Actual normal-start evidence is collected once in Tasks23/28, retaining losses.
- [ ] Cover all acts' opened chests with every captured relic already owned: unowned Sapphire stays available, owned Sapphire is Disabled, and optional Proceed stays legal. Do not retain the old non-M9-act3 exhaustion error in M10 or fabricate a replacement drop.

### Task 19: Real Shield/Spear and encounter-owned facing

**Files:** `enemies/act4.rs`, new `act4/shield_spear.rs`, IDs/encounters/EnemyMemory/canonical damage and scoped power/targeting/view paths; new `combat/tests/m10_shield_spear.rs`.
**Interfaces:** produces actual ShieldAndSpear setup/AI/payload/lifecycle and public facing/side state; later Run entry uses existing `enemies::setup`/`CombatEntry` rather than a special simulator.

- [ ] After Task1 qualification, literal A0 tests: Shield110HP/Spear160HP, Artifact1 each; Shield one Bash12/Strength−1 and Fortify30/group per qualified pair followed by Smash34 with Block based on calculated attack output, not HP lost; Spear opening5×2/Burn2 discard, Skewer10×3 on its scheduled turn, groupStrength2 with qualified non-Skewer pairing. Test conditional pair/history transitions, one partner dead and already-published group work.
- [ ] `m10_surrounded_changes_facing_at_the_qualified_checkpoint`: left/right stable actor identities, actual targeted-card/potion/autoplay rules, no facing change from a nested pile selection; rear damage1.5 with source-qualified rounding, AoE/random/no-target distinctions, removal/one-survivor behavior. Do not guess that every targeted action flips or that the Surrounded marker itself is the damage multiplier.
- [ ] Run `cargo test -p spire-core --lib m10_shield_spear --locked --offline` to runtime RED. Implement actual setup/typed memory/ordinary effects and facing state in the existing resolver, with a separate descriptive public view. Retained queued targeting/first faults must survive partner changes; no replacement/reward merely on one death.
- [ ] GREEN plus partner/Artifact/THORNS/dynamic intent/observer tests; constructor IDs/data citations updated only to the qualified scope, never promoted to original-runtime proof.

### Task 20: Real Heart, Beat of Death and Invincible

**Execution ruling — deterministic headless cutoff:** preserve the existing native
immediate settlement owner after lethal Heart damage and its synchronous qualified
callbacks/earned receipts. Terminal models `endBattle.clear` before undispatched
UseCard/Beat work. This is an explicit clockless simulator mapping of the unresolved
animation-clear versus action-dispatch race, not a claim that a dying Heart cancels
THORNS or an original-runtime parity result. Already-dispatched lethal player Beat
uses ordinary Death settlement, preventing later queued actions. Nonlethal ordinary
after-use work remains ordered normally; no universal simultaneous-lethal rule,
clock subsystem or precredited healing is added.

**Files:** new `act4/heart.rs`; typed setup/AI/damage/power/hook/terminal guards; new `combat/tests/m10_heart.rs`.
**Interfaces:** produces actual CorruptHeart body and public allowance/buff state for Tasks22/24; native combat outcome remains Victory/Death, Run alone decides HeartVictory.

- [ ] Qualified A0 literals: HP750, Beat1, Invincible300; Debilitate is the first selected enemy turn, not a prebattle effect:2 Weak/Vulnerable/Frail plus one Burn/Dazed/Slimed/Void/Wound at actual insertion checkpoints. Blood Shots is **twelve hits of2 damage** (the earlier2×12 notation means damage×hit count), with Echo40 as the other attack in the qualified fair-coin/opposite paired order. Buff clears only the qualified negative Strength state, addsStrength2 and per-use Artifact2/Beat+1/PainfulStabs/+10/+50 continuation as closed in Task1. Nominal50/50 is not an exact sampler until that chain is qualified.
- [ ] Test `m10_heart_cap_spends_actual_hp_loss_and_resets_only_at_owner_checkpoint`: Block does not spend allowance; per-hit damage, Reaper/Charon's/THORNS/HP_LOSS interactions follow the qualified guard/order. **Source clarification:** Invincible spends the capped post-Block callback value before the target's overkill HP clamp; the historical test-name wording does not mean spend only the smaller HP removed by a lethal overkill. Reaper's earned healing still uses actual committed HP loss, not quota spent. Expose remaining allowance only if its actual public display or explicitly entitled public-knowledge projection establishes it; otherwise its observation is unknown, not a private-counter read. Reset once at the actual callback, not generic player turn start by assumption.
- [ ] Test `m10_beat_card_reaction_keeps_payload_selection_and_terminal_owner`: once per qualifying card use, not hit; zero-cost, Havoc/repeat, Power/Skill, nested selection, full Block, lethal Beat/retaliation and lethal Heart damage. Retain the actual accepted action/pending work and source-qualified sequential outcome—no universal simultaneous-lethal priority.
- [ ] Run `cargo test -p spire-core --lib m10_heart --locked --offline` runtime RED→actual native handlers→GREEN. Long buff history is a small deterministic supplied diagnostic, not a policy/turn-count benchmark or Heart victory claim. Recheck Book PainfulStabs and old Time Warp/THORNS/census consumers.

### Task 21: The actual fourth ending choice and fixed Act4 rooms

**Files:** new `run/rooms/act4.rs`; scoped `act3_end`/room predicates/transition/map/rest/shop paths; new `run/tests/m10_act4.rs`.
**Interfaces:** defines `PreparedAct4`/`Act4Progress` and the Act4 methods pinned above. Reuses actual rest/shop/selection owners and public map/node/action types.

- [ ] Write `m10_fourth_all_key_choice_enters_the_real_act4_route`: M10 all-key third choice leaves the fourth actual Heart option; its accepted `Act3Continue{stage:4}` executes the qualified door/dungeon transition once. M9 all-key still stops pre-door3/50 without actions/heal. M10 missing-key branch still takes fourth no-key choice to Act3NoKeys; all-key plus Act4-disabled does not enter.
- [ ] Source-closed expected route is Rest51→Shop52→Elite53→Heart54, then explicit boss Proceed→TrueVictoryRoom55. A0 transition requests ordinary healing with maxHP nominal amount, respecting prevention; there is no separate invented door floor. Preserve source deck/resources/items, once-only ActAdvanced and typed accepted-phase flags through initialization/heal/publication faults. No direct jump from Act3 to Heart or auto-selecting room decisions.
- [ ] Test first legal room moves/source exits and mandatory Smith/purge children; +10 only on each actual first Act4 floor entry, transition/read/duplicate Proceed reward0. Campfire/shop are real owners, not decorative screens. Act4 map is its source-defined fixed route, not an ordinary15-row map or random encounter pool.
- [ ] Run `cargo test -p spire-core --lib m10_act4 --locked --offline` RED→prepare/commit/route implementation→GREEN. Source/rng/frontier preflight is detached; accepted fault freezes with exact original stage, never reruns heal/reset/map construction.

### Task 22: One Act4 combat/reward/Heart settlement owner

**Files:** existing `run/combat.rs`, new Act4 room completion/source code, existing reward/prepared-result machinery; new `run/tests/m10_settlement.rs`.
**Interfaces:** consumes actual Task19/20 bodies and Task21 route; produces Shield/Spear reward source→Heart admission and actual `RunTerminal::HeartVictory` through existing reward/terminal owner.

- [ ] `m10_act4_elite_origin_and_rewards_keep_real_source`: actual visited Elite53, Collar elite role/nonburning classification, both partners resolved, same owned inventory/card census and source-qualified reward contents/skip/mandatory children. Do not inherit ordinary Act3 gold/card/potion capture solely because both are Elite. No Heart admission until the real reward/source is complete.
- [ ] `m10_heart_victory_is_not_a_boss_chest_or_gameplay_death`: actual visited Heart54/A0/Boss source and fully resolved Victory only; retain Feed/gain/heal/disposition, then expose actual boss Proceed rather than terminate early. Proceed enters the source-qualified TrueVictoryRoom55 and commits HeartVictory+100 exactly once, actual floor55 entry+10 once. No synthetic relic screen/+100 on key collection or pre-door readiness; no cutscene/clock subsystem.
- [ ] Test real Death at either Act4 fight, failed setup/return/census/source preparation and accepted final packaging boundary. Fault preserves actual Combat/pending/source/items/first fault; no rollback/clear or retry. If actual ending has player choices, keep them explicit as qualified in Task1, not a guessed terminal checkpoint.
- [ ] Run `cargo test -p spire-core --lib m10_settlement --locked --offline` RED→single-owner routing/settlement→GREEN. Then M1–M9 public tests and gain/Stasis/revival/item-return/observer consumers; do not widen old predicates as a shortcut.

### Task 23: Public normal-start tools example and honest replay

**Files:** new `examples/m10_heart.rs`, `examples/m10_support/mod.rs`, `tests/m10_heart.rs` under spire-core; no old policy edits.
**Interfaces:** `start(seed:u64,history:M9BossHistory)->Result<Run,String>`, `rollout(seed,history,budget)->Result<M10Report,String>`, `M10Report{trace,final_decision,final_map,completion}` using existing public Step/actions/maps. Reuse the history meanings/root adapter, not arbitrary starting resources.

- [ ] Tests literal normal resources, current public legal steps/masks/reward/terminal, seed7/11×three histories and nonattacking Death. Use a fixed simple masked caller or the unchanged existing public combat helper; freeze it **before** observing outcomes, no subsequent strategy-improvement pass. Budgets0/1/>8192/invalid arguments are unfinished/errors, not loss/truncation.
- [ ] Replay every accepted semantic action from the same normal profile/root/partition/ordinal and compare all public boundaries and rewards. If a trace actually reaches no-key/Shield–Spear/Heart, assert its real source and result; otherwise report what was unobserved. Do not convert the roadmap's earlier winning-path wording into a strategy assignment.
- [ ] Complete core Heart tools require actual qualified mechanics and separate labelled native full-chain room/key/Act4 diagnostics, **not** a winning in-repo policy. Never substitute that supplied chain for the missing from-start victory. Real earlier key producers and actual final transition must be executable, not placeholders.
- [ ] Run `cargo test -p spire-core --test m10_heart --locked --offline` runtime RED→public caller/CLI→GREEN, execute both fixed seeds/all histories and every old M1–M9 example. Preserve all actual losses and unchanged M9 outputs. No seed sweep/resource/value edits.

### Task 24: Versioned M10 numeric public state, not a widened M9 ABI

**Files:** new `spire-env/src/run_schema_m10.rs`; shared Run encoder/view/rule adapters and report public snapshot/action formatting; new `spire-env/tests/run_m10_observation.rs`.
**Interfaces:** produces `m10_run_observation_spec(&RunCapacities)->Result<RunObservationSpec,RunSessionError>` and `encode_run_m10(&PublicDecision,Option<&PublicMap>,&RunCapacities)->Result<RunObservation,RunSessionError>`. Schema is **`spire-gym-run-observation-v2`**, M10 profile only. M9 functions/descriptor/fingerprint remain v1-identical.

- [ ] Fix v2 field order once: existing groups retained; `combat_cards` keeps its eight v1 columns then adds `base_damage,base_block,magic_number,damage_bonus,cost_kind,free_play,exhaust_on_finish,ethereal,innate,temporary_cost`; `enemies` keeps ten v1 fields then `side,intent_hits`; new `combat_state` fields `facing,surrounded,heart_damage_remaining,heart_cap`; `selection_counts` appends `minimum,maximum,confirm_available,operation`. Add padded `known_draw[combat_cards_capacity,3]` fields `from_top,offset,combat_card_row`, referencing **only** publicly known draw cards also included in v2 `combat_cards` under explicit known-draw zones. Publicly started playing cards get their own v2 zone; unknown draw cards never receive rows. Every optional field has its native known mask; no N/A value guessed as known0.
- [ ] Native vocabularies append all new profile identities and the actual confirm-selection verb/purposes; stable explicit IDs, no enum-discriminant or string-hash IDs. M9's subset encoder must explicitly reject an out-of-profile new value instead of appending it into v1. Remaining Heart allowance/facing/intent hit counts come only from qualified public views/entitled knowledge, never private RNG/AI/counter memory; test unknown masks as well as known values. Temporary/permanent upgrades, Rampage state, X/free cost and optional selectors remain distinguishable.
- [ ] Reuse row capacities with `RunCapacities::m10_default()` initially equal to m9_default; immutable construction overrides allowed. Test every axis overflow and new fields at source boundaries; measure actual high-water marks in Task28 and raise versioned preset only if needed. No silent truncation, variable episode spaces, paging or hidden draw rows.
- [ ] Run `cargo test -p spire-env --test run_m10_observation --locked --offline` runtime RED→shared encoder/profile tables→GREEN. Add report snapshot/action regressions for confirm/purpose, actual card values/facing, Heart terminal/Act4 rooms; preserve existing replay-observer exports and viewer behavior. Run `spire-report` tests, not a new browser/tool installation.

### Task 25: Same Run session, opt-in M10 and compatible semantic replay

**Files:** existing `spire-env/src/run_{session,replay}.rs` plus a focused private profile helper if needed; new `tests/run_m10_session.rs`, `run_m10_replay.rs`.
**Interfaces:** produces `RunSession::new_m10`/session descriptor as pinned above; old `new` and `RunSessionConfig` struct literals stay source-compatible. Record schema **`spire-gym-run-replay-v2`**, profile `M10IroncladHeart`, rules `spire-gym-native-m10-rules-v1`, content `spire-gym-m10-ironclad-a0-captured-v1`; seed adapter and selected reward ID unchanged.

- [ ] Tests M9 constructor/descriptor/header compatibility and M10 opt-in creation/reset; exact same u64-le root/Partition/ordinal semantics. FirstNone0/0, None continuation checked, explicit reseed ordinal0; all profiles reject non-Ironclad/non-A0/resources/unknown options. M10 continues the real door rather than swallowing UnsupportedBoundary.
- [ ] Replay-v2 encodes every actual new semantic action, including optional `ConfirmSelection`, typed profile/content/rules identities and portable fault summaries. Existing v1 record readers/encoders continue their old legal subset; unknown schema/incompatible fingerprint/profile rejects before Run creation. Do not accept a new M10 action under an apparently compatible M9 header.
- [ ] Pin core/session equality for real seed7/11 traces and ordinal continuation, real terminal classifications/manual reset/false truncation, stale/cross-session indices, initial/post-step capacity and delivery faults. Record accepted prefix/attempted work once; a reported external export fault is not guaranteed to recur. Same live native reproduction owner retains full original fault.
- [ ] Run both new session/replay test targets runtime RED→profile-aware single owner/codec→GREEN plus all existing spire-env tests. No second M10 session engine or private snapshot codec; opt-in recording must not affect gameplay RNG.

### Task 26: Small genuine native Run batch agreement

**Files:** new `spire-env/src/run_batch.rs`, existing scheduler generic map/checked-allocation helpers and pure session preflight; new `tests/run_m10_batch.rs`.
**Interfaces:** defines the RunBatch/RunBatchError API above, owning a stable ordered set of actual M10 RunSessions and a persistent native scheduler. Add pure `RunSession::preflight_step(token,index)->Result<(),RunSessionError>` and reset preflight used before batch mutation.

- [ ] Tests batch sizes1/3 and workers1/2 vs equivalent independent native sessions at seed7/11 and continued ordinals; same observations/masks/semantic actions/rewards/endings/records per slot. Compare actual semantic state, not session-token equality across separately constructed owners. Different slots keep independent roots/history/token ownership.
- [ ] Test whole-batch length/seed/index/stale/closed/terminal rejection before any gameplay advancement. Valid execution faults may partially commit: wait for every worker, fault the batch, retain **each** accepted slot's state/prefix/first fault, reject retry/reset, permit diagnostics/records/idempotent close. Pre-reserve scheduling/result storage; no rollback or automatic replacement episodes.
- [ ] When a failing batch cannot deliver newly advanced successful rows, mark their transport delivery loss too, without replacing an original core fault or rewriting older records. Explicit slot reproduction works even when replay recording was disabled. Test the distinction between unchanged preflight rejection, the actual failing slot and other committed-but-undelivered slots.
- [ ] Run `cargo test -p spire-env --test run_m10_batch --locked --offline` runtime RED→native scheduler/session ownership→GREEN. Existing CombatPool/SAME_STEP behavior unchanged. This is small manual native batching, not M35 throughput hardening or a Python vector facade/performance claim.

### Task 27: Actual refreshed M10 binding and thin Gym opt-in

**Files:** existing `spire-python/src/run_env.rs`, `_native.pyi`, `python/spire_lab/gymnasium/run.py`; new `tests/python/env/test_run_m10.py`, `test_run_m10_ownership.py`.
**Interfaces:** keep `RunEnv(character,ascension,*,...)` and NativeRunSession default M9. Add immutable constructor keyword `profile="m9-ironclad-act3"|"m10-ironclad-heart"`; M10 selects the Task25 same native owner and descriptor-v2. Reset still has no resource/profile override. Static replay dispatches the explicit supported record version/profile.

- [ ] **Refresh permission gate before behavioral body:** verify the existing Gym-only environment, no sync/install. Obtain explicit bounded local-extension refresh authority if not already granted for this M10 work; don't re-ask for the completed environment setup. Preserve the installed binary/evidence. No unvalidated facade or stale-extension acceptance.
- [ ] With authority, refresh minimal test scaffold and observe actual Python RED before body. Build in place using `UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv CARGO_TARGET_DIR=target/m10-python-extension-20261024 CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true uv run --locked --no-sync maturin develop --release --locked --offline --skip-install`; use ignored worktree-local TMPDIR if /tmp quota persists. Existing editable metadata remains; if it genuinely needs reinstall, use approved `uv pip ... --no-deps --no-build-isolation --editable .`, never dependency-expanding develop.
- [ ] Tests default-M9 unchanged, explicit M10 schema-v2/spaces, all new field dtypes/known masks, exact integer-not-bool input/reset/record semantics and public terminal info only. Unsupported profiles/keys/HP/decks rejected; actions are current legal indices, native tokens stay private. No Python interpretation of card text/stats, facing, damage caps or RNG.
- [ ] Extend existing native delivery lease through v2 numeric export and final tuple/RNG bookkeeping; test unprintable BaseException, nested optional selection, reentry/concurrency, retained arrays/info/record across step/reset/close, and accepted delivery fault with no retry. Keep original exception identity and first native fault.
- [ ] Rebuild real candidate GREEN, verify exact import path/build revision **and source/artifact snapshot**, Torch absent, then run new tests plus all existing195 environment tests (counts may grow). Run on-demand prek with only touched `.py`/`.pyi`; no shared hooks/broad format. Default-feature Cargo does not compile PyO3 and is not substitute proof.

### Task 28: Fresh-extension from-start agreement and runnable tools

**Files:** new `examples/run_m10_gymnasium.py`, `tests/python/env/test_run_m10_from_start.py`; extend a scoped native trace exporter or add `spire-env/examples/m10_gym_agreement.rs`; native small-batch evidence.
**Interfaces:** consumes Tasks23–27 actual numeric session/replay/facade, not a new Python policy/rule implementation.

- [ ] On the verified refreshed extension, compare the same seed7/11×three histories' current legal indices/core public decisions/session arrays/binding/Gym/replay records/reward/endings at every actual boundary. Retain actual outcomes, including early losses; name unobserved boss/Act4/Heart sections explicitly and keep supplied qualified branch diagnostics separate. Never inject state through ordinary reset or add strategy tuning to satisfy the demonstration.
- [ ] Conditional true Heart path reward is215 **only if** qualified floors1–50, all three keys, five actual Act4 floors51–55 including real final-room Proceed, and HeartVictory occurred:50+15+50+100. Test exact event sums instead of rewarding a claimed route label. Death−1/no-key0/operational-error0 semantics remain distinct. No terminal/bonus simply for collecting keys or leaving a shop.
- [ ] Measure actual per-group high-water marks and explicit overflow fixtures; preserve full candidate masks and immutable spaces. Check native batch slots agree with serial tools for the same public action streams. Recording remains compact semantic actions, not retaining every padded observation as production state.
- [ ] Example accepts profile/character/ascension/history/seed/budget, samples or uses a fixed untuned masked caller. Execute seeds7/11, nonattacking Death, budgets0/1/8193 and invalid profile. Genuine outcomes exit0; unfinished budget/fault/unsupported are errors, never Gym truncation. No training/strength/throughput claim.
- [ ] Run `UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv run --locked --no-sync python -B -m pytest tests/python/env -q`, relevant native batch/replay tests and changed-file prek to GREEN. Diagnose build vs runtime >threshold separately, no wall-clock correctness assertions.

### Task 29: Declared M10 profile, preservation matrix and ONE milestone review

**Files:** new `docs/milestones/m10-heart.md`; exact scope/status/README/development guidance and this plan's ledger. No automatic M11 product work/publication.
**Interfaces:** consumes the exact Tasks1–28 candidate and explicit qualifications; produces scoped M10 acceptance or named unresolved mechanics/source/binding failures. It does not substitute a good review count for implementation.

- [ ] Document all75 red kinds and real source supports, all11 exclusive relics/three potions, actual event/capture limitations, finite18 bound, all key/Act4/final-source laws, v2 descriptor/preset/seed64/replay/batch/fault semantics and actual normal-start results. State which paths were not observed; do not claim standard-profile completeness, all characters/A1–20, original runtime/Java parity or a winning strategy. M11 remains Silent planning-only after acceptance.
- [ ] One integrated candidate matrix: `cargo test --workspace --locked --offline` with exact historical failures; native `m10_`/`m9_`/`m8_`/`m7_`/`m6_`; accepted public M1–M10; all spire-env/spire-python/spire-cli/spire-report tests; other core integrations/reporting; all ten native examples/both fixed M10 seeds/budgets/unchanged M9 trajectories; real rebuilt Python environment suite/changed-file prek; `git diff --check`. Keep build time and runtime separate. After fixes rerun affected lanes, not endless broad audits.
- [ ] Existing broader Python/readiness/debug-verifier failures remain explicitly disclosed and named if encountered; don't install Torch or rewrite historical snapshots to produce a green label. If this task changes reference rows, run only affected offline reference checks/builders, preserving uncertainty/attribution; those checks are not mechanics proof.
- [ ] Self-check all five Review Focus classes and all cross-task interface definitions/ownership, then ONE fresh independent GPT6Luna reviewer in its own exact-source materialization/absolute target/jobs2. Supply specs/plan/profile/source rulings/actual diff/known baseline and the explicit strategy correction; require graded findings and exhaustive declined judgments. Fix scoped blocking findings natively with RED→GREEN and the same reviewer's scoped recheck, not a new review swarm.
- [ ] Verify exact reviewed product bytes and rerun bounded affected checks before normally closing actual implementation/outcome and activating M11 planning only. Preserve every worktree/stash/evidence; no commit/push/merge/cleanup or M11 implementation is implied by acceptance.

## Planning self-review and approval gate

- **Coverage:** red75/55 missing Tasks2–13+17, exclusive items Tasks14–16, real character producer/event-capture support17, key/source/finite stock18, actual Act4 actors19–20, door/rooms21, one settlement22, public normal-start evidence23, versioned tools24–28 and final integrated preservation/review29. This does not postpone ordinary Ironclad coverage to a future shared-content milestone.
- **Type consistency:** child payload signature remains the existing native one; Task2 defines typed effect/rules/public values and v1 out-of-profile rejection, Task5 defines optional selection/confirmation before Elixir; Infernal Blade follows its complete Attack dependencies in17, not an earlier reduced-pool placeholder; Task21 defines its prepared owner before22; Task24 defines v2 before25 consumes it;25 defines profile/session/replay before26/27;26 defines its own pure preflight and slot reproduction before batch execution. Every enum addition updates existing exhaustive consumers immediately. No undefined later type is a prerequisite for an earlier task.
- **ABI:** M9 remains default/v1 and exact old profile; M10 opt-in/v2 is explicit. New richer public state is not encoded by silently appending IDs/columns to old fingerprints. All profile/codec dispatch remains native; source roots and reward policy are not changed.
- **Proportion:**29 coherent native deliverables for55 missing cards, eight relics/two potions, actual Act4 and tools; no per-card spec/reviewer cycle, universal verifier, full standard-pool audit or learner. Small fixtures/table cases prove actual effects, not just roster counts.
- **Permission:** this written plan needs user review before implementation. Existing environment setup is complete; local M10 extension refresh is a separately explicit dependency-preserving write permission, not an install/sync grant. Keep native execution/useful isolated assistance/ONE final review; do not silently switch to per-task agent/reviewer overhead.

**Approved for native execution** on2026-10-24 by user "I approve". Planning `.32.1` closed; implementation `.32.2` owns the29 tasks. The latest merged baseline is PR45/ad40ee7; old uncommitted-M9/publication-in-progress/plan-wait notes are historical. Publication/install/cleanup and M10 extension-refresh gates remain separate.
