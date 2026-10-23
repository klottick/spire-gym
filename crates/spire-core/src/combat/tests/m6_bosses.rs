use super::super::*;
use crate::content::EncounterId;
use super::m5_elites::{hit,amount};

fn champ()->Combat{let mut e=super::m6_ordinary::entry(EncounterId::Champ);e.setup.player.hp=200;e.setup.player.max_hp=200;Combat::enter(e).unwrap()}
fn cursor(c:&mut Combat,index:u128){let old=c.state.enemies[0].full_ai_rng.as_ref().unwrap();c.state.enemies[0].full_ai_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],index).unwrap());}
fn roll(c:&mut Combat){let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();}
fn act(c:&mut Combat){let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();}
fn power(c:&mut Combat,id:&'static str,n:i32){let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor),kind:EffectKind::ApplyFullPower{id:FullPowerId(id),amount:n}}],None).unwrap();}

fn phase_two()->Combat {
    let mut c=champ();assert_eq!(c.state.enemies[0].hp,420);let initial=c.state.enemies[0].intent_id.clone();hit(&mut c,0,210);assert_eq!(c.state.enemies[0].intent_id,initial,"damage alone is not an intent interrupt");
    cursor(&mut c,33);roll(&mut c);assert_ne!(c.state.enemies[0].intent_id.as_str(),"champ_anger","exactly210 is NOT below half");
    let captured=c.state.enemies[0].intent_id.clone();hit(&mut c,0,1);assert_eq!(c.state.enemies[0].intent_id,captured);cursor(&mut c,1_u128<<64);let before=c.state.enemies[0].full_ai_rng.clone();roll(&mut c);assert_eq!(c.state.enemies[0].full_ai_rng,before);assert_eq!(c.state.enemies[0].intent_id.as_str(),"champ_anger");
    cursor(&mut c,33);act(&mut c);assert_eq!((amount(&c,0,"strength"),c.state.enemies[0].intent_id.as_str()),(6,"champ_execute"));c
}

#[test]
fn m6_champ_threshold_is_strict_and_transition_is_once() {
    let mut c=phase_two();let mut replay=phase_two();assert_eq!(c.snapshot(),replay.snapshot());
    for (hp,next) in [(168,"champ_slash"),(146,"champ_slap"),(128,"champ_execute")] {
        cursor(&mut c,33);cursor(&mut replay,33); // Same explicit bucket99 at each ordinary phase-two selection.
        c.step_full(&CombatAction::EndTurn).unwrap();replay.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.snapshot(),replay.snapshot());assert_eq!((c.state.player.hp,c.state.enemies[0].intent_id.as_str()),(hp,next));
    }
    assert_eq!(amount(&c,0,"strength"),6);assert_eq!(c.state.enemies[0].move_history.iter().filter(|m|m.as_str()=="champ_anger").count(),1);
}

#[test]
fn m6_champ_cleanse_keeps_buffs_and_refreshes_damage() {
    let mut c=champ();for (id,n) in [("weak",2),("vulnerable",3),("strength",2),("metallicize",5)]{power(&mut c,id,n)}
    hit(&mut c,0,211);roll(&mut c);assert_eq!(c.state.enemies[0].intent_id.as_str(),"champ_anger");act(&mut c);
    assert_eq!((amount(&c,0,"weak"),amount(&c,0,"vulnerable"),amount(&c,0,"strength"),amount(&c,0,"metallicize")),(0,0,8,5));assert_eq!(c.state.enemies[0].intent_damage,Some(18));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].block),(164,5),"Execute is two refreshed18 hits; retained Metallicize still triggers normally");
}

#[test]
fn m6_champ_stance_gloat_slap_and_fourth_turn_taunt_use_real_payloads() {
    let mut c=champ();cursor(&mut c,23);roll(&mut c);assert_eq!(c.state.enemies[0].intent_id.as_str(),"champ_stance");cursor(&mut c,33);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].block,amount(&c,0,"metallicize")),(200,20,5));
    let before=c.state.enemies[0].full_ai_rng.clone();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].intent_id.as_str()),(184,"champ_taunt"));assert_eq!(c.state.enemies[0].full_ai_rng,before,"forced fourth selection draws no probabilistic branch");
    c.step_full(&CombatAction::EndTurn).unwrap();assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="weak"&&p.state.amount==2));assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="vulnerable"&&p.state.amount==2));
    let mut g=champ();cursor(&mut g,0);roll(&mut g);assert_eq!(g.state.enemies[0].intent_id.as_str(),"champ_gloat");g.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&g,0,"strength"),2);
    let mut slap=champ();cursor(&mut slap,37);roll(&mut slap);assert_eq!(slap.state.enemies[0].intent_id.as_str(),"champ_slap");slap.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(slap.state.player.hp,188);assert!(slap.state.player_full_powers.iter().any(|p|p.id.0=="frail"&&p.state.amount==2));assert!(slap.state.player_full_powers.iter().any(|p|p.id.0=="vulnerable"&&p.state.amount==2));
}

#[test]
fn m6_champ_first_threshold_precedes_the_fourth_selection_taunt() {
    let mut c=champ();cursor(&mut c,33);c.step_full(&CombatAction::EndTurn).unwrap();cursor(&mut c,33);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].intent_id.as_str(),"champ_slash");
    hit(&mut c,0,211);cursor(&mut c,1_u128<<64);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,148);assert_eq!(c.state.enemies[0].intent_id.as_str(),"champ_anger");assert_eq!(c.state.enemies[0].full_ai_rng.as_ref().unwrap().word_index(),1_u128<<64);
}

#[test]
fn m6_champ_partial_anger_keeps_cleanse_and_phase_before_strength_fault() {
    let mut c=champ();for (id,n) in [("weak",2),("vulnerable",3),("strength",2),("metallicize",5)]{power(&mut c,id,n)}hit(&mut c,0,211);roll(&mut c);
    let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind:EffectKind::EnemyMove});
    for _ in 0..20{if amount(&c,0,"weak")==0&&amount(&c,0,"vulnerable")==0&&c.state.pending.front().is_some_and(|e|e.kind==EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:6}){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!((amount(&c,0,"weak"),amount(&c,0,"vulnerable"),amount(&c,0,"strength"),amount(&c,0,"metallicize")),(0,0,2,5));c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:6});assert_eq!(c.state.enemies[0].intent_id.as_str(),"champ_anger");assert!(c.state.pending.iter().any(|e|e.kind==EffectKind::RollMove));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_champ_cleanse_removes_poison_signed_debuffs_and_explicit_shackled() {
    let mut c=champ();for (id,n) in [("poison",4),("strength",-2),("dexterity",-1),("shackled",2),("metallicize",5),("artifact",1)]{power(&mut c,id,n)}
    hit(&mut c,0,211);roll(&mut c);act(&mut c);for id in ["poison","dexterity","shackled"]{assert_eq!(amount(&c,0,id),0);}
    assert_eq!((amount(&c,0,"strength"),amount(&c,0,"metallicize"),amount(&c,0,"artifact")),(6,5,1),"negative Strength is removed, not added to6; real buffs remain");
}

#[test]
fn m6_champ_unknown_power_category_faults_before_any_cleanse_removal() {
    let mut c=champ();power(&mut c,"weak",2);power(&mut c,"unknown_category_fixture",1);hit(&mut c,0,211);roll(&mut c);
    let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());assert!(c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).is_err());
    assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::Unsupported);assert_eq!((amount(&c,0,"weak"),amount(&c,0,"unknown_category_fixture"),amount(&c,0,"strength")),(2,1,0));assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::ChampCleanse);
    assert!(matches!(c.state.enemies[0].full_memory,Some(EnemyMemory::Champ{angered:true,..})));let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_champ_exact_buckets_repeat_limits_and_two_forge_cap() {
    use crate::content::EnemyId;
    let owner=EnemyHandle::parse("champ-fixture").unwrap();let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),owner.as_str().into()]).unwrap();
    let context=enemies::act2::EnemyRollContext{owner:owner.clone(),turn:1,actors:vec![enemies::act2::ActorSummary{owner,species:EnemyId::Champ,hp:420,max_hp:420,slot:0}]};
    for (history,turns,forges,cursor,want,after) in [
        (vec![],0,0,225,"champ_stance",1),(vec![],0,0,44,"champ_gloat",0),(vec![],0,0,356,"champ_gloat",0),(vec![],0,0,190,"champ_slap",0),(vec![],0,0,90,"champ_slap",0),(vec![],0,0,77,"champ_slash",0),
        (vec!["champ_stance"],1,1,23,"champ_slap",1),(vec!["champ_gloat"],1,0,44,"champ_slap",0),(vec!["champ_slap"],1,0,190,"champ_slash",0),(vec!["champ_slash"],1,0,77,"champ_slap",0),
        (vec!["champ_stance","champ_slap","champ_stance","champ_taunt"],0,2,23,"champ_gloat",2)
    ] {
        let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
        let selected=enemies::act2::select_move(EnemyId::Champ,&history,&EnemyMemory::Champ{turns,forges,angered:false},&stream,&context).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,EnemyMemory::Champ{turns:turns+1,forges:after,angered:false});assert_eq!(selected.stream.word_index(),cursor+1);
    }
}

fn collector()->Combat{let mut e=super::m6_ordinary::entry(EncounterId::Collector);e.setup.player.hp=200;e.setup.player.max_hp=200;Combat::enter(e).unwrap()}

#[test]
fn m6_collector_spawn_refills_only_missing_slots() {
    let mut c=collector();assert_eq!(c.state.enemies[0].hp,282);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies.len(),3);assert_eq!(c.state.player.hp,200);
    for head in &c.state.enemies[1..]{assert_eq!(head.enemy_id.as_str(),"torch_head");assert!((38..=40).contains(&head.hp));assert_eq!(head.hp,head.max_hp);assert_eq!(head.move_history.len(),1);assert!(head.full_powers.iter().any(|p|p.id.0=="minion"));}
    let survivor=c.state.enemies[2].clone();
    for (dead,next) in [(1,3),(3,4)] {
        hit(&mut c,dead,50);assert_eq!(c.state.outcome,None,"killing a head cannot win while Collector lives");cursor(&mut c,23);roll(&mut c);assert_eq!(c.state.enemies[0].intent_id.as_str(),"collector_revive");act(&mut c);
        assert_eq!(c.state.enemies.len(),next+1);assert_eq!(c.state.enemies[2],survivor);assert_eq!(c.state.enemies[dead].hp,0);assert!(!c.public_full().unwrap().targets[dead].1);assert!(c.state.enemies[next].hp>0);
        assert_ne!(c.state.enemies[dead].instance_id,c.state.enemies[next].instance_id);
    }
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatStarted).count(),1,"children do not restart the battle");
}

#[test]
fn m6_collector_fourth_turn_debuff_has_separate_artifact_applications() {
    for artifact in [0,1,3] {
        let mut c=collector();if artifact>0{let actor=Actor::Player;c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor),kind:EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:artifact}}],None).unwrap();}
        cursor(&mut c,33);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].intent_id.as_str(),"collector_buff");
        cursor(&mut c,33);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].block),(186,15));assert_eq!((amount(&c,0,"strength"),amount(&c,1,"strength"),amount(&c,2,"strength")),(3,3,3));assert_eq!(c.state.enemies[0].intent_id.as_str(),"collector_rake");
        cursor(&mut c,33);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,145);assert_eq!(c.state.enemies[0].intent_id.as_str(),"collector_ultimate");
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,125);
        for (id,present) in [("weak",artifact==0),("vulnerable",artifact<3),("frail",artifact<3)]{assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0==id).map(|p|p.state.amount),if present{Some(3)}else{None});}
    }
}

#[test]
fn m6_collector_fourth_turn_debuff_and_minion_cleanup() {
    let mut c=collector();c.step_full(&CombatAction::EndTurn).unwrap();let damage_hooks=c.state.hook_events.iter().filter(|e|e.hook==Hook::Damage).count();
    let boss=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.state.phase=Phase::EnemyTurn;c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(boss),kind:EffectKind::Damage(DamagePayload::Fixed(300))});
    for head in &c.state.enemies[1..]{c.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Enemy(head.instance_id.clone())),kind:EffectKind::EnemyMove});}c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndRound});
    engine::dispatch_one(&mut c.state);assert!(!enemies::act2::combat_victory_ready(&c.state),"living tagged heads are not silently ignored while actual source suicides remain pending");assert!(!c.state.pending.iter().any(|e|matches!(e.kind,EffectKind::Terminal(_))));
    engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.player.hp,200,"queued adds cannot attack after the real Collector cleanup");
    assert!(c.state.enemies.iter().all(|e|e.hp==0));assert!(c.state.enemies.iter().all(|e|e.life==ActorLife::Active),"source suicides are neither Slime replacement nor Gremlin escape");
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Damage).count(),damage_hooks+1,"suicide cleanup is not invented fatal NORMAL damage");
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),3,"Collector's actual die(true) suicides deliver add-death callbacks, without damage/card-fatal ownership");
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatStarted).count(),1);assert_eq!(c.exit().unwrap().outcome,Outcome::Victory);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_collector_cleanup_does_not_turn_boss_hp_into_a_global_victory_shortcut() {
    let mut c=collector();c.step_full(&CombatAction::EndTurn).unwrap();
    let other=Combat::enter(super::m5_elites::entry(EncounterId::Cultist)).unwrap();let mut actor=other.state.enemies[0].clone();actor.instance_id=EnemyHandle::parse("ordinary-survivor").unwrap();actor.hp=48;actor.max_hp=48;
    let current=c.state.enemies[0].full_ai_rng.as_ref().unwrap();let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec![current.id().scope()[0].clone(),actor.instance_id.as_str().into()]).unwrap();actor.full_ai_rng=Some(rng::StreamState::new(id,[19;32],0).unwrap());
    c.state.actor_slots.insert(actor.instance_id.clone(),3);c.state.enemies.push(actor);
    let boss=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(boss),kind:EffectKind::Damage(DamagePayload::Fixed(300))});engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.outcome,None);assert_eq!(c.state.enemies[3].hp,48);assert!(!c.state.pending.iter().any(|e|matches!(e.kind,EffectKind::Terminal(_))),"a still-living ordinary non-Minion blocks immediate engine victory; boss HP0 alone cannot settle");
    engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.enemies[3].hp,0,"Collector's explicit source suicide script, not a boss-HP predicate, can kill other room actors");assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);
}

#[test]
fn m6_collector_partial_spawn_fault_is_not_a_gameplay_death_or_second_battle_start() {
    let mut c=collector();let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});
    for _ in 0..16{if c.state.enemies.len()==2&&c.state.pending.front().is_some_and(|e|e.kind==EffectKind::CompleteSpawn){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!(c.state.enemies.len(),2);let first=c.state.enemies[1].clone();let stream=&c.state.spawn_progress.as_ref().unwrap().constructor_rng;c.state.spawn_progress.as_mut().unwrap().constructor_rng=rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap();
    engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);assert_eq!(c.state.enemies[1],first);assert_eq!((c.state.player.hp,c.state.enemies[0].hp,c.state.outcome),(200,282,None));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatStarted).count(),1);assert!(!c.state.hook_events.iter().any(|e|e.hook==Hook::CombatSettled));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_collector_queued_suicides_preserve_accepted_player_healing() {
    let mut e=super::m6_ordinary::entry(EncounterId::Collector);e.setup.player.hp=150;e.setup.player.max_hp=200;
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();hit(&mut c,0,281);assert_eq!(c.state.enemies[0].hp,1);
    let mut work=vec![];for (slot,n) in [(0,1),(1,4),(2,4)]{work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(c.state.enemies[slot].instance_id.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(n))});}
    work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Heal(9)});
    c.enqueue_native_effects(work,None).unwrap();assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.player.hp,159,"already accepted player damage/heal work resolves before separate suicide cleanup; no card handler is fabricated");assert_eq!(c.exit().unwrap().hp,159);
}

#[test]
fn m6_collector_exact_revive_rake_cutoffs_and_repeat_limits() {
    use crate::content::EnemyId;
    let owner=EnemyHandle::parse("collector-fixture").unwrap();let first=EnemyHandle::parse("torch-first").unwrap();let second=EnemyHandle::parse("torch-second").unwrap();
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),owner.as_str().into()]).unwrap();let slots=[(0,first.clone()),(1,second.clone())].into_iter().collect::<std::collections::BTreeMap<_,_>>();
    for (missing,history,cursor,want) in [
        (true,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_rake"],34,"collector_revive"),
        (true,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_rake"],67,"collector_rake"),
        (true,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_rake"],285,"collector_rake"),
        (true,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_rake"],42,"collector_buff"),
        (true,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_revive"],34,"collector_rake"),
        (false,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_rake"],34,"collector_rake"),
        (false,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_rake","collector_rake"],285,"collector_buff"),
        (false,vec!["collector_spawn","collector_buff","collector_rake","collector_ultimate","collector_buff"],33,"collector_rake")
    ] {
        let actors=vec![enemies::act2::ActorSummary{owner:owner.clone(),species:EnemyId::Collector,hp:282,max_hp:282,slot:2},enemies::act2::ActorSummary{owner:first.clone(),species:EnemyId::TorchHead,hp:if missing{0}else{39},max_hp:39,slot:0},enemies::act2::ActorSummary{owner:second.clone(),species:EnemyId::TorchHead,hp:39,max_hp:39,slot:1}];
        let context=enemies::act2::EnemyRollContext{owner:owner.clone(),turn:6,actors};let memory=EnemyMemory::Collector{turns_taken:history.len() as u32,ult_used:true,initial_spawn:false,slots:slots.clone()};let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
        let selected=enemies::act2::select_move(EnemyId::Collector,&history,&memory,&stream,&context).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,memory);assert_eq!(selected.stream.word_index(),cursor+1);
    }
}

#[test]
fn m6_collector_foreign_slot_custody_rejects_before_inventory_transfer() {
    let mut e=super::m6_ordinary::entry(EncounterId::Collector);e.setup.enemies[0].initial_memory=Some(EnemyMemory::Collector{turns_taken:1,ult_used:false,initial_spawn:false,slots:[(0,EnemyHandle::parse("not-owned-by-this-combat").unwrap())].into_iter().collect()});e.setup.enemies[0].intent=FullIntentId::from_name("collector_revive");
    let mut items=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::Anchor],vec![]));let before=items.clone();assert!(matches!(Combat::enter_with_item_context(e,&mut items),Err(CombatEntryFailure::Rejected(_))));assert_eq!(items,before);
}

#[test]
fn m6_automaton_six_step_cycle_and_independent_orbs() {
    let mut c=super::m6_stasis::combat(vec![permanent_live_card(crate::content::CardId::Strike,CardInstanceId(1),0)]);
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"artifact")),(300,3));assert_eq!(c.state.enemies[0].intent_id.as_str(),"automaton_flail");
    assert_ne!(c.state.enemies[1].full_ai_rng.as_ref().unwrap().id(),c.state.enemies[2].full_ai_rng.as_ref().unwrap().id());
    for orb in &c.state.enemies[1..]{assert_eq!(orb.enemy_id.as_str(),"bronze_orb");assert!((52..=58).contains(&orb.hp));assert_eq!(orb.move_history.len(),1);}
    hit(&mut c,1,100);hit(&mut c,2,100);let ai=c.state.enemies[0].full_ai_rng.clone();
    for (hp,strength,block,next) in [(286,0,0,"automaton_boost"),(286,3,9,"automaton_flail"),(266,3,0,"automaton_boost"),(266,6,9,"automaton_beam"),(215,6,0,"automaton_stun"),(215,6,0,"automaton_flail"),(189,6,0,"automaton_boost")] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,amount(&c,0,"strength"),c.state.enemies[0].block,c.state.enemies[0].intent_id.as_str()),(hp,strength,block,next));
    }
    assert_eq!(c.state.enemies.len(),3,"the opening summon is not repeated after Stun");assert_eq!(c.state.enemies[0].full_ai_rng,ai,"deterministic cycle/derived child streams do not spend ignored AI entropy");
}
