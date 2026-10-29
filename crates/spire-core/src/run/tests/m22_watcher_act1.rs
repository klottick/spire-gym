use super::super::*;
use crate::{combat::rng::Partition,content::{CardId,Character,RelicId}};
fn rng()->RunRng{RunRng::new([7;32],Partition::Test,0)}
fn profile()->CreationProfile{let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];p}
fn start()->Run{Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},profile(),rng(),RoomScenario::M22WatcherAct1).unwrap()}
#[test]
fn m22_start_uses_canonical_resources_and_real_short_neow(){
    let mut run=start();assert_eq!((run.state.hp,run.state.max_hp,run.state.gold),(72,72,99));
    assert_eq!(run.state.deck.len(),10);assert!(run.state.deck.iter().all(|c|c.instance>0&&c.upgrades==0&&c.misc==0));
    assert_eq!(run.state.deck.iter().map(|c|c.instance).collect::<std::collections::BTreeSet<_>>().len(),10);
    for (id,n) in [(CardId::StrikePurple,4),(CardId::DefendPurple,4),(CardId::Eruption,1),(CardId::Vigilance,1)]{assert_eq!(run.state.deck.iter().filter(|c|c.id==id).count(),n);}
    assert_eq!(run.state.relics,vec![RelicId::PureWater]);assert_eq!(run.state.potions,vec![None;3]);assert_eq!(run.state.keys,[false;3]);assert!(run.state.map.is_none());
    let d=run.decision().unwrap();let id=d.room.unwrap().choices.iter().find(|r|r.kind==RoomChoiceKind::NeowMaxHp{amount:7}).unwrap().id;
    run.step(&GameAction::Choose(id)).unwrap();assert_eq!((run.state.hp,run.state.max_hp),(79,79));assert_eq!(run.state.map.as_ref().unwrap().boss_icon,Some(crate::content::EncounterId::Guardian));
    let mut lament=start();let id=lament.decision().unwrap().room.unwrap().choices.iter().find(|r|r.kind==RoomChoiceKind::NeowLament).unwrap().id;lament.step(&GameAction::Choose(id)).unwrap();assert!(lament.state.relics.contains(&RelicId::NeowsLament));assert_eq!(lament.state.relic_counters.get(&RelicId::NeowsLament),Some(&3));
    for c in [Character::Silent,Character::Defect,Character::Ironclad]{assert!(Run::new_room_scenario(RunConfig{character:c,ascension:0},profile(),rng(),RoomScenario::M22WatcherAct1).is_err());}
    assert!(Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:1},profile(),rng(),RoomScenario::M22WatcherAct1).is_err());
    let mut full=profile();full.neow_eligible=true;assert!(Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},full,rng(),RoomScenario::M22WatcherAct1).is_err());
}
#[test]
fn m22_holy_water_replaces_starter_once_and_rejects_inconsistent_sources(){
    let mut run=Run::new(RunConfig{character:Character::Watcher,ascension:0},profile(),rng()).unwrap();
    crate::items::acquire_relic(&mut run.state,RelicId::Vajra).unwrap();let deck=run.state.deck.clone();let belt=run.state.potions.clone();
    crate::items::acquire_relic(&mut run.state,RelicId::HolyWater).unwrap();
    assert_eq!(run.state.relics,vec![RelicId::HolyWater,RelicId::Vajra]);assert_eq!(run.state.deck,deck);assert_eq!(run.state.potions,belt);
    let before=run.clone();assert!(crate::items::acquire_relic(&mut run.state,RelicId::HolyWater).is_err());assert!(run==before);
    for character in [Character::Silent,Character::Defect,Character::Ironclad]{let mut wrong=Run::new(RunConfig{character,ascension:0},profile(),rng()).unwrap();let before=wrong.clone();assert!(crate::items::acquire_relic(&mut wrong.state,RelicId::HolyWater).is_err());assert!(wrong==before);}
}
fn choose(r:&mut Run,kind:RoomChoiceKind)->RunTransition{super::m17_defect_act1::choose(r,kind)}
// Supplied visit/history and map position; actual shared entry and Lament settlement.
fn victory(visit:u64,kind:MapNodeKind)->Run{
    let mut r=start();choose(&mut r,RoomChoiceKind::NeowLament);
    r.state.combat_progress=CombatProgress{entered:visit,settled:visit};
    super::m17_defect_act1::diagnostic_room(&mut r,kind);
    super::m17_defect_act1::settle_lament(&mut r);
    assert_eq!(r.state.last_combat.as_ref().unwrap().visit,visit);r
}
#[test]
fn m22_reward_rows_use_full_visit_not_legacy_modulo_three(){
    use CardId::*;use crate::content::PotionId::*;
    let rows=[[EmptyFist,MentalFortress,CarveReality],[CutThroughFate,Rushdown,DeceiveReality],[ThirdEye,Foresight,Meditate],[JustLucky,Nirvana,Perseverance],[FlurryOfBlows,Weave,SandsOfTime],[Prostrate,Worship,WindmillStrike]];
    let potions=[BlockPotion,EnergyPotion,DexterityPotion,AncientPotion,StancePotion,Ambrosia,BottledMiracle];
    for visit in 0..14{
        let mut r=victory(visit,MapNodeKind::Monster);
        let d=r.decision().unwrap();assert!(d.room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(potions[(visit%7) as usize])));
        choose(&mut r,RoomChoiceKind::CardReward);
        assert_eq!(super::m17_defect_act1::cards(&r),rows[(visit%6) as usize]);
        assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().filter(|c|c.kind==RoomChoiceKind::Skip).count(),1);
        choose(&mut r,RoomChoiceKind::Card{id:rows[(visit%6) as usize][0],upgrades:0});
        assert_eq!(r.state.deck.last().unwrap().id,rows[(visit%6) as usize][0]);
    }
    let mut boss=victory(5,MapNodeKind::Boss);choose(&mut boss,RoomChoiceKind::CardReward);assert_eq!(super::m17_defect_act1::cards(&boss),vec![Brilliance,Devotion,Establishment]);
}
#[test]
fn m22_shop_rows_and_destination_visits_are_exact(){
    use CardId::*;use crate::content::PotionId::*;
    for (visit,cards,relic,potion) in [(0,[BattleHymn,LikeWater,MasterReality],RelicId::GoldenEye,StancePotion),(1,[MentalFortress,Meditate,Establishment],RelicId::TeardropLocket,BottledMiracle),(2,[MentalFortress,Meditate,Establishment],RelicId::TeardropLocket,BottledMiracle)]{
        let mut r=start();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});r.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().shop_visits=visit;
        super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Shop);
        assert_eq!(super::m17_defect_act1::cards(&r),cards);
        let room=r.decision().unwrap().room.unwrap();for (card,price) in cards.into_iter().zip([75,75,150]){assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:card,upgrades:0}&&c.price==Some(price)));}
        assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(relic)&&c.price==Some(150)));
        assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(potion)&&c.price==Some(50)));
        assert_eq!(r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().shop_visits,visit+1);
        let before=r.clone();let disabled=room.choices.iter().find(|c|c.kind==RoomChoiceKind::Card{id:cards[2],upgrades:0}).unwrap().id;assert!(r.step(&GameAction::Choose(disabled)).is_err());assert!(r==before);
        choose(&mut r,RoomChoiceKind::Card{id:cards[0],upgrades:0});assert_eq!(r.state.gold,24);
    }
}
#[test]
fn m22_holy_water_opens_three_before_draw_using_shared_startup(){
    let mut r=start();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});crate::items::acquire_relic(&mut r.state,RelicId::HolyWater).unwrap();
    let deck=r.state.deck.clone();super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Monster);
    let c=r.decision().unwrap().combat.unwrap();assert_eq!(c.hand.len(),8);assert_eq!(c.hand.iter().filter(|c|c.id==CardId::Miracle).count(),3);assert_eq!(c.draw_count,5);assert!(c.discard.is_empty());assert_eq!(r.state.deck,deck);
}
// Constructed acquisition/map position; actual shared Run entry, not a policy episode.
#[test]
fn m22_anchor_startup_waits_for_bottom_block_and_keeps_detached_resources(){
    for holy in [false,true]{for anchor_first in [false,true]{
        let mut r=start();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
        if holy{crate::items::acquire_relic(&mut r.state,RelicId::HolyWater).unwrap();}
        let others=[RelicId::Vajra,RelicId::Lantern,RelicId::BloodVial];
        let mut acquired=others.to_vec();if anchor_first{acquired.insert(0,RelicId::Anchor)}else{acquired.push(RelicId::Anchor)}
        for id in acquired{crate::items::acquire_relic(&mut r.state,id).unwrap();}
        r.state.hp=48;let deck=r.state.deck.clone();let relics=r.state.relics.clone();let rng=r.state.rng.clone();
        super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Monster);
        let d=r.decision().expect("Anchor bottom action must finish before the opening Decision");let c=d.combat.unwrap();
        assert_eq!((c.player.hp,c.player.block,c.player.energy),(50,10,4));
        assert_eq!(c.hand.len(),if holy{8}else{6});assert_eq!(c.hand.iter().filter(|c|c.id==CardId::Miracle).count(),if holy{3}else{1});
        assert_eq!(c.player.powers.iter().find(|p|p.id.0=="strength").unwrap().amount,1);
        assert_eq!(r.state.deck,deck);assert!(r.state.relics.is_empty()&&r.state.potions.is_empty());
        assert_eq!(r.state.combat.as_ref().unwrap().item_context().unwrap().relics(),relics);
        assert_ne!(r.state.rng,rng,"the real shared entry owns encounter RNG");
        let before=r.clone();let duplicate=crate::combat::HookEvent{source:crate::combat::Source::None,target:None,hook:crate::combat::Hook::CombatStarted};
        assert_eq!(r.state.combat.as_mut().unwrap().enqueue_hook(duplicate),Err(RunError::InvalidAction));assert!(r==before);
    }}
}
#[test]
fn m22_holy_water_full_opening_captures_three_and_preserves_permanent_census(){
    use crate::{combat::{self,Combat,CardInstanceId},items::CombatItemContext};
    for (relic,generated,remaining) in [(RelicId::PureWater,1,1),(RelicId::HolyWater,3,3)]{
        let entry=combat::enemies::setup(crate::content::EncounterId::JawWorm,1,0,&mut rng(),combat::enemies::SetupContext{
            player:combat::PlayerState{hp:72,max_hp:72,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Watcher,
            cards:(1..=10).map(|id|combat::permanent_live_card(CardId::DefendPurple,CardInstanceId(id),0)).collect(),opening_draw:10,encounter_visit:0,
        }).unwrap();let mut items=Some(CombatItemContext::new(vec![relic],vec![None;3]).with_m22_profile());
        let c=Combat::enter_with_item_context(entry,&mut items).unwrap();let v=c.public_full().unwrap();assert_eq!(v.hand.len(),10);assert_eq!(v.draw_count,remaining);assert_eq!(v.hand.iter().filter(|c|c.id==CardId::Miracle).count(),generated);assert!(v.discard.is_empty());
        assert_eq!(v.hand.iter().map(|c|c.instance).collect::<std::collections::BTreeSet<_>>().len(),10);
    }
}
#[test]
fn m22_startup_and_exit_keep_power_generated_history_and_resource_custody(){
    use crate::combat::{CombatAction,Stance};use crate::content::EncounterId;
    let mut r=start();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
    r.state.deck=vec![CardInstance{instance:31,id:CardId::MentalFortress,upgrades:0,misc:0},CardInstance{instance:32,id:CardId::Prostrate,upgrades:1,misc:0}];r.state.next_instance=33;
    crate::items::acquire_relic(&mut r.state,RelicId::TeardropLocket).unwrap();let original=r.state.deck.clone();let relics=r.state.relics.clone();
    super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Monster);
    assert_eq!(r.decision().unwrap().combat.unwrap().character.stance,Stance::Calm);
    r.step(&GameAction::Combat(CombatAction::PlayCard{instance:31,target:None})).unwrap();r.step(&GameAction::Combat(CombatAction::PlayCard{instance:32,target:None})).unwrap();
    assert_eq!(r.decision().unwrap().combat.unwrap().mantra_gained,Some(3));
    // Supplied lethal shared effect, not a played policy win or normal witness.
    super::m9_boss::hit(&mut r,0,999);r.settle_combat().unwrap();assert_eq!(r.state.deck,original);assert_eq!(r.state.relics,relics);
    assert_eq!(r.state.combat_progress,CombatProgress{entered:1,settled:1});let before=r.clone();assert!(r.step(&GameAction::Combat(CombatAction::EndTurn)).is_err());assert!(r==before);
    let node=r.state.map.as_ref().unwrap().current.unwrap();r.enter_owned(EncounterId::JawWorm,CombatOrigin::MapRoom{node,kind:MapNodeKind::Monster}).unwrap();
    let c=r.decision().unwrap().combat.unwrap();assert_eq!((c.character.stance,c.mantra_gained),(Stance::Calm,Some(0)));assert_eq!(c.hand.iter().filter(|c|c.id==CardId::Miracle).count(),1);assert_eq!(r.state.deck,original);
}
#[test]
fn m22_boss_exchange_and_transition_use_real_children_and_healthy_cutoff(){
    use diagnostics::{M22Diagnostic as D,M22DiagnosticBoss as B,M22DiagnosticBranch as R};
    for boss in [B::Guardian,B::Hexaghost,B::SlimeBoss]{for branch in [R::HolyWater,R::VioletLotus,R::SacredBark,R::EmptyCage,R::SlaversCollar,R::Skip]{
        let mut r=diagnostics::m22_diagnostic(D::BossRelicMenu{boss}).unwrap();
        assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().filter_map(|c|if let RoomChoiceKind::Relic(id)=c.kind{Some(id)}else{None}).collect::<Vec<_>>(),vec![RelicId::HolyWater,RelicId::VioletLotus,RelicId::SacredBark,RelicId::EmptyCage,RelicId::SlaversCollar]);
        let menu=r.decision().unwrap().room.unwrap();choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::CancelSkip);assert_eq!(r.decision().unwrap().room.unwrap(),menu);
        let r=diagnostics::m22_diagnostic(D::Act2Ready{boss,branch}).unwrap();
        assert_eq!((r.state.act,r.state.floor,r.state.hp),(2,17,79));assert_eq!(r.state.terminal,None);assert!(r.decision().unwrap().candidates.is_empty());assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2Ready);
        let t=r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap();assert!(t.reset_applied&&t.heal_applied&&t.initialized);assert!(r.state.map.as_ref().is_some_and(|m|m.act==2));
        assert_eq!(r.state.deck.len(),if branch==R::EmptyCage{8}else{10});assert_eq!(r.state.potions[0],Some(crate::content::PotionId::BottledMiracle));
        assert_eq!(r.state.relics.first(),Some(&if branch==R::HolyWater{RelicId::HolyWater}else{RelicId::PureWater}));
        assert_eq!(r.state.history.events.iter().filter(|e|matches!(e,CommittedEvent::ActAdvanced{from:1,to:2})).count(),1);
        let mut r=r;let before=r.clone();assert!(r.step(&GameAction::Move(NodeId(0))).is_err());assert!(r==before);
    }}
}
#[test]
fn m22_transition_and_startup_faults_retain_actual_committed_prefix(){
    use diagnostics::M22Diagnostic as D;
    for fixture in [D::TransitionResetFault,D::TransitionHealFault]{let mut r=diagnostics::m22_diagnostic(fixture).unwrap();let f=r.state.fault.clone().unwrap();assert_eq!(r.state.terminal,None);assert!(r.state.relics.contains(&RelicId::HolyWater));
        let t=r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap();assert!(t.reset_applied&&!t.initialized);assert_eq!(t.heal_applied,fixture==D::TransitionHealFault);
        let before=r.clone();assert_eq!(r.step(&GameAction::Confirm),Err(RunError::Faulted(f)));assert!(r==before);
    }
    let mut r=diagnostics::m22_diagnostic(D::BeforeStartupFault).unwrap();let action=r.decision().unwrap().candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();
    assert!(matches!(r.step(&action),Err(RunError::Faulted(_))));assert!(r.state.relics.is_empty());let c=r.state.combat.as_ref().unwrap();let exit=c.snapshot();
    assert!(r.state.active_combat.is_some());assert_eq!(r.state.combat_progress,CombatProgress{entered:1,settled:0});assert!(format!("{exit:?}").contains("Miracle"));
    let before=r.clone();let fault=r.state.fault.clone();assert!(matches!(r.step(&action),Err(RunError::Faulted(_))));assert!(r==before);assert_eq!(r.state.fault,fault);
}
#[test]
fn m22_diagnostic_modal_states_are_actual_shared_choices(){
    use diagnostics::M22Diagnostic as D;use crate::combat::SelectionOperation;
    for (fixture,operation) in [(D::ScryChoice,SelectionOperation::Scry),(D::MeditateChoice,SelectionOperation::MeditateReturn)]{
        let r=diagnostics::m22_diagnostic(fixture).unwrap();let c=r.decision().unwrap().combat.unwrap();assert_eq!(c.selection.unwrap().operation,operation);assert!(r.state.active_combat.is_some());
    }
    let r=diagnostics::m22_diagnostic(D::StanceChoice).unwrap();assert!(r.decision().unwrap().combat.unwrap().stance_choice.is_some());
}
#[test]
fn m22_readonly_diagnostic_checkpoint_reports_exact_fault_prefix_and_cannot_mutate_owner(){
    use diagnostics::{M22Diagnostic as D,M22DiagnosticZone as Z,m22_diagnostic_checkpoint as snap};
    for fixture in [D::TransitionResetFault,D::TransitionHealFault]{let r=diagnostics::m22_diagnostic(fixture).unwrap();let before=r.clone();let x=snap(&r).unwrap();let t=x.transition.unwrap();assert!(t.reset_applied&&!t.initialized);assert_eq!(t.heal_applied,fixture==D::TransitionHealFault);assert!(x.run_fault.is_some());assert!(r==before);}
    let mut r=diagnostics::m22_diagnostic(D::BeforeStartupFault).unwrap();let action=r.decision().unwrap().candidates[0].action.clone();assert!(matches!(r.step(&action),Err(RunError::Faulted(_))));let before=r.clone();let x=snap(&r).unwrap();assert!(x.run_relics.is_empty());assert!(x.run_potions.is_empty());let c=x.combat.unwrap();let hand=c.cards.iter().filter(|c|c.zone==Z::Hand).collect::<Vec<_>>();assert_eq!(hand.len(),1);assert_eq!((hand[0].id,hand[0].instance.0),(CardId::Miracle,u64::MAX));assert!(matches!(hand[0].origin,crate::combat::CombatCardRef::CombatLocal(_)));assert_eq!(c.draw_count,10);assert_eq!(c.inventory_relics,vec![RelicId::HolyWater]);assert_eq!(c.pending_opening_hand,Some(5));assert_eq!(c.current_creation.as_ref().map(|c|(c.id,c.count,c.next)),Some((CardId::Miracle,3,1)));assert!(c.fault.is_some());assert!(r==before);
}
#[test]
fn m22_producer_union_has_exact_taxonomy_and_actual_permanent_mutations(){
    use crate::content::{Catalog,CardRarity};let catalog=Catalog::standard();let mut r=start();
    let mut counts=[0;4];for &id in M22_WATCHER_ACT1_CARDS{let rarity=catalog.card(id).unwrap().rarity;counts[match rarity{CardRarity::Basic=>0,CardRarity::Common=>1,CardRarity::Uncommon=>2,CardRarity::Rare=>3,_=>panic!("unexpected rarity")}]+=1;let instance=mutations::obtain_card(&mut r.state,id,0).unwrap();mutations::upgrade_card(&mut r.state,instance).unwrap();assert_eq!(r.state.deck.last().unwrap().upgrades,1);assert!(!mutations::can_upgrade(&r.state,r.state.deck.last().unwrap()));mutations::remove_card(&mut r.state,instance).unwrap();}
    assert_eq!(counts,[4,6,14,4]);assert_eq!(M22_WATCHER_ACT1_RELICS.len(),18);assert_eq!(M22_WATCHER_ACT1_POTIONS.len(),7);
    assert_eq!(rooms::m22_content::RELIC_TAXONOMY.iter().map(|(id,_)|*id).collect::<Vec<_>>(),M22_WATCHER_ACT1_RELICS);
    for id in [CardId::Strike,CardId::DefendGreen,CardId::Zap,CardId::Miracle,CardId::Smite,CardId::Safety,CardId::Dazed,CardId::Burn,CardId::Slimed]{let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,0).is_err());assert!(r==before);}
}
#[test]
fn m22_initial_history_selects_twelve_actual_shared_setups_and_keeps_core_future_histories(){
    use crate::content::EncounterId::*;
    let cases=[(MapNodeKind::Monster,0,Guardian,Cultist),(MapNodeKind::Monster,1,Guardian,JawWorm),(MapNodeKind::Monster,2,Guardian,TwoLouse),(MapNodeKind::Monster,3,Guardian,LargeSlime),(MapNodeKind::Monster,4,Guardian,ThreeLouse),(MapNodeKind::Monster,5,Guardian,LotsOfSlimes),(MapNodeKind::Elite,0,Guardian,GremlinNob),(MapNodeKind::Elite,0,Hexaghost,Lagavulin),(MapNodeKind::Elite,0,SlimeBoss,ThreeSentries),(MapNodeKind::Boss,0,Guardian,Guardian),(MapNodeKind::Boss,0,Hexaghost,Hexaghost),(MapNodeKind::Boss,0,SlimeBoss,SlimeBoss)];
    for (kind,history,boss,expected) in cases{let mut p=profile();p.seen_bosses[0]=match boss{Guardian=>vec!["Hexaghost","SlimeBoss"],Hexaghost=>vec!["Guardian","SlimeBoss"],_=>vec!["Guardian","Hexaghost"]};
        // Core matched future-history admission, not an M23 equality/continuation proof.
        p.seen_bosses[1]=vec!["Automaton","Collector"];p.seen_bosses[2]=vec!["DonuAndDeca","TimeEater"];let future=[p.seen_bosses[1].clone(),p.seen_bosses[2].clone()];
        let mut r=Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,rng(),RoomScenario::M22WatcherAct1).unwrap();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});assert_eq!(r.public_map().unwrap().boss,Some(boss));assert_eq!([r.state.seen_bosses[1].clone(),r.state.seen_bosses[2].clone()],future);
        r.state.map.as_mut().unwrap().regular_encounter_history=vec![Cultist;history];super::m17_defect_act1::diagnostic_room(&mut r,kind);assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,expected);assert!(crate::combat::enemies::A0_ENCOUNTER_SETUPS.iter().any(|(id,act,_)|*id==expected&&*act==1));assert!(r.state.combat.as_ref().unwrap().item_context().unwrap().m22_profile());
    }
}
#[test]
fn m22_retained_growth_and_played_power_settle_then_reset_in_the_same_run(){
    use crate::combat::CombatAction;use crate::content::EncounterId;
    let mut r=start();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
    // Fixed supplied subset, real accepted plays/turn/callbacks and shared settlement.
    r.state.deck=[CardId::Establishment,CardId::WindmillStrike,CardId::Perseverance,CardId::SandsOfTime].into_iter().enumerate().map(|(i,id)|CardInstance{instance:i as u64+1,id,upgrades:0,misc:0}).collect();r.state.next_instance=5;let original=r.state.deck.clone();
    super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Monster);r.step(&GameAction::Combat(CombatAction::PlayCard{instance:1,target:None})).unwrap();r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();let c=r.decision().unwrap().combat.unwrap();assert_eq!(c.played_powers[0].instance.0,1);
    let wind=c.hand.iter().find(|c|c.id==CardId::WindmillStrike).unwrap();assert_eq!(wind.values.as_ref().unwrap().base_damage,Some(11));
    let block=c.hand.iter().find(|c|c.id==CardId::Perseverance).unwrap();assert_eq!(block.values.as_ref().unwrap().base_block,Some(7));
    let sands=c.hand.iter().find(|c|c.id==CardId::SandsOfTime).unwrap();assert_eq!((sands.watcher.as_ref().unwrap().combat_cost,sands.effective_cost),(Some(2),Some(2)));
    super::m9_boss::hit(&mut r,0,999);r.settle_combat().unwrap();assert_eq!(r.state.deck,original);assert_eq!(r.state.combat_progress,CombatProgress{entered:1,settled:1});
    let node=r.public_map().unwrap().current.unwrap();r.enter_owned(EncounterId::JawWorm,CombatOrigin::MapRoom{node,kind:MapNodeKind::Monster}).unwrap();let c=r.decision().unwrap().combat.unwrap();assert!(c.played_powers.is_empty());assert_eq!(c.hand.iter().find(|c|c.id==CardId::WindmillStrike).unwrap().values.as_ref().unwrap().base_damage,Some(7));assert_eq!(c.hand.iter().find(|c|c.id==CardId::SandsOfTime).unwrap().effective_cost,Some(4));
}
#[test]
fn m22_empty_chest_and_small_cage_keep_real_shared_choices(){
    let mut r=start();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});for id in rooms::m22_content::CHEST_RELICS{crate::items::acquire_relic(&mut r.state,id).unwrap();}super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Treasure);let before=r.state.rng.clone();choose(&mut r,RoomChoiceKind::OpenChest);assert_eq!(r.state.rng,before);assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().map(|r|r.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Sapphire,RoomChoiceKind::Proceed]);
    use diagnostics::{M22Diagnostic as D,M22DiagnosticBoss as B};
    for count in 0..=2{let mut r=diagnostics::m22_diagnostic(D::BossRelicMenu{boss:B::Guardian}).unwrap();r.state.deck.truncate(count);choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));if r.state.terminal.is_none(){choose(&mut r,RoomChoiceKind::Proceed);}assert_eq!(r.state.terminal,Some(RunTerminal::EmptyDeck));assert!(r.state.deck.is_empty());assert_eq!(r.state.act,1);assert!(r.state.room_loop.as_ref().unwrap().transition.is_none());}
}
#[test]
fn m22_fixed_initial_resources_and_rng_are_frozen_without_policy_actions(){
    use diagnostics::{M22DiagnosticBoss as B,m22_supplied_boss};
    let directory=std::env::var_os("M22_INITIAL_RESOURCES_DIR").map(std::path::PathBuf::from);if let Some(p)=&directory{std::fs::create_dir_all(p).unwrap();}
    let write=|name:&str,r:&Run,provenance:&str|{if let Some(p)=&directory{use std::io::Write;let value=serde_json::json!({"scope":"initialization-only; zero policy actions","provenance":provenance,"full_state_debug":format!("{:?}",r.state),"next_choice":r.next_choice,"checkpoint_debug":format!("{:?}",diagnostics::m22_diagnostic_checkpoint(r).unwrap()),"public_debug":format!("{:?}",r.decision()),"map_debug":format!("{:?}",r.public_map())});let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(p.join(format!("{name}.json"))).unwrap();f.write_all(&serde_json::to_vec_pretty(&value).unwrap()).unwrap();f.sync_all().unwrap();}};
    for seed in [7u64,11]{for boss in [B::Guardian,B::Hexaghost,B::SlimeBoss]{let mut p=profile();p.seen_bosses[0]=match boss{B::Guardian=>vec!["Hexaghost","SlimeBoss"],B::Hexaghost=>vec!["Guardian","SlimeBoss"],B::SlimeBoss=>vec!["Guardian","Hexaghost"]};let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());let random=RunRng::new(root,Partition::Test,0);let r=Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,random.clone(),RoomScenario::M22WatcherAct1).unwrap();assert_eq!(r.state.rng,random);assert_eq!((r.state.hp,r.state.max_hp,r.state.gold),(72,72,99));assert_eq!(r.state.deck.len(),10);write(&format!("normal-{seed}-{boss:?}"),&r,"canonical short-Neow; prescribed input only");}}
    for boss in [B::Guardian,B::Hexaghost,B::SlimeBoss]{let r=m22_supplied_boss(boss).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold),(72,72,99));assert_eq!(r.state.relics,vec![RelicId::PureWater,RelicId::VioletLotus,RelicId::Vajra,RelicId::OddlySmoothStone]);assert_eq!(r.state.deck.len(),16);for (i,c) in r.state.deck.iter().enumerate(){assert_eq!((c.instance,c.upgrades,c.misc),(i as u64+1,1,0));assert_eq!(c.id,[CardId::Eruption,CardId::Vigilance,CardId::EmptyFist,CardId::CarveReality][i/4]);}assert!(r.state.last_combat.is_none()&&r.state.combat.is_none()&&r.state.active_combat.is_none());write(&format!("supplied-7-{boss:?}"),&r,"predeclared conditional fixed16; no earned path, no prior record");}
}
#[test]
fn m22_actual_elite_settlement_filters_detached_owned_inventory_without_capture_request(){
    let mut r=start();choose(&mut r,RoomChoiceKind::NeowLament);for id in rooms::m22_content::ELITE_RELICS{crate::items::acquire_relic(&mut r.state,id).unwrap();}
    super::m17_defect_act1::diagnostic_room(&mut r,MapNodeKind::Elite);assert!(r.state.relics.is_empty());assert!(rooms::m22_content::ELITE_RELICS.iter().all(|id|r.state.combat.as_ref().unwrap().item_context().unwrap().relics().contains(id)));
    let node=r.public_map().unwrap().current.unwrap();let mut expected=r.state.rng.clone();expected.bounded_scoped(RunRngScope::CombatGoldReward,&[1,node.0,0,MapNodeKind::Elite as u64],11);
    super::m17_defect_act1::settle_lament(&mut r);assert_eq!(r.state.rng,expected);assert!(r.state.last_combat.as_ref().is_some_and(|c|c.outcome==crate::combat::Outcome::Victory));assert!(!r.decision().unwrap().room.unwrap().choices.iter().any(|r|matches!(r.kind,RoomChoiceKind::Relic(_))));
    assert!(rooms::m22_content::ELITE_RELICS.iter().all(|id|r.state.relics.contains(id)));
}
#[test]
fn m22_actual_reward_claims_retain_the_parent_and_full_belt_item_without_regeneration(){
    let mut r=victory(1,MapNodeKind::Monster);r.state.potions=vec![Some(crate::content::PotionId::AncientPotion);3];let parent=r.decision().unwrap().room.unwrap();let potion=parent.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Potion(_))).unwrap().clone();assert_eq!(potion.status,RoomChoiceStatus::Disabled);let before=r.clone();assert!(r.step(&GameAction::Choose(potion.id)).is_err());assert!(r==before);
    choose(&mut r,RoomChoiceKind::CardReward);let before=r.clone();let proceed=parent.choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;assert!(r.step(&GameAction::Choose(proceed)).is_err());assert!(r==before);choose(&mut r,RoomChoiceKind::Skip);
    r.step(&GameAction::DiscardPotion{slot:1}).unwrap();let offered=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.id==potion.id).unwrap();assert_eq!(offered.kind,potion.kind);assert_eq!(offered.status,RoomChoiceStatus::Offered);choose(&mut r,potion.kind.clone());assert_eq!(r.state.potions[1],Some(crate::content::PotionId::EnergyPotion));let before=r.clone();assert!(r.step(&GameAction::Choose(potion.id)).is_err());assert!(r==before);
    let gold=parent.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Gold(_))).unwrap();let old=r.state.gold;r.step(&GameAction::Choose(gold.id)).unwrap();if let RoomChoiceKind::Gold(n)=gold.kind{assert_eq!(r.state.gold,old+n)}let before=r.clone();assert!(r.step(&GameAction::Choose(gold.id)).is_err());assert!(r==before);
}
