#[path = "../examples/m20_support/mod.rs"]
mod m20_support;

use m20_support::{policy, Caller, Scenario, CALLER_POLICY_REVISION, FIXED_CASES};
use spire_core::{
    combat::{
        CardInstanceId, CharacterCombatView, CombatAction, CombatCardOfferView, CombatPublic,
        CombatCardRef, EnemyHandle, FullEnemyView,
        FullIntentId, FullPlayerView, LiveCardView, Stance, TargetId,
    },
    content::{CardId, Character, EncounterId, EnemyId},
    run::{
        Candidate, ChoiceId, CombatProgress, CommittedEvent, GameAction, MapNodeKind, NodeId,
        PublicDecision, PublicObject, PublicPlayer, PublicSelection, RoomChoiceKind,
        RoomChoiceStatus, RoomChoiceView, RoomScenario, RoomView, RoomViewKind, RunTransition,
        SelectionId,
    },
};

fn scenario() -> Scenario {
    FIXED_CASES[0].1
}

fn choice(id: u64, kind: RoomChoiceKind, price: Option<u32>) -> RoomChoiceView {
    RoomChoiceView { id: ChoiceId(id), kind, price, status: RoomChoiceStatus::Offered }
}

fn candidate(id: u64, action: GameAction) -> Candidate {
    Candidate { id: ChoiceId(id), action, label: "synthetic public candidate", neow_offer: None, object: None }
}

fn choose_candidate(id: u64) -> Candidate {
    candidate(id, GameAction::Choose(ChoiceId(id)))
}

fn decision(kind: RoomViewKind, choices: Vec<RoomChoiceView>) -> PublicDecision {
    PublicDecision {
        content_profile: Some(RoomScenario::M20DefectHeart),
        room: Some(RoomView { kind, choices }),
        reward_total: 0,
        act: 1,
        floor: 1,
        player: PublicPlayer {
            relic_counters: vec![], character: Character::Defect, hp: 40, max_hp: 70, gold: 99,
            relics: vec![], potions: vec![None; 3], keys: [false; 3],
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

fn empty_combat() -> CombatPublic {
    CombatPublic {
        mantra_gained:None, stance_choice:None,
        played_powers: vec![], playing_cards: vec![], stasis: vec![], content_id: "synthetic".into(), turn: 1,
        player: FullPlayerView { hp: 40, max_hp: 70, block: 0, energy: 3, max_energy: 3, powers: vec![] },
        enemies: vec![],
        character: CharacterCombatView { stance: Stance::Neutral, orbs: vec![], orb_dark_damage: vec![], focus: 0 },
        hand: vec![], draw_count: 0, discard: vec![], exhaust: vec![], outcome: None,
        hp_before_healing: None, hp_after_cleanup: None, targets: vec![], selection: None,
        known_draw: vec![], encounter_facing: None, inventory: None, card_offer: None,
    }
}

fn live_card(id: CardId, instance: u64) -> LiveCardView {
    LiveCardView { watcher:None,
        instance: CardInstanceId(instance), id, origin: CombatCardRef::Permanent(CardInstanceId(instance)),
        upgrades: 0, misc: 0, effective_cost: Some(1), values: None,
    }
}

fn enemy(hp: i32, instance: &str) -> FullEnemyView {
    FullEnemyView {
        instance_id: EnemyHandle::parse(instance).unwrap(), enemy_id: EnemyId::Cultist,
        hp, max_hp: 40, block: 0, powers: vec![], intent: FullIntentId::Attack,
        intent_damage: None, replaced: false, escaped: false, half_dead: false, logical_slot: 0,
    }
}

fn combat_decision(combat: CombatPublic, candidates: Vec<Candidate>) -> PublicDecision {
    let mut value = decision(RoomViewKind::Combat, vec![]);
    value.combat = Some(combat);
    value.candidates = candidates;
    value
}

fn illegal_public_action(
    _decision: &PublicDecision,
    _map: Option<&spire_core::run::map::PublicMap>,
    _caller: &Caller,
) -> Result<GameAction, String> {
    Ok(GameAction::Cancel)
}

#[test]
fn m20_caller_contract_freezes_case_triples_budget_and_normal_constructor_boundary() {
    assert_eq!(CALLER_POLICY_REVISION, "m20-public-caller-v1");
    assert_eq!(m20_support::ACTION_LIMIT_MAX, 8192);
    assert_eq!(FIXED_CASES.len(), 6);
    assert_eq!(FIXED_CASES.iter().map(|(seed, _)| *seed).collect::<Vec<_>>(), vec![7, 7, 7, 11, 11, 11]);
    assert_eq!(FIXED_CASES[0].1.act1_boss, EncounterId::Guardian);
    assert_eq!(FIXED_CASES[0].1.act2_boss, EncounterId::Champ);
    assert_eq!(FIXED_CASES[0].1.act3_boss, EncounterId::AwakenedOne);
    assert_eq!(FIXED_CASES[5].1.act1_boss, EncounterId::SlimeBoss);
    assert_eq!(FIXED_CASES[5].1.act2_boss, EncounterId::Champ);
    assert_eq!(FIXED_CASES[5].1.act3_boss, EncounterId::TimeEater);
    assert!(Caller::new_m20(0).is_ok());
    assert!(Caller::new_m20(8192).is_ok());
    assert!(Caller::new_m20(8193).is_err());

    let zero = m20_support::parse_args(Vec::<String>::new()).unwrap();
    assert_eq!(zero.limit, 8192);
    assert_eq!(zero.scenarios.len(), 3);
    assert!(m20_support::parse_args(["--budget", "8193"].map(str::to_owned)).is_err());
    assert!(m20_support::parse_args(["--budget", "-1"].map(str::to_owned)).is_err());
    assert_eq!(m20_support::parse_args(["--budget", "0"].map(str::to_owned)).unwrap().limit, 0);
    assert!(m20_support::parse_args(["--seed", "9"].map(str::to_owned)).is_err());
    assert!(m20_support::parse_args(["--scenario", "death"].map(str::to_owned)).is_err());
    assert!(m20_support::parse_args(["--unknown"].map(str::to_owned)).is_err());
    assert!(m20_support::parse_args(["--seed", "7", "--seed", "11"].map(str::to_owned)).is_err());

    for seed in [7, 11] {
        for scenario in m20_support::cases_for_seed(seed).unwrap() {
            let run = m20_support::make_run(seed, scenario).unwrap();
            let initial = run.decision().unwrap();
            assert_eq!(initial.content_profile, Some(RoomScenario::M20DefectHeart));
            assert_eq!(initial.player.character, Character::Defect);
            assert!(initial.terminal.is_none());
        }
    }
    assert!(m20_support::make_run(9, scenario()).is_err());
}

#[test]
fn m20_caller_shared_rollout_obeys_zero_and_single_action_budget_without_extending_history() {
    let (seed, scenario) = FIXED_CASES[0];
    let mut zero_run = m20_support::make_run(seed, scenario).unwrap();
    let mut zero = Caller::new_m20(0).unwrap();
    let zero_result = m20_support::shared::rollout_m20(&mut zero_run, &mut zero, policy).unwrap();
    assert_eq!(zero_result, m20_support::M20CallerOutcome::BudgetExhausted);
    assert_eq!(zero.accepted(), 0);

    let mut one_run = m20_support::make_run(seed, scenario).unwrap();
    let mut one = Caller::new_m20(1).unwrap();
    let one_result = m20_support::shared::rollout_m20(&mut one_run, &mut one, policy).unwrap();
    assert_eq!(one_result, m20_support::M20CallerOutcome::BudgetExhausted);
    assert_eq!(one.accepted(), 1, "the shared M20 loop stops after the one-action budget");
}

#[test]
fn m20_caller_public_policy_routes_missing_ruby_before_rest_and_returns_only_offered_key_action() {
    let mut value = decision(
        RoomViewKind::Campfire,
        vec![choice(11, RoomChoiceKind::Rest { heal: 0 }, None), choice(12, RoomChoiceKind::Recall, None)],
    );
    value.candidates = vec![choose_candidate(11), choose_candidate(12)];
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(12)));

    value.candidates.pop();
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(11)), "a disabled key row falls through to the current legal room policy");
}

#[test]
fn m20_caller_neow_selects_only_the_offered_max_hp_option() {
    let (seed, scenario) = FIXED_CASES[0];
    let run = m20_support::make_run(seed, scenario).unwrap();
    let decision = run.decision().unwrap();
    let action = policy(&decision, None, &Caller::new_m20(1).unwrap()).unwrap();
    let candidate = decision.candidates.iter().find(|candidate| candidate.action == action).unwrap();
    assert_eq!(candidate.neow_offer.as_ref().unwrap().reward, spire_core::run::NeowReward::MaxHp);
}

#[test]
fn m20_caller_public_policy_uses_shared_route_action_for_a_public_act_map() {
    let mut value = decision(RoomViewKind::Map, vec![]);
    value.candidates = vec![candidate(1, GameAction::Move(NodeId(1)))];
    let map = spire_core::run::map::PublicMap {
        boss: None, act: 1,
        nodes: vec![
            spire_core::run::map::PublicMapNode { id: NodeId(1), floor: 1, x: 0, y: 1, kind: MapNodeKind::Monster, resolved_room: None, edges: vec![NodeId(2)], visited: false, revealed: true, burning: None },
            spire_core::run::map::PublicMapNode { id: NodeId(2), floor: 2, x: 0, y: 2, kind: MapNodeKind::Elite, resolved_room: None, edges: vec![NodeId(3)], visited: false, revealed: true, burning: None },
            spire_core::run::map::PublicMapNode { id: NodeId(3), floor: 3, x: 0, y: 3, kind: MapNodeKind::Boss, resolved_room: None, edges: vec![], visited: false, revealed: true, burning: None },
        ],
        current: None, winged_moves: 0,
    };
    assert_eq!(policy(&value, Some(&map), &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Move(NodeId(1)));
}

#[test]
fn m20_caller_combat_policy_uses_public_offer_priority_and_only_legal_candidates() {
    let mut combat = empty_combat();
    combat.card_offer = Some(CombatCardOfferView { cards: vec![CardId::BallLightning, CardId::Electrodynamics], optional: false });
    let value = combat_decision(combat, vec![
        candidate(1, GameAction::Combat(CombatAction::ChooseCardOffer { index: 0 })),
        candidate(2, GameAction::Combat(CombatAction::ChooseCardOffer { index: 1 })),
        candidate(3, GameAction::Combat(CombatAction::SkipCardOffer)),
    ]);
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Combat(CombatAction::ChooseCardOffer { index: 1 }));

    let mut combat = empty_combat();
    combat.hand = vec![live_card(CardId::Electrodynamics, 10), live_card(CardId::Defragment, 11)];
    let value = combat_decision(combat, vec![
        candidate(4, GameAction::Combat(CombatAction::PlayCard { instance: 11, target: None })),
        candidate(5, GameAction::Combat(CombatAction::EndTurn)),
    ]);
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Combat(CombatAction::PlayCard { instance: 11, target: None }));

    let mut combat = empty_combat();
    combat.card_offer = Some(CombatCardOfferView { cards: vec![], optional: false });
    let value = combat_decision(combat, vec![
        candidate(9, GameAction::Combat(CombatAction::SkipCardOffer)),
        candidate(10, GameAction::Combat(CombatAction::EndTurn)),
    ]);
    assert!(policy(&value, None, &Caller::new_m20(1).unwrap()).is_err(), "mandatory offers cannot fall through to Skip");
}

#[test]
fn m20_caller_combat_policy_prefers_the_public_survival_order_only_under_published_pressure() {
    let mut combat = empty_combat();
    combat.player.block = 4;
    combat.enemies.push(FullEnemyView {
        instance_id: EnemyHandle::parse("enemy-1").unwrap(), enemy_id: EnemyId::Cultist,
        hp: 30, max_hp: 30, block: 0, powers: vec![], intent: FullIntentId::Attack,
        intent_damage: Some(5), replaced: false, escaped: false, half_dead: false, logical_slot: 0,
    });
    combat.targets.push((TargetId(1), true));
    combat.hand = vec![live_card(CardId::ColdSnap, 20), live_card(CardId::DefendBlue, 21)];
    let value = combat_decision(combat, vec![
        candidate(6, GameAction::Combat(CombatAction::PlayCard { instance: 20, target: None })),
        candidate(7, GameAction::Combat(CombatAction::PlayCard { instance: 21, target: None })),
        candidate(8, GameAction::Combat(CombatAction::EndTurn)),
    ]);
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Combat(CombatAction::PlayCard { instance: 21, target: None }));
}

#[test]
fn m20_caller_room_policy_prefers_affordable_unowned_relic_then_priority_card_without_purging() {
    let mut value = decision(
        RoomViewKind::Shop,
        vec![
            choice(31, RoomChoiceKind::Purge, Some(50)),
            choice(32, RoomChoiceKind::Card { id: CardId::BallLightning, upgrades: 0 }, Some(10)),
            choice(33, RoomChoiceKind::Relic(spire_core::content::RelicId::DataDisk), Some(90)),
            choice(34, RoomChoiceKind::Leave, None),
        ],
    );
    value.candidates = vec![choose_candidate(31), choose_candidate(32), choose_candidate(33), choose_candidate(34)];
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(33)));

    value.player.gold = 20;
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(32)));

    let mut reward = decision(
        RoomViewKind::CardReward,
        vec![
            choice(35, RoomChoiceKind::Card { id: CardId::BallLightning, upgrades: 0 }, None),
            choice(36, RoomChoiceKind::Card { id: CardId::Defragment, upgrades: 0 }, None),
            choice(37, RoomChoiceKind::Skip, None),
        ],
    );
    reward.candidates = vec![choose_candidate(35), choose_candidate(36), choose_candidate(37)];
    assert_eq!(policy(&reward, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(36)));
}

#[test]
fn m20_caller_mandatory_upgrade_selection_uses_zap_before_presentation_order() {
    let zap = PublicObject { instance: 60, id: CardId::Zap, upgrades: 0, misc: 0 };
    let defend = PublicObject { instance: 61, id: CardId::DefendBlue, upgrades: 0, misc: 0 };
    let mut value = decision(RoomViewKind::Campfire, vec![]);
    value.selection = Some(PublicSelection { required: 1, selected_count: 0, eligible: vec![zap.clone(), defend.clone()] });
    value.candidates = vec![
        Candidate { object: Some(defend), ..candidate(62, GameAction::Select(SelectionId(62))) },
        Candidate { object: Some(zap), ..candidate(63, GameAction::Select(SelectionId(63))) },
    ];
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Select(SelectionId(63)));

    value.selection.as_mut().unwrap().selected_count = 1;
    value.candidates.insert(0, candidate(64, GameAction::Confirm));
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Confirm);
}

#[test]
fn m20_caller_reward_and_campfire_actions_follow_public_priority_and_offered_order() {
    let mut rewards = decision(
        RoomViewKind::Rewards,
        vec![
            choice(90, RoomChoiceKind::Potion(spire_core::content::PotionId::FirePotion), None),
            choice(91, RoomChoiceKind::CardReward, None),
            choice(92, RoomChoiceKind::Gold(20), None),
            choice(93, RoomChoiceKind::Relic(spire_core::content::RelicId::DataDisk), None),
            choice(94, RoomChoiceKind::Proceed, None),
        ],
    );
    rewards.candidates = vec![choose_candidate(90), choose_candidate(91), choose_candidate(92), choose_candidate(93), choose_candidate(94)];
    assert_eq!(policy(&rewards, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(92)), "Gold and unowned relic share the first reward tier and retain candidate order");

    let mut campfire = decision(
        RoomViewKind::Campfire,
        vec![choice(95, RoomChoiceKind::Smith, None), choice(96, RoomChoiceKind::Rest { heal: 0 }, None), choice(97, RoomChoiceKind::Proceed, None)],
    );
    campfire.player.hp = campfire.player.max_hp;
    campfire.candidates = vec![choose_candidate(95), choose_candidate(96), choose_candidate(97)];
    assert_eq!(policy(&campfire, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Choose(ChoiceId(96)), "an offered Rest is preferred even at full HP");
}

#[test]
fn m20_caller_potion_uses_lowest_visible_hp_then_accepted_trace_once_per_encounter() {
    let mut combat = empty_combat();
    combat.enemies = vec![enemy(25, "enemy-1"), enemy(6, "enemy-2")];
    combat.targets = vec![(TargetId(1), true), (TargetId(2), true)];
    let value = combat_decision(combat, vec![
        candidate(70, GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) })),
        candidate(71, GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(2)) })),
        candidate(72, GameAction::Combat(CombatAction::EndTurn)),
    ]);
    let mut caller = Caller::new_m20(1).unwrap();
    assert_eq!(policy(&value, None, &caller).unwrap(), GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(2)) }));

    caller.trace.push(m20_support::TraceStep {
        decision: value.clone(), map: None,
        action: GameAction::Combat(CombatAction::EndTurn),
        transition: RunTransition {
            reward: 0, terminal: None,
            committed_events: vec![CommittedEvent::CombatEntered { visit: 1, encounter: EncounterId::Guardian }],
        },
    });
    caller.trace.push(m20_support::TraceStep {
        decision: value.clone(), map: None,
        action: GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(2)) }),
        transition: RunTransition { reward: 0, terminal: None, committed_events: vec![] },
    });
    assert_eq!(policy(&value, None, &caller).unwrap(), GameAction::Combat(CombatAction::EndTurn), "the accepted trace records the encounter's potion use");
}

#[test]
fn m20_caller_target_ties_follow_visible_hp_before_public_candidate_order() {
    let mut combat = empty_combat();
    combat.enemies = vec![enemy(30, "enemy-1"), enemy(5, "enemy-2")];
    combat.targets = vec![(TargetId(1), true), (TargetId(2), true)];
    combat.hand = vec![live_card(CardId::BallLightning, 80)];
    let value = combat_decision(combat, vec![
        candidate(81, GameAction::Combat(CombatAction::PlayCard { instance: 80, target: Some(TargetId(1)) })),
        candidate(82, GameAction::Combat(CombatAction::PlayCard { instance: 80, target: Some(TargetId(2)) })),
        candidate(83, GameAction::Combat(CombatAction::EndTurn)),
    ]);
    assert_eq!(policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(), GameAction::Combat(CombatAction::PlayCard { instance: 80, target: Some(TargetId(2)) }));
}

#[test]
fn m20_caller_contract_output_is_static_and_receipt_tests_do_not_run_gameplay() {
    let contract = m20_support::contract_text();
    assert!(contract.contains("m20-public-caller-v1"));
    assert!(contract.contains("m20-defect-heart-caller-attempt-v1"));
    assert!(contract.contains("SPIRE_M20_TRACE_DIR"));
    assert!(contract.contains("SPIRE_M20_FREEZE_SHA256"));
    assert!(contract.contains("action_limit_max=8192"));
    assert!(contract.contains("Guardian"));
}

#[test]
fn m20_caller_initial_public_hash_and_normal_provenance_are_bound_before_rollout() {
    let (seed, scenario) = FIXED_CASES[0];
    let run = m20_support::make_run(seed, scenario).unwrap();
    let hash = m20_support::initial_public_sha256(&run).unwrap();
    let provenance = m20_support::normal_provenance(&run).unwrap();
    assert!(m20_support::is_sha256(&hash));
    assert_eq!(provenance.kind, "normal");
    assert_eq!(provenance.loadout, None);
    assert_eq!(provenance.initial_manifest_sha256, None);
    assert_eq!(provenance.initial_public_sha256, hash);
    let supplied = m20_support::supplied_provenance(
        &run, m20_support::SuppliedLoadout::Defect, &"A".repeat(64),
    ).unwrap();
    assert_eq!(supplied.kind, "supplied");
    assert_eq!(supplied.loadout, Some("Defect"));
    let lower_manifest = "a".repeat(64);
    assert_eq!(supplied.initial_manifest_sha256.as_deref(), Some(lower_manifest.as_str()));
    assert!(m20_support::supplied_provenance(&run, m20_support::SuppliedLoadout::Strong, "bad").is_err());
}

#[test]
fn m20_caller_partial_receipt_preserves_compact_public_rows_and_refuses_overwrite() {
    use std::{fs, sync::atomic::{AtomicUsize, Ordering}};

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "m20-caller-receipt-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&directory).unwrap();

    let (seed, scenario) = FIXED_CASES[0];
    let run = m20_support::make_run(seed, scenario).unwrap();
    let initial = run.decision().unwrap();
    let action = initial.candidates[0].action.clone();
    let mut caller = Caller::new_m20(1).unwrap();
    caller.final_decision = Some(initial.clone());
    caller.final_map = run.public_map();
    caller.trace.push(m20_support::TraceStep {
        decision: initial.clone(), map: run.public_map(), action: action.clone(),
        transition: RunTransition {
            reward: 3, terminal: None,
            committed_events: vec![CommittedEvent::KeyAcquired(spire_core::run::Key::Ruby)],
        },
    });
    let result = Err("synthetic accepted-prefix fault for receipt-shape check".to_owned());
    let provenance = m20_support::normal_provenance(&run).unwrap();
    let path = m20_support::persist_attempt(
        &directory, seed, scenario, 1, &result, &caller, Some(&run), &provenance,
    ).unwrap();
    let bytes = fs::read(&path).unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    let mut top_keys = receipt.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    top_keys.sort_unstable();
    assert_eq!(top_keys, vec![
        "accepted_action_count", "action_budget", "caller_revision", "endpoint", "error",
        "failed_action", "outcome", "provenance", "scenario", "schema_id", "seed", "steps",
    ]);
    assert_eq!(receipt["schema_id"], "m20-defect-heart-caller-attempt-v1");
    assert_eq!(receipt["caller_revision"], "m20-public-caller-v1");
    assert_eq!(receipt["provenance"]["kind"], "normal");
    assert_eq!(receipt["provenance"]["initial_public_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(receipt["scenario"], serde_json::json!({
        "act1_boss": scenario.act1_boss.as_str(),
        "act2_boss": scenario.act2_boss.as_str(),
        "act3_boss": scenario.act3_boss.as_str(),
    }));
    assert_eq!(receipt["accepted_action_count"], 1);
    assert_eq!(receipt["steps"][0]["candidate_index"], initial.candidates.iter().position(|row| row.action == action).unwrap());
    assert_eq!(receipt["steps"][0]["action_debug"], format!("{action:?}"));
    assert_eq!(receipt["steps"][0]["pre"]["decision_debug"], format!("{initial:?}"));
    assert_eq!(receipt["steps"][0]["pre"]["map_debug"], "None");
    assert_eq!(receipt["steps"][0]["transition"]["reward"], 3);
    assert_eq!(receipt["steps"][0]["transition"]["terminal"], "None");
    assert_eq!(receipt["steps"][0]["transition"]["committed_events_debug"][0], "KeyAcquired(Ruby)");
    let mut step_keys = receipt["steps"][0].as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
    step_keys.sort_unstable();
    assert_eq!(step_keys, vec!["action_debug", "candidate_index", "index", "post", "pre", "transition"]);
    assert_eq!(receipt["endpoint"]["map_debug"], "None");
    assert!(receipt["steps"][0]["pre"]["decision_debug"].as_str().unwrap().lines().count() == 1, "Debug rows remain compact");
    assert_eq!(m20_support::preflight_attempt_files(&directory, &[scenario], seed).is_err(), true);
    assert!(m20_support::persist_attempt(
        &directory, seed, scenario, 1, &result, &caller, Some(&run), &provenance,
    ).is_err(), "create_new keeps an existing receipt immutable");
    assert_eq!(fs::read(path).unwrap(), bytes);

    let (_, failed_scenario) = FIXED_CASES[1];
    m20_support::preflight_attempt_files(&directory, &[failed_scenario], seed).unwrap();
    let mut failed_run = m20_support::make_run(seed, failed_scenario).unwrap();
    let mut failed_caller = Caller::new_m20(1).unwrap();
    let failed_result = m20_support::shared::rollout_m20(&mut failed_run, &mut failed_caller, illegal_public_action);
    assert!(failed_result.is_err());
    assert_eq!(failed_caller.accepted(), 0, "rejected actions do not enter the accepted trace");
    assert_eq!(failed_caller.failed_action_for_diagnostics(), Some(&GameAction::Cancel));
    assert!(failed_caller.failed_error_for_diagnostics().is_none(), "policy-menu rejection has no fabricated RunError");
    let failed_provenance = m20_support::normal_provenance(&failed_run).unwrap();
    let failed_path = m20_support::persist_attempt(
        &directory, seed, failed_scenario, 1, &failed_result, &failed_caller, Some(&failed_run), &failed_provenance,
    ).unwrap();
    let failed: serde_json::Value = serde_json::from_slice(&fs::read(failed_path).unwrap()).unwrap();
    assert_eq!(failed["failed_action"]["action_debug"], "Cancel");
    assert!(failed["failed_action"]["candidate_index"].is_null());
    assert_eq!(failed["failed_action"]["pre_map_debug"], "None");
    assert_eq!(failed["failed_action"]["error"], failed["error"]);
    assert_eq!(failed["accepted_action_count"], 0);
}

#[test]
fn m20_caller_v4_act4_map_uses_first_legal_move() {
    let mut value = decision(RoomViewKind::Map, vec![]);
    value.act = 4;
    let current = NodeId(401);
    let next = NodeId(402);
    let map = spire_core::run::map::PublicMap {
        boss: None,
        act: 4,
        nodes: vec![
            spire_core::run::map::PublicMapNode {
                id: current, floor: 50, x: 3, y: 0, kind: MapNodeKind::Rest,
                resolved_room: None, edges: vec![next], visited: true, revealed: true, burning: None,
            },
            spire_core::run::map::PublicMapNode {
                id: next, floor: 51, x: 3, y: 1, kind: MapNodeKind::Boss,
                resolved_room: None, edges: vec![], visited: false, revealed: true, burning: None,
            },
        ],
        current: Some(current),
        winged_moves: 0,
    };
    value.candidates = vec![candidate(402, GameAction::Move(next))];

    assert_eq!(
        policy(&value, Some(&map), &Caller::new_m20(1).unwrap()).unwrap(),
        GameAction::Move(next),
    );
}

#[test]
fn m20_caller_v4_potion_prefers_first_legal_slot_before_target_hp() {
    let mut combat = empty_combat();
    combat.enemies = vec![enemy(25, "enemy-1"), enemy(6, "enemy-2")];
    combat.targets = vec![(TargetId(1), true), (TargetId(2), true)];
    let value = combat_decision(combat, vec![
        candidate(410, GameAction::Combat(CombatAction::UsePotion { slot: 0, target: None })),
        candidate(411, GameAction::Combat(CombatAction::UsePotion { slot: 1, target: Some(TargetId(2)) })),
        candidate(412, GameAction::Combat(CombatAction::EndTurn)),
    ]);

    assert_eq!(
        policy(&value, None, &Caller::new_m20(1).unwrap()).unwrap(),
        GameAction::Combat(CombatAction::UsePotion { slot: 0, target: None }),
    );
}

#[test]
fn m20_caller_v4_unlisted_removal_cards_follow_public_instance_row_order() {
    let doom = PublicObject { instance: 420, id: CardId::DoomAndGloom, upgrades: 0, misc: 0 };
    let ball = PublicObject { instance: 421, id: CardId::BallLightning, upgrades: 0, misc: 0 };
    let mut removal = decision(RoomViewKind::Event(spire_core::content::EventId::Purifier), vec![]);
    removal.selection = Some(PublicSelection {
        required: 1, selected_count: 0, eligible: vec![doom.clone(), ball.clone()],
    });
    removal.candidates = vec![
        Candidate { object: Some(doom), ..candidate(422, GameAction::Select(SelectionId(422))) },
        Candidate { object: Some(ball), ..candidate(423, GameAction::Select(SelectionId(423))) },
    ];
    assert_eq!(
        policy(&removal, None, &Caller::new_m20(1).unwrap()).unwrap(),
        GameAction::Select(SelectionId(422)),
        "removal follows the actual public row when catalog kind order differs",
    );
}

#[test]
fn m20_caller_v4_unlisted_upgrade_cards_follow_public_instance_row_order() {
    let thunder = PublicObject { instance: 424, id: CardId::ThunderStrike, upgrades: 0, misc: 0 };
    let ball = PublicObject { instance: 425, id: CardId::BallLightning, upgrades: 0, misc: 0 };
    let mut upgrade = decision(RoomViewKind::Campfire, vec![]);
    upgrade.selection = Some(PublicSelection {
        required: 1, selected_count: 0, eligible: vec![thunder.clone(), ball.clone()],
    });
    upgrade.candidates = vec![
        Candidate { object: Some(thunder), ..candidate(426, GameAction::Select(SelectionId(426))) },
        Candidate { object: Some(ball), ..candidate(427, GameAction::Select(SelectionId(427))) },
    ];
    assert_eq!(
        policy(&upgrade, None, &Caller::new_m20(1).unwrap()).unwrap(),
        GameAction::Select(SelectionId(426)),
        "upgrade follows the actual public row when catalog kind order differs",
    );
}

fn v4_receipt_with_prefailure_view_and_no_live_run() -> (std::path::PathBuf, serde_json::Value, String) {
    use std::{fs, sync::atomic::{AtomicUsize, Ordering}};

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "m20-caller-v4-endpoint-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&directory).unwrap();

    let (seed, scenario) = FIXED_CASES[0];
    let mut run = m20_support::make_run(seed, scenario).unwrap();
    let provenance = m20_support::normal_provenance(&run).unwrap();
    let mut caller = Caller::new_m20(1).unwrap();
    let failed_result = m20_support::shared::rollout_m20(&mut run, &mut caller, illegal_public_action);
    assert!(failed_result.is_err());
    let prefailure_debug = format!("{:?}", caller.final_decision.clone().expect("failed action retains its public pre-view"));

    let trace_decision = combat_decision(
        empty_combat(),
        vec![candidate(430, GameAction::Combat(CombatAction::EndTurn))],
    );
    caller.trace.push(m20_support::TraceStep {
        decision: trace_decision.clone(),
        map: None,
        action: GameAction::Combat(CombatAction::EndTurn),
        transition: RunTransition { reward: 0, terminal: None, committed_events: vec![] },
    });
    let path = m20_support::persist_attempt(
        &directory, seed, scenario, 1, &failed_result, &caller, None, &provenance,
    ).unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();

    (directory, receipt, prefailure_debug)
}

#[test]
fn m20_caller_v4_unavailable_endpoint_does_not_reuse_prefailure_view() {
    use std::fs;

    let (directory, receipt, prefailure_debug) = v4_receipt_with_prefailure_view_and_no_live_run();
    assert_eq!(receipt["failed_action"]["pre_decision_debug"], prefailure_debug);
    assert!(receipt["endpoint"]["decision_debug"].is_null());
    assert!(receipt["endpoint"]["map_debug"].is_null());
    assert_eq!(receipt["endpoint"]["decision_error"], "no Run was created");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn m20_caller_v5_accepted_trace_post_keeps_pre_failure_state_separate_from_endpoint() {
    use std::fs;

    let (directory, receipt, prefailure_debug) = v4_receipt_with_prefailure_view_and_no_live_run();
    assert_eq!(receipt["steps"][0]["post"]["decision_debug"],prefailure_debug);
    assert_eq!(receipt["steps"][0]["post"]["map_debug"],"None");
    assert!(receipt["steps"][0]["post"]["decision_error"].is_null());
    assert!(receipt["endpoint"]["decision_debug"].is_null());

    fs::remove_dir_all(directory).unwrap();
}
#[test]
fn m20_caller_first_potion_slot_follows_public_candidate_order(){
    let mut combat=empty_combat();combat.enemies=vec![enemy(20,"enemy-1")];combat.targets=vec![(TargetId(1),true)];
    let d=combat_decision(combat,vec![candidate(900,GameAction::Combat(CombatAction::UsePotion{slot:2,target:None})),candidate(901,GameAction::Combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(1))})),candidate(902,GameAction::Combat(CombatAction::EndTurn))]);
    assert_eq!(policy(&d,None,&Caller::new_m20(1).unwrap()).unwrap(),GameAction::Combat(CombatAction::UsePotion{slot:2,target:None}),"first legal potion row determines the slot before its target ties");
}
