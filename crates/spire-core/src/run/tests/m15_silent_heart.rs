use super::super::*;
#[path = "../../../examples/m15_support/mod.rs"]
#[allow(dead_code)]
mod frozen_m15;
use crate::content::{CardId,Character,RelicId};
fn normal_start()->Run{
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},profile,RunRng::new([61;32],crate::combat::rng::Partition::Test,0),RoomScenario::M15SilentHeart).unwrap()
}
#[test]
fn m15_normal_creation_and_complete_green_admission_use_canonical_resources(){
    let mut run=normal_start();assert_eq!((run.state.hp,run.state.max_hp,run.state.gold),(70,70,99));
    assert_eq!(run.state.relics,vec![RelicId::RingOfTheSnake]);assert_eq!(run.state.potions,vec![None;3]);assert_eq!(run.state.keys,[false;3]);assert!(run.state.act4_enabled);assert_eq!(run.state.deck.len(),12);
    assert_eq!(run.state.deck.iter().filter(|card|card.id==CardId::StrikeGreen).count(),5);
    assert_eq!(run.state.deck.iter().filter(|card|card.id==CardId::DefendGreen).count(),5);
    assert_eq!(run.decision().unwrap().content_profile,Some(RoomScenario::M15SilentHeart));
    let before=run.state.rng.clone();assert_eq!(run.decision().unwrap(),run.decision().unwrap());assert_eq!(run.state.rng,before);
    assert!(!rooms::is_m10(&run.state));assert!(rooms::uses_act4(&run.state));
    for rarity in [crate::content::CardRarity::Basic,crate::content::CardRarity::Common,crate::content::CardRarity::Uncommon,crate::content::CardRarity::Rare]{for id in rooms::m15_content::green_candidates(rarity){
        assert!(mutations::supported_in(&run.state,*id));mutations::obtain_card(&mut run.state,*id,0).unwrap();mutations::obtain_card(&mut run.state,*id,1).unwrap();
    }}
    assert!(!mutations::supported_in(&run.state,CardId::Shiv));assert!(mutations::supported_in(&run.state,CardId::Parasite));
}

fn mapped()->Run{let mut run=normal_start();super::rooms::choose(&mut run,RoomChoiceKind::NeowLament);run}
fn install_screen(run:&mut Run,kind:RoomViewKind,choices:Vec<RoomChoiceKind>){
    let mut next=run.next_choice;run.state.room_loop.as_mut().unwrap().screen=rooms::view(&mut next,kind,choices.into_iter().map(|choice|(choice,None)).collect()).unwrap();run.next_choice=next;
}
#[test]
fn m15_chest_capture_consumes_stock_on_open_even_when_skipped_or_sapphire_taken(){
    let mut run=mapped();
    for (index,relic) in rooms::m15_content::M15_RELIC_STOCK.iter().enumerate(){
        install_screen(&mut run,RoomViewKind::Treasure,vec![RoomChoiceKind::OpenChest,RoomChoiceKind::Proceed]);
        super::rooms::choose(&mut run,RoomChoiceKind::OpenChest);
        let screen=run.decision().unwrap().room.unwrap();
        assert_eq!(screen.choices.iter().filter_map(|row|if let RoomChoiceKind::Relic(id)=row.kind{Some(id)}else{None}).collect::<Vec<_>>(),vec![*relic]);
        assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,index+1);
        if index==0 {super::rooms::choose(&mut run,RoomChoiceKind::Sapphire);assert!(run.state.keys[1]);}
        let before=run.state.rng.clone();assert_eq!(run.decision().unwrap(),run.decision().unwrap());assert_eq!(run.state.rng,before);
    }
    install_screen(&mut run,RoomViewKind::Treasure,vec![RoomChoiceKind::OpenChest]);
    let before=run.clone();assert!(matches!(run.step(&GameAction::Choose(run.state.room_loop.as_ref().unwrap().screen.choices[0].id)),Err(RunError::Unsupported("M15 normal relic stock exhausted"))));assert_eq!(run,before);
}
#[test]
fn m15_shop_captures_green_rarity_rows_and_exclusive_relic_once(){
    use crate::content::{CardRarity,Catalog,PotionId};
    let mut run=mapped();let mut next=run.next_choice;let mut rng=run.state.rng.clone();
    let screen=rooms::shop::screen_with_rng_at(&run,&mut next,&mut rng,(1,4,NodeId(123))).unwrap();
    let cards=screen.choices.iter().filter_map(|row|if let RoomChoiceKind::Card{id,..}=row.kind{Some((id,row.price))}else{None}).collect::<Vec<_>>();
    assert_eq!(cards.len(),3);for ((id,price),(rarity,cost)) in cards.iter().zip([(CardRarity::Common,50),(CardRarity::Uncommon,75),(CardRarity::Rare,150)]){assert_eq!(Catalog::standard().card(*id).unwrap().rarity,rarity);assert_eq!(*price,Some(cost));}
    assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Relic(RelicId::TwistedFunnel)&&row.price==Some(150)));
    assert_eq!(screen.choices.iter().filter_map(|row|if let RoomChoiceKind::Potion(id)=row.kind{Some(id)}else{None}).collect::<Vec<_>>(),vec![PotionId::FirePotion]);
    assert_ne!(rng,run.state.rng);run.next_choice=next;run.state.rng=rng;run.state.room_loop.as_mut().unwrap().screen=screen;
    let before=run.clone();assert_eq!(run.decision().unwrap(),run.decision().unwrap());assert_eq!(run,before);
}
#[test]
fn m15_boss_chest_has_five_real_options_and_starter_replacement_guard(){
    let mut run=mapped();run.state.floor=17;run.state.map.as_mut().unwrap().current=Some(run.state.map.as_ref().unwrap().nodes[0].id);
    run.state.room_loop.as_mut().unwrap().boss=super::m7_rewards::chest().state.room_loop.unwrap().boss;
    install_screen(&mut run,RoomViewKind::BossTreasure,vec![RoomChoiceKind::OpenChest]);
    super::rooms::choose(&mut run,RoomChoiceKind::OpenChest);
    let screen=run.decision().unwrap().room.unwrap();
    assert_eq!(screen.choices.iter().filter_map(|row|if let RoomChoiceKind::Relic(id)=row.kind{Some(id)}else{None}).collect::<Vec<_>>(),rooms::m15_content::M15_BOSS_RELICS);
    super::rooms::choose(&mut run,RoomChoiceKind::Relic(RelicId::RingOfTheSerpent));
    assert!(run.state.relics.contains(&RelicId::RingOfTheSerpent));assert!(!run.state.relics.contains(&RelicId::RingOfTheSnake));assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,0);
}

// Supplied late-entry diagnostics: real Silent resources and shared late map/
// progress, followed by native damage/settlement. These are not normal episodes.
fn supplied_act3_boss(mask:u8)->Run{
    let (late,node)=super::m9_boss::source(crate::content::EncounterId::TimeEater);
    let mut run=normal_start();run.state.map=late.state.map;run.state.room_loop=late.state.room_loop;run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M15SilentHeart;
    run.state.act=3;run.state.floor=49;run.state.room="Map";
    for (bit,key) in [Key::Ruby,Key::Sapphire,Key::Emerald].into_iter().enumerate(){if mask&(1<<bit)!=0{let event=mutations::acquire_key(&mut run.state,key).unwrap();reward::commit(&mut run.state,&[event]).unwrap();}}
    run.step(&GameAction::Move(node)).unwrap();assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().battle.m15);
    super::m9_boss::hit(&mut run,0,456);run.settle_combat().unwrap();
    super::rooms::choose(&mut run,RoomChoiceKind::Proceed);for stage in 1..=3{super::rooms::choose(&mut run,RoomChoiceKind::Act3Continue{stage});}run
}
#[test]
fn m15_actual_boss_record_requires_all_keys_for_shared_act4_route(){
    for mask in 0..8{
        let mut run=supplied_act3_boss(mask);
        assert_eq!(run.decision().unwrap().room.unwrap().choices[0].kind,RoomChoiceKind::Act3Continue{stage:4});
        let transition=super::rooms::choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
        if mask==7 {assert_eq!((run.state.act,run.state.floor,transition.terminal),(4,50,None));assert_eq!(run.public_map().unwrap().nodes.iter().map(|node|node.floor).collect::<Vec<_>>(),vec![51,52,53,54]);}
        else {assert_eq!(transition.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!((run.state.act,run.state.floor),(3,50));}
    }
}
#[test]
fn m15_outside_brew_menu_is_pure_and_real_use_calls_shared_factory(){
    let mut run=mapped();run.state.potions[0]=Some(crate::content::PotionId::EntropicBrew);
    let action=GameAction::UsePotion{slot:0,target:None};let before=run.clone();
    assert!(run.decision().unwrap().candidates.iter().any(|candidate|candidate.action==action));assert_eq!(run.decision().unwrap(),run.decision().unwrap());assert_eq!(run,before);
    run.step(&action).unwrap();assert!(run.state.potions.iter().all(Option::is_some));assert_ne!(run.state.rng,before.state.rng);
}
#[test]
fn m15_smoke_real_map_combat_settles_to_proceed_without_claims(){
    let mut run=mapped();run.state.potions[0]=Some(crate::content::PotionId::SmokeBomb);
    let node=run.decision().unwrap().candidates.iter().find_map(|candidate|if let GameAction::Move(node)=candidate.action{Some(node)}else{None}).unwrap();
    run.step(&GameAction::Move(node)).unwrap();let prior=run.state.combat_progress.settled;
    let transition=run.choose_combat(crate::combat::CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(transition.terminal,None);assert_eq!(run.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Escaped);assert_eq!(run.state.combat_progress.settled,prior+1);
    assert_eq!(run.decision().unwrap().room.unwrap().choices.iter().map(|row|row.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);
    super::rooms::choose(&mut run,RoomChoiceKind::Proceed);assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

fn supplied_shop(act:u8)->(Run,NodeId){
    let mut run=mapped();
    if act==2{let late=super::m7_rooms::continuing();run.state.map=late.state.map;run.state.room_loop=late.state.room_loop;run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M15SilentHeart;run.state.act=2;run.state.floor=17;}
    if act==3{let (late,_)=super::m9_boss::source(crate::content::EncounterId::TimeEater);run.state.map=late.state.map;run.state.room_loop=late.state.room_loop;run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M15SilentHeart;run.state.act=3;run.state.floor=34;run.state.map.as_mut().unwrap().current=None;}
    if act==4{run=supplied_act3_boss(7);super::rooms::choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();let choice=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|matches!(row.kind,RoomChoiceKind::Rest{..})).unwrap();run.step(&GameAction::Choose(choice.id)).unwrap();let shop=run.public_map().unwrap().nodes[1].id;return (run,shop)}
    let node=run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node|node.y==0).unwrap();node.kind=MapNodeKind::Shop;node.hidden_kind=None;let id=node.id;(run,id)
}
#[test]
fn m15_shop_rng_commits_once_and_choice_exhaustion_keeps_prepared_rng_detached(){
    for act in 1..=4{
        let (mut run,node)=supplied_shop(act);let floor=run.state.map.as_ref().unwrap().nodes.iter().find(|entry|entry.id==node).unwrap().floor;
        let mut next=run.next_choice;let mut expected_rng=run.state.rng.clone();let expected=rooms::shop::screen_with_rng_at(&run,&mut next,&mut expected_rng,(act,floor,node)).unwrap();
        let mut overflow=run.clone();overflow.next_choice=u64::MAX-2;let before=overflow.clone();assert!(overflow.step(&GameAction::Move(node)).is_err());assert_eq!(overflow.state.rng,before.state.rng,"act {act} must not commit a detached capture on choice exhaustion");
        run.step(&GameAction::Move(node)).unwrap();assert_eq!(run.state.rng,expected_rng,"act {act}");assert_eq!(run.state.room_loop.as_ref().unwrap().screen,expected);
        let before=run.clone();assert_eq!(run.decision().unwrap(),run.decision().unwrap());assert_eq!(run.public_map().unwrap(),run.public_map().unwrap());assert_eq!(run,before);assert_eq!(run.step(&GameAction::Move(node)),Err(RunError::InvalidAction));assert_eq!(run,before);
        let purchase=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.price==Some(50)&&matches!(row.kind,RoomChoiceKind::Card{..})).unwrap();let gold=run.state.gold;run.step(&GameAction::Choose(purchase.id)).unwrap();assert_eq!(run.state.gold,gold-50);assert_eq!(run.state.rng,expected_rng);
        super::rooms::choose(&mut run,RoomChoiceKind::Leave);assert_eq!(run.state.rng,expected_rng);
    }
}
#[test]
fn m15_earned_keys_real_burning_victory_and_reward_child_keep_source_custody(){
    use crate::content::PotionId;
    let mut run=mapped();let node=run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node|node.y==0).unwrap();node.kind=MapNodeKind::Elite;node.burning=true;let node=node.id;
    run.state.potions=vec![Some(PotionId::FirePotion);3];run.step(&GameAction::Move(node)).unwrap();
    for slot in 0..run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.len(){super::m9_boss::hit(&mut run,slot,10_000);}
    run.settle_combat().unwrap();let before=run.state.reward_ledger.total;let parent=run.decision().unwrap().room.unwrap();
    assert!(parent.choices.iter().any(|row|row.kind==RoomChoiceKind::Potion(PotionId::FirePotion)&&row.status==RoomChoiceStatus::Disabled));
    super::rooms::choose(&mut run,RoomChoiceKind::CardReward);let child=run.decision().unwrap().room.unwrap();assert_eq!(child.kind,RoomViewKind::CardReward);
    let ids=child.choices.iter().filter_map(|row|if let RoomChoiceKind::Card{id,..}=row.kind{Some(id)}else{None}).collect::<Vec<_>>();assert_eq!(ids.len(),3);assert_ne!(ids[1],ids[2]);
    super::rooms::choose(&mut run,RoomChoiceKind::Skip);assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Rewards);
    let emerald=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Emerald).unwrap().id;
    let transition=run.step(&GameAction::Choose(emerald)).unwrap();assert_eq!(transition.reward,5);assert!(run.state.keys[2]);assert_eq!(run.state.reward_ledger.total,before+5);
    let after=run.clone();assert_eq!(run.step(&GameAction::Choose(emerald)),Err(RunError::InvalidAction));assert_eq!(run,after);
    super::rooms::choose(&mut run,RoomChoiceKind::Proceed);
    install_screen(&mut run,RoomViewKind::Campfire,vec![RoomChoiceKind::Recall]);let transition=super::rooms::choose(&mut run,RoomChoiceKind::Recall);assert_eq!(transition.reward,5);assert!(run.state.keys[0]);
    install_screen(&mut run,RoomViewKind::Treasure,vec![RoomChoiceKind::OpenChest]);super::rooms::choose(&mut run,RoomChoiceKind::OpenChest);let transition=super::rooms::choose(&mut run,RoomChoiceKind::Sapphire);assert_eq!(transition.reward,5);assert_eq!(run.state.keys,[true;3]);
}
#[test]
fn m15_actual_shield_spear_rewards_and_heart_proceed_settle_once(){
    use crate::{content::EncounterId,combat::{CombatAction,Outcome}};
    let mut run=supplied_act3_boss(7);super::rooms::choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();super::rooms::choose(&mut run,RoomChoiceKind::Smith);
    let card=run.state.pending.as_ref().unwrap().selection.candidates[0].clone();run.step(&GameAction::Select(card.id)).unwrap();assert_eq!(run.state.deck.iter().find(|entry|entry.instance==card.card.instance).unwrap().upgrades,1);
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();super::rooms::choose(&mut run,RoomChoiceKind::Leave);
    let elite=run.public_map().unwrap().nodes[2].id;let heart=run.public_map().unwrap().nodes[3].id;run.step(&GameAction::Move(elite)).unwrap();
    let enemies=run.state.combat.as_ref().unwrap().public_full().unwrap().enemies;assert_eq!(enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),vec![110,160]);
    super::m9_boss::hit(&mut run,0,10_000);assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,None);assert_eq!(run.step(&GameAction::Move(heart)),Err(RunError::InvalidAction));
    super::m9_boss::hit(&mut run,1,10_000);run.settle_combat().unwrap();assert_eq!(run.state.last_combat.as_ref().unwrap().encounter,EncounterId::ShieldAndSpear);
    super::rooms::choose(&mut run,RoomChoiceKind::CardReward);let frozen=run.clone();assert_eq!(run.step(&GameAction::Move(heart)),Err(RunError::InvalidAction));assert_eq!(run,frozen);super::rooms::choose(&mut run,RoomChoiceKind::Skip);super::rooms::choose(&mut run,RoomChoiceKind::Proceed);
    // Diagnostic survival resources, never normal-start evidence.
    run.state.max_hp=999;run.state.hp=999;run.step(&GameAction::Move(heart)).unwrap();assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().enemies[0].hp,750);assert_eq!(frozen_m15::m15_stop_outcome(&run.decision().unwrap(),None),Ok(None));
    for _ in 0..4{super::m9_boss::hit(&mut run,0,1000);if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome==Some(Outcome::Victory){break}run.choose_combat(CombatAction::EndTurn).unwrap();}
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));run.settle_combat().unwrap();assert_eq!(run.state.terminal,None);assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::HeartVictory);
    let pending=run.decision().unwrap();assert_eq!(frozen_m15::m15_stop_outcome(&pending,None),Ok(None));let proceed=pending.room.as_ref().unwrap().choices[0].id;
    let before=run.clone();let mut empty_budget=frozen_m15::Caller::new_m15(0).unwrap();assert_eq!(frozen_m15::rollout_m15(&mut run,&mut empty_budget,frozen_m15::key_action),Ok(frozen_m15::M15CallerOutcome::BudgetExhausted));assert_eq!(run,before);assert!(empty_budget.trace.is_empty());
    let mut exact_budget=frozen_m15::Caller::new_m15(1).unwrap();assert_eq!(frozen_m15::rollout_m15(&mut run,&mut exact_budget,frozen_m15::key_action),Ok(frozen_m15::M15CallerOutcome::HeartVictory));assert_eq!(exact_budget.accepted(),1);let accepted=&exact_budget.trace[0];assert_eq!(accepted.action,GameAction::Choose(proceed));assert_eq!(accepted.decision,pending);
    assert_eq!(accepted.transition.reward,110);assert_eq!(accepted.transition.terminal,Some(RunTerminal::HeartVictory));assert_eq!(run.state.floor,55);
    assert_eq!(frozen_m15::m15_stop_outcome(&run.decision().unwrap(),Some(accepted)),Ok(Some(frozen_m15::M15CallerOutcome::HeartVictory)));assert!(frozen_m15::m15_stop_outcome(&run.decision().unwrap(),None).is_err());
    let settled=run.clone();assert!(run.step(&GameAction::Choose(proceed)).is_err());assert_eq!(run,settled);
}

#[test]
fn m15_all_green_permanent_instances_use_existing_upgrade_remove_and_event_eligibility(){
    let base=mapped();
    for id in rooms::m15_content::M15_GREEN_PERMANENT_CARDS{
        let mut state=base.state.clone();let instance=mutations::obtain_card(&mut state,*id,0).unwrap();let card=state.deck.iter().find(|card|card.instance==instance).unwrap().clone();assert!(mutations::can_upgrade(&state,&card),"{id:?}");
        mutations::upgrade_card(&mut state,instance).unwrap();assert_eq!(state.deck.iter().find(|card|card.instance==instance).unwrap().upgrades,1);assert_eq!(mutations::remove_card(&mut state,instance).unwrap().id,*id);
    }
    let mut shrine=base.clone();let node=shrine.state.map.as_ref().unwrap().nodes[0].id;shrine.state.map.as_mut().unwrap().current=Some(node);
    shrine.state.deck=vec![CardInstance{instance:41,id:CardId::WraithForm,upgrades:0},CardInstance{instance:42,id:CardId::WraithForm,upgrades:0}];shrine.state.next_instance=43;
    let mut next=shrine.next_choice;shrine.state.room_loop.as_mut().unwrap().screen=rooms::events::screen(&mut next,crate::content::EventId::UpgradeShrine).unwrap();shrine.next_choice=next;
    super::rooms::choose(&mut shrine,RoomChoiceKind::ShrineUpgrade);let selected=shrine.state.pending.as_ref().unwrap().selection.candidates.iter().find(|candidate|candidate.card.instance==42).unwrap().id;shrine.step(&GameAction::Select(selected)).unwrap();assert_eq!(shrine.state.deck.iter().map(|card|card.upgrades).collect::<Vec<_>>(),vec![0,1]);
}
#[test]
fn m15_exhausted_elite_stock_retains_completed_combat_inventory_and_first_fault(){
    let mut run=mapped();run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=19;
    let node=run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node|node.y==0).unwrap();node.kind=MapNodeKind::Elite;let node=node.id;
    run.step(&GameAction::Move(node)).unwrap();for slot in 0..run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.len(){super::m9_boss::hit(&mut run,slot,10_000);}
    let before=run.clone();let error=run.settle_combat().unwrap_err();let RunError::Faulted(ref fault)=error else{panic!("exhaustion must retain the accepted owner")};assert_eq!(fault.checkpoint,"run.combat.rewards");
    assert_eq!(run.state.combat_progress,before.state.combat_progress);assert_eq!(run.state.rng,before.state.rng);assert!(run.state.combat.is_some()&&run.state.active_combat.is_some());assert!(run.state.relics.is_empty());assert!(run.state.terminal.is_none());
    let saved=run.clone();assert_eq!(run.settle_combat(),Err(error));assert_eq!(run,saved);
}
#[test]
fn m15_supplied_act4_partial_source_and_entry_exhaustion_are_not_terminal_success(){
    let mut run=supplied_act3_boss(7);let fourth=run.decision().unwrap().room.unwrap().choices[0].id;
    let mut changed=run.clone();changed.state.last_combat=None;let before=changed.clone();assert_eq!(changed.step(&GameAction::Choose(fourth)),Err(RunError::InvalidAction));assert_eq!(changed,before);
    super::rooms::choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});let accepted=run.clone();assert_eq!(run.step(&GameAction::Choose(fourth)),Err(RunError::InvalidAction));assert_eq!(run,accepted);
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();let choice=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|matches!(row.kind,RoomChoiceKind::Rest{..})).unwrap();run.step(&GameAction::Choose(choice.id)).unwrap();
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();super::rooms::choose(&mut run,RoomChoiceKind::Leave);
    run.state.combat_progress.entered=u64::MAX;let prior=run.state.reward_ledger.total;let elite=run.public_map().unwrap().nodes[2].id;let error=run.step(&GameAction::Move(elite)).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(run.state.floor,53);assert_eq!(run.state.reward_ledger.total,prior+10);assert_eq!(run.state.terminal,None);assert!(run.state.combat.is_none());
    let saved=run.clone();assert_eq!(run.step(&GameAction::Move(elite)),Err(error));assert_eq!(run,saved);
}

#[test]
fn m15_real_heart_death_keeps_death_receipt_and_never_publishes_victory_proceed(){
    let mut run=supplied_act3_boss(7);super::rooms::choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    super::m10_settlement::complete_elite_rewards_for_supplied_heart_entry(&mut run);
    run.state.hp=1;let heart=run.public_map().unwrap().nodes[3].id;run.step(&GameAction::Move(heart)).unwrap();let prior=run.state.combat_progress.settled;
    let mut death=None;for _ in 0..4{let transition=run.choose_combat(crate::combat::CombatAction::EndTurn).unwrap();if transition.terminal.is_some(){death=Some(transition);break}}
    let death=death.unwrap();assert_eq!(death.terminal,Some(RunTerminal::Death));assert_eq!(death.reward,-1);assert_eq!(run.state.combat_progress.settled,prior+1);assert_eq!(run.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Death);
    assert_ne!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::HeartVictory);assert!(run.decision().unwrap().candidates.is_empty());let saved=run.clone();assert!(run.choose_combat(crate::combat::CombatAction::EndTurn).is_err());assert_eq!(run,saved);
}
#[test]
fn m15_actual_boss_rare_children_resume_real_boss_treasure_and_skip_confirmation(){
    use crate::content::{CardRarity,Catalog,EncounterId};
    for act in 1..=2{
        let mut run=mapped();let node;
        if act==2{let (late,target)=super::m7_rooms::boss_source(EncounterId::Champ);run.state.map=late.state.map;run.state.room_loop=late.state.room_loop;run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M15SilentHeart;run.state.act=2;run.state.floor=32;node=target;}
        else{node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Boss).unwrap().id;super::m7_rooms::before(&mut run,node);}
        run.step(&GameAction::Move(node)).unwrap();for slot in 0..run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.len(){super::m9_boss::hit(&mut run,slot,10_000);}run.settle_combat().unwrap();
        super::rooms::choose(&mut run,RoomChoiceKind::CardReward);let cards=run.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|row|if let RoomChoiceKind::Card{id,..}=row.kind{Some(id)}else{None}).collect::<Vec<_>>();assert_eq!(cards.len(),3);assert_eq!(cards.iter().collect::<std::collections::BTreeSet<_>>().len(),3);assert!(cards.iter().all(|id|Catalog::standard().card(*id).unwrap().rarity==CardRarity::Rare));
        super::rooms::choose(&mut run,RoomChoiceKind::Skip);super::rooms::choose(&mut run,RoomChoiceKind::Proceed);assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::BossTreasure);
        super::rooms::choose(&mut run,RoomChoiceKind::OpenChest);super::rooms::choose(&mut run,RoomChoiceKind::Skip);assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::ProceedWithoutRelic);super::rooms::choose(&mut run,RoomChoiceKind::CancelSkip);assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::BossRelicChoice);
    }
}

mod supplied_act4;
