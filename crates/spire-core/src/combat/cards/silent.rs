//! Silent/common card payloads using the shared native resolver.
use super::*;

mod poison;
mod discard;
mod shiv;
mod retain;
mod copy;

pub(super) fn manual_discard(state: &mut State, card: &LiveCard) -> Result<bool, crate::combat::InputError> {
    discard::manual_discard(state, card)
}

pub(super) fn apply_catalyst(state: &mut State, effect: &Effect) -> Result<(), super::super::EffectError> {
    poison::apply_catalyst(state, effect)
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    if let Some(result) = poison::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = shiv::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = copy::payload(card, target)? { return Ok(Some(result)); }
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let result = match card.id {
        Backflip => (1, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 8 } else { 5 })?)),
            (None, EffectKind::Draw(2)),
        ]),
        Deflect => (0, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 7 } else { 4 })?))]),
        QuickSlash | DaggerThrow => {
            let target_id = target.ok_or(crate::run::RunError::InvalidAction)?;
            let base = match (card.id, upgraded) {
                (QuickSlash, false) => 8, (QuickSlash, true) => 12,
                (DaggerThrow, false) => 9, (DaggerThrow, true) => 12,
                _ => unreachable!(),
            };
            let damage = super::modified_damage(state, target_id, base)?;
            let enemy = state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone();
            let mut effects = vec![(Some(PayloadTarget::Enemy(enemy)), EffectKind::Damage(DamagePayload::Fixed(damage)))];
            if card.id == QuickSlash {
                effects.push((None, EffectKind::Draw(1)));
            } else {
                effects.push((None, EffectKind::Draw(1)));
                effects.push((None, EffectKind::SelectHandCardsAfterDraw { required: 1,
                    disposition: super::super::SelectionDisposition::Discard,
                    empty: super::super::EmptySelection::Continue }));
            }
            (1, effects)
        }
        CloakAndDagger => {
            let mut effects = vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, 6)?)),
                (Some(PayloadTarget::Player), EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                    destination: crate::combat::CardDestination::Hand, upgrades: 0 })];
            if upgraded {
                effects.push((Some(PayloadTarget::Player), EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                    destination: crate::combat::CardDestination::Hand, upgrades: 0 }));
            }
            (1, effects)
        }
        WellLaidPlans => (1, vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower {
            id:FullPowerId("retain_cards"),amount:if upgraded{2}else{1},
        })]),
        _ => return Ok(None),
    };
    Ok(Some(result))
}

pub(super) fn rewrite_accuracy_shivs(state: &mut State, amount: i32) {
    shiv::rewrite_existing(state, amount);
}

pub(super) fn rewrite_accuracy_hand_shivs(state: &mut State, amount: i32) {
    shiv::rewrite_hand(state, amount);
}

pub(super) fn apply_retain_selection(state: &mut State, effect: &Effect, limit: i32)
    -> Result<(), super::super::EffectError> {
    retain::begin(state, effect, limit)
}

pub(super) fn capture_nightmare_template(state:&mut State,owner:crate::combat::CardHandle,
    source:&LiveCard)->Result<(),crate::combat::InputError>{copy::capture(state,owner,source)}

pub(super) fn nightmare_pre_draw(state:&State)->Vec<Effect>{copy::pre_draw(state)}

pub(super) fn apply_nightmare_copy(state:&mut State,effect:&Effect,owner:&crate::combat::CardHandle,installation_id:u64)
    ->Result<(),super::super::EffectError>{copy::deliver(state,effect,owner,installation_id)}

pub(super) fn validate_nightmare_templates(state:&State)->Result<(),crate::combat::InputError>{
    copy::validate_pending(state)
}
