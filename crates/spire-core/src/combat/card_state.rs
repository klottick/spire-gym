//! Authoritative persistent identity and combat-local card state. Pile structs
//! remain the legacy location/API adapter; this registry is keyed by instance,
//! and does not maintain a second set of pile memberships.
use super::{CardHandle, CardKind, InputError};
use crate::{combat::{CardInstanceId, CombatCardRef}, content::CardId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatCardData {
    /// Actual mutable native damage, not target-modified displayed damage.
    pub base_damage: Option<i32>,
    /// Actual combat-only upgrades, for exact persistent-exit census. Never
    /// infer permanent growth merely from the card's current upgrade count.
    pub temporary_upgrades:u32,
    pub randomized_cost: Option<i32>,
    /// Persistent in-combat base-cost reduction (Blood for Blood only).
    pub combat_cost_reduction: i32,
    pub temporary_cost: i32,
    /// Infernal Blade's setCostForTurn(0) override; expires at next player start.
    pub infernal_blade_cost_this_turn: bool,
    pub free_play: bool,
    pub x_cost: bool,
    pub exhaust_on_finish: bool,
    pub ethereal: bool,
    /// Innate opening-hand entitlement for the explicitly supported card rows.
    pub innate: bool,
    pub purge: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveCard {
    pub instance: CardInstanceId,
    pub id: CardId,
    pub origin: CombatCardRef,
    pub upgrades: u32,
    pub local: CombatCardData,
    pub copy_family: u64,
    pub copy_provenance: CopyProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyProvenance { Original, Generated, AdapterClone, StatEquivalent, SameInstanceFamily }

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
        upgrades: u32::from(upgraded), local: CombatCardData::default(),copy_family:instance.0,copy_provenance:CopyProvenance::Original }
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
    let native_cost=if card.id==CardId::BloodForBlood&&card.local.randomized_cost.is_none(){
        base_cost.saturating_sub(card.local.combat_cost_reduction)
    }else{
        card.local.randomized_cost.unwrap_or(base_cost)
    };
    let cost = native_cost.checked_add(card.local.temporary_cost)
        .ok_or_else(|| InputError::Invariant("temporary card cost overflow".into()))?;
    if card.local.combat_cost_reduction < 0 {
        return Err(InputError::Invariant("negative combat cost reduction".into()));
    }
    if card.id==CardId::BloodForBlood {return Ok(cost.max(0));}
    if cost<0{return Err(InputError::Invariant("negative effective card cost".into()));}
    Ok(cost)
}

pub fn permanent(id: CardId, instance: CardInstanceId, upgrades: u32) -> LiveCard {
    LiveCard { instance, id, origin: CombatCardRef::Permanent(instance),
         upgrades, local: CombatCardData { base_damage:if id==CardId::SearingBlow{Some(super::cards::ironclad::rules::reconstructed_searing_damage(upgrades))}else if id==CardId::Rampage{Some(8)}else if id==CardId::SeverSoul{Some(if upgrades>0{22}else{16})}else if id==CardId::FiendFire{Some(if upgrades>0{10}else{7})}else{None},exhaust_on_finish: matches!(id,CardId::Disarm|CardId::Impervious|CardId::Offering|CardId::Pummel|CardId::Intimidate|CardId::SeeingRed|CardId::Shockwave|CardId::Warcry|CardId::Exhume|CardId::Feed|CardId::Reaper|CardId::FiendFire|CardId::InfernalBlade|CardId::Catalyst|CardId::Nightmare)||(id==CardId::LimitBreak&&upgrades==0),ethereal:matches!(id,CardId::Carnage|CardId::GhostlyArmor|CardId::Dazed|CardId::Void),innate:id==CardId::Brutality&&upgrades>0,x_cost:id==CardId::Whirlwind, ..CombatCardData::default() },copy_family:instance.0,copy_provenance:CopyProvenance::Original }
}

/// Build a temporary native card with its actual use/disposition state.
pub(super) fn generated(id: CardId, instance: CardInstanceId, upgrades: u32) -> Option<LiveCard> {
    let exhaust_on_finish = match id {
        CardId::SearingBlow=>false,
        CardId::Pummel=>true,
        CardId::Feed|CardId::Reaper=>true,
        CardId::Whirlwind=>false,
        CardId::BodySlam|CardId::Carnage|CardId::Clash|CardId::Hemokinesis|CardId::Uppercut
        |CardId::Immolate|CardId::SwordBoomerang|CardId::WildStrike|CardId::RecklessCharge
        |CardId::Headbutt|CardId::Anger|CardId::IronWave|CardId::Cleave|CardId::PommelStrike
        |CardId::TwinStrike|CardId::Clothesline|CardId::HeavyBlade|CardId::PerfectedStrike
        |CardId::Thunderclap|CardId::Bludgeon=>false,
        CardId::Miracle | CardId::Slimed | CardId::Shiv | CardId::Nightmare => true,
        CardId::Dazed | CardId::Burn | CardId::Wound | CardId::Void => false,
        CardId::LimitBreak=>upgrades==0,
        CardId::Evolve|CardId::FireBreathing|CardId::FlameBarrier|CardId::Juggernaut|CardId::Rage|CardId::Rupture=>false,
        CardId::BloodForBlood|CardId::Flex|CardId::Rampage|CardId::Dropkick|CardId::Brutality
        |CardId::Corruption|CardId::DarkEmbrace|CardId::FeelNoPain|CardId::Sentinel=>false,
        CardId::SecondWind|CardId::SeverSoul=>false,
        CardId::FiendFire|CardId::InfernalBlade=>true,
        CardId::Barricade|CardId::Berserk|CardId::DemonForm|CardId::Metallicize|CardId::Combust=>false,
        _ => return None,
    };
    Some(LiveCard { instance, id, origin: CombatCardRef::CombatLocal(instance.0), upgrades,
        local: CombatCardData { base_damage:if id==CardId::SearingBlow{Some(12)}else if id==CardId::BloodForBlood{Some(if upgrades>0{22}else{18})}else if id==CardId::Feed{Some(if upgrades>0{12}else{10})}else if id==CardId::Reaper{Some(if upgrades>0{5}else{4})}else if id==CardId::Rampage{Some(8)}else if id==CardId::Dropkick{Some(if upgrades>0{8}else{5})}else if id==CardId::SeverSoul{Some(if upgrades>0{22}else{16})}else if id==CardId::FiendFire{Some(if upgrades>0{10}else{7})}else{None},exhaust_on_finish,ethereal:matches!(id,CardId::Carnage|CardId::Dazed|CardId::Void),innate:id==CardId::Brutality&&upgrades>0,x_cost:id==CardId::Whirlwind, ..CombatCardData::default() },copy_family:instance.0,copy_provenance:CopyProvenance::Generated })
}

pub(super) fn generated_with_combat_history(id:CardId,instance:CardInstanceId,upgrades:u32,damage_events:u32)->Option<LiveCard>{
    let mut card=generated(id,instance,upgrades)?;
    if id==CardId::BloodForBlood {for _ in 0..damage_events.min(4){reduce_blood_for_blood_cost(&mut card);}}
    Some(card)
}

pub(super) fn reduce_blood_for_blood_cost(card:&mut LiveCard){
    let base:i32=if card.upgrades>0{3}else{4};
    let current_base_cost=base.saturating_sub(card.local.combat_cost_reduction).max(0);
    // RetainedAbstractCard.updateCost(-1) returns without touching costForTurn
    // when the mutable base cost is already at zero.
    if current_base_cost==0{return}
    let current_turn_cost=card.local.randomized_cost.unwrap_or(current_base_cost)
        .saturating_add(card.local.temporary_cost).max(0);
    card.local.combat_cost_reduction=card.local.combat_cost_reduction.saturating_add(1).min(base);
    // updateCost changes costForTurn by the base delta, independent of the
    // native fields used to represent that turn price.
    let new_turn_cost=current_turn_cost.saturating_sub(1).max(0);
    encode_blood_for_blood_turn_cost(card,new_turn_cost,current_base_cost-1);
}

fn encode_blood_for_blood_turn_cost(card:&mut LiveCard,turn_cost:i32,base_cost:i32){
    if let Some(randomized)=&mut card.local.randomized_cost{
        let current_turn=randomized.saturating_add(card.local.temporary_cost);
        let decrease=(current_turn-turn_cost).max(0);
        let randomized_decrease=decrease.min((*randomized).max(0));
        *randomized-=randomized_decrease;
        card.local.temporary_cost-=decrease-randomized_decrease;
    }else{
        card.local.temporary_cost=turn_cost-base_cost;
    }
}

/// Apply Blood for Blood's source upgrade-cost helper while preserving the
/// adapter's representation of the resulting current-turn price. This is
/// intentionally card-specific: its mutable base cost and costForTurn can
/// differ, and either can be represented by the randomized or temporary field.
pub(super) fn upgrade_blood_for_blood_cost(card:&mut LiveCard){
    let old_base=(4-card.local.combat_cost_reduction).max(0);
    let old_cost_for_turn=card.local.randomized_cost.unwrap_or(old_base)
        .saturating_add(card.local.temporary_cost).max(0);
    // free_play only waives payment; it does not participate in source cost
    // updateCost/updateBaseCost arithmetic.
    let old_turn=old_cost_for_turn;
    let requested_base=if old_base<4{old_base-1}else{3};
    let helper_turn=if old_turn>0{
        (requested_base.saturating_add(old_turn-old_base)).max(0)
    }else{old_turn};
    let new_base=requested_base.max(0);
    card.local.combat_cost_reduction=3-new_base;

    encode_blood_for_blood_turn_cost(card,helper_turn,new_base);
}

impl LiveCard {
    /// Construct an independent combat copy with equivalent card state.
    pub fn copy_as(&self, instance: CardInstanceId) -> Self {
        Self { instance, id: self.id, origin: CombatCardRef::CombatLocal(instance.0),
            upgrades: self.upgrades, local: self.local.clone(),copy_family:instance.0,copy_provenance:CopyProvenance::AdapterClone }
    }

    pub fn stat_equivalent_copy(&self,instance:CardInstanceId)->Self{
        let mut local=self.local.clone();
        // Fresh-kind constructor and replayed upgrades own disposition flags.
        let defaults=generated(self.id,instance,self.upgrades).map(|c|c.local).unwrap_or_else(||permanent(self.id,instance,self.upgrades).local);
        local.exhaust_on_finish=defaults.exhaust_on_finish;local.ethereal=defaults.ethereal;
        local.x_cost=defaults.x_cost;local.innate=defaults.innate;
        local.temporary_upgrades=0;
        Self{instance,id:self.id,origin:CombatCardRef::CombatLocal(instance.0),upgrades:self.upgrades,local,
            copy_family:instance.0,copy_provenance:CopyProvenance::StatEquivalent}
    }

    pub fn same_instance_family_copy(&self,instance:CardInstanceId)->Self{
        let mut copy=self.stat_equivalent_copy(instance);copy.copy_family=self.copy_family;copy.copy_provenance=CopyProvenance::SameInstanceFamily;copy
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
        assert_eq!(copy.copy_provenance,CopyProvenance::AdapterClone);
        assert_eq!(copy.local,source.local,"legacy adapter preserves its established local clone behavior");
        assert_ne!(copy.instance, source.instance);
    }
}
