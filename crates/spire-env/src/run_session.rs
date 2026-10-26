//! One authoritative native Run, publication tokens, reset and retained fault/record ownership.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum RunSessionError {
    Input(&'static str),StaleToken,NotReset,Terminal,Closed,
    Core(spire_core::run::RunError),
    Capacity{group:&'static str,required:usize,capacity:usize},
    UnsupportedBoundary(&'static str),DeliveryFault(&'static str),IncompatibleReplay(&'static str),
}
use spire_core::{combat::rng::Partition,content::Character,run::*};
use crate::{RunCapacities,RunObservation};
pub const RUN_RECORD_SCHEMA_ID: &str = "spire-gym-run-replay-v1";
pub const RUN_RECORD_SCHEMA_ID_POLICY: &str = "spire-gym-run-replay-v3";
pub const RUN_RECORD_SCHEMA_ID_M10: &str = "spire-gym-run-replay-v2";
pub const RUN_RECORD_SCHEMA_ID_M12: &str = "spire-gym-m12-silent-act1-replay-v1";
pub const RUN_RECORD_SCHEMA_ID_M13: &str = "spire-gym-m13-silent-act2-replay-v1";
pub const RUN_RECORD_SCHEMA_ID_M14: &str = "spire-gym-m14-silent-act3-replay-v1";
pub const RUN_SEED_ADAPTER_ID: &str = "spire-gym-run-u64-le-v1";
pub const RUN_RULES_ID_V1: &str = "spire-gym-native-m9-rules-v1";
pub const RUN_RULES_ID: &str = "spire-gym-native-m9-rules-v2";
pub(crate) const RUN_CONTENT_ID: &str = "spire-gym-m9-ironclad-a0-captured-v1";
pub const RUN_RULES_ID_M10: &str = "spire-gym-native-m10-rules-v1";
pub const RUN_CONTENT_ID_M10: &str = "spire-gym-m10-ironclad-a0-captured-v1";
pub const RUN_RULES_ID_M12: &str = "spire-gym-native-m12-rules-v1";
pub const RUN_CONTENT_ID_M12: &str = "spire-gym-m12-silent-a0-act1-captured-v1";
pub const RUN_RULES_ID_M13: &str = "spire-gym-native-m13-rules-v1";
pub const RUN_CONTENT_ID_M13: &str = "spire-gym-m13-silent-a0-acts1-2-captured-v1";
pub const RUN_RULES_ID_M14: &str = "spire-gym-native-m14-rules-v1";
pub const RUN_CONTENT_ID_M14: &str = "spire-gym-m14-silent-a0-acts1-3-captured-v1";
pub(crate) const RUN_REWARD_ID: &str = "spire-gym-floor-key-terminal-reward-v1";
static SESSION_IDS: crate::identity::PoolIdAllocator = crate::identity::PoolIdAllocator::new(1);
/// All owners of RunDecisionToken share one checked transport namespace.
/// These IDs never participate in gameplay RNG, actor features or replay.
pub(crate) fn allocate_run_session_id() -> Result<u64, RunSessionError> {
    SESSION_IDS.allocate().map_err(|_| RunSessionError::DeliveryFault("Run session identity exhausted"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunProfile {
    M9IroncladAct3,
    M10IroncladHeart,
    M10IroncladPolicy,
    M12SilentAct1,
    M13SilentAct2,
    M14SilentAct3,
}
impl RunProfile {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::M9IroncladAct3 => "M9IroncladAct3",
            Self::M10IroncladHeart => "M10IroncladHeart",
            Self::M10IroncladPolicy => "M10IroncladPolicy",
            Self::M12SilentAct1 => "M12SilentAct1",
            Self::M13SilentAct2 => "M13SilentAct2",
            Self::M14SilentAct3 => "M14SilentAct3",
        }
    }
    pub(crate) fn schema_id(self) -> &'static str {
        match self {
            Self::M9IroncladAct3 => RUN_RECORD_SCHEMA_ID,
            Self::M10IroncladHeart => RUN_RECORD_SCHEMA_ID_M10,
            Self::M10IroncladPolicy => RUN_RECORD_SCHEMA_ID_POLICY,
            Self::M12SilentAct1 => RUN_RECORD_SCHEMA_ID_M12,
            Self::M13SilentAct2 => RUN_RECORD_SCHEMA_ID_M13,
            Self::M14SilentAct3 => RUN_RECORD_SCHEMA_ID_M14,
        }
    }
    pub(crate) fn rules_id(self) -> &'static str {
        match self {
            Self::M9IroncladAct3 => RUN_RULES_ID,
            Self::M10IroncladHeart | Self::M10IroncladPolicy => RUN_RULES_ID_M10,
            Self::M12SilentAct1 => RUN_RULES_ID_M12,
            Self::M13SilentAct2 => RUN_RULES_ID_M13,
            Self::M14SilentAct3 => RUN_RULES_ID_M14,
        }
    }
    pub(crate) fn content_id(self) -> &'static str {
        match self {
            Self::M9IroncladAct3 => RUN_CONTENT_ID,
            Self::M10IroncladHeart | Self::M10IroncladPolicy => RUN_CONTENT_ID_M10,
            Self::M12SilentAct1 => RUN_CONTENT_ID_M12,
            Self::M13SilentAct2 => RUN_CONTENT_ID_M13,
            Self::M14SilentAct3 => RUN_CONTENT_ID_M14,
        }
    }
    pub(crate) fn observation_spec(
        self,
        caps: &RunCapacities,
    ) -> Result<crate::RunObservationSpec, RunSessionError> {
        match self {
            Self::M9IroncladAct3 => crate::run_observation_spec(caps),
            Self::M10IroncladHeart => crate::m10_run_observation_spec(caps),
            Self::M10IroncladPolicy => crate::policy_run_observation_spec(caps),
            Self::M12SilentAct1 => crate::m12_run_observation_spec(caps),
            Self::M13SilentAct2 => crate::m13_run_observation_spec(caps),
            Self::M14SilentAct3 => crate::m14_run_observation_spec(caps),
        }
    }
    fn validate(self, config: &RunSessionConfig) -> Result<(), RunSessionError> {
        let expected_character = if matches!(self, Self::M12SilentAct1 | Self::M13SilentAct2 | Self::M14SilentAct3) {
            Character::Silent
        } else {
            Character::Ironclad
        };
        if config.character != expected_character || config.ascension != 0 {
            return Err(RunSessionError::Input(match self {
                Self::M9IroncladAct3 => "M9 requires Ironclad A0",
                Self::M10IroncladHeart | Self::M10IroncladPolicy => "M10 requires Ironclad A0",
                Self::M12SilentAct1 => "M12 requires Silent A0",
                Self::M13SilentAct2 => "M13 requires Silent A0",
                Self::M14SilentAct3 => "M14 requires Silent A0",
            }));
        }
        let history_matches = match self {
            Self::M12SilentAct1 => matches!(
                config.boss_history,
                RunBossHistory::Act1GuardianUnseen
                    | RunBossHistory::Act1HexaghostUnseen
                    | RunBossHistory::Act1SlimeBossUnseen
            ),
            Self::M13SilentAct2 => matches!(
                config.boss_history,
                RunBossHistory::M13SilentAct2 { .. }
            ),
            Self::M14SilentAct3 => matches!(
                config.boss_history,
                RunBossHistory::M14SilentAct3 { .. }
            ),
            _ => matches!(
                config.boss_history,
                RunBossHistory::AwakenedUnseen
                    | RunBossHistory::TimeEaterUnseen
                    | RunBossHistory::DonuDecaUnseen
            ),
        };
        if !history_matches {
            return Err(RunSessionError::Input("boss history does not match Run profile"));
        }
        match self {
            Self::M9IroncladAct3 => crate::run_schema::validate(&config.capacities),
            Self::M10IroncladHeart | Self::M10IroncladPolicy => {
                crate::run_schema_m10::validate(&config.capacities)
            }
            Self::M12SilentAct1 => crate::run_schema_m12::validate(&config.capacities),
            Self::M13SilentAct2 => crate::run_schema_m13::validate(&config.capacities),
            Self::M14SilentAct3 => crate::run_schema_m14::validate(&config.capacities),
        }
    }
    pub(crate) fn from_record(schema: &str, profile: &str) -> Result<Self, RunSessionError> {
        match (schema, profile) {
            (RUN_RECORD_SCHEMA_ID, "M9IroncladAct3") => Ok(Self::M9IroncladAct3),
            (RUN_RECORD_SCHEMA_ID_M10, "M10IroncladHeart") => Ok(Self::M10IroncladHeart),
            (RUN_RECORD_SCHEMA_ID_POLICY, "M10IroncladPolicy") => Ok(Self::M10IroncladPolicy),
            (RUN_RECORD_SCHEMA_ID_M12, "M12SilentAct1") => Ok(Self::M12SilentAct1),
            (RUN_RECORD_SCHEMA_ID_M13, "M13SilentAct2") => Ok(Self::M13SilentAct2),
            (RUN_RECORD_SCHEMA_ID_M14, "M14SilentAct3") => Ok(Self::M14SilentAct3),
            _ => Err(RunSessionError::IncompatibleReplay("Run identity mismatch")),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunAct1BossTarget {
    Guardian,
    Hexaghost,
    SlimeBoss,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunAct2BossTarget {
    Champ,
    BronzeAutomaton,
    Collector,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunAct3BossTarget {
    AwakenedOne,
    TimeEater,
    DonuAndDeca,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunBossHistory {
    AwakenedUnseen,
    TimeEaterUnseen,
    DonuDecaUnseen,
    Act1GuardianUnseen,
    Act1HexaghostUnseen,
    Act1SlimeBossUnseen,
    M13SilentAct2 { act1: RunAct1BossTarget, act2: RunAct2BossTarget },
    M14SilentAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
}
pub type M9BossHistory = RunBossHistory;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSessionConfig {
    pub character: Character,
    pub ascension: u8,
    pub partition: Partition,
    pub boss_history: RunBossHistory,
    pub capacities: RunCapacities,
    pub record_replay: bool,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RunDecisionToken(pub(crate) [u64;2]);
impl From<RunDecisionToken> for [u64;2]{fn from(t:RunDecisionToken)->Self{t.0}}
impl From<[u64;2]> for RunDecisionToken{fn from(value:[u64;2])->Self{Self(value)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RunActionIndex(pub i64);
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunBoundary{pub observation:RunObservation,pub token:RunDecisionToken,pub terminal:Option<RunTerminal>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunStepResult{pub boundary:RunBoundary,pub reward:i32,pub terminated:bool,pub truncated:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct RunSessionStepFailure{pub error:RunSessionError,pub committed:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunRecordedStep{pub action:GameAction,pub reward:i32,pub terminal:Option<RunTerminal>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunRecordedFault{pub action:Option<GameAction>,pub committed:bool,pub gameplay_recorded:bool,pub error:RunRecordError}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RunRecordErrorKind{Input,StaleToken,NotReset,Terminal,Closed,Core,Capacity,UnsupportedBoundary,DeliveryFault,IncompatibleReplay}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCapacityFailure{pub group:String,pub required:usize,pub capacity:usize}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordError{pub kind:RunRecordErrorKind,pub message:String,pub capacity:Option<RunCapacityFailure>}
impl RunRecordError{
    pub fn diagnostic_json(&self)->Result<String,RunSessionError>{serde_json::to_string(self).map_err(|_|RunSessionError::DeliveryFault("Run diagnostic export failed"))}
    pub(crate) fn from_error(error:&RunSessionError)->Self{
        use RunRecordErrorKind as K;
        let (kind,message,capacity)=match error{
            RunSessionError::Input(s)=>(K::Input,(*s).to_owned(),None),RunSessionError::StaleToken=>(K::StaleToken,"stale token".into(),None),
            RunSessionError::NotReset=>(K::NotReset,"not reset".into(),None),RunSessionError::Terminal=>(K::Terminal,"terminal".into(),None),RunSessionError::Closed=>(K::Closed,"closed".into(),None),
            RunSessionError::Core(e)=>{let message=match e{RunError::Faulted(f)=>format!("{}: {}",f.checkpoint,match &f.kind{RunFaultKind::UnsupportedBoundary(s)|RunFaultKind::Invariant(s)=>*s,RunFaultKind::Combat(_)=>"combat fault (full owner retained in reproduction)"}),_=>format!("{e:?}")};(K::Core,message,None)},
            RunSessionError::Capacity{group,required,capacity}=>(K::Capacity,"Run capacity exceeded".into(),Some(RunCapacityFailure{group:(*group).into(),required:*required,capacity:*capacity})),
            RunSessionError::UnsupportedBoundary(s)=>(K::UnsupportedBoundary,(*s).into(),None),RunSessionError::DeliveryFault(s)=>(K::DeliveryFault,(*s).into(),None),RunSessionError::IncompatibleReplay(s)=>(K::IncompatibleReplay,(*s).into(),None),
        };Self{kind,message,capacity}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunEpisodeRecord{
    pub schema_id:String,pub profile_id:String,pub rules_id:String,pub content_id:String,pub seed_adapter_id:String,pub rng_id:String,pub reward_id:String,
    pub observation_fingerprint:[u8;32],pub config:RunSessionConfig,pub seed:u64,pub ordinal:u64,
    pub steps:Vec<RunRecordedStep>,pub terminal:Option<RunTerminal>,pub fault:Option<RunRecordedFault>,pub closed:bool,
}
/// Explicitly privileged diagnostics, never ordinary observation/info fields.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunReproduction{
    pub config:RunSessionConfig,pub seed:Option<u64>,pub ordinal:u64,
    pub latest_decision:Option<Result<PublicDecision,RunError>>,pub latest_map:Option<spire_core::run::map::PublicMap>,
    pub last_accepted_step:Option<RunRecordedStep>,pub fault:Option<RunSessionError>,pub record:Option<RunEpisodeRecord>,pub closed:bool,
}
pub struct RunSession{
    config:RunSessionConfig,profile:RunProfile,id:u64,serial:u64,run:Option<Run>,published:Option<RunBoundary>,
    actions:Vec<GameAction>,seed:Option<u64>,ordinal:u64,record:Option<RunEpisodeRecord>,
    fault:Option<RunSessionError>,last_step:Option<RunRecordedStep>,closed:bool,at_m12_cutoff:bool,at_m13_cutoff:bool,at_m14_cutoff:bool,
}
pub(crate) fn create_run(
    profile: RunProfile,
    config: &RunSessionConfig,
    seed: u64,
    ordinal: u64,
) -> Result<Run, RunSessionError> {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    creation.seen_bosses[0] = match config.boss_history {
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::SlimeBoss, .. } => {
            vec!["Guardian", "Hexaghost"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::SlimeBoss, .. } => {
            vec!["Guardian", "Hexaghost"]
        }
        RunBossHistory::Act1GuardianUnseen => vec!["Hexaghost", "SlimeBoss"],
        RunBossHistory::Act1HexaghostUnseen => vec!["Guardian", "SlimeBoss"],
        RunBossHistory::Act1SlimeBossUnseen => vec!["Guardian", "Hexaghost"],
        _ => vec!["Hexaghost", "SlimeBoss"],
    };

    if matches!(profile, RunProfile::M13SilentAct2 | RunProfile::M14SilentAct3) {
        creation.seen_bosses[1] = match config.boss_history {
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::Collector, .. } => {
                vec!["Champ", "Automaton"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::Collector, .. } => {
                vec!["Champ", "Automaton"]
            }
            _ => unreachable!("M13 validation requires a composite boss history"),
        };
        // M13 deliberately keeps CreationProfile::experienced() Act3 history.
    }
    if profile == RunProfile::M14SilentAct3 {
        creation.seen_bosses[2] = match config.boss_history {
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::AwakenedOne, .. } => {
                vec!["TimeEater", "DonuAndDeca"]
            }
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::TimeEater, .. } => {
                vec!["AwakenedOne", "DonuAndDeca"]
            }
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::DonuAndDeca, .. } => {
                vec!["AwakenedOne", "TimeEater"]
            }
            _ => unreachable!("M14 validation requires a triple boss history"),
        };
    } else if profile != RunProfile::M12SilentAct1 && profile != RunProfile::M13SilentAct2 {
        creation.seen_bosses[1] = vec!["Champ", "Collector"];
        creation.seen_bosses[2] = match config.boss_history {
            RunBossHistory::AwakenedUnseen => vec!["TimeEater", "DonuAndDeca"],
            RunBossHistory::TimeEaterUnseen => vec!["AwakenedOne", "DonuAndDeca"],
            RunBossHistory::DonuDecaUnseen => vec!["AwakenedOne", "TimeEater"],
            _ => vec!["TimeEater", "DonuAndDeca"],
        };
    }

    let scenario = match profile {
        RunProfile::M9IroncladAct3 => RoomScenario::M9IroncladAct3,
        RunProfile::M10IroncladHeart | RunProfile::M10IroncladPolicy => {
            RoomScenario::M10IroncladHeart
        }
        RunProfile::M12SilentAct1 => RoomScenario::M12SilentAct1,
        RunProfile::M13SilentAct2 => RoomScenario::M13SilentAct2,
        RunProfile::M14SilentAct3 => RoomScenario::M14SilentAct3,
    };
    Run::new_room_scenario(
        RunConfig { character: config.character, ascension: config.ascension },
        creation,
        RunRng::new(root, config.partition, ordinal),
        scenario,
    )
    .map_err(RunSessionError::Core)
}
pub(crate) fn validate_config(profile:RunProfile,config:&RunSessionConfig)->Result<(),RunSessionError>{profile.validate(config)}
fn check_boundary(profile: RunProfile, decision: &PublicDecision) -> Result<(), RunSessionError> {
    if profile == RunProfile::M9IroncladAct3
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act4Ready)
    {
        return Err(RunSessionError::UnsupportedBoundary("M9 stops before the Act4 door"));
    }
    if profile == RunProfile::M12SilentAct1
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M12 Act2Ready must be nonterminal with no actions",
        ));
    }
    if profile == RunProfile::M13SilentAct2
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M13 Act3Ready must be nonterminal with no actions",
        ));
    }
    if profile == RunProfile::M14SilentAct3
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act4Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M14 Act4Ready must be nonterminal with no actions",
        ));
    }
    if decision.terminal.is_none() && decision.candidates.is_empty() {
        return Err(RunSessionError::UnsupportedBoundary(
            "nonterminal Run has no legal actions",
        ));
    }
    Ok(())
}

fn publish(
    profile: RunProfile,
    run: &Run,
    config: &RunSessionConfig,
    token: RunDecisionToken,
) -> Result<(RunBoundary, Vec<GameAction>, bool, bool, bool), RunSessionError> {
    let decision = run.decision().map_err(RunSessionError::Core)?;
    check_boundary(profile, &decision)?;
    let map = run.public_map();
    let observation = match profile {
        RunProfile::M9IroncladAct3 => crate::encode_run(&decision, map.as_ref(), &config.capacities)?,
        RunProfile::M10IroncladHeart => {
            crate::encode_run_m10(&decision, map.as_ref(), &config.capacities)?
        }
        RunProfile::M10IroncladPolicy => {
            crate::encode_run_policy(&decision, map.as_ref(), &config.capacities)?
        }
        RunProfile::M12SilentAct1 => {
            crate::encode_run_m12(&decision, map.as_ref(), &config.capacities)?
        }
        RunProfile::M13SilentAct2 => {
            crate::encode_run_m13(&decision, map.as_ref(), &config.capacities)?
        }
        RunProfile::M14SilentAct3 => {
            crate::encode_run_m14(&decision, map.as_ref(), &config.capacities)?
        }
    };
    let at_m12_cutoff = profile == RunProfile::M12SilentAct1
        && decision.terminal.is_none()
        && decision.candidates.is_empty()
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready);
    let at_m13_cutoff = profile == RunProfile::M13SilentAct2
        && decision.terminal.is_none()
        && decision.candidates.is_empty()
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3Ready);
    let at_m14_cutoff = profile == RunProfile::M14SilentAct3
        && decision.terminal.is_none()
        && decision.candidates.is_empty()
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act4Ready);
    let mut actions = Vec::new();
    actions.try_reserve_exact(decision.candidates.len())
        .map_err(|_| RunSessionError::DeliveryFault("Run candidate allocation failed"))?;
    actions.extend(decision.candidates.into_iter().map(|candidate| candidate.action));
    Ok((
        RunBoundary { observation, token, terminal: decision.terminal },
        actions,
        at_m12_cutoff,
        at_m13_cutoff,
        at_m14_cutoff,
    ))
}
fn new_record(profile:RunProfile,config:&RunSessionConfig,seed:u64,ordinal:u64)->Result<RunEpisodeRecord,RunSessionError>{
    Ok(RunEpisodeRecord{schema_id:profile.schema_id().into(),profile_id:profile.id().into(),rules_id:profile.rules_id().into(),content_id:profile.content_id().into(),seed_adapter_id:RUN_SEED_ADAPTER_ID.into(),rng_id:spire_core::combat::rng::RNG_CONTRACT_ID.into(),reward_id:RUN_REWARD_ID.into(),observation_fingerprint:profile.observation_spec(&config.capacities)?.fingerprint(),config:config.clone(),seed,ordinal,steps:vec![],terminal:None,fault:None,closed:false})
}
impl RunSession{
    /// Additive public provenance: no replay, seed, cursor or private Note.
    pub fn environment_spec(&self)->Result<serde_json::Value,RunSessionError>{
        let spec=self.observation_spec()?;
        let mut descriptor=serde_json::json!({"profile_id":self.profile.id(),"schema_id":spec.descriptor()["schema_id"],
            "observation_fingerprint":crate::combat_policy::hex(&spec.fingerprint()),"rules_id":self.profile.rules_id(),
            "content_id":self.profile.content_id(),"reward_id":RUN_REWARD_ID,"seed_adapter_id":RUN_SEED_ADAPTER_ID,
            "rng_id":spire_core::combat::rng::RNG_CONTRACT_ID,"combat_view_id":if matches!(self.profile,RunProfile::M12SilentAct1|RunProfile::M13SilentAct2|RunProfile::M14SilentAct3){serde_json::Value::Null}else{serde_json::json!(crate::COMBAT_POLICY_VIEW_ID)},
            "mode":"normal_run","partition":self.config.partition.as_str(),"autoreset":"disabled"});
        if self.profile==RunProfile::M12SilentAct1{
            descriptor["finite_content"]=serde_json::json!({
                "permanent_cards":spire_core::run::M12_SILENT_ACT1_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "generated_and_status_cards":["shiv","dazed","burn","slimed"],
                "relics":spire_core::run::M12_SILENT_ACT1_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "potions":spire_core::run::M12_SILENT_ACT1_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_enemies":crate::run_schema_m12::M12_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_encounters":crate::run_schema_m12::M12_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons":crate::run_schema_m12::M12_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
            });
            descriptor["cutoff"]=serde_json::json!("Act2Ready: nonterminal, zero actions, no completion reward");
        }
        if self.profile == RunProfile::M13SilentAct2 {
            descriptor["finite_content"] = serde_json::json!({
                "permanent_cards": spire_core::run::M12_SILENT_ACT1_CARDS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "generated_and_status_cards": ["shiv", "dazed", "burn", "slimed", "wound"],
                "relics": spire_core::run::M12_SILENT_ACT1_RELICS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "potions": spire_core::run::M12_SILENT_ACT1_POTIONS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "playable_enemies": crate::run_schema_m13::M13_PLAYABLE_ENEMIES.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "playable_encounters": crate::run_schema_m13::M13_PLAYABLE_ENCOUNTERS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons": crate::run_schema_m13::M13_MAP_ONLY_BOSS_ICONS.iter().map(|id| id.as_str()).collect::<Vec<_>>()
            });
            descriptor["cutoff"] = serde_json::json!("Act3Ready: nonterminal, zero actions, no completion reward");
        }
        if self.profile == RunProfile::M14SilentAct3 {
            descriptor["finite_content"] = serde_json::json!({
                "permanent_cards": spire_core::run::M14_SILENT_ACT3_CARDS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "generated_and_status_cards": ["shiv", "dazed", "burn", "slimed", "wound", "void"],
                "relics": spire_core::run::M14_SILENT_ACT3_RELICS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "potions": spire_core::run::M14_SILENT_ACT3_POTIONS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "playable_enemies": crate::run_schema_m13::M13_PLAYABLE_ENEMIES.iter()
                    .chain(crate::run_schema_m14::M14_ACT3_ENEMIES.iter().filter(|id| !crate::run_schema_m13::M13_PLAYABLE_ENEMIES.contains(id)))
                    .map(|id| id.as_str()).collect::<Vec<_>>(),
                "act3_enemy_species": crate::run_schema_m14::M14_ACT3_ENEMIES.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "playable_encounters": crate::run_schema_m13::M13_PLAYABLE_ENCOUNTERS.iter().chain(crate::run_schema_m14::M14_ACT3_ENCOUNTERS.iter()).map(|id| id.as_str()).collect::<Vec<_>>(),
                "act3_encounters": crate::run_schema_m14::M14_ACT3_ENCOUNTERS.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons": []
            });
            descriptor["cutoff"] = serde_json::json!({
                "normal_completion":"Act3NoKeys after actual Victory and Proceed/dialog; no Heart reward",
                "diagnostic":"Act4Ready is supplied-only, healthy, nonterminal, zero-action and stops before the Act4 door"
            });
        }
        Ok(descriptor)
    }
    pub fn combat_observation_spec(&self)->Result<crate::RunObservationSpec,RunSessionError>{
        if self.profile==RunProfile::M12SilentAct1{return Err(RunSessionError::UnsupportedBoundary("M12 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M13SilentAct2{return Err(RunSessionError::UnsupportedBoundary("M13 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M14SilentAct3{return Err(RunSessionError::UnsupportedBoundary("M14 does not provide the Ironclad combat policy sidecar"))}
        crate::combat_policy_observation_spec()
    }
    /// Owned public sidecar. Dispatch uses the same token/index as the full Run.
    /// Resources are current; snapshot on the settled-count transition. The
    /// visit-keyed last_exit remains sticky across later noncombat decisions.
    pub fn combat_view(&mut self)->Result<crate::CombatPolicyBoundary,RunSessionError>{
        if self.profile==RunProfile::M12SilentAct1{return Err(RunSessionError::UnsupportedBoundary("M12 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M13SilentAct2{return Err(RunSessionError::UnsupportedBoundary("M13 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M14SilentAct3{return Err(RunSessionError::UnsupportedBoundary("M14 does not provide the Ironclad combat policy sidecar"))}
        self.usable()?;let token=self.published.as_ref().ok_or(RunSessionError::NotReset)?.token;
        let run=self.run.as_ref().unwrap();let d=run.decision().map_err(RunSessionError::Core)?;let cp=run.combat_checkpoint();
        let observation=match crate::combat_policy::encode_combat_policy(&d,cp.as_ref()){
            Ok(o)=>o,Err(e)=>return Err(self.freeze(e.clone(),e,None,false,false)),
        };
        Ok(crate::CombatPolicyBoundary{observation,token,info:crate::combat_policy::lifecycle(&d,cp.as_ref())})
    }
    /// Transport-only Gym seeding metadata, never an observation feature.
    pub fn effective_seed(&self)->Option<u64>{self.seed}
    pub fn new(config:RunSessionConfig)->Result<Self,RunSessionError>{
        Self::with_profile(config,RunProfile::M9IroncladAct3)
    }
    /// Opt into the captured M10 Ironclad Heart Run profile on the same native session owner.
    /// The existing constructor and public configuration remain M9-compatible.
    pub fn new_m10(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M10IroncladHeart)}
    pub fn new_m10_policy(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M10IroncladPolicy)}
    /// Opt into the finite Silent A0 captured Act1 profile on the shared session owner.
    pub fn new_m12(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M12SilentAct1)}
    /// Opt into the finite Silent A0 captured Acts 1–2 profile on the shared session owner.
    pub fn new_m13(config: RunSessionConfig) -> Result<Self, RunSessionError> {
        Self::with_profile(config, RunProfile::M13SilentAct2)
    }
    /// Opt into the finite Silent A0 captured Acts 1–3 profile on the shared session owner.
    pub fn new_m14(config: RunSessionConfig) -> Result<Self, RunSessionError> {
        Self::with_profile(config, RunProfile::M14SilentAct3)
    }
    fn with_profile(config:RunSessionConfig,profile:RunProfile)->Result<Self,RunSessionError>{
        validate_config(profile,&config)?;
        let id=allocate_run_session_id()?;
        Ok(Self{config,profile,id,serial:0,run:None,published:None,actions:vec![],seed:None,ordinal:0,record:None,fault:None,last_step:None,closed:false,at_m12_cutoff:false,at_m13_cutoff:false,at_m14_cutoff:false})
    }
    /// The immutable numeric action/observation contract selected by this session.
    pub fn observation_spec(&self)->Result<crate::RunObservationSpec,RunSessionError>{self.profile.observation_spec(&self.config.capacities)}
    fn usable(&self)->Result<(),RunSessionError>{if self.closed{return Err(RunSessionError::Closed)}if let Some(error)=&self.fault{return Err(error.clone())}Ok(())}
    fn freeze(&mut self,error:RunSessionError,cause:RunSessionError,action:Option<GameAction>,committed:bool,gameplay_recorded:bool)->RunSessionError{
        if self.fault.is_none(){self.fault=Some(error);if let Some(record)=&mut self.record{record.fault=Some(RunRecordedFault{action,committed,gameplay_recorded,error:RunRecordError::from_error(&cause)});}}
        self.fault.clone().unwrap()
    }
    /// Pure reset validation shared by single-session reset and RunBatch's
    /// all-slot preflight. The tuple is effective seed, ordinal, next serial.
    pub(crate) fn preflight_reset(&self,requested:Option<u64>)->Result<(u64,u64,u64),RunSessionError>{
        self.usable()?;
        let (seed,ordinal)=match (requested,self.seed){
            (Some(seed),_)=>(seed,0),
            (None,Some(seed))=>(seed,self.ordinal.checked_add(1).ok_or(RunSessionError::Input("Run episode ordinal exhausted"))?),
            (None,None)=>(0,0),
        };
        let serial=self.serial.checked_add(1).ok_or(RunSessionError::Input("Run decision identity exhausted"))?;
        Ok((seed,ordinal,serial))
    }
    pub fn reset(&mut self,seed:Option<u64>)->Result<RunBoundary,RunSessionError>{
        let (seed,ordinal,serial)=self.preflight_reset(seed)?;
        // Healthy reset intentionally abandons the old episode, never a fault.
        let record=if self.config.record_replay{Some(new_record(self.profile,&self.config,seed,ordinal)?)}else{None};
        self.record=record;self.seed=Some(seed);self.ordinal=ordinal;self.last_step=None;self.published=None;self.actions.clear();
        let run=match create_run(self.profile,&self.config,seed,ordinal){Ok(run)=>run,Err(error)=>{self.run=None;return Err(self.freeze(error.clone(),error,None,false,false))}};
        self.run=Some(run);self.serial=serial;
        let (boundary,actions,at_m12_cutoff,at_m13_cutoff,at_m14_cutoff)=match publish(self.profile,self.run.as_ref().unwrap(),&self.config,RunDecisionToken([self.id,serial])){
            Ok(result)=>result,Err(error)=>return Err(self.freeze(error.clone(),error,None,false,false)),
        };
        let returned=match boundary.owned(){Ok(b)=>b,Err(error)=>return Err(self.freeze(error.clone(),error,None,false,false))};
        self.published=Some(boundary);self.actions=actions;self.at_m12_cutoff=at_m12_cutoff;self.at_m13_cutoff=at_m13_cutoff;self.at_m14_cutoff=at_m14_cutoff;Ok(returned)
    }
    pub fn decision(&self)->Result<RunBoundary,RunSessionError>{self.usable()?;self.published.as_ref().ok_or(RunSessionError::NotReset)?.owned()}
    fn step_preflight(&self,token:RunDecisionToken,index:RunActionIndex)->Result<(usize,u64),RunSessionError>{
        self.usable()?;let published=self.published.as_ref().ok_or(RunSessionError::NotReset)?;
        if token!=published.token{return Err(RunSessionError::StaleToken)}
        if self.at_m12_cutoff{return Err(RunSessionError::UnsupportedBoundary("M12 stops at Act2Ready"))}
        if self.at_m13_cutoff{return Err(RunSessionError::UnsupportedBoundary("M13 stops at Act3Ready"))}
        if self.at_m14_cutoff{return Err(RunSessionError::UnsupportedBoundary("M14 stops before the Act4 door"))}
        if published.terminal.is_some(){return Err(RunSessionError::Terminal)}
        let index=usize::try_from(index.0).ok().filter(|i|*i<self.actions.len()).ok_or(RunSessionError::Input("action index is not legal"))?;
        let serial=self.serial.checked_add(1).ok_or(RunSessionError::Input("Run decision identity exhausted"))?;
        Ok((index,serial))
    }
    /// Purely validate a proposed action without changing the owner or record.
    pub fn preflight_step(&self,token:RunDecisionToken,index:RunActionIndex)->Result<(),RunSessionError>{
        self.step_preflight(token,index).map(|_|())
    }
    pub fn step(&mut self,token:RunDecisionToken,index:RunActionIndex)->Result<RunStepResult,RunSessionError>{
        self.step_with_status(token,index).map_err(|failure|failure.error)
    }
    pub(crate) fn step_with_status(&mut self,token:RunDecisionToken,index:RunActionIndex)->Result<RunStepResult,RunSessionStepFailure>{
        let (index,serial)=self.step_preflight(token,index).map_err(|error|RunSessionStepFailure{error,committed:false})?;
        let action=self.actions[index].clone();
        if let Some(record)=&mut self.record{if record.steps.try_reserve(1).is_err(){
            let error=RunSessionError::DeliveryFault("Run recorder allocation failed");
            let error=self.freeze(error.clone(),error,Some(action),false,false);
            return Err(RunSessionStepFailure{error,committed:false});
        }}
        let transition=match self.run.as_mut().unwrap().step(&action){
            Ok(t)=>t,
            Err(error)=>{
                let committed=matches!(error,RunError::Faulted(_));let error=RunSessionError::Core(error);
                let error=self.freeze(error.clone(),error,Some(action),committed,false);
                return Err(RunSessionStepFailure{error,committed});
            },
        };
        self.serial=serial;let step=RunRecordedStep{action:action.clone(),reward:transition.reward,terminal:transition.terminal};self.last_step=Some(step.clone());
        if let Some(record)=&mut self.record{record.steps.push(step);record.terminal=transition.terminal;}
        let (boundary,actions,at_m12_cutoff,at_m13_cutoff,at_m14_cutoff)=match publish(self.profile,self.run.as_ref().unwrap(),&self.config,RunDecisionToken([self.id,serial])){
            Ok(result)=>result,
            Err(error)=>{
                let returned=if matches!(error,RunSessionError::UnsupportedBoundary(_)){error.clone()}else{RunSessionError::DeliveryFault("post-step Run encoding failed")};
                let error=self.freeze(returned,error,Some(action),true,true);
                return Err(RunSessionStepFailure{error,committed:true});
            },
        };
        let returned=match boundary.owned(){
            Ok(b)=>b,
            Err(error)=>{let error=self.freeze(error.clone(),error,Some(action),true,true);return Err(RunSessionStepFailure{error,committed:true})},
        };
        self.actions=actions;self.published=Some(boundary);self.at_m12_cutoff=at_m12_cutoff;self.at_m13_cutoff=at_m13_cutoff;self.at_m14_cutoff=at_m14_cutoff;
        Ok(RunStepResult{boundary:returned,reward:transition.reward,terminated:transition.terminal.is_some(),truncated:false})
    }
    pub(crate) fn publication_serial(&self)->u64{self.serial}
    pub(crate) fn retain_batch_error(&mut self,error:RunSessionError,committed:bool)->RunSessionError{
        let action=if committed{self.last_step.as_ref().map(|step|step.action.clone())}else{None};
        self.freeze(error.clone(),error,action,committed,committed)
    }
    pub(crate) fn retain_batch_panic(&mut self,committed:bool)->RunSessionError{
        self.retain_batch_error(
            RunSessionError::DeliveryFault("Run batch session operation panicked"),
            committed,
        )
    }
    pub fn record(&self)->Result<Option<RunEpisodeRecord>,RunSessionError>{
        let Some(record)=&self.record else{return Ok(None)};
        let mut steps=Vec::new();steps.try_reserve_exact(record.steps.len()).map_err(|_|RunSessionError::DeliveryFault("Run record export allocation failed"))?;steps.extend(record.steps.iter().cloned());
        let mut owned=new_record(self.profile,&record.config,record.seed,record.ordinal)?;owned.steps=steps;owned.terminal=record.terminal;owned.fault=record.fault.clone();owned.closed=record.closed;Ok(Some(owned))
    }
    pub fn reproduction(&self)->Result<RunReproduction,RunSessionError>{
        Ok(RunReproduction{config:self.config.clone(),seed:self.seed,ordinal:self.ordinal,latest_decision:self.run.as_ref().map(Run::decision),latest_map:self.run.as_ref().and_then(Run::public_map),last_accepted_step:self.last_step.clone(),fault:self.fault.clone(),record:self.record()?,closed:self.closed})
    }
    pub fn close(&mut self)->Result<(),RunSessionError>{self.closed=true;if let Some(record)=&mut self.record{record.closed=true;}Ok(())}
    /// Transport acknowledgement failure, not a gameplay callback. Keep the
    /// accepted native action; never rerun it because Python packaging failed.
    pub fn fail_delivery(&mut self,reason:&'static str)->Result<(),RunSessionError>{
        if self.run.is_none(){return Err(RunSessionError::NotReset)}
        let action=self.last_step.as_ref().map(|s|s.action.clone());let committed=action.is_some();
        self.freeze(RunSessionError::DeliveryFault(reason),RunSessionError::DeliveryFault(reason),action,committed,committed);Ok(())
    }
}
impl RunBoundary{
    fn owned(&self)->Result<Self,RunSessionError>{Ok(Self{observation:self.observation.owned()?,token:self.token,terminal:self.terminal})}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn run_boundary_distinguishes_no_key_empty_deck_and_pre_door_without_fake_terminal(){
        let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m9_default(),record_replay:false};
        let mut d=create_run(RunProfile::M9IroncladAct3,&config,11,0).unwrap().decision().unwrap();d.candidates.clear();
        for terminal in [RunTerminal::Act3NoKeys,RunTerminal::EmptyDeck,RunTerminal::Death]{d.terminal=Some(terminal);assert_eq!(check_boundary(RunProfile::M9IroncladAct3,&d),Ok(()));}
        d.terminal=None;d.room.as_mut().unwrap().kind=RoomViewKind::Act4Ready;assert_eq!(check_boundary(RunProfile::M9IroncladAct3,&d),Err(RunSessionError::UnsupportedBoundary("M9 stops before the Act4 door")));
        let mut m10=create_run(RunProfile::M10IroncladHeart,&config,11,0).unwrap().decision().unwrap();m10.room.as_mut().unwrap().kind=RoomViewKind::Act4Ready;
        assert_eq!(check_boundary(RunProfile::M10IroncladHeart,&m10),Ok(()),"the legacy M9 cutoff is not applied to the M10 profile");
    }
    #[test]
    fn m14_supplied_act4ready_is_healthy_and_stops_before_the_door(){
        let config=RunSessionConfig{character:Character::Silent,ascension:0,partition:Partition::Test,
            boss_history:RunBossHistory::M14SilentAct3{act1:RunAct1BossTarget::Guardian,
                act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},
            capacities:RunCapacities::m10_default(),record_replay:true};
        let mut session=RunSession::new_m14(config.clone()).unwrap();
        let start=session.reset(Some(11)).unwrap();
        let mut decision=create_run(RunProfile::M14SilentAct3,&config,11,0).unwrap().decision().unwrap();
        decision.act=3;decision.floor=50;decision.candidates.clear();
        let room=decision.room.as_mut().unwrap();room.kind=RoomViewKind::Act4Ready;room.choices.clear();
        assert_eq!(check_boundary(RunProfile::M14SilentAct3,&decision),Ok(()));
        let observation=crate::encode_run_m14(&decision,None,&config.capacities).unwrap();
        session.published=Some(RunBoundary{observation,token:start.token,terminal:None});
        session.actions.clear();session.at_m14_cutoff=true;
        let before_boundary=session.decision().unwrap();
        let before_record=session.record().unwrap();
        let expected=RunSessionError::UnsupportedBoundary("M14 stops before the Act4 door");
        assert_eq!(session.preflight_step(start.token,RunActionIndex(0)),Err(expected.clone()));
        assert_eq!(session.step(start.token,RunActionIndex(0)),Err(expected));
        assert_eq!(session.decision().unwrap(),before_boundary);
        assert_eq!(session.record().unwrap(),before_record);
        assert!(session.fault.is_none());
        let reset=session.reset(None).unwrap();
        assert!(!session.at_m14_cutoff);
        assert!(reset.terminal.is_none());
        assert!(reset.observation.action_mask.iter().any(|value|*value==1));
    }
    #[test]
    fn run_session_counter_exhaustion_is_preflight_not_wrap_or_reset(){
        let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m9_default(),record_replay:true};
        let mut s=RunSession::new(config).unwrap();let b=s.reset(Some(11)).unwrap();s.serial=u64::MAX;let before=s.record().unwrap();
        assert!(matches!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::Input(_))));assert_eq!(s.record().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
        s.serial=1;s.ordinal=u64::MAX;assert!(matches!(s.reset(None),Err(RunSessionError::Input(_))));assert_eq!(s.record().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
    }
}
