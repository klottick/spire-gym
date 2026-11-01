# M26 shared and off-colour card coverage ledger

One row per card kind and per producer pool, with the test keys that cover it and their
grade. Read with the [profile](m26-shared-cards.md), the
[reconciliation](m26-shared-cards-reconciliation.md), the
[design](../superpowers/specs/2026-10-31-m26-shared-cards-design.md) §14.2 and the
[plan](../superpowers/plans/2026-10-31-m26-shared-cards.md) Task M5.

Every key's tests pass on the freeze v2 tree (workspace 3461 passed / 0 failed / 11 ignored;
`m26_` 55 / 0; `tests/m26_catalog.rs` 5 / 0). Expected values are literals from the design
or cited facts, not values computed by the owner under test.

## Keys and grades

| Key | Test file (under `crates/spire-core/`) | Grade |
| --- | --- | --- |
| U | `tests/m26_catalog.rs`, `src/combat/tests/m26_capability.rs` | Constructed |
| K | `src/combat/tests/m26_cards.rs`, `src/combat/tests/m26_corrections.rs` | Constructed |
| W | `src/combat/tests/m26_offcolour_laws.rs` | Constructed |
| X | `src/combat/tests/m26_cross_character.rs` | Constructed |
| G | `src/combat/tests/m26_generation.rs`, `src/combat/tests/m26_pools.rs` | Constructed |
| Y | `src/combat/tests/m26_copies.rs` (helpers in `m26_support.rs`) | Constructed |
| R | `src/run/tests/m26_card_mutations.rs`, `src/run/tests/m26_pools.rs`, `src/run/tests/m26_capability.rs`, `src/combat/tests/m26_autoplay.rs` | Constructed |
| S | `src/run/tests/m26_supplied.rs` | Supplied played |

**Constructed** means an owner checkpoint built directly. **Supplied played** means a
supplied start, then only legal actions through the real owners. There are no Normal played
cases and no Native agreement runs; no outcome or win is a gate.

## Card kinds

| Kind | Count | Keys | Grade | Tests and what they establish |
| --- | --- | --- | --- | --- |
| Ironclad cards (incl. basics) | 75 | U, X, Y, R, S | Constructed; S supplied played | `m26_cross_character_equivalence` (base and upgraded, every character, delta equals the Ironclad delta); `m26_cross_character_own_inert` (overlay inert for own cards); `m26_copies_every_kind`; `m26_mutation_obtain_upgrade_remove` (352 kinds × 4 characters); the Silent, Defect and Watcher supplied decks hold True Grit |
| Silent cards (incl. basics) | 75 | U, X, Y, R, S, W | Constructed; S supplied played | As Ironclad; Alchemize per character in `m26_offcolour_alchemize_all_characters`; the Ironclad, Defect and Watcher supplied decks hold Cloak and Dagger |
| Defect cards (incl. basics) | 75 | U, W, X, Y, R, S | Constructed; S supplied played | As Ironclad, with the orb rows in `m26_cross_character_sensitive_rows` (literal per character), `m26_offcolour_orbs_zero_slots` (F12), `m26_offcolour_orb_passives_after_capacitor`, `m26_offcolour_turn_powers`; the Ironclad, Silent and Watcher supplied decks hold Cold Snap |
| Watcher cards (incl. basics) | 75 | U, W, X, Y, R, S | Constructed; S supplied played | As Ironclad, with `m26_offcolour_stance_mantra_scry`, `m26_offcolour_retain_discard`, `m26_offcolour_fasting_energy_down` (G7), `m26_offcolour_deus_ex_machina_draw`, `m26_offcolour_turn_structure`; the Ironclad, Silent and Defect supplied decks hold Crescendo |
| Ordinary colourless (incl. Bandage Up) | 35 | U, K, X, Y, R, G, S | Constructed; S supplied played | `m26_cards_colourless_values` (Bandage Up base and upgraded); X, Y and R as above; Jack of All Trades, Transmutation and Magnetism in `m26_generation_player_pool`; supplied decks hold Bandage Up, Flash of Steel and Trip |
| Event colourless (Apparition, Bite, J.A.X., Ritual Dagger) | 4 | U, K, X, Y, R, S | Constructed; S supplied played | `m26_cards_colourless_values`; `m26_capability_misc_growth_journal`, `m26_ritual_dagger_growth_tracks_instance_family`, `m26_ritual_dagger_combat_exclusion_frontier`, `m26_ritual_dagger_growth_exclusions_and_persistence` (journal and census); `m26_fatal_fires_on_the_final_darkling` (correction 9); every supplied deck holds Ritual Dagger (misc 15) and Bite, and its growth persists once after Victory |
| Curses (all but Pride) | 13 | U, K, X, Y, R, S | Constructed; S supplied played | `m26_cards_curse_rules`, `m26_cards_end_of_turn_order` (F8), `m26_normality_counts_autoplay_and_late_draw` and `m26_normality_blocks_autoplay` (F9), `m26_pain_stacks_and_kills_once` (F10), `m26_necronomicurse_full_hand_and_repeat` (F11), `m26_generation_master_reality_curse_return_stays_base`; `m26_end_turn_curse_debuffs_last_through_the_next_turn` and `m26_end_turn_self_playing_cards_discard_when_retained` (corrections 7–8, every character); `m26_pain_autoplay_duplicate_and_end_of_turn` (F10); removal rules in `m26_mutation_obtain_upgrade_remove`; supplied Pain, Decay, Doubt, Regret, Normality, Writhe, Shame, Clumsy |
| Generated children (Shiv, Miracle, Smite, Safety, Insight, Through Violence, Beta, Omega, Expunger) | 9 | U, X, Y, G | Constructed | `m26_generated_and_status_every_character` (each child from its producer for every character); `m26_cross_character_equivalence`; `m26_generated_watcher_kinds_match_their_m25_constructor` (correction 2) |
| Statuses (Burn, Dazed, Slimed, Wound, Void) | 5 | U, K, X, Y | Constructed | `m26_generated_and_status_every_character` (literal HP, energy and pile results for every character); `m26_cross_character_equivalence`; a retained Burn discards itself in `m26_end_turn_self_playing_cards_discard_when_retained` |
| Wish choices | 3 | U | Constructed | `m26_identity_complete_unions` (choice-only union = 3); their behaviour is pinned by the accepted M25 Wish tests |
| Pride | 1 | U | Constructed | Excluded from every union (`m26_identity_catalog_rows`, `m26_identity_complete_unions`) |

The 300 character cards, 35 ordinary colourless, 4 event colourless and 13 curses form the
352-card complete permanent union; the 9 children and 5 statuses form the 14-card
generated-only union (`m26_identity_complete_unions`, `m26_identity_catalog_rows`).
Overlay inertness for every character is `m26_capability_overlay_inert` (U) and
`m26_cross_character_own_inert` (X, 600 own rows); combat and Run admission are
`m26_capability_combat_admission` (U) and `m26_capability_run_admission` (R).

## Final review v1 corrections (reconciliation corrections 7–11)

| Correction | Cards | Keys | Grade | Test (every character) |
| --- | --- | --- | --- | --- |
| 7 Just-applied end-of-turn debuffs | Doubt, Shame; a monster's Weak | K | Constructed | `m26_end_turn_curse_debuffs_last_through_the_next_turn` |
| 8 Retained self-playing cards discard | Burn, Decay, Doubt, Shame, Regret | K | Constructed | `m26_end_turn_self_playing_cards_discard_when_retained`; amended `m26_cards_end_of_turn_order` |
| 9 Fatal on the final Darkling | Feed, Hand of Greed, Lesson Learned, Ritual Dagger | K | Constructed | `m26_fatal_fires_on_the_final_darkling` |
| 10 Master Reality on Searing Blow copies | Dual Wield, Searing Blow | K | Constructed | `m26_master_reality_upgrades_a_copied_searing_blow` |
| F10 Pain (review M3) | Pain, Havoc, Double Tap | K | Constructed | `m26_pain_autoplay_duplicate_and_end_of_turn` |

## Combat generator pools (design §7.1, HEALING excluded)

| Pool | Sizes I / S / D / W | Keys | Grade | Tests |
| --- | --- | --- | --- | --- |
| Any | 70 / 70 / 70 / 69 | U, G | Constructed | `m26_pools_combat_all_characters` (sizes and order); Discovery in `m26_generation_player_pool` (200 seeds per character); `m26_pools_generators_all_characters` |
| Attack | 28 / 26 / 23 / 24 | U, G, R | Constructed | As Any, with Infernal Blade and Metamorphosis; `m26_pools_infernal_blade_player_pool`; `m26_infernal_blade_clash_beside_a_skill` (correction 3); `m26_pools_infernal_blade_m10_replay` (16 frozen-M10 draws byte-identical) |
| Skill | 28 / 33 / 34 / 33 | U, G | Constructed | As Any, with Distraction and Chrysalis |
| Power | 14 / 11 / 13 / 12 | U, G | Constructed | As Any, with White Noise and Creative AI |
| Common (Hello World) | 20 / 19 / 18 / 19 | U, G | Constructed | As Any, with Hello World (F13) |
| Colourless L34 | 34 each | U, G | Constructed | `m26_pools_combat_all_characters` (members and order equal the accepted L34, G3); Jack of All Trades, Transmutation, Magnetism in `m26_generation_player_pool` |
| Foreign Influence | 106 each | U, X | Constructed | `m26_pools_combat_all_characters` (M25 order); `m26_cross_character_sensitive_rows` (Foreign Influence per character) |
| Master Reality on created cards | 11 generators + Dual Wield | G | Constructed | `m26_generation_master_reality_every_character` (all four characters; mutation-checked) |
| Alchemize class potion pools | 32 per class (30 shared + 3 class, Fruit Juice excluded) | W | Constructed | `m26_offcolour_alchemize_all_characters` (G6, F16); amended `items/m15_potion_generation_tests.rs` |

## Run pools (design §7.2, no HEALING exclusion)

| Pool | Sizes | Keys | Grade | Tests |
| --- | --- | --- | --- | --- |
| Ordinary (per colour, rarity, type) | Red 72, Green 71, Blue 71, Purple 71 | U, R | Constructed | `m26_pools_permanent_sizes_and_order`; `m26_mutation_pool_draws` (Purple Rare 17, three distinct draws) |
| AnyColour | 76 / 160 / 84 (320) | U, R | Constructed | `m26_pools_permanent_sizes_and_order`; `m26_any_colour_draw_is_uniform_over_the_flat_family` (F1) |
| Colourless | 0 / 20 / 15 (35) | U, R | Constructed | `m26_pools_permanent_sizes_and_order`; `m26_mutation_pool_draws` (the empty Common family rejects with no RNG committed) |
| Random curse | 10 | U, R | Constructed | `m26_pools_permanent_sizes_and_order`; `m26_mutation_pool_draws` |
| Transform (character, input) | 9 / 34 / 70–72 by input | U, R | Constructed | `m26_pools_transform_supports`; `m26_mutation_transform` |
| Note For Yourself | 349 | U, R | Constructed | `m26_pools_permanent_sizes_and_order`; `m26_mutation_duplicate_and_note` |
| `Catalog::selectable_pool` and reward eligibility | per producer | U | Constructed | `m26_pools_selectable_and_eligibility` (G1, G2) |
| Sampler and RNG scopes | 3 new scopes | R | Constructed | `m26_pools_sampler_generalization_is_rng_identical` (48 captured rows); `m26_pools_rng_scopes` |

## Run mutations (design §8)

| Owner | Keys | Grade | Tests |
| --- | --- | --- | --- |
| Obtain, upgrade (Searing Blow repeatable), remove | R | Constructed | `m26_mutation_obtain_upgrade_remove` (352 kinds × 4 characters) |
| Transform | R | Constructed | `m26_mutation_transform` |
| Duplicate and Note retrieval | R | Constructed | `m26_mutation_duplicate_and_note` |
| Atomic rejection | R | Constructed | `m26_mutation_rejections_are_atomic` |
| Diagnostics constructors (`m26-diagnostics`) | R | Constructed | `m26_diagnostics_constructors` |
| Combat-to-Run settlement of a mixed deck | S | Supplied played | `m26_supplied_mixed_deck_{ironclad,silent,defect,watcher}`: one combat entered and settled once, no fault, Run deck equals the supplied deck plus the journal |
