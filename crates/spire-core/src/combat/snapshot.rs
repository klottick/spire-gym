//! Detached in-memory checkpoints, not a serialized wire protocol.
use super::{CONTENT_ID, RestoreError};
use super::engine::{validate_runtime, Combat};
use super::model::State;
use super::rng::RNG_CONTRACT_ID;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub(super) schema_version: u32,
    pub(super) content_id: String,
    pub(super) rng_contract_id: String,
    pub(super) state: State,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence { pub component: &'static str, pub left: String, pub right: String }

impl Combat {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot { schema_version: 2, content_id: CONTENT_ID.into(),
            rng_contract_id: RNG_CONTRACT_ID.into(), state: self.state.clone() }
    }

    pub fn restore(snapshot: &Snapshot) -> Result<Self, RestoreError> {
        if snapshot.schema_version != 2 || snapshot.content_id != CONTENT_ID
            || snapshot.rng_contract_id != RNG_CONTRACT_ID
            || snapshot.state.content_id != snapshot.content_id
            || snapshot.state.rng.contract_id != snapshot.rng_contract_id {
            return Err(RestoreError::Compatibility);
        }
        if snapshot.state.fault.is_some() { return Err(RestoreError::Faulted); }
        validate_runtime(&snapshot.state).map_err(RestoreError::Invalid)?;
        Ok(Self { state: snapshot.state.clone() })
    }
}

pub fn compare(left: &Snapshot, right: &Snapshot) -> Option<Divergence> {
    macro_rules! component {
        ($label:expr, $a:expr, $b:expr) => {
            if $a != $b {
                return Some(Divergence { component: $label, left: format!("{:?}", $a), right: format!("{:?}", $b) });
            }
        };
    }
    component!("content_id", left.content_id, right.content_id);
    component!("schema_version", left.schema_version, right.schema_version);
    component!("rng_contract_id", left.rng_contract_id, right.rng_contract_id);
    macro_rules! state_fields {
        ($($field:ident),+ $(,)?) => { $(component!(stringify!($field), left.state.$field, right.state.$field);)+ };
    }
    component!("state.content_id", left.state.content_id, right.state.content_id);
    state_fields!(phase, turn, player, enemies, piles, card_ids_issued, rng, pending, current,
        in_play, resolved_effects, public, outcome, fault, hp_before_healing, hp_after_cleanup);
    None
}
