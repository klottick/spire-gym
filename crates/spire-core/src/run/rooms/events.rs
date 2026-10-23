use super::*;
pub(super) fn screen(next:&mut u64,event:EventId)->Result<RoomView,RunError>{view(next,RoomViewKind::Event(event),vec![(if event==EventId::UpgradeShrine{RoomChoiceKind::ShrineUpgrade}else{RoomChoiceKind::ShrineRemove},None),(RoomChoiceKind::Leave,None)])}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.into_iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?;
    match row.kind{RoomChoiceKind::Leave=>finish(run),RoomChoiceKind::ShrineUpgrade=>selection::open(run,id,RoomSelectionOperation::ShrineUpgrade),RoomChoiceKind::ShrineRemove=>selection::open(run,id,RoomSelectionOperation::ShrineRemove),_=>Err(RunError::InvalidAction)}
}
