# M8 Act3 Mechanics and Content Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:executing-plans for native single-owner implementation, useful isolated Luna larger-reading assistance and ONE independent milestone review. Track the checkboxes in the milestone ledger. No per-enemy specification/reviewer swarm or overlapping source ownership.

**Goal:** Implement the declared Ironclad A0 Act3 encounters, elites, all three boss mechanics and two persistent events on the existing resolver, then demonstrate a normal-start, same-player Act3 prefix.

**Architecture:** Extend native actor memory, lifecycle and queued effects rather than introduce another combat engine. Add a separate captured M8 room profile continuing the accepted M7 path through Act3's actual chest floor43; preserve M5/M6/M7 endpoints. Standalone boss diagnostics establish mechanics, while full normal-start Act3 boss/reward completion remains M9.

**Tech Stack:** Existing Rust workspace and deterministic native tests/examples, locked/offline dependencies; committed standard-library JSON references. No new dependencies, installations, training code or verification platform.

**Spec:** User-approved [36-milestone roadmap](../specs/2026-10-22-narrow-gym-milestones-design.md), delivery principles and §§M8–M9; accepted [M7 profile](../../milestones/m7-act2.md); [reference library](../../../data/README.md) at its stated evidence grades.

**Status:** User "I approve it" approves this actual written plan; planning `.30.1` closed normally and native implementation `.30.2` claimed. Baseline is local `9fa436c` including committed M7 `2d9a47e` and the public references, now also merged into local main under separate authority. Preserve native execution/TDD/one milestone review; no new commit/push/install/cleanup/private lookup authority.

## Global Constraints

- Approved M8: "Add the Act3 encounters, bosses, events and required player/item interactions, including distinctive counters, phases and interruption behavior. Reuse the same resolver and run coordinator. Preserve prior acts while demonstrating actual Act3 fight/room behavior. There is no new character-wide implementation prerequisite."
- Final target remains four fully unlocked characters, A0–20, Acts1–4/Heart and normally reachable content, A20-primary. This milestone adds an explicitly captured Ironclad A0 profile, NOT standard full pools or other-character/Ascension acceptance. New unsupported Ascension branches reject, not fall back to A0; preserve existing Hard Jaw/Cultist/Nob broader profiles.
- Normal creation stays 80/80HP,99gold, five Strikes/four Defends/Bash, Burning Blood, empty three-slot potion belt and no keys. Reuse the original preselected seeds7/11 and valid discovery histories. No seed search, resource injection, modified game values or near-end fixtures replacing normal-start evidence. Genuine Death is valid gameplay, not a failure to tune away.
- One authoritative Run/Combat, permanent deck, item context and queue. Preserve stable actor handles/logical slots, actor-owned RNG and retained source/child ownership. Extend shared mechanics only for named Act3 producers and preserve legacy combat/snapshot/RNG contracts.
- Reward policy unchanged: new highest Acts1–3 floor+1, each first distinct key+5, real Death/settled EmptyDeck−1, Heart success+100 only at Heart. No encounter-win, event-choice, prefix-readiness or act-initialization bonus. Faults/unfinished budgets are errors, never game losses, wins or truncations.
- Read domain guides and specific records/citations first; reference counts/identities/integrity checks do not prove mechanics. Keep null/provisional constructor identities, conflicting claims, local source grades and history-conditioned probability qualifications. Native internal enum names are not claims of exact original constructor strings.
- RNG-001 preserves conditional laws, exclusions and deliberate dependencies, not Java PRNG, seed mapping, incidental cursor consumption or stream coupling. TIME-001 remains the Portal time-eligibility assumption; Portal gameplay is not in this captured prefix and no clock/frame/wait system is introduced. Preserve earned terminal healing.
- Native single-owner integration, useful isolated Luna assistance, ONE final independent milestone review. Each assistant/reviewer has its own explicitly current-source-materialized worktree and unique absolute build directory. Reuse existing reference handoffs; no duplicate dataset research or guessed historical assignments.
- Tests are bounded deterministic rule examples: valid fixtures, literal expected values, exact branch tickets, no Debug goldens, random-frequency campaigns, sleeps/network or timing assertions. Diagnose warm focused commands >5s or workspace >30s; report compilation separately.
- Historical baseline evidence: core534/7 with the same seven underlying Poison/Catalyst/Artifact and Slaver/Fungi draft failures; public50/0, M7internal16/0, M6native90/0, consumers43/0, other21/0, reporting1/0, seven examples; reference51/0. Rerun after approval before edits; compare failure names/underlying assertions, not counts alone. Do not fix unrelated drafts to manufacture green.
- No original game execution, new private lookup, whole-archive audit, installation, generic verifier, source copying, commits/pushes/merges or cleanup. The earlier M6 private static-lookup grant and completed pre-M8 local commit request do not authorize M8 access/publication. Preserve every checkout, evidence file and unfamiliar change.

## Review Focus

1. A zero-HP Darkling or phase-one Awakened One must remain an owned pending revival, not a corpse that prematurely settles Victory or receives ordinary death rewards; final cleanup must not erase earned effects (Tasks2/7).
2. Time Warp's twelfth card, including a multi-hit/draw/selection-producing card, must retain the accepted card's legitimate work and stop further plays at the actual turn boundary without deleting unrelated effects or duplicating EndTurn (Task8).
3. Writhing Mass can reroll per qualifying hit and obtain a permanent Parasite during combat; no live permanent-card census, identity frontier, fault or subsequent room may lose/duplicate that acquisition (Task4).
4. Reptomancer/Daggers, Exploder self-destruction, Transient expiry and Donu/Deca use different lifecycle/settlement rules; generic Minion/HP-zero shortcuts must not conflate them or release pending spawn ownership (Tasks1/3/6/9).
5. A real floor43 chest, full potion belt, exhausted finite captures, event removal, or retained fault must not fabricate prefix completion, prune a generated edge, reapply a prior heal, expose future content or change older no-action cutoffs (Tasks10–12).

## Scope decisions and evidence boundary

**Mechanics roster:** Orb Walker; Repulsor/Exploder/Spiker and their actual Shapes groups; three Darklings; Spire Growth, Transient, Maw, Writhing Mass; existing hard Jaw Worm Horde; Giant Head, Nemesis, Reptomancer/Daggers; Awakened One with two Cultists, Time Eater, and Deca then Donu. Reuse existing Spheric Guardian in Sphere and Two Shapes. Every named supported species gets actual setup, AI, payloads, reactions and lifecycle, not definition-only registration.

**Required player/item additions:** real Inflame (Power,1energy,Strength2/3), Void (draw-energy-loss/Ethereal status), Parasite (permanent Curse and removal max-HP loss), and Red Mask (battle-entry Weak1 to each initial enemy, subject to Artifact). These make concrete boss/event interactions executable; they are not a new all-card/item prerequisite. Keep prior captured cards/relics/potions unchanged outside the M8 Act3 capture.

**Events:** Falling and Tomb of Lord Red Mask, with all their represented A0 choices. Falling captures one eligible permanent instance per present Attack/Skill/Power type at entry, publishes fixed choices in Skill/Power/Attack order and disables missing types; no types yields Leave. Tomb offers222gold when Red Mask is owned; otherwise paying ALL current gold, including0, obtains Red Mask; Leave remains legal. No duplicate grant or per-read reroll. Other Act3 events/one-time pools, bottled-card flags and unrepresented eligibility overrides remain explicitly unsupported, not silently implemented by these two examples. Mind Bloom, Winding Halls, Mysterious Sphere and Portal are not claimed here.

**Public progression:** `RoomScenario::M8Act3Prefix` continues the same accepted Acts1–2 path, second boss chest and ordinary2→3 heal/map/boss-order initialization. M8 alone publishes Map at act3/floor34. Admit every generated outgoing edge for floors35–43, resolving real ordinary/elite/shop/rest/question/treasure sources. Stop ONLY after the actual fixed chest43 source Proceed: `RoomViewKind::Act3PrefixComplete`, alive/nonterminal, no actions including potions. No entered44, boss50, Act3 clear, Act4 or Heart claim. M7 still stops at Act3Ready34 without actions.

**Captured scheduling:** first two ordinary fights are Orb Walker then Three Shapes. Subsequent ordinary fights rotate the explicit native strong list `[SpireGrowth,Transient,FourShapes,Maw,SphereAndTwoShapes,JawWormHorde,ThreeDarklings,WrithingMass]`, scanning forward to honor the first-strong Shapes exclusion and later last-two-family exclusions; no arbitrary history reset. Elites rotate `[GiantHead,Nemesis,Reptomancer]`, offset0/1/2 for the public AwakenedOne/TimeEater/DonuAndDeca icon, without immediate repeats. Questions resolve Falling→Tomb→Monster; this is a named capture, not standard event/pool probabilities. Actual combat setup samples the group's qualified composition, HP and AI; never script its outcome.

**Captured loot/economy:** retain current source-class gold minima10 ordinary/25elite plus bounded11, Fire/Strength/Weak potion cycle and three nonboss card rows, with only the first Act3 ordinary row changed to `[Inflame,ShrugItOff,Anger]`. Act3 uses its own settled-visit and elite-relic cursors, skipping owned entries in the existing finite eight-relic sequence. Treasure uses existing Strawberry/Anchor/BloodVial captures and Sapphire exclusivity; shop uses the existing later-shop Clothesline50/StrengthPotion50/unowned Anchor150 plus inherited purge price. Owned unique stock is disabled; exhausted captures remain explicit operational/eligibility limits, never invented replacements, easier rooms or silent edge removal. Demonstrations must retain sufficient genuine offered stock; no claim that all possible routes/inventories are supported. A chest's optional Proceed remains available without taking loot; exhaustion is never treated as a victory/loss or permission to fabricate a relic.

**Normal-start acceptance:** use seed11 with each of the three valid unseen Act3 boss histories, keeping the accepted first-act Guardian and second-act Automaton history. Exercise actual third-act ordinary combat, an elite, a persistent event consequence, real loot and chest43 resolution. Require at least one normal-start prefix completion and actual mechanics diagnostics for all three elites/bosses. Record each preselected run's real outcome; do not require every seed/icon to win. Replay all supplied accepted traces, including natural losses. If no preselected path reaches the prefix or no path exercises a persistent event/elite, report that concrete integration gap rather than substitute a late fixture or secretly search for a winner.

## Qualified facts and owning-task source closure

Use `data/enemies/{enemies,encounters,sources}.json`, `data/powers/powers.json`, the specific card/relic records and their guides; existing public `docs/research/2026-10-18-ironclad-a0-acts23-research.md` supplies qualified A0 formation/event/transition findings, not enemy-mechanics closure. Its clock/Reaper proposals are superseded. Preserve these reference files unchanged unless separately assigned a data correction.

The committed references deliberately leave the following exact implementation facts unclosed. Before fixing an oracle/body for the owning task, write a short task-local ruling using retained authored findings or a bounded read of the named cited public source/method. Resolve exact branch redirects/secondary coins, not a rounded rate. If those inputs cannot settle it, stop THAT task and name the missing method/interaction for separately authorized lookup; do not promote wiki nominal probabilities, guess, add an approximate supported handler or restart a whole-game audit. Written-plan approval does NOT itself grant private access.

| Owner | Named facts to settle before implementation |
|---|---|
| Task1 | Shape slot-construction laws (shrinking six-member pool versus mixed helper); Exploder's fixed30 damage type/modifier/prevention order and self-death callback sequence |
| Task2 | Darkling center/outer branch redirects, zero-HP/debuff/Block handling, Regrowing→Reincarnate timing and final all-down kill arbitration |
| Task3 | A0 Spire Growth live-Constricted selection/timing; exact Transient turn/Fading/Shifting checkpoint and Maw turn-based hit count; A17 threshold conflicts stay outside A0 |
| Task4 | Writhing Mass disputed opening support and all reroll redirects; Malleable/Reactive callback ordering including blocked/lethal hits; Parasite once-used versus prevented acquisition and permanent-card publication |
| Task5 | Nemesis exact integer partitions/redirects versus approximate35/35/30, Intangible application/expiry and modifier order; Slow's increment/damage rounding checkpoint |
| Task6 | Reptomancer exact integer partitions/redirects, initial logical slots, available-slot spawn order, Dagger explosion type and actual leader-death cleanup |
| Task7 | Awakened half-death/Rebirth power and queue handling, final Cultist escape, Curiosity play callback; Void draw/energy clamp |
| Task8 | Time Warp accepted-card/turn-boundary ordering and Haste strict threshold/selection latch; Draw Reduction application/removal checkpoints |
| Task9 | Group formation Deca then Donu; verify group buff publication/queued order and action-specific later target eligibility (Task9 source ruling: publication includes all current list entries, not a living-prefiltered snapshot) |
| Task10 | Falling constructor-time eligibility/sample and removal callbacks; Tomb action/source completion and Red Mask's real battle-entry trigger |

These are task-local acceptance prerequisites, not a new complete source-manifest or private-audit gate. Known literal payloads below stay at their supporting/community grades until the relevant source path is reconciled. No original-runtime claim follows.

## File responsibilities and shared interfaces

Paths below are relative to `crates/spire-core/` unless a repository-root prefix is stated.

| Files | Responsibility |
|---|---|
| `src/combat/enemies/act3.rs` and new `act3/{shapes,darklings,ordinary,writhing,elites,reptomancer,awakened,time_eater,partners}.rs` | Small native species/group handlers, not a second scheduler |
| Existing `src/combat/{model,engine,effects,turns,hooks,powers,view,card_state}.rs`, `enemies/{mod,act2}.rs`, `enemies/act2/spawn.rs` | Minimal shared lifecycle, reaction, interrupt and permanent-acquisition seams; preserve existing producers |
| `src/content/{ids/encounters,ids/cards,ids/items,ids/events,cards,items,encounters,events}.rs`, `src/combat/cards/ironclad.rs`, `src/items/relics.rs` | Actual named IDs/definitions and Inflame/Void/Parasite/RedMask consumers, registered alongside bodies |
| `src/run/rooms/{mod,act1,act2,transition,rewards,shop,events}.rs`, new `rooms/{act3,events/act3}.rs`, `src/run/{combat,model,mutations,rng}.rs` | Captured same-player Act3 progression, permanent gains and event source ownership |
| New `src/combat/tests/m8_*.rs`, `src/run/tests/{m8_rooms,m8_events,m8_acquisition}.rs`; corresponding `tests/mod.rs` | Focused lifecycle/ordering/source/fault diagnostics |
| New `tests/m8_act3.rs`, `examples/{m8_act3.rs,m8_support/mod.rs}` | Public-only normal-start/replay/outcome caller and explicit supplied-resource diagnostics |
| Root `docs/milestones/m8-act3.md`, README/scope/status/AGENTS and milestone ledger | Exact profile, commands, evidence, unsupported branches and one-review gate |

Retain `enemies::setup(encounter,act,ascension,&mut RunRng,SetupContext)->Result<CombatEntry,RunError>`, `Combat::{enter,step_full,public_full,exit}` and existing Run entry/settlement ownership. New `act3::select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&act2::EnemyRollContext)->Result<MoveSelection,SelectError>` and `act3::take_turn(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,EffectError>` plug into those same dispatchers. Add live powers/lifecycle information to the existing context where a named handler needs it; do not reconstruct hidden game state in an example policy.

Shared seams are narrowly named: add `ActorLife::HalfDead` for scheduled revival (not replacement/escape), expose its public phase without private memory/RNG, and make targeting/scheduler/victory predicates distinguish it. Add species-specific typed effects for regrowth/rebirth, fixed self-destruction, Time Warp and permanent-card gain only as required; all run through the existing queue/guard/fault owner. Preserve `act2::roll_move` canonical damage refresh and existing Stasis/death custody. Do not globally ignore actors carrying Minion.

For Parasite, use a retained, read-only combat acquisition journal rather than waiting until Victory: `PermanentCardGain{ordinal:u64,id:CardId,upgrades:u32}` and crate-visible `Combat::permanent_gains(&self)->&[PermanentCardGain]`, also readable after a retained fault. `ActiveCombat` records the applied ordinal and the exact newly allocated permanent instance IDs. Native `Run::sync_permanent_gains(&mut self)->Result<(),RunError>` preflights identity/capacity and applies each accepted entry once after every combat action, before another public decision or settlement, including accepted work preceding a fault. Acquired Parasite goes into the Run deck only, not current combat piles; census excludes ONLY those recorded new instances. Preserve original permanent origins and item-hook ownership. This journal records actual gains, not a duplicate deck/inventory or generic event bus.

Inflame also exposes a genuine existing census gap: a played Power leaves ordinary
combat piles but its permanent identity must survive settlement. Task7 adds a
`State.played_powers:Vec<OwnedCard>` inactive zone alongside existing owned piles
(Task7 native inspection corrected the earlier handle-only/arena assumption;
the current model owns card values directly, so this is a move, not a copy),
`CombatPublic.played_powers:Vec<LiveCardView>` for already-visible resolved cards,
and `CombatExit.played_powers:Vec<LiveCard>`. FinishCard moves an actually played
Power there, not to Exhaust/discard and not a perpetually pending in-play slot.
Run census includes that zone exactly once on Victory/Death. No second card store
or change to legacy snapshot format is implied.

`act3::Act3Progress{question_visits:usize,shop_visits:usize,combat_visits:u64,elite_relic_cursor:usize}` belongs to `RoomLoop.act3:Option<Act3Progress>`. New room methods retain existing patterns: `capture_room(state:&RunState,node:NodeId)->Result<act1::CapturedRoom,RunError>`, `enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>`, `finish_room(run:&mut Run)->Result<RunTransition,RunError>`. Add `is_m8(state:&RunState)->bool` and `uses_captured_act3(state:&RunState)->bool`; widen `uses_captured_act2` for M8 but keep `is_m7` exact. Validated2→3 transition allows M7/M8, only M8 initializes Act3Progress/Map. No new3→4 handoff.

Falling uses `RunRngScope::Act3Falling` / `run.act3.falling`, with scoped `[3,node_id,question_visit,card_type]` and canonical permanent-instance candidate order. `events::act3::prepare(run:&Run,event:EventId,next:&mut u64)->Result<PreparedAct3Event,RunError>` returns the detached screen, captured instance frame and RNG; `events::act3::choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>` validates that frame. `PreparedAct3Event` and private Falling frame live in `rooms/events/act3.rs`; commit only after move/reward preflight. Add `EventId::{Falling,TombOfLordRedMask}` and public `RoomChoiceKind::{FallingLose{card_type:CardType,instance:Option<u64>},TombGold,TombOfferGold}`. Missing-type rows are Disabled, not legal None selections. Use existing Leave for Tomb/finished events and Falling's no-eligible case.

## Tasks

### Task 1: Orb Walker and real Shapes formations/reactions

**Files:** create `src/combat/enemies/act3/shapes.rs`, implement the existing `act3.rs` router; modify `enemies/mod.rs`, content encounter IDs/definitions and shared damage/hooks only where needed; create/register `src/combat/tests/m8_shapes.rs`.
**Interfaces:** consumes existing setup/memory/actor RNG/CreateCard pipeline; produces real OrbWalker/ThreeShapes/FourShapes/SphereAndTwoShapes setup and Act3 routing above. Missing other Act3 species remain explicitly rejected until their task lands.

- [ ] After written-plan approval, normally close planning/create and claim implementation; run Task12's baseline using only EXISTING targets. Close Task1's named source facts before fixing the damage/composition oracle; preserve actual evidence grades.
- [ ] Write `m8_orb_growth_and_burn_destinations`, `m8_shapes_use_qualified_ordered_composition`, `m8_spiker_thorns_per_hit_and_six_buffs`, `m8_exploder_countdown_and_early_kill`. Assert OrbHP90–96/baseLaser10/Claw15/end-own-turnStrength3; Laser inserts one Burn in draw and one discard. Shapes select real members, not one fixed easy formation; Repulsor adds2Dazed to draw; Spiker startsThorns3, buffs+2 up to six executed buffs, Attack7, fully blocked NORMAL hit still reacts. ExploderHP30 attacks9 twice then actual fixed30/self-death; early killing it produces no explosion. Pin Block/Weak/Strength/Vulnerable/prevention behavior to the qualified named explosion path, not `EnemyMove` inference.
- [ ] Run `cargo test -p spire-core --lib m8_shapes --locked --offline`; require a behavioral missing/rejected-handler RED, not only enum compilation.
- [ ] Implement the real native bodies/setup/streams and reactions; reuse SphericGuardian and shared generated-status handling. Preserve actual self-death callbacks and queued player Death priority. Add registry entries with bodies, not false supported stubs.
- [ ] Rerun GREEN plus prior M6 ordinary/shared-damage consumers. Confirm actual group readiness with an exploding actor beside a live ally and player-lethal explosion; invalid setup/RNG exhaustion retains the correct pre/post-acceptance state.

### Task 2: Darkling half-death, regrowth and group victory

**Files:** create `act3/darklings.rs` and `tests/m8_darklings.rs`; modify shared lifecycle/targeting/turn/readiness seams in `model.rs`, `effects.rs`, `turns.rs`, `view.rs`, `enemies/{mod,act2}.rs` and setup.
**Interfaces:** produces shared HalfDead semantics for Task7 without changing existing Replaced/Escaped meaning or pretending every zero-HP actor is gone.

- [ ] Close Darkling source facts, then write `m8_darkling_center_identity_and_fixed_nip`, `m8_half_dead_is_untargetable_but_scheduled`, `m8_darklings_require_simultaneous_down`, `m8_regrowth_fault_retains_actor`. Use A0HP48–56, per-instance Nip7–11, Harden12 and outer Chomp8×2; center by original logical identity cannot Chomp. Down one actor while another lives: no Victory/death reward, no ordinary target, actual Regrowing/Reincarnate restores qualified half-max HP without new constructor rolls. All-down resolves one final encounter settlement, not three room rewards.
- [ ] Run `cargo test -p spire-core --lib m8_darklings --locked --offline` to actual lifecycle RED.
- [ ] Implement typed species memory/effects and shared lifecycle-aware actor iteration/readiness. Keep actor IDs, permanent cards, counters and RNG ownership through revival; distinguish temporary half-death from final real death callbacks and corpse pruning.
- [ ] Rerun GREEN with AoE/multi-hit all-down, one survivor, repeated cycles and accepted revival fault. Re-run Split/Collector/Automaton/Stasis consumers to catch changed shared victory predicates.

### Task 3: Spire Growth, Transient and Maw live turn mechanics

**Files:** create `act3/ordinary.rs`, `tests/m8_ordinary.rs`; extend shared post-damage/turn/power dispatch and setup only for named consumers.
**Interfaces:** reuses current live roll context and DamageResult; produces real SpireGrowth/Transient/Maw plus existing JawWormHorde admission.

- [ ] Close A0 timing/selection facts, then write `m8_growth_checks_actual_constricted`, `m8_transient_shifts_by_hp_loss_and_fades`, `m8_maw_uses_enemy_turn_not_card_count`, `m8_hard_jaw_is_reused`. Assert GrowthHP170/Constricted10/attacks16,22; Artifact-prevented Constricted does not count as present. TransientHP999, first base30 from20+10×turn1, Shifting uses HP lost not attempted/blocked damage and restores temporary reduction at its qualified boundary; expire after five actual actions with native cleanup, not a player kill. MawHP300 opens Weak3/Frail3; Slam25, DroolStrength3, Nom5×ceil(enemyTurn/2).
- [ ] Run `cargo test -p spire-core --lib m8_ordinary --locked --offline` to runtime RED.
- [ ] Implement actual moves, Constricted expiry/callback and turn counters on the same queue; distinguish Fading expiry from player damage/death and do not reset counter on a player card or move-selection read. Reuse HardJaw setup/ascension facts rather than duplicate them.
- [ ] Rerun GREEN with fully blocked hits, multi-hit actual HP losses, Artifact and player-lethal fifth attack. Re-run existing power duration/Artifact and hard-Jaw tests without changing historical Poison drafts.

### Task 4: Writhing Mass reactions and permanent Parasite custody

**Files:** create `act3/writhing.rs`, `tests/m8_writhing.rs`, `src/run/tests/m8_acquisition.rs`; modify shared damage/engine/model/exit projection, `src/run/{combat,model,mutations}.rs`, card IDs/metadata/status handling.
**Interfaces:** produces the permanent-gain journal/Run synchronization contract defined above; later events/shops remove the same actual Parasite identity, not a combat-local copy.

- [ ] Close disputed opening/redirect/callback/obtain facts; write `m8_writhing_rerolls_each_qualifying_hit`, `m8_parasite_used_not_selected`, `m8_permanent_gain_sync_survives_fault_and_settlement`, `m8_parasite_removal_not_combat_exhaust`. Pin HP160/Malleable4, Multi7×3/Debuff10/Big32/BlockAttack15 and Weak2/Vulnerable2. Task-local owned-static closure corrects the earlier wiki-derived fixedBlock16: Block uses the attack entry's base,15 at A0, not modified output; retain source/build qualifications in data/enemies/m8-static-findings.md. After each qualifying hit refresh the real intent/damage, preserving actual owner RNG. Selecting then rerolling away from Parasite must not latch usage; its actual accepted effect yields one permanent unplayable Curse and no combat-pile card. Removal loses3maxHP once (88→85 with HP clamped), not on combat exhaustion; no upgrade option.
- [ ] Run `cargo test -p spire-core --lib m8_writhing --locked --offline` and `cargo test -p spire-core --lib m8_acquisition --locked --offline` to actual RED, including the old exact-deck exit assumption.
- [ ] Implement native callbacks and retained journal with checked ordinals; synchronize on accepted action/fault before another decision, preflight allocation/capacity, and acknowledge only after commit. Exclude only exact recorded acquired IDs from current-combat origin census. Use active item ownership for applicable supported acquisition hooks; unsupported prevention relics remain explicit, not silently claimed as implemented.
- [ ] Rerun GREEN for identity/capacity overflow during synchronization (retain the accepted journal and fault, with no partial Run allocation), a fault after accepted acquisition, repeated synchronization, Victory and Death exits, then later ordinary purge of Parasite. Falling only accepts Attack/Skill/Power, so it must NOT offer this Curse. Retain original permanent origins; no grant-at-Victory shortcut, duplicate, disappearing Curse or new full inventory copy.

### Task 5: Giant Head Slow and Nemesis Intangible

**Files:** create `act3/elites.rs`, `tests/m8_elites.rs`; modify real card-play/hit/turn dispatch, power validation and damage refresh.
**Interfaces:** consumes native card-play checkpoint and incoming-damage reducer; produces two actual elite handlers without a per-card special damage calculator.

- [ ] Close Slow/Intangible and exact Nemesis selection facts; write `m8_slow_counts_cards_and_resets`, `m8_head_countdown_and_damage_cap`, `m8_nemesis_intangible_is_per_hit`, `m8_nemesis_history_redirect_tickets`. Assert HeadHP500, Count13/Weak1, A0 ItIsTime starts on the qualified fifth enemy turn with bases30,35,…60cap; Slow's precise within-turn order/rounding comes from the named fact ruling. NemesisHP185, TriAttack6×3/Scythe45/Burns3 to discard; no opening Scythe. Intangible caps each qualifying hit separately at its actual alternating boundaries, not the card total.
- [ ] Run `cargo test -p spire-core --lib m8_elites --locked --offline` to behavioral RED.
- [ ] Implement the shared modifier and actual native counters/power lifetimes; use qualified integer tickets/redirects and per-actor streams, not approximate35/35/30 renormalization.
- [ ] Rerun GREEN for zero/fully blocked/multi-hit/Orb/THORNS/HP_LOSS cases at the source-qualified modifier boundaries, TurnEnd then TurnStart and damage-preview refresh. Re-run existing Flight/Weak/Vulnerable/Strength consumers.

### Task 6: Reptomancer and stable-slot Daggers

**Files:** create `act3/reptomancer.rs`, `tests/m8_reptomancer.rs`; reuse/extend `enemies/act2/spawn.rs`, shared lifecycle/hooks and actor diagnostics.
**Interfaces:** existing SpawnRequest/CompleteSpawn ownership; no parallel spawn queue or replacement of the parent with its children.

- [ ] Close exact source facts, then write `m8_reptomancer_initial_slots_and_live_cap`, `m8_dagger_wound_then_explosion`, `m8_reptomancer_cleanup_not_global_minion_ignore`, `m8_dagger_spawn_fault_keeps_committed_child`. Assert parentHP180–190/two initial Daggers, A0 Spawn1 up to4live; DaggerHP20–25 attacks9+Wound1 to discard then explosion25/self-death. ParentSnakeStrike13×2+Weak1/BigBite30. A live Dagger count of4 redirects Spawn, never creates a fifth; dead historical children do not consume active slots.
- [ ] Run `cargo test -p spire-core --lib m8_reptomancer --locked --offline` to actual RED.
- [ ] Implement qualified initial formation, native child constructor/AI streams, stable logical slots, exact proposal redirects and actual parent-death cleanup. Cancel only invalid pending enemy work; do not extrapolate Collector suicide semantics or treat all Minions as absent.
- [ ] Rerun GREEN for reused vacant slots with new handles, retained constructor/partial-child RNG faults, parent death during pending summon and player death. Re-run GremlinLeader/Collector/Automaton spawning/custody checks.

### Task 7: Awakened One phases, real Power play and Void

**Files:** create `act3/awakened.rs`, `tests/m8_awakened.rs`; modify native card IDs/definitions, `cards/ironclad.rs`, draw/card-state/FinishCard/exit handling, `src/run/combat.rs` census and shared lifecycle/hooks.
**Interfaces:** reuses Task2 HalfDead and existing Cultist; adds actual Inflame/Void producers needed by boss/event diagnostics and later captured rewards.

- [ ] Close source phase/callback facts; write `m8_inflame_is_a_real_power_producer`, `m8_awakened_curiosity_and_regenerate`, `m8_awakened_rebirth_is_not_victory`, `m8_void_draw_and_final_cultist_escape`. Assert ordered Cultist/Cultist/Awakened300HP; Curiosity1 per qualifying Power use, Regenerate10 at end-own-turn; Inflame1energy grants2/3Strength and follows actual Power disposition. Phase1 Slash20/SoulStrike6×4; first depletion is half-death, Rebirth full heal/debuff cleanse/Curiosity removal with accumulated Strength retained; phase2 DarkEcho40 then Sludge18+Void-to-draw or Tackle10×3. Void loses1energy only on actual draw, is unplayable/Ethereal and does not imply permanent deck acquisition. Final boss death escapes surviving real Cultists through qualified cleanup.
- [ ] Run `cargo test -p spire-core --lib m8_awakened --locked --offline` to runtime RED.
- [ ] Implement native phase transition, real Power/card hooks and status handling without replacing live actor identity or blanket power removal. Only final phase allows settlement; preserve legitimate accepted card cleanup/healing across phase interruption.
- [ ] Rerun GREEN at phase lethal mid-multi-hit, blocked/debuff/negative-Strength cases, zero/full-energy Void draw, early Cultist deaths and final boss death. Assert a played permanent Inflame appears once in the resolved Power zone/exit, never in discard/Exhaust, and remains the same Run deck instance on Victory and Death. Re-run prior half-death/shared healing/terminal card-custody checks.

### Task 8: Time Eater counter, Haste and turn interruption

**Files:** create `act3/time_eater.rs`, `tests/m8_time_eater.rs`; modify actual card-use/turn queue hooks, Draw Reduction and damage refresh.
**Interfaces:** a named pending Time Warp turn-end request on the existing State/queue, not an external caller action, generic queue wipe or second turn scheduler.

- [ ] Close callback/threshold facts; write `m8_time_warp_twelfth_card_finishes_owned_work`, `m8_time_counter_survives_turns_and_invalid_actions`, `m8_time_haste_once_strict_threshold`, `m8_time_draw_reduction_lifecycle`. Assert HP456; counter11 plus one accepted card triggers at12, resets according to source and grantsStrength2; invalid/stale actions and potions do not count as card plays. Preserve current-card multi-hit, Offering payment/draw, BurningPact selection and FinishCard work in source order, then exactly one actual EndPlayerTurn; no thirteenth play opportunity. Haste is one-time below228, not at228; cleanse/heal to228 on its real action. Normal bases7×3/26 and RippleBlock20+Weak1/Vulnerable1; Draw Reduction uses the qualified next-draw timing.
- [ ] Run `cargo test -p spire-core --lib m8_time_eater --locked --offline` to actual interruption RED.
- [ ] Implement the named barrier on the shared queue, including selection resume, terminal priority and retained fault behavior; use real Haste selection/execution latches. No arbitrary pruning of earned effects or hidden extra player input.
- [ ] Rerun GREEN for counter across two ordinary turns, twelfth-card lethal boss/player outcomes, a pending selection and fault before/after the barrier. Re-run card selection/exhaust/draw/terminal ownership and all predecessor public tests.

### Task 9: Actual Deca/Donu independent partners

**Files:** create `act3/partners.rs`, `tests/m8_partners.rs`; native setup/actor context and existing group apply-power paths.
**Interfaces:** shared setup and actor-order queue; no single combined boss actor or ally-death phase reset.

- [ ] Write `m8_deca_donu_order_and_independent_cycles`, `m8_pair_buffs_target_live_actors`, `m8_pair_artifact_and_either_death_first`. Assert actual Deca then Donu, eachHP250/Artifact2; Deca opens10×2+2Dazed to discard then Block16 to each living enemy; Donu opensStrength3 to each living enemy then10×2. Each keeps its own cycle if the other dies, and surviving attacks use live Strength. Killing one alone never wins.
- [ ] Run `cargo test -p spire-core --lib m8_partners --locked --offline` to actual RED.
- [ ] Implement setup/payloads and source-qualified full-list snapshot/order with action-specific execution target guards using existing context; own cycle changes on move publication, not selection. Validate Artifact prevention without inventing a paired shared turn counter. (Task9 ledger records the source correction.)
- [ ] Rerun GREEN with either boss dying first, attacks between a group buff's publication/execution, Weak blocked by Artifact and final single settlement. Re-run Centurion/Mystic and existing group-target consumers.

### Task 10: Falling, Tomb and real Red Mask entry behavior

**Files:** create `src/run/rooms/events/act3.rs`, `src/run/tests/m8_events.rs`; modify events router, event/relic IDs/definitions, run mutation removal and `src/items/relics.rs`.
**Interfaces:** detached PreparedAct3Event/Falling frame and public choice types above; uses ordinary obtain/remove/gold and current item context, not bespoke event inventory.

- [ ] Close source facts, then write `m8_falling_captures_one_per_type_once`, `m8_falling_missing_types_and_last_card`, `m8_tomb_all_gold_includes_zero`, `m8_red_mask_weak_once_per_entry`, `m8_event_preflight_and_accepted_fault`. Falling samples canonical permanent identities once at constructor/entry, including duplicate card kinds, and removes only the selected captured instance; repeated public reads draw nothing. Missing types disable rows, all missing offers Leave; last eligible card removal settles EmptyDeck−1 only if the whole deck is empty. A Curse-only Parasite deck instead offers Leave and loses neither card nor max HP. TombwithMask pays222 once; noMask pays actual gold0/99/all then acquires one RedMask; Leave changes nothing. At battle entry RedMask appliesWeak1 to each initial eligible live enemy via ordinary Artifact handling, not later summons, and not on every turn.
- [ ] Run `cargo test -p spire-core --lib m8_events --locked --offline` to runtime RED.
- [ ] Implement the detached capture/atomic preflight and source/choice validation; add scope/parser round trip, actual acquire/remove callbacks and once-only event completion. No arbitrary purge grid, reselect on click, free curse removal or duplicated gold/relic if publication faults after acceptance.
- [ ] Rerun GREEN for no Power, real Inflame, two same-ID instances, empty/curse-only deck, full potion belt, token/RNG/gold overflow and accepted removal/grant fault. Re-run AncientWriting/Joust/shrine/purge and permanent acquisition consumers.

### Task 11: Same-player Act3 prefix, source loot and predecessor isolation

**Files:** create `src/run/rooms/act3.rs`, `src/run/tests/m8_rooms.rs`; modify rooms/transition/combat/rewards/shop/model/mutations gates as listed above. No boss50 admission or Act3 completion transition.
**Interfaces:** consumes Tasks1–10 native bodies and events, produces M8Act3Prefix/Act3Progress/Act3PrefixComplete and normal public move/room actions; existing Run method signatures stay unchanged.

- [ ] Write `m8_transition_continues_same_player_without_rehealing`, `m8_all_prefix_edges_keep_local_owners`, `m8_source_loot_and_capture_exhaustion`, `m8_chest_proceed_is_actual_cutoff`, `m8_old_ready_and_prefix_profiles_unchanged`. Admit every generated edge35–43 with real source kind and native encounter origin, selected burning buff/Collar behavior and once-only global/Act3 counters. Retain deck IDs/upgrades/Parasite gains, relics/counters, potions, gold, keys and purge cost; reset only new act-local state. No second heal after completed2→3. Pin first row35, chest43 and generated future boss50 without offering44/50 from the cutoff.
- [ ] Run `cargo test -p spire-core --lib m8_rooms --locked --offline` to runtime RED at the old Act3Ready/admission gates.
- [ ] Implement separate profile gates and exact captured scheduling above, source gold/card/potion/relic preparation before native settlement commit, actual event preparation before movement and completed-chest cutoff. Extend settlement local counter bookkeeping by actual act, not a bool that routes Act3 loot into a missing Act1Progress. Keep accepted M5Act2Ready/M6floor26/M7Act3Ready including full-belt potion gating unchanged.
- [ ] Rerun GREEN for repeated question/shop/elite visits, owned/exhausted captures, full-belt rejected potion take, skip/take/Sapphire chest actions and pending child before Proceed. Invalid move/overflow is atomic; accepted entry/acquisition/reward fault retains its original owner/resources/latest RNG and never advertises readiness. No latent future offers/encounters become route-selection inputs.

### Task 12: Public evidence, declared profile and ONE milestone review

**Files:** create `tests/m8_act3.rs`, `examples/m8_act3.rs`, `examples/m8_support/mod.rs`, root `docs/milestones/m8-act3.md`; update README/scope/status/AGENTS and the ignored ledger with observed state only.
**Interfaces:** public support exports `start(seed:u64,seen_act3:Vec<&'static str>)->Result<Run,String>`, `prefix(seed:u64,seen_act3:Vec<&'static str>,budget:usize)->Result<Act3Report,String>`, `death(seed:u64,budget:usize)->Result<Act3Report,String>`. Reuse public Step fields; `Act3Report` owns trace, final_decision, final_map, combats and events. Reuse prior public combat priorities; a diagnostic constructor is separate and labelled supplied-resource, never normal-start evidence.

- [ ] Write `m8_normal_start_prefix_or_genuine_death`, `m8_from_start_persistent_act3_event_and_elite`, `m8_public_replay_privacy_and_stale_cutoff`, `m8_budget_is_not_a_game_outcome`. Use seed11 across three valid Act3 histories and seed7 as originally selected; document real outcomes, require at least one prefix completion and an actual persisted event/elite path, not wins for every seed. Check the initial80/80/99/10card/no-key resources. Complete no-key prefix has `(act,floor,terminal,reward_total)==(3,43,None,43)`, alive/no actions/no44; add5 only for actual acquired keys. Replay each accepted action comparing decisions, maps and reward deltas, including natural Death and Parasite changes.
- [ ] Run `cargo test -p spire-core --test m8_act3 --locked --offline` before new rollout completion logic; capture real behavior RED or honestly record already passing reuse. Implement only public-state route/offer/selection handling; bounded caller improvements require a concrete public regression, not repeated forced-win tuning, hidden search or new game values.
- [ ] Add CLI `--scenario prefix|diagnostics|death|all --seed <u64> --budget <0..8192>`, default11/all/8192. Prefix starts at normal Neow; diagnostics exercise real ordinary/lifecycle/all3elite/all3boss mechanics with explicit supplied resources and bounded native actions; Death is a real nonattacking episode. Print true prefix/Death/supplied-diagnostic/unfinished labels. Genuine loss is valid execution, not completion; invalid args, budget0/1 and native faults are nonzero. Diagnostics' forced branches/internal state are never a public caller policy or full-run proof.
- [ ] Document exact roster, captures/events/choices, acquisition journal, public visibility, source grades/closed task facts, finite-capture limits, no-action boundary and all unsupported later branches. Do not label a count/hash/registered key as mechanics proof. Compare fresh `cargo test --workspace --locked --offline` failures against the actual pre-change underlying seven; run public M1–M8 tests together, `cargo test -p spire-core --lib m8_ --locked --offline`, native M6/M7, consumers `-p spire-env -p spire-python`, all other integrations/eight examples, both planned seeds and budgets0/1, and `git diff --check`. Run unchanged reference51 integrity/regeneration checks only as reference evidence. Diagnose timing under the global constraints; no predetermined inflated test-count target.
- [ ] Give ONE fresh independent reviewer its own verified exact-current-source worktree and unique target/jobs2, approved spec/plan, task-local rulings, actual public diff/profile and honest baseline failures. Require all five Review Focus assessments and explicit declined judgments on original-runtime parity, broader roadmap, unsupported handlers/flags, historical drafts, exhaustive profile space and unrelated reference/performance lanes. Fix relevant Critical/Important findings natively with that same reviewer's scoped re-review; no new review swarm.
- [ ] Confirm reviewed product bytes unchanged, rerun the bounded matrix, and only then normally close owned M8 implementation/outcome and activate M9 PLANNING ONLY. Administrative acceptance text stays distinct from reviewed source. No automatic M9 code, new commits/pushes/merges/installs or cleanup.

## Planning self-review and approval gate

- **Spec coverage:** real Act3 ordinary/elite/boss mechanics Tasks1–9; required player/item effects Tasks4/7/10; two persistent events Task10; same resolver/coordinator and same-player Act3 demonstration Tasks11/12. Full normal-start Act3 boss/reward/non-goal completion remains M9, keys/Act4/Heart M10 and broader content/Ascensions later roadmap gates.
- **Review Focus tests:** revival/nonterminal phases Tasks2/7; accepted twelfth-card interruption Task8; permanent gain/census/fault Task4; different cleanup/expiry/partner rules Tasks1/3/6/9; actual prefix source/resource/privacy/predecessor boundaries Tasks10–12. Evidence is explicitly split between diagnostics and normal-start gameplay.
- **Types/ownership:** existing setup/queue/lifecycle/context/Run owners retained; exact Act3 router, permanent-gain journal, progress/event-frame and support signatures defined above. No duplicated deck/inventory, silent unsupported handler, generic law interpreter or every-character prerequisite. Register and implement each named content item together.
- **Source limits:** task-specific unresolved facts have named owners and stop rules; approximate percentages are not exact sampling specifications. Planning grants no new private audit. Existing accepted M7 mechanics require no repeated source qualification. All source/reference files remain unchanged during planning.
- **Proportion/method:** twelve native, separately testable deliverables; one implementation plan and one milestone review, not per-card specs/reviews. Isolated factual assistance is not acceptance review. Preserve the chosen native method and existing controller workspace.

**Approved execution gate:** native `.30.2` owns this plan after explicit user approval. Close named task-local source prerequisites without guessing or expanding private access; keep the separate user-requested Luna stash assessment read-only. Ledger: `.superpowers/sdd/2026-10-23-m8-act3-mechanics/progress.md`.
