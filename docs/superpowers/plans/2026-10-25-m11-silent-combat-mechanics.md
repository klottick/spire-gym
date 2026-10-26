# M11 Silent Combat Mechanics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:executing-plans` for native controller-owned integration, useful isolated GPT-6 Luna MAX assistance, and ONE fresh independent final milestone review. Steps use checkbox (`- [ ]`) syntax. User approved this actual written plan on 2026-10-25 ("I approve"); native implementation `spire-dxo.33.2` now owns execution. Per-task review swarms, automatic commits/publication, installs, and cleanup are not part of approval.

**Goal:** Deliver actual Silent A0 Poison, discard, Shiv, retain, and representative power/trigger/copy mechanics through the existing Rust resolver, with canonical Silent starting resources, a real owned combat bridge, and a runnable native demonstration.

**Architecture:** Extend the current Silent payload dispatcher and existing card, pile, effect, power, selection, item, and Run owners. Keep one combat queue and one permanent-card/inventory census. Extend the existing authored-combat constructor for a narrowly validated Silent starter state; do not introduce another Run/session type or a Silent Gym profile.

**Tech Stack:** Existing Rust workspace and Cargo tests; existing offline reference library. No dependencies, Python rebuild, learner, or verification platform.

**Spec:** The approved [36-milestone roadmap](../specs/2026-10-22-narrow-gym-milestones-design.md), especially “Silent: milestones11–15, A0”; [accepted M10 profile](../../milestones/m10-heart.md). These are existing design authorities, not a new design-approval cycle.

**Planning baseline:** M10 PR [46](https://github.com/klottick/spire-gym/pull/46) merged at `36fbdc6310945caa194f2aa806c37769275228c3`, with the same public source tree as reviewed/published `5b293e7`. Local `main` matches that merge. Planning-only Bead `spire-dxo.33.1`; isolated branch `plan/m11-silent-combat-20261025`. Planning used three Luna 6 MAX assistants for card facts, runtime facts, and an initial draft; the controller reconciled factual disagreements and self-reviewed this plan. No M11 product code, build, or test was run while planning.

## Global Constraints

- M11 is **Silent, A0, representative combat mechanics**. M12 owns from-start Act1 progression/Neow consequences/rewards/boss alternatives/transition; M13–15 extend the ordinary character content and acts. Complete ordinary Silent cards/exclusive items/events remain a phase requirement by M15, not silently accepted here.
- Canonical A0 Silent resources are **70/70 HP, 99 gold, five StrikeGreen, five DefendGreen, Survivor, Neutralize, RingOfTheSnake, three empty potion slots, no keys**. Source these from existing `Run::new`/Catalog definitions. An explicitly authored combat populated from those resources is not a normal-start Neow/map episode.
- Existing Ring of the Snake `CombatStarted -> Draw(2)` is already implemented; base draw5 plus that hook is7. Reuse and characterize it. Metadata `definition_only` alone does not establish a missing runtime handler.
- Keep one authoritative Rust resolver, current instance IDs, source-attributed work, item custody, and exit census. Invalid/stale requests are preflight-atomic; accepted faults retain committed work, pending parents/children, RNG, and the first fault without rollback/retry or fabricated Death.
- Genuine gameplay Death remains valid. No policy tuning, winning-seed search, injected resources in ordinary initialization, edited enemy values, forced win, or win-rate gate. Supplied mechanic fixtures remain labelled diagnostics.
- Preserve [RNG-001/TIME-001](../../intentional-deviations.md), hidden information, existing M1–M10 cutoffs and workloads, M9 observation/replay-v1 and rules-v2, and M10 observation/replay-v2. Append new native IDs without changing existing semantic indices. No Silent numeric schema, Python profile, extension refresh, vector interface, or original-runtime/Java parity claim.
- Previous verification is historical until execution reruns it: workspace **1,402 passed / exact7 failures**, core1,149/7, native M10 453/0, Python251/0. Six Poison-related failures are directly within M11; the Blue Slaver/two Fungi encounter failure remains separately visible. Never delete/skip/weaken a historical test to improve the count.
- Use existing sources at their recorded grades. Previous M6/M8/M10 private-source permissions do **not** grant M11 archive access. No broad audit, new tools, installs, environment sync, blanket cleanup, commits, push, or merge follows from this plan. A real new source blocker must name its exact missing member/behavior and obtain separately bounded access if needed.

## Declared representative roster

The following is the finite implementation slice. Base/upgraded literals are existing-reference inputs; Task1 must rule any source conflict before they become new oracles. They are not newly executed mechanics proof.

| Producer | Base -> upgraded behavior and owner |
|---|---|
| Deadly Poison | Cost1; one enemy Poison5 ->7 application. |
| Poisoned Stab | Cost1; damage6 ->8, then Poison3 ->4 to its qualified target. |
| Catalyst | Cost1, Exhaust; if Poison exists, add its current amount -> twice its current amount through a separate application (total doubles -> triples before other modifiers). No Poison means no application. |
| Noxious Fumes | Cost1 Power; amount2 ->3, applying Poison separately to eligible enemies at player **post-draw** start-turn, not their Poison-tick checkpoint. |
| Survivor / Dagger Throw | Reuse existing real selected discard. Survivor Block8 ->11 then discard1; Dagger Throw damage9 ->12, draw1, then discard1. |
| Reflex / Tactician | Ordinary use is unplayable. Manual discard queues draw2 ->3 at bottom / energy1 ->2 at top respectively; end-turn cleanup does not grant those manual-discard effects. |
| Cloak and Dagger / Blade Dance / Shiv | Reuse Cloak's Block6 and1 ->2 ordinary Shivs. Blade Dance cost1 creates3 ->4 ordinary Shivs. Shiv cost0, damage4 ->6 by **its own** upgrade, Exhaust; upgrading the generator does not make Shiv+. |
| Accuracy | Cost1 Power, amount4 ->6; qualified Shiv-identity base-damage rewrite and new-Shiv construction, not a bonus to every zero-cost Attack. |
| Well-Laid Plans | Cost1 Power, retain limit1 ->2. Optional end-turn selection; selected non-Ethereal cards receive temporary retain. Ethereal may be selected but does not gain immunity. |
| Nightmare | Cost3 ->2, Exhaust; capture/reset one selected remaining-hand card template, then create **three in both versions** at the next qualified start-turn. Independent captured templates do not merge. |
| Ring of the Snake / Snecko Skull | Reuse Ring's existing opening Draw2. Add only Snecko Skull as the new item slice: +1 per eligible Poison application, not per Poison point/tick; Artifact-blocked applications create no Poison. |

Existing starter/shared cards and Backflip, Deflect, Quick Slash remain supported and regression-tested. Other green cards, Silent potions, remaining exclusive relics, full offer/acquisition pools, shared-event closure, and ordinary Act1/Heart routes are not M11 acceptance claims. Snecko Skull is tested as a genuinely owned item in an explicitly supplied native context; this is not proof of ordinary acquisition. Do not invent a Silent-exclusive event.

## Review Focus

1. **Poison application and lethal actor timing:** Artifact/Snecko Skull/Catalyst and enemy-start HP loss retain one correct application/tick owner; Task2 `m11_silent_poison_application_and_tick_order_are_source_owned`.
2. **Manual discard versus cleanup:** exact detached cards, callbacks, and parent continuation execute once even with draw/energy reactions and faults; Task3 `m11_silent_discard_triggers_follow_the_committed_owner`.
3. **Shiv identity and mutation:** generated/existing Shivs receive only the qualified Accuracy rewrite, preserve upgrades/overflow/exhaust and never become permanent gains; Task4 `m11_silent_shiv_generation_and_accuracy_preserve_instance_ownership`.
4. **Optional retain and turn ending:** zero-confirm, stale choices, Ethereal, and accepted selection faults preserve actual instances and the end-turn parent; Task5 `m11_silent_retain_preserves_selected_instances_at_end_turn`.
5. **Delayed copy capture and delivery:** templates are snapshots with distinct future-card identities, independent owners/cursors, and no duplicate delivery after faults; Task6 `m11_silent_nightmare_copies_are_owned_and_delivered_once`.

## Files, interfaces, and source gates

All abbreviated source paths below are relative to **`crates/spire-core/src/`**; examples are relative to `crates/spire-core/`. This keeps paths unambiguous without repeating their prefix. New test modules must be registered in `combat/tests/mod.rs` or `run/tests/mod.rs` in the task introducing them.

- `combat/cards/silent.rs` keeps its existing `payload(&State, &LiveCard, Option<u64>) -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, RunError>` dispatcher. Add focused `silent/{poison,discard,shiv,retain,copy}.rs` leaves as their tasks need them.
- `combat/cards/mod.rs`, `card_state.rs`, `model.rs`, `effects.rs`, `powers.rs`, `hooks.rs`, `piles.rs`, `turns.rs`, and `selection.rs` remain the shared preparation, instance, queue, lifecycle, movement, and suspended-choice owners. Do not add a second callback/pile scheduler.
- `content/ids/{cards,items}.rs`, `content/{cards,items}.rs`, `items/{mod,relics}.rs` own typed definitions, validation, and item callbacks. Promote only handlers actually implemented. Registry keys and reference tests do not prove behavior.
- Existing public full-runtime methods are `Combat::enter_with_item_context`, `public_full`, `legal_actions_full`, `step_full`, `exit`; existing Run methods are `Run::new`, `from_authored_combat`, `enter_combat`, and `step`. Keep their signatures. New narrow operation variants/state fields in Tasks3–6 are **proposed additions** to these owners, not pre-existing APIs.
- Read [data index](../../../data/README.md), the [card](../../../data/cards/README.md), [relic](../../../data/relics/README.md), and [power](../../../data/powers/README.md) guides, exact records/citations/uncertainties, and [Silent research](../../research/2026-10-19-act1-content-silent.md) §§3–8,10. The research is an author draft with explicit composition limits and unverified original runtime. The ignored planning facts are navigation aids; this plan and those public sources carry the requirements.

| Task-local fact to close before its oracle/body | Bounded evidence and unresolved checkpoint |
|---|---|
| Poison, Artifact, Snecko Skull, Catalyst | Silent research §§3–4,7; card/power records and `relic.snecko_skull`; actual `ApplyPowerAction`/Catalyst/Snecko Skull callback facts if the existing record leaves application/Artifact order unsettled. Distinguish damage, decrement, and new application. |
| Discard reactions | Research §6: chosen card moves, its manual callback, then discard notification; Reflex bottom/Tactician top. End-turn suppression and fixed original action membership must stay distinct. No need to audit Unload, Calculated Gamble, or Eviscerate to deliver this roster. |
| Accuracy/Shiv | Research §§3–4,8: constructor, stack, and draw/discard updates; qualify existing pile traversal and generated upgrade/copy fields without assuming a generic Strength bonus. |
| Well-Laid Plans/Ethereal | Research §6; actual retain-selection callback and existing native end-turn/Ethereal owner. Shared Establishment/Pyramid/Equilibrium compositions remain outside the captured roster; unsupported interactions must not be advertised as implemented. |
| Nightmare | Research §§5,7–8: template capture/reset, three copies, power scheduling/removal and independent captures. Close creation-versus-ordinary-draw ordering before pinning that ordering; do not guess a global turn-start rule. |

If a load-bearing fact cannot be closed from these recorded sources, stop only that behavior and report the named blocker. No successful placeholder or widened source permission is acceptable. Record semantic rulings and any corrected oracle in the ignored execution ledger; preserve prior evidence and attribution.

## Tasks

### Task 1: Establish the actual baseline and finite fact rulings

**Files:** Read this plan, accepted M10 profile, selected records/research, existing native tests. Write only the ignored M11 execution ledger; no product change.

**Interfaces:** Consumes merged M10 source and the source-gate table; produces actual baseline results, exact failure identities, and a ruled fact for each selected behavior.

- [x] After user approval, verify the branch/source snapshot and run `cargo test --workspace --locked --offline --no-fail-fast` once before editing. Record actual results, expected historical seven identities, any unexpected failure, build/runtime separation, and a source manifest. Do not silently attribute a new failure to the old baseline.
- [x] Preserve these six `combat::tests::full_cards` identities: `silent_deadly_poison_applies_base_and_upgraded_amount_for_one_energy`, `silent_poisoned_stab_resolves_attack_before_base_or_upgraded_poison`, `catalyst_permanent_card_state_carries_its_exhaust_flag`, `catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop`, `poison_ticks_at_afflicted_enemy_start_ignoring_block_then_decrements_and_can_kill`, `artifact_prevents_one_poison_application_without_erasing_existing_poison`. The seventh is `combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`.
- [x] Close each selected source gate before new assertions. A source-correct existing behavior may characterize GREEN; do not manufacture RED. A genuine missing handler/order/bridge must have behavioral RED before its minimal fix. If an old assertion is source-inaccurate, preserve it and document the specific correction before changing it.
- [x] Use existing absolute targets under `/home`, locked/offline dependencies, jobs2 and one test thread. Reuse a native M11 target; use a disposable own scratch directory under `/home` and remove only that command's inactive scratch. No new `/tmp` build targets, installs, or broad cleanup.

### Task 2: Poison producers, application, and actor-start lifecycle

**Files:** Modify `combat/cards/silent.rs`, new `combat/cards/silent/poison.rs`, `combat/{powers,effects,turns,hooks}.rs`, card definitions/IDs as needed, `items/{mod,relics}.rs` and item definitions/IDs for Snecko Skull; create `combat/tests/m11_silent_poison.rs`. Preserve the existing six tests in `combat/tests/full_cards.rs`.

**Interfaces:** Existing Silent payloads emit ordered `EffectKind` work; `ApplyFullPower`/Artifact remains the application owner, enemy turn scheduling owns the Poison tick, and `CombatItemContext` owns Snecko Skull. Add no separate Poison damage engine or inventory.

- [x] Write `m11_silent_poison_application_and_tick_order_are_source_owned`: base/upgraded Deadly Poison5/7; Poisoned Stab damage6/8 before Poison3/4; Catalyst adds current/2×current through an application, exhausts, and is a no-application no-op on absence. Check legal cost1, targeting, power classification as a debuff, and the permanent Catalyst Exhaust flag.
- [x] Add Artifact tests that consume exactly one prevention per application without erasing existing Poison; Snecko Skull adds1 once to eligible applications, including the qualified Catalyst path, never to tick/decay or an absent-Catalyst no-op. Reject any test oracle that silently multiplies the stored power directly.
- [x] Add enemy-start cases: Poison HP_LOSS bypasses Block, damage resolves before decrement/removal, zero clears, and lethal tick prevents that enemy's subsequent move without suppressing unrelated live actors. Include an accepted fault after committed work and exact retained power/queue state.
- [x] Noxious Fumes installs2/3 and applies separately at player post-draw start-turn; assert that this is distinct from enemy-start ticks and that the actual Power card has one owned played-power disposition/exit record. Use the source-qualified checkpoint, adding a narrow hook variant only if existing hooks cannot represent it faithfully.
- [x] Run `cargo test -p spire-core --lib --locked --offline m11_silent_poison` and the six existing tests to actual RED, implement the minimal native behavior, then rerun to GREEN. Run `full_cards`; do not delete/rename/skip the six historical tests. Missing enum compilation alone is not behavioral RED.

### Task 3: Explicit discard and real Reflex/Tactician reactions

**Files:** Modify `combat/cards/silent.rs`, new `combat/cards/silent/discard.rs`, shared `combat/{cards/mod,hooks,piles,selection,model}.rs` only where needed, card IDs/definitions; create `combat/tests/m11_silent_discard.rs`.

**Interfaces:** Reuse Survivor/Dagger Throw, current hand-selection continuation, `Hook::CardDiscarded`, source attribution, and the same pending effect queue. Reflex/Tactician are proposed new typed cards/listeners, not a new discard subsystem.

- [x] Write `m11_silent_discard_triggers_follow_the_committed_owner`: Dagger Throw resolves damage, draw1, then selects/discards an actual remaining-hand instance. Reflex2/3 appends draw; Tactician1/2 prepends energy. Neither appears as an ordinarily playable card. Preserve the exact movement/manual-callback/notification order and do not recursively discard reaction-drawn cards.
- [x] Test empty/undersized hand, stale/duplicate choice preflight, distinct card handles, and resumption once without repeating payment/damage/draw. Survivor still grants Block8/11 before its discard.
- [x] Add `m11_silent_end_turn_cleanup_does_not_fire_manual_discard_reactions`: cleanup and exhaust are not manual discard. Retained cards do not move; Ethereal follows its own exhaust path. If a shared counter is introduced, distinguish its source-specific increment from listener eligibility.
- [x] Add a retained-fault case after one selected discard/reaction commits; verify exact card zones, queued sibling work and first fault with no redelivery. Run `cargo test -p spire-core --lib --locked --offline m11_silent_discard` to real RED, implement the minimal listeners/continuation, rerun GREEN and existing `piles`, `selection`, `m10_exhaust_hooks` filters.

### Task 4: Ordinary Shivs and Accuracy instance semantics

**Files:** Modify `combat/cards/silent.rs`, new `combat/cards/silent/shiv.rs`, `combat/{card_state,model,effects,hooks,piles}.rs` and card IDs/definitions as needed; create `combat/tests/m11_silent_shiv.rs`.

**Interfaces:** Keep existing `CreateCard`, generated-card constructors, copy provenance, damage calculation, `FinishDisposition`, overflow insertion and exit census. Accuracy is a qualified Shiv-identity power; no item effect is invented for this task.

- [x] Write `m11_silent_shiv_generation_and_accuracy_preserve_instance_ownership`: Blade Dance creates3/4 ordinary Shivs for1 energy; Cloak remains Block6 plus1/2 ordinary Shivs. Every generated instance has a distinct identity, cost0, correct4 damage, and real exhaust disposition. Test Shiv+6 through a separately labelled upgrade fixture, not by upgrading Blade Dance.
- [x] Accuracy4/6 rewrites qualifying existing Shivs across hand/draw/discard/exhaust and affects later construction at its actual checkpoint. Assert qualified base-damage changes before ordinary modifiers, upgrades preserved, and no bonus to another zero-cost Attack or permanent master-deck mutation.
- [x] Cover full-hand overflow to discard as creation, not draw/manual discard; stale target/payment rejection; accepted insertion fault; and exact permanent/generated census at exit. Reuse existing working constructors rather than replacing them.
- [x] Run `cargo test -p spire-core --lib --locked --offline m11_silent_shiv` to actual RED for the new producer/Accuracy behavior, implement minimally, rerun GREEN plus existing common-Shiv, `m10_copy`, `m10_exhaust_actions`, and `m10_settlement` checks.

### Task 5: Real optional Well-Laid Plans retention

**Files:** Modify `combat/cards/silent.rs`, new `combat/cards/silent/retain.rs`, `combat/{card_state,model,selection,view,piles,turns,hooks}.rs`; card IDs/definitions; create `combat/tests/m11_silent_retain.rs`. If the new selection purpose needs a full-report adapter, modify only `crates/spire-report/src/actions.rs`, `crates/spire-report/src/snapshot.rs`, and `crates/spire-report/tests/public_snapshot.rs` (and its explicit protocol validation if required); no numeric/Gym descriptor changes.

**Interfaces:** Add only the narrowly required optional retain operation/state to the existing selection and turn-end owners. It must resume the existing EndTurn parent; it is not an Elixir/exhaust selection or a second turn scheduler. Register a corresponding public selection purpose in existing full-combat reporting if the new operation requires it, preserving prior report contracts.

- [x] Write `m11_silent_retain_preserves_selected_instances_at_end_turn`: Well-Laid Plans1/2 opens a real optional 0..min(limit,hand) choice at the qualified end-turn checkpoint. Confirm0 is legal when a selector exists; do not fabricate an empty-hand choice. Selected non-Ethereal identities survive cleanup; nonselected normal cards discard.
- [x] Selected Ethereal cards remain subject to exhaust and are not removed from the legal choice set merely to evade their rule. Temporary retain expires at its actual boundary; do not create permanent self-retain or retain every card.
- [x] Test stale/duplicate requests before mutation, forced versus ordinary turn ending where the supported shared pipeline reaches this checkpoint, and an accepted choice/callback fault retaining the parent, selected instances and first fault without replaying payment or cleanup.
- [x] Run `cargo test -p spire-core --lib --locked --offline m11_silent_retain` to behavioral RED, implement the minimal state/continuation, rerun GREEN plus existing `turns`, `piles`, `selection`, `m10_autoplay`, and optional-Elixir/native report regressions. Do not broaden to unsupported Pyramid/Equilibrium/Establishment mechanics.

### Task 6: Nightmare snapshots and independent delayed delivery

**Files:** Modify `combat/cards/silent.rs`, new `combat/cards/silent/copy.rs`, `combat/{model,card_state,effects,hooks,turns,selection}.rs`, card IDs/definitions; create `combat/tests/m11_silent_copy.rs`.

**Interfaces:** Reuse existing stat-equivalent-copy constructors and queue. Add a narrow owner-local pending-template record only if the current power representation cannot retain independent captures and delivery cursors; templates are snapshots, not duplicate owners of a permanent card. No generic copy framework or second scheduler.

- [x] Write `m11_silent_nightmare_copies_are_owned_and_delivered_once`: cost3/2, Exhaust, select one remaining-hand card, capture/reset its stat-equivalent template at the qualified construction point, create no immediate copies, then exactly3 at the next qualified start-turn in both versions. Pin source-qualified ordering against draw before asserting it.
- [x] Use an upgraded card and labelled real native local-state fixture to distinguish combat cost from turn cost, preserved free flag, and later mutation of the original. The captured template must not become a live lookup or an alias to a permanent instance. Copies have distinct generated identities and qualified overflow destinations.
- [x] Two selected templates retain independent delayed owners, even when display IDs match. Empty eligibility does not invent a Confirm menu. A later construction/insertion/callback failure retains the accepted template/cursor/children and first fault; already delivered copies are not created again. Terminal cleanup preserves the permanent-card census and never turns templates into gains.
- [x] Run `cargo test -p spire-core --lib --locked --offline m11_silent_copy` to behavioral RED, implement minimally, rerun GREEN plus existing `m10_copy`, `m10_autoplay`, `m10_exhaust_hooks`, and `m10_settlement` tests.

### Task 7: Canonical Silent resources, owned combat entry, and runnable tools

**Files:** Modify `run/combat.rs` for a bounded extension of existing `Run::from_authored_combat`; create `run/tests/m11_silent.rs`, `crates/spire-core/tests/m11_silent_combat.rs`, and `crates/spire-core/examples/m11_silent_combat.rs`. Use a focused example support module only if needed. No Run/Gym profile or Python files.

**Interfaces:** Preserve the existing constructor signature `Run::from_authored_combat(RunConfig, AuthoredRunStart, RunRng) -> Result<Run, RunError>`, `Run::new`, `enter_combat`, and `step`. For Silent A0, validate canonical initial resources against the shared initializer/Catalog; retain all existing Ironclad M3 authored configurations. The constructor's starter restriction is not reapplied to the evolved state at every subsequent entry/settlement.

- [x] Write `m11_silent_normal_start_enters_real_combat_with_owned_character_state`: obtain Silent's public starting decision from `Run::new`, check70/70HP,99gold, exact12 unupgraded starter instances, Ring, empty slots and no keys, then populate the existing authored-combat constructor from those values and enter a real supported Jaw Worm/Cultist encounter.
- [x] The actual RED is the existing Silent character rejection. Ring's already-working base5+hook2 draw is a GREEN characterization; do not implement it twice. Verify actual seven-card opening, public legal starter actions, one item owner, and permanent identity/census through accepted combat exit/settlement. A separate supplied diagnostic may cover an unobserved Victory/second-combat path; it is not ordinary run completion.
- [x] Reject modified initial HP/maxHP/gold/starter deck/upgrades/relics/potions/counters atomically; use canonical data rather than a second starter list. `AuthoredRunStart` has no key override: assert constructed keys remain false instead of adding such a parameter. Keep all old Ironclad M3 rejection/acceptance cases.
- [x] The mandatory native example takes `--seed`, `--budget`, and `--scenario starter|supplied|all`. Execute fixedseeds7/11 without search; an untuned public-legal caller exercises the canonical-resource authored combat. Separately labelled supplied native cases exercise Poison/discard/Shiv/retain/copy without pretending those cards were acquired in the starter episode. Print actual outcomes and scope; ordinary Neow/map is explicitly bypassed.
- [x] Genuine native Death/Victory ends successfully; exhausted budget, invalid arguments and operational faults are errors, never fake Death or Gym truncation. Run budgets0/1 and invalid scenario as expected-error controls. No winning outcome is required.
- [x] Run the focused bridge tests to RED -> minimal extension -> GREEN, then `cargo test -p spire-core --test m11_silent_combat --locked --offline` and `cargo run -p spire-core --example m11_silent_combat --locked --offline -- --seed 7 --scenario all`, followed by seed11 and the controls. These are canonical-resource authored combats and supplied diagnostics, **not** normal-Neow episodes, M11 RunSession/replay, or M12 Act1 evidence.

### Task 8: Candidate profile, preservation matrix, and ONE milestone review

**Files:** Create `docs/milestones/m11-silent-combat.md` as a candidate **before** review; scoped README/scope/work-status/guidance/plan status updates only. All implemented M11 tests and module registrations belong in the candidate snapshot.

**Interfaces:** Consumes Tasks1–7's exact source and actual evidence; produces a declared supported profile, one final independent review, and acceptance or a named unresolved defect. The user approves this plan before any of these implementation steps begin.

- [x] Document the actual implemented roster/items, canonical-resource authored-bridge boundary, supplied fixtures, observed outcomes, unobserved paths, source grades, M12–15 remainder, and retained faults. Do not claim a Silent Gym/normal map/Act1/Heart run or complete standard pools.
- [x] Run `cargo test --workspace --locked --offline --no-fail-fast` on the integrated candidate; all consumers/report/public integrations must be included. Recheck native `m11_silent`, `m10_`, `m9_`, `m8_`, `m7_`, `m6_`, relevant shared/old authored tests, all existing ten examples and the M11 example, current M9/M10 fixedseed outcomes, frozen descriptors/replay identities, and `git diff --check`. State actual counts and failure names; the six directly owned Poison regressions should pass from real fixes, while the unrelated seventh remains visible. Any new failure is investigated, never relabelled historical.
- [x] A normal default-feature workspace build is not Python proof. M11 makes no facade/extension change or fresh Python-delivery claim; preserve the previous artifact/environment. Do not rebuild/install to obtain a green label. If a concrete M11 change truly requires a facade/API alteration, name it and revise that scope for user review before proceeding.
- [x] If a task changes an existing reference row/citation, coordinate its owner and run only affected offline checks/builders, preserving uncertainty and CC BY-SA attribution. No blind regeneration or reference check substituted for behavior proof. No data change is presumed by the current plan.
- [x] Self-checked all five Review Focus classes and cross-task interfaces. Completed exactly one fresh independent GPT-6 Luna MAX review in the exact-source materialization. Review `m11_milestone_review` (`.superpowers/sdd/2026-10-25-m11-silent-combat-mechanics/task8-independent-review.md`) returned no unresolved findings; root recorded dispositions for each declined judgment. No per-task reviewer swarm was used.
- [x] Verified the reviewed 622-file candidate bytes unchanged in both checkouts at source root `8bd52da0d0fcb345f708340a28327058e2a133351acfe8da6495169345337ffd`; the preservation matrix recorded 1,476 passed / one retained Blue Slaver and Two Fungi failure. No post-review product-source change or Python extension refresh was made. Root accepted and normally closed M11 `.33.2`/`.33`. The user-prioritized `.66` implementation proceeds in parallel; M11 constrains final integration only. M12 remains planning-only and no M12 product child/start follows. All worktrees and evidence are preserved; no commit/push/PR/merge is implied.

## Controller planning self-review

- Spec coverage maps Poison/powers to Task2, discard/triggers to Task3, Shiv/Accuracy to Task4, retain to Task5, delayed copies to Task6, canonical character state/owned bridge/public demonstration to Task7, and declared scope/preservation/review to Task8.
- Existing Ring behavior is reused; the six in-scope historical tests retain their identities; the seventh stays separate. Normal canonical resources, authored combat, supplied diagnostics, and full normal-run progression have distinct evidence labels.
- Signatures above are existing APIs unless explicitly called a proposed narrow state/operation. New native IDs must not change frozen M9/M10 vocabulary. All five focus cases have named owning tests.
- This is a task/contract plan, not source implementation. Exact source uncertainties have bounded gates; no implementation or source-access approval is inferred from this planning exercise.

## Execution handoff

The user selected native controller integration with useful isolated Luna6 MAX assistants and ONE final independent milestone review. The user approved this actual written plan on 2026-10-25 ("I approve"). Planning `spire-dxo.33.1` and implementation `spire-dxo.33.2`/`.33` are normally closed after the accepted review. M11 approval does not authorize M12 product work or publication.


## Execution authorization addendum (2026-10-26)

The user authorized the Nightmare lookup and any other needed, bounded static
lookup for Gym source blockers using the existing locally owned archive and existing tools, without a
new approval for each named lookup. This authorization is limited to load-bearing
members and their called chains. It does not authorize original-game execution,
new tools or installation, a broad archive audit, private-source publication, or
changes to the approved M11 scope. Record each finding at its actual source grade.
The earlier source-gate text records the plan's original permission state and
remains historical; this dated addendum records the later bounded authorization.
