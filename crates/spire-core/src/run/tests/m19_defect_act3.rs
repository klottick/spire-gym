//! Supplied owner diagnostics. These do not establish normal-start completion.
use super::super::*;
use crate::{combat::{Outcome,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId}};
use super::m17_defect_act1::{choose,diagnostic_room,cards,settle_lament};
fn profile()->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses=[vec!["Hexaghost","SlimeBoss"],vec!["Automaton","Collector"],vec!["AwakenedOne","DonuAndDeca"]];p
}
fn run()->Run{Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile(),RunRng::new([7;32],Partition::Test,0),RoomScenario::M19DefectAct3).unwrap()}
fn advance(r:&mut Run){
    diagnostic_room(r,MapNodeKind::Boss);settle_lament(r);
    choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::ConfirmSkip);
}
fn act3()->Run{
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    // Supplied Lament counter isolates the real transition/settlement owners.
    advance(&mut r);r.state.relic_counters.insert(RelicId::NeowsLament,3);advance(&mut r);r
}
#[test]
fn m19_defect_act3_canonical_prefix_and_admission(){
    let r=run();let d=r.decision().unwrap();assert_eq!((d.player.hp,d.player.max_hp,d.player.gold),(75,75,99));
    assert_eq!(d.player.relics,vec![RelicId::CrackedCore]);assert_eq!(d.player.potions,vec![None;3]);assert_eq!(d.player.keys,[false;3]);assert!(r.public_map().is_none());assert_eq!(d.objects.len(),10);
    for (id,n) in [(CardId::StrikeBlue,4),(CardId::DefendBlue,4),(CardId::Zap,1),(CardId::Dualcast,1)]{assert_eq!(d.objects.iter().filter(|c|c.id==id).count(),n);}
    assert_eq!((M19_DEFECT_ACT3_CARDS.len(),M19_DEFECT_ACT3_RELICS.len(),M19_DEFECT_ACT3_POTIONS.len()),(30,19,5));
    for &id in M19_DEFECT_ACT3_CARDS{
        let mut r=run();let i=mutations::obtain_card(&mut r.state,id,0).unwrap();
        if id==CardId::Parasite{assert!(mutations::upgrade_card(&mut r.state,i).is_err());let before=r.state.max_hp;mutations::remove_card(&mut r.state,i).unwrap();assert_eq!(r.state.max_hp,before-3);let saved=r.clone();assert!(mutations::remove_card(&mut r.state,i).is_err());assert_eq!(r,saved);}else{mutations::upgrade_card(&mut r.state,i).unwrap();}
    }
    for id in [CardId::Inflame,CardId::Leap,CardId::Burn,CardId::Wound]{let mut r=run();let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,0).is_err());assert_eq!(r,before);}
    for (character,ascension,long) in [(Character::Silent,0,false),(Character::Ironclad,0,false),(Character::Defect,1,false),(Character::Defect,0,true)]{let mut p=profile();p.neow_eligible=long;assert_eq!(Run::new_room_scenario(RunConfig{character,ascension},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M19DefectAct3),Err(RunError::Unsupported("M19 requires Defect A0 short Neow")));}
}
#[test]
fn m19_defect_act3_matched_m18_prefix_keeps_complete_owners(){
    let mut old=Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile(),RunRng::new([7;32],Partition::Test,0),RoomScenario::M18DefectAct2).unwrap();let mut new=run();
    let equal=|old:&Run,new:&Run|{let mut tagged=old.clone();tagged.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M19DefectAct3;assert_eq!(&tagged,new);};equal(&old,&new);
    for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::NeowLament);}equal(&old,&new);
    for act in [1,2]{
        for kind in [MapNodeKind::Monster,MapNodeKind::Boss]{
            for r in [&mut old,&mut new]{r.state.relic_counters.insert(RelicId::NeowsLament,3);diagnostic_room(r,kind);settle_lament(r);}equal(&old,&new);
            for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::CardReward);let id=cards(r)[0];choose(r,RoomChoiceKind::Card{id,upgrades:0});}equal(&old,&new);
            for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::Proceed);}equal(&old,&new);
            if kind==MapNodeKind::Boss{
                for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::Proceed);}equal(&old,&new);
                for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::ConfirmSkip);}
                if act==1{equal(&old,&new);}else{
                    assert_eq!(old.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3Ready);
                    let mut normalized=old.clone();normalized.state.room="Map";let room=normalized.state.room_loop.as_mut().unwrap();room.scenario=RoomScenario::M19DefectAct3;room.act3=Some(Default::default());room.screen.kind=RoomViewKind::Map;assert_eq!(normalized,new);
                }
            }
        }
    }
}
#[test]
fn m19_defect_act3_full_visit_rotation_and_stock(){
    use CardId::*;let rows=[[BallLightning,ColdSnap,Coolheaded],[Barrage,CompileDriver,Darkness],[Fusion,Defragment,GeneticAlgorithm],[Claw,SteamBarrier,Streamline],[Storm,Recursion,BiasedCognition]];
    for visit in 0..10u64{
        let mut r=act3();r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().combat_visits=visit;r.state.relic_counters.insert(RelicId::NeowsLament,3);diagnostic_room(&mut r,MapNodeKind::Monster);settle_lament(&mut r);
        let potion=[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][visit as usize%3];let token=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Potion(potion)).unwrap().id;
        r.state.potions=vec![Some(potion);3];let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);r.step(&GameAction::DiscardPotion{slot:0}).unwrap();choose(&mut r,RoomChoiceKind::Potion(potion));let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
        choose(&mut r,RoomChoiceKind::CardReward);assert_eq!(cards(&r),rows[visit as usize%5]);
    }
    let sequence=[RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::DataDisk,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfMarbles,RelicId::BronzeScales];
    for (i,relic) in sequence.iter().enumerate(){assert_eq!(rooms::m19_content::elite_relic(i,&[]),Ok((*relic,i+1)));}
    assert_eq!(rooms::m19_content::elite_relic(0,&sequence[..3]),Ok((RelicId::DataDisk,4)));
    assert_eq!(rooms::m19_content::elite_relic(0,&sequence),Err(RunError::Unsupported("captured Defect Act3 elite relic sequence exhausted")));
    for (visit,row,relic,potion) in [(0,[Capacitor,Consume,Loop],RelicId::GoldPlatedCables,PotionId::FocusPotion),(1,[Bullseye,StaticDischarge,Heatsinks],RelicId::EmotionChip,PotionId::PotionOfCapacity)]{
        let mut r=act3();r.state.gold=500;r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().shop_visits=visit;diagnostic_room(&mut r,MapNodeKind::Shop);assert_eq!(cards(&r),row);
        for c in r.decision().unwrap().room.unwrap().choices{match c.kind{RoomChoiceKind::Card{..}=>assert_eq!(c.price,Some(75)),RoomChoiceKind::Relic(id)=>{assert_eq!(id,relic);assert_eq!(c.price,Some(150));},RoomChoiceKind::Potion(id)=>{assert_eq!(id,potion);assert_eq!(c.price,Some(50));},_=>()}}
    }
    let mut r=act3();r.state.relic_counters.insert(RelicId::NeowsLament,3);diagnostic_room(&mut r,MapNodeKind::Elite);settle_lament(&mut r);assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor,1);choose(&mut r,RoomChoiceKind::Proceed);assert!(!r.state.relics.contains(&RelicId::Lantern));
}
#[test]
fn m19_defect_act3_real_transition_and_named_setup_pool(){
    let r=act3();assert_eq!((r.state.act,r.state.floor,r.state.hp),(3,34,r.state.max_hp));assert_eq!(r.public_map().unwrap().boss,Some(EncounterId::TimeEater));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    use EncounterId::*;let regular=[OrbWalker,ThreeShapes,SpireGrowth,Transient,FourShapes,Maw,SphereAndTwoShapes,JawWormHorde,ThreeDarklings,WrithingMass];
    let mut r=r;let mut selected=Vec::new();
    for _ in 0..24{let map=r.state.map.as_ref().unwrap();let target=map.nodes.iter().find(|n|n.kind==MapNodeKind::Monster).unwrap().id;let captured=rooms::act3::capture_room(&r.state,target).unwrap().encounter.unwrap();assert!(regular.contains(&captured));if !selected.contains(&captured){selected.push(captured);}r.state.map.as_mut().unwrap().regular_encounter_history.push(captured);}
    assert!(regular.iter().all(|id|selected.contains(id)));
    for id in regular.into_iter().chain([GiantHead,Nemesis,Reptomancer,AwakenedOne,TimeEater,DonuAndDeca]){assert!(crate::combat::enemies::A0_ENCOUNTER_SETUPS.iter().any(|(eid,_,_)|*eid==id));}
    assert!(r.state.map.as_ref().unwrap().nodes.iter().any(|n|n.floor==43&&n.kind==MapNodeKind::Treasure));assert!(r.state.map.as_ref().unwrap().nodes.iter().any(|n|n.floor==49&&n.kind==MapNodeKind::Rest));assert!(r.state.map.as_ref().unwrap().nodes.iter().any(|n|n.floor==50&&n.kind==MapNodeKind::Boss));
}
#[test]
fn m19_defect_act3_events_and_treasure_keep_instance_custody(){
    use crate::content::{EventId,CardType};
    let mut r=act3();let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}
    let a=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();let b=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();r.state.deck[0].misc=17;r.state.deck[1].misc=29;
    diagnostic_room(&mut r,MapNodeKind::Question);let view=r.decision().unwrap().room.unwrap();assert_eq!(view.kind,RoomViewKind::Event(EventId::Falling));assert!(view.choices.iter().filter(|c|matches!(c.kind,RoomChoiceKind::FallingLose{instance:None,..})).all(|c|c.status==RoomChoiceStatus::Disabled));
    let row=view.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Skill,instance:Some(_)})).unwrap();let instance=match row.kind{RoomChoiceKind::FallingLose{instance:Some(i),..}=>i,_=>unreachable!()};let twin=r.state.deck.iter().find(|c|c.instance!=instance).unwrap().clone();assert!([a,b].contains(&instance));r.step(&GameAction::Choose(row.id)).unwrap();assert_eq!(r.state.deck,vec![twin]);let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,before);
    for gold in [0,99]{for mask in [false,true]{let mut r=act3();r.state.gold=gold;if mask{crate::items::acquire_relic(&mut r.state,RelicId::RedMask).unwrap();}r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=1;diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(EventId::TombOfLordRedMask));choose(&mut r,if mask{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold});assert_eq!(r.state.gold,if mask{gold+222}else{0});assert_eq!(r.state.relics.iter().filter(|id|**id==RelicId::RedMask).count(),1);}}
    let mut r=act3();for id in [RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial]{crate::items::acquire_relic(&mut r.state,id).unwrap();}diagnostic_room(&mut r,MapNodeKind::Treasure);choose(&mut r,RoomChoiceKind::OpenChest);let v=r.decision().unwrap().room.unwrap();assert_eq!(v.choices.iter().filter(|c|matches!(c.kind,RoomChoiceKind::Relic(_))&&c.status==RoomChoiceStatus::Disabled).count(),3);choose(&mut r,RoomChoiceKind::Sapphire);assert!(r.state.keys[1]);assert!(r.decision().unwrap().room.unwrap().choices.iter().all(|c|!matches!(c.kind,RoomChoiceKind::Relic(_))||c.status!=RoomChoiceStatus::Offered));
}
#[test]
fn m19_defect_act3_boss_ending_is_not_loot_or_heart(){
    for keys in [false,true]{
        let mut r=act3();r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);r.state.keys=[keys;3];r.state.act4_enabled=keys;
        diagnostic_room(&mut r,MapNodeKind::Boss);let rng=r.state.rng.clone();let gold=r.state.gold;let deck=r.state.deck.clone();super::m9_boss::hit(&mut r,0,456);assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));r.settle_combat().unwrap();assert_eq!(r.state.rng,rng);assert_eq!(r.state.gold,gold);assert_eq!(r.state.deck,deck);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3BossComplete);assert!(r.state.room_loop.as_ref().unwrap().active_rewards.is_none());
        let token=r.decision().unwrap().room.unwrap().choices[0].id;choose(&mut r,RoomChoiceKind::Proceed);let saved=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,saved);
        for stage in 1..=3{assert_eq!(choose(&mut r,RoomChoiceKind::Act3Continue{stage}).reward,0);}
        if keys{assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act4Ready);assert!(r.decision().unwrap().candidates.is_empty());assert!(r.state.terminal.is_none());}else{assert_eq!(choose(&mut r,RoomChoiceKind::Act3Continue{stage:4}).terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(r.state.history.events.iter().filter(|e|matches!(e,CommittedEvent::Ended(RunTerminal::Act3NoKeys))).count(),1);}
        assert_eq!((r.state.act,r.state.floor),(3,50));assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::ActAdvanced{from:3,..})));
    }
}
#[test]
fn m19_defect_act3_exhausted_stock_retains_earned_combat_and_first_fault(){
    let mut r=act3();r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().elite_relic_cursor=12;r.state.relic_counters.insert(RelicId::NeowsLament,3);diagnostic_room(&mut r,MapNodeKind::Elite);
    let visits=r.state.combat_progress;let error=r.step(&GameAction::Combat(crate::combat::CombatAction::EndTurn)).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.combat_progress,visits);assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor,12);assert!(r.state.active_combat.is_some());assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));let before=r.clone();assert_eq!(r.step(&GameAction::Combat(crate::combat::CombatAction::EndTurn)),Err(error));assert_eq!(r,before);
}
#[test]
fn m19_defect_act3_ending_preflight_and_accepted_source_fault_are_distinct(){
    let mut r=act3();r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);diagnostic_room(&mut r,MapNodeKind::Boss);super::m9_boss::hit(&mut r,0,456);r.settle_combat().unwrap();
    let mut preflight=r.clone();preflight.next_choice=u64::MAX;let id=preflight.decision().unwrap().room.unwrap().choices[0].id;let before=preflight.clone();assert!(preflight.step(&GameAction::Choose(id)).is_err());assert_eq!(preflight,before);
    choose(&mut r,RoomChoiceKind::Proceed);let screen=r.decision().unwrap().room.unwrap();let id=screen.choices[0].id;r.state.map.as_mut().unwrap().current=None;let error=r.step(&GameAction::Choose(id)).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.terminal,None);assert_eq!(r.state.room_loop.as_ref().unwrap().screen,screen);let before=r.clone();assert_eq!(r.step(&GameAction::Choose(id)),Err(error));assert_eq!(r,before);
}
#[test]
fn m19_defect_act3_gain_and_misc_survive_fault_death_and_later_entry(){
    use crate::combat::{CombatAction,Actor,Effect,EffectKind,Source,DamagePayload};use crate::combat::tests::m19_defect_interactions as support;
    let mut r=act3();r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}
    let learned=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();let twin=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();
    r.enter_owned(EncounterId::WrithingMass,CombatOrigin::Training{kind:MapNodeKind::Monster}).unwrap();r.step(&GameAction::Combat(CombatAction::PlayCard{instance:learned,target:None})).unwrap();assert_eq!(r.decision().unwrap().combat.unwrap().player.block,1);assert_eq!(r.state.deck.iter().find(|c|c.instance==learned).unwrap().misc,3);assert_eq!(r.state.deck.iter().find(|c|c.instance==twin).unwrap().misc,1);
    support::implant_checkpoint(r.state.combat.as_mut().unwrap());r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1);assert_eq!(r.state.active_combat.as_ref().unwrap().applied_gain_ordinal,1);
    let mut fault=r.clone();support::arm_later_dispatch_fault(fault.state.combat.as_mut().unwrap());let error=fault.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));let before=fault.clone();assert_eq!(fault.step(&GameAction::Combat(CombatAction::EndTurn)),Err(error));assert_eq!(fault,before);assert_eq!(fault.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1);assert_eq!(fault.state.deck.iter().find(|c|c.instance==learned).unwrap().misc,3);
    let mut dead=r.clone();let hp=dead.decision().unwrap().player.hp;dead.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp as i32))}],None).unwrap();dead.settle_combat().unwrap();assert_eq!(dead.state.terminal,Some(RunTerminal::Death));assert_eq!(dead.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1);assert_eq!(dead.state.deck.iter().find(|c|c.instance==learned).unwrap().misc,3);
    super::m9_boss::hit(&mut r,0,999);r.settle_combat().unwrap();r.enter_owned(EncounterId::OrbWalker,CombatOrigin::Training{kind:MapNodeKind::Monster}).unwrap();let d=r.decision().unwrap();assert!(d.combat.unwrap().hand.iter().any(|c|c.instance.0==learned&&c.misc==3));assert_eq!(r.state.deck.iter().find(|c|c.instance==twin).unwrap().misc,1);assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1);
}
#[test]
fn m19_defect_act3_room_acquired_items_use_the_same_orb_scheduler(){
    use crate::combat::CombatAction;
    let mut r=act3();r.state.gold=1000;r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().elite_relic_cursor=3;r.state.relic_counters.insert(RelicId::NeowsLament,3);diagnostic_room(&mut r,MapNodeKind::Elite);settle_lament(&mut r);choose(&mut r,RoomChoiceKind::Relic(RelicId::DataDisk));choose(&mut r,RoomChoiceKind::Proceed);
    for (relic,potion) in [(RelicId::GoldPlatedCables,PotionId::FocusPotion),(RelicId::EmotionChip,PotionId::PotionOfCapacity)]{diagnostic_room(&mut r,MapNodeKind::Shop);choose(&mut r,RoomChoiceKind::Relic(relic));choose(&mut r,RoomChoiceKind::Potion(potion));choose(&mut r,RoomChoiceKind::Leave);}
    let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}let a=mutations::obtain_card(&mut r.state,CardId::Loop,0).unwrap();let b=mutations::obtain_card(&mut r.state,CardId::StaticDischarge,0).unwrap();r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);diagnostic_room(&mut r,MapNodeKind::Monster);
    for slot in [0,1]{r.step(&GameAction::Combat(CombatAction::UsePotion{slot,target:None})).unwrap();}for instance in [a,b]{r.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();}
    let mut blocked=r.clone();crate::combat::tests::m18_custody_support::acquired_item_scheduler(blocked.state.combat.as_mut().unwrap(),true);crate::combat::tests::m18_custody_support::acquired_item_scheduler(r.state.combat.as_mut().unwrap(),false);
}
#[test]
fn m19_defect_act3_falling_last_instance_and_accepted_mutation_fault(){
    use crate::content::CardType;
    let prepared=||{let mut r=act3();let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}r};
    let mut r=prepared();mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();diagnostic_room(&mut r,MapNodeKind::Question);let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{instance:Some(_),..})).unwrap();assert_eq!(r.step(&GameAction::Choose(row.id)).unwrap().terminal,Some(RunTerminal::EmptyDeck));assert!(r.state.deck.is_empty());let saved=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,saved);
    let mut r=prepared();let a=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();let b=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();r.state.deck[0].misc=17;r.state.deck[1].misc=29;diagnostic_room(&mut r,MapNodeKind::Question);let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:CardType::Skill,instance:Some(_)})).unwrap();let removed=match row.kind{RoomChoiceKind::FallingLose{instance:Some(i),..}=>i,_=>unreachable!()};assert!([a,b].contains(&removed));let twin=r.state.deck.iter().find(|c|c.instance!=removed).unwrap().clone();
    // Stage the accepted child exactly between reservation and callback. A later
    // result-publication failure must retain its one actual removal.
    r.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap().accepted=Some((row.id,row.kind));r.next_choice=u64::MAX;let error=rooms::events::act3::drive(&mut r).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.deck,vec![twin]);assert!(r.state.room_loop.as_ref().unwrap().act3_event.as_ref().unwrap().mutation_complete);let saved=r.clone();assert_eq!(rooms::events::act3::drive(&mut r),Err(error));assert_eq!(r,saved);
}
