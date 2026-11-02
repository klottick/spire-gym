use std::collections::BTreeMap;
use serde_json::{Value, json};
use spire_core::{
    content::{CardId, CardRarity, Catalog, Character, ContentProducer, EncounterId, PotionId, RelicId},
    run::{AuthoredRunStart, CardInstance, GameAction, Run, RunError, TRAINING_ENCOUNTERS_V2, TRAINING_POTIONS, TRAINING_RELICS},
};
use spire_env::{RunFeatureGroup, RunObservation};
use crate::{Decision, Driver, DriverError, IllegalProbe, StepOut, mix64};

const COMPLETE_PUBLIC_SCHEMA: &str = "spire-fuzz-complete-public-v1";

pub struct CompleteCombat {
    character: Character,
    encounter: EncounterId,
    run: Option<Run>,
    seed: Option<u64>,
    start: Option<AuthoredRunStart>,
    start_value: Option<Value>,
    decision: Option<Decision>,
    publication: u64,
}

impl CompleteCombat {
    pub fn new(character: Character, encounter: EncounterId) -> Self {
        Self { character, encounter, run: None, seed: None, start: None, start_value: None, decision: None, publication: 0 }
    }

    pub fn from_repro(character: Character, encounter: EncounterId, start: AuthoredRunStart) -> Self {
        let value = start_json(&start);
        Self { character, encounter, run: None, seed: None, start: Some(start), start_value: Some(value), decision: None, publication: 0 }
    }

    fn encode(&self, decision: &spire_core::run::PublicDecision) -> Result<RunObservation, DriverError> {
        let catalog = Catalog::standard();
        let cards = catalog.cards();
        let mut context = vec![character_index(self.character), i64::from(decision.act), i64::from(decision.floor), i64::from(decision.terminal.is_some())];
        let mut context_group = group("context", vec![4], context.drain(..).collect(), 4, None);
        context_group.known.fill(1);
        let player = group("player", vec![4], vec![i64::from(decision.player.hp), i64::from(decision.player.max_hp), i64::from(decision.player.gold), decision.player.keys.iter().filter(|key| **key).count() as i64], 4, None);
        let mut permanent = Vec::with_capacity(decision.objects.len() * 2);
        for object in &decision.objects {
            let card_index = cards.iter().position(|row| row.id == object.id).ok_or_else(|| DriverError::new("encode", format!("unknown public card {}", object.id.as_str())))?;
            permanent.extend([card_index as i64 + 1, i64::from(object.upgrades)]);
        }
        let permanent_group = group("permanent_cards", vec![decision.objects.len(), 2], permanent, 2, Some(vec![1; decision.objects.len()]));
        let mut actions = Vec::with_capacity(decision.candidates.len() * 8);
        for candidate in &decision.candidates {
            let verb = verb(&candidate.action);
            if verb < 0 { return Err(DriverError::new("encode", format!("unsupported public action {:?}", candidate.action))); }
            actions.extend([verb, 0, 0, 0, 0, 0, 0, 0]);
        }
        let action_count = decision.candidates.len();
        let action_group = group("actions", vec![action_count, 8], actions, 8, Some(vec![1; action_count]));
        let observation = RunObservation { groups: vec![context_group, player, permanent_group, action_group], action_mask: vec![1; action_count] };
        // Exercise the harness's actual serializer on all public numeric groups.
        serde_json::to_vec(&crate::observation::observation_json(&observation))
            .map_err(|error| DriverError::new("encode", error.to_string()))?;
        Ok(observation)
    }

    fn current_decision(&self) -> Result<Decision, DriverError> {
        self.decision.clone().ok_or_else(|| DriverError::fault("complete combat has not been reset"))
    }
}

impl Driver for CompleteCombat {
    fn class(&self) -> &'static str { "complete" }
    fn describe(&self) -> Value {
        json!({"driver":"complete","schema":COMPLETE_PUBLIC_SCHEMA,
            "encode":"harness projection (spire-fuzz-complete-public-v1)","fingerprint":"n/a",
            "character":character_name(self.character),
            "encounter":self.encounter.as_str(),"generator":"complete-pools-v1","start":self.start_value})
    }
    fn fresh(&self) -> Box<dyn Driver> {
        match &self.start {
            Some(start) => Box::new(Self::from_repro(self.character, self.encounter, start.clone())),
            None => Box::new(Self::new(self.character, self.encounter)),
        }
    }
    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        let start = self.start.clone().unwrap_or_else(|| generate_start(seed, self.character));
        let value = start_json(&start);
        self.start = Some(start.clone());
        self.start_value = Some(value.clone());
        let run = match spire_core::run::diagnostics::complete_combat(self.character, start.clone(), self.encounter) {
            Ok(run) => run,
            Err(RunError::Unsupported(message)) => return Err(DriverError::new("unsupported", message)),
            Err(RunError::InvalidAction) => return Err(DriverError::new("unsupported", "complete supplied resources or encounter are unsupported")),
            Err(error @ RunError::Faulted(_)) => return Err(DriverError::fault(format!("complete combat startup: {error:?}"))),
            Err(error) => return Err(DriverError::new("unsupported", format!("complete combat startup rejected: {error:?}"))),
        };
        let decision = run.decision().map_err(|error| DriverError::fault(format!("complete decision after reset: {error:?}")))?;
        let observation = self.encode(&decision)?;
        self.seed = Some(seed); self.publication = 1;
        let boundary = Decision { observation, token: [seed, self.publication], terminal: settled_combat_outcome(&decision) };
        self.run = Some(run); self.decision = Some(boundary.clone());
        Ok(boundary)
    }
    fn decision(&self) -> Result<Decision, DriverError> { self.current_decision() }
    fn step(&mut self, index: usize) -> Result<StepOut, DriverError> {
        let run = self.run.as_mut().ok_or_else(|| DriverError::fault("complete Run is missing"))?;
        let current = run.decision().map_err(|error| DriverError::fault(format!("complete decision: {error:?}")))?;
        let action = current.candidates.get(index).ok_or_else(|| DriverError::fault(format!("candidate index {index} is outside {} actions", current.candidates.len())))?.action.clone();
        let transition = run.step(&action).map_err(|error| DriverError::fault(format!("Run::step index={index} action={action:?}: {error:?}")))?;
        let next = run.decision().map_err(|error| DriverError::fault(format!("complete post-step decision: {error:?}")))?;
        self.publication = self.publication.saturating_add(1);
        let token = [self.seed.unwrap_or_default(), self.publication];
        let observation = self.encode(&next)?;
        let decision = Decision { observation, token, terminal: settled_combat_outcome(&next) };
        self.decision = Some(decision.clone());
        let terminal = decision.terminal.clone().or_else(|| transition.terminal.map(|terminal| format!("{terminal:?}")));
        Ok(StepOut { decision, reward: transition.reward, terminal })
    }
    fn preflight(&self, index: usize) -> Result<(), DriverError> {
        let run = self.run.as_ref().ok_or_else(|| DriverError::new("legal_rejected", "complete Run is missing"))?;
        let decision = run.decision().map_err(|error| DriverError::new("legal_rejected", format!("complete decision: {error:?}")))?;
        if index < decision.candidates.len() { Ok(()) } else { Err(DriverError::new("legal_rejected", format!("advertised index {index} exceeds candidate list"))) }
    }
    fn probe_illegal(&mut self, _probe: IllegalProbe) -> Result<(), DriverError> {
        let run = self.run.as_mut().ok_or_else(|| DriverError::new("probe_rejected", "complete Run is missing"))?;
        let before = run.clone();
        let action = GameAction::Choose(spire_core::run::ChoiceId(u64::MAX));
        match run.step(&action) {
            Err(RunError::InvalidAction) => {},
            Err(error) => return Err(DriverError::new("probe_rejected", format!("unexpected invalid-action result {error:?}"))),
            Ok(_) => return Err(DriverError::new("probe_accepted", "invalid complete action was accepted")),
        }
        if *run != before { return Err(DriverError::new("probe_corrupts", "invalid complete action changed the Run")); }
        Err(DriverError::new("probe_rejected", "invalid complete action was rejected without changing the Run"))
    }
    fn descriptor_stable(&self) -> Result<(), DriverError> {
        self.run.as_ref().map(|_| ()).ok_or_else(|| DriverError::fault("complete Run is missing"))
    }
    fn diagnostic(&self) -> Value {
        json!({"seed":self.seed,"character":character_name(self.character),"encounter":self.encounter.as_str(),
            "decision":self.run.as_ref().and_then(|run|run.decision().ok()).map(|decision| format!("{decision:?}")),
            "start":self.start_value})
    }
    fn reached_combat(&self) -> bool { self.run.is_some() }
    fn labels(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("profile".into(), COMPLETE_PUBLIC_SCHEMA.into()), ("character".into(), character_name(self.character).into()), ("encounter".into(), self.encounter.as_str().into())])
    }
    fn shrink_candidates(&self) -> Vec<Box<dyn Driver>> {
        let Some(start) = self.start.as_ref() else { return Vec::new() };
        let mut starts = Vec::new();
        for index in (0..start.deck.len()).rev() {
            if start.deck.len() <= 1 { break; }
            let mut candidate = start.clone();
            candidate.deck.remove(index);
            starts.push(candidate);
        }
        for index in 0..start.deck.len() {
            if start.deck[index].upgrades == 0 { continue; }
            let mut candidate = start.clone();
            candidate.deck[index].upgrades = 0;
            starts.push(candidate);
        }
        for index in (0..start.relics.len()).rev() {
            let mut candidate = start.clone();
            candidate.relics.remove(index);
            starts.push(candidate);
        }
        for index in 0..start.potions.len() {
            if start.potions[index].is_none() { continue; }
            let mut candidate = start.clone();
            candidate.potions[index] = None;
            starts.push(candidate);
        }
        if start.hp > 1 || start.max_hp > 1 {
            let mut candidate = start.clone();
            candidate.hp = 1;
            candidate.max_hp = 1;
            starts.push(candidate);
        }
        if start.gold > 0 {
            let mut candidate = start.clone();
            candidate.gold = 0;
            starts.push(candidate);
        }
        if !start.relic_counters.is_empty() {
            let mut candidate = start.clone();
            candidate.relic_counters.clear();
            starts.push(candidate);
        }
        starts.into_iter().map(|candidate| Box::new(Self::from_repro(self.character, self.encounter, candidate)) as Box<dyn Driver>).collect()
    }
    fn validate(&self) -> Result<(), DriverError> {
        self.run.as_ref().ok_or_else(|| DriverError::new("validator", "complete Run is missing"))?.fuzz_validate()
            .map_err(|message| DriverError::new("validator", message))
    }
    fn all_legal_on_clones(&self) -> Result<(), DriverError> {
        let run = self.run.as_ref().ok_or_else(|| DriverError::new("legal_rejected", "complete Run is missing"))?;
        let decision = run.decision().map_err(|error| DriverError::new("legal_rejected", format!("complete decision: {error:?}")))?;
        let published = self.current_decision()?;
        if decision.candidates.len() != published.observation.action_mask.iter().filter(|mask| **mask != 0).count() {
            return Err(DriverError::new("legal_rejected", "complete encoder action mask differs from Run candidates"));
        }
        for (index, candidate) in decision.candidates.iter().enumerate() {
            let mut clone = run.clone();
            if let Err(error) = clone.step(&candidate.action) {
                if !matches!(error, RunError::Terminal) { return Err(DriverError::new("legal_rejected", format!("advertised candidate {index} rejected: {error:?}"))); }
            }
            clone.fuzz_validate().map_err(|message| DriverError::new("validator", format!("clone after candidate {index}: {message}")))?;
        }
        Ok(())
    }
    #[cfg(feature = "diagnostics")]
    fn inner_run(&self) -> Option<Run> { self.run.clone() }
}

/// A CompleteCombat episode ends when its one combat settles. The shared Run
/// intentionally leaves the training Run nonterminal after a victory, so its
/// settled result is the episode terminal even though the public Run has no
/// further actions.
fn settled_combat_outcome(decision: &spire_core::run::PublicDecision) -> Option<String> {
    if let Some(terminal) = decision.terminal { return Some(format!("{terminal:?}")); }
    if decision.combat.is_none() {
        return decision.last_combat.as_ref().map(|record| format!("{:?}", record.outcome));
    }
    None
}

pub fn complete_cases_for_smoke() -> Vec<(Character, EncounterId)> {
    let acts = [1, 2, 3];
    let encounters = acts.map(|act| TRAINING_ENCOUNTERS_V2.iter().find(|(_, candidate_act, _)| *candidate_act == act).map(|(encounter, _, _)| *encounter));
    Character::ALL.iter().flat_map(|character| encounters.iter().flatten().copied().map(move |encounter| (*character, encounter))).collect()
}

fn generate_start(seed: u64, character: Character) -> AuthoredRunStart {
    let catalog = Catalog::standard();
    let own = catalog.selectable_pool(ContentProducer::OrdinaryReward, character).unwrap_or_default();
    let any = catalog.selectable_pool(ContentProducer::ShardOrdinaryReward, character).unwrap_or_default();
    let off = any.into_iter().filter(|card| !own.contains(card)).collect::<Vec<_>>();
    let colorless = spire_core::content::pools::cards(spire_core::content::pools::CardPool::Colourless { rarity: None }).unwrap_or_default();
    let curses = spire_core::content::pools::cards(spire_core::content::pools::CardPool::Curse).unwrap_or_default();
    let _generated_statuses = catalog.cards().iter().filter(|definition| definition.rarity == CardRarity::Status
        && spire_core::content::complete::supports_generated(definition.id)).map(|definition| definition.id).collect::<Vec<_>>();
    let mut deck = Vec::new();
    let starter = catalog.starter_deck(character).unwrap_or(&[]);
    for card in starter { deck.push(card_instance(*card, deck.len() as u64 + 1, 0)); }
    let mut add = |card: CardId, upgrades: u32| {
        let instance = deck.len() as u64 + 1;
        deck.push(card_instance(card, instance, upgrades));
    };
    if let Some(card) = choose(&own, mix64(seed ^ 0x01)) { add(card, 0); }
    if let Some(card) = choose(&off, mix64(seed ^ 0x02)) { add(card, 0); }
    if let Some(card) = choose(&colorless, mix64(seed ^ 0x03)) { add(card, 0); }
    if let Some(card) = choose(&curses, mix64(seed ^ 0x04)) { add(card, 0); }
    // These standard permanent cards create real Wound and Dazed status cards in combat.
    for card in [CardId::WildStrike, CardId::RecklessCharge] {
        if spire_core::content::complete::supports_permanent(card) { add(card, 0); }
    }
    // Searing Blow's engine rule accepts any u32 upgrade value; sample a small bounded range.
    let searing_upgrades = (mix64(seed ^ 0x05) % 4) as u32;
    if spire_core::content::complete::supports_permanent(CardId::SearingBlow) { add(CardId::SearingBlow, searing_upgrades); }
    if !deck.is_empty() && mix64(seed ^ 0x06) & 1 != 0 { deck[0].upgrades = 1; }
    let relics = relic_pool(character);
    let starter_relic = catalog.starter_relic(character);
    let relic = choose(&relics, mix64(seed ^ 0x07)).or(starter_relic).unwrap_or(RelicId::BurningBlood);
    let potions = potion_pool(character);
    let potion = choose(&potions, mix64(seed ^ 0x08));
    AuthoredRunStart { hp: base_hp(character), max_hp: base_hp(character), gold: 99, deck,
        relics: vec![relic], potions: vec![potion, None, None], relic_counters: BTreeMap::new() }
}

fn relic_pool(character: Character) -> Vec<RelicId> {
    match character {
        Character::Ironclad => TRAINING_RELICS.to_vec(),
        Character::Silent => spire_core::run::M15_SILENT_HEART_RELICS.to_vec(),
        Character::Defect => spire_core::run::M20_DEFECT_HEART_RELICS.to_vec(),
        Character::Watcher => spire_core::run::M25_WATCHER_HEART_RELICS.to_vec(),
    }
}
fn potion_pool(character: Character) -> Vec<PotionId> {
    match character {
        Character::Ironclad => TRAINING_POTIONS.to_vec(),
        Character::Silent => spire_core::run::M15_SILENT_HEART_POTIONS.to_vec(),
        Character::Defect => spire_core::run::M20_DEFECT_HEART_POTIONS.to_vec(),
        Character::Watcher => spire_core::run::M25_WATCHER_HEART_POTIONS.to_vec(),
    }
}
fn choose<T: Copy>(pool: &[T], random: u64) -> Option<T> { (!pool.is_empty()).then(|| pool[random as usize % pool.len()]) }
fn card_instance(id: CardId, instance: u64, upgrades: u32) -> CardInstance {
    let misc = match id { CardId::GeneticAlgorithm => 1, CardId::RitualDagger => 15, _ => 0 };
    CardInstance { id, instance, upgrades, misc }
}
fn base_hp(character: Character) -> u32 {
    match character { Character::Ironclad => 80, Character::Silent => 70, Character::Defect => 75, Character::Watcher => 72 }
}
fn character_index(character: Character) -> i64 {
    match character { Character::Ironclad => 1, Character::Silent => 2, Character::Defect => 3, Character::Watcher => 4 }
}
fn character_name(character: Character) -> &'static str {
    match character { Character::Ironclad => "ironclad", Character::Silent => "silent", Character::Defect => "defect", Character::Watcher => "watcher" }
}
fn group(key: &'static str, shape: Vec<usize>, values: Vec<i64>, columns: usize, occupied: Option<Vec<i8>>) -> RunFeatureGroup {
    let rows = shape.first().copied().unwrap_or(1);
    RunFeatureGroup { key, shape, known: vec![1; rows.saturating_mul(columns)], values, occupied }
}
pub(crate) fn verb(action: &GameAction) -> i64 {
    match action {
        GameAction::Move(_) => 1, GameAction::Choose(_) => 2, GameAction::Select(_) => 3,
        GameAction::Confirm => 4, GameAction::Cancel => 5, GameAction::UsePotion { .. } => 6,
        GameAction::DiscardPotion { .. } => 7,
        GameAction::Combat(action) => match action {
            spire_core::combat::CombatAction::ChooseStance { .. } => 14,
            spire_core::combat::CombatAction::DeselectCard { .. } => 15,
            spire_core::combat::CombatAction::PlayCard { .. } => 8,
            spire_core::combat::CombatAction::SelectCard { .. } => 9,
            spire_core::combat::CombatAction::ConfirmSelection => 11,
            spire_core::combat::CombatAction::UsePotion { .. } => 6,
            spire_core::combat::CombatAction::DiscardPotion { .. } => 7,
            spire_core::combat::CombatAction::EndTurn => 10,
            spire_core::combat::CombatAction::ChooseCardOffer { .. } => 12,
            spire_core::combat::CombatAction::SkipCardOffer => 13,
        },
    }
}

fn start_json(start: &AuthoredRunStart) -> Value {
    json!({"hp":start.hp,"max_hp":start.max_hp,"gold":start.gold,
        "deck":start.deck.iter().map(|card|json!({"id":card.id.as_str(),"instance":card.instance,"upgrades":card.upgrades,"misc":card.misc})).collect::<Vec<_>>(),
        "relics":start.relics.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "potions":start.potions.iter().map(|id|id.map(PotionId::as_str)).collect::<Vec<_>>(),
        "relic_counters":start.relic_counters.iter().map(|(id,value)|(id.as_str().to_owned(),*value)).collect::<BTreeMap<_,_>>()})
}
