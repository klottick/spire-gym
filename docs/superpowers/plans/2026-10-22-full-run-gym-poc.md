# Full-run Spire Gym PoC Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Preserve the previously requested Luna implementation/component-review arrangement and separate Sol arbiter; do not substitute a fresh-agent-per-assertion workflow.

**Goal:** Deliver normal from-start runs through the Heart for all four fully unlocked characters at A0–20, with public decisions, native single/vector environments and replay for external ML callers.

**Architecture:** Extend the existing combat engine and add owned run coordination in `spire-core`. Native run sessions, encoding, rewards and batching live in `spire-env`; new thin bindings/facades coexist with combat-v1. Shared interfaces land before parallel consumers.

**Tech Stack:** Existing Rust workspace, HMAC/SHA256, serde, Rayon, PyO3/rust-numpy, CPython 3.12 and Gymnasium 1.4.0; no new dependencies.

**Spec:** [Approved full-run design](../specs/2026-10-22-full-run-gym-poc-design.md).

**Status:** user-approved plan/start/worktree authority recorded; `.3` closed. Registry1/.5, run2/.4, maps/world8/.11 and shared runtime3/.6 accepted/closed after component reviews/scoped fixes. Final controller workspace239/0+diffcheck on reviewed runtime; historical TDD gaps/warnings explicit. Only next disjoint cards4/.7, items5/.8 and enemies6/.9 wave allocated; full gameplay/bridge unfinished. Counts do not claim future in-flight green or original-runtime/Python/device/format validation. Main/historical worktrees preserved, no commits/installs.

## Global Constraints

- **Ironclad, Silent, Defect and Watcher, fully unlocked, Ascension 0–20, Acts 1–4 through the Heart.** A20 is the primary performance target.
- “No hidden pool cuts, inert content, approximate fights or automatic choice policy are acceptable shortcuts.” Daily/custom/modded modes, career progression, graphics and ML code are excluded.
- **Source-grounded, not original-runtime validated.** Reuse qualified facts; independently author product code. Do not copy or mechanically translate original source.
- Preserve existing public combat-v1 behavior/schema/seed streams; do not create a second combat simulator, Python rule engine or generic plugin framework.
- New feature values are signed int64; known/row/action masks are int8. Capacities are immutable per pool; overflow errors, never clips, pages, losses or changing spaces.
- Retain independent experienced profiles, project RNG/maps and closed reward decision `spire-8jm.9.1`. Reads/encoding/workers consume no game randomness.
- TIME-001: assume Secret Portal's 800-second condition is always satisfied; preserve other eligibility and random selection. No clock/frame/wait subsystem.
- Ordinary tests, small fixtures and replay only. No parser, symbolic executor, law DSL, generic verifier, receipt pipeline, whole-source census or mutation farm.
- No installs/removals, lockfile changes, commits, pushes, publication, original-game execution or destructive cleanup without separate explicit authority. Handoffs below are **not commits**.
- Use the existing shared Beads board/allocated actors. File ownership below is a future allocation, not a present claim. In-progress means actual current work.

## Review Focus

1. A selected duplicate card is removed/transformed exactly once, not its same-kind neighbor; pending child choices retain the original parent (Tasks 1, 4, 10).
2. Hidden state changes must not change public row counts, references, menu ordering or values (Task 13).
3. A failure after an accepted action cannot permit retry, pretend death, or overwrite old undrained replay evidence (Tasks 15–17).
4. Tiny capacities, huge integer seeds and mixed profiles must preserve shape/atomicity and fail honestly (Tasks 13, 15–17).
5. A20's second Act3 boss, escape and same-step reset must preserve the correct old-run resources/reward and new-run profile (Tasks 7, 12, 16–18).

---

## Planning checklist

- [x] Read the approved spec, current code and existing component/dependency scopes.
- [x] Map exact files and shared interfaces; assign nonoverlapping implementation ownership.
- [x] Write dependency-ordered TDD tasks and focused verification commands.
- [x] Self-review spec coverage, steps, types, review-focus tests and proportion; correct the unsupported Reaper/frame-simulation inference.
- [x] Record the draft/review handoff in Beads and release the active claim while waiting for the user.
- [x] Record user-approved TIME-001 Portal handling in the same spec/plan and intentional deviations.
- [x] Obtain user plan approval (“lgtm”) and explicit implementation-start authority (“You can start”).
- [x] Obtain fresh-worktree consent, preserve the uncommitted approved docs, and run the offline Rust baseline before dispatch (169 passed, 0 failed).

## Timing correction and Portal handling

The user corrected the planning claim: Reaper does not lose its healing because
combat ends. The retained historical notes explicitly did not demonstrate a
naturally reachable lost-heal case; inferring one from separate callbacks and
queue clearing was unjustified. Do not turn that inference into a gameplay rule,
an artificial regression expecting lost healing, or a frame-simulation requirement.
Task 4's Reaper behavior test and Task 7's settlement test must retain earned
healing when Reaper kills the last enemy, applying the actual healing modifiers.

**The global 60-Hz clock/frame proposal is withdrawn.** Implement deterministic
gameplay effect ordering and actual settlement in the existing engine. No
`combat/timing.rs`, animation waits, frame replay or original-runtime investigation
is required by this plan. This correction is user feedback and planning evidence,
not a new original-runtime-validation claim; preserve historical research as-is.

**Resolved:** user says “assume it is always possible,” explaining that ordinary
non-speedrun play meets Portal's requirement. Under
[TIME-001](../../intentional-deviations.md#time-001--secret-portals-elapsed-time-requirement-is-assumed-satisfied),
assume the original 800-second condition is satisfied. Preserve Act 3, other
eligibility/pool/history rules and normal random selection; do not guarantee the
event or change the boss fight, key requirements or A20 second-boss sequence.
No run clock, frame subsystem, wait action, elapsed-time input/feature or real
waiting is required. Task 10 tests this narrow policy; Tasks 15/17 retain normal
replay/worker invariance. Timing is no longer a planning blocker.

## File ownership and sequencing

Paths below are relative to the repository. A new directory's owner also owns its
`mod.rs`; explicitly listed shared roots are the exceptions. Do not edit another
owner's file to “just add an export”: send the required addition to that owner.
Ownership transfers only after the previous task's focused tests/review pass.

| Beads / task | Owned product files (create unless marked existing) |
|---|---|
| `.5` / 1 | Registry foundation in `crates/spire-core/src/content/{mod,ids,cards,items,encounters,events}.rs` and per-family `content/ids/*.rs`; existing core `lib.rs` export. Later content owners extend their own family files. |
| `.4` / 2 | `spire-core/src/run/{mod,model,profile,decision,engine,rng,public}.rs`; existing core `lib.rs` after Task 1; declaration-only `combat/command.rs` and `combat/mod.rs` export before Task 3 |
| `.6` / 3 | Existing `spire-core/src/combat/{mod,model,engine,effects,piles,turns,view,snapshot,content}.rs`; new `combat/{runtime,selection,hooks,card_state}.rs` |
| `.7` / 4 | `spire-core/src/combat/cards/{mod,ironclad,silent,defect,watcher,shared}.rs`; `combat/characters.rs`; `combat/card_state.rs` after Task 3's handoff; `content/cards.rs`, `content/ids/cards.rs` after Task 1 |
| `.8` / 5 | `spire-core/src/items/{mod,relics,potions}.rs`; `content/items.rs`, `content/ids/items.rs` after Task 1 |
| `.9` / 6 | `spire-core/src/combat/enemies/{mod,act1,act2,act3,act4}.rs`; `content/encounters.rs`, `content/ids/encounters.rs` after Task 1 |
| `.10` / 7 | `spire-core/src/run/combat.rs` |
| `.11` / 8 | `spire-core/src/run/{map,world}.rs` |
| `.12` / 9 | `spire-core/src/run/{economy,offers}.rs` |
| `.13` / 10 | `spire-core/src/run/events/{mod,neow,act1,act2,act3,shrines}.rs`; `content/events.rs`, `content/ids/events.rs` after Task 1 |
| `.14` / 11 | `spire-core/src/run/rest.rs` |
| `.15` / 12 | `spire-core/src/run/progression.rs` |
| `.16` / 13 | `crates/spire-env/src/run/{mod,actions,knowledge,schema,observation}.rs`; existing env `lib.rs` exports `run` |
| `.17` / 14 | `spire-env/src/run/reward.rs` |
| `.18` / 15 | `spire-env/src/run/{episode,replay,snapshot}.rs` |
| `.19` / 16 | `spire-env/src/run/{slot,pool,delivery}.rs`; `crates/spire-python/src/{run_env,run_config,run_numeric,run_errors}.rs`; `python/spire_lab/gymnasium/{run_single,_run_common}.py` |
| `.20` / 17 | Existing env `workers.rs` minimal generic-slot change; Task 16 pool/delivery files after handoff; `python/spire_lab/gymnasium/run_vector.py` |
| `.21` / 18 | `examples/run_gymnasium.py`, `docs/run-environments.md`, acceptance traces; README/development usage |
| `.22` / 19 | Integrated review/handoff documentation; defects return to their product owner |

For table rows abbreviated `spire-core/src` or `spire-env/src`, prefix `crates/`.
The controller alone integrates export/dispatch additions into root modules and
the shared combat/run coordinator files after their owning foundation task.
Feature owners supply exact requested calls/variants; no competing edits to shared
enums. The same rule covers test module registration. Test files are named below;
fixtures stay inside those tests except the four from-start traces in Task 18.

### Dependency waves (not dispatch authority)

1. **Registry foundation:** Task 1 first; preserve legacy IDs, establish typed
   lookup/route APIs and exact four-character base starts. This is not an upfront
   full-game inventory/source-manifest gate. Content owners extend the same tables
   with all remaining definitions alongside their real mechanics; incomplete pools
   error explicitly until then, never sample a reduced registered subset.
2. **Run foundation:** Task 2 pins the shared run state/action/error declarations.
   Task 3 then extends the combat runtime; Task 8 can run alongside it in disjoint
   map/world files. Accept shared signatures before content consumers start.
3. **Concrete mechanics:** Tasks 4, 5 and 6 run in parallel after Task 3. Task 7
   wires actual fights after Tasks 4–6. No fake fight fallback.
4. **Rooms/progression:** Tasks 9–11 after their item/card prerequisites; Task 12
   joins Tasks 6–9 and 11. Task 10 also must finish before full-run acceptance.
5. **Environment:** Task 13 after actual room/public types are wired; Task 14 after
   Task 12; Task 15 after Tasks 3, 13 and 14. Task 16 follows these and Task 7.
6. **Delivery:** Task 17 follows 16, Task 18 follows all product tasks, then 19.

These waves tighten the original coarse graph. Following plan/start approval,
the 20 missing blocking edges were added: `.4→.5`, `.6→.4/.5`, `.10→.7/.8/.9`,
`.16→.6/.12/.13/.14/.15`, `.18→.16/.17`, `.21→.7/.8/.9/.12/.14/.16/.18`.
Arrow means “depends on”; retain all existing edges. Check `bd dep cycles --json`
before dispatch. `.3` is closed; `.4`–`.22` remain blocked/unassigned until workspace
setup and actual predecessor readiness, not because plan/start approval is missing.
Every tracker command uses `BEADS_ACTOR=<allocated actor>` and
`/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab ... --json`;
verify `context`/`ping` before writes (database `spire`, project UUID
`a61e792f-9730-47c0-b262-7dac036d6bf5`). No separate worktree board.

Use at most the requested five Luna authors, reusable component reviewers up to
five, and a separate Sol arbiter. These are ceilings, not a demand to occupy every
slot. Start with one registry author, then one run-foundation author; the next
ready parallel pair is combat foundation plus maps/world. Do not resume
historical sessions or dispatch any agents while preparing/reviewing this plan.
Review at these component boundaries, not per card/test/field. A reviewer checks
behavior and interfaces, not merely passing counts. Ordinary TDD retries are fine;
repeated same-family review failures require a bounded root-cause decision.

Recommended operating level is one author during shared foundations, then two to
three authors and one independent reviewer concurrently; use four or five authors
only when disjoint ready work and review capacity justify it. Keep one owner for
shared engine/state/dispatch edits. Resume an author for fixes in its own component;
do not have it review itself or spawn nested helpers/reviewers. Give each worker a
small task brief, owned paths, concrete expected behavior and focused test commands,
not the whole conversation. Sol coordinates integration and arbitrates material
cross-component issues; the separately requested Sol reviewer remains independent.

## Shared interfaces to pin once

These are new contracts, not claims that the symbols already exist. Implement
them in their listed owner files; domain payload fields follow that owner's
mechanics. Do not proliferate generic action programs or a schema language.

| Owner | Contract |
|---|---|
| 1 `content/ids.rs` | `Character::{Ironclad,Silent,Defect,Watcher}`; typed `CardId, EnemyId, RelicId, PotionId, EncounterId, EventId` with stable canonical string/numeric descriptors |
| 1 `content/mod.rs` | `Catalog::standard() -> &'static Catalog`; typed definition/eligibility accessors; registration/support status is not mechanics proof |
| 2 `run/profile.rs` | `RunConfig { character: Character, ascension: u8 }`; `CreationProfile::experienced() -> Self`, immutable fully-unlocked/Act4-enabled defaults |
| 2 `run/model.rs` | `RunState` owns persistent resources/deck, instance issuance, keys/map/history, pending room and optional `Combat`; `RunTerminal::{HeartVictory,Death,EmptyDeck,Act3NoKeys}` |
| 2 `run/decision.rs` | `GameAction::{Move(NodeId),Combat(CombatAction),Choose(ChoiceId),Select(SelectionId),Confirm,Cancel,UsePotion{slot:u32,target:Option<TargetId>},DiscardPotion{slot:u32}}`; typed IDs refer to the current semantic choice, not array positions |
| 2 `run/public.rs` | `PublicDecision` owns typed context/player/object/candidate views and optional terminal; `PublicCandidate` contains public action features and the associated private `GameAction` |
| 2 `run/engine.rs` | `Run::new(config:RunConfig, profile:CreationProfile, rng:RunRng)->Result<Run,RunError>`; `decision(&self)->Result<PublicDecision,RunError>`; `step(&mut self, action:&GameAction)->Result<RunTransition,RunError>` |
| 2 `run/model.rs` | `RunTransition { terminal:Option<RunTerminal>, committed_events:Vec<CommittedEvent> }`; `CommittedEvent::{FloorEntered{act:u8,floor:u32},KeyAcquired(Key),Ended(RunTerminal)}`; reward events only, not a log for every effect |
| 2 `run/rng.rs` | `RunRng::new(root:[u8;32], partition:Partition, ordinal:u64)->Self`; checked semantic domain/site scopes, independent streams, no slot/pool/worker ID in gameplay derivation |
| 2 then 3 `combat/command.rs`, `combat/runtime.rs` | Task 2 declares shared `CombatAction`/target identity types to avoid a foundation compile-order cycle; Task 3 implements runtime semantics using those same types. `CombatEntry`, `CombatExit`, `CombatPublic`; `Combat::enter(entry:CombatEntry)->Result<Combat,RunError>`; `step_full(&mut self,action:&CombatAction)->Result<(),RunError>`; `public_full(&self)->Result<CombatPublic,RunError>` |
| 3 `combat/hooks.rs` | `Hook` typed enum for actual acquire/remove, room/combat/turn/card/draw/discard/exhaust/damage/death/settlement checkpoints; shared queue dispatch, no foreign callbacks |
| 3 then 4 `combat/card_state.rs` | Task 3 defines `LiveCard` with upgrade count, persistent origin and combat-local state; Task 4 implements card-specific mutation/copy rules. Permanent identity differs from transient card identity |
| 7 `run/combat.rs` | `enter(state:&mut RunState, encounter:EncounterId)->Result<(),RunError>`; `settle(state:&mut RunState)->Result<(),RunError>` |
| 8–12 domain modules | `enter(state:&mut RunState)->Result<(),RunError>` and `apply(state:&mut RunState,action:&GameAction)->Result<(),RunError>`; `world::enter` additionally consumes `NodeId` |
| 13 `run/observation.rs` | `encode(decision:&PublicDecision,knowledge:&PublicKnowledge,capacities:&RunCapacities)->Result<RunObservation,EnvError>`; owned run-only I64/I8 buffers, not widened combat-v1 tensors |
| 13 `run/schema.rs` | `RunCapacities::default()` versioned preset; `run_observation_spec(&RunCapacities)->Result<RunObservationSpec,EnvError>`; schema `spire-gym-run-observation-v1`; descriptor/fingerprint accessors mirror existing ObservationSpec without accessing its private fields |
| 14 `run/reward.rs` | `RewardLedger::apply(&mut self,events:&[CommittedEvent])->Result<i64,EnvError>`; ledger owns highest floor, acquired-key bits and terminal guard |
| 15 `run/episode.rs` | `RunJob` owns config/profile/version IDs/root/partition/ordinal; `RunSequence::prepare(&self,seed:Option<&IntegerSeed>)->Result<PreparedRun,EnvError>`; prepare/install separated |
| 15 `run/snapshot.rs` | `RunCheckpoint` owns run, knowledge, reward ledger and pending state; `restore(&RunCheckpoint)->Result<RestoredRun,EnvError>`; `RestoredRun` contains those three owned live values |
| 15 `run/replay.rs` | `RunRecord` owns job, accepted semantic actions, versions and completion/fault status; `replay(&RunRecord)->Result<RunReplayResult,EnvError>` |
| 16 `run/pool.rs` | `RunPool::new(configs:Vec<RunConfig>,capacities:RunCapacities,config:PoolConfig)->Result<RunPool,EnvError>`; `begin_operation(&self)->Result<RunPoolOperation<'_>,EnvError>` |
| 16 `run/delivery.rs` | Operation `reset_batch(&mut self,indices:&[usize],seeds:&[Option<IntegerSeed>],return_all:bool)->Result<RunResetRows,EnvError>`; `step_batch(&mut self,actions:&[i64],tokens:&[[u64;2]])->Result<RunStepBatch,EnvError>`; `mark_export_fault(&mut self,message:&str)` |

`RunError` (Task 2) distinguishes invalid input/not-ready/terminal from unsupported,
invariant, resource and RNG faults; it retains the first fault plus any committed
reward events if resolution failed. Faulted runs cannot continue/retry. Core
knowledge carries only player-entitled facts; environment `PublicKnowledge`
(Task 13) owns public reference/history mappings, never privileged state reads.
Task 3 imports shared run error/ID declarations only after Task 2's handoff; no
downstream author races an unfinished foundation's type edits.

Task 13 defines `RunTensorValues::{I64(Vec<i64>),I8(Vec<i8>)}`, `RunTensor { key:
&'static str, shape:Vec<usize>, values:RunTensorValues }`, `RunObservation { tensors:
Vec<RunTensor> }` and `RunObservationBatch { rows:usize, tensors:Vec<RunTensor> }`.
Task 16 defines `RunDecisionRows { slot_indices:Vec<usize>, observations:
RunObservationBatch, tokens:Vec<[u64;2]>, infos:Vec<RunInfo> }`, `RunResetRows {
decisions:RunDecisionRows, effective_seeds:Vec<Option<IntegerSeed>> }` and
`RunStepBatch { current:RunDecisionRows, rewards:Vec<f32>, terminated:Vec<bool>,
truncated:Vec<bool>, finals:RunDecisionRows }`. `RunInfo` carries episode ID/step
count, character/ascension, outcome, Heart-success and total reward, plus optional
HP checkpoints. Outcome codes are ongoing=0, Heart=1, death=2, empty-deck=3,
Act3-no-keys=4. Seeds/version internals stay in explicit privileged records, not
ordinary observations/info. Task 17 preserves these same rows and info masks.

The legacy `Scenario`, `Action`, `CardInstance`, public views and snapshot2 APIs
remain adapters to **one internal runtime/queue**, not a second state machine.
Their existing fields/validation/IDs stay intact; `Combat::enter` selects run
entry hooks and `Combat::new` selects supplied-ready semantics. Legacy snapshots
must not silently accept extended state. Internal migration belongs solely to
Task 3; do not force a `bool upgraded` or one-enemy RNG field onto new mechanics.

Module entry/apply calls consume and mutate the same owned `RunState`. Only the
coordinator advances automatic effects/publication. Room owners do not assign
rewards, increment floors or terminate episodes independently. Shared run dispatch
is integrated by the controller using each domain owner's tested calls.

## Task procedure and command conventions

Each task follows its checkboxes in order. For content-heavy tasks repeat the
red/green cycle for each distinct behavior family within the owned module; these
are not separate design/reviewer gates. Add a regression before fixing a concrete
defect. No product tests are run by this planning pass.

Core checks: `cargo test --locked --offline -p spire-core <filter>`.
Environment checks: `cargo test --locked --offline -p spire-env <filter>`.
Each expected result below means the selected tests pass with a **nonzero** test
count, not that a filter matched nothing. Start from the corresponding test name
in the task; the grouped filter reruns related behavior. Record the exact command
and result in that task's Beads, not in a new receipt system.

### Task 1 — Registry foundation and construction (`spire-dxo.5`)

**Files:** owned `content/` files above; test `crates/spire-core/tests/run_content.rs`.
**Interfaces:** produces extensible typed IDs/Catalog and explicit incomplete-pool
errors for subsequent core tasks; preserves legacy `combat::CONTENT_ID`, string
vocabulary and public starter definitions. Later content owners populate their own
family tables; full normally reachable coverage is still final acceptance, not a
cut justified by partial registration. Root-cause/task-boundary refinement is in
the execution ledger and first author's context response.

- [x] Write exact four-character base-start multiplicity tests (Ironclad 5/4/Bash=10; Silent 5/5/Survivor/Neutralize=12; Defect 4/4/Zap/Dualcast=10; Watcher 4/4/Eruption/Vigilance=10), named producer-route eligibility/exclusion tests, incomplete-pool rejection and `legacy_ids_unchanged`. Register Ascender's Bane identity and Iron Wave default. Basic cards are not ordinary rewards; no universal generated/event boolean. Existing facts plus the bounded public-reference start check in the execution context supply these values, without claiming original-runtime proof.
- [x] Run core filter `--test run_content`; observe missing registry APIs/assertion failures, not toolchain failures (history retained in Task1 report).
- [x] Implement the typed foundation, four exact starts, known named-route metadata and fail-closed unknown/incomplete pool construction in `content/`. Separate card type/color/rarity and per-producer eligibility. Split IDs by family for disjoint later ownership; do not manufacture a full manifest or synthetic pool. Track foundation definitions versus executable behavior and remaining family coverage in `docs/run-content.md`. Pin exports before handoff; no starter/no-op fallback.
- [x] Rerun focused/module and workspace tests; accept foundation after independent review and scoped re-review, with fresh controller174/0 workspace result. Tasks4–6/10 still must complete family definitions/mechanics; no full coverage or playable-catalog claim. Corrected NoteStorage and Result-returning starter API are the actual handoff.

### Task 2 — Run state, initialization and coordinator (`spire-dxo.4`)

**Files:** owned `run/` foundation files; tests `run/tests/{creation,decisions}.rs`.
**Interfaces:** consumes Catalog; produces Run/RunState/actions/events/errors above. Declare shared `CombatAction` and target identities in `combat/command.rs`, with a declaration-only `combat/mod.rs` export, before Task 3 consumes them; no alternate combat implementation here.

- [x] Write `all_84_starts`, `invalid_ascension_rejects`, `duplicate_instance_survives_remove`, `invalid_action_preserves_rng`, `nested_choice_returns_once`. Assert 4×21 normal profiles, unique permanent identities, no mutation on rejected actions, and parent choice resumption once. Assert experienced defaults: Neow eligible, boss history populated, Note Iron Wave/0, Act4 enabled. Pin starting-resource/ascension expected tables from qualified rules, not from the initializer under test.
- [x] Run core creation/decisions filters; red failures retained in report, including corrected literal A6/A14 and stale-selection atomicity regression.
- [x] Implement foundation creation/typed offers, ownership, incremental captured selections/continuations and automatic boundary/fault retention. Task10 still owns real Neow consequences: selected offer commits then faults at the unwired boundary, no fake success or extra player action.
- [x] Pass focused RNG1/creation3/decisions11 and controller workspace189/0; independent component review/fix1/scoped re-review resolves material findings. Legacy RNG unchanged. One Minor shared-hook warning deferred to actual consumers/final review; no playable full-run claim.

### Task 3 — Shared combat runtime (`spire-dxo.6`)

**Files:** combat foundation ownership above; tests `combat/tests/{full_runtime,selection}.rs`.
**Interfaces:** consumes shared IDs/RunError after Task 2; produces LiveCard, full-entry/action/projection and typed hooks. Maps/world can proceed separately without editing these combat files.

- [x] Implement named real multi-enemy/selection/empty/guard/legacy-entry regressions plus captured multi-pick/disposition, native first-fault, owned identity and true hook behavior cases.
- [x] Preserve observed red history, including initial wrongly targeted legacy entry and later real compatibility/phase/full-power failures; some slice2/Exit/actor-power test-first gaps explicitly recorded, not retroactively asserted.
- [x] Migrate one state/queue/OwnedCards/Piles; full actor/intent/power/per-actor RNG, action/projection/Exit/native hook and typed fault seams, old public types/snapshot2/RNG adapters intact. Actual family mechanics remain Tasks4–6.
- [x] Accept shared foundation after one independent review+2bounded fixes/scoped reviews and controller239/0+diffcheck on unchanged files; full_runtime24/selection8/combat134 author filters. Existing native-consumer warnings deferred to real integration/final review, no full-content/run/fidelity claim.

### Task 4 — Cards and four-character mechanics (`spire-dxo.7`)

**Files:** `combat/cards/`, `characters.rs`, `card_state.rs`; tests `combat/tests/full_cards.rs`.
**Interfaces:** consumes Catalog/runtime/hooks; produces `cards::resolve(&mut Combat,&CombatAction)->Result<(),RunError>` and owned character/card state used by the shared runtime.

- [ ] Write `upgrades_and_copies_do_not_alias`, `ironclad_exhaust_strength`, `silent_discard_poison`, `defect_channel_evoke_focus`, `watcher_stance_mantra`, `shared_off_color_and_curses`. Use explicit expected damage/resource/order values per distinct card/mechanic from qualified facts; pin duplicate-instance selection and multi-upgrade persistent state, not only base card labels.
- [ ] Run core filter `combat::tests::full_cards`; expect missing handlers/assertion failures.
- [ ] Implement all reachable colored/shared/colorless/status/curse/generated cards in ordinary Rust modules using shared hooks/queue; no generic “unimplemented but playable” dispatch. Maintain ordinary small fixture tables plus focused tests for exceptional trigger/payment/copy/selection rules. Off-color cards use the same runtime, not another character-specific engine.
- [ ] Complete card IDs/metadata and actual producer-pool membership in the owned registry family files as those mechanics land; incomplete pools cannot become smaller valid pools. Preserve stable IDs and four-character starts.
- [ ] Rerun the filter and `combat::tests`; expect PASS. Compare registrations with real dispatch and behavior tests in component review; row counts alone do not close the task.

### Task 5 — Relics, potions and lifecycle (`spire-dxo.8`)

**Files:** `items/`; tests `crates/spire-core/tests/run_items.rs`.
**Interfaces:** consumes typed Hook, RunState and full combat runtime; produces `items::on_hook(&mut RunState,Hook)->Result<(),RunError>` and potion target/use/discard validation through GameAction.

- [ ] Write `acquire_remove_replace_once`, `counter_survives_two_combats`, `potion_capacity_and_target`, `combat_local_resets`, `blocked_acquisition_is_not_inserted`. Assert permanent counters persist while combat-local charges reset; consumed/removal hooks run once; invalid targets consume nothing.
- [ ] Run core filter `--test run_items`; expect missing effects/assertion failures.
- [ ] Implement every reachable relic/potion, including acquisition children, replacement/removal, resource hooks and outside/combat use. Share the run/engine effect lifecycle; provide no inert fallback. Captured potency and later live modifiers must use each actual mechanic's checkpoint.
- [ ] Complete item IDs/definitions/eligibility in the owned registry family files alongside actual item behavior; missing entries are unfinished product work, not a final reduced item pool.
- [ ] Rerun `--test run_items`; expect PASS. Review resource/hook ordering, not just inventory membership.

### Task 6 — All-Act encounters (`spire-dxo.9`)

**Files:** `combat/enemies/`; tests `combat/tests/full_enemies.rs`.
**Interfaces:** `enemies::setup(EncounterId,u8,&mut RunRng)->Result<CombatEntry,RunError>` and `enemies::act(&mut Combat)->Result<(),RunError>` feed the same runtime.

- [ ] Write `every_standard_formation_constructs`, `intent_is_selected_move`, `summon_split_revive`, `boss_phases_and_heart`, `ascension_thresholds`. Assert multi-enemy death/revival and boss phases settle at their real checkpoints; hidden AI history is not public. Use table-driven below/at-threshold examples for each changed HP/damage/AI rule, not a sampled-frequency campaign.
- [ ] Run core filter `combat::tests::full_enemies`; expect FAIL for absent Acts/formations.
- [ ] Implement formations, opening/HP sampling, actual AI/histories/intents and effects for Acts1–4, including Shield/Spear and Heart. Keep canonical eligible choices and intended conditional probabilities, with independent semantic RNG sites.
- [ ] Complete encounter/enemy IDs/definitions and scheduled eligible sets in the owned registry family files alongside those behaviors; do not route only to the initially registered Cultist/Jaw Worm.
- [ ] Rerun the filter plus `combat::tests`; expect PASS. Review distinctive branches/ascension behavior; construction coverage alone is insufficient.

### Task 7 — Run/combat bridge (`spire-dxo.10`)

**Files:** `run/combat.rs`; tests `run/tests/combat_bridge.rs`.
**Interfaces:** consumes completed card/item/enemy behavior; produces run `enter`/`settle` above.

- [ ] Write `two_real_fights_preserve_resources`, `persistent_card_growth_and_copies`, `smoke_escape_is_nonterminal`, `boss_kill_waits_for_children`. Assert only permanent-origin mutations persist; HP/items/counters carry; combat-only powers clear; entry and settlement hooks occur once.
- [ ] Run core filter `run::tests::combat_bridge`; expect FAIL on absent bridge.
- [ ] Move ownership into the same Combat runtime and reconcile actual CombatExit into RunState once. Preserve HP checkpoints, permanent card identities, potion/relic state and pending lethal/escape consequences. Reward generation belongs to economy; gym reward belongs to the ledger.
- [ ] Rerun filter; expect PASS. Review real entry draws/hooks and terminal ordering; no fixed-damage battle stub is admissible.

### Task 8 — Maps, routing and encounters (`spire-dxo.11`)

**Files:** `run/{map,world}.rs`; tests `run/tests/world.rs`.
**Interfaces:** consumes registry/RunState; `map::generate(act:u8,rng:&mut RunRng)->Result<ActMap,RunError>`; `world::enter(&mut RunState,NodeId)` routes to actual room owners.

- [x] Write named map/move/hidden-encounter/floor-once regressions plus scoped RNG, real next-row Winged, persistent marker eligibility/no-draw, external API and sanitized projection/formatting cases.
- [x] Observe absent module/compile and actual behavior reds (history retained in Task8 report); initial0selected tests not counted as green.
- [x] Implement native representative maps/scoped drafts/Acts and movement/question/scheduling/history foundation, with retained faults at unwired actual room/constructor boundaries. No fixed win route. Actual room effects and Act4 route remain later owners.
- [x] Accept after one component review+3scoped fixes/reviews and controller external2/run30/workspace223 on unchanged world files. No exact original map/seed/runtime claim; shared native-consumer warnings deferred to real integration/final review.

### Task 9 — Loot, shops and chests (`spire-dxo.12`)

**Files:** `run/{economy,offers}.rs`; tests `run/tests/economy.rs`.
**Interfaces:** consumes card/item/registry behavior and RunState; exports room enter/apply, captured offers and actual acquisition consequences.

- [ ] Write `offer_is_not_acquisition`, `claim_cannot_repeat`, `shop_insufficient_gold_atomic`, `purchase_remove_restock`, `sapphire_competes_with_relic`. Assert chosen object identity, exact prices/eligibility/pity/pool updates, skipped offers and blocked insertion checkpoints; a full potion belt does not silently erase an offer.
- [ ] Run core filter `run::tests::economy`; expect absent economy failures.
- [ ] Implement actual combat/boss rewards, gold, offer skip/claim, merchant stock/pricing/removal/restock and treasure transactions. Run item children before room completion. Keep offer-time generation/history changes distinct from obtain/insertion; no preferred-pick policy.
- [ ] Rerun filter; expect PASS. Review exactly-once transaction/child semantics and real Sapphire tradeoff.

### Task 10 — Neow and events (`spire-dxo.13`)

**Files:** `run/events/`; tests `run/tests/events.rs`.
**Interfaces:** consumes generated Neow offers from Task 2 plus shared choices/cards/items; implements their consequences and all reachable ordinary/shrine/one-time events.

- [ ] Write `neow_drawback_then_reward`, `duplicate_transform_captures_instance`, `empty_eligible_branch`, `event_combat_returns_to_parent`, `note_is_episode_local`, `match_faces_remain_private`, `portal_time_gate_assumed_met`, `portal_other_eligibility_unchanged`. Assert real costs before rewards, owned selection targets, legal zero-candidate behavior, resume once and no cross-reset profile contamination. Portal is time-eligible in Act 3 without waiting, remains excluded outside its normal Act/pool/history conditions, and is not forced as the event outcome.
- [ ] Run core filter `run::tests::events`; expect absent event effects.
- [ ] Implement full reachable Acts1–3/event Neow behavior, deck/remove/upgrade/transform children, battles and persistent outcomes through the common coordinator. Use ordinary typed event state. Apply TIME-001 at Portal eligibility: omit only the elapsed-time restriction, not Act/pool/history rules or normal random selection. No clock or wait action; accepting/declining follows the real event/boss transition.
- [ ] Complete event IDs/definitions/actual eligibility and history in the owned registry family files; no preapproved whole-game manifest is a new upstream audit gate, and no missing event may silently disappear from the completed PoC.
- [ ] Rerun filter; expect PASS. Review eligibility/branch effects and public knowledge; text/menu-only shells do not count.

### Task 11 — Rest sites (`spire-dxo.14`)

**Files:** `run/rest.rs`; tests `run/tests/rest.rs`.
**Interfaces:** consumes RunState/card hooks; exports room enter/apply for legal rest/upgrade/relic actions and Ruby recall.

- [ ] Write `rest_actions_follow_eligibility`, `empty_upgrade_set`, `blocked_heal`, `ruby_once`, `relic_rest_action_children`. Assert only legal actions appear and execute their actual HP/card/resource effects; no action is selected automatically.
- [ ] Run core filter `run::tests::rest`; expect missing rest behavior.
- [ ] Implement resting, smithing, recall and all reachable relic-enabled actions/children with exact eligibility and heal/upgrade hooks. Add an actual KeyAcquired event only on first successful recall, not on opening the menu.
- [ ] Rerun filter; expect PASS. Review completion versus nested reward/selection resume.

### Task 12 — Keys, Acts, A20 and terminals (`spire-dxo.15`)

**Files:** `run/progression.rs`; tests `run/tests/progression.rs`.
**Interfaces:** consumes world/bridge/economy/rest/enemies; produces all real terminal classes and committed floor/key/end events.

- [ ] Write `ruby_emerald_sapphire_real_routes`, `a19_one_a20_two_act3_bosses`, `no_keys_is_non_goal`, `act4_order_and_heart`, `settled_empty_deck_loss`. Assert real recall/marked-Elite/chest acquisition, all keys checked at the proper checkpoint, A20 requires the second distinct boss, Act4 rooms/fights precede Heart, and no-key completion is neither victory nor death.
- [ ] Run core filter `run::tests::progression`; expect missing Act4/terminal branches.
- [ ] Implement Act initialization/transitions and boss/reward sequencing, marked Elite effects and key settlement, complete Act4 routing and terminal precedence. Empty permanent deck is tested only at its settled gameplay checkpoint, not an intermediate transform/removal.
- [ ] Rerun filter plus `run::tests`; expect PASS. Late-Act fixtures diagnose boundaries only; Task 18 still needs from-start runs.

### Task 13 — Public encoding, candidates and spaces (`spire-dxo.16`)

**Files:** env run encoding ownership above; tests `spire-env/src/run/tests/encoding.rs`.
**Interfaces:** consumes real PublicDecision types; produces numeric schema/capacities, PublicKnowledge, `RunMenu::resolve(i64)->Result<&GameAction,EnvError>` and RunObservation.

- [ ] Write `equal_public_states_equal_encoding`, `all_decision_kinds_actionable`, `padding_is_illegal`, `tiny_capacity_no_clipping`, `retained_buffers_owned`. Change hidden draws/AI/future offers and assert **all** public tensors/menu order/references agree; valid candidates map exactly once to legal core actions, terminals have no actions and nonterminals have some.
- [ ] Run env filter `run::tests::encoding`; expect absent full-run encoder/schema.
- [ ] Implement public-only typed projection encoding, local public references and known/occupied masks. Publish the literal column tables/vocabularies and one `RunCapacities` preset in native `schema.rs` before bindings use it. Use checked shapes/allocation and I64/I8 owned run tensors; native descriptor drives Python spaces. Choose/test finite defaults against the release workloads, document overrides; never size from privileged live counts.
- [ ] Rerun filter; expect PASS. Review hidden lengths/order/references as well as values. This is one encoder contract, not a schema compiler or another design approval loop.

### Task 14 — Selected reward ledger (`spire-dxo.17`)

**Files:** `run/reward.rs`; tests `spire-env/src/run/tests/reward.rs`.
**Interfaces:** consumes only committed events; produces RewardLedger and exact integer delta, plus Heart-success separate from return.

- [ ] Write `floor_key_terminal_values`, `compound_and_duplicate_events`, `no_key_no_penalty`, `fault_not_death`. Assert Acts1–3 new floor `1`, Act4 new floor `10` **instead**, first key `5`, Heart `100`, death/settled-empty-deck `-1`, non-goal `0`, repeats/read/reset `0`. For each real producer, assert the sum of its committed events (e.g. floor+key `6` in Acts1–3 if both commit in that step); do not invent Act4 key acquisition to make an artificial compound Heart reward.
- [ ] Run env filter `run::tests::reward`; expect missing ledger.
- [ ] Implement highest-floor/key/terminal guards and exact-once deltas, with checked bookkeeping. If execution faults after a committed event, preserve diagnostic bookkeeping but return no successful step or fabricated gameplay loss; reset abandonment earns nothing.
- [ ] Rerun filter; expect PASS. Review emitted event checkpoints against Task 12, not only synthetic ledger inputs.

### Task 15 — Run jobs, replay and snapshots (`spire-dxo.18`)

**Files:** `run/{episode,replay,snapshot}.rs`; tests `spire-env/src/run/tests/replay.rs`.
**Interfaces:** consumes Run, knowledge/ledger and existing IntegerSeed/expand_seeds; produces prepared jobs, RunRecord, RunCheckpoint and replay/restore above.

- [ ] Write `seed_more_than_256_bits`, `seed_restart_none_continue`, `snapshot_choice_and_act_boundary`, `semantic_replay_from_start`, `incompatible_record_rejects`, `fault_prefix_not_completed`. Assert explicit reseed ordinal `0`, None increments once, identical semantic actions reproduce decisions/rewards, and restored pending selectors/knowledge/ledger agree without reapplying rewards.
- [ ] Run env filter `run::tests::replay`; expect missing job/checkpoint/replay APIs.
- [ ] Stage immutable jobs/profile/version/root/ordinal before installation. Implement owned checkpoint and from-start semantic replay; record attempted fault actions separately from successful accepted actions. No ordinary-step snapshots/JSON/I/O, no universal codec or external saves. Reset replaces local Note/history with the configured creation profile.
- [ ] Rerun filter; expect PASS. Review incompatible identities, fault provenance, memory/drain ownership and profile isolation.

### Task 16 — Serial native pool, bindings and RunEnv (`spire-dxo.19`)

**Files:** table ownership; existing binding `lib.rs`/Gym `__init__.py` controller exports; tests `tests/python/run/{test_native,test_single,test_delivery}.py` and `spire-env/src/run/tests/lifecycle.rs`.
**Interfaces:** consumes all environment contracts; exposes `NativeRunPool(configs,*,workers=1,capacities=None,partition="train",autoreset="manual",record_replay=False)` and keyword-required `RunEnv(*,character,ascension,capacities=None,record_replay=False)`.

- [ ] Write `test_manual_terminal_requires_reset`, `test_invalid_actions_no_mutation`, `test_public_spaces_native_descriptor`, `test_close_and_retained_arrays`, `test_packaging_failure_faults_original_call`. Assert scalar input rejects bool/float, no reset options can inject deck/gold/keys, int64 features/int8 masks, no mutation/aliasing, and the delivery lease covers Python packaging/reentrancy.
- [ ] Run env filter `run::tests::lifecycle`, then `.venv/bin/python -B -m pytest tests/python/run/test_native.py tests/python/run/test_single.py tests/python/run/test_delivery.py -q`; expect missing run API/fixture failures.
- [ ] Implement B=1 serial slots/pool and original-operation delivery lease; use EnvError internally and dedicated `RunInputError/RunStateError/RunSetupError/RunFaultError` bindings. Detach native work; export owned I64/I8 buffers with run-only converters. Keep diagnostic/replay privileged, completed drains retained on failed export. Reuse `spaces_from_spec`, not combat-specific info/reset helpers. Same mask-aware Gymnasium checks as combat; no arbitrary unmasked action acceptance.
- [ ] Build/check only with existing locked dependencies under explicit extension-install authority (see verification below), then rerun tests; expect PASS and old combat Python tests unchanged. Runtime native/export faults poison further mutable operations, including reset; no retry/rollback. No general pool abstraction rewrite.

### Task 17 — Native batching and RunVectorEnv (`spire-dxo.20`)

**Files:** pool/delivery handoff, existing workers minimal generic slot operation, `run_vector.py`; tests `spire-env/src/run/tests/batching.rs`, `tests/python/run/{test_vector,test_compatibility}.py`.
**Interfaces:** extends RunPool to B profiles and existing persistent Scheduler; exposes `RunVectorEnv(*,configs,workers=1,capacities=None,record_replay=False)`, standard batched spaces and SAME_STEP metadata.

- [ ] Write `serial_worker_agreement`, `invalid_last_row_atomic`, `partial_reset_isolation`, `mixed_terminals_final_resources`, `export_failure_preserves_older_records`; Python tests assert `final_obs/final_info` masks, exact real terminal-step event sum, old-run final versus replacement resources, and no alias after close. Include mixed character/A0/A20 configs, huge scalar seed expansion by **actual** slot, malformed unselected reset entries and strided/read-only int64 action arrays.
- [ ] Run env filter `run::tests::batching` and `.venv/bin/python -B -m pytest tests/python/run/test_vector.py tests/python/run/test_compatibility.py -q`; expect B>1/same-step gaps.
- [ ] Generalize only Scheduler's slot element type, reuse native workers, validate the complete batch before mutation, stage replacements and publish slot order after quiescence. Keep atomic operation lease through final packaging. Partial reset updates only selected rows; finals belong to old runs, main rows to fresh configured profiles. On runtime peer fault, quiesce all and retain first fault records; no synthetic endings.
- [ ] Rebuild under the same authority and rerun both plus `tests/python/env`; expect PASS. Check native-vs-Python, single-vs-serial-vs-workers per-run equality and pinned Gymnasium wrapper subset, not universal learner/wrapper support.

### Task 18 — Actual from-start Heart runs and example (`spire-dxo.21`)

**Files:** example/docs above; `crates/spire-env/tests/full_run_acceptance.rs`; `tests/python/run/test_integration.py`; `tests/fixtures/run/heart_{ironclad,silent,defect,watcher}.json` (four small actual semantic action traces, not generated goldens).
**Interfaces:** consumes all product components; public callers use reset and current masks only. Normal reset cannot import the privileged diagnostic fixture state.

- [ ] Write `all_characters_from_start_to_heart` to initialize normally and replay actual legal actions; assert all three real keys, Acts1–4 actual fights, HeartVictory and matching exact reward event sum. Write `loss_non_goal_fault_are_distinct` and Python single/vector replay agreement. Initially fail for absent/unexecuted paths; never mark an invented trace successful.
- [ ] Run `cargo test --locked --offline -p spire-env --test full_run_acceptance` and `.venv/bin/python -B -m pytest tests/python/run/test_integration.py -q`; expect failures until real traces execute.
- [ ] Obtain four reproducible legal trajectories using the finished caller API and a small human/scripted action sequence, not a learner/search platform. Store the actual seed/config/actions without resources/keys/map injection. Include A20 integrated two-boss coverage; A0 traces per character are acceptable alongside the separate A20 tests, not a scope reduction. Fix concrete mechanics bugs under owning Beads with regressions. Deliver finite masked random CLI `examples/run_gymnasium.py --character ironclad --ascension 20 --seed 7 --num-envs 1 --policy random`; no winning seed promise for random mode.
- [ ] Rerun integration plus full suites below; expect supported behavior passes, accurately disclose any known readiness failures. Document real command/results, capacity defaults/overrides, replay/privacy/fault limits and fidelity grade. One small A20 release runtime/memory smoke is sufficient; no speed/strength claim without measurement.

### Task 19 — One integrated review and handoff (`spire-dxo.22`)

**Files:** `docs/work-status.md` dated outcome and `docs/run-environments.md`; defects go to their existing owner files/tasks.
**Interfaces:** consumes verified implementation, four legal Heart traces and component review outcomes; produces a qualified release/handoff decision, not merge/publication authority.

- [ ] Review the actual product against spec §§1–9 and the five Review Focus cases. Check every reachable handler/ascension branch is wired; missing content remains unfinished, never waived because random examples avoided it. Check no private material or unsupported fidelity claim entered public changes.
- [ ] Independently rerun the integrated commands below against the candidate extension/source; confirm tests did not use a stale extension. Classify known readiness failures separately from new failures. Reproduce any blocker with a focused test, then return it to its product owner; do not launch a verifier rewrite.
- [ ] Record one integrated reviewer disposition, executed from-start/single/vector/replay outcomes and remaining limits in Beads/docs. Heart paths and honest faults are required; counts/hashes/near-Heart fixtures alone cannot establish acceptance. No commits, merge, push, cleanup or publication without separate authority.

## Integrated verification and handoff

During execution, run a pre-change baseline once and the same scoped/full commands
at the integrated candidate. Historical counts are not current reruns. Use existing
toolchains/cache; if something is missing, stop rather than downloading/installing.

```sh
cargo test --workspace --locked --offline
cargo test --workspace --release --locked --offline
.venv/bin/python -B -m pytest tests/python/env tests/python/run -q
.venv/bin/python -B -m pytest tests/python -q
.venv/bin/python -B examples/run_gymnasium.py --character ironclad --ascension 20 --seed 7 --num-envs 1 --policy random
```

Rust checks protect shared core/env regressions and release arithmetic; env/run
Python checks protect the actual compiled binding/Gym contracts; full pytest also
detects collateral breakage. The random example protects external caller usability,
**not Heart success**. Four deterministic acceptance traces establish that path.
If `spire-vhh` persists, name its two readiness failures rather than claim green;
do not run CUDA checks as a gym prerequisite.

Binding tests require a freshly compiled candidate extension. The existing command
`uv run --no-sync maturin develop --release --locked --offline` installs the local
extension into `.venv`; obtain explicit permission for that local package refresh
before running it. No `uv sync`, dependency installs or lockfile changes. Record
source/candidate build identity and verify the imported module path before Python
verification; absence of rebuild authority is a reported verification blocker,
not permission to validate new Rust code against the old binary.

Before starting execution, use `using-git-worktrees` to inspect an isolated-workspace
option and preserve current uncommitted docs/private work. Do not create/remove
worktrees or move sessions as part of this planning pass. The executor must carry
the approved uncommitted spec/plan into its chosen workspace explicitly, without
resetting, committing or overwriting unfamiliar work.

User plan/start approval is now recorded in closed `.3`; the declared dependency
edges are tightened. After execution-workspace consent/setup and baseline, release
only the first ready registry/foundation work. No agents, product code, runtime
tests, installs or commits have been started by this administrative startup pass.

Self-review covered all nine spec sections, one-to-one component mapping, shared
type/ownership consistency and the five Review Focus cases. Corrected the
private-field ObservationSpec reuse, downstream LiveCard declaration and a
non-reachable compound-reward example. Dependency additions are acyclic and were
applied after plan/start approval. The unsupported global clock rationale is
withdrawn; user-approved TIME-001 resolves Portal handling without a time subsystem.
Plan/start/worktree consent and the Rust baseline are satisfied. Component execution
and focused independent review remain; no product completion claim is implied.
