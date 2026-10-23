//! Full generated topology with explicitly captured, not standard-random content.
use super::*;
use crate::content::EncounterId;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Act1Progress {pub boss_order:Vec<EncounterId>,pub question_visits:usize,pub shop_visits:usize,pub elite_relic_cursor:usize}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(in crate::run) struct CapturedRoom {pub kind:MapNodeKind,pub encounter:Option<EncounterId>,pub event:Option<EventId>}

pub(super) fn start(run:&mut Run)->Result<(),RunError>{
    let mut rng=run.state.rng.clone();
    let mut bosses=vec![EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss];
    let names=["Guardian","Hexaghost","SlimeBoss"];
    if let Some(i)=names.iter().position(|name|!run.state.seen_bosses[0].contains(name)){bosses=vec![bosses[i];3]}
    else{for i in (1..bosses.len()).rev(){let j=rng.bounded(RunRngScope::Act1BossOrder,(i+1) as u64);bosses.swap(i,j);}}
    let mut map=super::super::map::generate(1,&mut rng,super::super::map::MapEligibility::from_run(&run.state))?;
    map.boss_icon=Some(bosses[0]);run.state.map=Some(map);run.state.rng=rng;
    run.state.room_loop.as_mut().unwrap().act1=Some(Act1Progress{boss_order:bosses,question_visits:0,shop_visits:0,elite_relic_cursor:0});Ok(())
}
pub(super) fn capture_room(state:&RunState,node:NodeId)->Result<CapturedRoom,RunError>{
    let map=state.map.as_ref().ok_or(RunError::InvalidAction)?;
    let n=map.nodes.iter().find(|n|n.id==node).ok_or(RunError::InvalidAction)?;
    let progress=state.room_loop.as_ref().and_then(|r|r.act1.as_ref()).ok_or(RunError::InvalidAction)?;
    let mut kind=n.kind;let mut event=None;
    if kind==MapNodeKind::Question {
        match progress.question_visits {0=>{kind=MapNodeKind::Event;event=Some(EventId::UpgradeShrine)},1=>{kind=MapNodeKind::Event;event=Some(EventId::Purifier)},_=>kind=MapNodeKind::Monster}
    }
    let encounter=match kind {
        MapNodeKind::Monster=>Some(match map.regular_encounter_history.len(){0=>EncounterId::Cultist,1=>EncounterId::JawWorm,2=>EncounterId::TwoLouse,i=>[EncounterId::LargeSlime,EncounterId::ThreeLouse,EncounterId::LotsOfSlimes][(i-3)%3]}),
        MapNodeKind::Elite=>{let offset=match progress.boss_order[0]{EncounterId::Guardian=>0,EncounterId::Hexaghost=>1,_=>2};Some([EncounterId::GremlinNob,EncounterId::Lagavulin,EncounterId::ThreeSentries][(map.elite_encounter_history.len()+offset)%3])},
        MapNodeKind::Boss=>Some(progress.boss_order[0]),_=>None,
    };Ok(CapturedRoom{kind,encounter,event})
}
pub(super) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    let node=world::preflight_move(&run.state,target)?;let captured=capture_room(&run.state,target)?;
    let event=CommittedEvent::FloorEntered{act:run.state.act,floor:node.floor};reward::preflight(&run.state,std::slice::from_ref(&event))?;
    let map=run.state.map.as_mut().unwrap();map.current=Some(target);
    let n=map.nodes.iter_mut().find(|n|n.id==target).unwrap();n.visited=true;n.revealed=true;n.encounter=captured.encounter;
    if node.kind==MapNodeKind::Question{n.hidden_kind=Some(captured.kind)}
    if let Some(e)=captured.encounter{map.encounter_history.push(e);match captured.kind{MapNodeKind::Monster=>map.regular_encounter_history.push(e),MapNodeKind::Elite=>map.elite_encounter_history.push(e),_=>{}}}
    if node.kind==MapNodeKind::Question{run.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().question_visits+=1;}
    run.state.floor=node.floor;run.state.room=captured.kind.label();run.state.history.rooms.push(VisitedRoom{act:run.state.act,floor:node.floor,label:captured.kind.label()});
    let credit=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    crate::items::on_hook(&mut run.state,crate::combat::Hook::RoomEntered)?;
    if let Some(encounter)=captured.encounter {
        let mut transition=match run.enter_owned(encounter,CombatOrigin::MapRoom{node:target,kind:captured.kind}){Ok(t)=>t,Err(RunError::Faulted(f))=>return Err(RunError::Faulted(f)),Err(_)=>return Err(run.retain_fault(RunFaultKind::UnsupportedBoundary("M5 encounter entry"),"run.m5.encounter"))};
        transition.reward+=credit;transition.committed_events.insert(0,event);return Ok(transition)
    }
    let mut next=run.next_choice;
    let screen=match captured.kind {
        MapNodeKind::Event=>events::screen(&mut next,captured.event.ok_or(RunError::InvalidAction)?),
        MapNodeKind::Shop=>shop::screen(run,&mut next),
        MapNodeKind::Rest=>rest::screen(&mut next,run.state.max_hp),
        MapNodeKind::Treasure=>view(&mut next,RoomViewKind::Treasure,vec![(RoomChoiceKind::OpenChest,None),(RoomChoiceKind::Proceed,None)]),
        _=>Err(RunError::InvalidAction),
    };
    let screen=match screen{Ok(s)=>s,Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("M5 room preparation failed"),"run.m5.room"))};
    if captured.kind==MapNodeKind::Shop{run.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().shop_visits+=1;}
    run.next_choice=next;run.state.room_loop.as_mut().unwrap().screen=screen;
    Ok(RunTransition{reward:credit,terminal:None,committed_events:vec![event]})
}
