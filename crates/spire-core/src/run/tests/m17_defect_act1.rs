use super::super::*;
use crate::{combat::{CombatAction,rng::Partition},content::{Character,CardId,RelicId,EncounterId,PotionId}};
fn profile(boss:EncounterId)->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match boss{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],EncounterId::SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};p
}
fn run_boss(boss:EncounterId)->Run{Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile(boss),RunRng::new([7;32],Partition::Test,0),RoomScenario::M17DefectAct1).expect("canonical Defect short-Neow profile")}
fn run()->Run{run_boss(EncounterId::Guardian)}
pub(super) fn choose(r:&mut Run,kind:RoomChoiceKind)->RunTransition{
    let d=r.decision().unwrap();let room=d.room.unwrap();let row=room.choices.iter().find(|c|c.kind==kind&&c.status==RoomChoiceStatus::Offered).unwrap_or_else(||panic!("expected {kind:?}, got {room:?}"));
    r.step(&GameAction::Choose(row.id)).unwrap()
}
#[test]
fn m17_canonical_start_and_short_neow_use_catalog_and_initial_history(){
    for boss in [EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss]{
        let mut r=run_boss(boss);let d=r.decision().unwrap();
        assert_eq!((d.player.character,d.player.hp,d.player.max_hp,d.player.gold),(Character::Defect,75,75,99));
        assert_eq!(d.player.relics,vec![RelicId::CrackedCore]);assert_eq!(d.player.potions,vec![None;3]);assert_eq!(d.player.keys,[false;3]);
        assert_eq!(d.objects.len(),10);for (id,count) in [(CardId::StrikeBlue,4),(CardId::DefendBlue,4),(CardId::Zap,1),(CardId::Dualcast,1)]{assert_eq!(d.objects.iter().filter(|c|c.id==id).count(),count);}
        assert!(r.public_map().is_none());choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});assert_eq!((r.state.hp,r.state.max_hp),(82,82));assert_eq!(r.public_map().unwrap().boss,Some(boss));assert!(r.state.room_loop.as_ref().unwrap().act1.is_some());
    }
    for (character,ascension,full) in [(Character::Silent,0,false),(Character::Ironclad,0,false),(Character::Defect,1,false),(Character::Defect,0,true)]{
        let mut p=profile(EncounterId::Guardian);p.neow_eligible=full;
        assert!(Run::new_room_scenario(RunConfig{character,ascension},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M17DefectAct1).is_err());
    }
}

// Supplied predecessor boundaries below test shared owners, not traversal.
pub(super) fn diagnostic_room(r:&mut Run,kind:MapNodeKind){
    let map=r.state.map.as_mut().unwrap();
    let target=map.nodes.iter().find(|n|n.kind==kind&&!n.visited&&map.nodes.iter().any(|p|p.edges.contains(&n.id))).unwrap().id;
    let previous=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();previous.visited=true;r.state.floor=previous.floor;map.current=Some(previous.id);
    for n in &mut map.nodes{n.burning=false;}
    r.step(&GameAction::Move(target)).unwrap();
}
pub(super) fn settle_lament(r:&mut Run){
    for _ in 0..128{
        let d=r.decision().unwrap();if d.combat.is_none(){return}
        let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{target:Some(_),..})))
            .map(|c|c.action.clone()).unwrap_or(GameAction::Combat(CombatAction::EndTurn));r.step(&action).unwrap();
    }panic!("diagnostic Lament combat did not settle")
}
fn victory(visit:u64,kind:MapNodeKind,boss:EncounterId)->Run{
    let mut r=run_boss(boss);choose(&mut r,RoomChoiceKind::NeowLament);
    r.state.combat_progress=CombatProgress{entered:visit,settled:visit};diagnostic_room(&mut r,kind);settle_lament(&mut r);
    assert_eq!(r.state.last_combat.as_ref().unwrap().visit,visit);r
}
pub(super) fn cards(r:&Run)->Vec<CardId>{r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Card{id,..}=c.kind{Some(id)}else{None}).collect()}
#[test]
fn m17_profile_admission_matches_28_producers_and_29_capability(){
    let mut r=run();assert_eq!((M17_DEFECT_ACT1_CARDS.len(),M17_DEFECT_ACT1_RELICS.len(),M17_DEFECT_ACT1_POTIONS.len()),(28,14,5));
    let mut produced=vec![CardId::StrikeBlue,CardId::DefendBlue,CardId::Zap,CardId::Dualcast];
    for i in 0..5{produced.extend(rooms::m17_content::reward_cards(false,i));}
    produced.extend(rooms::m17_content::reward_cards(true,0));for later in [false,true]{produced.extend(rooms::m17_content::shop_cards(later));}
    assert_eq!(produced.len(),28);assert!(produced.iter().all(|id|M17_DEFECT_ACT1_CARDS.contains(id)));
    assert_eq!(crate::content::defect::M16_DEFECT_CARDS.iter().filter(|id|!produced.contains(id)).copied().collect::<Vec<_>>(),vec![CardId::EchoForm]);
    for &id in M17_DEFECT_ACT1_CARDS{
        assert!(crate::content::defect::supports_permanent(id));assert!(crate::content::defect::supports_generated(id));
        let instance=mutations::obtain_card(&mut r.state,id,0).unwrap();
        if id==CardId::GeneticAlgorithm{r.state.deck.iter_mut().find(|c|c.instance==instance).unwrap().misc=19;}
        mutations::upgrade_card(&mut r.state,instance).unwrap();let c=r.state.deck.iter().find(|c|c.instance==instance).unwrap();assert_eq!(c.upgrades,1);
        if id==CardId::GeneticAlgorithm{assert_eq!(c.misc,19);}
        assert!(!mutations::can_upgrade(&r.state,c));mutations::remove_card(&mut r.state,instance).unwrap();
        mutations::obtain_card(&mut r.state,id,1).unwrap();
    }
    for (id,up) in [(CardId::EchoForm,0),(CardId::Leap,0),(CardId::IronWave,0),(CardId::Burn,0),(CardId::Dazed,0),(CardId::Slimed,0),(CardId::Zap,2)]{
        let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,up).is_err());assert_eq!(r,before);
    }
}
#[test]
fn m17_finite_rows_cursors_and_belt_keep_shared_custody(){
    use CardId::*;
    let rows=[[BallLightning,ColdSnap,Coolheaded],[Barrage,CompileDriver,Darkness],[Fusion,Defragment,GeneticAlgorithm],[Claw,SteamBarrier,Streamline],[Storm,Recursion,BiasedCognition],[BallLightning,ColdSnap,Coolheaded]];
    for (visit,row) in rows.into_iter().enumerate(){
        let mut r=victory(visit as u64,MapNodeKind::Monster,EncounterId::Guardian);
        let potion=rooms::m17_content::combat_potion(visit as u64);let d=r.decision().unwrap();let token=d.room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Potion(potion)).unwrap().id;
        r.state.potions=vec![Some(PotionId::StrengthPotion);3];let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
        r.step(&GameAction::DiscardPotion{slot:0}).unwrap();choose(&mut r,RoomChoiceKind::Potion(potion));let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
        choose(&mut r,RoomChoiceKind::CardReward);assert_eq!(cards(&r),row);
        let next=r.state.next_instance;choose(&mut r,RoomChoiceKind::Card{id:row[0],upgrades:0});assert_eq!(r.state.deck.last().unwrap().instance,next);assert_eq!(r.state.combat_progress,CombatProgress{entered:visit as u64+1,settled:visit as u64+1});
    }
    let mut boss=victory(4,MapNodeKind::Boss,EncounterId::SlimeBoss);choose(&mut boss,RoomChoiceKind::CardReward);assert_eq!(cards(&boss),vec![Electrodynamics,Fission,Amplify]);
    for (visit,row,relic,potion) in [(0,[Capacitor,Consume,Loop],RelicId::GoldPlatedCables,PotionId::FocusPotion),(1,[Bullseye,StaticDischarge,Heatsinks],RelicId::EmotionChip,PotionId::PotionOfCapacity),(2,[Bullseye,StaticDischarge,Heatsinks],RelicId::EmotionChip,PotionId::PotionOfCapacity)]{
        let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().shop_visits=visit;diagnostic_room(&mut r,MapNodeKind::Shop);
        assert_eq!(cards(&r),row);let screen=r.decision().unwrap().room.unwrap();
        for c in &screen.choices{match c.kind{RoomChoiceKind::Card{..}=>assert_eq!(c.price,Some(75)),RoomChoiceKind::Relic(id)=>{assert_eq!(id,relic);assert_eq!(c.price,Some(150));assert_eq!(c.status,RoomChoiceStatus::Disabled);},RoomChoiceKind::Potion(id)=>{assert_eq!(id,potion);assert_eq!(c.price,Some(50));},_=>()}}
        let gold=r.state.gold;choose(&mut r,RoomChoiceKind::Purge);r.step(&GameAction::Cancel).unwrap();assert_eq!(r.state.gold,gold);
        choose(&mut r,RoomChoiceKind::Card{id:row[0],upgrades:0});assert_eq!(r.state.gold,gold-75);
    }
    let mut r=victory(0,MapNodeKind::Elite,EncounterId::Guardian);assert_eq!(r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().elite_relic_cursor,1);
    choose(&mut r,RoomChoiceKind::Proceed);assert!(!r.state.relics.contains(&RelicId::Lantern));
    assert_eq!(rooms::m17_content::elite_relic(0,&[RelicId::Lantern]).unwrap(),(RelicId::Pear,2));
    for (start,id) in [RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::DataDisk,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial].into_iter().enumerate(){assert_eq!(rooms::m17_content::elite_relic(start,&[]).unwrap(),(id,start+1));}
    assert_eq!(rooms::m17_content::elite_relic(8,&[]),Err(RunError::Unsupported("captured Defect Act1 elite relic sequence exhausted")));
}

#[test]
fn m17_all_twelve_captures_enter_actual_shared_setups(){
    use EncounterId::*;
    let cases=[(MapNodeKind::Monster,0,Guardian,Cultist),(MapNodeKind::Monster,1,Guardian,JawWorm),(MapNodeKind::Monster,2,Guardian,TwoLouse),(MapNodeKind::Monster,3,Guardian,LargeSlime),(MapNodeKind::Monster,4,Guardian,ThreeLouse),(MapNodeKind::Monster,5,Guardian,LotsOfSlimes),(MapNodeKind::Elite,0,Guardian,GremlinNob),(MapNodeKind::Elite,0,Hexaghost,Lagavulin),(MapNodeKind::Elite,0,SlimeBoss,ThreeSentries),(MapNodeKind::Boss,0,Guardian,Guardian),(MapNodeKind::Boss,0,Hexaghost,Hexaghost),(MapNodeKind::Boss,0,SlimeBoss,SlimeBoss)];
    for (kind,history,boss,expected) in cases{
        let mut r=run_boss(boss);choose(&mut r,RoomChoiceKind::NeowLament);
        crate::items::acquire_relic(&mut r.state,RelicId::DataDisk).unwrap();
        r.state.map.as_mut().unwrap().regular_encounter_history=vec![Cultist;history];
        let before=r.state.combat_progress.entered;diagnostic_room(&mut r,kind);
        assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,expected);
        assert!(crate::combat::enemies::A0_ENCOUNTER_SETUPS.iter().any(|(id,act,_)|*id==expected&&*act==1));
        let d=r.decision().unwrap();let c=d.combat.unwrap();assert_eq!(c.character.orbs,vec![Some(crate::combat::OrbKind::Lightning),None,None]);assert_eq!(c.character.focus,1);
        assert!(r.state.relics.is_empty()&&r.state.potions.is_empty()&&r.state.relic_counters.is_empty());
        assert_eq!(d.player.relics.iter().filter(|&&id|id==RelicId::CrackedCore).count(),1);
        assert!(c.enemies.iter().all(|e|e.hp==1));settle_lament(&mut r);
        assert_eq!(r.state.last_combat,Some(CombatRecord{visit:before,encounter:expected,outcome:crate::combat::Outcome::Victory}));
        assert_eq!(r.state.relic_counters.get(&RelicId::NeowsLament),Some(&2));
    }
}
fn boss_chest(boss:EncounterId)->Run{
    let mut r=victory(0,MapNodeKind::Boss,boss);choose(&mut r,RoomChoiceKind::CardReward);choose(&mut r,RoomChoiceKind::Skip);
    let floor=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(floor.reward,1);assert_eq!((r.state.act,r.state.floor),(1,17));choose(&mut r,RoomChoiceKind::OpenChest);r
}
#[test]
fn m17_child_and_transition_faults_never_repeat_accepted_work(){
    for boss in [EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss]{
        let mut r=boss_chest(boss);let chest=r.decision().unwrap().room.unwrap();
        assert_eq!(chest.choices.iter().filter_map(|c|if let RoomChoiceKind::Relic(id)=c.kind{Some(id)}else{None}).collect::<Vec<_>>(),vec![RelicId::EmptyCage,RelicId::SlaversCollar]);
        choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::CancelSkip);assert_eq!(r.decision().unwrap().room.unwrap(),chest);
        let original=r.state.deck.clone();choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));
        let first=r.decision().unwrap().candidates[0].action.clone();r.step(&first).unwrap();assert_eq!(r.state.deck,original);let frozen=r.clone();assert!(r.step(&first).is_err());assert_eq!(r,frozen);
        let second=r.decision().unwrap().candidates[0].action.clone();r.step(&second).unwrap();assert_eq!(r.state.deck,original[2..]);
        r.state.hp=41;let before=r.state.reward_ledger.total;let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!(r.state.reward_ledger.total,before);
        let d=r.decision().unwrap();assert_eq!((d.act,d.floor,d.player.hp),(2,17,d.player.max_hp));assert_eq!(d.terminal,None);assert_eq!(d.room.unwrap().kind,RoomViewKind::Act2Ready);assert!(d.candidates.is_empty());
        assert_eq!(r.state.last_combat.as_ref().unwrap().encounter,boss);assert!(r.public_map().unwrap().boss.is_some());assert_eq!(r.state.combat_progress,CombatProgress{entered:1,settled:1});
        let frozen=r.clone();assert_eq!(r.step(&GameAction::Confirm),Err(RunError::InvalidAction));assert_eq!(r,frozen);
    }
    let mut r=boss_chest(EncounterId::Guardian);choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r.state.hp=41;
    r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:1,to:3,floor:17,reset_applied:false,heal_applied:false,initialized:false});
    assert!(matches!(rooms::transition::drive(&mut r),Err(RunError::Faulted(_))));assert_eq!(r.state.hp,r.state.max_hp);let frame=r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap();assert!(frame.reset_applied&&frame.heal_applied&&!frame.initialized);
    let frozen=r.clone();assert!(rooms::transition::drive(&mut r).is_err());assert_eq!(r,frozen);
    let mut empty=boss_chest(EncounterId::Guardian);let ids=empty.state.deck.iter().skip(1).map(|c|c.instance).collect::<Vec<_>>();for id in ids{mutations::remove_card(&mut empty.state,id).unwrap();}
    choose(&mut empty,RoomChoiceKind::Relic(RelicId::EmptyCage));assert_eq!(empty.state.terminal,Some(RunTerminal::EmptyDeck));assert_eq!(empty.state.act,1);assert!(empty.state.room_loop.as_ref().unwrap().transition.is_none());
}
#[test]
fn m17_real_questions_rest_treasure_and_boss_children_use_shared_owners(){
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(crate::content::EventId::UpgradeShrine));choose(&mut r,RoomChoiceKind::ShrineUpgrade);
    let a=r.decision().unwrap().candidates.iter().find(|c|c.object.as_ref().is_some_and(|c|c.id==CardId::Zap)).unwrap().action.clone();r.step(&a).unwrap();assert_eq!(r.state.deck.iter().find(|c|c.id==CardId::Zap).unwrap().upgrades,1);
    diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(crate::content::EventId::Purifier));choose(&mut r,RoomChoiceKind::ShrineRemove);let a=r.decision().unwrap().candidates[0].action.clone();r.step(&a).unwrap();assert_eq!(r.state.deck.len(),9);
    diagnostic_room(&mut r,MapNodeKind::Question);assert!(r.decision().unwrap().combat.is_some());settle_lament(&mut r);choose(&mut r,RoomChoiceKind::Proceed);
    diagnostic_room(&mut r,MapNodeKind::Rest);r.state.hp=30;choose(&mut r,RoomChoiceKind::Rest{heal:22});assert_eq!(r.state.hp,52);
    diagnostic_room(&mut r,MapNodeKind::Treasure);choose(&mut r,RoomChoiceKind::OpenChest);choose(&mut r,RoomChoiceKind::Sapphire);assert!(r.state.keys[1]);assert!(!r.state.relics.contains(&RelicId::Strawberry));assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::Strawberry)).unwrap().status,RoomChoiceStatus::Skipped);
}

fn acquired_genetic()->(Run,u64){
    let mut r=victory(2,MapNodeKind::Monster,EncounterId::Guardian);choose(&mut r,RoomChoiceKind::CardReward);choose(&mut r,RoomChoiceKind::Card{id:CardId::GeneticAlgorithm,upgrades:0});choose(&mut r,RoomChoiceKind::Proceed);
    let instance=r.state.deck.last().unwrap().instance;assert_eq!(r.state.deck.last().unwrap().misc,1);
    // Compact supplied diagnostic deck, after actual reward acquisition.
    let ids=r.state.deck.iter().filter(|c|c.instance!=instance).map(|c|c.instance).collect::<Vec<_>>();for id in ids{mutations::remove_card(&mut r.state,id).unwrap();}(r,instance)
}
#[test]
fn m17_permanent_misc_and_combat_local_state_have_distinct_lifetimes(){
    let (mut r,instance)=acquired_genetic();diagnostic_room(&mut r,MapNodeKind::Monster);
    r.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();assert_eq!(r.state.deck[0].misc,3);assert_eq!(r.decision().unwrap().combat.unwrap().player.block,1);
    let frozen=r.clone();assert!(r.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).is_err());assert_eq!(r,frozen);
    r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();assert_eq!(r.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);assert_eq!(r.state.deck[0].misc,3);choose(&mut r,RoomChoiceKind::Proceed);
    diagnostic_room(&mut r,MapNodeKind::Rest);choose(&mut r,RoomChoiceKind::Smith);let a=r.decision().unwrap().candidates[0].action.clone();r.step(&a).unwrap();assert_eq!((r.state.deck[0].upgrades,r.state.deck[0].misc),(1,3));
    diagnostic_room(&mut r,MapNodeKind::Monster);r.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();assert_eq!(r.decision().unwrap().combat.unwrap().player.block,3);assert_eq!(r.state.deck[0].misc,6);r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();assert_eq!(r.state.deck[0].misc,6);
    let (mut dead,instance)=acquired_genetic();diagnostic_room(&mut dead,MapNodeKind::Monster);dead.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();
    let hp=dead.decision().unwrap().player.hp as i32;
    dead.state.combat.as_mut().unwrap().enqueue_native_effects(vec![crate::combat::Effect{source:crate::combat::Source::None,target:Some(crate::combat::Actor::Player),kind:crate::combat::EffectKind::Damage(crate::combat::DamagePayload::HpLoss(hp))}],None).unwrap();dead.settle_combat().unwrap();assert_eq!(dead.state.terminal,Some(RunTerminal::Death));assert_eq!(dead.state.deck[0].misc,3);
    let (mut fault,instance)=acquired_genetic();fault.state.map.as_mut().unwrap().regular_encounter_history=vec![EncounterId::Cultist;2];diagnostic_room(&mut fault,MapNodeKind::Monster);
    crate::combat::tests::m16_support::exhaust_juggernaut_rng(fault.state.combat.as_mut().unwrap());
    assert!(matches!(fault.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})),Err(RunError::Faulted(_))));assert_eq!(fault.state.deck[0].misc,3);
    let frozen=fault.clone();assert!(fault.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).is_err());assert_eq!(fault,frozen);
}

#[test]
fn m17_combat_local_values_orbs_and_focus_reset_on_next_shared_entry(){
    use CardId::*;
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
    let originals=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for id in originals{mutations::remove_card(&mut r.state,id).unwrap();}
    for id in [SteamBarrier,Streamline,Claw,Capacitor,Defragment]{mutations::obtain_card(&mut r.state,id,0).unwrap();}
    diagnostic_room(&mut r,MapNodeKind::Monster);
    // Real card steps mutate local cost/Block/family state; this supplied small
    // deck is a diagnostic and never a normal completion witness.
    for id in [SteamBarrier,Claw,Capacitor,Defragment]{let d=r.decision().unwrap();let c=d.combat.unwrap();if let Some(a)=d.candidates.iter().find(|a|matches!(a.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|v|v.instance.0==instance&&v.id==id))){r.step(&a.action).unwrap();}}
    let d=r.decision().unwrap();assert!(d.combat.as_ref().unwrap().character.orbs.len()>3);let enemies=d.combat.unwrap().enemies;
    let effects=enemies.into_iter().map(|e|crate::combat::Effect{source:crate::combat::Source::None,target:Some(crate::combat::Actor::Enemy(e.instance_id)),kind:crate::combat::EffectKind::Damage(crate::combat::DamagePayload::HpLoss(e.hp))}).collect();
    r.state.combat.as_mut().unwrap().enqueue_native_effects(effects,None).unwrap();r.settle_combat().unwrap();choose(&mut r,RoomChoiceKind::Proceed);diagnostic_room(&mut r,MapNodeKind::Monster);
    let c=r.decision().unwrap().combat.unwrap();assert_eq!(c.character.orbs,vec![Some(crate::combat::OrbKind::Lightning),None,None]);assert_eq!(c.character.focus,0);
    for card in c.hand{match card.id{SteamBarrier=>assert_eq!(card.values.unwrap().base_block,Some(6)),Streamline=>assert_eq!(card.effective_cost,Some(2)),Claw=>assert_eq!(card.values.unwrap().base_damage,Some(3)),_=>()}}
}

#[test]
fn m17_owned_stock_and_exhausted_reward_preparation_preserve_cursor_and_inventory(){
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);crate::items::acquire_relic(&mut r.state,RelicId::Lantern).unwrap();crate::items::acquire_relic(&mut r.state,RelicId::GoldPlatedCables).unwrap();r.state.gold=200;
    diagnostic_room(&mut r,MapNodeKind::Shop);assert!(!r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::GoldPlatedCables)));
    choose(&mut r,RoomChoiceKind::Leave);diagnostic_room(&mut r,MapNodeKind::Elite);settle_lament(&mut r);assert_eq!(r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().elite_relic_cursor,2);assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Pear)));
    let mut exhausted=run();choose(&mut exhausted,RoomChoiceKind::NeowLament);exhausted.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().elite_relic_cursor=8;diagnostic_room(&mut exhausted,MapNodeKind::Elite);
    let record=CombatRecord{visit:0,encounter:EncounterId::GremlinNob,outcome:crate::combat::Outcome::Victory};let before=exhausted.clone();
    assert!(matches!(rooms::rewards::prepare_combat(&exhausted,&record),Err(RunError::Unsupported("captured Defect Act1 elite relic sequence exhausted"))));assert_eq!(exhausted,before);
}
