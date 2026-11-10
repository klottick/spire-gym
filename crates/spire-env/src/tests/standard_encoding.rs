use crate::*;
use spire_core::{
    combat::{rng::Partition, CombatAction, FullPowerId, FullPowerView, TargetId},
    content::{CardId, CardType, Character, EnemyId, EventId, PotionId, RelicId},
    run::{
        Candidate, CreationProfile, RoomScenario, RunConfig, ChoiceId, GameAction, NeowDrawback,
        NeowReward, NeowSelectOp, PublicSelection, RoomChoiceKind, RoomChoiceStatus,
        RoomChoiceView, RoomParams, RoomSelectionOperation, RoomView, RoomViewKind, Run,
        RunRng, SelectionId, RunTerminal,
    },
};
use serde_json::Value;

/// The public Standard constructor (as `diagnostics::standard_run_at`, without the m29 feature).
fn standard_run_at(character: Character, ascension: u8, full_neow: bool, rng: RunRng) -> Result<Run, spire_core::run::RunError> {
    let profile = CreationProfile { neow_eligible: full_neow, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension }, profile, rng, RoomScenario::Standard)
}

fn standard_start(character: Character, ascension: u8, seed: u8) -> (Run, spire_core::run::map::PublicMap) {
    let mut run = standard_run_at(
        character,
        ascension,
        false,
        RunRng::new([seed; 32], Partition::Test, 0),
    )
    .unwrap();
    for _ in 0..500 {
        let decision = run.decision().unwrap();
        if decision.combat.is_some() {
            let map = run.public_map().unwrap();
            return (run, map);
        }
        let action = decision
            .candidates
            .first()
            .expect("nonterminal Standard decision has a candidate")
            .action
            .clone();
        run.step(&action).unwrap();
    }
    panic!("first-legal policy did not reach combat");
}

fn decision(run: &Run) -> spire_core::run::PublicDecision {
    run.decision().unwrap()
}

fn encoded(
    d: &spire_core::run::PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
    ascension: u8,
) -> RunObservation {
    crate::run_observation::encode_run_standard(
        d,
        map,
        &RunCapacities::standard_default(),
        crate::run_observation::StandardContext {
            ascension,
            run_selection: None,
        },
    )
    .unwrap()
}

fn group_value<'a>(observation: &'a RunObservation, key: &str) -> &'a crate::RunFeatureGroup {
    observation.group(key).unwrap()
}

fn descriptor_column(spec: &Value, group: &str, field: &str) -> (i64, i64) {
    let tensor = spec["tensors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["key"] == group)
        .unwrap();
    let i = tensor["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|v| v == field)
        .unwrap();
    (
        tensor["columns"][i]["low"].as_i64().unwrap(),
        tensor["columns"][i]["high"].as_i64().unwrap(),
    )
}

fn descriptor_field_index(spec: &Value, group: &str, field: &str) -> usize {
    spec["tensors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["key"] == group)
        .unwrap()["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|v| v == field)
        .unwrap()
}

#[test]
fn standard_preset_covers_engine_bounds() {
    let c = RunCapacities::standard_default();
    // Engine bounds and justified bounds from M34a lane B bounds-v2.json.
    for (name, got, minimum) in [
        ("permanent_cards", c.permanent_cards, 1000),
        ("selection", c.selection, 1000),
        ("combat_cards", c.combat_cards, 999),
        ("enemies", c.enemies, 9),
        ("powers", c.powers, 138 * (1 + 12)),
        ("relics", c.relics, 151),
        ("potions", c.potions, 5),
        ("offers", c.offers, 14),
        ("orbs", c.orbs, 10),
        ("map_nodes", c.map_nodes, 107),
        ("map_edges", c.map_edges, 90),
        ("actions", c.actions, 1000),
    ] {
        assert!(got >= minimum, "{name}: capacity {got} < engine bound {minimum}");
    }
    assert!(crate::run_schema_standard::validate(&c).is_ok());
}

#[test]
fn standard_full_and_short_neow_start_screens_encode() {
    for (full, seed) in [(true, 27), (false, 29)] {
        let run = standard_run_at(
            Character::Ironclad,
            0,
            full,
            RunRng::new([seed; 32], Partition::Test, 0),
        )
        .unwrap();
        let d = decision(&run);
        let obs = encoded(&d, run.public_map().as_ref(), 0);
        assert_eq!(
            obs.action_mask.iter().filter(|&&v| v == 1).count(),
            d.candidates.len(),
            "full_neow={full}"
        );
        assert_eq!(d.room.as_ref().unwrap().kind, RoomViewKind::Neow);
        let offers = group_value(&obs, "offers");
        let choice_col = descriptor_field_index(
            &standard_run_observation_spec(&RunCapacities::standard_default())
                .unwrap()
                .descriptor(),
            "offers",
            "choice_kind",
        );
        let offer_cols = offers.shape[1];
        assert!(offers.occupied.as_ref().unwrap().contains(&1));
        for (row, &occupied) in offers.occupied.as_ref().unwrap().iter().enumerate() {
            if occupied == 1 {
                assert_ne!(offers.values[row * offer_cols + choice_col], 0);
                assert_eq!(offers.known[row * offer_cols + choice_col], 1);
            }
        }
    }
}

#[test]
fn standard_domains_cover_engine_bounds() {
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap();
    let descriptor = spec.descriptor();
    // Domain facts are from the engine domain table in M34a lane B bounds-v2.json.
    let mut failures = Vec::new();
    for (group, field, low, high) in [
        ("player", "hp", 0, 999),
        ("player", "max_hp", 1, 999),
        ("player", "gold", 0, u32::MAX as i64),
        ("player", "block", 0, 999),
        ("player", "energy", 0, 999),
        ("player", "turn", 0, i32::MAX as i64),
        ("enemies", "hp", 0, 999),
        ("enemies", "max_hp", 1, 999),
        ("enemies", "block", 0, 999),
        ("enemies", "intent_damage", 0, 30012),
        ("enemies", "intent_hits", 0, i32::MAX as i64),
        ("combat_state", "heart_cap", 0, 300),
        ("powers", "amount", i32::MIN as i64, i32::MAX as i64),
        ("relics", "visible_counter", i32::MIN as i64, i32::MAX as i64),
        ("combat_cards", "effective_cost", i32::MIN as i64, i32::MAX as i64),
        ("combat_cards", "base_damage", i32::MIN as i64, i32::MAX as i64),
        ("permanent_cards", "upgrades", 0, u32::MAX as i64),
        ("context", "ascension", 0, 20),
        ("context", "floor", 0, i32::MAX as i64),
        // Lane B lumps turn/floor/act at i32::MAX; `PublicDecision.act` is a u8 (run/decision.rs), so its type bound is 255.
        ("context", "act", 0, u8::MAX as i64),
        ("context", "reward_total", i32::MIN as i64, i32::MAX as i64),
        ("context", "ruby", 0, 1),
        ("map_nodes", "floor", 0, i32::MAX as i64),
        ("map_nodes", "x", 0, u8::MAX as i64),
        ("map_nodes", "y", 0, u8::MAX as i64),
        ("offers", "price", 0, u32::MAX as i64),
        ("offers", "event_stage", 0, u8::MAX as i64),
        ("offers", "param0", i32::MIN as i64, i32::MAX as i64),
        ("room_params", "stage0", i32::MIN as i64, i32::MAX as i64),
    ] {
        let (actual_low, actual_high) = descriptor_column(&descriptor, group, field);
        if actual_low > low { failures.push(format!("{group}.{field} lower bound {actual_low} > {low}")); }
        if actual_high < high { failures.push(format!("{group}.{field} upper bound {actual_high} < {high}")); }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
    assert_eq!(descriptor_column(&descriptor, "enemies", "intent_hits").1, i32::MAX as i64);
    assert_eq!(descriptor_column(&descriptor, "combat_state", "heart_cap").1, 300);
    assert_eq!(descriptor_column(&descriptor, "context", "ascension"), (0, 20));
}

#[test]
fn standard_encodes_every_candidate_kind() {
    let (run, map) = standard_start(Character::Ironclad, 0, 31);
    let mut d = decision(&run);
    let choices = vec![
        RoomChoiceKind::NeowMaxHp { amount: 3 }, RoomChoiceKind::NeowLament,
        RoomChoiceKind::Gold(25), RoomChoiceKind::CardReward,
        RoomChoiceKind::Card { id: CardId::Strike, upgrades: 1 },
        RoomChoiceKind::Potion(PotionId::FirePotion), RoomChoiceKind::Relic(RelicId::BurningBlood),
        RoomChoiceKind::Sapphire, RoomChoiceKind::Emerald, RoomChoiceKind::OpenChest,
        RoomChoiceKind::Rest { heal: 10 }, RoomChoiceKind::Smith, RoomChoiceKind::Recall,
        RoomChoiceKind::Lift, RoomChoiceKind::Toke, RoomChoiceKind::Dig,
        RoomChoiceKind::SingingBowl, RoomChoiceKind::Purge, RoomChoiceKind::ShrineUpgrade,
        RoomChoiceKind::ShrineRemove, RoomChoiceKind::AncientElegance,
        RoomChoiceKind::AncientSimplicity, RoomChoiceKind::JoustMurderer,
        RoomChoiceKind::JoustOwner, RoomChoiceKind::JoustContinue,
        RoomChoiceKind::JoustResult { owner_wins: true },
        RoomChoiceKind::FallingLose { card_type: CardType::Attack, instance: None },
        RoomChoiceKind::TombGold, RoomChoiceKind::TombOfferGold, RoomChoiceKind::Leave,
        RoomChoiceKind::Proceed, RoomChoiceKind::Skip, RoomChoiceKind::ConfirmSkip,
        RoomChoiceKind::CancelSkip, RoomChoiceKind::Act3Continue { stage: 2 },
        RoomChoiceKind::NeowBlessing { category: 1, reward: NeowReward::ThreeCards, drawback: Some(NeowDrawback::NoGold) },
        RoomChoiceKind::EventOption { stage: 1, option: 2 },
        RoomChoiceKind::EventCard { stage: 1, option: 2, card: Some(CardId::Strike) },
        RoomChoiceKind::EventPotion { stage: 1, option: 2, potion: Some(PotionId::FirePotion) },
        RoomChoiceKind::EventRelic { stage: 1, option: 2, relic: Some(RelicId::BurningBlood) },
    ];
    let rows: Vec<_> = choices.iter().cloned().enumerate().map(|(i, kind)| RoomChoiceView {
        id: ChoiceId(i as u64 + 1), kind, price: None, status: RoomChoiceStatus::Offered,
    }).collect();
    d.room = Some(RoomView { kind: RoomViewKind::Event(EventId::Falling), choices: rows });
    d.room_params = Some(RoomParams {
        stage: [Some(3), None, None, None],
        rows: choices.iter().enumerate().map(|(i, _)| {
            (ChoiceId(i as u64 + 1), [Some(11), Some(22), Some(33)])
        }).collect(),
    });
    d.candidates = choices.iter().enumerate().map(|(i, _)| Candidate {
        id: ChoiceId(i as u64 + 1), action: GameAction::Choose(ChoiceId(i as u64 + 1)),
        label: "synthetic public choice", neow_offer: None, object: None,
    }).collect();
    let observation = encoded(&d, Some(&map), 0);
    assert_eq!(observation.action_mask.iter().filter(|&&v| v == 1).count(), d.candidates.len());
    assert_eq!(group_value(&observation, "offers").occupied.as_ref().unwrap().iter().filter(|&&v| v == 1).count(), choices.len());
    let actions = group_value(&observation, "actions");
    let cols = actions.shape[1];
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let choice_col = descriptor_field_index(&spec, "actions", "choice_kind");
    for row in 0..choices.len() {
        assert_eq!(observation.action_mask[row], 1);
        assert_ne!(actions.values[row * cols + choice_col], 0, "choice row {row} had a padding kind");
    }

    // The same candidate rows preserve event stage, option, and displayed parameters.
    let offers = group_value(&observation, "offers");
    let stage_col = descriptor_field_index(&spec, "offers", "event_stage");
    let option_col = descriptor_field_index(&spec, "offers", "event_option");
    let param_col = descriptor_field_index(&spec, "offers", "param0");
    let offer_cols = offers.shape[1];
    let event_row = choices.iter().position(|c| matches!(c, RoomChoiceKind::EventOption { .. })).unwrap();
    assert_eq!(offers.values[event_row * offer_cols + stage_col], 1);
    assert_eq!(offers.values[event_row * offer_cols + option_col], 2);
    assert_eq!(offers.values[event_row * offer_cols + param_col], 11);
    assert_eq!(group_value(&observation, "room_params").values[0], 3);

    let mut treasure = d.clone();
    treasure.room = Some(RoomView { kind: RoomViewKind::Treasure, choices: vec![
        RoomChoiceView { id: ChoiceId(800), kind: RoomChoiceKind::OpenChest, price: None, status: RoomChoiceStatus::Offered },
    ]});
    treasure.candidates = vec![Candidate { id: ChoiceId(800), action: GameAction::Choose(ChoiceId(800)), label: "open treasure", neow_offer: None, object: None }];
    treasure.room_params = Some(RoomParams { stage: [Some(3), None, None, None], rows: vec![] });
    let treasure_obs = encoded(&treasure, Some(&map), 0);
    assert_eq!(group_value(&treasure_obs, "room_params").values[0], 3, "large chest size is visible");

    // A map decision's legal Move candidates retain their map row references, including Boots context.
    let mut map_decision = d.clone();
    map_decision.room = Some(RoomView { kind: RoomViewKind::Map, choices: vec![] });
    map_decision.room_params = None;
    map_decision.candidates = map.nodes.iter().take(2).enumerate().map(|(i, node)| Candidate {
        id: ChoiceId(100 + i as u64), action: GameAction::Move(node.id),
        label: "synthetic map move", neow_offer: None, object: None,
    }).collect();
    let mut boots_map = map.clone();
    boots_map.winged_moves = 1;
    let map_obs = encoded(&map_decision, Some(&boots_map), 0);
    assert_eq!(map_obs.action_mask.iter().filter(|&&v| v == 1).count(), map_decision.candidates.len());
    assert_eq!(group_value(&map_obs, "context").values[descriptor_field_index(&spec, "context", "winged_moves")], 1);

    // Neow selectors publish a named operation and legal selection candidates.
    let mut selection_decision = d.clone();
    selection_decision.room = Some(RoomView { kind: RoomViewKind::Neow, choices: vec![] });
    selection_decision.room_params = None;
    selection_decision.selection = Some(PublicSelection {
        required: 1, selected_count: 0, eligible: selection_decision.objects.clone(),
    });
    selection_decision.candidates = selection_decision.objects.iter().map(|card| Candidate {
        id: ChoiceId(card.instance), action: GameAction::Select(SelectionId(card.instance)),
        label: "synthetic Neow selector", neow_offer: None, object: Some(card.clone()),
    }).collect();
    let selection_obs = crate::run_observation::encode_run_standard(
        &selection_decision, Some(&map), &RunCapacities::standard_default(),
        crate::run_observation::StandardContext {
            ascension: 0,
            run_selection: Some(RoomSelectionOperation::NeowSelect { op: NeowSelectOp::Remove, count: 1 }),
        },
    ).unwrap();
    assert_eq!(selection_obs.action_mask.iter().filter(|&&v| v == 1).count(), selection_decision.candidates.len());
    let selection_operation = descriptor_field_index(&spec, "selection_counts", "operation");
    let operation_id = group_value(&selection_obs, "selection_counts").values[selection_operation];
    assert_ne!(operation_id, 0);
    assert_eq!(spec["vocabularies"]["selection_operation"][operation_id as usize], "neow_remove");

    // Combat DiscardPotion has its own Standard verb and legal candidate row.
    let mut combat_decision = decision(&run);
    combat_decision.room = Some(RoomView { kind: RoomViewKind::Combat, choices: vec![] });
    combat_decision.room_params = None;
    combat_decision.combat.as_mut().unwrap().inventory.as_mut().unwrap().potions[0] = Some(PotionId::FirePotion);
    combat_decision.candidates = vec![Candidate {
        id: ChoiceId(999), action: GameAction::Combat(CombatAction::DiscardPotion { slot: 0 }),
        label: "synthetic discard potion", neow_offer: None, object: None,
    }];
    let combat_obs = encoded(&combat_decision, Some(&map), 0);
    let verb_col = descriptor_field_index(&spec, "actions", "verb");
    let verb_value = group_value(&combat_obs, "actions").values[verb_col];
    let verbs = spec["vocabularies"]["verb"].as_array().unwrap();
    assert_eq!(verbs[verb_value as usize], "discard_potion");
    assert_eq!(combat_obs.action_mask.iter().filter(|&&v| v == 1).count(), 1);

    // Parent card-reward screens publish no child card identities; those appear on the opened child.
    let mut reward_parent = d.clone();
    reward_parent.combat = None;
    reward_parent.room = Some(RoomView { kind: RoomViewKind::Rewards, choices: vec![
        RoomChoiceView { id: ChoiceId(700), kind: RoomChoiceKind::CardReward, price: None, status: RoomChoiceStatus::Offered },
        RoomChoiceView { id: ChoiceId(701), kind: RoomChoiceKind::Skip, price: None, status: RoomChoiceStatus::Offered },
    ]});
    reward_parent.room_params = None;
    reward_parent.candidates = [ChoiceId(700), ChoiceId(701)].into_iter().map(|id| Candidate {
        id, action: GameAction::Choose(id), label: "reward parent", neow_offer: None, object: None,
    }).collect();
    let parent_obs = encoded(&reward_parent, None, 0);
    let card_col = descriptor_field_index(&spec, "offers", "card_kind");
    assert_eq!(group_value(&parent_obs, "offers").known[card_col], 0);
    let mut reward_child = reward_parent;
    reward_child.room = Some(RoomView { kind: RoomViewKind::CardReward, choices: vec![
        RoomChoiceView { id: ChoiceId(702), kind: RoomChoiceKind::Card { id: CardId::Strike, upgrades: 0 }, price: None, status: RoomChoiceStatus::Offered },
        RoomChoiceView { id: ChoiceId(703), kind: RoomChoiceKind::Skip, price: None, status: RoomChoiceStatus::Offered },
    ]});
    reward_child.candidates = [ChoiceId(702), ChoiceId(703)].into_iter().map(|id| Candidate {
        id, action: GameAction::Choose(id), label: "reward child", neow_offer: None, object: None,
    }).collect();
    let child_obs = encoded(&reward_child, None, 0);
    assert_eq!(group_value(&child_obs, "offers").known[card_col], 1);

    // Screen kinds cover the Standard terminal and Act4/Heart surfaces. The second boss is absent until its room.
    let room_col = descriptor_field_index(&spec, "context", "room_kind");
    for kind in [
        RoomViewKind::Neow, RoomViewKind::Map, RoomViewKind::Combat, RoomViewKind::Rewards,
        RoomViewKind::CardReward, RoomViewKind::Shop, RoomViewKind::Campfire,
        RoomViewKind::Event(EventId::Falling), RoomViewKind::Treasure,
        RoomViewKind::CheckpointComplete, RoomViewKind::BossTreasure,
        RoomViewKind::BossRelicChoice, RoomViewKind::BossRelicChild,
        RoomViewKind::ProceedWithoutRelic, RoomViewKind::Act2Ready,
        RoomViewKind::Act2PrefixComplete, RoomViewKind::Act3Ready,
        RoomViewKind::Act3PrefixComplete, RoomViewKind::Act3BossComplete,
        RoomViewKind::Act3Ending, RoomViewKind::Act4Ready, RoomViewKind::HeartVictory,
        RoomViewKind::TrueVictory,
    ] {
        let mut screen = d.clone();
        screen.combat = None;
        screen.room = Some(RoomView { kind, choices: vec![] });
        screen.candidates.clear();
        screen.terminal = (kind == RoomViewKind::HeartVictory || kind == RoomViewKind::TrueVictory).then_some(RunTerminal::HeartVictory);
        let screen_obs = encoded(&screen, None, 20);
        assert_ne!(group_value(&screen_obs, "context").values[room_col], 0);
        assert!(screen_obs.action_mask.iter().all(|&v| v == 0));
    }

    // Character mechanics share the same encoder contract at real Standard combat boundaries.
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let (character_run, character_map) = standard_start(character, 0, 61 + character as u8);
        let character_decision = decision(&character_run);
        let character_obs = encoded(&character_decision, Some(&character_map), 0);
        assert_eq!(character_obs.action_mask.iter().filter(|&&v| v == 1).count(), character_decision.candidates.len());
    }
}

#[test]
fn standard_drops_retired_actors_and_keeps_rows_consistent() {
    let (run, map) = standard_start(Character::Ironclad, 0, 37);
    let mut d = decision(&run);
    let combat = d.combat.as_mut().unwrap();
    assert!(!combat.enemies.is_empty());
    // Keep one half-dead actor, then add escaped and replaced actors to the same real Standard view.
    let mut live = combat.enemies[0].clone();
    live.hp = 0;
    live.half_dead = true;
    live.replaced = false;
    live.escaped = false;
    live.powers.push(FullPowerView { id: FullPowerId("strength"), amount: 2 });
    let mut escaped = combat.enemies[0].clone();
    escaped.escaped = true;
    let mut replaced = combat.enemies[0].clone();
    replaced.replaced = true;
    combat.enemies = vec![live, escaped, replaced];
    let target = combat.targets.first().map(|v| v.0).unwrap_or(TargetId(1));
    combat.targets = vec![(target, true), (TargetId(target.0 + 1), true), (TargetId(target.0 + 2), true)];
    let card = combat.hand.first().expect("real combat has a hand card").instance;
    d.candidates = vec![Candidate {
        id: ChoiceId(600),
        action: GameAction::Combat(CombatAction::PlayCard { instance: card.0, target: Some(target) }),
        label: "synthetic targeted play", neow_offer: None, object: None,
    }];
    let obs = encoded(&d, Some(&map), 0);
    let enemies = group_value(&obs, "enemies");
    assert_eq!(enemies.occupied.as_ref().unwrap().iter().filter(|&&v| v == 1).count(), 1);
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let half_dead = descriptor_field_index(&spec, "enemies", "half_dead");
    let power = group_value(&obs, "powers");
    let power_cols = power.shape[1];
    let owner_row = descriptor_field_index(&spec, "powers", "owner_row");
    let owner_kind = descriptor_field_index(&spec, "powers", "owner_kind");
    let enemy_owner = spec["vocabularies"]["row_group"]
        .as_array().unwrap().iter().position(|v| v == "enemies").unwrap() as i64;
    assert!(power.occupied.as_ref().unwrap().iter().enumerate().any(|(i, &occupied)| {
        occupied == 1 && power.values[i * power_cols + owner_kind] == enemy_owner && power.values[i * power_cols + owner_row] == 0
    }));
    assert_eq!(enemies.known[half_dead], 1);
    assert_eq!(enemies.values[half_dead], 1);
    let actions = group_value(&obs, "actions");
    let action_cols = actions.shape[1];
    let target_col = descriptor_field_index(&spec, "actions", "target_row");
    let mut targeted_actions = 0;
    for row in 0..actions.shape[0] {
        if obs.action_mask[row] == 1 && actions.values[row * action_cols + target_col] >= 0 {
            assert_eq!(actions.values[row * action_cols + target_col], 0);
            targeted_actions += 1;
        }
    }
    assert_eq!(targeted_actions, 1);
}

#[test]
fn standard_heart_cap_needs_a_present_heart() {
    let (run, map) = standard_start(Character::Ironclad, 0, 41);
    let d = decision(&run);
    let obs = encoded(&d, Some(&map), 0);
    let heart_cap = group_value(&obs, "combat_state");
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let cap_col = descriptor_field_index(&spec, "combat_state", "heart_cap");
    assert_eq!(heart_cap.known[cap_col], 0, "a run without a Heart has no Heart cap");

    let (heart_run, heart_map) = standard_start(Character::Ironclad, 19, 43);
    let mut heart_d = decision(&heart_run);
    let heart = heart_d.combat.as_mut().unwrap();
    heart.enemies[0].enemy_id = EnemyId::CorruptHeart;
    heart.enemies[0].invincible_cap = Some(200);
    let heart_obs = encoded(&heart_d, Some(&heart_map), 19);
    let cap = group_value(&heart_obs, "combat_state");
    assert_eq!(cap.known[cap_col], 1);
    assert_eq!(cap.values[cap_col], 200);

    let (a0_run, a0_map) = standard_start(Character::Ironclad, 0, 44);
    let mut a0_heart_d = decision(&a0_run);
    let a0_heart = a0_heart_d.combat.as_mut().unwrap();
    a0_heart.enemies[0].enemy_id = EnemyId::CorruptHeart;
    a0_heart.enemies[0].invincible_cap = Some(300);
    let a0_heart_obs = encoded(&a0_heart_d, Some(&a0_map), 0);
    let a0_cap = group_value(&a0_heart_obs, "combat_state");
    assert_eq!(a0_cap.known[cap_col], 1);
    assert_eq!(a0_cap.values[cap_col], 300);
}

#[test]
fn standard_intent_columns_are_live_engine_values() {
    let (run, map) = standard_start(Character::Ironclad, 0, 45);
    let d = decision(&run);
    let obs = encoded(&d, Some(&map), 0);
    let full = &d.combat.as_ref().unwrap().enemies;
    let spec = standard_run_observation_spec(&RunCapacities::standard_default())
        .unwrap()
        .descriptor();
    let vocabulary = spec["vocabularies"]["intent_category"]
        .as_array()
        .unwrap();
    let enemies = group_value(&obs, "enemies");
    let hits_col = descriptor_field_index(&spec, "enemies", "intent_hits");
    let category_col = descriptor_field_index(&spec, "enemies", "intent_category");
    let cols = enemies.shape[1];
    for (row, actor) in full.iter().enumerate() {
        if actor.half_dead || (!actor.replaced && !actor.escaped && actor.hp > 0) {
            assert_eq!(enemies.known[row * cols + hits_col], i8::from(actor.intent_hits.is_some()));
            assert_eq!(enemies.values[row * cols + hits_col], actor.intent_hits.map(i64::from).unwrap_or(0));
            assert_eq!(enemies.known[row * cols + category_col], i8::from(actor.intent_category.is_some()));
            let expected_category = actor.intent_category.map(|category| {
                vocabulary.iter().position(|v| v == category.as_str()).unwrap() as i64
            }).unwrap_or(0);
            assert_eq!(enemies.values[row * cols + category_col], expected_category);
        }
    }
    // Synthetic high-growth actor view stands in for a late Book of Stabbing stab_count fixture.
    let mut growth = d.clone();
    growth.combat.as_mut().unwrap().enemies[0].intent_hits = Some(i32::MAX as u32);
    let growth_obs = encoded(&growth, Some(&map), 0);
    assert_eq!(group_value(&growth_obs, "enemies").known[hits_col], 1);
    assert_eq!(group_value(&growth_obs, "enemies").values[hits_col], i32::MAX as i64);
}

#[test]
fn runic_dome_hides_category_and_hits() {
    let (run, map) = standard_start(Character::Ironclad, 0, 47);
    let mut d = decision(&run);
    let combat = d.combat.as_mut().unwrap();
    combat.enemies[0].intent_category = None;
    combat.enemies[0].intent_hits = None;
    let obs = encoded(&d, Some(&map), 0);
    let enemies = group_value(&obs, "enemies");
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    for field in ["intent_category", "intent_hits"] {
        let col = descriptor_field_index(&spec, "enemies", field);
        assert_eq!(enemies.known[col], 0, "hidden enemy {field} must remain unknown");
        assert_eq!(enemies.values[col], 0);
    }
}

#[test]
fn second_a20_boss_not_named_before_its_room() {
    let (run, _) = standard_start(Character::Ironclad, 20, 49);
    let mut d = decision(&run);
    d.act = 3;
    d.room = Some(RoomView { kind: RoomViewKind::Act3Ending, choices: vec![] });
    d.combat = None;
    d.candidates.clear();
    let obs = encoded(&d, None, 20);
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let map_boss = descriptor_field_index(&spec, "context", "map_boss");
    let context = group_value(&obs, "context");
    assert_eq!(context.known[map_boss], 0);
    assert_eq!(context.values[map_boss], 0);
}

#[test]
fn card_reward_children_hidden_until_opened() {
    let (run, _) = standard_start(Character::Ironclad, 0, 51);
    let mut d = decision(&run);
    d.combat = None;
    d.room_params = None;
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let card_col = descriptor_field_index(&spec, "offers", "card_kind");
    d.room = Some(RoomView { kind: RoomViewKind::Rewards, choices: vec![
        RoomChoiceView { id: ChoiceId(1), kind: RoomChoiceKind::CardReward, price: None, status: RoomChoiceStatus::Offered },
    ]});
    d.candidates = vec![Candidate { id: ChoiceId(1), action: GameAction::Choose(ChoiceId(1)), label: "open reward", neow_offer: None, object: None }];
    let parent = encoded(&d, None, 0);
    assert_eq!(group_value(&parent, "offers").known[card_col], 0);

    d.room = Some(RoomView { kind: RoomViewKind::CardReward, choices: vec![
        RoomChoiceView { id: ChoiceId(2), kind: RoomChoiceKind::Card { id: CardId::Strike, upgrades: 0 }, price: None, status: RoomChoiceStatus::Offered },
    ]});
    d.candidates = vec![Candidate { id: ChoiceId(2), action: GameAction::Choose(ChoiceId(2)), label: "choose reward", neow_offer: None, object: None }];
    let child = encoded(&d, None, 0);
    assert_eq!(group_value(&child, "offers").known[card_col], 1);
}

#[test]
fn undisplayed_event_captures_stay_private() {
    let (run, _) = standard_start(Character::Ironclad, 0, 55);
    let mut d = decision(&run);
    d.combat = None;
    d.room = Some(RoomView { kind: RoomViewKind::Event(EventId::WorldOfGoop), choices: vec![
        RoomChoiceView { id: ChoiceId(5), kind: RoomChoiceKind::EventOption { stage: 0, option: 1 }, price: None, status: RoomChoiceStatus::Offered },
    ]});
    d.room_params = None;
    d.candidates = vec![Candidate { id: ChoiceId(5), action: GameAction::Choose(ChoiceId(5)), label: "event option", neow_offer: None, object: None }];
    let obs = encoded(&d, None, 0);
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let offers = group_value(&obs, "offers");
    let params = ["param0", "param1", "param2"];
    for field in params {
        let col = descriptor_field_index(&spec, "offers", field);
        assert_eq!(offers.known[col], 0, "undisplayed event value {field} leaked");
        assert_eq!(offers.values[col], 0);
    }
    let room_params = group_value(&obs, "room_params");
    assert!(room_params.known.iter().all(|&known| known == 0));
}

#[test]
fn standard_capacity_overflow_is_a_fault_not_a_loss() {
    let mut config = StandardRunConfig::new(Character::Ironclad, 0, Partition::Test);
    config.capacities.actions = 1;
    config.record_replay = true;
    let mut session = RunSession::new_standard(config).unwrap();
    let error = session.reset(Some(53)).unwrap_err();
    assert!(matches!(error, RunSessionError::Capacity { group: "actions", .. }));
    assert!(matches!(session.decision(), Err(RunSessionError::Capacity { group: "actions", .. })));
    let record = session.record().unwrap().unwrap();
    assert_eq!(record.terminal, None);
    let fault = record.fault.unwrap();
    assert!(!fault.committed);
    assert!(!fault.gameplay_recorded);
    assert_eq!(fault.error.kind, RunRecordErrorKind::Capacity);
    assert_eq!(fault.error.capacity.unwrap().group, "actions");
}

/// M34b review I1: Headbutt / Dagger Throw / Cut Through Fate can kill a Bronze Orb holding a Stasis card and open a
/// selection before the engine releases the card (core `combat::tests::m34b_stasis`). Standard drops the dead orb's row,
/// so the card stays in the Stasis zone with its owner unknown; the boundary encodes and stays actionable.
#[test]
fn standard_stasis_card_of_a_retired_holder_has_an_unknown_owner() {
    let (run, map) = standard_start(Character::Ironclad, 0, 37);
    let mut d = decision(&run);
    let combat = d.combat.as_mut().unwrap();
    let mut live = combat.enemies[0].clone();
    live.instance_id = spire_core::combat::EnemyHandle::parse("m34b-orb/spawn/1").unwrap();
    let mut dead = combat.enemies[0].clone();
    dead.hp = 0; dead.half_dead = false; dead.replaced = false; dead.escaped = false;
    dead.instance_id = spire_core::combat::EnemyHandle::parse("m34b-orb/spawn/0").unwrap();
    let held_by_dead = combat.hand.pop().expect("hand card");
    let held_by_live = combat.hand.pop().expect("second hand card");
    let (dead_orb, live_orb) = (dead.instance_id.clone(), live.instance_id.clone());
    combat.enemies.push(dead);
    combat.targets.push((TargetId(90), false));
    combat.enemies.push(live);
    combat.targets.push((TargetId(91), true));
    combat.stasis.push(spire_core::combat::StasisCardView { orb: dead_orb, card: held_by_dead.clone() });
    combat.stasis.push(spire_core::combat::StasisCardView { orb: live_orb, card: held_by_live.clone() });
    let enemy_rows = combat.enemies.len() - 1; // the dead orb is not published
    d.candidates.retain(|c| !matches!(c.action, GameAction::Combat(CombatAction::PlayCard { instance, .. }) if instance == held_by_dead.instance.0 || instance == held_by_live.instance.0));
    let obs = encoded(&d, Some(&map), 0);
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let enemies = group_value(&obs, "enemies");
    assert_eq!(enemies.occupied.as_ref().unwrap().iter().filter(|o| **o == 1).count(), enemy_rows);
    let cards = group_value(&obs, "combat_cards");
    let cols = cards.shape[1];
    let (zone, stasis_col) = (descriptor_field_index(&spec, "combat_cards", "zone"), descriptor_field_index(&spec, "combat_cards", "stasis_enemy_row"));
    let stasis_rows = (0..cards.shape[0]).filter(|r| cards.occupied.as_ref().unwrap()[*r] == 1 && cards.values[r * cols + zone] == 5).collect::<Vec<_>>();
    assert_eq!(stasis_rows.len(), 2, "both held cards stay in the Stasis zone");
    let owners = stasis_rows.iter().map(|r| (cards.known[r * cols + stasis_col], cards.values[r * cols + stasis_col])).collect::<Vec<_>>();
    assert_eq!(owners[0], (0, -1), "a retired holder is an unknown owner");
    assert_eq!(owners[1], (1, enemy_rows as i64 - 1), "a present holder is its filtered row");
    assert_eq!(obs.action_mask.iter().filter(|m| **m == 1).count(), d.candidates.len());
}
