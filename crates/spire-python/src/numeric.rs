//! Owned rust-numpy columns. No simulator state or borrowed buffer is exported.
use pyo3::{prelude::*, types::{PyBytes, PyDict, PyList}};
use rust_numpy::{Element, PyArray1, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods,
    PyArrayDescrMethods, PyFixedUnicode, ndarray::IxDyn};
use spire_env::{DecisionRows, EpisodeInfo, EpisodeRecord, InputReason, ObservationBatch,
    PoolDiagnostics, ResetRows, StepBatch, TensorValues};
use crate::errors::{faults, input};

fn array<'py, T: Element>(py: Python<'py>, values: Vec<T>, shape: &[usize]) -> PyResult<Bound<'py, PyAny>> {
    Ok(PyArray1::from_vec(py, values).reshape(IxDyn(shape))?.into_any())
}

fn input_values<T: Element + Copy>(value: &Bound<'_, PyAny>, shape: &[usize]) -> PyResult<Vec<T>> {
    let py = value.py();
    let array = value.cast::<PyArrayDyn<T>>()
        .map_err(|_|input(py, InputReason::Dtype, "exact native-endian NumPy dtype required", None))?;
    if array.dtype().is_native_byteorder() != Some(true) {
        return Err(input(py, InputReason::Dtype, "non-native byte order", None));
    }
    if array.shape() != shape {
        return Err(input(py, InputReason::Shape, "actions [B] and tokens [B,2] required", None));
    }
    let guard = array.try_readonly().map_err(|_|input(py, InputReason::Dtype, "input already mutably borrowed", None))?;
    if array.is_aligned() {
        Ok(guard.as_array().iter().copied().collect())
    } else {
        // ndarray references require alignment, which a valid NumPy buffer need
        // not have. NumPy's C copy handles byte strides/unaligned memory before
        // any Rust reference is formed. T is unchanged: this is not coercion.
        let aligned = array.cast_array::<T>(false)?;
        let aligned = aligned.try_readonly().map_err(|_|input(py, InputReason::Dtype, "copy borrow failed", None))?;
        Ok(aligned.as_array().iter().copied().collect())
    }
}

/// Guards and ndarray views die here, before the caller detaches.
pub(crate) fn step_inputs(actions: &Bound<'_, PyAny>, tokens: &Bound<'_, PyAny>, batch: usize)
    -> PyResult<(Vec<i64>, Vec<[u64;2]>)> {
    let actions = input_values::<i64>(actions, &[batch])?;
    let tokens = input_values::<u64>(tokens, &[batch,2])?;
    Ok((actions,tokens.chunks_exact(2).map(|t|[t[0],t[1]]).collect()))
}

fn observation<'py>(py: Python<'py>, batch: ObservationBatch) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    for tensor in batch.tensors {
        let value = match tensor.values {
            TensorValues::I32(v) => array(py, v, &tensor.shape)?,
            TensorValues::I8(v) => array(py, v, &tensor.shape)?,
        };
        result.set_item(tensor.key, value)?;
    }
    Ok(result)
}

fn text(value: &str) -> PyFixedUnicode<64> {
    // Episode IDs fit 28 chars, validated scenario IDs fit 64 ASCII chars.
    let mut codepoints = [0;64];
    for (target, source) in codepoints.iter_mut().zip(value.chars()) { *target=source as u32; }
    codepoints.into()
}
fn info<'py>(py: Python<'py>, values: Vec<EpisodeInfo>) -> PyResult<Bound<'py, PyDict>> {
    let count = values.len();
    let result = PyDict::new(py);
    macro_rules! column {
        ($key:literal, $value:expr) => { result.set_item($key, array(py,
            values.iter().map($value).collect::<Vec<_>>(), &[count])?)?; };
    }
    column!("episode_id", |v:&EpisodeInfo|text(&v.episode_id));
    column!("scenario_id", |v:&EpisodeInfo|text(&v.scenario_id));
    column!("episode_steps", |v:&EpisodeInfo|v.episode_steps);
    column!("outcome", |v:&EpisodeInfo|v.outcome);
    column!("hp_before_healing", |v:&EpisodeInfo|v.hp_before_healing.unwrap_or(0));
    column!("hp_after_cleanup", |v:&EpisodeInfo|v.hp_after_cleanup.unwrap_or(0));
    column!("hp_before_healing_mask", |v:&EpisodeInfo|v.hp_before_healing.is_some());
    column!("hp_after_cleanup_mask", |v:&EpisodeInfo|v.hp_after_cleanup.is_some());
    Ok(result)
}

fn decisions<'py>(py: Python<'py>, rows: DecisionRows) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    let count = rows.tokens.len();
    result.set_item("obs", observation(py, rows.observations)?)?;
    result.set_item("tokens", array(py, rows.tokens.into_iter().flatten().collect(), &[count,2])?)?;
    result.set_item("info", info(py, rows.infos)?)?;
    Ok(result)
}

pub(crate) fn reset<'py>(py: Python<'py>, rows: ResetRows, indices: Vec<usize>) -> PyResult<Bound<'py, PyDict>> {
    let result = decisions(py, rows.decisions)?;
    let count = indices.len();
    result.set_item("reset_indices", array(py, indices.into_iter().map(|i|i as i64).collect(), &[count])?)?;
    let seeds = PyList::empty(py);
    for seed in rows.effective_seeds {
        if let Some(seed) = seed { seeds.append(PyBytes::new(py, seed.as_be_bytes()))?; }
        else { seeds.append(py.None())?; }
    }
    result.set_item("effective_seed_bytes", seeds)?;
    Ok(result)
}

pub(crate) fn step<'py>(py: Python<'py>, batch: StepBatch) -> PyResult<Bound<'py, PyDict>> {
    let result = decisions(py, batch.current)?;
    let count = batch.rewards.len();
    result.set_item("rewards", array(py, batch.rewards, &[count])?)?;
    result.set_item("terminated", array(py, batch.terminated, &[count])?)?;
    result.set_item("truncated", array(py, batch.truncated, &[count])?)?;
    let finals = batch.finals;
    let count = finals.slot_indices.len();
    result.set_item("final_indices", array(py, finals.slot_indices.into_iter().map(|i|i as i64).collect(), &[count])?)?;
    result.set_item("final_obs", observation(py, finals.observations)?)?;
    result.set_item("final_info", info(py, finals.infos)?)?;
    Ok(result)
}

/// JSON is restricted to explicit static descriptor/reproduction exports, never
/// numeric reset/step. Native serde_json descriptors remain the schema authority.
pub(crate) fn descriptor<'py>(py: Python<'py>, json: String) -> PyResult<Bound<'py, PyAny>> {
    py.import("json")?.call_method1("loads", (json,))
}

pub(crate) fn record<'py>(py: Python<'py>, value: &EpisodeRecord) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    result.set_item("record_schema_id", &value.record_schema_id)?;
    result.set_item("catalog_descriptor", descriptor(py, value.catalog_descriptor.to_string())?)?;
    result.set_item("content_id", &value.content_id)?;
    result.set_item("schema_id", &value.schema_id)?;
    result.set_item("schema_fingerprint", PyBytes::new(py, &value.schema_fingerprint))?;
    result.set_item("seed_adapter_id", &value.seed_adapter_id)?;
    result.set_item("rng_contract_id", &value.rng_contract_id)?;
    result.set_item("effective_seed_bytes", PyBytes::new(py, value.job.seed().as_be_bytes()))?;
    result.set_item("root", PyBytes::new(py, &value.job.root()))?;
    result.set_item("catalog_fingerprint", PyBytes::new(py, &value.job.catalog_fingerprint()))?;
    result.set_item("partition", value.job.partition().as_str())?;
    result.set_item("episode_id", value.job.episode_id())?;
    result.set_item("ordinal", value.job.ordinal())?;
    result.set_item("scenario_id", &value.job.template().id)?;
    result.set_item("actions", array(py, value.actions.clone(), &[value.actions.len()])?)?;
    result.set_item("status", format!("{:?}", value.status).to_lowercase())?;
    result.set_item("attempted_action", value.attempted_action)?;
    let list = faults(py, value.fault.as_slice())?;
    result.set_item("fault", if list.is_empty() { py.None().into_bound(py) } else { list.get_item(0)? })?;
    Ok(result)
}

pub(crate) fn records<'py>(py: Python<'py>, values: &[EpisodeRecord]) -> PyResult<Bound<'py, PyList>> {
    let result = PyList::empty(py);
    for value in values { result.append(record(py, value)?)?; }
    Ok(result)
}

pub(crate) fn diagnostics<'py>(py: Python<'py>, value: PoolDiagnostics) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new(py);
    result.set_item("status", format!("{:?}", value.status).to_lowercase())?;
    result.set_item("faults", faults(py, &value.faults)?)?;
    let slots = PyList::empty(py);
    for value in value.slots {
        let slot = PyDict::new(py);
        slot.set_item("slot", value.slot)?;
        slot.set_item("status", format!("{:?}", value.status).to_lowercase())?;
        slot.set_item("episode_id", value.episode_id)?;
        slot.set_item("next_ordinal", value.next_ordinal)?;
        // Explicit privileged metadata only. Never expose snapshot import/export.
        slot.set_item("token", value.token.to_vec())?;
        slots.append(slot)?;
    }
    result.set_item("slots", slots)?;
    Ok(result)
}
