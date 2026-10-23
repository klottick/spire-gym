use super::super::*;
use crate::content::{CardId,EncounterId};
use super::m5_elites::{entry,amount,hit};

fn guardian()->Combat{Combat::enter(entry(EncounterId::Guardian)).unwrap()}
fn damage(c:&Combat,n:i32)->Effect{Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(n))}}
fn skip_damage_hook(c:&mut Combat){assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::Hook(Hook::Damage));engine::dispatch_one(&mut c.state);}

#[test]
fn guardian_hp_loss_latches_before_bottom_change() {
    let mut c=guardian();assert_eq!(c.state.enemies[0].hp,240);assert_eq!(amount(&c,0,"mode_shift"),30);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::GainBlock(3)}],None).unwrap();hit(&mut c,0,3);assert_eq!(amount(&c,0,"mode_shift"),30);
    c.state.pending.extend([damage(&c,29),damage(&c,1),damage(&c,7)]);
    engine::dispatch_one(&mut c.state);assert_eq!(c.state.enemies[0].hp,211);assert_eq!(amount(&c,0,"mode_shift"),1);
    skip_damage_hook(&mut c);engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.enemies[0].block,0);assert_eq!(c.state.enemies[0].intent_id.as_str(),"charging_up");
    assert_eq!(c.state.enemies[0].full_memory,Some(EnemyMemory::Guardian{quota:30,loss:0,open:true,latched:true}));
    skip_damage_hook(&mut c);engine::dispatch_one(&mut c.state);assert_eq!(c.state.enemies[0].hp,203);assert_eq!(c.state.enemies[0].block,0);
    engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.enemies[0].block,20);assert_eq!(c.state.enemies[0].intent_id.as_str(),"defensive_mode");
}

#[test]
fn guardian_threshold_does_not_roll_excess() {
    let mut c=guardian();
    for (loss,quota) in [(45,40),(40,50),(50,60)] {
        hit(&mut c,0,loss);assert_eq!(c.state.enemies[0].intent_id.as_str(),"defensive_mode");
        for _ in 0..3{c.step_full(&CombatAction::EndTurn).unwrap();}
        assert_eq!(amount(&c,0,"mode_shift"),quota);assert_eq!(c.state.enemies[0].block,0);assert_eq!(c.state.enemies[0].intent_id.as_str(),"whirlwind");
    }
    hit(&mut c,0,59);assert_eq!(amount(&c,0,"mode_shift"),1);assert_eq!(c.state.enemies[0].intent_id.as_str(),"whirlwind");
}

#[test]
fn guardian_sharp_hide_per_attack_not_hit() {
    let mut e=entry(EncounterId::Guardian);e.cards[0].id=CardId::TwinStrike;
    let mut c=Combat::enter(e).unwrap();hit(&mut c,0,30);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"sharp_hide"),3);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(1)},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}},Effect{source:Source::None,target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}}],None).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.player.hp,78,"one THORNS3 against Block1; no Weak/Vulnerable multiplier");assert_eq!(c.state.enemies[0].hp,200,"two separate hit5 attacks");
}

#[test]
fn guardian_twin_slam_queue_order() {
    let mut c=guardian();hit(&mut c,0,30);
    for _ in 0..2{c.step_full(&CombatAction::EndTurn).unwrap();}
    assert_eq!(c.state.enemies[0].intent_id.as_str(),"twin_slam");
    let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});
    engine::dispatch_one(&mut c.state);
    // Offensive change must be the first child; its reset/Block loss is bottom work.
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::GuardianChange(enemies::GuardianEffect::Offensive));
    engine::dispatch_one(&mut c.state);assert_eq!(amount(&c,0,"mode_shift"),0);
    assert!(matches!(c.state.pending.front().unwrap().kind,EffectKind::Damage(DamagePayload::FullEnemyMove(_))));
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);
    assert_eq!(c.state.player.hp,55);assert_eq!(amount(&c,0,"sharp_hide"),0);assert_eq!(amount(&c,0,"mode_shift"),40);
}

#[test]
fn guardian_offensive_cycle_uses_shared_damage_and_ordered_debuffs() {
    let mut e=entry(EncounterId::Guardian);e.setup.player.hp=200;e.setup.player.max_hp=200;
    let mut c=Combat::enter(e).unwrap();
    for (hp,next,block) in [(200,"fierce_bash",9),(168,"vent_steam",0),(168,"whirlwind",0),(140,"charging_up",0)] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);assert_eq!(c.state.enemies[0].intent_id.as_str(),next);assert_eq!(c.state.enemies[0].block,block);
    }
    // Steam's Vulnerable2 amplifies four separate hit5 actions to hit7.
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="weak"&&p.state.amount==1));
}

#[test]
fn guardian_transition_fault_retains_latch_owner_and_cannot_duplicate_block() {
    let mut c=guardian();c.state.resolved_effects=99_997;c.state.pending.push_back(damage(&c,30));
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert_eq!(c.state.enemies[0].hp,210);assert_eq!(c.state.enemies[0].block,0);assert!(c.state.current.is_some());
    let current=c.state.current.as_ref().unwrap();assert_eq!(current.kind,EffectKind::GuardianChange(enemies::GuardianEffect::Defensive));
    assert_eq!(current.source,Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone())));
    assert_eq!(c.state.enemies[0].full_memory,Some(EnemyMemory::Guardian{quota:30,loss:0,open:true,latched:true}));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

fn hexaghost_inferno()->Combat {
    let mut e=entry(EncounterId::Hexaghost);e.setup.player.hp=999;e.setup.player.max_hp=999;
    let mut c=Combat::enter(e).unwrap();assert_eq!(c.state.enemies[0].hp,250);
    for _ in 0..8{c.step_full(&CombatAction::EndTurn).unwrap();}
    assert_eq!(c.state.enemies[0].intent_id.as_str(),"inferno");
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::RemoveFullPower(FullPowerId("strength"))}],None).unwrap();c
}
fn queue_enemy_move(c:&mut Combat){let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});engine::dispatch_one(&mut c.state);}
fn create_burn(c:&mut Combat,destination:CardDestination)->CardInstanceId{
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{id:CardId::Burn,destination,upgrades:0}}],None).unwrap();
    CardInstanceId(c.state.card_ids_issued.iter().map(|h|h.as_str().parse::<u64>().unwrap()).max().unwrap())
}
fn burn_upgrade(c:&Combat,id:CardInstanceId)->u32{c.state.piles.all(c.state.in_play.as_ref()).find_map(|c|c.full().filter(|c|c.instance==id).map(|c|c.upgrades)).unwrap()}
fn six_inferno_hits(c:&mut Combat){for _ in 0..6{assert!(matches!(c.state.pending.front().unwrap().kind,EffectKind::Damage(DamagePayload::FullEnemyMove(_))));engine::dispatch_one(&mut c.state);skip_damage_hook(c);}}

#[test]
fn hexaghost_divider_captures_once() {
    for (hp,base,want) in [(11,1,62),(12,2,56),(23,2,56),(24,3,50)] {
        let mut e=entry(EncounterId::Hexaghost);e.setup.player.hp=hp;
        let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);
        let divider=FullIntentId::from_name("divider");assert_eq!(c.state.enemies[0].full_damage_entries[&divider].base,base);
        let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Heal(80)},Effect{source:Source::None,target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2}},Effect{source:Source::None,target:Some(actor),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}}],None).unwrap();
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,want);assert_eq!(c.state.enemies[0].full_damage_entries[&divider].base,base);
    }
}

#[test]
fn hexaghost_inferno_six_hits_before_burn_mutation() {
    let mut c=hexaghost_inferno();let burn=create_burn(&mut c,CardDestination::Discard);let hp=c.state.player.hp;
    queue_enemy_move(&mut c);six_inferno_hits(&mut c);assert_eq!(c.state.player.hp,hp-12);assert_eq!(burn_upgrade(&c,burn),0);
    engine::dispatch_one(&mut c.state);assert_eq!(burn_upgrade(&c,burn),1);
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::HexaghostBurn(enemies::HexaghostEffect::InsertBurns),"separate later insertion gate");
    let issued=c.state.card_ids_issued.len();engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.card_ids_issued.len(),issued+3);
}

#[test]
fn hexaghost_burn_membership_and_future_sear() {
    let mut c=hexaghost_inferno();let held=create_burn(&mut c,CardDestination::Hand);let drawn=create_burn(&mut c,CardDestination::Draw);let discarded=create_burn(&mut c,CardDestination::Discard);
    let exhausted=create_burn(&mut c,CardDestination::Hand);let playing=create_burn(&mut c,CardDestination::Hand);
    let in_play=c.state.piles.hand.pop().unwrap();assert_eq!(in_play.full().unwrap().instance,playing);
    let exhaust=c.state.piles.hand.pop().unwrap();assert_eq!(exhaust.full().unwrap().instance,exhausted);piles::exhaust_owned_card(&mut c.state,exhaust).unwrap();engine::resolve(&mut c.state);
    queue_enemy_move(&mut c);c.state.in_play=Some(in_play);c.state.phase=Phase::PlayerAction;
    c.state.pending.push_back(Effect{source:Source::Card(card_state::handle(playing)),target:None,kind:EffectKind::FinishCard});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    engine::resolve(&mut c.state);assert!(c.state.fault.is_none());
    for (id,want) in [(held,0),(drawn,1),(discarded,1),(exhausted,0),(playing,0)]{assert_eq!(burn_upgrade(&c,id),want);}
    let independent=create_burn(&mut c,CardDestination::Discard);assert_eq!(burn_upgrade(&c,independent),0,"future global Burns stay base");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(burn_upgrade(&c,CardInstanceId(independent.0+1)),1,"future Sear specifically creates Burn+");
}

#[test]
fn hexaghost_burn_gates_retain_distinct_fault_states_without_duplicate_ids() {
    for stage in 0..3 {
        let mut c=hexaghost_inferno();let burn=create_burn(&mut c,CardDestination::Discard);let original=c.state.card_ids_issued.len();
        queue_enemy_move(&mut c);six_inferno_hits(&mut c);
        if stage>=1{engine::dispatch_one(&mut c.state);}
        if stage==2{for _ in 0..4{engine::dispatch_one(&mut c.state);}}
        c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_some());
        assert_eq!(c.state.current.as_ref().unwrap().kind,match stage{0=>EffectKind::HexaghostBurn(enemies::HexaghostEffect::UpgradeBurns),1=>EffectKind::HexaghostBurn(enemies::HexaghostEffect::InsertBurns),_=>EffectKind::RollMove});
        assert!(matches!(c.state.enemies[0].full_memory,Some(EnemyMemory::Hexaghost{sear_upgraded:true,..})),"future-Sear flag commits at Inferno production, before all later gates");
        assert_eq!(burn_upgrade(&c,burn),if stage==0{0}else{1});assert_eq!(c.state.card_ids_issued.len(),original+if stage==2{3}else{0});
        let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
    }
}

#[test]
fn hexaghost_captured_divider_validates_with_shared_large_damage_output() {
    let mut e=entry(EncounterId::Hexaghost);e.setup.player.hp=999;e.setup.player.max_hp=999;
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:999}},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}}],None).unwrap();
    assert_eq!(c.state.enemies[0].intent_damage,Some(1624));
    assert!(engine::validate_runtime(&c.state).is_ok(),"captured84 + Strength999, Vulnerable gives floor1624.5, within native shared output surface");
    let mut bad=c.snapshot();bad.state.enemies[0].full_damage_entries.get_mut(&FullIntentId::from_name("divider")).unwrap().base=1;
    assert!(engine::validate_runtime(&bad.state).is_err(),"a captured base cannot silently return to species default");
}

fn m5_boss_boundary(seen:Vec<&'static str>)->crate::run::Run {
    use crate::{content::Character,run::*};
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;profile.seen_bosses[0]=seen;
    let mut r=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new([7;32],rng::Partition::Test,0),RoomScenario::M5Act1Captured).unwrap();
    let id=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::NeowMaxHp{..})).unwrap().id;r.step(&GameAction::Choose(id)).unwrap();
    let state=r.state_mut_for_test();let map=state.map.as_mut().unwrap();let rest=map.nodes.iter_mut().find(|n|n.y==14).unwrap();rest.visited=true;map.current=Some(rest.id);state.floor=15;
    r.step(&GameAction::Move(NodeId(108))).unwrap();r
}

#[test]
fn m5_all_three_bosses_enter_through_context_owned_native_bridge() {
    for (seen,name,hp) in [(vec![],"guardian",240),(vec!["Guardian"],"hexaghost",250),(vec!["Guardian","Hexaghost"],"slime_boss",140)] {
        let r=m5_boss_boundary(seen);let c=r.state().combat.as_ref().unwrap().public_full().unwrap();
        assert_eq!(c.enemies[0].enemy_id.as_str(),name);assert_eq!(c.enemies[0].hp,hp);assert_eq!(c.player.hp,88);assert_eq!(r.state().combat_progress.settled,0);
    }
}

#[test]
fn m5_slime_boss_requires_actual_descendant_deaths_before_loot_and_settlement() {
    use crate::run::*;
    let mut r=m5_boss_boundary(vec!["Guardian","Hexaghost"]);
    r.state_mut_for_test().combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:80}}],None).unwrap();
    let strike=r.state().combat.as_ref().unwrap().public_full().unwrap().hand.into_iter().find(|c|c.id==CardId::Strike).unwrap().instance.0;
    r.choose_combat(CombatAction::PlayCard{instance:strike,target:Some(TargetId(1))}).unwrap();r.choose_combat(CombatAction::EndTurn).unwrap();
    let c=r.state().combat.as_ref().unwrap();assert_eq!(c.state.enemies.len(),3);assert_eq!(c.state.enemies[0].life,ActorLife::Replaced);
    assert_eq!((c.state.enemies[1].hp,c.state.enemies[2].hp),(54,54));assert_eq!(r.state().combat_progress.settled,0);
    for target in [TargetId(2),TargetId(3)] {
        let mut killed=false;
        for _ in 0..4 {
            let d=r.decision().unwrap();let action=d.candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(action@CombatAction::PlayCard{target:Some(t),..}) if t==target=>Some(action),_=>None});
            if let Some(action)=action{r.choose_combat(action).unwrap();killed=true;break}else{r.choose_combat(CombatAction::EndTurn).unwrap();}
        }assert!(killed);
        if target==TargetId(2){assert_eq!(r.state().combat_progress.settled,0);assert!(r.state().combat.is_some());}
    }
    assert_eq!(r.state().combat_progress,CombatProgress{entered:1,settled:1});assert!(r.state().combat.is_none());assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Rewards);
}
