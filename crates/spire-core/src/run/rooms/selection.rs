use super::*;

pub(in crate::run) fn open(run:&mut Run,choice:ChoiceId,operation:RoomSelectionOperation)->Result<RunTransition,RunError>{
    let node=selection_node(&run.state)?;
    let mut next=run.next_choice;
    let candidates=run.state.deck.iter()
        .filter(|card|match operation{
            RoomSelectionOperation::Smith|RoomSelectionOperation::ShrineUpgrade
            |RoomSelectionOperation::NeowSelect{op:NeowSelectOp::Upgrade,..}=>mutations::can_upgrade(&run.state,card),
            // Every shop purge excludes the unremovable curses and bottled cards (design §6.3;
            // economy research :161; M28a G5 made this shop-wide, not standard-only).
            // G10: Purifier and Ancient Writing's Elegance remove a purgeable card (event-neow research
            // :245-251, :275; acts23 research :180-185), as every shop purge does.
            RoomSelectionOperation::NeowSelect{..}|RoomSelectionOperation::ShopPurge{..}
            |RoomSelectionOperation::ShrineRemove|RoomSelectionOperation::AncientElegance=>purgeable(&run.state,card),
            RoomSelectionOperation::EventSelect{op,..}=>standard::event_exec::select_candidates(&run.state,op).contains(&card),
            _=>true,
        })
        .map(|card|Ok(SelectionCandidate{id:SelectionId(token(&mut next)?.0),card:card.clone()}))
        .collect::<Result<Vec<_>,RunError>>()?;
    next.checked_add(candidates.len() as u64).ok_or(RunError::Unsupported("choice identity exhausted"))?;
    run.next_choice=next;
    let required=match operation{RoomSelectionOperation::NeowSelect{count,..}|RoomSelectionOperation::EventSelect{count,..}=>usize::from(count),_=>1};
    run.begin_selection(ContinuationTarget::Room{node,choice,operation},candidates,required,EmptySelectionRule::Fault)?;
    Ok(empty_transition())
}

/// Removable permanent cards minus bottled ones (design §5.6 "removable"; economy research :161):
/// never Necronomicurse, Curse of the Bell, Ascender's Bane or a card bottled by a relic.
/// Removability is `mutations::removable`, the owner `remove_card` enforces.
pub(in crate::run) fn purgeable(state:&RunState,card:&CardInstance)->bool{
    mutations::removable(card.id)&&!state.relic_cards.values().any(|instance|*instance==card.instance)
}

/// Neow's committed multi-card selector: collect `count` distinct captured cards, then hand them
/// in selection order to `standard::neow::selection_done`. No Cancel (design §5.6).
fn select_neow(run:&mut Run,id:SelectionId)->Result<RunTransition,RunError>{
    let pending=run.state.pending.as_ref().ok_or(RunError::InvalidAction)?.clone();
    let (node,choice,op)=match pending.parent{
        ContinuationTarget::Room{node,choice,operation:RoomSelectionOperation::NeowSelect{op,..}}=>(node,choice,op),
        _=>return Err(RunError::InvalidAction),
    };
    if pending.resumed||selection_node(&run.state).ok()!=Some(node){return Err(RunError::InvalidAction)}
    let captured=pending.selection.candidates.iter().find(|candidate|candidate.id==id).ok_or(RunError::InvalidAction)?.card.clone();
    if !run.state.deck.contains(&captured)||pending.selection.selected.iter().any(|card|card.instance==captured.instance){return Err(RunError::InvalidAction)}
    let mut selected=pending.selection.selected.clone();
    selected.push(captured);
    let mut staged=run.clone();
    if selected.len()<pending.selection.required{
        let selection=staged.state.pending.as_mut().ok_or(RunError::InvalidAction)?;
        selection.selection.candidates.retain(|candidate|candidate.id!=id);
        selection.selection.selected=selected;
        staged.publish_selection_candidates()?;
        *run=staged;
        return Ok(empty_transition())
    }
    staged.state.last_selection=Some(selected.clone());
    staged.state.last_resumed_parent=Some(choice);
    staged.state.pending=None;
    staged.candidates.clear();
    let transition=standard::neow::selection_done(&mut staged,choice,op,selected)?;
    *run=staged;
    Ok(transition)
}

pub(in crate::run) fn select(run:&mut Run,id:SelectionId)->Result<RunTransition,RunError>{
    let pending=run.state.pending.as_ref().ok_or(RunError::InvalidAction)?;
    if matches!(pending.parent,ContinuationTarget::Room{operation:RoomSelectionOperation::EmptyCage,..}){
        return super::boss::select_cage(run,id)
    }
    if matches!(pending.parent,ContinuationTarget::Room{operation:RoomSelectionOperation::RelicEquip{..},..}){
        return select_relic_equip(run,id)
    }
    if matches!(pending.parent,ContinuationTarget::Room{operation:RoomSelectionOperation::NeowSelect{..},..}){
        return select_neow(run,id)
    }
    if matches!(pending.parent,ContinuationTarget::Room{operation:RoomSelectionOperation::EventSelect{..},..}){
        return standard::event_frame::select(run,id)
    }
    let (node,choice,operation)=match pending.parent.clone(){
        ContinuationTarget::Room{node,choice,operation}=>(node,choice,operation),
        _=>return Err(RunError::InvalidAction),
    };
    let captured=pending.selection.candidates.iter().find(|candidate|candidate.id==id).ok_or(RunError::InvalidAction)?.card.clone();
    if selection_node(&run.state).ok()!=Some(node)||!run.state.deck.contains(&captured){return Err(RunError::InvalidAction)}
    let screen=&run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen;
    let row=screen.choices.iter().find(|candidate|candidate.id==choice&&candidate.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    let valid=match &operation {
        RoomSelectionOperation::ShopPurge{price}=>screen.kind==RoomViewKind::Shop&&row.kind==RoomChoiceKind::Purge&&row.price==Some(*price),
        RoomSelectionOperation::Smith=>screen.kind==RoomViewKind::Campfire&&row.kind==RoomChoiceKind::Smith,
        RoomSelectionOperation::ShrineUpgrade=>screen.kind==RoomViewKind::Event(EventId::UpgradeShrine)&&row.kind==RoomChoiceKind::ShrineUpgrade,
        RoomSelectionOperation::ShrineRemove=>screen.kind==RoomViewKind::Event(EventId::Purifier)&&row.kind==RoomChoiceKind::ShrineRemove,
        RoomSelectionOperation::AncientElegance=>screen.kind==RoomViewKind::Event(EventId::AncientWriting)&&row.kind==RoomChoiceKind::AncientElegance&&row.price.is_none(),
        RoomSelectionOperation::CampfireToke=>screen.kind==RoomViewKind::Campfire&&row.kind==RoomChoiceKind::Toke&&row.price.is_none()&&run.state.item_admission==ItemAdmission::Complete,
        RoomSelectionOperation::EmptyCage|RoomSelectionOperation::RelicEquip{..}|RoomSelectionOperation::NeowSelect{..}|RoomSelectionOperation::EventSelect{..}=>false,
    };
    if !valid{return Err(RunError::InvalidAction)}
    if operation==RoomSelectionOperation::CampfireToke&&(pending.selection.required!=1||!pending.selection.selected.is_empty()){
        return Err(RunError::InvalidAction)
    }
    match &operation {
        RoomSelectionOperation::ShopPurge{price}=>{
            let room=run.state.room_loop.as_ref().unwrap();
            // The displayed row price was checked above; a discount (Smiling Mask, Courier,
            // Membership Card) makes it differ from the run-wide base (M28a global fix).
            if run.state.gold<*price||!purgeable(&run.state,&captured){return Err(RunError::InvalidAction)}
            room.purge_cost.checked_add(25).ok_or(RunError::Unsupported("purge price overflow"))?;
        }
        RoomSelectionOperation::Smith|RoomSelectionOperation::ShrineUpgrade=>if !mutations::can_upgrade(&run.state,&captured){return Err(RunError::InvalidAction)},
        _=>(),
    }
    if matches!(&operation,RoomSelectionOperation::ShopPurge{..}|RoomSelectionOperation::ShrineRemove|RoomSelectionOperation::AncientElegance|RoomSelectionOperation::CampfireToke)&&run.state.deck.len()==1{
        reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;
    }
    if operation==RoomSelectionOperation::AncientElegance&&run.state.deck.len()>1{run.next_choice.checked_add(1).ok_or(RunError::Unsupported("choice identity exhausted"))?;}
    match &operation {
        RoomSelectionOperation::ShopPurge{price}=>{
            // Staged so a failed purge spends nothing (M28a G5).
            let mut staged=run.state.clone();
            staged.gold-=*price;
            if staged.item_admission==ItemAdmission::Complete {
                // M27 seam: gold spent in a ShopRoom (economy research §2.3 onSpendGold; Maw Bank).
                crate::items::dispatch::run_event(&mut staged,&mut crate::items::dispatch::RunItemEvent::ShopPurchase)?;
            }
            staged.room_loop.as_mut().unwrap().purge_cost+=25;
            mutations::remove_card(&mut staged,captured.instance)?;
            let next_price=(staged.item_admission==ItemAdmission::Complete)
                .then(||super::shop::next_removal_price(&staged,staged.room_loop.as_ref().unwrap().purge_cost));
            let row=staged.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|candidate|candidate.id==choice).ok_or(RunError::InvalidAction)?;
            row.status=RoomChoiceStatus::Claimed;
            // Complete shops show the source's next removal price on the claimed row (display only).
            if next_price.is_some(){row.price=next_price;}
            run.state=staged;
        }
        RoomSelectionOperation::ShrineRemove|RoomSelectionOperation::AncientElegance|RoomSelectionOperation::CampfireToke=>{
            mutations::remove_card(&mut run.state,captured.instance)?;
            if operation==RoomSelectionOperation::CampfireToke {
                run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|candidate|candidate.id==choice).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Claimed;
            }
        }
        RoomSelectionOperation::Smith|RoomSelectionOperation::ShrineUpgrade=>mutations::upgrade_card(&mut run.state,captured.instance)?,
        RoomSelectionOperation::EmptyCage|RoomSelectionOperation::RelicEquip{..}|RoomSelectionOperation::NeowSelect{..}|RoomSelectionOperation::EventSelect{..}=>return Err(RunError::InvalidAction),
    }
    run.state.last_selection=Some(vec![captured]);
    run.state.last_resumed_parent=Some(choice);
    run.state.pending=None;
    run.candidates.clear();
    if operation==RoomSelectionOperation::AncientElegance{return events::complete_ancient(run)}
    if run.state.deck.is_empty()||!matches!(&operation,RoomSelectionOperation::ShopPurge{..}){finish(run)}else{Ok(empty_transition())}
}

fn select_relic_equip(run:&mut Run,id:SelectionId)->Result<RunTransition,RunError>{
    let pending=run.state.pending.as_ref().ok_or(RunError::InvalidAction)?.clone();
    let (node,choice,relic)=match pending.parent.clone() {
        ContinuationTarget::Room{node,choice,operation:RoomSelectionOperation::RelicEquip{relic}}=>(node,choice,relic),
        _=>return Err(RunError::InvalidAction),
    };
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let boss_source=room.screen.kind==RoomViewKind::BossRelicChild
        &&room.boss.as_ref().is_some_and(|boss|boss.selected==Some(relic)&&!boss.source_complete);
    if run.state.item_admission!=ItemAdmission::Complete||pending.resumed||pending.selection.required==0
        ||pending.selection.selected.len()>=pending.selection.required
        ||selection_node(&run.state).ok()!=Some(node)||!run.state.relics.contains(&relic){return Err(RunError::InvalidAction)}
    if boss_source {
        if !room.screen.choices.iter().any(|row|row.id==choice&&row.kind==RoomChoiceKind::Relic(relic)&&row.status==RoomChoiceStatus::Offered&&row.price.is_none()){
            return Err(RunError::InvalidAction)
        }
    }else if matches!(room.screen.kind,RoomViewKind::CardReward|RoomViewKind::BossRelicChild)||room.reward_parent.is_some(){
        return Err(RunError::InvalidAction)
    }
    let captured=pending.selection.candidates.iter().find(|candidate|candidate.id==id).ok_or(RunError::InvalidAction)?.card.clone();
    if !run.state.deck.contains(&captured)||pending.selection.selected.iter().any(|card|card.instance==captured.instance){return Err(RunError::InvalidAction)}
    let mut seen=std::collections::BTreeSet::new();
    if pending.selection.selected.iter().chain(pending.selection.candidates.iter().map(|candidate|&candidate.card))
        .any(|card|!run.state.deck.contains(card)||!seen.insert(card.instance)){return Err(RunError::InvalidAction)}
    if relic==RelicId::EmptyCage {
        let eligible=super::boss::eligible(&run.state);
        let eligible_ids:std::collections::BTreeSet<_>=eligible.iter().map(|card|card.instance).collect();
        let pending_ids:std::collections::BTreeSet<_>=pending.selection.candidates.iter().map(|candidate|candidate.card.instance)
            .chain(pending.selection.selected.iter().map(|card|card.instance)).collect();
        if pending.selection.required!=2||eligible.len()<=2
            ||pending.selection.candidates.iter().any(|candidate|!eligible.contains(&candidate.card))
            ||pending.selection.selected.iter().any(|card|!eligible.contains(card))||eligible_ids!=pending_ids{return Err(RunError::InvalidAction)}
    }
    let mut selected=pending.selection.selected.clone();
    selected.push(captured);
    let mut staged=run.clone();
    let mut next=staged.next_choice;
    let complete=selected.len()==pending.selection.required;
    let next_screen=if complete&&boss_source{Some(super::boss::child(&mut next)?)}else{None};
    if complete&&staged.state.deck.iter().all(|card|selected.iter().any(|chosen|chosen.instance==card.instance)){
        reward::preflight(&staged.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;
    }
    let selection=staged.state.pending.as_mut().ok_or(RunError::InvalidAction)?;
    selection.selection.candidates.retain(|candidate|candidate.id!=id);
    selection.selection.selected=selected.clone();
    if complete {
        crate::items::dispatch::selection_done(relic,&mut staged.state,&selected.iter().map(|card|card.instance).collect::<Vec<_>>())?;
        staged.state.last_selection=Some(selected);
        staged.state.last_resumed_parent=Some(choice);
        staged.state.pending=None;
        staged.candidates.clear();
        staged.next_choice=next;
        if let Some(screen)=next_screen {
            let room=staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
            room.screen=screen;
            room.boss.as_mut().ok_or(RunError::InvalidAction)?.source_complete=true;
        }
        if staged.state.deck.is_empty(){let transition=staged.commit_terminal(RunTerminal::EmptyDeck)?;*run=staged;return Ok(transition)}
        let transition=rewards::resume_equip_selection(&mut staged,choice,relic)?;
        *run=staged;
        return Ok(transition)
    }else{
        staged.publish_selection_candidates()?;
    }
    *run=staged;
    Ok(empty_transition())
}
