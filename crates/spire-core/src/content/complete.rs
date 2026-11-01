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
