//! M34b Standard traversals through the public Run observation and legal action mask.
use super::super::*;
use crate::{
    RunActionIndex, RunCapacities, RunSession, StandardCreation, StandardNeow,
    StandardRunConfig,
};
use spire_core::{
    combat::{rng::Partition, CombatAction},
    content::{CardId, CardType, Character},
    run::{diagnostics::{standard_supplied_run_with, supplied_wound_enemies}, GameAction, Key, RewardItem, RewardItemKind, Run, RunError, RunRng, RunTerminal},
};

pub(super) const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn vocab_name(family: &str, index: i64) -> Option<String> {
    let index = usize::try_from(index).ok()?;
    Some(crate::standard_vocab_manifest()["families"][family].as_array()?.get(index)?.as_str()?.to_owned())
}

fn field(obs: &RunObservation, group: &str, row: usize, column: usize) -> i64 {
    let group = obs.group(group).unwrap_or_else(|| panic!("missing {group} group"));
    let columns = *group.shape.last().unwrap();
    group.values[row * columns + column]
}

fn legal_indices(obs: &RunObservation) -> Vec<usize> {
    let legal = obs.action_mask.iter().enumerate().filter_map(|(index, value)| (*value == 1).then_some(index)).collect::<Vec<_>>();
    let actions = obs.group("actions").unwrap();
    let occupied = actions.occupied.as_ref().unwrap();
    assert_eq!(legal.len(), occupied.iter().filter(|value| **value == 1).count(), "mask and action rows differ");
    for &index in &legal { assert_eq!(occupied[index], 1); }
    legal
}

/// The requested deliberately simple policy. Every index comes from the published legal mask.
pub(super) fn simple_action(obs: &RunObservation) -> usize {
    let legal = legal_indices(obs);
    assert!(!legal.is_empty(), "nonterminal Standard boundary had no legal candidate");
    let room = vocab_name("room", field(obs, "context", 0, 4)).unwrap_or_else(|| "unknown".to_owned());
    if room == "combat" {
        let mut lowest_targeted_attack: Option<(i64, usize)> = None;
        let mut any_attack = None;
        let mut any_card = None;
        let mut end_turn = None;
        let mut select_card = None;
        let mut confirm_selection = None;
        for &index in &legal {
            let verb = vocab_name("verb", field(obs, "actions", index, 0)).unwrap_or_default();
            match verb.as_str() {
                "play_card" => {
                    any_card.get_or_insert(index);
                    let source_group = vocab_name("row_group", field(obs, "actions", index, 1)).unwrap_or_default();
                    if source_group != "combat_cards" { continue; }
                    let source_row = usize::try_from(field(obs, "actions", index, 2)).unwrap_or(usize::MAX);
                    let card_name = vocab_name("card", field(obs, "combat_cards", source_row, 2)).unwrap_or_default();
                    let card_type = CardId::from_name(&card_name)
                        .and_then(|id| spire_core::content::Catalog::standard().card(id))
                        .map(|card| card.card_type);
                    if card_type != Some(CardType::Attack) { continue; }
                    any_attack.get_or_insert(index);
                    if vocab_name("row_group", field(obs, "actions", index, 3)).as_deref() == Some("enemies") {
                        let target_row = usize::try_from(field(obs, "actions", index, 4)).unwrap_or(usize::MAX);
                        let enemy_rows = obs.group("enemies").unwrap().occupied.as_ref().unwrap();
                        if enemy_rows.get(target_row) == Some(&1) {
                            let hp = field(obs, "enemies", target_row, 1);
                            if lowest_targeted_attack.map_or(true, |(lowest_hp, _)| hp < lowest_hp) {
                                lowest_targeted_attack = Some((hp, index));
                            }
                        }
                    }
                }
                "end_turn" => end_turn = Some(index),
                "combat_select_card" => { select_card.get_or_insert(index); }
                "confirm_selection" => confirm_selection = Some(index),
                _ => {}
            }
        }
        if let Some((_, index)) = lowest_targeted_attack { return index; }
        if let Some(index) = any_attack.or(any_card).or(end_turn) { return index; }
        // Resolve mandatory selection prompts without toggling selected cards indefinitely.
        if let Some(index) = select_card {
            let required = field(obs, "selection_counts", 0, 0);
            let selected = field(obs, "selection_counts", 0, 1);
            if selected < required { return index; }
        }
        return confirm_selection.unwrap_or(legal[0]);
    }

    if room == "event" {
        return legal[0];
    }
    if room == "rewards" {
        for &index in &legal {
            let choice = vocab_name("choice", field(obs, "actions", index, 5)).unwrap_or_default();
            if matches!(choice.as_str(), "gold" | "tomb_gold" | "tomb_offer_gold") { return index; }
        }
    }
    if room == "card_reward" {
        for &index in &legal {
            let verb = vocab_name("verb", field(obs, "actions", index, 0)).unwrap_or_default();
            let choice = vocab_name("choice", field(obs, "actions", index, 5)).unwrap_or_default();
            if verb == "skip_card_offer" || matches!(choice.as_str(), "skip" | "leave") { return index; }
        }
    }
    if room == "campfire" {
        for &index in &legal {
            if vocab_name("choice", field(obs, "actions", index, 5)).as_deref() == Some("rest") { return index; }
        }
    }
    if room == "shop" {
        for &index in &legal {
            if vocab_name("choice", field(obs, "actions", index, 5)).as_deref() == Some("leave") { return index; }
        }
    }
    // Events use the first offered legal row at each stage, then the event's Leave row.
    legal[0]
}

pub(super) fn context(obs: &RunObservation) -> (i64, i64, String) {
    (field(obs, "context", 0, 2), field(obs, "context", 0, 3), vocab_name("room", field(obs, "context", 0, 4)).unwrap())
}

pub(super) fn current_node_is_boss(obs: &RunObservation) -> bool {
    usize::try_from(field(obs, "current_node", 0, 0)).ok()
        .and_then(|row| vocab_name("map_kind", field(obs, "map_nodes", row, 3)))
        .as_deref() == Some("boss")
}

fn checkpoint(obs: &RunObservation) -> String {
    let (act, floor, room) = context(obs);
    let encounter_index = field(obs, "context", 0, 15);
    let encounter = if encounter_index == 0 { "none".to_owned() }
        else { vocab_name("encounter", encounter_index).unwrap_or_else(|| "unknown".to_owned()) };
    let turn = field(obs, "player", 0, 6);
    format!("act={act}, floor={floor}, room={room}, encounter={encounter}, turn={turn}")
}

fn action_checkpoint(obs: &RunObservation, index: usize) -> String {
    let verb = vocab_name("verb", field(obs, "actions", index, 0)).unwrap_or_else(|| "unknown".to_owned());
    let choice = vocab_name("choice", field(obs, "actions", index, 5)).unwrap_or_else(|| "padding".to_owned());
    let source_group = vocab_name("row_group", field(obs, "actions", index, 1)).unwrap_or_else(|| "none".to_owned());
    let source_row = field(obs, "actions", index, 2);
    let target_group = vocab_name("row_group", field(obs, "actions", index, 3)).unwrap_or_else(|| "none".to_owned());
    let target_row = field(obs, "actions", index, 4);
    format!("{verb} choice={choice} source={source_group}:{source_row} target={target_group}:{target_row}")
}

fn terminal_diagnosis(session: &RunSession, terminal: Option<RunTerminal>, turn: Option<i64>) -> String {
    let reproduction = session.reproduction().unwrap();
    let Some(Ok(decision)) = reproduction.latest_decision.as_ref() else { return "decision unavailable".to_owned(); };
    let encounter = decision.last_combat.as_ref().map(|combat| combat.encounter);
    let action = reproduction.last_accepted_step.as_ref().map(|step| &step.action);
    let cause = match terminal {
        Some(RunTerminal::EmptyDeck) => "empty deck".to_owned(),
        Some(RunTerminal::Death) if encounter == Some(spire_core::content::EncounterId::CorruptHeart)
            && matches!(action, Some(GameAction::Combat(CombatAction::PlayCard { .. }))) =>
                "Heart Beat of Death after card play".to_owned(),
        Some(RunTerminal::Death) if decision.player.hp == 0 =>
            format!("HP reached 0; last action={action:?}, last combat={encounter:?}"),
        Some(RunTerminal::Death) => format!("Death; last action={action:?}, last combat={encounter:?}"),
        other => format!("{other:?}"),
    };
    format!("act={}, floor={}, room={:?}, encounter={encounter:?}, turn={}, hp={}, deck_cards={}, cause={cause}",
        decision.act, decision.floor, decision.room.as_ref().map(|room| room.kind), turn.unwrap_or(0), decision.player.hp, decision.objects.len())
}

fn needs_supplied_wound(obs: &RunObservation) -> bool {
    if context(obs).2 != "combat" { return false; }
    let end_turn_is_legal = legal_indices(obs).into_iter().any(|index|
        vocab_name("verb", field(obs, "actions", index, 0)).as_deref() == Some("end_turn"));
    let enemies = obs.group("enemies").unwrap();
    let present_enemy_over_one_hp = enemies.occupied.as_ref().unwrap().iter().enumerate()
        .any(|(row, occupied)| *occupied == 1 && field(obs, "enemies", row, 1) > 1);
    end_turn_is_legal && present_enemy_over_one_hp
}

pub(super) fn short_creation() -> StandardCreation {
    StandardCreation { neow: StandardNeow::Short, ..StandardCreation::experienced() }
}

fn supplied(character: Character, ascension: u8, seed: u8, creation: StandardCreation) -> (Run, Run) {
    let run = standard_supplied_run_with(character, ascension, creation.profile(), 900, true,
        RunRng::new([seed; 32], Partition::Test, 0)).unwrap();
    // supplied frontier: HP, keys and the M28/M33 constructed hit (enemies to 1 HP); not a policy claim
    let original = run.clone();
    (run, original)
}

pub(super) fn standard_session(character: Character, ascension: u8, seed: u8, creation: StandardCreation) -> (RunSession, Run) {
    let (run, replay_start) = supplied(character, ascension, seed, creation);
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.creation = creation;
    config.record_replay = true;
    (RunSession::from_standard_supplied(config, run, u64::from(seed)).unwrap(), replay_start)
}

#[derive(Clone, Debug)]
pub(super) struct DriveProgress { steps: usize, boss_fights: usize, last_combat_turn: Option<i64>, wound_counts: Vec<usize>, reward_steps: Vec<(i32, Vec<RewardItem>, [i64; 6])> }
#[derive(Debug)]
pub(super) struct DriveFailure { progress: DriveProgress, action_row: Option<usize>, checkpoint: String, action: String, candidates: String, error: String }

fn candidate_list(session: &RunSession) -> String {
    session.reproduction().ok().and_then(|reproduction| reproduction.latest_decision)
        .and_then(Result::ok).map(|decision| decision.candidates.iter().enumerate()
            .map(|(row, candidate)| format!("{row}:{:?}", candidate.action)).collect::<Vec<_>>().join(" | "))
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn replay_wounded_supplied(record: &crate::RunEpisodeRecord, replay_start: Run, wound_counts: &[usize]) -> crate::run_replay::RunReplayReport {
    let mut applied = Vec::new();
    let mut hook = |accepted: usize, run: &mut Run| -> Result<bool, RunError> {
        if wound_counts.contains(&accepted) {
            supplied_wound_enemies(run)?;
            applied.push(accepted);
            Ok(true)
        } else { Ok(false) }
    };
    let replay = crate::run_replay::replay_standard_supplied_with(
        record, RunCapacities::standard_default(), replay_start, &mut hook).unwrap();
    assert_eq!(applied, wound_counts, "replay applied the supplied hit schedule");
    replay
}

/// Every published value lies inside its descriptor column domain (Gymnasium `Box.contains`), padding included.
fn assert_in_domains(obs: &RunObservation) {
    use std::sync::OnceLock;
    static SPEC: OnceLock<serde_json::Value> = OnceLock::new();
    let spec = SPEC.get_or_init(|| crate::standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor());
    for group in &obs.groups {
        let tensor = spec["tensors"].as_array().unwrap().iter().find(|t| t["key"] == group.key).unwrap();
        let cols = *group.shape.last().unwrap();
        for (i, value) in group.values.iter().enumerate() {
            let (lo, hi) = (tensor["low"][i % cols].as_i64().unwrap(), tensor["high"][i % cols].as_i64().unwrap());
            assert!((lo..=hi).contains(value), "{}[{}] {} = {value} outside {lo}..={hi} at {}", group.key, i / cols, tensor["fields"][i % cols], checkpoint(obs));
        }
    }
}

pub(super) fn drive(session: &mut RunSession, limit: usize, mut stop: impl FnMut(&RunObservation) -> bool, mut policy: impl FnMut(&RunObservation) -> usize) -> Result<DriveProgress, DriveFailure> {
    let mut boundary = session.decision().unwrap();
    let mut steps = 0;
    let mut act3_boss_fights = 0;
    let mut was_act3_boss = false;
    let mut last_combat_turn = None;
    let mut wound_counts = Vec::new();
    let mut reward_steps = Vec::new();
    while boundary.terminal.is_none() && !stop(&boundary.observation) {
        assert_in_domains(&boundary.observation);
        let (act, _, room) = context(&boundary.observation);
        let is_act3_boss = act == 3 && room == "combat" && current_node_is_boss(&boundary.observation);
        if is_act3_boss && !was_act3_boss { act3_boss_fights += 1; }
        was_act3_boss = is_act3_boss;
        if needs_supplied_wound(&boundary.observation) {
            let before = checkpoint(&boundary.observation);
            let candidates = candidate_list(session);
            match session.apply_supplied(supplied_wound_enemies) {
                Ok(next) => { wound_counts.push(steps); boundary = next; }
                Err(error) => return Err(DriveFailure {
                    progress: DriveProgress { steps, boss_fights: act3_boss_fights, last_combat_turn, wound_counts, reward_steps },
                    action_row: None, checkpoint: before, action: "apply_supplied(supplied_wound_enemies)".to_owned(), candidates,
                    error: format!("{error:?}"),
                }),
            }
        }
        let (_, _, room) = context(&boundary.observation);
        if room == "combat" { last_combat_turn = Some(field(&boundary.observation, "player", 0, 6)); }
        let index = policy(&boundary.observation);
        let before = checkpoint(&boundary.observation);
        let action = action_checkpoint(&boundary.observation, index);
        let candidates = candidate_list(session);
        let result = match session.step(boundary.token, RunActionIndex(index as i64)) {
            Ok(result) => result,
            Err(error) => return Err(DriveFailure { progress: DriveProgress { steps, boss_fights: act3_boss_fights, last_combat_turn, wound_counts, reward_steps }, action_row: Some(index), checkpoint: before, action, candidates, error: format!("{error:?}") }),
        };
        boundary = result.boundary;
        reward_steps.push((result.reward, result.reward_events, [
            field(&boundary.observation, "context", 0, 2), field(&boundary.observation, "context", 0, 3),
            field(&boundary.observation, "context", 0, 7), field(&boundary.observation, "context", 0, 8),
            field(&boundary.observation, "context", 0, 9), field(&boundary.observation, "context", 0, 10),
        ]));
        steps += 1;
        if steps >= limit && boundary.terminal.is_none() && !stop(&boundary.observation) {
            return Err(DriveFailure {
                progress: DriveProgress { steps, boss_fights: act3_boss_fights, last_combat_turn, wound_counts, reward_steps },
                action_row: Some(index),
                checkpoint: checkpoint(&boundary.observation),
                action: format!("last accepted {action}"),
                candidates: candidate_list(session),
                error: format!("step limit {limit} reached"),
            });
        }
    }
    Ok(DriveProgress { steps, boss_fights: act3_boss_fights, last_combat_turn, wound_counts, reward_steps })
}

fn assert_heart_rewards(progress: &DriveProgress, ascension: u8) -> (i32, usize, usize) {
    let mut highest = 0_i64;
    let mut expected_floors = Vec::new();
    let mut all = Vec::new();
    let mut step_rewards = 0_i32;
    for (reward, items, [act, floor, total, _ruby, _sapphire, _emerald]) in &progress.reward_steps {
        assert_eq!(items.iter().map(|item| item.amount).sum::<i32>(), *reward,
            "step reward equals its itemised credits at act={act} floor={floor}");
        step_rewards += reward;
        if (1..=4).contains(act) && *floor > highest {
            highest = *floor;
            expected_floors.push(if *act == 4 {
                RewardItemKind::NewAct4Floor { floor: *floor as u32 }
            } else {
                RewardItemKind::NewFloor { act: *act as u8, floor: *floor as u32 }
            });
        }
        all.extend(items.iter().copied());
        assert_eq!(*total, i64::from(step_rewards), "context reward_total tracks itemised credits");
    }
    let actual_floors = all.iter().filter_map(|item| match item.kind {
        kind @ RewardItemKind::NewFloor { .. } | kind @ RewardItemKind::NewAct4Floor { .. } => Some(kind),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(actual_floors, expected_floors, "floor credits follow new highest floors in Acts 1–4");
    assert!(!all.iter().any(|item| matches!(item.kind, RewardItemKind::FirstKey(_))), "the supplied frontier starts with all keys");
    let hearts = all.iter().enumerate().filter(|(_, item)| item.kind == RewardItemKind::HeartVictory).collect::<Vec<_>>();
    assert_eq!(hearts.len(), 1, "one HeartVictory reward");
    assert_eq!(hearts[0].0, all.len() - 1, "HeartVictory is the last credited item");
    assert_eq!(hearts[0].1.amount, 100);
    if ascension == 20 {
        assert!(all.iter().any(|item| item.kind == RewardItemKind::NewFloor { act: 3, floor: 51 }),
            "A20 second boss floor 51 is credited");
    }
    assert_eq!(all.iter().map(|item| item.amount).sum::<i32>(), step_rewards);
    let normal_floors = actual_floors.iter().filter(|kind| matches!(kind, RewardItemKind::NewFloor { .. })).count();
    let act4_floors = actual_floors.iter().filter(|kind| matches!(kind, RewardItemKind::NewAct4Floor { .. })).count();
    (step_rewards, normal_floors, act4_floors)
}

fn shortest_burning_elite_move(obs: &RunObservation, legal: &[usize]) -> Option<usize> {
    use std::collections::{HashMap, VecDeque};
    let start = usize::try_from(field(obs, "current_node", 0, 0)).ok()?;
    let nodes = obs.group("map_nodes")?;
    let occupied = nodes.occupied.as_ref()?;
    let targets = occupied.iter().enumerate().filter_map(|(row, used)| {
        (*used == 1 && field(obs, "map_nodes", row, 5) == 1
            && vocab_name("map_kind", field(obs, "map_nodes", row, 3)).as_deref() == Some("elite"))
            .then_some(row)
    }).collect::<Vec<_>>();
    if targets.is_empty() { return None; }
    let edges = obs.group("map_edges")?;
    let edge_occupied = edges.occupied.as_ref()?;
    let mut next = vec![Vec::new(); occupied.len()];
    for (row, used) in edge_occupied.iter().enumerate() {
        if *used == 1 {
            let from = usize::try_from(field(obs, "map_edges", row, 0)).ok()?;
            let to = usize::try_from(field(obs, "map_edges", row, 1)).ok()?;
            if from < next.len() && to < next.len() { next[from].push(to); }
        }
    }
    let mut queue = VecDeque::from([start]);
    let mut previous = HashMap::from([(start, start)]);
    while let Some(from) = queue.pop_front() {
        for &to in &next[from] {
            if let std::collections::hash_map::Entry::Vacant(entry) = previous.entry(to) {
                entry.insert(from);
                queue.push_back(to);
            }
        }
    }
    let target = targets.into_iter().filter(|row| previous.contains_key(row))
        .min_by_key(|row| {
            let mut distance = 0;
            let mut cursor = *row;
            while cursor != start { cursor = previous[&cursor]; distance += 1; }
            distance
        })?;
    let mut hop = target;
    while previous[&hop] != start { hop = previous[&hop]; }
    legal.iter().copied().find(|&index| {
        vocab_name("verb", field(obs, "actions", index, 0)).as_deref() == Some("move")
            && vocab_name("row_group", field(obs, "actions", index, 3)).as_deref() == Some("map_nodes")
            && usize::try_from(field(obs, "actions", index, 4)).ok() == Some(hop)
    })
}

fn keys_route_action(obs: &RunObservation) -> usize {
    let legal = legal_indices(obs);
    assert!(!legal.is_empty(), "nonterminal Standard boundary had no legal candidate");
    let room = context(obs).2;
    let preferred = match room.as_str() {
        "campfire" => Some("recall"),
        "treasure" => Some("sapphire"),
        _ => None,
    };
    if let Some(choice) = preferred {
        if let Some(index) = legal.iter().copied().find(|&index|
            vocab_name("choice", field(obs, "actions", index, 5)).as_deref() == Some(choice)) {
            return index;
        }
    }
    if room == "map" {
        if let Some(index) = shortest_burning_elite_move(obs, &legal) { return index; }
        if let Some(index) = legal.iter().copied().find(|&index|
            vocab_name("verb", field(obs, "actions", index, 0)).as_deref() == Some("move")) { return index; }
    }
    simple_action(obs)
}

#[test]
fn standard_supplied_heart_routes_all_characters() {
    let mut outcomes = Vec::new();
    let mut failures = Vec::new();
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0xB0 + character_index as u8 + if ascension == 20 { 8 } else { 0 };
            let creation = short_creation();
            let (mut session, replay_start) = standard_session(character, ascension, seed, creation);
            let result = drive(&mut session, 12_000, |_| false, simple_action);
            let record = session.record().unwrap().expect("record_replay is enabled");
            let (progress, step_failure) = match result {
                Ok(progress) => (progress, None),
                Err(failure) => (failure.progress.clone(), Some(failure)),
            };
            let steps = progress.steps;
            let boss_fights = progress.boss_fights;
            let (reward_total, normal_floor_items, act4_floor_items) = assert_heart_rewards(&progress, ascension);
            assert_eq!(record.steps.len(), steps);
            let replay = replay_wounded_supplied(&record, replay_start, &progress.wound_counts);
            assert_eq!(replay.final_decision.terminal, record.terminal);
            let final_context = session.reproduction().unwrap().latest_decision
                .map(|decision| decision.map(|d| format!("act={}, floor={}, room={:?}", d.act, d.floor, d.room.as_ref().map(|r| r.kind))))
                .and_then(Result::ok).unwrap_or_else(|| step_failure.as_ref().map(|f| f.checkpoint.clone()).unwrap_or_else(|| "unknown".to_owned()));
            let diagnosis = step_failure.as_ref().map(|f| format!("step error at {}, attempted action row {:?}, {}, candidates=[{}], error={}", f.checkpoint, f.action_row, f.action, f.candidates, f.error))
                .unwrap_or_else(|| terminal_diagnosis(&session, record.terminal, progress.last_combat_turn));
            outcomes.push(format!("{character:?} A{ascension}: {steps} steps, {boss_fights} Act 3 boss fights, reward={reward_total}, items={{new_floor:{normal_floor_items}, act4_floor:{act4_floor_items}, heart_victory:1}}, wounds={:?}, {:?}, final={final_context}, {diagnosis}, replay OK", progress.wound_counts, record.terminal));
            if record.terminal != Some(RunTerminal::HeartVictory) || step_failure.is_some() {
                failures.push(format!("{character:?} A{ascension} did not reach HeartVictory: {diagnosis}"));
            }
            let expected_boss_fights = if ascension == 20 { 2 } else { 1 };
            if boss_fights != expected_boss_fights {
                failures.push(format!("{character:?} A{ascension} saw {boss_fights} Act 3 bosses, expected {expected_boss_fights}"));
            }
        }
    }
    eprintln!("standard_supplied_heart_routes_all_characters: {}", outcomes.join("; "));
    assert!(failures.is_empty(), "heart route failures: {}", failures.join("; "));
}

#[test]
fn standard_keys_route_itemises_first_keys() {
    let creation = short_creation();
    let mut summaries = Vec::new();
    for ascension in [0_u8, 20] {
        let mut completed = false;
        for seed in [0xC1_u8, 0xC2, 0xC3, 0xC4] {
            let run = standard_supplied_run_with(Character::Ironclad, ascension, creation.profile(), 900, false,
                RunRng::new([seed; 32], Partition::Test, 0)).unwrap();
            let replay_start = run.clone();
            let mut config = StandardRunConfig::new(Character::Ironclad, ascension, Partition::Test);
            config.creation = creation;
            config.record_replay = true;
            let mut session = RunSession::from_standard_supplied(config, run, u64::from(seed)).unwrap();
            let initial = session.decision().unwrap().observation;
            let initial_keys = [
                field(&initial, "context", 0, 8), field(&initial, "context", 0, 9), field(&initial, "context", 0, 10),
            ];
            let result = drive(&mut session, 3_000, |_| false, keys_route_action);
            let (progress, failure) = match result {
                Ok(progress) => (progress, None),
                Err(failure) => (failure.progress.clone(), Some(failure)),
            };
            assert!(failure.is_none(), "key route drive failed: {:?}", failure.map(|f| format!("{}; {}", f.checkpoint, f.error)));
            let mut counts = [0_usize; 3];
            for (_, items, _) in &progress.reward_steps {
                for item in items {
                    match item.kind {
                        RewardItemKind::FirstKey(Key::Ruby) => counts[0] += 1,
                        RewardItemKind::FirstKey(Key::Sapphire) => counts[1] += 1,
                        RewardItemKind::FirstKey(Key::Emerald) => counts[2] += 1,
                        _ => {}
                    }
                }
            }
            let got_all = counts == [1, 1, 1];
            let record = session.record().unwrap().unwrap();
            let all_attempt_items = progress.reward_steps.iter().flat_map(|(_, items, _)| items.iter()).collect::<Vec<_>>();
            let floor_items = all_attempt_items.iter().filter(|item| matches!(item.kind, RewardItemKind::NewFloor { .. })).count();
            let act4_items = all_attempt_items.iter().filter(|item| matches!(item.kind, RewardItemKind::NewAct4Floor { .. })).count();
            let reward_total = progress.reward_steps.iter().map(|(reward, _, _)| *reward).sum::<i32>();
            summaries.push(format!("A{ascension} seed {seed:#x}: steps={}, reward={reward_total}, items=new_floor:{floor_items}/act4_floor:{act4_items}/ruby:{}/sapphire:{}/emerald:{}/heart_victory:{}, terminal={:?}", progress.steps, counts[0], counts[1], counts[2], all_attempt_items.iter().filter(|item| item.kind == RewardItemKind::HeartVictory).count(), record.terminal));
            if !got_all { continue; }

            let mut previous = initial_keys;
            let mut all_items = Vec::new();
            let mut total = 0_i32;
            for (reward, items, [_, _, reward_total, ruby, sapphire, emerald]) in &progress.reward_steps {
                assert_eq!(items.iter().map(|item| item.amount).sum::<i32>(), *reward);
                let current = [*ruby, *sapphire, *emerald];
                let mut flips = Vec::new();
                for (index, key) in [Key::Ruby, Key::Sapphire, Key::Emerald].into_iter().enumerate() {
                    if previous[index] == 0 && current[index] == 1 { flips.push(key); }
                }
                let earned = items.iter().filter_map(|item| match item.kind {
                    RewardItemKind::FirstKey(key) => { assert_eq!(item.amount, 5); Some(key) }
                    _ => None,
                }).collect::<Vec<_>>();
                assert_eq!(earned, flips, "key reward occurs on the context 0→1 flag flip");
                previous = current;
                total += reward;
                all_items.extend(items.iter().copied());
                assert_eq!(*reward_total, i64::from(total));
            }
            assert_eq!(counts, [1, 1, 1]);
            assert_eq!(total, all_items.iter().map(|item| item.amount).sum::<i32>());
            assert_eq!(record.terminal, Some(RunTerminal::HeartVictory), "the all-key route reaches the Heart");
            let replay = replay_wounded_supplied(&record, replay_start, &progress.wound_counts);
            assert_eq!(replay.accepted_count, progress.steps);
            assert_eq!(replay.rewards, progress.reward_steps.iter().map(|(reward, _, _)| *reward).collect::<Vec<_>>());
            assert_eq!(replay.final_decision.terminal, record.terminal);
            completed = true;
            break;
        }
        assert!(completed, "no seed reached all first keys for A{ascension}; {}", summaries.join("; "));
    }
    eprintln!("standard_keys_route_itemises_first_keys: {}", summaries.join("; "));
}

#[test]
fn a20_default_fights_two_act3_bosses_and_one_with_any_unseen() {
    let character = Character::Silent;
    let mut failures = Vec::new();
    let mut outcomes = Vec::new();
    for (seed, unseen, expected_fights) in [(0xD1_u8, false, 2), (0xD2_u8, true, 1)] {
        let mut creation = StandardCreation::experienced();
        if unseen { creation.seen_bosses[2][0] = false; }
        let (mut session, replay_start) = standard_session(character, 20, seed, creation);
        let result = drive(&mut session, 12_000, |obs| context(obs).2 == "act3_ending", simple_action);
        let record = session.record().unwrap().unwrap();
        let (progress, step_failure) = match result {
            Ok(progress) => (progress, None),
            Err(failure) => (failure.progress.clone(), Some(failure)),
        };
        let steps = progress.steps;
        let fights = progress.boss_fights;
        let latest = session.reproduction().unwrap().latest_decision.and_then(Result::ok);
        let ended_at_act3 = latest.as_ref().is_some_and(|d| d.room.as_ref().is_some_and(|r| format!("{:?}", r.kind) == "Act3Ending"));
        let final_decision = replay_wounded_supplied(&record, replay_start, &progress.wound_counts).final_decision;
        assert_eq!(final_decision.terminal, record.terminal);
        let diagnosis = step_failure.as_ref().map(|f| format!("step error at {}, attempted action row {:?}, {}, candidates=[{}], error={}", f.checkpoint, f.action_row, f.action, f.candidates, f.error))
            .unwrap_or_else(|| terminal_diagnosis(&session, record.terminal, progress.last_combat_turn));
        outcomes.push(format!("unseen={unseen}: {steps} steps, {fights} Act 3 boss fights, wounds={:?}, {:?}, act3_ending={ended_at_act3}, {diagnosis}, replay OK", progress.wound_counts, record.terminal));
        if !ended_at_act3 { failures.push(format!("unseen={unseen} stopped before Act 3 ending: {diagnosis}")); }
        if fights != expected_fights { failures.push(format!("unseen={unseen} saw {fights} Act 3 bosses, expected {expected_fights}")); }
    }
    eprintln!("a20_default_fights_two_act3_bosses_and_one_with_any_unseen: {}", outcomes.join("; "));
    assert!(failures.is_empty(), "A20 boss route failures: {}", failures.join("; "));
}

#[test]
fn standard_random_rollouts() {
    let mut outcomes = Vec::new();
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0x5A00_u64 + (character_index as u64 * 2) + u64::from(ascension == 20);
            let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
            config.record_replay = true;
            let mut session = RunSession::new_standard(config).unwrap();
            let mut boundary = session.reset(Some(seed)).unwrap();
            let mut state = seed ^ 0x9E3779B97F4A7C15;
            let mut steps = 0usize;
            while boundary.terminal.is_none() && steps < 3_000 {
                let legal = legal_indices(&boundary.observation);
                assert!(!legal.is_empty(), "nonterminal boundary has no legal action: {character:?} A{ascension}");
                state ^= state << 13; state ^= state >> 7; state ^= state << 17;
                let index = legal[(state as usize) % legal.len()];
                let result = session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
                boundary = result.boundary;
                steps += 1;
            }
            let record = session.record().unwrap().expect("random rollout record");
            assert_eq!(record.steps.len(), steps);
            let replay = crate::replay_run(&record, RunCapacities::standard_default()).unwrap();
            let terminal = record.terminal.map(|terminal| format!("{terminal:?}")).unwrap_or_else(|| "step_limit".to_owned());
            outcomes.push(format!("{character:?} A{ascension}: {terminal} after {steps} steps (replay OK)"));
            assert!(replay.final_decision.terminal == record.terminal);
        }
    }
    eprintln!("standard_random_rollouts: {}", outcomes.join("; "));
}
