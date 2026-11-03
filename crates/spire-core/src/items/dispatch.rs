//! The one relic dispatcher (M27 design §6.1–§6.2): the only loops that visit owned relics
//! for combat phases and Run events. Relics are visited in the player's relic order, which
//! is acquisition order; each relic is handled by exactly one handler owner.
//!
//! Queue law (H0 seam map §5, binding F1): each handler pushes its work to `top` (in front
//! of everything already queued) or `bottom` (after it). Both sides keep visit order, as
//! the shared resolver always has. Relics whose source rule says "at top" are the
//! exception of design §6.1 rule 3: among themselves, the later-visited relic's top work
//! runs first, so their groups are reversed in place inside the top batch.
use crate::combat::{Actor, EnemyHandle, Hook, InputError, Source};
use crate::content::{CardId, CardType, RelicId};
use crate::run::{RunError, RunState};

#[cfg(test)]
thread_local! { static PHASE_LOG: std::cell::RefCell<Vec<ItemPhase>> = std::cell::RefCell::new(Vec::new()); }
#[cfg(test)]
thread_local! { static PHASE_FACT_LOG: std::cell::RefCell<Vec<(ItemPhase,Option<PhaseCard>,bool,bool,usize)>> = std::cell::RefCell::new(Vec::new()); }

#[cfg(test)]
pub(crate) fn take_phase_log() -> Vec<ItemPhase> {
    PHASE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
#[cfg(test)]
pub(crate) fn take_phase_fact_log() -> Vec<(ItemPhase,Option<PhaseCard>,bool,bool,usize)> {
    PHASE_FACT_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

use super::{CombatItemContext, RelicHookWork};
use super::lifecycle::EquipWork;

/// The documented combat phases (design §6.1, §6.5). Single-relic modifier points are
/// laws (`items::laws`), not phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemPhase {
    CombatStartPreDraw, CombatStart, AfterOpeningDraw, TurnStartPreDraw, AfterDraw,
    CardDrawn, CardUsed, CardPlayed, AfterUse, AttackResolved, ManualDiscard, Exhaust,
    DrawPileShuffled, BlockBroken, HpLost, PowerApplied, EnemyDied, EnemySpawned, HandEmptied,
    PotionUsed, EndTurnBeforeHandCards, CombatEnd,
}

/// The card a card phase is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PhaseCard { pub id: CardId, pub card_type: CardType, pub cost_paid: i32 }

/// The facts a phase's handlers may read. Each dispatch site fills what its phase has;
/// everything else keeps its default.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PhaseFacts<'a> {
    /// The engine hook that raised the phase; `None` for the combat-entry startup dispatch.
    pub hook: Option<Hook>,
    pub target: Option<&'a Actor>,
    pub source: Option<&'a Source>,
    pub hp: i32, pub max_hp: i32, pub block: i32,
    /// Every enemy in formation order, and the active living subset.
    pub enemies: &'a [EnemyHandle], pub live_enemies: &'a [EnemyHandle],
    pub hand_size: usize, pub has_empty_orb: bool,
    pub draw_size: usize, pub discard_size: usize, pub no_draw: bool,
    pub card: Option<PhaseCard>,
    pub power: Option<(&'a str, i32)>,
    pub spawned: Option<&'a EnemyHandle>,
    pub elite: bool, pub boss: bool, pub deck_curses: usize,
    pub hp_lost: i32,
    /// The dead enemy, its Poison at callback time and the live eligible count.
    pub dead_enemy: Option<(&'a EnemyHandle, Option<i32>, usize)>,
}

impl<'a> PhaseFacts<'a> {
    pub(crate) fn new(hook: Option<Hook>, hp: i32, max_hp: i32, block: i32) -> Self {
        Self { hook, target: None, source: None, hp, max_hp, block, enemies: &[], live_enemies: &[], hand_size: 0,
            has_empty_orb: false, draw_size: 0, discard_size: 0, no_draw: false, card: None,
            power: None, spawned: None, elite:false, boss:false, deck_curses:0, hp_lost: 0, dead_enemy: None }
    }
}

/// The phase a combat hook raises in the generic lifecycle dispatch (`effects.rs` Hook
/// arm). `Hook::Death` is dispatched at its own site inside `combat::hooks::dispatch`,
/// after the monster callbacks, so it raises nothing here.
pub(crate) fn hook_phase(hook: Hook) -> Option<ItemPhase> {
    Some(match hook {
        Hook::CombatStartedPreDraw => ItemPhase::CombatStartPreDraw,
        Hook::CombatStarted => ItemPhase::CombatStart,
        Hook::TurnStartedPreDraw => ItemPhase::TurnStartPreDraw,
        Hook::CardDrawn => ItemPhase::CardDrawn,
        Hook::CardPlayed => ItemPhase::CardPlayed,
        Hook::CardAfterUse => ItemPhase::CardUsed,
        Hook::AttackResolved => ItemPhase::AttackResolved,
        Hook::CardManuallyDiscarded => ItemPhase::ManualDiscard,
        Hook::DrawPileShuffled => ItemPhase::DrawPileShuffled,
        Hook::PlayerPreEndTurnCards => ItemPhase::EndTurnBeforeHandCards,
        Hook::CombatSettled => ItemPhase::CombatEnd,
        _ => return None,
    })
}

/// The relic slots of a player turn-start checkpoint, in source order
/// (source-check-turnstart-hooks-v1). Each relic's arm reads only its source hook's slot:
///
/// | Source hook | Phase | Raised at |
/// |---|---|---|
/// | atTurnStart | `TurnStartPreDraw` | `Hook::TurnStartedPreDraw`, every turn, before the card, power and orb callbacks |
/// | atTurnStartPostDraw | `AfterOpeningDraw` | turn 1: also `Hook::TurnStartedPreDraw`; the battle-start block calls it right after the start relics |
/// | atTurnStartPostDraw | `AfterDraw` | later turns: `Hook::TurnStarted`, once the turn draw has resolved |
///
/// No relic overrides onEnergyRecharge; the post-draw slots belong to the complete item layer.
pub(crate) fn turn_start_phases(hook: Hook, first_turn: bool, complete: bool) -> &'static [ItemPhase] {
    match (hook, first_turn, complete) {
        (Hook::TurnStartedPreDraw, true, true) => &[ItemPhase::TurnStartPreDraw, ItemPhase::AfterOpeningDraw],
        (Hook::TurnStartedPreDraw, _, _) => &[ItemPhase::TurnStartPreDraw],
        (Hook::TurnStarted, false, true) => &[ItemPhase::AfterDraw],
        _ => &[],
    }
}

/// Relics that consume a phase whose engine hook is produced only on demand (M25's
/// `Hook::DrawPileShuffled`), so the hook stays absent when no owned relic reads it.
fn consumes(id: RelicId, phase: ItemPhase) -> bool {
    matches!((phase, id), (ItemPhase::DrawPileShuffled, RelicId::Melange | RelicId::Sundial | RelicId::TheAbacus))
}

/// Whether an owned relic consumes `phase` (the producer's one question to the dispatcher).
pub(crate) fn phase_consumed(context: &CombatItemContext, phase: ItemPhase) -> bool {
    context.relics().iter().any(|id| consumes(*id, phase))
}

/// Relics whose source rule queues their work "at top" (`task0/a2-order/f1-queue-sides.tsv`).
fn source_top(id: RelicId) -> bool {
    matches!(id, RelicId::BloodVial | RelicId::Lantern | RelicId::Orichalcum | RelicId::Brimstone | RelicId::RunicCube
        | RelicId::CharonsAshes | RelicId::AncientTeaSet | RelicId::BottledFlame | RelicId::BirdFacedUrn
        | RelicId::CoffeeDripper | RelicId::CursedKey | RelicId::NlothsHungryFace | RelicId::CentennialPuzzle
        // atBattleStart addToTop (source-check-turnstart-hooks-v1).
        | RelicId::Akabeko | RelicId::BronzeScales | RelicId::ClockworkSouvenir | RelicId::DataDisk | RelicId::DuVuDoll
        | RelicId::Girya | RelicId::MutagenicStrength | RelicId::OddlySmoothStone | RelicId::Pantograph
        | RelicId::SlingOfCourage | RelicId::ThreadAndNeedle | RelicId::Vajra | RelicId::NeowsLament
        | RelicId::PreservedInsect | RelicId::PhilosophersStone | RelicId::TeardropLocket)
}

/// The one per-relic handler chain: accepted owners first, then the M27 lane files. A relic
/// is handled by exactly one of them (`m27_each_relic_has_one_handler`).
fn handle(id: RelicId, context: &mut CombatItemContext, phase: ItemPhase, facts: &PhaseFacts<'_>, work: &mut RelicHookWork)
    -> Result<bool, InputError>
{
    Ok(super::relics::phase(id, context, phase, facts, work)?
        || super::silent::phase(id, context, phase, facts, work)?
        || super::ironclad::phase(id, context, phase, facts, work)?
        || super::ironclad_turns::phase(id, context, phase, facts, work)?
        || super::defect_phase(id, context, phase, facts, work)?
        || super::relic_effects::turns::phase(id, context, phase, facts, work)?
        || super::relic_effects::play::phase(id, context, phase, facts, work)?
        || super::relic_effects::run::phase(id, context, phase, facts, work)?
        || super::relic_effects::structural::phase(id, context, phase, facts, work)?)
}

/// Reverse, in place, the top groups of source-top relics (design §6.1 rule 3). Groups are
/// `(start, len, source_top)` ranges of `top` in visit order.
pub(crate) fn order_top_groups<T>(top: &mut Vec<T>, groups: &[(usize, usize, bool)]) {
    let marked: Vec<_> = groups.iter().filter(|(_, len, top_side)| *top_side && *len > 0).copied().collect();
    if marked.len() < 2 { return; }
    let mut slots: Vec<Option<T>> = std::mem::take(top).into_iter().map(Some).collect();
    let taken: Vec<Vec<T>> = marked.iter().map(|(start, len, _)| (*start..start + len).map(|i| slots[i].take().unwrap()).collect()).collect();
    let mut out = Vec::with_capacity(slots.len());
    let mut next = taken.into_iter().rev();
    let mut i = 0;
    while i < slots.len() {
        if let Some((_, len, _)) = marked.iter().find(|(start, _, _)| *start == i) {
            out.extend(next.next().unwrap());
            i += len;
        } else {
            out.push(slots[i].take().unwrap());
            i += 1;
        }
    }
    *top = out;
}

/// The only combat relic visit loop.
pub(crate) fn combat_phase(context: &mut CombatItemContext, phase: ItemPhase, facts: &PhaseFacts<'_>)
    -> Result<RelicHookWork, InputError>
{
    #[cfg(test)]
    PHASE_LOG.with(|log| log.borrow_mut().push(phase));
    #[cfg(test)]
    PHASE_FACT_LOG.with(|log|log.borrow_mut().push((phase,facts.card,facts.elite,facts.boss,facts.deck_curses)));
    let mut work = RelicHookWork::default();
    let mut groups = Vec::new();
    for id in context.relics().to_vec() {
        let start = work.top.len();
        handle(id, context, phase, facts, &mut work)?;
        groups.push((start, work.top.len() - start, source_top(id)));
    }
    order_top_groups(&mut work.top, &groups);
    Ok(work)
}

/// Run item events (design §6.5 Run rows, §11). Events that carry a value let handlers
/// change it (prices, heal amounts, gold).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RunItemEvent {
    FloorEntered { floor: u32 },
    RoomEntered { kind: crate::run::RoomViewKind },
    CampfireOptions { options: Vec<crate::run::RoomChoiceKind> },
    RestHeal { amount: u32 },
    Rested { card_reward: bool },
    ShopEntered,
    ShopPrice { price: u32 },
    /// Gold spent in a ShopRoom: a product purchase or the card-removal service (Maw Bank).
    ShopPurchase,
    RemovalPrice { price: u32 },
    RewardsBuilding {
        room: super::run_laws::RewardRoom,
        /// How many cards appear in each choose-one reward screen.
        card_choices: u32,
        /// How many additional choose-one card reward screens are offered.
        extra_card_rewards: u32,
        /// Percent bonus applied to the base gold reward, rounded to nearest.
        gold_percent: u32,
        /// How many additional elite relic rows the owner should produce.
        extra_elite_relics: u32,
        /// Whether the ordinary potion row is present.
        include_potion: bool,
        /// Whether a Singing Bowl choice is appended to card reward screens.
        singing_bowl: bool,
    },
    ChestOpened { boss: bool },
    /// `direct`: Note For Yourself's direct obtain, which bypasses Omamori's obtain-effect
    /// constructor but not the other obtain hooks (event-neow research §6).
    CardObtaining { id: CardId, upgrades: u32, negated: bool, direct: bool },
    CardObtained { id: CardId, instance: u64 },
    CardRemoved,
    GoldGaining { amount: u32 },
    GoldGained { amount: u32 },
    CombatSettled { outcome: crate::combat::Outcome },
}

/// The one Run relic handler chain (accepted Run relics, then the lane files).
fn handle_run(id: RelicId, state: &mut RunState, event: &mut RunItemEvent) -> Result<bool, RunError> {
    Ok(super::relics::run_event(id, state, event)?
        || super::relic_effects::turns::run_event(id, state, event)?
        || super::relic_effects::play::run_event(id, state, event)?
        || super::relic_effects::run::run_event(id, state, event)?
        || super::relic_effects::structural::run_event(id, state, event)?)
}

/// Route obtain-time equipment through the same single-owner chain as item effects.
pub(crate) fn equip(id:RelicId,state:&mut RunState,work:&mut EquipWork)->Result<(),RunError>{
    let _handled=super::relics::equip(id,state,work)?
        ||super::relic_effects::turns::equip(id,state,work)?
        ||super::relic_effects::play::equip(id,state,work)?
        ||super::relic_effects::run::equip(id,state,work)?
        ||super::relic_effects::structural::equip(id,state,work)?
        ||super::main_relics::equip(id,state,work)?;
    Ok(())
}

/// Route the completion of obtain-time selections to their one relic owner.
pub(crate) fn selection_done(id:RelicId,state:&mut RunState,chosen:&[u64])->Result<(),RunError>{
    let _handled=super::relics::selection_done(id,state,chosen)?
        ||super::relic_effects::turns::selection_done(id,state,chosen)?
        ||super::relic_effects::play::selection_done(id,state,chosen)?
        ||super::relic_effects::run::selection_done(id,state,chosen)?
        ||super::relic_effects::structural::selection_done(id,state,chosen)?;
    Ok(())
}

// The only Run relic visit loop.
#[cfg(test)]
thread_local! { static RUN_EVENT_LOG: std::cell::RefCell<Vec<&'static str>> = std::cell::RefCell::new(Vec::new()); }
#[cfg(test)]
thread_local! { static RUN_COMBAT_SETTLED_OUTCOMES: std::cell::RefCell<Vec<crate::combat::Outcome>> = std::cell::RefCell::new(Vec::new()); }

#[cfg(test)]
pub(crate) fn take_run_event_log() -> Vec<&'static str> {
    RUN_EVENT_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
#[cfg(test)]
pub(crate) fn take_combat_settled_outcomes() -> Vec<crate::combat::Outcome> {
    RUN_COMBAT_SETTLED_OUTCOMES.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

#[cfg(test)]
fn run_event_name(event: &RunItemEvent) -> &'static str {
    match event {
        RunItemEvent::FloorEntered { .. } => "FloorEntered",
        RunItemEvent::RoomEntered { .. } => "RoomEntered",
        RunItemEvent::CampfireOptions { .. } => "CampfireOptions",
        RunItemEvent::RestHeal { .. } => "RestHeal",
        RunItemEvent::Rested { .. } => "Rested",
        RunItemEvent::ShopEntered => "ShopEntered",
        RunItemEvent::ShopPrice { .. } => "ShopPrice",
        RunItemEvent::ShopPurchase => "ShopPurchase",
        RunItemEvent::RemovalPrice { .. } => "RemovalPrice",
        RunItemEvent::RewardsBuilding { .. } => "RewardsBuilding",
        RunItemEvent::ChestOpened { .. } => "ChestOpened",
        RunItemEvent::CardObtaining { .. } => "CardObtaining",
        RunItemEvent::CardObtained { .. } => "CardObtained",
        RunItemEvent::CardRemoved => "CardRemoved",
        RunItemEvent::GoldGaining { .. } => "GoldGaining",
        RunItemEvent::GoldGained { .. } => "GoldGained",
        RunItemEvent::CombatSettled { .. } => "CombatSettled",
    }
}

pub(crate) fn run_event(state: &mut RunState, event: &mut RunItemEvent) -> Result<(), RunError> {
    #[cfg(test)]
    RUN_EVENT_LOG.with(|log| log.borrow_mut().push(run_event_name(event)));
    #[cfg(test)]
    if let RunItemEvent::CombatSettled { outcome } = event {
        RUN_COMBAT_SETTLED_OUTCOMES.with(|log| log.borrow_mut().push(*outcome));
    }
    for id in state.relics.clone() {
        handle_run(id, state, event)?;
    }
    Ok(())
}
