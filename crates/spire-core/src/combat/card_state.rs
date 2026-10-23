//! Authoritative persistent identity and combat-local card state. Pile structs
//! remain the legacy location/API adapter; this registry is keyed by instance,
//! and does not maintain a second set of pile memberships.
use super::{CardHandle, CardKind, InputError};
use crate::{combat::{CardInstanceId, CombatCardRef}, content::CardId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatCardData {
    pub randomized_cost: Option<i32>,
    pub temporary_cost: i32,
    pub free_play: bool,
    pub x_cost: bool,
    pub exhaust_on_finish: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveCard {
    pub instance: CardInstanceId,
    pub id: CardId,
    pub origin: CombatCardRef,
    pub upgrades: u32,
    pub local: CombatCardData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Payment { pub remaining_energy: i32, pub paid: i32 }

pub(super) fn handle(instance: CardInstanceId) -> CardHandle {
    CardHandle::parse(&instance.0.to_string()).expect("numeric card instance is canonical")
}

pub(super) fn legacy_identity(kind: CardKind, local_id: u64, upgraded: bool) -> LiveCard {
    let id = card_id(kind);
    let instance = CardInstanceId(local_id);
    LiveCard { instance, id,
        origin: CombatCardRef::CombatLocal(instance.0),
        upgrades: u32::from(upgraded), local: CombatCardData::default() }
}

pub(super) fn card_id(kind: CardKind) -> CardId {
    match kind { CardKind::Strike => CardId::Strike, CardKind::Defend => CardId::Defend,
        CardKind::Bash => CardId::Bash }
}

pub(super) fn legacy_kind(id: CardId) -> Option<CardKind> {
    match id { CardId::Strike => Some(CardKind::Strike), CardId::Defend => Some(CardKind::Defend), CardId::Bash => Some(CardKind::Bash), _ => None }
}

pub(super) fn payment(card: &LiveCard, energy: i32, base_cost: i32) -> Result<Payment, InputError> {
    if card.local.free_play { return Ok(Payment { remaining_energy: energy, paid: 0 }); }
    if card.local.x_cost { return Ok(Payment { remaining_energy: 0, paid: energy }); }
    let cost = effective_cost(card,base_cost)?;
    if energy < cost { return Err(InputError::Unsupported("insufficient energy".into())); }
    Ok(Payment { remaining_energy: energy - cost, paid: cost })
}

pub(super) fn effective_cost(card:&LiveCard,base_cost:i32)->Result<i32,InputError>{
    if card.local.free_play{return Ok(0)}
    let cost = card.local.randomized_cost.unwrap_or(base_cost).checked_add(card.local.temporary_cost)
        .ok_or_else(|| InputError::Invariant("temporary card cost overflow".into()))?;
    if cost < 0 { return Err(InputError::Invariant("negative effective card cost".into())); }
    Ok(cost)
}

pub fn permanent(id: CardId, instance: CardInstanceId, upgrades: u32) -> LiveCard {
    LiveCard { instance, id, origin: CombatCardRef::Permanent(instance),
        upgrades, local: CombatCardData { exhaust_on_finish: matches!(id,CardId::Disarm|CardId::Impervious|CardId::Offering), ..CombatCardData::default() } }
}

/// Build a temporary native card with its actual use/disposition state.
pub(super) fn generated(id: CardId, instance: CardInstanceId, upgrades: u32) -> Option<LiveCard> {
    let exhaust_on_finish = match id {
        CardId::Miracle | CardId::Slimed | CardId::Shiv => true,
        CardId::Dazed | CardId::Burn | CardId::Wound => false,
        _ => return None,
    };
    Some(LiveCard { instance, id, origin: CombatCardRef::CombatLocal(instance.0), upgrades,
        local: CombatCardData { exhaust_on_finish, ..CombatCardData::default() } })
}

impl LiveCard {
    /// Construct an independent combat copy with equivalent card state.
    pub fn copy_as(&self, instance: CardInstanceId) -> Self {
        Self { instance, id: self.id, origin: CombatCardRef::CombatLocal(instance.0),
            upgrades: self.upgrades, local: self.local.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copied_card_keeps_persistent_upgrade_without_aliasing_instance_state() {
        let source = permanent(CardId::Strike, CardInstanceId(4), 2);
        let copy = source.copy_as(CardInstanceId(9));

        assert_eq!(source.instance, CardInstanceId(4));
        assert_eq!(copy.instance, CardInstanceId(9));
        assert_eq!(copy.id, source.id);
        assert_eq!(copy.upgrades, 2);
        assert_eq!(copy.origin, CombatCardRef::CombatLocal(9));
        assert_ne!(copy.instance, source.instance);
    }
}
