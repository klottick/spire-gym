use super::*;
pub(in crate::run) struct PreparedRoomRewards {screen:RoomView,rng:RunRng,next:u64}
pub(in crate::run) fn prepare_combat(run:&Run,record:&CombatRecord)->Result<Option<PreparedRoomRewards>,RunError>{
    if !matches!(run.state.active_combat.as_ref().map(|a|a.origin),Some(CombatOrigin::MapRoom(_)))||record.outcome!=crate::combat::Outcome::Victory{return Ok(None)}
    let node=run.state.map.as_ref().and_then(|m|m.current).ok_or(RunError::InvalidAction)?;
    let mut rng=run.state.rng.clone();let gold=10+rng.bounded_scoped(RunRngScope::CombatGoldReward,&[run.state.act as u64,node.0,record.visit],11) as u32;
    let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::Rewards,vec![(RoomChoiceKind::Gold(gold),None),(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Potion(PotionId::FirePotion),None),(RoomChoiceKind::Proceed,None)])?;
    Ok(Some(PreparedRoomRewards{screen,rng,next}))
}
pub(in crate::run) fn commit_combat(run:&mut Run,prepared:PreparedRoomRewards){
    run.state.rng=prepared.rng;run.next_choice=prepared.next;run.state.room_loop.as_mut().unwrap().screen=prepared.screen;
}
fn claimed(run:&mut Run,id:ChoiceId){run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().status=RoomChoiceStatus::Claimed;}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let screen=public(run)?;let row=screen.choices.iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).cloned().ok_or(RunError::InvalidAction)?;
    match row.kind {
        RoomChoiceKind::Gold(n)=>{mutations::gain_gold(&mut run.state,n)?;claimed(run,id);}
        RoomChoiceKind::Potion(potion)=>{let slot=run.state.potions.iter().position(|p|p.is_none()).ok_or(RunError::InvalidAction)?;run.state.potions[slot]=Some(potion);claimed(run,id);}
        RoomChoiceKind::CardReward=>{
            let child=view(&mut run.next_choice,RoomViewKind::CardReward,vec![(RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::ShrugItOff,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::Anger,upgrades:0},None),(RoomChoiceKind::Skip,None)])?;
            let room=run.state.room_loop.as_mut().unwrap();room.reward_parent=Some(room.screen.clone());room.screen=child;
        }
        RoomChoiceKind::Card{id:card,upgrades}=>{
            mutations::obtain_card(&mut run.state,card,upgrades)?;
            return close_card(run,RoomChoiceStatus::Claimed)
        }
        RoomChoiceKind::Skip=>return close_card(run,RoomChoiceStatus::Skipped),
        RoomChoiceKind::Proceed=>return finish(run),
        RoomChoiceKind::OpenChest=>{
            let rows=view(&mut run.next_choice,RoomViewKind::Treasure,vec![(RoomChoiceKind::Relic(RelicId::Strawberry),None),(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)])?;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
            run.state.room_loop.as_mut().unwrap().screen=rows;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
        }
        RoomChoiceKind::Relic(relic)=>{
            crate::items::acquire_relic(&mut run.state,relic)?;claimed(run,id);exclude_base(run);
        }
        RoomChoiceKind::Sapphire=>{
            let event=mutations::acquire_key(&mut run.state,Key::Sapphire)?;
            let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;claimed(run,id);exclude_base(run);
            return Ok(RunTransition{reward,terminal:None,committed_events:vec![event]})
        }
        _=>return Err(RunError::Unsupported("room handler pending")),
    }Ok(empty_transition())
}
fn close_card(run:&mut Run,status:RoomChoiceStatus)->Result<RunTransition,RunError>{
    let room=run.state.room_loop.as_mut().unwrap();let mut parent=room.reward_parent.take().ok_or(RunError::InvalidAction)?;
    parent.choices.iter_mut().find(|c|c.kind==RoomChoiceKind::CardReward).ok_or(RunError::InvalidAction)?.status=status;room.screen=parent;Ok(empty_transition())
}
fn exclude_base(run:&mut Run){
    for row in &mut run.state.room_loop.as_mut().unwrap().screen.choices {if matches!(row.kind,RoomChoiceKind::Relic(_)|RoomChoiceKind::Sapphire)&&row.status==RoomChoiceStatus::Offered{row.status=RoomChoiceStatus::Skipped}}
}
