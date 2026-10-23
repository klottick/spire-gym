use std::collections::{BTreeMap, BTreeSet, VecDeque};
use super::*;
use super::model::{OwnedCard, OwnedPiles, State};
use super::rng::{Domain, RNG_CONTRACT_ID};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combat { pub(super) state: State, pub(super) legacy_cache: Piles }

/// Preserve post-mutation combat state when startup hooks or validation fail.
#[derive(Clone, PartialEq, Eq)]
pub enum CombatEntryFailure {
    Rejected(crate::run::RunError),
    Faulted { combat: Box<Combat>, fault: crate::run::RunFault },
}

impl std::fmt::Debug for CombatEntryFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(error) => f.debug_tuple("Rejected").field(error).finish(),
            Self::Faulted { fault, .. } => f.debug_struct("Faulted").field("fault", fault).finish_non_exhaustive(),
        }
    }
}

fn require(ok: bool, message: &str) -> Result<(), InputError> {
    if ok { Ok(()) } else { Err(InputError::Invariant(message.into())) }
}

fn enemy_bounds(kind: EnemyKind) -> std::ops::RangeInclusive<i32> {
    match kind { EnemyKind::Cultist => 48..=54, EnemyKind::JawWorm => 40..=44 }
}

fn first_move(kind: EnemyKind) -> MoveId {
    match kind { EnemyKind::Cultist => MoveId::Incantation, EnemyKind::JawWorm => MoveId::Chomp }
}

pub(super) fn supports_move_for_turn(kind: EnemyKind, id: MoveId) -> bool {
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

fn supported_full_damage(enemy: crate::content::EnemyId, ascension: u8) -> BTreeMap<super::FullIntentId, DamageEntry> {
    if matches!(enemy, crate::content::EnemyId::Unknown(_)) { return BTreeMap::new(); }
    super::enemies::damage_bases_at_ascension(enemy, ascension)
        .expect("validated full enemy has canonical damage bases")
        .into_iter().map(|(id, base)| (id, DamageEntry { base, output: base })).collect()
}

fn valid_full_powers(powers: &[super::FullPowerEntry]) -> bool {
    let mut ids = BTreeSet::new();
    powers.iter().all(|p| (if matches!(p.id.0, "strength" | "dexterity") {
            (-999..=999).contains(&p.state.amount) && p.state.amount != 0
        } else if p.id.0=="mode_shift" {p.state.amount>0}
        else { (1..=999).contains(&p.state.amount) }) && ids.insert(p.id)
        && (!p.state.skip_first || p.id.0 == "ritual")
        && (!p.state.monster_source_grace || matches!(p.id.0, "vulnerable" | "weak" | "frail")))
}

fn removing_power(state: &State, actor: &Actor, id: &str) -> bool {
    state.pending.front().is_some_and(|effect| effect.target.as_ref() == Some(actor)
        && match effect.kind {
            EffectKind::RemovePower(power) => power.as_str() == id,
            EffectKind::RemoveFullPower(power) => power.0 == id,
            _ => false,
        })
}

fn valid_runtime_full_powers(powers: &[FullPowerEntry], state: &State, actor: &Actor) -> bool {
    let mut seen = BTreeSet::new();
    powers.iter().all(|power| seen.insert(power.id) &&
        (valid_full_powers(std::slice::from_ref(power))
            || (power.state.amount == 0 && removing_power(state, actor, power.id.0)
                && !power.state.skip_first && !power.state.monster_source_grace)))
}

fn validate_player(player: &PlayerState, full_runtime: bool) -> Result<(), InputError> {
    require((1..=999).contains(&player.max_hp) && (0..=player.max_hp).contains(&player.hp), "player HP bounds")?;
    let energy_limit = if full_runtime { 999 } else { player.max_energy };
    require((1..=999).contains(&player.max_energy) && (0..=energy_limit).contains(&player.energy), "player energy bounds")?;
    require((0..=999).contains(&player.block) && (1..=10).contains(&player.draw_per_turn), "block/draw bounds")
}

fn validate_rng(rng: &CombatRng, owner: &EnemyHandle, full: bool) -> Result<(), InputError> {
    require(rng.contract_id == RNG_CONTRACT_ID, "RNG contract")?;
    require(&rng.enemy_instance_id == owner, "RNG owner")?;
    let shuffle = rng.shuffle.id();
    let ai = rng.enemy_ai.id();
    let domains_ok = if full {
        shuffle.domain() == Domain::RunCombatShuffle && ai.domain() == Domain::RunCombatEnemyAi
    } else {
        shuffle.domain() == Domain::CombatShuffle && ai.domain() == Domain::CombatEnemyAi
    };
    require(domains_ok,
        "RNG domains")?;
    require(shuffle.scope().len() == 1 && ai.scope().len() == 2
        && shuffle.scope()[0] == ai.scope()[0], "RNG combat scope")
    // StreamId/StreamState constructors already enforce private identity/cursor bounds.
}

fn owned_cards<'a>(piles: &'a OwnedPiles, in_play: Option<&'a OwnedCard>) -> impl Iterator<Item = &'a OwnedCard> {
    piles.all(in_play)
}

fn validate_cards<'a>(piles: &'a OwnedPiles, in_play: Option<&'a OwnedCard>,held:impl Iterator<Item=&'a OwnedCard>) -> Result<BTreeSet<CardHandle>, InputError> {
    require(piles.hand.len() <= 10, "hand cap")?;
    let mut issued = BTreeSet::new();
    for card in owned_cards(piles, in_play).chain(held) {
        require(issued.insert(card.handle()), "duplicate card ownership")?;
    }
    require((1..=999).contains(&issued.len()), "deck bounds")?;
    Ok(issued)
}

pub(super) fn validate_scenario(scenario: &Scenario) -> Result<(), InputError> {
    validate_scenario_mode(scenario, false)
}

pub(super) fn validate_scenario_mode(scenario: &Scenario, full: bool) -> Result<(), InputError> {
    let p = &scenario.player;
    validate_player(p, false)?;
    if full {
        require(p.hp > 0 && p.block == 0 && p.powers.is_empty(), "full-entry player setup")?;
        require(scenario.piles.hand.is_empty() && scenario.piles.draw.is_empty(), "full-entry cards supplied separately")?;
    } else {
        require(p.hp > 0 && p.energy == p.max_energy && p.block == 0 && p.powers.is_empty(), "first player decision")?;
    }
    require(if full { (1..=9).contains(&scenario.enemies.len()) } else { scenario.enemies.len() == 1 },
        if full { "full-entry enemy count" } else { "exactly one enemy" })?;
    require(scenario.piles.discard.is_empty() && scenario.piles.exhaust.is_empty(), "initial discard/exhaust")?;
    let mut enemy_ids = BTreeSet::new();
    for e in &scenario.enemies {
        require(enemy_ids.insert(&e.instance_id), "duplicate enemy identity")?;
        if !full { require(scenario.encounter_id == e.enemy_id, "encounter/enemy mismatch")?; }
        require(enemy_bounds(e.enemy_id).contains(&e.max_hp)
            && if full { (1..=e.max_hp).contains(&e.hp) } else { e.hp == e.max_hp }
            && e.block == 0 && e.powers.is_empty(), "first enemy values")?;
        let first = first_move(e.enemy_id);
        require(e.intent_id == first && e.move_history == [first], "first selected move/history")?;
    }
    if !full {
        let owned = OwnedPiles::from_legacy(&scenario.piles);
        validate_cards(&owned, None,std::iter::empty())?;
    }
    validate_rng(&scenario.rng, &scenario.enemies[0].instance_id, full)
}

impl Combat {
    /// Enqueue a typed full-runtime lifecycle event through the authoritative
    /// resolver. Producers supply owned source/target identities; hook handlers
    /// and resulting effects remain combat-owned.
    pub fn enqueue_hook(&mut self, event: super::HookEvent) -> Result<(), crate::run::RunError> {
        if !self.state.full_runtime { return Err(crate::run::RunError::InvalidAction); }
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if self.state.phase != Phase::Decision || super::view::ready(&self.state).is_err() {
            return Err(crate::run::RunError::InvalidAction);
        }
        if event.hook == Hook::CombatStarted
            && self.state.hook_events.iter().any(|record| record.hook == Hook::CombatStarted) {
            return Err(crate::run::RunError::InvalidAction);
        }
        let effect = Effect { source: event.source.clone(), target: event.target.clone(), kind: EffectKind::Hook(event.hook) };
        validate_effect(&effect, &self.state).map_err(|_| crate::run::RunError::InvalidAction)?;
        self.state.resolved_effects = 0;
        self.state.pending.push_back(effect);
        let transition = resolve(&mut self.state);
        self.refresh_legacy_cache();
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        match transition.status {
            TransitionStatus::Decision | TransitionStatus::Terminal => Ok(()),
            TransitionStatus::Selection | TransitionStatus::Fault => Err(crate::run::RunError::InvalidAction),
        }
    }

    pub(crate) fn install_item_context(&mut self, context: crate::items::CombatItemContext)
        -> Result<(), crate::run::RunError> {
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if !self.state.full_runtime || self.state.item_context.is_some()
            || self.state.phase != Phase::Decision || super::view::ready(&self.state).is_err() {
            return Err(crate::run::RunError::InvalidAction);
        }
        self.state.item_context = Some(context);
        Ok(())
    }

    pub(crate) fn item_context(&self) -> Option<&crate::items::CombatItemContext> {
        self.state.item_context.as_ref()
    }

    /// Actual combat-owned values needed to prepare a manual potion use. This
    /// narrow crate-private projection is not part of the public observation.
    pub(crate) fn native_potion_use_state(&self)
        -> Result<(Phase, i32, i32, Vec<EnemyHandle>), crate::run::RunError>
    {
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if !self.state.full_runtime || super::view::ready(&self.state).is_err() {
            return Err(crate::run::RunError::InvalidAction);
        }
        let live_targets = self.state.enemies.iter()
            .filter(|enemy| enemy.life == super::ActorLife::Active && enemy.hp > 0)
            .map(|enemy| enemy.instance_id.clone()).collect();
        Ok((self.state.phase, self.state.player.hp, self.state.player.max_hp, live_targets))
    }

    pub(crate) fn preflight_native_effects(&self, effects: &[Effect]) -> Result<(), crate::run::RunError> {
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if !self.state.full_runtime || self.state.phase != Phase::Decision
            || super::view::ready(&self.state).is_err()
            {
            return Err(crate::run::RunError::InvalidAction);
        }
        for effect in effects {
            validate_effect(effect, &self.state).map_err(|_| crate::run::RunError::InvalidAction)?;
            if let (Some(Actor::Enemy(target)), EffectKind::Damage(_)) =
                (effect.target.as_ref(), &effect.kind) {
                if self.state.enemies.iter().find(|enemy| &enemy.instance_id == target)
                    .map_or(true, |enemy| enemy.hp <= 0) {
                    return Err(crate::run::RunError::InvalidAction);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn enqueue_native_effects(&mut self, effects: Vec<Effect>,
        next_items: Option<crate::items::CombatItemContext>) -> Result<(), crate::run::RunError> {
        self.preflight_native_effects(&effects)?;
        if next_items.is_some() && self.state.item_context.is_none() {
            return Err(crate::run::RunError::InvalidAction);
        }
        self.state.resolved_effects = 0;
        for effect in effects.into_iter().rev() { self.state.pending.push_front(effect); }
        if let Some(items) = next_items { self.state.item_context = Some(items); }
        let transition = resolve(&mut self.state);
        self.refresh_legacy_cache();
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        match transition.status {
            TransitionStatus::Decision | TransitionStatus::Terminal => Ok(()),
            TransitionStatus::Selection | TransitionStatus::Fault => Err(crate::run::RunError::InvalidAction),
        }
    }

    pub fn legal_actions(&self) -> Result<Vec<Action>, AccessError> {
        if self.state.full_runtime { return Err(AccessError::NotReady); }
        super::view::legal_actions(&self.state)
    }

    pub fn public_view(&self) -> Result<PublicView, AccessError> {
        if self.state.full_runtime { return Err(AccessError::NotReady); }
        super::view::public_view(&self.state)
    }

    pub fn step(&mut self, action: &Action) -> Result<Transition, StepError> {
        if self.state.full_runtime { return Err(StepError::NotReady); }
        self.step_internal(action)
    }

    fn step_internal(&mut self, action: &Action) -> Result<Transition, StepError> {
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
                let index = self.state.piles.hand.iter().position(|c| c.handle() == *card_instance_id)
                    .ok_or(StepError::InvalidAction)?;
                let card = &self.state.piles.hand[index];
                let (kind, upgraded, live) = if let Some(legacy) = card.legacy() {
                    (legacy.card_id, legacy.upgraded,
                        super::card_state::legacy_identity(legacy.card_id, 0, legacy.upgraded))
                } else if let Some(full) = card.full() {
                    (super::card_state::legacy_kind(full.id).ok_or(StepError::InvalidAction)?,
                        full.upgrades > 0, full.clone())
                } else { return Err(StepError::InvalidAction); };
                let stats = card_stats(kind, upgraded);
                let payment = super::card_state::payment(&live, self.state.player.energy, stats.cost)
                    .map_err(|_| StepError::InvalidAction)?;
                let target = match kind {
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
                let vulnerable = match &target {
                    Actor::Enemy(id) => self.state.enemies.iter().find(|e| &e.instance_id == id)
                        .map_or(false, |e| e.powers.iter().any(|p| p.id == PowerId::Vulnerable && p.state.amount > 0)),
                    Actor::Player => false,
                };
                let damage = if kind == CardKind::Defend { 0 } else {
                    rules::normal_damage(stats.damage, strength, vulnerable).map_err(|_| StepError::InvalidAction)?
                };
                Some((index, stats, target, damage, payment.remaining_energy, kind == CardKind::Defend))
            }
        };
        self.state.resolved_effects = 0;
        self.state.phase = Phase::PlayerAction;
        if let Some((index, stats, target, damage, remaining_energy, defend)) = prepared {
            let mut card = self.state.piles.hand.remove(index);
            let card_id = card.handle();
            let source = Source::Card(card_id.clone());
            if let Some(full) = card.full_mut() { full.local.free_play = false; }
            self.state.in_play = Some(card);
            self.state.player.energy = remaining_energy;
            if self.state.full_runtime {
                self.state.pending.push_back(Effect { source: source.clone(), target: Some(target.clone()),
                    kind: EffectKind::Hook(Hook::CardPlayed) });
            }
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
        let transition = resolve(&mut self.state);
        self.refresh_legacy_cache();
        Ok(transition)
    }

    /// Enter combat from a run-owned deck. The single combat state and resolver
    /// are reused; this entry shuffles and draws before publishing the decision.
    pub fn enter(entry: CombatEntry) -> Result<Self, crate::run::RunError> {
        match Self::enter_with_item_context(entry, &mut None) {
            Ok(combat) => Ok(combat),
            Err(CombatEntryFailure::Rejected(error)) => Err(error),
            Err(CombatEntryFailure::Faulted { fault, .. }) => Err(crate::run::RunError::Faulted(fault)),
        }
    }

    /// Lossless run bridge entry. Rejection before inventory transfer leaves
    /// `item_context` untouched; a post-transfer fault owns the mutated combat.
    pub fn enter_with_item_context(entry: CombatEntry,
        item_context: &mut Option<crate::items::CombatItemContext>)
        -> Result<Self, CombatEntryFailure> {
        if let Some(items) = item_context.as_ref() {
            crate::items::validate_combat_inventory(items).map_err(CombatEntryFailure::Rejected)?;
        }
        let setup = entry.setup;
        if entry.cards.is_empty() || setup.enemies.is_empty() || setup.enemies.len() > 9 {
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
        }
        if entry.opening_draw > 10 {
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::Unsupported("opening draw exceeds hand cap")));
        }
        validate_player(&setup.player, false).map_err(|_| CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction))?;
        if setup.player.hp == 0 || setup.player.block != 0 || !setup.player.powers.is_empty()
            || !valid_full_powers(&setup.player_powers)
            || setup.shuffle_rng.id().domain() != Domain::RunCombatShuffle
            || setup.shuffle_rng.id().scope().len() != 1 {
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
        }
        if setup.card_rng.id().domain() != Domain::RunCombatCardRandom
            || setup.card_rng.id().scope().len() != 1
            || setup.card_rng.id().scope()[0] != setup.shuffle_rng.id().scope()[0] {
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
        }
        let expected_bite_slots = setup.enemies.iter().enumerate().filter_map(|(slot, enemy)|
            matches!(enemy.enemy_id, crate::content::EnemyId::LouseRed | crate::content::EnemyId::LouseGreen)
                .then_some((slot as u32, super::FullIntentId::Bite))).collect::<BTreeSet<_>>();
        if setup.enemy_damage_bases.keys().copied().collect::<BTreeSet<_>>() != expected_bite_slots
            || setup.enemy_damage_bases.values().any(|base| !(5..=7).contains(base)) {
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
        }
        let mut ids = BTreeSet::new();
        let mut ai_streams = Vec::new();
        for enemy in &setup.enemies {
            if !ids.insert(enemy.instance_id.clone()) || enemy.hp <= 0 || enemy.hp > enemy.max_hp
                || enemy.max_hp <= 0 || enemy.block != 0 || !valid_full_powers(&enemy.powers)
                || enemy.ascension > 20
                || (enemy.ascension != 0 && !matches!(enemy.enemy_id,
                    crate::content::EnemyId::Cultist | crate::content::EnemyId::JawWorm
                        | crate::content::EnemyId::GremlinNob))
                || !enemy.profile.valid_for(enemy.enemy_id)
                || enemy.initial_memory.as_ref().is_some_and(|memory|!memory.valid_for(enemy.enemy_id)||memory.ascension()!=enemy.ascension)
                || enemy.ai_rng.id().domain() != Domain::RunCombatEnemyAi
                || enemy.ai_rng.id().scope().len() != 2
                || enemy.ai_rng.id().scope()[0] != setup.shuffle_rng.id().scope()[0]
                || ai_streams.contains(enemy.ai_rng.id()) {
                return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
            }
            ai_streams.push(enemy.ai_rng.id().clone());
        }
        if setup.enemies.iter().any(|e|e.enemy_id==crate::content::EnemyId::GremlinLeader)
            && (setup.enemies.len()>4||setup.enemies.iter().filter(|e|e.enemy_id==crate::content::EnemyId::GremlinLeader).count()!=1
                ||setup.enemies.iter().any(|e|e.enemy_id!=crate::content::EnemyId::GremlinLeader&&!super::enemies::act2::gremlin_leader::is_small(e.enemy_id))){
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
        }
        let first = setup.enemies.first().unwrap();
        let rng = super::model::CombatRng { contract_id: RNG_CONTRACT_ID.into(),
            shuffle: setup.shuffle_rng.clone(), enemy_ai: first.ai_rng.clone(), enemy_instance_id: first.instance_id.clone() };
        let mut combat = Self::build_full(setup, rng);
        super::enemies::act2::collector::validate_slots(&combat.state)
            .map_err(|_|CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction))?;
        super::enemies::act2::stasis::validate_custody(&combat.state)
            .map_err(|_|CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction))?;
        let prebattle = combat.state.enemies.iter().try_fold(Vec::new(), |mut work, actor| {
            work.extend(super::enemies::prebattle_work(actor).map_err(|_|
                CombatEntryFailure::Rejected(crate::run::RunError::Unsupported("invalid enemy prebattle setup")))?);
            Ok::<_, CombatEntryFailure>(work)
        })?;
        super::effects::refresh_enemy_values(&mut combat.state)
            .map_err(|_| CombatEntryFailure::Rejected(crate::run::RunError::Unsupported("invalid full combat damage setup")))?;
        let mut issued = BTreeSet::new();
        for live in entry.cards {
            if live.instance.0 == 0 || !issued.insert(live.instance) {
                return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
            }
            if let CombatCardRef::Permanent(instance) = &live.origin {
                if *instance != live.instance { return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction)); }
            }
            combat.state.card_ids_issued.insert(super::card_state::handle(live.instance));
            combat.state.piles.draw.push(OwnedCard::Full(live));
        }
        if !(1..=999).contains(&issued.len()) {
            return Err(CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction));
        }
        let (shuffled, next) = super::rng::shuffle(&combat.state.rng.shuffle, &combat.state.piles.draw)
            .map_err(|_| CombatEntryFailure::Rejected(crate::run::RunError::Unsupported("full combat shuffle failed")))?;
        combat.state.rng.shuffle = next;
        combat.state.piles.draw = shuffled;
        // Collar's temporary master-energy change belongs to beforeEnergyPrep,
        // not equip or a recurring turn-energy action. Reject overflow before transfer.
        let collar=item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::SlaversCollar)&&(items.battle.elite_trigger||items.battle.boss));
        let master=combat.state.player.max_energy.checked_add(i32::from(collar)).filter(|n|*n<=999)
            .ok_or(CombatEntryFailure::Rejected(crate::run::RunError::Unsupported("Collar master energy overflow")))?;
        combat.state.item_context=item_context.take();
        if let Some(items)=combat.state.item_context.as_mut(){items.collar_activated=collar;}
        combat.state.player.max_energy=master;
        let count = entry.opening_draw.min(combat.state.piles.draw.len());
        for _ in 0..count {
            let card = combat.state.piles.draw.pop().expect("count bounded by draw size");
            combat.state.public.revealed_order.push(card.handle());
            combat.state.piles.hand.push(card);
        }
        combat.state.player.energy = combat.state.player.max_energy;
        combat.state.pending.extend(prebattle);
        combat.state.pending.push_back(Effect { source: Source::None, target: None,
            kind: EffectKind::Hook(Hook::CombatStarted) });
        // The opening decision follows the first turn-start checkpoint. This
        // keeps prebattle arming and first-turn relic work on the same resolver.
        combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
            target: Some(Actor::Player), kind: EffectKind::Hook(Hook::TurnStarted) });
        let entry_transition = resolve(&mut combat.state);
        if entry_transition.status == TransitionStatus::Fault {
            let crate::run::RunError::Faulted(fault) = retained_fault(&combat.state) else {
                unreachable!("retained combat fault has typed RunFault")
            };
            combat.refresh_legacy_cache();
            return Err(CombatEntryFailure::Faulted { combat: Box::new(combat), fault });
        }
        if let Err(error) = validate_runtime(&combat.state) {
            super::model::mark_fault(&mut combat.state, super::FaultKind::Invariant, format!("{error:?}"));
            let crate::run::RunError::Faulted(fault) = retained_fault(&combat.state) else {
                unreachable!("post-mutation validation fault has typed RunFault")
            };
            combat.refresh_legacy_cache();
            return Err(CombatEntryFailure::Faulted { combat: Box::new(combat), fault });
        }
        combat.refresh_legacy_cache();
        Ok(combat)
    }

    /// Read a full-mode card/EndTurn menu at a ready decision without advancing
    /// state or RNG. Unsupported handlers are errors, not a silently reduced
    /// profile. Selection screens expose only the captured eligible picks.
    pub fn legal_actions_full(&self) -> Result<Vec<CombatAction>, crate::run::RunError> {
        if !self.state.full_runtime { return Err(crate::run::RunError::InvalidAction); }
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if self.state.phase == Phase::Selection {
            let selection = self.state.pending_selection.as_ref().ok_or(crate::run::RunError::InvalidAction)?;
            let mut actions = Vec::new();
            for candidate in &selection.candidates {
                let instance = candidate.as_str().parse().map_err(|_| crate::run::RunError::InvalidAction)?;
                if self.preflight_full_selection(instance).is_ok() { actions.push(CombatAction::SelectCard { instance }); }
            }
            return Ok(actions);
        }
        super::view::ready(&self.state).map_err(|_| crate::run::RunError::InvalidAction)?;
        if self.state.phase == Phase::Terminal { return Ok(vec![]); }
        let targets = std::iter::once(None).chain(self.state.enemies.iter().enumerate()
            .filter(|(_, enemy)| enemy.life == super::ActorLife::Active && enemy.hp > 0)
            .map(|(slot, _)| Some(TargetId(slot as u64 + 1)))).collect::<Vec<_>>();
        let mut actions = Vec::new();
        for owned in &self.state.piles.hand {
            let card = owned.full().ok_or(crate::run::RunError::Unsupported("legacy card in full combat menu"))?;
            for target in &targets {
                let action = CombatAction::PlayCard { instance: card.instance.0, target: *target };
                match super::cards::prepare(self, &action) {
                    Ok(_) => actions.push(action),
                    Err(crate::run::RunError::InvalidAction) => (),
                    Err(error) => return Err(error),
                }
            }
        }
        if let Some(items) = &self.state.item_context {
            for slot in 0..items.potions().len() {
                for target in &targets {
                    let owner = self.full_target(*target)?;
                    match crate::items::prepare_combat_potion(self, slot, owner, false) {
                        Ok(_) => actions.push(CombatAction::UsePotion { slot, target: *target }),
                        Err(crate::run::RunError::InvalidAction) => (),
                        Err(error) => return Err(error),
                    }
                }
            }
        }
        actions.push(CombatAction::EndTurn);
        Ok(actions)
    }

    /// Full-run action adapter. Target ids are stable 1-based actor slots for
    /// this encounter; dead actors retain their slot and cannot be selected.
    pub fn step_full(&mut self, action: &CombatAction) -> Result<(), crate::run::RunError> {
        if !self.state.full_runtime { return Err(crate::run::RunError::InvalidAction); }
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if let CombatAction::SelectCard { instance } = action {
            return self.select_full_card(*instance);
        }
        // Reject non-decision and terminal actions before an unsupported handler
        // can accept/fault them. This mirrors step_internal's phase boundary.
        if self.state.phase == Phase::Terminal || self.state.phase != Phase::Decision
            || super::view::ready(&self.state).is_err() {
            return Err(crate::run::RunError::InvalidAction);
        }
        if let CombatAction::UsePotion { slot, target } = action {
            let owner = self.full_target(*target)?;
            crate::items::prepare_combat_potion(self, *slot, owner.clone(), false)?;
            self.state.attempted_action = Some(action.clone());
            let result = crate::items::use_combat_potion(self, *slot, owner, false);
            if self.state.fault.is_none() { self.state.attempted_action = None; }
            return result;
        }
        if matches!(action, CombatAction::PlayCard { .. }) {
            match super::cards::prepare(self, action) {
                Ok(prepared) => {
                    let instance = prepared.instance;
                    let source = Source::Card(super::card_state::handle(instance));
                    self.state.resolved_effects = 0;
                    self.state.attempted_action = Some(action.clone());
                    self.state.phase = Phase::PlayerAction;
                    let mut card = self.state.piles.hand.remove(prepared.hand_index);
                    if let Some(full) = card.full_mut() { full.local.free_play = false; }
                    self.state.in_play = Some(card);
                    self.state.player.energy = prepared.remaining_energy;
                    self.state.pending.push_back(Effect { source: source.clone(),
                        target: prepared.target, kind: EffectKind::Hook(Hook::CardPlayed) });
                    self.state.pending.extend(prepared.effects);
                    self.state.pending.push_back(Effect { source, target: None, kind: EffectKind::FinishCard });
                    self.state.pending.push_back(Effect { source: Source::None,
                        target: None, kind: EffectKind::Decision });
                    let transition = resolve(&mut self.state);
                    self.refresh_legacy_cache();
                    if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
                    self.state.attempted_action = None;
                    return match transition.status {
                        TransitionStatus::Decision | TransitionStatus::Terminal
                        | TransitionStatus::Selection => Ok(()),
                        TransitionStatus::Fault => Err(crate::run::RunError::InvalidAction),
                    };
                }
                Err(crate::run::RunError::Unsupported(_)) => (),
                Err(error) => return Err(error),
            }
        }
        let old = match action {
            CombatAction::EndTurn => Action::EndTurn,
            CombatAction::SelectCard { .. } => unreachable!("selection is handled before combat-action translation"),
            CombatAction::UsePotion { .. } => unreachable!("potions are handled before legacy translation"),
            CombatAction::PlayCard { instance, target } => {
                let card = CardHandle::parse(&instance.to_string())
                    .map_err(|_| crate::run::RunError::InvalidAction)?;
                let owned = self.state.piles.hand.iter().find(|owned| owned.handle() == card)
                    .ok_or(crate::run::RunError::InvalidAction)?;
                let full = owned.full().ok_or(crate::run::RunError::InvalidAction)?;
                let target = match target {
                    None => None,
                    Some(TargetId(id)) if *id > 0 => self.state.enemies.get((*id - 1) as usize)
                        .filter(|e| e.life == super::ActorLife::Active && e.hp > 0)
                        .map(|e| e.instance_id.clone()),
                    Some(_) => None,
                };
                if matches!(action, CombatAction::PlayCard { target: Some(_), .. }) && target.is_none() {
                    return Err(crate::run::RunError::InvalidAction);
                }
                if super::card_state::legacy_kind(full.id).is_none() {
                    let card_name = full.id.as_str();
                    // This accepted action has its own dispatch budget, like the
                    // normal step_internal path, but only after every preflight.
                    self.state.resolved_effects = 0;
                    self.state.attempted_action = Some(action.clone());
                    super::model::mark_fault(&mut self.state, FaultKind::Unsupported,
                        format!("unhandled full-run card {card_name}"));
                    return Err(retained_fault(&self.state));
                }
                if target.is_none() && target_required(&self.state, &card) {
                    return Err(crate::run::RunError::InvalidAction);
                }
                Action::PlayCard { card_instance_id: card, target_id: target }
            }
        };
        self.state.attempted_action = Some(action.clone());
        let result = self.step_internal(&old);
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        self.state.attempted_action = None;
        result.map(|_| ()).map_err(|_| crate::run::RunError::InvalidAction)
    }

    fn full_target(&self, target: Option<TargetId>) -> Result<Option<EnemyHandle>, crate::run::RunError> {
        match target {
            None => Ok(None),
            Some(TargetId(id)) => id.checked_sub(1).and_then(|index| usize::try_from(index).ok())
                .and_then(|index| self.state.enemies.get(index))
                .filter(|enemy| enemy.life == ActorLife::Active && enemy.hp > 0)
                .map(|enemy| Some(enemy.instance_id.clone())).ok_or(crate::run::RunError::InvalidAction),
        }
    }

    fn preflight_full_selection(&self, instance: u64) -> Result<(), crate::run::RunError> {
        let Some(selection) = self.state.pending_selection.as_ref() else {
            return Err(crate::run::RunError::InvalidAction);
        };
        if self.state.phase != Phase::Selection { return Err(crate::run::RunError::InvalidAction); }
        let handle = CardHandle::parse(&instance.to_string()).map_err(|_| crate::run::RunError::InvalidAction)?;
        if !selection.candidates.contains(&handle) || selection.selected.contains(&handle)
            || selection.selected.len() >= selection.required { return Err(crate::run::RunError::InvalidAction); }
        let zone = if self.state.piles.hand.iter().any(|c| c.handle() == handle) { 0 }
            else if self.state.piles.draw.iter().any(|c| c.handle() == handle) { 1 }
            else if self.state.piles.discard.iter().any(|c| c.handle() == handle) { 2 }
            else if self.state.piles.exhaust.iter().any(|c| c.handle() == handle) { 3 }
            else { return Err(crate::run::RunError::InvalidAction); };
        if zone != 0 && selection.disposition == SelectionDisposition::Hand && self.state.piles.hand.len() >= 10 {
            return Err(crate::run::RunError::InvalidAction);
        }
        Ok(())
    }

    fn select_full_card(&mut self, instance: u64) -> Result<(), crate::run::RunError> {
        self.preflight_full_selection(instance)?;
        let handle = CardHandle::parse(&instance.to_string()).map_err(|_| crate::run::RunError::InvalidAction)?;
        self.state.pending_selection.as_mut().unwrap().selected.push(handle.clone());
        if self.state.pending_selection.as_ref().unwrap().selected.len()
            < self.state.pending_selection.as_ref().unwrap().required { return Ok(()); }
        let selection = self.state.pending_selection.take().unwrap();
        for selected in &selection.selected {
            let (card, source_zone) = take_owned_card(&mut self.state.piles, selected)
                .ok_or(crate::run::RunError::InvalidAction)?;
            if source_zone == 1 { self.state.public.revealed_order.push(selected.clone()); }
            match selection.disposition {
                SelectionDisposition::Hand => self.state.piles.hand.push(card),
                SelectionDisposition::Discard => self.state.piles.discard.push(card),
                SelectionDisposition::Exhaust => super::piles::exhaust_owned_card(&mut self.state, card)
                    .map_err(|_| crate::run::RunError::InvalidAction)?,
            }
        }
        self.state.fault_selection = Some(selection.clone());
        self.state.attempted_action = Some(CombatAction::SelectCard { instance });
        let transition = resolve(&mut self.state);
        self.refresh_legacy_cache();
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        self.state.fault_selection = None;
        self.state.attempted_action = None;
        match transition.status {
            TransitionStatus::Decision | TransitionStatus::Terminal => Ok(()),
            TransitionStatus::Fault => Err(crate::run::RunError::InvalidAction),
            TransitionStatus::Selection => Err(crate::run::RunError::InvalidAction),
        }
    }

    /// Player-entitled full projection, with target ids added without modifying
    /// the legacy public-view schema.
    pub fn public_full(&self) -> Result<CombatPublic, crate::run::RunError> {
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if !self.state.full_runtime { return Err(crate::run::RunError::InvalidAction); }
        if self.state.phase == Phase::Selection {
            validate_runtime(&self.state).map_err(|_| crate::run::RunError::InvalidAction)?;
        } else {
            super::view::ready(&self.state).map_err(|_| crate::run::RunError::InvalidAction)?;
        }
        let cards = |pile: &[OwnedCard]| -> Result<Vec<LiveCardView>, crate::run::RunError> {
            pile.iter().map(|card| card.full().map(|live| LiveCardView { instance: live.instance,
                id: live.id, origin: live.origin.clone(), upgrades: live.upgrades, effective_cost:super::cards::visible_cost(&self.state,live) })
                .ok_or(crate::run::RunError::InvalidAction)).collect()
        };
        let selection = if let Some(pending) = &self.state.pending_selection {
            let candidates = pending.candidates.iter().map(|id| self.state.piles.all(self.state.in_play.as_ref())
                .find(|card| card.handle() == *id).and_then(OwnedCard::full)
                .map(|live| LiveCardView { instance: live.instance, id: live.id,
                    origin: live.origin.clone(), upgrades: live.upgrades, effective_cost:super::cards::visible_cost(&self.state,live) }).ok_or(crate::run::RunError::InvalidAction))
                .collect::<Result<Vec<_>, _>>()?;
            let selected = pending.selected.iter().map(|id| id.as_str().parse::<u64>()
                .map(CardInstanceId).map_err(|_| crate::run::RunError::InvalidAction)).collect::<Result<Vec<_>, _>>()?;
            Some(SelectionView { candidates, required: pending.required, selected, disposition: pending.disposition })
        } else { None };
        Ok(CombatPublic { content_id: self.state.content_id.clone(), turn: self.state.turn,
            stasis:self.state.stasis.iter().map(|(orb,card)|card.full().map(|live|super::view::StasisCardView{orb:orb.clone(),card:LiveCardView{instance:live.instance,id:live.id,origin:live.origin.clone(),upgrades:live.upgrades,effective_cost:super::cards::visible_cost(&self.state,live)}}).ok_or(crate::run::RunError::InvalidAction)).collect::<Result<Vec<_>,_>>()?,
            player: super::view::full_player(&self.state), enemies: super::view::full_enemies(&self.state),
            inventory: self.state.item_context.as_ref().map(|items| super::view::CombatInventoryView {
                relic_counters:crate::items::visible_counters(items.counters()),relics: items.relics().to_vec(), potions: items.potions().to_vec() }),
            character: super::view::CharacterCombatView { stance: self.state.character.stance,
                orbs: self.state.character.orb_slots.clone(), focus: self.state.character.focus },
            hand: cards(&self.state.piles.hand)?, draw_count: self.state.piles.draw.len(),
            discard: cards(&self.state.piles.discard)?, exhaust: cards(&self.state.piles.exhaust)?,
            outcome: self.state.outcome, hp_before_healing: self.state.hp_before_healing,
            hp_after_cleanup: self.state.hp_after_cleanup,
            targets: self.state.enemies.iter().enumerate().map(|(i, e)|
                (TargetId((i + 1) as u64), e.life==ActorLife::Active&&e.hp > 0)).collect(), selection })
    }

    /// Export the settled full-combat state for a native run owner. This does
    /// not apply rewards or mutate/consume the combat checkpoint.
    pub fn exit(&self) -> Result<CombatExit, crate::run::RunError> {
        if self.state.fault.is_some() { return Err(retained_fault(&self.state)); }
        if !self.state.full_runtime { return Err(crate::run::RunError::InvalidAction); }
        let outcome = self.state.outcome.ok_or(crate::run::RunError::InvalidAction)?;
        let export = |pile: &[OwnedCard]| -> Result<Vec<super::card_state::LiveCard>, crate::run::RunError> {
            pile.iter().map(|card| card.full().cloned().ok_or(crate::run::RunError::Unsupported(
                "legacy card in full combat exit"))).collect()
        };
        let in_play = self.state.in_play.as_ref().map(|card| card.full().cloned()
            .ok_or(crate::run::RunError::Unsupported("legacy in-play card in full combat exit")))
            .transpose()?;
        Ok(CombatExit { hp: self.state.player.hp, max_hp: self.state.player.max_hp,
            stasis:self.state.stasis.values().map(|card|card.full().cloned().ok_or(crate::run::RunError::InvalidAction)).collect::<Result<Vec<_>,_>>()?,
            block: self.state.player.block, energy: self.state.player.energy, outcome,
            item_context: self.state.item_context.clone(),
            hand: export(&self.state.piles.hand)?, draw: export(&self.state.piles.draw)?,
            discard: export(&self.state.piles.discard)?, exhaust: export(&self.state.piles.exhaust)?, in_play })
    }

    pub fn new(scenario: &Scenario) -> Result<Self, InputError> {
        validate_scenario(scenario)?;
        Ok(Self::build(scenario, false))
    }

    fn build(scenario: &Scenario, full_runtime: bool) -> Self {
        // Only borrow until validation succeeds; no stream is sampled or normalized.
        let enemies = scenario.enemies.iter().map(|e| {
            let damage_entries = base_damage(e.enemy_id);
            let intent_damage = damage_entries.get(&e.intent_id).map(|d| d.output);
            EnemyState { instance_id: e.instance_id.clone(), enemy_id: ActorEnemyId::Legacy(e.enemy_id),
                hp: e.hp, max_hp: e.max_hp, block: e.block, powers: e.powers.clone(),
                full_powers: vec![], intent_id: ActorIntentId::Legacy(e.intent_id),
                move_history: e.move_history.iter().copied().map(ActorIntentId::Legacy).collect(),
                selected_move: ActorIntentId::Legacy(e.intent_id), damage_entries,
                full_damage_entries: BTreeMap::new(), full_ai_rng: None, full_memory: None, intent_damage,
                life: super::ActorLife::Active }
        }).collect();
        let piles = OwnedPiles::from_legacy(&scenario.piles);
        let issued = owned_cards(&piles, None).map(OwnedCard::handle).collect();
        Self { legacy_cache: scenario.piles.clone(), state: State { content_id: CONTENT_ID.into(), phase: Phase::Decision, turn: 1,
            player: scenario.player.clone(), enemies, piles,
            card_ids_issued: issued,
            rng: scenario.rng.clone(), pending: VecDeque::new(), current: None, in_play: None,
            resolved_effects: 0, public: PublicKnowledge { revealed_order: vec![] },
            outcome: None, fault: None, hp_before_healing: None, hp_after_cleanup: None, full_runtime,
            pending_selection: None, hook_events: vec![], attempted_action: None,
             fault_selection: None, player_full_powers: vec![], item_context: None,
             character: super::characters::CharacterState::for_character(crate::content::Character::Ironclad),
             card_rng: None, replacement_progress: None, actor_slots:BTreeMap::new(),spawn_ordinals:BTreeMap::new(),spawn_progress:None,stasis:BTreeMap::new() } }
    }

    fn build_full(setup: super::model::FullCombatSetup, rng: super::model::CombatRng) -> Self {
        let damage_bases = setup.enemy_damage_bases;
        let enemies:Vec<_> = setup.enemies.into_iter().enumerate().map(|(slot, e)| {
            let mut full_damage_entries = supported_full_damage(e.enemy_id, e.ascension);
            if matches!(e.enemy_id, crate::content::EnemyId::LouseRed | crate::content::EnemyId::LouseGreen) {
                if let Some(base) = damage_bases.get(&(slot as u32, super::FullIntentId::Bite)) {
                    full_damage_entries.insert(super::FullIntentId::Bite, DamageEntry { base: *base, output: *base });
                }
            }
            let intent_damage = full_damage_entries.get(&e.intent).map(|d| d.output);
            EnemyState { instance_id: e.instance_id, enemy_id: ActorEnemyId::Full(e.enemy_id),
                hp: e.hp, max_hp: e.max_hp, block: e.block, powers: vec![], full_powers: e.powers,
                intent_id: ActorIntentId::Full(e.intent.clone()),
                move_history: vec![ActorIntentId::Full(e.intent.clone())], selected_move: ActorIntentId::Full(e.intent),
                damage_entries: BTreeMap::new(), full_damage_entries,
                full_ai_rng: Some(e.ai_rng),
                full_memory: e.initial_memory.or_else(||super::enemies::EnemyMemory::for_profile(e.enemy_id, e.ascension, e.profile).ok()), intent_damage,
                life: super::ActorLife::Active }
        }).collect();
        let actor_slots=super::enemies::act2::spawn::initial_slots(&enemies);
        Self { legacy_cache: Piles { hand: vec![], draw: vec![], discard: vec![], exhaust: vec![] },
            state: State { content_id: CONTENT_ID.into(), phase: Phase::Decision, turn: 1,
                player: setup.player, enemies, piles: OwnedPiles::default(), card_ids_issued: BTreeSet::new(),
                rng, pending: VecDeque::new(), current: None, in_play: None, resolved_effects: 0,
                public: PublicKnowledge { revealed_order: vec![] }, outcome: None, fault: None,
                hp_before_healing: None, hp_after_cleanup: None, full_runtime: true, pending_selection: None,
                hook_events: vec![], attempted_action: None, fault_selection: None,
                player_full_powers: setup.player_powers, item_context: None,
                character: super::characters::CharacterState::for_character(setup.character),
                card_rng: Some(setup.card_rng), replacement_progress: None,actor_slots,spawn_ordinals:BTreeMap::new(),spawn_progress:None,stasis:BTreeMap::new() } }
    }

    pub(super) fn refresh_legacy_cache(&mut self) {
        if let Some(piles) = self.state.piles.legacy_projection() { self.legacy_cache = piles; }
    }
}

fn target_required(state: &State, card: &CardHandle) -> bool {
    state.piles.hand.iter().find(|c| c.handle() == *card)
        .and_then(|c| c.legacy().map(|legacy| legacy.card_id)
            .or_else(|| c.full().and_then(|full| super::card_state::legacy_kind(full.id))))
        .map_or(false, |kind| matches!(kind, CardKind::Strike | CardKind::Bash))
}

fn take_owned_card(piles: &mut OwnedPiles, handle: &CardHandle) -> Option<(OwnedCard, u8)> {
    for (zone, pile) in [&mut piles.hand, &mut piles.draw, &mut piles.discard, &mut piles.exhaust].into_iter().enumerate() {
        if let Some(index) = pile.iter().position(|card| card.handle() == *handle) {
            return Some((pile.remove(index), zone as u8));
        }
    }
    None
}

fn retained_fault(state: &State) -> crate::run::RunError {
    let native = super::model::CombatFault {
        record: state.fault.as_ref().expect("retained fault requested only for faulted state").clone(),
        current: state.current.clone(), pending: state.pending.clone(),
        attempted_action: state.attempted_action.clone(),
        selection: state.pending_selection.clone().or_else(|| state.fault_selection.clone()),
        spawn_progress:state.spawn_progress.as_ref().map(|p|super::EnemySpawnDiagnostics{request:p.request.clone(),ordinal:p.ordinal,prepared_child:p.prepared_child.as_ref().map(|e|e.instance_id.clone()),constructor_cursor:p.constructor_rng.word_index(),latest_child_cursor:p.latest_child_rng.word_index()}),
        replacement_progress: state.replacement_progress.as_ref().map(|progress|
            super::model::EnemyReplacementDiagnostics {
                parent: progress.parent.clone(), next_child: progress.next_child,
                prepared_children: progress.prepared.iter()
                    .map(|enemy| enemy.instance_id.clone()).collect(),
                latest_child_cursor: progress.latest_child_rng.as_ref()
                    .map(super::rng::StreamState::word_index),
            }),
    };
    crate::run::RunError::Faulted(crate::run::RunFault {
        kind: crate::run::RunFaultKind::Combat(Box::new(native)),
        checkpoint: "combat.runtime", committed_events: Vec::new(),
    })
}

fn validate_powers(powers: &[PowerEntry], enemy: Option<EnemyKind>, state: &State, actor: &Actor) -> Result<(), InputError> {
    let mut seen = BTreeSet::new();
    for entry in powers {
        require(seen.insert(entry.id), "duplicate power")?;
        let p = &entry.state;
        // A full reduction leaves the object at zero until its front-queued removal.
        let removing = removing_power(state, actor, entry.id.as_str());
        let signed = state.full_runtime && entry.id == PowerId::Strength;
        require((if signed { p.amount != 0 && p.amount >= -999 } else { p.amount > 0 })
            || (p.amount == 0 && removing), "power amount or pending removal")?;
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
    require(match actor { Actor::Player => true, Actor::Enemy(id) => state.enemies.iter().any(|e| &e.instance_id == id) }, "effect actor owner")
}

pub(super) fn validate_effect(effect: &Effect, state: &State) -> Result<(), InputError> {
    require(if state.full_runtime { !state.enemies.is_empty() } else { state.enemies.len() == 1 }, "effect enemy count")?;
    match &effect.source {
        Source::None => (),
        Source::Card(id) => require(state.card_ids_issued.contains(id), "effect card owner")?,
        Source::Actor(actor) => validate_actor(actor, state)?,
    }
    if let Some(actor) = &effect.target { validate_actor(actor, state)?; }
    if matches!(&effect.kind, EffectKind::Damage(_)) {
        if let Some(Actor::Enemy(target)) = &effect.target {
            let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == target)
                .ok_or_else(|| InputError::Invariant("damage target missing".into()))?;
            require(enemy.life == super::ActorLife::Active && enemy.hp > 0,
                "damage target must be active")?;
        }
    }
    if matches!(&effect.kind, EffectKind::Damage(_) | EffectKind::GainBlock(_)
        | EffectKind::ApplyPower { .. } | EffectKind::ReducePower { .. } | EffectKind::RemovePower(_)
        | EffectKind::ApplyFullPower { .. } | EffectKind::ReduceFullPower { .. } | EffectKind::RemoveFullPower(_)) {
        require(effect.target.is_some(), "targeted payload requires an actor")?;
    }
    if matches!(&effect.kind, EffectKind::ApplyPower { id: PowerId::Ritual, .. }
        | EffectKind::ReducePower { id: PowerId::Ritual, .. } | EffectKind::RemovePower(PowerId::Ritual)) {
        let cultist_target = match &effect.target {
            Some(Actor::Enemy(id)) => state.enemies.iter().any(|e| &e.instance_id == id && e.enemy_id == EnemyKind::Cultist),
            _ => false,
        };
        require(cultist_target, "Ritual payload owner")?;
    }
    match &effect.kind {
        EffectKind::Damage(DamagePayload::Fixed(n)) => {
            let maximum = if state.full_runtime { 30_012 } else { 1_515 };
            require((0..=maximum).contains(n), "damage payload bounds")?;
        }
        EffectKind::Damage(DamagePayload::Orb(n)) => require((0..=1515).contains(n), "orb payload bounds")?,
        EffectKind::Damage(DamagePayload::HpLoss(n)) => require(state.full_runtime && (0..=999).contains(n), "HP_LOSS mode/bounds")?,
        EffectKind::Damage(DamagePayload::Thorns(n)) => {
            require(state.full_runtime && (0..=1515).contains(n), "THORNS payload mode/bounds")?;
            let burn = match &effect.source {
                Source::Card(owner) => state.piles.hand.iter().any(|card| card.handle()==*owner
                    && card.full().is_some_and(|c| c.id==crate::content::CardId::Burn)),
                _=>false,
            };
            require((burn && effect.target==Some(Actor::Player))
                || (matches!(&effect.source,Source::Actor(Actor::Enemy(owner)) if state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Guardian)&&e.full_powers.iter().any(|p|p.id.0=="sharp_hide")))&&effect.target==Some(Actor::Player))
                || (effect.source==Source::Actor(Actor::Player) && matches!(&effect.target,Some(Actor::Enemy(_)))),
                "THORNS source/target must be player retaliation, Sharp Hide or held Burn")?;
        }
        EffectKind::Character(_) => require(matches!(effect.target.as_ref(), Some(Actor::Player)), "character effect target")?,
        EffectKind::GuardianChange(_) => {
            require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Guardian))),"Guardian change source/target")?;
        },
        EffectKind::HexaghostBurn(_) => {
            require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Hexaghost))),"Hexaghost gate source/target")?;
        },
        EffectKind::GroundByrd=>{
            require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Byrd))),"Byrd ground source/target")?;
        },
        EffectKind::CardContinuation(continuation) => {
            let owner_matches = matches!((&effect.source, &state.in_play),
                (Source::Card(owner), Some(card)) if owner == &card.handle());
            let expected_card = match continuation {
                super::cards::DefectContinuation::GoForTheEyesWeak { .. } => crate::content::CardId::GoForTheEyes,
                super::cards::DefectContinuation::BarrageStart { .. }
                | super::cards::DefectContinuation::BarrageNext { .. } => crate::content::CardId::Barrage,
                super::cards::DefectContinuation::CompileDriverDraw => crate::content::CardId::CompileDriver,
            };
            let card_matches = state.in_play.as_ref().and_then(|card| card.full())
                .is_some_and(|card| card.id == expected_card);
            require(state.full_runtime && owner_matches && card_matches,
                "card continuation requires matching in-play card/source")?;
            match continuation {
                super::cards::DefectContinuation::GoForTheEyesWeak { amount } => {
                    let expected = state.in_play.as_ref().and_then(|card| card.full())
                        .map(|card| if card.upgrades > 0 { 2 } else { 1 });
                    require(expected == Some(*amount)
                        && matches!(effect.target.as_ref(), Some(Actor::Enemy(id))
                            if state.enemies.iter().any(|enemy| &enemy.instance_id == id)),
                        "Go for the Eyes continuation amount/target")?;
                }
                super::cards::DefectContinuation::BarrageStart { damage } => {
                    require((0..=30_012).contains(damage)
                        && matches!(effect.target.as_ref(), Some(Actor::Enemy(id))
                            if state.enemies.iter().any(|enemy| &enemy.instance_id == id)),
                        "Barrage start damage/target")?;
                }
                super::cards::DefectContinuation::BarrageNext { damage, remaining } => {
                    require((0..=30_012).contains(damage) && (1..=9).contains(remaining)
                        && matches!(effect.target.as_ref(), Some(Actor::Enemy(id))
                            if state.enemies.iter().any(|enemy| &enemy.instance_id == id)),
                        "Barrage continuation damage/count/target")?;
                }
                super::cards::DefectContinuation::CompileDriverDraw =>
                    require(effect.target.is_none(), "Compile Driver continuation target")?,
            }
        }
        EffectKind::Damage(DamagePayload::EnemyMove(id)) => {
            let owner = match &effect.source { Source::Actor(Actor::Enemy(id)) => id, _ => return Err(InputError::Unsupported("enemy move without enemy source".into())) };
            let enemy = state.enemies.iter().find(|e| &e.instance_id == owner)
                .ok_or_else(|| InputError::Invariant("enemy move owner missing".into()))?;
            if !enemy.damage_entries.contains_key(id) {
                return Err(InputError::Unsupported("incompatible enemy damage move".into()));
            }
            require(effect.target == Some(Actor::Player), "enemy damage source/target")?;
        }
        EffectKind::ReplaceEnemy { parent, children } => {
            require(state.full_runtime && !children.is_empty(), "native replacement mode/children")?;
            let owner = match &effect.source {
                Source::Actor(Actor::Enemy(source)) if source == parent => source,
                _ => return Err(InputError::Unsupported("replacement source must be its parent actor".into())),
            };
            let actor = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
                .ok_or_else(|| InputError::Invariant("replacement parent missing".into()))?;
            require(effect.target.is_none(), "replacement has no target")?;
            require(actor.life == super::ActorLife::Active && actor.hp > 0,
                "replacement parent must be active")?;
            let split_latched = matches!((&actor.enemy_id, &actor.intent_id, &actor.full_memory),
                (ActorEnemyId::Full(crate::content::EnemyId::AcidSlimeL),
                    ActorIntentId::Full(super::FullIntentId::AcidSplit),
                    Some(super::enemies::EnemyMemory::AcidSlime { split_triggered: true }))
                | (ActorEnemyId::Full(crate::content::EnemyId::SpikeSlimeL),
                    ActorIntentId::Full(super::FullIntentId::SpikeSplit),
                    Some(super::enemies::EnemyMemory::SpikeSlime { split_triggered: true }))
                | (ActorEnemyId::Full(crate::content::EnemyId::SlimeBoss),
                    ActorIntentId::Full(super::FullIntentId::SlimeBossSplit),
                    Some(super::enemies::EnemyMemory::SlimeBoss { split_triggered: true })));
            require(split_latched, "replacement requires a selected Split latch")?;
            let expected = match (&actor.enemy_id, &actor.intent_id) {
                (ActorEnemyId::Full(crate::content::EnemyId::AcidSlimeL), ActorIntentId::Full(super::FullIntentId::AcidSplit)) => vec![crate::content::EnemyId::AcidSlimeM; 2],
                (ActorEnemyId::Full(crate::content::EnemyId::SpikeSlimeL), ActorIntentId::Full(super::FullIntentId::SpikeSplit)) => vec![crate::content::EnemyId::SpikeSlimeM; 2],
                (ActorEnemyId::Full(crate::content::EnemyId::SlimeBoss), ActorIntentId::Full(super::FullIntentId::SlimeBossSplit)) => vec![
                    crate::content::EnemyId::SpikeSlimeL, crate::content::EnemyId::AcidSlimeL],
                _ => return Err(InputError::Unsupported("replacement owner is not a qualified splitting Slime".into())),
            };
            require(children.as_slice() == expected.as_slice(), "replacement child identity/owner")?;
            if let Some(progress) = &state.replacement_progress {
                require(progress.parent == *parent && progress.children == *children
                    && progress.inherited_hp > 0 && progress.next_child == progress.prepared.len()
                    && progress.next_child <= progress.children.len(),
                    "replacement continuation matches current request")?;
            }
        }
        EffectKind::CollectorSuicide|EffectKind::AutomatonSuicide=>{
            let species=if effect.kind==EffectKind::CollectorSuicide{crate::content::EnemyId::Collector}else{crate::content::EnemyId::BronzeAutomaton};
            require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source!=target&&state.enemies.iter().any(|e|e.instance_id==*source&&e.enemy_id==ActorEnemyId::Full(species)&&e.hp==0)),"qualified boss death suicide source/target")?;
        },
        EffectKind::CaptureStasis{orb}=>require(state.full_runtime&&!state.stasis.contains_key(orb)&&effect.source==Source::Actor(Actor::Enemy(orb.clone()))&&effect.target==Some(Actor::Enemy(orb.clone()))&&state.enemies.iter().any(|e|e.instance_id==*orb&&e.hp>0&&e.life==ActorLife::Active&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::BronzeOrb)&&e.intent_id.as_str()=="orb_stasis"),"Stasis capture producer/custody")?,
        EffectKind::ReleaseStasis{orb}=>require(state.full_runtime&&state.stasis.contains_key(orb)&&effect.source==Source::Actor(Actor::Enemy(orb.clone()))&&effect.target==Some(Actor::Enemy(orb.clone()))&&state.enemies.iter().any(|e|e.instance_id==*orb&&e.hp==0&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::BronzeOrb)),"Stasis return death/custody")?,
        EffectKind::ChampCleanse=>require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Champ)&&e.life==ActorLife::Active&&e.hp>0)),"Champ cleanse source/target")?,
        EffectKind::BookStab{remaining}=>require(state.full_runtime&&(1..=i32::MAX as u32).contains(remaining)&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::BookOfStabbing)&&e.life==ActorLife::Active&&e.hp>0)),"Book stab producer/count")?,
        EffectKind::SpawnEnemy(request)=>{
            require(effect.source==Source::Actor(Actor::Enemy(request.summoner.clone()))&&effect.target.is_none()&&state.spawn_progress.is_none(),"summon request custody/source")?;
            super::enemies::act2::spawn::validate_request(state,request)?;
        },
        EffectKind::CompleteSpawn=>{
            let progress=state.spawn_progress.as_ref().ok_or_else(||InputError::Invariant("summon continuation custody absent".into()))?;
            require(effect.source==Source::Actor(Actor::Enemy(progress.request.summoner.clone()))&&effect.target==Some(Actor::Enemy(progress.request.summoner.clone())),"summon continuation source/target")?;
            super::enemies::act2::spawn::validate_request(state,&progress.request)?;
        },
        EffectKind::RallyGremlins{remaining}=>require(state.full_runtime&&(1..=2).contains(remaining)&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::GremlinLeader)&&e.life==super::ActorLife::Active&&e.hp>0)),"Rally producer/count")?,
        EffectKind::ProtectGremlin=>require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::GremlinShield)&&e.life==super::ActorLife::Active&&e.hp>0)),"Shield Protect producer")?,
        EffectKind::SetEnemyMove{intent}=>require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source==target&&state.enemies.iter().any(|e|e.instance_id==*target&&match (&e.enemy_id,intent.as_str()){
            (ActorEnemyId::Full(crate::content::EnemyId::GremlinMad),"gremlin_scratch")|(ActorEnemyId::Full(crate::content::EnemyId::GremlinSneaky),"gremlin_puncture")|(ActorEnemyId::Full(crate::content::EnemyId::GremlinShield),"gremlin_protect"|"gremlin_bash")|(ActorEnemyId::Full(crate::content::EnemyId::GremlinWizard),"gremlin_charge"|"gremlin_blast")|(ActorEnemyId::Full(crate::content::EnemyId::TorchHead),"torch_attack")=>true,_=>false})),"deterministic actor SetMove producer/intent")?,
        EffectKind::EscapeEnemy=>require(state.full_runtime&&matches!((&effect.source,&effect.target),(Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target))) if source!=target&&state.enemies.iter().any(|e|e.instance_id==*source&&e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::GremlinLeader)&&e.hp==0)),"Leader death escape source")?,
        EffectKind::Damage(DamagePayload::FullEnemyMove(id)) => {
            let owner = match &effect.source { Source::Actor(Actor::Enemy(id)) => id,
                _ => return Err(InputError::Unsupported("full enemy move without typed enemy source".into())) };
            let enemy = state.enemies.iter().find(|e| &e.instance_id == owner)
                .ok_or_else(|| InputError::Invariant("full enemy move owner missing".into()))?;
            if enemy.life != super::ActorLife::Active || enemy.hp <= 0
                || !matches!(&enemy.enemy_id, ActorEnemyId::Full(_))
                || !enemy.full_damage_entries.contains_key(id) {
                return Err(InputError::Unsupported("unhandled full enemy damage intent".into()));
            }
            require(effect.target == Some(Actor::Player), "full enemy damage source/target")?;
        }
        EffectKind::GainBlock(n) => require(*n >= 0 && (state.full_runtime || *n <= 1007), "block payload bounds")?,
        EffectKind::Heal(n) => require((0..=999).contains(n)
            && (effect.target == Some(Actor::Player) || (state.full_runtime&&matches!(effect.target,Some(Actor::Enemy(_))))), "healing bounds/target")?,
        EffectKind::NeowsLament=>require(state.full_runtime&&effect.source==Source::None&&effect.target==Some(Actor::Player),"Lament mode/source/target")?,
        EffectKind::IncreaseMaxHpThenHeal(n) => require(state.full_runtime && (1..=10).contains(n)
            && effect.source == Source::Actor(Actor::Player)
            && effect.target == Some(Actor::Player)
            && state.player.max_hp.checked_add(*n).is_some_and(|max| max <= 999),
            "native max-HP increase/queued heal mode/source/target/bounds")?,
        EffectKind::Draw(n) => require((0..=10).contains(n), "draw payload bounds")?,
        EffectKind::DrawOne=>require(state.full_runtime&&state.piles.hand.len()<10&&!state.piles.draw.is_empty(),"native draw-one membership/cap")?,
        EffectKind::AfterCardDrawn{card}=>require(state.full_runtime&&effect.target==Some(Actor::Player)&&state.piles.hand.iter().any(|c|c.handle()==*card),"native on-draw identity/target")?,
        EffectKind::GainEnergy(n) => {
            require(state.full_runtime && (1..=4).contains(n)
                && effect.target == Some(Actor::Player), "native energy payload bounds/target")?;
            if i64::from(state.player.energy) + i64::from(*n) > 999 {
                return Err(InputError::Unsupported("player energy overflow".into()));
            }
        }
        EffectKind::CreateCard { id, destination: _, upgrades } => {
            let supported = matches!((*id, *upgrades),
                (crate::content::CardId::Miracle, 0 | 1)
                    | (crate::content::CardId::Slimed, 0)
                    | (crate::content::CardId::Dazed, 0)
                    | (crate::content::CardId::Wound, 0)
                    | (crate::content::CardId::Burn, 0 | 1)
                    | (crate::content::CardId::Shiv, 0));
            require(state.full_runtime && supported && state.card_ids_issued.len() < 999
                && effect.target == Some(Actor::Player),
                "native generated-card support/count/target")?;
        }
        EffectKind::ExhaustHandCard { instance } => {
            require(state.full_runtime && effect.target.is_none()
                && state.piles.hand.iter().any(|card| card.full().is_some_and(|c| c.instance == *instance)),
                "forced exhaustion hand owner/mode")?;
            require(matches!((&effect.source, &state.in_play), (Source::Card(owner), Some(card)) if *owner == card.handle()),
                "forced exhaustion in-play parent")?;
        }
        EffectKind::CopyCard { source, destination } => {
            require(state.full_runtime
                && *destination == super::command::CardDestination::Discard
                && effect.target.is_none() && state.card_ids_issued.len() < 999,
                "native copy mode/destination/count")?;
            require(matches!((&effect.source, &state.in_play),
                (Source::Card(owner), Some(card)) if owner == source && owner == &card.handle()),
                "copy source must be the current in-play card")?;
        }
        EffectKind::ApplyPower { id, amount } | EffectKind::ReducePower { id, amount } => {
            let signed_apply = state.full_runtime && *id == PowerId::Strength
                && matches!(effect.kind, EffectKind::ApplyPower { .. });
            require(if signed_apply { *amount != 0 && *amount >= -1002 } else { *amount > 0 }, "power payload bounds")?;
            if *id == PowerId::Strength { require(*amount <= 1002, "Strength payload bounds")?; }
            if *id == PowerId::Ritual {
                let cultist_target = match &effect.target {
                    Some(Actor::Enemy(id)) => state.enemies.iter().any(|e| &e.instance_id == id && e.enemy_id == EnemyKind::Cultist),
                    _ => false,
                };
                require(*amount == 3 && cultist_target, "Ritual payload")?;
            }
        }
        EffectKind::ApplyFullPower { id, amount } => require(state.full_runtime &&
            (if id.0=="mode_shift"{*amount>0}
                else if matches!(id.0, "strength" | "dexterity") { (-1002..=1002).contains(amount) && *amount != 0 }
                else { (1..=1002).contains(amount) }), "full power payload bounds")?,
        EffectKind::ReduceFullPower { amount, .. } =>
            require(state.full_runtime && (1..=1002).contains(amount), "full power reduction bounds")?,
        EffectKind::FinishCard => require(match (&effect.source, &state.in_play) {
            (Source::Card(id), Some(card)) => id == &card.handle(),
            _ => false,
        }, "card continuation requires matching in-play source")?,
        EffectKind::SelectCard { candidates, .. } | EffectKind::SelectCards { candidates, .. } => {
            require(match (&effect.source, &state.in_play) {
                (Source::Card(parent), Some(card)) => parent == &card.handle(),
                _ => false,
            }, "selection requires current in-play parent")?;
            let mut unique = BTreeSet::new();
            for candidate in candidates {
                require(unique.insert(candidate), "duplicate selection candidate")?;
                require(owned_cards(&state.piles, state.in_play.as_ref()).any(|card| &card.handle() == candidate),
                    "selection candidate ownership")?;
            }
        }
        EffectKind::SelectHandCardsAfterDraw { required, .. } => {
            require(*required > 0, "captured selection count")?;
            require(match (&effect.source, &state.in_play) {
                (Source::Card(parent), Some(card)) => parent == &card.handle(),
                _ => false,
            }, "captured selection requires current in-play parent")?;
        }
        EffectKind::Terminal(outcome) => require(match outcome {
            Outcome::Victory => super::enemies::act2::combat_victory_ready(state) && state.player.hp > 0,
            Outcome::Death => state.player.hp == 0 && state.enemies.iter().any(|e|
                e.life == super::ActorLife::Active && e.hp > 0),
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
    require(state.attempted_action.is_none() && state.fault_selection.is_none(),
        "fault diagnostic capture without retained fault")?;
    require(state.current.is_none(), "interrupted current payload cannot resume")?;
    validate_player(&state.player, state.full_runtime)?;
    require(state.character.orb_slots.len() <= 10 && (-999..=999).contains(&state.character.focus),
        "character runtime bounds")?;
    require(if state.full_runtime { state.card_rng.as_ref().is_some_and(|rng|
        rng.id().domain() == Domain::RunCombatCardRandom && rng.id().scope().len() == 1) }
        else { state.card_rng.is_none() }, "card random stream mode/domain")?;
    validate_powers(&state.player.powers, None, state, &Actor::Player)?;
    require(valid_runtime_full_powers(&state.player_full_powers, state, &Actor::Player), "full player powers")?;
    super::enemies::act2::collector::validate_slots(state)?;
    super::enemies::act2::stasis::validate_custody(state)?;
    require(if state.full_runtime {
        !state.enemies.is_empty() && u64::try_from(state.enemies.len()).is_ok()
            && state.enemies.iter().filter(|enemy|
                enemy.life == super::ActorLife::Active && enemy.hp > 0).count() <= 9
            && (state.phase == Phase::Terminal || state.enemies.iter().any(|enemy|
                enemy.life == super::ActorLife::Active && enemy.hp > 0))
    } else { state.enemies.len() == 1 }, "runtime active/stored enemy count")?;
    if state.full_runtime {
        let mut handles=BTreeSet::new();let mut live_slots=BTreeSet::new();
        require(state.actor_slots.len()==state.enemies.len(),"logical slot membership")?;
        for enemy in &state.enemies {
            let slot=state.actor_slots.get(&enemy.instance_id).ok_or_else(||InputError::Invariant("actor logical slot missing".into()))?;
            require(handles.insert(&enemy.instance_id)&&*slot<9,"unique stored actor/logical slot bounds")?;
            if enemy.life==super::ActorLife::Active&&enemy.hp>0{require(live_slots.insert(slot),"unique living logical slots")?;}
        }
        require(state.spawn_progress.is_none(),"unfinished summon cannot reach a ready boundary")?;
    }
    for e in &state.enemies {
        require((0..=e.max_hp).contains(&e.hp) && (0..=999).contains(&e.block), "enemy bounds")?;
        match &e.enemy_id {
            ActorEnemyId::Legacy(kind) => {
                require(!state.full_runtime && e.life == super::ActorLife::Active
                    && enemy_bounds(*kind).contains(&e.max_hp), "legacy enemy bounds")?;
                validate_powers(&e.powers, Some(*kind), state, &Actor::Enemy(e.instance_id.clone()))?;
                let intent = e.intent_id.legacy().ok_or_else(|| InputError::Invariant("legacy intent missing".into()))?;
                let selected = e.selected_move.legacy().ok_or_else(|| InputError::Invariant("legacy selected move missing".into()))?;
                require(supports_move_for_turn(*kind, intent) && supports_move_for_turn(*kind, selected), "enemy moves")?;
                let history = e.move_history.iter().map(ActorIntentId::legacy).collect::<Option<Vec<_>>>()
                    .ok_or_else(|| InputError::Invariant("legacy history contains full intent".into()))?;
                require(history.first() == Some(&first_move(*kind)) && history.last() == Some(&selected)
                    && history.iter().all(|id| supports_move_for_turn(*kind, *id)), "selected history")?;
                if *kind == EnemyKind::Cultist { require(history.iter().skip(1).all(|id| *id == MoveId::Attack), "Cultist history")?; }
                let bases = base_damage(*kind);
                require(e.damage_entries.len() == bases.len(), "damage entry membership")?;
                for (id, base) in bases {
                    let entry = e.damage_entries.get(&id).ok_or_else(|| InputError::Invariant("missing damage entry".into()))?;
                    require(entry.base == base.base && (0..=1515).contains(&entry.output), "damage entry bounds")?;
                }
                require(e.full_powers.is_empty() && e.full_damage_entries.is_empty() && e.full_ai_rng.is_none(), "legacy-only actor state")?;
                require(e.full_memory.is_none(), "legacy-only actor memory")?;
            }
            ActorEnemyId::Full(enemy_id) => {
                require(state.full_runtime && (1..=999).contains(&e.max_hp), "full enemy identity/HP")?;
                require(e.powers.is_empty() && e.damage_entries.is_empty()
                    && e.full_ai_rng.is_some(), "full-only actor stream ownership")?;
                require(e.life != super::ActorLife::Replaced || e.hp == 0,
                    "replaced actor is retired at zero HP")?;
                if matches!(e.enemy_id, ActorEnemyId::Full(crate::content::EnemyId::Cultist | crate::content::EnemyId::JawWorm
                    | crate::content::EnemyId::GremlinNob | crate::content::EnemyId::LouseRed
                    | crate::content::EnemyId::LouseGreen | crate::content::EnemyId::SlimeBoss
                    | crate::content::EnemyId::Lagavulin | crate::content::EnemyId::Sentry | crate::content::EnemyId::Guardian | crate::content::EnemyId::Hexaghost
                    | crate::content::EnemyId::SphericGuardian | crate::content::EnemyId::Byrd | crate::content::EnemyId::Chosen | crate::content::EnemyId::Snecko
                    | crate::content::EnemyId::Centurion | crate::content::EnemyId::Mystic
                    | crate::content::EnemyId::GremlinLeader | crate::content::EnemyId::GremlinMad | crate::content::EnemyId::GremlinSneaky | crate::content::EnemyId::GremlinFat | crate::content::EnemyId::GremlinShield | crate::content::EnemyId::GremlinWizard
                    | crate::content::EnemyId::SlaverBlue | crate::content::EnemyId::Taskmaster | crate::content::EnemyId::SlaverRed | crate::content::EnemyId::BookOfStabbing | crate::content::EnemyId::Champ | crate::content::EnemyId::Collector | crate::content::EnemyId::TorchHead | crate::content::EnemyId::BronzeAutomaton | crate::content::EnemyId::BronzeOrb)) {
                    require(e.full_memory.as_ref().is_some_and(|memory| memory.valid_for(*enemy_id)),
                        "supported full actor memory/configuration")?;
                }
                require(valid_runtime_full_powers(&e.full_powers, state, &Actor::Enemy(e.instance_id.clone())), "full enemy powers")?;
                let intent = match &e.intent_id { ActorIntentId::Full(id) => *id, _ => return Err(InputError::Invariant("legacy intent in full actor".into())) };
                let selected = match &e.selected_move { ActorIntentId::Full(id) => *id, _ => return Err(InputError::Invariant("legacy selection in full actor".into())) };
                require(intent == selected && e.move_history.last() == Some(&ActorIntentId::Full(selected))
                    && e.move_history.iter().all(|id| matches!(id, ActorIntentId::Full(_))), "full intent history")?;
                let ascension = e.full_memory.as_ref().map_or(0, super::enemies::EnemyMemory::ascension);
                let bases = supported_full_damage(*enemy_id, ascension);
                require(e.full_damage_entries.len() == bases.len(), "full damage entry membership")?;
                for (id, base) in bases {
                    let entry = e.full_damage_entries.get(&id).ok_or_else(|| InputError::Invariant("missing full damage entry".into()))?;
                    let louse_bite = matches!(enemy_id, crate::content::EnemyId::LouseRed | crate::content::EnemyId::LouseGreen)
                        && id == super::FullIntentId::Bite;
                    let divider=if *enemy_id==crate::content::EnemyId::Hexaghost&&id.as_str()=="divider"{match e.full_memory {Some(super::EnemyMemory::Hexaghost{divider_base,..})=>Some(divider_base.unwrap_or(1)),_=>None}}else{None};
                    require((if louse_bite { (5..=7).contains(&entry.base) } else { entry.base == divider.unwrap_or(base.base) })
                        && (0..=30_012).contains(&entry.output), "full damage bounds")?;
                }
            }
        }
        let maximum=if state.full_runtime{30_012}else{1515};
        require(e.intent_damage.map_or(true, |n| (0..=maximum).contains(&n)), "intent damage bounds")?;
        let has_damage = match &e.intent_id {
            ActorIntentId::Legacy(id) => e.damage_entries.contains_key(id),
            ActorIntentId::Full(id) => e.full_damage_entries.contains_key(id),
        };
        require(e.intent_damage.is_some() == has_damage, "published intent damage shape")?;
    }
    validate_rng(&state.rng, &state.enemies[0].instance_id, state.full_runtime)?;
    let owned = validate_cards(&state.piles, state.in_play.as_ref(),state.stasis.values())?;
    require(owned.len() == state.card_ids_issued.len()
        && owned.iter().all(|id| state.card_ids_issued.contains(id)), "issued card ownership completeness")?;
    for card in state.piles.all(state.in_play.as_ref()).chain(state.stasis.values()) {
        if let Some(live) = card.full() {
            require(state.full_runtime && super::card_state::handle(live.instance) == card.handle(),
                "full card instance identity/mode")?;
            require((-99..=99).contains(&live.local.temporary_cost), "temporary card cost bounds")?;
            if let CombatCardRef::Permanent(instance) = live.origin {
                require(instance == live.instance, "permanent card origin")?;
            }
        } else {
            require(!state.full_runtime, "legacy card in full combat")?;
        }
    }
    require(state.public.revealed_order.iter().all(|id| state.card_ids_issued.contains(id)), "public reveal owner")?;
    for effect in &state.pending { validate_effect(effect, state)?; }
    let quiet = state.pending.is_empty() && state.in_play.is_none();
    match state.phase {
        Phase::Decision => {
            require(quiet && state.player.hp > 0 && state.enemies.iter().any(|e|
                e.life == super::ActorLife::Active && e.hp > 0), "live quiescent decision")?;
            for e in state.enemies.iter().filter(|e|
                e.life == super::ActorLife::Active && e.hp > 0) {
            require(e.intent_id == e.selected_move, "decision published move")?;
            for entry in e.damage_entries.values().chain(e.full_damage_entries.values()) {
                let expected = super::effects::refreshed_damage(entry.base, &e.powers, &e.full_powers,
                    &state.player.powers, &state.player_full_powers,
                    state.character.stance == super::characters::Stance::Wrath, state.full_runtime)?;
                require(entry.output == expected, "decision refreshed damage")?;
            }
            let damage = match &e.intent_id {
                ActorIntentId::Legacy(id) => e.damage_entries.get(id).map(|d| d.output),
                ActorIntentId::Full(id) => e.full_damage_entries.get(id).map(|d| d.output),
            };
            require(e.intent_damage == damage, "decision published damage")?;
            }
        }
        Phase::Terminal => {
            // Lethality prunes FinishCard: the lethal card keeps its in-play owner.
            require(state.pending.is_empty(), "terminal cleanup work")?;
            require(state.hp_after_cleanup == Some(state.player.hp), "terminal cleanup HP")?;
            match state.outcome {
                Some(Outcome::Victory) => {
                    require(super::enemies::act2::combat_victory_ready(state)
                        && state.player.hp > 0, "victory actors")?;
                    let before = state.hp_before_healing.ok_or_else(|| InputError::Invariant("missing prehealing HP".into()))?;
                    require((1..=state.player.max_hp).contains(&before)
                        && if state.full_runtime { before == state.player.hp }
                           else { (i64::from(before) + 6).min(i64::from(state.player.max_hp)) == i64::from(state.player.hp) },
                        "victory healing")?;
                }
                Some(Outcome::Death) => require(state.player.hp == 0 && state.enemies.iter().any(|e|
                    e.life == super::ActorLife::Active && e.hp > 0)
                    && state.hp_before_healing == Some(0), "death HP")?,
                None => return Err(InputError::Invariant("terminal outcome missing".into())),
            }
        }
        Phase::Selection => {
            require(state.pending_selection.is_some() && state.in_play.is_some()
                && state.pending.iter().any(|effect| effect.kind == EffectKind::FinishCard),
                "selection continuation ownership")?;
        }
        _ => {
            require(!state.pending.is_empty(), "busy continuation missing")?;
            require(state.in_play.is_none() || state.phase == Phase::PlayerAction, "in-play phase")?;
            if state.in_play.is_some() {
                require(state.pending.iter().any(|w| matches!(&w.kind,
                    EffectKind::FinishCard | EffectKind::Terminal(_))), "in-play continuation missing")?;
            }
            if state.player.hp == 0 || super::enemies::act2::combat_victory_ready(state) {
                require(!(state.player.hp == 0 && super::enemies::act2::combat_victory_ready(state)), "unsupported simultaneous lethality")?;
                let outcome = if state.player.hp == 0 { Outcome::Death } else { Outcome::Victory };
                require(state.pending.iter().any(|w| w.kind == EffectKind::Terminal(outcome)), "lethal pending terminal work")?;
            }
        }
    }
    if let Some(selection) = &state.pending_selection {
        require(state.phase == Phase::Selection && state.in_play.as_ref()
            .map(|card| card.handle().as_str() == selection.parent.as_str()).unwrap_or(false),
            "selection retains exact in-play parent")?;
        require(!selection.candidates.is_empty(), "pending selection has eligible choices")?;
        require(selection.candidates.iter().all(|candidate|
            owned_cards(&state.piles, state.in_play.as_ref()).any(|card| &card.handle() == candidate)
                && state.in_play.as_ref().map(OwnedCard::handle).as_ref() != Some(candidate)),
            "selection candidates remain uniquely owned")?;
        require(selection.required > 0 && selection.required <= selection.candidates.len()
            && selection.selected.len() < selection.required
            && selection.selected.iter().enumerate().all(|(index, id)|
                selection.candidates.contains(id) && !selection.selected[..index].contains(id)
                    && owned_cards(&state.piles, state.in_play.as_ref()).any(|card| &card.handle() == id)),
            "selection ordered progress")?;
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
            let mut summon_owned=false;
            if let Some(progress)=state.spawn_progress.as_mut(){
                for owned in [&mut progress.constructor_rng,&mut progress.latest_child_rng]{
                    if owned.id()==latest.id()&&owned.key()==latest.key()&&latest.word_index()>=owned.word_index(){*owned=latest.clone();summon_owned=true;break;}
                }
            }
            if summon_owned{mark_fault(state,FaultKind::RngExhausted,"owned summon child stream exhausted".into());return;}
            if let Some(stream)=state.card_rng.as_mut().filter(|stream|stream.id()==latest.id()){
                if stream.key()==latest.key()&&latest.word_index()>=stream.word_index(){
                    *stream=latest;mark_fault(state,FaultKind::RngExhausted,"owned full card-random stream exhausted".into());return;
                }
            }
            if let Some(progress) = state.replacement_progress.as_mut() {
                if let Some(owned) = progress.latest_child_rng.as_mut().filter(|owned|
                    owned.id() == latest.id() && owned.key() == latest.key()
                        && latest.word_index() >= owned.word_index()) {
                    *owned = latest;
                    mark_fault(state, FaultKind::RngExhausted,
                        "staged split child AI stream exhausted".into());
                    return;
                }
            }
            let mut full_owner = state.enemies.iter_mut().find_map(|enemy| enemy.full_ai_rng.as_mut()
                .filter(|stream| stream.id() == latest.id()));
            if let Some(stream) = full_owner.as_mut() {
                if stream.key() == latest.key() && latest.word_index() >= stream.word_index() {
                    **stream = latest;
                    mark_fault(state, FaultKind::RngExhausted, "owned full enemy AI stream exhausted".into());
                    return;
                }
            }
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
        if state.pending_selection.is_some() { return Transition { status: TransitionStatus::Selection }; }
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
            Phase::Selection => TransitionStatus::Selection,
            Phase::Terminal => TransitionStatus::Terminal,
            _ => {
                super::model::mark_fault(state, FaultKind::Invariant, "resolution has no continuation".into());
                TransitionStatus::Fault
            }
        };
        return Transition { status };
    }
}
