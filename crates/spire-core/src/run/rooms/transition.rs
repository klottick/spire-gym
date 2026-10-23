use super::*;
use crate::content::EncounterId;

pub(super) fn begin(run:&mut Run)->Result<RunTransition,RunError>{
    let (from,to,floor,boss_floor)=match run.state.act {
        1 if uses_captured_act1(&run.state)=>(1,2,17,16),
        2 if is_m7(&run.state)=>(2,3,34,33),
        _=>return Err(RunError::InvalidAction),
    };
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let boss=room.boss.as_ref().ok_or(RunError::InvalidAction)?;
    let source=match room.screen.kind {
        RoomViewKind::BossRelicChild=>boss.selected.is_some()&&boss.source_complete,
        RoomViewKind::ProceedWithoutRelic=>boss.selected.is_none()&&boss.skip_parent.as_ref().is_some_and(|p|matches!(p.kind,RoomViewKind::BossTreasure|RoomViewKind::BossRelicChoice)),
        _=>false,
    };
    let predecessor=match (&room.transition,from) {
        (None,1)=>true,
        (Some(frame),2)=>frame.from==1&&frame.to==2&&frame.floor==17&&frame.reset_applied&&frame.heal_applied&&frame.initialized,
        _=>false,
    };
    if !source||!predecessor||run.state.floor!=floor||run.state.pending.is_some()||run.state.combat.is_some()||run.state.active_combat.is_some()||run.state.terminal.is_some()||run.state.deck.is_empty()||run.state.hp==0||run.state.hp>run.state.max_hp||run.state.max_hp>999{return Err(RunError::InvalidAction)}
    if !run.state.map.as_ref().is_some_and(|map|map.act==from&&map.current.is_some_and(|id|map.nodes.iter().any(|n|n.id==id&&n.kind==MapNodeKind::Boss&&n.floor==boss_floor&&n.visited))){return Err(RunError::InvalidAction)}
    run.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from,to,floor,reset_applied:false,heal_applied:false,initialized:false});
    drive(run)
}

fn successor(run:&Run,frame:&ActTransition)->Result<(RunMap,RunRng,Vec<EncounterId>),RunError>{
    let mut rng=run.state.rng.clone();
    let (mut order,priority,scope)=match frame.to {
        2=>(vec![EncounterId::BronzeAutomaton,EncounterId::Collector,EncounterId::Champ],[("Champ",EncounterId::Champ),("Automaton",EncounterId::BronzeAutomaton),("Collector",EncounterId::Collector)],RunRngScope::Act2BossOrder),
        3=>(vec![EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca],[("AwakenedOne",EncounterId::AwakenedOne),("DonuAndDeca",EncounterId::DonuAndDeca),("TimeEater",EncounterId::TimeEater)],RunRngScope::Act3BossOrder),
        _=>return Err(RunError::InvalidAction),
    };
    let unseen=priority.into_iter().find(|(name,_)|!run.state.seen_bosses[(frame.to-1) as usize].contains(name));
    if let Some((_,boss))=unseen {order=vec![boss];}else{for i in (1..order.len()).rev(){let j=rng.bounded(scope,(i+1) as u64);order.swap(i,j);}}
    let mut map=super::super::map::generate(frame.to,&mut rng,super::super::map::MapEligibility::from_run(&run.state))?;
    for node in &mut map.nodes{node.floor=node.floor.checked_add(frame.floor).ok_or(RunError::Unsupported("successor global floor overflow"))?;}
    map.boss_icon=Some(order[0]);Ok((map,rng,order))
}

pub(in crate::run) fn drive(run:&mut Run)->Result<RunTransition,RunError>{
    if let Some(fault)=&run.state.fault{return Err(RunError::Faulted(fault.clone()))}
    let frame=run.state.room_loop.as_ref().and_then(|r|r.transition.as_ref()).cloned().ok_or(RunError::InvalidAction)?;
    if frame.initialized{return Err(RunError::InvalidAction)}
    if !frame.reset_applied {
        if run.state.act!=frame.from{return Err(run.retain_fault(RunFaultKind::Invariant("transition source act changed"),"run.act.transition"))}
        run.state.act=frame.to;run.state.map=None;run.state.room="ActTransition";run.candidates.clear();
        let room=run.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=None;room.joust=None;room.joust_visits=0;room.active_rewards=None;room.reward_parent=None;room.boss=None;room.act2_boss_order=None;room.act3_boss_order=None;room.transition.as_mut().unwrap().reset_applied=true;
    }
    if !frame.heal_applied {
        let amount=run.state.max_hp;
        if mutations::heal(&mut run.state,amount).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted act healing failed"),"run.act.transition"))}
        run.state.room_loop.as_mut().unwrap().transition.as_mut().unwrap().heal_applied=true;
    }
    let valid=(frame.from,frame.to,frame.floor)==(1,2,17)||((frame.from,frame.to,frame.floor)==(2,3,34)&&is_m7(&run.state));
    if !valid||run.state.floor!=frame.floor||run.state.act!=frame.to{return Err(run.retain_fault(RunFaultKind::Invariant("accepted successor transition invariant"),"run.act.transition"))}
    let (map,rng,order)=match successor(run,&frame){Ok(value)=>value,Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("accepted successor map initialization failed"),"run.act.transition"))};
    let event=CommittedEvent::ActAdvanced{from:frame.from,to:frame.to};
    if reward::preflight(&run.state,std::slice::from_ref(&event)).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted act event preflight failed"),"run.act.transition"))}
    let continuing=frame.to==2&&uses_captured_act2(&run.state);
    let kind=if frame.to==3{RoomViewKind::Act3Ready}else if continuing{RoomViewKind::Map}else{RoomViewKind::Act2Ready};
    run.state.map=Some(map);run.state.rng=rng;run.state.room=if kind==RoomViewKind::Act3Ready{"Act3Ready"}else if continuing{"Map"}else{"Act2Ready"};
    let room=run.state.room_loop.as_mut().unwrap();if frame.to==2{room.act2_boss_order=Some(order);}else{room.act3_boss_order=Some(order);}
    room.act2=if continuing{Some(Default::default())}else{None};room.screen=RoomView{kind,choices:vec![]};room.transition.as_mut().unwrap().initialized=true;
    let delta=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    Ok(RunTransition{reward:delta,terminal:None,committed_events:vec![event]})
}
