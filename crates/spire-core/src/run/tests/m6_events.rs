use crate::{content::{CardId,EventId},run::*};
use super::rooms::choose;

fn event(id:EventId)->Run{
    // Supplied-room diagnostic only; normal Act2 admission remains Task10-owned.
    let mut r=super::rooms::run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});r.state.act=2;r.state.floor=20;
    let map=r.state.map.as_mut().unwrap();map.act=2;map.current=Some(NodeId(10));for node in &mut map.nodes{node.floor+=17;}
    let screen=super::super::rooms::events::screen(&mut r.next_choice,id).unwrap();r.state.room_loop.as_mut().unwrap().screen=screen;r
}
fn row(r:&Run,kind:RoomChoiceKind)->ChoiceId{r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==kind).expect("actual event option must be published").id}

#[test]
fn m6_joust_prepares_cost_and_outcome_before_commit(){
    for bet in [RoomChoiceKind::JoustMurderer,RoomChoiceKind::JoustOwner]{
        let mut r=event(EventId::Joust);r.state.gold=51;let id=row(&r,bet.clone());let before=r.state.rng.clone();
        assert!(!r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Leave));
        r.step(&GameAction::Choose(id)).unwrap();assert_eq!(r.state.gold,1);assert_eq!(r.state.rng,before,"payment stores the bet; actual source winner is only drawn on Continue");
        let paid=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==paid);
        let mut expected=r.state.rng.clone();let ticket=expected.bounded_scoped(RunRngScope::Act2Joust,&[2,10,0],100);let win=if bet==RoomChoiceKind::JoustOwner{ticket<30}else{ticket>=30};
        choose(&mut r,RoomChoiceKind::JoustContinue);assert_eq!(r.state.gold,1,"winner revelation is not the result-button payout");assert_eq!(r.state.rng,expected);
        choose(&mut r,RoomChoiceKind::JoustResult{owner_wins:ticket<30});assert_eq!(r.state.gold,1+if win{if bet==RoomChoiceKind::JoustOwner{250}else{100}}else{0});assert_eq!(r.state.rng,expected);
        let result=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==result);choose(&mut r,RoomChoiceKind::Leave);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    }
}

#[test]
fn m6_joust_wrong_price_overflow_stale_and_ui_draw_no_rng(){
    let mut r=event(EventId::Joust);r.state.gold=51;let id=row(&r,RoomChoiceKind::JoustOwner);let before=r.clone();let _=r.decision().unwrap();assert!(r.step(&GameAction::Choose(ChoiceId(u64::MAX))).is_err());assert!(r==before);
    r.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().price=Some(49);let wrong=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==wrong);
    r.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==id).unwrap().price=Some(50);r.state.gold=u32::MAX;let overflow=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==overflow);
    r.state.gold=51;r.next_choice=u64::MAX;let exhausted=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==exhausted);
}

#[test]
fn m6_ancient_writing_mutates_only_eligible_objects_once(){
    let mut r=event(EventId::AncientWriting);r.state.deck=[(CardId::Strike,0),(CardId::Defend,0),(CardId::Bash,0),(CardId::TwinStrike,0),(CardId::Strike,1)].into_iter().enumerate().map(|(i,(id,upgrades))|CardInstance{ misc:0,instance:i as u64+1,id,upgrades}).collect();let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();let rng=r.state.rng.clone();let id=row(&r,RoomChoiceKind::AncientSimplicity);
    r.step(&GameAction::Choose(id)).unwrap();assert_eq!(r.state.deck.iter().map(|c|c.upgrades).collect::<Vec<_>>(),vec![1,1,0,0,1]);assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),ids);assert_eq!(r.state.rng,rng);let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
}

#[test]
fn m6_ancient_elegance_captures_exact_one_and_checks_empty_only_on_callback(){
    let mut r=event(EventId::AncientWriting);r.state.deck=vec![CardInstance{ misc:0,instance:1,id:CardId::Strike,upgrades:0}];let purge=r.state.room_loop.as_ref().unwrap().purge_cost;let id=row(&r,RoomChoiceKind::AncientElegance);r.step(&GameAction::Choose(id)).unwrap();assert_eq!(r.state.deck.len(),1);assert_eq!(r.state.terminal,None);
    let selected=r.state.pending.as_ref().unwrap().selection.candidates[0].id;let before=r.clone();assert!(r.step(&GameAction::Cancel).is_err());assert!(r==before);let t=r.step(&GameAction::Select(selected)).unwrap();assert_eq!(t.terminal,Some(RunTerminal::EmptyDeck));assert_eq!(t.reward,-1);assert!(r.state.deck.is_empty());assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,purge);let settled=r.clone();assert!(r.step(&GameAction::Select(selected)).is_err());assert!(r==settled);
    let mut empty=event(EventId::AncientWriting);empty.state.deck.clear();choose(&mut empty,RoomChoiceKind::AncientElegance);assert!(empty.state.pending.is_none());assert!(empty.state.fault.is_none());assert_eq!(empty.state.terminal,Some(RunTerminal::EmptyDeck));
}

#[test]
fn m6_ancient_selection_rejects_stale_parent_price_and_identity(){
    let mut r=event(EventId::AncientWriting);let choice=row(&r,RoomChoiceKind::AncientElegance);r.step(&GameAction::Choose(choice)).unwrap();let selected=r.state.pending.as_ref().unwrap().selection.candidates[0].id;
    r.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==choice).unwrap().price=Some(50);let before=r.clone();assert!(r.step(&GameAction::Select(selected)).is_err());assert!(r==before);
    r.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.id==choice).unwrap().price=None;r.state.deck[0].upgrades=1;let before=r.clone();assert!(r.step(&GameAction::Select(selected)).is_err());assert!(r==before);
}

#[test]
fn m6_joust_ticket29_30_share_winner_for_both_bets_and_gold50_is_eligible(){
    for (seed,owner_wins) in [(100_u64,true),(12,false)]{for (bet,want) in [(RoomChoiceKind::JoustOwner,if owner_wins{250}else{0}),(RoomChoiceKind::JoustMurderer,if owner_wins{0}else{100})]{
        let mut r=event(EventId::Joust);r.state.gold=50;let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());r.state.rng=RunRng::new(root,crate::combat::rng::Partition::Test,0);assert!(super::super::rooms::events::joust_eligible(&r.state));
        choose(&mut r,bet);assert_eq!(r.state.gold,0);let rng=r.state.rng.clone();choose(&mut r,RoomChoiceKind::JoustContinue);assert_ne!(r.state.rng,rng);assert_eq!(r.state.gold,0);assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::JoustResult{owner_wins}));
        let rng=r.state.rng.clone();choose(&mut r,RoomChoiceKind::JoustResult{owner_wins});assert_eq!(r.state.gold,want);assert_eq!(r.state.rng,rng);
    }}
    let mut poor=event(EventId::Joust);poor.state.gold=49;assert!(!super::super::rooms::events::joust_eligible(&poor.state));let before=poor.clone();let id=row(&poor,RoomChoiceKind::JoustOwner);assert!(poor.step(&GameAction::Choose(id)).is_err());assert!(poor==before);
}

#[test]
fn m6_joust_accepted_payout_invariant_fault_keeps_payment_and_winner(){
    let mut r=event(EventId::Joust);r.state.gold=51;choose(&mut r,RoomChoiceKind::JoustOwner);choose(&mut r,RoomChoiceKind::JoustContinue);let winner=r.state.room_loop.as_ref().unwrap().joust.as_ref().unwrap().winner;let rng=r.state.rng.clone();
    // At the real coordinator boundary after result acceptance, corrupt only the
    // retained source balance. There is no fictional fallible on-win callback.
    r.state.room_loop.as_mut().unwrap().joust.as_mut().unwrap().payout_accepted=true;r.state.gold=2;
    let error=super::super::rooms::events::drive_joust_payout(&mut r).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!((r.state.gold,r.state.terminal),(2,None));assert_eq!(r.state.rng,rng);assert_eq!(r.state.room_loop.as_ref().unwrap().joust.as_ref().unwrap().winner,winner);
    let before=r.clone();assert_eq!(r.step(&GameAction::Choose(ChoiceId(0))),Err(error));assert!(r==before);
}

#[test]
fn m6_ancient_upgrade_removal_feed_native_combat_and_free_removal_keeps_purge_price(){
    use crate::combat::{Combat,CombatAction,TargetId,PlayerState,CardInstanceId,permanent_live_card,enemies::{self,SetupContext}};
    let mut upgraded=event(EventId::AncientWriting);upgraded.state.deck=vec![CardInstance{ misc:0,instance:1,id:CardId::Strike,upgrades:0},CardInstance{ misc:0,instance:2,id:CardId::Defend,upgrades:0}];choose(&mut upgraded,RoomChoiceKind::AncientSimplicity);choose(&mut upgraded,RoomChoiceKind::Leave);
    let mut entry=enemies::setup(crate::content::EncounterId::SphericGuardian,2,0,&mut upgraded.state.rng,SetupContext{
        player:PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},player_powers:vec![],character:crate::content::Character::Ironclad,
        cards:upgraded.state.deck.iter().map(|c|permanent_live_card(c.id,CardInstanceId(c.instance),c.upgrades)).collect(),opening_draw:2,encounter_visit:0}).unwrap();
    let mut c=Combat::enter(entry.clone()).unwrap();c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();assert_eq!(c.public_full().unwrap().enemies[0].block,31,"real upgraded Strike deals9 against Sphere's40Block");
    let mut removed=event(EventId::AncientWriting);removed.state.room_loop.as_mut().unwrap().purge_cost=100;choose(&mut removed,RoomChoiceKind::AncientElegance);let action=removed.decision().unwrap().candidates.into_iter().find(|c|c.object.as_ref().is_some_and(|o|o.instance==1)).unwrap().action;removed.step(&action).unwrap();choose(&mut removed,RoomChoiceKind::Leave);assert!(!removed.state.deck.iter().any(|c|c.instance==1));
    entry.cards=removed.state.deck.iter().map(|c|permanent_live_card(c.id,CardInstanceId(c.instance),c.upgrades)).collect();entry.opening_draw=5;let mut c=Combat::enter(entry).unwrap();let snapshot=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).is_err());assert_eq!(c.snapshot(),snapshot);
    removed.state.gold=151;let mut next=removed.next_choice;let screen=super::super::rooms::shop::screen(&removed,&mut next).unwrap();removed.next_choice=next;removed.state.room_loop.as_mut().unwrap().screen=screen;
    choose(&mut removed,RoomChoiceKind::Purge);let action=removed.decision().unwrap().candidates[0].action.clone();removed.step(&action).unwrap();assert_eq!((removed.state.gold,removed.state.room_loop.as_ref().unwrap().purge_cost),(51,125));
}
