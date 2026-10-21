//! Strict once-only Rust parsing of the Shared setup schema and reset transport.
use pyo3::{prelude::*, types::{PyAny, PyBool, PyBytes, PyDict, PyInt, PyList, PyString}};
use spire_core::combat::{CardKind, EnemyKind};
use spire_env::{CardSpec, InputReason, IntegerSeed, PublicContext,
    ScenarioCatalog, ScenarioTemplate, SeedBatch};
use crate::errors::{input, native};

pub(crate) fn integer<T: for<'a, 'py> FromPyObject<'a, 'py>>(
    value: &Bound<'_, PyAny>, reason: InputReason, slot: Option<usize>,
) -> PyResult<T> {
    if !value.is_exact_instance_of::<PyInt>() {
        return Err(input(value.py(), reason, "expected a built-in integer", slot));
    }
    value.extract().map_err(|_|input(value.py(), reason, "integer out of range", slot))
}

pub(crate) fn boolean(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    if !value.is_exact_instance_of::<PyBool>() {
        return Err(input(value.py(), InputReason::Dtype, "expected bool", None));
    }
    value.extract().map_err(Into::into)
}

pub(crate) fn size(value: &Bound<'_, PyAny>) -> PyResult<usize> {
    integer(value, InputReason::Range, None)
}

pub(crate) fn label(value: &Bound<'_, PyAny>) -> PyResult<String> {
    string(value, None)
}

pub(crate) fn string(value: &Bound<'_, PyAny>, slot: Option<usize>) -> PyResult<String> {
    if !value.is_exact_instance_of::<PyString>() {
        return Err(input(value.py(), InputReason::Scenario, "expected string", slot));
    }
    value.extract().map_err(|_|input(value.py(), InputReason::Scenario, "expected UTF-8 string", slot))
}

fn list<'a, 'py>(value: &'a Bound<'py, PyAny>) -> PyResult<&'a Bound<'py, PyList>> {
    value.cast::<PyList>().map_err(|_|input(value.py(), InputReason::Shape, "expected list", None))
}
fn dict<'a, 'py>(value: &'a Bound<'py, PyAny>, allowed: &[&str], required: &[&str]) -> PyResult<&'a Bound<'py, PyDict>> {
    let dict = value.cast::<PyDict>().map_err(|_|input(value.py(), InputReason::Scenario, "expected dictionary", None))?;
    for key in dict.keys() {
        let key = string(&key, None)?;
        if !allowed.contains(&key.as_str()) { return Err(input(value.py(), InputReason::Scenario, "unknown setup key", None)); }
    }
    for key in required {
        if !dict.contains(key)? { return Err(input(value.py(), InputReason::Scenario, "missing setup field", None)); }
    }
    Ok(dict)
}
fn field<'py>(dict: &Bound<'py, PyDict>, key: &str) -> PyResult<Bound<'py, PyAny>> {
    dict.get_item(key)?.ok_or_else(||input(dict.py(), InputReason::Scenario, "missing setup field", None))
}
fn enemy(value: &Bound<'_, PyAny>) -> PyResult<EnemyKind> {
    match string(value, None)?.as_str() {
        "cultist" => Ok(EnemyKind::Cultist), "jaw_worm" => Ok(EnemyKind::JawWorm),
        _ => Err(input(value.py(), InputReason::Scenario, "unsupported enemy", None)),
    }
}
fn cards(value: &Bound<'_, PyAny>) -> PyResult<Vec<CardSpec>> {
    list(value)?.iter().map(|value| {
        let d = dict(&value, &["kind", "upgraded"], &["kind", "upgraded"])?;
        let kind = match string(&field(d, "kind")?, None)?.as_str() {
            "strike" => CardKind::Strike, "defend" => CardKind::Defend, "bash" => CardKind::Bash,
            _ => return Err(input(value.py(), InputReason::Scenario, "unsupported card", None)),
        };
        Ok(CardSpec { kind, upgraded:boolean(&field(d, "upgraded")?)? })
    }).collect()
}

pub(crate) fn catalog(value: Option<&Bound<'_, PyAny>>) -> PyResult<ScenarioCatalog> {
    let Some(value) = value.filter(|v|!v.is_none()) else { return Ok(ScenarioCatalog::starter()); };
    let fields = ["id", "encounter", "player_hp", "player_max_hp", "energy_per_turn",
        "draw_per_turn", "enemy_hp", "hand", "draw", "context"];
    let templates = list(value)?.iter().map(|value| {
        let d = dict(&value, &fields, &fields[..9])?;
        let mut context = PublicContext { act:None, next_encounter:None };
        if let Some(value) = d.get_item("context")? {
            let c = dict(&value, &["act", "next_encounter"], &[])?;
            if let Some(act) = c.get_item("act")?.filter(|v|!v.is_none()) {
                context.act = Some(integer(&act, InputReason::Range, None)?);
            }
            if let Some(next) = c.get_item("next_encounter")?.filter(|v|!v.is_none()) {
                context.next_encounter = Some(enemy(&next)?);
            }
        }
        let quantity = |key|integer(&field(d, key)?, InputReason::Range, None);
        Ok(ScenarioTemplate { id:string(&field(d, "id")?, None)?, encounter:enemy(&field(d, "encounter")?)?,
            player_hp:quantity("player_hp")?, player_max_hp:quantity("player_max_hp")?,
            energy_per_turn:quantity("energy_per_turn")?, draw_per_turn:quantity("draw_per_turn")?,
            enemy_hp:quantity("enemy_hp")?, hand:cards(&field(d, "hand")?)?,
            draw:cards(&field(d, "draw")?)?, context })
    }).collect::<PyResult<Vec<_>>>()?;
    ScenarioCatalog::new(templates).map_err(|e|native(value.py(), e))
}

fn seed(value: &Bound<'_, PyAny>, slot: Option<usize>) -> PyResult<IntegerSeed> {
    let py = value.py();
    if !value.is_exact_instance_of::<PyInt>() || value.lt(0)? {
        return Err(input(py, InputReason::Seed, "seed must be a nonnegative built-in int", slot));
    }
    // Only numeric marshalling in Python. Expansion and hashing are native.
    let bits: usize = value.call_method0("bit_length")?.extract()?;
    let bytes = bits.div_ceil(8).max(1);
    let buffer = value.call_method1("to_bytes", (bytes, "big"))?;
    let bytes = buffer.cast::<PyBytes>()?;
    IntegerSeed::from_be_bytes(bytes.as_bytes()).map_err(|e|native(py, e))
}

pub(crate) fn seeds(value: Option<&Bound<'_, PyAny>>, batch: usize) -> PyResult<SeedBatch> {
    let Some(value) = value.filter(|v|!v.is_none()) else { return Ok(SeedBatch::Continue); };
    if let Ok(values) = value.cast::<PyList>() {
        if values.len() != batch { return Err(input(value.py(), InputReason::Shape, "seed list must have B entries", None)); }
        let seeds = values.iter().enumerate().map(|(i, v)| {
            if v.is_none() { Ok(None) } else { seed(&v, Some(i)).map(Some) }
        }).collect::<PyResult<Vec<_>>>()?;
        Ok(SeedBatch::PerSlot(seeds))
    } else { Ok(SeedBatch::Scalar(seed(value, None)?)) }
}

pub(crate) fn indices(value: &Bound<'_, PyAny>, batch: usize) -> PyResult<Vec<usize>> {
    let mut result = Vec::new();
    for value in list(value)?.iter() {
        let index:usize = integer(&value, InputReason::Range, None)?;
        if index >= batch || result.contains(&index) {
            return Err(input(value.py(), InputReason::Range, "out-of-range or duplicate reset index", Some(index)));
        }
        result.push(index);
    }
    Ok(result)
}

pub(crate) fn scenarios(value: Option<&Bound<'_, PyAny>>, batch: usize, ids: &[String]) -> PyResult<Vec<Option<String>>> {
    let Some(value) = value.filter(|v|!v.is_none()) else { return Ok(vec![None;batch]); };
    let values = list(value)?;
    if values.len() != batch { return Err(input(value.py(), InputReason::Shape, "scenario_ids must have B entries", None)); }
    values.iter().enumerate().map(|(i,v)| {
        if v.is_none() { return Ok(None); }
        let id = string(&v, Some(i))?;
        if !ids.contains(&id) { return Err(input(value.py(), InputReason::Scenario, "unknown scenario ID", Some(i))); }
        Ok(Some(id))
    }).collect()
}
