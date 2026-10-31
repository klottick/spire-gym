//! Frozen finite M24 normal histories over the shared M22/M23/M13 caller owners;
//! loss-preserving receipts with invocation and executable-hash provenance.
#[path="../m23_support/mod.rs"]pub mod m23;
pub mod policy;
pub mod inputs;
pub use m23::{shared::{self,Caller,M24CallerOutcome},semantic_actions,sha,initial_value,write_new};
pub use inputs::{Scenario,SCENARIOS,SEEDS,Options,ACTION_LIMIT_MAX,parse_args,make_run,triple};
pub use policy::{choose,act_progress,PublicCallerProgress};
use spire_core::run::*;
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::path::{Path,PathBuf};
pub const CALLER_POLICY_REVISION:&str="m24-public-caller-v1";
pub const RECEIPT_SCHEMA_ID:&str="m24-watcher-act3-caller-attempt-v1";
/// Settled signal for supplied fixtures (shared with M23): a non-combat decision after the
/// fixture combat. A settled Victory is a successful fixture completion.
pub use m23::fixtures::{SETTLED,settled_victory};
pub fn policy(d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>,caller:&Caller)->Result<GameAction,String>{
    choose(d,map,&act_progress(caller.trace.iter().map(|s|(&s.decision,&s.action,&s.transition)),d.act))
}
/// Supplied interaction fixtures stop once their combat has settled (same policy otherwise).
pub fn fixture_policy(d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>,caller:&Caller)->Result<GameAction,String>{
    if d.combat.is_none(){return Err(SETTLED.into())}policy(d,map,caller)
}
pub fn attempt_path(directory:&Path,seed:u64,scenario:Scenario,label:&str,budget:usize)->PathBuf{directory.join(format!("seed-{seed}-{}-{label}-budget-{budget}.attempt.json",scenario.name()))}
pub fn case(act1:spire_core::content::EncounterId,act2:spire_core::content::EncounterId,scenario:Scenario)->Value{json!({"act1_boss":act1.as_str(),"act2_boss":act2.as_str(),"act3_boss":scenario.act3().as_str()})}
/// Exact command line and the SHA-256 of the executable actually running.
pub fn invocation()->Value{
    let exe=std::env::current_exe().ok();let hash=exe.as_ref().and_then(|p|std::fs::read(p).ok()).map(|b|format!("{:x}",Sha256::digest(&b)));
    json!({"argv":std::env::args().collect::<Vec<_>>(),"executable":exe.map(|p|p.display().to_string()),"executable_sha256":hash})
}
pub fn persist(path:&Path,seed:u64,case:Value,budget:usize,label:&str,freeze:&str,initial:&Value,caller:&Caller,run:Option<&Run>,result:&Result<M24CallerOutcome,String>)->Result<(),String>{
    let mut v=m23::receipt(RECEIPT_SCHEMA_ID,CALLER_POLICY_REVISION,seed,case,budget,label,freeze,initial,caller,run,&result.as_ref().map(ToString::to_string).map_err(Clone::clone));
    v["invocation"]=invocation();write_new(path,&v)
}
