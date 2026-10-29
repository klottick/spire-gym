//! Shared command identities for the run and combat boundaries.
use crate::content::{CardId, EnemyId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TargetId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombatAction {
    ChooseStance { stance:super::Stance },
    DeselectCard { instance:u64 },
    PlayCard { instance: u64, target: Option<TargetId> },
    SelectCard { instance: u64 },
    ConfirmSelection,
    UsePotion { slot: usize, target: Option<TargetId> },
    EndTurn,
    ChooseCardOffer { index:u32 },
    SkipCardOffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardInstanceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardDestination { Hand, Draw, Discard, ShuffledDraw, BottomDraw }
pub type GeneratedCardDestination = CardDestination;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombatCardRef { Permanent(CardInstanceId), CombatLocal(u64) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnemyTarget { pub id: EnemyId }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PermanentCard { pub id: CardId, pub instance: CardInstanceId }
