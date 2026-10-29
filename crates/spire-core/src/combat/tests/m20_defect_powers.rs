use super::super::*;
use crate::content::{CardId,Character};
fn fight(id:CardId,up:u32)->Combat{
    let mut c=super::full_runtime::entered_combat(9,9,|card|*card=permanent_live_card(id,card.instance,up));
    c.state.silent.source_character=Some(Character::Defect);c.state.character=characters::CharacterState::for_character(Character::Defect);
    c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c
}
#[test]
fn m20_defect_powers_constructor_stack_callbacks_and_finish(){
    for (id,power,amounts) in [(CardId::Buffer,"buffer",[1,2]),(CardId::SelfRepair,"repair",[7,10]),
        (CardId::MachineLearning,"draw",[1,1]),(CardId::HelloWorld,"hello_world",[1,1]),(CardId::CreativeAI,"creative_ai",[1,1])]{
        for up in 0..=1 {
            let mut c=fight(id,up);let before_draw=c.state.player.draw_per_turn;let before_hand=c.state.piles.hand.len();
            c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
            assert_eq!(c.state.player_full_powers.iter().find(|entry|entry.id.0==power).unwrap().state.amount,amounts[up as usize]);
            assert_eq!(c.state.player.draw_per_turn,before_draw+i32::from(id==CardId::MachineLearning));
            assert_eq!(c.state.piles.hand.len(),before_hand-1,"MachineLearning construction does not draw immediately");
            assert!(c.state.played_powers.iter().any(|card|card.handle().as_str()=="3"));
            assert!(!c.state.piles.discard.iter().any(|card|card.handle().as_str()=="3"));
        }
    }
}
#[test]
fn m20_defect_powers_buffer_change_damage_hook_and_fully_blocked_charge(){
    for block in [0,10] {
        let mut c=fight(CardId::Buffer,0);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        c.state.player.block=block;let hp=c.state.player.hp;
        let enemy=c.state.enemies[0].instance_id.clone();
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(enemy)),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(5))}],None).unwrap();
        assert_eq!(c.state.player.hp,hp);
        let charges=c.state.player_full_powers.iter().find(|entry|entry.id.0=="buffer").map_or(0,|power|power.state.amount);
        assert_eq!(charges,i32::from(block>0));assert_eq!(c.state.player.block,(block-5).max(0));
    }
}
#[test]
fn m20_defect_powers_repair_direct_positive_hp_and_no_resurrection(){
    for up in 0..=1 {
        let mut c=fight(CardId::SelfRepair,up);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        c.state.player.hp=10;
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::CombatSettled)}],None).unwrap();
        assert_eq!(c.state.player.hp,if up>0{20}else{17});
    }
}
#[test]
fn m20_defect_powers_machine_learning_constructor_stack_removal_and_wide_draw(){
    let mut c=fight(CardId::MachineLearning,0);let before=c.state.player.draw_per_turn;
    for card in &mut c.state.piles.hand{let instance=card.full().unwrap().instance;*card=model::OwnedCard::Full(permanent_live_card(CardId::MachineLearning,instance,0));}
    for n in [1,2,3]{c.step_full(&CombatAction::PlayCard{instance:n,target:None}).unwrap();}
    assert_eq!(c.state.player.draw_per_turn,before+3);assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="draw").unwrap().state.amount,3);
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("draw"))}],None).unwrap();assert_eq!(c.state.player.draw_per_turn,before);
    let mut c=fight(CardId::MachineLearning,0);c.state.silent.history_enabled=true;c.state.player.draw_per_turn=12;
    for n in 100..120{let card=permanent_live_card(CardId::DefendBlue,CardInstanceId(n),0);c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(model::OwnedCard::Full(card));}
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert_eq!(c.state.player.draw_per_turn,13);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.piles.hand.len(),10,"draw request is wider than hand admission; hand cap stays10");assert!(engine::validate_runtime(&c.state).is_ok());
}
