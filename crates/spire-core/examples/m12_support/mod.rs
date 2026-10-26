//! Small public-data caller for the shared Silent A0 Act 1 Run.
//! Normal policy: take Lament, route to one reachable elite, then score public
//! rest/event/treasure/shop nodes; in combat, read current intents and cards,
//! mitigate visible pressure, and prefer Poison/Shiv effects. Death control
//! ends turns. It chooses offered actions and owns no game simulation rules.

use std::{collections::BTreeMap, fmt};

use spire_core::{
    combat::{rng::Partition, CombatAction, Outcome, SelectionDisposition, SelectionOperation, TargetId},
    content::{CardId, Character, EncounterId, EventId, PotionId, RelicId},
    run::{
        CommittedEvent, CreationProfile, GameAction, PublicDecision, RoomChoiceKind,
        RoomChoiceStatus, RoomViewKind, Run, RunConfig, RunError, RunRng, RunTerminal,
        RunTransition,
    },
};

pub const ACTION_LIMIT_MAX: usize = 4096;
pub const FIXED_SEEDS: [u64; 2] = [7, 11];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    Guardian,
    Hexaghost,
    SlimeBoss,
    Death,
}

impl Scenario {
    pub const fn encounter(self) -> Option<EncounterId> {
        match self {
            Self::Guardian => Some(EncounterId::Guardian),
            Self::Hexaghost => Some(EncounterId::Hexaghost),
            Self::SlimeBoss => Some(EncounterId::SlimeBoss),
            Self::Death => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Guardian => "guardian",
            Self::Hexaghost => "hexaghost",
            Self::SlimeBoss => "slime",
            Self::Death => "death",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallerOutcome {
    Act2Ready,
    Death,
    BudgetExhausted,
}

impl fmt::Display for CallerOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Act2Ready => write!(f, "Act2Ready"),
            Self::Death => write!(f, "Death (normal terminal outcome)"),
            Self::BudgetExhausted => write!(f, "caller action budget exhausted"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub seed: u64,
    pub scenarios: Vec<Scenario>,
    pub limit: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceStep {
    /// Public decision and sanitized map immediately before the action.
    pub decision: PublicDecision,
    pub map: Option<spire_core::run::map::PublicMap>,
    pub action: GameAction,
    pub transition: RunTransition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Caller {
    limit: usize,
    elite_reached: bool,
    pub trace: Vec<TraceStep>,
    pub final_decision: Option<PublicDecision>,
    pub final_map: Option<spire_core::run::map::PublicMap>,
}

impl Caller {
    pub fn new(limit: usize) -> Result<Self, String> {
        if limit > ACTION_LIMIT_MAX {
            return Err(format!("caller limit must be in 0..={ACTION_LIMIT_MAX}"));
        }
        Ok(Self { limit, elite_reached: false, trace: Vec::new(), final_decision: None, final_map: None })
    }

    pub fn accepted(&self) -> usize {
        self.trace.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub ordinary_moves: usize,
    pub elite_moves: usize,
    pub boss_moves: usize,
    pub reward_screens: usize,
    pub card_reward_screens: usize,
    pub chests_opened: usize,
    pub boss_rewards: usize,
    pub boss_combat_entries: Vec<EncounterId>,
    pub boss_combat_records: Vec<(EncounterId, Outcome)>,
}

/// Boss histories are installed before Run construction. The normal map and
/// encounter owners choose the actual boss from this initial unseen set.
pub fn act1_seen_bosses(scenario: Scenario) -> Vec<&'static str> {
    match scenario {
        Scenario::Guardian | Scenario::Death => vec!["Hexaghost", "SlimeBoss"],
        Scenario::Hexaghost => vec!["Guardian", "SlimeBoss"],
        Scenario::SlimeBoss => vec!["Guardian", "Hexaghost"],
    }
}

pub fn make_run(seed: u64, scenario: Scenario) -> Result<Run, String> {
    if !FIXED_SEEDS.contains(&seed) {
        return Err(format!("seed must be one of the fixed values {FIXED_SEEDS:?}"));
    }
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = act1_seen_bosses(scenario);
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        profile,
        rng(seed),
        spire_core::run::RoomScenario::M12SilentAct1,
    )
    .map_err(|error| format!("M12 Silent A0 creation: {error:?}"))
}

fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(root, Partition::Test, 0)
}

pub fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut seed = 7;
    let mut scenarios = None;
    let mut limit = ACTION_LIMIT_MAX;
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => {
                seed = args.next().ok_or("missing value for --seed")?
                    .parse::<u64>().map_err(|_| "seed must be an integer")?;
                if !FIXED_SEEDS.contains(&seed) {
                    return Err(format!("--seed must be one of {FIXED_SEEDS:?}"));
                }
            }
            "--budget" => {
                limit = args.next().ok_or("missing value for --budget")?
                    .parse::<usize>().map_err(|_| "budget must be an integer")?;
            }
            "--scenario" => {
                scenarios = Some(match args.next().ok_or("missing value for --scenario")?.as_str() {
                    "all" => vec![Scenario::Guardian, Scenario::Hexaghost, Scenario::SlimeBoss],
                    "guardian" => vec![Scenario::Guardian],
                    "hexaghost" => vec![Scenario::Hexaghost],
                    "slime" => vec![Scenario::SlimeBoss],
                    "death" => vec![Scenario::Death],
                    _ => return Err("scenario must be all|guardian|hexaghost|slime|death".into()),
                });
            }
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    if limit > ACTION_LIMIT_MAX {
        return Err(format!("budget must be in 0..={ACTION_LIMIT_MAX}"));
    }
    Ok(Options {
        seed,
        scenarios: scenarios.unwrap_or_else(|| vec![Scenario::Guardian, Scenario::Hexaghost, Scenario::SlimeBoss]),
        limit,
    })
}

/// Returns normal Act2Ready/Death/caller-budget outcomes; invalid actions and
/// retained native faults remain operational errors rather than fake outcomes.
pub fn rollout(run: &mut Run, caller: &mut Caller, scenario: Scenario) -> Result<CallerOutcome, String> {
    loop {
        let decision = run.decision().map_err(classify_run_error)?;
        let map = run.public_map();
        caller.final_decision = Some(decision.clone());
        caller.final_map = map.clone();

        if decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready) {
            return Ok(CallerOutcome::Act2Ready);
        }
        if decision.terminal == Some(RunTerminal::Death) {
            return Ok(CallerOutcome::Death);
        }
        if decision.terminal.is_some() {
            return Err(format!("unexpected native terminal state: {:?}", decision.terminal));
        }
        if caller.accepted() >= caller.limit {
            return Ok(CallerOutcome::BudgetExhausted);
        }

        let action = if scenario == Scenario::Death && decision.combat.is_some() {
            GameAction::Combat(CombatAction::EndTurn)
        } else {
            action_for(&decision, map.as_ref(), caller)?
        };
        if !decision.candidates.iter().any(|candidate| candidate.action == action) {
            return Err(format!("caller selected an action outside the public menu: {action:?}"));
        }
        let transition = run.step(&action).map_err(classify_run_error)?;
        if let (GameAction::Move(id), Some(public_map)) = (&action, &map) {
            if public_map.nodes.iter().any(|node| node.id == *id && node.kind == spire_core::run::MapNodeKind::Elite) {
                caller.elite_reached = true;
            }
        }
        caller.trace.push(TraceStep { decision, map, action, transition });
    }
}

fn classify_run_error(error: RunError) -> String {
    match error {
        RunError::InvalidAction => "invalid public action".into(),
        RunError::Faulted(fault) => format!("retained native fault at {}: {:?}", fault.checkpoint, fault.kind),
        other => format!("native run error: {other:?}"),
    }
}

fn action_for(
    decision: &PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
    caller: &Caller,
) -> Result<GameAction, String> {
    if let Some(combat) = &decision.combat {
        return combat_action(decision, combat);
    }
    if decision.selection.is_some() {
        return room_selection_action(decision);
    }
    let room = decision.room.as_ref().ok_or("missing public room")?;
    let action = match room.kind {
        RoomViewKind::Neow => room_choice(decision, |kind| kind == &RoomChoiceKind::NeowLament),
        RoomViewKind::Map => map.and_then(|map| map_action(decision, map, caller.elite_reached).ok()),
        RoomViewKind::Rewards => room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(id) if !decision.player.relics.contains(id)))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Gold(_))))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::CardReward))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Potion(_))))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed)),
        RoomViewKind::CardReward => card_reward_action(decision),
        RoomViewKind::Shop => shop_action(decision),
        RoomViewKind::Campfire => campfire_action(decision),
        RoomViewKind::Event(EventId::UpgradeShrine) => room_choice(decision, |kind| kind == &RoomChoiceKind::ShrineUpgrade)
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::Purifier) => room_choice(decision, |kind| kind == &RoomChoiceKind::ShrineRemove)
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)),
        RoomViewKind::Event(_) => room_choice(decision, |kind| kind == &RoomChoiceKind::Leave),
        RoomViewKind::Treasure | RoomViewKind::BossTreasure => room_choice(decision, |kind| kind == &RoomChoiceKind::OpenChest)
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(_))))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed)),
        RoomViewKind::BossRelicChoice => room_choice(decision, |kind| kind == &RoomChoiceKind::Relic(RelicId::SlaversCollar))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Relic(RelicId::EmptyCage)))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(_)))),
        RoomViewKind::BossRelicChild => room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed),
        RoomViewKind::ProceedWithoutRelic => room_choice(decision, |kind| kind == &RoomChoiceKind::ConfirmSkip),
        _ => None,
    };
    action.ok_or_else(|| format!("no public caller rule for floor {} in {room_kind:?}", decision.floor, room_kind = room.kind))
}

fn room_choice(decision: &PublicDecision, wanted: impl Fn(&RoomChoiceKind) -> bool) -> Option<GameAction> {
    let row = decision.room.as_ref()?.choices.iter()
        .find(|choice| choice.status == RoomChoiceStatus::Offered && wanted(&choice.kind))?;
    decision.candidates.iter().find(|candidate| candidate.action == GameAction::Choose(row.id))
        .map(|candidate| candidate.action.clone())
}

const REWARD_PRIORITY: [CardId; 15] = [
    CardId::NoxiousFumes, CardId::BladeDance, CardId::Backflip, CardId::DeadlyPoison,
    CardId::PoisonedStab, CardId::CloakAndDagger, CardId::DaggerThrow, CardId::Accuracy,
    CardId::WellLaidPlans, CardId::QuickSlash, CardId::Reflex, CardId::Tactician,
    CardId::Deflect, CardId::Catalyst, CardId::Nightmare,
];

fn card_reward_action(decision: &PublicDecision) -> Option<GameAction> {
    for wanted in REWARD_PRIORITY {
        if let Some(action) = room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Card { id, .. } if *id == wanted)) {
            return Some(action);
        }
    }
    room_choice(decision, |kind| kind == &RoomChoiceKind::Skip)
}

fn shop_action(decision: &PublicDecision) -> Option<GameAction> {
    for wanted in [CardId::QuickSlash, CardId::Deflect, CardId::Backflip, CardId::BladeDance, CardId::DeadlyPoison, CardId::Catalyst, CardId::Nightmare] {
        if let Some(action) = room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Card { id, .. } if *id == wanted)) {
            return Some(action);
        }
    }
    room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Potion(PotionId::FirePotion | PotionId::DexterityPotion)))
        .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(_))))
        .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave))
}

fn campfire_action(decision: &PublicDecision) -> Option<GameAction> {
    if decision.player.max_hp.saturating_sub(decision.player.hp) >= 15 {
        room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Rest { .. }))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Smith))
    } else {
        room_choice(decision, |kind| kind == &RoomChoiceKind::Smith)
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Rest { .. })))
    }
}

fn map_action(
    decision: &PublicDecision,
    map: &spire_core::run::map::PublicMap,
    elite_reached: bool,
) -> Result<GameAction, String> {
    let hp = decision.player.hp as i32;
    let max_hp = decision.player.max_hp as i32;
    let mut memo = BTreeMap::new();
    let score = |id, require_elite, memo: &mut BTreeMap<(spire_core::run::NodeId, bool), i32>| {
        path_score(map, id, elite_reached, require_elite, hp, max_hp, memo)
    };
    let mut candidates = decision.candidates.iter().filter_map(|candidate| {
        let GameAction::Move(id) = candidate.action else { return None };
        score(id, !elite_reached, &mut memo).map(|value| (value, id, candidate.action.clone()))
    }).collect::<Vec<_>>();
    if candidates.is_empty() && !elite_reached {
        memo.clear();
        candidates = decision.candidates.iter().filter_map(|candidate| {
            let GameAction::Move(id) = candidate.action else { return None };
            score(id, false, &mut memo).map(|value| (value, id, candidate.action.clone()))
        }).collect();
    }
    candidates.into_iter().max_by_key(|(score, id, _)| (*score, std::cmp::Reverse(*id)))
        .map(|(_, _, action)| action).ok_or_else(|| "map has no offered route to a boss".into())
}

fn path_score(
    map: &spire_core::run::map::PublicMap,
    id: spire_core::run::NodeId,
    elite_reached: bool,
    require_elite: bool,
    hp: i32,
    max_hp: i32,
    memo: &mut BTreeMap<(spire_core::run::NodeId, bool), i32>,
) -> Option<i32> {
    let key = (id, elite_reached);
    if let Some(score) = memo.get(&key) { return Some(*score); }
    let node = map.nodes.iter().find(|node| node.id == id)?;
    let now_reached = elite_reached || node.kind == spire_core::run::MapNodeKind::Elite;
    let score = if node.kind == spire_core::run::MapNodeKind::Boss {
        if require_elite && !now_reached { return None; }
        0
    } else {
        let next = node.edges.iter().filter_map(|next| path_score(map, *next, now_reached, require_elite, hp, max_hp, memo)).max()?;
        let weight = if !elite_reached && node.kind == spire_core::run::MapNodeKind::Elite {
            100
        } else if now_reached {
            match node.kind {
                spire_core::run::MapNodeKind::Rest if hp < max_hp => 24,
                spire_core::run::MapNodeKind::Rest => 8,
                spire_core::run::MapNodeKind::Question | spire_core::run::MapNodeKind::Event => 16,
                spire_core::run::MapNodeKind::Treasure => 17,
                spire_core::run::MapNodeKind::Shop => 14,
                spire_core::run::MapNodeKind::Monster => 4,
                spire_core::run::MapNodeKind::Elite => -8,
                spire_core::run::MapNodeKind::Boss => 0,
            }
        } else {
            0
        };
        next + weight
    };
    memo.insert(key, score);
    Some(score)
}

fn room_selection_action(decision: &PublicDecision) -> Result<GameAction, String> {
    let selection = decision.selection.as_ref().ok_or("missing public room selection")?;
    if selection.eligible.is_empty() {
        return Err("public room selection has no eligible card".into());
    }
    let prefer_high = matches!(decision.room.as_ref().map(|room| room.kind), Some(RoomViewKind::Event(EventId::UpgradeShrine)));
    let candidate = decision.candidates.iter().filter_map(|candidate| {
        let GameAction::Select(_) = candidate.action else { return None };
        let object = candidate.object.as_ref()?;
        let value = card_value(object.id);
        Some((if prefer_high { value } else { -value }, object.instance, candidate.action.clone()))
    }).max_by_key(|(value, instance, _)| (*value, std::cmp::Reverse(*instance)));
    candidate.map(|(_, _, action)| action).ok_or_else(|| "public room selection has no offered card".into())
}

fn combat_action(decision: &PublicDecision, combat: &spire_core::combat::CombatPublic) -> Result<GameAction, String> {
    let legal = decision.candidates.iter().filter_map(|candidate| match &candidate.action {
        GameAction::Combat(action) => Some(action.clone()),
        _ => None,
    }).collect::<Vec<_>>();
    if legal.is_empty() { return Err("combat has no public legal action".into()); }

    if let Some(selection) = &combat.selection {
        if (selection.operation == SelectionOperation::Retain || selection.required == 0)
            && legal.contains(&CombatAction::ConfirmSelection)
        {
            return Ok(GameAction::Combat(CombatAction::ConfirmSelection));
        }
        let choose_high = matches!(selection.operation,
            SelectionOperation::CopyToHand { .. } | SelectionOperation::NightmareCapture
                | SelectionOperation::Upgrade | SelectionOperation::Topdeck | SelectionOperation::ReturnFromExhaust)
            || matches!(selection.operation, SelectionOperation::Move(SelectionDisposition::Hand | SelectionDisposition::DrawTop));
        let selected = selection.candidates.iter().filter_map(|card| {
            let action = CombatAction::SelectCard { instance: card.instance.0 };
            legal.contains(&action).then_some((card_value(card.id), card.instance.0, action))
        });
        let chosen = if choose_high {
            selected.max_by_key(|(value, instance, _)| (*value, std::cmp::Reverse(*instance)))
        } else {
            selected.min_by_key(|(value, instance, _)| (*value, *instance))
        };
        return chosen.map(|(_, _, action)| GameAction::Combat(action))
            .ok_or_else(|| "combat selection has no currently offered card".into());
    }

    if legal.contains(&CombatAction::ConfirmSelection) {
        return Ok(GameAction::Combat(CombatAction::ConfirmSelection));
    }
    let incoming = combat.enemies.iter().filter(|enemy| enemy.hp > 0 && !enemy.replaced && !enemy.escaped)
        .filter_map(|enemy| enemy.intent_damage).sum::<i32>();
    let under_pressure = incoming > combat.player.block;
    if let Some(action) = potion_action(combat, &legal, incoming, under_pressure) {
        return Ok(GameAction::Combat(action));
    }
    let target = combat.enemies.iter().enumerate()
        .filter(|(_, enemy)| enemy.hp > 0 && !enemy.replaced && !enemy.escaped)
        .min_by_key(|(_, enemy)| (enemy.hp + enemy.block, enemy.hp))
        .map(|(index, _)| TargetId(index as u64 + 1));
    let priorities: &[CardId] = if under_pressure {
        &[CardId::Backflip, CardId::Deflect, CardId::Survivor, CardId::DefendGreen,
            CardId::NoxiousFumes, CardId::BladeDance, CardId::DeadlyPoison, CardId::PoisonedStab,
            CardId::Neutralize, CardId::DaggerThrow, CardId::QuickSlash, CardId::StrikeGreen]
    } else {
        &[CardId::NoxiousFumes, CardId::BladeDance, CardId::DeadlyPoison, CardId::PoisonedStab,
            CardId::Accuracy, CardId::CloakAndDagger, CardId::Neutralize, CardId::DaggerThrow,
            CardId::QuickSlash, CardId::Survivor, CardId::Backflip, CardId::Deflect,
            CardId::StrikeGreen, CardId::DefendGreen]
    };
    for id in priorities {
        if let Some(action) = play_card_action(combat, &legal, *id, target) {
            return Ok(GameAction::Combat(action));
        }
    }
    if let Some(action) = legal.iter().find(|action| matches!(action, CombatAction::PlayCard { .. })) {
        return Ok(GameAction::Combat(action.clone()));
    }
    if legal.contains(&CombatAction::EndTurn) {
        return Ok(GameAction::Combat(CombatAction::EndTurn));
    }
    Ok(GameAction::Combat(legal[0].clone()))
}

fn play_card_action(
    combat: &spire_core::combat::CombatPublic,
    legal: &[CombatAction],
    wanted: CardId,
    target: Option<TargetId>,
) -> Option<CombatAction> {
    combat.hand.iter().filter(|card| card.id == wanted).find_map(|card| {
        legal.iter().find_map(|action| match action {
            CombatAction::PlayCard { instance, target: selected } if *instance == card.instance.0 => {
                if selected.is_none() || selected == &target { Some(action.clone()) } else { None }
            }
            _ => None,
        })
    })
}

fn potion_action(
    combat: &spire_core::combat::CombatPublic,
    legal: &[CombatAction],
    incoming: i32,
    under_pressure: bool,
) -> Option<CombatAction> {
    let target = combat.enemies.iter().enumerate()
        .filter(|(_, enemy)| enemy.hp > 0 && !enemy.replaced && !enemy.escaped)
        .min_by_key(|(_, enemy)| (enemy.hp + enemy.block, enemy.hp))
        .map(|(index, _)| TargetId(index as u64 + 1));
    let inventory = combat.inventory.as_ref()?;
    for (slot, potion) in inventory.potions.iter().enumerate() {
        let use_now = match potion {
            Some(PotionId::FirePotion) => target.and_then(|id| combat.enemies.get(id.0.saturating_sub(1) as usize)).is_some_and(|enemy| enemy.hp >= 15),
            Some(PotionId::WeakPotion) => under_pressure && incoming >= 7,
            Some(PotionId::DexterityPotion | PotionId::BlockPotion) => under_pressure && incoming >= 7,
            Some(PotionId::StrengthPotion) => combat.enemies.iter().any(|enemy| enemy.hp >= 25 && !enemy.replaced),
            _ => false,
        };
        if !use_now { continue; }
        if let Some(action) = legal.iter().find(|action| matches!(action,
            CombatAction::UsePotion { slot: offered, target: selected }
                if *offered == slot && (selected.is_none() || *selected == target)))
        {
            return Some(action.clone());
        }
    }
    None
}

fn card_value(id: CardId) -> i32 {
    match id {
        CardId::NoxiousFumes => 100,
        CardId::BladeDance | CardId::DeadlyPoison | CardId::Backflip => 90,
        CardId::PoisonedStab | CardId::CloakAndDagger | CardId::DaggerThrow => 80,
        CardId::Accuracy | CardId::WellLaidPlans | CardId::Nightmare => 75,
        CardId::QuickSlash | CardId::Catalyst => 70,
        CardId::Neutralize | CardId::Survivor => 60,
        CardId::Deflect | CardId::Reflex | CardId::Tactician => 55,
        CardId::StrikeGreen | CardId::Strike => 40,
        CardId::DefendGreen | CardId::Defend => 35,
        CardId::Wound | CardId::Dazed | CardId::Burn | CardId::Slimed => 5,
        _ => 25,
    }
}

pub fn summarize(caller: &Caller) -> Summary {
    let mut summary = Summary {
        ordinary_moves: 0,
        elite_moves: 0,
        boss_moves: 0,
        reward_screens: 0,
        card_reward_screens: 0,
        chests_opened: 0,
        boss_rewards: 0,
        boss_combat_entries: Vec::new(),
        boss_combat_records: Vec::new(),
    };
    for step in &caller.trace {
        if let (GameAction::Move(id), Some(map)) = (&step.action, &step.map) {
            if let Some(node) = map.nodes.iter().find(|node| node.id == *id) {
                match node.kind {
                    spire_core::run::MapNodeKind::Monster => summary.ordinary_moves += 1,
                    spire_core::run::MapNodeKind::Elite => summary.elite_moves += 1,
                    spire_core::run::MapNodeKind::Boss => summary.boss_moves += 1,
                    _ => (),
                }
            }
        }
        if let Some(room) = &step.decision.room {
            summary.reward_screens += usize::from(room.kind == RoomViewKind::Rewards);
            summary.card_reward_screens += usize::from(room.kind == RoomViewKind::CardReward);
            summary.boss_rewards += usize::from(matches!(room.kind, RoomViewKind::BossTreasure | RoomViewKind::BossRelicChoice));
            if matches!(room.kind, RoomViewKind::Treasure | RoomViewKind::BossTreasure)
                && room.choices.iter().any(|choice| choice.kind == RoomChoiceKind::OpenChest && step.action == GameAction::Choose(choice.id))
            {
                summary.chests_opened += 1;
            }
        }
        for event in &step.transition.committed_events {
            match event {
                CommittedEvent::CombatEntered { encounter, .. }
                    if matches!(encounter, EncounterId::Guardian | EncounterId::Hexaghost | EncounterId::SlimeBoss) => {
                        summary.boss_combat_entries.push(*encounter);
                    }
                CommittedEvent::CombatFinished(record)
                    if matches!(record.encounter, EncounterId::Guardian | EncounterId::Hexaghost | EncounterId::SlimeBoss) => {
                        summary.boss_combat_records.push((record.encounter, record.outcome));
                    }
                _ => (),
            }
        }
    }
    summary
}
