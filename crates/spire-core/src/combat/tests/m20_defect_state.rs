use super::super::*;
use crate::content::{CardId,Character};
use model::OwnedCard;

fn fight()->Combat {
    let mut c=super::full_runtime::entered_combat(9,9,|card|*card=permanent_live_card(CardId::Defragment,card.instance,0));
    c.state.silent.source_character=Some(Character::Defect);
    c.state.character=characters::CharacterState::for_character(Character::Defect);
    for owned in &mut c.state.piles.hand { let instance=owned.full().unwrap().instance;*owned=OwnedCard::Full(permanent_live_card(CardId::Defragment,instance,0)); }
    c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c
}

#[test]
fn m20_defect_state_actual_draw_history_retains_physical_identity_and_root_reset(){
    let mut c=fight();
    c.state.silent.history_enabled=true;
    for n in [100,101] {
        let card=permanent_live_card(CardId::DefendBlue,CardInstanceId(n),0);
        assert!(c.state.card_ids_issued.insert(card_state::handle(card.instance)));
        c.state.piles.draw.push(OwnedCard::Full(card));
    }
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::Draw(2)}],None).unwrap();
    assert_eq!(c.state.silent.drawn_cards.iter().map(|entry|entry.instance).collect::<Vec<_>>(),vec![Some(CardInstanceId(101)),Some(CardInstanceId(100))]);
    assert_eq!(format!("{:?}",c.state.silent.drawn_cards),"[DefendBlue, DefendBlue]","old ID-only Debug remains frozen");
    let moved=c.state.piles.hand.iter().position(|card|card.handle().as_str()=="101").unwrap();
    let moved=c.state.piles.hand.remove(moved);c.state.piles.discard.push(moved);
    assert_eq!(c.state.silent.drawn_cards[0].instance,Some(CardInstanceId(101)),"movement does not rewrite actual draw references");
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::Draw(0)}],None).unwrap();
    assert!(c.state.silent.drawn_cards.is_empty(),"even root zero draw clears the shared history");
}

#[test]
fn m20_defect_state_combat_power_history_counts_uses_after_retirement_not_installed_stacks(){
    let mut c=fight();
    assert_eq!(c.state.silent.powers_played_this_combat,0);
    for instance in [1,2,3] {
        c.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
    }
    assert_eq!(c.state.silent.powers_played_this_combat,3);
    assert_eq!(c.state.played_powers.len(),3);
    assert_eq!(c.state.player_full_powers.iter().filter(|power|power.id.0=="focus").count(),0,"Focus uses the slot owner rather than an installed Power row");
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.silent.powers_played_this_combat,3,"turn reset keeps combat history");
    assert_eq!(fight().state.silent.powers_played_this_combat,0,"new combat resets it");
}

#[test]
fn m20_defect_state_successful_channel_history_counts_installation_once(){
    let mut c=fight();
    let source=Source::Actor(Actor::Player);
    c.state.character.orb_slots=vec![None];
    characters::channel(&mut c.state,&source,OrbKind::Frost.into(),true).unwrap();
    assert_eq!(c.state.defect.frost_channeled,1);
    characters::channel(&mut c.state,&source,OrbKind::Lightning.into(),true).unwrap();
    assert_eq!(c.state.defect.lightning_channeled,0,"full-slot request counts only after actual eviction retry");
    engine::resolve(&mut c.state);
    assert_eq!(c.state.defect.lightning_channeled,1);
    assert_eq!(c.state.defect.frost_channeled,1);
    characters::apply(&mut c.state,&source,CharacterEffect::EndTurn).unwrap();
    engine::resolve(&mut c.state);
    assert_eq!(c.state.defect.lightning_channeled,1,"passive invocation is not channeling");
    c.state.character.orb_slots.clear();
    characters::channel(&mut c.state,&source,OrbKind::Frost.into(),true).unwrap();
    assert_eq!(c.state.defect.frost_channeled,1,"zero-capacity request never installs");
    assert_eq!((fight().state.defect.frost_channeled,fight().state.defect.lightning_channeled),(0,0));
}

fn card_fight(id:CardId,up:u32)->Combat{
    let mut c=super::full_runtime::entered_combat(9,9,|card|*card=permanent_live_card(id,card.instance,up));
    c.state.character=characters::CharacterState::for_character(Character::Defect);c.state.silent.source_character=Some(Character::Defect);
    c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c
}
fn add_card(c:&mut Combat,id:CardId,n:u64,zone:u8){
    let card=permanent_live_card(id,CardInstanceId(n),0);assert!(c.state.card_ids_issued.insert(card_state::handle(card.instance)));
    let pile=match zone{0=>&mut c.state.piles.hand,1=>&mut c.state.piles.draw,_=>&mut c.state.piles.discard};pile.push(OwnedCard::Full(card));
}
#[test]
fn m20_defect_state_history_prices_existing_live_and_fresh_forcefield(){
    let mut c=fight();add_card(&mut c,CardId::ForceField,100,0);add_card(&mut c,CardId::ForceField,101,2);
    for instance in [1,2,3]{c.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();}
    assert_eq!(c.state.silent.powers_played_this_combat,3);
    for card in c.state.piles.hand.iter().chain(&c.state.piles.discard).filter_map(OwnedCard::full).filter(|card|card.id==CardId::ForceField){
        assert_eq!(card_state::effective_cost(card,4).unwrap(),1,"notified ordinary piles discount each Power once");
    }
    let fresh=card_state::generated_with_history(CardId::ForceField,CardInstanceId(200),1,card_state::CardConstructionHistory::from_state(&c.state)).unwrap();
    assert_eq!(card_state::effective_cost(&fresh,4).unwrap(),1,"fresh constructor uses combat history including retired Powers");
}
#[test]
fn m20_defect_state_x_zero_positive_upgrade_chemical_and_no_front(){
    for up in 0..=1{for energy in [0,2]{for chemical in [false,true]{
        for id in [CardId::ReinforcedBody,CardId::Tempest,CardId::MultiCast]{
            let mut c=card_fight(id,up);c.state.player.energy=energy;
            if chemical{c.state.item_context=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::ChemicalX],vec![]));}
            if id==CardId::MultiCast{c.state.character.orb_slots[0]=Some(OrbKind::Frost.into());}
            c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
            let count=energy+if chemical{2}else{0}+i32::from(up>0&&id!=CardId::ReinforcedBody);
            if id==CardId::ReinforcedBody{assert_eq!(c.state.player.block,count*if up>0{9}else{7});}
            if id==CardId::Tempest{assert_eq!(c.state.defect.lightning_channeled,count as u32);}
            if id==CardId::MultiCast{assert_eq!(c.state.player.block,count*5);assert_eq!(c.state.character.orb_slots.iter().flatten().count(),usize::from(count==0));}
            assert_eq!(c.state.player.energy,0);
        }
    }}}
    let mut c=card_fight(CardId::MultiCast,1);c.state.player.energy=2;
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert_eq!(c.state.player.energy,2,"no front means no late payment");
}
#[test]
fn m20_defect_state_moves_are_not_draws_and_full_hand_destinations_differ(){
    for id in [CardId::Hologram,CardId::Seek]{
        let mut c=card_fight(id,0);add_card(&mut c,CardId::Void,100,if id==CardId::Seek{1}else{2});
        let energy=c.state.player.energy;
        c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(c.state.player.energy,energy-if id==CardId::Hologram{1}else{0},"Void movement does not lose draw energy");
        assert!(c.state.piles.hand.iter().any(|card|card.handle().as_str()=="100"));
    }
    let mut c=card_fight(CardId::Seek,1);add_card(&mut c,CardId::Void,100,1);add_card(&mut c,CardId::DefendBlue,101,1);add_card(&mut c,CardId::Zap,102,1);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let selection=c.public_full().unwrap().selection.unwrap();assert_eq!(selection.operation,SelectionOperation::SeekRetrieve);
    c.step_full(&CombatAction::SelectCard{instance:100}).unwrap();c.step_full(&CombatAction::SelectCard{instance:102}).unwrap();
    assert_eq!(c.state.piles.hand.iter().rev().take(2).map(OwnedCard::handle).map(|h|h.as_str().to_owned()).collect::<Vec<_>>(),vec!["102","100"]);
}

#[test]
fn m20_defect_state_ftl_tests_current_history_at_followup_and_draw_precedes_damage(){
    for up in 0..=1{for prior in [2usize,3,4]{
        let mut c=card_fight(CardId::FTL,up);c.state.silent.history_enabled=true;c.state.silent.played_this_turn=vec![CardId::StrikeBlue;prior];add_card(&mut c,CardId::DefendBlue,100,1);
        c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!(c.state.piles.hand.iter().any(|card|card.handle().as_str()=="100"),prior<if up>0{4}else{3});
        assert_eq!(c.state.enemies[0].hp,500-if up>0{6}else{5});
        if prior<if up>0{4}else{3}{
            let draw=c.state.hook_events.iter().position(|event|event.hook==Hook::CardDrawn).unwrap();let after=c.state.hook_events.iter().position(|event|event.hook==Hook::CardAfterUse).unwrap();assert!(draw<after);
        }
    }}
}
#[test]
fn m20_defect_state_x_capture_and_late_payment_are_separate_checkpoints(){
    for (id,free,front) in [(CardId::ReinforcedBody,false,false),(CardId::Tempest,false,false),(CardId::MultiCast,false,true),(CardId::ReinforcedBody,true,false),(CardId::MultiCast,false,false)]{
        let mut c=card_fight(id,0);c.state.player.energy=2;
        if free{c.state.piles.hand.iter_mut().find(|card|card.handle().as_str()=="3").unwrap().full_mut().unwrap().local.free_to_play_once=true;}
        if front{c.state.character.orb_slots[0]=Some(OrbKind::Frost.into());}
        let p=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert_eq!(p.remaining_energy,2,"preparation does not spend blue X");
        let card=c.state.piles.hand.remove(p.hand_index);c.state.in_play=Some(card);c.state.player.energy=7;
        effects::apply_effect(&mut c.state,&p.effects[0]).unwrap();
        assert_eq!(c.state.player.energy,if free||id==CardId::MultiCast&&!front{7}else{0},"positive action spends then-current7, using captured2 count");
        assert_eq!(c.state.pending.len(),if id==CardId::MultiCast&&!front{0}else{2});
    }
    let mut c=card_fight(CardId::ReinforcedBody,0);c.state.player.energy=2;c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("echo_form"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert_eq!(c.state.player.block,28,"actual repeat keeps captured X2 and gains2 more times");assert_eq!(c.state.player.energy,0);assert!(c.state.play_frames.is_empty());
}
#[test]
fn m20_defect_state_recycle_reads_x_or_turn_cost_before_exhaust_and_free_waiver_is_distinct(){
    for (id,cost,free,want) in [(CardId::Tempest,None,false,6),(CardId::Glacier,Some(0),false,3),(CardId::Glacier,Some(2),true,5),(CardId::Void,None,false,3)]{
        let mut c=card_fight(CardId::Recycle,0);c.state.player.energy=4;add_card(&mut c,id,100,0);
        let card=c.state.piles.hand.last_mut().unwrap().full_mut().unwrap();card.local.turn_cost=cost;card.local.free_to_play_once=free;
        c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert_eq!(c.public_full().unwrap().selection.unwrap().operation,SelectionOperation::Recycle);
        c.step_full(&CombatAction::SelectCard{instance:100}).unwrap();assert_eq!(c.state.player.energy,want);assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="100"));
        assert_eq!(c.state.silent.discarded_this_turn,0);
    }
}
#[test]
fn m20_defect_state_all_for_one_uses_combat_cost_scrape_uses_physical_draw_turn_cost(){
    let mut c=card_fight(CardId::AllForOne,0);
    for (n,combat,turn,free) in [(100,1,0,false),(101,0,1,false),(102,2,2,true)]{
        add_card(&mut c,CardId::DefendBlue,n,2);let card=c.state.piles.discard.last_mut().unwrap().full_mut().unwrap();card.local.mutable_combat_cost=Some(combat);card.local.turn_cost=Some(turn);card.local.free_to_play_once=free;
    }
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="100"));assert!(c.state.piles.hand.iter().any(|card|card.handle().as_str()=="101"));assert!(c.state.piles.hand.iter().any(|card|card.handle().as_str()=="102"));
    let mut c=card_fight(CardId::Scrape,0);c.state.silent.history_enabled=true;
    for (n,turn,free) in [(100,1,false),(101,0,false),(102,2,true),(103,1,false)]{
        add_card(&mut c,CardId::DefendBlue,n,1);let card=c.state.piles.draw.last_mut().unwrap().full_mut().unwrap();card.local.turn_cost=Some(turn);card.local.free_to_play_once=free;
    }
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    for n in [100,103]{assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()==n.to_string()));}
    for n in [101,102]{assert!(c.state.piles.hand.iter().any(|card|card.handle().as_str()==n.to_string()));}
    assert_eq!(c.state.silent.drawn_cards.iter().filter_map(|entry|entry.instance).collect::<Vec<_>>(),vec![CardInstanceId(103),CardInstanceId(102),CardInstanceId(101),CardInstanceId(100)]);
    assert_eq!(c.state.silent.discarded_this_turn,2);
}
#[test]
fn m20_defect_state_reboot_excludes_retired_and_exhausted_cards_and_fault_keeps_transfers(){
    let mut c=card_fight(CardId::Reboot,0);add_card(&mut c,CardId::DefendBlue,100,2);add_card(&mut c,CardId::Void,101,2);
    let excluded=permanent_live_card(CardId::Dazed,CardInstanceId(102),0);assert!(c.state.card_ids_issued.insert(card_state::handle(excluded.instance)));c.state.piles.exhaust.push(OwnedCard::Full(excluded));
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();assert_eq!(c.state.silent.drawn_cards.len(),4);assert_eq!(c.state.piles.exhaust.len(),2);assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="102"));
    let mut c=card_fight(CardId::Reboot,0);add_card(&mut c,CardId::DefendBlue,100,2);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1u128<<64).unwrap());
    assert!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}).is_err());assert!(c.state.piles.discard.is_empty(),"accepted discard transfer precedes hand RNG fault");assert!(c.state.piles.draw.iter().any(|card|card.handle().as_str()=="100"));assert_eq!(c.state.piles.hand.len(),2);assert_eq!(c.fault_record().unwrap().kind,FaultKind::RngExhausted);let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
}

#[test]
fn m20_defect_state_equilibrium_and_rebound_share_ordinary_finish_and_cleanup(){
    let mut c=card_fight(CardId::Equilibrium,0);add_card(&mut c,CardId::Void,100,0);add_card(&mut c,CardId::DefendBlue,101,0);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="100"));assert!(c.state.piles.hand.iter().any(|card|card.handle().as_str()=="101"));assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="equilibrium"));assert!(engine::validate_runtime(&c.state).is_ok());
    for next in [CardId::DefendBlue,CardId::GeneticAlgorithm,CardId::Buffer]{
        let mut c=card_fight(CardId::Rebound,0);add_card(&mut c,next,100,0);
        c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="3"),"just-installed guard leaves Rebound itself ordinary");
        c.step_full(&CombatAction::PlayCard{instance:100,target:None}).unwrap();assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="rebound"));
        match next{
            CardId::DefendBlue=>assert_eq!(c.state.piles.draw.last().unwrap().handle().as_str(),"100"),
            CardId::GeneticAlgorithm=>assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="100")),
            CardId::Buffer=>assert!(c.state.played_powers.iter().any(|card|card.handle().as_str()=="100")),_=>unreachable!(),
        }
        assert!(engine::validate_runtime(&c.state).is_ok());
    }
}
#[test]
fn m20_defect_state_retrieval_full_hand_source_destinations_and_selected_order(){
    for id in [CardId::Hologram,CardId::Seek]{
        let mut c=card_fight(id,1);for n in 100..107{add_card(&mut c,CardId::DefendBlue,n,0)}
        for n in 200..203{add_card(&mut c,CardId::Void,n,if id==CardId::Seek{1}else{2})}
        assert_eq!(c.state.piles.hand.len(),10);let drawn=c.state.silent.drawn_cards.clone();
        c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        if c.state.phase==Phase::Selection{
            c.step_full(&CombatAction::SelectCard{instance:200}).unwrap();if id==CardId::Seek{c.step_full(&CombatAction::SelectCard{instance:201}).unwrap();}
        }
        assert_eq!(c.state.piles.hand.len(),10);
        if id==CardId::Seek{assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="201"),"Seek's extra chosen output goes to discard at cap");}
        assert_eq!(c.state.silent.drawn_cards,drawn,"retrieval never becomes an on-draw callback or rewrites opening history");
    }
}
#[test]
fn m20_defect_state_chemical_x_is_a_real_diagnostic_startup_only_resource(){
    use crate::{items::{CombatItemContext,NativeBattleContext},content::RelicId};
    let mut e=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);e.setup.character=Character::Defect;e.setup.player.energy=0;e.setup.player.max_energy=3;
    e.cards=vec![permanent_live_card(CardId::ReinforcedBody,CardInstanceId(1),0)];e.opening_draw=1;
    let context=CombatItemContext::new(vec![RelicId::ChemicalX],vec![None;3]);
    let mut old=Some(context.clone());let original=old.clone();assert!(Combat::enter_with_item_context(e.clone(),&mut old).is_err());assert_eq!(old,original,"no old-profile constructor admission expansion");
    let mut m20=Some(context.with_battle_context(NativeBattleContext{m20:true,..Default::default()}));
    let mut c=Combat::enter_with_item_context(e,&mut m20).expect("approved core-only ChemicalX joins real M20 startup");c.state.player.energy=0;c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();assert_eq!(c.state.player.block,14);
    assert!(!crate::run::M20_DEFECT_HEART_RELICS.contains(&RelicId::ChemicalX));
}
#[test]
fn m20_defect_state_reboot_last_singleton_random_move_consumes_and_retains_first_transfer(){
    let mut c=card_fight(CardId::Reboot,0);add_card(&mut c,CardId::DefendBlue,100,2);
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),(1u128<<64)-1).unwrap());
    assert!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}).is_err(),"source getRandomCard(random) also calls random(size-1) for final singleton");
    assert_eq!(c.fault_record().unwrap().kind,FaultKind::RngExhausted);assert_eq!(c.state.piles.hand.len(),1);assert_eq!(c.state.piles.draw.len(),2);assert!(c.state.piles.discard.is_empty());assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1u128<<64);
    let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
}

#[test]
fn m20_defect_state_stat_copy_does_not_copy_a_use_action_rebound_flag(){
    let mut source=permanent_live_card(CardId::GeneticAlgorithm,CardInstanceId(1),1);source.misc=19;source.local.rebound_on_finish=true;
    let copy=source.stat_equivalent_copy(CardInstanceId(2));assert_eq!(copy.misc,19);assert_eq!(copy.upgrades,1);assert!(!copy.local.rebound_on_finish);assert!(source.local.rebound_on_finish);assert!(copy.local.exhaust_on_finish);
}
