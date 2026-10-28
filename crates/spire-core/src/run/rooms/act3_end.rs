//! Source-qualified A0 Act3 ending; no combat reward screen or third boss chest.
use super::*;
use crate::{combat::Outcome,content::EncounterId};

pub(in crate::run) fn fault_checkpoint(state:&RunState)->&'static str {
    if is_m15(state){"run.m15.act3.end"}else if is_m14(state){"run.m14.act3.end"}else{"run.m9.act3.end"}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Act3EndPhase { AwaitingSource,Dialog(u8),Ready,Ended }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Act3EndFrame {
    pub node:NodeId,pub record:CombatRecord,pub phase:Act3EndPhase,
    pub accepted:Option<ChoiceId>,
}
pub(in crate::run) struct PreparedAct3End {screen:RoomView,next:u64,frame:Act3EndFrame}

fn source(state:&RunState,node:NodeId,record:&CombatRecord)->bool{
    uses_full_act3(state)&&state.act==3&&state.floor==50&&state.ascension==0
        &&record.outcome==Outcome::Victory
        &&matches!(record.encounter,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca)
        &&state.map.as_ref().is_some_and(|m|m.act==3&&m.current==Some(node)
            &&m.boss_icon==Some(record.encounter)&&m.nodes.iter().any(|n|n.id==node&&n.floor==50&&n.kind==MapNodeKind::Boss&&n.visited&&n.encounter==Some(record.encounter)))
}
pub(in crate::run) fn prepare(run:&Run,record:&CombatRecord)->Result<Option<PreparedAct3End>,RunError>{
    if !uses_full_act3(&run.state)||run.state.act!=3||record.outcome!=Outcome::Victory{return Ok(None)}
    let Some(CombatOrigin::MapRoom{node,kind:MapNodeKind::Boss})=run.state.active_combat.as_ref().map(|a|a.origin)else{return Ok(None)};
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if !source(&run.state,node,record)||run.state.pending.is_some()||run.state.terminal.is_some()||run.state.fault.is_some()
        ||run.state.deck.is_empty()||room.act3_end.is_some()||room.reward_parent.is_some()||room.active_rewards.is_some()
        ||run.state.combat.as_ref().ok_or(RunError::InvalidAction)?.public_full()?.player.hp<=0{return Err(RunError::InvalidAction)}
    let mut next=run.next_choice;
    let screen=view(&mut next,RoomViewKind::Act3BossComplete,vec![(RoomChoiceKind::Proceed,None)])?;
    Ok(Some(PreparedAct3End{screen,next,frame:Act3EndFrame{node,record:record.clone(),phase:Act3EndPhase::AwaitingSource,accepted:None}}))
}
pub(in crate::run) fn commit(run:&mut Run,prepared:PreparedAct3End){
    run.next_choice=prepared.next;let room=run.state.room_loop.as_mut().unwrap();
    room.screen=prepared.screen;room.act3_end=Some(prepared.frame);run.candidates.clear();
}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let frame=room.act3_end.as_ref().cloned().ok_or(RunError::InvalidAction)?;
    if frame.phase==Act3EndPhase::Dialog(4)&&uses_act4(&run.state)&&run.state.act4_enabled&&run.state.keys==[true;3]{
        if frame.accepted.is_some()||room.screen.kind!=RoomViewKind::Act3Ending
            ||!room.screen.choices.iter().any(|row|row.id==id&&row.kind==RoomChoiceKind::Act3Continue{stage:4}&&row.status==RoomChoiceStatus::Offered){return Err(RunError::InvalidAction)}
        let prepared=act4::prepare_transition(run)?;
        run.state.room_loop.as_mut().unwrap().act3_end.as_mut().unwrap().accepted=Some(id);
        act4::commit_transition(run,prepared);
        if let Some(fault)=&run.state.fault{return Err(RunError::Faulted(fault.clone()))}
        return Ok(RunTransition{reward:0,terminal:None,committed_events:vec![CommittedEvent::ActAdvanced{from:3,to:4}]})
    }
    let (kind,choice,next_phase)=match frame.phase{
        Act3EndPhase::AwaitingSource=>(RoomViewKind::Act3BossComplete,RoomChoiceKind::Proceed,Act3EndPhase::Dialog(1)),
        Act3EndPhase::Dialog(stage @ 1..=2)=>(RoomViewKind::Act3Ending,RoomChoiceKind::Act3Continue{stage},Act3EndPhase::Dialog(stage+1)),
        Act3EndPhase::Dialog(3)=>(RoomViewKind::Act3Ending,RoomChoiceKind::Act3Continue{stage:3},if uses_act4(&run.state){Act3EndPhase::Dialog(4)}else if run.state.act4_enabled&&run.state.keys==[true;3]{Act3EndPhase::Ready}else{Act3EndPhase::Dialog(4)}),
        Act3EndPhase::Dialog(4)=>(RoomViewKind::Act3Ending,RoomChoiceKind::Act3Continue{stage:4},Act3EndPhase::Ended),
        _=>return Err(RunError::InvalidAction),
    };
    if frame.accepted.is_some()||room.screen.kind!=kind||!room.screen.choices.iter().any(|r|r.id==id&&r.kind==choice&&r.status==RoomChoiceStatus::Offered){return Err(RunError::InvalidAction)}
    // Cursor/ledger exhaustion rejects before the choice is accepted.
    let mut next=run.next_choice;
    let screen=match next_phase{
        Act3EndPhase::Dialog(stage)=>view(&mut next,RoomViewKind::Act3Ending,vec![(RoomChoiceKind::Act3Continue{stage},None)])?,
        Act3EndPhase::Ready=>RoomView{kind:RoomViewKind::Act4Ready,choices:vec![]},
        Act3EndPhase::Ended=>{reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::Act3NoKeys)])?;RoomView{kind:RoomViewKind::Act3Ending,choices:vec![]}},
        _=>return Err(RunError::InvalidAction),
    };
    run.state.room_loop.as_mut().unwrap().act3_end.as_mut().unwrap().accepted=Some(id);
    if !source(&run.state,frame.node,&frame.record)||run.state.last_combat.as_ref()!=Some(&frame.record)
        ||run.state.combat.is_some()||run.state.active_combat.is_some()||run.state.pending.is_some()
        ||run.state.hp==0||run.state.deck.is_empty(){
        return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 ending source changed"),fault_checkpoint(&run.state)))
    }
    let transition=if next_phase==Act3EndPhase::Ended{match run.commit_terminal(RunTerminal::Act3NoKeys){Ok(t)=>t,Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 ending terminal failed"),fault_checkpoint(&run.state)))}}else{empty_transition()};
    let room=run.state.room_loop.as_mut().unwrap();let frame=room.act3_end.as_mut().unwrap();
    frame.phase=next_phase;frame.accepted=None;room.screen=screen;run.next_choice=next;run.candidates.clear();
    Ok(transition)
}
