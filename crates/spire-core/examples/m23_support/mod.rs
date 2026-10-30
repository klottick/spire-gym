//! Frozen finite M23 normal histories over the shared M22/M13 caller owners; loss-preserving receipts.
#[path="../m22_support/mod.rs"]pub mod m22;
pub mod policy;
pub mod inputs;
pub mod fixtures;
pub use m22::shared::{self,Caller,M23CallerOutcome};
pub use m22::{semantic_actions,sha,initial_value,write_new};
pub use inputs::{Scenario,SCENARIOS,SEEDS,Options,ACTION_LIMIT_MAX,parse_args,make_run,act1};
pub use policy::{choose,act_progress,PublicCallerProgress};
use spire_core::run::*;
use serde_json::{Value,json};
use std::path::{Path,PathBuf};
pub const CALLER_POLICY_REVISION:&str="m23-public-caller-v1";
pub const RECEIPT_SCHEMA_ID:&str="m23-watcher-act2-caller-attempt-v1";
/// Accepted public progress for the decision's own act only.
pub fn policy(d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>,caller:&Caller)->Result<GameAction,String>{
    choose(d,map,&act_progress(caller.trace.iter().map(|s|(&s.decision,&s.action,&s.transition)),d.act))
}
pub fn attempt_path(directory:&Path,seed:u64,scenario:Scenario,label:&str,budget:usize)->PathBuf{directory.join(format!("seed-{seed}-{}-{label}-budget-{budget}.attempt.json",scenario.name()))}
pub fn case(act1_boss:spire_core::content::EncounterId,scenario:Scenario)->Value{json!({"act1_boss":act1_boss.as_str(),"act2_boss":scenario.act2().as_str()})}
pub fn persist(path:&Path,seed:u64,case:Value,budget:usize,label:&str,freeze:&str,initial:&Value,caller:&Caller,run:Option<&Run>,result:&Result<M23CallerOutcome,String>)->Result<(),String>{
    let live=run.map(Run::decision);let map=run.and_then(Run::public_map);
    let steps=caller.trace.iter().enumerate().map(|(i,s)|{
        let (post,post_map)=if let Some(next)=caller.trace.get(i+1){(Some(Ok(next.decision.clone())),next.map.clone())}else if caller.failed_action_for_diagnostics().is_some(){(caller.final_decision.clone().map(Ok),caller.final_map.clone())}else{(live.clone(),map.clone())};
        json!({"index":i,"candidate_index":s.decision.candidates.iter().position(|c|c.action==s.action),"action":semantic_actions::action_value(&s.action),"action_debug":format!("{:?}",s.action),"pre":{"decision_debug":format!("{:?}",s.decision),"map_debug":format!("{:?}",s.map)},"transition":{"debug":format!("{:?}",s.transition),"reward":s.transition.reward,"terminal":format!("{:?}",s.transition.terminal),"committed_events_debug":s.transition.committed_events.iter().map(|e|format!("{e:?}")).collect::<Vec<_>>()},"observed":{"effects_debug":format!("{:?}",caller.m22_observed.get(i).map(|o|&o.0)),"boundaries_debug":format!("{:?}",caller.m22_observed.get(i).map(|o|&o.1))},"post":{"decision_debug":post.as_ref().and_then(|d|d.as_ref().ok()).map(|d|format!("{d:?}")),"decision_error":post.as_ref().and_then(|d|d.as_ref().err()).map(|e|format!("{e:?}")),"map_debug":format!("{post_map:?}")}})
    }).collect::<Vec<_>>();
    let failed=caller.failed_action_for_diagnostics().map(|a|json!({"action":semantic_actions::action_value(a),"action_debug":format!("{a:?}"),"candidate_index":caller.final_decision.as_ref().and_then(|d|d.candidates.iter().position(|c|c.action==*a)),"pre_decision_debug":caller.final_decision.as_ref().map(|d|format!("{d:?}")),"pre_map_debug":format!("{:?}",caller.final_map),"error":caller.failed_error_for_diagnostics().map(|e|format!("{e:?}")),"observed_debug":format!("{:?}",caller.m22_observed.get(caller.accepted()))}));
    let endpoint=json!({"decision_debug":live.as_ref().and_then(|d|d.as_ref().ok()).map(|d|format!("{d:?}")),"decision_error":live.as_ref().and_then(|d|d.as_ref().err()).map(|e|format!("{e:?}")),"map_debug":format!("{map:?}"),"combat_checkpoint_debug":run.map(|r|format!("{:?}",r.combat_checkpoint()))});
    let bootstrap_accepted=initial["supplied_entry"]["accepted"].as_bool().unwrap_or(false) as usize;
    let bootstrap_successful=initial["supplied_entry"]["successful"].as_bool().unwrap_or(false) as usize;
    let policy_fault=usize::from(matches!(caller.failed_error_for_diagnostics(),Some(RunError::Faulted(_))));
    let policy_accepted=caller.accepted()+policy_fault;
    write_new(path,&json!({"bootstrap_accepted_count":bootstrap_accepted,"bootstrap_successful_count":bootstrap_successful,"policy_accepted_count":policy_accepted,"policy_successful_count":caller.accepted(),"policy_committed_fault_count":policy_fault,"total_actual_accepted_run_actions":bootstrap_accepted+policy_accepted,"schema_id":RECEIPT_SCHEMA_ID,"caller_revision":CALLER_POLICY_REVISION,"source_freeze_sha256":freeze,"seed":seed,"scenario":case,"provenance":{"kind":label,"initial":initial},"action_budget":budget,"outcome":result.as_ref().ok().map(ToString::to_string),"error":result.as_ref().err(),"accepted_action_count":caller.accepted(),"steps":steps,"failed_action":failed,"endpoint":endpoint}))
}

/// Public Task3 script facts from the same fixture's accepted trace.
pub fn fixture_state(caller:&Caller,d:&PublicDecision)->fixtures::ScriptState{
    use spire_core::combat::{CombatAction,SelectionOperation};
    let decisions=caller.trace.iter().map(|s|&s.decision).chain(std::iter::once(d));
    let mut s=fixtures::ScriptState::default();
    for x in decisions{if let Some(c)=&x.combat{
        s.confused_seen|=c.player.powers.iter().any(|p|p.id.0=="confused"&&p.amount>0);
        s.stasis_seen|=!c.stasis.is_empty();
    }}
    let scry=|x:&PublicDecision|x.combat.as_ref().and_then(|c|c.selection.as_ref()).is_some_and(|sel|sel.operation==SelectionOperation::Scry);
    if scry(d){
        let modal=caller.trace.iter().rev().take_while(|st|scry(&st.decision)&&st.action!=GameAction::Combat(CombatAction::ConfirmSelection)).collect::<Vec<_>>();
        s.scry_steps=modal.len();
        let opening=modal.last().map(|st|&st.decision).unwrap_or(d);
        s.scry_opening=opening.combat.as_ref().and_then(|c|c.selection.as_ref()).map(|sel|sel.candidates.iter().map(|x|x.instance.0).collect());
    }s
}
