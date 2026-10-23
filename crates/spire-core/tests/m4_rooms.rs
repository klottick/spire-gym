use spire_core::{content::{CardId,RelicId},run::*};
#[path="../examples/m4_support/mod.rs"] mod support;
use support::{Caller,Scenario};

#[test]
fn normal_resource_route_completes_three_real_fights_and_nine_rooms() {
    let mut r=support::run(7);let start=r.decision().unwrap();assert_eq!((start.player.hp,start.player.gold,start.objects.len()),(80,99,10));
    let mut caller=Caller::new(512);support::rollout(&mut r,&mut caller,Scenario::Route).unwrap();
    let d=r.decision().unwrap();assert_eq!(d.room.unwrap().kind,RoomViewKind::CheckpointComplete);
    assert_eq!((d.floor,d.player.max_hp,d.objects.len(),d.reward_total),(9,95,13,9));
    assert_eq!(d.combat_progress,CombatProgress{entered:3,settled:3});assert_eq!(d.terminal,None);
    assert!(d.player.relics.contains(&RelicId::Strawberry));assert!(d.objects.iter().any(|c|c.id==CardId::Bash&&c.upgrades==1));assert!(caller.accepted()<=512);
}
#[test]
fn keys_are_real_alternatives_and_lament_expires_without_resetting_on_potion_use() {
    let mut r=support::run(7);let mut caller=Caller::new(512);support::rollout(&mut r,&mut caller,Scenario::Keys).unwrap();
    let d=r.decision().unwrap();assert_eq!(d.player.keys,[true,true,false]);assert_eq!(d.reward_total,19);assert_eq!(d.player.max_hp,80);assert!(!d.player.relics.contains(&RelicId::Strawberry));
    assert_eq!(d.player.relic_counters.iter().find(|c|c.relic==RelicId::NeowsLament).unwrap().value,-2);
}
#[test]
fn public_actions_and_reward_deltas_replay_exactly() {
    let mut a=support::run(7);let mut b=support::run(7);let mut caller=Caller::new(512);
    support::rollout(&mut a,&mut caller,Scenario::Purge).unwrap();
    for (action,transition,decision) in &caller.trace {
        assert_eq!(&b.step(action).unwrap(),transition);assert_eq!(&b.decision().unwrap(),decision);
    }
    assert_eq!(a.decision().unwrap(),b.decision().unwrap());
}
#[test]
fn death_is_native_and_budget_exhaustion_never_fabricates_a_terminal() {
    let mut r=support::run(7);let before=r.decision().unwrap();let mut zero=Caller::new(0);
    assert!(support::rollout(&mut r,&mut zero,Scenario::Death).is_err());assert_eq!(r.decision().unwrap(),before);
    let mut one=Caller::new(1);assert!(support::rollout(&mut r,&mut one,Scenario::Death).is_err());assert_eq!(one.accepted(),1);assert_eq!(r.decision().unwrap().terminal,None);
    let mut r=support::run(7);let mut caller=Caller::new(512);support::rollout(&mut r,&mut caller,Scenario::Death).unwrap();
    let d=r.decision().unwrap();assert_eq!((d.player.hp,d.floor,d.reward_total),(0,1,0));assert_eq!(d.terminal,Some(RunTerminal::Death));
}
