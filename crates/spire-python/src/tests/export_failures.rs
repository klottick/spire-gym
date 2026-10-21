use std::sync::Arc;
use spire_env::{CombatPool, ScenarioCatalog, PoolConfig, ResetRequest, IntegerSeed,
    EnvError, StateReason, FaultStage, PoolStatus};
use crate::export_guard::export_committed;

#[test]
fn export_failure_faults_before_release() {
    let pool = CombatPool::new(Arc::new(ScenarioCatalog::starter()),
        PoolConfig { record_replay:true, ..PoolConfig::default() }).unwrap();
    let mut operation = pool.begin_operation().unwrap();
    operation.reset_batch(&[0], &[ResetRequest {
        seed:Some(IntegerSeed::from_be_bytes(&[0]).unwrap()),scenario_id:None }]).unwrap();
    let result:Result<(), &str> = export_committed(&mut operation, || Err("owned export failed"));
    assert_eq!(result, Err("owned export failed"));
    assert!(matches!(pool.begin_operation(), Err(EnvError::State { reason:StateReason::Busy,.. })));
    let diagnostics = operation.diagnostics().unwrap();
    assert_eq!(diagnostics.status, PoolStatus::Faulted);
    assert_eq!(diagnostics.faults[0].stage, FaultStage::Export);
    assert_eq!(diagnostics.faults[0].message, "owned export failed");
    operation.mark_export_fault("later export");
    assert_eq!(operation.diagnostics().unwrap(), diagnostics);
    drop(operation);
    assert!(matches!(pool.reset_batch(&[0], &[ResetRequest::default()]),
        Err(EnvError::State { reason:StateReason::Faulted,.. })));
}

#[test]
fn successful_export_and_rejected_input_remain_recoverable() {
    let pool = CombatPool::new(Arc::new(ScenarioCatalog::starter()), PoolConfig::default()).unwrap();
    let mut operation = pool.begin_operation().unwrap();
    assert!(operation.step_batch(&[0], &[[0,0]]).is_err());
    assert_eq!(export_committed(&mut operation, || Ok::<_, &str>(17)), Ok(17));
    assert_eq!(operation.diagnostics().unwrap().status, PoolStatus::Uninitialized);
    operation.reset_batch(&[0], &[ResetRequest::default()]).unwrap();
    assert_eq!(export_committed(&mut operation, || Ok::<_, &str>(23)), Ok(23));
    assert_eq!(operation.diagnostics().unwrap().status, PoolStatus::Active);
}

#[test]
fn runtime_unwind_never_replaces_first_export_fault() {
    let pool = CombatPool::new(Arc::new(ScenarioCatalog::starter()), PoolConfig::default()).unwrap();
    let first = {
        let mut operation = pool.begin_operation().unwrap();
        operation.mark_export_fault("first");
        operation.diagnostics().unwrap()
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _operation = pool.begin_operation().unwrap();
        panic!("later unwind");
    }));
    assert!(result.is_err());
    assert_eq!(pool.diagnostics().unwrap(), first);
}

#[test]
fn failed_explicit_drain_retains_owned_evidence() {
    let pool = CombatPool::new(Arc::new(ScenarioCatalog::starter()),
        PoolConfig { record_replay:true, ..PoolConfig::default() }).unwrap();
    pool.reset_batch(&[0], &[ResetRequest::default()]).unwrap();
    pool.close().unwrap();
    let mut operation = pool.begin_operation().unwrap();
    let mut pending = operation.take_completed_records().unwrap();
    assert_eq!(pending.len(), 1);
    let original = pending.clone();
    let failed = crate::export_guard::export_retained(&mut pending,
        |_| Err::<(), _>("conversion failed"));
    assert_eq!(failed, Err("conversion failed"));
    assert_eq!(pending, original);
    assert!(operation.take_completed_records().unwrap().is_empty());
    let delivered = crate::export_guard::export_retained(&mut pending,
        |records| Ok::<_, &str>(records.to_vec())).unwrap();
    assert_eq!(delivered, original);
    assert!(pending.is_empty());
}

#[test]
fn return_all_reset_export_fault_owns_only_current_delivery() {
    let pool = CombatPool::new(Arc::new(ScenarioCatalog::starter()),
        PoolConfig { record_replay:true, batch_size:3, ..PoolConfig::default() }).unwrap();
    pool.reset_batch(&[0,1,2], &vec![ResetRequest::default();3]).unwrap();
    // Successfully delivered abandonment belongs to an older operation.
    pool.reset_batch(&[0], &[ResetRequest::default()]).unwrap();
    let mut operation = pool.begin_operation().unwrap();
    let rows = operation.reset_batch_return_all(&[0], &[ResetRequest::default()]).unwrap();
    assert_eq!(rows.decisions.slot_indices, vec![0,1,2]);
    assert_eq!(rows.effective_seeds, vec![None]);
    let result = export_committed(&mut operation, || Err::<(), _>("reset export"));
    assert_eq!(result, Err("reset export"));
    let records = operation.take_completed_records().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].status, spire_env::RecordStatus::Abandoned);
    assert!(records[0].fault.is_none());
    assert_eq!(records[1].status, spire_env::RecordStatus::Faulted);
    assert_eq!(records[1].fault.as_ref().unwrap().stage, FaultStage::Export);
}
