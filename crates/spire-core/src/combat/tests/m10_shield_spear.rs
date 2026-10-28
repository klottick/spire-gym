use super::super::*;
use crate::content::{CardId, Character, EncounterId};

fn shield_spear_entry() -> CombatEntry {
    shield_spear_entry_for(Character::Ironclad)
}

fn shield_spear_entry_for(character:Character)->CombatEntry{
    enemies::setup(EncounterId::ShieldAndSpear, 4, 0,
        &mut crate::run::RunRng::new([19;32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![] },
            player_powers:vec![], character,
            cards:(0..5).map(|i|permanent_live_card(CardId::Strike,CardInstanceId(i+1),0)).collect(),
            opening_draw:5, encounter_visit:0,
        }).unwrap()
}

#[test]
fn m10_shield_spear_real_setup_uses_two_canonical_actors_and_initial_intents() {
    let catalog=crate::content::Catalog::standard();
    assert!(catalog.enemy(crate::content::EnemyId::SpireShield).is_some());
    assert!(catalog.enemy(crate::content::EnemyId::SpireSpear).is_some());
    assert!(catalog.encounter(EncounterId::ShieldAndSpear).is_some());
    let entry=shield_spear_entry();
    assert_eq!(entry.setup.enemies.iter().map(|e|(e.enemy_id,e.hp)).collect::<Vec<_>>(),[
        (crate::content::EnemyId::SpireShield,110),
        (crate::content::EnemyId::SpireSpear,160),
    ]);
    assert!(matches!(entry.setup.enemies[0].intent.as_str(),"shield_bash"|"shield_fortify"));
    assert_eq!(entry.setup.enemies[1].intent.as_str(),"spear_burn_strike");
    assert_eq!(entry.setup.enemies[0].powers.iter().find(|p|p.id.0=="artifact").unwrap().state.amount,1);
    assert_eq!(entry.setup.enemies[1].powers.iter().find(|p|p.id.0=="artifact").unwrap().state.amount,1);
    assert!(entry.setup.player_powers.iter().any(|p|p.id.0=="surrounded"&&p.state.amount==1));
    let combat=Combat::enter(entry).unwrap();
    assert_eq!(combat.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),[110,160]);
}

#[test]
fn m10_shield_spear_rejects_malformed_encounter_owned_facing_setup(){
    let mut lone=shield_spear_entry();lone.setup.enemies.pop();
    assert!(Combat::enter(lone).is_err(),"Surrounded/facing requires exactly the canonical Shield-left/Spear-right encounter");
    let mut malformed=shield_spear_entry();
    malformed.setup.player_powers[0].state.amount=2;
    assert!(Combat::enter(malformed).is_err(),"encounter-owned Surrounded marker is the single nonstacking state");
}

#[test]
fn m10_shield_spear_roll_selection_advances_history_without_executing_the_selected_move(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let initial= combat.state.enemies.iter().map(|e|e.intent_id.as_str()).collect::<Vec<_>>();
    assert_eq!(initial,["shield_fortify","spear_burn_strike"]);
    assert_eq!(combat.state.enemies[0].full_memory,Some(EnemyMemory::SpireShield{selections:1}));
    assert_eq!(combat.state.enemies[1].full_memory,Some(EnemyMemory::SpireSpear{selections:1}));
    for slot in 0..2{
        let owner=combat.state.enemies[slot].instance_id.clone();let actor=Actor::Enemy(owner.clone());
        combat.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();
    }
    assert_eq!(combat.state.enemies.iter().map(|e|e.intent_id.as_str()).collect::<Vec<_>>(),["shield_bash","spear_skewer"]);
    assert_eq!(combat.state.player.hp,80);
    assert_eq!(combat.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),[110,160]);
    assert_eq!(super::m5_elites::amount(&combat,0,"strength"),0);
    assert_eq!(super::m5_elites::amount(&combat,1,"strength"),0);
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let shield=combat.state.enemies[0].instance_id.clone();let actor=Actor::Enemy(shield.clone());
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();
    assert_eq!(combat.state.enemies[0].intent_id.as_str(),"shield_smash");
    assert_eq!(combat.state.enemies[0].intent_damage,Some(34));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_shield_spear_rejects_selection_counter_not_matching_retained_history(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    combat.state.enemies[0].full_memory=Some(EnemyMemory::SpireShield{selections:0});
    assert!(engine::validate_runtime(&combat.state).is_err(),"selection counter must match the actual retained selection history");
}

#[test]
fn m10_shield_spear_selector_covers_both_opening_orders_and_spear_repeat_guard(){
    let template=Combat::enter(shield_spear_entry()).unwrap();
    let stream=template.state.enemies[0].full_ai_rng.as_ref().unwrap();
    let choices=(0..2).map(|cursor|{
        let stream=rng::StreamState::new(stream.id().clone(),[0;32],cursor).unwrap();
        enemies::select_move(crate::content::EnemyId::SpireShield,&[],&EnemyMemory::SpireShield{selections:0},&stream).unwrap().intent.as_str()
    }).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(choices,["shield_bash","shield_fortify"].into_iter().collect());

    for (previous,expected) in [("spear_burn_strike","spear_strength"),("spear_strength","spear_burn_strike")] {
        let history=["spear_burn_strike","spear_skewer",previous].into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();
        let next=enemies::select_move(crate::content::EnemyId::SpireSpear,&history,
            &EnemyMemory::SpireSpear{selections:3},template.state.enemies[1].full_ai_rng.as_ref().unwrap()).unwrap();
        assert_eq!(next.intent.as_str(),expected,"non-Skewer alternation is selected from prior history before execution");
        assert_eq!(next.stream,template.state.enemies[1].full_ai_rng.clone().unwrap(),"deterministic non-Skewer guard consumes no RNG");
    }
}

#[test]
fn m10_shield_spear_random_selection_fault_retains_counter_history_and_stream(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let owner=combat.state.enemies[0].instance_id.clone();
    let history=["shield_bash","shield_fortify","shield_smash"].into_iter()
        .map(|name|ActorIntentId::Full(FullIntentId::from_name(name))).collect::<Vec<_>>();
    let selected=history.last().unwrap().clone();
    combat.state.enemies[0].move_history=history;
    combat.state.enemies[0].intent_id=selected.clone();combat.state.enemies[0].selected_move=selected;
    combat.state.enemies[0].full_memory=Some(EnemyMemory::SpireShield{selections:3});
    let stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    combat.state.enemies[0].full_ai_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap());
    effects::refresh_enemy_values(&mut combat.state).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok(),"exhausted but identity-valid stream is a valid producer frontier");
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let before=(combat.state.enemies[0].intent_id.clone(),combat.state.enemies[0].move_history.clone(),combat.state.enemies[0].full_memory.clone(),combat.state.enemies[0].full_ai_rng.clone());
    combat.state.pending.push_front(Effect{source:Source::Actor(Actor::Enemy(owner.clone())),target:Some(Actor::Enemy(owner)),kind:EffectKind::RollMove});
    engine::resolve(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    assert_eq!((combat.state.enemies[0].intent_id.clone(),combat.state.enemies[0].move_history.clone(),combat.state.enemies[0].full_memory.clone(),combat.state.enemies[0].full_ai_rng.clone()),before);
    assert_eq!(combat.state.current.as_ref().unwrap().kind,EffectKind::RollMove);
}

#[test]
fn m10_shield_bash_uses_focus_branch_for_real_defect_orb_slots(){
    let mut empty=Combat::enter(shield_spear_entry_for(Character::Defect)).unwrap();
    select_intent(&mut empty,0,"shield_bash");
    let no_orb_rng=empty.state.enemies[0].full_ai_rng.clone();
    let shield=empty.state.enemies[0].instance_id.clone();
    let work=enemies::act4::move_work(&mut empty.state,&shield).unwrap();
    empty.enqueue_native_effects(work,None).unwrap();
    assert_eq!(empty.state.character.focus,0,"empty Orb Slots take the Strength branch even for Defect");
    assert_eq!(empty.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,-1);
    assert_eq!(empty.state.enemies[0].full_ai_rng,no_orb_rng,"no active Orb consumes no AI coin");

    let mut combat=Combat::enter(shield_spear_entry_for(Character::Defect)).unwrap();
    combat.state.character.orb_slots[0]=Some(OrbKind::Frost.into());
    select_intent(&mut combat,0,"shield_bash");
    let shield=combat.state.enemies[0].instance_id.clone();
    let focus_stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    let (focus_coin,_)=rng::bernoulli(combat.state.enemies[0].full_ai_rng.as_ref().unwrap(),1,2).unwrap();
    assert!(focus_coin,"fixed active-Orb fixture must exercise the qualified Focus outcome");
    let work=enemies::act4::move_work(&mut combat.state,&shield).unwrap();
    combat.enqueue_native_effects(work,None).unwrap();
    assert_eq!(combat.state.character.focus,-1,"an active Orb plus the deterministic coin selects -1 Focus");
    assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref().unwrap().word_index(),focus_stream.word_index()+1,"active-Orb Bash consumes exactly one enemy-AI coin");

    let mut combat=Combat::enter(shield_spear_entry_for(Character::Defect)).unwrap();
    combat.state.character.orb_slots[0]=Some(OrbKind::Frost.into());
    select_intent(&mut combat,0,"shield_bash");
    let stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    combat.state.enemies[0].full_ai_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),stream.word_index()+1).unwrap());
    let coin_stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    let (focus_coin,_)=rng::bernoulli(combat.state.enemies[0].full_ai_rng.as_ref().unwrap(),1,2).unwrap();
    assert!(!focus_coin,"adjacent fixed AI frontier exercises the alternate Strength outcome");
    let shield=combat.state.enemies[0].instance_id.clone();
    let work=enemies::act4::move_work(&mut combat.state,&shield).unwrap();
    combat.enqueue_native_effects(work,None).unwrap();
    assert_eq!(combat.state.character.focus,0);
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,-1);
    assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref().unwrap().word_index(),coin_stream.word_index()+1,"the alternate orb-slot outcome consumes the same single coin");
}

#[test]
fn m10_shield_bash_focus_uses_player_artifact_then_applies_after_it_is_spent(){
    let mut combat=Combat::enter(shield_spear_entry_for(Character::Defect)).unwrap();
    combat.state.character.orb_slots[0]=Some(OrbKind::Frost.into());
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let focus_coin_frontier=combat.state.enemies[0].full_ai_rng.clone();

    for expected_focus in [0,-1]{
        select_intent(&mut combat,0,"shield_bash");
        let shield=combat.state.enemies[0].instance_id.clone();
        // Replay the same deterministic Focus coin fixture to isolate the
        // Artifact-blocked and post-Artifact application outcomes.
        combat.state.enemies[0].full_ai_rng=focus_coin_frontier.clone();
        let stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
        let (focus_coin,_)=rng::bernoulli(&stream,1,2).unwrap();
        assert!(focus_coin,"fixed active-Orb test frontier must choose the Focus branch");
        let work=enemies::act4::move_work(&mut combat.state,&shield).unwrap();
        combat.enqueue_native_effects(work,None).unwrap();
        assert_eq!(combat.state.character.focus,expected_focus);
        assert!(!combat.state.player_full_powers.iter().any(|power|power.id.0=="artifact")
            &&!combat.state.player.powers.iter().any(|power|power.id.as_str()=="artifact"),
            "Artifact is consumed by the first Focus debuff and absent thereafter");
    }
}

#[test]
fn m10_shield_bash_coin_fault_retains_previously_published_hit_without_executing_or_retrying_it(){
    let mut combat=Combat::enter(shield_spear_entry_for(Character::Defect)).unwrap();
    combat.state.character.orb_slots[0]=Some(OrbKind::Frost.into());
    select_intent(&mut combat,0,"shield_bash");
    let owner=combat.state.enemies[0].instance_id.clone();
    let stream=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    combat.state.enemies[0].full_ai_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap());
    effects::refresh_enemy_values(&mut combat.state).unwrap();
    let hp_before=combat.state.player.hp;
    let actor=Actor::Enemy(owner.clone());
    combat.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});
    engine::resolve(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    assert_eq!(combat.state.player.hp,hp_before,"the producer faults before the published attack dispatches");
    assert!(matches!(combat.state.current.as_ref().map(|effect|&effect.kind),Some(EffectKind::ShieldBashFocus)));
    assert!(combat.state.pending.iter().any(|effect|matches!(effect.kind,EffectKind::Damage(DamagePayload::FullEnemyMove(intent)) if intent.as_str()=="shield_bash")),
        "the already-published attack remains owned behind the producer fault");
    assert!(combat.state.pending.iter().any(|effect|matches!(effect.kind,EffectKind::RollMove)),
        "the next-move roll remains behind the attack and retained producer");
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the retained producer, queued hit, and next roll remain a valid fault checkpoint");
    let faulted=combat.state.clone();
    engine::resolve(&mut combat.state);
    assert_eq!(combat.state,faulted,"a retained RNG fault cannot retry the coin or execute the queued hit");
}

#[test]
fn m10_shield_bash_samples_coin_at_producer_before_hit_then_resolves_hit_before_debuff(){
    let mut combat=Combat::enter(shield_spear_entry_for(Character::Defect)).unwrap();
    combat.state.character.orb_slots[0]=Some(OrbKind::Frost.into());
    select_intent(&mut combat,0,"shield_bash");
    let owner=combat.state.enemies[0].instance_id.clone();
    let ai_before=combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    let(focus_coin,_)=rng::bernoulli(&ai_before,1,2).unwrap();
    assert!(focus_coin,"the fixed producer fixture selects the Focus debuff");
    let hp_before=combat.state.player.hp;
    combat.state.phase=Phase::EnemyTurn;
    let actor=Actor::Enemy(owner.clone());
    combat.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});
    engine::validate_runtime(&combat.state).unwrap_or_else(|error|panic!("queued enemy-turn runtime: {error:?}"));

    engine::dispatch_one(&mut combat.state);
    assert!(matches!(combat.state.pending.front().map(|effect|&effect.kind),Some(EffectKind::ShieldBashFocus)));
    assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref().unwrap(),&ai_before,"EnemyMove queues the producer without sampling its coin");
    assert_eq!(combat.state.player.hp,hp_before);
    engine::validate_runtime(&combat.state).unwrap_or_else(|error|panic!("pre-producer runtime: {error:?}"));

    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref().unwrap().word_index(),ai_before.word_index()+1,
        "the producer samples at its qualified checkpoint before any hit dispatch");
    assert_eq!(combat.state.player.hp,hp_before,"sampling alone does not execute the already-published hit");
    assert!(matches!(combat.state.pending.get(0).map(|effect|&effect.kind),Some(EffectKind::Damage(DamagePayload::FullEnemyMove(intent))) if intent.as_str()=="shield_bash"));
    assert!(matches!(combat.state.pending.get(1).map(|effect|&effect.kind),Some(EffectKind::Character(CharacterEffect::Focus(-1)))));
    assert!(matches!(combat.state.pending.get(2).map(|effect|&effect.kind),Some(EffectKind::RollMove)));
    engine::validate_runtime(&combat.state).unwrap_or_else(|error|panic!("producer checkpoint runtime: {error:?}"));

    engine::dispatch_one(&mut combat.state);
    assert!(combat.state.player.hp<hp_before,"the hit dispatches after the producer and before the selected debuff");
    assert!(matches!(combat.state.pending.get(0).map(|effect|&effect.kind),Some(EffectKind::Hook(Hook::Damage))));
    assert!(matches!(combat.state.pending.get(1).map(|effect|&effect.kind),Some(EffectKind::Character(CharacterEffect::Focus(-1)))));
    engine::validate_runtime(&combat.state).unwrap_or_else(|error|panic!("post-hit runtime: {error:?}"));
    engine::dispatch_one(&mut combat.state); // Damage callbacks
    engine::dispatch_one(&mut combat.state); // Focus request enters shared power/Artifact handling
    engine::dispatch_one(&mut combat.state); // The unblocked Focus delta commits
    assert_eq!(combat.state.character.focus,-1);
    assert!(combat.state.fault.is_none());
    assert!(matches!(combat.state.pending.front().map(|effect|&effect.kind),Some(EffectKind::RollMove)));
    engine::validate_runtime(&combat.state).unwrap_or_else(|error|panic!("post-debuff runtime: {error:?}"));
}

#[test]
fn m10_focus_debuff_rejects_outside_active_orb_shield_bash_branch(){
    let combat=Combat::enter(shield_spear_entry()).unwrap();
    let effect=Effect{source:Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),target:Some(Actor::Player),
        kind:EffectKind::Character(CharacterEffect::Focus(-1))};
    assert!(engine::validate_effect(&effect,&combat.state).is_err(),"Focus movement is supported only as Shield Bash's orb-slot branch");
}

#[test]
fn m10_shield_spear_artifact_blocks_one_debuff_then_allows_the_next(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let shield=combat.state.enemies[0].instance_id.clone();
    let debuff=||Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(shield.clone())),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}};
    combat.enqueue_native_effects(vec![debuff()],None).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|power.id.0=="artifact").map(|power|power.state.amount),None);
    assert!(!combat.state.enemies[0].full_powers.iter().any(|power|power.id.0=="vulnerable"));
    combat.enqueue_native_effects(vec![debuff()],None).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|power.id.0=="vulnerable").unwrap().state.amount,1);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_shield_spear_first_pair_turn_executes_burn_strike_and_discards_burns() {
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player.hp,66);
    assert_eq!(combat.state.piles.all(combat.state.in_play.as_ref()).filter(|card|card.full().unwrap().id==CardId::Burn).count(),2);
    assert_eq!(combat.state.enemies[1].intent_id.as_str(),"spear_skewer");
    assert_eq!(combat.state.enemies[0].intent_id.as_str(),"shield_bash");
    assert_eq!(combat.state.enemies[1].intent_damage,Some(15));
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player.hp,5,"second selected pair executes Shield Bash12, rear Spear Skewer15x3 and two drawn Burn HP-losses");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_spear_burn_strike_publishes_both_burns_to_discard_before_turn_cleanup(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let spear=combat.state.enemies[1].instance_id.clone();
    let work=enemies::act4::move_work(&mut combat.state,&spear).unwrap();
    let burns=work.iter().filter_map(|effect|match effect.kind{EffectKind::CreateCard{id,destination,..} if id==CardId::Burn=>Some(destination),_=>None}).collect::<Vec<_>>();
    assert_eq!(burns,[CardDestination::Discard,CardDestination::Discard]);
    combat.enqueue_native_effects(work,None).unwrap();
    assert_eq!(combat.state.piles.discard.iter().filter(|card|card.full().unwrap().id==CardId::Burn).count(),2);
    assert_eq!(combat.state.player.hp,66,"right-side Spear's 5x2 receives rear damage at the current left-facing checkpoint");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

fn select_intent(combat:&mut Combat,slot:usize,name:&'static str){
    let selected=ActorIntentId::Full(FullIntentId::from_name(name));
    let history=match (slot,name){
        (0,"shield_smash")=>vec!["shield_bash","shield_fortify","shield_smash"],
        (0,_)=>vec![name],
        (1,"spear_strength")=>vec!["spear_burn_strike","spear_skewer","spear_strength"],
        (1,_)=>vec![name],
        _=>unreachable!(),
    }.into_iter().map(|name|ActorIntentId::Full(FullIntentId::from_name(name))).collect::<Vec<_>>();
    let selections=history.len() as u32;
    combat.state.enemies[slot].intent_id=selected.clone();combat.state.enemies[slot].selected_move=selected;combat.state.enemies[slot].move_history=history;
    combat.state.enemies[slot].full_memory=Some(if slot==0{EnemyMemory::SpireShield{selections}}else{EnemyMemory::SpireSpear{selections}});
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

#[test]
fn m10_shield_bash_fortify_and_smash_apply_actual_power_damage_and_block_effects(){
    for (shield,spear,hp,shield_strength,shield_block,spear_strength) in [
        ("shield_bash","spear_strength",68,2,0,2),
        ("shield_fortify","spear_strength",80,2,30,2),
        ("shield_smash","spear_strength",46,2,34,2),
    ]{
        let mut combat=Combat::enter(shield_spear_entry()).unwrap();
        select_intent(&mut combat,0,shield);select_intent(&mut combat,1,spear);
        if shield=="shield_smash"{combat.state.player.block=999;}
        let shield_owner=combat.state.enemies[0].instance_id.clone();
        let shield_rng_before=combat.state.enemies[0].full_ai_rng.clone();
        let shield_work=enemies::act4::move_work(&mut combat.state,&shield_owner).unwrap();
        combat.enqueue_native_effects(shield_work,None).unwrap();
        if shield=="shield_bash"{assert_eq!(combat.state.enemies[0].full_ai_rng,shield_rng_before,"no-Orb-Slot Bash applies Strength directly without consuming a coin draw");}
        let spear_owner=combat.state.enemies[1].instance_id.clone();
        let spear_work=enemies::act4::move_work(&mut combat.state,&spear_owner).unwrap();
        combat.enqueue_native_effects(spear_work,None).unwrap();
        assert_eq!(combat.state.player.hp,if shield=="shield_smash"{80}else{hp}, "{shield}");
        assert_eq!(super::m5_elites::amount(&combat,0,"strength"),shield_strength,"{shield}");
        assert_eq!(super::m5_elites::amount(&combat,1,"strength"),spear_strength,"{shield}");
        assert_eq!(combat.state.enemies[0].block,shield_block,"{shield}");
        if shield=="shield_bash"{assert_eq!(combat.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,-1);}
        if shield=="shield_fortify"{assert_eq!(combat.state.enemies[1].block,30);}
        assert!(engine::validate_runtime(&combat.state).is_ok());
    }
}

#[test]
fn m10_shield_fortify_published_group_targets_survive_source_death_with_live_target_guard(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();select_intent(&mut combat,0,"shield_fortify");
    let source=combat.state.enemies[0].instance_id.clone();
    let work=enemies::act4::move_work(&mut combat.state,&source).unwrap();
    assert_eq!(work.iter().map(|e|e.target.clone()).collect::<Vec<_>>(),[
        Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),
        Some(Actor::Enemy(combat.state.enemies[1].instance_id.clone())),
    ]);
    combat.state.pending.extend(work);
    combat.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(source.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(500))});
    engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none());
    assert_eq!(combat.state.enemies[0].hp,0);
    assert_eq!(combat.state.enemies[0].block,0);
    assert_eq!(combat.state.enemies[1].block,30);
    assert_eq!(combat.state.outcome,None);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_surrounded_targeted_play_turns_toward_enemy_and_changes_rear_damage_once() {
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let selected=ActorIntentId::Full(FullIntentId::from_name("shield_bash"));
    combat.state.enemies[0].intent_id=selected.clone();combat.state.enemies[0].selected_move=selected.clone();combat.state.enemies[0].move_history=vec![selected];
    effects::refresh_enemy_values(&mut combat.state).unwrap();
    let initial=combat.encounter_facing_view().unwrap().unwrap();
    assert_eq!(initial.player_facing,PlayerFacing::Left);
    assert_eq!(initial.enemies.iter().map(|e|(e.side,e.back_attack)).collect::<Vec<_>>(),[(EncounterSide::Left,false),(EncounterSide::Right,true)]);
    assert_eq!(combat.state.enemies[0].intent_damage,Some(12));
    assert_eq!(combat.state.enemies[1].intent_damage,Some(7));
    let before=combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(999))}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(snapshot::compare(&before,&combat.snapshot()),None);
    combat.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(2))}).unwrap();
    let faced=combat.encounter_facing_view().unwrap().unwrap();
    assert_eq!(faced.player_facing,PlayerFacing::Right);
    assert_eq!(combat.state.enemies[0].intent_damage,Some(18));
    assert_eq!(combat.state.enemies[1].intent_damage,Some(5));
    combat.step_full(&CombatAction::PlayCard{instance:2,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.encounter_facing_view().unwrap().unwrap().player_facing,PlayerFacing::Left);
    assert_eq!(combat.state.enemies[0].intent_damage,Some(12));
    assert_eq!(combat.state.enemies[1].intent_damage,Some(7));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_surrounded_targeted_skill_and_targeted_potion_change_facing_only_after_preflight(){
    let mut entry=shield_spear_entry();entry.cards[0]=permanent_live_card(CardId::Disarm,CardInstanceId(1),0);
    let mut combat=Combat::enter(entry).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(2))}).unwrap();
    assert_eq!(combat.encounter_facing_view().unwrap().unwrap().player_facing,PlayerFacing::Right);
    let inventory=crate::items::CombatItemContext::new(vec![],vec![Some(crate::content::PotionId::FirePotion),None,None]);
    let mut items=Some(inventory);
    let mut potion=Combat::enter_with_item_context(shield_spear_entry(),&mut items).unwrap();
    let before=potion.snapshot();
    assert_eq!(potion.step_full(&CombatAction::UsePotion{slot:0,target:Some(TargetId(999))}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(snapshot::compare(&before,&potion.snapshot()),None);
    potion.step_full(&CombatAction::UsePotion{slot:0,target:Some(TargetId(2))}).unwrap();
    assert_eq!(potion.encounter_facing_view().unwrap().unwrap().player_facing,PlayerFacing::Right);
    assert_eq!(potion.state.item_context.as_ref().unwrap().potions()[0],None,"successful targeted potion custody remains committed");
    let accepted_facing=potion.state.facing;
    let accepted_cursor=potion.state.card_rng.clone();
    potion.state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(1)});
    potion.state.resolved_effects=99_999;
    engine::dispatch_one(&mut potion.state);
    assert_eq!(potion.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(potion.state.facing,accepted_facing,"a later retained resolver fault does not roll back the accepted target checkpoint");
    assert_eq!(potion.state.card_rng,accepted_cursor);
    assert_eq!(potion.state.item_context.as_ref().unwrap().potions()[0],None);
    let frozen=potion.snapshot();let first=potion.state.fault.clone();let current=potion.state.current.clone();
    assert!(potion.step_full(&CombatAction::EndTurn).is_err());
    assert_eq!(potion.snapshot(),frozen);assert_eq!(potion.state.fault,first);assert_eq!(potion.state.current,current);
}

#[test]
fn m10_surrounded_aoe_random_and_nontargeted_plays_do_not_turn_player(){
    for (id,instance) in [(CardId::Thunderclap,1),(CardId::SwordBoomerang,1),(CardId::Inflame,1)]{
        let mut entry=shield_spear_entry();entry.cards[0]=permanent_live_card(id,CardInstanceId(instance),0);
        let mut combat=Combat::enter(entry).unwrap();
        combat.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
        assert_eq!(combat.encounter_facing_view().unwrap().unwrap().player_facing,PlayerFacing::Left,"{id:?}");
    }
}

#[test]
fn m10_surrounded_nested_hand_selection_and_havoc_autoplay_do_not_turn_player(){
    let mut entry=shield_spear_entry();entry.cards[0]=permanent_live_card(CardId::Armaments,CardInstanceId(1),0);
    let mut selection=Combat::enter(entry).unwrap();
    selection.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(selection.state.phase,Phase::Selection);
    selection.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    if selection.state.phase==Phase::Selection{selection.step_full(&CombatAction::ConfirmSelection).unwrap();}
    assert_eq!(selection.state.facing,Some(PlayerFacing::Left));
    assert_eq!(selection.encounter_facing_view().unwrap().unwrap().player_facing,PlayerFacing::Left);

    let mut entry=shield_spear_entry();
    entry.cards[0]=permanent_live_card(CardId::Havoc,CardInstanceId(1),0);
    let mut combat=Combat::enter(entry).unwrap();
    let index=combat.state.piles.hand.iter().position(|card|card.full().is_some_and(|card|card.instance.0==2)).unwrap();
    let mut child=combat.state.piles.hand.remove(index);child.full_mut().unwrap().id=CardId::Strike;
    combat.state.piles.draw.clear();combat.state.piles.draw.push(child);
    combat.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(combat.state.facing,Some(PlayerFacing::Left));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_manual_facing_persists_through_opposite_side_havoc_autoplay_target(){
    let mut entry=shield_spear_entry();
    entry.cards[0]=permanent_live_card(CardId::Havoc,CardInstanceId(1),0);
    entry.cards[1]=permanent_live_card(CardId::Disarm,CardInstanceId(2),0);
    let mut combat=Combat::enter(entry).unwrap();
    let index=combat.state.piles.hand.iter().position(|card|card.full().is_some_and(|card|card.instance.0==3)).unwrap();
    let child=combat.state.piles.hand.remove(index);combat.state.piles.draw.push(child);
    let old_stream=combat.state.card_rng.as_ref().unwrap().clone();
    combat.state.card_rng=Some(rng::StreamState::new(old_stream.id().clone(),[0;32],0).unwrap());
    let (target_index,_)=rng::uniform_below(combat.state.card_rng.as_ref().unwrap(),2).unwrap();
    assert_eq!(target_index,0,"fixed card-random frontier captures left Shield as Havoc's child target");
    combat.step_full(&CombatAction::PlayCard{instance:2,target:Some(TargetId(2))}).unwrap();
    assert_eq!(combat.state.facing,Some(PlayerFacing::Right));
    let left_hp=combat.state.enemies[0].hp;let right_hp=combat.state.enemies[1].hp;
    combat.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert!(combat.state.enemies[0].hp<left_hp);
    assert_eq!(combat.state.enemies[1].hp,right_hp);
    assert_eq!(combat.state.facing,Some(PlayerFacing::Right),"autoplay does not refresh the manual facing checkpoint");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_surrounded_partner_death_faces_survivor_and_removes_pair_markers() {
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    super::m5_elites::hit(&mut combat,0,500);
    assert_eq!(combat.state.enemies[0].hp,0);
    assert_eq!(combat.state.enemies[1].hp,160);
    assert_eq!(combat.state.outcome,None);
    assert_eq!(combat.state.facing,Some(PlayerFacing::Right));
    assert!(!combat.state.player_full_powers.iter().any(|p|p.id.0=="surrounded"));
    assert!(!combat.state.enemies[1].full_powers.iter().any(|p|p.id.0=="back_attack"));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_surrounded_spear_death_keeps_left_survivor_as_stable_facing_owner(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let shield=combat.state.enemies[0].instance_id.clone();
    super::m5_elites::hit(&mut combat,1,500);
    assert_eq!(combat.state.enemies[1].hp,0);
    assert_eq!(combat.state.enemies[0].instance_id,shield);
    assert_eq!(combat.state.facing,Some(PlayerFacing::Left));
    assert!(!combat.state.player_full_powers.iter().any(|p|p.id.0=="surrounded"));
    assert!(!combat.state.enemies[0].full_powers.iter().any(|p|p.id.0=="back_attack"));
    assert_eq!(combat.state.outcome,None);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_partner_death_faces_before_queued_marker_removal_and_retains_fault_order(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    let shield=combat.state.enemies[0].instance_id.clone();
    combat.state.resolved_effects=99_998;
    combat.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(shield)),kind:EffectKind::Damage(DamagePayload::Fixed(500))});
    engine::resolve(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(combat.state.facing,Some(PlayerFacing::Right),"the encounter flips to the live Spear immediately on Shield death");
    assert_eq!(combat.state.current.as_ref().unwrap().kind,EffectKind::Hook(Hook::Damage));
    let cleanup=combat.state.pending.iter().filter_map(|effect|match &effect.kind{
        EffectKind::RemoveFullPower(id)=>Some(id.0),_=>None,
    }).collect::<Vec<_>>();
    assert_eq!(cleanup,["surrounded","back_attack"],"facing changes immediately; queued cleanup removes player marker before survivor marker");
    assert_eq!(combat.state.enemies[1].hp,160,"retained fault occurs after parent death but before queued survivor marker removal");
}

#[test]
fn m10_back_attack_uses_post_calculated_damage_binary32_checkpoint(){
    let mut combat=Combat::enter(shield_spear_entry()).unwrap();
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    effects::refresh_enemy_values(&mut combat.state).unwrap();
    assert_eq!(combat.state.enemies[1].intent_damage,Some(10),"calculateDamage truncates 5 × Vulnerable 1.5 to 7; applyPowers then truncates 7 × Back Attack 1.5 to 10");
    combat.state.player_full_powers.retain(|power|power.id.0!="vulnerable");
    combat.state.enemies[1].full_powers.push(FullPowerEntry{id:FullPowerId("weak"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    effects::refresh_enemy_values(&mut combat.state).unwrap();
    assert_eq!(combat.state.enemies[1].intent_damage,Some(4),"Weak truncates 5 × 0.75 to 3 before Back Attack truncates 3 × 1.5");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_back_attack_does_not_retroactively_scale_direct_payload_categories(){
    for payload in [DamagePayload::Thorns(5),DamagePayload::HpLoss(5),DamagePayload::Fixed(5)]{
        let mut combat=Combat::enter(shield_spear_entry()).unwrap();
        let spear=combat.state.enemies[1].instance_id.clone();
        let before=combat.state.enemies[1].hp;
        let label=format!("{payload:?}");
        combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(spear.clone())),kind:EffectKind::Damage(payload)}],None).unwrap_or_else(|error|panic!("{label}: {error:?}"));
        assert_eq!(before-combat.state.enemies[1].hp,5,"Back Attack is not a global scalar on producer-supplied direct {label}");
        assert!(engine::validate_runtime(&combat.state).is_ok());
    }
}
