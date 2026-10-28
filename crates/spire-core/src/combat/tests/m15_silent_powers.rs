use super::super::*;
use crate::content::{CardId,Catalog};
use super::full_runtime::entered_combat;
use model::OwnedCard;

fn id(token:&str)->CardId{Catalog::standard().cards().iter().find(|r|r.id.as_str()==token).unwrap_or_else(||panic!("missing green power family {token}")).id}
fn fight(token:&str,up:u32)->Combat{
    let mut c=entered_combat(9,9,|card|*card=permanent_live_card(id(token),card.instance,up));
    // Cultist's first action is Incantation. Start on its real six-damage turn
    // so the lifetime/retaliation checks actually exercise an incoming hit.
    c.step_full(&CombatAction::EndTurn).unwrap();
    c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c
}
fn power(c:&mut Combat,name:&'static str,amount:i32){c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId(name),amount}}],None).unwrap();}
fn amount(c:&Combat,name:&str)->i32{c.state.player_full_powers.iter().find(|p|p.id.0==name).map_or(0,|p|p.state.amount)}
fn play(c:&mut Combat,target:bool){c.step_full(&CombatAction::PlayCard{instance:3,target:target.then_some(TargetId(1))}).unwrap();}
fn add_draw(c:&mut Combat,count:u64){for n in 100..100+count{let card=permanent_live_card(CardId::DefendGreen,CardInstanceId(n),0);c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(OwnedCard::Full(card));}}

#[test]
fn m15_powers_capture_block_and_recharge_once(){
    for up in 0..=1 {
        let mut c=fight("dodge_and_roll",up);power(&mut c,"dexterity",2);play(&mut c,false);
        assert_eq!(c.state.player.block,6+2*up as i32);
        power(&mut c,"dexterity",8);
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(c.state.player.block,6+2*up as i32,"delayed calculated Block does not read new Dexterity");
        assert_eq!(amount(&c,"next_turn_block"),0);
        for (token,energy,damage) in [("outmaneuver",2+up as i32,0),("flying_knee",1,8+3*up as i32)]{
            let mut c=fight(token,up);play(&mut c,token=="flying_knee");
            assert_eq!(c.state.player.energy,8);
            assert_eq!(c.state.enemies[0].hp,500-damage);
            c.step_full(&CombatAction::EndTurn).unwrap();
            assert_eq!(c.state.player.energy,9+energy);
            assert_eq!(amount(&c,"energized"),0);
            c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.energy,9);
        }
    }
}

#[test]
fn m15_powers_draw_predraw_and_postdraw_keep_actual_cards(){
    for up in 0..=1 {
        let mut c=fight("predator",up);add_draw(&mut c,12);play(&mut c,true);
        assert_eq!(c.state.enemies[0].hp,485-5*up as i32);
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.piles.hand.len(),7);
        assert_eq!(amount(&c,"draw_card_next_turn"),0);
        let mut c=fight("infinite_blades",up);add_draw(&mut c,8);play(&mut c,false);
        assert!(!c.state.piles.hand.iter().any(|x|x.full().unwrap().id==CardId::Shiv));
        c.step_full(&CombatAction::EndTurn).unwrap();
        let shivs=c.public_full().unwrap().hand.into_iter().filter(|c|c.id==CardId::Shiv).collect::<Vec<_>>();
        assert_eq!(shivs.len(),1);assert_eq!(shivs[0].upgrades,0);
        let mut c=fight("tools_of_the_trade",up);add_draw(&mut c,12);play(&mut c,false);
        c.step_full(&CombatAction::EndTurn).unwrap();
        let view=c.public_full().unwrap();assert_eq!(view.hand.len(),6);
        assert_eq!(view.selection.as_ref().unwrap().required,1);
        let chosen=view.hand[0].instance.0;c.step_full(&CombatAction::SelectCard{instance:chosen}).unwrap();
        assert_eq!(c.state.piles.hand.len(),5);assert_eq!(c.state.player.energy,9);
        assert_eq!(c.state.silent.discarded_this_turn,1);
        assert!(c.state.pending_selection.is_none());
    }
}

#[test]
fn m15_powers_wraith_duration_and_negative_downside_are_independent(){
    for up in 0..=1 {
        let mut c=fight("wraith_form",up);play(&mut c,false);
        assert_eq!(amount(&c,"intangible_player"),2+up as i32);assert_eq!(amount(&c,"wraith_form"),-1);
        let hp=c.state.player.hp;
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp-1);
        assert_eq!(amount(&c,"dexterity"),-1);
        assert_eq!(amount(&c,"intangible_player"),1+up as i32);
        for _ in 0..1+up{c.step_full(&CombatAction::EndTurn).unwrap();}
        assert_eq!(amount(&c,"intangible_player"),0);assert_eq!(amount(&c,"wraith_form"),-1);
        assert_eq!(amount(&c,"dexterity"),-2-up as i32);
        let mut protected=fight("wraith_form",up);power(&mut protected,"artifact",1);play(&mut protected,false);
        assert_eq!(amount(&protected,"wraith_form"),0);assert_eq!(amount(&protected,"artifact"),0);
        assert_eq!(amount(&protected,"intangible_player"),2+up as i32);
    }
}

#[test]
fn m15_powers_footwork_blur_and_phantasmal_have_distinct_lifetimes(){
    for up in 0..=1 {
        let mut c=fight("footwork",up);play(&mut c,false);assert_eq!(amount(&c,"dexterity"),2+up as i32);
        let mut c=fight("blur",up);power(&mut c,"dexterity",10);play(&mut c,false);
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.block,9+3*up as i32,"retained after six incoming damage");
        assert_eq!(amount(&c,"blur"),0);
        let mut c=fight("phantasmal_killer",up);play(&mut c,false);
        assert_eq!(amount(&c,"double_damage"),0);
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,"double_damage"),1);
        let strike=c.public_full().unwrap().hand.into_iter().find(|x|x.id==CardId::Strike).unwrap().instance.0;
        c.step_full(&CombatAction::PlayCard{instance:strike,target:Some(TargetId(1))}).unwrap();
        assert_eq!(c.state.enemies[0].hp,488);
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,"double_damage"),0);
    }
}

#[test]
fn m15_powers_thorns_and_envenom_use_actual_per_hit_categories(){
    for up in 0..=1 {
        let mut c=fight("caltrops",up);play(&mut c,false);c.state.player.block=50;
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(c.state.enemies[0].hp,497-2*up as i32);
        let mut c=fight("envenom",up);play(&mut c,false);
        c.state.enemies[0].block=6;
        c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
        assert!(!c.state.enemies[0].full_powers.iter().any(|p|p.id.0=="poison"));
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(2))}],None).unwrap();
        assert_eq!(c.state.enemies[0].full_powers.iter().find(|p|p.id.0=="poison").unwrap().state.amount,1);
    }
}

#[test]
fn m15_powers_use_and_after_played_callbacks_do_not_trigger_new_installation(){
    for up in 0..=1 {
        let mut c=fight("after_image",up);play(&mut c,false);
        assert_eq!(c.state.player.block,0);
        c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
        assert_eq!(c.state.player.block,1);
        let mut c=fight("a_thousand_cuts",up);play(&mut c,false);
        assert_eq!(c.state.enemies[0].hp,500);
        c.state.enemies[0].hp=7+up as i32;
        c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
        assert_eq!(c.state.outcome,Some(Outcome::Victory));
        assert!(c.state.in_play.is_none(),"use cleanup precedes Thousand Cuts lethal action");
        assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="1"));
        let mut c=fight("choke",up);play(&mut c,true);assert_eq!(c.state.enemies[0].hp,488);
        c.state.enemies[0].block=100;
        c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
        assert_eq!(c.state.enemies[0].hp,485-2*up as i32);
        assert_eq!(c.state.enemies[0].block,100,"Choke is HP loss");
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert!(!c.state.enemies[0].full_powers.iter().any(|p|p.id.0=="choked"));
    }
}

#[test]
fn m15_powers_intangible_limits_each_named_damage_category(){
    let mut c=fight("wraith_form",0);play(&mut c,false);
    for damage in [DamagePayload::Fixed(8),DamagePayload::Thorns(8),DamagePayload::HpLoss(8)]{
        let before=c.state.player.hp;
        let source=if matches!(damage,DamagePayload::Thorns(_)){
            c.state.enemies[0].enemy_id=model::ActorEnemyId::Full(crate::content::EnemyId::Spiker);
            c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("thorns"),state:PowerState{amount:8,skip_first:false,monster_source_grace:false}});
            Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone()))
        }else{Source::Actor(Actor::Player)};
        c.enqueue_native_effects(vec![Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(damage)}],None).unwrap();
        assert_eq!(c.state.player.hp,before-1);
    }
}

#[test]
fn m15_powers_tools_large_stack_uses_hand_capacity_without_draw_cap(){
    let mut c=fight("tools_of_the_trade",0);play(&mut c,false);
    power(&mut c,"tools_of_the_trade",10);add_draw(&mut c,20);
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.piles.hand.len(),0,"draw fills ten slots, discard11 takes all available");
    assert_eq!(c.state.silent.discarded_this_turn,10);
    assert!(c.state.pending_selection.is_none());
}

#[test]
fn independent_review_phantasmal_constructor() {
    let mut c = fight("phantasmal_killer", 0);
    let second = permanent_live_card(CardId::PhantasmalKiller, CardInstanceId(100), 0);
    c.state.card_ids_issued.insert(card_state::handle(second.instance));
    c.state.piles.hand.push(OwnedCard::Full(second));
    play(&mut c, false);
    c.step_full(&CombatAction::PlayCard { instance:100, target:None }).unwrap();
    assert_eq!(amount(&c,"phantasmal"),2);
    c.step_full(&CombatAction::EndTurn).unwrap();
    println!("stacked Phantasmal first delivery: phantasmal={} double_damage={}",amount(&c,"phantasmal"),amount(&c,"double_damage"));
    assert_eq!(amount(&c,"double_damage"),1,"new Double Damage constructor starts at one; application amount is used only on existing stacks");
}


#[test]
fn m15_review_phantasmal_burst_and_existing_stack_have_separate_duration_rules() {
    let mut c=fight("burst",0);
    let card=permanent_live_card(CardId::PhantasmalKiller,CardInstanceId(100),0);
    c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(OwnedCard::Full(card));
    play(&mut c,false);c.step_full(&CombatAction::PlayCard{instance:100,target:None}).unwrap();
    assert_eq!(amount(&c,"phantasmal"),2);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((amount(&c,"phantasmal"),amount(&c,"double_damage")),(1,1));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((amount(&c,"phantasmal"),amount(&c,"double_damage")),(0,1));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,"double_damage"),0);
    let mut existing=fight("phantasmal_killer",0);
    play(&mut existing,false);power(&mut existing,"phantasmal",1);
    power(&mut existing,"double_damage",2);assert_eq!(amount(&existing,"double_damage"),1);
    power(&mut existing,"double_damage",2);assert_eq!(amount(&existing,"double_damage"),3);
    existing.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::TurnStartedPreDraw)}],None).unwrap();
    assert_eq!((amount(&existing,"phantasmal"),amount(&existing,"double_damage")),(1,5),"existing power receives full queued Phantasmal delta2");
}
