//! Diagnostics-only public-search invariance and belief-perturbation campaign.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use spire_core::{
    combat::rng::Partition,
    content::{Catalog, Character},
};
use spire_env::{
    BranchView, CombatTrainingCatalog, CombatTrainingSession, PublicRoot, PublicSearch,
    RunActionIndex, RunSession, RunSessionError, RunObservation, SearchConfig, SearchLimits,
    StandardRunConfig, STANDARD_COMBAT_CATALOG_SCHEMA_ID, STANDARD_COMBAT_PROFILE_ID,
};

use crate::{Decision, Driver, DriverError, IllegalProbe, StepOut};

const ENCOUNTERS: [&str; 3] = ["two_louse", "three_darklings", "gremlin_nob"];
const CHARACTER_DOMAIN: u64 = 0xA076_1D64_78BD_642F;
const ASCENSION_DOMAIN: u64 = 0xE703_7ED1_A0B4_28DB;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchSelection {
    pub character: Character,
    pub ascension: u8,
    pub standalone_encounter: &'static str,
}

/// Select a deterministic finite search scenario from the episode seed.
pub fn search_selection(seed: u64) -> SearchSelection {
    let character = Character::ALL[(crate::mix64(seed ^ CHARACTER_DOMAIN) as usize) % Character::ALL.len()];
    let ascension = if crate::mix64(seed ^ ASCENSION_DOMAIN) & 1 == 0 { 0 } else { 20 };
    let standalone_encounter = ENCOUNTERS[(seed as usize) % ENCOUNTERS.len()];
    SearchSelection { character, ascension, standalone_encounter }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchFuzzStats {
    pub character: String,
    pub ascension: u8,
    pub standalone_encounter: String,
    pub run_combats_seen: u64,
    pub run_combat_decisions: u64,
    pub standalone_combat_decisions: u64,
    pub searches: u64,
    pub branches: u64,
    pub steps: u64,
    pub perturbations_applied: u64,
    pub hidden_order_refusals: u64,
    pub unsupported_roots: BTreeMap<String, u64>,
}

impl SearchFuzzStats {
    fn for_selection(selection: SearchSelection) -> Self {
        Self {
            character: selection.character.as_str().into(),
            ascension: selection.ascension,
            standalone_encounter: selection.standalone_encounter.into(),
            ..Self::default()
        }
    }
}

fn legal(mask: &[i8]) -> Vec<usize> {
    mask.iter().enumerate().filter_map(|(index, mask)| (*mask == 1).then_some(index)).collect()
}

fn pick(mask: &[i8], state: &mut u64) -> Result<usize, String> {
    let options = legal(mask);
    if options.is_empty() {
        return Err("combat decision has no legal actions".into());
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    Ok(options[(*state as usize) % options.len()])
}

fn step_with_hidden_order_retries(
    search: &mut PublicSearch, branch: spire_env::BranchId, token: spire_env::RunDecisionToken,
    mask: &[i8], view: BranchView, policy: &mut u64, preferred_first: Option<usize>, stats: &mut SearchFuzzStats,
) -> Result<(spire_env::BranchStep, usize), String> {
    let legal_actions = legal(mask);
    if legal_actions.is_empty() { return Err("combat decision has no legal actions".into()); }
    let first = preferred_first.map_or_else(|| pick(mask, policy), Ok)?;
    let start = legal_actions.iter().position(|action| *action == first).unwrap();
    for offset in 0..legal_actions.len() {
        let action = legal_actions[(start + offset) % legal_actions.len()];
        match search.step(branch, token, RunActionIndex(action as i64), view) {
            Ok(step) => return Ok((step, action)),
            Err(RunSessionError::UnsupportedBoundary(spire_env::HIDDEN_ORDER_PICK)) => stats.hidden_order_refusals += 1,
            Err(error) => return Err(format!("branch step: {error:?}")),
        }
    }
    Err("all legal search actions were refused for hidden draw order".into())
}

fn limits() -> SearchLimits {
    SearchLimits { max_live_branches: 8, max_branches: 32, max_steps_per_branch: 8, max_total_steps: 64 }
}

fn root_reason(error: RunSessionError) -> String { format!("{error:?}") }

fn audit(
    root: PublicRoot,
    run: &spire_core::run::Run,
    seed: u64,
    stats: &mut SearchFuzzStats,
) -> Result<(), String> {
    let mut perturbed = run.clone();
    let bits = perturbed.fuzz_combat_mut().ok_or("fuzz Run lost its combat")?.fuzz_perturb_hidden(seed as u8);
    if bits != 0 {
        stats.perturbations_applied += 1;
        let perturbed_root = PublicRoot::fuzz_of(&perturbed, &spire_env::RunCapacities::standard_default())
            .map_err(|error| format!("perturbed root rejected: {error:?}"))?;
        if !perturbed_root.same_world(&root) {
            return Err(format!("P2 public root changed after hidden perturbation mask={bits:#x}"));
        }
    }

    stats.searches += 1;
    let mut search = PublicSearch::new(root, SearchConfig { research_seed: seed, limits: limits() })
        .map_err(|error| format!("new search: {error:?}"))?;
    let mut branch_state = seed | 1;
    let root_branch = search.branch(Some(seed)).map_err(|error| format!("root branch: {error:?}"))?;
    stats.branches += 1;
    // Keep a same-world fallback if the first rollout ends on its first action; live branches cannot be forked after terminal.
    let fallback = search.fork(root_branch).map_err(|error| format!("root fallback fork: {error:?}"))?;
    stats.branches += 1;
    let first = search.decision(root_branch).map_err(|error| format!("root decision: {error:?}"))?;
    let (first_step, first_action) = step_with_hidden_order_retries(&mut search, root_branch, first.token,
        &first.observation.action_mask, BranchView::Full, &mut branch_state, None, stats)?;
    stats.steps += 1;
    if first_step.observation.is_none() { return Err("Full step omitted observation".into()); }
    let mut active = root_branch;
    let mut prefix = 1usize;
    if first_step.terminated || first_step.truncated {
        active = fallback;
        prefix = 0;
    } else {
        let counters = search.counters();
        let stale = search.step(root_branch, first.token, RunActionIndex(first_action as i64), BranchView::Mask);
        if !matches!(stale, Err(RunSessionError::StaleToken)) { return Err(format!("stale branch token was not rejected: {stale:?}")); }
        if search.counters() != counters { return Err("stale token rejection changed search counters".into()); }
        search.close(fallback).map_err(|error| format!("close unused fallback: {error:?}"))?;
    }

    // Advance a live lineage to two actions and fork it mid-rollout. If it reaches terminal sooner, the rollout ends there.
    let mut ended = false;
    while prefix < 2 {
        let d = search.decision(active).map_err(|error| format!("rollout decision: {error:?}"))?;
        let view = if prefix == 0 { BranchView::Full } else { BranchView::Mask };
        let (stepped, _) = step_with_hidden_order_retries(&mut search, active, d.token,
            &d.observation.action_mask, view, &mut branch_state, None, stats)?;
        stats.steps += 1;
        if view == BranchView::Full && stepped.observation.is_none() { return Err("Full step omitted observation".into()); }
        if view == BranchView::Mask && stepped.observation.is_some() { return Err("Mask step returned observation".into()); }
        if stepped.terminated || stepped.truncated { ended = true; break; }
        prefix += 1;
    }

    let mut lineages = Vec::new();
    if !ended {
        lineages.push(active);
        let fork = search.fork(active).map_err(|error| format!("mid-rollout fork: {error:?}"))?;
        stats.branches += 1;
        lineages.push(fork);
    }

    // Finish each live lineage with the same research-seeded legal policy, up to the declared eight-step horizon.
    for branch in lineages {
        let mut d = search.decision(branch).map_err(|error| format!("branch decision: {error:?}"))?;
        loop {
            let completed = search.actions(branch).map_err(|error| format!("branch actions: {error:?}"))?.len();
            if completed >= 8 { break; }
            let (stepped, _) = step_with_hidden_order_retries(&mut search, branch, d.token,
                &d.observation.action_mask, BranchView::Mask, &mut branch_state, None, stats)?;
            stats.steps += 1;
            if stepped.observation.is_some() { return Err("Mask step returned observation".into()); }
            if stepped.terminated || stepped.truncated { break; }
            d = search.decision(branch).map_err(|error| format!("post-rollout decision: {error:?}"))?;
        }
    }

    let closed = search.branch(Some(seed.wrapping_add(1))).map_err(|error| format!("closed-token branch: {error:?}"))?;
    stats.branches += 1;
    let token = search.decision(closed).map_err(|error| format!("closed-token decision: {error:?}"))?.token;
    search.close(closed).map_err(|error| format!("close: {error:?}"))?;
    let counters = search.counters();
    if !matches!(search.step(closed, token, RunActionIndex(0), BranchView::Mask), Err(RunSessionError::Closed)) {
        return Err("closed branch token was not rejected".into());
    }
    if search.counters() != counters || !matches!(search.decision(closed), Err(RunSessionError::Closed)) {
        return Err("closed token rejection changed search state".into());
    }
    Ok(())
}

fn standard_catalog(selection: SearchSelection) -> Result<CombatTrainingCatalog, String> {
    let catalog = Catalog::standard();
    let deck = catalog.starter_deck(selection.character).map_err(|error| format!("starter deck: {error:?}"))?
        .iter().map(|card| json!({"card": card.as_str(), "upgrades": 0})).collect::<Vec<_>>();
    let relic = catalog.starter_relic(selection.character).ok_or("starter relic missing")?;
    let max_hp = match selection.character {
        Character::Ironclad => 80,
        Character::Silent => 70,
        Character::Defect => 75,
        Character::Watcher => 72,
    };
    let potion_slots = if selection.ascension >= 11 { 2 } else { 3 };
    let potions = std::iter::repeat(Value::Null).take(potion_slots).collect::<Vec<_>>();
    let entries = ENCOUNTERS.iter().map(|encounter| json!({
        "id": format!("search-{encounter}"),
        "character": selection.character.as_str(),
        "ascension": selection.ascension,
        "encounter": encounter,
        "hp": max_hp,
        "max_hp": max_hp,
        "gold": 99,
        "deck": deck.clone(),
        "relics": [relic.as_str()],
        "potions": potions.clone(),
    })).collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&json!({
        "catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID,
        "profile_id": STANDARD_COMBAT_PROFILE_ID,
        "entries": entries,
    })).map_err(|error| error.to_string())?;
    CombatTrainingCatalog::standard_from_json(&bytes).map_err(|error| format!("catalog: {error:?}"))
}

fn run_search_mode_detailed(seed: u64) -> Result<SearchFuzzStats, (SearchFuzzStats, String)> {
    let selection = search_selection(seed);
    let mut stats = SearchFuzzStats::for_selection(selection);
    let result = (|| -> Result<(), String> {
        let mut state = seed | 1;
        let mut run = RunSession::new_standard(StandardRunConfig::new(selection.character, selection.ascension, Partition::Test))
            .map_err(|error| format!("Run session: {error:?}"))?;
        run.reset(Some(seed)).map_err(|error| format!("Run reset: {error:?}"))?;

        let mut combats_seen = 0u64;
        let mut was_in_combat = false;
        for _ in 0..12_000 {
            let boundary = run.decision().map_err(|error| format!("Run decision: {error:?}"))?;
            let in_combat = run.fuzz_run().ok_or("Run fuzz world unavailable")?
                .decision().map_err(|error| format!("Run combat state: {error:?}"))?.combat.is_some();
            if in_combat && !was_in_combat {
                combats_seen += 1;
                if combats_seen <= 2 { stats.run_combats_seen = combats_seen; }
            }
            if combats_seen > 2 { break; }
            if !in_combat && was_in_combat && combats_seen >= 2 { break; }

            if in_combat && combats_seen <= 2 && stats.run_combat_decisions < 40 {
                stats.run_combat_decisions += 1;
                let before = boundary.clone();
                match run.public_root() {
                    Ok(root) => {
                        let core = run.fuzz_run().ok_or("Run fuzz world unavailable")?.clone();
                        audit(root, &core, state, &mut stats)?;
                        let after_search = run.decision().map_err(|error| format!("Run decision: {error:?}"))?;
                        if before.observation != after_search.observation { return Err("live observation changed while search ran".into()); }
                    }
                    Err(error) => { *stats.unsupported_roots.entry(root_reason(error)).or_default() += 1; }
                }
            }
            if boundary.terminal.is_some() || stats.run_combat_decisions >= 40 { break; }
            let action = pick(&boundary.observation.action_mask, &mut state)?;
            run.step(boundary.token, RunActionIndex(action as i64)).map_err(|error| format!("Run step: {error:?}"))?;
            was_in_combat = in_combat;
        }

        let catalog = standard_catalog(selection)?;
        let mut combat = CombatTrainingSession::new_standard(catalog, Partition::Test)
            .map_err(|error| format!("combat session: {error:?}"))?;
        let entry_id = format!("search-{}", selection.standalone_encounter);
        let mut decision = combat.reset(Some(seed), Some(&entry_id)).map_err(|error| format!("combat reset: {error:?}"))?;
        for _ in 0..40 {
            stats.standalone_combat_decisions += 1;
            let before = combat.decision().map_err(|error| format!("combat decision: {error:?}"))?;
            match combat.public_root() {
                Ok(root) => {
                    let core = combat.fuzz_run().ok_or("combat fuzz world unavailable")?.clone();
                    audit(root, &core, state, &mut stats)?;
                    let after_search = combat.decision().map_err(|error| format!("combat decision: {error:?}"))?;
                    if before.observation != after_search.observation { return Err("live observation changed while search ran".into()); }
                }
                Err(error) => { *stats.unsupported_roots.entry(root_reason(error)).or_default() += 1; }
            }
            let action = pick(&decision.observation.action_mask, &mut state)?;
            let stepped = combat.step(decision.token, RunActionIndex(action as i64))
                .map_err(|error| format!("combat step: {error:?}"))?;
            if stepped.terminated || stepped.truncated { break; }
            decision = stepped.boundary;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(stats),
        Err(error) => Err((stats, error)),
    }
}

/// Audits every combat decision in the first two Standard Run combats, then up to 40
/// decisions in the seed-selected standalone Standard combat.
pub fn run_search_mode(seed: u64) -> Result<SearchFuzzStats, String> {
    run_search_mode_detailed(seed).map_err(|(_, error)| error)
}

/// Runner adapter: one seed executes the whole search campaign as a single episode.
pub struct SearchDriver {
    selection: SearchSelection,
    stats: SearchFuzzStats,
    decision: Option<Decision>,
}

impl SearchDriver {
    pub fn new() -> Self {
        let selection = search_selection(0);
        Self { selection, stats: SearchFuzzStats::for_selection(selection), decision: None }
    }
}

impl Default for SearchDriver {
    fn default() -> Self { Self::new() }
}

impl Driver for SearchDriver {
    fn class(&self) -> &'static str { "search" }
    fn describe(&self) -> Value { json!({"driver": "search", "mode": "m34s-b-public-search-v1"}) }
    fn fresh(&self) -> Box<dyn Driver> { Box::new(Self::new()) }

    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        self.selection = search_selection(seed);
        self.stats = SearchFuzzStats::for_selection(self.selection);
        self.decision = None;
        match run_search_mode_detailed(seed) {
            Ok(stats) => self.stats = stats,
            Err((stats, error)) => {
                self.stats = stats;
                return Err(DriverError::new("search", error));
            }
        }
        let decision = Decision {
            observation: RunObservation { groups: vec![], action_mask: vec![] },
            token: [seed, 0],
            terminal: Some("search-complete".into()),
        };
        self.decision = Some(decision.clone());
        Ok(decision)
    }

    fn decision(&self) -> Result<Decision, DriverError> {
        self.decision.clone().ok_or_else(|| DriverError::fault("search episode not reset"))
    }
    fn step(&mut self, _index: usize) -> Result<StepOut, DriverError> {
        Err(DriverError::fault("search episode is terminal"))
    }
    fn preflight(&self, _index: usize) -> Result<(), DriverError> {
        Err(DriverError::fault("search episode has no actions"))
    }
    fn probe_illegal(&mut self, _probe: IllegalProbe) -> Result<(), DriverError> {
        Err(DriverError::new("probe_rejected", "search episode has no actions"))
    }
    fn descriptor_stable(&self) -> Result<(), DriverError> { Ok(()) }
    fn reached_combat(&self) -> bool { self.stats.run_combats_seen > 0 || self.stats.standalone_combat_decisions > 0 }
    fn diagnostic(&self) -> Value { json!({"search_fuzz": self.stats}) }
    fn labels(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("profile".into(), "M34sSearch".into()),
            ("character".into(), self.selection.character.as_str().into()),
            ("ascension".into(), self.selection.ascension.to_string()),
            ("encounter".into(), self.selection.standalone_encounter.into()),
            ("entry".into(), format!("search-{}", self.selection.standalone_encounter)),
        ])
    }
    fn search_fuzz_stats(&self) -> Option<SearchFuzzStats> { Some(self.stats.clone()) }
}
