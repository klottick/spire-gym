# M21 Watcher Combat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans after actual user approval and root release. Implement with one senior owner, meaningful TDD and one fresh independent final milestone review; no per-task review swarm. Do not execute this planning proposal now.

**Goal:** Deliver finite native Watcher A0 authored combat with correct stance, Mantra, Scry, retain, generated-card and representative item/power timing.

**Architecture:** Extend the existing shared Rust character, resolver, piles, selection and Run owners. Add one explicit native standalone profile to the current catalog/session and versioned encoder/replay owners. Preserve accepted predecessors and distinguish supplied diagnostics from canonical-resource entry.

**Tech Stack:** Existing locked Rust workspace, native int64 observation transport, existing serde/JSON/fingerprints and project RNG; no new dependencies or Python build.

**Spec:** [M21 design](../specs/2026-10-27-m21-watcher-combat-design.md). Read both documents; exact finite roster and paired payloads are in the design.

## Global Constraints

- Planning only until independently reviewed written package, actual primary-session user approval and root release; roadmap delivery is after accepted M20.
- One authoritative Rust engine. No Watcher engine, copied damage/cost/draw/resolver, encounter inventory, currency journal, scheduler or Python facade loop.
- Exact finite admission: 28 permanent kinds, generated-only Miracle/Smite/Safety; all base/once-upgraded. Seven relics and seven potions exactly as listed in the design.
- Canonical-resource authored entry bypasses Neow/map/Act1; supplied resources are diagnostic. M22 owns normal progression and M25 ordinary character completeness.
- Preserve combat-v1, default/old43 standalone, opt-in57 encounter v2, captured Run pools and accepted M9–20 profile identities/results.
- Source-grounded, not original-runtime validated. Owned archive cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673 has unverified upstream build/channel. No game execution/private publication.
- Invalid input commits no state/RNG; accepted effects retain their exact committed prefix and first fault. Faults/budgets never become Death, truncation, retries, silent clamps or dropped content.
- Freeze public-data caller, seeds, resources and scenarios before outcomes. No tuning/search afterward without actual user authorization; retain genuine losses.
- No new dependency/install/commit/push/PR/cleanup/archive authority. Stage nothing in this planning lane.

## Review Focus

1. Same-stance requests, Calm→Divinity and threshold Mantra while already Divine: exactly one real transition and the correct energy/history, without duplicate callbacks (Task2).
2. Empty/undersized/full-hand Scry and Meditate choices, reversed pick order and power-owned suspension: exact revealed/selected order and one resumption, no hidden draw leak (Task3).
3. Retained growth/upgrade and differing combat/turn prices across cleanup: mutate only the exact instance; Ethereal wins its qualified exhaustion; no master-deck mutation or invented caps (Task4).
4. Last-enemy/half-dead/player-death cancellation and child-insertion/selection faults: preserve accepted custody and prefix; never force-drain pending work or fabricate a terminal (Tasks3–5).
5. Potion-owned choice, stale tokens and publication-capacity/replay mismatch: consume once, retain source owner, reject incompatible schemas, and keep old descriptors/actions unchanged (Tasks5–7).

---

## File and interface map

All core paths below begin `crates/spire-core/src/`; environment paths begin
`crates/spire-env/src/`. Existing files remain their owners. Register new modules
in existing `mod.rs`/`lib.rs` files only as required by their task.

| Files | Responsibility |
| --- | --- |
| `content/{ids/cards,ids/items,cards,items}.rs`, new `content/watcher.rs` | IDs/metadata and finite admission; no mechanics dispatch |
| `combat/characters.rs`, `combat/{rules,effects,engine}.rs` | Shared stance transitions, damage and validation |
| New `combat/cards/watcher.rs`, optional focused `watcher/{scry,retain,powers}.rs` | Local card payloads and typed continuations, using the shared queue |
| `combat/{card_state,model,selection,piles,turns,hooks,powers,view}.rs` | Authoritative instance state, ordered modal custody, turn callbacks, public projections |
| `items/{mod,relics,potions}.rs`, `run/{training,combat}.rs` | Real inventory callbacks and validated owned entry/settlement |
| `combat_policy.rs`, `run_observation.rs`, new `run_schema_m21.rs` | Opt-in descriptor/encoding over `encode_run_with_schema` |
| `combat_training.rs`, `run_replay.rs`, optional focused `combat_replay.rs` | Existing native session/catalog branch and semantic replay adapter |
| New core `combat/tests/m21_watcher_{stances,scry,retain,creation}.rs`, `run/tests/m21_watcher.rs` | Behavior, faults, complete custody and entry tests |
| New `crates/spire-env/tests/m21_watcher_combat.rs` | Numeric/session/replay and compatibility tests |
| New `crates/spire-core/examples/m21_watcher_combat.rs` and focused support module | Frozen native caller/demonstrations; no policy search |

Names below are exact proposed M21 interfaces, not assertions about unaccepted
M15–20 source. On accepted-source reconciliation, reuse equivalent upstream
fields/helpers; record any necessary naming adjustment in this pair before code.

### Task 1: Bind accepted dependencies and finite source checkpoints

**Files:** Retained M21 evidence; design source/qualification sections. No production edits before release.

**Interfaces:** Consumes actual accepted M15–20 revisions/exports, the accepted
d1fa706 shared source, approved M16/M18 documents and retained source facts.
Produces a short seam-to-owner reconciliation with exact source hashes and a
finite source checkpoint disposition; it does not create a verification framework.

- [ ] Confirm task checkout/Bead ownership and actual user approval/root release. Bind accepted predecessor source/export manifests and revision-qualified preservation commands/results. Do not read a changing M15 tree as acceptance or transfer M14's supplied-start amendment.
- [ ] Reconcile accepted mutable combat/turn cost, state-aware copy, actual draw versus movement, ordered pending-selection cursor, Power parent/resume phase and all-zone settlement owners. Preserve accepted M16 CharacterState/orb payloads and state projections. Use existing ownership journals; this roster requires none of its own.
- [ ] Consume the design's retained static qualifications (report SHA150d6dc1420ce5ed05bbc39cf0e5ca8e8142ea1f6ca6ce52c644e7ade3bbed20); do not rediscover selected-list order, Meditate flags, guarded Weave return, local retain writers, fixed-creation constructors or potion laws. Rule only remaining Scry continuation-count/finite semantic-collapse, later EnergyManager.recharge caller, callEndOfTurnActions↔AbstractRoom.endTurn/queued Ethereal-retain-Establishment relationship, and deferred MasterReality apply/construction schedule. Bind the concrete PureWater pre-draw/legacy mismatch to accepted upstream startup ownership. Only missing named members may be inspected from the pinned archive, never an all-content extraction or original execution.
- [ ] Write independent public rule paraphrases and concrete oracles in this design before the affected tests. Distinguish direct callback order, queued action insertion and later delivery. An unclosed named caller blocks its affected producer, not unrelated work; report a material scope conflict to root without inventing a rule or silently cutting a roster row.
- [ ] Retain input/source/fact SHA/mode manifests outside Paseo. Confirm all expected new IDs and test module paths against accepted source. Task1 is complete when every prescribed affected outcome has named evidence or an explicit blocking disposition; blockers are not passed gates.

### Task 2: Finite cards, shared stances and Mantra

**Files:** `content/*` above; `combat/{characters,rules,effects,engine,model,powers}.rs`, `combat/cards/{mod,watcher}.rs`; new `combat/tests/m21_watcher_stances.rs`.

**Interfaces:** Consumes shared `State`, `Effect`, `LiveCard`, `Stance`,
`CharacterEffect`, `FullPowerEntry` and accepted cost/payment. Produces admission
functions from the design, appended Divinity, `CharacterState.mantra_gained: i32`,
and `WatcherEffect` dispatched by `apply(state: &mut State, effect: &Effect,
action: &WatcherEffect) -> Result<(), EffectError>`. Its initial variant is
`GainMantra { amount: i32 }`. `CharacterEffect::Stance` remains authoritative.
The stance-aware NORMAL calculation signature is fixed in the design; old bool
entry points delegate to the same internal calculation with their prior meanings.

- [ ] Write `m21_watcher_admission_and_pairs` to assert the exact28/3 IDs, every base/+ metadata/cost/target/flag/payload pair from the design, generated-only permanent rejection and no ordinary-pool expansion. Use actual constructors; missing enum compilation alone is not behavioral RED.
- [ ] Write `m21_watcher_stance_requests_are_not_transitions`: same Calm grants0/no MentalFortress/Rushdown/Flurry; Calm→Neutral grants2; Calm→Divinity grants5; same Divinity grants0. Verify source-qualified notification/queue order, blocked transition and next-turn queued Neutral. Eruption's first Neutral9-damage hit precedes Wrath; incoming NORMAL doubles only in Wrath. Test Divinity outgoing base7 with Weak/Vulnerable gives23 after one final binary32 floor; HP_LOSS/THORNS/orb are unchanged.
- [ ] Write `m21_watcher_mantra_history_is_not_remainder`: existing9+3 leaves2 and history increases3; synthetic9+23 leaves22 with one transition; first installation follows its distinct path; repeated threshold while Divine adds no energy. Stacked Devotion≥10 with no Mantra creates neither history nor excess, while the existing-Mantra branch does. Brilliance uses history12/16+gained, and turn boundaries do not erase combat history.
- [ ] Add an accepted-prefix fault after the first transition-energy request and before later callbacks: exact stance/history/queued sibling work remains retained, no second exit grant on observation/retry. Pin source int32 writer behavior and existing administrative bounds separately.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m21_watcher_stance` and `cargo test -p spire-core --lib --locked --offline m21_watcher_mantra` to behavioral RED, implement the shared extension, rerun GREEN. Run existing `shared_mechanics`, `rules_ai` and accepted M16 orb filters once after the shared damage/character change.

### Task 3: Ordered Scry, return cards and modal continuations

**Files:** `combat/cards/watcher.rs` and optional `watcher/scry.rs`; shared `combat/{model,selection,engine,effects,piles,hooks,view}.rs`; new `combat/tests/m21_watcher_scry.rs`.

**Interfaces:** Adds `WatcherEffect::Scry { amount: i32 }` and
`WatcherEffect::ReturnDiscard { card: CardHandle }`; `SelectionOperation::Scry`
and `MeditateReturn` extend the single pending selection. Reuse the accepted
`SelectionParent::Power { turn, id: FullPowerId }` with exact Foresight entitlement
and retained resume phase; Card and Potion parents remain typed. No second pile,
selected vector, action loop or fake generated instance represents revealed cards.
Add `CombatAction::DeselectCard { instance: u64 }` for only those live uncommitted
M21 operations; remove the selected handle without reordering survivors.

- [ ] Write `m21_watcher_scry_reveals_only_entitled_top_order`: empty draw still requests Nirvana when enemies live; basically-dead group aborts before it; ordinary Scry never reshuffles; reveal3 from draw length2 offers exactly the top2; none/all/reversed picks preserve kept order and qualified discard order. Nirvana3 stays3 with Dexterity2 while ThirdEye7 becomes9; MentalFortress/LikeWater likewise use direct installed amounts. Later draw shifts known kept order; reshuffle invalidates it. Invalid hidden/noncandidate/duplicate/cross-parent picks preserve state/RNG byte-for-byte.
- [ ] Write `m21_watcher_scry_suspends_the_parent_once`: CutThroughFate commits its hit/payment once, waits, then draws after selection; JustLucky's later Block/hit do not run before the choice. Select A,B, deselect A, reselect A yields selected order B,A without changing reveal/cursor order. Foresight with empty draw qualifies its separate shuffle and later-turn pre-draw Power parent; an invalid ToolsOfTheTrade/Foresight phase swap rejects. Characterize the distinct initial-combat callback placement instead of reusing the later-turn schedule. GoldenEye's construction-time+2 is not reapplied at resume.
- [ ] Write `m21_watcher_returns_use_exact_discard_membership`: Weave newly Scry-discarded and already in discard, repeated qualified notification, vanished membership, hand9/10, Flurry same-stance no-op and real transition. Assert exact retained source callback/return order and at-most-one ownership, rather than assume UI update count is a gameplay trigger count.
- [ ] Add Review Focus4 cases: CutThroughFate/JustLucky lethal first hit and half-dead enemy; pending return removed by source-qualified cancellation; a fault after selected child1 moves retains child2/cursor without re-moving child1. No fabricated full drain or Victory at half-death.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m21_watcher_scry` and `cargo test -p spire-core --lib --locked --offline m21_watcher_returns` to real RED, implement typed work on the shared queue, rerun GREEN plus existing `selection`, `piles`, M15 choice and accepted death/cancellation filters.

### Task 4: Retention, growth, Meditate and turn-cost mutation

**Files:** `combat/cards/watcher.rs` and optional `watcher/retain.rs`; shared `combat/{card_state,model,selection,piles,turns,hooks,engine}.rs`; new `combat/tests/m21_watcher_retain.rs`.

**Interfaces:** Reuses accepted mutable base Block/combat-cost/turn-cost fields;
adds `self_retain: bool` / `retain: bool` only where absent. Adds
`WatcherEffect::Meditate { count: usize }`, `RetainRestore` and
`EstablishmentDiscount { amount: i32 }`. Shared helper
`modify_cost_for_combat(card: &mut LiveCard, delta: i32) -> Result<(), InputError>`
is a distinct writer in `card_state.rs`, not an alias for updateCost/setCostForTurn.
M11's chosen identities join this one retention owner; its public EndTurn contract
and Ethereal exhaustion remain unchanged.

- [ ] Write `m21_watcher_retain_growth_and_upgrade_are_instance_local`: two actual base retentions then upgrade yield Windmill18/future+5 and Perseverance11/future+3. A same-kind sibling, permanent deck and fresh generated card do not inherit growth. A qualified stat-equivalent copy preserves captured state and gets a fresh identity. New combat resets only combat-local mutation.
- [ ] Write `m21_watcher_cost_writers_keep_distinct_prices`: Sands' own−1 and Establishment's independent discount both apply at their qualified checkpoints. With combat4/turn2 and delta−1, modifyCostForCombat yields combat1/turn1; with combat4/turn0 it yields combat3/turn0. Nonnegative values floor0; X/unplayable sentinels remain distinct; temporary turn reset uses mutated combat cost. No blanket power/card cap or free-play substitution.
- [ ] Write `m21_watcher_meditate_auto_and_selected_overflow_differ`: discard size≤count flags all before capacity checks; larger discard uses selected order and flags only moved cards. At hand9 with two selected, reversing picks changes which card moves; the overflow stays in discard. Empty discard still reaches Calm/end-turn; payment is not repeated. Use the qualified selected-list law from Task1.
- [ ] Write `m21_watcher_turn_phases_do_not_infer_retention_from_hand_presence`: inherent versus temporary Retain, Ethereal+Retain, actual callback once, Establishment flag scan, LikeWater's then-current Calm, Divinity exit and Devotion/Foresight relative order. Preserve existing WellLaidPlans/Equilibrium/Pyramid behavior wherever predecessors accept it; do not widen profile admission to those producers.
- [ ] Inject failure after one growth/return/discount commit and assert exact cursor/flags/costs, unaffected master resources and first-fault retention. Run `cargo test -p spire-core --lib --locked --offline m21_watcher_retain`, then the same command with filters `m21_watcher_cost` and `m21_watcher_meditate`, to RED→minimal implementation→GREEN, plus M11 retain and accepted M15/M16 mutation regressions.

### Task 5: Fixed creation, powers and actual items/potions

**Files:** `combat/cards/watcher.rs` and optional `watcher/powers.rs`; shared `combat/{card_state,piles,powers,hooks,model,command,view,engine}.rs`, `items/{mod,relics,potions}.rs`, content metadata/IDs; new `combat/tests/m21_watcher_creation.rs` and relevant existing item tests.

**Interfaces:** Reuses `CreateCard`, generated constructors, fresh identity and
hand insertion. Adds `CombatAction::ChooseStance { stance: Stance }`,
`StanceChoiceView { options: Vec<Stance> }` and one potion-owned pending
continuation/resume phase; at most one modal owner is active. Card offers remain
card offers. Power callbacks construct shared effects; no separate scheduler.

- [ ] Write `m21_watcher_fixed_children_keep_route_and_identity`: paired Carve/Deceive create base Smite/Safety regardless of parent upgrade; BattleHymn creates stacked count at its qualified start hook; generated Miracle/Smite/Safety have actual Retain+Exhaust defaults. Hand9 with count3 commits1 to hand/2 to discard, each unique; no permanent gains. MasterReality upgrades only qualified routes and never upgrades an existing card retroactively.
- [ ] Write `m21_watcher_items_and_stored_potions_are_real_owners`: PureWater's child once through the M21 pre-draw phase; Teardrop opening Calm through the transition owner; VioletLotus queues separate1 before old-Calm exit2; GoldenEye Scry+2; StancePotion options exactly Wrath,Calm with potency0 regardless of Bark; Ambrosia metadata2/4 but exactly one Divinity request; BottledMiracle2/4 base children. SacredBark reinitializes held potion potency. Finite item admission, catalog metadata and real use match.
- [ ] Write `m21_watcher_potion_choice_commits_once`: consume slot at the qualified use boundary; stale/invalid/cross-parent stance rejected; accepted choice resumes once without another consumption. No cancel/skip is offered; reject Neutral/Divinity when not offered. Capture a fault after consumption and another after child1 insertion: retain slot, potion source, child identities, remaining cursor and first error.
- [ ] Add last-enemy/player-death/canceled-power callbacks from Review Focus4: BattleHymn's basically-dead guard, canceled pending creation and surviving current effects follow actual shared cancellation, not guaranteed whole-queue drain. Full exit census has every original once and generated children once in their real zones.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m21_watcher_creation`, then the same command with filters `m21_watcher_items` and `m21_watcher_potion`, to RED→minimal extension→GREEN; rerun affected existing item, creation, copy and settlement tests. Never refresh Python for these native changes.

### Task 6: Owned authored entry and versioned numeric/session delivery

**Files:** `run/{training,combat,mod}.rs`, shared `combat/{engine,model,hooks}.rs` and `items/{mod,relics}.rs` for staged startup; new `run/tests/m21_watcher.rs`; environment `combat_training.rs`, `combat_policy.rs`, `run_observation.rs`, new `run_schema_m21.rs`, existing exports; new `crates/spire-env/tests/m21_watcher_combat.rs`.

**Interfaces:** Produces `M21WatcherCombatStart`,
`Run::prepare_m21_watcher_combat(start, encounter, rng)` and
`M21WatcherCombatEntry` / `CombatTrainingCatalog::m21(entries)` exactly as in the
design. Existing `enter_training_combat` and `CombatTrainingSession::new` do the
work. Produces the two Watcher encoding functions and `RunEncodingProfile::M21Watcher`
with the design's exact IDs, vocabulary and fields. No RoomScenario/normal Run
profile or new Python transport constructor. `CombatTrainingSession::new_m21(catalog,
partition, caps)` is a capacity-configurable constructor on the same session
type/initializer; ordinary new uses the existing default capacities.
If accepted upstream lacks staged startup, shared CombatItemContext carries the
default-Legacy CombatStartupProfile; M21 selects M21Watcher before entry. Reuse
one extracted opening-hand owner through EffectKind::OpeningHand and dispatch
the existing PureWater effect once at Hook::CombatStartedPreDraw for M21.
The same catalog owner provides internal `m21_from_descriptor(&serde_json::Value)
-> Result<CombatTrainingCatalog, RunSessionError>`: strict shared descriptor codec
to typed entries, then `m21(entries)` and normalized-descriptor equality. Preserve
catalog entry order; hash the full descriptor with the design's NUL-terminated
domain and existing canonical_json. Replay owns no duplicate catalog parser,
admission/normalization or fingerprint algorithm.

- [ ] Write `m21_watcher_entry_transfers_canonical_or_diagnostic_resources`: canonical resources equal Run::new including72/72,99, exact10 starter instances, PureWater, empty slots/no keys; Neow stays unselected. Supplied rejects generated-only permanent cards, other cards/items, malformed IDs/counters, duplicate relics, upgrades>1, hidden inputs and unsupported numeric/resources. All57 encounter metadata comes from the shared setup owner; captured pools remain unchanged.
- [ ] Write `m21_watcher_startup_is_predraw_without_changing_legacy`: use the accepted staged startup seam, or proposed shared CombatStartupProfile/CombatStartedPreDraw/OpeningHand continuation. On M21, create one Miracle before opening draw; with requested opening10, hand holds Miracle plus9 permanent cards and the remaining permanent card stays in draw. Legacy/default still deals10 permanent cards then overflows Miracle to discard, preserving its accepted test. Canonical opening5 yields Miracle plus5 permanent cards. Check actual draw callbacks/Innate order, no duplicate later PureWater dispatch, and unrelated old starter relic behavior. A fault after Miracle insertion but before opening draw retains that child, all permanent cards in draw, detached inventory and the pending opening cursor without replay.
- [ ] Write `m21_watcher_entry_and_settlement_have_one_inventory`: preparation returns before startup; startup fault retains Run and detached inventory; genuine Victory/Death settles once, exact permanent instance census includes played powers/held zones; Watcher children never enter master deck. Enemy-generated status/curses and permanent Parasite still use accepted shared output/gain/encoding owners. A second core-owned JawWorm fight after JawWorm Victory keeps evolved HP/inventory and resets stance/Mantra/combat mutations. A standalone episode stops at first settlement.
- [ ] Write `m21_watcher_numeric_view_exposes_only_public_order`: encode Divinity3/history/current costs/base stats/Retain; revealed position and selected ordinal distinguish reversed choices; unknown remainder is unknown, not zeroed fake knowledge. Stance choice is separate from card offers. Candidate i is row i, mask shape[caps.actions], exact padding/known masks. No RNG, unseen order, family IDs or cursors in observations.
- [ ] Write `m21_watcher_session_retains_delivery_fault_and_tokens`: stale/wrong-session token changes nothing; reset/implicit ordinal behavior matches shared session; capacity too small for revealed choices/actions/children retains accepted core owner, serial and diagnostic prefix with no truncation/retry. Faulted owner cannot reset into a new fabricated loss.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m21_watcher_entry` and `cargo test -p spire-env --test m21_watcher_combat --locked --offline` to RED→minimal shared branches→GREEN. Compare all frozen predecessor descriptor/fingerprint fixtures and existing standalone v1/v2 catalog validation/token tests once.

### Task 7: Semantic replay on the same native session

**Files:** Existing environment `run_replay.rs`, `combat_training.rs`, optional focused `combat_replay.rs`, exports and `tests/m21_watcher_combat.rs`.

**Interfaces:** Adds `CombatEpisodeRecord` with profile/rules/content/observation/
replay/reward/RNG/seed identities and observation fingerprint;
`catalog_descriptor: serde_json::Value` (full normalized catalog),
`catalog_fingerprint: [u8; 32]` (whole-catalog fingerprint), `entry_id: String`,
`selected_entry: serde_json::Value` (complete normalized entry receipt),
partition/seed/ordinal, `Vec<RunRecordedStep>`, `Option<RunRecordedFault>`,
`Option<Outcome>` and closed status. Produces session `record()` and
`replay_m21_combat(record, caps)` from the design; `CombatReplayReport` contains
latest readable public decision, optional decision error, accepted step count,
rewards, final outcome and reported fault. Extract/reuse semantic combat action
and structured error helpers from existing Run replay; keep old schema guards.
Record JSON uses those exact catalog field names, the existing 32-byte-array
fingerprint convention, and the shared catalog's descriptor/entry representation.
Import/export and replay use Task6's m21_from_descriptor bridge. Replay order is
identity/capacity checks → reconstruct/validate full catalog → compare recomputed
whole-catalog fingerprint → resolve explicit entry_id and compare complete
selected_entry receipt → construct session/initialize seed+ordinal+selected ID →
existing semantic steps. Mismatch returns IncompatibleReplay before initialization.

- [ ] Write `m21_watcher_semantic_replay_preserves_choices_and_outcomes`: full canonical/supplied catalogs, whole-catalog fingerprints and selected-entry receipts round-trip through CombatEpisodeRecord::to_json/from_json; interrupted Scry/Meditate, deselect/reselect, zero-confirm and ChooseStance semantic actions replay by matching currently offered rows, including reversed selection histories and actual Victory/Death. Explicit-reset ordinal0 and later implicit ordinal1 replay through the same extracted episode initializer, without playing earlier episodes or resetting the ordinal. Candidate indices/session tokens are not portable identities.
- [ ] Write `m21_watcher_replay_validates_full_catalog_before_nonfirst_entry`: catalog order is A=canonical/JawWorm, B=supplied/Cultist with60/72HP,7gold, one unupgraded DefendPurple (permanent instance31), no relics/counters and three empty potion slots. Reset seed7/A, then implicit reset/B; record B at ordinal1 with an EndTurn prefix. JSON round-trip retains both entries, fingerprint equals the original full catalog, entry_id/selected_entry identify B including instance mapping, and replay matches B's public boundaries/reward/outcome/prefix at ordinal1. Clone the record, change only unused A's encounter to the supported Cultist while retaining the old fingerprint, and require IncompatibleReplay with zero shared episode-initializer calls (a local counter in the owning module's test, not production telemetry); no startup or step can run. Tamper the exported JSON equivalently and require import rejection too. Separately change entry_id to A while leaving selected_entry=B, or use an absent ID: both reject before initialization. Never repair the hash from selected B or silently fall back to entry0.
- [ ] Write `m21_watcher_replay_checks_every_identity_and_fault_prefix`: wrong profile/schema/rules/content/RNG/seed adapter/catalog/observation fingerprint rejects before stepping; unsupported choice verbs reject old schemas; committed core faults reproduce exactly once with readable accepted prefix; delivery-only failure is reported without injecting a fictional core error. Export remains readable after fault/close and None before reset/for legacy profiles. Unknown/extra fields and modified rewards/outcomes reject.
- [ ] Run `cargo test -p spire-env --locked --offline m21_watcher_replay` to exercise the owning-module and integration regressions through RED→shared replay adapter→GREEN, then existing `run_replay` and standalone session tests. Replayed execution must call the same session/core step, with no rule or session implementation copied into replay.

### Task 8: Freeze demonstrations, preserve the integrated candidate and review

**Files:** New native example/support and `crates/spire-core/tests/m21_watcher_combat.rs`; future candidate `docs/milestones/m21-watcher-combat.md`; scoped guide links/status only at implementation. Evidence outside Paseo.

**Interfaces:** Example accepts `--seed`, `--budget`, `--scenario starter|stance|mantra|scry|retain|creation|potions|death|all`. Defaults seed7/budget8192. It uses only public observations/legal candidates, no hidden state, altered enemy HP or synthetic results.

Freeze these scenario resources before first outcome, base cards unless marked:

| Scenario / encounter | Supplied deck (multiplicity explicit) | Relics / potions |
| --- | --- | --- |
| starter / JawWorm | Canonical; not supplied | Canonical |
| stance / Cultist | Vigilance,Eruption,EmptyFist,FlurryOfBlows,MentalFortress,Rushdown,LikeWater | PureWater,VioletLotus / empty |
| mantra / Cultist | Prostrate×3,Worship,Brilliance,Devotion,DefendPurple | PureWater / empty |
| scry / Cultist | ThirdEye,CutThroughFate,JustLucky,Weave,Foresight,Nirvana,StrikePurple,DefendPurple | GoldenEye / empty |
| retain / Cultist | WindmillStrike,Perseverance,SandsOfTime,Establishment,Meditate,Vigilance,DefendPurple | PureWater / empty |
| creation / Cultist | CarveReality,DeceiveReality,MasterReality,BattleHymn,DefendPurple×2 | PureWater,SacredBark / BottledMiracle,None,None |
| potions / Cultist | DefendPurple×5,Eruption,Vigilance,EmptyFist | TeardropLocket / StancePotion,Ambrosia,BottledMiracle |
| death / Cultist | DefendPurple | none / empty |

Supplied rows use72/72HP,99gold, three potion slots; death uses1/72HP. Instance
IDs follow input order then the profile's declared normalization. Potions are used
in ascending slot before playing cards. Stance choice always chooses Calm.
During Scry, select revealed Weave candidates in native order then confirm;
otherwise confirm zero. Other mandatory pile selections take the first legal
candidate until complete. At normal combat, card priority is the row's listed
kind order; ties/targets use first native candidate, then any other legal play,
then EndTurn. Starter always takes the first legal candidate. Death always ends
turn without playing. No policy reads seed, hidden draw order or privileged trace.

- [ ] Freeze the complete public caller source, scenario/catalog descriptor, seeds7/11 and budget8192 in retained evidence before first rollout. Unit tests pin its semantic choices on public fabricated decision inputs, not expected successful game outcomes. Any later authorized revision gets its own identity and preserves old results.
- [ ] Execute all scenarios at seeds7/11 once. Record actual actions, mechanic witnesses, terminal/unfinished/fault, HP/resources and settlement records. Genuine losses are valid; lack of a prescribed witness remains visible. Supplied diagnostics do not substitute for normal-start progression and no win is required. Run budget0/1 and bad-scenario controls; expected errors stay distinct from gameplay.
- [ ] Run one final stable matrix: `cargo test --workspace --locked --offline --no-fail-fast`, focused M21 core/env/replay checks, accepted predecessor preservation commands fixed in Task1, frozen descriptors/catalogs, relevant fixed-seed CLI summaries and all changed doc links/paths/commands/whitespace. Do not rerun broad baselines without a new change/failure/concern. Native results are not Python proof.
- [ ] Retain source/test/doc snapshots with SHA/mode manifests, exact commands/toolchain/features/results, source qualifications and every genuine loss/operational failure outside Paseo. Write the candidate profile before review, with exact limits and unresolved evidence. Planning's accepted2043 count is historical context, never the future result target.
- [ ] Self-check this plan against the design, including all five Review Focus classes and source-dependent signatures. Send root the exact frozen source/docs/facts/manifest hashes. Root schedules one fresh independent final milestone reviewer; author never final-reviews their own work. Resolve findings on preserved candidates, then record actual root acceptance only if obtained. No acceptance, publication or archival claim follows merely from a completed checklist.
