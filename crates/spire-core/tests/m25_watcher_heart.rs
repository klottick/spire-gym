//! M25 public caller policy and input contract; no canonical Run is rolled out.
#[path = "../examples/m25_support/mod.rs"]
mod m25_support;

#[path = "../examples/m24_support/policy.rs"]
mod m24_policy;

use m25_support::{policy, *};
use spire_core::{
    combat::{
        self, CombatAction, CombatPublic, CharacterCombatView, FullPlayerView, SelectionOperation,
        Stance,
    },
    content::{CardId, Character, EncounterId, RelicId},
    run::{
        Candidate, ChoiceId, CommittedEvent, CombatProgress, GameAction, MapNodeKind, NodeId,
        PublicDecision, PublicPlayer, RoomChoiceKind,
        RoomChoiceStatus, RoomChoiceView, RoomView, RoomViewKind, RoomScenario, RunTransition,
        map::{PublicMap, PublicMapNode},
    },
};

fn candidate(id: u64, action: GameAction) -> Candidate {
    Candidate {
        id: ChoiceId(id),
        action,
        label: "fabricated public candidate",
        neow_offer: None,
        object: None,
    }
}

fn decision(actions: Vec<GameAction>) -> PublicDecision {
    PublicDecision {
        content_profile: Some(RoomScenario::M25WatcherHeart),
        room: None,
        reward_total: 0,
        act: 1,
        floor: 0,
        player: PublicPlayer {
            relic_counters: vec![],
            character: Character::Watcher,
            hp: 72,
            max_hp: 72,
            gold: 99,
            relics: vec![RelicId::PureWater],
            potions: vec![None; 3],
            keys: [false; 3],
        },
        combat: None,
        combat_progress: CombatProgress::default(),
        last_combat: None,
        objects: vec![],
        candidates: actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| candidate(index as u64, action))
            .collect(),
        offers: vec![],
        selection: None,
        terminal: None,
    }
}

fn room(
    kind: RoomViewKind,
    rows: Vec<(RoomChoiceKind, Option<u32>, RoomChoiceStatus)>,
) -> PublicDecision {
    let choices = rows
        .into_iter()
        .enumerate()
        .map(|(index, (kind, price, status))| RoomChoiceView {
            id: ChoiceId(index as u64),
            kind,
            price,
            status,
        })
        .collect::<Vec<_>>();
    let mut decision = decision(
        choices
            .iter()
            .filter(|choice| choice.status == RoomChoiceStatus::Offered)
            .map(|choice| GameAction::Choose(choice.id))
            .collect(),
    );
    decision.room = Some(RoomView { kind, choices });
    decision
}

fn offered(kind: RoomViewKind, rows: Vec<(RoomChoiceKind, Option<u32>)>) -> PublicDecision {
    room(
        kind,
        rows.into_iter()
            .map(|(choice, price)| (choice, price, RoomChoiceStatus::Offered))
            .collect(),
    )
}

fn combat_decision(actions: Vec<CombatAction>) -> PublicDecision {
    let mut decision = decision(actions.into_iter().map(GameAction::Combat).collect());
    decision.combat = Some(CombatPublic {
        mantra_gained: Some(0), previous_card_type: None,
        stance_choice: None,
        played_powers: vec![],
        playing_cards: vec![],
        stasis: vec![],
        content_id: "fabricated public decision".into(),
        turn: 1,
        player: FullPlayerView {
            hp: 72,
            max_hp: 72,
            block: 0,
            energy: 3,
            max_energy: 3,
            powers: vec![],
        },
        enemies: vec![],
        character: CharacterCombatView {
            stance: Stance::Neutral,
            orbs: vec![],
            orb_dark_damage: vec![],
            focus: 0,
        },
        hand: vec![],
        draw_count: 0,
        discard: vec![],
        exhaust: vec![],
        outcome: None,
        phase: spire_core::combat::Phase::Decision,
        selection_resume_phase: None,
        hp_before_healing: None,
        hp_after_cleanup: None,
        targets: vec![],
        selection: None,
        known_draw: vec![],
        encounter_facing: None,
        inventory: None,
        card_offer: None,
    });
    decision
}

fn card_offer(actions: Vec<CombatAction>) -> PublicDecision {
    let mut decision = combat_decision(actions);
    decision.combat.as_mut().unwrap().card_offer = Some(combat::CombatCardOfferView {
        cards: vec![CardId::Tantrum, CardId::Wallop],
        optional: true,
    });
    decision
}

fn public_map(act: u8, nodes: Vec<PublicMapNode>, current: Option<NodeId>) -> PublicMap {
    PublicMap {
        boss: Some(EncounterId::CorruptHeart),
        act,
        nodes,
        current,
        winged_moves: 0,
    }
}

fn map_node(
    id: u64,
    floor: u32,
    kind: MapNodeKind,
    edges: Vec<NodeId>,
    burning: Option<bool>,
) -> PublicMapNode {
    PublicMapNode {
        id: NodeId(id),
        floor,
        x: 0,
        y: floor as u8,
        kind,
        resolved_room: None,
        edges,
        visited: false,
        revealed: true,
        burning,
    }
}

#[test]
fn m25_caller_keeps_m24_choices_in_acts_1_and_2() {
    let progress = PublicCallerProgress::default();
    let mut cases = vec![
        offered(
            RoomViewKind::Neow,
            vec![
                (RoomChoiceKind::NeowLament, None),
                (RoomChoiceKind::NeowMaxHp { amount: 7 }, None),
            ],
        ),
        offered(
            RoomViewKind::CardReward,
            vec![
                (RoomChoiceKind::Skip, None),
                (
                    RoomChoiceKind::Card {
                        id: CardId::JustLucky,
                        upgrades: 0,
                    },
                    None,
                ),
            ],
        ),
        offered(
            RoomViewKind::Campfire,
            vec![
                (RoomChoiceKind::Rest { heal: 21 }, None),
                (RoomChoiceKind::Smith, None),
                (RoomChoiceKind::Recall, None),
            ],
        ),
        offered(
            RoomViewKind::Treasure,
            vec![
                (RoomChoiceKind::Sapphire, None),
                (RoomChoiceKind::Relic(RelicId::Anchor), None),
                (RoomChoiceKind::Proceed, None),
            ],
        ),
        combat_decision(vec![
            CombatAction::EndTurn,
            CombatAction::PlayCard {
                instance: 8,
                target: Some(combat::TargetId(2)),
            },
        ]),
    ];
    let mut act2 = cases.clone();
    for decision in &mut act2 {
        decision.act = 2;
        decision.floor = 20;
    }
    cases.extend(act2);

    for decision in &cases {
        assert_eq!(
            policy::choose(decision, None, &progress),
            m24_policy::choose(
                decision,
                None,
                &m24_policy::PublicCallerProgress::default(),
            ),
            "M25 must keep M24's Act1/Act2 action at {:?}",
            decision.room.as_ref().map(|room| room.kind),
        );
    }
}

#[test]
fn m25_caller_key_objective_from_act3() {
    let progress = PublicCallerProgress::default();

    let mut fire = offered(
        RoomViewKind::Campfire,
        vec![
            (RoomChoiceKind::Rest { heal: 21 }, None),
            (RoomChoiceKind::Recall, None),
        ],
    );
    fire.act = 3;
    fire.player.keys = [false, true, true];
    assert_eq!(
        policy::choose(&fire, None, &progress),
        Ok(GameAction::Choose(ChoiceId(1))),
        "Act3 campfire takes Recall when Ruby is missing",
    );

    let mut chest = offered(
        RoomViewKind::Treasure,
        vec![
            (RoomChoiceKind::Relic(RelicId::Anchor), None),
            (RoomChoiceKind::Sapphire, None),
            (RoomChoiceKind::Proceed, None),
        ],
    );
    chest.act = 3;
    chest.player.keys = [true, false, true];
    assert_eq!(
        policy::choose(&chest, None, &progress),
        Ok(GameAction::Choose(ChoiceId(1))),
        "Act3 chest takes Sapphire when it is missing",
    );

    let mut map_decision = decision(vec![GameAction::Move(NodeId(1))]);
    map_decision.act = 3;
    map_decision.room = Some(RoomView {
        kind: RoomViewKind::Map,
        choices: vec![],
    });
    map_decision.player.keys = [true, true, false];
    let map = public_map(
        3,
        vec![
            map_node(1, 1, MapNodeKind::Rest, vec![NodeId(2)], None),
            map_node(2, 2, MapNodeKind::Elite, vec![NodeId(3)], Some(true)),
            map_node(3, 3, MapNodeKind::Boss, vec![], None),
        ],
        Some(NodeId(0)),
    );
    assert_eq!(
        policy::choose(&map_decision, Some(&map), &progress),
        Ok(GameAction::Move(NodeId(1))),
        "Act3 map routes toward a reachable burning Elite",
    );

    let mut stage4 = offered(
        RoomViewKind::Act3Ending,
        vec![(RoomChoiceKind::Act3Continue { stage: 4 }, None)],
    );
    stage4.act = 3;
    assert_eq!(
        policy::choose(&stage4, None, &progress),
        Ok(GameAction::Choose(ChoiceId(0))),
    );

    let mut heart_victory = offered(RoomViewKind::HeartVictory, vec![(RoomChoiceKind::Proceed, None)]);
    heart_victory.act = 4;
    assert_eq!(
        policy::choose(&heart_victory, None, &progress),
        Ok(GameAction::Choose(ChoiceId(0))),
    );
}

#[test]
fn m25_caller_handles_every_act4_and_new_selection() {
    // The shared M22 route scorer is used for each fixed Act4 Move. Its ordinary
    // encounter quota is treated as satisfied because Act4 has none.
    let caller = Caller::new_m25(8).unwrap();
    let route = [NodeId(51), NodeId(52), NodeId(53), NodeId(54)];
    let kinds = [
        MapNodeKind::Rest,
        MapNodeKind::Shop,
        MapNodeKind::Elite,
        MapNodeKind::Boss,
    ];
    let nodes = (0..4)
        .map(|index| {
            map_node(
                route[index].0,
                51 + index as u32,
                kinds[index],
                route.get(index + 1).copied().into_iter().collect(),
                None,
            )
        })
        .collect::<Vec<_>>();
    for (index, target) in route.into_iter().enumerate() {
        let mut decision = decision(vec![GameAction::Move(target)]);
        decision.act = 4;
        decision.floor = 50 + index as u32;
        decision.room = Some(RoomView {
            kind: RoomViewKind::Map,
            choices: vec![],
        });
        let mut route_caller = caller.clone();
        if index == 3 {
            route_caller.trace.push(m25_support::shared::TraceStep {
                decision: decision.clone(),
                map: None,
                action: GameAction::Move(target),
                transition: RunTransition {
                    reward: 0,
                    terminal: None,
                    committed_events: vec![CommittedEvent::CombatEntered {
                        visit: 1,
                        encounter: EncounterId::ShieldAndSpear,
                    }],
                },
            });
        }
        let current = if index > 0 {
            Some(route[index - 1])
        } else {
            None
        };
        let map = public_map(4, nodes.clone(), current);
        assert_eq!(
            m25_support::caller_policy(&decision, Some(&map), &route_caller),
            Ok(GameAction::Move(target)),
            "Act4 route edge {} must be offered by the shared path scorer",
            index + 1,
        );
    }

    // Rest 51 remains the M24 choice when Smith is disabled at full HP.
    let mut campfire = room(
        RoomViewKind::Campfire,
        vec![
            (
                RoomChoiceKind::Rest { heal: 200 },
                None,
                RoomChoiceStatus::Offered,
            ),
            (RoomChoiceKind::Smith, None, RoomChoiceStatus::Disabled),
        ],
    );
    campfire.act = 4;
    campfire.floor = 51;
    assert_eq!(
        policy::choose(&campfire, None, &PublicCallerProgress::default()),
        Ok(GameAction::Choose(ChoiceId(0))),
    );

    // The merchant buys the first affordable card, then leaves after the purchase.
    let mut shop = offered(
        RoomViewKind::Shop,
        vec![
            (
                RoomChoiceKind::Card {
                    id: CardId::Wish,
                    upgrades: 0,
                },
                Some(120),
            ),
            (
                RoomChoiceKind::Card {
                    id: CardId::Wallop,
                    upgrades: 0,
                },
                Some(80),
            ),
            (RoomChoiceKind::Leave, None),
        ],
    );
    shop.act = 4;
    shop.floor = 52;
    assert_eq!(
        policy::choose(&shop, None, &PublicCallerProgress::default()),
        Ok(GameAction::Choose(ChoiceId(1))),
    );
    let mut after_purchase = PublicCallerProgress::default();
    after_purchase.purchased_shops.insert((4, 52));
    assert_eq!(
        policy::choose(&shop, None, &after_purchase),
        Ok(GameAction::Choose(ChoiceId(2))),
    );

    let mut rewards = offered(
        RoomViewKind::Rewards,
        vec![
            (RoomChoiceKind::Gold(25), None),
            (RoomChoiceKind::CardReward, None),
            (RoomChoiceKind::Proceed, None),
        ],
    );
    rewards.act = 4;
    rewards.floor = 53;
    assert_eq!(
        policy::choose(&rewards, None, &PublicCallerProgress::default()),
        Ok(GameAction::Choose(ChoiceId(0))),
    );
    let mut card_child = offered(
        RoomViewKind::CardReward,
        vec![
            (
                RoomChoiceKind::Card {
                    id: CardId::Tantrum,
                    upgrades: 0,
                },
                None,
            ),
            (RoomChoiceKind::Skip, None),
        ],
    );
    card_child.act = 4;
    card_child.floor = 53;
    assert_eq!(
        policy::choose(&card_child, None, &PublicCallerProgress::default()),
        Ok(GameAction::Choose(ChoiceId(0))),
    );

    // Foreign Influence and Wish expose legal card-offer actions; Omniscience
    // exposes a legal card-selection action.
    for (name, decision, expected) in [
        (
            "Foreign Influence offer",
            card_offer(vec![
                CombatAction::ChooseCardOffer { index: 0 },
                CombatAction::ChooseCardOffer { index: 1 },
                CombatAction::SkipCardOffer,
            ]),
            GameAction::Combat(CombatAction::ChooseCardOffer { index: 0 }),
        ),
        (
            "Wish choice",
            card_offer(vec![
                CombatAction::ChooseCardOffer { index: 0 },
                CombatAction::ChooseCardOffer { index: 1 },
            ]),
            GameAction::Combat(CombatAction::ChooseCardOffer { index: 0 }),
        ),
        (
            "Omniscience selection",
            {
                let mut decision = combat_decision(vec![
                    CombatAction::SelectCard { instance: 9 },
                    CombatAction::ConfirmSelection,
                ]);
                decision.combat.as_mut().unwrap().selection = Some(combat::SelectionView {
                    parent: combat::SelectionParent::EndTurn { turn: 1 },
                    operation: SelectionOperation::Omniscience,
                    candidates: vec![],
                    required: 1,
                    selected: vec![],
                    disposition: combat::SelectionDisposition::Hand,
                    source_zone: combat::SelectionSourceZone::Draw,
                    minimum: 1,
                    maximum: 1,
                });
                decision
            },
            GameAction::Combat(CombatAction::SelectCard { instance: 9 }),
        ),
    ] {
        let action = policy::choose(&decision, None, &PublicCallerProgress::default())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(action, expected, "{name}");
        assert!(decision.candidates.iter().any(|candidate| candidate.action == action));
    }
}

#[cfg(feature = "m25-diagnostics")]
#[test]
fn m25_caller_new_selections_close_and_return_to_play() {
    use spire_core::run::diagnostics::{self, M25Diagnostic};
    use std::collections::BTreeSet;

    for stage in [
        M25Diagnostic::ForeignInfluenceOffer,
        M25Diagnostic::WishChoice,
        M25Diagnostic::OmniscienceChoice,
    ] {
        let mut run = diagnostics::m25_diagnostic(stage).unwrap();
        let mut caller = Caller::new_m25(4).unwrap();
        let mut seen = BTreeSet::new();
        let mut returned_to_play = false;
        for _ in 0..4 {
            let decision = run.decision().unwrap();
            let combat_view = decision.combat.as_ref().expect("constructed stage is combat");
            let modal = combat_view.card_offer.is_some() || combat_view.selection.is_some();
            if !modal
                && decision.candidates.iter().any(|candidate| {
                    matches!(
                        candidate.action,
                        GameAction::Combat(
                            CombatAction::PlayCard { .. } | CombatAction::EndTurn
                        )
                    )
                })
            {
                returned_to_play = true;
                break;
            }
            let fingerprint = format!("{decision:?}\n{:?}", run.public_map());
            assert!(seen.insert(fingerprint), "{stage:?} caller cycled in a modal stage");
            let map = run.public_map();
            let action = m25_support::caller_policy(&decision, map.as_ref(), &caller)
                .unwrap_or_else(|error| panic!("{stage:?}: {error}"));
            assert!(
                decision.candidates.iter().any(|candidate| candidate.action == action),
                "{stage:?} caller action must be a public candidate: {action:?}"
            );
            let transition = run.step(&action).unwrap_or_else(|error| {
                panic!("{stage:?}: {error:?} while applying offered action {action:?}")
            });
            caller.trace.push(m25_support::shared::TraceStep {
                decision,
                map,
                action,
                transition,
            });
        }
        assert!(
            returned_to_play,
            "{stage:?} must close and return to card play or EndTurn in at most four steps"
        );
    }
}

#[test]
fn m25_caller_input_contract_rejects_invalid_values_without_creation() {
    let parse = |values: &[&str]| parse_args(values.iter().map(|value| value.to_string()));
    assert!(parse(&["--scenario", "unknown"]).is_err());
    assert!(parse(&["--seed", "13"]).is_err());
    assert!(parse(&["--budget", "32769"]).is_err());
    assert!(parse(&["--budget", "not-a-number"]).is_err());

    let default = parse(&[]).unwrap();
    assert_eq!(
        (default.seed, default.budget, default.scenarios.len(), default.nonattacking),
        (7, 32768, 3, false)
    );
    let death = parse(&["--scenario", "death"]).unwrap();
    assert_eq!(death.scenarios, vec![Scenario::AwakenedOne]);
    assert!(death.nonattacking);
    assert!(parse(&["--seed", "11", "--scenario", "death"]).is_err());

    use EncounterId as E;
    let cases = [
        (7, Scenario::AwakenedOne, E::Guardian, E::Champ),
        (7, Scenario::TimeEater, E::Hexaghost, E::BronzeAutomaton),
        (7, Scenario::DonuAndDeca, E::SlimeBoss, E::Collector),
        (11, Scenario::DonuAndDeca, E::Guardian, E::BronzeAutomaton),
        (11, Scenario::AwakenedOne, E::Hexaghost, E::Collector),
        (11, Scenario::TimeEater, E::SlimeBoss, E::Champ),
    ];
    for (seed, scenario, act1, act2) in cases {
        assert_eq!(triple(seed, scenario, false), Ok((act1, act2, scenario.act3())));
        let run = make_run(seed, scenario, false).unwrap();
        let decision = run.decision().unwrap();
        assert_eq!(decision.content_profile, Some(RoomScenario::M25WatcherHeart));
        assert_eq!(
            (decision.player.hp, decision.player.max_hp, decision.player.gold),
            (72, 72, 99)
        );
        assert_eq!(decision.player.relics, vec![RelicId::PureWater]);
        assert_eq!(decision.player.potions, vec![None; 3]);
        assert_eq!(decision.player.keys, [false; 3]);
        let caller = Caller::new_m25(0).unwrap();
        assert_eq!(caller.accepted(), 0, "initial case must have zero accepted actions");
    }
}

#[test]
fn m25_supplied_series_arguments_are_checked_before_any_run() {
    use fixtures::{parse_series, Series};
    let args = |values: &[&str]| values.iter().map(|value| value.to_string()).collect::<Vec<_>>();
    assert_eq!(parse_series(&[]), Ok(Series::Full));
    assert_eq!(
        parse_series(&args(&["--series", "f1-corrective", "--v1-supplied", "/v1/supplied"])),
        Ok(Series::F1Corrective { v1_supplied: "/v1/supplied".into() })
    );
    for bad in [
        &["--series", "full"][..],
        &["--series", "f1-corrective"],
        &["--series", "f1-corrective", "--v1", "/v1"],
        &["--series", "f1-corrective", "--v1-supplied", "/v1", "extra"],
        &["--seed", "7"],
    ] {
        assert!(parse_series(&args(bad)).is_err(), "{bad:?}");
    }
}

#[test]
fn m25_f1_corrective_reads_retained_v1_outcomes_exactly() {
    use fixtures::{outcome_from_name, retained_outcome, V1_FREEZE_SHA256};
    use sha2::{Digest, Sha256};
    for outcome in [
        M25CallerOutcome::HeartVictory,
        M25CallerOutcome::Act3NoKeys,
        M25CallerOutcome::Death,
        M25CallerOutcome::EmptyDeck,
        M25CallerOutcome::BudgetExhausted,
    ] {
        assert_eq!(outcome_from_name(&outcome.to_string()), Some(outcome));
    }
    assert_eq!(outcome_from_name("Victory"), None);

    let receipt = |outcome: serde_json::Value, error: serde_json::Value, freeze: &str| {
        serde_json::to_vec(&serde_json::json!({
            "schema_id": RECEIPT_SCHEMA_ID,
            "source_freeze_sha256": freeze,
            "outcome": outcome,
            "error": error,
        }))
        .unwrap()
    };
    let sha = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));

    let death = receipt("Death".into(), serde_json::Value::Null, V1_FREEZE_SHA256);
    assert_eq!(retained_outcome(&death, &sha(&death)), Ok(Ok(M25CallerOutcome::Death)));
    let fault = receipt(serde_json::Value::Null, "invalid public action".into(), V1_FREEZE_SHA256);
    assert_eq!(retained_outcome(&fault, &sha(&fault)), Ok(Err("invalid public action".into())));

    assert!(retained_outcome(&death, &sha(&fault)).is_err(), "exact retained bytes only");
    let other_freeze = receipt("Death".into(), serde_json::Value::Null, &"0".repeat(64));
    assert!(retained_outcome(&other_freeze, &sha(&other_freeze)).is_err(), "freeze-v1 receipts only");
    let both = receipt("Death".into(), "x".into(), V1_FREEZE_SHA256);
    assert!(retained_outcome(&both, &sha(&both)).is_err());
}

#[test]
fn m25_g1_corrective_arguments_and_v2_retained_receipts() {
    use fixtures::{parse_series, retained_outcome_from, Series, V1_FREEZE_SHA256, V2_FREEZE_SHA256};
    use sha2::{Digest, Sha256};
    let args = |values: &[&str]| values.iter().map(|value| value.to_string()).collect::<Vec<_>>();
    assert_eq!(
        parse_series(&args(&["--series", "g1-corrective", "--v1-supplied", "/v1", "--v2-supplied", "/v2"])),
        Ok(Series::G1Corrective { v1_supplied: "/v1".into(), v2_supplied: "/v2".into() })
    );
    for bad in [
        &["--series", "g1-corrective", "--v1-supplied", "/v1"][..],
        &["--series", "g1-corrective", "--v2-supplied", "/v2", "--v1-supplied", "/v1"],
        &["--series", "g1-corrective", "--v1-supplied", "/v1", "--v2-supplied", "/v2", "extra"],
    ] {
        assert!(parse_series(&args(bad)).is_err(), "{bad:?}");
    }
    let receipt = serde_json::to_vec(&serde_json::json!({
        "schema_id": RECEIPT_SCHEMA_ID, "source_freeze_sha256": V2_FREEZE_SHA256, "outcome": "Death", "error": null,
    })).unwrap();
    let sha = format!("{:x}", Sha256::digest(&receipt));
    assert_eq!(retained_outcome_from(&receipt, &sha, V2_FREEZE_SHA256), Ok(Ok(M25CallerOutcome::Death)));
    assert!(retained_outcome_from(&receipt, &sha, V1_FREEZE_SHA256).is_err(), "the freeze must match the receipt");
}
