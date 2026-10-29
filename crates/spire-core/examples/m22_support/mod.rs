//! Frozen finite normal histories, pure caller adapter and loss-preserving receipts.
pub mod policy;
#[path="../m13_support/mod.rs"]pub mod shared;
#[path="../shared/semantic_actions.rs"]pub mod semantic_actions;
pub use policy::{choose,PublicCallerProgress};
pub use shared::{Caller,M22CallerOutcome};
use spire_core::run::*;
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{path::{Path,PathBuf},io::Write};
pub const CALLER_POLICY_REVISION:&str="m22-public-caller-v1";
pub const RECEIPT_SCHEMA_ID:&str="m22-watcher-act1-caller-attempt-v1";
pub mod inputs;
pub use inputs::{Scenario,SCENARIOS,Options,ACTION_LIMIT_MAX,parse_args,make_run};
pub fn public_progress(caller:&Caller)->PublicCallerProgress{
    let mut p=PublicCallerProgress::default();
    for step in &caller.trace{
        for event in &step.transition.committed_events{if let CommittedEvent::CombatEntered{encounter,..}=event{
            if let Some((_,_,kind))=spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter().find(|(id,_,_)|id==encounter){
                p.ordinary_reached|=*kind==spire_core::combat::enemies::EncounterKind::Monster;p.elite_reached|=*kind==spire_core::combat::enemies::EncounterKind::Elite;
            }
        }}
        if step.decision.room.as_ref().is_some_and(|room|room.kind==RoomViewKind::Shop&&room.choices.iter().any(|row|step.action==GameAction::Choose(row.id)&&matches!(row.kind,RoomChoiceKind::Card{..})&&row.price.is_some())){p.purchased_shops.insert((step.decision.act,step.decision.floor));}
    }p
}
pub fn policy(d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>,caller:&Caller)->Result<GameAction,String>{choose(d,map,&public_progress(caller))}
pub fn sha(value:&str)->String{format!("{:x}",Sha256::digest(value.as_bytes()))}
pub fn initial_value(run:&Run)->Result<Value,String>{let d=run.decision().map_err(|e|format!("{e:?}"))?;let map=run.public_map();Ok(json!({"decision_debug":format!("{d:?}"),"map_debug":format!("{map:?}"),"combat_checkpoint_debug":format!("{:?}",run.combat_checkpoint()),"public_sha256":sha(&format!("{d:?}\n{map:?}"))}))}
pub fn attempt_path(directory:&Path,seed:u64,scenario:Scenario,label:&str,budget:usize)->PathBuf{directory.join(format!("seed-{seed}-{}-{label}-budget-{budget}.attempt.json",scenario.name()))}
pub fn write_new(path:&Path,value:&Value)->Result<(),String>{let bytes=serde_json::to_vec_pretty(value).map_err(|e|e.to_string())?;let mut f=std::fs::OpenOptions::new().create_new(true).write(true).open(path).map_err(|e|format!("{}: {e}",path.display()))?;f.write_all(&bytes).and_then(|_|f.write_all(b"\n")).and_then(|_|f.sync_all()).map_err(|e|e.to_string())}
pub fn persist(path:&Path,seed:u64,scenario:Scenario,budget:usize,label:&str,freeze:&str,initial:&Value,caller:&Caller,run:Option<&Run>,result:&Result<M22CallerOutcome,String>)->Result<(),String>{
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
    write_new(path,&json!({"bootstrap_accepted_count":bootstrap_accepted,"bootstrap_successful_count":bootstrap_successful,"policy_accepted_count":policy_accepted,"policy_successful_count":caller.accepted(),"policy_committed_fault_count":policy_fault,"total_actual_accepted_run_actions":bootstrap_accepted+policy_accepted,"schema_id":RECEIPT_SCHEMA_ID,"caller_revision":CALLER_POLICY_REVISION,"source_freeze_sha256":freeze,"seed":seed,"scenario":{"act1_boss":scenario.encounter().as_str()},"provenance":{"kind":label,"initial":initial},"action_budget":budget,"outcome":result.as_ref().ok().map(ToString::to_string),"error":result.as_ref().err(),"accepted_action_count":caller.accepted(),"steps":steps,"failed_action":failed,"endpoint":endpoint}))
}
