use super::*;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum RewardReturn { Map,BossTreasure }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct CapturedRewards {pub cards:RoomView,pub return_to:RewardReturn}
pub(in crate::run) struct PreparedRoomRewards {screen:RoomView,rng:RunRng,next:u64,captured:Option<CapturedRewards>,elite_cursor:Option<usize>,act:u8,m10:bool}
const ELITE_RELICS:[RelicId;8]=[RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial];
const M10_ELITE_RELICS:[RelicId;19]=[RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,RelicId::BronzeScales,RelicId::RedSkull,RelicId::PaperPhrog,RelicId::ChampionsBelt,RelicId::CharonsAshes,RelicId::SelfFormingClay,RelicId::Orichalcum];
fn capture_with_extra(start:usize,owned:&[RelicId],extra:&[RelicId])->Result<(RelicId,usize),RunError>{
    let (index,relic)=ELITE_RELICS.iter().chain(extra).enumerate().skip(start).find(|(_,id)|!owned.contains(id)).ok_or(RunError::Unsupported("captured elite relic sequence exhausted"))?;
    Ok((*relic,index+1))
}
pub(in crate::run) fn capture_elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{capture_with_extra(start,owned,&[])}
pub(in crate::run) fn capture_m9_elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    capture_with_extra(start,owned,&[RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,RelicId::BronzeScales])
}
pub(in crate::run) fn capture_m10_elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    let (index,relic)=M10_ELITE_RELICS.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id)).ok_or(RunError::Unsupported("captured M10 elite relic sequence exhausted"))?;
    Ok((*relic,index+1))
}
pub(in crate::run) fn capture_m10_chest_relic(start:usize,owned:&[RelicId])->Option<(RelicId,usize)>{
    M10_ELITE_RELICS.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id)).map(|(index,id)|(*id,index+1))
}
pub(in crate::run) fn prepare_combat(run:&Run,record:&CombatRecord)->Result<Option<PreparedRoomRewards>,RunError>{
    if !matches!(run.state.active_combat.as_ref().map(|a|a.origin),Some(CombatOrigin::MapRoom{..}))||record.outcome!=crate::combat::Outcome::Victory{return Ok(None)}
    let node=run.state.map.as_ref().and_then(|m|m.current).ok_or(RunError::InvalidAction)?;
    let m10=is_m10(&run.state);let m5=uses_captured_act1(&run.state);let m6=uses_captured_act2(&run.state)&&run.state.act==2;let m8=uses_captured_act3(&run.state)&&run.state.act==3;let kind=match run.state.active_combat.as_ref().unwrap().origin{CombatOrigin::MapRoom{kind,..}=>kind,_=>unreachable!()};
    let local=if m10&&run.state.act==4{run.state.room_loop.as_ref().and_then(|r|r.act4.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else if m8{run.state.room_loop.as_ref().and_then(|r|r.act3.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else if m6{run.state.room_loop.as_ref().and_then(|r|r.act2.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else{record.visit};
    let act4_local=m10&&run.state.act==4;
    let mut rng=run.state.rng.clone();let mut semantic=vec![run.state.act as u64,node.0,if act4_local{local}else{record.visit}];if m5{semantic.push(kind as u64);}
    let minimum=if m5{match kind{MapNodeKind::Elite=>25,MapNodeKind::Boss=>95,_=>10}}else{10};
    let gold=minimum+rng.bounded_scoped(RunRngScope::CombatGoldReward,&semantic,11) as u32;
    let ordinal=(local%3) as usize;let potion=if is_m13(&run.state)&&run.state.act==2{super::m13_content::combat_potion(local)}else if is_m12(&run.state)||is_m13(&run.state){super::m12_content::combat_potion(record.visit)}else if m10{[PotionId::BloodPotion,PotionId::Elixir,PotionId::HeartOfIron][ordinal]}else if m5{[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][ordinal]}else{PotionId::FirePotion};
    let mut rows=vec![(RoomChoiceKind::Gold(gold),None),(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Potion(potion),None)];let mut elite_cursor=None;
    if m5&&kind==MapNodeKind::Elite {
        let owned=run.state.combat.as_ref().and_then(|c|c.item_context()).map_or(run.state.relics.as_slice(),|c|c.relics());
        let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        let start=if m10{room.m10_elite_relic_cursor}else if m8{room.act3.as_ref().ok_or(RunError::InvalidAction)?.elite_relic_cursor}else if m6{room.act2.as_ref().ok_or(RunError::InvalidAction)?.elite_relic_cursor}else{room.act1.as_ref().ok_or(RunError::InvalidAction)?.elite_relic_cursor};
        let (relic,next)=if m10{capture_m10_elite_relic(start,owned)?}else if is_m9(&run.state)&&run.state.act==3{capture_m9_elite_relic(start,owned)?}else if is_m13(&run.state)&&run.state.act==2{super::m13_content::elite_relic(start,owned)?}else if is_m12(&run.state)||is_m13(&run.state){super::m12_content::elite_relic(start,owned)?}else{capture_elite_relic(start,owned)?};
        rows.push((RoomChoiceKind::Relic(relic),None));elite_cursor=Some(next);
        if !run.state.keys[2]&&run.state.map.as_ref().unwrap().nodes.iter().any(|n|n.id==node&&n.burning){rows.push((RoomChoiceKind::Emerald,None));}
    }
    rows.push((RoomChoiceKind::Proceed,None));
    let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::Rewards,rows)?;
    let captured=if m10 {
        let rarities=if kind==MapNodeKind::Boss{vec![crate::content::CardRarity::Rare;3]}else{vec![crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Uncommon]};
        let cards=super::m10_content::sample_capture_row(&mut rng,&[u64::from(run.state.act),u64::from(run.state.floor),node.0,if act4_local{local}else{record.visit},kind as u64],&rarities)?;
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards{cards:view(&mut next,RoomViewKind::CardReward,rows)?,return_to:if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}})
    }else if is_m13(&run.state)&&run.state.act==2{
        let cards=super::m13_content::reward_cards(kind==MapNodeKind::Boss,ordinal);
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards{cards:view(&mut next,RoomViewKind::CardReward,rows)?,return_to:if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}})
    }else if is_m12(&run.state)||is_m13(&run.state){
        let cards=super::m12_content::reward_cards(kind==MapNodeKind::Boss,ordinal);
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards{cards:view(&mut next,RoomViewKind::CardReward,rows)?,return_to:if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}})
    }else if m5 {
        let triple=if m8&&local==0&&kind==MapNodeKind::Monster{[CardId::Inflame,CardId::ShrugItOff,CardId::Anger]}else if kind==MapNodeKind::Boss{[CardId::Bludgeon,CardId::Impervious,CardId::Offering]}else{[[CardId::IronWave,CardId::ShrugItOff,CardId::Anger],[CardId::Clothesline,CardId::PommelStrike,CardId::Disarm],[CardId::Cleave,CardId::TwinStrike,CardId::Thunderclap]][ordinal]};
        let mut rows=triple.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards{cards:view(&mut next,RoomViewKind::CardReward,rows)?,return_to:if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}})
    }else{None};
    Ok(Some(PreparedRoomRewards{screen,rng,next,captured,elite_cursor,act:run.state.act,m10}))
}
pub(in crate::run) fn commit_combat(run:&mut Run,prepared:PreparedRoomRewards){
    run.state.rng=prepared.rng;run.next_choice=prepared.next;let room=run.state.room_loop.as_mut().unwrap();room.screen=prepared.screen;room.active_rewards=prepared.captured;
    if let Some(cursor)=prepared.elite_cursor{if prepared.m10{room.m10_elite_relic_cursor=cursor}else{match prepared.act{3=>room.act3.as_mut().unwrap().elite_relic_cursor=cursor,2=>room.act2.as_mut().unwrap().elite_relic_cursor=cursor,_=>room.act1.as_mut().unwrap().elite_relic_cursor=cursor}}}
}
fn claimed(run:&mut Run,id:ChoiceId){run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().status=RoomChoiceStatus::Claimed;}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let screen=public(run)?;let row=screen.choices.iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).cloned().ok_or(RunError::InvalidAction)?;
    match row.kind {
        RoomChoiceKind::Gold(n)=>{mutations::gain_gold(&mut run.state,n)?;claimed(run,id);}
        RoomChoiceKind::Potion(potion)=>{let slot=run.state.potions.iter().position(|p|p.is_none()).ok_or(RunError::InvalidAction)?;run.state.potions[slot]=Some(potion);claimed(run,id);}
        RoomChoiceKind::CardReward=>{
            if screen.kind!=RoomViewKind::Rewards||run.state.room_loop.as_ref().unwrap().reward_parent.is_some(){return Err(RunError::InvalidAction)}
            let child=if let Some(captured)=run.state.room_loop.as_ref().unwrap().active_rewards.as_ref(){captured.cards.clone()}else{view(&mut run.next_choice,RoomViewKind::CardReward,vec![(RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::ShrugItOff,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::Anger,upgrades:0},None),(RoomChoiceKind::Skip,None)])?};
            let room=run.state.room_loop.as_mut().unwrap();room.reward_parent=Some(room.screen.clone());room.screen=child;
        }
        RoomChoiceKind::Card{id:card,upgrades}=>{
            card_parent(run)?;
            mutations::obtain_card(&mut run.state,card,upgrades)?;
            return close_card(run,RoomChoiceStatus::Claimed)
        }
        RoomChoiceKind::Skip=>return close_card(run,RoomChoiceStatus::Skipped),
        RoomChoiceKind::Proceed=>{
            if run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().is_some_and(|r|r.return_to==RewardReturn::BossTreasure){return boss::enter_treasure(run)}
            run.state.room_loop.as_mut().unwrap().active_rewards=None;return finish(run)
        },
        RoomChoiceKind::OpenChest=>{
            if screen.kind!=RoomViewKind::Treasure{return Err(RunError::InvalidAction)}
            if is_m10(&run.state){
                let start=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.m10_elite_relic_cursor;
                let relic=capture_m10_chest_relic(start,&run.state.relics);
                let mut rows=Vec::new();if let Some((id,_))=relic{rows.push((RoomChoiceKind::Relic(id),None));}
                rows.extend([(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)]);
                let mut next=run.next_choice;let mut prepared=view(&mut next,RoomViewKind::Treasure,rows)?;
                for row in &mut prepared.choices{if row.kind==RoomChoiceKind::Sapphire&&run.state.keys[1]{row.status=RoomChoiceStatus::Disabled;}}
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
                run.next_choice=next;let room=run.state.room_loop.as_mut().unwrap();room.screen=prepared;
                if let Some((_,cursor))=relic{room.m10_elite_relic_cursor=cursor;}
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
                return Ok(empty_transition())
            }
            if is_m9(&run.state)&&run.state.act==3 {
                let captures=[RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial];
                let available=captures.iter().find(|id|!run.state.relics.contains(id));
                let relics=available.map_or_else(||captures.to_vec(),|id|vec![*id]);
                let mut rows=relics.into_iter().map(|id|(RoomChoiceKind::Relic(id),None)).collect::<Vec<_>>();
                rows.extend([(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)]);
                let mut next=run.next_choice;let mut prepared=view(&mut next,RoomViewKind::Treasure,rows)?;
                for row in &mut prepared.choices{if matches!(row.kind,RoomChoiceKind::Relic(id) if run.state.relics.contains(&id))||row.kind==RoomChoiceKind::Sapphire&&run.state.keys[1]{row.status=RoomChoiceStatus::Disabled;}}
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
                run.next_choice=next;run.state.room_loop.as_mut().unwrap().screen=prepared;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
                return Ok(empty_transition())
            }
            let relic=if uses_captured_act1(&run.state){[RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial].into_iter().find(|id|!run.state.relics.contains(id)).ok_or(RunError::Unsupported("captured ordinary treasure relics exhausted"))?}else{RelicId::Strawberry};
            let rows=view(&mut run.next_choice,RoomViewKind::Treasure,vec![(RoomChoiceKind::Relic(relic),None),(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)])?;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
            run.state.room_loop.as_mut().unwrap().screen=rows;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
        }
        RoomChoiceKind::Relic(relic)=>{
            if run.state.relics.contains(&relic){return Err(RunError::InvalidAction)}
            crate::items::acquire_relic(&mut run.state,relic)?;claimed(run,id);if screen.kind==RoomViewKind::Treasure{exclude_base(run);}
        }
        RoomChoiceKind::Sapphire=>{
            let event=mutations::acquire_key(&mut run.state,Key::Sapphire)?;
            let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;claimed(run,id);exclude_base(run);
            return Ok(RunTransition{reward,terminal:None,committed_events:vec![event]})
        }
        RoomChoiceKind::Emerald=>{
            let event=mutations::acquire_key(&mut run.state,Key::Emerald)?;
            let reward=reward::commit(&mut run.state,std::slice::from_ref(&event))?;claimed(run,id);
            return Ok(RunTransition{reward,terminal:None,committed_events:vec![event]})
        }
        _=>return Err(RunError::Unsupported("room handler pending")),
    }Ok(empty_transition())
}
fn close_card(run:&mut Run,status:RoomChoiceStatus)->Result<RunTransition,RunError>{
    card_parent(run)?;
    let room=run.state.room_loop.as_mut().unwrap();let mut parent=room.reward_parent.take().ok_or(RunError::InvalidAction)?;
    parent.choices.iter_mut().find(|c|c.kind==RoomChoiceKind::CardReward).ok_or(RunError::InvalidAction)?.status=status;room.screen=parent;Ok(empty_transition())
}
fn card_parent(run:&Run)->Result<(),RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let parent=room.reward_parent.as_ref().ok_or(RunError::InvalidAction)?;
    if room.screen.kind!=RoomViewKind::CardReward||parent.kind!=RoomViewKind::Rewards||!parent.choices.iter().any(|c|c.kind==RoomChoiceKind::CardReward&&c.status==RoomChoiceStatus::Offered){return Err(RunError::InvalidAction)}
    if let Some(captured)=&room.active_rewards{if room.screen!=captured.cards{return Err(RunError::InvalidAction)}}
    Ok(())
}
fn exclude_base(run:&mut Run){
    for row in &mut run.state.room_loop.as_mut().unwrap().screen.choices {if matches!(row.kind,RoomChoiceKind::Relic(_)|RoomChoiceKind::Sapphire)&&row.status==RoomChoiceStatus::Offered{row.status=RoomChoiceStatus::Skipped}}
}
