use super::*;
use crate::content::EncounterId;
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub(crate) struct Act2Progress {pub question_visits:usize,pub shop_visits:usize,pub combat_visits:u64,pub elite_relic_cursor:usize}
pub(in crate::run) fn capture_room(state:&RunState,node:NodeId)->Result<super::act1::CapturedRoom,RunError>{
    if !uses_captured_act2(state)||state.act!=2{return Err(RunError::InvalidAction)}
    let map=state.map.as_ref().filter(|m|m.act==2).ok_or(RunError::InvalidAction)?;let n=map.nodes.iter().find(|n|n.id==node).ok_or(RunError::InvalidAction)?;
    let continues_to_boss=is_m7(state)||is_m13(state)||uses_captured_act3(state);
    let last=if continues_to_boss{33}else{26};
    if !(18..=last).contains(&n.floor){return Err(RunError::InvalidAction)}
    let p=state.room_loop.as_ref().and_then(|r|r.act2.as_ref()).ok_or(RunError::InvalidAction)?;let mut kind=n.kind;let mut event=None;
    if kind==MapNodeKind::Question{match p.question_visits{0=>{kind=MapNodeKind::Event;event=Some(EventId::AncientWriting)},1=>{kind=MapNodeKind::Event;event=Some(if events::joust_eligible(state){EventId::Joust}else{EventId::Purifier})},_=>kind=MapNodeKind::Monster}}
    let encounter=match kind {
        MapNodeKind::Monster=>{
            let history=&map.regular_encounter_history;let id=match history.len(){0=>EncounterId::SphericGuardian,1=>EncounterId::ThreeByrds,2=>EncounterId::CenturionAndHealer,i=>{
                let cycle=[EncounterId::Snecko,EncounterId::ChosenAndByrds,EncounterId::ThreeCultists,EncounterId::CenturionAndHealer];
                (0..4).map(|offset|cycle[(i-3+offset)%4]).find(|id|!history.iter().rev().take(2).any(|old|old==id)).ok_or(RunError::InvalidAction)?
            }};Some(id)
        },
        MapNodeKind::Elite=>{
            let offset=match map.boss_icon{Some(EncounterId::Champ)=>0,Some(EncounterId::BronzeAutomaton)=>1,Some(EncounterId::Collector)=>2,_=>return Err(RunError::InvalidAction)};
            let cycle=[EncounterId::GremlinLeader,EncounterId::Slavers,EncounterId::BookOfStabbing];let history=&map.elite_encounter_history;
            Some((0..3).map(|i|cycle[(history.len()+offset+i)%3]).find(|id|history.last()!=Some(id)).ok_or(RunError::InvalidAction)?)
        },
        MapNodeKind::Boss if continues_to_boss&&n.floor==33=>{
            let boss=state.room_loop.as_ref().and_then(|r|r.act2_boss_order.as_ref()).and_then(|order|order.first()).copied().ok_or(RunError::InvalidAction)?;
            if map.boss_icon!=Some(boss)||!matches!(boss,EncounterId::Champ|EncounterId::BronzeAutomaton|EncounterId::Collector){return Err(RunError::InvalidAction)}
            Some(boss)
        },
        MapNodeKind::Event|MapNodeKind::Shop|MapNodeKind::Rest|MapNodeKind::Treasure=>None,
        _=>return Err(RunError::Unsupported("room beyond declared M6 prefix")),
    };Ok(super::act1::CapturedRoom{kind,encounter,event})
}

pub(super) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    let node=world::preflight_move(&run.state,target)?;let captured=capture_room(&run.state,target)?;
    let p=run.state.room_loop.as_ref().and_then(|r|r.act2.as_ref()).ok_or(RunError::InvalidAction)?;
    let questions=if node.kind==MapNodeKind::Question{p.question_visits.checked_add(1).ok_or(RunError::Unsupported("Act2 question visits exhausted"))?}else{p.question_visits};
    let shops=if captured.kind==MapNodeKind::Shop{p.shop_visits.checked_add(1).ok_or(RunError::Unsupported("Act2 shop visits exhausted"))?}else{p.shop_visits};
    let mut next=run.next_choice;let mut rng=run.state.rng.clone();let screen=if captured.encounter.is_none(){Some(match captured.kind{
        MapNodeKind::Event=>events::screen(&mut next,captured.event.ok_or(RunError::InvalidAction)?)?,
        MapNodeKind::Shop=>shop::screen_with_rng_at(run,&mut next,&mut rng,(2,node.floor,target))?,MapNodeKind::Rest=>rest::screen_for_run(run,&mut next)?,
        MapNodeKind::Treasure=>view(&mut next,RoomViewKind::Treasure,vec![(RoomChoiceKind::OpenChest,None),(RoomChoiceKind::Proceed,None)])?,_=>return Err(RunError::InvalidAction),
    })}else{None};
    let event=CommittedEvent::FloorEntered{act:2,floor:node.floor};reward::preflight(&run.state,std::slice::from_ref(&event))?;
    let map=run.state.map.as_mut().unwrap();map.current=Some(target);let n=map.nodes.iter_mut().find(|n|n.id==target).unwrap();n.visited=true;n.revealed=true;n.encounter=captured.encounter;if node.kind==MapNodeKind::Question{n.hidden_kind=Some(captured.kind)}
    if let Some(id)=captured.encounter{map.encounter_history.push(id);match captured.kind{MapNodeKind::Monster=>map.regular_encounter_history.push(id),MapNodeKind::Elite=>map.elite_encounter_history.push(id),_=>{}}}
    run.state.floor=node.floor;run.state.room=captured.kind.label();run.state.history.rooms.push(VisitedRoom{act:2,floor:node.floor,label:captured.kind.label()});
    let room=run.state.room_loop.as_mut().unwrap();let p=room.act2.as_mut().unwrap();p.question_visits=questions;p.shop_visits=shops;room.joust=None;
    let credit=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    if crate::items::on_hook(&mut run.state,crate::combat::Hook::RoomEntered).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted M6 room-entry hook failed"),"run.m6.room"))}
    if let Some(encounter)=captured.encounter{
        let entered=run.enter_owned(encounter,CombatOrigin::MapRoom{node:target,kind:captured.kind});
        // Entry's native owner records accepted transfers even when startup faults.
        // Consume the selected boss only then, never during detached preparation.
        if captured.kind==MapNodeKind::Boss&&run.state.active_combat.as_ref().is_some_and(|active|active.encounter==encounter){
            run.state.room_loop.as_mut().unwrap().act2_boss_order.as_mut().unwrap().remove(0);
        }
        let mut t=match entered{Ok(t)=>t,Err(RunError::Faulted(f))=>return Err(RunError::Faulted(f)),Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("accepted M6 encounter entry failed"),"run.m6.encounter"))};
        t.reward+=credit;t.committed_events.insert(0,event);return Ok(t)
    }
    run.next_choice=next;if uses_act4(&run.state)&&captured.kind==MapNodeKind::Shop{run.state.rng=rng;}run.state.room_loop.as_mut().unwrap().screen=screen.unwrap();Ok(RunTransition{reward:credit,terminal:None,committed_events:vec![event]})
}

pub(super) fn finish_room(run:&mut Run)->Result<RunTransition,RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if !uses_captured_act2(&run.state)||run.state.act!=2||run.state.combat.is_some()||run.state.active_combat.is_some()||run.state.pending.is_some()||room.reward_parent.is_some(){return Err(RunError::InvalidAction)}
    let continues_to_boss=is_m7(&run.state)||is_m13(&run.state)||uses_captured_act3(&run.state);
    let kind=if continues_to_boss&&(18..=32).contains(&run.state.floor){RoomViewKind::Map}else if run.state.floor==26{
        if room.screen.kind!=RoomViewKind::Treasure||!run.state.map.as_ref().is_some_and(|m|m.current.is_some_and(|id|m.nodes.iter().any(|n|n.id==id&&n.floor==26&&n.kind==MapNodeKind::Treasure&&n.visited))){return Err(RunError::InvalidAction)}
        if continues_to_boss{RoomViewKind::Map}else{RoomViewKind::Act2PrefixComplete}
    }else if (18..26).contains(&run.state.floor){RoomViewKind::Map}else{return Err(RunError::InvalidAction)};
    run.state.room_loop.as_mut().unwrap().screen=RoomView{kind,choices:vec![]};run.state.room=if kind==RoomViewKind::Act2PrefixComplete{"Act2PrefixComplete"}else{"Map"};run.candidates.clear();Ok(empty_transition())
}
