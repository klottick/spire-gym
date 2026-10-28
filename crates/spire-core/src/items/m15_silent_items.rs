use crate::content::{Catalog, Character, PotionId, RelicId};

#[test]
fn m15_silent_items_register_source_named_exclusives() {
    let relics = [
        (RelicId::NinjaScroll, "ninja_scroll"),
        (RelicId::PaperKrane, "paper_krane"),
        (RelicId::TheSpecimen, "the_specimen"),
        (RelicId::Tingsha, "tingsha"),
        (RelicId::ToughBandages, "tough_bandages"),
        (RelicId::TwistedFunnel, "twisted_funnel"),
        (RelicId::HoveringKite, "hovering_kite"),
        (RelicId::RingOfTheSerpent, "ring_of_the_serpent"),
        (RelicId::WristBlade, "wrist_blade"),
    ];
    for (id, source_key) in relics {
        assert_eq!(id.as_str(), source_key, "native item ID must retain source key {source_key}");
        assert!(Catalog::standard().relic(id).is_some(), "catalog must register source relic {source_key}");
    }
    for (id, source_key) in [
        (PotionId::PoisonPotion, "poison_potion"),
        (PotionId::CunningPotion, "cunning_potion"),
        (PotionId::GhostInAJar, "ghost_in_a_jar"),
    ] {
        assert_eq!(id.as_str(), source_key, "native potion ID must retain source key {source_key}");
        assert!(Catalog::standard().potion(id).is_some(), "catalog must register source potion {source_key}");
    }

    let relic_tiers = [
        (RelicId::RingOfTheSnake, crate::content::RelicTier::Starter),
        (RelicId::SneckoSkull, crate::content::RelicTier::Common),
        (RelicId::NinjaScroll, crate::content::RelicTier::Uncommon),
        (RelicId::PaperKrane, crate::content::RelicTier::Uncommon),
        (RelicId::TheSpecimen, crate::content::RelicTier::Rare),
        (RelicId::Tingsha, crate::content::RelicTier::Rare),
        (RelicId::ToughBandages, crate::content::RelicTier::Rare),
        (RelicId::TwistedFunnel, crate::content::RelicTier::Shop),
        (RelicId::HoveringKite, crate::content::RelicTier::Boss),
        (RelicId::RingOfTheSerpent, crate::content::RelicTier::Boss),
        (RelicId::WristBlade, crate::content::RelicTier::Boss),
    ];
    for (id, tier) in relic_tiers {
        assert_eq!(Catalog::standard().relic_tier(id), Some(tier), "source rarity for {id:?}");
    }
    for (id, rarity) in [
        (PotionId::PoisonPotion, crate::content::PotionRarity::Common),
        (PotionId::CunningPotion, crate::content::PotionRarity::Uncommon),
        (PotionId::GhostInAJar, crate::content::PotionRarity::Rare),
    ] {
        assert_eq!(Catalog::standard().potion_rarity(id), Some(rarity), "source rarity for {id:?}");
        assert_eq!(Catalog::standard().potion_eligible_for(id, Character::Silent), Some(true));
        assert_eq!(Catalog::standard().potion_eligible_for(id, Character::Ironclad), Some(false));
    }
}
