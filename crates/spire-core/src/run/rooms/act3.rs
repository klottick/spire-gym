use super::*;
use crate::content::EncounterId;
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub(crate) struct Act3Progress{pub question_visits:usize,pub shop_visits:usize,pub combat_visits:u64,pub elite_relic_cursor:usize}
pub(in crate::run) fn capture_room(state:&RunState,node:NodeId)->Result<super::act1::CapturedRoom,RunError>{
    if !uses_captured_act3(state)||state.act!=3{return Err(RunError::InvalidAction)}
    let map=state.map.as_ref().filter(|m|m.act==3).ok_or(RunError::InvalidAction)?;let n=map.nodes.iter().find(|n|n.id==node).ok_or(RunError::InvalidAction)?;
    if !(35..=if uses_full_act3(state){50}else{43}).contains(&n.floor){return Err(RunError::InvalidAction)}
    let p=state.room_loop.as_ref().and_then(|r|r.act3.as_ref()).ok_or(RunError::InvalidAction)?;let mut kind=n.kind;let mut event=None;
    if kind==MapNodeKind::Question{match p.question_visits{0=>{kind=MapNodeKind::Event;event=Some(EventId::Falling)},1=>{kind=MapNodeKind::Event;event=Some(EventId::TombOfLordRedMask)},_=>kind=MapNodeKind::Monster}}
    let encounter=match kind{
        MapNodeKind::Monster=>{let history=&map.regular_encounter_history;Some(match history.len(){0=>EncounterId::OrbWalker,1=>EncounterId::ThreeShapes,i=>{
            let cycle=[EncounterId::SpireGrowth,EncounterId::Transient,EncounterId::FourShapes,EncounterId::Maw,EncounterId::SphereAndTwoShapes,EncounterId::JawWormHorde,EncounterId::ThreeDarklings,EncounterId::WrithingMass];
            (0..cycle.len()).map(|offset|cycle[((i-2)%cycle.len()+offset)%cycle.len()]).find(|id|{
                if i==2&&history.last()==Some(&EncounterId::ThreeShapes)&&*id==EncounterId::FourShapes{return false}
                !history.iter().rev().take(2).any(|old|old==id||matches!((old,id),(EncounterId::ThreeShapes|EncounterId::FourShapes,EncounterId::ThreeShapes|EncounterId::FourShapes)))
            }).ok_or(RunError::InvalidAction)?
        }})},
        MapNodeKind::Elite=>{let offset=match map.boss_icon{Some(EncounterId::AwakenedOne)=>0,Some(EncounterId::TimeEater)=>1,Some(EncounterId::DonuAndDeca)=>2,_=>return Err(RunError::InvalidAction)};let cycle=[EncounterId::GiantHead,EncounterId::Nemesis,EncounterId::Reptomancer];Some((0..3).map(|i|cycle[(map.elite_encounter_history.len()%3+offset+i)%3]).find(|id|map.elite_encounter_history.last()!=Some(id)).ok_or(RunError::InvalidAction)?)},
        MapNodeKind::Boss if uses_full_act3(state)&&n.floor==50=>{
            let boss=state.room_loop.as_ref().and_then(|r|r.act3_boss_order.as_ref()).and_then(|order|order.first()).copied().ok_or(RunError::InvalidAction)?;
            if !matches!(boss,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca)||map.boss_icon!=Some(boss){return Err(RunError::InvalidAction)}
            Some(boss)
        },
        MapNodeKind::Event|MapNodeKind::Shop|MapNodeKind::Rest|MapNodeKind::Treasure=>None,
        _=>return Err(RunError::Unsupported("room beyond declared Act3 capture")),
    };Ok(super::act1::CapturedRoom{kind,encounter,event})
}

pub(super) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    let node=world::preflight_move(&run.state,target)?;let captured=capture_room(&run.state,target)?;
    let p=run.state.room_loop.as_ref().and_then(|r|r.act3.as_ref()).ok_or(RunError::InvalidAction)?;
    let questions=if node.kind==MapNodeKind::Question{p.question_visits.checked_add(1).ok_or(RunError::Unsupported("Act3 question visits exhausted"))?}else{p.question_visits};
    let shops=if captured.kind==MapNodeKind::Shop{p.shop_visits.checked_add(1).ok_or(RunError::Unsupported("Act3 shop visits exhausted"))?}else{p.shop_visits};
    let mut next=run.next_choice;let mut shop_rng=run.state.rng.clone();let prepared_event=if let Some(event)=captured.event{Some(events::act3::prepare(run,event,target,u64::try_from(p.question_visits).map_err(|_|RunError::InvalidAction)?,&mut next)?)}else{None};
    let screen=if let Some(event)=&prepared_event{Some(event.screen.clone())}else if captured.encounter.is_none(){Some(match captured.kind{
        MapNodeKind::Shop=>shop::screen_with_rng_at(run,&mut next,&mut shop_rng,(3,node.floor,target))?,MapNodeKind::Rest=>rest::screen_for_run(run,&mut next)?,
        MapNodeKind::Treasure=>view(&mut next,RoomViewKind::Treasure,vec![(RoomChoiceKind::OpenChest,None),(RoomChoiceKind::Proceed,None)])?,_=>return Err(RunError::InvalidAction),
    })}else{None};
    let event=CommittedEvent::FloorEntered{act:3,floor:node.floor};reward::preflight(&run.state,std::slice::from_ref(&event))?;
    if uses_act4(&run.state)&&captured.kind==MapNodeKind::Shop{run.state.rng=shop_rng;}
    let map=run.state.map.as_mut().unwrap();map.current=Some(target);let n=map.nodes.iter_mut().find(|n|n.id==target).unwrap();n.visited=true;n.revealed=true;n.encounter=captured.encounter;if node.kind==MapNodeKind::Question{n.hidden_kind=Some(captured.kind)}
    if let Some(id)=captured.encounter{map.encounter_history.push(id);match captured.kind{MapNodeKind::Monster=>map.regular_encounter_history.push(id),MapNodeKind::Elite=>map.elite_encounter_history.push(id),_=>{}}}
    run.state.floor=node.floor;run.state.room=captured.kind.label();run.state.history.rooms.push(VisitedRoom{act:3,floor:node.floor,label:captured.kind.label()});
    let room=run.state.room_loop.as_mut().unwrap();let p=room.act3.as_mut().unwrap();p.question_visits=questions;p.shop_visits=shops;room.act3_event=None;
    if let Some(screen)=screen{room.screen=screen;run.next_choice=next;}
    if let Some(prepared)=prepared_event{room.act3_event=Some(prepared.frame);run.state.rng=prepared.rng;}
    let credit=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    let (hook_error,hook_checkpoint,entry_error,entry_checkpoint)=if is_m10(&run.state){("accepted M10 room-entry hook failed","run.m10.room","accepted M10 encounter entry failed","run.m10.encounter")}else if is_m9(&run.state){("accepted M9 room-entry hook failed","run.m9.room","accepted M9 encounter entry failed","run.m9.encounter")}else if is_m14(&run.state){("accepted M14 room-entry hook failed","run.m14.room","accepted M14 encounter entry failed","run.m14.encounter")}else{("accepted M8 room-entry hook failed","run.m8.room","accepted M8 encounter entry failed","run.m8.encounter")};
    if crate::items::on_hook(&mut run.state,crate::combat::Hook::RoomEntered).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant(hook_error),hook_checkpoint))}
    if let Some(encounter)=captured.encounter{let mut t=match run.enter_owned(encounter,CombatOrigin::MapRoom{node:target,kind:captured.kind}){Ok(t)=>t,Err(RunError::Faulted(f))=>return Err(RunError::Faulted(f)),Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant(entry_error),entry_checkpoint))};t.reward+=credit;t.committed_events.insert(0,event);return Ok(t)}
    Ok(RunTransition{reward:credit,terminal:None,committed_events:vec![event]})
}

pub(super) fn finish_room(run:&mut Run)->Result<RunTransition,RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if !uses_captured_act3(&run.state)||run.state.act!=3||run.state.combat.is_some()||run.state.active_combat.is_some()||run.state.pending.is_some()||room.reward_parent.is_some(){return Err(RunError::InvalidAction)}
    let kind=if run.state.floor==43{
        if room.screen.kind!=RoomViewKind::Treasure||!run.state.map.as_ref().is_some_and(|m|m.current.is_some_and(|id|m.nodes.iter().any(|n|n.id==id&&n.floor==43&&n.kind==MapNodeKind::Treasure&&n.visited))){return Err(RunError::InvalidAction)}if is_m8(&run.state){RoomViewKind::Act3PrefixComplete}else{RoomViewKind::Map}
    }else if (35..=if uses_full_act3(&run.state){49}else{42}).contains(&run.state.floor){RoomViewKind::Map}else{return Err(RunError::InvalidAction)};
    let room=run.state.room_loop.as_mut().unwrap();room.screen=RoomView{kind,choices:vec![]};run.state.room=if kind==RoomViewKind::Act3PrefixComplete{"Act3PrefixComplete"}else{"Map"};run.candidates.clear();Ok(empty_transition())
}
