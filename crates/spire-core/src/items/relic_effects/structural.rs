//! Lane J (Opus helper): selection and structural relic bodies (plan Task J1).
//! Main (M2) created every stub; each body returns `false` until the lane fills it.
//! Lanes edit only this file and their test file.
#![allow(dead_code)]
use crate::combat::{Actor, CardDestination, Effect, EffectKind, FullPowerId, GenerationCost, GenerationPool,
    GenerationWork, InputError, Source};
use crate::content::{CardId, CardType, Character, PotionId, RelicId, RelicTier};
use crate::run::{RunError, RunRngScope, RunState};

use super::super::dispatch::{ItemPhase, PhaseCard, PhaseFacts, RunItemEvent};
use super::super::laws::{PlayRule, PublicViewRules};
use super::super::lifecycle::{EquipWork, RewardSpec, SelectionSpec};
use super::super::{CombatItemContext, RelicHookWork};

/// The relics this lane owns (design Appendix A, Lane J); each is handled here only.
pub(crate) const RELICS: &[RelicId] = &[
    RelicId::BottledFlame, RelicId::BottledLightning, RelicId::BottledTornado, RelicId::GamblingChip,
    RelicId::UnceasingTop, RelicId::Astrolabe, RelicId::CallingBell, RelicId::PandorasBox,
    RelicId::RunicDome, RelicId::RunicPyramid, RelicId::SneckoEye, RelicId::TinyHouse, RelicId::VelvetChoker,
    RelicId::Cauldron, RelicId::DollysMirror, RelicId::FrozenEye, RelicId::Orrery, RelicId::StrangeSpoon,
    RelicId::Toolbox, RelicId::Necronomicon, RelicId::NilrysCodex,
];

/// Combat phase work of this lane's relics. Returns whether `id` is this lane's relic.
pub(crate) fn phase(id: RelicId, context: &mut CombatItemContext, phase: ItemPhase, facts: &PhaseFacts<'_>,
    work: &mut RelicHookWork) -> Result<bool, InputError> {
    if !RELICS.contains(&id) { return Ok(false); }
    let player_turn_start = phase == ItemPhase::TurnStartPreDraw && facts.target == Some(&Actor::Player);
    match id {
        // ChooseOneColorless before the opening draw (spec ruling TOOLBOX-POOL): three distinct
        // ordinary colourless ids, choose one into the hand; nothing auto-plays.
        RelicId::Toolbox if phase == ItemPhase::CombatStartPreDraw && context.is_pre_draw_visit(facts.hook) => {
            bottom(work, id, EffectKind::Generate(choose_one(GenerationPool::Colorless, CardDestination::Hand, false)));
        }
        // End-of-turn optional 1-of-3 into the draw pile (spec ruling NILRYS-CODEX-POOL).
        RelicId::NilrysCodex if phase == ItemPhase::EndTurnBeforeHandCards => {
            bottom(work, id, EffectKind::Generate(choose_one(GenerationPool::Any, CardDestination::ShuffledDraw, true)));
        }
        // Retain every non-Ethereal hand card at the existing pre-end-turn checkpoint.
        RelicId::RunicPyramid if phase == ItemPhase::EndTurnBeforeHandCards => {
            top(work, id, EffectKind::Relic(crate::combat::RelicEffect::RetainHand));
        }
        // Once-per-combat latch; the discard-any/redraw selection needs a Main relic selection
        // owner (REPORT "Requests to Main", J-R3), so the body only arms the latch until then.
        RelicId::GamblingChip => match phase {
            ItemPhase::CombatStartPreDraw if context.is_pre_draw_visit(facts.hook) => context.set_local(id, 0),
            ItemPhase::AfterOpeningDraw if context.local(id) == 0 => {
                context.set_local(id, 1);
                // M27 seam: arm one opening discard/redraw selection.
                bottom(work, id, EffectKind::Relic(crate::combat::RelicEffect::GamblingChipSelect));
            }
            _ => (),
        },
        // Refresh-hand: draw 1 when the hand empties with no pending work during the turn
        // (the engine's HandEmptied site), unless No Draw or both piles are empty.
        RelicId::UnceasingTop if phase == ItemPhase::HandEmptied
            && !facts.no_draw && facts.draw_size + facts.discard_size > 0 => {
            bottom(work, id, EffectKind::Draw(1));
        }
        // Prebattle Confusion; its +2 master hand size is a Main entry seam (J-R4).
        RelicId::SneckoEye if phase == ItemPhase::CombatStartPreDraw && context.is_pre_draw_visit(facts.hook) => {
            work.bottom.push(Effect { source: Source::Relic(id), target: Some(Actor::Player),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("confused"), amount: 1 } });
        }
        // Per-turn play counter: onPlayCard increments up to 6; battle/turn start resets.
        RelicId::VelvetChoker => match phase {
            ItemPhase::CombatStart => context.set_local(id, 0),
            _ if player_turn_start => context.set_local(id, 0),
            ItemPhase::CardPlayed => context.set_local(id, (context.local(id) + 1).min(CHOKER_LIMIT)),
            _ => (),
        },
        // Used-this-turn latch for repeat_play; turn start rearms it.
        RelicId::Necronomicon if player_turn_start => context.set_local(id, 0),
        _ => (),
    }
    Ok(true)
}

const CHOKER_LIMIT: i32 = 6;

fn bottom(work: &mut RelicHookWork, id: RelicId, kind: EffectKind) {
    work.bottom.push(Effect { source: Source::Relic(id), target: Some(Actor::Player), kind });
}

fn top(work: &mut RelicHookWork, id: RelicId, kind: EffectKind) {
    work.top.push(Effect { source: Source::Relic(id), target: Some(Actor::Player), kind });
}

fn choose_one(pool: GenerationPool, destination: CardDestination, optional: bool) -> GenerationWork {
    GenerationWork { pool, count: 3, unique: true, destination, cost: GenerationCost::Ordinary, upgraded: false,
        offer: Some((optional, 1)), captured: false, cards: vec![], next: 0 }
}

pub(crate) fn outgoing_damage(_id: RelicId, _context: &CombatItemContext, _card: Option<PhaseCard>, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn incoming_damage(_id: RelicId, _context: &CombatItemContext, _attacker: Option<&Actor>, _amount: &mut i32) -> bool { false }
pub(crate) fn hp_loss(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn block_decay(_id: RelicId, _context: &CombatItemContext, _current: i32, _kept: &mut i32) -> bool { false }
pub(crate) fn recharge(_id: RelicId, _context: &CombatItemContext, _character: Character, _unspent: i32, _energy: &mut i32) -> bool { false }
pub(crate) fn master_energy(_id: RelicId, _context: &CombatItemContext, _elite_or_boss: bool, _bonus: &mut i32) -> bool { false }
pub(crate) fn play_allowed(id: RelicId, context: &CombatItemContext, _card: PhaseCard, rule: &mut PlayRule) -> bool {
    if id != RelicId::VelvetChoker { return false; }
    if context.local(id) >= CHOKER_LIMIT { rule.allow = Some(false); }
    true
}
pub(crate) fn repeat_play(id: RelicId, context: &mut CombatItemContext, card: PhaseCard, extra: &mut u32) -> bool {
    if id != RelicId::Necronomicon { return false; }
    // The first Attack each turn whose paid cost (X included) is 2 or more plays once more.
    if card.card_type == CardType::Attack && card.cost_paid >= 2 && context.local(id) == 0 {
        context.set_local(id, 1);
        *extra += 1;
    }
    true
}
pub(crate) fn would_exhaust(id: RelicId, context: &mut CombatItemContext, card: PhaseCard, exhaust: &mut bool) -> bool {
    if id != RelicId::StrangeSpoon { return false; }
    // One Boolean per eligible played-card cleanup (spec ruling STRANGE-SPOON-OMNISCIENCE).
    // The roll is owned by the cleanup site (J-R1): it stores 1 (save) or 2 (exhaust) in the
    // Spoon local immediately before this law; the law consumes it.
    if *exhaust && card.card_type != CardType::Power && context.local(id) == SPOON_SAVE { *exhaust = false; }
    context.set_local(id, 0);
    true
}

pub(crate) const SPOON_SAVE: i32 = 1;
pub(crate) fn power_attempt(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _power: &str, _amount: i32, _allowed: &mut bool) -> bool { false }
pub(crate) fn would_die(_id: RelicId, _context: &mut CombatItemContext, _max_hp: i32, _revive: &mut Option<i32>) -> bool { false }
pub(crate) fn heal_amount(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn public_view(id: RelicId, _context: &CombatItemContext, rules: &mut PublicViewRules) -> bool {
    match id {
        RelicId::RunicDome => rules.hide_intents = true,
        RelicId::FrozenEye => rules.reveal_draw_order = true,
        _ => return false,
    }
    true
}

pub(crate) fn unblocked_outgoing_damage(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn player_vulnerable_factor(_id: RelicId, _context: &CombatItemContext, _factor: &mut f32) -> bool { false }

/// Run event work of this lane's relics.
pub(crate) fn run_event(_id: RelicId, _state: &mut RunState, _event: &mut RunItemEvent) -> Result<bool, RunError> { Ok(false) }
/// Obtain-time (equip) work of this lane's relics (relic research §3.1 latches; design §7.1).
pub(crate) fn equip(id: RelicId, state: &mut RunState, work: &mut EquipWork) -> Result<bool, RunError> {
    match id {
        // One-card selection over the purgeable cards of the bottle's type, basics included
        // (relic research §3.8: the spawn's nonbasic check is not the equipment filter).
        RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado => {
            let card_type = bottle_type(id).expect("bottle relic");
            work.select(SelectionSpec { eligible: purgeable(state, Some(card_type)), count: 1, cancellable: false });
        }
        // Empty group opens nothing; three or fewer are all processed; otherwise choose three.
        RelicId::Astrolabe => work.select(SelectionSpec { eligible: purgeable(state, None), count: 3, cancellable: false }),
        // Master-deck one-card selection.
        RelicId::DollysMirror => work.select(SelectionSpec {
            eligible: state.deck.iter().map(|card| card.instance).collect(), count: 1, cancellable: false }),
        // Curse of the Bell first, then one screenless Common, Uncommon and Rare relic offer. The
        // screen's own card row is generated and cleared (CallingBell.java:43–51; M31 G1(b)).
        RelicId::CallingBell => {
            crate::run::mutations::obtain_card_or_negated(state, CardId::CurseOfTheBell, 0)?;
            work.offer(RewardSpec { relic_tiers: vec![RelicTier::Common, RelicTier::Uncommon, RelicTier::Rare],
                ..RewardSpec::default() });
        }
        // Every starter-tagged Strike and Defend is transformed (no upgrade); no selection.
        RelicId::PandorasBox => {
            let starters: Vec<u64> = state.deck.iter().filter(|card| crate::content::card_tags(card.id).iter()
                .any(|tag| matches!(tag, crate::content::CardTag::StarterStrike | crate::content::CardTag::StarterDefend)))
                .map(|card| card.instance).collect();
            for instance in starters {
                let key = run_key(state, id, instance);
                crate::run::mutations::transform_card_or_negated(state, instance, false, &key)?;
            }
        }
        // Random upgrade of one upgradable card and max HP +5 at equip; 50 gold, one flat potion
        // and one ordinary card reward stay offers until claimed (research §4.5 Tiny House).
        RelicId::TinyHouse => {
            let upgradable: Vec<u64> = state.deck.iter().filter(|card| crate::run::mutations::can_upgrade(state, card))
                .map(|card| card.instance).collect();
            if !upgradable.is_empty() {
                let key = run_key(state, id, 0);
                let at = state.rng.bounded_scoped_checked(RunRngScope::M26CardTransform, &key, upgradable.len() as u64)?;
                crate::run::mutations::upgrade_card(state, upgradable[at])?;
            }
            crate::run::mutations::increase_max_hp(state, TINY_HOUSE_MAX_HP)?;
            let potions = flat_potions(state, id, 1)?;
            work.offer(RewardSpec { card_rewards: 1, potions, gold: TINY_HOUSE_GOLD, ..RewardSpec::default() });
        }
        // Five flat-pool potion offers; the automatically generated card row is removed
        // (Cauldron.java:24–39; M31 G1(b): its draws still move the Standard offset).
        RelicId::Cauldron => {
            let potions = flat_potions(state, id, CAULDRON_POTIONS)?;
            work.offer(RewardSpec { potions, ..RewardSpec::default() });
        }
        // Four explicit card rewards plus the screen's ordinary one: five separate rows.
        RelicId::Orrery => work.offer(RewardSpec { card_rewards: ORRERY_CARD_REWARDS, ..RewardSpec::default() }),
        // Pickup adds Necronomicurse.
        RelicId::Necronomicon => { crate::run::mutations::obtain_card_or_negated(state, CardId::Necronomicurse, 0)?; }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Completion of a selection this lane's relic opened.
pub(crate) fn selection_done(id: RelicId, state: &mut RunState, chosen: &[u64]) -> Result<bool, RunError> {
    let owned = |state: &RunState, instance: &u64| state.deck.iter().find(|card| card.instance == *instance).cloned();
    match id {
        // Store the selected permanent instance (an inventory link, not a card flag).
        RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado => {
            let [instance] = chosen else { return Err(RunError::InvalidAction) };
            let card = owned(state, instance).ok_or(RunError::InvalidAction)?;
            if !bottle_eligible(id, card.id) { return Err(RunError::InvalidAction) }
            state.relic_cards.insert(id, *instance);
        }
        // Each chosen card is transformed once with the upgraded-transform flag.
        RelicId::Astrolabe => {
            if chosen.is_empty() || chosen.len() > 3 || chosen.iter().any(|instance| owned(state, instance).is_none()) {
                return Err(RunError::InvalidAction)
            }
            for instance in chosen {
                let key = run_key(state, id, *instance);
                crate::run::mutations::transform_card_or_negated(state, *instance, true, &key)?;
            }
        }
        // One stat-equivalent copy (id, upgrades, misc); a new instance is never bottled.
        RelicId::DollysMirror => {
            let [instance] = chosen else { return Err(RunError::InvalidAction) };
            crate::run::mutations::duplicate_card_or_negated(state, *instance)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Whether `instance` is the permanent card linked by an owned bottle: `initializeDeck` places it
/// on top with the Innate cards (relic research §3.8). A copy has a new instance and is not bottled.
pub(crate) fn bottled(context: &CombatItemContext, instance: u64) -> bool {
    [RelicId::BottledFlame, RelicId::BottledLightning, RelicId::BottledTornado].into_iter()
        .any(|id| context.owns_relic(id) && context.relic_card(id) == Some(instance))
}

const TINY_HOUSE_MAX_HP: u32 = 5;
const TINY_HOUSE_GOLD: u32 = 50;
const CAULDRON_POTIONS: usize = 5;
const ORRERY_CARD_REWARDS: usize = 5;

/// Whether `card` may be bottled by `bottle` (the pickup selection's rule: a purgeable card of the bottle's type).
pub(crate) fn bottle_eligible(bottle: RelicId, card: CardId) -> bool {
    crate::run::mutations::removable(card) && card_type(card).ok().is_some_and(|t| Some(t) == bottle_type(bottle))
}

fn bottle_type(id: RelicId) -> Option<CardType> {
    match id {
        RelicId::BottledFlame => Some(CardType::Attack),
        RelicId::BottledLightning => Some(CardType::Skill),
        RelicId::BottledTornado => Some(CardType::Power),
        _ => None,
    }
}

fn card_type(id: CardId) -> Result<CardType, RunError> {
    crate::content::Catalog::standard().card(id).map(|card| card.card_type).ok_or(RunError::InvalidAction)
}

/// `getPurgeableCards` master-deck instances in deck order, optionally of one type. Astrolabe and the
/// bottles apply no bottle filter (source check §E), so this is `mutations::removable` alone.
fn purgeable(state: &RunState, card_type: Option<CardType>) -> Vec<u64> {
    state.deck.iter().filter(|card| crate::run::mutations::removable(card.id)
        && card_type.is_none_or(|wanted| crate::content::Catalog::standard().card(card.id)
            .is_some_and(|definition| definition.card_type == wanted)))
        .map(|card| card.instance).collect()
}

/// Semantic RNG key of one equip mutation: act, floor, relic, subject.
fn run_key(state: &RunState, id: RelicId, subject: u64) -> [u64; 5] {
    [u64::from(state.act), u64::from(state.floor), RELICS.iter().position(|relic| *relic == id).unwrap_or(0) as u64,
        subject, 0x5127]
}

/// Flat (1/33) proposals from the character's potion pool (design §8.5).
fn flat_potions(state: &mut RunState, id: RelicId, count: usize) -> Result<Vec<PotionId>, RunError> {
    let character = state.character;
    (0..count).map(|index| {
        let key = run_key(state, id, index as u64);
        super::super::potion_generation::flat(character, |bound| {
            Ok(state.rng.bounded_scoped_checked(RunRngScope::M27PotionOffer, &key, bound)? as usize)
        })
    }).collect()
}
