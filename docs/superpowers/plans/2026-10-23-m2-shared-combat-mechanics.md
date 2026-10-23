# M2 Shared Combat Mechanics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for native single-owner implementation. Steps use checkbox (`- [ ]`) syntax for tracking. One independent milestone review follows integration, not each task/card.

**Goal:** Extend the accepted authored Ironclad A0 combat path with usable shared powers, damage types, drawing/recycling, exhaustion, explicit selection and real potion consumers needed for initial Act1 integration.

**Architecture:** Keep one `Combat` state, effect queue and resolver. Reuse existing pile/selection/item ownership; complete missing shared calculations and expose thin selection/potion adapters. Three new Ironclad cards and two finite potions supply real producers, rather than injecting effects from the demonstration or implementing whole content families.

**Tech Stack:** Existing Rust workspace and locked/offline dependencies; standard-library example/argument parsing. No installs, Python extension rebuild, new dependency or verification framework.

**Spec:** User-approved `docs/superpowers/specs/2026-10-22-narrow-gym-milestones-design.md`, M2. Accepted M1: `docs/milestones/m1-combat.md`. Planning issue: `spire-dxo.24.1`; outcome: `spire-dxo.24`.

## Global Constraints

- Final scope remains four fully unlocked characters, A0–20, Acts1–4/Heart and normally reachable content. M2 is a named authored Ironclad A0 combat profile, not normal runs or complete pools. M3 owns run persistence/settlement.
- One authoritative Rust simulator; models/training remain external. No second queue, approximate combat, generic rules language, private-source audit or original-game execution.
- Preserve current uncommitted/untracked work, old drafts/failures and legacy combat-v1/schema/RNG behavior. No commits/staging/pushes/merges, dependency changes, service changes or destructive cleanup.
- Native single-owner execution is retained. Every subagent, including a reviewer/investigator, requires its OWN native-created worktree, materialized current public tracked/untracked source, unique absolute `CARGO_TARGET_DIR` and bounded build jobs. Bare HEAD lacks the current simulator. Never dispatch into a live controller/other-agent checkout; preserve worktrees after use.
- Tests must be deterministic, small, valid, bounded and semantic: exact rule values, meaningful resource headroom, no incidental log/Debug/JSON goldens, duplicate per-card rule tests, sleeps/network or timing assertions. Investigate warm focused commands above5s/workspace above30s, separating compilation from test execution; never weaken useful coverage to meet those diagnostic targets.
- Only M2 is executable after this plan is reviewed/approved. No old broad family allocation or speculative later-milestone task forest. One integration owner and one final milestone review.

## Review Focus

1. Artifact consumes one charge for an entire supported debuff application, including an attempted stack; it neither erases the old debuff nor blocks buffs, HP loss or ordinary reductions/removals.
2. Dexterity/Frail modify calculated **card** Block once; direct potion/orb/enemy Block must not acquire card modifiers. Signed Strength and zero-crossing remain compatible with both full-mode power representations.
3. Selection menus contain only captured, eligible, unselected identities; a suspended parent is not repaid, redrawn, finished or resumed twice. Forced/empty branches must not deadlock or invent choices.
4. Potion menu reads and invalid uses cannot consume a slot, HP/energy or RNG. Entry rejection retains the caller's inventory; post-transfer faults retain the mutated combat and accepted-use provenance, not a retryable fresh inventory.
5. NORMAL, THORNS and HP_LOSS retain their distinct modifiers/Block/trigger behavior; lethal self-loss interrupts later energy/draw. Terminal/fault menus and bounded-demo errors must not manufacture wins/losses.

## Declared profile and facts

Retain M1 Strike/Defend/Bash/Clothesline. Reuse Pommel Strike, Shrug It Off and native Slimed production. Add **Bloodletting, Burning Pact, Disarm**; examples use base cards, with cheap base/upgraded producer cases for their differing payloads. Reuse Jaw Worm, Green Louse and Gremlin Nob; add the already-implemented A0 **medium Spike Slime** as the Frail/Slimed producer. No new enemy AI or other-character roster is required.

Use existing Strength/Block/Fire/Swift/Weak/Fear/Energy potion handlers where exercised; add **Dexterity Potion** and **Ancient Potion**. Sacred Bark is the only needed modifier-relic fixture. Public potion entry/actions are combat-only; normal room acquisition and run-inventory persistence remain M3/later work.

Keep each authored A0 inventory at at most3 slots; use separate small fixtures
for the different comparisons, not an oversized inventory or mid-combat resource
injection. Stat/Block comparisons must account for all powers and actual incoming
intent; do not let Nob Enrage or an HP/Block cap hide the expected rule.

Qualified existing authored references, not a new audit or original-runtime evidence:

| Producer/rule | Required literal behavior | Existing reference |
|---|---|---|
| Bloodletting | Cost0, self HP_LOSS3, then energy2/3; not exhaust | `docs/research/2026-10-19-act1-content-red-colorless.md:111` |
| Burning Pact | Cost1, exhaust one remaining hand instance then draw2/3; empty still draws, singleton forced, multiple require choice; parent discards | Same file:112 |
| Disarm | Cost1, apply enemy Strength−2/−3, exhaust itself; Artifact can prevent that application | Same file:116 |
| Card Block | B5+Dexterity2=7; Frail-only B5=3; combined B5/Dex2/Frail gives5, not double scaling | Same file:588; card Block versus direct sources:119 and `2026-10-19-act1-content-economy-potions.md:355` |
| Dexterity/Ancient potions | Dexterity2/4 and Artifact1/2 (ordinary/Sacred Bark); both untargeted combat uses | Potion reference:356,373 |
| Signed Strength | Clamp constructor/stack/reduce result to[−999,999]; do not infer this game-rule clamp for unrelated powers | `2026-10-19-act1-complete-catalog-numeric.md:1269` |
| Artifact ordering | Shared ordinary application gate before stack/insert; one charge per prevented debuff application | `2026-10-19-act1-transitive-execution-closure.md:93` |

This covers the supported ordinary power pipeline, not every future immunity, callback or direct-application exception. Unimplemented Ginger/Turnip/Champion Belt, temporary-stat powers, Poison ticks/cards and other families are not silently accepted. Future producers must reuse the shared checkpoint and add their actual exceptions.

## Current reuse/gap map (read-only inspection,2026-10-23)

| Area | Reuse | Concrete gap |
|---|---|---|
| Attack damage | Full damage helper already accepts signed Strength and applies Weak/Vulnerable | Full power storage/effect validation reject negative amounts; stacking can leave zero objects |
| Card Block | One raw `GainBlock` commit; existing card handlers | No card Dexterity/Frail calculation; applying it globally would incorrectly scale potion/Frost/enemy gains |
| Powers | Typed full powers, source flags, shared legacy-compatible stacking | Artifact has no prevention body; Frail has no expiry; positive-only validation is insufficient |
| Damage types | NORMAL and typed Fire THORNS; shared HP/death/fault handling | No typed HP_LOSS payload; it must bypass Block without duplicating death handling |
| Draw/piles | Owned instances, hand cap10, transactional shuffle/recycle, exhaust destination | Selection-to-exhaust currently moves ownership without a `CardExhausted` notification; no standalone forced hand-exhaust effect |
| Selection | Captured candidates, incremental picks, disposition and automatic parent resume | `legal_actions_full` rejects selection screens; public selection lacks required/selected/disposition values |
| Potions | One combat-owned `CombatItemContext`, pure use preparation, validated queued effects, retained failed entry | Entry/use are crate-private; no public potion action/menu/inventory view; don't introduce another inventory |

No product tests were run for this planning turn. M1's recorded core335/8 is historical evidence for M2 until execution captures a fresh baseline. Six old Poison/Catalyst/Artifact failures and BlueSlaver/Fungi setup remain outside M2; the old six-potion consumer fixture becomes a directly relevant consumer check, not a reason to finish its entire old lane.

## Files and ownership

One native implementer owns the complete integration across these paths; there are no overlapping worker/shared-root requests.

| Paths | Responsibility |
|---|---|
| `combat/powers.rs` (new), `combat/{effects,engine,hooks,rules,model,mod}.rs` | Small finite shared power operations, card Block helper, signed validation, Frail timing and HP_LOSS dispatch |
| `combat/{command,selection,view,engine,piles}.rs` | Public selection/potion actions, pure preflights, actual owned movement/notifications and inventory/selection observations |
| `combat/cards/{mod,ironclad,silent}.rs`, `combat/card_state.rs` | Apply shared card Block to existing producers; three new native card payloads/disposition |
| `content/ids/{cards,items}.rs`, `content/{cards,items}.rs`, `items/{mod,potions}.rs` | Real producer identities/definitions; existing item ownership made usable without a second inventory |
| `combat/tests/{mod,shared_mechanics,selection,differential}.rs`, `items/tests.rs` | Focused rule/ownership/fault regressions and exhaustive diagnostic matches, not new golden-output testing |
| `tests/m2_combat.rs`, `examples/m2_support/mod.rs`, `examples/m2_combat.rs` | Small public fixtures and bounded caller; no gameplay rules in the helper |
| `tests/m1_combat.rs`, `docs/milestones/{m1-combat,m2-combat}.md`, `README.md`, status/guidance/ledger | Preserve M1 behavior; explicitly extend its former selection rejection; record qualified evidence |

All crate paths above are under `crates/spire-core/src/` except `tests/` and `examples/`, which are under `crates/spire-core/`. Do not restructure unrelated large modules or build a scenario framework.

### Task 1: Complete shared power, Block and damage-type semantics

**Interfaces:** reuse `State`, `Effect`, `FullPowerId`, existing actor-owned power lanes and resolver. Add private `powers::apply_effect(state: &mut State, effect: &Effect) -> Result<(), InputError>` for full-mode apply/reduce/remove operations; legacy-v1 keeps its existing path. Add `rules::full_card_block(base: i32, dexterity: i32, frail: bool) -> Result<i32, InputError>`, card preparation helper `modified_block(state: &State, base: i32) -> Result<i32, RunError>`, and `DamagePayload::HpLoss(i32)`.

- [ ] After plan approval, verify context/ping on the existing shared board: database`spire`, project`a61e792f-9730-47c0-b262-7dac036d6bf5`; use only `/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab ... --json`, actor`opencode-session-recovery-20261021-v1`. Create/reuse ONE executable implementation child under M2 `.24` (metadata `roadmap_version=2026-10-22-v2`, `milestone_number=2`, `role=implementation`), with actual accepted M1 `.23` prerequisite, plan/profile/interfaces/tests/commands in its own description, and no dependency on closing its own parent or complete old families. Discover/record its actual returned ID and claim normally. Do not use the planning issue `.24.1` as a product claim.
- [ ] Capture actual current `cargo check -p spire-core --lib --locked --offline` and one `cargo test --workspace --locked --offline` before source edits, preserving source/draft state and exact failures. If Cargo stops before consumers, run `cargo test -p spire-env -p spire-python --locked --offline` directly once. Record build/test time separately; do not infer M2 baseline from old counts.
- [ ] Add/register `combat/tests/shared_mechanics.rs`. Small rule tables must pin: Strike6+Strength2=8; Chomp11 with Strength−2=9; Strength+2 followed by−2 removes the zero object; Strength clamps−999/999; card Block `(base,dex,frail)=(5,0,false)→5,(5,2,false)→7,(5,0,true)→3,(5,2,true)→5,(5,−2,true)→2,(5,−9,false)→0`. Direct Block12 stays12 with Dex2/Frail. Artifact1 blocks the whole Weak3 application and removes its charge, preserves pre-existing Weak2 on a prevented stack, does not block Strength+2, blocks Strength−2 once, and consumes independently for two different debuff applications. Read/reject comparisons use detached state equality, not stored goldens.
- [ ] Run `cargo test -p spire-core --lib combat::tests::shared_mechanics --locked --offline`. Expected initial RED at actual missing signed/prevention/scaling behavior (a missing new helper is compile evidence only). Implement the small finite shared operations; re-run only this target to GREEN. Keep one application checkpoint across full-mode legacy and full representations, with no duplicated logical powers. Classify positive Weak/Vulnerable/Frail and negative Strength/Dexterity as the supported debuffs; buffs, reductions/removals and HP loss do not spend Artifact. Existing source/target identity remains attached to the work.
- [ ] Permit signed full-mode Strength/Dexterity with checked arithmetic and zero-removal semantics. Strength has the qualified[−999,999] game clamp. Dexterity keeps an explicit supported native signed surface[−999,999] and errors outside it, not an invented game-rule clamp. Initial/ready zero objects are absent; runtime validation recognizes a transient zero only with its matching queued removal. Apply the same full-mode rules to an existing legacy-represented stat, while preserving original legacy-v1 validation/calculation behavior. Frail uses the M1 Weak source-sensitive grace/expiry pattern, never permanent by omission.
- [ ] Calculate card Block once at preparation: add signed Dexterity, multiply by binary32 `0.75` iff Frail, final floor/nonnegative clamp. Route existing native card Block producers in `cards/{mod,ironclad,silent}.rs` through it, not `GainBlock` globally. Full-mode raw Block addition accepts nonnegative i32 gains using a wide intermediate and caps actual Block at999; do not reject a legitimate calculated gain merely because it exceeds the old1007 input bound. Legacy gain bounds remain unchanged. Add one small producer-wiring table (Defend/Iron Wave/Shrug), not per-card copies of the modifier tests.
- [ ] Add HP_LOSS to the existing damage validation/commit/death path. It ignores Block, Strength/Weak/Vulnerable and NORMAL-only hit triggers, leaves current Block intact and still emits real damage/death/terminal work. Pin HP20/Block5/loss3→HP17/Block5 and lethal HP3/Block5→Death with no later energy child executing. Keep existing typed THORNS behavior (20 against Block7→13HP loss, unaffected by player Weak/Strength or enemy Vulnerable) and raw NORMAL absorption distinct. Update exhaustive diagnostic matches without freezing debug logs.
- [ ] Run the shared-mechanics target again; expected all its rules pass. Run affected existing `combat::tests::selection` and `combat::tests::full_runtime` filters once after shared changes compile; preserve/report any unrelated baseline failures rather than widening into Poison/enemy completion.

### Task 2: Make real selection and potion actions publicly usable

**Interfaces:** extend `CombatAction` with `UsePotion { slot: usize, target: Option<TargetId> }`. Keep `PlayCard`, `SelectCard`, `EndTurn` and legacy `Action` unchanged. Extend the existing `legal_actions_full`/`step_full` pair, not another action engine. Publicly export the existing `CombatEntryFailure` and `items::CombatItemContext`, with its current `new(Vec<RelicId>, Vec<Option<PotionId>>) -> Self` constructor; retain private mutable fields.

Expose the existing lossless `Combat::enter_with_item_context(entry: CombatEntry, item_context: &mut Option<CombatItemContext>) -> Result<Combat, CombatEntryFailure>`. It leaves the caller's Option untouched on rejection, takes it once on accepted entry, and retains the mutated Combat on post-transfer faults. No lossy new entry wrapper. Add `CombatInventoryView { relics: Vec<RelicId>, potions: Vec<Option<PotionId>> }` to `CombatPublic` as `inventory: Option<CombatInventoryView>`; this is a detached public projection, never another mutable inventory. Extend `SelectionView` with `required: usize`, `selected: Vec<CardInstanceId>`, `disposition: SelectionDisposition`, preserving captured candidate identities.

- [ ] Write public tests in `tests/m2_combat.rs` before the adapter changes: inventory entry/mutation ownership, initial potion slots, read-only menus, and selection-only menus using the existing Survivor handler as a negative/compatibility fixture (not an added Silent profile). Run `cargo test -p spire-core --test m2_combat --locked --offline`; initially expect the missing public contract/selection refusal, not invented behavior RED.
- [ ] Extract pure `Combat::preflight_full_selection(&self, instance: u64) -> Result<(), RunError>` from the existing selector and call it from both enumeration and execution. During Selection, list only eligible unselected `SelectCard` actions, with no PlayCard/EndTurn/potion actions. Preserve partial-pick order, captured ownership, hand-cap checks and one parent continuation. Extend existing two-pick tests to check the menu excludes the first pick; invalid/stale/duplicate picks and reads preserve snapshot/RNG. Do not auto-select all candidates globally: required-all selections can still have meaningful ordered choices.
- [ ] Share pure potion preparation between menu and use: `items::prepare_combat_potion(combat: &Combat, slot: usize, target: Option<EnemyHandle>, we_meet_again: bool) -> Result<PotionUseEffects, RunError>`, reusing current native room/HP/live-target facts and existing effect/slot preparation. Stable TargetIds are resolved by the combat adapter, not interpreted as enemy handles. Invalid candidates are excluded; unsupported stored content returns an explicit error, not a smaller silently supported menu. Public manual combat use supplies `we_meet_again=false`; it does not invent room/event state.
- [ ] Expose lossless item entry with pre-transfer validation of unknown/unimplemented inventory IDs; never accept an inert item by its definition-only flag. The public failure's Debug/display must not dump retained Combat/RNG. Keep inventory fields mutable only in the existing owner. Enumerate potion choices only at ready live decisions, validating the same effect batch as execution before slot commit. Record the accepted `UsePotion` as attempted-action provenance; preflight errors leave all state/slots unchanged, and post-commit faults do not refund/retry accepted effects.
- [ ] Pin one slot/target/read table using already implemented Strength, Fire, Block and Swift consumers: empty/out-of-range slot, foreign/dead target, unwanted target for self-use, selection/terminal/fault use; no energy cost or slot reuse after acceptance. Test accepted-use fault retention with a valid named internal fault-state fixture, not impossible public HP/setup values. Initial rejected entry retains Some(items); a post-startup fault retains its actual mutated combat and consumed resources. Compare failure rendering for equivalent faults with differing hidden stream keys to prove privacy, not exact Debug wording/goldens. Existing failure-ownership tests count—extend only missing public coverage.
- [ ] Update the M1 selection-negative case to the intentional M2 extension: menu offers only real picks and reads remain nonmutating. Keep every other M1 rule/terminal/replay assertion. Run `cargo test -p spire-core --test m1_combat --test m2_combat --locked --offline` and the affected selection filter; expected all M1 assertions and the new public-contract tests pass. No full snapshot restoration is added.

### Task 3: Compose real Ironclad/potion producers and a bounded demonstration

**Interfaces:** add typed `CardId::{Bloodletting,BurningPact,Disarm}`, `PotionId::{DexterityPotion,AncientPotion}`, and `EffectKind::ExhaustHandCard { instance: CardInstanceId }`. Add private `piles::exhaust_owned_card(state: &mut State, card: OwnedCard) -> Result<(), InputError>` for an already detached owned card, preserving one destination/notification; callers remove it from its original zone exactly once. Reuse owned piles/selection/Draw and the shared power/damage/Block interfaces from Tasks1–2. Register identities alongside actual handlers; do not make a pool complete by adding these definitions.

- [ ] Add behavior tests first for the literal producer table above, then run the focused M2 target for actual handler/ordering RED. Implement Bloodletting as HP_LOSS3 before GainEnergy2/3; widen the full-mode GainEnergy payload surface to1..=4 for this producer and existing Bark Energy Potion4, retaining total-energy≤999 and unchanged max-energy. Pin base/Bark Energy Potion at initial energy3 as `(energy,max_energy)=(5,3)/(7,3)`, and reject an overflowing use without spending its slot. Disarm uses one ordinary negative Strength application and its real exhaust-on-use state in the permanent constructor, not test-only manual flags.
- [ ] Implement Burning Pact without a new chooser: prepare remaining hand candidates excluding the parent. Empty emits only Draw2/3; singleton emits a queued `ExhaustHandCard` then Draw; multiple emits the existing captured `SelectCards(required=1, Exhaust)` then Draw. Do not auto-choose a multi-option screen or discard the parent before draw. Introduce one native owned-exhaust movement/notification helper reused by forced exhaustion, selection disposition and exhaust-on-finish; each actual exhaustion emits CardExhausted once before continuation, not a duplicated debug event list.
- [ ] Pin the six-card public sequence: two Strikes, two Defends, Burning Pact, Pommel Strike; draw all6 initially. Play one Defend and one Strike (discard2, energy1), then Burning Pact (energy0); exhaust the **other Defend identity** via the menu, draw the two discarded cards through ordinary recycle, and finish the parent once. Assert hand4/exhaust1/discard1, selected identity absent from later recycle, retained same-kind sibling correct, no extra payment/draw, invalid repeat selection nonmutation. Separate three-card/ four-card variants prove empty/singleton branches still draw and do not expose a fake choice. Include hand-cap10 and insufficient draw supply as one small existing-rule boundary table, not duplicate per-card scenarios.
- [ ] Add Dexterity/Ancient potion bodies to the same finite native potion path:2/4 Dexterity and1/2 Artifact, with existing Bark/target/slot/gate behavior. Public cases: Strength Potion→Strike8; Dexterity Potion→Defend7; real Spike Lick Frail1→Defend3, or5 with Dexterity2; Ancient Potion blocks one real SpitWeb/Frail application, consumes its charge, and does not cleanse an existing debuff. Use valid fixed seeded AI histories with no reapplication while testing expiry, documenting the chosen fixture once—not seed searches or probability assertions in tests. Frail remains1 at its first affected player decision and expires after the next full round without reapplication.
- [ ] Preserve the old eight-failure history honestly. The old `six_combat_potions_use_real_block_energy_draw_strength_and_debuff_consumers` test has an invalid Chomp entry in the Louse-only `enemy_damage_bases` override map (`items/tests.rs:573` at planning time). Correct that fixture by removing the invalid override, leaving native canonical Chomp11 and every damage/Block/slot assertion intact; then execute this directly relevant consumer once. Do not populate canonical bases merely to satisfy the fixture, weaken its rules, skip it or finish unrelated Poison/Slaver/Fungi work.
- [ ] Create `examples/m2_support/mod.rs` for these small typed authored fixtures/caller helpers and `examples/m2_combat.rs`: `--scenario all|powers|selection|damage`, `--seed <u64>` default7. Each scenario has at most64 accepted public-menu actions; exhausted budget is an explicit nonzero error, not Death. Output public observations/inventory/selection/menu/actions/outcomes only. Demonstrate native Strength/Dexterity/Frail/Artifact, selected exhaustion/recycle, and NORMAL/THORNS/HP_LOSS including actual lethal self-loss. No state/effect injection after entry and no gameplay rule calculation in the caller.
- [ ] Run `cargo test -p spire-core --test m1_combat --test m2_combat --locked --offline` and `cargo run -p spire-core --example m2_combat --locked --offline -- --scenario all --seed 7`. Expected exact semantic cases pass and the demo completes all named checkpoints without faults/timeouts/hidden output. Also retain the M1 example command. Existing passing mechanics need no rewrite just to manufacture RED.

### Task 4: Verify and accept the integrated M2 capability

**Consumes:** the complete native candidate from Tasks1–3; public M1/M2 tests/examples and actual pre-change evidence. Produces qualified M2 acceptance plus M3's just-in-time planning handoff, not full-run acceptance.

- [ ] Run `cargo test --workspace --locked --offline` once on the integrated candidate and compare exact executed targets/failure names/causes to the actual Task1 baseline. If it stops early, directly run accepted consumers and `cargo test -p spire-core --test run_content --test run_items --test run_map_api --test trace_protocol --test m1_combat --test m2_combat --locked --offline`. Record build/test/wall time separately. Re-run only affected targets during fixes; no unchanged compile storms. All M1/M2 and accepted-behavior regressions must be resolved. An unchanged unrelated draft failure is disclosed, never a pass; a formerly unseen failure is not called pre-existing by assumption.
- [ ] Write `docs/milestones/m2-combat.md`: exact commands/profile, literal representative output, public selection/potion entry/use, inventory/fault ownership, source-dependent timing and limitations. Link README/status and update the formerly selection-limited M1 API note without rewriting its historical evidence. Keep source-grounded/reference-informed versus original-runtime-validated claims separate. `git diff --check` must be clean.
- [ ] Dispatch one fresh-context independent milestone reviewer in its OWN materialized candidate worktree/unique build directory. Supply this plan, approved roadmap, actual M2 delta/baseline, all five Review Focus items and existing command evidence. No broad repeat builds solely for the reviewer; reproduce concrete suspicions only. Fix substantive findings under the same owner and rerun covering checks; no per-card review/review swarm.
- [ ] After unchanged reviewed-source verification and qualified evidence, close only the real M2 implementation child/outcome under normal ownership. Activate M3 `spire-dxo.25` and create its one bounded planning issue for consecutive real fights/resource settlement. Keep old umbrellas incomplete and M4–36 gated. No commits/cleanup or later product start is implied.

## Execution handoff and self-review

**Preserved method:** native single-owner execution in the existing isolated checkout, then one independent milestone review in a dedicated materialized worktree. No implementer subagents or new source worktrees are needed for planning. The actual M2 implementation issue is created/claimed only after this plan's review/start decision; `.24.1` is planning, not a product claim.

Self-review: each M2 mechanic has a shared-rule task, real producer and public executable consumer; selection/potion adapters reuse actual ownership rather than fake actions; parent exhaustion/empty/singleton/zero stats/lethal HP-loss and non-card Block exceptions are explicit. Task1→2→3 interfaces are named; Task4 compares actual baselines and qualifies unchanged old failures. There is no per-card spec/review, full-family prerequisite, invented complete pool, new verifier or commit/install step.

**Approval status:** user-approved2026-10-23 ("I approve"); native execution authorized. Actual implementation child `spire-dxo.24.2` is claimed by the allocated native owner; planning `.24.1` is closed as approved planning only. User also permits GPT-6 Luna fresh eyes concurrently in isolated worktrees; this does not authorize overlapping source ownership or extra per-card review gates.

**Execution outcome2026-10-23:** completed/accepted at declared M2 scope;
`.24.2`/`.24` normally closed after one isolated independent review with no
findings and unchanged-source/fresh focused verification. Exact evidence and
preserved seven draft failures: `docs/milestones/m2-combat.md` and this plan's
execution ledger. M3 planning-only `.25.1` is open; no later product start implied.
