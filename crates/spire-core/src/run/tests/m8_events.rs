use crate::{content::{CardId,CardType,EventId,RelicId,PotionId},run::*};
use super::rooms::choose;
use super::super::rooms::events::act3;

fn base()->Run{
    let mut r=super::rooms::run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});r.state.act=3;r.state.floor=36;let map=r.state.map.as_mut().unwrap();map.act=3;map.current=Some(NodeId(10));r
}
fn install(r:&mut Run,event:EventId){let mut next=r.next_choice;let p=act3::prepare(r,event,NodeId(10),0,&mut next).unwrap();r.state.rng=p.rng;r.next_choice=next;let room=r.state.room_loop.as_mut().unwrap();room.screen=p.screen;room.act3_event=Some(p.frame);}
fn cards(r:&mut Run,kinds:&[CardId]){r.state.deck=kinds.iter().enumerate().map(|(i,id)|CardInstance{ misc:0,id:*id,instance:i as u64+1,upgrades:0}).collect();}
fn offered(r:&Run)->Vec<RoomChoiceView>{r.decision().unwrap().room.unwrap().choices}

#[test]
fn m8_falling_captures_one_per_type_once(){
    let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Strike,CardId::Defend,CardId::Inflame]);let before=r.state.rng.clone();install(&mut r,EventId::Falling);assert_ne!(r.state.rng,before);let rng=r.state.rng.clone();let rows=offered(&r);assert_eq!(rows.iter().map(|c|match c.kind{RoomChoiceKind::FallingLose{card_type,..}=>card_type,_=>panic!("fixed type choice expected")}).collect::<Vec<_>>(),[CardType::Skill,CardType::Power,CardType::Attack]);for _ in 0..3{assert_eq!(offered(&r),rows);assert_eq!(r.state.rng,rng);}
    let attack=rows.iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Attack,..})).unwrap();let RoomChoiceKind::FallingLose{instance:Some(id),..}=attack.kind else{panic!("captured identity missing")};assert!([1,2].contains(&id));let purge=r.state.room_loop.as_ref().unwrap().purge_cost;r.step(&GameAction::Choose(attack.id)).unwrap();assert!(!r.state.deck.iter().any(|c|c.instance==id));assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Strike).count(),1);assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,purge);assert_eq!(r.state.rng,rng);let frozen=r.clone();assert!(r.step(&GameAction::Choose(attack.id)).is_err());assert!(r==frozen);choose(&mut r,RoomChoiceKind::Leave);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

#[test]
fn m8_falling_missing_types_and_last_card(){
    for only in [Some(CardId::Strike),Some(CardId::Inflame),Some(CardId::Parasite),None]{let mut r=base();cards(&mut r,&only.into_iter().collect::<Vec<_>>());let hp=(r.state.hp,r.state.max_hp);install(&mut r,EventId::Falling);let rows=offered(&r);
        if matches!(only,Some(CardId::Strike|CardId::Inflame)){assert_eq!(rows.iter().filter(|c|c.status==RoomChoiceStatus::Disabled).count(),2);let row=rows.iter().find(|c|c.status==RoomChoiceStatus::Offered).unwrap();let t=r.step(&GameAction::Choose(row.id)).unwrap();assert!(r.state.deck.is_empty());assert_eq!((t.terminal,t.reward),(Some(RunTerminal::EmptyDeck),-1));}
        else{assert_eq!(rows.len(),1);assert_eq!(rows[0].kind,RoomChoiceKind::Leave);let t=r.step(&GameAction::Choose(rows[0].id)).unwrap();assert_eq!((r.state.hp,r.state.max_hp),hp);assert_eq!(t.terminal,if only.is_none(){Some(RunTerminal::EmptyDeck)}else{None});assert_eq!(r.state.deck.len(),usize::from(only.is_some()));}
    }
}

#[test]
fn m8_tomb_all_gold_includes_zero(){
    for owned in [false,true]{for gold in [0,99,1234]{let mut r=base();r.state.gold=gold;r.state.potions=vec![Some(PotionId::FirePotion);3];if owned{r.state.relics.push(RelicId::RedMask);}install(&mut r,EventId::TombOfLordRedMask);let rng=r.state.rng.clone();let kind=if owned{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold};let row=offered(&r).into_iter().find(|c|c.kind==kind).unwrap();r.step(&GameAction::Choose(row.id)).unwrap();assert_eq!(r.state.gold,if owned{gold+222}else{0});assert_eq!(r.state.relics.iter().filter(|id|**id==RelicId::RedMask).count(),1);assert_eq!(r.state.rng,rng);let frozen=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==frozen);choose(&mut r,RoomChoiceKind::Leave);}}
    let mut r=base();install(&mut r,EventId::TombOfLordRedMask);choose(&mut r,RoomChoiceKind::Leave);assert_eq!(r.state.gold,99);assert!(!r.state.relics.contains(&RelicId::RedMask));
}

#[test]
fn m8_red_mask_weak_once_per_entry(){
    use crate::combat::*;
    for encounter in [crate::content::EncounterId::DonuAndDeca,crate::content::EncounterId::Reptomancer]{
        let mut items=Some(crate::items::CombatItemContext::new(vec![RelicId::RedMask],vec![]));let e=enemies::setup(encounter,3,0,&mut RunRng::new([41;32],rng::Partition::Test,0),enemies::SetupContext{player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},player_powers:vec![],character:crate::content::Character::Ironclad,cards:vec![permanent_live_card(CardId::Defend,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0}).unwrap();let mut c=Combat::enter_with_item_context(e,&mut items).unwrap();
        for enemy in &c.public_full().unwrap().enemies{let weak=enemy.powers.iter().find(|p|p.id.0=="weak").map_or(0,|p|p.amount);assert_eq!(weak,if encounter==crate::content::EncounterId::DonuAndDeca{0}else{1});}
        if encounter==crate::content::EncounterId::DonuAndDeca{assert!(c.public_full().unwrap().enemies.iter().all(|e|e.powers.iter().any(|p|p.id.0=="artifact"&&p.amount==1)));}
        c.step_full(&CombatAction::EndTurn).unwrap();if encounter==crate::content::EncounterId::Reptomancer{let view=c.public_full().unwrap();assert_eq!(view.enemies.len(),4);assert!(view.enemies.iter().all(|e|e.powers.iter().all(|p|p.id.0!="weak")));}
    }
}

#[test]
fn m8_event_preflight_and_accepted_fault(){
    let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Defend]);r.next_choice=u64::MAX;let before=r.clone();let mut next=r.next_choice;assert!(act3::prepare(&r,EventId::Falling,NodeId(10),0,&mut next).is_err());assert_eq!(next,u64::MAX);assert!(r==before);
    let mut r=base();r.state.relics.push(RelicId::RedMask);r.state.gold=u32::MAX;install(&mut r,EventId::TombOfLordRedMask);let row=offered(&r).into_iter().find(|c|c.kind==RoomChoiceKind::TombGold).unwrap();let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==before);
    let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Defend]);install(&mut r,EventId::Falling);let row=offered(&r).into_iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Attack,..})).unwrap();r.state.deck[0].upgrades=1;let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==before);
}

#[test]
fn m8_event_post_mutation_publication_fault_never_repeats_remove_or_grant(){
    for event in [EventId::Falling,EventId::TombOfLordRedMask]{
        let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Defend]);install(&mut r,event);let row=offered(&r).into_iter().find(|c|if event==EventId::Falling{matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Attack,..})}else{c.kind==RoomChoiceKind::TombOfferGold}).unwrap();
        // Drive the real coordinator's accepted, already-mutated boundary. The
        // only injected defect is exhaustion before result-screen publication.
        if event==EventId::Falling{super::super::mutations::remove_card(&mut r.state,1).unwrap();}else{r.state.gold=0;crate::items::acquire_relic(&mut r.state,RelicId::RedMask).unwrap();}
        let frame=r.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap();frame.accepted=Some((row.id,row.kind.clone()));frame.mutation_complete=true;let rng=r.state.rng.clone();r.next_choice=u64::MAX;
        let error=act3::drive(&mut r).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.terminal,None);assert_eq!(r.state.rng,rng);assert!(r.state.room_loop.as_ref().unwrap().act3_event.as_ref().unwrap().mutation_complete);
        if event==EventId::Falling{assert_eq!(r.state.deck.len(),1);assert_eq!(r.state.deck[0].instance,2);}else{assert_eq!(r.state.gold,0);assert_eq!(r.state.relics.iter().filter(|id|**id==RelicId::RedMask).count(),1);}
        let frozen=r.clone();assert_eq!(act3::drive(&mut r),Err(error.clone()));assert_eq!(r.step(&GameAction::Choose(row.id)),Err(error));assert!(r==frozen);
    }
}

#[test]
fn m8_event_accepted_grant_failure_keeps_paid_gold_and_source(){
    let mut r=base();install(&mut r,EventId::TombOfLordRedMask);let row=offered(&r).into_iter().find(|c|c.kind==RoomChoiceKind::TombOfferGold).unwrap();r.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap().accepted=Some((row.id,row.kind));
    r.state.max_hp=u32::MAX;let error=act3::drive(&mut r).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.gold,0);assert!(!r.state.relics.contains(&RelicId::RedMask));assert!(r.state.room_loop.as_ref().unwrap().act3_event.as_ref().unwrap().accepted.is_some());let frozen=r.clone();assert_eq!(r.step(&GameAction::Choose(row.id)),Err(error));assert!(r==frozen);
}

#[test]
fn m8_event_disabled_fixed_choices_and_token_exhaustion_are_atomic(){
    let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Defend]);install(&mut r,EventId::Falling);let missing=offered(&r).into_iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Power,instance:None})).unwrap();let before=r.clone();assert!(r.step(&GameAction::Choose(missing.id)).is_err());assert!(r==before);
    let row=offered(&r).into_iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Attack,..})).unwrap();r.next_choice=u64::MAX;let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==before);
}

#[test]
fn m8_falling_uses_canonical_instances_and_rejects_duplicate_identity(){
    let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Strike,CardId::Defend]);r.state.rng=RunRng::new([0;32],crate::combat::rng::Partition::Test,0);let mut reordered=r.clone();reordered.state.deck.reverse();install(&mut r,EventId::Falling);install(&mut reordered,EventId::Falling);let rows=offered(&r);assert_eq!(offered(&reordered),rows);assert!(rows.iter().any(|c|c.kind==RoomChoiceKind::FallingLose{card_type:CardType::Attack,instance:Some(2)}),"fixed native ticket chooses the second duplicate kind, not an ID-deduplicated first object");
    let mut r=base();cards(&mut r,&[CardId::Strike,CardId::Defend]);r.state.deck[1].instance=1;let frozen=r.clone();let mut next=r.next_choice;assert!(act3::prepare(&r,EventId::Falling,NodeId(10),0,&mut next).is_err());assert!(r==frozen);assert_eq!(next,r.next_choice);
}
