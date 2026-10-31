// M24 caller, input contract and public played-prefix checks. Only the frozen
// M22/M23 policy and input modules are compiled; no rollout loop or outcome run.
#[path="../examples/m23_support/policy.rs"]mod m23_policy;
#[path="../examples/m23_support/inputs.rs"]mod m23_inputs;
#[path="../examples/m24_support/policy.rs"]mod m24_policy;
#[path="../examples/m24_support/inputs.rs"]mod m24_inputs;
use spire_core::{combat::{self,CombatAction,CombatPublic,CharacterCombatView,FullPlayerView,Stance},content::{Character,CardId,EncounterId,EventId,RelicId},run::*};
use spire_core::combat::rng::Partition;

/// Matched-history played prefix through real Act1 and Act2 rooms. The frozen M23
/// caller chooses on the M23 Run; the identical action is applied to an M24 Run built
/// from the same creation history, root, partition and ordinal. Public decisions (up to
/// the scenario tag), maps and transitions must agree at every step until M23 stops.
#[test]
fn m24_caller_prefix_matches_m23_through_real_act2_rooms(){
    let scenario=m23_inputs::Scenario::Collector;let seed=7;
    let mut old=m23_inputs::make_run(seed,scenario,false).unwrap();
    let p=m23_inputs::creation(seed,scenario,false).unwrap();let mut root=[0;32];root[..8].copy_from_slice(&u64::to_le_bytes(seed));
    let mut new=Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),RoomScenario::M24WatcherAct3).unwrap();
    let public=|r:&Run|{let mut d=r.decision().unwrap();d.content_profile=Some(RoomScenario::M23WatcherAct2);(d,r.public_map())};
    assert_eq!(public(&old),public(&new),"initial");
    let mut trace:Vec<(PublicDecision,GameAction,RunTransition)>=vec![];let mut acts=std::collections::BTreeSet::new();let mut act2_rooms=0;
    loop{
        let d=old.decision().unwrap();
        if d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready)||trace.len()>=32768{break}
        let progress=m23_policy::act_progress(trace.iter().map(|(d,a,t)|(d,a,t)),d.act);
        let action=m23_policy::choose(&d,old.public_map().as_ref(),&progress).unwrap();
        let a=old.step(&action).unwrap();let b=new.step(&action).unwrap();assert_eq!(a,b,"transition at step {}: {action:?}",trace.len());
        assert_eq!(public(&old),public(&new),"public state after step {}: {action:?}",trace.len());
        acts.insert(d.act);if d.act==2&&matches!(action,GameAction::Move(_)){act2_rooms+=1;}
        trace.push((d,action,a));
    }
    let end=old.decision().unwrap();
    eprintln!("m24 caller prefix: {} matched steps, endpoint act {} floor {} terminal {:?}, Act2 rooms entered {act2_rooms}",trace.len(),end.act,end.floor,end.terminal);
    // The retained M23 receipt for this canonical case is Death at act 2, floor 23 after
    // 425 accepted actions; this test re-derives it and requires the Act2 prefix.
    assert!(acts.contains(&2)&&act2_rooms>0,"the frozen M23 caller reaches real Act2 rooms on this canonical case");
}

fn decision(actions:Vec<GameAction>)->PublicDecision{PublicDecision{content_profile:Some(RoomScenario::M24WatcherAct3),room:None,reward_total:0,act:1,floor:0,player:PublicPlayer{relic_counters:vec![],character:Character::Watcher,hp:72,max_hp:72,gold:99,relics:vec![RelicId::PureWater],potions:vec![None;3],keys:[false;3]},combat:None,combat_progress:CombatProgress::default(),last_combat:None,objects:vec![],candidates:actions.into_iter().enumerate().map(|(i,action)|Candidate{id:ChoiceId(i as u64),action,label:"fabricated public candidate",neow_offer:None,object:None}).collect(),offers:vec![],selection:None,terminal:None}}
fn room(kind:RoomViewKind,rows:Vec<(RoomChoiceKind,Option<u32>,RoomChoiceStatus)>)->PublicDecision{let choices=rows.into_iter().enumerate().map(|(i,(kind,price,status))|RoomChoiceView{id:ChoiceId(i as u64),kind,price,status}).collect::<Vec<_>>();let mut d=decision(choices.iter().filter(|c|c.status==RoomChoiceStatus::Offered).map(|c|GameAction::Choose(c.id)).collect());d.room=Some(RoomView{kind,choices});d}
fn offered(kind:RoomViewKind,rows:Vec<(RoomChoiceKind,Option<u32>)>)->PublicDecision{room(kind,rows.into_iter().map(|(k,p)|(k,p,RoomChoiceStatus::Offered)).collect())}
fn combat_decision(actions:Vec<CombatAction>)->PublicDecision{let mut d=decision(actions.into_iter().map(GameAction::Combat).collect());d.combat=Some(CombatPublic{mantra_gained:Some(0),stance_choice:None,played_powers:vec![],playing_cards:vec![],stasis:vec![],content_id:"fabricated public decision".into(),turn:1,player:FullPlayerView{hp:72,max_hp:72,block:0,energy:3,max_energy:3,powers:vec![]},enemies:vec![],character:CharacterCombatView{stance:Stance::Neutral,orbs:vec![],orb_dark_damage:vec![],focus:0},hand:vec![],draw_count:0,discard:vec![],exhaust:vec![],outcome:None,hp_before_healing:None,hp_after_cleanup:None,targets:vec![],selection:None,known_draw:vec![],encounter_facing:None,inventory:None,card_offer:None});d}

#[test]fn m24_caller_keeps_m23_choices(){
    // Act1/Act2 decisions: the M24 policy returns exactly the frozen M23 policy's choice.
    let p24=m24_policy::PublicCallerProgress::default();let p23=m23_policy::PublicCallerProgress::default();
    let mut cases=vec![
        offered(RoomViewKind::Neow,vec![(RoomChoiceKind::NeowLament,None),(RoomChoiceKind::NeowMaxHp{amount:7},None)]),
        offered(RoomViewKind::CardReward,vec![(RoomChoiceKind::Skip,None),(RoomChoiceKind::Card{id:CardId::JustLucky,upgrades:0},None)]),
        offered(RoomViewKind::Rewards,vec![(RoomChoiceKind::Gold(13),None),(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Proceed,None)]),
        offered(RoomViewKind::Shop,vec![(RoomChoiceKind::Card{id:CardId::MasterReality,upgrades:0},Some(150)),(RoomChoiceKind::Leave,None)]),
        offered(RoomViewKind::Campfire,vec![(RoomChoiceKind::Rest{heal:21},None),(RoomChoiceKind::Smith,None),(RoomChoiceKind::Recall,None)]),
        offered(RoomViewKind::BossRelicChoice,vec![(RoomChoiceKind::Relic(RelicId::SlaversCollar),None),(RoomChoiceKind::Relic(RelicId::HolyWater),None),(RoomChoiceKind::Skip,None)]),
        offered(RoomViewKind::Treasure,vec![(RoomChoiceKind::Sapphire,None),(RoomChoiceKind::Relic(RelicId::Anchor),None),(RoomChoiceKind::Proceed,None)]),
        offered(RoomViewKind::Event(EventId::AncientWriting),vec![(RoomChoiceKind::AncientElegance,None),(RoomChoiceKind::AncientSimplicity,None)]),
        combat_decision(vec![CombatAction::EndTurn,CombatAction::PlayCard{instance:8,target:Some(combat::TargetId(2))}]),
    ];
    let mut act2=cases.clone();for d in &mut act2{d.act=2;d.floor=20;}cases.extend(act2);
    for d in &cases{assert_eq!(m24_policy::choose(d,None,&p24),m23_policy::choose(d,None,&p23),"{:?}",d.room.as_ref().map(|r|r.kind));}
    // Act3 rooms: the shared default rule makes the documented deterministic choice.
    let mut act3=vec![
        (offered(RoomViewKind::Event(EventId::Falling),vec![(RoomChoiceKind::FallingLose{card_type:spire_core::content::CardType::Skill,instance:Some(3)},None),(RoomChoiceKind::FallingLose{card_type:spire_core::content::CardType::Power,instance:Some(4)},None),(RoomChoiceKind::FallingLose{card_type:spire_core::content::CardType::Attack,instance:Some(5)},None)]),0),
        (offered(RoomViewKind::Event(EventId::TombOfLordRedMask),vec![(RoomChoiceKind::TombOfferGold,Some(40)),(RoomChoiceKind::Leave,None)]),0),
        (offered(RoomViewKind::Act3BossComplete,vec![(RoomChoiceKind::Proceed,None)]),0),
        (offered(RoomViewKind::Act3Ending,vec![(RoomChoiceKind::Act3Continue{stage:2},None)]),0),
    ];
    for (d,want) in &mut act3{d.act=3;d.floor=40;assert_eq!(m24_policy::choose(d,None,&p24),Ok(GameAction::Choose(ChoiceId(*want))),"{:?}",d.room.as_ref().map(|r|r.kind));}
}

#[test]fn m24_caller_campfire_without_smith_rests(){
    let p24=m24_policy::PublicCallerProgress::default();let p23=m23_policy::PublicCallerProgress::default();
    let fire=|hp:u32,smith:RoomChoiceStatus|{let mut d=room(RoomViewKind::Campfire,vec![(RoomChoiceKind::Rest{heal:21},None,RoomChoiceStatus::Offered),(RoomChoiceKind::Smith,None,smith),(RoomChoiceKind::Recall,None,RoomChoiceStatus::Offered)]);d.player.hp=hp;d};
    // HP 72/72, Smith disabled: M24 rests; the frozen M23 policy has no prescribed action.
    let d=fire(72,RoomChoiceStatus::Disabled);assert_eq!(m24_policy::choose(&d,None,&p24),Ok(GameAction::Choose(ChoiceId(0))));assert!(m23_policy::choose(&d,None,&p23).is_err());
    // HP 30/72: both rest. HP 72/72 with Smith offered: both smith.
    let d=fire(30,RoomChoiceStatus::Disabled);assert_eq!(m24_policy::choose(&d,None,&p24),Ok(GameAction::Choose(ChoiceId(0))));assert_eq!(m23_policy::choose(&d,None,&p23),Ok(GameAction::Choose(ChoiceId(0))));
    let d=fire(72,RoomChoiceStatus::Offered);assert_eq!(m24_policy::choose(&d,None,&p24),Ok(GameAction::Choose(ChoiceId(1))));assert_eq!(m23_policy::choose(&d,None,&p23),Ok(GameAction::Choose(ChoiceId(1))));
}

#[test]fn m24_caller_input_contract_rejects_invalid_values_without_creation(){
    let parse=|v:&[&str]|m24_inputs::parse_args(v.iter().map(|s|s.to_string()));
    assert!(parse(&["--scenario","unknown"]).is_err());assert!(parse(&["--budget","32769"]).is_err());assert!(parse(&["--budget","x"]).is_err());assert!(parse(&["--seed","13"]).is_err());
    assert!(parse(&["--seed","11","--scenario","death"]).is_err(),"death control is seed 7 only");
    let o=parse(&[]).unwrap();assert_eq!((o.seed,o.budget,o.scenarios.len(),o.nonattacking),(7,32768,3,false));
    let o=parse(&["--scenario","death"]).unwrap();assert!(o.nonattacking);assert_eq!(o.scenarios,vec![m24_inputs::Scenario::AwakenedOne]);
}

#[test]fn m24_caller_exact_six_initial_histories_have_canonical_resources(){
    use EncounterId as E;
    let want=[(7,m24_inputs::Scenario::AwakenedOne,E::Guardian,E::Champ),(7,m24_inputs::Scenario::TimeEater,E::Hexaghost,E::BronzeAutomaton),(7,m24_inputs::Scenario::DonuAndDeca,E::SlimeBoss,E::Collector),
        (11,m24_inputs::Scenario::DonuAndDeca,E::Guardian,E::BronzeAutomaton),(11,m24_inputs::Scenario::AwakenedOne,E::Hexaghost,E::Collector),(11,m24_inputs::Scenario::TimeEater,E::SlimeBoss,E::Champ)];
    for (seed,s,a1,a2) in want{
        assert_eq!(m24_inputs::triple(seed,s,false),Ok((a1,a2)));let p=m24_inputs::creation(seed,s,false).unwrap();assert_eq!(p.seen_bosses.iter().map(Vec::len).collect::<Vec<_>>(),vec![2,2,2]);
        let r=m24_inputs::make_run(seed,s,false).unwrap();let d=r.decision().unwrap();
        assert_eq!(d.content_profile,Some(RoomScenario::M24WatcherAct3));assert_eq!((d.player.hp,d.player.max_hp,d.player.gold),(72,72,99));assert_eq!(d.player.relics,vec![RelicId::PureWater]);assert_eq!(d.player.potions,vec![None;3]);assert_eq!(d.player.keys,[false;3]);
        // Same seed/Act1/Act2 as the M23 canonical case: identical public start.
        let m23=m23_inputs::make_run(seed,match a2{E::Champ=>m23_inputs::Scenario::Champ,E::BronzeAutomaton=>m23_inputs::Scenario::BronzeAutomaton,_=>m23_inputs::Scenario::Collector},false).unwrap();
        let mut m=m23.decision().unwrap();m.content_profile=Some(RoomScenario::M24WatcherAct3);assert_eq!(m,d);
    }
}
