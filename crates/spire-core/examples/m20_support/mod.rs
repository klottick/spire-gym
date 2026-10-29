//! Public-only M20 caller policy and attempt receipt adapter.
//!
//! All choices are drawn from the current public candidate menu. The shared
//! M13 support owns the rollout loop, accepted-action journal, and terminal
//! validation; this module adds only the frozen M20 policy and receipt shape.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use spire_core::{
    combat::{CombatAction, CombatPublic, TargetId},
    content::{CardId, EncounterId, EventId},
    run::{
        Candidate, CommittedEvent, GameAction, PublicDecision, RoomChoiceKind,
        RoomChoiceStatus, RoomScenario, RoomViewKind, Run, RunConfig,
    },
};

#[path = "../m13_support/mod.rs"]
#[allow(dead_code)]
pub mod shared;

#[path = "../shared/public_keys.rs"]
mod public_keys;

#[path = "../shared/public_routes.rs"]
mod public_routes;

#[allow(unused_imports)]
pub use self::shared::{Caller, M20CallerOutcome, TraceStep};

pub const CALLER_POLICY_REVISION: &str = "m20-public-caller-v1";
pub const ACTION_LIMIT_MAX: usize = 8192;
pub const RECEIPT_SCHEMA_ID: &str = "m20-defect-heart-caller-attempt-v1";
pub const TRACE_DIR_ENV: &str = "SPIRE_M20_TRACE_DIR";
pub const FREEZE_SHA256_ENV: &str = "SPIRE_M20_FREEZE_SHA256";

/// The order used by combat, rewards, generated offers, and normal acquisition.
/// It mirrors the approved M20 public policy; unsupported rows fall behind this
/// list and retain the public candidate order.
pub const CARD_PRIORITY: &[CardId] = &[
    CardId::Electrodynamics, CardId::Defragment, CardId::BiasedCognition,
    CardId::EchoForm, CardId::Capacitor, CardId::Loop, CardId::Buffer,
    CardId::SelfRepair, CardId::Heatsinks, CardId::Storm, CardId::MachineLearning,
    CardId::CreativeAI, CardId::HelloWorld, CardId::WhiteNoise,
    CardId::GeneticAlgorithm, CardId::CoreSurge, CardId::Seek, CardId::Fission,
    CardId::Fusion, CardId::Darkness, CardId::Consume, CardId::DoubleEnergy,
    CardId::AllForOne, CardId::Sunder, CardId::MeteorStrike, CardId::ThunderStrike,
    CardId::Rainbow, CardId::DoomAndGloom, CardId::Hyperbeam,
    CardId::SearingBlow, CardId::BallLightning, CardId::ColdSnap,
    CardId::Coolheaded, CardId::CompileDriver, CardId::Barrage, CardId::Bullseye,
    CardId::Claw, CardId::SteamBarrier, CardId::Streamline, CardId::Zap,
    CardId::StrikeBlue, CardId::DefendBlue,
];

const SURVIVAL_PRIORITY: &[CardId] = &[
    CardId::Glacier, CardId::ReinforcedBody, CardId::DefendBlue,
    CardId::SteamBarrier, CardId::GeneticAlgorithm, CardId::ColdSnap,
    CardId::Coolheaded,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub act1_boss: EncounterId,
    pub act2_boss: EncounterId,
    pub act3_boss: EncounterId,
}

impl Scenario {
    pub const fn label(self) -> &'static str {
        match self.act1_boss {
            EncounterId::Guardian => "guardian",
            EncounterId::Hexaghost => "hexaghost",
            EncounterId::SlimeBoss => "slime",
            _ => "invalid",
        }
    }
}

/// The six source-approved seed/boss triples, kept in their accepted order.
pub const FIXED_CASES: [(u64, Scenario); 6] = [
    (7, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::Champ, act3_boss: EncounterId::AwakenedOne }),
    (7, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::TimeEater }),
    (7, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Collector, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::Collector, act3_boss: EncounterId::AwakenedOne }),
    (11, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Champ, act3_boss: EncounterId::TimeEater }),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub seed: u64,
    pub scenarios: Vec<Scenario>,
    pub limit: usize,
}

pub fn cases_for_seed(seed: u64) -> Result<Vec<Scenario>, String> {
    if seed != 7 && seed != 11 {
        return Err("seed must be 7 or 11".into());
    }
    Ok(FIXED_CASES.iter().filter(|(case_seed, _)| *case_seed == seed).map(|(_, case)| *case).collect())
}

/// Parse the complete CLI before any Run is created. The defaults match the
/// frozen caller contract, but the executable separately requires the freeze
/// marker and retained trace directory before entering the gameplay loop.
pub fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut seed = 7;
    let mut scenario_label = "all".to_owned();
    let mut limit = ACTION_LIMIT_MAX;
    let mut seen_seed = false;
    let mut seen_scenario = false;
    let mut seen_budget = false;
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => {
                if seen_seed { return Err("--seed may be specified only once".into()); }
                seed = args.next().ok_or("missing --seed value")?.parse().map_err(|_| "seed must be integer")?;
                seen_seed = true;
            }
            "--budget" => {
                if seen_budget { return Err("--budget may be specified only once".into()); }
                limit = args.next().ok_or("missing --budget value")?.parse().map_err(|_| "budget must be integer")?;
                seen_budget = true;
            }
            "--scenario" => {
                if seen_scenario { return Err("--scenario may be specified only once".into()); }
                scenario_label = args.next().ok_or("missing --scenario value")?;
                seen_scenario = true;
            }
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    let all = cases_for_seed(seed)?;
    let scenarios = match scenario_label.as_str() {
        "all" => all,
        "guardian" | "hexaghost" | "slime" => all.into_iter().filter(|case| case.label() == scenario_label).collect(),
        _ => return Err("scenario must be all|guardian|hexaghost|slime".into()),
    };
    if limit > ACTION_LIMIT_MAX {
        return Err(format!("budget must be in 0..={ACTION_LIMIT_MAX}"));
    }
    Ok(Options { seed, scenarios, limit })
}

/// Construct the ordinary Defect A0 short-Neow M20 Run with only the three
/// approved unseen-boss history fields changed before the normal constructor.
pub fn make_run(seed: u64, scenario: Scenario) -> Result<Run, String> {
    use spire_core::{combat::rng::Partition, content::Character, run::{CreationProfile, RoomScenario, RunRng}};

    if !FIXED_CASES.contains(&(seed, scenario)) {
        return Err("seed/scenario is outside the six frozen M20 seed/boss triples".into());
    }
    let m13_scenario = shared::Scenario { act1_boss: scenario.act1_boss, act2_boss: scenario.act2_boss };
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = shared::act1_seen_bosses(m13_scenario);
    profile.seen_bosses[1] = shared::act2_seen_bosses(m13_scenario);
    profile.seen_bosses[2] = act3_seen_bosses(scenario.act3_boss)?;
    let mut root = [0u8; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    Run::new_room_scenario(
        RunConfig { character: Character::Defect, ascension: 0 },
        profile,
        RunRng::new(root, Partition::Test, 0),
        RoomScenario::M20DefectHeart,
    ).map_err(|error| format!("M20 Defect A0 creation: {error:?}"))
}

fn act3_seen_bosses(target: EncounterId) -> Result<Vec<&'static str>, String> {
    match target {
        EncounterId::AwakenedOne => Ok(vec!["TimeEater", "DonuAndDeca"]),
        EncounterId::TimeEater => Ok(vec!["AwakenedOne", "DonuAndDeca"]),
        EncounterId::DonuAndDeca => Ok(vec!["AwakenedOne", "TimeEater"]),
        _ => Err(format!("invalid M20 Act3 boss-history target: {target:?}")),
    }
}

/// Frozen M20 key objective, delegated to the shared key implementation.
pub fn key_action(decision: &PublicDecision, map: Option<&spire_core::run::map::PublicMap>) -> Option<GameAction> {
    public_keys::key_action(decision, map, RoomScenario::M20DefectHeart)
}

/// Pure caller policy over the supplied public decision, map, and accepted
/// trace. It never reads private Run state, future intents, hidden piles, or RNG.
pub fn policy(
    decision: &PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
    caller: &Caller,
) -> Result<GameAction, String> {
    if decision.content_profile != Some(RoomScenario::M20DefectHeart) {
        return Err("M20 public policy requires the M20DefectHeart profile".into());
    }
    if let Some(action) = key_action(decision, map) {
        return Ok(action);
    }
    if let Some(combat) = &decision.combat {
        return combat_policy(decision, combat, caller);
    }
    if decision.selection.is_some() {
        return room_selection_policy(decision);
    }
    room_policy(decision, map, caller)
}

fn room_policy(
    decision: &PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
    caller: &Caller,
) -> Result<GameAction, String> {
    let room = decision.room.as_ref().ok_or("missing public room")?;
    let action = match room.kind {
        RoomViewKind::Neow => neow_action(decision),
        RoomViewKind::Map => {
            let map = map.ok_or("public route decision has no public map")?;
            let (elite, ordinary) = caller.route_progress(decision.act);
            return match decision.act {
                1 | 2 => public_routes::action(decision, map, elite, ordinary),
                3 => public_routes::action_m14(decision, map, elite, ordinary),
                4 => decision.candidates.iter().find_map(|candidate| {
                    matches!(&candidate.action, GameAction::Move(_)).then(|| candidate.action.clone())
                }).ok_or_else(|| "Act 4 public map has no legal Move".to_owned()),
                _ => Err(format!("M20 public route policy has no route owner for act {}", decision.act)),
            };
        }
        RoomViewKind::Campfire => campfire_action(decision),
        RoomViewKind::Rewards => reward_action(decision),
        RoomViewKind::CardReward => card_reward_action(decision),
        RoomViewKind::Shop => shop_action(decision),
        RoomViewKind::Event(EventId::UpgradeShrine) => room_choice_action(decision, |kind| kind == &RoomChoiceKind::ShrineUpgrade)
            .or_else(|| room_choice_action(decision, |kind| kind == &RoomChoiceKind::Leave)),
        // Purifier launches a room-owned selection; if a malformed public
        // screen omits it, use the current offered row order without inventing
        // a removal.
        RoomViewKind::Event(EventId::Purifier) => first_offered_candidate(decision),
        // Falling and Tomb rows intentionally keep their published candidate
        // order; the caller adds no removal valuation or hidden card search.
        RoomViewKind::Event(EventId::Falling | EventId::TombOfLordRedMask) => first_offered_candidate(decision),
        RoomViewKind::Treasure | RoomViewKind::BossTreasure => first_offered_candidate(decision),
        RoomViewKind::ProceedWithoutRelic => room_choice_action(decision, |kind| kind == &RoomChoiceKind::ConfirmSkip)
            .or_else(|| first_offered_candidate(decision)),
        RoomViewKind::HeartVictory => room_choice_action(decision, |kind| kind == &RoomChoiceKind::Proceed),
        _ => first_offered_candidate(decision),
    };
    action.ok_or_else(|| format!("no legal M20 public action for act {} floor {} in {:?}", decision.act, decision.floor, room.kind))
}

fn neow_action(decision: &PublicDecision) -> Option<GameAction> {
    decision.candidates.iter().find(|candidate| {
        candidate.neow_offer.as_ref().is_some_and(|offer| offer.reward == spire_core::run::NeowReward::MaxHp)
    }).map(|candidate| candidate.action.clone())
        .or_else(|| room_choice_action(decision, |kind| matches!(kind, RoomChoiceKind::NeowMaxHp { .. })))
        .or_else(|| first_offered_candidate(decision))
}

fn campfire_action(decision: &PublicDecision) -> Option<GameAction> {
    room_choice_action(decision, |kind| matches!(kind, RoomChoiceKind::Rest { .. }))
        .or_else(|| room_choice_action(decision, |kind| kind == &RoomChoiceKind::Smith))
        .or_else(|| room_choice_action(decision, |kind| kind == &RoomChoiceKind::Proceed))
}

fn reward_action(decision: &PublicDecision) -> Option<GameAction> {
    best_choice(decision, |kind| match kind {
        RoomChoiceKind::Gold(_) => Some(0),
        RoomChoiceKind::Relic(id) if !decision.player.relics.contains(id) => Some(0),
        _ => None,
    })
    .or_else(|| room_choice_action(decision, |kind| kind == &RoomChoiceKind::CardReward))
    .or_else(|| best_choice(decision, |kind| matches!(kind, RoomChoiceKind::Potion(_)).then_some(0)))
    .or_else(|| room_choice_action(decision, |kind| kind == &RoomChoiceKind::Proceed))
}

fn card_reward_action(decision: &PublicDecision) -> Option<GameAction> {
    best_card_choice(decision, false)
        .or_else(|| room_choice_action(decision, |kind| matches!(kind, RoomChoiceKind::Skip | RoomChoiceKind::ConfirmSkip)))
}

fn shop_action(decision: &PublicDecision) -> Option<GameAction> {
    best_choice(decision, |kind| match kind {
        RoomChoiceKind::Relic(id) if !decision.player.relics.contains(id) => Some(0),
        _ => None,
    })
    .or_else(|| best_card_choice(decision, true))
    .or_else(|| best_choice(decision, |kind| matches!(kind, RoomChoiceKind::Potion(_)).then_some(0)))
    .or_else(|| room_choice_action(decision, |kind| kind == &RoomChoiceKind::Leave))
}

/// Return the highest-priority offered card; `shop` filters unaffordable rows.
fn best_card_choice(decision: &PublicDecision, shop: bool) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    decision.candidates.iter().enumerate().filter_map(|(index, candidate)| {
        let GameAction::Choose(id) = &candidate.action else { return None };
        let row = room.choices.iter().find(|row| row.id == *id && row.status == RoomChoiceStatus::Offered)?;
        if shop && row.price.is_some_and(|price| price > decision.player.gold) {
            return None;
        }
        let RoomChoiceKind::Card { id, .. } = &row.kind else { return None };
        Some((card_priority(*id), index, candidate.action.clone()))
    }).min_by_key(|(priority, index, _)| (*priority, *index)).map(|(_, _, action)| action)
}

fn best_choice(
    decision: &PublicDecision,
    mut rank: impl FnMut(&RoomChoiceKind) -> Option<usize>,
) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    decision.candidates.iter().enumerate().filter_map(|(index, candidate)| {
        let GameAction::Choose(id) = &candidate.action else { return None };
        let row = room.choices.iter().find(|row| row.id == *id && row.status == RoomChoiceStatus::Offered)?;
        let priority = rank(&row.kind)?;
        if row.price.is_some_and(|price| price > decision.player.gold) {
            return None;
        }
        Some((priority, index, candidate.action.clone()))
    }).min_by_key(|(priority, index, _)| (*priority, *index)).map(|(_, _, action)| action)
}

fn room_choice_action(decision: &PublicDecision, wanted: impl Fn(&RoomChoiceKind) -> bool) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    decision.candidates.iter().find_map(|candidate| {
        let GameAction::Choose(id) = &candidate.action else { return None };
        room.choices.iter().any(|row| row.id == *id && row.status == RoomChoiceStatus::Offered && wanted(&row.kind))
            .then(|| candidate.action.clone())
    })
}

fn first_offered_candidate(decision: &PublicDecision) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    decision.candidates.iter().find_map(|candidate| {
        let GameAction::Choose(id) = &candidate.action else { return None };
        room.choices.iter().any(|row| row.id == *id && row.status == RoomChoiceStatus::Offered)
            .then(|| candidate.action.clone())
    })
}

fn room_selection_policy(decision: &PublicDecision) -> Result<GameAction, String> {
    let selection = decision.selection.as_ref().ok_or("missing public room selection")?;
    if selection.selected_count >= selection.required {
        if let Some(confirm) = decision.candidates.iter().find(|candidate| candidate.action == GameAction::Confirm) {
            return Ok(confirm.action.clone());
        }
    }
    let room_kind = decision.room.as_ref().map(|room| room.kind);
    let removal = matches!(room_kind,
        Some(RoomViewKind::Event(EventId::Purifier) | RoomViewKind::BossRelicChoice | RoomViewKind::BossRelicChild));
    let upgrade = matches!(room_kind,
        Some(RoomViewKind::Campfire | RoomViewKind::Event(EventId::UpgradeShrine)));
    let eligible = |candidate: &Candidate| {
        let GameAction::Select(_) = &candidate.action else { return false };
        let Some(object) = candidate.object.as_ref() else { return false };
        selection.eligible.iter().any(|row| row == object)
    };
    let chosen = decision.candidates.iter().filter(|candidate| eligible(candidate)).min_by_key(|candidate| {
        let card = candidate.object.as_ref().expect("eligible candidate has object");
        let rank = if removal {
            removal_rank(card.id)
        } else if upgrade {
            upgrade_rank(card.id)
        } else {
            usize::MAX
        };
        (rank, candidate_position(decision, candidate))
    });
    chosen.map(|candidate| candidate.action.clone())
        .ok_or_else(|| "public room selection has no eligible offered row".into())
}

fn candidate_position(decision: &PublicDecision, wanted: &Candidate) -> usize {
    decision.candidates.iter().position(|candidate| candidate == wanted).unwrap_or(usize::MAX)
}

fn removal_rank(id: CardId) -> usize {
    match id {
        CardId::StrikeBlue => 0,
        CardId::DefendBlue => 1,
        _ => 2,
    }
}

fn upgrade_rank(id: CardId) -> usize {
    match id {
        CardId::Zap => 0,
        CardId::Dualcast => 1,
        _ => 2,
    }
}

fn card_priority(id: CardId) -> usize {
    CARD_PRIORITY.iter().position(|candidate| *candidate == id).unwrap_or(usize::MAX / 2)
}

fn combat_policy(decision: &PublicDecision, combat: &CombatPublic, caller: &Caller) -> Result<GameAction, String> {
    let candidates = decision.candidates.iter().enumerate().filter_map(|(index, candidate)| {
        match &candidate.action {
            GameAction::Combat(action) => Some((index, action, candidate.action.clone())),
            _ => None,
        }
    }).collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err("combat has no public legal action".into());
    }

    if let Some(offer) = combat.card_offer.as_ref() {
        if let Some((_, _, action)) = candidates.iter().filter_map(|(candidate_index, action, full)| {
            let CombatAction::ChooseCardOffer { index } = action else { return None };
            let card = offer.cards.get(*index as usize)?;
            Some((card_priority(*card), *candidate_index, full.clone()))
        }).min_by_key(|(rank, index, _)| (*rank, *index)) {
            return Ok(action);
        }
        if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| **action == CombatAction::SkipCardOffer) {
            if offer.optional {
                return Ok(action.clone());
            }
        }
        if !offer.optional {
            return Err("mandatory public card offer has no offered legal row".into());
        }
    } else if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| matches!(action, CombatAction::ChooseCardOffer { .. })) {
        // The current legal offer row is still preferred over skipping when a
        // deliberately minimal public fixture omits the descriptive view.
        return Ok(action.clone());
    }

    if let Some(selection) = &combat.selection {
        if selection.selected.len() >= selection.required.max(selection.minimum) {
            if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| **action == CombatAction::ConfirmSelection) {
                return Ok(action.clone());
            }
        }
        if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| matches!(action, CombatAction::SelectCard { .. })) {
            return Ok(action.clone());
        }
        return Err("combat selection has no offered row or confirmation".into());
    }
    if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| **action == CombatAction::ConfirmSelection) {
        return Ok(action.clone());
    }

    if !potion_already_used(caller) {
        if let Some(action) = best_potion_candidate(&candidates, combat) {
            return Ok(action);
        }
    }

    let incoming = combat.enemies.iter().enumerate()
        .filter(|(index, _)| targetable(combat, *index))
        .filter_map(|(_, enemy)| enemy.intent_damage)
        .map(|damage| damage.max(0))
        .sum::<i32>();
    if incoming > combat.player.block {
        if let Some(action) = prioritized_play(combat, &candidates, SURVIVAL_PRIORITY) {
            return Ok(action);
        }
    }
    if let Some(action) = prioritized_play(combat, &candidates, CARD_PRIORITY) {
        return Ok(action);
    }

    // Unlisted but currently legal card rows stay in source candidate order.
    if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| {
        matches!(action, CombatAction::PlayCard { instance, .. }
            if combat.hand.iter().any(|card| card.instance.0 == *instance && !CARD_PRIORITY.contains(&card.id)))
    }) {
        return Ok(action.clone());
    }
        if let Some((_, _, action)) = candidates.iter().find(|(_, action, _)| {
            !matches!(**action, CombatAction::EndTurn | CombatAction::UsePotion { .. } | CombatAction::SkipCardOffer)
        }) {
        return Ok(action.clone());
    }
    candidates.iter().find(|(_, action, _)| **action == CombatAction::EndTurn)
        .map(|(_, _, action)| action.clone())
        .ok_or_else(|| "combat has no legal action after public card priorities".into())
}

fn prioritized_play(
    combat: &CombatPublic,
    candidates: &[(usize, &CombatAction, GameAction)],
    priority: &[CardId],
) -> Option<GameAction> {
    for &wanted in priority {
        let selected = candidates.iter().filter_map(|(candidate_index, action, full)| {
            let CombatAction::PlayCard { instance, target } = action else { return None };
            combat.hand.iter().find(|card| card.instance.0 == *instance && card.id == wanted)?;
            Some((visible_target_hp(combat, *target), *candidate_index, full.clone()))
        }).min_by_key(|(hp, index, _)| (*hp, *index));
        if let Some((_, _, action)) = selected {
            return Some(action);
        }
    }
    None
}

fn visible_target_hp(combat: &CombatPublic, target: Option<TargetId>) -> i32 {
    target.and_then(|target| target.0.checked_sub(1).and_then(|row| combat.enemies.get(row as usize)))
        .filter(|_| target.is_some_and(|target| targetable(combat, target.0.saturating_sub(1) as usize)))
        .map(|enemy| enemy.hp)
        .unwrap_or(i32::MAX)
}

fn targetable(combat: &CombatPublic, row: usize) -> bool {
    combat.targets.get(row).is_some_and(|(id, active)| *active && id.0 == row as u64 + 1)
}

fn best_potion_candidate(
    candidates: &[(usize, &CombatAction, GameAction)],
    combat: &CombatPublic,
) -> Option<GameAction> {
    let first_slot=candidates.iter().find_map(|(_,action,_)|match action{
        CombatAction::UsePotion{slot,..}=>Some(*slot),_=>None})?;
    candidates.iter().filter_map(|(candidate_index, action, full)| {
        let CombatAction::UsePotion { slot, target } = action else { return None };
        (*slot==first_slot).then(||(visible_target_hp(combat,*target),*candidate_index,full.clone()))
    }).min_by_key(|(hp,index,_)|(*hp,*index)).map(|(_,_,action)|action)
}

fn potion_already_used(caller: &Caller) -> bool {
    let last_encounter = caller.trace.iter().rposition(|step| step.transition.committed_events.iter()
        .any(|event| matches!(event, CommittedEvent::CombatEntered { .. })));
    caller.trace.iter().enumerate().skip(last_encounter.map_or(0, |index| index + 1)).any(|(_, step)| {
        matches!(step.action,
            GameAction::Combat(CombatAction::UsePotion { .. }) | GameAction::UsePotion { .. })
    })
}

/// Human-readable public contract for the non-gameplay `--contract` path.
pub fn contract_text() -> String {
    let cases = FIXED_CASES.iter().map(|(seed, scenario)| format!(
        "seed={seed} act1={:?} act2={:?} act3={:?}", scenario.act1_boss, scenario.act2_boss, scenario.act3_boss,
    )).collect::<Vec<_>>().join("\n");
    format!(
        "caller_policy={CALLER_POLICY_REVISION}\ncli=--seed 7|11 --scenario all|guardian|hexaghost|slime --budget 0..={ACTION_LIMIT_MAX}\naction_limit_min=0\naction_limit_max={ACTION_LIMIT_MAX}\ndefault_budget={ACTION_LIMIT_MAX}\ncase_count={}\nfixed_cases:\n{cases}\ncard_priority={:?}\nsurvival_priority={:?}\nreceipt_schema={RECEIPT_SCHEMA_ID}\ntrace_dir_env={TRACE_DIR_ENV}\nfreeze_sha256_env={FREEZE_SHA256_ENV}\n",
        FIXED_CASES.len(), CARD_PRIORITY, SURVIVAL_PRIORITY,
    )
}

pub fn endpoint_summary(run: Option<&Run>, _caller: &Caller) -> String {
    match run {
        Some(run) => match run.decision() {
            Ok(decision) => format!(
                "act={} floor={} terminal={:?} room={:?}",
                decision.act, decision.floor, decision.terminal,
                decision.room.as_ref().map(|room| room.kind),
            ),
            Err(error) => format!("unavailable: {error:?}"),
        },
        None => "unavailable: no Run".into(),
    }
}

/// The original state identity is captured before rollout. `None` is used only
/// when normal construction itself fails before any initial Run exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    pub kind: &'static str,
    pub loadout: Option<&'static str>,
    pub initial_manifest_sha256: Option<String>,
    pub initial_public_sha256: String,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppliedLoadout { Defect, Strong }

#[allow(dead_code)]
impl SuppliedLoadout {
    fn label(self) -> &'static str {
        match self { Self::Defect => "Defect", Self::Strong => "Strong" }
    }
}

pub fn normal_provenance(run: &Run) -> Result<Provenance, String> {
    Ok(Provenance {
        kind: "normal", loadout: None, initial_manifest_sha256: None,
        initial_public_sha256: initial_public_sha256(run)?,
    })
}

#[allow(dead_code)]
pub fn supplied_provenance(
    run: &Run,
    loadout: SuppliedLoadout,
    initial_manifest_sha256: &str,
) -> Result<Provenance, String> {
    if !is_sha256(initial_manifest_sha256) {
        return Err("supplied initial manifest SHA-256 must be 64 hexadecimal characters".into());
    }
    Ok(Provenance {
        kind: "supplied", loadout: Some(loadout.label()),
        initial_manifest_sha256: Some(initial_manifest_sha256.to_ascii_lowercase()),
        initial_public_sha256: initial_public_sha256(run)?,
    })
}

pub fn initial_public_sha256(run: &Run) -> Result<String, String> {
    let decision = run.decision().map_err(|error| format!("initial public decision: {error:?}"))?;
    let map = run.public_map();
    let combat_checkpoint = run.combat_checkpoint();
    let canonical = format!("{decision:?}\n{map:?}\n{combat_checkpoint:?}");
    Ok(format!("{:x}", Sha256::digest(canonical.as_bytes())))
}

pub fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Preflight every normal-case receipt filename before the first Run exists.
pub fn preflight_attempt_files(directory: &Path, scenarios: &[Scenario], seed: u64) -> Result<(), String> {
    std::fs::create_dir_all(directory).map_err(|error| format!("create M20 trace directory: {error}"))?;
    for scenario in scenarios {
        let path = attempt_path(directory, seed, *scenario, "normal");
        match std::fs::symlink_metadata(&path) {
            Ok(_) => return Err(format!("M20 attempt output already exists for seed={seed} scenario={scenario:?}; refusing overwrite")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(format!("inspect M20 attempt output {}: {error}", path.display())),
        }
    }
    Ok(())
}

/// Persist one accepted-action trace without stepping or reconstructing Run
/// state. Writes use `create_new` and sync before returning.
pub fn persist_attempt(
    directory: &Path,
    seed: u64,
    scenario: Scenario,
    limit: usize,
    result: &Result<M20CallerOutcome, String>,
    caller: &Caller,
    run: Option<&Run>,
    provenance: &Provenance,
) -> Result<PathBuf, String> {
    let path = attempt_path(directory, seed, scenario, provenance.loadout.unwrap_or("normal"));
    let receipt = receipt_value(seed, scenario, limit, result, caller, run, provenance)?;
    let bytes = serde_json::to_vec_pretty(&receipt).map_err(|error| format!("serialize M20 attempt receipt: {error}"))?;
    write_new(&path, &bytes)?;
    Ok(path)
}

fn receipt_value(
    seed: u64,
    scenario: Scenario,
    limit: usize,
    result: &Result<M20CallerOutcome, String>,
    caller: &Caller,
    run: Option<&Run>,
    provenance: &Provenance,
) -> Result<serde_json::Value, String> {
    use serde_json::json;

    let (live_decision, decision_error) = match run {
        Some(run) => match run.decision() {
            Ok(decision) => (Some(decision), None),
            Err(error) => (None, Some(format!("{error:?}"))),
        },
        None => (None, Some("no Run was created".to_owned())),
    };
    let endpoint_decision = live_decision.as_ref();
    let endpoint_map_debug = run.map(|run| format!("{:?}", run.public_map()));

    let mut steps = Vec::with_capacity(caller.trace.len());
    for (index, step) in caller.trace.iter().enumerate() {
        let candidate_index = step.decision.candidates.iter().position(|candidate| candidate.action == step.action)
            .ok_or_else(|| format!("accepted M20 trace step {index} action is absent from its public candidate list: {:?}", step.action))?;
        let (post_decision_debug, post_map_debug, post_decision_error) = if let Some(next) = caller.trace.get(index + 1) {
            (Some(format!("{:?}", next.decision)), Some(format!("{:?}", next.map)), None)
        } else if caller.failed_action_for_diagnostics().is_some() {
            // This is the post of the last ACCEPTED action, before the next
            // failed action. The independent endpoint below describes the fault.
            (caller.final_decision.as_ref().map(|decision|format!("{decision:?}")),
                Some(format!("{:?}",caller.final_map)),None)
        } else if let Some(run) = run {
            match run.decision() {
                Ok(decision) => (Some(format!("{decision:?}")), Some(format!("{:?}", run.public_map())), None),
                Err(error) => (None, Some(format!("{:?}", run.public_map())), Some(format!("{error:?}"))),
            }
        } else {
            (None, None, Some("live Run unavailable for final accepted trace step".into()))
        };
        steps.push(json!({
            "index": index,
            "candidate_index": candidate_index,
            "action_debug": format!("{:?}", step.action),
            "pre": {
                "decision_debug": format!("{:?}", step.decision),
                "map_debug": format!("{:?}", step.map),
            },
            "transition": {
                "debug": format!("{:?}", step.transition),
                "reward": step.transition.reward,
                "terminal": format!("{:?}", step.transition.terminal),
                "committed_events_debug": step.transition.committed_events.iter().map(|event| format!("{event:?}")).collect::<Vec<_>>(),
            },
            "post": {
                "decision_debug": post_decision_debug,
                "map_debug": post_map_debug,
                "decision_error": post_decision_error,
            },
        }));
    }

    let failed_action = if let Some(action) = caller.failed_action_for_diagnostics() {
        let pre = caller.final_decision.as_ref().ok_or("failed M20 action has no retained pre-action decision")?;
        let error = caller.failed_error_for_diagnostics()
            .map(|error| format!("{error:?}"))
            .or_else(|| result.as_ref().err().cloned())
            .ok_or("failed M20 action has no retained error")?;
        let candidate_index = pre.candidates.iter().position(|candidate| candidate.action == *action);
        Some(json!({
            "action_debug": format!("{action:?}"),
            "candidate_index": candidate_index,
            "pre_decision_debug": format!("{pre:?}"),
            "pre_map_debug": format!("{:?}", caller.final_map),
            "error": error,
        }))
    } else {
        None
    };

    let outcome = result.as_ref().ok().map(ToString::to_string);
    let error = result.as_ref().err();
    let endpoint = json!({
        "decision_debug": endpoint_decision.map(|decision| format!("{decision:?}")),
        "map_debug": endpoint_map_debug,
        "decision_error": decision_error,
    });

    Ok(json!({
        "schema_id": RECEIPT_SCHEMA_ID,
        "caller_revision": CALLER_POLICY_REVISION,
        "seed": seed,
        "scenario": {
            "act1_boss": scenario.act1_boss.as_str(),
            "act2_boss": scenario.act2_boss.as_str(),
            "act3_boss": scenario.act3_boss.as_str(),
        },
        "provenance": {
            "kind": provenance.kind,
            "loadout": provenance.loadout,
            "initial_manifest_sha256": provenance.initial_manifest_sha256,
            "initial_public_sha256": provenance.initial_public_sha256,
        },
        "action_budget": limit,
        "outcome": outcome,
        "error": error,
        "accepted_action_count": caller.accepted(),
        "steps": steps,
        "failed_action": failed_action,
        "endpoint": endpoint,
    }))
}

fn attempt_path(directory: &Path, seed: u64, scenario: Scenario, loadout: &str) -> PathBuf {
    directory.join(format!(
        "seed-{seed}-{:?}-{:?}-{:?}-{loadout}.attempt.json",
        scenario.act1_boss, scenario.act2_boss, scenario.act3_boss,
    ))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("create M20 receipt {}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("write M20 receipt {}: {error}", path.display()))?;
    file.sync_all().map_err(|error| format!("sync M20 receipt {}: {error}", path.display()))
}

/// Enforce the execution freeze before a CLI Run is constructed.
pub fn required_execution_environment() -> Result<(PathBuf, String), String> {
    let directory = std::env::var_os(TRACE_DIR_ENV).map(PathBuf::from)
        .ok_or("set SPIRE_M20_TRACE_DIR to a retained trace directory before gameplay")?;
    let freeze = std::env::var(FREEZE_SHA256_ENV)
        .map_err(|_| "set SPIRE_M20_FREEZE_SHA256 to the frozen source SHA-256 before gameplay")?;
    if !is_sha256(&freeze) {
        return Err("SPIRE_M20_FREEZE_SHA256 must be exactly 64 hexadecimal characters".into());
    }
    Ok((directory, freeze.to_ascii_lowercase()))
}

/// Check both frozen supplied filenames before either supplied choice executes.
pub fn preflight_supplied_attempt_files(directory:&Path)->Result<(),String>{
    std::fs::create_dir_all(directory).map_err(|e|format!("create supplied trace directory: {e}"))?;
    let scenario=FIXED_CASES[0].1;
    for path in [attempt_path(directory,7,scenario,"Defect"),attempt_path(directory,7,scenario,"Strong"),directory.join("supplied-conditional-decision.json")]{
        match std::fs::symlink_metadata(&path){Ok(_)=>return Err(format!("supplied output already exists: {}",path.display())),Err(e)if e.kind()==std::io::ErrorKind::NotFound=>(),Err(e)=>return Err(format!("inspect {}: {e}",path.display()))}
    }Ok(())
}

/// Retain a constructor/public-initialization error without inventing an
/// initial digest or a playable episode. Such an error blocks complete agreement.
pub fn persist_initialization_error(directory:&Path,seed:u64,scenario:Scenario,limit:usize,error:&str)->Result<PathBuf,String>{
    let path=directory.join(format!("seed-{seed}-{:?}-{:?}-{:?}-initialization-error.json",scenario.act1_boss,scenario.act2_boss,scenario.act3_boss));
    let value=serde_json::json!({"schema_id":"m20-caller-initialization-error-v1","seed":seed,"scenario":{"act1_boss":scenario.act1_boss.as_str(),"act2_boss":scenario.act2_boss.as_str(),"act3_boss":scenario.act3_boss.as_str()},"caller_revision":CALLER_POLICY_REVISION,"action_budget":limit,"accepted_action_count":0,"initial_public_sha256":null,"error":error});
    let bytes=serde_json::to_vec_pretty(&value).map_err(|e|format!("serialize initialization error: {e}"))?;write_new(&path,&bytes)?;Ok(path)
}
