//! Opt-in M15 public caller policy over the shared M13/M14 rollout owner.

#[path = "../m13_support/mod.rs"]
#[allow(dead_code)]
pub mod m13_support;

#[allow(unused_imports)]
pub use self::m13_support::{m15_stop_outcome, rollout_m15, Caller, M15CallerOutcome, TraceStep};

pub const CALLER_POLICY_REVISION: &str = "m15-key-aware-public-caller-v1";
pub const M15_ACTION_LIMIT_MAX: usize = m13_support::M15_ACTION_LIMIT_MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub act1_boss: spire_core::content::EncounterId,
    pub act2_boss: spire_core::content::EncounterId,
    pub act3_boss: spire_core::content::EncounterId,
}

impl Scenario {
    pub const fn label(self) -> &'static str {
        match self.act1_boss {
            spire_core::content::EncounterId::Guardian => "guardian",
            spire_core::content::EncounterId::Hexaghost => "hexaghost",
            spire_core::content::EncounterId::SlimeBoss => "slime",
            _ => "invalid",
        }
    }
}

/// The original six seed/triples from M14, reused unchanged for M15.
pub const FIXED_CASES: [(u64, Scenario); 6] = [
    (7, Scenario { act1_boss: spire_core::content::EncounterId::Guardian, act2_boss: spire_core::content::EncounterId::Champ, act3_boss: spire_core::content::EncounterId::AwakenedOne }),
    (7, Scenario { act1_boss: spire_core::content::EncounterId::Hexaghost, act2_boss: spire_core::content::EncounterId::BronzeAutomaton, act3_boss: spire_core::content::EncounterId::TimeEater }),
    (7, Scenario { act1_boss: spire_core::content::EncounterId::SlimeBoss, act2_boss: spire_core::content::EncounterId::Collector, act3_boss: spire_core::content::EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: spire_core::content::EncounterId::Guardian, act2_boss: spire_core::content::EncounterId::BronzeAutomaton, act3_boss: spire_core::content::EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: spire_core::content::EncounterId::Hexaghost, act2_boss: spire_core::content::EncounterId::Collector, act3_boss: spire_core::content::EncounterId::AwakenedOne }),
    (11, Scenario { act1_boss: spire_core::content::EncounterId::SlimeBoss, act2_boss: spire_core::content::EncounterId::Champ, act3_boss: spire_core::content::EncounterId::TimeEater }),
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
    Ok(FIXED_CASES.iter().filter(|(case_seed, _)| *case_seed == seed).map(|(_, scenario)| *scenario).collect())
}

pub fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut seed = 7;
    let mut scenario_label = "all".to_owned();
    let mut limit = M15_ACTION_LIMIT_MAX;
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => seed = args.next().ok_or("missing --seed value")?.parse().map_err(|_| "seed must be integer")?,
            "--budget" => limit = args.next().ok_or("missing --budget value")?.parse().map_err(|_| "budget must be integer")?,
            "--scenario" => scenario_label = args.next().ok_or("missing --scenario value")?,
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    let all = cases_for_seed(seed)?;
    let scenarios = match scenario_label.as_str() {
        "all" => all,
        "guardian" | "hexaghost" | "slime" => all.into_iter().filter(|scenario| scenario.label() == scenario_label).collect(),
        _ => return Err("scenario must be all|guardian|hexaghost|slime".into()),
    };
    if limit > M15_ACTION_LIMIT_MAX {
        return Err(format!("budget must be in 0..={M15_ACTION_LIMIT_MAX}"));
    }
    Ok(Options { seed, scenarios, limit })
}

pub fn make_run(seed: u64, scenario: Scenario) -> Result<spire_core::run::Run, String> {
    use spire_core::{content::Character, run::{CreationProfile, Run, RunConfig, RoomScenario}};

    if !cases_for_seed(seed)?.contains(&scenario) {
        return Err("scenario is outside the six frozen M15 seed/boss triples".into());
    }
    let m13_scenario = m13_support::Scenario { act1_boss: scenario.act1_boss, act2_boss: scenario.act2_boss };
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = m13_support::act1_seen_bosses(m13_scenario);
    profile.seen_bosses[1] = m13_support::act2_seen_bosses(m13_scenario);
    profile.seen_bosses[2] = act3_seen_bosses(scenario.act3_boss)?;
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        profile,
        m13_support::rng(seed),
        RoomScenario::M15SilentHeart,
    ).map_err(|error| format!("M15 Silent A0 creation: {error:?}"))
}

fn act3_seen_bosses(target: spire_core::content::EncounterId) -> Result<Vec<&'static str>, String> {
    use spire_core::content::EncounterId;
    match target {
        EncounterId::AwakenedOne => Ok(vec!["TimeEater", "DonuAndDeca"]),
        EncounterId::TimeEater => Ok(vec!["AwakenedOne", "DonuAndDeca"]),
        EncounterId::DonuAndDeca => Ok(vec!["AwakenedOne", "TimeEater"]),
        _ => Err(format!("invalid M15 Act3 boss-history target: {target:?}")),
    }
}

pub fn preflight_attempt_files(directory: &std::path::Path, scenarios: &[Scenario], seed: u64) -> Result<(), String> {
    std::fs::create_dir_all(directory).map_err(|error| format!("create M15 trace directory: {error}"))?;
    for scenario in scenarios {
        let (json_path, trace_path) = attempt_paths(directory, seed, *scenario);
        if json_path.exists() || trace_path.exists() {
            return Err(format!("M15 attempt output already exists for seed={seed} scenario={scenario:?}; refusing a rerun"));
        }
    }
    Ok(())
}

pub fn persist_attempt(
    directory: &std::path::Path,
    seed: u64,
    scenario: Scenario,
    limit: usize,
    result: &Result<M15CallerOutcome, String>,
    caller: &Caller,
    run: Option<&spire_core::run::Run>,
) -> Result<(), String> {
    use serde_json::json;

    let (json_path, trace_path) = attempt_paths(directory, seed, scenario);
    let (live_decision, decision_error) = match run {
        Some(run) => match run.decision() {
            Ok(decision) => (Some(decision), None),
            Err(error) => (None, Some(format!("{error:?}"))),
        },
        None => (None, Some("no Run was created".to_owned())),
    };
    let endpoint_decision = live_decision.as_ref().or(caller.final_decision.as_ref());
    let endpoint_map = run.and_then(spire_core::run::Run::public_map).or_else(|| caller.final_map.clone());
    let mut steps = Vec::with_capacity(caller.trace.len());
    for (index, step) in caller.trace.iter().enumerate() {
        let candidate_index = step.decision.candidates.iter()
            .position(|candidate| candidate.action == step.action)
            .ok_or_else(|| format!("accepted M15 trace step {index} action is absent from its public candidate list: {:?}", step.action))?;
        let (post_decision_debug, post_map_debug) = if let Some(next) = caller.trace.get(index + 1) {
            (Ok(format!("{:#?}", next.decision)), next.map.as_ref().map(|map| format!("{map:#?}")))
        } else if caller.failed_action_for_diagnostics().is_some() {
            (
                caller.final_decision.as_ref().map(|decision| format!("{decision:#?}"))
                    .ok_or_else(|| "prefailure decision unavailable for final accepted M15 trace step".to_owned()),
                caller.final_map.as_ref().map(|map| format!("{map:#?}")),
            )
        } else if let Some(run) = run {
            (
                run.decision().map(|decision| format!("{decision:#?}"))
                    .map_err(|error| format!("{error:?}")),
                run.public_map().map(|map| format!("{map:#?}")),
            )
        } else {
            (Err("live Run unavailable for final accepted M15 trace step".to_owned()), None)
        };
        let post_decision = match &post_decision_debug {
            Ok(debug) => json!({"debug": debug, "error": null}),
            Err(error) => json!({"debug": null, "error": error}),
        };
        steps.push(json!({
            "index": index,
            "candidate_index": candidate_index,
            "pre": {
                "decision_debug": format!("{:#?}", step.decision),
                "map_debug": step.map.as_ref().map(|map| format!("{map:#?}")),
            },
            "action": native_action_value(&step.action),
            "transition": {
                "reward": step.transition.reward,
                "terminal": terminal_value(step.transition.terminal),
                "committed_events": step.transition.committed_events.iter().map(event_value).collect::<Vec<_>>(),
                "committed_events_debug": step.transition.committed_events.iter().map(|event| format!("{event:?}")).collect::<Vec<_>>(),
            },
            "post": {
                "decision": post_decision,
                "map_debug": post_map_debug,
            },
        }));
    }
    let receipt = json!({
        "schema_id": "m15-silent-heart-caller-attempt-v1",
        "seed": seed,
        "scenario": {
            "act1_boss": format!("{:?}", scenario.act1_boss),
            "act2_boss": format!("{:?}", scenario.act2_boss),
            "act3_boss": format!("{:?}", scenario.act3_boss),
        },
        "caller_revision": CALLER_POLICY_REVISION,
        "action_budget": limit,
        "outcome": result.as_ref().ok().map(ToString::to_string),
        "error": result.as_ref().err(),
        "accepted_action_count": caller.trace.len(),
        "actions": caller.trace.iter().map(|step| native_action_value(&step.action)).collect::<Vec<_>>(),
        "steps": steps,
        "failed_action": caller.failed_action_for_diagnostics().map(native_action_value),
        "endpoint": {
            "available": live_decision.is_some(),
            "decision_error": decision_error,
            "act": endpoint_decision.map(|decision| decision.act),
            "floor": endpoint_decision.map(|decision| decision.floor),
            "room": endpoint_decision.and_then(|decision| decision.room.as_ref()).map(|room| format!("{:?}", room.kind)),
            "terminal": endpoint_decision.map(|decision| terminal_value(decision.terminal)).unwrap_or(serde_json::Value::Null),
            "decision_debug": endpoint_decision.map(|decision| format!("{decision:#?}")),
            "map_debug": endpoint_map.as_ref().map(|map| format!("{map:#?}")),
        },
    });
    let json_bytes = serde_json::to_vec_pretty(&receipt).map_err(|error| format!("serialize M15 attempt receipt: {error}"))?;
    let trace_text = format!(
        "schema_id=m15-silent-heart-caller-trace-v1\nseed={seed}\nscenario={scenario:?}\ncaller_revision={CALLER_POLICY_REVISION}\naction_budget={limit}\noutcome={:?}\nerror={:?}\nfailed_action={:?}\nendpoint={:?}\n{caller:#?}\n",
        result.as_ref().ok(), result.as_ref().err(), caller.failed_action_for_diagnostics(),
        endpoint_decision.map(|decision| (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind))),
    );
    write_new(&json_path, &json_bytes)?;
    write_new(&trace_path, trace_text.as_bytes())?;
    Ok(())
}

fn attempt_paths(directory: &std::path::Path, seed: u64, scenario: Scenario) -> (std::path::PathBuf, std::path::PathBuf) {
    let stem = format!("seed-{seed}-{:?}-{:?}-{:?}", scenario.act1_boss, scenario.act2_boss, scenario.act3_boss);
    (directory.join(format!("{stem}.attempt.json")), directory.join(format!("{stem}.trace.txt")))
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("create M15 evidence {}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("write M15 evidence {}: {error}", path.display()))?;
    file.sync_all().map_err(|error| format!("sync M15 evidence {}: {error}", path.display()))
}

pub(crate) fn native_action_value(action: &spire_core::run::GameAction) -> serde_json::Value {
    use serde_json::json;
    use spire_core::{combat::CombatAction, run::GameAction};

    match action {
        GameAction::Move(id) => json!({"verb":"move","node":id.0}),
        GameAction::Choose(id) => json!({"verb":"choose","choice":id.0}),
        GameAction::Select(id) => json!({"verb":"select","selection":id.0}),
        GameAction::Confirm => json!({"verb":"confirm"}),
        GameAction::Cancel => json!({"verb":"cancel"}),
        GameAction::UsePotion { slot, target } => json!({"verb":"use_potion","slot":slot,"target":target.map(|target|target.0)}),
        GameAction::DiscardPotion { slot } => json!({"verb":"discard_potion","slot":slot}),
        GameAction::Combat(action) => match action {
            CombatAction::ChooseCardOffer { index } => json!({"verb":"combat.choose_card_offer","index":index}),
            CombatAction::SkipCardOffer => json!({"verb":"combat.skip_card_offer"}),
            CombatAction::PlayCard { instance, target } => json!({"verb":"combat.play_card","instance":instance,"target":target.map(|target|target.0)}),
            CombatAction::SelectCard { instance } => json!({"verb":"combat.select_card","instance":instance}),
            CombatAction::ConfirmSelection => json!({"verb":"combat.confirm_selection"}),
            CombatAction::UsePotion { slot, target } => json!({"verb":"combat.use_potion","slot":slot,"target":target.map(|target|target.0)}),
            CombatAction::EndTurn => json!({"verb":"combat.end_turn"}),
        },
    }
}

pub fn endpoint_summary(run: Option<&spire_core::run::Run>, caller: &Caller) -> String {
    match run {
        Some(run) => match run.decision() {
            Ok(decision) => format!("act={} floor={} terminal={:?} room={:?}", decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind)),
            Err(error) => format!("unavailable: {error:?}; last caller decision={:?}", caller.final_decision.as_ref().map(|decision| (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind)))),
        },
        None => format!("unavailable: no Run; last caller decision={:?}", caller.final_decision.as_ref().map(|decision| (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind)))),
    }
}

fn terminal_value(terminal: Option<spire_core::run::RunTerminal>) -> serde_json::Value {
    use serde_json::{json, Value};
    use spire_core::run::RunTerminal;
    match terminal {
        None => Value::Null,
        Some(RunTerminal::Death) => json!("death"),
        Some(RunTerminal::EmptyDeck) => json!("empty_deck"),
        Some(RunTerminal::Act3NoKeys) => json!("act3_no_keys"),
        Some(RunTerminal::HeartVictory) => json!("heart_victory"),
    }
}

fn event_value(event: &spire_core::run::CommittedEvent) -> serde_json::Value {
    use serde_json::json;
    use spire_core::run::{CommittedEvent, Key};
    match event {
        CommittedEvent::FloorEntered { act, floor } => json!({"kind":"floor_entered","act":act,"floor":floor}),
        CommittedEvent::KeyAcquired(key) => json!({"kind":"key_acquired","key":match key {Key::Ruby=>"ruby",Key::Sapphire=>"sapphire",Key::Emerald=>"emerald"}}),
        CommittedEvent::Ended(terminal) => json!({"kind":"ended","terminal":terminal_value(Some(*terminal))}),
        CommittedEvent::ActAdvanced { from, to } => json!({"kind":"act_advanced","from":from,"to":to}),
        CommittedEvent::CombatEntered { visit, encounter } => json!({"kind":"combat_entered","visit":visit,"encounter":format!("{encounter:?}")}),
        CommittedEvent::CombatFinished(record) => json!({"kind":"combat_finished","visit":record.visit,"encounter":format!("{:?}",record.encounter),"outcome":format!("{:?}",record.outcome)}),
    }
}

/// Prefer only offered key objectives visible in the current public decision.
pub fn key_action(
    decision: &spire_core::run::PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
) -> Option<spire_core::run::GameAction> {
    use spire_core::run::{RoomChoiceKind, RoomViewKind};

    if decision.content_profile != Some(spire_core::run::RoomScenario::M15SilentHeart)
        || decision.combat.is_some()
        || decision.selection.is_some()
    {
        return None;
    }
    let room = decision.room.as_ref()?;

    if !decision.player.keys[0] {
        if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::Recall) {
            return Some(action);
        }
    }
    if !decision.player.keys[1] {
        if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::Sapphire) {
            return Some(action);
        }
        if matches!(room.kind, RoomViewKind::Treasure | RoomViewKind::BossTreasure) {
            if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::OpenChest) {
                return Some(action);
            }
        }
    }
    if !decision.player.keys[2] {
        if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::Emerald) {
            return Some(action);
        }
    }

    match room.kind {
        RoomViewKind::Map if !decision.player.keys[2] => map.and_then(|map| emerald_route_action(decision, map)),
        RoomViewKind::Act3Ending => room_choice_action(decision, |kind| kind == &RoomChoiceKind::Act3Continue { stage: 4 }),
        RoomViewKind::HeartVictory => room_choice_action(decision, |kind| kind == &RoomChoiceKind::Proceed),
        _ => None,
    }
}

fn room_choice_action(
    decision: &spire_core::run::PublicDecision,
    wanted: impl Fn(&spire_core::run::RoomChoiceKind) -> bool,
) -> Option<spire_core::run::GameAction> {
    use spire_core::run::{GameAction, RoomChoiceStatus};

    let room = decision.room.as_ref()?;
    let choice = room.choices.iter().find(|choice| choice.status == RoomChoiceStatus::Offered && wanted(&choice.kind))?;
    let action = GameAction::Choose(choice.id);
    decision.candidates.iter().any(|candidate| candidate.id == choice.id && candidate.action == action).then_some(action)
}

fn emerald_route_action(
    decision: &spire_core::run::PublicDecision,
    map: &spire_core::run::map::PublicMap,
) -> Option<spire_core::run::GameAction> {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    use spire_core::run::{GameAction, MapNodeKind, NodeId};

    if !(1..=3).contains(&decision.act) || decision.act != map.act {
        return None;
    }
    let nodes = map.nodes.iter().map(|node| (node.id, node)).collect::<BTreeMap<_, _>>();
    let mut best: Option<(usize, NodeId)> = None;
    for candidate in &decision.candidates {
        let GameAction::Move(start) = &candidate.action else { continue };
        let start = *start;
        let Some(start_node) = nodes.get(&start) else { continue };
        if start_node.visited {
            continue;
        }
        let mut queue = VecDeque::from([(start, 0usize)]);
        let mut seen = BTreeSet::from([start]);
        while let Some((at, distance)) = queue.pop_front() {
            let Some(node) = nodes.get(&at) else { continue };
            if node.kind == MapNodeKind::Elite
                && !node.visited
                && node.burning == Some(true)
                && reaches_boss(at, &nodes)
            {
                let route = (distance, start);
                if best.map_or(true, |current| route < current) {
                    best = Some(route);
                }
                break;
            }
            for next in &node.edges {
                if nodes.contains_key(next) && seen.insert(*next) {
                    queue.push_back((*next, distance + 1));
                }
            }
        }
    }
    best.map(|(_, node)| GameAction::Move(node))
}

fn reaches_boss(
    start: spire_core::run::NodeId,
    nodes: &std::collections::BTreeMap<spire_core::run::NodeId, &spire_core::run::map::PublicMapNode>,
) -> bool {
    use std::collections::{BTreeSet, VecDeque};
    use spire_core::run::MapNodeKind;

    let mut queue = VecDeque::from([start]);
    let mut seen = BTreeSet::from([start]);
    while let Some(at) = queue.pop_front() {
        let Some(node) = nodes.get(&at) else { continue };
        for next in &node.edges {
            let Some(next_node) = nodes.get(next) else { continue };
            if next_node.kind == MapNodeKind::Boss {
                return true;
            }
            if seen.insert(*next) {
                queue.push_back(*next);
            }
        }
    }
    false
}

#[cfg(test)]
mod caller_tests;
