#![allow(dead_code)]
use spire_core::{combat::rng::Partition, content::*, run::*};
use spire_env::*;
pub fn config(act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3:RunAct3BossTarget) -> RunSessionConfig {
    RunSessionConfig { character: Character::Defect, ascension: 0, partition: Partition::Test,
        boss_history: RunBossHistory::M19DefectAct3 { act1, act2, act3 },
        capacities: RunCapacities::m10_default(), record_replay: true }
}
pub fn default_config() -> RunSessionConfig { config(RunAct1BossTarget::Guardian, RunAct2BossTarget::Champ, RunAct3BossTarget::AwakenedOne) }
// Short supplied transport diagnostic, independent of the held six outcome runs.
pub fn diagnostic_run() -> Run {
    let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());
    let mut creation=CreationProfile::experienced();creation.neow_eligible=false;
    creation.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
    creation.seen_bosses[1]=vec!["Automaton","Collector"];
    creation.seen_bosses[2]=vec!["TimeEater","DonuAndDeca"];
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},creation,
        RunRng::new(root,Partition::Test,0),RoomScenario::M19DefectAct3).unwrap()
}
pub fn combat_diagnostic() -> (PublicDecision, Option<map::PublicMap>) {
    let mut run=diagnostic_run();let d=run.decision().unwrap();run.step(&d.candidates[0].action).unwrap();
    let d=run.decision().unwrap();let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();
    run.step(&action).unwrap();(run.decision().unwrap(),run.public_map())
}

/// Supplied actual encounter diagnostic. The returned public decision is a
/// projection harness, never a normal Run traversal or a session checkpoint.
pub fn supplied_combat(encounter: EncounterId, cards: Vec<spire_core::combat::LiveCard>) -> spire_core::combat::Combat {
    use spire_core::combat::{enemies::{self,SetupContext},PlayerState};
    let context=SetupContext {
        player:PlayerState {hp:75,max_hp:75,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
        player_powers:vec![],character:Character::Defect,opening_draw:cards.len().min(10),cards,encounter_visit:0,
    };
    let mut rng=RunRng::new([37;32],Partition::Test,0);
    let setup=enemies::setup(encounter,if ACT3.contains(&encounter){3}else{2},0,&mut rng,context).unwrap();
    let mut items=Some(spire_core::items::CombatItemContext::new(vec![],vec![None;3]));
    spire_core::combat::Combat::enter_with_item_context(setup,&mut items).unwrap()
}
pub fn project_combat(combat:&spire_core::combat::Combat, objects:Vec<PublicObject>) -> PublicDecision {
    let mut d=diagnostic_run().decision().unwrap();
    d.act=3;d.floor=35;d.objects=objects;d.combat=Some(combat.public_full().unwrap());
    d.room.as_mut().unwrap().kind=RoomViewKind::Combat;d.room.as_mut().unwrap().choices.clear();
    d.offers.clear();d.candidates=combat.legal_actions_full().unwrap().into_iter().enumerate().map(|(i,action)|Candidate {
        id:ChoiceId(i as u64+1),action:GameAction::Combat(action),label:"supplied diagnostic",neow_offer:None,object:None,
    }).collect();d
}

#[path="../../../spire-core/examples/m19_support/mod.rs"]
pub mod caller;
pub fn case_config(s:caller::Scenario)->RunSessionConfig {
    config(match s.act1_boss {
        EncounterId::Guardian=>RunAct1BossTarget::Guardian,EncounterId::Hexaghost=>RunAct1BossTarget::Hexaghost,EncounterId::SlimeBoss=>RunAct1BossTarget::SlimeBoss,_=>panic!("foreign fixed Act1 target"),
    },match s.act2_boss {
        EncounterId::Champ=>RunAct2BossTarget::Champ,EncounterId::BronzeAutomaton=>RunAct2BossTarget::BronzeAutomaton,EncounterId::Collector=>RunAct2BossTarget::Collector,_=>panic!("foreign fixed Act2 target"),
    },match s.act3_boss {
        EncounterId::AwakenedOne=>RunAct3BossTarget::AwakenedOne,EncounterId::TimeEater=>RunAct3BossTarget::TimeEater,EncounterId::DonuAndDeca=>RunAct3BossTarget::DonuAndDeca,_=>panic!("foreign Act3 target"),
    })
}
/// Consumes versioned original frozen choices after source-bound release;
/// no caller rollout, alternative policy, or generated outcome is used here.
pub fn retained()->Vec<(u64,caller::Scenario,Run,caller::Caller)> {
    use sha2::{Digest,Sha256};
    let root=std::env::var_os("M19_CASE_RECEIPTS").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spire-core/tests/fixtures/m19"));
    caller::FIXED_CASES.iter().map(|(seed,scenario)| {
        let path=root.join(format!("{seed}-{}.json",scenario.label()));let bytes=std::fs::read(&path).expect("released original receipt");
        let value:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["seed"],*seed);assert_eq!(value["act1"],scenario.act1_boss.as_str());assert_eq!(value["act2"],scenario.act2_boss.as_str());assert_eq!(value["act3"],scenario.act3_boss.as_str());
        assert_eq!(value["budget"],caller::ACTION_LIMIT_MAX);assert_eq!(value["policy"],caller::CALLER_POLICY_REVISION);
        println!("M19_ORIGINAL_RECEIPT path={} sha256={} size={}",path.display(),Sha256::digest(&bytes).iter().map(|b|format!("{b:02x}")).collect::<String>(),bytes.len());
        let (run,caller)=caller::replay_receipt(&value).expect("same original choices and public transcript");(*seed,*scenario,run,caller)
    }).collect()
}
pub fn consume_single(seed:u64,scenario:caller::Scenario,core:&Run,trace:&caller::Caller)->(RunSession,RunEpisodeRecord) {
    let cfg=case_config(scenario);let mut s=RunSession::new_m19(cfg.clone()).unwrap();let mut b=s.reset(Some(seed)).unwrap();
    for step in &trace.trace {
        let r=s.reproduction().unwrap();assert_eq!(r.latest_decision.unwrap().unwrap(),step.decision);assert_eq!(r.latest_map,step.map);
        assert_eq!(b.observation,encode_run_m19(&step.decision,step.map.as_ref(),&cfg.capacities).unwrap());
        let index=step.decision.candidates.iter().position(|c|c.action==step.action).unwrap();
        let t=s.step(b.token,RunActionIndex(index as i64)).unwrap();assert_eq!((t.reward,t.boundary.terminal),(step.transition.reward,step.transition.terminal));
        assert_eq!(t.terminated,step.transition.terminal.is_some());assert!(!t.truncated);b=t.boundary;
    }
    if let Some((d,map,action,error))=&trace.fault_attempt {
        let r=s.reproduction().unwrap();assert_eq!(r.latest_decision.unwrap().unwrap(),*d);assert_eq!(r.latest_map,*map);
        let index=d.candidates.iter().position(|c|c.action==*action).unwrap();let actual=s.step(b.token,RunActionIndex(index as i64)).unwrap_err();
        let RunSessionError::Core(actual)=actual else {panic!("recorded failed attempt must keep original core error")};assert_eq!(format!("{actual:?}"),*error);
    }
    let reproduction=s.reproduction().unwrap();assert_eq!(reproduction.latest_decision.unwrap(),core.decision());assert_eq!(reproduction.latest_map,core.public_map());
    let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),trace.accepted());
    let decoded=RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();assert_eq!(decoded,record);
    let replay=replay_run(&decoded,cfg.capacities.clone()).unwrap();
    assert_eq!(replay.final_decision_error,core.decision().err());
    if let Ok(d)=core.decision() {assert_eq!(replay.final_decision,d);}
    assert_eq!(replay.final_map,core.public_map());assert_eq!(replay.accepted_count,trace.accepted());
    assert_eq!(replay.rewards,trace.trace.iter().map(|s|s.transition.reward).collect::<Vec<_>>());assert_eq!(replay.terminals,trace.trace.iter().map(|s|s.transition.terminal).collect::<Vec<_>>());assert_eq!(replay.reported_fault,record.fault);
    if let Some(dir)=std::env::var_os("M19_NATIVE_EVIDENCE_DIR") {let dir=std::path::PathBuf::from(dir);std::fs::create_dir_all(&dir).unwrap();std::fs::write(dir.join(format!("{seed}-{}-portable.json",scenario.label())),record.to_json().unwrap()).unwrap();}
    (s,record)
}

pub const ACT3:&[EncounterId]=&[EncounterId::JawWormHorde,EncounterId::OrbWalker,EncounterId::ThreeShapes,EncounterId::FourShapes,EncounterId::SphereAndTwoShapes,EncounterId::ThreeDarklings,EncounterId::SpireGrowth,EncounterId::Transient,EncounterId::Maw,EncounterId::WrithingMass,EncounterId::GiantHead,EncounterId::Nemesis,EncounterId::Reptomancer,EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca];

/// Constructor differential only: uses the main-owned constructor without caller rollout.
pub fn diagnostic_run_for_config(cfg:&RunSessionConfig,seed:u64)->Run {
    let RunBossHistory::M19DefectAct3{act1,act2,act3}=cfg.boss_history else{panic!("M19 config required")};
    caller::make_run(seed,caller::Scenario{
        act1_boss:match act1{RunAct1BossTarget::Guardian=>EncounterId::Guardian,RunAct1BossTarget::Hexaghost=>EncounterId::Hexaghost,RunAct1BossTarget::SlimeBoss=>EncounterId::SlimeBoss},
        act2_boss:match act2{RunAct2BossTarget::Champ=>EncounterId::Champ,RunAct2BossTarget::BronzeAutomaton=>EncounterId::BronzeAutomaton,RunAct2BossTarget::Collector=>EncounterId::Collector},
        act3_boss:match act3{RunAct3BossTarget::AwakenedOne=>EncounterId::AwakenedOne,RunAct3BossTarget::TimeEater=>EncounterId::TimeEater,RunAct3BossTarget::DonuAndDeca=>EncounterId::DonuAndDeca},
    }).unwrap()
}
