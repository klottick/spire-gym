//! Engine registry of every public power and intent identity.
//!
//! The public combat view only publishes names listed here (checked in debug
//! builds at the view boundary). Observation encoders derive their power and
//! intent vocabularies from these tables, so an encoder cannot lack an identity
//! the engine can publish. Append new identities when a producer is added.

/// Every power name the public combat view can publish, in vocabulary order.
pub const PUBLIC_POWERS: &[&str] = &[
    "strength", "dexterity", "weak", "vulnerable", "frail", "artifact", "ritual", "curl_up",
    "split", "enrage", "metallicize", "mode_shift", "sharp_hide", "barricade", "flight", "hex",
    "confused", "minion", "angry", "entangled", "painful_stabs", "stasis", "regeneration", "thorns",
    "explosive", "life_link", "regrow", "fading", "shifting", "shackled", "constricted", "reactive",
    "malleable", "slow", "intangible", "regenerate", "curiosity", "unawakened", "time_warp",
    "draw_reduction", "poison", "strength_up", "berserk", "brutality", "combust", "corruption",
    "dark_embrace", "demon_form", "double_tap", "evolve", "feel_no_pain", "fire_breathing",
    "flame_barrier", "juggernaut", "no_draw", "rage", "rupture", "beat_of_death", "invincible",
    "surrounded", "back_attack", "lose_strength", "spore_cloud", "thievery", "plated_armor",
    "noxious_fumes", "accuracy", "retain_cards", "a_thousand_cuts", "after_image", "blur", "burst",
    "choked", "corpse_explosion", "double_damage", "draw_card_next_turn", "energized", "envenom",
    "infinite_blades", "intangible_player", "magnetism", "mayhem", "next_turn_block", "no_block",
    "panache", "phantasmal", "sadistic_nature", "tools_of_the_trade", "wraith_form",
    "lose_dexterity", "duplication", "panache_countdown", "the_bomb", "the_bomb_plus", "focus",
    "bias", "loop", "electrodynamics", "lock_on", "static_discharge", "storm", "heatsinks",
    "amplify", "echo_form", "buffer", "repair", "draw", "energized_blue", "hello_world",
    "creative_ai", "rebound", "equilibrium", "mantra", "devotion", "foresight", "nirvana",
    "establishment", "master_reality", "battle_hymn", "mental_fortress", "rushdown", "like_water",
    "mark", "block_return", "vigor", "wave_of_the_hand", "free_attack", "simmering_fury", "study",
    "energy_down", "deva_form", "collect", "omega", "blasphemer", "pen_nib", "melange",
    "cannot_change_stance",
];

/// Frozen shared-policy intent vocabulary, in descriptor order.
/// `hidden` is the masked intent shown while intents are concealed. Event-local
/// intents visible through CombatPublic are listed separately below so this table
/// does not change the frozen observation schemas.
pub const PUBLIC_INTENTS: &[&str] = &[
    "incantation", "attack", "chomp", "thrash", "bellow", "bite", "grow", "spit_web", "skull_bash",
    "bull_rush", "acid_tackle", "acid_spit", "acid_lick", "acid_split", "spike_tackle",
    "spike_flame_tackle", "spike_lick", "spike_split", "sticky", "prep_slam", "slam", "split",
    "sleep", "siphon", "stun", "lag_attack", "sentry_bolt", "sentry_beam", "charge_up",
    "fierce_bash", "vent_steam", "whirlwind", "defensive_mode", "roll_attack", "twin_slam",
    "activate", "divider", "sear", "tackle", "inflame", "inferno", "sphere_activate",
    "sphere_debuff", "sphere_slam", "sphere_harden", "byrd_caw", "byrd_peck", "byrd_swoop",
    "byrd_headbutt", "byrd_stunned", "byrd_go_airborne", "chosen_poke", "chosen_hex",
    "chosen_debilitate", "chosen_drain", "chosen_zap", "snecko_glare", "snecko_bite",
    "snecko_tail_whip", "centurion_slash", "centurion_protect", "centurion_fury", "mystic_attack",
    "mystic_buff", "mystic_heal", "leader_rally", "leader_stab", "leader_encourage",
    "gremlin_scratch", "gremlin_puncture", "gremlin_smash", "gremlin_bash", "gremlin_protect",
    "gremlin_charge", "gremlin_blast", "gremlin_escape", "slaver_stab", "slaver_rake",
    "slaver_scrape", "slaver_entangle", "taskmaster_whip", "book_big", "book_multi", "champ_anger",
    "champ_execute", "champ_taunt", "champ_stance", "champ_gloat", "champ_slap", "champ_slash",
    "collector_spawn", "collector_ultimate", "collector_revive", "collector_rake", "collector_buff",
    "torch_attack", "automaton_spawn", "automaton_beam", "automaton_stun", "automaton_flail",
    "automaton_boost", "orb_stasis", "orb_support", "orb_beam", "orb_claw", "orb_laser",
    "repulsor_attack", "repulsor_daze", "exploder_attack", "exploder_explode", "spiker_attack",
    "spiker_buff", "darkling_reincarnate", "darkling_harden", "darkling_nip", "darkling_chomp",
    "darkling_regrowing", "growth_quick", "growth_constrict", "growth_smash", "transient_attack",
    "maw_roar", "maw_nom", "maw_drool", "maw_slam", "writhing_multi", "writhing_block",
    "writhing_debuff", "writhing_parasite", "writhing_big", "head_count", "head_time", "head_glare",
    "nemesis_tri", "nemesis_scythe", "nemesis_burn", "repto_spawn", "repto_snake", "repto_bite",
    "dagger_wound", "dagger_explode", "awa_rebirth", "awa_slash", "awa_echo", "awa_soul",
    "awa_sludge", "awa_tackle", "time_haste", "time_reverb", "time_slam", "time_ripple",
    "deca_beam", "deca_square", "donu_beam", "donu_circle", "charging_up", "beam", "bolt",
    "stunned", "shield_bash", "shield_fortify", "shield_smash", "spear_burn_strike", "spear_skewer",
    "spear_strength", "heart_debilitate", "heart_blood_shots", "heart_echo", "heart_buff",
    "looter_mug", "looter_lunge", "looter_smoke_bomb", "looter_escape", "mugger_mug",
    "mugger_lunge", "mugger_smoke_bomb", "mugger_escape", "shelled_parasite_double_strike",
    "shelled_parasite_life_suck", "shelled_parasite_fell", "shelled_parasite_stunned",
    "snake_plant_chomp_chomp", "snake_plant_enfeebling_spores", "hidden",
];

/// Finite M28e event intents are visible in CombatPublic but stay outside the
/// frozen shared policy vocabulary until a profile admits them.
pub const EVENT_INTENTS: &[&str] = &[
    "pointy_special", "romeo_mock", "romeo_agonizing_slash", "romeo_cross_slash",
    "bear_hug", "bear_lunge", "bear_maul",
];

pub fn is_public_power(name: &str) -> bool { PUBLIC_POWERS.contains(&name) }
pub fn is_public_intent(name: &str) -> bool { PUBLIC_INTENTS.contains(&name) || EVENT_INTENTS.contains(&name) }
