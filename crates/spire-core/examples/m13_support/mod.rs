//! Frozen public-data caller for the shared Silent A0 Acts 1–2 Run.
//! Revision v2: retain Lament and ordinary room choices; compare visible
//! routes by health/resources with strict ordinary/elite coverage, handle
//! immediate survival before setup, and use threat-based potions. Death control
//! ends turns. It chooses offered actions and owns no game simulation rules.

use std::fmt;

mod public_routes_v2;
mod m14_revision;
mod m14_combat_choices;
mod m14_threat;

use spire_core::{
    combat::{rng::Partition, CombatAction, Outcome, SelectionDisposition, SelectionOperation, TargetId},
    content::{CardId, Character, EncounterId, EventId, PotionId, RelicId},
    run::{
        CommittedEvent, CreationProfile, GameAction, PublicDecision, RoomChoiceKind,
        RoomChoiceStatus, RoomViewKind, Run, RunConfig, RunError, RunRng, RunTerminal,
        RunTransition, RoomScenario,
    },
};

pub const CALLER_POLICY_REVISION: &str = "m13-public-caller-v2";
pub const ACTION_LIMIT_MAX: usize = 4096;
pub const M14_ACTION_LIMIT_MAX: usize = 8192;
pub const M15_ACTION_LIMIT_MAX: usize = 8192;
pub const FIXED_SEEDS: [u64; 2] = [7, 11];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub act1_boss: EncounterId,
    pub act2_boss: EncounterId,
}

pub const FIXED_CASES: [(u64, Scenario); 6] = [
    (7, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::Champ }),
    (7, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::BronzeAutomaton }),
    (7, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Collector }),
    (11, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::BronzeAutomaton }),
    (11, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::Collector }),
    (11, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Champ }),
];

pub fn cases_for_seed(seed: u64) -> Result<Vec<Scenario>, String> {
    if !FIXED_SEEDS.contains(&seed) { return Err(format!("seed must be one of {FIXED_SEEDS:?}")); }
    Ok(FIXED_CASES.iter().filter(|(s, _)| *s == seed).map(|(_, scenario)| *scenario).collect())
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallerOutcome {
    Act3Ready,
    Death,
    BudgetExhausted,
    Act3NoKeys,
    Act4Ready,
    EmptyDeck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum M15CallerOutcome {
    HeartVictory,
    Death,
    BudgetExhausted,
    Act3NoKeys,
    EmptyDeck,
}

impl fmt::Display for M15CallerOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeartVictory => write!(f, "HeartVictory"),
            Self::Death => write!(f, "Death"),
            Self::BudgetExhausted => write!(f, "BudgetExhausted"),
            Self::Act3NoKeys => write!(f, "Act3NoKeys"),
            Self::EmptyDeck => write!(f, "EmptyDeck"),
        }
    }
}

impl fmt::Display for CallerOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Act3Ready => write!(f, "Act3Ready"),
            Self::Death => write!(f, "Death (normal terminal outcome)"),
            Self::BudgetExhausted => write!(f, "caller action budget exhausted"),
            Self::Act3NoKeys => write!(f, "Act3NoKeys (normal non-Heart completion)"),
            Self::Act4Ready => write!(f, "Act4Ready (supplied M14 cutoff)"),
            Self::EmptyDeck => write!(f, "EmptyDeck (normal terminal outcome)"),
        }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M22CallerOutcome{Act2Ready,Death,EmptyDeck,BudgetExhausted}
impl fmt::Display for M22CallerOutcome{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{self:?}")}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M23CallerOutcome{Act3Ready,Death,EmptyDeck,BudgetExhausted}
impl fmt::Display for M23CallerOutcome{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{self:?}")}}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub seed: u64,
    pub scenarios: Vec<Scenario>,
    pub limit: usize,
    pub nonattacking: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceStep {
    /// Public decision and sanitized map immediately before the action.
    pub decision: PublicDecision,
    pub map: Option<spire_core::run::map::PublicMap>,
    pub action: GameAction,
    pub transition: RunTransition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallerProfile {
    M13,
    M14,
    M15,
    M20,
    M22,
    M23,
}

enum SharedOutcome {
    Legacy(CallerOutcome),
    M15(M15CallerOutcome),
    M22(M22CallerOutcome),
    M23(M23CallerOutcome),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Caller {
    limit: usize,
    profile: CallerProfile,
    elite_reached: [bool; 2],
    ordinary_reached: [bool; 2],
    act3_elite_reached: bool,
    act3_ordinary_reached: bool,
    failed_action: Option<GameAction>,
    failed_error: Option<RunError>,
    pub nonattacking: bool,
    pub trace: Vec<TraceStep>,
    pub m22_observed:Vec<(Vec<spire_core::combat::ResolvedEffect>,Vec<spire_core::combat::CombatBoundary>)>,
    pub final_decision: Option<PublicDecision>,
    pub final_map: Option<spire_core::run::map::PublicMap>,
}

// Keep M13's published Debug transcript byte shape unchanged. M14-only state
// is included for its diagnostics, while accepted M13 headers and TraceSteps
// retain their previous representation.
impl fmt::Debug for Caller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Caller");
        debug
            .field("limit", &self.limit)
            .field("elite_reached", &self.elite_reached)
            .field("ordinary_reached", &self.ordinary_reached);
        if self.profile != CallerProfile::M13 {
            debug
                .field("profile", &self.profile)
                .field("act3_elite_reached", &self.act3_elite_reached)
                .field("act3_ordinary_reached", &self.act3_ordinary_reached)
                .field("failed_action", &self.failed_action);
        }
        debug
            .field("nonattacking", &self.nonattacking)
            .field("trace", &self.trace)
            .field("final_decision", &self.final_decision)
            .field("final_map", &self.final_map)
            .finish()
    }
}

impl Caller {
    pub fn new(limit: usize) -> Result<Self, String> {
        if limit > ACTION_LIMIT_MAX {
            return Err(format!("caller limit must be in 0..={ACTION_LIMIT_MAX}"));
        }
        Ok(Self::with_profile(limit, CallerProfile::M13))
    }

    pub fn new_m14(limit: usize) -> Result<Self, String> {
        if limit > M14_ACTION_LIMIT_MAX {
            return Err(format!("M14 caller limit must be in 0..={M14_ACTION_LIMIT_MAX}"));
        }
        Ok(Self::with_profile(limit, CallerProfile::M14))
    }

    pub fn new_m15(limit: usize) -> Result<Self, String> {
        if limit > M15_ACTION_LIMIT_MAX {
            return Err(format!("M15 caller limit must be in 0..={M15_ACTION_LIMIT_MAX}"));
        }
        Ok(Self::with_profile(limit, CallerProfile::M15))
    }

    /// M20 uses the same trace owner and loop with its frozen public policy.
    pub fn new_m20(limit:usize)->Result<Self,String>{
        if limit>8192{return Err("M20 caller limit must be in 0..=8192".into())}
        Ok(Self::with_profile(limit,CallerProfile::M20))
    }

    pub fn new_m22(limit:usize)->Result<Self,String>{
        if limit>32768{return Err("M22 caller limit must be in 0..=32768".into())}
        Ok(Self::with_profile(limit,CallerProfile::M22))
    }
    /// M23 uses the same trace owner/loop and observed-step capture as M22.
    pub fn new_m23(limit:usize)->Result<Self,String>{
        if limit>32768{return Err("M23 caller limit must be in 0..=32768".into())}
        Ok(Self::with_profile(limit,CallerProfile::M23))
    }
    fn with_profile(limit: usize, profile: CallerProfile) -> Self {
        Self {
            limit,
            profile,
            elite_reached: [false; 2],
            ordinary_reached: [false; 2],
            act3_elite_reached: false,
            act3_ordinary_reached: false,
            failed_action: None,
            failed_error: None,
            nonattacking: false,
            trace: Vec::new(),
            m22_observed:Vec::new(),
            final_decision: None,
            final_map: None,
        }
    }

    pub fn accepted(&self) -> usize {
        self.trace.len()
    }

    pub fn failed_action_for_diagnostics(&self) -> Option<&GameAction> {
        self.failed_action.as_ref()
    }

    /// Raw accepted M20 error for receipt consumers; old caller text/Debug stays frozen.
    pub fn failed_error_for_diagnostics(&self)->Option<&RunError>{self.failed_error.as_ref()}

    pub fn route_progress(&self, act: u8) -> (bool, bool) {
        match act {
            1 | 2 => {
                let index = (act - 1) as usize;
                (self.elite_reached[index], self.ordinary_reached[index])
            }
            3 if self.profile != CallerProfile::M13 => {
                (self.act3_elite_reached, self.act3_ordinary_reached)
            }
            // M13 has no Act3 map decision. Keep its prior no-extra-quota
            // behavior if a diagnostic caller presents one.
            _ => (true, true),
        }
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
    match scenario.act1_boss {
        EncounterId::Guardian => vec!["Hexaghost", "SlimeBoss"],
        EncounterId::Hexaghost => vec!["Guardian", "SlimeBoss"],
        EncounterId::SlimeBoss => vec!["Guardian", "Hexaghost"],
        _ => vec![],
    }
}

pub fn act2_seen_bosses(scenario: Scenario) -> Vec<&'static str> {
    match scenario.act2_boss {
        EncounterId::Champ => vec!["Automaton", "Collector"],
        EncounterId::BronzeAutomaton => vec!["Champ", "Collector"],
        EncounterId::Collector => vec!["Champ", "Automaton"],
        _ => vec![],
    }
}

pub fn make_run(seed: u64, scenario: Scenario) -> Result<Run, String> {
    if !FIXED_SEEDS.contains(&seed) {
        return Err(format!("seed must be one of the fixed values {FIXED_SEEDS:?}"));
    }
    if !cases_for_seed(seed)?.contains(&scenario) { return Err("scenario is outside the six frozen seed/boss pairs".into()); }
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = act1_seen_bosses(scenario);
    profile.seen_bosses[1] = act2_seen_bosses(scenario);
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        profile,
        rng(seed),
        spire_core::run::RoomScenario::M13SilentAct2,
    )
    .map_err(|error| format!("M13 Silent A0 creation: {error:?}"))
}

pub(super) fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(root, Partition::Test, 0)
}

pub fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut seed = 7;
    let mut scenario_label = "all".to_owned();
    let mut nonattacking = false;
    let mut limit = ACTION_LIMIT_MAX;
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
        "death" => { nonattacking = true; vec![all[0]] },
        "guardian" | "hexaghost" | "slime" => all.into_iter().filter(|scenario| scenario.label() == scenario_label).collect(),
        _ => return Err("scenario must be all|guardian|hexaghost|slime|death".into()),
    };
    if limit > ACTION_LIMIT_MAX { return Err(format!("budget must be in 0..={ACTION_LIMIT_MAX}")); }
    Ok(Options { seed, scenarios, limit, nonattacking })
}

/// Returns normal Act3Ready/Death/caller-budget outcomes; invalid actions and
/// retained native faults remain operational errors rather than fake outcomes.
pub fn rollout(run: &mut Run, caller: &mut Caller, _scenario: Scenario) -> Result<CallerOutcome, String> {
    if caller.profile != CallerProfile::M13 {
        return Err("M13 rollout requires an M13 caller".into());
    }
    match rollout_shared(run, caller, CallerProfile::M13, None, None)? {
        SharedOutcome::Legacy(outcome) => Ok(outcome),
        SharedOutcome::M15(_) | SharedOutcome::M22(_) | SharedOutcome::M23(_) => Err("M13 rollout received an M15 finish result".into()),
    }
}

/// Runs the same public caller policy through M14's additive Act3 endpoints.
/// Act3NoKeys, real Death/EmptyDeck, and the supplied-only Act4Ready cutoff
/// remain distinct outcomes.
pub fn rollout_m14(run: &mut Run, caller: &mut Caller, death_control: bool) -> Result<CallerOutcome, String> {
    if caller.profile != CallerProfile::M14 {
        return Err("M14 rollout requires an M14 caller".into());
    }
    let initial = run.decision().map_err(classify_run_error)?;
    if initial.content_profile != Some(RoomScenario::M14SilentAct3) {
        return Err("M14 rollout requires the M14SilentAct3 Run profile".into());
    }
    caller.nonattacking = death_control;
    match rollout_shared(run, caller, CallerProfile::M14, None, None)? {
        SharedOutcome::Legacy(outcome) => Ok(outcome),
        SharedOutcome::M15(_) | SharedOutcome::M22(_) | SharedOutcome::M23(_) => Err("M14 rollout received an M15 finish result".into()),
    }
}

/// Runs M15 through the same shared public rollout loop with one pure key override.
pub fn rollout_m15(
    run: &mut Run,
    caller: &mut Caller,
    key_action: fn(&PublicDecision, Option<&spire_core::run::map::PublicMap>) -> Option<GameAction>,
) -> Result<M15CallerOutcome, String> {
    if caller.profile != CallerProfile::M15 {
        return Err("M15 rollout requires an M15 caller".into());
    }
    let initial = run.decision().map_err(classify_run_error)?;
    if initial.content_profile != Some(RoomScenario::M15SilentHeart) {
        return Err("M15 rollout requires the M15SilentHeart Run profile".into());
    }
    match rollout_shared(run, caller, CallerProfile::M15, Some(key_action), None)? {
        SharedOutcome::M15(outcome) => Ok(outcome),
        SharedOutcome::Legacy(_) | SharedOutcome::M22(_) | SharedOutcome::M23(_) => Err("M15 rollout received a legacy finish result".into()),
    }
}

/// One loop/TraceStep/ending owner; this callback receives only public data.
pub type M20CallerOutcome=M15CallerOutcome;
pub fn rollout_m20(run:&mut Run,caller:&mut Caller,
    policy:fn(&PublicDecision,Option<&spire_core::run::map::PublicMap>,&Caller)->Result<GameAction,String>)
    ->Result<M20CallerOutcome,String>{
    if caller.profile!=CallerProfile::M20{return Err("M20 rollout requires an M20 caller".into())}
    if run.decision().map_err(classify_run_error)?.content_profile!=Some(RoomScenario::M20DefectHeart){
        return Err("M20 rollout requires the M20DefectHeart Run profile".into())
    }
    match rollout_shared(run,caller,CallerProfile::M20,None,Some(policy))? {
        SharedOutcome::M15(outcome)=>Ok(outcome),SharedOutcome::Legacy(_)|SharedOutcome::M22(_)|SharedOutcome::M23(_)=>Err("M20 received a legacy finish result".into())
    }
}

/// M22 extends the accepted shared trace/rollout owner with its own healthy cutoff.
pub fn rollout_m22(run:&mut Run,caller:&mut Caller,
    policy:fn(&PublicDecision,Option<&spire_core::run::map::PublicMap>,&Caller)->Result<GameAction,String>)
    ->Result<M22CallerOutcome,String>{
    if caller.profile!=CallerProfile::M22||run.decision().map_err(classify_run_error)?.content_profile!=Some(RoomScenario::M22WatcherAct1){return Err("M22 rollout requires its M22 caller and Run profile".into())}
    match rollout_shared(run,caller,CallerProfile::M22,None,Some(policy))?{SharedOutcome::M22(outcome)=>Ok(outcome),_=>Err("M22 received a foreign finish result".into())}
}

/// M23 continues the same shared owner through Act2 to its healthy Act3Ready cutoff.
pub fn rollout_m23(run:&mut Run,caller:&mut Caller,
    policy:fn(&PublicDecision,Option<&spire_core::run::map::PublicMap>,&Caller)->Result<GameAction,String>)
    ->Result<M23CallerOutcome,String>{
    if caller.profile!=CallerProfile::M23||run.decision().map_err(classify_run_error)?.content_profile!=Some(RoomScenario::M23WatcherAct2){return Err("M23 rollout requires its M23 caller and Run profile".into())}
    match rollout_shared(run,caller,CallerProfile::M23,None,Some(policy))?{SharedOutcome::M23(outcome)=>Ok(outcome),_=>Err("M23 received a foreign finish result".into())}
}

pub(crate) fn m14_stop_outcome(decision: &PublicDecision) -> Option<CallerOutcome> {
    match decision.terminal {
        Some(RunTerminal::Act3NoKeys) => Some(CallerOutcome::Act3NoKeys),
        Some(RunTerminal::Death) => Some(CallerOutcome::Death),
        Some(RunTerminal::EmptyDeck) => Some(CallerOutcome::EmptyDeck),
        Some(RunTerminal::HeartVictory) => None,
        None if decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act4Ready) => {
            Some(CallerOutcome::Act4Ready)
        }
        None => None,
    }
}

fn rollout_shared(
    run: &mut Run,
    caller: &mut Caller,
    profile: CallerProfile,
    key_action: Option<fn(&PublicDecision, Option<&spire_core::run::map::PublicMap>) -> Option<GameAction>>,
    policy:Option<fn(&PublicDecision,Option<&spire_core::run::map::PublicMap>,&Caller)->Result<GameAction,String>>,
) -> Result<SharedOutcome, String> {
    loop {
        let decision = run.decision().map_err(classify_run_error)?;
        let map = run.public_map();
        caller.final_decision = Some(decision.clone());
        caller.final_map = map.clone();

        if profile == CallerProfile::M13 {
            if decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3Ready) {
                return Ok(SharedOutcome::Legacy(CallerOutcome::Act3Ready));
            }
            if decision.terminal == Some(RunTerminal::Death) {
                return Ok(SharedOutcome::Legacy(CallerOutcome::Death));
            }
            if decision.terminal.is_some() {
                return Err(format!("unexpected native terminal state: {:?}", decision.terminal));
            }
        } else if profile == CallerProfile::M14 {
            if let Some(outcome) = m14_stop_outcome(&decision) {
                return Ok(SharedOutcome::Legacy(outcome));
            }
            if decision.terminal.is_some() {
                return Err(format!("unexpected native terminal state: {:?}", decision.terminal));
            }
        } else if profile==CallerProfile::M22{
            match decision.terminal{
                Some(RunTerminal::Death)=>return Ok(SharedOutcome::M22(M22CallerOutcome::Death)),
                Some(RunTerminal::EmptyDeck)=>return Ok(SharedOutcome::M22(M22CallerOutcome::EmptyDeck)),
                Some(other)=>return Err(format!("unexpected M22 terminal: {other:?}")),
                None=>(),
            }
            if decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready){
                if (decision.act,decision.floor)!=(2,17)||decision.player.hp==0||decision.objects.is_empty()||!decision.candidates.is_empty(){return Err("invalid M22 healthy cutoff".into())}
                return Ok(SharedOutcome::M22(M22CallerOutcome::Act2Ready))
            }
            if caller.accepted()>=caller.limit{return Ok(SharedOutcome::M22(M22CallerOutcome::BudgetExhausted))}
        } else if profile==CallerProfile::M23{
            match decision.terminal{
                Some(RunTerminal::Death)=>return Ok(SharedOutcome::M23(M23CallerOutcome::Death)),
                Some(RunTerminal::EmptyDeck)=>return Ok(SharedOutcome::M23(M23CallerOutcome::EmptyDeck)),
                Some(other)=>return Err(format!("unexpected M23 terminal: {other:?}")),
                None=>(),
            }
            if decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready){
                if (decision.act,decision.floor)!=(3,34)||decision.player.hp==0||decision.objects.is_empty()||!decision.candidates.is_empty()||decision.combat.is_some(){return Err("invalid M23 healthy cutoff".into())}
                return Ok(SharedOutcome::M23(M23CallerOutcome::Act3Ready))
            }
            if caller.accepted()>=caller.limit{return Ok(SharedOutcome::M23(M23CallerOutcome::BudgetExhausted))}
        } else {
            let expected=if profile==CallerProfile::M20{RoomScenario::M20DefectHeart}else{RoomScenario::M15SilentHeart};
            if let Some(outcome)=heart_stop_outcome(&decision,caller.trace.last(),expected)? {
                return Ok(SharedOutcome::M15(outcome));
            }
            if caller.accepted()>=caller.limit{return Ok(SharedOutcome::M15(M15CallerOutcome::BudgetExhausted))}
        }
        if !matches!(profile,CallerProfile::M15|CallerProfile::M20|CallerProfile::M22|CallerProfile::M23) && caller.accepted() >= caller.limit {
            return Ok(match profile {
                CallerProfile::M13 | CallerProfile::M14 => SharedOutcome::Legacy(CallerOutcome::BudgetExhausted),
                CallerProfile::M15 | CallerProfile::M20 | CallerProfile::M22 | CallerProfile::M23 => unreachable!("M15 budget handled before the shared legacy budget branch"),
            });
        }

        let action = if caller.nonattacking && decision.combat.is_some() {
            GameAction::Combat(CombatAction::EndTurn)
        } else if let Some(policy)=policy {
            policy(&decision,map.as_ref(),caller)?
        } else {
            let override_action = key_action.and_then(|policy| policy(&decision, map.as_ref()));
            if let Some(action) = override_action {
                action
            } else {
                match action_for(&decision, map.as_ref(), caller) {
                    Ok(action) => action,
                    Err(error) if profile == CallerProfile::M15 => {
                        m15_first_offered_legal(&decision).ok_or(error)?
                    }
                    Err(error) if profile == CallerProfile::M14 => {
                        return Err(format!("caller policy error at act {} floor {} in {:?}: {error}", decision.act, decision.floor, decision.room.as_ref().map(|room| room.kind)));
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        if !decision.candidates.iter().any(|candidate| candidate.action == action) {
            if profile != CallerProfile::M13 {
                caller.failed_action = Some(action.clone());
                return Err(format!("caller selected an action outside the public menu at act {} floor {}: {action:?}", decision.act, decision.floor));
            }
            return Err(format!("caller selected an action outside the public menu: {action:?}"));
        }
        let result=if matches!(profile,CallerProfile::M22|CallerProfile::M23){let observed=run.step_observed(&action);caller.m22_observed.push((observed.effects,observed.boundaries));observed.result}else{run.step(&action)};
        let transition = match result {
            Ok(transition) => transition,
            Err(error) if profile != CallerProfile::M13 => {
                if matches!(profile,CallerProfile::M20|CallerProfile::M22|CallerProfile::M23){caller.failed_error=Some(error.clone());}
                caller.failed_action = Some(action.clone());
                return Err(format!(
                    "{} at act {} floor {} while applying {action:?}",
                    classify_run_error(error), decision.act, decision.floor
                ));
            }
            Err(error) => return Err(classify_run_error(error)),
        };
        if let (GameAction::Move(id), Some(public_map)) = (&action, &map) {
            if let Some(node) = public_map.nodes.iter().find(|node| node.id == *id) {
                if (1..=2).contains(&decision.act) {
                    let act = (decision.act - 1) as usize;
                    caller.elite_reached[act] |= node.kind == spire_core::run::MapNodeKind::Elite;
                    caller.ordinary_reached[act] |= node.kind == spire_core::run::MapNodeKind::Monster;
                } else if decision.act == 3 && profile != CallerProfile::M13 {
                    caller.act3_elite_reached |= node.kind == spire_core::run::MapNodeKind::Elite;
                    caller.act3_ordinary_reached |= node.kind == spire_core::run::MapNodeKind::Monster;
                }
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
        return if caller.profile != CallerProfile::M13 {
            combat_action_with_profile(decision, combat, true)
        } else {
            combat_action(decision, combat)
        };
    }
    if decision.selection.is_some() {
        return if caller.profile != CallerProfile::M13 {
            m14_revision::room_selection_action(decision)
        } else {
            room_selection_action(decision)
        };
    }
    let room = decision.room.as_ref().ok_or("missing public room")?;
    let action = match room.kind {
        RoomViewKind::Neow => room_choice(decision, |kind| kind == &RoomChoiceKind::NeowLament),
        RoomViewKind::Map => map.and_then(|map| {
            let (elite_reached, ordinary_reached) = caller.route_progress(decision.act);
            if caller.profile != CallerProfile::M13 {
                public_routes_v2::action_m14(decision, map, elite_reached, ordinary_reached).ok()
            } else {
                map_action(decision, map, elite_reached, ordinary_reached).ok()
            }
        }),
        RoomViewKind::Rewards => room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(id) if !decision.player.relics.contains(id)))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Gold(_))))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::CardReward))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Potion(_))))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed)),
        RoomViewKind::CardReward => if caller.profile != CallerProfile::M13 {
            m14_revision::card_reward_action(decision)
        } else {
            card_reward_action(decision)
        },
        RoomViewKind::Shop => if caller.profile != CallerProfile::M13 {
            m14_revision::shop_action(decision)
        } else {
            shop_action(decision)
        },
        RoomViewKind::Campfire => campfire_action(decision),
        RoomViewKind::Event(EventId::UpgradeShrine) => room_choice(decision, |kind| kind == &RoomChoiceKind::ShrineUpgrade)
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::Purifier) => room_choice(decision, |kind| kind == &RoomChoiceKind::ShrineRemove)
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::AncientWriting) => room_choice(decision, |kind| kind == &RoomChoiceKind::AncientSimplicity)
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::Joust) => room_choice(decision, |kind| kind == &RoomChoiceKind::JoustMurderer)
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::JoustContinue))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::JoustResult { .. })))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::Falling) => if caller.profile != CallerProfile::M13 {
            m14_revision::falling_action(decision)
        } else {
            falling_action(decision)
        },
        RoomViewKind::Event(EventId::TombOfLordRedMask) => {
            if decision.player.relics.contains(&RelicId::RedMask) {
                room_choice(decision, |kind| kind == &RoomChoiceKind::TombGold)
                    .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave))
            } else {
                room_choice(decision, |kind| kind == &RoomChoiceKind::Leave)
            }
        }
        RoomViewKind::Event(_) => room_choice(decision, |kind| kind == &RoomChoiceKind::Leave),
        RoomViewKind::Treasure | RoomViewKind::BossTreasure => room_choice(decision, |kind| kind == &RoomChoiceKind::OpenChest)
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(_))))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed)),
        RoomViewKind::BossRelicChoice => room_choice(decision, |kind| kind == &RoomChoiceKind::Relic(RelicId::SlaversCollar))
            .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Relic(RelicId::EmptyCage)))
            .or_else(|| room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Relic(_)))),
        RoomViewKind::BossRelicChild => room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed),
        RoomViewKind::ProceedWithoutRelic => room_choice(decision, |kind| kind == &RoomChoiceKind::ConfirmSkip),
        RoomViewKind::Act3BossComplete => room_choice(decision, |kind| kind == &RoomChoiceKind::Proceed),
        RoomViewKind::Act3Ending => room_choice(decision, |kind| matches!(kind, RoomChoiceKind::Act3Continue { .. })),
        _ => None,
    };
    action.ok_or_else(|| format!("no public caller rule for floor {} in {room_kind:?}", decision.floor, room_kind = room.kind))
}

pub(crate) fn m15_first_offered_legal(decision: &PublicDecision) -> Option<GameAction> {
    if decision.combat.is_some() || decision.selection.is_some() {
        return None;
    }
    let room = decision.room.as_ref()?;
    room.choices.iter().filter(|choice| choice.status == RoomChoiceStatus::Offered)
        .filter(|choice| room.kind != RoomViewKind::HeartVictory || choice.kind == RoomChoiceKind::Proceed)
        .find_map(|choice| {
            let action = GameAction::Choose(choice.id);
            decision.candidates.iter().any(|candidate| candidate.id == choice.id && candidate.action == action).then_some(action)
        })
}

/// Convert only actual M15 endpoints, validating the accepted Proceed receipt for Heart victory.
pub fn m15_stop_outcome(
    decision: &PublicDecision,
    last: Option<&TraceStep>,
) -> Result<Option<M15CallerOutcome>, String> {
    if decision.content_profile != Some(RoomScenario::M15SilentHeart) {
        return Err("M15 stop check requires the M15SilentHeart profile".into());
    }
    heart_stop_outcome(decision,last,RoomScenario::M15SilentHeart)
}

fn heart_stop_outcome(decision:&PublicDecision,last:Option<&TraceStep>,expected:RoomScenario)
    ->Result<Option<M15CallerOutcome>,String>{
    if decision.content_profile != Some(expected) {
        return Err("M15 stop check requires the M15SilentHeart profile".into());
    }
    match decision.terminal {
        None => Ok(None),
        Some(RunTerminal::Death) => Ok(Some(M15CallerOutcome::Death)),
        Some(RunTerminal::Act3NoKeys) => Ok(Some(M15CallerOutcome::Act3NoKeys)),
        Some(RunTerminal::EmptyDeck) => Ok(Some(M15CallerOutcome::EmptyDeck)),
        Some(RunTerminal::HeartVictory) => {
            if decision.act != 4 || decision.floor != 55
                || !decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::TrueVictory)
            {
                return Err("HeartVictory terminal is not the M15 floor-55 TrueVictory endpoint".into());
            }
            let trace = last.ok_or("HeartVictory terminal has no accepted final Proceed trace")?;
            let before = &trace.decision;
            if before.content_profile != Some(expected)
                || before.act != 4 || before.floor != 54
                || before.terminal.is_some()
                || !before.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::HeartVictory)
            {
                return Err("HeartVictory terminal is missing the M15 floor-54 HeartVictory source decision".into());
            }
            let proceed = before.room.as_ref().and_then(|room| room.choices.iter()
                .find(|choice| choice.kind == RoomChoiceKind::Proceed && choice.status == RoomChoiceStatus::Offered))
                .ok_or("HeartVictory source decision did not offer Proceed")?;
            let expected = GameAction::Choose(proceed.id);
            if !before.candidates.iter().any(|candidate| candidate.id == proceed.id && candidate.action == expected) || trace.action != expected {
                return Err("HeartVictory terminal is not backed by the offered legal Proceed action".into());
            }
            if trace.transition.terminal != Some(RunTerminal::HeartVictory) {
                return Err("accepted Proceed transition did not terminate in HeartVictory".into());
            }
            let entered = trace.transition.committed_events.iter()
                .position(|event| *event == CommittedEvent::FloorEntered { act: 4, floor: 55 })
                .ok_or("accepted Proceed transition omitted floor-55 entry")?;
            let ended = trace.transition.committed_events.iter()
                .position(|event| *event == CommittedEvent::Ended(RunTerminal::HeartVictory))
                .ok_or("accepted Proceed transition omitted HeartVictory settlement")?;
            if entered >= ended {
                return Err("accepted Proceed transition has invalid floor/terminal event order".into());
            }
            Ok(Some(M15CallerOutcome::HeartVictory))
        }
    }
}

pub(crate) fn m15_stop_or_budget(
    decision: &PublicDecision,
    last: Option<&TraceStep>,
    accepted: usize,
    limit: usize,
) -> Result<Option<M15CallerOutcome>, String> {
    if let Some(outcome) = m15_stop_outcome(decision, last)? {
        return Ok(Some(outcome));
    }
    Ok((accepted >= limit).then_some(M15CallerOutcome::BudgetExhausted))
}

fn room_choice(decision: &PublicDecision, wanted: impl Fn(&RoomChoiceKind) -> bool) -> Option<GameAction> {
    let row = decision.room.as_ref()?.choices.iter()
        .find(|choice| choice.status == RoomChoiceStatus::Offered && wanted(&choice.kind))?;
    decision.candidates.iter().find(|candidate| candidate.action == GameAction::Choose(row.id))
        .map(|candidate| candidate.action.clone())
}

fn falling_action(decision: &PublicDecision) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    room.choices.iter()
        .filter(|choice| choice.status == RoomChoiceStatus::Offered)
        .filter_map(|choice| {
            let RoomChoiceKind::FallingLose { instance: Some(instance), .. } = &choice.kind else {
                return None;
            };
            let card = decision.objects.iter().find(|card| card.instance == *instance)?;
            Some((card_value(card.id), *instance, GameAction::Choose(choice.id)))
        })
        // Drop the lowest-value captured card; an equal value uses the
        // smaller permanent instance id, matching the public selection tie.
        .min_by_key(|(value, instance, _)| (*value, *instance))
        .map(|(_, _, action)| action)
        .or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave))
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
    ordinary_reached: bool,
) -> Result<GameAction, String> {
    public_routes_v2::action(decision, map, elite_reached, ordinary_reached)
}

fn room_selection_action(decision: &PublicDecision) -> Result<GameAction, String> {
    let selection = decision.selection.as_ref().ok_or("missing public room selection")?;
    if selection.eligible.is_empty() {
        return Err("public room selection has no eligible card".into());
    }
    let prefer_high = matches!(decision.room.as_ref().map(|room| room.kind), Some(RoomViewKind::Event(EventId::UpgradeShrine) | RoomViewKind::Campfire));
    let candidate = decision.candidates.iter().filter_map(|candidate| {
        let GameAction::Select(_) = candidate.action else { return None };
        let object = candidate.object.as_ref()?;
        let value = card_value(object.id);
        Some((if prefer_high { value } else { -value }, object.instance, candidate.action.clone()))
    }).max_by_key(|(value, instance, _)| (*value, std::cmp::Reverse(*instance)));
    candidate.map(|(_, _, action)| action).ok_or_else(|| "public room selection has no offered card".into())
}

fn combat_action(decision: &PublicDecision, combat: &spire_core::combat::CombatPublic) -> Result<GameAction, String> {
    combat_action_with_profile(decision, combat, false)
}

fn combat_action_with_profile(decision: &PublicDecision, combat: &spire_core::combat::CombatPublic, m14: bool) -> Result<GameAction, String> {
    let mut legal = decision.candidates.iter().filter_map(|candidate| match &candidate.action {
        GameAction::Combat(action) => Some(action.clone()),
        _ => None,
    }).collect::<Vec<_>>();
    if legal.is_empty() { return Err("combat has no public legal action".into()); }

    if let Some(selection) = &combat.selection {
        if m14 {
            if let Some(action) = m14_combat_choices::selection_action(combat, &legal) {
                return Ok(GameAction::Combat(action));
            }
        }
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
    if m14 {
        // Conservatively decline attacks whose published retaliation already
        // reaches current HP+block. No kill or future-block prediction is made.
        legal.retain(|action| {
            let CombatAction::PlayCard { instance, target: Some(target) } = action else { return true };
            let attack = combat.hand.iter().any(|card| card.instance.0 == *instance
                && card.values.as_ref().is_some_and(|value| value.base_damage.is_some_and(|damage| damage > 0)));
            !attack || combat.enemies.get((target.0 - 1) as usize).is_none_or(|enemy|
                m14_retaliation(enemy) < combat.player.hp.saturating_add(combat.player.block))
        });
    }
    // Published damage is current damage PER HIT. This is a pressure lower
    // bound, not a combat prediction; no private move/hit table is consulted.
    let incoming = if m14 { m14_threat::incoming(combat) } else {
        combat.enemies.iter().enumerate().filter(|(row, _)| targetable(combat, *row))
            .filter_map(|(_, enemy)| enemy.intent_damage).map(|damage| damage.max(0)).sum::<i32>()
    };
    let under_pressure = incoming > combat.player.block;
    let urgent = survival_pressure(combat, incoming);
    let ordinary_target = combat.enemies.iter().enumerate()
        .filter(|(row, _)| targetable(combat, *row))
        .min_by_key(|(_, enemy)| (enemy.hp + enemy.block, enemy.hp))
        .map(|(index, _)| TargetId(index as u64 + 1));
    let target = if urgent { threat_target(combat).or(ordinary_target) } else { ordinary_target };
    let target = if m14 { m14_preferred_target(combat, target) } else { target };
    if under_pressure {
        if let Some(action) = play_card_action(combat, &legal, CardId::Deflect, None) {
            return Ok(GameAction::Combat(action));
        }
        if m14 {
            if let Some(action) = play_card_action(combat, &legal, CardId::Neutralize, debuff_target(combat)) {
                return Ok(GameAction::Combat(action));
            }
        }
    }
    if urgent {
        let debuff_target = debuff_target(combat).or(target);
        if let Some(action) = play_card_action(combat, &legal, CardId::Neutralize, debuff_target) {
            return Ok(GameAction::Combat(action));
        }
    }
    if let Some(action) = potion_action(combat, &legal, incoming, under_pressure) {
        return Ok(GameAction::Combat(action));
    }
    if m14 {
        if under_pressure {
            if target.is_some_and(|id| combat.enemies.get((id.0 - 1) as usize)
                .is_some_and(|enemy| enemy.powers.iter().any(|power| power.id.0 == "shifting" && power.amount > 0))) {
                for id in [CardId::Shiv, CardId::BladeDance, CardId::DaggerThrow,
                    CardId::QuickSlash, CardId::PoisonedStab, CardId::StrikeGreen] {
                    if let Some(action) = play_card_action(combat, &legal, id, target) {
                        return Ok(GameAction::Combat(action));
                    }
                }
            }
        }
        if let Some(action) = m14_preparation_action(combat, &legal, incoming) {
            return Ok(GameAction::Combat(action));
        }
        if under_pressure {
            if combat.player.powers.iter().any(|power| power.id.0 == "noxious_fumes" && power.amount > 0) {
                if let Some(action) = m14_best_printed_block(combat, &legal) {
                    return Ok(GameAction::Combat(action));
                }
            }
        }
    }
    if urgent || (m14 && under_pressure) {
        // M14 also covers nonurgent current damage before optional setup.
        // Preserve the existing draw/block ordering, adding the supported
        // block-producing Cloak and Dagger before paying for setup powers.
        for id in [CardId::Backflip, CardId::Survivor, CardId::CloakAndDagger, CardId::DefendGreen] {
            if let Some(action) = play_card_action(combat, &legal, id, target) {
                return Ok(GameAction::Combat(action));
            }
        }
    }
    if urgent {
        // With legal defense exhausted, spend on current threats before setup.
        // This is a fixed priority, not a search over simulated sequences.
        for id in [CardId::Shiv, CardId::BladeDance, CardId::PoisonedStab,
            CardId::DaggerThrow, CardId::QuickSlash, CardId::StrikeGreen] {
            if let Some(action) = play_card_action(combat, &legal, id, target) {
                return Ok(GameAction::Combat(action));
            }
        }
    }
    // Outside immediate survival pressure the original priorities are intact.
    for id in [CardId::NoxiousFumes, CardId::Accuracy, CardId::WellLaidPlans, CardId::Neutralize] {
        if let Some(action) = play_card_action(combat, &legal, id, target) {
            return Ok(GameAction::Combat(action));
        }
    }
    let priorities: &[CardId] = if under_pressure {
        &[CardId::Backflip, CardId::Deflect, CardId::Survivor, CardId::DefendGreen,
            CardId::Shiv, CardId::NoxiousFumes, CardId::BladeDance, CardId::DeadlyPoison, CardId::PoisonedStab,
            CardId::Neutralize, CardId::DaggerThrow, CardId::QuickSlash, CardId::StrikeGreen]
    } else {
        &[CardId::Shiv, CardId::NoxiousFumes, CardId::BladeDance, CardId::DeadlyPoison, CardId::PoisonedStab,
            CardId::Accuracy, CardId::CloakAndDagger, CardId::Neutralize, CardId::DaggerThrow,
            CardId::QuickSlash, CardId::Backflip, CardId::StrikeGreen]
    };
    for id in priorities {
        if let Some(action) = play_card_action(combat, &legal, *id, target) {
            return Ok(GameAction::Combat(action));
        }
    }
    if let Some(action) = legal.iter().find(|action| {
        let CombatAction::PlayCard { instance, target } = action else { return false };
        combat.hand.iter().any(|card| card.instance.0 == *instance && !matches!(card.id,
            CardId::DefendGreen | CardId::Survivor | CardId::CloakAndDagger | CardId::Deflect)
            && !(m14 && card.id == CardId::Catalyst && !target.is_some_and(|id| {
                combat.enemies.get((id.0 - 1) as usize).is_some_and(|enemy|
                    enemy.powers.iter().any(|power| power.id.0 == "poison" && power.amount > 0))
            })))
    }) {
        return Ok(GameAction::Combat(action.clone()));
    }
    if legal.contains(&CombatAction::EndTurn) {
        return Ok(GameAction::Combat(CombatAction::EndTurn));
    }
    Ok(GameAction::Combat(legal[0].clone()))
}

// Current public costs reserve energy for an offered defense. This ranks the
// current decision; it does not execute the defense or predict its draw/block.
fn m14_preparation_action(
    combat: &spire_core::combat::CombatPublic,
    legal: &[CombatAction],
    incoming: i32,
) -> Option<CombatAction> {
    let gap = incoming.saturating_sub(combat.player.block).max(0);
    if gap >= combat.player.hp { return None; }
    let defense_cost = combat.hand.iter().filter(|card| {
        card.values.as_ref().is_some_and(|values| values.base_block.is_some_and(|block| block > 0))
            && legal.iter().any(|action| matches!(action, CombatAction::PlayCard { instance, .. } if *instance == card.instance.0))
    }).filter_map(|card| card.effective_cost.map(|cost| cost.max(0))).min();
    let poison_target = combat.enemies.iter().enumerate()
        .filter(|(row, _)| targetable(combat, *row))
        .filter_map(|(row, enemy)| enemy.powers.iter().find(|power| power.id.0 == "poison" && power.amount > 0)
            .map(|power| (power.amount, std::cmp::Reverse(row), TargetId(row as u64 + 1))))
        .max_by_key(|(poison, row, _)| (*poison, *row)).map(|(_, _, target)| target);
    for (id, target) in [(CardId::Catalyst, poison_target), (CardId::NoxiousFumes, None)] {
        if id == CardId::Catalyst && target.is_none() { continue; }
        for card in combat.hand.iter().filter(|card| card.id == id) {
            let Some(cost) = card.effective_cost.map(|cost| cost.max(0)) else { continue };
            let reserved = gap == 0 || defense_cost.map_or(
                i64::from(gap) * 3 < i64::from(combat.player.hp),
                |defense| cost.saturating_add(defense) <= combat.player.energy,
            );
            if !reserved { continue; }
            if id == CardId::Catalyst {
                let reserve = if gap > 0 { defense_cost.unwrap_or(0) } else { 0 };
                for source in combat.hand.iter().filter(|source| source.id == CardId::DeadlyPoison) {
                    let Some(source_cost) = source.effective_cost else { continue };
                    if cost.saturating_add(source_cost.max(0)).saturating_add(reserve) > combat.player.energy { continue; }
                    if let Some(action) = legal.iter().find(|action| matches!(action,
                        CombatAction::PlayCard { instance, target: chosen }
                            if *instance == source.instance.0 && *chosen == target)) {
                        return Some(action.clone());
                    }
                }
            }
            if let Some(action) = legal.iter().find(|action| matches!(action,
                CombatAction::PlayCard { instance, target: chosen }
                    if *instance == card.instance.0 && *chosen == target)) {
                return Some(action.clone());
            }
        }
    }
    None
}

fn m14_best_printed_block(
    combat: &spire_core::combat::CombatPublic,
    legal: &[CombatAction],
) -> Option<CombatAction> {
    combat.hand.iter().filter_map(|card| {
        let block = card.values.as_ref()?.base_block.filter(|block| *block > 0)?;
        let cost = card.effective_cost?.max(0);
        let action = legal.iter().find(|action| matches!(action,
            CombatAction::PlayCard { instance, .. } if *instance == card.instance.0))?;
        Some(((cost == 0, i64::from(block) * 1024 / i64::from(cost.max(1)),
            card_value(card.id), std::cmp::Reverse(card.instance.0)), action.clone()))
    }).max_by_key(|(key, _)| *key).map(|(_, action)| action)
}

fn m14_retaliation(enemy: &spire_core::combat::FullEnemyView) -> i32 {
    enemy.powers.iter().filter(|power| matches!(power.id.0, "thorns" | "sharp_hide"))
        .map(|power| power.amount.max(0)).fold(0, i32::saturating_add)
}

fn m14_preferred_target(
    combat: &spire_core::combat::CombatPublic,
    original: Option<TargetId>,
) -> Option<TargetId> {
    let hazard = |enemy: &spire_core::combat::FullEnemyView| (
        enemy.powers.iter().any(|power| power.id.0 == "intangible" && power.amount > 0),
        m14_retaliation(enemy),
    );
    let best = combat.enemies.iter().enumerate().filter(|(row, _)| targetable(combat, *row))
        .min_by_key(|(row, enemy)| (hazard(enemy), enemy.hp.saturating_add(enemy.block), enemy.hp, *row))?;
    if original.is_some_and(|target| combat.enemies.get((target.0 - 1) as usize)
        .is_some_and(|enemy| hazard(enemy) == hazard(best.1))) {
        original
    } else {
        Some(TargetId(best.0 as u64 + 1))
    }
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

fn targetable(combat: &spire_core::combat::CombatPublic, row: usize) -> bool {
    combat.targets.get(row).is_some_and(|(id, active)| *active && *id == TargetId(row as u64 + 1))
        && combat.enemies.get(row).is_some_and(|enemy| enemy.hp > 0 && !enemy.replaced && !enemy.escaped && !enemy.half_dead)
}

fn survival_pressure(combat: &spire_core::combat::CombatPublic, incoming: i32) -> bool {
    let gap = (incoming - combat.player.block).max(0);
    gap > 0 && (combat.player.hp * 2 <= combat.player.max_hp
        || gap >= (combat.player.hp / 3).max(1)
        || combat.player.powers.iter().any(|power| power.id.0 == "vulnerable" && power.amount > 0))
}

fn threat_target(combat: &spire_core::combat::CombatPublic) -> Option<TargetId> {
    combat.enemies.iter().enumerate().filter(|(row, _)| targetable(combat, *row))
        .max_by_key(|(row, enemy)| {
            let threat = i64::from(enemy.intent_damage.unwrap_or(0).max(0));
            let durability = i64::from(enemy.hp.saturating_add(enemy.block).max(1));
            ((threat + 1) * 1024 / durability, threat, std::cmp::Reverse(*row))
        }).map(|(row, _)| TargetId(row as u64 + 1))
}

fn debuff_target(combat: &spire_core::combat::CombatPublic) -> Option<TargetId> {
    combat.enemies.iter().enumerate().filter(|(row, enemy)| targetable(combat, *row)
        && enemy.intent_damage.is_some_and(|damage| damage > 0)
        && !enemy.powers.iter().any(|power| matches!(power.id.0, "artifact" | "weak") && power.amount > 0))
        .max_by_key(|(row, enemy)| (enemy.intent_damage.unwrap_or(0), std::cmp::Reverse(*row)))
        .map(|(row, _)| TargetId(row as u64 + 1))
}

fn potion_action(
    combat: &spire_core::combat::CombatPublic,
    legal: &[CombatAction],
    incoming: i32,
    under_pressure: bool,
) -> Option<CombatAction> {
    let inventory = combat.inventory.as_ref()?;
    let urgent = survival_pressure(combat, incoming);
    let attack_offered = combat.hand.iter().any(|card| {
        (card.values.as_ref().is_some_and(|value| value.base_damage.is_some_and(|damage| damage > 0))
            || card.id == CardId::BladeDance)
            && legal.iter().any(|action| matches!(action, CombatAction::PlayCard { instance, .. } if *instance == card.instance.0))
    });
    let defense_offered = combat.hand.iter().any(|card| card.values.as_ref()
        .is_some_and(|value| value.base_block.is_some_and(|block| block > 0))
        && legal.iter().any(|action| matches!(action, CombatAction::PlayCard { instance, .. } if *instance == card.instance.0)));
    for (slot, potion) in inventory.potions.iter().enumerate() {
        let (use_now, target) = match potion {
            Some(PotionId::WeakPotion) => {
                let target = debuff_target(combat);
                let threat = target.and_then(|id| combat.enemies.get((id.0 - 1) as usize))
                    .and_then(|enemy| enemy.intent_damage).unwrap_or(0);
                (under_pressure && (urgent || threat >= 10), target)
            }
            Some(PotionId::FirePotion) => {
                // Twenty is the ordinary Fire Potion's printed damage. The
                // public HP+block comparison is a targeting heuristic, not a
                // guarantee of a kill through every enemy modifier.
                let finish = combat.enemies.iter().enumerate()
                    .filter(|(row, enemy)| targetable(combat, *row) && enemy.hp.saturating_add(enemy.block) <= 20)
                    .max_by_key(|(row, enemy)| (enemy.intent_damage.unwrap_or(0), std::cmp::Reverse(*row)))
                    .map(|(row, _)| TargetId(row as u64 + 1));
                let target = finish.or_else(|| threat_target(combat));
                let attacking_finish = finish.is_some_and(|id| combat.enemies[(id.0 - 1) as usize].intent_damage.unwrap_or(0) > 0);
                (attacking_finish || urgent, target)
            }
            Some(PotionId::DexterityPotion | PotionId::BlockPotion) =>
                (under_pressure && (urgent || incoming >= 10) && defense_offered, None),
            Some(PotionId::StrengthPotion) => {
                let remaining_risk = combat.enemies.iter().enumerate()
                    .filter(|(row, _)| targetable(combat, *row))
                    .map(|(_, enemy)| enemy.hp.saturating_add(enemy.block)).sum::<i32>();
                (attack_offered && (!urgent || !defense_offered)
                    && (under_pressure || remaining_risk >= combat.player.hp), None)
            }
            _ => (false, None),
        };
        if !use_now { continue; }
        if let Some(action) = legal.iter().find(|action| matches!(action,
            CombatAction::UsePotion { slot: offered, target: selected }
                if *offered == slot && *selected == target)) {
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
                    if matches!(encounter, EncounterId::Guardian | EncounterId::Hexaghost | EncounterId::SlimeBoss | EncounterId::Champ | EncounterId::BronzeAutomaton | EncounterId::Collector)
                        || caller.profile != CallerProfile::M13 && matches!(encounter, EncounterId::AwakenedOne | EncounterId::TimeEater | EncounterId::DonuAndDeca) => {
                        summary.boss_combat_entries.push(*encounter);
                    }
                CommittedEvent::CombatFinished(record)
                    if matches!(record.encounter, EncounterId::Guardian | EncounterId::Hexaghost | EncounterId::SlimeBoss | EncounterId::Champ | EncounterId::BronzeAutomaton | EncounterId::Collector)
                        || caller.profile != CallerProfile::M13 && matches!(record.encounter, EncounterId::AwakenedOne | EncounterId::TimeEater | EncounterId::DonuAndDeca) => {
                        summary.boss_combat_records.push((record.encounter, record.outcome));
                    }
                _ => (),
            }
        }
    }
    summary
}

#[cfg(test)]
mod policy_revision_tests;

#[cfg(test)]
mod m14_revision_tests;

#[cfg(test)]
mod m14_acquisition_tests;

#[cfg(test)]
mod m14_selection_tests;

#[cfg(test)]
mod m14_hazard_tests;
