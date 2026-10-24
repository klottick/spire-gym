//! Item-owned mechanics. Effects returned here are ordinary work for the shared
//! combat resolver; this module does not own or drain a second effect queue.
mod relics;
mod potions;

#[cfg(test)]
mod tests;

use crate::{combat::{Combat, Effect, EnemyHandle, Hook}, content::{PotionId, RelicId}, run::{RunError, RunState}};

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RelicCounterView {pub relic:RelicId,pub value:i32}
pub(crate) fn visible_counters(counters:&std::collections::BTreeMap<RelicId,i32>)->Vec<RelicCounterView>{
    counters.get(&RelicId::NeowsLament).map(|value|RelicCounterView{relic:RelicId::NeowsLament,value:*value}).into_iter().collect()
}
pub(crate) fn replace_burning_blood(state:&mut RunState)->Result<(),RunError>{relics::replace_burning_blood(state)}

/// Item ownership is transferred into active full combat and returned with its
/// exit payload. Outside combat, RunState remains the permanent authority.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatItemContext {
    pub(crate) battle:NativeBattleContext,
    pub(crate) collar_activated:bool,
    counters: std::collections::BTreeMap<RelicId,i32>,
    relics: Vec<RelicId>,
    potions: Vec<Option<PotionId>>,
    lantern_armed: bool,
    heal_room: NativeHealRoom,
}

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub(crate) struct NativeBattleContext {pub elite_trigger:bool,pub boss:bool,pub burning:Option<crate::combat::enemies::BurningBuff>}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RelicHookWork {
    pub top: Vec<Effect>,
    pub bottom: Vec<Effect>,
}

impl RelicHookWork {
    pub(crate) fn is_empty(&self) -> bool { self.top.is_empty() && self.bottom.is_empty() }
}

impl CombatItemContext {
    pub fn new(relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Self {
        let mut counters=std::collections::BTreeMap::new();
        if relics.contains(&RelicId::NeowsLament){counters.insert(RelicId::NeowsLament,3);}
        Self { counters, relics, potions, lantern_armed: false, heal_room: NativeHealRoom::default(),battle:NativeBattleContext::default(),collar_activated:false }
    }

    /// Move permanent inventory into the single active combat owner.
    pub(crate) fn take_from_run(state: &mut RunState) -> Self {
        Self { battle:NativeBattleContext::default(),collar_activated:false,counters:std::mem::take(&mut state.relic_counters),relics: std::mem::take(&mut state.relics),
            potions: std::mem::take(&mut state.potions), lantern_armed: false,
            heal_room: NativeHealRoom::from_run(state) }
    }

    /// Return active ownership once without replacing another live inventory.
    pub(crate) fn return_to_run(self, state: &mut RunState) -> Result<(), RunError> {
        if !state.relics.is_empty() || !state.potions.is_empty() || !state.relic_counters.is_empty() {
            return Err(RunError::InvalidAction);
        }
        state.relics = self.relics;
        state.potions = self.potions;
        state.relic_counters=self.counters;
        Ok(())
    }

    pub(crate) fn owns_relic(&self, id: RelicId) -> bool { self.relics.contains(&id) }
    pub(crate) fn with_battle_context(mut self,battle:NativeBattleContext)->Self{self.battle=battle;self}
    pub(crate) fn relics(&self) -> &[RelicId] { &self.relics }
    pub(crate) fn potions(&self) -> &[Option<PotionId>] { &self.potions }
    pub(crate) fn counters(&self)->&std::collections::BTreeMap<RelicId,i32>{&self.counters}
    pub(crate) fn with_counters(mut self,counters:std::collections::BTreeMap<RelicId,i32>)->Self{self.counters=counters;self}
    pub(crate) fn heal_room(&self) -> NativeHealRoom { self.heal_room }

    pub(crate) fn with_native_room(mut self, phase: PotionRoomPhase,
        map: Option<&crate::run::RunMap>) -> Self {
        self.heal_room = NativeHealRoom::from_map(phase, map);
        self
    }

    fn with_potions(&self, potions: Vec<Option<PotionId>>) -> Self {
        Self { battle:self.battle,collar_activated:self.collar_activated,counters:self.counters.clone(),relics: self.relics.clone(), potions, lantern_armed: self.lantern_armed,
            heal_room: self.heal_room }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeHealRoom { combat: bool, current_map_node: bool }

impl NativeHealRoom {
    fn from_map(phase: PotionRoomPhase, map: Option<&crate::run::RunMap>) -> Self {
        let current_map_node = map.is_some_and(|map| map.current
            .is_some_and(|current| map.nodes.iter().any(|node| node.id == current)));
        Self { combat: phase == PotionRoomPhase::Combat, current_map_node }
    }

    fn from_run(state: &RunState) -> Self {
        Self::from_map(if state.combat.is_some() { PotionRoomPhase::Combat }
            else { PotionRoomPhase::OutsideCombat }, state.map.as_ref())
    }

    pub(crate) fn is_live_combat_with_node(self) -> bool { self.combat && self.current_map_node }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PotionRoomPhase { Combat, OutsideCombat }

/// Borrowed native facts for a single potion-use attempt. This is an ephemeral
/// projection supplied by the current owner, never another inventory/state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativePotionUseContext<'a> {
    pub phase: PotionRoomPhase,
    pub turn_ended: bool,
    pub we_meet_again: bool,
    pub hp: u32,
    pub max_hp: u32,
    pub live_targets: &'a [EnemyHandle],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PotionUseEffects {
    pub effects: Vec<Effect>,
    pub potions: Vec<Option<PotionId>>,
}

/// Dispatch run-level item checkpoints that mutate persistent run resources.
/// Active-combat item effects use the same handlers with the combat-owned item
/// context and are committed through Combat's ordinary resolver.
pub fn on_hook(state: &mut RunState, hook: Hook) -> Result<(), RunError> {
    relics::on_run_hook(state, hook)
}

/// Native relic acquisition while RunState owns the permanent inventory.
pub(crate) fn acquire_relic(state: &mut RunState, id: RelicId) -> Result<(), RunError> {
    relics::acquire(state, id)
}

/// Apply the finite item-owned ordinary-heal callback to an already computed
/// requested amount. Other healing modifiers remain explicitly unsupported.
pub(crate) fn ordinary_heal_amount(relics: &[RelicId], amount: i32) -> i32 {
    ordinary_heal_amount_in_room(relics, amount, NativeHealRoom::default())
}

pub(crate) fn ordinary_heal_amount_in_room(relics: &[RelicId], amount: i32,
    room: NativeHealRoom) -> i32 {
    if relics.contains(&RelicId::MarkOfBloom) { return 0; }
    if relics.contains(&RelicId::MagicFlower) && room.is_live_combat_with_node() {
        let scaled = (amount as f32) * 1.5_f32;
        return (scaled + 0.5_f32).floor() as i32;
    }
    amount
}

/// Produce ordered native effects for a combat lifecycle checkpoint.
pub(crate) fn relic_hook_effects(context: &mut CombatItemContext, hook: Hook, hp: i32) -> RelicHookWork {
    relics::on_combat_hook(context, hook, hp)
}

pub(crate) fn battle_start_enemy_effects(context:&CombatItemContext,targets:&[EnemyHandle])->Vec<Effect>{relics::battle_start_enemy_effects(context,targets)}

/// Validate and prepare a potion use without mutating the caller's item owner.
/// The returned post-use slots are committed only alongside validated effects.
pub(crate) fn potion_use_effects(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>) -> Result<PotionUseEffects, RunError> {
    potions::use_potion(context, slot, target, facts)
}

/// Finite supported startup/use IDs, independent of registration metadata.
pub(crate) fn validate_combat_inventory(context: &CombatItemContext) -> Result<(), RunError> {
    if context.owns_relic(RelicId::BurningBlood)&&context.owns_relic(RelicId::BlackBlood){return Err(RunError::InvalidAction)}
    if context.owns_relic(RelicId::NeowsLament)&&!matches!(context.counters.get(&RelicId::NeowsLament),Some(1..=3)|Some(-2)){return Err(RunError::InvalidAction)}
    for id in &context.relics {
        if !matches!(id, RelicId::BurningBlood | RelicId::RingOfTheSnake | RelicId::CrackedCore
            | RelicId::PureWater | RelicId::BloodVial | RelicId::Anchor | RelicId::Lantern
            | RelicId::Strawberry | RelicId::Pear | RelicId::Mango | RelicId::SacredBark
            | RelicId::MarkOfBloom | RelicId::MagicFlower | RelicId::ToyOrnithopter | RelicId::NeowsLament
            | RelicId::BlackBlood | RelicId::EmptyCage | RelicId::SlaversCollar | RelicId::RedMask
            | RelicId::Vajra | RelicId::OddlySmoothStone | RelicId::BagOfPreparation | RelicId::BagOfMarbles | RelicId::BronzeScales) {
            return Err(RunError::Unsupported("unimplemented combat relic"));
        }
    }
    for id in context.potions.iter().flatten() {
        if !matches!(id, PotionId::FirePotion | PotionId::BloodPotion | PotionId::FruitJuice
            | PotionId::BlockPotion | PotionId::EnergyPotion | PotionId::StrengthPotion
            | PotionId::SwiftPotion | PotionId::WeakPotion | PotionId::FearPotion
            | PotionId::DexterityPotion | PotionId::AncientPotion) {
            return Err(RunError::Unsupported("unimplemented combat potion"));
        }
    }
    Ok(())
}

/// Pure full-combat preparation used by both legal menus and accepted use.
pub(crate) fn prepare_combat_potion(combat: &Combat, slot: usize,
    target: Option<EnemyHandle>, we_meet_again: bool) -> Result<PotionUseEffects, RunError> {
    let context = combat.item_context().ok_or(RunError::InvalidAction)?;
    let (phase, hp, max_hp, live_targets) = combat.native_potion_use_state()?;
    let facts = NativePotionUseContext { phase: PotionRoomPhase::Combat,
        turn_ended: phase == crate::combat::Phase::Terminal, we_meet_again,
        hp: hp as u32, max_hp: max_hp as u32, live_targets: &live_targets };
    let mut prepared = potion_use_effects(context, slot, target, &facts)?;
    if context.owns_relic(RelicId::ToyOrnithopter) {
        prepared.effects.push(Effect { source: crate::combat::Source::None,
            target: Some(crate::combat::Actor::Player), kind: crate::combat::EffectKind::Heal(5) });
    }
    combat.preflight_native_effects(&prepared.effects)?;
    Ok(prepared)
}

/// Resolve a combat-owned potion through the shared queue, committing its slot
/// only with the validated native effects.
pub(crate) fn use_combat_potion(combat: &mut Combat, slot: usize,
    target: Option<EnemyHandle>, we_meet_again: bool) -> Result<(), RunError> {
    let context = combat.item_context().cloned().ok_or(RunError::InvalidAction)?;
    let prepared = prepare_combat_potion(combat, slot, target, we_meet_again)?;
    combat.enqueue_native_effects(prepared.effects,
        Some(context.with_potions(prepared.potions)))
}

/// Native noncombat use while RunState is the sole owner of HP and potion slots.
/// This deliberately does not create a public action or advance room progression.
pub(crate) fn use_run_potion(state: &mut RunState, slot: usize,
    turn_ended: bool, we_meet_again: bool) -> Result<(), RunError> {
    let (hp,max_hp,potions)=prepare_run_potion(state,slot,turn_ended,we_meet_again)?;
    state.hp=hp;state.max_hp=max_hp;state.potions=potions;Ok(())
}

pub(crate) fn prepare_run_potion(state:&RunState,slot:usize,turn_ended:bool,we_meet_again:bool)->Result<(u32,u32,Vec<Option<PotionId>>),RunError>{
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    if state.combat.is_some() { return Err(RunError::InvalidAction); }
    if state.max_hp == 0 || state.max_hp > 999 || state.hp > state.max_hp {
        return Err(RunError::InvalidAction);
    }
    let facts = NativePotionUseContext { phase: PotionRoomPhase::OutsideCombat,
        turn_ended, we_meet_again, hp: state.hp, max_hp: state.max_hp,
        live_targets: &[] };
    let owned = CombatItemContext::new(state.relics.clone(), state.potions.clone());
    let prepared = potion_use_effects(&owned, slot, None, &facts)?;
    let mut hp = state.hp;
    let mut max_hp = state.max_hp;
    for effect in &prepared.effects {
        match &effect.kind {
            crate::combat::EffectKind::Heal(amount) => {
                let amount = ordinary_heal_amount(owned.relics(), *amount).max(0) as u32;
                if hp > 0 { hp = hp.saturating_add(amount).min(max_hp); }
            }
            crate::combat::EffectKind::IncreaseMaxHpThenHeal(amount) => {
                max_hp = max_hp.checked_add(*amount as u32)
                    .filter(|value| *value <= 999)
                    .ok_or(RunError::Unsupported("maximum HP overflow"))?;
                let heal = ordinary_heal_amount(owned.relics(), *amount).max(0) as u32;
                if hp > 0 { hp = hp.saturating_add(heal).min(max_hp); }
            }
            _ => return Err(RunError::InvalidAction),
        }
    }
    if owned.owns_relic(RelicId::ToyOrnithopter) {
        let heal = ordinary_heal_amount(owned.relics(), 5) as u32;
        if hp > 0 { hp = hp.saturating_add(heal).min(max_hp); }
    }
    Ok((hp,max_hp,prepared.potions))
}
