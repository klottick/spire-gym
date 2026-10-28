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
