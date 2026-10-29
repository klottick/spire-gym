//! Finite M22 Watcher Act1 producer data and ordered content witnesses.

use crate::content::{CardId, PotionId, RelicId};

/// Permanent cards admitted by the already accepted M21 Watcher capability set.
pub const M22_WATCHER_ACT1_CARDS: &[CardId] = crate::content::watcher::M21_WATCHER_CARDS;

/// Ordered union: starter/Neow, elite, chest, then boss relic producers.
pub const M22_WATCHER_ACT1_RELICS: &[RelicId] = &[
    RelicId::PureWater,
    RelicId::NeowsLament,
    RelicId::Vajra,
    RelicId::OddlySmoothStone,
    RelicId::GoldenEye,
    RelicId::TeardropLocket,
    RelicId::Lantern,
    RelicId::Pear,
    RelicId::Mango,
    RelicId::ToyOrnithopter,
    RelicId::Strawberry,
    RelicId::Anchor,
    RelicId::BloodVial,
    RelicId::HolyWater,
    RelicId::VioletLotus,
    RelicId::SacredBark,
    RelicId::EmptyCage,
    RelicId::SlaversCollar,
];

pub const M22_WATCHER_ACT1_POTIONS: &[PotionId] = &POTION_CYCLE;

pub const NONBOSS_REWARD_ROWS: [[CardId; 3]; 6] = [
    [CardId::EmptyFist, CardId::MentalFortress, CardId::CarveReality],
    [CardId::CutThroughFate, CardId::Rushdown, CardId::DeceiveReality],
    [CardId::ThirdEye, CardId::Foresight, CardId::Meditate],
    [CardId::JustLucky, CardId::Nirvana, CardId::Perseverance],
    [CardId::FlurryOfBlows, CardId::Weave, CardId::SandsOfTime],
    [CardId::Prostrate, CardId::Worship, CardId::WindmillStrike],
];

pub const BOSS_CARD_REWARDS: [CardId; 3] = [
    CardId::Brilliance,
    CardId::Devotion,
    CardId::Establishment,
];

pub const SHOP_CARD_ROWS: [[CardId; 3]; 2] = [
    [CardId::BattleHymn, CardId::LikeWater, CardId::MasterReality],
    [CardId::MentalFortress, CardId::Meditate, CardId::Establishment],
];
pub const SHOP_CARD_PRICES: [u32; 3] = [75, 75, 150];

pub const ELITE_RELICS: [RelicId; 8] = [
    RelicId::Vajra,
    RelicId::OddlySmoothStone,
    RelicId::GoldenEye,
    RelicId::TeardropLocket,
    RelicId::Lantern,
    RelicId::Pear,
    RelicId::Mango,
    RelicId::ToyOrnithopter,
];
pub const CHEST_RELICS: [RelicId; 3] = [
    RelicId::Strawberry,
    RelicId::Anchor,
    RelicId::BloodVial,
];
pub const BOSS_RELICS: [RelicId; 5] = [
    RelicId::HolyWater,
    RelicId::VioletLotus,
    RelicId::SacredBark,
    RelicId::EmptyCage,
    RelicId::SlaversCollar,
];
pub const POTION_CYCLE: [PotionId; 7] = [
    PotionId::BlockPotion,
    PotionId::EnergyPotion,
    PotionId::DexterityPotion,
    PotionId::AncientPotion,
    PotionId::StancePotion,
    PotionId::Ambrosia,
    PotionId::BottledMiracle,
];
pub const SHOP_RELICS: [RelicId; 2] = [RelicId::GoldenEye, RelicId::TeardropLocket];
pub const SHOP_POTIONS: [PotionId; 2] = [PotionId::StancePotion, PotionId::BottledMiracle];

pub fn reward_cards(boss: bool, visit: u64) -> Vec<CardId> {
    if boss {
        BOSS_CARD_REWARDS.to_vec()
    } else {
        NONBOSS_REWARD_ROWS[(visit % 6) as usize].to_vec()
    }
}

pub fn shop_cards(later: bool) -> [CardId; 3] {
    SHOP_CARD_ROWS[usize::from(later)]
}

pub fn combat_potion(visit: u64) -> PotionId {
    POTION_CYCLE[(visit % 7) as usize]
}

/// Descriptor metadata and producer code share these finite capture facts.
pub const RELIC_CAPTURE_SCOPE: crate::run::RunRngScope = crate::run::RunRngScope::M22RelicCapture;
pub const RELIC_CAPTURE_SCOPE_NAME: &str = "run.m22.relic.capture";
pub const RELIC_CAPTURE_KEY_FIELDS: [&str; 5] = ["act", "floor", "node_id", "visit", "source_tag"];
pub const ELITE_SOURCE_TAG: u64 = 0;
pub const CHEST_SOURCE_TAG: u64 = 1;
pub const ELITE_VISIT_IDENTITY: &str = "CombatRecord.visit";
pub const CHEST_VISIT_IDENTITY: &str = "RunState.combat_progress.entered";
pub const RELIC_CAPTURE_FILTER_OWNED: bool = true;
pub const RELIC_CAPTURE_EMPTY_REQUEST: bool = false;
pub const RELIC_CAPTURE_SINGLETON_REQUEST: bool = true;
pub const RELIC_CAPTURE_DRAW_LAW: &str = "RunRng::bounded_scoped_checked; accept value < u64::MAX-(u64::MAX%bound); reject u64::MAX including bound1";
pub const SHOP_RELIC_PRICE: u32 = 150;
pub const SHOP_POTION_PRICE: u32 = 50;

/// Qualified reference taxonomy, not tier-preserving loot probability.
/// Kept in this profile owner so older catalog/descriptor metadata stays frozen.
pub const RELIC_TAXONOMY: &[(RelicId, &str)] = &[
    (RelicId::PureWater,"starter"),(RelicId::NeowsLament,"special"),
    (RelicId::Vajra,"common"),(RelicId::OddlySmoothStone,"common"),
    (RelicId::GoldenEye,"rare"),(RelicId::TeardropLocket,"uncommon"),
    (RelicId::Lantern,"common"),(RelicId::Pear,"uncommon"),(RelicId::Mango,"rare"),
    (RelicId::ToyOrnithopter,"common"),(RelicId::Strawberry,"common"),
    (RelicId::Anchor,"common"),(RelicId::BloodVial,"common"),
    (RelicId::HolyWater,"boss"),(RelicId::VioletLotus,"boss"),(RelicId::SacredBark,"boss"),
    (RelicId::EmptyCage,"boss"),(RelicId::SlaversCollar,"boss"),
];
