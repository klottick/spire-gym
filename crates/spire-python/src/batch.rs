//! M35 batch transport (design §3.4.1): `NativeRunBatch` and `NativeCombatTrainingBatch` over the one native owner,
//! `spire_env::SessionBatch`. The binding parses inputs, releases the GIL around every native operation and stacks rows
//! into owned `[B, ...]` arrays; it holds no batching, seed or reward logic. A failed export freezes the batch, and
//! `recover_committed()` repackages the retained results from native memory (nothing is re-executed).
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

use pyo3::{create_exception, prelude::*, types::{PyAny, PyBytes, PyDict, PyInt, PyList, PyTuple}};
use rust_numpy::{PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use spire_core::run::{RewardItem, RunTerminal};
use spire_env::{
    AutoresetMode, BatchConfig, BatchFault, BatchFaultStage, BatchOperation, BatchSession, BranchStepOutcome,
    CombatPolicyBoundary, CombatTrainingCatalog, CombatTrainingSession, CompletedRecord, FaultPolicy, InputReason,
    RetiredFault, RunActionIndex, RunBoundary, RunDecisionToken, RunObservation, RunSession, RunSessionError, SessionBatch,
    SessionBatchError, SlotDelivery, SlotReport, SlotRow,
};

use crate::{
    errors, numeric,
    run_env::{self, RunFaultError, RunInputError, RunProfile, RunStateError, boolean, integer, label},
    stacking::{self, Column},
};

create_exception!(_native, BatchFaultError, RunFaultError);

/// The session-specific parts of packaging (Run boundaries carry a terminal; combat boundaries carry their info JSON).
pub(crate) trait Transport: BatchSession {
    const COMBAT: bool;
    /// M38b: whether steps encode straight into the stacked columns (Standard Run batches).
    fn slab(profile: Option<RunProfile>) -> bool;
    fn observation(b: &Self::Boundary) -> &RunObservation;
    /// Move the boundary's observation out for stacking (M37b), leaving an empty one.
    fn take_observation(b: &mut Self::Boundary) -> RunObservation;
    fn token(b: &Self::Boundary) -> RunDecisionToken;
    fn record_json(r: &Self::Record) -> Result<Vec<u8>, RunSessionError>;
    /// A published boundary's info: `events` and `episode_return` belong to the transition that produced it.
    fn info<'py>(py: Python<'py>, profile: Option<RunProfile>, b: &Self::Boundary, events: &[RewardItem], episode_return: i64)
        -> PyResult<Bound<'py, PyAny>>;
}
impl Transport for RunSession {
    const COMBAT: bool = false;
    fn slab(profile: Option<RunProfile>) -> bool { profile == Some(RunProfile::Standard) }
    fn observation(b: &RunBoundary) -> &RunObservation { &b.observation }
    fn take_observation(b: &mut RunBoundary) -> RunObservation { std::mem::replace(&mut b.observation, empty_observation()) }
    fn token(b: &RunBoundary) -> RunDecisionToken { b.token }
    fn record_json(r: &spire_env::RunEpisodeRecord) -> Result<Vec<u8>, RunSessionError> { r.to_json() }
    fn info<'py>(py: Python<'py>, profile: Option<RunProfile>, b: &RunBoundary, events: &[RewardItem], episode_return: i64)
        -> PyResult<Bound<'py, PyAny>> {
        let profile = profile.expect("Run batches carry their profile");
        let info = PyDict::new(py);
        info.set_item("schema_id", profile.schema_id())?;
        info.set_item("terminal_reason", run_env::terminal(b.terminal))?;
        if profile == RunProfile::Standard {
            info.set_item("reward_events", run_env::reward_events(py, events)?)?;
            info.set_item("episode_return", episode_return)?;
            info.set_item("heart_victory", b.terminal == Some(RunTerminal::HeartVictory))?;
        }
        Ok(info.into_any())
    }
}
impl Transport for CombatTrainingSession {
    const COMBAT: bool = true;
    fn slab(_: Option<RunProfile>) -> bool { false }
    fn observation(b: &CombatPolicyBoundary) -> &RunObservation { &b.observation }
    fn take_observation(b: &mut CombatPolicyBoundary) -> RunObservation { std::mem::replace(&mut b.observation, empty_observation()) }
    fn token(b: &CombatPolicyBoundary) -> RunDecisionToken { b.token }
    fn record_json(r: &spire_env::CombatEpisodeRecord) -> Result<Vec<u8>, RunSessionError> { r.to_json() }
    fn info<'py>(py: Python<'py>, _: Option<RunProfile>, b: &CombatPolicyBoundary, _: &[RewardItem], _: i64) -> PyResult<Bound<'py, PyAny>> {
        numeric::descriptor(py, b.info.to_string())
    }
}

fn empty_observation() -> RunObservation { RunObservation { groups: Vec::new(), action_mask: Vec::new() } }
/// The stacked `obs` and `final_obs` columns of one packaged result (M37b: built outside the GIL on the batch's workers).
/// `pooled`: the `obs` columns come from the batch's column pool and return to it (M38b).
struct Columns { obs: Vec<stacking::Stacked>, final_obs: Option<Vec<stacking::Stacked>>, pooled: bool }
fn stacked<'py>(py: Python<'py>, columns: Vec<stacking::Stacked>, pool: Option<&Arc<stacking::ColumnPool>>) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    for tensor in columns {
        match (tensor.column, pool) {
            (Column::I64(v), Some(pool)) => out.set_item(tensor.key, numeric::pooled_array(py, v, &tensor.shape, pool, Column::I64)?)?,
            (Column::I8(v), Some(pool)) => out.set_item(tensor.key, numeric::pooled_array(py, v, &tensor.shape, pool, Column::I8)?)?,
            (Column::I64(v), None) => out.set_item(tensor.key, numeric::array(py, v, &tensor.shape)?)?,
            (Column::I8(v), None) => out.set_item(tensor.key, numeric::array(py, v, &tensor.shape)?)?,
        }
    }
    Ok(out)
}
fn stacked_rows<'py>(py: Python<'py>, rows: &[&RunObservation]) -> PyResult<Bound<'py, PyDict>> {
    stacked(py, stacking::stack(rows).map_err(RunFaultError::new_err)?, None)
}
fn tokens<'py>(py: Python<'py>, tokens: impl Iterator<Item = RunDecisionToken>, count: usize) -> PyResult<Bound<'py, PyAny>> {
    numeric::array(py, tokens.flat_map(<[u64; 2]>::from).collect(), &[count, 2])
}
fn fault_dict<'py>(py: Python<'py>, stage: &str, message: &str, committed: bool) -> PyResult<Bound<'py, PyAny>> {
    let d = PyDict::new(py);
    d.set_item("stage", stage)?; d.set_item("message", message)?; d.set_item("committed", committed)?;
    Ok(d.into_any())
}
fn status_dict<'py>(py: Python<'py>, s: &stacking::SlotStatus) -> PyResult<Bound<'py, PyAny>> {
    let d = PyDict::new(py);
    d.set_item("status", s.status)?; d.set_item("stage", s.stage)?; d.set_item("committed", s.committed)?;
    d.set_item("recoverable", s.recoverable)?; d.set_item("message", s.message.as_deref())?;
    Ok(d.into_any())
}

/// `BatchFaultError` with `.operation`, `.policy` and one status dict per slot (design §3.3).
fn batch_fault(py: Python<'_>, fault: &BatchFault, combat: bool) -> PyErr {
    let prefix = if combat { "native combat fault" } else { "native Run fault" };
    let error = BatchFaultError::new_err(format!("{prefix}: batch {} failed", fault.operation.label()));
    let attach = || -> PyResult<()> {
        let value = error.value(py);
        value.setattr("operation", fault.operation.label())?;
        value.setattr("policy", fault.policy.label())?;
        let slots = fault.slots.iter().map(|r| status_dict(py, &stacking::slot_status(r, combat))).collect::<PyResult<Vec<_>>>()?;
        value.setattr("slots", PyTuple::new(py, slots)?)?;
        Ok(())
    };
    match attach() { Ok(()) => error, Err(e) => e }
}
/// The scalar error class and message (unchanged, so pre-M35 consumer logs compare equal), with `.slot`. Only the isolate
/// resume rejection names its slot in the message (PM ruling: "faulted slot needs an explicit seed (slot k)").
fn slot_error(py: Python<'_>, error: RunSessionError, slot: Option<usize>) -> PyErr {
    let names_slot = matches!(error, RunSessionError::Input("faulted slot needs an explicit seed"));
    let base = run_env::error(error);
    let error = match slot.filter(|_| names_slot) {
        Some(slot) => {
            let message = base.value(py).str().map(|s| s.to_string()).unwrap_or_default();
            PyErr::from_type(base.get_type(py), format!("{message} (slot {slot})"))
        }
        None => base,
    };
    match error.value(py).setattr("slot", slot) { Ok(()) => error, Err(e) => e }
}
fn batch_error(py: Python<'_>, error: SessionBatchError, combat: bool) -> PyErr {
    match error {
        SessionBatchError::Input { slot, error } => slot_error(py, error, slot),
        SessionBatchError::Diagnostic { slot, error } => slot_error(py, error, Some(slot)),
        SessionBatchError::Setup(error) => errors::native(py, error),
        SessionBatchError::Fault(fault) => batch_fault(py, &fault, combat),
    }
}

fn autoreset_mode(value: &str) -> PyResult<AutoresetMode> {
    match value {
        "disabled" => Ok(AutoresetMode::Disabled), "next_step" => Ok(AutoresetMode::NextStep), "same_step" => Ok(AutoresetMode::SameStep),
        _ => Err(RunInputError::new_err("autoreset must be 'disabled', 'next_step' or 'same_step'")),
    }
}
fn fault_policy(value: &str) -> PyResult<FaultPolicy> {
    match value {
        "freeze" => Ok(FaultPolicy::Freeze), "isolate" => Ok(FaultPolicy::Isolate),
        _ => Err(RunInputError::new_err("fault_policy must be 'freeze' or 'isolate'")),
    }
}
fn batch_config(workers: &Bound<'_, PyAny>, autoreset: &str, policy: &str) -> PyResult<BatchConfig> {
    Ok(BatchConfig { workers: integer(workers)?, autoreset: autoreset_mode(autoreset)?, fault_policy: fault_policy(policy)? })
}

/// Input errors keep each surface's existing classes (combat: `CombatInputError`; Run: `RunInputError`).
fn bad(py: Python<'_>, combat: bool, reason: InputReason, message: &'static str) -> PyErr {
    if combat { errors::input(py, reason, message, None) } else { RunInputError::new_err(message) }
}
fn option_list<T: Clone>(value: Option<&Bound<'_, PyAny>>, count: usize, combat: bool, parse: impl Fn(&Bound<'_, PyAny>) -> PyResult<T>)
    -> PyResult<Vec<Option<T>>> {
    let Some(value) = value.filter(|v| !v.is_none()) else { return Ok(vec![None; count]) };
    let py = value.py();
    let list = value.cast::<PyList>().map_err(|_| bad(py, combat, InputReason::Shape, "batch reset values must be a length-B list"))?;
    if list.len() != count { return Err(bad(py, combat, InputReason::Shape, "batch reset values must have length B")) }
    list.iter().map(|item| if item.is_none() { Ok(None) } else { parse(&item).map(Some) }).collect()
}
/// `None`, one `int` (expanded natively to `seed + slot`, the shared combat-v1 derivation), or a length-B list.
fn seeds(value: Option<&Bound<'_, PyAny>>, count: usize, combat: bool) -> PyResult<Vec<Option<u64>>> {
    match value.filter(|v| !v.is_none()) {
        Some(v) if v.is_exact_instance_of::<PyInt>() => {
            let seed = integer::<u64>(v)?;
            let expanded = spire_env::expand_scalar_seed(seed, count)
                .map_err(|e| if combat { errors::native(v.py(), e) } else { RunInputError::new_err("scalar seed expansion exceeds u64") })?;
            Ok(expanded.into_iter().map(Some).collect())
        }
        other => option_list(other, count, combat, integer::<u64>),
    }
}
fn reset_mask(value: Option<&Bound<'_, PyAny>>, count: usize, combat: bool) -> PyResult<Vec<bool>> {
    let Some(value) = value.filter(|v| !v.is_none()) else { return Ok(vec![true; count]) };
    let py = value.py();
    let array = value.cast::<PyArrayDyn<bool>>().map_err(|_| bad(py, combat, InputReason::Dtype, "reset_mask must be a native bool NumPy array"))?;
    if array.shape() != [count] { return Err(bad(py, combat, InputReason::Shape, "reset_mask must have shape [B]")) }
    let guard = array.try_readonly().map_err(|_| bad(py, combat, InputReason::Dtype, "reset_mask is already mutably borrowed"))?;
    Ok(guard.as_array().iter().copied().collect())
}

/// Per-slot episode returns (Run Standard info): updated from rows the native owner committed, never recomputed.
#[derive(Clone, Default)]
struct Returns { running: Vec<i64>, last_final: Vec<i64> }
impl Returns {
    fn new(count: usize) -> Self { Self { running: vec![0; count], last_final: vec![0; count] } }
    fn apply<B>(&mut self, slot: usize, row: &SlotRow<B>, reset: bool) {
        if reset || row.autoreset { self.running[slot] = 0; return }
        if row.final_boundary.is_some() {
            self.last_final[slot] = self.running[slot] + i64::from(row.reward);
            self.running[slot] = 0;
        } else if row.fault.is_none() {
            self.running[slot] += i64::from(row.reward);
        }
    }
}

struct State<S: BatchSession> {
    batch: SessionBatch<S>,
    returns: Returns,
    /// Drained but not yet delivered evidence (kept until a successful export).
    completed: Vec<CompletedRecord<S::Record>>,
    retired: Vec<RetiredFault<S::Record>>,
}
impl<S: BatchSession> State<S> {
    /// After a failed operation, account the committed results the caller can recover.
    fn note_recoverable(&mut self, fault: &BatchFault) {
        for (slot, report) in fault.slots.iter().enumerate() {
            let recoverable = matches!(report.delivery, SlotDelivery::CommittedUndelivered { recoverable: true }
                | SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: true, .. });
            if !recoverable { continue }
            if let Ok(row) = self.batch.committed_result(slot) { self.returns.apply(slot, &row, fault.operation == BatchOperation::Reset); }
        }
    }
}

/// Lock order (final review c1 finding 3): every method takes the non-blocking lease before the mutex, so no thread ever
/// waits on the mutex while holding the GIL, and the mutex is never held across Python code (packaging works on copies).
struct Core<S: BatchSession> {
    state: Mutex<State<S>>, busy: AtomicBool, num_envs: usize, profile: Option<RunProfile>, pool: Arc<stacking::ColumnPool>,
}
struct Lease<'a>(&'a AtomicBool);
impl Drop for Lease<'_> { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }

impl<S: Transport> Core<S> {
    fn new(batch: SessionBatch<S>, profile: Option<RunProfile>) -> Self {
        let num_envs = batch.len();
        Self { state: Mutex::new(State { batch, returns: Returns::new(num_envs), completed: Vec::new(), retired: Vec::new() }),
            busy: AtomicBool::new(false), num_envs, profile, pool: Arc::default() }
    }
    fn begin(&self) -> PyResult<Lease<'_>> {
        self.busy.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| RunStateError::new_err("batch operation is busy"))?;
        Ok(Lease(&self.busy))
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, State<S>> { self.state.lock().unwrap_or_else(|p| p.into_inner()) }
    fn error(&self, py: Python<'_>, error: SessionBatchError) -> PyErr { batch_error(py, error, S::COMBAT) }

    /// The stacked observation columns of `rows` (run without the GIL). Each row's observations move into the copy jobs,
    /// which run on the batch's workers and drop them; `None` rows (recovery) and rows without a final boundary read the
    /// zero layout.
    fn columns(batch: &SessionBatch<S>, zero: Option<&S::Boundary>, rows: &mut [Option<SlotRow<S::Boundary>>]) -> PyResult<Columns> {
        let zeroed;
        let zero = match rows.iter().flatten().next() {
            Some(row) => { zeroed = S::batch_zeroed(&row.boundary).map_err(run_env::error)?; &zeroed }
            None => zero.ok_or_else(|| RunStateError::new_err("no published batch layout"))?,
        };
        let zero = S::observation(zero);
        let finals = rows.iter().any(|r| r.as_ref().is_some_and(|r| r.final_boundary.is_some()));
        let mut obs = Vec::with_capacity(rows.len());
        let mut final_obs = Vec::with_capacity(if finals { rows.len() } else { 0 });
        for row in rows.iter_mut() {
            match row {
                Some(r) => {
                    obs.push(stacking::SlotSource::Owned(S::take_observation(&mut r.boundary)));
                    if finals {
                        final_obs.push(match &mut r.final_boundary {
                            Some(b) => stacking::SlotSource::Owned(S::take_observation(b)),
                            None => stacking::SlotSource::Shared(zero),
                        });
                    }
                }
                None => {
                    obs.push(stacking::SlotSource::Shared(zero));
                    if finals { final_obs.push(stacking::SlotSource::Shared(zero)); }
                }
            }
        }
        let stack = |sources| stacking::stack_slots(sources, |jobs| batch.map_jobs(jobs, stacking::SlotCopy::run).map(|_| ()))
            .map_err(RunFaultError::new_err);
        let obs = stack(obs)?;
        let final_obs = if finals { Some(stack(final_obs)?) } else { None };
        Ok(Columns { obs, final_obs, pooled: false })
    }
    /// M38b: [`Self::columns`] for a step that encoded into `slab`. Rows encoded into their chunks (no fault, and an
    /// observation without groups) are already in place; every other row is copied as [`Self::columns`] copies it.
    /// `final_obs` is stacked exactly as there, with the batch's zero template for rows without a final boundary.
    fn slab_columns(batch: &SessionBatch<S>, slab: stacking::Slab<'_>, rows: &mut [Option<SlotRow<S::Boundary>>]) -> PyResult<Columns> {
        let zero = S::observation(batch.zero_boundary().ok_or_else(|| RunStateError::new_err("no published batch layout"))?);
        let finals = rows.iter().any(|r| r.as_ref().is_some_and(|r| r.final_boundary.is_some()));
        let mut fill = Vec::with_capacity(rows.len());
        let mut final_obs = Vec::with_capacity(if finals { rows.len() } else { 0 });
        for row in rows.iter_mut() {
            match row {
                Some(r) => {
                    let encoded = r.fault.is_none() && S::observation(&r.boundary).groups.is_empty();
                    fill.push(if encoded { None } else { Some(stacking::SlotSource::Owned(S::take_observation(&mut r.boundary))) });
                    if finals {
                        final_obs.push(match &mut r.final_boundary {
                            Some(b) => stacking::SlotSource::Owned(S::take_observation(b)),
                            None => stacking::SlotSource::Shared(zero),
                        });
                    }
                }
                None => {
                    fill.push(Some(stacking::SlotSource::Shared(zero)));
                    if finals { final_obs.push(stacking::SlotSource::Shared(zero)); }
                }
            }
        }
        let obs = slab.finish(fill, |jobs| batch.map_jobs(jobs, stacking::SlotCopy::run).map(|_| ())).map_err(RunFaultError::new_err)?;
        let final_obs = if finals {
            Some(stacking::stack_slots(final_obs, |jobs| batch.map_jobs(jobs, stacking::SlotCopy::run).map(|_| ())).map_err(RunFaultError::new_err)?)
        } else { None };
        Ok(Columns { obs, final_obs, pooled: true })
    }
    /// The raw batch dict (binding-api-v1). `None` rows (recovery) are zero rows with `recovered` false. `columns` holds
    /// the rows' stacked observations (`Core::columns`), which no longer sit in `rows`.
    fn package<'py>(&self, py: Python<'py>, returns: &Returns, columns: Columns, rows: &[Option<SlotRow<S::Boundary>>], recovered: bool)
        -> PyResult<Bound<'py, PyDict>> {
        let count = rows.len();
        let Columns { obs, final_obs, pooled } = columns;
        let out = PyDict::new(py);
        out.set_item("obs", stacked(py, obs, pooled.then_some(&self.pool))?)?;
        out.set_item("tokens", tokens(py, rows.iter().map(|r| r.as_ref().map_or(RunDecisionToken::from([0_u64, 0]), |r| S::token(&r.boundary))), count)?)?;
        let column = |f: &dyn Fn(&SlotRow<S::Boundary>) -> bool| rows.iter().map(|r| r.as_ref().is_some_and(f)).collect::<Vec<_>>();
        out.set_item("rewards", numeric::array(py, rows.iter().map(|r| r.as_ref().map_or(0, |r| r.reward)).collect(), &[count])?)?;
        out.set_item("terminated", numeric::array(py, column(&|r| r.terminated), &[count])?)?;
        out.set_item("truncated", numeric::array(py, column(&|r| r.truncated), &[count])?)?;
        out.set_item("autoreset", numeric::array(py, column(&|r| r.autoreset), &[count])?)?;
        out.set_item("slot_fault", numeric::array(py, column(&|r| r.fault.is_some()), &[count])?)?;
        let faults = PyList::empty(py);
        let infos = PyList::empty(py);
        let final_infos = PyList::empty(py);
        for (slot, row) in rows.iter().enumerate() {
            let Some(row) = row else { faults.append(py.None())?; infos.append(PyDict::new(py))?; final_infos.append(py.None())?; continue };
            match &row.fault {
                Some(f) => { faults.append(fault_dict(py, f.stage.label(), &f.message, f.committed)?)?; infos.append(PyDict::new(py))?; }
                None => {
                    faults.append(py.None())?;
                    infos.append(S::info(py, self.profile, &row.boundary, &row.reward_events, returns.running[slot])?)?;
                }
            }
            match &row.final_boundary {
                Some(b) => final_infos.append(S::info(py, self.profile, b, &row.reward_events, returns.last_final[slot])?)?,
                None => final_infos.append(py.None())?,
            }
        }
        out.set_item("fault", faults)?;
        out.set_item("info", infos)?;
        let final_mask = column(&|r| r.final_boundary.is_some());
        if let Some(final_obs) = final_obs {
            out.set_item("final_obs", stacked(py, final_obs, None)?)?;
        } else {
            out.set_item("final_obs", py.None())?;
        }
        out.set_item("final_mask", numeric::array(py, final_mask, &[count])?)?;
        out.set_item("final_info", final_infos)?;
        if recovered { out.set_item("recovered", numeric::array(py, rows.iter().map(Option::is_some).collect(), &[count])?)?; }
        Ok(out)
    }
    /// Package and deliver; a failed export freezes the batch (every committed slot recoverable) and the caller's own
    /// exception propagates unchanged.
    fn deliver<'py, T>(&self, py: Python<'py>, rows: Vec<SlotRow<S::Boundary>>, slab: Option<stacking::Slab<'_>>,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>) -> PyResult<T> {
        let mut rows = rows.into_iter().map(Some).collect::<Vec<_>>();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (returns, columns) = py.detach(|| {
                let state = self.lock();
                (state.returns.clone(), match slab {
                    Some(slab) => Self::slab_columns(&state.batch, slab, &mut rows),
                    None => Self::columns(&state.batch, None, &mut rows),
                })
            });
            let raw = self.package(py, &returns, columns?, &rows, false)?;
            package(raw)
        })).unwrap_or_else(|_| Err(RunFaultError::new_err("native batch packaging panicked")));
        if result.is_err() { self.lock().batch.fail_export(); }
        result
    }

    fn reset<'py, T>(&self, py: Python<'py>, seeds: Vec<Option<u64>>, entries: Vec<Option<String>>, mask: Vec<bool>,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>) -> PyResult<T> {
        let _lease = self.begin()?;
        let rows = py.detach(|| {
            let mut state = self.lock();
            let result = state.batch.reset(&seeds, &entries, Some(&mask));
            match &result {
                Ok(rows) => for (slot, row) in rows.iter().enumerate() { if mask[slot] && row.fault.is_none() { state.returns.apply(slot, row, true); } },
                Err(SessionBatchError::Fault(fault)) => { let fault = fault.clone(); state.note_recoverable(&fault); }
                Err(_) => {}
            }
            result
        }).map_err(|e| self.error(py, e))?;
        self.deliver(py, rows, None, package)
    }
    fn step<'py, T>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>, tokens: &Bound<'py, PyAny>,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>) -> PyResult<T> {
        let _lease = self.begin()?;
        let (actions, tokens) = numeric::step_inputs(actions, tokens, self.num_envs)
            .map_err(|e| if S::COMBAT { e } else { RunInputError::new_err(e.value(py).to_string()) })?;
        let tokens = tokens.into_iter().map(RunDecisionToken::from).collect::<Vec<_>>();
        let indices = actions.into_iter().map(RunActionIndex).collect::<Vec<_>>();
        let (rows, slab) = py.detach(|| {
            let mut state = self.lock();
            // M38b: Standard steps encode into columns allocated (or pooled) up front; without a layout yet, or if the
            // columns cannot be allocated, the step runs as before and stacking reports any allocation failure.
            let mut slab = match state.batch.zero_boundary() {
                Some(zero) if S::slab(self.profile) => stacking::Slab::new(S::observation(zero), self.num_envs, &self.pool).ok(),
                _ => None,
            };
            let result = match &mut slab {
                Some(slab) => state.batch.step_into(&tokens, &indices, slab.targets()),
                None => state.batch.step(&tokens, &indices),
            };
            match &result {
                Ok(rows) => for (slot, row) in rows.iter().enumerate() { state.returns.apply(slot, row, false); },
                Err(SessionBatchError::Fault(fault)) => { let fault = fault.clone(); state.note_recoverable(&fault); }
                Err(_) => {}
            }
            (result, slab)
        });
        let rows = rows.map_err(|e| self.error(py, e))?;
        self.deliver(py, rows, slab, package)
    }
    fn recover_committed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let _lease = self.begin()?;
        let (rows, returns, zero) = py.detach(|| {
            let state = self.lock();
            (state.batch.recover_committed(), state.returns.clone(), state.batch.zero_boundary().cloned())
        });
        let mut rows = rows.map_err(|e| self.error(py, e))?;
        let target = &mut rows;
        let columns = py.detach(move || Self::columns(&self.lock().batch, zero.as_ref(), target))?;
        self.package(py, &returns, columns, &rows, true)
    }
    fn committed_result<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        let _lease = self.begin()?;
        let slot = integer::<usize>(slot)?;
        let (row, running, last_final) = {
            let state = self.lock();
            (state.batch.committed_result(slot), state.returns.running.get(slot).copied(), state.returns.last_final.get(slot).copied())
        };
        let row = row.map_err(|e| self.error(py, e))?;
        let (running, last_final) = (running.unwrap_or(0), last_final.unwrap_or(0));
        let out = PyDict::new(py);
        out.set_item("obs", run_env::observation(py, S::observation(&row.boundary).clone())?)?;
        let token: [u64; 2] = S::token(&row.boundary).into();
        out.set_item("token", (token[0], token[1]))?;
        out.set_item("reward", row.reward)?; out.set_item("terminated", row.terminated)?; out.set_item("truncated", row.truncated)?;
        out.set_item("autoreset", row.autoreset)?;
        out.set_item("info", S::info(py, self.profile, &row.boundary, &row.reward_events, running)?)?;
        match &row.final_boundary {
            Some(b) => {
                out.set_item("final_obs", run_env::observation(py, S::observation(b).clone())?)?;
                out.set_item("final_info", S::info(py, self.profile, b, &row.reward_events, last_final)?)?;
            }
            None => { out.set_item("final_obs", py.None())?; out.set_item("final_info", py.None())?; }
        }
        Ok(out)
    }
    fn decision<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        let _lease = self.begin()?;
        let slot = integer::<usize>(slot)?;
        let (b, running) = { let state = self.lock(); (state.batch.decision(slot), state.returns.running.get(slot).copied().unwrap_or(0)) };
        let b = b.map_err(|e| self.error(py, e))?;
        let out = PyDict::new(py);
        out.set_item("obs", run_env::observation(py, S::observation(&b).clone())?)?;
        let token: [u64; 2] = S::token(&b).into();
        out.set_item("token", (token[0], token[1]))?;
        out.set_item("info", S::info(py, self.profile, &b, &[], running)?)?;
        Ok(out)
    }
    fn record<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let _lease = self.begin()?;
        let slot = integer::<usize>(slot)?;
        let record = { let state = self.lock(); state.batch.record(slot) };
        let record = record.map_err(|e| self.error(py, e))?;
        let bytes = record.as_ref().map(S::record_json).transpose().map_err(run_env::error)?;
        Ok(bytes.map_or_else(|| py.None().into_bound(py), |b| PyBytes::new(py, &b).into_any()))
    }
    fn take_completed_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let _lease = self.begin()?;
        let mut pending = {
            let mut state = self.lock();
            let drained = state.batch.take_completed_records();
            state.completed.extend(drained);
            std::mem::take(&mut state.completed)
        };
        let result = crate::export_guard::export_retained(&mut pending, |records| {
            let out = PyList::empty(py);
            for r in records { out.append((r.slot, PyBytes::new(py, &S::record_json(&r.record).map_err(run_env::error)?)))?; }
            Ok(out)
        });
        if !pending.is_empty() { self.lock().completed.splice(0..0, pending); }
        result
    }
    fn take_fault_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let _lease = self.begin()?;
        let mut pending = {
            let mut state = self.lock();
            let drained = state.batch.take_fault_records();
            state.retired.extend(drained);
            std::mem::take(&mut state.retired)
        };
        let result = crate::export_guard::export_retained(&mut pending, |records| {
            let out = PyList::empty(py);
            for r in records {
                let d = PyDict::new(py);
                d.set_item("slot", r.slot)?;
                d.set_item("fault", r.error.as_ref().map(|e| stacking::error_message(e, S::COMBAT)))?;
                match &r.record {
                    Some(record) => d.set_item("record", PyBytes::new(py, &S::record_json(record).map_err(run_env::error)?))?,
                    None => d.set_item("record", py.None())?,
                }
                out.append(d)?;
            }
            Ok(out)
        });
        if !pending.is_empty() { self.lock().retired.splice(0..0, pending); }
        result
    }
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _lease = self.begin()?;
        let text = { let state = self.lock(); state.batch.diagnostics().to_string() };
        numeric::descriptor(py, text)
    }
    /// A read of slot 0's session (specs) under the lease; the mutex is released before any Python packaging.
    fn read_first<T>(&self, read: impl FnOnce(&S) -> Result<T, RunSessionError>) -> PyResult<T> {
        let _lease = self.begin()?;
        let result = { let state = self.lock(); read(state.batch.session(0).expect("batches hold at least one slot")) };
        result.map_err(run_env::error)
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        let _lease = self.begin()?;
        let result = py.detach(|| self.lock().batch.close()).map_err(|e| self.error(py, e));
        self.pool.clear();
        result
    }
}

fn run_config(py: Python<'_>, profile: RunProfile, item: &Bound<'_, PyAny>, caps: &spire_env::RunCapacities) -> PyResult<spire_env::RunSessionConfig> {
    let d = item.cast::<PyDict>().map_err(|_| RunInputError::new_err("each batch config must be a dictionary"))?;
    for (key, _) in d.iter() {
        if !matches!(label(&key)?.as_str(), "character" | "ascension" | "partition" | "creation" | "boss_history" | "record_replay") {
            return Err(RunInputError::new_err("unknown batch config key"))
        }
    }
    let character = d.get_item("character")?.ok_or_else(|| RunInputError::new_err("batch config requires character"))?;
    let ascension = d.get_item("ascension")?.ok_or_else(|| RunInputError::new_err("batch config requires ascension"))?;
    let partition = d.get_item("partition")?.map(|v| label(&v)).transpose()?.unwrap_or_else(|| "train".to_owned());
    let record_replay = d.get_item("record_replay")?.map(|v| boolean(&v)).transpose()?.unwrap_or(false);
    let boss_history = d.get_item("boss_history")?;
    let creation = d.get_item("creation")?;
    run_env::session_config(py, profile, &character, &ascension, &partition, boss_history.as_ref(), creation.as_ref(), caps.clone(), record_replay)
}

/// Run batches for the Python profiles (`standard`, `m9-ironclad-act3`, `m10-ironclad-heart`, `m10-ironclad-policy`).
#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeRunBatch { core: Core<RunSession> }
#[pymethods]
impl NativeRunBatch {
    #[new]
    #[pyo3(signature=(configs, *, profile="standard".to_owned(), capacities=None, workers=None, autoreset="next_step".to_owned(), fault_policy="freeze".to_owned()))]
    fn new(py: Python<'_>, configs: &Bound<'_, PyAny>, #[pyo3(from_py_with = label)] profile: String, capacities: Option<&Bound<'_, PyAny>>,
        workers: Option<&Bound<'_, PyAny>>, #[pyo3(from_py_with = label)] autoreset: String, #[pyo3(from_py_with = label)] fault_policy: String) -> PyResult<Self> {
        let profile = RunProfile::parse(&profile)?;
        let config = batch_config(&run_env::one_worker(py, workers), &autoreset, &fault_policy)?;
        let caps = run_env::capacities(capacities, profile)?;
        let list = configs.cast::<PyList>().map_err(|_| RunInputError::new_err("configs must be a list of dictionaries"))?;
        let configs = list.iter().map(|item| run_config(py, profile, &item, &caps)).collect::<PyResult<Vec<_>>>()?;
        let batch = py.detach(|| spire_env::RunSessionBatch::new_run(configs, |c| profile.new_session(c), config)).map_err(|e| batch_error(py, e, false))?;
        Ok(Self { core: Core::new(batch, Some(profile)) })
    }
    #[getter]
    fn num_envs(&self) -> usize { self.core.num_envs }
    #[pyo3(signature=(seeds=None, reset_mask=None))]
    fn reset<'py>(&self, py: Python<'py>, seeds: Option<&Bound<'py, PyAny>>, reset_mask: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyDict>> {
        let n = self.core.num_envs;
        self.core.reset(py, self::seeds(seeds, n, false)?, vec![None; n], self::reset_mask(reset_mask, n, false)?, Ok)
    }
    #[pyo3(signature=(package, *, seeds=None, reset_mask=None))]
    fn _reset_deliver<'py>(&self, py: Python<'py>, package: &Bound<'py, PyAny>, seeds: Option<&Bound<'py, PyAny>>, reset_mask: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() { return Err(RunInputError::new_err("package must be callable")) }
        let n = self.core.num_envs;
        self.core.reset(py, self::seeds(seeds, n, false)?, vec![None; n], self::reset_mask(reset_mask, n, false)?, |raw| package.call1((raw,)))
    }
    fn step<'py>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>, tokens: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        self.core.step(py, actions, tokens, Ok)
    }
    fn _step_deliver<'py>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>, tokens: &Bound<'py, PyAny>, package: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() { return Err(RunInputError::new_err("package must be callable")) }
        self.core.step(py, actions, tokens, |raw| package.call1((raw,)))
    }
    fn decision<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> { self.core.decision(py, slot) }
    fn committed_result<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> { self.core.committed_result(py, slot) }
    fn recover_committed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> { self.core.recover_committed(py) }
    /// Every slot's combat view (design §3.2): the scalar `combat_view()` per slot and an `in_combat` mask.
    fn combat_views<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let _lease = self.core.begin()?;
        let (views, in_combat) = py.detach(|| self.core.lock().batch.combat_views()).map_err(|e| batch_error(py, e, false))?;
        let first = views.iter().flatten().next().ok_or_else(|| RunStateError::new_err("no live batch slot"))?;
        let zero = <CombatTrainingSession as BatchSession>::batch_zeroed(first).map_err(run_env::error)?;
        let count = views.len();
        let out = PyDict::new(py);
        out.set_item("obs", stacked_rows(py, &views.iter().map(|v| v.as_ref().map_or(&zero.observation, |v| &v.observation)).collect::<Vec<_>>())?)?;
        out.set_item("tokens", tokens(py, views.iter().map(|v| v.as_ref().map_or(RunDecisionToken::from([0_u64, 0]), |v| v.token)), count)?)?;
        let infos = PyList::empty(py);
        for v in &views { match v { Some(v) => infos.append(numeric::descriptor(py, v.info.to_string())?)?, None => infos.append(py.None())? } }
        out.set_item("info", infos)?;
        out.set_item("in_combat", numeric::array(py, in_combat, &[count])?)?;
        Ok(out)
    }
    fn record<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> { self.core.record(py, slot) }
    fn reproduction<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        let _lease = self.core.begin()?;
        let slot = integer::<usize>(slot)?;
        let r = { let state = self.core.lock(); state.batch.session(slot).map(RunSession::reproduction) };
        let r = r.ok_or_else(|| slot_error(py, RunSessionError::Input("batch slot index out of range"), Some(slot)))?.map_err(run_env::error)?;
        run_env::reproduction(py, r)
    }
    fn take_completed_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> { self.core.take_completed_records(py) }
    fn take_fault_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> { self.core.take_fault_records(py) }
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> { self.core.diagnostics(py) }
    fn observation_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        run_env::specification(py, self.core.read_first(|s| s.observation_spec())?)
    }
    fn environment_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        numeric::descriptor(py, self.core.read_first(|s| s.environment_spec())?.to_string())
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> { self.core.close(py) }
}

/// The pre-M35 Gym66 `first_fault.reason` constants (final review c1 finding 5: the diagnostic delta stays `slots` only).
fn gym66_reason(operation: &str, panicked: bool) -> String {
    if panicked { format!("native combat batch {operation} panicked") } else { "native combat batch runtime operation failed".to_owned() }
}

/// Gym66 v1/v2 (`kind="ironclad"`) and `standard-combat-v1` (`kind="standard"`) batches over one shared catalog.
#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeCombatTrainingBatch { core: Core<CombatTrainingSession>, catalog: Arc<CombatTrainingCatalog> }
#[pymethods]
impl NativeCombatTrainingBatch {
    #[new]
    #[pyo3(signature=(num_envs, catalog_json, *, kind, records=None, partition="train".to_owned(), entry_ids=None, workers=None, autoreset="disabled".to_owned(), fault_policy="freeze".to_owned()))]
    #[allow(clippy::too_many_arguments)]
    fn new(py: Python<'_>, num_envs: &Bound<'_, PyAny>, catalog_json: &Bound<'_, PyAny>, #[pyo3(from_py_with = label)] kind: String,
        records: Option<&Bound<'_, PyAny>>, #[pyo3(from_py_with = label)] partition: String, entry_ids: Option<&Bound<'_, PyAny>>,
        workers: Option<&Bound<'_, PyAny>>, #[pyo3(from_py_with = label)] autoreset: String, #[pyo3(from_py_with = label)] fault_policy: String) -> PyResult<Self> {
        let num_envs = integer::<usize>(num_envs)?;
        if !(1..=spire_env::BATCH_MAX_SLOTS).contains(&num_envs) { return Err(errors::input(py, InputReason::Range, "num_envs must be in 1..=256", None)) }
        let standard = match kind.as_str() {
            "ironclad" => false, "standard" => true,
            _ => return Err(RunInputError::new_err("kind must be 'ironclad' (Gym66) or 'standard' (standard-combat-v1)")),
        };
        let records = crate::combat_training::records_bytes(records)?;
        if !standard && !records.is_empty() {
            return Err(RunInputError::new_err("kind 'ironclad' (Gym66) takes no records; records belong to kind 'standard' (standard-combat-v1)"))
        }
        let partition = crate::combat_training::parse_partition(py, &partition)?;
        let defaults = entry_ids.filter(|v| !v.is_none()).map(|v| option_list(Some(v), num_envs, true, label)).transpose()?;
        let config = batch_config(&run_env::one_worker(py, workers), &autoreset, &fault_policy)?;
        let bytes = crate::combat_training::catalog_bytes(catalog_json)?;
        let catalog = py.detach(|| if standard { CombatTrainingCatalog::standard_from_json_with_records(&bytes, &records) } else { CombatTrainingCatalog::from_json(&bytes) })
            .map_err(run_env::error)?;
        let catalog = Arc::new(catalog);
        let batch = py.detach(|| spire_env::CombatTrainingBatch::new_combat(catalog.clone(), partition, num_envs, defaults, config))
            .map_err(|e| batch_error(py, e, true))?;
        Ok(Self { core: Core::new(batch, None), catalog })
    }
    #[getter]
    fn num_envs(&self) -> usize { self.core.num_envs }
    fn catalog_descriptor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> { numeric::descriptor(py, self.catalog.descriptor().to_string()) }
    #[pyo3(signature=(seeds=None, entry_ids=None, reset_mask=None))]
    fn reset<'py>(&self, py: Python<'py>, seeds: Option<&Bound<'py, PyAny>>, entry_ids: Option<&Bound<'py, PyAny>>, reset_mask: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyDict>> {
        let n = self.core.num_envs;
        self.core.reset(py, self::seeds(seeds, n, true)?, option_list(entry_ids, n, true, label)?, self::reset_mask(reset_mask, n, true)?, Ok)
    }
    #[pyo3(signature=(package, *, seeds=None, entry_ids=None, reset_mask=None))]
    fn _reset_deliver<'py>(&self, py: Python<'py>, package: &Bound<'py, PyAny>, seeds: Option<&Bound<'py, PyAny>>, entry_ids: Option<&Bound<'py, PyAny>>,
        reset_mask: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() { return Err(errors::input(py, InputReason::Dtype, "package must be callable", None)) }
        let n = self.core.num_envs;
        self.core.reset(py, self::seeds(seeds, n, true)?, option_list(entry_ids, n, true, label)?, self::reset_mask(reset_mask, n, true)?, |raw| package.call1((raw,)))
    }
    fn step<'py>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>, tokens: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
        self.core.step(py, actions, tokens, Ok)
    }
    fn _step_deliver<'py>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>, tokens: &Bound<'py, PyAny>, package: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() { return Err(errors::input(py, InputReason::Dtype, "package must be callable", None)) }
        self.core.step(py, actions, tokens, |raw| package.call1((raw,)))
    }
    fn decision<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> { self.core.decision(py, slot) }
    fn committed_result<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> { self.core.committed_result(py, slot) }
    fn recover_committed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> { self.core.recover_committed(py) }
    fn record<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> { self.core.record(py, slot) }
    fn take_completed_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> { self.core.take_completed_records(py) }
    fn take_fault_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> { self.core.take_fault_records(py) }
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> { self.core.diagnostics(py) }
    /// The Gym66 diagnostic (design §3.4.3): `{status, accepted_steps, last_step_committed, sessions, first_fault}` plus
    /// `slots`, the per-slot delivery statuses. `first_fault` is derived from the lowest-index faulted slot.
    fn diagnostic<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _lease = self.core.begin()?;
        let (status, sessions, reports, first) = {
            let state = self.core.lock();
            let batch = &state.batch;
            let status = if batch.is_closed() { "closed" } else if batch.fault().is_some() { "faulted" } else { "active" };
            let sessions = (0..batch.len()).map(|slot| batch.session(slot).expect("slot in range").diagnostic().to_string()).collect::<Vec<_>>();
            let (reports, first) = match batch.fault() {
                Some(fault) => {
                    let first = match fault.operation {
                        BatchOperation::Export => ("delivery", None, "Python combat batch delivery failed".to_owned()),
                        operation => {
                            let slot = fault.slots.iter().position(|r| matches!(r.delivery, SlotDelivery::Faulted { .. }));
                            let panicked = slot.is_none_or(|k| matches!(fault.slots[k].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Panic, .. }));
                            (operation.label(), slot, gym66_reason(operation.label(), panicked))
                        }
                    };
                    (fault.slots.clone(), Some(first))
                }
                None => {
                    let reports = batch.last_statuses().iter().enumerate().map(|(slot, delivery)| SlotReport { delivery: *delivery,
                        error: batch.slot_fault(slot).map(|_| RunSessionError::DeliveryFault("slot isolated after a fault")) }).collect::<Vec<_>>();
                    let first = (0..batch.len()).find_map(|slot| batch.slot_fault(slot).map(|row| {
                        let operation = if row.stage == BatchFaultStage::Reset { "reset" } else { "step" };
                        (operation, Some(slot), gym66_reason(operation, row.stage == BatchFaultStage::Panic))
                    }));
                    (reports, first)
                }
            };
            (status, sessions, reports, first)
        };
        let out = PyDict::new(py);
        out.set_item("status", status)?;
        let accepted = PyList::empty(py);
        let committed = PyList::empty(py);
        let rows = PyList::empty(py);
        for diagnostic in sessions {
            let row = numeric::descriptor(py, diagnostic)?;
            accepted.append(row.get_item("accepted_steps")?)?;
            committed.append(row.get_item("last_step_committed")?)?;
            rows.append(row)?;
        }
        out.set_item("accepted_steps", accepted)?;
        out.set_item("last_step_committed", committed)?;
        out.set_item("sessions", rows)?;
        match first {
            Some((operation, failing_slot, reason)) => {
                let fault = PyDict::new(py);
                fault.set_item("operation", operation)?; fault.set_item("failing_slot", failing_slot)?; fault.set_item("reason", reason)?;
                out.set_item("first_fault", fault)?;
            }
            None => out.set_item("first_fault", py.None())?,
        }
        let slots = PyList::empty(py);
        for r in &reports { slots.append(status_dict(py, &stacking::slot_status(r, true))?)?; }
        out.set_item("slots", slots)?;
        Ok(out.into_any())
    }
    fn observation_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        run_env::specification(py, self.core.read_first(|s| s.observation_spec())?)
    }
    fn environment_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        numeric::descriptor(py, self.core.read_first(|s| s.environment_spec())?.to_string())
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> { self.core.close(py) }
}

/// `NativePublicSearch.step_many` rows (design §3.6): stacked in input order; a faulted entry is a zero row with `fault`.
pub(crate) fn branch_outcomes<'py>(py: Python<'py>, outcomes: Vec<BranchStepOutcome>) -> PyResult<Bound<'py, PyDict>> {
    let count = outcomes.len();
    let stepped = outcomes.iter().filter_map(|o| match o { BranchStepOutcome::Stepped(s) => Some(s), _ => None }).collect::<Vec<_>>();
    let width = stepped.first().map_or(0, |s| s.action_mask.len());
    let out = PyDict::new(py);
    let full = stepped.first().and_then(|s| s.observation.as_ref());
    match full {
        Some(first) => {
            let zero = <RunSession as BatchSession>::batch_zeroed(&RunBoundary { observation: first.clone(), token: RunDecisionToken::from([0_u64, 0]), terminal: None })
                .map_err(run_env::error)?;
            let rows = outcomes.iter().map(|o| match o { BranchStepOutcome::Stepped(s) => s.observation.as_ref().unwrap_or(&zero.observation), _ => &zero.observation })
                .collect::<Vec<_>>();
            out.set_item("obs", stacked_rows(py, &rows)?)?;
        }
        None => out.set_item("obs", py.None())?,
    }
    let mut mask = Vec::with_capacity(count * width);
    for o in &outcomes { match o { BranchStepOutcome::Stepped(s) => mask.extend_from_slice(&s.action_mask), _ => mask.extend(std::iter::repeat_n(0_i8, width)) } }
    out.set_item("action_mask", numeric::array(py, mask, &[count, width])?)?;
    out.set_item("tokens", tokens(py, outcomes.iter().map(|o| match o { BranchStepOutcome::Stepped(s) => s.token, _ => RunDecisionToken::from([0_u64, 0]) }), count)?)?;
    let pick = |f: &dyn Fn(&spire_env::BranchStep) -> bool| outcomes.iter().map(|o| matches!(o, BranchStepOutcome::Stepped(s) if f(s))).collect::<Vec<_>>();
    out.set_item("rewards", numeric::array(py, outcomes.iter().map(|o| match o { BranchStepOutcome::Stepped(s) => s.reward, _ => 0 }).collect(), &[count])?)?;
    out.set_item("terminated", numeric::array(py, pick(&|s| s.terminated), &[count])?)?;
    out.set_item("truncated", numeric::array(py, pick(&|s| s.truncated), &[count])?)?;
    let infos = PyList::empty(py);
    let faults = PyList::empty(py);
    for o in &outcomes {
        match o {
            BranchStepOutcome::Stepped(s) => { infos.append(numeric::descriptor(py, s.info.to_string())?)?; faults.append(py.None())?; }
            BranchStepOutcome::Fault { error, stage, committed } => {
                infos.append(py.None())?;
                faults.append(fault_dict(py, stage, &stacking::error_message(error, false), *committed)?)?;
            }
        }
    }
    out.set_item("info", infos)?;
    out.set_item("fault", faults)?;
    Ok(out)
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NativeRunBatch>()?;
    m.add_class::<NativeCombatTrainingBatch>()?;
    m.add("BatchFaultError", m.py().get_type::<BatchFaultError>())?;
    Ok(())
}
