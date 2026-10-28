#![allow(dead_code)]
#[path="../../../spire-core/examples/m17_support/mod.rs"]
pub mod caller;
use spire_core::{combat::rng::Partition,content::Character};
use spire_env::*;
pub fn config(s:caller::Scenario)->RunSessionConfig{RunSessionConfig{character:Character::Defect,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M17DefectAct1{act1:match s{caller::Scenario::Guardian|caller::Scenario::Death=>RunAct1BossTarget::Guardian,caller::Scenario::Hexaghost=>RunAct1BossTarget::Hexaghost,caller::Scenario::SlimeBoss=>RunAct1BossTarget::SlimeBoss}},capacities:RunCapacities::m10_default(),record_replay:true}}
pub fn receipts()->Vec<serde_json::Value>{
    let root=std::env::var_os("M17_CASE_RECEIPTS").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spire-core/tests/fixtures/m17"));
    caller::FIXED_CASES.iter().map(|(seed,s)|serde_json::from_slice(&std::fs::read(root.join(format!("{seed}-{}.json",s.label()))).expect("retained original fixed-case receipt")).unwrap()).collect()
}
pub fn retained()->Vec<(u64,caller::Scenario,spire_core::run::Run,caller::Caller)>{receipts().iter().zip(caller::FIXED_CASES).map(|(v,(seed,s))|{let (run,c)=caller::replay_receipt(v).unwrap();(seed,s,run,c)}).collect()}
