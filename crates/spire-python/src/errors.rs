//! Owned, typed Python errors; no automatic lossy input coercions.
use pyo3::{create_exception, exceptions::{PyRuntimeError, PyValueError}, prelude::*, types::{PyDict, PyList, PyTuple}};
use spire_env::{EnvError, FaultStage, InputReason, PoolSlotStatus, SlotFault};

create_exception!(_native, CombatInputError, PyValueError);
create_exception!(_native, CombatStateError, PyRuntimeError);
create_exception!(_native, CombatSetupError, PyRuntimeError);
create_exception!(_native, CombatFaultError, PyRuntimeError);

pub(crate) fn input(py: Python<'_>, reason: InputReason, message: &str, slot: Option<usize>) -> PyErr {
    native(py, EnvError::Input { slot, reason, message:message.into() })
}

pub(crate) fn faults<'py>(py: Python<'py>, records: &[SlotFault]) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for record in records {
        let dict = PyDict::new(py);
        dict.set_item("slot", record.slot)?;
        dict.set_item("stage", format!("{:?}", record.stage).to_lowercase())?;
        dict.set_item("message", &record.message)?;
        if let Some(core) = &record.core_fault {
            let summary = PyDict::new(py);
            summary.set_item("kind", format!("{:?}", core.kind))?;
            summary.set_item("message", &core.message)?;
            summary.set_item("resolved_effects", core.resolved_effects)?;
            dict.set_item("core_fault", summary)?;
        } else { dict.set_item("core_fault", py.None())?; }
        list.append(dict)?;
    }
    Ok(list)
}

fn slot_statuses<'py>(py: Python<'py>, statuses: &[PoolSlotStatus]) -> PyResult<Bound<'py, PyTuple>> {
    let mut entries=Vec::with_capacity(statuses.len());
    for status in statuses {
        let dict=PyDict::new(py);
        dict.set_item("status",status.status)?;
        dict.set_item("stage",status.stage.map(FaultStage::label))?;
        dict.set_item("committed",status.committed)?;
        dict.set_item("recoverable",status.recoverable)?;
        dict.set_item("message",status.message.as_deref())?;
        entries.push(dict);
    }
    PyTuple::new(py,entries)
}

pub(crate) fn native(py: Python<'_>, error: EnvError) -> PyErr {
    let message = error.to_string();
    let (exception, slot, reason, records, statuses) = match error {
        EnvError::Input { slot, reason, .. } => {
            let reason = match reason { InputReason::IllegalAction => "illegal_action".into(),
                r => format!("{r:?}").to_lowercase() };
            (CombatInputError::new_err(message), slot, reason, vec![], vec![])
        },
        EnvError::State { slot, reason } =>
            (CombatStateError::new_err(message), slot, format!("{reason:?}").to_lowercase(), vec![], vec![]),
        EnvError::Setup { slot, .. } => (CombatSetupError::new_err(message), slot, "setup".into(), vec![], vec![]),
        EnvError::Fault { records, slots } => (CombatFaultError::new_err(message),
            records.first().and_then(|f|f.slot), "fault".into(), records, slots),
    };
    let attach = || -> PyResult<()> {
        exception.value(py).setattr("slot", slot)?;
        exception.value(py).setattr("reason", reason)?;
        exception.value(py).setattr("faults", faults(py, &records)?)?;
        exception.value(py).setattr("slots", slot_statuses(py, &statuses)?)?;
        Ok(())
    };
    match attach() { Ok(()) => exception, Err(error) => error }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("CombatInputError", py.get_type::<CombatInputError>())?;
    m.add("CombatStateError", py.get_type::<CombatStateError>())?;
    m.add("CombatSetupError", py.get_type::<CombatSetupError>())?;
    m.add("CombatFaultError", py.get_type::<CombatFaultError>())?;
    Ok(())
}
