# M27 relic and potion lifecycles

Status: source merged as PR69 (`5d6a6be`). Historical candidate status at freeze v5 (2026-11-02): the 1,090-entry tree manifest is
`ec633c788b49b937029efa27f05713ea153493f3fbe2f964d4b956e3114014c1`. Freeze v5 adds the fixes for the
first final review's findings (reconciliation D5–D16) to freeze v4. At freeze v5 (2026-11-02), M27 was uncommitted on base `db42bf4d` (the merged PR #68 No Draw and native-fault
fix), which follows PR #67 (`a86df922`) and merged M26 (PR #66, `918f628`). The freeze packet
and its verification records are retained under `main-v1/freeze-v5/`.

This candidate follows the [M27 design](../superpowers/specs/2026-10-31-m27-items-design.md)
and [plan](../superpowers/plans/2026-10-31-m27-items.md). The
[coverage ledger](m27-items-coverage-ledger.md) maps every item to its test key; the
[reconciliation](m27-items-reconciliation.md) records source bindings, owner corrections,
decisions, replay and gate-audit evidence, and remaining boundaries. The
[source binding](m27-items-reconciliation.md#source-binding) uses the retained M27 design and
freeze evidence.

## What M27 is

M27 provides the native shared relic and potion layer for the M28 room flows. The
complete-item capability registers the item unions and routes item effects through shared
owners. M27 adds 117 shared relic identities, completes the effects of
the 62 relics already present after M25, and gives all 42 existing potion identities their
source-specific use and lifecycle rules. The complete item identities are still reachable only
through core tests and the default-off `m27-diagnostics` feature. M27 adds no public or
Gym profile, descriptor, fingerprint or Python surface.

One catalog-derived capability controls complete item admission. One dispatcher visits relics
in acquisition order, and one inventory moves between Run and combat custody at entry and
settlement. The item lifecycles cover obtain, equip, selection, replacement, loss, counters,
potions, healing, prevention, rescue, escape and the existing shared room owners. M28 still owns
the broader room flows and offer screens.

## Shared-owner corrections

Corrections were made at shared owners and are listed with amended assertions in the
[global-corrections record](m27-items-reconciliation.md#global-corrections-g1g10) and
[user and PM rulings](m27-items-reconciliation.md#corrections-from-user-and-pm-rulings).
Among them, the energy law now includes Sozu and the new boss energy relics; Sozu applies to
potion reward claims and shop purchases; Potion Belt equips two empty slots; relic visits use
one acquisition-ordered dispatcher; and complete item catalog, tier and potion-class metadata
are shared. Constructed evidence includes `m27_g1_sozu_energy_was_missing`,
`m27_potion_owner_obtain_consumers_reward`,
`m27_potion_owner_obtain_consumers_shop`,
`m27_obtain_equip_resource_laws`, and
`m27_identity_catalog_rows`. G10 moves legacy Pure Water and Holy Water to the single pre-draw
visit in every context; it and G1 amend six accepted assertions, whose original bytes are retained.

Following a user correction, shop card rows offer only coloured and colourless cards and card
rewards never offer curses. The shop's only curse interaction is removal. Constructed invariant
`m27_shop_card_offers_never_contain_a_curse` checks frozen, complete Act 4 and Courier rows;
relic-authored curse obtains remain Omamori-aware.

The Energy Potion capped-energy source in
[defect.rs](../../crates/spire-core/src/combat/cards/defect.rs) now admits complete-item contexts
for all four characters; the Ironclad and Silent paths previously rejected it. The shared
[potions.rs](../../crates/spire-core/src/items/potions.rs) owner receives base potency once, and
[effects.rs](../../crates/spire-core/src/combat/effects.rs) settles Smoke Bomb after queued work.
The Smoke Bomb source basis is static analysis, not original-game execution. Constructed checks in
[combat potion tests](../../crates/spire-core/src/combat/tests/m27_potions.rs) include
`m27_potion_energy_potion_ironclad`, `m27_potion_energy_potion_silent`,
`m27_potion_energy_potion_defect`, `m27_potion_energy_potion_watcher`,
`m27_potion_owner_potency_survives_bark_loss_in_combat`,
`m27_smoke_bomb_rules_and_settlement` and `m27_g10_legacy_pure_water_pre_draw`. Their
source notes are in the [Lane-P fixes](m27-items-reconciliation.md#lane-p-potion-fixes) and
[amended accepted assertions](m27-items-reconciliation.md#amended-accepted-assertions).

The supplied Act 4 Runs found three more shared-owner defects, fixed for every character under
complete admission. The complete elite's potion appearance no longer counts the card-reward row
toward its four-row limit. Golden Idol's +25% applies to elite, boss and event combat gold as
well as ordinary gold. Every room owner now dispatches Maw Bank's floor-entry event. See
[D1–D3](m27-items-reconciliation.md#shared-owner-fixes-from-x2-d1d3).

After the rebase onto PR #68, every M27 all-enemy relic producer selects its targets with the shared
`targetable()` rule, which skips dying, half-dead and escaped enemies. The result is unchanged
([D4](m27-items-reconciliation.md#rebase-onto-pr-68-d4)).

## Complete item capability and unions

The complete unions are computed from catalog metadata, not maintained as duplicate identity
lists:

| Union | Members | Count |
| --- | --- | ---: |
| Complete relics | 178 normally obtainable relics plus Circlet as the pool fallback | 179 |
| Relics per character | 139 shared plus class relics | Ironclad 150; Silent 150; Defect 148; Watcher 148 |
| Complete potions | 30 shared plus 3 class potions per character | 42 |
| Potions per character | 30 shared plus the character's own 3 | 33 |
| Persistent state | 18 relic counters plus 3 bottled-card links | — |

Red Circlet is outside the complete union. The relic union has 40 class relics (Ironclad 11,
Silent 11, Defect 9, Watcher 9), and all 117 new M27 relics are shared. The complete potion
union admits only a character's own class potions. The Constructed
catalog and admission tests are `m27_catalog_matches_reference`,
`m27_identity_complete_unions`,
`m27_catalog_potion_table_matches_design`,
`m27_watcher_class_potions_are_class_restricted` and
`m27_capability_combat_admission`.
The counter and card-link custody is checked by Constructed test
`m27_counters_and_links_move_once`.

Run admission stores `ItemAdmission::Complete` and requires M26's
`CardAdmission::Complete`. Combat admission uses
`CombatItemContext::with_complete_items()` only with complete cards. The complete
validator checks catalog membership, duplicate relics (except Circlet's stacking counter),
starter/replacement exclusivity, potion capacity, and counter ownership and domains. Profile
admission and its finite allow-lists remain unchanged. Complete admission is selected only by
core diagnostics and tests.

## Dispatcher and gate rule

[`dispatch.rs`](../../crates/spire-core/src/items/dispatch.rs) owns the combat-phase and
Run-event relic visit loops. In each
phase, relics run in acquisition order; each handler keeps its source queue side, and top-queued
groups follow the resolver's insertion law. Non-queued modifiers use one law at their existing
owner. The Constructed checks are
`m27_dispatch_visit_order_is_acquisition_order`,
`m27_dispatch_cross_path_order_follows_acquisition`,
`m27_dispatch_top_work_order` and
`m27_single_relic_visit_loop`.

The source gate audit classifies item comparisons as capability gates (I), admission (A),
potion use (P), item state (S), retained profile structure (C), complete-only overlays (O), or
debug-only fields. It keeps the finite profile branches intact and applies complete behavior at
shared owners. The
[frozen transcript replay](m27-items-reconciliation.md#frozen-transcript-replay) and
[gate audit (design §6.3)](m27-items-reconciliation.md#gate-audit-design-63) record the
reconciled sites and the disclosed differences.

## Relic lifecycle

The shared obtain owner validates the complete union, then appends and equips a relic once.
Equip effects include max-HP and gold changes, Potion Belt slots, M26-owned card mutations,
relic and potion offers, and selections through Run continuations. The four boss relics replace
their starter in place. Loss runs the relic's unequip rule and removes its counters and links.
The complete pool owner in [relic_pools.rs](../../crates/spire-core/src/items/relic_pools.rs)
owns tier rolls, front/back selection, candidate consumption, spawn restrictions, tier
promotion and Circlet fallback. **Static source research** records `returnRandomRelicKey`
returning Red Circlet when the Boss pool is empty
([Act 1 continuations](../research/2026-10-19-act1-continuations.md#L268)).
PM ruling F7 refines planning ruling 3 and supersedes design §7.4's invariant-error expectation;
Red Circlet remains outside ordinary unions. Constructed test
`m27_relic_pool_law_empty_boss_returns_red_circlet` checks the empty-pool fallback; standard-run
reachability is not established. See the [frontier binding](m27-items-reconciliation.md#frontier-bindings-design-13).
Other Constructed evidence includes
`m27_obtain_equip_resource_laws`, `m27_replacement_and_loss`,
`m27_bottled_flame_selected_attack_innate`,
`m27_astrolabe_three_transform_upgrade`,
`m27_relic_pool_law_build_excludes_owned_and_other_classes_for_each_character`,
`m27_relic_pool_law_front_back_boss_front_and_consumed_if_skipped` and
`m27_relic_pool_law_empty_boss_returns_red_circlet`.

Persistent counters, potion slots and potencies move once between the Run and combat context;
combat-local counters are dropped at settlement. The three bottled-card links also move with the
inventory. Shared Run owners apply relic laws at combat entry and settlement, campfire, shop,
combat rewards, chest opening, floor and room entry, and card, gold, healing and maximum-HP
changes. Constructed checks include `m27_counters_and_links_move_once`,
`m27_counters_persist_through_every_combat_end`,
`m27_ancient_tea_set_rest_arms_energy` and
`m27_membership_card_purchase_discounts_current_shop`.

M27 implements named relic laws for the rooms M28 completes, including map-path, unknown room,
chest, event, Neow and card-reward rarity effects. Constructed tests are
`m27_run_seams_reached`, `m27_room_entered_every_owner` and
`m27_extra_card_rewards_are_distinct` in the
[run seam tests](../../crates/spire-core/src/run/tests/m27_run_seams.rs). They exercise these
entry points but do not establish M28 room flows. See the
[frontier bindings](m27-items-reconciliation.md#frontier-bindings-design-13) and
[M28 call list](m27-items-reconciliation.md#m28-call-list-design-113).

## Potion lifecycle, healing, rescue and escape

Potion slots have one owner in the Run and move into and out of combat once. Capacity is three
slots, plus two from Potion Belt. Obtaining uses the first empty slot; with every slot full the
potion is refused. Sozu is checked by the reward and shop consumers. Potion use follows each potion's own
phase, target and source law for every character whose union includes it. Potency is set when
obtained, recalculated for held potions when Sacred Bark is equipped, and remains doubled if
Sacred Bark is later lost. Combat discard empties a slot without use callbacks.

The rarity-first, limited and flat potion factories use the player's 33-potion union. The
complete Act 4 shop and elite producers use those shared factories and the appearance law while
keeping accepted row structure and frozen profile offers. Constructed checks include
`m27_potion_owner_use_law_all_characters`,
`m27_potion_lifecycle_blood_potion_outside_combat`,
`m27_potion_lifecycle_entropic_brew_unrestricted_and_sozu_first`,
`m27_potion_lifecycle_discard_clears_combat_and_run_potency`,
`m27_act4_potion_producers_under_complete_elite` and
`m27_act4_potion_producers_under_complete_shop`.

Healing, prevention and rescue share their existing owners. Fairy in a Bottle is the first rescue
before Lizard Tail; Mark of the Bloom prevents rescue. Smoke Bomb is barred against boss or
Back Attack enemies. Escape settles after queued work, grants no combat rewards, returns the
inventory once, preserves persistent counters and drops combat-local state. Constructed tests
`m27_rescue_order_fairy_then_lizard_tail_once` and
`m27_smoke_bomb_rules_and_settlement` cover these boundaries.

G9/F3 is a PM-bound global correction: Burning Blood, Black Blood, Meat on the Bone and Face of
Cleric do not receive Victory effects when Smoke Bomb ends combat as Escaped. Static source
reading through retained research supports the correction: Burning Blood's `onVictory` hook is
recorded in [relic research](../research/2026-10-18-act1-relics-closure.md#L40). Catalog primary
rules record Burning Blood's `Victory:` heal, Black Blood's living victory heal, Meat on the Bone's
pre-victory `endBattle` hook, and Face of Cleric's Victory effect
([relic catalog](../../data/relics/relics.json#L1305),
[Black Blood](../../data/relics/relics.json#L654),
[Meat on the Bone](../../data/relics/relics.json#L5087),
[Face of Cleric](../../data/relics/relics.json#L3093)). Main-3's source reading records that
`endBattle` skips `onVictory` in a smoked room
(`main-v1/assistant-contracts/a26-combat-potions/CONTRACT.md`).
Constructed test `m27_g9_escape_skips_victory_relics` pins the boundary. The complete-item change
is frozen-unreachable: M15 Silent and M20 Defect Smoke Bomb contexts cannot own these Ironclad
Victory relics. See [G9](m27-items-reconciliation.md#global-corrections-g1g10) and
[F3](m27-items-reconciliation.md#frontier-bindings-design-13) in the reconciliation. This is
static source evidence, not original-runtime execution.

## Evidence (grades kept separate)

The design defines **Constructed** evidence as direct owner checkpoints and **Supplied played**
evidence as a supplied start followed only by legal actions through the real owners. This freeze has
both. It reports no Normal played cases or Native agreement runs.

**Constructed checks.**

- Catalog and admission: [catalog tests](../../crates/spire-core/tests/m27_catalog.rs) and
  [capability tests](../../crates/spire-core/src/items/tests/m27_capability.rs):
  `m27_identity_catalog_rows`, `m27_identity_complete_unions`,
  `m27_catalog_matches_reference`, `m27_capability_combat_admission`.
- Dispatcher: [dispatch tests](../../crates/spire-core/src/items/tests/m27_dispatch.rs):
  `m27_dispatch_cross_path_order_follows_acquisition`,
  `m27_dispatch_top_work_order`, `m27_single_relic_visit_loop`.
- Combat relics: [accepted relic tests](../../crates/spire-core/src/items/tests/m27_existing_relics.rs),
  [turn tests](../../crates/spire-core/src/items/tests/m27_relics_turns.rs),
  [play tests](../../crates/spire-core/src/items/tests/m27_relics_play.rs) and
  [structural tests](../../crates/spire-core/src/items/tests/m27_relics_structural.rs):
  `m27_existing_relics_equal_accepted`, `m27_akabeko_first_attack_bonus`,
  `m27_pen_nib_tenth_attack_doubles`, `m27_the_boot_four_damage_minimum`,
  `m27_necronomicon_with_duplicators`.
- Run relics: [lifecycle tests](../../crates/spire-core/src/run/tests/m27_relic_lifecycle.rs),
  [pool tests](../../crates/spire-core/src/run/tests/m27_relic_pools.rs),
  [selection tests](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) and
  [Run effect tests](../../crates/spire-core/src/run/tests/m27_relics_run.rs):
  `m27_obtain_equip_resource_laws`, `m27_replacement_and_loss`,
  `m27_relic_pool_law_roll_tier_boundaries_for_both_tables`,
  `m27_relic_pool_law_empty_boss_returns_red_circlet`,
  `m27_bottled_flame_selected_attack_innate`.
- Potions and rescue: [potion owner tests](../../crates/spire-core/src/items/tests/m27_potion_owner.rs),
  [Run potion tests](../../crates/spire-core/src/run/tests/m27_potion_lifecycle.rs) and
  [rescue and escape tests](../../crates/spire-core/src/combat/tests/m27_rescue_escape.rs):
  `m27_potion_owner_use_law_all_characters`,
  `m27_potion_lifecycle_entropic_brew_unrestricted_and_sozu_first`,
  `m27_rescue_order_fairy_then_lizard_tail_once`,
  `m27_smoke_bomb_rules_and_settlement`.
- Cross-character matrix (design §15.3):
  [cross-character tests](../../crates/spire-core/src/items/tests/m27_cross_character.rs):
  `m27_cross_character_shared_relic_deltas` and `m27_cross_character_shared_potion_deltas`. These
  check that each of the 139 shared relics and 30 shared potions has the same public-state effect for
  every character as for Ironclad. The other seven tests give literal rows for the character-sensitive
  energy, orb-slot, card-pool and potion-pool items. Inertness is `m27_capability_overlay_inert`.

**Supplied played checks (design §15.4).** The
[supplied-run tests](../../crates/spire-core/src/run/tests/m27_supplied.rs)
`m27_supplied_act4_ironclad`, `m27_supplied_act4_silent`, `m27_supplied_act4_defect` and
`m27_supplied_act4_watcher` (feature `m27-diagnostics`) each play a supplied set of at least 20
combat relics and three potions. The route is the Act 4 campfire, shop, elite rewards and Heart,
using legal actions only. They assert the section 11.4 shop and elite identities and prices, potion
appearance, heal and gold laws, counters across both combats, inventory return and rescue order.
Each Run ends in a Heart Death that settles with no fault. The sets, rulings and coverage bounds are in
the [reconciliation](m27-items-reconciliation.md#supplied-act-4-runs-x2-design-154).

**Source audit (not a gameplay grade).** The gate-audit tests
`m27_no_inert_items` and `m27_single_item_owners` check one test key and one
owner for each registered item and classify the source gate sites.

The independently verified freeze v5 retained 108 workspace binaries with 3,986 passed,
0 failed and 11 ignored, and 37 `m27-diagnostics` core binaries with 3,380 passed,
0 failed and 5 ignored. The `m27_` library filter reported 494 passed, 0 failed and 0 ignored
(521 with `m27-diagnostics`). These are suite results, not additional evidence grades.
The toolchain was rustc and cargo 1.99.0. Logs, invocation records and the manifest are retained
under `main-v1/freeze-v5/`.

The gate audit reconciled 639 H0 rows: 514 matched, 104 matched-changed and 21 removed because
they moved; none were dropped. It classified 167 new sites (A 87, I 51, P 17, C 4, S 4, O 3,
Debug 1). Among 51 frozen-reachable sites, 38 were unchanged or complete-gated, 11 were
reachable-identical and 2 were disclosed corrections (G4 and G10). There were zero unexplained
frozen changes and zero combat-v1/M9/M10 public-observation changes. Freeze v3 adds eight
complete-gated sites for the D1–D3 fixes, none frozen-reachable. Freeze v4 adds the rebase
sites, all reachable with identical results. Freeze v5 adds the complete-gated D5–D16 sites, none
frozen-reachable. The Constructed audit tests
`m27_no_inert_items` and `m27_single_item_owners` passed.

The frozen transcript replay used `m27_frozen_replay` over 323 inputs. Against pre-M27
BASE-A86, the only differences were disclosed G4 reorder-only inputs pair-059 and pair-060
(M15 Silent co-firing order). On a fresh baseline built from `db42bf4d` the same two inputs
are the only differences, and the freeze-v4 tree compared with the pre-rebase copy was 323/323
identical. The freeze-v5 copy is 323/323 identical to the freeze-v4 copy. This replay is frozen-profile regression evidence; it does not provide M27 Run
evidence. See the
[replay record](m27-items-reconciliation.md#frozen-transcript-replay).

## Limits

M27 does not claim:

- a public or Gym profile, descriptor, fingerprint, Python surface or extension delivery;
- complete shop stock, general reward rows, events, Neow flow or other room offers (M28);
- unknown-room resolution, map-path selection or the card-reward rarity schedule (M28);
- A1–A20, including the A11 potion-slot reduction (M31), or Red Circlet as a union member or
  normally obtainable relic;
- supplied coverage of Burning Blood, Sozu, Circlet, Omamori, Darkstone Periapt or the §11.3
  room laws on the Act 4 route; Constructed tests cover them instead;
- a score screen (M34), original-runtime or Java RNG parity, strategy or a win.

The named M28 room laws are Constructed entry-point evidence until M28 calls them. F7's source
fallback and Constructed empty-Boss-pool check are described above. G9/F3 is bound by PM ruling
and static source reading, and frozen-unreachable as described above. M27 merged as PR69;
its finite lifecycle scope is complete. Remaining evidence and scope boundaries are
recorded in [Not established](m27-items-reconciliation.md#not-established).
