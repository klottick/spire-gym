//! M34s-b search arena `spire-gym-public-search-v1` (design §4.7) over the public belief `spire-gym-public-belief-v1`
//! (§4.6). A [`PublicRoot`] is the canonical combat-scope Run of a public decision (no hidden value of the live world);
//! a [`PublicSearch`] owns that root and every branch world sampled from it with a caller research seed, and holds no
//! reference to the live session. Branches are observed exactly like `standard-combat-v1` sessions (the Standard
//! combat view), carry their own tokens from the shared session allocator, and end at the combat's own terminal.
use crate::{RunActionIndex, RunCapacities, RunDecisionToken, RunObservation, RunSessionError as Error};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::run::{GameAction, Run};

pub const PUBLIC_SEARCH_ID: &str = "spire-gym-public-search-v1";
pub const PUBLIC_BELIEF_ID: &str = spire_core::combat::belief::PUBLIC_BELIEF_ID;
pub const PUBLIC_KNOWLEDGE_ID: &str = "spire-gym-public-knowledge-v1";
const ROOT_DIGEST_DOMAIN: &[u8] = b"spire-gym-public-belief-v1\0root\0";

/// The canonical public root of a combat decision: equal for public-equivalent worlds (design §4.6.3), with a public
/// digest over its Standard combat view and its public knowledge. Opaque: no accessor reveals the Run.
/// Equality is public-content equality (the digest and the capacities; final review I1): the canonical worlds of
/// public-equivalent combats may still differ in the labels of identical combat-local copies, which no output depends
/// on (a branch refuses a pick among identical draw-pile copies, `HIDDEN_ORDER_PICK`).
#[derive(Clone)]
pub struct PublicRoot { run: Run, digest: [u8; 32], caps: RunCapacities }
impl PartialEq for PublicRoot { fn eq(&self, other: &Self) -> bool { self.digest == other.digest && self.caps == other.caps } }
impl Eq for PublicRoot {}
impl std::fmt::Debug for PublicRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "PublicRoot {{ digest: {} }}", crate::combat_policy::hex(&self.digest)) }
}
impl PublicRoot {
    /// The root of `run`'s live combat (a Standard Run or a combat-scope Run) at a published decision.
    pub(crate) fn of(run: &Run, caps: &RunCapacities) -> Result<Self, Error> {
        let run = run.public_root_run().map_err(|u| Error::UnsupportedBoundary(u.reason()))?;
        let (observation, _, _) = crate::combat_training::standard_combat_publication(&run, caps).map_err(redact)?;
        let knowledge = run.public_knowledge_bytes().ok_or(Error::UnsupportedBoundary("draw knowledge exceeds the exact model"))?;
        let mut h = Sha256::new();
        h.update(ROOT_DIGEST_DOMAIN);
        hash_observation(&mut h, &observation);
        h.update((knowledge.len() as u64).to_le_bytes());
        h.update(&knowledge);
        Ok(Self { run, digest: h.finalize().into(), caps: caps.clone() })
    }
    /// Fuzz oracle only (never public): the root of an arbitrary (perturbed) world, for the P2 check.
    #[cfg(feature = "fuzz-diagnostics")]
    pub fn fuzz_of(run: &Run, caps: &RunCapacities) -> Result<Self, Error> { Self::of(run, caps) }
    /// SHA-256 over the root's encoded Standard combat view and its public knowledge in normal form (public).
    pub fn digest(&self) -> [u8; 32] { self.digest }
    /// M34s-c `reachable_recipe` (design §4.8): the world of one reset, keyed by its seed, partition and ordinal (research
    /// purpose `start`), sampled from this root exactly as a branch is.
    pub(crate) fn sample_start(&self, seed: u64, partition: spire_core::combat::rng::Partition, ordinal: u64) -> Result<Run, Error> {
        let key = spire_core::run::research_start_key(seed, partition, ordinal, &self.digest);
        self.run.belief_sample(&key).map_err(|u| Error::UnsupportedBoundary(u.reason()))
    }
    /// Oracle only (never public): whether two roots hold the same canonical world, labels included (P1/P2 checks that
    /// canonicalisation removed every hidden value; stronger than root equality).
    #[cfg(any(test, feature = "fuzz-diagnostics"))]
    pub fn same_world(&self, other: &Self) -> bool { self.run == other.run && self == other }
}
#[cfg(test)]
pub(crate) fn hash_observation_for_test(h: &mut Sha256, o: &RunObservation) { hash_observation(h, o) }
fn hash_observation(h: &mut Sha256, o: &RunObservation) {
    for g in &o.groups {
        h.update(g.key.as_bytes()); h.update([0]);
        for d in &g.shape { h.update((*d as u64).to_le_bytes()); }
        for v in &g.values { h.update(v.to_le_bytes()); }
        h.update(g.known.iter().map(|k| *k as u8).collect::<Vec<_>>());
        if let Some(occ) = &g.occupied { h.update([1]); h.update(occ.iter().map(|k| *k as u8).collect::<Vec<_>>()); } else { h.update([0]); }
    }
    h.update(o.action_mask.iter().map(|k| *k as u8).collect::<Vec<_>>());
}

/// The refusal reason for a branch pick among identical draw-pile copies (a class, never a value; final review I1).
pub const HIDDEN_ORDER_PICK: &str = "a pick among identical draw-pile copies depends on hidden order";
/// The class-only error for a core fault or core error inside a root or a branch (final review v3): a core error can carry
/// the attempted action, whose card instance handle is outside the public information set, so the arena never returns it.
pub const BRANCH_CORE_FAULT: &str = "search core fault (redacted)";
fn redacted(_: spire_core::run::RunError) -> Error { Error::DeliveryFault(BRANCH_CORE_FAULT) }
/// Redact a core error coming out of a shared publication or session helper.
fn redact(e: Error) -> Error { match e { Error::Core(core) => redacted(core), e => e } }
/// Declared search limits (design §4.7), validated against [`SEARCH_HARD_CAPS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchLimits { pub max_live_branches: usize, pub max_branches: u64, pub max_steps_per_branch: u64, pub max_total_steps: u64 }
pub const SEARCH_HARD_CAPS: SearchLimits = SearchLimits { max_live_branches: 4_096, max_branches: 1 << 20, max_steps_per_branch: 100_000, max_total_steps: 100_000_000 };
/// Estimated heap bytes of one combat-scope Run (final review I2, v2): the counting allocator of the spire-env test binary
/// measured at most 12,889 bytes (draw knowledge included) for a root at the first Act 3 boss of the supplied routes, 4
/// characters × {A0, A20}, and at standalone starts (`m34s_search_measure`); about 5× headroom. The Run's inline size is
/// counted separately (the boxed branch record). A measured estimate: the one term of the bound that is not structural.
pub const BRANCH_RUN_BYTES_ESTIMATE: u64 = 64 * 1024;
/// Bytes one entry of an ordered collection holding a `u64` (and a `value_bytes` value) can take (final review v2 I2): a
/// B-tree node other than the root holds at least 5 of its 11 slots, so an entry takes at most 11/5 of its slot plus its
/// share of the node header and child edges; 3 × (8 + `value_bytes`) + 48 bounds that, allocator header included. The
/// branch map stores each branch boxed (an 8-byte slot), so its nodes stay small.
const fn collection_entry_bytes(value_bytes: u64) -> u64 { 3 * (8 + value_bytes) + 48 }
/// One collection's root node, which may be nearly empty: 11 slots, 12 child edges and a header.
const fn collection_root_bytes(value_bytes: u64) -> u64 { 11 * (8 + value_bytes) + 12 * 8 + 32 }
/// A live branch's record: its box (the `Branch` with its inline Run, allocator header included) and its map slot.
fn branch_record_bytes() -> u64 { std::mem::size_of::<Branch>() as u64 + 16 }
const BOX_SLOT_BYTES: u64 = 8;
/// The per-live-branch retained-bytes bound published in `spec()`: the Run estimate, plus the enforced knowledge cap
/// (`KNOWLEDGE_BYTES_CAP`; a larger knowledge is Unrepresentable), plus the accepted-action history (8 bytes per step up to
/// `max_steps_per_branch`, doubled for vector growth), the legal-action table (`caps.actions` actions) and the branch's
/// entry in the arena's branch map (the branch record is stored in the map's node).
pub fn branch_retained_bytes_bound(limits: &SearchLimits, caps: &RunCapacities) -> u64 {
    BRANCH_RUN_BYTES_ESTIMATE + spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP as u64
        + 2 * limits.max_steps_per_branch * std::mem::size_of::<i64>() as u64 + (caps.actions * std::mem::size_of::<GameAction>()) as u64
        + branch_record_bytes() + collection_entry_bytes(BOX_SLOT_BYTES)
}
/// The whole arena's retained-bytes bound (`spec()["max_retained_bytes"]`; final review v2 I2): the arena and its root (a
/// Run estimate plus the knowledge cap), the three collections' root nodes, `max_live_branches` live branches, and one
/// closed-branch entry and one used-sample entry per branch created (at most `max_branches`). Every term except
/// `BRANCH_RUN_BYTES_ESTIMATE` is an upper bound from the data structures; that one is a measured estimate (so the total is
/// a declared bound resting on it, not allocator accounting).
pub fn search_retained_bytes_bound(limits: &SearchLimits, caps: &RunCapacities) -> u64 {
    search_fixed_bytes() + limits.max_live_branches as u64 * branch_retained_bytes_bound(limits, caps)
        + limits.max_branches * created_branch_bytes()
}
fn search_fixed_bytes() -> u64 {
    std::mem::size_of::<PublicSearch>() as u64 + BRANCH_RUN_BYTES_ESTIMATE + spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP as u64
        + collection_root_bytes(BOX_SLOT_BYTES) + 2 * collection_root_bytes(0)
}
fn created_branch_bytes() -> u64 { 2 * collection_entry_bytes(0) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchConfig { pub research_seed: u64, pub limits: SearchLimits }
/// What a step returns: the full Standard combat view, or only its legal-action mask (rollouts; no encoding).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchView { Full, Mask }
/// One branch handle (its id is from the shared session allocator, so it never equals a session's).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchId(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchBoundary { pub observation: RunObservation, pub token: RunDecisionToken, pub info: Value }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchStep {
    /// The full view (`BranchView::Full`) or `None` (`Mask`).
    pub observation: Option<RunObservation>, pub action_mask: Vec<i8>, pub token: RunDecisionToken,
    pub reward: i32, pub terminated: bool, pub truncated: bool, pub info: Value,
}

/// One `step_many` entry (design §3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchStepRequest { pub branch: BranchId, pub token: RunDecisionToken, pub index: RunActionIndex }
/// One `step_many` outcome, in input order: the step, or the branch's fault (`stage`: `step`, `encode` or `panic`; the
/// error is the class-only redacted one `step` returns). A faulted branch is frozen; its prefix stays readable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchStepOutcome { Stepped(BranchStep), Fault { error: Error, stage: &'static str, committed: bool } }
/// A world after one action: its publication, or its fault.
enum Advanced {
    Done { observation: Option<RunObservation>, action_mask: Vec<i8>, info: Value, legal: Vec<GameAction>, reward: i32, terminated: bool },
    Fault { error: Error, stage: &'static str, committed: bool },
}
/// The single-branch step on a world: the shared resolver, then the requested view (`step` and `step_many` both use it).
fn advance(run: &mut Run, action: &GameAction, view: BranchView, caps: &RunCapacities) -> Advanced {
    if let Err(e) = run.step(action) {
        let committed = matches!(e, spire_core::run::RunError::Faulted(_));
        return Advanced::Fault { error: redacted(e), stage: "step", committed }
    }
    let terminal = run.combat_scope_terminal();
    let reward = terminal.map_or(0, |t| crate::combat_training::outcome_reward(t.outcome));
    let published = match view {
        BranchView::Full => crate::combat_training::standard_combat_publication(run, caps).map(|(o, info, legal)| {
            let mask = o.action_mask.clone();
            (Some(o), mask, info, legal)
        }),
        BranchView::Mask => run.decision().map_err(redacted).and_then(|d| {
            let legal: Vec<GameAction> = if terminal.is_some() { vec![] } else { d.candidates.into_iter().map(|c| c.action).collect() };
            if legal.len() > caps.actions { return Err(Error::Capacity { group: "actions", required: legal.len(), capacity: caps.actions }) }
            let mut mask = vec![0; caps.actions];
            mask[..legal.len()].iter_mut().for_each(|m| *m = 1);
            Ok((None, mask, json!({"terminal_reason": terminal.map(|t| crate::combat_training::outcome_label(t.outcome))}), legal))
        }),
    };
    // Final review M3: the engine has committed, so a publication error faults the branch (never a Live branch with the
    // previous boundary's action table).
    match published {
        Ok((observation, action_mask, info, legal)) => Advanced::Done { observation, action_mask, info, legal, reward, terminated: terminal.is_some() },
        Err(e) => Advanced::Fault { error: redact(e), stage: "encode", committed: true },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status { Live, Terminal, Truncated, Faulted }
#[derive(Clone)]
/// `origin` is the sample key of the world the branch started in; with `actions` it replays the branch exactly (the
/// last boundary of a faulted branch). It is never output.
struct Branch { run: Run, origin: [u8; 32], serial: u64, steps: u64, actions: Vec<i64>, legal: Vec<GameAction>, status: Status }

/// The arena (design §4.7): owns its root and branches; deterministic budgets; no wall clock; no live-session reference.
pub struct PublicSearch { root: PublicRoot, research_root: [u8; 32], limits: SearchLimits, branches: std::collections::BTreeMap<u64, Box<Branch>>,
    closed: std::collections::BTreeSet<u64>, used_samples: std::collections::BTreeSet<u64>, next_auto: u64, created: u64, total_steps: u64,
    /// M35 `step_many` workers (execution configuration, not a budget; design §3.6).
    workers: usize }
impl std::fmt::Debug for PublicSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicSearch {{ root: {:?}, live: {}, created: {}, total_steps: {} }}", self.root, self.branches.len(), self.created, self.total_steps)
    }
}
impl PublicSearch {
    pub fn new(root: PublicRoot, config: SearchConfig) -> Result<Self, Error> {
        let (l, c) = (config.limits, SEARCH_HARD_CAPS);
        if l.max_live_branches == 0 || l.max_branches == 0 || l.max_steps_per_branch == 0 || l.max_total_steps == 0 { return Err(Error::Input("search limits must be positive")) }
        if l.max_live_branches > c.max_live_branches || l.max_branches > c.max_branches || l.max_steps_per_branch > c.max_steps_per_branch || l.max_total_steps > c.max_total_steps {
            return Err(Error::Input("search limits exceed the hard caps"))
        }
        Ok(Self { root, research_root: spire_core::run::research_root(config.research_seed), limits: l, branches: Default::default(), closed: Default::default(),
            used_samples: Default::default(), next_auto: 0, created: 0, total_steps: 0, workers: 1 })
    }
    /// M35: `step_many` runs on `workers` threads of the shared process-wide pool for that count (1: no pool).
    pub fn with_workers(mut self, workers: usize) -> Result<Self, Error> {
        if workers == 0 { return Err(Error::Input("search workers must be positive")) }
        self.workers = workers;
        Ok(self)
    }
    pub fn workers(&self) -> usize { self.workers }
    fn admit_branch(&self) -> Result<(), Error> {
        if self.branches.len() >= self.limits.max_live_branches { return Err(Error::Budget("live branch limit reached")) }
        if self.created >= self.limits.max_branches { return Err(Error::Budget("branch limit reached")) }
        Ok(())
    }
    fn insert(&mut self, branch: Branch) -> Result<BranchId, Error> {
        #[cfg(test)]
        if FAIL_ID_ALLOCATION.with(std::cell::Cell::get) { return Err(Error::DeliveryFault("Run session identity exhausted")) }
        let id = crate::run_session::allocate_run_session_id()?;
        self.created += 1;
        self.branches.insert(id, Box::new(branch));
        Ok(BranchId(id))
    }
    /// A new branch at sample `k` (`None`: the smallest index this search has not used). A repeated `k` gives an
    /// identical world (common random numbers).
    pub fn branch(&mut self, sample: Option<u64>) -> Result<BranchId, Error> {
        self.admit_branch()?;
        let k = match sample { Some(k) => k, None => { let mut k = self.next_auto; while self.used_samples.contains(&k) { k = k.checked_add(1).ok_or(Error::Budget("sample index exhausted"))?; } k } };
        let key = spire_core::run::research_sample_key(&self.research_root, &self.root.digest, k);
        let run = self.root.run.belief_sample(&key).map_err(|u| Error::UnsupportedBoundary(u.reason()))?;
        let legal = run.decision().map_err(redacted)?.candidates.into_iter().map(|c| c.action).collect();
        // Final review v3: the sample is recorded only once the branch exists, so a failed creation retains nothing and
        // `used_samples` never outgrows `max_branches`.
        let id = self.insert(Branch { run, origin: key, serial: 0, steps: 0, actions: vec![], legal, status: Status::Live })?;
        self.used_samples.insert(k);
        if sample.is_none() { self.next_auto = k + 1; }
        Ok(id)
    }
    fn get(&self, b: BranchId) -> Result<&Branch, Error> {
        if self.closed.contains(&b.0) { return Err(Error::Closed) }
        self.branches.get(&b.0).map(|b| &**b).ok_or(Error::Input("unknown branch"))
    }
    /// The branch sidecar from its step count and status alone (recheck R2: a step never clones its branch for this).
    fn info(steps: u64, status: Status, mut info: Value) -> Value {
        info["start_kind"] = json!("branch");
        info["branch_steps"] = json!(steps);
        info["end_reason"] = match status { Status::Truncated => json!("step_limit"), Status::Faulted => json!("fault"), _ => info["terminal_reason"].clone() };
        info
    }
    /// The branch's current boundary (encoded on demand, owned). A faulted branch shows its last boundary before the
    /// fault (replayed from its origin and accepted actions; its token is spent).
    pub fn decision(&self, b: BranchId) -> Result<BranchBoundary, Error> {
        let branch = self.get(b)?;
        let replayed;
        let run = if branch.status == Status::Faulted { replayed = self.replay(branch)?; &replayed } else { &branch.run };
        let (observation, info, _) = crate::combat_training::standard_combat_publication(run, &self.root.caps).map_err(redact)?;
        Ok(BranchBoundary { observation, token: RunDecisionToken([b.0, branch.serial]), info: Self::info(branch.steps, branch.status, info) })
    }
    fn replay(&self, branch: &Branch) -> Result<Run, Error> {
        let mut run = self.root.run.belief_sample(&branch.origin).map_err(|u| Error::UnsupportedBoundary(u.reason()))?;
        for index in &branch.actions {
            let d = run.decision().map_err(redacted)?;
            let action = usize::try_from(*index).ok().and_then(|i| d.candidates.get(i)).map(|c| c.action.clone()).ok_or(Error::DeliveryFault("branch replay diverged"))?;
            run.step(&action).map_err(redacted)?;
        }
        Ok(run)
    }
    /// One action through the shared resolver. Budget, stale-token, illegal-index and hidden-order-pick
    /// (`UnsupportedBoundary(HIDDEN_ORDER_PICK)`) errors have no effect; the branch stays live.
    pub fn step(&mut self, b: BranchId, token: RunDecisionToken, index: RunActionIndex, view: BranchView) -> Result<BranchStep, Error> {
        let action = self.preflight(b, token, index)?;
        if self.total_steps >= self.limits.max_total_steps { return Err(Error::Budget("total step limit reached")) }
        let (caps, max_steps) = (self.root.caps.clone(), self.limits.max_steps_per_branch);
        self.total_steps += 1;
        let branch = self.branches.get_mut(&b.0).unwrap();
        let advanced = advance(&mut branch.run, &action, view, &caps);
        match Self::settle(b, branch, index, advanced, max_steps) { BranchStepOutcome::Stepped(step) => Ok(step), BranchStepOutcome::Fault { error, .. } => Err(error) }
    }
    /// The no-effect checks of `step` for one entry; returns the action.
    fn preflight(&self, b: BranchId, token: RunDecisionToken, index: RunActionIndex) -> Result<GameAction, Error> {
        let branch = self.get(b)?;
        match branch.status { Status::Faulted => return Err(Error::BranchFaulted), Status::Terminal | Status::Truncated => return Err(Error::Terminal), Status::Live => {} }
        if token != RunDecisionToken([b.0, branch.serial]) { return Err(Error::StaleToken) }
        let action = usize::try_from(index.0).ok().and_then(|i| branch.legal.get(i)).cloned().ok_or(Error::Input("branch action index is not legal"))?;
        // Final review I1: a sampled world keeps the true labels of identical copies, so a pick that removes a copy chosen
        // by hidden instance order would make the branch depend on hidden state; refused with no effect (fail closed).
        if branch.run.pick_depends_on_hidden_order(&action) { return Err(Error::UnsupportedBoundary(HIDDEN_ORDER_PICK)) }
        Ok(action)
    }
    /// Apply one advanced world to its branch record: serial and step count always advance (the step was attempted); a
    /// fault freezes the branch (never retried or revived; `actions` keeps the accepted prefix).
    fn settle(b: BranchId, branch: &mut Branch, index: RunActionIndex, advanced: Advanced, max_steps: u64) -> BranchStepOutcome {
        branch.serial += 1;
        branch.steps += 1;
        let (observation, action_mask, info, legal, reward, terminated) = match advanced {
            Advanced::Done { observation, action_mask, info, legal, reward, terminated } => (observation, action_mask, info, legal, reward, terminated),
            Advanced::Fault { error, stage, committed } => {
                branch.status = Status::Faulted;
                branch.legal.clear();
                // As `step`: the accepted prefix includes an action whose engine step committed only when its publication
                // failed (`encode`); an engine fault or a panic leaves the prefix at the last published boundary.
                if stage == "encode" { branch.actions.push(index.0); }
                return BranchStepOutcome::Fault { error, stage, committed }
            }
        };
        branch.actions.push(index.0);
        branch.legal = legal;
        let truncated = !terminated && branch.steps >= max_steps;
        branch.status = if terminated { Status::Terminal } else if truncated { Status::Truncated } else { Status::Live };
        let token = RunDecisionToken([b.0, branch.serial]);
        BranchStepOutcome::Stepped(BranchStep { observation, action_mask, token, reward, terminated, truncated, info: Self::info(branch.steps, branch.status, info) })
    }
    /// M35 batched branch stepping (design §3.6, G8): every entry is checked first (a repeated branch, a branch that is
    /// not live, closed, foreign, faulted, terminal or truncated, a stale token, an illegal index, a hidden-order pick, or a
    /// total-step budget that cannot cover every entry rejects the whole call naming the first offending entry, with no
    /// effect). The selected worlds then step in parallel on disjoint borrows, each through exactly `step`'s resolver; the
    /// arena records counters, step counts, truncations, faults and accepted actions after the join in input order. So the
    /// result equals sequential `step` calls in input order and depends on no other entry, grouping or worker count.
    /// A fault freezes only its branch (`Fault` outcome); siblings are unaffected and nothing is retried.
    pub fn step_many(&mut self, steps: &[BranchStepRequest], view: BranchView) -> Result<Vec<BranchStepOutcome>, Error> {
        let mut seen = std::collections::BTreeSet::new();
        let mut actions = Vec::new();
        actions.try_reserve_exact(steps.len()).map_err(|_| Error::DeliveryFault("step_many allocation failed"))?;
        for s in steps {
            if !seen.insert(s.branch.0) { return Err(Error::Input("a branch appears twice in one step_many call")) }
            actions.push(self.preflight(s.branch, s.token, s.index)?);
        }
        if (steps.len() as u64) > self.limits.max_total_steps.saturating_sub(self.total_steps) { return Err(Error::Budget("total step limit reached")) }
        let scheduler = crate::workers::Scheduler::shared(self.workers).map_err(|_| Error::DeliveryFault("search worker pool unavailable"))?;
        let (caps, max_steps) = (self.root.caps.clone(), self.limits.max_steps_per_branch);
        // Disjoint borrows in sorted id order; each job owns its entry's action and touches only its own world.
        let position: std::collections::BTreeMap<u64, usize> = steps.iter().enumerate().map(|(i, s)| (s.branch.0, i)).collect();
        let mut worlds: Vec<(usize, &mut Branch)> = self.branches.iter_mut().filter_map(|(id, b)| position.get(id).map(|i| (*i, &mut **b))).collect();
        let jobs: Vec<GameAction> = worlds.iter().map(|(i, _)| actions[*i].clone()).collect();
        let advanced = scheduler.map_mut(&mut worlds, jobs, |_, (_, branch), action| {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| advance(&mut branch.run, &action, view, &caps))) {
                Ok(advanced) => advanced,
                Err(_) => Advanced::Fault { error: Error::DeliveryFault(BRANCH_CORE_FAULT), stage: "panic", committed: true },
            }
        }).map_err(|_| Error::DeliveryFault("step_many allocation failed"))?;
        let mut by_input: Vec<Option<Advanced>> = (0..steps.len()).map(|_| None).collect();
        for ((i, _), a) in worlds.iter().zip(advanced) { by_input[*i] = Some(a); }
        drop(worlds);
        self.total_steps += steps.len() as u64;
        Ok(steps.iter().zip(by_input).map(|(s, a)| {
            let branch = self.branches.get_mut(&s.branch.0).unwrap();
            Self::settle(s.branch, branch, s.index, a.expect("every entry was stepped"), max_steps)
        }).collect())
    }
    /// An exact copy of a live branch inside its sampled world (new id and tokens); inherits its step count.
    pub fn fork(&mut self, b: BranchId) -> Result<BranchId, Error> {
        let branch = self.get(b)?;
        match branch.status { Status::Faulted => return Err(Error::BranchFaulted), Status::Terminal | Status::Truncated => return Err(Error::Terminal), Status::Live => {} }
        self.admit_branch()?;
        let copy = Branch { serial: 0, ..branch.clone() };
        self.insert(copy)
    }
    /// The branch's accepted action indices since its root (readable after a fault or the terminal; not after close).
    pub fn actions(&self, b: BranchId) -> Result<Vec<i64>, Error> { Ok(self.get(b)?.actions.clone()) }
    /// Frees the branch; later use is `Closed`.
    pub fn close(&mut self, b: BranchId) -> Result<(), Error> {
        self.get(b)?;
        self.branches.remove(&b.0);
        self.closed.insert(b.0);
        Ok(())
    }
    /// A new public root from a live branch (re-determinisation).
    pub fn public_root(&self, b: BranchId) -> Result<PublicRoot, Error> {
        let branch = self.get(b)?;
        match branch.status { Status::Faulted => return Err(Error::BranchFaulted), Status::Terminal | Status::Truncated => return Err(Error::Terminal), Status::Live => {} }
        PublicRoot::of(&branch.run, &self.root.caps)
    }
    pub fn counters(&self) -> Value {
        json!({"live_branches": self.branches.len(), "branches_created": self.created, "branches_closed": self.closed.len(), "total_steps": self.total_steps})
    }
    pub fn spec(&self) -> Value {
        let l = self.limits;
        json!({"search_id": PUBLIC_SEARCH_ID, "belief_id": PUBLIC_BELIEF_ID, "knowledge_id": PUBLIC_KNOWLEDGE_ID,
            "combat_view_id": crate::combat_policy::STANDARD_COMBAT_VIEW_ID,
            "observation_fingerprint": crate::combat_policy::hex(&crate::standard_combat_observation_spec(&self.root.caps).map(|s| s.fingerprint()).unwrap_or([0; 32])),
            "root_digest": crate::combat_policy::hex(&self.root.digest), "horizon": "combat",
            "limits": {"max_live_branches": l.max_live_branches, "max_branches": l.max_branches, "max_steps_per_branch": l.max_steps_per_branch, "max_total_steps": l.max_total_steps},
            "hard_caps": {"max_live_branches": SEARCH_HARD_CAPS.max_live_branches, "max_branches": SEARCH_HARD_CAPS.max_branches, "max_steps_per_branch": SEARCH_HARD_CAPS.max_steps_per_branch, "max_total_steps": SEARCH_HARD_CAPS.max_total_steps},
            "max_retained_bytes": search_retained_bytes_bound(&l, &self.root.caps),
            "retained_bytes": {"fixed": search_fixed_bytes(), "per_live_branch": branch_retained_bytes_bound(&l, &self.root.caps),
                "per_created_branch": created_branch_bytes(), "run_estimate_is_measured": true},
            "branch_retained_bytes_bound": branch_retained_bytes_bound(&l, &self.root.caps),
            "branch_retained_bytes": {"run_estimate": BRANCH_RUN_BYTES_ESTIMATE, "knowledge_cap": spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP,
                "action_history": 2 * l.max_steps_per_branch * std::mem::size_of::<i64>() as u64, "legal_actions": self.root.caps.actions * std::mem::size_of::<GameAction>(),
                "branch_entry": branch_record_bytes() + collection_entry_bytes(BOX_SLOT_BYTES)},
            "reward": "combat outcome at the combat's own terminal: victory +1, death -1, escape 0; truncated at max_steps_per_branch (end_reason step_limit)"})
    }
    /// TEST-ONLY: a branch's world (fault injection and P3 comparisons).
    #[cfg(test)]
    pub(crate) fn branch_run_for_test(&mut self, b: BranchId) -> &mut Run { &mut self.branches.get_mut(&b.0).unwrap().run }
    /// TEST-ONLY: a branch's key material (its sample key; M35 R8 privacy oracle on the batched path).
    #[cfg(test)]
    pub(crate) fn origin_for_test(&self, b: BranchId) -> [u8; 32] { self.branches[&b.0].origin }
    /// TEST-ONLY: the number of recorded sample indices (final review v3).
    #[cfg(test)]
    pub(crate) fn used_samples_for_test(&self) -> usize { self.used_samples.len() }
    /// TEST-ONLY: the live branch ids, ascending.
    #[cfg(test)]
    pub(crate) fn branches_for_test(&self) -> Vec<BranchId> { self.branches.keys().map(|id| BranchId(*id)).collect() }
    /// TEST-ONLY: the capacities branches publish with (post-commit publication error, final review M3).
    #[cfg(test)]
    pub(crate) fn root_caps_for_test(&mut self) -> &mut RunCapacities { &mut self.root.caps }
}

#[cfg(test)]
thread_local! {
    /// TEST-ONLY: simulate an exhausted shared session-id allocator (final review v3).
    pub(crate) static FAIL_ID_ALLOCATION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
