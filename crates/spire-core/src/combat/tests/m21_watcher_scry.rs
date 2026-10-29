use super::super::*;
#[test]
fn m21_watcher_scry_reveals_entitled_top_order_and_waits() {
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::ThirdEye,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    for instance in [20,21] {
        let card=permanent_live_card(crate::content::CardId::StrikePurple,CardInstanceId(instance),0);
        c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(model::OwnedCard::Full(card));
    }
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.player.block,7);
    let selection=c.public_full().unwrap().selection.unwrap();
    assert_eq!(selection.candidates.iter().map(|c|c.instance.0).collect::<Vec<_>>(),vec![21,20]);
    assert_eq!((selection.minimum,selection.maximum),(0,2));
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(c.public_full().unwrap().selection.is_none());
    assert_eq!(c.state.piles.draw.len(),2);
}

fn choice()->Combat {
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::ThirdEye,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    for instance in [20,21] {
        let card=permanent_live_card(crate::content::CardId::Weave,CardInstanceId(instance),0);
        c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(model::OwnedCard::Full(card));
    }
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();c
}
#[test]
fn m21_watcher_scry_selected_handles_can_be_deselected() {
    let mut c=choice();c.step_full(&CombatAction::SelectCard{instance:21}).unwrap();
    assert!(c.legal_actions_full().unwrap().iter().any(|a|format!("{a:?}")=="DeselectCard { instance: 21 }"));
}
#[test]
fn m21_watcher_scry_foresight_owns_the_predraw_choice() {
    let mut c=super::full_runtime::entered_combat(3,3, |_|{});
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("foresight"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}});
    c.step_full(&CombatAction::EndTurn).unwrap();
    let selection=c.public_full().unwrap().selection.expect("Foresight must suspend normal draw");
    assert_eq!(selection.parent,SelectionParent::Power{turn:2,id:FullPowerId("foresight")});
}
#[test]
fn m21_watcher_meditate_choice_returns_selected_physical_instance() {
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::Meditate,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    for instance in [20,21] {
        let card=permanent_live_card(crate::content::CardId::WindmillStrike,CardInstanceId(instance),0);
        c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.discard.push(model::OwnedCard::Full(card));
    }
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.public_full().unwrap().selection.unwrap().minimum,1);
    c.step_full(&CombatAction::SelectCard{instance:21}).unwrap();
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(c.state.character.stance,Stance::Calm);
    let selected=c.state.piles.hand.iter().find_map(|o|o.full().filter(|c|c.instance.0==21)).unwrap();
    assert_eq!(selected.local.base_damage,Some(11));
}
#[test]
fn m21_watcher_scry_deselect_reselect_appends_and_invalid_is_atomic(){
    let mut c=choice();
    c.step_full(&CombatAction::SelectCard{instance:21}).unwrap();c.step_full(&CombatAction::SelectCard{instance:20}).unwrap();
    let before=c.clone();assert!(c.step_full(&CombatAction::SelectCard{instance:20}).is_err());assert_eq!(c,before);
    assert!(c.step_full(&CombatAction::SelectCard{instance:999}).is_err());assert_eq!(c,before);
    c.step_full(&CombatAction::DeselectCard{instance:21}).unwrap();c.step_full(&CombatAction::SelectCard{instance:21}).unwrap();
    assert_eq!(c.public_full().unwrap().selection.unwrap().selected.iter().map(|id|id.0).collect::<Vec<_>>(),vec![20,21]);
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    // These two Weaves start in draw, so P sees the newly selected cards.
    // Genuine O/P duplicates are covered by m21_watcher_review, with Weaves
    // already in discard before Scry opens.
    assert_eq!(c.public_full().unwrap().hand.iter().filter(|c|c.id==crate::content::CardId::Weave).map(|c|c.instance.0).collect::<Vec<_>>(),vec![20,21]);
}
#[test]
fn m21_watcher_scry_empty_draw_notifies_nirvana_without_dexterity_or_shuffle(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::ThirdEye,card.instance,0));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    for (name,amount) in [("nirvana",3),("dexterity",2)]{c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId(name),state:PowerState{amount,skip_first:false,monster_source_grace:false}});}
    let discard=permanent_live_card(crate::content::CardId::Weave,CardInstanceId(20),0);c.state.card_ids_issued.insert(card_state::handle(discard.instance));c.state.piles.discard.push(model::OwnedCard::Full(discard));
    let random=c.state.rng.clone();c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.player.block,12);assert_eq!(c.state.rng,random);assert!(c.state.piles.draw.is_empty());assert!(c.public_full().unwrap().selection.is_none());
    assert!(!c.state.piles.hand.iter().any(|c|c.handle().as_str()=="20"));
}
#[test]
fn m21_watcher_scry_fault_retains_first_selected_movement_without_replay(){
    use cards::watcher::WatcherEffect;
    let mut c=choice();for instance in [20,21]{c.step_full(&CombatAction::SelectCard{instance}).unwrap();}
    // Stop at the real selection continuation, using the same accepted snapshot
    // and shared typed movement owner; the normal API cannot resume a fault.
    let selection=c.state.pending_selection.take().unwrap();c.state.phase=c.state.watcher.resume_phase.take().unwrap();
    c.state.pending.push_front(Effect{source:Source::Card(CardHandle::parse("3").unwrap()),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::CommitSelection{selection,next:0})});c.state.resolved_effects=99998;
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert!(c.state.piles.discard.iter().any(|c|c.handle().as_str()=="20"));assert!(c.state.piles.draw.iter().any(|c|c.handle().as_str()=="21"));
    assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::CommitSelection{next:1,..}))));
    let before=c.clone();assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());assert_eq!(c,before);
}

fn meditate_time_join(selected:bool)->Combat{
    use crate::content::{CardId,Character,EncounterId};
    let mut e=super::m8_shapes::entry(EncounterId::TimeEater);e.setup.character=Character::Watcher;e.opening_draw=0;
    e.cards=(1..=if selected{3}else{1}).map(|id|m21_watcher_live_card(if id==1{CardId::Meditate}else{CardId::WindmillStrike},CardInstanceId(id),0).unwrap()).collect();
    e.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;
    e.setup.enemies[0].intent=FullIntentId::from_name("time_ripple");
    let mut items=Some(crate::items::CombatItemContext::new(vec![],vec![None;3]).with_startup_profile(crate::items::CombatStartupProfile::M21Watcher));
    let mut c=Combat::enter_with_item_context(e,&mut items).unwrap();let cards=std::mem::take(&mut c.state.piles.draw);
    for card in cards{if card.handle().as_str()=="1"{c.state.piles.hand.push(card)}else{c.state.piles.discard.push(card)}}c
}
#[test]
fn m21_review_i2_meditate_timewarp11_empty_discard_finishes_once(){meditate_time_assert(false)}
#[test]
fn m21_review_i2_meditate_timewarp11_selected_return_finishes_once(){meditate_time_assert(true)}
fn meditate_time_assert(selected:bool){
    let mut c=meditate_time_join(selected);assert!(engine::validate_runtime(&c.state).is_ok());
    let start=c.state.card_ids_issued.clone();
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    if selected{
        assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"1");assert_eq!(c.state.turn,1);
        c.step_full(&CombatAction::SelectCard{instance:3}).unwrap();c.step_full(&CombatAction::ConfirmSelection).unwrap();
        let returned=piles::owned_full_cards(&c.state).find(|c|c.instance.0==3).unwrap();assert_eq!(returned.local.base_damage,Some(11));
    }
    assert_eq!(c.state.character.stance,Stance::Calm);assert!(c.fault_record().is_none());
    assert_eq!((c.state.turn,super::m5_elites::amount(&c,0,"time_warp"),super::m5_elites::amount(&c,0,"strength")),(2,0,2));
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardPlayed&&e.source==Source::Card(CardHandle::parse("1").unwrap())).count(),1);
    assert!(c.state.in_play.is_none()&&c.state.pending_time_warp.is_none()&&c.state.pending.is_empty());
    assert_eq!(c.state.card_ids_issued,start);
    assert_eq!(piles::owned_full_cards(&c.state).map(|c|card_state::handle(c.instance)).collect::<std::collections::BTreeSet<_>>(),start);
    assert!(engine::validate_runtime(&c.state).is_ok());assert!(c.public_full().is_ok());assert!(c.exit().is_err(),"exit requires a terminal");
    let frozen=c.clone();assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());assert_eq!(c,frozen);
    let target=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(999))}],None).unwrap();
    assert_eq!(c.state.outcome,Some(Outcome::Victory));let exit=c.exit().unwrap();
    assert_eq!(exit.hand.len()+exit.draw.len()+exit.discard.len()+exit.exhaust.len(),start.len());
}
