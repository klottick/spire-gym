use super::super::*;
use crate::{combat::rng::Partition, content::{CardId,Character,EncounterId,RelicId,PotionId,EventId}};
fn profile(a:EncounterId,b:EncounterId)->CreationProfile {
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match a{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],EncounterId::SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};
    p.seen_bosses[1]=match b{EncounterId::Champ=>vec!["Automaton","Collector"],EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],EncounterId::Collector=>vec!["Champ","Automaton"],_=>unreachable!()};p
}
fn run()->Run {Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile(EncounterId::Guardian,EncounterId::Champ),RunRng::new([7;32],Partition::Test,0),RoomScenario::M18DefectAct2).unwrap()}
#[test]
fn m18_defect_act2_profile_acquires_echo_without_foreign_fallback(){
    let mut r=run();let d=r.decision().unwrap();assert_eq!((d.player.hp,d.player.max_hp,d.player.gold),(75,75,99));assert_eq!(d.objects.len(),10);assert!(r.public_map().is_none());
    let instance=mutations::obtain_card(&mut r.state,CardId::EchoForm,0).expect("M18 Act2 boss EchoForm must be obtainable");
    mutations::upgrade_card(&mut r.state,instance).unwrap();
    for id in [CardId::Leap,CardId::IronWave,CardId::Burn,CardId::Wound]{let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,0).is_err());assert_eq!(r,before);}
}

use super::m17_defect_act1::{choose,diagnostic_room,settle_lament,cards};
// Supplied predecessor/counter/deck boundaries in this module establish owner
// behavior, never canonical traversal or the prescribed normal outcome gate.
fn act2()->Run {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    diagnostic_room(&mut r,MapNodeKind::Boss);settle_lament(&mut r);
    choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::Proceed);
    let t=choose(&mut r,RoomChoiceKind::ConfirmSkip);assert_eq!(t.reward,0);
    assert_eq!((r.state.act,r.state.floor),(2,17));assert_eq!(r.public_map().unwrap().boss,Some(EncounterId::Champ));r
}
fn victory(local:u64,kind:MapNodeKind)->Run{
    let mut r=act2();r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().combat_visits=local;
    diagnostic_room(&mut r,kind);settle_lament(&mut r);r
}
#[test]
fn m18_defect_act2_finite_offers_preserve_local_cursors_and_claim_custody(){
    use CardId::*;
    assert_eq!((M18_DEFECT_ACT2_CARDS.len(),M18_DEFECT_ACT2_RELICS.len(),M18_DEFECT_ACT2_POTIONS.len()),(29,14,5));
    for &id in M18_DEFECT_ACT2_CARDS{let mut r=run();let i=mutations::obtain_card(&mut r.state,id,0).unwrap();mutations::upgrade_card(&mut r.state,i).unwrap();assert_eq!(r.state.deck.last().unwrap().upgrades,1);}
    let rows=[[BallLightning,ColdSnap,Coolheaded],[Barrage,CompileDriver,Darkness],[Fusion,Defragment,GeneticAlgorithm],[Claw,SteamBarrier,Streamline],[Storm,Recursion,BiasedCognition]];
    for (visit,row) in rows.into_iter().enumerate(){
        let mut r=victory(visit as u64,MapNodeKind::Monster);let potion=[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][visit%3];
        let token=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Potion(potion)).unwrap().id;
        r.state.potions=vec![Some(PotionId::StrengthPotion);3];let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
        r.step(&GameAction::DiscardPotion{slot:0}).unwrap();choose(&mut r,RoomChoiceKind::Potion(potion));let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
        choose(&mut r,RoomChoiceKind::CardReward);assert_eq!(cards(&r),row);choose(&mut r,RoomChoiceKind::Card{id:row[0],upgrades:0});
    }
    for (visit,row,relic,potion) in [(0,[Capacitor,Consume,Loop],RelicId::GoldPlatedCables,PotionId::FocusPotion),(1,[Bullseye,StaticDischarge,Heatsinks],RelicId::EmotionChip,PotionId::PotionOfCapacity)]{
        let mut r=act2();r.state.gold=500;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().shop_visits=visit;diagnostic_room(&mut r,MapNodeKind::Shop);
        assert_eq!(cards(&r),row);let view=r.decision().unwrap().room.unwrap();
        for c in &view.choices{match c.kind{RoomChoiceKind::Card{..}=>assert_eq!(c.price,Some(75)),RoomChoiceKind::Relic(id)=>{assert_eq!(id,relic);assert_eq!(c.price,Some(150));},RoomChoiceKind::Potion(id)=>{assert_eq!(id,potion);assert_eq!(c.price,Some(50));},_=>()}}
        choose(&mut r,RoomChoiceKind::Relic(relic));choose(&mut r,RoomChoiceKind::Potion(potion));assert_eq!(r.state.gold,300);let before=r.clone();let token=view.choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(relic)).unwrap().id;assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);
        choose(&mut r,RoomChoiceKind::Purge);r.step(&GameAction::Cancel).unwrap();assert_eq!(r.state.gold,300);
    }
    let mut owned=act2();crate::items::acquire_relic(&mut owned.state,RelicId::GoldPlatedCables).unwrap();owned.state.gold=200;diagnostic_room(&mut owned,MapNodeKind::Shop);assert!(!owned.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::GoldPlatedCables)));
    assert_eq!(rooms::m18_content::elite_relic(0,M18_DEFECT_ACT2_RELICS),Err(RunError::Unsupported("captured Defect Act2 elite relic sequence exhausted")));
    let mut r=victory(0,MapNodeKind::Elite);assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().elite_relic_cursor,1);choose(&mut r,RoomChoiceKind::Proceed);assert!(!r.state.relics.contains(&RelicId::Lantern));
    assert_eq!(rooms::m18_content::elite_relic(0,&[RelicId::Lantern]).unwrap(),(RelicId::Pear,2));
    for owned in [vec![],M18_DEFECT_ACT2_RELICS.to_vec()]{assert_eq!(rooms::m18_content::elite_relic(8,&owned),Err(RunError::Unsupported("captured Defect Act2 elite relic sequence exhausted")));}
    let mut r=act2();r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().elite_relic_cursor=8;diagnostic_room(&mut r,MapNodeKind::Elite);
    let before=r.clone();let record=CombatRecord{visit:1,encounter:EncounterId::GremlinLeader,outcome:crate::combat::Outcome::Victory};assert!(matches!(rooms::rewards::prepare_combat(&r,&record),Err(RunError::Unsupported("captured Defect Act2 elite relic sequence exhausted"))));assert_eq!(r,before);
}
#[test]
fn m18_defect_act2_actual_questions_and_boss_children_transition_once(){
    let mut r=act2();diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(EventId::AncientWriting));
    choose(&mut r,RoomChoiceKind::AncientSimplicity);assert!(r.state.deck.iter().filter(|c|matches!(c.id,CardId::StrikeBlue|CardId::DefendBlue)).all(|c|c.upgrades==1));assert!(r.state.deck.iter().filter(|c|matches!(c.id,CardId::Zap|CardId::Dualcast)).all(|c|c.upgrades==0));
    for (gold,event) in [(50,EventId::Joust),(49,EventId::Purifier)]{let mut r=act2();r.state.gold=gold;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=1;diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(event));}
    let mut r=act2();r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=2;diagnostic_room(&mut r,MapNodeKind::Question);assert!(r.decision().unwrap().combat.is_some());
    let mut r=victory(4,MapNodeKind::Boss);choose(&mut r,RoomChoiceKind::CardReward);assert_eq!(cards(&r),vec![CardId::Electrodynamics,CardId::Amplify,CardId::EchoForm]);choose(&mut r,RoomChoiceKind::Card{id:CardId::EchoForm,upgrades:0});
    let floor=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(floor.reward,1);assert_eq!((r.state.act,r.state.floor),(2,34));choose(&mut r,RoomChoiceKind::OpenChest);
    let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.choices.iter().filter_map(|c|if let RoomChoiceKind::Relic(id)=c.kind{Some(id)}else{None}).collect::<Vec<_>>(),vec![RelicId::EmptyCage,RelicId::SlaversCollar]);
    choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::CancelSkip);assert_eq!(r.decision().unwrap().room.unwrap(),screen);
    choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));let original=r.state.deck.clone();let a=r.decision().unwrap().candidates[0].action.clone();r.step(&a).unwrap();assert_eq!(r.state.deck,original);let before=r.clone();assert!(r.step(&a).is_err());assert_eq!(r,before);let a=r.decision().unwrap().candidates[0].action.clone();r.step(&a).unwrap();assert_eq!(r.state.deck,original[2..]);
    r.state.hp=31;let total=r.state.reward_ledger.total;let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!(r.state.reward_ledger.total,total);assert_eq!(t.committed_events,vec![CommittedEvent::ActAdvanced{from:2,to:3}]);
    let d=r.decision().unwrap();assert_eq!((d.act,d.floor,d.player.hp),(3,34,d.player.max_hp));assert_eq!(d.room.unwrap().kind,RoomViewKind::Act3Ready);assert!(d.candidates.is_empty()&&d.terminal.is_none());
    let before=r.clone();assert!(r.step(&GameAction::Confirm).is_err());assert!(rooms::transition::drive(&mut r).is_err());assert_eq!(r,before);
}
#[test]
fn m18_defect_act2_matched_creation_histories_preserve_full_m17_prefix(){
    for a in [EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss]{
        for b in [EncounterId::Champ,EncounterId::BronzeAutomaton,EncounterId::Collector]{
            let p=profile(a,b);let mut old=Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},p.clone(),RunRng::new([19;32],Partition::Test,0),RoomScenario::M17DefectAct1).unwrap();
            let mut new=Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},p,RunRng::new([19;32],Partition::Test,0),RoomScenario::M18DefectAct2).unwrap();
            let equal=|old:&Run,new:&Run|{let mut tagged=old.clone();tagged.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M18DefectAct2;assert_eq!(&tagged,new,"complete matched-input prefix, including RNG");};
            equal(&old,&new);choose(&mut old,RoomChoiceKind::NeowLament);choose(&mut new,RoomChoiceKind::NeowLament);equal(&old,&new);
            diagnostic_room(&mut old,MapNodeKind::Monster);diagnostic_room(&mut new,MapNodeKind::Monster);equal(&old,&new);settle_lament(&mut old);settle_lament(&mut new);equal(&old,&new);
            choose(&mut old,RoomChoiceKind::CardReward);choose(&mut new,RoomChoiceKind::CardReward);equal(&old,&new);choose(&mut old,RoomChoiceKind::Card{id:CardId::Coolheaded,upgrades:0});choose(&mut new,RoomChoiceKind::Card{id:CardId::Coolheaded,upgrades:0});equal(&old,&new);
            choose(&mut old,RoomChoiceKind::Proceed);choose(&mut new,RoomChoiceKind::Proceed);diagnostic_room(&mut old,MapNodeKind::Boss);diagnostic_room(&mut new,MapNodeKind::Boss);equal(&old,&new);settle_lament(&mut old);settle_lament(&mut new);equal(&old,&new);
            for kind in [RoomChoiceKind::Proceed,RoomChoiceKind::Proceed]{choose(&mut old,kind.clone());choose(&mut new,kind);equal(&old,&new);}
            choose(&mut old,RoomChoiceKind::ConfirmSkip);choose(&mut new,RoomChoiceKind::ConfirmSkip);
            assert_eq!(old.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2Ready);assert_eq!(new.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
            // Only the intentional continuation screen/act-local bookkeeping
            // differs. Resources, successor map and complete RNG stay exact.
            let mut endpoint=old.clone();endpoint.state.room="Map";let room=endpoint.state.room_loop.as_mut().unwrap();room.scenario=RoomScenario::M18DefectAct2;room.act2=Some(Default::default());room.screen.kind=RoomViewKind::Map;assert_eq!(endpoint,new);
        }
    }
}
fn acquired_genetic()->(Run,u64){
    let mut r=Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile(EncounterId::Guardian,EncounterId::BronzeAutomaton),RunRng::new([7;32],Partition::Test,0),RoomScenario::M18DefectAct2).unwrap();choose(&mut r,RoomChoiceKind::NeowLament);
    r.state.combat_progress=CombatProgress{entered:2,settled:2};diagnostic_room(&mut r,MapNodeKind::Monster);settle_lament(&mut r);
    choose(&mut r,RoomChoiceKind::CardReward);choose(&mut r,RoomChoiceKind::Card{id:CardId::GeneticAlgorithm,upgrades:0});choose(&mut r,RoomChoiceKind::Proceed);
    let i=r.state.deck.last().unwrap().instance;let ids=r.state.deck.iter().filter(|c|c.instance!=i).map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}(r,i)
}
fn learned_genetic_act2()->(Run,u64){
    let (mut r,i)=acquired_genetic();diagnostic_room(&mut r,MapNodeKind::Boss);r.step(&GameAction::Combat(crate::combat::CombatAction::PlayCard{instance:i,target:None})).unwrap();assert_eq!(r.state.deck[0].misc,3);assert_eq!(r.decision().unwrap().combat.unwrap().player.block,1);
    settle_lament(&mut r);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::ConfirmSkip);(r,i)
}
#[test]
fn m18_genetic_algorithm_actual_stasis_return_cleanup_death_and_fault(){
    use crate::combat::{Actor,DamagePayload,Effect,EffectKind,Source,CombatAction};
    use crate::combat::tests::m18_custody_support as support;
    let (mut base,i)=learned_genetic_act2();
    diagnostic_room(&mut base,MapNodeKind::Boss);assert_eq!(base.decision().unwrap().combat.unwrap().hand[0].misc,3);
    support::capture_learned_genetic(base.state.combat.as_mut().unwrap(),i);
    let stasis=base.decision().unwrap().combat.unwrap().stasis;assert_eq!(stasis.len(),1);assert_eq!(stasis[0].card.misc,3);assert_eq!(stasis[0].card.instance.0,i);
    let mut accepted=base.clone();support::cleanup_preserves_accepted_player_work(accepted.state.combat.as_mut().unwrap(),i);accepted.settle_combat().unwrap();assert_eq!(accepted.state.deck[0].misc,3);
    let mut full=base.clone();support::return_learned_genetic(full.state.combat.as_mut().unwrap(),true);let d=full.decision().unwrap().combat.unwrap();assert_eq!(d.hand.len(),10);assert!(d.discard.iter().any(|c|c.instance.0==i&&c.misc==3));assert_eq!(full.state.deck[0].misc,3);
    let mut returned=base.clone();support::return_learned_genetic(returned.state.combat.as_mut().unwrap(),false);assert_eq!(returned.decision().unwrap().combat.unwrap().hand[0].instance.0,i);returned.step(&GameAction::Combat(CombatAction::PlayCard{instance:i,target:None})).unwrap();assert_eq!(returned.state.deck[0].misc,5);assert_eq!(returned.decision().unwrap().combat.unwrap().player.block,3);
    let mut fresh=base.clone();support::return_learned_genetic(fresh.state.combat.as_mut().unwrap(),false);let copy=support::fresh_genetic_copy(fresh.state.combat.as_mut().unwrap(),i);fresh.step(&GameAction::Combat(CombatAction::PlayCard{instance:copy,target:None})).unwrap();assert_eq!(fresh.state.deck[0].misc,3);assert_eq!(fresh.decision().unwrap().combat.unwrap().hand.iter().find(|c|c.instance.0==i).unwrap().misc,3);
    let mut family=base.clone();support::return_learned_genetic(family.state.combat.as_mut().unwrap(),false);let copy=support::family_genetic_copy(family.state.combat.as_mut().unwrap(),i);family.step(&GameAction::Combat(CombatAction::PlayCard{instance:i,target:None})).unwrap();assert_eq!(family.state.deck[0].misc,5);family.step(&GameAction::Combat(CombatAction::PlayCard{instance:copy,target:None})).unwrap();assert_eq!(family.state.deck[0].misc,7);assert_eq!(family.decision().unwrap().combat.unwrap().player.block,8);
    // Actual boss cleanup comes from the real CrackedCore passive against the
    // Lament-reduced boss, not a synthetic Victory or altered enemy setup.
    base.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();assert_eq!(base.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);assert_eq!(base.state.deck[0].misc,3);
    // Source-owner re-entry diagnostic on a settled Act2 boss clone; this does
    // not publish an Act3 M18 action or claim another normal room traversal.
    let mut later=base.clone();later.enter_owned(EncounterId::ThreeByrds,CombatOrigin::Training{kind:MapNodeKind::Monster}).unwrap();assert_eq!(later.decision().unwrap().combat.unwrap().hand[0].misc,3);assert_eq!(later.state.deck[0].misc,3);
    let stale=returned.clone();assert!(returned.step(&GameAction::Combat(CombatAction::PlayCard{instance:i,target:None})).is_err());assert_eq!(returned,stale);
    let mut dead=returned.clone();let hp=dead.decision().unwrap().player.hp as i32;dead.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp))}],None).unwrap();dead.settle_combat().unwrap();assert_eq!(dead.state.terminal,Some(RunTerminal::Death));assert_eq!(dead.state.deck[0].misc,5);
    let mut held=full.clone();support::capture_returned_genetic(held.state.combat.as_mut().unwrap(),i);let hp=held.decision().unwrap().player.hp as i32;held.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp))}],None).unwrap();held.settle_combat().unwrap();assert_eq!(held.state.terminal,Some(RunTerminal::Death));assert_eq!(held.state.deck[0].misc,3);
    let (mut fault,i)=learned_genetic_act2();fault.state.map.as_mut().unwrap().regular_encounter_history=vec![EncounterId::SphericGuardian];diagnostic_room(&mut fault,MapNodeKind::Monster);crate::combat::tests::m16_support::exhaust_juggernaut_rng(fault.state.combat.as_mut().unwrap());assert!(matches!(fault.step(&GameAction::Combat(CombatAction::PlayCard{instance:i,target:None})),Err(RunError::Faulted(_))));assert_eq!(fault.state.deck[0].misc,5);let before=fault.clone();assert!(fault.step(&GameAction::Combat(CombatAction::PlayCard{instance:i,target:None})).is_err());assert_eq!(fault,before);
}

#[test]
fn m18_defect_act2_room_acquired_items_use_shared_scheduler(){
    let mut r=act2();r.state.gold=1000;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().elite_relic_cursor=3;
    diagnostic_room(&mut r,MapNodeKind::Elite);settle_lament(&mut r);choose(&mut r,RoomChoiceKind::Relic(RelicId::DataDisk));choose(&mut r,RoomChoiceKind::Proceed);
    for (relic,potion) in [(RelicId::GoldPlatedCables,PotionId::FocusPotion),(RelicId::EmotionChip,PotionId::PotionOfCapacity)]{
        diagnostic_room(&mut r,MapNodeKind::Shop);choose(&mut r,RoomChoiceKind::Relic(relic));choose(&mut r,RoomChoiceKind::Potion(potion));choose(&mut r,RoomChoiceKind::Leave);
    }
    assert_eq!(r.state.gold,600);let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}
    let loop_i=mutations::obtain_card(&mut r.state,CardId::Loop,0).unwrap();let discharge_i=mutations::obtain_card(&mut r.state,CardId::StaticDischarge,0).unwrap();
    diagnostic_room(&mut r,MapNodeKind::Monster);assert_eq!(r.decision().unwrap().combat.unwrap().character.focus,1);
    use crate::combat::CombatAction;
    for slot in [0,1]{r.step(&GameAction::Combat(CombatAction::UsePotion{slot,target:None})).unwrap();let before=r.clone();assert!(r.step(&GameAction::Combat(CombatAction::UsePotion{slot,target:None})).is_err());assert_eq!(r,before);}
    for instance in [loop_i,discharge_i]{r.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();}
    let mut blocked=r.clone();crate::combat::tests::m18_custody_support::acquired_item_scheduler(blocked.state.combat.as_mut().unwrap(),true);crate::combat::tests::m18_custody_support::acquired_item_scheduler(r.state.combat.as_mut().unwrap(),false);
}

#[test]
fn m18_defect_act2_initial_contract_and_all_actual_captures(){
    for (character,ascension,full) in [(Character::Silent,0,false),(Character::Ironclad,0,false),(Character::Defect,1,false),(Character::Defect,0,true)]{
        let mut p=profile(EncounterId::Guardian,EncounterId::Champ);p.neow_eligible=full;assert_eq!(Run::new_room_scenario(RunConfig{character,ascension},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M18DefectAct2),Err(RunError::Unsupported("M18 requires Defect A0 short Neow")));
    }
    use EncounterId::*;
    for (kind,boss,history,expected) in [(MapNodeKind::Monster,Champ,0,SphericGuardian),(MapNodeKind::Monster,Champ,1,ThreeByrds),(MapNodeKind::Monster,Champ,2,CenturionAndHealer),(MapNodeKind::Monster,Champ,3,Snecko),(MapNodeKind::Monster,Champ,4,ChosenAndByrds),(MapNodeKind::Monster,Champ,5,ThreeCultists),(MapNodeKind::Elite,Champ,0,GremlinLeader),(MapNodeKind::Elite,BronzeAutomaton,0,Slavers),(MapNodeKind::Elite,Collector,0,BookOfStabbing),(MapNodeKind::Boss,Champ,0,Champ),(MapNodeKind::Boss,BronzeAutomaton,0,BronzeAutomaton),(MapNodeKind::Boss,Collector,0,Collector)]{
        let mut r=Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile(Guardian,boss),RunRng::new([7;32],Partition::Test,0),RoomScenario::M18DefectAct2).unwrap();choose(&mut r,RoomChoiceKind::NeowLament);
        // Supplied Act1 visit counts prove the shared transition resets Act2.
        let p=r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap();p.shop_visits=4;p.elite_relic_cursor=5;
        diagnostic_room(&mut r,MapNodeKind::Boss);settle_lament(&mut r);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::ConfirmSkip);
        let p=r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap();assert_eq!((p.shop_visits,p.elite_relic_cursor,p.combat_visits),(0,0,0));
        assert_eq!(r.public_map().unwrap().boss,Some(boss));r.state.map.as_mut().unwrap().regular_encounter_history=vec![SphericGuardian;history];diagnostic_room(&mut r,kind);assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,expected);
        assert!(crate::combat::enemies::A0_ENCOUNTER_SETUPS.iter().any(|(id,act,_)|*id==expected&&*act==2));let d=r.decision().unwrap();assert!(d.player.relics.contains(&RelicId::CrackedCore));assert!(r.state.relics.is_empty());assert!(d.combat.unwrap().enemies.iter().all(|e|e.hp==1));
    }
}
#[test]
fn m18_defect_act2_transition_fault_retains_heal_once_and_joust_retains_payment(){
    let mut r=victory(0,MapNodeKind::Boss);choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::OpenChest);choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r.state.hp=31;
    assert_eq!(r.decision().unwrap().player.hp,31,"true pre-heal child checkpoint");
    r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:2,to:4,floor:34,reset_applied:false,heal_applied:false,initialized:false});assert!(matches!(rooms::transition::drive(&mut r),Err(RunError::Faulted(_))));assert_eq!(r.state.hp,r.state.max_hp);let before=r.clone();assert!(rooms::transition::drive(&mut r).is_err());assert_eq!(r,before);
    let mut r=act2();r.state.gold=100;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=1;diagnostic_room(&mut r,MapNodeKind::Question);let token=r.decision().unwrap().room.unwrap().choices[0].id;
    choose(&mut r,RoomChoiceKind::JoustMurderer);assert_eq!(r.state.gold,50);let before=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert_eq!(r,before);choose(&mut r,RoomChoiceKind::JoustContinue);
    let row=r.decision().unwrap().room.unwrap().choices[0].clone();let winner=if let RoomChoiceKind::JoustResult{owner_wins}=row.kind{owner_wins}else{panic!()};choose(&mut r,row.kind);assert_eq!(r.state.gold,if winner{50}else{150});let before=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,before);
}
