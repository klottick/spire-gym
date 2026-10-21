//! Owned, typed Python errors; no automatic lossy input coercions.
use pyo3::{create_exception, exceptions::{PyRuntimeError, PyValueError}, prelude::*, types::{PyDict, PyList}};
use spire_env::{EnvError, InputReason, SlotFault};

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

pub(crate) fn native(py: Python<'_>, error: EnvError) -> PyErr {
    let message = error.to_string();
    let (exception, slot, reason, records) = match error {
        EnvError::Input { slot, reason, .. } => {
            let reason = match reason { InputReason::IllegalAction => "illegal_action".into(),
                r => format!("{r:?}").to_lowercase() };
            (CombatInputError::new_err(message), slot, reason, vec![])
        },
        EnvError::State { slot, reason } =>
            (CombatStateError::new_err(message), slot, format!("{reason:?}").to_lowercase(), vec![]),
        EnvError::Setup { slot, .. } => (CombatSetupError::new_err(message), slot, "setup".into(), vec![]),
        EnvError::Fault { records } => (CombatFaultError::new_err(message),
            records.first().and_then(|f|f.slot), "fault".into(), records),
    };
    let attach = || -> PyResult<()> {
        exception.value(py).setattr("slot", slot)?;
        exception.value(py).setattr("reason", reason)?;
        exception.value(py).setattr("faults", faults(py, &records)?)?;
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
