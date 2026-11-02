use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{RunAct1BossTarget as A1, RunAct2BossTarget as A2, RunAct3BossTarget as A3,
    RunBossHistory as History, RunCapacities, RunSession, RunSessionConfig, RunSessionError};

pub type Constructor = fn(RunSessionConfig) -> Result<RunSession, RunSessionError>;

#[derive(Clone, Copy)]
pub struct RunProfileSpec {
    pub id: &'static str,
    pub character: Character,
    pub constructor: Constructor,
}

impl RunProfileSpec {
    pub fn config(self) -> RunSessionConfig {
        let history = match self.id {
            "M9" | "M10" | "M10Policy" => History::AwakenedUnseen,
            "M12" | "M22" => History::Act1GuardianUnseen,
            "M13" => History::M13SilentAct2 { act1: A1::Guardian, act2: A2::Champ },
            "M14" => History::M14SilentAct3 { act1: A1::Guardian, act2: A2::Champ, act3: A3::AwakenedOne },
            "M15" => History::M15SilentHeart { act1: A1::Guardian, act2: A2::Champ, act3: A3::AwakenedOne },
            "M17" => History::M17DefectAct1 { act1: A1::Guardian },
            "M18" => History::M18DefectAct2 { act1: A1::Guardian, act2: A2::Champ },
            "M19" => History::M19DefectAct3 { act1: A1::Guardian, act2: A2::Champ, act3: A3::AwakenedOne },
            "M20" => History::M20DefectHeart { act1: A1::Guardian, act2: A2::Champ, act3: A3::AwakenedOne },
            "M23" => History::M23WatcherAct2 { act1: A1::Guardian, act2: A2::Champ },
            "M24" => History::M24WatcherAct3 { act1: A1::Guardian, act2: A2::Champ, act3: A3::AwakenedOne },
            "M25" => History::M25WatcherHeart { act1: A1::Guardian, act2: A2::Champ, act3: A3::AwakenedOne },
            _ => History::AwakenedUnseen,
        };
        RunSessionConfig { character: self.character, ascension: 0, partition: Partition::Test,
            boss_history: history, capacities: RunCapacities::m10_default(), record_replay: true }
    }
}

/// Every accepted Run session constructor. Add new finite profile rows at the marked seams.
pub const RUN_PROFILES: &[RunProfileSpec] = &[
    RunProfileSpec { id: "M9", character: Character::Ironclad, constructor: RunSession::new },
    RunProfileSpec { id: "M10", character: Character::Ironclad, constructor: RunSession::new_m10 },
    RunProfileSpec { id: "M10Policy", character: Character::Ironclad, constructor: RunSession::new_m10_policy },
    RunProfileSpec { id: "M12", character: Character::Silent, constructor: RunSession::new_m12 },
    RunProfileSpec { id: "M13", character: Character::Silent, constructor: RunSession::new_m13 },
    RunProfileSpec { id: "M14", character: Character::Silent, constructor: RunSession::new_m14 },
    RunProfileSpec { id: "M15", character: Character::Silent, constructor: RunSession::new_m15 },
    RunProfileSpec { id: "M17", character: Character::Defect, constructor: RunSession::new_m17 },
    RunProfileSpec { id: "M18", character: Character::Defect, constructor: RunSession::new_m18 },
    RunProfileSpec { id: "M19", character: Character::Defect, constructor: RunSession::new_m19 },
    RunProfileSpec { id: "M20", character: Character::Defect, constructor: RunSession::new_m20 },
    RunProfileSpec { id: "M22", character: Character::Watcher, constructor: RunSession::new_m22 },
    RunProfileSpec { id: "M23", character: Character::Watcher, constructor: RunSession::new_m23 },
    RunProfileSpec { id: "M24", character: Character::Watcher, constructor: RunSession::new_m24 },
    RunProfileSpec { id: "M25", character: Character::Watcher, constructor: RunSession::new_m25 },
    // SEAM: M27 complete-items profile
    // SEAM: M28 complete events
    // SEAM: M28 StandardA0 Neow-start full Run — spire_core::run::diagnostics::standard_run(character, full_neow, rng) behind m28-diagnostics (diagnostics-only until M28f); future driver, not built.
];

/// Complete-item diagnostics begin at the constructed Act 4 map, without Neow.
#[cfg(feature = "diagnostics")]
#[derive(Clone, Copy)]
pub struct CompleteItemsRunProfile {
    pub id: &'static str,
    pub character: Character,
}

#[cfg(feature = "diagnostics")]
pub const COMPLETE_ITEMS_RUN_PROFILES: &[CompleteItemsRunProfile] = &[
    CompleteItemsRunProfile { id: "CompleteItemsIronclad", character: Character::Ironclad },
    CompleteItemsRunProfile { id: "CompleteItemsSilent", character: Character::Silent },
    CompleteItemsRunProfile { id: "CompleteItemsDefect", character: Character::Defect },
    CompleteItemsRunProfile { id: "CompleteItemsWatcher", character: Character::Watcher },
];

pub fn profile(id: &str) -> Option<RunProfileSpec> { RUN_PROFILES.iter().copied().find(|row| row.id == id) }

#[cfg(feature = "diagnostics")]
pub fn complete_items_profile(id: &str) -> Option<CompleteItemsRunProfile> {
    COMPLETE_ITEMS_RUN_PROFILES.iter().copied().find(|row| row.id == id)
}
