use crate::{combat::{CombatAction,TargetId},content::{CardId,PotionId,RelicId},run::*};
use super::rooms::{run,choose};
pub(super) fn victory()->Run {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);r.step(&GameAction::Move(NodeId(2))).unwrap();
    let d=r.decision().unwrap();let id=d.combat.unwrap().hand.into_iter().find(|c|matches!(c.id,CardId::Strike|CardId::Bash)).unwrap().instance.0;
    r.step(&GameAction::Combat(CombatAction::PlayCard{instance:id,target:Some(TargetId(1))})).unwrap();r
}
#[test]
fn native_victory_captures_unclaimed_loot_and_one_permanent_card() {
    let mut r=victory();let d=r.decision().unwrap();assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Rewards);
    assert_eq!((d.player.gold,d.objects.len(),d.reward_total),(99,10,1));
    let amount=d.room.unwrap().choices.into_iter().find_map(|c|if let RoomChoiceKind::Gold(n)=c.kind{Some(n)}else{None}).unwrap();assert!((10..=20).contains(&amount));
    choose(&mut r,RoomChoiceKind::Gold(amount));assert_eq!(r.state.gold,99+amount);
    let before=r.clone();assert_eq!(r.decision().unwrap(),r.decision().unwrap());assert!(r==before);
    choose(&mut r,RoomChoiceKind::CardReward);choose(&mut r,RoomChoiceKind::Card{id:CardId::ShrugItOff,upgrades:0});
    assert_eq!(r.state.deck.len(),11);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Rewards);
    choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}
#[test]
fn full_belt_reward_remains_claimable_after_explicit_discard() {
    let mut r=victory();r.state.potions=vec![Some(PotionId::FirePotion);3];
    let d=r.decision().unwrap();let row=d.room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Potion(PotionId::FirePotion)).unwrap();
    let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==before);
    r.step(&GameAction::DiscardPotion{slot:1}).unwrap();choose(&mut r,RoomChoiceKind::Potion(PotionId::FirePotion));
    assert_eq!(r.state.potions,vec![Some(PotionId::FirePotion);3]);
}
#[test]
fn failed_reward_preparation_retains_terminal_combat_and_first_fault() {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);r.step(&GameAction::Move(NodeId(2))).unwrap();
    r.next_choice=u64::MAX;
    let d=r.decision().unwrap();let id=d.combat.unwrap().hand.into_iter().find(|c|matches!(c.id,CardId::Strike|CardId::Bash)).unwrap().instance.0;
    let error=r.step(&GameAction::Combat(CombatAction::PlayCard{instance:id,target:Some(TargetId(1))})).unwrap_err();
    assert!(matches!(error,RunError::Faulted(_)));assert!(r.state.combat.is_some());assert_eq!(r.state.combat_progress.settled,0);
    assert!(r.state.relics.is_empty());let before=r.clone();assert_eq!(r.step(&GameAction::Combat(CombatAction::EndTurn)),Err(error));assert!(r==before);
}
#[test]
fn chest_strawberry_and_sapphire_are_exclusive_actual_acquisitions() {
    for key in [false,true] {
        let mut r=run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});
        // Native room-boundary fixture: legal last edge with injured80max player.
        r.state.hp=40;r.state.max_hp=80;
        let m=r.state.map.as_mut().unwrap();m.current=Some(NodeId(52));m.nodes.iter_mut().find(|n|n.id==NodeId(52)).unwrap().visited=true;
        r.step(&GameAction::Move(NodeId(59))).unwrap();assert!(!r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Strawberry)));
        choose(&mut r,RoomChoiceKind::OpenChest);
        choose(&mut r,if key{RoomChoiceKind::Sapphire}else{RoomChoiceKind::Relic(RelicId::Strawberry)});
        if key {assert_eq!((r.state.hp,r.state.max_hp),(40,80));assert_eq!(r.state.keys,[false,true,false]);assert_eq!(r.decision().unwrap().reward_total,6)}
        else{assert_eq!((r.state.hp,r.state.max_hp),(47,87));assert!(r.state.relics.contains(&RelicId::Strawberry));}
        choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::CheckpointComplete);assert_eq!(r.state.terminal,None);
    }
}

#[test]
fn acquisition_overflow_and_stale_claims_preserve_offer_and_resources() {
    let mut r=victory();choose(&mut r,RoomChoiceKind::CardReward);r.state.next_instance=u64::MAX;
    let d=r.decision().unwrap();let id=d.room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::Card{..})).unwrap().id;
    let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
    r.state.next_instance=12;r.step(&GameAction::Choose(id)).unwrap();let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
}

#[test]
fn unopened_offers_and_hidden_future_rooms_are_not_public_debug_data() {
    let mut r=victory();let before=r.clone();let public=r.decision().unwrap();
    assert!(!public.room.as_ref().unwrap().choices.iter().any(|c|matches!(c.kind,RoomChoiceKind::Card{..})));
    assert!(!format!("{r:?}").contains("Purifier"));assert!(!format!("{r:?}").contains("Strawberry"));
    let mut other=r.clone();other.state.stored_note=(CardId::Anger,1);other.state.map.as_mut().unwrap().draft_ordinal=999;
    assert_eq!(r.decision().unwrap(),other.decision().unwrap());assert_eq!(r.public_map(),other.public_map());assert!(r==before);
    choose(&mut r,RoomChoiceKind::CardReward);assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().filter(|c|matches!(c.kind,RoomChoiceKind::Card{..})).count(),3);
}
