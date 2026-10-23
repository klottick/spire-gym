use crate::{combat::CombatAction,content::{CardId,EncounterId,PotionId,RelicId},run::*};
use super::rooms::choose;

pub(super) fn victory(boss:EncounterId)->Run {
    let (mut r,node)=super::m7_rooms::boss_source(boss);
    r.step(&GameAction::Move(node)).unwrap();
    for _ in 0..32 {
        let d=r.decision().unwrap();if d.combat.is_none(){return r}
        let a=d.candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap_or(CombatAction::EndTurn);
        r.choose_combat(a).unwrap();
    }panic!("supplied Lament room diagnostic did not settle")
}

pub(super) fn chest()->Run {
    let mut r=victory(EncounterId::Champ);choose(&mut r,RoomChoiceKind::Proceed);
    assert_eq!((r.state.act,r.state.floor),(2,34));choose(&mut r,RoomChoiceKind::OpenChest);r
}

#[test]
fn m7_boss_rewards_are_source_ordered_and_not_elite_loot() {
    for boss in [EncounterId::Champ,EncounterId::BronzeAutomaton,EncounterId::Collector] {
        let (mut r,node)=super::m7_rooms::boss_source(boss);r.step(&GameAction::Move(node)).unwrap();
        let mut expected=r.state.rng.clone();let visit=r.state.active_combat.as_ref().unwrap().visit;
        let gold=95+expected.bounded_scoped(RunRngScope::CombatGoldReward,&[2,node.0,visit,MapNodeKind::Boss as u64],11) as u32;
        for _ in 0..32 {let d=r.decision().unwrap();if d.combat.is_none(){break}let a=d.candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap_or(CombatAction::EndTurn);r.choose_combat(a).unwrap();}
        assert_eq!(r.state.rng,expected);
        let screen=r.decision().unwrap().room.unwrap();
        assert!(screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Gold(gold)));
        assert!(!screen.choices.iter().any(|c|matches!(c.kind,RoomChoiceKind::Relic(_)|RoomChoiceKind::Emerald)));
        let potion=screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Potion(PotionId::FirePotion)).unwrap();assert_eq!(potion.status,RoomChoiceStatus::Disabled);
        let old=r.clone();assert!(r.step(&GameAction::Choose(potion.id)).is_err());assert!(r==old);
        choose(&mut r,RoomChoiceKind::CardReward);
        let cards=r.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
        assert_eq!(cards,vec![CardId::Bludgeon,CardId::Impervious,CardId::Offering]);
        let source_proceed=screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;
        let old=r.clone();assert!(r.step(&GameAction::Choose(source_proceed)).is_err());assert!(r==old);
        choose(&mut r,RoomChoiceKind::Skip);let t=choose(&mut r,RoomChoiceKind::Proceed);
        assert_eq!(t.reward,1);assert_eq!(t.committed_events,vec![CommittedEvent::FloorEntered{act:2,floor:34}]);
        assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::BossTreasure);
        let old=r.clone();assert!(r.step(&GameAction::Choose(source_proceed)).is_err());assert!(r==old);
    }
}

#[test]
fn m7_second_chest_disables_owned_or_ineligible_captures() {
    let mut r=chest();let screen=r.decision().unwrap().room.unwrap();
    assert_eq!(screen.choices.iter().filter(|c|matches!(c.kind,RoomChoiceKind::Relic(_))).count(),3);
    let collar=screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::SlaversCollar)).unwrap();
    assert_eq!(collar.status,RoomChoiceStatus::Disabled);let old=r.clone();assert!(r.step(&GameAction::Choose(collar.id)).is_err());assert!(r==old);
    choose(&mut r,RoomChoiceKind::Skip);let warning=r.decision().unwrap().room.unwrap();
    assert_eq!(warning.kind,RoomViewKind::ProceedWithoutRelic);
    let old=r.clone();assert!(r.step(&GameAction::Choose(screen.choices[0].id)).is_err());assert!(r==old);
    choose(&mut r,RoomChoiceKind::CancelSkip);assert_eq!(r.decision().unwrap().room.unwrap(),screen);
    let len=r.state.relics.len();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));
    assert_eq!(r.state.relics.len(),len);assert_eq!(r.state.relics[0],RelicId::BlackBlood);assert!(!r.state.relics.contains(&RelicId::BurningBlood));assert!(r.state.relics.contains(&RelicId::SlaversCollar));
    let mut r=chest();r.state.relics.remove(0);
    let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::BlackBlood)).unwrap();
    let old=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==old);
}

#[test]
fn m7_empty_cage_and_confirmed_skip_keep_source_custody() {
    let mut r=chest();let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();
    choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));
    assert!(r.state.relics.contains(&RelicId::EmptyCage));assert_eq!(r.state.deck.len(),ids.len());
    assert!(!r.decision().unwrap().candidates.iter().any(|c|matches!(c.action,GameAction::Choose(_)|GameAction::Cancel)));
    for _ in 0..2 {let id=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(id)).unwrap();}
    assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),ids[2..]);
    assert!(r.state.pending.is_none());assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::BossRelicChild);
    assert_eq!((r.state.act,r.state.floor),(2,34)); // Actual handoff belongs to Task4.
}

#[test]
fn m7_empty_cage_small_decks_never_fabricate_readiness() {
    for count in [0,1,2] {
        let mut r=chest();r.state.deck.truncate(count);choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));
        if count==0 {choose(&mut r,RoomChoiceKind::Proceed);}
        assert_eq!(r.state.terminal,Some(RunTerminal::EmptyDeck));assert_eq!((r.state.act,r.state.floor),(2,34));
        assert!(r.decision().unwrap().candidates.is_empty());
    }
}

#[test]
fn m7_reward_preflight_and_accepted_child_fault_differ() {
    let mut r=victory(EncounterId::Champ);r.next_choice=u64::MAX;
    let id=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;
    let old=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==old);
    let (mut r,node)=super::m7_rooms::boss_source(EncounterId::Champ);r.step(&GameAction::Move(node)).unwrap();r.next_choice=u64::MAX;
    let a=r.decision().unwrap().candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap();
    let err=r.choose_combat(a).unwrap_err();assert!(matches!(err,RunError::Faulted(_)));
    assert!(r.state.combat.is_some());assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(crate::combat::Outcome::Victory));
    assert!(r.state.last_combat.as_ref().is_some_and(|c|c.encounter!=EncounterId::Champ));
    assert_eq!((r.state.act,r.state.floor,r.state.terminal),(2,33,None));
    let old=r.clone();assert_eq!(r.step(&GameAction::Choose(id)),Err(err));assert!(r==old);
}
