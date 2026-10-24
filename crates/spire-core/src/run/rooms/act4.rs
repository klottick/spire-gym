//! Source-qualified M10 Act4 door transition and fixed Rest→Shop→Elite→Heart route.
use super::*;
use crate::{combat::Outcome,content::EncounterId};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Act4TransitionPhase { ChoiceAccepted, DungeonInitialized, Healed, Published }

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Act4Progress {
    pub route:[NodeId;4],
    pub source_node:NodeId,
    pub source_record:CombatRecord,
    pub accepted_choice:ChoiceId,
    pub phase:Act4TransitionPhase,
    pub combat_visits:u64,
}

pub(in crate::run) struct PreparedAct4 {
    map:RunMap,
    progress:Act4Progress,
    event:CommittedEvent,
    nominal_heal:u32,
}

fn route_ids()->[NodeId;4]{[NodeId(0xA400_0051),NodeId(0xA400_0052),NodeId(0xA400_0053),NodeId(0xA400_0054)]}

fn fixed_map()->RunMap{
    let route=route_ids();
    let floors=[51,52,53,54];
    let kinds=[MapNodeKind::Rest,MapNodeKind::Shop,MapNodeKind::Elite,MapNodeKind::Boss];
    let labels=["Rest","Shop","Elite","Boss"];
    let nodes=(0..4).map(|i|MapNode{
        id:route[i],floor:floors[i],label:labels[i],edges:if i<3{vec![route[i+1]]}else{vec![]},
        x:0,y:i as u8,kind:kinds[i],hidden_kind:None,visited:false,revealed:true,
        encounter:None,burning:false,
    }).collect();
    RunMap{boss_icon:Some(EncounterId::CorruptHeart),act:4,draft_ordinal:0,nodes,current:None,
        winged_moves:0,encounter_history:vec![],regular_encounter_history:vec![],elite_encounter_history:vec![],
        question_ramps:[0;3],question_roll_ordinal:0}
}

fn source_is_current(run:&Run,frame:&act3_end::Act3EndFrame)->bool{
    let state=&run.state;
    frame.phase==act3_end::Act3EndPhase::Dialog(4)
        &&frame.accepted.is_none()
        &&frame.record.outcome==Outcome::Victory
        &&matches!(frame.record.encounter,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca)
        &&state.last_combat.as_ref()==Some(&frame.record)
        &&state.active_combat.is_none()&&state.combat.is_none()&&state.pending.is_none()
        &&state.terminal.is_none()&&state.fault.is_none()&&state.hp>0&&state.hp<=state.max_hp
        &&!state.deck.is_empty()&&state.act==3&&state.floor==50&&state.act4_enabled&&state.keys==[true;3]
        &&state.map.as_ref().is_some_and(|map|map.act==3&&map.current==Some(frame.node)
            &&map.boss_icon==Some(frame.record.encounter)
            &&map.nodes.iter().any(|node|node.id==frame.node&&node.floor==50&&node.kind==MapNodeKind::Boss&&node.visited&&node.encounter==Some(frame.record.encounter)))
}

/// Prepare every fallible transition input while the accepted Act3 stage remains untouched.
pub(in crate::run) fn prepare_transition(run:&Run)->Result<PreparedAct4,RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let frame=room.act3_end.as_ref().ok_or(RunError::InvalidAction)?;
    if !is_m10(&run.state)||room.screen.kind!=RoomViewKind::Act3Ending
        ||!room.screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Act3Continue{stage:4}&&row.status==RoomChoiceStatus::Offered)
        ||!source_is_current(run,frame)||run.state.max_hp==0||run.state.max_hp>999{return Err(RunError::InvalidAction)}
    let event=CommittedEvent::ActAdvanced{from:3,to:4};
    reward::preflight(&run.state,std::slice::from_ref(&event))?;
    // Validate the ordinary healing owner's complete input before accepting the
    // door choice. The detached amount is the actual maxHP nominal request.
    let mut heal_probe=run.state.clone();
    mutations::heal(&mut heal_probe,run.state.max_hp)?;
    let route=route_ids();
    let progress=Act4Progress{route,source_node:frame.node,source_record:frame.record.clone(),
        accepted_choice:room.screen.choices.iter().find(|row|row.kind==RoomChoiceKind::Act3Continue{stage:4}).unwrap().id,
        phase:Act4TransitionPhase::ChoiceAccepted,combat_visits:0};
    Ok(PreparedAct4{map:fixed_map(),progress,event,nominal_heal:run.state.max_hp})
}

/// Commit the already accepted door exactly once. Any impossible post-accept
/// failure is retained at its typed phase; this path never reconstructs/retries.
pub(in crate::run) fn commit_transition(run:&mut Run,prepared:PreparedAct4){
    let Some(room)=run.state.room_loop.as_mut() else{return};
    room.act4=Some(prepared.progress.clone());
    run.state.act=4;run.state.room="ActTransition";run.state.map=None;
    room.act1=None;room.act2=None;room.act3=None;room.act3_event=None;room.joust=None;room.joust_visits=0;
    room.active_rewards=None;room.reward_parent=None;room.boss=None;room.act2_boss_order=None;room.act3_boss_order=None;
    if let Some(progress)=room.act4.as_mut(){progress.phase=Act4TransitionPhase::DungeonInitialized;}
    if mutations::heal(&mut run.state,prepared.nominal_heal).is_err(){
        let _=run.retain_fault(RunFaultKind::Invariant("accepted Act4 ordinary healing failed"),"run.act4.transition");return
    }
    if let Some(progress)=run.state.room_loop.as_mut().and_then(|room|room.act4.as_mut()){
        progress.phase=Act4TransitionPhase::Healed;
    }
    run.state.map=Some(prepared.map.clone());run.state.room="Map";
    let room=run.state.room_loop.as_mut().unwrap();room.screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};
    if let Some(progress)=room.act4.as_mut(){progress.phase=Act4TransitionPhase::Published;}
    run.candidates.clear();
    if !matches!(reward::commit(&mut run.state,std::slice::from_ref(&prepared.event)),Ok(0)){
        let _=run.retain_fault(RunFaultKind::Invariant("accepted Act4 transition event publication failed"),"run.act4.transition");return
    }
    run.state.room_loop.as_mut().unwrap().act3_end=None;
}

pub(in crate::run) fn enter(run:&mut Run,target:NodeId)->Result<RunTransition,RunError>{
    let node=world::preflight_move(&run.state,target)?;
    let progress=run.state.room_loop.as_ref().and_then(|room|room.act4.as_ref()).ok_or(RunError::InvalidAction)?;
    if progress.phase!=Act4TransitionPhase::Published||progress.route!=route_ids()||run.state.act!=4
        ||run.state.floor<50||run.state.floor>54||run.state.map.as_ref().is_none_or(|map|map.act!=4||map.boss_icon!=Some(EncounterId::CorruptHeart))
        ||!matches!((node.floor,node.kind),(51,MapNodeKind::Rest)|(52,MapNodeKind::Shop)|(53,MapNodeKind::Elite)|(54,MapNodeKind::Boss))
        ||progress.route[(node.floor-51) as usize]!=target{return Err(RunError::InvalidAction)}
    let encounter=match node.kind{MapNodeKind::Elite=>Some(EncounterId::ShieldAndSpear),MapNodeKind::Boss=>Some(EncounterId::CorruptHeart),_=>None};
    let event=CommittedEvent::FloorEntered{act:4,floor:node.floor};reward::preflight(&run.state,std::slice::from_ref(&event))?;
    let mut next=run.next_choice;let mut rng=run.state.rng.clone();
    let screen=if encounter.is_none(){Some(match node.kind{
        MapNodeKind::Rest=>rest::screen_for_run(run,&mut next)?,
        MapNodeKind::Shop=>shop::screen_with_rng_at(run,&mut next,&mut rng,(4,node.floor,target))?,
        _=>return Err(RunError::InvalidAction),
    })}else{None};
    let map=run.state.map.as_mut().ok_or(RunError::InvalidAction)?;
    map.current=Some(target);let entered=map.nodes.iter_mut().find(|n|n.id==target).ok_or(RunError::InvalidAction)?;
    entered.visited=true;entered.revealed=true;entered.encounter=encounter;
    if let Some(encounter)=encounter{map.encounter_history.push(encounter);if node.kind==MapNodeKind::Elite{map.elite_encounter_history.push(encounter)}}
    run.state.floor=node.floor;run.state.room=node.kind.label();run.state.history.rooms.push(VisitedRoom{act:4,floor:node.floor,label:node.kind.label()});
    run.next_choice=next;run.state.rng=rng;
    if let Some(screen)=screen{run.state.room_loop.as_mut().unwrap().screen=screen;}
    let credit=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    if crate::items::on_hook(&mut run.state,crate::combat::Hook::RoomEntered).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act4 room-entry hook failed"),"run.m10.act4.room"))}
    if let Some(encounter)=encounter{
        match run.enter_owned(encounter,CombatOrigin::MapRoom{node:target,kind:node.kind}){
            Ok(mut transition)=>{transition.reward+=credit;transition.committed_events.insert(0,event);Ok(transition)}
            Err(RunError::Faulted(fault))=>Err(RunError::Faulted(fault)),
            Err(_)=>Err(run.retain_fault(RunFaultKind::Invariant("accepted Act4 encounter entry failed"),"run.m10.act4.encounter")),
        }
    }else{Ok(RunTransition{reward:credit,terminal:None,committed_events:vec![event]})}
}

pub(in crate::run) fn finish_room(run:&mut Run)->Result<RunTransition,RunError>{
    if !is_m10(&run.state)||run.state.act!=4||run.state.combat.is_some()||run.state.active_combat.is_some()
        ||run.state.pending.is_some()||run.state.fault.is_some(){return Err(RunError::InvalidAction)}
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let map=run.state.map.as_ref().ok_or(RunError::InvalidAction)?;
    let current=map.current.ok_or(RunError::InvalidAction)?;
    let node=map.nodes.iter().find(|node|node.id==current&&node.visited).ok_or(RunError::InvalidAction)?;
    if !matches!((node.floor,node.kind,room.screen.kind),(51,MapNodeKind::Rest,RoomViewKind::Campfire)|(52,MapNodeKind::Shop,RoomViewKind::Shop))
        ||run.state.floor!=node.floor||room.reward_parent.is_some(){return Err(RunError::InvalidAction)}
    let room=run.state.room_loop.as_mut().unwrap();room.screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};run.state.room="Map";run.candidates.clear();Ok(empty_transition())
}
