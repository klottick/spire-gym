use super::*;
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let choice=public(run)?.choices.into_iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?;
    let mut obtained=None;
    match choice.kind {
        RoomChoiceKind::NeowMaxHp{amount}=>mutations::increase_max_hp(&mut run.state,amount)?,
        RoomChoiceKind::NeowLament=>{
            let result=crate::items::obtain_relic(&mut run.state,RelicId::NeowsLament,crate::items::lifecycle::RelicSource::Neow)?;
            if run.state.item_admission==ItemAdmission::Complete{obtained=Some(result);}
        },
        _=>return Err(RunError::Unsupported("Lament pending")),
    }
    run.state.selected_neow=run.candidates.iter().find(|c|c.id==id).and_then(|c|c.neow_offer.clone());
    if uses_captured_act1(&run.state){act1::start(run)?;}else{run.state.map=Some(profile::map());}
    run.state.room="Map";run.candidates.clear();
    run.state.room_loop.as_mut().unwrap().screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};
    if let Some(obtained)=obtained{return crate::items::lifecycle::after_obtain(run,id,RelicId::NeowsLament,obtained)}
    Ok(empty_transition())
}
