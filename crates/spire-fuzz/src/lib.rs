//! Deterministic native fuzz drivers for the public Spire Lab Rust owners.

mod catalog;
mod driver;
mod failure;
mod observation;
mod policy;
mod profiles;
mod runner;
mod run_driver;

#[cfg(feature = "diagnostics")]
mod complete;
#[cfg(feature = "diagnostics")]
mod items_run;

pub use catalog::{generated_catalog_json, generated_standard_catalog_json, CatalogCombat};
pub use driver::{Decision, Driver, DriverError, IllegalProbe, StepOut};
pub use failure::{FuzzFailure, ReproCase, failure_key, failure_key_parts, read_repro, shrink_actions, write_failure};
pub use observation::{observation_json, observation_hash, TraceHasher};
pub use policy::{Policy, POLICIES, choose_action, mix64};
pub use profiles::{RunProfileSpec, RUN_PROFILES};
#[cfg(feature = "diagnostics")]
pub use profiles::{CompleteItemsRunProfile, COMPLETE_ITEMS_RUN_PROFILES, complete_items_profile};
pub use run_driver::FullRun;
pub use runner::{EpisodeOptions, EpisodeResult, OutcomeTotals, RunSummary, run_episode, run_repro, shrink_failure_case};
#[cfg(feature = "diagnostics")]
pub use runner::DrawKnowledgeStats;

#[cfg(feature = "diagnostics")]
pub use complete::{complete_cases_for_smoke, CompleteCombat};
#[cfg(feature = "diagnostics")]
pub use items_run::CompleteItemsRun;

pub const HARNESS_VERSION: &str = "spire-fuzz-v1";
