use super::*;
fn options(hp:u32)->Vec<RoomChoiceKind>{vec![RoomChoiceKind::Rest{heal:(hp as f32*0.3_f32) as u32},RoomChoiceKind::Smith,RoomChoiceKind::Recall]}
pub(super) fn screen(next:&mut u64,hp:u32)->Result<RoomView,RunError>{view(next,RoomViewKind::Campfire,options(hp).into_iter().map(|kind|(kind,None)).collect())}
pub(super) fn screen_for_run(run:&mut Run,next:&mut u64)->Result<RoomView,RunError>{
    let max_hp=run.state.max_hp;let recall_disabled=uses_act4(&run.state)&&run.state.keys[0];
    let mut options=options(max_hp);
    if run.state.item_admission==ItemAdmission::Complete {
        let mut item_event=crate::items::dispatch::RunItemEvent::CampfireOptions{options};
        // M27 seam: campfire options
        crate::items::dispatch::run_event(&mut run.state,&mut item_event)?;
        let crate::items::dispatch::RunItemEvent::CampfireOptions{options:returned}=item_event else {unreachable!("campfire event kept its kind")};
        options=returned;
    }
    let mut screen=view(next,RoomViewKind::Campfire,options.into_iter().map(|kind|(kind,None)).collect())?;
    if recall_disabled{if let Some(choice)=screen.choices.iter_mut().find(|choice|choice.kind==RoomChoiceKind::Recall){choice.status=RoomChoiceStatus::Disabled;}}
    Ok(screen)
}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let row=public(run)?.choices.into_iter().find(|c|c.id==id).ok_or(RunError::InvalidAction)?;
    match row.kind {
        RoomChoiceKind::Smith=>selection::open(run,id,RoomSelectionOperation::Smith),
        RoomChoiceKind::Rest{heal}=>{
            let mut amount=heal;
            if run.state.item_admission==ItemAdmission::Complete {
                let mut item_event=crate::items::dispatch::RunItemEvent::RestHeal{amount};
                // M27 seam: rest heal
                crate::items::dispatch::run_event(&mut run.state,&mut item_event)?;
                let crate::items::dispatch::RunItemEvent::RestHeal{amount:returned}=item_event else {unreachable!("rest-heal event kept its kind")};
                amount=returned;
            }
            mutations::heal(&mut run.state,amount)?;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::Rested)?;
            if run.state.item_admission==ItemAdmission::Complete {
                // M27 seam: rested
                let mut rested=crate::items::dispatch::RunItemEvent::Rested{card_reward:false};
                crate::items::dispatch::run_event(&mut run.state,&mut rested)?;
                // Dream Catcher: the source opens a combat reward screen with one card reward after resting.
                if let crate::items::dispatch::RunItemEvent::Rested{card_reward:true}=rested {
                    let offer=crate::items::lifecycle::RewardSpec{card_rewards:1,relic_tiers:vec![],potions:vec![],gold:0};
                    return super::rewards::open_equip_offers(run,id,crate::content::RelicId::DreamCatcher,vec![offer],true)
                }
            }
            finish(run)
        },
        RoomChoiceKind::Lift=>{
            if row.status!=RoomChoiceStatus::Offered||run.state.item_admission!=ItemAdmission::Complete{return Err(RunError::InvalidAction)}
            let counter=run.state.relic_counters.entry(crate::content::RelicId::Girya).or_insert(0);
            *counter=counter.saturating_add(1).min(3);
            finish(run)
        },
        RoomChoiceKind::Toke=>{
            if row.status!=RoomChoiceStatus::Offered||run.state.item_admission!=ItemAdmission::Complete{return Err(RunError::InvalidAction)}
            if run.state.deck.is_empty(){return Err(RunError::InvalidAction)}
            selection::open(run,id,RoomSelectionOperation::CampfireToke)
        },
        RoomChoiceKind::Dig=>{
            if row.status!=RoomChoiceStatus::Offered||run.state.item_admission!=ItemAdmission::Complete{return Err(RunError::InvalidAction)}
            let node=run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
            let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,id.0,5];
            let mut staged=run.clone();
            let tier=crate::items::relic_pools::roll_tier(&mut staged.state,crate::items::relic_pools::TierTable::Ordinary,&key);
            let relic=crate::items::relic_pools::take(&mut staged.state,tier,crate::items::relic_pools::Selector::Front,
                crate::items::lifecycle::RelicSource::Dig,&key)?;
            staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen.choices.iter_mut()
                .find(|choice|choice.id==id&&choice.kind==RoomChoiceKind::Dig).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Claimed;
            let transition=rewards::open_dig_relic_offer(&mut staged,id,relic)?;
            *run=staged;
            Ok(transition)
        },
        RoomChoiceKind::Recall=>{if uses_act4(&run.state)&&run.state.keys[0]{return Err(RunError::InvalidAction)}let event=mutations::acquire_key(&mut run.state,Key::Ruby)?;let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;let mut t=finish(run)?;t.reward+=reward;t.committed_events.insert(0,event);Ok(t)},
        _=>Err(RunError::InvalidAction),
    }
}
