//! Internal room diagnostics; normal-start acceptance lives in tests/m7_act2.rs.
use crate::{combat::CombatAction, content::{CardId, EncounterId, PotionId, RelicId}, run::*};
use super::rooms::choose;

pub(super) fn continuing() -> Run {
    continuing_with_seen(vec![])
}

fn continuing_with_seen(seen:Vec<&'static str>) -> Run {
    let mut r = super::m5_rewards::boss_chest();
    choose(&mut r, RoomChoiceKind::Relic(RelicId::SlaversCollar));
    r.state.room_loop.as_mut().unwrap().scenario = RoomScenario::M7Act2Captured;
    r.state.seen_bosses[1] = seen;
    choose(&mut r, RoomChoiceKind::Proceed);
    r
}

pub(super) fn before(r: &mut Run, target: NodeId) {
    let map = r.state.map.as_mut().unwrap();
    let prev = map.nodes.iter_mut().find(|n| n.edges.contains(&target)).unwrap();
    prev.visited = true;
    r.state.floor = prev.floor;
    map.current = Some(prev.id);
}

#[test]
fn m7_late_act2_edges_keep_visit_and_loot_owners() {
    let mut r = continuing();
    assert_eq!((r.state.act, r.state.floor), (2, 17));
    assert_eq!(r.decision().unwrap().room.unwrap().kind, RoomViewKind::Map);
    let room = r.state.room_loop.as_mut().unwrap();
    room.purge_cost = 125;
    let p = room.act2.as_mut().unwrap();
    p.combat_visits = 5; p.question_visits = 3; p.shop_visits = 2; p.elite_relic_cursor = 4;
    r.state.map.as_mut().unwrap().regular_encounter_history = vec![EncounterId::Snecko, EncounterId::ChosenAndByrds];
    let nodes = r.state.map.as_ref().unwrap().nodes.clone();
    for node in nodes.iter().filter(|n| (27..=32).contains(&n.floor)) {
        let mut at = r.clone(); before(&mut at, node.id);
        let offered = at.decision().unwrap().candidates.into_iter().filter_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).collect::<Vec<_>>();
        assert_eq!(offered, at.state.map.as_ref().unwrap().nodes.iter().find(|n| Some(n.id)==at.state.map.as_ref().unwrap().current).unwrap().edges);
        let captured = super::super::rooms::act2::capture_room(&at.state, node.id).unwrap();
        if node.kind == MapNodeKind::Question { assert!(captured.encounter.is_some()); }
        let p = at.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap();
        assert_eq!((p.combat_visits,p.question_visits,p.shop_visits,p.elite_relic_cursor),(5,3,2,4));
        at.step(&GameAction::Move(node.id)).unwrap();
        assert_eq!(at.state.floor,node.floor);
        assert_eq!(at.state.room_loop.as_ref().unwrap().purge_cost,125);
        if captured.encounter.is_some() {assert!(at.state.combat.is_some());}
    }
    let chest = nodes.iter().find(|n| n.floor==26).unwrap().id;
    before(&mut r, chest); r.step(&GameAction::Move(chest)).unwrap();
    choose(&mut r, RoomChoiceKind::Proceed);
    assert_eq!(r.decision().unwrap().room.unwrap().kind, RoomViewKind::Map);
    assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,125);
    let screen = super::super::rooms::shop::screen(&r,&mut r.next_choice.clone()).unwrap();
    assert!(screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:CardId::Clothesline,upgrades:0}&&c.price==Some(50)));
}

#[test]
fn m7_old_profile_cutoffs_and_potion_gating_unchanged() {
    for (scenario, kind) in [(RoomScenario::M5Act1Captured, RoomViewKind::Act2Ready), (RoomScenario::M6Act2Prefix, RoomViewKind::Act2PrefixComplete)] {
        let mut r=super::m5_rewards::boss_chest();
        choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));
        r.state.room_loop.as_mut().unwrap().scenario=scenario;
        choose(&mut r,RoomChoiceKind::Proceed);
        if scenario==RoomScenario::M6Act2Prefix {
            let id=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==26).unwrap().id;
            before(&mut r,id);r.step(&GameAction::Move(id)).unwrap();choose(&mut r,RoomChoiceKind::Proceed);
        }
        assert_eq!(r.decision().unwrap().room.unwrap().kind,kind);
        assert!(r.state.potions.iter().all(Option::is_some));
        assert!(r.decision().unwrap().candidates.is_empty());
        let frozen=r.clone();assert!(r.step(&GameAction::DiscardPotion{slot:0}).is_err());assert!(r==frozen);
    }
}

#[test]
fn m7_late_pre_move_rejects_without_mutation() {
    let mut r=continuing();
    let rest=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==32).unwrap().id;
    before(&mut r,rest);r.next_choice=u64::MAX;
    let old=r.clone();assert!(r.step(&GameAction::Move(rest)).is_err());assert!(r==old);
    r.next_choice=1000;
    let old=r.clone();assert!(r.step(&GameAction::Move(NodeId(u64::MAX))).is_err());assert!(r==old);
}

#[test]
fn m7_native_entry_and_settlement_advance_act_local_visit_once() {
    let mut r=continuing();let global=r.state.combat_progress.settled;
    let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;
    r.step(&action).unwrap();assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,EncounterId::SphericGuardian);
    for _ in 0..32 {
        let d=r.decision().unwrap();if d.combat.is_none(){break}
        let a=d.candidates.into_iter().find_map(|c|match c.action {GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap_or(CombatAction::EndTurn);
        r.choose_combat(a).unwrap();
    }
    assert!(r.state.combat.is_none());assert_eq!(r.state.combat_progress.settled,global+1);
    assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().combat_visits,1);
    assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(PotionId::FirePotion)));
    let old=r.clone();assert!(r.choose_combat(CombatAction::EndTurn).is_err());assert!(r==old);
}

pub(super) fn boss_source(boss:EncounterId) -> (Run,NodeId) {
    let seen=match boss {
        EncounterId::Champ=>vec!["Automaton","Collector"],
        EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],
        EncounterId::Collector=>vec!["Champ","Automaton"],
        _=>panic!("not an Act2 boss"),
    };
    let mut r=continuing_with_seen(seen);
    let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.kind==MapNodeKind::Boss).unwrap().id;
    before(&mut r,node);(r,node)
}

#[test]
fn m7_boss_entry_uses_selected_icon_and_once_discovery() {
    for (boss,hp,name) in [(EncounterId::Champ,420,"Champ"),(EncounterId::BronzeAutomaton,300,"Automaton"),(EncounterId::Collector,282,"Collector")] {
        let (mut r,node)=boss_source(boss);
        assert_eq!(r.public_map().unwrap().boss,Some(boss));
        assert!(!r.state.seen_bosses[1].contains(&name));
        r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);
        r.step(&GameAction::Move(node)).unwrap();
        let active=r.state.active_combat.as_ref().unwrap();
        assert_eq!((r.state.act,r.state.floor,active.encounter),(2,33,boss));
        assert_eq!(active.origin,CombatOrigin::MapRoom{node,kind:MapNodeKind::Boss});
        let c=r.decision().unwrap().combat.unwrap();
        assert_eq!((c.enemies[0].hp,c.player.max_energy),(hp,4));
        assert_eq!(r.state.seen_bosses[1].iter().filter(|n|**n==name).count(),1);
        let old=r.clone();assert!(r.step(&GameAction::Move(node)).is_err());assert!(r==old);
    }
}

#[test]
fn m7_boss_cleanup_and_death_reach_one_settlement_owner() {
    for boss in [EncounterId::BronzeAutomaton,EncounterId::Collector] {
        let (mut r,node)=boss_source(boss);
        // Supplied-resource lifecycle diagnostic, NEVER normal-start evidence.
        r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);
        r.state.hp=300;r.state.max_hp=300;
        r.state.deck=[CardId::Bludgeon,CardId::Bludgeon,CardId::Bash,CardId::Disarm,CardId::Impervious,CardId::Offering,CardId::Anger,CardId::ShrugItOff].into_iter().enumerate().map(|(i,id)|CardInstance{ misc:0,instance:i as u64+1,id,upgrades:1}).collect();r.state.next_instance=9;
        let deck=r.state.deck.clone();let global=r.state.combat_progress.settled;
        r.step(&GameAction::Move(node)).unwrap();
        for _ in 0..2 {r.choose_combat(CombatAction::EndTurn).unwrap();}
        assert!(r.decision().unwrap().combat.unwrap().enemies.len()>=3);
        // Drive actual actions through native damage/cleanup and Run settlement.
        for _ in 0..256 {
            let d=r.decision().unwrap();if d.combat.is_none(){break}
            let legal=d.candidates.iter().filter_map(|c|match &c.action{GameAction::Combat(a)=>Some(a.clone()),_=>None}).collect::<Vec<_>>();
            let c=d.combat.as_ref().unwrap();
            // Prefer boss over adds; explicit native diagnostics do not replace caller evidence.
            let a=legal.iter().filter_map(|a|match a {CombatAction::PlayCard{instance,target}=>Some((a,instance,target)),_=>None}).find(|(_,id,target)|c.hand.iter().any(|card|card.instance.0==**id&&matches!(card.id,CardId::Bludgeon|CardId::Bash|CardId::Anger))&&target.is_some_and(|t|t.0==1)).map(|(a,_,_)|a.clone()).or_else(||legal.iter().find(|a|matches!(a,CombatAction::PlayCard{target:None,..})).cloned()).unwrap_or(CombatAction::EndTurn);
            r.choose_combat(a).unwrap();
        }
        assert_eq!(r.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);
        assert!(r.state.combat.is_none());assert_eq!(r.state.deck,deck);
        assert_eq!(r.state.combat_progress.settled,global+1);
        assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().combat_visits,1);
        assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Rewards);
    }
    let (mut r,node)=boss_source(EncounterId::Champ);r.step(&GameAction::Move(node)).unwrap();
    for _ in 0..32 {if r.state.terminal.is_some(){break}r.choose_combat(CombatAction::EndTurn).unwrap();}
    assert_eq!(r.state.terminal,Some(RunTerminal::Death));assert!(r.decision().unwrap().candidates.is_empty());
    assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::ActAdvanced{from:2,to:3})));
}

#[test]
fn m7_boss_entry_fault_keeps_accepted_source() {
    let (mut r,node)=boss_source(EncounterId::Champ);
    r.state.relics.push(RelicId::PureWater);r.state.deck=vec![CardInstance{ misc:0,instance:u64::MAX,id:CardId::Strike,upgrades:0}];
    let prior=r.state.reward_ledger.total;let err=r.step(&GameAction::Move(node)).unwrap_err();
    assert!(matches!(err,RunError::Faulted(_)));assert_eq!((r.state.floor,r.state.reward_ledger.total,r.state.terminal),(33,prior+1,None));
    assert!(r.state.combat.is_some());assert!(r.state.relics.is_empty());
    let frozen=r.clone();assert_eq!(r.step(&GameAction::Move(node)),Err(err));assert!(r==frozen);
}
