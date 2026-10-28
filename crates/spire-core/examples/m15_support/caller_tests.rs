use super::*;
use spire_core::{
    content::Character,
    run::{
        Candidate, ChoiceId, CombatProgress, CommittedEvent, GameAction, MapNodeKind, NodeId,
        PublicDecision, PublicPlayer, RoomChoiceKind, RoomChoiceStatus, RoomChoiceView,
        RoomScenario, RoomView, RoomViewKind, RunTerminal, RunTransition,
    },
};

fn choice(id: u64, kind: RoomChoiceKind) -> RoomChoiceView {
    RoomChoiceView { id: ChoiceId(id), kind, price: None, status: RoomChoiceStatus::Offered }
}

fn candidate(id: u64) -> Candidate {
    Candidate { id: ChoiceId(id), action: GameAction::Choose(ChoiceId(id)), label: "choice", neow_offer: None, object: None }
}

fn move_candidate(id: u64) -> Candidate {
    Candidate { id: ChoiceId(id), action: GameAction::Move(NodeId(id)), label: "move", neow_offer: None, object: None }
}

fn decision(kind: RoomViewKind, choices: Vec<RoomChoiceView>, keys: [bool; 3]) -> PublicDecision {
    PublicDecision {
        content_profile: Some(RoomScenario::M15SilentHeart),
        room: Some(RoomView { kind, choices }),
        reward_total: 0,
        act: 1,
        floor: 1,
        player: PublicPlayer {
            relic_counters: vec![], character: Character::Silent, hp: 50, max_hp: 70, gold: 99,
            relics: vec![], potions: vec![None; 3], keys,
        },
        combat: None,
        combat_progress: CombatProgress::default(),
        last_combat: None,
        objects: vec![],
        candidates: vec![],
        offers: vec![],
        selection: None,
        terminal: None,
    }
}

fn map_node(id: u64, kind: MapNodeKind, edges: &[u64], visited: bool, burning: Option<bool>) -> spire_core::run::map::PublicMapNode {
    spire_core::run::map::PublicMapNode {
        id: NodeId(id), floor: id as u32, x: id as u8, y: id as u8, kind, resolved_room: None,
        edges: edges.iter().copied().map(NodeId).collect(), visited, revealed: true, burning,
    }
}

fn public_map(nodes: Vec<spire_core::run::map::PublicMapNode>) -> spire_core::run::map::PublicMap {
    spire_core::run::map::PublicMap { boss: None, act: 3, nodes, current: Some(NodeId(0)), winged_moves: 0 }
}

fn with_choice_candidates(mut value: PublicDecision) -> PublicDecision {
    value.candidates = value.room.as_ref().unwrap().choices.iter()
        .map(|row| Candidate {
            id: row.id, action: GameAction::Choose(row.id), label: "choice", neow_offer: None, object: None,
        }).collect();
    value
}

fn empty_combat() -> spire_core::combat::CombatPublic {
    use spire_core::combat::{CharacterCombatView, CombatPublic, FullPlayerView, Stance};
    CombatPublic {
        played_powers: vec![], playing_cards: vec![], stasis: vec![], content_id: "synthetic".into(), turn: 1,
        player: FullPlayerView { hp: 50, max_hp: 70, block: 0, energy: 3, max_energy: 3, powers: vec![] },
        enemies: vec![], character: CharacterCombatView { stance: Stance::Neutral, orbs: vec![], orb_dark_damage:vec![], focus: 0 },
        hand: vec![], draw_count: 0, discard: vec![], exhaust: vec![], outcome: None,
        hp_before_healing: None, hp_after_cleanup: None, targets: vec![], selection: None,
        known_draw: vec![], encounter_facing: None, inventory: None, card_offer: None,
    }
}

#[test]
fn m15_public_caller_pursues_only_missing_keys_with_deterministic_ties() {
    let mut campfire = decision(
        RoomViewKind::Campfire,
        vec![choice(1, RoomChoiceKind::Rest { heal: 20 }), choice(2, RoomChoiceKind::Recall), choice(3, RoomChoiceKind::Smith)],
        [false, false, false],
    );
    campfire.candidates = vec![candidate(1), candidate(2), candidate(3)];
    assert_eq!(key_action(&campfire, None), Some(GameAction::Choose(ChoiceId(2))));

    campfire.player.keys[0] = true;
    assert_eq!(key_action(&campfire, None), None, "owned Ruby ends this key objective");

    let unopened = with_choice_candidates(decision(
        RoomViewKind::Treasure,
        vec![choice(4, RoomChoiceKind::OpenChest), choice(5, RoomChoiceKind::Relic(spire_core::content::RelicId::Vajra))],
        [true, false, false],
    ));
    assert_eq!(key_action(&unopened, None), Some(GameAction::Choose(ChoiceId(4))));
    let opened = with_choice_candidates(decision(
        RoomViewKind::Treasure,
        vec![choice(6, RoomChoiceKind::Relic(spire_core::content::RelicId::Vajra)), choice(7, RoomChoiceKind::Sapphire)],
        [true, false, false],
    ));
    assert_eq!(key_action(&opened, None), Some(GameAction::Choose(ChoiceId(7))));
    let sapphire_owned = with_choice_candidates(decision(
        RoomViewKind::Treasure,
        vec![choice(18, RoomChoiceKind::Sapphire), choice(19, RoomChoiceKind::Relic(spire_core::content::RelicId::Vajra))],
        [true, true, false],
    ));
    assert_eq!(key_action(&sapphire_owned, None), None, "owned Sapphire leaves ordinary chest policy in charge");

    let emerald = with_choice_candidates(decision(
        RoomViewKind::Rewards,
        vec![choice(8, RoomChoiceKind::Relic(spire_core::content::RelicId::Vajra)), choice(9, RoomChoiceKind::Emerald)],
        [true, true, false],
    ));
    assert_eq!(key_action(&emerald, None), Some(GameAction::Choose(ChoiceId(9))));

    let mut ruby_priority = decision(
        RoomViewKind::Campfire,
        vec![choice(50, RoomChoiceKind::Emerald), choice(51, RoomChoiceKind::Recall)],
        [false, false, false],
    );
    ruby_priority.candidates = vec![candidate(50), candidate(51)];
    assert_eq!(
        key_action(&ruby_priority, None),
        Some(GameAction::Choose(ChoiceId(51))),
        "offered legal Ruby objective precedes Emerald when rows coexist",
    );

    let mut sapphire_priority = decision(
        RoomViewKind::Treasure,
        vec![choice(52, RoomChoiceKind::Emerald), choice(53, RoomChoiceKind::Sapphire)],
        [true, false, false],
    );
    sapphire_priority.candidates = vec![candidate(52), candidate(53)];
    assert_eq!(
        key_action(&sapphire_priority, None),
        Some(GameAction::Choose(ChoiceId(53))),
        "offered legal Sapphire objective precedes Emerald when rows coexist",
    );

    let route_map = public_map(vec![
        map_node(1, MapNodeKind::Monster, &[4], false, None),
        map_node(2, MapNodeKind::Shop, &[3], false, None),
        map_node(3, MapNodeKind::Monster, &[5], false, None),
        map_node(4, MapNodeKind::Elite, &[9], false, Some(true)),
        map_node(5, MapNodeKind::Elite, &[9], false, Some(true)),
        map_node(9, MapNodeKind::Boss, &[], false, None),
    ]);
    let mut route = decision(RoomViewKind::Map, vec![], [true, true, false]);
    route.act = 3;
    route.candidates = vec![move_candidate(2), move_candidate(1)];
    assert_eq!(key_action(&route, Some(&route_map)), Some(GameAction::Move(NodeId(1))), "minimum distance outranks presentation order");

    for act in [1, 2] {
        let mut early_decision = route.clone();
        early_decision.act = act;
        let mut early_map = route_map.clone();
        early_map.act = act;
        assert_eq!(
            key_action(&early_decision, Some(&early_map)),
            Some(GameAction::Move(NodeId(1))),
            "missing Emerald may route to a burning Elite across Acts 1–3",
        );
        early_map.act = if act == 1 { 2 } else { 1 };
        assert_eq!(
            key_action(&early_decision, Some(&early_map)),
            None,
            "route decision and map must describe the same act",
        );
    }

    let tied_map = public_map(vec![
        map_node(3, MapNodeKind::Monster, &[13], false, None),
        map_node(7, MapNodeKind::Monster, &[17], false, None),
        map_node(13, MapNodeKind::Elite, &[20], false, Some(true)),
        map_node(17, MapNodeKind::Elite, &[20], false, Some(true)),
        map_node(20, MapNodeKind::Boss, &[], false, None),
    ]);
    route.candidates = vec![move_candidate(7), move_candidate(3)];
    assert_eq!(key_action(&route, Some(&tied_map)), Some(GameAction::Move(NodeId(3))), "equal route distance uses the smaller next NodeId");
    route.player.keys[2] = true;
    assert_eq!(key_action(&route, Some(&tied_map)), None, "owned Emerald ends route seeking");

    let dead_end = public_map(vec![
        map_node(1, MapNodeKind::Monster, &[4], false, None),
        map_node(4, MapNodeKind::Elite, &[], false, Some(true)),
        map_node(5, MapNodeKind::Monster, &[9], false, Some(true)),
        map_node(6, MapNodeKind::Elite, &[9], true, Some(true)),
        map_node(7, MapNodeKind::Elite, &[9], false, None),
        map_node(9, MapNodeKind::Boss, &[], false, None),
    ]);
    route.player.keys[2] = false;
    route.candidates = vec![move_candidate(1), move_candidate(5), move_candidate(6), move_candidate(7)];
    assert_eq!(key_action(&route, Some(&dead_end)), None, "dead-end, non-Elite, and visited burning markers are not targets");

    let stage4 = with_choice_candidates(decision(
        RoomViewKind::Act3Ending,
        vec![choice(10, RoomChoiceKind::Act3Continue { stage: 1 }), choice(11, RoomChoiceKind::Act3Continue { stage: 4 })],
        [true, true, false],
    ));
    assert_eq!(key_action(&stage4, None), Some(GameAction::Choose(ChoiceId(11))));
    let stage1_only = with_choice_candidates(decision(
        RoomViewKind::Act3Ending,
        vec![choice(12, RoomChoiceKind::Act3Continue { stage: 1 })],
        [true, true, false],
    ));
    assert_eq!(key_action(&stage1_only, None), None, "earlier dialog stages use the ordinary selector");

    let heart = with_choice_candidates(decision(RoomViewKind::HeartVictory, vec![choice(13, RoomChoiceKind::Proceed)], [true; 3]));
    assert_eq!(key_action(&heart, None), Some(GameAction::Choose(ChoiceId(13))));
    let non_proceed_heart = with_choice_candidates(decision(RoomViewKind::HeartVictory, vec![choice(22, RoomChoiceKind::Leave)], [true; 3]));
    assert_eq!(key_action(&non_proceed_heart, None), None);
    assert_eq!(m13_support::m15_first_offered_legal(&non_proceed_heart), None, "Act4 fallback cannot substitute a non-Proceed choice");

    let mut disabled = decision(RoomViewKind::Campfire, vec![choice(14, RoomChoiceKind::Recall)], [false; 3]);
    disabled.room.as_mut().unwrap().choices[0].status = RoomChoiceStatus::Disabled;
    disabled.candidates = vec![candidate(14)];
    assert_eq!(key_action(&disabled, None), None, "disabled offer is ignored even when a stale candidate remains");
    let mut absent = decision(RoomViewKind::Campfire, vec![choice(15, RoomChoiceKind::Recall)], [false; 3]);
    absent.candidates.clear();
    assert_eq!(key_action(&absent, None), None, "an offered row without a legal candidate is ignored");
    let mut stale = decision(RoomViewKind::Campfire, vec![choice(20, RoomChoiceKind::Recall)], [false; 3]);
    stale.candidates = vec![candidate(21)];
    assert_eq!(key_action(&stale, None), None, "a different legal choice cannot satisfy the offered key row");
    let mut suspended = campfire.clone();
    suspended.selection = Some(spire_core::run::PublicSelection { required: 1, selected_count: 0, eligible: vec![] });
    assert_eq!(key_action(&suspended, None), None, "selection child suppresses parent key choice");
    suspended.selection = None;
    suspended.combat = Some(empty_combat());
    assert_eq!(key_action(&suspended, None), None, "combat child suppresses parent key choice");

    let fallback = with_choice_candidates(decision(
        RoomViewKind::Rewards,
        vec![choice(16, RoomChoiceKind::Leave), choice(17, RoomChoiceKind::Skip)],
        [true; 3],
    ));
    assert_eq!(m13_support::m15_first_offered_legal(&fallback), Some(GameAction::Choose(ChoiceId(16))), "new-choice fallback keeps offer order");
    let mut reordered = fallback.clone();
    reordered.room.as_mut().unwrap().choices.reverse();
    assert_eq!(m13_support::m15_first_offered_legal(&reordered), Some(GameAction::Choose(ChoiceId(17))));
}

#[test]
fn m15_public_caller_routes_to_the_emerald_elite_during_acts_one_and_two() {
    let mut map = public_map(vec![
        map_node(1, MapNodeKind::Monster, &[4], false, None),
        map_node(4, MapNodeKind::Elite, &[9], false, Some(true)),
        map_node(9, MapNodeKind::Boss, &[], false, None),
    ]);
    for act in [1, 2] {
        let mut current = decision(RoomViewKind::Map, vec![], [true, true, false]);
        current.act = act;
        current.candidates = vec![move_candidate(1)];
        map.act = act;
        assert_eq!(
            key_action(&current, Some(&map)),
            Some(GameAction::Move(NodeId(1))),
            "Act {act} route remains directed through the burning Elite to the Boss",
        );
    }
}

fn final_heart_decision() -> PublicDecision {
    let mut decision = decision(RoomViewKind::TrueVictory, vec![], [true; 3]);
    decision.act = 4;
    decision.floor = 55;
    decision.terminal = Some(RunTerminal::HeartVictory);
    decision
}

fn heart_proceed_trace() -> TraceStep {
    let mut before = decision(RoomViewKind::HeartVictory, vec![choice(42, RoomChoiceKind::Proceed)], [true; 3]);
    before.act = 4;
    before.floor = 54;
    before.candidates = vec![candidate(42)];
    TraceStep {
        decision: before,
        map: None,
        action: GameAction::Choose(ChoiceId(42)),
        transition: RunTransition {
            reward: 110,
            terminal: Some(RunTerminal::HeartVictory),
            committed_events: vec![
                CommittedEvent::FloorEntered { act: 4, floor: 55 },
                CommittedEvent::Ended(RunTerminal::HeartVictory),
            ],
        },
    }
}

#[test]
fn m15_public_caller_heart_success_requires_final_proceed_terminal() {
    let entry = {
        let mut value = decision(RoomViewKind::Act4Ready, vec![], [true; 3]);
        value.act = 4;
        value.floor = 50;
        value
    };
    assert_eq!(m13_support::m15_stop_outcome(&entry, None).unwrap(), None);
    let mut pending = decision(RoomViewKind::HeartVictory, vec![choice(42, RoomChoiceKind::Proceed)], [true; 3]);
    pending.act = 4;
    pending.floor = 54;
    assert_eq!(m13_support::m15_stop_outcome(&pending, None).unwrap(), None);
    assert_eq!(m13_support::m15_stop_or_budget(&pending, None, 12, 12).unwrap(), Some(M15CallerOutcome::BudgetExhausted));

    let final_decision = final_heart_decision();
    let trace = heart_proceed_trace();
    assert_eq!(m13_support::m15_stop_outcome(&final_decision, Some(&trace)).unwrap(), Some(M15CallerOutcome::HeartVictory));
    assert_eq!(m13_support::m15_stop_or_budget(&final_decision, Some(&trace), 12, 12).unwrap(), Some(M15CallerOutcome::HeartVictory), "the final accepted action may use the whole budget");
    assert_eq!(M15CallerOutcome::HeartVictory.to_string(), "HeartVictory");

    assert!(m13_support::m15_stop_outcome(&final_decision, None).is_err(), "terminal without accepted trace is forged");
    let mut wrong_action = trace.clone();
    wrong_action.action = GameAction::Choose(ChoiceId(99));
    assert!(m13_support::m15_stop_outcome(&final_decision, Some(&wrong_action)).is_err());
    let mut disabled_proceed = trace.clone();
    disabled_proceed.decision.room.as_mut().unwrap().choices[0].status = RoomChoiceStatus::Disabled;
    assert!(m13_support::m15_stop_outcome(&final_decision, Some(&disabled_proceed)).is_err());
    let mut wrong_transition = trace.clone();
    wrong_transition.transition.committed_events.pop();
    assert!(m13_support::m15_stop_outcome(&final_decision, Some(&wrong_transition)).is_err());
    let mut wrong_endpoint = final_decision.clone();
    wrong_endpoint.floor = 54;
    assert!(m13_support::m15_stop_outcome(&wrong_endpoint, Some(&trace)).is_err());

    for (terminal, expected) in [
        (RunTerminal::Death, M15CallerOutcome::Death),
        (RunTerminal::Act3NoKeys, M15CallerOutcome::Act3NoKeys),
        (RunTerminal::EmptyDeck, M15CallerOutcome::EmptyDeck),
    ] {
        let mut terminal_decision = decision(RoomViewKind::TrueVictory, vec![], [false; 3]);
        terminal_decision.terminal = Some(terminal);
        assert_eq!(m13_support::m15_stop_outcome(&terminal_decision, None).unwrap(), Some(expected));
    }

    let mut old_m14_heart = final_decision.clone();
    old_m14_heart.content_profile = Some(RoomScenario::M14SilentAct3);
    assert_eq!(m13_support::m14_stop_outcome(&old_m14_heart), None);
    let mut old_m14_entry = entry.clone();
    old_m14_entry.content_profile = Some(RoomScenario::M14SilentAct3);
    assert_eq!(m13_support::m14_stop_outcome(&old_m14_entry), Some(m13_support::CallerOutcome::Act4Ready));
    assert_eq!(m13_support::CallerOutcome::Act4Ready.to_string(), "Act4Ready (supplied M14 cutoff)");
    assert_eq!(m13_support::CallerOutcome::Death.to_string(), "Death (normal terminal outcome)");
    let legacy_labels = [
        m13_support::CallerOutcome::Act3Ready,
        m13_support::CallerOutcome::Death,
        m13_support::CallerOutcome::BudgetExhausted,
        m13_support::CallerOutcome::Act3NoKeys,
        m13_support::CallerOutcome::Act4Ready,
        m13_support::CallerOutcome::EmptyDeck,
    ];
    let legacy_exhaustive = |outcome| match outcome {
        m13_support::CallerOutcome::Act3Ready => "Act3Ready",
        m13_support::CallerOutcome::Death => "Death (normal terminal outcome)",
        m13_support::CallerOutcome::BudgetExhausted => "caller action budget exhausted",
        m13_support::CallerOutcome::Act3NoKeys => "Act3NoKeys (normal non-Heart completion)",
        m13_support::CallerOutcome::Act4Ready => "Act4Ready (supplied M14 cutoff)",
        m13_support::CallerOutcome::EmptyDeck => "EmptyDeck (normal terminal outcome)",
    };
    assert_eq!(legacy_labels.map(|outcome| legacy_exhaustive(outcome)), [
        "Act3Ready", "Death (normal terminal outcome)", "caller action budget exhausted",
        "Act3NoKeys (normal non-Heart completion)", "Act4Ready (supplied M14 cutoff)",
        "EmptyDeck (normal terminal outcome)",
    ]);
    assert!(Caller::new_m15(8192).is_ok());
    assert!(Caller::new_m15(8193).is_err());
    let _: fn(
        &mut spire_core::run::Run,
        &mut Caller,
        fn(&PublicDecision, Option<&spire_core::run::map::PublicMap>) -> Option<GameAction>,
    ) -> Result<M15CallerOutcome, String> = rollout_m15;
}

#[test]
fn m15_public_caller_receipt_uses_native_replay_action_verbs() {
    use spire_core::combat::{CombatAction, TargetId};

    assert_eq!(m13_support::m15_first_offered_legal(&with_choice_candidates(decision(
        RoomViewKind::Rewards,
        vec![choice(30, RoomChoiceKind::Leave)],
        [true; 3],
    ))), Some(GameAction::Choose(ChoiceId(30))));
    assert_eq!(super::native_action_value(&GameAction::Move(NodeId(5))), serde_json::json!({"verb":"move","node":5}));
    assert_eq!(super::native_action_value(&GameAction::Choose(ChoiceId(8))), serde_json::json!({"verb":"choose","choice":8}));
    assert_eq!(super::native_action_value(&GameAction::Combat(CombatAction::PlayCard { instance: 17, target: Some(TargetId(2)) })),
        serde_json::json!({"verb":"combat.play_card","instance":17,"target":2}));
    assert_eq!(super::native_action_value(&GameAction::Combat(CombatAction::ChooseCardOffer { index: 1 })),
        serde_json::json!({"verb":"combat.choose_card_offer","index":1}));
    assert_eq!(super::native_action_value(&GameAction::Combat(CombatAction::SkipCardOffer)),
        serde_json::json!({"verb":"combat.skip_card_offer"}));
    assert_eq!(FIXED_CASES.len(), 6);
    assert_eq!(FIXED_CASES[0].0, 7);
    assert_eq!(FIXED_CASES[5].0, 11);
}

#[cfg(test)]
mod m15_receipt_journal_tests {
    use super::super as m15_support;
    use m15_support::{Caller,TraceStep,M15CallerOutcome};
    use spire_core::run::RoomScenario;
    use spire_core::{
        content::Character,
        run::{
            Candidate, ChoiceId, CommittedEvent, GameAction, NodeId, PublicDecision,
            PublicPlayer, RoomView, RoomViewKind, RunTransition, CombatProgress,
        },
    };

    fn decision(actions: &[u64], floor: u32) -> PublicDecision {
        PublicDecision {
            content_profile: Some(RoomScenario::M15SilentHeart),
            room: Some(RoomView { kind: RoomViewKind::Map, choices: vec![] }),
            reward_total: 0,
            act: 1,
            floor,
            player: PublicPlayer {
                relic_counters: vec![], character: Character::Silent, hp: 50, max_hp: 70, gold: 99,
                relics: vec![], potions: vec![None; 3], keys: [false; 3],
            },
            combat: None,
            combat_progress: CombatProgress::default(),
            last_combat: None,
            objects: vec![],
            candidates: actions.iter().enumerate().map(|(index, node)| Candidate {
                id: ChoiceId(index as u64),
                action: GameAction::Move(NodeId(*node)),
                label: "move",
                neow_offer: None,
                object: None,
            }).collect(),
            offers: vec![],
            selection: None,
            terminal: None,
        }
    }

    fn public_map(current: u64) -> spire_core::run::map::PublicMap {
        spire_core::run::map::PublicMap {
            boss: None, act: 1, nodes: vec![], current: Some(NodeId(current)), winged_moves: 0,
        }
    }

    fn trace_step(decision: PublicDecision, map: spire_core::run::map::PublicMap, node: u64, floor: u32) -> TraceStep {
        TraceStep {
            decision,
            map: Some(map),
            action: GameAction::Move(NodeId(node)),
            transition: RunTransition {
                reward: floor as i32,
                terminal: None,
                committed_events: vec![CommittedEvent::FloorEntered { act: 1, floor: floor + 1 }],
            },
        }
    }

    fn output_dir(tag: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("m15-receipt-{tag}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn receipt_steps_derive_candidates_and_posts_from_the_single_trace_journal() {
        let directory = output_dir("trace-derivation");
        let scenario = m15_support::FIXED_CASES[0].1;
        m15_support::preflight_attempt_files(&directory, &[scenario], 7).unwrap();
        let mut caller = Caller::new_m15(8192).unwrap();
        // Exercise the public illegal-override guard to obtain its real failure
        // flag, then stage the explicitly synthetic accepted-prefix fixture.
        let mut control=m15_support::make_run(7,scenario).unwrap();
        let failure=m15_support::rollout_m15(&mut control,&mut caller,|_,_|Some(GameAction::Move(NodeId(31)))).unwrap_err();
        assert!(failure.contains("outside the public menu"));assert!(caller.trace.is_empty());
        let first_decision = decision(&[9, 10], 1);
        let first_map = public_map(1);
        let second_decision = decision(&[20], 2);
        let second_map = public_map(2);
        let failed_boundary = decision(&[30], 3);
        let failed_boundary_map = public_map(3);
        caller.trace.push(trace_step(first_decision.clone(), first_map.clone(), 10, 1));
        caller.trace.push(trace_step(second_decision.clone(), second_map.clone(), 20, 2));
        caller.final_decision = Some(failed_boundary.clone());
        caller.final_map = Some(failed_boundary_map.clone());

        let error = "synthetic failure after two accepted actions".to_owned();
        m15_support::persist_attempt(
            &directory,
            7,
            scenario,
            8192,
            &Err::<M15CallerOutcome, String>(error.clone()),
            &caller,
            None,
        ).unwrap();

        let filename = format!("seed-7-{:?}-{:?}-{:?}.attempt.json", scenario.act1_boss, scenario.act2_boss, scenario.act3_boss);
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(directory.join(filename)).unwrap()).unwrap();
        assert_eq!(value["error"], error);
        assert_eq!(value["accepted_action_count"], 2);
        assert_eq!(value["actions"], serde_json::json!([
            {"verb":"move","node":10},
            {"verb":"move","node":20}
        ]));
        assert_eq!(value["steps"].as_array().unwrap().len(), 2);
        assert_eq!(value["steps"][0]["index"], 0, "index remains the trace ordinal");
        assert_eq!(value["steps"][0]["candidate_index"], 1, "candidate index is the position in the public legal menu");
        assert_eq!(value["steps"][0]["pre"]["decision_debug"], format!("{first_decision:#?}"));
        assert_eq!(value["steps"][0]["pre"]["map_debug"], format!("{first_map:#?}"));
        assert_eq!(value["steps"][0]["post"]["decision"]["debug"], format!("{second_decision:#?}"));
        assert_eq!(value["steps"][0]["post"]["map_debug"], format!("{second_map:#?}"));
        assert_eq!(value["steps"][1]["index"], 1);
        assert_eq!(value["steps"][1]["candidate_index"], 0);
        assert_eq!(value["steps"][1]["post"]["decision"]["debug"], format!("{failed_boundary:#?}"));
        assert_eq!(value["steps"][1]["post"]["map_debug"], format!("{failed_boundary_map:#?}"));
        assert_eq!(value["failed_action"], serde_json::json!({"verb":"move","node":31}));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn receipt_export_fails_closed_when_an_accepted_action_is_absent_from_its_public_menu() {
        let directory = output_dir("missing-candidate");
        let scenario = m15_support::FIXED_CASES[0].1;
        m15_support::preflight_attempt_files(&directory, &[scenario], 7).unwrap();
        let mut caller = Caller::new_m15(8192).unwrap();
        caller.trace.push(trace_step(decision(&[], 1), public_map(1), 4, 1));

        let result = m15_support::persist_attempt(
            &directory,
            7,
            scenario,
            8192,
            &Err::<M15CallerOutcome, String>("synthetic error".into()),
            &caller,
            None,
        );
        assert!(result.is_err(), "receipt export must not invent a candidate index");
        let filename = format!("seed-7-{:?}-{:?}-{:?}.attempt.json", scenario.act1_boss, scenario.act2_boss, scenario.act3_boss);
        assert!(!directory.join(filename).exists(), "invalid receipt must fail before writing either output");
        std::fs::remove_dir_all(directory).unwrap();
    }
}
