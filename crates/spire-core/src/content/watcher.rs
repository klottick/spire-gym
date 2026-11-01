//! Finite M21 and complete M25 Watcher construction admission.
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

/// The 47 ordinary Watcher kinds M25 adds to the M21 roster (design §5.1).
const M25_NEW_WATCHER_CARDS: &[CardId] = &[
    CardId::BowlingBash, CardId::Consecrate, CardId::Crescendo, CardId::CrushJoints, CardId::EmptyBody,
    CardId::Evaluate, CardId::FlyingSleeves, CardId::FollowUp, CardId::Halt, CardId::PressurePoints,
    CardId::Protect, CardId::SashWhip, CardId::Tranquility,
    CardId::Collect, CardId::Conclude, CardId::EmptyMind, CardId::Fasting, CardId::FearNoEvil,
    CardId::ForeignInfluence, CardId::Indignation, CardId::InnerPeace, CardId::Pray, CardId::ReachHeaven,
    CardId::Sanctity, CardId::SignatureMove, CardId::SimmeringFury, CardId::Study, CardId::Swivel,
    CardId::TalkToTheHand, CardId::Tantrum, CardId::Wallop, CardId::WaveOfTheHand, CardId::WheelKick,
    CardId::WreathOfFlame,
    CardId::Alpha, CardId::Blasphemy, CardId::ConjureBlade, CardId::DeusExMachina, CardId::DevaForm,
    CardId::Judgment, CardId::LessonLearned, CardId::Omniscience, CardId::Ragnarok, CardId::Scrawl,
    CardId::SpiritShield, CardId::Vault, CardId::Wish,
];

/// The five Watcher children M25 adds to the M21 generated kinds (design §5.2).
const M25_NEW_WATCHER_GENERATED: &[CardId] = &[
    CardId::Insight, CardId::ThroughViolence, CardId::Beta, CardId::Omega, CardId::Expunger,
];

const fn concat<const N: usize>(head: &[CardId], tail: &[CardId]) -> [CardId; N] {
    assert!(head.len() + tail.len() == N);
    let mut out = [CardId::StrikePurple; N];
    let mut index = 0;
    while index < head.len() { out[index] = head[index]; index += 1; }
    let mut offset = 0;
    while offset < tail.len() { out[head.len() + offset] = tail[offset]; offset += 1; }
    out
}

/// The complete 75-kind Watcher roster: the M21 list followed by the M25 additions.
pub const M25_WATCHER_CARDS: &[CardId] = &concat::<75>(M21_WATCHER_CARDS, M25_NEW_WATCHER_CARDS);

/// All eight generated-only Watcher children: the M21 three followed by the M25 five.
pub const M25_WATCHER_GENERATED: &[CardId] = &concat::<8>(M21_WATCHER_GENERATED, M25_NEW_WATCHER_GENERATED);

/// Wish's choice-only options. They are never physical cards, never enter a pile
/// and are never permanent, so neither M25 predicate admits them.
pub const M25_WATCHER_CHOICES: &[CardId] = &[CardId::BecomeAlmighty, CardId::FameAndFortune, CardId::LiveForever];

/// Whether this card kind is admitted for permanent complete-Watcher (M25) construction.
pub fn supports_m25_permanent(id: CardId) -> bool {
    M25_WATCHER_CARDS.contains(&id)
}

/// Whether this card kind is admitted for explicit generated/copy construction under M25.
/// Foreign Influence outputs keep their own colour's construction and are not listed here.
pub fn supports_m25_generated(id: CardId) -> bool {
    supports_m25_permanent(id) || M25_WATCHER_GENERATED.contains(&id)
}

/// Foreign Influence's support (design §5.3): every ordinary Attack of Common,
/// Uncommon or Rare rarity in catalog row order, with no colour predicate and
/// HEALING-tagged kinds excluded. Derived from the catalog, never a copied list.
pub fn foreign_influence_support() -> &'static [CardId] {
    // M26: owned by `content::pools` (M25's order, size 106 checked there).
    static SUPPORT: std::sync::OnceLock<Vec<CardId>> = std::sync::OnceLock::new();
    SUPPORT.get_or_init(|| super::pools::cards(super::pools::CardPool::ForeignInfluence).unwrap_or_default())
}

/// Authoritative spellings of every new `FullPowerId` that M25 cards and relics
/// create. Existing rows are reused where the semantics match (Strength,
/// Dexterity, Lose Dexterity, Plated Armor, Mantra, Draw Card Next Turn).
pub const M25_WATCHER_POWER_IDS: &[&str] = &[
    "mark", "block_return", "vigor", "wave_of_the_hand", "free_attack", "simmering_fury",
    "study", "energy_down", "deva_form", "collect", "omega", "blasphemer",
];
