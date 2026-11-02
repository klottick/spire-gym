use std::collections::BTreeMap;
use serde_json::{Value, json};
use spire_core::{combat::rng::Partition, content::CardId, run::{TRAINING_CARDS, TRAINING_ENCOUNTERS_V2, TRAINING_POTIONS, TRAINING_RELICS}};
use spire_env::{CombatTrainingCatalog, CombatTrainingSession, RunActionIndex};
use crate::{Decision, Driver, DriverError, IllegalProbe, StepOut, mix64, observation::hex};

#[derive(Clone)]
struct Seal { observation: [u8; 32], catalog: Value, environment: Value }

pub struct CatalogCombat {
    catalog: CombatTrainingCatalog,
    catalog_json: Vec<u8>,
    entry_ids: Vec<String>,
    encounters: BTreeMap<String, String>,
    session: CombatTrainingSession,
    active_id: Option<String>,
    entry_cursor: usize,
    forced_entry_id: Option<String>,
    seal: Option<Seal>,
    saw_combat: bool,
}

impl CatalogCombat {
    pub fn from_json(bytes: &[u8]) -> Result<Self, DriverError> {
        let catalog = CombatTrainingCatalog::from_json(bytes).map_err(|error| DriverError::new("fault", format!("catalog rejected: {error:?}")))?;
        let value: Value = serde_json::from_slice(bytes).map_err(|error| DriverError::fault(error.to_string()))?;
        let entries = value["entries"].as_array().ok_or_else(|| DriverError::fault("catalog has no entries"))?;
        let mut entry_ids = Vec::with_capacity(entries.len());
        let mut encounters = BTreeMap::new();
        for entry in entries {
            let id = entry["id"].as_str().ok_or_else(|| DriverError::fault("catalog entry has no id"))?.to_owned();
            let encounter = entry["encounter"].as_str().ok_or_else(|| DriverError::fault("catalog entry has no encounter"))?.to_owned();
            encounters.insert(id.clone(), encounter);
            entry_ids.push(id);
        }
        let session = CombatTrainingSession::new(catalog.clone(), Partition::Test)
            .map_err(|error| DriverError::fault(format!("session creation: {error:?}")))?;
        Ok(Self { catalog, catalog_json: bytes.to_vec(), entry_ids, encounters, session, active_id: None,
            entry_cursor: 0, forced_entry_id: None, seal: None, saw_combat: false })
    }

    pub fn generated(seed: u64) -> Result<Self, DriverError> { Self::from_json(&generated_catalog_json(seed)) }

    pub fn entry_count(&self) -> usize { self.entry_ids.len() }

    /// Select the next entry used by reset; the cursor advances after selection.
    pub fn set_entry_cursor(&mut self, cursor: usize) {
        self.entry_cursor = cursor % self.entry_ids.len();
        self.forced_entry_id = None;
    }

    /// Pin this driver to a catalog ID when rebuilding a replay driver.
    pub fn select_entry_id(&mut self, id: &str) -> Result<(), DriverError> {
        if !self.entry_ids.iter().any(|entry| entry == id) {
            return Err(DriverError::new("fault", format!("catalog entry {id:?} is absent")));
        }
        self.forced_entry_id = Some(id.to_owned());
        Ok(())
    }

    fn session_decision(&self) -> Result<Decision, DriverError> {
        let boundary = self.session.decision().map_err(|error| DriverError::fault(format!("decision: {error:?}")))?;
        Ok(Decision { observation: boundary.observation, token: boundary.token.into(), terminal: None })
    }

    fn capture_seal(&self) -> Result<Seal, DriverError> {
        Ok(Seal {
            observation: self.session.observation_spec().map_err(|error| DriverError::new("encode", format!("observation spec: {error:?}")))?.fingerprint(),
            catalog: self.session.catalog_descriptor(),
            environment: self.session.environment_spec().map_err(|error| DriverError::new("encode", format!("environment spec: {error:?}")))?,
        })
    }
}

impl Driver for CatalogCombat {
    fn class(&self) -> &'static str { "catalog" }
    fn describe(&self) -> Value {
        json!({"driver":"catalog","catalog":serde_json::from_slice::<Value>(&self.catalog_json).unwrap_or(Value::Null),
            "catalog_fingerprint":hex(&self.catalog.fingerprint()),"partition":"test",
            "entry_id":self.active_id.as_deref().or(self.forced_entry_id.as_deref())})
    }
    fn fresh(&self) -> Box<dyn Driver> {
        let mut fresh = Self::from_json(&self.catalog_json).expect("validated catalog remains constructible");
        if let Some(id) = self.active_id.as_deref().or(self.forced_entry_id.as_deref()) {
            fresh.forced_entry_id = Some(id.to_owned());
        } else {
            fresh.entry_cursor = self.entry_cursor;
        }
        Box::new(fresh)
    }
    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        self.saw_combat = false;
        let id = if let Some(id) = &self.forced_entry_id {
            id.clone()
        } else {
            let id = self.entry_ids[self.entry_cursor].clone();
            self.entry_cursor = (self.entry_cursor + 1) % self.entry_ids.len();
            id
        };
        self.active_id = Some(id.clone());
        let boundary = self.session.reset(Some(seed), Some(&id))
            .map_err(|error| DriverError::fault(format!("reset: {error:?}; diagnostic={}", self.session.diagnostic())))?;
        self.saw_combat = true;
        self.seal = Some(self.capture_seal()?);
        Ok(Decision { observation: boundary.observation, token: boundary.token.into(), terminal: None })
    }
    fn decision(&self) -> Result<Decision, DriverError> { self.session_decision() }
    fn step(&mut self, index: usize) -> Result<StepOut, DriverError> {
        let decision = self.session_decision()?;
        let result = self.session.step(decision.token.into(), RunActionIndex(index as i64))
            .map_err(|error| DriverError::fault(format!("step index={index}: {error:?}; diagnostic={}", self.session.diagnostic())))?;
        let terminal = if result.terminated { result.boundary.info["outcome"].as_str().map(str::to_owned) } else { None };
        Ok(StepOut { decision: Decision { observation: result.boundary.observation, token: result.boundary.token.into(), terminal: terminal.clone() }, reward: result.reward, terminal })
    }
    fn preflight(&self, index: usize) -> Result<(), DriverError> {
        let decision = self.session_decision()?;
        self.session.preflight_step(decision.token.into(), RunActionIndex(index as i64))
            .map_err(|error| DriverError::new("legal_rejected", format!("advertised index={index}: {error:?}")))
    }
    fn probe_illegal(&mut self, probe: IllegalProbe) -> Result<(), DriverError> {
        let decision = self.session_decision()?;
        let token = match probe {
            IllegalProbe::StaleToken => spire_env::RunDecisionToken::from([decision.token[0], decision.token[1].wrapping_sub(1)]),
            _ => spire_env::RunDecisionToken::from(decision.token),
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
        if before.observation != after.observation || before.catalog != after.catalog || before.environment != after.environment {
            return Err(DriverError::new("encode", "catalog, environment or observation fingerprint changed during episode"));
        }
        Ok(())
    }
    fn stability_identity(&self) -> Value {
        self.seal.as_ref().map(|seal| json!({"observation":seal.observation,"catalog":seal.catalog,"environment":seal.environment}))
            .unwrap_or(Value::Null)
    }
    fn diagnostic(&self) -> Value { self.session.diagnostic() }
    fn reached_combat(&self) -> bool { self.saw_combat }
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("profile".into(), self.session.environment_spec().ok().and_then(|spec| spec["profile_id"].as_str().map(str::to_owned)).unwrap_or_else(|| "unknown".into()));
        labels.insert("character".into(), "ironclad".into());
        if let Some(id) = &self.active_id {
            labels.insert("entry".into(), id.clone());
            if let Some(encounter) = self.encounters.get(id) { labels.insert("encounter".into(), encounter.clone()); }
        }
        labels
    }
    fn shrink_candidates(&self) -> Vec<Box<dyn Driver>> {
        let Ok(mut catalog) = serde_json::from_slice::<Value>(&self.catalog_json) else { return Vec::new() };
        let Some(entries) = catalog["entries"].as_array() else { return Vec::new() };
        let selected = self.active_id.as_deref().and_then(|id| entries.iter().find(|entry| entry["id"].as_str() == Some(id)))
            .or_else(|| entries.first()).cloned();
        let Some(selected) = selected else { return Vec::new() };
        catalog["entries"] = json!([selected]);
        let base = catalog;
        let mut variants = Vec::new();
        let mut add = |candidate: Value| {
            if let Ok(bytes) = serde_json::to_vec(&candidate) {
                if let Ok(driver) = Self::from_json(&bytes) { variants.push(Box::new(driver) as Box<dyn Driver>); }
            }
        };
        add(base.clone());
        if let Some(deck) = base["entries"][0]["deck"].as_array() {
            for index in (0..deck.len()).rev() {
                if deck.len() <= 1 { break; }
                let mut candidate = base.clone();
                if let Some(cards) = candidate["entries"][0]["deck"].as_array_mut() { cards.remove(index); }
                add(candidate);
            }
            for index in 0..deck.len() {
                if deck[index]["upgrades"].as_u64().unwrap_or(0) == 0 { continue; }
                let mut candidate = base.clone();
                candidate["entries"][0]["deck"][index]["upgrades"] = json!(0);
                add(candidate);
            }
        }
        if let Some(relics) = base["entries"][0]["relics"].as_array() {
            for index in (0..relics.len()).rev() {
                let mut candidate = base.clone();
                if let Some(items) = candidate["entries"][0]["relics"].as_array_mut() { items.remove(index); }
                add(candidate);
            }
        }
        if let Some(potions) = base["entries"][0]["potions"].as_array() {
            for index in 0..potions.len() {
                if !potions[index].is_string() { continue; }
                let mut candidate = base.clone();
                candidate["entries"][0]["potions"][index] = Value::Null;
                add(candidate);
            }
        }
        let mut low_noise = base.clone();
        low_noise["entries"][0]["hp"] = json!(1);
        low_noise["entries"][0]["max_hp"] = json!(1);
        low_noise["entries"][0]["gold"] = json!(0);
        add(low_noise);
        variants
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
            let published = self.session_decision()?;
            if candidates.len() != published.observation.action_mask.iter().filter(|mask| **mask != 0).count() {
                return Err(DriverError::new("legal_rejected", "session action mask and inner candidate counts differ"));
            }
            for (index, candidate) in candidates.iter().enumerate() {
                let mut clone = run.clone();
                if let Err(error) = clone.step(&candidate.action) {
                    if !matches!(error, spire_core::run::RunError::Terminal) {
                        return Err(DriverError::new("legal_rejected", format!("advertised candidate {index} rejected: {error:?}")));
                    }
                }
                clone.fuzz_validate().map_err(|message| DriverError::new("validator", format!("clone after candidate {index}: {message}")))?;
            }
        }
        Ok(())
    }
    #[cfg(feature = "diagnostics")]
    fn inner_run(&self) -> Option<spire_core::run::Run> { self.session.fuzz_run().cloned() }
}

pub fn generated_catalog_json(seed: u64) -> Vec<u8> {
    let entries = TRAINING_ENCOUNTERS_V2.iter().enumerate().map(|(entry_index, (encounter, _, _))| {
        let deck = (0..10).map(|slot| {
            let selected = (mix64(seed ^ (entry_index as u64).wrapping_mul(0x9E37_79B9) ^ slot as u64) as usize) % TRAINING_CARDS.len();
            let card: CardId = TRAINING_CARDS[selected];
            json!({"card":card.as_str(),"upgrades":0})
        }).collect::<Vec<_>>();
        let relic_index = (mix64(seed.wrapping_add(entry_index as u64)) as usize) % TRAINING_RELICS.len();
        let potion = TRAINING_POTIONS[(mix64(seed ^ 0xA076_1D64_78BD_642F ^ entry_index as u64) as usize) % TRAINING_POTIONS.len()];
        json!({"id":format!("encounter-{:02}-{}",entry_index,encounter.as_str().to_ascii_lowercase()),
            "encounter":encounter.as_str(),"hp":80,"max_hp":80,"deck":deck,
            "relics":[TRAINING_RELICS[relic_index].as_str()],"potions":[potion.as_str(),null,null],"relic_counters":{},"gold":99})
    }).collect::<Vec<_>>();
    serde_json::to_vec(&json!({"profile_id":spire_env::COMBAT_TRAINING_PROFILE_V2_ID,"entries":entries})).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_entries_are_selected_round_robin_independently_of_episode_seed() {
        let bytes = generated_catalog_json(77);
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let expected = value["entries"].as_array().unwrap().iter().take(4)
            .map(|entry| entry["id"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
        let mut driver = CatalogCombat::from_json(&bytes).unwrap();
        for (index, seed) in [77, 9, 400, 2].into_iter().enumerate() {
            driver.reset(seed).unwrap();
            assert_eq!(driver.labels().get("entry"), Some(&expected[index]));
        }
    }
}
