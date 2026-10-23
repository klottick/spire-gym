//! Rust-native supplied combat setup, source-grounded, not original-runtime validated.

mod canonical;
mod episode;
mod error;
mod scenario;
mod seed;
mod actions;
mod knowledge;
mod observation;
mod schema;
mod identity;
mod slot;
mod pool;
mod workers;
mod diagnostics;
mod replay;

pub use canonical::canonical_json;
pub use episode::EpisodeJob;
pub use error::{EnvError, FaultStage, InputReason, SlotFault, StateReason};
pub use scenario::{CardSpec, PublicContext, ScenarioCatalog, ScenarioTemplate};
pub use seed::{IntegerSeed, SEED_ADAPTER_ID, SeedBatch, expand_seeds};
pub use observation::{NamedTensor, ObservationBatch, ObservationRow, TensorValues};
pub use schema::{ObservationSpec, OBSERVATION_SCHEMA_ID, observation_spec};
pub use identity::DecisionToken;
pub use slot::SlotStatus;
pub use pool::{AutoReset, CombatPool, DecisionRows, EpisodeInfo, PoolConfig, PoolStatus, ResetRequest, ResetRows, StepBatch};
pub use pool::PoolOperation;
pub use diagnostics::{PoolDiagnostics, SlotDiagnostic};
pub use replay::{EpisodeRecord, RecordStatus, ReplayBoundary, ReplayResult, RECORD_SCHEMA_ID, replay};
pub use replay::{ReplayVisit,ReplayVisitKind,replay_observed};

#[cfg(test)]
mod tests;
