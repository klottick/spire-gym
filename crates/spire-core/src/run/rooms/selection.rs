use super::*;
pub(super) fn open(run:&mut Run,choice:ChoiceId,operation:RoomSelectionOperation)->Result<RunTransition,RunError>{
    let node=run.state.map.as_ref().and_then(|m|m.current).ok_or(RunError::InvalidAction)?;
    let mut next=run.next_choice;
    let candidates=run.state.deck.iter().filter(|c|!matches!(operation,RoomSelectionOperation::Smith|RoomSelectionOperation::ShrineUpgrade)||c.upgrades==0&&mutations::supported_in(&run.state,c.id)).map(|c|Ok(SelectionCandidate{id:SelectionId(token(&mut next)?.0),card:c.clone()})).collect::<Result<Vec<_>,RunError>>()?;
    // Reserve the publication tokens before committing a mandatory selector.
    next.checked_add(candidates.len() as u64).ok_or(RunError::Unsupported("choice identity exhausted"))?;
    run.next_choice=next;
    run.begin_selection(ContinuationTarget::Room{node,choice,operation},candidates,1,EmptySelectionRule::Fault)?;Ok(empty_transition())
}
pub(in crate::run) fn select(run:&mut Run,id:SelectionId)->Result<RunTransition,RunError>{
    let pending=run.state.pending.as_ref().ok_or(RunError::InvalidAction)?;
    if matches!(pending.parent,ContinuationTarget::Room{operation:RoomSelectionOperation::EmptyCage,..}){return super::boss::select_cage(run,id)}
    let (node,choice,operation)=match pending.parent{ContinuationTarget::Room{node,choice,operation}=>(node,choice,operation),_=>return Err(RunError::InvalidAction)};
    let captured=pending.selection.candidates.iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?.card.clone();
    if run.state.map.as_ref().and_then(|m|m.current)!=Some(node)||!run.state.deck.contains(&captured){return Err(RunError::InvalidAction)}
    let screen=&run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen;
    let row=screen.choices.iter().find(|c|c.id==choice&&c.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    let valid=match operation {
        RoomSelectionOperation::ShopPurge{price}=>screen.kind==RoomViewKind::Shop&&row.kind==RoomChoiceKind::Purge&&row.price==Some(price),
        RoomSelectionOperation::Smith=>screen.kind==RoomViewKind::Campfire&&row.kind==RoomChoiceKind::Smith,
        RoomSelectionOperation::ShrineUpgrade=>screen.kind==RoomViewKind::Event(EventId::UpgradeShrine)&&row.kind==RoomChoiceKind::ShrineUpgrade,
        RoomSelectionOperation::ShrineRemove=>screen.kind==RoomViewKind::Event(EventId::Purifier)&&row.kind==RoomChoiceKind::ShrineRemove,
        RoomSelectionOperation::AncientElegance=>screen.kind==RoomViewKind::Event(EventId::AncientWriting)&&row.kind==RoomChoiceKind::AncientElegance&&row.price.is_none(),
        RoomSelectionOperation::EmptyCage=>false,
    };if !valid{return Err(RunError::InvalidAction)}
    match operation {
        RoomSelectionOperation::ShopPurge{price}=>{
            let room=run.state.room_loop.as_ref().unwrap();if price!=room.purge_cost||run.state.gold<price{return Err(RunError::InvalidAction)}
            room.purge_cost.checked_add(25).ok_or(RunError::Unsupported("purge price overflow"))?;
        }
        RoomSelectionOperation::Smith|RoomSelectionOperation::ShrineUpgrade=>if captured.upgrades!=0||!mutations::supported_in(&run.state,captured.id){return Err(RunError::InvalidAction)},
        _=>{},
    }
    if matches!(operation,RoomSelectionOperation::ShopPurge{..}|RoomSelectionOperation::ShrineRemove|RoomSelectionOperation::AncientElegance)&&run.state.deck.len()==1{reward::preflight(&run.state,&[CommittedEvent::Ended(RunTerminal::EmptyDeck)])?;}
    if operation==RoomSelectionOperation::AncientElegance&&run.state.deck.len()>1{run.next_choice.checked_add(1).ok_or(RunError::Unsupported("choice identity exhausted"))?;}
    match operation {
        RoomSelectionOperation::ShopPurge{price}=>{
            run.state.gold-=price;run.state.room_loop.as_mut().unwrap().purge_cost+=25;
            mutations::remove_card(&mut run.state,captured.instance)?;
            run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==choice).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Claimed;
        }
        RoomSelectionOperation::ShrineRemove|RoomSelectionOperation::AncientElegance=>{mutations::remove_card(&mut run.state,captured.instance)?;}
        RoomSelectionOperation::Smith|RoomSelectionOperation::ShrineUpgrade=>mutations::upgrade_card(&mut run.state,captured.instance)?,
        RoomSelectionOperation::EmptyCage=>return Err(RunError::InvalidAction),
    }
    run.state.last_selection=Some(vec![captured]);run.state.last_resumed_parent=Some(choice);run.state.pending=None;run.candidates.clear();
    if operation==RoomSelectionOperation::AncientElegance{return events::complete_ancient(run)}
    if run.state.deck.is_empty()||!matches!(operation,RoomSelectionOperation::ShopPurge{..}){finish(run)}else{Ok(empty_transition())}
}
