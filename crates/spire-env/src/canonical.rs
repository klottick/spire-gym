//! Setup/export-only canonical JSON; never used in the stepping path.
use crate::{EnvError, InputReason};
use crate::error::{input, setup};
use serde_json::Value;

/// Recursively sorted UTF-8 JSON, preserving array order and explicit defaults/nulls.
/// Only integer JSON numbers are accepted, including the full signed/unsigned range.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>, EnvError> {
    fn normalize(value: &Value) -> Result<(), EnvError> {
        match value {
            Value::Number(n) if !n.is_i64() && !n.is_u64() =>
                Err(input(InputReason::Range, "canonical JSON requires integer numbers")),
            Value::Array(a) => a.iter().try_for_each(normalize),
            Value::Object(o) => o.values().try_for_each(normalize),
            _ => Ok(()),
        }
    }
    normalize(value)?;
    let mut sorted = value.clone();
    // Explicit even if serde_json currently uses BTreeMap; feature unification may
    // enable preserve_order in another consumer of this crate.
    sorted.sort_all_objects();
    serde_json::to_vec(&sorted).map_err(|e| setup(format!("canonical JSON export failed: {e}")))
}
