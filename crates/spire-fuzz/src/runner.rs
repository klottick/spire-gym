use std::{collections::BTreeMap, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spire_env::RunObservation;
use crate::{CatalogCombat, Decision, Driver, DriverError, FuzzFailure, FullRun, IllegalProbe, Policy, ReproCase, TraceHasher,
    profiles::profile, shrink_actions};
use crate::failure::{failure_key, failure_key_for_config};

#[cfg(feature = "diagnostics")]
use crate::CompleteCombat;
#[cfg(feature = "diagnostics")]
use crate::{CompleteItemsRun, profiles::complete_items_profile};
#[cfg(feature = "diagnostics")]
use spire_core::{content::{Character, EncounterId, PotionId, RelicId}, run::{AuthoredRunStart, CardInstance, TRAINING_ENCOUNTERS_V2, TRAINING_POTIONS, TRAINING_RELICS}};
#[cfg(feature = "diagnostics")]
use spire_core::run::diagnostics::CompleteItemSupply;
#[cfg(feature = "diagnostics")]
use spire_core::run::Run;

#[derive(Clone, Debug)]
pub struct EpisodeOptions {
    pub max_steps: usize,
    pub all_legal_every: usize,
    pub determinism: bool,
    pub replay: bool,
    pub shrink_budget: Duration,
    /// Fixed action prefix used by repro and tests. When it ends, `continue_first`
    /// controls whether deterministic first-legal continuation is used.
    pub scripted_actions: Option<Vec<usize>>,
    pub continue_first: bool,
}

impl Default for EpisodeOptions {
    fn default() -> Self { Self { max_steps: 4096, all_legal_every: 0, determinism: false, replay: true,
        shrink_budget: Duration::from_secs(60), scripted_actions: None, continue_first: false } }
}

#[derive(Clone, Debug)]
pub struct EpisodeResult {
    pub driver: String,
    pub labels: BTreeMap<String, String>,
    pub policy: Policy,
    pub seed: u64,
    pub trace_hash: String,
    pub steps: u64,
    pub wins: u64,
    pub deaths: u64,
    pub escapes: u64,
    pub reached_combat: bool,
    pub all_legal_checks_after_step_zero: u64,
    pub capped: u64,
    pub capped_reason: Option<String>,
    pub profile_cutoff: Option<String>,
    pub unsupported: u64,
    pub failure: Option<FuzzFailure>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CappedExample {
    pub seed: u64,
    pub labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OutcomeTotals {
    pub episodes: u64,
    pub steps: u64,
    pub wins: u64,
    pub deaths: u64,
    pub escapes: u64,
    pub capped: u64,
    pub profile_cutoffs: u64,
    pub unsupported: u64,
    pub combat_episodes: u64,
    pub all_legal_checks_after_step_zero: u64,
}

impl OutcomeTotals {
    fn add(&mut self, result: &EpisodeResult) {
        self.episodes += 1;
        self.steps += result.steps;
        self.wins += result.wins;
        self.deaths += result.deaths;
        self.escapes += result.escapes;
        self.combat_episodes += u64::from(result.reached_combat);
        self.all_legal_checks_after_step_zero += result.all_legal_checks_after_step_zero;
        self.capped += result.capped;
        self.profile_cutoffs += u64::from(result.profile_cutoff.is_some());
        self.unsupported += result.unsupported;
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub feature_set: String,
    pub revision: String,
    pub fingerprint_stability_scope: String,
    pub driver_descriptions: BTreeMap<String, Value>,
    pub weights: BTreeMap<String, u32>,
    pub combat_max_steps: usize,
    pub run_max_steps: usize,
    pub all_legal_every: usize,
    pub determinism_every: u64,
    pub replay_every: u64,
    pub elapsed_secs: f64,
    pub episodes: u64,
    pub steps: u64,
    pub wins: u64,
    pub deaths: u64,
    pub escapes: u64,
    pub capped: u64,
    pub capped_by_reason: BTreeMap<String, u64>,
    pub capped_by_driver: BTreeMap<String, u64>,
    pub capped_by_policy: BTreeMap<String, u64>,
    pub capped_by_profile: BTreeMap<String, u64>,
    pub capped_by_driver_policy_profile: BTreeMap<String, u64>,
    pub capped_example_seeds: BTreeMap<String, Vec<u64>>,
    pub capped_examples: BTreeMap<String, Vec<CappedExample>>,
    pub profile_cutoffs: u64,
    pub profile_cutoffs_by_kind: BTreeMap<String, u64>,
    pub profile_cutoffs_by_profile: BTreeMap<String, u64>,
    pub unsupported: u64,
    pub failures_by_known_bead: BTreeMap<String, u64>,
    pub episodes_by_driver: BTreeMap<String, u64>,
    pub episodes_by_profile: BTreeMap<String, u64>,
    pub episodes_by_character: BTreeMap<String, u64>,
    pub episodes_by_encounter: BTreeMap<String, u64>,
    pub episodes_by_entry: BTreeMap<String, u64>,
    pub episodes_by_policy: BTreeMap<String, u64>,
    pub outcomes_by_driver: BTreeMap<String, OutcomeTotals>,
    pub outcomes_by_profile: BTreeMap<String, OutcomeTotals>,
    pub outcomes_by_character: BTreeMap<String, OutcomeTotals>,
    pub outcomes_by_encounter: BTreeMap<String, OutcomeTotals>,
    pub outcomes_by_policy: BTreeMap<String, OutcomeTotals>,
    pub unsupported_encounters: BTreeMap<String, u64>,
    pub failures: Vec<FuzzFailure>,
    pub trace_hashes: Vec<String>,
}

impl RunSummary {
    pub fn new(revision: String) -> Self {
        let mut summary = Self { feature_set: if cfg!(feature = "diagnostics") { "diagnostics" } else { "default" }.into(), revision,
            fingerprint_stability_scope: "stable_within_run_and_identical_across_determinism_rerun".into(), ..Self::default() };
        summary.driver_descriptions.insert("catalog".into(), json!({"driver":"CatalogCombat","source":"generated or supplied v2 catalog"}));
        summary.driver_descriptions.insert("run".into(), json!({"driver":"FullRun","profiles":"accepted finite Run profile table"}));
        #[cfg(feature = "diagnostics")]
        {
            summary.driver_descriptions.insert("complete".into(), json!({"driver":"CompleteCombat","scope":"one constructed combat",
                "encode":"harness projection (spire-fuzz-complete-public-v1)","fingerprint":"n/a"}));
            summary.driver_descriptions.insert("items".into(), json!({"driver":"CompleteItemsRun","profile":"M27 complete items",
                "starts_at":"Act4 map","neow":false,"encode":"n/a"}));
        }
        summary
    }
    pub fn add(&mut self, result: EpisodeResult) {
        self.episodes += 1; self.steps += result.steps; self.wins += result.wins; self.deaths += result.deaths; self.escapes += result.escapes;
        self.capped += result.capped; self.unsupported += result.unsupported;
        self.outcomes_by_driver.entry(result.driver.clone()).or_default().add(&result);
        self.outcomes_by_profile.entry(result.labels.get("profile").cloned().unwrap_or_else(|| "unprofiled".into())).or_default().add(&result);
        self.outcomes_by_character.entry(result.labels.get("character").cloned().unwrap_or_else(|| "unprofiled".into())).or_default().add(&result);
        self.outcomes_by_encounter.entry(result.labels.get("encounter").cloned().unwrap_or_else(|| "unprofiled".into())).or_default().add(&result);
        self.outcomes_by_policy.entry(result.policy.name().into()).or_default().add(&result);
        if result.capped > 0 {
            bump(&mut self.capped_by_reason, result.capped_reason.as_deref().unwrap_or("unspecified"));
            bump(&mut self.capped_by_driver, &result.driver);
            bump(&mut self.capped_by_policy, result.policy.name());
            let profile = result.labels.get("profile").map(String::as_str).unwrap_or("unprofiled");
            bump(&mut self.capped_by_profile, profile);
            let key = format!("{}|{}|{}", result.driver, result.policy.name(), profile);
            bump(&mut self.capped_by_driver_policy_profile, &key);
            let examples = self.capped_example_seeds.entry(key).or_default();
            if examples.len() < 3 && !examples.contains(&result.seed) { examples.push(result.seed); }
            let examples = self.capped_examples.entry(format!("{}|{}|{}", result.driver, result.policy.name(), profile)).or_default();
            if examples.len() < 3 && !examples.iter().any(|example| example.seed == result.seed) {
                examples.push(CappedExample { seed: result.seed, labels: result.labels.clone() });
            }
        }
        if let Some(kind) = result.profile_cutoff.as_deref() {
            self.profile_cutoffs += 1;
            bump(&mut self.profile_cutoffs_by_kind, kind);
            bump(&mut self.profile_cutoffs_by_profile, result.labels.get("profile").map(String::as_str).unwrap_or("unprofiled"));
        }
        bump(&mut self.episodes_by_driver, &result.driver);
        bump(&mut self.episodes_by_policy, result.policy.name());
        for (key, value) in &result.labels {
            match key.as_str() {
                "profile" => bump(&mut self.episodes_by_profile, value),
                "character" => bump(&mut self.episodes_by_character, value),
                "encounter" => bump(&mut self.episodes_by_encounter, value),
                "entry" => bump(&mut self.episodes_by_entry, value),
                _ => {}
            }
        }
        if result.unsupported > 0 {
            if let Some(encounter) = result.labels.get("encounter") { bump(&mut self.unsupported_encounters, encounter); }
        }
        if !result.trace_hash.is_empty() { self.trace_hashes.push(result.trace_hash); }
        if let Some(mut failure) = result.failure {
            if failure.occurrences == 0 { failure.occurrences = 1; }
            if failure.example_seeds.is_empty() { failure.example_seeds.push(failure.seed); }
            if let Some(bead) = failure.known_bead.as_deref() {
                *self.failures_by_known_bead.entry(bead.into()).or_default() += failure.occurrences;
            }
            if let Some(existing) = self.failures.iter_mut().find(|existing| failure_key(existing) == failure_key(&failure)) {
                existing.occurrences += failure.occurrences;
                for seed in failure.example_seeds {
                    if existing.example_seeds.len() == 3 { break; }
                    if !existing.example_seeds.contains(&seed) { existing.example_seeds.push(seed); }
                }
            } else {
                failure.example_seeds.truncate(3);
                self.failures.push(failure);
            }
        }
    }
    pub fn stable_hash(&self) -> String {
        let failures = self.failures.iter().map(|failure| json!({"driver":failure.driver,"seed":failure.seed,
            "driver_config":failure.driver_config,"feature_set":failure.feature_set,"policy":failure.policy.name(),
            "actions":failure.actions,"step":failure.step,"all_legal_every":failure.all_legal_every,
            "kind":failure.kind,"message":failure.message,"occurrences":failure.occurrences,
            "known_bead":failure.known_bead,"example_seeds":failure.example_seeds})).collect::<Vec<_>>();
        let stable = json!({"feature_set":self.feature_set,"fingerprint_stability_scope":self.fingerprint_stability_scope,
            "driver_descriptions":self.driver_descriptions,
            "episodes":self.episodes,"steps":self.steps,"wins":self.wins,
            "weights":self.weights,"combat_max_steps":self.combat_max_steps,"run_max_steps":self.run_max_steps,
            "all_legal_every":self.all_legal_every,"determinism_every":self.determinism_every,"replay_every":self.replay_every,
            "deaths":self.deaths,"escapes":self.escapes,"capped":self.capped,"profile_cutoffs":self.profile_cutoffs,
            "profile_cutoffs_by_kind":self.profile_cutoffs_by_kind,"profile_cutoffs_by_profile":self.profile_cutoffs_by_profile,
            "unsupported":self.unsupported,"failures_by_known_bead":self.failures_by_known_bead,"episodes_by_driver":self.episodes_by_driver,
            "capped_by_reason":self.capped_by_reason,"capped_by_driver":self.capped_by_driver,
            "capped_by_policy":self.capped_by_policy,"capped_by_profile":self.capped_by_profile,
            "capped_by_driver_policy_profile":self.capped_by_driver_policy_profile,"capped_example_seeds":self.capped_example_seeds,"capped_examples":self.capped_examples,
            "episodes_by_profile":self.episodes_by_profile,"episodes_by_character":self.episodes_by_character,
            "episodes_by_encounter":self.episodes_by_encounter,"episodes_by_entry":self.episodes_by_entry,
            "episodes_by_policy":self.episodes_by_policy,
            "outcomes_by_driver":self.outcomes_by_driver,"outcomes_by_profile":self.outcomes_by_profile,
            "outcomes_by_character":self.outcomes_by_character,"outcomes_by_encounter":self.outcomes_by_encounter,
            "outcomes_by_policy":self.outcomes_by_policy,
            "unsupported_encounters":self.unsupported_encounters,"failures":failures,"trace_hashes":self.trace_hashes});
        let bytes = serde_json::to_vec(&stable).unwrap_or_default();
        hex(&Sha256::digest(bytes))
    }
}

pub fn run_episode(driver: Box<dyn Driver>, seed: u64, policy: Policy, options: EpisodeOptions) -> EpisodeResult {
    run_episode_inner(driver, seed, policy, options, true)
}

fn run_episode_inner(mut driver: Box<dyn Driver>, seed: u64, policy: Policy, options: EpisodeOptions, allow_shrink: bool) -> EpisodeResult {
    let driver_class = driver.class().to_owned();
    let mut result = EpisodeResult { driver: driver_class.clone(), labels: BTreeMap::new(), policy, seed, trace_hash: String::new(),
        steps: 0, wins: 0, deaths: 0, escapes: 0, reached_combat: false, all_legal_checks_after_step_zero: 0,
        capped: 0, capped_reason: None, profile_cutoff: None,
        unsupported: 0, failure: None };
    let mut actions = Vec::<usize>::new();
    let mut prefix_hashes = Vec::<String>::new();
    #[cfg(feature = "diagnostics")]
    let prefix_target = (crate::mix64(seed ^ 0xE703_7ED1_A0B4_28DB) as usize) % options.max_steps.saturating_add(1);
    #[cfg(feature = "diagnostics")]
    let mut prefix_run = None;
    #[cfg(feature = "diagnostics")]
    let mut last_run = None;
    let mut trace: Option<TraceHasher> = None;
    let mut failure: Option<DriverError> = None;
    let mut failure_step = 0usize;
    let mut decision: Decision;

    match driver.reset(seed) {
        Ok(boundary) => {
            result.labels = driver.labels();
            trace = Some(TraceHasher::new(&trace_identity(driver.as_ref()), seed, &boundary.observation));
            prefix_hashes.push(trace.as_ref().unwrap().snapshot());
            decision = boundary;
            #[cfg(feature = "diagnostics")]
            {
                last_run = driver.inner_run();
                if prefix_target == 0 { prefix_run = last_run.clone(); }
            }
        }
        Err(error) if error.kind == "unsupported" => {
            result.labels = driver.labels(); result.unsupported = 1; return result;
        }
        Err(error) => {
            result.labels = driver.labels(); failure = Some(error); failure_step = 0; decision = empty_decision();
        }
    }

    if failure.is_none() {
        if let Err(error) = driver.validate() { failure = Some(error); failure_step = result.steps as usize; }
        if failure.is_none() { if let Err(error) = driver.descriptor_stable() { failure = Some(error); failure_step = result.steps as usize; } }
    }

    let mut probe_ordinal = 0usize;
    while failure.is_none() {
        if let Some(terminal) = decision_terminal(&decision) {
            count_terminal(terminal, &mut result);
            break;
        }
        let legal = legal_indices(&decision);
        if legal.is_empty() {
            if decision_terminal(&decision).is_none() {
                if let Some(kind) = driver.profile_cutoff() {
                    result.profile_cutoff = Some(kind.into());
                } else {
                    failure = Some(DriverError::new("stuck", "non-terminal decision publishes zero legal actions"));
                    failure_step = result.steps as usize;
                }
            }
            break;
        }
        if result.steps as usize >= options.max_steps {
            result.capped = 1;
            result.capped_reason = Some("max_steps".into());
            break;
        }

        if let Err(error) = driver.validate() { failure = Some(error); failure_step = result.steps as usize; break; }
        if let Err(error) = driver.descriptor_stable() { failure = Some(error); failure_step = result.steps as usize; break; }
        for index in legal.iter().copied() {
            if let Err(error) = driver.preflight(index) { failure = Some(error); failure_step = result.steps as usize; break; }
        }
        if failure.is_some() { break; }
        if options.all_legal_every > 0 && result.steps as usize % options.all_legal_every == 0 {
            if result.steps > 0 { result.all_legal_checks_after_step_zero += 1; }
            if let Err(error) = driver.all_legal_on_clones() { failure = Some(error); failure_step = result.steps as usize; break; }
        }

        let probe = match probe_ordinal % 3 {
            0 => IllegalProbe::AtLegalCount,
            1 => IllegalProbe::NegativeIndex,
            _ => IllegalProbe::StaleToken,
        };
        probe_ordinal += 1;
        match check_illegal_probe(driver.as_mut(), probe) {
            Ok(()) => {}
            Err(error) => { failure = Some(error); failure_step = result.steps as usize; break; }
        }

        let policy_selected = options.scripted_actions.is_none();
        let index = if let Some(scripted) = options.scripted_actions.as_ref() {
            if let Some(index) = scripted.get(result.steps as usize) { *index }
            else if options.continue_first { legal[0] }
            else { break; }
        } else {
            driver.choose_action(&decision, policy, seed, result.steps as usize).unwrap_or(legal[0])
        };
        if policy_selected {
            if let Some(error) = driver.policy_guard() {
                failure = Some(error);
                failure_step = result.steps as usize;
                break;
            }
        }
        if !legal.contains(&index) {
            failure = Some(DriverError::new("fault", format!("selected action index {index} is not advertised among {legal:?}")));
            failure_step = result.steps as usize;
            break;
        }
        actions.push(index);
        match driver.step(index) {
            Ok(output) => {
                result.steps += 1;
                if let Some(trace) = trace.as_mut() { trace.step(index, output.reward, &output.decision.observation); }
                prefix_hashes.push(trace.as_ref().unwrap().snapshot());
                if let Some(terminal) = &output.terminal { count_terminal(terminal, &mut result); }
                let terminal = output.terminal.is_some() || output.decision.terminal.is_some();
                decision = output.decision;
                #[cfg(feature = "diagnostics")]
                {
                    last_run = driver.inner_run();
                    if result.steps as usize == prefix_target { prefix_run = last_run.clone(); }
                }
                if let Err(error) = driver.validate() { failure = Some(error); failure_step = result.steps.saturating_sub(1) as usize; break; }
                if let Err(error) = driver.descriptor_stable() { failure = Some(error); failure_step = result.steps.saturating_sub(1) as usize; break; }
                if terminal { break; }
            }
            Err(error) if error.kind == "profile_cutoff" => {
                result.profile_cutoff = Some(error.message);
                break;
            }
            Err(error) if error.kind == "unsupported" => {
                result.unsupported = 1;
                break;
            }
            Err(error) => { result.steps += 1; failure = Some(error); failure_step = result.steps.saturating_sub(1) as usize; break; }
        }
    }

    if let Some(hasher) = trace.take() { result.trace_hash = hasher.finish(); }

    if failure.is_none() && result.unsupported == 0 && options.replay {
        #[cfg(feature = "diagnostics")]
        let prefix = prefix_target.min(actions.len());
        #[cfg(not(feature = "diagnostics"))]
        let prefix = (crate::mix64(seed ^ 0xE703_7ED1_A0B4_28DB) as usize) % (actions.len() + 1);
        #[cfg(feature = "diagnostics")]
        let expected_run = if prefix == prefix_target { prefix_run.as_ref() } else { last_run.as_ref() };
        if let Err(error) = verify_random_prefix(driver.as_ref(), seed, &actions, &prefix_hashes, prefix,
            #[cfg(feature = "diagnostics")]
            expected_run,
        ) { failure = Some(error); failure_step = actions.len().saturating_sub(1); }
        if failure.is_none() { if let Err(error) = driver.verify_replay_contract(seed, &actions) { failure = Some(error); failure_step = actions.len().saturating_sub(1); } }
    }
    if failure.is_none() && result.unsupported == 0 && options.determinism {
        match replay_actions(driver.as_ref(), seed, &actions, policy, options.max_steps) {
            Ok(hash) if hash == result.trace_hash => {}
            Ok(hash) => { failure = Some(DriverError::new("determinism", format!("trace hash mismatch: {} != {}", hash, result.trace_hash))); failure_step = actions.len().saturating_sub(1); },
            Err(error) => { failure = Some(DriverError::new("determinism", error.message)); failure_step = actions.len().saturating_sub(1); },
        }
    }

    if let Some(error) = failure {
        let step = failure_step;
        let feature_set = if cfg!(feature = "diagnostics") { "diagnostics" } else { "default" }.to_owned();
        let diagnostic = driver.diagnostic();
        let mut found = FuzzFailure { driver: driver_class, driver_config: driver.describe(), feature_set, seed, policy, actions: actions.clone(), step,
            all_legal_every: options.all_legal_every,
            kind: error.kind.clone(), message: error.message.clone(), diagnostic, known_bead: None, occurrences: 1,
            example_seeds: vec![seed], repro_path: None };
        if allow_shrink && !options.shrink_budget.is_zero() {
            shrink_failure(driver.as_ref(), seed, policy, &options, &mut found);
        }
        result.failure = Some(found);
    }
    result.reached_combat = driver.reached_combat();
    result.labels = driver.labels();
    result
}

fn shrink_failure(driver: &dyn Driver, seed: u64, policy: Policy, options: &EpisodeOptions, failure: &mut FuzzFailure) {
    let deadline = Instant::now() + options.shrink_budget;
    let expected_kind = failure.kind.clone();
    let expected_key = failure_key(failure);
    let mut smallest_driver = driver.fresh();
    let mut actions = failure.actions.clone();
    let mut step = failure.step;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() { break; }
        if expected_kind != "livelock" {
            actions = shrink_actions(&actions, step, &expected_kind, remaining, |candidate| {
                run_failure_trial(smallest_driver.fresh(), seed, policy, options, candidate, &expected_kind)
                    .filter(|observed| failure_key(observed) == expected_key)
                    .map(|observed| { step = observed.step; observed.kind })
            });
        }

        let mut accepted = None;
        for candidate in smallest_driver.shrink_candidates() {
            if Instant::now() >= deadline { break; }
            if let Some(observed) = run_failure_trial(candidate.fresh(), seed, policy, options, &actions, &expected_kind)
                .filter(|observed| failure_key(observed) == expected_key) {
                step = observed.step;
                accepted = Some(candidate);
                break;
            }
        }
        match accepted {
            Some(candidate) => smallest_driver = candidate,
            None => break,
        }
    }
    failure.actions = actions;
    failure.step = step;
    failure.driver_config = smallest_driver.describe();
}

fn run_failure_trial(driver: Box<dyn Driver>, seed: u64, policy: Policy, options: &EpisodeOptions,
    actions: &[usize], expected_kind: &str) -> Option<FuzzFailure> {
    let mut trial = options.clone();
    trial.determinism = false;
    trial.replay = false;
    trial.shrink_budget = Duration::ZERO;
    if expected_kind == "livelock" {
        trial.scripted_actions = None;
        trial.continue_first = false;
    } else {
        trial.scripted_actions = Some(actions.to_vec());
        trial.continue_first = true;
    }
    run_episode_inner(driver, seed, policy, trial, false).failure
        .filter(|failure| failure.kind == expected_kind)
}

fn check_illegal_probe(driver: &mut dyn Driver, probe: IllegalProbe) -> Result<(), DriverError> {
    let before = driver.decision().map_err(|error| DriverError::new("illegal_corrupts", format!("cannot snapshot before illegal input: {}", error.message)))?;
    #[cfg(feature = "diagnostics")]
    let before_run = driver.inner_run();
    match driver.probe_illegal(probe) {
        Ok(()) => return Err(DriverError::new("illegal_accepted", format!("{probe:?} was accepted"))),
        Err(error) if error.kind == "probe_rejected" => {}
        Err(error) if error.kind == "probe_accepted" => return Err(DriverError::new("illegal_accepted", error.message)),
        Err(error) if error.kind == "probe_corrupts" => return Err(DriverError::new("illegal_corrupts", error.message)),
        Err(error) => return Err(DriverError::new("illegal_corrupts", format!("{probe:?} caused {}: {}", error.kind, error.message))),
    }
    let after = driver.decision().map_err(|error| DriverError::new("illegal_corrupts", format!("decision after {probe:?}: {}", error.message)))?;
    if before != after { return Err(DriverError::new("illegal_corrupts", format!("{probe:?} changed decision()"))); }
    #[cfg(feature = "diagnostics")]
    if before_run != driver.inner_run() { return Err(DriverError::new("illegal_corrupts", format!("{probe:?} changed inner Run"))); }
    Ok(())
}

fn verify_random_prefix(driver: &dyn Driver, seed: u64, actions: &[usize], expected: &[String], prefix: usize,
    #[cfg(feature = "diagnostics")] expected_run: Option<&Run>,
) -> Result<(), DriverError> {
    let replay = replay_actions_prefix(driver, seed, &actions[..prefix])?;
    if expected.get(prefix) != Some(&replay.hash) {
        return Err(DriverError::new("replay", format!("fresh prefix {prefix} public trace differs")));
    }
    #[cfg(feature = "diagnostics")]
    if expected_run != replay.inner_run.as_ref() {
        return Err(DriverError::new("replay", format!("fresh prefix {prefix} inner Run differs")));
    }
    Ok(())
}

fn replay_actions(driver: &dyn Driver, seed: u64, actions: &[usize], _policy: Policy, _max_steps: usize) -> Result<String, DriverError> {
    replay_actions_prefix(driver, seed, actions).map(|replay| replay.hash)
}

struct PrefixReplay {
    hash: String,
    #[cfg(feature = "diagnostics")]
    inner_run: Option<Run>,
}

fn replay_actions_prefix(driver: &dyn Driver, seed: u64, actions: &[usize]) -> Result<PrefixReplay, DriverError> {
    let mut replay = driver.fresh();
    let boundary = replay.reset(seed).map_err(|error| DriverError::new("replay", format!("fresh reset: {}", error.message)))?;
    replay.descriptor_stable().map_err(|error| DriverError::new("replay", format!("prefix reset descriptor stability: {}", error.message)))?;
    let mut trace = TraceHasher::new(&trace_identity(replay.as_ref()), seed, &boundary.observation);
    let mut decision = boundary;
    for (step, index) in actions.iter().copied().enumerate() {
        if !legal_indices(&decision).contains(&index) { return Err(DriverError::new("replay", format!("action {index} not legal at replay step {step}"))); }
        match replay.step(index) {
            Ok(output) => {
                replay.descriptor_stable().map_err(|error| DriverError::new("replay", format!("prefix descriptor stability at step {step}: {}", error.message)))?;
                trace.step(index, output.reward, &output.decision.observation); decision = output.decision;
            }
            Err(error) if error.kind == "profile_cutoff" => break,
            Err(error) => return Err(DriverError::new("replay", format!("replay step {step}: {}", error.message))),
        }
    }
    Ok(PrefixReplay {
        hash: trace.finish(),
        #[cfg(feature = "diagnostics")]
        inner_run: replay.inner_run(),
    })
}

fn trace_identity(driver: &dyn Driver) -> Value {
    json!({"driver":driver.describe(),"stability_identity":driver.stability_identity()})
}

fn legal_indices(decision: &Decision) -> Vec<usize> {
    decision.observation.action_mask.iter().enumerate().filter_map(|(index, mask)| (*mask != 0).then_some(index)).collect()
}

fn empty_decision() -> Decision {
    Decision { observation: RunObservation { groups: vec![], action_mask: vec![] }, token: [0, 0], terminal: None }
}
fn decision_terminal(decision: &Decision) -> Option<&str> { decision.terminal.as_deref() }
fn count_terminal(terminal: &str, result: &mut EpisodeResult) {
    if terminal.eq_ignore_ascii_case("victory") || terminal == "HeartVictory" { result.wins = 1; }
    if terminal.eq_ignore_ascii_case("death") || terminal == "Death" { result.deaths = 1; }
    if terminal.eq_ignore_ascii_case("escaped") { result.escapes = 1; }
}
fn bump(map: &mut BTreeMap<String, u64>, key: &str) { *map.entry(key.into()).or_default() += 1; }

pub fn run_repro(case: &ReproCase) -> Result<(), String> {
    let active_features = if cfg!(feature = "diagnostics") { "diagnostics" } else { "default" };
    if case.feature_set != active_features {
        let hint = if case.feature_set == "diagnostics" { " --features diagnostics" } else { " without --features diagnostics" };
        return Err(format!("repro requires the {} feature set; rerun{}", case.feature_set, hint));
    }
    let driver = driver_from_config(&case.driver, &case.driver_config)?;
    let mut options = EpisodeOptions::default();
    options.max_steps = case.step.saturating_add(1).max(case.actions.len());
    options.all_legal_every = case.all_legal_every;
    options.determinism = case.expected_kind == "determinism";
    options.replay = case.expected_kind == "replay";
    options.shrink_budget = Duration::ZERO;
    if case.expected_kind == "livelock" {
        options.scripted_actions = None;
        options.continue_first = false;
    } else {
        options.scripted_actions = Some(case.actions.clone());
        options.continue_first = true;
    }
    let result = run_episode_inner(driver, case.seed, case.policy, options, false);
    match result.failure {
        Some(failure) if failure_key_for_config(&case.driver, &case.driver_config, &Value::Null,
            &case.expected_kind, &case.expected_message) == failure_key(&failure) => Ok(()),
        Some(failure) => Err(format!("repro mismatch: expected {} / {}, observed {} / {}", case.expected_kind,
            failure_key_for_config(&case.driver, &case.driver_config, &Value::Null, &case.expected_kind, &case.expected_message),
            failure.kind, failure_key(&failure))),
        None => Err(format!("expected {} but no failure reproduced", case.expected_kind)),
    }
}

/// Shrink one already-grouped campaign failure. The CLI defers this until failures
/// have been grouped so repeated instances do not each spend a shrink budget.
pub fn shrink_failure_case(failure: &mut FuzzFailure, budget: Duration) -> Result<(), String> {
    let driver = driver_from_config(&failure.driver, &failure.driver_config)?;
    let mut options = EpisodeOptions::default();
    options.max_steps = failure.step.saturating_add(1).max(failure.actions.len()).max(1);
    options.all_legal_every = failure.all_legal_every;
    options.determinism = false;
    options.replay = false;
    options.shrink_budget = budget;
    shrink_failure(driver.as_ref(), failure.seed, failure.policy, &options, failure);
    Ok(())
}

pub(crate) fn driver_from_config(class: &str, config: &Value) -> Result<Box<dyn Driver>, String> {
    match class {
        "catalog" => {
            let bytes = serde_json::to_vec(&config["catalog"]).map_err(|error| error.to_string())?;
            let mut driver = CatalogCombat::from_json(&bytes).map_err(|error| error.message)?;
            if let Some(id) = config["entry_id"].as_str() { driver.select_entry_id(id).map_err(|error| error.message)?; }
            Ok(Box::new(driver))
        }
        "run" => {
            let id = config["profile"].as_str().ok_or_else(|| "Run repro config lacks profile".to_owned())?;
            let spec = profile(id).ok_or_else(|| format!("unknown Run profile {id}"))?;
            Ok(Box::new(FullRun::new(spec).map_err(|error| error.message)?))
        }
        #[cfg(feature = "diagnostics")]
        "complete" => {
            let character = parse_character(config["character"].as_str().ok_or_else(|| "complete repro config lacks character".to_owned())?)?;
            let encounter = parse_encounter(config["encounter"].as_str().ok_or_else(|| "complete repro config lacks encounter".to_owned())?)?;
            let driver = if config["start"].is_object() { CompleteCombat::from_repro(character, encounter, parse_start(&config["start"])?) }
                else { CompleteCombat::new(character, encounter) };
            Ok(Box::new(driver))
        }
        #[cfg(not(feature = "diagnostics"))]
        "complete" => Err("complete repro requires --features diagnostics".into()),
        #[cfg(feature = "diagnostics")]
        "items" => {
            let id = config["profile"].as_str().ok_or_else(|| "CompleteItemsRun repro config lacks profile".to_owned())?;
            let profile = complete_items_profile(id).ok_or_else(|| format!("unknown CompleteItemsRun profile {id}"))?;
            let supply = config.get("supply").filter(|value| value.is_object())
                .map(|value| parse_item_supply(value, profile.character)).transpose()?;
            let driver = supply.map(|supply| CompleteItemsRun::from_repro(profile, supply))
                .unwrap_or_else(|| CompleteItemsRun::new(profile));
            Ok(Box::new(driver))
        }
        #[cfg(not(feature = "diagnostics"))]
        "items" => Err("CompleteItemsRun repro requires --features diagnostics".into()),
        _ => Err(format!("unknown fuzz driver {class}")),
    }
}

#[cfg(feature = "diagnostics")]
fn parse_start(value: &Value) -> Result<AuthoredRunStart, String> {
    let cards = spire_core::content::Catalog::standard().cards();
    let mut deck = Vec::new();
    for card in value["deck"].as_array().ok_or_else(|| "complete start lacks deck".to_owned())? {
        let name = card["id"].as_str().ok_or_else(|| "complete card lacks id".to_owned())?;
        let id = cards.iter().find(|row| row.id.as_str() == name).map(|row| row.id).ok_or_else(|| format!("unknown card {name}"))?;
        deck.push(CardInstance { id, instance: card["instance"].as_u64().ok_or_else(|| "invalid instance".to_owned())?,
            upgrades: card["upgrades"].as_u64().ok_or_else(|| "invalid upgrades".to_owned())? as u32,
            misc: card["misc"].as_i64().ok_or_else(|| "invalid misc".to_owned())? as i32 });
    }
    let relics = value["relics"].as_array().ok_or_else(|| "complete start lacks relics".to_owned())?.iter()
        .map(|item| parse_relic(item.as_str().ok_or_else(|| "invalid relic".to_owned())?)).collect::<Result<Vec<_>,_>>()?;
    let potions = value["potions"].as_array().ok_or_else(|| "complete start lacks potions".to_owned())?.iter()
        .map(|item| item.as_str().map(parse_potion).transpose()).collect::<Result<Vec<_>,_>>()?;
    let mut counters = BTreeMap::new();
    if let Some(values) = value["relic_counters"].as_object() { for (name, count) in values {
        counters.insert(parse_relic(name)?, count.as_i64().ok_or_else(|| "invalid relic counter".to_owned())? as i32);
    }}
    Ok(AuthoredRunStart { hp: number(&value["hp"], "hp")?, max_hp: number(&value["max_hp"], "max_hp")?, gold: number(&value["gold"], "gold")?,
        deck, relics, potions, relic_counters: counters })
}

#[cfg(feature = "diagnostics")]
fn parse_item_supply(value: &Value, character: Character) -> Result<CompleteItemSupply, String> {
    let relic_catalog = spire_core::content::complete::relic_union(character);
    let potion_catalog = spire_core::content::complete::potion_union(character);
    let relics = value["relics"].as_array().ok_or_else(|| "item supply lacks relics".to_owned())?.iter()
        .map(|item| {
            let name = item.as_str().ok_or_else(|| "invalid complete item relic".to_owned())?;
            relic_catalog.iter().copied().find(|id| id.as_str() == name).ok_or_else(|| format!("unknown complete item relic {name}"))
        }).collect::<Result<Vec<_>, _>>()?;
    let potions = value["potions"].as_array().ok_or_else(|| "item supply lacks potions".to_owned())?.iter()
        .map(|item| item.as_str().map(|name| {
            potion_catalog.iter().copied().find(|id| id.as_str() == name).ok_or_else(|| format!("unknown complete item potion {name}"))
        }).transpose()).collect::<Result<Vec<_>, _>>()?;
    let mut counters = BTreeMap::new();
    if let Some(values) = value["counters"].as_object() {
        for (name, count) in values {
            let id = relic_catalog.iter().copied().find(|id| id.as_str() == name).ok_or_else(|| format!("unknown counter relic {name}"))?;
            let count = count.as_i64().and_then(|count| i32::try_from(count).ok()).ok_or_else(|| format!("invalid counter for {name}"))?;
            counters.insert(id, count);
        }
    }
    Ok(CompleteItemSupply { relics, potions, counters })
}

#[cfg(feature = "diagnostics")]
fn parse_relic(name: &str) -> Result<RelicId, String> { all_relics().into_iter().find(|id| id.as_str() == name).ok_or_else(|| format!("unknown relic {name}")) }
#[cfg(feature = "diagnostics")]
fn parse_potion(name: &str) -> Result<PotionId, String> { all_potions().into_iter().find(|id| id.as_str() == name).ok_or_else(|| format!("unknown potion {name}")) }
#[cfg(feature = "diagnostics")]
fn all_relics() -> Vec<RelicId> {
    TRAINING_RELICS.iter().chain(spire_core::run::M15_SILENT_HEART_RELICS).chain(spire_core::run::M20_DEFECT_HEART_RELICS).chain(spire_core::run::M25_WATCHER_HEART_RELICS)
        .copied().collect::<std::collections::BTreeSet<_>>().into_iter().collect()
}
#[cfg(feature = "diagnostics")]
fn all_potions() -> Vec<PotionId> {
    let mut potions = TRAINING_POTIONS.iter().chain(spire_core::run::M15_SILENT_HEART_POTIONS).chain(spire_core::run::M20_DEFECT_HEART_POTIONS).chain(spire_core::run::M25_WATCHER_HEART_POTIONS)
        .copied().collect::<Vec<_>>();
    potions.sort_by_key(|potion| potion.as_str());
    potions.dedup();
    potions
}
#[cfg(feature = "diagnostics")]
fn parse_character(name: &str) -> Result<Character, String> { Character::ALL.into_iter().find(|character| character.as_str() == name).ok_or_else(|| format!("unknown character {name}")) }
#[cfg(feature = "diagnostics")]
fn parse_encounter(name: &str) -> Result<EncounterId, String> { TRAINING_ENCOUNTERS_V2.iter().map(|entry| entry.0).find(|id| id.as_str() == name).ok_or_else(|| format!("unknown encounter {name}")) }
#[cfg(feature = "diagnostics")]
fn number(value: &Value, name: &str) -> Result<u32, String> { value.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or_else(|| format!("invalid {name}")) }

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|value| format!("{value:02x}")).collect() }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StepOut;

    struct EmptyBoundaryDriver {
        cutoff: Option<&'static str>,
        decision: Option<Decision>,
    }

    impl EmptyBoundaryDriver {
        fn new(cutoff: Option<&'static str>) -> Self { Self { cutoff, decision: None } }
    }

    impl Driver for EmptyBoundaryDriver {
        fn class(&self) -> &'static str { "run" }
        fn describe(&self) -> Value { json!({"driver":"run","profile":"M12"}) }
        fn fresh(&self) -> Box<dyn Driver> { Box::new(Self::new(self.cutoff)) }
        fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
            let decision = Decision { observation: RunObservation { groups: vec![], action_mask: vec![0] }, token: [seed, 1], terminal: None };
            self.decision = Some(decision.clone());
            Ok(decision)
        }
        fn decision(&self) -> Result<Decision, DriverError> {
            self.decision.clone().ok_or_else(|| DriverError::fault("not reset"))
        }
        fn step(&mut self, _index: usize) -> Result<StepOut, DriverError> { Err(DriverError::fault("empty boundary was stepped")) }
        fn preflight(&self, _index: usize) -> Result<(), DriverError> { Err(DriverError::fault("empty boundary was preflighted")) }
        fn probe_illegal(&mut self, _probe: IllegalProbe) -> Result<(), DriverError> { Err(DriverError::new("probe_rejected", "rejected")) }
        fn descriptor_stable(&self) -> Result<(), DriverError> { Ok(()) }
        fn diagnostic(&self) -> Value { Value::Null }
        fn labels(&self) -> BTreeMap<String, String> { BTreeMap::from([("profile".into(), "M12".into())]) }
        fn profile_cutoff(&self) -> Option<&'static str> { self.cutoff }
    }

    fn no_step_options() -> EpisodeOptions {
        EpisodeOptions { max_steps: 5, all_legal_every: 0, determinism: false, replay: false,
            shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false }
    }

    #[test]
    fn accepted_zero_action_profile_cutoff_is_an_ending_not_a_cap() {
        let result = run_episode(Box::new(EmptyBoundaryDriver::new(Some("Act2Ready"))), 1, Policy::First, no_step_options());
        assert_eq!(result.profile_cutoff.as_deref(), Some("Act2Ready"));
        assert_eq!(result.capped, 0);
        assert!(result.failure.is_none());
        let mut summary = RunSummary::new("test-revision".into());
        summary.add(result);
        assert_eq!(summary.profile_cutoffs, 1);
        assert_eq!(summary.profile_cutoffs_by_kind.get("Act2Ready"), Some(&1));
        assert_eq!(summary.profile_cutoffs_by_profile.get("M12"), Some(&1));
        assert_eq!(summary.capped, 0);
    }

    #[test]
    fn unexpected_nonterminal_zero_action_decision_is_stuck() {
        let result = run_episode(Box::new(EmptyBoundaryDriver::new(None)), 1, Policy::First, no_step_options());
        assert_eq!(result.capped, 0);
        assert_eq!(result.failure.as_ref().map(|failure| failure.kind.as_str()), Some("stuck"));
        assert_eq!(result.failure.as_ref().map(|failure| failure.message.as_str()), Some("non-terminal decision publishes zero legal actions"));
    }

    #[test]
    fn accepted_m12_and_m17_act2ready_boundaries_are_not_caps() {
        let options = EpisodeOptions { max_steps: 512, all_legal_every: 0, determinism: false, replay: true,
            shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false };
        // spire-oto.22 C2: the earlier M12 seed (12_138_998_340_561_802_975) now dies at the Guardian.
        // Poison's Mode Shift resolves before its takeTurn, so it plays Close Up rather than Charge Up.
        // A First-policy run reaching the boundary is a policy outcome; this seed reaches it on the C2 engine.
        for (profile_id, seed) in [("M12", 3_997_841_687_880_340_916), ("M17", 4_827_708_856_933_560_645)] {
            let spec = crate::profiles::profile(profile_id).expect("accepted Run profile");
            let driver = FullRun::new(spec).expect("accepted Run constructor");
            let result = run_episode(Box::new(driver), seed, Policy::First, options.clone());
            assert_eq!(result.profile_cutoff.as_deref(), Some("Act2Ready"), "{profile_id}");
            assert_eq!(result.capped, 0, "{profile_id}");
            assert!(result.failure.is_none(), "{profile_id}: {:?}", result.failure);
        }
    }

    fn failing_result(seed: u64, resolved_effects: u8) -> EpisodeResult {
        let message = format!("complete combat startup: Faulted(RunFault {{ kind: Combat(CombatFault {{ record: FaultRecord {{ kind: Invariant, message: \"decision before quiescence\", resolved_effects: {resolved_effects} }}, current: Some(Effect {{ source: None, target: None, kind: Decision }}), pending: [Effect {{ source: None, target: Some(Player), kind: GainBlock(10) }}] }} }})");
        EpisodeResult {
            driver: "complete".into(), labels: BTreeMap::new(), policy: Policy::First, seed,
            trace_hash: String::new(), steps: 0, wins: 0, deaths: 0, escapes: 0, reached_combat: false,
            all_legal_checks_after_step_zero: 0, capped: 0, capped_reason: None,
            profile_cutoff: None, unsupported: 0,
            failure: Some(FuzzFailure {
                driver: "complete".into(), driver_config: json!({"encounter":format!("cultist-{seed}")}),
                feature_set: "diagnostics".into(), seed, policy: Policy::First, actions: vec![], step: 0,
                all_legal_every: 0, kind: "fault".into(), message, diagnostic: Value::Null, known_bead: None, occurrences: 1,
                example_seeds: vec![seed], repro_path: None,
            }),
        }
    }

    #[test]
    fn summary_deduplicates_engine_fault_identity_across_numeric_state_ids() {
        let mut summary = RunSummary::new("test-revision".into());
        summary.add(failing_result(7, 14));
        summary.add(failing_result(8, 20));
        assert_eq!(summary.failures.len(), 1, "numeric resolver progress and encounter suffix must not split a fault group");
        assert_eq!(summary.failures[0].occurrences, 2);
        assert_eq!(summary.failures[0].example_seeds, [7, 8]);
    }

    #[test]
    fn summary_breaks_caps_down_by_driver_policy_profile_and_reason() {
        let mut summary = RunSummary::new("test-revision".into());
        let mut labels = BTreeMap::new();
        labels.insert("profile".into(), "M15SilentHeart".into());
        summary.add(EpisodeResult { driver: "run".into(), labels, policy: Policy::ChooseNone, seed: 1,
            trace_hash: String::new(), steps: 12, wins: 0, deaths: 0, escapes: 0, reached_combat: true,
            all_legal_checks_after_step_zero: 2, capped: 1,
            capped_reason: Some("max_steps".into()), profile_cutoff: None, unsupported: 0, failure: None });
        assert_eq!(summary.capped_by_driver.get("run"), Some(&1));
        assert_eq!(summary.capped_by_policy.get("choose_none"), Some(&1));
        assert_eq!(summary.capped_by_profile.get("M15SilentHeart"), Some(&1));
        assert_eq!(summary.capped_by_reason.get("max_steps"), Some(&1));
        assert_eq!(summary.capped_by_driver_policy_profile.get("run|choose_none|M15SilentHeart"), Some(&1));
        assert_eq!(summary.outcomes_by_driver.get("run").unwrap().episodes, 1);
        assert_eq!(summary.outcomes_by_driver.get("run").unwrap().steps, 12);
        assert_eq!(summary.outcomes_by_driver.get("run").unwrap().capped, 1);
        assert_eq!(summary.outcomes_by_profile.get("M15SilentHeart").unwrap().episodes, 1);
        assert_eq!(summary.outcomes_by_policy.get("choose_none").unwrap().episodes, 1);
    }
}
