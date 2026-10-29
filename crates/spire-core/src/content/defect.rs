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

/// The exact 75 blue card kinds in the M20 source roster.
///
/// This is an explicit construction/copy capability only. It does not define
/// reward probabilities, selector pools, or executable mechanics support.
pub const M20_DEFECT_CARDS: &[CardId] = &[
    CardId::StrikeBlue,
    CardId::DefendBlue,
    CardId::Zap,
    CardId::Dualcast,
    CardId::BallLightning,
    CardId::Barrage,
    CardId::BeamCell,
    CardId::ChargeBattery,
    CardId::Claw,
    CardId::ColdSnap,
    CardId::CompileDriver,
    CardId::Coolheaded,
    CardId::GoForTheEyes,
    CardId::Hologram,
    CardId::Leap,
    CardId::Rebound,
    CardId::Recursion,
    CardId::Stack,
    CardId::SteamBarrier,
    CardId::Streamline,
    CardId::SweepingBeam,
    CardId::Turbo,
    CardId::Aggregate,
    CardId::AutoShields,
    CardId::Blizzard,
    CardId::BootSequence,
    CardId::Bullseye,
    CardId::Capacitor,
    CardId::Chaos,
    CardId::Chill,
    CardId::Consume,
    CardId::Darkness,
    CardId::Defragment,
    CardId::DoomAndGloom,
    CardId::DoubleEnergy,
    CardId::Equilibrium,
    CardId::ForceField,
    CardId::FTL,
    CardId::Fusion,
    CardId::GeneticAlgorithm,
    CardId::Glacier,
    CardId::Heatsinks,
    CardId::HelloWorld,
    CardId::Loop,
    CardId::Melter,
    CardId::Overclock,
    CardId::Recycle,
    CardId::ReinforcedBody,
    CardId::Reprogram,
    CardId::RipAndTear,
    CardId::Scrape,
    CardId::SelfRepair,
    CardId::Skim,
    CardId::StaticDischarge,
    CardId::Storm,
    CardId::Sunder,
    CardId::Tempest,
    CardId::WhiteNoise,
    CardId::AllForOne,
    CardId::Amplify,
    CardId::BiasedCognition,
    CardId::Buffer,
    CardId::CoreSurge,
    CardId::CreativeAI,
    CardId::EchoForm,
    CardId::Electrodynamics,
    CardId::Fission,
    CardId::Hyperbeam,
    CardId::MachineLearning,
    CardId::MeteorStrike,
    CardId::MultiCast,
    CardId::Rainbow,
    CardId::Reboot,
    CardId::Seek,
    CardId::ThunderStrike,
];

/// Whether a blue M20 card kind may be explicitly constructed permanently.
pub fn supports_m20_permanent(id: CardId) -> bool {
    M20_DEFECT_CARDS.contains(&id)
}

/// Whether a blue M20 card kind may be explicitly constructed as generated/copy content.
/// This does not enable a selector pool or assert executable mechanics support.
pub fn supports_m20_generated(id: CardId) -> bool {
    M20_DEFECT_CARDS.contains(&id)
}

#[cfg(test)]
mod tests {
    use super::{
        supports_generated, supports_m20_generated, supports_m20_permanent,
        supports_permanent, M16_DEFECT_CARDS, M20_DEFECT_CARDS,
    };
    use crate::content::{
        CardColor, CardId, Character, ContentProducer, RegistryError,
    };
    use std::collections::HashSet;

    #[test]
    fn m20_blue_capability_is_exact_and_keeps_m16_finite() {
        assert_eq!(M16_DEFECT_CARDS.len(), 29);
        assert_eq!(M20_DEFECT_CARDS.len(), 75);
        assert_eq!(M20_DEFECT_CARDS.iter().copied().collect::<HashSet<_>>().len(), 75);
        for id in M16_DEFECT_CARDS {
            assert!(supports_permanent(*id));
            assert!(supports_generated(*id));
            assert!(supports_m20_permanent(*id));
            assert!(supports_m20_generated(*id));
        }
        for id in M20_DEFECT_CARDS {
            assert!(supports_m20_permanent(*id));
            assert!(supports_m20_generated(*id));
            let definition = crate::content::Catalog::standard().card(*id).expect("blue catalog row");
            assert_eq!(definition.color, CardColor::Blue);
            assert!(definition.blue_metadata.is_some());
        }

        for id in [CardId::BeamCell, CardId::AllForOne] {
            assert!(!supports_permanent(id));
            assert!(!supports_generated(id));
            assert!(supports_m20_permanent(id));
            assert!(supports_m20_generated(id));
        }
        for non_blue in [CardId::Parasite, CardId::Dazed, CardId::Apotheosis, CardId::Anger] {
            assert!(!supports_m20_permanent(non_blue));
            assert!(!supports_m20_generated(non_blue));
        }

        assert_eq!(
            crate::content::Catalog::standard().selectable_pool(
                ContentProducer::OrdinaryReward,
                Character::Defect,
            ),
            Err(RegistryError::IncompletePool),
        );
    }
}
