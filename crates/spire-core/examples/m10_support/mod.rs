//! Public-only M10 normal-start tooling. Supplied native diagnostics are separate.
#![allow(dead_code)]

#[path = "../m8_support/mod.rs"]
mod existing_combat_policy;
#[path = "../m9_support/mod.rs"]
mod previous;
pub use previous::{M9BossHistory, Step};

use spire_core::{
    combat::{CombatAction, rng::Partition},
    content::{CardId, Character, EncounterId, EventId, RelicId},
    run::{
        CreationProfile, GameAction, MapNodeKind, NodeId, PublicDecision, RoomChoiceKind,
        RoomChoiceStatus, RoomScenario, RoomViewKind, Run, RunConfig, RunRng, RunTerminal,
        map::PublicMap,
    },
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M10Completion {
    Act3NoKeys,
    Death,
    EmptyDeck,
    Act4Ready,
    HeartVictory,
    Unfinished,
}

pub struct M10Report {
    pub trace: Vec<Step>,
    pub final_decision: PublicDecision,
    pub final_map: Option<PublicMap>,
    pub completion: M10Completion,
}

pub struct CliOptions {
    pub seed: u64,
    pub budget: usize,
    pub scenario: String,
}

/// Construct the ordinary Ironclad A0 M10 Run from the frozen M9 seed adapter.
pub fn start(seed: u64, history: M9BossHistory) -> Result<Run, String> {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = vec!["Hexaghost", "SlimeBoss"];
    profile.seen_bosses[1] = vec!["Champ", "Collector"];
    profile.seen_bosses[2] = history.seen();
    Run::new_room_scenario(
        RunConfig {
            character: Character::Ironclad,
            ascension: 0,
        },
        profile,
        RunRng::new(root, Partition::Test, 0),
        RoomScenario::M10IroncladHeart,
    )
    .map_err(|error| format!("{error:?}"))
}

pub fn rollout(seed: u64, history: M9BossHistory, budget: usize) -> Result<M10Report, String> {
    rollout_run(start(seed, history)?, budget, false)
}

pub fn nonattacking(
    seed: u64,
    history: M9BossHistory,
    budget: usize,
) -> Result<M10Report, String> {
    rollout_run(start(seed, history)?, budget, true)
}

pub fn parse_args<I>(args: I) -> Result<CliOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let mut options = CliOptions {
        seed: 11,
        budget: 8192,
        scenario: "all".into(),
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let value = args.next().ok_or_else(|| format!("missing value for {arg}"))?;
        match arg.as_str() {
            "--seed" => options.seed = value.parse().map_err(|_| "seed must be u64")?,
            "--budget" => options.budget = value.parse().map_err(|_| "budget must be in 0..8192")?,
            "--scenario" => options.scenario = value,
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if options.budget > 8192 || !matches!(options.scenario.as_str(), "complete" | "death" | "all") {
        return Err("scenario complete|death|all; budget 0..8192".into());
    }
    Ok(options)
}

pub fn classify(decision: &PublicDecision) -> M10Completion {
    match decision.terminal {
        Some(RunTerminal::Act3NoKeys) => M10Completion::Act3NoKeys,
        Some(RunTerminal::Death) => M10Completion::Death,
        Some(RunTerminal::EmptyDeck) => M10Completion::EmptyDeck,
        Some(RunTerminal::HeartVictory) => M10Completion::HeartVictory,
        None if decision
            .room
            .as_ref()
            .is_some_and(|room| room.kind == RoomViewKind::Act4Ready) =>
        {
            M10Completion::Act4Ready
        }
        None => M10Completion::Unfinished,
    }
}

fn rollout_run(mut run: Run, budget: usize, nonattacking: bool) -> Result<M10Report, String> {
    if budget > 8192 {
        return Err("budget must be in 0..8192".into());
    }
    let mut trace = Vec::new();
    let mut combats = Vec::new();
    let mut act1_elite = false;
    let mut act2_event = false;
    for _ in 0..budget {
        let decision = run.decision().map_err(|error| format!("{error:?}"))?;
        if classify(&decision) != M10Completion::Unfinished {
            break;
        }
        let action = if let Some(combat) = &decision.combat {
            let legal = decision
                .candidates
                .iter()
                .filter_map(|candidate| match &candidate.action {
                    GameAction::Combat(action) => Some(action.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            GameAction::Combat(if nonattacking {
                CombatAction::EndTurn
            } else {
                existing_combat_policy::combat_action(combat, &legal)
            })
        } else if decision.selection.is_some() {
            let upgrade = matches!(
                decision.room.as_ref().map(|room| room.kind),
                Some(RoomViewKind::Campfire | RoomViewKind::Event(EventId::UpgradeShrine))
            );
            let priorities = if upgrade {
                vec![
                    CardId::Bludgeon,
                    CardId::Bash,
                    CardId::Disarm,
                    CardId::Clothesline,
                    CardId::Anger,
                    CardId::Cleave,
                    CardId::ShrugItOff,
                    CardId::IronWave,
                ]
            } else {
                vec![CardId::Strike, CardId::Defend]
            };
            priorities
                .into_iter()
                .find_map(|id| {
                    decision
                        .candidates
                        .iter()
                        .find(|candidate| candidate.object.as_ref().is_some_and(|object| object.id == id))
                        .map(|candidate| candidate.action.clone())
                })
                .or_else(|| {
                    decision
                        .candidates
                        .iter()
                        .find(|candidate| candidate.object.is_some())
                        .map(|candidate| candidate.action.clone())
                })
                .ok_or("missing public selection")?
        } else {
            choose_public_action(&run, &decision, &combats, &mut act1_elite, act2_event)?
        };
        if !decision.candidates.iter().any(|candidate| candidate.action == action) {
            return Err(format!("unoffered action {action:?}"));
        }
        let transition = run
            .step(&action)
            .map_err(|error| format!("act{} floor{} {action:?}: {error:?}", decision.act, decision.floor))?;
        let after = run.decision().map_err(|error| format!("{error:?}"))?;
        if let Some(record) = &after.last_combat {
            if combats.last().is_none_or(|(_, last): &(u8, spire_core::run::CombatRecord)| last.visit != record.visit) {
                combats.push((after.act, record.clone()));
            }
        }
        if after.act == 2 && after.room.as_ref().is_some_and(|room| matches!(room.kind, RoomViewKind::Event(_))) {
            act2_event = true;
        }
        trace.push(Step {
            action,
            reward: transition.reward,
            decision: after,
            map: run.public_map(),
        });
    }
    let final_decision = run.decision().map_err(|error| format!("{error:?}"))?;
    let completion = classify(&final_decision);
    Ok(M10Report {
        trace,
        final_decision,
        final_map: run.public_map(),
        completion,
    })
}

fn choose_public_action(
    run: &Run,
    decision: &PublicDecision,
    combats: &[(u8, spire_core::run::CombatRecord)],
    act1_elite: &mut bool,
    act2_event: bool,
) -> Result<GameAction, String> {
    let room = decision.room.as_ref().ok_or("missing room")?;
    let offered = |kind: &RoomChoiceKind| {
        room.choices
            .iter()
            .find(|choice| choice.status == RoomChoiceStatus::Offered && &choice.kind == kind)
            .map(|choice| GameAction::Choose(choice.id))
    };
    let action = match room.kind {
        RoomViewKind::Neow => offered(&RoomChoiceKind::NeowMaxHp { amount: 8 }),
        RoomViewKind::Map if decision.act == 4 => decision
            .candidates
            .iter()
            .find(|candidate| matches!(candidate.action, GameAction::Move(_)))
            .map(|candidate| candidate.action.clone()),
        RoomViewKind::Map => {
            let map = run.public_map().ok_or("missing public map")?;
            let ordinary = combats
                .iter()
                .filter(|(act, record)| {
                    *act == 2
                        && matches!(
                            record.encounter,
                            EncounterId::SphericGuardian
                                | EncounterId::ThreeByrds
                                | EncounterId::CenturionAndHealer
                                | EncounterId::Snecko
                                | EncounterId::ChosenAndByrds
                                | EncounterId::ThreeCultists
                        )
                })
                .count()
                .min(2) as u8;
            let elite = combats.iter().any(|(act, record)| {
                *act == 2
                    && matches!(record.encounter, EncounterId::GremlinLeader | EncounterId::Slavers | EncounterId::BookOfStabbing)
            });
            let mut memo = BTreeMap::new();
            decision
                .candidates
                .iter()
                .filter_map(|candidate| match candidate.action {
                    GameAction::Move(id) => path_score(&map, id, ordinary, if decision.act == 1 { *act1_elite } else { elite }, act2_event, &mut memo)
                        .map(|score| (score, id, candidate.action.clone())),
                    _ => None,
                })
                .max_by_key(|(score, id, _)| (*score, std::cmp::Reverse(*id)))
                .map(|(_, id, action)| {
                    if decision.act == 1
                        && map.nodes.iter().any(|node| node.id == id && node.kind == MapNodeKind::Elite)
                    {
                        *act1_elite = true;
                    }
                    action
                })
        }
        RoomViewKind::Rewards => room
            .choices
            .iter()
            .find(|choice| {
                choice.status == RoomChoiceStatus::Offered
                    && matches!(choice.kind, RoomChoiceKind::Gold(_) | RoomChoiceKind::Relic(_))
            })
            .map(|choice| GameAction::Choose(choice.id))
            .or_else(|| offered(&RoomChoiceKind::CardReward))
            .or_else(|| {
                room.choices
                    .iter()
                    .find(|choice| {
                        choice.status == RoomChoiceStatus::Offered
                            && matches!(choice.kind, RoomChoiceKind::Potion(_))
                    })
                    .map(|choice| GameAction::Choose(choice.id))
            })
            .or_else(|| offered(&RoomChoiceKind::Proceed)),
        RoomViewKind::CardReward => [
            CardId::Bludgeon,
            CardId::Impervious,
            CardId::Offering,
            CardId::Inflame,
            CardId::Disarm,
            CardId::Clothesline,
            CardId::Cleave,
            CardId::Anger,
            CardId::ShrugItOff,
            CardId::PommelStrike,
            CardId::TwinStrike,
            CardId::Thunderclap,
            CardId::IronWave,
        ]
        .into_iter()
        .filter(|id| {
            decision.objects.iter().filter(|card| card.id == *id).count()
                < match id {
                    CardId::ShrugItOff => 3,
                    CardId::Anger => 2,
                    _ => 1,
                }
        })
        .find_map(|id| offered(&RoomChoiceKind::Card { id, upgrades: 0 }))
        .or_else(|| offered(&RoomChoiceKind::Skip)),
        RoomViewKind::Shop => [CardId::IronWave, CardId::Clothesline]
            .into_iter()
            .filter(|id| decision.objects.iter().filter(|card| card.id == *id).count() < 2)
            .find_map(|id| offered(&RoomChoiceKind::Card { id, upgrades: 0 }))
            .or_else(|| {
                room.choices
                    .iter()
                    .find(|choice| {
                        choice.status == RoomChoiceStatus::Offered
                            && matches!(choice.kind, RoomChoiceKind::Potion(_))
                    })
                    .map(|choice| GameAction::Choose(choice.id))
            })
            .or_else(|| offered(&RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::UpgradeShrine) => {
            offered(&RoomChoiceKind::ShrineUpgrade).or_else(|| offered(&RoomChoiceKind::Leave))
        }
        RoomViewKind::Event(EventId::Purifier) => {
            offered(&RoomChoiceKind::ShrineRemove).or_else(|| offered(&RoomChoiceKind::Leave))
        }
        RoomViewKind::Event(EventId::AncientWriting) => {
            offered(&RoomChoiceKind::AncientSimplicity).or_else(|| offered(&RoomChoiceKind::Leave))
        }
        RoomViewKind::Event(EventId::Joust) => offered(&RoomChoiceKind::JoustMurderer)
            .or_else(|| offered(&RoomChoiceKind::JoustContinue))
            .or_else(|| {
                room.choices
                    .iter()
                    .find(|choice| {
                        choice.status == RoomChoiceStatus::Offered
                            && matches!(choice.kind, RoomChoiceKind::JoustResult { .. })
                    })
                    .map(|choice| GameAction::Choose(choice.id))
            })
            .or_else(|| offered(&RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::Falling) => room
            .choices
            .iter()
            .filter(|choice| choice.status == RoomChoiceStatus::Offered)
            .filter_map(|choice| match choice.kind {
                RoomChoiceKind::FallingLose { instance: Some(instance), .. } => decision
                    .objects
                    .iter()
                    .find(|card| card.instance == instance)
                    .map(|card| {
                        (
                            (
                                match card.id {
                                    CardId::Strike => 100,
                                    CardId::Defend => 90,
                                    CardId::Anger => 70,
                                    CardId::IronWave => 60,
                                    CardId::Bludgeon => -10,
                                    _ => 0,
                                },
                                std::cmp::Reverse(instance),
                            ),
                            GameAction::Choose(choice.id),
                        )
                    }),
                _ => None,
            })
            .max_by_key(|(score, _)| *score)
            .map(|(_, action)| action)
            .or_else(|| offered(&RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::TombOfLordRedMask) => offered(&RoomChoiceKind::TombGold)
            .or_else(|| offered(&RoomChoiceKind::TombOfferGold))
            .or_else(|| offered(&RoomChoiceKind::Leave)),
        RoomViewKind::Campfire => {
            if decision.player.hp + 15 >= decision.player.max_hp {
                offered(&RoomChoiceKind::Smith).or_else(|| {
                    room.choices.iter().find_map(|choice| match choice.kind {
                        RoomChoiceKind::Rest { .. } if choice.status == RoomChoiceStatus::Offered => {
                            Some(GameAction::Choose(choice.id))
                        }
                        _ => None,
                    })
                })
            } else {
                room.choices.iter().find_map(|choice| match choice.kind {
                    RoomChoiceKind::Rest { .. } if choice.status == RoomChoiceStatus::Offered => {
                        Some(GameAction::Choose(choice.id))
                    }
                    _ => None,
                })
            }
        }
        RoomViewKind::Treasure => offered(&RoomChoiceKind::OpenChest)
            .or_else(|| {
                room.choices.iter().find_map(|choice| match choice.kind {
                    RoomChoiceKind::Relic(_) if choice.status == RoomChoiceStatus::Offered => {
                        Some(GameAction::Choose(choice.id))
                    }
                    _ => None,
                })
            })
            .or_else(|| offered(&RoomChoiceKind::Proceed)),
        RoomViewKind::BossTreasure => offered(&RoomChoiceKind::OpenChest),
        RoomViewKind::BossRelicChoice => offered(&RoomChoiceKind::Relic(if decision.act == 1 {
            RelicId::SlaversCollar
        } else {
            RelicId::BlackBlood
        }))
        .or_else(|| offered(&RoomChoiceKind::Relic(RelicId::EmptyCage)))
        .or_else(|| offered(&RoomChoiceKind::Skip)),
        RoomViewKind::BossRelicChild | RoomViewKind::Act3BossComplete => {
            offered(&RoomChoiceKind::Proceed)
        }
        RoomViewKind::ProceedWithoutRelic => offered(&RoomChoiceKind::ConfirmSkip),
        RoomViewKind::Act3Ending => room
            .choices
            .iter()
            .find(|choice| {
                choice.status == RoomChoiceStatus::Offered
                    && matches!(choice.kind, RoomChoiceKind::Act3Continue { .. })
            })
            .map(|choice| GameAction::Choose(choice.id)),
        RoomViewKind::HeartVictory => offered(&RoomChoiceKind::Proceed),
        _ => None,
    };
    action.ok_or_else(|| {
        format!(
            "no public policy at act{} floor{} {:?}, HP{}",
            decision.act, decision.floor, room.kind, decision.player.hp
        )
    })
}

type RouteMemo = BTreeMap<(NodeId, u8, bool, bool), Option<i32>>;

fn path_score(
    map: &PublicMap,
    id: NodeId,
    ordinary: u8,
    elite: bool,
    event: bool,
    memo: &mut RouteMemo,
) -> Option<i32> {
    let key = (id, ordinary, elite, event);
    if let Some(score) = memo.get(&key) {
        return *score;
    }
    let node = map.nodes.iter().find(|node| node.id == id)?;
    let ordinary = if map.act == 2 && node.kind == MapNodeKind::Monster {
        (ordinary + 1).min(2)
    } else {
        ordinary
    };
    let elite = elite || node.kind == MapNodeKind::Elite;
    let event = event || map.act == 2 && node.kind == MapNodeKind::Question;
    let result = if node.kind == MapNodeKind::Boss {
        (map.act == 3 || elite && (map.act == 1 || ordinary >= 2 && event)).then_some(0)
    } else if map.act == 2 && node.floor == 26 && !(ordinary >= 2 && elite && event) {
        None
    } else {
        let weight = match node.kind {
            MapNodeKind::Rest => 40,
            MapNodeKind::Question => 22,
            MapNodeKind::Shop => 12,
            MapNodeKind::Treasure => 25,
            MapNodeKind::Elite => -25,
            _ => 8,
        };
        node.edges
            .iter()
            .filter_map(|next| path_score(map, *next, ordinary, elite, event, memo))
            .max()
            .map(|score| score + weight)
    };
    memo.insert(key, result);
    result
}
