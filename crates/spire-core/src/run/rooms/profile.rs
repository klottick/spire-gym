use super::*;
use crate::content::EncounterId;
pub(super) fn map()->RunMap{
    let slots=[(2,MapNodeKind::Monster),(4,MapNodeKind::Monster),(10,MapNodeKind::Question),(17,MapNodeKind::Shop),(24,MapNodeKind::Monster),(31,MapNodeKind::Question),(38,MapNodeKind::Rest),(45,MapNodeKind::Monster),(52,MapNodeKind::Shop),(59,MapNodeKind::Treasure)];
    let nodes=slots.iter().map(|(id,kind)|{let y=(id/7) as u8;MapNode{id:NodeId(*id),floor:y as u32+1,x:(id%7) as u8,y,kind:*kind,label:kind.label(),edges:if *id==59{vec![]}else{vec![NodeId(if *id<10{10}else{id+7})]},hidden_kind:None,visited:false,revealed:false,encounter:None,burning:false}}).collect();
    RunMap{act:1,nodes,..RunMap::default()}
}
pub(super) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    let node=world::preflight_move(&run.state,target)?;
    let event=CommittedEvent::FloorEntered{act:1,floor:node.floor};reward::preflight(&run.state,std::slice::from_ref(&event))?;
    let kind=if node.kind==MapNodeKind::Question{MapNodeKind::Event}else{node.kind};
    let encounter=match target.0 {2=>Some(EncounterId::Cultist),4=>Some(EncounterId::JawWorm),24=>Some(if run.state.map.as_ref().unwrap().encounter_history.first()==Some(&EncounterId::Cultist){EncounterId::JawWorm}else{EncounterId::Cultist}),45=>Some(EncounterId::TwoLouse),_=>None};
    let map=run.state.map.as_mut().unwrap();map.current=Some(target);
    let n=map.nodes.iter_mut().find(|n|n.id==target).unwrap();n.visited=true;n.revealed=true;
    if n.kind==MapNodeKind::Question {n.hidden_kind=Some(kind)}
    n.encounter=encounter;
    if let Some(e)=encounter {map.encounter_history.push(e);map.regular_encounter_history.push(e)}
    run.state.floor=node.floor;run.state.room=kind.label();
    run.state.history.rooms.push(VisitedRoom{act:1,floor:node.floor,label:kind.label()});
    let credited=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    crate::items::on_hook(&mut run.state,crate::combat::Hook::RoomEntered)?;
    if let Some(e)=encounter {
        let mut t=match run.enter_owned(e,CombatOrigin::MapRoom(target)) {Ok(t)=>t,Err(RunError::Faulted(f))=>return Err(RunError::Faulted(f)),Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("room combat entry failed"),"run.room.combat"))};
        t.committed_events.insert(0,event);t.reward+=credited;return Ok(t)
    }
    let mut next=run.next_choice;
    let built=match target.0 {
        10=>events::screen(&mut next,EventId::UpgradeShrine),31=>events::screen(&mut next,EventId::Purifier),
        17|52=>shop::screen(run,&mut next),38=>rest::screen(&mut next,run.state.max_hp),
        59=>view(&mut next,RoomViewKind::Treasure,vec![(RoomChoiceKind::OpenChest,None),(RoomChoiceKind::Proceed,None)]),
        _=>Err(RunError::InvalidAction),
    };
    let screen=match built {Ok(s)=>s,Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("room preparation failed"),"run.room.entry"))};
    run.next_choice=next;run.state.room_loop.as_mut().unwrap().screen=screen;
    Ok(RunTransition{reward:credited,terminal:None,committed_events:vec![event]})
}
