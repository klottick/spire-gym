use super::super::*;
use crate::{content::{CardId,Character,RelicId},combat::rng::Partition};

pub(super) fn run()->Run {
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile,
        RunRng::new([7;32],Partition::Test,0),RoomScenario::M20DefectHeart).unwrap()
}

#[test]
fn m20_defect_content_canonical_and_opt_in_progression(){
    let mut r=run();let d=r.decision().unwrap();
    assert_eq!((d.player.hp,d.player.max_hp,d.player.gold),(75,75,99));
    assert_eq!(d.player.relics,vec![RelicId::CrackedCore]);assert_eq!(d.player.potions,vec![None;3]);
    assert_eq!(d.player.keys,[false;3]);assert!(r.public_map().is_none());assert_eq!(d.objects.len(),10);
    for (id,n) in [(CardId::StrikeBlue,4),(CardId::DefendBlue,4),(CardId::Zap,1),(CardId::Dualcast,1)] {
        assert_eq!(d.objects.iter().filter(|card|card.id==id).count(),n);
    }
    assert!(rooms::uses_defect_content(&r.state));assert!(rooms::uses_full_act3(&r.state));assert!(rooms::uses_act4(&r.state));
    super::m17_defect_act1::choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
    assert_eq!((r.state.hp,r.state.max_hp),(82,82));assert!(r.public_map().is_some());
    for scenario in [RoomScenario::M17DefectAct1,RoomScenario::M18DefectAct2,RoomScenario::M19DefectAct3] {
        let mut old=r.clone();old.state.room_loop.as_mut().unwrap().scenario=scenario;
        assert!(!rooms::uses_act4(&old.state));
    }
}

#[test]
fn m20_defect_content_complete_permanent_admission_and_finite_stock(){
    assert_eq!((M20_DEFECT_HEART_CARDS.len(),M20_DEFECT_HEART_RELICS.len(),M20_DEFECT_HEART_POTIONS.len()),(76,26,33));
    for id in crate::content::defect::M20_DEFECT_CARDS {
        let mut r=run();let instance=mutations::obtain_card(&mut r.state,*id,0).expect("every ordinary blue kind is a real permanent option");
        mutations::upgrade_card(&mut r.state,instance).unwrap();assert_eq!(r.state.deck.last().unwrap().upgrades,1);
        assert_eq!(mutations::remove_card(&mut r.state,instance).unwrap().id,*id);
    }
    for id in [CardId::SearingBlow,CardId::Inflame,CardId::Dazed,CardId::Discovery]{let mut r=run();let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,0).is_err());assert_eq!(r,before);}
    let mut r=run();let parasite=mutations::obtain_card(&mut r.state,CardId::Parasite,0).unwrap();let hp=r.state.max_hp;
    assert!(mutations::upgrade_card(&mut r.state,parasite).is_err());mutations::remove_card(&mut r.state,parasite).unwrap();assert_eq!(r.state.max_hp,hp-3);
    let stock=rooms::m20_content::NORMAL_RELIC_STOCK;
    for (index,id) in stock.iter().enumerate(){assert_eq!(rooms::m20_content::normal_relic(index,&[]),Ok((*id,index+1)));}
    assert_eq!(rooms::m20_content::normal_relic(0,&stock[..3]),Ok((RelicId::ToyOrnithopter,4)));
    assert_eq!(rooms::m20_content::normal_relic(0,stock),Err(RunError::Unsupported("M20 normal relic stock exhausted")));
}

fn mapped()->Run{let mut r=run();super::rooms::choose(&mut r,RoomChoiceKind::NeowLament);r}
fn install(r:&mut Run,kind:RoomViewKind,choices:Vec<RoomChoiceKind>){r.state.room_loop.as_mut().unwrap().screen=rooms::view(&mut r.next_choice,kind,choices.into_iter().map(|kind|(kind,None)).collect()).unwrap();}
#[test]
fn m20_defect_content_shop_captures_complete_rarities_mod11_potions_and_stale_rows(){
    use crate::content::{CardColor,CardRarity,Catalog};
    for visit in [0,1,5,10,11,12]{
        let mut r=mapped();r.state.gold=999;r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().shop_visits=visit;
        let mut next=r.next_choice;let mut rng=r.state.rng.clone();let screen=rooms::shop::screen_with_rng_at(&r,&mut next,&mut rng,(1,4,NodeId(123))).unwrap();
        let cards=screen.choices.iter().filter_map(|row|if let RoomChoiceKind::Card{id,..}=row.kind{Some((id,row.price))}else{None}).collect::<Vec<_>>();
        assert_eq!(cards.len(),3);for ((id,price),(rarity,cost)) in cards.iter().zip([(CardRarity::Common,50),(CardRarity::Uncommon,75),(CardRarity::Rare,150)]){let row=Catalog::standard().card(*id).unwrap();assert_eq!((row.color,row.rarity,*price),(CardColor::Blue,rarity,Some(cost)));}
        assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Relic(RelicId::RunicCapacitor)&&row.price==Some(150)));
        assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Potion(rooms::m20_content::POTION_CYCLE[visit%11])&&row.price==Some(50)));
        assert_ne!(rng,r.state.rng);r.next_choice=next;r.state.rng=rng;r.state.room_loop.as_mut().unwrap().screen=screen;
        let before=r.clone();assert_eq!(r.decision().unwrap(),r.decision().unwrap());assert_eq!(r,before,"publication cannot reroll");
        let id=r.decision().unwrap().room.unwrap().choices.iter().find(|row|row.kind==RoomChoiceKind::Card{id:cards[0].0,upgrades:0}).unwrap().id;
        r.step(&GameAction::Choose(id)).unwrap();let accepted=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert_eq!(r,accepted);
        assert_eq!(r.state.deck.last().unwrap().id,cards[0].0);assert_eq!(r.state.gold,949);
    }
}
#[test]
fn m20_defect_content_actual_reward_uses_unreduced_actlocal_mod11_and_retains_children(){
    use crate::content::{Catalog,CardRarity};
    for local in [0,1,5,10,11,12]{
        let mut r=mapped();r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().combat_visits=local;
        // Deliberately unrelated global count: cycle belongs to act-local source.
        r.state.combat_progress=CombatProgress{entered:30,settled:30};
        super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Monster);super::m17_defect_act1::settle_lament(&mut r);
        let d=r.decision().unwrap();assert!(d.room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Potion(rooms::m20_content::POTION_CYCLE[(local%11)as usize])));
        assert_eq!(r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().combat_visits,local+1);
        let capture=r.state.room_loop.as_ref().unwrap().active_rewards.clone().unwrap();let rng=r.state.rng.clone();
        super::rooms::choose(&mut r,RoomChoiceKind::CardReward);let cards=super::m17_defect_act1::cards(&r);assert_eq!(cards.len(),3);assert_ne!(cards[1],cards[2]);
        assert_eq!(cards.iter().map(|id|Catalog::standard().card(*id).unwrap().rarity).collect::<Vec<_>>(),vec![CardRarity::Common,CardRarity::Uncommon,CardRarity::Uncommon]);
        assert_eq!(r.state.rng,rng);super::rooms::choose(&mut r,RoomChoiceKind::Skip);assert_eq!(r.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap().cards,capture.cards);
    }
}
#[test]
fn m20_defect_content_shared_stock_capture_and_exhaustion_keep_exact_source_frontier(){
    let mut r=mapped();
    for (i,relic) in rooms::m20_content::NORMAL_RELIC_STOCK.iter().enumerate(){
        install(&mut r,RoomViewKind::Treasure,vec![RoomChoiceKind::OpenChest,RoomChoiceKind::Proceed]);super::rooms::choose(&mut r,RoomChoiceKind::OpenChest);
        assert_eq!(r.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,i+1);
        assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Relic(*relic)));
        if i==0{super::rooms::choose(&mut r,RoomChoiceKind::Sapphire);assert!(r.state.keys[1]);}
    }
    install(&mut r,RoomViewKind::Treasure,vec![RoomChoiceKind::OpenChest]);let id=r.state.room_loop.as_ref().unwrap().screen.choices[0].id;let before=r.clone();
    assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::Unsupported("M20 normal relic stock exhausted")));assert_eq!(r,before);
    let mut r=mapped();r.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=17;super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Elite);
    for i in 0..r.decision().unwrap().combat.unwrap().enemies.len(){super::m9_boss::hit(&mut r,i,10_000);}
    let before=r.clone();let error=r.settle_combat().unwrap_err();let RunError::Faulted(fault)=&error else{panic!("accepted source owns settlement failure")};assert_eq!(fault.checkpoint,"run.combat.rewards");assert!(matches!(&fault.kind,RunFaultKind::UnsupportedBoundary(message) if message.contains("M20 normal relic stock exhausted")));
    assert_eq!(r.state.combat_progress,before.state.combat_progress);assert_eq!(r.state.rng,before.state.rng);assert!(r.state.combat.is_some()&&r.state.active_combat.is_some());let frozen=r.clone();assert_eq!(r.settle_combat(),Err(error));assert_eq!(r,frozen);
}
fn actual_act3_source(mask:u8)->Run{
    use crate::content::EncounterId;
    let (late,node)=super::m9_boss::source(EncounterId::TimeEater);let mut r=run();r.state.map=late.state.map;r.state.room_loop=late.state.room_loop;r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M20DefectHeart;r.state.act=3;r.state.floor=49;r.state.room="Map";
    for (bit,key)in [Key::Ruby,Key::Sapphire,Key::Emerald].into_iter().enumerate(){if mask&(1<<bit)!=0{let event=mutations::acquire_key(&mut r.state,key).unwrap();reward::commit(&mut r.state,&[event]).unwrap();}}
    r.step(&GameAction::Move(node)).unwrap();assert!(r.state.combat.as_ref().unwrap().item_context().unwrap().m20_profile());super::m9_boss::hit(&mut r,0,456);r.settle_combat().unwrap();
    super::rooms::choose(&mut r,RoomChoiceKind::Proceed);for stage in 1..=3{super::rooms::choose(&mut r,RoomChoiceKind::Act3Continue{stage});}r
}
#[test]
fn m20_defect_content_all_eight_key_masks_real_source_and_door_are_owned_once(){
    for mask in 0..8{
        let mut r=actual_act3_source(mask);let id=r.decision().unwrap().room.unwrap().choices[0].id;let before=r.clone();
        let transition=super::rooms::choose(&mut r,RoomChoiceKind::Act3Continue{stage:4});
        if mask==7{assert_eq!((r.state.act,r.state.floor),(4,50));assert_eq!(transition.committed_events,vec![CommittedEvent::ActAdvanced{from:3,to:4}]);assert_eq!(r.public_map().unwrap().nodes.iter().map(|n|n.floor).collect::<Vec<_>>(),vec![51,52,53,54]);}
        else{assert_eq!(r.state.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(r.state.act,3);}
        let accepted=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert_eq!(r,accepted);
        let mut invalid=before;invalid.state.last_combat=None;let invalid_before=invalid.clone();let error=invalid.step(&GameAction::Choose(id)).unwrap_err();
        if mask==7{assert_eq!(invalid,invalid_before,"door preflight rejects before acceptance");}
        else{assert!(matches!(&error,RunError::Faulted(fault) if fault.checkpoint=="run.m20.act3.end"));let frozen=invalid.clone();assert_eq!(invalid.step(&GameAction::Choose(id)),Err(error));assert_eq!(invalid,frozen,"accepted ending source failure is retained");}
    }
}
#[test]
fn m20_defect_content_actual_act4_elite_children_and_heart_proceed_are_distinct(){
    use crate::{combat::{CombatAction,Outcome},content::EncounterId};
    let mut r=actual_act3_source(7);super::rooms::choose(&mut r,RoomChoiceKind::Act3Continue{stage:4});
    super::m10_settlement::complete_elite_rewards_for_supplied_heart_entry(&mut r);
    assert_eq!(r.state.last_combat.as_ref().unwrap().encounter,EncounterId::ShieldAndSpear);assert_eq!(r.state.last_combat.as_ref().unwrap().outcome,Outcome::Victory);
    r.state.hp=999;r.state.max_hp=999;let heart=r.public_map().unwrap().nodes[3].id;r.step(&GameAction::Move(heart)).unwrap();assert_eq!(r.decision().unwrap().combat.unwrap().enemies[0].hp,750);
    for _ in 0..3{super::m9_boss::hit(&mut r,0,1_000);if r.state.combat.as_ref().unwrap().public_full().unwrap().outcome==Some(Outcome::Victory){break}r.choose_combat(CombatAction::EndTurn).unwrap();}
    assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));r.settle_combat().unwrap();assert_eq!(r.state.terminal,None);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::HeartVictory);
    let id=r.decision().unwrap().room.unwrap().choices[0].id;let t=r.step(&GameAction::Choose(id)).unwrap();assert_eq!(t.reward,110);assert_eq!(t.terminal,Some(RunTerminal::HeartVictory));assert_eq!((r.state.floor,r.state.room),(55,"TrueVictoryRoom"));let frozen=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert_eq!(r,frozen);
}
#[test]
fn m20_defect_content_power_retirement_misc_and_parasite_survive_death_and_fault_once(){
    use crate::{combat::{Actor,CombatAction,DamagePayload,Effect,EffectKind,Source},content::EncounterId};
    use crate::combat::tests::m19_defect_interactions as support;
    let (late,_)=super::m9_boss::source(EncounterId::TimeEater);let mut r=run();r.state.act=3;r.state.floor=34;r.state.map=late.state.map;r.state.room_loop.as_mut().unwrap().act3=Some(Default::default());
    let instances=r.state.deck.iter().map(|card|card.instance).collect::<Vec<_>>();for n in instances{mutations::remove_card(&mut r.state,n).unwrap();}
    let ga=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();let repair=mutations::obtain_card(&mut r.state,CardId::SelfRepair,0).unwrap();let twin=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();
    r.enter_owned(EncounterId::WrithingMass,CombatOrigin::Training{kind:MapNodeKind::Monster}).unwrap();
    r.step(&GameAction::Combat(CombatAction::PlayCard{instance:ga,target:None})).unwrap();r.step(&GameAction::Combat(CombatAction::PlayCard{instance:repair,target:None})).unwrap();
    assert_eq!(r.state.deck.iter().find(|card|card.instance==ga).unwrap().misc,3);assert_eq!(r.state.deck.iter().find(|card|card.instance==twin).unwrap().misc,1);
    support::implant_checkpoint(r.state.combat.as_mut().unwrap());r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();assert_eq!(r.state.deck.iter().filter(|card|card.id==CardId::Parasite).count(),1);
    let mut fault=r.clone();support::arm_later_dispatch_fault(fault.state.combat.as_mut().unwrap());let error=fault.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));let frozen=fault.clone();assert_eq!(fault.step(&GameAction::Combat(CombatAction::EndTurn)),Err(error));assert_eq!(fault,frozen);assert_eq!(fault.state.deck.iter().filter(|card|card.id==CardId::Parasite).count(),1);
    let hp=r.decision().unwrap().player.hp;r.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp as i32))}],None).unwrap();r.settle_combat().unwrap();
    assert_eq!(r.state.terminal,Some(RunTerminal::Death));assert_eq!(r.state.hp,0,"Repair never resurrects the dead source");assert_eq!(r.state.deck.iter().find(|card|card.instance==ga).unwrap().misc,3);assert_eq!(r.state.deck.iter().find(|card|card.instance==repair).unwrap().upgrades,0);assert_eq!(r.state.deck.iter().filter(|card|card.id==CardId::Parasite).count(),1);
}
#[test]
fn m20_defect_content_stasis_temporary_upgrade_hologram_and_ga_keep_one_permanent_owner(){
    use crate::{combat::CombatAction,content::{EncounterId,PotionId}};
    use crate::combat::tests::m18_custody_support as support;
    let mut r=run();r.state.act=2;r.state.floor=17;r.state.room_loop.as_mut().unwrap().act2=Some(Default::default());
    let all=r.state.deck.iter().map(|card|card.instance).collect::<Vec<_>>();for n in all{mutations::remove_card(&mut r.state,n).unwrap();}
    let ga=mutations::obtain_card(&mut r.state,CardId::GeneticAlgorithm,0).unwrap();let hologram=mutations::obtain_card(&mut r.state,CardId::Hologram,0).unwrap();
    r.enter_owned(EncounterId::BronzeAutomaton,CombatOrigin::Training{kind:MapNodeKind::Boss}).unwrap();r.step(&GameAction::Combat(CombatAction::PlayCard{instance:ga,target:None})).unwrap();super::m9_boss::hit(&mut r,0,999);r.settle_combat().unwrap();assert_eq!(r.state.deck.iter().find(|card|card.instance==ga).unwrap().misc,3);
    crate::items::obtain_run_potion(&mut r.state,PotionId::BlessingOfTheForge).unwrap();r.enter_owned(EncounterId::BronzeAutomaton,CombatOrigin::Training{kind:MapNodeKind::Boss}).unwrap();
    r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap();support::capture_learned_genetic(r.state.combat.as_mut().unwrap(),ga);
    let held=r.decision().unwrap().combat.unwrap().stasis;assert_eq!((held[0].card.instance.0,held[0].card.misc,held[0].card.upgrades),(ga,3,1));assert_eq!(r.state.deck.iter().find(|card|card.instance==ga).unwrap().upgrades,0);
    support::return_learned_genetic(r.state.combat.as_mut().unwrap(),true);assert!(r.decision().unwrap().combat.unwrap().discard.iter().any(|card|card.instance.0==ga));
    r.step(&GameAction::Combat(CombatAction::PlayCard{instance:hologram,target:None})).unwrap();
    if r.decision().unwrap().combat.unwrap().selection.is_some(){r.step(&GameAction::Combat(CombatAction::SelectCard{instance:ga})).unwrap();}
    assert!(r.decision().unwrap().combat.unwrap().hand.iter().any(|card|card.instance.0==ga&&card.misc==3&&card.upgrades==1));
    r.step(&GameAction::Combat(CombatAction::PlayCard{instance:ga,target:None})).unwrap();let learned=r.state.deck.iter().find(|card|card.instance==ga).unwrap();assert_eq!((learned.misc,learned.upgrades),(6,0));assert_eq!(r.state.deck.iter().filter(|card|card.instance==ga).count(),1);assert_eq!(r.state.deck.iter().find(|card|card.instance==hologram).unwrap().upgrades,0);
}

#[test]
fn m20_defect_content_seven_captured_events_use_actual_blue_objects(){
    use crate::content::{EventId,CardType};
    // Supplied event-unit boundaries, not policy traversal or normal outcomes.
    fn event(mut r:Run,id:EventId,act:u8)->Run{
        r.state.act=act;r.state.floor=if act==3{36}else{20};let map=r.state.map.as_mut().unwrap();map.act=act;let node=map.nodes[0].id;map.current=Some(node);
        if act==3{let mut next=r.next_choice;let prepared=rooms::events::act3::prepare(&r,id,node,0,&mut next).unwrap();r.state.rng=prepared.rng;r.next_choice=next;let room=r.state.room_loop.as_mut().unwrap();room.screen=prepared.screen;room.act3_event=Some(prepared.frame);}
        else{let screen=rooms::events::screen(&mut r.next_choice,id).unwrap();r.state.room_loop.as_mut().unwrap().screen=screen;}r
    }
    fn select(r:&mut Run,instance:u64){let action=r.decision().unwrap().candidates.into_iter().find(|c|c.object.as_ref().is_some_and(|o|o.instance==instance)).expect("physical permanent choice").action;r.step(&action).unwrap();}
    let neow=mapped();assert!(neow.state.relics.contains(&RelicId::NeowsLament));assert_eq!((neow.state.hp,neow.state.max_hp),(75,75));
    let mut ancient=event(mapped(),EventId::AncientWriting,2);let extra=mutations::obtain_card(&mut ancient.state,CardId::BeamCell,0).unwrap();let identities=ancient.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();
    super::rooms::choose(&mut ancient,RoomChoiceKind::AncientSimplicity);assert_eq!(ancient.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),identities);
    for c in &ancient.state.deck{assert_eq!(c.upgrades,if matches!(c.id,CardId::StrikeBlue|CardId::DefendBlue){1}else{0});}assert_eq!(ancient.state.deck.iter().find(|c|c.instance==extra).unwrap().upgrades,0);
    let mut shrine=event(mapped(),EventId::UpgradeShrine,1);let instance=mutations::obtain_card(&mut shrine.state,CardId::Reboot,0).unwrap();super::rooms::choose(&mut shrine,RoomChoiceKind::ShrineUpgrade);select(&mut shrine,instance);assert_eq!(shrine.state.deck.iter().find(|c|c.instance==instance).unwrap().upgrades,1);
    let mut purifier=event(mapped(),EventId::Purifier,1);purifier.state.deck.clear();let instance=mutations::obtain_card(&mut purifier.state,CardId::Parasite,0).unwrap();let hp=purifier.state.max_hp;super::rooms::choose(&mut purifier,RoomChoiceKind::ShrineRemove);select(&mut purifier,instance);assert!(purifier.state.deck.is_empty());assert_eq!(purifier.state.max_hp,hp-3);assert_eq!(purifier.state.terminal,Some(RunTerminal::EmptyDeck));
    let mut joust=event(mapped(),EventId::Joust,2);let initial_rng=joust.state.rng.clone();super::rooms::choose(&mut joust,RoomChoiceKind::JoustMurderer);assert_eq!(joust.state.gold,49);assert_eq!(joust.state.rng,initial_rng);super::rooms::choose(&mut joust,RoomChoiceKind::JoustContinue);let result=joust.decision().unwrap().room.unwrap().choices[0].kind.clone();let RoomChoiceKind::JoustResult{owner_wins}=result else{panic!("actual captured result")};let rng=joust.state.rng.clone();super::rooms::choose(&mut joust,result);assert_eq!(joust.state.gold,if owner_wins{49}else{149});assert_eq!(joust.state.rng,rng);
    let mut falling=mapped();falling.state.deck.clear();let a=mutations::obtain_card(&mut falling.state,CardId::GeneticAlgorithm,0).unwrap();let b=mutations::obtain_card(&mut falling.state,CardId::GeneticAlgorithm,0).unwrap();falling.state.deck[0].misc=17;falling.state.deck[1].misc=29;falling=event(falling,EventId::Falling,3);
    let rows=falling.decision().unwrap().room.unwrap().choices;assert_eq!(rows.iter().filter(|r|r.status==RoomChoiceStatus::Disabled).count(),2);let row=rows.iter().find(|r|matches!(r.kind,RoomChoiceKind::FallingLose{card_type:CardType::Skill,instance:Some(_)})).unwrap();let RoomChoiceKind::FallingLose{instance:Some(instance),..}=row.kind else{unreachable!()};assert!([a,b].contains(&instance));let twin=falling.state.deck.iter().find(|c|c.instance!=instance).unwrap().clone();falling.step(&GameAction::Choose(row.id)).unwrap();assert_eq!(falling.state.deck,vec![twin]);let before=falling.clone();assert!(falling.step(&GameAction::Choose(row.id)).is_err());assert_eq!(falling,before);
    let mut last=mapped();last.state.deck.clear();mutations::obtain_card(&mut last.state,CardId::GeneticAlgorithm,0).unwrap();last=event(last,EventId::Falling,3);let action=last.decision().unwrap().candidates.into_iter().next().unwrap().action;let transition=last.step(&action).unwrap();assert_eq!(transition.terminal,Some(RunTerminal::EmptyDeck));assert!(last.state.deck.is_empty());
    for gold in [0,99]{for mask in [false,true]{let mut tomb=mapped();tomb.state.gold=gold;if mask{crate::items::acquire_relic(&mut tomb.state,RelicId::RedMask).unwrap();}tomb=event(tomb,EventId::TombOfLordRedMask,3);super::rooms::choose(&mut tomb,if mask{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold});assert_eq!(tomb.state.gold,if mask{gold+222}else{0});assert_eq!(tomb.state.relics.iter().filter(|id|**id==RelicId::RedMask).count(),1);}}
}
