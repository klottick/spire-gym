use std::{cell::Cell, collections::BTreeMap};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spire_core::{
    content::{Character, PotionId, RelicId},
    run::{GameAction, Run, RunError},
};
use spire_core::run::diagnostics::CompleteItemSupply;
use spire_env::RunObservation;
use crate::{Decision, Driver, DriverError, IllegalProbe, Policy, StepOut, mix64, profiles::CompleteItemsRunProfile};

const MAX_REPEATED_BOUNDARY_FALLBACKS: usize = 8;

pub struct CompleteItemsRun {
    profile: CompleteItemsRunProfile,
    supply: Option<CompleteItemSupply>,
    run: Option<Run>,
    seed: Option<u64>,
    decision: Option<Decision>,
    saw_combat: bool,
    action_kinds: Vec<i64>,
    public_visits: BTreeMap<[u8; 32], usize>,
    publication: u64,
    descriptor_seal: Option<[u8; 32]>,
    fallback_streak: Cell<usize>,
    fallback_livelock: Cell<bool>,
}

impl CompleteItemsRun {
    pub fn new(profile: CompleteItemsRunProfile) -> Self {
        Self { profile, supply: None, run: None, seed: None, decision: None,
            action_kinds: Vec::new(), public_visits: BTreeMap::new(), publication: 0, descriptor_seal: None, saw_combat: false,
            fallback_streak: Cell::new(0), fallback_livelock: Cell::new(false) }
    }

    pub fn from_repro(profile: CompleteItemsRunProfile, supply: CompleteItemSupply) -> Self {
        let mut driver = Self::new(profile);
        driver.supply = Some(supply);
        driver
    }

    fn observation(&mut self, decision: &spire_core::run::PublicDecision) -> Decision {
        self.action_kinds = decision.candidates.iter().map(|candidate| crate::complete::verb(&candidate.action)).collect();
        Decision {
            // No M27 numeric profile encoder exists. The harness carries only the
            // legal mask and uses the candidate kinds directly for policy selection.
            observation: RunObservation { groups: Vec::new(), action_mask: vec![1; decision.candidates.len()] },
            token: [self.seed.unwrap_or_default(), self.publication],
            terminal: decision.terminal.map(|terminal| format!("{terminal:?}")),
        }
    }

    fn current(&self) -> Result<Decision, DriverError> {
        self.decision.clone().ok_or_else(|| DriverError::fault("CompleteItemsRun has not been reset"))
    }

    fn supply_json(&self) -> Value {
        self.supply.as_ref().map(|supply| json!({
            "relics": supply.relics.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
            "potions": supply.potions.iter().map(|id| id.map(PotionId::as_str)).collect::<Vec<_>>(),
            "counters": supply.counters.iter().map(|(id, value)| (id.as_str(), *value)).collect::<BTreeMap<_, _>>(),
        })).unwrap_or(Value::Null)
    }
}

impl Driver for CompleteItemsRun {
    fn class(&self) -> &'static str { "items" }

    fn describe(&self) -> Value {
        json!({"driver":"items","profile":self.profile.id,"character":self.profile.character.as_str(),
            "starts_at":"Act4 map","neow":false,"encode":"n/a","generator":"engine-complete-catalog-v1",
            "supply":self.supply_json()})
    }

    fn fresh(&self) -> Box<dyn Driver> {
        match self.supply.as_ref() {
            Some(supply) => Box::new(Self::from_repro(self.profile, supply.clone())),
            None => Box::new(Self::new(self.profile)),
        }
    }

    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        let supply = self.supply.clone().unwrap_or_else(|| generate_supply(seed, self.profile.character));
        self.supply = Some(supply.clone());
        let run = match spire_core::run::diagnostics::complete_act4_run(self.profile.character, supply) {
            Ok(run) => run,
            Err(error @ RunError::Faulted(_)) => return Err(DriverError::fault(format!("complete items startup: {error:?}"))),
            Err(RunError::Unsupported(message)) => return Err(DriverError::new("unsupported", message)),
            Err(RunError::InvalidAction) => return Err(DriverError::new("unsupported", "complete item supply rejected")),
            Err(error) => return Err(DriverError::new("unsupported", format!("complete items startup rejected: {error:?}"))),
        };
        let public = run.decision().map_err(|error| DriverError::fault(format!("complete items decision after reset: {error:?}")))?;
        self.run = Some(run);
        self.saw_combat = public.combat.is_some();
        self.seed = Some(seed);
        self.publication = 1;
        self.fallback_streak.set(0);
        self.fallback_livelock.set(false);
        self.public_visits.clear();
        *self.public_visits.entry(public_decision_fingerprint(&public)).or_default() += 1;
        let decision = self.observation(&public);
        self.descriptor_seal = Some(Sha256::digest(serde_json::to_vec(&self.describe()).unwrap_or_default()).into());
        self.decision = Some(decision.clone());
        Ok(decision)
    }

    fn decision(&self) -> Result<Decision, DriverError> { self.current() }

    fn step(&mut self, index: usize) -> Result<StepOut, DriverError> {
        let run = self.run.as_mut().ok_or_else(|| DriverError::fault("complete items Run is missing"))?;
        let public = run.decision().map_err(|error| DriverError::fault(format!("complete items decision: {error:?}")))?;
        let action = public.candidates.get(index).ok_or_else(|| DriverError::fault(
            format!("candidate index {index} is outside {} actions", public.candidates.len())))?.action.clone();
        let transition = run.step(&action).map_err(|error| DriverError::fault(format!(
            "Run::step index={index} action={action:?}: {error:?}")))?;
        let next = run.decision().map_err(|error| DriverError::fault(format!("complete items post-step decision: {error:?}")))?;
        self.saw_combat |= next.combat.is_some();
        *self.public_visits.entry(public_decision_fingerprint(&next)).or_default() += 1;
        self.publication = self.publication.saturating_add(1);
        let decision = self.observation(&next);
        let terminal = decision.terminal.clone().or_else(|| transition.terminal.map(|terminal| format!("{terminal:?}")));
        self.decision = Some(decision.clone());
        Ok(StepOut { decision, reward: transition.reward, terminal })
    }

    fn choose_action(&self, decision: &Decision, policy: Policy, seed: u64, step: usize) -> Option<usize> {
        let repeated = policy == Policy::ChooseNone && self.run.as_ref().and_then(|run| run.decision().ok())
            .and_then(|public| self.public_visits.get(&public_decision_fingerprint(&public)).copied())
            .is_some_and(|visits| visits > 1);
        let streak = next_fallback_streak(self.fallback_streak.get(), repeated);
        self.fallback_streak.set(streak);
        self.fallback_livelock.set(fallback_limit_exceeded(streak));
        let policy = if repeated { Policy::First } else { policy };
        crate::policy::choose_action_with_kinds(&decision.observation.action_mask, &self.action_kinds, policy, seed, step)
    }

    fn policy_guard(&self) -> Option<DriverError> {
        self.fallback_livelock.get().then(|| DriverError::new(
            "livelock",
            format!("ChooseNone repeated-boundary fallback fired more than {MAX_REPEATED_BOUNDARY_FALLBACKS} consecutive times"),
        ))
    }

    fn preflight(&self, index: usize) -> Result<(), DriverError> {
        let run = self.run.as_ref().ok_or_else(|| DriverError::new("legal_rejected", "complete items Run is missing"))?;
        let decision = run.decision().map_err(|error| DriverError::new("legal_rejected", format!("complete items decision: {error:?}")))?;
        if index < decision.candidates.len() { Ok(()) }
        else { Err(DriverError::new("legal_rejected", format!("advertised index {index} exceeds candidate list"))) }
    }

    fn probe_illegal(&mut self, _probe: IllegalProbe) -> Result<(), DriverError> {
        let run = self.run.as_mut().ok_or_else(|| DriverError::new("probe_rejected", "complete items Run is missing"))?;
        let before = run.clone();
        match run.step(&GameAction::Choose(spire_core::run::ChoiceId(u64::MAX))) {
            Err(RunError::InvalidAction) => {},
            Err(error) => return Err(DriverError::new("probe_rejected", format!("unexpected invalid-action result {error:?}"))),
            Ok(_) => return Err(DriverError::new("probe_accepted", "invalid complete items action was accepted")),
        }
        if *run != before { return Err(DriverError::new("probe_corrupts", "invalid action changed the complete items Run")); }
        Err(DriverError::new("probe_rejected", "invalid complete items action was rejected without changing the Run"))
    }

    fn descriptor_stable(&self) -> Result<(), DriverError> {
        let before = self.descriptor_seal.ok_or_else(|| DriverError::new("encode", "CompleteItemsRun descriptor seal missing"))?;
        let after: [u8; 32] = Sha256::digest(serde_json::to_vec(&self.describe()).unwrap_or_default()).into();
        if before == after { Ok(()) } else { Err(DriverError::new("encode", "CompleteItemsRun driver description changed")) }
    }
    fn stability_identity(&self) -> Value { json!({"driver_description":self.descriptor_seal}) }

    fn diagnostic(&self) -> Value {
        json!({"profile":self.profile.id,"character":self.profile.character.as_str(),"seed":self.seed,
            "supply":self.supply_json(),"decision":self.run.as_ref().and_then(|run|run.decision().ok()).map(|value|format!("{value:?}")),
            "run":self.run.as_ref().map(|run|format!("{run:?}")),
            "repeated_boundary_fallback_streak":self.fallback_streak.get(),
            "repeated_boundary_fallback_limit":MAX_REPEATED_BOUNDARY_FALLBACKS})
    }
    fn reached_combat(&self) -> bool { self.saw_combat }

    fn labels(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("profile".into(), self.profile.id.into()),
            ("character".into(), self.profile.character.as_str().into())])
    }

    fn shrink_candidates(&self) -> Vec<Box<dyn Driver>> {
        let Some(supply) = self.supply.as_ref() else { return Vec::new() };
        let mut candidates = Vec::new();
        for index in (0..supply.relics.len()).rev() {
            let mut next = supply.clone();
            let removed = next.relics.remove(index);
            next.counters.remove(&removed);
            if !next.relics.contains(&RelicId::PotionBelt) && next.potions.len() == 5 { next.potions.truncate(3); }
            candidates.push(Box::new(Self::from_repro(self.profile, next)) as Box<dyn Driver>);
        }
        for index in 0..supply.potions.len() {
            if supply.potions[index].is_none() { continue; }
            let mut next = supply.clone();
            next.potions[index] = None;
            candidates.push(Box::new(Self::from_repro(self.profile, next)) as Box<dyn Driver>);
        }
        for relic in supply.counters.keys() {
            let mut next = supply.clone();
            next.counters.remove(relic);
            candidates.push(Box::new(Self::from_repro(self.profile, next)) as Box<dyn Driver>);
        }
        candidates
    }

    fn validate(&self) -> Result<(), DriverError> {
        self.run.as_ref().ok_or_else(|| DriverError::new("validator", "complete items Run is missing"))?.fuzz_validate()
            .map_err(|message| DriverError::new("validator", message))
    }

    fn all_legal_on_clones(&self) -> Result<(), DriverError> {
        let run = self.run.as_ref().ok_or_else(|| DriverError::new("legal_rejected", "complete items Run is missing"))?;
        let decision = run.decision().map_err(|error| DriverError::new("legal_rejected", format!("complete items decision: {error:?}")))?;
        let published = self.current()?;
        if decision.candidates.len() != published.observation.action_mask.iter().filter(|mask| **mask != 0).count() {
            return Err(DriverError::new("legal_rejected", "complete items action mask differs from Run candidates"));
        }
        for (index, candidate) in decision.candidates.iter().enumerate() {
            let mut clone = run.clone();
            if let Err(error) = clone.step(&candidate.action) {
                if !matches!(error, RunError::Terminal) {
                    return Err(DriverError::new("legal_rejected", format!("advertised candidate {index} rejected: {error:?}")));
                }
            }
            clone.fuzz_validate().map_err(|message| DriverError::new("validator", format!("clone after candidate {index}: {message}")))?;
        }
        Ok(())
    }

    #[cfg(feature = "diagnostics")]
    fn inner_run(&self) -> Option<Run> { self.run.clone() }
}

fn public_decision_fingerprint(decision: &spire_core::run::PublicDecision) -> [u8; 32] {
    let room = decision.room.as_ref().map(|room| (room.kind, room.choices.iter()
        .map(|choice| (&choice.kind, choice.price, choice.status)).collect::<Vec<_>>()));
    let selection = decision.selection.as_ref().map(|selection| (selection.required, selection.selected_count,
        selection.eligible.iter().map(|object| (object.instance, object.id, object.upgrades, object.misc)).collect::<Vec<_>>()));
    let candidates = decision.candidates.iter().map(|candidate| (action_shape(&candidate.action), candidate.label,
        candidate.neow_offer.as_ref().map(|offer| format!("{offer:?}")), candidate.object.as_ref().map(|object|
            (object.instance, object.id, object.upgrades, object.misc)))).collect::<Vec<_>>();
    let identity = format!("{:?}|{:?}|{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        decision.content_profile, room, decision.reward_total, decision.act, decision.floor, decision.player,
        decision.combat, decision.combat_progress, decision.last_combat, decision.objects, candidates, selection);
    Sha256::digest(identity.as_bytes()).into()
}

fn action_shape(action: &GameAction) -> String {
    match action {
        GameAction::Move(_) => "move".into(),
        GameAction::Combat(action) => format!("combat:{action:?}"),
        GameAction::Choose(_) => "choose".into(),
        GameAction::Select(_) => "select".into(),
        GameAction::Confirm => "confirm".into(),
        GameAction::Cancel => "cancel".into(),
        GameAction::UsePotion { slot, target } => format!("use_potion:{slot}:{target:?}"),
        GameAction::DiscardPotion { slot } => format!("discard_potion:{slot}"),
    }
}

fn next_fallback_streak(previous: usize, repeated_boundary: bool) -> usize {
    if repeated_boundary { previous.saturating_add(1) } else { 0 }
}

fn fallback_limit_exceeded(streak: usize) -> bool { streak > MAX_REPEATED_BOUNDARY_FALLBACKS }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_none_fallback_becomes_livelock_after_eight_consecutive_repeats() {
        let mut streak = 0;
        for _ in 0..MAX_REPEATED_BOUNDARY_FALLBACKS {
            streak = next_fallback_streak(streak, true);
            assert!(!fallback_limit_exceeded(streak));
        }
        streak = next_fallback_streak(streak, true);
        assert_eq!(streak, MAX_REPEATED_BOUNDARY_FALLBACKS + 1);
        assert!(fallback_limit_exceeded(streak));
        assert_eq!(next_fallback_streak(streak, false), 0, "a fresh public boundary resets the streak");
    }
}

fn generate_supply(seed: u64, character: Character) -> CompleteItemSupply {
    let relic_catalog = spire_core::content::complete::relic_union(character);
    let potion_catalog = spire_core::content::complete::potion_union(character);
    let mut relics = Vec::new();
    let requested = 5 + (mix64(seed ^ 0xA076_1D64_78BD_642F) % 5) as usize;
    let offset = (mix64(seed ^ 0xE703_7ED1_A0B4_28DB) as usize) % relic_catalog.len();
    for delta in 0..relic_catalog.len() {
        let candidate = relic_catalog[(offset + delta) % relic_catalog.len()];
        if replacement_conflict(candidate, &relics) { continue; }
        relics.push(candidate);
        if relics.len() == requested { break; }
    }
    let counters = relics.iter().filter_map(|id| generated_counter(*id, mix64(seed ^ id.as_str().len() as u64)))
        .collect::<BTreeMap<_, _>>();
    let potion_slots = if relics.contains(&RelicId::PotionBelt) { 5 } else { 3 };
    let potion_offset = mix64(seed ^ 0xD6E8_FEB8_6659_FD93) as usize % potion_catalog.len();
    let potions = (0..potion_slots).map(|slot| Some(potion_catalog[(potion_offset + slot) % potion_catalog.len()])).collect();
    CompleteItemSupply { relics, potions, counters }
}

fn replacement_conflict(candidate: RelicId, selected: &[RelicId]) -> bool {
    const REPLACEMENTS: [(RelicId, RelicId); 4] = [
        (RelicId::BurningBlood, RelicId::BlackBlood),
        (RelicId::RingOfTheSnake, RelicId::RingOfTheSerpent),
        (RelicId::CrackedCore, RelicId::FrozenCore),
        (RelicId::PureWater, RelicId::HolyWater),
    ];
    REPLACEMENTS.iter().any(|(starter, boss)|
        (candidate == *starter && selected.contains(boss)) || (candidate == *boss && selected.contains(starter)))
}

fn generated_counter(id: RelicId, random: u64) -> Option<(RelicId, i32)> {
    let value = match id {
        RelicId::PenNib | RelicId::Nunchaku | RelicId::InkBottle => (random % 10) as i32,
        RelicId::IncenseBurner => (random % 6) as i32,
        RelicId::HappyFlower | RelicId::Sundial | RelicId::Omamori | RelicId::Matryoshka => (random % 3) as i32,
        RelicId::WingBoots | RelicId::Girya | RelicId::TinyChest => (random % 4) as i32,
        RelicId::Inserter => (random % 3) as i32 - 1,
        RelicId::NeowsLament => if random % 4 == 0 { -2 } else { (random % 3) as i32 + 1 },
        RelicId::Circlet => (random % 4) as i32 + 1,
        RelicId::LizardTail | RelicId::MawBank | RelicId::NlothsHungryFace | RelicId::AncientTeaSet => 1,
        _ => return None,
    };
    Some((id, value))
}
