//! M24 finite Watcher Acts1–3 on the shared Run owners. Supplied map positions,
//! visit counters and Lament HP establish owner behavior only; they are never a
//! canonical traversal or normal-completion witness.
use super::super::*;
use crate::{combat::{CombatAction,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId}};
use super::m17_defect_act1::{choose,diagnostic_room,settle_lament};
use rooms::m22_content as data;

fn history(a:EncounterId,b:EncounterId,c:EncounterId)->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match a{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],EncounterId::SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};
    p.seen_bosses[1]=match b{EncounterId::Champ=>vec!["Automaton","Collector"],EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],EncounterId::Collector=>vec!["Champ","Automaton"],_=>unreachable!()};
    p.seen_bosses[2]=match c{EncounterId::AwakenedOne=>vec!["TimeEater","DonuAndDeca"],EncounterId::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],EncounterId::DonuAndDeca=>vec!["AwakenedOne","TimeEater"],_=>unreachable!()};p
}
fn watcher(p:CreationProfile,root:[u8;32],scenario:RoomScenario)->Result<Run,RunError>{Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),scenario)}
/// Matched inputs: identical creation history, root, partition and ordinal.
fn pair(b:EncounterId,c:EncounterId,root:[u8;32])->(Run,Run){
    let p=history(EncounterId::Guardian,b,c);(watcher(p.clone(),root,RoomScenario::M23WatcherAct2).unwrap(),watcher(p,root,RoomScenario::M24WatcherAct3).unwrap())
}
/// The only allowed differences before the (2,3,34) continuation are the scenario tag
/// and, inside an active combat, the M24 item-context profile flag (design §5, I2).
fn assert_prefix(old:&Run,new:&Run,context:&str){
    let mut tagged=old.clone();tagged.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M24WatcherAct3;
    if let Some(combat)=tagged.state.combat.as_mut(){combat.mark_watcher_act3_for_test();}
    assert_eq!(&tagged,new,"{context}");
}
/// Supplied Lament path through a real Act1 boss source and confirmed skip.
fn act2(r:&mut Run){
    choose(r,RoomChoiceKind::NeowLament);diagnostic_room(r,MapNodeKind::Boss);settle_lament(r);
    choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::ConfirmSkip);
    assert_eq!((r.state.act,r.state.floor),(2,17));
}
fn rows(r:&Run)->Vec<(RoomChoiceKind,Option<u32>)>{r.decision().unwrap().room.unwrap().choices.iter().map(|c|(c.kind.clone(),c.price)).collect()}

#[test]
fn m24_rejects_unsupported_creation(){
    for (character,ascension,full) in [(Character::Silent,0,false),(Character::Defect,0,false),(Character::Ironclad,0,false),(Character::Watcher,1,false),(Character::Watcher,0,true)]{
        let mut p=history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne);p.neow_eligible=full;
        assert_eq!(Run::new_room_scenario(RunConfig{character,ascension},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M24WatcherAct3),Err(RunError::Unsupported("M24 requires Watcher A0 short Neow")));
    }
    for unlocked in [false,true]{
        let mut p=history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne);if unlocked{p.act4_enabled=false}else{p.fully_unlocked=false}
        assert_eq!(watcher(p,[7;32],RoomScenario::M24WatcherAct3),Err(RunError::Unsupported("profile must be fully unlocked with Act 4 enabled")),"existing Run::new owner, same as M22/M23");
    }
    let r=watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],RoomScenario::M24WatcherAct3).unwrap();
    assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act4_enabled),(72,72,99,true));assert_eq!(r.state.relics,vec![RelicId::PureWater]);
    assert_eq!(r.decision().unwrap().content_profile,Some(RoomScenario::M24WatcherAct3));
}

#[test]
fn m24_content_union_is_closed(){
    use rooms::m24_content as m24;
    let mut cards=data::M22_WATCHER_ACT1_CARDS.to_vec();cards.push(CardId::Parasite);assert_eq!(m24::M24_WATCHER_ACT3_CARDS,&cards[..]);assert_eq!(cards.len(),29);
    let mut relics=data::M22_WATCHER_ACT1_RELICS.to_vec();relics.push(RelicId::RedMask);assert_eq!(m24::M24_WATCHER_ACT3_RELICS,&relics[..]);assert_eq!(relics.len(),19);
    assert_eq!(m24::M24_WATCHER_ACT3_POTIONS,data::M22_WATCHER_ACT1_POTIONS);assert_eq!(m24::M24_WATCHER_ACT3_POTIONS.len(),7);
    assert_eq!(m24::GENERATED_ONLY,&[CardId::Miracle,CardId::Smite,CardId::Safety]);
    assert_eq!(m24::STATUSES,&[CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Wound,CardId::Void]);
    assert_eq!(m24::RELIC_TAXONOMY_ADDITIONS,&[(RelicId::RedMask,"special")]);
    // Run-deck admission: Parasite for M24 only; M22/M23 unchanged.
    let mut r=watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],RoomScenario::M24WatcherAct3).unwrap();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
    assert!(mutations::supported_in(&r.state,CardId::Parasite));
    for id in m24::M24_WATCHER_ACT3_CARDS{assert!(mutations::supported_in(&r.state,*id),"{id:?}");}
    for id in [CardId::Wound,CardId::Dazed,CardId::Void,CardId::Miracle,CardId::Smite,CardId::Safety,CardId::Strike,CardId::StrikeGreen,CardId::StrikeBlue,CardId::Inflame,CardId::Regret]{assert!(!mutations::supported_in(&r.state,id),"{id:?} must stay outside the M24 permanent union");}
    for scenario in [RoomScenario::M22WatcherAct1,RoomScenario::M23WatcherAct2]{
        let r=watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],scenario).unwrap();assert!(!mutations::supported_in(&r.state,CardId::Parasite),"{scenario:?}");
    }
}

#[test]
fn m24_act3_capture_uses_shared_owner(){
    use crate::content::EncounterId as E;
    for boss in [E::AwakenedOne,E::TimeEater,E::DonuAndDeca]{
        let (_,mut r)=pair(E::Champ,boss,[7;32]);choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
        let node=NodeId(0xF240_0040);r.state.act=3;r.state.floor=39;
        r.state.map=Some(RunMap{act:3,boss_icon:Some(boss),current:None,nodes:vec![MapNode{id:node,floor:40,label:"Monster",edges:vec![],x:0,y:0,kind:MapNodeKind::Monster,hidden_kind:None,visited:false,revealed:true,encounter:None,burning:false}],..RunMap::default()});
        let room=r.state.room_loop.as_mut().unwrap();room.act1=None;room.act3=Some(Default::default());room.act3_boss_order=Some(vec![boss]);
        let cycle=[E::SpireGrowth,E::Transient,E::FourShapes,E::Maw,E::SphereAndTwoShapes,E::JawWormHorde,E::ThreeDarklings,E::WrithingMass];
        let mut history=vec![];let mut seen=vec![];
        for _ in 0..10{
            r.state.map.as_mut().unwrap().regular_encounter_history=history.clone();
            let got=rooms::act3::capture_room(&r.state,node).unwrap().encounter.unwrap();
            match history.len(){0=>assert_eq!(got,E::OrbWalker),1=>assert_eq!(got,E::ThreeShapes),_=>assert!(cycle.contains(&got))}
            assert!(!history.iter().rev().take(2).any(|old|*old==got),"no repeat of the previous two");history.push(got);seen.push(got);
        }
        // Elites: boss-icon offset, no immediate repeat.
        let first=match boss{E::AwakenedOne=>E::GiantHead,E::TimeEater=>E::Nemesis,_=>E::Reptomancer};
        let map=r.state.map.as_mut().unwrap();map.nodes[0].kind=MapNodeKind::Elite;map.elite_encounter_history=vec![];
        assert_eq!(rooms::act3::capture_room(&r.state,node).unwrap().encounter,Some(first));
        r.state.map.as_mut().unwrap().elite_encounter_history=vec![first];assert_ne!(rooms::act3::capture_room(&r.state,node).unwrap().encounter,Some(first));
        // Questions: Falling, Tomb, then a captured Monster.
        r.state.map.as_mut().unwrap().nodes[0].kind=MapNodeKind::Question;
        for (visits,event) in [(0,Some(crate::content::EventId::Falling)),(1,Some(crate::content::EventId::TombOfLordRedMask)),(2,None)]{
            r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=visits;
            let c=rooms::act3::capture_room(&r.state,node).unwrap();assert_eq!(c.event,event);if event.is_none(){assert_eq!(c.kind,MapNodeKind::Monster)}
        }
        // Boss50: only the first boss-order entry matching the icon.
        let map=r.state.map.as_mut().unwrap();map.nodes[0].kind=MapNodeKind::Boss;map.nodes[0].floor=50;
        assert_eq!(rooms::act3::capture_room(&r.state,node).unwrap().encounter,Some(boss));
        r.state.map.as_mut().unwrap().boss_icon=Some(if boss==E::AwakenedOne{E::TimeEater}else{E::AwakenedOne});assert!(rooms::act3::capture_room(&r.state,node).is_err());
    }
    for id in [E::OrbWalker,E::ThreeShapes,E::SpireGrowth,E::Transient,E::FourShapes,E::Maw,E::SphereAndTwoShapes,E::JawWormHorde,E::ThreeDarklings,E::WrithingMass,E::GiantHead,E::Nemesis,E::Reptomancer,E::AwakenedOne,E::TimeEater,E::DonuAndDeca]{
        assert!(crate::combat::enemies::A0_ENCOUNTER_SETUPS.iter().any(|(setup,_,_)|*setup==id),"{id:?}");
    }
}

/// One fixed test rule (not a policy under evaluation): Lament at Neow, confirm any
/// selection, otherwise the first legal card then EndTurn, and never a potion.
fn prefix_action(d:&PublicDecision)->GameAction{
    let has=|a:&GameAction|d.candidates.iter().any(|c|&c.action==a);
    let confirm=GameAction::Combat(CombatAction::ConfirmSelection);if has(&confirm){return confirm}
    if d.combat.is_some(){
        let combat=|f:&dyn Fn(&CombatAction)->bool|d.candidates.iter().find(|c|matches!(&c.action,GameAction::Combat(a) if f(a))).map(|c|c.action.clone());
        return combat(&|a|matches!(a,CombatAction::ChooseStance{..}|CombatAction::SelectCard{..})).or_else(||combat(&|a|matches!(a,CombatAction::PlayCard{..})))
            .unwrap_or(GameAction::Combat(CombatAction::EndTurn))
    }
    if let Some(room)=d.room.as_ref().filter(|r|r.kind==RoomViewKind::Neow){
        if let Some(row)=room.choices.iter().find(|c|c.kind==RoomChoiceKind::NeowLament){return GameAction::Choose(row.id)}
    }
    d.candidates.iter().find(|c|!matches!(c.action,GameAction::UsePotion{..}|GameAction::DiscardPotion{..}|GameAction::Combat(_))).or_else(||d.candidates.first()).map(|c|c.action.clone()).expect("M23 published a legal candidate")
}

#[test]
fn m24_played_prefix_matches_m23(){
    // Matched history, played rooms: take each action from M23's own legal
    // candidates by one fixed test rule and apply it identically to M24.
    let (mut old,mut new)=pair(EncounterId::Champ,EncounterId::AwakenedOne,[7;32]);assert_prefix(&old,&new,"initial");
    let mut steps=0;
    while steps<4096{
        let d=old.decision().unwrap();
        if d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready){break}
        let action=prefix_action(&d);
        let a=old.step(&action);let b=new.step(&action);assert_eq!(a,b,"step {steps}: {action:?}");
        assert_prefix(&old,&new,&format!("after step {steps}: {action:?}"));steps+=1;
    }
    eprintln!("m24 played prefix: {steps} matched steps, endpoint act {} floor {} terminal {:?}",old.state.act,old.state.floor,old.state.terminal);
    assert!(steps>0);
}

#[test]
fn m24_act2_merchant_and_elite_match_m23(){
    // Matched owner level: equal constructed Act2 states, real room entries.
    for (b,c) in [(EncounterId::Champ,EncounterId::AwakenedOne),(EncounterId::Collector,EncounterId::DonuAndDeca)]{
        let (mut old,mut new)=pair(b,c,[19;32]);act2(&mut old);act2(&mut new);assert_prefix(&old,&new,"Act2 start");
        for r in [&mut old,&mut new]{r.state.gold=999;diagnostic_room(r,MapNodeKind::Shop);}
        assert_prefix(&old,&new,"first Act2 merchant");assert_eq!(rows(&new)[0],(RoomChoiceKind::Card{id:CardId::BattleHymn,upgrades:0},Some(75)));
        assert_eq!(new.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().shop_visits,1);
        for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::Leave);diagnostic_room(r,MapNodeKind::Shop);}
        assert_prefix(&old,&new,"later Act2 merchant");assert_eq!(rows(&new)[0],(RoomChoiceKind::Card{id:CardId::MentalFortress,upgrades:0},Some(75)));
        for r in [&mut old,&mut new]{choose(r,RoomChoiceKind::Leave);r.state.combat_progress=CombatProgress{entered:9,settled:9};diagnostic_room(r,MapNodeKind::Elite);settle_lament(r);}
        assert_prefix(&old,&new,"Act2 elite reward under run.m23.relic.capture");
        assert_eq!(rows(&old),rows(&new));assert_eq!(old.state.rng,new.state.rng);
        assert!(rows(&new).iter().any(|(k,_)|matches!(k,RoomChoiceKind::Relic(_))),"Act2 elite captured one relic");
    }
    let _=PotionId::StancePotion;
}

// ---- Task 2: Act3 producers, Parasite admission and relic capture ----

fn m24(c:EncounterId)->Run{watcher(history(EncounterId::Guardian,EncounterId::Champ,c),[7;32],RoomScenario::M24WatcherAct3).unwrap()}
fn advance(r:&mut Run){
    diagnostic_room(r,MapNodeKind::Boss);settle_lament(r);
    choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::ConfirmSkip);
}
/// Supplied Lament path through real Act1 and Act2 boss sources and both transitions.
fn act3(c:EncounterId)->Run{
    let mut r=m24(c);choose(&mut r,RoomChoiceKind::NeowLament);advance(&mut r);r.state.relic_counters.insert(RelicId::NeowsLament,3);advance(&mut r);
    assert_eq!((r.state.act,r.state.floor),(3,34));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);r
}
fn relic_rows(r:&Run)->Vec<RelicId>{r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Relic(id)=c.kind{Some(id)}else{None}).collect()}
fn card_rows(r:&Run)->Vec<CardId>{r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Card{id,..}=c.kind{Some(id)}else{None}).collect()}
fn kill_all(r:&mut Run){let alive=r.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().enumerate().filter(|(_,e)|e.hp>0).map(|(i,_)|i).collect::<Vec<_>>();for slot in alive{super::m9_boss::hit(r,slot,9999);}r.settle_combat().unwrap();}
fn insert_raw(r:&mut Run,id:CardId)->u64{let instance=r.state.next_instance;r.state.next_instance+=1;r.state.deck.push(CardInstance{instance,id,upgrades:0,misc:0});instance}

#[test]
fn m24_watcher_combat_admits_curse_permanent(){
    let mut r=act3(EncounterId::AwakenedOne);let parasite=mutations::obtain_card(&mut r.state,CardId::Parasite,0).unwrap();
    assert!(mutations::obtain_card(&mut r.state.clone(),CardId::Parasite,1).is_err(),"Parasite is never upgraded");
    r.enter_owned(EncounterId::OrbWalker,CombatOrigin::Training{kind:MapNodeKind::Monster}).expect("a Curse permanent enters Watcher combat");
    // Public view, unplayability, Scry and Meditate custody of a Parasite are pinned
    // unconditionally at combat level: combat::tests::m24_watcher_act3::m24_curse_parasite_watcher_view_scry_and_meditate_custody.
    let d=r.decision().unwrap();
    assert!(!d.candidates.iter().any(|x|matches!(x.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if instance==parasite)),"never a legal play");
    kill_all(&mut r);assert_eq!(r.state.deck.iter().filter(|c|c.instance==parasite).count(),1,"settlement census returns it once");
    // The law is not M24-only: an M23 Watcher combat with a constructed Parasite enters too.
    let (mut m23,_)=pair(EncounterId::Champ,EncounterId::AwakenedOne,[19;32]);act2(&mut m23);insert_raw(&mut m23,CardId::Parasite);
    m23.enter_owned(EncounterId::SphericGuardian,CombatOrigin::Training{kind:MapNodeKind::Monster}).expect("M23 Watcher combat admits a Curse permanent");
    // Status and off-color permanents still reject at Watcher construction.
    for id in [CardId::Dazed,CardId::Strike]{let mut bad=act3(EncounterId::AwakenedOne);insert_raw(&mut bad,id);assert!(bad.enter_owned(EncounterId::OrbWalker,CombatOrigin::Training{kind:MapNodeKind::Monster}).is_err(),"{id:?}");}
}

#[test]
fn m24_parasite_gain_through_journal(){
    use crate::combat::{Actor,Effect,EffectKind,Source,DamagePayload};use crate::combat::tests::m19_defect_interactions as support;
    let mut r=act3(EncounterId::AwakenedOne);r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);
    r.enter_owned(EncounterId::WrithingMass,CombatOrigin::Training{kind:MapNodeKind::Monster}).unwrap();
    support::implant_checkpoint(r.state.combat.as_mut().unwrap());r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
    assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1);assert_eq!(r.state.active_combat.as_ref().unwrap().applied_gain_ordinal,1);
    let mut dead=r.clone();let hp=dead.decision().unwrap().player.hp;
    dead.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp as i32))}],None).unwrap();
    dead.settle_combat().unwrap();assert_eq!(dead.state.terminal,Some(RunTerminal::Death));assert_eq!(dead.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1,"Death settles the gain once");
    kill_all(&mut r);assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1,"Victory settles the gain once");
    r.enter_owned(EncounterId::OrbWalker,CombatOrigin::Training{kind:MapNodeKind::Monster}).expect("the next Act3 combat enters with the Parasite");
    kill_all(&mut r);let p=r.state.deck.iter().find(|c|c.id==CardId::Parasite).unwrap().clone();
    assert!(!mutations::can_upgrade(&r.state,&p),"Smith never offers Parasite");
    let before=r.state.max_hp;mutations::remove_card(&mut r.state,p.instance).unwrap();assert_eq!(r.state.max_hp,before-3,"removal applies the shared max-HP loss once");
}

/// Act3 combat Victory at a supplied global visit and Act3-local visit.
fn act3_victory(visit:u64,local:u64,kind:MapNodeKind)->Run{
    let mut r=act3(EncounterId::AwakenedOne);r.state.combat_progress=CombatProgress{entered:visit,settled:visit};r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().combat_visits=local;
    diagnostic_room(&mut r,kind);settle_lament(&mut r);assert_eq!(r.state.last_combat.as_ref().unwrap().visit,visit);r
}

#[test]
fn m24_act3_rewards_use_full_global_visit(){
    for (visit,local) in [(12u64,0u64),(13,1),(17,2),(18,0)]{
        let mut r=act3_victory(visit,local,MapNodeKind::Monster);
        let potions=r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Potion(id)=c.kind{Some(id)}else{None}).collect::<Vec<_>>();
        assert_eq!(potions,vec![data::POTION_CYCLE[(visit%7) as usize]],"full global visit mod7");
        choose(&mut r,RoomChoiceKind::CardReward);assert_eq!(card_rows(&r),data::NONBOSS_REWARD_ROWS[(visit%6) as usize].to_vec(),"full global visit mod6, no Act3 reset");
        assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Skip));
    }
}

#[test]
fn m24_act3_shop_uses_act3_counter(){
    let mut r=act3(EncounterId::AwakenedOne);r.state.gold=999;diagnostic_room(&mut r,MapNodeKind::Shop);
    assert_eq!(rows(&r)[..3],[(RoomChoiceKind::Card{id:CardId::BattleHymn,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::LikeWater,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::MasterReality,upgrades:0},Some(150))],"first Act3 merchant");
    assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().shop_visits,1);
    choose(&mut r,RoomChoiceKind::Leave);crate::items::acquire_relic(&mut r.state,RelicId::TeardropLocket).unwrap();diagnostic_room(&mut r,MapNodeKind::Shop);
    assert_eq!(rows(&r),vec![(RoomChoiceKind::Card{id:CardId::MentalFortress,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::Meditate,upgrades:0},Some(75)),(RoomChoiceKind::Card{id:CardId::Establishment,upgrades:0},Some(150)),(RoomChoiceKind::Potion(PotionId::BottledMiracle),Some(50)),(RoomChoiceKind::Purge,Some(75)),(RoomChoiceKind::Leave,None)],"later Act3 merchant; owned Teardrop omitted");
}

#[test]
fn m24_act3_relic_capture_scope_and_keys(){
    assert_eq!(RunRngScope::parse("run.m24.relic.capture"),Ok(RunRngScope::M24RelicCapture));
    let mut r=act3(EncounterId::AwakenedOne);r.state.combat_progress=CombatProgress{entered:9,settled:9};
    diagnostic_room(&mut r,MapNodeKind::Elite);let node=r.public_map().unwrap().current.unwrap();let floor=u64::from(r.state.floor);
    let mut expected=r.state.rng.clone();expected.bounded_scoped(RunRngScope::CombatGoldReward,&[3,node.0,9,MapNodeKind::Elite as u64],11);
    let index=expected.bounded_scoped_checked(RunRngScope::M24RelicCapture,&[3,floor,node.0,9,data::ELITE_SOURCE_TAG],data::ELITE_RELICS.len() as u64).unwrap();
    settle_lament(&mut r);assert_eq!(r.state.rng,expected,"Act3 elite key [3,floor,node,full visit,0] in the M24 scope only");assert_eq!(relic_rows(&r),vec![data::ELITE_RELICS[index]]);
    let mut r=act3(EncounterId::AwakenedOne);diagnostic_room(&mut r,MapNodeKind::Treasure);let node=r.public_map().unwrap().current.unwrap();
    let key=[3,u64::from(r.state.floor),node.0,r.state.combat_progress.entered,data::CHEST_SOURCE_TAG];
    let mut expected=r.state.rng.clone();let index=expected.bounded_scoped_checked(RunRngScope::M24RelicCapture,&key,data::CHEST_RELICS.len() as u64).unwrap();
    choose(&mut r,RoomChoiceKind::OpenChest);assert_eq!(r.state.rng,expected);assert_eq!(relic_rows(&r),vec![data::CHEST_RELICS[index]]);
    // Act1/Act2 keep their scopes.
    let mut act1=m24(EncounterId::AwakenedOne);act1.state.act=1;assert_eq!(rooms::m23_content::relic_capture_scope(&act1.state),RunRngScope::M22RelicCapture);
    act1.state.act=2;assert_eq!(rooms::m23_content::relic_capture_scope(&act1.state),RunRngScope::M23RelicCapture);
    act1.state.act=3;assert_eq!(rooms::m23_content::relic_capture_scope(&act1.state),RunRngScope::M24RelicCapture);
}

#[test]
fn m24_capture_empty_singleton_saturation(){
    use rooms::rewards::capture_relic_in_scope;let key=[3,40,77,5,0];
    for pool in [&data::ELITE_RELICS[..],&data::CHEST_RELICS[..]]{
        let rng=RunRng::new([41;32],Partition::Test,0);
        let mut empty=rng.clone();assert_eq!(capture_relic_in_scope(&mut empty,RunRngScope::M24RelicCapture,&key,pool,pool),Ok(None));assert_eq!(empty,rng);
        let owned=pool[1..].to_vec();let mut actual=rng.clone();let mut expected=rng.clone();assert_eq!(expected.bounded_scoped_checked(RunRngScope::M24RelicCapture,&key,1),Ok(0));
        assert_eq!(capture_relic_in_scope(&mut actual,RunRngScope::M24RelicCapture,&key,pool,&owned),Ok(Some(pool[0])));assert_eq!(actual,expected,"singleton requests one ticket");
    }
    let mut sat=RunRng::new([41;32],Partition::Test,0);sat.set_counter_for_test(RunRngScope::M24RelicCapture,&key,u64::MAX);let before=sat.clone();
    assert_eq!(capture_relic_in_scope(&mut sat,RunRngScope::M24RelicCapture,&key,&data::ELITE_RELICS,&[]),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(sat,before);
    let mut r=act3(EncounterId::AwakenedOne);diagnostic_room(&mut r,MapNodeKind::Treasure);let node=r.public_map().unwrap().current.unwrap();
    let key=[3,u64::from(r.state.floor),node.0,r.state.combat_progress.entered,data::CHEST_SOURCE_TAG];
    let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;
    r.state.rng.set_counter_for_test(RunRngScope::M24RelicCapture,&key,u64::MAX);let before=r.clone();
    assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::Unsupported("run scoped RNG exhausted")));assert_eq!(r,before);
}

#[test]
fn m24_act3_elite_claim_never_rerolls_on_inspect_exhausted_ledger_or_decline(){
    // Final review M4/N1: the Act3 elite relic row is captured once at settlement. Inspection,
    // a claim after an exhausted choice ledger (relic claims allocate no choice id, so it
    // succeeds once without a fault) and a decline (Proceed without claiming) never reroll.
    let mut r=act3(EncounterId::AwakenedOne);r.state.combat_progress=CombatProgress{entered:9,settled:9};diagnostic_room(&mut r,MapNodeKind::Elite);settle_lament(&mut r);
    let rows=relic_rows(&r);assert_eq!(rows.len(),1);let rng=r.state.rng.clone();
    assert_eq!(relic_rows(&r),rows,"inspect twice");assert_eq!(r.state.rng,rng);
    let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Relic(_))).unwrap().id;
    let mut once=r.clone();once.next_choice=u64::MAX;once.step(&GameAction::Choose(id)).expect("claim succeeds once after an exhausted choice ledger");
    assert!(once.state.fault.is_none());assert_eq!(once.state.rng,rng,"claim never rerolls");assert_eq!(once.state.relics.iter().filter(|x|**x==rows[0]).count(),1,"acquired exactly once");
    assert!(once.step(&GameAction::Choose(id)).is_err(),"no second claim");assert_eq!(once.state.relics.iter().filter(|x|**x==rows[0]).count(),1);assert_eq!(once.state.rng,rng);
    choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.state.rng,rng,"decline never rerolls");assert!(!r.state.relics.contains(&rows[0]),"declined relic not acquired");
}

#[test]
fn m24_act3_claims_never_reroll(){
    let mut r=act3(EncounterId::AwakenedOne);diagnostic_room(&mut r,MapNodeKind::Treasure);
    let open=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;choose(&mut r,RoomChoiceKind::OpenChest);
    let screen=r.decision().unwrap().room.unwrap();let opened=r.clone();assert_eq!(r.decision().unwrap().room.unwrap(),screen);assert_eq!(r,opened,"inspection never rerolls");
    assert!(r.step(&GameAction::Choose(open)).is_err(),"stale OpenChest");assert_eq!(r,opened);
    // A chest claim after an exhausted choice ledger succeeds once (relic claims allocate no
    // choice id); it and a decline both keep the captured row and RNG.
    let row=relic_rows(&r);let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Relic(_))).unwrap().id;
    let mut once=r.clone();once.next_choice=u64::MAX;once.step(&GameAction::Choose(id)).expect("claim succeeds once after an exhausted choice ledger");
    assert!(once.state.fault.is_none());assert_eq!(once.state.rng,opened.state.rng,"claim never rerolls");assert_eq!(once.state.relics.iter().filter(|x|**x==row[0]).count(),1,"acquired exactly once");
    assert!(once.step(&GameAction::Choose(id)).is_err(),"no second claim");assert_eq!(once.state.relics.iter().filter(|x|**x==row[0]).count(),1);
    choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.state.rng,opened.state.rng,"decline never rerolls");assert!(!r.state.relics.contains(&row[0]));
}

#[test]
fn m24_act3_pools_exhausted_before_act3(){
    let mut r=act3(EncounterId::AwakenedOne);for id in data::ELITE_RELICS.iter().chain(data::CHEST_RELICS.iter()){crate::items::acquire_relic(&mut r.state,*id).unwrap();}
    diagnostic_room(&mut r,MapNodeKind::Treasure);let before=r.state.rng.clone();choose(&mut r,RoomChoiceKind::OpenChest);assert_eq!(r.state.rng,before,"empty chest pool requests no RNG");
    assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Sapphire,RoomChoiceKind::Proceed]);
    choose(&mut r,RoomChoiceKind::Proceed);r.state.combat_progress=CombatProgress{entered:9,settled:9};diagnostic_room(&mut r,MapNodeKind::Elite);settle_lament(&mut r);
    assert!(relic_rows(&r).is_empty(),"exhausted elite pool offers no relic, never a fault");assert!(r.state.fault.is_none());
}

// ---- Task 3: Red Mask admission, events, ending and fault labels ----

fn question(r:&mut Run,visits:usize){r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=visits;diagnostic_room(r,MapNodeKind::Question);}
fn enemy_powers(r:&Run)->Vec<Vec<(String,i32)>>{r.decision().unwrap().combat.unwrap().enemies.iter().map(|e|e.powers.iter().map(|p|(p.id.0.to_string(),p.amount)).collect()).collect()}

#[test]
fn m24_red_mask_admitted_through_real_tomb(){
    let mut r=act3(EncounterId::AwakenedOne);r.state.gold=123;question(&mut r,1);
    assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(crate::content::EventId::TombOfLordRedMask));
    choose(&mut r,RoomChoiceKind::TombOfferGold);assert_eq!(r.state.gold,0);assert!(r.state.relics.contains(&RelicId::RedMask));choose(&mut r,RoomChoiceKind::Leave);
    r.state.potions[0]=Some(PotionId::BlockPotion);
    diagnostic_room(&mut r,MapNodeKind::Monster);assert!(r.state.combat.is_some(),"a Watcher combat with Red Mask enters");
    assert!(enemy_powers(&r).iter().all(|p|p.iter().any(|(id,n)|id=="weak"&&*n==1)),"Red Mask applies Weak 1 at combat start: {:?}",enemy_powers(&r));
    // An in-combat potion use rebuilds the context; Red Mask admission persists (N3).
    r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap();kill_all(&mut r);
    assert!(r.state.relics.contains(&RelicId::RedMask));choose(&mut r,RoomChoiceKind::Proceed);
    diagnostic_room(&mut r,MapNodeKind::Elite);assert!(r.state.combat.is_some(),"later Act3 combats still admit Red Mask");
}

#[test]
fn m24_falling_and_tomb_with_watcher_deck(){
    let mut r=act3(EncounterId::AwakenedOne);question(&mut r,0);
    let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Event(crate::content::EventId::Falling));
    let offers=screen.choices.iter().filter_map(|c|if let RoomChoiceKind::FallingLose{card_type,instance}=c.kind{Some((card_type,instance,c.status))}else{None}).collect::<Vec<_>>();
    assert_eq!(offers.len(),3);let (_,skill,_)=offers.iter().find(|(t,_,_)|*t==crate::content::CardType::Skill).copied().unwrap();let skill=skill.unwrap();
    let twin=r.state.deck.iter().find(|c|c.instance!=skill&&c.id==r.state.deck.iter().find(|d|d.instance==skill).unwrap().id).map(|c|c.instance);
    let row=screen.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::FallingLose{card_type:crate::content::CardType::Skill,..})).unwrap().id;
    r.step(&GameAction::Choose(row)).unwrap();assert!(!r.state.deck.iter().any(|c|c.instance==skill),"exactly the captured instance");if let Some(t)=twin{assert!(r.state.deck.iter().any(|c|c.instance==t),"not a same-kind twin");}
    // Tomb with a mask grants 222 gold.
    let mut r=act3(EncounterId::AwakenedOne);crate::items::acquire_relic(&mut r.state,RelicId::RedMask).unwrap();let gold=r.state.gold;question(&mut r,1);choose(&mut r,RoomChoiceKind::TombGold);assert_eq!(r.state.gold,gold+222);
}

#[test]
fn m24_falling_with_parasite_and_last_card(){
    // [Parasite, X]: only X's type is offered; removing X leaves [Parasite], no EmptyDeck.
    let mut r=act3(EncounterId::AwakenedOne);let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}
    let parasite=mutations::obtain_card(&mut r.state,CardId::Parasite,0).unwrap();let x=mutations::obtain_card(&mut r.state,CardId::Vigilance,0).unwrap();question(&mut r,0);
    let screen=r.decision().unwrap().room.unwrap();
    for c in &screen.choices{if let RoomChoiceKind::FallingLose{card_type,instance}=c.kind{
        if card_type==crate::content::CardType::Skill{assert_eq!((instance,c.status),(Some(x),RoomChoiceStatus::Offered));}else{assert_eq!((instance,c.status),(None,RoomChoiceStatus::Disabled));}
    }}
    choose(&mut r,RoomChoiceKind::FallingLose{card_type:crate::content::CardType::Skill,instance:Some(x)});
    assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),vec![parasite]);assert_eq!(r.state.terminal,None);
    // [X]: removing X ends the Run with EmptyDeck.
    let mut r=act3(EncounterId::AwakenedOne);let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}
    let x=mutations::obtain_card(&mut r.state,CardId::Vigilance,0).unwrap();question(&mut r,0);
    choose(&mut r,RoomChoiceKind::FallingLose{card_type:crate::content::CardType::Skill,instance:Some(x)});assert_eq!(r.state.terminal,Some(RunTerminal::EmptyDeck));
}

#[test]
fn m24_tomb_with_zero_gold(){
    let mut r=act3(EncounterId::AwakenedOne);r.state.gold=0;question(&mut r,1);
    let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::TombOfferGold).unwrap();assert_eq!((row.price,row.status),(Some(0),RoomChoiceStatus::Offered));
    r.step(&GameAction::Choose(row.id)).unwrap();assert_eq!(r.state.relics.iter().filter(|id|**id==RelicId::RedMask).count(),1);assert_eq!(r.state.gold,0);
    assert!(r.step(&GameAction::Choose(row.id)).is_err(),"once");choose(&mut r,RoomChoiceKind::Leave);
    diagnostic_room(&mut r,MapNodeKind::Monster);assert!(r.state.combat.is_some(),"the next combat starts with Red Mask");
}

/// Real floor-50 boss entry, then a supplied lethal frontier (M19 pattern) and settlement.
fn boss_victory(c:EncounterId,keys:[bool;3])->Run{
    let mut r=act3(c);r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);r.state.keys=keys;
    diagnostic_room(&mut r,MapNodeKind::Boss);assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,c);
    for _ in 0..8{
        let alive=r.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().enumerate().filter(|(_,e)|e.hp>0).map(|(i,_)|i).collect::<Vec<_>>();
        for slot in alive{super::m9_boss::hit(&mut r,slot,9999);if r.state.combat.as_ref().unwrap().public_full().unwrap().outcome.is_some(){break}}
        if r.state.combat.as_ref().unwrap().public_full().unwrap().outcome.is_some(){break}
        r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
    }
    r.settle_combat().unwrap();r
}

#[test]
fn m24_act3_ending_key_masks(){
    for c in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca]{for mask in 0u8..8{
        let keys=[mask&1!=0,mask&2!=0,mask&4!=0];let mut r=boss_victory(c,keys);
        let d=r.decision().unwrap();assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3BossComplete);
        assert_eq!(d.room.unwrap().choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed],"no card, gold, potion or chest reward after the Act3 boss");
        let (hp,events)=(r.state.hp,r.state.history.events.len());let total=d.reward_total;
        choose(&mut r,RoomChoiceKind::Proceed);for stage in 1..=3{choose(&mut r,RoomChoiceKind::Act3Continue{stage});}
        let d=r.decision().unwrap();
        if keys==[true;3]{
            assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act4Ready);assert!(d.candidates.is_empty()&&d.terminal.is_none());assert_eq!((d.act,d.floor),(3,50));
            assert_eq!(r.state.hp,hp,"no heal");assert_eq!(d.reward_total,total,"no readiness reward");
            assert!(!r.state.history.events[events..].iter().any(|e|matches!(e,CommittedEvent::ActAdvanced{..})));
        }else{
            choose(&mut r,RoomChoiceKind::Act3Continue{stage:4});let d=r.decision().unwrap();assert_eq!(d.terminal,Some(RunTerminal::Act3NoKeys),"{c:?} {keys:?}");
            assert_eq!(d.reward_total,total,"terminal reward contribution 0");assert_eq!((d.act,d.floor),(3,50));
        }
    }}
}

#[test]
fn m24_ending_faults_retain_first_fault(){
    let mut r=boss_victory(EncounterId::TimeEater,[false;3]);choose(&mut r,RoomChoiceKind::Proceed);
    let id=r.decision().unwrap().room.unwrap().choices[0].id;r.state.map.as_mut().unwrap().current=None;
    let error=r.step(&GameAction::Choose(id)).unwrap_err();let RunError::Faulted(fault)=&error else{panic!("{error:?}")};assert_eq!(fault.checkpoint,"run.m24.act3.end");
    let before=r.clone();assert_eq!(r.step(&GameAction::Choose(id)),Err(error.clone()));assert_eq!(r,before,"nothing commits twice");assert_eq!(r.state.terminal,None);
}

#[test]
fn m24_fault_labels_name_m24(){
    let r=act3(EncounterId::AwakenedOne);assert_eq!(rooms::act3_end::fault_checkpoint(&r.state),"run.m24.act3.end");
    // Room-entry hook failure names M24 (the Run's room-entry label selector).
    let mut bad=act3(EncounterId::AwakenedOne);insert_raw(&mut bad,CardId::Dazed);
    let error={let map=bad.state.map.as_mut().unwrap();let target=map.nodes.iter().find(|n|n.kind==MapNodeKind::Monster&&!n.visited&&map.nodes.iter().any(|p|p.edges.contains(&n.id))).unwrap().id;
        let previous=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();previous.visited=true;bad.state.floor=previous.floor;map.current=Some(previous.id);bad.step(&GameAction::Move(target)).unwrap_err()};
    let RunError::Faulted(fault)=error else{panic!("{error:?}")};assert_eq!(fault.checkpoint,"run.m24.encounter");
    // Existing profiles keep their labels.
    let (m23,_)=pair(EncounterId::Champ,EncounterId::AwakenedOne,[7;32]);assert_eq!(rooms::act3_end::fault_checkpoint(&m23.state),"run.m9.act3.end");
}
