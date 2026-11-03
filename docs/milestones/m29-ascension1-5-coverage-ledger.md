# M29 Ascension 1–5 coverage ledger

**Status:** source-complete candidate (M29d, the M29 close). This ledger accompanies the [M29 profile](m29-ascension1-5.md) and its [reconciliation](m29-ascension1-5-reconciliation.md). It maps every row of the M29 ascension inventory (planning record, outside the repository) to the native Rust tests that pin it, for M29a1, a2, b, c and d (which includes the planned e1, e2 and f). Test results are in the chunk receipts; this page is the map, not a run record.

**Evidence grades:** **Run** means the value/rule is checked on a Standard Run at A1–A5; **Combat live** means the check is at or below the species ceiling and is eligible for a live combat; **Table** means a per-species value check above its ceiling; **Traversal** means the M29d runner receipt: `examples/m28_traversal.rs`, 25 seeds × 4 characters per level and mode.

Source inputs: the M29 ascension inventory v2 and plan erratum v3 (planning records), and the reviewed M29b, M29c and M29d spec sheets (static source checks). Test names are repository-relative `file::fn` and were checked against source declarations on base `4546d65e` plus the M29d changes.

## 1. Run rules (inventory §1)

| Check | Expected behavior | Test pin(s) | Grade |
|---|---|---|---|
| A1 elite quota | Binary32 / Java rounding; A1 elite target uses `n × 0.08f × 1.6f`, other quotas unchanged. | `crates/spire-core/src/run/tests/m29_run_rules.rs::m29_java_round_f32_edges`<br>`crates/spire-core/src/run/tests/m29_run_rules.rs::m29_a1_elite_quota_table`<br>`crates/spire-core/src/run/tests/m29_run_rules.rs::m29_a1_elite_placement_on_generated_maps_acts_1_3` | Run + table |
| A1 Act 4 map | Fixed Act 4 map has no room quota. | `crates/spire-core/src/run/tests/m29_run_rules.rs::m29_act4_map_unchanged_by_ascension` | Run |
| A5 fresh-act heal | At 1→2, 2→3, 3→4, heal rounded 75% of missing HP; ordinary heal hooks apply. | `crates/spire-core/src/run/tests/m29_run_rules.rs::m29_a5_heal_transitions_1_2_2_3_3_4_per_character`<br>`crates/spire-core/src/run/tests/m29_run_rules.rs::m29_a5_heal_respects_heal_hooks`<br>`crates/spire-core/src/run/tests/m29_run_rules.rs::m29_a4_heals_full` | Run |
| No heal at Act 3 Proceed / after Heart | No transition heal on either excluded edge. | `crates/spire-core/src/run/tests/m29_run_rules.rs::m29_no_heal_at_act3_proceed_or_after_heart` | Run |

## 1a. Shared combat ascension owner (M29a2)

| Check | Test pin(s) | Grade |
|---|---|---|
| Single combat ascension field; legacy per-species memories folded without changing their values. | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_combat_ascension_owner_single_source`<br>`crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_legacy_species_unchanged_after_fold` | Combat live |
| Spawned / split children inherit combat ascension; split call sites and Awakened rebirth use owner. | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_spawned_and_split_children_read_combat_ascension`<br>`crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_split_call_site_reads_combat_ascension`<br>`crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_awakened_rebirth_reads_the_owner` | Combat live |
| Runtime validator reads combat ascension. | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_live_validator_reads_combat_ascension` | Combat live |
| Static formation/spawn closure ceilings and no setup RNG on rejection. | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_spawn_closure_gates_encounter`<br>`crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_playable_ceiling_rejects_above`<br>`crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension` | Combat live / Table |
| Event combat gate returns typed M32 rejection above a ceiling. | `crates/spire-core/src/run/tests/m29_a2_event_gate.rs::m29_event_fight_above_ceiling_names_m32` | Run |

## 2. Monster A1–A5 band (62 inventory checks)

Each row is one species × threshold check. All thresholds here are within the accepted species ceiling, so the grade is Combat live. Dedicated Standard Run witnesses are listed after the table.

| # | Species | Threshold | Value(s) | Test function(s) | Grade | Ceiling |
|---:|---|---:|---|---|---|---:|
| 1 | Acid Slime L | A2 | Spit 11→12; Tackle 16→18 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_acid_slime_l_ascension_values` | Combat live | 16 |
| 2 | Acid Slime M | A2 | Spit 7→8; Tackle 10→12 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_acid_slime_m_ascension_values` | Combat live | 16 |
| 3 | Acid Slime S | A2 | Tackle 3→4 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_acid_slime_s_ascension_values` | Combat live | 16 |
| 4 | Spike Slime L | A2 | Flame Tackle 16→18 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_l_ascension_values` | Combat live | 16 |
| 5 | Spike Slime M | A2 | Flame Tackle 8→10 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_m_ascension_values` | Combat live | 16 |
| 6 | Spike Slime S | A2 | Tackle 5→6 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_s_ascension_values` | Combat live | 20 |
| 7 | Red Louse | A2 | Bite range 5–7→6–8 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_red_ascension_values`<br>`crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_bite_and_curl_up_ranges` | Combat live | 16 |
| 8 | Green Louse | A2 | Bite range 5–7→6–8 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_green_ascension_values`<br>`crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_bite_and_curl_up_ranges` | Combat live | 16 |
| 9 | Cultist | A2 | Ritual Strength 3→4 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_cultist_ascension_values` | Combat live | 20 |
| 10 | Jaw Worm | A2 | Bellow Strength 3→4; Chomp 11→12 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_jaw_worm_ascension_values` | Combat live | 20 |
| 11 | Fungi Beast | A2 | Grow Strength 3→4 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_fungi_beast_ascension_values` | Combat live | 20 |
| 12 | Looter | A2 | Swipe 10→11; Lunge 12→14 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_looter_ascension_values` | Combat live | 20 |
| 13 | Fat Gremlin | A2 | Smash 4→5 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_fat_ascension_values` | Combat live | 16 |
| 14 | Sneaky Gremlin | A2 | Scratch 9→10 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_sneaky_ascension_values` | Combat live | 20 |
| 15 | Shield Gremlin | A2 | Bash 6→8 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_shield_ascension_values` | Combat live | 20 |
| 16 | Mad Gremlin | A2 | Scratch 4→5 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_mad_ascension_values` | Combat live | 20 |
| 17 | Gremlin Wizard | A2 | Blast 25→30 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_wizard_ascension_values` | Combat live | 16 |
| 18 | Gremlin Nob | A3 | Skull Bash 6→8; Bull Rush 14→16 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_nob_ascension_values` | Combat live | 20 |
| 19 | Lagavulin | A3 | Attack 18→20 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_lagavulin_ascension_values` | Combat live | 20 |
| 20 | Sentry | A3 | Beam 9→10 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_sentry_ascension_values` | Combat live | 20 |
| 21 | Hexaghost | A4 | Fire Tackle 5→6; Inferno 2→3 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_hexaghost_ascension_values` | Combat live | 20 |
| 22 | Slime Boss | A4 | Tackle 9→10; Slam 35→38 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slime_boss_ascension_values` | Combat live | 20 |
| 23 | Guardian | A4 | Fierce Bash 32→36; Roll 9→10 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_guardian_ascension_values` | Combat live | 20 |
| 24 | Slaver Blue | A2 | Stab 12→13; Rake 7→8 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slaver_blue_ascension_values` | Combat live | 20 |
| 25 | Slaver Red | A2 | Stab 13→14; Scrape 8→9 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slaver_red_ascension_values` | Combat live | 16 |
| 26 | Bandit Bear | A2 | Maul 18→20; Lunge 9→10 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_bear_ascension_values` | Combat live | 20 |
| 27 | Romeo / Bandit Leader | A2 | Slash 15→17; Agonize 10→12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_leader_ascension_values` | Combat live | 16 |
| 28 | Pointy / Bandit Pointy | A2 | Pointy 5→6 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_pointy_ascension_values` | Combat live | 20 |
| 29 | Byrd | A2 | Peck hits 5→6; Swoop 12→14 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_byrd_ascension_values` | Combat live | 20 |
| 30 | Centurion | A2 | Slash 12→14; Fury 6→7 (3 hits) | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_centurion_ascension_values` | Combat live | 20 |
| 31 | Mystic | A2 | Attack 8→9; Strength 2→3 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mystic_ascension_values` | Combat live | 16 |
| 32 | Chosen | A2 | Zap 18→21; Debilitate 10→12; Poke 5→6 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_chosen_ascension_values` | Combat live | 16 |
| 33 | Mugger | A2 | Swipe 10→11; Big Swipe 16→18 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mugger_ascension_values` | Combat live | 20 |
| 34 | Shelled Parasite | A2 | Double Strike 6→7; Fell 18→21; Life Suck 10→12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_shelled_parasite_ascension_values` | Combat live | 16 |
| 35 | Snake Plant | A2 | Chomp 7→8 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_snake_plant_ascension_values` | Combat live | 16 |
| 36 | Snecko | A2 | Bite 15→18; Tail Whip 8→10 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_snecko_ascension_values` | Combat live | 16 |
| 37 | Spheric Guardian | A2 | Attack 10→11 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spheric_guardian_ascension_values` | Combat live | 20 |
| 38 | Book of Stabbing | A3 | Multi-Stab 6→7 per hit; Big Stab 21→24 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_book_of_stabbing_ascension_values` | Combat live | 17 |
| 39 | Gremlin Leader | A3 | Encourage Strength 3→4 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_leader_ascension_values` | Combat live | 20 |
| 40 | Taskmaster | A3 | Whip Wounds 1→2 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_taskmaster_ascension_values` | Combat live | 17 |
| 41 | Bronze Automaton | A4 | Flail 7→8; Beam 45→50; Strength 3→4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bronze_automaton_ascension_values` | Combat live | 18 |
| 42 | Champ | A4 | Slash 16→18; Slap 12→14; Strength 2→3 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Combat live | 18 |
| 43 | Collector | A4 | Rake 18→21; Strength 3→4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 44 | Darkling | A2 | Chomp 8→9; Nip range 7–11→9–13 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_darkling_ascension_values` | Combat live | 16 |
| 45 | Exploder | A2 | Attack 9→11 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_exploder_ascension_values` | Combat live | 20 |
| 46 | Maw | A2 | Slam 25→30 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_maw_ascension_values` | Combat live | 20 |
| 47 | Orb Walker | A2 | Claw 15→16; Laser 10→11 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_orb_walker_ascension_values` | Combat live | 20 |
| 48 | Repulsor | A2 | Attack 11→13 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_repulsor_ascension_values` | Combat live | 20 |
| 49 | Spiker | A2 | Thorns 3→4; Attack 7→9 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spiker_ascension_values` | Combat live | 20 |
| 50 | Spire Growth | A2 | Tackle 16→18; Smash 22→25 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_growth_ascension_values` | Combat live | 16 |
| 51 | Transient | A2 | Attack sequence 30…90→40…100 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_transient_ascension_values` | Combat live | 20 |
| 52 | Writhing Mass | A2 | Bases 32/7/15/10→38/9/16/12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_writhing_mass_ascension_values` | Combat live | 20 |
| 53 | Giant Head | A3 | It Is Time sequence 30…60→40…70 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_giant_head_ascension_values` | Combat live | 20 |
| 54 | Nemesis | A3 | Tri Attack 6→7 per hit | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_nemesis_ascension_values` | Combat live | 20 |
| 55 | Reptomancer | A3 | Snake Strike 13→16; Bite 30→34 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_reptomancer_ascension_values` | Combat live | 20 |
| 56 | Awakened One | A4 | Pre-battle Strength +2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_awakened_one_ascension_values` | Combat live | 20 |
| 57 | Deca | A4 | Beam 10→12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_deca_ascension_values` | Combat live | 18 |
| 58 | Donu | A4 | Beam 10→12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_donu_ascension_values` | Combat live | 20 |
| 59 | Time Eater | A4 | Reverb 7→8; Head Slam 26→32 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_time_eater_ascension_values` | Combat live | 18 |
| 60 | Spire Shield | A3 | Bash 12→14; Smash 34→38 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_shield_ascension_values` | Combat live | 17 |
| 61 | Spire Spear | A3 | Burn Strike 5→6; Skewer hits 3→4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_spear_ascension_values` | Combat live | 17 |
| 62 | Corrupt Heart | A4 | Echo 40→45; Blood Shots hits 12→15 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_corrupt_heart_ascension_values`<br>`crates/spire-core/src/combat/tests/m29_d_values.rs::m29_heart_blood_shots_live_turn` | Combat live | 20 |

**Standard Run witnesses for A1–A5:** `crates/spire-core/src/run/tests/m29_d_close.rs::m29_run_combats_use_run_ascension`; `crates/spire-core/src/run/tests/m29_d_close.rs::m29_act3_boss_and_act4_a1_to_a5`; `crates/spire-core/src/run/tests/m29_c_event_fights.rs::m29_event_fights_landed_species_accepted_a1_to_a5`; `crates/spire-core/src/run/tests/m29_c_event_fights.rs::m29_colosseum_both_fights_a1_to_a5`; `crates/spire-core/src/run/tests/m29_c_event_fights.rs::m29_mind_bloom_act1_bosses_a1_to_a5`; `crates/spire-core/src/run/tests/m29_c_event_fights.rs::m29_dead_adventurer_fights_a1_to_a5`; `crates/spire-core/src/run/tests/m29_d_close.rs::m29_d_event_fights_accepted_a1_to_a5`; `crates/spire-core/src/run/tests/m29_d_close.rs::m29_act2_bosses_a4_each_character`; `crates/spire-core/src/run/tests/m29_d_close.rs::m29_act4_shield_spear_a3_and_heart_a4`. These Run tests establish Run routing/entry; species value rows still point to the per-species test.

## 3. Monster A6–A20 high-tier value rows

Inventory §3 declares 147 checks. The five reviewed chunk sheets yield 142 individually named A6–A20 threshold/value rows below. Values and thresholds are taken from the sheets; each row maps to the species ladder test. Grade follows the brief: Combat live at or below the species ceiling, Table above it. Fields whose numeric payload belongs to an M32 behavior row are marked `†` and are also listed in §3a.

| # | Species | Threshold | Field | Value | Test function | Grade | Ceiling |
|---:|---|---:|---|---|---|---|---:|
| 1 | Acid Slime L | A7 | hp_range | 68–72 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_acid_slime_l_ascension_values` | Combat live | 16 |
| 2 | Acid Slime M | A7 | hp_range | 29–34 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_acid_slime_m_ascension_values` | Combat live | 16 |
| 3 | Acid Slime S | A7 | hp_range | 9–13 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_acid_slime_s_ascension_values` | Combat live | 16 |
| 4 | Spike Slime L | A7 | hp_range | 67–73 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_l_ascension_values` | Combat live | 16 |
| 5 | Spike Slime L | A17 | frail amount on Lick | 3 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_l_ascension_values` | Table | 16 |
| 6 | Spike Slime M | A7 | hp_range | 29–34 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_m_ascension_values` | Combat live | 16 |
| 7 | Spike Slime S | A7 | hp_range | 11–15 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_spike_slime_s_ascension_values` | Combat live | 20 |
| 8 | Red Louse | A7 | hp_range | 11–16 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_red_ascension_values` | Combat live | 16 |
| 9 | Red Louse | A7 | curl_up amount range | 4–8 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_red_ascension_values` | Combat live | 16 |
| 10 | Red Louse | A17 | curl_up amount range | 9–12 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_red_ascension_values` | Table | 16 |
| 11 | Red Louse | A17 | grow.strength | 4 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_red_ascension_values` | Table | 16 |
| 12 | Green Louse | A7 | hp_range | 12–18 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_green_ascension_values` | Combat live | 16 |
| 13 | Green Louse | A7 | curl_up amount range | 4–8 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_green_ascension_values` | Combat live | 16 |
| 14 | Green Louse | A17 | curl_up amount range | 9–12 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_louse_green_ascension_values` | Table | 16 |
| 15 | Cultist | A7 | hp_range | 50–56 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_cultist_ascension_values` | Combat live | 20 |
| 16 | Cultist | A17 | ritual.amount | 5 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_cultist_ascension_values` | Combat live | 20 |
| 17 | Jaw Worm | A7 | hp_range | 42–46 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_jaw_worm_ascension_values` | Combat live | 20 |
| 18 | Jaw Worm | A17 | bellow.strength | 5 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_jaw_worm_ascension_values` | Combat live | 20 |
| 19 | Jaw Worm | A17 | bellow.block | 9 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_jaw_worm_ascension_values` | Combat live | 20 |
| 20 | Fungi Beast | A7 | hp_range | 24–28 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_fungi_beast_ascension_values` | Combat live | 20 |
| 21 | Fungi Beast | A17 | grow.strength | 5 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_fungi_beast_ascension_values` | Combat live | 20 |
| 22 | Looter | A7 | hp_range | 46–50 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_looter_ascension_values` | Combat live | 20 |
| 23 | Looter | A17 | thievery.amount / gold per steal | 20 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_looter_ascension_values` | Combat live | 20 |
| 24 | Mad Gremlin | A7 | hp_range | 21–25 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_mad_ascension_values` | Combat live | 20 |
| 25 | Mad Gremlin | A17 | angry.amount | 2 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_mad_ascension_values` | Combat live | 20 |
| 26 | Sneaky Gremlin | A7 | hp_range | 11–15 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_sneaky_ascension_values` | Combat live | 20 |
| 27 | Fat Gremlin | A7 | hp_range | 14–18 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_fat_ascension_values` | Combat live | 16 |
| 28 | Fat Gremlin | A17 | frail.amount on Smash † | 1 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_fat_ascension_values` | Table | 16 |
| 29 | Shield Gremlin | A7 | hp_range | 13–17 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_shield_ascension_values` | Combat live | 20 |
| 30 | Shield Gremlin | A7 | gremlin_protect block | 8 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_shield_ascension_values` | Combat live | 20 |
| 31 | Shield Gremlin | A17 | gremlin_protect block | 11 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_shield_ascension_values` | Combat live | 20 |
| 32 | Gremlin Wizard | A7 | hp_range | 22–26 | `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_gremlin_wizard_ascension_values` | Combat live | 16 |
| 33 | Gremlin Nob | A8 | hp_range | 85–90 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_nob_ascension_values` | Combat live | 20 |
| 34 | Gremlin Nob | A18 | bellow.enrage amount | 3 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_nob_ascension_values` | Combat live | 20 |
| 35 | Lagavulin | A8 | hp_range | 112–115 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_lagavulin_ascension_values` | Combat live | 20 |
| 36 | Lagavulin | A18 | siphon_soul.dexterity loss | -2 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_lagavulin_ascension_values` | Combat live | 20 |
| 37 | Lagavulin | A18 | siphon_soul.strength loss | -2 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_lagavulin_ascension_values` | Combat live | 20 |
| 38 | Sentry | A8 | hp_range | 39–45 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_sentry_ascension_values` | Combat live | 20 |
| 39 | Sentry | A18 | bolt.dazed count | 3 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_sentry_ascension_values` | Combat live | 20 |
| 40 | Hexaghost | A9 | hp_range | 264 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_hexaghost_ascension_values` | Combat live | 20 |
| 41 | Hexaghost | A19 | inflame.strength | 3 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_hexaghost_ascension_values` | Combat live | 20 |
| 42 | Hexaghost | A19 | sear.burn count | 2 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_hexaghost_ascension_values` | Combat live | 20 |
| 43 | Slime Boss | A9 | hp_range | 150 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slime_boss_ascension_values` | Combat live | 20 |
| 44 | Slime Boss | A19 | goop_spray.slimed count | 5 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slime_boss_ascension_values` | Combat live | 20 |
| 45 | Guardian | A9 | hp_range | 250 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_guardian_ascension_values` | Combat live | 20 |
| 46 | Guardian | A9 | mode_shift.threshold | initial 35 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_guardian_ascension_values` | Combat live | 20 |
| 47 | Guardian | A19 | mode_shift.threshold | initial 40 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_guardian_ascension_values` | Combat live | 20 |
| 48 | Guardian | A19 | sharp_hide amount | 4 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_guardian_ascension_values` | Combat live | 20 |
| 49 | Slaver Blue | A7 | hp_range | 48–52 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slaver_blue_ascension_values` | Combat live | 20 |
| 50 | Slaver Blue | A17 | rake.weak | 2 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slaver_blue_ascension_values` | Combat live | 20 |
| 51 | Slaver Red | A7 | hp_range | 48–52 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slaver_red_ascension_values` | Combat live | 16 |
| 52 | Slaver Red | A17 | scrape.vulnerable | 2 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_slaver_red_ascension_values` | Table | 16 |
| 53 | Book of Stabbing | A8 | hp_range | 168–172 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_book_of_stabbing_ascension_values` | Combat live | 17 |
| 54 | Gremlin Leader | A8 | hp_range | 145–155 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_leader_ascension_values` | Combat live | 20 |
| 55 | Gremlin Leader | A18 | encourage.strength | 5 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_leader_ascension_values` | Combat live | 20 |
| 56 | Gremlin Leader | A18 | encourage.block | 10 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_gremlin_leader_ascension_values` | Combat live | 20 |
| 57 | Taskmaster | A8 | hp_range | 57–64 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_taskmaster_ascension_values` | Combat live | 17 |
| 58 | Taskmaster | A18 | scouring_whip.wound count | 3 | `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_taskmaster_ascension_values` | Table | 17 |
| 59 | BanditPointy | A7 | hp_range | 34 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_pointy_ascension_values` | Combat live | 20 |
| 60 | BanditLeader | A7 | hp_range | 37–41 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_leader_ascension_values` | Combat live | 16 |
| 61 | BanditLeader | A17 | agonizing_slash.weak | 3 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_leader_ascension_values` | Table | 16 |
| 62 | BanditBear | A7 | hp_range | 40–44 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_bear_ascension_values` | Combat live | 20 |
| 63 | BanditBear | A17 | bear_hug.dexterity | -4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bandit_bear_ascension_values` | Combat live | 20 |
| 64 | Byrd | A7 | hp_range | 26–33 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_byrd_ascension_values` | Combat live | 20 |
| 65 | Byrd | A17 | flight.amount | 4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_byrd_ascension_values`<br>`crates/spire-core/src/combat/tests/m29_d_values.rs::m29_byrd_flight_refill_live` | Combat live | 20 |
| 66 | Centurion | A7 | hp_range | 78–83 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_centurion_ascension_values` | Combat live | 20 |
| 67 | Centurion | A17 | centurion_protect.block | 20 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_centurion_ascension_values` | Combat live | 20 |
| 68 | Mystic | A7 | hp_range | 50–58 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mystic_ascension_values` | Combat live | 16 |
| 69 | Mystic | A17 | mystic_attack damage base | 9 (reassigned unchanged) | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mystic_ascension_values` | Table | 16 |
| 70 | Mystic | A17 | mystic_buff.strength | 4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mystic_ascension_values` | Table | 16 |
| 71 | Mystic | A17 | mystic_heal.hp | 20 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mystic_ascension_values` | Table | 16 |
| 72 | Mystic | A17 | heal-selection missing-HP cutoff † | >20 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mystic_ascension_values` | Table | 16 |
| 73 | Chosen | A7 | hp_range | 98–103 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_chosen_ascension_values` | Combat live | 16 |
| 74 | Mugger | A7 | hp_range | 50–54 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mugger_ascension_values` | Combat live | 20 |
| 75 | Mugger | A17 | thievery.amount / mugger_mug and mugger_lunge gold cap | 20 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mugger_ascension_values` | Combat live | 20 |
| 76 | Mugger | A17 | smoke_bomb.block | 17 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_mugger_ascension_values` | Combat live | 20 |
| 77 | ShelledParasite | A7 | hp_range | 70–75 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_shelled_parasite_ascension_values` | Combat live | 16 |
| 78 | SnakePlant | A7 | hp_range | 78–82 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_snake_plant_ascension_values` | Combat live | 16 |
| 79 | Snecko | A7 | hp_range | 120–125 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_snecko_ascension_values` | Combat live | 16 |
| 80 | Snecko | A17 | tail_whip.weak | 2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_snecko_ascension_values` | Table | 16 |
| 81 | SphericGuardian | A17 | sphere_activate.block | 35 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spheric_guardian_ascension_values` | Combat live | 20 |
| 82 | BronzeAutomaton | A9 | hp_range | 320 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bronze_automaton_ascension_values` | Combat live | 18 |
| 83 | BronzeAutomaton | A9 | boost.block | 12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bronze_automaton_ascension_values` | Combat live | 18 |
| 84 | BronzeOrb | A9 | hp_range | 54–60 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_bronze_orb_ascension_values` | Combat live | 20 |
| 85 | Champ | A9 | hp_range | 440 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Combat live | 18 |
| 86 | Champ | A19 | gloat.strength | 4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Table | 18 |
| 87 | Champ | A19 | anger.strength | 12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Table | 18 |
| 88 | Champ | A9 | stance.block | 18 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Combat live | 18 |
| 89 | Champ | A19 | stance.block | 20 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Table | 18 |
| 90 | Champ | A9 | stance.metallicize | 6 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Combat live | 18 |
| 91 | Champ | A19 | stance.metallicize | 7 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Table | 18 |
| 92 | Champ | A19 | forge-selection roll cutoff † | roll ≤30 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_champ_ascension_values` | Table | 18 |
| 93 | Collector | A9 | hp_range | 300 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 94 | Collector | A19 | buff.strength | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 95 | Collector | A9 | buff.block | 18 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 96 | Collector | A19 | buff.block | 23 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 97 | Collector | A19 | mega_debuff.weak | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 98 | Collector | A19 | mega_debuff.vulnerable | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 99 | Collector | A19 | mega_debuff.frail | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_collector_ascension_values` | Combat live | 20 |
| 100 | TorchHead | A9 | hp_range | 40–45 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_torch_head_ascension_values` | Combat live | 20 |
| 101 | Darkling | A7 | hp_range | 50–59 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_darkling_ascension_values` | Combat live | 16 |
| 102 | Darkling | A17 | darkling_harden.strength † | +2 Strength | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_darkling_ascension_values` | Table | 16 |
| 103 | Exploder | A7 | hp_range | 30–35 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_exploder_ascension_values` | Combat live | 20 |
| 104 | Maw | A17 | maw_drool.strength | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_maw_ascension_values` | Combat live | 20 |
| 105 | Maw | A17 | maw_roar.weak duration | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_maw_ascension_values` | Combat live | 20 |
| 106 | Maw | A17 | maw_roar.frail duration | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_maw_ascension_values` | Combat live | 20 |
| 107 | Orb Walker | A7 | hp_range | 92–102 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_orb_walker_ascension_values` | Combat live | 20 |
| 108 | Orb Walker | A17 | strength_up.amount | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_orb_walker_ascension_values` | Combat live | 20 |
| 109 | Repulsor | A7 | hp_range | 31–38 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_repulsor_ascension_values` | Combat live | 20 |
| 110 | Spiker | A7 | hp_range | 44–60 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spiker_ascension_values` | Combat live | 20 |
| 111 | Spiker | A17 | starting_thorns.amount | 7 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spiker_ascension_values` | Combat live | 20 |
| 112 | Spire Growth | A7 | hp_range | 190 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_growth_ascension_values` | Combat live | 16 |
| 113 | Spire Growth | A17 | growth_constrict.amount | 12 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_growth_ascension_values` | Table | 16 |
| 114 | Transient | A17 | fading.amount | 6 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_transient_ascension_values` | Combat live | 20 |
| 115 | Writhing Mass | A7 | hp_range | 175 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_writhing_mass_ascension_values` | Combat live | 20 |
| 116 | Giant Head | A8 | hp_range | 520 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_giant_head_ascension_values` | Combat live | 20 |
| 117 | Giant Head | A18 | head_time.initial_count | 4 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_giant_head_ascension_values` | Combat live | 20 |
| 118 | Nemesis | A8 | hp_range | 200 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_nemesis_ascension_values` | Combat live | 20 |
| 119 | Nemesis | A18 | nemesis_burn.cards | 5 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_nemesis_ascension_values` | Combat live | 20 |
| 120 | Reptomancer | A8 | hp_range | 190–200 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_reptomancer_ascension_values` | Combat live | 20 |
| 121 | Reptomancer | A18 | repto_spawn.dagger_count | 2 per Spawn, subject to four-Dagger cap | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_reptomancer_ascension_values` | Combat live | 20 |
| 122 | Awakened One | A9 | hp_range | 320 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_awakened_one_ascension_values` | Combat live | 20 |
| 123 | Awakened One | A19 | awa_prebattle.regenerate | 15 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_awakened_one_ascension_values` | Combat live | 20 |
| 124 | Awakened One | A19 | awa_prebattle.curiosity | 2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_awakened_one_ascension_values` | Combat live | 20 |
| 125 | Awakened One | A9 | awa_rebirth.max_hp | 320 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_awakened_one_ascension_values` | Combat live | 20 |
| 126 | Time Eater | A9 | hp_range | 480 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_time_eater_ascension_values` | Combat live | 18 |
| 127 | Time Eater | A19 | time_ripple.frail † | 1 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_time_eater_ascension_values` | Table | 18 |
| 128 | Time Eater | A19 | time_slam.slimed count † | 2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_time_eater_ascension_values` | Table | 18 |
| 129 | Time Eater | A19 | time_haste.block † | 32 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_time_eater_ascension_values` | Table | 18 |
| 130 | Deca | A9 | hp_range | 265 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_deca_ascension_values` | Combat live | 18 |
| 131 | Deca | A19 | deca_prebattle.artifact | 3 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_deca_ascension_values` | Table | 18 |
| 132 | Deca | A19 | deca_square.plated_armor † | 3 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_deca_ascension_values` | Table | 18 |
| 133 | Donu | A9 | hp_range | 265 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_donu_ascension_values` | Combat live | 20 |
| 134 | Donu | A19 | donu_prebattle.artifact | 3 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_donu_ascension_values` | Combat live | 20 |
| 135 | Spire Shield | A8 | hp_range | 125 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_shield_ascension_values` | Combat live | 17 |
| 136 | Spire Shield | A18 | shield_prebattle.artifact | 2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_shield_ascension_values` | Table | 17 |
| 137 | Spire Shield | A18 | shield_smash.block † | 99 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_shield_ascension_values` | Table | 17 |
| 138 | Spire Spear | A8 | hp_range | 180 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_spear_ascension_values` | Combat live | 17 |
| 139 | Spire Spear | A18 | spear_prebattle.artifact | 2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_spire_spear_ascension_values` | Table | 17 |
| 140 | Corrupt Heart | A9 | hp_range | 800 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_corrupt_heart_ascension_values` | Combat live | 20 |
| 141 | Corrupt Heart | A19 | heart_prebattle.invincible | 200 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_corrupt_heart_ascension_values` | Combat live | 20 |
| 142 | Corrupt Heart | A19 | heart_prebattle.beat_of_death | 2 | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_corrupt_heart_ascension_values` | Combat live | 20 |

## 3a. Behavior deferred to M32

Inventory §3a names 26 species with pending behavior (18 at ceiling 16, four at 17, four at 18). Each behavior row remains M32. Ceiling pins are the explicit formation checks where present plus the all-encounter admission sweep.

| Species | M32 behavior row | Species ceiling | Ceiling test pin(s) |
|---|---|---:|---|
| Acid Slime L | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Acid Slime M | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Acid Slime S | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Spike Slime L | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Spike Slime M | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Red Louse | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Green Louse | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Gremlin Wizard | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Fat Gremlin | A17: Smash adds Frail | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_b_act1_normals.rs::m29_act1_normal_formations_leave_interim` |
| Slaver Red | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings`; `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_c_formations_leave_interim` |
| Romeo / Bandit Leader | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Chosen | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Mystic | A17: two AI / heal-selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Shelled Parasite | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Snake Plant | A17: AI move selection changes | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Snecko | A17: Tail Whip adds Weak | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Darkling | A17: DEFEND_BUFF intent and Harden Strength | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension` |
| Spire Growth | A17: Constrict selection / amount | 16 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Book of Stabbing | A18: Big Stab advances stab counter | 17 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings`; `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_c_formations_leave_interim` |
| Taskmaster | A18: Whip grants Strength | 17 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings`; `crates/spire-core/src/combat/tests/m29_c_elites_bosses.rs::m29_c_formations_leave_interim` |
| Spire Shield | A18: Smash Block becomes 99 | 17 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Spire Spear | A18: Burns go to draw pile | 17 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Champ | A19: Forge cutoff changes | 18 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Bronze Automaton | A19: post-Beam move selection | 18 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Deca | A19: Square adds Plated Armor; DEFEND_BUFF intent | 18 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |
| Time Eater | A19: Ripple Frail, Slam Slimed, Haste Block | 18 | `crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension`; `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_d_formations_ceilings` |

Already-landed behavior is not listed as pending M32: Gremlin Nob A18 cycle and Slaver Blue A17 guard. Slime Boss and Gremlin Leader encounter ceilings remain 16 through child closure; these are not species-level M32 rows.

† Numeric payloads shared with behavior scope: Fat Gremlin Frail, Mystic heal cutoff, Champ Forge cutoff, Darkling Strength, Time Eater three A19 additions, Deca Plated Armor and Spire Shield 99 Block. They are shown in §3 because the spec sheets give a threshold/value row, and in §3a because the behavior itself remains M32.

## 4. M29f close items

| Close item | Test / evidence | Grade |
|---|---|---|
| Public Standard A1–A5 gate, all four characters; no advertised action is Unsupported through Act 1 boss. | `crates/spire-core/src/run/tests/m29_d_close.rs::m29_public_a1_a5_gate` | Run |
| Constructor admission and retargeted default gate; A6 remains rejected. | `crates/spire-core/src/run/tests/m29_run_rules.rs::m29_standard_creation_all_characters_a0_to_a5`<br>`crates/spire-core/src/run/tests/m29_run_rules.rs::m29_standard_rejects_a6_and_above`<br>`crates/spire-core/src/run/tests/m29_run_rules.rs::m29_standard_default_build_admits_a1_to_a5_rejects_a6`<br>`crates/spire-core/src/run/tests/m28_standard.rs::m28_standard_creation_all_characters`<br>`crates/spire-core/tests/m28_standard_gate.rs::m28_standard_a0_is_public_in_a_default_build` | Run |
| Every encounter accepted through its ceiling; above-ceiling setup is typed and species values landed. | `crates/spire-core/src/combat/tests/m29_d_values.rs::m29_all_species_landed`<br>`crates/spire-core/src/combat/tests/m29_a2_owner.rs::m29_every_encounter_every_ascension` | Combat live / Table |
| Event fights accepted and settled at A1–A5 for landed species, including Act 2 bosses and Act 4. | `crates/spire-core/src/run/tests/m29_d_close.rs::m29_d_event_fights_accepted_a1_to_a5`<br>`crates/spire-core/src/run/tests/m29_d_close.rs::m29_act2_bosses_a4_each_character`<br>`crates/spire-core/src/run/tests/m29_d_close.rs::m29_act4_shield_spear_a3_and_heart_a4`<br>`crates/spire-core/src/run/tests/m29_d_close.rs::m29_act3_boss_and_act4_a1_to_a5` | Run |
| Act 3 boss and Act 4 (Shield and Spear, then the Heart) at each of A1–A5 on a public Run; every advertised Act 4 action steps without `Unsupported`. The traversal reaches no Act 4 at any level, the A0 base included, so this constructed Run (supplied HP and keys) is the Act 4 evidence (PM acceptance, review m1). | `crates/spire-core/src/run/tests/m29_d_close.rs::m29_act3_boss_and_act4_a1_to_a5` | Run |
| Full A1–A5 traversal / zero-interim receipt: default build (uniform, reach) and `m29-diagnostics` (uniform, reach, supplied) at each of A1–A5, 0 faults, 0 interim rejections, 0 interim-blocked Runs; A0 control vs base. | M29d runner receipt (`c1-checks/r2-traversal-summary.md` in the M29d evidence) | Traversal |

## 5. Unmapped

None. Two notes on the counts:

- **Inventory total.** Inventory §3 gives "147 checks" as an aggregate of approximate counts (≈110 numeric values and ≈35
  behaviour sites). This ledger is keyed by the reviewed sheet rows instead: 142 numeric threshold rows (§3) and 26 behaviour
  species (§3a). Every value the inventory names in §3 has a §3 or §3a row here.
- **Giant Head A18 countdown.** The first countdown of 4 at A18 is numeric memory set at construction, not a selection change,
  so the Giant Head's ceiling is 20 (inventory §3 lists it numeric; main-senior ruling, M29d plan-check). The Act 3 sheet's
  suggested ceiling of 17 is superseded. `m29_d_formations_ceilings` pins ceiling 20 and
  `m29_giant_head_time_sequence_live` runs the A18 countdown live.

### Mapping counts

- Run rules: 4 named rows mapped.
- Combat ascension owner/admission: 5 named rows mapped.
- §2 A1–A5: 62 / 62 mapped.
- §3 A6–A20: 142 / 142 sheet rows mapped.
- §3a M32: 26 / 26 species rows mapped to a ceiling test.
- M29f close: 6 / 6 items mapped (traversal from the M29d runner receipt).
