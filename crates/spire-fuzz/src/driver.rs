use std::collections::BTreeMap;
use serde_json::Value;
use spire_env::RunObservation;
use crate::Policy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub observation: RunObservation,
    /// Transport token, deliberately separate from the public observation.
    pub token: [u64; 2],
    pub terminal: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepOut {
    pub decision: Decision,
    pub reward: i32,
    pub terminal: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriverError {
    pub kind: String,
    pub message: String,
}

impl DriverError {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self { kind: kind.into(), message: message.into() }
    }
    pub fn fault(message: impl Into<String>) -> Self { Self::new("fault", message) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IllegalProbe { AtLegalCount, NegativeIndex, StaleToken }

pub trait Driver: Send {
    fn class(&self) -> &'static str;
    fn describe(&self) -> Value;
    fn fresh(&self) -> Box<dyn Driver>;
    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError>;
    fn decision(&self) -> Result<Decision, DriverError>;
    fn step(&mut self, index: usize) -> Result<StepOut, DriverError>;
    fn choose_action(&self, decision: &Decision, policy: Policy, seed: u64, step: usize) -> Option<usize> {
        crate::policy::choose_action(&decision.observation, decision.token, policy, seed, step)
    }
    /// A policy-specific guard checked immediately after `choose_action` and before stepping.
    fn policy_guard(&self) -> Option<DriverError> { None }
    fn preflight(&self, index: usize) -> Result<(), DriverError>;
    fn probe_illegal(&mut self, probe: IllegalProbe) -> Result<(), DriverError>;
    fn descriptor_stable(&self) -> Result<(), DriverError>;
    /// Current descriptor identity captured at reset; sampled determinism replay
    /// includes this value alongside observations and actions.
    fn stability_identity(&self) -> Value { Value::Null }
    /// Whether this episode has entered a live combat boundary.
    fn reached_combat(&self) -> bool { false }
    fn diagnostic(&self) -> Value;
    fn labels(&self) -> BTreeMap<String, String>;
    /// Return the named public profile endpoint when the current decision has
    /// no legal actions by design, such as the accepted Act2Ready cutoff.
    fn profile_cutoff(&self) -> Option<&'static str> { None }
    /// One-step reductions of driver-owned setup data for a reproducing failure.
    fn shrink_candidates(&self) -> Vec<Box<dyn Driver>> { Vec::new() }

    /// Additional semantic replay lane for drivers that own a record API.
    fn verify_replay_contract(&self, _seed: u64, _actions: &[usize]) -> Result<(), DriverError> { Ok(()) }

    /// Diagnostics-only hooks. Their default bodies keep the default feature set free
    /// of gated accessors and complete-card APIs.
    fn validate(&self) -> Result<(), DriverError> { Ok(()) }
    fn all_legal_on_clones(&self) -> Result<(), DriverError> { Ok(()) }

    #[cfg(feature = "diagnostics")]
    fn inner_run(&self) -> Option<spire_core::run::Run> { None }

    /// Diagnostics-only exact draw-posterior check for Standard combat boundaries.
    /// `Ok(None)` means the public posterior is Unrepresentable; `Ok(Some(n))` is
    /// a representable posterior with `n` components.
    #[cfg(feature = "diagnostics")]
    fn draw_knowledge_check(&self) -> Option<Result<Option<usize>, DriverError>> { None }

    /// Per-episode counters for the public-search gate driver.
    #[cfg(feature = "diagnostics")]
    fn search_fuzz_stats(&self) -> Option<crate::SearchFuzzStats> { None }
}
