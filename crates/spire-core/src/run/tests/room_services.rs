use crate::{content::{CardId,PotionId},run::*};
use super::rooms::{run,choose};
pub(super) fn room(target:u64)->Run {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});
    let m=r.state.map.as_mut().unwrap();m.current=Some(NodeId(target-7));m.nodes.iter_mut().find(|n|n.id==NodeId(target-7)).unwrap().visited=true;
    r.step(&GameAction::Move(NodeId(target))).unwrap();r
}
fn select(r:&mut Run,instance:u64)->GameAction{r.decision().unwrap().candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.instance==instance)).unwrap().action.clone()}
#[test]
fn shop_paid_purge_cancel_and_later_price_retain_exact_parent() {
    let mut r=room(17);let shop=r.decision().unwrap().room.unwrap();
    choose(&mut r,RoomChoiceKind::Purge);assert_eq!(r.state.gold,99);
    let old=select(&mut r,1);r.step(&GameAction::Cancel).unwrap();assert_eq!(r.decision().unwrap().room.unwrap(),shop);
    choose(&mut r,RoomChoiceKind::Purge);let before=r.clone();assert!(r.step(&old).is_err());assert!(r==before);
    let action=select(&mut r,1);r.step(&action).unwrap();assert_eq!(r.state.gold,24);assert_eq!(r.state.deck.len(),9);assert!(!r.state.deck.iter().any(|c|c.instance==1));
    assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Shop);
    choose(&mut r,RoomChoiceKind::Leave);
    let m=r.state.map.as_mut().unwrap();m.current=Some(NodeId(45));m.nodes.iter_mut().find(|n|n.id==NodeId(45)).unwrap().visited=true;
    r.step(&GameAction::Move(NodeId(52))).unwrap();let d=r.decision().unwrap();let purge=d.room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Purge).unwrap();
    assert_eq!((purge.price,purge.status),(Some(100),RoomChoiceStatus::Disabled));let before=r.clone();assert!(r.step(&GameAction::Choose(purge.id)).is_err());assert!(r==before);
    r.state.gold=114;choose(&mut r,RoomChoiceKind::Purge);let action=select(&mut r,2);r.step(&action).unwrap();
    assert_eq!((r.state.gold,r.state.deck.len(),r.state.room_loop.as_ref().unwrap().purge_cost),(14,8,125));
}
#[test]
fn shop_stock_acquires_once_and_full_belt_never_spends() {
    let mut r=room(17);choose(&mut r,RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0});assert_eq!((r.state.gold,r.state.deck.len()),(74,11));
    r.state.potions=vec![Some(PotionId::FirePotion);3];let before=r.clone();
    let id=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Potion(PotionId::FirePotion)).unwrap().id;
    assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
    r.step(&GameAction::DiscardPotion{slot:0}).unwrap();r.step(&GameAction::Choose(id)).unwrap();assert_eq!(r.state.gold,24);assert_eq!(r.state.potions[0],Some(PotionId::FirePotion));
}
#[test]
fn shrine_removal_is_committed_and_smith_cancel_returns_to_campfire() {
    let mut r=room(31);choose(&mut r,RoomChoiceKind::ShrineRemove);
    let before=r.clone();assert!(r.step(&GameAction::Cancel).is_err());assert!(r==before);
    let action=select(&mut r,1);r.step(&action).unwrap();assert_eq!(r.state.deck.len(),9);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    let mut r=room(38);let parent=r.decision().unwrap().room.unwrap();choose(&mut r,RoomChoiceKind::Smith);r.step(&GameAction::Cancel).unwrap();assert_eq!(r.decision().unwrap().room.unwrap(),parent);
    choose(&mut r,RoomChoiceKind::Smith);let bash=r.state.deck.iter().find(|c|c.id==CardId::Bash).unwrap().instance;let action=select(&mut r,bash);r.step(&action).unwrap();assert_eq!(r.state.deck.iter().find(|c|c.instance==bash).unwrap().upgrades,1);
}
#[test]
fn rest_and_recall_have_exclusive_real_consequences() {
    let mut r=room(38);r.state.hp=40;choose(&mut r,RoomChoiceKind::Rest{heal:26});assert_eq!(r.state.hp,66);
    let mut r=room(38);r.state.hp=40;let t=choose(&mut r,RoomChoiceKind::Recall);assert_eq!(t.reward,5);assert_eq!(r.state.hp,40);assert_eq!(r.state.keys,[true,false,false]);
    let mut r=room(38);r.state.hp=80;choose(&mut r,RoomChoiceKind::Rest{heal:26});assert_eq!(r.state.hp,88);
}
#[test]
fn outside_potion_menu_and_use_share_preflight() {
    let mut r=room(17);r.state.hp=40;r.state.potions=vec![Some(PotionId::BloodPotion),Some(PotionId::FruitJuice),Some(PotionId::FirePotion)];
    let before=r.clone();assert!(r.step(&GameAction::UsePotion{slot:2,target:None}).is_err());assert!(r==before);
    assert!(r.step(&GameAction::UsePotion{slot:0,target:Some(crate::combat::TargetId(1))}).is_err());assert!(r==before);
    r.step(&GameAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(r.state.hp,57);
    r.step(&GameAction::UsePotion{slot:1,target:None}).unwrap();assert_eq!((r.state.hp,r.state.max_hp),(62,93));
}

#[test]
fn stale_room_parent_is_rejected_before_payment_or_removal() {
    let mut r=room(17);choose(&mut r,RoomChoiceKind::Purge);let action=select(&mut r,1);
    if let ContinuationTarget::Room{choice,..}=&mut r.state.pending.as_mut().unwrap().parent{*choice=ChoiceId(u64::MAX-1)}
    let before=r.clone();assert!(r.step(&action).is_err());assert!(r==before);
}
#[test]
fn removing_last_card_settles_empty_deck_once_after_callback() {
    let mut r=room(31);r.state.deck.truncate(1);choose(&mut r,RoomChoiceKind::ShrineRemove);let action=select(&mut r,1);
    let t=r.step(&action).unwrap();assert_eq!(t.terminal,Some(RunTerminal::EmptyDeck));assert_eq!(t.reward,-1);assert!(r.state.deck.is_empty());
}

#[test]
fn empty_and_ineligible_grids_keep_source_specific_controls() {
    let mut r=room(38);for card in &mut r.state.deck {card.upgrades=1}
    let d=r.decision().unwrap();let view=d.room.unwrap();assert_eq!(view.choices.iter().find(|c|c.kind==RoomChoiceKind::Smith).unwrap().status,RoomChoiceStatus::Disabled);
    assert!(view.choices.iter().any(|c|matches!(c.kind,RoomChoiceKind::Rest{..})&&c.status==RoomChoiceStatus::Offered));assert!(!view.choices.iter().any(|c|c.kind==RoomChoiceKind::Leave));
    let mut r=room(31);r.state.deck.clear();let id=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::ShrineRemove).unwrap().id;
    let error=r.step(&GameAction::Choose(id)).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.terminal,None);
    let before=r.clone();assert_eq!(r.step(&GameAction::Cancel),Err(error));assert!(r==before);
}

#[test]
fn smith_upgrade_is_consumed_by_native_bash_not_only_an_observation() {
    use crate::combat::{Combat,CombatAction,CardInstanceId,TargetId,PlayerState,permanent_live_card,enemies::{self,SetupContext}};
    let mut r=room(38);choose(&mut r,RoomChoiceKind::Smith);let bash=r.state.deck.iter().find(|c|c.id==CardId::Bash).unwrap().instance;
    let action=select(&mut r,bash);r.step(&action).unwrap();let card=r.state.deck.iter().find(|c|c.instance==bash).unwrap();
    let mut entry=enemies::setup(crate::content::EncounterId::JawWorm,1,0,&mut r.state.rng,SetupContext{
        player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],
        character:crate::content::Character::Ironclad,cards:vec![permanent_live_card(card.id,CardInstanceId(card.instance),card.upgrades)],opening_draw:1,encounter_visit:0}).unwrap();
    entry.setup.enemies[0].hp=42;entry.setup.enemies[0].max_hp=42;
    let mut combat=Combat::enter(entry).unwrap();combat.step_full(&CombatAction::PlayCard{instance:bash,target:Some(TargetId(1))}).unwrap();
    let e=&combat.public_full().unwrap().enemies[0];assert_eq!(e.hp,32);assert_eq!(e.powers.iter().find(|p|p.id.0=="vulnerable").unwrap().amount,3);
}
