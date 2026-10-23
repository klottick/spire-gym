use super::super::*;
use crate::combat::{self,*,rng::Partition};
use crate::content::{CardId,Character,EncounterId,RelicId};
use crate::run::CardInstance;

// Explicit supplied encounter/Run diagnostic, not normal-start M8 acceptance.
fn fixture(exhausted:bool,capacity:bool)->Run{
    let cards=if capacity{(1..=999).map(|instance|CardInstance{instance,id:CardId::Defend,upgrades:0}).collect()}else{vec![CardInstance{instance:1,id:CardId::Defend,upgrades:0},CardInstance{instance:2,id:CardId::Strike,upgrades:0}]};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},AuthoredRunStart{hp:88,max_hp:88,gold:99,deck:cards,relics:vec![RelicId::BurningBlood],potions:vec![None;3],relic_counters:Default::default()},RunRng::new([41;32],Partition::Test,0)).unwrap();
    let mut e=combat::enemies::setup(EncounterId::WrithingMass,3,0,&mut RunRng::new([41;32],Partition::Test,0),combat::enemies::SetupContext{player:PlayerState{hp:88,max_hp:88,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},player_powers:vec![],character:Character::Ironclad,cards:run.state.deck.iter().map(|c|permanent_live_card(c.id,CardInstanceId(c.instance),c.upgrades)).collect(),opening_draw:1,encounter_visit:0}).unwrap();
    e.setup.enemies[0].intent=FullIntentId::from_name("writhing_parasite");if exhausted{let id=e.setup.enemies[0].ai_rng.id().clone();e.setup.enemies[0].ai_rng=combat::rng::StreamState::new(id,[0;32],1_u128<<64).unwrap();}
    let mut items=Some(crate::items::CombatItemContext::take_from_run(&mut run.state));let c=Combat::enter_with_item_context(e,&mut items).unwrap();assert!(items.is_none());
    run.state.combat=Some(c);run.state.active_combat=Some(ActiveCombat{visit:0,encounter:EncounterId::WrithingMass,origin:CombatOrigin::Authored,applied_gain_ordinal:0,acquired_instances:vec![]});run.state.combat_progress=CombatProgress{entered:1,settled:0};run
}

#[test]
fn m8_permanent_gain_sync_survives_fault_and_settlement(){
    let mut r=fixture(true,false);assert!(matches!(r.choose_combat(CombatAction::EndTurn),Err(RunError::Faulted(_))));
    assert_eq!(r.state.deck.last(),Some(&CardInstance{instance:3,id:CardId::Parasite,upgrades:0}));assert_eq!(r.state.next_instance,4);assert_eq!(r.state.combat.as_ref().unwrap().permanent_gains().len(),1);
    assert!(r.state.relics.is_empty()&&r.state.potions.is_empty());assert_eq!(r.state.combat.as_ref().unwrap().item_context().unwrap().relics(),&[RelicId::BurningBlood]);
    let active=r.state.active_combat.as_ref().unwrap();assert_eq!((active.applied_gain_ordinal,active.acquired_instances.as_slice()),(1,[3].as_slice()));
    let before=r.clone();r.sync_permanent_gains().unwrap();assert_eq!(r,before);assert!(r.choose_combat(CombatAction::EndTurn).is_err());assert_eq!(r,before);
    for death in [false,true]{
        let mut r=fixture(false,false);r.choose_combat(CombatAction::EndTurn).unwrap();assert_eq!(r.state.deck.last().unwrap().id,CardId::Parasite);
        let c=r.state.combat.as_mut().unwrap();let target=if death{Actor::Player}else{Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone())};
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();
        r.settle_combat().unwrap();assert!(r.state.combat.is_none());assert_eq!(r.state.deck.len(),3);assert_eq!(r.state.deck[2],CardInstance{instance:3,id:CardId::Parasite,upgrades:0});assert_eq!(r.state.combat_progress.settled,1);assert_eq!(r.state.terminal,if death{Some(RunTerminal::Death)}else{None});
        assert_eq!(r.state.relics,vec![RelicId::BurningBlood]);assert_eq!(r.state.potions,vec![None;3]);
    }
}

#[test]
fn m8_acquisition_capacity_and_identity_overflow_keep_journal_without_partial_allocation(){
    for capacity in [false,true]{
        let mut r=fixture(false,capacity);if !capacity{r.state.next_instance=u64::MAX;}let before=(r.state.deck.clone(),r.state.next_instance);
        assert!(matches!(r.choose_combat(CombatAction::EndTurn),Err(RunError::Faulted(_))));assert_eq!((r.state.deck.clone(),r.state.next_instance),before);assert_eq!(r.state.combat.as_ref().unwrap().permanent_gains().len(),1);
        let frozen=r.clone();assert!(r.sync_permanent_gains().is_err());assert_eq!(r,frozen);
    }
}

#[test]
fn m8_acquisition_census_exempts_only_exact_recorded_new_instances(){
    let mut r=fixture(false,false);r.choose_combat(CombatAction::EndTurn).unwrap();r.state.deck.push(CardInstance{instance:4,id:CardId::Strike,upgrades:0});
    let c=r.state.combat.as_mut().unwrap();let target=Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();
    assert!(matches!(r.settle_combat(),Err(RunError::Faulted(_))));assert!(r.state.combat.is_some());assert_eq!(r.state.combat_progress.settled,0);
}

#[test]
fn m8_acquisition_parasite_permanent_removal_loses_max_hp_once_and_cannot_upgrade(){
    let mut r=fixture(false,false);r.choose_combat(CombatAction::EndTurn).unwrap();let c=r.state.combat.as_mut().unwrap();let target=Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();r.settle_combat().unwrap();
    assert_eq!((r.state.hp,r.state.max_hp),(88,88));assert!(mutations::upgrade_card(&mut r.state,3).is_err());
    mutations::remove_card(&mut r.state,3).unwrap();assert_eq!((r.state.hp,r.state.max_hp),(85,85));let before=r.clone();assert!(mutations::remove_card(&mut r.state,3).is_err());assert_eq!(r,before);
}

#[test]
fn m8_acquisition_parasite_removal_clamps_only_excess_hp_and_floors_maximum_at_one(){
    for (hp,max_hp,want_hp,want_max) in [(40,88,40,85),(3,3,1,1),(1,1,1,1)]{
        let mut r=fixture(false,false);r.choose_combat(CombatAction::EndTurn).unwrap();let c=r.state.combat.as_mut().unwrap();let target=Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();r.settle_combat().unwrap();
        r.state.hp=hp;r.state.max_hp=max_hp;mutations::remove_card(&mut r.state,3).unwrap();assert_eq!((r.state.hp,r.state.max_hp),(want_hp,want_max));
    }
}

#[test]
fn m8_acquisition_numeric_id_overlap_with_combat_local_card_does_not_hide_an_origin(){
    let mut r=fixture(false,false);let c=r.state.combat.as_mut().unwrap();let owner=Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(owner),target:Some(Actor::Player),kind:EffectKind::CreateCard{id:CardId::Wound,destination:CardDestination::Draw,upgrades:0}}],None).unwrap();
    r.choose_combat(CombatAction::EndTurn).unwrap();assert_eq!(r.state.deck[2],CardInstance{instance:3,id:CardId::Parasite,upgrades:0});
    let c=r.state.combat.as_mut().unwrap();let view=c.public_full().unwrap();assert!(view.hand.iter().chain(&view.discard).any(|c|c.instance==CardInstanceId(3)&&c.id==CardId::Wound&&matches!(c.origin,CombatCardRef::CombatLocal(_))));
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(999))}],None).unwrap();r.settle_combat().unwrap();assert_eq!(r.state.deck.len(),3);assert_eq!(r.state.deck[2].id,CardId::Parasite);
}

#[test]
fn m8_acquisition_later_smith_excludes_curse_and_paid_purge_removes_the_same_instance(){
    let mut r=fixture(false,false);r.choose_combat(CombatAction::EndTurn).unwrap();let c=r.state.combat.as_mut().unwrap();let target=Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();r.settle_combat().unwrap();
    let parasite=r.state.deck[2].clone();
    // Supply valid room frames only, retaining this actual Run's acquired deck,
    // HP, gold, inventory and identity frontier. Not a normal-start route claim.
    for (node,choice) in [(38,RoomChoiceKind::Smith),(17,RoomChoiceKind::Purge)]{
        let frame=super::room_services::room(node);r.state.floor=frame.state.floor;r.state.room=frame.state.room;r.state.map=frame.state.map;r.state.room_loop=frame.state.room_loop;r.next_choice=frame.next_choice;r.candidates.clear();
        let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==choice).unwrap().id;r.step(&GameAction::Choose(id)).unwrap();
        let d=r.decision().unwrap();let candidate=d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.instance==parasite.instance));
        if choice==RoomChoiceKind::Smith{assert!(candidate.is_none());r.step(&GameAction::Cancel).unwrap();}
        else{let action=candidate.unwrap().action.clone();r.step(&action).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold),(85,85,24));assert!(!r.state.deck.contains(&parasite));assert_eq!(r.state.next_instance,4);let before=r.clone();assert!(r.step(&action).is_err());assert_eq!(r,before);}
    }
}
