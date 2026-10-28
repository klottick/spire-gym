//! Bounded regressions for final review I1/M1/M2; supplied checkpoints are explicit.
use super::super::*;
use super::super::cards::DefectEffect;
use crate::content::{CardId,Character};
fn power(id:&'static str,amount:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}}
fn ready()->Combat{
    let mut combat=super::full_runtime::entered_combat(3,3, |_|{});
    combat.state.character=CharacterState::for_character(Character::Defect);
    combat.state.silent.source_character=Some(Character::Defect);combat.state.silent.history_enabled=true;combat
}

#[test]
fn m16_review_i1_heatsinks_top_draw_precedes_payload_and_both_repeats(){
    for intensity in [1,2]{
        let mut combat=ready();combat.state.player_full_powers.extend([power("storm",1),power("heatsinks",intensity),power("echo_form",1),power("amplify",1)]);
        let instance=combat.state.piles.hand[0].full().unwrap().instance;
        *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(CardId::Defragment,instance,0);
        for n in 20..26{let card=permanent_live_card(CardId::DefendBlue,CardInstanceId(n),0).copy_as(CardInstanceId(n));super::super::piles::insert_generated(&mut combat.state,card,CardDestination::Draw).unwrap();}
        let action=CombatAction::PlayCard{instance:instance.0,target:None};
        let prepared=super::super::cards::prepare(&combat,&action).unwrap();
        let owned=combat.state.piles.hand.remove(prepared.hand_index);let owner=owned.handle();let source=Source::Card(owner.clone());
        combat.state.in_play=Some(owned);combat.state.phase=Phase::PlayerAction;combat.state.player.energy=prepared.remaining_energy;
        combat.state.play_context=Some(super::super::model::PlayContext{card:owner,target:prepared.target,energy_on_use:prepared.captured_energy,paid:prepared.paid,disposition:super::super::model::FinishDisposition::Natural});
        combat.state.pending.extend(prepared.effects);
        combat.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardAfterUse)});
        combat.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::FinishCard});
        combat.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
        // Actual shared notification, with original payload/finish already queued.
        super::super::hooks::dispatch(&mut combat.state,Hook::CardPlayed,&source,None).unwrap();
        assert_eq!(combat.state.pending.front().unwrap().kind,EffectKind::Draw(intensity));
        let finish=combat.state.pending.iter().position(|e|e.kind==EffectKind::FinishCard).unwrap();
        let repeat=combat.state.pending.iter().position(|e|matches!(e.kind,EffectKind::Defect(DefectEffect::Repeat{..}))).unwrap();
        let storm=combat.state.pending.iter().position(|e|e.kind==EffectKind::Character(CharacterEffect::Channel(OrbKind::Lightning))).unwrap();
        assert!(finish<repeat&&repeat<storm,"Storm remains bottom work after retained repeats");
        let hand=combat.state.piles.hand.len();
        for _ in 0..32{
            if combat.state.pending.front().is_some_and(|e|matches!(e.kind,EffectKind::ApplyFullPower{id:FullPowerId("focus"),..})){break;}
            super::super::engine::dispatch_one(&mut combat.state);assert!(combat.state.fault.is_none());
        }
        assert_eq!(combat.state.piles.hand.len(),hand+intensity as usize,"earned actual draws precede original Power payload");
        assert_eq!(combat.state.character.focus,0);assert_eq!(combat.state.queued_autoplay.len(),2);
        assert_eq!(super::super::engine::resolve(&mut combat.state).status,TransitionStatus::Decision);
        assert_eq!(combat.state.character.focus,3);assert_eq!(combat.state.player.energy,2);
        assert_eq!(combat.state.piles.draw.len(),6-3*intensity as usize,"original and both purge children notify Heatsinks");
        assert_eq!(combat.state.character.orb_slots.iter().flatten().count(),3);
        assert_eq!(combat.state.played_powers.len(),1);assert_eq!(combat.state.inactive_cards.len(),2);
        assert!(combat.state.play_frames.is_empty()&&combat.state.queued_autoplay.is_empty());
        super::super::engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m16_review_m1_real_heatsinks_998_to_1000_validates(){
    let mut combat=ready();combat.state.player_full_powers.push(power("heatsinks",998));
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(CardId::Heatsinks,instance,1);
    super::super::engine::validate_runtime(&combat.state).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    assert_eq!(combat.state.player_full_powers.iter().find(|p|p.id.0=="heatsinks").unwrap().state.amount,1000);
    super::super::engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m16_review_m1_real_bias_999_to_1000_validates_and_focus_clamps(){
    let mut combat=ready();combat.state.player_full_powers.push(power("bias",999));
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(CardId::BiasedCognition,instance,1);
    super::super::engine::validate_runtime(&combat.state).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    assert_eq!(combat.state.character.focus,5);
    assert_eq!(combat.state.player_full_powers.iter().find(|p|p.id.0=="bias").unwrap().state.amount,1000);
    super::super::engine::validate_runtime(&combat.state).unwrap();
    combat.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(combat.state.character.focus,-995);
    combat.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(combat.state.character.focus,-999);
    super::super::engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m16_review_m1_qualified_positive_domains_preserve_other_bounds(){
    for id in ["heatsinks","bias"]{
        for amount in [1,1000,i32::MAX]{let mut combat=ready();combat.state.player_full_powers.push(power(id,amount));super::super::engine::validate_runtime(&combat.state).unwrap();}
        for amount in [0,-1,i32::MIN]{let mut combat=ready();combat.state.player_full_powers.push(power(id,amount));assert!(super::super::engine::validate_runtime(&combat.state).is_err());}
        let mut duplicate=ready();duplicate.state.player_full_powers.extend([power(id,1),power(id,2)]);assert!(super::super::engine::validate_runtime(&duplicate.state).is_err());
    }
    for id in ["storm","strength","dexterity"]{let mut combat=ready();combat.state.player_full_powers.push(power(id,1000));assert!(super::super::engine::validate_runtime(&combat.state).is_err(),"old producer bound {id}");}
    let mut focus=ready();focus.state.character.focus=1000;assert!(super::super::engine::validate_runtime(&focus.state).is_err());
}

#[test]
fn m16_review_m2_snapshot_counter_only_divergence(){
    let combat=ready();let left=combat.snapshot();let mut right=left.clone();right.state.defect.echo_doubled_this_turn=1;
    assert_ne!(left,right);assert_eq!(super::super::snapshot::compare(&left,&right).unwrap().component,"defect");
    right.state.turn+=1;assert_eq!(super::super::snapshot::compare(&left,&right).unwrap().component,"turn","existing label priority stays unchanged");
    assert!(super::super::snapshot::compare(&left,&left).is_none());
}

#[test]
fn m16_review_m2_snapshot_journal_only_divergence(){
    let mut combat=ready();let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(CardId::GeneticAlgorithm,instance,0);
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    let left=combat.snapshot();let mut right=left.clone();right.state.defect.permanent_mutations[0].misc+=1;
    assert_ne!(left,right);let divergence=super::super::snapshot::compare(&left,&right).unwrap();
    assert_eq!(divergence.component,"defect");assert_ne!(divergence.left,divergence.right);
}

#[test]
fn m16_review_m1_privileged_selection_and_constructor_exclusion(){
    use crate::{content::PotionId,run::*};
    let start=AuthoredRunStart{hp:75,max_hp:75,gold:99,deck:vec![CardInstance{id:CardId::Heatsinks,instance:1,upgrades:1,misc:0}],relics:vec![],potions:vec![Some(PotionId::GamblersBrew),None,None],relic_counters:Default::default()};
    assert!(Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(start),RunRng::new([7;32],rng::Partition::Test,0)).is_err(),"Gambler remains outside M16 constructor admission");
    // Privileged shared-core composition only: no public initial-power API or
    // admitted M16 history to998 is claimed by this constructed prestate.
    let mut combat=ready();combat.state.player_full_powers.push(power("heatsinks",998));
    combat.state.item_context=Some(crate::items::CombatItemContext::new(vec![],vec![Some(PotionId::GamblersBrew),None,None]));
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(CardId::Heatsinks,instance,1);
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    combat.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection);assert!(combat.public_full().unwrap().selection.is_some());
}
