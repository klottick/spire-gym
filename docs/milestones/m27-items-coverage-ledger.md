# M27 relic and potion coverage ledger

One row per complete-union relic and potion identity, following the M26 coverage-ledger
structure. Read with the [profile](m27-items.md), the
[reconciliation](m27-items-reconciliation.md#gate-audit-design-63), the
[design](../superpowers/specs/2026-10-31-m27-items-design.md) §§4.1–4.4 and 15.1–15.3,
and the [plan](../superpowers/plans/2026-10-31-m27-items.md) Task M7.

The source is `main-v1/gate-audit-v1/ledger.tsv`, checked against
[m27_gate_audit.rs](../../crates/spire-core/src/items/tests/m27_gate_audit.rs). Its
`file` value is a repository-relative test-module path and its `test`
value is the literal test name. The per-row link therefore names the test module the audit pins; it is not the production
handler file. The shared state-owner census is summarized separately below.

M27 remains a candidate with no public/Gym profile and no Python-delivery claim. The gate-audit
REPORT.md records 2 focused audit tests and 475 M27 tests passing on its audited tree. Freeze v3
added the X1 and X2 tests, and freeze v4 (rebased onto PR #68) keeps the same 484 `m27_` library
tests, or 508 with `m27-diagnostics`. Freeze v5 adds the 13 final-review tests (`m27_review_*`, named in
the rows below): 494 `m27_` library tests, or 521 with `m27-diagnostics`. This documentation
pass verified test names and paths with `grep -nF`; it did not run Cargo.

## Inventory totals

| Union | Ledger rows | Design §4.4 target | Check |
| --- | ---: | ---: | --- |
| Complete relic identities | 179 | 179 | Match |
| Complete potion identities | 42 | 42 | Match |

The gate-audit report says the 221 rows have no duplicates or extras and equal the complete
catalog unions; the catalog tests `m27_identity_catalog_rows` and
`m27_identity_complete_unions` pin the same totals. Per character, design §4.4 gives
150 / 150 / 148 / 148 relics (Ironclad / Silent / Defect / Watcher) and 33 potions each. The
grade for these catalog checks is **Constructed**. There is no row-count mismatch.

The relic tier counts are Starter 4, Common 36, Uncommon 36, Rare 34, Boss 30, Shop 20 and
Special 19. They sum to 179, matching design §§4.1 and 4.4. The 139 shared relics plus the
character's own relics make the four per-character totals. Potions are 30 shared plus 3 for each
character, matching §§4.3–4.4.

## Keys and grades

**Constructed** is an owner checkpoint built directly; **Supplied played** is a supplied start
followed only by legal actions through the real owners (key S). Design §15.1 defines no Normal played
cases or Native agreement runs for M27 because it publishes no profile. Every item row carries
the common **K** identity/union checks plus the per-item key assigned in the gate-audit ledger.
The per-row file link is the linked test module; it is not a claim that the test module contains
the production handler.

| Key | Coverage source | Grade |
| --- | --- | --- |
| K | [m27_catalog.rs](../../crates/spire-core/tests/m27_catalog.rs): m27_identity_catalog_rows, m27_catalog_matches_reference, m27_catalog_potion_table_matches_design, m27_watcher_class_potions_are_class_restricted; [m27_capability.rs](../../crates/spire-core/src/items/tests/m27_capability.rs): m27_identity_complete_unions | Constructed |
| D | [m27_dispatch.rs](../../crates/spire-core/src/items/tests/m27_dispatch.rs) | Constructed |
| C1 | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | Constructed |
| C2 | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | Constructed |
| C3 | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs); [m27_potion_rewards.rs](../../crates/spire-core/src/run/tests/m27_potion_rewards.rs) for Prismatic Shard | Constructed |
| J | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs); [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | Constructed |
| E | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs), with the row-specific Main tests linked below | Constructed |
| L | [m27_relic_lifecycle.rs](../../crates/spire-core/src/run/tests/m27_relic_lifecycle.rs) | Constructed |
| P | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs); [m27_potion_lifecycle.rs](../../crates/spire-core/src/run/tests/m27_potion_lifecycle.rs); [m27_potion_owner.rs](../../crates/spire-core/src/items/tests/m27_potion_owner.rs) | Constructed |
| H | [m27_rescue_escape.rs](../../crates/spire-core/src/combat/tests/m27_rescue_escape.rs); [m27_energy_law.rs](../../crates/spire-core/src/items/tests/m27_energy_law.rs) | Constructed |
| X | [m27_cross_character.rs](../../crates/spire-core/src/items/tests/m27_cross_character.rs): m27_cross_character_shared_relic_deltas, m27_cross_character_shared_potion_deltas, m27_cross_character_energy_recharge_literals, m27_cross_character_prismatic_shard_orb_slots, m27_cross_character_card_generator_pools, m27_cross_character_run_card_generator_pools, m27_cross_character_cauldron_potion_offer_pools, m27_cross_character_white_beast_statue_potion_offer_pool, m27_cross_character_potion_generator_pools | Constructed |
| I | [m27_inert.rs](../../crates/spire-core/src/items/tests/m27_inert.rs) | Constructed |
| S | [m27_supplied.rs](../../crates/spire-core/src/run/tests/m27_supplied.rs) (feature m27-diagnostics): m27_supplied_act4_ironclad, m27_supplied_act4_silent, m27_supplied_act4_defect, m27_supplied_act4_watcher | Supplied played |

The gate-audit row's `form` field is `fn` for a function declaration or
`macro` for a macro-generated test name. The potion macros instantiate the named test
cases for the characters in each potion's union. The row tables retain the exact test name from
`ledger.tsv`; K's catalog checks apply to every identity row.

## Shared state owners

`owners.tsv` records 24 Owner, Projection and TransientInput fields; the gate-audit
report says the source census matched all 24. The combat inventory owner is
[CombatItemContext](../../crates/spire-core/src/items/mod.rs), and the persistent Run owner is
[RunState](../../crates/spire-core/src/run/model.rs). Public projections are
[CombatInventoryView](../../crates/spire-core/src/combat/view.rs) and
[PublicPlayer](../../crates/spire-core/src/run/decision.rs). Transient inputs include
[CompleteItemSupply](../../crates/spire-core/src/run/diagnostics/m27.rs),
[AuthoredRunStart](../../crates/spire-core/src/run/model.rs),
[CombatTrainingEntry](../../crates/spire-core/src/run/training.rs), and the potion-use frame in
[items/mod.rs](../../crates/spire-core/src/items/mod.rs). This census describes shared storage and
projection owners; it does not assign a production handler file per item.

## Relics

### Starter (4)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `burning_blood` | Relic — Starter | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `cracked_core` | Relic — Starter | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `pure_water` | Relic — Starter | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_g10_legacy_pure_water_pre_draw` | Constructed |
| `ring_of_the_snake` | Relic — Starter | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |

### Common (36)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `akabeko` | Relic — Common | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_akabeko_first_attack_bonus` | Constructed |
| `anchor` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `ancient_tea_set` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_ancient_tea_set_rest_arms_energy` | Constructed |
| `art_of_war` | Relic — Common | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_art_of_war_no_attack_energy` | Constructed |
| `bag_of_marbles` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `bag_of_preparation` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `blood_vial` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `the_boot` | Relic — Common | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_the_boot_four_damage_minimum` | Constructed |
| `bronze_scales` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `centennial_puzzle` | Relic — Common | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_centennial_puzzle_first_hp_loss_draw` | Constructed |
| `ceramic_fish` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_ceramic_fish_card_obtain_gold` | Constructed |
| `damaru` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `data_disk` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `dream_catcher` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_dream_catcher_rest_card_reward` | Constructed |
| `happy_flower` | Relic — Common | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_happy_flower_third_turn_energy` | Constructed |
| `juzu_bracelet` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_juzu_bracelet_monster_to_event` | Constructed |
| `lantern` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `maw_bank` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_maw_bank_floor_gold_shop_disable`; Main-10 review fix: `m27_review_i1_maw_bank_shop_purge_spends_gold_and_disables` (m27_relics_run.rs) | Constructed |
| `meal_ticket` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_meal_ticket_shop_entry_heal` | Constructed |
| `nunchaku` | Relic — Common | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_nunchaku_tenth_attack_energy` | Constructed |
| `oddly_smooth_stone` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `omamori` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_omamori_negates_through_the_card_owner` | Constructed |
| `orichalcum` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `pen_nib` | Relic — Common | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_pen_nib_tenth_attack_doubles` | Constructed |
| `potion_belt` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters`; Main-10 review fix: `m27_review_m2_lost_potion_belt_keeps_five_slot_capacity` (m27_relic_lifecycle.rs) | Constructed |
| `preserved_insect` | Relic — Common | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_preserved_insect_elite_hp_floor` | Constructed |
| `red_skull` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `regal_pillow` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_regal_pillow_rest_heal_plus_fifteen` | Constructed |
| `smiling_mask` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_smiling_mask_removal_price_fifty` | Constructed |
| `snecko_skull` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `strawberry` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `tiny_chest` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_tiny_chest_fourth_unknown_treasure` | Constructed |
| `toy_ornithopter` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `vajra` | Relic — Common | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `war_paint` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_war_paint_two_skill_upgrades` | Constructed |
| `whetstone` | Relic — Common | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_whetstone_two_attack_upgrades` | Constructed |

### Uncommon (36)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `blue_candle` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_blue_candle_curse_play_exhaust` | Constructed |
| `bottled_flame` | Relic — Uncommon | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_bottled_flame_selected_attack_innate` | Constructed |
| `bottled_lightning` | Relic — Uncommon | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_bottled_lightning_selected_skill_innate` | Constructed |
| `bottled_tornado` | Relic — Uncommon | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_bottled_tornado_selected_power_innate` | Constructed |
| `the_courier` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_the_courier_shop_price_twenty_off` | Constructed |
| `darkstone_periapt` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_darkstone_periapt_curse_max_hp_six` | Constructed |
| `duality` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `eternal_feather` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_eternal_feather_rest_room_heal` | Constructed |
| `frozen_egg` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_frozen_egg_power_obtain_upgrade`; Main-10 review fix: `m27_review_m4_reward_screen_eggs_upgrade_matching_generated_cards` (m27_relics_run.rs) | Constructed |
| `gold_plated_cables` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `gremlin_horn` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_gremlin_horn_nonfinal_kill_payoff` | Constructed |
| `horn_cleat` | Relic — Uncommon | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_horn_cleat_second_turn_block` | Constructed |
| `ink_bottle` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_ink_bottle_tenth_card_draw` | Constructed |
| `kunai` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_kunai_third_attack_dexterity` | Constructed |
| `letter_opener` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_letter_opener_third_skill_thorns` | Constructed |
| `matryoshka` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_matryoshka_nonboss_chest_charge` | Constructed |
| `meat_on_the_bone` | Relic — Uncommon | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_meat_on_the_bone_half_hp_heal`; Main-10 review fix: `m27_review_i3b_magic_flower_scales_victory_heal` (m27_relics_turns.rs) | Constructed |
| `mercury_hourglass` | Relic — Uncommon | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_mercury_hourglass_turn_start_thorns` | Constructed |
| `molten_egg` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_molten_egg_attack_obtain_upgrade`; Main-10 review fix: `m27_review_m4_reward_screen_eggs_upgrade_matching_generated_cards` (m27_relics_run.rs) | Constructed |
| `mummified_hand` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_mummified_hand_power_cost_zero` | Constructed |
| `ninja_scroll` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `ornamental_fan` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_ornamental_fan_third_attack_block` | Constructed |
| `pantograph` | Relic — Uncommon | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_pantograph_boss_start_heal` | Constructed |
| `paper_krane` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `paper_phrog` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `pear` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `question_card` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_question_card_reward_choice_plus_one` | Constructed |
| `self_forming_clay` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `shuriken` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_shuriken_third_attack_strength` | Constructed |
| `singing_bowl` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_singing_bowl_bowl_max_hp_two` | Constructed |
| `strike_dummy` | Relic — Uncommon | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_strike_dummy_strike_tag_bonus` | Constructed |
| `sundial` | Relic — Uncommon | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_sundial_third_shuffle_energy` | Constructed |
| `symbiotic_virus` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `teardrop_locket` | Relic — Uncommon | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `toxic_egg` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_toxic_egg_skill_obtain_upgrade`; Main-10 review fix: `m27_review_m4_reward_screen_eggs_upgrade_matching_generated_cards` (m27_relics_run.rs) | Constructed |
| `white_beast_statue` | Relic — Uncommon | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_white_beast_statue_potion_chance_override`; Main-10 review fix: `m27_review_m6_escaped_potion_miss_keeps_offset_history` (m27_potion_factory.rs) | Constructed |

### Rare (34)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `bird_faced_urn` | Relic — Rare | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_bird_faced_urn_power_use_heal` | Constructed |
| `calipers` | Relic — Rare | [m27_energy_law.rs](../../crates/spire-core/src/items/tests/m27_energy_law.rs) | `K` (common); `H`: `m27_calipers_keeps_fifteen_less` | Constructed |
| `captains_wheel` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_captains_wheel_third_turn_block` | Constructed |
| `champions_belt` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `charons_ashes` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `cloak_clasp` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `dead_branch` | Relic — Rare | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_dead_branch_actual_exhaust_card`; Main-10 review fix: `m27_review_m1_dead_branch_precedes_dark_embrace_at_full_hand` (m27_relics_play.rs) | Constructed |
| `du_vu_doll` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_du_vu_doll_curse_count_strength` | Constructed |
| `emotion_chip` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `fossilized_helix` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_fossilized_helix_first_hit_buffer` | Constructed |
| `gambling_chip` | Relic — Rare | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_gambling_chip_discards_trigger_tough_bandages` | Constructed |
| `ginger` | Relic — Rare | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_ginger_weak_prevention` | Constructed |
| `girya` | Relic — Rare | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_girya_three_lifts_strength` | Constructed |
| `golden_eye` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `ice_cream` | Relic — Rare | [m27_energy_law.rs](../../crates/spire-core/src/items/tests/m27_energy_law.rs) | `K` (common); `H`: `m27_energy_law_all_characters` | Constructed |
| `incense_burner` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_incense_burner_sixth_turn_intangible` | Constructed |
| `lizard_tail` | Relic — Rare | [m27_rescue_escape.rs](../../crates/spire-core/src/combat/tests/m27_rescue_escape.rs) | `K` (common); `H`: `m27_rescue_order_fairy_then_lizard_tail_once` | Constructed |
| `magic_flower` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; Main-10 review fix: `m27_review_i3b_magic_flower_scales_victory_heal` (m27_relics_turns.rs) | Constructed |
| `mango` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `old_coin` | Relic — Rare | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_old_coin_equip_gold_300` | Constructed |
| `peace_pipe` | Relic — Rare | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_peace_pipe_toke_remove_card` | Constructed |
| `pocketwatch` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_pocketwatch_later_turn_draw` | Constructed |
| `prayer_wheel` | Relic — Rare | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_prayer_wheel_monster_second_card_row` | Constructed |
| `shovel` | Relic — Rare | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_shovel_dig_front_relic` | Constructed |
| `stone_calendar` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_stone_calendar_seventh_turn_thorns` | Constructed |
| `the_specimen` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `thread_and_needle` | Relic — Rare | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_thread_and_needle_start_plated_armor` | Constructed |
| `tingsha` | Relic — Rare | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `torii` | Relic — Rare | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_torii_two_to_five_damage` | Constructed |
| `tough_bandages` | Relic — Rare | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `E`: `m27_gambling_chip_discards_trigger_tough_bandages` | Constructed |
| `tungsten_rod` | Relic — Rare | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_tungsten_rod_positive_loss_minus_one` | Constructed |
| `turnip` | Relic — Rare | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_turnip_frail_prevention` | Constructed |
| `unceasing_top` | Relic — Rare | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_unceasing_top_empty_hand_draw` | Constructed |
| `wing_boots` | Relic — Rare | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_wing_boots_three_offpath_moves` | Constructed |

### Boss (30)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `astrolabe` | Relic — Boss | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_astrolabe_three_transform_upgrade` | Constructed |
| `black_blood` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `black_star` | Relic — Boss | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_black_star_elite_second_relic` | Constructed |
| `busted_crown` | Relic — Boss | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_busted_crown_one_reward_choice` | Constructed |
| `calling_bell` | Relic — Boss | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_calling_bell_curse_three_tiers` | Constructed |
| `coffee_dripper` | Relic — Boss | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_coffee_dripper_no_rest_plus_energy` | Constructed |
| `cursed_key` | Relic — Boss | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_cursed_key_nonboss_chest_curse` | Constructed |
| `ectoplasm` | Relic — Boss | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_ectoplasm_gold_gain_blocked` | Constructed |
| `empty_cage` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `frozen_core` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `fusion_hammer` | Relic — Boss | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_fusion_hammer_no_smith_plus_energy` | Constructed |
| `holy_water` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_pure_water_phase` | Constructed |
| `hovering_kite` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `inserter` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `mark_of_pain` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `nuclear_battery` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `pandoras_box` | Relic — Boss | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_pandoras_box_starter_card_transform` | Constructed |
| `philosophers_stone` | Relic — Boss | [m27_energy_law.rs](../../crates/spire-core/src/items/tests/m27_energy_law.rs) | `K` (common); `H`: `m27_philosophers_stone_strength_and_energy` | Constructed |
| `ring_of_the_serpent` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `runic_cube` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `runic_dome` | Relic — Boss | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_runic_dome_hide_enemy_intents` | Constructed |
| `runic_pyramid` | Relic — Boss | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_runic_pyramid_retain_hand` | Constructed |
| `sacred_bark` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `slavers_collar` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `snecko_eye` | Relic — Boss | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_snecko_eye_confused_on_draw` | Constructed |
| `sozu` | Relic — Boss | [m27_energy_law.rs](../../crates/spire-core/src/items/tests/m27_energy_law.rs) | `K` (common); `E`: `m27_g1_sozu_energy_was_missing`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `tiny_house` | Relic — Boss | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_tiny_house_equip_reward_bundle` | Constructed |
| `velvet_choker` | Relic — Boss | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_velvet_choker_six_play_limit` | Constructed |
| `violet_lotus` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `wrist_blade` | Relic — Boss | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |

### Shop (20)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `the_abacus` | Relic — Shop | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_the_abacus_shuffle_block` | Constructed |
| `brimstone` | Relic — Shop | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `cauldron` | Relic — Shop | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_cauldron_five_potion_offers` | Constructed |
| `chemical_x` | Relic — Shop | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `clockwork_souvenir` | Relic — Shop | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_clockwork_souvenir_start_artifact`; Main-10 review fix: `m27_review_i2_mutagenic_artifact_blocks_strength_down` (m27_relics_turns.rs) | Constructed |
| `dollys_mirror` | Relic — Shop | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_dollys_mirror_duplicate_card_instance` | Constructed |
| `frozen_eye` | Relic — Shop | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_frozen_eye_draw_order_visible` | Constructed |
| `hand_drill` | Relic — Shop | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_hand_drill_block_break_vulnerable` | Constructed |
| `medical_kit` | Relic — Shop | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_medical_kit_status_play_exhaust` | Constructed |
| `melange` | Relic — Shop | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `membership_card` | Relic — Shop | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_membership_card_shop_price_half` | Constructed |
| `orange_pellets` | Relic — Shop | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_orange_pellets_complete_card_trio`; Main-10 review fix: `m27_review_i2_mutagenic_orange_pellets_removes_strength_down` (m27_relics_turns.rs) | Constructed |
| `orrery` | Relic — Shop | [m27_relic_selections.rs](../../crates/spire-core/src/run/tests/m27_relic_selections.rs) | `K` (common); `J`: `m27_orrery_five_card_rewards` | Constructed |
| `prismatic_shard` | Relic — Shop | [m27_potion_rewards.rs](../../crates/spire-core/src/run/tests/m27_potion_rewards.rs) | `K` (common); `C3`: `m27_prismatic_shard_any_colour_card_reward` | Constructed |
| `runic_capacitor` | Relic — Shop | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `sling_of_courage` | Relic — Shop | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_sling_of_courage_elite_trigger_strength` | Constructed |
| `strange_spoon` | Relic — Shop | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_strange_spoon_eligible_exhaust_save` | Constructed |
| `toolbox` | Relic — Shop | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_toolbox_three_colorless_choose` | Constructed |
| `twisted_funnel` | Relic — Shop | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted` | Constructed |
| `lees_waffle` | Relic — Shop | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_lees_waffle_max_hp_full_heal`; Main-10 review fix: `m27_review_i3a_lees_waffle_max_hp_with_mark_of_bloom` (m27_relic_lifecycle.rs) | Constructed |

### Special (19)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `bloody_idol` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_bloody_idol_gold_gain_heal_five` | Constructed |
| `circlet` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_circlet_rare_pool_fallback` | Constructed |
| `cultist_headpiece` | Relic — Special | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_cultist_headpiece_speech_only` | Constructed |
| `enchiridion` | Relic — Special | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_enchiridion_class_power_zero` | Constructed |
| `face_of_cleric` | Relic — Special | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_face_of_cleric_victory_max_hp`; Main-10 review fix: `m27_review_i3b_magic_flower_scales_victory_heal` (m27_relics_turns.rs) | Constructed |
| `golden_idol` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_golden_idol_stored_gold_bonus` | Constructed |
| `gremlin_visage` | Relic — Special | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_gremlin_visage_player_weak` | Constructed |
| `mark_of_bloom` | Relic — Special | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters`; Main-10 review fix: `m27_review_i3a_lees_waffle_max_hp_with_mark_of_bloom` (m27_relic_lifecycle.rs) | Constructed |
| `mutagenic_strength` | Relic — Special | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_mutagenic_strength_first_turn_strength`; Main-10 review fix: `m27_review_i2_mutagenic_first_turn_shows_strength_down` (m27_relics_turns.rs), `m27_review_i2_mutagenic_plain_first_end_turn_removes_three_strength` (m27_relics_turns.rs) | Constructed |
| `necronomicon` | Relic — Special | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_necronomicon_first_attack_repeat` | Constructed |
| `neows_lament` | Relic — Special | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `nilrys_codex` | Relic — Special | [m27_relics_structural.rs](../../crates/spire-core/src/items/tests/m27_relics_structural.rs) | `K` (common); `J`: `m27_nilrys_codex_choose_one_of_three` | Constructed |
| `nloths_gift` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_nloths_gift_rare_chance_tripled` | Constructed |
| `nloths_hungry_face` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_nloths_hungry_face_next_chest_relic_removal` | Constructed |
| `odd_mushroom` | Relic — Special | [m27_relics_play.rs](../../crates/spire-core/src/items/tests/m27_relics_play.rs) | `K` (common); `C2`: `m27_odd_mushroom_vulnerable_damage` | Constructed |
| `red_mask` | Relic — Special | [m27_existing_relics.rs](../../crates/spire-core/src/items/tests/m27_existing_relics.rs) | `K` (common); `E`: `m27_existing_relics_equal_accepted`; `E`: `m27_shared_accepted_relics_all_characters` | Constructed |
| `spirit_poop` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_spirit_poop_score_minus_one` | Constructed |
| `ssserpent_head` | Relic — Special | [m27_relics_run.rs](../../crates/spire-core/src/run/tests/m27_relics_run.rs) | `K` (common); `C3`: `m27_ssserpent_head_event_room_gold_fifty` | Constructed |
| `warped_tongs` | Relic — Special | [m27_relics_turns.rs](../../crates/spire-core/src/items/tests/m27_relics_turns.rs) | `K` (common); `C1`: `m27_warped_tongs_postdraw_upgrade` | Constructed |

## Potions

### Shared (30)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `fire_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_fire_potion_ironclad` | Constructed |
| `fruit_juice` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_fruit_juice_ironclad` | Constructed |
| `block_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_block_potion_ironclad` | Constructed |
| `energy_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_energy_potion_ironclad` | Constructed |
| `strength_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_strength_potion_ironclad` | Constructed |
| `swift_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_swift_potion_ironclad` | Constructed |
| `weak_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_weak_potion_ironclad` | Constructed |
| `fear_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_fear_potion_ironclad` | Constructed |
| `dexterity_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_dexterity_potion_ironclad` | Constructed |
| `ancient_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_ancient_potion_ironclad` | Constructed |
| `smoke_bomb` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_smoke_bomb_ironclad` | Constructed |
| `attack_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_attack_potion_ironclad` | Constructed |
| `skill_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_skill_potion_ironclad` | Constructed |
| `power_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_power_potion_ironclad` | Constructed |
| `colorless_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_colorless_potion_ironclad` | Constructed |
| `flex_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_flex_potion_ironclad` | Constructed |
| `speed_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_speed_potion_ironclad` | Constructed |
| `regen_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_regen_potion_ironclad` | Constructed |
| `liquid_bronze` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_liquid_bronze_ironclad` | Constructed |
| `essence_of_steel` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_essence_of_steel_ironclad` | Constructed |
| `cultist_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_cultist_potion_ironclad` | Constructed |
| `blessing_of_the_forge` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_blessing_of_the_forge_ironclad` | Constructed |
| `snecko_oil` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_snecko_oil_ironclad` | Constructed |
| `fairy_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_fairy_potion_ironclad` | Constructed |
| `explosive_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_explosive_potion_ironclad` | Constructed |
| `gamblers_brew` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_gamblers_brew_ironclad` | Constructed |
| `liquid_memories` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_liquid_memories_ironclad` | Constructed |
| `duplication_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_duplication_potion_ironclad` | Constructed |
| `distilled_chaos` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_distilled_chaos_ironclad` | Constructed |
| `entropic_brew` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_entropic_brew_ironclad`; Main-10 review fix: `m27_review_m5_outside_entropic_brew_draws_per_live_slot` (m27_potion_lifecycle.rs) | Constructed |

### Ironclad (3)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `blood_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_blood_potion_ironclad` | Constructed |
| `elixir` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_elixir_ironclad` | Constructed |
| `heart_of_iron` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_heart_of_iron_ironclad` | Constructed |

### Silent (3)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `poison_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_poison_potion_silent` | Constructed |
| `cunning_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_cunning_potion_silent` | Constructed |
| `ghost_in_a_jar` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_ghost_in_a_jar_silent` | Constructed |

### Defect (3)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `focus_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_focus_potion_defect` | Constructed |
| `potion_of_capacity` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_potion_of_capacity_defect` | Constructed |
| `essence_of_darkness` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_essence_of_darkness_defect` | Constructed |

### Watcher (3)

| Item | Kind | Test module | Test key(s) and name(s) | Grade |
| --- | --- | --- | --- | --- |
| `stance_potion` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_stance_potion_watcher` | Constructed |
| `ambrosia` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_ambrosia_watcher` | Constructed |
| `bottled_miracle` | Potion | [m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs) | `K` (common); `P`: `m27_potion_bottled_miracle_watcher` | Constructed |

## Cross-character matrix (design §15.3)

Key X applies to every shared row: the 139 shared relics and the 30 shared potions. For each
character, `m27_cross_character_shared_relic_deltas` and
`m27_cross_character_shared_potion_deltas` compare the item's public-state delta, against the same
script without it, with Ironclad's delta. The two tests take the identities from the complete
unions and assert the literal 139 and 30 counts. Character-sensitive rows get literal
per-character expectations instead. Those are the +1 energy relics, Slaver's Collar and Ice Cream
(`m27_cross_character_energy_recharge_literals`) and Prismatic Shard
(`m27_cross_character_prismatic_shard_orb_slots`). The card generators are Dead Branch,
Enchiridion, Nilry's Codex, Toolbox, Orrery, Tiny House and the Attack, Skill and Power Potions
(`m27_cross_character_card_generator_pools` and `m27_cross_character_run_card_generator_pools`).
The potion generators are Cauldron, White Beast Statue's offer and Entropic Brew
(`m27_cross_character_cauldron_potion_offer_pools`,
`m27_cross_character_white_beast_statue_potion_offer_pool` and
`m27_cross_character_potion_generator_pools`). The E-key test `m27_shared_accepted_relics_all_characters`
remains for the 22 shared accepted relics. Grade: Constructed.

## Supplied Act 4 Runs (design §15.4)

Key S is the four supplied Runs in
[m27_supplied.rs](../../crates/spire-core/src/run/tests/m27_supplied.rs). Every set holds the same
20 combat relics (Akabeko, Anchor, Bag of Marbles, Bag of Preparation, Bronze Scales, The Boot,
Oddly Smooth Stone, Orichalcum, Vajra, Preserved Insect, Sling of Courage, Du-Vu Doll, Fossilized
Helix, Thread and Needle, Gremlin Visage, Red Mask, Mutagenic Strength, Tungsten Rod, Ginger and
Turnip), plus Neow's Lament, White Beast Statue and three potions. Per set:

| Set | Further relics with an Act 4 Run trigger |
| --- | --- |
| Ironclad | Black Blood, Ancient Tea Set, Dream Catcher, Girya, Regal Pillow, Eternal Feather, Peace Pipe, Shovel, Meat on the Bone, Face of Cleric, Magic Flower, Golden Idol, Bloody Idol, Ceramic Fish, Molten Egg, Cursed Key, Mark of Pain, Pantograph |
| Silent | Ring of the Serpent, Coffee Dripper, Maw Bank, Meal Ticket, Membership Card, The Courier, Smiling Mask, Black Star, Question Card, Frozen Egg, Toy Ornithopter, Velvet Choker |
| Defect | Frozen Core, Fusion Hammer, Philosopher's Stone, Prismatic Shard, Singing Bowl, Question Card, Lizard Tail, Sacred Bark (and a Fairy potion) |
| Watcher | Holy Water, Busted Crown, Ectoplasm, Runic Dome, Snecko Eye, Mark of the Bloom, Slaver's Collar, Toxic Egg |

The S key is in addition to each row's Constructed key; the per-row tables above keep the
gate-audit key. Not in a supplied set: Burning Blood (it excludes Black Blood in the one Ironclad
Run), Sozu (it blocks the scripted potion purchase), Circlet, Omamori and Darkstone Periapt (no
curse obtain on the route), the §11.3 room laws, and the equip-only relics that were not obtained
on the route. The [reconciliation](m27-items-reconciliation.md#supplied-act-4-runs-x2-design-154)
records the rulings and bounds.

## Item sites

The gate-audit site table `updated-sites-final.tsv` in execution evidence under
`main-v1/gate-audit-v1/` has 806 rows. Its 639 H0 rows are 514 MATCHED, 104 MATCHED-CHANGED and
21 REMOVED (all moved, none dropped); its 167 NEW rows are A 87, I 51, P 17, C 4, S 4, O 3 and
Debug 1. The triage in `main-v1/gate-audit-v1/site-triage.md` records 51 frozen-reachable sites:
38 unchanged or Complete-gated, 11 reachable-identical and 2 disclosed corrections (G4 and G10),
with 0 real frozen changes and 0 combat-v1/M9/M10 public observation changes. The
reconciliation's [Gate audit (design §6.3)](m27-items-reconciliation.md#gate-audit-design-63)
section summarizes the triage.
