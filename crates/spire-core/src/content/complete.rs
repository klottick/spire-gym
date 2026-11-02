//! M26 complete card capability unions (design §4.4, §5.1).
//!
//! Computed from the catalog rows (availability and note storage), never a
//! literal list. They define construction admission only, not any reward or
//! generator pool. The finite M10–M25 lists stay their profiles' laws.

use super::cards::NoteStorage;
use super::{CardId, Catalog};

/// Wish's three choice options: offered, never physical cards.
const CHOICE_ONLY: [CardId; 3] = [CardId::BecomeAlmighty, CardId::FameAndFortune, CardId::LiveForever];

fn executable(id: CardId) -> Option<NoteStorage> {
    Catalog::standard().card(id).filter(|definition| !definition.definition_only).map(|definition| definition.note_storage)
}

/// Deck-legal kinds: every executable row a deck can hold (352; never Pride).
pub fn supports_permanent(id: CardId) -> bool {
    executable(id).is_some_and(|storage| storage != NoteStorage::TemporaryOnly)
}

/// Kinds that may exist as physical combat cards: permanent or generated-only
/// (the 9 generated children and 5 statuses). Choice-only options never are.
pub fn supports_generated(id: CardId) -> bool {
    executable(id).is_some() && !is_choice_only(id)
}

pub fn is_choice_only(id: CardId) -> bool {
    CHOICE_ONLY.contains(&id) && executable(id).is_some()
}

/// The purgeable-card predicate for remove and transform selections.
pub fn is_removable(id: CardId) -> bool {
    executable(id).is_some_and(|storage| storage == NoteStorage::RemovablePermanent)
}

// M27 complete item capability unions (design §4.4, §5.1): computed from the catalog
// rows (catalog order) and the one class table, never literal lists.

/// Every ordinary executable relic row (179); Red Circlet is fallback-only under F7.
pub fn supports_relic(id: super::RelicId) -> bool {
    !matches!(id, super::RelicId::Unknown(_) | super::RelicId::RedCirclet)
        && Catalog::standard().relic(id).is_some_and(|definition| !definition.definition_only)
}

/// The relics a character may hold: the shared rows plus its own class rows.
pub fn relic_union(character: super::Character) -> Vec<super::RelicId> {
    super::items::RELICS.iter().map(|definition| definition.id)
        .filter(|id| supports_relic(*id) && Catalog::standard().relic_eligible_for(*id, character) == Some(true))
        .collect()
}

/// Every executable potion row (42).
pub fn supports_potion(id: super::PotionId) -> bool {
    !matches!(id, super::PotionId::Unknown(_))
        && Catalog::standard().potion(id).is_some_and(|definition| !definition.definition_only)
}

/// The potions a character may hold: the 30 shared rows plus its own three.
pub fn potion_union(character: super::Character) -> Vec<super::PotionId> {
    super::items::POTIONS.iter().map(|definition| definition.id)
        .filter(|id| supports_potion(*id) && Catalog::standard().potion_eligible_for(*id, character) == Some(true))
        .collect()
}
