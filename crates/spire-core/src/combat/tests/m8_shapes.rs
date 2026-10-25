use super::super::*;
use crate::content::{CardId, Character, EncounterId, EnemyId};
use super::m5_elites::{amount, hit};

pub(super) fn entry(encounter: EncounterId) -> CombatEntry {
    enemies::setup(encounter, 3, 0,
        &mut crate::run::RunRng::new([41;32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![] },
            player_powers:vec![],character:Character::Ironclad,
            cards:vec![permanent_live_card(CardId::Defend,CardInstanceId(1),0)],
            opening_draw:1,encounter_visit:0,
        }).unwrap()
}

fn supplied_actor(actor:&mut FullEnemySetup,id:EnemyId) {
    actor.enemy_id=id;actor.profile=enemies::EnemyProfile::Standard;
    actor.hp=match id {EnemyId::OrbWalker=>90,EnemyId::Spiker=>42,EnemyId::Exploder=>30,EnemyId::Repulsor=>29,_=>unreachable!()};
    actor.max_hp=actor.hp;actor.block=0;
    actor.powers=match id {EnemyId::OrbWalker=>vec![power("strength_up",3)],EnemyId::Spiker=>vec![power("thorns",3)],EnemyId::Exploder=>vec![power("explosive",3)],_=>vec![]};
    let memory=enemies::EnemyMemory::for_enemy(id).unwrap();
    let selected=enemies::select_move(id,&[],&memory,&actor.ai_rng).unwrap();
    actor.intent=selected.intent;actor.initial_memory=Some(selected.memory);actor.ai_rng=selected.stream;
}
pub(super) fn single(id:EnemyId)->CombatEntry {
    let mut e=entry(EncounterId::OrbWalker);supplied_actor(&mut e.setup.enemies[0],id);e.setup.enemy_damage_bases.clear();e
}
fn power(id:&'static str,amount:i32)->FullPowerEntry {
    FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}
}
fn select(c:&mut Combat,slot:usize,name:&'static str) {
    let value=ActorIntentId::Full(FullIntentId::from_name(name));
    let actor=&mut c.state.enemies[slot];actor.intent_id=value.clone();actor.selected_move=value.clone();actor.move_history=vec![value];
    effects::refresh_enemy_values(&mut c.state).unwrap();
}

// Missing native setup, wrong status destination or per-player-action growth breaks this.
#[test]
fn m8_orb_growth_and_burn_destinations() {
    let mut c=Combat::enter(entry(EncounterId::OrbWalker)).unwrap();
    assert!((90..=96).contains(&c.state.enemies[0].hp));assert_eq!(amount(&c,0,"strength_up"),3);
    let card_stream=c.state.card_rng.clone();let shuffle_stream=c.state.rng.shuffle.clone();
    select(&mut c,0,"orb_laser");c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((c.state.player.hp,amount(&c,0,"strength")),(290,3));
    assert_eq!(c.state.piles.hand.iter().filter(|c|c.full().unwrap().id==CardId::Burn).count(),1);
    assert_eq!(c.state.piles.discard.iter().filter(|c|c.full().unwrap().id==CardId::Burn).count(),1);
    assert_eq!(c.state.card_rng,card_stream,"Shapes' first Burn enters empty draw deterministically");
    assert_eq!(c.state.rng.shuffle,shuffle_stream,"Shapes status insertion is not opening/combat shuffle");
    assert!(c.state.piles.hand.iter().chain(&c.state.piles.discard).filter(|c|c.full().unwrap().id==CardId::Burn).all(|c|matches!(c.full().unwrap().origin,CombatCardRef::CombatLocal(_))));
    let enemy=&c.state.enemies[0];let want=match enemy.intent_id.as_str(){"orb_laser"=>13,"orb_claw"=>18,_=>panic!("wrong ordinary intent")};
    assert_eq!(enemy.intent_damage,Some(want));
}

#[test]
fn m8_shapes_use_qualified_ordered_composition() {
    for (encounter,key,count) in [(EncounterId::ThreeShapes,28,3),(EncounterId::FourShapes,29,4),(EncounterId::SphereAndTwoShapes,30,3)] {
        let e=entry(encounter);assert_eq!(e.setup.enemies.len(),count);
        let mut expected_rng=crate::run::RunRng::new([41;32],rng::Partition::Test,0);
        let mut pool=vec![EnemyId::Repulsor,EnemyId::Repulsor,EnemyId::Exploder,EnemyId::Exploder,EnemyId::Spiker,EnemyId::Spiker];
        for (slot,actor) in e.setup.enemies.iter().enumerate() {
            let want=if encounter==EncounterId::SphereAndTwoShapes {
                if slot==2 {EnemyId::SphericGuardian} else {let n=expected_rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,&[3,key,0,slot as u64,0],3);[EnemyId::Spiker,EnemyId::Repulsor,EnemyId::Exploder][n]}
            } else {let n=expected_rng.bounded_scoped(crate::run::RunRngScope::CombatEnemySetup,&[3,key,0,slot as u64,0],pool.len() as u64);pool.remove(n)};
            assert_eq!(actor.enemy_id,want);
            for prior in &e.setup.enemies[..slot] {assert_ne!(actor.instance_id,prior.instance_id);assert_ne!(actor.ai_rng.id(),prior.ai_rng.id());}
            let (low,high)=match actor.enemy_id {EnemyId::Repulsor=>(29,35),EnemyId::Exploder=>(30,30),EnemyId::Spiker=>(42,56),EnemyId::SphericGuardian=>(20,20),_=>unreachable!()};
            assert!((low..=high).contains(&actor.hp));
        }
        Combat::enter(e).unwrap();
    }
}

#[test]
fn m8_spiker_thorns_per_hit_and_six_buffs() {
    let mut e=single(EnemyId::Spiker);e.cards[0]=permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(owner),kind:EffectKind::GainBlock(20)},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(5)}],None).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,c.state.player.hp,c.state.player.block),(42,10,299,0));
    hit(&mut c,0,0);assert_eq!(c.state.player.hp,296,"owned zero-damage NORMAL still invokes onAttacked");
    for (turn,thorns) in [(1,5),(2,7),(3,9),(4,11),(5,13),(6,15)] {
        select(&mut c,0,"spiker_buff");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"thorns"),thorns,"executed buff {turn}");
    }
    assert_eq!(c.state.enemies[0].intent_id.as_str(),"spiker_attack");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].intent_id.as_str(),"spiker_attack");assert_eq!(c.state.player.hp,289);
}

#[test]
fn m8_lethal_spiker_hit_keeps_already_earned_retaliation() {
    let mut c=Combat::enter(single(EnemyId::Spiker)).unwrap();c.state.player.hp=2;
    hit(&mut c,0,42);assert_eq!((c.state.enemies[0].hp,c.state.player.hp,c.state.outcome),(0,0,Some(Outcome::Death)));
}

#[test]
fn m8_exploder_countdown_and_early_kill() {
    let mut c=Combat::enter(single(EnemyId::Exploder)).unwrap();
    for (hp,countdown,next) in [(291,2,"exploder_attack"),(282,1,"exploder_explode")] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,amount(&c,0,"explosive"),c.state.enemies[0].intent_id.as_str()),(hp,countdown,next));
    }
    let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(owner.clone()),target:Some(owner.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:20}},Effect{source:Source::Actor(Actor::Player),target:Some(owner),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(7)}],None).unwrap();
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].hp,c.state.outcome),(259,0,Some(Outcome::Victory)));
    let mut early=Combat::enter(single(EnemyId::Exploder)).unwrap();hit(&mut early,0,30);assert_eq!((early.state.player.hp,early.state.outcome),(300,Some(Outcome::Victory)));
}

#[test]
fn m8_explosion_suicide_precedes_damage_without_erasing_death_or_pending_fault() {
    for lethal in [false,true] {
        let mut c=Combat::enter(single(EnemyId::Exploder)).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
        if lethal {c.state.player.hp=20;}
        // Inspect the real queued source sequence at the native enemy boundary.
        c.state.phase=Phase::EnemyTurn;
        let owner=c.state.enemies[0].instance_id.clone();enemies::act(&mut c.state,&owner).unwrap();
        for _ in 0..32 {if c.state.enemies[0].hp==0{break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
        assert_eq!(c.state.enemies[0].hp,0);
        assert_eq!(c.state.outcome,None,"suicide alone is not settlement while its THORNS payload is owned");
        if lethal {engine::resolve(&mut c.state);assert_eq!((c.state.player.hp,c.state.outcome),(0,Some(Outcome::Death)));}
        else {
            c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_some());
            assert_eq!((c.state.enemies[0].hp,c.state.player.hp,c.state.outcome),(0,282,None));
            assert!(c.state.pending.iter().chain(c.state.current.iter()).any(|e|matches!(e.kind,EffectKind::Damage(DamagePayload::Thorns(30)))));
        }
    }
}

#[test]
fn m8_repulsor_creates_only_two_temporary_draw_statuses_and_live_ally_keeps_combat_open() {
    let mut e=entry(EncounterId::SphereAndTwoShapes);e.setup.enemies.truncate(2);
    supplied_actor(&mut e.setup.enemies[0],EnemyId::Exploder);supplied_actor(&mut e.setup.enemies[1],EnemyId::Repulsor);e.setup.enemy_damage_bases.clear();
    let mut c=Combat::enter(e).unwrap();let card_stream=c.state.card_rng.clone();let shuffle_stream=c.state.rng.shuffle.clone();select(&mut c,1,"repulsor_daze");c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.card_rng,card_stream,"Repulsor's empty then singleton insertions consume no native card RNG");
    assert_eq!(c.state.rng.shuffle,shuffle_stream,"Repulsor status insertion is not opening/combat shuffle");
    let all=c.state.piles.hand.iter().chain(&c.state.piles.draw).chain(&c.state.piles.discard).chain(&c.state.piles.exhaust).collect::<Vec<_>>();
    assert_eq!(all.iter().filter(|c|c.full().unwrap().id==CardId::Dazed).count(),2);
    assert_eq!(all.iter().filter(|c|matches!(c.full().unwrap().origin,CombatCardRef::Permanent(_))).count(),1);
    for _ in 0..2 {select(&mut c,1,"repulsor_daze");c.step_full(&CombatAction::EndTurn).unwrap();}
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[1].hp,c.state.player.hp,c.state.outcome),(0,29,252,None));
    assert_eq!(c.state.phase,Phase::Decision);assert!(c.exit().is_err());
    let view=c.public_full().unwrap();assert_eq!(view.enemies.iter().filter(|e|e.hp>0).count(),1);
}


#[test]
fn m8_shapes_invalid_setup_is_precommit_and_exhausted_live_roll_retains_damage() {
    let e=entry(EncounterId::OrbWalker);
    for (act,ascension) in [(2,0),(3,1)] {
        let mut rng=crate::run::RunRng::new([41;32],rng::Partition::Test,0);let before=rng.clone();
        let result=enemies::setup(EncounterId::OrbWalker,act,ascension,&mut rng,enemies::SetupContext{player:e.setup.player.clone(),player_powers:vec![],character:Character::Ironclad,cards:e.cards.clone(),opening_draw:1,encounter_visit:0});
        assert!(result.is_err());assert_eq!(rng,before);
    }
    let mut c=Combat::enter(e).unwrap();select(&mut c,0,"orb_claw");
    let stream=c.state.enemies[0].full_ai_rng.as_ref().unwrap();c.state.enemies[0].full_ai_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],1_u128<<64).unwrap());
    assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.state.player.hp,285);
    assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::RollMove);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m8_shapes_conditional_ai_redirects_and_executed_counters_are_not_flat_rates() {
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"shapes".into()]).unwrap();
    let stream=|cursor|rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
    // Boundary input comes from the real RNG; the oracle is the named integer law,
    // not a second call to the selector or an expected value copied from its output.
    for cursor in 0..16 {
        let s=stream(cursor);let (bucket,next)=rng::uniform_below(&s,100).unwrap();
        for (species,memory,history,want) in [
            (EnemyId::OrbWalker,EnemyMemory::OrbWalker,vec!["orb_claw","orb_claw"],"orb_laser"),
            (EnemyId::OrbWalker,EnemyMemory::OrbWalker,vec!["orb_laser","orb_laser"],"orb_claw"),
            (EnemyId::Repulsor,EnemyMemory::Repulsor,vec!["repulsor_attack"],"repulsor_daze"),
            (EnemyId::Repulsor,EnemyMemory::Repulsor,vec!["repulsor_daze"],if bucket<20{"repulsor_attack"}else{"repulsor_daze"}),
            (EnemyId::Spiker,EnemyMemory::Spiker{buffs:5},vec!["spiker_attack"],"spiker_buff"),
            (EnemyId::Spiker,EnemyMemory::Spiker{buffs:5},vec!["spiker_buff"],if bucket<50{"spiker_attack"}else{"spiker_buff"}),
        ] {
            let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::select_move(species,&history,&memory,&s).unwrap();
            assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,next);assert_eq!(selected.memory,memory);
        }
    }
    let exhausted=stream(1_u128<<64);
    for (species,memory,want) in [(EnemyId::Spiker,EnemyMemory::Spiker{buffs:6},"spiker_attack"),(EnemyId::Exploder,EnemyMemory::Exploder{turns:2},"exploder_explode")] {
        let selected=enemies::select_move(species,&[],&memory,&exhausted).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,exhausted);
    }
}
