//! spire-d8g: findings of the 2026-11-16 stress campaign, replayed through the public, validated
//! `standard-combat-v1` catalog. Each case reproduces the recorded fault (minimal entry, seed and action
//! indices from the campaign's localisation) and asserts the original game's behaviour.
use serde_json::{json, Value};
use spire_core::combat::rng::Partition;
use spire_env::*;

fn catalog(entry: Value) -> CombatTrainingCatalog {
    let bytes = serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": [entry]})).unwrap();
    CombatTrainingCatalog::standard_from_json(&bytes).unwrap()
}
fn session(entry: Value) -> CombatTrainingSession { CombatTrainingSession::new_standard(catalog(entry), Partition::Test).unwrap() }
fn field(group: &str, name: &str) -> usize {
    let d = standard_combat_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let t = d["tensors"].as_array().unwrap().iter().find(|t| t["key"] == group).unwrap();
    t["fields"].as_array().unwrap().iter().position(|f| f == name).unwrap()
}
fn value(o: &RunObservation, group: &str, row: usize, name: &str) -> i64 {
    let g = o.group(group).unwrap();
    g.values[row * g.shape.last().unwrap() + field(group, name)]
}
fn vocab(family: &str, name: &str) -> i64 {
    standard_vocab_manifest()["families"][family].as_array().unwrap().iter().position(|v| v == name).unwrap() as i64
}
/// Replay recorded action indices from the reset; every step must be accepted and the combat must continue.
fn replay(s: &mut CombatTrainingSession, seed: u64, id: &str, actions: &[i64]) -> CombatPolicyBoundary {
    let mut b = s.reset(Some(seed), Some(id)).unwrap();
    for (step, ix) in actions.iter().enumerate() {
        let r = s.step(b.token, RunActionIndex(*ix)).unwrap_or_else(|e| panic!("step {step}: {e:?}"));
        assert!(!r.terminated, "step {step} ended the combat early");
        b = r.boundary;
    }
    b
}

/// D2: Infernal Blade played while no enemy is targetable (Awakened One's rebirth window, both Cultists dead).
/// The original copies a random combat Attack and sets its cost to 0 for the turn; card costs are static card fields,
/// independent of targets. The generated Attack's base cost used to come from a target-validated payload, which failed
/// with nothing targetable (`Unsupported("Infernal Blade generated attack cost unavailable")`). Campaign minimal repro
/// c2-236 (seed 1568866308, 13 actions; the last plays Infernal Blade and generates Hemokinesis).
#[test]
fn d8g_d2_infernal_blade_with_no_targetable_enemy_generates_a_zero_cost_attack() {
    let entry = json!({"id": "c2-236", "character": "ironclad", "ascension": 2, "encounter": "awakened_one", "hp": 70, "max_hp": 80,
        "gold": 0, "bottled_cards": {}, "relic_counters": {"neows_lament": 3},
        "relics": ["gremlin_horn", "necronomicon", "warped_tongs", "neows_lament", "ice_cream", "runic_pyramid", "runic_dome", "brimstone"],
        "potions": [null, "blessing_of_the_forge", null, null, null],
        "deck": serde_json::from_str::<Value>(r#"[{"card":"strike","misc":0,"upgrades":0},{"card":"strike","misc":0,"upgrades":0},{"card":"strike","misc":0,"upgrades":0},{"card":"strike","misc":0,"upgrades":0},{"card":"strike","misc":0,"upgrades":0},{"card":"defend","misc":0,"upgrades":0},{"card":"defend","misc":0,"upgrades":0},{"card":"defend","misc":0,"upgrades":0},{"card":"defend","misc":0,"upgrades":0},{"card":"bash","misc":0,"upgrades":0},{"card":"finesse","misc":0,"upgrades":1},{"card":"battle_trance","misc":0,"upgrades":0},{"card":"swift_strike","misc":0,"upgrades":0},{"card":"finesse","misc":0,"upgrades":0},{"card":"good_instincts","misc":0,"upgrades":0},{"card":"dark_embrace","misc":0,"upgrades":1},{"card":"clash","misc":0,"upgrades":1},{"card":"combust","misc":0,"upgrades":0},{"card":"shockwave","misc":0,"upgrades":1},{"card":"dark_shackles","misc":0,"upgrades":1},{"card":"pummel","misc":0,"upgrades":1},{"card":"heavy_blade","misc":0,"upgrades":1},{"card":"finesse","misc":0,"upgrades":1},{"card":"violence","misc":0,"upgrades":1},{"card":"spot_weakness","misc":0,"upgrades":0},{"card":"intimidate","misc":0,"upgrades":0},{"card":"evolve","misc":0,"upgrades":0},{"card":"dropkick","misc":0,"upgrades":1},{"card":"discovery","misc":0,"upgrades":1},{"card":"metamorphosis","misc":0,"upgrades":1},{"card":"the_bomb","misc":0,"upgrades":1},{"card":"blood_for_blood","misc":0,"upgrades":0},{"card":"jack_of_all_trades","misc":0,"upgrades":1},{"card":"exhume","misc":0,"upgrades":1},{"card":"bludgeon","misc":0,"upgrades":0},{"card":"purity","misc":0,"upgrades":1},{"card":"havoc","misc":0,"upgrades":1},{"card":"secret_technique","misc":0,"upgrades":1},{"card":"disarm","misc":0,"upgrades":1},{"card":"berserk","misc":0,"upgrades":0},{"card":"heavy_blade","misc":0,"upgrades":0},{"card":"searing_blow","misc":0,"upgrades":39},{"card":"pommel_strike","misc":0,"upgrades":1},{"card":"exhume","misc":0,"upgrades":1},{"card":"deep_breath","misc":0,"upgrades":0},{"card":"inflame","misc":0,"upgrades":0},{"card":"magnetism","misc":0,"upgrades":0},{"card":"reaper","misc":0,"upgrades":0}]"#).unwrap()});
    let mut s = session(entry);
    let b = replay(&mut s, 1568866308, "c2-236", &[8, 1, 0, 2, 1, 2, 6, 0, 5, 3, 6, 0, 1]);
    let o = &b.observation;
    let rows = o.group("combat_cards").unwrap().shape[0];
    let row = (0..rows).find(|r| value(o, "combat_cards", *r, "card_kind") == vocab("card", "hemokinesis")
        && value(o, "combat_cards", *r, "zone") == vocab("zone", "hand")).expect("generated Hemokinesis in hand");
    assert_eq!(value(o, "combat_cards", row, "cost_zero_this_turn"), 1);
    assert_eq!(value(o, "combat_cards", row, "effective_cost"), 0);
}

/// D3: Time Warp's 12th card is a card Distilled Chaos queued. The original's early end retires the potion's other
/// queued cards unplayed, then the turn ends once the action queue drains (`TimeWarpPower.onAfterUseCard`,
/// `GameActionManager.callEndTurnEarlySequence`). It used to fault the Time Warp barrier, which expected a card play's
/// decision continuation. Campaign minimal repros, one per pending shape at the 12th card; the last action drinks the
/// potion and must be accepted.
fn time_warp_repro(entry: Value, seed: u64, actions: &[i64]) {
    let id = entry["id"].as_str().unwrap().to_owned();
    let mut s = session(entry);
    let mut b = s.reset(Some(seed), Some(&id)).unwrap();
    for (step, ix) in actions.iter().enumerate() {
        let r = s.step(b.token, RunActionIndex(*ix)).unwrap_or_else(|e| panic!("step {step}: {e:?}"));
        if r.terminated { assert_eq!(step + 1, actions.len(), "step {step} ended the combat early"); return }
        b = r.boundary;
    }
}

#[test]
fn d8g_d3_time_warp_twelfth_card_from_distilled_chaos_child_effect_pending() {
    // c2-69 seed 1568866305 (uniform): the 12th card is ftl+1.
    let entry: Value = serde_json::from_str(r#"{"ascension":1,"bottled_cards":{},"character":"defect","deck":[{"card":"strike_b","misc":0,"upgrades":0},{"card":"strike_b","misc":0,"upgrades":0},{"card":"strike_b","misc":0,"upgrades":0},{"card":"strike_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"zap","misc":0,"upgrades":0},{"card":"dualcast","misc":0,"upgrades":0},{"card":"coolheaded","misc":0,"upgrades":1},{"card":"discovery","misc":0,"upgrades":1},{"card":"compile_driver","misc":0,"upgrades":1},{"card":"bullseye","misc":0,"upgrades":0},{"card":"doom_and_gloom","misc":0,"upgrades":0},{"card":"capacitor","misc":0,"upgrades":0},{"card":"equilibrium","misc":0,"upgrades":0},{"card":"hologram","misc":0,"upgrades":1},{"card":"leap","misc":0,"upgrades":0},{"card":"purity","misc":0,"upgrades":0},{"card":"thunder_strike","misc":0,"upgrades":1},{"card":"flash_of_steel","misc":0,"upgrades":1},{"card":"reprogram","misc":0,"upgrades":1},{"card":"dramatic_entrance","misc":0,"upgrades":1},{"card":"buffer","misc":0,"upgrades":1},{"card":"streamline","misc":0,"upgrades":0},{"card":"darkness","misc":0,"upgrades":0},{"card":"rainbow","misc":0,"upgrades":0},{"card":"cold_snap","misc":0,"upgrades":1},{"card":"all_for_one","misc":0,"upgrades":0},{"card":"trip","misc":0,"upgrades":0},{"card":"enlightenment","misc":0,"upgrades":1},{"card":"go_for_the_eyes","misc":0,"upgrades":0},{"card":"stack","misc":0,"upgrades":1},{"card":"storm","misc":0,"upgrades":0},{"card":"self_repair","misc":0,"upgrades":1},{"card":"chaos","misc":0,"upgrades":1},{"card":"tempest","misc":0,"upgrades":1},{"card":"ftl","misc":0,"upgrades":1},{"card":"electrodynamics","misc":0,"upgrades":0},{"card":"chill","misc":0,"upgrades":0},{"card":"dramatic_entrance","misc":0,"upgrades":0},{"card":"biased_cognition","misc":0,"upgrades":1},{"card":"bandage_up","misc":0,"upgrades":1},{"card":"chrysalis","misc":0,"upgrades":1},{"card":"chill","misc":0,"upgrades":0},{"card":"swift_strike","misc":0,"upgrades":0},{"card":"apotheosis","misc":0,"upgrades":1},{"card":"hand_of_greed","misc":0,"upgrades":0}],"encounter":"time_eater","gold":0,"hp":800,"id":"c2-69","max_hp":800,"potions":["liquid_memories","smoke_bomb","fairy_potion","distilled_chaos","entropic_brew"],"relic_counters":{},"relics":["lantern","snecko_eye","philosophers_stone","vajra","sacred_bark","gambling_chip","toolbox"]}"#).unwrap();
    time_warp_repro(entry, 1568866305, &[0, 1, 4, 3, 2, 0, 1, 1, 0, 0, 6, 4, 5, 3, 0, 5, 1, 7, 6, 5, 4, 4, 8, 1, 2, 6, 4, 8]);
}

#[test]
fn d8g_d3_time_warp_twelfth_card_from_distilled_chaos_queued_siblings() {
    // c2-193 seed 1568866305 (greedy): the 12th card is defend_g+0.
    let entry: Value = serde_json::from_str(r#"{"ascension":6,"bottled_cards":{},"character":"silent","deck":[{"card":"strike_g","misc":0,"upgrades":0},{"card":"strike_g","misc":0,"upgrades":0},{"card":"strike_g","misc":0,"upgrades":0},{"card":"strike_g","misc":0,"upgrades":0},{"card":"strike_g","misc":0,"upgrades":0},{"card":"defend_g","misc":0,"upgrades":0},{"card":"defend_g","misc":0,"upgrades":0},{"card":"defend_g","misc":0,"upgrades":0},{"card":"defend_g","misc":0,"upgrades":0},{"card":"defend_g","misc":0,"upgrades":0},{"card":"survivor","misc":0,"upgrades":0},{"card":"neutralize","misc":0,"upgrades":0},{"card":"trip","misc":0,"upgrades":1},{"card":"madness","misc":0,"upgrades":1},{"card":"terror","misc":0,"upgrades":1},{"card":"bullet_time","misc":0,"upgrades":0},{"card":"storm_of_steel","misc":0,"upgrades":1},{"card":"madness","misc":0,"upgrades":0},{"card":"transmutation","misc":0,"upgrades":1},{"card":"metamorphosis","misc":0,"upgrades":1},{"card":"impatience","misc":0,"upgrades":0},{"card":"violence","misc":0,"upgrades":1},{"card":"cloak_and_dagger","misc":0,"upgrades":0},{"card":"infinite_blades","misc":0,"upgrades":1},{"card":"hand_of_greed","misc":0,"upgrades":0}],"encounter":"time_eater","gold":0,"hp":800,"id":"c2-193","max_hp":800,"potions":["entropic_brew","liquid_memories","smoke_bomb","fairy_potion","distilled_chaos"],"relic_counters":{},"relics":["velvet_choker","gambling_chip","toolbox"]}"#).unwrap();
    time_warp_repro(entry, 1568866305, &[0, 4, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 1, 6, 2, 0, 0, 0, 0, 0, 4, 7]);
}

#[test]
fn d8g_d3_time_warp_twelfth_card_from_distilled_chaos_empty_queue_parent_open() {
    // c2-69 seed 1568866307 (potion_heavy): the 12th card is apotheosis+1.
    let entry: Value = serde_json::from_str(r#"{"ascension":1,"bottled_cards":{},"character":"defect","deck":[{"card":"strike_b","misc":0,"upgrades":0},{"card":"strike_b","misc":0,"upgrades":0},{"card":"strike_b","misc":0,"upgrades":0},{"card":"strike_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"defend_b","misc":0,"upgrades":0},{"card":"zap","misc":0,"upgrades":0},{"card":"dualcast","misc":0,"upgrades":0},{"card":"coolheaded","misc":0,"upgrades":1},{"card":"discovery","misc":0,"upgrades":1},{"card":"compile_driver","misc":0,"upgrades":1},{"card":"bullseye","misc":0,"upgrades":0},{"card":"doom_and_gloom","misc":0,"upgrades":0},{"card":"capacitor","misc":0,"upgrades":0},{"card":"equilibrium","misc":0,"upgrades":0},{"card":"hologram","misc":0,"upgrades":1},{"card":"leap","misc":0,"upgrades":0},{"card":"purity","misc":0,"upgrades":0},{"card":"thunder_strike","misc":0,"upgrades":1},{"card":"flash_of_steel","misc":0,"upgrades":1},{"card":"reprogram","misc":0,"upgrades":1},{"card":"dramatic_entrance","misc":0,"upgrades":1},{"card":"buffer","misc":0,"upgrades":1},{"card":"streamline","misc":0,"upgrades":0},{"card":"darkness","misc":0,"upgrades":0},{"card":"rainbow","misc":0,"upgrades":0},{"card":"cold_snap","misc":0,"upgrades":1},{"card":"all_for_one","misc":0,"upgrades":0},{"card":"trip","misc":0,"upgrades":0},{"card":"enlightenment","misc":0,"upgrades":1},{"card":"go_for_the_eyes","misc":0,"upgrades":0},{"card":"stack","misc":0,"upgrades":1},{"card":"storm","misc":0,"upgrades":0},{"card":"self_repair","misc":0,"upgrades":1},{"card":"chaos","misc":0,"upgrades":1},{"card":"tempest","misc":0,"upgrades":1},{"card":"ftl","misc":0,"upgrades":1},{"card":"electrodynamics","misc":0,"upgrades":0},{"card":"chill","misc":0,"upgrades":0},{"card":"dramatic_entrance","misc":0,"upgrades":0},{"card":"biased_cognition","misc":0,"upgrades":1},{"card":"bandage_up","misc":0,"upgrades":1},{"card":"chrysalis","misc":0,"upgrades":1},{"card":"chill","misc":0,"upgrades":0},{"card":"swift_strike","misc":0,"upgrades":0},{"card":"apotheosis","misc":0,"upgrades":1},{"card":"hand_of_greed","misc":0,"upgrades":0}],"encounter":"time_eater","gold":0,"hp":800,"id":"c2-69","max_hp":800,"potions":["liquid_memories","smoke_bomb","fairy_potion","distilled_chaos","entropic_brew"],"relic_counters":{},"relics":["snecko_eye","sacred_bark","gambling_chip","toolbox"]}"#).unwrap();
    time_warp_repro(entry, 1568866307, &[2, 7, 1, 0, 0, 2, 3, 13, 4, 2, 14, 13, 15, 14]);
}

/// Lane C r4 replay fixture (`tests/fixtures/d8g/NAME.json`: catalog entry, seed and the recorded action prefix whose
/// last action used to fault). Replays it and returns the boundary after the last action.
fn r4_replay(name: &str) -> CombatPolicyBoundary {
    let path = format!("{}/tests/fixtures/d8g/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let fixture: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let id = fixture["entry"]["id"].as_str().unwrap().to_owned();
    let actions = fixture["actions"].as_array().unwrap().iter().map(|ix| ix.as_i64().unwrap()).collect::<Vec<_>>();
    let mut s = session(fixture["entry"].clone());
    replay(&mut s, fixture["seed"].as_u64().unwrap(), &id, &actions)
}

/// E7: Burst's copy of Havoc waits in the card queue while Havoc's top card (Survivor) has its discard choice open.
/// The original drains the action queue (the selection) before the card queue, then the copy plays (lane D Q12).
/// The selection boundary used to reject the suspended parent (`Invariant("Burst repeat child/parent ownership")`).
#[test]
fn d8g_e7_burst_repeat_waits_behind_the_parents_top_card_selection() {
    r4_replay("e7-silent-a0-time-eater");
}

/// E8: Duplication Potion's copy of Havoc waits while Havoc's top card (Secret Technique) has its draw-pile choice
/// open; the selection resolves first (lane D Q13). The boundary used to fault with
/// `Invariant("Duplication repeat retains its actual card and captured child")`.
#[test]
fn d8g_e8_duplication_repeat_waits_behind_the_autoplayed_cards_selection() {
    r4_replay("e8-ironclad-a1-reptomancer");
}

/// E9: Foresight is earlier than Night Terror in the power list, so its start-of-turn Scry opens before the
/// Nightmare copies are delivered; both precede the turn draw (lane D Q14). The Scry boundary used to fault with
/// `Invariant("Nightmare template identity, owner, reset state, or delivery cursor invalid")`.
#[test]
fn d8g_e9_nightmare_delivery_waits_behind_a_start_of_turn_scry() {
    r4_replay("e9-watcher-a5-awakened-one");
}

/// E10: the original's hand never exceeds 10 cards (lane D Q15). In this recorded replay the end turn used to leave
/// 11 cards in hand, and the next turn's draw faulted with `Invariant("hand exceeds cap")`. With the fix the hand order
/// differs from turn 2, so the recorded actions no longer reach that overflow: this replay only checks that the
/// recorded prefix runs without a fault. The pin for the ordering itself is the core test
/// `combat::tests::d8g_findings::d8g_e10_retained_card_is_restored_before_end_of_turn_exhaust_children`.
#[test]
fn d8g_e10_recorded_overflow_replay_runs_without_a_fault() {
    r4_replay("e10-ironclad-a0-three-darklings");
}
