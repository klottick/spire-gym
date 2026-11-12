//! spire-3xq: findings of the M38e capacity lanes. The combat faults found by the W5 stress are replayed through the
//! public, validated `standard-combat-v1` catalog; each case reproduces the recorded fault (the W5 entry and seed, or a
//! minimal entry) and asserts the original game's behaviour. F1 pins the dense `offers` capacity's fail-closed edge.
use serde_json::{json, Value};
use spire_core::{combat::{rng::Partition, FaultKind}, run::{RunError, RunFaultKind}};
use spire_env::*;

fn catalog(entry: Value) -> CombatTrainingCatalog {
    let bytes = serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": [entry]})).unwrap();
    CombatTrainingCatalog::standard_from_json(&bytes).unwrap()
}
fn session(entry: Value) -> CombatTrainingSession { CombatTrainingSession::new_standard(catalog(entry), Partition::Test).unwrap() }
fn cards(names: &[&str]) -> Vec<Value> { names.iter().map(|c| json!({"card": c, "upgrades": 0, "misc": 0})).collect() }
fn entry(id: &str, character: &str, encounter: &str, deck: Vec<Value>) -> Value {
    json!({"id": id, "character": character, "ascension": 0, "encounter": encounter, "hp": 999, "max_hp": 999, "gold": 0,
        "deck": deck, "relics": [], "potions": [null, null, null]})
}
fn legal(o: &RunObservation) -> Vec<usize> { o.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect() }
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
fn enemy_hp(o: &RunObservation, enemy: &str) -> i64 {
    let rows = o.group("enemies").unwrap().shape[0];
    let row = (0..rows).find(|r| value(o, "enemies", *r, "enemy_kind") == vocab("enemy", enemy)).unwrap();
    value(o, "enemies", row, "hp")
}
fn verb_index(o: &RunObservation, verb: &str) -> Option<usize> {
    legal(o).into_iter().find(|i| value(o, "actions", *i, "verb") == vocab("verb", verb))
}
/// The W5 harness's random legal choice (`m38_w5_measure.rs`), so a recorded (entry, seed) replays its trajectory.
fn w5_pick(seed: u64, step: usize, legal: &[usize]) -> usize {
    let mut z = seed.wrapping_add((step as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    legal[(z % legal.len() as u64) as usize]
}
/// Play `steps` W5 choices from the reset; every step must be accepted.
fn w5_replay(s: &mut CombatTrainingSession, seed: u64, id: &str, steps: usize) -> CombatPolicyBoundary {
    let mut b = s.reset(Some(seed), Some(id)).unwrap();
    for step in 0..steps {
        let ix = w5_pick(seed, step, &legal(&b.observation));
        let r = s.step(b.token, RunActionIndex(ix as i64)).unwrap_or_else(|e| panic!("step {step}: {e:?}"));
        assert!(!r.terminated, "step {step} ended the combat early");
        b = r.boundary;
    }
    b
}
fn combat_fault(e: &RunSessionError) -> Option<(FaultKind, String)> {
    match e {
        RunSessionError::Core(RunError::Faulted(f)) => match &f.kind {
            RunFaultKind::Combat(c) => Some((c.record.kind, c.record.message.clone())),
            _ => None,
        },
        _ => None,
    }
}

/// F2-A: Berserk stacks additively without a cap and grants its whole amount at each start of turn (original
/// `BerserkPower`). Five or six stacks used to fault the energy validator, which admitted only 1..=4.
#[test]
fn x3q_berserk_five_and_six_stacks_grant_their_amount_at_turn_start() {
    // Six Berserks: the opening hand plays five; the sixth is drawn and played on turn two.
    let mut s = session(entry("berserk", "ironclad", "jaw_worm", cards(&["berserk"; 6])));
    let mut b = s.reset(Some(1), None).unwrap();
    for stacks in [5i64, 6] {
        while let Some(ix) = verb_index(&b.observation, "play_card") {
            b = s.step(b.token, RunActionIndex(ix as i64)).unwrap().boundary;
        }
        let end = verb_index(&b.observation, "end_turn").unwrap();
        let r = s.step(b.token, RunActionIndex(end as i64)).unwrap_or_else(|e| panic!("{stacks} stacks: {e:?}"));
        assert!(!r.terminated);
        assert_eq!(value(&r.boundary.observation, "player", 0, "energy"), 3 + stacks, "{stacks} Berserk stacks");
        b = r.boundary;
    }
}

/// F2-B (W5 m38e_0059, seed 954205641): four drawn Dazed queue Fire Breathing's Thorns(12); Mayhem then autoplays a
/// Fire Breathing (12 -> 18) before the last queued hits resolve. The original snapshots the amount when it queues the
/// damage, so the queued hits stay 12 and resolve; they used to fault the Thorns producer check (amount must equal 18).
#[test]
fn x3q_w5_fire_breathing_queued_thorns_keep_their_queued_amount() {
    let pool = ["inflame", "evolve", "fire_breathing", "juggernaut", "rupture", "corruption", "dark_embrace", "feel_no_pain",
        "barricade", "berserk", "brutality", "demon_form", "metallicize", "combust", "mayhem", "panache", "sadistic_nature",
        "magnetism", "corruption", "feel_no_pain", "dark_embrace", "fire_breathing", "second_wind", "discovery",
        "jack_of_all_trades", "metamorphosis", "chrysalis", "transmutation", "magnetism", "mayhem"];
    let deck = (0..100).map(|i| pool[i % pool.len()]).collect::<Vec<_>>();
    let mut s = session(entry("m38e_0059", "ironclad", "donu_and_deca", cards(&deck)));
    let seed = 954205641;
    let b = w5_replay(&mut s, seed, "m38e_0059", 45);
    let ix = w5_pick(seed, 45, &legal(&b.observation));
    assert_eq!(Some(ix), verb_index(&b.observation, "end_turn"), "the recorded step 45 is End Turn");
    s.step(b.token, RunActionIndex(ix as i64)).unwrap();
}

/// F2-C: a combat issues at most 999 card identities (deck included). Reaching that bound is the engine's typed
/// `Unsupported` boundary on every generation path; the Corrupt Heart's status cards used to report it as `Invariant`.
/// (The original has no such cap; this is a documented engine limit.)
#[test]
fn x3q_issued_card_bound_is_unsupported_on_the_create_card_path() {
    let mut s = session(entry("heart995", "ironclad", "corrupt_heart", cards(&vec!["strike"; 995])));
    let b = s.reset(Some(1), None).unwrap();
    let end = verb_index(&b.observation, "end_turn").unwrap();
    let e = s.step(b.token, RunActionIndex(end as i64)).unwrap_err();
    assert_eq!(combat_fault(&e), Some((FaultKind::Unsupported, "generated card count/runtime boundary".into())), "{e:?}");
    // Generation through `Generate` (Chrysalis) is the same typed boundary.
    let mut s = session(entry("chrysalis997", "ironclad", "jaw_worm", cards(&vec!["chrysalis"; 997])));
    let b = s.reset(Some(1), None).unwrap();
    let play = verb_index(&b.observation, "play_card").unwrap();
    let e = s.step(b.token, RunActionIndex(play as i64)).unwrap_err();
    assert_eq!(combat_fault(&e), Some((FaultKind::Unsupported, "generated card count/runtime boundary".into())), "{e:?}");
}

/// F2-D (W5 m38e_0044, minimised): Neow's Lament leaves every enemy at 1 HP; Mercury Hourglass kills the Reptomancer and a
/// Dagger at turn start and the last Dagger suicides. The combat is won before Gambling Chip's post-draw selection, so it
/// ends at Victory with no selection: the engine admits no selection without a living enemy (the original's action has no
/// guard, but its battle-end cleanup drops the action, so a discard chosen there has no effect). The engine used to
/// publish the selection with no living enemy (an Invariant surfaced as `InvalidAction`).
#[test]
fn x3q_w5_reptomancer_turn_start_victory_opens_no_gambling_chip_selection() {
    let mut e = entry("m38e_0044", "ironclad", "reptomancer", cards(&["strike", "defend"]));
    e["relics"] = json!(["neows_lament", "bottled_flame", "bottled_lightning", "bottled_tornado", "mercury_hourglass", "gambling_chip", "toolbox"]);
    e["relic_counters"] = json!({"neows_lament": 3});
    e["bottled_cards"] = json!({"bottled_flame": {"card": "strike", "upgrades": 0, "misc": 0}, "bottled_lightning": {"card": "defend", "upgrades": 0, "misc": 0}, "bottled_tornado": null});
    e["potions"] = json!([null, null, null, null, null]);
    let mut s = session(e);
    let b = s.reset(Some(954205260), None).unwrap();
    assert_eq!(value(&b.observation, "actions", 1, "verb"), vocab("verb", "choose_card_offer"), "Toolbox's opening offer");
    let r = s.step(b.token, RunActionIndex(1)).unwrap();
    assert!(r.terminated);
    assert_eq!(r.boundary.info["terminal_reason"], "victory");
}

/// F2-E (W5 m38e_0520, minimised; seed 954217178, step 37): Eruption (cost 2) qualifies for Necronomicon's repeat; its
/// Wrath entry triggers Rushdown's draw, the draw pile reshuffles and Melange opens a Scry. In the original the Scry
/// (action queue) resolves first and the repeat (card queue) plays afterwards. The engine used to reject that legal
/// selection boundary (an autoplay ownership Invariant surfaced as `InvalidAction`).
#[test]
fn x3q_w5_necronomicon_repeat_waits_for_melange_scry() {
    let mut e = entry("m38e_0520", "watcher", "awakened_one",
        cards(&["strike_p", "strike_p", "strike_p", "strike_p", "defend_p", "defend_p", "defend_p", "defend_p", "eruption", "vigilance"]));
    e["relics"] = json!(["holy_water", "neows_lament", "sozu", "golden_eye", "melange", "bottled_flame", "bottled_lightning",
        "bottled_tornado", "ink_bottle", "mercury_hourglass", "mummified_hand", "dead_branch", "gambling_chip", "runic_dome",
        "runic_pyramid", "snecko_eye", "velvet_choker", "clockwork_souvenir", "strange_spoon", "toolbox", "enchiridion",
        "necronomicon", "nilrys_codex", "warped_tongs"]);
    e["relic_counters"] = json!({"neows_lament": 3});
    e["bottled_cards"] = json!({"bottled_flame": {"card": "strike_p", "upgrades": 0, "misc": 0}, "bottled_lightning": {"card": "defend_p", "upgrades": 0, "misc": 0}, "bottled_tornado": null});
    e["potions"] = json!([null, null, null, null, null]);
    let mut s = session(e);
    let seed = 954217178;
    let b = w5_replay(&mut s, seed, "m38e_0520", 37);
    let before = enemy_hp(&b.observation, "awakened_one");
    let ix = w5_pick(seed, 37, &legal(&b.observation));
    let r = s.step(b.token, RunActionIndex(ix as i64)).unwrap();
    let first = before - enemy_hp(&r.boundary.observation, "awakened_one");
    assert!(first > 0, "Eruption hit");
    let confirm = verb_index(&r.boundary.observation, "confirm_selection").expect("Melange's Scry is open before the repeat");
    let r = s.step(r.boundary.token, RunActionIndex(confirm as i64)).unwrap();
    let second = before - first - enemy_hp(&r.boundary.observation, "awakened_one");
    // Eruption hits for 9 and then enters Wrath, so the repeat that plays after the Scry hits for 18.
    assert_eq!((first, second), (9, 18), "the Necronomicon repeat plays after the Scry, in Wrath");
}

/// F2-D, normal play: a Silent's Dagger Throw (damage, draw 1, then discard 1) kills the Reptomancer at 1 HP and its
/// Daggers suicide. The combat is won; the discard choice queued behind the kill must not open (the engine admits no
/// selection without a living enemy; the original's discard checks that the battle is ending). It used to fault.
#[test]
fn x3q_dagger_throw_killing_the_reptomancer_wins_without_a_discard_choice() {
    let deck = ["dagger_throw", "strike_g", "strike_g", "strike_g", "strike_g", "defend_g", "defend_g", "defend_g", "defend_g"];
    let mut e = entry("dagger_throw", "silent", "reptomancer", cards(&deck));
    let card = |c: &str| json!({"card": c, "upgrades": 0, "misc": 0, "generated": false});
    e["position"] = json!({"enemies": [{"slot": 1, "hp": 1}], "piles": {"hand": [card("dagger_throw"), card("strike_g"), card("strike_g"), card("defend_g"), card("defend_g")],
        "draw_top": [], "draw": [card("strike_g"), card("strike_g"), card("defend_g"), card("defend_g")], "discard": [], "exhaust": []}});
    let mut s = session(e);
    let b = s.reset(Some(1), None).unwrap();
    let o = &b.observation;
    let rows = o.group("enemies").unwrap().shape[0];
    let repto = (0..rows).find(|r| value(o, "enemies", *r, "enemy_kind") == vocab("enemy", "reptomancer")).unwrap();
    assert_eq!(value(o, "enemies", repto, "hp"), 1);
    let play = legal(o).into_iter().find(|i| value(o, "actions", *i, "verb") == vocab("verb", "play_card")
        && value(o, "combat_cards", value(o, "actions", *i, "source_row") as usize, "card_kind") == vocab("card", "dagger_throw")
        && value(o, "actions", *i, "target_row") == repto as i64).unwrap();
    let r = s.step(b.token, RunActionIndex(play as i64)).unwrap();
    assert!(r.terminated);
    assert_eq!(r.boundary.info["terminal_reason"], "victory");
}

/// F1: a room screen with more rows than the dense `offers` capacity fails closed with a typed `Capacity` fault; it never
/// truncates the row list. Since spire-0kr The Courier restocks in place, so a shop holds at most 15 rows; this synthetic
/// 513-row screen keeps the encoder's fail-closed path covered.
#[test]
fn x3q_more_shop_rows_than_the_dense_offers_capacity_is_a_typed_capacity_fault() {
    use spire_core::run::{ChoiceId, CreationProfile, RoomChoiceKind, RoomChoiceStatus, RoomChoiceView, RoomScenario, RoomView, RoomViewKind, Run, RunConfig, RunRng};
    let run = Run::new_room_scenario(RunConfig { character: spire_core::content::Character::Ironclad, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([3; 32], Partition::Test, 0), RoomScenario::Standard).unwrap();
    let mut d = run.decision().unwrap();
    let caps = RunCapacities::standard_default();
    assert_eq!(caps.offers, 512);
    d.room = Some(RoomView { kind: RoomViewKind::Shop, choices: (0..513).map(|i| RoomChoiceView { id: ChoiceId(i),
        kind: RoomChoiceKind::Leave, price: None, status: RoomChoiceStatus::Claimed }).collect() });
    assert_eq!(encode_run_standard_decision(&d, None, &caps, 0, None).unwrap_err(),
        RunSessionError::Capacity { group: "offers", required: 513, capacity: 512 });
}
