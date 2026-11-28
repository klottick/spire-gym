//! M34s-b search arena `spire-gym-public-search-v1` (design §4.7) over the public belief `spire-gym-public-belief-v1`
//! (§4.6). A [`PublicRoot`] is the canonical combat-scope Run of a public decision (no hidden value of the live world);
//! a [`PublicSearch`] owns that root and every branch world sampled from it with a caller research seed, and holds no
//! reference to the live session. Branches are observed exactly like `standard-combat-v1` sessions (the Standard
//! combat view), carry their own tokens from the shared session allocator, and end at the combat's own terminal.
use crate::{RunObservationLayout, RunObservationSpec, StandardCombatSessionConfig};
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
/// on (a branch applies a label-free rule to a pick among identical draw-pile copies where it is exact, and refuses the rest, `HIDDEN_ORDER_PICK`).
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
/// spire-3zm: a non-bound core error keeps only its closed class and static checkpoint (`RunError::fault_class`).
fn redacted(core: spire_core::run::RunError) -> Error {
    core.engine_bound().map_or_else(|| Error::SearchCoreFault(core.fault_class()), Error::EngineBound)
}
/// The redacted fault of a panic contained by the arena's unwind guard.
const PANIC_FAULT: Error = Error::SearchCoreFault(spire_core::run::CoreFaultClass::PANIC);
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
/// 372,632 bytes peak excluding knowledge in the 999-card and high-generation heap probes.
/// Round two times the sample peak to the next 64 KiB; this estimates the measured envelope.
const COMPACT_RUN_ESTIMATE: u64 = 768 * 1024;
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
    branch_bytes(limits, caps.actions, BRANCH_RUN_BYTES_ESTIMATE)
}
fn branch_bytes(limits: &SearchLimits, legal_width: usize, run_estimate: u64) -> u64 {
    run_estimate + spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP as u64
        + 2 * limits.max_steps_per_branch * std::mem::size_of::<i64>() as u64
        + (legal_width * std::mem::size_of::<GameAction>()) as u64
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
    fixed_bytes(BRANCH_RUN_BYTES_ESTIMATE)
}
fn fixed_bytes(run_estimate: u64) -> u64 {
    std::mem::size_of::<PublicSearch>() as u64 + run_estimate + spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP as u64
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
/// M41 (spire-n6b): the declared public playout rule, RNG domain and stop semantics of [`PublicSearch::playout_many`].
pub const PUBLIC_PLAYOUT_RULE_ID: &str = "spire-gym-public-playout-v1";
/// M42 (spire-9wa): the declared tiered-verb public playout rule (never-pick and last-resort verb tiers).
pub const PUBLIC_PLAYOUT_TIERED_RULE_ID: &str = "spire-gym-public-playout-tiered-v1";
/// The verbs Learn's rule excludes by default (`View.heuristic_choice`).
pub const DEFAULT_PLAYOUT_EXCLUDED_VERBS: [&str; 2] = ["end_turn", "discard_potion"];
/// M42: the default never-pick verbs for [`PlayoutRule::tiered_verbs`] (chosen only when no other legal action exists).
pub const DEFAULT_PLAYOUT_NEVER_VERBS: [&str; 2] = ["use_potion", "discard_potion"];
/// M42: the default last-resort verbs for [`PlayoutRule::tiered_verbs`] (chosen only when nothing outside both sets is legal).
pub const DEFAULT_PLAYOUT_LAST_RESORT_VERBS: [&str; 1] = ["end_turn"];
/// The `refusal` code of a playout stopped at a pick among identical draw-pile copies that depends on hidden order.
pub const REFUSAL_HIDDEN_ORDER_PICK: &str = "HIDDEN_ORDER_PICK";
/// The `refusal` code of a playout stopped at a live boundary with no legal action (defensive; no known path).
pub const REFUSAL_NO_LEGAL_ACTION: &str = "NO_LEGAL_ACTION";
/// The Standard `verb` family after `padding`, in vocab order (pinned by test against the vocab manifest).
pub const STANDARD_PLAYOUT_VERBS: [&str; 15] = ["move", "choose", "select", "confirm", "cancel", "use_potion", "discard_potion", "play_card",
    "combat_select_card", "end_turn", "confirm_selection", "choose_stance", "combat_deselect_card", "choose_card_offer", "skip_card_offer"];
/// The Standard verb name of an action (exhaustive, no wildcard: a new action variant must be mapped here). Equals the
/// encoder's `actions[:, verb]` vocab entry (M41 T2 parity test).
fn standard_verb_name(action: &GameAction) -> &'static str {
    use spire_core::combat::CombatAction as C;
    match action {
        GameAction::Move(_) => "move", GameAction::Choose(_) => "choose", GameAction::Select(_) => "select",
        GameAction::Confirm => "confirm", GameAction::Cancel => "cancel",
        GameAction::UsePotion { .. } => "use_potion", GameAction::DiscardPotion { .. } => "discard_potion",
        GameAction::Combat(c) => match c {
            C::ChooseStance { .. } => "choose_stance", C::DeselectCard { .. } => "combat_deselect_card", C::PlayCard { .. } => "play_card",
            C::SelectCard { .. } => "combat_select_card", C::ConfirmSelection => "confirm_selection", C::UsePotion { .. } => "use_potion",
            C::DiscardPotion { .. } => "discard_potion", C::EndTurn => "end_turn", C::ChooseCardOffer { .. } => "choose_card_offer",
            C::SkipCardOffer => "skip_card_offer",
        },
    }
}
/// The Standard vocab id of an action's verb.
pub(crate) fn standard_verb_id(action: &GameAction) -> i64 {
    static IDS: std::sync::OnceLock<std::collections::BTreeMap<&'static str, i64>> = std::sync::OnceLock::new();
    IDS.get_or_init(|| STANDARD_PLAYOUT_VERBS.iter().map(|v| (*v, crate::run_schema_standard::verb(v).expect("a Standard verb"))).collect())[standard_verb_name(action)]
}
/// M41/M42: declared public playout rule over legal-action Standard verbs.
/// Valid by construction (verb names are resolved to Standard vocab ids when the rule is built).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayoutRule { kind: PlayoutRuleKind }
#[derive(Clone, Debug, PartialEq, Eq)]
enum PlayoutRuleKind {
    UniformExclude { excluded: u64 },
    Tiered { never: u64, last_resort: u64 },
}
fn parse_verb_mask<I: IntoIterator<Item = S>, S: AsRef<str>>(verbs: I) -> Result<u64, Error> {
    let mut mask = 0u64;
    for verb in verbs {
        let id = crate::run_schema_standard::verb(verb.as_ref()).ok().filter(|id| (1..64).contains(id)).ok_or(Error::Input("unknown playout verb"))?;
        mask |= 1 << id;
    }
    Ok(mask)
}
fn verbs_from_mask(mask: u64) -> Vec<&'static str> {
    let mut v = STANDARD_PLAYOUT_VERBS.iter().copied().filter(|n| mask & (1 << crate::run_schema_standard::verb(n).expect("a Standard verb")) != 0).collect::<Vec<_>>();
    v.sort_by_key(|n| crate::run_schema_standard::verb(n).expect("a Standard verb"));
    v
}
impl PlayoutRule {
    /// M41: uniform over the legal actions whose Standard verb is not excluded; over all legal actions when none remain.
    /// Unknown names and `padding` are `Input("unknown playout verb")`; repeats are ignored; no names means all legal actions.
    pub fn uniform_exclude_verbs<I: IntoIterator<Item = S>, S: AsRef<str>>(verbs: I) -> Result<Self, Error> {
        Ok(Self { kind: PlayoutRuleKind::UniformExclude { excluded: parse_verb_mask(verbs)? } })
    }
    /// M42: tiered-verb rule: uniform among legal actions outside `never_verbs` and `last_resort_verbs`, else uniform
    /// among `last_resort_verbs`, else uniform among all legal actions. Rejects unknown/`padding` names and overlap
    /// between `never_verbs` and `last_resort_verbs`.
    pub fn tiered_verbs<I1: IntoIterator<Item = S1>, S1: AsRef<str>, I2: IntoIterator<Item = S2>, S2: AsRef<str>>(
        never_verbs: I1, last_resort_verbs: I2,
    ) -> Result<Self, Error> {
        let never = parse_verb_mask(never_verbs)?;
        let last_resort = parse_verb_mask(last_resort_verbs)?;
        if never & last_resort != 0 {
            return Err(Error::Input("playout verb cannot be both never-pick and last-resort"));
        }
        Ok(Self { kind: PlayoutRuleKind::Tiered { never, last_resort } })
    }
    /// M42: the default tiered-verb rule (`never_verbs`: `use_potion`, `discard_potion`; `last_resort_verbs`: `end_turn`).
    pub fn default_tiered() -> Self {
        Self::tiered_verbs(DEFAULT_PLAYOUT_NEVER_VERBS, DEFAULT_PLAYOUT_LAST_RESORT_VERBS)
            .expect("default tiered playout verbs are disjoint Standard verbs")
    }
    /// The declared rule identifier for this rule variant.
    pub fn rule_id(&self) -> &'static str {
        match self.kind {
            PlayoutRuleKind::UniformExclude { .. } => PUBLIC_PLAYOUT_RULE_ID,
            PlayoutRuleKind::Tiered { .. } => PUBLIC_PLAYOUT_TIERED_RULE_ID,
        }
    }
    /// The excluded verb names for `uniform_exclude_verbs`, in vocab order (empty for `tiered_verbs`).
    pub fn excluded_verbs(&self) -> Vec<&'static str> {
        match self.kind { PlayoutRuleKind::UniformExclude { excluded } => verbs_from_mask(excluded), PlayoutRuleKind::Tiered { .. } => vec![] }
    }
    /// The never-pick verb names for `tiered_verbs`, in vocab order (empty for `uniform_exclude_verbs`).
    pub fn never_verbs(&self) -> Vec<&'static str> {
        match self.kind { PlayoutRuleKind::Tiered { never, .. } => verbs_from_mask(never), PlayoutRuleKind::UniformExclude { .. } => vec![] }
    }
    /// The last-resort verb names for `tiered_verbs`, in vocab order (empty for `uniform_exclude_verbs`).
    pub fn last_resort_verbs(&self) -> Vec<&'static str> {
        match self.kind { PlayoutRuleKind::Tiered { last_resort, .. } => verbs_from_mask(last_resort), PlayoutRuleKind::UniformExclude { .. } => vec![] }
    }
}
impl Default for PlayoutRule {
    fn default() -> Self { Self::uniform_exclude_verbs(DEFAULT_PLAYOUT_EXCLUDED_VERBS).expect("default playout verbs are Standard verbs") }
}
/// M41: one `playout_many` call's rule, optional per-call step cap (`Some(0)` is rejected) and playout seed (mixed into
/// the research pick; 0 keeps common random numbers across branches of one sample).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlayoutConfig { pub rule: PlayoutRule, pub max_steps: Option<u64>, pub playout_seed: u64 }
/// One `playout_many` entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchPlayoutRequest { pub branch: BranchId, pub token: RunDecisionToken }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemyHp { pub hp: i32, pub max_hp: i32 }
/// The final public HP and resource state of a playout: the published player row (`player_hp`, `player_max_hp`,
/// `player_gold`), enemy rows (the enemies the Standard view shows, in row order), and slot-indexed Standard `potion`
/// vocab ids (`0` for an empty slot, `> 0` for a held potion, matching `observation["potions"][:, 0]`).
/// For any terminal branch (`terminated == true`), `player_hp` equals the combat's `hp_after_cleanup`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlayoutSummary { pub player_hp: i32, pub player_max_hp: i32, pub player_gold: u32, pub enemies: Vec<EnemyHp>, pub potions: Vec<i32> }
/// A finished playout. `terminated`/`truncated` are exactly the flags of its last step taken one action at a time (both
/// false with no step). `end_reason`: `victory`, `death`, `escaped`, `step_limit` (truncated at `max_steps_per_branch`),
/// `max_steps` (the call cap; the branch is live) or `refused` (`refusal` names the stop; no effect).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchPlayout {
    pub token: RunDecisionToken, pub reward: i32, pub terminated: bool, pub truncated: bool,
    pub end_reason: &'static str, pub refusal: Option<&'static str>,
    /// Steps attempted in this call (charged to the total budget) and the branch's steps since its root after the call.
    pub steps: u64, pub branch_steps: u64,
    pub summary: PlayoutSummary,
}
/// One `playout_many` outcome, in input order: the finished playout, or its branch's fault (as `BranchStepOutcome::Fault`,
/// plus the step counts). A faulted branch is frozen; its accepted prefix stays readable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchPlayoutOutcome {
    Finished(BranchPlayout),
    Fault { error: Error, stage: &'static str, committed: bool, steps: u64, branch_steps: u64 },
}
/// A playout's result inside its worker job (the arena adds the token after the join).
enum PlayoutJob {
    Finished { reward: i32, terminated: bool, truncated: bool, end_reason: &'static str, refusal: Option<&'static str>, steps: u64, summary: PlayoutSummary },
    Fault { error: Error, stage: &'static str, committed: bool, steps: u64 },
}
fn playout_summary(combat: Option<&spire_core::combat::CombatPublic>, gold: u32) -> Result<PlayoutSummary, Error> {
    let Some(c) = combat else { return Ok(PlayoutSummary { player_gold: gold, ..PlayoutSummary::default() }) };
    let inventory = c.inventory.as_ref().ok_or(Error::Input("combat inventory missing"))?;
    let mut potions = Vec::with_capacity(inventory.potions.len());
    for slot in &inventory.potions {
        let kind = match slot {
            None => 0,
            Some(id) => crate::run_schema_standard::potion(*id)? as i32,
        };
        potions.push(kind);
    }
    Ok(PlayoutSummary {
        player_hp: c.player.hp, player_max_hp: c.player.max_hp, player_gold: gold,
        enemies: c.enemies.iter().filter(|e| crate::run_observation::standard_enemy_present(e)).map(|e| EnemyHp { hp: e.hp, max_hp: e.max_hp }).collect(),
        potions,
    })
}
/// The rule's candidate index at a boundary (M41 design-v2 §2.4, M42 design-v1 §2.2); `None` for an empty legal table.
fn playout_pick(legal: &[GameAction], rule: &PlayoutRule, origin: &[u8; 32], seed: u64, ordinal: u64) -> Option<usize> {
    match rule.kind {
        PlayoutRuleKind::UniformExclude { excluded } => {
            let preferred = legal.iter().filter(|a| excluded & (1 << standard_verb_id(a)) == 0).count();
            if preferred > 0 {
                let n = spire_core::run::research_playout_pick(origin, seed, ordinal, preferred);
                return legal.iter().enumerate().filter(|(_, a)| excluded & (1 << standard_verb_id(a)) == 0).nth(n).map(|(i, _)| i)
            }
            (!legal.is_empty()).then(|| spire_core::run::research_playout_pick(origin, seed, ordinal, legal.len()))
        }
        PlayoutRuleKind::Tiered { never, last_resort } => {
            let avoid = never | last_resort;
            let (mut primary, mut resort) = (0usize, 0usize);
            for a in legal {
                let bit = 1u64 << standard_verb_id(a);
                if bit & avoid == 0 { primary += 1; } else if bit & last_resort != 0 { resort += 1; }
            }
            if primary > 0 {
                let n = spire_core::run::research_playout_pick(origin, seed, ordinal, primary);
                return legal.iter().enumerate().filter(|(_, a)| (1u64 << standard_verb_id(a)) & avoid == 0).nth(n).map(|(i, _)| i)
            }
            if resort > 0 {
                let n = spire_core::run::research_playout_pick(origin, seed, ordinal, resort);
                return legal.iter().enumerate().filter(|(_, a)| (1u64 << standard_verb_id(a)) & last_resort != 0).nth(n).map(|(i, _)| i)
            }
            (!legal.is_empty()).then(|| spire_core::run::research_playout_pick(origin, seed, ordinal, legal.len()))
        }
    }
}
/// One step of a playout, inside the per-step unwind guard.
enum PlayoutStep { Stop(&'static str), Advanced(usize, Result<PublicStep, StepFault>) }
/// One branch's playout on its own world (design-v2 §5.2): each step picks, resolves or refuses a hidden-order pick, runs
/// the shared Mask step under an unwind guard and settles the branch record before the next step.
fn play_branch(id: u64, branch: &mut Branch, cap: u64, max_steps: u64, caps: &RunCapacities, rule: &PlayoutRule, seed: u64) -> PlayoutJob {
    let mut steps = 0u64;
    let mut last: Option<PublicStep> = None;
    let (end_reason, refusal) = loop {
        if steps == cap { break ("max_steps", None) }
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let ordinal = branch.actions.len() as u64;
            #[cfg(test)]
            if PANIC_PLAYOUT_AT.with(std::cell::Cell::get) == Some((id, ordinal)) { panic!("M41 test panic") }
            let Some(i) = playout_pick(&branch.legal, rule, &branch.origin, seed, ordinal) else { return PlayoutStep::Stop(REFUSAL_NO_LEGAL_ACTION) };
            let mut action = branch.legal[i].clone();
            if branch.run.pick_depends_on_hidden_order(&action) {
                match branch.run.resolve_identical_pick(&action, &branch.origin, ordinal) { Some(resolved) => action = resolved, None => return PlayoutStep::Stop(REFUSAL_HIDDEN_ORDER_PICK) }
            }
            PlayoutStep::Advanced(i, advance_public(&mut branch.run, &action, caps))
        }));
        let (index, result) = match attempt {
            Ok(PlayoutStep::Stop(code)) => break ("refused", Some(code)),
            Ok(PlayoutStep::Advanced(i, r)) => (i as i64, r),
            Err(_) => (0, Err((PANIC_FAULT, "panic", true))),
        };
        steps += 1;
        match result {
            Ok(step) => {
                let terminated = step.terminal.is_some();
                let legal = std::mem::take(&mut last.insert(step).legal);
                let truncated = PublicSearch::settle_record(branch, index, Ok((legal, terminated)), max_steps);
                if terminated || truncated { break ("", None) }
            }
            Err((error, stage, committed)) => {
                PublicSearch::settle_record(branch, index, Err(stage), max_steps);
                return PlayoutJob::Fault { error, stage, committed, steps }
            }
        }
    };
    let (summary, reward, terminated, truncated, end_reason) = match &last {
        Some(step) => {
            let terminal = step.terminal;
            let truncated = branch.status == Status::Truncated;
            let reason = if end_reason.is_empty() { terminal.map_or("step_limit", |t| crate::combat_training::outcome_label(t.outcome)) } else { end_reason };
            #[cfg(test)]
            let failed = FAIL_PLAYOUT_SUMMARY_BRANCH.with(std::cell::Cell::get) == Some(id);
            #[cfg(not(test))]
            let (failed, _) = (false, id);
            let summary = if failed {
                Err(Error::DeliveryFault(BRANCH_CORE_FAULT))
            } else {
                playout_summary(step.combat.as_ref(), step.gold)
            };
            match summary {
                Ok(summary) => (summary, terminal.map_or(0, |t| crate::combat_training::outcome_reward(t.outcome)), terminal.is_some(), truncated, reason),
                Err(e) => {
                    branch.status = Status::Faulted;
                    branch.legal.clear();
                    return PlayoutJob::Fault { error: redact(e), stage: "encode", committed: true, steps }
                }
            }
        }
        None => {
            // No step in this call (an immediate refusal): read the branch's current published boundary.
            #[cfg(test)]
            let failed = FAIL_PLAYOUT_SUMMARY.with(std::cell::Cell::get);
            #[cfg(not(test))]
            let failed = false;
            let summary = if failed {
                Err(Error::DeliveryFault(BRANCH_CORE_FAULT))
            } else {
                branch.run.decision().map_err(redacted).and_then(|d| playout_summary(d.combat.as_ref(), d.player.gold).map_err(redact))
            };
            match summary {
                Ok(summary) => (summary, 0, false, false, end_reason),
                Err(error) => {
                    branch.status = Status::Faulted;
                    branch.legal.clear();
                    return PlayoutJob::Fault { error, stage: "encode", committed: false, steps: 0 }
                }
            }
        }
    };
    PlayoutJob::Finished { reward, terminated, truncated, end_reason, refusal, steps, summary }
}
/// A world after one action: its publication, or its fault.
enum Advanced {
    Done { observation: Option<RunObservation>, action_mask: Vec<i8>, info: Value, legal: Vec<GameAction>, reward: i32, terminated: bool },
    Fault { error: Error, stage: &'static str, committed: bool },
}
/// M41 (design-v2 §5.1): the Mask step on a world, shared by `step`/`step_many` with `BranchView::Mask` and by
/// `playout_many`: the shared resolver, the combat-scope terminal, the published decision and the action capacity, with
/// no view encoding. An error carries its fault stage and whether the engine committed.
struct PublicStep { legal: Vec<GameAction>, terminal: Option<spire_core::run::CombatScopeTerminal>, combat: Option<spire_core::combat::CombatPublic>, gold: u32 }
type StepFault = (Error, &'static str, bool);
fn advance_public(run: &mut Run, action: &GameAction, caps: &RunCapacities) -> Result<PublicStep, StepFault> {
    if let Err(e) = run.step(action) {
        let committed = matches!(e, spire_core::run::RunError::Faulted(_));
        return Err((redacted(e), "step", committed))
    }
    let terminal = run.combat_scope_terminal();
    // Final review M3: the engine has committed, so a publication error faults the branch.
    let d = run.decision().map_err(|e| (redacted(e), "encode", true))?;
    let gold = d.player.gold;
    let combat = d.combat;
    let legal: Vec<GameAction> = if terminal.is_some() { vec![] } else { d.candidates.into_iter().map(|c| c.action).collect() };
    if legal.len() > caps.actions { return Err((Error::Capacity { group: "actions", required: legal.len(), capacity: caps.actions }, "encode", true)) }
    Ok(PublicStep { legal, terminal, combat, gold })
}
/// The single-branch step on a world: the shared resolver, then the requested view (`step` and `step_many` both use it).
fn advance(run: &mut Run, action: &GameAction, view: BranchView, caps: &RunCapacities, layout: RunObservationLayout) -> Advanced {
    if view == BranchView::Mask {
        return match advance_public(run, action, caps) {
            Ok(PublicStep { legal, terminal, .. }) => {
                let mut mask = vec![0; caps.actions];
                mask[..legal.len()].iter_mut().for_each(|m| *m = 1);
                let info = json!({"terminal_reason": terminal.map(|t| crate::combat_training::outcome_label(t.outcome))});
                let reward = terminal.map_or(0, |t| crate::combat_training::outcome_reward(t.outcome));
                Advanced::Done { observation: None, action_mask: mask, info, legal, reward, terminated: terminal.is_some() }
            }
            Err((error, stage, committed)) => Advanced::Fault { error, stage, committed },
        }
    }
    if let Err(e) = run.step(action) {
        let committed = matches!(e, spire_core::run::RunError::Faulted(_));
        return Advanced::Fault { error: redacted(e), stage: "step", committed }
    }
    let terminal = run.combat_scope_terminal();
    let reward = terminal.map_or(0, |t| crate::combat_training::outcome_reward(t.outcome));
    let published = crate::combat_training::standard_combat_publication_into(run, caps, layout, None).map(|(o, info, legal)| {
        let mask = o.action_mask.clone();
        (Some(o), mask, info, legal)
    });
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
/// M36 (spire-dmm M5): the most `step_many` workers a search admits, equal to the batch slot ceiling.
pub const MAX_SEARCH_WORKERS: usize = 256;
pub struct PublicSearch { root: PublicRoot, research_root: [u8; 32], limits: SearchLimits, branches: std::collections::BTreeMap<u64, Box<Branch>>,
    closed: std::collections::BTreeSet<u64>, used_samples: std::collections::BTreeSet<u64>, next_auto: u64, created: u64, total_steps: u64,
    /// M35 `step_many` workers (execution configuration, not a budget; design §3.6).
    workers: usize,
    /// M36 (spire-dmm M5): the shared pool, taken on the first parallel `step_many` and released when the search drops.
    pool: Option<crate::workers::Scheduler>,
    compact: Option<Box<StandardCombatSessionConfig>>,
}
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
            used_samples: Default::default(), next_auto: 0, created: 0, total_steps: 0, workers: 1, pool: None, compact: None })
    }
    pub fn new_with_combat_layout(root: PublicRoot, config: SearchConfig, observation: StandardCombatSessionConfig) -> Result<Self, Error> {
        observation.validate()?;
        let mut search = Self::new(root, config)?;
        if observation.layout == RunObservationLayout::Compact { search.compact = Some(Box::new(observation)); }
        Ok(search)
    }

    pub fn observation_layout(&self) -> RunObservationLayout {
        if self.compact.is_some() { RunObservationLayout::Compact } else { RunObservationLayout::Dense }
    }

    fn view_capacities(&self) -> &RunCapacities {
        self.compact.as_ref().map_or(&self.root.caps, |c| &c.capacities)
    }

    pub fn action_capacity(&self) -> usize { self.view_capacities().actions }

    pub fn observation_spec(&self) -> Result<RunObservationSpec, Error> {
        match self.compact.as_ref().and_then(|c| c.capacity_preset) {
            Some(preset) => crate::standard_compact_combat_observation_spec(self.view_capacities(), preset),
            None => crate::standard_combat_observation_spec(&self.root.caps),
        }
    }

    #[doc(hidden)]
    pub fn zero_observation(&self) -> Result<RunObservation, Error> {
        crate::run_observation::empty_combat_observation(self.view_capacities())
    }
    /// M35: `step_many` runs on `workers` threads of the shared pool for that count (1: no pool). M36: 1 ≤ `workers` ≤
    /// `MAX_SEARCH_WORKERS`; the search holds its pool from its first parallel `step_many` until it drops or its worker
    /// count changes.
    pub fn with_workers(mut self, workers: usize) -> Result<Self, Error> {
        if !(1..=MAX_SEARCH_WORKERS).contains(&workers) { return Err(Error::Input("search workers must be 1..=256")) }
        if workers != self.workers { self.pool = None; }
        self.workers = workers;
        Ok(self)
    }
    pub fn workers(&self) -> usize { self.workers }
    /// TEST-ONLY (M36): the pool this search took on its first parallel `step_many`.
    #[cfg(test)]
    pub(crate) fn pool_for_test(&self) -> Option<std::sync::Weak<rayon::ThreadPool>> {
        match &self.pool { Some(crate::workers::Scheduler::Parallel(p)) => Some(std::sync::Arc::downgrade(p)), _ => None }
    }
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
        let legal = if let Some(config) = &self.compact {
            crate::combat_training::standard_combat_publication_into(&run, &config.capacities, config.layout, None).map_err(redact)?.2
        } else {
            run.decision().map_err(redacted)?.candidates.into_iter().map(|c| c.action).collect()
        };
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
    /// Encode the current full boundary on demand. For a faulted branch, replay the accepted prefix (the token is spent).
    /// That prefix can include a committed action whose full publication failed and can return the same capacity fault.
    pub fn decision(&self, b: BranchId) -> Result<BranchBoundary, Error> {
        let branch = self.get(b)?;
        let replayed;
        let run = if branch.status == Status::Faulted { replayed = self.replay(branch)?; &replayed } else { &branch.run };
        let (observation, info, _) = crate::combat_training::standard_combat_publication_into(run, self.view_capacities(), self.observation_layout(), None).map_err(redact)?;
        Ok(BranchBoundary { observation, token: RunDecisionToken([b.0, branch.serial]), info: Self::info(branch.steps, branch.status, info) })
    }
    fn replay(&self, branch: &Branch) -> Result<Run, Error> {
        let mut run = self.root.run.belief_sample(&branch.origin).map_err(|u| Error::UnsupportedBoundary(u.reason()))?;
        for (ordinal, index) in branch.actions.iter().enumerate() {
            let d = run.decision().map_err(redacted)?;
            let mut action = usize::try_from(*index).ok().and_then(|i| d.candidates.get(i)).map(|c| c.action.clone()).ok_or(Error::DeliveryFault("branch replay diverged"))?;
            if run.pick_depends_on_hidden_order(&action) {
                action = run.resolve_identical_pick(&action, &branch.origin, ordinal as u64).ok_or(Error::DeliveryFault("branch replay diverged"))?;
            }
            run.step(&action).map_err(redacted)?;
        }
        Ok(run)
    }
    /// One action through the shared resolver. Budget, stale-token, illegal-index and hidden-order-pick
    /// (`UnsupportedBoundary(HIDDEN_ORDER_PICK)`) errors have no effect; the branch stays live.
    pub fn step(&mut self, b: BranchId, token: RunDecisionToken, index: RunActionIndex, view: BranchView) -> Result<BranchStep, Error> {
        let action = self.preflight(b, token, index)?;
        if self.total_steps >= self.limits.max_total_steps { return Err(Error::Budget("total step limit reached")) }
        let (caps, layout, max_steps) = (self.view_capacities().clone(), self.observation_layout(), self.limits.max_steps_per_branch);
        self.total_steps += 1;
        let branch = self.branches.get_mut(&b.0).unwrap();
        let advanced = advance(&mut branch.run, &action, view, &caps, layout);
        match Self::settle(b, branch, index, advanced, max_steps) { BranchStepOutcome::Stepped(step) => Ok(step), BranchStepOutcome::Fault { error, .. } => Err(error) }
    }
    /// The no-effect checks of `step` for one entry; returns the action.
    fn preflight(&self, b: BranchId, token: RunDecisionToken, index: RunActionIndex) -> Result<GameAction, Error> {
        let branch = self.check_branch(b, Some(token))?;
        let action = usize::try_from(index.0).ok().and_then(|i| branch.legal.get(i)).cloned().ok_or(Error::Input("branch action index is not legal"))?;
        // Final review I1: a sampled world keeps the true labels of identical copies, so a pick that removes a copy chosen
        // by hidden instance order would make the branch depend on hidden state; refused with no effect (fail closed).
        // M40a: where the pick can be made independent of labels (single-pick grid, exchangeable copies, draw order not
        // revealed) the branch applies the rule; every other such pick keeps the refusal.
        if branch.run.pick_depends_on_hidden_order(&action) {
            return branch.run.resolve_identical_pick(&action, &branch.origin, branch.actions.len() as u64).ok_or(Error::UnsupportedBoundary(HIDDEN_ORDER_PICK))
        }
        Ok(action)
    }
    /// M41: the branch checks shared by `step`'s preflight, `playout_many` and `playout_action`, in `step`'s order: known
    /// and open, live, the token (when one is given), the compact action capacity.
    fn check_branch(&self, b: BranchId, token: Option<RunDecisionToken>) -> Result<&Branch, Error> {
        let branch = self.get(b)?;
        match branch.status { Status::Faulted => return Err(Error::BranchFaulted), Status::Terminal | Status::Truncated => return Err(Error::Terminal), Status::Live => {} }
        if token.is_some_and(|t| t != RunDecisionToken([b.0, branch.serial])) { return Err(Error::StaleToken) }
        if let Some(config) = &self.compact && branch.legal.len() > config.capacities.actions {
            return Err(Error::Capacity { group: "actions", required: branch.legal.len(), capacity: config.capacities.actions })
        }
        Ok(branch)
    }
    /// Apply one advanced world to its branch record: serial and step count always advance (the step was attempted); a
    /// fault freezes the branch (never retried or revived; `actions` keeps the accepted prefix).
    fn settle(b: BranchId, branch: &mut Branch, index: RunActionIndex, advanced: Advanced, max_steps: u64) -> BranchStepOutcome {
        match advanced {
            Advanced::Done { observation, action_mask, info, legal, reward, terminated } => {
                let truncated = Self::settle_record(branch, index.0, Ok((legal, terminated)), max_steps);
                let token = RunDecisionToken([b.0, branch.serial]);
                BranchStepOutcome::Stepped(BranchStep { observation, action_mask, token, reward, terminated, truncated, info: Self::info(branch.steps, branch.status, info) })
            }
            Advanced::Fault { error, stage, committed } => {
                Self::settle_record(branch, index.0, Err(stage), max_steps);
                BranchStepOutcome::Fault { error, stage, committed }
            }
        }
    }
    /// M41: the branch-record part of `settle` (shared with `playout_many`): serial and step count advance; on success the
    /// index is accepted and the legal table and status follow the step (returns whether it truncated); a fault at
    /// `stage` freezes the branch, and the accepted prefix includes the index only when the engine step committed and
    /// its publication failed (`encode`); an engine fault or a panic leaves the prefix at the last published boundary.
    fn settle_record(branch: &mut Branch, index: i64, done: Result<(Vec<GameAction>, bool), &'static str>, max_steps: u64) -> bool {
        branch.serial += 1;
        branch.steps += 1;
        match done {
            Ok((legal, terminated)) => {
                branch.actions.push(index);
                branch.legal = legal;
                let truncated = !terminated && branch.steps >= max_steps;
                branch.status = if terminated { Status::Terminal } else if truncated { Status::Truncated } else { Status::Live };
                truncated
            }
            Err(stage) => {
                branch.status = Status::Faulted;
                branch.legal.clear();
                if stage == "encode" { branch.actions.push(index); }
                false
            }
        }
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
        if self.pool.is_none() { self.pool = Some(crate::workers::Scheduler::shared(self.workers).map_err(|_| Error::DeliveryFault("search worker pool unavailable"))?); }
        let (caps, layout, max_steps) = (self.view_capacities().clone(), self.observation_layout(), self.limits.max_steps_per_branch);
        let scheduler = self.pool.as_ref().expect("the search pool was just taken");
        // Disjoint borrows in sorted id order; each job owns its entry's action and touches only its own world.
        let position: std::collections::BTreeMap<u64, usize> = steps.iter().enumerate().map(|(i, s)| (s.branch.0, i)).collect();
        let mut worlds: Vec<(usize, &mut Branch)> = self.branches.iter_mut().filter_map(|(id, b)| position.get(id).map(|i| (*i, &mut **b))).collect();
        let jobs: Vec<GameAction> = worlds.iter().map(|(i, _)| actions[*i].clone()).collect();
        let advanced = scheduler.map_mut(&mut worlds, jobs, |_, (_, branch), action| {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| advance(&mut branch.run, &action, view, &caps, layout))) {
                Ok(advanced) => advanced,
                Err(_) => Advanced::Fault { error: PANIC_FAULT, stage: "panic", committed: true },
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
    /// M41 (spire-n6b, design-v2 §5.2): advance distinct live branches until their combat ends, they reach
    /// `max_steps_per_branch`, the call cap `config.max_steps`, or a declared stop, choosing every move by `config.rule`
    /// over public information (the legal table's Standard verbs) and the playout research stream. Every entry is checked
    /// first, and the worst-case step count must fit the remaining total budget; any failure rejects the whole call naming
    /// the first offending entry, with no effect. Each branch then plays on its own world in parallel through the Mask step
    /// that `step` uses, settled step by step; only attempted steps are charged. A refusal stops its branch with no effect;
    /// a fault freezes only its branch. Results are in input order and depend on no other entry, grouping or worker count,
    /// and equal stepping the same branches one action at a time (`playout_action`, then `step` with `BranchView::Mask`).
    pub fn playout_many(&mut self, requests: &[BranchPlayoutRequest], config: &PlayoutConfig) -> Result<Vec<BranchPlayoutOutcome>, Error> {
        if config.max_steps == Some(0) { return Err(Error::Input("playout max_steps must be positive")) }
        let mut seen = std::collections::BTreeSet::new();
        let mut caps_by_input = Vec::new();
        caps_by_input.try_reserve_exact(requests.len()).map_err(|_| Error::DeliveryFault("playout_many allocation failed"))?;
        let mut worst = 0u128;
        for r in requests {
            if !seen.insert(r.branch.0) { return Err(Error::Input("a branch appears twice in one playout_many call")) }
            let branch = self.check_branch(r.branch, Some(r.token))?;
            let cap = (self.limits.max_steps_per_branch - branch.steps).min(config.max_steps.unwrap_or(u64::MAX));
            worst += u128::from(cap);
            caps_by_input.push(cap);
        }
        if worst > u128::from(self.limits.max_total_steps.saturating_sub(self.total_steps)) { return Err(Error::Budget("total step limit reached")) }
        if self.pool.is_none() { self.pool = Some(crate::workers::Scheduler::shared(self.workers).map_err(|_| Error::DeliveryFault("search worker pool unavailable"))?); }
        let (caps, max_steps) = (self.view_capacities().clone(), self.limits.max_steps_per_branch);
        let scheduler = self.pool.as_ref().expect("the search pool was just taken");
        let position: std::collections::BTreeMap<u64, usize> = requests.iter().enumerate().map(|(i, r)| (r.branch.0, i)).collect();
        let mut worlds: Vec<(u64, &mut Branch)> = self.branches.iter_mut().filter_map(|(id, b)| position.contains_key(id).then_some((*id, &mut **b))).collect();
        let jobs: Vec<u64> = worlds.iter().map(|(id, _)| caps_by_input[position[id]]).collect();
        let played = scheduler.map_mut(&mut worlds, jobs, |_, (id, branch), cap| play_branch(*id, branch, cap, max_steps, &caps, &config.rule, config.playout_seed))
            .map_err(|_| Error::DeliveryFault("playout_many allocation failed"))?;
        let mut by_input: Vec<Option<PlayoutJob>> = (0..requests.len()).map(|_| None).collect();
        for ((id, _), job) in worlds.iter().zip(played) { by_input[position[id]] = Some(job); }
        drop(worlds);
        Ok(requests.iter().zip(by_input).map(|(r, job)| {
            let branch = &self.branches[&r.branch.0];
            match job.expect("every entry was played") {
                PlayoutJob::Finished { reward, terminated, truncated, end_reason, refusal, steps, summary } => {
                    self.total_steps += steps;
                    BranchPlayoutOutcome::Finished(BranchPlayout { token: RunDecisionToken([r.branch.0, branch.serial]), reward, terminated, truncated, end_reason, refusal, steps, branch_steps: branch.steps, summary })
                }
                PlayoutJob::Fault { error, stage, committed, steps } => {
                    self.total_steps += steps;
                    BranchPlayoutOutcome::Fault { error, stage, committed, steps, branch_steps: branch.steps }
                }
            }
        }).collect())
    }
    /// M41: the action index `rule` picks at the branch's current boundary under `playout_seed` (no effect): the move
    /// `playout_many` would take next. Same branch checks as `step` without the token. The raw candidate is returned; a
    /// hidden-order pick is resolved or refused by the following `step`, as in a playout. An empty legal table is
    /// `Input("branch has no legal action")`.
    pub fn playout_action(&self, b: BranchId, rule: &PlayoutRule, playout_seed: u64) -> Result<RunActionIndex, Error> {
        let branch = self.check_branch(b, None)?;
        playout_pick(&branch.legal, rule, &branch.origin, playout_seed, branch.actions.len() as u64)
            .map(|i| RunActionIndex(i as i64)).ok_or(Error::Input("branch has no legal action"))
    }
    /// M41: the enemy rows of this search's view capacities (compact or dense), as [`Self::action_capacity`].
    pub fn enemy_capacity(&self) -> usize { self.view_capacities().enemies }
    /// M42: the potion rows of this search's view capacities (compact or dense), as [`Self::enemy_capacity`].
    pub fn potion_capacity(&self) -> usize { self.view_capacities().potions }
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
        let (run_estimate, fixed, legal_width, view_id, fingerprint) = if let Some(config) = &self.compact {
            (
                COMPACT_RUN_ESTIMATE,
                fixed_bytes(COMPACT_RUN_ESTIMATE) + std::mem::size_of::<StandardCombatSessionConfig>() as u64 + 16,
                2 * RunCapacities::standard_default().actions,
                crate::STANDARD_COMBAT_COMPACT_VIEW_ID,
                crate::combat_policy::compact_combat_spec(&config.capacities, config.capacity_preset.expect("validated compact preset"))
                    .map(|s| s.fingerprint()).unwrap_or([0; 32]),
            )
        } else {
            (
                BRANCH_RUN_BYTES_ESTIMATE, search_fixed_bytes(), self.root.caps.actions,
                crate::STANDARD_COMBAT_VIEW_ID,
                crate::standard_combat_observation_spec(&self.root.caps).map(|s| s.fingerprint()).unwrap_or([0; 32]),
            )
        };
        let branch_bytes = branch_bytes(&l, legal_width, run_estimate);
        let total = fixed + l.max_live_branches as u64 * branch_bytes + l.max_branches * created_branch_bytes();
        json!({"search_id": PUBLIC_SEARCH_ID, "belief_id": PUBLIC_BELIEF_ID, "knowledge_id": PUBLIC_KNOWLEDGE_ID,
            "combat_view_id": view_id,
            "observation_fingerprint": crate::combat_policy::hex(&fingerprint),
            "root_digest": crate::combat_policy::hex(&self.root.digest), "horizon": "combat",
            "limits": {"max_live_branches": l.max_live_branches, "max_branches": l.max_branches, "max_steps_per_branch": l.max_steps_per_branch, "max_total_steps": l.max_total_steps},
            "hard_caps": {"max_live_branches": SEARCH_HARD_CAPS.max_live_branches, "max_branches": SEARCH_HARD_CAPS.max_branches, "max_steps_per_branch": SEARCH_HARD_CAPS.max_steps_per_branch, "max_total_steps": SEARCH_HARD_CAPS.max_total_steps},
            "max_retained_bytes": total,
            "retained_bytes": {"fixed": fixed, "per_live_branch": branch_bytes,
                "per_created_branch": created_branch_bytes(), "run_estimate_is_measured": true},
            "branch_retained_bytes_bound": branch_bytes,
            "branch_retained_bytes": {"run_estimate": run_estimate, "knowledge_cap": spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP,
                "action_history": 2 * l.max_steps_per_branch * std::mem::size_of::<i64>() as u64, "legal_actions": legal_width * std::mem::size_of::<GameAction>(),
                "branch_entry": branch_record_bytes() + collection_entry_bytes(BOX_SLOT_BYTES)},
            "reward": "combat outcome at the combat's own terminal: victory +1, death -1, escape 0; truncated at max_steps_per_branch (end_reason step_limit)"})
    }
    /// TEST-ONLY: a branch's world (fault injection and P3 comparisons).
    #[cfg(test)]
    pub(crate) fn replay_for_test(&self, b: BranchId) -> Result<Run, Error> { self.replay(&self.branches[&b.0]) }
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
    #[cfg(test)]
    pub(crate) fn widen_legal_table_for_test(&mut self, branch: BranchId, len: usize) {
        let legal = &mut self.branches.get_mut(&branch.0).unwrap().legal;
        legal.resize(len, legal[0].clone());
    }
    /// TEST-ONLY (M41): a branch's legal table (verb parity).
    #[cfg(test)]
    pub(crate) fn legal_for_test(&self, branch: BranchId) -> Vec<GameAction> { self.branches[&branch.0].legal.clone() }
    /// TEST-ONLY (M41): an empty legal table at a live boundary (`NO_LEGAL_ACTION`).
    #[cfg(test)]
    pub(crate) fn clear_legal_table_for_test(&mut self, branch: BranchId) { self.branches.get_mut(&branch.0).unwrap().legal.clear(); }
    #[cfg(test)]
    pub(crate) fn take_run_for_test(&mut self, branch: BranchId) -> Run {
        self.branches.remove(&branch.0).unwrap().run
    }
    #[cfg(test)]
    pub(crate) fn into_root_run_for_test(self) -> Run { self.root.run }

    /// TEST-ONLY: the capacities branches publish with (post-commit publication error, final review M3).
    #[cfg(test)]
    pub(crate) fn root_caps_for_test(&mut self) -> &mut RunCapacities { &mut self.root.caps }
}

#[cfg(test)]
thread_local! {
    /// TEST-ONLY: simulate an exhausted shared session-id allocator (final review v3).
    pub(crate) static FAIL_ID_ALLOCATION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// TEST-ONLY (M41): panic inside the playout step of branch `.0` at accepted-action ordinal `.1` (workers = 1).
    pub(crate) static PANIC_PLAYOUT_AT: std::cell::Cell<Option<(u64, u64)>> = const { std::cell::Cell::new(None) };
    /// TEST-ONLY (M41): fail the 0-step summary read of a playout (workers = 1).
    pub(crate) static FAIL_PLAYOUT_SUMMARY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// TEST-ONLY (M42): fail the post-step summary build for branch `.0` (workers = 1).
    pub(crate) static FAIL_PLAYOUT_SUMMARY_BRANCH: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}
