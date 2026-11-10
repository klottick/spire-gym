//! M34s-a a4: `standard-combat-v1` fresh starts (design §4.4).
use serde_json::{json, Value};
use spire_core::{combat::rng::Partition, content::{Catalog, Character}};
use spire_env::*;

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn starter_relic(c: Character) -> &'static str {
    match c { Character::Ironclad => "burning_blood", Character::Silent => "ring_of_the_snake", Character::Defect => "cracked_core", Character::Watcher => "pure_water" }
}
fn slots(ascension: u8) -> usize { if ascension >= 11 { 2 } else { 3 } }
/// A starter-resource entry for `character` at `ascension` fighting `encounter`.
fn entry(id: &str, character: Character, ascension: u8, encounter: &str) -> Value {
    let deck = Catalog::standard().starter_deck(character).unwrap().iter().map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect::<Vec<_>>();
    json!({"id": id, "character": character.as_str(), "ascension": ascension, "encounter": encounter, "hp": 70, "max_hp": 80, "gold": 99,
        "deck": deck, "relics": [starter_relic(character)], "potions": vec![Value::Null; slots(ascension)]})
}
fn catalog(entries: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": entries})).unwrap()
}
fn session(entries: Vec<Value>) -> CombatTrainingSession {
    CombatTrainingSession::new_standard(CombatTrainingCatalog::standard_from_json(&catalog(entries)).unwrap(), Partition::Test).unwrap()
}
fn vocab_index(family: &str, name: &str) -> i64 {
    standard_vocab_manifest()["families"][family].as_array().unwrap().iter().position(|v| v == name).unwrap() as i64
}
fn value(o: &RunObservation, group: &str, row: usize, col: usize) -> i64 { let g = o.group(group).unwrap(); g.values[row * g.shape.last().unwrap() + col] }
fn legal(o: &RunObservation) -> Vec<usize> { o.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect() }
fn with_verb(o: &RunObservation, verb: &str) -> Option<usize> { legal(o).into_iter().find(|i| value(o, "actions", *i, 0) == vocab_index("verb", verb)) }
/// Play cards while possible, otherwise end the turn (selections take their first row).
fn aggressive(o: &RunObservation) -> usize { with_verb(o, "play_card").or_else(|| with_verb(o, "end_turn")).unwrap_or_else(|| legal(o)[0]) }
fn passive(o: &RunObservation) -> usize { with_verb(o, "end_turn").unwrap_or_else(|| legal(o)[0]) }
fn run_to_terminal(s: &mut CombatTrainingSession, b: CombatPolicyBoundary, mut policy: impl FnMut(&RunObservation) -> usize) -> (CombatTrainingStep, usize) {
    let mut b = b;
    for steps in 1..=3_000 {
        let r = s.step(b.token, RunActionIndex(policy(&b.observation) as i64)).unwrap();
        assert!(!r.truncated);
        if r.terminated { return (r, steps) }
        assert_eq!(r.reward, 0);
        b = r.boundary;
    }
    panic!("no terminal in 3000 steps")
}
fn input_error(bytes: &[u8]) -> bool { matches!(CombatTrainingCatalog::standard_from_json(bytes), Err(RunSessionError::Input(_))) }

#[test]
fn standard_combat_catalog_is_strict() {
    let base = entry("a", Character::Ironclad, 0, "jaw_worm");
    assert!(CombatTrainingCatalog::standard_from_json(&catalog(vec![base.clone()])).is_ok());
    let edits: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        ("unknown entry field", Box::new(|e| e["act"] = json!(2))),
        ("draw_order", Box::new(|e| e["draw_order"] = json!(["strike"]))),
        ("seed", Box::new(|e| e["seed"] = json!(7))),
        ("rng", Box::new(|e| e["rng"] = json!({}))),
        ("hand", Box::new(|e| e["hand"] = json!([]))),
        ("position (M34s-c)", Box::new(|e| e["position"] = json!({}))),
        ("recipe (M34s-c)", Box::new(|e| e["recipe"] = json!({}))),
        ("unknown character", Box::new(|e| e["character"] = json!("necromancer"))),
        ("ascension 21", Box::new(|e| e["ascension"] = json!(21))),
        ("unknown encounter", Box::new(|e| e["encounter"] = json!("jaw_worm_deluxe"))),
        ("hp 0", Box::new(|e| e["hp"] = json!(0))),
        ("hp above max", Box::new(|e| e["hp"] = json!(81))),
        ("max hp 1000", Box::new(|e| { e["max_hp"] = json!(1000); e["hp"] = json!(1000) })),
        ("empty deck", Box::new(|e| e["deck"] = json!([]))),
        ("unknown card", Box::new(|e| e["deck"][0]["card"] = json!("strike_deluxe"))),
        ("upgrade domain", Box::new(|e| e["deck"][0]["upgrades"] = json!(2))),
        ("misc on a plain card", Box::new(|e| e["deck"][0]["misc"] = json!(3))),
        ("Genetic Algorithm below its obtain misc", Box::new(|e| e["deck"][0] = json!({"card": "genetic_algorithm", "upgrades": 0, "misc": 0}))),
        ("other character's relic", Box::new(|e| e["relics"] = json!(["pure_water"]))),
        ("duplicate relic", Box::new(|e| e["relics"] = json!(["burning_blood", "burning_blood"]))),
        ("starter and its boss replacement", Box::new(|e| e["relics"] = json!(["burning_blood", "black_blood"]))),
        ("counter without its relic", Box::new(|e| e["relic_counters"] = json!({"neows_lament": 2}))),
        ("other character's potion", Box::new(|e| e["potions"][0] = json!("poison_potion"))),
        ("A0 slot count", Box::new(|e| e["potions"] = json!([null, null]))),
        ("burning on a monster", Box::new(|e| e["burning"] = json!("strength"))),
        ("unknown burning buff", Box::new(|e| { e["encounter"] = json!("gremlin_nob"); e["burning"] = json!("haste") })),
        ("gold type", Box::new(|e| e["gold"] = json!(-1))),
    ];
    for (label, edit) in &edits {
        let mut e = base.clone(); edit(&mut e);
        assert!(input_error(&catalog(vec![e])), "{label} is rejected");
    }
    let mut a11 = entry("a", Character::Silent, 11, "jaw_worm");
    assert!(CombatTrainingCatalog::standard_from_json(&catalog(vec![a11.clone()])).is_ok(), "A11 has two slots");
    a11["potions"] = json!([null, null, null]);
    assert!(input_error(&catalog(vec![a11])), "A11 three slots");
    let mut burning = entry("a", Character::Ironclad, 0, "gremlin_nob"); burning["burning"] = json!("metallicize");
    assert!(CombatTrainingCatalog::standard_from_json(&catalog(vec![burning])).is_ok());
    let mut belt = entry("a", Character::Defect, 12, "jaw_worm"); belt["relics"] = json!(["cracked_core", "potion_belt"]); belt["potions"] = json!([null, null, null, null]);
    assert!(CombatTrainingCatalog::standard_from_json(&catalog(vec![belt])).is_ok(), "Potion Belt adds two slots");
    // Catalog-level rules.
    let top = |v: Value| serde_json::to_vec(&v).unwrap();
    assert!(input_error(&top(json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": [base], "extra": 1}))));
    assert!(input_error(&top(json!({"catalog_schema_id": "spire-gym-combat-entry-catalog-v2", "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": [base]}))));
    assert!(input_error(&catalog(vec![])));
    assert!(input_error(&catalog(vec![base.clone(), base.clone()])), "duplicate id");
    assert!(input_error(&catalog((0..257).map(|i| entry(&format!("e{i}"), Character::Ironclad, 0, "cultist")).collect())));
    // Normalised and fingerprinted: the deck is an unordered multiset.
    let mut shuffled = base.clone(); shuffled["deck"].as_array_mut().unwrap().reverse();
    let (a, b) = (CombatTrainingCatalog::standard_from_json(&catalog(vec![base.clone()])).unwrap(), CombatTrainingCatalog::standard_from_json(&catalog(vec![shuffled])).unwrap());
    assert_eq!((a.descriptor(), a.fingerprint()), (b.descriptor(), b.fingerprint()));
}

#[test]
fn standard_combat_starts_every_character_and_ascension() {
    let encounters = [("cultist", 1), ("small_slimes", 1), ("gremlin_nob", 1), ("hexaghost", 1), ("snecko", 2), ("book_of_stabbing", 2), ("champ", 2),
        ("orb_walker", 3), ("nemesis", 3), ("donu_and_deca", 3), ("shield_and_spear", 4), ("corrupt_heart", 4)];
    for character in CHARACTERS {
        for ascension in [0, 10, 20] {
            let mut entries = encounters.iter().map(|(e, _)| entry(e, character, ascension, e)).collect::<Vec<_>>();
            let mut burning = entry("burning", character, ascension, "lagavulin"); burning["burning"] = json!("strength"); entries.push(burning);
            let mut s = session(entries);
            for (e, act) in encounters.iter().copied().chain([("burning", 1)]) {
                let b = s.reset(Some(17), Some(e)).unwrap();
                let cc = b.observation.group("combat_context").unwrap();
                let encounter = if e == "burning" { "lagavulin" } else { e };
                assert_eq!(cc.values[1..], [i64::from(ascension), act, vocab_index("room", "combat"), vocab_index("encounter", encounter), i64::from(e == "burning")], "{character:?} A{ascension} {e}");
                assert_eq!(cc.known, [1; 6]);
                assert!(!legal(&b.observation).is_empty());
                assert_eq!((b.info["start_kind"].as_str(), b.info["entry_id"].as_str(), &b.info["terminal_reason"]), (Some("fresh_start"), Some(e), &Value::Null));
                assert_eq!(b.info["combat_view_id"], STANDARD_COMBAT_VIEW_ID);
            }
        }
    }
}

#[test]
fn standard_combat_reset_seed_semantics() {
    let mut s = session(vec![entry("a", Character::Watcher, 5, "gremlin_gang"), entry("b", Character::Watcher, 5, "three_louse")]);
    let first = s.reset(Some(42), None).unwrap();
    assert_eq!((first.info["entry_id"].as_str(), first.info["episode_ordinal"].as_u64()), (Some("a"), Some(0)), "the first entry is the default");
    let again = s.reset(Some(42), Some("a")).unwrap();
    assert_eq!(again.observation, first.observation, "an explicit seed reproduces the start");
    let next = s.reset(None, Some("a")).unwrap();
    assert_eq!(next.info["episode_ordinal"], 1, "an implicit reset advances the ordinal");
    assert_eq!(s.reset(Some(42), Some("a")).unwrap().info["episode_ordinal"], 0, "an explicit seed resets the ordinal");
    assert_ne!(first.token, again.token);
    assert!(matches!(s.reset(Some(1), Some("missing")), Err(RunSessionError::Input(_))));
    assert!(matches!(s.step(first.token, RunActionIndex(0)), Err(RunSessionError::StaleToken)));
}

#[test]
fn standard_combat_view_layout_equals_sidecar() {
    let mut s = session(vec![entry("a", Character::Defect, 20, "slavers")]);
    let spec = s.observation_spec().unwrap();
    let mut run = RunSession::new_standard(StandardRunConfig::new(Character::Defect, 20, Partition::Test)).unwrap();
    assert_eq!(spec, run.combat_observation_spec().unwrap(), "one Standard combat view descriptor and fingerprint");
    let env = s.environment_spec().unwrap();
    assert_eq!((env["profile_id"].as_str(), env["schema_id"].as_str(), env["combat_view_id"].as_str()), (Some(STANDARD_COMBAT_PROFILE_ID), Some(STANDARD_COMBAT_VIEW_ID), Some(STANDARD_COMBAT_VIEW_ID)));
    assert_eq!(env["observation_fingerprint"], spec.fingerprint().iter().map(|b| format!("{b:02x}")).collect::<String>());
    assert_eq!((env["rules_id"].as_str(), env["content_id"].as_str()), (Some(RUN_RULES_ID_STANDARD), Some(RUN_CONTENT_ID_STANDARD)));
    let b = s.reset(Some(3), None).unwrap();
    run.reset(Some(3)).unwrap();
    let sidecar = run.combat_view().unwrap();
    let keys = |o: &RunObservation| o.groups.iter().map(|g| (g.key, g.shape.clone())).collect::<Vec<_>>();
    assert_eq!(keys(&b.observation), keys(&sidecar.observation));
    let tensors = spec.descriptor()["tensors"].as_array().unwrap().iter().filter(|t| t["dtype"] == "int64").map(|t| (t["key"].as_str().unwrap().to_owned(), t["shape"].clone())).collect::<Vec<_>>();
    assert_eq!(b.observation.groups.iter().map(|g| (g.key.to_owned(), json!(g.shape))).collect::<Vec<_>>(), tensors);
    assert_eq!(b.observation.action_mask.len(), spec.descriptor()["action_space"]["n"].as_u64().unwrap() as usize);
}

#[test]
fn standard_combat_terminals_are_distinct() {
    let mut win = entry("win", Character::Ironclad, 0, "cultist");
    win["deck"] = json!(vec![json!({"card": "bludgeon", "upgrades": 1}); 10]);
    let mut lose = entry("lose", Character::Silent, 0, "jaw_worm"); lose["hp"] = json!(1);
    let mut escape = entry("escape", Character::Defect, 0, "cultist"); escape["potions"] = json!(["smoke_bomb", null, null]);
    let looter = entry("looter", Character::Watcher, 0, "looter");
    let mut s = session(vec![win, lose, escape, looter]);
    let (victory, _) = { let b = s.reset(Some(5), Some("win")).unwrap(); run_to_terminal(&mut s, b, aggressive) };
    assert_eq!((victory.reward, victory.boundary.info["terminal_reason"].as_str(), victory.boundary.info["all_enemies_escaped"].as_bool()), (1, Some("victory"), Some(false)));
    assert!(legal(&victory.boundary.observation).is_empty(), "no action after the terminal");
    assert!(matches!(s.step(victory.boundary.token, RunActionIndex(0)), Err(RunSessionError::Terminal)));
    let (death, _) = { let b = s.reset(Some(5), Some("lose")).unwrap(); run_to_terminal(&mut s, b, passive) };
    assert_eq!((death.reward, death.boundary.info["terminal_reason"].as_str()), (-1, Some("death")));
    let b = s.reset(Some(5), Some("escape")).unwrap();
    let smoke = with_verb(&b.observation, "use_potion").expect("Smoke Bomb is usable against a monster");
    let escaped = s.step(b.token, RunActionIndex(smoke as i64)).unwrap();
    assert!(escaped.terminated && !escaped.truncated);
    assert_eq!((escaped.reward, escaped.boundary.info["terminal_reason"].as_str()), (0, Some("escaped")));
    let (fled, _) = { let b = s.reset(Some(5), Some("looter")).unwrap(); run_to_terminal(&mut s, b, passive) };
    assert_eq!((fled.reward, fled.boundary.info["terminal_reason"].as_str(), fled.boundary.info["all_enemies_escaped"].as_bool()), (1, Some("victory"), Some(true)),
        "a thief's escape is a victory with its flag");
}

#[test]
fn standard_combat_terminal_is_the_combat_terminal() {
    // Burning Blood (a combat-end hook) applies; Meat on the Bone (settlement) and rewards never do.
    let mut burning_blood = entry("bb", Character::Ironclad, 0, "cultist");
    burning_blood["deck"] = json!(vec![json!({"card": "bludgeon", "upgrades": 1}); 10]); burning_blood["hp"] = json!(30);
    let mut meat = burning_blood.clone(); meat["id"] = json!("meat"); meat["relics"] = json!(["meat_on_the_bone"]);
    let mut s = session(vec![burning_blood, meat]);
    let (bb, _) = { let b = s.reset(Some(9), Some("bb")).unwrap(); run_to_terminal(&mut s, b, aggressive) };
    let (before, after) = (bb.boundary.info["hp_before_healing"].as_u64().unwrap(), bb.boundary.info["hp_after_cleanup"].as_u64().unwrap());
    assert_eq!(after, (before + 6).min(80), "Burning Blood heals at the combat's end");
    let (m, _) = { let b = s.reset(Some(9), Some("meat")).unwrap(); run_to_terminal(&mut s, b, aggressive) };
    assert!(m.boundary.info["hp_before_healing"].as_u64().unwrap() <= 40, "the fixture ends at or below half HP");
    assert_eq!(m.boundary.info["hp_after_cleanup"], m.boundary.info["hp_before_healing"], "Meat on the Bone is a settlement effect and does not apply");
    for t in [&bb, &m] {
        assert_eq!((t.boundary.info["combats_entered"].as_u64(), t.boundary.info["combats_settled"].as_u64()), (Some(1), Some(0)), "never settled");
        assert_eq!(t.boundary.info["last_exit"], Value::Null, "no settlement checkpoint");
        assert_eq!(t.boundary.info["active"], true, "the combat is still the published owner");
        assert!(t.boundary.observation.group("combat_state").unwrap().known.contains(&1));
    }
}

#[test]
fn standard_combat_faults_freeze_without_retry() {
    let mut s = session(vec![entry("a", Character::Silent, 3, "cultist")]);
    let b = s.reset(Some(1), None).unwrap();
    s.fail_delivery("test delivery fault");
    let fault = RunSessionError::DeliveryFault("test delivery fault");
    assert_eq!(s.decision().unwrap_err(), fault);
    assert_eq!(s.step(b.token, RunActionIndex(0)).unwrap_err(), fault);
    assert_eq!(s.reset(Some(1), None).unwrap_err(), fault, "a frozen session never resets into a fresh episode");
    assert_eq!(s.diagnostic()["fault"], json!(format!("{fault:?}")));
}

#[test]
fn standard_combat_outputs_carry_no_hidden_values() {
    const SEED: u64 = 0x5EED_CAFE_F00D_1234;
    let mut s = session(vec![entry("a", Character::Watcher, 20, "reptomancer")]);
    let b = s.reset(Some(SEED), None).unwrap();
    let r = s.step(b.token, RunActionIndex(aggressive(&b.observation) as i64)).unwrap();
    let texts = [b.info.to_string(), r.boundary.info.to_string(), s.environment_spec().unwrap().to_string(), s.observation_spec().unwrap().descriptor().to_string(), s.catalog_descriptor().to_string()];
    for text in &texts {
        assert!(!text.contains(&SEED.to_string()) && !text.contains(&format!("{SEED:x}")), "no seed in {}", &text[..text.len().min(80)]);
        for hidden in ["effective_seed", "draw_order", "\"rng\"", "ai_rng"] { assert!(!text.contains(hidden), "{hidden} in an output"); }
    }
    for o in [&b.observation, &r.boundary.observation] {
        assert!(o.groups.iter().all(|g| g.values.iter().all(|v| *v != SEED as i64)), "no seed in the observation");
    }
    assert_eq!(s.diagnostic()["effective_seed"], json!(SEED), "the effective seed is the explicit diagnostic oracle");
}

#[test]
fn ironclad_surfaces_reject_standard_catalogs() {
    let standard = catalog(vec![entry("a", Character::Ironclad, 0, "cultist")]);
    let boundary = RunSessionError::UnsupportedBoundary("standard-combat-v1 catalogs require the Standard combat session");
    assert_eq!(CombatTrainingCatalog::from_json(&standard).unwrap_err(), boundary, "the Gym66 catalog parser rejects Standard catalogs");
    let parsed = CombatTrainingCatalog::standard_from_json(&standard).unwrap();
    assert_eq!(CombatTrainingSession::new(parsed, Partition::Test).err(), Some(boundary));
    let gym66 = serde_json::to_vec(&json!({"profile_id": "ironclad-combat-v2", "entries": [{"id": "a", "encounter": "cultist", "hp": 80, "max_hp": 80,
        "deck": [{"card": "bash", "upgrades": 0}], "relics": [], "potions": [null, null, null]}]})).unwrap();
    assert_eq!(CombatTrainingCatalog::standard_from_json(&gym66).err(),
        Some(RunSessionError::UnsupportedBoundary("the Standard combat session requires a standard-combat-v1 catalog")));
    let gym66 = CombatTrainingCatalog::from_json(&gym66).unwrap();
    assert!(matches!(CombatTrainingSession::new_standard(gym66, Partition::Test), Err(RunSessionError::UnsupportedBoundary(_))));
}

#[test]
fn standard_combat_random_legal_rollouts() {
    let mut outcomes = Vec::new();
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let mut s = session(["exordium_thugs", "centurion_and_healer", "giant_head"].iter().map(|e| entry(e, character, ascension, e)).collect());
            for (ei, e) in ["exordium_thugs", "centurion_and_healer", "giant_head"].into_iter().enumerate() {
                let mut state = 0x9E37_79B9_7F4A_7C15_u64 ^ (ci * 8 + ei) as u64 ^ u64::from(ascension);
                let b = s.reset(Some(state), Some(e)).unwrap();
                let (t, steps) = run_to_terminal(&mut s, b, |o| { state ^= state << 13; state ^= state >> 7; state ^= state << 17; let l = legal(o); l[(state as usize) % l.len()] });
                outcomes.push(format!("{character:?} A{ascension} {e}: {} after {steps}", t.boundary.info["terminal_reason"]));
            }
        }
    }
    eprintln!("standard_combat_random_legal_rollouts: {}", outcomes.join("; "));
}

/// Review M1: the catalog admits only values the engine runs, so a valid entry never freezes at reset, whatever is drawn.
#[test]
fn standard_combat_catalog_bounds_match_the_engine() {
    let with_deck = |deck: Value| { let mut e = entry("a", Character::Ironclad, 0, "cultist"); e["deck"] = deck; e };
    for (label, deck) in [
        ("Ritual Dagger misc above the damage surface", json!([{"card": "ritual_dagger", "upgrades": 0, "misc": 3004}])),
        ("Ritual Dagger misc far above", json!([{"card": "ritual_dagger", "upgrades": 0, "misc": 2_000_000_000}])),
        ("Searing Blow 74", json!([{"card": "searing_blow", "upgrades": 74}])),
        ("Searing Blow 70,000", json!([{"card": "searing_blow", "upgrades": 70_000}])),
        ("Searing Blow 4,000,000,000", json!([{"card": "searing_blow", "upgrades": 4_000_000_000u32}])),
        ("Genetic Algorithm misc 1000", json!([{"card": "genetic_algorithm", "upgrades": 0, "misc": 1000}])),
        ("1000 cards", json!(vec![json!({"card": "strike", "upgrades": 0}); 1000])),
    ] {
        assert!(input_error(&catalog(vec![with_deck(deck)])), "{label} is rejected at validation");
    }
    // The largest admitted values run: every card is drawn and played (aggressive policy) without a fault.
    for deck in [json!(vec![json!({"card": "ritual_dagger", "upgrades": 1, "misc": 3003}); 5]),
        json!(vec![json!({"card": "searing_blow", "upgrades": 73}); 5])] {
        let mut s = session(vec![with_deck(deck)]);
        for seed in 1..=4 {
            let b = s.reset(Some(seed), None).unwrap();
            let (t, _) = run_to_terminal(&mut s, b, aggressive);
            assert_eq!(t.boundary.info["terminal_reason"], "victory");
        }
    }
    // 999 cards are admitted.
    assert!(CombatTrainingCatalog::standard_from_json(&catalog(vec![with_deck(json!(vec![json!({"card": "strike", "upgrades": 0}); 999]))])).is_ok());
}
