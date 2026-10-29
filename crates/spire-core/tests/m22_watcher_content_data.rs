pub mod content {
    pub use spire_core::content::*;
}

use spire_core::run::m22_content;

use content::{CardId, PotionId, RelicId};
use std::collections::HashSet;

fn assert_unique<T: Copy + Eq + std::hash::Hash>(items: &[T]) {
    assert_eq!(items.iter().copied().collect::<HashSet<_>>().len(), items.len());
}

#[test]
fn card_union_is_the_exact_m21_watcher_capability_set() {
    let expected = [
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

    assert_eq!(m22_content::M22_WATCHER_ACT1_CARDS, &expected);
    assert_eq!(
        m22_content::M22_WATCHER_ACT1_CARDS,
        content::watcher::M21_WATCHER_CARDS
    );
    assert_eq!(m22_content::M22_WATCHER_ACT1_CARDS.len(), 28);
    assert_unique(m22_content::M22_WATCHER_ACT1_CARDS);
}

#[test]
fn relic_and_potion_unions_are_exact_ordered_producer_unions() {
    let expected_relics = [
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
    let expected_potions = [
        PotionId::BlockPotion,
        PotionId::EnergyPotion,
        PotionId::DexterityPotion,
        PotionId::AncientPotion,
        PotionId::StancePotion,
        PotionId::Ambrosia,
        PotionId::BottledMiracle,
    ];

    assert_eq!(m22_content::M22_WATCHER_ACT1_RELICS, &expected_relics);
    assert_eq!(m22_content::M22_WATCHER_ACT1_RELICS.len(), 18);
    assert_unique(m22_content::M22_WATCHER_ACT1_RELICS);
    assert_eq!(m22_content::ELITE_RELICS, [
        RelicId::Vajra,
        RelicId::OddlySmoothStone,
        RelicId::GoldenEye,
        RelicId::TeardropLocket,
        RelicId::Lantern,
        RelicId::Pear,
        RelicId::Mango,
        RelicId::ToyOrnithopter,
    ]);
    assert_eq!(m22_content::CHEST_RELICS, [
        RelicId::Strawberry,
        RelicId::Anchor,
        RelicId::BloodVial,
    ]);
    assert_eq!(m22_content::BOSS_RELICS, [
        RelicId::HolyWater,
        RelicId::VioletLotus,
        RelicId::SacredBark,
        RelicId::EmptyCage,
        RelicId::SlaversCollar,
    ]);
    assert_eq!(m22_content::SHOP_RELICS, [RelicId::GoldenEye, RelicId::TeardropLocket]);

    assert_eq!(m22_content::M22_WATCHER_ACT1_POTIONS, &expected_potions);
    assert_eq!(m22_content::POTION_CYCLE, expected_potions);
    assert_eq!(m22_content::SHOP_POTIONS, [PotionId::StancePotion, PotionId::BottledMiracle]);
    assert_eq!(m22_content::M22_WATCHER_ACT1_POTIONS.len(), 7);
    assert_unique(m22_content::M22_WATCHER_ACT1_POTIONS);
}

#[test]
fn card_producer_rows_and_shop_prices_match_the_approved_order() {
    let expected_rewards = [
        [CardId::EmptyFist, CardId::MentalFortress, CardId::CarveReality],
        [CardId::CutThroughFate, CardId::Rushdown, CardId::DeceiveReality],
        [CardId::ThirdEye, CardId::Foresight, CardId::Meditate],
        [CardId::JustLucky, CardId::Nirvana, CardId::Perseverance],
        [CardId::FlurryOfBlows, CardId::Weave, CardId::SandsOfTime],
        [CardId::Prostrate, CardId::Worship, CardId::WindmillStrike],
    ];
    let expected_boss = [CardId::Brilliance, CardId::Devotion, CardId::Establishment];
    let expected_shops = [
        [CardId::BattleHymn, CardId::LikeWater, CardId::MasterReality],
        [CardId::MentalFortress, CardId::Meditate, CardId::Establishment],
    ];

    assert_eq!(m22_content::NONBOSS_REWARD_ROWS, expected_rewards);
    assert_eq!(m22_content::BOSS_CARD_REWARDS, expected_boss);
    assert_eq!(m22_content::SHOP_CARD_ROWS, expected_shops);
    assert_eq!(m22_content::SHOP_CARD_PRICES, [75, 75, 150]);
    assert_eq!(m22_content::shop_cards(false), expected_shops[0]);
    assert_eq!(m22_content::shop_cards(true), expected_shops[1]);
}

#[test]
fn producer_helpers_use_the_full_visit_and_fixed_cycles() {
    let expected_rewards = [
        [CardId::EmptyFist, CardId::MentalFortress, CardId::CarveReality],
        [CardId::CutThroughFate, CardId::Rushdown, CardId::DeceiveReality],
        [CardId::ThirdEye, CardId::Foresight, CardId::Meditate],
        [CardId::JustLucky, CardId::Nirvana, CardId::Perseverance],
        [CardId::FlurryOfBlows, CardId::Weave, CardId::SandsOfTime],
        [CardId::Prostrate, CardId::Worship, CardId::WindmillStrike],
    ];
    let expected_boss = [CardId::Brilliance, CardId::Devotion, CardId::Establishment];
    let expected_potions = [
        PotionId::BlockPotion,
        PotionId::EnergyPotion,
        PotionId::DexterityPotion,
        PotionId::AncientPotion,
        PotionId::StancePotion,
        PotionId::Ambrosia,
        PotionId::BottledMiracle,
    ];

    for visit in 0..12u64 {
        assert_eq!(
            m22_content::reward_cards(false, visit),
            expected_rewards[(visit % 6) as usize],
            "nonboss visit {visit}"
        );
    }
    assert_ne!(m22_content::reward_cards(false, 0), m22_content::reward_cards(false, 3));
    assert_eq!(m22_content::reward_cards(false, 0), m22_content::reward_cards(false, 6));
    for visit in [0, 3, 11, u64::MAX] {
        assert_eq!(m22_content::reward_cards(true, visit), expected_boss);
    }

    for visit in 0..14u64 {
        assert_eq!(
            m22_content::combat_potion(visit),
            expected_potions[(visit % 7) as usize],
            "potion visit {visit}"
        );
    }
    assert_eq!(m22_content::combat_potion(0), m22_content::combat_potion(7));
    assert_eq!(m22_content::reward_cards(false, u64::MAX), expected_rewards[3]);
    assert_eq!(m22_content::reward_cards(false, u64::MAX - 1), expected_rewards[2]);
    assert_eq!(m22_content::combat_potion(u64::MAX), PotionId::EnergyPotion);
    assert_eq!(m22_content::combat_potion(u64::MAX - 1), PotionId::BlockPotion);
}
