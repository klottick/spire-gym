use crate::{combat::CombatAction,content::{CardId,EncounterId,PotionId,RelicId},run::*};
use super::rooms::choose;

pub(super) fn victory(kind:MapNodeKind,marked:bool)->Run {
    victory_with_ordinal(kind,marked,0)
}
fn victory_with_ordinal(kind:MapNodeKind,marked:bool,ordinal:u64)->Run {
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;profile.seen_bosses[0]=vec!["Guardian","Hexaghost"];
    let mut r=Run::new_room_scenario(RunConfig{character:crate::content::Character::Ironclad,ascension:0},profile,RunRng::new([7;32],crate::combat::rng::Partition::Test,ordinal),RoomScenario::M5Act1Captured).unwrap();choose(&mut r,RoomChoiceKind::NeowLament);
    let map=r.state.map.as_mut().unwrap();let target=map.nodes.iter().find(|n|n.kind==kind).unwrap().id;
    if kind!=MapNodeKind::Monster {let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=prev.floor;}
    for node in &mut map.nodes{node.burning=marked&&node.id==target;}
    r.state.potions=vec![Some(PotionId::StrengthPotion);3];r.step(&GameAction::Move(target)).unwrap();
    for _ in 0..32 {
        let d=r.decision().unwrap();if d.combat.is_none(){return r}
        let strikes=d.combat.as_ref().unwrap().hand.iter().filter(|c|c.id==CardId::Strike).map(|c|c.instance.0).collect::<Vec<_>>();
        let action=d.candidates.iter().find_map(|c|match c.action {GameAction::Combat(ref a@CombatAction::PlayCard{instance,target:Some(_),..})if strikes.contains(&instance)=>Some(a.clone()),_=>None})
            .or_else(||d.candidates.iter().find_map(|c|match c.action{GameAction::Combat(ref a@CombatAction::PlayCard{target:Some(_),..})=>Some(a.clone()),_=>None})).unwrap_or(CombatAction::EndTurn);
        r.choose_combat(action).unwrap();
    }panic!("bounded fixture failed to settle")
}

#[test]
fn m5_terminal_loot_is_classified_captured_and_not_claimed_by_preparation() {
    for (kind,low,high) in [(MapNodeKind::Monster,10,20),(MapNodeKind::Elite,25,35),(MapNodeKind::Boss,95,105)] {
        let mut r=victory(kind,false);assert_eq!(r.state.gold,99);assert_eq!(r.state.deck.len(),10);assert_eq!(r.state.combat_progress.settled,1);
        let screen=r.decision().unwrap().room.unwrap();let gold=screen.choices.iter().find_map(|c|match c.kind{RoomChoiceKind::Gold(n)=>Some(n),_=>None}).unwrap();assert!((low..=high).contains(&gold));
        let potion=screen.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Potion(_))).unwrap();assert_eq!(potion.status,RoomChoiceStatus::Disabled);
        let before=r.clone();assert!(r.step(&GameAction::Choose(potion.id)).is_err());assert!(r==before);
        choose(&mut r,RoomChoiceKind::CardReward);let cards=r.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
        assert_eq!(cards,if kind==MapNodeKind::Boss{vec![CardId::Bludgeon,CardId::Impervious,CardId::Offering]}else{vec![CardId::IronWave,CardId::ShrugItOff,CardId::Anger]});
        choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::Gold(gold));assert_eq!(r.state.gold,99+gold);
        let before=r.clone();let id=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::Gold(_))).unwrap().id;assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
    }
}

#[test]
fn m5_marked_elite_relic_and_emerald_are_independent_once_only_claims() {
    let mut r=victory(MapNodeKind::Elite,true);assert!(!r.state.keys[2]);
    choose(&mut r,RoomChoiceKind::Relic(RelicId::Lantern));assert!(!r.state.keys[2]);
    let t=choose(&mut r,RoomChoiceKind::Emerald);assert_eq!(t.reward,5);assert!(r.state.keys[2]);assert!(r.state.relics.contains(&RelicId::Lantern));
    let before=r.clone();let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Emerald).unwrap().id;
    assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
}

#[test]
fn m5_rare_take_has_one_fresh_permanent_identity_and_status_obtain_rejects() {
    let mut r=victory(MapNodeKind::Boss,false);let next=r.state.next_instance;choose(&mut r,RoomChoiceKind::CardReward);
    choose(&mut r,RoomChoiceKind::Card{id:CardId::Offering,upgrades:0});assert_eq!(r.state.deck.last().unwrap(),&CardInstance{id:CardId::Offering,instance:next,upgrades:0});assert_eq!(r.state.next_instance,next+1);
    let before=r.clone();assert!(super::super::mutations::obtain_card(&mut r.state,CardId::Burn,0).is_err());assert!(r==before);
    assert_eq!(r.state.last_combat.as_ref().unwrap().encounter,EncounterId::SlimeBoss);
}

pub(super) fn boss_chest()->Run {let mut r=victory(MapNodeKind::Boss,false);choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.state.floor,17);choose(&mut r,RoomChoiceKind::OpenChest);r}

#[test]
fn m5_boss_treasure_enters_once_and_skip_warning_returns_same_captured_chest() {
    let mut r=victory(MapNodeKind::Boss,false);let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,1);assert_eq!(r.state.floor,17);
    let before_open=r.decision().unwrap().room.unwrap();assert_eq!(before_open.kind,RoomViewKind::BossTreasure);assert!(!before_open.choices.iter().any(|c|matches!(c.kind,RoomChoiceKind::Relic(_))));
    choose(&mut r,RoomChoiceKind::OpenChest);let captured=r.decision().unwrap().room.unwrap();
    assert_eq!(captured.choices.iter().filter_map(|c|match c.kind{RoomChoiceKind::Relic(id)=>Some(id),_=>None}).collect::<Vec<_>>(),vec![RelicId::BlackBlood,RelicId::EmptyCage,RelicId::SlaversCollar]);assert!(!captured.choices.iter().any(|c|c.kind==RoomChoiceKind::Sapphire));
    choose(&mut r,RoomChoiceKind::Skip);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::ProceedWithoutRelic);
    choose(&mut r,RoomChoiceKind::CancelSkip);assert_eq!(r.decision().unwrap().room.unwrap(),captured);
    let stale=before_open.choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;let before=r.clone();assert!(r.step(&GameAction::Choose(stale)).is_err());assert!(r==before);
    let len=r.state.relics.len();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));assert_eq!(r.state.relics.len(),len);assert_eq!(r.state.relics[0],RelicId::BlackBlood);assert!(!r.state.relics.contains(&RelicId::BurningBlood));
    assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::BossRelicChild);let before=r.clone();assert!(r.step(&GameAction::Choose(captured.choices[1].id)).is_err());assert!(r==before);
}

#[test]
fn m5_empty_cage_keeps_zero_auto_and_mandatory_two_pick_branches_distinct() {
    for count in [0,1,2,3] {
        let mut r=boss_chest();r.state.deck.truncate(count);let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();let gold=r.state.gold;let next=r.state.next_instance;
        choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));assert!(r.state.relics.contains(&RelicId::EmptyCage));assert_eq!((r.state.gold,r.state.next_instance),(gold,next));
        match count {
            0=>{assert_eq!(r.state.terminal,None);assert!(r.state.pending.is_none());choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.state.terminal,Some(RunTerminal::EmptyDeck));},
            1|2=>{assert!(r.state.deck.is_empty());assert_eq!(r.state.terminal,Some(RunTerminal::EmptyDeck));assert_eq!(r.state.last_selection.as_ref().unwrap().iter().map(|c|c.instance).collect::<Vec<_>>(),ids);},
            _=>{
                assert_eq!(r.state.pending.as_ref().unwrap().selection.required,2);assert_eq!(r.state.deck.len(),3);
                assert!(!r.decision().unwrap().candidates.iter().any(|c|matches!(c.action,GameAction::Cancel|GameAction::Choose(_))));
                let first=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(first)).unwrap();assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),ids);
                let before=r.clone();assert!(r.step(&GameAction::Select(first)).is_err());assert!(r==before);
                let second=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(second)).unwrap();assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),vec![ids[2]]);assert!(r.state.pending.is_none());assert_eq!(r.state.terminal,None);
            }
        }
    }
}

#[test]
fn m5_boss_gold_one_hundred_adds_to_ninety_nine_exactly_once() {
    let mut r=victory_with_ordinal(MapNodeKind::Boss,false,0);assert_eq!(r.state.gold,99);choose(&mut r,RoomChoiceKind::Gold(100));assert_eq!(r.state.gold,199);
}

#[test]
fn m5_stale_card_parent_and_cage_publication_overflow_reject_before_mutation() {
    let mut r=victory(MapNodeKind::Boss,false);choose(&mut r,RoomChoiceKind::CardReward);r.state.room_loop.as_mut().unwrap().reward_parent=None;
    let id=r.decision().unwrap().room.unwrap().choices[0].id;let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before,"stale card parent must not acquire a permanent card");
    let mut r=boss_chest();r.state.deck.truncate(3);choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));r.next_choice=u64::MAX;
    let first=r.state.pending.as_ref().unwrap().selection.candidates[0].id;let before=r.clone();assert!(r.step(&GameAction::Select(first)).is_err());assert!(r==before,"overflow must not consume a captured pick");
}

#[test]
fn m5_cards_and_potions_capture_each_zero_based_combat_visit_once() {
    let mut r=super::m5_act1::run();choose(&mut r,RoomChoiceKind::NeowLament);let mut verified=0;
    for _ in 0..128 {
        let d=r.decision().unwrap();
        if d.combat.is_some(){let action=d.candidates.iter().find_map(|c|match &c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a.clone()),_=>None}).unwrap_or(CombatAction::EndTurn);r.choose_combat(action).unwrap();continue}
        match d.room.unwrap().kind {
            RoomViewKind::Rewards=>{
                let potion=[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][verified];assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(potion)));
                choose(&mut r,RoomChoiceKind::CardReward);let triple=r.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
                assert_eq!(triple,[[CardId::IronWave,CardId::ShrugItOff,CardId::Anger],[CardId::Clothesline,CardId::PommelStrike,CardId::Disarm],[CardId::Cleave,CardId::TwinStrike,CardId::Thunderclap]][verified]);
                choose(&mut r,RoomChoiceKind::Skip);verified+=1;if verified==3{return}choose(&mut r,RoomChoiceKind::Proceed);
            },
            RoomViewKind::Map=>{let action=d.candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;r.step(&action).unwrap();},
            _=>{choose(&mut r,RoomChoiceKind::Leave);},
        }
    }panic!("three captured combat visits not reached")
}

#[test]
fn m5_elite_capture_advances_on_offer_even_when_skipped_and_never_omits_owned_rows() {
    use super::super::rooms::rewards::capture_elite_relic;
    let mut r=victory(MapNodeKind::Elite,false);assert_eq!(r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().elite_relic_cursor,1);
    choose(&mut r,RoomChoiceKind::Proceed);assert!(!r.state.relics.contains(&RelicId::Lantern));
    let cursor=r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().elite_relic_cursor;assert_eq!(capture_elite_relic(cursor,&r.state.relics).unwrap(),(RelicId::Pear,2));
    assert_eq!(capture_elite_relic(0,&[RelicId::Lantern,RelicId::Pear]).unwrap(),(RelicId::Mango,3));
    assert_eq!(capture_elite_relic(5,&[RelicId::Strawberry]).unwrap(),(RelicId::Anchor,7));
    let mut cursor=0;let mut unclaimed=vec![];for _ in 0..8{let (relic,next)=capture_elite_relic(cursor,&[]).unwrap();assert!(!unclaimed.contains(&relic));unclaimed.push(relic);cursor=next;}
    assert!(capture_elite_relic(cursor,&[]).is_err(),"exhaustion is explicit, not a missing or duplicate drop");
}

#[test]
fn m5_later_merchant_capture_changes_stock_and_suppresses_owned_relic_without_changing_m4() {
    let mut r=super::m5_act1::run();choose(&mut r,RoomChoiceKind::NeowLament);r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().shop_visits=1;
    let screen=super::super::rooms::shop::screen(&r,&mut r.next_choice.clone()).unwrap();
    assert!(screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:CardId::Clothesline,upgrades:0}&&c.price==Some(50)));
    assert!(screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(PotionId::StrengthPotion)&&c.price==Some(50)));
    r.state.relics.push(RelicId::Anchor);let screen=super::super::rooms::shop::screen(&r,&mut r.next_choice.clone()).unwrap();assert!(!screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Anchor)));
    let m4=super::rooms::run();let screen=super::super::rooms::shop::screen(&m4,&mut m4.next_choice.clone()).unwrap();assert!(screen.choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0}&&c.price==Some(25)));
}

#[test]
fn m5_ordinary_chest_captures_next_unowned_relic_and_keeps_sapphire_exclusive() {
    let mut r=super::m5_act1::run();choose(&mut r,RoomChoiceKind::NeowLament);r.state.relics.push(RelicId::Strawberry);
    let map=r.state.map.as_mut().unwrap();let target=map.nodes.iter().find(|n|n.kind==MapNodeKind::Treasure).unwrap().id;let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=prev.floor;
    r.step(&GameAction::Move(target)).unwrap();choose(&mut r,RoomChoiceKind::OpenChest);
    assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Anchor)));
    choose(&mut r,RoomChoiceKind::Sapphire);assert!(!r.state.relics.contains(&RelicId::Anchor));assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::Anchor)).unwrap().status,RoomChoiceStatus::Skipped);
}

#[test]
fn m5_boss_relic_eligibility_and_cage_source_price_reject_stale_children_atomically() {
    let mut r=boss_chest();r.state.relics.remove(0);let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::BlackBlood)).unwrap().id;
    let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
    let mut r=boss_chest();r.state.deck.truncate(3);choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));
    r.state.room_loop.as_mut().unwrap().screen.choices[0].price=Some(1);let id=r.state.pending.as_ref().unwrap().selection.candidates[0].id;
    let before=r.clone();assert!(r.step(&GameAction::Select(id)).is_err());assert!(r==before);
}
