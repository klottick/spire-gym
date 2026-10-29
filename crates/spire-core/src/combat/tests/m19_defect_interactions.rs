//! Supplied Defect/Act3 compositions; actual shared setup and resolver throughout.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,EnemyId},items::CombatItemContext};
use characters::{CharacterEffect,OrbPassiveGroup};
use super::m5_elites::{amount,hit};
fn power(id:&'static str,amount:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}}
fn enter(encounter:EncounterId,cards:&[CardId])->Combat{
    let mut e=super::m8_shapes::entry(encounter);e.setup.character=Character::Defect;
    e.cards=cards.iter().enumerate().map(|(i,id)|permanent_live_card(*id,CardInstanceId(i as u64+1),0)).collect();e.opening_draw=cards.len().min(10);e.setup.player.draw_per_turn=cards.len().clamp(1,10) as i32;
    let mut items=Some(CombatItemContext::new(vec![],vec![None;3]));Combat::enter_with_item_context(e,&mut items).unwrap()
}
fn character(c:&mut Combat,work:CharacterEffect){c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(work)}],None).unwrap();}
fn play(c:&mut Combat,instance:u64,target:Option<TargetId>){c.step_full(&CombatAction::PlayCard{instance,target}).unwrap_or_else(|e|panic!("{e:?} playing {instance} at turn{}; legal={:?}",c.state.turn,c.legal_actions_full()));}
fn set_intent(c:&mut Combat,slot:usize,name:&'static str){let id=ActorIntentId::Full(FullIntentId::from_name(name));let enemy=&mut c.state.enemies[slot];enemy.intent_id=id.clone();enemy.selected_move=id.clone();enemy.move_history.push(id);effects::refresh_enemy_values(&mut c.state).unwrap();}
#[test]
fn m19_defect_interactions_slow_intangible_and_spiker(){
    let mut head=enter(EncounterId::GiantHead,&[CardId::Barrage]);head.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="slow").unwrap().state.amount=5;
    head.state.character.orb_slots=vec![Some(OrbKind::Frost.into()),None,Some(OrbKind::Frost.into()),None,Some(OrbKind::Frost.into()),None,Some(OrbKind::Frost.into()),None,None,None];
    play(&mut head,1,Some(TargetId(1)));assert_eq!(head.state.enemies[0].hp,476,"four actual Barrage hits, each floor4*1.5=6");assert_eq!(amount(&head,0,"slow"),6);
    head.state.character.orb_slots=vec![Some(OrbKind::Lightning.into()),None,None];character(&mut head,CharacterEffect::PassiveGroup(OrbPassiveGroup::NaturalEnd));assert_eq!(head.state.enemies[0].hp,473,"Slow does not scale orb THORNS");assert_eq!(amount(&head,0,"slow"),6);
    let mut nemesis=enter(EncounterId::Nemesis,&[CardId::Barrage]);nemesis.state.player.draw_per_turn=10;nemesis.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&nemesis,0,"intangible"),1);let hp=nemesis.state.enemies[0].hp;
    nemesis.state.character.orb_slots=vec![Some(OrbKind::Frost.into());4];play(&mut nemesis,1,Some(TargetId(1)));assert_eq!(nemesis.state.enemies[0].hp,hp-4);
    nemesis.state.character.orb_slots=vec![Some(OrbKind::Lightning.into()),Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(20)}),None];character(&mut nemesis,CharacterEffect::EvokeFront{remove:true});character(&mut nemesis,CharacterEffect::EvokeFront{remove:true});assert_eq!(nemesis.state.enemies[0].hp,hp-6);
    // This actual setup's shape alternatives are deterministic under the shared
    // test stream. No synthetic actor species or copied HP table is used.
    let mut shapes=enter(EncounterId::FourShapes,&[CardId::Barrage]);let slot=shapes.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::Spiker)).expect("actual FourShapes includes Spiker");let hp=shapes.state.player.hp;
    shapes.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];shapes.state.enemies[slot].hp=1;
    character(&mut shapes,CharacterEffect::EvokeFront{remove:true});assert_eq!(shapes.state.player.hp,hp,"orb does not trigger thorns");
    let mut shapes=enter(EncounterId::FourShapes,&[CardId::Barrage]);let slot=shapes.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::Spiker)).unwrap();let thorns=amount(&shapes,slot,"thorns");shapes.state.character.orb_slots=vec![Some(OrbKind::Frost.into());4];let hp=shapes.state.player.hp;play(&mut shapes,1,Some(TargetId(slot as u64+1)));assert_eq!(shapes.state.player.hp,hp-4*thorns);
    let mut lethal=enter(EncounterId::FourShapes,&[CardId::Barrage]);let slot=lethal.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::Spiker)).unwrap();lethal.state.character.orb_slots=vec![Some(OrbKind::Frost.into());4];lethal.state.player.hp=1;let hp=lethal.state.enemies[slot].hp;play(&mut lethal,1,Some(TargetId(slot as u64+1)));assert_eq!(lethal.state.outcome,Some(Outcome::Death));assert_eq!(lethal.state.enemies[slot].hp,hp-4);
}
#[test]
fn m19_defect_interactions_awakened_power_orb_rebirth(){
    let mut c=enter(EncounterId::AwakenedOne,&[CardId::Defragment]);c.state.player_full_powers.extend([power("amplify",1),power("echo_form",1)]);play(&mut c,1,None);assert_eq!(c.state.character.focus,3);assert_eq!(amount(&c,2,"strength"),3,"three actual notified Power uses");
    let mut c=enter(EncounterId::AwakenedOne,&[CardId::Zap]);for slot in [0,1]{let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp);}let hp=c.state.enemies[2].hp;hit(&mut c,2,hp-3);
    c.state.character.orb_slots=vec![Some(OrbKind::Lightning.into()),None,None];character(&mut c,CharacterEffect::PassiveGroup(OrbPassiveGroup::NaturalEnd));assert_eq!(c.state.enemies[2].life,ActorLife::HalfDead);assert_eq!(c.state.outcome,None);let actor=c.state.enemies[2].instance_id.clone();
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[2].instance_id,actor);assert_eq!(c.state.enemies[2].hp,300);assert_eq!(amount(&c,2,"curiosity"),0);assert_eq!(c.state.outcome,None);
    set_intent(&mut c,2,"awa_sludge");c.step_full(&CombatAction::EndTurn).unwrap();assert!(c.state.piles.draw.iter().any(|card|card.full().is_some_and(|card|card.id==CardId::Void))||c.state.piles.hand.iter().any(|card|card.full().is_some_and(|card|card.id==CardId::Void)));
    c.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(300)}),None,None];character(&mut c,CharacterEffect::EvokeFront{remove:true});assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(engine::validate_runtime(&c.state).is_ok());
}
/// Run-owned diagnostic checkpoint; intent payload remains the actual shared move.
pub(crate) fn implant_checkpoint(c:&mut Combat){assert_eq!(c.state.enemies[0].enemy_id,ActorEnemyId::Full(EnemyId::WrithingMass));set_intent(c,0,"writhing_parasite");}
pub(crate) fn arm_later_dispatch_fault(c:&mut Combat){
    // A staged accepted continuation after the real gain. Public step resets
    // its dispatch count, so exhaustion belongs at the shared resolver frontier.
    c.state.phase=Phase::PlayerAction;c.state.pending.push_front(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn});c.state.resolved_effects=99_999;
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
}
#[test]
fn m19_defect_interactions_timewarp_unwinds_blue_repeat_once(){
    for (count,focus,uses) in [(11,1,1),(10,2,2)]{
        let mut c=enter(EncounterId::TimeEater,&[CardId::Defragment]);set_intent(&mut c,0,"time_ripple");c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=count;
        c.state.player_full_powers.extend([power("echo_form",1),power("amplify",1)]);c.state.character.orb_slots=vec![Some(OrbKind::Frost.into()),Some(OrbKind::Dark.into()),Some(OrbKind::Plasma.into())];
        play(&mut c,1,None);assert_eq!(c.state.turn,2);assert_eq!(c.state.character.focus,focus);assert_eq!(amount(&c,0,"time_warp"),0);assert_eq!(amount(&c,0,"strength"),2);assert!(c.state.pending_time_warp.is_none());assert!(c.state.in_play.is_none()&&c.state.play_frames.is_empty()&&c.state.queued_autoplay.is_empty());assert_eq!(c.state.played_powers.len(),1);assert_eq!(c.state.inactive_cards.len(),2,"used or suppressed purge children retire once");
        assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardPlayed).count(),uses);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);assert_eq!(c.state.character.orb_slots[1].unwrap().dark_damage,Some(12+focus));assert_eq!(c.state.player.energy,4);assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="amplify"));assert_eq!(c.state.defect.echo_doubled_this_turn,0);assert!(engine::validate_runtime(&c.state).is_ok());
    }
    // Ordinary turns and orb hooks do not reset/increment the boss's use counter.
    let mut c=enter(EncounterId::TimeEater,&[CardId::Zap]);c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=9;set_intent(&mut c,0,"time_ripple");play(&mut c,1,None);assert_eq!(amount(&c,0,"time_warp"),10);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"time_warp"),10);
}
#[test]
fn m19_defect_interactions_dark_partner_and_halfdead_targets(){
    let mut pair=enter(EncounterId::DonuAndDeca,&[CardId::Bullseye,CardId::Bullseye,CardId::Bullseye]);assert_eq!(pair.state.enemies.iter().map(|e|e.enemy_id.clone()).collect::<Vec<_>>(),vec![ActorEnemyId::Full(EnemyId::Deca),ActorEnemyId::Full(EnemyId::Donu)]);assert_eq!((amount(&pair,0,"artifact"),amount(&pair,1,"artifact")),(2,2));
    pair.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(12)}),None,None];character(&mut pair,CharacterEffect::EvokeFront{remove:true});assert_eq!((pair.state.enemies[0].hp,pair.state.enemies[1].hp),(238,250));assert_eq!(amount(&pair,0,"artifact"),2);
    for i in [1,2]{play(&mut pair,i,Some(TargetId(1)));assert_eq!(amount(&pair,0,"artifact"),2-i as i32);assert_eq!(amount(&pair,0,"lock_on"),0);}play(&mut pair,3,Some(TargetId(1)));assert_eq!(amount(&pair,0,"lock_on"),2);let hp=pair.state.enemies[0].hp;pair.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(3)}),None,None];character(&mut pair,CharacterEffect::EvokeFront{remove:true});assert_eq!(pair.state.enemies[0].hp,hp-4,"binary32 LockOn truncates3*1.5");
    // Capture the initial equal-HP formation tie, then lower the surviving partner
    // before delivery. Queued Dark retains Deca, rather than selecting again.
    let mut pair=enter(EncounterId::DonuAndDeca,&[CardId::Zap]);pair.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];characters::apply(&mut pair.state,&Source::Actor(Actor::Player),CharacterEffect::EvokeFront{remove:true}).unwrap();let captured=pair.state.pending.pop_front().unwrap();assert_eq!(captured.target,Some(Actor::Enemy(pair.state.enemies[0].instance_id.clone())));hit(&mut pair,1,249);pair.enqueue_native_effects(vec![captured],None).unwrap();assert_eq!((pair.state.enemies[0].hp,pair.state.enemies[1].hp),(244,1));assert_eq!(pair.state.outcome,None);
    let mut c=enter(EncounterId::ThreeDarklings,&[CardId::Recursion]);let hp=c.state.enemies[0].hp;hit(&mut c,0,hp);let before=c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();let eligible=(1..3).min_by_key(|i|(before[*i],*i)).unwrap();let deaths=c.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count();c.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(12)}),None,None];play(&mut c,1,None);assert_eq!(c.state.enemies[0].life,ActorLife::HalfDead);assert_eq!(c.state.enemies[0].hp,0);assert_eq!(c.state.enemies[eligible].hp,before[eligible]-12,"constructor excludes the already-HalfDead minimum via inherited helper");assert_eq!(c.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count(),deaths);
}
#[test]
fn m19_defect_interactions_dark_captured_before_halfdeath_keeps_reference(){
    let mut c=enter(EncounterId::ThreeDarklings,&[CardId::Zap]);let minimum=c.state.enemies.iter().enumerate().min_by_key(|(i,e)|(e.hp,*i)).unwrap().0;c.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];characters::apply(&mut c.state,&Source::Actor(Actor::Player),CharacterEffect::EvokeFront{remove:true}).unwrap();let captured=c.state.pending.pop_front().unwrap();let owner=c.state.enemies[minimum].instance_id.clone();assert_eq!(captured.target,Some(Actor::Enemy(owner)));
    let hp=c.state.enemies[minimum].hp;hit(&mut c,minimum,hp);assert_eq!(c.state.enemies[minimum].life,ActorLife::HalfDead);let others=c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();let deaths=c.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count();engine::validate_effect(&captured,&c.state).unwrap();c.state.pending.push_front(captured);assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);assert_eq!(c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),others);assert_eq!(c.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count(),deaths);assert_eq!(c.state.outcome,None);assert!(engine::validate_runtime(&c.state).is_ok());
}
#[test]
fn m19_defect_interactions_timewarp_actual_death_and_electro_collapse(){
    let mut c=enter(EncounterId::TimeEater,&[CardId::Defragment]);c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;set_intent(&mut c,0,"time_reverb");c.state.player.hp=1;c.state.player_full_powers.extend([power("echo_form",1),power("amplify",1)]);play(&mut c,1,None);assert_eq!(c.state.outcome,Some(Outcome::Death));assert_eq!(c.state.player.hp,0);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),0,"Death cancels the later turn-end notification");assert!(c.legal_actions_full().unwrap().is_empty());assert!(engine::validate_runtime(&c.state).is_ok());
    let mut c=enter(EncounterId::ThreeDarklings,&[CardId::Electrodynamics]);play(&mut c,1,None);for slot in 0..3{let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp-1);}c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(c.state.enemies.iter().all(|e|e.hp==0&&e.life!=ActorLife::HalfDead));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);assert!(engine::validate_runtime(&c.state).is_ok());
}
#[test]
fn m19_defect_interactions_dark_dead_capture_does_not_retarget_partner(){
    let mut c=enter(EncounterId::DonuAndDeca,&[CardId::Zap]);c.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];characters::apply(&mut c.state,&Source::Actor(Actor::Player),CharacterEffect::EvokeFront{remove:true}).unwrap();let owner=c.state.enemies[0].instance_id.clone();assert_eq!(c.state.pending.front().unwrap().target,Some(Actor::Enemy(owner.clone())));
    // A supplied preceding hit exercises common corpse cleanup while the Dark
    // target reference is already queued. It must never redirect to Donu.
    effects::apply_effect(&mut c.state,&Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(250))}).unwrap();assert!(!c.state.pending.iter().any(|e|e.target==Some(Actor::Enemy(owner.clone()))&&matches!(e.kind,EffectKind::Damage(_))));assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);assert_eq!((c.state.enemies[0].hp,c.state.enemies[1].hp),(0,250));assert_eq!(c.state.outcome,None);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),1);assert!(engine::validate_runtime(&c.state).is_ok());
}
