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
pub const RUN_RECORD_SCHEMA_ID_M22:&str="spire-gym-m22-watcher-act1-replay-v1";
pub const RUN_RULES_ID_M22:&str="spire-gym-native-m22-rules-v1";
pub const RUN_CONTENT_ID_M22:&str="spire-gym-m22-watcher-a0-act1-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M23:&str="spire-gym-m23-watcher-act2-replay-v1";
pub const RUN_RULES_ID_M23:&str="spire-gym-native-m23-rules-v1";
pub const RUN_CONTENT_ID_M23:&str="spire-gym-m23-watcher-a0-acts1-2-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M24:&str="spire-gym-m24-watcher-act3-replay-v1";
pub const RUN_RULES_ID_M24:&str="spire-gym-native-m24-rules-v1";
pub const RUN_CONTENT_ID_M24:&str="spire-gym-m24-watcher-a0-acts1-3-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M25:&str="spire-gym-m25-watcher-heart-replay-v1";
pub const RUN_RULES_ID_M25:&str="spire-gym-native-m25-rules-v1";
pub const RUN_CONTENT_ID_M25:&str="spire-gym-m25-watcher-a0-heart-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M20:&str="spire-gym-m20-defect-heart-replay-v1";
pub const RUN_RULES_ID_M20:&str="spire-gym-native-m20-rules-v1";
pub const RUN_CONTENT_ID_M20:&str="spire-gym-m20-defect-a0-heart-captured-v1";
pub const RUN_RECORD_SCHEMA_ID_M19:&str="spire-gym-m19-defect-act3-replay-v1";
pub const RUN_RULES_ID_M19:&str="spire-gym-native-m19-rules-v1";
pub const RUN_CONTENT_ID_M19:&str="spire-gym-m19-defect-a0-acts1-3-captured-v1";
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
pub const RUN_RECORD_SCHEMA_ID_STANDARD:&str="spire-gym-standard-run-replay-v1";
/// v2 since M34s-a: G-a4 (autoplay free-play lifecycle) and R-a1 (Secret Technique/Weapon candidate order) change Standard gameplay.
pub const RUN_RULES_ID_STANDARD:&str="spire-gym-native-standard-rules-v2";
pub const RUN_CONTENT_ID_STANDARD:&str="spire-gym-standard-content-v1";
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
    M19DefectAct3,
    M20DefectHeart,
    M22WatcherAct1,
    M23WatcherAct2,
    M24WatcherAct3,
    M25WatcherHeart,
    /// M34b: the core Standard Run (every character, A0–A20; design §4).
    Standard,
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
            Self::M19DefectAct3 => "M19DefectAct3",
            Self::M20DefectHeart => "M20DefectHeart",
            Self::M22WatcherAct1 => "M22WatcherAct1",
            Self::M23WatcherAct2 => "M23WatcherAct2",
            Self::M24WatcherAct3 => "M24WatcherAct3",
            Self::M25WatcherHeart => "M25WatcherHeart",
            Self::Standard => "Standard",
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
            Self::M19DefectAct3 => RUN_RECORD_SCHEMA_ID_M19,
            Self::M20DefectHeart => RUN_RECORD_SCHEMA_ID_M20,
            Self::M22WatcherAct1 => RUN_RECORD_SCHEMA_ID_M22,
            Self::M23WatcherAct2 => RUN_RECORD_SCHEMA_ID_M23,
            Self::M24WatcherAct3 => RUN_RECORD_SCHEMA_ID_M24,
            Self::M25WatcherHeart => RUN_RECORD_SCHEMA_ID_M25,
            Self::Standard => RUN_RECORD_SCHEMA_ID_STANDARD,
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
            Self::M19DefectAct3 => RUN_RULES_ID_M19,
            Self::M20DefectHeart => RUN_RULES_ID_M20,
            Self::M22WatcherAct1 => RUN_RULES_ID_M22,
            Self::M23WatcherAct2 => RUN_RULES_ID_M23,
            Self::M24WatcherAct3 => RUN_RULES_ID_M24,
            Self::M25WatcherHeart => RUN_RULES_ID_M25,
            Self::Standard => RUN_RULES_ID_STANDARD,
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
            Self::M19DefectAct3 => RUN_CONTENT_ID_M19,
            Self::M20DefectHeart => RUN_CONTENT_ID_M20,
            Self::M22WatcherAct1 => RUN_CONTENT_ID_M22,
            Self::M23WatcherAct2 => RUN_CONTENT_ID_M23,
            Self::M24WatcherAct3 => RUN_CONTENT_ID_M24,
            Self::M25WatcherHeart => RUN_CONTENT_ID_M25,
            Self::Standard => RUN_CONTENT_ID_STANDARD,
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
            Self::M19DefectAct3 => crate::m19_run_observation_spec(caps),
            Self::M20DefectHeart => crate::m20_run_observation_spec(caps),
            Self::M22WatcherAct1 => crate::m22_run_observation_spec(caps),
            Self::M23WatcherAct2 => crate::m23_run_observation_spec(caps),
            Self::M24WatcherAct3 => crate::m24_run_observation_spec(caps),
            Self::M25WatcherHeart => crate::m25_run_observation_spec(caps),
            Self::Standard => crate::standard_run_observation_spec(caps),
        }
    }
    fn validate(self, config: &RunSessionConfig) -> Result<(), RunSessionError> {
        if self == Self::Standard {
            if config.ascension > spire_core::run::STANDARD_MAX_ASCENSION { return Err(RunSessionError::Input("Standard supports ascension 0 to 20")) }
            let RunBossHistory::Standard(creation) = config.boss_history else { return Err(RunSessionError::Input("Standard requires a Standard creation profile")) };
            creation.validate()?;
            crate::run_schema_standard::validate(&config.capacities)?;
            // The core owns creation legality (the Note must be storable with legal upgrades): probe it once so invalid
            // input is rejected before any episode Run is built (design §4.2).
            return Run::new_room_scenario(RunConfig{character:config.character,ascension:config.ascension},creation.profile(),
                RunRng::new([0;32],config.partition,0),RoomScenario::Standard).map(|_|()).map_err(|_|RunSessionError::Input("invalid Standard creation profile"));
        }
        let expected_character = if matches!(self,Self::M22WatcherAct1|Self::M23WatcherAct2|Self::M24WatcherAct3|Self::M25WatcherHeart) { Character::Watcher } else if matches!(self,Self::M17DefectAct1|Self::M18DefectAct2|Self::M19DefectAct3|Self::M20DefectHeart) { Character::Defect } else if matches!(self, Self::M12SilentAct1 | Self::M13SilentAct2 | Self::M14SilentAct3 | Self::M15SilentHeart) {
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
            Self::M19DefectAct3 => "M19 requires Defect A0",
            Self::M20DefectHeart => "M20 requires Defect A0",
            Self::M22WatcherAct1 => "M22 requires Watcher A0",
            Self::M23WatcherAct2 => "M23 requires Watcher A0",
            Self::M24WatcherAct3 => "M24 requires Watcher A0",
            Self::M25WatcherHeart => "M25 requires Watcher A0",
            Self::Standard => unreachable!("Standard validated above"),
            }));
        }
        let history_matches = match self {
            Self::M17DefectAct1 => matches!(config.boss_history,RunBossHistory::M17DefectAct1{..}),
            Self::M18DefectAct2 => matches!(config.boss_history,RunBossHistory::M18DefectAct2{..}),
            Self::M19DefectAct3 => matches!(config.boss_history,RunBossHistory::M19DefectAct3{..}),
            Self::M20DefectHeart => matches!(config.boss_history,RunBossHistory::M20DefectHeart{..}),
            Self::M23WatcherAct2 => matches!(config.boss_history,RunBossHistory::M23WatcherAct2{..}),
            Self::M24WatcherAct3 => matches!(config.boss_history,RunBossHistory::M24WatcherAct3{..}),
            Self::M25WatcherHeart => matches!(config.boss_history,RunBossHistory::M25WatcherHeart{..}),
            Self::M12SilentAct1 | Self::M22WatcherAct1 => matches!(
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
            Self::M19DefectAct3 => crate::run_schema_m19::validate(&config.capacities),
            Self::M20DefectHeart => crate::run_schema_m20::validate(&config.capacities),
            Self::M22WatcherAct1 => crate::run_schema_m22::validate(&config.capacities),
            Self::M23WatcherAct2 => crate::run_schema_m23::validate(&config.capacities),
            Self::M24WatcherAct3 => crate::run_schema_m24::validate(&config.capacities),
            Self::M25WatcherHeart => crate::run_schema_m25::validate(&config.capacities),
            Self::Standard => unreachable!("Standard validated above"),
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
            (RUN_RECORD_SCHEMA_ID_M19, "M19DefectAct3") => Ok(Self::M19DefectAct3),
            (RUN_RECORD_SCHEMA_ID_M20, "M20DefectHeart") => Ok(Self::M20DefectHeart),
            (RUN_RECORD_SCHEMA_ID_M22, "M22WatcherAct1") => Ok(Self::M22WatcherAct1),
            (RUN_RECORD_SCHEMA_ID_M23, "M23WatcherAct2") => Ok(Self::M23WatcherAct2),
            (RUN_RECORD_SCHEMA_ID_M24, "M24WatcherAct3") => Ok(Self::M24WatcherAct3),
            (RUN_RECORD_SCHEMA_ID_M25, "M25WatcherHeart") => Ok(Self::M25WatcherHeart),
            (RUN_RECORD_SCHEMA_ID_STANDARD, "Standard") => Ok(Self::Standard),
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
    M20DefectHeart { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3:RunAct3BossTarget },
    M19DefectAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3:RunAct3BossTarget },
    M13SilentAct2 { act1: RunAct1BossTarget, act2: RunAct2BossTarget },
    M14SilentAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
    M15SilentHeart { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
    M23WatcherAct2 { act1: RunAct1BossTarget, act2: RunAct2BossTarget },
    M24WatcherAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
    M25WatcherHeart { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget },
    /// M34b: the Standard creation profile (internal carrier; the public API is `StandardRunConfig`).
    Standard(StandardCreation),
}
/// Neow at the Standard start: the full blessing screen, or the short (ineligible) screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardNeow { Full, Short }
/// M34b explicit Standard creation inputs (design §4.2). Every reset builds a fresh Run from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StandardCreation {
    pub neow: StandardNeow,
    /// Seen bosses per act, indexed by the act's boss target order (Act 1 Guardian/Hexaghost/SlimeBoss, …).
    pub seen_bosses: [[bool; 3]; 3],
    /// Note For Yourself's stored card and upgrades (the core checks it is storable).
    pub stored_note: (spire_core::content::CardId, u32),
}
impl StandardCreation {
    /// `CreationProfile::experienced()`: full Neow, every boss seen, Iron Wave+0.
    pub fn experienced() -> Self { Self { neow: StandardNeow::Full, seen_bosses: [[true; 3]; 3], stored_note: (spire_core::content::CardId::IronWave, 0) } }
    const BOSSES: [[&'static str; 3]; 3] = [["Guardian", "Hexaghost", "SlimeBoss"], ["Champ", "BronzeAutomaton", "Collector"], ["AwakenedOne", "TimeEater", "DonuAndDeca"]];
    pub(crate) fn validate(&self) -> Result<(), RunSessionError> {
        if matches!(self.stored_note.0, spire_core::content::CardId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard stored note card")) }
        Ok(())
    }
    /// The core creation profile this Standard creation builds every episode from.
    pub fn profile(&self) -> CreationProfile {
        let mut profile = CreationProfile::experienced();
        profile.neow_eligible = self.neow == StandardNeow::Full;
        profile.seen_bosses = std::array::from_fn(|act| (0..3).filter(|&i| self.seen_bosses[act][i]).map(|i| Self::BOSSES[act][i]).collect());
        profile.stored_note = self.stored_note;
        profile
    }
    /// Strict inverse of `to_json` (record codec and Python): unknown keys, wrong-act or unknown bosses, unknown Note cards are `Input`.
    pub fn from_json(value:&serde_json::Value)->Result<Self,RunSessionError>{crate::run_replay::parse_standard_creation(value)}
    /// `from_json` over JSON text (for callers without a `serde_json` dependency, e.g. the Python binding).
    pub fn from_json_str(text:&str)->Result<Self,RunSessionError>{let value=serde_json::from_str(text).map_err(|_|RunSessionError::Input("invalid Standard creation profile"))?;Self::from_json(&value)}
    /// `to_json` as canonical JSON text.
    pub fn to_json_string(&self)->String{self.to_json().to_string()}
    /// Public JSON form (environment spec, reset info, record): player knowledge, never a per-step column.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({"neow": match self.neow { StandardNeow::Full => "full", StandardNeow::Short => "short" },
            "seen_bosses": (0..3).map(|act| (0..3).filter(|&i| self.seen_bosses[act][i]).map(|i| Self::BOSSES[act][i]).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "stored_note": {"card": self.stored_note.0.as_str(), "upgrades": self.stored_note.1}})
    }
}
/// M34b public Standard session configuration (design §4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandardRunConfig {
    pub character: Character, pub ascension: u8, pub partition: Partition,
    pub creation: StandardCreation, pub capacities: RunCapacities, pub record_replay: bool,
}
impl StandardRunConfig {
    /// Default creation and the `standard-default-v1` capacities.
    pub fn new(character: Character, ascension: u8, partition: Partition) -> Self {
        Self { character, ascension, partition, creation: StandardCreation::experienced(), capacities: RunCapacities::standard_default(), record_replay: false }
    }
    pub(crate) fn session_config(&self) -> RunSessionConfig {
        RunSessionConfig { character: self.character, ascension: self.ascension, partition: self.partition, boss_history: RunBossHistory::Standard(self.creation), capacities: self.capacities.clone(), record_replay: self.record_replay }
    }
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
pub struct RunStepResult{pub boundary:RunBoundary,pub reward:i32,pub terminated:bool,pub truncated:bool,
    /// M34b: the itemised credits of this transition (PoC §7); they sum to `reward`.
    pub reward_events:Vec<RewardItem>}
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
    #[cfg(test)]
    m22_diagnostic:Option<spire_core::run::diagnostics::M22Diagnostic>,
    #[cfg(test)]
    m23_diagnostic:Option<spire_core::run::diagnostics::M23Diagnostic>,
    #[cfg(test)]
    m24_diagnostic:Option<M24DiagnosticStart>,
    #[cfg(test)]
    m25_diagnostic:Option<spire_core::run::diagnostics::M25Diagnostic>,
    #[cfg(test)]
    test_reset_panic:Option<TestResetPanic>,
    #[cfg(test)]
    m20_diagnostic:Option<spire_core::run::diagnostics::M20DiagnosticLoadout>,
    config:RunSessionConfig,profile:RunProfile,id:u64,serial:u64,run:Option<Run>,published:Option<RunBoundary>,
    actions:Vec<GameAction>,seed:Option<u64>,ordinal:u64,record:Option<RunEpisodeRecord>,
    fault:Option<RunSessionError>,last_step:Option<RunRecordedStep>,closed:bool,at_m12_cutoff:bool,at_m13_cutoff:bool,at_m14_cutoff:bool,
}
/// M34c privileged exact-world checkpoint (design §4.8): an opaque, immutable deep copy of one session's episode (core
/// Run, config and creation, seed, ordinal, published boundary, action table, record prefix, last step). It reveals the
/// hidden future (restoring replays the same draws), so it is an oracle, never an observation: no accessor, no
/// `Clone`/`PartialEq`/`Hash` (state equality would be a hidden-state side channel) and a redacted `Debug`.
/// In-memory only; the portable form is the record.
pub struct ExactCheckpoint{state:Box<RunSession>}
impl std::fmt::Debug for ExactCheckpoint{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("ExactCheckpoint { .. }")}}
/// TEST-ONLY reset worker-panic stage for the shared RunBatch catch_unwind path. Not a production hook.
#[cfg(test)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum TestResetPanic{BeforeMutation,AfterStartup}
#[cfg(test)]
thread_local!{static M22_INITIALIZATIONS:std::cell::RefCell<Vec<u64>>=const{std::cell::RefCell::new(Vec::new())};}
#[cfg(test)]
thread_local!{static M23_INITIALIZATIONS:std::cell::RefCell<Vec<u64>>=const{std::cell::RefCell::new(Vec::new())};}
// TEST-ONLY M24 initializer entries: ordinal and the installed seen-boss complements, recorded immediately before Run startup.
#[cfg(test)]
thread_local!{static M24_INITIALIZATIONS:std::cell::RefCell<Vec<(u64,[Vec<&'static str>;3])>>=const{std::cell::RefCell::new(Vec::new())};}
/// TEST-ONLY fixed M24 diagnostic starts from spire_core::run::diagnostics (constructed or supplied provenance; never config/JSON).
#[cfg(test)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum M24DiagnosticStart{Stage(spire_core::run::diagnostics::M24Diagnostic),Traversal(spire_core::run::diagnostics::M24DiagnosticBoss),Boss(spire_core::run::diagnostics::M24DiagnosticBoss,[bool;3],u64),Played(spire_core::run::diagnostics::M24PlayedFixture)}
#[cfg(test)]
impl M24DiagnosticStart{pub(crate) fn run(self)->Result<Run,RunError>{use spire_core::run::diagnostics as d;match self{Self::Stage(s)=>d::m24_diagnostic(s),Self::Traversal(b)=>d::m24_supplied_traversal(b),Self::Boss(b,keys,seed)=>d::m24_supplied_boss_at(b,keys,seed),Self::Played(f)=>d::m24_played_fixture(f)}}
    /// Supplied bosses run at seed 7 or 11 (design §11); every other start is seed 7.
    pub(crate) fn seed(self)->u64{if let Self::Boss(_,_,seed)=self{seed}else{7}}}
/// Fixed M25 supplied starts used only by the frozen native agreement test.
#[cfg(test)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum M25SuppliedStart{
    Traversal(spire_core::run::diagnostics::M25DiagnosticBoss),
    Act4(spire_core::run::diagnostics::M25DiagnosticLoadout),
    PlayedFixture(spire_core::run::diagnostics::M25PlayedFixture),
}
#[cfg(test)]
impl M25SuppliedStart{
    pub(crate) fn run(self)->Result<Run,RunError>{use spire_core::run::diagnostics as d;match self{
        Self::Traversal(boss)=>d::m25_supplied_traversal(boss),
        Self::Act4(loadout)=>d::m25_supplied_act4(loadout),
        Self::PlayedFixture(fixture)=>d::m25_played_fixture(fixture),
    }}
    fn bootstrap_action(self,run:&Run)->Result<Option<GameAction>,RunError>{match self{
        Self::PlayedFixture(_)=>Ok(Some(run.decision()?.candidates.into_iter().find_map(|candidate|match &candidate.action{GameAction::Move(_)=>Some(candidate.action),_=>None}).ok_or(RunError::InvalidAction)?)),
        _=>Ok(None),
    }}
}
pub(crate) fn create_run(
    profile: RunProfile,
    config: &RunSessionConfig,
    seed: u64,
    ordinal: u64,
) -> Result<Run, RunSessionError> {
    #[cfg(test)] if profile==RunProfile::M22WatcherAct1{M22_INITIALIZATIONS.with(|v|v.borrow_mut().push(ordinal));}
    #[cfg(test)] if profile==RunProfile::M23WatcherAct2{M23_INITIALIZATIONS.with(|v|v.borrow_mut().push(ordinal));}
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    if profile == RunProfile::Standard {
        let RunBossHistory::Standard(creation) = config.boss_history else { return Err(RunSessionError::Input("Standard requires a Standard creation profile")) };
        return Run::new_room_scenario(RunConfig { character: config.character, ascension: config.ascension }, creation.profile(),
            RunRng::new(root, config.partition, ordinal), RoomScenario::Standard).map_err(RunSessionError::Core);
    }
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    creation.seen_bosses[0] = match config.boss_history {
        RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget::SlimeBoss } => vec!["Guardian", "Hexaghost"],
        RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget::Hexaghost } => vec!["Guardian", "SlimeBoss"],
        RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget::Guardian } => vec!["Hexaghost", "SlimeBoss"],
        RunBossHistory::M20DefectHeart { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M19DefectAct3 { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M23WatcherAct2 { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M24WatcherAct3 { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M25WatcherHeart { act1: RunAct1BossTarget::Guardian, .. } |
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M15SilentHeart { act1: RunAct1BossTarget::Guardian, .. } => {
            vec!["Hexaghost", "SlimeBoss"]
        }
        RunBossHistory::M20DefectHeart { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M19DefectAct3 { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M23WatcherAct2 { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M24WatcherAct3 { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M25WatcherHeart { act1: RunAct1BossTarget::Hexaghost, .. } |
        RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M14SilentAct3 { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M15SilentHeart { act1: RunAct1BossTarget::Hexaghost, .. } => {
            vec!["Guardian", "SlimeBoss"]
        }
        RunBossHistory::M20DefectHeart { act1: RunAct1BossTarget::SlimeBoss, .. } |
        RunBossHistory::M19DefectAct3 { act1: RunAct1BossTarget::SlimeBoss, .. } |
        RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget::SlimeBoss, .. } |
        RunBossHistory::M23WatcherAct2 { act1: RunAct1BossTarget::SlimeBoss, .. } |
        RunBossHistory::M24WatcherAct3 { act1: RunAct1BossTarget::SlimeBoss, .. } |
        RunBossHistory::M25WatcherHeart { act1: RunAct1BossTarget::SlimeBoss, .. } |
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

    if matches!(profile, RunProfile::M13SilentAct2 | RunProfile::M14SilentAct3 | RunProfile::M15SilentHeart | RunProfile::M18DefectAct2|RunProfile::M19DefectAct3|RunProfile::M20DefectHeart|RunProfile::M23WatcherAct2|RunProfile::M24WatcherAct3|RunProfile::M25WatcherHeart) {
        creation.seen_bosses[1] = match config.boss_history {
            RunBossHistory::M20DefectHeart { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M19DefectAct3 { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M18DefectAct2 { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M23WatcherAct2 { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M24WatcherAct3 { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M25WatcherHeart { act2: RunAct2BossTarget::Champ, .. } |
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M15SilentHeart { act2: RunAct2BossTarget::Champ, .. } => {
                vec!["Automaton", "Collector"]
            }
            RunBossHistory::M20DefectHeart { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M19DefectAct3 { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M18DefectAct2 { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M23WatcherAct2 { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M24WatcherAct3 { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M25WatcherHeart { act2: RunAct2BossTarget::BronzeAutomaton, .. } |
            RunBossHistory::M13SilentAct2 { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M14SilentAct3 { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M15SilentHeart { act2: RunAct2BossTarget::BronzeAutomaton, .. } => {
                vec!["Champ", "Collector"]
            }
            RunBossHistory::M20DefectHeart { act2: RunAct2BossTarget::Collector, .. } |
            RunBossHistory::M19DefectAct3 { act2: RunAct2BossTarget::Collector, .. } |
            RunBossHistory::M18DefectAct2 { act2: RunAct2BossTarget::Collector, .. } |
            RunBossHistory::M23WatcherAct2 { act2: RunAct2BossTarget::Collector, .. } |
            RunBossHistory::M24WatcherAct3 { act2: RunAct2BossTarget::Collector, .. } |
            RunBossHistory::M25WatcherHeart { act2: RunAct2BossTarget::Collector, .. } |
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
    if matches!(profile, RunProfile::M14SilentAct3 | RunProfile::M15SilentHeart | RunProfile::M19DefectAct3 | RunProfile::M20DefectHeart | RunProfile::M24WatcherAct3 | RunProfile::M25WatcherHeart) {
        creation.seen_bosses[2] = match config.boss_history {
            RunBossHistory::M20DefectHeart { act3: RunAct3BossTarget::AwakenedOne, .. } |
            RunBossHistory::M19DefectAct3 { act3: RunAct3BossTarget::AwakenedOne, .. } |
            RunBossHistory::M24WatcherAct3 { act3: RunAct3BossTarget::AwakenedOne, .. } |
            RunBossHistory::M25WatcherHeart { act3: RunAct3BossTarget::AwakenedOne, .. } |
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::AwakenedOne, .. } => {
                vec!["TimeEater", "DonuAndDeca"]
            }
            RunBossHistory::M15SilentHeart { act3: RunAct3BossTarget::AwakenedOne, .. } => {
                vec!["TimeEater", "DonuAndDeca"]
            }
            RunBossHistory::M20DefectHeart { act3: RunAct3BossTarget::TimeEater, .. } |
            RunBossHistory::M19DefectAct3 { act3: RunAct3BossTarget::TimeEater, .. } |
            RunBossHistory::M24WatcherAct3 { act3: RunAct3BossTarget::TimeEater, .. } |
            RunBossHistory::M25WatcherHeart { act3: RunAct3BossTarget::TimeEater, .. } |
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::TimeEater, .. } => {
                vec!["AwakenedOne", "DonuAndDeca"]
            }
            RunBossHistory::M15SilentHeart { act3: RunAct3BossTarget::TimeEater, .. } => {
                vec!["AwakenedOne", "DonuAndDeca"]
            }
            RunBossHistory::M20DefectHeart { act3: RunAct3BossTarget::DonuAndDeca, .. } |
            RunBossHistory::M19DefectAct3 { act3: RunAct3BossTarget::DonuAndDeca, .. } |
            RunBossHistory::M24WatcherAct3 { act3: RunAct3BossTarget::DonuAndDeca, .. } |
            RunBossHistory::M25WatcherHeart { act3: RunAct3BossTarget::DonuAndDeca, .. } |
            RunBossHistory::M14SilentAct3 { act3: RunAct3BossTarget::DonuAndDeca, .. } => {
                vec!["AwakenedOne", "TimeEater"]
            }
            RunBossHistory::M15SilentHeart { act3: RunAct3BossTarget::DonuAndDeca, .. } => {
                vec!["AwakenedOne", "TimeEater"]
            }
            _ => unreachable!("M14 validation requires a triple boss history"),
        };
    } else if profile != RunProfile::M12SilentAct1 && profile != RunProfile::M13SilentAct2 && profile != RunProfile::M17DefectAct1 && profile != RunProfile::M18DefectAct2 && profile != RunProfile::M22WatcherAct1 && profile != RunProfile::M23WatcherAct2 {
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
        RunProfile::M19DefectAct3 => RoomScenario::M19DefectAct3,
        RunProfile::M20DefectHeart => RoomScenario::M20DefectHeart,
            RunProfile::M22WatcherAct1 => RoomScenario::M22WatcherAct1,
            RunProfile::M23WatcherAct2 => RoomScenario::M23WatcherAct2,
        RunProfile::M24WatcherAct3 => RoomScenario::M24WatcherAct3,
        RunProfile::M25WatcherHeart => RoomScenario::M25WatcherHeart,
        RunProfile::Standard => unreachable!("Standard created above"),
    };
    #[cfg(test)] if profile==RunProfile::M24WatcherAct3{M24_INITIALIZATIONS.with(|v|v.borrow_mut().push((ordinal,creation.seen_bosses.clone())));}
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
    if matches!(profile, RunProfile::M12SilentAct1 | RunProfile::M17DefectAct1 | RunProfile::M22WatcherAct1)
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M12 Act2Ready must be nonterminal with no actions",
        ));
    }
    if matches!(profile,RunProfile::M13SilentAct2|RunProfile::M18DefectAct2|RunProfile::M23WatcherAct2)
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M13 Act3Ready must be nonterminal with no actions",
        ));
    }
    if profile == RunProfile::M19DefectAct3
        && decision.room.as_ref().is_some_and(|room|room.kind==RoomViewKind::Act4Ready) {
        return crate::run_schema_m19::validate_public_decision(decision);
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
    if profile == RunProfile::M24WatcherAct3
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act4Ready)
    {
        if decision.terminal.is_none() && decision.candidates.is_empty() {
            return Ok(());
        }
        return Err(RunSessionError::UnsupportedBoundary(
            "M24 Act4Ready must be nonterminal with no actions",
        ));
    }
    if decision.terminal.is_none() && decision.candidates.is_empty() {
        return Err(RunSessionError::UnsupportedBoundary(
            "nonterminal Run has no legal actions",
        ));
    }
    Ok(())
}

type RunEncoder=fn(&PublicDecision,Option<&spire_core::run::map::PublicMap>,&RunCapacities)->Result<RunObservation,RunSessionError>;
fn publish(
    profile: RunProfile,
    run: &Run,
    config: &RunSessionConfig,
    token: RunDecisionToken,
    encoder:Option<RunEncoder>,
) -> Result<(RunBoundary, Vec<GameAction>, bool, bool, bool), RunSessionError> {
    let decision = run.decision().map_err(RunSessionError::Core)?;
    check_boundary(profile, &decision)?;
    let map = run.public_map();
    let observation = if let Some(encode)=encoder {encode(&decision,map.as_ref(),&config.capacities)?} else { match profile {
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
        RunProfile::M19DefectAct3 => crate::encode_run_m19(&decision, map.as_ref(), &config.capacities)?,
        RunProfile::M20DefectHeart => crate::encode_run_m20(&decision, map.as_ref(), &config.capacities)?,
            RunProfile::M22WatcherAct1 => crate::encode_run_m22(&decision, map.as_ref(), &config.capacities)?,
            RunProfile::M23WatcherAct2 => crate::encode_run_m23(&decision, map.as_ref(), &config.capacities)?,
            RunProfile::M24WatcherAct3 => crate::encode_run_m24(&decision, map.as_ref(), &config.capacities)?,
        RunProfile::M25WatcherHeart => crate::encode_run_m25(&decision, map.as_ref(), &config.capacities)?,
        RunProfile::Standard => crate::run_observation::encode_run_standard(&decision, map.as_ref(), &config.capacities,
            crate::run_observation::StandardContext{ascension:run.ascension(),run_selection:run.room_selection_operation()})?,
    }
    };
    let at_m12_cutoff = matches!(profile,RunProfile::M12SilentAct1|RunProfile::M17DefectAct1|RunProfile::M22WatcherAct1)
        && decision.terminal.is_none()
        && decision.candidates.is_empty()
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act2Ready);
    let at_m13_cutoff = matches!(profile,RunProfile::M13SilentAct2|RunProfile::M18DefectAct2|RunProfile::M23WatcherAct2)
        && decision.terminal.is_none()
        && decision.candidates.is_empty()
        && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3Ready);
    let at_m14_cutoff = matches!(profile,RunProfile::M14SilentAct3|RunProfile::M19DefectAct3|RunProfile::M24WatcherAct3)
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
pub(crate) const REWARD_ITEMS_FAULT:&str="reward items do not sum to the transition reward";
#[cfg(test)]
thread_local!{
    /// TEST-ONLY: drop the first reward item to exercise the sum guard in sessions and replay.
    pub(crate) static TEST_REWARD_SKEW:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};
}
/// The itemised credits of one committed transition, or `None` if they do not sum to its reward (one check for
/// sessions and replay).
pub(crate) fn checked_reward_items(before:&RewardLedgerSnapshot,transition:&RunTransition)->Option<Vec<RewardItem>>{
    #[allow(unused_mut)]
    let mut items=itemise_rewards(before,&transition.committed_events).ok()?;
    #[cfg(test)] if TEST_REWARD_SKEW.with(|s|s.get())&&!items.is_empty(){items.remove(0);}
    (items.iter().try_fold(0i32,|n,item|n.checked_add(item.amount))==Some(transition.reward)).then_some(items)
}
fn new_record(profile:RunProfile,config:&RunSessionConfig,seed:u64,ordinal:u64)->Result<RunEpisodeRecord,RunSessionError>{
    Ok(RunEpisodeRecord{schema_id:profile.schema_id().into(),profile_id:profile.id().into(),rules_id:profile.rules_id().into(),content_id:profile.content_id().into(),seed_adapter_id:RUN_SEED_ADAPTER_ID.into(),rng_id:spire_core::combat::rng::RNG_CONTRACT_ID.into(),reward_id:RUN_REWARD_ID.into(),observation_fingerprint:profile.observation_spec(&config.capacities)?.fingerprint(),config:config.clone(),seed,ordinal,steps:vec![],terminal:None,fault:None,closed:false})
}
impl RunSession{
    #[cfg(feature="fuzz-diagnostics")]
    pub fn fuzz_run(&self)->Option<&Run>{self.run.as_ref()}
    /// Additive public provenance: no replay, seed, cursor or private Note.
    pub fn environment_spec(&self)->Result<serde_json::Value,RunSessionError>{
        let spec=self.observation_spec()?;
        let mut descriptor=serde_json::json!({"profile_id":self.profile.id(),"schema_id":spec.descriptor()["schema_id"],
            "observation_fingerprint":crate::combat_policy::hex(&spec.fingerprint()),"rules_id":self.profile.rules_id(),
            "content_id":self.profile.content_id(),"reward_id":RUN_REWARD_ID,"seed_adapter_id":RUN_SEED_ADAPTER_ID,
            "rng_id":spire_core::combat::rng::RNG_CONTRACT_ID,"combat_view_id":if matches!(self.profile,RunProfile::M12SilentAct1|RunProfile::M13SilentAct2|RunProfile::M14SilentAct3|RunProfile::M15SilentHeart|RunProfile::M17DefectAct1|RunProfile::M18DefectAct2|RunProfile::M19DefectAct3|RunProfile::M20DefectHeart|RunProfile::M22WatcherAct1|RunProfile::M23WatcherAct2|RunProfile::M24WatcherAct3|RunProfile::M25WatcherHeart){serde_json::Value::Null}else{serde_json::json!(crate::COMBAT_POLICY_VIEW_ID)},
            "mode":"normal_run","partition":self.config.partition.as_str(),"autoreset":"disabled"});
        if self.profile==RunProfile::M22WatcherAct1{descriptor["finite_content"]=crate::run_schema_m22::finite_content();descriptor["cutoff"]=serde_json::json!("Act2Ready: healthy nonterminal zero actions; no completion bonus");}
        if self.profile==RunProfile::M24WatcherAct3{descriptor["finite_content"]=crate::run_schema_m24::finite_content();descriptor["cutoff"]=serde_json::json!({"normal_completion":"Act3NoKeys after actual Act3 boss Victory and Proceed/dialog without all keys; no Heart reward","act4_ready":"healthy nonterminal zero actions at act3 floor50 with all keys; M24 stops before the Act4 door; no completion bonus"});}
        if self.profile==RunProfile::M23WatcherAct2{descriptor["finite_content"]=crate::run_schema_m23::finite_content();descriptor["cutoff"]=serde_json::json!("Act3Ready: healthy nonterminal zero actions after actual Act2 boss settlement; no completion bonus");}
        if self.profile==RunProfile::M20DefectHeart{
            descriptor["finite_content"]=crate::run_schema_m20::finite_content();
            descriptor["ending"]=serde_json::json!("HeartVictory after the actual Act4 Proceed action");
        }
        if self.profile==RunProfile::M25WatcherHeart{
            descriptor["ending"]=serde_json::json!("HeartVictory after the actual Act4 Proceed action");
        }
        if self.profile==RunProfile::M19DefectAct3{
            descriptor["finite_content"]=crate::run_schema_m19::finite_content();
            descriptor["cutoff"]=serde_json::json!("Act4Ready: nonterminal, zero actions, no completion reward; supplied diagnostic only");
        }
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
        if self.profile==RunProfile::Standard{
            descriptor["combat_view_id"]=serde_json::json!(crate::STANDARD_COMBAT_VIEW_ID);
            descriptor["character"]=serde_json::json!(self.config.character.as_str());
            descriptor["ascension"]=serde_json::json!(self.config.ascension);
            // Standard creation is caller configuration and player knowledge (design §4.2), unlike captured profiles' fixed Note.
            descriptor["creation"]=self.standard_creation().map(|c|c.to_json()).unwrap_or(serde_json::Value::Null);
            descriptor["capacity_preset"]=serde_json::json!(if self.config.capacities==RunCapacities::standard_default(){crate::STANDARD_CAPACITY_PRESET_ID}else{"custom"});
            descriptor["vocabulary_manifest"]=serde_json::json!(crate::STANDARD_VOCAB_MANIFEST_ID);
            descriptor["ending"]=serde_json::json!("HeartVictory after the actual Act4 Proceed action; truncated is always false");
        }
        Ok(descriptor)
    }
    pub fn combat_observation_spec(&self)->Result<crate::RunObservationSpec,RunSessionError>{
        if self.profile==RunProfile::Standard{return crate::standard_combat_observation_spec(&self.config.capacities)}
        if self.profile==RunProfile::M22WatcherAct1{return Err(RunSessionError::UnsupportedBoundary("M22 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M24WatcherAct3{return Err(RunSessionError::UnsupportedBoundary("M24 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M25WatcherHeart{return Err(RunSessionError::UnsupportedBoundary("M25 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M23WatcherAct2{return Err(RunSessionError::UnsupportedBoundary("M23 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M20DefectHeart{return Err(RunSessionError::UnsupportedBoundary("M20 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M19DefectAct3{return Err(RunSessionError::UnsupportedBoundary("M19 does not provide the Ironclad combat policy sidecar"))}
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
        if self.profile==RunProfile::Standard{
            // M34s-a design §4.2: the Standard combat view of the current decision, dispatched with the Run's token.
            self.usable()?;let token=self.published.as_ref().ok_or(RunSessionError::NotReset)?.token;
            let run=self.run.as_ref().unwrap();let d=run.decision().map_err(RunSessionError::Core)?;let cp=run.combat_checkpoint();
            let observation=match crate::combat_policy::encode_standard_combat(&d,&self.config.capacities,run.ascension(),run.room_selection_operation(),run.combat_identity()){
                Ok(o)=>o,Err(e)=>return Err(self.freeze(e.clone(),e,None,false,false)),
            };
            return Ok(crate::CombatPolicyBoundary{observation,token,info:crate::combat_policy::standard_lifecycle(&d,cp.as_ref())})
        }
        if self.profile==RunProfile::M22WatcherAct1{return Err(RunSessionError::UnsupportedBoundary("M22 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M24WatcherAct3{return Err(RunSessionError::UnsupportedBoundary("M24 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M25WatcherHeart{return Err(RunSessionError::UnsupportedBoundary("M25 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M23WatcherAct2{return Err(RunSessionError::UnsupportedBoundary("M23 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M20DefectHeart{return Err(RunSessionError::UnsupportedBoundary("M20 does not provide the Ironclad combat policy sidecar"))}
        if self.profile==RunProfile::M19DefectAct3{return Err(RunSessionError::UnsupportedBoundary("M19 does not provide the Ironclad combat policy sidecar"))}
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
    /// Opt into the finite Defect A0 Acts 1–3 profile on the shared native owner.
    pub fn new_m22(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M22WatcherAct1)}
    /// Opt into the finite Watcher A0 Acts 1–2 profile on the shared native owner.
    pub fn new_m23(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M23WatcherAct2)}
    /// Opt into the finite Watcher A0 Acts 1–3 profile on the shared native owner.
    pub fn new_m24(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M24WatcherAct3)}
    pub fn new_m25(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M25WatcherHeart)}
    /// M34b: a Standard session; invalid input is rejected before any Run is built.
    pub fn new_standard(config:StandardRunConfig)->Result<Self,RunSessionError>{Self::with_profile(config.session_config(),RunProfile::Standard)}
    /// The Standard creation profile, if this is a Standard session.
    pub fn standard_creation(&self)->Option<StandardCreation>{match self.config.boss_history{RunBossHistory::Standard(c) if self.profile==RunProfile::Standard=>Some(c),_=>None}}
    pub fn new_m20(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M20DefectHeart)}
    pub fn new_m19(config:RunSessionConfig)->Result<Self,RunSessionError>{Self::with_profile(config,RunProfile::M19DefectAct3)}
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
    /// Fixed constructed/supplied unit installation; absent from normal builds and config/JSON.
    #[cfg(test)]
    pub(crate) fn from_m22_diagnostic(config:RunSessionConfig,fixture:spire_core::run::diagnostics::M22Diagnostic)->Result<Self,RunSessionError>{
        let mut s=Self::new_m22(config)?;
        let run=spire_core::run::diagnostics::m22_diagnostic(fixture).map_err(RunSessionError::Core)?;
        s.m22_diagnostic=Some(fixture);s.run=Some(run);s.seed=Some(7);s.serial=1;
        s.record=if s.config.record_replay{let mut r=new_record(s.profile,&s.config,7,0)?;r.observation_fingerprint=s.observation_spec()?.fingerprint();Some(r)}else{None};
        match s.publish_current(1){Ok((boundary,actions,a,b,c))=>{s.published=Some(boundary);s.actions=actions;s.at_m12_cutoff=a;s.at_m13_cutoff=b;s.at_m14_cutoff=c;},Err(error)=>{s.freeze(error.clone(),error,None,false,false);}}
        Ok(s)
    }
    /// TEST-ONLY owner inconsistency for the shared batch panic path: drops the Run owner while the published
    /// boundary remains, so the next accepted step job panics inside RunBatch's catch_unwind. Not a production hook.
    #[cfg(test)]
    pub(crate) fn m23_test_detach_run_owner(&mut self){self.run=None;}
    /// TEST-ONLY: the next reset panics at the given stage (consumed once).
    #[cfg(test)]
    pub(crate) fn set_test_reset_panic(&mut self,stage:TestResetPanic){self.test_reset_panic=Some(stage);}
    /// Fixed constructed M23 owner stage; absent from normal builds and config/JSON.
    #[cfg(test)]
    pub(crate) fn from_m23_diagnostic(config:RunSessionConfig,fixture:spire_core::run::diagnostics::M23Diagnostic)->Result<Self,RunSessionError>{
        let mut s=Self::new_m23(config)?;
        let run=spire_core::run::diagnostics::m23_diagnostic(fixture).map_err(RunSessionError::Core)?;
        s.m23_diagnostic=Some(fixture);s.run=Some(run);s.seed=Some(7);s.serial=1;
        s.record=if s.config.record_replay{let mut r=new_record(s.profile,&s.config,7,0)?;r.observation_fingerprint=s.observation_spec()?.fingerprint();Some(r)}else{None};
        match s.publish_current(1){Ok((boundary,actions,a,b,c))=>{s.published=Some(boundary);s.actions=actions;s.at_m12_cutoff=a;s.at_m13_cutoff=b;s.at_m14_cutoff=c;},Err(error)=>{s.freeze(error.clone(),error,None,false,false);}}
        Ok(s)
    }
    /// Fixed constructed/supplied M24 owner stage; absent from normal builds and config/JSON.
    #[cfg(test)]
    pub(crate) fn from_m24_diagnostic(config:RunSessionConfig,start:M24DiagnosticStart)->Result<Self,RunSessionError>{
        let mut s=Self::new_m24(config)?;
        let run=start.run().map_err(RunSessionError::Core)?;
        s.m24_diagnostic=Some(start);s.run=Some(run);s.seed=Some(start.seed());s.serial=1;
        s.record=if s.config.record_replay{let mut r=new_record(s.profile,&s.config,start.seed(),0)?;r.observation_fingerprint=s.observation_spec()?.fingerprint();Some(r)}else{None};
        match s.publish_current(1){Ok((boundary,actions,a,b,c))=>{s.published=Some(boundary);s.actions=actions;s.at_m12_cutoff=a;s.at_m13_cutoff=b;s.at_m14_cutoff=c;},Err(error)=>{s.freeze(error.clone(),error,None,false,false);}}
        Ok(s)
    }
    /// TEST-ONLY supplied intervention between steps (e.g. `diagnostics::supplied_wound_enemies`): applied to the live
    /// Run, then the boundary is republished under a new serial (earlier tokens are stale). Replay applies the same
    /// intervention at the same accepted-step count through `run_replay::replay_standard_supplied_with`.
    #[cfg(test)]
    pub(crate) fn apply_supplied(&mut self,f:impl FnOnce(&mut Run)->Result<(),RunError>)->Result<RunBoundary,RunSessionError>{
        if self.closed||self.fault.is_some(){return Err(RunSessionError::Closed)}
        f(self.run.as_mut().ok_or(RunSessionError::NotReset)?).map_err(RunSessionError::Core)?;
        let serial=self.serial.checked_add(1).ok_or(RunSessionError::DeliveryFault("Run publication serial exhausted"))?;
        let (boundary,actions,a,b,c)=self.publish_current(serial)?;
        self.serial=serial;self.actions=actions;self.at_m12_cutoff=a;self.at_m13_cutoff=b;self.at_m14_cutoff=c;self.published=Some(boundary.clone());
        boundary.owned()
    }
    /// TEST-ONLY supplied Standard start (e.g. `diagnostics::m29::standard_supplied_run_at`): the session publishes the
    /// supplied Run as its first boundary at `seed`/ordinal 0; replay it with the same supplied Run.
    #[cfg(test)]
    pub(crate) fn from_standard_supplied(config:StandardRunConfig,run:Run,seed:u64)->Result<Self,RunSessionError>{
        let mut s=Self::new_standard(config)?;
        s.run=Some(run);s.seed=Some(seed);s.serial=1;
        s.record=if s.config.record_replay{Some(new_record(s.profile,&s.config,seed,0)?)}else{None};
        match s.publish_current(1){Ok((boundary,actions,a,b,c))=>{s.published=Some(boundary);s.actions=actions;s.at_m12_cutoff=a;s.at_m13_cutoff=b;s.at_m14_cutoff=c;},Err(error)=>{s.freeze(error.clone(),error,None,false,false);}}
        Ok(s)
    }
    /// Fixed constructed M25 owner stage; absent from normal builds and config/JSON.
    #[cfg(test)]
    pub(crate) fn from_m25_diagnostic(config:RunSessionConfig,fixture:spire_core::run::diagnostics::M25Diagnostic)->Result<Self,RunSessionError>{
        let mut s=Self::new_m25(config)?;
        let run=spire_core::run::diagnostics::m25_diagnostic(fixture).map_err(RunSessionError::Core)?;
        s.m25_diagnostic=Some(fixture);s.run=Some(run);s.seed=Some(7);s.serial=1;
        s.record=if s.config.record_replay{let mut r=new_record(s.profile,&s.config,7,0)?;r.observation_fingerprint=s.observation_spec()?.fingerprint();Some(r)}else{None};
        match s.publish_current(1){Ok((boundary,actions,a,b,c))=>{s.published=Some(boundary);s.actions=actions;s.at_m12_cutoff=a;s.at_m13_cutoff=b;s.at_m14_cutoff=c;},Err(error)=>{s.freeze(error.clone(),error,None,false,false);}}
        Ok(s)
    }
    /// Fixed supplied M25 starts; played-fixture bootstrap is applied to the owned Run
    /// before session recording begins, so it remains outside the recorded policy count.
    #[cfg(test)]
    pub(crate) fn from_m25_supplied(config:RunSessionConfig,start:M25SuppliedStart)->Result<Self,RunSessionError>{
        let mut s=Self::new_m25(config)?;
        let mut run=start.run().map_err(RunSessionError::Core)?;
        if let Some(action)=start.bootstrap_action(&run).map_err(RunSessionError::Core)?{run.step(&action).map_err(RunSessionError::Core)?;}
        s.run=Some(run);s.seed=Some(7);s.serial=1;
        s.record=if s.config.record_replay{Some(new_record(s.profile,&s.config,7,0)?)}else{None};
        let (boundary,actions,a,b,c)=s.publish_current(1)?;
        s.published=Some(boundary);s.actions=actions;s.at_m12_cutoff=a;s.at_m13_cutoff=b;s.at_m14_cutoff=c;
        Ok(s)
    }
    #[cfg(test)]
    pub(crate) fn from_m20_diagnostic(config:RunSessionConfig,loadout:spire_core::run::diagnostics::M20DiagnosticLoadout)->Result<Self,RunSessionError>{
        validate_m20_diagnostic_config(&config)?;
        let mut session=Self::new_m20(config)?;
        let run=spire_core::run::diagnostics::m20_supplied_act4(loadout).map_err(RunSessionError::Core)?;
        let mut record=if session.config.record_replay{Some(new_record(session.profile,&session.config,7,0)?)}else{None};
        if let Some(r)=&mut record{r.observation_fingerprint=crate::run_schema_m20::diagnostic_spec(&session.config.capacities)?.fingerprint();}
        session.m20_diagnostic=Some(loadout);session.run=Some(run);session.seed=Some(7);session.serial=1;session.record=record;
        let (boundary,actions,a,b,c)=session.publish_current(1)?;
        session.published=Some(boundary);session.actions=actions;session.at_m12_cutoff=a;session.at_m13_cutoff=b;session.at_m14_cutoff=c;
        Ok(session)
    }
    fn with_profile(config:RunSessionConfig,profile:RunProfile)->Result<Self,RunSessionError>{
        validate_config(profile,&config)?;
        let id=allocate_run_session_id()?;
        Ok(Self{#[cfg(test)] m22_diagnostic:None,#[cfg(test)] m23_diagnostic:None,#[cfg(test)] m24_diagnostic:None,#[cfg(test)] m25_diagnostic:None,#[cfg(test)] test_reset_panic:None,#[cfg(test)] m20_diagnostic:None,config,profile,id,serial:0,run:None,published:None,actions:vec![],seed:None,ordinal:0,record:None,fault:None,last_step:None,closed:false,at_m12_cutoff:false,at_m13_cutoff:false,at_m14_cutoff:false})
    }
    /// The immutable numeric action/observation contract selected by this session.
    pub fn observation_spec(&self)->Result<crate::RunObservationSpec,RunSessionError>{
        #[cfg(test)] if let Some(fixture)=self.m22_diagnostic{return crate::run_schema_m22::diagnostic_spec(&self.config.capacities,fixture)}
        #[cfg(test)] if let Some(fixture)=self.m23_diagnostic{return crate::run_schema_m23::diagnostic_spec(&self.config.capacities,fixture)}
        #[cfg(test)] if let Some(start)=self.m24_diagnostic{return crate::run_schema_m24::diagnostic_spec(&self.config.capacities,start)}
        #[cfg(test)] if let Some(fixture)=self.m25_diagnostic{return crate::run_schema_m25::diagnostic_spec(&self.config.capacities,fixture)}
        #[cfg(test)] if self.m20_diagnostic.is_some(){return crate::run_schema_m20::diagnostic_spec(&self.config.capacities)}
        self.profile.observation_spec(&self.config.capacities)
    }
    fn publish_current(&self,serial:u64)->Result<(RunBoundary,Vec<GameAction>,bool,bool,bool),RunSessionError>{
        let encoder:Option<RunEncoder>=None;
        #[cfg(test)] let encoder=if self.m20_diagnostic.is_some(){Some(crate::run_observation::encode_run_m20_diagnostic as RunEncoder)}else{encoder};
        publish(self.profile,self.run.as_ref().unwrap(),&self.config,RunDecisionToken([self.id,serial]),encoder)
    }
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
        self.start_episode(seed,ordinal,serial)
    }
    fn start_episode(&mut self,seed:u64,ordinal:u64,serial:u64)->Result<RunBoundary,RunSessionError>{
        #[cfg(test)] if self.test_reset_panic==Some(TestResetPanic::BeforeMutation){self.test_reset_panic=None;panic!("TEST-ONLY injected reset worker panic before mutation");}
        #[cfg(test)] {self.m20_diagnostic=None;}
        // Healthy reset intentionally abandons the old episode, never a fault.
        #[cfg(test)] {self.m22_diagnostic=None;}
        #[cfg(test)] {self.m23_diagnostic=None;}
        #[cfg(test)] {self.m24_diagnostic=None;}
        let record=if self.config.record_replay{Some(new_record(self.profile,&self.config,seed,ordinal)?)}else{None};
        self.record=record;self.seed=Some(seed);self.ordinal=ordinal;self.last_step=None;self.published=None;self.actions.clear();
        let run=match create_run(self.profile,&self.config,seed,ordinal){Ok(run)=>run,Err(error)=>{self.run=None;return Err(self.freeze(error.clone(),error,None,false,false))}};
        self.run=Some(run);self.serial=serial;
        #[cfg(test)] if self.test_reset_panic==Some(TestResetPanic::AfterStartup){self.test_reset_panic=None;panic!("TEST-ONLY injected reset worker panic after startup");}
        let (boundary,actions,at_m12_cutoff,at_m13_cutoff,at_m14_cutoff)=match self.publish_current(serial){
            Ok(result)=>result,Err(error)=>return Err(self.freeze(error.clone(),error,None,false,false)),
        };
        let returned=match boundary.owned(){Ok(b)=>b,Err(error)=>return Err(self.freeze(error.clone(),error,None,false,false))};
        self.published=Some(boundary);self.actions=actions;self.at_m12_cutoff=at_m12_cutoff;self.at_m13_cutoff=at_m13_cutoff;self.at_m14_cutoff=at_m14_cutoff;Ok(returned)
    }
    pub fn decision(&self)->Result<RunBoundary,RunSessionError>{self.usable()?;self.published.as_ref().ok_or(RunSessionError::NotReset)?.owned()}
    fn step_preflight(&self,token:RunDecisionToken,index:RunActionIndex)->Result<(usize,u64),RunSessionError>{
        self.usable()?;let published=self.published.as_ref().ok_or(RunSessionError::NotReset)?;
        if token!=published.token{return Err(RunSessionError::StaleToken)}
        if self.at_m12_cutoff{return Err(RunSessionError::UnsupportedBoundary(if self.profile==RunProfile::M22WatcherAct1{"M22 stops at Act2Ready"}else if self.profile==RunProfile::M17DefectAct1{"M17 stops at Act2Ready"}else{"M12 stops at Act2Ready"}))}
        if self.at_m13_cutoff{return Err(RunSessionError::UnsupportedBoundary(if self.profile==RunProfile::M23WatcherAct2{"M23 stops at Act3Ready"}else if self.profile==RunProfile::M18DefectAct2{"M18 stops at Act3Ready"}else{"M13 stops at Act3Ready"}))}
        if self.at_m14_cutoff{return Err(RunSessionError::UnsupportedBoundary(if self.profile==RunProfile::M24WatcherAct3{"M24 stops before the Act4 door"}else if self.profile==RunProfile::M19DefectAct3{"M19 stops before the Act4 door"}else{"M14 stops before the Act4 door"}))}
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
        let ledger=self.run.as_ref().unwrap().reward_snapshot();
        let transition=match self.run.as_mut().unwrap().step(&action){
            Ok(t)=>t,
            Err(error)=>{
                let committed=matches!(error,RunError::Faulted(_));let error=RunSessionError::Core(error);
                let error=self.freeze(error.clone(),error,Some(action),committed,false);
                return Err(RunSessionStepFailure{error,committed});
            },
        };
        let reward_events=checked_reward_items(&ledger,&transition);
        self.serial=serial;let step=RunRecordedStep{action:action.clone(),reward:transition.reward,terminal:transition.terminal};self.last_step=Some(step.clone());
        if let Some(record)=&mut self.record{record.steps.push(step);record.terminal=transition.terminal;}
        // A committed step whose items do not sum to its reward is recorded, then frozen (replay reproduces it; review m1).
        let Some(reward_events)=reward_events else{
            let error=RunSessionError::DeliveryFault(REWARD_ITEMS_FAULT);
            let error=self.freeze(error.clone(),error,Some(action),true,true);return Err(RunSessionStepFailure{error,committed:true})
        };
        let (boundary,actions,at_m12_cutoff,at_m13_cutoff,at_m14_cutoff)=match self.publish_current(serial){
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
        Ok(RunStepResult{boundary:returned,reward:transition.reward,terminated:transition.terminal.is_some(),truncated:false,reward_events})
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
        let mut owned=new_record(self.profile,&record.config,record.seed,record.ordinal)?;owned.observation_fingerprint=record.observation_fingerprint;owned.steps=steps;owned.terminal=record.terminal;owned.fault=record.fault.clone();owned.closed=record.closed;Ok(Some(owned))
    }
    pub fn reproduction(&self)->Result<RunReproduction,RunSessionError>{
        Ok(RunReproduction{config:self.config.clone(),seed:self.seed,ordinal:self.ordinal,latest_decision:self.run.as_ref().map(Run::decision),latest_map:self.run.as_ref().and_then(Run::public_map),last_accepted_step:self.last_step.clone(),fault:self.fault.clone(),record:self.record()?,closed:self.closed})
    }
    pub fn close(&mut self)->Result<(),RunSessionError>{self.closed=true;if let Some(record)=&mut self.record{record.closed=true;}Ok(())}
    /// M34c: exact copy of this session's current episode. Allowed on any reset, unfrozen, unclosed session (terminal
    /// included); reading it changes nothing (no draw, no serial). Privileged: see [`ExactCheckpoint`].
    pub fn exact_checkpoint(&self)->Result<ExactCheckpoint,RunSessionError>{
        self.usable()?;if self.published.is_none(){return Err(RunSessionError::NotReset)}
        Ok(ExactCheckpoint{state:Box::new(self.exact_copy(self.id,self.serial)?)})
    }
    /// M34c: abandon the current episode and continue from `checkpoint`, which must come from a session with the same
    /// profile and configuration (capacities, creation, recording). Seed, ordinal (so `reset(None)` continues at its
    /// ordinal + 1), record prefix and last step come from the checkpoint; the boundary is republished under a new serial,
    /// so every earlier token is stale. A frozen or closed session cannot restore; a rejected restore changes nothing.
    pub fn exact_restore(&mut self,checkpoint:&ExactCheckpoint)->Result<RunBoundary,RunSessionError>{
        self.usable()?;let source=&checkpoint.state;
        if source.profile!=self.profile||source.config!=self.config{return Err(RunSessionError::Input("exact checkpoint belongs to another session configuration"))}
        let serial=self.serial.checked_add(1).ok_or(RunSessionError::Input("Run decision identity exhausted"))?;
        let restored=source.exact_copy(self.id,serial)?;let returned=restored.decision()?;
        *self=restored;Ok(returned)
    }
    /// M34c: a new independent session (new identity, tokens and record) at `checkpoint`. Allowed even after the source
    /// session froze or closed: the fork never inherits a later fault. That is an explicit caller choice, not a retry.
    pub fn exact_fork(checkpoint:&ExactCheckpoint)->Result<Self,RunSessionError>{
        let source=&checkpoint.state;let id=allocate_run_session_id()?;
        let serial=source.serial.checked_add(1).ok_or(RunSessionError::Input("Run decision identity exhausted"))?;
        source.exact_copy(id,serial)
    }
    /// Deep copy of the episode state under identity `[id, serial]` (the published token is reissued), never faulted or closed.
    fn exact_copy(&self,id:u64,serial:u64)->Result<Self,RunSessionError>{
        let published=match &self.published{Some(b)=>Some(RunBoundary{token:RunDecisionToken([id,serial]),..b.owned()?}),None=>None};
        let mut actions=Vec::new();actions.try_reserve_exact(self.actions.len()).map_err(|_|RunSessionError::DeliveryFault("exact checkpoint allocation failed"))?;actions.extend(self.actions.iter().cloned());
        let record=match &self.record{Some(r)=>{let mut steps=Vec::new();steps.try_reserve_exact(r.steps.len()).map_err(|_|RunSessionError::DeliveryFault("exact checkpoint allocation failed"))?;steps.extend(r.steps.iter().cloned());Some(RunEpisodeRecord{schema_id:r.schema_id.clone(),profile_id:r.profile_id.clone(),rules_id:r.rules_id.clone(),content_id:r.content_id.clone(),seed_adapter_id:r.seed_adapter_id.clone(),rng_id:r.rng_id.clone(),reward_id:r.reward_id.clone(),observation_fingerprint:r.observation_fingerprint,config:r.config.clone(),seed:r.seed,ordinal:r.ordinal,steps,terminal:r.terminal,fault:r.fault.clone(),closed:r.closed})},None=>None};
        Ok(Self{#[cfg(test)] m22_diagnostic:self.m22_diagnostic,#[cfg(test)] m23_diagnostic:self.m23_diagnostic,#[cfg(test)] m24_diagnostic:self.m24_diagnostic,#[cfg(test)] m25_diagnostic:self.m25_diagnostic,#[cfg(test)] test_reset_panic:None,#[cfg(test)] m20_diagnostic:self.m20_diagnostic.clone(),
            config:self.config.clone(),profile:self.profile,id,serial,run:self.run.clone(),published,actions,seed:self.seed,ordinal:self.ordinal,record,
            fault:None,last_step:self.last_step.clone(),closed:false,at_m12_cutoff:self.at_m12_cutoff,at_m13_cutoff:self.at_m13_cutoff,at_m14_cutoff:self.at_m14_cutoff})
    }
    /// M34c replay to a chosen decision (design §4.7): a live session at decision `steps` of `record`, i.e. after its first
    /// `steps` accepted actions. The record is validated as for `replay_run`; the episode starts at the record's seed and
    /// ordinal under its configuration with `capacities`, and each recorded action goes through this session's own `step`
    /// (offered-action, reward/terminal, publication, capacity and fault checks unchanged). A recorded fault inside the
    /// prefix that recurs (a core, capacity or reward-item fault) is reproduced and returned as that error; one that cannot
    /// recur (a transport or batch delivery fault, or a capacity fault under larger capacities) does not stop the prefix,
    /// as in `replay_run`. Any divergence is `IncompatibleReplay`. The new record holds the prefix and continues from it.
    pub fn from_record(record:&RunEpisodeRecord,steps:usize,capacities:RunCapacities)->Result<Self,RunSessionError>{
        let profile=crate::run_replay::validate_record(record)?;
        if steps>record.steps.len(){return Err(RunSessionError::Input("replay-to-decision step is past the end of the record"))}
        let mut session=Self::with_profile(RunSessionConfig{capacities,..record.config.clone()},profile)?;
        session.start_episode(record.seed,record.ordinal,1)?;
        for step in &record.steps[..steps]{
            let index=session.actions.iter().position(|a|a==&step.action).ok_or(RunSessionError::IncompatibleReplay("Run action not currently offered"))?;
            let token=session.published.as_ref().ok_or(RunSessionError::NotReset)?.token;
            let result=session.step(token,RunActionIndex(index as i64))?;
            if (result.reward,result.boundary.terminal)!=(step.reward,step.terminal){return Err(RunSessionError::IncompatibleReplay("Run reward or terminal mismatch"))}
        }
        Ok(session)
    }
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
            "270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c"
        );
        let m10 = RunSession::new_m10(ironclad).unwrap();
        assert_eq!(m10.environment_spec().unwrap()["profile_id"], "M10IroncladHeart");
        assert_eq!(
            m10.observation_spec().unwrap().fingerprint().iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21"
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

#[cfg(test)]
mod m19_boundary_characterization {
    use super::*;
    #[test]
    fn m19_constructed_published_cutoff_preserves_preflight_reset_and_close() {
        // PRIVATE PROJECTION ONLY: the owned Run remains a canonical Neow Run.
        // This is not an actual native late Run or all-key ending receipt.
        // Actual supplied core ending proof belongs to run/tests/m19_defect_act3.
        let config=RunSessionConfig{character:Character::Defect,ascension:0,partition:Partition::Test,
            boss_history:RunBossHistory::M19DefectAct3{act1:RunAct1BossTarget::Guardian,
                act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},
            capacities:RunCapacities::m10_default(),record_replay:true};
        let mut session=RunSession::new_m19(config.clone()).unwrap();
        let stale=session.reset(Some(7)).unwrap().token;let start=session.reset(None).unwrap();
        let mut projected=session.run.as_ref().unwrap().decision().unwrap();
        projected.act=3;projected.floor=50;projected.player.keys=[true;3];projected.candidates.clear();
        let room=projected.room.as_mut().unwrap();room.kind=RoomViewKind::Act4Ready;room.choices.clear();
        assert_eq!(check_boundary(RunProfile::M19DefectAct3,&projected),Ok(()));
        let observation=crate::encode_run_m19(&projected,None,&config.capacities).unwrap();
        assert!(observation.action_mask.iter().all(|v|*v==0));
        session.published=Some(RunBoundary{observation,token:start.token,terminal:None});
        session.actions.clear();session.at_m14_cutoff=true;
        let boundary=session.decision().unwrap();let before=session.reproduction().unwrap();
        assert_eq!(session.step(stale,RunActionIndex(0)),Err(RunSessionError::StaleToken));
        let expected=RunSessionError::UnsupportedBoundary("M19 stops before the Act4 door");
        assert_eq!(session.preflight_step(start.token,RunActionIndex(i64::MAX)),Err(expected.clone()));
        assert_eq!(session.step(start.token,RunActionIndex(0)),Err(expected));
        assert_eq!(session.decision().unwrap(),boundary);assert_eq!(session.reproduction().unwrap(),before);
        assert!(session.fault.is_none());assert!(session.record().unwrap().unwrap().steps.is_empty());
        let reset=session.reset(None).unwrap();assert_ne!(reset.token,start.token);
        assert!(!session.at_m14_cutoff);assert_eq!(session.ordinal,2);
        let actual=session.run.as_ref().unwrap().decision().unwrap();assert_eq!((actual.act,actual.floor),(1,0));
        assert_eq!(actual.room.unwrap().kind,RoomViewKind::Neow);assert!(session.run.as_ref().unwrap().public_map().is_none());
        assert_eq!(session.step(start.token,RunActionIndex(0)),Err(RunSessionError::StaleToken));
        session.close().unwrap();assert_eq!(session.decision(),Err(RunSessionError::Closed));
    }
}

#[cfg(test)]
pub(crate) fn validate_m20_diagnostic_config(config:&RunSessionConfig)->Result<(),RunSessionError>{
    validate_config(RunProfile::M20DefectHeart,config)?;
    if config.partition!=Partition::Test || config.boss_history!=(RunBossHistory::M20DefectHeart{act1:crate::RunAct1BossTarget::Guardian,act2:crate::RunAct2BossTarget::Champ,act3:crate::RunAct3BossTarget::AwakenedOne}){
        return Err(RunSessionError::Input("M20 fixed diagnostic requires seed7/test and first frozen history triple"))
    }Ok(())
}

#[cfg(test)]
pub(crate) fn m20_test_config()->RunSessionConfig{
    RunSessionConfig{character:Character::Defect,ascension:0,partition:Partition::Test,
        boss_history:RunBossHistory::M20DefectHeart{act1:crate::RunAct1BossTarget::Guardian,act2:crate::RunAct2BossTarget::Champ,act3:crate::RunAct3BossTarget::AwakenedOne},
        capacities:RunCapacities::m10_default(),record_replay:true}
}
#[cfg(test)]
mod m20_diagnostic_installation {
    use super::*;
    use spire_core::{content::CardId,run::diagnostics::M20DiagnosticLoadout};
    #[test]
    fn m20_native_diagnostic_installation_and_normal_preservation(){
        for loadout in [M20DiagnosticLoadout::Defect,M20DiagnosticLoadout::Strong]{
            let config=m20_test_config();
            let core=spire_core::run::diagnostics::m20_supplied_act4(loadout).unwrap();
            let mut session=RunSession::from_m20_diagnostic(config.clone(),loadout).unwrap();
            let d=core.decision().unwrap();let b=session.decision().unwrap();
            assert_eq!(session.reproduction().unwrap().latest_decision,Some(Ok(d.clone())));
            assert_eq!((d.act,d.floor),(3,50));assert!(d.terminal.is_none());assert!(!d.candidates.is_empty());
            assert_eq!(b.observation,crate::run_observation::encode_run_m20_diagnostic(&d,core.public_map().as_ref(),&config.capacities).unwrap());
            assert_eq!(session.observation_spec().unwrap().descriptor()["schema_id"],crate::run_schema_m20::DIAGNOSTIC_SCHEMA_ID);
            assert_eq!(d.objects.len(),if loadout==M20DiagnosticLoadout::Strong{10}else{18});
            if loadout==M20DiagnosticLoadout::Strong{
                assert!(d.objects.iter().all(|c|c.id==CardId::SearingBlow&&c.upgrades==15));
                assert_eq!((d.player.hp,d.player.max_hp),(250,250));
                assert!(crate::encode_run_m20(&d,None,&config.capacities).is_err());
            }
            // Diagnostic records cannot claim ordinary portable reconstruction.
            let record=session.record().unwrap().unwrap();assert!(record.to_json().is_err());
            assert!(crate::replay_run(&record,config.capacities.clone()).is_err());
            let mut ordinary=RunSession::new_m20(config.clone()).unwrap();let expected=ordinary.reset(Some(7)).unwrap();
            let reset=session.reset(Some(7)).unwrap();assert_ne!(b.token,reset.token);
            assert_eq!(reset.observation,expected.observation);assert_eq!(session.reproduction().unwrap(),ordinary.reproduction().unwrap());
            assert_eq!(session.observation_spec().unwrap(),crate::m20_run_observation_spec(&config.capacities).unwrap());
            assert_eq!(session.step(b.token,crate::RunActionIndex(0)),Err(RunSessionError::StaleToken));
        }
        let mut wrong=m20_test_config();wrong.boss_history=RunBossHistory::M20DefectHeart{act1:crate::RunAct1BossTarget::Hexaghost,act2:crate::RunAct2BossTarget::Champ,act3:crate::RunAct3BossTarget::AwakenedOne};
        assert!(RunSession::from_m20_diagnostic(wrong,M20DiagnosticLoadout::Defect).is_err());
    }
}

/// Held consumers use original actions only. No caller/policy is linked here.
#[cfg(test)]
mod m20_recorded_consumers {
    use super::*;
    use crate::{RunBatch,RunBatchError,RunActionIndex};
    use crate::run_replay::{parse_unique_json,M20DiagnosticEnvelope,m20_initial_public_sha256,replay_m20_diagnostic};
    use serde_json::Value;
    use sha2::{Digest,Sha256};
    use spire_core::run::diagnostics::M20DiagnosticLoadout;
    use std::path::{Path,PathBuf};

    fn text(v:&Value)->&str{v.as_str().expect("released receipt string")}
    fn hash(bytes:&[u8])->[u8;32]{Sha256::digest(bytes).into()}
    fn hex(bytes:[u8;32])->String{bytes.iter().map(|b|format!("{b:02x}")).collect()}
    fn unhex(value:&str)->[u8;32]{
        assert_eq!(value.len(),64,"receipt SHA256 length");
        let mut bytes=[0;32];for (i,b) in bytes.iter_mut().enumerate(){*b=u8::from_str_radix(&value[i*2..i*2+2],16).expect("receipt SHA256 hex");}bytes
    }
    fn keys(v:&Value,expected:&[&str]){
        let o=v.as_object().expect("receipt object");assert_eq!(o.len(),expected.len(),"unexpected receipt fields");
        for key in expected{assert!(o.contains_key(*key),"missing receipt field {key}");}
    }
    struct Case{value:Value,config:RunSessionConfig,seed:u64,loadout:Option<M20DiagnosticLoadout>,freeze:[u8;32]}
    impl Case{
        fn core(&self)->Run{
            match self.loadout{Some(loadout)=>spire_core::run::diagnostics::m20_supplied_act4(loadout).unwrap(),None=>create_run(RunProfile::M20DefectHeart,&self.config,self.seed,0).unwrap()}
        }
        fn scalar(&self)->RunSession{
            match self.loadout{Some(loadout)=>RunSession::from_m20_diagnostic(self.config.clone(),loadout).unwrap(),None=>{let mut s=RunSession::new_m20(self.config.clone()).unwrap();s.reset(Some(self.seed)).unwrap();s}}
        }
        fn batch(&self,workers:usize)->(RunBatch,Vec<RunDecisionToken>){
            let configs=vec![self.config.clone();2];
            if let Some(loadout)=self.loadout{
                let b=RunBatch::from_m20_diagnostic(configs,workers,loadout).unwrap();
                // Tokens are transport-only. Read them from the owned sessions
                // through an owner-local accessor, never from gameplay receipts.
                let tokens=b.m20_test_tokens();(b,tokens)
            }else{
                let mut b=RunBatch::new_m20(configs,workers).unwrap();let bounds=b.reset(&[Some(self.seed);2]).unwrap();(b,bounds.iter().map(|b|b.token).collect())
            }
        }
        fn encode(&self,d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>)->RunObservation{
            if self.loadout.is_some(){crate::run_observation::encode_run_m20_diagnostic(d,map,&self.config.capacities).unwrap()}
            else{crate::encode_run_m20(d,map,&self.config.capacities).unwrap()}
        }
        fn rows(&self)->&[Value]{self.value["steps"].as_array().unwrap()}
        fn failed(&self)->Option<&Value>{(!self.value["failed_action"].is_null()).then_some(&self.value["failed_action"])}
    }
    fn read_case(path:&Path,bytes:&[u8],freeze:[u8;32])->Case{
        let v=parse_unique_json(bytes).expect("duplicate-safe original receipt");
        keys(&v,&["schema_id","caller_revision","seed","scenario","provenance","action_budget","outcome","error","accepted_action_count","steps","failed_action","endpoint"]);
        assert_eq!(v["schema_id"],"m20-defect-heart-caller-attempt-v1");assert_eq!(v["caller_revision"],"m20-public-caller-v1");assert_eq!(v["action_budget"],8192);
        keys(&v["scenario"],&["act1_boss","act2_boss","act3_boss"]);
        let mut config=m20_test_config();
        config.boss_history=RunBossHistory::M20DefectHeart{
            act1:match text(&v["scenario"]["act1_boss"]){"guardian"=>crate::RunAct1BossTarget::Guardian,"hexaghost"=>crate::RunAct1BossTarget::Hexaghost,"slime_boss"=>crate::RunAct1BossTarget::SlimeBoss,_=>panic!("foreign Act1 target")},
            act2:match text(&v["scenario"]["act2_boss"]){"champ"=>crate::RunAct2BossTarget::Champ,"bronze_automaton"=>crate::RunAct2BossTarget::BronzeAutomaton,"collector"=>crate::RunAct2BossTarget::Collector,_=>panic!("foreign Act2 target")},
            act3:match text(&v["scenario"]["act3_boss"]){"awakened_one"=>crate::RunAct3BossTarget::AwakenedOne,"time_eater"=>crate::RunAct3BossTarget::TimeEater,"donu_and_deca"=>crate::RunAct3BossTarget::DonuAndDeca,_=>panic!("foreign Act3 target")},
        };
        let seed=v["seed"].as_u64().expect("fixed seed");assert!(matches!(seed,7|11));
        keys(&v["provenance"],&["kind","loadout","initial_manifest_sha256","initial_public_sha256"]);
        let loadout=match text(&v["provenance"]["kind"]){
            "normal"=>{assert!(v["provenance"]["loadout"].is_null());assert!(v["provenance"]["initial_manifest_sha256"].is_null());None},
            "supplied"=>{assert_eq!(seed,7);validate_m20_diagnostic_config(&config).unwrap();assert_eq!(unhex(text(&v["provenance"]["initial_manifest_sha256"])),freeze);
                Some(match text(&v["provenance"]["loadout"]){"Defect"=>M20DiagnosticLoadout::Defect,"Strong"=>M20DiagnosticLoadout::Strong,_=>panic!("foreign supplied loadout")})},
            _=>panic!("foreign receipt provenance"),
        };
        let rows=v["steps"].as_array().expect("original steps");assert_eq!(v["accepted_action_count"].as_u64().unwrap() as usize,rows.len());assert!(rows.len()<=8192);
        for (i,row) in rows.iter().enumerate(){
            keys(row,&["index","candidate_index","action_debug","pre","transition","post"]);assert_eq!(row["index"].as_u64().unwrap() as usize,i);
            keys(&row["pre"],&["decision_debug","map_debug"]);keys(&row["transition"],&["debug","reward","terminal","committed_events_debug"]);
            keys(&row["post"],&["decision_debug","map_debug","decision_error"]);
        }
        keys(&v["endpoint"],&["decision_debug","map_debug","decision_error"]);
        if !v["failed_action"].is_null(){keys(&v["failed_action"],&["action_debug","candidate_index","pre_decision_debug","pre_map_debug","error"]);}
        let case=Case{value:v,config,seed,loadout,freeze};let core=case.core();
        assert_eq!(unhex(text(&case.value["provenance"]["initial_public_sha256"])),m20_initial_public_sha256(&core).unwrap());
        println!("M20_ORIGINAL_RECEIPT path={} sha256={} bytes={} steps={} provenance={} loadout={:?}",path.display(),hex(hash(bytes)),bytes.len(),case.rows().len(),case.value["provenance"]["kind"],loadout);
        case
    }
    fn read_released()->Vec<Case>{
        let path=PathBuf::from(std::env::var_os("SPIRE_M20_NATIVE_RECEIPT_MANIFEST").expect("HOLD: exact original receipt manifest not released"));
        let expected=unhex(&std::env::var("SPIRE_M20_NATIVE_RECEIPT_MANIFEST_SHA256").expect("HOLD: exact receipt manifest hash not released"));
        let freeze=unhex(&std::env::var("SPIRE_M20_NATIVE_FREEZE_SHA256").expect("HOLD: global source/caller/fixtures/binaries freeze not released"));
        let bytes=std::fs::read(&path).expect("read retained release manifest");assert_eq!(hash(&bytes),expected);
        let v=parse_unique_json(&bytes).expect("duplicate-safe release manifest");keys(&v,&["schema_id","freeze_manifest_sha256","cases","fallback"]);
        assert_eq!(v["schema_id"],"m20-native-receipt-release-v1");assert_eq!(unhex(text(&v["freeze_manifest_sha256"])),freeze);
        let base=path.parent().unwrap();let mut cases=Vec::new();
        for entry in v["cases"].as_array().expect("release cases"){
            keys(entry,&["path","sha256"]);let receipt=base.join(text(&entry["path"]));let bytes=std::fs::read(&receipt).expect("read original receipt");
            assert_eq!(hash(&bytes),unhex(text(&entry["sha256"])));cases.push(read_case(&receipt,&bytes,freeze));
        }
        let normal=cases.iter().filter(|c|c.loadout.is_none()).collect::<Vec<_>>();assert_eq!(normal.len(),6,"all six originals required including losses/faults");
        let expected=[(7,"guardian","champ","awakened_one"),(7,"hexaghost","bronze_automaton","time_eater"),(7,"slime_boss","collector","donu_and_deca"),(11,"guardian","bronze_automaton","donu_and_deca"),(11,"hexaghost","collector","awakened_one"),(11,"slime_boss","champ","time_eater")];
        for (seed,a,b,c) in expected{assert_eq!(normal.iter().filter(|x|x.seed==seed&&x.value["scenario"]["act1_boss"]==a&&x.value["scenario"]["act2_boss"]==b&&x.value["scenario"]["act3_boss"]==c).count(),1,"fixed case missing/duplicated");}
        let first=cases.iter().filter(|c|c.loadout==Some(M20DiagnosticLoadout::Defect)).collect::<Vec<_>>();assert_eq!(first.len(),1);
        let strong=cases.iter().filter(|c|c.loadout==Some(M20DiagnosticLoadout::Strong)).count();
        match text(&v["fallback"]){
            "executed"=>{assert_eq!(first[0].value["outcome"],"Death");assert_eq!(strong,1);},
            "skipped"=>{assert_eq!(first[0].value["outcome"],"HeartVictory");assert_eq!(strong,0);},
            "blocked"=>{assert!(first[0].value["outcome"]!="Death"&&first[0].value["outcome"]!="HeartVictory");assert_eq!(strong,0);},
            _=>panic!("unknown fallback release condition"),
        }
        assert_eq!(cases.len(),7+strong);cases
    }
    // Corrective supplied verification is separate from the original seven-case
    // release. It shares the exact receipt/scalar/replay/batch consumers below.
    fn corrective_manifest(bytes:&[u8],base:&Path,freeze:[u8;32])->Vec<Case>{
        let v=parse_unique_json(bytes).expect("duplicate-safe corrective release");keys(&v,&["schema_id","freeze_manifest_sha256","cases","fallback"]);
        assert_eq!(v["schema_id"],"m20-native-corrective-receipt-release-v1");assert_eq!(unhex(text(&v["freeze_manifest_sha256"])),freeze);
        let mut cases=Vec::new();
        for entry in v["cases"].as_array().expect("corrective cases"){
            keys(entry,&["path","sha256"]);let path=base.join(text(&entry["path"]));let bytes=std::fs::read(&path).unwrap();assert_eq!(hash(&bytes),unhex(text(&entry["sha256"])));
            let receipt=parse_unique_json(&bytes).unwrap();assert_eq!(receipt["provenance"]["kind"],"supplied","corrective release cannot contain normal cases");
            cases.push(read_case(&path,&bytes,freeze));
        }
        let first=cases.iter().filter(|c|c.loadout==Some(M20DiagnosticLoadout::Defect)).collect::<Vec<_>>();assert_eq!(first.len(),1,"one corrective Defect attempt required");
        let strong=cases.iter().filter(|c|c.loadout==Some(M20DiagnosticLoadout::Strong)).count();
        match text(&v["fallback"]){
            "executed"=>{assert_eq!(first[0].value["outcome"],"Death");assert_eq!(strong,1);},
            "skipped"=>{assert_eq!(first[0].value["outcome"],"HeartVictory");assert_eq!(strong,0);},
            "blocked"=>{assert!(first[0].value["outcome"]!="Death"&&first[0].value["outcome"]!="HeartVictory");assert_eq!(strong,0);},
            _=>panic!("unknown corrective fallback condition"),
        }
        assert_eq!(cases.len(),1+strong);cases
    }
    fn read_corrective_released()->Vec<Case>{
        let path=PathBuf::from(std::env::var_os("SPIRE_M20_NATIVE_RECEIPT_MANIFEST").expect("corrective release path"));let bytes=std::fs::read(&path).unwrap();
        assert_eq!(hash(&bytes),unhex(&std::env::var("SPIRE_M20_NATIVE_RECEIPT_MANIFEST_SHA256").expect("corrective release hash")));
        let freeze=unhex(&std::env::var("SPIRE_M20_NATIVE_FREEZE_SHA256").expect("corrective freeze hash"));corrective_manifest(&bytes,path.parent().unwrap(),freeze)
    }
    #[test]
    fn m20_corrective_manifest_admission_keeps_provenance_and_death_only_branch(){
        // Pure receipt admission controls; no actions/policy or claimed outcomes.
        let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();let base=std::env::temp_dir().join(format!("m20-corrective-reader-{}-{stamp}",std::process::id()));std::fs::create_dir(&base).unwrap();let freeze=[9;32];
        let receipt=|loadout|{let run=spire_core::run::diagnostics::m20_supplied_act4(loadout).unwrap();serde_json::json!({"schema_id":"m20-defect-heart-caller-attempt-v1","caller_revision":"m20-public-caller-v1","seed":7,"scenario":{"act1_boss":"guardian","act2_boss":"champ","act3_boss":"awakened_one"},"provenance":{"kind":"supplied","loadout":format!("{loadout:?}"),"initial_manifest_sha256":hex(freeze),"initial_public_sha256":hex(m20_initial_public_sha256(&run).unwrap())},"action_budget":8192,"outcome":null,"error":null,"accepted_action_count":0,"steps":[],"failed_action":null,"endpoint":{"decision_debug":format!("{:?}",run.decision().unwrap()),"map_debug":format!("{:?}",run.public_map()),"decision_error":null}})};
        let write=|name:&str,value:&Value|{let bytes=serde_json::to_vec(value).unwrap();std::fs::write(base.join(name),&bytes).unwrap();serde_json::json!({"path":name,"sha256":hex(hash(&bytes))})};
        let defect=receipt(M20DiagnosticLoadout::Defect);let strong=receipt(M20DiagnosticLoadout::Strong);let d=write("defect.json",&defect);let s=write("strong.json",&strong);
        let manifest=|cases:Vec<Value>,fallback:&str|serde_json::json!({"schema_id":"m20-native-corrective-receipt-release-v1","freeze_manifest_sha256":hex(freeze),"cases":cases,"fallback":fallback});
        let parse=|v:&Value|corrective_manifest(&serde_json::to_vec(v).unwrap(),&base,freeze);
        assert_eq!(parse(&manifest(vec![d.clone()],"blocked")).len(),1);
        let rejects=|v:Value|assert!(std::panic::catch_unwind(||{parse(&v);}).is_err());
        rejects(manifest(vec![],"blocked"));rejects(manifest(vec![d.clone(),d.clone()],"blocked"));rejects(manifest(vec![s.clone()],"blocked"));rejects(manifest(vec![d.clone()],"executed"));rejects(manifest(vec![d.clone(),s.clone()],"blocked"));rejects(manifest(vec![d.clone()],"skipped"));
        let mut death=defect.clone();death["outcome"]=serde_json::json!("Death");let dd=write("death-index-only.json",&death);assert_eq!(parse(&manifest(vec![dd.clone(),s.clone()],"executed")).len(),2);rejects(manifest(vec![dd.clone()],"blocked"));rejects(manifest(vec![dd,s.clone()],"skipped"));
        let mut win=defect.clone();win["outcome"]=serde_json::json!("HeartVictory");let w=write("win-index-only.json",&win);assert_eq!(parse(&manifest(vec![w.clone()],"skipped")).len(),1);rejects(manifest(vec![w,s],"executed"));
        let mut foreign=defect.clone();foreign["provenance"]["kind"]=serde_json::json!("normal");rejects(manifest(vec![write("normal.json",&foreign)],"blocked"));foreign=defect.clone();foreign["seed"]=serde_json::json!(11);rejects(manifest(vec![write("wrong-seed.json",&foreign)],"blocked"));foreign=defect.clone();foreign["provenance"]["loadout"]=serde_json::json!("Other");rejects(manifest(vec![write("wrong-loadout.json",&foreign)],"blocked"));
        let mut wrong=manifest(vec![d.clone()],"blocked");wrong["freeze_manifest_sha256"]=serde_json::json!(hex([8;32]));rejects(wrong);let mut wrong=manifest(vec![d],"blocked");wrong["cases"][0]["sha256"]=serde_json::json!(hex([0;32]));rejects(wrong);
    }
    #[test]
    #[ignore="HOLD: exact corrected source/binary/fixture freeze and corrective supplied receipts required; same recorded actions only"]
    fn m20_corrected_recorded_native_agreement(){for case in read_corrective_released(){consume(&case);}}

    fn compare_pre(case:&Case,run:&Run,session:&RunSession,row:&Value){
        let d=run.decision().unwrap();let map=run.public_map();
        assert_eq!(text(&row["pre"]["decision_debug"]),format!("{d:?}"));assert_eq!(text(&row["pre"]["map_debug"]),format!("{map:?}"));
        let r=session.reproduction().unwrap();assert_eq!(r.latest_decision,Some(Ok(d.clone())));assert_eq!(r.latest_map,map);
        assert_eq!(session.decision().unwrap().observation,case.encode(&d,map.as_ref()));
    }
    fn compare_post(run:&Run,value:&Value){
        match run.decision(){
            Ok(d)=>{assert_eq!(text(&value["decision_debug"]),format!("{d:?}"));assert!(value["decision_error"].is_null());},
            Err(e)=>{assert!(value["decision_debug"].is_null());assert_eq!(text(&value["decision_error"]),format!("{e:?}"));},
        }
        assert_eq!(text(&value["map_debug"]),format!("{:?}",run.public_map()));
    }
    fn consume(case:&Case){
        let mut core=case.core();let mut s=case.scalar();
        let mut batches=[case.batch(1),case.batch(2)];
        for row in case.rows(){
            compare_pre(case,&core,&s,row);let d=core.decision().unwrap();let index=row["candidate_index"].as_u64().unwrap() as usize;
            let action=d.candidates.get(index).expect("recorded candidate is offered").action.clone();assert_eq!(text(&row["action_debug"]),format!("{action:?}"));
            let before=s.reproduction().unwrap();let b=s.decision().unwrap();
            assert!(s.preflight_step(b.token,RunActionIndex(-1)).is_err());assert_eq!(s.reproduction().unwrap(),before);
            let transition=core.step(&action).unwrap();assert_eq!(text(&row["transition"]["debug"]),format!("{transition:?}"));
            assert_eq!(row["transition"]["reward"],transition.reward);assert_eq!(text(&row["transition"]["terminal"]),format!("{:?}",transition.terminal));
            assert_eq!(row["transition"]["committed_events_debug"],serde_json::json!(transition.committed_events.iter().map(|e|format!("{e:?}")).collect::<Vec<_>>()));
            let next=s.step(b.token,RunActionIndex(index as i64)).unwrap();assert_eq!((next.reward,next.boundary.terminal),(transition.reward,transition.terminal));
            assert_eq!(next.terminated,transition.terminal.is_some());assert!(!next.truncated);compare_post(&core,&row["post"]);
            let after=core.decision().unwrap();assert_eq!(s.reproduction().unwrap().latest_decision,Some(Ok(after.clone())));assert_eq!(s.reproduction().unwrap().latest_map,core.public_map());
            assert_eq!(next.boundary.observation,case.encode(&after,core.public_map().as_ref()));
            assert_eq!(s.step(b.token,RunActionIndex(index as i64)),Err(RunSessionError::StaleToken));
            for (batch,tokens) in &mut batches{
                for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),before);}
                let next=batch.step(tokens,&[RunActionIndex(index as i64);2]).unwrap();
                for slot in 0..2{assert_eq!(next[slot].reward,transition.reward);assert_eq!(next[slot].boundary.observation,s.decision().unwrap().observation);assert_eq!(batch.reproduction(slot).unwrap(),s.reproduction().unwrap());assert_eq!(batch.record(slot).unwrap(),s.record().unwrap());}
                *tokens=next.iter().map(|r|r.boundary.token).collect();
            }
        }
        if let Some(failed)=case.failed(){
            let d=core.decision().unwrap();assert_eq!(text(&failed["pre_decision_debug"]),format!("{d:?}"));assert_eq!(text(&failed["pre_map_debug"]),format!("{:?}",core.public_map()));
            if let Some(index)=failed["candidate_index"].as_u64(){
                let action=d.candidates.get(index as usize).unwrap().action.clone();assert_eq!(text(&failed["action_debug"]),format!("{action:?}"));
                let error=core.step(&action).expect_err("recorded failed semantic action must still fail");assert_eq!(text(&failed["error"]),format!("{error:?}"));
                let native=s.step(s.decision().unwrap().token,RunActionIndex(index as i64)).unwrap_err();assert_eq!(native,RunSessionError::Core(error.clone()));
                for (batch,tokens) in &mut batches{
                    let RunBatchError::Fault{slots}=batch.step(tokens,&[RunActionIndex(index as i64);2]).unwrap_err() else{panic!("expected shared core fault")};
                    assert_eq!(slots.len(),2);for slot in slots{assert_eq!(slot.error,RunSessionError::Core(error.clone()));assert!(!slot.delivery_lost);}
                }
            }else{println!("M20_NATIVE_HOLD unoffered/policy failure retained as original prefix only: {}",failed["error"]);}
        }
        compare_post(&core,&case.value["endpoint"]);
        match core.decision(){
            Ok(d)=>if let Some(terminal)=d.terminal{
                assert_eq!(text(&case.value["outcome"]),format!("{terminal:?}"));assert!(case.value["error"].is_null());assert!(case.failed().is_none());
            }else if case.value["outcome"]=="BudgetExhausted"{
                assert_eq!(case.rows().len(),8192);assert!(case.value["error"].is_null());assert!(case.failed().is_none());
            }else{assert!(case.value["outcome"].is_null());assert!(case.value["error"].is_string());},
            Err(_)=>{assert!(case.value["outcome"].is_null());assert!(case.value["error"].is_string());},
        }

        for (batch,_) in &batches{for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),s.reproduction().unwrap());}}
        let record=s.record().unwrap().unwrap();let report=if let Some(loadout)=case.loadout{
            let initial=case.core();let e=M20DiagnosticEnvelope{loadout,initial_manifest_sha256:case.freeze,initial_public_sha256:m20_initial_public_sha256(&initial).unwrap()};
            assert!(record.to_json().is_err());replay_m20_diagnostic(&record,case.config.capacities.clone(),&e,case.freeze).unwrap()
        }else{let wire=record.to_json().unwrap();let imported=crate::RunEpisodeRecord::from_json(&wire).unwrap();assert_eq!(record,imported);crate::replay_run(&imported,case.config.capacities.clone()).unwrap()};
        assert_eq!(report.accepted_count,case.rows().len());assert_eq!(report.final_map,core.public_map());assert_eq!(report.reported_fault,record.fault);
        match core.decision(){Ok(d)=>{assert_eq!(report.final_decision,d);assert!(report.final_decision_error.is_none());},Err(e)=>assert_eq!(report.final_decision_error,Some(e))}
        println!("M20_NATIVE_AGREEMENT seed={} loadout={:?} scalar={} replay={} workers1={} workers2={} endpoint={}",case.seed,case.loadout,case.rows().len(),report.accepted_count,case.rows().len()*2,case.rows().len()*2,case.value["outcome"]);
        if s.fault.is_none(){
            if s.published.as_ref().unwrap().terminal.is_some(){assert_eq!(s.step(s.decision().unwrap().token,RunActionIndex(0)),Err(RunSessionError::Terminal));}
            let old=s.decision().unwrap().token;let reset=s.reset(Some(case.seed)).unwrap();assert_ne!(old,reset.token);assert!(s.record().unwrap().unwrap().steps.is_empty());assert!(s.reproduction().unwrap().latest_map.is_none());assert!(s.m20_diagnostic.is_none());
            for (batch,tokens) in &mut batches{
                let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();assert!(batch.reset(&[None]).is_err());
                for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),before[slot]);}
                batch.reset(&[Some(case.seed),None]).unwrap();let reset=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
                for slot in 0..2{
                    let mut expected=RunSession::new_m20(case.config.clone()).unwrap();expected.reset(Some(case.seed)).unwrap();if slot==1{expected.reset(None).unwrap();}
                    assert_eq!(reset[slot],expected.reproduction().unwrap());assert!(batch.record(slot).unwrap().unwrap().to_json().is_ok(),"supplied diagnostic encoding cleared on every reset slot");
                }
                assert!(matches!(batch.step(tokens,&[RunActionIndex(0);2]),Err(RunBatchError::Input{error:RunSessionError::StaleToken,..})));
                for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),reset[slot]);}
            }
        }
    }
    fn heterogeneous(cases:&[Case]){
        let cases=cases.iter().filter(|c|c.loadout.is_none()).collect::<Vec<_>>();let pair=[cases[0],cases[5]];
        for workers in [1,2]{
            let mut b=RunBatch::new_m20(pair.iter().map(|c|c.config.clone()).collect(),workers).unwrap();let mut bounds=b.reset(&pair.map(|c|Some(c.seed))).unwrap();
            let length=pair[0].rows().len().min(pair[1].rows().len());
            for i in 0..length{
                let actions=pair.map(|c|RunActionIndex(c.rows()[i]["candidate_index"].as_u64().unwrap() as i64));
                for slot in 0..2{let r=b.reproduction(slot).unwrap();assert_eq!(format!("{:?}",r.latest_decision.unwrap().unwrap()),text(&pair[slot].rows()[i]["pre"]["decision_debug"]));}
                let next=b.step(&bounds.iter().map(|x|x.token).collect::<Vec<_>>(),&actions).unwrap();
                for slot in 0..2{
                    let row=&pair[slot].rows()[i];assert_eq!(next[slot].reward,row["transition"]["reward"].as_i64().unwrap() as i32);
                    assert_eq!(format!("{:?}",next[slot].boundary.terminal),text(&row["transition"]["terminal"]));
                    let r=b.reproduction(slot).unwrap();assert_eq!(format!("{:?}",r.latest_decision.unwrap().unwrap()),text(&row["post"]["decision_debug"]));assert_eq!(format!("{:?}",r.latest_map),text(&row["post"]["map_debug"]));
                }
                bounds=next.into_iter().map(|r|r.boundary).collect();
            }
            let before=(0..2).map(|i|b.reproduction(i).unwrap()).collect::<Vec<_>>();let earliest=usize::from(pair[1].rows().len()<pair[0].rows().len());
            let terminal=before[earliest].latest_decision.as_ref().unwrap().as_ref().unwrap().terminal;
            if terminal.is_none(){println!("M20_NATIVE_HOLD heterogeneous earliest receipt ended without terminal at {length}; invalid-member preflight still checked");}
            let mut actions=[RunActionIndex(0);2];actions[earliest]=RunActionIndex(i64::MAX);
            assert!(matches!(b.step(&bounds.iter().map(|x|x.token).collect::<Vec<_>>(),&actions),Err(RunBatchError::Input{..})));
            for slot in 0..2{assert_eq!(b.reproduction(slot).unwrap(),before[slot]);}
            assert!(b.reset(&[None]).is_err());for slot in 0..2{assert_eq!(b.reproduction(slot).unwrap(),before[slot]);}
            b.reset(&[Some(pair[0].seed),None]).unwrap();for slot in 0..2{assert!(b.record(slot).unwrap().unwrap().steps.is_empty());assert!(b.reproduction(slot).unwrap().latest_map.is_none());}
        }
    }
    #[test]
    #[ignore="HOLD: requires global pre-outcome source/caller/cases/resources/RNG/caps/binary freeze and hash-released originals; never invokes a policy"]
    fn m20_recorded_native_agreement(){let cases=read_released();for case in &cases{consume(case);}heterogeneous(&cases);}
}

#[cfg(test)]
mod m20_freeze_contract_export{
    use super::*;
    #[test]
    #[ignore="Static contract export only; requires new retained SPIRE_M20_NATIVE_CONTRACT_PATH, no reset/policy/actions"]
    fn m20_export_freeze_contract(){
        use std::io::Write;
        let path=std::env::var_os("SPIRE_M20_NATIVE_CONTRACT_PATH").expect("retained contract path");let c=RunCapacities::m10_default();
        let normal=crate::m20_run_observation_spec(&c).unwrap();let supplied=crate::run_schema_m20::diagnostic_spec(&c).unwrap();
        let hex=|bytes:[u8;32]|bytes.iter().map(|byte|format!("{byte:02x}")).collect::<String>();
        let value=serde_json::json!({"schema_id":"m20-native-frozen-contract-v1","capacities":{"map_nodes":c.map_nodes,"map_edges":c.map_edges,"permanent_cards":c.permanent_cards,"combat_cards":c.combat_cards,"enemies":c.enemies,"powers":c.powers,"relics":c.relics,"potions":c.potions,"offers":c.offers,"selection":c.selection,"orbs":c.orbs,"actions":c.actions},"normal_fingerprint":hex(normal.fingerprint()),"diagnostic_fingerprint":hex(supplied.fingerprint()),"normal_descriptor":normal.descriptor(),"diagnostic_descriptor":supplied.descriptor(),"ordinary_portable_supplied_replay":false,"diagnostic_installation":"cfg(test) only; two fixed core constructors","workers":[1,2],"prescribed_receipt_consumer":"m20_recorded_native_agreement"});
        let bytes=serde_json::to_vec_pretty(&value).unwrap();let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();f.write_all(&bytes).unwrap();f.sync_all().unwrap();
    }
}

#[cfg(test)]
mod m22_actual_owner_tests{
 use super::*;
 use spire_core::{combat::{CombatAction,Stance},content::{CardId,RelicId},run::diagnostics::{M22Diagnostic as D,M22DiagnosticBoss as B,M22DiagnosticBranch as R}};
 fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:spire_core::combat::rng::Partition::Test,boss_history:RunBossHistory::Act1GuardianUnseen,capacities:RunCapacities::m10_default(),record_replay:true}}
 fn dispatch(s:&mut RunSession,a:GameAction)->Result<RunStepResult,RunSessionError>{let b=s.decision()?;let d=s.run.as_ref().unwrap().decision().unwrap();let i=d.candidates.iter().position(|c|c.action==a).unwrap();s.step(b.token,crate::RunActionIndex(i as i64))}
 fn internal_replay(s:&RunSession,fixture:D){let r=s.record().unwrap().unwrap();assert!(crate::replay_run(&r,s.config.capacities.clone()).is_err(),"supplied initializer is never ordinary portable proof");assert!(r.to_json().is_err());let report=crate::run_replay::replay_m22_diagnostic(&r,s.config.capacities.clone(),fixture).unwrap();assert_eq!(report.final_decision,s.run.as_ref().unwrap().decision().unwrap());assert_eq!(report.final_map,s.run.as_ref().unwrap().public_map());assert_eq!(report.accepted_count,r.steps.len());}
 #[test]
 fn m22_actual_cutoff_children_and_reset_are_healthy(){for boss in [B::Guardian,B::Hexaghost,B::SlimeBoss]{for branch in [R::HolyWater,R::VioletLotus,R::SacredBark,R::EmptyCage,R::SlaversCollar,R::Skip]{let fixture=D::Act2Ready{boss,branch};let mut s=RunSession::from_m22_diagnostic(config(),fixture).unwrap();let b=s.decision().unwrap();assert!(b.terminal.is_none());assert!(b.observation.action_mask.iter().all(|n|*n==0));let d=s.run.as_ref().unwrap().decision().unwrap();assert_eq!((d.act,d.floor),(2,17));assert_eq!(d.room.unwrap().kind,RoomViewKind::Act2Ready);assert!(s.run.as_ref().unwrap().public_map().is_some());let before=s.reproduction().unwrap();assert_eq!(s.step(b.token,crate::RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M22 stops at Act2Ready")));assert_eq!(s.reproduction().unwrap(),before);internal_replay(&s,fixture);let next=s.reset(None).unwrap();assert!(s.m22_diagnostic.is_none());assert!(next.observation.action_mask.iter().any(|n|*n==1));assert_eq!(s.ordinal,1);}}}
 #[test]
 fn m22_actual_scry_selected_order_and_zero_confirm_replay(){let fixture=D::ScryChoice;let mut s=RunSession::from_m22_diagnostic(config(),fixture).unwrap();let sel=s.run.as_ref().unwrap().decision().unwrap().combat.unwrap().selection.unwrap();let a=sel.candidates[0].instance.0;let b=sel.candidates[1].instance.0;for action in [CombatAction::SelectCard{instance:a},CombatAction::SelectCard{instance:b},CombatAction::DeselectCard{instance:a},CombatAction::SelectCard{instance:a}]{dispatch(&mut s,GameAction::Combat(action)).unwrap();}let d=s.run.as_ref().unwrap().decision().unwrap();assert_eq!(d.combat.as_ref().unwrap().selection.as_ref().unwrap().selected.iter().map(|x|x.0).collect::<Vec<_>>(),vec![b,a]);let group=s.decision().unwrap().observation.group("selection").unwrap().clone();assert_eq!((group.values[5],group.values[11]),(2,1));internal_replay(&s,fixture);dispatch(&mut s,GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();internal_replay(&s,fixture);let mut zero=RunSession::from_m22_diagnostic(config(),fixture).unwrap();dispatch(&mut zero,GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();internal_replay(&zero,fixture);}
 #[test]
 fn m22_actual_meditate_and_stance_choices_use_same_codec(){for fixture in [D::MeditateChoice,D::StanceChoice]{let mut s=RunSession::from_m22_diagnostic(config(),fixture).unwrap();internal_replay(&s,fixture);if fixture==D::StanceChoice{let o=s.decision().unwrap().observation;assert_eq!(o.group("stance_choices").unwrap().values,vec![1,2]);dispatch(&mut s,GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm})).unwrap();}else{let id=s.run.as_ref().unwrap().decision().unwrap().combat.unwrap().selection.unwrap().candidates[0].instance.0;dispatch(&mut s,GameAction::Combat(CombatAction::SelectCard{instance:id})).unwrap();dispatch(&mut s,GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();}internal_replay(&s,fixture);}}
 #[test]
 fn m22_actual_startup_fault_retains_holy_water_child_and_first_fault(){let fixture=D::BeforeStartupFault;let mut s=RunSession::from_m22_diagnostic(config(),fixture).unwrap();let d=s.run.as_ref().unwrap().decision().unwrap();assert_eq!(d.player.relics[0],RelicId::HolyWater);let a=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();let serial=s.serial;assert!(matches!(dispatch(&mut s,a),Err(RunSessionError::Core(RunError::Faulted(_)))));let run=s.run.as_ref().unwrap();let owner=run.clone();let cp=spire_core::run::diagnostics::m22_diagnostic_checkpoint(run).unwrap();assert_eq!(*run,owner);assert!(cp.run_relics.is_empty());let combat=cp.combat.unwrap();assert_eq!(combat.draw_count,10);assert_eq!(combat.pending_opening_hand,Some(5));assert!(combat.inventory_relics.contains(&RelicId::HolyWater));assert!(combat.fault.is_some());let created=combat.cards.iter().filter(|c|c.zone==spire_core::run::diagnostics::M22DiagnosticZone::Hand).collect::<Vec<_>>();assert_eq!(created.len(),1);assert_eq!(created[0].id,CardId::Miracle);assert_eq!(created[0].instance.0,u64::MAX);assert!(matches!(created[0].origin,spire_core::combat::CombatCardRef::CombatLocal(_)));let creation=combat.current_creation.unwrap();assert_eq!((creation.id,creation.count,creation.next,creation.template_upgrades),(CardId::Miracle,3,1,0));let rec=s.record().unwrap().unwrap();assert!(rec.steps.is_empty());assert!(rec.fault.as_ref().unwrap().committed);assert!(!rec.fault.as_ref().unwrap().gameplay_recorded);assert_eq!(s.serial,serial);let report=crate::run_replay::replay_m22_diagnostic(&rec,config().capacities,fixture).unwrap();assert!(report.final_decision_error.is_some());let before=s.reproduction().unwrap();assert!(s.reset(Some(9)).is_err());s.fail_delivery("later").unwrap();assert_eq!(s.reproduction().unwrap(),before);s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);}
 #[test]
 fn m22_actual_transition_fault_initializers_keep_flags_and_fault(){for fixture in [D::TransitionResetFault,D::TransitionHealFault]{let mut s=RunSession::from_m22_diagnostic(config(),fixture).unwrap();assert!(s.fault.is_some());let run=s.run.as_ref().unwrap();let owner=run.clone();let cp=spire_core::run::diagnostics::m22_diagnostic_checkpoint(run).unwrap();assert_eq!(*run,owner);let frame=cp.transition.unwrap();assert!(frame.reset_applied);assert_eq!(frame.heal_applied,fixture==D::TransitionHealFault);assert!(!frame.initialized);assert!(cp.run_fault.is_some());let before=s.reproduction().unwrap();assert!(s.reset(Some(7)).is_err());assert_eq!(s.reproduction().unwrap(),before);assert!(crate::run_replay::replay_m22_diagnostic(&s.record().unwrap().unwrap(),config().capacities,fixture).is_err());}}
 #[test]
 fn m22_encode_current_fields_are_public_and_owned(){let fixture=D::ScryChoice;let s=RunSession::from_m22_diagnostic(config(),fixture).unwrap();let mut d=s.run.as_ref().unwrap().decision().unwrap();let map=s.run.as_ref().unwrap().public_map();let c=d.combat.as_mut().unwrap();c.character.stance=Stance::Divinity;c.mantra_gained=Some(23);let card=&mut c.hand[0];card.values.as_mut().unwrap().base_damage=Some(18);card.values.as_mut().unwrap().base_block=Some(11);let w=card.watcher.as_mut().unwrap();w.combat_cost=Some(1);w.turn_cost=Some(0);w.retain=true;w.self_retain=true;let first=crate::encode_run_m22(&d,map.as_ref(),&config().capacities).unwrap();assert_eq!(first.group("player").unwrap().values[8],3);assert_eq!(first.group("player").unwrap().values[12],23);let row=first.group("combat_cards").unwrap();assert_eq!([row.values[8],row.values[9],row.values[19],row.values[20],row.values[21],row.values[22]],[18,11,1,0,1,1]);d.combat.as_mut().unwrap().hand[0].id=CardId::DefendPurple;assert_eq!(first.group("combat_cards").unwrap().values[8],18);}
}

#[cfg(test)]
mod m22_preinitializer_tests{
 use super::*;
 fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:spire_core::combat::rng::Partition::Test,boss_history:RunBossHistory::Act1GuardianUnseen,capacities:RunCapacities::m10_default(),record_replay:true}}
 #[test]
 fn m22_replay_identity_preflight_and_direct_ordinal_one_do_not_start_other_runs(){let mut s=RunSession::new_m22(config()).unwrap();s.reset(Some(7)).unwrap();s.reset(None).unwrap();let record=s.record().unwrap().unwrap();M22_INITIALIZATIONS.with(|v|v.borrow_mut().clear());let report=crate::replay_run(&record,config().capacities).unwrap();assert_eq!(report.final_decision,s.run.as_ref().unwrap().decision().unwrap());M22_INITIALIZATIONS.with(|v|assert_eq!(&*v.borrow(),&[1]));let mut variants=Vec::new();for field in 0..9{let mut r=record.clone();match field{0=>r.profile_id="wrong".into(),1=>r.schema_id="wrong".into(),2=>r.rules_id="wrong".into(),3=>r.content_id="wrong".into(),4=>r.seed_adapter_id="wrong".into(),5=>r.rng_id="wrong".into(),6=>r.reward_id="wrong".into(),7=>r.observation_fingerprint[0]^=1,_=>r.config.boss_history=RunBossHistory::AwakenedUnseen}variants.push(r);}for r in variants{M22_INITIALIZATIONS.with(|v|v.borrow_mut().clear());assert!(crate::replay_run(&r,config().capacities).is_err());M22_INITIALIZATIONS.with(|v|assert!(v.borrow().is_empty()));}}
}
#[cfg(test)]
mod m23_owner_tests{
 use super::*;
 fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:spire_core::combat::rng::Partition::Test,boss_history:RunBossHistory::M23WatcherAct2{act1:RunAct1BossTarget::SlimeBoss,act2:RunAct2BossTarget::BronzeAutomaton},capacities:RunCapacities::m10_default(),record_replay:true}}
 fn count()->Vec<u64>{M23_INITIALIZATIONS.with(|v|v.borrow().clone())}
 fn clear(){M23_INITIALIZATIONS.with(|v|v.borrow_mut().clear());}
 #[test]
 fn m23_config_history_rejection_precedes_initializer_and_ordinal_one_is_direct(){
  let bad=[RunSessionConfig{boss_history:RunBossHistory::Act1SlimeBossUnseen,..config()},RunSessionConfig{boss_history:RunBossHistory::M18DefectAct2{act1:RunAct1BossTarget::SlimeBoss,act2:RunAct2BossTarget::Champ},..config()},RunSessionConfig{character:Character::Silent,..config()},RunSessionConfig{ascension:1,..config()},RunSessionConfig{capacities:RunCapacities{actions:0,..RunCapacities::m10_default()},..config()}];
  clear();for c in bad{assert!(RunSession::new_m23(c).is_err());}assert!(count().is_empty(),"rejected configs never reach the initializer");
  let mut s=RunSession::new_m23(config()).unwrap();assert!(count().is_empty(),"construction alone does not start a Run");s.reset(Some(7)).unwrap();s.reset(None).unwrap();assert_eq!(count(),vec![0,1]);
  let record=s.record().unwrap().unwrap();clear();let report=crate::replay_run(&record,config().capacities).unwrap();assert_eq!(report.final_decision,s.run.as_ref().unwrap().decision().unwrap());assert_eq!(count(),vec![1],"recorded implicit ordinal initialized directly");
  let mut variants=Vec::new();for field in 0..11{let mut r=record.clone();match field{0=>r.profile_id="M22WatcherAct1".into(),1=>r.schema_id=RUN_RECORD_SCHEMA_ID_M22.into(),2=>r.rules_id=RUN_RULES_ID_M22.into(),3=>r.content_id=RUN_CONTENT_ID_M22.into(),4=>r.seed_adapter_id="wrong".into(),5=>r.rng_id="wrong".into(),6=>r.reward_id="wrong".into(),7=>r.observation_fingerprint[0]^=1,8=>r.config.boss_history=RunBossHistory::Act1SlimeBossUnseen,9=>r.config.character=Character::Defect,_=>r.config.capacities.actions=1}variants.push(r);}
  for r in variants{clear();assert!(matches!(crate::replay_run(&r,config().capacities),Err(RunSessionError::IncompatibleReplay(_))));assert!(count().is_empty(),"identity preflight precedes startup");}
 }
 #[test]
 fn m23_act3ready_boundary_and_labelled_forced_cutoff_are_healthy_nonfaulting(){
  // Constructed public boundary (labelled): no retained M23 transcript reaches the Act2 boss yet.
  let mut s=RunSession::new_m23(config()).unwrap();s.reset(Some(7)).unwrap();let mut d=s.run.as_ref().unwrap().decision().unwrap();d.act=3;d.floor=34;d.room.as_mut().unwrap().kind=RoomViewKind::Act3Ready;d.room.as_mut().unwrap().choices.clear();let with_actions=d.clone();d.candidates.clear();
  assert_eq!(check_boundary(RunProfile::M23WatcherAct2,&d),Ok(()));assert!(check_boundary(RunProfile::M23WatcherAct2,&with_actions).is_err());
  let mut other=d.clone();other.room.as_mut().unwrap().kind=RoomViewKind::Map;assert_eq!(check_boundary(RunProfile::M23WatcherAct2,&other),Err(RunSessionError::UnsupportedBoundary("nonterminal Run has no legal actions")));
  // TEST-ONLY forced cutoff flag on a real session: not normal-play proof.
  s.at_m13_cutoff=true;let b=s.decision().unwrap();let before=s.reproduction().unwrap();let serial=s.serial;
  assert_eq!(s.step(b.token,crate::RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M23 stops at Act3Ready")));assert_eq!(s.preflight_step(b.token,crate::RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M23 stops at Act3Ready")));
  assert_eq!(s.reproduction().unwrap(),before);assert!(s.fault.is_none());assert_eq!(s.serial,serial);assert!(s.record().unwrap().unwrap().fault.is_none());
  let next=s.reset(None).unwrap();assert!(!s.at_m13_cutoff);assert!(next.observation.action_mask.iter().any(|m|*m==1));assert_eq!(s.ordinal,1);s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);
  let mut t=RunSession::new_m23(config()).unwrap();assert_eq!(t.combat_observation_spec().err(),Some(RunSessionError::UnsupportedBoundary("M23 does not provide the Ironclad combat policy sidecar")));t.reset(Some(7)).unwrap();assert_eq!(t.combat_view().err(),Some(RunSessionError::UnsupportedBoundary("M23 does not provide the Ironclad combat policy sidecar")));assert!(t.fault.is_none());
  let env=t.environment_spec().unwrap();assert_eq!(env["profile_id"],"M23WatcherAct2");assert!(env["combat_view_id"].is_null());assert_eq!(env["finite_content"],crate::run_schema_m23::finite_content());assert!(env["cutoff"].as_str().unwrap().starts_with("Act3Ready"));
 }
}
#[cfg(test)]
mod m23_actual_owner_tests{
 //! Constructed/supplied actual core owner stages (spire_core::run::diagnostics::M23Diagnostic): a constructed (1,2,17)
 //! predecessor and visit0 boss record. Internal SAME-initializer replay only; never normal portable replay or played Victory.
 use super::*;
 use spire_core::{combat::{CombatAction,Stance},content::{EncounterId,RelicId},run::diagnostics::{M23Diagnostic as D,M23DiagnosticBoss as B,M22DiagnosticBranch as R}};
 const BRANCHES:[R;6]=[R::HolyWater,R::VioletLotus,R::SacredBark,R::EmptyCage,R::SlaversCollar,R::Skip];
 fn config(b:B)->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:spire_core::combat::rng::Partition::Test,boss_history:RunBossHistory::M23WatcherAct2{act1:RunAct1BossTarget::Guardian,act2:match b{B::Champ=>RunAct2BossTarget::Champ,B::BronzeAutomaton=>RunAct2BossTarget::BronzeAutomaton,B::Collector=>RunAct2BossTarget::Collector}},capacities:RunCapacities::m10_default(),record_replay:true}}
 fn decision(s:&RunSession)->PublicDecision{s.run.as_ref().unwrap().decision().unwrap()}
 fn dispatch(s:&mut RunSession,a:GameAction)->Result<RunStepResult,RunSessionError>{let b=s.decision()?;let i=decision(s).candidates.iter().position(|c|c.action==a).expect("offered");s.step(b.token,crate::RunActionIndex(i as i64))}
 fn choose(s:&mut RunSession,kind:RoomChoiceKind)->RunStepResult{let d=decision(s);let id=d.room.unwrap().choices.iter().find(|c|c.kind==kind&&c.status==RoomChoiceStatus::Offered).unwrap_or_else(||panic!("{kind:?} offered")).id;dispatch(s,GameAction::Choose(id)).unwrap()}
 fn internal_replay(s:&RunSession,fixture:D){let r=s.record().unwrap().unwrap();assert!(crate::replay_run(&r,s.config.capacities.clone()).is_err(),"constructed initializer is never ordinary portable proof");assert!(r.to_json().is_err(),"constructed records are not portable JSON");let report=crate::run_replay::replay_m23_diagnostic(&r,s.config.capacities.clone(),fixture).unwrap();assert_eq!(report.final_decision,decision(s));assert_eq!(report.final_map,s.run.as_ref().unwrap().public_map());assert_eq!(report.accepted_count,r.steps.len());}
 fn assert_act3ready(s:&mut RunSession){
  let b=s.decision().unwrap();let d=decision(s);assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind),(3,34,RoomViewKind::Act3Ready));assert!(d.candidates.is_empty()&&d.terminal.is_none()&&b.terminal.is_none());assert_eq!(b.observation.action_mask,vec![0;s.config.capacities.actions]);
  let map=s.run.as_ref().unwrap().public_map().unwrap();assert_eq!(map.act,3);let icon=map.boss.unwrap();assert!(matches!(icon,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca),"{icon:?}");
  assert_eq!(b.observation.group("context").unwrap().values[11],crate::run_schema_m23::encounter(icon).unwrap(),"Act3 map-only boss icon encoded");
  let spec=s.observation_spec().unwrap().descriptor();assert!(!spec["semantics"]["finite_content"]["playable_encounters"].as_array().unwrap().contains(&serde_json::json!(icon.as_str())));
  let before=s.reproduction().unwrap();let serial=s.serial;assert_eq!(s.step(b.token,crate::RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M23 stops at Act3Ready")));assert_eq!(s.preflight_step(b.token,crate::RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M23 stops at Act3Ready")));
  assert_eq!(s.reproduction().unwrap(),before);assert!(s.fault.is_none());assert_eq!(s.serial,serial);assert!(s.record().unwrap().unwrap().fault.is_none());
 }
 #[test]
 fn m23_lifecycle_actual_boss_choices_reach_healthy_act3ready_for_every_branch(){
  for boss in B::ALL{
   let s=RunSession::from_m23_diagnostic(config(boss),D::BossRewards{boss}).unwrap();let d=decision(&s);assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind),(2,33,RoomViewKind::Rewards));assert_eq!(d.content_profile,Some(RoomScenario::M23WatcherAct2));internal_replay(&s,D::BossRewards{boss});
   let menu=RunSession::from_m23_diagnostic(config(boss),D::BossRelicMenu{boss}).unwrap();let md=decision(&menu);assert_eq!((md.floor,md.room.as_ref().unwrap().kind),(34,RoomViewKind::BossRelicChoice));
   let offered=md.room.as_ref().unwrap().choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>();for id in spire_core::run::m22_content::BOSS_RELICS{assert!(offered.contains(&RoomChoiceKind::Relic(id)),"{id:?}");}assert!(offered.contains(&RoomChoiceKind::Skip));internal_replay(&menu,D::BossRelicMenu{boss});
   for branch in BRANCHES{
    // Constructed child stage, then the actual child/Proceed through the env session to the real (2,3,34) transition.
    let fixture=D::BossChild{boss,branch};let mut c=RunSession::from_m23_diagnostic(config(boss),fixture).unwrap();assert!(c.fault.is_none(),"{boss:?}/{branch:?}");internal_replay(&c,fixture);
    while let Some(a)=decision(&c).candidates.iter().find(|x|matches!(x.action,GameAction::Select(_))).map(|x|x.action.clone()){dispatch(&mut c,a).unwrap();}
    choose(&mut c,if branch==R::Skip{RoomChoiceKind::ConfirmSkip}else{RoomChoiceKind::Proceed});assert_act3ready(&mut c);internal_replay(&c,fixture);
    // Same cutoff from the actual Rewards stage, every step through the env session.
    let mut e=RunSession::from_m23_diagnostic(config(boss),D::BossRewards{boss}).unwrap();choose(&mut e,RoomChoiceKind::Proceed);choose(&mut e,RoomChoiceKind::OpenChest);
    if branch==R::Skip{choose(&mut e,RoomChoiceKind::Skip);}else{let id=match branch{R::HolyWater=>RelicId::HolyWater,R::VioletLotus=>RelicId::VioletLotus,R::SacredBark=>RelicId::SacredBark,R::EmptyCage=>RelicId::EmptyCage,R::SlaversCollar=>RelicId::SlaversCollar,R::Skip=>unreachable!()};choose(&mut e,RoomChoiceKind::Relic(id));assert!(decision(&e).player.relics.contains(&id)||branch==R::HolyWater&&decision(&e).player.relics[0]==RelicId::HolyWater);}
    while let Some(a)=decision(&e).candidates.iter().find(|x|matches!(x.action,GameAction::Select(_))).map(|x|x.action.clone()){dispatch(&mut e,a).unwrap();}
    choose(&mut e,if branch==R::Skip{RoomChoiceKind::ConfirmSkip}else{RoomChoiceKind::Proceed});assert_act3ready(&mut e);internal_replay(&e,D::BossRewards{boss});
    // Constructed Act3Ready stage itself; healthy reset leaves the diagnostic owner.
    let fixture=D::Act3Ready{boss,branch};let mut r=RunSession::from_m23_diagnostic(config(boss),fixture).unwrap();assert_act3ready(&mut r);internal_replay(&r,fixture);
    let next=r.reset(None).unwrap();assert!(r.m23_diagnostic.is_none());assert!(next.observation.action_mask.iter().any(|n|*n==1));assert_eq!(r.ordinal,1);
   }
  }
 }
 #[test]
 fn m23_semantic_actual_scry_meditate_and_stance_replay_with_m23_identity(){
  let cfg=config(B::Champ);let fixture=D::ScryChoice;let mut s=RunSession::from_m23_diagnostic(cfg.clone(),fixture).unwrap();assert_eq!(decision(&s).content_profile,Some(RoomScenario::M23WatcherAct2));
  let sel=decision(&s).combat.unwrap().selection.unwrap();assert_eq!(sel.operation,spire_core::combat::SelectionOperation::Scry);let (a,b)=(sel.candidates[0].instance.0,sel.candidates[1].instance.0);
  for x in [CombatAction::SelectCard{instance:a},CombatAction::SelectCard{instance:b},CombatAction::DeselectCard{instance:a},CombatAction::SelectCard{instance:a}]{dispatch(&mut s,GameAction::Combat(x)).unwrap();}
  assert_eq!(decision(&s).combat.unwrap().selection.unwrap().selected.iter().map(|x|x.0).collect::<Vec<_>>(),vec![b,a]);let o=s.decision().unwrap().observation;let g=o.group("selection").unwrap();assert_eq!((g.values[4],g.values[5],g.values[10],g.values[11]),(0,2,1,1));
  assert_eq!(o.group("context").unwrap().values[17],1,"M23 content profile row");internal_replay(&s,fixture);dispatch(&mut s,GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();internal_replay(&s,fixture);
  let mut zero=RunSession::from_m23_diagnostic(cfg.clone(),fixture).unwrap();dispatch(&mut zero,GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();internal_replay(&zero,fixture);
  let fixture=D::MeditateChoice;let mut m=RunSession::from_m23_diagnostic(cfg.clone(),fixture).unwrap();let sel=decision(&m).combat.unwrap().selection.unwrap();assert_eq!(sel.operation,spire_core::combat::SelectionOperation::MeditateReturn);
  assert_eq!(m.decision().unwrap().observation.group("selection").unwrap().occupied.as_ref().unwrap().iter().filter(|x|**x==1).count(),sel.candidates.len());internal_replay(&m,fixture);dispatch(&mut m,GameAction::Combat(CombatAction::SelectCard{instance:sel.candidates[0].instance.0})).unwrap();dispatch(&mut m,GameAction::Combat(CombatAction::ConfirmSelection)).unwrap();internal_replay(&m,fixture);
  let fixture=D::StanceChoice;let mut t=RunSession::from_m23_diagnostic(cfg,fixture).unwrap();assert_eq!(t.decision().unwrap().observation.group("stance_choices").unwrap().values,vec![1,2]);internal_replay(&t,fixture);
  dispatch(&mut t,GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm})).unwrap();assert_eq!(decision(&t).combat.unwrap().character.stance,Stance::Calm);assert_eq!(t.decision().unwrap().observation.group("player").unwrap().values[8],2);internal_replay(&t,fixture);
  // Diagnostic descriptors are distinct from the normal M23 fingerprint; ordinary M23 stays exportable.
  assert_ne!(t.observation_spec().unwrap().fingerprint(),crate::m23_run_observation_spec(&RunCapacities::m10_default()).unwrap().fingerprint());
 }
 #[test]
 fn m23_delivery_fault_record_preserves_committed_semantics_with_actual_core_faults(){
  let fixture=D::BeforeStartupFault;let mut s=RunSession::from_m23_diagnostic(config(B::Champ),fixture).unwrap();let d=decision(&s);assert!(d.player.relics.contains(&RelicId::HolyWater));
  let a=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();let serial=s.serial;let token=s.decision().unwrap().token;
  let err=dispatch(&mut s,a.clone()).unwrap_err();assert!(matches!(err,RunSessionError::Core(RunError::Faulted(_))));assert_eq!(s.fault.as_ref(),Some(&err),"first structured error retained");assert_eq!(s.serial,serial);
  let run=s.run.as_ref().unwrap();let owner=run.clone();let cp=spire_core::run::diagnostics::m22_diagnostic_checkpoint(run).unwrap();assert_eq!(*run,owner,"checkpoint is read-only");let combat=cp.combat.unwrap();assert!(combat.fault.is_some());assert!(combat.inventory_relics.contains(&RelicId::HolyWater));
  let rec=s.record().unwrap().unwrap();assert!(rec.steps.is_empty());let f=rec.fault.as_ref().unwrap();assert!(f.committed&&!f.gameplay_recorded);assert_eq!(f.action.as_ref(),Some(&a));assert_eq!(f.error.kind,RunRecordErrorKind::Core);assert!(f.error.diagnostic_json().is_ok());
  let report=crate::run_replay::replay_m23_diagnostic(&rec,config(B::Champ).capacities,fixture).unwrap();assert!(report.final_decision_error.is_some(),"core fault reproduced once at its checkpoint");assert_eq!(report.reported_fault,rec.fault);assert_eq!(report.accepted_count,0);
  let mut tampered=rec.clone();tampered.fault.as_mut().unwrap().error.message="other checkpoint".into();assert!(crate::run_replay::replay_m23_diagnostic(&tampered,config(B::Champ).capacities,fixture).is_err(),"checkpoint must match");
  let before=s.reproduction().unwrap();assert!(s.reset(Some(9)).is_err());assert!(s.reset(None).is_err());assert_eq!(s.step(token,crate::RunActionIndex(0)),Err(err.clone()));s.fail_delivery("later").unwrap();assert_eq!(s.reproduction().unwrap(),before,"faulted owner cannot retry into a new loss");
  s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);assert!(s.reproduction().unwrap().closed);assert_eq!(s.reproduction().unwrap().fault,Some(err));
  for fixture in [D::TransitionResetFault,D::TransitionHealFault]{
   let mut s=RunSession::from_m23_diagnostic(config(B::Champ),fixture).unwrap();assert!(matches!(s.fault,Some(RunSessionError::Core(RunError::Faulted(_)))));let run=s.run.as_ref().unwrap();let cp=spire_core::run::diagnostics::m22_diagnostic_checkpoint(run).unwrap();
   let frame=cp.transition.unwrap();assert!(frame.reset_applied);assert_eq!(frame.heal_applied,fixture==D::TransitionHealFault);assert!(!frame.initialized);assert!(cp.run_fault.is_some());
   let rec=s.record().unwrap().unwrap();let f=rec.fault.as_ref().unwrap();assert!(!f.committed&&!f.gameplay_recorded&&f.action.is_none());assert_eq!(f.error.kind,RunRecordErrorKind::Core);
   let before=s.reproduction().unwrap();assert!(s.reset(Some(7)).is_err());assert_eq!(s.reproduction().unwrap(),before,"no reheal/regenerate");assert!(crate::run_replay::replay_m23_diagnostic(&rec,config(B::Champ).capacities,fixture).is_err());s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);
  }
 }
}
#[cfg(test)]
mod m24_owner_tests{
 use super::*;
 fn config(a1:RunAct1BossTarget,a2:RunAct2BossTarget,a3:RunAct3BossTarget)->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:spire_core::combat::rng::Partition::Test,boss_history:RunBossHistory::M24WatcherAct3{act1:a1,act2:a2,act3:a3},capacities:RunCapacities::m10_default(),record_replay:true}}
 fn count()->Vec<(u64,[Vec<&'static str>;3])>{M24_INITIALIZATIONS.with(|v|v.borrow().clone())}
 fn clear(){M24_INITIALIZATIONS.with(|v|v.borrow_mut().clear());}
 #[test]
 fn m24_config_history_validation_precedes_initializer_and_installs_all_three_complements(){
  let base=config(RunAct1BossTarget::SlimeBoss,RunAct2BossTarget::BronzeAutomaton,RunAct3BossTarget::TimeEater);let (g,c,a)=(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne);
  let bad=[RunSessionConfig{boss_history:RunBossHistory::Act1SlimeBossUnseen,..base.clone()},RunSessionConfig{boss_history:RunBossHistory::M23WatcherAct2{act1:g,act2:c},..base.clone()},RunSessionConfig{boss_history:RunBossHistory::M19DefectAct3{act1:g,act2:c,act3:a},..base.clone()},RunSessionConfig{boss_history:RunBossHistory::M14SilentAct3{act1:g,act2:c,act3:a},..base.clone()},RunSessionConfig{character:Character::Silent,..base.clone()},RunSessionConfig{ascension:1,..base.clone()},RunSessionConfig{capacities:RunCapacities{actions:0,..RunCapacities::m10_default()},..base.clone()}];
  clear();for cfg in bad{assert!(RunSession::new_m24(cfg).is_err());}assert!(count().is_empty(),"rejected configs never reach the initializer");
  // All 27 typed triples install the three unseen-boss complements before Run startup (Automaton is the Bronze Automaton history string).
  let c1=|t|match t{RunAct1BossTarget::Guardian=>vec!["Hexaghost","SlimeBoss"],RunAct1BossTarget::Hexaghost=>vec!["Guardian","SlimeBoss"],RunAct1BossTarget::SlimeBoss=>vec!["Guardian","Hexaghost"]};
  let c2=|t|match t{RunAct2BossTarget::Champ=>vec!["Automaton","Collector"],RunAct2BossTarget::BronzeAutomaton=>vec!["Champ","Collector"],RunAct2BossTarget::Collector=>vec!["Champ","Automaton"]};
  let c3=|t|match t{RunAct3BossTarget::AwakenedOne=>vec!["TimeEater","DonuAndDeca"],RunAct3BossTarget::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],RunAct3BossTarget::DonuAndDeca=>vec!["AwakenedOne","TimeEater"]};
  for a1 in [RunAct1BossTarget::Guardian,RunAct1BossTarget::Hexaghost,RunAct1BossTarget::SlimeBoss]{for a2 in [RunAct2BossTarget::Champ,RunAct2BossTarget::BronzeAutomaton,RunAct2BossTarget::Collector]{for a3 in [RunAct3BossTarget::AwakenedOne,RunAct3BossTarget::TimeEater,RunAct3BossTarget::DonuAndDeca]{
   clear();let mut s=RunSession::new_m24(config(a1,a2,a3)).unwrap();assert!(count().is_empty(),"construction alone does not start a Run");s.reset(Some(7)).unwrap();assert_eq!(count(),vec![(0,[c1(a1),c2(a2),c3(a3)])],"{a1:?}/{a2:?}/{a3:?}");
  }}}
  // The M14 Silent triple installs the same shared complements (no second complement table).
  let m14=RunSessionConfig{character:Character::Silent,boss_history:RunBossHistory::M14SilentAct3{act1:RunAct1BossTarget::SlimeBoss,act2:RunAct2BossTarget::BronzeAutomaton,act3:RunAct3BossTarget::TimeEater},..base.clone()};assert!(create_run(RunProfile::M14SilentAct3,&m14,7,0).is_ok());
  // Implicit ordinal and replay identity preflight.
  clear();let mut s=RunSession::new_m24(base.clone()).unwrap();s.reset(Some(7)).unwrap();s.reset(None).unwrap();assert_eq!(count().iter().map(|x|x.0).collect::<Vec<_>>(),vec![0,1]);
  let record=s.record().unwrap().unwrap();clear();let report=crate::replay_run(&record,base.capacities.clone()).unwrap();assert_eq!(report.final_decision,s.run.as_ref().unwrap().decision().unwrap());assert_eq!(count().iter().map(|x|x.0).collect::<Vec<_>>(),vec![1],"recorded implicit ordinal initialized directly");
  let mut variants=Vec::new();for field in 0..12{let mut r=record.clone();match field{0=>r.profile_id="M23WatcherAct2".into(),1=>r.schema_id=RUN_RECORD_SCHEMA_ID_M23.into(),2=>r.rules_id=RUN_RULES_ID_M23.into(),3=>r.content_id=RUN_CONTENT_ID_M23.into(),4=>r.seed_adapter_id="wrong".into(),5=>r.rng_id="wrong".into(),6=>r.reward_id="wrong".into(),7=>r.observation_fingerprint[0]^=1,8=>r.config.boss_history=RunBossHistory::M23WatcherAct2{act1:g,act2:c},9=>r.config.boss_history=RunBossHistory::M19DefectAct3{act1:g,act2:c,act3:a},10=>r.config.character=Character::Defect,_=>r.config.capacities.actions=1}variants.push(r);}
  for r in variants{clear();assert!(matches!(crate::replay_run(&r,base.capacities.clone()),Err(RunSessionError::IncompatibleReplay(_))));assert!(count().is_empty(),"identity preflight precedes startup");}
 }
 #[test]
 fn m24_check_boundary_act4ready_is_m14_style_healthy_or_named_error(){
  // Constructed public boundary (labelled): only a nonterminal zero-candidate Act4Ready is a healthy M24 boundary.
  let cfg=config(RunAct1BossTarget::Guardian,RunAct2BossTarget::Champ,RunAct3BossTarget::AwakenedOne);let mut d=create_run(RunProfile::M24WatcherAct3,&cfg,7,0).unwrap().decision().unwrap();
  d.act=3;d.floor=50;d.room.as_mut().unwrap().kind=RoomViewKind::Act4Ready;d.room.as_mut().unwrap().choices.clear();let with_actions=d.clone();d.candidates.clear();
  assert_eq!(check_boundary(RunProfile::M24WatcherAct3,&d),Ok(()));
  assert_eq!(check_boundary(RunProfile::M24WatcherAct3,&with_actions),Err(RunSessionError::UnsupportedBoundary("M24 Act4Ready must be nonterminal with no actions")));
  let mut terminal=d.clone();terminal.terminal=Some(RunTerminal::Act3NoKeys);assert_eq!(check_boundary(RunProfile::M24WatcherAct3,&terminal),Err(RunSessionError::UnsupportedBoundary("M24 Act4Ready must be nonterminal with no actions")));
  let mut other=d.clone();other.room.as_mut().unwrap().kind=RoomViewKind::Map;assert_eq!(check_boundary(RunProfile::M24WatcherAct3,&other),Err(RunSessionError::UnsupportedBoundary("nonterminal Run has no legal actions")));
  // M23 and M9 keep their own boundary laws.
  assert_eq!(check_boundary(RunProfile::M9IroncladAct3,&d),Err(RunSessionError::UnsupportedBoundary("M9 stops before the Act4 door")));assert!(check_boundary(RunProfile::M23WatcherAct2,&d).is_err());
 }
}
#[cfg(test)]
mod m24_actual_owner_tests{
 //! Constructed/supplied actual core owner stages (spire_core::run::diagnostics m24): a constructed boss50 Victory record and
 //! constructed Act3 room edges. Internal SAME-initializer replay only; never normal portable replay or a played Victory.
 use super::*;
 use spire_core::{combat::{CombatAction,Stance},run::diagnostics::{M24Diagnostic as D,M24DiagnosticBoss as B}};
 const CUTOFF:RunSessionError=RunSessionError::UnsupportedBoundary("M24 stops before the Act4 door");
 fn config(b:B)->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:spire_core::combat::rng::Partition::Test,boss_history:RunBossHistory::M24WatcherAct3{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:match b{B::AwakenedOne=>RunAct3BossTarget::AwakenedOne,B::TimeEater=>RunAct3BossTarget::TimeEater,B::DonuAndDeca=>RunAct3BossTarget::DonuAndDeca}},capacities:RunCapacities::m10_default(),record_replay:true}}
 fn stage(d:D)->M24DiagnosticStart{M24DiagnosticStart::Stage(d)}
 fn decision(s:&RunSession)->PublicDecision{s.run.as_ref().unwrap().decision().unwrap()}
 fn dispatch(s:&mut RunSession,a:GameAction)->Result<RunStepResult,RunSessionError>{let b=s.decision()?;let i=decision(s).candidates.iter().position(|c|c.action==a).expect("offered");s.step(b.token,crate::RunActionIndex(i as i64))}
 fn choose(s:&mut RunSession,kind:RoomChoiceKind)->RunStepResult{let d=decision(s);let id=d.room.unwrap().choices.iter().find(|c|c.kind==kind&&c.status==RoomChoiceStatus::Offered).unwrap_or_else(||panic!("{kind:?} offered")).id;dispatch(s,GameAction::Choose(id)).unwrap()}
 fn internal_replay(s:&RunSession,start:M24DiagnosticStart){let r=s.record().unwrap().unwrap();assert!(crate::replay_run(&r,s.config.capacities.clone()).is_err(),"constructed initializer is never ordinary portable proof");assert!(r.to_json().is_err(),"constructed records are not portable JSON");let report=crate::run_replay::replay_m24_diagnostic(&r,s.config.capacities.clone(),start).unwrap();if let Ok(d)=s.run.as_ref().unwrap().decision(){assert_eq!(report.final_decision,d);}assert_eq!(report.final_map,s.run.as_ref().unwrap().public_map());assert_eq!(report.accepted_count,r.steps.len());}
 const MASKS:[[bool;3];8]=[[false,false,false],[true,false,false],[false,true,false],[false,false,true],[true,true,false],[true,false,true],[false,true,true],[true,true,true]];
 #[test]
 fn m24_lifecycle_and_cutoffs_on_constructed_ending(){
  for boss in B::ALL{for keys in MASKS{
   let start=stage(D::BossSource{boss,keys});let mut s=RunSession::from_m24_diagnostic(config(boss),start).unwrap();assert!(s.fault.is_none());let d=decision(&s);
   assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind,d.content_profile),(3,50,RoomViewKind::Act3BossComplete,Some(RoomScenario::M24WatcherAct3)));assert_eq!(d.player.keys,keys);internal_replay(&s,start);
   let p=choose(&mut s,RoomChoiceKind::Proceed);assert!(!p.terminated&&p.boundary.terminal.is_none());
   for st in 1..=2u8{let x=choose(&mut s,RoomChoiceKind::Act3Continue{stage:st});assert!(!x.terminated);assert_eq!(decision(&s).room.unwrap().kind,RoomViewKind::Act3Ending);}
   // The step that accepts dialog stage 3 succeeds.
   let x=choose(&mut s,RoomChoiceKind::Act3Continue{stage:3});
   if keys==[true;3]{
    assert!(!x.terminated&&!x.truncated);assert_eq!(x.boundary.terminal,None);assert_eq!(x.reward,0,"no readiness reward");assert_eq!(x.boundary.observation.action_mask,vec![0;s.config.capacities.actions]);
    let d=decision(&s);assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind),(3,50,RoomViewKind::Act4Ready));assert!(d.candidates.is_empty()&&d.terminal.is_none());assert!(s.at_m14_cutoff&&s.fault.is_none());
    // The next step returns the exact cutoff and changes nothing; a stale token at the cutoff is still StaleToken.
    let b=s.decision().unwrap();let before=s.reproduction().unwrap();let serial=s.serial;let steps=s.record().unwrap().unwrap().steps.len();
    assert_eq!(s.step(b.token,crate::RunActionIndex(0)),Err(CUTOFF));assert_eq!(s.preflight_step(b.token,crate::RunActionIndex(0)),Err(CUTOFF));assert_eq!(s.step(b.token,crate::RunActionIndex(-1)),Err(CUTOFF),"cutoff precedes index validation");
    assert_eq!(s.step(RunDecisionToken([s.id,serial-1]),crate::RunActionIndex(0)),Err(RunSessionError::StaleToken));
    assert_eq!(s.reproduction().unwrap(),before);assert!(s.fault.is_none());assert_eq!(s.serial,serial);let rec=s.record().unwrap().unwrap();assert!(rec.fault.is_none());assert_eq!(rec.steps.len(),steps);assert_eq!(rec.terminal,None);
    internal_replay(&s,start);
    // Reset, record, reproduction and close stay available; healthy reset leaves the diagnostic owner.
    let next=s.reset(None).unwrap();assert!(s.m24_diagnostic.is_none()&&!s.at_m14_cutoff);assert!(next.observation.action_mask.iter().any(|n|*n==1));assert_eq!(s.ordinal,1);s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);
   }else{
    assert!(!x.terminated);let d=decision(&s);assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3Ending);assert!(!s.at_m14_cutoff);
    let t=choose(&mut s,RoomChoiceKind::Act3Continue{stage:4});assert!(t.terminated&&!t.truncated);assert_eq!(t.boundary.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(t.reward,0,"Act3NoKeys terminal contributes 0");assert!(t.boundary.observation.action_mask.iter().all(|m|*m==0));
    let d=decision(&s);assert_eq!((d.act,d.floor,d.terminal),(3,50,Some(RunTerminal::Act3NoKeys)));assert!(!s.at_m14_cutoff);
    assert_eq!(s.step(t.boundary.token,crate::RunActionIndex(0)),Err(RunSessionError::Terminal));let rec=s.record().unwrap().unwrap();assert_eq!(rec.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(rec.steps.len(),5);internal_replay(&s,start);
    s.reset(Some(7)).unwrap();assert_eq!(s.ordinal,0);
   }
  }}
 }
 #[test]
 fn m24_semantic_replay_on_constructed_stages(){
  let cfg=config(B::AwakenedOne);
  // Falling: the offered type choice replays at the recorded ordinal.
  let start=stage(D::Falling);let mut s=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();let d=decision(&s);assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Event(spire_core::content::EventId::Falling));internal_replay(&s,start);
  let kind=d.room.unwrap().choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{..})&&c.status==RoomChoiceStatus::Offered).expect("Falling type offered").kind.clone();choose(&mut s,kind);internal_replay(&s,start);
  // Tomb with and without a mask.
  for has_mask in [false,true]{let start=stage(D::Tomb{has_mask});let mut t=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();let d=decision(&t);internal_replay(&t,start);
   let pick=if has_mask{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold};assert!(d.room.as_ref().unwrap().choices.iter().any(|c|c.kind==pick),"{pick:?}");choose(&mut t,pick);internal_replay(&t,start);}
  // Chest.
  let start=stage(D::Chest);let mut c=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();choose(&mut c,RoomChoiceKind::OpenChest);internal_replay(&c,start);
  // Scry: select/deselect/confirm.
  let start=stage(D::ScryChoice);let mut s=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();let sel=decision(&s).combat.unwrap().selection.unwrap();let (a,b)=(sel.candidates[0].instance.0,sel.candidates[1].instance.0);
  for x in [CombatAction::SelectCard{instance:a},CombatAction::SelectCard{instance:b},CombatAction::DeselectCard{instance:a},CombatAction::ConfirmSelection]{dispatch(&mut s,GameAction::Combat(x)).unwrap();}internal_replay(&s,start);
  // Meditate return.
  let start=stage(D::MeditateChoice);let mut m=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();let sel=decision(&m).combat.unwrap().selection.unwrap();assert_eq!(sel.operation,spire_core::combat::SelectionOperation::MeditateReturn);let confirm=GameAction::Combat(CombatAction::ConfirmSelection);for c in &sel.candidates{if decision(&m).candidates.iter().any(|x|x.action==confirm){break}dispatch(&mut m,GameAction::Combat(CombatAction::SelectCard{instance:c.instance.0})).unwrap();}assert!(sel.candidates.len()>=2);dispatch(&mut m,confirm).unwrap();internal_replay(&m,start);
  // Stance choice.
  let start=stage(D::StanceChoice);let mut t=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();assert_eq!(t.decision().unwrap().observation.group("stance_choices").unwrap().values,vec![1,2]);dispatch(&mut t,GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Wrath})).unwrap();assert_eq!(decision(&t).combat.unwrap().character.stance,Stance::Wrath);internal_replay(&t,start);
  // The real M24 (2,3,34) continuation reaches the Act3 Map with Move candidates; the first Move replays.
  for boss in B::ALL{let start=stage(D::Act3Map{boss});let mut s=RunSession::from_m24_diagnostic(config(boss),start).unwrap();let d=decision(&s);assert_eq!((d.act,d.room.as_ref().unwrap().kind),(3,RoomViewKind::Map));let map=s.run.as_ref().unwrap().public_map().unwrap();assert_eq!((map.act,map.boss),(3,Some(boss.encounter())));
   let mv=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();internal_replay(&s,start);dispatch(&mut s,mv).unwrap();internal_replay(&s,start);}
  // Supplied starts (labelled supplied): traversal, boss and fixture enter real owners through the same session hook.
  for boss in B::ALL{for start in [M24DiagnosticStart::Traversal(boss),M24DiagnosticStart::Boss(boss,[true;3],7),M24DiagnosticStart::Boss(boss,[false;3],11)]{let mut s=RunSession::from_m24_diagnostic(config(boss),start).unwrap();assert!(s.fault.is_none());let d=decision(&s);let a=d.candidates[0].action.clone();dispatch(&mut s,a).unwrap();internal_replay(&s,start);}}
  for f in spire_core::run::diagnostics::M24PlayedFixture::ALL{let start=M24DiagnosticStart::Played(f);let mut s=RunSession::from_m24_diagnostic(cfg.clone(),start).unwrap();let mv=decision(&s).candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();dispatch(&mut s,mv).unwrap();assert!(decision(&s).combat.is_some(),"{}",f.name());internal_replay(&s,start);}
  // Diagnostic descriptors are distinct from the normal M24 fingerprint and from each other.
  let a=RunSession::from_m24_diagnostic(cfg.clone(),stage(D::Falling)).unwrap().observation_spec().unwrap().fingerprint();let b=RunSession::from_m24_diagnostic(cfg.clone(),stage(D::Chest)).unwrap().observation_spec().unwrap().fingerprint();
  assert_ne!(a,crate::m24_run_observation_spec(&RunCapacities::m10_default()).unwrap().fingerprint());assert_ne!(a,b);
 }
 #[test]
 fn m24_delivery_fault_record_with_actual_core_faults(){
  // Ending source changes after an accepted dialog choice: the core fault (checkpoint run.m24.act3.end) is retained once.
  let start=stage(D::EndingSourceFault);let mut s=RunSession::from_m24_diagnostic(config(B::TimeEater),start).unwrap();
  let Some(RunSessionError::Core(RunError::Faulted(f)))=s.fault.clone() else{panic!("ending source fault retained: {:?}",s.fault)};assert_eq!(f.checkpoint,"run.m24.act3.end");assert!(matches!(f.kind,RunFaultKind::Invariant("accepted Act3 ending source changed")));
  let rec=s.record().unwrap().unwrap();let rf=rec.fault.as_ref().unwrap();assert!(!rf.committed&&!rf.gameplay_recorded&&rf.action.is_none());assert_eq!(rf.error.kind,RunRecordErrorKind::Core);assert_eq!(rf.error.message,"run.m24.act3.end: accepted Act3 ending source changed");assert!(rf.error.diagnostic_json().is_ok());
  let before=s.reproduction().unwrap();assert!(s.reset(Some(7)).is_err());assert!(s.reset(None).is_err());assert!(s.decision().is_err());s.fail_delivery("later").unwrap();assert_eq!(s.reproduction().unwrap(),before,"no second fault, reheal or regeneration");
  assert!(crate::run_replay::replay_m24_diagnostic(&rec,config(B::TimeEater).capacities,start).is_err(),"a faulted initial decision is not a replayable start");s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);assert!(s.reproduction().unwrap().closed);
  // Exhausted choice ledger: Proceed rejects in preflight without any Run mutation; the first structured error is retained.
  let start=stage(D::EndingPreflightReject);let mut s=RunSession::from_m24_diagnostic(config(B::TimeEater),start).unwrap();assert!(s.fault.is_none());let run_before=s.run.clone();let d=decision(&s);assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3BossComplete);
  let token=s.decision().unwrap().token;let proceed=d.candidates[0].action.clone();let err=dispatch(&mut s,proceed.clone()).unwrap_err();assert_eq!(err,RunSessionError::Core(RunError::Unsupported("choice identity exhausted")));assert!(s.run==run_before,"no mutation");
  assert_eq!(s.fault.as_ref(),Some(&err));let rec=s.record().unwrap().unwrap();assert!(rec.steps.is_empty());let rf=rec.fault.as_ref().unwrap();assert!(!rf.committed&&!rf.gameplay_recorded);assert_eq!(rf.action.as_ref(),Some(&proceed));assert_eq!(rf.error.kind,RunRecordErrorKind::Core);
  assert_eq!(s.step(token,crate::RunActionIndex(0)),Err(err.clone()));let report=crate::run_replay::replay_m24_diagnostic(&rec,config(B::TimeEater).capacities,start).unwrap();assert_eq!(report.accepted_count,0);assert_eq!(report.reported_fault,rec.fault);
  // Capture publication fault (constructed Act3 chest): OpenChest is accepted, then its relic/reward screen exceeds the action
  // capacity; the committed prefix, serial and first structured capacity error are retained and the record stays readable.
  let start=stage(D::Chest);let probe=RunSession::from_m24_diagnostic(config(B::AwakenedOne),start).unwrap();let n0=decision(&probe).candidates.len();
  let mut grown=RunSession::from_m24_diagnostic(config(B::AwakenedOne),start).unwrap();choose(&mut grown,RoomChoiceKind::OpenChest);let n1=decision(&grown).candidates.len();assert!(n1>n0,"chest reward screen offers more actions ({n0}->{n1})");
  let mut small=config(B::AwakenedOne);small.capacities.actions=n0;let mut s=RunSession::from_m24_diagnostic(small,start).unwrap();assert!(s.fault.is_none());let serial=s.serial;
  let e=choose_err(&mut s,RoomChoiceKind::OpenChest);assert_eq!(e,RunSessionError::DeliveryFault("post-step Run encoding failed"));assert_eq!(s.fault.as_ref(),Some(&e));assert_eq!(s.serial,serial+1,"accepted action keeps its serial");
  let rec=s.record().unwrap().unwrap();assert_eq!(rec.steps.len(),1);let rf=rec.fault.as_ref().unwrap();assert!(rf.committed&&rf.gameplay_recorded);assert_eq!(rf.error.kind,RunRecordErrorKind::Capacity);assert_eq!(rf.error.capacity.as_ref().unwrap().group,"actions");assert_eq!(rf.action.as_ref(),Some(&rec.steps[0].action));
  let before=s.reproduction().unwrap();assert!(s.reset(None).is_err());assert!(s.decision().is_err());s.fail_delivery("later").unwrap();assert_eq!(s.reproduction().unwrap(),before);
  let report=crate::run_replay::replay_m24_diagnostic(&rec,s.config.capacities.clone(),start).unwrap();assert_eq!(report.accepted_count,1);assert_eq!(report.reported_fault,rec.fault);assert_eq!(report.final_decision,decision(&s));
  let mut tampered=rec.clone();tampered.fault.as_mut().unwrap().error.capacity.as_mut().unwrap().required+=1;assert!(crate::run_replay::replay_m24_diagnostic(&tampered,s.config.capacities.clone(),start).is_err(),"recorded publication error must match");
  s.close().unwrap();assert!(s.record().unwrap().unwrap().closed);
 }
 fn choose_err(s:&mut RunSession,kind:RoomChoiceKind)->RunSessionError{let d=decision(s);let id=d.room.unwrap().choices.iter().find(|c|c.kind==kind).unwrap().id;let b=s.decision().unwrap();let i=decision(s).candidates.iter().position(|c|c.action==GameAction::Choose(id)).unwrap();s.step(b.token,crate::RunActionIndex(i as i64)).unwrap_err()}
}

/// M24 supplied/played native agreement (design §11, H4): the frozen supplied traversal,
/// boss and interaction-fixture receipts are replayed action by action through the
/// test-only diagnostic initializer (same core constructor), then through the internal
/// diagnostic replay. Same recorded actions only; no chooser. HOLD until the freeze.
#[cfg(test)]
mod m24_supplied_recorded_consumers{
    use super::*;use crate::run_replay::replay_m24_diagnostic;
    use spire_core::run::diagnostics::{M24DiagnosticBoss as B,M24PlayedFixture as F};
    fn hash(b:&[u8])->String{use sha2::{Digest,Sha256};format!("{:x}",Sha256::digest(b))}
    fn read(path:&std::path::Path,sha:&str)->serde_json::Value{let b=std::fs::read(path).unwrap();assert_eq!(hash(&b),sha,"{}",path.display());serde_json::from_slice(&b).unwrap()}
    fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M24WatcherAct3{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},capacities:RunCapacities::m10_default(),record_replay:true}}
    fn boss(name:&str)->B{B::ALL.into_iter().find(|b|b.encounter().as_str()==name).unwrap()}
    fn start(series:&str,v:&serde_json::Value)->M24DiagnosticStart{let c=&v["scenario"];match series{
        "supplied-traversal"=>M24DiagnosticStart::Traversal(boss(c["supplied_traversal"].as_str().unwrap())),
        "supplied-bosses"=>{let k=c["keys"].as_array().unwrap();M24DiagnosticStart::Boss(boss(c["supplied_boss"].as_str().unwrap()),[k[0].as_bool().unwrap(),k[1].as_bool().unwrap(),k[2].as_bool().unwrap()],v["seed"].as_u64().unwrap())},
        "played-fixtures"=>M24DiagnosticStart::Played(F::ALL.into_iter().find(|f|f.name()==c["fixture"].as_str().unwrap()).unwrap()),
        other=>panic!("foreign series {other}")}}
    fn snap(s:&RunSession)->(String,String){let r=s.reproduction().unwrap();(r.latest_decision.as_ref().map(|d|match d{Ok(d)=>format!("{d:?}"),Err(e)=>format!("{e:?}")}).unwrap_or_default(),format!("{:?}",r.latest_map))}
    fn apply(s:&mut RunSession,action_debug:&str,crosscheck:Option<u64>)->Result<RunStepResult,RunSessionError>{
        let r=s.reproduction().unwrap();let d=r.latest_decision.as_ref().unwrap().as_ref().unwrap();
        let i=d.candidates.iter().position(|c|format!("{:?}",c.action)==action_debug).expect("recorded action offered");if let Some(x)=crosscheck{assert_eq!(x,i as u64,"recorded candidate order");}
        let token=s.decision().unwrap().token;s.step(token,RunActionIndex(i as i64))
    }
    fn consume(series:&str,v:&serde_json::Value){
        let st=start(series,v);let mut s=RunSession::from_m24_diagnostic(config(),st).unwrap();
        let init=&v["provenance"]["initial"];
        if let Some(entry)=init.get("supplied_entry").filter(|e|!e.is_null()){assert_eq!(snap(&s).0,init["decision_debug"].as_str().unwrap(),"initial public state");
            let a=s.reproduction().unwrap().latest_decision.unwrap().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;
            apply(&mut s,&format!("{a:?}"),None).unwrap();assert_eq!(format!("Ok({})",snap(&s).0),entry["post_public_debug"].as_str().unwrap(),"bootstrap post state");}
        else{assert_eq!(snap(&s).0,init["decision_debug"].as_str().unwrap(),"initial public state");}
        for row in v["steps"].as_array().unwrap(){
            assert_eq!(snap(&s).0,row["pre"]["decision_debug"].as_str().unwrap(),"pre {}",row["index"]);
            let out=apply(&mut s,row["action_debug"].as_str().unwrap(),row["candidate_index"].as_u64());
            if let Ok(o)=&out{assert_eq!(i64::from(o.reward),row["transition"]["reward"].as_i64().unwrap());}
            if let Some(post)=row["post"]["decision_debug"].as_str(){assert_eq!(snap(&s).0,post,"post {}",row["index"]);}
        }
        let end=&v["endpoint"];if let Some(d)=end["decision_debug"].as_str(){assert_eq!(snap(&s).0,d,"endpoint");}assert_eq!(snap(&s).1,end["map_debug"].as_str().unwrap(),"endpoint map");
        let record=s.record().unwrap().unwrap();let report=replay_m24_diagnostic(&record,RunCapacities::m10_default(),st).unwrap();
        assert_eq!(report.accepted_count,record.steps.len());assert_eq!(format!("{:?}",report.final_decision),snap(&s).0,"internal replay final state");
    }
    /// Pre-freeze consumer validation on a labelled non-outcome input: a receipt-shaped
    /// value is built here from three first-candidate core steps (not a policy, not an
    /// outcome) for each start kind, then consumed exactly like a released receipt.
    #[test]
    fn m24_supplied_consumer_validates_on_labelled_non_outcome_input(){
        for (series,scenario,seed) in [("supplied-traversal",serde_json::json!({"supplied_traversal":B::AwakenedOne.encounter().as_str()}),7u64),("supplied-bosses",serde_json::json!({"supplied_boss":B::TimeEater.encounter().as_str(),"keys":[false,false,false]}),11),("played-fixtures",serde_json::json!({"fixture":"implant-writhing-mass"}),7)]{
            let mut v=serde_json::json!({"seed":seed,"scenario":scenario,"provenance":{"kind":"pre-freeze-consumer-validation (non-outcome)"}});
            let mut run=start(series,&v).run().unwrap();let dbg=|r:&spire_core::run::Run|format!("{:?}",r.decision().unwrap());
            let mut initial=serde_json::json!({"decision_debug":dbg(&run)});
            if series!="supplied-traversal"{let a=run.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;run.step(&a).unwrap();initial["supplied_entry"]=serde_json::json!({"post_public_debug":format!("{:?}",run.decision())});}
            let mut steps=vec![];for i in 0..3{let d=run.decision().unwrap();let pre=dbg(&run);let a=d.candidates[0].action.clone();let t=run.step(&a).unwrap();
                steps.push(serde_json::json!({"index":i,"candidate_index":0,"action_debug":format!("{a:?}"),"pre":{"decision_debug":pre},"transition":{"reward":t.reward},"post":{"decision_debug":dbg(&run)}}));}
            v["provenance"]["initial"]=initial;v["steps"]=serde_json::json!(steps);v["endpoint"]=serde_json::json!({"decision_debug":dbg(&run),"map_debug":format!("{:?}",run.public_map())});
            consume(series,&v);
        }
    }
    #[test]
    #[ignore="HOLD: requires the frozen M24 supplied/played receipts (SPIRE_M24_SUPPLIED_RELEASE=<path>, SPIRE_M24_SUPPLIED_RELEASE_SHA256=<sha>); same recorded actions only"]
    fn m24_supplied_recorded_native_agreement(){
        let path=std::path::PathBuf::from(std::env::var("SPIRE_M24_SUPPLIED_RELEASE").unwrap());let release=read(&path,&std::env::var("SPIRE_M24_SUPPLIED_RELEASE_SHA256").unwrap());
        assert_eq!(release["schema_id"],"m24-native-supplied-release-v1");let base=path.parent().unwrap();let cases=release["cases"].as_array().unwrap();assert_eq!(cases.len(),18,"3 traversal + 6 bosses + 9 fixtures");
        for c in cases{let v=read(&base.join(c["path"].as_str().unwrap()),c["sha256"].as_str().unwrap());assert_eq!(v["source_freeze_sha256"],release["freeze_manifest_sha256"]);consume(c["series"].as_str().unwrap(),&v);}
    }
}

#[cfg(test)]
mod m25_tests {
    use super::*;
    use crate::RunBatch;
    use spire_core::{
        combat::rng::Partition,
        content::Character,
        combat::CombatAction,
        run::{diagnostics::{M25Diagnostic, M25DiagnosticBoss, M25DiagnosticLoadout, M25PlayedFixture}, GameAction, RoomChoiceKind, RoomViewKind, RunTerminal},
    };

    fn config() -> RunSessionConfig {
        RunSessionConfig {
            character: Character::Watcher,
            ascension: 0,
            partition: Partition::Test,
            boss_history: RunBossHistory::M25WatcherHeart {
                act1: RunAct1BossTarget::Guardian,
                act2: RunAct2BossTarget::Champ,
                act3: RunAct3BossTarget::AwakenedOne,
            },
            capacities: RunCapacities::m10_default(),
            record_replay: true,
        }
    }

    fn latest(session: &RunSession) -> PublicDecision {
        session.reproduction().unwrap().latest_decision.unwrap().unwrap()
    }

    fn choose(session: &mut RunSession, kind: RoomChoiceKind) -> RunStepResult {
        let decision = latest(session);
        let choice = decision.room.unwrap().choices.into_iter().find(|choice| choice.kind == kind).unwrap();
        let candidate = decision.candidates.iter().position(|candidate| candidate.action == GameAction::Choose(choice.id)).unwrap();
        let token = session.decision().unwrap().token;
        session.step(token, crate::RunActionIndex(candidate as i64)).unwrap()
    }

    fn dispatch(session: &mut RunSession, action: GameAction) -> Result<RunStepResult, RunSessionError> {
        let boundary = session.decision()?;
        let decision = latest(session);
        let candidate = decision.candidates.iter().position(|candidate| candidate.action == action)
            .expect("requested M25 action is offered");
        session.step(boundary.token, crate::RunActionIndex(candidate as i64))
    }

    fn end_door(keys: [bool; 3]) -> RunSession {
        let mut session = RunSession::from_m25_diagnostic(config(), M25Diagnostic::DoorSource {
            boss: M25DiagnosticBoss::TimeEater,
            keys,
        }).unwrap();
        choose(&mut session, RoomChoiceKind::Proceed);
        for stage in 1..=3 { choose(&mut session, RoomChoiceKind::Act3Continue { stage }); }
        session
    }

    #[test]
    fn m25_lifecycle_no_cutoff_and_terminals() {
        let mut with_keys = end_door([true; 3]);
        let before_door = latest(&with_keys);
        assert_eq!(before_door.room.as_ref().unwrap().kind, RoomViewKind::Act3Ending);
        assert!(!with_keys.at_m14_cutoff, "M25 never inherits the M14 cutoff");
        assert!(with_keys.reproduction().unwrap().fault.is_none(), "constructed Act3 door boundary publishes healthily");
        assert!(with_keys.decision().unwrap().observation.action_mask.iter().any(|value| *value == 1));
        let door = choose(&mut with_keys, RoomChoiceKind::Act3Continue { stage: 4 });
        assert_eq!(door.boundary.terminal, None);
        let act4 = latest(&with_keys);
        assert_eq!((act4.act, act4.floor, act4.room.unwrap().kind), (4, 50, RoomViewKind::Map));
        assert!(with_keys.fault.is_none());

        let mut pending = RunSession::from_m25_diagnostic(config(), M25Diagnostic::HeartVictoryPending).unwrap();
        let heart = latest(&pending);
        assert_eq!(heart.room.as_ref().unwrap().kind, RoomViewKind::HeartVictory);
        assert_eq!(heart.terminal, None);
        let result = choose(&mut pending, RoomChoiceKind::Proceed);
        assert_eq!((result.reward, result.boundary.terminal, result.terminated), (110, Some(RunTerminal::HeartVictory), true));
        let terminal = latest(&pending);
        assert_eq!((terminal.floor, terminal.terminal, terminal.room.unwrap().kind), (55, Some(RunTerminal::HeartVictory), RoomViewKind::TrueVictory));
        let victory_record = pending.record().unwrap().unwrap();
        assert_eq!(victory_record.terminal, Some(RunTerminal::HeartVictory));
        assert_eq!(victory_record.steps.len(), 1);
        assert_eq!(victory_record.steps[0].reward, 110, "the final Proceed records +110 once");
        assert_eq!(pending.step(result.boundary.token, crate::RunActionIndex(0)), Err(RunSessionError::Terminal));
        assert_eq!(pending.record().unwrap().unwrap(), victory_record);

        let mut no_keys = end_door([false, true, true]);
        let no_keys_result = choose(&mut no_keys, RoomChoiceKind::Act3Continue { stage: 4 });
        assert_eq!((no_keys_result.reward, no_keys_result.boundary.terminal), (0, Some(RunTerminal::Act3NoKeys)));
        assert!(no_keys_result.terminated);
        assert_eq!(no_keys.record().unwrap().unwrap().terminal, Some(RunTerminal::Act3NoKeys));

        let mut act4_death = RunSession::from_m25_diagnostic(config(), M25Diagnostic::Act4Room { floor: 53 }).unwrap();
        assert_eq!(latest(&act4_death).room.as_ref().unwrap().kind, RoomViewKind::Combat);
        for _ in 0..32 {
            if latest(&act4_death).terminal.is_some() { break; }
            dispatch(&mut act4_death, GameAction::Combat(CombatAction::EndTurn)).unwrap();
        }
        let death = latest(&act4_death);
        assert_eq!(death.terminal, Some(RunTerminal::Death), "Act4 combat may terminate through Death");
        assert_eq!(act4_death.record().unwrap().unwrap().terminal, Some(RunTerminal::Death));

        let mut normal = RunSession::new_m25(config()).unwrap();
        let explicit = normal.reset(Some(7)).unwrap();
        let stale = normal.decision().unwrap().token;
        assert_eq!((normal.record().unwrap().unwrap().seed, normal.record().unwrap().unwrap().ordinal), (7, 0));
        let before_invalid = normal.reproduction().unwrap();
        let serial = normal.serial;
        assert_eq!(normal.step(explicit.token, crate::RunActionIndex(-1)), Err(RunSessionError::Input("action index is not legal")));
        assert_eq!(normal.reproduction().unwrap(), before_invalid, "invalid index changes nothing");
        assert_eq!(normal.serial, serial);
        let advanced = normal.step(explicit.token, crate::RunActionIndex(0)).unwrap();
        let after_advance = normal.reproduction().unwrap();
        assert_eq!(normal.step(stale, crate::RunActionIndex(0)), Err(RunSessionError::StaleToken));
        assert_eq!(normal.reproduction().unwrap(), after_advance, "stale token changes nothing");
        assert_eq!(normal.serial, serial + 1);
        let implicit = normal.reset(None).unwrap();
        assert_ne!(implicit.token, advanced.boundary.token);
        let record = normal.record().unwrap().unwrap();
        assert_eq!((record.seed, record.ordinal), (7, 1));
        assert_eq!(RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(), record);
        assert_eq!(normal.reproduction().unwrap().ordinal, 1);
        normal.close().unwrap();
        assert!(normal.record().unwrap().unwrap().closed);
        assert!(normal.reproduction().unwrap().closed);

        let mut sidecar = RunSession::new_m25(config()).unwrap();
        sidecar.reset(Some(7)).unwrap();
        let before_sidecar = sidecar.reproduction().unwrap();
        assert!(sidecar.environment_spec().unwrap()["combat_view_id"].is_null());
        let unsupported = RunSessionError::UnsupportedBoundary("M25 does not provide the Ironclad combat policy sidecar");
        assert_eq!(sidecar.combat_observation_spec().unwrap_err(), unsupported);
        assert_eq!(sidecar.combat_view().unwrap_err(), unsupported);
        assert_eq!(sidecar.reproduction().unwrap(), before_sidecar, "sidecar rejection preserves the owner");
    }

    #[test]
    fn m25_delivery_fault_record() {
        let pending = RunSession::from_m25_diagnostic(config(), M25Diagnostic::HeartVictoryPending).unwrap();
        assert!(pending.reproduction().unwrap().fault.is_none(), "the actual pre-fault HeartVictory boundary is publishable");
        assert_eq!(latest(&pending).room.unwrap().kind, RoomViewKind::HeartVictory);
        for (fixture, checkpoint) in [
            (M25Diagnostic::DoorFault, "run.m25.act4.transition"),
            (M25Diagnostic::HeartProceedFault, "run.m25.heart.victory"),
        ] {
            let mut session = RunSession::from_m25_diagnostic(config(), fixture).unwrap();
            let first = session.fault.clone().expect("diagnostic owner fault is retained");
            let RunSessionError::Core(RunError::Faulted(core_fault)) = &first else { panic!("structured core fault: {first:?}") };
            assert_eq!(core_fault.checkpoint, checkpoint);
            let serial = session.serial;
            let reproduction = session.reproduction().unwrap();
            assert!(matches!(
                reproduction.latest_decision,
                Some(Err(RunError::Faulted(ref fault))) if fault.checkpoint == checkpoint
            ), "the faulted core Run owner remains available");
            let record = session.record().unwrap().unwrap().clone();
            let fault = record.fault.as_ref().expect("fault record");
            assert!(!fault.committed && !fault.gameplay_recorded);
            assert_eq!(fault.error.kind, RunRecordErrorKind::Core);
            assert!(fault.error.message.contains(checkpoint));
            assert!(fault.error.diagnostic_json().is_ok());
            assert_eq!(session.reset(Some(7)), Err(first.clone()));
            assert_eq!(session.decision(), Err(first.clone()));
            assert_eq!(session.fault.as_ref(), Some(&first), "first structured error is retained");
            assert_eq!(session.serial, serial);
            assert_eq!(session.reproduction().unwrap(), reproduction);
            assert_eq!(session.record().unwrap().unwrap(), record);
            session.close().unwrap();
            let closed = session.record().unwrap().unwrap();
            assert!(closed.closed);
            assert_eq!(closed.fault, record.fault);
        }
    }

    // Release v2 (G1 ruling: Act4 coverage C from freeze v3); release v1 was 820adf82….
    const M25_RELEASE_SHA256:&str="172a818e466b6abc2954832784b54be69708e12fa7e4d73187c37c7292723603";
    const M25_ATTEMPT_SCHEMA:&str="m25-watcher-heart-caller-attempt-v1";

    struct M25RecordedCase{
        path:String,
        series:String,
        freeze:String,
        receipt:serde_json::Value,
        config:RunSessionConfig,
        seed:u64,
        accepted:usize,
        bootstrap_count:usize,
        budget:usize,
        start:Option<M25SuppliedStart>,
        bootstrap_action:Option<GameAction>,
        actions:Vec<GameAction>,
    }

    struct M25BatchLane{
        workers:usize,
        batch:RunBatch,
        boundaries:Vec<RunBoundary>,
    }

    fn m25_hex(bytes:&[u8])->String{
        use sha2::{Digest,Sha256};
        Sha256::digest(bytes).iter().map(|byte|format!("{byte:02x}")).collect()
    }

    fn m25_sha_text(value:&str)->String{m25_hex(value.as_bytes())}

    fn m25_read_unique(bytes:&[u8],context:&str)->Result<serde_json::Value,String>{
        crate::run_replay::parse_unique_json(bytes).map_err(|error|format!("{context}: {error:?}"))
    }

    fn m25_fields(value:&serde_json::Value,expected:&[&str],context:&str)->Result<(),String>{
        let Some(object)=value.as_object() else{return Err(format!("{context} is not an object"))};
        if object.len()!=expected.len()||expected.iter().any(|field|!object.contains_key(*field)){
            return Err(format!("{context} has missing or unknown fields"));
        }
        Ok(())
    }

    fn m25_text<'a>(value:&'a serde_json::Value,context:&str)->Result<&'a str,String>{
        value.as_str().ok_or_else(||format!("{context} is not a string"))
    }

    fn m25_number(value:&serde_json::Value,context:&str)->Result<u64,String>{
        value.as_u64().ok_or_else(||format!("{context} is not an unsigned integer"))
    }

    fn m25_same<T:std::fmt::Debug+PartialEq>(context:&str,actual:T,expected:T)->Result<(),String>{
        if actual==expected{Ok(())}else{Err(context.to_owned())}
    }

    fn m25_act1(name:&str)->Result<RunAct1BossTarget,String>{Ok(match name{
        "guardian"=>RunAct1BossTarget::Guardian,"hexaghost"=>RunAct1BossTarget::Hexaghost,
        "slime_boss"=>RunAct1BossTarget::SlimeBoss,other=>return Err(format!("unknown M25 Act1 boss {other}")),
    })}

    fn m25_act2(name:&str)->Result<RunAct2BossTarget,String>{Ok(match name{
        "champ"=>RunAct2BossTarget::Champ,"bronze_automaton"=>RunAct2BossTarget::BronzeAutomaton,
        "collector"=>RunAct2BossTarget::Collector,other=>return Err(format!("unknown M25 Act2 boss {other}")),
    })}

    fn m25_act3(name:&str)->Result<RunAct3BossTarget,String>{Ok(match name{
        "awakened_one"=>RunAct3BossTarget::AwakenedOne,"time_eater"=>RunAct3BossTarget::TimeEater,
        "donu_and_deca"=>RunAct3BossTarget::DonuAndDeca,other=>return Err(format!("unknown M25 Act3 boss {other}")),
    })}

    fn m25_diagnostic_boss(name:&str)->Result<M25DiagnosticBoss,String>{Ok(match name{
        "awakened_one"=>M25DiagnosticBoss::AwakenedOne,"time_eater"=>M25DiagnosticBoss::TimeEater,
        "donu_and_deca"=>M25DiagnosticBoss::DonuAndDeca,other=>return Err(format!("unknown M25 diagnostic boss {other}")),
    })}

    fn m25_config(act1:&str,act2:&str,act3:&str)->Result<RunSessionConfig,String>{Ok(RunSessionConfig{
        character:Character::Watcher,ascension:0,partition:Partition::Test,
        boss_history:RunBossHistory::M25WatcherHeart{act1:m25_act1(act1)?,act2:m25_act2(act2)?,act3:m25_act3(act3)?},
        capacities:RunCapacities::m10_default(),record_replay:true,
    })}

    fn m25_config_and_start(series:&str,scenario:&serde_json::Value)->Result<(RunSessionConfig,Option<M25SuppliedStart>),String>{
        match series{
            "normal"|"normal-control"=>Ok((m25_config(
                m25_text(&scenario["act1_boss"],"scenario.act1_boss")?,
                m25_text(&scenario["act2_boss"],"scenario.act2_boss")?,
                m25_text(&scenario["act3_boss"],"scenario.act3_boss")?,
            )?,None)),
            "supplied-traversal"=>{
                let boss=m25_diagnostic_boss(m25_text(&scenario["supplied_traversal"],"scenario.supplied_traversal")?)?;
                let act2=m25_text(&scenario["act2_boss_source"],"scenario.act2_boss_source")?;
                let act3=m25_text(&scenario["supplied_traversal"],"scenario.supplied_traversal")?;
                Ok((m25_config("guardian",act2,act3)?,Some(M25SuppliedStart::Traversal(boss))))
            }
            "supplied-act4"=>{
                let boss=m25_text(&scenario["act3_source_boss"],"scenario.act3_source_boss")?;
                let loadout=match m25_text(&scenario["supplied_act4_loadout"],"scenario.supplied_act4_loadout")?{
                    "coverage-a"=>M25DiagnosticLoadout::CoverageA,"coverage-b"=>M25DiagnosticLoadout::CoverageB,
                    "coverage-c"=>M25DiagnosticLoadout::CoverageC,"strong-v1"=>M25DiagnosticLoadout::Strong,
                    other=>return Err(format!("unknown M25 supplied Act4 loadout {other}")),
                };
                Ok((m25_config("guardian","champ",boss)?,Some(M25SuppliedStart::Act4(loadout))))
            }
            "played-fixture"=>{
                let name=m25_text(&scenario["fixture"],"scenario.fixture")?;
                let fixture=M25PlayedFixture::ALL.into_iter().find(|fixture|fixture.name()==name)
                    .ok_or_else(||format!("unknown M25 played fixture {name}"))?;
                Ok((m25_config("guardian","champ","awakened_one")?,Some(M25SuppliedStart::PlayedFixture(fixture))))
            }
            other=>Err(format!("unknown M25 release series {other}")),
        }
    }

    /// Decode caller semantic actions only through the strict versioned M25 Run record codec.
    /// The caller's candidate_index field is deliberately not read or used.
    fn m25_decode_semantic_actions(config:&RunSessionConfig,seed:u64,values:&[serde_json::Value])->Result<Vec<GameAction>,String>{
        let mut template=RunSession::new_m25(config.clone()).map_err(|error|format!("M25 action codec session: {error:?}"))?;
        template.reset(Some(seed)).map_err(|error|format!("M25 action codec reset: {error:?}"))?;
        let record=template.record().map_err(|error|format!("M25 action codec record: {error:?}"))?
            .ok_or_else(||"M25 action codec recording disabled".to_owned())?;
        let bytes=record.to_json().map_err(|error|format!("M25 action codec export: {error:?}"))?;
        let mut wire=m25_read_unique(&bytes,"M25 action codec template")?;
        wire["steps"]=serde_json::Value::Array(values.iter().map(|action|serde_json::json!({
            "action":action,"reward":0,"terminal":null,
        })).collect());
        let wire_bytes=serde_json::to_vec(&wire).map_err(|error|error.to_string())?;
        let imported=match RunEpisodeRecord::from_json(&wire_bytes){
            Ok(imported)=>imported,
            Err(error)=>{
                for (index,action) in values.iter().enumerate(){
                    let mut one=wire.clone();
                    one["steps"]=serde_json::json!([{"action":action,"reward":0,"terminal":null}]);
                    let bytes=serde_json::to_vec(&one).map_err(|error|error.to_string())?;
                    if let Err(codec_error)=RunEpisodeRecord::from_json(&bytes){
                        return Err(format!("strict M25 semantic action decode receipt_step={index} action_json={action} codec={codec_error:?}"));
                    }
                }
                return Err(format!("strict M25 semantic action decode: {error:?}"));
            }
        };
        let exported=m25_read_unique(&imported.to_json().map_err(|error|format!("M25 action codec roundtrip export: {error:?}"))?,"M25 action codec roundtrip")?;
        let exported_steps=exported["steps"].as_array().ok_or_else(||"strict M25 action codec lost steps".to_owned())?;
        if exported_steps.len()!=values.len(){return Err("strict M25 action codec changed step count".into())}
        for (index,(step,source)) in exported_steps.iter().zip(values).enumerate(){
            if step["action"]!=*source{return Err(format!("strict M25 semantic action roundtrip differs at {index}"))}
        }
        Ok(imported.steps.into_iter().map(|step|step.action).collect())
    }

    fn m25_load_recorded_cases()->Result<Vec<M25RecordedCase>,String>{
        use std::path::{Component,Path,PathBuf};
        let release_path=PathBuf::from(std::env::var_os("SPIRE_M25_RECORDED_RELEASE")
            .ok_or_else(||"HOLD: SPIRE_M25_RECORDED_RELEASE is required".to_owned())?);
        let expected_sha=std::env::var("SPIRE_M25_RECORDED_RELEASE_SHA256")
            .map_err(|_|"HOLD: SPIRE_M25_RECORDED_RELEASE_SHA256 is required".to_owned())?;
        if expected_sha!=M25_RELEASE_SHA256{return Err("HOLD: supplied M25 release SHA is not the frozen N3 manifest".into())}
        let release_bytes=std::fs::read(&release_path).map_err(|error|format!("read M25 release manifest: {error}"))?;
        if m25_hex(&release_bytes)!=M25_RELEASE_SHA256{return Err("M25 release manifest SHA-256 mismatch".into())}
        let release=m25_read_unique(&release_bytes,"M25 release manifest")?;
        if release["schema_id"]!="m25-native-recorded-release-v1"{return Err("unsupported M25 release manifest schema".into())}
        let base=PathBuf::from(m25_text(&release["base"],"release.base")?);
        if !base.is_absolute()||base!=release_path.parent().and_then(Path::parent).ok_or_else(||"M25 release manifest parent missing".to_owned())?{
            return Err("M25 release base does not match its retained manifest location".into())
        }
        let freeze_v1=m25_text(&release["freezes"]["v1"],"release.freezes.v1")?;
        let freeze_v2=m25_text(&release["freezes"]["v2"],"release.freezes.v2")?;
        // Optional third corrective freeze (G1 coverage-C rerun); absent in release v1.
        let freeze_v3=release["freezes"].get("v3").and_then(serde_json::Value::as_str);
        let manifest_cases=release["cases"].as_array().ok_or_else(||"release.cases is not an array".to_owned())?;
        if manifest_cases.len()!=24{return Err(format!("release has {} cases, expected 24",manifest_cases.len()))}
        let mut seen_paths=std::collections::BTreeSet::new();
        let mut series_counts=std::collections::BTreeMap::<String,usize>::new();
        let mut cases=Vec::with_capacity(manifest_cases.len());
        for (case_index,entry) in manifest_cases.iter().enumerate(){
            let relative=m25_text(&entry["path"],"release case.path")?;
            let relative_path=Path::new(relative);
            if relative_path.is_absolute()||relative_path.components().any(|part|matches!(part,Component::ParentDir)){
                return Err(format!("release case {case_index} path is not root-relative"))
            }
            if !seen_paths.insert(relative.to_owned()){return Err(format!("duplicate M25 receipt path {relative}"))}
            let expected_receipt_sha=m25_text(&entry["sha256"],"release case.sha256")?;
            let freeze=m25_text(&entry["freeze_sha256"],"release case.freeze_sha256")?.to_owned();
            if freeze!=freeze_v1&&freeze!=freeze_v2&&Some(freeze.as_str())!=freeze_v3{return Err(format!("receipt {relative} names an unknown frozen source"))}
            let receipt_bytes=std::fs::read(base.join(relative)).map_err(|error|format!("read M25 receipt {relative}: {error}"))?;
            if m25_hex(&receipt_bytes)!=expected_receipt_sha{return Err(format!("M25 receipt SHA-256 mismatch: {relative}"))}
            let receipt=m25_read_unique(&receipt_bytes,&format!("M25 receipt {relative}"))?;
            if receipt["schema_id"]!=M25_ATTEMPT_SCHEMA{return Err(format!("unsupported M25 caller attempt schema: {relative}"))}
            if m25_text(&receipt["source_freeze_sha256"],"receipt.source_freeze_sha256")?!=freeze{return Err(format!("M25 receipt source freeze mismatch: {relative}"))}
            let series=m25_text(&entry["series"],"release case.series")?.to_owned();
            *series_counts.entry(series.clone()).or_default()+=1;
            let seed=m25_number(&receipt["seed"],"receipt.seed")?;
            if seed!=m25_number(&entry["seed"],"release case.seed")?{return Err(format!("M25 receipt seed mismatch: {relative}"))}
            if receipt["scenario"]!=entry["case"]{return Err(format!("M25 receipt scenario differs from release case: {relative}"))}
            if receipt["outcome"]!=entry["outcome"]||receipt["error"]!=entry["error"]{return Err(format!("M25 receipt outcome/error differs from release: {relative}"))}
            if m25_text(&receipt["provenance"]["kind"],"receipt.provenance.kind")?!=m25_text(&entry["label"],"release case.label")?{
                return Err(format!("M25 receipt label differs from release: {relative}"))
            }
            let accepted=usize::try_from(m25_number(&receipt["accepted_action_count"],"receipt.accepted_action_count")?).map_err(|_|"M25 accepted count exceeds usize")?;
            let bootstrap_count=usize::try_from(m25_number(&receipt["bootstrap_accepted_count"],"receipt.bootstrap_accepted_count")?).map_err(|_|"M25 bootstrap count exceeds usize")?;
            let budget=usize::try_from(m25_number(&receipt["action_budget"],"receipt.action_budget")?).map_err(|_|"M25 action budget exceeds usize")?;
            let rows=receipt["steps"].as_array().ok_or_else(||format!("M25 receipt steps is not an array: {relative}"))?;
            if accepted!=rows.len()||accepted!=m25_number(&entry["accepted_action_count"],"release case.accepted_action_count")? as usize
                ||accepted!=m25_number(&entry["steps"],"release case.steps")? as usize{
                return Err(format!("M25 receipt accepted-action count/rows differ from release: {relative}"))
            }
            if bootstrap_count!=m25_number(&entry["bootstrap_accepted_count"],"release case.bootstrap_accepted_count")? as usize
                ||m25_number(&receipt["policy_accepted_count"],"receipt.policy_accepted_count")? as usize!=accepted
                ||m25_number(&receipt["policy_successful_count"],"receipt.policy_successful_count")? as usize!=accepted
                ||m25_number(&receipt["policy_committed_fault_count"],"receipt.policy_committed_fault_count")?!=0
                ||m25_number(&receipt["total_actual_accepted_run_actions"],"receipt.total_actual_accepted_run_actions")? as usize!=accepted+bootstrap_count{
                return Err(format!("M25 receipt action accounting differs from release: {relative}"))
            }
            let scenario=&receipt["scenario"];
            let (config,start)=m25_config_and_start(&series,scenario)?;
            let semantic_values=rows.iter().map(|row|row["action"].clone()).collect::<Vec<_>>();
            let actions=m25_decode_semantic_actions(&config,seed,&semantic_values)
                .map_err(|error|format!("receipt={relative}: {error}"))?;
            for (step_index,(row,action)) in rows.iter().zip(&actions).enumerate(){
                m25_fields(row,&["action","action_debug","candidate_index","index","observed","post","pre","transition"],&format!("{relative} steps[{step_index}]"))?;
                if m25_number(&row["index"],"step.index")? as usize!=step_index{return Err(format!("M25 receipt step ordinal differs: {relative} step {step_index}"))}
                if m25_text(&row["action_debug"],"step.action_debug")?!=format!("{action:?}"){
                    return Err(format!("strict decoded semantic action differs from action_debug: {relative} step {step_index}"))
                }
                m25_fields(&row["pre"],&["decision_debug","map_debug"],"step.pre")?;
                m25_fields(&row["transition"],&["committed_events_debug","debug","reward","terminal"],"step.transition")?;
                m25_fields(&row["observed"],&["boundaries_debug","effects_debug"],"step.observed")?;
                m25_fields(&row["post"],&["decision_debug","decision_error","map_debug"],"step.post")?;
            }
            m25_fields(&receipt["endpoint"],&["combat_checkpoint_debug","decision_debug","decision_error","map_debug"],"receipt.endpoint")?;
            let bootstrap_action=if series=="played-fixture"{
                let value=&receipt["provenance"]["initial"]["supplied_entry"]["action"];
                Some(m25_decode_semantic_actions(&config,seed,std::slice::from_ref(value))?.remove(0))
            }else{None};
            if (series=="played-fixture")!=(bootstrap_action.is_some())||bootstrap_count!=usize::from(bootstrap_action.is_some()){
                return Err(format!("M25 bootstrap provenance/count mismatch: {relative}"))
            }
            cases.push(M25RecordedCase{path:relative.to_owned(),series,freeze,receipt,config,seed,accepted,bootstrap_count,budget,start,bootstrap_action,actions});
        }
        for (series,expected) in [("normal",6usize),("normal-control",3),("supplied-traversal",3),("supplied-act4",4),("played-fixture",8)]{
            if series_counts.get(series).copied().unwrap_or(0)!=expected{return Err(format!("release {series} count is not {expected}"))}
        }
        Ok(cases)
    }

    fn m25_compare_initial_run(case:&M25RecordedCase,run:&Run)->Result<(),String>{
        let initial=&case.receipt["provenance"]["initial"];
        let decision=run.decision().map_err(|error|format!("{} initial core decision: {error:?}",case.path))?;
        let map=run.public_map();
        m25_same("initial decision",format!("{decision:?}"),m25_text(&initial["decision_debug"],"initial.decision_debug")?.to_owned())?;
        m25_same("initial map",format!("{map:?}"),m25_text(&initial["map_debug"],"initial.map_debug")?.to_owned())?;
        m25_same("initial combat checkpoint",format!("{:?}",run.combat_checkpoint()),m25_text(&initial["combat_checkpoint_debug"],"initial.combat_checkpoint_debug")?.to_owned())?;
        m25_same("initial public fingerprint",m25_sha_text(&format!("{decision:?}\n{map:?}")),m25_text(&initial["public_sha256"],"initial.public_sha256")?.to_owned())?;
        for key in ["diagnostic_checkpoint","diagnostic_checkpoint_before_bootstrap"]{
            if let Some(expected)=initial.get(key).and_then(serde_json::Value::as_str){
                let checkpoint=spire_core::run::diagnostics::m22_diagnostic_checkpoint(run)
                    .map_err(|error|format!("{} M25 diagnostic checkpoint: {error:?}",case.path))?;
                m25_same(key,format!("{checkpoint:?}"),expected.to_owned())?;
            }
        }
        Ok(())
    }

    fn m25_compare_bootstrap(case:&M25RecordedCase,run:&mut Run,action:&GameAction)->Result<(),String>{
        let entry=&case.receipt["provenance"]["initial"]["supplied_entry"];
        m25_same("bootstrap scope",m25_text(&entry["scope"],"supplied_entry.scope")?.to_owned(),"supplied encounter bootstrap; excluded from policy budget/count".to_owned())?;
        m25_same("bootstrap semantic action",action.clone(),case.bootstrap_action.clone().ok_or_else(||"played fixture bootstrap action missing".to_owned())?)?;
        let before=run.decision().map_err(|error|format!("{} bootstrap pre-decision: {error:?}",case.path))?;
        if !before.candidates.iter().any(|candidate|candidate.action==*action){return Err("bootstrap semantic action is not currently offered".into())}
        let observed=run.step_observed(action);
        let successful=observed.result.is_ok();
        let accepted=successful||matches!(&observed.result,Err(spire_core::run::RunError::Faulted(_)));
        let first_move=before.candidates.iter().find(|candidate|matches!(candidate.action,GameAction::Move(_))).map(|candidate|candidate.action.clone());
        m25_same("bootstrap is the recorded fixed Move",first_move,Some(action.clone()))?;
        m25_same("bootstrap accepted",accepted,entry["accepted"].as_bool().unwrap_or(false))?;
        m25_same("bootstrap successful",successful,entry["successful"].as_bool().unwrap_or(false))?;
        m25_same("bootstrap transition",format!("{:?}",observed.result),m25_text(&entry["transition_debug"],"supplied_entry.transition_debug")?.to_owned())?;
        m25_same("bootstrap effects",format!("{:?}",observed.effects),m25_text(&entry["effects_debug"],"supplied_entry.effects_debug")?.to_owned())?;
        m25_same("bootstrap combat boundaries",format!("{:?}",observed.boundaries),m25_text(&entry["boundaries_debug"],"supplied_entry.boundaries_debug")?.to_owned())?;
        let checkpoint=spire_core::run::diagnostics::m22_diagnostic_checkpoint(run)
            .map_err(|error|format!("{} bootstrap diagnostic checkpoint: {error:?}",case.path))?;
        m25_same("bootstrap post diagnostic checkpoint",format!("Ok({checkpoint:?})"),m25_text(&entry["post_diagnostic_checkpoint_debug"],"supplied_entry.post_diagnostic_checkpoint_debug")?.to_owned())?;
        m25_same("bootstrap post public decision",format!("{:?}",run.decision()),m25_text(&entry["post_public_debug"],"supplied_entry.post_public_debug")?.to_owned())?;
        m25_same("bootstrap post map",format!("{:?}",run.public_map()),m25_text(&entry["map_debug"],"supplied_entry.map_debug")?.to_owned())?;
        Ok(())
    }

    fn m25_action_index(decision:&spire_core::run::PublicDecision,action:&GameAction,context:&str)->Result<usize,String>{
        let matches=decision.candidates.iter().enumerate().filter_map(|(index,candidate)|(&candidate.action==action).then_some(index)).collect::<Vec<_>>();
        if matches.len()!=1{return Err(format!("{context}: semantic action has {} matching offered candidates",matches.len()))}
        Ok(matches[0])
    }

    fn m25_compare_decision_fields(
        result:&Result<spire_core::run::PublicDecision,spire_core::run::RunError>,
        map:Option<&spire_core::run::map::PublicMap>,
        debug:&serde_json::Value,
        error:&serde_json::Value,
        map_debug:&serde_json::Value,
        context:&str,
    )->Result<(),String>{
        match result{
            Ok(decision)=>{
                m25_same(&format!("{context} decision"),serde_json::Value::String(format!("{decision:?}")),debug.clone())?;
                m25_same(&format!("{context} decision error"),error.clone(),serde_json::Value::Null)?;
            }
            Err(actual)=>{
                m25_same(&format!("{context} decision"),debug.clone(),serde_json::Value::Null)?;
                m25_same(&format!("{context} decision error"),serde_json::Value::String(format!("{actual:?}")),error.clone())?;
            }
        }
        m25_same(&format!("{context} map"),serde_json::Value::String(format!("{map:?}")),map_debug.clone())
    }

    fn m25_compare_run_row(case:&M25RecordedCase,run:&Run,row:&serde_json::Value,context:&str,post:bool)->Result<(),String>{
        let decision=run.decision();let map=run.public_map();
        if post{
            m25_compare_decision_fields(&decision,map.as_ref(),&row["decision_debug"],&row["decision_error"],&row["map_debug"],context)
        }else{
            let decision=decision.map_err(|error|format!("{} {context} expected readable decision: {error:?}",case.path))?;
            m25_same(&format!("{context} decision"),format!("{decision:?}"),m25_text(&row["decision_debug"],"decision_debug")?.to_owned())?;
            m25_same(&format!("{context} map"),format!("{map:?}"),m25_text(&row["map_debug"],"map_debug")?.to_owned())
        }
    }

    fn m25_compare_session_boundary(case:&M25RecordedCase,session:&RunSession,context:&str)->Result<(),String>{
        let run=session.run.as_ref().ok_or_else(||format!("{context}: session has no owned Run"))?;
        let decision=run.decision().map_err(|error|format!("{context}: live Run decision failed: {error:?}"))?;
        let map=run.public_map();
        let expected=crate::encode_run_m25(&decision,map.as_ref(),&case.config.capacities)
            .map_err(|error|format!("{context}: M25 boundary encoding failed: {error:?}"))?;
        let boundary=session.decision().map_err(|error|format!("{context}: session boundary failed: {error:?}"))?;
        m25_same(&format!("{context} encoded observation"),boundary.observation,expected)?;
        m25_same(&format!("{context} terminal"),boundary.terminal,decision.terminal)
    }

    fn m25_new_single(case:&M25RecordedCase)->Result<RunSession,String>{
        if let Some(start)=case.start{
            RunSession::from_m25_supplied(case.config.clone(),start).map_err(|error|format!("{} supplied native session: {error:?}",case.path))
        }else{
            let mut session=RunSession::new_m25(case.config.clone()).map_err(|error|format!("{} normal native session: {error:?}",case.path))?;
            session.reset(Some(case.seed)).map_err(|error|format!("{} normal native reset: {error:?}",case.path))?;
            Ok(session)
        }
    }

    fn m25_new_batch(case:&M25RecordedCase,workers:usize)->Result<M25BatchLane,String>{
        let configs=vec![case.config.clone(),case.config.clone()];
        let mut batch=if let Some(start)=case.start{
            RunBatch::from_m25_supplied(configs,workers,start).map_err(|error|format!("{} supplied {workers}-worker batch: {error:?}",case.path))?
        }else{
            RunBatch::new_m25(configs,workers).map_err(|error|format!("{} normal {workers}-worker batch: {error:?}",case.path))?
        };
        let boundaries=if case.start.is_some(){batch.m25_test_boundaries()}else{
            batch.reset(&[Some(case.seed),Some(case.seed)]).map_err(|error|format!("{} {workers}-worker batch reset: {error:?}",case.path))?
        };
        Ok(M25BatchLane{workers,batch,boundaries})
    }

    fn m25_compare_lane_initial(case:&M25RecordedCase,session:&RunSession,lane:&M25BatchLane)->Result<(),String>{
        let expected=session.reproduction().map_err(|error|format!("{} single initial reproduction: {error:?}",case.path))?;
        for slot in 0..2{
            let actual=lane.batch.reproduction(slot).map_err(|error|format!("{} batch initial reproduction: {error:?}",case.path))?;
            m25_same(&format!("{} workers {} initial slot {slot} reproduction",case.path,lane.workers),actual,expected.clone())?;
            let record=lane.batch.record(slot).map_err(|error|format!("{} batch initial record: {error:?}",case.path))?;
            m25_same(&format!("{} workers {} initial slot {slot} record",case.path,lane.workers),record,expected.record.clone())?;
            let direct=lane.boundaries.get(slot).ok_or_else(||"M25 batch initial boundary missing".to_owned())?;
            let single_boundary=session.decision().map_err(|error|format!("{} single initial boundary: {error:?}",case.path))?;
            m25_same(&format!("{} workers {} initial slot {slot} observation",case.path,lane.workers),direct.observation.clone(),single_boundary.observation)?;
            m25_same(&format!("{} workers {} initial slot {slot} terminal",case.path,lane.workers),direct.terminal,single_boundary.terminal)?;
        }
        Ok(())
    }

    fn m25_compare_endpoint(case:&M25RecordedCase,run:&Run)->Result<(),String>{
        let endpoint=&case.receipt["endpoint"];
        m25_compare_decision_fields(&run.decision(),run.public_map().as_ref(),&endpoint["decision_debug"],&endpoint["decision_error"],&endpoint["map_debug"],"endpoint")?;
        m25_same("endpoint combat checkpoint",format!("{:?}",run.combat_checkpoint()),m25_text(&endpoint["combat_checkpoint_debug"],"endpoint.combat_checkpoint_debug")?.to_owned())?;
        let decision=run.decision().map_err(|error|format!("{} endpoint decision: {error:?}",case.path))?;
        match m25_text(&case.receipt["outcome"],"receipt.outcome")?{
            "Death"=>m25_same("final outcome",decision.terminal,Some(RunTerminal::Death))?,
            "EmptyDeck"=>m25_same("final outcome",decision.terminal,Some(RunTerminal::EmptyDeck))?,
            "Act3NoKeys"=>m25_same("final outcome",decision.terminal,Some(RunTerminal::Act3NoKeys))?,
            "HeartVictory"=>m25_same("final outcome",decision.terminal,Some(RunTerminal::HeartVictory))?,
            "BudgetExhausted"=>{
                m25_same("budget outcome remains nonterminal",decision.terminal,None)?;
                m25_same("budget outcome action count",case.accepted,case.budget)?;
            }
            other=>return Err(format!("unknown frozen M25 outcome {other}")),
        }
        m25_same("final terminal field",case.receipt["error"].is_null(),true)
    }

    fn m25_consume_recorded_case(case:&M25RecordedCase)->Result<(),String>{
        let rows=case.receipt["steps"].as_array().ok_or_else(||"receipt steps missing".to_owned())?;
        m25_same("recorded action vector count",case.actions.len(),case.accepted)?;
        let mut direct_bootstrap=None;
        if let Some(start)=case.start{
            let mut direct=start.run().map_err(|error|format!("{} supplied initial core run: {error:?}",case.path))?;
            m25_compare_initial_run(case,&direct)?;
            if let Some(action)=&case.bootstrap_action{
                if !matches!(start,M25SuppliedStart::PlayedFixture(_)){return Err("bootstrap action occurs outside played-fixture series".into())}
                m25_compare_bootstrap(case,&mut direct,action)?;
                direct_bootstrap=Some(direct);
            }else if matches!(start,M25SuppliedStart::PlayedFixture(_)){
                return Err("played-fixture bootstrap action missing".into())
            }
            if let Some(post_bootstrap)=&direct_bootstrap{
                let session=m25_new_single(case)?;
                let actual=session.run.as_ref().ok_or_else(||"played-fixture session has no Run".to_owned())?;
                m25_same("played-fixture bootstrap initial decision",format!("{:?}",actual.decision()),format!("{:?}",post_bootstrap.decision()))?;
                m25_same("played-fixture bootstrap initial map",format!("{:?}",actual.public_map()),format!("{:?}",post_bootstrap.public_map()))?;
                m25_same("played-fixture bootstrap is outside policy record",session.record().map_err(|error|format!("record: {error:?}"))?.unwrap().steps.len(),0usize)?;
            }
        }

        let mut single=m25_new_single(case)?;
        let mut replay=m25_new_single(case)?;
        m25_compare_session_boundary(case,&single,"single initial")?;
        let initial_record=single.record().map_err(|error|format!("{} initial record: {error:?}",case.path))?.ok_or_else(||"M25 session did not record".to_owned())?;
        m25_same("single initial policy record count",initial_record.steps.len(),0usize)?;
        m25_same("same-initializer replay initial reproduction",replay.reproduction().map_err(|error|format!("replay initial: {error:?}"))?,single.reproduction().map_err(|error|format!("single initial: {error:?}"))?)?;
        if case.start.is_none(){
            let first=single.run.as_ref().ok_or_else(||"normal session has no Run".to_owned())?;
            m25_compare_initial_run(case,first)?;
        }else if case.bootstrap_action.is_none(){
            m25_compare_initial_run(case,single.run.as_ref().ok_or_else(||"supplied session has no Run".to_owned())?)?;
        }else{
            let entry=&case.receipt["provenance"]["initial"]["supplied_entry"];
            let first=rows.first().ok_or_else(||"played fixture has no policy actions".to_owned())?;
            m25_compare_run_row(case,single.run.as_ref().ok_or_else(||"played-fixture session has no Run".to_owned())?,&first["pre"],"played-fixture post-bootstrap policy pre",false)?;
            m25_same("played-fixture policy starts after bootstrap",format!("{:?}",single.run.as_ref().unwrap().decision()),m25_text(&entry["post_public_debug"],"supplied_entry.post_public_debug")?.to_owned())?;
        }
        let mut lanes=Vec::new();
        for workers in [1,2]{
            let lane=m25_new_batch(case,workers)?;
            m25_compare_lane_initial(case,&single,&lane)?;
            lanes.push(lane);
        }

        for (step_index,(row,action)) in rows.iter().zip(&case.actions).enumerate(){
            let context=format!("step {step_index}");
            m25_compare_run_row(case,single.run.as_ref().ok_or_else(||"single session has no Run".to_owned())?,&row["pre"],&format!("{context} pre"),false)?;
            m25_compare_run_row(case,replay.run.as_ref().ok_or_else(||"replay session has no Run".to_owned())?,&row["pre"],&format!("{context} replay pre"),false)?;
            let pre=single.reproduction().map_err(|error|format!("{context} single pre reproduction: {error:?}"))?;
            for lane in &lanes{
                for slot in 0..2{
                    let actual=lane.batch.reproduction(slot).map_err(|error|format!("{context} workers {} pre reproduction: {error:?}",lane.workers))?;
                    m25_same(&format!("{context} workers {} slot {slot} pre reproduction",lane.workers),actual,pre.clone())?;
                }
            }
            let current=single.run.as_ref().unwrap().decision().map_err(|error|format!("{context} decision: {error:?}"))?;
            let action_index=m25_action_index(&current,action,&context)?;
            let mut observed_run=single.run.as_ref().unwrap().clone();
            let observed=observed_run.step_observed(action);
            let transition=observed.result.as_ref().map_err(|error|format!("{context} receipt action failed in shared Run: {error:?}"))?;
            m25_same(&format!("{context} transition debug"),format!("{transition:?}"),m25_text(&row["transition"]["debug"],"transition.debug")?.to_owned())?;
            m25_same(&format!("{context} reward"),transition.reward,row["transition"]["reward"].as_i64().ok_or_else(||"transition.reward is not signed".to_owned())? as i32)?;
            m25_same(&format!("{context} terminal"),format!("{:?}",transition.terminal),m25_text(&row["transition"]["terminal"],"transition.terminal")?.to_owned())?;
            let event_debug=transition.committed_events.iter().map(|event|format!("{event:?}")).collect::<Vec<_>>();
            let receipt_event_debug=row["transition"]["committed_events_debug"].as_array().ok_or_else(||"committed event debug is not an array".to_owned())?.iter().map(|event|event.as_str().unwrap_or("<invalid>").to_owned()).collect::<Vec<_>>();
            m25_same(&format!("{context} committed combat/run events"),event_debug,receipt_event_debug)?;
            m25_same(&format!("{context} observed effects"),format!("{:?}",Some(&observed.effects)),m25_text(&row["observed"]["effects_debug"],"observed.effects_debug")?.to_owned())?;
            m25_same(&format!("{context} observed combat boundaries"),format!("{:?}",Some(&observed.boundaries)),m25_text(&row["observed"]["boundaries_debug"],"observed.boundaries_debug")?.to_owned())?;

            let token=single.decision().map_err(|error|format!("{context} single boundary: {error:?}"))?.token;
            let single_result=single.step(token,RunActionIndex(action_index as i64)).map_err(|error|format!("{context} single session step: {error:?}"))?;
            m25_same(&format!("{context} native reward"),single_result.reward,transition.reward)?;
            m25_same(&format!("{context} native terminal"),single_result.boundary.terminal,transition.terminal)?;
            m25_same(&format!("{context} native terminated"),single_result.terminated,transition.terminal.is_some())?;
            m25_same(&format!("{context} native truncation"),single_result.truncated,false)?;
            let native_run=single.run.as_ref().ok_or_else(||"single session Run missing after step".to_owned())?;
            let native_decision=native_run.decision().map_err(|error|format!("{context} post decision: {error:?}"))?;
            let native_map=native_run.public_map();
            let encoded=crate::encode_run_m25(&native_decision,native_map.as_ref(),&case.config.capacities).map_err(|error|format!("{context} native observation encoding: {error:?}"))?;
            m25_same(&format!("{context} single boundary observation"),single_result.boundary.observation.clone(),encoded.clone())?;
            m25_same(&format!("{context} native Run equals observed transition"),format!("{:?}",native_run.decision()),format!("{:?}",observed_run.decision()))?;
            m25_same(&format!("{context} native map equals observed transition"),native_run.public_map(),observed_run.public_map())?;
            m25_compare_run_row(case,native_run,&row["post"],&format!("{context} post"),true)?;
            let single_record=single.record().map_err(|error|format!("{context} single record: {error:?}"))?.ok_or_else(||"single record disappeared".to_owned())?;
            m25_same(&format!("{context} recorded policy step count"),single_record.steps.len(),step_index+1)?;
            m25_same(&format!("{context} recorded semantic action"),single_record.steps.last().map(|step|step.action.clone()),Some(action.clone()))?;
            m25_same(&format!("{context} recorded reward/terminal"),single_record.steps.last().map(|step|(step.reward,step.terminal)),Some((transition.reward,transition.terminal)))?;

            let replay_decision=replay.run.as_ref().unwrap().decision().map_err(|error|format!("{context} replay pre-decision: {error:?}"))?;
            let replay_index=m25_action_index(&replay_decision,action,&format!("{context} same-initializer replay"))?;
            let replay_token=replay.decision().map_err(|error|format!("{context} replay boundary: {error:?}"))?.token;
            let replay_result=replay.step(replay_token,RunActionIndex(replay_index as i64)).map_err(|error|format!("{context} same-initializer replay step: {error:?}"))?;
            m25_same(&format!("{context} same-initializer replay reward"),replay_result.reward,transition.reward)?;
            m25_same(&format!("{context} same-initializer replay terminal"),replay_result.boundary.terminal,transition.terminal)?;
            m25_same(&format!("{context} same-initializer replay observation"),replay_result.boundary.observation,single_result.boundary.observation.clone())?;
            m25_same(&format!("{context} same-initializer replay reproduction"),replay.reproduction().map_err(|error|format!("{context} replay reproduction: {error:?}"))?,single.reproduction().map_err(|error|format!("{context} single reproduction: {error:?}"))?)?;
            m25_same(&format!("{context} same-initializer replay record"),replay.record().map_err(|error|format!("{context} replay record: {error:?}"))?,Some(single_record.clone()))?;

            for lane in &mut lanes{
                let tokens=lane.boundaries.iter().map(|boundary|boundary.token).collect::<Vec<_>>();
                let mut indices=Vec::with_capacity(2);
                for slot in 0..2{
                    let reproduction=lane.batch.reproduction(slot).map_err(|error|format!("{context} workers {} slot {slot} prestate: {error:?}",lane.workers))?;
                    let decision=reproduction.latest_decision.ok_or_else(||format!("{context} workers {} slot {slot} decision missing",lane.workers))?.map_err(|error|format!("{context} workers {} slot {slot} decision error: {error:?}",lane.workers))?;
                    indices.push(RunActionIndex(m25_action_index(&decision,action,&format!("{context} workers {} slot {slot}",lane.workers))? as i64));
                }
                let results=lane.batch.step(&tokens,&indices).map_err(|error|format!("{context} workers {} batch step: {error:?}",lane.workers))?;
                for slot in 0..2{
                    let result=&results[slot];
                    m25_same(&format!("{context} workers {} slot {slot} reward",lane.workers),result.reward,transition.reward)?;
                    m25_same(&format!("{context} workers {} slot {slot} terminal",lane.workers),result.boundary.terminal,transition.terminal)?;
                    m25_same(&format!("{context} workers {} slot {slot} observation",lane.workers),result.boundary.observation.clone(),encoded.clone())?;
                    m25_same(&format!("{context} workers {} slot {slot} reproduction",lane.workers),lane.batch.reproduction(slot).map_err(|error|format!("batch reproduction: {error:?}"))?,single.reproduction().map_err(|error|format!("single reproduction: {error:?}"))?)?;
                    m25_same(&format!("{context} workers {} slot {slot} record",lane.workers),lane.batch.record(slot).map_err(|error|format!("batch record: {error:?}"))?,Some(single_record.clone()))?;
                }
                lane.boundaries=results.into_iter().map(|result|result.boundary).collect();
            }
        }

        let run=single.run.as_ref().ok_or_else(||"single final Run missing".to_owned())?;
        m25_compare_endpoint(case,run)?;
        let record=single.record().map_err(|error|format!("{} final single record: {error:?}",case.path))?.ok_or_else(||"M25 final record missing".to_owned())?;
        m25_same("final accepted policy action count",record.steps.len(),case.accepted)?;
        m25_same("final record terminal",record.terminal,run.decision().map_err(|error|format!("final decision: {error:?}"))?.terminal)?;
        m25_same("final same-initializer replay",replay.reproduction().map_err(|error|format!("final replay reproduction: {error:?}"))?,single.reproduction().map_err(|error|format!("final single reproduction: {error:?}"))?)?;
        m25_same("final same-initializer replay record",replay.record().map_err(|error|format!("final replay record: {error:?}"))?,Some(record.clone()))?;
        for lane in &lanes{
            for slot in 0..2{
                m25_same(&format!("final workers {} slot {slot} reproduction",lane.workers),lane.batch.reproduction(slot).map_err(|error|format!("final batch reproduction: {error:?}"))?,single.reproduction().map_err(|error|format!("final single reproduction: {error:?}"))?)?;
                m25_same(&format!("final workers {} slot {slot} record",lane.workers),lane.batch.record(slot).map_err(|error|format!("final batch record: {error:?}"))?,Some(record.clone()))?;
            }
        }
        if case.start.is_none(){
            let bytes=record.to_json().map_err(|error|format!("{} normal strict JSON export: {error:?}",case.path))?;
            let imported=RunEpisodeRecord::from_json(&bytes).map_err(|error|format!("{} normal strict JSON import: {error:?}",case.path))?;
            m25_same("normal strict record JSON roundtrip",imported.clone(),record.clone())?;
            let report=crate::replay_run(&imported,case.config.capacities.clone()).map_err(|error|format!("{} normal replay: {error:?}",case.path))?;
            m25_same("normal replay accepted count",report.accepted_count,case.accepted)?;
            m25_same("normal replay final decision",report.final_decision,run.decision().map_err(|error|format!("normal final decision: {error:?}"))?)?;
            m25_same("normal replay final map",report.final_map,run.public_map())?;
            m25_same("normal replay rewards",report.rewards,record.steps.iter().map(|step|step.reward).collect::<Vec<_>>())?;
            m25_same("normal replay terminals",report.terminals,record.steps.iter().map(|step|step.terminal).collect::<Vec<_>>())?;
        }else{
            m25_same("supplied replay policy-only count",record.steps.len(),case.accepted)?;
            m25_same("supplied replay bootstrap excluded",record.steps.len()+case.bootstrap_count,case.accepted+case.bootstrap_count)?;
        }
        m25_same("release freeze preserved",matches!(case.freeze.as_str(),"b0e4c5ffe6d021fb0d59fde36daed7294fe54dc2782c27581f4605c4639fa72c"|"879fbb1be3fb396802c315916052494768265df93ba21d5461ace735f03f344e"|"5577735eaefaad40855627ff228818b9ac08ab37b2ee2dc0e2730dbe4b3658d0"),true)
    }

    #[test]
    #[ignore="HOLD: requires the frozen M25 receipts (SPIRE_M25_RECORDED_RELEASE=<path>, SPIRE_M25_RECORDED_RELEASE_SHA256=<sha>); recorded actions only, no policy"]
    fn m25_recorded_native_agreement(){
        let cases=m25_load_recorded_cases().unwrap_or_else(|error|panic!("M25 recorded agreement admission failed: {error}"));
        let mut failures=Vec::new();
        for case in &cases{
            match m25_consume_recorded_case(case){
                Ok(())=>println!("M25_NATIVE_AGREEMENT path={} series={} agreement=yes first_differing_step=none actions={} workers=1,2",case.path,case.series,case.accepted),
                Err(first)=>{println!("M25_NATIVE_AGREEMENT path={} series={} agreement=no first_differing_step={first}",case.path,case.series);failures.push(format!("{}: {first}",case.path));}
            }
        }
        assert!(failures.is_empty(),"M25 frozen native agreement mismatches: {}",failures.join("; "));
    }
}
