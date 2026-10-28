//! Finite M16 Defect card admission metadata.
//!
//! This list admits explicit permanent and generated/copy construction for the
//! profile. It does not define a selector, ordinary reward pool, random
//! generator, card rules, or executable mechanics support.

use super::CardId;

/// The exact 29 card kinds admitted by the finite M16 Defect profile.
pub const M16_DEFECT_CARDS: &[CardId] = &[
    CardId::StrikeBlue,
    CardId::DefendBlue,
    CardId::Zap,
    CardId::Dualcast,
    CardId::BallLightning,
    CardId::ColdSnap,
    CardId::Coolheaded,
    CardId::Barrage,
    CardId::CompileDriver,
    CardId::Darkness,
    CardId::Fusion,
    CardId::Capacitor,
    CardId::Consume,
    CardId::Defragment,
    CardId::BiasedCognition,
    CardId::Loop,
    CardId::Electrodynamics,
    CardId::Bullseye,
    CardId::Recursion,
    CardId::Fission,
    CardId::StaticDischarge,
    CardId::Storm,
    CardId::Heatsinks,
    CardId::Amplify,
    CardId::EchoForm,
    CardId::GeneticAlgorithm,
    CardId::Claw,
    CardId::SteamBarrier,
    CardId::Streamline,
];

/// Whether this card kind is admitted for explicit permanent M16 construction.
pub fn supports_permanent(id: CardId) -> bool {
    M16_DEFECT_CARDS.contains(&id)
}

/// Whether this card kind is admitted for explicit generated/copy construction.
/// This does not authorize random selection or add a generator pool.
pub fn supports_generated(id: CardId) -> bool {
    M16_DEFECT_CARDS.contains(&id)
}
