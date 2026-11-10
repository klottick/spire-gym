//! M34s-c c1/c2: synthetic positions on `standard-combat-v1` (design §4.8; c0 `task0-v1.md`): strict validation before any
//! effect, start labels, the gym-owned seeded draw order, and the lesson and smoke fixtures continuing to a terminal.
use serde_json::{json, Value};
use spire_core::{combat::rng::Partition, content::{Catalog, Character}};
use spire_env::*;

fn slots(ascension: u8) -> usize { if ascension >= 11 { 2 } else { 3 } }
fn starter(character: Character) -> Vec<Value> {
    Catalog::standard().starter_deck(character).unwrap().iter().map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect()
}
fn entry(id: &str, character: Character, ascension: u8, encounter: &str, deck: Vec<Value>, relics: Vec<&str>) -> Value {
    json!({"id": id, "character": character.as_str(), "ascension": ascension, "encounter": encounter, "hp": 70, "max_hp": 80, "gold": 99,
        "deck": deck, "relics": relics, "potions": vec![Value::Null; slots(ascension)]})
}
fn catalog(entries: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": entries})).unwrap()
}
fn parse(entries: Vec<Value>) -> Result<CombatTrainingCatalog, RunSessionError> { CombatTrainingCatalog::standard_from_json(&catalog(entries)) }
fn session(entries: Vec<Value>) -> CombatTrainingSession { CombatTrainingSession::new_standard(parse(entries).unwrap(), Partition::Test).unwrap() }
fn c(card: &str) -> Value { json!({"card": card, "upgrades": 0}) }
fn cu(card: &str) -> Value { json!({"card": card, "upgrades": 1}) }
fn g(card: &str) -> Value { json!({"card": card, "generated": true}) }

/// The c0 lesson and smoke fixtures (design §4.8; plan c0): two lessons native-consistent at turn 1 and one smoke per
/// remaining character, each at A0 and A20.
pub fn lessons(ascension: u8) -> Vec<Value> {
    let ironclad = vec![c("strike"), c("strike"), c("strike"), c("strike"), c("defend"), c("defend"), c("defend"), c("bash"), cu("true_grit"), c("burning_pact"), c("second_wind"), c("shrug_it_off")];
    let mut l1 = entry(&format!("l1-ironclad-nob-a{ascension}"), Character::Ironclad, ascension, "gremlin_nob", ironclad, vec!["burning_blood"]);
    l1["position"] = json!({"player": {"powers": {"feel_no_pain": 3, "dark_embrace": 1}},
        "piles": {"hand": [cu("true_grit"), c("burning_pact"), c("second_wind"), c("shrug_it_off"), c("bash")],
                  "draw": [c("strike"), c("strike"), c("strike"), c("defend"), c("defend"), c("defend")], "discard": [c("strike")]}});
    let silent = vec![c("strike_g"), c("strike_g"), c("strike_g"), c("defend_g"), c("defend_g"), c("neutralize"), c("survivor"),
        c("deadly_poison"), c("bouncing_flask"), cu("catalyst"), c("footwork")];
    let mut l2 = entry(&format!("l2-silent-poison-a{ascension}"), Character::Silent, ascension, "jaw_worm", silent, vec!["ring_of_the_snake"]);
    l2["position"] = json!({"enemies": [{"slot": 0, "powers": {"poison": 4}}],
        "piles": {"hand": [c("deadly_poison"), c("bouncing_flask"), cu("catalyst"), c("neutralize"), c("footwork")],
                  "draw": [c("strike_g"), c("strike_g"), c("strike_g"), c("defend_g"), c("defend_g"), c("survivor")]}});
    let mut defect = entry(&format!("s-defect-a{ascension}"), Character::Defect, ascension, "three_sentries", starter(Character::Defect), vec!["cracked_core"]);
    defect["position"] = json!({"player": {"powers": {"focus": 1}, "orb_slots": 3, "orbs": ["frost", "lightning"]}});
    let mut watcher = entry(&format!("s-watcher-a{ascension}"), Character::Watcher, ascension, "cultist", starter(Character::Watcher), vec!["pure_water"]);
    watcher["position"] = json!({"player": {"stance": "calm", "mantra": 5, "energy": 4},
        "piles": {"hand": [c("eruption"), c("vigilance"), c("strike_p"), c("defend_p"), g("miracle")],
                  "draw_top": [c("strike_p")], "draw": [c("strike_p"), c("strike_p"), c("defend_p"), c("defend_p")],
                  "discard": [c("defend_p")]}});
    vec![l1, l2, defect, watcher]
}

fn first_legal(o: &RunObservation) -> Vec<usize> { o.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect() }
fn end_turn(o: &RunObservation) -> usize {
    let verb = standard_vocab_manifest()["families"]["verb"].as_array().unwrap().iter().position(|v| v == "end_turn").unwrap() as i64;
    let g = o.group("actions").unwrap();
    first_legal(o).into_iter().find(|i| g.values[i * g.shape.last().unwrap()] == verb).unwrap()
}

/// c1: every invalid input is `Input` naming the field, at catalog parse (before any session exists).
#[test]
fn position_rejects_invalid_inputs() {
    let base = lessons(0).remove(0);
    assert!(parse(vec![base.clone()]).is_ok());
    let edits: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        ("player.powers", Box::new(|e| e["position"]["player"]["powers"]["not_a_power"] = json!(1))),
        ("player.powers", Box::new(|e| e["position"]["player"]["powers"]["combust"] = json!(1))),
        ("player.powers", Box::new(|e| e["position"]["player"]["powers"]["evolve"] = json!(1))),
        ("player.powers", Box::new(|e| e["position"]["player"]["powers"]["strength"] = json!(5000))),
        ("piles", Box::new(|e| e["position"]["piles"]["hand"][0] = c("not_a_card"))),
        ("piles", Box::new(|e| e["position"]["piles"]["hand"][0] = cu("bash"))),
        ("piles", Box::new(|e| e["position"]["piles"]["hand"][0]["misc"] = json!(3))),
        ("piles", Box::new(|e| { e["position"]["piles"]["discard"].as_array_mut().unwrap().clear(); })),
        ("piles", Box::new(|e| { e["position"]["piles"]["discard"].as_array_mut().unwrap().push(c("strike")); })),
        ("piles", Box::new(|e| { e["position"]["piles"]["discard"].as_array_mut().unwrap().push(c("anger")); })),
        ("piles", Box::new(|e| { e["position"]["piles"]["discard"].as_array_mut().unwrap().push(g("shiv")); })),
        ("piles.hand", Box::new(|e| { for _ in 0..6 { e["position"]["piles"]["hand"].as_array_mut().unwrap().push(g("wound")); } })),
        ("piles.hand", Box::new(|e| { e["position"]["piles"]["hand"].as_array_mut().unwrap().push(g("void")); })),
        ("piles", Box::new(|e| { for _ in 0..990 { e["position"]["piles"]["discard"].as_array_mut().unwrap().push(g("wound")); } })),
        ("enemies.powers", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 0, "powers": {"enrage": 2}}]))),
        ("enemies.powers", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 0, "powers": {"artifact": 2}}]))),
        ("enemies.slot", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 1}]))),
        ("enemies.hp", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 0, "hp": 0}]))),
        ("enemies.hp", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 0, "hp": 900}]))),
        ("enemies.block", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 0, "block": -1}]))),
        ("player.block", Box::new(|e| e["position"]["player"]["block"] = json!(1000))),
        ("player.energy", Box::new(|e| e["position"]["player"]["energy"] = json!(-1))),
        ("player.mantra", Box::new(|e| e["position"]["player"]["mantra"] = json!(10))),
        ("player.orbs", Box::new(|e| { e["position"]["player"]["orb_slots"] = json!(1); e["position"]["player"]["orbs"] = json!(["frost", "frost"]); })),
        ("player.stance", Box::new(|e| e["position"]["player"]["stance"] = json!("rage"))),
        // Unknown keys are rejected at every level, naming that level (verifier v1 note 1).
        ("position: unknown field", Box::new(|e| e["position"]["unknown"] = json!(1))),
        ("position.player: unknown field", Box::new(|e| e["position"]["player"]["unknown"] = json!(1))),
        ("position.enemies[]: unknown field", Box::new(|e| e["position"]["enemies"] = json!([{"slot": 0, "unknown": 1}]))),
        ("position.piles: unknown field", Box::new(|e| e["position"]["piles"]["unknown"] = json!([]))),
        ("position.piles card row: unknown field", Box::new(|e| e["position"]["piles"]["hand"][0]["cost_modifier"] = json!(0))),
        ("position.piles card row: unknown field", Box::new(|e| e["position"]["piles"]["draw"][0]["unknown"] = json!(0))),
        ("position: invalid type", Box::new(|e| e["position"]["player"]["block"] = json!("x"))),
    ];
    for (field, edit) in edits {
        let mut e = base.clone();
        edit(&mut e);
        match parse(vec![e]) { Err(RunSessionError::Input(m)) => assert!(m.starts_with(field), "{field}: {m}"), other => panic!("{field}: {other:?}") }
    }
    // Hidden-state fields at any level are named.
    for name in ["order", "seed", "rng", "intent", "memory", "instance", "handle", "turn"] {
        for at in [vec!["position"], vec!["position", "player"], vec!["position", "piles"]] {
            let mut e = base.clone();
            let mut v = &mut e;
            for k in &at { v = &mut v[*k]; }
            v[name] = json!(1);
            match parse(vec![e]) { Err(RunSessionError::Input(m)) => assert!(m.contains(name), "{name} at {at:?}: {m}"), other => panic!("{name}: {other:?}") }
        }
        let mut e = base.clone();
        e["position"]["enemies"] = json!([{"slot": 0, name: 1}]);
        assert!(matches!(parse(vec![e]), Err(RunSessionError::Input(m)) if m.contains(name)), "{name} on an enemy");
        let mut e = base.clone();
        e["position"]["piles"]["hand"][0][name] = json!(1);
        assert!(matches!(parse(vec![e]), Err(RunSessionError::Input(m)) if m.contains(name)), "{name} on a card");
    }
    // Innate / bottled: the bottled Bash must be in the hand; an owned bottle must name its card (PM ruling c0 (6)).
    let mut e = base.clone();
    e["relics"] = json!(["burning_blood", "bottled_flame"]);
    assert!(matches!(parse(vec![e.clone()]), Err(RunSessionError::Input(m)) if m.starts_with("bottled_cards")));
    e["bottled_cards"] = json!({"bottled_flame": c("defend")});
    assert!(matches!(parse(vec![e.clone()]), Err(RunSessionError::Input(m)) if m.starts_with("bottled_cards")), "Bottled Flame takes an Attack");
    // Final review m1: an owned bottle with no card is explicit (`null`).
    e["bottled_cards"] = json!({"bottled_flame": null});
    let unlinked = parse(vec![e.clone()]).unwrap();
    assert_eq!(unlinked.descriptor()["entries"][0]["bottled_cards"], json!({"bottled_flame": null}));
    e["bottled_cards"] = json!({"bottled_flame": c("bash")});
    assert!(parse(vec![e.clone()]).is_ok());
    let hand = e["position"]["piles"]["hand"].as_array_mut().unwrap();
    hand.retain(|x| x["card"] != "bash");
    e["position"]["piles"]["draw"].as_array_mut().unwrap().push(c("bash"));
    assert!(matches!(parse(vec![e]), Err(RunSessionError::Input(m)) if m.starts_with("piles.hand")));
}

/// c1 (review I6): enemy states native play cannot produce at the opening decision are rejected: move-only powers (Gremlin
/// Nob's Enrage before Bellow, Cultist Ritual), structural powers, and edits to random formations.
#[test]
fn position_rejects_history_inconsistent_enemy_state() {
    let deck = starter(Character::Ironclad);
    for (encounter, edit) in [("gremlin_nob", json!({"slot": 0, "powers": {"enrage": 2}})), ("cultist", json!({"slot": 0, "powers": {"ritual": 3}})),
        ("guardian", json!({"slot": 0, "powers": {"mode_shift": 10}})), ("lagavulin", json!({"slot": 0, "powers": {"curl_up": 3}})),
        ("two_louse", json!({"slot": 0, "hp": 5})), ("cultist", json!({"slot": 0, "powers": {"metallicize": 3}}))] {
        let mut e = entry("x", Character::Ironclad, 0, encounter, deck.clone(), vec!["burning_blood"]);
        e["position"] = json!({"enemies": [edit]});
        assert!(matches!(parse(vec![e]), Err(RunSessionError::Input(m)) if m.starts_with("enemies")), "{encounter}");
    }
    // Final review I1 (the reviewer's probe): the shapes re-choose each slot's species from the seed, so an edit is rejected at
    // parse, naming the field (an A7 Repulsor admits HP 31 while an Exploder can roll 30).
    for encounter in ["three_shapes", "four_shapes", "sphere_and_two_shapes"] {
        let mut e = entry("x", Character::Ironclad, 7, encounter, deck.clone(), vec!["burning_blood"]);
        e["position"] = json!({"enemies": [{"slot": 0, "hp": 31}]});
        assert!(matches!(parse(vec![e]), Err(RunSessionError::Input(m)) if m.starts_with("enemies")), "{encounter}");
    }
    // Final review m3: a burning buff needs its declared buff (amended: the Nob control below now declares it).
    let mut e = entry("x", Character::Ironclad, 0, "gremlin_nob", deck.clone(), vec!["burning_blood"]);
    e["position"] = json!({"enemies": [{"slot": 0, "powers": {"metallicize": 3}}]});
    assert!(matches!(parse(vec![e]), Err(RunSessionError::Input(m)) if m.starts_with("enemies")), "undeclared burning buff");
    // Controls: start-applicable powers are admitted (the declared burning buff on an elite, a species' own setup power).
    for (encounter, burning, edit) in [("gremlin_nob", json!("metallicize"), json!({"slot": 0, "powers": {"metallicize": 3, "strength": 2}})),
        ("three_sentries", Value::Null, json!({"slot": 1, "powers": {"artifact": 0}}))] {
        let mut e = entry("x", Character::Ironclad, 0, encounter, deck.clone(), vec!["burning_blood"]);
        e["burning"] = burning;
        e["position"] = json!({"enemies": [edit]});
        assert!(parse(vec![e]).is_ok(), "{encounter}");
    }
}

/// c1: a rejected reset (here an unknown entry id; a rejected position at reset is `tests::m34s_recipes::
/// position_rejected_at_reset_has_no_effect`) changes nothing: boundary, token, serial and seed are unchanged, no RNG consumed.
#[test]
fn rejected_reset_has_no_effect() {
    let mut toolbox = entry("toolbox", Character::Ironclad, 0, "jaw_worm", starter(Character::Ironclad), vec!["burning_blood", "toolbox"]);
    toolbox["position"] = json!({});
    // Toolbox's opening offer is found by the trial construction at parse: UnsupportedBoundary before any session.
    assert!(matches!(parse(vec![toolbox]), Err(RunSessionError::UnsupportedBoundary(_))));
    let mut s = session(lessons(0));
    let before = s.reset(Some(5), Some("l1-ironclad-nob-a0")).unwrap();
    let diag = s.diagnostic();
    assert!(matches!(s.reset(Some(6), Some("missing")), Err(RunSessionError::Input(_))));
    assert_eq!(s.decision().unwrap(), before);
    assert_eq!(s.diagnostic(), diag);
    let again = s.reset(Some(5), Some("l1-ironclad-nob-a0")).unwrap();
    assert_eq!(again.observation, before.observation, "no RNG consumed: the same seed gives the same start");
}

/// c2: start labels; synthetic positions are never presented as reachable.
#[test]
fn position_is_labelled_synthetic() {
    let mut fresh = entry("fresh", Character::Ironclad, 0, "jaw_worm", starter(Character::Ironclad), vec!["burning_blood"]);
    fresh["id"] = json!("fresh");
    let mut entries = lessons(0);
    entries.push(fresh);
    let cat = parse(entries).unwrap();
    for e in cat.descriptor()["entries"].as_array().unwrap() {
        let kind = if e["id"] == "fresh" { "fresh_start" } else { "synthetic_position" };
        assert_eq!(e["start_kind"], kind, "{}", e["id"]);
    }
    let mut s = CombatTrainingSession::new_standard(cat, Partition::Test).unwrap();
    assert_eq!(s.reset(Some(1), Some("l2-silent-poison-a0")).unwrap().info["start_kind"], "synthetic_position");
    assert_eq!(s.reset(Some(1), Some("fresh")).unwrap().info["start_kind"], "fresh_start");
}

/// c2: through a session, the draw order is the gym's: same (entry, seed) ⇒ identical starts; different seeds ⇒ different
/// hidden orders behind the same published view (the hand, the known top and the draw multiset are the position's).
#[test]
fn position_draw_order_is_gym_owned_and_seeded() {
    let mut s = session(lessons(0));
    let a = s.reset(Some(10), Some("s-watcher-a0")).unwrap();
    assert_eq!(s.reset(Some(10), Some("s-watcher-a0")).unwrap(), CombatPolicyBoundary { token: s.decision().unwrap().token, ..a.clone() });
    let mut later = vec![];
    for seed in 10..16 {
        let b = s.reset(Some(seed), Some("s-watcher-a0")).unwrap();
        assert_eq!(b.observation.group("combat_cards").unwrap().values, a.observation.group("combat_cards").unwrap().values, "the published cards are the position's");
        // End the turn: the next hand reveals the hidden order below the known top.
        let next = s.step(b.token, RunActionIndex(end_turn(&b.observation) as i64)).unwrap();
        later.push(next.boundary.observation.group("combat_cards").unwrap().values.clone());
    }
    assert!(later.iter().any(|h| *h != later[0]), "different seeds draw different hidden orders");
}

/// c2: every lesson and smoke fixture, at A0 and A20, continues with random legal actions to the combat's terminal.
#[test]
fn position_continues_with_normal_actions_to_combat_end() {
    for ascension in [0, 20] {
        let fixtures = lessons(ascension);
        let ids = fixtures.iter().map(|e| e["id"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
        let mut s = session(fixtures);
        for id in ids {
            for seed in 0..3u64 {
                let mut b = s.reset(Some(seed), Some(&id)).unwrap();
                let mut x = seed;
                for step in 0..4_000 {
                    let legal = first_legal(&b.observation);
                    x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let r = s.step(b.token, RunActionIndex(legal[(x >> 33) as usize % legal.len()] as i64)).unwrap();
                    if r.terminated { break }
                    assert!(step < 3_999, "{id}: no terminal");
                    b = r.boundary;
                }
            }
        }
    }
}
