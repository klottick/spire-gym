//! Native passive/reactive Ironclad relic behavior through acquired inventory.
use super::super::*;
use crate::combat::model::{ExhaustBatchFilter,ExhaustBatchFinish,FinishDisposition,PlayContext};
use crate::{content::{CardId,Character,EncounterId,RelicId},items::CombatItemContext,run::*};

fn entry(encounter:EncounterId)->CombatEntry {
    let mut e=super::m8_shapes::entry(encounter);
    e.cards=(1..=12).map(|id|permanent_live_card(CardId::Defend,CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;e
}
fn acquired(ids:&[RelicId])->CombatItemContext {
    acquired_for(Character::Ironclad,ids)
}
fn acquired_for(character:Character,ids:&[RelicId])->CombatItemContext {
    let mut run=Run::new(RunConfig{character,ascension:0},CreationProfile::experienced(),RunRng::new([41;32],rng::Partition::Test,0)).unwrap();
    for id in ids {crate::items::acquire_relic(run.state_mut_for_test(),*id).unwrap();}
    CombatItemContext::take_from_run(run.state_mut_for_test())
}
fn enter(encounter:EncounterId,ids:&[RelicId])->Combat {
    let c=Combat::enter_with_item_context(entry(encounter),&mut Some(acquired(ids))).unwrap();
    assert!(engine::validate_runtime(&c.state).is_ok());
    c
}
fn enter_for_character(encounter:EncounterId,character:Character,ids:&[RelicId])->Combat {
    let mut e=entry(encounter);e.setup.character=character;
    let mut items=Some(acquired_for(character,ids));
    let c=Combat::enter_with_item_context(e,&mut items).unwrap();
    assert!(engine::validate_runtime(&c.state).is_ok());c
}
fn enemy_power(c:&Combat,slot:usize,id:&str)->i32 {c.state.enemies[slot].full_powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.state.amount)}

#[test]
fn m10_orichalcum_triggers_six_block_only_when_player_has_none_at_end_turn(){
    for character in [Character::Ironclad,Character::Silent,Character::Defect,Character::Watcher]{
        let mut c=enter_for_character(EncounterId::OrbWalker,character,&[RelicId::Orichalcum]);
        c.state.player_full_powers.extend([
            FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}},
            FullPowerEntry{id:FullPowerId("frail"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}},
        ]);
        engine::validate_runtime(&c.state).expect("actual common relic ownership before command");
        let ended=c.step_full_observed(&CombatAction::EndTurn);ended.result.unwrap();
        let grant=ended.effects.iter().find(|event|event.kind==ResolvedEffectKind::Block{before:0,after:6}).unwrap();
        let enemy_work=ended.effects.iter().find(|event|matches!(event.kind,ResolvedEffectKind::EnemyMove{..}|ResolvedEffectKind::Damage{category:DamageCategory::Normal,..}));
        assert!(enemy_work.is_none_or(|event|grant.sequence<event.sequence),"Orichalcum resolves at PlayerEndTurn before actual enemy actions for {character:?}");
        engine::validate_runtime(&c.state).expect("valid state after zero-Block actual EndTurn");
        c.state.player.block=4;
        engine::validate_runtime(&c.state).expect("valid positive-Block fixture before next EndTurn");
        let ended=c.step_full_observed(&CombatAction::EndTurn);ended.result.unwrap();
        assert!(!ended.effects.iter().any(|event|event.kind==ResolvedEffectKind::Block{before:4,after:10}),"positive existing Block suppresses Orichalcum for {character:?}");
        engine::validate_runtime(&c.state).expect("valid state after positive-Block actual EndTurn");
    }
}

#[test]
fn m10_orichalcum_explicit_trigger_queues_block_even_when_player_already_has_block(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::Orichalcum]);
    c.state.player.block=4;
    c.state.item_context.as_mut().unwrap().orichalcum_trigger=true;
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::TurnStarted)}],None).unwrap();
    assert!(!c.state.item_context.as_ref().unwrap().orichalcum_trigger,"actual TurnStarted callback clears any prior flag");
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::PlayerEndTurn)}],None).unwrap();
    assert_eq!(c.state.player.block,4,"a cleared flag cannot independently retrigger at positive Block");
    c.state.item_context.as_mut().unwrap().orichalcum_trigger=true;
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::PlayerEndTurn)}],None).unwrap();
    assert_eq!(c.state.player.block,10,"source-qualified trigger flag independently requests the top-queued six Block");
    assert!(!c.state.item_context.as_ref().unwrap().orichalcum_trigger,"accepted callback consumes the trigger before its Block child");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_orichalcum_checks_block_after_metallicize_and_ethereal_feel_no_pain(){
    for character in [Character::Ironclad,Character::Silent,Character::Defect,Character::Watcher]{
        let mut e=entry(EncounterId::OrbWalker);
        e.setup.character=character;
        e.cards=vec![permanent_live_card(CardId::Dazed,CardInstanceId(1),0)];
        e.opening_draw=1;
        e.setup.player_powers.extend([
            FullPowerEntry{id:FullPowerId("metallicize"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}},
            FullPowerEntry{id:FullPowerId("feel_no_pain"),state:PowerState{amount:4,skip_first:false,monster_source_grace:false}},
        ]);
        let mut owner=Some(CombatItemContext::new(vec![RelicId::Orichalcum],vec![]));
        let mut c=Combat::enter_with_item_context(e,&mut owner).unwrap();
        assert!(engine::validate_runtime(&c.state).is_ok(),"valid {character:?} setup before EndTurn");
        let observed=c.step_full_observed(&CombatAction::EndTurn);observed.result.unwrap();
        let blocks=observed.effects.iter().filter_map(|event|match event.kind{
            ResolvedEffectKind::Block{before,after}=>Some((before,after)),_=>None,
        }).collect::<Vec<_>>();
        assert_eq!(&blocks[..2],&[(0,2),(2,6)],"Metallicize is an earlier player pre-end-turn callback, then actual Dazed exhaust invokes Feel No Pain before the later Orichalcum decision ({character:?}); effects={:?}; hooks={:?}; exhaust={:?}",observed.effects,c.state.hook_events,c.state.piles.exhaust);
        assert_eq!(blocks.len(),3,"the final Block change is the actual enemy attack consuming the six existing Block, not an Orichalcum grant");
        let hooks=&c.state.hook_events;
        let pre=hooks.iter().position(|event|event.hook==Hook::PlayerPreEndTurnCards).unwrap();
        let exhausted=hooks.iter().position(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(CardInstanceId(1)))).unwrap();
        let end=hooks.iter().position(|event|event.hook==Hook::PlayerEndTurn).unwrap();
        assert!(pre<exhausted&&exhausted<end,"actual callback order remains pre-end-turn, Ethereal/FNP, then PlayerEndTurn ({character:?})");
        assert!(engine::validate_runtime(&c.state).is_ok(),"valid {character:?} state after EndTurn");
    }
}

#[test]
fn m10_orichalcum_callback_budget_fault_retains_consumed_trigger_and_current_block_child(){
    let mut e=entry(EncounterId::OrbWalker);e.opening_draw=0;
    let mut owner=Some(acquired(&[RelicId::Orichalcum]));
    let mut c=Combat::enter_with_item_context(e,&mut owner).unwrap();
    c.state.player.block=4;
    c.state.item_context.as_mut().unwrap().orichalcum_trigger=true;
    c.state.resolved_effects=99_994;
    engine::validate_runtime(&c.state).expect("canonical live owner before existing effect budget boundary");
    let end_turn=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::EndPlayerTurn};
    engine::validate_effect(&end_turn,&c.state).expect("the real accepted EndTurn continuation is valid before queueing");
    c.state.pending.push_front(end_turn);
    let transition=engine::resolve(&mut c.state);
    assert_eq!(transition.status,TransitionStatus::Fault);
    assert_eq!(c.state.player.block,4,"the following GainBlock child itself is the 100,000th rejected dispatch");
    assert!(c.state.item_context.as_ref().unwrap().owns_relic(RelicId::Orichalcum));
    assert!(!c.state.item_context.as_ref().unwrap().orichalcum_trigger,"the accepted callback consumed its flag before the child budget fault");
    assert_eq!(c.state.current.as_ref().map(|effect|&effect.kind),Some(&EffectKind::GainBlock(6)),"the already queued grant remains the retained current owner");
    assert!(c.state.fault.is_some());
    assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::PlayerEndTurn).count(),1,"actual end-turn callback committed once before the child fault");
    assert!(c.state.pending.iter().any(|effect|effect.kind==EffectKind::MonsterStart),"the remaining accepted turn continuation stays pending behind the retained current child");
    let frozen=c.state.clone();assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);assert_eq!(c.state,frozen,"retry cannot grant Block twice or replace first fault");
}

#[test]
fn m10_ironclad_relics_registry_and_acquisition_are_ironclad_scoped(){
    let ids=[RelicId::RedSkull,RelicId::PaperPhrog,RelicId::ChampionsBelt,RelicId::CharonsAshes,RelicId::SelfFormingClay];
    for id in ids {assert!(crate::content::Catalog::standard().relic(id).is_some_and(|definition|!definition.definition_only));}
    let mut silent=Run::new(RunConfig{character:Character::Silent,ascension:0},CreationProfile::experienced(),RunRng::new([42;32],rng::Partition::Test,0)).unwrap();
    for id in ids {
        let before=silent.state().clone();
        assert_eq!(crate::items::acquire_relic(silent.state_mut_for_test(),id),Err(RunError::InvalidAction));
        assert_eq!(silent.state(),&before,"rejected off-character acquisition keeps RunState unchanged for {id:?}");
    }
}

#[test]
fn m10_ironclad_relics_red_skull_tracks_bloodied_strength_without_drift(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::RedSkull]);
    assert_eq!(c.state.player.hp,300);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),0);
    let source=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects((0..10).map(|_|Effect{source:Source::Actor(source.clone()),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(15))}).collect(),None).unwrap();
    assert_eq!(c.state.player.hp,150);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),3);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Heal(1)}],None).unwrap();
    assert_eq!(c.state.player.hp,151);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),0,"healing above half removes exactly the counted Strength");
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(source),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),3,"crossing down rearms once");
}

#[test]
fn m10_ironclad_relics_red_skull_enters_at_inclusive_half_hp(){
    let mut e=entry(EncounterId::OrbWalker);e.setup.player.hp=150;
    let c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::RedSkull]))).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),3);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_relics_red_skull_battle_start_directly_stacks_signed_strength(){
    let mut e=entry(EncounterId::OrbWalker);e.setup.player.hp=150;
    e.setup.player_powers=vec![FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:-5,skip_first:false,monster_source_grace:false}},FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}}];
    let control=e.clone();
    let base=Combat::enter(control.clone()).unwrap();
    let c=Combat::enter_with_item_context(control,&mut Some(acquired(&[RelicId::RedSkull]))).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,-2);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="artifact").unwrap().state.amount,1);
    assert_eq!(c.state.resolved_effects,base.state.resolved_effects,"already-bloodied battle-start Red Skull uses direct addPower, not an ApplyPowerAction child");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_relics_red_skull_artifact_consumes_queued_removal_before_latch_rearms(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::RedSkull]);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let source=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects((0..10).map(|_|Effect{source:Source::Actor(source.clone()),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(15))}).collect(),None).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,3);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Heal(1)}],None).unwrap();
    assert_eq!(c.state.player.hp,151);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="artifact").map_or(0,|p|p.state.amount),0);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,3,"negative Strength is an Artifact-eligible application in the native adapter; source callback clears its latch before queued application");
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(source),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,6,"the next bloodied callback rearms after the accepted Artifact-blocked child");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_relics_red_skull_crossing_commits_latch_before_strength_child_budget_fault(){
    let mut e=entry(EncounterId::OrbWalker);e.setup.player.hp=151;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::RedSkull]))).unwrap();
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::RedSkull),0);
    c.state.resolved_effects=99_998;
    engine::validate_runtime(&c.state).expect("canonical live combat before the crossing hit");
    let source=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let hit=Effect{source:Source::Actor(source),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(2))};
    c.preflight_native_effects(std::slice::from_ref(&hit)).unwrap();
    c.state.pending.push_front(hit);
    let transition=engine::resolve(&mut c.state);
    assert_eq!(transition.status,TransitionStatus::Fault,"the accepted threshold child reaches the existing effect budget fault");
    assert_eq!(c.state.player.hp,149,"the real hit committed before the callback child fault");
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::RedSkull),1,"Red Skull mutates its active latch before queued Strength work");
    assert!(c.state.fault.is_some());
    let frozen=c.state.clone();engine::resolve(&mut c.state);assert_eq!(c.state,frozen,"faulted callback is retained and never retried");
}

#[test]
fn m10_ironclad_relics_red_skull_rechecks_real_feed_max_hp_and_heal(){
    let mut e=super::m5_elites::entry(EncounterId::ThreeLouse);
    e.setup.player.hp=300;e.setup.player.max_hp=300;
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::Feed}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::RedSkull]))).unwrap();
    let enemy=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let to_five=c.state.enemies[0].hp-5;
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(enemy),kind:EffectKind::Damage(DamagePayload::Fixed(to_five))}],None).unwrap();
    c.enqueue_native_effects((0..10).map(|_|Effect{source:Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(15))}).collect(),None).unwrap();
    assert_eq!(c.state.player.hp,150);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),3);
    let max_before=c.state.player.max_hp;
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.player.max_hp,max_before+3);
    assert_eq!(c.state.player.hp,153,"Feed heals3 while the other actual Cultists keep combat live");
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").map_or(0,|p|p.state.amount),0,"Feed's actual max-HP gain/heal crosses the updated threshold without stacking drift");
}

#[test]
fn m10_ironclad_relics_paper_phrog_modifies_normal_vulnerable_damage_once(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::PaperPhrog]);
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    assert_eq!(cards::normal_damage(&c.state,8,0,false,&c.state.enemies[0]),Ok(14));
    assert_eq!(cards::normal_damage(&c.state,8,0,true,&c.state.enemies[0]),Ok(10),"Weak applies before the replacement Vulnerable factor");
    let before=c.state.enemies[0].hp;
    let instance=c.state.piles.hand[0].handle().as_str().parse::<u64>().unwrap();
    c.state.piles.hand[0].full_mut().unwrap().id=CardId::Strike;
    c.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.enemies[0].hp,before-10,"the real 6-damage Strike uses Paper Phrog's 1.75 receive factor");
    let mut absent=enter(EncounterId::OrbWalker,&[]);
    absent.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    assert_eq!(cards::normal_damage(&absent.state,8,0,false,&absent.state.enemies[0]),Ok(12));
    let before=absent.state.enemies[0].hp;
    let instance=absent.state.piles.hand[0].handle().as_str().parse::<u64>().unwrap();
    absent.state.piles.hand[0].full_mut().unwrap().id=CardId::Strike;
    absent.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
    assert_eq!(absent.state.enemies[0].hp,before-9,"without Paper Phrog, 6 damage under Vulnerable is 9");
}

#[test]
fn m10_ironclad_relics_champions_belt_requires_successful_vulnerable(){
    let mut e=entry(EncounterId::OrbWalker);e.setup.enemies[0].powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::ChampionsBelt]))).unwrap();
    let target=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}}],None).unwrap();
    assert_eq!(enemy_power(&c,0,"weak"),0);
    let target=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}}],None).unwrap();
    assert_eq!(enemy_power(&c,0,"weak"),1);
    let target=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}}],None).unwrap();
    assert_eq!(enemy_power(&c,0,"weak"),2,"successful stacked Vulnerable application queues its own Weak");

    let mut e=entry(EncounterId::OrbWalker);
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::Uppercut}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut from_card=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::ChampionsBelt]))).unwrap();
    from_card.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(enemy_power(&from_card,0,"vulnerable"),1);
    assert_eq!(enemy_power(&from_card,0,"weak"),2,"Uppercut's real card-owned Vulnerable application queues one Belt Weak in addition to Uppercut's own Weak");
}

#[test]
fn m10_ironclad_relics_champions_belt_requires_player_owned_action_not_null_or_potion_source(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::ChampionsBelt]);
    c.state.item_context=Some(CombatItemContext::new(vec![RelicId::ChampionsBelt],vec![Some(crate::content::PotionId::WeakPotion)]));
    for source in [Source::None,Source::Potion{slot:0,id:crate::content::PotionId::WeakPotion}] {
        let target=Actor::Enemy(c.state.enemies[0].instance_id.clone());
        c.enqueue_native_effects(vec![Effect{source:source.clone(),target:Some(target),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}}],None).unwrap();
        assert_eq!(enemy_power(&c,0,"weak"),0,"source={source:?} is not the player's successful Vulnerable action");
        assert!(engine::validate_runtime(&c.state).is_ok());
    }
}

#[test]
fn m10_ironclad_relics_champions_belt_bottom_weak_waits_behind_already_queued_work(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::ChampionsBelt]);
    let target=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let observer=observe::Session::combat(&c);
    let observed=observer.finish(c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}},Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None));
    observed.result.unwrap();
    let damage=observed.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::Damage{..})).unwrap();
    let weak=observed.effects.iter().position(|event|matches!(&event.kind,ResolvedEffectKind::Power{id,..} if id.0=="weak")).unwrap();
    assert!(damage<weak,"Champion Belt's onTrigger child is addToBot work and stays behind pre-existing queued work");
}

#[test]
fn m10_ironclad_relics_paper_phrog_does_not_amplify_orb_or_thorns_payloads(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::PaperPhrog]);
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    let owner=c.state.enemies[0].instance_id.clone();let before=c.state.enemies[0].hp;
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Damage(DamagePayload::Orb(8))},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),kind:EffectKind::Damage(DamagePayload::Thorns(8))}],None).unwrap();
    assert_eq!(c.state.enemies[0].hp,before-16,"Paper Phrog changes only Normal receive damage; Orb and THORNS remain unamplified");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_relics_clay_records_hp_loss_not_blocked_hits(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::SelfFormingClay]);
    assert!(engine::validate_runtime(&c.state).is_ok());
    let enemy=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.state.player.block=10;
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(enemy.clone()),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(5))},Effect{source:Source::Actor(enemy.clone()),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(12))},Effect{source:Source::Actor(enemy),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None).unwrap();
    assert_eq!(c.state.player.hp,292);
    assert_eq!(c.state.player.block,0);
    let start=c.step_full_observed(&CombatAction::EndTurn);
    start.result.unwrap();
    assert_eq!(c.state.player.block,9,"two prior positive HP-loss notifications plus the real enemy-turn loss accrue; the fully blocked hit does not");
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::SelfFormingClay),0,"paid pending block expires");
    let clay_block=start.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::Block{after:9,..})).unwrap();
    let first_draw=start.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::CardMove{after:PublicZone::Hand,..})).unwrap();
    assert!(clay_block<first_draw,"NextTurnBlockPower pays at player-start pre-draw, before ordinary draw");
}

#[test]
fn m10_ironclad_relics_clay_counts_actual_self_card_hp_loss_for_next_turn(){
    let mut e=entry(EncounterId::OrbWalker);
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::Hemokinesis}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::SelfFormingClay]))).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.player.hp,298,"Hemokinesis commits its own HP loss through shared damage ownership");
    assert_eq!(c.state.player.block,0,"Clay block is deferred");
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::SelfFormingClay),3);
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.player.block,6,"Hemokinesis and the subsequent real enemy HP-loss event both pay at the next player start");
}

#[test]
fn m10_ironclad_relics_clay_pending_gain_overflow_keeps_first_fault_and_counter(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::SelfFormingClay]);
    c.state.item_context.as_mut().unwrap().set_counter(RelicId::SelfFormingClay,999);
    let before=c.state.player.hp;
    let source=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let result=c.enqueue_native_effects(vec![Effect{source:Source::Actor(source),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(3))}],None);
    assert!(result.is_err(),"unsupported pending gain must retain an operational fault, not wrap or silently cap");
    assert_eq!(c.state.player.hp,before-3,"the accepted hit is not replayed or rolled back after its callback faults");
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::SelfFormingClay),999,"failed callback leaves the prior pending amount unchanged");
    assert!(c.state.fault.is_some(),"the callback fault remains owned by the combat");
}

#[test]
fn m10_ironclad_relics_clay_zero_and_repeated_pre_draw_hooks_pay_only_owned_pending_block(){
    let mut e=entry(EncounterId::OrbWalker);e.setup.player_powers.push(FullPowerEntry{id:FullPowerId("barricade"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::SelfFormingClay]))).unwrap();
    let source=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(source),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(0))}],None).unwrap();
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::SelfFormingClay),0,"zero damage is not an HP-loss event");
    c.state.item_context.as_mut().unwrap().set_counter(RelicId::SelfFormingClay,3);
    c.state.pending.clear();super::super::turns::schedule_next_player_turn(&mut c.state).unwrap();
    engine::validate_runtime(&c.state).expect("actual PlayerStart queue before pending payout");
    engine::resolve(&mut c.state);
    assert_eq!(c.state.player.block,3);
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::SelfFormingClay),0);
    c.state.pending.clear();super::super::turns::schedule_next_player_turn(&mut c.state).unwrap();
    engine::validate_runtime(&c.state).expect("subsequent real PlayerStart has no Clay event");
    engine::resolve(&mut c.state);
    assert_eq!(c.state.player.block,3,"repeated actual pre-draw hook cannot replay a drained payout");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_relics_clay_drained_payout_fault_is_retained_without_retry(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::SelfFormingClay]);
    c.state.item_context.as_mut().unwrap().set_counter(RelicId::SelfFormingClay,3);
    c.state.pending.clear();super::super::turns::schedule_next_player_turn(&mut c.state).unwrap();
    c.state.resolved_effects=99_998;
    engine::validate_runtime(&c.state).expect("valid pending Clay payout frontier");
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.item_context.as_ref().unwrap().counter(RelicId::SelfFormingClay),0,"payout ownership is drained before its accepted child dispatch");
    assert!(c.state.pending.iter().any(|work|work.kind==EffectKind::GainBlock(3)));
    engine::dispatch_one(&mut c.state);
    assert!(c.state.fault.is_some());
    assert_eq!(c.state.player.block,0);
    let frozen=c.state.clone();engine::dispatch_one(&mut c.state);assert_eq!(c.state,frozen,"faulted accepted block child is not retried");
}

#[test]
fn m10_ironclad_relics_charons_ashes_fires_per_real_exhaust_for_acquired_owner(){
    let mut e=super::m8_shapes::entry(EncounterId::DonuAndDeca);
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::SecondWind}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::CharonsAshes]))).unwrap();
    let before=c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),vec![before[0]-12,before[1]-12]);
    assert_eq!(c.state.piles.exhaust.len(),4);
}

#[test]
fn m10_ironclad_relics_charons_ashes_preserves_donu_deca_target_order_inside_exhaust_batch(){
    let mut e=super::m8_shapes::entry(EncounterId::DonuAndDeca);
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::SecondWind}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::CharonsAshes]))).unwrap();
    let enemy_order=c.state.enemies.iter().map(|enemy|Actor::Enemy(enemy.instance_id.clone())).collect::<Vec<_>>();
    assert_eq!(enemy_order.len(),2,"the regression uses the real two-enemy Donu/Deca formation");
    let observed=c.step_full_observed(&CombatAction::PlayCard{instance:1,target:None});
    observed.result.unwrap();
    let actual=observed.effects.iter().filter(|event|event.source==Source::None
        &&matches!(event.kind,ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..}))
        .map(|event|event.target.clone().unwrap()).collect::<Vec<_>>();
    let expected=(0..4).flat_map(|_|enemy_order.iter().cloned()).collect::<Vec<_>>();
    assert_eq!(actual,expected,"each source-qualified DamageAllEnemies action preserves formation iteration order, with all targets resolved before the next exhaust continuation");
}

#[test]
fn m10_ironclad_relics_charons_ashes_top_work_interrupts_second_wind_between_exhausts(){
    let mut e=entry(EncounterId::OrbWalker);
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::SecondWind}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::CharonsAshes]))).unwrap();
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("feel_no_pain"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}});
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dark_embrace"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    engine::validate_runtime(&c.state).unwrap();
    let result=c.step_full_observed(&CombatAction::PlayCard{instance:1,target:None});
    result.result.unwrap();
    let exhausted=result.effects.iter().enumerate().filter_map(|(i,event)|matches!(event.kind,ResolvedEffectKind::CardMove{after:PublicZone::Exhaust,..}).then_some(i)).collect::<Vec<_>>();
    let (first_exhaust,next_exhaust)=(exhausted[0],exhausted[1]);
    let ashes_damage=result.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..})).unwrap();
    assert_eq!(result.effects[ashes_damage].source,Source::None,"Charon's Ashes DamageAllEnemiesAction has a null DamageInfo source");
    assert!(first_exhaust<ashes_damage&&ashes_damage<next_exhaust,"Charon's Ashes addToTop damage must resolve between one batch exhaust and its next continuation");
    assert_eq!(c.state.player.block,32,"four Second Wind grants and four Feel No Pain callbacks resolve once each");
}

#[test]
fn m10_ironclad_relics_charons_ashes_committed_top_child_survives_effect_budget_fault(){
    let mut e=entry(EncounterId::OrbWalker);
    e.cards=(1..=5).map(|id|permanent_live_card(if id==1{CardId::SecondWind}else{CardId::Defend},CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::CharonsAshes]))).unwrap();
    let parent=c.state.piles.hand.remove(0);let owner=parent.handle();
    c.state.in_play=Some(parent);c.state.phase=Phase::PlayerAction;
    c.state.play_context=Some(PlayContext{card:owner.clone(),target:None,energy_on_use:0,paid:0,disposition:FinishDisposition::Exhaust});
    let begin=Effect{source:Source::Card(owner.clone()),target:None,kind:EffectKind::ExhaustBatchStart{filter:ExhaustBatchFilter::AllHand,random:false,finish:ExhaustBatchFinish::BlockPerExhaust(5)}};
    exhaust::begin_batch(&mut c.state,&begin,ExhaustBatchFilter::AllHand,false,ExhaustBatchFinish::BlockPerExhaust(5)).unwrap();
    c.state.pending.push_back(Effect{source:Source::Card(owner.clone()),target:None,kind:EffectKind::FinishCard});
    c.state.resolved_effects=99_997;
    engine::validate_runtime(&c.state).expect("actual in-play parent, issued hand candidates, committed exhaust, and owned continuation");
    engine::dispatch_one(&mut c.state);
    assert!(c.state.pending.iter().any(|work|matches!(work.kind,EffectKind::Damage(DamagePayload::Thorns(3)))));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.resolved_effects,99_999,"first Ashes child commits before the next effect-budget checkpoint");
    assert_eq!(c.state.enemies[0].hp,93);
    assert_eq!(c.state.piles.exhaust.len(),1,"committed callback source remains owned by Exhaust");
    assert!(c.state.pending.iter().any(|work|matches!(work.kind,EffectKind::ExhaustBatchContinue(_))),"batch continuation remains owned behind the accepted relic child");
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard,"existing finite dispatch budget is the first retained fault");
    let frozen=c.state.clone();engine::dispatch_one(&mut c.state);assert_eq!(c.state,frozen,"accepted Ashes damage and source exhaust are not retried after first fault");
}
