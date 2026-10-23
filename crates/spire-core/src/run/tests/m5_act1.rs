use crate::{combat::rng::Partition,content::{Character,EncounterId,RelicId},run::*};
use super::rooms::choose;

pub(super) fn run_with_seen(seen:Vec<&'static str>)->Run {
    let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=seen;
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M5Act1Captured).unwrap()
}
pub(super) fn run()->Run{run_with_seen(vec!["Guardian","Hexaghost","SlimeBoss"])}

#[test]
fn m5_standard_creation_and_full_map() {
    let mut r=run();let d=r.decision().unwrap();
    assert_eq!(d.content_profile,Some(RoomScenario::M5Act1Captured));
    assert_eq!((d.player.hp,d.player.max_hp,d.player.gold,d.objects.len()),(80,80,99,10));
    assert_eq!(d.player.relics,vec![RelicId::BurningBlood]);assert_eq!(d.player.potions,vec![None;3]);
    choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});
    let map=r.public_map().unwrap();assert_eq!(map.nodes.iter().filter(|n|n.y==0).count(),6);
    assert_eq!(map.nodes.iter().filter(|n|n.kind==MapNodeKind::Boss).map(|n|n.floor).collect::<Vec<_>>(),vec![16]);
    assert!(map.nodes.iter().filter(|n|n.y==8).all(|n|n.kind==MapNodeKind::Treasure));
    assert!(map.nodes.iter().filter(|n|n.y==14).all(|n|n.kind==MapNodeKind::Rest));
    assert_eq!(r.decision().unwrap().candidates.iter().filter(|c|matches!(c.action,GameAction::Move(_))).count(),6);
}

#[test]
fn m5_boss_seen_selection_and_prebattle_discovery() {
    for (seen,boss) in [(vec![],EncounterId::Guardian),(vec!["Guardian"],EncounterId::Hexaghost),(vec!["Guardian","Hexaghost"],EncounterId::SlimeBoss)] {
        let mut r=run_with_seen(seen.clone());let rng=r.state.rng.clone();choose(&mut r,RoomChoiceKind::NeowLament);
        assert_eq!(r.public_map().unwrap().boss,Some(boss));assert_eq!(r.state.seen_bosses[0],seen);
        let mut before=rng;let mut after=r.state.rng.clone();
        assert_eq!(before.draw(RunRngScope::Act1BossOrder),after.draw(RunRngScope::Act1BossOrder),"unseen branch consumes no boss-order draw");
        let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;
        r.step(&action).unwrap();assert_eq!(r.state.seen_bosses[0],seen,"ordinary fight is not boss discovery");
    }
}

#[test]
fn m5_invalid_move_is_atomic() {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});
    let before=r.clone();assert!(r.step(&GameAction::Move(NodeId(108))).is_err());assert!(r==before);
    let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;
    let t=r.step(&action).unwrap();assert_eq!(t.reward,1);assert_eq!(r.decision().unwrap().floor,1);
}

#[test]
fn m5_first_three_real_fights_reuse_lament_and_settlement() {
    use crate::combat::CombatAction;
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    for _ in 0..128 {
        if r.state.combat_progress.settled==3{break}
        let d=r.decision().unwrap();let action=if d.combat.is_some(){
            d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{target:Some(_),..}))).map(|c|c.action.clone()).unwrap_or(GameAction::Combat(CombatAction::EndTurn))
        }else{match d.room.as_ref().unwrap().kind{
            RoomViewKind::Map=>d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone(),
            RoomViewKind::Rewards=>{let id=d.room.as_ref().unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;GameAction::Choose(id)},
            _=>{let id=d.room.as_ref().unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Leave).unwrap().id;GameAction::Choose(id)},
        }};r.step(&action).unwrap();
    }
    assert_eq!(r.state.combat_progress,CombatProgress{entered:3,settled:3});assert_eq!(r.state.deck.len(),10);assert_eq!(r.state.gold,99);
    assert_eq!(r.state.relic_counters.get(&RelicId::NeowsLament),Some(&-2));
    assert_eq!(r.state.map.as_ref().unwrap().regular_encounter_history,vec![EncounterId::Cultist,EncounterId::JawWorm,EncounterId::TwoLouse]);
}

#[test]
fn m5_actual_boss_prebattle_commits_discovery_once() {
    let mut r=run_with_seen(vec!["Guardian","Hexaghost"]);choose(&mut r,RoomChoiceKind::NeowLament);
    // Valid native final-rest→boss boundary fixture; full from-start paths are Task7.
    let map=r.state.map.as_mut().unwrap();let rest=map.nodes.iter_mut().find(|n|n.y==14).unwrap();rest.visited=true;let id=rest.id;map.current=Some(id);r.state.floor=15;
    r.step(&GameAction::Move(NodeId(108))).unwrap();
    assert_eq!(r.state.seen_bosses[0],vec!["Guardian","Hexaghost","SlimeBoss"]);
    let before=r.clone();assert!(r.step(&GameAction::Move(NodeId(108))).is_err());assert!(r==before);
}

#[test]
fn m5_marked_elite_rolls_one_formation_buff_before_lament_and_never_rerolls_fault() {
    use crate::{combat::CombatAction,content::CardId};
    for (marked,fault) in [(true,false),(false,false),(true,true)] {
        let mut r=run_with_seen(vec!["Guardian","Hexaghost"]);choose(&mut r,RoomChoiceKind::NeowLament);
        let map=r.state.map.as_mut().unwrap();let target=map.nodes.iter().find(|n|n.kind==MapNodeKind::Elite).unwrap().id;
        let previous=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();previous.visited=true;let from=previous.id;r.state.floor=previous.floor;map.current=Some(from);
        for node in &mut map.nodes{node.burning=marked&&node.id==target;}
        if fault {r.state.relics.push(RelicId::PureWater);r.state.deck=vec![crate::run::CardInstance{instance:u64::MAX,id:CardId::Strike,upgrades:0}];}
        let mut expected=r.state.rng.clone();let semantic=[1,target.0,0];
        let buff=if marked{Some(expected.bounded_scoped(RunRngScope::BurningEliteBuff,&semantic,4))}else{None};
        let result=r.step(&GameAction::Move(target));assert_eq!(result.is_err(),fault);
        let mut actual=r.state.rng.clone();assert_eq!(actual.draw_scoped(RunRngScope::BurningEliteBuff,&semantic),expected.draw_scoped(RunRngScope::BurningEliteBuff,&semantic),"one draw for marked formation, none for ordinary elite");
        let combat=r.state.combat.as_ref().unwrap();
        if !fault {for enemy in combat.public_full().unwrap().enemies {
            assert_eq!(enemy.hp,1);
            for (index,id,want) in [(0,"strength",2),(2,"metallicize",4),(3,"regeneration",3)] {assert_eq!(enemy.powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.amount),if buff==Some(index){want}else{0});}
        }}
        if fault {let before=r.clone();assert_eq!(r.choose_combat(CombatAction::EndTurn),result);assert!(r==before);}
    }
}
