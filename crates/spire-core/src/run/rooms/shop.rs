use super::*;
pub(in crate::run) fn screen(run:&Run,next:&mut u64)->Result<RoomView,RunError>{
    if uses_captured_act1(&run.state){
        let later=(uses_captured_act2(&run.state)&&run.state.act==2)||(uses_captured_act3(&run.state)&&run.state.act==3)||run.state.room_loop.as_ref().and_then(|r|r.act1.as_ref()).is_some_and(|p|p.shop_visits>0);
        let (card,price,potion,relic)=if later{(CardId::Clothesline,50,PotionId::StrengthPotion,RelicId::Anchor)}else{(CardId::IronWave,25,PotionId::FirePotion,RelicId::BloodVial)};
        let mut rows=vec![(RoomChoiceKind::Card{id:card,upgrades:0},Some(price)),(RoomChoiceKind::Potion(potion),Some(50))];
        if !run.state.relics.contains(&relic)||(uses_captured_act3(&run.state)&&run.state.act==3){rows.push((RoomChoiceKind::Relic(relic),Some(150)));}
        rows.extend([(RoomChoiceKind::Purge,Some(run.state.room_loop.as_ref().unwrap().purge_cost)),(RoomChoiceKind::Leave,None)]);
        return view(next,RoomViewKind::Shop,rows)
    }
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
