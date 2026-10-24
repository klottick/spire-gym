use super::super::*;
use super::full_runtime::entered_combat;
use crate::content::CardId;
use crate::combat::model::OwnedCard;

fn armaments(upgrades:u32)->Combat {
    entered_combat(3,3,|card|{card.id=CardId::Armaments;card.upgrades=upgrades;})
}
fn play_armaments(c:&mut Combat){
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
}

// Break caught: upgrading all piles, upgrading the master source, or treating
// Armaments' base child as an exhaust/discard instead of an owned upgrade.
#[test]
fn m10_upgrade_armaments_changes_owned_combat_cards_only(){
    let mut c=armaments(0);
    play_armaments(&mut c);
    let public=c.public_full().unwrap();
    assert_eq!((public.player.block,public.player.energy),(5,2));
    assert_eq!(public.selection.unwrap().required,1);
    assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"3");
    let before=c.clone();
    assert_eq!(c.step_full(&CombatAction::SelectCard{instance:999}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c,before);
    c.step_full(&CombatAction::SelectCard{instance:1}).unwrap();
    let public=c.public_full().unwrap();
    assert_eq!(public.hand.iter().find(|card|card.instance.0==1).unwrap().upgrades,1);
    assert_eq!(public.hand.iter().find(|card|card.instance.0==2).unwrap().upgrades,0);
    assert_eq!(public.discard.iter().map(|card|card.instance.0).collect::<Vec<_>>(),[3]);
    assert_eq!(public.hand.iter().find(|card|card.instance.0==1).unwrap().origin,CombatCardRef::Permanent(CardInstanceId(1)));
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.public_full().unwrap().enemies[0].hp,39,"temporary upgrade changes the actual Strike hit");
}

#[test]
fn m10_upgrade_armaments_all_zero_and_single_eligibility(){
    for (upgrade,eligible,expected) in [(0,0,0),(0,1,1),(1,2,2)]{
        let mut c=armaments(upgrade);
        for card in &mut c.state.piles.hand{
            if card.handle().as_str()=="3"{continue}
            let live=card.full_mut().unwrap();
            if live.instance.0>eligible{live.upgrades=1;}
        }
        let before=c.state.piles.hand.iter().filter(|c|c.full().unwrap().upgrades==0).count();
        play_armaments(&mut c);
        assert!(c.public_full().unwrap().selection.is_none());
        let remaining=c.state.piles.hand.iter().filter(|c|c.full().unwrap().upgrades==0).count();
        assert_eq!(before-usize::from(upgrade==0)-remaining,expected);
        assert_eq!(c.public_full().unwrap().player.block,5);
    }
}

// Break caught: flattening Searing Blow's count to an upgraded boolean or
// rebuilding an independently copied, actually upgraded card as fresh damage12.
#[test]
fn m10_upgrade_searing_blow_upgrade_and_copy_are_distinct(){
    let mut c=armaments(1);
    *c.state.piles.hand.iter_mut().find(|c|c.handle().as_str()=="1").unwrap().full_mut().unwrap()=
        permanent_live_card(CardId::SearingBlow,CardInstanceId(1),0);
    for (count,damage) in [(0,12),(1,16),(2,21),(3,27),(4,34)]{
        let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
        assert_eq!(prepared.effects[0].kind,EffectKind::Damage(DamagePayload::Fixed(damage)));
        let card=c.state.piles.hand.iter().find(|c|c.handle().as_str()=="1").unwrap().full().unwrap();
        assert_eq!(card.upgrades,count);
        let copied=card.copy_as(CardInstanceId(100));
        assert_eq!(copied.upgrades,count);
        assert_eq!(copied.origin,CombatCardRef::CombatLocal(100));
        assert_eq!(copied.local,card.local);
        if count<4{play_armaments(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();}
    }
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.public_full().unwrap().enemies[0].hp,14);
    let fresh=permanent_live_card(CardId::SearingBlow,CardInstanceId(1),0);
    assert_eq!(fresh.upgrades,0,"combat upgrades never rewrite a permanent descriptor");
}

#[test]
fn m10_upgrade_fault_retains_selected_parent(){
    let mut c=armaments(0);play_armaments(&mut c);
    c.state.resolved_effects=99_999;
    let error=c.step_full(&CombatAction::SelectCard{instance:1}).unwrap_err();
    let crate::run::RunError::Faulted(packet)=&error else{panic!("retained fault expected")};
    let crate::run::RunFaultKind::Combat(native)=&packet.kind else{panic!("combat fault expected")};
    assert_eq!(native.current.as_ref().unwrap().source,Source::Card(CardHandle::parse("3").unwrap()));
    assert_eq!(native.selection.as_ref().unwrap().selected,[CardHandle::parse("1").unwrap()]);
    assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"3");
    assert_eq!(c.state.piles.hand.iter().find(|c|c.handle().as_str()=="1").unwrap().full().unwrap().upgrades,0);
    let before=c.clone();assert_eq!(c.step_full(&CombatAction::SelectCard{instance:1}),Err(error));assert_eq!(c,before);
}

#[test]
fn m10_upgrade_copy_preserves_actual_damage_but_fresh_count_does_not_replay(){
    let source=permanent_live_card(CardId::SearingBlow,CardInstanceId(41),4);
    let mut copied=source.copy_as(CardInstanceId(100));
    assert_eq!(cards::ironclad::rules::public_values(&source).unwrap().base_damage,Some(34));
    assert_eq!(cards::ironclad::rules::public_values(&copied).unwrap().base_damage,Some(34));
    cards::ironclad::rules::upgrade(&mut copied).unwrap();
    assert_eq!(cards::ironclad::rules::public_values(&copied).unwrap().base_damage,Some(42));
    assert_eq!(cards::ironclad::rules::public_values(&source).unwrap().base_damage,Some(34),"copy growth does not mutate its source");

    let mut c=armaments(0);
    let hand_index=c.state.piles.hand.iter().position(|card|card.handle().as_str()=="1").unwrap();
    c.state.piles.hand[hand_index]=OwnedCard::Full(copied.copy_as(CardInstanceId(1)));
    assert!(super::super::engine::validate_runtime(&c.state).is_ok(),"copied played identity uses an already-issued combat handle: {:?}",super::super::engine::validate_runtime(&c.state));
    let before=c.public_full().unwrap().enemies[0].hp;
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert!(super::super::engine::validate_runtime(&c.state).is_ok(),"{:?}",super::super::engine::validate_runtime(&c.state));
    assert_eq!(before-c.public_full().unwrap().enemies[0].hp,42,"copied Searing Blow executes its retained damage");

    let fresh=super::super::card_state::generated(CardId::SearingBlow,CardInstanceId(101),4).unwrap();
    assert_eq!(fresh.upgrades,4);
    assert_eq!(cards::ironclad::rules::public_values(&fresh).unwrap().base_damage,Some(12),"fresh constructor count does not replay historical upgrades");
    let mut fresh_combat=armaments(0);
    let hand_index=fresh_combat.state.piles.hand.iter().position(|card|card.handle().as_str()=="1").unwrap();
    fresh_combat.state.piles.hand[hand_index]=OwnedCard::Full(fresh.copy_as(CardInstanceId(1)));
    assert!(super::super::engine::validate_runtime(&fresh_combat.state).is_ok(),"fresh played identity uses an already-issued combat handle: {:?}",super::super::engine::validate_runtime(&fresh_combat.state));
    let before=fresh_combat.public_full().unwrap().enemies[0].hp;
    fresh_combat.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert!(super::super::engine::validate_runtime(&fresh_combat.state).is_ok(),"{:?}",super::super::engine::validate_runtime(&fresh_combat.state));
    assert_eq!(before-fresh_combat.public_full().unwrap().enemies[0].hp,12);
}

#[test]
fn m10_upgrade_armaments_does_not_upgrade_other_piles(){
    for armaments_upgrades in [0,1]{
        let mut c=armaments(armaments_upgrades);
        c.state.piles.hand[1].full_mut().unwrap().upgrades=1;
        let extras=[(201,CardId::Strike),(202,CardId::Strike),(203,CardId::Strike)];
        for (pile,(instance,id)) in [&mut c.state.piles.draw,&mut c.state.piles.discard,&mut c.state.piles.exhaust].into_iter().zip(extras){
            pile.push(OwnedCard::Full(permanent_live_card(id,CardInstanceId(instance),0)));
            c.state.card_ids_issued.insert(super::super::card_state::handle(CardInstanceId(instance)));
        }
        assert!(super::super::engine::validate_runtime(&c.state).is_ok(),"fixture must have complete issued-card ownership before the action");
        play_armaments(&mut c);
        assert!(super::super::engine::validate_runtime(&c.state).is_ok(),"Armaments must preserve valid ownership across its action");
        let upgraded=c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.upgrades==1).collect::<Vec<_>>();
        assert_eq!(upgraded.iter().find(|card|card.instance.0==1).unwrap().upgrades,1);
        assert_eq!(cards::ironclad::rules::public_values(upgraded.iter().find(|card|card.instance.0==1).unwrap()).unwrap().base_damage,Some(9));
        assert_eq!(c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.instance.0==2).unwrap().upgrades,1);
        for pile in [&c.state.piles.draw,&c.state.piles.discard,&c.state.piles.exhaust]{
            let card=pile.iter().find_map(OwnedCard::full).unwrap();
            assert_eq!(card.upgrades,0);
            assert_eq!(cards::ironclad::rules::public_values(card).unwrap().base_damage,Some(6));
        }
    }
}

#[test]
fn m10_upgrade_public_values_are_owned_actual_stats(){
    let mut c=armaments(0);
    let armaments_view=c.public_full().unwrap().hand.into_iter().find(|card|card.id==CardId::Armaments).unwrap();
    assert_eq!(armaments_view.values.as_ref().unwrap().base_block,Some(5));
    assert_eq!(armaments_view.values.as_ref().unwrap().base_damage,None);
    let mut searing=permanent_live_card(CardId::SearingBlow,CardInstanceId(1),0);
    for _ in 0..4{cards::ironclad::rules::upgrade(&mut searing).unwrap();}
    let actual=cards::ironclad::rules::public_values(&searing).unwrap();
    assert_eq!(actual.base_damage,Some(34));
    assert_eq!(actual.base_block,None);
    assert_eq!(actual.magic_number,None);
    assert_eq!(actual.damage_bonus,None);

    let hand_index=c.state.piles.hand.iter().position(|card|card.handle().as_str()=="1").unwrap();
    c.state.piles.hand[hand_index]=OwnedCard::Full(searing);
    assert!(super::super::engine::validate_runtime(&c.state).is_ok(),"{:?}",super::super::engine::validate_runtime(&c.state));
    let retained=c.public_full().unwrap().hand.into_iter().find(|card|card.instance.0==1).unwrap();
    assert_eq!(retained.values.as_ref().unwrap().base_damage,Some(34));
    cards::ironclad::rules::upgrade(c.state.piles.hand[hand_index].full_mut().unwrap()).unwrap();
    let later=c.public_full().unwrap().hand.into_iter().find(|card|card.instance.0==1).unwrap();
    assert_eq!(later.values.as_ref().unwrap().base_damage,Some(42));
    assert_eq!(retained.values.as_ref().unwrap().base_damage,Some(34),"owned public view is a detached snapshot");
    let copy=c.state.piles.hand[hand_index].full().unwrap().copy_as(CardInstanceId(51));
    assert_eq!(cards::ironclad::rules::public_values(&copy).unwrap().base_damage,Some(42));
    let before=c.public_full().unwrap().enemies[0].hp;
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert!(super::super::engine::validate_runtime(&c.state).is_ok(),"{:?}",super::super::engine::validate_runtime(&c.state));
    assert_eq!(before-c.public_full().unwrap().enemies[0].hp,42);
    assert_eq!(retained.values.as_ref().unwrap().base_damage,Some(34),"old public view remains an owned snapshot after a later play");
    assert_eq!(cards::ironclad::rules::public_values(&permanent_live_card(CardId::IronWave,CardInstanceId(51),0)),None,"unsupported card stats remain unknown");
}
