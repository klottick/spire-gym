//! Nightmare's captured card template and three-copy next-turn continuation.
use crate::combat::{Actor, CardDestination, CardHandle, CardInstanceId, Effect, EffectError,
    EffectKind, InputError, LiveCard, Source};
use crate::combat::cards::PayloadTarget;
use crate::combat::model::State;
use crate::content::CardId;

pub(super) fn payload(card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    if card.id != CardId::Nightmare { return Ok(None); }
    if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
    Ok(Some((if card.upgrades > 0 { 2 } else { 3 },
        vec![(None, EffectKind::SelectNightmareCard)])))
}

fn installation_cursor(state:&State)->Result<(u32,u64,u64),InputError>{
    let delivery_turn = state.turn.checked_add(1).filter(|turn| *turn <= i32::MAX as u32)
        .ok_or_else(|| InputError::Unsupported("Nightmare next turn outside supported numeric surface".into()))?;
    if !state.full_runtime || state.next_nightmare_installation_id == 0 {
        return Err(InputError::Invariant("Nightmare installation identity is unavailable in this combat".into()));
    }
    let installation_id = state.next_nightmare_installation_id;
    let next_installation_id = installation_id.checked_add(1)
        .ok_or_else(|| InputError::Unsupported("Nightmare installation identity exhausted".into()))?;
    Ok((delivery_turn,installation_id,next_installation_id))
}

fn reset_template_costs(state:&State,template:&mut LiveCard)->Result<(),InputError>{
    template.local.temporary_cost=0;
    template.local.infernal_blade_cost_this_turn=false;
    if let Some(base)=super::super::base_cost(state,template){
        template.local.turn_cost=Some(super::super::super::card_state::combat_price(template,base)?);
    }
    Ok(())
}

fn store_template(state:&mut State,owner:CardHandle,delivery_turn:u32,installation_id:u64,
    next_installation_id:u64,template:LiveCard){
    state.nightmare_templates.push(super::super::super::model::NightmareTemplateOwner {
        installation_id, owner, delivery_turn, delivered: 0, template,
    });
    state.next_nightmare_installation_id = next_installation_id;
}

/// Preserve the established value-taking helper for compatibility callers.
pub(super) fn capture(state:&mut State,owner:CardHandle,source:&LiveCard)->Result<(),InputError>{
    let (delivery_turn,installation_id,next_installation_id)=installation_cursor(state)?;
    let mut template=source.stat_equivalent_copy(CardInstanceId(0));
    reset_template_costs(state,&mut template)?;
    store_template(state,owner,delivery_turn,installation_id,next_installation_id,template);
    Ok(())
}

/// Runtime capture performs native makeCopy against the actual selected owner.
pub(super) fn capture_from_owner(state:&mut State,owner:CardHandle,source:&CardHandle)->Result<(),InputError>{
    let (delivery_turn,installation_id,next_installation_id)=installation_cursor(state)?;
    let mut template=super::super::super::piles::make_stat_equivalent_copy_from_owner(state,source,CardInstanceId(0))?;
    reset_template_costs(state,&mut template)?;
    store_template(state,owner,delivery_turn,installation_id,next_installation_id,template);
    Ok(())
}

/// Queue one owner action per template due at this qualified player pre-draw.
pub(super) fn pre_draw(state: &State) -> Vec<Effect> {
    state.nightmare_templates.iter().filter(|pending| pending.delivery_turn == state.turn)
        .map(|pending| Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
            kind: EffectKind::NightmareCopy { owner: pending.owner.clone(), installation_id: pending.installation_id } })
        .collect()
}

pub(super) fn deliver(state: &mut State, effect: &Effect, owner: &CardHandle, installation_id:u64)
    -> Result<(), EffectError>
{
    if effect.source != Source::Actor(Actor::Player) || effect.target != Some(Actor::Player)
        || state.phase != super::super::super::Phase::PlayerStart {
        return Err(InputError::Invariant("Nightmare delivery lacks its live player pre-draw owner".into()).into());
    }
    let index = state.nightmare_templates.iter().position(|pending|
        pending.owner == *owner && pending.installation_id == installation_id)
        .ok_or_else(|| InputError::Invariant("Nightmare delivery installation owner absent".into()))?;
    let pending = state.nightmare_templates[index].clone();
    if pending.delivery_turn != state.turn || pending.delivered >= 3 {
        return Err(InputError::Invariant("Nightmare delivery turn/cursor mismatch".into()).into());
    }
    let instance = super::super::super::piles::next_generated_instance(state)?;
    let history=super::super::copy_history(state);
    let copy={
        let template=&mut state.nightmare_templates[index].template;
        super::super::make_stat_equivalent_copy(template,instance,history)?
    };
    super::super::super::piles::insert_generated(state, copy, CardDestination::Hand)?;

    let current = state.nightmare_templates.get_mut(index)
        .ok_or_else(|| InputError::Invariant("Nightmare owner disappeared after insertion".into()))?;
    current.delivered += 1;
    if current.delivered == 3 {
        state.nightmare_templates.remove(index);
    } else {
        state.pending.push_front(Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
            kind: EffectKind::NightmareCopy { owner: owner.clone(), installation_id } });
    }
    Ok(())
}

pub(super) fn validate_pending(state: &State) -> Result<(), InputError> {
    if state.next_nightmare_installation_id == 0 {
        return Err(InputError::Invariant("Nightmare installation counter is zero".into()));
    }
    if state.nightmare_templates.is_empty() { return Ok(()); }
    if !state.full_runtime { return Err(InputError::Invariant("legacy combat cannot own Nightmare templates".into())); }
    let mut installations = std::collections::BTreeSet::new();
    for pending in &state.nightmare_templates {
        let template = &pending.template;
        let owner_card = state.piles.all(state.in_play.as_ref())
            .chain(state.stasis.values()).chain(&state.played_powers).chain(&state.inactive_cards)
            .chain(state.play_frames.iter().filter_map(|frame|frame.parent.as_ref()))
            .chain(state.queued_autoplay.values())
            .any(|card| card.handle() == pending.owner
                && card.full().is_some_and(|live|live.id==CardId::Nightmare));
        let continuation = state.pending.iter().any(|effect|
            matches!(&effect.kind,EffectKind::NightmareCopy{owner,installation_id}
                if owner==&pending.owner&&*installation_id==pending.installation_id));
        let current = state.current.as_ref().is_some_and(|effect|
            matches!(&effect.kind,EffectKind::NightmareCopy{owner,installation_id}
                if owner==&pending.owner&&*installation_id==pending.installation_id));
        let scheduled_hook = state.pending.iter().chain(state.current.iter()).any(|effect|
            effect.kind==EffectKind::Hook(crate::combat::Hook::TurnStartedPreDraw));
        if pending.installation_id == 0 || pending.installation_id >= state.next_nightmare_installation_id
            || !installations.insert(pending.installation_id) || !owner_card
            || pending.owner.as_str().parse::<u64>().is_err()
            || pending.delivery_turn < state.turn || pending.delivery_turn > i32::MAX as u32
            || pending.delivered >= 3 || template.instance != CardInstanceId(0)
            || template.origin != crate::combat::CombatCardRef::CombatLocal(0)
            || template.copy_family != 0 || template.copy_provenance != crate::combat::card_state::CopyProvenance::StatEquivalent
            || template.local.temporary_cost != 0 || template.local.infernal_blade_cost_this_turn
            || template.local.temporary_upgrades != 0
            || (pending.delivery_turn == state.turn
                && (state.phase != super::super::super::Phase::PlayerStart
                    || !(continuation || current || scheduled_hook))) {
            return Err(InputError::Invariant("Nightmare template identity, owner, reset state, or delivery cursor invalid".into()));
        }
    }
    Ok(())
}
