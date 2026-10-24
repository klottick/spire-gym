//! Rust-native combat and captured Run adapters; source-grounded, not original-runtime validated.

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
mod run_schema;
mod run_observation;
mod run_session;
mod run_replay;

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
pub use run_schema::{RunCapacities,RunObservationSpec,RUN_OBSERVATION_SCHEMA_ID,run_observation_spec};
pub use run_observation::{RunFeatureGroup,RunObservation,encode_run};
pub use run_session::{RunSessionError,M9BossHistory,RunSessionConfig,RunDecisionToken,RunActionIndex,RunBoundary,RunStepResult,RunRecordedStep,RunRecordedFault,RunRecordError,RunRecordErrorKind,RunCapacityFailure,RunEpisodeRecord,RunReproduction,RunSession,RUN_RECORD_SCHEMA_ID,RUN_SEED_ADAPTER_ID};
pub use run_replay::{RunReplayReport,replay_run};
pub use replay::{ReplayVisit,ReplayVisitKind,replay_observed};

#[cfg(test)]
mod tests;
