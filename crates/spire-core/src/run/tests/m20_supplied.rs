//! Explicitly released supplied driver and pre-release initialization/source-fault controls.
use super::super::*;
#[path="../../../examples/m20_support/mod.rs"]
#[allow(dead_code)]
mod frozen;
use diagnostics::{m20_supplied_act4,M20DiagnosticLoadout as L};
use std::path::{Path,PathBuf};
fn write_new(path:&Path,value:&serde_json::Value)->Result<(),String>{
    use std::io::Write;
    let bytes=serde_json::to_vec_pretty(value).map_err(|e|format!("serialize {}: {e}",path.display()))?;
    let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("create {}: {e}",path.display()))?;
    f.write_all(&bytes).map_err(|e|format!("write {}: {e}",path.display()))?;f.sync_all().map_err(|e|format!("sync {}: {e}",path.display()))
}
fn temp_unit_dir(label:&str)->PathBuf{
    let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();let path=std::env::temp_dir().join(format!("m20-{label}-{}-{now}",std::process::id()));std::fs::create_dir(&path).unwrap();path
}
fn first_public_choice(d:&PublicDecision,_:Option<&map::PublicMap>,_:&frozen::Caller)->Result<GameAction,String>{d.candidates.iter().find_map(|c|matches!(c.action,GameAction::Choose(_)).then(||c.action.clone())).ok_or("no public Choose".into())}
#[test]
fn m20_caller_actual_source_fault_receipt_keeps_accepted_post_raw_error_and_null_endpoint(){
    // Distinct owner-local negative fixture: an invisible source-frame change
    // after one actual dialog action. No battle, prescribed case, frozen policy
    // or portable-replay proof is executed or claimed by this unit control.
    let mut run=m20_supplied_act4(L::Defect).unwrap();run.state.keys=[false;3];
    run.state.room_loop.as_mut().unwrap().act3_end.as_mut().unwrap().phase=rooms::act3_end::Act3EndPhase::Dialog(3);
    run.state.room_loop.as_mut().unwrap().screen=rooms::view(&mut run.next_choice,RoomViewKind::Act3Ending,vec![(RoomChoiceKind::Act3Continue{stage:3},None)]).unwrap();
    let initial=frozen::supplied_provenance(&run,frozen::SuppliedLoadout::Defect,&"0".repeat(64)).unwrap();
    let pre=run.decision().unwrap();let map=run.public_map();let action=pre.candidates[0].action.clone();let transition=run.step(&action).unwrap();
    let accepted_post=run.decision().unwrap();
    run.state.room_loop.as_mut().unwrap().act3_end.as_mut().unwrap().record.visit=1;
    assert_eq!(run.decision().unwrap(),accepted_post,"the injected private source discrepancy does not rewrite public prefix data");
    let mut caller=frozen::Caller::new_m20(2).unwrap();caller.trace.push(frozen::TraceStep{decision:pre,map,action,transition});
    let result=frozen::shared::rollout_m20(&mut run,&mut caller,first_public_choice);assert!(result.is_err());assert_eq!(caller.accepted(),1);
    let error=caller.failed_error_for_diagnostics().expect("actual shared RunError captured");assert!(matches!(error,RunError::Faulted(_)));assert_eq!(caller.final_decision,Some(accepted_post.clone()));
    let directory=temp_unit_dir("source-fault-receipt");let path=frozen::persist_attempt(&directory,7,frozen::FIXED_CASES[0].1,2,&result,&caller,Some(&run),&initial).unwrap();
    let value:serde_json::Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();assert_eq!(value["failed_action"]["error"],format!("{error:?}"));
    assert!(value["endpoint"]["decision_debug"].is_null());assert_eq!(value["endpoint"]["decision_error"],format!("{:?}",run.decision().unwrap_err()));
    assert_eq!(value["steps"][0]["post"]["decision_debug"],format!("{accepted_post:?}"),"accepted prefix post precedes the next failed action");assert!(value["steps"][0]["post"]["decision_error"].is_null());
}
#[test]
#[ignore="Initialization export only; requires new retained SPIRE_M20_INITIAL_DIR. No policy/actions."]
fn m20_diagnostic_support_export_initials(){
    let directory=std::env::var_os("SPIRE_M20_INITIAL_DIR").map(PathBuf::from).expect("initial export path");std::fs::create_dir_all(&directory).unwrap();
    let snapshot=|run:&Run,kind:&str,loadout:Option<&str>,seed:u64,scenario:frozen::Scenario|serde_json::json!({
        "schema_id":"m20-initial-source-snapshot-v1","kind":kind,"loadout":loadout,"seed":seed,"scenario":{"act1_boss":scenario.act1_boss.as_str(),"act2_boss":scenario.act2_boss.as_str(),"act3_boss":scenario.act3_boss.as_str()},"action_budget":8192,
        "initial_public_sha256":frozen::initial_public_sha256(run).unwrap(),"decision_debug":format!("{:?}",run.decision().unwrap()),"map_debug":format!("{:?}",run.public_map()),"combat_checkpoint_debug":format!("{:?}",run.combat_checkpoint()),"full_state_debug":format!("{:?}",run.state),"initial_rng_debug":format!("{:?}",run.state.rng),"next_choice":run.next_choice});
    for (seed,scenario)in frozen::FIXED_CASES{let run=frozen::make_run(seed,scenario).unwrap();write_new(&directory.join(format!("normal-{seed}-{}.json",scenario.label())),&snapshot(&run,"normal",None,seed,scenario)).unwrap();}
    for (loadout,label)in [(L::Defect,"Defect"),(L::Strong,"Strong")]{let run=m20_supplied_act4(loadout).unwrap();write_new(&directory.join(format!("supplied-{label}.json")),&snapshot(&run,"supplied",Some(label),7,frozen::FIXED_CASES[0].1)).unwrap();}
}
fn verify_frozen_initial(run:&Run,path:&Path){
    let v:serde_json::Value=serde_json::from_slice(&std::fs::read(path).expect("frozen initial snapshot")).unwrap();
    assert_eq!(v["initial_public_sha256"],frozen::initial_public_sha256(run).unwrap());assert_eq!(v["full_state_debug"],format!("{:?}",run.state));assert_eq!(v["initial_rng_debug"],format!("{:?}",run.state.rng));assert_eq!(v["next_choice"],run.next_choice);assert_eq!(v["action_budget"],8192);
}
fn supplied_attempt(directory:&Path,freeze:&str,run:&mut Run,loadout:frozen::SuppliedLoadout)->(Result<frozen::M20CallerOutcome,String>,PathBuf){
    let provenance=frozen::supplied_provenance(run,loadout,freeze).unwrap();let mut caller=frozen::Caller::new_m20(8192).unwrap();
    let result=frozen::shared::rollout_m20(run,&mut caller,frozen::policy);
    let path=frozen::persist_attempt(directory,7,frozen::FIXED_CASES[0].1,8192,&result,&caller,Some(run),&provenance).expect("persist original supplied attempt before conditional decision");
    println!("M20_SUPPLIED loadout={loadout:?} outcome={result:?} accepted={} endpoint={} receipt={}",caller.accepted(),frozen::endpoint_summary(Some(run),&caller),path.display());
    (result,path)
}
#[test]
#[ignore="HOLD: global exact source/caller/native consumers/configs/resources/RNG/caps/binary freeze required; one authorized sequence only"]
fn m20_supplied_act4_runtime_first_sequence(){
    use sha2::{Digest,Sha256};
    let(directory,freeze)=frozen::required_execution_environment().expect("global execution release");
    let manifest=std::env::var_os("SPIRE_M20_FREEZE_MANIFEST").map(PathBuf::from).expect("actual frozen manifest path");let bytes=std::fs::read(manifest).unwrap();assert_eq!(format!("{:x}",Sha256::digest(&bytes)),freeze,"freeze marker binds actual retained manifest");
    let initials=std::env::var_os("SPIRE_M20_INITIAL_DIR").map(PathBuf::from).expect("frozen initial snapshots");frozen::preflight_supplied_attempt_files(&directory).unwrap();
    // Freeze/check both definitions before either outcome; no later patches.
    let mut defect=m20_supplied_act4(L::Defect).unwrap();let mut strong=m20_supplied_act4(L::Strong).unwrap();
    verify_frozen_initial(&defect,&initials.join("supplied-Defect.json"));verify_frozen_initial(&strong,&initials.join("supplied-Strong.json"));
    let(first,first_path)=supplied_attempt(&directory,&freeze,&mut defect,frozen::SuppliedLoadout::Defect);
    let genuine_death=first==Ok(frozen::M20CallerOutcome::Death)&&defect.state.terminal==Some(RunTerminal::Death)
        &&defect.state.last_combat.as_ref().is_some_and(|record|record.outcome==crate::combat::Outcome::Death&&record.visit+1==defect.state.combat_progress.settled)
        &&defect.state.combat_progress.entered==defect.state.combat_progress.settled;
    let(mut fallback,mut second,mut second_path)=("blocked",None,None);
    if genuine_death{fallback="executed";let(result,path)=supplied_attempt(&directory,&freeze,&mut strong,frozen::SuppliedLoadout::Strong);second=Some(result);second_path=Some(path);}
    else if first==Ok(frozen::M20CallerOutcome::HeartVictory){fallback="skipped";}
    let decision=serde_json::json!({"schema_id":"m20-supplied-conditional-decision-v1","freeze_manifest_sha256":freeze,"first_outcome":first.as_ref().ok().map(ToString::to_string),"first_error":first.as_ref().err(),"first_receipt":first_path.file_name().unwrap().to_str().unwrap(),"genuine_supplied_death":genuine_death,"fallback":fallback,"fallback_outcome":second.as_ref().and_then(|r|r.as_ref().ok()).map(ToString::to_string),"fallback_error":second.as_ref().and_then(|r|r.as_ref().err()),"fallback_receipt":second_path.as_ref().map(|p|p.file_name().unwrap().to_str().unwrap())});
    write_new(&directory.join("supplied-conditional-decision.json"),&decision).unwrap();println!("M20_SUPPLIED_CONDITIONAL {decision}");
    assert_ne!(fallback,"blocked","non-Death/non-success first outcome is retained; fallback stays unexecuted");
    if let Some(result)=second{assert!(matches!(result,Ok(frozen::M20CallerOutcome::Death|frozen::M20CallerOutcome::HeartVictory)),"fallback incomplete: {result:?}");}
}

#[test]
fn m20_supplied_anchor_opening_retains_exact_recorded_prefix_then_reaches_quiescence(){
    // Replay only the original seven recorded setup actions, with no policy or
    // new episode. The next Move isolates the original Anchor queue fault.
    let mut run=m20_supplied_act4(L::Defect).unwrap();
    for action in [GameAction::Choose(ChoiceId(2)),GameAction::Move(NodeId(2751463505)),
        GameAction::Choose(ChoiceId(3)),GameAction::Move(NodeId(2751463506)),
        GameAction::Choose(ChoiceId(9)),GameAction::Choose(ChoiceId(6)),GameAction::Choose(ChoiceId(12))]{
        run.step(&action).unwrap();
    }
    assert_eq!((run.state.act,run.state.floor,run.state.gold),(4,52,0));
    assert!(run.state.relics.contains(&crate::content::RelicId::RunicCapacitor));
    assert_eq!((run.state.combat_progress.entered,run.state.combat_progress.settled),(1,1));
    let action=GameAction::Move(NodeId(2751463507));let transition=run.step(&action).expect("Anchor's bottom Block must resolve before the opening Decision marker");
    assert!(transition.committed_events.contains(&CommittedEvent::FloorEntered{act:4,floor:53}));
    assert_eq!((run.state.combat_progress.entered,run.state.combat_progress.settled),(2,1));
    let checkpoint=run.state.combat.as_ref().unwrap().public_full().unwrap();assert_eq!(checkpoint.player.block,10);
    assert_eq!(checkpoint.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),vec![110,160]);
    assert_eq!(checkpoint.character.orbs.len(),6);let snapshot=run.clone();let decision=run.decision().unwrap();assert_eq!(run.decision().unwrap(),decision);assert_eq!(run,snapshot);
    assert!(run.step(&action).is_err());assert_eq!(run,snapshot,"rejected stale Move does not rerun Anchor or enter a second encounter");
}
