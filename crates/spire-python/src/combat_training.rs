//! Owned Python transport for standalone Ironclad combat and manual-reset batches.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::panic::{catch_unwind, AssertUnwindSafe};

use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyAny, PyBytes, PyDict, PyList, PyString},
};
use rust_numpy::{PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use spire_core::combat::rng::Partition;
use spire_env::{
    CombatPolicyBoundary, CombatTrainingCatalog, CombatTrainingSession, CombatTrainingStep,
    InputReason, RunActionIndex, RunSessionError,
};

use crate::{
    errors, numeric,
    run_env::{
        error as run_error, integer, label, policy_boundary as boundary, specification as spec,
        token,
    },
};

fn guarded_session<T>(
    session: &mut CombatTrainingSession,
    operation: impl FnOnce(&mut CombatTrainingSession) -> Result<T, RunSessionError>,
) -> Result<T, RunSessionError> {
    match catch_unwind(AssertUnwindSafe(|| operation(session))) {
        Ok(result) => result,
        Err(_) => {
            session.fail_delivery("native combat session operation panicked");
            Err(RunSessionError::DeliveryFault(
                "native combat session operation panicked",
            ))
        }
    }
}

fn guarded_delivery<T>(operation: impl FnOnce() -> PyResult<T>) -> PyResult<T> {
    catch_unwind(AssertUnwindSafe(operation)).unwrap_or_else(|_| {
        Err(PyRuntimeError::new_err(
            "native combat transport packaging panicked",
        ))
    })
}

fn catalog_bytes(value: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if value.is_exact_instance_of::<PyString>() {
        return value
            .extract::<String>()
            .map(String::into_bytes)
            .map_err(Into::into);
    }
    if let Ok(bytes) = value.cast::<PyBytes>() {
        return Ok(bytes.as_bytes().to_vec());
    }
    Err(errors::input(
        value.py(),
        InputReason::Dtype,
        "catalog must be UTF-8 JSON text or bytes",
        None,
    ))
}

fn parse_partition(py: Python<'_>, value: &str) -> PyResult<Partition> {
    match value {
        "train" => Ok(Partition::Train),
        "selection" => Ok(Partition::Selection),
        "final" => Ok(Partition::Final),
        "test" => Ok(Partition::Test),
        _ => Err(errors::input(
            py,
            InputReason::Scenario,
            "unknown partition",
            None,
        )),
    }
}

fn optional_seed(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<u64>> {
    value
        .filter(|value| !value.is_none())
        .map(integer::<u64>)
        .transpose()
}

fn optional_label(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<String>> {
    value
        .filter(|value| !value.is_none())
        .map(label)
        .transpose()
}

fn step_raw<'py>(py: Python<'py>, value: CombatTrainingStep) -> PyResult<Bound<'py, PyDict>> {
    let result = boundary(py, value.boundary)?;
    result.set_item("reward", value.reward)?;
    result.set_item("terminated", value.terminated)?;
    result.set_item("truncated", value.truncated)?;
    Ok(result)
}

fn json_value<'py, T: std::fmt::Display>(
    py: Python<'py>,
    value: T,
) -> PyResult<Bound<'py, PyAny>> {
    numeric::descriptor(py, value.to_string())
}

struct SingleLease<'a>(&'a NativeIroncladCombatSession);
impl Drop for SingleLease<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::Release);
    }
}

#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeIroncladCombatSession {
    session: Mutex<CombatTrainingSession>,
    catalog: CombatTrainingCatalog,
    busy: AtomicBool,
}

impl NativeIroncladCombatSession {
    fn begin(&self) -> PyResult<SingleLease<'_>> {
        self.busy
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| PyRuntimeError::new_err("combat training operation is busy"))?;
        Ok(SingleLease(self))
    }

    fn fail_delivery(&self) {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .fail_delivery("Python combat transport delivery failed");
    }

    fn deliver<T>(&self, operation: impl FnOnce() -> PyResult<T>) -> PyResult<T> {
        let result = guarded_delivery(operation);
        if result.is_err() {
            self.fail_delivery();
        }
        result
    }

    fn reset_transport<'py, T>(
        &self,
        py: Python<'py>,
        seed: Option<&Bound<'py, PyAny>>,
        entry_id: Option<&Bound<'py, PyAny>>,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>,
    ) -> PyResult<T> {
        let _lease = self.begin()?;
        let seed = optional_seed(seed)?;
        let entry_id = optional_label(entry_id)?;
        let (published, effective_seed) = py
            .detach(|| {
                let mut session = self
                    .session
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guarded_session(&mut session, |session| {
                    session.preflight_reset(seed, entry_id.as_deref())?;
                    let boundary = session.reset(seed, entry_id.as_deref())?;
                    let effective_seed = session.effective_seed().ok_or(
                        RunSessionError::DeliveryFault("successful combat reset has no seed"),
                    )?;
                    Ok((boundary, effective_seed))
                })
            })
            .map_err(|error| run_error(error))?;
        self.deliver(|| {
            let raw = boundary(py, published)?;
            raw.set_item("effective_seed", effective_seed)?;
            package(raw)
        })
    }

    fn step_transport<'py, T>(
        &self,
        py: Python<'py>,
        index: &Bound<'py, PyAny>,
        token_value: &Bound<'py, PyAny>,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>,
    ) -> PyResult<T> {
        let _lease = self.begin()?;
        let index = RunActionIndex(integer(index)?);
        let token_value = token(token_value)?;
        let step = py
            .detach(|| {
                let mut session = self
                    .session
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guarded_session(&mut session, |session| {
                    session.preflight_step(token_value, index)?;
                    session.step(token_value, index)
                })
            })
            .map_err(|error| run_error(error))?;
        self.deliver(|| package(step_raw(py, step)?))
    }
}

#[pymethods]
impl NativeIroncladCombatSession {
    #[new]
    #[pyo3(signature=(catalog_json,*,partition="train".to_owned()))]
    fn new(
        py: Python<'_>,
        catalog_json: &Bound<'_, PyAny>,
        #[pyo3(from_py_with = label)] partition: String,
    ) -> PyResult<Self> {
        let partition = parse_partition(py, &partition)?;
        let bytes = catalog_bytes(catalog_json)?;
        let catalog = py
            .detach(move || CombatTrainingCatalog::from_json(&bytes))
            .map_err(|error| run_error(error))?;
        let session_catalog = catalog.clone();
        let session = py
            .detach(move || CombatTrainingSession::new(session_catalog, partition))
            .map_err(|error| run_error(error))?;
        Ok(Self {
            session: Mutex::new(session),
            catalog,
            busy: AtomicBool::new(false),
        })
    }

    fn observation_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let value = self
            .session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observation_spec()
            .map_err(|error| run_error(error))?;
        spec(py, value)
    }

    fn environment_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let value = self
            .session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .environment_spec()
            .map_err(|error| run_error(error))?;
        json_value(py, value)
    }

    fn catalog_descriptor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        json_value(py, self.catalog.descriptor())
    }

    #[pyo3(signature=(seed=None,entry_id=None))]
    fn reset<'py>(
        &self,
        py: Python<'py>,
        seed: Option<&Bound<'py, PyAny>>,
        entry_id: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        self.reset_transport(py, seed, entry_id, Ok)
    }

    fn step<'py>(
        &self,
        py: Python<'py>,
        index: &Bound<'py, PyAny>,
        token_value: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyDict>> {
        self.step_transport(py, index, token_value, Ok)
    }

    fn decision<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let _lease = self.begin()?;
        let value = py
            .detach(|| {
                let mut session = self.session
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guarded_session(&mut session, |session| session.decision())
            })
            .map_err(|error| run_error(error))?;
        self.deliver(|| boundary(py, value))
    }

    #[pyo3(signature=(package,*,seed=None,entry_id=None))]
    fn _reset_deliver<'py>(
        &self,
        py: Python<'py>,
        package: &Bound<'py, PyAny>,
        seed: Option<&Bound<'py, PyAny>>,
        entry_id: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() {
            return Err(errors::input(
                py,
                InputReason::Dtype,
                "package must be callable",
                None,
            ));
        }
        self.reset_transport(py, seed, entry_id, |raw| package.call1((raw,)))
    }

    fn _step_deliver<'py>(
        &self,
        py: Python<'py>,
        index: &Bound<'py, PyAny>,
        token_value: &Bound<'py, PyAny>,
        package: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() {
            return Err(errors::input(
                py,
                InputReason::Dtype,
                "package must be callable",
                None,
            ));
        }
        self.step_transport(py, index, token_value, |raw| package.call1((raw,)))
    }

    fn diagnostic<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _lease = self.begin()?;
        let value = py
            .detach(|| {
                self.session
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .diagnostic()
            });
        json_value(py, value)
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        let _lease = self.begin()?;
        py.detach(|| {
            let mut session = self.session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guarded_session(&mut session, |session| session.close())
        })
        .map_err(|error| run_error(error))
    }
}

struct ParsedReset {
    seeds: Vec<Option<u64>>,
    entry_ids: Vec<Option<String>>,
    reset_mask: Vec<bool>,
}

fn parse_option_list<T: Clone>(
    value: Option<&Bound<'_, PyAny>>,
    count: usize,
    parse: impl Fn(&Bound<'_, PyAny>) -> PyResult<T>,
) -> PyResult<Vec<Option<T>>> {
    let Some(value) = value.filter(|value| !value.is_none()) else {
        return Ok(vec![None; count]);
    };
    let list = value.cast::<PyList>().map_err(|_| {
        errors::input(
            value.py(),
            InputReason::Shape,
            "batch reset values must be a length-B list",
            None,
        )
    })?;
    if list.len() != count {
        return Err(errors::input(
            value.py(),
            InputReason::Shape,
            "batch reset values must have length B",
            None,
        ));
    }
    list.iter()
        .map(|item| {
            if item.is_none() {
                Ok(None)
            } else {
                parse(&item).map(Some)
            }
        })
        .collect()
}

fn parse_reset_mask(value: Option<&Bound<'_, PyAny>>, count: usize) -> PyResult<Vec<bool>> {
    let Some(value) = value.filter(|value| !value.is_none()) else {
        return Ok(vec![true; count]);
    };
    let array = value.cast::<PyArrayDyn<bool>>().map_err(|_| {
        errors::input(
            value.py(),
            InputReason::Dtype,
            "reset_mask must be a native bool NumPy array",
            None,
        )
    })?;
    if array.shape() != [count] {
        return Err(errors::input(
            value.py(),
            InputReason::Shape,
            "reset_mask must have shape [B]",
            None,
        ));
    }
    let guard = array.try_readonly().map_err(|_| {
        errors::input(
            value.py(),
            InputReason::Dtype,
            "reset_mask is already mutably borrowed",
            None,
        )
    })?;
    Ok(guard.as_array().iter().copied().collect())
}

fn parse_reset(
    seeds: Option<&Bound<'_, PyAny>>,
    entry_ids: Option<&Bound<'_, PyAny>>,
    reset_mask: Option<&Bound<'_, PyAny>>,
    count: usize,
) -> PyResult<ParsedReset> {
    Ok(ParsedReset {
        seeds: parse_option_list(seeds, count, integer::<u64>)?,
        entry_ids: parse_option_list(entry_ids, count, label)?,
        reset_mask: parse_reset_mask(reset_mask, count)?,
    })
}

fn stack_observations<'py>(
    py: Python<'py>,
    rows: &[CombatPolicyBoundary],
) -> PyResult<Bound<'py, PyDict>> {
    let Some(first) = rows.first() else {
        return Err(PyRuntimeError::new_err("empty combat observation batch"));
    };
    let result = PyDict::new(py);
    for (group_index, first_group) in first.observation.groups.iter().enumerate() {
        let mut shape = Vec::with_capacity(first_group.shape.len() + 1);
        shape.push(rows.len());
        shape.extend_from_slice(&first_group.shape);
        let mut values = Vec::new();
        let mut known = Vec::new();
        let mut occupied = first_group.occupied.as_ref().map(|_| Vec::new());
        for row in rows {
            let Some(group) = row.observation.groups.get(group_index) else {
                return Err(PyRuntimeError::new_err("combat observation groups disagree"));
            };
            if group.key != first_group.key || group.shape != first_group.shape {
                return Err(PyRuntimeError::new_err("combat observation layouts disagree"));
            }
            values.extend_from_slice(&group.values);
            known.extend_from_slice(&group.known);
            match (&mut occupied, &group.occupied) {
                (Some(target), Some(source)) => target.extend_from_slice(source),
                (None, None) => {}
                _ => return Err(PyRuntimeError::new_err("combat occupancy layouts disagree")),
            }
        }
        result.set_item(first_group.key, numeric::array(py, values, &shape)?)?;
        result.set_item(
            format!("{}_known", first_group.key),
            numeric::array(py, known, &shape)?,
        )?;
        if let Some(occupied) = occupied {
            result.set_item(
                format!("{}_occupied", first_group.key),
                numeric::array(py, occupied, &[rows.len(), first_group.occupied.as_ref().unwrap().len()])?,
            )?;
        }
    }
    let mut action_mask = Vec::new();
    for row in rows {
        action_mask.extend_from_slice(&row.observation.action_mask);
    }
    result.set_item(
        "action_mask",
        numeric::array(
            py,
            action_mask,
            &[rows.len(), first.observation.action_mask.len()],
        )?,
    )?;
    Ok(result)
}

fn batch_boundaries<'py>(
    py: Python<'py>,
    rows: Vec<CombatPolicyBoundary>,
) -> PyResult<Bound<'py, PyDict>> {
    let count = rows.len();
    let result = PyDict::new(py);
    result.set_item("obs", stack_observations(py, &rows)?)?;
    let mut tokens = Vec::with_capacity(count * 2);
    let mut infos = Vec::with_capacity(count);
    for row in rows {
        let token: [u64; 2] = row.token.into();
        tokens.extend_from_slice(&token);
        infos.push(row.info);
    }
    result.set_item("tokens", numeric::array(py, tokens, &[count, 2])?)?;
    let info_rows = PyList::empty(py);
    for info in infos {
        info_rows.append(json_value(py, info)?)?;
    }
    result.set_item("info", info_rows)?;
    Ok(result)
}

fn batch_step_raw<'py>(
    py: Python<'py>,
    rows: Vec<CombatTrainingStep>,
) -> PyResult<Bound<'py, PyDict>> {
    let count = rows.len();
    let mut boundaries = Vec::with_capacity(count);
    let mut rewards = Vec::with_capacity(count);
    let mut terminated = Vec::with_capacity(count);
    let mut truncated = Vec::with_capacity(count);
    for row in rows {
        boundaries.push(row.boundary);
        rewards.push(row.reward);
        terminated.push(row.terminated);
        truncated.push(row.truncated);
    }
    let result = batch_boundaries(py, boundaries)?;
    result.set_item("rewards", numeric::array(py, rewards, &[count])?)?;
    result.set_item("terminated", numeric::array(py, terminated, &[count])?)?;
    result.set_item("truncated", numeric::array(py, truncated, &[count])?)?;
    Ok(result)
}

struct BatchState {
    sessions: Vec<CombatTrainingSession>,
    faulted: bool,
    closed: bool,
    first_fault: Option<BatchFault>,
    current_slot: Option<usize>,
    mutation_started: bool,
}

#[derive(Clone, Copy)]
struct BatchFault {
    operation: &'static str,
    failing_slot: Option<usize>,
    reason: &'static str,
}

#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeIroncladCombatBatch {
    state: Mutex<BatchState>,
    catalog: CombatTrainingCatalog,
    num_envs: usize,
    busy: AtomicBool,
}

struct BatchLease<'a>(&'a NativeIroncladCombatBatch);
impl Drop for BatchLease<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::Release);
    }
}

fn retained_error(error: &RunSessionError) -> bool {
    matches!(
        error,
        RunSessionError::Core(_)
            | RunSessionError::Capacity { .. }
            | RunSessionError::UnsupportedBoundary(_)
            | RunSessionError::DeliveryFault(_)
    )
}

impl NativeIroncladCombatBatch {
    fn begin(&self) -> PyResult<BatchLease<'_>> {
        self.busy
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| PyRuntimeError::new_err("combat batch operation is busy"))?;
        Ok(BatchLease(self))
    }

    fn freeze(
        state: &mut BatchState,
        reason: &'static str,
        operation: &'static str,
        failing_slot: Option<usize>,
    ) {
        if !state.faulted {
            for session in &mut state.sessions {
                session.fail_delivery(reason);
            }
            state.faulted = true;
            state.first_fault = Some(BatchFault {
                operation,
                failing_slot,
                reason,
            });
        }
    }

    fn freeze_delivery(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Self::freeze(
            &mut state,
            "Python combat batch delivery failed",
            "delivery",
            None,
        );
    }

    fn deliver<T>(&self, operation: impl FnOnce() -> PyResult<T>) -> PyResult<T> {
        let result = guarded_delivery(operation);
        if result.is_err() {
            self.freeze_delivery();
        }
        result
    }

    fn reset_transport<'py, T>(
        &self,
        py: Python<'py>,
        inputs: ParsedReset,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>,
    ) -> PyResult<T> {
        let _lease = self.begin()?;
        let boundaries = py
            .detach(|| {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guarded_batch(
                    &mut state,
                    "reset",
                    "native combat batch reset panicked",
                    |state| {
                        if state.closed {
                            return Err(RunSessionError::Closed);
                        }
                        if state.faulted {
                            return Err(RunSessionError::DeliveryFault("combat batch is faulted"));
                        }
                        // Input and lifecycle preflight cover every row before any reset.
                        for index in 0..self.num_envs {
                            state.current_slot = Some(index);
                            guarded_session(&mut state.sessions[index], |session| {
                                session.preflight_reset(
                                    inputs.seeds[index],
                                    inputs.entry_ids[index].as_deref(),
                                )?;
                                if !inputs.reset_mask[index] {
                                    session.decision()?;
                                }
                                Ok(())
                            })?;
                        }
                        let mut boundaries = Vec::with_capacity(self.num_envs);
                        for index in 0..self.num_envs {
                            state.current_slot = Some(index);
                            state.mutation_started |= inputs.reset_mask[index];
                            let boundary = guarded_session(&mut state.sessions[index], |session| {
                                if inputs.reset_mask[index] {
                                    session.reset(
                                        inputs.seeds[index],
                                        inputs.entry_ids[index].as_deref(),
                                    )
                                } else {
                                    session.decision()
                                }
                            })?;
                            boundaries.push(boundary);
                        }
                        Ok(boundaries)
                    },
                )
            })
            .map_err(|error| run_error(error))?;
        self.deliver(|| package(batch_boundaries(py, boundaries)?))
    }

    fn step_transport<'py, T>(
        &self,
        py: Python<'py>,
        actions: &Bound<'py, PyAny>,
        tokens: &Bound<'py, PyAny>,
        package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>,
    ) -> PyResult<T> {
        let _lease = self.begin()?;
        let (actions, tokens) = numeric::step_inputs(actions, tokens, self.num_envs)?;
        let steps = py
            .detach(|| {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guarded_batch(
                    &mut state,
                    "step",
                    "native combat batch step panicked",
                    |state| {
                        if state.closed {
                            return Err(RunSessionError::Closed);
                        }
                        if state.faulted {
                            return Err(RunSessionError::DeliveryFault("combat batch is faulted"));
                        }
                        // Validate every token, terminal state and action before dispatching row zero.
                        for index in 0..self.num_envs {
                            state.current_slot = Some(index);
                            guarded_session(&mut state.sessions[index], |session| {
                                session.preflight_step(
                                    tokens[index].into(),
                                    RunActionIndex(actions[index]),
                                )
                            })?;
                        }
                        let mut steps = Vec::with_capacity(self.num_envs);
                        for index in 0..self.num_envs {
                            state.current_slot = Some(index);
                            state.mutation_started = true;
                            let step = guarded_session(&mut state.sessions[index], |session| {
                                session.step(
                                    tokens[index].into(),
                                    RunActionIndex(actions[index]),
                                )
                            })?;
                            steps.push(step);
                        }
                        Ok(steps)
                    },
                )
            })
            .map_err(|error| run_error(error))?;
        self.deliver(|| package(batch_step_raw(py, steps)?))
    }
}

fn guarded_batch<T>(
    state: &mut BatchState,
    operation: &'static str,
    panic_reason: &'static str,
    call: impl FnOnce(&mut BatchState) -> Result<T, RunSessionError>,
) -> Result<T, RunSessionError> {
    state.current_slot = None;
    state.mutation_started = false;
    let result = catch_unwind(AssertUnwindSafe(|| call(state)));
    match result {
        Ok(Ok(value)) => {
            state.current_slot = None;
            state.mutation_started = false;
            Ok(value)
        }
        Ok(Err(error)) => {
            if retained_error(&error) || state.mutation_started {
                NativeIroncladCombatBatch::freeze(
                    state,
                    "native combat batch runtime operation failed",
                    operation,
                    state.current_slot,
                );
            }
            state.current_slot = None;
            state.mutation_started = false;
            Err(error)
        }
        Err(_) => {
            let failing_slot = state.current_slot;
            NativeIroncladCombatBatch::freeze(state, panic_reason, operation, failing_slot);
            state.current_slot = None;
            state.mutation_started = false;
            Err(RunSessionError::DeliveryFault(panic_reason))
        }
    }
}

#[cfg(test)]
mod transport_fault_tests {
    use super::*;
    use spire_env::RunDecisionToken;

    fn catalog() -> CombatTrainingCatalog {
        CombatTrainingCatalog::from_json(
            br#"{"entries":[{"id":"fixture","encounter":"cultist","hp":80,"max_hp":80,"deck":[{"card":"strike","upgrades":0}],"relics":[],"potions":[null,null,null],"relic_counters":{}}]}"#,
        )
        .unwrap()
    }

    fn new_session() -> CombatTrainingSession {
        CombatTrainingSession::new(catalog(), Partition::Test).unwrap()
    }

    fn reset_session(session: &mut CombatTrainingSession) -> (RunDecisionToken, RunActionIndex) {
        let boundary = guarded_session(session, |session| {
            session.reset(Some(7), Some("fixture"))
        })
        .unwrap();
        let index = boundary
            .observation
            .action_mask
            .iter()
            .position(|legal| *legal == 1)
            .unwrap();
        (boundary.token, RunActionIndex(index as i64))
    }

    #[test]
    fn accepted_single_step_followed_by_native_panic_is_retained_and_not_retryable() {
        let mut session = new_session();
        let (token, index) = reset_session(&mut session);
        let result = guarded_session(&mut session, |session| {
            session.preflight_step(token, index)?;
            let _accepted = session.step(token, index)?;
            panic!("test panic after an accepted combat step");
            #[allow(unreachable_code)]
            Ok(())
        });

        assert!(matches!(result, Err(RunSessionError::DeliveryFault(_))));
        let diagnostic = session.diagnostic();
        assert_eq!(diagnostic["accepted_steps"].as_u64(), Some(1));
        assert_eq!(diagnostic["last_step_committed"], true);
        assert!(diagnostic["fault"].is_string());
        assert!(session.reset(Some(8), Some("fixture")).is_err());
    }

    #[test]
    fn partial_batch_step_error_freezes_every_session_and_keeps_accepted_counts() {
        let mut first = new_session();
        let mut second = new_session();
        let first_action = reset_session(&mut first);
        let _second_action = reset_session(&mut second);
        let mut state = BatchState {
            sessions: vec![first, second],
            faulted: false,
            closed: false,
            first_fault: None,
            current_slot: None,
            mutation_started: false,
        };

        let result = guarded_batch::<()>(
            &mut state,
            "step",
            "native combat batch step panicked",
            |state| {
                state.current_slot = Some(0);
                state.mutation_started = true;
                guarded_session(&mut state.sessions[0], |session| {
                    session.preflight_step(first_action.0, first_action.1)?;
                    session.step(first_action.0, first_action.1).map(|_| ())
                })?;
                state.current_slot = Some(1);
                Err(RunSessionError::Input(
                    "test callback error after row zero accepted",
                ))
            },
        );

        assert!(matches!(result, Err(RunSessionError::Input(_))));
        assert!(state.faulted);
        assert_eq!(state.first_fault.unwrap().failing_slot, Some(1));
        let accepted = state
            .sessions
            .iter()
            .map(|session| session.diagnostic()["accepted_steps"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(accepted, [1, 0]);
        assert!(state.sessions[0].reset(Some(9), Some("fixture")).is_err());
        assert!(state.sessions[1].reset(Some(9), Some("fixture")).is_err());
    }

    #[test]
    fn batch_panic_after_accepted_row_freezes_every_session() {
        let mut first = new_session();
        let mut second = new_session();
        let first_action = reset_session(&mut first);
        let _second_action = reset_session(&mut second);
        let mut state = BatchState {
            sessions: vec![first, second],
            faulted: false,
            closed: false,
            first_fault: None,
            current_slot: None,
            mutation_started: false,
        };

        let result = guarded_batch(
            &mut state,
            "step",
            "native combat batch step panicked",
            |state| {
                state.current_slot = Some(0);
                state.mutation_started = true;
                guarded_session(&mut state.sessions[0], |session| {
                    session.preflight_step(first_action.0, first_action.1)?;
                    session.step(first_action.0, first_action.1).map(|_| ())
                })?;
                panic!("test panic after first batch row was accepted");
                #[allow(unreachable_code)]
                Ok(())
            },
        );

        assert!(matches!(result, Err(RunSessionError::DeliveryFault(_))));
        assert!(state.faulted);
        assert_eq!(state.first_fault.unwrap().failing_slot, Some(0));
        let accepted = state
            .sessions
            .iter()
            .map(|session| session.diagnostic()["accepted_steps"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(accepted, [1, 0]);
        assert!(state.sessions[0].reset(Some(9), Some("fixture")).is_err());
        assert!(state.sessions[1].reset(Some(9), Some("fixture")).is_err());
    }
}

#[pymethods]
impl NativeIroncladCombatBatch {
    #[new]
    #[pyo3(signature=(num_envs,catalog_json,*,partition="train".to_owned()))]
    fn new(
        py: Python<'_>,
        num_envs: &Bound<'_, PyAny>,
        catalog_json: &Bound<'_, PyAny>,
        #[pyo3(from_py_with = label)] partition: String,
    ) -> PyResult<Self> {
        let num_envs = integer::<usize>(num_envs)?;
        if !(1..=256).contains(&num_envs) {
            return Err(errors::input(
                py,
                InputReason::Range,
                "num_envs must be in 1..=256",
                None,
            ));
        }
        let partition = parse_partition(py, &partition)?;
        let bytes = catalog_bytes(catalog_json)?;
        let catalog = py
            .detach(move || CombatTrainingCatalog::from_json(&bytes))
            .map_err(|error| run_error(error))?;
        let session_catalog = catalog.clone();
        let sessions = py
            .detach(move || {
                let mut sessions = Vec::new();
                sessions
                    .try_reserve_exact(num_envs)
                    .map_err(|_| RunSessionError::DeliveryFault("combat batch allocation failed"))?;
                for _ in 0..num_envs {
                    sessions.push(CombatTrainingSession::new(
                        session_catalog.clone(),
                        partition,
                    )?);
                }
                Ok::<_, RunSessionError>(sessions)
            })
            .map_err(|error| run_error(error))?;
        Ok(Self {
            state: Mutex::new(BatchState {
                sessions,
                faulted: false,
                closed: false,
                first_fault: None,
                current_slot: None,
                mutation_started: false,
            }),
            catalog,
            num_envs,
            busy: AtomicBool::new(false),
        })
    }

    fn observation_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = state.sessions[0]
            .observation_spec()
            .map_err(|error| run_error(error))?;
        spec(py, value)
    }

    fn environment_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = state.sessions[0]
            .environment_spec()
            .map_err(|error| run_error(error))?;
        json_value(py, value)
    }

    fn catalog_descriptor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        json_value(py, self.catalog.descriptor())
    }

    #[pyo3(signature=(seeds=None,entry_ids=None,reset_mask=None))]
    fn reset<'py>(
        &self,
        py: Python<'py>,
        seeds: Option<&Bound<'py, PyAny>>,
        entry_ids: Option<&Bound<'py, PyAny>>,
        reset_mask: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let inputs = parse_reset(seeds, entry_ids, reset_mask, self.num_envs)?;
        self.reset_transport(py, inputs, Ok)
    }

    fn step<'py>(
        &self,
        py: Python<'py>,
        actions: &Bound<'py, PyAny>,
        tokens: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyDict>> {
        self.step_transport(py, actions, tokens, Ok)
    }

    #[pyo3(signature=(package,*,seeds=None,entry_ids=None,reset_mask=None))]
    fn _reset_deliver<'py>(
        &self,
        py: Python<'py>,
        package: &Bound<'py, PyAny>,
        seeds: Option<&Bound<'py, PyAny>>,
        entry_ids: Option<&Bound<'py, PyAny>>,
        reset_mask: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() {
            return Err(errors::input(
                py,
                InputReason::Dtype,
                "package must be callable",
                None,
            ));
        }
        let inputs = parse_reset(seeds, entry_ids, reset_mask, self.num_envs)?;
        self.reset_transport(py, inputs, |raw| package.call1((raw,)))
    }

    fn _step_deliver<'py>(
        &self,
        py: Python<'py>,
        actions: &Bound<'py, PyAny>,
        tokens: &Bound<'py, PyAny>,
        package: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if !package.is_callable() {
            return Err(errors::input(
                py,
                InputReason::Dtype,
                "package must be callable",
                None,
            ));
        }
        self.step_transport(py, actions, tokens, |raw| package.call1((raw,)))
    }

    fn diagnostic<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _lease = self.begin()?;
        let (status, first_fault, session_diagnostics) = py.detach(|| {
            let state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let status = if state.closed {
                "closed"
            } else if state.faulted {
                "faulted"
            } else {
                "active"
            };
            let first_fault = state.first_fault;
            let sessions = state
                .sessions
                .iter()
                .map(|session| session.diagnostic().to_string())
                .collect::<Vec<_>>();
            (status, first_fault, sessions)
        });
        let result = PyDict::new(py);
        result.set_item("status", status)?;
        let accepted = PyList::empty(py);
        let committed = PyList::empty(py);
        let sessions = PyList::empty(py);
        for diagnostic in session_diagnostics {
            let row = numeric::descriptor(py, diagnostic)?;
            accepted.append(row.get_item("accepted_steps")?)?;
            committed.append(row.get_item("last_step_committed")?)?;
            sessions.append(row)?;
        }
        result.set_item("accepted_steps", accepted)?;
        result.set_item("last_step_committed", committed)?;
        result.set_item("sessions", sessions)?;
        if let Some(fault) = first_fault {
            let fault_row = PyDict::new(py);
            fault_row.set_item("operation", fault.operation)?;
            fault_row.set_item("failing_slot", fault.failing_slot)?;
            fault_row.set_item("reason", fault.reason)?;
            result.set_item("first_fault", fault_row)?;
        } else {
            result.set_item("first_fault", py.None())?;
        }
        Ok(result.into_any())
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        let _lease = self.begin()?;
        let result = py.detach(|| {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guarded_batch(
                &mut state,
                "close",
                "native combat batch close panicked",
                |state| {
                    let mut first_error = None;
                    for index in 0..state.sessions.len() {
                        state.current_slot = Some(index);
                        if let Err(error) = guarded_session(&mut state.sessions[index], |session| {
                            session.close()
                        }) {
                            first_error.get_or_insert(error);
                        }
                    }
                    state.current_slot = None;
                    state.closed = true;
                    first_error.map_or(Ok(()), Err)
                }
            )
        });
        result.map_err(|error| run_error(error))
    }
}
