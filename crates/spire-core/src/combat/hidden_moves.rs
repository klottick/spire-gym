//! M40c (spire-61l stage 3, design-v4 §3–§5): hidden-move records, the public knowledge that lets a Runic Dome combat
//! be a public root. Under hidden intents each enemy's current rolled move is hidden until it executes; a record holds
//! only the public inputs of the roll chain that produced it (pre-roll history in normal form, pre-roll memory in normal
//! form, public roll contexts), never a roll's output. The belief replays the chain through the same selection dispatch
//! (`enemies::act2::select_for`) with the root's or a sample's stream (belief R4). A fail-closed change monitor
//! (`observe`) classifies every change to an enemy's AI state after each dispatched effect; anything unclassified makes
//! the record opaque. Nothing here is read by a rule.
use super::enemies::act2::{install_selection, select_for, EnemyRollContext, RollSite};
use super::enemies::EnemyMemory;
use super::model::{EnemyHandle, State};
use super::*;
use crate::content::{EnemyId, RelicId};
use std::collections::{BTreeMap, BTreeSet};

/// One roll of a chain: its site and public context (living actors' summaries, turn, the player's Constricted flag).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RollContext { pub site: RollSite, pub context: EnemyRollContext, pub constricted: Option<bool> }
/// The roll chain since the enemy's AI state was last public (several rolls only for Writhing Mass's Reactive re-rolls).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Chain { pub history: Vec<FullIntentId>, pub memory: EnemyMemory, pub rolls: Vec<RollContext>, pub thief_branch: bool }
/// The record of one enemy handle (kept for the whole combat, also after death, escape or replacement).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EnemyMoves {
    pub pending: Option<Chain>,
    /// History positions holding a hidden or unidentifiable entry, with its fixed public stand-in.
    pub inert: BTreeMap<usize, FullIntentId>,
    /// Executed entries that could not be identified (same-shape or truncated executions); each is also `inert`.
    pub ambiguous: BTreeSet<usize>,
    /// A card read whether the hidden move of `pending` is an attack.
    pub read: bool,
    /// The enemy's pre-rolled value (Bite, Nip) was shown by damage under hidden intents.
    pub shown: bool,
    /// An unclassified change: refused for the rest of the combat.
    pub opaque: bool,
}
/// Every record plus the transient monitor state. Equality and the root's knowledge bytes cover `enemies` only.
#[derive(Clone, Debug, Default)]
pub(crate) struct HiddenMoves {
    pub enemies: BTreeMap<EnemyHandle, EnemyMoves>,
    /// The enemy whose move is executing and the history position of the executed entry (rule 2d); cleared at the next
    /// `EnemyMove`, `EndRound` or `StartNextPlayer`; a root is refused while it is set.
    pub executing: Option<(EnemyHandle, usize)>,
    /// Rolls recorded during the current dispatch (transient).
    rolled: Vec<EnemyHandle>,
}
impl PartialEq for HiddenMoves { fn eq(&self, other: &Self) -> bool { self.enemies == other.enemies } }
impl Eq for HiddenMoves {}

#[cfg(any(test, feature = "m34-diagnostics"))]
thread_local! {
    /// Diagnostic only: rule 2d (truncated execution) marks by species, for the coverage report.
    pub(crate) static TRUNCATIONS: std::cell::RefCell<BTreeMap<&'static str, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

// ---------------------------------------------------------------- classification tables (exhaustive, no wildcard)

/// Whether a species' rolls are random (else every select is a deterministic function of public history).
pub(crate) fn random_law(id: EnemyId) -> bool {
    use EnemyId::*;
    match id {
        Cultist | SpikeSlimeS | SlimeBoss | Lagavulin | Sentry | Guardian | Hexaghost | SphericGuardian | GremlinMad
        | GremlinSneaky | GremlinFat | GremlinShield | GremlinWizard | Taskmaster | BanditPointy | BanditLeader | BanditBear
        | TorchHead | BronzeAutomaton | Dagger | Transient | Deca | Donu => false,
        JawWorm | GremlinNob | LouseRed | LouseGreen | FungiBeast | AcidSlimeS | AcidSlimeM | AcidSlimeL | SpikeSlimeM
        | SpikeSlimeL | Chosen | Snecko | Centurion | Mystic | GremlinLeader | SlaverBlue | SlaverRed | Collector | SnakePlant
        | ShelledParasite | Byrd | Darkling | OrbWalker | Repulsor | Exploder | Spiker | SpireGrowth | Maw | GiantHead
        | Reptomancer | AwakenedOne | SpireShield | SpireSpear | WrithingMass | Looter | Mugger | BookOfStabbing | Champ
        | BronzeOrb | Nemesis | TimeEater | CorruptHeart => true,
        Unknown(_) => true,
    }
}
/// Whether the roll writes memory that depends on the chosen move (C-MEM): the replay's memory is then the root's.
/// Every other species' live memory is public (the select leaves it unchanged or writes it the same for every outcome).
pub(crate) fn replay_memory(id: EnemyId) -> bool {
    use EnemyId::*;
    match id {
        BookOfStabbing | Champ | BronzeOrb | Nemesis | TimeEater | CorruptHeart => true,
        Cultist | JawWorm | GremlinNob | Lagavulin | Sentry | Guardian | Hexaghost | LouseRed | LouseGreen | AcidSlimeS
        | AcidSlimeM | AcidSlimeL | SpikeSlimeS | SpikeSlimeM | SpikeSlimeL | SlimeBoss | SphericGuardian | Byrd | Chosen
        | Snecko | Centurion | Mystic | GremlinLeader | GremlinMad | GremlinSneaky | GremlinFat | GremlinShield
        | GremlinWizard | SlaverBlue | Taskmaster | SlaverRed | Collector | TorchHead | BronzeAutomaton | OrbWalker
        | Repulsor | Exploder | Spiker | Darkling | SpireGrowth | Transient | Maw | WrithingMass | GiantHead | Reptomancer
        | Dagger | AwakenedOne | Deca | Donu | SpireShield | SpireSpear | FungiBeast | Looter | Mugger | ShelledParasite
        | SnakePlant | BanditPointy | BanditLeader | BanditBear => false,
        Unknown(_) => true,
    }
}
/// Species whose pending roll may be overwritten before it executes (public trigger, deterministic, the overwritten
/// entry never read again; none is a C-MEM species).
pub(crate) fn override_species(id: EnemyId) -> bool {
    use EnemyId::*;
    match id {
        Byrd | ShelledParasite | Darkling | AwakenedOne | AcidSlimeL | SpikeSlimeL | SlimeBoss | Lagavulin | Guardian => true,
        Cultist | JawWorm | GremlinNob | Sentry | Hexaghost | LouseRed | LouseGreen | AcidSlimeS | AcidSlimeM | SpikeSlimeS
        | SpikeSlimeM | SphericGuardian | Chosen | Snecko | Centurion | Mystic | GremlinLeader | GremlinMad | GremlinSneaky
        | GremlinFat | GremlinShield | GremlinWizard | SlaverBlue | Taskmaster | SlaverRed | BookOfStabbing | Champ
        | Collector | TorchHead | BronzeAutomaton | BronzeOrb | OrbWalker | Repulsor | Exploder | Spiker | SpireGrowth
        | Transient | Maw | WrithingMass | GiantHead | Nemesis | Reptomancer | Dagger | TimeEater | Deca | Donu
        | SpireShield | SpireSpear | CorruptHeart | FungiBeast | Looter | Mugger | SnakePlant | BanditPointy
        | BanditLeader | BanditBear | Unknown(_) => false,
    }
}
/// Species that may roll again before executing (Writhing Mass Reactive).
pub(crate) fn chain_species(id: EnemyId) -> bool { matches!(id, EnemyId::WrithingMass) }
/// The pre-rolled move of a species (its damage value is a hidden setup roll).
pub(crate) fn preroll_move(id: EnemyId) -> Option<&'static str> {
    match id { EnemyId::LouseRed | EnemyId::LouseGreen => Some("bite"), EnemyId::Darkling => Some("darkling_nip"), _ => None }
}
/// Same-shape candidate sets (unnamed moves differing only in per-hit damage) and their fixed stand-in.
pub(crate) fn same_shape(id: EnemyId) -> Option<(&'static [&'static str], &'static str)> {
    match id {
        EnemyId::SpireGrowth => Some((&["growth_quick", "growth_smash"], "growth_quick")),
        EnemyId::Maw => Some((&["maw_nom", "maw_slam"], "maw_slam")),
        _ => None,
    }
}
/// How many trailing history entries a species' select can read (census Q1): Maw reads `last` only; every other random
/// species reads at most the last two entries (2 is the conservative bound for all of them).
pub(crate) fn read_window(id: EnemyId) -> usize { if id == EnemyId::Maw { 1 } else if random_law(id) { 2 } else { 0 } }

fn species(e: &model::EnemyState) -> Option<EnemyId> { match e.enemy_id { ActorEnemyId::Full(id) => Some(id), ActorEnemyId::Legacy(_) => None } }
/// Records are maintained only in a combat with public knowledge, the full runtime and hidden intents.
pub(in crate::combat) fn active(state: &State) -> bool { state.knowledge.maintained && state.full_runtime && view::public_rules(state).hide_intents }
/// Awakened One between `AwakenedRebirth` (Active, hp 0) and its RevivalHeal.
fn rebirth(e: &model::EnemyState) -> bool {
    species(e) == Some(EnemyId::AwakenedOne) && e.intent_id.as_str() == "awa_rebirth"
}
/// Final death, escape or replacement (a predicate on current state; Darklings stay HalfDead while downed).
pub(in crate::combat) fn retired(e: &model::EnemyState) -> bool {
    matches!(e.life, ActorLife::Escaped | ActorLife::Replaced) || (e.life == ActorLife::Active && e.hp <= 0 && !rebirth(e))
}
pub(in crate::combat) fn living(e: &model::EnemyState) -> bool { e.life == ActorLife::HalfDead || (e.life == ActorLife::Active && (e.hp > 0 || rebirth(e))) }

// ---------------------------------------------------------------- normal form

/// Memory in normal form: Darkling Nip (owned by the pre-roll candidates) zeroed; a thief's take_turn branch removed
/// (returned as the flag; the replay redraws it).
pub(crate) fn normal_memory(memory: &EnemyMemory) -> (EnemyMemory, bool) {
    let mut m = memory.clone();
    let mut branch = false;
    match &mut m {
        EnemyMemory::Darkling { nip, .. } => *nip = 0,
        EnemyMemory::Thief(t) => { branch = t.pending_next.take().is_some() }
        _ => {}
    }
    (m, branch)
}
fn normal_history(history: &[FullIntentId], inert: &BTreeMap<usize, FullIntentId>) -> Vec<FullIntentId> {
    let mut h = history.to_vec();
    if super::belief::leak(super::belief::Leak::RawChainHistory) { return h }
    for (p, s) in inert { if let Some(x) = h.get_mut(*p) { *x = *s } }
    h
}

// ---------------------------------------------------------------- capture

/// Record a roll (every site: setup at entry, live, summon commit, split commit), before its selection.
#[allow(clippy::too_many_arguments)]
pub(in crate::combat) fn record_roll(state: &mut State, owner: &EnemyHandle, id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory,
    site: RollSite, context: &EnemyRollContext, constricted: Option<bool>) {
    if !active(state) { return }
    let moves = &mut state.knowledge.moves;
    moves.rolled.push(owner.clone());
    let record = moves.enemies.entry(owner.clone()).or_default();
    let roll = RollContext { site, context: context.clone(), constricted };
    match &mut record.pending {
        None => {
            let (memory, thief_branch) = normal_memory(memory);
            record.pending = Some(Chain { history: normal_history(history, &record.inert), memory, rolls: vec![roll], thief_branch });
        }
        Some(chain) if chain_species(id) => { chain.rolls.push(roll); if super::belief::leak(super::belief::Leak::ClearReadOnReroll) { record.read = false } }
        Some(_) => record.opaque = true,
    }
}
/// Test-only negative control `RecordPostRollMemory`: the record keeps the roll's output memory instead of its input.
pub(in crate::combat) fn leak_post_roll_memory(state: &mut State, owner: &EnemyHandle, memory: &EnemyMemory) {
    if !super::belief::leak(super::belief::Leak::RecordPostRollMemory) { return }
    if let Some(chain) = state.knowledge.moves.enemies.get_mut(owner).and_then(|r| r.pending.as_mut()) { chain.memory = normal_memory(memory).0 }
}
/// Record the opening rolls at combat entry (after the item context is installed, so hidden intents are known).
pub(in crate::combat) fn record_openings(state: &mut State, openings: Vec<(EnemyHandle, EnemyId, Option<model::OpeningRoll>)>) {
    if !active(state) { return }
    for (owner, id, opening) in openings {
        match opening {
            Some(o) if o.rolled => record_roll(state, &owner, id, &[], &o.memory, RollSite::Setup, &o.context, o.constricted),
            Some(_) => { state.knowledge.moves.enemies.entry(owner).or_default(); }
            None => {}
        }
    }
    state.knowledge.moves.rolled.clear();
}
/// Record the first roll of a summoned or split child at commit (empty history, fresh memory, context-free).
pub(in crate::combat) fn record_child(state: &mut State, owner: &EnemyHandle, id: EnemyId, site: RollSite) {
    if !active(state) { return }
    let Ok(memory) = EnemyMemory::for_profile(id, state.ascension, super::enemies::EnemyProfile::Standard) else {
        state.knowledge.moves.enemies.entry(owner.clone()).or_default().opaque = true; return
    };
    let context = EnemyRollContext { owner: owner.clone(), turn: state.turn, actors: vec![], ascension: state.ascension };
    record_roll(state, owner, id, &[], &memory, site, &context, None);
    state.knowledge.moves.rolled.retain(|h| h != owner);
}
/// A card play's prepared payload read its target's intent: Fear No Evil requests Calm iff the target intends to attack
/// (`cards/watcher/m25_direct.rs`), so mark it where that payload is committed (hand play and every autoplay/repeat child).
pub(in crate::combat) fn mark_card_read(state: &mut State, card: crate::content::CardId, target: &EnemyHandle) {
    if card == crate::content::CardId::FearNoEvil && !super::belief::leak(super::belief::Leak::SkipMarkFne) { mark_intent_read(state, target) }
}
/// A C-ATK card read the target's intent (Spot Weakness, Go for the Eyes, Fear No Evil).
pub(in crate::combat) fn mark_intent_read(state: &mut State, target: &EnemyHandle) {
    if !active(state) { return }
    let random = state.enemies.iter().find(|e| &e.instance_id == target).and_then(species).is_some_and(random_law);
    if let Some(r) = state.knowledge.moves.enemies.get_mut(target) { if r.pending.is_some() && random { r.read = true } }
}

// ---------------------------------------------------------------- the change monitor

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EnemySnap { life: ActorLife, hp: i32, intent: ActorIntentId, selected: ActorIntentId, history: Vec<ActorIntentId>, memory: Option<EnemyMemory>, stream: Option<u128> }
pub(crate) type Snapshot = Vec<(EnemyHandle, EnemySnap)>;
fn snap(e: &model::EnemyState) -> EnemySnap {
    EnemySnap { life: e.life, hp: e.hp, intent: e.intent_id.clone(), selected: e.selected_move.clone(), history: e.move_history.clone(),
        memory: e.full_memory.clone(), stream: e.full_ai_rng.as_ref().map(|s| s.word_index()) }
}
/// Before a dispatch: every enemy's AI state (only under hidden intents).
pub(in crate::combat) fn snapshot(state: &State) -> Option<Snapshot> {
    if !active(state) { return None }
    Some(state.enemies.iter().map(|e| (e.instance_id.clone(), snap(e))).collect())
}
fn full(id: &ActorIntentId) -> Option<FullIntentId> { match id { ActorIntentId::Full(id) => Some(*id), ActorIntentId::Legacy(_) => None } }
/// Whether an executed same-shape member is ambiguous (design-v4 rule 2c; symmetric across the members of a set).
fn ambiguous_execution(state: &State, e: &model::EnemyState, members: &[&'static str]) -> bool {
    if super::belief::leak(super::belief::Leak::AcceptAmbiguous) { return false }
    let power = |id: &str| state.player_full_powers.iter().any(|p| p.id.0 == id && p.state.amount > 0);
    let relic = |id: RelicId| state.item_context.as_ref().is_some_and(|items| items.owns_relic(id));
    let low = members.iter().any(|m| e.full_damage_entries.get(&FullIntentId::from_name(m)).is_none_or(|d| d.output <= 5));
    power("intangible_player") || power("intangible") || power("buffer") || relic(RelicId::Torii) || relic(RelicId::TungstenRod) || low || state.player.hp <= 0
}
fn stand_in(id: EnemyId, ascension: u8) -> Option<FullIntentId> {
    super::enemies::damage_bases_at_ascension(id, ascension).ok().and_then(|b| b.keys().next().copied())
}
/// After a dispatch: classify every enemy's change (design-v4 §3.3; branches tried in order, exactly one applies).
pub(in crate::combat) fn observe(state: &mut State, effect: &Effect, before: Snapshot) {
    if super::belief::leak(super::belief::Leak::NoMonitor) { state.knowledge.moves.rolled.clear(); return }
    let rolled = std::mem::take(&mut state.knowledge.moves.rolled);
    if matches!(effect.kind, EffectKind::EnemyMove | EffectKind::EndRound | EffectKind::StartNextPlayer) { state.knowledge.moves.executing = None; }
    let ascension = state.ascension;
    for (handle, old) in before {
        let Some(e) = state.enemies.iter().find(|e| e.instance_id == handle) else { continue };
        let Some(id) = species(e) else { continue };
        let now = snap(e);
        // An actor's queued move executes only if it is living when the dispatch starts (`turns.rs` EnemyMove skips a dead
        // actor, e.g. one Poison killed at the start of the enemy turn); a skipped move stays hidden (retirement path).
        let was_living = old.life == ActorLife::HalfDead || (old.life == ActorLife::Active && old.hp > 0);
        let executing_here = matches!((&effect.kind, &effect.target), (EffectKind::EnemyMove, Some(Actor::Enemy(t))) if *t == handle)
            && (was_living || super::belief::leak(super::belief::Leak::DeadMoveExecutes));
        let truncated = state.knowledge.moves.executing.as_ref().is_some_and(|(h, _)| *h == handle) && old.hp > 0 && now.hp <= 0;
        let (shape, members_ambiguous) = match same_shape(id) { Some((m, s)) => (Some((m, s)), ambiguous_execution(state, e, m)), None => (None, false) };
        if !state.knowledge.moves.enemies.contains_key(&handle) { continue }
        // Rule 2d: the acting enemy reached hp 0 during its own move (edge-triggered; public in every world).
        if truncated && random_law(id) && !super::belief::leak(super::belief::Leak::IgnoreTruncation) {
            let p = state.knowledge.moves.executing.as_ref().map(|(_, p)| *p).unwrap();
            let record = state.knowledge.moves.enemies.get_mut(&handle).unwrap();
            match stand_in(id, ascension) { Some(s) => { record.inert.insert(p, s); record.ambiguous.insert(p); } None => record.opaque = true }
            if preroll_move(id).is_some() { record.shown = true; }
            #[cfg(any(test, feature = "m34-diagnostics"))]
            TRUNCATIONS.with(|t| *t.borrow_mut().entry(id.as_str()).or_default() += 1);
        }
        let record = state.knowledge.moves.enemies.get_mut(&handle).unwrap();
        if old == now && !executing_here { continue }
        if rolled.contains(&handle) {
            // Negative control (the v2 defect): the stand-in taken from the entry pushed after the ambiguous one.
            if super::belief::leak(super::belief::Leak::AmbiguityStandInFromLater) {
                let positions = record.ambiguous.iter().copied().collect::<Vec<_>>();
                for p in positions { if p + 2 == now.history.len() { if let Some(x) = full(&now.history[p + 1]) { record.inert.insert(p, x); } } }
            }
            continue
        }
        if executing_here {
            let pos = old.history.len().saturating_sub(1);
            let executed = full(&old.intent);
            if let Some(chain) = record.pending.take() {
                if chain.rolls.len() > 1 { if let Some(x) = executed { for p in chain.history.len()..pos { record.inert.insert(p, x); } } }
            }
            record.read = false;
            if executed.is_some_and(|x| preroll_move(id) == Some(x.as_str())) && !super::belief::leak(super::belief::Leak::SkipPrerollShown) { record.shown = true; }
            if let (Some((members, stand)), Some(x)) = (shape, executed) {
                if members.contains(&x.as_str()) && members_ambiguous { record.inert.insert(pos, FullIntentId::from_name(stand)); record.ambiguous.insert(pos); }
            }
            state.knowledge.moves.executing = Some((handle.clone(), pos));
            continue
        }
        // Negative control (the v2 defect): retirement at hp 0 during the rebirth window sticks, so phase 2 is lost.
        if super::belief::leak(super::belief::Leak::StickyRetire) && id == EnemyId::AwakenedOne && e.life == ActorLife::Active && e.hp <= 0 { record.opaque = true; }
        if retired(e) {
            if old.memory != now.memory && replay_memory(id) { record.opaque = true; }
            continue
        }
        let Some(chain) = record.pending.as_ref() else { continue };
        if old.history != now.history {
            if override_species(id) {
                let start = chain.history.len();
                let end = start + chain.rolls.len();
                if let Some(over) = now.history.last().and_then(full) { for p in start..end { record.inert.insert(p, over); } } else { record.opaque = true }
                record.pending = None;
                record.read = false;
            } else { record.opaque = true }
            continue
        }
        if (old.memory != now.memory && replay_memory(id)) || old.selected != now.selected || old.stream != now.stream { record.opaque = true; }
    }
}

// ---------------------------------------------------------------- root checks and replay

/// Design-v4 §5: the hidden-intent root checks (a class of reason, never a value).
pub(in crate::combat) fn support(state: &State) -> Result<(), super::belief::Unsupported> {
    use super::belief::Unsupported;
    let moves = &state.knowledge.moves;
    if moves.executing.is_some() { return Err(Unsupported::NotAtDecision) }
    let mut out = Ok(());
    let mut worst = |u: Unsupported| { if out.is_ok() || u == Unsupported::HiddenIntents { out = Err(u) } };
    for e in &state.enemies {
        let record = moves.enemies.get(&e.instance_id);
        if record.is_some_and(|r| r.opaque) { worst(Unsupported::HiddenIntents); continue }
        // Every actor, living or not, needs a record: the replay canonicalises dead actors from theirs.
        let (Some(id), Some(r)) = (species(e), record) else { worst(Unsupported::HiddenIntents); continue };
        if !living(e) { continue }
        if r.read { worst(Unsupported::HiddenIntentRead) }
        if r.shown { worst(Unsupported::HiddenPrerollShown) }
        if let Some(chain) = &r.pending {
            let w = read_window(id);
            let len = chain.history.len();
            let early = super::belief::leak(super::belief::Leak::EarlyAmbiguityClear);
            if r.ambiguous.iter().any(|p| w > 0 && *p + w >= len + usize::from(early) && *p < len) { worst(Unsupported::HiddenMoveAmbiguous) }
        }
    }
    out
}

/// Belief R4 (design-v4 §4): replace every hidden-roll-dependent field of every recorded enemy (retired ones included)
/// by the replay of its public record through the shared selection with the world's (already re-derived) actor stream.
pub(in crate::combat) fn replay(state: &mut State) -> Result<(), super::belief::Unsupported> {
    use super::belief::{leak, Leak, Unsupported};
    if !active(state) { return Ok(()) }
    if leak(Leak::KeepHiddenMove) { return Ok(()) }
    let records = state.knowledge.moves.enemies.clone();
    for (handle, r) in records {
        let Some(index) = state.enemies.iter().position(|e| e.instance_id == handle) else { continue };
        let Some(id) = species(&state.enemies[index]) else { return Err(Unsupported::HiddenIntents) };
        if retired(&state.enemies[index]) && leak(Leak::KeepRetiredResidue) { continue }
        let nip = state.enemies[index].full_damage_entries.get(&FullIntentId::from_name("darkling_nip")).map(|d| d.base);
        let restore_nip = |m: &mut EnemyMemory| if let (EnemyMemory::Darkling { nip: n, .. }, Some(v)) = (m, nip) { *n = v };
        let live_memory = state.enemies[index].full_memory.clone().map(|m| if leak(Leak::LiveMemoryKeepsNip) { m } else { normal_memory(&m).0 });
        let restore_nip = |m: &mut EnemyMemory| if !leak(Leak::LiveMemoryKeepsNip) { restore_nip(m) };
        if let Some(chain) = &r.pending {
            let mut memory = chain.memory.clone();
            restore_nip(&mut memory);
            let mut stream = state.enemies[index].full_ai_rng.clone().ok_or(Unsupported::HiddenIntents)?;
            if leak(Leak::ReplayLiveStream) { /* test-only: the stream the live world left (no re-derivation reset) */ }
            if chain.thief_branch {
                let (smoke, next) = rng::bernoulli(&stream, 1, 2).map_err(|_| Unsupported::HiddenIntents)?;
                if let EnemyMemory::Thief(t) = &mut memory {
                    t.pending_next = Some(FullIntentId::from_name(super::enemies::act1::thieves::intent_name(id, if smoke { "smoke_bomb" } else { "lunge" })));
                }
                stream = next;
            }
            let mut history = chain.history.clone();
            state.enemies[index].move_history = history.iter().map(|h| ActorIntentId::Full(*h)).collect();
            for roll in &chain.rolls {
                let selected = select_for(roll.site, id, &history, &memory, &stream, &roll.context, roll.constricted).map_err(|_| Unsupported::HiddenIntents)?;
                history.push(selected.intent);
                memory = selected.memory.clone();
                stream = selected.stream.clone();
                install_selection(state, &handle, id, selected);
            }
            let e = &mut state.enemies[index];
            if !replay_memory(id) { if let Some(mut m) = live_memory.clone() { restore_nip(&mut m); e.full_memory = Some(m) } }
        } else if let Some(mut m) = live_memory {
            restore_nip(&mut m);
            state.enemies[index].full_memory = Some(m);
        }
        let e = &mut state.enemies[index];
        let last = e.move_history.len().checked_sub(1);
        for (p, s) in &r.inert {
            if let Some(x) = e.move_history.get_mut(*p) { *x = ActorIntentId::Full(*s) }
            if Some(*p) == last { e.intent_id = ActorIntentId::Full(*s); e.selected_move = ActorIntentId::Full(*s); }
        }
    }
    Ok(())
}
/// The records in normal form, as root digest input (only under hidden intents; empty otherwise).
pub(in crate::combat) fn knowledge_bytes(state: &State) -> Option<String> {
    if !active(state) || super::belief::leak(super::belief::Leak::DigestWithoutMoves) { return None }
    Some(format!("{:?}", state.knowledge.moves.enemies))
}

/// Fuzz oracle only: move each inert (never-read) history entry to another move of the species' vocabulary. Never the
/// last entry of an actor without a pending roll (its executed move is public).
#[cfg(any(test, feature = "fuzz-diagnostics"))]
pub(in crate::combat) fn fuzz_shift_inert(state: &mut State, salt: u8) {
    let records = state.knowledge.moves.enemies.clone();
    let ascension = state.ascension;
    for e in &mut state.enemies {
        let (Some(r), Some(id)) = (records.get(&e.instance_id), species(e)) else { continue };
        let Ok(bases) = super::enemies::damage_bases_at_ascension(id, ascension) else { continue };
        let moves = bases.keys().copied().collect::<Vec<_>>();
        if moves.is_empty() { continue }
        let last = e.move_history.len().saturating_sub(1);
        for p in r.inert.keys() {
            if *p >= last { continue }
            if let Some(x) = e.move_history.get_mut(*p) { *x = ActorIntentId::Full(moves[(*p + usize::from(salt)) % moves.len()]) }
        }
    }
}
