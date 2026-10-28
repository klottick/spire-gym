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
pub const RUN_RECORD_SCHEMA_ID_M18:&str="spire-gym-m18-defect-act2-replay-v1";
pub const RUN_RULES_ID_M18:&str="spire-gym-native-m18-rules-v1";
pub const RUN_CONTENT_ID_M18:&str="spire-gym-m18-defect-a0-acts1-2-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M17:&str="spire-gym-m17-defect-act1-replay-v1";
pub const RUN_RULES_ID_M17:&str="spire-gym-native-m17-rules-v1";
pub const RUN_CONTENT_ID_M17:&str="spire-gym-m17-defect-a0-act1-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M15: &str = "spire-gym-m15-silent-heart-replay-v1";
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
pub const RUN_RULES_ID_M15: &str = "spire-gym-native-m15-rules-v1";
pub const RUN_CONTENT_ID_M15: &str = "spire-gym-m15-silent-a0-heart-captured-v1";
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
    M15SilentHeart,
    M17DefectAct1,
    M18DefectAct2,
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
            Self::M15SilentHeart => "M15SilentHeart",
            Self::M17DefectAct1 => "M17DefectAct1",
            Self::M18DefectAct2 => "M18DefectAct2",
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
            Self::M15SilentHeart => RUN_RECORD_SCHEMA_ID_M15,
            Self::M17DefectAct1 => RUN_RECORD_SCHEMA_ID_M17,
            Self::M18DefectAct2 => RUN_RECORD_SCHEMA_ID_M18,
        }
    }
    pub(crate) fn rules_id(self) -> &'static str {
        match self {
            Self::M9IroncladAct3 => RUN_RULES_ID,
            Self::M10IroncladHeart | Self::M10IroncladPolicy => RUN_RULES_ID_M10,
            Self::M12SilentAct1 => RUN_RULES_ID_M12,
            Self::M13SilentAct2 => RUN_RULES_ID_M13,
            Self::M14SilentAct3 => RUN_RULES_ID_M14,
            Self::M15SilentHeart => RUN_RULES_ID_M15,
            Self::M17DefectAct1 => RUN_RULES_ID_M17,
            Self::M18DefectAct2 => RUN_RULES_ID_M18,
        }
    }
    pub(crate) fn content_id(self) -> &'static str {
        match self {
            Self::M9IroncladAct3 => RUN_CONTENT_ID,
            Self::M10IroncladHeart | Self::M10IroncladPolicy => RUN_CONTENT_ID_M10,
            Self::M12SilentAct1 => RUN_CONTENT_ID_M12,
            Self::M13SilentAct2 => RUN_CONTENT_ID_M13,
            Self::M14SilentAct3 => RUN_CONTENT_ID_M14,
            Self::M15SilentHeart => RUN_CONTENT_ID_M15,
            Self::M17DefectAct1 => RUN_CONTENT_ID_M17,
            Self::M18DefectAct2 => RUN_CONTENT_ID_M18,
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
            Self::M15SilentHeart => crate::m15_run_observation_spec(caps),
            Self::M17DefectAct1 => crate::m17_run_observation_spec(caps),
            Self::M18DefectAct2 => crate::m18_run_observation_spec(caps),
        }
    }
    fn validate(self, config: &RunSessionConfig) -> Result<(), RunSessionError> {
        let expected_character = if matches!(self,Self::M17DefectAct1|Self::M18DefectAct2) { Character::Defect } else if matches!(self, Self::M12SilentAct1 | Self::M13SilentAct2 | Self::M14SilentAct3 | Self::M15SilentHeart) {
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
                Self::M15SilentHeart => "M15 requires Silent A0",
            Self::M17DefectAct1 => "M17 requires Defect A0",
            Self::M18DefectAct2 => "M18 requires Defect A0",
            }));
        }
        let history_matches = match self {
            Self::M17DefectAct1 => matches!(config.boss_history,RunBossHistory::M17DefectAct1{..}),
            Self::M18DefectAct2 => matches!(config.boss_history,RunBossHistory::M18DefectAct2{..}),
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
            Self::M15SilentHeart => matches!(
                config.boss_history,
                RunBossHistory::M15SilentHeart { .. }
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
            Self::M15SilentHeart => crate::run_schema_m15::validate(&config.capacities),
            Self::M17DefectAct1 => crate::run_schema_m17::validate(&config.capacities),
            Self::M18DefectAct2 => crate::run_schema_m18::validate(&config.capacities),
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
            (RUN_RECORD_SCHEMA_ID_M15, "M15SilentHeart") => Ok(Self::M15SilentHeart),
            (RUN_RECORD_SCHEMA_ID_M17, "M17DefectAct1") => Ok(Self::M17DefectAct1),
            (RUN_RECORD_SCHEMA_ID_M18, "M18DefectAct2") => Ok(Self::M18DefectAct2),
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
    M17DefectAct1 { act1: RunAct1BossTarget },
    M18DefectAct2 { act1: RunAct1BossTarget, act2: RunAct2BossTarget },
    M13SilentAct2 { act1: RunAct1BossTarget, act2: RunAct2BossTarget },
    M14SilentAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
    M15SilentHeart { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
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
        RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget::SlimeBoss } => vec!["Guardian", "Hexaghost"],
        RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget::Hexaghost } => vec!["Guardian", "SlimeBoss"],
        RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget::Guardian } => vec!["Hexaghost", "SlimeBoss"],
        RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M15SilentHeart { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M15SilentHeart { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget::SlimeBoss, .. } |
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::SlimeBoss, .. } => {
            vec!["Guardian", "Hexaghost"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::SlimeBoss, .. } => {
            vec!["Guardian", "Hexaghost"]
        }
        RunBossHistory::M15SilentHeart { act1: RunAct1BossTarget::SlimeBoss, .. } => {
            vec!["Guardian", "Hexaghost"]
        }
        RunBossHistory::Act1GuardianUnseen => vec!["Hexaghost", "SlimeBoss"],
        RunBossHistory::Act1HexaghostUnseen => vec!["Guardian", "SlimeBoss"],
        RunBossHistory::Act1SlimeBossUnseen => vec!["Guardian", "Hexaghost"],
        _ => vec!["Hexaghost", "SlimeBoss"],
    };

    if matches!(profile, RunProfile::M13SilentAct2 | RunProfile::M14SilentAct3 | RunProfile::M15SilentHeart | RunProfile::M18DefectAct2) {
        creation.seen_bosses[1] = match config.boss_history {
            RunBossHistory::M18DefectAct2 { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M15SilentHeart { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M18DefectAct2 { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M15SilentHeart { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M18DefectAct2 { act2: RunAct2BossTarget::Collector, .. } |
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::Collector, .. } => {
                vec!["Champ", "Automaton"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::Collector, .. } => {
                vec!["Champ", "Automaton"]
            }
            RunBossHistory::M15SilentHeart { act2: RunAct2BossTarget::Collector, .. } => {
                vec!["Champ", "Automaton"]
            }
            _ => unreachable!("M13 validation requires a composite boss history"),
        };
        // M13 deliberately keeps CreationProfile::experienced() Act3 history.
    }
    if matches!(profile, RunProfile::M14SilentAct3 | RunProfile::M15SilentHeart) {
        creation.seen_bosses[2] = match config.boss_history {
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::AwakenedOne, .. } => {
                vec!["TimeEater", "DonuAndDeca"]
            }
            RunBossHistory::M15SilentHeart { act3: RunAct3BossTarget::AwakenedOne, .. } => {
                vec!["TimeEater", "DonuAndDeca"]
            }
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::TimeEater, .. } => {
                vec!["AwakenedOne", "DonuAndDeca"]
            }
            RunBossHistory::M15SilentHeart { act3: RunAct3BossTarget::TimeEater, .. } => {
                vec!["AwakenedOne", "DonuAndDeca"]
            }
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::DonuAndDeca, .. } => {
                vec!["AwakenedOne", "TimeEater"]
            }
            RunBossHistory::M15SilentHeart { act3: RunAct3BossTarget::DonuAndDeca, .. } => {
                vec!["AwakenedOne", "TimeEater"]
            }
            _ => unreachable!("M14 validation requires a triple boss history"),
        };
    } else if profile != RunProfile::M12SilentAct1 && profile != RunProfile::M13SilentAct2 && profile != RunProfile::M17DefectAct1 && profile != RunProfile::M18DefectAct2 {
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
        RunProfile::M15SilentHeart => RoomScenario::M15SilentHeart,
        RunProfile::M17DefectAct1 => RoomScenario::M17DefectAct1,
        RunProfile::M18DefectAct2 => RoomScenario::M18DefectAct2,
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
    if matches!(profile, RunProfile::M12SilentAct1 | RunProfile::M17DefectAct1)
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M12 Act2Ready must be nonterminal with no actions",
        ));
    }
    if matches!(profile,RunProfile::M13SilentAct2|RunProfile::M18DefectAct2)
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
        RunProfile::M15SilentHeart => {
            crate::encode_run_m15(&decision, map.as_ref(), &config.capacities)?
        }
        RunProfile::M17DefectAct1 => crate::encode_run_m17(&decision, map.as_ref(), &config.capacities)?,
        RunProfile::M18DefectAct2 => crate::encode_run_m18(&decision, map.as_ref(), &config.capacities)?,
    };
    let at_m12_cutoff = matches!(profile,RunProfile::M12SilentAct1|RunProfile::M17DefectAct1)
        && decision.terminal.is_none()
        && decision.candidates.is_empty()
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready);
    let at_m13_cutoff = matches!(profile,RunProfile::M13SilentAct2|RunProfile::M18DefectAct2)
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
            "rng_id":spire_core::combat::rng::RNG_CONTRACT_ID,"combat_view_id":if matches!(self.profile,RunProfile::M12SilentAct1|RunProfile::M13SilentAct2|RunProfile::M14SilentAct3|RunProfile::M15SilentHeart|RunProfile::M17DefectAct1|RunProfile::M18DefectAct2){serde_json::Value::Null}else{serde_json::json!(crate::COMBAT_POLICY_VIEW_ID)},
            "mode":"normal_run","partition":self.config.partition.as_str(),"autoreset":"disabled"});
        if self.profile==RunProfile::M18DefectAct2{
            descriptor["finite_content"]=serde_json::json!({
                "permanent_cards":M18_DEFECT_ACT2_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "generated_and_status_cards":["dazed","burn","slimed","wound"],
                "relics":M18_DEFECT_ACT2_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "potions":M18_DEFECT_ACT2_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_enemies":crate::run_schema_m13::M13_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_encounters":crate::run_schema_m13::M13_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons":crate::run_schema_m13::M13_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
            });
            descriptor["cutoff"]=serde_json::json!("Act3Ready: nonterminal, zero actions, no completion reward");
        }
        if self.profile==RunProfile::M17DefectAct1{
            descriptor["finite_content"]=serde_json::json!({
                "permanent_cards":M17_DEFECT_ACT1_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "generated_and_status_cards":["dazed","burn","slimed"],
                "relics":M17_DEFECT_ACT1_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "potions":M17_DEFECT_ACT1_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_enemies":crate::run_schema_m12::M12_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_encounters":crate::run_schema_m12::M12_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons":crate::run_schema_m12::M12_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
            });
            descriptor["cutoff"]=serde_json::json!("Act2Ready: nonterminal, zero actions, no completion reward");
        }
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
        if self.profile == RunProfile::M15SilentHeart {
            // Reuse the versioned schema's full entitlements and encounter domains,
            // including its generated/colorless and Act4 vocabulary.
            let observation_descriptor=spec.descriptor();
            let semantics=&observation_descriptor["semantics"];
            descriptor["finite_content"]=semantics["entitlements"].clone();
            descriptor["finite_content"]["playable_enemies"]=semantics["enemy_species"]["cumulative_through_act4"].clone();
            descriptor["finite_content"]["playable_encounters"]=semantics["encounters"]["playable"].clone();
            descriptor["finite_content"]["act4_encounters"]=semantics["encounters"]["act4"].clone();
            descriptor["finite_content"]["map_only_boss_icons"]=semantics["encounters"]["map_only_boss_icons"].clone();
            descriptor["ending"] = serde_json::json!("HeartVictory after the actual Act4 Proceed action");
        }
        Ok(descriptor)
    }
    pub fn combat_observation_spec(&self)->Result<crate::RunObservationSpec,RunSessionError>{
        if self.profile==RunProfile::M18DefectAct2{return Err(RunSessionError::UnsupportedBoundary("M18 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M17DefectAct1{return Err(RunSessionError::UnsupportedBoundary("M17 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M12SilentAct1{return Err(RunSessionError::UnsupportedBoundary("M12 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M13SilentAct2{return Err(RunSessionError::UnsupportedBoundary("M13 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M14SilentAct3{return Err(RunSessionError::UnsupportedBoundary("M14 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M15SilentHeart{return Err(RunSessionError::UnsupportedBoundary("M15 does not provide the Ironclad combat policy sidecar"))}
        crate::combat_policy_observation_spec()
    }
    /// Owned public sidecar. Dispatch uses the same token/index as the full Run.
    /// Resources are current; snapshot on the settled-count transition. The
    /// visit-keyed last_exit remains sticky across later noncombat decisions.
    pub fn combat_view(&mut self)->Result<crate::CombatPolicyBoundary,RunSessionError>{
        if self.profile==RunProfile::M18DefectAct2{return Err(RunSessionError::UnsupportedBoundary("M18 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M17DefectAct1{return Err(RunSessionError::UnsupportedBoundary("M17 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M12SilentAct1{return Err(RunSessionError::UnsupportedBoundary("M12 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M13SilentAct2{return Err(RunSessionError::UnsupportedBoundary("M13 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M14SilentAct3{return Err(RunSessionError::UnsupportedBoundary("M14 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M15SilentHeart{return Err(RunSessionError::UnsupportedBoundary("M15 does not provide the Ironclad combat policy sidecar"))}
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
    /// Opt into the finite Defect A0 Acts 1–2 profile on the shared native owner.
    pub fn new_m18(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M18DefectAct2)}
    pub fn new_m17(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M17DefectAct1)}
    pub fn new_m12(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M12SilentAct1)}
    /// Opt into the finite Silent A0 captured Acts 1–2 profile on the shared session owner.
    pub fn new_m13(config: RunSessionConfig) -> Result<Self, RunSessionError> {
        Self::with_profile(config, RunProfile::M13SilentAct2)
    }
    /// Opt into the finite Silent A0 captured Acts 1–3 profile on the shared session owner.
    pub fn new_m14(config: RunSessionConfig) -> Result<Self, RunSessionError> {
        Self::with_profile(config, RunProfile::M14SilentAct3)
    }
    /// Opt into the full finite Silent A0 Run through the Heart on the shared session owner.
    pub fn new_m15(config: RunSessionConfig) -> Result<Self, RunSessionError> {
        Self::with_profile(config, RunProfile::M15SilentHeart)
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
        if self.at_m12_cutoff{return Err(RunSessionError::UnsupportedBoundary(if self.profile==RunProfile::M17DefectAct1{"M17 stops at Act2Ready"}else{"M12 stops at Act2Ready"}))}
        if self.at_m13_cutoff{return Err(RunSessionError::UnsupportedBoundary(if self.profile==RunProfile::M18DefectAct2{"M18 stops at Act3Ready"}else{"M13 stops at Act3Ready"}))}
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
    #[test]
    fn m15_native_profiles_keep_old_bytes_and_boundaries() {
        let ironclad = RunSessionConfig {
            character: Character::Ironclad,
            ascension: 0,
            partition: Partition::Test,
            boss_history: RunBossHistory::AwakenedUnseen,
            capacities: RunCapacities::m10_default(),
            record_replay: true,
        };
        let m9 = RunSession::new(ironclad.clone()).unwrap();
        assert_eq!(m9.environment_spec().unwrap()["profile_id"], "M9IroncladAct3");
        assert_eq!(
            m9.observation_spec().unwrap().fingerprint().iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"
        );
        let m10 = RunSession::new_m10(ironclad).unwrap();
        assert_eq!(m10.environment_spec().unwrap()["profile_id"], "M10IroncladHeart");
        assert_eq!(
            m10.observation_spec().unwrap().fingerprint().iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"
        );
        assert_eq!(m10.environment_spec().unwrap()["combat_view_id"], crate::COMBAT_POLICY_VIEW_ID);

        let silent = |boss_history| RunSessionConfig {
            character: Character::Silent,
            ascension: 0,
            partition: Partition::Test,
            boss_history,
            capacities: RunCapacities::m10_default(),
            record_replay: true,
        };
        let m15 = RunSession::new_m15(silent(RunBossHistory::M15SilentHeart {
            act1: RunAct1BossTarget::Guardian,
            act2: RunAct2BossTarget::Champ,
            act3: RunAct3BossTarget::AwakenedOne,
        })).unwrap();
        let m15_environment = m15.environment_spec().unwrap();
        assert_eq!(m15_environment["profile_id"], "M15SilentHeart");
        assert!(m15_environment["combat_view_id"].is_null());

        let cutoffs = [
            (RunProfile::M12SilentAct1, silent(RunBossHistory::Act1GuardianUnseen), RoomViewKind::Act2Ready, 2, 17, "M12 stops at Act2Ready"),
            (RunProfile::M13SilentAct2, silent(RunBossHistory::M13SilentAct2 {
                act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ,
            }), RoomViewKind::Act3Ready, 3, 34, "M13 stops at Act3Ready"),
            (RunProfile::M14SilentAct3, silent(RunBossHistory::M14SilentAct3 {
                act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ,
                act3: RunAct3BossTarget::AwakenedOne,
            }), RoomViewKind::Act4Ready, 3, 50, "M14 stops before the Act4 door"),
        ];
        for (profile, config, cutoff, act, floor, message) in cutoffs {
            let mut session = match profile {
                RunProfile::M12SilentAct1 => RunSession::new_m12(config.clone()).unwrap(),
                RunProfile::M13SilentAct2 => RunSession::new_m13(config.clone()).unwrap(),
                RunProfile::M14SilentAct3 => RunSession::new_m14(config.clone()).unwrap(),
                _ => unreachable!(),
            };
            let start = session.reset(Some(11)).unwrap();
            let mut decision = create_run(profile, &config, 11, 0).unwrap().decision().unwrap();
            decision.act = act;
            decision.floor = floor;
            decision.terminal = None;
            decision.candidates.clear();
            let room = decision.room.as_mut().unwrap();
            room.kind = cutoff;
            room.choices.clear();
            assert_eq!(check_boundary(profile, &decision), Ok(()));
            let observation = match profile {
                RunProfile::M12SilentAct1 => crate::encode_run_m12(&decision, None, &config.capacities).unwrap(),
                RunProfile::M13SilentAct2 => crate::encode_run_m13(&decision, None, &config.capacities).unwrap(),
                RunProfile::M14SilentAct3 => crate::encode_run_m14(&decision, None, &config.capacities).unwrap(),
                _ => unreachable!(),
            };
            session.published = Some(RunBoundary { observation, token: start.token, terminal: None });
            session.actions.clear();
            match profile {
                RunProfile::M12SilentAct1 => session.at_m12_cutoff = true,
                RunProfile::M13SilentAct2 => session.at_m13_cutoff = true,
                RunProfile::M14SilentAct3 => session.at_m14_cutoff = true,
                _ => unreachable!(),
            }
            let before_boundary = session.decision().unwrap();
            assert!(before_boundary.observation.action_mask.iter().all(|value| *value == 0));
            let before_record = session.record().unwrap();
            let expected = RunSessionError::UnsupportedBoundary(message);
            assert_eq!(session.preflight_step(start.token, RunActionIndex(0)), Err(expected.clone()));
            assert_eq!(session.step(start.token, RunActionIndex(0)), Err(expected));
            assert_eq!(session.decision().unwrap(), before_boundary);
            assert_eq!(session.record().unwrap(), before_record);
            assert!(session.fault.is_none());

            let reset = session.reset(None).unwrap();
            assert_eq!(session.effective_seed(), Some(11));
            assert!(reset.terminal.is_none());
            assert!(!session.at_m12_cutoff && !session.at_m13_cutoff && !session.at_m14_cutoff);
            assert!(reset.observation.action_mask.iter().any(|value| *value != 0));
        }
    }

    #[test]
    fn m15_synthetic_first_fault_custody_keeps_core_error_and_readable_prefix() {
        // Custody-only injection: this is not a natural M15 rule fault or
        // evidence that a real core fault is replayable.
        let config = RunSessionConfig {
            character: Character::Silent,
            ascension: 0,
            partition: Partition::Test,
            boss_history: RunBossHistory::M15SilentHeart {
                act1: RunAct1BossTarget::Guardian,
                act2: RunAct2BossTarget::Champ,
                act3: RunAct3BossTarget::AwakenedOne,
            },
            capacities: RunCapacities::m10_default(),
            record_replay: true,
        };
        let mut session = RunSession::new_m15(config.clone()).unwrap();
        let start = session.reset(Some(7)).unwrap();
        let accepted = session.step(start.token, RunActionIndex(0)).unwrap();
        let before = session.reproduction().unwrap();
        let accepted_action = before.last_accepted_step.as_ref().unwrap().action.clone();
        let fault_action = session.actions[0].clone();
        let synthetic = RunSessionError::Core(RunError::Faulted(RunFault {
            kind: RunFaultKind::Invariant("synthetic M15 fault custody probe"),
            checkpoint: "m15.synthetic.custody",
            committed_events: vec![],
        }));
        assert_eq!(
            session.freeze(synthetic.clone(), synthetic.clone(), Some(fault_action.clone()), true, false),
            synthetic
        );
        let capacity = RunSessionError::Capacity { group: "map_nodes", required: 2, capacity: 1 };
        assert_eq!(
            session.freeze(capacity.clone(), capacity.clone(), Some(fault_action.clone()), true, true),
            synthetic
        );
        let delivery = RunSessionError::DeliveryFault("synthetic later delivery fault");
        assert_eq!(
            session.freeze(delivery.clone(), delivery, Some(fault_action.clone()), true, true),
            synthetic
        );

        let after = session.reproduction().unwrap();
        assert_eq!(after.fault, Some(synthetic.clone()));
        assert_eq!(after.latest_decision, before.latest_decision);
        assert_eq!(after.latest_map, before.latest_map);
        assert_eq!(after.last_accepted_step.as_ref().map(|step| &step.action), Some(&accepted_action));
        assert_eq!(after.record.as_ref().unwrap().steps.len(), 1);
        let retained = after.record.as_ref().unwrap().fault.as_ref().unwrap();
        assert_eq!(retained.action.as_ref(), Some(&fault_action));
        assert!(retained.committed && !retained.gameplay_recorded);
        assert_eq!(retained.error.kind, RunRecordErrorKind::Core);

        let wire = after.record.unwrap().to_json().unwrap();
        let restored = crate::RunEpisodeRecord::from_json(&wire).unwrap();
        assert_eq!(
            crate::replay_run(&restored, config.capacities.clone()).unwrap_err(),
            RunSessionError::IncompatibleReplay("recorded core fault no longer occurs")
        );
        assert_eq!(
            session.step(accepted.boundary.token, RunActionIndex(0)),
            Err(synthetic.clone())
        );

        let mut no_record_config = config;
        no_record_config.record_replay = false;
        let mut no_record = RunSession::new_m15(no_record_config).unwrap();
        let boundary = no_record.reset(Some(7)).unwrap();
        no_record.step(boundary.token, RunActionIndex(0)).unwrap();
        let before_no_record = no_record.reproduction().unwrap();
        let fault_action = no_record.actions[0].clone();
        let no_record_core = RunSessionError::Core(RunError::Faulted(RunFault {
            kind: RunFaultKind::Invariant("synthetic M15 no-record custody probe"),
            checkpoint: "m15.synthetic.no-record",
            committed_events: vec![],
        }));
        no_record.freeze(no_record_core.clone(), no_record_core.clone(), Some(fault_action.clone()), true, false);
        let later = RunSessionError::DeliveryFault("synthetic later delivery fault");
        assert_eq!(
            no_record.freeze(later.clone(), later, Some(fault_action), true, true),
            no_record_core
        );
        let after_no_record = no_record.reproduction().unwrap();
        assert_eq!(after_no_record.record, None);
        assert_eq!(after_no_record.fault, Some(no_record_core));
        assert_eq!(after_no_record.latest_decision, before_no_record.latest_decision);
        assert_eq!(after_no_record.latest_map, before_no_record.latest_map);
        assert_eq!(after_no_record.last_accepted_step, before_no_record.last_accepted_step);
    }

}

#[cfg(test)]
mod m15_attempt_tests {
    use super::*;
    use crate::{RunBatch, RunBatchError, RunRecordError, RunSessionConfig};
    use serde_json::{json, Value};
    use spire_core::{
        combat::rng::Partition,
        content::Character,
        run::{GameAction, PublicDecision, Run, RunTerminal},
    };
    use std::fs;

    const RECEIPT_SCHEMA: &str = "m15-silent-heart-caller-attempt-v1";
    const CALLER_REVISION: &str = "m15-key-aware-public-caller-v1";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct FrozenCase {
        seed: u64,
        act1: &'static str,
        act2: &'static str,
        act3: &'static str,
    }

    // The six seed/triple pairs from the caller's frozen fixture list.
    // This consumer never creates a caller or invokes its policy.
    const FROZEN_CASES: [FrozenCase; 6] = [
        FrozenCase { seed: 7, act1: "Guardian", act2: "Champ", act3: "AwakenedOne" },
        FrozenCase { seed: 7, act1: "Hexaghost", act2: "BronzeAutomaton", act3: "TimeEater" },
        FrozenCase { seed: 7, act1: "SlimeBoss", act2: "Collector", act3: "DonuAndDeca" },
        FrozenCase { seed: 11, act1: "Guardian", act2: "BronzeAutomaton", act3: "DonuAndDeca" },
        FrozenCase { seed: 11, act1: "Hexaghost", act2: "Collector", act3: "AwakenedOne" },
        FrozenCase { seed: 11, act1: "SlimeBoss", act2: "Champ", act3: "TimeEater" },
    ];

    struct ParsedAttempt {
        value: Value,
        config: RunSessionConfig,
        seed: u64,
        action_budget: usize,
        candidate_indices: Vec<usize>,
        record: crate::RunEpisodeRecord,
        failed_action: Option<GameAction>,
        outcome: Option<String>,
        error: Option<String>,
    }

    fn problem(message: impl Into<String>) -> String {
        message.into()
    }

    fn exact_fields(value: &Value, path: &str, expected: &[&str]) -> Result<(), String> {
        let object = value.as_object().ok_or_else(|| problem(format!("{path} must be an object")))?;
        if object.len() != expected.len() || expected.iter().any(|field| !object.contains_key(*field)) {
            return Err(problem(format!("{path} has missing or unknown fields")));
        }
        Ok(())
    }

    fn string<'a>(value: &'a Value, path: &str) -> Result<&'a str, String> {
        value.as_str().ok_or_else(|| problem(format!("{path} must be a string")))
    }

    fn number(value: &Value, path: &str) -> Result<u64, String> {
        value.as_u64().ok_or_else(|| problem(format!("{path} must be an unsigned integer")))
    }

    fn optional_string(value: &Value, path: &str) -> Result<Option<String>, String> {
        if value.is_null() { Ok(None) } else { Ok(Some(string(value, path)?.to_owned())) }
    }

    fn parse_act1(name: &str) -> Result<RunAct1BossTarget, String> {
        match name {
            "Guardian" => Ok(RunAct1BossTarget::Guardian),
            "Hexaghost" => Ok(RunAct1BossTarget::Hexaghost),
            "SlimeBoss" => Ok(RunAct1BossTarget::SlimeBoss),
            _ => Err(problem(format!("unknown Act1 history target {name:?}"))),
        }
    }

    fn parse_act2(name: &str) -> Result<RunAct2BossTarget, String> {
        match name {
            "Champ" => Ok(RunAct2BossTarget::Champ),
            "BronzeAutomaton" => Ok(RunAct2BossTarget::BronzeAutomaton),
            "Collector" => Ok(RunAct2BossTarget::Collector),
            _ => Err(problem(format!("unknown Act2 history target {name:?}"))),
        }
    }

    fn parse_act3(name: &str) -> Result<RunAct3BossTarget, String> {
        match name {
            "AwakenedOne" => Ok(RunAct3BossTarget::AwakenedOne),
            "TimeEater" => Ok(RunAct3BossTarget::TimeEater),
            "DonuAndDeca" => Ok(RunAct3BossTarget::DonuAndDeca),
            _ => Err(problem(format!("unknown Act3 history target {name:?}"))),
        }
    }

    fn case_from_value(value: &Value) -> Result<FrozenCase, String> {
        let scenario = value.get("scenario").ok_or_else(|| problem("missing scenario"))?;
        exact_fields(scenario, "scenario", &["act1_boss", "act2_boss", "act3_boss"])?;
        let seed = number(value.get("seed").ok_or_else(|| problem("missing seed"))?, "seed")?;
        let act1 = string(&scenario["act1_boss"], "scenario.act1_boss")?;
        let act2 = string(&scenario["act2_boss"], "scenario.act2_boss")?;
        let act3 = string(&scenario["act3_boss"], "scenario.act3_boss")?;
        FROZEN_CASES.iter().copied()
            .find(|case| case.seed == seed && case.act1 == act1 && case.act2 == act2 && case.act3 == act3)
            .ok_or_else(|| problem(format!("unknown frozen M15 seed/triple: seed={seed}, {act1}/{act2}/{act3}")))
    }

    fn config_for(case: FrozenCase) -> Result<RunSessionConfig, String> {
        Ok(RunSessionConfig {
            character: Character::Silent,
            ascension: 0,
            partition: Partition::Test,
            boss_history: RunBossHistory::M15SilentHeart {
                act1: parse_act1(case.act1)?,
                act2: parse_act2(case.act2)?,
                act3: parse_act3(case.act3)?,
            },
            capacities: RunCapacities::m10_default(),
            record_replay: true,
        })
    }

    fn template_value(config: &RunSessionConfig, seed: u64) -> Result<Value, String> {
        let mut session = RunSession::new_m15(config.clone())
            .map_err(|error| problem(format!("create M15 replay template: {error:?}")))?;
        session.reset(Some(seed))
            .map_err(|error| problem(format!("reset M15 replay template: {error:?}")))?;
        let record = session.record()
            .map_err(|error| problem(format!("read M15 replay template: {error:?}")))?
            .ok_or_else(|| problem("M15 replay template unexpectedly disabled recording"))?;
        let bytes = record.to_json()
            .map_err(|error| problem(format!("encode M15 replay template: {error:?}")))?;
        crate::run_replay::parse_unique_json(&bytes)
            .map_err(|error| problem(format!("parse M15 replay template: {error:?}")))
    }

    fn record_from_value(value: &Value) -> Result<crate::RunEpisodeRecord, String> {
        let bytes = serde_json::to_vec(value)
            .map_err(|error| problem(format!("encode portable record: {error}")))?;
        crate::RunEpisodeRecord::from_json(&bytes)
            .map_err(|error| problem(format!("decode portable record: {error:?}")))
    }

    /// The existing versioned Run parser is the only GameAction parser used.
    /// A one-step dummy record exists only to decode a failed_action value.
    fn decode_failed_action(
        config: &RunSessionConfig,
        seed: u64,
        action_value: &Value,
    ) -> Result<GameAction, String> {
        let mut template = template_value(config, seed)?;
        template["steps"] = json!([{ "action": action_value, "reward": 0, "terminal": null }]);
        let record = record_from_value(&template)?;
        record.steps.first().map(|step| step.action.clone())
            .ok_or_else(|| problem("dummy action record was not decoded"))
    }

    fn validate_terminal(value: &Value, path: &str) -> Result<(), String> {
        if value.is_null() || matches!(value.as_str(), Some("death" | "empty_deck" | "act3_no_keys" | "heart_victory")) {
            Ok(())
        } else {
            Err(problem(format!("{path} has an unknown terminal")))
        }
    }

    fn parse_receipt(value: Value, expected: Option<FrozenCase>) -> Result<ParsedAttempt, String> {
        exact_fields(&value, "receipt", &[
            "schema_id", "seed", "scenario", "caller_revision", "action_budget", "outcome", "error",
            "accepted_action_count", "actions", "steps", "failed_action", "endpoint",
        ])?;
        if string(&value["schema_id"], "schema_id")? != RECEIPT_SCHEMA {
            return Err(problem("unsupported M15 caller receipt schema"));
        }
        if string(&value["caller_revision"], "caller_revision")? != CALLER_REVISION {
            return Err(problem("unexpected M15 caller revision"));
        }
        let case = case_from_value(&value)?;
        if !FROZEN_CASES.contains(&case) {
            return Err(problem(format!("receipt is outside the six frozen seed/triple cases: {case:?}")));
        }
        if expected.is_some_and(|expected| expected != case) {
            return Err(problem(format!("receipt does not match frozen case {expected:?}")));
        }
        let action_budget = usize::try_from(number(&value["action_budget"], "action_budget")?)
            .map_err(|_| problem("action_budget exceeds usize"))?;
        if expected.is_some() && action_budget != 8192 {
            return Err(problem("frozen source attempt must preserve the caller's 8192-action budget"));
        }
        let accepted_count = usize::try_from(number(&value["accepted_action_count"], "accepted_action_count")?)
            .map_err(|_| problem("accepted_action_count exceeds usize"))?;
        let actions = value["actions"].as_array().ok_or_else(|| problem("actions must be an array"))?;
        let steps = value["steps"].as_array().ok_or_else(|| problem("steps must be an array"))?;
        if accepted_count != actions.len() || accepted_count != steps.len() || accepted_count > action_budget {
            return Err(problem("accepted action count, actions, steps, and budget disagree"));
        }

        let outcome = optional_string(&value["outcome"], "outcome")?;
        if outcome.as_deref().is_some_and(|name| !matches!(name,
            "HeartVictory" | "Death" | "BudgetExhausted" | "Act3NoKeys" | "EmptyDeck")) {
            return Err(problem("unknown M15 caller outcome"));
        }
        let error = optional_string(&value["error"], "error")?;
        if outcome.is_some() == error.is_some() {
            return Err(problem("receipt must contain exactly one of outcome or operational error"));
        }
        let failed_value = if value["failed_action"].is_null() { None } else { Some(&value["failed_action"]) };
        if failed_value.is_some() && error.is_none() {
            return Err(problem("failed_action requires an operational caller error"));
        }

        let mut candidate_indices = Vec::with_capacity(steps.len());
        for (ordinal, step) in steps.iter().enumerate() {
            exact_fields(step, &format!("steps[{ordinal}]"), &[
                "index", "candidate_index", "pre", "action", "transition", "post",
            ])?;
            if number(&step["index"], "step.index")? != ordinal as u64 {
                return Err(problem(format!("steps[{ordinal}].index is not its accepted-action ordinal")));
            }
            let candidate_index = usize::try_from(number(&step["candidate_index"], "step.candidate_index")?)
                .map_err(|_| problem(format!("steps[{ordinal}].candidate_index exceeds usize")))?;
            candidate_indices.push(candidate_index);
            if actions[ordinal] != step["action"] {
                return Err(problem(format!("actions[{ordinal}] differs from steps[{ordinal}].action")));
            }

            exact_fields(&step["pre"], &format!("steps[{ordinal}].pre"), &["decision_debug", "map_debug"])?;
            string(&step["pre"]["decision_debug"], "pre.decision_debug")?;
            optional_string(&step["pre"]["map_debug"], "pre.map_debug")?;

            exact_fields(&step["transition"], &format!("steps[{ordinal}].transition"), &[
                "reward", "terminal", "committed_events", "committed_events_debug",
            ])?;
            if !step["transition"]["reward"].is_i64() {
                return Err(problem(format!("steps[{ordinal}].transition.reward must be signed integer")));
            }
            validate_terminal(&step["transition"]["terminal"], "transition.terminal")?;
            let events = step["transition"]["committed_events"].as_array()
                .ok_or_else(|| problem(format!("steps[{ordinal}].transition.committed_events must be array")))?;
            let event_debug = step["transition"]["committed_events_debug"].as_array()
                .ok_or_else(|| problem(format!("steps[{ordinal}].transition.committed_events_debug must be array")))?;
            if !event_debug.iter().all(Value::is_string) || events.len() != event_debug.len() {
                return Err(problem(format!("steps[{ordinal}] committed event fields disagree")));
            }

            exact_fields(&step["post"], &format!("steps[{ordinal}].post"), &["decision", "map_debug"])?;
            exact_fields(&step["post"]["decision"], &format!("steps[{ordinal}].post.decision"), &["debug", "error"])?;
            let post_debug = optional_string(&step["post"]["decision"]["debug"], "post.decision.debug")?;
            let post_error = optional_string(&step["post"]["decision"]["error"], "post.decision.error")?;
            if post_debug.is_some() == post_error.is_some() {
                return Err(problem(format!("steps[{ordinal}] post-decision must contain debug or error")));
            }
            optional_string(&step["post"]["map_debug"], "post.map_debug")?;
        }

        exact_fields(&value["endpoint"], "endpoint", &[
            "available", "decision_error", "act", "floor", "room", "terminal", "decision_debug", "map_debug",
        ])?;
        let available = value["endpoint"]["available"].as_bool()
            .ok_or_else(|| problem("endpoint.available must be boolean"))?;
        let decision_error = optional_string(&value["endpoint"]["decision_error"], "endpoint.decision_error")?;
        let decision_debug = optional_string(&value["endpoint"]["decision_debug"], "endpoint.decision_debug")?;
        if available != decision_error.is_none() {
            return Err(problem("endpoint.available and decision_error disagree"));
        }
        if decision_debug.is_none() && (value["endpoint"]["act"].is_number() || value["endpoint"]["floor"].is_number()) {
            return Err(problem("endpoint numeric summary has no decision snapshot"));
        }
        optional_string(&value["endpoint"]["room"], "endpoint.room")?;
        optional_string(&value["endpoint"]["map_debug"], "endpoint.map_debug")?;
        validate_terminal(&value["endpoint"]["terminal"], "endpoint.terminal")?;

        if outcome.as_deref() == Some("BudgetExhausted") && accepted_count != action_budget {
            return Err(problem("BudgetExhausted receipt did not consume its action budget"));
        }
        if outcome.as_deref().is_some_and(|name| name != "BudgetExhausted") && !available {
            return Err(problem("terminal caller outcome has no readable live endpoint"));
        }

        let config = config_for(case)?;
        let mut record_value = template_value(&config, case.seed)?;
        record_value["steps"] = Value::Array(steps.iter().map(|step| json!({
            "action": step["action"].clone(),
            "reward": step["transition"]["reward"].clone(),
            "terminal": step["transition"]["terminal"].clone(),
        })).collect());
        record_value["terminal"] = steps.last()
            .map(|step| step["transition"]["terminal"].clone()).unwrap_or(Value::Null);
        let record = record_from_value(&record_value)?;
        let failed_action = failed_value.map(|value| decode_failed_action(&config, case.seed, value)).transpose()?;

        Ok(ParsedAttempt {
            value, config, seed: case.seed, action_budget, candidate_indices, record, failed_action, outcome, error,
        })
    }

    fn terminal_value(terminal: Option<RunTerminal>) -> Value {
        match terminal {
            None => Value::Null,
            Some(RunTerminal::Death) => json!("death"),
            Some(RunTerminal::EmptyDeck) => json!("empty_deck"),
            Some(RunTerminal::Act3NoKeys) => json!("act3_no_keys"),
            Some(RunTerminal::HeartVictory) => json!("heart_victory"),
        }
    }

    fn decision_debug(decision: &PublicDecision) -> String {
        format!("{decision:#?}")
    }

    fn map_debug(map: Option<&spire_core::run::map::PublicMap>) -> Option<String> {
        map.map(|map| format!("{map:#?}"))
    }

    fn compare_observation(
        context: &str,
        observation: &crate::RunObservation,
        decision: &PublicDecision,
        map: Option<&spire_core::run::map::PublicMap>,
        capacities: &RunCapacities,
    ) -> Result<(), String> {
        let encoded = crate::encode_run_m15(decision, map, capacities)
            .map_err(|error| problem(format!("{context}: M15 encoder failed: {error:?}")))?;
        if observation != &encoded {
            return Err(problem(format!("{context}: adapter observation differs from direct M15 encoding")));
        }
        Ok(())
    }

    fn expected_fault_record(
        prefix: &crate::RunEpisodeRecord,
        action_value: &Value,
        error: &RunSessionError,
        committed: bool,
        gameplay_recorded: bool,
    ) -> Result<crate::RunEpisodeRecord, String> {
        let bytes = prefix.to_json()
            .map_err(|error| problem(format!("encode prefix before failed action: {error:?}")))?;
        let mut wire = crate::run_replay::parse_unique_json(&bytes)
            .map_err(|error| problem(format!("parse prefix before failed action: {error:?}")))?;
        wire["fault"] = json!({
            "action": action_value,
            "committed": committed,
            "gameplay_recorded": gameplay_recorded,
            "error": RunRecordError::from_error(error),
        });
        record_from_value(&wire)
    }

    fn compare_report(
        record: &crate::RunEpisodeRecord,
        report: &crate::RunReplayReport,
        last_readable: &PublicDecision,
        core: &Run,
    ) -> Result<(), String> {
        if report.accepted_count != record.steps.len() {
            return Err(problem("portable replay accepted count differs from caller prefix"));
        }
        if report.rewards != record.steps.iter().map(|step|step.reward).collect::<Vec<_>>()
            || report.terminals != record.steps.iter().map(|step|step.terminal).collect::<Vec<_>>() {
            return Err(problem("portable replay reward or terminal vector differs from caller prefix"));
        }
        if &report.final_decision != last_readable {
            return Err(problem("portable replay final decision differs from last readable core decision"));
        }
        if report.final_map != core.public_map() {
            return Err(problem("portable replay map differs from direct core public map"));
        }
        if report.reported_fault != record.fault {
            return Err(problem("portable replay reported fault differs from retained native record"));
        }
        if report.final_decision_error != core.decision().err() {
            return Err(problem("portable replay final decision error differs from direct core"));
        }
        Ok(())
    }

    fn compare_outcome(
        attempt: &ParsedAttempt,
        live: &Result<PublicDecision, spire_core::run::RunError>,
    ) -> Result<(), String> {
        if let Some(error) = &attempt.error {
            if attempt.outcome.is_some() {
                return Err(problem(format!("operational caller error {error:?} was converted into an outcome")));
            }
            if attempt.failed_action.is_none() {
                live.as_ref()
                    .map_err(|core_error| problem(format!("caller error without failed_action retained a core fault: {core_error:?}")))?;
            }
            return Ok(());
        }
        let outcome = attempt.outcome.as_deref().ok_or_else(|| problem("successful receipt has no outcome"))?;
        let decision = live.as_ref().map_err(|error| problem(format!("outcome has no live decision: {error:?}")))?;
        let terminal = terminal_value(decision.terminal);
        match outcome {
            "HeartVictory" if terminal == json!("heart_victory") => Ok(()),
            "Death" if terminal == json!("death") => Ok(()),
            "Act3NoKeys" if terminal == json!("act3_no_keys") => Ok(()),
            "EmptyDeck" if terminal == json!("empty_deck") => Ok(()),
            "BudgetExhausted" if attempt.record.steps.len() == attempt.action_budget && terminal.is_null() => Ok(()),
            _ => Err(problem(format!("caller outcome {outcome} disagrees with terminal or action budget"))),
        }
    }

    fn compare_endpoint(
        attempt: &ParsedAttempt,
        core: &Run,
        last_readable: &PublicDecision,
        last_map: Option<&spire_core::run::map::PublicMap>,
    ) -> Result<(), String> {
        let endpoint = &attempt.value["endpoint"];
        let current = core.decision();
        let available = current.is_ok();
        if endpoint["available"].as_bool() != Some(available) {
            return Err(problem("endpoint availability differs from direct core decision"));
        }
        let fallback = match &current {
            Ok(decision) => decision,
            Err(error) => {
                let expected_error = format!("{error:?}");
                if endpoint["decision_error"].as_str() != Some(expected_error.as_str()) {
                    return Err(problem("endpoint decision_error differs from direct core"));
                }
                last_readable
            }
        };
        let room = fallback.room.as_ref().map(|room| format!("{:?}", room.kind));
        let debug = decision_debug(fallback);
        if endpoint["act"].as_u64() != Some(fallback.act as u64)
            || endpoint["floor"].as_u64() != Some(fallback.floor as u64)
            || endpoint["room"].as_str() != room.as_deref()
            || endpoint["terminal"] != terminal_value(fallback.terminal)
            || endpoint["decision_debug"].as_str() != Some(debug.as_str())
        {
            return Err(problem("endpoint summary differs from live or last-readable core decision"));
        }
        let map = core.public_map().or_else(|| last_map.cloned());
        let debug = map_debug(map.as_ref());
        if endpoint["map_debug"].as_str() != debug.as_deref() {
            return Err(problem("endpoint map Debug differs from direct public map"));
        }
        Ok(())
    }

    fn compare_attempt(value: Value) -> Result<(), String> {
        let attempt = parse_receipt(value, None)?;
        let mut core = create_run(RunProfile::M15SilentHeart, &attempt.config, attempt.seed, 0)
            .map_err(|error| problem(format!("create canonical M15 core: {error:?}")))?;
        let first_decision = core.decision()
            .map_err(|error| problem(format!("read canonical M15 start: {error:?}")))?;
        let mut last_readable = first_decision.clone();
        let mut last_map = core.public_map();

        let mut session = RunSession::new_m15(attempt.config.clone())
            .map_err(|error| problem(format!("create M15 session: {error:?}")))?;
        let mut session_boundary = session.reset(Some(attempt.seed))
            .map_err(|error| problem(format!("reset M15 session: {error:?}")))?;
        compare_observation(
            "single-session reset", &session_boundary.observation, &first_decision,
            last_map.as_ref(), &attempt.config.capacities,
        )?;

        let mut batches = Vec::new();
        for workers in [1, 2] {
            let mut batch = RunBatch::new_m15(vec![attempt.config.clone(), attempt.config.clone()], workers)
                .map_err(|error| problem(format!("create {workers}-worker M15 batch: {error:?}")))?;
            let boundaries = batch.reset(&[Some(attempt.seed), Some(attempt.seed)])
                .map_err(|error| problem(format!("reset {workers}-worker M15 batch: {error:?}")))?;
            for (slot, boundary) in boundaries.iter().enumerate() {
                compare_observation(
                    &format!("{workers}-worker batch reset slot {slot}"), &boundary.observation,
                    &first_decision, last_map.as_ref(), &attempt.config.capacities,
                )?;
            }
            batches.push((workers, batch, boundaries));
        }

        let mut expected_record = attempt.record.clone();
        let mut accepted_post_fault = false;
        let mut expected_reproduction_fault: Option<RunSessionError> = None;
        for (ordinal, recorded) in attempt.record.steps.iter().enumerate() {
            let row = &attempt.value["steps"][ordinal];
            let decision = core.decision()
                .map_err(|error| problem(format!("accepted step {ordinal} has no live core decision: {error:?}")))?;
            let map = core.public_map();
            let pre_debug = decision_debug(&decision);
            if row["pre"]["decision_debug"].as_str() != Some(pre_debug.as_str())
                || row["pre"]["map_debug"].as_str() != map_debug(map.as_ref()).as_deref()
            {
                return Err(problem(format!("step {ordinal} pre-decision/map Debug differs from direct core")));
            }
            let candidate_index = attempt.candidate_indices[ordinal];
            let candidate = decision.candidates.get(candidate_index)
                .ok_or_else(|| problem(format!("step {ordinal} candidate_index {candidate_index} is not currently offered")))?;
            if candidate.action != recorded.action {
                return Err(problem(format!("step {ordinal} action differs from current candidate_index")));
            }

            let transition = core.step(&recorded.action)
                .map_err(|error| problem(format!("receipt marks step {ordinal} accepted but core rejected it: {error:?}")))?;
            let row_reward = row["transition"]["reward"].as_i64().unwrap();
            if transition.reward != recorded.reward || transition.terminal != recorded.terminal
                || transition.reward as i64 != row_reward
                || terminal_value(transition.terminal) != row["transition"]["terminal"]
            {
                return Err(problem(format!("step {ordinal} reward/terminal differs from receipt")));
            }
            let event_debug = transition.committed_events.iter().map(|event| format!("{event:?}")).collect::<Vec<_>>();
            let receipt_event_debug = row["transition"]["committed_events_debug"].as_array().unwrap()
                .iter().map(|event| event.as_str().unwrap().to_owned()).collect::<Vec<_>>();
            let receipt_events = row["transition"]["committed_events"].as_array().unwrap();
            if event_debug != receipt_event_debug || transition.committed_events.len() != receipt_events.len() {
                return Err(problem(format!("step {ordinal} committed event Debug/count differs from receipt")));
            }

            let post = core.decision();
            match &post {
                Ok(post_decision) => {
                    let debug = decision_debug(post_decision);
                    if row["post"]["decision"]["debug"].as_str() != Some(debug.as_str())
                        || !row["post"]["decision"]["error"].is_null()
                    {
                        return Err(problem(format!("step {ordinal} post-decision Debug differs from receipt")));
                    }
                    last_readable = post_decision.clone();
                    last_map = core.public_map();
                }
                Err(error) => {
                    let debug = format!("{error:?}");
                    if row["post"]["decision"]["error"].as_str() != Some(debug.as_str())
                        || !row["post"]["decision"]["debug"].is_null()
                    {
                        return Err(problem(format!("step {ordinal} post-decision error differs from receipt")));
                    }
                }
            }
            let post_map_debug = map_debug(core.public_map().as_ref());
            if row["post"]["map_debug"].as_str() != post_map_debug.as_deref() {
                return Err(problem(format!("step {ordinal} post-map Debug differs from receipt")));
            }

            let index = i64::try_from(candidate_index).map_err(|_| problem("candidate_index exceeds i64"))?;
            let single_result = session.step(session_boundary.token, RunActionIndex(index));
            let mut batch_results = Vec::new();
            for (workers, batch, boundaries) in &mut batches {
                let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
                let result = batch.step(&tokens, &[RunActionIndex(index); 2]);
                batch_results.push((*workers, result));
            }
            match &post {
                Ok(post_decision) => {
                    let single_step = single_result.map_err(|error| problem(format!(
                        "single session diverged at accepted step {ordinal}: {error:?}"
                    )))?;
                    if single_step.reward != transition.reward
                        || single_step.boundary.terminal != transition.terminal
                        || single_step.terminated != transition.terminal.is_some()
                        || single_step.truncated
                    {
                        return Err(problem(format!("single session reward/terminal differs at step {ordinal}")));
                    }
                    compare_observation(
                        &format!("single-session step {ordinal}"), &single_step.boundary.observation,
                        post_decision, core.public_map().as_ref(), &attempt.config.capacities,
                    )?;
                    session_boundary = single_step.boundary;
                    for (batch_index, (workers, result)) in batch_results.into_iter().enumerate() {
                        let steps = result.map_err(|error| problem(format!(
                            "{workers}-worker batch diverged at accepted step {ordinal}: {error:?}"
                        )))?;
                        if steps.len() != 2 {
                            return Err(problem(format!("{workers}-worker batch returned the wrong row count")));
                        }
                        for (slot, step) in steps.iter().enumerate() {
                            if step.reward != transition.reward || step.boundary.terminal != transition.terminal
                                || step.terminated != transition.terminal.is_some() || step.truncated
                            {
                                return Err(problem(format!("{workers}-worker batch slot {slot} reward/terminal differs at step {ordinal}")));
                            }
                            compare_observation(
                                &format!("{workers}-worker batch slot {slot} step {ordinal}"),
                                &step.boundary.observation, post_decision, core.public_map().as_ref(),
                                &attempt.config.capacities,
                            )?;
                        }
                        let current_boundaries = steps.into_iter().map(|step| step.boundary).collect();
                        let lane = batches.get_mut(batch_index)
                            .ok_or_else(|| problem("batch worker lane disappeared"))?;
                        lane.2 = current_boundaries;
                    }
                }
                Err(core_error) => {
                    if ordinal + 1 != attempt.record.steps.len() || attempt.failed_action.is_some() || attempt.error.is_none() {
                        return Err(problem("accepted post-decision core fault is not a final operational prefix"));
                    }
                    let core_cause = RunSessionError::Core(core_error.clone());
                    let adapter_error = RunSessionError::DeliveryFault("post-step Run encoding failed");
                    if single_result != Err(adapter_error.clone()) {
                        return Err(problem(format!("accepted post-decision adapter error differs: {single_result:?}")));
                    }
                    for (workers, result) in batch_results {
                        match result {
                            Err(RunBatchError::Fault { slots }) if slots.len() == 2 => {
                                for slot in slots {
                                    if slot.error != adapter_error || !slot.committed || slot.delivery_lost {
                                        return Err(problem(format!("{workers}-worker batch post-decision custody differs: {slot:?}")));
                                    }
                                }
                            }
                            other => return Err(problem(format!("{workers}-worker batch missed post-decision fault: {other:?}"))),
                        }
                    }
                    let action_value = &row["action"];
                    expected_record = expected_fault_record(
                        &attempt.record, action_value, &core_cause, true, true,
                    )?;
                    accepted_post_fault = true;
                    expected_reproduction_fault = Some(adapter_error);
                }
            }
        }

        if let Some(failed_action) = &attempt.failed_action {
            if accepted_post_fault {
                return Err(problem("receipt has failed_action after its accepted prefix already faulted"));
            }
            let failed_value = &attempt.value["failed_action"];
            let decision_before = core.decision()
                .map_err(|error| problem(format!("failed_action has no readable pre-attempt boundary: {error:?}")))?;
            let candidate_index = decision_before.candidates.iter()
                .position(|candidate| candidate.action == *failed_action);
            // Apply the failed action once to the direct canonical core.
            let direct_result = core.step(failed_action);
            match candidate_index {
                Some(candidate_index) => {
                    let index = i64::try_from(candidate_index).map_err(|_| problem("failed candidate index exceeds i64"))?;
                    let single_result = session.step(session_boundary.token, RunActionIndex(index));
                    let mut batch_results = Vec::new();
                    for (workers, batch, boundaries) in &mut batches {
                        let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
                        batch_results.push((*workers, batch.step(&tokens, &[RunActionIndex(index); 2])));
                    }
                    match direct_result {
                        Err(core_error) => {
                            let expected_error = RunSessionError::Core(core_error.clone());
                            if single_result != Err(expected_error.clone()) {
                                return Err(problem(format!("failed action differs in native session: {single_result:?}")));
                            }
                            let committed = matches!(core_error, spire_core::run::RunError::Faulted(_));
                            for (workers, result) in batch_results {
                                match result {
                                    Err(RunBatchError::Fault { slots }) if slots.len() == 2 => {
                                        for slot in slots {
                                            if slot.error != expected_error || slot.committed != committed || slot.delivery_lost {
                                                return Err(problem(format!("{workers}-worker failed-action custody differs: {slot:?}")));
                                            }
                                        }
                                    }
                                    other => return Err(problem(format!("{workers}-worker batch did not retain failed action: {other:?}"))),
                                }
                            }
                            expected_record = expected_fault_record(
                                &attempt.record, failed_value, &expected_error, committed, false,
                            )?;
                            expected_reproduction_fault = Some(expected_error);
                        }
                        Ok(transition) => {
                            if !matches!(single_result, Ok(_)) {
                                return Err(problem(format!("core accepted failed_action but native session rejected it: {single_result:?}")));
                            }
                            for (workers, result) in batch_results {
                                if !matches!(result, Ok(_)) {
                                    return Err(problem(format!("core accepted failed_action but {workers}-worker batch rejected it: {result:?}")));
                                }
                            }
                            return Err(problem(format!("receipt marks accepted failed_action as error (reward {})", transition.reward)));
                        }
                    }
                }
                None => {
                    if !matches!(direct_result, Err(spire_core::run::RunError::InvalidAction)) {
                        return Err(problem(format!("unoffered failed_action produced unexpected core result: {direct_result:?}")));
                    }
                    // The adapters accept numeric offered-action indices. An
                    // unoffered GameAction has no valid index; reject before
                    // mutating either owner instead of substituting an action.
                    let before = session.reproduction()
                        .map_err(|error| problem(format!("read session before invalid index: {error:?}")))?;
                    if !matches!(
                        session.preflight_step(session_boundary.token, RunActionIndex(i64::MAX)),
                        Err(RunSessionError::Input(_)),
                    ) {
                        return Err(problem("session did not reject an unoffered failed-action index"));
                    }
                    let mut batch_reproductions = Vec::new();
                    for (workers, batch, boundaries) in &mut batches {
                        let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
                        if !matches!(
                            batch.step(&tokens, &[RunActionIndex(i64::MAX); 2]),
                            Err(RunBatchError::Input { .. }),
                        ) {
                            return Err(problem(format!("{workers}-worker batch did not preflight invalid index")));
                        }
                        batch_reproductions.push((*workers, batch.reproduction(0)
                            .map_err(|error| problem(format!("read {workers}-worker reproduction: {error:?}")))?));
                    }
                    if session.reproduction()
                        .map_err(|error| problem(format!("read session after invalid index: {error:?}")))? != before
                    {
                        return Err(problem("unoffered failed_action mutated the single session prefix"));
                    }
                    for (workers, reproduction) in batch_reproductions {
                        if reproduction != before {
                            return Err(problem(format!("{workers}-worker batch mutated on invalid index")));
                        }
                    }
                }
            }
        }

        let expected_steps = expected_record.steps.len();
        let single_record = session.record()
            .map_err(|error| problem(format!("read final session record: {error:?}")))?
            .ok_or_else(|| problem("native session lost its replay record"))?;
        if single_record != expected_record {
            return Err(problem(format!(
                "single session record differs from caller prefix/fault (expected {expected_steps} steps, got {})",
                single_record.steps.len(),
            )));
        }
        let single_reproduction = session.reproduction()
            .map_err(|error| problem(format!("read final session reproduction: {error:?}")))?;
        if single_reproduction.fault != expected_reproduction_fault {
            return Err(problem(format!(
                "single reproduction fault differs from accepted/failing core attempt: {:?}",
                single_reproduction.fault,
            )));
        }
        for (workers, batch, _) in &batches {
            for slot in 0..2 {
                let batch_record = batch.record(slot)
                    .map_err(|error| problem(format!("read {workers}-worker record {slot}: {error:?}")))?
                    .ok_or_else(|| problem(format!("{workers}-worker batch slot {slot} lost its record")))?;
                if batch_record != expected_record {
                    return Err(problem(format!("{workers}-worker batch slot {slot} record differs")));
                }
                let reproduction = batch.reproduction(slot)
                    .map_err(|error| problem(format!("read {workers}-worker reproduction {slot}: {error:?}")))?;
                if reproduction != single_reproduction
                    || reproduction.latest_decision.as_ref() != Some(&core.decision())
                    || reproduction.latest_map != core.public_map()
                    || reproduction.last_accepted_step.as_ref() != expected_record.steps.last()
                {
                    return Err(problem(format!("{workers}-worker batch slot {slot} reproduction differs")));
                }
            }
        }
        if single_reproduction.latest_decision.as_ref() != Some(&core.decision())
            || single_reproduction.latest_map != core.public_map()
            || single_reproduction.last_accepted_step.as_ref() != expected_record.steps.last()
        {
            return Err(problem("single session reproduction differs from direct core prefix"));
        }

        match crate::replay_run(&expected_record, attempt.config.capacities.clone()) {
            Ok(report) => compare_report(&expected_record, &report, &last_readable, &core)?,
            Err(error) if accepted_post_fault => {
                if Some(error.clone()) != core.decision().err().map(RunSessionError::Core) {
                    return Err(problem(format!("portable replay post-step fault differs from direct core: {error:?}")));
                }
            }
            Err(error) => return Err(problem(format!("portable accepted-prefix replay failed: {error:?}"))),
        }
        compare_endpoint(&attempt, &core, &last_readable, last_map.as_ref())?;
        if attempt.error.is_some() && attempt.failed_action.is_none() && accepted_post_fault {
            if attempt.outcome.is_some() {
                return Err(problem("accepted post-step core fault was classified as an outcome"));
            }
        } else {
            compare_outcome(&attempt, &core.decision())?;
        }
        Ok(())
    }

    fn action_value_from_record(
        config: &RunSessionConfig,
        seed: u64,
        action: &GameAction,
        reward: i32,
        terminal: Option<RunTerminal>,
    ) -> Result<Value, String> {
        let mut session = RunSession::new_m15(config.clone())
            .map_err(|error| problem(format!("create action codec session: {error:?}")))?;
        session.reset(Some(seed))
            .map_err(|error| problem(format!("reset action codec session: {error:?}")))?;
        let mut record = session.record()
            .map_err(|error| problem(format!("read action codec record: {error:?}")))?
            .ok_or_else(|| problem("action codec session has no replay record"))?;
        record.steps.push(crate::RunRecordedStep { action: action.clone(), reward, terminal });
        let bytes = record.to_json()
            .map_err(|error| problem(format!("serialize action through RunEpisodeRecord: {error:?}")))?;
        let wire = crate::run_replay::parse_unique_json(&bytes)
            .map_err(|error| problem(format!("parse action RunEpisodeRecord JSON: {error:?}")))?;
        wire["steps"].as_array().and_then(|steps| steps.first()).and_then(|step| step.get("action"))
            .cloned().ok_or_else(|| problem("serialized Run action is missing"))
    }

    fn one_neow_diagnostic() -> Result<Value, String> {
        let case = FROZEN_CASES[0];
        let config = config_for(case)?;
        let seed = case.seed;
        // No caller or policy is used: take public action 0 once from the
        // shared private native M15 constructor also used by RunSession.
        let mut core = create_run(RunProfile::M15SilentHeart, &config, seed, 0)
            .map_err(|error| problem(format!("create one-Neow M15 diagnostic: {error:?}")))?;
        let pre = core.decision().map_err(|error| problem(format!("read Neow decision: {error:?}")))?;
        if pre.room.as_ref().map(|room| room.kind) != Some(spire_core::run::RoomViewKind::Neow) {
            return Err(problem("one-Neow diagnostic did not begin at Neow"));
        }
        let action = pre.candidates.get(0)
            .ok_or_else(|| problem("Neow has no public action 0"))?.action.clone();
        let action_value = action_value_from_record(&config, seed, &action, 0, None)?;
        let pre_map = core.public_map();
        let transition = core.step(&action)
            .map_err(|error| problem(format!("public Neow action 0 failed: {error:?}")))?;
        let post = core.decision()
            .map_err(|error| problem(format!("read post-Neow decision: {error:?}")))?;
        let post_map = core.public_map();
        let event_debug = transition.committed_events.iter()
            .map(|event| format!("{event:?}")).collect::<Vec<_>>();
        if !transition.committed_events.is_empty() {
            return Err(problem("one-Neow diagnostic unexpectedly committed events; serializer remains caller-owned"));
        }
        Ok(json!({
            "schema_id": RECEIPT_SCHEMA,
            "seed": seed,
            "scenario": {"act1_boss": case.act1, "act2_boss": case.act2, "act3_boss": case.act3},
            "caller_revision": CALLER_REVISION,
            "action_budget": 1,
            "outcome": "BudgetExhausted",
            "error": null,
            "accepted_action_count": 1,
            "actions": [action_value.clone()],
            "steps": [{
                "index": 0,
                "candidate_index": 0,
                "pre": {
                    "decision_debug": decision_debug(&pre),
                    "map_debug": map_debug(pre_map.as_ref()),
                },
                "action": action_value,
                "transition": {
                    "reward": transition.reward,
                    "terminal": terminal_value(transition.terminal),
                    "committed_events": [],
                    "committed_events_debug": event_debug,
                },
                "post": {
                    "decision": {"debug": decision_debug(&post), "error": null},
                    "map_debug": map_debug(post_map.as_ref()),
                },
            }],
            "failed_action": null,
            "endpoint": {
                "available": true,
                "decision_error": null,
                "act": post.act,
                "floor": post.floor,
                "room": post.room.as_ref().map(|room| format!("{:?}", room.kind)),
                "terminal": terminal_value(post.terminal),
                "decision_debug": decision_debug(&post),
                "map_debug": map_debug(post_map.as_ref()),
            },
        }))
    }

    fn parse_receipt_bytes(bytes: &[u8], expected: FrozenCase) -> Result<Value, String> {
        let value = crate::run_replay::parse_unique_json(bytes)
            .map_err(|error| problem(format!("raw M15 receipt rejected: {error:?}")))?;
        parse_receipt(value.clone(), Some(expected))?;
        Ok(value)
    }

    #[test]
    fn m15_canonical_attempt_consumer_accepts_supplied_one_neow_receipt() {
        // A deterministic one-action caller-shaped diagnostic is constructed
        // from the native M15 core and versioned Run action exporter.
        let receipt = one_neow_diagnostic().unwrap();
        let bytes = serde_json::to_vec(&receipt).unwrap();
        let value = crate::run_replay::parse_unique_json(&bytes).unwrap();
        compare_attempt(value).unwrap();

        let raw = String::from_utf8(bytes).unwrap();
        let field = format!("\"schema_id\":\"{RECEIPT_SCHEMA}\"");
        let duplicate = raw.replacen(&field, &format!("{field},{field}"), 1);
        assert_ne!(duplicate, raw);
        assert!(crate::run_replay::parse_unique_json(duplicate.as_bytes()).is_err());

        let mut bad_snapshot = receipt.clone();
        bad_snapshot["steps"][0]["pre"]["decision_debug"] = Value::Null;
        assert!(parse_receipt(bad_snapshot, None).is_err());

        let mut changed_snapshot=receipt.clone();
        changed_snapshot["steps"][0]["pre"]["decision_debug"]=json!("changed public snapshot");
        assert!(compare_attempt(changed_snapshot).is_err());

        let mut bad_candidate_index = receipt;
        bad_candidate_index["steps"][0]["candidate_index"] = json!(u64::MAX);
        assert!(compare_attempt(bad_candidate_index).is_err());
    }

    #[test]
    fn m15_canonical_attempt_consumer_preserves_operational_error_prefix() {
        let mut receipt = one_neow_diagnostic().unwrap();
        receipt["outcome"] = Value::Null;
        receipt["error"] = json!("synthetic caller-side diagnostic error");
        receipt["failed_action"] = Value::Null;
        compare_attempt(receipt.clone()).unwrap();
        let case=FROZEN_CASES[0];
        let config=config_for(case).unwrap();
        receipt["failed_action"]=action_value_from_record(
            &config,case.seed,&GameAction::Move(spire_core::run::NodeId(u64::MAX)),0,None,
        ).unwrap();
        compare_attempt(receipt).unwrap();
    }

    #[test]
    #[ignore = "HELD pending integrator freeze and explicit release of all six original M15 attempt receipts"]
    fn m15_frozen_original_attempts_have_native_parity() {
        use sha2::Digest;

        let directory = std::env::var_os("SPIRE_M15_NATIVE_ATTEMPT_DIR")
            .map(std::path::PathBuf::from)
            .expect("SPIRE_M15_NATIVE_ATTEMPT_DIR must name the held six-receipt directory");
        let mut files = fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("read held M15 receipt directory {}: {error}", directory.display()))
            .map(|entry| entry.expect("read M15 receipt directory entry").path())
            .filter(|path| path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with(".attempt.json")))
            .collect::<Vec<_>>();
        files.sort();
        assert_eq!(files.len(), 6, "held lane requires exactly six attempt JSON receipts");

        let expected_paths = FROZEN_CASES.iter().map(|case| {
            directory.join(format!("seed-{}-{}-{}-{}.attempt.json", case.seed, case.act1, case.act2, case.act3))
        }).collect::<Vec<_>>();
        let mut actual_names = files.iter().map(|path| path.file_name().unwrap().to_owned()).collect::<Vec<_>>();
        let mut expected_names = expected_paths.iter().map(|path| path.file_name().unwrap().to_owned()).collect::<Vec<_>>();
        actual_names.sort();
        expected_names.sort();
        assert_eq!(actual_names, expected_names, "receipt filenames differ from the frozen six seed/triple pairs");

        for (path, expected) in expected_paths.iter().zip(FROZEN_CASES) {
            let bytes = fs::read(path)
                .unwrap_or_else(|error| panic!("read held receipt {}: {error}", path.display()));
            let value = parse_receipt_bytes(&bytes, expected)
                .unwrap_or_else(|error| panic!("invalid held receipt {}: {error}", path.display()));
            let accepted = value["accepted_action_count"].as_u64().unwrap_or_default();
            let outcome = value["outcome"].as_str().unwrap_or("<error>");
            let error = value["error"].as_str().unwrap_or("<none>");
            let digest = sha2::Sha256::digest(&bytes).iter()
                .map(|byte| format!("{byte:02x}")).collect::<String>();
            println!(
                "M15_ATTEMPT path={} sha256={} accepted={} outcome={} error={}",
                path.display(), digest, accepted, outcome, error,
            );
            compare_attempt(value)
                .unwrap_or_else(|error| panic!("native parity failed for {}: {error}", path.display()));
        }
    }
}

#[cfg(test)]
mod m17_boundary_tests{
    use super::*;
    #[test]
    fn m17_native_cutoff_and_sidecar_are_healthy_boundaries(){
        let cfg=RunSessionConfig{character:Character::Defect,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M17DefectAct1{act1:RunAct1BossTarget::Guardian},capacities:RunCapacities::m10_default(),record_replay:true};
        let mut s=RunSession::new_m17(cfg.clone()).unwrap();let initial=s.reset(Some(7)).unwrap();
        // Transport diagnostic of the exact cutoff publication; actual boss,
        // child, heal and map initialization are covered by core owner tests.
        let mut d=create_run(RunProfile::M17DefectAct1,&cfg,7,0).unwrap().decision().unwrap();d.act=2;d.floor=17;d.candidates.clear();d.room.as_mut().unwrap().kind=RoomViewKind::Act2Ready;d.room.as_mut().unwrap().choices.clear();
        assert_eq!(check_boundary(RunProfile::M17DefectAct1,&d),Ok(()));
        let observation=crate::encode_run_m17(&d,None,&cfg.capacities).unwrap();s.published=Some(RunBoundary{observation,token:initial.token,terminal:None});s.actions.clear();s.at_m12_cutoff=true;
        let before=s.reproduction().unwrap();let b=s.decision().unwrap();assert!(b.terminal.is_none());assert!(b.observation.action_mask.iter().all(|v|*v==0));assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M17 stops at Act2Ready")));assert_eq!(s.reproduction().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
        let reset=s.reset(Some(11)).unwrap();assert_ne!(reset.token,b.token);assert!(!s.at_m12_cutoff);assert_eq!(s.reproduction().unwrap().latest_decision.unwrap().unwrap().player.hp,75);s.close().unwrap();
    }
}
