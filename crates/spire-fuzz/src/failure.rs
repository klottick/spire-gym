use std::{fs, path::{Path, PathBuf}, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use crate::Policy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuzzFailure {
    pub driver: String,
    pub driver_config: Value,
    pub feature_set: String,
    pub seed: u64,
    pub policy: Policy,
    pub actions: Vec<usize>,
    pub step: usize,
    pub all_legal_every: usize,
    pub kind: String,
    pub message: String,
    pub diagnostic: Value,
    #[serde(default)]
    pub known_bead: Option<String>,
    #[serde(default = "one_occurrence")]
    pub occurrences: u64,
    #[serde(default)]
    pub example_seeds: Vec<u64>,
    #[serde(default)]
    pub repro_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReproCase {
    pub driver: String,
    pub driver_config: Value,
    #[serde(default = "default_feature_set")]
    pub feature_set: String,
    pub seed: u64,
    pub policy: Policy,
    pub actions: Vec<usize>,
    #[serde(default)]
    pub all_legal_every: usize,
    pub expected_kind: String,
    pub expected_message: String,
    pub step: usize,
    #[serde(default)]
    pub known_bead: Option<String>,
}

fn default_feature_set() -> String { "default".into() }
fn one_occurrence() -> u64 { 1 }

pub fn read_repro(path: impl AsRef<Path>) -> Result<ReproCase, String> {
    let bytes = fs::read(path.as_ref()).map_err(|error| format!("read repro: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse repro: {error}"))
}

pub fn shrink_actions<F>(actions: &[usize], failing_step: usize, expected_kind: &str, budget: Duration, mut reproduces: F) -> Vec<usize>
where F: FnMut(&[usize]) -> Option<String> {
    let mut current = actions[..actions.len().min(failing_step.saturating_add(1))].to_vec();
    let deadline = Instant::now() + budget;
    let mut chunk = (current.len() / 2).max(1);
    while chunk > 0 && !current.is_empty() && Instant::now() < deadline {
        let mut changed = false;
        let starts = (0..current.len()).step_by(chunk).collect::<Vec<_>>();
        for start in starts {
            if Instant::now() >= deadline { break; }
            let end = (start + chunk).min(current.len());
            let mut candidate = current[..start].to_vec();
            candidate.extend_from_slice(&current[end..]);
            if reproduces(&candidate).as_deref() == Some(expected_kind) {
                current = candidate;
                changed = true;
                break;
            }
        }
        if !changed { chunk /= 2; }
    }
    current
}

pub fn write_failure(out: impl AsRef<Path>, failure: &FuzzFailure) -> Result<PathBuf, String> {
    let mut hash = Sha256::new();
    hash.update(failure_key(failure).as_bytes());
    let id = hex(&hash.finalize()[..12]);
    let directory = out.as_ref().join("failures").join(&id);
    fs::create_dir_all(&directory).map_err(|error| format!("create failure directory: {error}"))?;
    let repro = ReproCase { driver: failure.driver.clone(), driver_config: failure.driver_config.clone(), feature_set: failure.feature_set.clone(), seed: failure.seed,
        policy: failure.policy, actions: failure.actions.clone(), all_legal_every: failure.all_legal_every, expected_kind: failure.kind.clone(),
        expected_message: failure.message.clone(), step: failure.step, known_bead: failure.known_bead.clone() };
    let path = directory.join("repro.json");
    let bytes = serde_json::to_vec_pretty(&repro).map_err(|error| format!("serialize repro: {error}"))?;
    fs::write(&path, bytes).map_err(|error| format!("write repro: {error}"))?;
    let command = if failure.feature_set == "diagnostics" {
        format!("cargo run --release -p spire-fuzz --features diagnostics --example native_fuzz -- repro {}", path.display())
    } else {
        format!("cargo run --release -p spire-fuzz --example native_fuzz -- repro {}", path.display())
    };
    let tail = failure.actions.iter().rev().take(8).copied().collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>();
    let known_bead = failure.known_bead.as_deref().map(|bead| format!("- Known bead: `{bead}`\n")).unwrap_or_default();
    let beads = format!("# Native fuzz failure {id}\n\n- Title: {} {}\n- Cause guess: {}\n- Driver: {}\n- Seed: {}\n- Occurrences: {}\n- Example seeds: {:?}\n- Policy: {}\n- Step: {}\n- Kind: {}\n- Message: {}\n{}- Last actions: {:?}\n- Repro: `{}`\n- Labels: `native-fault,fuzz`\n",
        failure.driver, failure.kind, cause_guess(&failure.kind, &failure.message), failure.driver, failure.seed,
        failure.occurrences, failure.example_seeds, failure.policy.name(), failure.step, failure.kind, failure.message, known_bead, tail, command);
    fs::write(directory.join("beads.md"), beads).map_err(|error| format!("write beads.md: {error}"))?;
    Ok(path)
}

fn cause_guess(kind: &str, message: &str) -> String {
    match kind {
        "fault" => format!("advertised operation returned an error: {message}"),
        "validator" => format!("retained state failed a read-only validator: {message}"),
        "legal_rejected" => format!("an advertised action was rejected: {message}"),
        "illegal_accepted" | "probe_accepted" => "an unadvertised input was accepted".into(),
        "illegal_corrupts" | "probe_corrupts" => "an invalid input changed the published state".into(),
        "encode" => format!("public observation or descriptor encoding failed: {message}"),
        "determinism" => "same seed and action trace produced different public output".into(),
        "replay" => format!("semantic replay did not reproduce: {message}"),
        _ => message.into(),
    }
}

fn normalize_message(message: &str) -> String {
    let (primary, diagnostic) = message.split_once("; diagnostic=").map_or((message, None), |(primary, diagnostic)| (primary, Some(diagnostic)));
    if let (Some(fault_kind), Some(fault_message)) = (record_fault_kind(primary), record_fault_message(primary)) {
        let current = section_after(primary, "current: ").unwrap_or_default().split("pending:").next().unwrap_or_default();
        let pending = section_after(primary, "pending: ").unwrap_or_default()
            .split(", attempted_action:").next().unwrap_or_default();
        return format!("fault_kind={fault_kind}; message={fault_message}; current={}; pending={}",
            effect_kinds(current), effect_kinds(pending));
    }
    let mut normalized = normalize_dynamic_tokens(primary);
    if let Some(diagnostic) = diagnostic.and_then(|value| serde_json::from_str::<Value>(value).ok()) {
        let nested_message = diagnostic.pointer("/record/fault/error/message").and_then(Value::as_str)
            .or_else(|| diagnostic.pointer("/record/fault/error/Debug/message").and_then(Value::as_str));
        if let Some(nested_message) = nested_message {
            normalized.push_str("; error=");
            normalized.push_str(&normalize_dynamic_tokens(nested_message));
        }
    }
    normalized
}

/// Stable identity for grouping equivalent failures independently of seed and setup IDs.
pub fn failure_key(failure: &FuzzFailure) -> String {
    failure_key_for_config(&failure.driver, &failure.driver_config, &failure.diagnostic,
        &failure.kind, &failure.message)
}

pub fn failure_key_parts(driver: &str, kind: &str, message: &str) -> String {
    format!("{driver}\0{kind}\0{}", normalize_message(message))
}

pub(crate) fn failure_key_for_config(driver: &str, config: &Value, diagnostic: &Value, kind: &str, message: &str) -> String {
    let field = |name: &str| config.get(name).and_then(Value::as_str)
        .or_else(|| diagnostic.get(name).and_then(Value::as_str)).unwrap_or("");
    format!("{driver}\0{}\0{}\0{kind}\0{}", field("profile"), field("character"), normalize_message(message))
}

// Historical note: the run-1 encoder-vocabulary family was fixed by spire-85d / PR71.
// Do not classify post-PR71 encoder failures as known; they are new findings.

fn record_fault_kind(message: &str) -> Option<&str> {
    let rest = section_after(message, "FaultRecord { kind: ")?;
    let end = rest.find(',')?;
    Some(rest[..end].trim())
}

fn record_fault_message(message: &str) -> Option<String> {
    let rest = section_after(message, "FaultRecord {")?;
    let rest = section_after(rest, "message: \"")?;
    let mut value = String::new();
    let mut escaped = false;
    for ch in rest.chars() {
        if escaped { value.push(ch); escaped = false; continue; }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(value),
            _ => value.push(ch),
        }
    }
    None
}

fn section_after<'a>(value: &'a str, marker: &str) -> Option<&'a str> {
    value.find(marker).map(|index| &value[index + marker.len()..])
}

fn effect_kinds(section: &str) -> String {
    let mut kinds = Vec::new();
    let mut rest = section;
    while let Some(next) = section_after(rest, "kind: ") {
        let end = next.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')).unwrap_or(next.len());
        if end > 0 { kinds.push(&next[..end]); }
        rest = &next[end..];
        if rest.is_empty() { break; }
    }
    kinds.join(",")
}

fn normalize_dynamic_tokens(message: &str) -> String {
    let chars = message.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '0' && chars.get(index + 1) == Some(&'x') {
            output.push_str("<hex>");
            index += 2;
            while index < chars.len() && chars[index].is_ascii_hexdigit() { index += 1; }
        } else if chars[index].is_ascii_digit() {
            output.push_str("<n>");
            while index < chars.len() && chars[index].is_ascii_digit() { index += 1; }
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }
    let mut output = output.split_whitespace().collect::<Vec<_>>().join(" ");
    for prefix in ["seed=", "entry=", "encounter="] {
        output = strip_labelled_id(&output, prefix);
    }
    output
}

fn strip_labelled_id(value: &str, prefix: &str) -> String {
    let mut output = String::new();
    let mut rest = value;
    while let Some(index) = rest.find(prefix) {
        output.push_str(&rest[..index + prefix.len()]);
        output.push_str("<id>");
        rest = &rest[index + prefix.len()..];
        let end = rest.find(|ch: char| ch.is_whitespace() || matches!(ch, ',' | ')' | '}' | ']')).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    output.push_str(rest);
    output
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|value| format!("{value:02x}")).collect() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_identity_normalizes_numbers_but_keeps_effect_kinds() {
        let base = "Faulted(RunFault { record: FaultRecord { kind: Invariant, message: \"decision before quiescence\", resolved_effects: 14 }, current: Some(Effect { kind: Decision }), pending: [Effect { kind: GainBlock(10) }] })";
        let changed_numbers = base.replace("resolved_effects: 14", "resolved_effects: 29").replace("GainBlock(10)", "GainBlock(6)");
        let other_effect = base.replace("GainBlock(10)", "Draw(2)");
        assert_eq!(failure_key_parts("complete", "fault", base), failure_key_parts("complete", "fault", &changed_numbers));
        assert_ne!(failure_key_parts("complete", "fault", base), failure_key_parts("complete", "fault", &other_effect));
        assert_ne!(failure_key_parts("complete", "fault", base), failure_key_parts("catalog", "fault", base));
        assert_ne!(failure_key_parts("complete", "fault", base), failure_key_parts("complete", "validator", base));
    }

    #[test]
    fn failure_identity_keeps_nested_delivery_cause_without_run_ids() {
        let message = |index: u32, cause: &str| format!(
            "step index={index}: DeliveryFault(\"post-step Run encoding failed\"); diagnostic={{\"record\":{{\"fault\":{{\"error\":{{\"message\":\"{cause}\"}}}}}}}}"
        );
        let first = message(7, "unsupported public power or intent");
        let same = message(23, "unsupported public power or intent");
        let different = message(23, "unsupported public intent");
        assert_eq!(failure_key_parts("run", "fault", &first), failure_key_parts("run", "fault", &same));
        assert_ne!(failure_key_parts("run", "fault", &first), failure_key_parts("run", "fault", &different));
    }

    #[test]
    fn failure_identity_keeps_run_profile_and_character() {
        let message = "step index=7: unsupported public power";
        let config = |profile: &str, character: &str| serde_json::json!({"profile":profile,"character":character});
        assert_ne!(failure_key_for_config("run", &config("M22", "watcher"), &Value::Null, "fault", message),
            failure_key_for_config("run", &config("M23", "watcher"), &Value::Null, "fault", message));
        assert_ne!(failure_key_for_config("run", &config("M22", "watcher"), &Value::Null, "fault", message),
            failure_key_for_config("run", &config("M22", "ironclad"), &Value::Null, "fault", message));
    }

}
