use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{RunAct1BossTarget as A1, RunAct2BossTarget as A2, RunAct3BossTarget as A3,
    RunBossHistory as History, RunCapacities, RunSession, RunSessionConfig, RunSessionError,
    StandardCreation, StandardNeow};

pub type Constructor = fn(RunSessionConfig) -> Result<RunSession, RunSessionError>;

#[derive(Clone, Copy)]
pub struct RunProfileSpec {
    pub id: &'static str,
    pub character: Character,
    pub ascension: u8,
    /// Present for Standard specs. Captured specs retain their original A0 creation behavior.
    pub creation: Option<StandardCreation>,
    pub constructor: Option<Constructor>,
}

impl RunProfileSpec {
    pub fn config(self) -> RunSessionConfig {
        let history = if let Some(creation) = self.creation { History::Standard(creation) } else { match self.id {
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
        }};
        RunSessionConfig { character: self.character, ascension: self.ascension, partition: Partition::Test,
            boss_history: history, capacities: if self.creation.is_some() { RunCapacities::standard_default() }
                else { RunCapacities::m10_default() }, record_replay: true }
    }
}

const fn captured(id: &'static str, character: Character, constructor: Constructor) -> RunProfileSpec {
    RunProfileSpec { id, character, ascension: 0, creation: None, constructor: Some(constructor) }
}

const fn standard(id: &'static str, character: Character, ascension: u8) -> RunProfileSpec {
    RunProfileSpec { id, character, ascension,
        creation: Some(StandardCreation { neow: StandardNeow::Full, seen_bosses: [[true; 3]; 3],
            stored_note: (spire_core::content::CardId::IronWave, 0) }), constructor: None }
}

/// Every accepted Run session constructor. Add new finite profile rows at the marked seams.
pub const RUN_PROFILES: &[RunProfileSpec] = &[
    captured("M9", Character::Ironclad, RunSession::new),
    captured("M10", Character::Ironclad, RunSession::new_m10),
    captured("M10Policy", Character::Ironclad, RunSession::new_m10_policy),
    captured("M12", Character::Silent, RunSession::new_m12),
    captured("M13", Character::Silent, RunSession::new_m13),
    captured("M14", Character::Silent, RunSession::new_m14),
    captured("M15", Character::Silent, RunSession::new_m15),
    captured("M17", Character::Defect, RunSession::new_m17),
    captured("M18", Character::Defect, RunSession::new_m18),
    captured("M19", Character::Defect, RunSession::new_m19),
    captured("M20", Character::Defect, RunSession::new_m20),
    captured("M22", Character::Watcher, RunSession::new_m22),
    captured("M23", Character::Watcher, RunSession::new_m23),
    captured("M24", Character::Watcher, RunSession::new_m24),
    captured("M25", Character::Watcher, RunSession::new_m25),
    standard("standard-ironclad-a0", Character::Ironclad, 0),
    standard("standard-ironclad-a10", Character::Ironclad, 10),
    standard("standard-ironclad-a20", Character::Ironclad, 20),
    standard("standard-silent-a0", Character::Silent, 0),
    standard("standard-silent-a10", Character::Silent, 10),
    standard("standard-silent-a20", Character::Silent, 20),
    standard("standard-defect-a0", Character::Defect, 0),
    standard("standard-defect-a10", Character::Defect, 10),
    standard("standard-defect-a20", Character::Defect, 20),
    standard("standard-watcher-a0", Character::Watcher, 0),
    standard("standard-watcher-a10", Character::Watcher, 10),
    standard("standard-watcher-a20", Character::Watcher, 20),
    // SEAM: M27 complete-items profile
    // SEAM: M28 complete events
    // SEAM: M28 Standard Neow-start full Run is a public native constructor (M28f2) but has no spire-env session or encoder (PM ruling Q1, deferred to M34–M36); `examples/m28_traversal.rs` in spire-core drives it instead.
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
