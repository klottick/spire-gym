//! M25 Watcher A0 Acts 1–4 and Heart: the M24 producers in Acts 1–3 (no row is copied) and the
//! Act4 merchant and elite laws of design §6.2 on the shared staged sampler. The unions are
//! built from `content::watcher`'s complete roster and the M24 union.

use super::*;
use crate::content::{CardColor, CardRarity, Catalog};
use super::m24_content as m24;

const fn extend<T: Copy, const N: usize>(base: &[T], extra: &[T]) -> [T; N] {
    let mut out = [extra[0]; N];
    let mut i = 0;
    while i < base.len() { out[i] = base[i]; i += 1; }
    let mut j = 0;
    while j < extra.len() { out[base.len() + j] = extra[j]; j += 1; }
    out
}
const CARDS: [CardId; crate::content::watcher::M25_WATCHER_CARDS.len() + 1] = extend(crate::content::watcher::M25_WATCHER_CARDS, &[CardId::Parasite]);
/// The four new Watcher-exclusive relics (design §5.4), in union order.
pub const NEW_RELICS: [RelicId; 4] = [RelicId::Damaru, RelicId::Duality, RelicId::CloakClasp, RelicId::Melange];
const RELICS: [RelicId; m24::M24_WATCHER_ACT3_RELICS.len() + 4] = extend(m24::M24_WATCHER_ACT3_RELICS, &NEW_RELICS);

/// The 75 ordinary Watcher cards plus Parasite (design §5.6).
pub const M25_WATCHER_HEART_CARDS: &[CardId] = &CARDS;
/// M24's 19 relic kinds followed by Damaru, Duality, CloakClasp and Melange.
pub const M25_WATCHER_HEART_RELICS: &[RelicId] = &RELICS;
/// M22's seven potion kinds, unchanged.
pub const M25_WATCHER_HEART_POTIONS: &[PotionId] = m24::M24_WATCHER_ACT3_POTIONS;
/// The eight combat-generated Watcher children; never permanent acquisitions.
pub const GENERATED_ONLY: &[CardId] = crate::content::watcher::M25_WATCHER_GENERATED;
/// Choice-only rows (Wish).
pub const CHOICES: &[CardId] = crate::content::watcher::M25_WATCHER_CHOICES;
/// Same five statuses as M24.
pub const STATUSES: &[CardId] = m24::STATUSES;
/// Descriptor taxonomy rows added after the M22 taxonomy and M24's additions.
pub const RELIC_TAXONOMY_ADDITIONS: &[(RelicId, &str)] = &[(RelicId::Damaru, "common"), (RelicId::Duality, "uncommon"), (RelicId::CloakClasp, "rare"), (RelicId::Melange, "shop")];

/// Floor-53 elite relic pool, in capture order.
pub const ACT4_ELITE_RELICS: [RelicId; 3] = [RelicId::Damaru, RelicId::Duality, RelicId::CloakClasp];
pub const ACT4_SHOP_RELIC: RelicId = RelicId::Melange;
pub const ACT4_SHOP_RELIC_PRICE: u32 = 150;
pub const ACT4_SHOP_CARD_PRICES: [u32; 3] = [50, 75, 150];
pub const ACT4_SHOP_POTION: PotionId = PotionId::StancePotion;
pub const ACT4_SHOP_POTION_PRICE: u32 = 50;
pub const CARD_OFFER_SCOPE: crate::run::RunRngScope = crate::run::RunRngScope::M25CardOffer;
pub const CARD_OFFER_SCOPE_NAME: &str = "run.m25.card.offer";
pub const ACT4_RELIC_CAPTURE_SCOPE: crate::run::RunRngScope = crate::run::RunRngScope::M25RelicCapture;
pub const ACT4_RELIC_CAPTURE_SCOPE_NAME: &str = "run.m25.relic.capture";

/// Nonbasic Watcher catalog cards of one rarity, in spelling order (the M20 family pattern).
pub(in crate::run) fn purple_candidates(rarity: CardRarity) -> &'static [CardId] {
    static FAMILIES: std::sync::OnceLock<[Vec<CardId>; 3]> = std::sync::OnceLock::new();
    let families = FAMILIES.get_or_init(|| [CardRarity::Common, CardRarity::Uncommon, CardRarity::Rare].map(|rarity| {
        let mut cards = Catalog::standard().cards().iter().filter(|row| row.color == CardColor::Purple && row.rarity == rarity)
            .map(|row| row.id).collect::<Vec<_>>();
        cards.sort_by_key(|id| id.as_str());
        cards
    }));
    match rarity { CardRarity::Common => &families[0], CardRarity::Uncommon => &families[1], CardRarity::Rare => &families[2], _ => &[] }
}
pub(in crate::run) fn sample_capture_row(rng: &mut RunRng, semantic: &[u64], rarities: &[CardRarity]) -> Result<Vec<CardId>, RunError> {
    sample_capture_row_with(rng, semantic, rarities, purple_candidates)
}
/// Before any RNG request, each family must have exactly its design size (19/35/17); the
/// shared sampler then rejects an empty family or a definition-only kind.
pub(in crate::run) fn sample_capture_row_with(rng: &mut RunRng, semantic: &[u64], rarities: &[CardRarity], family: fn(CardRarity) -> &'static [CardId]) -> Result<Vec<CardId>, RunError> {
    for rarity in rarities {
        let expected = match rarity { CardRarity::Common => 19, CardRarity::Uncommon => 35, CardRarity::Rare => 17, _ => 0 };
        if expected == 0 || family(*rarity).len() != expected { return Err(RunError::Unsupported("M25 incomplete Purple rarity family")) }
    }
    super::m15_content::sample_capture_row_for(rng, semantic, rarities, family, CARD_OFFER_SCOPE,
        "M25 incomplete Purple rarity family", "M25 captured card row has no remaining Purple candidate")
}

/// Floor-52 merchant rows (design §6.2); the shared Act4 room owner commits `rng` once.
pub(in crate::run) fn act4_shop_rows(run: &Run, rng: &mut RunRng, floor: u32, node: NodeId) -> Result<Vec<(RoomChoiceKind, Option<u32>)>, RunError> {
    let cards = sample_capture_row(rng, &[4, u64::from(floor), node.0], &[CardRarity::Common, CardRarity::Uncommon, CardRarity::Rare])?;
    let mut rows = cards.into_iter().zip(ACT4_SHOP_CARD_PRICES).map(|(id, price)| (RoomChoiceKind::Card { id, upgrades: 0 }, Some(price))).collect::<Vec<_>>();
    if !run.state.relics.contains(&ACT4_SHOP_RELIC) { rows.push((RoomChoiceKind::Relic(ACT4_SHOP_RELIC), Some(ACT4_SHOP_RELIC_PRICE))); }
    let purge = run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.purge_cost;
    rows.extend([(RoomChoiceKind::Potion(ACT4_SHOP_POTION), Some(ACT4_SHOP_POTION_PRICE)), (RoomChoiceKind::Purge, Some(purge)), (RoomChoiceKind::Leave, None)]);
    Ok(rows)
}
