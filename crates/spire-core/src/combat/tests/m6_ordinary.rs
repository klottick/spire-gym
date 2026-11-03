use super::super::*;
use crate::content::{CardId,Character,EncounterId};
use super::m5_elites::{amount,hit};

pub(super) fn entry(encounter:EncounterId)->CombatEntry {
    enemies::setup(encounter,2,0,&mut crate::run::RunRng::new([41;32],rng::Partition::Test,0),enemies::SetupContext {
        player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:Character::Ironclad,
        cards:vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0,
    }).unwrap()
}
fn intent(c:&Combat)->&str{c.state.enemies[0].intent_id.as_str()}

// Removing Barricade's native Block-clear exclusion must break this cycle.
#[test]
fn m6_sphere_retains_40_then_65_block_and_artifact3() {
    let mut c=Combat::enter(entry(EncounterId::SphericGuardian)).unwrap();
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"artifact")),(20,40,3));assert_eq!(amount(&c,0,"barricade"),1);
    for (hp,block,next) in [(80,65,"sphere_debuff"),(70,65,"sphere_slam"),(50,65,"sphere_harden"),(40,80,"sphere_slam")] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].block,intent(&c)),(hp,block,next));
    }
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="frail"&&p.state.amount==3));
}

#[test]
fn m6_sphere_artifact_blocks_real_disarm_without_losing_block() {
    let mut e=entry(EncounterId::SphericGuardian);e.cards[0]=permanent_live_card(CardId::Disarm,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!((amount(&c,0,"artifact"),amount(&c,0,"strength"),c.state.enemies[0].block),(2,0,40));assert_eq!(c.state.piles.exhaust.len(),1);
}

// Losing one Flight charge per card rather than per separate NORMAL hit fails.
#[test]
fn m6_flight_counts_hits_not_cards_and_resets_or_stuns() {
    let mut e=entry(EncounterId::ThreeByrds);e.setup.enemies.truncate(1);e.cards[0]=permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();let hp=c.state.enemies[0].hp;assert!((25..=31).contains(&hp));assert_eq!(amount(&c,0,"flight"),3);
    hit(&mut c,0,6);assert_eq!((c.state.enemies[0].hp,amount(&c,0,"flight")),(hp-3,2));
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();assert_eq!((c.state.enemies[0].hp,amount(&c,0,"flight")),(hp-7,0));assert_eq!(intent(&c),"byrd_stunned");
    for (player_hp,next,flying) in [(80,"byrd_headbutt",0),(77,"byrd_go_airborne",0),(77,"airborne",3)] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,player_hp);assert_eq!(amount(&c,0,"flight"),flying);
        if next=="airborne"{assert!(matches!(intent(&c),"byrd_peck"|"byrd_caw"|"byrd_swoop"));}else{assert_eq!(intent(&c),next)}
    }
}

#[test]
fn m6_flight_ignores_fully_blocked_zero_and_non_normal_damage_and_refills_at_monster_start() {
    let mut e=entry(EncounterId::ThreeByrds);e.setup.enemies.truncate(1);let mut c=Combat::enter(e).unwrap();let hp=c.state.enemies[0].hp;
    let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor.clone()),kind:EffectKind::GainBlock(3)}],None).unwrap();
    hit(&mut c,0,6);hit(&mut c,0,1);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"flight")),(hp,0,3));
    hit(&mut c,0,4);assert_eq!((c.state.enemies[0].hp,amount(&c,0,"flight")),(hp-2,2));
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(actor.clone()),kind:EffectKind::Damage(DamagePayload::Thorns(6))},Effect{source:Source::Actor(Actor::Player),target:Some(actor),kind:EffectKind::Damage(DamagePayload::HpLoss(2))}],None).unwrap();
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"flight")),(hp-10,2));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"flight"),3);
}

#[test]
fn m6_flight_reductions_wait_behind_all_earlier_hits_and_fault_keeps_accepted_damage() {
    for fault in [false,true] {
        let mut e=entry(EncounterId::ThreeByrds);e.setup.enemies.truncate(1);let mut c=Combat::enter(e).unwrap();let hp=c.state.enemies[0].hp;
        let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());
        c.state.pending.extend((0..4).map(|_|Effect{source:Source::Actor(Actor::Player),target:Some(actor.clone()),kind:EffectKind::Damage(DamagePayload::Fixed(2))}));
        for _ in 0..8{engine::dispatch_one(&mut c.state);}
        assert_eq!((c.state.enemies[0].hp,amount(&c,0,"flight")),(hp-4,3),"all four earlier NORMAL hits still receive Flight reduction");
        engine::dispatch_one(&mut c.state);assert_eq!(amount(&c,0,"flight"),2);
        if fault {
            c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_some());assert_eq!(amount(&c,0,"flight"),2);
            assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::ReduceFullPower{id:FullPowerId("flight"),amount:1});
            let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
        }else{engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(amount(&c,0,"flight"),0);assert_eq!(intent(&c),"byrd_stunned");}
    }
}

#[test]
fn m6_byrd_native_ai_uses_exact_bucket_order_fallbacks_and_no_grounded_draw() {
    use crate::content::EnemyId;
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"byrd".into()]).unwrap();
    let stream=|cursor|rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
    let memory=EnemyMemory::Byrd{flying:true};
    for (cursor,want) in [(0,"byrd_caw"),(1,"byrd_peck")] {
        let selected=enemies::select_move(EnemyId::Byrd,&[],&memory,&stream(cursor)).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream.word_index(),cursor+1);
    }
    // HMAC fixture cursors independently located for literal buckets0/49/50/69/70/99.
    for (cursor,want) in [(23,"byrd_peck"),(138,"byrd_peck"),(66,"byrd_swoop"),(257,"byrd_swoop"),(285,"byrd_caw"),(33,"byrd_caw")] {
        let selected=enemies::select_move(EnemyId::Byrd,&[FullIntentId::from_name("byrd_go_airborne")],&memory,&stream(cursor)).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream.word_index(),cursor+1);
    }
    for (cursor,history,want) in [(0,vec!["byrd_peck","byrd_peck"],"byrd_swoop"),(3,vec!["byrd_peck","byrd_peck"],"byrd_caw"),(2,vec!["byrd_swoop"],"byrd_caw"),(5,vec!["byrd_swoop"],"byrd_peck"),(4,vec!["byrd_caw"],"byrd_swoop"),(1,vec!["byrd_caw"],"byrd_peck")] {
        let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::select_move(EnemyId::Byrd,&history,&memory,&stream(cursor)).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream.word_index(),cursor+2);
    }
    let exhausted=stream(1_u128<<64);let selected=enemies::select_move(EnemyId::Byrd,&[FullIntentId::from_name("byrd_stunned")],&EnemyMemory::Byrd{flying:false},&exhausted).unwrap();assert_eq!(selected.intent.as_str(),"byrd_headbutt");assert_eq!(selected.stream,exhausted);
}

#[test]
fn m6_three_byrds_have_independent_native_setup_and_existing_profiles_stay_closed() {
    let e=entry(EncounterId::ThreeByrds);assert_eq!(e.setup.enemies.len(),3);
    for (i,actor) in e.setup.enemies.iter().enumerate(){assert!((25..=31).contains(&actor.hp));assert_eq!(actor.hp,actor.max_hp);assert!(matches!(actor.intent.as_str(),"byrd_peck"|"byrd_caw"));for other in &e.setup.enemies[..i]{assert_ne!(actor.instance_id,other.instance_id);assert_ne!(actor.ai_rng.id(),other.ai_rng.id());}}
    for (act,ascension) in [(1,0),(2,21)] {
        let mut rng=crate::run::RunRng::new([41;32],rng::Partition::Test,0);let before=rng.clone();
        let context=enemies::SetupContext{player:e.setup.player.clone(),player_powers:vec![],character:Character::Ironclad,cards:e.cards.clone(),opening_draw:1,encounter_visit:0};
        assert!(enemies::setup(EncounterId::ThreeByrds,act,ascension,&mut rng,context).is_err());assert_eq!(rng,before);
    }
}

// spire-bhl: a multi-hit attacker killed by the player's THORNS retaliation must stop
// its own remaining queued hits (the original damage action does nothing once its owner is
// dying; source-settled by static analysis), mirroring the corpse-target rule, not fault.
fn pecking_byrds(byrd_hp:i32)->(Combat,i32,i32){
    for seed in 1..=64u8 {
        let mut e=enemies::setup(EncounterId::ThreeByrds,2,0,&mut crate::run::RunRng::new([seed;32],rng::Partition::Test,0),enemies::SetupContext {
            player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
            player_powers:vec![],character:Character::Ironclad,
            cards:vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0,
        }).unwrap();
        if e.setup.enemies[0].intent.as_str()!="byrd_peck"||e.setup.enemies[1].intent.as_str()!="byrd_caw"{continue}
        e.setup.enemies.truncate(2);e.setup.enemies[0].hp=byrd_hp;
        let mut c=Combat::enter(e).unwrap();
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("thorns"),amount:3}}],None).unwrap();
        let peck=c.state.enemies[0].intent_damage.unwrap();let partner=c.state.enemies[1].hp;
        return (c,peck,partner)
    }
    panic!("no fixture seed gives a pecking lead Byrd beside a cawing partner")
}

#[test]
fn bhl_thorns_lethal_on_first_peck_cancels_the_dying_byrds_remaining_hits() {
    let (mut c,peck,partner)=pecking_byrds(3);let before=c.state.player.hp;
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.fault.is_none(),"the dying Byrd's queued pecks must not fault: {:?}",c.state.fault);
    assert_eq!(c.state.outcome,None,"the cawing partner keeps combat live");
    assert_eq!(c.state.enemies[0].hp,0,"THORNS 3 kills the 3-HP Byrd on its first peck");
    assert_eq!(c.state.player.hp,before-peck,"only the first of five pecks lands");
    assert_eq!(c.state.enemies[1].hp,partner,"the partner's Caw deals and receives no damage");
}

#[test]
fn bhl_nonlethal_thorns_keeps_all_five_pecks() {
    let (mut c,peck,_)=pecking_byrds(20);let before=c.state.player.hp;
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.fault.is_none());
    assert_eq!(c.state.enemies[0].hp,5,"five THORNS 3 retaliations against 20 HP");
    assert_eq!(c.state.player.hp,before-5*peck,"a surviving attacker resolves every queued hit");
}
