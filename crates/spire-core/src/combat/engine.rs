use std::collections::{BTreeMap, BTreeSet, VecDeque};
use super::*;
use super::model::State;
use super::rng::{Domain, RNG_CONTRACT_ID};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combat { pub(super) state: State }

fn require(ok: bool, message: &str) -> Result<(), InputError> {
    if ok { Ok(()) } else { Err(InputError::Invariant(message.into())) }
}

fn enemy_bounds(kind: EnemyKind) -> std::ops::RangeInclusive<i32> {
    match kind { EnemyKind::Cultist => 48..=54, EnemyKind::JawWorm => 40..=44 }
}

fn first_move(kind: EnemyKind) -> MoveId {
    match kind { EnemyKind::Cultist => MoveId::Incantation, EnemyKind::JawWorm => MoveId::Chomp }
}

fn supports_move(kind: EnemyKind, id: MoveId) -> bool {
    match kind {
        EnemyKind::Cultist => matches!(id, MoveId::Incantation | MoveId::Attack),
        EnemyKind::JawWorm => matches!(id, MoveId::Chomp | MoveId::Thrash | MoveId::Bellow),
    }
}

fn base_damage(kind: EnemyKind) -> BTreeMap<MoveId, DamageEntry> {
    let entries = match kind {
        EnemyKind::Cultist => vec![(MoveId::Attack, 6)],
        EnemyKind::JawWorm => vec![(MoveId::Chomp, 11), (MoveId::Thrash, 7)],
    };
    entries.into_iter().map(|(id, base)| (id, DamageEntry { base, output: base })).collect()
}

fn validate_player(player: &PlayerState) -> Result<(), InputError> {
    require((1..=999).contains(&player.max_hp) && (0..=player.max_hp).contains(&player.hp), "player HP bounds")?;
    require((1..=999).contains(&player.max_energy) && (0..=player.max_energy).contains(&player.energy), "player energy bounds")?;
    require((0..=999).contains(&player.block) && (1..=10).contains(&player.draw_per_turn), "block/draw bounds")
}

fn validate_rng(rng: &CombatRng, owner: &EnemyHandle) -> Result<(), InputError> {
    require(rng.contract_id == RNG_CONTRACT_ID, "RNG contract")?;
    require(&rng.enemy_instance_id == owner, "RNG owner")?;
    let shuffle = rng.shuffle.id();
    let ai = rng.enemy_ai.id();
    require(shuffle.domain() == Domain::CombatShuffle && ai.domain() == Domain::CombatEnemyAi,
        "RNG domains")?;
    require(shuffle.scope().len() == 1 && ai.scope().len() == 2
        && shuffle.scope()[0] == ai.scope()[0], "RNG combat scope")
    // StreamId/StreamState constructors already enforce private identity/cursor bounds.
}

fn owned_cards<'a>(piles: &'a Piles, in_play: Option<&'a CardInstance>) -> impl Iterator<Item = &'a CardInstance> {
    piles.hand.iter().chain(&piles.draw).chain(&piles.discard).chain(&piles.exhaust).chain(in_play)
}

fn validate_cards<'a>(piles: &'a Piles, in_play: Option<&'a CardInstance>) -> Result<BTreeSet<&'a CardHandle>, InputError> {
    require(piles.hand.len() <= 10, "hand cap")?;
    let mut issued = BTreeSet::new();
    for card in owned_cards(piles, in_play) {
        require(issued.insert(&card.instance_id), "duplicate card ownership")?;
    }
    require((1..=999).contains(&issued.len()), "deck bounds")?;
    Ok(issued)
}

pub(super) fn validate_scenario(scenario: &Scenario) -> Result<(), InputError> {
    let p = &scenario.player;
    validate_player(p)?;
    require(p.hp > 0 && p.energy == p.max_energy && p.block == 0 && p.powers.is_empty(), "first player decision")?;
    require(scenario.enemies.len() == 1, "exactly one enemy")?;
    let e = &scenario.enemies[0];
    require(scenario.encounter_id == e.enemy_id, "encounter/enemy mismatch")?;
    require(enemy_bounds(e.enemy_id).contains(&e.max_hp) && e.hp == e.max_hp
        && e.block == 0 && e.powers.is_empty(), "first enemy values")?;
    let first = first_move(e.enemy_id);
    require(e.intent_id == first && e.move_history == [first], "first selected move/history")?;
    require(scenario.piles.discard.is_empty() && scenario.piles.exhaust.is_empty(), "initial discard/exhaust")?;
    validate_cards(&scenario.piles, None)?;
    validate_rng(&scenario.rng, &e.instance_id)
}

impl Combat {
    pub fn legal_actions(&self) -> Result<Vec<Action>, AccessError> {
        super::view::legal_actions(&self.state)
    }

    pub fn public_view(&self) -> Result<PublicView, AccessError> {
        super::view::public_view(&self.state)
    }

    pub fn step(&mut self, action: &Action) -> Result<Transition, StepError> {
        if self.state.fault.is_some() { return Err(StepError::Faulted); }
        if self.state.phase == Phase::Terminal { return Err(StepError::Terminal); }
        if self.state.phase != Phase::Decision || super::view::ready(&self.state).is_err() {
            return Err(StepError::NotReady);
        }
        // Preflight every caller-controlled condition and calculation before changing
        // ownership, energy, queues or the previous external step's dispatch budget.
        let prepared = match action {
            Action::EndTurn => None,
            Action::PlayCard { card_instance_id, target_id } => {
                let index = self.state.piles.hand.iter().position(|c| &c.instance_id == card_instance_id)
                    .ok_or(StepError::InvalidAction)?;
                let card = &self.state.piles.hand[index];
                let stats = card_stats(card.card_id, card.upgraded);
                if self.state.player.energy < stats.cost { return Err(StepError::InvalidAction); }
                let target = match card.card_id {
                    CardKind::Defend => {
                        if target_id.is_some() { return Err(StepError::InvalidAction); }
                        Actor::Player
                    }
                    CardKind::Strike | CardKind::Bash => {
                        let enemy = self.state.enemies.iter().find(|e| e.hp > 0
                            && target_id.as_ref() == Some(&e.instance_id)).ok_or(StepError::InvalidAction)?;
                        Actor::Enemy(enemy.instance_id.clone())
                    }
                };
                let strength = self.state.player.powers.iter().find(|p| p.id == PowerId::Strength)
                    .map_or(0, |p| p.state.amount);
                let vulnerable = self.state.enemies[0].powers.iter().any(|p|
                    p.id == PowerId::Vulnerable && p.state.amount > 0);
                let damage = if card.card_id == CardKind::Defend { 0 } else {
                    rules::normal_damage(stats.damage, strength, vulnerable).map_err(|_| StepError::InvalidAction)?
                };
                Some((index, stats, target, damage))
            }
        };
        self.state.resolved_effects = 0;
        self.state.phase = Phase::PlayerAction;
        if let Some((index, stats, target, damage)) = prepared {
            let card = self.state.piles.hand.remove(index);
            let source = Source::Card(card.instance_id.clone());
            let defend = card.card_id == CardKind::Defend;
            self.state.in_play = Some(card);
            self.state.player.energy -= stats.cost;
            self.state.pending.push_back(Effect { source: source.clone(), target: Some(target.clone()),
                kind: if defend { EffectKind::GainBlock(stats.block) }
                    else { EffectKind::Damage(DamagePayload::Fixed(damage)) } });
            if stats.vulnerable > 0 {
                self.state.pending.push_back(Effect { source: source.clone(), target: Some(target),
                    kind: EffectKind::ApplyPower { id: PowerId::Vulnerable, amount: stats.vulnerable } });
            }
            self.state.pending.push_back(Effect { source, target: None, kind: EffectKind::FinishCard });
            self.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
        } else {
            self.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::EndPlayerTurn });
        }
        Ok(resolve(&mut self.state))
    }

    pub fn new(scenario: &Scenario) -> Result<Self, InputError> {
        validate_scenario(scenario)?;
        // Only borrow until validation succeeds; no stream is sampled or normalized.
        let enemies = scenario.enemies.iter().map(|e| {
            let damage_entries = base_damage(e.enemy_id);
            let intent_damage = damage_entries.get(&e.intent_id).map(|d| d.output);
            EnemyState { instance_id: e.instance_id.clone(), enemy_id: e.enemy_id,
                hp: e.hp, max_hp: e.max_hp, block: e.block, powers: e.powers.clone(),
                intent_id: e.intent_id, move_history: e.move_history.clone(), selected_move: e.intent_id,
                damage_entries, intent_damage }
        }).collect();
        Ok(Self { state: State { content_id: CONTENT_ID.into(), phase: Phase::Decision, turn: 1,
            player: scenario.player.clone(), enemies, piles: scenario.piles.clone(),
            card_ids_issued: owned_cards(&scenario.piles, None).map(|c| c.instance_id.clone()).collect(),
            rng: scenario.rng.clone(), pending: VecDeque::new(), current: None, in_play: None,
            resolved_effects: 0, public: PublicKnowledge { revealed_order: vec![] },
            outcome: None, fault: None, hp_before_healing: None, hp_after_cleanup: None } })
    }
}

fn validate_powers(powers: &[PowerEntry], enemy: Option<EnemyKind>, state: &State, actor: &Actor) -> Result<(), InputError> {
    let mut seen = BTreeSet::new();
    for entry in powers {
        require(seen.insert(entry.id), "duplicate power")?;
        let p = &entry.state;
        // A full reduction leaves the object at zero until its front-queued removal.
        let removing = state.pending.front().map_or(false, |w|
            w.target.as_ref() == Some(actor) && w.kind == EffectKind::RemovePower(entry.id));
        require(p.amount > 0 || (p.amount == 0 && removing), "positive power amount or pending removal")?;
        match entry.id {
            PowerId::Strength => require(p.amount <= 999 && !p.skip_first && !p.monster_source_grace, "Strength bounds/flags")?,
            PowerId::Ritual => require(enemy == Some(EnemyKind::Cultist) && p.amount % 3 == 0
                && !p.monster_source_grace, "Ritual owner/amount/flags")?,
            PowerId::Vulnerable => require(!p.skip_first, "Vulnerable flags")?,
        }
    }
    Ok(())
}

fn validate_actor(actor: &Actor, state: &State) -> Result<(), InputError> {
    require(match actor { Actor::Player => true, Actor::Enemy(id) => id == &state.enemies[0].instance_id }, "effect actor owner")
}

pub(super) fn validate_effect(effect: &Effect, state: &State) -> Result<(), InputError> {
    require(state.enemies.len() == 1, "effect enemy count")?;
    match &effect.source {
        Source::None => (),
        Source::Card(id) => require(state.card_ids_issued.contains(id), "effect card owner")?,
        Source::Actor(actor) => validate_actor(actor, state)?,
    }
    if let Some(actor) = &effect.target { validate_actor(actor, state)?; }
    if matches!(&effect.kind, EffectKind::Damage(_) | EffectKind::GainBlock(_)
        | EffectKind::ApplyPower { .. } | EffectKind::ReducePower { .. } | EffectKind::RemovePower(_)) {
        require(effect.target.is_some(), "targeted payload requires an actor")?;
    }
    if matches!(&effect.kind, EffectKind::ApplyPower { id: PowerId::Ritual, .. }
        | EffectKind::ReducePower { id: PowerId::Ritual, .. } | EffectKind::RemovePower(PowerId::Ritual)) {
        require(matches!(&effect.target, Some(Actor::Enemy(_)))
            && state.enemies[0].enemy_id == EnemyKind::Cultist, "Ritual payload owner")?;
    }
    match &effect.kind {
        EffectKind::Damage(DamagePayload::Fixed(n)) => require((0..=1515).contains(n), "damage payload bounds")?,
        EffectKind::Damage(DamagePayload::EnemyMove(id)) => {
            if !state.enemies[0].damage_entries.contains_key(id) {
                return Err(InputError::Unsupported("incompatible enemy damage move".into()));
            }
            require(effect.source == Source::Actor(Actor::Enemy(state.enemies[0].instance_id.clone()))
                && effect.target == Some(Actor::Player), "enemy damage source/target")?;
        }
        EffectKind::GainBlock(n) => require((0..=1007).contains(n), "block payload bounds")?,
        EffectKind::Draw(n) => require((0..=10).contains(n), "draw payload bounds")?,
        EffectKind::ApplyPower { id, amount } | EffectKind::ReducePower { id, amount } => {
            require(*amount > 0, "power payload bounds")?;
            if *id == PowerId::Strength { require(*amount <= 1002, "Strength payload bounds")?; }
            if *id == PowerId::Ritual {
                require(*amount == 3 && matches!(&effect.target, Some(Actor::Enemy(_)))
                    && state.enemies[0].enemy_id == EnemyKind::Cultist, "Ritual payload")?;
            }
        }
        EffectKind::FinishCard => require(match (&effect.source, &state.in_play) {
            (Source::Card(id), Some(card)) => id == &card.instance_id,
            _ => false,
        }, "card continuation requires matching in-play source")?,
        EffectKind::Terminal(outcome) => require(match outcome {
            Outcome::Victory => state.enemies[0].hp == 0 && state.player.hp > 0,
            Outcome::Death => state.player.hp == 0 && state.enemies[0].hp > 0,
        }, "terminal payload actors")?,
        _ => (),
    }
    Ok(())
}

pub(super) fn validate_runtime(state: &State) -> Result<(), InputError> {
    require(state.content_id == CONTENT_ID, "runtime content")?;
    require((1..=i32::MAX as u32).contains(&state.turn), "turn bounds")?;
    require(state.resolved_effects <= 100000, "dispatch count bounds")?;
    if let Some(fault) = &state.fault {
        // Diagnostic state may retain an interrupted payload and inconsistent gameplay.
        return require(fault.resolved_effects == state.resolved_effects, "fault count");
    }
    require(state.resolved_effects < 100000, "unfaulted dispatch guard")?;
    require(state.current.is_none(), "interrupted current payload cannot resume")?;
    validate_player(&state.player)?;
    validate_powers(&state.player.powers, None, state, &Actor::Player)?;
    require(state.enemies.len() == 1, "runtime enemy count")?;
    let e = &state.enemies[0];
    require(enemy_bounds(e.enemy_id).contains(&e.max_hp) && (0..=e.max_hp).contains(&e.hp)
        && (0..=999).contains(&e.block), "enemy bounds")?;
    validate_powers(&e.powers, Some(e.enemy_id), state, &Actor::Enemy(e.instance_id.clone()))?;
    require(supports_move(e.enemy_id, e.intent_id) && supports_move(e.enemy_id, e.selected_move), "enemy moves")?;
    require(e.move_history.first() == Some(&first_move(e.enemy_id))
        && e.move_history.last() == Some(&e.selected_move)
        && e.move_history.iter().all(|id| supports_move(e.enemy_id, *id)), "selected history")?;
    if e.enemy_id == EnemyKind::Cultist {
        require(e.move_history.iter().skip(1).all(|id| *id == MoveId::Attack), "Cultist history")?;
    }
    let bases = base_damage(e.enemy_id);
    require(e.damage_entries.len() == bases.len(), "damage entry membership")?;
    for (id, base) in bases {
        let entry = e.damage_entries.get(&id).ok_or_else(|| InputError::Invariant("missing damage entry".into()))?;
        require(entry.base == base.base && (0..=1515).contains(&entry.output), "damage entry bounds")?;
    }
    require(e.intent_damage.map_or(true, |n| (0..=1515).contains(&n)), "intent damage bounds")?;
    require(e.intent_damage.is_some() == e.damage_entries.contains_key(&e.intent_id), "published intent damage shape")?;
    validate_rng(&state.rng, &e.instance_id)?;
    let owned = validate_cards(&state.piles, state.in_play.as_ref())?;
    require(owned.len() == state.card_ids_issued.len()
        && owned.iter().all(|id| state.card_ids_issued.contains(*id)), "issued card ownership completeness")?;
    require(state.public.revealed_order.iter().all(|id| state.card_ids_issued.contains(id)), "public reveal owner")?;
    for effect in &state.pending { validate_effect(effect, state)?; }
    let quiet = state.pending.is_empty() && state.in_play.is_none();
    match state.phase {
        Phase::Decision => {
            require(quiet && state.player.hp > 0 && e.hp > 0, "live quiescent decision")?;
            require(e.intent_id == e.selected_move, "decision published move")?;
            let strength = e.powers.iter().find(|p| p.id == PowerId::Strength).map_or(0, |p| p.state.amount);
            let vulnerable = state.player.powers.iter().any(|p| p.id == PowerId::Vulnerable);
            for entry in e.damage_entries.values() {
                let normal = i64::from(entry.base) + i64::from(strength);
                let expected = if vulnerable { normal * 3 / 2 } else { normal };
                require(i64::from(entry.output) == expected, "decision refreshed damage")?;
            }
            require(e.intent_damage == e.damage_entries.get(&e.intent_id).map(|d| d.output), "decision published damage")?;
        }
        Phase::Terminal => {
            // Lethality prunes FinishCard: the lethal card keeps its in-play owner.
            require(state.pending.is_empty(), "terminal cleanup work")?;
            require(state.hp_after_cleanup == Some(state.player.hp), "terminal cleanup HP")?;
            match state.outcome {
                Some(Outcome::Victory) => {
                    require(e.hp == 0 && state.player.hp > 0, "victory actors")?;
                    let before = state.hp_before_healing.ok_or_else(|| InputError::Invariant("missing prehealing HP".into()))?;
                    require((1..=state.player.max_hp).contains(&before)
                        && (i64::from(before) + 6).min(i64::from(state.player.max_hp)) == i64::from(state.player.hp), "victory healing")?;
                }
                Some(Outcome::Death) => require(state.player.hp == 0 && e.hp > 0
                    && state.hp_before_healing == Some(0), "death HP")?,
                None => return Err(InputError::Invariant("terminal outcome missing".into())),
            }
        }
        _ => {
            require(!state.pending.is_empty(), "busy continuation missing")?;
            require(state.in_play.is_none() || state.phase == Phase::PlayerAction, "in-play phase")?;
            if state.in_play.is_some() {
                require(state.pending.iter().any(|w| matches!(&w.kind,
                    EffectKind::FinishCard | EffectKind::Terminal(_))), "in-play continuation missing")?;
            }
            if state.player.hp == 0 || e.hp == 0 {
                require(!(state.player.hp == 0 && e.hp == 0), "unsupported simultaneous lethality")?;
                let outcome = if state.player.hp == 0 { Outcome::Death } else { Outcome::Victory };
                require(state.pending.iter().any(|w| w.kind == EffectKind::Terminal(outcome)), "lethal pending terminal work")?;
            }
        }
    }
    if state.phase != Phase::Terminal {
        require(state.outcome.is_none() && state.hp_before_healing.is_none()
            && state.hp_after_cleanup.is_none(), "premature terminal values")?;
    }
    Ok(())
}

/// Convert checked payload errors without replacing the first fault or another
/// stream's progress. Future random owners use this same boundary.
pub(super) fn retain_error(state: &mut State, error: EffectError) {
    use super::model::mark_fault;
    use super::rng::RngError;
    if state.fault.is_some() { return; }
    let (kind, message) = match error {
        EffectError::Invalid(InputError::Unsupported(message)) => (FaultKind::Unsupported, message),
        EffectError::Invalid(InputError::Invariant(message)) |
        EffectError::Random(RngError::Invalid(message)) => (FaultKind::Invariant, message),
        EffectError::Random(RngError::Exhausted { state: latest }) => {
            let owned = if latest.id() == state.rng.shuffle.id() {
                Some(&mut state.rng.shuffle)
            } else if latest.id() == state.rng.enemy_ai.id() {
                Some(&mut state.rng.enemy_ai)
            } else { None };
            match owned {
                Some(stream) if stream.key() == latest.key()
                    && latest.word_index() >= stream.word_index() => {
                    *stream = latest;
                    (FaultKind::RngExhausted, "owned random stream exhausted".into())
                }
                _ => (FaultKind::Invariant, "exhaustion stream identity/key/cursor mismatch".into()),
            }
        }
    };
    mark_fault(state, kind, message);
}

pub(super) fn dispatch_one(state: &mut State) {
    use super::model::mark_fault;
    if state.fault.is_some() { return; }
    if state.current.is_some() {
        mark_fault(state, FaultKind::Invariant, "interrupted current payload cannot retry".into());
        return;
    }
    let Some(effect) = state.pending.pop_front() else { return; };
    state.current = Some(effect);
    // The attempted 100,000th dispatch is recorded, but never executes payload.
    if state.resolved_effects >= 100000 {
        mark_fault(state, FaultKind::Invariant, "dispatch count already exhausted".into());
        return;
    }
    state.resolved_effects += 1;
    if state.resolved_effects == 100000 {
        mark_fault(state, FaultKind::EffectGuard, "effect dispatch limit".into());
        return;
    }
    // Clone only the selected data-only work, never the combat. Keeping the owner
    // in current protects it from front insertion and preserves failures verbatim.
    let effect = state.current.as_ref().unwrap().clone();
    match super::effects::apply_effect(state, &effect) {
        Ok(()) => state.current = None,
        Err(error) => retain_error(state, error),
    }
}

pub(super) fn resolve(state: &mut State) -> Transition {
    loop {
        if state.fault.is_some() { return Transition { status: TransitionStatus::Fault }; }
        if state.current.is_some() {
            dispatch_one(state);
            continue;
        }
        if !state.pending.is_empty() {
            dispatch_one(state);
            continue;
        }
        let status = match state.phase {
            Phase::Decision => TransitionStatus::Decision,
            Phase::Terminal => TransitionStatus::Terminal,
            _ => {
                super::model::mark_fault(state, FaultKind::Invariant, "resolution has no continuation".into());
                TransitionStatus::Fault
            }
        };
        return Transition { status };
    }
}
