use super::*;
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let choice=public(run)?.choices.into_iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?;
    match choice.kind {
        RoomChoiceKind::NeowMaxHp{amount}=>mutations::increase_max_hp(&mut run.state,amount)?,
        RoomChoiceKind::NeowLament=>crate::items::acquire_relic(&mut run.state,RelicId::NeowsLament)?,
        _=>return Err(RunError::Unsupported("Lament pending")),
    }
    run.state.selected_neow=run.candidates.iter().find(|c|c.id==id).and_then(|c|c.neow_offer.clone());
    run.state.map=Some(profile::map());run.state.room="Map";run.candidates.clear();
    run.state.room_loop.as_mut().unwrap().screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};Ok(empty_transition())
}
