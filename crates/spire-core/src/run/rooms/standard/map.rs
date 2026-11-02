//! Standard map-room entry (design §7): `world::enter_standard` commits the move, then the room
//! producer for the resolved kind runs. The whole entry is staged: a rejected or interim room
//! leaves the Run unchanged; a combat-entry fault keeps its committed progress as the captured
//! owners do.
use super::*;

/// Typed interim until M28c lands the event core (chunk plan §3).
pub(in crate::run) const EVENT_ROOMS_INTERIM:&str="M28 standard event rooms land in M28c";

pub(in crate::run) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    if !is_standard(&run.state)||run.state.act>3{return Err(RunError::InvalidAction)}
    let mut staged=run.clone();
    let (kind,encounter,event,credit)=crate::run::world::enter_standard(&mut staged.state,target)?;
    let entered_kind=match kind{
        MapNodeKind::Monster|MapNodeKind::Elite|MapNodeKind::Boss=>RoomViewKind::Combat,
        MapNodeKind::Shop=>RoomViewKind::Shop,
        MapNodeKind::Rest=>RoomViewKind::Campfire,
        MapNodeKind::Treasure=>RoomViewKind::Treasure,
        MapNodeKind::Event|MapNodeKind::Question=>return Err(RunError::Unsupported(EVENT_ROOMS_INTERIM)),
    };
    let room=staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.active_rewards=None;room.reward_parent=None;
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
        MapNodeKind::Event|MapNodeKind::Question=>unreachable!("rejected above"),
    };
    transition.reward+=credit;transition.committed_events.insert(0,event);
    *run=staged;
    Ok(transition)
}

/// Open the ordinary chest of a standard Treasure room (design §6.4): M27's `onChestOpen` pass,
/// gold and base relic with the linked Sapphire, then the separate after-open pass.
pub(in crate::run) fn open_chest(run:&mut Run)->Result<RunTransition,RunError>{
    let mut staged=run.clone();
    let node=staged.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
    let key=[u64::from(staged.state.act),u64::from(staged.state.floor),node.0,0x7C];
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
