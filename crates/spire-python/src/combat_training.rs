//! Owned Python transport for standalone Ironclad combat and manual-reset batches.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::panic::{catch_unwind, AssertUnwindSafe};

use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyAny, PyBytes, PyDict, PySequence, PyString},
};
use pyo3::exceptions::PyTypeError;
use spire_core::combat::rng::Partition;
use spire_env::{
    CombatTrainingCatalog, CombatTrainingSession, CombatTrainingStep,
    InputReason, RunActionIndex, RunSessionError,
};

use crate::{
    errors, numeric,
    run_env::{
        error as run_error, integer, label, policy_boundary as boundary,
        specification as spec, token, search_limits, public_search, NativePublicSearch,
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

pub(crate) fn catalog_bytes(value: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
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

pub(crate) fn records_bytes(value: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<Vec<u8>>> {
    let Some(value) = value.filter(|value| !value.is_none()) else {
        return Ok(Vec::new());
    };
    if value.is_instance_of::<PyBytes>() || value.is_instance_of::<PyString>() {
        return Err(PyTypeError::new_err("records must be a sequence of bytes"));
    }
    let sequence = value
        .cast::<PySequence>()
        .map_err(|_| PyTypeError::new_err("records must be a sequence of bytes"))?;
    (0..sequence.len()?)
        .map(|index| {
            let item = sequence.get_item(index)?;
            item.cast::<PyBytes>()
                .map(|bytes| bytes.as_bytes().to_vec())
                .map_err(|_| PyTypeError::new_err("records must contain only bytes"))
        })
        .collect()
}

pub(crate) fn parse_partition(py: Python<'_>, value: &str) -> PyResult<Partition> {
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

struct SingleLease<'a>(&'a OwnedCombatSession);
impl Drop for SingleLease<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::Release);
    }
}

struct OwnedCombatSession {
    session: Mutex<CombatTrainingSession>,
    catalog: std::sync::Arc<CombatTrainingCatalog>,
    busy: AtomicBool,
}

impl OwnedCombatSession {
    fn new(
        py: Python<'_>,
        catalog_json: &Bound<'_, PyAny>,
        partition: String,
        records: Vec<Vec<u8>>,
        standard: bool,
    ) -> PyResult<Self> {
        let partition = parse_partition(py, &partition)?;
        let bytes = catalog_bytes(catalog_json)?;
        let catalog = py
            .detach(move || {
                if standard {
                    CombatTrainingCatalog::standard_from_json_with_records(&bytes, &records)
                } else {
                    CombatTrainingCatalog::from_json(&bytes)
                }
            })
            .map_err(|error| run_error(error))?;
        // One shared catalog `Arc` (design §3.1), as the batch holds it.
        let catalog = std::sync::Arc::new(catalog);
        let session_catalog = catalog.clone();
        let session = py
            .detach(move || {
                if standard {
                    CombatTrainingSession::new_standard_shared(session_catalog, partition)
                } else {
                    CombatTrainingSession::new_shared(session_catalog, partition)
                }
            })
            .map_err(|error| run_error(error))?;
        Ok(Self {
            session: Mutex::new(session),
            catalog,
            busy: AtomicBool::new(false),
        })
    }

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

impl OwnedCombatSession {
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

    fn public_search(&self,research_seed:&Bound<'_,PyAny>,live:&Bound<'_,PyAny>,branches:&Bound<'_,PyAny>,per_branch:&Bound<'_,PyAny>,total:&Bound<'_,PyAny>,workers:&Bound<'_,PyAny>)->PyResult<NativePublicSearch>{
        let limits=search_limits(live,branches,per_branch,total)?;
        let _lease=self.begin()?;
        let root=self.session.lock().unwrap_or_else(|p|p.into_inner()).public_root().map_err(run_error)?;
        public_search(root,integer(research_seed)?,limits,workers)
    }
}

#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeIroncladCombatSession {
    inner: OwnedCombatSession,
}

#[pyclass(frozen, module = "spire_lab._native")]
pub(crate) struct NativeStandardCombatSession {
    inner: OwnedCombatSession,
}

macro_rules! single_combat_methods {
    ($class:ident, $standard:expr, $($constructor:tt)*) => {
        #[pymethods]
        impl $class {
            $($constructor)*

            fn observation_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                self.inner.observation_spec(py)
            }
            fn environment_spec<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                self.inner.environment_spec(py)
            }
            fn catalog_descriptor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                self.inner.catalog_descriptor(py)
            }
            #[pyo3(signature=(seed=None,entry_id=None))]
            fn reset<'py>(
                &self,
                py: Python<'py>,
                seed: Option<&Bound<'py, PyAny>>,
                entry_id: Option<&Bound<'py, PyAny>>,
            ) -> PyResult<Bound<'py, PyDict>> {
                self.inner.reset(py, seed, entry_id)
            }
            fn step<'py>(
                &self,
                py: Python<'py>,
                index: &Bound<'py, PyAny>,
                token_value: &Bound<'py, PyAny>,
            ) -> PyResult<Bound<'py, PyDict>> {
                self.inner.step(py, index, token_value)
            }
            fn decision<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
                self.inner.decision(py)
            }
            #[pyo3(signature=(research_seed,*,max_live_branches,max_branches,max_steps_per_branch,max_total_steps,workers=None))]
            #[allow(clippy::too_many_arguments)]
            fn public_search<'py>(&self,py:Python<'py>,research_seed:&Bound<'_,PyAny>,max_live_branches:&Bound<'_,PyAny>,max_branches:&Bound<'_,PyAny>,max_steps_per_branch:&Bound<'_,PyAny>,max_total_steps:&Bound<'_,PyAny>,workers:Option<&Bound<'py,PyAny>>)->PyResult<NativePublicSearch>{
                self.inner.public_search(research_seed,max_live_branches,max_branches,max_steps_per_branch,max_total_steps,&crate::run_env::one_worker(py,workers))
            }
            #[pyo3(signature=(package,*,seed=None,entry_id=None))]
            fn _reset_deliver<'py>(
                &self,
                py: Python<'py>,
                package: &Bound<'py, PyAny>,
                seed: Option<&Bound<'py, PyAny>>,
                entry_id: Option<&Bound<'py, PyAny>>,
            ) -> PyResult<Bound<'py, PyAny>> {
                self.inner._reset_deliver(py, package, seed, entry_id)
            }
            fn _step_deliver<'py>(
                &self,
                py: Python<'py>,
                index: &Bound<'py, PyAny>,
                token_value: &Bound<'py, PyAny>,
                package: &Bound<'py, PyAny>,
            ) -> PyResult<Bound<'py, PyAny>> {
                self.inner._step_deliver(py, index, token_value, package)
            }
            fn diagnostic<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                self.inner.diagnostic(py)
            }
            fn close(&self, py: Python<'_>) -> PyResult<()> {
                self.inner.close(py)
            }
        }
    };
}
single_combat_methods!(NativeIroncladCombatSession, false,
    #[new]
    #[pyo3(signature=(catalog_json,*,partition="train".to_owned()))]
    fn new(py: Python<'_>, catalog_json: &Bound<'_, PyAny>, #[pyo3(from_py_with = label)] partition: String) -> PyResult<Self> {
        Ok(Self { inner: OwnedCombatSession::new(py, catalog_json, partition, Vec::new(), false)? })
    }
);
single_combat_methods!(NativeStandardCombatSession, true,
    #[new]
    #[pyo3(signature=(catalog_json,*,records=None,partition="train".to_owned()))]
    fn new(py: Python<'_>, catalog_json: &Bound<'_, PyAny>, records: Option<&Bound<'_, PyAny>>, #[pyo3(from_py_with = label)] partition: String) -> PyResult<Self> {
        let records = records_bytes(records)?;
        Ok(Self { inner: OwnedCombatSession::new(py, catalog_json, partition, records, true)? })
    }
);

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
}
