use super::*;
pub(in crate::run) mod act3;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct JoustProgress {pub node:NodeId,pub visit:u64,pub owner_bet:bool,pub post_cost:u32,pub winner:Option<bool>,pub payout_accepted:bool,pub complete:bool}
pub(in crate::run) struct PreparedJoust {node:NodeId,visit:u64,owner_bet:bool,post_cost:u32,next:u64,screen:RoomView}
pub(in crate::run) fn joust_eligible(state:&RunState)->bool{state.act==2&&state.gold>=50}
pub(in crate::run) fn screen(next:&mut u64,event:EventId)->Result<RoomView,RunError>{
    let rows=match event{
        EventId::AncientWriting=>vec![(RoomChoiceKind::AncientElegance,None),(RoomChoiceKind::AncientSimplicity,None)],
        EventId::Joust=>vec![(RoomChoiceKind::JoustMurderer,Some(50)),(RoomChoiceKind::JoustOwner,Some(50))],
        EventId::UpgradeShrine=>vec![(RoomChoiceKind::ShrineUpgrade,None),(RoomChoiceKind::Leave,None)],
        EventId::Purifier=>vec![(RoomChoiceKind::ShrineRemove,None),(RoomChoiceKind::Leave,None)],
        _=>return Err(RunError::Unsupported("event handler unavailable")),
    };view(next,RoomViewKind::Event(event),rows)
}
pub(in crate::run) fn prepare_joust(run:&Run,id:ChoiceId)->Result<PreparedJoust,RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let row=public(run)?.choices.into_iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered&&c.price==Some(50)).ok_or(RunError::InvalidAction)?;
    if room.screen.kind!=RoomViewKind::Event(EventId::Joust)||room.joust.is_some()||!joust_eligible(&run.state)||run.state.pending.is_some(){return Err(RunError::InvalidAction)}
    let owner_bet=match row.kind{RoomChoiceKind::JoustOwner=>true,RoomChoiceKind::JoustMurderer=>false,_=>return Err(RunError::InvalidAction)};
    let node=run.state.map.as_ref().and_then(|m|m.current).ok_or(RunError::InvalidAction)?;let post_cost=run.state.gold-50;
    post_cost.checked_add(if owner_bet{250}else{100}).ok_or(RunError::Unsupported("Joust payout overflow"))?;
    room.joust_visits.checked_add(1).ok_or(RunError::Unsupported("Joust visit exhausted"))?;
    let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::Event(EventId::Joust),vec![(RoomChoiceKind::JoustContinue,None)])?;
    Ok(PreparedJoust{node,visit:room.joust_visits,owner_bet,post_cost,next,screen})
}
pub(in crate::run) fn complete_ancient(run:&mut Run)->Result<RunTransition,RunError>{
    if run.state.deck.is_empty(){return finish(run)}
    let screen=view(&mut run.next_choice,RoomViewKind::Event(EventId::AncientWriting),vec![(RoomChoiceKind::Leave,None)])?;
    run.state.room_loop.as_mut().unwrap().screen=screen;run.candidates.clear();Ok(empty_transition())
}
pub(in crate::run) fn drive_joust_payout(run:&mut Run)->Result<RunTransition,RunError>{
    if let Some(f)=&run.state.fault{return Err(RunError::Faulted(f.clone()))}
    let frame=run.state.room_loop.as_ref().and_then(|r|r.joust.as_ref()).cloned().ok_or(RunError::InvalidAction)?;
    if !frame.payout_accepted||frame.complete||frame.winner.is_none()||run.state.map.as_ref().and_then(|m|m.current)!=Some(frame.node)||run.state.gold!=frame.post_cost {
        return Err(run.retain_fault(RunFaultKind::Invariant("accepted Joust payout parent changed"),"run.event.joust"))
    }
    let payout=if frame.winner==Some(frame.owner_bet){if frame.owner_bet{250}else{100}}else{0};
    let mut next=run.next_choice;let screen=match view(&mut next,RoomViewKind::Event(EventId::Joust),vec![(RoomChoiceKind::Leave,None)]){Ok(s)=>s,Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("accepted Joust completion publication failed"),"run.event.joust"))};
    if mutations::gain_gold(&mut run.state,payout).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Joust gold mutation failed"),"run.event.joust"))}
    let room=run.state.room_loop.as_mut().unwrap();room.joust.as_mut().unwrap().complete=true;room.screen=screen;run.next_choice=next;Ok(empty_transition())
}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    if matches!(public(run)?.kind,RoomViewKind::Event(EventId::Falling|EventId::TombOfLordRedMask)){return act3::choose(run,id)}
    let screen=public(run)?;let row=screen.choices.into_iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    match (screen.kind,row.kind){
        (RoomViewKind::Event(EventId::Joust),RoomChoiceKind::JoustMurderer|RoomChoiceKind::JoustOwner)=>{
            let p=prepare_joust(run,id)?;run.state.gold=p.post_cost;run.next_choice=p.next;let room=run.state.room_loop.as_mut().unwrap();room.screen=p.screen;room.joust_visits+=1;room.joust=Some(JoustProgress{node:p.node,visit:p.visit,owner_bet:p.owner_bet,post_cost:p.post_cost,winner:None,payout_accepted:false,complete:false});Ok(empty_transition())
        },
        (RoomViewKind::Event(EventId::Joust),RoomChoiceKind::JoustContinue)=>{
            let frame=run.state.room_loop.as_ref().and_then(|r|r.joust.as_ref()).ok_or(RunError::InvalidAction)?;
            if row.price.is_some()||frame.winner.is_some()||frame.complete||frame.payout_accepted||run.state.map.as_ref().and_then(|m|m.current)!=Some(frame.node)||run.state.gold!=frame.post_cost{return Err(RunError::InvalidAction)}
            let mut rng=run.state.rng.clone();let winner=rng.bounded_scoped(RunRngScope::Act2Joust,&[run.state.act as u64,frame.node.0,frame.visit],100)<30;
            let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::Event(EventId::Joust),vec![(RoomChoiceKind::JoustResult{owner_wins:winner},None)])?;
            run.state.rng=rng;run.next_choice=next;let room=run.state.room_loop.as_mut().unwrap();room.joust.as_mut().unwrap().winner=Some(winner);room.screen=screen;Ok(empty_transition())
        },
        (RoomViewKind::Event(EventId::Joust),RoomChoiceKind::JoustResult{owner_wins})=>{
            let frame=run.state.room_loop.as_ref().and_then(|r|r.joust.as_ref()).ok_or(RunError::InvalidAction)?;
            if row.price.is_some()||frame.winner!=Some(owner_wins)||frame.complete||frame.payout_accepted||run.state.map.as_ref().and_then(|m|m.current)!=Some(frame.node)||run.state.gold!=frame.post_cost{return Err(RunError::InvalidAction)}
            run.next_choice.checked_add(1).ok_or(RunError::Unsupported("choice identity exhausted"))?;
            run.state.room_loop.as_mut().unwrap().joust.as_mut().unwrap().payout_accepted=true;drive_joust_payout(run)
        },
        (RoomViewKind::Event(EventId::Joust),RoomChoiceKind::Leave)=>{
            if row.price.is_some()||!run.state.room_loop.as_ref().and_then(|r|r.joust.as_ref()).is_some_and(|p|p.complete&&run.state.map.as_ref().and_then(|m|m.current)==Some(p.node)){return Err(RunError::InvalidAction)}run.state.room_loop.as_mut().unwrap().joust=None;finish(run)
        },
        (RoomViewKind::Event(EventId::AncientWriting),RoomChoiceKind::AncientSimplicity)=>{
            if row.price.is_some(){return Err(RunError::InvalidAction)}run.next_choice.checked_add(1).ok_or(RunError::Unsupported("choice identity exhausted"))?;
            let ids=run.state.deck.iter().filter(|c|(matches!(c.id,CardId::Strike|CardId::Defend)||uses_silent_content(&run.state)&&matches!(c.id,CardId::StrikeGreen|CardId::DefendGreen)||uses_defect_content(&run.state)&&matches!(c.id,CardId::StrikeBlue|CardId::DefendBlue))&&c.upgrades==0).map(|c|c.instance).collect::<Vec<_>>();for id in ids{mutations::upgrade_card(&mut run.state,id)?;}complete_ancient(run)
        },
        (RoomViewKind::Event(EventId::AncientWriting),RoomChoiceKind::AncientElegance)=>{
            if row.price.is_some(){return Err(RunError::InvalidAction)}if run.state.deck.is_empty(){complete_ancient(run)}else{selection::open(run,id,RoomSelectionOperation::AncientElegance)}
        },
        (RoomViewKind::Event(_),RoomChoiceKind::Leave) if row.price.is_none()=>finish(run),
        (RoomViewKind::Event(EventId::UpgradeShrine),RoomChoiceKind::ShrineUpgrade)=>selection::open(run,id,RoomSelectionOperation::ShrineUpgrade),
        (RoomViewKind::Event(EventId::Purifier),RoomChoiceKind::ShrineRemove)=>selection::open(run,id,RoomSelectionOperation::ShrineRemove),_=>Err(RunError::InvalidAction),
    }
}
