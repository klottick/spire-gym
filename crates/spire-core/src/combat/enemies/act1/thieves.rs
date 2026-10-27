//! A0 Looter/Mugger turn production and theft lifecycle on the shared resolver.
use super::super::{Actor, ActorEnemyId, ActorIntentId, DamagePayload, Effect, EffectKind,
    EnemyMemory, EnemyState, FullIntentId, InputError, Source};
use super::super::super::model::{ActorLife, EnemyHandle, State};
use super::super::rng::{self, StreamState};
use crate::content::EnemyId;

const THIEVERY_GOLD_A0: u32 = 15;

const LOOTER_MUG: &str = "looter_mug";
const LOOTER_LUNGE: &str = "looter_lunge";
const LOOTER_SMOKE: &str = "looter_smoke_bomb";
const LOOTER_ESCAPE: &str = "looter_escape";
const MUGGER_MUG: &str = "mugger_mug";
const MUGGER_LUNGE: &str = "mugger_lunge";
const MUGGER_SMOKE: &str = "mugger_smoke_bomb";
const MUGGER_ESCAPE: &str = "mugger_escape";

/// Per-actor theft accounting and the outstanding ledger-to-debit checkpoint.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ThiefMemory {
    /// Source-owned ledger; the later actual gold debit does not rewrite it.
    pub stolen_gold: u32,
    /// The ledger action committed and the matching DamageAction debit is queued.
    pub pending_theft: Option<FullIntentId>,
    /// The fair post-Mug branch was rolled in takeTurn and awaits RollMove.
    pub pending_next: Option<FullIntentId>,
    /// Guards the one STOLEN_GOLD receipt earned by an actual death.
    pub reward_recorded: bool,
}

impl ThiefMemory {
    pub fn valid_for(&self, id: EnemyId) -> bool {
        matches!(id, EnemyId::Looter | EnemyId::Mugger)
            && self.pending_theft.is_none_or(|intent| is_attack_intent(id, intent))
            && self.pending_next.is_none_or(|intent|
                intent.as_str() == intent_name(id, "lunge")
                    || intent.as_str() == intent_name(id, "smoke_bomb"))
            && (!self.reward_recorded || (self.stolen_gold > 0 && self.pending_theft.is_none()
                && self.pending_next.is_none()))
    }
}

/// Active combat currency owner plus receipts earned by killed thieves. Run's
/// cached currency is synchronized from this state after attempted actions.
/// This exists only for fights containing a Looter or Mugger.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ThiefGoldState {
    pub initial_gold: u32,
    pub current_gold: u32,
    pub earned_rewards: Vec<StolenGoldReceipt>,
    /// True after Run entry attaches its currency or the first thief effect runs.
    pub initialized: bool,
    pub mugged: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StolenGoldReceipt {
    pub owner: EnemyHandle,
    pub amount: u32,
}

/// Species-owned queued work. Each variant runs through the ordinary shared
/// effect dispatcher and retains its source/target in the resolver fault frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThiefEffect {
    /// First DamageAction checkpoint: accrue min(cap, current gold) to this actor.
    Ledger { owner: EnemyHandle, cap: u32, intent: FullIntentId },
    /// Second checkpoint: debit current gold, then queue ordinary shared HP damage.
    TheftDamage { owner: EnemyHandle, cap: u32, intent: FullIntentId },
    /// Set the room's mugged flag and use the common escaped-actor settlement path.
    Escape { owner: EnemyHandle },
    /// Earn the actor's recorded ledger as a claimable receipt without refunding gold.
    DeathReward { owner: EnemyHandle },
}

fn is_thief_id(id: EnemyId) -> bool {
    matches!(id, EnemyId::Looter | EnemyId::Mugger)
}

fn intent_name(id: EnemyId, suffix: &'static str) -> &'static str {
    match (id, suffix) {
        (EnemyId::Looter, "mug") => LOOTER_MUG,
        (EnemyId::Looter, "lunge") => LOOTER_LUNGE,
        (EnemyId::Looter, "smoke_bomb") => LOOTER_SMOKE,
        (EnemyId::Looter, "escape") => LOOTER_ESCAPE,
        (EnemyId::Mugger, "mug") => MUGGER_MUG,
        (EnemyId::Mugger, "lunge") => MUGGER_LUNGE,
        (EnemyId::Mugger, "smoke_bomb") => MUGGER_SMOKE,
        (EnemyId::Mugger, "escape") => MUGGER_ESCAPE,
        _ => unreachable!("supported thief move suffix"),
    }
}

fn is_attack_intent(id: EnemyId, intent: FullIntentId) -> bool {
    intent.as_str() == intent_name(id, "mug") || intent.as_str() == intent_name(id, "lunge")
}

fn is_valid_intent(id: EnemyId, intent: FullIntentId) -> bool {
    ["mug", "lunge", "smoke_bomb", "escape"]
        .into_iter().any(|suffix| intent.as_str() == intent_name(id, suffix))
}

fn make_effect(source: Actor, target: Actor, kind: EffectKind) -> Effect {
    Effect { source: Source::Actor(source), target: Some(target), kind }
}

fn thief_owner<'a>(state: &'a State, owner: &EnemyHandle) -> Result<&'a EnemyState, InputError> {
    let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("thief effect owner is not retained".into()))?;
    let ActorEnemyId::Full(id) = &enemy.enemy_id else {
        return Err(InputError::Invariant("thief owner has a legacy identity".into()));
    };
    let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else {
        return Err(InputError::Invariant("thief actor memory is absent".into()));
    };
    if !is_thief_id(*id) || !memory.valid_for(*id) {
        return Err(InputError::Invariant("thief owner/memory identity mismatch".into()));
    }
    Ok(enemy)
}

fn require_active_thief<'a>(state: &'a State, owner: &EnemyHandle) -> Result<&'a EnemyState, InputError> {
    let enemy = thief_owner(state, owner)?;
    if enemy.life != ActorLife::Active || enemy.hp <= 0 {
        return Err(InputError::Invariant("thief effect requires a living active actor".into()));
    }
    Ok(enemy)
}

fn current_intent(enemy: &EnemyState) -> Result<FullIntentId, InputError> {
    match &enemy.intent_id {
        ActorIntentId::Full(intent) => Ok(*intent),
        ActorIntentId::Legacy(_) => Err(InputError::Invariant("thief actor has a legacy move".into())),
    }
}

fn expected_actor_source(effect: &Effect, owner: &EnemyHandle, self_target: bool) -> bool {
    let source_matches = effect.source == Source::Actor(Actor::Enemy(owner.clone()));
    let target_matches = if self_target {
        effect.target == Some(Actor::Enemy(owner.clone()))
    } else {
        effect.target == Some(Actor::Player)
    };
    source_matches && target_matches
}

/// Check all identity, lifecycle, intent, cap and custody boundaries before a
/// ThiefEffect can change the shared State.
pub(in crate::combat) fn validate_effect(
    state: &State,
    effect: &Effect,
    thief_effect: &ThiefEffect,
) -> Result<(), InputError> {
    if !state.full_runtime {
        return Err(InputError::Unsupported("thief effects require native full combat".into()));
    }
    let gold = state.thief_gold.as_ref()
        .ok_or_else(|| InputError::Invariant("thief gold custody is absent".into()))?;
    match thief_effect {
        ThiefEffect::Ledger { owner, cap, intent } => {
            if *cap != THIEVERY_GOLD_A0 || !expected_actor_source(effect, owner, false) {
                return Err(InputError::Invariant("thief ledger source, target or A0 cap mismatch".into()));
            }
            let enemy = require_active_thief(state, owner)?;
            let id = match &enemy.enemy_id { ActorEnemyId::Full(id) => *id, _ => unreachable!() };
            if !is_attack_intent(id, *intent) || current_intent(enemy)? != *intent
                || enemy.selected_move != ActorIntentId::Full(*intent) {
                return Err(InputError::Invariant("thief ledger move does not match its actor".into()));
            }
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
            if memory.pending_theft.is_some() || memory.reward_recorded {
                return Err(InputError::Invariant("thief ledger overlaps an earlier checkpoint".into()));
            }
            let _ = gold;
        }
        ThiefEffect::TheftDamage { owner, cap, intent } => {
            if *cap != THIEVERY_GOLD_A0 || !expected_actor_source(effect, owner, false) {
                return Err(InputError::Invariant("thief debit source, target or A0 cap mismatch".into()));
            }
            let enemy = require_active_thief(state, owner)?;
            let id = match &enemy.enemy_id { ActorEnemyId::Full(id) => *id, _ => unreachable!() };
            if !is_attack_intent(id, *intent) || current_intent(enemy)? != *intent
                || enemy.selected_move != ActorIntentId::Full(*intent) {
                return Err(InputError::Invariant("thief debit move does not match its actor".into()));
            }
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
            if memory.pending_theft != Some(*intent) || memory.reward_recorded {
                return Err(InputError::Invariant("thief debit has no matching committed ledger".into()));
            }
            let _ = gold;
        }
        ThiefEffect::Escape { owner } => {
            if !expected_actor_source(effect, owner, true) {
                return Err(InputError::Invariant("thief escape source/target mismatch".into()));
            }
            let enemy = require_active_thief(state, owner)?;
            let id = match &enemy.enemy_id { ActorEnemyId::Full(id) => *id, _ => unreachable!() };
            if current_intent(enemy)?.as_str() != intent_name(id, "escape") {
                return Err(InputError::Invariant("thief escape requires its Escape intent".into()));
            }
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
            if memory.pending_theft.is_some() || memory.pending_next.is_some() {
                return Err(InputError::Invariant("thief cannot escape during an unresolved theft".into()));
            }
            let _ = gold;
        }
        ThiefEffect::DeathReward { owner } => {
            if effect.source != Source::None || effect.target != Some(Actor::Enemy(owner.clone())) {
                return Err(InputError::Invariant("stolen-gold reward must target its own death hook".into()));
            }
            let enemy = thief_owner(state, owner)?;
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
            if enemy.life != ActorLife::Active || enemy.hp != 0 || memory.stolen_gold == 0
                || memory.pending_theft.is_some() || memory.reward_recorded
                || gold.earned_rewards.iter().any(|receipt| &receipt.owner == owner) {
                return Err(InputError::Invariant("stolen-gold receipt is not backed by a fresh thief death".into()));
            }
        }
    }
    Ok(())
}

/// Apply one species-owned checkpoint. Theft remains independent of Block and
/// normal HP damage remains the shared resolver's next effect.
pub(in crate::combat) fn apply_effect(
    state: &mut State,
    effect: &Effect,
    thief_effect: &ThiefEffect,
) -> Result<(), super::super::super::EffectError> {
    validate_effect(state, effect, thief_effect)?;
    match thief_effect {
        ThiefEffect::Ledger { owner, cap, intent } => {
            let gold = state.thief_gold.as_ref().expect("validated thief gold state").current_gold;
            let recorded = gold.min(*cap);
            let current = thief_owner(state, owner)?;
            let Some(EnemyMemory::Thief(memory)) = current.full_memory.as_ref() else { unreachable!() };
            let next_ledger = memory.stolen_gold.checked_add(recorded)
                .ok_or_else(|| InputError::Unsupported("thief ledger overflow".into()))?;
            let enemy = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner)
                .expect("validated thief owner");
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_mut() else { unreachable!() };
            memory.stolen_gold = next_ledger;
            memory.pending_theft = Some(*intent);
            state.thief_gold.as_mut().expect("validated thief gold state").initialized = true;
        }
        ThiefEffect::TheftDamage { owner, cap, intent } => {
            let actor = Actor::Enemy(owner.clone());
            let damage = make_effect(actor.clone(), Actor::Player,
                EffectKind::Damage(DamagePayload::FullEnemyMove(*intent)));
            // Reject malformed child damage before committing the independent currency debit.
            super::super::super::engine::validate_effect(&damage, state)?;
            let current_gold = state.thief_gold.as_ref().expect("validated thief gold state").current_gold;
            let debit = current_gold.min(*cap);
            let remaining = current_gold.checked_sub(debit)
                .ok_or_else(|| InputError::Invariant("thief current gold underflow".into()))?;
            state.thief_gold.as_mut().expect("validated thief gold state").current_gold = remaining;
            state.thief_gold.as_mut().expect("validated thief gold state").initialized = true;
            let enemy = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner)
                .expect("validated thief owner");
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_mut() else { unreachable!() };
            memory.pending_theft = None;
            state.pending.push_front(damage);
        }
        ThiefEffect::Escape { owner } => {
            {
                let gold = state.thief_gold.as_mut().expect("validated thief gold state");
                gold.initialized = true;
                gold.mugged = true;
            }
            // This shared helper sets Escaped, and settles only if all retained
            // actors have actually left or died; escape itself never emits Death.
            super::super::act2::gremlin_leader::escape(state, owner);
        }
        ThiefEffect::DeathReward { owner } => {
            let enemy = thief_owner(state, owner)?;
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
            let amount = memory.stolen_gold;
            state.thief_gold.as_mut().expect("validated thief gold state").earned_rewards
                .try_reserve(1).map_err(|_| InputError::Unsupported("stolen-gold receipt allocation failed".into()))?;
            state.thief_gold.as_mut().expect("validated thief gold state").earned_rewards
                .push(StolenGoldReceipt { owner: owner.clone(), amount });
            state.thief_gold.as_mut().expect("validated thief gold state").initialized = true;
            let enemy = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner)
                .expect("validated thief owner");
            let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_mut() else { unreachable!() };
            memory.pending_next = None;
            memory.reward_recorded = true;
        }
    }
    Ok(())
}

/// Attach the Run's initial currency once, before the first player or actor action.
pub(in crate::combat) fn initialize_gold(state: &mut State, gold: u32) -> Result<(), InputError> {
    if !state.full_runtime || !state.enemies.iter().any(|enemy|
        matches!(&enemy.enemy_id, ActorEnemyId::Full(id) if is_thief_id(*id))) {
        return Err(InputError::Unsupported("initial thief gold requires a native thief encounter".into()));
    }
    let current = state.thief_gold.as_ref()
        .ok_or_else(|| InputError::Invariant("thief gold entry state is absent".into()))?;
    if current.initialized || current.initial_gold != 0 || current.current_gold != 0
        || !current.earned_rewards.is_empty() || current.mugged
        || state.attempted_action.is_some() || state.current.is_some() || state.turn != 1
        || state.enemies.iter().any(|enemy| enemy.move_history.len() != 1 || enemy.life != ActorLife::Active)
        || state.enemies.iter().filter(|enemy| matches!(&enemy.enemy_id, ActorEnemyId::Full(EnemyId::Looter | EnemyId::Mugger))).any(|enemy| {
            !matches!(enemy.full_memory.as_ref(), Some(EnemyMemory::Thief(memory))
                if memory.stolen_gold == 0 && memory.pending_theft.is_none()
                    && memory.pending_next.is_none() && !memory.reward_recorded)
        }) {
        return Err(InputError::Invariant("initial thief gold can only attach before combat actions".into()));
    }
    let current = state.thief_gold.as_mut().expect("checked thief gold entry state");
    current.initial_gold = gold;
    current.current_gold = gold;
    current.initialized = true;
    Ok(())
}

pub(in crate::combat) fn current_gold(state: &State) -> Option<u32> {
    state.thief_gold.as_ref().map(|gold| gold.current_gold)
}

pub(in crate::combat) fn stolen_gold_rewards(state: &State) -> Vec<StolenGoldReceipt> {
    state.thief_gold.as_ref().map(|gold| gold.earned_rewards.clone()).unwrap_or_default()
}

pub(in crate::combat) fn mugged(state: &State) -> bool {
    state.thief_gold.as_ref().is_some_and(|gold| gold.mugged)
}

pub(in crate::combat) fn all_escaped(state: &State) -> bool {
    state.thief_gold.is_some() && !state.enemies.is_empty()
        && state.enemies.iter().all(|enemy| enemy.life == ActorLife::Escaped)
}

/// Validate the optional currency custody and receipt ledger without conflating
/// a killed thief, an escaped thief, or an ordinary non-thief battle.
pub(in crate::combat) fn validate_state(state: &State) -> Result<(), InputError> {
    let thieves = state.enemies.iter().filter(|enemy|
        matches!(&enemy.enemy_id, ActorEnemyId::Full(EnemyId::Looter | EnemyId::Mugger))).collect::<Vec<_>>();
    let Some(gold) = state.thief_gold.as_ref() else {
        return if thieves.is_empty() { Ok(()) }
            else { Err(InputError::Invariant("thief encounter has no gold custody state".into())) };
    };
    if thieves.is_empty() {
        return Err(InputError::Invariant("non-thief combat has thief gold custody".into()));
    }
    for enemy in &thieves {
        let ActorEnemyId::Full(id) = &enemy.enemy_id else { unreachable!() };
        let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else {
            return Err(InputError::Invariant("thief encounter memory is absent".into()));
        };
        if !memory.valid_for(*id) {
            return Err(InputError::Invariant("thief encounter memory is invalid".into()));
        }
        let owned_move = |move_id: &ActorIntentId| matches!(move_id,
            ActorIntentId::Full(intent) if is_valid_intent(*id, *intent));
        if !is_valid_intent(*id, current_intent(enemy)?)
            || !owned_move(&enemy.selected_move)
            || enemy.move_history.iter().any(|move_id| !owned_move(move_id)) {
            return Err(InputError::Invariant("thief current, selected, or historical move is not source-owned".into()));
        }
    }
    if !gold.initialized && (gold.initial_gold != 0 || gold.current_gold != 0
        || !gold.earned_rewards.is_empty() || gold.mugged
        || thieves.iter().any(|enemy| !matches!(enemy.full_memory.as_ref(),
            Some(EnemyMemory::Thief(memory)) if memory.stolen_gold == 0
                && memory.pending_theft.is_none() && memory.pending_next.is_none()
                && !memory.reward_recorded))) {
        return Err(InputError::Invariant("unattached thief currency contains committed work".into()));
    }
    if gold.mugged != thieves.iter().any(|enemy| enemy.life == ActorLife::Escaped) {
        return Err(InputError::Invariant("thief mugged flag and escape lifecycle disagree".into()));
    }
    let mut receipt_owners = std::collections::BTreeSet::new();
    for receipt in &gold.earned_rewards {
        if receipt.amount == 0 || !receipt_owners.insert(receipt.owner.clone()) {
            return Err(InputError::Invariant("duplicate or empty stolen-gold receipt".into()));
        }
        let enemy = thieves.iter().find(|enemy| enemy.instance_id == receipt.owner)
            .ok_or_else(|| InputError::Invariant("stolen-gold receipt owner is absent".into()))?;
        let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
        if enemy.hp != 0 || enemy.life != ActorLife::Active || memory.stolen_gold != receipt.amount
            || !memory.reward_recorded {
            return Err(InputError::Invariant("stolen-gold receipt does not match a killed actor ledger".into()));
        }
    }
    for enemy in &thieves {
        let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { unreachable!() };
        let has_receipt = receipt_owners.contains(&enemy.instance_id);
        if memory.reward_recorded != has_receipt {
            return Err(InputError::Invariant("thief reward latch and receipt diverge".into()));
        }
        if let Some(intent) = memory.pending_theft {
            let owner = &enemy.instance_id;
            let effect_matches = |effect: &Effect| matches!(&effect.kind,
                EffectKind::Thief(ThiefEffect::TheftDamage { owner: effect_owner, cap: THIEVERY_GOLD_A0, intent: effect_intent })
                    if effect_owner == owner && *effect_intent == intent);
            if state.fault.is_none()
                && !state.current.as_ref().is_some_and(effect_matches)
                && !state.pending.iter().any(effect_matches) {
                return Err(InputError::Invariant("thief ledger has no queued matching debit checkpoint".into()));
            }
        }
        if let Some(next) = memory.pending_next {
            let ActorEnemyId::Full(id) = &enemy.enemy_id else {
                return Err(InputError::Invariant("thief branch latch has a legacy identity".into()));
            };
            let mug = FullIntentId::from_name(intent_name(*id, "mug"));
            let branches = [
                FullIntentId::from_name(intent_name(*id, "lunge")),
                FullIntentId::from_name(intent_name(*id, "smoke_bomb")),
            ];
            if current_intent(enemy)? != mug || enemy.selected_move != ActorIntentId::Full(mug) {
                return Err(InputError::Invariant("thief branch latch outlived the second Mug move".into()));
            }
            let history = enemy.move_history.iter().map(|intent| match intent {
                ActorIntentId::Full(intent) => Ok(*intent),
                ActorIntentId::Legacy(_) => Err(InputError::Invariant("mixed thief move history".into())),
            }).collect::<Result<Vec<_>, _>>()?;
            if !branches.contains(&next)
                || history.as_slice() != [mug, mug] {
                return Err(InputError::Invariant("thief next-move latch is not owned by the two-Mug branch".into()));
            }
        }
    }
    Ok(())
}

/// Select the finite two-Mug branch already rolled by takeTurn. Lunge/Smoke/
/// Escape are deterministic after that source-owned choice.
pub(in crate::combat) fn select_move(
    id: EnemyId,
    history: &[FullIntentId],
    memory: &EnemyMemory,
    stream: &StreamState,
) -> Result<super::super::MoveSelection, super::super::SelectError> {
    if !is_thief_id(id) || !matches!(memory, EnemyMemory::Thief(thief) if thief.valid_for(id)) {
        return Err(InputError::Invariant("thief selector identity/memory mismatch".into()).into());
    }
    if stream.id().domain() != rng::Domain::RunCombatEnemyAi {
        return Err(InputError::Unsupported("thief requires actor-owned AI stream".into()).into());
    }
    let mug = FullIntentId::from_name(intent_name(id, "mug"));
    let lunge = FullIntentId::from_name(intent_name(id, "lunge"));
    let smoke = FullIntentId::from_name(intent_name(id, "smoke_bomb"));
    let escape = FullIntentId::from_name(intent_name(id, "escape"));
    if history.iter().any(|intent| ![mug, lunge, smoke, escape].contains(intent)) {
        return Err(InputError::Unsupported("thief move history/species mismatch".into()).into());
    }
    let (intent, next, next_memory) = match history {
        [] => (mug, stream.clone(), memory.clone()),
        [previous] if *previous == mug => (mug, stream.clone(), memory.clone()),
        [first, second] if *first == mug && *second == mug => {
            let EnemyMemory::Thief(thief_memory) = memory else { unreachable!() };
            let branch = thief_memory.pending_next.ok_or_else(||
                InputError::Invariant("thief second-Mug branch was not committed in takeTurn".into()))?;
            if branch != lunge && branch != smoke {
                return Err(InputError::Invariant("thief pending branch is not Lunge or Smoke".into()).into());
            }
            let mut next_memory = thief_memory.clone();
            next_memory.pending_next = None;
            (branch, stream.clone(), EnemyMemory::Thief(next_memory))
        }
        [first, second, third] if *first == mug && *second == mug && *third == lunge =>
            (smoke, stream.clone(), memory.clone()),
        [first, second, third] if *first == mug && *second == mug && *third == smoke =>
            (escape, stream.clone(), memory.clone()),
        [first, second, third, fourth] if *first == mug && *second == mug && *third == lunge && *fourth == smoke =>
            (escape, stream.clone(), memory.clone()),
        _ => return Err(InputError::Unsupported("thief history has no authored continuation".into()).into()),
    };
    Ok(super::super::MoveSelection { intent, stream: next, memory: next_memory })
}

/// Produce one turn's work; the resolver performs ledger, debit and HP commits
/// as distinct queued effects. The actor's history and selection stay shared.
pub(in crate::combat) fn take_turn(
    state: &mut State,
    owner: &EnemyHandle,
) -> Result<Vec<Effect>, super::super::super::EffectError> {
    if !state.full_runtime {
        return Err(InputError::Unsupported("thief turn requires native full combat".into()).into());
    }
    let enemy = require_active_thief(state, owner)?;
    let id = match &enemy.enemy_id { ActorEnemyId::Full(id) => *id, _ => unreachable!() };
    let intent = current_intent(enemy)?;
    if !is_valid_intent(id, intent) || enemy.selected_move != ActorIntentId::Full(intent) {
        return Err(InputError::Invariant("thief turn intent/memory mismatch".into()).into());
    }
    let move_history = enemy.move_history.iter().map(|intent| match intent {
        ActorIntentId::Full(intent) => Ok(*intent),
        ActorIntentId::Legacy(_) => Err(InputError::Invariant("mixed thief move history".into())),
    }).collect::<Result<Vec<_>, _>>()?;
    let mug = FullIntentId::from_name(intent_name(id, "mug"));
    let lunge = FullIntentId::from_name(intent_name(id, "lunge"));
    let smoke = FullIntentId::from_name(intent_name(id, "smoke_bomb"));
    let memory = match enemy.full_memory.as_ref() {
        Some(EnemyMemory::Thief(memory)) => memory,
        _ => return Err(InputError::Invariant("thief turn memory is absent".into()).into()),
    };
    let next_branch = if intent == mug && move_history.as_slice() == [mug, mug] {
        if memory.pending_next.is_some() {
            return Err(InputError::Invariant("thief two-Mug branch is already committed".into()).into());
        }
        let stream = enemy.full_ai_rng.as_ref()
            .ok_or_else(|| InputError::Invariant("thief actor AI stream is absent".into()))?;
        if stream.id().domain() != rng::Domain::RunCombatEnemyAi {
            return Err(InputError::Unsupported("thief requires actor-owned AI stream".into()).into());
        }
        let (smoke_branch, next_stream) = rng::bernoulli(stream, 1, 2)?;
        Some((if smoke_branch { smoke } else { lunge }, next_stream))
    } else {
        if memory.pending_next.is_some() {
            return Err(InputError::Invariant("thief branch latch escaped its second-Mug turn".into()).into());
        }
        None
    };
    if let Some((branch, next_stream)) = next_branch {
        let enemy = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner)
            .expect("validated thief turn owner");
        enemy.full_ai_rng = Some(next_stream);
        let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_mut() else { unreachable!() };
        memory.pending_next = Some(branch);
    }
    let actor = Actor::Enemy(owner.clone());
    if is_attack_intent(id, intent) {
        let ledger = ThiefEffect::Ledger { owner: owner.clone(), cap: THIEVERY_GOLD_A0, intent };
        let damage = ThiefEffect::TheftDamage { owner: owner.clone(), cap: THIEVERY_GOLD_A0, intent };
        Ok(vec![
            make_effect(actor.clone(), Actor::Player, EffectKind::Thief(ledger)),
            make_effect(actor, Actor::Player, EffectKind::Thief(damage)),
        ])
    } else if intent.as_str() == intent_name(id, "smoke_bomb") {
        let block = if id == EnemyId::Looter { 6 } else { 11 };
        Ok(vec![make_effect(actor.clone(), actor, EffectKind::GainBlock(block))])
    } else if intent.as_str() == intent_name(id, "escape") {
        Ok(vec![make_effect(actor.clone(), actor,
            EffectKind::Thief(ThiefEffect::Escape { owner: owner.clone() }))])
    } else {
        Err(InputError::Invariant("thief move has no source-qualified payload".into()).into())
    }
}

/// Hook producer for an actual final death; escapes and pending deaths earn none.
pub(in crate::combat) fn death_reward_work(state: &State, target: Option<&Actor>) -> Option<Effect> {
    let Some(Actor::Enemy(owner)) = target else { return None; };
    let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)?;
    let ActorEnemyId::Full(id) = &enemy.enemy_id else { return None; };
    let id = *id;
    if !is_thief_id(id) || enemy.hp != 0 || enemy.life != ActorLife::Active { return None; }
    let Some(EnemyMemory::Thief(memory)) = enemy.full_memory.as_ref() else { return None; };
    if memory.stolen_gold == 0 || memory.pending_theft.is_some() || memory.reward_recorded { return None; }
    Some(Effect {
        source: Source::None,
        target: Some(Actor::Enemy(owner.clone())),
        kind: EffectKind::Thief(ThiefEffect::DeathReward { owner: owner.clone() }),
    })
}
