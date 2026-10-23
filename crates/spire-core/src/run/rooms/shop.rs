use super::*;
pub(super) fn screen(run:&Run,next:&mut u64)->Result<RoomView,RunError>{
    view(next,RoomViewKind::Shop,vec![(RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0},Some(25)),(RoomChoiceKind::Potion(PotionId::FirePotion),Some(50)),(RoomChoiceKind::Relic(RelicId::BloodVial),Some(150)),(RoomChoiceKind::Purge,Some(run.state.room_loop.as_ref().unwrap().purge_cost)),(RoomChoiceKind::Leave,None)])
}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.into_iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?;
    if row.kind==RoomChoiceKind::Leave{return finish(run)}
    let price=row.price.ok_or(RunError::InvalidAction)?;
    if row.kind==RoomChoiceKind::Purge{return selection::open(run,id,RoomSelectionOperation::ShopPurge{price})}
    match row.kind{
        RoomChoiceKind::Card{id,upgrades}=>{mutations::obtain_card(&mut run.state,id,upgrades)?;},
        RoomChoiceKind::Potion(potion)=>{let slot=run.state.potions.iter().position(|p|p.is_none()).ok_or(RunError::InvalidAction)?;run.state.potions[slot]=Some(potion);},
        RoomChoiceKind::Relic(relic)=>crate::items::acquire_relic(&mut run.state,relic)?,
        _=>return Err(RunError::InvalidAction),
    }
    run.state.gold-=price;run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().status=RoomChoiceStatus::Claimed;Ok(empty_transition())
}
