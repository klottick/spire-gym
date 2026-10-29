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
mod run_schema_m10;
mod run_schema_m12;
mod run_schema_m13;
mod run_schema_m14;
mod run_schema_m15;
mod run_observation;
mod run_session;
mod run_replay;
mod run_batch;
mod combat_policy;
mod combat_training;
pub use combat_policy::{defect_combat_observation_spec,encode_defect_combat,RUN_OBSERVATION_POLICY_SCHEMA_ID,policy_run_observation_spec,encode_run_policy,COMBAT_POLICY_VIEW_ID,COMBAT_POLICY_VIEW_V2_ID,CombatPolicyBoundary,combat_policy_capacities,combat_policy_observation_spec,combat_policy_observation_spec_v2};
pub use combat_training::{CombatTrainingCatalog,CombatTrainingSession,CombatTrainingStep,combat_training_support,combat_training_support_v2,COMBAT_TRAINING_PROFILE_ID,COMBAT_TRAINING_PROFILE_V2_ID,COMBAT_TRAINING_CONTENT_ID,COMBAT_TRAINING_CONTENT_V2_ID,COMBAT_TRAINING_REWARD_ID};

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
pub use run_schema_m10::{m10_run_observation_spec,RUN_OBSERVATION_M10_SCHEMA_ID};
pub use run_schema_m12::{m12_run_observation_spec,RUN_OBSERVATION_M12_SCHEMA_ID};
pub use run_schema_m13::{m13_run_observation_spec,RUN_OBSERVATION_M13_SCHEMA_ID};
pub use run_schema_m15::{m15_run_observation_spec,RUN_OBSERVATION_M15_SCHEMA_ID};
pub use run_schema_m14::{m14_run_observation_spec,RUN_OBSERVATION_M14_SCHEMA_ID};
pub use run_observation::{RunFeatureGroup,RunObservation,encode_run,encode_run_m10,encode_run_m12,encode_run_m13,encode_run_m14,encode_run_m15};
pub use run_session::{RunSessionError,RunBossHistory,RunAct1BossTarget,RunAct2BossTarget,RunAct3BossTarget,M9BossHistory,RunSessionConfig,RunDecisionToken,RunActionIndex,RunBoundary,RunStepResult,RunRecordedStep,RunRecordedFault,RunRecordError,RunRecordErrorKind,RunCapacityFailure,RunEpisodeRecord,RunReproduction,RunSession,RUN_RECORD_SCHEMA_ID,RUN_RECORD_SCHEMA_ID_M10,RUN_RECORD_SCHEMA_ID_POLICY,RUN_RECORD_SCHEMA_ID_M12,RUN_RECORD_SCHEMA_ID_M13,RUN_RECORD_SCHEMA_ID_M14,RUN_RECORD_SCHEMA_ID_M15,RUN_RULES_ID,RUN_RULES_ID_V1,RUN_RULES_ID_M10,RUN_RULES_ID_M12,RUN_RULES_ID_M13,RUN_RULES_ID_M14,RUN_RULES_ID_M15,RUN_CONTENT_ID_M10,RUN_CONTENT_ID_M12,RUN_CONTENT_ID_M13,RUN_CONTENT_ID_M14,RUN_CONTENT_ID_M15,RUN_SEED_ADAPTER_ID};
pub use run_batch::{RunBatch,RunBatchError,RunBatchSlotFault};
pub use run_replay::{RunReplayReport,replay_run};
pub use replay::{ReplayVisit,ReplayVisitKind,replay_observed};

#[cfg(test)]
mod tests;

mod run_schema_m16;
pub use run_schema_m16::M16_DEFECT_COMBAT_OBSERVATION_SCHEMA_ID;

mod run_schema_m17;
pub use run_schema_m17::{m17_run_observation_spec,RUN_OBSERVATION_M17_SCHEMA_ID};
pub use run_session::{RUN_RECORD_SCHEMA_ID_M17,RUN_RULES_ID_M17,RUN_CONTENT_ID_M17};
pub use run_observation::encode_run_m17;

mod run_schema_m18;
pub use run_schema_m18::{m18_run_observation_spec,RUN_OBSERVATION_M18_SCHEMA_ID};
pub use run_observation::encode_run_m18;
pub use run_session::{RUN_RECORD_SCHEMA_ID_M18,RUN_RULES_ID_M18,RUN_CONTENT_ID_M18};

mod run_schema_m19;
pub use run_schema_m19::{m19_run_observation_spec,RUN_OBSERVATION_M19_SCHEMA_ID};
pub use run_observation::encode_run_m19;
pub use run_session::{RUN_RECORD_SCHEMA_ID_M19,RUN_RULES_ID_M19,RUN_CONTENT_ID_M19};
