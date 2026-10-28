//! Supplied owner diagnostics; normal-start evidence lives in the public caller target.
use super::super::*;
use crate::{combat::{self, CombatAction, Outcome, rng::Partition}, content::{CardId, CardType, Character, EncounterId, EventId, PotionId, RelicId}};

fn run() -> Run {
    let mut profile=CreationProfile::experienced(); profile.neow_eligible=false;
    profile.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
    profile.seen_bosses[1]=vec!["Automaton","Collector"];
    profile.seen_bosses[2]=vec!["AwakenedOne","DonuAndDeca"];
    Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},profile,RunRng::new([7;32],Partition::Test,0),RoomScenario::M14SilentAct3).unwrap()
}
fn choose(r:&mut Run, kind:RoomChoiceKind)->RunTransition {
    let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==kind&&row.status==RoomChoiceStatus::Offered).unwrap();
    r.step(&GameAction::Choose(row.id)).unwrap()
}
fn act3(boss:EncounterId)->Run {
    let mut r=run(); choose(&mut r,RoomChoiceKind::NeowLament);
    r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);
    let mut rng=r.state.rng.clone();let mut map=map::generate(3,&mut rng,map::MapEligibility::from_run(&r.state)).unwrap();
    for node in &mut map.nodes {node.floor+=34;} map.boss_icon=Some(boss);
    r.state.act=3;r.state.floor=34;r.state.map=Some(map);r.state.rng=rng;r.state.room="Map";
    let room=r.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=None;room.act3=Some(Default::default());room.act3_boss_order=Some(vec![boss]);room.screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};r
}
fn before(r:&mut Run,target:NodeId) {
    let map=r.state.map.as_mut().unwrap();let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();prev.visited=true;
    r.state.floor=prev.floor;map.current=Some(prev.id);
}
fn enter_kind(r:&mut Run,kind:MapNodeKind)->NodeId {
    let id=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.kind==kind&&r.state.map.as_ref().unwrap().nodes.iter().any(|p|p.edges.contains(&n.id))).unwrap().id;
    before(r,id);r.step(&GameAction::Move(id)).unwrap();id
}
#[test]
fn m14_silent_act3_continues_existing_acts_and_all_late_edges() {
    let mut r=run();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.deck.len()),(70,70,99,12));
    assert_eq!(r.state.relics,vec![RelicId::RingOfTheSnake]);assert_eq!(r.state.potions,vec![None;3]);assert_eq!(r.state.keys,[false;3]);
    choose(&mut r,RoomChoiceKind::NeowLament);assert_eq!(r.public_map().unwrap().boss,Some(EncounterId::Guardian));
    assert!(rooms::uses_captured_act2(&r.state));assert!(rooms::uses_captured_act3(&r.state));
    for boss in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca] {
        let r=act3(boss);let map=r.state.map.as_ref().unwrap();
        for node in &map.nodes {if (35..=50).contains(&node.floor){assert!(rooms::act3::capture_room(&r.state,node.id).is_ok(),"floor {}",node.floor);}}
        assert!(map.nodes.iter().any(|n|n.floor==49&&n.kind==MapNodeKind::Rest));
        let node=map.nodes.iter().find(|n|n.floor==50&&n.kind==MapNodeKind::Boss).unwrap();assert_eq!(rooms::act3::capture_room(&r.state,node.id).unwrap().encounter,Some(boss));
    }
    assert_eq!(M14_SILENT_ACT3_CARDS.len(),20);assert!(M14_SILENT_ACT3_CARDS.contains(&CardId::Parasite));assert_eq!(M14_SILENT_ACT3_RELICS.len(),18);assert_eq!(M14_SILENT_ACT3_POTIONS.len(),4);
    let mut r=run();for &id in M14_SILENT_ACT3_CARDS {mutations::obtain_card(&mut r.state,id,0).unwrap();}
    let before=r.clone();assert!(mutations::obtain_card(&mut r.state,CardId::Strike,0).is_err());assert_eq!(r,before);
    assert!(!mutations::can_upgrade(&r.state,r.state.deck.last().unwrap()));
}
#[test]
fn m14_silent_act3_local_stock_and_exhaustion() {
    let mut r=act3(EncounterId::TimeEater);
    for ordinal in 0..3 {assert_eq!(rooms::m14_content::reward_cards(ordinal),[[CardId::PoisonedStab,CardId::Backflip,CardId::BladeDance],[CardId::DeadlyPoison,CardId::CloakAndDagger,CardId::DaggerThrow],[CardId::NoxiousFumes,CardId::Accuracy,CardId::WellLaidPlans]][ordinal].to_vec());assert!(!rooms::m14_content::reward_cards(ordinal).contains(&CardId::Parasite));}
    assert_eq!([0,1,2].map(rooms::m14_content::combat_potion),[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion]);
    let stock=[RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::SneckoSkull,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,RelicId::BronzeScales];
    for (i,id) in stock.iter().enumerate(){assert_eq!(rooms::m14_content::elite_relic(i,&[]).unwrap(),(*id,i+1));}
    assert_eq!(rooms::m14_content::elite_relic(0,&stock),Err(RunError::Unsupported("captured Silent Act3 elite relic sequence exhausted")));
    for later in [false,true] {
        r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().shop_visits=usize::from(later);
        let mut next=r.next_choice;let screen=rooms::shop::screen(&r,&mut next).unwrap();
        assert_eq!(screen.choices.iter().filter_map(|row|if let RoomChoiceKind::Card{id,..}=row.kind{Some((id,row.price.unwrap()))}else{None}).collect::<Vec<_>>(),(if later{[CardId::Deflect,CardId::Tactician,CardId::Nightmare]}else{[CardId::QuickSlash,CardId::Reflex,CardId::Catalyst]}).into_iter().zip(if later{[50,75,150]}else{[50,75,75]}).collect::<Vec<_>>());
        assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Potion(if later{PotionId::FirePotion}else{PotionId::DexterityPotion})&&row.price==Some(50)));
    }
    enter_kind(&mut r,MapNodeKind::Elite);let active=r.state.active_combat.as_ref().unwrap();let record=CombatRecord{visit:active.visit,encounter:active.encounter,outcome:Outcome::Victory};
    let prepared=rooms::rewards::prepare_combat(&r,&record).unwrap().unwrap();rooms::rewards::commit_combat(&mut r,prepared);
    assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor,1);
    assert!(!r.state.combat.as_ref().unwrap().item_context().unwrap().relics().contains(&RelicId::Lantern));
    r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().elite_relic_cursor=13;
    let frozen=r.clone();assert!(matches!(rooms::rewards::prepare_combat(&r,&record),Err(RunError::Unsupported("captured Silent Act3 elite relic sequence exhausted"))));assert_eq!(r,frozen);
    let mut chest=act3(EncounterId::TimeEater);chest.state.relics.extend([RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial]);enter_kind(&mut chest,MapNodeKind::Treasure);choose(&mut chest,RoomChoiceKind::OpenChest);
    let screen=chest.decision().unwrap().room.unwrap();assert_eq!(screen.choices.iter().filter(|row|matches!(row.kind,RoomChoiceKind::Relic(_))&&row.status==RoomChoiceStatus::Disabled).count(),3);
    choose(&mut chest,RoomChoiceKind::Sapphire);assert!(chest.decision().unwrap().room.unwrap().choices.iter().filter(|row|matches!(row.kind,RoomChoiceKind::Relic(_))).all(|row|row.status!=RoomChoiceStatus::Offered));choose(&mut chest,RoomChoiceKind::Proceed);
}
fn install_event(r:&mut Run,event:EventId) {
    r.state.floor=36;
    let node=r.state.map.as_ref().unwrap().nodes[0].id;
    r.state.map.as_mut().unwrap().current=Some(node);
    let mut next=r.next_choice;let p=rooms::events::act3::prepare(r,event,node,0,&mut next).unwrap();
    r.next_choice=next;r.state.rng=p.rng;let room=r.state.room_loop.as_mut().unwrap();room.screen=p.screen;room.act3_event=Some(p.frame);
}
fn deck(r:&mut Run,ids:&[CardId]) {
    r.state.deck=ids.iter().enumerate().map(|(i,id)|CardInstance{ misc:0,id:*id,instance:i as u64+1,upgrades:0}).collect();r.state.next_instance=ids.len() as u64+1;
}
#[test]
fn m14_silent_act3_falling_tomb_and_parasite_use_shared_owners() {
    let mut r=act3(EncounterId::TimeEater);deck(&mut r,&[CardId::StrikeGreen,CardId::StrikeGreen,CardId::DefendGreen,CardId::NoxiousFumes,CardId::Parasite]);
    install_event(&mut r,EventId::Falling);let rows=r.decision().unwrap().room.unwrap().choices;
    assert_eq!(rows.iter().map(|row|match row.kind{RoomChoiceKind::FallingLose{card_type,..}=>card_type,_=>panic!()}).collect::<Vec<_>>(),[CardType::Skill,CardType::Power,CardType::Attack]);
    let row=rows.into_iter().find(|row|matches!(row.kind,RoomChoiceKind::FallingLose{card_type:CardType::Attack,..})).unwrap();
    let RoomChoiceKind::FallingLose{instance:Some(instance),..}=row.kind else{panic!()};
    let hp=(r.state.hp,r.state.max_hp);r.step(&GameAction::Choose(row.id)).unwrap();
    assert!(!r.state.deck.iter().any(|c|c.instance==instance));assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::StrikeGreen).count(),1);assert_eq!((r.state.hp,r.state.max_hp),hp);
    let frozen=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert_eq!(r,frozen);
    for only in [CardId::Parasite,CardId::StrikeGreen] {
        let mut r=act3(EncounterId::TimeEater);deck(&mut r,&[only]);install_event(&mut r,EventId::Falling);
        let screen=r.decision().unwrap().room.unwrap();let row=screen.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered).unwrap();
        if only==CardId::Parasite {assert_eq!(row.kind,RoomChoiceKind::Leave);assert_eq!(screen.choices.len(),1);}else{assert_eq!(screen.choices.iter().filter(|c|c.status==RoomChoiceStatus::Disabled).count(),2);}
        let t=r.step(&GameAction::Choose(row.id)).unwrap();assert_eq!(t.terminal,if only==CardId::Parasite{None}else{Some(RunTerminal::EmptyDeck)});
    }
    for owned in [false,true] {for gold in [0,99] {
        let mut r=act3(EncounterId::TimeEater);r.state.gold=gold;if owned{crate::items::acquire_relic(&mut r.state,RelicId::RedMask).unwrap();}
        install_event(&mut r,EventId::TombOfLordRedMask);let rng=r.state.rng.clone();let kind=if owned{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold};choose(&mut r,kind);
        assert_eq!(r.state.gold,if owned{gold+222}else{0});assert_eq!(r.state.relics.iter().filter(|id|**id==RelicId::RedMask).count(),1);assert_eq!(r.state.rng,rng);
    }}
}
fn writhing(fault:bool)->Run {
    let mut r=act3(EncounterId::TimeEater);deck(&mut r,&[CardId::DefendGreen,CardId::StrikeGreen]);
    r.state.floor=35;r.state.map.as_mut().unwrap().current=Some(r.state.map.as_ref().unwrap().nodes[0].id);
    // Actual native setup with only its next intent supplied to the acquisition frontier.
    let player=combat::PlayerState{hp:70,max_hp:70,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]};
    let mut entry=combat::enemies::setup(EncounterId::WrithingMass,3,0,&mut r.state.rng,combat::enemies::SetupContext{player,player_powers:vec![],character:Character::Silent,cards:r.state.deck.iter().map(|c|combat::permanent_live_card(c.id,combat::CardInstanceId(c.instance),c.upgrades)).collect(),opening_draw:1,encounter_visit:0}).unwrap();
    entry.setup.enemies[0].intent=combat::FullIntentId::from_name("writhing_parasite");
    if fault {let id=entry.setup.enemies[0].ai_rng.id().clone();entry.setup.enemies[0].ai_rng=combat::rng::StreamState::new(id,[0;32],1_u128<<64).unwrap();}
    let mut items=Some(crate::items::CombatItemContext::take_from_run(&mut r.state));let c=combat::Combat::enter_with_item_context(entry,&mut items).unwrap();
    r.state.combat=Some(c);r.state.active_combat=Some(ActiveCombat{ applied_mutation_ordinal:0,visit:0,encounter:EncounterId::WrithingMass,origin:CombatOrigin::Authored,applied_gain_ordinal:0,acquired_instances:vec![]});r.state.combat_progress=CombatProgress{entered:1,settled:0};r
}
fn hit(r:&mut Run,slot:Option<usize>,amount:i32) {
    let c=r.state.combat.as_mut().unwrap();let actor=slot.map_or(combat::Actor::Player,|slot|combat::Actor::Enemy(c.public_full().unwrap().enemies[slot].instance_id.clone()));
    c.enqueue_native_effects(vec![combat::Effect{source:combat::Source::Actor(combat::Actor::Player),target:Some(actor),kind:combat::EffectKind::Damage(combat::DamagePayload::Fixed(amount))}],None).unwrap();
}
#[test]
fn m14_silent_act3_actual_parasite_gain_survives_victory_death_and_paid_purge() {
    for death in [false,true] {
        let mut r=writhing(false);let hp=(r.state.hp,r.state.max_hp);r.choose_combat(CombatAction::EndTurn).unwrap();assert_eq!(r.state.deck.last().unwrap().id,CardId::Parasite);assert_eq!(r.state.max_hp,hp.1);
        let gain=r.state.deck.last().unwrap().instance;let before=r.clone();r.sync_permanent_gains().unwrap();assert_eq!(r,before);
        hit(&mut r,if death{None}else{Some(0)},2000);r.settle_combat().unwrap();assert_eq!(r.state.deck.iter().filter(|c|c.id==CardId::Parasite).count(),1);assert_eq!(r.state.terminal,if death{Some(RunTerminal::Death)}else{None});
        assert!(!mutations::can_upgrade(&r.state,r.state.deck.last().unwrap()));
        if !death {
            let mut next=r.next_choice;let screen=rooms::shop::screen(&r,&mut next).unwrap();r.next_choice=next;r.state.room_loop.as_mut().unwrap().screen=screen;
            choose(&mut r,RoomChoiceKind::Purge);let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Select(_))&&c.object.as_ref().is_some_and(|object|object.instance==gain)).unwrap().action;
            let gold=r.state.gold;r.step(&action).unwrap();assert_eq!(r.state.gold,gold-75);assert_eq!((r.state.hp,r.state.max_hp),(67,67));let frozen=r.clone();assert!(r.step(&action).is_err());assert_eq!(r,frozen);
        }
    }
    for (hp,max_hp,want) in [(3,3,1),(1,1,1)] {let mut r=run();deck(&mut r,&[CardId::Parasite]);r.state.hp=hp;r.state.max_hp=max_hp;mutations::remove_card(&mut r.state,1).unwrap();assert_eq!((r.state.hp,r.state.max_hp),(want,want));}
}
fn defeated(boss:EncounterId,keys:[bool;3],enabled:bool)->Run {
    let mut r=act3(boss);
    for (i,key) in [Key::Ruby,Key::Sapphire,Key::Emerald].into_iter().enumerate(){if keys[i]{let event=mutations::acquire_key(&mut r.state,key).unwrap();assert_eq!(reward::commit(&mut r.state,&[event]).unwrap(),5);}}
    r.state.act4_enabled=enabled;enter_kind(&mut r,MapNodeKind::Boss);
    match boss {EncounterId::AwakenedOne=>{hit(&mut r,Some(2),300);r.choose_combat(CombatAction::EndTurn).unwrap();hit(&mut r,Some(2),300);},EncounterId::TimeEater=>hit(&mut r,Some(0),456),EncounterId::DonuAndDeca=>{hit(&mut r,Some(0),250);hit(&mut r,Some(1),250);},_=>panic!()}
    assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));r
}
#[test]
fn m14_silent_act3_actual_boss_ending_all_key_combinations() {
    for boss in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca] {for mask in 0..8 {for enabled in [false,true] {
        let mut r=defeated(boss,[mask&1!=0,mask&2!=0,mask&4!=0],enabled);let resources=(r.state.gold,r.state.deck.clone());let settled=r.state.combat_progress.settled;
        assert_eq!(r.settle_combat().unwrap().reward,0);assert_eq!(r.state.combat_progress.settled,settled+1);assert_eq!((r.state.gold,r.state.deck.clone()),resources);
        assert_eq!(r.state.last_combat.as_ref().unwrap().encounter,boss);let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Act3BossComplete);assert_eq!(screen.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);assert!(r.state.room_loop.as_ref().unwrap().active_rewards.is_none());
        let resources=(r.state.hp,r.state.max_hp,r.state.gold,r.state.deck.clone(),r.state.relics.clone(),r.state.potions.clone(),r.state.reward_ledger.total);
        choose(&mut r,RoomChoiceKind::Proceed);for stage in 1..=3{assert_eq!(choose(&mut r,RoomChoiceKind::Act3Continue{stage}).reward,0);}
        if mask==7&&enabled {assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act4Ready);assert_eq!(r.state.terminal,None);}else{assert_eq!(choose(&mut r,RoomChoiceKind::Act3Continue{stage:4}).terminal,Some(RunTerminal::Act3NoKeys));}
        assert_eq!((r.state.act,r.state.floor),(3,50));assert!(r.decision().unwrap().candidates.is_empty());assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.deck.clone(),r.state.relics.clone(),r.state.potions.clone(),r.state.reward_ledger.total),resources);
        assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::FloorEntered{floor:51,..}|CommittedEvent::Ended(RunTerminal::HeartVictory))));
    }}}
}
#[test]
fn m14_silent_act3_accepted_sources_survive_fault_once() {
    let mut r=writhing(true);assert!(matches!(r.choose_combat(CombatAction::EndTurn),Err(RunError::Faulted(_))));assert_eq!(r.state.deck.last().unwrap().id,CardId::Parasite);assert_eq!(r.state.active_combat.as_ref().unwrap().applied_gain_ordinal,1);let frozen=r.clone();r.sync_permanent_gains().unwrap();assert_eq!(r,frozen);assert!(r.choose_combat(CombatAction::EndTurn).is_err());assert_eq!(r,frozen);
    for event in [EventId::Falling,EventId::TombOfLordRedMask] {
        let mut r=act3(EncounterId::TimeEater);deck(&mut r,&[CardId::StrikeGreen,CardId::DefendGreen]);install_event(&mut r,event);
        let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|row|if event==EventId::Falling{matches!(row.kind,RoomChoiceKind::FallingLose{card_type:CardType::Attack,..})}else{row.kind==RoomChoiceKind::TombOfferGold}).unwrap();
        if event==EventId::Falling{mutations::remove_card(&mut r.state,1).unwrap();}else{r.state.gold=0;crate::items::acquire_relic(&mut r.state,RelicId::RedMask).unwrap();}
        let frame=r.state.room_loop.as_mut().unwrap().act3_event.as_mut().unwrap();frame.accepted=Some((row.id,row.kind));frame.mutation_complete=true;r.next_choice=u64::MAX;
        let err=rooms::events::act3::drive(&mut r).unwrap_err();assert!(matches!(err,RunError::Faulted(_)));let frozen=r.clone();assert_eq!(rooms::events::act3::drive(&mut r),Err(err));assert_eq!(r,frozen);assert_eq!(r.state.terminal,None);
    }
    let mut r=defeated(EncounterId::TimeEater,[false;3],true);r.next_choice=u64::MAX;let before=r.state.combat_progress;let err=r.settle_combat().unwrap_err();assert!(matches!(&err,RunError::Faulted(f) if f.checkpoint=="run.m14.act3.end"));assert_eq!(r.state.combat_progress,before);assert!(r.state.active_combat.is_some());let frozen=r.clone();assert_eq!(r.choose_combat(CombatAction::EndTurn),Err(err));assert_eq!(r,frozen);
    let mut r=defeated(EncounterId::TimeEater,[false;3],true);r.settle_combat().unwrap();choose(&mut r,RoomChoiceKind::Proceed);r.state.map.as_mut().unwrap().current=None;
    let action=GameAction::Choose(r.decision().unwrap().room.unwrap().choices[0].id);let err=r.step(&action).unwrap_err();assert!(matches!(&err,RunError::Faulted(f) if f.checkpoint=="run.m14.act3.end"));let frozen=r.clone();assert_eq!(r.step(&action),Err(err));assert_eq!(r,frozen);assert_eq!(r.state.terminal,None);
}
#[test]
fn m14_silent_act3_actual_act2_settlement_initializes_successor_once() {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);
    let mut rng=r.state.rng.clone();let mut map=map::generate(2,&mut rng,map::MapEligibility::from_run(&r.state)).unwrap();for n in &mut map.nodes{n.floor+=17;}map.boss_icon=Some(EncounterId::Champ);
    r.state.act=2;r.state.floor=17;r.state.map=Some(map);r.state.rng=rng;
    let room=r.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=Some(Default::default());room.act2_boss_order=Some(vec![EncounterId::Champ]);room.transition=Some(ActTransition{from:1,to:2,floor:17,reset_applied:true,heal_applied:true,initialized:true});
    enter_kind(&mut r,MapNodeKind::Boss);hit(&mut r,Some(0),2000);r.settle_combat().unwrap();assert_eq!(r.state.last_combat.as_ref().unwrap().encounter,EncounterId::Champ);
    choose(&mut r,RoomChoiceKind::CardReward);assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Card{id:CardId::Nightmare,upgrades:0}));choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::Proceed);
    choose(&mut r,RoomChoiceKind::OpenChest);assert!(!r.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Relic(RelicId::BlackBlood)));
    choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r.state.hp=41;let resources=(r.state.deck.clone(),r.state.gold,r.state.keys);let t=choose(&mut r,RoomChoiceKind::Proceed);
    assert_eq!((r.state.act,r.state.floor,r.state.hp,t.reward),(3,34,r.state.max_hp,0));assert_eq!((r.state.deck.clone(),r.state.gold,r.state.keys),resources);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);assert_eq!(r.public_map().unwrap().boss,Some(EncounterId::TimeEater));assert!(r.state.room_loop.as_ref().unwrap().act3.is_some());let frozen=r.clone();assert!(rooms::transition::drive(&mut r).is_err());assert_eq!(r,frozen);
}
#[test]
fn m14_silent_act3_sampler_uses_all_sixteen_existing_setup_ids() {
    let mut reached=std::collections::BTreeSet::new();
    for boss in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca] {
        let mut r=act3(boss);let map=r.state.map.as_ref().unwrap();let monster=map.nodes.iter().find(|n|n.kind==MapNodeKind::Monster).unwrap().id;let elite=map.nodes.iter().find(|n|n.kind==MapNodeKind::Elite).unwrap().id;let boss_node=map.nodes.iter().find(|n|n.kind==MapNodeKind::Boss).unwrap().id;
        for _ in 0..10 {let id=rooms::act3::capture_room(&r.state,monster).unwrap().encounter.unwrap();reached.insert(format!("{id:?}"));r.state.map.as_mut().unwrap().regular_encounter_history.push(id);}
        for _ in 0..3 {let id=rooms::act3::capture_room(&r.state,elite).unwrap().encounter.unwrap();reached.insert(format!("{id:?}"));r.state.map.as_mut().unwrap().elite_encounter_history.push(id);}
        let id=rooms::act3::capture_room(&r.state,boss_node).unwrap().encounter.unwrap();reached.insert(format!("{id:?}"));
    }
    assert_eq!(reached.len(),16);assert!(reached.contains("JawWormHorde"));
}
#[test]
fn m14_silent_act3_exhausted_publication_retains_real_victory_without_cursor_advance() {
    let mut r=act3(EncounterId::AwakenedOne);r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().elite_relic_cursor=13;enter_kind(&mut r,MapNodeKind::Elite);
    hit(&mut r,Some(0),2000);assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));let before=r.state.combat_progress;let err=r.settle_combat().unwrap_err();assert!(matches!(err,RunError::Faulted(_)));assert_eq!(r.state.combat_progress,before);assert!(r.state.active_combat.is_some());assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor,13);assert_eq!(r.state.terminal,None);
    let frozen=r.clone();assert_eq!(r.choose_combat(CombatAction::EndTurn),Err(err));assert_eq!(r,frozen);
}
#[test]
fn m14_silent_act3_prepares_act_local_rows_and_skipped_elite_stock_once() {
    let mut r=act3(EncounterId::AwakenedOne);enter_kind(&mut r,MapNodeKind::Elite);
    let active=r.state.active_combat.as_ref().unwrap();let record=CombatRecord{visit:active.visit,encounter:active.encounter,outcome:Outcome::Victory};
    for ordinal in 0..3 {
        r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().combat_visits=ordinal as u64;
        let prepared=rooms::rewards::prepare_combat(&r,&record).unwrap().unwrap();let mut published=r.clone();rooms::rewards::commit_combat(&mut published,prepared);
        let room=published.state.room_loop.as_ref().unwrap();assert!(room.screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Potion([PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][ordinal])));
        assert_eq!(room.active_rewards.as_ref().unwrap().cards.choices.iter().filter_map(|row|match row.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>(),rooms::m14_content::reward_cards(ordinal));
    }
    r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().combat_visits=0;hit(&mut r,Some(0),2000);r.settle_combat().unwrap();
    assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor,1);choose(&mut r,RoomChoiceKind::Proceed);
    assert!(!r.state.relics.contains(&RelicId::Lantern));assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor,1);
    assert_eq!(rooms::m14_content::elite_relic(0,&[RelicId::Lantern,RelicId::Pear]).unwrap(),(RelicId::Mango,3));
    let r=run();for (character,ascension,long_neow) in [(Character::Ironclad,0,false),(Character::Silent,1,false),(Character::Silent,0,true)] {
        let mut profile=CreationProfile::experienced();profile.neow_eligible=long_neow;
        assert!(Run::new_room_scenario(RunConfig{character,ascension},profile,r.state.rng.clone(),RoomScenario::M14SilentAct3).is_err());
    }
}

#[test]
fn m14_silent_act3_accepted_entry_fault_keeps_m14_owner_and_first_fault() {
    let mut r=act3(EncounterId::TimeEater);
    let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==50&&n.kind==MapNodeKind::Boss).unwrap().id;
    before(&mut r,node);
    // Exhaust the existing native entry counter, after a valid map preflight
    // can accept the real boss room but before a combat can be constructed.
    r.state.combat_progress.entered=u64::MAX;
    let prior=r.state.reward_ledger.total;
    let error=r.step(&GameAction::Move(node)).unwrap_err();
    let RunError::Faulted(first)=&error else { panic!("accepted entry must retain its owner failure") };
    assert_eq!(first.checkpoint,"run.m14.encounter");
    assert_eq!(first.kind,RunFaultKind::Invariant("accepted M14 encounter entry failed"));
    assert_eq!(r.state.fault,Some(first.clone()));
    assert_eq!((r.state.floor,r.state.reward_ledger.total,r.state.terminal),(50,prior+1,None));
    assert!(r.state.combat.is_none()&&r.state.active_combat.is_none());
    assert!(r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::FloorEntered{act:3,floor:50})));
    assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::CombatEntered{..}|CommittedEvent::CombatFinished(_))));
    let retained=r.clone();
    assert_eq!(r.step(&GameAction::Move(node)),Err(error));
    assert_eq!(r,retained,"retry preserves the first fault and accepted room credit");
}
