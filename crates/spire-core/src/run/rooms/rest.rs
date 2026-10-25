use super::*;
pub(super) fn screen(next:&mut u64,hp:u32)->Result<RoomView,RunError>{view(next,RoomViewKind::Campfire,vec![(RoomChoiceKind::Rest{heal:(hp as f32*0.3_f32) as u32},None),(RoomChoiceKind::Smith,None),(RoomChoiceKind::Recall,None)])}
pub(super) fn screen_for_run(run:&Run,next:&mut u64)->Result<RoomView,RunError>{
    let mut screen=screen(next,run.state.max_hp)?;
    if is_m10(&run.state)&&run.state.keys[0]{screen.choices.iter_mut().find(|choice|choice.kind==RoomChoiceKind::Recall).unwrap().status=RoomChoiceStatus::Disabled;}
    Ok(screen)
}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.into_iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?;
    match row.kind {
        RoomChoiceKind::Smith=>selection::open(run,id,RoomSelectionOperation::Smith),
        RoomChoiceKind::Rest{heal}=>{mutations::heal(&mut run.state,heal)?;crate::items::on_hook(&mut run.state,crate::combat::Hook::Rested)?;finish(run)},
        RoomChoiceKind::Recall=>{if is_m10(&run.state)&&run.state.keys[0]{return Err(RunError::InvalidAction)}let event=mutations::acquire_key(&mut run.state,Key::Ruby)?;let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;let mut t=finish(run)?;t.reward+=reward;t.committed_events.insert(0,event);Ok(t)},
        _=>Err(RunError::InvalidAction),
    }
}
