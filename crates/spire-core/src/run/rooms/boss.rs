//! Captured boss chest and source-specific relic children, not ordinary treasure.
use super::*;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct BossProgress {pub selected:Option<RelicId>,pub source_complete:bool,pub skip_parent:Option<RoomView>}

fn source_floors(state:&RunState)->Result<(u32,u32),RunError>{
    match state.act {
        1 if uses_captured_act1(state)=>Ok((16,17)),
        2 if is_m7(state)||is_m8(state)=>Ok((33,34)),
        _=>Err(RunError::InvalidAction),
    }
}

pub(super) fn enter_treasure(run:&mut Run)->Result<RunTransition,RunError>{
    let (boss_floor,treasure_floor)=source_floors(&run.state)?;
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if run.state.floor!=boss_floor||room.boss.is_some()||run.state.pending.is_some()||room.screen.kind!=RoomViewKind::Rewards||!room.active_rewards.as_ref().is_some_and(|r|r.return_to==rewards::RewardReturn::BossTreasure){return Err(RunError::InvalidAction)}
    if !run.state.map.as_ref().is_some_and(|map|map.act==run.state.act&&map.current.is_some_and(|id|map.nodes.iter().any(|n|n.id==id&&n.floor==boss_floor&&n.kind==MapNodeKind::Boss&&n.visited))){return Err(RunError::InvalidAction)}
    let event=CommittedEvent::FloorEntered{act:run.state.act,floor:treasure_floor};reward::preflight(&run.state,std::slice::from_ref(&event))?;
    let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::BossTreasure,vec![(RoomChoiceKind::OpenChest,None),(RoomChoiceKind::Proceed,None)])?;
    run.state.floor=treasure_floor;run.state.room="BossTreasure";run.state.history.rooms.push(VisitedRoom{act:run.state.act,floor:treasure_floor,label:"BossTreasure"});
    run.next_choice=next;let room=run.state.room_loop.as_mut().unwrap();room.screen=screen;room.active_rewards=None;room.reward_parent=None;room.boss=Some(BossProgress{selected:None,source_complete:false,skip_parent:None});
    let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;
    crate::items::on_hook(&mut run.state,crate::combat::Hook::RoomEntered)?;
    Ok(RunTransition{reward,terminal:None,committed_events:vec![event]})
}

fn eligible(state:&RunState)->Vec<CardInstance>{state.deck.iter().filter(|card|crate::content::Catalog::standard().card(card.id).is_some_and(|c|c.note_storage==crate::content::NoteStorage::RemovablePermanent)).cloned().collect()}
fn child(next:&mut u64)->Result<RoomView,RunError>{view(next,RoomViewKind::BossRelicChild,vec![(RoomChoiceKind::Proceed,None)])}

fn warn(run:&mut Run)->Result<RunTransition,RunError>{
    let mut next=run.next_choice;let warning=view(&mut next,RoomViewKind::ProceedWithoutRelic,vec![(RoomChoiceKind::ConfirmSkip,None),(RoomChoiceKind::CancelSkip,None)])?;
    run.next_choice=next;let room=run.state.room_loop.as_mut().unwrap();room.boss.as_mut().unwrap().skip_parent=Some(room.screen.clone());room.screen=warning;Ok(empty_transition())
}

pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let (_,treasure_floor)=source_floors(&run.state)?;
    if run.state.floor!=treasure_floor||run.state.pending.is_some(){return Err(RunError::InvalidAction)}
    let screen=public(run)?;let row=screen.choices.iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered&&c.price.is_none()).cloned().ok_or(RunError::InvalidAction)?;
    match (screen.kind,row.kind.clone()){
        (RoomViewKind::BossTreasure,RoomChoiceKind::OpenChest)=>{
            let mut next=run.next_choice;let mut screen=view(&mut next,RoomViewKind::BossRelicChoice,vec![(RoomChoiceKind::Relic(RelicId::BlackBlood),None),(RoomChoiceKind::Relic(RelicId::EmptyCage),None),(RoomChoiceKind::Relic(RelicId::SlaversCollar),None),(RoomChoiceKind::Skip,None)])?;
            for row in &mut screen.choices{if let RoomChoiceKind::Relic(relic)=row.kind{if run.state.relics.contains(&relic)||(relic==RelicId::BlackBlood&&run.state.relics.first()!=Some(&RelicId::BurningBlood)){row.status=RoomChoiceStatus::Disabled;}}}
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:true})?;
            run.next_choice=next;run.state.room_loop.as_mut().unwrap().screen=screen;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:true})?;
        },
        (RoomViewKind::BossTreasure,RoomChoiceKind::Proceed)|(RoomViewKind::BossRelicChoice,RoomChoiceKind::Skip)=>return warn(run),
        (RoomViewKind::ProceedWithoutRelic,RoomChoiceKind::CancelSkip)=>{
            let room=run.state.room_loop.as_mut().unwrap();room.screen=room.boss.as_mut().ok_or(RunError::InvalidAction)?.skip_parent.take().ok_or(RunError::InvalidAction)?;
        },
        (RoomViewKind::ProceedWithoutRelic,RoomChoiceKind::ConfirmSkip)=>return leave(run,true),
        (RoomViewKind::BossRelicChild,RoomChoiceKind::Proceed)=>return leave(run,false),
        (RoomViewKind::BossRelicChoice,RoomChoiceKind::Relic(relic))=>{
            if run.state.room_loop.as_ref().unwrap().boss.as_ref().unwrap().selected.is_some()||run.state.relics.contains(&relic){return Err(RunError::InvalidAction)}
            if !matches!(relic,RelicId::BlackBlood|RelicId::EmptyCage|RelicId::SlaversCollar){return Err(RunError::InvalidAction)}
            let cards=if relic==RelicId::EmptyCage{eligible(&run.state)}else{vec![]};let grid=relic==RelicId::EmptyCage&&cards.len()>2;
            if !cards.is_empty()&&!grid&&cards.len()==run.state.deck.len(){reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;}
            let mut next=run.next_choice;
            let candidates=if grid{let c=cards.iter().map(|card|Ok(SelectionCandidate{id:SelectionId(token(&mut next)?.0),card:card.clone()})).collect::<Result<Vec<_>,RunError>>()?;next.checked_add(cards.len() as u64).ok_or(RunError::Unsupported("Cage publication identity exhausted"))?;c}else{vec![]};
            let child=if grid{RoomView{kind:RoomViewKind::BossRelicChild,choices:vec![row.clone()]}}else{child(&mut next)?};
            let node=run.state.map.as_ref().and_then(|m|m.current).ok_or(RunError::InvalidAction)?;
            if relic==RelicId::BlackBlood{crate::items::replace_burning_blood(&mut run.state)?;}else{crate::items::acquire_relic(&mut run.state,relic)?;}
            run.next_choice=next;let room=run.state.room_loop.as_mut().unwrap();room.screen=child;let boss=room.boss.as_mut().unwrap();boss.selected=Some(relic);boss.source_complete=!grid;
            if grid {
                if run.begin_selection(ContinuationTarget::Room{node,choice:id,operation:RoomSelectionOperation::EmptyCage},candidates,2,EmptySelectionRule::Fault).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Cage selector preparation failed"),"run.boss.cage"))}
            }else if relic==RelicId::EmptyCage&&!cards.is_empty(){
                for card in &cards{if mutations::remove_card(&mut run.state,card.instance).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Cage removal callback failed"),"run.boss.cage"))}}
                run.state.last_selection=Some(cards);run.state.last_resumed_parent=Some(id);
                if run.state.deck.is_empty(){return run.commit_terminal(RunTerminal::EmptyDeck)}
            }
        },
        _=>return Err(RunError::InvalidAction),
    }Ok(empty_transition())
}

pub(super) fn select_cage(run:&mut Run,id:SelectionId)->Result<RunTransition,RunError>{
    let pending=run.state.pending.as_ref().ok_or(RunError::InvalidAction)?;
    let (node,choice)=match pending.parent{ContinuationTarget::Room{node,choice,operation:RoomSelectionOperation::EmptyCage}=>(node,choice),_=>return Err(RunError::InvalidAction)};
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if pending.resumed||pending.selection.required!=2||pending.selection.selected.len()>=2||room.screen.kind!=RoomViewKind::BossRelicChild||!room.boss.as_ref().is_some_and(|b|b.selected==Some(RelicId::EmptyCage)&&!b.source_complete)||!room.screen.choices.iter().any(|c|c.id==choice&&c.kind==RoomChoiceKind::Relic(RelicId::EmptyCage)&&c.status==RoomChoiceStatus::Offered&&c.price.is_none())||run.state.map.as_ref().and_then(|m|m.current)!=Some(node){return Err(RunError::InvalidAction)}
    let captured=pending.selection.candidates.iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?.card.clone();
    let mut selected=pending.selection.selected.clone();if selected.iter().any(|c|c.instance==captured.instance){return Err(RunError::InvalidAction)}selected.push(captured);
    let valid=eligible(&run.state);if !run.state.relics.contains(&RelicId::EmptyCage)||pending.selection.selected.len()+pending.selection.candidates.len()<2||pending.selection.candidates.iter().any(|c|!valid.contains(&c.card))||selected.iter().any(|c|!valid.contains(c)){return Err(RunError::InvalidAction)}
    let mut next=run.next_choice;
    let screen=if selected.len()==2 {
        if run.state.deck.len()==selected.len(){reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;}
        Some(child(&mut next)?)
    }else {next.checked_add((pending.selection.candidates.len()-1) as u64).ok_or(RunError::Unsupported("Cage pick publication identity exhausted"))?;None};
    let pending=run.state.pending.as_mut().unwrap();pending.selection.candidates.retain(|c|c.id!=id);pending.selection.selected=selected.clone();
    if let Some(screen)=screen {
        for card in &selected{if mutations::remove_card(&mut run.state,card.instance).is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Cage removal callback failed"),"run.boss.cage"))}}
        run.state.last_selection=Some(selected);run.state.last_resumed_parent=Some(choice);run.state.pending=None;run.candidates.clear();run.next_choice=next;
        let room=run.state.room_loop.as_mut().unwrap();room.screen=screen;room.boss.as_mut().unwrap().source_complete=true;
        if run.state.deck.is_empty(){return run.commit_terminal(RunTerminal::EmptyDeck)}
    }else if run.publish_selection_candidates().is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Cage pick publication failed"),"run.boss.cage"))}
    Ok(empty_transition())
}

fn leave(run:&mut Run,skipped:bool)->Result<RunTransition,RunError>{
    let boss=run.state.room_loop.as_ref().and_then(|r|r.boss.as_ref()).ok_or(RunError::InvalidAction)?;
    if !skipped&&(!boss.source_complete||boss.selected.is_none()){return Err(RunError::InvalidAction)}
    if run.state.deck.is_empty(){return run.commit_terminal(RunTerminal::EmptyDeck)}
    transition::begin(run)
}
