//! M34s-a a3: the Standard combat view and sidecar (design §4.2).
use super::standard_traversal::{context, current_node_is_boss, keys_route_action, legal_indices, needs_supplied_wound, short_creation, simple_action, standard_session, CHARACTERS};
use crate::*;
use spire_core::{combat::rng::Partition, content::Character, run::diagnostics::supplied_wound_enemies};
use std::collections::BTreeSet;

const STANDARD_COMBAT_VIEW_FINGERPRINT: &str = "5d8147e13ded030c7faa501a8202eb2588915c2e1330b30cac5b06afc9f9916e";

fn hex(bytes: &[u8; 32]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn tensor<'a>(d: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    d["tensors"].as_array().unwrap().iter().find(|t| t["key"] == key).unwrap_or_else(|| panic!("missing tensor {key}"))
}
fn column<'a>(t: &'a serde_json::Value, field: &str) -> &'a serde_json::Value {
    &t["columns"][t["fields"].as_array().unwrap().iter().position(|f| f == field).unwrap_or_else(|| panic!("missing column {field}"))]
}
fn value(o: &RunObservation, group: &str, row: usize, col: usize) -> i64 {
    let g = o.group(group).unwrap(); g.values[row * g.shape.last().unwrap() + col]
}
fn vocab_name(family: &str, index: i64) -> Option<String> {
    Some(crate::standard_vocab_manifest()["families"][family].as_array()?.get(usize::try_from(index).ok()?)?.as_str()?.to_owned())
}

#[test]
fn standard_combat_view_is_a_projection_of_standard() {
    for caps in [RunCapacities::standard_default(), RunCapacities { combat_cards: 300, actions: 900, ..RunCapacities::standard_default() }] {
        let full = standard_run_observation_spec(&caps).unwrap().descriptor();
        let view = standard_combat_observation_spec(&caps).unwrap().descriptor();
        assert_eq!(view["schema_id"], STANDARD_COMBAT_VIEW_ID);
        for key in ["vocabularies", "capacities", "action_space", "array_layout"] { assert_eq!(view[key], full[key], "{key}"); }
        let keys = view["tensors"].as_array().unwrap().iter().map(|t| t["key"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
        let groups = ["player", "permanent_cards", "combat_cards", "enemies", "powers", "relics", "potions", "orbs", "selection",
            "selection_counts", "actions", "combat_state", "known_draw", "stance_choices", "combat_card_offers"];
        let mut expected = vec!["combat_context".to_owned(), "combat_context_known".to_owned()];
        expected.extend(full["tensors"].as_array().unwrap().iter().map(|t| t["key"].as_str().unwrap().to_owned())
            .filter(|k| k == "action_mask" || groups.iter().any(|g| k == g || *k == format!("{g}_known") || *k == format!("{g}_occupied"))));
        assert_eq!(keys, expected);
        for key in &keys[2..] { assert_eq!(tensor(&view, key), tensor(&full, key), "{key} is Standard's"); }
        // combat_context ⊆ context, plus the live encounter (context's encounter column) and the map's burning flag.
        let (cc, ctx) = (tensor(&view, "combat_context"), tensor(&full, "context"));
        assert_eq!(cc["fields"], serde_json::json!(["character", "ascension", "act", "room_kind", "encounter", "burning"]));
        for field in ["character", "ascension", "act", "room_kind"] { assert_eq!(column(cc, field), column(ctx, field), "{field}"); }
        let mut encounter = column(ctx, "last_encounter").clone(); encounter["field"] = serde_json::json!("encounter");
        assert_eq!(column(cc, "encounter"), &encounter);
        assert_eq!(column(cc, "encounter")["vocabulary"], "encounter");
        assert_eq!(column(cc, "burning"), column(tensor(&full, "map_nodes"), "burning"));
        for (i, c) in cc["columns"].as_array().unwrap().iter().enumerate() {
            assert_eq!((&cc["low"][i], &cc["high"][i], &cc["units"][i]), (&c["low"], &c["high"], &c["unit"]));
        }
    }
}

#[test]
fn standard_combat_view_descriptor_is_deterministic() {
    use sha2::{Digest, Sha256};
    let caps = RunCapacities::standard_default();
    let (a, b) = (standard_combat_observation_spec(&caps).unwrap(), standard_combat_observation_spec(&caps).unwrap());
    assert_eq!((a.descriptor(), a.fingerprint()), (b.descriptor(), b.fingerprint()));
    let mut h = Sha256::new(); h.update(b"spire-gym/combat-view/standard/v1\0"); h.update(crate::canonical_json(&a.descriptor()).unwrap());
    assert_eq!(a.fingerprint(), <[u8; 32]>::from(h.finalize()));
    assert_eq!(hex(&a.fingerprint()), STANDARD_COMBAT_VIEW_FINGERPRINT);
    assert_ne!(a.fingerprint(), standard_run_observation_spec(&caps).unwrap().fingerprint());
    // The session's spec is this one, and the environment names the view.
    let session = RunSession::new_standard(StandardRunConfig::new(Character::Silent, 7, Partition::Test)).unwrap();
    assert_eq!(session.combat_observation_spec().unwrap(), a);
    assert_eq!(session.environment_spec().unwrap()["combat_view_id"], STANDARD_COMBAT_VIEW_ID);
}

/// The sidecar at one boundary: groups equal the full observation's (all of them in combat; outside combat the resources,
/// with empty actions, selection and mask), values inside the view descriptor domains, and the shared token.
struct Bounds(std::collections::BTreeMap<String, (serde_json::Value, Vec<(i64, i64)>)>);
impl Bounds {
    fn of(spec: &serde_json::Value) -> Self {
        Bounds(spec["tensors"].as_array().unwrap().iter().filter(|t| t["dtype"] == "int64").map(|t| {
            let cols = t["low"].as_array().unwrap().iter().zip(t["high"].as_array().unwrap()).map(|(l, h)| (l.as_i64().unwrap(), h.as_i64().unwrap())).collect();
            (t["key"].as_str().unwrap().to_owned(), (t["shape"].clone(), cols))
        }).collect())
    }
}
fn assert_sidecar(session: &mut RunSession, bounds: &Bounds) -> Option<(i64, i64)> {
    let full = session.decision().unwrap();
    let view = session.combat_view().unwrap();
    assert_eq!(view.token, full.token);
    let in_combat = full.observation.group("combat_state").unwrap().known.iter().any(|k| *k == 1);
    assert_eq!(view.info["active"], in_combat);
    assert_eq!(view.info["combat_view_id"], STANDARD_COMBAT_VIEW_ID);
    for g in &view.observation.groups {
        let (shape, columns) = &bounds.0[g.key]; let cols = *g.shape.last().unwrap();
        assert_eq!(&serde_json::json!(g.shape), shape, "{} shape", g.key);
        for (i, v) in g.values.iter().enumerate() {
            let (low, high) = columns[i % cols];
            assert!((low..=high).contains(v), "{}[{i}] = {v}", g.key);
        }
        if g.key == "combat_context" { continue; }
        let expected = full.observation.group(g.key).unwrap();
        if in_combat || !matches!(g.key, "actions" | "selection" | "selection_counts") { assert_eq!(g, expected, "{} equals the full observation", g.key); }
        else { assert!(g.occupied.as_ref().map_or(true, |o| o.iter().all(|x| *x == 0)), "{} is empty outside combat", g.key); }
    }
    let cc = view.observation.group("combat_context").unwrap();
    for (i, col) in [0, 1, 2, 4].into_iter().enumerate() {
        let ctx = full.observation.group("context").unwrap();
        assert_eq!((cc.values[i], cc.known[i]), (ctx.values[col], ctx.known[col]), "combat_context[{i}]");
    }
    if in_combat {
        assert_eq!(view.observation.action_mask, full.observation.action_mask);
        assert_eq!(view.info["routing"], if full.observation.group("selection").unwrap().occupied.as_ref().unwrap().contains(&1) { "combat_selection" } else { "combat" });
        assert_eq!(cc.known[4..], [1, 1]);
        // Burning = the current map node is a burning elite (independent public source: map_nodes); event combats are not.
        let node = usize::try_from(value(&full.observation, "current_node", 0, 0)).ok();
        let burning = node.is_some_and(|row| value(&full.observation, "map_nodes", row, 5) == 1
            && vocab_name("map_kind", value(&full.observation, "map_nodes", row, 3)).as_deref() == Some("elite")
            && context(&full.observation).2 == "combat");
        assert_eq!(cc.values[5], i64::from(burning), "burning at {:?}", context(&full.observation));
        Some((cc.values[4], cc.values[5]))
    } else {
        assert!(view.observation.action_mask.iter().all(|m| *m == 0));
        assert_eq!(view.info["routing"], "noncombat");
        assert_eq!((&cc.values[4..], &cc.known[4..]), (&[0, 0][..], &[0, 0][..]));
        None
    }
}

/// The current map node's room: its resolved room when published (a `?` node), else its kind.
fn current_node_kind(obs: &RunObservation) -> Option<String> {
    let row = usize::try_from(value(obs, "current_node", 0, 0)).ok()?;
    let g = obs.group("map_nodes").unwrap(); let cols = *g.shape.last().unwrap();
    let resolved = (g.known[row * cols + 4] == 1).then(|| g.values[row * cols + 4]).filter(|v| *v != 0);
    vocab_name("map_kind", resolved.unwrap_or(g.values[row * cols + 3]))
}
/// `simple_action`, except that on the map it moves to a `?` node when one is offered (event combats).
fn event_seeking(obs: &RunObservation) -> usize {
    if context(obs).2 == "map" {
        let question = legal_indices(obs).into_iter().find(|&i| vocab_name("verb", value(obs, "actions", i, 0)).as_deref() == Some("move")
            && vocab_name("row_group", value(obs, "actions", i, 3)).as_deref() == Some("map_nodes")
            && usize::try_from(value(obs, "actions", i, 4)).ok().and_then(|row| vocab_name("map_kind", value(obs, "map_nodes", row, 3))).as_deref() == Some("question"));
        if let Some(index) = question { return index }
    }
    simple_action(obs)
}

#[derive(Default)]
struct RouteStats { error: Option<String>, steps: usize, act3_bosses: usize, event_combats: usize, burning_elites: i64, encounters: BTreeSet<String>, terminal: Option<spire_core::run::RunTerminal> }

/// Drive `session` with `policy` (and the supplied hit, as `standard_traversal::drive`), checking the sidecar at every
/// published boundary.
fn check_route(session: &mut RunSession, bounds: &Bounds, wound: bool, limit: usize, mut policy: impl FnMut(&RunObservation) -> usize) -> RouteStats {
    let mut stats = RouteStats::default();
    let mut boundary = session.decision().unwrap();
    let (mut was_act3_boss, mut current) = (false, None::<i64>);
    while boundary.terminal.is_none() && stats.steps < limit {
        let seen = assert_sidecar(session, bounds);
        let obs = &boundary.observation;
        if let Some((encounter, burning)) = seen {
            if current.is_none() {
                stats.encounters.insert(vocab_name("encounter", encounter).unwrap());
                if current_node_kind(obs).as_deref() == Some("event") { stats.event_combats += 1; }
                stats.burning_elites += burning;
            }
            assert!(current.is_none_or(|c| c == encounter), "encounter changed inside one combat");
            current = Some(encounter);
        } else if let Some(encounter) = current.take() {
            // After settlement the Run context names the combat just fought.
            assert_eq!(value(obs, "context", 0, 15), encounter, "last_encounter after the combat");
        }
        let is_act3_boss = context(obs).0 == 3 && context(obs).2 == "combat" && current_node_is_boss(obs);
        if is_act3_boss && !was_act3_boss { stats.act3_bosses += 1; }
        was_act3_boss = is_act3_boss;
        if wound && needs_supplied_wound(obs) {
            match session.apply_supplied(supplied_wound_enemies) {
                Ok(next) => { boundary = next; assert_sidecar(session, bounds); }
                // As `drive`: a supplied hit the Run rejects ends the route (reported; Heart routes still require HeartVictory).
                Err(error) => { stats.error = Some(format!("{error:?}")); break; }
            }
        }
        let index = policy(&boundary.observation);
        assert!(legal_indices(&boundary.observation).contains(&index));
        boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
        stats.steps += 1;
    }
    if stats.error.is_none() { assert_sidecar(session, bounds); }
    stats.terminal = boundary.terminal;
    stats
}

#[test]
fn standard_sidecar_equals_full_observation_combat_groups() {
    let bounds = Bounds::of(&standard_combat_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor());
    let mut summary = Vec::new();
    let (mut event_combats, mut burning_elites) = (0, 0);
    // The supplied Heart routes: every Act, the A20 second Act 3 boss and the Heart.
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0xB0 + character_index as u8 + if ascension == 20 { 8 } else { 0 };
            let (mut session, _) = standard_session(character, ascension, seed, short_creation());
            let stats = check_route(&mut session, &bounds, true, 12_000, simple_action);
            assert_eq!(stats.terminal, Some(spire_core::run::RunTerminal::HeartVictory), "{character:?} A{ascension}");
            assert!(stats.encounters.contains("corrupt_heart"), "{character:?} A{ascension} fought the Heart");
            assert_eq!(stats.act3_bosses, if ascension == 20 { 2 } else { 1 }, "{character:?} A{ascension} Act 3 bosses");
            summary.push(format!("{character:?} A{ascension} Heart route: {} steps, {} encounters", stats.steps, stats.encounters.len()));
            (event_combats, burning_elites) = (event_combats + stats.event_combats, burning_elites + stats.burning_elites);
        }
    }
    // The keys route (burning elite for the Emerald key) and random-legal Standard rollouts (event combats).
    for (ascension, seed) in [0_u8, 20].into_iter().flat_map(|a| [0xC1_u8, 0xC2, 0xC3, 0xC4].map(|s| (a, s))) {
        let mut config = StandardRunConfig::new(Character::Ironclad, ascension, Partition::Test);
        config.creation = short_creation();
        let run = spire_core::run::diagnostics::standard_supplied_run_with(Character::Ironclad, ascension, short_creation().profile(), 900, false,
            spire_core::run::RunRng::new([seed; 32], Partition::Test, 0)).unwrap();
        let mut session = RunSession::from_standard_supplied(config, run, u64::from(seed)).unwrap();
        let stats = check_route(&mut session, &bounds, true, 3_000, keys_route_action);
        summary.push(format!("keys route A{ascension} seed {seed:#x}: {} steps, {} burning, {:?}, error {:?}", stats.steps, stats.burning_elites, stats.terminal, stats.error));
        (event_combats, burning_elites) = (event_combats + stats.event_combats, burning_elites + stats.burning_elites);
    }
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        let (mut session, _) = standard_session(character, 0, 0xE0 + character_index as u8, short_creation());
        let stats = check_route(&mut session, &bounds, true, 12_000, event_seeking);
        summary.push(format!("{character:?} A0 event-seeking route: {} steps, {} event combats, {:?}, error {:?}", stats.steps, stats.event_combats, stats.terminal, stats.error));
        event_combats += stats.event_combats;
    }
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0x5A00_u64 + (character_index as u64 * 2) + u64::from(ascension == 20);
            let mut session = RunSession::new_standard(StandardRunConfig::new(character, ascension, Partition::Test)).unwrap();
            session.reset(Some(seed)).unwrap();
            let mut state = seed ^ 0x9E3779B97F4A7C15;
            let stats = check_route(&mut session, &bounds, false, 3_000, |obs| {
                state ^= state << 13; state ^= state >> 7; state ^= state << 17;
                let legal = legal_indices(obs); legal[(state as usize) % legal.len()]
            });
            summary.push(format!("{character:?} A{ascension} random: {} steps, {} event combats, {:?}", stats.steps, stats.event_combats, stats.terminal));
            event_combats += stats.event_combats;
        }
    }
    eprintln!("standard_sidecar_equals_full_observation_combat_groups: {}; event combats {event_combats}, burning elites {burning_elites}", summary.join("; "));
    assert!(event_combats > 0, "the fixtures include an event combat");
    assert!(burning_elites > 0, "the fixtures include a burning elite");
}

#[test]
fn standard_sidecar_routing_mask_and_token() {
    let mut config = StandardRunConfig::new(Character::Defect, 3, Partition::Test);
    config.record_replay = true;
    let mut session = RunSession::new_standard(config).unwrap();
    assert!(matches!(session.combat_view(), Err(RunSessionError::NotReset)));
    let mut boundary = session.reset(Some(11)).unwrap();
    let view = session.combat_view().unwrap();
    assert_eq!((view.info["routing"].as_str(), view.info["active"].as_bool(), &view.info["last_exit"]), (Some("noncombat"), Some(false), &serde_json::Value::Null));
    assert_eq!(view.token, boundary.token);
    // Dispatch through the sidecar: its legal rows and token drive the Run until the first combat ends.
    let (mut entered, mut settled_exit) = (false, None);
    for _ in 0..400 {
        let view = session.combat_view().unwrap();
        if view.info["active"] == true {
            entered = true;
            let index = view.observation.action_mask.iter().position(|m| *m == 1).unwrap();
            assert_eq!(view.observation.action_mask, boundary.observation.action_mask);
            boundary = session.step(view.token, RunActionIndex(index as i64)).unwrap().boundary;
        } else {
            if entered { settled_exit = Some(view.info["last_exit"].clone()); break; }
            let index = legal_indices(&boundary.observation)[0];
            boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
        }
        // A stale token from the sidecar is rejected like the Run's.
        assert!(session.step(view.token, RunActionIndex(0)).is_err() || view.token == boundary.token);
    }
    let exit = settled_exit.expect("the first combat settled");
    assert_eq!(exit["visit"], 0);
    assert!(exit["encounter"].is_string());
    // last_exit is sticky across later noncombat decisions.
    let index = legal_indices(&boundary.observation)[0];
    session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
    let later = session.combat_view().unwrap();
    if later.info["active"] == false { assert_eq!(later.info["last_exit"], exit); }
}

#[test]
fn captured_sidecars_unchanged() {
    // Latest packet (builds/d6572940/packet/logs/fingerprints.json): view-v1, view-v2.
    let m10 = RunSession::new_m10(RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test,
        boss_history: RunBossHistory::AwakenedUnseen, capacities: RunCapacities::m10_default(), record_replay: false }).unwrap();
    assert_eq!(hex(&m10.combat_observation_spec().unwrap().fingerprint()), "4c73f7de04d7b2875f473f8034c1cbacf984ea2d52d0a9428289c30cf1dff463");
    assert_eq!(m10.environment_spec().unwrap()["combat_view_id"], COMBAT_POLICY_VIEW_ID);
    assert_eq!(hex(&combat_policy_observation_spec_v2().unwrap().fingerprint()), "5ba729fd1b10a067fb8fc5d777d78b47fc9526ec0af0030d6868a32b52a7824b");
    assert_ne!(standard_combat_observation_spec(&RunCapacities::standard_default()).unwrap().fingerprint(), combat_policy_observation_spec().unwrap().fingerprint());
}
