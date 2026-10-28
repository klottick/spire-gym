use crate::{run::*,content::{CardId,Character,EncounterId},combat::{CombatAction,TargetId}};

#[test]
fn m3_authored_profile_keeps_existing_resources_and_rejects_task17_fatal_cards(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:vec![CardInstance{ misc:0,instance:1,id:CardId::Strike,upgrades:0}],relics:vec![],
        potions:vec![None;3],relic_counters:Default::default()};
    assert!(Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start.clone(),
        RunRng::new([8;32],crate::combat::rng::Partition::Test,0)).is_ok(),"the prior M3 authored profile remains available");
    for id in [CardId::Feed,CardId::Reaper] {
        let mut new_card=start.clone();new_card.deck[0].id=id;
        assert!(matches!(Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},new_card,
            RunRng::new([8;32],crate::combat::rng::Partition::Test,0)),Err(RunError::Unsupported(_))),
            "{id:?} belongs to the separately approved later profile, not M3");
    }
}

#[test]
fn m10_feed_max_hp_is_settled_once_into_the_same_run_player(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:vec![CardInstance{ misc:0,instance:1,id:CardId::Strike,upgrades:0}],relics:vec![],
        potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([7;32],crate::combat::rng::Partition::Test,0)).unwrap();
    run.state.deck=(1..=5).map(|instance|CardInstance{ misc:0,instance,id:CardId::Feed,upgrades:0}).collect();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    for _ in 0..20 {
        if run.state.combat.is_none(){break}
        let decision=run.decision().unwrap();
        if let Some(action)=decision.candidates.iter().find_map(|candidate|match &candidate.action{
            GameAction::Combat(action @ CombatAction::PlayCard{instance,target:Some(TargetId(1))})
                if decision.combat.as_ref().is_some_and(|combat|combat.hand.iter().any(|card|card.instance.0==*instance&&card.id==CardId::Feed))=>Some(action.clone()),
            _=>None,
        }){
            run.choose_combat(action).unwrap();
        }else{
            run.choose_combat(CombatAction::EndTurn).unwrap();
        }
        if run.state.combat.is_some(){
            assert_eq!(run.decision().unwrap().player.max_hp,run.state.max_hp,
                "read-only public projection and persistent player agree after each accepted combat step");
        }
    }
    assert!(run.state.combat.is_none());
    assert_eq!(run.state.max_hp,83,"Feed's max-HP mutation reaches the persistent player exactly once");
    assert_eq!(run.state.combat_progress.settled,1);
}

#[test]
fn m10_feed_gain_survives_accepted_settlement_fault_without_reapplication(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:vec![CardInstance{ misc:0,instance:1,id:CardId::Strike,upgrades:0}],relics:vec![],
        potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([73;32],crate::combat::rng::Partition::Test,0)).unwrap();
    // Privileged Task 9 diagnostic setup after a valid M3 Run constructor.
    run.state.deck=vec![CardInstance{ misc:0,instance:1,id:CardId::Feed,upgrades:0}];
    run.enter_combat(EncounterId::JawWorm).unwrap();
    // Force an existing settlement-census coordinator fault after the accepted
    // Feed action. The extra master-deck row is privileged diagnostic state,
    // not a new injection facility or M3 profile producer.
    run.state.deck.push(CardInstance{ misc:0,instance:99,id:CardId::Strike,upgrades:0});
    let enemy=run.decision().unwrap().combat.unwrap().enemies[0].clone();
    run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![crate::combat::Effect{
        source:crate::combat::Source::Actor(crate::combat::Actor::Player),
        target:Some(crate::combat::Actor::Enemy(enemy.instance_id.clone())),
        kind:crate::combat::EffectKind::Damage(crate::combat::DamagePayload::HpLoss(enemy.hp-1)),
    }],None).unwrap();
    let decision=run.decision().unwrap();
    let feed=decision.candidates.iter().find_map(|candidate|match &candidate.action{
        GameAction::Combat(action @ CombatAction::PlayCard{instance:1,target:Some(TargetId(1))})=>Some(action.clone()),_=>None
    }).expect("actual Feed card and live target were offered");
    let result=run.choose_combat(feed.clone());
    assert!(matches!(result,Err(RunError::Faulted(_))),
        "accepted Feed reaches the existing settlement-census coordinator fault after its native Fatal checkpoint");
    assert_eq!(run.state.max_hp,83,"Run synchronizes committed max HP before the later settlement fault");
    let combat=run.state.combat.as_ref().unwrap().public_full().unwrap();
    assert_eq!((combat.player.hp,combat.player.max_hp),(83,83),"native Feed max HP and ordinary healing remain durably committed before failed Run settlement");
    let exit=run.state.combat.as_ref().unwrap().exit().unwrap();
    assert_eq!((exit.hp,exit.max_hp,exit.outcome),(83,83,crate::combat::Outcome::Victory),"the original native source and combat census remain available behind the coordinator fault");
    assert_eq!(run.state.combat_progress.settled,0);
    assert!(run.state.last_combat.is_none());
    let fault=run.state.fault.clone().expect("first Run fault retained");
    assert_eq!(fault.checkpoint,"run.combat.settlement");
    assert!(matches!(fault.kind,RunFaultKind::Invariant(_)));
    assert!(matches!(run.decision(),Err(RunError::Faulted(_))));
    let frozen=run.clone();
    assert!(run.choose_combat(feed).is_err());
    assert_eq!(run,frozen,"reads/retries cannot replay Feed healing or its max-HP gain");
}

#[test]
fn m10_feed_max_hp_and_current_hp_settle_once_on_real_spiker_retaliation_death(){
    let start=AuthoredRunStart{hp:1,max_hp:80,gold:99,
        deck:vec![CardInstance{ misc:0,instance:1,id:CardId::Strike,upgrades:0}],
        relics:vec![crate::content::RelicId::MarkOfBloom],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([79;32],crate::combat::rng::Partition::Test,0)).unwrap();
    run.state.deck=vec![CardInstance{ misc:0,instance:1,id:CardId::Feed,upgrades:0}];
    // Privileged M10 Run/combat bridge diagnostic: construct the canonical
    // native encounter through the same source entry used by Run ownership,
    // while keeping the M3 authored reset/profile rejection intact.
    let mut setup_rng=RunRng::new([41;32],crate::combat::rng::Partition::Test,0);
    let entry=crate::combat::enemies::setup(EncounterId::ThreeShapes,3,0,&mut setup_rng,crate::combat::enemies::SetupContext{
        player:crate::combat::PlayerState{hp:1,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:Character::Ironclad,
        cards:vec![crate::combat::permanent_live_card(CardId::Feed,crate::combat::CardInstanceId(1),0)],
        opening_draw:1,encounter_visit:0,
    }).unwrap();
    let mut items=Some(crate::items::CombatItemContext::take_from_run(&mut run.state));
    let combat=crate::combat::Combat::enter_with_item_context(entry,&mut items).unwrap();
    assert!(items.is_none());
    run.state.combat=Some(combat);
    run.state.active_combat=Some(ActiveCombat{ applied_mutation_ordinal:0,visit:0,encounter:EncounterId::ThreeShapes,origin:CombatOrigin::Authored,applied_gain_ordinal:0,acquired_instances:vec![]});
    run.state.combat_progress=CombatProgress{entered:1,settled:0};
    let combat_view=run.decision().unwrap().combat.unwrap();
    let spiker=combat_view.enemies.iter().find(|enemy|enemy.enemy_id==crate::content::EnemyId::Spiker).unwrap();
    let target=TargetId(combat_view.enemies.iter().position(|enemy|enemy.instance_id==spiker.instance_id).unwrap() as u64+1);
    run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![crate::combat::Effect{
        source:crate::combat::Source::Actor(crate::combat::Actor::Player),
        target:Some(crate::combat::Actor::Enemy(spiker.instance_id.clone())),
        kind:crate::combat::EffectKind::Damage(crate::combat::DamagePayload::HpLoss(41)),
    }],None).unwrap();
    let decision=run.decision().unwrap();
    let feed=decision.candidates.iter().find_map(|candidate|match &candidate.action{
        GameAction::Combat(action @ CombatAction::PlayCard{instance:1,target:Some(id)}) if *id==target=>Some(action.clone()),_=>None
    }).expect("Feed and actual Spiker target are offered");
    run.choose_combat(feed).unwrap();
    assert_eq!((run.state.hp,run.state.max_hp),(0,83),"Feed commits +3 max HP, Bloom's actual no-heal owner, then queued Spiker THORNS causes Death");
    assert!(run.state.combat.is_none(),"the actual Run settlement consumed the Death combat exactly once");
    assert_eq!((run.state.hp,run.state.max_hp,run.state.terminal),(0,83,Some(RunTerminal::Death)),
        "real Death settlement preserves both committed resources without replay");
}
