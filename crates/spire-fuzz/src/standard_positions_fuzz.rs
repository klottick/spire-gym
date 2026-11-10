use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};
use spire_core::{
    combat::{enemies::{A0_ENCOUNTER_SETUPS, EncounterKind}, position::PLAYER_POWERS},
    content::{complete, CardId, Character, Catalog},
};
use spire_env::{CombatTrainingCatalog, RunSessionError};

use crate::{CatalogCombat, Decision, Driver, DriverError, IllegalProbe, StepOut, mix64};

const MAX_ATTEMPTS: usize = 8;
const FIXED_FORMATIONS: &[&str] = &[
    "cultist", "jaw_worm", "three_cultists", "jaw_worm_horde", "gremlin_nob", "lagavulin", "three_sentries",
    "guardian", "hexaghost", "spheric_guardian", "three_byrds", "chosen_and_byrds", "snecko",
    "centurion_and_healer", "slavers", "book_of_stabbing", "champ", "collector", "bronze_automaton",
    "orb_walker", "three_darklings", "spire_growth", "transient", "maw", "writhing_mass", "giant_head",
    "nemesis", "reptomancer", "awakened_one", "time_eater", "donu_and_deca", "shield_and_spear",
    "corrupt_heart", "blue_slaver", "red_slaver", "looter", "chosen", "shelled_parasite", "snake_plant",
    "two_thieves", "sentry_and_sphere", "cultist_and_chosen", "shelled_parasite_and_fungi", "three_fungi_beasts",
    "lagavulin_event", "colosseum_slavers", "colosseum_nobs", "masked_bandits", "two_orb_walkers", "two_fungi_beasts",
];

pub fn standard_position_spec(character: Character, ascension: u8) -> String {
    format!("standard-position-{}-a{}", character.as_str(), ascension)
}

fn spec_parts(class: &str) -> Option<(Character, u8)> {
    let name = class.strip_prefix("standard-position-")?;
    let (character, ascension) = name.rsplit_once("-a")?;
    let character = Character::ALL.into_iter().find(|c| c.as_str() == character)?;
    let ascension = ascension.parse::<u8>().ok()?;
    matches!(ascension, 0 | 20).then_some((character, ascension))
}

/// Episode driver for the M34s-c synthetic position fuzz specs. A fresh catalog is built from the
/// episode seed, parsed through the public Standard catalog, and then driven by CatalogCombat.
pub struct StandardPositionCombat {
    character: Character,
    ascension: u8,
    catalog_json: Option<Vec<u8>>,
    inner: Option<CatalogCombat>,
    entry_id: Option<String>,
    rejected: usize,
}

impl StandardPositionCombat {
    pub fn new(character: Character, ascension: u8) -> Self {
        Self { character, ascension, catalog_json: None, inner: None, entry_id: None, rejected: 0 }
    }

    pub(crate) fn from_config(config: &Value) -> Result<Self, String> {
        let class = config["position_spec"].as_str().ok_or_else(|| "position repro config lacks position_spec".to_owned())?;
        let (character, ascension) = spec_parts(class).ok_or_else(|| format!("invalid Standard position spec {class}"))?;
        let bytes = serde_json::to_vec(&config["catalog"]).map_err(|error| error.to_string())?;
        let mut inner = CatalogCombat::standard_from_json(&bytes, character, ascension).map_err(|error| error.message)?;
        let entry_id = config["entry_id"].as_str().ok_or_else(|| "position repro config lacks entry_id".to_owned())?.to_owned();
        inner.select_entry_id(&entry_id).map_err(|error| error.message)?;
        Ok(Self { character, ascension, catalog_json: Some(bytes), inner: Some(inner), entry_id: Some(entry_id),
            rejected: config["rejected_candidates"].as_u64().unwrap_or(0) as usize })
    }

    pub fn rejected_candidates(&self) -> usize { self.rejected }

    fn generate(&mut self, seed: u64) -> Result<(), DriverError> {
        for attempt in 0..MAX_ATTEMPTS {
            let bytes = position_catalog(seed, self.character, self.ascension, attempt);
            match CombatTrainingCatalog::standard_from_json(&bytes) {
                Ok(_) => {
                    let mut driver = CatalogCombat::standard_from_json(&bytes, self.character, self.ascension)?;
                    let id = "position-seed";
                    driver.select_entry_id(id)?;
                    self.catalog_json = Some(bytes);
                    self.entry_id = Some(id.to_owned());
                    self.inner = Some(driver);
                    return Ok(());
                }
                Err(RunSessionError::Input(_) | RunSessionError::UnsupportedBoundary(_)) => self.rejected += 1,
                Err(error) => return Err(DriverError::fault(format!("unexpected Standard position catalog error: {error:?}"))),
            }
        }
        Err(DriverError::fault(format!("no valid position in {MAX_ATTEMPTS} candidates (rejected={})", self.rejected)))
    }

    fn inner(&self) -> Result<&CatalogCombat, DriverError> {
        self.inner.as_ref().ok_or_else(|| DriverError::fault("position driver has not been reset"))
    }
    fn inner_mut(&mut self) -> Result<&mut CatalogCombat, DriverError> {
        self.inner.as_mut().ok_or_else(|| DriverError::fault("position driver has not been reset"))
    }
}

impl Driver for StandardPositionCombat {
    fn class(&self) -> &'static str {
        match (self.character, self.ascension) {
            (Character::Ironclad, 0) => "standard-position-ironclad-a0",
            (Character::Ironclad, 20) => "standard-position-ironclad-a20",
            (Character::Silent, 0) => "standard-position-silent-a0",
            (Character::Silent, 20) => "standard-position-silent-a20",
            (Character::Defect, 0) => "standard-position-defect-a0",
            (Character::Defect, 20) => "standard-position-defect-a20",
            (Character::Watcher, 0) => "standard-position-watcher-a0",
            (Character::Watcher, 20) => "standard-position-watcher-a20",
            _ => "standard-position-unsupported-spec",
        }
    }
    fn describe(&self) -> Value {
        let mut config = self.inner.as_ref().map(Driver::describe).unwrap_or_else(|| json!({"driver":"standard-position"}));
        config["driver"] = json!("standard-position");
        config["position_spec"] = json!(self.class());
        config["rejected_candidates"] = json!(self.rejected);
        config
    }
    fn fresh(&self) -> Box<dyn Driver> {
        let mut fresh = Self::new(self.character, self.ascension);
        if let Some(bytes) = &self.catalog_json {
            if let Ok(mut driver) = CatalogCombat::standard_from_json(bytes, self.character, self.ascension) {
                if let Some(id) = &self.entry_id { let _ = driver.select_entry_id(id); }
                fresh.catalog_json = Some(bytes.clone());
                fresh.entry_id = self.entry_id.clone();
                fresh.inner = Some(driver);
                fresh.rejected = self.rejected;
            }
        }
        Box::new(fresh)
    }
    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        self.rejected = 0;
        self.generate(seed)?;
        self.inner_mut()?.reset(seed)
    }
    fn decision(&self) -> Result<Decision, DriverError> { self.inner()?.decision() }
    fn step(&mut self, index: usize) -> Result<StepOut, DriverError> { self.inner_mut()?.step(index) }
    fn choose_action(&self, decision: &Decision, policy: crate::Policy, seed: u64, step: usize) -> Option<usize> {
        self.inner.as_ref()?.choose_action(decision, policy, seed, step)
    }
    fn preflight(&self, index: usize) -> Result<(), DriverError> { self.inner()?.preflight(index) }
    fn probe_illegal(&mut self, probe: IllegalProbe) -> Result<(), DriverError> { self.inner_mut()?.probe_illegal(probe) }
    fn descriptor_stable(&self) -> Result<(), DriverError> { self.inner()?.descriptor_stable() }
    fn stability_identity(&self) -> Value { self.inner.as_ref().map(Driver::stability_identity).unwrap_or(Value::Null) }
    fn reached_combat(&self) -> bool { self.inner.as_ref().is_some_and(Driver::reached_combat) }
    fn diagnostic(&self) -> Value { self.inner.as_ref().map(Driver::diagnostic).unwrap_or(Value::Null) }
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = self.inner.as_ref().map(Driver::labels).unwrap_or_default();
        labels.insert("spec".into(), self.class().into());
        labels.insert("ascension".into(), self.ascension.to_string());
        labels.insert("character".into(), self.character.as_str().into());
        labels.insert("rejected_candidates".into(), self.rejected.to_string());
        labels
    }
    fn shrink_candidates(&self) -> Vec<Box<dyn Driver>> { Vec::new() }
    fn validate(&self) -> Result<(), DriverError> { self.inner()?.validate() }
    fn all_legal_on_clones(&self) -> Result<(), DriverError> { self.inner()?.all_legal_on_clones() }
    #[cfg(feature = "diagnostics")]
    fn inner_run(&self) -> Option<spire_core::run::Run> { self.inner.as_ref()?.inner_run() }
    #[cfg(feature = "diagnostics")]
    fn draw_knowledge_check(&self) -> Option<Result<Option<usize>, DriverError>> { self.inner.as_ref()?.draw_knowledge_check() }
}

fn position_catalog(seed: u64, character: Character, ascension: u8, attempt: usize) -> Vec<u8> {
    let mut rng = PositionRng(mix64(seed ^ (attempt as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93) ^ 0xC5F0_77A1));
    let mut catalog: Value = serde_json::from_slice(&crate::generated_standard_catalog_json(seed, character, ascension)).expect("generated Standard catalog JSON");
    let act = 1 + rng.below(4) as u8;
    let encounters = A0_ENCOUNTER_SETUPS.iter().filter(|(_, encounter_act, _)| *encounter_act == act).collect::<Vec<_>>();
    let (encounter, _, kind) = encounters[rng.below(encounters.len() as u64) as usize];
    let standard = Catalog::standard();
    let starter = standard.starter_deck(character).expect("Standard starter deck");
    let starter_relic = standard.starter_relic(character).expect("Standard starter relic");
    let mut deck = starter.iter().map(|card| *card).collect::<Vec<_>>();
    let extra_pool = standard.cards().iter().map(|row| row.id).filter(|card| complete::supports_permanent(*card)).collect::<Vec<_>>();
    let extras = rng.below(5) as usize;
    for _ in 0..extras { deck.push(extra_pool[rng.below(extra_pool.len() as u64) as usize]); }

    let mut rows = deck.iter().map(|card| json!({"card":card.as_str(),"upgrades":0,"misc":0,"generated":false})).collect::<Vec<_>>();
    let generated_pool = generated_cards(character);
    for _ in 0..rng.below(4) {
        let card = generated_pool[rng.below(generated_pool.len() as u64) as usize];
        rows.push(json!({"card":card.as_str(),"upgrades":0,"misc":0,"generated":true}));
    }
    rng.shuffle(&mut rows);
    let mut piles = json!({"hand": [], "draw_top": [], "draw": [], "discard": [], "exhaust": []});
    for row in rows {
        let mut destination = rng.below(5) as usize;
        if row["card"] == "void" && destination == 0 { destination = 1 + rng.below(4) as usize; }
        if piles["hand"].as_array().unwrap().len() >= 10 && destination == 0 { destination = 1 + rng.below(4) as usize; }
        piles.as_object_mut().unwrap().get_mut(["hand", "draw_top", "draw", "discard", "exhaust"][destination])
            .unwrap().as_array_mut().unwrap().push(row);
    }

    let mut position = json!({"player":{},"piles":piles});
    let mut player = serde_json::Map::new();
    let mut power_names = PLAYER_POWERS.to_vec();
    rng.shuffle(&mut power_names);
    let mut powers = serde_json::Map::new();
    for name in power_names.into_iter().take(rng.below(5) as usize) {
        let mut amount = 1 + rng.below(3) as i32;
        if matches!(name, "strength" | "dexterity" | "focus") && rng.below(3) == 0 { amount = -amount; }
        powers.insert(name.into(), json!(amount));
    }
    player.insert("powers".into(), json!(powers));
    if rng.below(2) == 0 { player.insert("block".into(), json!(rng.below(11))); }
    if rng.below(2) == 0 { player.insert("energy".into(), json!(rng.below(7))); }
    if character == Character::Defect && rng.below(2) == 0 {
        let slots = 1 + rng.below(5) as usize;
        let orb_kinds = ["lightning", "frost", "dark", "plasma"];
        let orbs = (0..rng.below(slots as u64 + 1)).map(|_| json!(orb_kinds[rng.below(orb_kinds.len() as u64) as usize])).collect::<Vec<_>>();
        player.insert("orb_slots".into(), json!(slots));
        player.insert("orbs".into(), json!(orbs));
    }
    if character == Character::Watcher {
        if rng.below(2) == 0 { player.insert("stance".into(), json!(["neutral", "wrath", "calm", "divinity"][rng.below(4) as usize])); }
        if rng.below(2) == 0 { player.insert("mantra".into(), json!(rng.below(10))); }
    }
    position["player"] = Value::Object(player);

    let mut burning = Value::Null;
    if FIXED_FORMATIONS.contains(&encounter.as_str()) && rng.below(2) == 0 {
        let formation_size = formation_size(encounter.as_str());
        let count = 1 + rng.below(formation_size.min(2) as u64) as usize;
        let mut edits = Vec::new();
        let mut slots = BTreeSet::new();
        while edits.len() < count {
            let slot = rng.below(formation_size as u64) as u8;
            if !slots.insert(slot) { continue; }
            let mut powers = serde_json::Map::new();
            if rng.below(2) == 0 { powers.insert(["strength", "vulnerable", "weak", "poison"][rng.below(4) as usize].into(), json!(1 + rng.below(3))); }
            edits.push(json!({"slot":slot,"hp":1 + rng.below(8),"block":rng.below(6),"powers":powers}));
        }
        // A burning buff as native: the entry declares it and every member lists it with one amount.
        if *kind == EncounterKind::Elite && rng.below(3) == 0 {
            let (power, buff) = [("metallicize", "metallicize"), ("regenerate", "regeneration")][rng.below(2) as usize];
            let amount = json!(1 + rng.below(3));
            for slot in 0..formation_size as u8 {
                if !slots.contains(&slot) { edits.push(json!({"slot": slot, "powers": {}})); }
                let edit = edits.iter_mut().find(|e| e["slot"] == json!(slot)).unwrap();
                edit["powers"][power] = amount.clone();
            }
            burning = json!(buff);
        }
        position["enemies"] = json!(edits);
    }

    let entry = &mut catalog["entries"][0];
    entry["id"] = json!("position-seed");
    entry["encounter"] = json!(encounter.as_str());
    entry["burning"] = burning;
    entry["deck"] = json!(deck.iter().map(|card| json!({"card":card.as_str(),"upgrades":0,"misc":0})).collect::<Vec<_>>());
    entry["relics"] = json!([starter_relic.as_str()]);
    entry.as_object_mut().unwrap().remove("bottled_cards");
    if let Some(potions) = entry["potions"].as_array_mut() { for potion in potions { *potion = Value::Null; } }
    entry["relic_counters"] = json!({});
    entry["position"] = position;
    catalog["entries"] = json!([entry.clone()]);
    serde_json::to_vec(&catalog).expect("position catalog serialization")
}

fn generated_cards(character: Character) -> Vec<CardId> {
    let mut cards = vec![CardId::Wound, CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Void];
    match character {
        Character::Silent => cards.push(CardId::Shiv),
        Character::Watcher => cards.extend([CardId::Miracle, CardId::Insight, CardId::Smite]),
        _ => {}
    }
    cards
}

fn formation_size(encounter: &str) -> usize {
    match encounter {
        "three_cultists" | "jaw_worm_horde" | "three_sentries" | "three_byrds" | "slavers"
        | "three_darklings" | "reptomancer" | "shield_and_spear" | "colosseum_slavers"
        | "masked_bandits" => 3,
        "chosen_and_byrds" | "centurion_and_healer" | "collector" | "donu_and_deca"
        | "two_thieves" | "sentry_and_sphere" | "cultist_and_chosen" | "shelled_parasite_and_fungi"
        | "colosseum_nobs" | "two_orb_walkers" | "two_fungi_beasts" => 2,
        _ => 1,
    }
}

struct PositionRng(u64);
impl PositionRng {
    fn next(&mut self) -> u64 { self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15); mix64(self.0) }
    fn below(&mut self, bound: u64) -> u64 { if bound == 0 { 0 } else { self.next() % bound } }
    fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() { let j = self.below((i + 1) as u64) as usize; values.swap(i, j); }
    }
}
