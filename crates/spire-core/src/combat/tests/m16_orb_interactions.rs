use super::super::*;
use super::full_runtime::entered_combat;
use crate::{content::{Character,RelicId},items::CombatItemContext};
fn power(id:&'static str,amount:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}}
fn defect()->Combat{let mut combat=entered_combat(3,0, |_|{});combat.state.character=CharacterState::for_character(Character::Defect);combat.state.silent.source_character=Some(Character::Defect);combat}
#[test]
fn m16_orb_interactions_bias_loop_cables_chip_order(){
    let mut combat=defect();
    combat.state.character.focus=2;
    combat.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];
    combat.state.player_full_powers.extend([power("loop",1),power("bias",1)]);
    let mut items=CombatItemContext::new(vec![RelicId::GoldPlatedCables,RelicId::EmotionChip],vec![None;3]);
    items.emotion_chip_armed=true;combat.state.item_context=Some(items);
    super::super::turns::schedule_next_player_turn(&mut combat.state).unwrap();
    assert_eq!(super::super::engine::resolve(&mut combat.state).status,TransitionStatus::Decision);
    assert_eq!(combat.state.character.orb_slots[0].unwrap().dark_damage,Some(30));
    assert_eq!(combat.state.character.focus,1);
    assert!(!combat.state.item_context.as_ref().unwrap().emotion_chip_armed);
    for slots in [0,3]{
        let mut combat=defect();combat.state.character.orb_slots=vec![None;slots];combat.state.player_full_powers.push(power("bias",1));
        super::super::turns::schedule_next_player_turn(&mut combat.state).unwrap();
        super::super::engine::resolve(&mut combat.state);
        assert_eq!(combat.state.character.focus,-1);
    }
}
#[test]
fn m16_orb_interactions_electro_lockon_defenses(){
    let mut combat=defect();
    combat.state.enemies[0].full_powers.extend([power("lock_on",3),power("vulnerable",2)]);
    let hp=combat.state.enemies[0].hp;
    let effect=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),kind:EffectKind::Damage(DamagePayload::Orb(3))};
    super::super::effects::apply_effect(&mut combat.state,&effect).unwrap();
    assert_eq!(hp-combat.state.enemies[0].hp,4,"binary32 Lock-On; no Vulnerable");
}
#[test]
fn m16_orb_interactions_discharge_damage_stage(){
    let mut combat=defect();combat.state.player_full_powers.push(power("static_discharge",1));
    combat.state.item_context=Some(CombatItemContext::new(vec![RelicId::EmotionChip],vec![None;3]));
    let source=Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone()));
    let effect=Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))};
    super::super::effects::apply_effect(&mut combat.state,&effect).unwrap();
    assert!(combat.state.pending.iter().any(|effect|effect.kind==EffectKind::Character(CharacterEffect::Channel(OrbKind::Lightning))));
    assert!(combat.state.item_context.as_ref().unwrap().emotion_chip_armed);
}

#[test]
fn m16_orb_interactions_numeric_and_continuation_faults(){
    for raw in [-1,i32::MIN,2_000,i32::MAX]{
        let mut combat=defect();combat.state.enemies[0].block=17;
        let hp=combat.state.enemies[0].hp;
        let effect=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),kind:EffectKind::Damage(DamagePayload::Orb(raw))};
        super::super::effects::apply_effect(&mut combat.state,&effect).unwrap();
        assert_eq!(combat.state.enemies[0].hp,if raw<=0{hp}else{0});
        assert_eq!(combat.state.enemies[0].block,if raw<=0{17}else{0});
    }
    let mut combat=defect();combat.state.player.energy=998;
    super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::OrbEnergy(2)).unwrap();
    assert_eq!(combat.state.player.energy,999);
    combat.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(i32::MAX)}),None,None];
    super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::EndTurn).unwrap();
    assert_eq!(combat.state.character.orb_slots[0].unwrap().dark_damage,Some(i32::MIN+5));
    let before=combat.state.character.clone();
    let bad=Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Slots(10))};
    assert!(super::super::engine::validate_effect(&bad,&combat.state).is_err());
    assert_eq!(combat.state.character,before);
}

#[test]
fn m16_orb_interactions_artifact_and_discharge_qualifiers(){
    use crate::content::CardId;
    for up in [0,1]{
        let mut combat=defect();combat.state.player.energy=3;combat.state.player_full_powers.push(power("artifact",1));
        let instance=combat.state.piles.hand[0].full().unwrap().instance;
        *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(CardId::BiasedCognition,instance,up);
        combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
        assert_eq!(combat.state.character.focus,if up>0{5}else{4});assert!(!combat.state.player_full_powers.iter().any(|p|p.id.0=="bias"));
    }
    for (payload,block,qualifies) in [(DamagePayload::Fixed(1),1,false),(DamagePayload::Thorns(1),0,false),(DamagePayload::HpLoss(1),0,false),(DamagePayload::Fixed(1),0,true)]{
        let mut combat=defect();combat.state.player.block=block;combat.state.player_full_powers.push(power("static_discharge",2));
        let effect=Effect{source:Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(payload)};
        if let EffectKind::Damage(payload @ DamagePayload::Thorns(_))=&effect.kind {
            super::super::cards::defect::on_attacked(&mut combat.state,&effect,payload,1);
        }else{super::super::effects::apply_effect(&mut combat.state,&effect).unwrap();}
        let channels=combat.state.pending.iter().filter(|e|e.kind==EffectKind::Character(CharacterEffect::Channel(OrbKind::Lightning))).count();
        assert_eq!(channels,if qualifies{2}else{0});
    }
    // Typed onAttacked checkpoint: a later loss reducer may prevent HP loss,
    // without undoing Discharge's earlier callback. Chip remains unarmed here.
    let mut combat=defect();combat.state.player_full_powers.push(power("static_discharge",1));
    combat.state.item_context=Some(CombatItemContext::new(vec![RelicId::EmotionChip],vec![None;3]));
    let payload=DamagePayload::Fixed(1);let effect=Effect{source:Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(payload.clone())};
    super::super::cards::defect::on_attacked(&mut combat.state,&effect,&payload,1);
    assert_eq!(combat.state.pending.len(),1);assert!(!combat.state.item_context.as_ref().unwrap().emotion_chip_armed);
}

#[test]
fn m16_orb_interactions_reject_forged_power_and_wrong_focus_target(){
    let mut combat=defect();combat.state.player_full_powers.push(power("bias",1));
    let wrong=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),kind:EffectKind::Character(CharacterEffect::Focus(-1))};
    assert!(super::super::engine::validate_effect(&wrong,&combat.state).is_err());
    let forged=Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("loop"),amount:1}};
    assert!(super::super::engine::validate_effect(&forged,&combat.state).is_err());
    let before=combat.state.character.clone();
    assert!(super::super::characters::channel(&mut combat.state,&Source::None,OrbState{kind:OrbKind::Frost,dark_damage:Some(7)},true).is_err());
    assert_eq!(combat.state.character,before);
}

#[test]
fn m16_orb_interactions_queued_impulse_energy_waits_until_after_draw(){
    let mut combat=defect();combat.state.player_full_powers.push(power("loop",1));
    combat.state.character.orb_slots=vec![Some(OrbKind::Plasma.into()),None,None];
    let mut items=CombatItemContext::new(vec![RelicId::GoldPlatedCables,RelicId::EmotionChip],vec![None;3]);items.emotion_chip_armed=true;combat.state.item_context=Some(items);
    super::super::turns::schedule_next_player_turn(&mut combat.state).unwrap();
    super::super::engine::dispatch_one(&mut combat.state); // invoke start hooks and recharge
    assert_eq!(combat.state.pending.front().unwrap().kind,EffectKind::Character(CharacterEffect::PassiveGroup(super::super::characters::OrbPassiveGroup::All)));
    super::super::engine::dispatch_one(&mut combat.state); // Impulse invokes callbacks, queues children
    let draw=combat.state.pending.iter().position(|e|matches!(e.kind,EffectKind::Draw(_))).unwrap();
    let energy=|e:&&Effect|matches!(e.kind,EffectKind::Character(CharacterEffect::OrbEnergy(1)));
    assert_eq!(combat.state.pending.iter().take(draw).filter(energy).count(),3,"Loop1 + natural1 + natural Cables1 precede draw");
    assert_eq!(combat.state.pending.iter().skip(draw+1).filter(energy).count(),2,"Impulse1 + its Cables1 append after the already queued draw");
    assert_eq!(super::super::engine::resolve(&mut combat.state).status,TransitionStatus::Decision);
    assert_eq!(combat.state.player.energy,8);
}

#[test]
fn m16_orb_interactions_frost_gain_hook_finishes_before_monster_turn(){
    let mut combat=entered_combat(3,3, |_|{});
    combat.step_full(&CombatAction::EndTurn).unwrap();
    combat.state.character=CharacterState::for_character(Character::Defect);combat.state.silent.source_character=Some(Character::Defect);
    combat.state.character.orb_slots[0]=Some(OrbKind::Frost.into());combat.state.player.hp=1;
    combat.state.player_full_powers.push(power("juggernaut",60));
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.outcome,Some(Outcome::Victory));assert_eq!(combat.state.player.hp,1);
}

#[test]
fn m16_orb_interactions_electro_all_targets_and_lockon_duration(){
    for duration in [1,3]{
        let mut combat=Combat::enter(super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes)).unwrap();
        combat.state.character=CharacterState::for_character(Character::Defect);combat.state.silent.source_character=Some(Character::Defect);
        combat.state.player_full_powers.extend([power("electrodynamics",1),power("strength",20),power("weak",2)]);
        combat.state.enemies[0].full_powers.extend([power("lock_on",duration),power("vulnerable",2)]);
        let hp=combat.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();let rng=combat.state.card_rng.clone();
        super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::Lightning{amount:8}).unwrap();
        combat.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
        assert_eq!(super::super::engine::resolve(&mut combat.state).status,TransitionStatus::Decision);
        for (i,enemy) in combat.state.enemies.iter().enumerate(){assert_eq!(hp[i]-enemy.hp,if i==0{12}else{8});}
        assert_eq!(combat.state.card_rng,rng,"Electrodynamics uses no random target draw");
    }
}
