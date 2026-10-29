//! Once-only conditional fixed-loadout diagnostics; no normal-prefix provenance.
mod m22_support;
#[cfg(feature="m22-diagnostics")]
fn execute()->Result<(),String>{
    use m22_support::*;
    use spire_core::run::diagnostics::{m22_supplied_boss,M22DiagnosticBoss as B};
    let directory=std::env::var_os("M22_TRACE_DIR").map(std::path::PathBuf::from).ok_or("M22_TRACE_DIR required")?;
    let freeze=std::env::var("M22_FREEZE_SHA256").map_err(|_|"M22_FREEZE_SHA256 required")?;
    if freeze.len()!=64||!freeze.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid freeze hash".into())}
    std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
    for scenario in SCENARIOS{if attempt_path(&directory,7,scenario,"supplied-fixed16",32768).exists(){return Err("refusing existing supplied attempt".into())}}
    let mut failures=Vec::new();
    for (scenario,boss) in [(Scenario::Guardian,B::Guardian),(Scenario::Hexaghost,B::Hexaghost),(Scenario::SlimeBoss,B::SlimeBoss)]{
        let path=attempt_path(&directory,7,scenario,"supplied-fixed16",32768);let mut caller=Caller::new_m22(32768)?;
        let mut run=m22_supplied_boss(boss).map_err(|e|format!("{e:?}"))?;let mut initial=initial_value(&run)?;initial["diagnostic_checkpoint_before_bootstrap"]=serde_json::json!(format!("{:?}",spire_core::run::diagnostics::m22_diagnostic_checkpoint(&run).map_err(|e|format!("{e:?}"))?));
        // A supplied encounter has no ordinary/elite route. Enter its fixed real
        // boss source as bootstrap, then use the SAME frozen combat caller.
        // The actual entry/record is retained separately from policy decisions.
        let action=run.decision().map_err(|e|format!("{e:?}"))?.candidates.into_iter().find(|c|matches!(c.action,spire_core::run::GameAction::Move(_))).ok_or("missing fixed supplied boss entry")?.action;
        let entry=run.step_observed(&action);let bootstrap_successful=entry.result.is_ok();let bootstrap_accepted=bootstrap_successful||matches!(&entry.result,Err(spire_core::run::RunError::Faulted(_)));
        initial["supplied_entry"]=serde_json::json!({"scope":"supplied encounter bootstrap; excluded from policy budget/count","accepted":bootstrap_accepted,"successful":bootstrap_successful,"action":semantic_actions::action_value(&action),"post_diagnostic_checkpoint_debug":format!("{:?}",spire_core::run::diagnostics::m22_diagnostic_checkpoint(&run)),"transition_debug":format!("{:?}",entry.result),"effects_debug":format!("{:?}",entry.effects),"boundaries_debug":format!("{:?}",entry.boundaries),"post_public_debug":format!("{:?}",run.decision()),"map_debug":format!("{:?}",run.public_map())});
        let result=match entry.result{Ok(_)=>shared::rollout_m22(&mut run,&mut caller,policy),Err(error)=>Err(format!("fixed supplied entry: {error:?}"))};persist(&path,7,scenario,32768,"supplied-fixed16",&freeze,&initial,&caller,Some(&run),&result)?;
        println!("supplied-fixed16 seed7 scenario={} result={result:?} bootstrap_accepted_count={} policy_successful_count={} policy_accepted_count={} total_actual_accepted_run_actions={}",scenario.name(),usize::from(bootstrap_accepted),caller.accepted(),caller.accepted()+usize::from(matches!(caller.failed_error_for_diagnostics(),Some(spire_core::run::RunError::Faulted(_)))),usize::from(bootstrap_accepted)+caller.accepted()+usize::from(matches!(caller.failed_error_for_diagnostics(),Some(spire_core::run::RunError::Faulted(_)))));if let Err(e)=result{failures.push(e)}
    }if failures.is_empty(){Ok(())}else{Err(failures.join("\n"))}
}
#[cfg(not(feature="m22-diagnostics"))]
fn execute()->Result<(),String>{Err("fixed supplied diagnostics require default-off m22-diagnostics".into())}
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
