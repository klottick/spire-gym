//! M34s-a a1: Standard draw rows (design §4.3) and the unchanged captured/policy fingerprints.
use crate::*;
use spire_core::{combat::rng::Partition, content::{Character, RelicId}, run::{CreationProfile, RoomScenario, RunConfig, Run, RunRng}};

fn standard_combat(character: Character, ascension: u8, seed: u8) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile, RunRng::new([seed; 32], Partition::Test, 0), RoomScenario::Standard).unwrap();
    for _ in 0..500 {
        let decision = run.decision().unwrap();
        if decision.combat.is_some() { return run; }
        let action = decision.candidates.first().unwrap().action.clone();
        run.step(&action).unwrap();
    }
    panic!("first-legal policy did not reach combat");
}

fn encode(d: &spire_core::run::PublicDecision, ascension: u8) -> RunObservation {
    crate::run_observation::encode_run_standard(d, None, &RunCapacities::standard_default(),
        crate::run_observation::StandardContext { ascension, run_selection: None }).unwrap()
}

fn column(field: &str) -> usize {
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let tensor = spec["tensors"].as_array().unwrap().iter().find(|t| t["key"] == "combat_cards").unwrap();
    tensor["fields"].as_array().unwrap().iter().position(|f| f == field).unwrap()
}

/// (values, known) of every occupied combat_cards row in zone `draw`.
fn draw_rows(o: &RunObservation) -> Vec<(Vec<i64>, Vec<i8>)> {
    let g = o.group("combat_cards").unwrap(); let cols = *g.shape.last().unwrap();
    let draw = crate::run_schema_standard::zone("draw").unwrap(); let zone = column("zone");
    (0..g.shape[0]).filter(|r| g.occupied.as_ref().unwrap()[*r] == 1 && g.values[r * cols + zone] == draw)
        .map(|r| (g.values[r * cols..(r + 1) * cols].to_vec(), g.known[r * cols..(r + 1) * cols].iter().map(|k| *k as i8).collect())).collect()
}

#[test]
fn standard_publishes_draw_multiset_in_canonical_order() {
    for character in Character::ALL {
        let run = standard_combat(character, 0, 41);
        let d = run.decision().unwrap(); let c = d.combat.as_ref().unwrap();
        let known = c.known_draw.iter().map(|k| k.card.instance).collect::<std::collections::BTreeSet<_>>();
        assert!(c.draw_count > 0, "{character:?}: a combat start has a draw pile");
        let rows = draw_rows(&encode(&d, 0));
        assert_eq!(rows.len(), c.draw_count - known.len(), "{character:?}: one row per unknown draw card");
        let (position, card) = (column("zone_position"), column("card_kind"));
        for (values, known) in &rows {
            assert_eq!((values[position], known[position]), (-1, 0), "{character:?}: draw position is unknown");
            assert_eq!(known[card], 1);
        }
        let mut published = rows.iter().map(|(v, _)| v[card]).collect::<Vec<_>>();
        let mut expected = c.draw_unknown.iter().map(|v| crate::run_schema_standard::card(v.id).unwrap()).collect::<Vec<_>>();
        published.sort(); expected.sort();
        assert_eq!(published, expected, "{character:?}: the rows are the draw multiset");
        // Draw rows come after every other occupied row, so known references are unchanged.
        let g = encode(&d, 0); let kd = g.group("known_draw").unwrap();
        assert!(kd.occupied.as_ref().unwrap().iter().filter(|o| **o == 1).count() == c.known_draw.len());
    }
}

#[test]
fn draw_rows_do_not_follow_physical_order() {
    let run = standard_combat(Character::Ironclad, 0, 42);
    let d = run.decision().unwrap();
    let base = encode(&d, 0);
    let n = d.combat.as_ref().unwrap().draw_unknown.len(); assert!(n > 2);
    for rotation in 1..n {
        let mut permuted = d.clone(); permuted.combat.as_mut().unwrap().draw_unknown.rotate_left(rotation);
        assert_eq!(encode(&permuted, 0), base, "rotation {rotation}");
    }
    // Same-id generated cards whose value columns differ (e.g. Steam Barrier or Glass Knife after decrements)
    // are ordered by their values, not by input order.
    let mut with_copies = d.clone(); let draw = &mut with_copies.combat.as_mut().unwrap().draw_unknown;
    let mut a = draw[0].clone(); a.instance = spire_core::combat::CardInstanceId(9_001); a.origin = spire_core::combat::CombatCardRef::CombatLocal(9_001);
    let mut b = a.clone(); b.instance = spire_core::combat::CardInstanceId(9_002); b.origin = spire_core::combat::CombatCardRef::CombatLocal(9_002);
    if let Some(values) = b.values.as_mut() { values.base_damage = Some(values.base_damage.unwrap_or(0) + 3); } else { b.misc += 3; }
    draw.push(a.clone()); draw.push(b.clone());
    with_copies.combat.as_mut().unwrap().draw_count += 2;
    let forward = encode(&with_copies, 0);
    let draw = &mut with_copies.combat.as_mut().unwrap().draw_unknown; let len = draw.len(); draw.swap(len - 1, len - 2);
    assert_eq!(encode(&with_copies, 0), forward, "same-id copies with different values are canonically ordered");
    let rows = draw_rows(&forward);
    assert_eq!(rows.len(), n + 2);
    // Review M6: the published order IS the canonical key (the row without zone and zone_position, permanent_row last;
    // unknown < known), so the two copies appear by their values, never by input order.
    let (zone_position, permanent_row) = (column("zone_position"), column("permanent_row"));
    let key = |(values, known): &(Vec<i64>, Vec<i8>)| (0..values.len()).filter(|c| *c != column("zone") && *c != zone_position && *c != permanent_row)
        .chain([permanent_row]).map(|c| (known[c] == 1, if known[c] == 1 { values[c] } else { 0 })).collect::<Vec<_>>();
    let mut sorted = rows.clone(); sorted.sort_by_key(key);
    assert_eq!(rows, sorted, "draw rows follow the canonical card key");
}

#[test]
fn frozen_eye_draw_rows_are_known_in_order() {
    let run = standard_combat(Character::Silent, 0, 43);
    let mut d = run.decision().unwrap();
    // A Frozen Eye combat publishes the whole pile in order as known draw rows, so no unknown draw rows remain.
    let c = d.combat.as_mut().unwrap();
    let all = std::mem::take(&mut c.draw_unknown);
    c.known_draw = all.into_iter().enumerate().map(|(offset, card)| spire_core::combat::KnownDrawCardView { card, from_top: true, offset }).collect();
    assert!(draw_rows(&encode(&d, 0)).is_empty());
    let _ = RelicId::FrozenEye;
}

/// Final review N2: a Standard combat whose core view lacks the draw multiset (entered without public knowledge) is a typed
/// error, never an observation with the draw rows silently missing.
#[test]
fn standard_encoder_rejects_a_combat_without_public_knowledge() {
    let run = standard_combat(Character::Defect, 0, 43);
    let mut d = run.decision().unwrap();
    assert!(encode_checked(&d).is_ok());
    let c = d.combat.as_mut().unwrap();
    assert!(c.draw_count > 0 && !c.draw_unknown.is_empty());
    c.draw_unknown.clear();
    assert_eq!(encode_checked(&d).unwrap_err(), RunSessionError::Input("Standard combat draw multiset missing (combat entered without public knowledge)"));
}
fn encode_checked(d: &spire_core::run::PublicDecision) -> Result<RunObservation, RunSessionError> {
    crate::run_observation::encode_run_standard(d, None, &RunCapacities::standard_default(),
        crate::run_observation::StandardContext { ascension: 0, run_selection: None })
}

#[test]
fn captured_and_policy_fingerprints_unchanged() {
    let hex = |s: RunObservationSpec| s.fingerprint.iter().map(|b| format!("{b:02x}")).collect::<String>();
    // Latest packet (builds/d6572940/packet/logs/fingerprints.json).
    assert_eq!(hex(combat_policy_observation_spec().unwrap()), "4c73f7de04d7b2875f473f8034c1cbacf984ea2d52d0a9428289c30cf1dff463");
    assert_eq!(hex(combat_policy_observation_spec_v2().unwrap()), "5ba729fd1b10a067fb8fc5d777d78b47fc9526ec0af0030d6868a32b52a7824b");
    assert_eq!(hex(m10_run_observation_spec(&RunCapacities::m10_default()).unwrap()), "dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21");
    assert_eq!(hex(policy_run_observation_spec(&RunCapacities::m10_default()).unwrap()), "28d92d89c437eea7fdc260240585bca466dee1fd74b9b3eb38513d2b34b0be64");
    assert_eq!(hex(run_observation_spec(&RunCapacities::m9_default()).unwrap()), "270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c");
}

/// M34s-a item 5 (PM ruling): every Standard card row carries the card's lifecycle values, so two rows are equal only
/// when their free-play waivers, rolled and modified costs, Blood for Blood discount and Infernal Blade cost-0 agree.
#[test]
fn standard_rows_publish_card_lifecycle() {
    let run = standard_combat(Character::Ironclad, 0, 41);
    let mut d = run.decision().unwrap();
    let c = d.combat.as_mut().unwrap();
    let lifecycle = spire_core::combat::CardLifecycleView { free_play: true, free_to_play_once: true, randomized_cost: Some(3),
        mutable_combat_cost: Some(2), combat_cost_reduction: 1, infernal_cost_this_turn: true };
    for card in c.hand.iter_mut().chain(c.draw_unknown.iter_mut()) { card.lifecycle = lifecycle; }
    let expected_rows = c.hand.len() + c.draw_unknown.len();
    let o = encode(&d, 0); let g = o.group("combat_cards").unwrap(); let cols = *g.shape.last().unwrap();
    let fields = ["autoplay_free", "free_play_once", "rolled_cost", "modified_combat_cost", "hp_loss_discount", "cost_zero_this_turn"];
    let rows = (0..g.shape[0]).filter(|r| g.occupied.as_ref().unwrap()[*r] == 1).collect::<Vec<_>>();
    assert_eq!(rows.len(), expected_rows, "a combat start publishes only hand and draw rows");
    for row in rows {
        let values = fields.map(|f| (g.values[row * cols + column(f)], g.known[row * cols + column(f)]));
        assert_eq!(values, [(1, 1), (1, 1), (3, 1), (2, 1), (1, 1), (1, 1)], "row {row}");
    }
    assert_eq!(column("cost_zero_this_turn"), cols - 1, "the lifecycle columns are the last six");
}
