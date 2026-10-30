//! M23 finite Watcher Acts1–2 on the shared Run owners. Supplied map positions,
//! visit counters and Lament HP establish owner behavior only; they are never a
//! canonical traversal or normal-completion witness.
use super::super::*;
use crate::{combat::rng::Partition,content::{CardId,Character,EncounterId,EventId,PotionId,RelicId}};
use super::m17_defect_act1::{choose,diagnostic_room,settle_lament,cards};
use rooms::m22_content as data;

fn history(a:EncounterId,b:EncounterId)->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match a{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],EncounterId::SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};
    p.seen_bosses[1]=match b{EncounterId::Champ=>vec!["Automaton","Collector"],EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],EncounterId::Collector=>vec!["Champ","Automaton"],_=>unreachable!()};p
}
fn watcher(p:CreationProfile,root:[u8;32],scenario:RoomScenario)->Result<Run,RunError>{Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),scenario)}
fn run(b:EncounterId)->Run{watcher(history(EncounterId::Guardian,b),[7;32],RoomScenario::M23WatcherAct2).unwrap()}
/// Supplied Lament path through a real Act1 boss source and confirmed skip.
fn act2(b:EncounterId)->Run{
    let mut r=run(b);choose(&mut r,RoomChoiceKind::NeowLament);
    diagnostic_room(&mut r,MapNodeKind::Boss);settle_lament(&mut r);
    choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::ConfirmSkip);
    assert_eq!((r.state.act,r.state.floor),(2,17));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);r
}
fn relic_rows(r:&Run)->Vec<RelicId>{r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Relic(id)=c.kind{Some(id)}else{None}).collect()}
fn potion_rows(r:&Run)->Vec<PotionId>{r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Potion(id)=c.kind{Some(id)}else{None}).collect()}

#[test]
fn m23_start_and_act1_prefix_match_m22(){
    // Canonical Catalog start and actual short Neow.
    let mut r=run(EncounterId::Champ);assert_eq!((r.state.hp,r.state.max_hp,r.state.gold),(72,72,99));
    for (id,n) in [(CardId::StrikePurple,4),(CardId::DefendPurple,4),(CardId::Eruption,1),(CardId::Vigilance,1)]{assert_eq!(r.state.deck.iter().filter(|c|c.id==id&&c.upgrades==0).count(),n);}
    assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<std::collections::BTreeSet<_>>().len(),10);
    assert_eq!(r.state.relics,vec![RelicId::PureWater]);assert_eq!(r.state.potions,vec![None;3]);assert_eq!(r.state.keys,[false;3]);assert!(r.state.map.is_none());
    assert_eq!(r.decision().unwrap().content_profile,Some(RoomScenario::M23WatcherAct2));
    choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});assert_eq!((r.state.hp,r.state.max_hp),(79,79));
    let mut lament=run(EncounterId::Champ);choose(&mut lament,RoomChoiceKind::NeowLament);assert_eq!(lament.state.relic_counters.get(&RelicId::NeowsLament),Some(&3));
    for (character,ascension,full) in [(Character::Silent,0,false),(Character::Defect,0,false),(Character::Ironclad,0,false),(Character::Watcher,1,false),(Character::Watcher,0,true)]{
        let mut p=history(EncounterId::Guardian,EncounterId::Champ);p.neow_eligible=full;
        assert_eq!(Run::new_room_scenario(RunConfig{character,ascension},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M23WatcherAct2),Err(RunError::Unsupported("M23 requires Watcher A0 short Neow")));
    }
    // Matched core creation input: the SAME explicit future history on both
    // constructors. Only the scenario tag and the continuation boundary differ.
    for a in [EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss]{for b in [EncounterId::Champ,EncounterId::BronzeAutomaton,EncounterId::Collector]{
        let p=history(a,b);let mut old=watcher(p.clone(),[19;32],RoomScenario::M22WatcherAct1).unwrap();let mut new=watcher(p,[19;32],RoomScenario::M23WatcherAct2).unwrap();
        let equal=|old:&Run,new:&Run|{let mut tagged=old.clone();tagged.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M23WatcherAct2;assert_eq!(&tagged,new,"complete matched-input prefix, including RNG/captures/records");};
        let both=|old:&mut Run,new:&mut Run,kind:RoomChoiceKind|{let a=choose(old,kind.clone());let b=choose(new,kind);assert_eq!(a,b);};
        equal(&old,&new);both(&mut old,&mut new,RoomChoiceKind::NeowLament);equal(&old,&new);
        diagnostic_room(&mut old,MapNodeKind::Monster);diagnostic_room(&mut new,MapNodeKind::Monster);equal(&old,&new);settle_lament(&mut old);settle_lament(&mut new);equal(&old,&new);
        both(&mut old,&mut new,RoomChoiceKind::CardReward);both(&mut old,&mut new,RoomChoiceKind::Card{id:CardId::EmptyFist,upgrades:0});equal(&old,&new);both(&mut old,&mut new,RoomChoiceKind::Proceed);
        diagnostic_room(&mut old,MapNodeKind::Elite);diagnostic_room(&mut new,MapNodeKind::Elite);equal(&old,&new);settle_lament(&mut old);settle_lament(&mut new);equal(&old,&new);
        assert_eq!(relic_rows(&old),relic_rows(&new));assert_eq!(relic_rows(&new).len(),1,"Act1 elite captures one relic through the M22 scope");
        both(&mut old,&mut new,RoomChoiceKind::Proceed);
        diagnostic_room(&mut old,MapNodeKind::Boss);diagnostic_room(&mut new,MapNodeKind::Boss);equal(&old,&new);settle_lament(&mut old);settle_lament(&mut new);equal(&old,&new);
        both(&mut old,&mut new,RoomChoiceKind::Proceed);both(&mut old,&mut new,RoomChoiceKind::Proceed);equal(&old,&new);both(&mut old,&mut new,RoomChoiceKind::ConfirmSkip);
        assert_eq!(old.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2Ready);assert!(old.decision().unwrap().candidates.is_empty());
        let d=new.decision().unwrap();assert_eq!(d.room.unwrap().kind,RoomViewKind::Map);assert!(d.candidates.iter().any(|c|matches!(c.action,GameAction::Move(_))));
        assert_eq!(new.public_map().unwrap().boss,Some(b));
        let mut endpoint=old.clone();endpoint.state.room="Map";let room=endpoint.state.room_loop.as_mut().unwrap();room.scenario=RoomScenario::M23WatcherAct2;room.act2=Some(Default::default());room.screen.kind=RoomViewKind::Map;
        assert_eq!(endpoint,new,"successor map, resources, records, transition flags and RNG are identical under matched history");
    }}
}

#[test]
fn m23_unmatched_native_m22_history_differs_only_after_successor_selection(){
    // Native M22 keeps the all-seen Act2 history and may shuffle; M23's targeted
    // unseen boss draws no shuffle ticket. Disclosed, never normalized.
    let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
    let mut old=watcher(p,[19;32],RoomScenario::M22WatcherAct1).unwrap();let mut new=watcher(history(EncounterId::Guardian,EncounterId::Champ),[19;32],RoomScenario::M23WatcherAct2).unwrap();
    for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::NeowLament);diagnostic_room(r,MapNodeKind::Boss);settle_lament(r);}
    assert_eq!(old.state.rng,new.state.rng,"Act1 gameplay RNG agrees before successor selection");assert_eq!(old.state.deck,new.state.deck);
    for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::ConfirmSkip);}
    assert_ne!(old.state.rng,new.state.rng,"all-seen native M22 draws Act2 boss-order tickets");
    assert_ne!(old.state.rng.clone().draw(RunRngScope::Act2BossOrder),new.state.rng.clone().draw(RunRngScope::Act2BossOrder));
    assert_eq!(new.public_map().unwrap().boss,Some(EncounterId::Champ));
}

#[test]
fn m23_captured_act2_uses_shared_history_and_owned_setup(){
    use EncounterId::*;
    for (kind,boss,history,expected) in [(MapNodeKind::Monster,Champ,0,SphericGuardian),(MapNodeKind::Monster,Champ,1,ThreeByrds),(MapNodeKind::Monster,Champ,2,CenturionAndHealer),(MapNodeKind::Monster,Champ,3,Snecko),(MapNodeKind::Monster,Champ,4,ChosenAndByrds),(MapNodeKind::Monster,Champ,5,ThreeCultists),(MapNodeKind::Elite,Champ,0,GremlinLeader),(MapNodeKind::Elite,BronzeAutomaton,0,Slavers),(MapNodeKind::Elite,Collector,0,BookOfStabbing),(MapNodeKind::Boss,Champ,0,Champ),(MapNodeKind::Boss,BronzeAutomaton,0,BronzeAutomaton),(MapNodeKind::Boss,Collector,0,Collector)]{
        let mut r=run(boss);choose(&mut r,RoomChoiceKind::NeowLament);
        // Supplied Act1 counters prove that transition initializes Act2 once.
        let p=r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap();p.shop_visits=4;p.question_visits=3;
        diagnostic_room(&mut r,MapNodeKind::Boss);settle_lament(&mut r);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::ConfirmSkip);
        let p=r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap();assert_eq!((p.shop_visits,p.question_visits,p.combat_visits),(0,0,0));
        assert_eq!(r.public_map().unwrap().boss,Some(boss));assert_eq!(r.state.room_loop.as_ref().unwrap().act2_boss_order,Some(vec![boss]));
        r.state.map.as_mut().unwrap().regular_encounter_history=vec![SphericGuardian;history];diagnostic_room(&mut r,kind);
        assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,expected);
        assert!(crate::combat::enemies::A0_ENCOUNTER_SETUPS.iter().any(|(id,act,_)|*id==expected&&*act==2));
        assert!(r.state.combat.as_ref().unwrap().item_context().unwrap().m22_profile(),"shared Watcher progression item owner");
        if kind==MapNodeKind::Boss{assert_eq!(r.state.room_loop.as_ref().unwrap().act2_boss_order,Some(vec![]),"target consumed only at accepted entry");assert_eq!(r.state.floor,33);}
        let d=r.decision().unwrap();assert!(d.player.relics.contains(&RelicId::PureWater));assert!(d.combat.unwrap().enemies.iter().all(|e|e.hp==1));
    }
}

#[test]
fn m23_producer_union_is_closed_without_extra_acquisition(){
    assert_eq!(M23_WATCHER_ACT2_CARDS,data::M22_WATCHER_ACT1_CARDS);assert_eq!(M23_WATCHER_ACT2_RELICS,data::M22_WATCHER_ACT1_RELICS);assert_eq!(M23_WATCHER_ACT2_POTIONS,data::M22_WATCHER_ACT1_POTIONS);
    assert_eq!((M23_WATCHER_ACT2_CARDS.len(),M23_WATCHER_ACT2_RELICS.len(),M23_WATCHER_ACT2_POTIONS.len()),(28,18,7));
    // Every nonbasic permanent has a named producer witness; the union is closed.
    let mut produced=std::collections::BTreeSet::new();
    produced.extend(crate::content::Catalog::standard().starter_deck(Character::Watcher).unwrap().iter().copied());
    for row in data::NONBOSS_REWARD_ROWS.iter().chain(&data::SHOP_CARD_ROWS){produced.extend(row.iter().copied());}produced.extend(data::BOSS_CARD_REWARDS);
    assert_eq!(produced,M23_WATCHER_ACT2_CARDS.iter().copied().collect());
    assert_eq!(rooms::m23_content::GENERATED_ONLY,crate::content::watcher::M21_WATCHER_GENERATED);
    assert_eq!(rooms::m23_content::STATUSES,&[CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Wound]);
    let mut r=run(EncounterId::Champ);choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
    for id in M23_WATCHER_ACT2_CARDS{let instance=mutations::obtain_card(&mut r.state,*id,0).unwrap();mutations::upgrade_card(&mut r.state,instance).unwrap();assert!(mutations::obtain_card(&mut r.state,*id,1).is_ok());}
    for id in [CardId::Wound,CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Miracle,CardId::Smite,CardId::Safety,CardId::Parasite,CardId::Strike,CardId::StrikeGreen,CardId::StrikeBlue,CardId::SearingBlow]{let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,0).is_err(),"{id:?} has no M23 permanent producer");assert_eq!(r,before);}
}

fn victory(b:EncounterId,visit:u64,local:u64,kind:MapNodeKind,regular:usize)->Run{
    let mut r=act2(b);r.state.combat_progress=CombatProgress{entered:visit,settled:visit};r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().combat_visits=local;
    r.state.map.as_mut().unwrap().regular_encounter_history=[EncounterId::SphericGuardian,EncounterId::ThreeByrds,EncounterId::CenturionAndHealer].into_iter().cycle().take(regular).collect();
    diagnostic_room(&mut r,kind);settle_lament(&mut r);assert_eq!(r.state.last_combat.as_ref().unwrap().visit,visit);r
}

#[test]
fn m23_reward_calls_full_global_visit_across_act_boundary(){
    for visit in [3u64,5,6,7,11,12]{
        let mut r=victory(EncounterId::Champ,visit,visit%3+1,MapNodeKind::Monster,3);
        assert_eq!(potion_rows(&r),vec![data::POTION_CYCLE[(visit%7) as usize]],"full global visit mod7, not local");
        choose(&mut r,RoomChoiceKind::CardReward);let expected=data::NONBOSS_REWARD_ROWS[(visit%6) as usize].to_vec();assert_eq!(cards(&r),expected);
        assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Skip));
    }
    assert_eq!(data::reward_cards(false,0),data::reward_cards(false,6));assert_eq!(data::combat_potion(0),data::combat_potion(7));
    let mut r=victory(EncounterId::Champ,4,0,MapNodeKind::Boss,0);choose(&mut r,RoomChoiceKind::CardReward);
    assert_eq!(cards(&r),vec![CardId::Brilliance,CardId::Devotion,CardId::Establishment]);
}

#[test]
fn m23_shop_uses_destination_act_counter_and_single_stock_owner(){
    let mut r=act2(EncounterId::Champ);r.state.gold=999;
    diagnostic_room(&mut r,MapNodeKind::Shop);let screen=r.decision().unwrap().room.unwrap();
    let rows=screen.choices.iter().map(|c|(c.kind.clone(),c.price)).collect::<Vec<_>>();
    assert_eq!(rows,vec![(RoomChoiceKind::Card{id:CardId::BattleHymn,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::LikeWater,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::MasterReality,upgrades:0},Some(150)),(RoomChoiceKind::Relic(RelicId::GoldenEye),Some(150)),(RoomChoiceKind::Potion(PotionId::StancePotion),Some(50)),(RoomChoiceKind::Purge,Some(75)),(RoomChoiceKind::Leave,None)]);
    assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().shop_visits,1);
    choose(&mut r,RoomChoiceKind::Card{id:CardId::BattleHymn,upgrades:0});assert_eq!(r.state.gold,999-75);choose(&mut r,RoomChoiceKind::Leave);
    crate::items::acquire_relic(&mut r.state,RelicId::TeardropLocket).unwrap();
    diagnostic_room(&mut r,MapNodeKind::Shop);let rows=r.decision().unwrap().room.unwrap().choices.iter().map(|c|(c.kind.clone(),c.price)).collect::<Vec<_>>();
    assert_eq!(rows,vec![(RoomChoiceKind::Card{id:CardId::MentalFortress,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::Meditate,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::Establishment,upgrades:0},Some(150)),(RoomChoiceKind::Potion(PotionId::BottledMiracle),Some(50)),(RoomChoiceKind::Purge,Some(75)),(RoomChoiceKind::Leave,None)],"owned Teardrop omitted, no replenishment");
}

#[test]
fn m23_relic_capture_filters_owned_in_detached_context(){
    assert_eq!(RunRngScope::parse("run.m23.relic.capture"),Ok(RunRngScope::M23RelicCapture));
    assert_eq!(rooms::m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME,"run.m23.relic.capture");
    // Book of Stabbing elite (Collector offset) with one unrelated and one pool relic owned.
    let mut r=act2(EncounterId::Collector);crate::items::acquire_relic(&mut r.state,data::ELITE_RELICS[0]).unwrap();crate::items::acquire_relic(&mut r.state,RelicId::Strawberry).unwrap();
    r.state.combat_progress=CombatProgress{entered:9,settled:9};r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().combat_visits=1;
    diagnostic_room(&mut r,MapNodeKind::Elite);assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,EncounterId::BookOfStabbing);assert!(r.state.relics.is_empty(),"detached inventory owns relics during combat");
    let node=r.public_map().unwrap().current.unwrap();let floor=u64::from(r.state.floor);
    let eligible=data::ELITE_RELICS.iter().copied().filter(|id|*id!=data::ELITE_RELICS[0]).collect::<Vec<_>>();
    let mut expected=r.state.rng.clone();expected.bounded_scoped(RunRngScope::CombatGoldReward,&[2,node.0,9,MapNodeKind::Elite as u64],11);
    let index=expected.bounded_scoped_checked(RunRngScope::M23RelicCapture,&[2,floor,node.0,9,data::ELITE_SOURCE_TAG],eligible.len() as u64).unwrap();
    settle_lament(&mut r);assert_eq!(r.state.rng,expected,"Act2 elite key [2,floor,node,full visit,0] in the M23 scope only");
    assert_eq!(relic_rows(&r),vec![eligible[index]]);
    // Ordinary Act2 chest: key [2,floor,node,entered,1], owned filtered first.
    let mut r=act2(EncounterId::Champ);crate::items::acquire_relic(&mut r.state,RelicId::Anchor).unwrap();diagnostic_room(&mut r,MapNodeKind::Treasure);
    let node=r.public_map().unwrap().current.unwrap();let key=[2,u64::from(r.state.floor),node.0,r.state.combat_progress.entered,data::CHEST_SOURCE_TAG];
    let mut expected=r.state.rng.clone();let index=expected.bounded_scoped_checked(RunRngScope::M23RelicCapture,&key,2).unwrap();
    choose(&mut r,RoomChoiceKind::OpenChest);assert_eq!(r.state.rng,expected);assert_eq!(relic_rows(&r),vec![[RelicId::Strawberry,RelicId::BloodVial][index]]);
    let screen=r.decision().unwrap().room.unwrap();let before=r.clone();assert_eq!(r.decision().unwrap().room.unwrap(),screen);assert_eq!(r,before,"inspection never rerolls");
}

#[test]
fn m23_capture_empty_singleton_rejection_and_saturation(){
    use rooms::rewards::{capture_m22_relic,capture_relic_in_scope};
    let key=[2,20,77,5,0];
    for pool in [&data::ELITE_RELICS[..],&data::CHEST_RELICS[..]]{
        let rng=RunRng::new([41;32],Partition::Test,0);
        let mut empty=rng.clone();assert_eq!(capture_relic_in_scope(&mut empty,RunRngScope::M23RelicCapture,&key,pool,pool),Ok(None));assert_eq!(empty,rng,"empty pool requests no ticket");
        for unowned in pool{let owned=pool.iter().copied().filter(|id|id!=unowned).collect::<Vec<_>>();
            let mut actual=rng.clone();let mut expected=rng.clone();assert_eq!(expected.bounded_scoped_checked(RunRngScope::M23RelicCapture,&key,1),Ok(0));
            assert_eq!(capture_relic_in_scope(&mut actual,RunRngScope::M23RelicCapture,&key,pool,&owned),Ok(Some(*unowned)));assert_eq!(actual,expected,"singleton still requests one checked ticket");
            assert_ne!(actual,{let mut m22=rng.clone();capture_m22_relic(&mut m22,&key,pool,&owned).unwrap();m22},"Act2 scope is distinct from the M22 scope");
        }
    }
    // Saturated M23 key: checked request fails before any authoritative mutation;
    // the M22 scope with the identical key and other scopes stay usable.
    let mut sat=RunRng::new([41;32],Partition::Test,0);sat.set_counter_for_test(RunRngScope::M23RelicCapture,&key,u64::MAX);let before=sat.clone();
    assert_eq!(capture_relic_in_scope(&mut sat,RunRngScope::M23RelicCapture,&key,&data::ELITE_RELICS,&[]),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(sat,before);
    assert_eq!(capture_relic_in_scope(&mut sat,RunRngScope::M23RelicCapture,&key,&[RelicId::Vajra],&[]),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(sat,before);
    assert!(capture_m22_relic(&mut sat,&key,&data::ELITE_RELICS,&[]).unwrap().is_some());
    let mut r=act2(EncounterId::Champ);diagnostic_room(&mut r,MapNodeKind::Treasure);let node=r.public_map().unwrap().current.unwrap();
    let key=[2,u64::from(r.state.floor),node.0,r.state.combat_progress.entered,data::CHEST_SOURCE_TAG];
    let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;
    r.state.rng.set_counter_for_test(RunRngScope::M23RelicCapture,&key,u64::MAX);let before=r.clone();
    assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(r,before,"no screen/hook/RNG mutation before capture acceptance");
}

#[test]
fn m23_captured_claims_and_chests_never_reroll(){
    let mut r=act2(EncounterId::Champ);for id in data::CHEST_RELICS{crate::items::acquire_relic(&mut r.state,id).unwrap();}diagnostic_room(&mut r,MapNodeKind::Treasure);
    let open=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;
    let before=r.state.rng.clone();choose(&mut r,RoomChoiceKind::OpenChest);assert_eq!(r.state.rng,before,"empty stock omits relic without RNG");
    let opened=r.clone();assert!(r.step(&GameAction::Choose(open)).is_err(),"stale OpenChest");assert_eq!(r,opened);
    assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Sapphire,RoomChoiceKind::Proceed]);
    // Full belt disables the captured potion; discard admits the same item once.
    let mut r=victory(EncounterId::Champ,4,0,MapNodeKind::Monster,3);r.state.potions=vec![Some(PotionId::AncientPotion);3];
    let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::Potion(_))).unwrap();assert_eq!(row.status,RoomChoiceStatus::Disabled);
    let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,before);
    r.step(&GameAction::DiscardPotion{slot:2}).unwrap();r.step(&GameAction::Choose(row.id)).unwrap();assert_eq!(r.state.potions[2],Some(data::combat_potion(4)));
    let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,before);
}

#[test]
fn m23_ancient_writing_upgrades_exact_purple_starter_instances(){
    let mut r=act2(EncounterId::Champ);r.state.deck[0].upgrades=1;let fist=mutations::obtain_card(&mut r.state,CardId::EmptyFist,0).unwrap();let ids=r.state.deck.iter().map(|c|(c.instance,c.id)).collect::<Vec<_>>();
    diagnostic_room(&mut r,MapNodeKind::Question);let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Event(EventId::AncientWriting));
    assert_eq!(screen.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::AncientElegance,RoomChoiceKind::AncientSimplicity]);
    choose(&mut r,RoomChoiceKind::AncientSimplicity);assert_eq!(r.state.deck.iter().map(|c|(c.instance,c.id)).collect::<Vec<_>>(),ids,"same permanent identities");
    for c in &r.state.deck{let want=u32::from(matches!(c.id,CardId::StrikePurple|CardId::DefendPurple));assert_eq!(c.upgrades,want,"{:?} {}",c.id,c.instance);}
    assert_eq!(r.state.deck.iter().find(|c|c.instance==fist).unwrap().upgrades,0);
    choose(&mut r,RoomChoiceKind::Leave);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    // Elegance uses the shared exact-instance removal selector.
    let mut r=act2(EncounterId::Champ);diagnostic_room(&mut r,MapNodeKind::Question);choose(&mut r,RoomChoiceKind::AncientElegance);
    let target=r.decision().unwrap().candidates.iter().find_map(|c|if let GameAction::Select(id)=c.action{Some(id)}else{None}).unwrap();let n=r.state.deck.len();r.step(&GameAction::Select(target)).unwrap();assert_eq!(r.state.deck.len(),n-1);
}

#[test]
fn m23_joust_cost_capture_and_payout_commit_once(){
    for (gold,event) in [(49,EventId::Purifier),(50,EventId::Joust)]{let mut r=act2(EncounterId::Champ);r.state.gold=gold;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=1;diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(event));}
    let mut r=act2(EncounterId::Champ);r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=2;r.state.map.as_mut().unwrap().regular_encounter_history=vec![EncounterId::SphericGuardian;3];diagnostic_room(&mut r,MapNodeKind::Question);assert!(r.decision().unwrap().combat.is_some(),"later questions are captured Monster rooms");
    let mut r=act2(EncounterId::Champ);r.state.gold=120;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=1;diagnostic_room(&mut r,MapNodeKind::Question);
    let token=r.decision().unwrap().room.unwrap().choices[0].id;choose(&mut r,RoomChoiceKind::JoustMurderer);assert_eq!(r.state.gold,70);
    let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
    let node=r.public_map().unwrap().current.unwrap();let mut expected=r.state.rng.clone();let owner=expected.bounded_scoped(RunRngScope::Act2Joust,&[2,node.0,0],100)<30;
    choose(&mut r,RoomChoiceKind::JoustContinue);assert_eq!(r.state.rng,expected);
    let row=r.decision().unwrap().room.unwrap().choices[0].clone();assert_eq!(row.kind,RoomChoiceKind::JoustResult{owner_wins:owner});
    choose(&mut r,row.kind);assert_eq!(r.state.gold,if owner{70}else{170},"murderer bet pays100 only when correct");
    let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,before);assert_eq!(r.state.rng,expected);
    choose(&mut r,RoomChoiceKind::Leave);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

fn boss_menu(b:EncounterId)->Run{let mut r=victory(b,4,0,MapNodeKind::Boss,0);choose(&mut r,RoomChoiceKind::CardReward);choose(&mut r,RoomChoiceKind::Skip);
    let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.committed_events,vec![CommittedEvent::FloorEntered{act:2,floor:34}]);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::BossTreasure);choose(&mut r,RoomChoiceKind::OpenChest);r}

#[test]
fn m23_boss_rewards_exchange_and_second_transition_are_source_owned(){
    let r=boss_menu(EncounterId::Champ);let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::BossRelicChoice);
    assert_eq!(relic_rows(&r),data::BOSS_RELICS.to_vec());assert_eq!(screen.choices.last().unwrap().kind,RoomChoiceKind::Skip);
    assert!(screen.choices.iter().all(|c|c.status==RoomChoiceStatus::Offered));
    // HolyWater replaces actual PureWater slot0 only; no fourth Miracle source.
    let mut h=r.clone();let owned=h.state.relics.clone();assert_eq!(owned[0],RelicId::PureWater);choose(&mut h,RoomChoiceKind::Relic(RelicId::HolyWater));let mut want=owned.clone();want[0]=RelicId::HolyWater;assert_eq!(h.state.relics,want,"slot0 replacement, no append");assert_eq!(h.decision().unwrap().room.unwrap().kind,RoomViewKind::BossRelicChild);
    // Prior replacement disables HolyWater; owned offers are disabled.
    let mut prior=act2(EncounterId::Champ);crate::items::acquire_relic(&mut prior.state,RelicId::HolyWater).unwrap();crate::items::acquire_relic(&mut prior.state,RelicId::VioletLotus).unwrap();
    prior.state.combat_progress=CombatProgress{entered:4,settled:4};diagnostic_room(&mut prior,MapNodeKind::Boss);settle_lament(&mut prior);choose(&mut prior,RoomChoiceKind::Proceed);choose(&mut prior,RoomChoiceKind::OpenChest);
    let disabled=prior.decision().unwrap().room.unwrap().choices.into_iter().filter(|c|c.status==RoomChoiceStatus::Disabled).map(|c|c.kind).collect::<Vec<_>>();
    assert_eq!(disabled,vec![RoomChoiceKind::Relic(RelicId::HolyWater),RoomChoiceKind::Relic(RelicId::VioletLotus)]);
    // Skip uses the shared confirm/cancel parent.
    let mut s=r.clone();choose(&mut s,RoomChoiceKind::Skip);choose(&mut s,RoomChoiceKind::CancelSkip);assert_eq!(s.decision().unwrap().room.unwrap(),screen);
    // EmptyCage mandatory child before Proceed.
    let mut c=r.clone();choose(&mut c,RoomChoiceKind::Relic(RelicId::EmptyCage));let original=c.state.deck.clone();
    assert!(c.decision().unwrap().candidates.iter().all(|x|matches!(x.action,GameAction::Select(_))),"no Proceed before the child completes");
    for _ in 0..2{let a=c.decision().unwrap().candidates[0].action.clone();c.step(&a).unwrap();}assert_eq!(c.state.deck.len(),original.len()-2);
    let t=choose(&mut c,RoomChoiceKind::Proceed);assert_eq!(t.committed_events,vec![CommittedEvent::ActAdvanced{from:2,to:3}]);assert_eq!(c.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3Ready);
}

#[test]
fn m23_act3ready_requires_complete_predecessor_and_live_source(){
    let mut r=boss_menu(EncounterId::BronzeAutomaton);choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));
    let deck=r.state.deck.clone();let relics=r.state.relics.clone();let gold=r.state.gold;let records=r.state.history.clone();r.state.hp=31;
    // Missing completed (1,2,17) predecessor or a dead/terminal owner rejects before mutation.
    for mutate in [0,1,2]{let mut bad=r.clone();match mutate{0=>bad.state.room_loop.as_mut().unwrap().transition=None,1=>bad.state.hp=0,_=>bad.state.room_loop.as_mut().unwrap().transition.as_mut().unwrap().heal_applied=false}
        let before=bad.clone();let id=bad.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).map(|c|c.id);
        if let Some(id)=id{assert!(bad.step(&GameAction::Choose(id)).is_err());assert_eq!(bad,before);}}
    let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0,"no readiness bonus");assert_eq!(t.committed_events,vec![CommittedEvent::ActAdvanced{from:2,to:3}]);
    let frame=r.state.room_loop.as_ref().unwrap().transition.clone().unwrap();assert_eq!((frame.from,frame.to,frame.floor,frame.reset_applied,frame.heal_applied,frame.initialized),(2,3,34,true,true,true));
    let d=r.decision().unwrap();assert_eq!((d.act,d.floor),(3,34));assert_eq!(d.player.hp,d.player.max_hp);assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3Ready);assert!(d.candidates.is_empty()&&d.terminal.is_none());
    assert_eq!(r.state.deck,deck);assert_eq!(r.state.relics,relics);assert_eq!(r.state.gold,gold);assert!(r.state.history.rooms.starts_with(&records.rooms));
    let map=r.public_map().unwrap();assert!(matches!(map.boss,Some(EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca)));
    assert!(r.state.map.as_ref().unwrap().nodes.iter().all(|n|n.floor>=35&&!n.visited));assert!(r.state.room_loop.as_ref().unwrap().act2.is_none()&&r.state.room_loop.as_ref().unwrap().act3.is_none());
    let before=r.clone();assert!(r.step(&GameAction::Confirm).is_err());assert!(rooms::transition::drive(&mut r).is_err());assert_eq!(r,before);
}

#[test]
fn m23_transition_faults_keep_flags_and_source_history(){
    let mut r=boss_menu(EncounterId::Collector);choose(&mut r,RoomChoiceKind::Relic(RelicId::VioletLotus));r.state.hp=31;
    r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:2,to:4,floor:34,reset_applied:false,heal_applied:false,initialized:false});
    assert!(matches!(rooms::transition::drive(&mut r),Err(RunError::Faulted(_))));assert_eq!(r.state.hp,r.state.max_hp,"heal applied once before the invariant fault");
    let frame=r.state.room_loop.as_ref().unwrap().transition.clone().unwrap();assert!(frame.reset_applied&&frame.heal_applied&&!frame.initialized);
    let before=r.clone();assert!(rooms::transition::drive(&mut r).is_err());assert_eq!(r,before,"no reheal/regeneration");
    // Genuine Death settles a record with no reward or transition.
    let mut d=act2(EncounterId::Champ);d.state.hp=1;diagnostic_room(&mut d,MapNodeKind::Elite);
    let end=d.decision().unwrap().candidates.iter().find(|c|c.action==GameAction::Combat(crate::combat::CombatAction::EndTurn)).unwrap().action.clone();
    for _ in 0..16{if d.state.terminal.is_some(){break}d.step(&end).unwrap();}assert_eq!(d.state.terminal,Some(RunTerminal::Death));assert_eq!(d.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Death);assert_eq!(d.state.act,2);assert!(d.state.room_loop.as_ref().unwrap().transition.as_ref().is_some_and(|t|t.from==1));
}
