use super::{EncounterId, EnemyId};

#[derive(Clone, Copy, Debug)]
pub struct EnemyDefinition { pub id: EnemyId, pub definition_only: bool }
#[derive(Clone, Copy, Debug)]
pub struct EncounterDefinition { pub id: EncounterId, pub enemies: &'static [EnemyId], pub definition_only: bool }

const CULTIST: &[EnemyId] = &[EnemyId::Cultist];
const JAW_WORM: &[EnemyId] = &[EnemyId::JawWorm];
const GREMLIN_NOB: &[EnemyId] = &[EnemyId::GremlinNob];
const TWO_LOUSE: &[EnemyId] = &[EnemyId::LouseRed, EnemyId::LouseGreen];
const SLIME_BOSS: &[EnemyId] = &[EnemyId::SlimeBoss];
const THREE_CULTISTS: &[EnemyId] = &[EnemyId::Cultist, EnemyId::Cultist, EnemyId::Cultist];
const JAW_WORM_HORDE: &[EnemyId] = &[EnemyId::JawWorm, EnemyId::JawWorm, EnemyId::JawWorm];
pub(crate) const ENEMIES: &[EnemyDefinition] = &[
    EnemyDefinition { id: EnemyId::Cultist, definition_only: true },
    EnemyDefinition { id: EnemyId::JawWorm, definition_only: true },
    EnemyDefinition { id: EnemyId::GremlinNob, definition_only: false },
    EnemyDefinition { id: EnemyId::LouseRed, definition_only: true },
    EnemyDefinition { id: EnemyId::LouseGreen, definition_only: true },
    EnemyDefinition { id: EnemyId::SlimeBoss, definition_only: false },
];
pub(crate) const ENCOUNTERS: &[EncounterDefinition] = &[
    EncounterDefinition { id: EncounterId::Cultist, enemies: CULTIST, definition_only: true },
    EncounterDefinition { id: EncounterId::JawWorm, enemies: JAW_WORM, definition_only: true },
    EncounterDefinition { id: EncounterId::GremlinNob, enemies: GREMLIN_NOB, definition_only: false },
    EncounterDefinition { id: EncounterId::TwoLouse, enemies: TWO_LOUSE, definition_only: true },
    EncounterDefinition { id: EncounterId::SlimeBoss, enemies: SLIME_BOSS, definition_only: false },
    EncounterDefinition { id: EncounterId::ThreeCultists, enemies: THREE_CULTISTS, definition_only: false },
    EncounterDefinition { id: EncounterId::JawWormHorde, enemies: JAW_WORM_HORDE, definition_only: false },
];
