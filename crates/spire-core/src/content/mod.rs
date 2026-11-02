//! Authored run-content identities and route metadata.
//!
//! This is a definition registry, not an executable mechanics claim. Combat-v1
//! keeps its original `combat::CONTENT_ID` and content vocabulary untouched.
mod ids;
mod cards;
mod items;
mod encounters;
mod events;
pub mod complete;
pub mod defect;
pub mod pools;
pub mod watcher;

pub use cards::{
    BlueCardMetadata, CardColor, CardCost, CardDefinition, CardFaceMetadata, CardKeywords,
    CardRarity, CardTag, CardTarget, CardType, ContentProducer, NoteStorage, RegistryError,
    card_tags, is_healing, is_strike, plays_itself_at_end_of_turn,
};
pub use encounters::{EncounterDefinition, EnemyDefinition};
pub use events::{EventDefinition, EventPool};
pub use ids::{CardId, Character, ContentId, EncounterId, EnemyId, EventId, PotionId, RelicId};
pub use items::{PotionDefinition, PotionRarity, RelicDefinition, RelicTier};

/// Canonical static catalog. Missing entries remain missing; lookups never
/// substitute starter content.
pub struct Catalog {
    cards: &'static [CardDefinition],
    relics: &'static [RelicDefinition],
    potions: &'static [PotionDefinition],
    enemies: &'static [EnemyDefinition],
    encounters: &'static [EncounterDefinition],
    events: &'static [EventDefinition],
    starters: &'static [(Character, &'static [CardId])],
    starter_relics: &'static [(Character, RelicId)],
}

impl Catalog {
    pub fn standard() -> &'static Self { &STANDARD }
    pub fn card(&self, id: CardId) -> Option<&'static CardDefinition> {
        self.cards.iter().find(|definition| definition.id == id)
    }
    pub fn starter_deck(&self, character: Character) -> Result<&'static [CardId], RegistryError> {
        starter_deck_from(self.starters, character)
    }
    pub fn starter_relic(&self, character: Character) -> Option<RelicId> {
        self.starter_relics.iter().find(|(owner, _)| *owner == character).map(|(_, relic)| *relic)
    }
    pub fn cards(&self) -> &'static [CardDefinition] { self.cards }
    pub fn eligibility(&self, id: CardId, producer: ContentProducer, character: Character) -> Result<bool, RegistryError> {
        self.card(id).ok_or(RegistryError::UnknownDefinition)?.eligible_for(producer, character)
    }
    /// The complete selector domain of a producer, read from the pool owner
    /// (M26 G2; design §7.2). An incomplete family fails closed there.
    pub fn selectable_pool(&self, producer: ContentProducer, character: Character) -> Result<Vec<CardId>, RegistryError> {
        match producer {
            ContentProducer::OrdinaryReward => pools::cards(pools::CardPool::Ordinary { character, rarity: None, card_type: None }),
            ContentProducer::ShardOrdinaryReward => pools::cards(pools::CardPool::AnyColour { rarity: None }),
            ContentProducer::NoteForYourself => pools::cards(pools::CardPool::NoteForYourself),
            ContentProducer::PureWater => Ok(vec![CardId::Miracle]),
            ContentProducer::Unregistered(_) => Err(RegistryError::UnknownProducer),
        }
    }
    pub fn relic(&self, id: RelicId) -> Option<&'static RelicDefinition> { self.relics.iter().find(|x| x.id == id) }
    pub fn relic_tier(&self, id: RelicId) -> Option<RelicTier> {
        items::RELIC_TIERS.iter().find(|(candidate, _)| *candidate == id).map(|(_, tier)| *tier)
    }
    /// Character eligibility for a registered relic. `None` means the ID has no definition;
    /// relics without an exclusive-character row are shared by all characters.
    pub fn relic_eligible_for(&self, id: RelicId, character: Character) -> Option<bool> {
        self.relic(id)?;
        Some(items::EXCLUSIVE_RELIC_CHARACTERS.iter().find(|(candidate, _)| *candidate == id)
            .is_none_or(|(_, owner)| *owner == character))
    }
    pub fn potion(&self, id: PotionId) -> Option<&'static PotionDefinition> { self.potions.iter().find(|x| x.id == id) }
    pub fn potion_rarity(&self, id: PotionId) -> Option<PotionRarity> {
        items::POTION_RARITIES.iter().find(|(candidate, _)| *candidate == id).map(|(_, rarity)| *rarity)
    }
    /// Character eligibility for a registered potion definition. `None` means
    /// the ID has no definition; shared potions are eligible for all characters.
    pub fn potion_eligible_for(&self,id:PotionId,character:Character)->Option<bool>{
        self.potion(id)?;
        Some(items::EXCLUSIVE_POTION_CHARACTERS.iter().find(|(candidate,_)|*candidate==id)
            .is_none_or(|(_,owner)|*owner==character))
    }
    pub fn enemy(&self, id: EnemyId) -> Option<&'static EnemyDefinition> { self.enemies.iter().find(|x| x.id == id) }
    pub fn encounter(&self, id: EncounterId) -> Option<&'static EncounterDefinition> { self.encounters.iter().find(|x| x.id == id) }
    pub fn event(&self, id: EventId) -> Option<&'static EventDefinition> { self.events.iter().find(|x| x.id == id) }
    pub fn events(&self) -> &'static [EventDefinition] { self.events }
}

fn starter_deck_from(
    starters: &'static [(Character, &'static [CardId])],
    character: Character,
) -> Result<&'static [CardId], RegistryError> {
    starters.iter().find(|(owner, _)| *owner == character)
        .map(|(_, deck)| *deck)
        .ok_or(RegistryError::MissingStarterDeck)
}

static STANDARD: Catalog = Catalog { cards: cards::CARDS, relics: items::RELICS, potions: items::POTIONS,
    enemies: encounters::ENEMIES, encounters: encounters::ENCOUNTERS, events: events::EVENTS,
    starters: cards::STARTERS, starter_relics: items::STARTER_RELICS };

#[cfg(test)]
mod tests {
    use super::{starter_deck_from, Character, RegistryError};

    #[test]
    fn missing_starter_mapping_is_not_an_empty_valid_deck() {
        assert_eq!(starter_deck_from(&[], Character::Silent), Err(RegistryError::MissingStarterDeck));
    }
}
