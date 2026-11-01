mod rng_contract;
mod support;
mod entry_replay;
mod rules_ai;
mod resolution;
mod actions_views;
mod public_projection;
mod piles;
mod turns;
mod whole_combat;
mod differential;
mod differential_decode;
mod full_runtime;
mod full_enemies;
mod full_cards;
mod selection;
mod shared_mechanics;
mod m5_elites;
mod m5_status;
mod m5_bosses;
mod m6_ordinary;
mod m6_pressure;
mod m6_partners;
mod m6_summons;
mod m6_elites;
mod m6_bosses;
mod m6_stasis;
mod observe;
mod m8_shapes;
mod m8_darklings;
mod m8_ordinary;
mod m8_writhing;
mod m8_elites;
mod m8_reptomancer;
mod m8_awakened;
mod m8_time_eater;
mod m8_partners;
mod m9_relics;
mod m10_upgrade;
mod m10_direct;
mod m10_support;
mod m10_selection;
mod m10_copy;
mod m10_autoplay;
mod m10_growth;
mod m10_exhaust_hooks;
mod exhaust;
mod m10_exhaust_actions;
mod m10_fatal;
mod m10_power_turns;
mod m10_reactive;
mod m10_ironclad_relics;
mod m10_ironclad_potions;
mod m10_heart;
mod m10_generation;
mod m10_shield_spear;
mod m11_silent_poison;
mod m11_silent_discard;
mod m11_silent_shiv;
mod m11_silent_retain;
mod m11_silent_copy;

mod m12_silent_values;

mod m13_silent_interactions;
mod m14_silent_interactions;
mod m15_silent_items;

mod encounter_thieves;
mod encounter_ordinary;
mod m15_silent_direct;
mod m15_silent_cost_copy;
mod m15_silent_x_repeat;
mod m15_silent_history;
mod m15_silent_powers;
mod m15_silent_poison_death;
mod m15_colorless_direct;
mod m15_colorless_powers;

mod m15_card_offers;
mod m15_colorless_choices;

mod m15_generation;
mod m15_potion_actions;

mod m15_smoke;
mod m15_potion_generation;

mod m15_act4_interactions;
mod m16_orbs;
mod m16_mutations;
mod m16_power_callbacks;
mod m16_orb_interactions;
pub(crate) mod m16_support;
mod m16_content;
mod m16_review_corrections;

pub(crate) mod m18_custody_support;
mod m18_defect_interactions;

pub(crate) mod m19_defect_interactions;

mod m20_defect_state;
mod m20_defect_generation;
mod m20_defect_direct;
mod m20_defect_powers;

mod m20_defect_interactions;

mod m21_watcher_stances;

mod m21_watcher_retain;

mod m21_watcher_scry;

mod m21_watcher_creation;
mod m21_watcher_review;

mod m22_watcher_progression;
mod m23_watcher_act2;
mod m23_confused_cost;
mod m23_cost_law;
mod m24_watcher_act3;
mod m24_time_warp_barrier;
mod m25_watcher_generation;
mod m25_watcher_direct;
mod m25_watcher_state;
mod m25_watcher_powers;
mod m25_watcher_foreign_influence;
mod m25_watcher_interactions;
mod m25_watcher_interactions_w;
mod m25_watcher_nested_autoplay;
mod m25_watcher_selection_during_play;
mod m25_watcher_generated_construction;
mod m25_watcher_unplayable_in_hand;
mod m25_burn_retain_all_characters;
mod m26_capability;
mod m26_cards;
mod m26_offcolour_laws;
mod m26_pools;
mod m26_autoplay;
mod m26_support;
mod m26_cross_character;
mod m26_generation;
mod m26_copies;
mod m26_corrections;
