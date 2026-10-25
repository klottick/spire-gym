use super::{EncounterId, EnemyId};

#[derive(Clone, Copy, Debug)]
pub struct EnemyDefinition { pub id: EnemyId, pub definition_only: bool }
#[derive(Clone, Copy, Debug)]
pub struct EncounterDefinition { pub id: EncounterId, pub enemies: &'static [EnemyId], pub definition_only: bool }

const CULTIST: &[EnemyId] = &[EnemyId::Cultist];
const JAW_WORM: &[EnemyId] = &[EnemyId::JawWorm];
const GREMLIN_NOB: &[EnemyId] = &[EnemyId::GremlinNob];
const LAGAVULIN: &[EnemyId] = &[EnemyId::Lagavulin];
const GUARDIAN: &[EnemyId] = &[EnemyId::Guardian];
const HEXAGHOST: &[EnemyId] = &[EnemyId::Hexaghost];
const SPHERIC_GUARDIAN: &[EnemyId] = &[EnemyId::SphericGuardian];
const THREE_BYRDS: &[EnemyId] = &[EnemyId::Byrd;3];
const CHOSEN_BYRD: &[EnemyId] = &[EnemyId::Byrd,EnemyId::Chosen];
const SNECKO: &[EnemyId] = &[EnemyId::Snecko];
const CENTURION_MYSTIC: &[EnemyId] = &[EnemyId::Centurion,EnemyId::Mystic];
// The actual two weighted small species are constructed by native setup.
const GREMLIN_LEADER: &[EnemyId] = &[EnemyId::GremlinLeader];
const SLAVERS:&[EnemyId]=&[EnemyId::SlaverBlue,EnemyId::Taskmaster,EnemyId::SlaverRed];
const BOOK:&[EnemyId]=&[EnemyId::BookOfStabbing];
const CHAMP:&[EnemyId]=&[EnemyId::Champ];
const COLLECTOR:&[EnemyId]=&[EnemyId::Collector];
const AUTOMATON:&[EnemyId]=&[EnemyId::BronzeAutomaton];
const THREE_SENTRIES: &[EnemyId] = &[EnemyId::Sentry;3];
const THREE_LOUSE: &[EnemyId] = &[EnemyId::LouseRed,EnemyId::LouseGreen,EnemyId::LouseRed];
const TWO_LOUSE: &[EnemyId] = &[EnemyId::LouseRed, EnemyId::LouseGreen];
const SLIME_BOSS: &[EnemyId] = &[EnemyId::SlimeBoss];
const THREE_CULTISTS: &[EnemyId] = &[EnemyId::Cultist, EnemyId::Cultist, EnemyId::Cultist];
const JAW_WORM_HORDE: &[EnemyId] = &[EnemyId::JawWorm, EnemyId::JawWorm, EnemyId::JawWorm];
const ORB_WALKER:&[EnemyId]=&[EnemyId::OrbWalker];
const DARKLINGS:&[EnemyId]=&[EnemyId::Darkling;3];
const WRITHING:&[EnemyId]=&[EnemyId::WrithingMass];
const GROWTH:&[EnemyId]=&[EnemyId::SpireGrowth];const TRANSIENT:&[EnemyId]=&[EnemyId::Transient];const MAW:&[EnemyId]=&[EnemyId::Maw];
// Dynamic groups list their supported species, not a fabricated fixed formation.
const SHAPES:&[EnemyId]=&[EnemyId::Repulsor,EnemyId::Exploder,EnemyId::Spiker];
const SPHERE_SHAPES:&[EnemyId]=&[EnemyId::Repulsor,EnemyId::Exploder,EnemyId::Spiker,EnemyId::SphericGuardian];
const HEART:&[EnemyId]=&[EnemyId::CorruptHeart];
pub(crate) const ENEMIES: &[EnemyDefinition] = &[
    EnemyDefinition{id:EnemyId::Deca,definition_only:false},
    EnemyDefinition{id:EnemyId::CorruptHeart,definition_only:false},
    EnemyDefinition{id:EnemyId::Donu,definition_only:false},
    EnemyDefinition{id:EnemyId::SpireShield,definition_only:false},
    EnemyDefinition{id:EnemyId::SpireSpear,definition_only:false},
    EnemyDefinition{id:EnemyId::TimeEater,definition_only:false},
    EnemyDefinition{id:EnemyId::AwakenedOne,definition_only:false},
    EnemyDefinition{id:EnemyId::Reptomancer,definition_only:false},
    EnemyDefinition{id:EnemyId::Dagger,definition_only:false},
    EnemyDefinition{id:EnemyId::GiantHead,definition_only:false},
    EnemyDefinition{id:EnemyId::Nemesis,definition_only:false},
    EnemyDefinition{id:EnemyId::WrithingMass,definition_only:false},
    EnemyDefinition{id:EnemyId::SpireGrowth,definition_only:false},EnemyDefinition{id:EnemyId::Transient,definition_only:false},EnemyDefinition{id:EnemyId::Maw,definition_only:false},
    EnemyDefinition{id:EnemyId::Darkling,definition_only:false},
    EnemyDefinition{id:EnemyId::OrbWalker,definition_only:false},EnemyDefinition{id:EnemyId::Repulsor,definition_only:false},EnemyDefinition{id:EnemyId::Exploder,definition_only:false},EnemyDefinition{id:EnemyId::Spiker,definition_only:false},
    EnemyDefinition {id:EnemyId::Lagavulin,definition_only:false},
    EnemyDefinition {id:EnemyId::Sentry,definition_only:false},
    EnemyDefinition {id:EnemyId::Guardian,definition_only:false},
    EnemyDefinition {id:EnemyId::Hexaghost,definition_only:false},
    EnemyDefinition {id:EnemyId::SphericGuardian,definition_only:false},
    EnemyDefinition {id:EnemyId::Byrd,definition_only:false},
    EnemyDefinition {id:EnemyId::Chosen,definition_only:false},
    EnemyDefinition {id:EnemyId::Snecko,definition_only:false},
    EnemyDefinition {id:EnemyId::Centurion,definition_only:false},
    EnemyDefinition {id:EnemyId::Mystic,definition_only:false},
    EnemyDefinition {id:EnemyId::GremlinLeader,definition_only:false},
    EnemyDefinition {id:EnemyId::GremlinMad,definition_only:false},
    EnemyDefinition {id:EnemyId::GremlinSneaky,definition_only:false},
    EnemyDefinition {id:EnemyId::GremlinFat,definition_only:false},
    EnemyDefinition {id:EnemyId::GremlinShield,definition_only:false},
    EnemyDefinition {id:EnemyId::GremlinWizard,definition_only:false},
    EnemyDefinition {id:EnemyId::SlaverBlue,definition_only:false},
    EnemyDefinition {id:EnemyId::Taskmaster,definition_only:false},
    EnemyDefinition {id:EnemyId::SlaverRed,definition_only:false},
    EnemyDefinition {id:EnemyId::BookOfStabbing,definition_only:false},
    EnemyDefinition {id:EnemyId::Champ,definition_only:false},
    EnemyDefinition {id:EnemyId::Collector,definition_only:false},
    EnemyDefinition {id:EnemyId::TorchHead,definition_only:false},
    EnemyDefinition {id:EnemyId::BronzeAutomaton,definition_only:false},
    EnemyDefinition {id:EnemyId::BronzeOrb,definition_only:false},
    EnemyDefinition { id: EnemyId::Cultist, definition_only: true },
    EnemyDefinition { id: EnemyId::JawWorm, definition_only: true },
    EnemyDefinition { id: EnemyId::GremlinNob, definition_only: false },
    EnemyDefinition { id: EnemyId::LouseRed, definition_only: true },
    EnemyDefinition { id: EnemyId::LouseGreen, definition_only: true },
    EnemyDefinition { id: EnemyId::SlimeBoss, definition_only: false },
];
pub(crate) const ENCOUNTERS: &[EncounterDefinition] = &[
    EncounterDefinition{id:EncounterId::ShieldAndSpear,enemies:&[EnemyId::SpireShield,EnemyId::SpireSpear],definition_only:false},
    EncounterDefinition{id:EncounterId::DonuAndDeca,enemies:&[EnemyId::Deca,EnemyId::Donu],definition_only:false},
    EncounterDefinition{id:EncounterId::CorruptHeart,enemies:HEART,definition_only:false},
    EncounterDefinition{id:EncounterId::TimeEater,enemies:&[EnemyId::TimeEater],definition_only:false},
    EncounterDefinition{id:EncounterId::AwakenedOne,enemies:&[EnemyId::Cultist,EnemyId::Cultist,EnemyId::AwakenedOne],definition_only:false},
    EncounterDefinition{id:EncounterId::Reptomancer,enemies:&[EnemyId::Dagger,EnemyId::Reptomancer,EnemyId::Dagger],definition_only:false},
    EncounterDefinition{id:EncounterId::GiantHead,enemies:&[EnemyId::GiantHead],definition_only:false},
    EncounterDefinition{id:EncounterId::Nemesis,enemies:&[EnemyId::Nemesis],definition_only:false},
    EncounterDefinition{id:EncounterId::WrithingMass,enemies:WRITHING,definition_only:false},
    EncounterDefinition{id:EncounterId::SpireGrowth,enemies:GROWTH,definition_only:false},EncounterDefinition{id:EncounterId::Transient,enemies:TRANSIENT,definition_only:false},EncounterDefinition{id:EncounterId::Maw,enemies:MAW,definition_only:false},
    EncounterDefinition{id:EncounterId::ThreeDarklings,enemies:DARKLINGS,definition_only:false},
    EncounterDefinition{id:EncounterId::OrbWalker,enemies:ORB_WALKER,definition_only:false},EncounterDefinition{id:EncounterId::ThreeShapes,enemies:SHAPES,definition_only:false},EncounterDefinition{id:EncounterId::FourShapes,enemies:SHAPES,definition_only:false},EncounterDefinition{id:EncounterId::SphereAndTwoShapes,enemies:SPHERE_SHAPES,definition_only:false},
    EncounterDefinition {id:EncounterId::Lagavulin,enemies:LAGAVULIN,definition_only:false},
    EncounterDefinition {id:EncounterId::ThreeSentries,enemies:THREE_SENTRIES,definition_only:false},
    EncounterDefinition {id:EncounterId::ThreeLouse,enemies:THREE_LOUSE,definition_only:false},
    EncounterDefinition {id:EncounterId::Guardian,enemies:GUARDIAN,definition_only:false},
    EncounterDefinition {id:EncounterId::Hexaghost,enemies:HEXAGHOST,definition_only:false},
    EncounterDefinition {id:EncounterId::SphericGuardian,enemies:SPHERIC_GUARDIAN,definition_only:false},
    EncounterDefinition {id:EncounterId::ThreeByrds,enemies:THREE_BYRDS,definition_only:false},
    EncounterDefinition {id:EncounterId::ChosenAndByrds,enemies:CHOSEN_BYRD,definition_only:false},
    EncounterDefinition {id:EncounterId::Snecko,enemies:SNECKO,definition_only:false},
    EncounterDefinition {id:EncounterId::CenturionAndHealer,enemies:CENTURION_MYSTIC,definition_only:false},
    EncounterDefinition {id:EncounterId::GremlinLeader,enemies:GREMLIN_LEADER,definition_only:false},
    EncounterDefinition {id:EncounterId::Slavers,enemies:SLAVERS,definition_only:false},
    EncounterDefinition {id:EncounterId::BookOfStabbing,enemies:BOOK,definition_only:false},
    EncounterDefinition {id:EncounterId::Champ,enemies:CHAMP,definition_only:false},
    EncounterDefinition {id:EncounterId::Collector,enemies:COLLECTOR,definition_only:false},
    EncounterDefinition {id:EncounterId::BronzeAutomaton,enemies:AUTOMATON,definition_only:false},
    EncounterDefinition { id: EncounterId::Cultist, enemies: CULTIST, definition_only: true },
    EncounterDefinition { id: EncounterId::JawWorm, enemies: JAW_WORM, definition_only: true },
    EncounterDefinition { id: EncounterId::GremlinNob, enemies: GREMLIN_NOB, definition_only: false },
    EncounterDefinition { id: EncounterId::TwoLouse, enemies: TWO_LOUSE, definition_only: true },
    EncounterDefinition { id: EncounterId::SlimeBoss, enemies: SLIME_BOSS, definition_only: false },
    EncounterDefinition { id: EncounterId::ThreeCultists, enemies: THREE_CULTISTS, definition_only: false },
    EncounterDefinition { id: EncounterId::JawWormHorde, enemies: JAW_WORM_HORDE, definition_only: false },
];
