use super::*;
pub(in crate::run) fn screen(run:&Run,next:&mut u64)->Result<RoomView,RunError>{let mut rng=run.state.rng.clone();screen_with_rng(run,next,&mut rng)}
pub(in crate::run) fn screen_with_rng(run:&Run,next:&mut u64,rng:&mut RunRng)->Result<RoomView,RunError>{
    let node=if is_m10(&run.state){run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?}else{NodeId(0)};
    screen_with_rng_at(run,next,rng,(run.state.act,run.state.floor,node))
}
pub(in crate::run) fn screen_with_rng_at(run:&Run,next:&mut u64,rng:&mut RunRng,destination:(u8,u32,NodeId))->Result<RoomView,RunError>{
    if is_m10(&run.state){
        let (act,floor,node)=destination;
        let cards=super::m10_content::sample_capture_row(rng,&[u64::from(act),u64::from(floor),node.0],&[crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare])?;
        let mut rows=cards.into_iter().zip([50,75,150]).map(|(id,price)|(RoomChoiceKind::Card{id,upgrades:0},Some(price))).collect::<Vec<_>>();
        rows.extend([(RoomChoiceKind::Relic(RelicId::Brimstone),Some(150)),
            (RoomChoiceKind::Potion(PotionId::BloodPotion),Some(50)),
            (RoomChoiceKind::Potion(PotionId::Elixir),Some(75)),
            (RoomChoiceKind::Potion(PotionId::HeartOfIron),Some(100)),
            (RoomChoiceKind::Purge,Some(run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return view(next,RoomViewKind::Shop,rows)
    }
    if is_m12(&run.state)||is_m13(&run.state){
        let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        let later=if is_m13(&run.state)&&run.state.act==2{room.act2.as_ref().is_some_and(|p|p.shop_visits>0)}else{room.act1.as_ref().is_some_and(|p|p.shop_visits>0)};
        let cards=super::m12_content::shop_cards(later);
        let prices=if later{[50,75,150]}else{[50,75,75]};
        let mut rows=cards.into_iter().zip(prices).map(|(id,price)|(RoomChoiceKind::Card{id,upgrades:0},Some(price))).collect::<Vec<_>>();
        let relic=if later{RelicId::BagOfPreparation}else{RelicId::SneckoSkull};
        if !run.state.relics.contains(&relic){rows.push((RoomChoiceKind::Relic(relic),Some(150)));}
        rows.extend([(RoomChoiceKind::Potion(if later{PotionId::FirePotion}else{PotionId::DexterityPotion}),Some(50)),
            (RoomChoiceKind::Purge,Some(room.purge_cost)),(RoomChoiceKind::Leave,None)]);
        return view(next,RoomViewKind::Shop,rows)
    }
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
