use super::*;
pub(super) fn screen(next:&mut u64,hp:u32)->Result<RoomView,RunError>{view(next,RoomViewKind::Campfire,vec![(RoomChoiceKind::Rest{heal:(hp as f32*0.3_f32) as u32},None),(RoomChoiceKind::Smith,None),(RoomChoiceKind::Recall,None)])}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.into_iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?;
    match row.kind {
        RoomChoiceKind::Smith=>selection::open(run,id,RoomSelectionOperation::Smith),
        RoomChoiceKind::Rest{heal}=>{mutations::heal(&mut run.state,heal)?;crate::items::on_hook(&mut run.state,crate::combat::Hook::Rested)?;finish(run)},
        RoomChoiceKind::Recall=>{let event=mutations::acquire_key(&mut run.state,Key::Ruby)?;let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;let mut t=finish(run)?;t.reward+=reward;t.committed_events.insert(0,event);Ok(t)},
        _=>Err(RunError::InvalidAction),
    }
}
