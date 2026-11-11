//! M35 native batch owner (design §3.1–§3.3): one ordered batch over Run sessions (every profile) and combat-training
//! sessions (Gym66 v1/v2, M21, `standard-combat-v1`). Slots may mix characters, ascensions, creations and catalog
//! entries within one observation descriptor. Rows come back in slot order, and counters, record queues and fault
//! records are applied after the join in slot order, so results never depend on the worker count or completion order.
//!
//! Resets may be masked; autoreset is DISABLED, NEXT_STEP or SAME_STEP. A fault never freezes a healthy peer: under the
//! default `Freeze` policy the batch refuses further resets and steps and reports one status per slot, every committed
//! result stays readable (`committed_result`), and nothing is retried. Under the opt-in `Isolate` policy a faulted slot
//! stays frozen behind a fault row until an explicit masked reset with a seed builds it a new session.
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde_json::{json, Value};
use spire_core::run::RewardItem;

use crate::{
    CombatPolicyBoundary, CombatTrainingCatalog, CombatTrainingSession, EnvError, RunActionIndex, RunBoundary,
    RunDecisionToken, RunEpisodeRecord, RunObservation, RunSession, RunSessionError as Error,
    workers::Scheduler,
};

/// Batch bounds (design §3.1).
pub const BATCH_MAX_SLOTS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoresetMode { Disabled, NextStep, SameStep }
impl AutoresetMode {
    pub fn label(self) -> &'static str { match self { Self::Disabled => "disabled", Self::NextStep => "next_step", Self::SameStep => "same_step" } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPolicy { Freeze, Isolate }
impl FaultPolicy {
    pub fn label(self) -> &'static str { match self { Self::Freeze => "freeze", Self::Isolate => "isolate" } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatchConfig { pub workers: usize, pub autoreset: AutoresetMode, pub fault_policy: FaultPolicy }
impl BatchConfig {
    /// `workers` with DISABLED autoreset and the `Freeze` policy (the pre-M35 batch protocol).
    pub fn new(workers: usize) -> Self { Self { workers, autoreset: AutoresetMode::Disabled, fault_policy: FaultPolicy::Freeze } }
}

/// Where a slot's fault happened. `Encode`: the action committed but its publication (encoding, capacity, reward items)
/// failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchFaultStage { Reset, Step, Autoreset, Encode, Panic }
impl BatchFaultStage {
    pub fn label(self) -> &'static str {
        match self { Self::Reset => "reset", Self::Step => "step", Self::Autoreset => "autoreset", Self::Encode => "encode", Self::Panic => "panic" }
    }
}
/// One slot's delivery status in a failed batch operation (design §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotDelivery {
    Delivered,
    /// The slot's reset or step committed and its result is readable with `committed_result`.
    CommittedUndelivered { recoverable: bool },
    NotCommitted,
    Faulted { stage: BatchFaultStage, committed: bool, gameplay_recorded: bool },
    /// Not selected by the operation (masked reset), or a skipped isolated slot.
    Unaffected,
}
impl SlotDelivery {
    pub fn label(self) -> &'static str {
        match self {
            Self::Delivered => "delivered", Self::CommittedUndelivered { .. } => "committed_undelivered", Self::NotCommitted => "not_committed",
            Self::Faulted { .. } => "faulted", Self::Unaffected => "unaffected",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotReport { pub delivery: SlotDelivery, pub error: Option<Error> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchOperation { Reset, Step, Export }
impl BatchOperation {
    pub fn label(self) -> &'static str { match self { Self::Reset => "reset", Self::Step => "step", Self::Export => "export" } }
}
/// A failed batch operation: one status per slot, in slot order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchFault { pub operation: BatchOperation, pub policy: FaultPolicy, pub slots: Vec<SlotReport> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionBatchError {
    /// Whole-operation validation failed before any session changed (no statuses).
    Input { slot: Option<usize>, error: Error },
    Setup(EnvError),
    Fault(BatchFault),
    /// A read of one slot failed.
    Diagnostic { slot: usize, error: Error },
}

/// The fault shown on an isolated slot's row (design §3.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotFaultRow { pub stage: BatchFaultStage, pub message: String, pub committed: bool }
/// One session step as the batch sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchStep<B> { pub boundary: B, pub reward: i32, pub reward_events: Vec<RewardItem>, pub terminated: bool, pub truncated: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchStepFailure { pub error: Error, pub committed: bool }
/// One slot's row of a batch operation. A reset row has reward 0 and no flags. `final_boundary` is the terminal boundary
/// of a SAME_STEP slot that terminated in this step (its row is the reset boundary). A fault row (isolate) has the zeroed
/// boundary and `fault`; on the SAME_STEP autoreset-fault composite it also carries the terminal reward and final.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotRow<B> {
    pub boundary: B, pub reward: i32, pub reward_events: Vec<RewardItem>, pub terminated: bool, pub truncated: bool,
    pub autoreset: bool, pub final_boundary: Option<B>, pub fault: Option<SlotFaultRow>,
}
impl<B> SlotRow<B> {
    fn plain(boundary: B) -> Self {
        Self { boundary, reward: 0, reward_events: Vec::new(), terminated: false, truncated: false, autoreset: false, final_boundary: None, fault: None }
    }
}
/// A finished episode's record, moved out when its slot was reset (masked or automatic).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletedRecord<R> { pub slot: usize, pub record: R }
/// The fault and record of a session an isolate resume replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetiredFault<R> { pub slot: usize, pub error: Option<Error>, pub record: Option<R> }

pub mod sealed { pub trait Sealed {} }
/// The session surface a batch drives (implemented by [`RunSession`] and [`CombatTrainingSession`] only).
#[doc(hidden)]
pub trait BatchSession: sealed::Sealed + Send + Sized {
    type Boundary: Clone + Send;
    type Record: Send;
    fn batch_preflight_reset(&self, seed: Option<u64>, entry: Option<&str>) -> Result<(), Error>;
    /// Validation of a reset of a new session built by `batch_fresh` (isolate resume).
    fn batch_preflight_fresh(&self, entry: Option<&str>) -> Result<(), Error>;
    fn batch_reset(&mut self, seed: Option<u64>, entry: Option<&str>) -> Result<Self::Boundary, Error>;
    fn batch_preflight_step(&self, token: RunDecisionToken, index: RunActionIndex) -> Result<(), Error>;
    fn batch_step(&mut self, token: RunDecisionToken, index: RunActionIndex) -> Result<BatchStep<Self::Boundary>, BatchStepFailure>;
    fn batch_publication_serial(&self) -> u64;
    fn batch_retain_error(&mut self, error: Error, committed: bool) -> Error;
    fn batch_retain_panic(&mut self, committed: bool) -> Error;
    fn batch_decision(&self) -> Result<Self::Boundary, Error>;
    fn batch_record(&self) -> Result<Option<Self::Record>, Error>;
    fn batch_fault(&self) -> Option<Error>;
    fn batch_close(&mut self) -> Result<(), Error>;
    fn batch_fingerprint(&self) -> Result<[u8; 32], Error>;
    /// The current episode's catalog entry (`None` for Run sessions).
    fn batch_current_entry(&self) -> Option<String>;
    fn batch_is_terminal(&self) -> bool;
    fn batch_fresh(&self) -> Result<Self, Error>;
    /// The fault row's boundary: `boundary`'s layout with every value, known flag, occupancy and mask entry zero.
    fn batch_zeroed(boundary: &Self::Boundary) -> Result<Self::Boundary, Error>;
}

fn zeroed_observation(o: &RunObservation) -> Result<RunObservation, Error> {
    let mut o = o.owned()?;
    for g in &mut o.groups {
        g.values.iter_mut().for_each(|v| *v = 0);
        g.known.iter_mut().for_each(|v| *v = 0);
        if let Some(occ) = &mut g.occupied { occ.iter_mut().for_each(|v| *v = 0); }
    }
    o.action_mask.iter_mut().for_each(|v| *v = 0);
    Ok(o)
}

impl sealed::Sealed for RunSession {}
impl BatchSession for RunSession {
    type Boundary = RunBoundary;
    type Record = RunEpisodeRecord;
    fn batch_preflight_reset(&self, seed: Option<u64>, entry: Option<&str>) -> Result<(), Error> {
        self.batch_preflight_fresh(entry)?;
        self.preflight_reset(seed).map(|_| ())
    }
    fn batch_preflight_fresh(&self, entry: Option<&str>) -> Result<(), Error> {
        if entry.is_some() { return Err(Error::Input("Run batches take no catalog entry")) }
        Ok(())
    }
    fn batch_reset(&mut self, seed: Option<u64>, _: Option<&str>) -> Result<RunBoundary, Error> { self.reset(seed) }
    fn batch_preflight_step(&self, token: RunDecisionToken, index: RunActionIndex) -> Result<(), Error> { self.preflight_step(token, index) }
    fn batch_step(&mut self, token: RunDecisionToken, index: RunActionIndex) -> Result<BatchStep<RunBoundary>, BatchStepFailure> {
        self.step_with_status(token, index)
            .map(|s| BatchStep { boundary: s.boundary, reward: s.reward, reward_events: s.reward_events, terminated: s.terminated, truncated: s.truncated })
            .map_err(|f| BatchStepFailure { error: f.error, committed: f.committed })
    }
    fn batch_publication_serial(&self) -> u64 { self.publication_serial() }
    fn batch_retain_error(&mut self, error: Error, committed: bool) -> Error { self.retain_batch_error(error, committed) }
    fn batch_retain_panic(&mut self, committed: bool) -> Error { self.retain_batch_panic(committed) }
    fn batch_decision(&self) -> Result<RunBoundary, Error> { self.decision() }
    fn batch_record(&self) -> Result<Option<RunEpisodeRecord>, Error> { self.record() }
    fn batch_fault(&self) -> Option<Error> { self.retained_fault() }
    fn batch_close(&mut self) -> Result<(), Error> { self.close() }
    fn batch_fingerprint(&self) -> Result<[u8; 32], Error> { self.observation_spec().map(|s| s.fingerprint()) }
    fn batch_current_entry(&self) -> Option<String> { None }
    fn batch_is_terminal(&self) -> bool { self.is_terminal() }
    fn batch_fresh(&self) -> Result<Self, Error> { self.fresh_like() }
    fn batch_zeroed(b: &RunBoundary) -> Result<RunBoundary, Error> {
        Ok(RunBoundary { observation: zeroed_observation(&b.observation)?, token: RunDecisionToken([0, 0]), terminal: None })
    }
}

impl sealed::Sealed for CombatTrainingSession {}
impl BatchSession for CombatTrainingSession {
    type Boundary = CombatPolicyBoundary;
    type Record = crate::CombatEpisodeRecord;
    fn batch_preflight_reset(&self, seed: Option<u64>, entry: Option<&str>) -> Result<(), Error> { self.preflight_reset(seed, entry) }
    fn batch_preflight_fresh(&self, entry: Option<&str>) -> Result<(), Error> { self.shared_catalog().has_entry(entry) }
    fn batch_reset(&mut self, seed: Option<u64>, entry: Option<&str>) -> Result<CombatPolicyBoundary, Error> { self.reset(seed, entry) }
    fn batch_preflight_step(&self, token: RunDecisionToken, index: RunActionIndex) -> Result<(), Error> { self.preflight_step(token, index) }
    fn batch_step(&mut self, token: RunDecisionToken, index: RunActionIndex) -> Result<BatchStep<CombatPolicyBoundary>, BatchStepFailure> {
        self.step_with_status(token, index)
            .map(|s| BatchStep { boundary: s.boundary, reward: s.reward, reward_events: Vec::new(), terminated: s.terminated, truncated: s.truncated })
            .map_err(|f| BatchStepFailure { error: f.error, committed: f.committed })
    }
    fn batch_publication_serial(&self) -> u64 { self.publication_serial() }
    fn batch_retain_error(&mut self, error: Error, committed: bool) -> Error { self.retain_batch_error(error, committed) }
    fn batch_retain_panic(&mut self, committed: bool) -> Error { self.retain_batch_panic(committed) }
    fn batch_decision(&self) -> Result<CombatPolicyBoundary, Error> { self.decision() }
    fn batch_record(&self) -> Result<Option<crate::CombatEpisodeRecord>, Error> { self.record() }
    fn batch_fault(&self) -> Option<Error> { self.retained_fault() }
    fn batch_close(&mut self) -> Result<(), Error> { self.close() }
    fn batch_fingerprint(&self) -> Result<[u8; 32], Error> { self.observation_spec().map(|s| s.fingerprint()) }
    fn batch_current_entry(&self) -> Option<String> { self.current_entry().map(str::to_owned) }
    fn batch_is_terminal(&self) -> bool { self.is_terminal() }
    fn batch_fresh(&self) -> Result<Self, Error> { self.fresh_like() }
    fn batch_zeroed(b: &CombatPolicyBoundary) -> Result<CombatPolicyBoundary, Error> {
        Ok(CombatPolicyBoundary { observation: zeroed_observation(&b.observation)?, token: RunDecisionToken([0, 0]), info: json!({}) })
    }
}

/// A slot's last committed result (design §3.3, N4): only the non-boundary parts, except a boundary that is no longer the
/// published one (a SAME_STEP final). `final_is_current`: the slot's autoreset faulted, so the retained final is also
/// the result's current boundary (stored once).
struct Retained<B> {
    reward: i32, reward_events: Vec<RewardItem>, terminated: bool, truncated: bool, autoreset: bool,
    final_boundary: Option<B>, final_is_current: bool,
}
/// `record_completed`: the session's finished episode is already in the completed queue (its reset then faulted), so an
/// isolate resume must not file that episode again as a fault record (PM final review M1).
struct SlotState<B> { pending_autoreset: bool, isolated: Option<SlotFaultRow>, retained: Option<Retained<B>>, record_completed: bool }

/// TEST-ONLY fault injection at a slot's next batch job (design §4 fault corpus). Uninhabited outside tests.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TestInject {
    /// The step fails before the engine runs (not committed).
    StepError,
    /// The step panics before the engine runs.
    StepPanic,
    /// The action commits, then the job panics.
    StepPanicAfterCommit,
    /// The action commits, then its publication fails (stage `encode`).
    StepErrorAfterCommit,
    /// The slot's next reset (masked, NEXT_STEP or SAME_STEP) fails / panics.
    ResetError,
    ResetPanic,
    /// The step commits normally, then the batch panics after the join (outside every per-slot guard).
    PostJoinPanic,
}
#[cfg(not(test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TestInject {}
enum ResetJob { Reset { seed: Option<u64>, entry: Option<String>, replace: bool, inject: Option<TestInject> }, Keep, KeepIsolated }
enum StepJob { Step { token: RunDecisionToken, index: RunActionIndex, entry: Option<String>, inject: Option<TestInject> }, Autoreset { entry: Option<String>, inject: Option<TestInject> }, Skip }
struct JobFault { stage: BatchFaultStage, error: Error, committed: bool }
enum Done<S: BatchSession> {
    Kept(Result<S::Boundary, Error>),
    KeptIsolated,
    Reset { boundary: S::Boundary, finished: Option<S::Record>, retired: Option<S> },
    Stepped(BatchStep<S::Boundary>),
    SameStep { step: BatchStep<S::Boundary>, reset: S::Boundary, finished: Option<S::Record> },
    /// SAME_STEP: the terminal step committed, then its reset faulted.
    SameStepFault { step: BatchStep<S::Boundary>, finished: Option<S::Record>, fault: JobFault },
    Autoreset { boundary: S::Boundary, finished: Option<S::Record> },
    Skipped,
    Fault(JobFault, Option<S::Record>),
    /// An isolate resume whose new session faulted on reset: the replaced session retires.
    FaultRetired { fault: JobFault, retired: S },
}

/// The fault stage of a session error after an action of `operation`.
fn stage(operation: BatchFaultStage, committed: bool, error: &Error) -> BatchFaultStage {
    if committed && !matches!(error, Error::Core(_)) { BatchFaultStage::Encode } else { operation }
}
fn fault_row(fault: &JobFault) -> SlotFaultRow {
    SlotFaultRow { stage: fault.stage, message: crate::RunRecordError::from_error(&fault.error).message, committed: fault.committed }
}
/// Reset a slot (records its finished episode first). Errors and panics are retained by the session.
fn reset_slot<S: BatchSession>(session: &mut S, seed: Option<u64>, entry: Option<&str>, operation: BatchFaultStage, inject: Option<TestInject>)
    -> Result<(S::Boundary, Option<S::Record>), (JobFault, Option<S::Record>)> {
    let finished = match session.batch_record() {
        Ok(record) => record,
        Err(error) => { let error = session.batch_retain_error(error, false); return Err((JobFault { stage: operation, error, committed: false }, None)) }
    };
    #[cfg(test)]
    if inject == Some(TestInject::ResetError) {
        let error = session.batch_retain_error(Error::DeliveryFault("TEST-ONLY injected reset fault"), false);
        return Err((JobFault { stage: operation, error, committed: false }, finished))
    }
    let _ = inject;
    match catch_unwind(AssertUnwindSafe(|| {
        #[cfg(test)]
        if inject == Some(TestInject::ResetPanic) { panic!("TEST-ONLY injected reset panic") }
        session.batch_reset(seed, entry)
    })) {
        Ok(Ok(boundary)) => Ok((boundary, finished)),
        Ok(Err(error)) => { let error = session.batch_retain_error(error, false); Err((JobFault { stage: operation, error, committed: false }, finished)) }
        Err(_) => { let error = session.batch_retain_panic(false); Err((JobFault { stage: BatchFaultStage::Panic, error, committed: false }, finished)) }
    }
}

pub struct SessionBatch<S: BatchSession> {
    pub(crate) sessions: Vec<S>,
    slots: Vec<SlotState<S::Boundary>>,
    scheduler: Scheduler,
    config: BatchConfig,
    default_entries: Vec<Option<String>>,
    frozen: Option<BatchFault>,
    started: bool,
    closed: bool,
    template: Option<S::Boundary>,
    last: Vec<SlotDelivery>,
    completed: Vec<CompletedRecord<S::Record>>,
    fault_records: Vec<RetiredFault<S::Record>>,
    #[cfg(test)]
    inject: Vec<Option<TestInject>>,
}

#[cfg(test)]
fn is_reset_inject(inject: TestInject) -> bool { matches!(inject, TestInject::ResetError | TestInject::ResetPanic) }
#[cfg(not(test))]
fn is_reset_inject(inject: TestInject) -> bool { match inject {} }

fn faulted_status(fault: JobFault) -> SlotReport {
    SlotReport { delivery: SlotDelivery::Faulted { stage: fault.stage, committed: fault.committed, gameplay_recorded: fault.committed }, error: Some(fault.error) }
}
const COMMITTED: SlotReport = SlotReport { delivery: SlotDelivery::CommittedUndelivered { recoverable: true }, error: None };
const UNAFFECTED: SlotReport = SlotReport { delivery: SlotDelivery::Unaffected, error: None };

impl<S: BatchSession> SessionBatch<S> {
    /// A batch over `sessions` (1..=256 slots, `workers >= 1`). Every slot must publish the same observation descriptor;
    /// `default_entries[slot]` is the entry an explicit reset uses when it omits one (combat training only).
    pub fn from_sessions(sessions: Vec<S>, default_entries: Vec<Option<String>>, config: BatchConfig) -> Result<Self, SessionBatchError> {
        Self::build(sessions, default_entries, config, true)
    }
    /// `uniform: false` is the pre-M35 finite-profile `RunBatch` admission (per-slot capacities; Rust rows, no stacking).
    pub(crate) fn build(sessions: Vec<S>, default_entries: Vec<Option<String>>, config: BatchConfig, uniform: bool) -> Result<Self, SessionBatchError> {
        let input = |slot, error| SessionBatchError::Input { slot, error };
        if !(1..=BATCH_MAX_SLOTS).contains(&sessions.len()) || config.workers == 0 {
            return Err(input(None, Error::Input("batch size must be 1..=256 and workers must be positive")))
        }
        if default_entries.len() != sessions.len() { return Err(input(None, Error::Input("default entry count does not match batch size"))) }
        let first = sessions[0].batch_fingerprint().map_err(|e| input(Some(0), e))?;
        for (slot, session) in sessions.iter().enumerate() {
            if uniform && session.batch_fingerprint().map_err(|e| input(Some(slot), e))? != first {
                return Err(input(Some(slot), Error::Input("batch slots must share one observation descriptor")))
            }
            session.batch_preflight_fresh(default_entries[slot].as_deref()).map_err(|e| input(Some(slot), e))?;
        }
        let scheduler = Scheduler::new(config.workers, sessions.len()).map_err(SessionBatchError::Setup)?;
        let count = sessions.len();
        let mut slots = crate::error::reserved(count).map_err(SessionBatchError::Setup)?;
        slots.extend((0..count).map(|_| SlotState { pending_autoreset: false, isolated: None, retained: None, record_completed: false }));
        Ok(Self { sessions, slots, scheduler, config, default_entries, frozen: None, started: false, closed: false, template: None,
            last: vec![SlotDelivery::Unaffected; count], completed: Vec::new(), fault_records: Vec::new(),
            #[cfg(test)]
            inject: vec![None; count] })
    }
    pub fn len(&self) -> usize { self.sessions.len() }
    pub fn is_empty(&self) -> bool { self.sessions.is_empty() }
    pub fn config(&self) -> BatchConfig { self.config }
    /// The slot's session (read-only: specs, diagnostics, reproduction).
    pub fn session(&self, slot: usize) -> Option<&S> { self.sessions.get(slot) }
    /// The batch's retained fault (`Freeze` policy, or an export failure).
    pub fn fault(&self) -> Option<&BatchFault> { self.frozen.as_ref() }
    pub fn is_closed(&self) -> bool { self.closed }
    /// Whether the slot is isolated behind a fault row (`Isolate` policy).
    pub fn slot_fault(&self, slot: usize) -> Option<&SlotFaultRow> { self.slots.get(slot).and_then(|s| s.isolated.as_ref()) }
    /// Whether the slot's next step is an automatic reset (NEXT_STEP).
    pub fn pending_autoreset(&self, slot: usize) -> bool { self.slots.get(slot).is_some_and(|s| s.pending_autoreset) }
    /// Each slot's status in the last operation that returned rows (`delivered`, `unaffected` or `faulted`).
    pub fn last_statuses(&self) -> &[SlotDelivery] { &self.last }
    /// The fault row's zeroed boundary (the published layout), once any slot has published.
    pub fn zero_boundary(&self) -> Option<&S::Boundary> { self.template.as_ref() }
    fn usable(&self) -> Result<(), SessionBatchError> {
        if self.closed { return Err(SessionBatchError::Input { slot: None, error: Error::Closed }) }
        if let Some(fault) = &self.frozen { return Err(SessionBatchError::Fault(fault.clone())) }
        Ok(())
    }
    fn input(slot: Option<usize>, error: Error) -> SessionBatchError { SessionBatchError::Input { slot, error } }
    fn setup<T>(result: Result<T, EnvError>) -> Result<T, SessionBatchError> { result.map_err(SessionBatchError::Setup) }
    fn note_template(&mut self, boundary: &S::Boundary) -> Result<(), SessionBatchError> {
        if self.template.is_none() { self.template = Some(S::batch_zeroed(boundary).map_err(|e| Self::input(None, e))?); }
        Ok(())
    }
    fn freeze(&mut self, operation: BatchOperation, slots: Vec<SlotReport>) -> SessionBatchError {
        let fault = BatchFault { operation, policy: self.config.fault_policy, slots };
        self.frozen = Some(fault.clone());
        SessionBatchError::Fault(fault)
    }
    #[cfg(test)]
    fn inject_for(&mut self, slot: usize) -> Option<TestInject> { self.inject[slot].take() }
    #[cfg(not(test))]
    fn inject_for(&mut self, _: usize) -> Option<TestInject> { None }
    /// TEST-ONLY: adopt sessions that already published (supplied starts) as a started batch.
    #[cfg(test)]
    pub(crate) fn adopt_started(&mut self) {
        self.started = true;
        let first = self.sessions[0].batch_decision().unwrap();
        self.template = Some(S::batch_zeroed(&first).unwrap());
    }
    /// TEST-ONLY: how many boundaries the slot's retained result holds by value (N4 size probe).
    #[cfg(test)]
    pub(crate) fn retained_views(&self, slot: usize) -> usize {
        self.slots[slot].retained.as_ref().map_or(0, |r| usize::from(r.final_boundary.is_some()))
    }
    /// TEST-ONLY: the batch's private scheduler.
    #[cfg(test)]
    pub(crate) fn scheduler_for_test(&self) -> &Scheduler { &self.scheduler }
    /// TEST-ONLY: fault the slot's next batch job.
    #[cfg(test)]
    pub(crate) fn inject(&mut self, slot: usize, inject: TestInject) { self.inject[slot] = Some(inject); }
    fn isolate(&mut self, slot: usize, fault: &JobFault) {
        let state = &mut self.slots[slot];
        state.retained = None;
        state.pending_autoreset = false;
        state.isolated = Some(fault_row(fault));
    }
    /// The replaced session's fault, and its record unless that episode was already queued as completed.
    fn retire(&mut self, slot: usize, old: &S) {
        let record = if self.slots[slot].record_completed { None } else { old.batch_record().ok().flatten() };
        self.fault_records.push(RetiredFault { slot, error: old.batch_fault(), record });
        self.slots[slot].record_completed = false;
    }
    /// Queue a finished episode whose slot then faulted (the session keeps that record, now stamped with the fault).
    fn complete_before_fault(&mut self, slot: usize, finished: Option<S::Record>) {
        if let Some(record) = finished { self.completed.push(CompletedRecord { slot, record }); self.slots[slot].record_completed = true; }
    }

    /// Masked reset (design §3.2). `seeds[slot]`: `Some(s)` starts ordinal 0 of `s`, `None` continues the slot's ordinal
    /// (a first `None` is the session's own default). `entries[slot]`: `None` uses the constructor's default entry for
    /// the slot, else the catalog's first (explicit resets keep the scalar rule; PM ruling on N1). Unmasked slots are
    /// untouched and their rows re-read. The first reset must select every slot. An isolated slot resumes only with an
    /// explicit seed, on a new session.
    pub fn reset(&mut self, seeds: &[Option<u64>], entries: &[Option<String>], mask: Option<&[bool]>) -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError> {
        self.usable()?;
        self.guarded(BatchOperation::Reset, |batch| batch.reset_inner(seeds, entries, mask))
    }
    fn reset_inner(&mut self, seeds: &[Option<u64>], entries: &[Option<String>], mask: Option<&[bool]>) -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError> {
        let count = self.sessions.len();
        if seeds.len() != count || entries.len() != count || mask.is_some_and(|m| m.len() != count) {
            return Err(Self::input(None, Error::Input("reset seed, entry and mask counts must match batch size")))
        }
        let selected = |slot: usize| mask.is_none_or(|m| m[slot]);
        if !self.started && (0..count).any(|slot| !selected(slot)) {
            return Err(Self::input(None, Error::Input("the first batch reset must select every slot")))
        }
        let mut jobs = Self::setup(crate::error::reserved(count))?;
        for slot in 0..count {
            let session = &self.sessions[slot];
            let isolated = self.slots[slot].isolated.is_some();
            if selected(slot) {
                let entry = entries[slot].clone().or_else(|| self.default_entries[slot].clone());
                if isolated {
                    if seeds[slot].is_none() { return Err(Self::input(Some(slot), Error::Input("faulted slot needs an explicit seed"))) }
                    session.batch_preflight_fresh(entry.as_deref()).map_err(|e| Self::input(Some(slot), e))?;
                } else {
                    session.batch_preflight_reset(seeds[slot], entry.as_deref()).map_err(|e| Self::input(Some(slot), e))?;
                }
                jobs.push(ResetJob::Reset { seed: seeds[slot], entry, replace: isolated, inject: None });
                continue
            }
            // A given entry id is validated even on an unselected slot (the pre-M35 Gym66 batch rule): atomic rejection.
            if let Some(entry) = &entries[slot] { session.batch_preflight_fresh(Some(entry)).map_err(|e| Self::input(Some(slot), e))?; }
            if isolated {
                jobs.push(ResetJob::KeepIsolated);
            } else {
                session.batch_decision().map_err(|e| Self::input(Some(slot), e))?;
                jobs.push(ResetJob::Keep);
            }
        }
        for (slot, job) in jobs.iter_mut().enumerate() { if let ResetJob::Reset { inject, .. } = job { *inject = self.inject_for(slot); } }
        let mut rows = Self::setup(crate::error::reserved(count))?;
        let mut statuses = Self::setup(crate::error::reserved(count))?;
        let results = Self::setup(self.scheduler.map_mut(&mut self.sessions, jobs, |_, session, job| match job {
            ResetJob::Keep => Done::<S>::Kept(session.batch_decision()),
            ResetJob::KeepIsolated => Done::KeptIsolated,
            ResetJob::Reset { seed, entry, replace, inject } => {
                let retired = if replace {
                    match session.batch_fresh() {
                        Ok(fresh) => Some(std::mem::replace(session, fresh)),
                        Err(error) => return Done::Fault(JobFault { stage: BatchFaultStage::Reset, error, committed: false }, None),
                    }
                } else { None };
                match (reset_slot(session, seed, entry.as_deref(), BatchFaultStage::Reset, inject), retired) {
                    (Ok((boundary, _)), Some(old)) => Done::Reset { boundary, finished: None, retired: Some(old) },
                    (Ok((boundary, finished)), None) => Done::Reset { boundary, finished, retired: None },
                    (Err((fault, _)), Some(old)) => Done::FaultRetired { fault, retired: old },
                    (Err((fault, finished)), None) => Done::Fault(fault, finished),
                }
            }
        }))?;
        for result in &results { if let Done::Reset { boundary, .. } = result { self.note_template(boundary)?; break } }
        let mut faulted = false;
        for (slot, result) in results.into_iter().enumerate() {
            match result {
                Done::Kept(Ok(boundary)) => { statuses.push(UNAFFECTED); rows.push(SlotRow::plain(boundary)); }
                Done::Kept(Err(error)) => {
                    // A read the preflight accepted; failing now is a fault of that slot.
                    faulted = true;
                    let error = self.sessions[slot].batch_retain_error(error, false);
                    let fault = JobFault { stage: BatchFaultStage::Reset, error, committed: false };
                    self.isolate(slot, &fault);
                    statuses.push(faulted_status(fault)); self.push_fault_row(&mut rows, slot);
                }
                Done::KeptIsolated => { statuses.push(UNAFFECTED); self.push_fault_row(&mut rows, slot); }
                Done::Reset { boundary, finished, retired } => {
                    if let Some(old) = &retired { self.retire(slot, old); }
                    self.slots[slot] = SlotState { pending_autoreset: false, isolated: None, retained: Some(Retained {
                        reward: 0, reward_events: Vec::new(), terminated: false, truncated: false, autoreset: false, final_boundary: None, final_is_current: false }), record_completed: false };
                    if let Some(record) = finished { self.completed.push(CompletedRecord { slot, record }); }
                    statuses.push(COMMITTED); rows.push(SlotRow::plain(boundary));
                }
                Done::Fault(fault, finished) => {
                    self.complete_before_fault(slot, finished);
                    faulted = true;
                    self.isolate(slot, &fault);
                    statuses.push(faulted_status(fault)); self.push_fault_row(&mut rows, slot);
                }
                Done::FaultRetired { fault, retired } => {
                    faulted = true;
                    self.retire(slot, &retired);
                    self.isolate(slot, &fault);
                    statuses.push(faulted_status(fault)); self.push_fault_row(&mut rows, slot);
                }
                _ => unreachable!("reset jobs produce reset outcomes"),
            }
        }
        self.started = true;
        self.finish(BatchOperation::Reset, rows, statuses, faulted)
    }

    /// One action per slot (design §3.2). Isolated slots and NEXT_STEP pending slots ignore their action and token.
    pub fn step(&mut self, tokens: &[RunDecisionToken], indices: &[RunActionIndex]) -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError> {
        self.usable()?;
        self.guarded(BatchOperation::Step, |batch| batch.step_inner(tokens, indices))
    }
    /// Design §3.3 "Panics": a panic outside the per-slot jobs (scheduler, record reads, post-join application) freezes the
    /// batch, and every slot's status is computed from its publication serial (changed: committed, not recoverable).
    fn guarded(&mut self, operation: BatchOperation, run: impl FnOnce(&mut Self) -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError>)
        -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError> {
        let before = self.sessions.iter().map(BatchSession::batch_publication_serial).collect::<Vec<_>>();
        match catch_unwind(AssertUnwindSafe(|| run(self))) {
            Ok(result) => result,
            Err(_) => {
                let mut slots = Vec::with_capacity(self.sessions.len());
                for (slot, (session, before)) in self.sessions.iter().zip(&before).enumerate() {
                    let delivery = if session.batch_publication_serial() != *before {
                        // The retained result predates this commit: drop it so no read can mix it with the new boundary.
                        self.slots[slot].retained = None;
                        SlotDelivery::CommittedUndelivered { recoverable: false }
                    } else if self.slots[slot].isolated.is_some() {
                        SlotDelivery::Unaffected
                    } else {
                        SlotDelivery::NotCommitted
                    };
                    slots.push(SlotReport { delivery, error: None });
                }
                Err(self.freeze(operation, slots))
            }
        }
    }
    fn step_inner(&mut self, tokens: &[RunDecisionToken], indices: &[RunActionIndex]) -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError> {
        if !self.started { return Err(Self::input(None, Error::NotReset)) }
        let count = self.sessions.len();
        if tokens.len() != count || indices.len() != count {
            return Err(Self::input(None, Error::Input("step token/index count does not match batch size")))
        }
        let mut jobs = Self::setup(crate::error::reserved(count))?;
        for slot in 0..count {
            let state = &self.slots[slot];
            let session = &self.sessions[slot];
            if state.isolated.is_some() { jobs.push(StepJob::Skip); continue }
            if state.pending_autoreset { jobs.push(StepJob::Autoreset { entry: session.batch_current_entry(), inject: None }); continue }
            session.batch_preflight_step(tokens[slot], indices[slot]).map_err(|e| Self::input(Some(slot), e))?;
            jobs.push(StepJob::Step { token: tokens[slot], index: indices[slot], entry: session.batch_current_entry(), inject: None });
        }
        for (slot, job) in jobs.iter_mut().enumerate() {
            match job { StepJob::Step { inject, .. } | StepJob::Autoreset { inject, .. } => *inject = self.inject_for(slot), StepJob::Skip => {} }
        }
        #[cfg(test)]
        let post_join_panic = jobs.iter().any(|j| matches!(j, StepJob::Step { inject: Some(TestInject::PostJoinPanic), .. }));
        let same_step = self.config.autoreset == AutoresetMode::SameStep;
        let mut rows = Self::setup(crate::error::reserved(count))?;
        let mut statuses = Self::setup(crate::error::reserved(count))?;
        let results = Self::setup(self.scheduler.map_mut(&mut self.sessions, jobs, |_, session, job| match job {
            StepJob::Skip => Done::<S>::Skipped,
            StepJob::Autoreset { entry, inject } => match reset_slot(session, None, entry.as_deref(), BatchFaultStage::Autoreset, inject) {
                Ok((boundary, finished)) => Done::Autoreset { boundary, finished },
                Err((fault, finished)) => Done::Fault(fault, finished),
            },
            StepJob::Step { token, index, entry, inject } => {
                let before = session.batch_publication_serial();
                #[cfg(test)]
                if inject == Some(TestInject::StepError) {
                    let error = session.batch_retain_error(Error::DeliveryFault("TEST-ONLY injected step fault"), false);
                    return Done::Fault(JobFault { stage: BatchFaultStage::Step, error, committed: false }, None)
                }
                let stepped = catch_unwind(AssertUnwindSafe(|| {
                    #[cfg(test)]
                    if inject == Some(TestInject::StepPanic) { panic!("TEST-ONLY injected step panic") }
                    let result = session.batch_step(token, index);
                    #[cfg(test)]
                    if inject == Some(TestInject::StepPanicAfterCommit) { panic!("TEST-ONLY injected panic after commit") }
                    #[cfg(test)]
                    if inject == Some(TestInject::StepErrorAfterCommit) && result.is_ok() {
                        return Err(BatchStepFailure { error: Error::DeliveryFault("TEST-ONLY injected post-commit fault"), committed: true })
                    }
                    result
                }));
                let reset_inject = inject.filter(|i| is_reset_inject(*i));
                match stepped {
                    Ok(Ok(step)) if same_step && step.terminated => match reset_slot(session, None, entry.as_deref(), BatchFaultStage::Autoreset, reset_inject) {
                        Ok((reset, finished)) => Done::SameStep { step, reset, finished },
                        // The composite is stage `autoreset` even when the reset panicked (the panic stays in the error text).
                        Err((mut fault, finished)) => { fault.committed = true; fault.stage = BatchFaultStage::Autoreset; Done::SameStepFault { step, finished, fault } }
                    },
                    Ok(Ok(step)) => Done::Stepped(step),
                    Ok(Err(BatchStepFailure { error, committed })) => {
                        let error = session.batch_retain_error(error, committed);
                        Done::Fault(JobFault { stage: stage(BatchFaultStage::Step, committed, &error), error, committed }, None)
                    }
                    Err(_) => {
                        let committed = session.batch_publication_serial() != before;
                        let error = session.batch_retain_panic(committed);
                        Done::Fault(JobFault { stage: BatchFaultStage::Panic, error, committed }, None)
                    }
                }
            }
        }))?;
        #[cfg(test)]
        if post_join_panic { panic!("TEST-ONLY injected panic after the join") }
        let next_step = self.config.autoreset == AutoresetMode::NextStep;
        let isolate = self.config.fault_policy == FaultPolicy::Isolate;
        let mut faulted = false;
        for (slot, result) in results.into_iter().enumerate() {
            let retained = |step: &BatchStep<S::Boundary>, final_boundary: Option<S::Boundary>, final_is_current: bool| Retained {
                reward: step.reward, reward_events: step.reward_events.clone(), terminated: step.terminated, truncated: step.truncated,
                autoreset: false, final_boundary, final_is_current };
            match result {
                Done::Skipped => { statuses.push(UNAFFECTED); self.push_fault_row(&mut rows, slot); }
                Done::Stepped(step) => {
                    let state = &mut self.slots[slot];
                    state.pending_autoreset = next_step && step.terminated;
                    state.retained = Some(retained(&step, None, false));
                    statuses.push(COMMITTED);
                    rows.push(SlotRow { boundary: step.boundary, reward: step.reward, reward_events: step.reward_events, terminated: step.terminated,
                        truncated: step.truncated, autoreset: false, final_boundary: None, fault: None });
                }
                Done::SameStep { step, reset, finished } => {
                    if let Some(record) = finished { self.completed.push(CompletedRecord { slot, record }); }
                    self.slots[slot].pending_autoreset = false;
                    self.slots[slot].retained = Some(retained(&step, Some(step.boundary.clone()), false));
                    statuses.push(COMMITTED);
                    rows.push(SlotRow { boundary: reset, reward: step.reward, reward_events: step.reward_events, terminated: true,
                        truncated: step.truncated, autoreset: false, final_boundary: Some(step.boundary), fault: None });
                }
                Done::SameStepFault { step, finished, fault } => {
                    // Design §3.3: the terminal transition is delivered (isolate) or recoverable (freeze); nothing is retried.
                    self.complete_before_fault(slot, finished);
                    faulted = true;
                    if isolate && self.template.is_some() {
                        let row = fault_row(&fault);
                        self.isolate(slot, &fault);
                        // Kept until the slot's resume, so an export failure of this row can still recover the terminal.
                        self.slots[slot].retained = Some(retained(&step, Some(step.boundary.clone()), true));
                        let mut out = SlotRow::plain(self.template.clone().unwrap());
                        out.reward = step.reward; out.reward_events = step.reward_events; out.terminated = true; out.truncated = step.truncated;
                        out.final_boundary = Some(step.boundary); out.fault = Some(row);
                        rows.push(out);
                    } else {
                        self.slots[slot].pending_autoreset = false;
                        self.slots[slot].retained = Some(retained(&step, Some(step.boundary.clone()), true));
                        self.push_fault_row(&mut rows, slot);
                    }
                    statuses.push(faulted_status(fault));
                }
                Done::Autoreset { boundary, finished } => {
                    if let Some(record) = finished { self.completed.push(CompletedRecord { slot, record }); }
                    self.slots[slot] = SlotState { pending_autoreset: false, isolated: None, retained: Some(Retained {
                        reward: 0, reward_events: Vec::new(), terminated: false, truncated: false, autoreset: true, final_boundary: None, final_is_current: false }), record_completed: false };
                    statuses.push(COMMITTED);
                    let mut row = SlotRow::plain(boundary);
                    row.autoreset = true;
                    rows.push(row);
                }
                Done::Fault(fault, finished) => {
                    self.complete_before_fault(slot, finished);
                    faulted = true;
                    self.isolate(slot, &fault);
                    statuses.push(faulted_status(fault)); self.push_fault_row(&mut rows, slot);
                }
                _ => unreachable!("step jobs produce step outcomes"),
            }
        }
        self.finish(BatchOperation::Step, rows, statuses, faulted)
    }

    /// An isolated slot's fault row (pushed only when it can be delivered: `Isolate` with a published layout).
    fn push_fault_row(&self, rows: &mut Vec<SlotRow<S::Boundary>>, slot: usize) {
        if self.config.fault_policy != FaultPolicy::Isolate { return }
        if let Some(template) = &self.template {
            let mut row = SlotRow::plain(template.clone());
            row.fault = self.slots[slot].isolated.clone();
            rows.push(row);
        }
    }
    /// Common tail: freeze (policy `Freeze`, or no published layout for a fault row) or return the rows.
    fn finish(&mut self, operation: BatchOperation, rows: Vec<SlotRow<S::Boundary>>, statuses: Vec<SlotReport>, faulted: bool)
        -> Result<Vec<SlotRow<S::Boundary>>, SessionBatchError> {
        let isolate = self.config.fault_policy == FaultPolicy::Isolate;
        if faulted && (!isolate || self.template.is_none()) {
            // Freeze: faulted sessions stay frozen; isolation does not apply. Committed peers stay readable.
            for (slot, status) in statuses.iter().enumerate() {
                if matches!(status.delivery, SlotDelivery::Faulted { .. }) { self.slots[slot].isolated = None; }
            }
            return Err(self.freeze(operation, statuses))
        }
        debug_assert_eq!(rows.len(), statuses.len());
        let out = rows;
        self.last = statuses.into_iter().map(|s| match s.delivery { SlotDelivery::CommittedUndelivered { .. } => SlotDelivery::Delivered, other => other }).collect();
        Ok(out)
    }

    /// The transport could not deliver the last returned rows (design §3.3, export failures): the batch freezes under
    /// either policy, and every slot that committed in that operation is `committed_undelivered` and recoverable.
    pub fn fail_export(&mut self) -> BatchFault {
        if let Some(fault) = &self.frozen { return fault.clone() }
        let slots = self.last.iter().enumerate().map(|(slot, delivery)| match delivery {
            SlotDelivery::Delivered => SlotReport { delivery: SlotDelivery::CommittedUndelivered { recoverable: self.slots[slot].retained.is_some() }, error: None },
            SlotDelivery::Faulted { .. } => SlotReport { delivery: *delivery, error: self.sessions[slot].batch_fault() },
            other => SlotReport { delivery: *other, error: None },
        }).collect();
        let SessionBatchError::Fault(fault) = self.freeze(BatchOperation::Export, slots) else { unreachable!() };
        fault
    }

    /// The slot's last committed result (design §3.3): read-only and repeatable; never re-executes anything. The row's
    /// boundary is the session's published one (or the retained terminal of a SAME_STEP autoreset fault).
    pub fn committed_result(&self, slot: usize) -> Result<SlotRow<S::Boundary>, SessionBatchError> {
        let state = self.slots.get(slot).ok_or_else(|| Self::input(Some(slot), Error::Input("batch slot index out of range")))?;
        let r = state.retained.as_ref().ok_or_else(|| SessionBatchError::Diagnostic { slot, error: Error::Input("slot has no retained committed result") })?;
        let boundary = match (&r.final_boundary, r.final_is_current) {
            (Some(b), true) => b.clone(),
            _ => self.sessions[slot].batch_decision().map_err(|error| SessionBatchError::Diagnostic { slot, error })?,
        };
        Ok(SlotRow { boundary, reward: r.reward, reward_events: r.reward_events.clone(), terminated: r.terminated, truncated: r.truncated,
            autoreset: r.autoreset, final_boundary: r.final_boundary.clone(), fault: None })
    }
    /// Every recoverable slot's committed result after a failed operation (`None` for the others).
    pub fn recover_committed(&self) -> Result<Vec<Option<SlotRow<S::Boundary>>>, SessionBatchError> {
        let Some(fault) = &self.frozen else { return Err(Self::input(None, Error::Input("no failed batch operation to recover"))) };
        fault.slots.iter().enumerate().map(|(slot, status)| match status.delivery {
            SlotDelivery::CommittedUndelivered { recoverable: true } | SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: true, .. }
                if self.slots[slot].retained.is_some() => self.committed_result(slot).map(Some),
            _ => Ok(None),
        }).collect()
    }
    pub fn decision(&self, slot: usize) -> Result<S::Boundary, SessionBatchError> {
        self.sessions.get(slot).ok_or_else(|| Self::input(Some(slot), Error::Input("batch slot index out of range")))?
            .batch_decision().map_err(|error| SessionBatchError::Diagnostic { slot, error })
    }
    pub fn record(&self, slot: usize) -> Result<Option<S::Record>, SessionBatchError> {
        self.sessions.get(slot).ok_or_else(|| Self::input(Some(slot), Error::Input("batch slot index out of range")))?
            .batch_record().map_err(|error| SessionBatchError::Diagnostic { slot, error })
    }
    /// Finished episode records, in the order their slots were reset (slot order within one operation).
    pub fn take_completed_records(&mut self) -> Vec<CompletedRecord<S::Record>> { std::mem::take(&mut self.completed) }
    /// Faults and records of sessions replaced by an isolate resume.
    pub fn take_fault_records(&mut self) -> Vec<RetiredFault<S::Record>> { std::mem::take(&mut self.fault_records) }
    pub fn diagnostics(&self) -> Value {
        json!({
            "status": if self.closed { "closed" } else if self.frozen.is_some() { "faulted" } else { "active" },
            "policy": self.config.fault_policy.label(), "autoreset": self.config.autoreset.label(), "workers": self.config.workers,
            "completed_records": self.completed.len(), "fault_records": self.fault_records.len(),
            "slots": self.last.iter().map(|d| d.label()).collect::<Vec<_>>(),
            "isolated": self.slots.iter().map(|s| s.isolated.is_some()).collect::<Vec<_>>(),
            "pending_autoreset": self.slots.iter().map(|s| s.pending_autoreset).collect::<Vec<_>>(),
        })
    }
    /// Close every session (allowed in a frozen batch; idempotent). Records stay readable and are marked closed.
    pub fn close(&mut self) -> Result<(), SessionBatchError> {
        let mut first = None;
        for (slot, session) in self.sessions.iter_mut().enumerate() {
            if let Err(error) = session.batch_close() { first.get_or_insert(SessionBatchError::Diagnostic { slot, error }); }
        }
        self.closed = true;
        first.map_or(Ok(()), Err)
    }
}

/// Gym66 v1/v2, M21 and `standard-combat-v1` batches (design §3.4.1 `NativeCombatTrainingBatch`).
pub type CombatTrainingBatch = SessionBatch<CombatTrainingSession>;
impl SessionBatch<CombatTrainingSession> {
    /// `num_envs` sessions over one shared catalog (Standard catalogs build `standard-combat-v1` sessions).
    pub fn new_combat(catalog: std::sync::Arc<CombatTrainingCatalog>, partition: spire_core::combat::rng::Partition, num_envs: usize,
        default_entries: Option<Vec<Option<String>>>, config: BatchConfig) -> Result<Self, SessionBatchError> {
        if !(1..=BATCH_MAX_SLOTS).contains(&num_envs) { return Err(Self::input(None, Error::Input("batch size must be 1..=256 and workers must be positive"))) }
        let mut sessions = Self::setup(crate::error::reserved(num_envs))?;
        for slot in 0..num_envs {
            let session = if catalog.is_standard() { CombatTrainingSession::new_standard_shared(catalog.clone(), partition) } else { CombatTrainingSession::new_shared(catalog.clone(), partition) };
            sessions.push(session.map_err(|e| Self::input(Some(slot), e))?);
        }
        Self::from_sessions(sessions, default_entries.unwrap_or_else(|| vec![None; num_envs]), config)
    }
}
/// Run batches over every profile, including M9, M10 policy-v3 and Standard (design §3.1).
pub type RunSessionBatch = SessionBatch<RunSession>;
impl SessionBatch<RunSession> {
    /// One session per config from `constructor` (e.g. `RunSession::new_standard`'s config form or `RunSession::new_m10`).
    pub fn new_run<C>(configs: Vec<C>, constructor: impl Fn(C) -> Result<RunSession, Error>, config: BatchConfig) -> Result<Self, SessionBatchError> {
        if !(1..=BATCH_MAX_SLOTS).contains(&configs.len()) || config.workers == 0 {
            return Err(Self::input(None, Error::Input("batch size must be 1..=256 and workers must be positive")))
        }
        let count = configs.len();
        let mut sessions = Self::setup(crate::error::reserved(count))?;
        for (slot, c) in configs.into_iter().enumerate() { sessions.push(constructor(c).map_err(|e| Self::input(Some(slot), e))?); }
        Self::from_sessions(sessions, vec![None; count], config)
    }
    /// Every slot's combat view (design §3.2 `combat_views()`): exactly the scalar `RunSession::combat_view()` of each
    /// slot (Standard and policy-v3 profiles; others are `UnsupportedBoundary`), and `in_combat` from the view's
    /// `active` flag. Isolated and faulted (frozen-session) slots are `None`. A read: no draw and no token change.
    pub fn combat_views(&mut self) -> Result<(Vec<Option<CombatPolicyBoundary>>, Vec<bool>), SessionBatchError> {
        if self.closed { return Err(Self::input(None, Error::Closed)) }
        if !self.started { return Err(Self::input(None, Error::NotReset)) }
        let count = self.sessions.len();
        self.sessions[0].combat_observation_spec().map_err(|e| Self::input(None, e))?;
        let mut jobs = Self::setup(crate::error::reserved(count))?;
        jobs.extend(self.slots.iter().zip(&self.sessions).map(|(s, session)| s.isolated.is_none() && session.batch_fault().is_none()));
        let views = Self::setup(self.scheduler.map_mut(&mut self.sessions, jobs, |_, session, live| live.then(|| session.combat_view())))?;
        let mut out = Self::setup(crate::error::reserved(count))?;
        let mut in_combat = Self::setup(crate::error::reserved(count))?;
        for (slot, view) in views.into_iter().enumerate() {
            match view.transpose().map_err(|error| SessionBatchError::Diagnostic { slot, error })? {
                Some(view) => { in_combat.push(view.info["active"] == json!(true)); out.push(Some(view)); }
                None => { in_combat.push(false); out.push(None); }
            }
        }
        Ok((out, in_combat))
    }
}
