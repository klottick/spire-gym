//! Family IDs are split so later content owners can extend disjoint files.
mod cards;
mod encounters;
mod events;
mod items;
pub use cards::CardId;
pub use encounters::{EncounterId, EnemyId};
pub use events::EventId;
pub use items::{PotionId, RelicId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Character { Ironclad, Silent, Defect, Watcher }
impl Character {
    pub const ALL: [Self; 4] = [Self::Ironclad, Self::Silent, Self::Defect, Self::Watcher];
    pub const fn as_str(self) -> &'static str { match self {
        Self::Ironclad => "ironclad", Self::Silent => "silent", Self::Defect => "defect", Self::Watcher => "watcher"
    }}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContentId { Card(CardId), Relic(RelicId), Potion(PotionId), Enemy(EnemyId), Encounter(EncounterId), Event(EventId) }
