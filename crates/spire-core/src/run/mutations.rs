use super::*;
use crate::content::CardId;
pub(super) fn gain_gold(state:&mut RunState,amount:u32)->Result<(),RunError>{
    state.gold=state.gold.checked_add(amount).ok_or(RunError::Unsupported("gold overflow"))?;Ok(())
}
pub(super) fn supported(id:CardId)->bool {matches!(id,CardId::Strike|CardId::Defend|CardId::Bash|CardId::Clothesline
    |CardId::IronWave|CardId::PommelStrike|CardId::ShrugItOff|CardId::Anger|CardId::Bloodletting|CardId::BurningPact|CardId::Disarm)}
pub(super) fn supported_in(state:&RunState,id:CardId)->bool{if super::rooms::is_m19(state){return super::M19_DEFECT_ACT3_CARDS.contains(&id)&&(id==CardId::Parasite||crate::content::defect::supports_permanent(id))}if super::rooms::is_m18(state){return super::M18_DEFECT_ACT2_CARDS.contains(&id)&&crate::content::defect::supports_permanent(id)}if super::rooms::is_m17(state){return super::M17_DEFECT_ACT1_CARDS.contains(&id)}if state.authored_combat&&state.character==crate::content::Character::Defect{return crate::content::defect::supports_permanent(id)}if super::rooms::is_m15(state){return super::M15_SILENT_HEART_CARDS.contains(&id)}if super::rooms::is_m14(state){return super::M14_SILENT_ACT3_CARDS.contains(&id)}if super::rooms::is_m12(state)||super::rooms::is_m13(state){return super::M12_SILENT_ACT1_CARDS.contains(&id)}supported(id)||(super::rooms::is_m10(state)&&[crate::content::CardRarity::Basic,crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare].into_iter().any(|rarity|super::rooms::m10_content::red_candidates(rarity).contains(&id)))||(super::rooms::uses_captured_act1(state)&&matches!(id,CardId::Cleave|CardId::TwinStrike|CardId::Thunderclap|CardId::Bludgeon|CardId::Impervious|CardId::Offering))||(super::rooms::uses_captured_act3(state)&&state.act==3&&id==CardId::Inflame)}
pub(super) fn obtain_card(state:&mut RunState,id:CardId,upgrades:u32)->Result<u64,RunError>{
    if !supported_in(state,id)||upgrades>1||state.deck.len()>=999{return Err(RunError::InvalidAction)}
    let instance=state.next_instance;let next=instance.checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
    if instance==0||state.deck.iter().any(|c|c.instance==instance){return Err(RunError::InvalidAction)}
    state.deck.push(CardInstance{ misc:if id==CardId::GeneticAlgorithm{1}else{0},id,instance,upgrades});state.next_instance=next;
    crate::items::on_hook(state,crate::combat::Hook::CardAcquired)?;Ok(instance)
}
pub(super) fn remove_card(state:&mut RunState,id:u64)->Result<CardInstance,RunError>{
    let at=state.deck.iter().position(|c|c.instance==id).ok_or(RunError::InvalidAction)?;
    let parasite=state.deck[at].id==CardId::Parasite;
    if parasite&&(state.combat.is_some()||state.max_hp==0||state.max_hp>999||state.hp>state.max_hp){return Err(RunError::InvalidAction)}
    let card=state.deck.remove(at);
    if parasite{state.max_hp=state.max_hp.saturating_sub(3).max(1);state.hp=state.hp.min(state.max_hp);}
    crate::items::on_hook(state,crate::combat::Hook::CardRemoved)?;Ok(card)
}
pub(super) fn upgrade_card(state:&mut RunState,id:u64)->Result<(),RunError>{
    let at=state.deck.iter().position(|c|c.instance==id&&can_upgrade(state,c)).ok_or(RunError::InvalidAction)?;
    let original=&state.deck[at];
    let mut live=crate::combat::permanent_live_card_with_misc(original.id,crate::combat::CardInstanceId(original.instance),original.upgrades,original.misc);
    crate::combat::cards::ironclad::rules::upgrade(&mut live)?;
    state.deck[at].upgrades=live.upgrades;Ok(())
}
pub(super) fn can_upgrade(state:&RunState,card:&CardInstance)->bool{
    (supported_in(state,card.id)||!super::rooms::uses_silent_content(state)&&matches!(card.id,CardId::Armaments|CardId::SearingBlow))
        &&crate::combat::cards::ironclad::rules::can_upgrade(card.id,card.upgrades)
}
pub(super) fn heal(state:&mut RunState,amount:u32)->Result<(),RunError>{
    if state.max_hp==0||state.max_hp>999||state.hp>state.max_hp{return Err(RunError::InvalidAction)}
    let amount=i32::try_from(amount).map_err(|_|RunError::InvalidAction)?;
    let heal=crate::items::ordinary_heal_amount(&state.relics,amount).max(0) as u32;
    let before=state.hp;
    if state.hp>0{state.hp=state.hp.saturating_add(heal).min(state.max_hp)}
    crate::combat::observe::run_heal(state,before);Ok(())
}
pub(super) fn increase_max_hp(state:&mut RunState,amount:u32)->Result<(),RunError>{
    let max=state.max_hp.checked_add(amount).filter(|m|*m<=999).ok_or(RunError::Unsupported("maximum HP overflow"))?;
    if state.max_hp==0||state.hp>state.max_hp||amount>i32::MAX as u32{return Err(RunError::InvalidAction)}
    state.max_hp=max;heal(state,amount)
}
pub(super) fn acquire_key(state:&mut RunState,key:Key)->Result<CommittedEvent,RunError>{
    let i=super::reward::key_index(key);if state.keys[i]{return Err(RunError::InvalidAction)}
    let event=CommittedEvent::KeyAcquired(key);super::reward::preflight(state,std::slice::from_ref(&event))?;
    state.keys[i]=true;Ok(event)
}
