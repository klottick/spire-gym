//! Stable ordered native batching over the shared RunSession profiles.
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::{
    EnvError, RunActionIndex, RunBoundary, RunDecisionToken, RunEpisodeRecord, RunReproduction,
    RunSession, RunSessionConfig, RunSessionError, RunStepResult,
    workers::Scheduler,
};
use crate::run_session::RunSessionStepFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunBatchSlotFault {
    pub slot: usize,
    pub error: RunSessionError,
    pub committed: bool,
    /// The session committed a valid row, but the batch could not deliver it.
    pub delivery_lost: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunBatchError {
    /// Whole-operation validation failed before any session was mutated.
    Input { slot: Option<usize>, error: RunSessionError },
    Setup(EnvError),
    /// Indexed session execution faults retained by their original owners.
    Fault { slots: Vec<RunBatchSlotFault> },
    Diagnostic { slot: usize, error: RunSessionError },
}

enum JobOutcome<T> {
    Success(T),
    Fault { error: RunSessionError, committed: bool },
}

pub struct RunBatch {
    sessions: Vec<RunSession>,
    scheduler: Scheduler,
    fault: Option<Vec<RunBatchSlotFault>>,
}

impl RunBatch {
    pub fn new_m10(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError> {
        Self::new_with(configs, workers, RunSession::new_m10)
    }

    /// Construct finite Defect Acts 1–3 lanes using the shared ordered scheduler.
    pub fn new_m22(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m22)}
    pub fn new_m23(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m23)}
    pub fn new_m24(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m24)}
    pub fn new_m20(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m20)}
    pub fn new_m19(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m19)}
    /// Construct finite Defect Acts 1–2 lanes using the shared ordered scheduler.
    pub fn new_m18(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m18)}
    pub fn new_m17(configs:Vec<RunSessionConfig>,workers:usize)->Result<Self,RunBatchError>{Self::new_with(configs,workers,RunSession::new_m17)}
    pub fn new_m12(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError> {
        Self::new_with(configs, workers, RunSession::new_m12)
    }

    pub fn new_m13(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError> {
        Self::new_with(configs, workers, RunSession::new_m13)
    }

    pub fn new_m14(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError> {
        Self::new_with(configs, workers, RunSession::new_m14)
    }

    pub fn new_m15(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError> {
        Self::new_with(configs, workers, RunSession::new_m15)
    }

    fn new_with(
        configs: Vec<RunSessionConfig>,
        workers: usize,
        constructor: fn(RunSessionConfig) -> Result<RunSession, RunSessionError>,
    ) -> Result<Self, RunBatchError> {
        if configs.is_empty() || workers == 0 {
            return Err(RunBatchError::Input {
                slot: None,
                error: RunSessionError::Input("batch size and workers must be positive"),
            });
        }
        let mut sessions = crate::error::reserved(configs.len()).map_err(RunBatchError::Setup)?;
        for (slot, config) in configs.into_iter().enumerate() {
            sessions.push(constructor(config).map_err(|error| RunBatchError::Input {
                slot: Some(slot),
                error,
            })?);
        }
        let scheduler = Scheduler::new(workers, sessions.len()).map_err(RunBatchError::Setup)?;
        Ok(Self { sessions, scheduler, fault: None })
    }

    fn check_healthy(&self) -> Result<(), RunBatchError> {
        if let Some(slots) = &self.fault {
            return Err(RunBatchError::Fault { slots: slots.clone() });
        }
        Ok(())
    }

    fn input(slot: Option<usize>, error: RunSessionError) -> RunBatchError {
        RunBatchError::Input { slot, error }
    }

    fn setup<T>(result: Result<T, EnvError>) -> Result<T, RunBatchError> {
        result.map_err(RunBatchError::Setup)
    }

    pub fn reset(&mut self, seeds: &[Option<u64>]) -> Result<Vec<RunBoundary>, RunBatchError> {
        self.check_healthy()?;
        let count = self.sessions.len();
        if seeds.len() != count {
            return Err(Self::input(None, RunSessionError::Input("reset seed count does not match batch size")));
        }
        for (slot, (session, seed)) in self.sessions.iter().zip(seeds).enumerate() {
            session.preflight_reset(*seed).map_err(|error| Self::input(Some(slot), error))?;
        }

        // All caller-owned and result/fault buffers are reserved before dispatch.
        let mut jobs = Self::setup(crate::error::reserved(count))?;
        jobs.extend_from_slice(seeds);
        let mut boundaries = Self::setup(crate::error::reserved(count))?;
        let mut faults = Self::setup(crate::error::reserved(count))?;
        let results = Self::setup(self.scheduler.map_mut(&mut self.sessions, jobs, |_, session, seed| {
            match catch_unwind(AssertUnwindSafe(|| session.reset(seed))) {
                Ok(Ok(boundary)) => JobOutcome::Success(boundary),
                Ok(Err(error)) => {
                    let retained = session.retain_batch_error(error, false);
                    JobOutcome::Fault { error: retained, committed: false }
                }
                Err(_) => {
                    let error = session.retain_batch_panic(false);
                    JobOutcome::Fault { error, committed: false }
                }
            }
        }))?;

        if results.iter().any(|result| matches!(result, JobOutcome::Fault { .. })) {
            for (slot, result) in results.into_iter().enumerate() {
                match result {
                    JobOutcome::Success(_) => {
                        let error = match self.sessions[slot].fail_delivery("Run batch reset row was not delivered") {
                            Ok(()) => RunSessionError::DeliveryFault("Run batch reset row was not delivered"),
                            Err(error) => error,
                        };
                        faults.push(RunBatchSlotFault { slot, error, committed: false, delivery_lost: true });
                    }
                    JobOutcome::Fault { error, committed } => {
                        faults.push(RunBatchSlotFault { slot, error, committed, delivery_lost: false });
                    }
                }
            }
            self.fault = Some(faults.clone());
            return Err(RunBatchError::Fault { slots: faults });
        }
        for result in results {
            if let JobOutcome::Success(boundary) = result { boundaries.push(boundary); }
        }
        Ok(boundaries)
    }

    pub fn step(
        &mut self,
        tokens: &[RunDecisionToken],
        indices: &[RunActionIndex],
    ) -> Result<Vec<RunStepResult>, RunBatchError> {
        self.check_healthy()?;
        let count = self.sessions.len();
        if tokens.len() != count || indices.len() != count {
            return Err(Self::input(None, RunSessionError::Input("step token/index count does not match batch size")));
        }
        for (slot, ((session, token), index)) in self.sessions.iter().zip(tokens).zip(indices).enumerate() {
            session.preflight_step(*token, *index).map_err(|error| Self::input(Some(slot), error))?;
        }

        let mut jobs = Self::setup(crate::error::reserved(count))?;
        jobs.extend(tokens.iter().copied().zip(indices.iter().copied()));
        let mut steps = Self::setup(crate::error::reserved(count))?;
        let mut faults = Self::setup(crate::error::reserved(count))?;
        let results = Self::setup(self.scheduler.map_mut(&mut self.sessions, jobs, |_, session, (token, index)| {
            let before = session.publication_serial();
            match catch_unwind(AssertUnwindSafe(|| session.step_with_status(token, index))) {
                Ok(Ok(result)) => JobOutcome::Success(result),
                Ok(Err(RunSessionStepFailure { error, committed })) => {
                    let retained = session.retain_batch_error(error, committed);
                    JobOutcome::Fault { error: retained, committed }
                }
                Err(_) => {
                    let committed = session.publication_serial() != before;
                    let error = session.retain_batch_panic(committed);
                    JobOutcome::Fault { error, committed }
                }
            }
        }))?;

        if results.iter().any(|result| matches!(result, JobOutcome::Fault { .. })) {
            for (slot, result) in results.into_iter().enumerate() {
                match result {
                    JobOutcome::Success(_) => {
                        let error = match self.sessions[slot].fail_delivery("Run batch step row was not delivered") {
                            Ok(()) => RunSessionError::DeliveryFault("Run batch step row was not delivered"),
                            Err(error) => error,
                        };
                        faults.push(RunBatchSlotFault { slot, error, committed: true, delivery_lost: true });
                    }
                    JobOutcome::Fault { error, committed } => {
                        faults.push(RunBatchSlotFault { slot, error, committed, delivery_lost: false });
                    }
                }
            }
            self.fault = Some(faults.clone());
            return Err(RunBatchError::Fault { slots: faults });
        }
        for result in results {
            if let JobOutcome::Success(step) = result { steps.push(step); }
        }
        Ok(steps)
    }

    pub fn record(&self, slot: usize) -> Result<Option<RunEpisodeRecord>, RunBatchError> {
        self.sessions.get(slot).ok_or_else(|| Self::input(Some(slot), RunSessionError::Input("batch slot index out of range")))?
            .record().map_err(|error| RunBatchError::Diagnostic { slot, error })
    }

    pub fn reproduction(&self, slot: usize) -> Result<RunReproduction, RunBatchError> {
        self.sessions.get(slot).ok_or_else(|| Self::input(Some(slot), RunSessionError::Input("batch slot index out of range")))?
            .reproduction().map_err(|error| RunBatchError::Diagnostic { slot, error })
    }

    pub fn close(&mut self) -> Result<(), RunBatchError> {
        for (slot, session) in self.sessions.iter_mut().enumerate() {
            session.close().map_err(|error| RunBatchError::Diagnostic { slot, error })?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl RunBatch{
    pub(crate) fn from_m20_diagnostic(configs:Vec<RunSessionConfig>,workers:usize,loadout:spire_core::run::diagnostics::M20DiagnosticLoadout)->Result<Self,RunBatchError>{
        use spire_core::run::diagnostics::M20DiagnosticLoadout;
        let constructor:fn(RunSessionConfig)->Result<RunSession,RunSessionError>=match loadout{
            M20DiagnosticLoadout::Defect=>|c|RunSession::from_m20_diagnostic(c,M20DiagnosticLoadout::Defect),
            M20DiagnosticLoadout::Strong=>|c|RunSession::from_m20_diagnostic(c,M20DiagnosticLoadout::Strong),
        };
        Self::new_with(configs,workers,constructor)
    }
}
#[cfg(test)]
mod m20_diagnostic_batch_installation{
    use super::*;
    use spire_core::run::diagnostics::M20DiagnosticLoadout;
    #[test]
    fn m20_native_diagnostic_paired_installation_and_full_reset(){
        for workers in [1,2]{for loadout in [M20DiagnosticLoadout::Defect,M20DiagnosticLoadout::Strong]{
            let config=crate::run_session::m20_test_config();
            let mut batch=RunBatch::from_m20_diagnostic(vec![config.clone();2],workers,loadout).unwrap();
            let single=RunSession::from_m20_diagnostic(config.clone(),loadout).unwrap();
            for i in 0..2{assert_eq!(batch.reproduction(i).unwrap(),single.reproduction().unwrap());assert_eq!(batch.record(i).unwrap(),single.record().unwrap());}
            let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
            assert!(batch.reset(&[None]).is_err());for i in 0..2{assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
            let reset=batch.reset(&[Some(7),None]).unwrap();
            for i in 0..2{
                let mut expected=RunSession::new_m20(config.clone()).unwrap();
                // None is a full next-ordinal reset, not an untouched slot.
                expected.reset(Some(7)).unwrap();let b=if i==0{expected.reset(Some(7)).unwrap()}else{expected.reset(None).unwrap()};
                assert_eq!(reset[i].observation,b.observation);assert_eq!(batch.reproduction(i).unwrap(),expected.reproduction().unwrap());
            }
        }}
    }
}
#[cfg(test)]
impl RunBatch{
    pub(crate) fn m20_test_tokens(&self)->Vec<RunDecisionToken>{self.sessions.iter().map(|s|s.decision().unwrap().token).collect()}
}

#[cfg(test)]
mod m22_actual_batch_tests{
 use super::*;
 use spire_core::{content::Character,combat::rng::Partition,run::diagnostics::{M22Diagnostic as D,M22DiagnosticBoss as B,M22DiagnosticBranch as R}};
 fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:crate::RunBossHistory::Act1GuardianUnseen,capacities:crate::RunCapacities::m10_default(),record_replay:true}}
 #[test]
 fn m22_batch_actual_cutoff_slot_rejects_every_peer_before_dispatch(){for workers in [1,2]{let mut b=RunBatch::new_m22(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=RunSession::from_m22_diagnostic(config(),D::Act2Ready{boss:B::Guardian,branch:R::HolyWater}).unwrap();let tokens=b.sessions.iter().map(|s|s.decision().unwrap().token).collect::<Vec<_>>();let before=[b.reproduction(0).unwrap(),b.reproduction(1).unwrap()];assert!(matches!(b.step(&tokens,&[RunActionIndex(0);2]),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::UnsupportedBoundary("M22 stops at Act2Ready")})));assert_eq!([b.reproduction(0).unwrap(),b.reproduction(1).unwrap()],before);assert!(b.fault.is_none());b.reset(&[Some(7),None]).unwrap();assert!(b.sessions.iter().all(|s|s.decision().unwrap().observation.action_mask.iter().any(|n|*n==1)));}}
 #[test]
 fn m22_batch_actual_core_fault_retains_owners_and_delivery_lost(){for workers in [1,2]{let mut b=RunBatch::new_m22(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=RunSession::from_m22_diagnostic(config(),D::BeforeStartupFault).unwrap();let tokens=b.sessions.iter().map(|s|s.decision().unwrap().token).collect::<Vec<_>>();let d=b.reproduction(1).unwrap().latest_decision.unwrap().unwrap();let i=d.candidates.iter().position(|c|matches!(c.action,spire_core::run::GameAction::Move(_))).unwrap();let error=b.step(&tokens,&[RunActionIndex(0),RunActionIndex(i as i64)]).unwrap_err();let RunBatchError::Fault{slots}=error else{panic!("expected execution fault")};assert_eq!(slots.len(),2);assert!(slots[0].committed&&slots[0].delivery_lost);assert!(slots[1].committed&&!slots[1].delivery_lost);let first=b.record(0).unwrap().unwrap();let second=b.record(1).unwrap().unwrap();assert_eq!(first.steps.len(),1);assert!(second.steps.is_empty());assert!(second.fault.as_ref().unwrap().committed);let before=[b.reproduction(0).unwrap(),b.reproduction(1).unwrap()];assert!(b.step(&tokens,&[RunActionIndex(0);2]).is_err());assert_eq!([b.reproduction(0).unwrap(),b.reproduction(1).unwrap()],before);b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed);assert!(b.record(1).unwrap().unwrap().closed);}}
}

#[cfg(test)]
mod m23_actual_batch_tests{
 //! Constructed actual core owner stages (M23Diagnostic) installed into homogeneous M23 batches; labelled, not normal play.
 use super::*;
 use spire_core::{content::Character,combat::rng::Partition,run::{GameAction,diagnostics::{M23Diagnostic as D,M23DiagnosticBoss as B,M22DiagnosticBranch as R}}};
 fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:crate::RunBossHistory::M23WatcherAct2{act1:crate::RunAct1BossTarget::Guardian,act2:crate::RunAct2BossTarget::Champ},capacities:crate::RunCapacities::m10_default(),record_replay:true}}
 fn snapshot(b:&RunBatch)->Vec<RunReproduction>{(0..b.sessions.len()).map(|i|b.reproduction(i).unwrap()).collect()}
 fn tokens(b:&RunBatch)->Vec<RunDecisionToken>{b.sessions.iter().map(|s|s.decision().unwrap().token).collect()}
 #[test]
 fn m23_batch_actual_act3ready_slot_rejects_every_peer_before_dispatch(){for workers in [1,2]{for branch in [R::HolyWater,R::Skip]{
  let mut b=RunBatch::new_m23(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=RunSession::from_m23_diagnostic(config(),D::Act3Ready{boss:B::Collector,branch}).unwrap();
  let before=snapshot(&b);assert!(b.sessions[1].decision().unwrap().observation.action_mask.iter().all(|n|*n==0));
  assert_eq!(b.step(&tokens(&b),&[RunActionIndex(0);2]),Err(RunBatchError::Input{slot:Some(1),error:RunSessionError::UnsupportedBoundary("M23 stops at Act3Ready")}));assert_eq!(snapshot(&b),before,"no peer mutation, no cutoff auto-skip");assert!(b.fault.is_none());
  b.reset(&[Some(7),None]).unwrap();assert!(b.sessions.iter().all(|s|s.decision().unwrap().observation.action_mask.iter().any(|n|*n==1)));assert_eq!(b.record(1).unwrap().unwrap().ordinal,1);
 }}}
 #[test]
 fn m23_batch_postdispatch_faults_keep_indexed_peer_delivery_with_actual_core_fault(){for workers in [1,2]{
  let mut b=RunBatch::new_m23(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=RunSession::from_m23_diagnostic(config(),D::BeforeStartupFault).unwrap();
  let t=tokens(&b);let d=b.reproduction(1).unwrap().latest_decision.unwrap().unwrap();let i=d.candidates.iter().position(|c|matches!(c.action,GameAction::Move(_))).unwrap();
  let RunBatchError::Fault{slots}=b.step(&t,&[RunActionIndex(0),RunActionIndex(i as i64)]).unwrap_err() else{panic!("expected execution fault")};assert_eq!(slots.len(),2);
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,true,false));assert!(matches!(slots[1].error,RunSessionError::Core(spire_core::run::RunError::Faulted(_))));
  let (first,second)=(b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap());assert_eq!(first.steps.len(),1);assert_eq!(first.fault.as_ref().unwrap().error.kind,crate::RunRecordErrorKind::DeliveryFault);assert!(second.steps.is_empty());let f=second.fault.as_ref().unwrap();assert!(f.committed&&!f.gameplay_recorded);
  assert!(crate::run_replay::replay_m23_diagnostic(&second,config().capacities,D::BeforeStartupFault).unwrap().final_decision_error.is_some());assert_eq!(crate::replay_run(&first,config().capacities).unwrap().accepted_count,1);
  let before=snapshot(&b);assert!(b.step(&t,&[RunActionIndex(0);2]).is_err());assert!(b.reset(&[Some(7);2]).is_err());assert_eq!(snapshot(&b),before,"retry cannot replay either action");
  b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed&&b.record(1).unwrap().unwrap().closed);
  // Reset: actual startup publication failure in the last slot (a valid config cannot core-fault create_run); the peer reset row is indexed delivery_lost.
  let mut tiny=config();tiny.capacities.actions=1;let mut b=RunBatch::new_m23(vec![config(),tiny],workers).unwrap();
  let RunBatchError::Fault{slots}=b.reset(&[Some(7);2]).unwrap_err() else{panic!("expected reset fault")};assert_eq!((slots[0].committed,slots[0].delivery_lost,slots[1].committed,slots[1].delivery_lost),(false,true,false,false));assert!(b.reset(&[Some(7);2]).is_err());
 }}
 #[test]
 fn m23_batch_worker_panic_retains_original_ownership(){for workers in [1,2]{
  let mut b=RunBatch::new_m23(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();let t=tokens(&b);
  b.sessions[1].m23_test_detach_run_owner();let peer_before=b.reproduction(0).unwrap();
  let RunBatchError::Fault{slots}=b.step(&t,&[RunActionIndex(0);2]).unwrap_err() else{panic!("expected panic fault")};
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,false,false));assert_eq!(slots[1].error,RunSessionError::DeliveryFault("Run batch session operation panicked"));
  let (first,second)=(b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap());assert_eq!(first.steps.len(),1,"successful peer keeps its accepted action");assert_eq!(first.steps[0].action,peer_before.latest_decision.unwrap().unwrap().candidates[0].action);
  assert!(second.steps.is_empty());let f=second.fault.as_ref().unwrap();assert!(!f.committed&&!f.gameplay_recorded&&f.action.is_none());
  let before=snapshot(&b);assert!(b.step(&t,&[RunActionIndex(0);2]).is_err());assert!(b.reset(&[Some(7);2]).is_err());assert_eq!(snapshot(&b),before,"no retry or replay after the panic");
  b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed&&b.record(1).unwrap().unwrap().closed);
 }}
 fn state(b:&RunBatch)->Vec<(RunReproduction,Option<RunEpisodeRecord>,u64)>{(0..b.sessions.len()).map(|i|(b.reproduction(i).unwrap(),b.record(i).unwrap(),b.sessions[i].publication_serial())).collect()}
 fn fail_reservation(n:usize){crate::error::FAIL_RESERVED_AFTER.with(|c|c.set(Some(n)));}
 fn reservation_hook_consumed()->bool{crate::error::FAIL_RESERVED_AFTER.with(|c|{let fired=c.get().is_none();c.set(None);fired})}
 #[test]
 fn m23_batch_preflight_reservation_failure_on_step_and_reset_leaves_every_slot_unchanged(){for workers in [1,2]{
  // TEST-ONLY injected reservation failure (crate::error::FAIL_RESERVED_AFTER) at each of the four pre-dispatch reservations
  // (jobs, results, faults, scheduler results) of RunBatch::step and RunBatch::reset.
  let configs=vec![config(),RunSessionConfig{boss_history:crate::RunBossHistory::M23WatcherAct2{act1:crate::RunAct1BossTarget::SlimeBoss,act2:crate::RunAct2BossTarget::Collector},..config()}];
  let mut b=RunBatch::new_m23(configs.clone(),workers).unwrap();let mut reference=RunBatch::new_m23(configs,workers).unwrap();
  b.reset(&[Some(7),Some(11)]).unwrap();reference.reset(&[Some(7),Some(11)]).unwrap();let t=tokens(&b);b.step(&t,&[RunActionIndex(0);2]).unwrap();let t2=tokens(&reference);reference.step(&t2,&[RunActionIndex(0);2]).unwrap();
  let before=state(&b);let t=tokens(&b);
  for n in 0..4{
   fail_reservation(n);assert_eq!(b.step(&t,&[RunActionIndex(0);2]),Err(RunBatchError::Setup(crate::EnvError::Setup{slot:None,message:"setup allocation exceeds available capacity".into()})),"step reservation {n}");assert!(reservation_hook_consumed(),"step reservation {n} fired");assert_eq!(state(&b),before,"step reservation {n}: owner/RNG/ordinal/record/serial unchanged");assert!(b.fault.is_none());
   fail_reservation(n);assert_eq!(b.reset(&[None,Some(13)]),Err(RunBatchError::Setup(crate::EnvError::Setup{slot:None,message:"setup allocation exceeds available capacity".into()})),"reset reservation {n}");assert!(reservation_hook_consumed(),"reset reservation {n} fired");assert_eq!(state(&b),before,"reset reservation {n}: unchanged");assert!(b.fault.is_none());
  }
  // The batch stays healthy and continues exactly like an uninjected peer batch (RNG and ordinals untouched).
  let rows=b.step(&t,&[RunActionIndex(0);2]).unwrap();let rrows=reference.step(&tokens(&reference),&[RunActionIndex(0);2]).unwrap();for (x,y) in rows.iter().zip(&rrows){assert_eq!((&x.boundary.observation,x.reward,x.terminated),(&y.boundary.observation,y.reward,y.terminated));}
  b.reset(&[None,Some(13)]).unwrap();reference.reset(&[None,Some(13)]).unwrap();for i in 0..2{assert_eq!(b.reproduction(i).unwrap(),reference.reproduction(i).unwrap());assert_eq!(b.record(i).unwrap(),reference.record(i).unwrap());}
 }}
 #[test]
 fn m23_batch_reset_worker_panic_retains_original_ownership(){for workers in [1,2]{for stage in [crate::run_session::TestResetPanic::BeforeMutation,crate::run_session::TestResetPanic::AfterStartup]{
  let mut b=RunBatch::new_m23(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();let t=tokens(&b);b.step(&t,&[RunActionIndex(0);2]).unwrap();let old=b.reproduction(1).unwrap();
  b.sessions[1].set_test_reset_panic(stage);
  let RunBatchError::Fault{slots}=b.reset(&[None,None]).unwrap_err() else{panic!("expected reset panic fault")};
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,false,true));assert_eq!(slots[0].error,RunSessionError::DeliveryFault("Run batch reset row was not delivered"));
  assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,false,false));assert_eq!(slots[1].error,RunSessionError::DeliveryFault("Run batch session operation panicked"));
  let peer=b.record(0).unwrap().unwrap();assert_eq!((peer.ordinal,peer.steps.len()),(1,0),"successful peer reset retained, row reported lost");
  let rep=b.reproduction(1).unwrap();let rec=rep.record.clone().unwrap();let f=rec.fault.as_ref().unwrap();assert!(!f.committed&&!f.gameplay_recorded&&f.action.is_none());assert_eq!(rep.fault,Some(RunSessionError::DeliveryFault("Run batch session operation panicked")));
  match stage{
   // Panic before any reset mutation: the previous episode owner, seed/ordinal, accepted prefix and decision stay exactly as they were.
   crate::run_session::TestResetPanic::BeforeMutation=>{assert_eq!((rep.seed,rep.ordinal),(old.seed,old.ordinal));assert_eq!(rep.latest_decision,old.latest_decision);assert_eq!(rep.latest_map,old.latest_map);assert_eq!(rep.last_accepted_step,old.last_accepted_step);assert_eq!(rec.steps,old.record.as_ref().unwrap().steps);}
   // Panic after the new Run started but before publication: the actual started owner is retained, never rolled back or rerolled.
   crate::run_session::TestResetPanic::AfterStartup=>{assert_eq!((rep.seed,rep.ordinal),(Some(7),1));assert!(rec.steps.is_empty());assert!(rep.latest_decision.unwrap().is_ok());assert!(rep.last_accepted_step.is_none());}
  }
  let after=snapshot(&b);assert!(b.reset(&[None,None]).is_err());assert!(b.step(&tokens_or_stale(&old),&[RunActionIndex(0);2]).is_err());assert_eq!(snapshot(&b),after,"no retry or second reset");
  b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed&&b.record(1).unwrap().unwrap().closed);
 }}}
 fn tokens_or_stale(_:&RunReproduction)->Vec<RunDecisionToken>{vec![RunDecisionToken::from([0,0]);2]}

}

#[cfg(test)]
mod m24_actual_batch_tests{
 //! Constructed actual core owner stages (M24 diagnostics) installed into homogeneous M24 batches; labelled, not normal play.
 use super::*;
 use crate::run_session::M24DiagnosticStart as S;
 use spire_core::{content::Character,combat::rng::Partition,run::{GameAction,RoomChoiceKind,RoomChoiceStatus,RoomViewKind,RunTerminal,diagnostics::{M24Diagnostic as D,M24DiagnosticBoss as B}}};
 fn config()->RunSessionConfig{RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:crate::RunBossHistory::M24WatcherAct3{act1:crate::RunAct1BossTarget::Guardian,act2:crate::RunAct2BossTarget::Champ,act3:crate::RunAct3BossTarget::TimeEater},capacities:crate::RunCapacities::m10_default(),record_replay:true}}
 fn snapshot(b:&RunBatch)->Vec<RunReproduction>{(0..b.sessions.len()).map(|i|b.reproduction(i).unwrap()).collect()}
 fn tokens(b:&RunBatch)->Vec<RunDecisionToken>{b.sessions.iter().map(|s|s.decision().unwrap().token).collect()}
 fn latest(s:&RunSession)->spire_core::run::PublicDecision{s.reproduction().unwrap().latest_decision.unwrap().unwrap()}
 /// Accept one offered room choice on a single session through the public token/index API.
 fn choose(s:&mut RunSession,kind:RoomChoiceKind){let d=latest(s);let id=d.room.as_ref().unwrap().choices.iter().find(|c|c.kind==kind&&c.status==RoomChoiceStatus::Offered).unwrap().id;let i=d.candidates.iter().position(|c|c.action==GameAction::Choose(id)).unwrap();let t=s.decision().unwrap().token;s.step(t,RunActionIndex(i as i64)).unwrap();}
 /// Constructed boss source driven through the actual ending owner to Act4Ready (all keys) or the Act3NoKeys terminal.
 fn ended(keys:[bool;3])->RunSession{let mut s=RunSession::from_m24_diagnostic(config(),S::Stage(D::BossSource{boss:B::TimeEater,keys})).unwrap();choose(&mut s,RoomChoiceKind::Proceed);for stage in 1..=3{choose(&mut s,RoomChoiceKind::Act3Continue{stage});}if keys!=[true;3]{choose(&mut s,RoomChoiceKind::Act3Continue{stage:4});}s}
 #[test]
 fn m24_batch_rejects_step_with_cutoff_slot(){for workers in [1,2]{
  for (keys,want) in [([true;3],RunSessionError::UnsupportedBoundary("M24 stops before the Act4 door")),([true,false,true],RunSessionError::Terminal)]{
   let mut b=RunBatch::new_m24(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=ended(keys);
   let d=latest(&b.sessions[1]);if keys==[true;3]{assert_eq!((d.act,d.floor,d.room.as_ref().unwrap().kind,d.terminal),(3,50,RoomViewKind::Act4Ready,None));}else{assert_eq!(d.terminal,Some(RunTerminal::Act3NoKeys));}
   assert!(b.sessions[1].decision().unwrap().observation.action_mask.iter().all(|n|*n==0));let before=snapshot(&b);
   // The whole step rejects before any peer changes; no cutoff/terminal auto-skip.
   assert_eq!(b.step(&tokens(&b),&[RunActionIndex(0);2]),Err(RunBatchError::Input{slot:Some(1),error:want.clone()}));assert_eq!(snapshot(&b),before,"no peer mutation");assert!(b.fault.is_none());
   // Replaying the Act4Ready (or Act3NoKeys) record ends at the same healthy cutoff (or terminal).
   let rec=b.record(1).unwrap().unwrap();let report=crate::run_replay::replay_m24_diagnostic(&rec,config().capacities,S::Stage(D::BossSource{boss:B::TimeEater,keys})).unwrap();assert_eq!(report.final_decision,d);assert!(report.final_decision_error.is_none());
   // Full reset only.
   b.reset(&[Some(7),None]).unwrap();assert!(b.sessions.iter().all(|s|s.decision().unwrap().observation.action_mask.iter().any(|n|*n==1)));assert_eq!(b.record(1).unwrap().unwrap().ordinal,1);
  }
 }}
 #[test]
 fn m24_batch_postdispatch_faults_with_actual_core_errors(){for workers in [1,2]{
  // Step-time core rejection (EndingPreflightReject): the slot's owner keeps its first error uncommitted; the peer's committed row is delivery_lost.
  let mut b=RunBatch::new_m24(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=RunSession::from_m24_diagnostic(config(),S::Stage(D::EndingPreflightReject)).unwrap();
  let RunBatchError::Fault{slots}=b.step(&tokens(&b),&[RunActionIndex(0);2]).unwrap_err() else{panic!("expected execution fault")};assert_eq!(slots.len(),2);
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,false,false));assert_eq!(slots[1].error,RunSessionError::Core(spire_core::run::RunError::Unsupported("choice identity exhausted")));
  let (first,second)=(b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap());assert_eq!(first.steps.len(),1);assert_eq!(first.fault.as_ref().unwrap().error.kind,crate::RunRecordErrorKind::DeliveryFault);assert!(second.steps.is_empty());assert!(!second.fault.as_ref().unwrap().committed);
  assert_eq!(crate::replay_run(&first,config().capacities).unwrap().accepted_count,1);
  let before=snapshot(&b);assert!(b.step(&tokens_or_zero(&b),&[RunActionIndex(0);2]).is_err());assert!(b.reset(&[Some(7);2]).is_err());assert_eq!(snapshot(&b),before,"retry cannot replay either action");b.close().unwrap();
  // A slot already holding the actual ending source fault rejects the whole step in preflight (indexed), nothing dispatched.
  let mut b=RunBatch::new_m24(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();b.sessions[1]=RunSession::from_m24_diagnostic(config(),S::Stage(D::EndingSourceFault)).unwrap();let before=snapshot(&b);
  let t=vec![b.sessions[0].decision().unwrap().token,RunDecisionToken::from([0,0])];let err=b.step(&t,&[RunActionIndex(0);2]).unwrap_err();
  assert!(matches!(err,RunBatchError::Input{slot:Some(1),error:RunSessionError::Core(spire_core::run::RunError::Faulted(_))}),"{err:?}");assert_eq!(snapshot(&b),before);assert!(b.reset(&[Some(7);2]).is_err(),"faulted owner cannot reset into a new loss");assert_eq!(snapshot(&b),before);
 }}
 fn tokens_or_zero(b:&RunBatch)->Vec<RunDecisionToken>{b.sessions.iter().map(|s|s.decision().map(|x|x.token).unwrap_or(RunDecisionToken::from([0,0]))).collect()}
 #[test]
 fn m24_batch_worker_panic_retains_original_ownership(){for workers in [1,2]{
  let mut b=RunBatch::new_m24(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();let t=tokens(&b);
  b.sessions[1].m23_test_detach_run_owner();let peer_before=b.reproduction(0).unwrap();
  let RunBatchError::Fault{slots}=b.step(&t,&[RunActionIndex(0);2]).unwrap_err() else{panic!("expected panic fault")};
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,false,false));assert_eq!(slots[1].error,RunSessionError::DeliveryFault("Run batch session operation panicked"));
  let (first,second)=(b.record(0).unwrap().unwrap(),b.record(1).unwrap().unwrap());assert_eq!(first.steps.len(),1);assert_eq!(first.steps[0].action,peer_before.latest_decision.unwrap().unwrap().candidates[0].action);
  assert!(second.steps.is_empty());let f=second.fault.as_ref().unwrap();assert!(!f.committed&&!f.gameplay_recorded&&f.action.is_none());
  let before=snapshot(&b);assert!(b.step(&t,&[RunActionIndex(0);2]).is_err());assert!(b.reset(&[Some(7);2]).is_err());assert_eq!(snapshot(&b),before,"no retry or replay after the panic");
  b.close().unwrap();assert!(b.record(0).unwrap().unwrap().closed&&b.record(1).unwrap().unwrap().closed);
 }}
 #[test]
 fn m24_batch_reset_worker_panic_retains_original_ownership(){for workers in [1,2]{for stage in [crate::run_session::TestResetPanic::BeforeMutation,crate::run_session::TestResetPanic::AfterStartup]{
  let mut b=RunBatch::new_m24(vec![config();2],workers).unwrap();b.reset(&[Some(7);2]).unwrap();let t=tokens(&b);b.step(&t,&[RunActionIndex(0);2]).unwrap();let old=b.reproduction(1).unwrap();
  b.sessions[1].set_test_reset_panic(stage);
  let RunBatchError::Fault{slots}=b.reset(&[None,None]).unwrap_err() else{panic!("expected reset panic fault")};
  assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,false,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,false,false));assert_eq!(slots[1].error,RunSessionError::DeliveryFault("Run batch session operation panicked"));
  let rep=b.reproduction(1).unwrap();match stage{
   crate::run_session::TestResetPanic::BeforeMutation=>{assert_eq!((rep.seed,rep.ordinal),(old.seed,old.ordinal));assert_eq!(rep.latest_decision,old.latest_decision);assert_eq!(rep.last_accepted_step,old.last_accepted_step);}
   crate::run_session::TestResetPanic::AfterStartup=>{assert_eq!((rep.seed,rep.ordinal),(Some(7),1));assert!(rep.record.unwrap().steps.is_empty());assert!(rep.last_accepted_step.is_none());}
  }
  let after=snapshot(&b);assert!(b.reset(&[None,None]).is_err());assert_eq!(snapshot(&b),after,"no retry or second reset");
 }}}
 fn state(b:&RunBatch)->Vec<(RunReproduction,Option<RunEpisodeRecord>,u64)>{(0..b.sessions.len()).map(|i|(b.reproduction(i).unwrap(),b.record(i).unwrap(),b.sessions[i].publication_serial())).collect()}
 fn fail_reservation(n:usize){crate::error::FAIL_RESERVED_AFTER.with(|c|c.set(Some(n)));}
 fn reservation_hook_consumed()->bool{crate::error::FAIL_RESERVED_AFTER.with(|c|{let fired=c.get().is_none();c.set(None);fired})}
 #[test]
 fn m24_batch_preflight_reservation_failure_on_step_and_reset_leaves_every_slot_unchanged(){for workers in [1,2]{
  // TEST-ONLY injected reservation failure (crate::error::FAIL_RESERVED_AFTER) at each pre-dispatch reservation of step and reset.
  let configs=vec![config(),RunSessionConfig{boss_history:crate::RunBossHistory::M24WatcherAct3{act1:crate::RunAct1BossTarget::SlimeBoss,act2:crate::RunAct2BossTarget::Collector,act3:crate::RunAct3BossTarget::DonuAndDeca},..config()}];
  let mut b=RunBatch::new_m24(configs.clone(),workers).unwrap();let mut reference=RunBatch::new_m24(configs,workers).unwrap();
  b.reset(&[Some(7),Some(11)]).unwrap();reference.reset(&[Some(7),Some(11)]).unwrap();let t=tokens(&b);b.step(&t,&[RunActionIndex(0);2]).unwrap();let t2=tokens(&reference);reference.step(&t2,&[RunActionIndex(0);2]).unwrap();
  let before=state(&b);let t=tokens(&b);
  for n in 0..4{
   fail_reservation(n);assert_eq!(b.step(&t,&[RunActionIndex(0);2]),Err(RunBatchError::Setup(crate::EnvError::Setup{slot:None,message:"setup allocation exceeds available capacity".into()})),"step reservation {n}");assert!(reservation_hook_consumed());assert_eq!(state(&b),before);assert!(b.fault.is_none());
   fail_reservation(n);assert_eq!(b.reset(&[None,Some(13)]),Err(RunBatchError::Setup(crate::EnvError::Setup{slot:None,message:"setup allocation exceeds available capacity".into()})),"reset reservation {n}");assert!(reservation_hook_consumed());assert_eq!(state(&b),before);assert!(b.fault.is_none());
  }
  let rows=b.step(&t,&[RunActionIndex(0);2]).unwrap();let rrows=reference.step(&tokens(&reference),&[RunActionIndex(0);2]).unwrap();for (x,y) in rows.iter().zip(&rrows){assert_eq!((&x.boundary.observation,x.reward,x.terminated),(&y.boundary.observation,y.reward,y.terminated));}
  b.reset(&[None,Some(13)]).unwrap();reference.reset(&[None,Some(13)]).unwrap();for i in 0..2{assert_eq!(b.reproduction(i).unwrap(),reference.reproduction(i).unwrap());assert_eq!(b.record(i).unwrap(),reference.record(i).unwrap());}
 }}
}
