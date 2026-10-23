use spire_core::{content::{EncounterId,RelicId},run::*};
#[path="../examples/m5_support/mod.rs"] mod support;
use support::{Caller,Scenario};

#[test]
fn standard_resource_public_paths_defeat_all_boss_and_elite_alternatives() {
    let mut elites=vec![];
    for scenario in [Scenario::Guardian,Scenario::Hexaghost,Scenario::Slime] {
        let mut r=support::run(7,scenario);let d=r.decision().unwrap();assert_eq!((d.player.hp,d.player.max_hp,d.player.gold,d.objects.len()),(80,80,99,10));assert_eq!(d.player.relics,vec![RelicId::BurningBlood]);assert_eq!(d.player.potions,vec![None;3]);assert_eq!(d.player.keys,[false;3]);
        let mut caller=Caller::new(4096);support::rollout(&mut r,&mut caller,scenario).unwrap();let d=r.decision().unwrap();assert_eq!((d.act,d.floor),(2,17));assert_eq!(d.room.unwrap().kind,RoomViewKind::Act2Ready);assert!(d.player.hp>0);assert_eq!(d.terminal,None);assert_eq!(d.reward_total,if scenario==Scenario::Slime{32}else{17});
        for (_,t,_) in &caller.trace{for e in &t.committed_events{if let CommittedEvent::CombatEntered{encounter,..}=e{if matches!(encounter,EncounterId::GremlinNob|EncounterId::Lagavulin|EncounterId::ThreeSentries){elites.push(*encounter);}}}}
        assert!(caller.elites>0);assert!(caller.accepted()<=4096);
        let mut replay=support::run(7,scenario);for (action,t,decision) in &caller.trace{assert_eq!(&replay.step(action).unwrap(),t);assert_eq!(&replay.decision().unwrap(),decision);}
    }
    for id in [EncounterId::GremlinNob,EncounterId::Lagavulin,EncounterId::ThreeSentries]{assert!(elites.contains(&id));}
}

#[test]
fn second_public_seed_and_exact_action_reward_replay_reach_nonterminal_boundary() {
    let mut a=support::run(11,Scenario::Guardian);let mut b=support::run(11,Scenario::Guardian);let mut caller=Caller::new(4096);support::rollout(&mut a,&mut caller,Scenario::Guardian).unwrap();
    for (action,transition,decision) in &caller.trace{assert_eq!(&b.step(action).unwrap(),transition);assert_eq!(&b.decision().unwrap(),decision);}
    let d=a.decision().unwrap();assert_eq!(d.reward_total,17);assert_eq!(d.act,2);assert!(d.candidates.is_empty());assert_eq!(d.terminal,None);assert_eq!(a.public_map(),b.public_map());
    let before=b.decision().unwrap();assert!(b.step(&caller.trace[0].0).is_err());assert_eq!(b.decision().unwrap(),before);
}

#[test]
fn public_native_death_and_zero_one_budgets_never_fabricate_completion() {
    let mut r=support::run(7,Scenario::Death);let before=r.decision().unwrap();let mut zero=Caller::new(0);assert!(support::rollout(&mut r,&mut zero,Scenario::Death).is_err());assert_eq!(r.decision().unwrap(),before);
    let mut one=Caller::new(1);assert!(support::rollout(&mut r,&mut one,Scenario::Death).is_err());assert_eq!(one.accepted(),1);assert_eq!(r.decision().unwrap().terminal,None);
    let mut r=support::run(7,Scenario::Death);let mut caller=Caller::new(4096);support::rollout(&mut r,&mut caller,Scenario::Death).unwrap();let d=r.decision().unwrap();assert_eq!((d.player.hp,d.floor,d.reward_total),(0,1,0));assert_eq!(d.terminal,Some(RunTerminal::Death));
}

#[test]
fn public_views_keep_future_offers_private_and_retained_rewards_detached() {
    let mut r=support::run(7,Scenario::Guardian);let mut caller=Caller::new(4096);support::rollout(&mut r,&mut caller,Scenario::Guardian).unwrap();
    let mut retained=None;
    for (_,_,d) in &caller.trace {
        if let Some(room)=&d.room {
            if matches!(room.kind,RoomViewKind::Map|RoomViewKind::Act2Ready){assert!(room.choices.is_empty());assert!(d.offers.is_empty());}
            if room.kind==RoomViewKind::BossTreasure{assert!(room.choices.iter().all(|c|matches!(c.kind,RoomChoiceKind::OpenChest|RoomChoiceKind::Proceed)));}
            if retained.is_none()&&room.kind==RoomViewKind::Rewards{retained=room.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Gold(_))&&c.status==RoomChoiceStatus::Offered).cloned();}
        }
    }
    let retained=retained.unwrap();assert_eq!(retained.status,RoomChoiceStatus::Offered);
    assert!(caller.trace.iter().any(|(_,_,d)|d.room.as_ref().is_some_and(|room|room.kind==RoomViewKind::Rewards&&room.choices.iter().any(|c|c.id==retained.id&&c.kind==retained.kind&&c.status==RoomChoiceStatus::Claimed))));
    let map=r.public_map().unwrap();assert!(map.nodes.iter().filter(|n|!n.visited).all(|n|n.resolved_room.is_none()));
}
