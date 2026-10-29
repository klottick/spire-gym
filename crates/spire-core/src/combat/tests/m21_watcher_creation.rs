use super::super::*;
use crate::{content::{CardId,PotionId,RelicId,Character},items::{CombatItemContext,CombatStartupProfile}};
fn fight(card:CardId,potions:Vec<Option<PotionId>>,relics:Vec<RelicId>)->Combat{
    let mut c=super::full_runtime::entered_combat(3,3,|c|*c=permanent_live_card(card,c.instance,0));
    c.state.silent.source_character=Some(Character::Watcher);
    c.state.item_context=Some(CombatItemContext::new(relics,potions).with_startup_profile(CombatStartupProfile::M21Watcher));c
}
#[test]
fn m21_watcher_creation_carve_delivers_a_fresh_base_smite() {
    let mut c=fight(CardId::CarveReality,vec![None;3],vec![]);
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    let child=c.public_full().unwrap().hand.into_iter().find(|c|c.id==CardId::Smite).unwrap();
    assert_eq!(child.upgrades,0);assert_ne!(child.instance.0,3);
    assert_eq!(child.origin,CombatCardRef::CombatLocal(child.instance.0));
    assert!(child.values.unwrap().exhaust_on_finish);
}
#[test]
fn m21_watcher_potion_stance_is_a_real_nonphysical_choice() {
    let mut c=fight(CardId::DefendPurple,vec![Some(PotionId::StancePotion),None,None],vec![RelicId::SacredBark]);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.state.item_context.as_ref().unwrap().potions()[0],None);
    assert_eq!(c.legal_actions_full().unwrap().iter().map(|a|format!("{a:?}")).collect::<Vec<_>>(),vec!["ChooseStance { stance: Wrath }","ChooseStance { stance: Calm }"]);
}
#[test]
fn m21_watcher_potion_bottled_miracle_uses_bark_count_once() {
    let mut c=fight(CardId::DefendPurple,vec![Some(PotionId::BottledMiracle),None,None],vec![RelicId::SacredBark]);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.public_full().unwrap().hand.iter().filter(|c|c.id==CardId::Miracle).count(),4);
    assert_eq!(c.state.item_context.as_ref().unwrap().potions()[0],None);
}
#[test]
fn m21_watcher_potion_stance_rejects_unoffered_and_resumes_without_reconsumption(){
    let mut c=fight(CardId::DefendPurple,vec![Some(PotionId::StancePotion),None,None],vec![]);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let before=c.clone();assert!(c.step_full(&CombatAction::ChooseStance{stance:Stance::Divinity}).is_err());assert_eq!(c,before);
    assert_eq!(c.public_full().unwrap().stance_choice.unwrap().options,vec![Stance::Wrath,Stance::Calm]);
    c.step_full(&CombatAction::ChooseStance{stance:Stance::Calm}).unwrap();
    assert_eq!(c.state.character.stance,Stance::Calm);assert!(c.public_full().unwrap().stance_choice.is_none());
    let before=c.clone();assert!(c.step_full(&CombatAction::ChooseStance{stance:Stance::Wrath}).is_err());assert_eq!(c,before);
}
#[test]
fn m21_watcher_potion_ambrosia_metadata_is_not_transition_count(){
    for bark in [false,true]{
        let mut c=fight(CardId::DefendPurple,vec![Some(PotionId::Ambrosia),Some(PotionId::Ambrosia),None],if bark{vec![RelicId::SacredBark]}else{vec![]});
        assert_eq!(c.state.item_context.as_ref().unwrap().potion_potency(0),Some(if bark{4}else{2}));
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Divinity,6));
        c.step_full(&CombatAction::UsePotion{slot:1,target:None}).unwrap();assert_eq!(c.state.player.energy,6);
    }
}
#[test]
fn m21_watcher_potion_energy_uses_capture_and_source_cap(){
    let mut c=fight(CardId::DefendPurple,vec![Some(PotionId::EnergyPotion),None,None],vec![RelicId::SacredBark]);c.state.player.energy=998;
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(c.state.player.energy,999);
}
#[test]
fn m21_watcher_creation_master_reality_early_and_route_specific_late_identity(){
    use cards::watcher::{create_action,WatcherEffect};
    for early in [false,true]{for late in [false,true]{for overflow in [false,true]{
        let mut c=fight(CardId::DefendPurple,vec![None;3],vec![]);
        let power=FullPowerEntry{id:FullPowerId("master_reality"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}};
        if early{c.state.player_full_powers.push(power.clone());}
        let kind=create_action(&c.state,CardId::Smite,1).unwrap();
        c.state.player_full_powers.retain(|p|p.id.0!="master_reality");if late{c.state.player_full_powers.push(power);}
        if overflow{for id in 20..29{let card=m21_watcher_live_card(CardId::DefendPurple,CardInstanceId(id),0).unwrap();c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));}}
        c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
        assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault,"{:?}",c.fault_record());
        let child=c.state.piles.all(c.state.in_play.as_ref()).find_map(|c|c.full().filter(|c|c.id==CardId::Smite)).unwrap();
        assert_eq!(child.upgrades,u32::from(early||late&&!overflow));
        assert_eq!(child.local.base_damage,Some(if early||late&&!overflow{16}else{12}));
        assert!(!matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::Create{..}))));
    }}}
}
#[test]
fn m21_watcher_potion_energy_rejects_wrong_slot_amount_and_unowned_source(){
    use cards::watcher::WatcherEffect;
    let c=fight(CardId::DefendPurple,vec![Some(PotionId::EnergyPotion),None,None],vec![]);
    for (source,slot,amount) in [(Source::Actor(Actor::Player),0,2),(Source::Potion{slot:1,id:PotionId::EnergyPotion},1,2),(Source::Potion{slot:0,id:PotionId::EnergyPotion},0,4)]{
        let bad=Effect{source,target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::PotionEnergy{slot,amount})};
        assert!(c.preflight_native_effects(&[bad]).is_err());
    }
}
#[test]
fn m21_watcher_potion_generated_miracle_uses_source_gain_cap(){
    for up in 0..=1{
        let mut c=fight(CardId::DefendPurple,vec![None;3],vec![]);
        let card=m21_watcher_live_card(CardId::Miracle,CardInstanceId(20),up).unwrap();c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));
        c.state.player.energy=999;c.step_full(&CombatAction::PlayCard{instance:20,target:None}).unwrap();assert_eq!(c.state.player.energy,999);
        assert!(c.state.piles.exhaust.iter().any(|c|c.handle().as_str()=="20"));
    }
}
#[test]
fn m21_watcher_creation_fault_retains_issued_child_and_exact_next_cursor(){
    use cards::watcher::{create_action,WatcherEffect};
    let mut c=fight(CardId::DefendPurple,vec![None;3],vec![]);let kind=create_action(&c.state,CardId::Miracle,4).unwrap();
    c.state.resolved_effects=99998;c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert_eq!(c.state.piles.hand.iter().filter(|c|c.full().is_some_and(|c|c.id==CardId::Miracle)).count(),1);
    assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(WatcherEffect::Create{count:4,next:1,..}))));
    let before=c.clone();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert!(c.public_full().is_err());assert_eq!(c,before);
}
#[test]
fn m21_watcher_creation_hymn_uses_actual_later_turn_and_installed_master_reality(){
    let mut c=fight(CardId::BattleHymn,vec![None;3],vec![]);let card=m21_watcher_live_card(CardId::MasterReality,CardInstanceId(20),0).unwrap();c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));
    c.step_full(&CombatAction::PlayCard{instance:20,target:None}).unwrap();c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert!(!c.state.piles.hand.iter().any(|c|c.full().is_some_and(|c|c.id==CardId::Smite)));
    c.step_full(&CombatAction::EndTurn).unwrap();let child=c.public_full().unwrap().hand.into_iter().find(|c|c.id==CardId::Smite).unwrap();assert_eq!(child.upgrades,1);assert_eq!(child.values.unwrap().base_damage,Some(16));
}
#[test]
fn m21_watcher_creation_parent_upgrade_does_not_upgrade_child(){
    for (id,target,child) in [(CardId::CarveReality,Some(TargetId(1)),CardId::Smite),(CardId::DeceiveReality,None,CardId::Safety)]{let mut c=fight(CardId::DefendPurple,vec![None;3],vec![]);let parent=c.state.piles.hand.iter_mut().find_map(|c|c.full_mut().filter(|c|c.instance.0==3)).unwrap();*parent=m21_watcher_live_card(id,parent.instance,1).unwrap();c.step_full(&CombatAction::PlayCard{instance:3,target}).unwrap();assert_eq!(c.public_full().unwrap().hand.iter().find(|c|c.id==child).unwrap().upgrades,0);}
}
#[test]
fn m21_watcher_potion_choice_and_creation_reject_wrong_owned_payloads(){
    use cards::watcher::{WatcherEffect,create_action};
    let c=fight(CardId::DefendPurple,vec![Some(PotionId::StancePotion),None,None],vec![]);
    let bad=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::OpenStanceChoice)};assert!(c.preflight_native_effects(&[bad]).is_err());
    let mut kind=create_action(&c.state,CardId::Smite,1).unwrap();if let EffectKind::Watcher(WatcherEffect::Create{next,..})=&mut kind{*next=2;}
    assert!(c.preflight_native_effects(&[Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind}]).is_err());
}

// Supplied owner-list composition: source equal-priority powers retain installed
// order, bottom producers precede normal draw, Foresight's recycle is top work.
fn start_power_join(hymn_first:bool, empty_draw:bool)->Combat {
    let mut c=fight(CardId::DefendPurple,vec![None;3],vec![]);
    let cards=std::mem::take(&mut c.state.piles.hand);
    if empty_draw{c.state.piles.discard.extend(cards)}else{c.state.piles.draw.extend(cards)}
    c.state.turn=2;c.state.phase=Phase::PlayerStart;c.state.player.energy=0;
    for id in if hymn_first{["battle_hymn","foresight"]}else{["foresight","battle_hymn"]}{
        c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId(id),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    }
    c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    hooks::dispatch(&mut c.state,Hook::TurnStartedPreDraw,&Source::Actor(Actor::Player),Some(&Actor::Player)).unwrap();c
}
#[test]
fn m21_review_i1_both_installed_orders_publish_the_actual_hymn_child_before_scry(){
    for empty in [false,true]{for hymn_first in [true,false]{
        let mut c=start_power_join(hymn_first,empty);
        assert_eq!(c.state.player.energy,3,"recharge is synchronous before queued callbacks run");
        if empty{assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::Recycle,"source addToTop beats all bottom producers");}
        assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault,"{:?}",c.fault_record());
        let public=c.public_full().unwrap();assert!(public.selection.is_some());
        let children=public.hand.iter().filter(|c|c.id==CardId::Smite).collect::<Vec<_>>();
        assert_eq!(children.len(),usize::from(hymn_first),"installed order, empty={empty}, hymn_first={hymn_first}");
        if hymn_first{assert_eq!(children[0].origin,CombatCardRef::CombatLocal(children[0].instance.0));assert!(c.state.card_ids_issued.contains(&card_state::handle(children[0].instance)));}
        assert_eq!(public.draw_count,3,"normal draw waits behind the modal");
        c.step_full(&CombatAction::ConfirmSelection).unwrap();assert!(c.fault_record().is_none());
        assert_eq!(c.public_full().unwrap().hand.iter().filter(|c|c.id==CardId::Smite).count(),1);
    }}
}
#[test]
fn m21_review_i1_hymn_template_captures_installed_mr_at_callback_construction(){
    use cards::watcher::WatcherEffect;
    let mut c=start_power_join(true,false);
    // Installation after callback construction cannot mutate its early template.
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("master_reality"),state:PowerState{amount:-1,skip_first:false,monster_source_grace:false}});
    let captured=c.state.pending.iter().find_map(|e|match &e.kind{EffectKind::Watcher(WatcherEffect::Create{template,..})=>Some(template),_=>None}).unwrap();
    assert_eq!(captured.upgrades,0);
    assert_ne!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert_eq!(c.public_full().unwrap().hand.iter().find(|c|c.id==CardId::Smite).unwrap().upgrades,1,"actual hand child receives late upgrade");
}
