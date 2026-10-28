//! Per-instance cost, draw-copy, and family-damage callbacks for the bounded
//! Silent cost/copy slice.
use super::*;
use super::state::SilentEffect;
use crate::combat::model::OwnedCard;
use crate::content::CardId;

pub(super) fn starting_cost(id: CardId, upgrades: u32) -> Option<i32> {
    Some(match id {
        CardId::MasterfulStab | CardId::EndlessAgony | CardId::GrandFinale => 0,
        CardId::Eviscerate => 3,
        CardId::GlassKnife => 1,
        CardId::Skewer | CardId::Doppelganger | CardId::Malaise => -1,
        CardId::Burst => 1,
        CardId::StormOfSteel => 1,
        CardId::Setup => if upgrades > 0 { 0 } else { 1 },
        CardId::BulletTime => if upgrades > 0 { 2 } else { 3 },
        _ => return None,
    })
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use CardId::*;
    if !matches!(card.id, MasterfulStab | Eviscerate | Setup | BulletTime | EndlessAgony | GlassKnife) {
        return Ok(None);
    }
    let attack = matches!(card.id, MasterfulStab | Eviscerate | EndlessAgony | GlassKnife);
    if attack != target.is_some() { return Err(crate::run::RunError::InvalidAction); }
    let upgraded = card.upgrades > 0;
    let mut effects = Vec::new();
    let cost = match card.id {
        MasterfulStab | EndlessAgony | Eviscerate | GlassKnife => {
            let base = card.local.base_damage.unwrap_or(match (card.id, upgraded) {
                (MasterfulStab, false) => 12, (MasterfulStab, true) => 16,
                (Eviscerate, false) => 7, (Eviscerate, true) => 9,
                (EndlessAgony, false) => 4, (EndlessAgony, true) => 6,
                (GlassKnife, false) => 8, (GlassKnife, true) => 12,
                _ => unreachable!(),
            });
            let damage = super::super::modified_damage(state, target.unwrap(), base)?;
            let hits = match card.id { Eviscerate => 3, GlassKnife => 2, _ => 1 };
            for _ in 0..hits {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            if card.id == GlassKnife {
                effects.push((None, EffectKind::Silent(SilentEffect::GlassKnifeAfterHits)));
            }
            starting_cost(card.id, card.upgrades).expect("cost card has a starting cost")
        }
        Setup => {
            effects.push((None, EffectKind::Silent(SilentEffect::Setup)));
            starting_cost(card.id, card.upgrades).unwrap()
        }
        BulletTime => {
            effects.push((Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
                id: FullPowerId("no_draw"), amount: 1,
            }));
            effects.push((None, EffectKind::Silent(SilentEffect::BulletTime)));
            starting_cost(card.id, card.upgrades).unwrap()
        }
        _ => unreachable!(),
    };
    Ok(Some((cost, effects)))
}

fn native_discard_count(count: u32) -> Result<i32, InputError> {
    i32::try_from(count).map_err(|_| InputError::Invariant(
        "Silent discard history exceeds the source signed counter domain".into()))
}

fn eviscerate_turn_price(combat_cost: i32, discarded: i32) -> Result<i32, InputError> {
    if combat_cost < 0 || discarded < 0 {
        return Err(InputError::Invariant("negative Eviscerate price/history input".into()));
    }
    combat_cost.checked_sub(discarded).map(|price| price.max(0))
        .ok_or_else(|| InputError::Unsupported("Eviscerate turn-cost arithmetic overflow".into()))
}

pub(super) fn before_make_copy(source: &mut LiveCard, history: super::super::CopyHistory) -> Result<(), InputError> {
    if history.full_runtime && source.id == CardId::Eviscerate {
        let discarded = native_discard_count(history.discarded_this_turn)?;
        if source.local.turn_cost.is_some_and(|turn_cost| turn_cost < 0) { return Ok(()); }
        let combat_cost = source.local.mutable_combat_cost
            .or_else(|| starting_cost(source.id, source.upgrades))
            .ok_or_else(|| InputError::Invariant("Eviscerate source cost is absent".into()))?;
        source.local.turn_cost = Some(eviscerate_turn_price(combat_cost, discarded)?);
    }
    Ok(())
}

pub(super) fn player_hp_lost(state: &mut State) -> Result<(), InputError> {
    for owned in state.piles.hand.iter_mut().chain(&mut state.piles.draw).chain(&mut state.piles.discard) {
        if let Some(card) = owned.full_mut().filter(|card| card.id == CardId::MasterfulStab) {
            update_combat_cost(card, 1)?;
        }
    }
    Ok(())
}

pub(super) fn manual_discard(state: &mut State) -> Result<(), InputError> {
    for owned in state.piles.hand.iter_mut().chain(&mut state.piles.discard).chain(&mut state.piles.draw) {
        if let Some(card) = owned.full_mut().filter(|card| card.id == CardId::Eviscerate) {
            if let Some(turn_cost) = card.local.turn_cost.as_mut() {
                if *turn_cost >= 0 {
                    *turn_cost = turn_cost.checked_sub(1)
                        .ok_or_else(|| InputError::Unsupported("Eviscerate discard cost arithmetic overflow".into()))?.max(0);
                }
            }
        }
    }
    Ok(())
}

pub(super) fn actual_draw(state: &mut State, owner: &CardHandle) -> Result<(), InputError> {
    let discarded_count = state.silent.discarded_this_turn;
    let Some(card) = state.piles.hand.iter_mut().find(|owned| owned.handle() == *owner).and_then(OwnedCard::full_mut) else {
        return Err(InputError::Invariant("Silent actual-draw owner absent from hand".into()));
    };
    if card.id == CardId::Eviscerate {
        let discarded = native_discard_count(discarded_count)?;
        if card.local.turn_cost.is_some_and(|turn_cost| turn_cost < 0) { return Ok(()); }
        let combat_cost = card.local.mutable_combat_cost
            .or_else(|| starting_cost(card.id, card.upgrades))
            .ok_or_else(|| InputError::Invariant("Eviscerate drawn cost is absent".into()))?;
        card.local.turn_cost = Some(eviscerate_turn_price(combat_cost, discarded)?);
    }
    Ok(())
}

fn update_combat_cost(card: &mut LiveCard, delta: i32) -> Result<(), InputError> {
    let old_combat = card.local.mutable_combat_cost.unwrap_or(0);
    let new_combat = old_combat.checked_add(delta)
        .ok_or_else(|| InputError::Unsupported("Masterful Stab combat-cost arithmetic overflow".into()))?.max(0);
    let old_turn = card.local.turn_cost.unwrap_or(old_combat);
    let turn_delta = new_combat.checked_sub(old_combat)
        .ok_or_else(|| InputError::Unsupported("Masterful Stab turn-cost delta overflow".into()))?;
    let new_turn = old_turn.checked_add(turn_delta)
        .ok_or_else(|| InputError::Unsupported("Masterful Stab turn-cost arithmetic overflow".into()))?.max(0);
    card.local.mutable_combat_cost = Some(new_combat);
    card.local.turn_cost = Some(new_turn);
    Ok(())
}

pub(super) fn apply(state: &mut State, effect: &Effect, work: &SilentEffect) -> Result<(), EffectError> {
    match work {
        SilentEffect::Setup => setup(state, effect),
        SilentEffect::BulletTime => {
            let costs = state.piles.hand.iter().filter_map(|owned| owned.full()).map(|card| {
                (card.instance, super::super::base_cost(state, card))
            }).collect::<std::collections::BTreeMap<_, _>>();
            for owned in &mut state.piles.hand {
                if let Some(card) = owned.full_mut() {
                    if card.local.x_cost { continue; }
                    let base = costs.get(&card.instance).copied().flatten();
                    let turn_cost = current_turn_cost(card, base)?;
                    if turn_cost.is_some_and(|cost| cost >= 0) { card.local.turn_cost = Some(0); }
                }
            }
            Ok(())
        }
        SilentEffect::GlassKnifeAfterHits => {
            let Source::Card(owner) = &effect.source else {
                return Err(InputError::Invariant("Glass Knife mutation requires its card owner".into()).into());
            };
            let family = state.in_play.as_ref().filter(|owned| owned.handle() == *owner)
                .and_then(OwnedCard::full).filter(|card| card.id == CardId::GlassKnife)
                .map(|card| card.copy_family)
                .ok_or_else(|| InputError::Invariant("Glass Knife source family absent".into()))?;
            crate::combat::piles::for_each_owned_card_mut(state, |card| {
                if card.id == CardId::GlassKnife && card.copy_family == family {
                    let base = card.local.base_damage.unwrap_or(if card.upgrades > 0 { 12 } else { 8 });
                    card.local.base_damage = Some(base.saturating_sub(2).max(0));
                }
            });
            Ok(())
        }
        SilentEffect::EndlessAgonyCopy { template, .. } => {
            let instance = crate::combat::piles::next_generated_instance(state)?;
            let copy = template.stat_equivalent_copy(instance);
            crate::combat::piles::insert_generated(state, copy, crate::combat::CardDestination::Hand)?;
            Ok(())
        }
        _ => Err(InputError::Invariant("non-cost Silent work reached cost dispatcher".into()).into()),
    }
}

fn current_turn_cost(card: &LiveCard, base_cost: Option<i32>) -> Result<Option<i32>, InputError> {
    if let Some(turn_cost) = card.local.turn_cost { return Ok(Some(turn_cost)); }
    let Some(base) = base_cost else { return Ok(None); };
    if card.local.combat_cost_reduction < 0 {
        return Err(InputError::Invariant("negative combat cost reduction".into()));
    }
    Ok(Some(super::super::super::card_state::combat_price(card,base)?))
}

fn setup(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    let Source::Card(parent) = &effect.source else {
        return Err(InputError::Invariant("Setup selection has no live card parent".into()).into());
    };
    let candidates = state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(()),
        [only] => {
            let index = state.piles.hand.iter().position(|owned| owned.handle() == *only)
                .ok_or_else(|| InputError::Invariant("automatic Setup card left hand".into()))?;
            let card = state.piles.hand.remove(index);
            Ok(setup_move_card(state, card, only)?)
        }
        _ => {
            crate::combat::effects::begin_selection(state, parent.clone(), candidates, 1,
                crate::combat::SelectionDisposition::DrawTop, crate::combat::EmptySelection::Continue)?;
            state.pending_selection.as_mut().expect("Setup creates a nonempty selection").operation =
                crate::combat::SelectionOperation::Setup;
            Ok(())
        }
    }
}

pub(super) fn setup_move_card(state: &mut State, mut owned: OwnedCard, handle: &CardHandle) -> Result<(), InputError> {
    let Some(card) = owned.full_mut() else {
        return Err(InputError::Invariant("Setup selected a non-full card".into()));
    };
    if !card.local.x_cost {
        let combat_cost = card.local.mutable_combat_cost.or_else(|| super::super::base_cost(state, card));
        if combat_cost.is_some_and(|cost| cost > 0) { card.local.free_to_play_once = true; }
    }
    crate::combat::piles::topdeck_owned_card(state, owned, handle.clone());
    Ok(())
}

/// The same source upgrade writers are used by temporary and all-pile upgrades.
/// Mutable damage retains prior Glass Knife degradation; cost upgrades preserve
/// the costForTurn/base difference only when the old turn price was positive.
pub(super) fn upgrade_local(card:&mut LiveCard)->Result<(),crate::run::RunError> {
    use crate::run::RunError;
    let damage_delta=match card.id {
        CardId::MasterfulStab|CardId::GlassKnife=>Some(4),
        CardId::Eviscerate|CardId::EndlessAgony=>Some(2),
        CardId::Skewer=>Some(3),
        CardId::Shiv if card.local.base_damage.is_some()=>Some(2),_=>None,
    };
    if let Some(delta)=damage_delta {
        let original=match card.id {CardId::MasterfulStab=>12,CardId::GlassKnife=>8,CardId::Eviscerate|CardId::Skewer=>7,_=>4};
        let next=card.local.base_damage.unwrap_or(original).checked_add(delta)
            .ok_or(RunError::Unsupported("Silent upgrade damage overflow"))?;
        card.local.base_damage=Some(next);
    }
    let (original,requested)=match card.id {
        CardId::Setup|CardId::ToolsOfTheTrade|CardId::PhantasmalKiller=>(1,0),CardId::Envenom=>(2,1),CardId::BulletTime=>(3,2),CardId::Terror=>(1,0),CardId::Nightmare=>(3,2),
        _=>return Ok(()),
    };
    super::super::super::card_state::upgrade_base_cost(card,original,requested)
}
