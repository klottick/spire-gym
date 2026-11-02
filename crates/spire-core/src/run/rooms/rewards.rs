use super::*;
use crate::items::lifecycle::{RelicSource,RewardSpec};
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum RewardReturn {
    Map,
    BossTreasure,
    EquipSelection{source_relic:RelicId,source_choice:ChoiceId,previous:Option<Box<CapturedRewards>>,offers:Vec<RewardSpec>},
    EquipOffers{parent:Box<RoomView>,previous:Option<Box<CapturedRewards>>,queued:Vec<RewardSpec>,finish_parent:bool,source_relic:Option<RelicId>,source_choice:ChoiceId},
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct CapturedRewards {pub cards:RoomView,pub return_to:RewardReturn}
impl CapturedRewards {
    fn new(cards:RoomView,return_to:RewardReturn)->Self{
        Self{cards,return_to}
    }
}
pub(in crate::run) struct PreparedRoomRewards {pub(in crate::run) screen:RoomView,rng:RunRng,next:u64,captured:Option<CapturedRewards>,elite_cursor:Option<usize>,complete_relic_pools:Option<crate::items::relic_pools::RelicPools>,complete_potion_offset:Option<i32>,act:u8,m10:bool}
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
fn next_finite_elite_relic(run:&Run,start:usize,owned:&[RelicId],m10:bool,m15:bool,m20:bool,m6:bool,m8:bool)->Result<(RelicId,usize),RunError>{
    if m20{super::m20_content::normal_relic(start,owned)}
    else if is_m19(&run.state)&&run.state.act==3{super::m19_content::elite_relic(start,owned)}
    else if (is_m18(&run.state)||is_m19(&run.state))&&run.state.act==2{super::m18_content::elite_relic(start,owned)}
    else if uses_defect_content(&run.state){super::m17_content::elite_relic(start,owned)}
    else if m15{super::m15_content::capture_relic(start,owned)}
    else if m10{capture_m10_elite_relic(start,owned)}
    else if is_m14(&run.state)&&run.state.act==3{super::m14_content::elite_relic(start,owned)}
    else if is_m9(&run.state)&&run.state.act==3{capture_m9_elite_relic(start,owned)}
    else if (is_m13(&run.state)||is_m14(&run.state))&&run.state.act==2{super::m13_content::elite_relic(start,owned)}
    else if uses_silent_content(&run.state){super::m12_content::elite_relic(start,owned)}
    else if m6||m8{capture_elite_relic(start,owned)}
    else{capture_elite_relic(start,owned)}
}
fn card_reward_rarities(base:&[crate::content::CardRarity],choices:u32)->Result<Vec<crate::content::CardRarity>,RunError>{
    let count=usize::try_from(choices).map_err(|_|RunError::InvalidAction)?;
    if count>32{return Err(RunError::Unsupported("card reward choice count exceeds the native bound"))}
    let fallback=base.last().copied().unwrap_or(crate::content::CardRarity::Common);
    let mut rarities=base.iter().copied().take(count).collect::<Vec<_>>();rarities.resize(count,fallback);Ok(rarities)
}
fn insert_extra_card_rewards(rows:&mut Vec<(RoomChoiceKind,Option<u32>)>,count:u32){
    let mut insert_at=rows.iter().position(|(kind,_)|*kind==RoomChoiceKind::CardReward).map_or(rows.len(),|index|index+1);
    for _ in 0..count{rows.insert(insert_at,(RoomChoiceKind::CardReward,None));insert_at+=1;}
}
fn sample_shard_card_row(rng:&mut RunRng,semantic:&[u64],rarities:&[crate::content::CardRarity])->Result<Vec<CardId>,RunError>{
    let families=rarities.iter().map(|rarity|crate::content::pools::cards(crate::content::pools::CardPool::AnyColour{rarity:Some(*rarity)})
        .map_err(|_|RunError::Unsupported("Prismatic Shard any-colour reward pool incomplete"))).collect::<Result<Vec<_>,_>>()?;
    let slots=families.iter().map(Vec::as_slice).collect::<Vec<_>>();
    super::m15_content::sample_capture_row_for(rng,semantic,&slots,crate::run::RunRngScope::M26CardOffer,
        "Prismatic Shard any-colour reward pool incomplete","Prismatic Shard any-colour reward pool has too few cards")
}
fn reward_owns_relic(run:&Run,id:RelicId)->bool{
    run.state.combat.as_ref().and_then(|combat|combat.item_context())
        .map_or_else(||run.state.relics.contains(&id),|items|items.owns_relic(id))
}
fn card_child(next:&mut u64,cards:Vec<CardId>,singing_bowl:bool)->Result<RoomView,RunError>{
    let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();
    if singing_bowl{rows.push((RoomChoiceKind::SingingBowl,None));}
    rows.push((RoomChoiceKind::Skip,None));view(next,RoomViewKind::CardReward,rows)
}
fn additional_card_choices(
    run:&Run,rng:&mut RunRng,record:&CombatRecord,kind:MapNodeKind,node:NodeId,local:u64,act4_local:bool,
    card_choices:u32,row:u32,m10:bool,m15:bool,m20:bool,m5:bool,m8:bool,
)->Result<Vec<CardId>,RunError>{
    let boss=kind==MapNodeKind::Boss;
    let row_key=u64::from(row);
    let ordinal=(local%3) as usize;
    let mut cards=if is_m25(&run.state)&&run.state.act==4{
        let rarities=card_reward_rarities(&[
            crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Uncommon,
        ],card_choices)?;
        let mut semantic=vec![4,u64::from(run.state.floor),node.0,record.visit,kind as u64];semantic.push(row_key);
        super::m25_content::sample_capture_row(rng,&semantic,&rarities)?
    }else if uses_watcher_content(&run.state){
        super::m22_content::reward_cards(boss,record.visit.checked_add(row_key).ok_or(RunError::Unsupported("card reward visit exhausted"))?)
    }else if m10||m15||m20{
        let base=if boss{vec![crate::content::CardRarity::Rare;3]}else{vec![crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Uncommon]};
        let rarities=card_reward_rarities(&base,card_choices)?;
        let visit=if act4_local{local}else{record.visit};
        let mut semantic=vec![u64::from(run.state.act),u64::from(run.state.floor),node.0,visit,kind as u64];semantic.push(row_key);
        if m20{super::m20_content::sample_capture_row(rng,&semantic,&rarities)?}
        else if m15{super::m15_content::sample_capture_row(rng,&semantic,&rarities)?}
        else{super::m10_content::sample_capture_row(rng,&semantic,&rarities)?}
    }else if is_m14(&run.state)&&run.state.act==3{
        super::m14_content::reward_cards((ordinal+row as usize)%3)
    }else if (is_m13(&run.state)||is_m14(&run.state))&&run.state.act==2{
        super::m13_content::reward_cards(boss,(ordinal+row as usize)%3)
    }else if uses_defect_content(&run.state){
        let visit=local.checked_add(row_key).ok_or(RunError::Unsupported("card reward visit exhausted"))?;
        if is_m19(&run.state)&&run.state.act==3{super::m19_content::reward_cards(visit)}
        else if (is_m18(&run.state)||is_m19(&run.state))&&run.state.act==2{super::m18_content::reward_cards(boss,visit)}
        else{super::m17_content::reward_cards(boss,visit)}
    }else if uses_silent_content(&run.state){
        super::m12_content::reward_cards(boss,(ordinal+row as usize)%3)
    }else if m5{
        let visit=local.checked_add(row_key).ok_or(RunError::Unsupported("card reward visit exhausted"))?;
        let row_ordinal=(visit%3) as usize;
        let triple=if m8&&visit==0&&kind==MapNodeKind::Monster{[CardId::Inflame,CardId::ShrugItOff,CardId::Anger]}
            else if boss{[CardId::Bludgeon,CardId::Impervious,CardId::Offering]}
            else{[[CardId::IronWave,CardId::ShrugItOff,CardId::Anger],[CardId::Clothesline,CardId::PommelStrike,CardId::Disarm],[CardId::Cleave,CardId::TwinStrike,CardId::Thunderclap]][row_ordinal]};
        triple.to_vec()
    }else{vec![CardId::IronWave,CardId::ShrugItOff,CardId::Anger]};
    if !(is_m25(&run.state)&&run.state.act==4||m10||m15||m20){cards.truncate(card_choices as usize);}
    Ok(cards)
}
fn append_card_children(cards:&mut RoomView,children:impl IntoIterator<Item=RoomView>)->Result<(),RunError>{
    if cards.kind!=RoomViewKind::CardReward{return Err(RunError::InvalidAction)}
    for child in children{
        if child.kind!=RoomViewKind::CardReward||!matches!(child.choices.last().map(|choice|&choice.kind),Some(RoomChoiceKind::Skip)){
            return Err(RunError::InvalidAction)
        }
        cards.choices.extend(child.choices);
    }
    Ok(())
}
fn captured_child_at(cards:&RoomView,index:usize)->Result<RoomView,RunError>{
    if cards.kind!=RoomViewKind::CardReward{return Err(RunError::InvalidAction)}
    let mut start=0;let mut current=0;
    for (end,choice) in cards.choices.iter().enumerate(){
        if choice.kind==RoomChoiceKind::Skip{
            if current==index{return Ok(RoomView{kind:cards.kind,choices:cards.choices[start..=end].to_vec()})}
            current+=1;start=end+1;
        }
    }
    Err(RunError::InvalidAction)
}
fn card_reward_index(parent:&RoomView,id:ChoiceId)->Result<usize,RunError>{
    let mut index=0;
    for choice in &parent.choices{
        if choice.kind==RoomChoiceKind::CardReward{
            if choice.id==id{return Ok(index)}
            index+=1;
        }
    }
    Err(RunError::InvalidAction)
}
fn active_card_reward_index(parent:&RoomView)->Result<usize,RunError>{
    let id=parent.choices.iter().find(|choice|choice.kind==RoomChoiceKind::CardReward&&choice.status==RoomChoiceStatus::Disabled)
        .map(|choice|choice.id).ok_or(RunError::InvalidAction)?;
    card_reward_index(parent,id)
}
pub(crate) fn open_equip_offers(run:&mut Run,choice:ChoiceId,relic:RelicId,offers:Vec<RewardSpec>,finish_parent:bool)->Result<RunTransition,RunError>{
    if run.state.item_admission!=ItemAdmission::Complete||offers.is_empty(){return Err(RunError::InvalidAction)}
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if room.screen.kind==RoomViewKind::CardReward||room.reward_parent.is_some(){return Err(RunError::InvalidAction)}
    let parent=room.screen.clone();
    let previous=room.active_rewards.clone().map(Box::new);
    let mut offers=offers.into_iter();
    let first=offers.next().ok_or(RunError::InvalidAction)?;
    open_equip_offer_group(run,choice,relic,first,parent,previous,offers.collect(),finish_parent)
}

pub(crate) fn defer_equip_offers(run:&mut Run,choice:ChoiceId,relic:RelicId,offers:Vec<RewardSpec>)->Result<(),RunError>{
    if run.state.item_admission!=ItemAdmission::Complete{return Err(RunError::InvalidAction)}
    if offers.is_empty(){return Ok(())}
    let room=run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    if room.active_rewards.as_ref().is_some_and(|captured|matches!(&captured.return_to,RewardReturn::EquipSelection{..})){
        return Err(RunError::InvalidAction)
    }
    let previous=room.active_rewards.take().map(Box::new);
    room.active_rewards=Some(CapturedRewards{
        cards:RoomView{kind:RoomViewKind::CardReward,choices:vec![]},
        return_to:RewardReturn::EquipSelection{source_relic:relic,source_choice:choice,previous,offers},
    });
    Ok(())
}

pub(crate) fn resume_equip_selection(run:&mut Run,choice:ChoiceId,relic:RelicId)->Result<RunTransition,RunError>{
    let captured=run.state.room_loop.as_ref().and_then(|room|room.active_rewards.clone());
    let Some(CapturedRewards{return_to:RewardReturn::EquipSelection{source_relic,source_choice,previous,offers},..})=captured else{
        return Ok(empty_transition())
    };
    if source_relic!=relic||source_choice!=choice{return Err(RunError::InvalidAction)}
    let mut staged=run.clone();
    staged.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.active_rewards=previous.map(|captured|*captured);
    let transition=if offers.is_empty(){empty_transition()}else{open_equip_offers(&mut staged,choice,relic,offers,false)?};
    *run=staged;
    Ok(transition)
}

pub(crate) fn open_dig_relic_offer(run:&mut Run,choice:ChoiceId,relic:RelicId)->Result<RunTransition,RunError>{
    if run.state.item_admission!=ItemAdmission::Complete{return Err(RunError::InvalidAction)}
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    if room.screen.kind!=RoomViewKind::Campfire||room.reward_parent.is_some(){return Err(RunError::InvalidAction)}
    let parent=room.screen.clone();
    let previous=room.active_rewards.clone().map(Box::new);
    let mut staged=run.clone();
    let mut next=staged.next_choice;
    let screen=view(&mut next,RoomViewKind::Rewards,vec![(RoomChoiceKind::Relic(relic),None),(RoomChoiceKind::Proceed,None)])?;
    let cards=view(&mut next,RoomViewKind::CardReward,vec![])?;
    staged.next_choice=next;
    install_offer_frame(&mut staged,choice,None,screen,cards,parent,previous,vec![],true);
    *run=staged;
    Ok(empty_transition())
}

fn open_equip_offer_group(run:&mut Run,choice:ChoiceId,relic:RelicId,spec:RewardSpec,parent:RoomView,
    previous:Option<Box<CapturedRewards>>,queued:Vec<RewardSpec>,finish_parent:bool)->Result<RunTransition,RunError>{
    if run.state.item_admission!=ItemAdmission::Complete{return Err(RunError::InvalidAction)}
    let RewardSpec{card_rewards,relic_tiers,potions,gold}=spec;
    let node=run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
    let mut staged=run.clone();
    let mut next=staged.next_choice;
    let mut rows=Vec::new();
    for _ in 0..card_rewards{rows.push((RoomChoiceKind::CardReward,None));}
    let mut key_index=0_u64;
    for tier in &relic_tiers{
        let key=[u64::from(staged.state.act),u64::from(staged.state.floor),node.0,choice.0,key_index,0xA25];
        let offer=crate::items::relic_pools::take(&mut staged.state,*tier,crate::items::relic_pools::Selector::Front,
            RelicSource::Equip(relic),&key)?;
        rows.push((RoomChoiceKind::Relic(offer),None));
        key_index=key_index.checked_add(1).ok_or(RunError::Unsupported("equip offer index exhausted"))?;
    }
    rows.extend(potions.into_iter().map(|potion|(RoomChoiceKind::Potion(potion),None)));
    if gold>0{rows.push((RoomChoiceKind::Gold(gold),None));}
    rows.push((RoomChoiceKind::Proceed,None));
    let screen=view(&mut next,RoomViewKind::Rewards,rows)?;

    let mut rng=staged.state.rng.clone();
    let rarities=[crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Uncommon];
    let slots=rarities.iter().map(|rarity|super::m15_content::ordinary_family(staged.state.character,*rarity)).collect::<Vec<_>>();
    let shard=staged.state.relics.contains(&RelicId::PrismaticShard);
    let mut children=Vec::with_capacity(card_rewards);
    for index in 0..card_rewards{
        let semantic=[u64::from(staged.state.act),u64::from(staged.state.floor),node.0,choice.0,index as u64,0xA25];
        let cards=if shard{sample_shard_card_row(&mut rng,&semantic,&rarities)?}else{
            super::m15_content::sample_capture_row_for(&mut rng,&semantic,&slots,crate::run::RunRngScope::M26CardOffer,
                "equip card offer has an incomplete character rarity family","equip card offer has no remaining character card")?
        };
        children.push(card_child(&mut next,cards,false)?);
    }
    let mut captured_cards=view(&mut next,RoomViewKind::CardReward,vec![])?;
    append_card_children(&mut captured_cards,children)?;
    staged.next_choice=next;
    staged.state.rng=rng;
    install_offer_frame(&mut staged,choice,Some(relic),screen,captured_cards,parent,previous,queued,finish_parent);
    *run=staged;
    Ok(empty_transition())
}

fn install_offer_frame(run:&mut Run,choice:ChoiceId,source_relic:Option<RelicId>,screen:RoomView,cards:RoomView,
    parent:RoomView,previous:Option<Box<CapturedRewards>>,queued:Vec<RewardSpec>,finish_parent:bool){
    let room=run.state.room_loop.as_mut().expect("offer parent was validated");
    room.screen=screen;
    room.active_rewards=Some(CapturedRewards{cards,return_to:RewardReturn::EquipOffers{
        parent:Box::new(parent),previous,queued,finish_parent,source_relic,source_choice:choice,
    }});
}

fn proceed_equip_offers(run:&mut Run,captured:CapturedRewards)->Result<RunTransition,RunError>{
    let RewardReturn::EquipOffers{parent,previous,mut queued,finish_parent,source_relic,source_choice}=captured.return_to else {
        return Err(RunError::InvalidAction)
    };
    if !queued.is_empty(){
        let next=queued.remove(0);
        let relic=source_relic.ok_or(RunError::InvalidAction)?;
        return open_equip_offer_group(run,source_choice,relic,next,*parent,previous,queued,finish_parent)
    }
    let room=run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.screen=*parent;
    room.active_rewards=previous.map(|captured|*captured);
    if finish_parent{finish(run)}else{Ok(empty_transition())}
}

fn apply_gold_bonus(rows:&mut [(RoomChoiceKind,Option<u32>)],percent:u32)->Result<(),RunError>{
    if percent==0{return Ok(())}
    if let Some((RoomChoiceKind::Gold(base),_))=rows.iter_mut().find(|(kind,_)|matches!(kind,RoomChoiceKind::Gold(_))){
        let rounded=(u64::from(*base)*u64::from(percent)+50)/100;
        let bonus=u32::try_from(rounded).map_err(|_|RunError::Unsupported("gold reward overflow"))?;
        *base=base.checked_add(bonus).ok_or(RunError::Unsupported("gold reward overflow"))?;
    }
    Ok(())
}
pub(super) fn chest_opened(run:&mut Run,boss:bool)->Result<(),RunError>{
    if run.state.item_admission==ItemAdmission::Complete {
        let mut item_event=crate::items::dispatch::RunItemEvent::ChestOpened{boss};
        // M27 seam: chest opened
        crate::items::dispatch::run_event(&mut run.state,&mut item_event)?;
    }
    Ok(())
}
pub(in crate::run) fn prepare_combat(run:&Run,record:&CombatRecord)->Result<Option<PreparedRoomRewards>,RunError>{
    prepare_combat_impl(run,record,None)
}
#[cfg(test)]
pub(in crate::run) fn prepare_combat_with_building(run:&Run,record:&CombatRecord,building:crate::items::dispatch::RunItemEvent)->Result<Option<PreparedRoomRewards>,RunError>{
    prepare_combat_impl(run,record,Some(building))
}
fn prepare_combat_impl(run:&Run,record:&CombatRecord,mut injected_building:Option<crate::items::dispatch::RunItemEvent>)->Result<Option<PreparedRoomRewards>,RunError>{
    if !matches!(run.state.active_combat.as_ref().map(|a|a.origin),Some(CombatOrigin::MapRoom{..}))||!(record.outcome==crate::combat::Outcome::Victory||(is_m15(&run.state)||is_m20(&run.state))&&record.outcome==crate::combat::Outcome::Escaped){return Ok(None)}
    let node=run.state.map.as_ref().and_then(|m|m.current).ok_or(RunError::InvalidAction)?;
    let m20=is_m20(&run.state);let m10=is_m10(&run.state);let m15=is_m15(&run.state);let m5=uses_captured_act1(&run.state);let m6=uses_captured_act2(&run.state)&&run.state.act==2;let m8=uses_captured_act3(&run.state)&&run.state.act==3;let kind=match run.state.active_combat.as_ref().unwrap().origin{CombatOrigin::MapRoom{kind,..}=>kind,_=>unreachable!()};
    let complete_act4_elite=run.state.item_admission==ItemAdmission::Complete&&run.state.relic_pools.is_some()
        &&run.state.act==4&&kind==MapNodeKind::Elite&&record.outcome==crate::combat::Outcome::Victory;
    let local=if (m10||m15||m20)&&run.state.act==4{run.state.room_loop.as_ref().and_then(|r|r.act4.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else if m8{run.state.room_loop.as_ref().and_then(|r|r.act3.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else if m6{run.state.room_loop.as_ref().and_then(|r|r.act2.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else if m20{run.state.room_loop.as_ref().and_then(|r|r.act1.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits}else{record.visit};
    let act4_local=(m10||m15||m20)&&run.state.act==4;
    let mut rng=run.state.rng.clone();let mut semantic=vec![run.state.act as u64,node.0,if act4_local{local}else{record.visit}];if m5{semantic.push(kind as u64);}
    let minimum=if m5{match kind{MapNodeKind::Elite=>25,MapNodeKind::Boss=>95,_=>10}}else{10};
    let all_escaped=run.state.combat.as_ref().is_some_and(|combat|combat.all_enemies_escaped());
    let gold=if all_escaped{None}else{Some(minimum+rng.bounded_scoped(RunRngScope::CombatGoldReward,&semantic,11) as u32)};
    let ordinal=(local%3) as usize;let potion=if uses_watcher_content(&run.state){super::m22_content::combat_potion(record.visit)}else if m20{super::m20_content::POTION_CYCLE[(local%11) as usize]}else if is_m19(&run.state)&&run.state.act==3{super::m19_content::combat_potion(local)}else if uses_defect_content(&run.state){super::m17_content::combat_potion(local)}else if m15{super::m15_content::M15_POTION_CYCLE[(local%6) as usize]}else if is_m14(&run.state)&&run.state.act==3{super::m14_content::combat_potion(local)}else if (is_m13(&run.state)||is_m14(&run.state))&&run.state.act==2{super::m13_content::combat_potion(local)}else if uses_silent_content(&run.state){super::m12_content::combat_potion(record.visit)}else if m10{[PotionId::BloodPotion,PotionId::Elixir,PotionId::HeartOfIron][ordinal]}else if m5{[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][ordinal]}else{PotionId::FirePotion};
    let mut rows=Vec::new();if let Some(gold)=gold{rows.push((RoomChoiceKind::Gold(gold),None));}
    for stolen in run.state.combat.as_ref().map(|combat|combat.stolen_gold_rewards()).unwrap_or_default(){
        rows.push((RoomChoiceKind::Gold(stolen),None));
    }
    rows.push((RoomChoiceKind::CardReward,None));
    if !complete_act4_elite{rows.push((RoomChoiceKind::Potion(potion),None));}
    let mut elite_cursor=None;
    let mut complete_relic_pools=None;
    if complete_act4_elite {
        let mut producer_state=run.state.clone();producer_state.rng=rng.clone();
        let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,record.visit,super::m22_content::ELITE_SOURCE_TAG];
        let tier=crate::items::relic_pools::roll_tier(&mut producer_state,crate::items::relic_pools::TierTable::Ordinary,&key);
        let relic=crate::items::relic_pools::take(&mut producer_state,tier,crate::items::relic_pools::Selector::Front,
            crate::items::lifecycle::RelicSource::Elite,&key)?;
        rng=producer_state.rng.clone();complete_relic_pools=producer_state.relic_pools.clone();
        rows.push((RoomChoiceKind::Relic(relic),None));
    } else if m5&&kind==MapNodeKind::Elite {
        let owned=run.state.combat.as_ref().and_then(|c|c.item_context()).map_or(run.state.relics.as_slice(),|c|c.relics());
        if uses_watcher_content(&run.state){
            let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,record.visit,super::m22_content::ELITE_SOURCE_TAG];
            let pool:&[RelicId]=if is_m25(&run.state)&&run.state.act==4{&super::m25_content::ACT4_ELITE_RELICS}else{&super::m22_content::ELITE_RELICS};
            if let Some(relic)=capture_relic_in_scope(&mut rng,super::m23_content::relic_capture_scope(&run.state),&key,pool,owned)?{rows.push((RoomChoiceKind::Relic(relic),None));}
        }else{
        let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
        let start=if m10||m15||m20{room.m10_elite_relic_cursor}else if m8{room.act3.as_ref().ok_or(RunError::InvalidAction)?.elite_relic_cursor}else if m6{room.act2.as_ref().ok_or(RunError::InvalidAction)?.elite_relic_cursor}else{room.act1.as_ref().ok_or(RunError::InvalidAction)?.elite_relic_cursor};
        let (relic,next)=if m20{super::m20_content::normal_relic(start,owned)?}else if is_m19(&run.state)&&run.state.act==3{super::m19_content::elite_relic(start,owned)?}else if (is_m18(&run.state)||is_m19(&run.state))&&run.state.act==2{super::m18_content::elite_relic(start,owned)?}else if uses_defect_content(&run.state){super::m17_content::elite_relic(start,owned)?}else if m15{super::m15_content::capture_relic(start,owned)?}else if m10{capture_m10_elite_relic(start,owned)?}else if is_m14(&run.state)&&run.state.act==3{super::m14_content::elite_relic(start,owned)?}else if is_m9(&run.state)&&run.state.act==3{capture_m9_elite_relic(start,owned)?}else if (is_m13(&run.state)||is_m14(&run.state))&&run.state.act==2{super::m13_content::elite_relic(start,owned)?}else if uses_silent_content(&run.state){super::m12_content::elite_relic(start,owned)?}else{capture_elite_relic(start,owned)?};
        rows.push((RoomChoiceKind::Relic(relic),None));elite_cursor=Some(next);
        }
        if !run.state.keys[2]&&run.state.map.as_ref().unwrap().nodes.iter().any(|n|n.id==node&&n.burning){rows.push((RoomChoiceKind::Emerald,None));}
    }
    if !complete_act4_elite{rows.push((RoomChoiceKind::Proceed,None));}
    let reward_room=if record.outcome==crate::combat::Outcome::Escaped {
        crate::items::run_laws::RewardRoom::Ordinary{enemy_escaped:true}
    }else{match kind{
        MapNodeKind::Monster=>crate::items::run_laws::RewardRoom::Ordinary{enemy_escaped:false},
        MapNodeKind::Elite=>crate::items::run_laws::RewardRoom::Elite,
        MapNodeKind::Boss=>crate::items::run_laws::RewardRoom::Boss,
        _=>crate::items::run_laws::RewardRoom::Event,
    }};
    let (mut card_choices,mut extra_card_rewards,mut gold_percent,mut extra_elite_relics,mut include_potion,mut singing_bowl)=(3,0,0,0,true,false);
    if run.state.item_admission==ItemAdmission::Complete {
        let mut item_event=injected_building.take().unwrap_or_else(||crate::items::dispatch::RunItemEvent::RewardsBuilding{
            room:reward_room,card_choices,extra_card_rewards,gold_percent,extra_elite_relics,include_potion,singing_bowl,
        });
        let mut event_state=run.state.clone();
        if let Some(items)=run.state.combat.as_ref().and_then(|combat|combat.item_context()) {
            event_state.relics=items.relics().to_vec();
            event_state.relic_counters=items.relics().iter().filter_map(|id|{
                let counter=items.counter(*id);(counter!=0).then_some((*id,counter))
            }).collect();
        }
        // M27 seam: rewards building
        crate::items::dispatch::run_event(&mut event_state,&mut item_event)?;
        let crate::items::dispatch::RunItemEvent::RewardsBuilding{
            card_choices:choices,extra_card_rewards:extra,gold_percent:percent,extra_elite_relics:elite_extra,include_potion:potion,singing_bowl:bowl,..}=item_event else {unreachable!("reward-building event kept its kind")};
        (card_choices,extra_card_rewards,gold_percent,extra_elite_relics,include_potion,singing_bowl)=(choices,extra,percent,elite_extra,potion,bowl);
    }
    if !complete_act4_elite&&!include_potion{rows.retain(|(kind,_)|!matches!(kind,RoomChoiceKind::Potion(_)));}
    if !complete_act4_elite&&run.state.item_admission==ItemAdmission::Complete&&extra_card_rewards>0 {
        insert_extra_card_rewards(&mut rows,extra_card_rewards);
    }
    apply_gold_bonus(&mut rows,gold_percent)?;
    if run.state.item_admission==ItemAdmission::Complete&&kind==MapNodeKind::Elite&&record.outcome==crate::combat::Outcome::Victory&&extra_elite_relics>0 {
        if extra_elite_relics>4{return Err(RunError::Unsupported("extra elite relic count exceeds the native bound"))}
        if run.state.item_admission==ItemAdmission::Complete&&run.state.relic_pools.is_some()&&run.state.act==4 {
            let mut producer_state=run.state.clone();
            if let Some(pools)=complete_relic_pools.take(){producer_state.relic_pools=Some(pools);}
            producer_state.rng=rng.clone();
            for extra in 0..extra_elite_relics {
                let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,record.visit,
                    super::m22_content::ELITE_SOURCE_TAG,1+u64::from(extra)];
                let tier=crate::items::relic_pools::roll_tier(&mut producer_state,crate::items::relic_pools::TierTable::Ordinary,&key);
                let relic=crate::items::relic_pools::exclude_take(&mut producer_state,tier,crate::items::relic_pools::Selector::Front,
                    crate::items::lifecycle::RelicSource::Elite,&[RelicId::Girya,RelicId::PeacePipe,RelicId::Shovel],&key)?;
                rows.push((RoomChoiceKind::Relic(relic),None));
            }
            rng=producer_state.rng.clone();complete_relic_pools=producer_state.relic_pools.clone();
        } else if uses_watcher_content(&run.state) {
            let pool:&[RelicId]=if is_m25(&run.state)&&run.state.act==4{&super::m25_content::ACT4_ELITE_RELICS}else{&super::m22_content::ELITE_RELICS};
            let mut owned=run.state.combat.as_ref().and_then(|combat|combat.item_context()).map_or_else(||run.state.relics.clone(),|items|items.relics().to_vec());
            owned.extend(rows.iter().filter_map(|(choice,_)|if let RoomChoiceKind::Relic(id)=choice{Some(*id)}else{None}));
            let scope=super::m23_content::relic_capture_scope(&run.state);
            for extra in 0..extra_elite_relics {
                let mut key=vec![u64::from(run.state.act),u64::from(run.state.floor),node.0,record.visit,super::m22_content::ELITE_SOURCE_TAG,1+u64::from(extra)];
                if is_m25(&run.state)&&run.state.act==4{key[4]=super::m22_content::ELITE_SOURCE_TAG;}
                let relic=capture_relic_in_scope(&mut rng,scope,&key,pool,&owned)?
                    .ok_or(RunError::Unsupported("extra elite relic pool exhausted"))?;
                rows.push((RoomChoiceKind::Relic(relic),None));owned.push(relic);
            }
        } else {
            let mut start=elite_cursor.ok_or(RunError::Unsupported("extra elite relic requires a finite producer cursor"))?;
            let mut owned=run.state.combat.as_ref().and_then(|combat|combat.item_context()).map_or_else(||run.state.relics.clone(),|items|items.relics().to_vec());
            owned.extend(rows.iter().filter_map(|(choice,_)|if let RoomChoiceKind::Relic(id)=choice{Some(*id)}else{None}));
            for _ in 0..extra_elite_relics {
                let (relic,next)=next_finite_elite_relic(run,start,&owned,m10,m15,m20,m6,m8)?;
                rows.push((RoomChoiceKind::Relic(relic),None));owned.push(relic);start=next;
            }
            elite_cursor=Some(start);
        }
    }
    let mut complete_potion_offset=None;
    if complete_act4_elite {
        if include_potion {
            let mut producer_state=run.state.clone();producer_state.rng=rng.clone();
            if let Some(items)=run.state.combat.as_ref().and_then(|combat|combat.item_context()){
                producer_state.relics.clear();
                if items.owns_relic(RelicId::WhiteBeastStatue){producer_state.relics.push(RelicId::WhiteBeastStatue);}
            }
            let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,record.visit,super::m22_content::ELITE_SOURCE_TAG];
            // The four-row check precedes the auto-added card reward (economy research §5.2).
            let existing_rows=rows.iter().filter(|(kind,_)|*kind!=RoomChoiceKind::CardReward).count();
            let potion=crate::items::potion_offers::combat_appearance(&mut producer_state,crate::items::potion_offers::RewardRoom::Elite,existing_rows,&key)?;
            if let Some(potion)=potion{rows.push((RoomChoiceKind::Potion(potion),None));}
            rng=producer_state.rng.clone();complete_potion_offset=Some(producer_state.potion_offset);
        }
        if extra_card_rewards>0{insert_extra_card_rewards(&mut rows,extra_card_rewards);}
        rows.push((RoomChoiceKind::Proceed,None));
    }
    // Smoke still captures internal room drops, including the finite elite
    // stock, but its reward screen bypasses setup and offers only Proceed.
    if record.outcome==crate::combat::Outcome::Escaped{
        let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::Rewards,vec![(RoomChoiceKind::Proceed,None)])?;
        return Ok(Some(PreparedRoomRewards{screen,rng,next,captured:None,elite_cursor,complete_relic_pools,complete_potion_offset,act:run.state.act,m10:m10||m15||m20}))
    }
    let mut next=run.next_choice;let screen=view(&mut next,RoomViewKind::Rewards,rows)?;
    let captured=if is_m25(&run.state)&&run.state.act==4{
        let base_rarities=[crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Uncommon];
        let rarities=if run.state.item_admission==ItemAdmission::Complete{card_reward_rarities(&base_rarities,card_choices)?}else{base_rarities.to_vec()};
        let semantic=[4,u64::from(run.state.floor),node.0,record.visit,kind as u64];
        let cards=if complete_act4_elite&&reward_owns_relic(run,RelicId::PrismaticShard){sample_shard_card_row(&mut rng,&semantic,&rarities)?}
            else{super::m25_content::sample_capture_row(&mut rng,&semantic,&rarities)?};
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,RewardReturn::Map))
    }else if uses_watcher_content(&run.state){
        let cards=super::m22_content::reward_cards(kind==MapNodeKind::Boss,record.visit);
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}))
    }else if m10||m15||m20 {
        let base_rarities=if kind==MapNodeKind::Boss{vec![crate::content::CardRarity::Rare;3]}else{vec![crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Uncommon]};
        let rarities=if run.state.item_admission==ItemAdmission::Complete{card_reward_rarities(&base_rarities,card_choices)?}else{base_rarities};
        let semantic=[u64::from(run.state.act),u64::from(run.state.floor),node.0,if act4_local{local}else{record.visit},kind as u64];
        let cards=if complete_act4_elite&&reward_owns_relic(run,RelicId::PrismaticShard){sample_shard_card_row(&mut rng,&semantic,&rarities)?}
            else if m20{super::m20_content::sample_capture_row(&mut rng,&semantic,&rarities)?}else if m15{super::m15_content::sample_capture_row(&mut rng,&semantic,&rarities)?}else{super::m10_content::sample_capture_row(&mut rng,&semantic,&rarities)?};
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}))
    }else if is_m14(&run.state)&&run.state.act==3{
        let cards=super::m14_content::reward_cards(ordinal);
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,RewardReturn::Map))
    }else if (is_m13(&run.state)||is_m14(&run.state))&&run.state.act==2{
        let cards=super::m13_content::reward_cards(kind==MapNodeKind::Boss,ordinal);
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}))
    }else if uses_defect_content(&run.state){
        // M17 consumes the full settled visit; legacy rows retain modulo3.
        let cards=if is_m19(&run.state)&&run.state.act==3{super::m19_content::reward_cards(local)}else if (is_m18(&run.state)||is_m19(&run.state))&&run.state.act==2{super::m18_content::reward_cards(kind==MapNodeKind::Boss,local)}else{super::m17_content::reward_cards(kind==MapNodeKind::Boss,local)};
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}))
    }else if uses_silent_content(&run.state){
        let cards=super::m12_content::reward_cards(kind==MapNodeKind::Boss,ordinal);
        let mut rows=cards.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}))
    }else if m5 {
        let triple=if m8&&local==0&&kind==MapNodeKind::Monster{[CardId::Inflame,CardId::ShrugItOff,CardId::Anger]}else if kind==MapNodeKind::Boss{[CardId::Bludgeon,CardId::Impervious,CardId::Offering]}else{[[CardId::IronWave,CardId::ShrugItOff,CardId::Anger],[CardId::Clothesline,CardId::PommelStrike,CardId::Disarm],[CardId::Cleave,CardId::TwinStrike,CardId::Thunderclap]][ordinal]};
        let mut rows=triple.into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();rows.push((RoomChoiceKind::Skip,None));
        Some(CapturedRewards::new(view(&mut next,RoomViewKind::CardReward,rows)?,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}))
    }else{None};
    let mut captured=captured;
    if run.state.item_admission==ItemAdmission::Complete {
        if extra_card_rewards>8{return Err(RunError::Unsupported("extra card reward count exceeds the native bound"))}
        if let Some(rewards)=captured.as_mut() {
            if card_choices!=3||singing_bowl {
                let cards=rewards.cards.choices.iter().filter_map(|choice|if let RoomChoiceKind::Card{id,..}=choice.kind{Some(id)}else{None}).collect::<Vec<_>>();
                let cards=if card_choices==3{cards}else{cards.into_iter().take(card_choices as usize).collect()};
                rewards.cards=card_child(&mut next,cards,singing_bowl)?;
            }
        }
        if extra_card_rewards>0 {
            if captured.is_none(){
                let cards=card_child(&mut next,vec![CardId::IronWave,CardId::ShrugItOff,CardId::Anger],singing_bowl)?;
                captured=Some(CapturedRewards::new(cards,if kind==MapNodeKind::Boss{RewardReturn::BossTreasure}else{RewardReturn::Map}));
            }
            let mut children=Vec::with_capacity(extra_card_rewards as usize);
            for row in 1..=extra_card_rewards {
                let cards=additional_card_choices(run,&mut rng,record,kind,node,local,act4_local,card_choices,row,m10,m15,m20,m5,m8)?;
                children.push(card_child(&mut next,cards,singing_bowl)?);
            }
            append_card_children(&mut captured.as_mut().ok_or(RunError::InvalidAction)?.cards,children)?;
        }
    }
    Ok(Some(PreparedRoomRewards{screen,rng,next,captured,elite_cursor,complete_relic_pools,complete_potion_offset,act:run.state.act,m10:m10||m15||m20}))
}
pub(in crate::run) fn commit_combat(run:&mut Run,prepared:PreparedRoomRewards){
    run.state.rng=prepared.rng;run.next_choice=prepared.next;
    if let Some(pools)=prepared.complete_relic_pools{run.state.relic_pools=Some(pools);}
    if let Some(offset)=prepared.complete_potion_offset{run.state.potion_offset=offset;}
    let room=run.state.room_loop.as_mut().unwrap();room.screen=prepared.screen;room.active_rewards=prepared.captured;
    if let Some(cursor)=prepared.elite_cursor{if prepared.m10{room.m10_elite_relic_cursor=cursor}else{match prepared.act{3=>room.act3.as_mut().unwrap().elite_relic_cursor=cursor,2=>room.act2.as_mut().unwrap().elite_relic_cursor=cursor,_=>room.act1.as_mut().unwrap().elite_relic_cursor=cursor}}}
}
fn claimed(run:&mut Run,id:ChoiceId){run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().status=RoomChoiceStatus::Claimed;}
pub(super) fn choose(run:&mut Run,id:ChoiceId)->Result<RunTransition,RunError>{
    let screen=public(run)?;let row=screen.choices.iter().find(|c|c.id==id&&c.status==RoomChoiceStatus::Offered).cloned().ok_or(RunError::InvalidAction)?;
    match row.kind {
        RoomChoiceKind::Gold(n)=>{mutations::gain_gold(&mut run.state,n)?;claimed(run,id);}
        RoomChoiceKind::Potion(potion)=>{
            if run.state.item_admission==ItemAdmission::Complete{
                let mut staged=run.clone();
                if !staged.state.relics.contains(&RelicId::Sozu)&&!crate::items::obtain_run_potion(&mut staged.state,potion)?{
                    return Err(RunError::InvalidAction)
                }
                claimed(&mut staged,id);*run=staged;
            }else{
                let slot=run.state.potions.iter().position(|p|p.is_none()).ok_or(RunError::InvalidAction)?;
                run.state.potions[slot]=Some(potion);claimed(run,id);
            }
        }
        RoomChoiceKind::CardReward=>{
            if screen.kind!=RoomViewKind::Rewards||run.state.room_loop.as_ref().unwrap().reward_parent.is_some(){return Err(RunError::InvalidAction)}
            let complete=run.state.item_admission==ItemAdmission::Complete;
            let child=if let Some(captured)=run.state.room_loop.as_ref().unwrap().active_rewards.as_ref(){
                if complete{captured_child_at(&captured.cards,card_reward_index(&screen,id)?)?}else{captured.cards.clone()}
            }else{view(&mut run.next_choice,RoomViewKind::CardReward,vec![(RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::ShrugItOff,upgrades:0},None),(RoomChoiceKind::Card{id:CardId::Anger,upgrades:0},None),(RoomChoiceKind::Skip,None)])?};
            let room=run.state.room_loop.as_mut().unwrap();
            if complete {room.screen.choices.iter_mut().find(|choice|choice.id==id).ok_or(RunError::InvalidAction)?.status=RoomChoiceStatus::Disabled;}
            room.reward_parent=Some(room.screen.clone());room.screen=child;
        }
        RoomChoiceKind::Card{id:card,upgrades}=>{
            card_parent(run)?;
            mutations::obtain_card(&mut run.state,card,upgrades)?;
            return close_card(run,RoomChoiceStatus::Claimed)
        }
        RoomChoiceKind::SingingBowl=>{
            card_parent(run)?;
            mutations::increase_max_hp(&mut run.state,2)?;
            return close_card(run,RoomChoiceStatus::Claimed)
        }
        RoomChoiceKind::Skip=>return close_card(run,RoomChoiceStatus::Skipped),
        RoomChoiceKind::Proceed=>{
            if let Some(captured)=run.state.room_loop.as_ref().unwrap().active_rewards.clone(){
                if captured.return_to==RewardReturn::BossTreasure{return boss::enter_treasure(run)}
                if matches!(captured.return_to,RewardReturn::EquipOffers{..}){return proceed_equip_offers(run,captured)}
            }
            run.state.room_loop.as_mut().unwrap().active_rewards=None;return finish(run)
        },
        RoomChoiceKind::OpenChest=>{
            if screen.kind!=RoomViewKind::Treasure{return Err(RunError::InvalidAction)}
            if uses_watcher_content(&run.state){
                let node=run.state.map.as_ref().and_then(|map|map.current).ok_or(RunError::InvalidAction)?;
                let key=[u64::from(run.state.act),u64::from(run.state.floor),node.0,run.state.combat_progress.entered,super::m22_content::CHEST_SOURCE_TAG];
                let mut rng=run.state.rng.clone();
                let relic=capture_relic_in_scope(&mut rng,super::m23_content::relic_capture_scope(&run.state),&key,&super::m22_content::CHEST_RELICS,&run.state.relics)?;
                let mut rows=Vec::new();if let Some(id)=relic{rows.push((RoomChoiceKind::Relic(id),None));}
                rows.extend([(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)]);
                let mut next=run.next_choice;let mut prepared=view(&mut next,RoomViewKind::Treasure,rows)?;
                for row in &mut prepared.choices{if row.kind==RoomChoiceKind::Sapphire&&run.state.keys[1]{row.status=RoomChoiceStatus::Disabled;}}
                run.next_choice=next;run.state.rng=rng;run.state.room_loop.as_mut().unwrap().screen=prepared;
                chest_opened(run,false)?;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
                return Ok(empty_transition())
            }
            if uses_act4(&run.state){
                let start=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.m10_elite_relic_cursor;
                let relic=if is_m20(&run.state){Some(super::m20_content::normal_relic(start,&run.state.relics)?)}else if is_m15(&run.state){Some(super::m15_content::capture_relic(start,&run.state.relics)?)}else{capture_m10_chest_relic(start,&run.state.relics)};
                let mut rows=Vec::new();if let Some((id,_))=relic{rows.push((RoomChoiceKind::Relic(id),None));}
                rows.extend([(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)]);
                let mut next=run.next_choice;let mut prepared=view(&mut next,RoomViewKind::Treasure,rows)?;
                for row in &mut prepared.choices{if row.kind==RoomChoiceKind::Sapphire&&run.state.keys[1]{row.status=RoomChoiceStatus::Disabled;}}
                run.next_choice=next;let room=run.state.room_loop.as_mut().unwrap();room.screen=prepared;
                if let Some((_,cursor))=relic{room.m10_elite_relic_cursor=cursor;}
                chest_opened(run,false)?;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
                return Ok(empty_transition())
            }
            if (is_m9(&run.state)||is_m14(&run.state)||is_m19(&run.state))&&run.state.act==3 {
                let captures=[RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial];
                let available=captures.iter().find(|id|!run.state.relics.contains(id));
                let relics=available.map_or_else(||captures.to_vec(),|id|vec![*id]);
                let mut rows=relics.into_iter().map(|id|(RoomChoiceKind::Relic(id),None)).collect::<Vec<_>>();
                rows.extend([(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)]);
                let mut next=run.next_choice;let mut prepared=view(&mut next,RoomViewKind::Treasure,rows)?;
                for row in &mut prepared.choices{if matches!(row.kind,RoomChoiceKind::Relic(id) if run.state.relics.contains(&id))||row.kind==RoomChoiceKind::Sapphire&&run.state.keys[1]{row.status=RoomChoiceStatus::Disabled;}}
                run.next_choice=next;run.state.room_loop.as_mut().unwrap().screen=prepared;
                chest_opened(run,false)?;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
                crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
                return Ok(empty_transition())
            }
            let relic=if uses_captured_act1(&run.state){[RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial].into_iter().find(|id|!run.state.relics.contains(id)).ok_or(RunError::Unsupported("captured ordinary treasure relics exhausted"))?}else{RelicId::Strawberry};
            let rows=view(&mut run.next_choice,RoomViewKind::Treasure,vec![(RoomChoiceKind::Relic(relic),None),(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Proceed,None)])?;
            run.state.room_loop.as_mut().unwrap().screen=rows;
            chest_opened(run,false)?;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpened{boss:false})?;
            crate::items::on_hook(&mut run.state,crate::combat::Hook::ChestOpenedAfter{boss:false})?;
        }
        RoomChoiceKind::Relic(relic)=>{
            if run.state.relics.contains(&relic){return Err(RunError::InvalidAction)}
            let node_kind=run.state.map.as_ref().and_then(|map|map.current.and_then(|node|map.nodes.iter().find(|candidate|candidate.id==node))).map(|node|node.kind);
            let source=if screen.kind==RoomViewKind::Treasure{crate::items::lifecycle::RelicSource::Chest}else if node_kind==Some(MapNodeKind::Elite){crate::items::lifecycle::RelicSource::Elite}else{crate::items::lifecycle::RelicSource::Reward};
            if run.state.item_admission==ItemAdmission::Complete{
                let mut staged=run.clone();
                let obtained=crate::items::obtain_relic(&mut staged.state,relic,source)?;
                claimed(&mut staged,id);if screen.kind==RoomViewKind::Treasure{exclude_base(&mut staged);}
                let transition=crate::items::lifecycle::after_obtain(&mut staged,id,relic,obtained)?;
                *run=staged;
                return Ok(transition)
            }
            crate::items::obtain_relic(&mut run.state,relic,source)?;claimed(run,id);if screen.kind==RoomViewKind::Treasure{exclude_base(run);}
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
    let complete=run.state.item_admission==ItemAdmission::Complete;
    let room=run.state.room_loop.as_mut().unwrap();let mut parent=room.reward_parent.take().ok_or(RunError::InvalidAction)?;
    let pending=if complete{RoomChoiceStatus::Disabled}else{RoomChoiceStatus::Offered};
    parent.choices.iter_mut().find(|choice|choice.kind==RoomChoiceKind::CardReward&&choice.status==pending).ok_or(RunError::InvalidAction)?.status=status;
    room.screen=parent;Ok(empty_transition())
}
fn card_parent(run:&Run)->Result<(),RunError>{
    let room=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?;
    let parent=room.reward_parent.as_ref().ok_or(RunError::InvalidAction)?;
    let pending=if run.state.item_admission==ItemAdmission::Complete{RoomChoiceStatus::Disabled}else{RoomChoiceStatus::Offered};
    if room.screen.kind!=RoomViewKind::CardReward||parent.kind!=RoomViewKind::Rewards
        ||!parent.choices.iter().any(|choice|choice.kind==RoomChoiceKind::CardReward&&choice.status==pending){return Err(RunError::InvalidAction)}
    if let Some(captured)=&room.active_rewards{
        let child=if run.state.item_admission==ItemAdmission::Complete{captured_child_at(&captured.cards,active_card_reward_index(parent)?)?}else{captured.cards.clone()};
        if room.screen!=child{return Err(RunError::InvalidAction)}
    }
    Ok(())
}
fn exclude_base(run:&mut Run){
    for row in &mut run.state.room_loop.as_mut().unwrap().screen.choices {if matches!(row.kind,RoomChoiceKind::Relic(_)|RoomChoiceKind::Sapphire)&&row.status==RoomChoiceStatus::Offered{row.status=RoomChoiceStatus::Skipped}}
}

/// M22 finite capture: ownership filtering precedes every nonempty checked request.
/// Stage even the rejection loop so a failed preparation leaves its caller intact.
pub(in crate::run) fn capture_m22_relic(rng:&mut RunRng,key:&[u64],pool:&[RelicId],owned:&[RelicId])->Result<Option<RelicId>,RunError>{
    capture_relic_in_scope(rng,super::m22_content::RELIC_CAPTURE_SCOPE,key,pool,owned)
}
/// The single finite capture implementation; callers supply only the scope.
pub(in crate::run) fn capture_relic_in_scope(rng:&mut RunRng,scope:RunRngScope,key:&[u64],pool:&[RelicId],owned:&[RelicId])->Result<Option<RelicId>,RunError>{
    let eligible=pool.iter().copied().filter(|id|!owned.contains(id)).collect::<Vec<_>>();
    if eligible.is_empty(){return Ok(None)}
    let mut staged=rng.clone();
    let index=staged.bounded_scoped_checked(scope,key,eligible.len() as u64)?;
    *rng=staged;Ok(Some(eligible[index]))
}
