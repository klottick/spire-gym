//! Native late-entry lifecycle diagnostics, not normal-start/full-act acceptance.
use crate::{combat::{Actor,CombatAction,DamagePayload,Effect,EffectKind,FullPowerId,Source,TargetId},content::{CardId,EncounterId,RelicId},run::*};
use super::rooms::choose;

pub(super) fn source(boss:EncounterId)->(Run,NodeId){
    let mut r=super::m7_rewards::chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));
    r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M9IroncladAct3;
    r.state.seen_bosses[2]=match boss{EncounterId::AwakenedOne=>vec!["TimeEater","DonuAndDeca"],EncounterId::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],EncounterId::DonuAndDeca=>vec!["AwakenedOne","TimeEater"],_=>panic!("not Act3 boss")};
    choose(&mut r,RoomChoiceKind::Proceed);
    r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);
    let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.kind==MapNodeKind::Boss).unwrap().id;
    super::m9_rooms::before(&mut r,node);r.state.floor=49;(r,node)
}
pub(super) fn hit(r:&mut Run,slot:usize,amount:i32){
    let combat=r.state.combat.as_mut().unwrap();let actor=Actor::Enemy(combat.public_full().unwrap().enemies[slot].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(actor),kind:EffectKind::Damage(DamagePayload::Fixed(amount))}],None).unwrap();
}

#[test]
fn m9_selected_boss_enters_real_floor50_source(){
    for (boss,slot,hp,name) in [(EncounterId::AwakenedOne,2,300,"AwakenedOne"),(EncounterId::TimeEater,0,456,"TimeEater"),(EncounterId::DonuAndDeca,0,250,"DonuAndDeca")]{
        let(mut r,node)=source(boss);assert_eq!(r.public_map().unwrap().boss,Some(boss));assert!(!r.state.seen_bosses[2].contains(&name));
        let cards=r.state.deck.clone();let entered=r.state.combat_progress.entered;let settled=r.state.combat_progress.settled;
        r.step(&GameAction::Move(node)).unwrap();
        assert_eq!((r.state.act,r.state.floor,r.state.active_combat.as_ref().unwrap().encounter),(3,50,boss));
        assert_eq!(r.state.active_combat.as_ref().unwrap().origin,CombatOrigin::MapRoom{node,kind:MapNodeKind::Boss});
        let c=r.decision().unwrap().combat.unwrap();assert_eq!((c.enemies[slot].hp,c.player.max_energy),(hp,4));
        assert_eq!((r.state.combat_progress.entered,r.state.combat_progress.settled),(entered+1,settled));assert_eq!(r.state.deck,cards);
        assert_eq!(r.state.seen_bosses[2].iter().filter(|n|**n==name).count(),1);
        assert!(r.state.relics.is_empty());let items=r.state.combat.as_ref().unwrap().item_context().unwrap();assert!(items.battle.boss&&!items.battle.elite_trigger&&items.battle.burning.is_none());
        assert!(items.relics().contains(&RelicId::SlaversCollar)&&items.relics().contains(&RelicId::BlackBlood));
        let frozen=r.clone();assert!(r.step(&GameAction::Move(node)).is_err());assert!(r==frozen);
    }
}

#[test]
fn m9_boss_settlement_waits_for_actual_lifecycle(){
    let(mut awakened,node)=source(EncounterId::AwakenedOne);awakened.step(&GameAction::Move(node)).unwrap();let settled=awakened.state.combat_progress.settled;
    hit(&mut awakened,2,300);assert!(awakened.decision().unwrap().combat.unwrap().enemies[2].half_dead);
    assert_eq!(awakened.state.combat_progress.settled,settled);assert!(awakened.state.active_combat.is_some());
    awakened.choose_combat(CombatAction::EndTurn).unwrap();assert_eq!(awakened.decision().unwrap().combat.unwrap().enemies[2].hp,300);assert_eq!(awakened.state.combat_progress.settled,settled);
    hit(&mut awakened,2,300);let deck=awakened.state.deck.clone();awakened.settle_combat().unwrap();
    assert_eq!(awakened.state.deck,deck);assert_eq!(awakened.state.combat_progress.settled,settled+1);
    assert_eq!(awakened.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);
    let frozen=awakened.clone();assert!(awakened.choose_combat(CombatAction::EndTurn).is_err());assert!(awakened==frozen);

    let(mut pair,node)=source(EncounterId::DonuAndDeca);pair.step(&GameAction::Move(node)).unwrap();let settled=pair.state.combat_progress.settled;
    hit(&mut pair,0,250);pair.choose_combat(CombatAction::EndTurn).unwrap();assert_eq!(pair.state.combat_progress.settled,settled);
    assert!(pair.state.combat.is_some());hit(&mut pair,1,250);pair.settle_combat().unwrap();
    assert_eq!(pair.state.combat_progress.settled,settled+1);assert_eq!(pair.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().combat_visits,1);
    assert_eq!(pair.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);
    assert!(pair.state.combat.is_none()&&pair.state.active_combat.is_none());assert!(pair.state.relics.contains(&RelicId::SlaversCollar));
}

#[test]
fn m9_time_warp_keeps_twelfth_card_payload_in_real_run(){
    let(mut r,node)=source(EncounterId::TimeEater);r.state.deck=vec![CardInstance{instance:1,id:CardId::TwinStrike,upgrades:0}];r.state.next_instance=2;
    r.step(&GameAction::Move(node)).unwrap();let settled=r.state.combat_progress.settled;
    let combat=r.state.combat.as_mut().unwrap();let actor=Actor::Enemy(combat.public_full().unwrap().enemies[0].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::ApplyFullPower{id:FullPowerId("time_warp"),amount:11}}],None).unwrap();
    r.choose_combat(CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    let c=r.decision().unwrap().combat.unwrap();assert_eq!((c.enemies[0].hp,c.turn),(446,2));
    assert_eq!(c.enemies[0].powers.iter().find(|p|p.id.0=="time_warp").unwrap().amount,0);
    assert_eq!(r.state.combat_progress.settled,settled);assert!(r.state.active_combat.is_some());
}

#[test]
fn m9_boss_entry_fault_keeps_accepted_source(){
    let(mut r,node)=source(EncounterId::TimeEater);r.state.relics.push(RelicId::PureWater);r.state.deck=vec![CardInstance{instance:u64::MAX,id:CardId::Strike,upgrades:0}];
    let prior=r.state.reward_ledger.total;let err=r.step(&GameAction::Move(node)).unwrap_err();
    assert!(matches!(err,RunError::Faulted(_)));assert_eq!((r.state.floor,r.state.reward_ledger.total,r.state.terminal),(50,prior+1,None));assert!(r.state.combat.is_some()&&r.state.active_combat.is_some());assert!(r.state.relics.is_empty());
    let frozen=r.clone();assert_eq!(r.step(&GameAction::Move(node)),Err(err));assert!(r==frozen);

    // Qualified Victory publication/cursor faults are covered in m9_end.
}

#[test]
fn m9_boss_death_has_one_settlement_and_no_victory_source(){
    for boss in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca]{
        let(mut r,node)=source(boss);r.state.hp=1;r.step(&GameAction::Move(node)).unwrap();let settled=r.state.combat_progress.settled;
        for _ in 0..8{if r.state.terminal.is_some(){break}r.choose_combat(CombatAction::EndTurn).unwrap();}
        assert_eq!((r.state.hp,r.state.terminal),(0,Some(RunTerminal::Death)));assert_eq!(r.state.combat_progress.settled,settled+1);
        assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().combat_visits,1);assert!(r.decision().unwrap().candidates.is_empty());
        assert!(r.state.room_loop.as_ref().unwrap().active_rewards.is_none());assert_ne!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::BossTreasure);
    }
}

#[test]
fn m9_boss_preflight_rejects_inconsistent_selected_icon_without_movement(){
    let(mut r,node)=source(EncounterId::TimeEater);r.state.map.as_mut().unwrap().boss_icon=Some(EncounterId::AwakenedOne);
    let frozen=r.clone();assert!(r.step(&GameAction::Move(node)).is_err());assert!(r==frozen);
}

#[test]
fn m9_boss_result_cannot_publish_inherited_act12_loot(){
    let(mut r,node)=source(EncounterId::DonuAndDeca);r.step(&GameAction::Move(node)).unwrap();
    let settled=r.state.combat_progress.settled;let gold=r.state.gold;hit(&mut r,0,250);hit(&mut r,1,250);
    r.settle_combat().unwrap();
    assert_eq!(r.state.combat_progress.settled,settled+1);assert!(r.state.active_combat.is_none()&&r.state.combat.is_none());
    assert_eq!(r.state.gold,gold);let screen=r.decision().unwrap().room.unwrap();
    assert_eq!(screen.kind,RoomViewKind::Act3BossComplete);assert_eq!(screen.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);
    assert!(r.state.room_loop.as_ref().unwrap().active_rewards.is_none());
    let frozen=r.clone();assert!(r.choose_combat(CombatAction::EndTurn).is_err());assert!(r==frozen);
}
