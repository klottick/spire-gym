//! Well-Laid Plans' one-turn, identity-only end-turn selection.
use crate::combat::{Actor,Effect,EffectError,EffectKind,EmptySelection,SelectionDisposition,
    SelectionParent,SelectionSourceZone,Source};
use crate::combat::model::State;

pub(super) fn begin(state:&mut State,effect:&Effect,limit:i32)->Result<(),EffectError>{
    let from_player=matches!((&effect.source,&effect.target),
        (Source::Actor(Actor::Player),Some(Actor::Player)));
    let queued_checkpoint=state.phase==super::super::super::Phase::EnemyTurn
        &&state.pending_selection.is_none()
        &&state.pending.iter().filter(|pending|pending.kind==EffectKind::DiscardHand).count()==1
        &&state.hook_events.iter().any(|event|event.hook==super::super::super::Hook::PlayerPreEndTurnCards
            &&event.source==Source::Actor(Actor::Player)&&event.target==Some(Actor::Player));
    let power=state.player_full_powers.iter().find(|power|power.id.0=="retain_cards")
        .map(|power|power.state.amount);
    if !state.full_runtime||!from_player||!queued_checkpoint||limit<=0||power!=Some(limit){
        return Err(crate::combat::InputError::Invariant(
            "retain selection requires the live player pre-discard EndTurn owner".into()).into());
    }
    if state.piles.hand.is_empty(){return Ok(())}
    let candidates=state.piles.hand.iter().map(|card|card.handle()).collect::<Vec<_>>();
    let maximum=usize::try_from(limit).map_err(|_|crate::combat::InputError::Invariant(
        "retain amount is not a positive card count".into()))?.min(candidates.len());
    super::super::super::effects::begin_selection_range(state,
        SelectionParent::EndTurn{turn:state.turn},SelectionSourceZone::Hand,candidates,0,maximum,
        SelectionDisposition::Hand,EmptySelection::Continue)?;
    state.pending_selection.as_mut().expect("nonempty hand creates retain selection").operation=
        crate::combat::SelectionOperation::Retain;
    Ok(())
}
