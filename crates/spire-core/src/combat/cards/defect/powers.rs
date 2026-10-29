//! New blue Power producers and source-specific constructor side effects.
use super::*;
pub(super) fn payload(_state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>{
    let up=card.upgrades>0;let (id,amount)=match card.id {
        CardId::Buffer=>("buffer",if up{2}else{1}),CardId::SelfRepair=>("repair",if up{10}else{7}),
        CardId::MachineLearning=>("draw",1),CardId::HelloWorld=>("hello_world",1),CardId::CreativeAI=>("creative_ai",1),_=>return Ok(None),
    };
    if target.is_some(){return Err(RunError::InvalidAction)}
    Ok(Some((starting_cost(card.id,card.upgrades).unwrap(),vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId(id),amount})])))
}
pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    let up=card.upgrades>0;let magic=match card.id {
        CardId::Buffer=>if up{2}else{1},CardId::SelfRepair=>if up{10}else{7},CardId::MachineLearning|CardId::HelloWorld|CardId::CreativeAI=>1,_=>return None,
    };
    Some(CardPublicValues{base_damage:None,base_block:None,magic_number:Some(magic),damage_bonus:None,cost_kind:CardCostKind::Fixed,
        free_play:card.local.free_play||card.local.free_to_play_once,exhaust_on_finish:card.local.exhaust_on_finish,
        innate:card.local.innate,ethereal:card.local.ethereal,temporary_cost:card.local.temporary_cost})
}
pub(super) fn capture(state:&mut State,effects:&[Effect]){
    // DrawPower(amount) mutates gameHandSize at construction, before later
    // ApplyPowerAction chooses between initial installation and stacking.
    for effect in effects {
        if let EffectKind::ApplyFullPower{id:FullPowerId("draw"),amount}=&effect.kind {
            if matches!(&effect.source,Source::Card(owner) if state.in_play.as_ref().filter(|card|card.handle()==*owner)
                .or_else(||state.queued_autoplay.get(owner)).and_then(|card|card.full()).is_some_and(|card|card.id==CardId::MachineLearning)){
                state.player.draw_per_turn=state.player.draw_per_turn.wrapping_add(*amount);
            }
        }
    }
}
pub(super) fn after_use(state:&mut State){
    let Some(index)=state.player_full_powers.iter().position(|power|power.id.0=="rebound"&&power.state.amount>0)else{return};
    if state.defect.rebound_just_applied{state.defect.rebound_just_applied=false;return}
    if let Some(card)=state.in_play.as_mut().and_then(|card|card.full_mut()){
        if crate::content::Catalog::standard().card(card.id).is_some_and(|row|row.card_type!=crate::content::CardType::Power){card.local.rebound_on_finish=true;}
    }
    state.player_full_powers[index].state.amount-=1;
    if state.player_full_powers[index].state.amount==0{state.player_full_powers.remove(index);}
}
pub(super) fn hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    if hook==Hook::CardAfterUse{after_use(state);}
    if hook==Hook::PlayerPreEndTurnCards&&target==Some(&Actor::Player){
        state.player_full_powers.retain(|power|power.id.0!="rebound");state.defect.rebound_just_applied=false;
        if state.player_full_powers.iter().any(|power|power.id.0=="equilibrium"&&power.state.amount>0){
            state.temporary_retain=Some(crate::combat::model::TemporaryRetain{turn:state.turn,cards:state.piles.hand.iter()
                .filter(|card|card.full().is_some_and(|card|!card.local.ethereal)).map(|card|card.handle()).collect()});
        }
    }
    if hook==Hook::CombatSettled&&state.player.hp>0 {
        let amount=state.player_full_powers.iter().find(|power|power.id.0=="repair").map_or(0,|power|power.state.amount);
        if amount>0{crate::combat::effects::ordinary_heal_now(state,Source::Actor(Actor::Player),amount)?;}
    }
    if hook==Hook::TurnEnded&&target==Some(&Actor::Player){
        if state.player_full_powers.iter().any(|power|power.id.0=="equilibrium"&&power.state.amount>0){
            work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ReduceFullPower{id:FullPowerId("equilibrium"),amount:1}});
        }
    }
    Ok(())
}
