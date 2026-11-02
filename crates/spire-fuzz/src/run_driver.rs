use std::collections::BTreeMap;
use serde_json::{Value, json};
#[cfg(feature = "diagnostics")]
use spire_core::run::Run;
#[cfg(feature = "diagnostics")]
use spire_core::run::RunError;
use spire_env::{RunActionIndex, RunBoundary, RunDecisionToken, RunEpisodeRecord, RunSession, RunSessionError,
    RunStepResult, encode_run, encode_run_m10, encode_run_m12, encode_run_m13, encode_run_m14, encode_run_m15,
    encode_run_m17, encode_run_m18, encode_run_m19, encode_run_m20, encode_run_m22, encode_run_m23,
    encode_run_m24, encode_run_m25, encode_run_policy, replay_run};
use crate::{Decision, Driver, DriverError, IllegalProbe, RunProfileSpec, StepOut, mix64};

#[derive(Clone)]
struct Seal { observation: [u8; 32], environment: Value }

pub struct FullRun {
    spec: RunProfileSpec,
    session: RunSession,
    seal: Option<Seal>,
    saw_combat: bool,
}

impl FullRun {
    pub fn new(spec: RunProfileSpec) -> Result<Self, DriverError> {
        let session = (spec.constructor)(spec.config()).map_err(|error| DriverError::fault(format!("{} session creation: {error:?}", spec.id)))?;
        Ok(Self { spec, session, seal: None, saw_combat: false })
    }

    fn from_boundary(boundary: RunBoundary) -> Decision { Decision { observation: boundary.observation, token: boundary.token.into(), terminal: boundary.terminal.map(|terminal| format!("{terminal:?}")) } }
    fn config_value(&self) -> Value {
        json!({"driver":"run","profile":self.spec.id,"character":character_name(self.spec.character),
            "ascension":0,"partition":"test","record_replay":true})
    }
    fn capture_seal(&self) -> Result<Seal, DriverError> {
        Ok(Seal { observation: self.session.observation_spec().map_err(|error| DriverError::new("encode", format!("observation spec: {error:?}")))?.fingerprint(),
            environment: self.session.environment_spec().map_err(|error| DriverError::new("encode", format!("environment spec: {error:?}")))? })
    }
    fn record(&self) -> Result<RunEpisodeRecord, DriverError> {
        self.session.record().map_err(|error| DriverError::new("replay", format!("record: {error:?}")))?
            .ok_or_else(|| DriverError::new("replay", "record_replay was not enabled"))
    }
    fn replay_observation(&self, record: &RunEpisodeRecord) -> Result<spire_env::RunObservation, DriverError> {
        let report = replay_run(record, record.config.capacities.clone())
            .map_err(|error| DriverError::new("replay", format!("replay_run: {error:?}")))?;
        if report.accepted_count != record.steps.len() {
            return Err(DriverError::new("replay", format!("replay accepted {} of {} steps", report.accepted_count, record.steps.len())));
        }
        encode_public_decision(self.spec, &report.final_decision, report.final_map.as_ref(), &record.config.capacities)
            .map_err(|error| DriverError::new("replay", format!("replay final encode: {error:?}")))
    }

    fn observe_combat(&mut self) {
        if let Ok(reproduction) = self.session.reproduction() {
            self.saw_combat |= reproduction.latest_decision
                .is_some_and(|decision| decision.is_ok_and(|decision| decision.combat.is_some()));
        }
    }
}

impl Driver for FullRun {
    fn class(&self) -> &'static str { "run" }
    fn describe(&self) -> Value { self.config_value() }
    fn fresh(&self) -> Box<dyn Driver> { Box::new(Self::new(self.spec).expect("accepted profile remains constructible")) }
    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        self.saw_combat = false;
        let boundary = self.session.reset(Some(seed)).map_err(|error| DriverError::fault(format!("reset: {error:?}; diagnostic={}", self.session_diagnostic())))?;
        self.seal = Some(self.capture_seal()?);
        self.observe_combat();
        Ok(Self::from_boundary(boundary))
    }
    fn decision(&self) -> Result<Decision, DriverError> {
        self.session.decision().map(Self::from_boundary).map_err(|error| DriverError::fault(format!("decision: {error:?}; diagnostic={}", self.session_diagnostic())))
    }
    fn step(&mut self, index: usize) -> Result<StepOut, DriverError> {
        let decision = self.decision()?;
        match self.session.step(decision.token.into(), RunActionIndex(index as i64)) {
            Ok(result) => {
                self.observe_combat();
                Ok(step_out(result))
            }
            Err(RunSessionError::UnsupportedBoundary(message)) if is_profile_cutoff(message) => Err(DriverError::new("profile_cutoff", message)),
            Err(error) => Err(DriverError::fault(format!("step index={index}: {error:?}; diagnostic={}", self.session_diagnostic()))),
        }
    }
    fn preflight(&self, index: usize) -> Result<(), DriverError> {
        let decision = self.decision()?;
        self.session.preflight_step(decision.token.into(), RunActionIndex(index as i64))
            .map_err(|error| DriverError::new("legal_rejected", format!("advertised index={index}: {error:?}")))
    }
    fn probe_illegal(&mut self, probe: IllegalProbe) -> Result<(), DriverError> {
        let decision = self.decision()?;
        let token = match probe {
            IllegalProbe::StaleToken => RunDecisionToken::from([decision.token[0], decision.token[1].wrapping_sub(1)]),
            _ => RunDecisionToken::from(decision.token),
        };
        let index = match probe {
            IllegalProbe::AtLegalCount => decision.observation.action_mask.iter().filter(|mask| **mask != 0).count() as i64,
            IllegalProbe::NegativeIndex => -1,
            IllegalProbe::StaleToken => 0,
        };
        self.session.step(token, RunActionIndex(index)).map(|_| ())
            .map_err(|error| DriverError::new("probe_rejected", format!("{probe:?}: {error:?}")))
    }
    fn descriptor_stable(&self) -> Result<(), DriverError> {
        let before = self.seal.as_ref().ok_or_else(|| DriverError::new("encode", "descriptor seal missing"))?;
        let after = self.capture_seal()?;
        if before.observation != after.observation || before.environment != after.environment {
            return Err(DriverError::new("encode", "Run environment or observation fingerprint changed during episode"));
        }
        Ok(())
    }
    fn stability_identity(&self) -> Value {
        self.seal.as_ref().map(|seal| json!({"observation":seal.observation,"environment":seal.environment}))
            .unwrap_or(Value::Null)
    }
    fn diagnostic(&self) -> Value { self.session_diagnostic() }
    fn reached_combat(&self) -> bool { self.saw_combat }
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("profile".into(), self.spec.id.into());
        labels.insert("character".into(), character_name(self.spec.character).into());
        labels
    }
    fn profile_cutoff(&self) -> Option<&'static str> {
        let reproduction = self.session.reproduction().ok()?;
        let decision = reproduction.latest_decision?.ok()?;
        if decision.terminal.is_some() || !decision.candidates.is_empty() { return None; }
        match decision.room.as_ref()?.kind {
            spire_core::run::RoomViewKind::Act2Ready => Some("Act2Ready"),
            spire_core::run::RoomViewKind::Act3Ready => Some("Act3Ready"),
            spire_core::run::RoomViewKind::Act4Ready => Some("Act4Ready"),
            _ => None,
        }
    }
    fn verify_replay_contract(&self, seed: u64, actions: &[usize]) -> Result<(), DriverError> {
        let record = self.record()?;
        let final_observation = self.replay_observation(&record)?;
        let current = self.decision()?.observation;
        if final_observation != current { return Err(DriverError::new("replay", "replay_run final public observation differs from live session")); }
        let prefix = (mix64(seed ^ 0xA076_1D64_78BD_642F) as usize) % (record.steps.len() + 1);
        let mut prefix_record = record.clone();
        prefix_record.steps.truncate(prefix);
        prefix_record.terminal = prefix_record.steps.last().and_then(|step| step.terminal);
        prefix_record.fault = None;
        prefix_record.closed = false;
        let prefix_observation = self.replay_observation(&prefix_record)?;
        if prefix == actions.len() {
            let current = self.decision()?.observation;
            if prefix_observation != current { return Err(DriverError::new("replay", "prefix replay final observation differs from live session")); }
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), DriverError> {
        #[cfg(feature = "diagnostics")]
        self.session.fuzz_run().ok_or_else(|| DriverError::new("validator", "inner Run is missing"))?.fuzz_validate()
            .map_err(|message| DriverError::new("validator", message))?;
        Ok(())
    }
    fn all_legal_on_clones(&self) -> Result<(), DriverError> {
        #[cfg(feature = "diagnostics")]
        {
            let run = self.session.fuzz_run().ok_or_else(|| DriverError::new("legal_rejected", "inner Run is missing"))?;
            let decision = run.decision().map_err(|error| DriverError::new("legal_rejected", format!("inner decision: {error:?}")))?;
            let candidates = decision.candidates;
            let published = self.decision()?;
            if candidates.len() != published.observation.action_mask.iter().filter(|mask| **mask != 0).count() {
                return Err(DriverError::new("legal_rejected", "session action mask and inner candidate counts differ"));
            }
            for (index, candidate) in candidates.iter().enumerate() {
                let mut clone = run.clone();
                if let Err(error) = clone.step(&candidate.action) {
                    if !matches!(error, RunError::Terminal) {
                        return Err(DriverError::new("legal_rejected", format!("advertised candidate {index} rejected: {error:?}")));
                    }
                }
                clone.fuzz_validate().map_err(|message| DriverError::new("validator", format!("clone after candidate {index}: {message}")))?;
            }
        }
        Ok(())
    }
    #[cfg(feature = "diagnostics")]
    fn inner_run(&self) -> Option<Run> { self.session.fuzz_run().cloned() }
}

impl FullRun {
    fn session_diagnostic(&self) -> Value {
        match self.session.reproduction() {
            Ok(reproduction) => {
                let record = reproduction.record.as_ref().and_then(|record| record.to_json().ok())
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
                json!({"profile":self.spec.id,"seed":reproduction.seed,"ordinal":reproduction.ordinal,
                    "fault":reproduction.fault.map(|error|format!("{error:?}")),
                    "latest_decision":reproduction.latest_decision.map(|decision|format!("{decision:?}")),
                    "latest_map":reproduction.latest_map.map(|map|format!("{map:?}")),"record":record,"closed":reproduction.closed})
            }
            Err(error) => json!({"profile":self.spec.id,"diagnostic_error":format!("{error:?}")}),
        }
    }
}

fn step_out(result: RunStepResult) -> StepOut {
    let decision = FullRun::from_boundary(result.boundary);
    StepOut { terminal: decision.terminal.clone(), decision, reward: result.reward }
}

fn is_profile_cutoff(message: &str) -> bool {
    message.contains("stops at Act2Ready") || message.contains("stops at Act3Ready")
        || message.contains("stops before the Act4 door") || message.contains("Act2Ready must be nonterminal")
        || message.contains("Act3Ready must be nonterminal")
}

fn character_name(character: spire_core::content::Character) -> &'static str {
    match character { spire_core::content::Character::Ironclad => "ironclad", spire_core::content::Character::Silent => "silent",
        spire_core::content::Character::Defect => "defect", spire_core::content::Character::Watcher => "watcher" }
}

fn encode_public_decision(spec: RunProfileSpec, decision: &spire_core::run::PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>, caps: &spire_env::RunCapacities) -> Result<spire_env::RunObservation, RunSessionError> {
    match spec.id {
        "M9" => encode_run(decision, map, caps),
        "M10" => encode_run_m10(decision, map, caps),
        "M10Policy" => encode_run_policy(decision, map, caps),
        "M12" => encode_run_m12(decision, map, caps),
        "M13" => encode_run_m13(decision, map, caps),
        "M14" => encode_run_m14(decision, map, caps),
        "M15" => encode_run_m15(decision, map, caps),
        "M17" => encode_run_m17(decision, map, caps),
        "M18" => encode_run_m18(decision, map, caps),
        "M19" => encode_run_m19(decision, map, caps),
        "M20" => encode_run_m20(decision, map, caps),
        "M22" => encode_run_m22(decision, map, caps),
        "M23" => encode_run_m23(decision, map, caps),
        "M24" => encode_run_m24(decision, map, caps),
        "M25" => encode_run_m25(decision, map, caps),
        _ => Err(RunSessionError::Input("unknown native fuzz Run profile")),
    }
}
