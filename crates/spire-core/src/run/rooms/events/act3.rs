use super::*;
use crate::content::CardType;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Act3EventFrame{
    pub node:NodeId,pub event:EventId,pub visit:u64,pub captures:Vec<(CardType,Option<CardInstance>)>,pub has_mask:bool,pub gold:u32,
    pub accepted:Option<(ChoiceId,RoomChoiceKind)>,pub mutation_complete:bool,pub complete:bool,
}
pub(in crate::run) struct PreparedAct3Event{pub screen:RoomView,pub frame:Act3EventFrame,pub rng:RunRng}
pub(in crate::run) fn prepare(run:&Run,event:EventId,node:NodeId,visit:u64,next:&mut u64)->Result<PreparedAct3Event,RunError>{
    if run.state.act!=3||run.state.ascension!=0||run.state.combat.is_some()||run.state.pending.is_some()||run.state.fault.is_some()||run.state.terminal.is_some()
        ||!run.state.map.as_ref().is_some_and(|m|m.act==3&&m.nodes.iter().any(|n|n.id==node)){return Err(RunError::InvalidAction)}
    let mut rng=run.state.rng.clone();let has_mask=run.state.relics.contains(&RelicId::RedMask);let mut captures=vec![];
    let rows=match event{
        EventId::Falling=>{
            let mut ids=std::collections::BTreeSet::new();if run.state.deck.iter().any(|c|c.instance==0||!ids.insert(c.instance)||crate::content::Catalog::standard().card(c.id).is_none()){return Err(RunError::InvalidAction)}
            for card_type in [CardType::Attack,CardType::Skill,CardType::Power]{
                let mut candidates=run.state.deck.iter().filter(|c|crate::content::Catalog::standard().card(c.id).unwrap().card_type==card_type).collect::<Vec<_>>();candidates.sort_by_key(|c|c.instance);
                let card=if candidates.is_empty(){None}else{Some(candidates[rng.bounded_scoped_checked(RunRngScope::Act3Falling,&[3,node.0,visit,card_type as u64],candidates.len() as u64)?].clone())};captures.push((card_type,card));
            }
            if captures.iter().all(|(_,c)|c.is_none()){vec![(RoomChoiceKind::Leave,None)]}else{[CardType::Skill,CardType::Power,CardType::Attack].into_iter().map(|card_type|(RoomChoiceKind::FallingLose{card_type,instance:captures.iter().find(|(t,_)|*t==card_type).unwrap().1.as_ref().map(|c|c.instance)},None)).collect()}
        },
        EventId::TombOfLordRedMask=>vec![(if has_mask{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold},if has_mask{None}else{Some(run.state.gold)}),(RoomChoiceKind::Leave,None)],
        _=>return Err(RunError::Unsupported("unrepresented Act3 event")),
    };
    let mut screen=view(next,RoomViewKind::Event(event),rows)?;for row in &mut screen.choices{if matches!(row.kind,RoomChoiceKind::FallingLose{instance:None,..}){row.status=RoomChoiceStatus::Disabled;}}
    Ok(PreparedAct3Event{screen,rng,frame:Act3EventFrame{node,event,visit,captures,has_mask,gold:run.state.gold,accepted:None,mutation_complete:false,complete:false}})
}

fn captured(frame:&Act3EventFrame,kind:&RoomChoiceKind)->Option<CardInstance>{
    if let RoomChoiceKind::FallingLose{card_type,instance:Some(instance)}=kind{frame.captures.iter().find(|(t,card)|t==card_type&&card.as_ref().is_some_and(|c|c.instance==*instance)).and_then(|(_,c)|c.clone())}else{None}
}
fn check_source(run:&Run,frame:&Act3EventFrame)->Result<(),RunError>{
    if run.state.act!=3||run.state.combat.is_some()||run.state.pending.is_some()||run.state.map.as_ref().and_then(|m|m.current)!=Some(frame.node)||run.state.room_loop.as_ref().map(|r|r.screen.kind)!=Some(RoomViewKind::Event(frame.event)){return Err(RunError::InvalidAction)}Ok(())
}
pub(in crate::run) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let frame=run.state.room_loop.as_ref().and_then(|r|r.act3_event.as_ref()).cloned().ok_or(RunError::InvalidAction)?;check_source(run,&frame)?;
    let row=public(run)?.choices.into_iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    if row.kind==RoomChoiceKind::Leave&&row.price.is_none(){
        if frame.accepted.is_some()&&!frame.complete{return Err(RunError::InvalidAction)}
        if frame.event==EventId::Falling&&!frame.complete&&frame.captures.iter().any(|(_,c)|c.is_some()){return Err(RunError::InvalidAction)}
        if run.state.deck.is_empty(){reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;}
        run.state.room_loop.as_mut().unwrap().act3_event=None;return finish(run)
    }
    if frame.complete||frame.accepted.is_some(){return Err(RunError::InvalidAction)}
    match (&frame.event,&row.kind){
        (EventId::Falling,RoomChoiceKind::FallingLose{..})=>{
            let card=captured(&frame,&row.kind).ok_or(RunError::InvalidAction)?;if row.price.is_some()||!run.state.deck.contains(&card){return Err(RunError::InvalidAction)}
            if run.state.deck.len()==1{reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;}
        },
        (EventId::TombOfLordRedMask,RoomChoiceKind::TombGold) if frame.has_mask&&row.price.is_none()&&run.state.gold==frame.gold&&run.state.relics.contains(&RelicId::RedMask)=>{run.state.gold.checked_add(222).ok_or(RunError::Unsupported("Tomb gold overflow"))?;},
        (EventId::TombOfLordRedMask,RoomChoiceKind::TombOfferGold) if !frame.has_mask&&row.price==Some(frame.gold)&&run.state.gold==frame.gold&&!run.state.relics.contains(&RelicId::RedMask)=>{
            if run.state.hp>run.state.max_hp||!(1..=999).contains(&run.state.max_hp){return Err(RunError::InvalidAction)}
            if crate::content::Catalog::standard().relic(RelicId::RedMask).is_none(){return Err(RunError::Registry("Red Mask unavailable"))}
        },_=>return Err(RunError::InvalidAction),
    }
    // Reserve result publication before the accepted irreversible mutation.
    let mut next=run.next_choice;view(&mut next,RoomViewKind::Event(frame.event),vec![(RoomChoiceKind::Leave,None)])?;
    run.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap().accepted=Some((id,row.kind));drive(run)
}

pub(in crate::run) fn drive(run:&mut Run)->Result<RunTransition,RunError>{
    if let Some(f)=&run.state.fault{return Err(RunError::Faulted(f.clone()))}
    let frame=run.state.room_loop.as_ref().and_then(|r|r.act3_event.as_ref()).cloned().ok_or(RunError::InvalidAction)?;
    let Some((choice,kind))=&frame.accepted else{return Err(RunError::InvalidAction)};
    if frame.complete||check_source(run,&frame).is_err()||!run.state.room_loop.as_ref().unwrap().screen.choices.iter().any(|r|r.id==*choice&&r.kind==*kind){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 event source changed"),"run.event.act3"))}
    if !frame.mutation_complete{
        let result=match kind{
            RoomChoiceKind::FallingLose{..}=>captured(&frame,kind).filter(|card|run.state.deck.contains(card)).ok_or(RunError::InvalidAction).and_then(|card|mutations::remove_card(&mut run.state,card.instance).map(|_|())),
            RoomChoiceKind::TombGold if frame.has_mask&&run.state.gold==frame.gold&&run.state.relics.contains(&RelicId::RedMask)=>mutations::gain_gold(&mut run.state,222),
            RoomChoiceKind::TombOfferGold if !frame.has_mask&&run.state.gold==frame.gold&&!run.state.relics.contains(&RelicId::RedMask)=>{run.state.gold=0;crate::items::acquire_relic(&mut run.state,RelicId::RedMask)},
            _=>Err(RunError::InvalidAction),
        };
        if result.is_err(){return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 event mutation failed"),"run.event.act3"))}
        run.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap().mutation_complete=true;
    }
    let mutation_valid=match kind{
        RoomChoiceKind::FallingLose{..}=>captured(&frame,kind).is_some_and(|card|run.state.deck.iter().all(|c|c.instance!=card.instance)),
        RoomChoiceKind::TombGold=>frame.gold.checked_add(222)==Some(run.state.gold)&&run.state.relics.contains(&RelicId::RedMask),
        RoomChoiceKind::TombOfferGold=>run.state.gold==0&&run.state.relics.iter().filter(|r|**r==RelicId::RedMask).count()==1,
        _=>false,
    };
    if !mutation_valid{return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 event mutation frontier changed"),"run.event.act3"))}
    if frame.event==EventId::Falling&&run.state.deck.is_empty(){
        return match run.commit_terminal(RunTerminal::EmptyDeck){Ok(t)=>{run.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap().complete=true;Ok(t)},Err(_)=>Err(run.retain_fault(RunFaultKind::Invariant("accepted Falling terminal settlement failed"),"run.event.act3"))}
    }
    let mut next=run.next_choice;let screen=match view(&mut next,RoomViewKind::Event(frame.event),vec![(RoomChoiceKind::Leave,None)]){Ok(s)=>s,Err(_)=>return Err(run.retain_fault(RunFaultKind::Invariant("accepted Act3 event result publication failed"),"run.event.act3"))};
    let room=run.state.room_loop.as_mut().unwrap();room.act3_event.as_mut().unwrap().complete=true;room.screen=screen;run.next_choice=next;run.candidates.clear();Ok(empty_transition())
}
