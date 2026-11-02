//! M25 Watcher powers, draw callbacks, and turn-control producers.
use super::*;
use crate::content::CardId;
use crate::run::RunError;

type Payload = (i32, Vec<(Option<PayloadTarget>, EffectKind)>);

fn selected(state: &State, target: Option<u64>) -> Result<crate::combat::EnemyHandle, RunError> {
    let slot = target.ok_or(RunError::InvalidAction)?;
    slot.checked_sub(1).and_then(|i| usize::try_from(i).ok())
        .and_then(|i| state.enemies.get(i))
        .filter(|enemy| enemy.life == ActorLife::Active && enemy.hp > 0)
        .map(|enemy| enemy.instance_id.clone()).ok_or(RunError::InvalidAction)
}

fn fixed_hit(state: &State, target: u64, base: i32) -> Result<(Option<PayloadTarget>, EffectKind), RunError> {
    Ok((Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(
        DamagePayload::Fixed(super::super::modified_mutable_damage(state, target, base)?))))
}

pub(in crate::combat) fn payload(state: &State, card: &LiveCard, target: Option<u64>) -> Result<Option<Payload>, RunError> {
    use CardId::*;
    if !matches!(card.id, PressurePoints | TalkToTheHand | WreathOfFlame | WaveOfTheHand | Swivel
        | SimmeringFury | Fasting | DevaForm | Collect | Blasphemy | Vault | Conclude) {
        return Ok(None);
    }
    let cost = super::m25::m25_cost(card.id, card.upgrades > 0).ok_or(RunError::InvalidAction)?;
    let up = card.upgrades > 0;
    let power = |id: &'static str, amount: i32| (Some(PayloadTarget::Player),
        EffectKind::ApplyFullPower { id: FullPowerId(id), amount });
    let no_target = || if target.is_some() { Err(RunError::InvalidAction) } else { Ok(()) };
    let effects = match card.id {
        PressurePoints => {
            let owner = selected(state, target)?;
            vec![
                (Some(PayloadTarget::Enemy(owner)), EffectKind::ApplyFullPower {
                    id: FullPowerId("mark"), amount: card.local.magic_number.ok_or(RunError::InvalidAction)? }),
                (Some(PayloadTarget::Player), EffectKind::Watcher(WatcherEffect::PressurePoints)),
            ]
        }
        TalkToTheHand => {
            let owner = selected(state, target)?;
            vec![
                fixed_hit(state, target.unwrap(), card.local.base_damage.ok_or(RunError::InvalidAction)?)?,
                (Some(PayloadTarget::Enemy(owner)), EffectKind::ApplyFullPower {
                    id: FullPowerId("block_return"), amount: card.local.magic_number.ok_or(RunError::InvalidAction)? }),
            ]
        }
        WreathOfFlame => { no_target()?; vec![power("vigor", card.local.magic_number.ok_or(RunError::InvalidAction)?)] }
        WaveOfTheHand => { no_target()?; vec![power("wave_of_the_hand", card.local.magic_number.ok_or(RunError::InvalidAction)?)] }
        Swivel => {
            no_target()?;
            vec![
                (Some(PayloadTarget::Player), EffectKind::GainBlock(super::super::modified_mutable_block(
                    state, card.local.base_block.ok_or(RunError::InvalidAction)?)?)),
                power("free_attack", 1),
            ]
        }
        SimmeringFury => { no_target()?; vec![power("simmering_fury", card.local.magic_number.ok_or(RunError::InvalidAction)?)] }
        Fasting => {
            no_target()?;
            let amount = card.local.magic_number.ok_or(RunError::InvalidAction)?;
            vec![power("strength", amount), power("dexterity", amount), power("energy_down", 1)]
        }
        DevaForm => { no_target()?; vec![power("deva_form", 1)] }
        Collect => {
            no_target()?;
            // X-cost: Chemical X adds 2, as for Conjure Blade and every other X card.
            let turns = super::super::captured_x(state,card)?.checked_add(super::super::chemical_x_bonus(state)?)
                .and_then(|x| x.checked_add(i32::from(up))).ok_or(RunError::InvalidAction)?;
            if turns > 0 { vec![power("collect", turns)] } else { vec![] }
        }
        Blasphemy => {
            no_target()?;
            vec![
                (Some(PayloadTarget::Player), EffectKind::Character(CharacterEffect::Stance(Stance::Divinity))),
                power("blasphemer", 1),
            ]
        }
        Vault => {
            no_target()?;
            vec![
                (Some(PayloadTarget::Player), EffectKind::Watcher(WatcherEffect::GrantExtraTurn)),
                (Some(PayloadTarget::Player), EffectKind::Watcher(WatcherEffect::RequestEndTurn)),
            ]
        }
        Conclude => {
            no_target()?;
            let damage = card.local.base_damage.ok_or(RunError::InvalidAction)?;
            let mut effects = state.enemies.iter().enumerate()
                .filter(|(_, enemy)| enemy.targetable())
                .map(|(slot, enemy)| Ok((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(super::super::modified_mutable_damage(state,slot as u64+1,damage)?))))
                ).collect::<Result<Vec<_>, RunError>>()?;
            effects.push((Some(PayloadTarget::Player), EffectKind::Watcher(WatcherEffect::RequestEndTurn)));
            effects
        }
        _ => unreachable!(),
    };
    Ok(Some((cost, effects)))
}

/// Deus Ex Machina only reacts to the shared actual-draw callback. Its create
/// actions use the Watcher creation owner, then a typed continuation exhausts
/// the still-hand-owned drawn card.
pub(in crate::combat) fn actual_draw(state: &mut State, owner: &CardHandle) -> Result<(), EffectError> {
    if !state.item_context.as_ref().is_some_and(|items| items.m25_cards()) { return Ok(()) }
    let Some(card) = state.piles.hand.iter().find(|owned| owned.handle() == *owner)
        .and_then(OwnedCard::full).cloned().filter(|card| card.id == CardId::DeusExMachina) else { return Ok(()) };
    let count = u32::try_from(card.local.magic_number.ok_or_else(|| InputError::Invariant("Deus Ex Machina draw count absent".into()))?)
        .map_err(|_| InputError::Invariant("Deus Ex Machina draw count negative".into()))?;
    let create = super::create_action(state, CardId::Miracle, count)?;
    // The card remains in hand through every shared creation/overflow action.
    // Its exact physical identity is exhausted by the final continuation.
    state.pending.push_front(Effect { source: Source::Card(owner.clone()), target: Some(Actor::Player),
        kind: EffectKind::Watcher(WatcherEffect::ExhaustDrawnDeus { card: owner.clone() }) });
    state.pending.push_front(Effect { source: Source::Card(owner.clone()), target: Some(Actor::Player), kind: create });
    Ok(())
}

pub(in crate::combat) fn after_card_use(state: &mut State, source: &Source) {
    let Source::Card(owner) = source else { return };
    let attack = state.in_play.as_ref().filter(|card| card.handle() == *owner)
        .and_then(OwnedCard::full).is_some_and(|card| crate::content::Catalog::standard().card(card.id)
            .is_some_and(|row| row.card_type == crate::content::CardType::Attack));
    if attack {
        for (id, present) in [("pen_nib", state.player_full_powers.iter().any(|power|power.id.0=="pen_nib")),
            ("vigor", state.player_full_powers.iter().any(|power|power.id.0=="vigor"))] {
            if present {
                state.pending.insert(0, Effect { source: source.clone(), target: Some(Actor::Player),
                    kind: EffectKind::RemoveFullPower(FullPowerId(id)) });
            }
        }
    }
}
