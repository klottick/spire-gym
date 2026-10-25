use super::super::*;
use crate::{content::{CardId,CardRarity,CardType,Catalog,EncounterId},combat::model::OwnedCard};

fn combat_with(id:CardId,upgraded:bool)->Combat{
    let mut entry=super::m8_shapes::entry(EncounterId::ThreeShapes);
    entry.cards=vec![
        permanent_live_card(id,CardInstanceId(1),u32::from(upgraded)),
        permanent_live_card(CardId::Strike,CardInstanceId(2),0),
        permanent_live_card(CardId::Wound,CardInstanceId(3),0),
        permanent_live_card(CardId::Regret,CardInstanceId(4),0),
        permanent_live_card(CardId::Offering,CardInstanceId(5),0),
    ];
    entry.opening_draw=5;
    let combat=Combat::enter(entry).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat
}
fn play(combat:&mut Combat,id:CardId){
    let instance=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap().instance.0;
    let live=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap();
    let cost=cards::base_cost(&combat.state,live).expect("playable reactive-card cost");
    let energy=combat.state.player.energy;
    let target=Catalog::standard().card(id).is_some_and(|definition|definition.card_type==CardType::Attack).then(||
        TargetId(combat.state.enemies.iter().position(|enemy|enemy.enemy_id!=ActorEnemyId::Full(crate::content::EnemyId::Spiker)).unwrap() as u64+1));
    combat.step_full(&CombatAction::PlayCard{instance,target}).unwrap();
    if matches!(id,CardId::Evolve|CardId::FireBreathing|CardId::FlameBarrier|CardId::Juggernaut|CardId::Rage|CardId::Rupture){
        assert_eq!(combat.state.player.energy,energy-cost,"actual {id:?} play pays its base/current cost");
    }
}
fn literal(id:CardId,rarity:CardRarity,kind:CardType){
    let def=Catalog::standard().card(id).expect("registered card");
    assert_eq!((def.color,def.rarity,def.card_type),(crate::content::CardColor::Red,rarity,kind));
}

#[test]
fn m10_reactive_juggernaut_then_actual_defend_returns_ready_with_card_disposition(){
    let mut entry=super::m8_shapes::entry(EncounterId::ThreeShapes);
    entry.cards=vec![
        permanent_live_card(CardId::Juggernaut,CardInstanceId(1),0),
        permanent_live_card(CardId::Defend,CardInstanceId(2),0),
        permanent_live_card(CardId::Strike,CardInstanceId(3),0),
        permanent_live_card(CardId::Wound,CardInstanceId(4),0),
        permanent_live_card(CardId::Regret,CardInstanceId(5),0),
    ];
    entry.opening_draw=5;
    let mut combat=Combat::enter(entry).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    play(&mut combat,CardId::Juggernaut);
    assert_eq!(combat.state.phase,Phase::Decision,"the setup power play ends at the normal ready boundary");
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let before_rng=combat.state.card_rng.clone();
    let defend_handle=combat.state.piles.hand.iter().find(|owned|owned.full().is_some_and(|card|card.id==CardId::Defend)).unwrap().handle();
    play(&mut combat,CardId::Defend);
    assert_eq!(combat.state.phase,Phase::Decision,"real Defend plus Juggernaut resolves through Decision to ready");
    assert!(combat.state.fault.is_none(),"ordinary card Decision must not fault on a pending random-damage continuation");
    assert_eq!(combat.state.player.block,5);
    assert_ne!(combat.state.card_rng,before_rng,"Juggernaut selects exactly one live target after Defend Block");
    assert!(combat.state.piles.discard.iter().any(|owned|owned.handle()==defend_handle),"Defend follows ordinary played-card disposition");
    assert!(!combat.state.piles.hand.iter().any(|owned|owned.handle()==defend_handle));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_reactive_real_second_wind_fnp_blocks_feed_juggernaut_and_skip_the_earlier_dead_target(){
    let mut entry=super::m8_shapes::entry(EncounterId::ThreeShapes);
    entry.cards=vec![
        permanent_live_card(CardId::SecondWind,CardInstanceId(1),0),
        permanent_live_card(CardId::Defend,CardInstanceId(2),0),
        permanent_live_card(CardId::Wound,CardInstanceId(3),0),
        permanent_live_card(CardId::Regret,CardInstanceId(4),0),
        permanent_live_card(CardId::Offering,CardInstanceId(5),0),
        permanent_live_card(CardId::Strike,CardInstanceId(6),0),
    ];
    entry.opening_draw=6;
    entry.setup.player_powers=vec![
        FullPowerEntry{id:FullPowerId("feel_no_pain"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}},
        FullPowerEntry{id:FullPowerId("juggernaut"),state:PowerState{amount:5,skip_first:false,monster_source_grace:false}},
    ];
    let mut combat=Combat::enter(entry).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let (first_target_index,_)=rng::uniform_below(combat.state.card_rng.as_ref().unwrap(),combat.state.enemies.len() as u128).unwrap();
    let first_target=combat.state.enemies[first_target_index as usize].instance_id.clone();
    combat.state.enemies[first_target_index as usize].hp=1;
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let second_wind=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::SecondWind).unwrap().instance.0;
    let observed=combat.step_full_observed(&CombatAction::PlayCard{instance:second_wind,target:None});
    assert!(observed.result.is_ok(),"actual Second Wind should resolve exhaust callbacks through the ordinary ready boundary: {:?}",observed.result);
    assert_eq!(combat.state.phase,Phase::Decision);
    assert!(combat.state.fault.is_none());
    assert_eq!(combat.state.player.block,32,"four actual exhausts grant Feel No Pain 3 each plus Second Wind 5 each");
    let first_target_hits=observed.effects.iter().filter(|effect|effect.target==Some(Actor::Enemy(first_target.clone()))&&matches!(effect.kind,ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..})).collect::<Vec<_>>();
    assert_eq!(first_target_hits.len(),1,"the first captured live target dies once; later deferred Juggernaut work must select from the surviving formation");
    assert_eq!(combat.state.enemies.iter().find(|enemy|enemy.instance_id==first_target).unwrap().hp,0);
    assert!(combat.state.piles.exhaust.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::Wound));
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::Strike));
    assert!(combat.state.piles.discard.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::SecondWind));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_reactive_evolve_red(){
    let id=CardId::Evolve;literal(id,CardRarity::Uncommon,CardType::Power);
    let mut combat=combat_with(id,false);play(&mut combat,id);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="evolve"&&power.state.amount==1));
}

#[test]
fn m10_reactive_fire_breathing_red(){
    let id=CardId::FireBreathing;literal(id,CardRarity::Uncommon,CardType::Power);
    let mut combat=combat_with(id,false);play(&mut combat,id);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="fire_breathing"&&power.state.amount==6));
}
#[test]
fn m10_reactive_flame_barrier_red(){
    let id=CardId::FlameBarrier;literal(id,CardRarity::Uncommon,CardType::Skill);
    let mut combat=combat_with(id,false);play(&mut combat,id);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="flame_barrier"&&power.state.amount==4));
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|live|live.id==id)));
}

#[test]
fn m10_reactive_flame_barrier_uses_dexterity_and_frail_block_modifiers(){
    let mut combat=combat_with(CardId::FlameBarrier,false);
    combat.enqueue_native_effects(vec![
        Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("dexterity"),amount:2}},
        Effect{source:Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("frail"),amount:1}},
    ],None).unwrap();
    play(&mut combat,CardId::FlameBarrier);
    assert_eq!(combat.state.player.block,10,"12 base Block +2 Dexterity, then 25% Frail, floors to 10");
}
#[test]
fn m10_reactive_juggernaut_red(){
    let id=CardId::Juggernaut;literal(id,CardRarity::Rare,CardType::Power);
    let mut combat=combat_with(id,false);play(&mut combat,id);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="juggernaut"&&power.state.amount==5));
}
#[test]
fn m10_reactive_rage_red(){
    let id=CardId::Rage;literal(id,CardRarity::Uncommon,CardType::Skill);
    let mut combat=combat_with(id,false);play(&mut combat,id);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="rage"&&power.state.amount==3));
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|live|live.id==id)));
}
#[test]
fn m10_reactive_rupture_red(){
    let id=CardId::Rupture;literal(id,CardRarity::Uncommon,CardType::Power);
    let mut combat=combat_with(id,false);play(&mut combat,id);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="rupture"&&power.state.amount==1));
}

#[test]
fn m10_reactive_public_values_and_costs_match_each_base_and_upgraded_card(){
    for (id,cost,damage,block,magic) in [
        (CardId::Evolve,1,None,None,Some(1)),(CardId::FireBreathing,1,None,None,Some(6)),
        (CardId::FlameBarrier,2,None,Some(12),Some(4)),(CardId::Juggernaut,2,None,None,Some(5)),
        (CardId::Rage,0,None,None,Some(3)),(CardId::Rupture,1,None,None,Some(1)),
    ]{
        let combat=combat_with(id,false);let live=card_state::permanent(id,CardInstanceId(99),0);
        let values=cards::ironclad::rules::public_values(&live).expect("base values");
        assert_eq!((values.base_damage,values.base_block,values.magic_number),(damage,block,magic),"{id:?}");
        assert_eq!(cards::base_cost(&combat.state,&live),Some(cost),"base cost {id:?}");
        let generated=card_state::generated(id,CardInstanceId(101),0).expect("each supported reactive card has a real generated constructor");
        assert_eq!(cards::ironclad::rules::public_values(&generated),Some(values),"generated card public values {id:?}");
    }
    for (id,cost,damage,block,magic) in [
        (CardId::Evolve,1,None,None,Some(2)),(CardId::FireBreathing,1,None,None,Some(10)),
        (CardId::FlameBarrier,2,None,Some(16),Some(6)),(CardId::Juggernaut,2,None,None,Some(7)),
        (CardId::Rage,0,None,None,Some(5)),(CardId::Rupture,1,None,None,Some(2)),
    ]{
        let combat=combat_with(id,true);let live=card_state::permanent(id,CardInstanceId(99),1);
        let values=cards::ironclad::rules::public_values(&live).expect("upgraded values");
        assert_eq!((values.base_damage,values.base_block,values.magic_number),(damage,block,magic),"{id:?}");
        assert_eq!(cards::base_cost(&combat.state,&live),Some(cost),"upgraded cost {id:?}");
        let generated=card_state::generated(id,CardInstanceId(102),1).expect("each upgraded reactive card has a real generated constructor");
        assert_eq!(cards::ironclad::rules::public_values(&generated),Some(values),"upgraded generated card public values {id:?}");
    }
}

#[test]
fn m10_reactive_upgraded_cards_play_real_upgraded_payloads_and_skills_discard_normally(){
    for (id,power,amount) in [
        (CardId::Evolve,true,2),(CardId::FireBreathing,true,10),(CardId::FlameBarrier,false,6),
        (CardId::Juggernaut,true,7),(CardId::Rage,false,5),(CardId::Rupture,true,2),
    ]{
        let mut combat=combat_with(id,true);play(&mut combat,id);
        let installed=combat.state.player_full_powers.iter().find(|entry|entry.state.amount==amount&&match id{
            CardId::Evolve=>entry.id.0=="evolve",CardId::FireBreathing=>entry.id.0=="fire_breathing",
            CardId::FlameBarrier=>entry.id.0=="flame_barrier",CardId::Juggernaut=>entry.id.0=="juggernaut",
            CardId::Rage=>entry.id.0=="rage",CardId::Rupture=>entry.id.0=="rupture",_=>false,
        });
        assert!(installed.is_some(),"{id:?} upgraded payload must install its upgraded amount");
        if !power{assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.id==id)),"upgraded Skill returns through standard discard");}
        let live=permanent_live_card(id,CardInstanceId(99),1).stat_equivalent_copy(CardInstanceId(100));
        assert_eq!(cards::ironclad::rules::public_values(&live).unwrap().magic_number,
            cards::ironclad::rules::public_values(&permanent_live_card(id,CardInstanceId(99),1)).unwrap().magic_number,
            "native stat-equivalent copy preserves public upgrade values for {id:?}");
    }
}

#[test]
fn m10_reactive_actual_status_and_curse_draw_trigger_only_the_qualified_powers(){
    let mut combat=combat_with(CardId::Evolve,false);
    play(&mut combat,CardId::Evolve);
    let wound_index=combat.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Wound)).unwrap();
    let regret_index=combat.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Regret)).unwrap();
    let (low,high)=if wound_index<regret_index{(wound_index,regret_index)}else{(regret_index,wound_index)};
    let high_card=combat.state.piles.hand.remove(high);let low_card=combat.state.piles.hand.remove(low);
    let (wound,regret)=if high_card.full().unwrap().id==CardId::Wound{(high_card,low_card)}else{(low_card,high_card)};
    combat.state.piles.draw.extend([regret,wound]);
    let before=combat.state.piles.hand.len();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    assert_eq!(combat.state.piles.hand.len(),before+2,"actual Status draw recursively requests one ordinary draw");
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::Wound));
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.id==CardId::Regret).count(),1,"Curse draw is not Evolve-eligible but remains an ordinary drawn card");

    let mut fire=combat_with(CardId::FireBreathing,false);play(&mut fire,CardId::FireBreathing);
    let hp=fire.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    let curse=fire.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Regret)).unwrap();
    let curse=fire.state.piles.hand.remove(curse);fire.state.piles.draw.push(curse);
    fire.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    assert_eq!(fire.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),hp.iter().map(|hp|hp-6).collect::<Vec<_>>(),"Fire Breathing reacts once to actual Curse movement across all enemies");

    let mut created=combat_with(CardId::Evolve,false);play(&mut created,CardId::Evolve);
    let hand_before=created.state.piles.hand.len();
    created.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::CreateCard{id:CardId::Wound,destination:CardDestination::Hand,upgrades:0}}],None).unwrap();
    assert_eq!(created.state.piles.hand.len(),hand_before+1,"Status creation in hand is not an actual draw and does not recurse");
    let draws=created.state.piles.draw.len();
    created.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("no_draw"),amount:1}},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).unwrap();
    assert_eq!(created.state.piles.draw.len(),draws,"No Draw prevents actual movement and both reactive powers");
}

#[test]
fn m10_reactive_evolve_fire_breathing_both_install_orders_keep_draw2_and_void_frontier(){
    for evolve_first in [true,false]{
        let first=if evolve_first{CardId::Evolve}else{CardId::FireBreathing};
        let second=if evolve_first{CardId::FireBreathing}else{CardId::Evolve};
        let mut combat=combat_with(first,false);
        let replaced=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Strike).unwrap().instance;
        let replacement=permanent_live_card(second,replaced,0);
        *combat.state.piles.hand.iter_mut().find(|owned|owned.handle()==card_state::handle(replaced)).unwrap().full_mut().unwrap()=replacement;
        assert!(engine::validate_runtime(&combat.state).is_ok());
        play(&mut combat,first);play(&mut combat,second);
        let new_cards=[(CardId::Strike,804),(CardId::Defend,801),(CardId::Void,802),(CardId::Wound,803)];
        combat.state.piles.draw.clear();
        for (id,raw) in new_cards{
            let card=permanent_live_card(id,CardInstanceId(raw),0);combat.state.card_ids_issued.insert(card_state::handle(card.instance));combat.state.piles.draw.push(OwnedCard::Full(card));
        }
        assert!(engine::validate_runtime(&combat.state).is_ok(),"registered Draw2 frontier fixture before action");
        let initial_hp=combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();let energy=combat.state.player.energy;
        combat.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(2)});
        let mut dispatch=Vec::new();
        while let Some(effect)=combat.state.pending.front().cloned(){dispatch.push(effect);engine::dispatch_one(&mut combat.state);assert!(combat.state.fault.is_none(),"valid callback frontier dispatches without fault");}
        assert!(engine::validate_runtime(&combat.state).is_ok(),"runtime remains valid after Draw2, child draw and callback frontier");
        assert!(combat.state.public.revealed_order.iter().any(|handle|handle.as_str()=="803")&&combat.state.public.revealed_order.iter().any(|handle|handle.as_str()=="802")&&combat.state.public.revealed_order.iter().any(|handle|handle.as_str()=="801"),"two requested cards and Evolve's queued child draw all cross the ordinary draw owner");
        assert_eq!(combat.state.player.energy,energy-1,"actual Void callback loses energy after ordinary Draw2 completes");
        assert_eq!(combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),initial_hp.iter().map(|hp|hp-12).collect::<Vec<_>>(),"both Status draws trigger one Fire Breathing action apiece");
        let draw_one=dispatch.iter().enumerate().filter_map(|(index,e)|(e.kind==EffectKind::DrawOne).then_some(index)).collect::<Vec<_>>();
        let fire=dispatch.iter().position(|e|matches!(e.kind,EffectKind::Damage(DamagePayload::Thorns(6)))).unwrap();
        let void_loss=dispatch.iter().position(|e|e.kind==EffectKind::LoseEnergy(1)).unwrap();
        assert!(draw_one.len()>=4&&draw_one[0]<draw_one[1]&&draw_one[1]<void_loss,"both requested Draw2 cards precede Void's energy callback and Evolve's child draws");
        if evolve_first{assert!(draw_one[2]<fire,"Evolve's earlier-installed child-draw callback precedes Fire Breathing");}
        else{assert!(fire<draw_one[2],"Fire Breathing's earlier-installed damage callback precedes Evolve's child draw");}
        assert_eq!(dispatch.iter().filter(|e|matches!(e.kind,EffectKind::Damage(DamagePayload::Thorns(6)))).count(),6,"both Status callbacks damage all three live enemies");
        let third_draw=draw_one[2];
        if evolve_first{assert!(third_draw<fire,"earlier-installed Evolve child draw resolves before Fire Breathing callback");}
        else{assert!(fire<third_draw,"earlier-installed Fire Breathing callback resolves before Evolve child draw");}
    }
}

#[test]
fn m10_reactive_full_hand_draw_does_not_trigger_on_a_card_that_never_moved(){
    let mut combat=combat_with(CardId::Evolve,false);play(&mut combat,CardId::Evolve);
    while combat.state.piles.hand.len()<10{
        let instance=CardInstanceId(700+combat.state.piles.hand.len() as u64);
        let card=permanent_live_card(CardId::Strike,instance,0);
        combat.state.card_ids_issued.insert(card_state::handle(card.instance));
        combat.state.piles.hand.push(OwnedCard::Full(card));
    }
    let wound=card_state::generated(CardId::Wound,CardInstanceId(901),0).expect("Status constructor");
    combat.state.card_ids_issued.insert(card_state::handle(wound.instance));
    combat.state.piles.draw.push(OwnedCard::Full(wound));
    assert!(engine::validate_runtime(&combat.state).is_ok(),"full-hand and draw-pile fixture uses issued owned IDs before public native command");
    let before_draw=combat.state.piles.draw.len();
    let before_hand=combat.state.piles.hand.len();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    assert_eq!(combat.state.piles.hand.len(),before_hand,"full-hand draw leaves the Status in the draw pile");
    assert_eq!(combat.state.piles.draw.len(),before_draw,"a card that did not move cannot trigger Evolve");
}

#[test]
fn m10_reactive_confused_randomizes_generated_free_play_card_and_clears_free_play(){
    let mut combat=combat_with(CardId::Evolve,false);
    let generated=card_state::generated(CardId::BloodForBlood,CardInstanceId(902),0).unwrap();
    combat.state.card_ids_issued.insert(card_state::handle(generated.instance));
    let mut generated=generated;generated.local.free_play=true;
    combat.state.piles.draw.push(OwnedCard::Full(generated));
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("confused"),amount:1}}],None).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let rng=combat.state.card_rng.clone();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let drawn=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.instance.0==902).unwrap();
    assert!(drawn.local.randomized_cost.is_some_and(|cost|(0..4).contains(&cost)));
    assert!(!drawn.local.free_play,"Confused clears free-to-play on a generated ordinary-cost card");
    assert_ne!(combat.state.card_rng,rng,"the supported non-X draw consumes the scoped card stream");
}

#[test]
fn m10_reactive_confused_does_not_randomize_x_cost_or_unplayable_status_draws(){
    let mut combat=combat_with(CardId::Evolve,false);
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    combat.state.piles.draw.clear();
    for (id,raw) in [(CardId::Whirlwind,805),(CardId::Dazed,806)]{
        let card=card_state::generated(id,CardInstanceId(raw),0).unwrap();combat.state.card_ids_issued.insert(card_state::handle(card.instance));combat.state.piles.draw.push(OwnedCard::Full(card));
    }
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let rng=combat.state.card_rng.clone();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    for raw in [805,806]{let card=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.instance.0==raw).unwrap();assert_eq!(card.local.randomized_cost,None);}
    assert_eq!(combat.state.card_rng,rng,"source-qualified X and unplayable status draws do not consume Confused RNG");
}

#[test]
fn m10_reactive_flame_barrier_triggers_on_qualifying_enemy_owned_damage_even_when_blocked(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::OrbWalker);
    entry.cards=vec![permanent_live_card(CardId::FlameBarrier,CardInstanceId(1),0)];entry.opening_draw=1;
    let mut combat=Combat::enter(entry).unwrap();
    let laser=ActorIntentId::Full(FullIntentId::from_name("orb_laser"));
    combat.state.enemies[0].intent_id=laser.clone();combat.state.enemies[0].selected_move=laser.clone();combat.state.enemies[0].move_history=vec![laser.clone()];
    effects::refresh_enemy_values(&mut combat.state).unwrap();assert!(engine::validate_runtime(&combat.state).is_ok());
    play(&mut combat,CardId::FlameBarrier);
    let attacker=combat.state.enemies[0].instance_id.clone();let hp=combat.state.enemies[0].hp;let player_hp=combat.state.player.hp;
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(attacker.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::from_name("orb_laser")))}],None).unwrap();
    assert_eq!(combat.state.player.hp,player_hp,"Flame Barrier still reacts to an actual attack absorbed by Block");
    assert_eq!(combat.state.enemies[0].hp,hp-4,"the actual attacking actor receives one ordinary retaliation");
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(attacker.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::from_name("orb_laser")))}],None).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp-8,"each separate actual attack event retaliates once before the next player turn begins");
    let hp=combat.state.enemies[0].hp;
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(attacker.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Orb(1))}],None).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp,"native Orb is Defect Lightning's original THORNS lane, not a fourth original DamageInfo type");
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(attacker.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(0))}],None).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp-4,"foreign-owner NORMAL callback triggers even when the attempted damage is zero");
    let hp=combat.state.enemies[0].hp;
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp,"self-owner damage cannot trigger Flame Barrier");
    let hp=combat.state.enemies[0].hp;
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp,"player HP_LOSS cannot trigger Flame Barrier");
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::TurnStartedPreDraw)}],None).unwrap();
    assert!(!combat.state.player_full_powers.iter().any(|power|power.id.0=="flame_barrier"),"Flame Barrier survives through this turn and is removed at next player start");
}

#[test]
fn m10_reactive_committed_attack_retains_flame_callback_child_at_existing_effect_budget_fault(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::OrbWalker);
    entry.cards=vec![permanent_live_card(CardId::FlameBarrier,CardInstanceId(1),0)];entry.opening_draw=1;
    let mut combat=Combat::enter(entry).unwrap();let laser=ActorIntentId::Full(FullIntentId::from_name("orb_laser"));
    combat.state.enemies[0].intent_id=laser.clone();combat.state.enemies[0].selected_move=laser.clone();combat.state.enemies[0].move_history=vec![laser];
    effects::refresh_enemy_values(&mut combat.state).unwrap();play(&mut combat,CardId::FlameBarrier);
    let attacker=combat.state.enemies[0].instance_id.clone();let block=combat.state.player.block;let player=combat.state.player.hp;
    combat.state.pending.clear();combat.state.current=None;
    combat.state.resolved_effects=99_998;
    combat.state.pending.push_back(Effect{source:Source::Actor(Actor::Enemy(attacker.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::from_name("orb_laser")))});
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.resolved_effects,99_999);
    assert_eq!(combat.state.player.block,block-10,"entered real attack committed through the shared damage path before the callback-work budget fault");
    assert!(combat.state.pending.iter().any(|effect|effect.target==Some(Actor::Enemy(attacker.clone()))&&effect.kind==EffectKind::Damage(DamagePayload::Thorns(4))),"reaction child is durably queued before next budget checkpoint");
    assert!(combat.state.pending.iter().any(|effect|effect.kind==EffectKind::Hook(Hook::Damage)));
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(combat.state.player.hp,player);
    let fault=combat.state.fault.clone();let frozen=combat.state.clone();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());assert_eq!(combat.state,frozen);
    assert_eq!(combat.state.fault,fault,"accepted callback child cannot be retried or replace the retained first fault");
}

#[test]
fn m10_runic_cube_retains_committed_self_loss_and_queued_draw_at_existing_callback_guard_fault(){
    use crate::{content::{Character,RelicId},items::CombatItemContext,run::{CreationProfile,Run,RunConfig,RunRng}};
    let mut run=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),
        RunRng::new([204;32],crate::combat::rng::Partition::Test,0)).unwrap();
    crate::items::acquire_relic(run.state_mut_for_test(),RelicId::RunicCube).unwrap();
    let entry=super::m8_shapes::single(crate::content::EnemyId::OrbWalker);
    let mut owner=Some(CombatItemContext::take_from_run(run.state_mut_for_test()));
    let mut combat=Combat::enter_with_item_context(entry,&mut owner).unwrap();assert!(owner.is_none());
    let card_rng=combat.state.card_rng.clone();let shuffle_rng=combat.state.rng.shuffle.clone();
    combat.state.resolved_effects=99_997;
    combat.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::Damage(DamagePayload::HpLoss(1))});
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.resolved_effects,99_998);
    assert_eq!(combat.state.player.hp,combat.state.player.max_hp-1,"positive self-loss committed before its queued callback child");
    assert_eq!(combat.state.pending.front().unwrap().kind,EffectKind::Hook(Hook::Damage));
    assert_eq!(combat.state.pending.get(1).unwrap().kind,EffectKind::Draw(1),"the accepted Runic Cube work follows ordinary Damage notification in the native queue");
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.resolved_effects,99_999);
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(combat.state.current.as_ref().unwrap().kind,EffectKind::Draw(1),"the interrupted ordinary Cube draw remains the current owned callback effect");
    assert!(combat.item_context().unwrap().owns_relic(RelicId::RunicCube));
    assert_eq!(combat.state.card_rng,card_rng,"draw was retained but not executed/retried");
    assert_eq!(combat.state.rng.shuffle,shuffle_rng);
    let first_fault=combat.state.fault.clone();let frozen=combat.state.clone();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());assert_eq!(combat.state,frozen);
    assert_eq!(combat.state.fault,first_fault,"accepted callback child cannot replace the original first fault");
}

#[test]
fn m10_runic_cube_lethal_self_loss_leaves_queued_draw_to_existing_terminal_cancellation(){
    use crate::{content::{Character,RelicId},items::CombatItemContext,run::{CreationProfile,Run,RunConfig,RunRng}};
    let mut run=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),
        RunRng::new([205;32],crate::combat::rng::Partition::Test,0)).unwrap();
    crate::items::acquire_relic(run.state_mut_for_test(),RelicId::RunicCube).unwrap();
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::OrbWalker);
    entry.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(2),0));
    entry.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(3),0));
    entry.setup.player.hp=1;
    let mut owner=Some(CombatItemContext::take_from_run(run.state_mut_for_test()));
    let mut combat=Combat::enter_with_item_context(entry,&mut owner).unwrap();assert!(owner.is_none());
    let hand=combat.state.piles.hand.len();let draw=combat.state.piles.draw.len();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    assert_eq!(combat.state.outcome,Some(Outcome::Death));
    assert_eq!(combat.state.player.hp,0);
    assert_eq!(combat.state.piles.hand.len(),hand,"the lethal Cube draw is not an unearned rescue");
    assert_eq!(combat.state.piles.draw.len(),draw,"the existing terminal owner cancels the queued draw");
    assert!(combat.state.pending.is_empty());
    assert!(combat.item_context().unwrap().owns_relic(RelicId::RunicCube));
}

#[test]
fn m10_reactive_juggernaut_uses_positive_gain_notification_even_at_block_cap(){
    let mut combat=combat_with(CardId::Juggernaut,false);play(&mut combat,CardId::Juggernaut);
    let before=combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    let rng=combat.state.card_rng.clone();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(5)}],None).unwrap();
    let after=combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    assert_eq!(before.iter().zip(&after).filter(|(before,after)|before!=after).count(),1,"one positive Block delta produces one random target");
    assert_eq!(before.iter().zip(&after).filter_map(|(before,after)|(*before-*after>0).then_some(*before-*after)).collect::<Vec<_>>(),vec![5]);
    assert_ne!(combat.state.card_rng,rng,"the player-owned scoped card stream advances only for an actual eligible trigger");
    let after_rng=combat.state.card_rng.clone();combat.state.player.block=999;
    let enemy_hp=combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(5)}],None).unwrap();
    assert_ne!(combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),enemy_hp,"positive onGainedBlock notification triggers before the 999 Block cap");
    assert_ne!(combat.state.card_rng,after_rng,"the positive callback consumes one scoped target selection even when net Block delta is zero");
}

#[test]
fn m10_reactive_juggernaut_selects_after_previously_queued_effects_resolve(){
    let mut combat=combat_with(CardId::Juggernaut,false);play(&mut combat,CardId::Juggernaut);
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let soon_dead=combat.state.enemies.iter().find(|enemy|enemy.enemy_id!=ActorEnemyId::Full(crate::content::EnemyId::Spiker)).unwrap().instance_id.clone();
    let survivors=combat.state.enemies.iter().filter(|enemy|enemy.instance_id!=soon_dead&&enemy.life==ActorLife::Active).map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
    combat.state.enemies.iter_mut().find(|enemy|enemy.instance_id==soon_dead).unwrap().hp=1;
    let earlier=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(soon_dead.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(1))};
    combat.state.phase=Phase::PlayerAction;
    combat.state.pending.clear();
    combat.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(3)});
    combat.state.pending.push_back(earlier.clone());
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.pending.front(),Some(&earlier),"random-target action is queued behind pre-existing work");
    assert!(matches!(combat.state.pending.back().map(|effect|&effect.kind),Some(EffectKind::JuggernautRandomDamage(5))));
    combat.state.player_full_powers.iter_mut().find(|power|power.id.0=="juggernaut").unwrap().state.amount=1;
    assert!(engine::validate_runtime(&combat.state).is_ok(),"runtime remains canonical with queued deferred action: {:?}",engine::validate_runtime(&combat.state));
    engine::dispatch_one(&mut combat.state);
    assert!(!combat.state.enemies.iter().any(|enemy|enemy.instance_id==soon_dead&&enemy.hp>0));
    while !matches!(combat.state.pending.front().map(|effect|&effect.kind),Some(EffectKind::JuggernautRandomDamage(_)))&&!combat.state.pending.is_empty(){engine::dispatch_one(&mut combat.state);}
    engine::dispatch_one(&mut combat.state);
    let chosen=combat.state.pending.front().and_then(|effect|match &effect.kind{EffectKind::Damage(DamagePayload::Thorns(5))=>match &effect.target{Some(Actor::Enemy(id))=>Some(id.clone()),_=>None},_=>None}).expect("deferred random action emits damage against a living target");
    assert!(matches!(combat.state.pending.front().map(|effect|&effect.kind),Some(EffectKind::Damage(DamagePayload::Thorns(5)))),"queued DamageInfo retains the originally captured amount if its power changes before dispatch");
    assert!(survivors.contains(&chosen)&&combat.state.enemies.iter().any(|enemy|enemy.instance_id==chosen&&enemy.hp>0),"target selection uses a real surviving Three Shapes actor after earlier queued lethal work");
    assert!(engine::validate_runtime(&combat.state).is_ok(),"terminal no-live-target checkpoint: {:?}",engine::validate_runtime(&combat.state));
}

#[test]
fn m10_reactive_juggernaut_rng_fault_retains_random_action_and_cursor_without_retry(){
    let mut combat=combat_with(CardId::Juggernaut,false);play(&mut combat,CardId::Juggernaut);
    let stream=combat.state.card_rng.as_ref().unwrap();combat.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],1_u128<<64).unwrap());
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let cursor=combat.state.card_rng.clone();
    assert!(combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(3)}],None).is_err());
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    assert_eq!(combat.state.current.as_ref().unwrap().kind,EffectKind::JuggernautRandomDamage(5),"the failed deferred dispatch retains its exact owning effect");
    assert_eq!(combat.state.card_rng,cursor,"failed scoped selection retains the exhausted cursor");
    let first_fault=combat.state.fault.clone().unwrap();let frozen=combat.snapshot();assert!(combat.step_full(&CombatAction::EndTurn).is_err());
    assert_eq!(combat.snapshot(),frozen,"retained random-selection fault cannot retry or advance its cursor");
    assert_eq!(combat.state.fault.as_ref(),Some(&first_fault),"no retry replaces the original dispatch fault");
}

#[test]
fn m10_reactive_juggernaut_random_action_with_no_living_target_consumes_no_rng(){
    let mut combat=combat_with(CardId::Juggernaut,false);play(&mut combat,CardId::Juggernaut);
    for enemy in &mut combat.state.enemies{enemy.hp=0;enemy.life=ActorLife::Replaced;}
    combat.state.phase=Phase::Terminal;combat.state.outcome=Some(Outcome::Victory);
    combat.state.hp_before_healing=Some(combat.state.player.hp);combat.state.hp_after_cleanup=Some(combat.state.player.hp);
    let rng=combat.state.card_rng.clone();
    assert!(engine::validate_runtime(&combat.state).is_ok(),"no-target pending terminal frontier: {:?}",engine::validate_runtime(&combat.state));
    effects::apply_effect(&mut combat.state,&Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(5)}).unwrap();
    assert!(combat.state.fault.is_none());assert_eq!(combat.state.card_rng,rng);assert!(combat.state.pending.is_empty());
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_reactive_juggernaut_random_effect_public_validation_is_atomic_and_finite(){
    let mut combat=combat_with(CardId::Juggernaut,false);play(&mut combat,CardId::Juggernaut);
    let enemy=Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    let invalid=vec![
        Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(-1)},
        Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(0)},
        Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(1516)},
        Effect{source:Source::None,target:None,kind:EffectKind::JuggernautRandomDamage(5)},
        Effect{source:Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),target:None,kind:EffectKind::JuggernautRandomDamage(5)},
        Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::JuggernautRandomDamage(5)},
        Effect{source:Source::Actor(Actor::Player),target:Some(enemy),kind:EffectKind::JuggernautRandomDamage(5)},
    ];
    for effect in invalid{let before=combat.snapshot();let description=format!("{effect:?}");assert!(combat.enqueue_native_effects(vec![effect],None).is_err(),"validator rejects {description}");assert_eq!(combat.snapshot(),before,"invalid public random-damage effect rejects atomically");}
    let mut legacy=Combat::new(&super::whole_combat::scenario(EnemyKind::Cultist)).unwrap();
    let before=legacy.snapshot();assert!(legacy.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(5)}],None).is_err());assert_eq!(legacy.snapshot(),before,"the M10 continuation is not admitted by the legacy combat API");
    legacy.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(5)});
    assert_eq!(legacy.try_snapshot_v2(),Err(RestoreError::Compatibility),"the public legacy snapshot boundary rejects an unrepresentable M10 continuation");
    let rejected=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||super::differential::encode_checkpoint(&legacy)))
        .expect_err("the actual M9 v1 checkpoint encoder must reject a pending M10 random action");
    let message=rejected.downcast_ref::<String>().map(String::as_str).or_else(||rejected.downcast_ref::<&str>().copied()).unwrap_or("");
    assert!(message.contains("M9 v1 cannot encode M10 reactive work"),"specific v1 boundary error: {message}");
}

#[test]
fn m10_reactive_rage_triggers_on_actual_attack_card_use_and_expires_at_player_end_turn(){
    let mut combat=combat_with(CardId::Rage,true);play(&mut combat,CardId::Rage);
    assert_eq!(combat.state.player.block,0,"Rage power play itself is not an Attack");
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="rage").unwrap().state.amount,5);
    play(&mut combat,CardId::Strike);
    assert_eq!(combat.state.player.block,5,"upgraded Rage triggers once at CardPlayed, not per damage callback; powers={:?}",combat.state.player_full_powers);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="rage"));
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::PlayerEndTurn)}],None).unwrap();
    assert!(!combat.state.player_full_powers.iter().any(|power|power.id.0=="rage"),"Rage is removed at the actual player end-turn power checkpoint");
}

#[test]
fn m10_reactive_rage_pummel_block_follows_the_real_multihit_payload_once(){
    let mut combat=combat_with(CardId::Rage,false);play(&mut combat,CardId::Rage);
    let pummel_instance=combat.state.piles.hand.iter().find_map(OwnedCard::full).unwrap().instance;
    let pummel=card_state::permanent(CardId::Pummel,pummel_instance,0);
    *combat.state.piles.hand.iter_mut().find(|owned|owned.handle()==card_state::handle(pummel_instance)).unwrap().full_mut().unwrap()=pummel;
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let target=combat.state.enemies.iter().position(|enemy|enemy.enemy_id!=ActorEnemyId::Full(crate::content::EnemyId::Spiker)).unwrap();
    let hp=combat.state.enemies[target].hp;
    let observed=combat.step_full_observed(&CombatAction::PlayCard{instance:pummel_instance.0,target:Some(TargetId(target as u64+1))});
    assert!(observed.result.is_ok());
    let hits=observed.effects.iter().enumerate().filter(|(_,effect)|effect.target==Some(Actor::Enemy(combat.state.enemies[target].instance_id.clone()))&&matches!(effect.kind,ResolvedEffectKind::Damage{..})).map(|(i,_)|i).collect::<Vec<_>>();
    let rage_block=observed.effects.iter().position(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,ResolvedEffectKind::Block{after:3,..})).unwrap();
    assert_eq!(hits.len(),4,"actual base Pummel resolves four hits");
    assert!(hits.iter().all(|hit|*hit<rage_block),"Rage's one raw Block notification follows the whole multi-hit payload");
    assert_eq!(combat.state.player.block,3);
    assert!(combat.state.enemies[target].hp<hp);
}

#[test]
fn m10_reactive_rage_expires_at_canonical_time_eater_twelfth_card_end(){
    let mut entry=super::m8_shapes::entry(EncounterId::TimeEater);
    entry.cards=std::iter::once(permanent_live_card(CardId::Rage,CardInstanceId(1),0))
        .chain((2..=12).map(|id|permanent_live_card(CardId::Strike,CardInstanceId(id),0))).collect();
    entry.opening_draw=10;
    let mut combat=Combat::enter(entry).unwrap();assert!(engine::validate_runtime(&combat.state).is_ok());
    for card in combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut){if card.id==CardId::Strike{card.local.free_play=true;}}
    assert!(engine::validate_runtime(&combat.state).is_ok());
    play(&mut combat,CardId::Rage);
    for _ in 0..9{play(&mut combat,CardId::Strike);}
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|power.id.0=="time_warp").unwrap().state.amount,10);
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|power.id.0=="time_warp").unwrap().state.amount,10,"drawing remaining attacks does not advance Time Warp's card-use counter");
    play(&mut combat,CardId::Strike);
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|power.id.0=="time_warp").unwrap().state.amount,11);
    play(&mut combat,CardId::Strike);
    assert_eq!(combat.state.turn,2,"canonical Time Eater's real twelfth played card enters its end-turn route");
    assert!(!combat.state.player_full_powers.iter().any(|power|power.id.0=="rage"),"Rage expires at Time Eater's actual PlayerEndTurn checkpoint");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_reactive_rupture_qualifies_player_owned_hp_loss_from_cards_and_powers(){
    let mut combat=combat_with(CardId::Rupture,false);play(&mut combat,CardId::Rupture);
    let hp=combat.state.player.hp;
    play(&mut combat,CardId::Offering);
    assert_eq!(combat.state.player.hp,hp-6);
    assert!(combat.state.player_full_powers.iter().any(|power|power.id.0=="strength"&&power.state.amount==1),"a real card-owned positive HP_LOSS event grants Strength once");
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,2,"player-owned Actor/power HP_LOSS qualifies just like card-attributed HP_LOSS");
    let enemy=combat.state.enemies[0].instance_id.clone();
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(enemy)),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Thorns(1))}],None).unwrap();
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,2,"enemy-owned THORNS does not match the player's DamageInfo owner");
}

#[test]
fn m10_reactive_rupture_triggers_on_actual_brutality_player_power_hp_loss(){
    let mut entry=super::m8_shapes::entry(EncounterId::OrbWalker);
    entry.cards=vec![permanent_live_card(CardId::Rupture,CardInstanceId(1),0),permanent_live_card(CardId::Brutality,CardInstanceId(2),0),permanent_live_card(CardId::Defend,CardInstanceId(3),0)];
    entry.opening_draw=2;
    let mut combat=Combat::enter(entry).unwrap();assert!(engine::validate_runtime(&combat.state).is_ok());
    play(&mut combat,CardId::Rupture);play(&mut combat,CardId::Brutality);
    let hp=combat.state.player.hp;
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::TurnStarted)}],None).unwrap();
    assert_eq!(combat.state.player.hp,hp-1,"actual Brutality loses one HP through its player-owned power callback");
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,1,
        "Rupture has no damage-type exclusion for a positive player-owned power HP-loss callback");
}

#[test]
fn m10_reactive_rupture_qualifies_held_burn_self_thorns_only_when_hp_is_lost(){
    for (block,expect_strength) in [(0,1),(2,0)]{
        let mut entry=super::m8_shapes::entry(EncounterId::OrbWalker);
        entry.cards=vec![permanent_live_card(CardId::Rupture,CardInstanceId(1),0),permanent_live_card(CardId::Burn,CardInstanceId(2),0)];entry.opening_draw=2;
        let mut combat=Combat::enter(entry).unwrap();assert!(engine::validate_runtime(&combat.state).is_ok());play(&mut combat,CardId::Rupture);
        combat.state.player.block=block;assert!(engine::validate_runtime(&combat.state).is_ok());
        let hp=combat.state.player.hp;let held_burn=combat.state.piles.hand.iter().find_map(OwnedCard::full).filter(|card|card.id==CardId::Burn).unwrap().instance;
        let work=status::end_turn_work(&combat.state);assert!(work.iter().any(|effect|effect.source==Source::Card(card_state::handle(held_burn))&&effect.kind==EffectKind::Damage(DamagePayload::Thorns(2))));
        combat.enqueue_native_effects(work,None).unwrap();
        assert_eq!(combat.state.player.hp,hp-(2-block).max(0),"held Burn uses its actual card-owned THORNS payload");
        assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").map_or(0,|power|power.state.amount),expect_strength,"Rupture requires positive post-Block HP loss");
        let source=Source::Card(card_state::handle(held_burn));
        combat.enqueue_native_effects(vec![Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Thorns(0))}],None).unwrap();
        assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").map_or(0,|power|power.state.amount),expect_strength,"zero self-owned THORNS HP loss does not trigger Rupture");
        assert!(engine::validate_runtime(&combat.state).is_ok());
    }
}

#[test]
fn m10_reactive_rupture_qualifies_actual_combust_power_hp_loss(){
    let mut entry=super::m8_shapes::entry(EncounterId::OrbWalker);
    entry.cards=vec![permanent_live_card(CardId::Rupture,CardInstanceId(1),0),permanent_live_card(CardId::Combust,CardInstanceId(2),0),permanent_live_card(CardId::Defend,CardInstanceId(3),0)];entry.opening_draw=2;
    let mut combat=Combat::enter(entry).unwrap();assert!(engine::validate_runtime(&combat.state).is_ok());play(&mut combat,CardId::Rupture);play(&mut combat,CardId::Combust);
    let hp=combat.state.player.hp;combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::PlayerEndTurn)}],None).unwrap();
    assert_eq!(combat.state.player.hp,hp-1,"Combust's real player-owned HP_LOSS is retained independently from its enemy THORNS");
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,1);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}
