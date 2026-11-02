use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spire_env::RunObservation;

pub fn observation_json(observation: &RunObservation) -> Value {
    json!({
        "groups": observation.groups.iter().map(|group| json!({
            "key": group.key,
            "shape": group.shape,
            "values": group.values,
            "known": group.known,
            "occupied": group.occupied,
        })).collect::<Vec<_>>(),
        "action_mask": observation.action_mask,
    })
}

pub fn observation_hash(observation: &RunObservation) -> String {
    hex(&Sha256::digest(serde_json::to_vec(&observation_json(observation)).unwrap_or_default()))
}

#[derive(Clone)]
pub struct TraceHasher(Sha256);

impl TraceHasher {
    pub fn new(config: &Value, seed: u64, observation: &RunObservation) -> Self {
        let mut hash = Sha256::new();
        update_json(&mut hash, &json!({"config":config,"seed":seed,"observation":observation_json(observation)}));
        Self(hash)
    }

    pub fn step(&mut self, index: usize, reward: i32, observation: &RunObservation) {
        update_json(&mut self.0, &json!({"action":index,"reward":reward,"observation":observation_json(observation)}));
    }

    pub fn snapshot(&self) -> String { hex(&self.0.clone().finalize()) }
    pub fn finish(self) -> String { hex(&self.0.finalize()) }
}

fn update_json(hash: &mut Sha256, value: &Value) {
    if let Ok(bytes) = serde_json::to_vec(value) {
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}
