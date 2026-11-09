//! Standard map-room entry (design §7): `world::enter_standard` commits the move, then the room
//! producer for the resolved kind runs. The whole entry is staged: a rejected or interim room
//! leaves the Run unchanged; a combat-entry fault keeps its committed progress as the captured
//! owners do.
use super::*;

/// The room kind `RoomEntered` sees in an event room: the source generates the event on player
/// entry, after every relic's `onEnterRoom` (Ssserpent Head's gold counts for eligibility).
const EVENT_ROOM:RoomViewKind=RoomViewKind::Event(EventId::Unknown("event room"));

pub(in crate::run) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    if !is_standard(&run.state)||run.state.act>3{return Err(RunError::InvalidAction)}
    let mut staged=run.clone();
    let (kind,encounter,event,credit)=crate::run::world::enter_standard(&mut staged.state,target)?;
    enter_room(run,staged,target,kind,encounter,event,credit)
}

/// Secret Portal's boss entry (`EnterBoss`): the act's boss room through the same producer as a map
/// move, at the current floor + 1 (PM check m28f/source-check-events-v1). The normal boss settlement
/// and Act 3 ending follow.
pub(in crate::run) fn enter_portal_boss(run:&mut Run)->Result<RunTransition,RunError>{
    if !is_standard(&run.state)||run.state.act>3{return Err(RunError::InvalidAction)}
    let mut staged=run.clone();
    let (target,kind,encounter,event,credit)=crate::run::world::enter_portal_boss(&mut staged.state)?;
    enter_room(run,staged,target,kind,encounter,event,credit)
}

/// The A20 Act 3 second boss (M33 design §6.3; source `ProceedButton.goToDoubleBoss`): Proceed from the
/// first boss's room enters the order's next boss through the same producer as a map move, at the
/// current floor + 1 (PM ruling m28f Q2). The first boss's ending frame ends here.
pub(in crate::run) fn enter_second_boss(run:&mut Run)->Result<RunTransition,RunError>{
    if !is_standard(&run.state)||run.state.act!=3{return Err(RunError::InvalidAction)}
    let mut staged=run.clone();
    staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.act3_end=None;
    let (target,kind,encounter,event,credit)=crate::run::world::enter_second_boss(&mut staged.state)?;
    enter_room(run,staged,target,kind,encounter,event,credit)
}

/// The room producer after a committed entry, on the staged Run.
fn enter_room(run:&mut Run,mut staged:Run,target:NodeId,kind:MapNodeKind,encounter:Option<crate::content::EncounterId>,event:CommittedEvent,credit:i32)->Result<RunTransition,RunError>{
    let entered_kind=match kind{
        MapNodeKind::Monster|MapNodeKind::Elite|MapNodeKind::Boss=>RoomViewKind::Combat,
        MapNodeKind::Shop=>RoomViewKind::Shop,
        MapNodeKind::Rest=>RoomViewKind::Campfire,
        MapNodeKind::Treasure=>RoomViewKind::Treasure,
        MapNodeKind::Event=>EVENT_ROOM,
        MapNodeKind::Question=>return Err(RunError::Registry("question room left unresolved")),
    };
    let room=staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.active_rewards=None;room.reward_parent=None;room.standard_event=None;
    // The previous room's event stops being the current room's: We Meet Again's potion lock ends here.
    staged.state.we_meet_again=false;
    {
        let mut item_event=crate::items::dispatch::RunItemEvent::RoomEntered{kind:entered_kind};
        // M27 seam: RoomEntered (world::enter_standard already dispatched FloorEntered).
        crate::items::dispatch::run_event(&mut staged.state,&mut item_event)?;
    }
    crate::items::on_hook(&mut staged.state,crate::combat::Hook::RoomEntered)?;
    let mut transition=match kind{
        MapNodeKind::Monster|MapNodeKind::Elite|MapNodeKind::Boss=>{
            let encounter=encounter.ok_or(RunError::Registry("standard combat room without encounter"))?;
            match staged.enter_owned(encounter,CombatOrigin::MapRoom{node:target,kind}){
                Ok(transition)=>transition,
                Err(RunError::Faulted(fault))=>{*run=staged;return Err(RunError::Faulted(fault))}
                Err(error)=>return Err(error),
            }
        }
        MapNodeKind::Shop=>super::shop::enter(&mut staged)?,
        MapNodeKind::Rest=>{
            let mut next=staged.next_choice;
            let screen=crate::run::rooms::rest::screen_for_run(&mut staged,&mut next)?;
            staged.next_choice=next;
            staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen=screen;
            crate::run::rooms::rest::complete_if_unusable(&mut staged)?;
            empty_transition()
        }
        MapNodeKind::Treasure=>{
            let mut next=staged.next_choice;
            let screen=view(&mut next,RoomViewKind::Treasure,vec![(RoomChoiceKind::OpenChest,None),(RoomChoiceKind::Proceed,None)])?;
            staged.next_choice=next;
            staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen=screen;
            empty_transition()
        }
        // Design §5.2: the selection law, then the picked event's frame; E6 falls back to a monster
        // room. An event whose body has not landed rejects the move (staged; nothing consumed).
        MapNodeKind::Event=>match super::event_pool::select(&mut staged.state,target)?{
            super::event_pool::Selected::Monster=>{
                let encounter=crate::run::world::standard_event_fallback(&mut staged.state,target)?;
                match staged.enter_owned(encounter,CombatOrigin::MapRoom{node:target,kind:MapNodeKind::Monster}){
                    Ok(transition)=>transition,
                    Err(RunError::Faulted(fault))=>{*run=staged;return Err(RunError::Faulted(fault))}
                    Err(error)=>return Err(error),
                }
            }
            super::event_pool::Selected::Event(event)=>{enter_event(&mut staged,event,target)?;empty_transition()}
        },
        MapNodeKind::Question=>unreachable!("rejected above"),
    };
    transition.reward+=credit;transition.committed_events.insert(0,event);
    *run=staged;
    Ok(transition)
}

/// Construct the selected event: the six accepted events keep their handlers and screens (design
/// §5.5); every other event runs on the standard frame.
fn enter_event(run:&mut Run,event:EventId,node:NodeId)->Result<(),RunError>{
    use crate::run::rooms::events;
    let mut next=run.next_choice;
    match event{
        EventId::UpgradeShrine|EventId::Purifier|EventId::AncientWriting|EventId::Joust=>{
            let screen=events::screen(&mut next,event)?;
            run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen=screen;
        }
        EventId::Falling|EventId::TombOfLordRedMask=>{
            let prepared=events::act3::prepare(run,event,node,0,&mut next)?;
            run.state.rng=prepared.rng;
            let room=run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
            room.act3_event=Some(prepared.frame);room.screen=prepared.screen;
        }
        _=>return super::event_frame::construct(run,event,node),
    }
    run.next_choice=next;
    run.candidates.clear();
    Ok(())
}

/// The ordinary chest's RNG key: act, floor and the current node.
pub(in crate::run) fn chest_key(run:&Run)->Result<[u64;4],RunError>{
    let node=run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
    Ok([u64::from(run.state.act),u64::from(run.state.floor),node.0,0x7C])
}

/// Open the ordinary chest of a standard Treasure room (design §6.4): M27's `onChestOpen` pass,
/// gold and base relic with the linked Sapphire, then the separate after-open pass.
pub(in crate::run) fn open_chest(run:&mut Run)->Result<RunTransition,RunError>{
    let mut staged=run.clone();
    let key=chest_key(&staged)?;
    let rows=super::treasure::open_rows(&mut staged.state,&key)?;
    let mut rows=rows.into_iter().map(|kind|(kind,None)).collect::<Vec<_>>();
    rows.push((RoomChoiceKind::Proceed,None));
    let mut next=staged.next_choice;
    let screen=view(&mut next,RoomViewKind::Treasure,rows)?;
    staged.next_choice=next;
    staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen=screen;
    crate::run::rooms::rewards::chest_opened(&mut staged,false)?;
    crate::items::on_hook(&mut staged.state,crate::combat::Hook::ChestOpened{boss:false})?;
    crate::items::on_hook(&mut staged.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
    *run=staged;
    Ok(empty_transition())
}

/// The standard Treasure claim link (economy research :275-291): claiming the base relic disables
/// its linked Sapphire and vice versa; Matryoshka's extra relic and gold are independent.
pub(in crate::run) fn chest_linked(screen:&RoomView,claimed:ChoiceId)->Option<ChoiceId>{
    let index=screen.choices.iter().position(|row|row.id==claimed)?;
    match screen.choices[index].kind{
        RoomChoiceKind::Sapphire=>index.checked_sub(1).and_then(|i|screen.choices.get(i)).filter(|row|matches!(row.kind,RoomChoiceKind::Relic(_))).map(|row|row.id),
        RoomChoiceKind::Relic(_)=>screen.choices.get(index+1).filter(|row|row.kind==RoomChoiceKind::Sapphire).map(|row|row.id),
        _=>None,
    }
}
