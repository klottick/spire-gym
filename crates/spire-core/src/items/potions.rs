use crate::{
    combat::{Actor, DamagePayload, Effect, EffectKind, EnemyHandle, Source},
    content::PotionId,
    run::RunError,
};

use super::{CombatItemContext, NativePotionUseContext, PotionRoomPhase, PotionUseEffects};

pub(super) fn use_potion(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>) -> Result<PotionUseEffects, RunError> {
    let potion = context.potions.get(slot).copied().flatten().ok_or(RunError::InvalidAction)?;
    if facts.we_meet_again { return Err(RunError::InvalidAction); }
    if facts.max_hp == 0 || facts.max_hp > 999 || facts.hp > facts.max_hp {
        return Err(RunError::InvalidAction);
    }
    let target_required = matches!(potion, PotionId::FirePotion | PotionId::WeakPotion | PotionId::FearPotion);
    if !target_required && target.is_some() {
        return Err(RunError::InvalidAction);
    }
    let effect = match potion {
        PotionId::Elixir => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended || facts.live_targets.is_empty() {
                return Err(RunError::InvalidAction);
            }
            Effect { source: Source::Potion { slot, id: potion }, target: Some(Actor::Player),
                kind: EffectKind::SelectHandCardsAfterDraw { required: 0,
                    disposition: crate::combat::SelectionDisposition::Exhaust,
                    empty: crate::combat::EmptySelection::Continue } }
        }
        PotionId::HeartOfIron => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended || facts.live_targets.is_empty() {
                return Err(RunError::InvalidAction);
            }
            Effect { source: Source::Potion { slot, id: potion }, target: Some(Actor::Player),
                kind: EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("metallicize"), amount:
                    if context.owns_relic(crate::content::RelicId::SacredBark) { 12 } else { 6 } } }
        }
        PotionId::FirePotion => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended {
                return Err(RunError::InvalidAction);
            }
            let target = target.ok_or(RunError::InvalidAction)?;
            if !facts.live_targets.contains(&target) { return Err(RunError::InvalidAction); }
            let amount = if context.owns_relic(crate::content::RelicId::SacredBark) { 40 } else { 20 };
            Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(target)),
                kind: EffectKind::Damage(DamagePayload::Thorns(amount)) }
        }
        PotionId::BloodPotion => {
            if facts.phase == PotionRoomPhase::Combat && facts.turn_ended {
                return Err(RunError::InvalidAction);
            }
            let percent = if context.owns_relic(crate::content::RelicId::SacredBark) { 40.0_f32 } else { 20.0_f32 };
            let fraction = percent / 100.0_f32;
            let amount = ((facts.max_hp as f32) * fraction) as i32;
            Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                kind: EffectKind::Heal(amount) }
        }
        PotionId::FruitJuice => {
            if facts.phase == PotionRoomPhase::Combat && facts.turn_ended {
                return Err(RunError::InvalidAction);
            }
            let amount: i32 = if context.owns_relic(crate::content::RelicId::SacredBark) { 10 } else { 5 };
            let hp_gain = u32::try_from(amount).map_err(|_| RunError::InvalidAction)?;
            if facts.max_hp.checked_add(hp_gain).map_or(true, |max_hp| max_hp > 999) {
                return Err(RunError::InvalidAction);
            }
            Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                kind: EffectKind::IncreaseMaxHpThenHeal(amount) }
        }
        PotionId::BlockPotion | PotionId::EnergyPotion | PotionId::StrengthPotion
        | PotionId::SwiftPotion | PotionId::WeakPotion | PotionId::FearPotion
        | PotionId::DexterityPotion | PotionId::AncientPotion => {
            if facts.phase != PotionRoomPhase::Combat || facts.turn_ended || facts.live_targets.is_empty() {
                return Err(RunError::InvalidAction);
            }
            let bark = context.owns_relic(crate::content::RelicId::SacredBark);
            let actor_target = match potion {
                PotionId::WeakPotion | PotionId::FearPotion => {
                    let enemy = target.ok_or(RunError::InvalidAction)?;
                    if !facts.live_targets.contains(&enemy) { return Err(RunError::InvalidAction); }
                    Actor::Enemy(enemy)
                }
                _ => Actor::Player,
            };
            let kind = match potion {
                PotionId::BlockPotion => EffectKind::GainBlock(if bark { 24 } else { 12 }),
                PotionId::EnergyPotion => EffectKind::GainEnergy(if bark { 4 } else { 2 }),
                PotionId::StrengthPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("strength"), amount: if bark { 4 } else { 2 } },
                PotionId::DexterityPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("dexterity"), amount: if bark { 4 } else { 2 } },
                PotionId::AncientPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("artifact"), amount: if bark { 2 } else { 1 } },
                PotionId::SwiftPotion => EffectKind::Draw(if bark { 6 } else { 3 }),
                PotionId::WeakPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("weak"), amount: if bark { 6 } else { 3 } },
                PotionId::FearPotion => EffectKind::ApplyFullPower {
                    id: crate::combat::FullPowerId("vulnerable"), amount: if bark { 6 } else { 3 } },
                _ => unreachable!("finite combat potion arm"),
            };
            Effect { source: Source::Actor(Actor::Player), target: Some(actor_target), kind }
        }
        PotionId::Unknown(_) => return Err(RunError::Unsupported("unimplemented potion effect")),
    };
    let mut potions = context.potions.clone();
    potions[slot] = None;
    Ok(PotionUseEffects { effects: vec![effect], potions })
}
