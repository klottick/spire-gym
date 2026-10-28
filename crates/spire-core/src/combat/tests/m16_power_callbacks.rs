use super::super::*;
use super::full_runtime::entered_combat;
fn power(id:&'static str,amount:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}}
#[test]
fn m16_power_callbacks_notify_repeat_and_expire(){
    use crate::content::CardId::*;
    for (charges,expected) in [(1,2),(2,2)] {
        let mut combat=entered_combat(3,3, |_|{});
        combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
        combat.state.player_full_powers.extend([power("storm",1),power("amplify",charges)]);
        let instance=combat.state.piles.hand[0].full().unwrap().instance;
        *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(Defragment,instance,0);
        combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
        assert_eq!(combat.state.character.focus,expected,"original + one actual purge repeat");
        assert_eq!(combat.state.player.energy,2);
        assert_eq!(combat.state.character.orb_slots.iter().flatten().count(),2,"both notified Powers trigger Storm");
        assert_eq!(combat.state.played_powers.len(),1,"real original Power retires once");
        assert_eq!(combat.state.inactive_cards.len(),1,"repeat is purged");
        assert!(combat.state.play_frames.is_empty());
        assert!(combat.state.queued_autoplay.is_empty());
    }
}
#[test]
fn m16_power_callbacks_echo_ethereal_flags(){
    assert!(permanent_live_card(crate::content::CardId::EchoForm,CardInstanceId(1),0).local.ethereal);
    assert!(!permanent_live_card(crate::content::CardId::EchoForm,CardInstanceId(1),1).local.ethereal);
}

#[test]
fn m16_power_callbacks_combined_echo_amplify_purges_once_each(){
    use crate::content::CardId::*;
    let mut combat=entered_combat(3,3, |_|{});combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.player_full_powers.extend([power("storm",1),power("echo_form",1),power("amplify",1)]);
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(Defragment,instance,0);
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    assert_eq!(combat.state.character.focus,3);
    assert_eq!(combat.state.played_powers.len(),1);assert_eq!(combat.state.inactive_cards.len(),2);
    assert_eq!(combat.state.character.orb_slots.iter().flatten().count(),3);
    assert_eq!(combat.state.silent.played_this_turn,vec![Defragment;3]);
    assert_eq!(combat.state.defect.echo_doubled_this_turn,1);
    assert!(combat.state.play_frames.is_empty());assert!(combat.state.queued_autoplay.is_empty());
}

#[test]
fn m16_power_callbacks_full_hand_draw_large_heatsinks_is_not_a_fault(){
    let mut combat=entered_combat(3,3, |_|{});combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.silent.source_character=Some(crate::content::Character::Defect);
    combat.state.player_full_powers.push(power("heatsinks",20));
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(crate::content::CardId::Defragment,instance,0);
    for n in 10..17{super::super::piles::insert_generated(&mut combat.state,permanent_live_card(crate::content::CardId::DefendBlue,CardInstanceId(n),0).copy_as(CardInstanceId(n)),CardDestination::Hand).unwrap();}
    for n in 20..30{super::super::piles::insert_generated(&mut combat.state,permanent_live_card(crate::content::CardId::DefendBlue,CardInstanceId(n),0).copy_as(CardInstanceId(n)),CardDestination::Draw).unwrap();}
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    assert_eq!(combat.state.piles.hand.len(),10);assert_eq!(combat.state.piles.draw.len(),9);
}

#[test]
fn m16_power_callbacks_fault_custody_and_terminal_cancel(){
    // Stage an accepted Power at the existing shared FinishCard checkpoint,
    // then force the dispatch guard precisely between original and repeat.
    let mut combat=entered_combat(3,3, |_|{});combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.player_full_powers.push(power("echo_form",1));
    let mut original=combat.state.piles.hand.remove(0);let instance=original.full().unwrap().instance;
    *original.full_mut().unwrap()=permanent_live_card(crate::content::CardId::Defragment,instance,0);
    let owner=original.handle();let source=Source::Card(owner.clone());combat.state.in_play=Some(original);
    combat.state.play_context=Some(super::super::model::PlayContext{card:owner,target:None,energy_on_use:3,paid:1,disposition:super::super::model::FinishDisposition::Natural});
    combat.state.phase=Phase::PlayerAction;combat.state.player.energy=2;
    combat.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::FinishCard});
    super::super::hooks::dispatch(&mut combat.state,Hook::CardPlayed,&source,None).unwrap();
    super::super::effects::apply_effect(&mut combat.state,&Effect{source,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("focus"),amount:1}}).unwrap();
    combat.state.resolved_effects=99_998;
    assert_eq!(super::super::engine::resolve(&mut combat.state).status,TransitionStatus::Fault);
    assert_eq!(combat.state.character.focus,1);assert_eq!(combat.state.player.energy,2);
    assert_eq!(combat.state.queued_autoplay.len(),1);assert_eq!(combat.state.play_frames.len(),1);
    assert!(combat.state.play_frames[0].parent_finish_checkpoint_done);
    super::super::engine::validate_runtime(&combat.state).unwrap();let frozen=combat.clone();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());assert_eq!(combat,frozen);
    let mut lethal=entered_combat(3,3, |_|{});lethal.state.character=CharacterState::for_character(crate::content::Character::Defect);
    lethal.state.player_full_powers.push(power("echo_form",1));lethal.state.enemies[0].hp=3;
    let instance=lethal.state.piles.hand[0].full().unwrap().instance;
    *lethal.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(crate::content::CardId::Claw,instance,0);
    lethal.step_full(&CombatAction::PlayCard{instance:instance.0,target:Some(TargetId(1))}).unwrap();
    assert_eq!(lethal.state.outcome,Some(Outcome::Victory));assert_eq!(lethal.state.queued_autoplay.len(),1,"terminal owner retains unstarted child custody");
    assert_eq!(lethal.state.silent.played_this_turn,vec![crate::content::CardId::Claw]);
    assert_eq!(lethal.exit().unwrap().queued_autoplay.len(),1);
    super::super::engine::validate_runtime(&lethal.state).unwrap();
}

#[test]
fn m16_power_callbacks_amplify_expiry_echo_turn_reset_and_free_repeat(){
    let mut combat=entered_combat(1,1, |_|{});combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.silent.source_character=Some(crate::content::Character::Defect);
    combat.state.player_full_powers.extend([power("echo_form",1),power("amplify",2)]);
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(crate::content::CardId::Defragment,instance,0);
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    assert_eq!(combat.state.player.energy,0);assert_eq!(combat.state.character.focus,3);
    assert_eq!(combat.state.player_full_powers.iter().find(|p|p.id.0=="amplify").unwrap().state.amount,1);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(!combat.state.player_full_powers.iter().any(|p|p.id.0=="amplify"));assert_eq!(combat.state.defect.echo_doubled_this_turn,0);
    assert!(combat.state.silent.played_this_turn.is_empty());
}

#[test]
fn m16_power_callbacks_terminal_census_rejects_forged_owners(){
    use crate::content::CardId;
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=permanent_live_card(CardId::Claw,CardInstanceId(1),0);
    let mut death=Combat::enter(entry).unwrap();death.state.character=CharacterState::for_character(crate::content::Character::Defect);
    death.state.player.hp=3;death.state.player_full_powers.push(power("echo_form",1));
    death.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(death.state.outcome,Some(Outcome::Death));
    super::super::engine::validate_runtime(&death.state).unwrap();
    let exit=death.exit().unwrap();assert_eq!(exit.queued_autoplay.len(),1);assert_eq!(exit.in_play.as_ref().unwrap().instance,CardInstanceId(1));
    let mut duplicate=death.clone();duplicate.state.inactive_cards.push(duplicate.state.queued_autoplay.values().next().unwrap().clone());
    assert!(super::super::engine::validate_runtime(&duplicate.state).is_err());
    let mut missing=death.clone();missing.state.card_ids_issued.remove(&super::super::card_state::handle(CardInstanceId(1)));
    assert!(super::super::engine::validate_runtime(&missing.state).is_err());
    let mut forged=death.clone();forged.state.in_play.as_mut().unwrap().full_mut().unwrap().origin=CombatCardRef::Permanent(CardInstanceId(999));
    assert!(super::super::engine::validate_runtime(&forged.state).is_err());
    let frozen=death.clone();assert!(death.step_full(&CombatAction::EndTurn).is_err());assert_eq!(death,frozen);
    // The terminal exemption cannot authorize a missing continuation before terminal.
    let mut unfinished=frozen;unfinished.state.outcome=None;unfinished.state.phase=Phase::PlayerAction;
    assert!(super::super::engine::validate_runtime(&unfinished.state).is_err());
}
