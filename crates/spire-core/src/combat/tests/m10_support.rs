use super::super::*;
use super::full_runtime::entered_combat;
use crate::{content::{CardId,CardRarity,CardType,Catalog},combat::model::OwnedCard};

fn with_card(id:CardId,upgraded:bool)->Combat{
    entered_combat(3,3,|card|*card=card_state::permanent(id,card.instance,u32::from(upgraded)))
}
fn play(c:&mut Combat,target:Option<TargetId>){
    let instance=c.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card|matches!(card.id,CardId::Entrench|CardId::GhostlyArmor|CardId::Intimidate|CardId::PowerThrough|CardId::RecklessCharge|CardId::SeeingRed|CardId::Shockwave|CardId::SpotWeakness))
        .expect("support card in hand").instance.0;
    c.step_full(&CombatAction::PlayCard{instance,target}).unwrap();
}
fn card(c:&Combat,id:CardId)->&LiveCard{
    c.state.piles.all(c.state.in_play.as_ref()).filter_map(OwnedCard::full).find(|card|card.id==id).unwrap()
}
fn assert_uncommon(id:CardId){
    assert_eq!(Catalog::standard().card(id).unwrap().rarity,CardRarity::Uncommon);
}
fn stat_power(id:&'static str,amount:i32)->FullPowerEntry{
    FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}
}
fn three_shapes_card(id:CardId,upgraded:bool)->Combat{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    entry.cards[0]=card_state::permanent(id,CardInstanceId(3),u32::from(upgraded));
    Combat::enter(entry).unwrap()
}
fn orb_walker_card(id:CardId,upgraded:bool)->Combat{
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::OrbWalker);
    let instance=entry.cards[0].instance;
    entry.cards[0]=card_state::permanent(id,instance,u32::from(upgraded));
    let mut c=Combat::enter(entry).unwrap();
    let laser=ActorIntentId::Full(FullIntentId::from_name("orb_laser"));
    c.state.enemies[0].intent_id=laser.clone();
    c.state.enemies[0].selected_move=laser.clone();
    c.state.enemies[0].move_history=vec![laser];
    effects::refresh_enemy_values(&mut c.state).unwrap();
    c
}

#[test]
fn m10_support_entrench_doubles_current_block(){
    for (upgraded,cost) in [(false,2),(true,1)]{
        let mut c=with_card(CardId::Entrench,upgraded);c.state.player.block=11;
        assert_uncommon(CardId::Entrench);
        play(&mut c,None);
        assert_eq!(c.state.player.block,22);
        assert_eq!(cards::ironclad::rules::public_values(card(&c,CardId::Entrench)).unwrap().magic_number,None);
        assert_eq!(c.state.player.energy,3-cost);
    }
}

#[test]
fn m10_support_ghostly_armor_block_and_ethereal(){
    for (upgraded,block) in [(false,10),(true,13)]{
        let mut c=with_card(CardId::GhostlyArmor,upgraded);assert_uncommon(CardId::GhostlyArmor);
        play(&mut c,None);assert_eq!(c.state.player.block,block);
        assert!(cards::ironclad::rules::public_values(card(&c,CardId::GhostlyArmor)).unwrap().ethereal);

        let mut unplayed=with_card(CardId::GhostlyArmor,upgraded);
        unplayed.step_full(&CombatAction::EndTurn).unwrap();
        assert!(unplayed.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|live|live.id==CardId::GhostlyArmor)));
    }
}

#[test]
fn m10_support_intimidate_applies_weak_to_all_and_exhausts(){
    for (upgraded,amount) in [(false,1),(true,2)]{
        let mut c=with_card(CardId::Intimidate,upgraded);assert_uncommon(CardId::Intimidate);
        play(&mut c,None);
        assert!(c.state.enemies.iter().all(|enemy|enemy.full_powers.iter().any(|power|power.id.0=="weak"&&power.state.amount==amount)));
        assert!(c.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|live|live.id==CardId::Intimidate)));
    }
}

#[test]
fn m10_support_power_through_adds_two_wounds_before_block(){
    for upgraded in [false,true]{
        let mut c=with_card(CardId::PowerThrough,upgraded);assert_uncommon(CardId::PowerThrough);
        c.state.player_full_powers=vec![stat_power("dexterity",2),stat_power("frail",1)];
        play(&mut c,None);
        assert_eq!(c.state.player.block,if upgraded{16}else{12},"printed Block uses the ordinary Dexterity/Frail card calculation");
        assert_eq!(c.state.piles.hand.iter().filter(|owned|owned.full().is_some_and(|live|live.id==CardId::Wound)).count(),2);
        assert!(c.state.piles.draw.iter().chain(&c.state.piles.discard).chain(&c.state.piles.exhaust).all(|owned|owned.full().is_none_or(|live|live.id!=CardId::Wound)));
    }
}

#[test]
fn m10_support_reckless_charge_hits_then_inserts_dazed(){
    for (upgraded,damage) in [(false,7),(true,10)]{
        let mut c=with_card(CardId::RecklessCharge,upgraded);assert_uncommon(CardId::RecklessCharge);let hp=c.state.enemies[0].hp;
        play(&mut c,Some(TargetId(1)));
        assert_eq!(hp-c.state.enemies[0].hp,damage);
        assert!(c.state.piles.draw.iter().chain(&c.state.piles.hand).any(|owned|owned.full().is_some_and(|live|live.id==CardId::Dazed)));
    }
}

#[test]
fn m10_support_seeing_red_grants_energy_and_exhausts(){
    for (upgraded,cost) in [(false,1),(true,0)]{
        let mut c=with_card(CardId::SeeingRed,upgraded);assert_uncommon(CardId::SeeingRed);play(&mut c,None);
        assert_eq!(c.state.player.energy,3-cost+2);
        assert!(c.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|live|live.id==CardId::SeeingRed)));
    }
}

#[test]
fn m10_support_shockwave_applies_both_debuffs_and_exhausts(){
    for (upgraded,amount) in [(false,3),(true,5)]{
        let mut c=with_card(CardId::Shockwave,upgraded);assert_uncommon(CardId::Shockwave);play(&mut c,None);
        for enemy in &c.state.enemies{
            assert!(enemy.full_powers.iter().any(|power|power.id.0=="weak"&&power.state.amount==amount));
            assert!(enemy.full_powers.iter().any(|power|power.id.0=="vulnerable"&&power.state.amount==amount));
        }
        assert!(c.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|live|live.id==CardId::Shockwave)));
    }
}

#[test]
fn m10_support_spot_weakness_grants_strength_from_attack_intent(){
    for (upgraded,amount) in [(false,3),(true,4)]{
        let mut c=orb_walker_card(CardId::SpotWeakness,upgraded);assert_uncommon(CardId::SpotWeakness);
        assert_eq!(c.state.enemies[0].intent_id.as_str(),"orb_laser");
        let laser=c.state.enemies[0].full_damage_entries.get(&FullIntentId::from_name("orb_laser")).unwrap();
        assert_eq!(laser.base,10,"canonical Orb Walker mixed Attack+status damage base");
        assert!(engine::validate_runtime(&c.state).is_ok(),"real Orb Walker mixed-move fixture before public play");
        play(&mut c,Some(TargetId(1)));
        assert_eq!(c.state.player_full_powers.iter().find(|power|power.id.0=="strength").map(|power|power.state.amount),Some(amount));
        assert!(engine::validate_runtime(&c.state).is_ok(),"real Orb Walker mixed-move fixture after public play");
    }
}

#[test]
fn m10_support_catalog_values_flags_costs_and_target_shapes_are_literal(){
    let cases=[
        (CardId::Entrench,false,false,false),
        (CardId::GhostlyArmor,true,false,false),
        (CardId::Intimidate,false,true,false),
        (CardId::PowerThrough,false,false,false),
        (CardId::RecklessCharge,false,false,true),
        (CardId::SeeingRed,false,true,false),
        (CardId::Shockwave,false,true,false),
        (CardId::SpotWeakness,false,false,true),
    ];
    for (id,ethereal,exhaust,targeted) in cases{
        let definition=Catalog::standard().card(id).unwrap();
        assert_eq!((definition.rarity,definition.card_type,definition.definition_only),(CardRarity::Uncommon,
            if id==CardId::RecklessCharge{CardType::Attack}else{CardType::Skill},false),"domain identity for {id:?}");
        for upgraded in [false,true]{
            let mut c=with_card(id,upgraded);
            let (expected_damage,expected_block,expected_magic,expected_cost)=match id{
                CardId::Entrench=>(None,None,None,if upgraded{1}else{2}),
                CardId::GhostlyArmor=>(None,Some(if upgraded{13}else{10}),None,1),
                CardId::Intimidate=>(None,None,Some(if upgraded{2}else{1}),0),
                CardId::PowerThrough=>(None,Some(if upgraded{20}else{15}),None,1),
                CardId::RecklessCharge=>(Some(if upgraded{10}else{7}),None,None,0),
                CardId::SeeingRed=>(None,None,Some(2),if upgraded{0}else{1}),
                CardId::Shockwave=>(None,None,Some(if upgraded{5}else{3}),2),
                CardId::SpotWeakness=>(None,None,Some(if upgraded{4}else{3}),1),
                _=>unreachable!(),
            };
            let values=cards::ironclad::rules::public_values(card(&c,id)).unwrap();
            assert_eq!((values.base_damage,values.base_block,values.magic_number,values.ethereal,values.exhaust_on_finish),
                (expected_damage,expected_block,expected_magic,ethereal,exhaust),"public values for {id:?} upgraded={upgraded}");
            let target=targeted.then_some(TargetId(1));
            let action=CombatAction::PlayCard{instance:3,target};
            let prepared=cards::prepare(&c,&action).unwrap();
            assert_eq!(prepared.remaining_energy,3-expected_cost,"cost for {id:?} upgraded={upgraded}");
            let wrong=CombatAction::PlayCard{instance:3,target:if targeted{None}else{Some(TargetId(1))}};
            let before=c.clone();
            assert_eq!(c.step_full(&wrong),Err(crate::run::RunError::InvalidAction),"target shape for {id:?}");
            assert_eq!(c,before,"invalid target shape is atomic for {id:?}");
            if targeted{
                for target in [TargetId(99),TargetId(1)]{
                    let mut invalid=with_card(id,upgraded);
                    if target==TargetId(1){invalid.state.enemies[0].hp=0;}
                    let before=invalid.clone();
                    assert_eq!(invalid.step_full(&CombatAction::PlayCard{instance:3,target:Some(target)}),Err(crate::run::RunError::InvalidAction));
                    assert_eq!(invalid,before,"unknown/dead target is atomic for {id:?}");
                }
            }
        }
    }
}

#[test]
fn m10_support_entrench_doubles_without_card_block_modifiers_and_keeps_cap(){
    for (start,expected) in [(11,22),(0,0),(998,999)]{
        let mut c=with_card(CardId::Entrench,false);c.state.player.block=start;
        c.state.player_full_powers=vec![stat_power("dexterity",2),stat_power("frail",1)];
        assert!(engine::validate_runtime(&c.state).is_ok());
        play(&mut c,None);
        assert_eq!(c.state.player.block,expected,"Entrench directly adds current Block {start}, then ordinary add/cap applies");
    }
    let mut normal=with_card(CardId::Defend,false);
    normal.state.player_full_powers=vec![stat_power("dexterity",2),stat_power("frail",1)];
    assert!(engine::validate_runtime(&normal.state).is_ok());
    let defend=normal.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Defend).unwrap().instance.0;
    normal.step_full(&CombatAction::PlayCard{instance:defend,target:None}).unwrap();
    assert_eq!(normal.state.player.block,5,"ordinary printed 5 Block still applies Dexterity then Frail");
}

#[test]
fn m10_support_power_through_overflows_after_parent_leaves_hand_without_draw_callbacks(){
    for (upgraded,block) in [(false,15),(true,20)]{
      for (initial_hand,filled) in [(9,6),(10,7)]{
        let mut c=with_card(CardId::PowerThrough,upgraded);
        for instance in 10..10+filled{
            let handle=CardHandle::parse(&instance.to_string()).unwrap();c.state.card_ids_issued.insert(handle);
            c.state.piles.hand.push(OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(instance),0)));
        }
        assert_eq!(c.state.piles.hand.len(),initial_hand);
        assert!(engine::validate_runtime(&c.state).is_ok());
        let before_stream=c.state.card_rng.clone();let before_draw=c.state.piles.draw.clone();
        let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert!(matches!(prepared.effects[0].kind,EffectKind::CreateCard{id:CardId::Wound,destination:CardDestination::Hand,..}));
        assert!(matches!(prepared.effects[1].kind,EffectKind::CreateCard{id:CardId::Wound,destination:CardDestination::Hand,..}));
        assert!(matches!(prepared.effects[2].kind,EffectKind::GainBlock(_)),"both status insertions precede the ordinary Block action");
        play(&mut c,None);
        assert_eq!(c.state.player.block,block);
        assert_eq!(c.state.piles.hand.len(),10);
        let in_hand=c.state.piles.hand.iter().filter(|owned|owned.full().is_some_and(|live|live.id==CardId::Wound)).count();
        let in_discard=c.state.piles.discard.iter().filter(|owned|owned.full().is_some_and(|live|live.id==CardId::Wound)).count();
        assert_eq!((in_hand,in_discard),if initial_hand==9{(2,0)}else{(1,1)},"the parent vacates its hand slot before generated Wounds; the second create sees zero room in the full-hand case");
        assert_eq!(c.state.piles.draw,before_draw,"generated Wounds are not draws");
        assert_eq!(c.state.card_rng,before_stream,"generated Wounds do not invoke draw or shuffle callbacks");
        let wounds=c.state.piles.hand.iter().chain(&c.state.piles.discard).filter_map(OwnedCard::full).filter(|live|live.id==CardId::Wound).collect::<Vec<_>>();
        assert_eq!(wounds.len(),2);assert_ne!(wounds[0].instance,wounds[1].instance);
        assert!(engine::validate_runtime(&c.state).is_ok());
      }
    }
}

#[test]
fn m10_support_intimidate_and_shockwave_cover_every_live_enemy(){
    for (id,upgraded,weak,vulnerable) in [
        (CardId::Intimidate,false,1,None),(CardId::Intimidate,true,2,None),
        (CardId::Shockwave,false,3,Some(3)),(CardId::Shockwave,true,5,Some(5)),
    ]{
        let mut c=three_shapes_card(id,upgraded);assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,None);
        for enemy in c.state.enemies.iter().filter(|enemy|enemy.hp>0){
            assert!(enemy.full_powers.iter().any(|power|power.id.0=="weak"&&power.state.amount==weak));
            if let Some(amount)=vulnerable{assert!(enemy.full_powers.iter().any(|power|power.id.0=="vulnerable"&&power.state.amount==amount));}
        }
        assert!(engine::validate_runtime(&c.state).is_ok());
    }
}

#[test]
fn m10_support_intimidate_and_shockwave_share_individual_artifact_applications(){
    let mut c=three_shapes_card(CardId::Shockwave,false);
    for enemy in &mut c.state.enemies{enemy.full_powers.push(stat_power("artifact",1));}
    assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,None);
    for enemy in &c.state.enemies{
        assert!(!enemy.full_powers.iter().any(|power|power.id.0=="weak"),"first Shockwave debuff is individually blocked by Artifact");
        assert!(enemy.full_powers.iter().any(|power|power.id.0=="vulnerable"&&power.state.amount==3),"next Shockwave debuff applies after Artifact is spent");
        assert!(!enemy.full_powers.iter().any(|power|power.id.0=="artifact"));
    }
}

#[test]
fn m10_support_spot_weakness_candidates_do_not_reveal_current_intent_result(){
    let nonattack=with_card(CardId::SpotWeakness,false);
    let nonattack_actions=nonattack.legal_actions_full().unwrap();
    assert!(nonattack_actions.contains(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}));
    let attack=orb_walker_card(CardId::SpotWeakness,false);
    assert!(engine::validate_runtime(&nonattack.state).is_ok(),"nonattack candidate fixture remains runtime-valid");
    assert!(engine::validate_runtime(&attack.state).is_ok(),"canonical Orb Walker mixed-move candidate fixture remains runtime-valid after refresh");
    let attack_actions=attack.legal_actions_full().unwrap();
    assert!(attack_actions.contains(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}));
    let play_candidates=|actions:&[CombatAction],instance|actions.iter().filter(|action|matches!(action,CombatAction::PlayCard{instance:found,..} if *found==instance)).count();
    assert_eq!(play_candidates(&attack_actions,1),play_candidates(&nonattack_actions,3),"the conditional outcome does not remove a target candidate; unrelated enemy actions need not be identical");
}

#[test]
fn m10_support_spot_weakness_reads_intent_base_when_its_queued_action_dispatches(){
    // Narrow dispatcher test: the runtime-valid Cultist is given an Attack metadata
    // entry only to isolate the queued predicate's base-zero branch. This is not a
    // claim that Cultist has that move in gameplay; the public mixed move is tested
    // with Orb Walker's canonical Laser above.
    let mut c=with_card(CardId::SpotWeakness,false);
    let action=CombatAction::PlayCard{instance:3,target:Some(TargetId(1))};
    let prepared=cards::prepare(&c,&action).unwrap();
    assert!(matches!(prepared.effects[0].kind,EffectKind::Ironclad(IroncladEffect::SpotWeakness{amount:3})));
    let parent=c.state.piles.hand.remove(prepared.hand_index);c.state.in_play=Some(parent);
    assert_eq!(c.state.in_play.as_ref().and_then(OwnedCard::full).map(|live|live.id),Some(CardId::SpotWeakness));
    c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    c.state.enemies[0].intent_id=ActorIntentId::Full(FullIntentId::Attack);
    c.state.enemies[0].selected_move=ActorIntentId::Full(FullIntentId::Attack);
    c.state.enemies[0].full_damage_entries.get_mut(&FullIntentId::Attack).unwrap().base=0;
    let effect=prepared.effects[0].clone();
    effects::apply_effect(&mut c.state,&effect).unwrap();
    assert!(matches!(c.state.pending.front().map(|work|&work.kind),Some(EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3})));
    let strength=c.state.pending.pop_front().unwrap();effects::apply_effect(&mut c.state,&strength).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|power|power.id.0=="strength").map(|power|power.state.amount),Some(3));
}

#[test]
fn m10_support_spot_weakness_uses_nonnegative_base_even_for_mixed_intent(){
    let mut c=orb_walker_card(CardId::SpotWeakness,true);
    assert_eq!(c.state.enemies[0].intent_id.as_str(),"orb_laser");
    let laser=c.state.enemies[0].full_damage_entries.get(&FullIntentId::from_name("orb_laser")).unwrap();
    assert_eq!(laser.base,10,"canonical Orb Walker Laser base is produced by the real intent refresh");
    assert!(engine::validate_runtime(&c.state).is_ok(),"canonical mixed-intent state before public play");
    play(&mut c,Some(TargetId(1)));
    assert_eq!(c.state.player_full_powers.iter().find(|power|power.id.0=="strength").map(|power|power.state.amount),Some(4),
        "the native predicate accepts the selected canonical mixed Attack+status move from its nonnegative base");
    assert!(engine::validate_runtime(&c.state).is_ok(),"canonical mixed-intent state after public play");
}

#[test]
fn m10_support_spot_weakness_rejects_negative_base_at_dispatch(){
    // Narrow signed-metadata dispatcher case. Keep the actual parent identity in
    // `in_play`; this does not fabricate a gameplay move or public observation.
    let mut c=with_card(CardId::SpotWeakness,false);
    let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    let parent=c.state.piles.hand.remove(prepared.hand_index);c.state.in_play=Some(parent);
    assert_eq!(c.state.in_play.as_ref().and_then(OwnedCard::full).map(|live|live.id),Some(CardId::SpotWeakness));
    c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    c.state.enemies[0].intent_id=ActorIntentId::Full(FullIntentId::Attack);
    c.state.enemies[0].selected_move=ActorIntentId::Full(FullIntentId::Attack);
    c.state.enemies[0].full_damage_entries.get_mut(&FullIntentId::Attack).unwrap().base=-1;
    effects::apply_effect(&mut c.state,&prepared.effects[0]).unwrap();
    assert!(!c.state.pending.iter().any(|effect|matches!(effect.kind,EffectKind::ApplyFullPower{id:FullPowerId("strength"),..})),
        "negative current intent base does not grant Strength");
}

#[test]
fn m10_support_reckless_charge_uses_actual_nonempty_random_insertion_route(){
    let mut c=with_card(CardId::RecklessCharge,false);
    for instance in 10..12{
        let handle=CardHandle::parse(&instance.to_string()).unwrap();c.state.card_ids_issued.insert(handle);
        c.state.piles.draw.push(OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(instance),0)));
    }
    assert!(engine::validate_runtime(&c.state).is_ok());
    let stream=c.state.card_rng.as_ref().unwrap().clone();let shuffle=c.state.rng.shuffle.clone();
    let (expected,next)=rng::uniform_below(&stream,2).unwrap();
    play(&mut c,Some(TargetId(1)));
    let actual=c.state.piles.draw.iter().position(|owned|owned.full().is_some_and(|live|live.id==CardId::Dazed)).unwrap();
    assert_eq!(actual,expected as usize,"Reckless Charge producer samples current n positions");
    assert_eq!(c.state.card_rng.as_ref().unwrap(),&next);
    assert_eq!(c.state.rng.shuffle,shuffle,"generated Dazed insertion does not reuse opening/combat shuffle");
    assert!(engine::validate_runtime(&c.state).is_ok());
}
