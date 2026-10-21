use std::sync::Arc;
use crate::*;
use spire_core::combat::{CardKind, EnemyKind};

fn request() -> ResetRequest {
    ResetRequest { seed:Some(IntegerSeed::from_be_bytes(&[37]).unwrap()),scenario_id:None }
}
fn pool(t:ScenarioTemplate, autoreset:AutoReset, recording:bool) -> CombatPool {
    CombatPool::new(Arc::new(ScenarioCatalog::new(vec![t]).unwrap()),
        PoolConfig { autoreset,record_replay:recording,..PoolConfig::default() }).unwrap()
}
fn boundary(p:&CombatPool, reward:f32) -> ReplayBoundary {
    let s=p.state.lock().unwrap(); let l=s.slots[0].live.as_ref().unwrap();
    ReplayBoundary { observation:l.row.clone(),info:l.info.clone(),reward }
}

// Catches dropped inputs, wrong menu replay, hidden-state restoration and loss of
// terminal in-play knowledge, context, intent history or core cleanup checkpoints.
#[test]
fn replay_matches_every_completed_boundary() {
    for enemy in [EnemyKind::Cultist,EnemyKind::JawWorm] {
        for upgraded in [false,true] {
            for autoreset in [AutoReset::Manual,AutoReset::SameStep] {
                let mut t=super::support::template();
                t.encounter=enemy;t.enemy_hp=if enemy==EnemyKind::Cultist {52} else {42};
                t.player_hp=500;t.player_max_hp=500;t.energy_per_turn=20;
                t.hand=vec![CardSpec { kind:CardKind::Strike,upgraded };8];
                t.hand.extend([CardSpec {kind:CardKind::Defend,upgraded},CardSpec {kind:CardKind::Bash,upgraded}]);
                t.context=PublicContext { act:Some(1),next_encounter:Some(EnemyKind::JawWorm) };
                let manual=pool(t.clone(),AutoReset::Manual,false);
                let mut manual_tokens=manual.reset_batch(&[0],&[request()]).unwrap().decisions.tokens;
                let p=pool(t,autoreset,true);
                let mut tokens=p.reset_batch(&[0],&[request()]).unwrap().decisions.tokens;
                let mut original=vec![boundary(&p,0.0)];
                // Repeated EndTurns expose intent history, then play to victory.
                for turn in 0..100 {
                    let index=if turn<4 {0} else {
                        let s=p.state.lock().unwrap();
                        if s.slots[0].live.as_ref().unwrap().row.action_mask[1]==1 {1} else {0}
                    };
                    assert_ne!(manual_tokens,tokens);
                    let m=manual.step_batch(&[index],&manual_tokens).unwrap();
                    let expected=boundary(&manual,m.rewards[0]);
                    let snapshot=manual.diagnostics().unwrap().slots[0].snapshot.clone().unwrap();
                    let result=p.step_batch(&[index],&tokens).unwrap();
                    let observed=if result.terminated[0] {&result.finals} else {&result.current};
                    assert_eq!(observed.observations,m.current.observations);
                    assert_eq!(observed.infos,m.current.infos);
                    assert_eq!(result.rewards,m.rewards);
                    original.push(expected);
                    if result.terminated[0] {
                        let records=p.take_completed_records().unwrap();assert_eq!(records.len(),1);
                        let record=&records[0];assert_eq!(record.status,RecordStatus::Completed);
                        let reproduced=replay(record).unwrap();
                        assert_eq!(reproduced.boundaries,original);
                        assert_eq!(reproduced.final_snapshot,snapshot);
                        assert_eq!(reproduced.fault,None);
                        assert_eq!(record.actions.len()+1,original.len());
                        let info=&original.last().unwrap().info;
                        assert_eq!(info.hp_after_cleanup,info.hp_before_healing.map(|hp|(hp+6).min(500)));
                        assert!(original.last().unwrap().observation.in_play_counts.iter().flatten().any(|&n|n==1));
                        break;
                    }
                    tokens=result.current.tokens;
                    manual_tokens=m.current.tokens;
                    assert!(turn<99,"fixture must complete");
                }
            }
        }
    }
}

// Catches marking only the replacement episode when terminal native work commits
// but same-step attached packaging fails; old queued records must retain delivery
// diagnostics without inventing a second accepted/attempted action.
#[test]
fn terminal_delivery_fault_marks_queued_episode_once() {
    let mut t=super::support::template();t.energy_per_turn=20;
    t.hand=vec![CardSpec {kind:CardKind::Strike,upgraded:true};10];
    let p=pool(t,AutoReset::SameStep,true);
    let mut tokens=p.reset_batch(&[0],&[request()]).unwrap().decisions.tokens;
    // Leave an earlier completed record undrained. A later delivery failure must
    // not retroactively relabel that successfully delivered episode.
    for _ in 0..6 {tokens=p.step_batch(&[1],&tokens).unwrap().current.tokens;}
    for _ in 0..5 {tokens=p.step_batch(&[1],&tokens).unwrap().current.tokens;}
    let mut op=p.begin_operation().unwrap();
    let b=op.step_batch(&[1],&tokens).unwrap();assert!(b.terminated[0]);
    op.mark_export_fault("terminal delivery failed");
    let old=op.take_completed_records().unwrap();assert_eq!(old.len(),2);
    assert_eq!(old[0].status,RecordStatus::Completed);assert_eq!(old[0].fault,None);
    assert_eq!(old[1].status,RecordStatus::Faulted);
    assert_eq!(old[1].actions,[1;6]);assert_eq!(old[1].attempted_action,None);
    assert_eq!(old[1].fault.as_ref().unwrap().stage,FaultStage::Export);
    let reproduced=replay(&old[1]).unwrap();
    assert_eq!(reproduced.boundaries.len(),7);assert_eq!(reproduced.fault,None);
    assert_eq!(reproduced.boundaries.last().unwrap().reward,1.0);
}

// A new lease must not inherit queued-record delivery ownership from a successful
// terminal step. Neither read-only export nor rejected input redelivers that old
// episode; only the current replacement may acquire the new export fault.
#[test]
fn later_lease_export_fault_never_relabels_delivered_terminal() {
    delivered_terminal_survives_later_export_fault(false);
}
#[test]
fn later_rejected_input_export_fault_never_relabels_delivered_terminal() {
    delivered_terminal_survives_later_export_fault(true);
}
fn delivered_terminal_survives_later_export_fault(rejected:bool) {
    let mut t=super::support::template();t.encounter=EnemyKind::JawWorm;
    t.enemy_hp=42;t.player_hp=1;
    let p=pool(t,AutoReset::SameStep,true);
    let r=p.reset_batch(&[0],&[request()]).unwrap();
    let replacement_tokens={
        let mut delivered=p.begin_operation().unwrap();
        let terminal=delivered.step_batch(&[0],&r.decisions.tokens).unwrap();
        assert_eq!(terminal.terminated,[true]);
        terminal.current.tokens
    }; // successful terminal delivery; keep its completed record undrained
    let mut later=p.begin_operation().unwrap();
    assert!(p.state.try_lock().is_ok(),"lease must not retain native state guard");
    if rejected {assert!(later.step_batch(&[32],&replacement_tokens).is_err());}
    else {
        let current=later.read_rows(&[0]).unwrap();
        assert_eq!(current.infos[0].episode_id,"episode-1");
    }
    later.mark_export_fault("later call packaging failed");
    let replacement=later.reproduction(0).unwrap().unwrap();
    assert_eq!(replacement.status,RecordStatus::Faulted);
    assert_eq!(replacement.fault.as_ref().unwrap().stage,FaultStage::Export);
    assert!(replacement.actions.is_empty());assert_eq!(replacement.attempted_action,None);
    let old=later.take_completed_records().unwrap();assert_eq!(old.len(),1);
    assert_eq!(old[0].job.episode_id(),"episode-0");
    assert_eq!(old[0].status,RecordStatus::Completed,"rejected input: {rejected}");
    assert_eq!(old[0].fault,None);assert_eq!(old[0].actions,[0]);
    assert_eq!(old[0].attempted_action,None);
}

fn injection(stage:FaultStage, panic:bool) -> super::workers_faults::Injection {
    super::workers_faults::Injection {stage,panic,start:Arc::new(std::sync::Barrier::new(1)),
        finish:Arc::new(std::sync::Barrier::new(1)),first:true,slot:0,
        completed:Arc::new(std::sync::Mutex::new(vec![])),
        worker_ids:Arc::new(vec![std::thread::current().id()]) }
}

// Injected evidence only: the record does not make unrelated external failures
// replayable. The same private seam recreates a fault attempt after a real prefix.
#[test]
fn injected_fault_attempt_is_separate_from_completed_indices() {
    for panic in [false,true] {
        let p=pool(super::support::template(),AutoReset::Manual,true);
        let r=p.reset_batch(&[0],&[request()]).unwrap();
        let b=p.step_batch(&[0],&r.decisions.tokens).unwrap();
        p.state.lock().unwrap().slots[0].injection=Some(injection(FaultStage::Resolution,panic));
        assert!(matches!(p.step_batch(&[0],&b.current.tokens),Err(EnvError::Fault {..})));
        let record=p.reproduction(0).unwrap().unwrap();
        assert_eq!(record.status,RecordStatus::Faulted);assert_eq!(record.actions,[0]);
        assert_eq!(record.attempted_action,Some(0));
        assert_eq!(record.fault.as_ref().unwrap().stage,if panic {FaultStage::Panic} else {FaultStage::Resolution});
        if !panic {
            assert!(replay(&record).is_err());
            let reproduced=crate::replay::replay_injected(&record,injection(FaultStage::Resolution,false)).unwrap();
            assert_eq!(reproduced.boundaries.len(),2);
            assert_eq!(reproduced.fault.as_ref().unwrap().stage,FaultStage::Resolution);
            assert_eq!(reproduced.final_snapshot,p.diagnostics().unwrap().slots[0].snapshot.clone().unwrap());
        }
    }
}

// Catches rejection trace mutation, disabled allocation, false completion on reset,
// and duplicating an accepted action when only attached export failed.
#[test]
fn rejected_faulted_abandoned_and_disabled_records() {
    let p=pool(super::support::template(),AutoReset::Manual,true);
    let r=p.reset_batch(&[0],&[request()]).unwrap();
    let initial=p.reproduction(0).unwrap().unwrap();
    for (a,t) in [(32,r.decisions.tokens.clone()),(0,vec![[0,0]])] {
        assert!(p.step_batch(&[a],&t).is_err());
        assert_eq!(p.reproduction(0).unwrap().unwrap(),initial);
    }
    p.step_batch(&[0],&r.decisions.tokens).unwrap();
    let active=p.reproduction(0).unwrap().unwrap();
    assert_eq!(replay(&active).unwrap().final_snapshot,p.diagnostics().unwrap().slots[0].snapshot.clone().unwrap());
    p.reset_batch(&[0],&[ResetRequest::default()]).unwrap();
    let abandoned=p.take_completed_records().unwrap();
    assert_eq!(abandoned[0].status,RecordStatus::Abandoned);
    assert_eq!(abandoned[0].actions,[0]);assert_eq!(replay(&abandoned[0]).unwrap().boundaries.len(),2);
    assert!(p.take_completed_records().unwrap().is_empty());
    let mut op=p.begin_operation().unwrap();
    let rows=op.read_rows(&[0]).unwrap();op.step_batch(&[0],&rows.tokens).unwrap();
    op.mark_export_fault("injected delivery failure");
    let fault=op.reproduction(0).unwrap().unwrap();
    assert_eq!(fault.status,RecordStatus::Faulted);assert_eq!(fault.actions,[0]);
    assert_eq!(fault.attempted_action,None);assert_eq!(fault.fault.as_ref().unwrap().stage,FaultStage::Export);
    assert_eq!(replay(&fault).unwrap().boundaries.len(),2);
    assert_eq!(replay(&fault).unwrap().fault,None); // no invented external failure
    assert!(op.take_completed_records().unwrap().is_empty());
    drop(op);
    for recording in [false,true] {
        let p=pool(super::support::template(),AutoReset::Manual,recording);
        let r=p.reset_batch(&[0],&[request()]).unwrap();p.step_batch(&[0],&r.decisions.tokens).unwrap();
        if !recording {
            assert_eq!(p.reproduction(0).unwrap(),None);
            let s=p.state.lock().unwrap();assert!(s.records.is_none());
            assert!(s.slots[0].live.as_ref().unwrap().trace.is_none());
        }
        p.close().unwrap();
        let records=p.take_completed_records().unwrap();
        assert_eq!(records.len(),usize::from(recording));
        if recording {assert_eq!(records[0].status,RecordStatus::Abandoned);}
    }
}

// Catches preflight tracing early rows before a late invalid row and losing
// unbounded undrained same-step records when slots are reused repeatedly.
#[test]
fn late_batch_errors_leave_all_traces_unchanged_and_queue_is_explicit() {
    let p=CombatPool::new(Arc::new(ScenarioCatalog::starter()),PoolConfig {
        batch_size:3,workers:3,record_replay:true,..PoolConfig::default() }).unwrap();
    let requests=vec![request();3];
    let r=p.reset_batch(&[0,1,2],&requests).unwrap();
    let before:Vec<_>=(0..3).map(|i|p.reproduction(i).unwrap()).collect();
    assert!(p.step_batch(&[0,0,32],&r.decisions.tokens).is_err());
    let mut tokens=r.decisions.tokens.clone();tokens[2]=[0,0];
    assert!(p.step_batch(&[0,0,0],&tokens).is_err());
    let mut bad=requests.clone();bad[2].scenario_id=Some("absent".into());
    assert!(p.reset_batch(&[0,1,2],&bad).is_err());
    assert_eq!((0..3).map(|i|p.reproduction(i).unwrap()).collect::<Vec<_>>(),before);
    assert!(p.take_completed_records().unwrap().is_empty());
    let mut op=p.begin_operation().unwrap();
    assert!(matches!(p.reproduction(0),Err(EnvError::State {reason:StateReason::Busy,..})));
    assert!(matches!(p.take_completed_records(),Err(EnvError::State {reason:StateReason::Busy,..})));
    assert_eq!(op.reproduction(0).unwrap(),before[0]);
    assert!(op.take_completed_records().unwrap().is_empty());drop(op);

    let mut t=super::support::template();t.encounter=EnemyKind::JawWorm;t.enemy_hp=42;t.player_hp=1;
    let p=pool(t,AutoReset::SameStep,true);
    let mut tokens=p.reset_batch(&[0],&[request()]).unwrap().decisions.tokens;
    for _ in 0..25 {
        let b=p.step_batch(&[0],&tokens).unwrap();assert_eq!(b.rewards,[-1.0]);
        assert_eq!(b.terminated,[true]);tokens=b.current.tokens;
    }
    let records=p.take_completed_records().unwrap();assert_eq!(records.len(),25);
    for (i,r) in records.iter().enumerate() {
        assert_eq!(r.job.ordinal(),i as u64);assert_eq!(r.status,RecordStatus::Completed);
        let replayed=replay(r).unwrap();assert_eq!(replayed.boundaries.len(),2);
        assert_eq!(replayed.boundaries[1].reward,-1.0);
        assert_eq!(replayed.boundaries[1].info.hp_after_cleanup,Some(0));
    }
    assert!(p.take_completed_records().unwrap().is_empty());
}

// Catches confusing an encoding/reset delivery fault after a completed native
// boundary with a failed gameplay attempt, and losing records during close.
#[test]
fn injected_completed_boundary_faults_keep_the_action_once() {
    for stage in [FaultStage::Encoding,FaultStage::Reset] {
        let mut t=super::support::template();t.encounter=EnemyKind::JawWorm;t.enemy_hp=42;t.player_hp=1;
        let p=pool(t,AutoReset::SameStep,true);
        let r=p.reset_batch(&[0],&[request()]).unwrap();
        p.state.lock().unwrap().slots[0].injection=Some(injection(stage,false));
        assert!(p.step_batch(&[0],&r.decisions.tokens).is_err());
        let record=p.reproduction(0).unwrap().unwrap();
        assert_eq!(record.actions,[0]);assert_eq!(record.attempted_action,None);
        assert_eq!(record.status,RecordStatus::Faulted);assert_eq!(record.fault.as_ref().unwrap().stage,stage);
        let replayed=replay(&record).unwrap();assert_eq!(replayed.boundaries.len(),2);
        assert_eq!(replayed.fault,None);
        assert_eq!(replayed.final_snapshot,p.diagnostics().unwrap().slots[0].snapshot.clone().unwrap());
        p.close().unwrap();p.close().unwrap();
        let queued=p.take_completed_records().unwrap();assert_eq!(queued.len(),1);assert_eq!(queued[0],record);
    }
}

// Catches queue ownership being inferred from current queue contents: draining a
// fault record and then closing must not emit the same episode a second time.
#[test]
fn draining_fault_records_then_close_never_requeues_them() {
    let p=pool(super::support::template(),AutoReset::Manual,true);
    let r=p.reset_batch(&[0],&[request()]).unwrap();
    p.state.lock().unwrap().slots[0].injection=Some(injection(FaultStage::Resolution,false));
    assert!(p.step_batch(&[0],&r.decisions.tokens).is_err());
    assert_eq!(p.take_completed_records().unwrap().len(),1);
    p.close().unwrap();
    assert!(p.take_completed_records().unwrap().is_empty());
}

// Catches compatibility checks performed after execution and catalog/template
// substitution disguised by keeping only a claimed fingerprint.
#[test]
fn incompatible_records_reject() {
    let p=pool(super::support::template(),AutoReset::Manual,true);
    p.reset_batch(&[0],&[request()]).unwrap();
    let original=p.reproduction(0).unwrap().unwrap();
    for field in 0..12 {
        let mut r=original.clone();
        match field {
            0=>r.record_schema_id.push('!'),1=>r.content_id.push('!'),2=>r.schema_id.push('!'),
            3=>r.schema_fingerprint[0]^=1,4=>r.seed_adapter_id.push('!'),5=>r.rng_contract_id.push('!'),
            6=>r.catalog_descriptor["templates"][0]["enemy_hp"]=53.into(),
            7=>r.actions.push(32),
            8=>{ let other=pool({let mut t=super::support::template();t.enemy_hp=53;t},AutoReset::Manual,true);
                other.reset_batch(&[0],&[request()]).unwrap();r.job=other.reproduction(0).unwrap().unwrap().job; },
            9=>r.status=RecordStatus::Completed,
            10=>r.attempted_action=Some(0),
            _=>r.status=RecordStatus::Faulted,
        }
        assert!(replay(&r).is_err(),"tamper {field}");
    }
}

// Catches hiding a genuinely terminal boundary under an abandoned-prefix label.
#[test]
fn terminal_prefix_cannot_claim_abandonment() {
    let mut t=super::support::template();t.encounter=EnemyKind::JawWorm;t.enemy_hp=42;t.player_hp=1;
    let p=pool(t,AutoReset::Manual,true);
    let r=p.reset_batch(&[0],&[request()]).unwrap();p.step_batch(&[0],&r.decisions.tokens).unwrap();
    let mut record=p.reproduction(0).unwrap().unwrap();record.status=RecordStatus::Abandoned;
    assert!(replay(&record).is_err());
}

// Catches counting a committed core dispatch as an exported observation boundary
// when environment encoding failed. This is private corruption evidence only.
#[test]
fn encoding_failure_retains_only_successfully_encoded_prefix() {
    let p=pool(super::support::template(),AutoReset::Manual,true);
    let r=p.reset_batch(&[0],&[request()]).unwrap();
    p.state.lock().unwrap().slots[0].live.as_mut().unwrap().knowledge.deck=[[0;2];3];
    assert!(p.step_batch(&[1],&r.decisions.tokens).is_err());
    let record=p.reproduction(0).unwrap().unwrap();
    assert_eq!(record.status,RecordStatus::Faulted);assert!(record.actions.is_empty());
    assert_eq!(record.attempted_action,Some(1));
    assert_eq!(record.fault.as_ref().unwrap().stage,FaultStage::Encoding);
    assert_eq!(p.state.lock().unwrap().slots[0].live.as_ref().unwrap().steps,1);
    assert!(replay(&record).is_err()); // fresh knowledge does not recreate corruption
}

// Catches returned records retaining mutable aliases into live recording state.
#[test]
fn owned_record_exports_do_not_mutate_retained_evidence() {
    let mut t=super::support::template();t.encounter=EnemyKind::JawWorm;t.enemy_hp=42;t.player_hp=1;
    let p=pool(t,AutoReset::Manual,true);
    let r=p.reset_batch(&[0],&[request()]).unwrap();p.step_batch(&[0],&r.decisions.tokens).unwrap();
    let retained=p.reproduction(0).unwrap().unwrap();
    let mut drained=p.take_completed_records().unwrap();
    drained[0].actions.clear();drained[0].catalog_descriptor["templates"]=serde_json::json!([]);
    assert_eq!(p.reproduction(0).unwrap().unwrap(),retained);
    p.reset_batch(&[0],&[ResetRequest::default()]).unwrap();
    assert!(p.take_completed_records().unwrap().is_empty());
}
