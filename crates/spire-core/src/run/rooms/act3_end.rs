//! Source-qualified Act3 ending; no combat reward screen or third boss chest. At A20 with every
//! Act 3 boss seen, a second boss comes first (M33 design §6.3).
use super::*;
use crate::{combat::Outcome,content::EncounterId};
use crate::run::world::SECOND_BOSS_NODE;

pub(in crate::run) fn fault_checkpoint(state:&RunState)->&'static str {
    if is_standard(state){"run.m28.act3.end"}else if is_m25(state){"run.m25.act3.end"}else if is_m24(state){"run.m24.act3.end"}else if is_m20(state){"run.m20.act3.end"}else if is_m15(state){"run.m15.act3.end"}else if is_m14(state){"run.m14.act3.end"}else{"run.m9.act3.end"}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Act3EndPhase { AwaitingSource,SecondBoss,Dialog(u8),Ready,Ended }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Act3EndFrame {
    pub node:NodeId,pub record:CombatRecord,pub phase:Act3EndPhase,
    pub accepted:Option<ChoiceId>,
}
pub(in crate::run) struct PreparedAct3End {screen:RoomView,next:u64,frame:Act3EndFrame}

/// The Act 3 boss victory at the map's boss node. The Run's floor is not tested: Secret Portal
/// enters that node from a lower floor (PM check m28f/source-check-events-v1: the ending follows any
/// cleared Beyond `MonsterRoomBoss`; M28f global fix, every map path still beats it at floor 50).
fn source(state:&RunState,node:NodeId,record:&CombatRecord)->bool{
    uses_act3_ending(state)&&state.act==3&&final_boss_source(state,node,record)
}

/// The final Act 3 boss victory's source, shared by the ending and the Act 4 door (M33 design §6.4):
/// the map's boss node (floor 50, the icon's boss) while no second boss exists, else the A20 second
/// boss's node (floor 51, the other boss) after the icon's boss was won at the map's boss node.
pub(in crate::run) fn final_boss_source(state:&RunState,node:NodeId,record:&CombatRecord)->bool{
    if record.outcome!=Outcome::Victory||!matches!(record.encounter,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca){return false}
    let Some(map)=state.map.as_ref().filter(|m|m.act==3&&m.current==Some(node)) else{return false};
    let at=|id:NodeId,floor:u32,boss:EncounterId|map.nodes.iter().any(|n|n.id==id&&n.floor==floor&&n.kind==MapNodeKind::Boss&&n.visited&&n.encounter==Some(boss));
    if !map.nodes.iter().any(|n|n.id==SECOND_BOSS_NODE){return map.boss_icon==Some(record.encounter)&&at(node,50,record.encounter)}
    let Some(first)=map.boss_icon.filter(|first|*first!=record.encounter) else{return false};
    node==SECOND_BOSS_NODE&&at(node,51,record.encounter)
        &&map.nodes.iter().any(|n|n.id!=SECOND_BOSS_NODE&&n.floor==50&&n.kind==MapNodeKind::Boss&&n.visited&&n.encounter==Some(first))
        &&state.history.events.iter().any(|event|matches!(event,CommittedEvent::CombatFinished(r) if r.encounter==first&&r.outcome==Outcome::Victory))
}

/// The A20 double boss is still ahead (source `ProceedButton:129`: Beyond boss room, A ≥ 20, two
/// boss-list entries left). The order has two entries only at A20 with every Act 3 boss seen.
fn double_boss_pending(state:&RunState)->bool{
    is_standard(state)&&state.act==3&&state.ascension>=20
        &&state.room_loop.as_ref().and_then(|room|room.act3_boss_order.as_ref()).is_some_and(|order|order.len()==2)
}
const SECOND_BOSS_CHECKPOINT:&str="run.m33.act3.second_boss";

pub(in crate::run) fn prepare(run:&Run,record:&CombatRecord)->Result<Option<PreparedAct3End>,RunError>{
    if !uses_act3_ending(&run.state)||run.state.act!=3||record.outcome!=Outcome::Victory{return Ok(None)}
    let Some(CombatOrigin::MapRoom{node,kind:MapNodeKind::Boss})=run.state.active_combat.as_ref().map(|a|a.origin)else{return Ok(None)};
    // Every non-Standard scenario is an A0 contract (`rooms/mod.rs` scenario creation): a hand-edited
    // one at A20 is out of contract (M33 design §6.3).
    if run.state.ascension>=20&&!is_standard(&run.state){return Err(RunError::Unsupported("A20 Act 3 double boss is Standard-only"))}
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if !source(&run.state,node,record)||run.state.pending.is_some()||run.state.terminal.is_some()||run.state.fault.is_some()
        ||run.state.deck.is_empty()||room.act3_end.is_some()||room.reward_parent.is_some()||room.active_rewards.is_some()
        ||run.state.combat.as_ref().ok_or(RunError::InvalidAction)?.public_full()?.player.hp<=0{return Err(RunError::InvalidAction)}
    // The first of two A20 bosses: the same screen, whose Proceed enters the second boss
    // (`goToDoubleBoss`); no reward, chest, key or heal in between (lane A §3).
    let phase=if double_boss_pending(&run.state){
        let order=run.state.room_loop.as_ref().and_then(|room|room.act3_boss_order.as_ref()).ok_or(RunError::InvalidAction)?;
        if node==SECOND_BOSS_NODE||record.encounter!=order[0]{return Err(RunError::InvalidAction)}
        Act3EndPhase::SecondBoss
    }else{Act3EndPhase::AwaitingSource};
    let mut next=run.next_choice;
    let screen=view(&mut next,RoomViewKind::Act3BossComplete,vec![(RoomChoiceKind::Proceed,None)])?;
    Ok(Some(PreparedAct3End{screen,next,frame:Act3EndFrame{node,record:record.clone(),phase,accepted:None}}))
}
pub(in crate::run) fn commit(run:&mut Run,prepared:PreparedAct3End){
    run.next_choice=prepared.next;let room=run.state.room_loop.as_mut().unwrap();
    room.screen=prepared.screen;room.act3_end=Some(prepared.frame);run.candidates.clear();
}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let frame=room.act3_end.as_ref().cloned().ok_or(RunError::InvalidAction)?;
    if frame.phase==Act3EndPhase::SecondBoss{
        if frame.accepted.is_some()||room.screen.kind!=RoomViewKind::Act3BossComplete
            ||!room.screen.choices.iter().any(|r|r.id==id&&r.kind==RoomChoiceKind::Proceed&&r.status==RoomChoiceStatus::Offered){return Err(RunError::InvalidAction)}
        let order_head=run.state.room_loop.as_ref().and_then(|room|room.act3_boss_order.as_ref()).and_then(|order|order.first()).copied();
        if !source(&run.state,frame.node,&frame.record)||!double_boss_pending(&run.state)||order_head!=Some(frame.record.encounter)
            ||run.state.last_combat.as_ref()!=Some(&frame.record)||run.state.combat.is_some()||run.state.active_combat.is_some()
            ||run.state.pending.is_some()||run.state.hp==0||run.state.deck.is_empty(){
            run.state.room_loop.as_mut().unwrap().act3_end.as_mut().unwrap().accepted=Some(id);
            return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 double boss source changed"),SECOND_BOSS_CHECKPOINT))
        }
        // Staged: a rejected entry leaves the Run unchanged; the frame ends with the entry.
        return standard::map::enter_second_boss(run)
    }
    if frame.phase==Act3EndPhase::Dialog(4)&&uses_act4_route(&run.state)&&run.state.act4_enabled&&run.state.keys==[true;3]{
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
        Act3EndPhase::Dialog(3)=>(RoomViewKind::Act3Ending,RoomChoiceKind::Act3Continue{stage:3},if uses_act4_route(&run.state){Act3EndPhase::Dialog(4)}else if run.state.act4_enabled&&run.state.keys==[true;3]{Act3EndPhase::Ready}else{Act3EndPhase::Dialog(4)}),
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
