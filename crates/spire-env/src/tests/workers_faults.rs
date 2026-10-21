use std::sync::Arc;
use crate::*;

fn config(b:usize, workers:usize) -> PoolConfig {
    PoolConfig { batch_size:b, workers, autoreset:AutoReset::SameStep, ..PoolConfig::default() }
}
fn requests(b:usize) -> Vec<ResetRequest> {
    (0..b).map(|i| ResetRequest { seed:Some(IntegerSeed::from_be_bytes(&[11+i as u8]).unwrap()),
        scenario_id:None }).collect()
}

// Catches global/default scheduling, slot-dependent seeds, completion-order outputs,
// and replacement boundaries that differ from direct serial execution.
#[test]
fn worker_counts_and_slot_permutations_match() {
    for b in [1,3,8] {
        for workers in [1,2,4] {
            let catalog=Arc::new(ScenarioCatalog::starter());
            let serial=CombatPool::new(catalog.clone(),config(b,1)).unwrap();
            let parallel=CombatPool::new(catalog,config(b,workers)).unwrap();
            let positions:Vec<_>=(0..b).collect();
            let mut a=serial.reset_batch(&positions,&requests(b)).unwrap().decisions;
            let mut z=parallel.reset_batch(&positions,&requests(b)).unwrap().decisions;
            // Move whole owned jobs/slots, never scalar-seed-expand after movement.
            serial.state.lock().unwrap().slots.reverse();
            parallel.state.lock().unwrap().slots.reverse();
            a.tokens=serial.read_rows(&positions).unwrap().tokens;
            z.tokens=parallel.read_rows(&positions).unwrap().tokens;
            let mut endings=0;
            for turn in 0..100 {
                let actions:Vec<_>={let s=serial.state.lock().unwrap();s.slots.iter().enumerate().map(|(i,s)| {
                    if (turn+i)%3!=0 && s.live.as_ref().unwrap().row.action_mask[1]==1 {1} else {0}
                }).collect()};
                let x=serial.step_batch(&actions,&a.tokens).unwrap();
                let y=parallel.step_batch(&actions,&z.tokens).unwrap();
                assert_eq!(x.current.observations,y.current.observations);
                assert_eq!(x.current.infos,y.current.infos);
                assert_eq!(x.rewards,y.rewards);
                assert_eq!(x.terminated,y.terminated);assert_eq!(x.truncated,y.truncated);
                assert_eq!(x.finals.observations,y.finals.observations);
                assert_eq!(x.finals.infos,y.finals.infos);assert_eq!(x.finals.slot_indices,y.finals.slot_indices);
                let sa=serial.state.lock().unwrap();let sz=parallel.state.lock().unwrap();
                for (a,z) in sa.slots.iter().zip(&sz.slots) {
                    assert_eq!(a.live.as_ref().unwrap().combat.snapshot(),z.live.as_ref().unwrap().combat.snapshot());
                }
                endings+=x.terminated.iter().filter(|&&x|x).count();
                a=x.current;z=y.current;
            }
            assert!(endings>=b*2,"must exercise repeated autoreset");
        }
    }
}

// Catches a guard surviving into export, recursive convenience gating, premature
// release, cross-pool gating, and clearing native faults on export/unwind.
#[test]
fn busy_includes_export_and_close() {
    fn send_sync<T:Send+Sync>() {}
    send_sync::<CombatPool>();send_sync::<PoolOperation<'static>>();
    let p=Arc::new(CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(1,4)).unwrap());
    let mut op=p.begin_operation().unwrap();
    let rows=op.reset_batch(&[0],&requests(1)).unwrap();
    let batch=op.step_batch(&[0],&rows.decisions.tokens).unwrap();
    // Execution has released state but the binding's export lease is still held.
    assert!(p.state.try_lock().is_ok());
    std::thread::scope(|s| { s.spawn(|| {
        for r in [p.close(),p.read_rows(&[0]).map(|_|()),p.reset_batch(&[0],&requests(1)).map(|_|()),
            p.step_batch(&[0],&batch.current.tokens).map(|_|()),p.diagnostics().map(|_|())] {
            assert!(matches!(r,Err(EnvError::State { reason:StateReason::Busy,.. })));
        }
        let other=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(1,1)).unwrap();
        other.reset_batch(&[0],&requests(1)).unwrap();
        assert_eq!(observation_spec().descriptor()["schema_id"],OBSERVATION_SCHEMA_ID);
    }).join().unwrap(); });
    op.mark_export_fault("numeric conversion failed");
    let first=op.diagnostics().unwrap();
    assert_eq!(first.status,PoolStatus::Faulted);assert_eq!(first.faults[0].stage,FaultStage::Export);
    op.mark_export_fault("later failure");assert_eq!(op.diagnostics().unwrap(),first);
    drop(op);
    assert!(matches!(p.step_batch(&[0],&batch.current.tokens),Err(EnvError::State { reason:StateReason::Faulted,.. })));
    let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _lease=p.begin_operation().unwrap();panic!("export unwind"); }));
    assert_eq!(p.diagnostics().unwrap(),first);
    p.close().unwrap();p.close().unwrap();
    let fresh=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(1,1)).unwrap();
    { let mut op=fresh.begin_operation().unwrap();assert!(op.step_batch(&[],&[]).is_err()); }
    fresh.reset_batch(&[0],&requests(1)).unwrap();
    let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _lease=fresh.begin_operation().unwrap();panic!("unreported export panic"); }));
    assert_eq!(fresh.diagnostics().unwrap().status,PoolStatus::Faulted);
    fresh.close().unwrap();
}

// Catches early parallel return, loss of actual native RNG provenance, snapshots
// taken before quiescence and attempts to resume partially advanced slots.
#[test]
fn runtime_faults_quiesce_and_keep_first_provenance() {
    use spire_core::combat::{Combat,FaultKind};
    use spire_core::combat::rng::StreamState;
    let p=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(3,3)).unwrap();
    let mut req=requests(3);for r in &mut req {r.scenario_id=Some("starter-jaw-worm-a0-v1".into());}
    let r=p.reset_batch(&[0,1,2],&req).unwrap();
    let mut native=vec![];
    { let mut s=p.state.lock().unwrap();for i in [0,2] {
        let l=s.slots[i].live.as_mut().unwrap();let mut scenario=l.job.scenario().unwrap();
        scenario.rng.enemy_ai=StreamState::new(scenario.rng.enemy_ai.id().clone(),[3;32],1u128<<64).unwrap();
        l.combat=Combat::new(&scenario).unwrap();
        let mut core=Combat::new(&scenario).unwrap();
        assert_eq!(core.step(&spire_core::combat::Action::EndTurn).unwrap().status,spire_core::combat::TransitionStatus::Fault);
        native.push(core.snapshot());
    } }
    let f=match p.step_batch(&[0;3],&r.decisions.tokens) { Err(EnvError::Fault { records })=>records,x=>panic!("{x:?}") };
    assert_eq!(f.iter().map(|f|f.slot).collect::<Vec<_>>(),[Some(0),Some(2)]);
    let d=p.diagnostics().unwrap();assert_eq!(d.status,PoolStatus::Faulted);assert_eq!(d.faults,f);
    let s=p.state.lock().unwrap();assert_eq!(s.slots[1].live.as_ref().unwrap().steps,1);
    for i in [0,2] {
        let l=s.slots[i].live.as_ref().unwrap();let fault=d.faults.iter().find(|f|f.slot==Some(i)).unwrap();
        assert_eq!(fault.stage,FaultStage::Resolution);assert_eq!(fault.core_fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
        assert_eq!(fault.core_fault,l.combat.fault_record());assert_eq!(d.slots[i].snapshot,Some(l.combat.snapshot()));
        assert_eq!(d.slots[i].snapshot.as_ref(),Some(&native[if i==0 {0} else {1}]));
        assert!(fault.core_fault.as_ref().unwrap().resolved_effects>0);
        assert!(matches!(Combat::restore(d.slots[i].snapshot.as_ref().unwrap()),Err(spire_core::combat::RestoreError::Faulted)));
        assert_eq!(l.steps,0);assert_eq!(d.slots[i].next_ordinal,Some(1));
    }
    drop(s);
    { let mut lease=p.begin_operation().unwrap();lease.mark_export_fault("must not replace core fault"); }
    assert_eq!(p.diagnostics().unwrap(),d);
    for result in [p.read_rows(&[0]).map(|_|()),p.reset_batch(&[0],&requests(1)).map(|_|()),p.step_batch(&[0;3],&r.decisions.tokens).map(|_|())] {
        assert!(matches!(result,Err(EnvError::State { reason:StateReason::Faulted,.. })));
    }
    p.close().unwrap();
}

// Everything below is compiled only as private env unit-test support. There is
// no production failure backend, cursor setting, callback or scheduling override.
#[derive(Clone)]
pub(crate) struct Injection {
    pub stage:FaultStage,
    pub panic:bool,
    pub start:Arc<std::sync::Barrier>,
    pub finish:Arc<std::sync::Barrier>,
    pub first:bool,
    pub slot:usize,
    pub completed:Arc<std::sync::Mutex<Vec<usize>>>,
    pub worker_ids:Arc<Vec<std::thread::ThreadId>>,
}
impl Injection {
    pub(crate) fn enter(&self) {
        self.start.wait();
        assert!(self.worker_ids.contains(&std::thread::current().id()),"dispatch must use this pool's private workers");
    }
    pub(crate) fn at(&self,stage:FaultStage) -> Result<(),EnvError> {
        if self.stage!=stage {return Ok(());}
        if self.panic {panic!("private stage panic");}
        Err(EnvError::Fault { records:vec![SlotFault { slot:None,stage,
            message:"private stage failure".into(),core_fault:None }] })
    }
    pub(crate) fn complete(&self) {
        if !self.first {self.finish.wait();}
        self.completed.lock().unwrap().push(self.slot);
        if self.first {self.finish.wait();}
    }
}

// Catches parallel Result short-circuiting, uncaught worker panic and errors
// selected in worker completion order rather than deterministic slot order.
#[test]
fn injected_stages_quiesce_with_reversed_completion() {
    for stage in [FaultStage::Resolution,FaultStage::Encoding,FaultStage::Reset] {
        let mut t=super::support::template();t.encounter=spire_core::combat::EnemyKind::JawWorm;
        t.enemy_hp=42;t.player_hp=1;
        let p=CombatPool::new(Arc::new(ScenarioCatalog::new(vec![t]).unwrap()),config(3,3)).unwrap();
        let r=p.reset_batch(&[0,1,2],&requests(3)).unwrap();
        let start=Arc::new(std::sync::Barrier::new(2));let finish=Arc::new(std::sync::Barrier::new(2));
        let completed=Arc::new(std::sync::Mutex::new(vec![]));
        let worker_ids=Arc::new(match &p.state.lock().unwrap().scheduler {
            crate::workers::Scheduler::Parallel(pool)=>pool.broadcast(|_|std::thread::current().id()),
            _=>panic!("fixture requires actual workers"),
        });
        { let mut s=p.state.lock().unwrap();for i in [0,2] {
            s.slots[i].injection=Some(Injection {stage,panic:i==2,start:start.clone(),finish:finish.clone(),
                first:i==2,slot:i,completed:completed.clone(),worker_ids:worker_ids.clone()});
        } }
        let faults=match p.step_batch(&[0;3],&r.decisions.tokens) {
            Err(EnvError::Fault { records })=>records,x=>panic!("{x:?}")
        };
        assert_eq!(*completed.lock().unwrap(),[2,0]);
        assert_eq!(faults.iter().map(|f|f.slot).collect::<Vec<_>>(),[Some(0),Some(2)]);
        assert_eq!(faults[0].stage,stage);assert_eq!(faults[1].stage,FaultStage::Panic);
        let d=p.diagnostics().unwrap();assert_eq!(d.faults,faults);
        assert_eq!(d.slots[1].episode_id.as_deref(),Some("episode-1"));
        assert_eq!(d.slots[0].episode_id.as_deref(),Some("episode-0"));
        assert_eq!(d.slots[2].episode_id.as_deref(),Some("episode-0"));
        let state=p.state.lock().unwrap();
        assert_eq!(state.slots[0].live.as_ref().unwrap().steps,if stage==FaultStage::Resolution {0} else {1});
        drop(state);p.close().unwrap();
    }
}

// Catches implicit all-CPU/global pools and rebuilding threads between calls.
#[test]
fn scheduler_is_private_persistent_and_effective_one_is_serial() {
    use crate::workers::Scheduler;
    for (b,w) in [(1,4),(3,1),(3,2),(8,4)] {
        let p=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(b,w)).unwrap();
        match &p.state.lock().unwrap().scheduler {
            Scheduler::Serial=>assert_eq!(b.min(w),1),
            Scheduler::Parallel(pool)=> {
                assert_eq!(pool.current_num_threads(),b.min(w));
                assert_eq!(pool.broadcast(|_|std::thread::current().id()).len(),b.min(w));
            }
        };
        let ids=||match &p.state.lock().unwrap().scheduler {
            Scheduler::Serial=>vec![],
            Scheduler::Parallel(pool)=>pool.broadcast(|_|std::thread::current().id()),
        };
        let before=ids();
        let positions:Vec<_>=(0..b).collect();
        let r=p.reset_batch(&positions,&requests(b)).unwrap();
        p.step_batch(&vec![0;b],&r.decisions.tokens).unwrap();
        assert_eq!(ids(),before);
        p.close().unwrap();
        assert!(matches!(p.state.lock().unwrap().scheduler,Scheduler::Serial),"close must release private worker resources");
    }
}

// Catches a late input/counter failure dispatching earlier rows, reset installing
// early results, or parallel workers allocating tokens in completion order.
#[test]
fn parallel_preflight_and_staged_reset_remain_atomic() {
    let p=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(8,4)).unwrap();
    let positions:Vec<_>=(0..8).collect();let req=requests(8);
    let r=p.reset_batch(&positions,&req).unwrap();
    let before=p.diagnostics().unwrap();
    let counter=p.state.lock().unwrap().serials.next();
    let mut actions=vec![0;8];actions[7]=32;
    assert!(matches!(p.step_batch(&actions,&r.decisions.tokens),Err(EnvError::Input { slot:Some(7),.. })));
    let mut tokens=r.decisions.tokens.clone();tokens[7]=[0,0];
    assert!(matches!(p.step_batch(&[0;8],&tokens),Err(EnvError::Input { slot:Some(7),.. })));
    let mut invalid=req.clone();invalid[7].scenario_id=Some("absent".into());
    assert!(p.reset_batch(&positions,&invalid).is_err());
    assert_eq!(p.diagnostics().unwrap(),before);assert_eq!(p.state.lock().unwrap().serials.next(),counter);
    let reversed:Vec<_>=(0..8).rev().collect();
    let reset=p.reset_batch(&reversed,&req).unwrap();
    assert_eq!(reset.decisions.slot_indices,reversed);
    assert!(reset.decisions.tokens.windows(2).all(|p|p[0][1]>p[1][1]));
    let current=p.read_rows(&positions).unwrap();
    let step=p.step_batch(&[0;8],&current.tokens).unwrap();
    assert!(step.current.tokens.windows(2).all(|p|p[0][1]<p[1][1]));
    p.state.lock().unwrap().serials=crate::identity::CheckedCounter::new(u64::MAX-6);
    let before=p.diagnostics().unwrap();
    assert!(matches!(p.step_batch(&[0;8],&step.current.tokens),Err(EnvError::Setup {..})));
    assert!(p.reset_batch(&positions,&req).is_err());
    assert_eq!(p.diagnostics().unwrap(),before);
    p.state.lock().unwrap().serials=crate::identity::CheckedCounter::new(100);
    {
        let seed=IntegerSeed::from_be_bytes(&[7]).unwrap();
        let prepared=crate::episode::exhaustion_test_sequence(&seed).prepare_reset(None,None,&p.catalog,p.config.partition).unwrap();
        p.state.lock().unwrap().slots[7].sequence=prepared.sequence;
    }
    let before=p.diagnostics().unwrap();
    assert!(matches!(p.step_batch(&[0;8],&step.current.tokens),Err(EnvError::Setup {slot:Some(7),..})));
    assert_eq!(p.diagnostics().unwrap(),before);
}
