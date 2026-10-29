//! Finite M21 Watcher construction admission.
//!
//! These lists are explicit capability boundaries. They do not define an
//! ordinary reward pool, a random generator, or card mechanics.

use super::CardId;

/// The exact 28 permanent card kinds admitted by the finite M21 profile.
pub const M21_WATCHER_CARDS: &[CardId] = &[
    CardId::StrikePurple,
    CardId::DefendPurple,
    CardId::Eruption,
    CardId::Vigilance,
    CardId::EmptyFist,
    CardId::FlurryOfBlows,
    CardId::Prostrate,
    CardId::Worship,
    CardId::Brilliance,
    CardId::Devotion,
    CardId::ThirdEye,
    CardId::CutThroughFate,
    CardId::JustLucky,
    CardId::Weave,
    CardId::Foresight,
    CardId::Nirvana,
    CardId::WindmillStrike,
    CardId::Perseverance,
    CardId::SandsOfTime,
    CardId::Establishment,
    CardId::Meditate,
    CardId::CarveReality,
    CardId::DeceiveReality,
    CardId::MasterReality,
    CardId::BattleHymn,
    CardId::MentalFortress,
    CardId::Rushdown,
    CardId::LikeWater,
];

/// Generated-only Watcher card kinds; these are never valid permanent cards.
pub const M21_WATCHER_GENERATED: &[CardId] = &[
    CardId::Miracle,
    CardId::Smite,
    CardId::Safety,
];

/// Whether this card kind is admitted for permanent M21 Watcher construction.
pub fn supports_permanent(id: CardId) -> bool {
    M21_WATCHER_CARDS.contains(&id)
}

/// Whether this card kind is admitted for explicit generated/copy construction.
/// This does not authorize random selection or add a generator pool.
pub fn supports_generated(id: CardId) -> bool {
    supports_permanent(id) || M21_WATCHER_GENERATED.contains(&id)
}
