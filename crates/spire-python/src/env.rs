//! Frozen shared Python owner; each mutable call holds a metadata-only lease
//! across detached native computation and attached owned export.
use std::sync::{Arc, Mutex};
use pyo3::{prelude::*, types::{PyAny, PyDict, PyList}};
use spire_core::combat::rng::Partition;
use spire_env::{AutoReset, CombatPool, EpisodeRecord, InputReason, PoolConfig,
    ResetRequest, ScenarioCatalog, expand_seeds};
use crate::{config, errors::{input, native}, export_guard::{export_committed, export_retained}, numeric};

#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeCombatPool {
    pool: CombatPool,
    catalog: Arc<ScenarioCatalog>,
    scenario_ids: Vec<String>,
    batch: usize,
    // Native drain is a transfer. Until attached conversion succeeds, this
    // binding retains the full owned evidence, including after close/fault.
    pending_records: Mutex<Vec<EpisodeRecord>>,
}

#[pymethods]
impl NativeCombatPool {
    #[new]
    #[pyo3(signature = (catalog=None, *, num_envs=1, workers=1,
        partition="train".to_owned(), autoreset="manual".to_owned(), record_replay=false),
        text_signature = "(catalog=None, *, num_envs=1, workers=1, partition='train', autoreset='manual', record_replay=False)")]
    fn new(py: Python<'_>, catalog: Option<&Bound<'_, PyAny>>,
        #[pyo3(from_py_with = config::size)] num_envs: usize,
        #[pyo3(from_py_with = config::size)] workers: usize,
        #[pyo3(from_py_with = config::label)] partition: String,
        #[pyo3(from_py_with = config::label)] autoreset: String,
        #[pyo3(from_py_with = config::boolean)] record_replay: bool) -> PyResult<Self> {
        let catalog = Arc::new(config::catalog(catalog)?);
        let batch = num_envs;
        let partition = match partition.as_str() {
            "train" => Partition::Train, "selection" => Partition::Selection,
            "final" => Partition::Final, "test" => Partition::Test,
            _ => return Err(input(py, InputReason::Range, "unknown partition", None)),
        };
        let autoreset = match autoreset.as_str() {
            "manual" => AutoReset::Manual, "same_step" => AutoReset::SameStep,
            _ => return Err(input(py, InputReason::Range, "unknown autoreset mode", None)),
        };
        let config = PoolConfig { batch_size:batch, workers, partition, autoreset,
            record_replay };
        let scenario_ids = catalog.descriptor()["templates"].as_array().expect("native catalog schema")
            .iter().map(|t|t["id"].as_str().expect("native ID").to_owned()).collect();
        let owner = catalog.clone();
        let pool = py.detach(move || CombatPool::new(owner, config)).map_err(|e|native(py, e))?;
        Ok(Self { pool, catalog, scenario_ids, batch, pending_records:Mutex::new(Vec::new()) })
    }

    fn observation_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let spec = spire_env::observation_spec();
        let descriptor = numeric::descriptor(py, spec.descriptor().to_string())?;
        // Publish native metadata without modifying the descriptor hash input.
        let fingerprint = spec.fingerprint().iter().map(|byte|format!("{byte:02x}")).collect::<String>();
        descriptor.set_item("fingerprint", fingerprint)?;
        Ok(descriptor)
    }
    fn catalog_descriptor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        numeric::descriptor(py, self.catalog.descriptor().to_string())
    }

    #[pyo3(signature = (indices, *, seed=None, scenario_ids=None, return_all=false))]
    fn reset<'py>(&self, py: Python<'py>, indices: &Bound<'py, PyAny>,
        seed: Option<&Bound<'py, PyAny>>, scenario_ids: Option<&Bound<'py, PyAny>>,
        #[pyo3(from_py_with = config::boolean)] return_all: bool) -> PyResult<Bound<'py, PyDict>> {
        self.reset_deliver(py, indices, seed, scenario_ids, return_all, Ok)
    }

    fn step_batch<'py>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>, tokens: &Bound<'py, PyAny>)
        -> PyResult<Bound<'py, PyDict>> {
        self.step_deliver(py, actions, tokens, Ok)
    }

    // Internal facade transport, not a gameplay callback: invoked attached only
    // after workers quiesce and numeric export completes. The original atomic
    // operation lease (and its exact delivery range) lives until packaging ends.
    #[pyo3(signature = (indices, package, *, seed=None, scenario_ids=None, return_all=false))]
    fn _reset_deliver<'py>(&self, py: Python<'py>, indices: &Bound<'py, PyAny>,
        package: &Bound<'py, PyAny>, seed: Option<&Bound<'py, PyAny>>,
        scenario_ids: Option<&Bound<'py, PyAny>>,
        #[pyo3(from_py_with = config::boolean)] return_all: bool) -> PyResult<Bound<'py, PyAny>> {
        self.reset_deliver(py, indices, seed, scenario_ids, return_all, |raw|package.call1((raw,)))
    }

    fn _step_deliver<'py>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>,
        tokens: &Bound<'py, PyAny>, package: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        self.step_deliver(py, actions, tokens, |raw|package.call1((raw,)))
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        py.detach(||operation.close()).map_err(|e|native(py, e))
    }

    fn reproduction<'py>(&self, py: Python<'py>, slot: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        let slot = config::integer(slot, InputReason::Range, None)?;
        let value = py.detach(||operation.reproduction(slot)).map_err(|e|native(py, e))?;
        // This is non-mutating explicit export; conversion failure creates no
        // gameplay fault and the native current record remains retrievable.
        match value { Some(value) => Ok(numeric::record(py, &value)?.into_any()), None => Ok(py.None().into_bound(py)) }
    }

    fn take_completed_records<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        let records = py.detach(||operation.take_completed_records()).map_err(|e|native(py, e))?;
        let mut pending = self.pending_records.lock().unwrap_or_else(|p|p.into_inner());
        pending.extend(records);
        export_retained(&mut pending, |values|numeric::records(py, values))
    }

    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        let value = py.detach(||operation.diagnostics()).map_err(|e|native(py, e))?;
        numeric::diagnostics(py, value)
    }

    fn mark_export_fault(&self, py: Python<'_>, message: &Bound<'_, PyAny>) -> PyResult<()> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        let message = config::string(message, None)?;
        py.detach(||operation.mark_export_fault(&message));
        Ok(())
    }
}

impl NativeCombatPool {
    fn reset_deliver<'py, T>(&self, py: Python<'py>, indices: &Bound<'py, PyAny>,
        seed: Option<&Bound<'py, PyAny>>, scenario_ids: Option<&Bound<'py, PyAny>>,
        return_all: bool, package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>) -> PyResult<T> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        let indices = config::indices(indices, self.batch)?;
        let seeds = config::seeds(seed, self.batch)?;
        let scenarios = config::scenarios(scenario_ids, self.batch, &self.scenario_ids)?;
        let seeds = expand_seeds(seeds, self.batch).map_err(|e|native(py, e))?;
        let requests: Vec<_> = indices.iter().map(|&index|ResetRequest {
            seed:seeds[index].clone(), scenario_id:scenarios[index].clone() }).collect();
        let rows = py.detach(|| {
            if return_all { operation.reset_batch_return_all(&indices, &requests) }
            else { operation.reset_batch(&indices, &requests) }
        }).map_err(|e|native(py, e))?;
        if indices.is_empty() {
            // Read-only empty selection creates no committed delivery to fault.
            package(numeric::reset(py, rows, indices)?)
        } else {
            let raw = export_committed(&mut operation, ||numeric::reset(py, rows, indices))?;
            facade_delivery(&mut operation, ||package(raw))
        }
    }

    fn step_deliver<'py, T>(&self, py: Python<'py>, actions: &Bound<'py, PyAny>,
        tokens: &Bound<'py, PyAny>, package: impl FnOnce(Bound<'py, PyDict>) -> PyResult<T>) -> PyResult<T> {
        let mut operation = self.pool.begin_operation().map_err(|e|native(py, e))?;
        let (actions, tokens) = numeric::step_inputs(actions, tokens, self.batch)?;
        let rows = py.detach(||operation.step_batch(&actions, &tokens)).map_err(|e|native(py, e))?;
        let raw = export_committed(&mut operation, ||numeric::step(py, rows))?;
        facade_delivery(&mut operation, ||package(raw))
    }
}

fn facade_delivery<T>(operation: &mut spire_env::PoolOperation<'_>,
    package: impl FnOnce() -> PyResult<T>) -> PyResult<T> {
    let result = package();
    if result.is_err() {
        // Never call user exception formatting while preserving the original
        // exception object (including BaseException and unprintable exceptions).
        operation.mark_export_fault("facade export: packaging failed");
    }
    result
}
