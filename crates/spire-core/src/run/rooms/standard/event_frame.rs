//! The standard event frame (design §5.3): one frame type for every event body, generalizing the
//! accepted Act 3 frame. Values fixed at construction live in `EventCaptures` and never recompute;
//! a choice is validated against the executor's preconditions before anything is applied, and a
//! failure after acceptance is a retained fault.
use super::*;
use crate::content::{Catalog, EncounterId};
use super::event_exec::{self, Outcome, RewardSpec};
use super::events::{self, BodyContext};

/// The result stage every body reaches after its last mutation: a single Leave row.
pub(in crate::run) const DONE:u8=u8::MAX;

/// Values the source fixes when the event is constructed (amounts from HP, max HP and gold;
/// drawn cards, relics and encounters). Bodies own the meaning of each slot.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub(crate) struct EventCaptures { pub values:Vec<u32>, pub cards:Vec<u64>, pub relics:Vec<RelicId>, pub encounter:Option<EncounterId>,
    /// Card identities fixed at construction (Match and Keep's board).
    pub card_ids:Vec<CardId> }

/// A suspended event combat (design §5.7).
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct EventCombat { pub encounter:EncounterId, pub rewards:Option<RewardSpec>, pub next_stage:u8, pub elite:bool }

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct StandardEventFrame {
    pub node:NodeId, pub event:EventId, pub stage:u8, pub captured:EventCaptures,
    pub accepted:Option<(ChoiceId,RoomChoiceKind)>,
    /// Outcomes after a suspending selection or combat, applied when it returns.
    pub pending_outcomes:Vec<Outcome>, pub combat:Option<EventCombat>, pub select_skippable:bool,
    /// Executor draw ordinal (each outcome's RNG key).
    pub applied:u64, pub complete:bool,
}

fn frame(run:&Run)->Result<&StandardEventFrame,RunError>{run.state.room_loop.as_ref().and_then(|r|r.standard_event.as_ref()).ok_or(RunError::InvalidAction)}
fn frame_mut(run:&mut Run)->Result<&mut StandardEventFrame,RunError>{run.state.room_loop.as_mut().and_then(|r|r.standard_event.as_mut()).ok_or(RunError::InvalidAction)}

fn catalog_index(event:EventId)->u64{Catalog::standard().events().iter().position(|row|row.id==event).map_or(u64::MAX,|i|i as u64)}

/// The RNG key of the frame's next draw (design §8): act, floor, node, event, stage, ordinal.
pub(in crate::run) fn next_key(run:&mut Run)->Result<Vec<u64>,RunError>{
    let (act,floor)=(u64::from(run.state.act),u64::from(run.state.floor));
    let frame=frame_mut(run)?;
    let key=vec![act,floor,frame.node.0,catalog_index(frame.event),u64::from(frame.stage),frame.applied];
    frame.applied=frame.applied.checked_add(1).ok_or(RunError::Unsupported("event draw ordinal exhausted"))?;
    Ok(key)
}

/// Construct `event` at `node` on the staged Run: capture its values and publish its first stage.
/// An event whose body has not landed is the typed interim naming its chunk (chunk plan §3).
pub(in crate::run) fn construct(run:&mut Run,event:EventId,node:NodeId)->Result<(),RunError>{
    let body=events::body(event).ok_or(RunError::Unsupported(events::interim(event)))?;
    let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,catalog_index(event),0xC0];
    let mut rng=run.state.rng.clone();
    let captured=(body.construct)(&mut BodyContext{state:&run.state,rng:&mut rng,key})?;
    run.state.rng=rng;
    run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.standard_event=Some(StandardEventFrame{node,event,stage:0,captured,accepted:None,
        pending_outcomes:vec![],combat:None,select_skippable:false,applied:0,complete:false});
    publish(run)
}

/// Publish the frame's current stage: the body's option rows, or the result stage's Leave.
pub(in crate::run) fn publish(run:&mut Run)->Result<(),RunError>{
    let frame=frame(run)?.clone();
    let rows=if frame.stage==DONE{vec![(RoomChoiceKind::Leave,None)]}else{
        let body=events::body(frame.event).ok_or(RunError::Unsupported(events::interim(frame.event)))?;
        (body.rows)(&run.state,&frame.captured,frame.stage).into_iter().map(|(option,price)|(match body.card{
            // A card-board row shows the identity of a revealed position (D2, Match and Keep).
            Some(card)=>RoomChoiceKind::EventCard{stage:frame.stage,option,card:card(&frame.captured,frame.stage,option)},
            None=>RoomChoiceKind::EventOption{stage:frame.stage,option},
        },price)).collect()
    };
    let mut next=run.next_choice;
    let screen=view(&mut next,RoomViewKind::Event(frame.event),rows)?;
    run.next_choice=next;
    run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen=screen;
    run.candidates.clear();
    Ok(())
}

fn outcomes(run:&Run,frame:&StandardEventFrame,stage:u8,option:u8)->Result<Vec<Outcome>,RunError>{
    if stage!=frame.stage||frame.complete||frame.combat.is_some()||!frame.pending_outcomes.is_empty(){return Err(RunError::InvalidAction)}
    let body=events::body(frame.event).ok_or(RunError::InvalidAction)?;
    (body.choose)(&run.state,&frame.captured,stage,option)
}

/// Whether an option row is offered now: its outcomes pass every executor precondition.
pub(in crate::run) fn enabled(run:&Run,kind:&RoomChoiceKind)->bool{
    let (RoomChoiceKind::EventOption{stage,option}|RoomChoiceKind::EventCard{stage,option,..})=*kind else {return true};
    frame(run).ok().and_then(|frame|outcomes(run,frame,stage,option).ok()).is_some_and(|list|event_exec::check(&run.state,&list).is_ok())
}

/// Choose a row of the standard event screen (two-phase: validate, then apply on a staged Run).
pub(in crate::run) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    if let Some(fault)=&run.state.fault{return Err(RunError::Faulted(fault.clone()))}
    let frame=frame(run)?.clone();
    if run.state.map.as_ref().and_then(|m|m.current)!=Some(frame.node)||run.state.pending.is_some(){return Err(RunError::InvalidAction)}
    let row=public(run)?.choices.into_iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    if frame.stage==DONE{
        if row.kind!=RoomChoiceKind::Leave{return Err(RunError::InvalidAction)}
        return leave(run)
    }
    let (RoomChoiceKind::EventOption{stage,option}|RoomChoiceKind::EventCard{stage,option,..})=row.kind else {return Err(RunError::InvalidAction)};
    let list=outcomes(run,&frame,stage,option)?;
    accept(run,id,row.kind,list)
}

/// Diagnostics: run a constructed outcome list as if the screen's first row had been chosen.
#[cfg(any(test,feature="m28-diagnostics"))]
pub(in crate::run) fn choose_constructed(run:&mut Run,list:Vec<Outcome>)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.first().cloned().ok_or(RunError::InvalidAction)?;
    accept(run,row.id,row.kind,list)
}

/// Two-phase acceptance: every precondition first (a rejection leaves the Run unchanged), then the
/// staged application; a failure after acceptance is a retained fault.
fn accept(run:&mut Run,id:ChoiceId,kind:RoomChoiceKind,list:Vec<Outcome>)->Result<RunTransition,RunError>{
    event_exec::check(&run.state,&list)?;
    let mut staged=run.clone();
    frame_mut(&mut staged)?.accepted=Some((id,kind));
    match event_exec::apply(&mut staged,list){
        Ok(transition)=>{*run=staged;Ok(transition)}
        Err(RunError::Faulted(fault))=>{*run=staged;Err(RunError::Faulted(fault))}
        Err(RunError::Unsupported(message)) if message==event_exec::ENTER_BOSS_INTERIM=>Err(RunError::Unsupported(message)),
        Err(_)=>Err(run.retain_fault(RunFaultKind::Invariant("accepted standard event outcome failed"),"run.event.standard")),
    }
}

pub(in crate::run) fn accepted_choice(run:&Run)->Result<ChoiceId,RunError>{frame(run)?.accepted.as_ref().map(|(id,_)|*id).ok_or(RunError::InvalidAction)}
pub(in crate::run) fn set_stage(run:&mut Run,stage:u8)->Result<(),RunError>{frame_mut(run)?.stage=stage;Ok(())}
pub(in crate::run) fn set_value(run:&mut Run,slot:u8,value:u32)->Result<(),RunError>{
    let values=&mut frame_mut(run)?.captured.values;
    let slot=usize::from(slot);
    if values.len()<=slot{values.resize(slot+1,0)}
    values[slot]=value;Ok(())
}

/// Publish an event's non-combat reward screen (Lab, Wheel of Change; X4): the rows are claimed
/// through the shared reward screen and its Proceed finishes the room. The event is complete.
pub(in crate::run) fn open_rewards(run:&mut Run,rows:Vec<(RoomChoiceKind,Option<u32>)>)->Result<RunTransition,RunError>{
    frame_mut(run)?.complete=true;
    let mut next=run.next_choice;
    let screen=view(&mut next,RoomViewKind::Rewards,rows)?;
    run.next_choice=next;
    let room=run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.screen=screen;room.active_rewards=None;room.reward_parent=None;
    run.candidates.clear();
    Ok(empty_transition())
}

/// Leave the event: the room closes through the shared standard `finish`.
pub(in crate::run) fn leave(run:&mut Run)->Result<RunTransition,RunError>{
    frame_mut(run)?.complete=true;
    finish(run)
}

/// Park the rest of the outcome list behind a selection or a combat.
pub(in crate::run) fn suspend(run:&mut Run,rest:Vec<Outcome>,combat:Option<EventCombat>,skippable:bool)->Result<(),RunError>{
    let frame=frame_mut(run)?;
    frame.pending_outcomes=rest;frame.combat=combat;frame.select_skippable=skippable;
    Ok(())
}
/// Continue the parked outcomes (after a selection or a reward-less combat).
pub(in crate::run) fn resume(run:&mut Run)->Result<RunTransition,RunError>{
    let rest=std::mem::take(&mut frame_mut(run)?.pending_outcomes);
    event_exec::apply(run,rest)
}

/// Open an `EventSelect` (design §5.6); completion returns to this frame.
pub(in crate::run) fn open_select(run:&mut Run,op:EventSelectOp,count:u8)->Result<RunTransition,RunError>{
    let (event,stage)={let f=frame(run)?;(f.event,f.stage)};
    let choice=accepted_choice(run)?;
    selection::open(run,choice,RoomSelectionOperation::EventSelect{event,stage,op,count})
}

/// One card of an `EventSelect`: collect `count` distinct cards, then apply the operation to them
/// in selection order and resume the frame's outcomes.
pub(in crate::run) fn select(run:&mut Run,id:SelectionId)->Result<RunTransition,RunError>{
    let pending=run.state.pending.as_ref().ok_or(RunError::InvalidAction)?.clone();
    let ContinuationTarget::Room{node,choice,operation:RoomSelectionOperation::EventSelect{event,stage,op,..}}=pending.parent else {return Err(RunError::InvalidAction)};
    let frame=frame(run)?.clone();
    if pending.resumed||frame.node!=node||frame.event!=event||frame.stage!=stage||frame.accepted.as_ref().map(|(c,_)|*c)!=Some(choice)
        ||run.state.map.as_ref().and_then(|m|m.current)!=Some(node){return Err(RunError::InvalidAction)}
    let captured=pending.selection.candidates.iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?.card.clone();
    if !run.state.deck.contains(&captured)||pending.selection.selected.iter().any(|c|c.instance==captured.instance){return Err(RunError::InvalidAction)}
    let mut selected=pending.selection.selected.clone();
    selected.push(captured);
    let mut staged=run.clone();
    if selected.len()<pending.selection.required{
        let selection=staged.state.pending.as_mut().ok_or(RunError::InvalidAction)?;
        selection.selection.candidates.retain(|c|c.id!=id);
        selection.selection.selected=selected;
        staged.publish_selection_candidates()?;
        *run=staged;
        return Ok(empty_transition())
    }
    if op==EventSelectOp::Remove&&selected.len()==staged.state.deck.len(){
        reward::preflight(&staged.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;
    }
    for (slot,card) in selected.iter().enumerate(){
        match op{
            EventSelectOp::Remove=>{mutations::remove_card(&mut staged.state,card.instance)?;}
            EventSelectOp::Upgrade=>mutations::upgrade_card(&mut staged.state,card.instance)?,
            EventSelectOp::Transform=>{
                let key=[u64::from(staged.state.act),u64::from(staged.state.floor),node.0,catalog_index(event),u64::from(stage),0x7F,slot as u64];
                mutations::transform_card(&mut staged.state,card.instance,false,&key)?;
            }
            EventSelectOp::Duplicate=>{mutations::duplicate_card(&mut staged.state,card.instance)?;}
            // The body decides what a chosen card does (X3: Bonfire Spirits, Note For Yourself).
            EventSelectOp::Choose=>{}
        }
    }
    if op==EventSelectOp::Choose{
        let body=events::body(event).ok_or(RunError::InvalidAction)?;
        let hook=body.selected.ok_or(RunError::InvalidAction)?;
        let mut list=hook(&staged.state,&frame.captured,stage,&selected)?;
        list.append(&mut frame_mut(&mut staged)?.pending_outcomes);
        event_exec::check(&staged.state,&list)?;
        frame_mut(&mut staged)?.pending_outcomes=list;
    }
    staged.state.last_selection=Some(selected);
    staged.state.last_resumed_parent=Some(choice);
    staged.state.pending=None;
    staged.candidates.clear();
    if staged.state.deck.is_empty(){let transition=staged.commit_terminal(RunTerminal::EmptyDeck)?;*run=staged;return Ok(transition)}
    let transition=resume(&mut staged)?;
    *run=staged;
    Ok(transition)
}

/// Cancel a skippable `EventSelect`: nothing is selected and the frame continues.
pub(in crate::run) fn cancel_select(run:&mut Run)->Result<RunTransition,RunError>{
    if !frame(run)?.select_skippable{return Err(RunError::InvalidAction)}
    let mut staged=run.clone();
    staged.state.pending=None;staged.candidates.clear();
    let transition=resume(&mut staged)?;
    *run=staged;
    Ok(transition)
}
pub(in crate::run) fn skippable_select(run:&Run)->bool{frame(run).is_ok_and(|f|f.select_skippable)}

/// Start the suspended event combat through the shared Run owner (design §5.7).
pub(in crate::run) fn enter_combat(run:&mut Run,encounter:EncounterId)->Result<RunTransition,RunError>{
    let (node,event,stage)={let f=frame(run)?;(f.node,f.event,f.stage)};
    run.enter_owned(encounter,CombatOrigin::Event{node,event,stage})
}

/// The combat entry gate's event arm: the live frame at the current node started this encounter.
pub(in crate::run) fn admits_combat(state:&RunState,encounter:EncounterId,node:NodeId,event:EventId,stage:u8)->bool{
    state.map.as_ref().and_then(|m|m.current)==Some(node)&&state.room_loop.as_ref().and_then(|r|r.standard_event.as_ref()).is_some_and(|f|
        f.node==node&&f.event==event&&f.stage==stage&&!f.complete&&f.combat.as_ref().is_some_and(|c|c.encounter==encounter))
}

/// The suspended combat's reward spec, read by the standard reward builder.
pub(in crate::run) fn combat_rewards(state:&RunState)->Option<RewardSpec>{
    state.room_loop.as_ref()?.standard_event.as_ref()?.combat.as_ref()?.rewards.clone()
}

/// Whether the suspended event combat sets the source's `eliteTrigger`.
pub(in crate::run) fn combat_elite(state:&RunState)->bool{
    state.room_loop.as_ref().and_then(|r|r.standard_event.as_ref()).and_then(|f|f.combat.as_ref()).is_some_and(|c|c.elite)
}

/// After a won event combat without a reward screen: continue at the combat's next stage.
pub(in crate::run) fn after_combat(run:&mut Run)->Result<RunTransition,RunError>{
    let combat=frame_mut(run)?.combat.take().ok_or(RunError::InvalidAction)?;
    set_stage(run,combat.next_stage)?;
    resume(run)
}
