//! Lane R3 (M27): Run-owned relic bodies (plan Task R3).
//!
//! Every new effect is gated on the complete item capability. Frozen profiles
//! continue through their existing owners and never reach these rules.
#![allow(dead_code)]
use crate::combat::{Actor, Effect, EffectKind, FullPowerId, InputError, Source};
use crate::content::{CardType, Character, RelicId};
use crate::run::{ItemAdmission, RunError, RunRngScope, RunState};

use super::super::dispatch::{ItemPhase, PhaseCard, PhaseFacts, RunItemEvent};
use super::super::laws::{PlayRule, PublicViewRules};
use super::super::lifecycle::EquipWork;
use super::super::{CombatItemContext, RelicHookWork};

/// The relics this lane owns (design Appendix A, Lane R3).
pub(crate) const RELICS: &[RelicId] = &[
    RelicId::AncientTeaSet, RelicId::CeramicFish, RelicId::DreamCatcher, RelicId::JuzuBracelet,
    RelicId::MawBank, RelicId::MealTicket, RelicId::Omamori, RelicId::RegalPillow, RelicId::SmilingMask,
    RelicId::TinyChest, RelicId::WarPaint, RelicId::Whetstone, RelicId::TheCourier,
    RelicId::DarkstonePeriapt, RelicId::EternalFeather, RelicId::FrozenEgg, RelicId::Matryoshka,
    RelicId::MoltenEgg, RelicId::QuestionCard, RelicId::SingingBowl, RelicId::ToxicEgg,
    RelicId::WhiteBeastStatue, RelicId::Girya, RelicId::OldCoin, RelicId::PeacePipe, RelicId::PrayerWheel,
    RelicId::Shovel, RelicId::WingBoots, RelicId::BlackStar, RelicId::BustedCrown, RelicId::CoffeeDripper,
    RelicId::CursedKey, RelicId::Ectoplasm, RelicId::FusionHammer, RelicId::MembershipCard,
    RelicId::LeesWaffle, RelicId::BloodyIdol, RelicId::Circlet, RelicId::GoldenIdol, RelicId::NlothsGift,
    RelicId::NlothsHungryFace, RelicId::SpiritPoop, RelicId::SsserpentHead,
];

fn owned_complete(id: RelicId, context: &CombatItemContext) -> bool {
    RELICS.contains(&id) && context.complete_items()
}

fn complete_run(state: &RunState) -> bool { state.item_admission == ItemAdmission::Complete }

fn player_effect(id: RelicId, kind: EffectKind) -> Effect {
    Effect { source: Source::Relic(id), target: Some(Actor::Player), kind }
}

/// Combat phase work of this lane's relics. Returns whether `id` is this lane's relic.
pub(crate) fn phase(id: RelicId, context: &mut CombatItemContext, phase: ItemPhase, facts: &PhaseFacts<'_>,
    work: &mut RelicHookWork) -> Result<bool, InputError> {
    if !owned_complete(id, context) { return Ok(false); }
    match (id, phase) {
        (RelicId::AncientTeaSet, ItemPhase::TurnStart) if facts.target == Some(&Actor::Player)
            && context.counter(id) == 1 => {
                context.set_counter(id, 0);
                work.top.push(player_effect(id, EffectKind::GainEnergy(2)));
            }
        (RelicId::Girya, ItemPhase::CombatStart) => {
            let lifts = context.counter(id).clamp(0, 3);
            if lifts > 0 {
                work.bottom.push(player_effect(id, EffectKind::ApplyFullPower {
                    id: FullPowerId("strength"), amount: lifts,
                }));
            }
        }
        _ => (),
    }
    Ok(true)
}
pub(crate) fn outgoing_damage(_id: RelicId, _context: &CombatItemContext, _card: Option<PhaseCard>, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn incoming_damage(_id: RelicId, _context: &CombatItemContext, _attacker: Option<&Actor>, _amount: &mut i32) -> bool { false }
pub(crate) fn hp_loss(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn block_decay(_id: RelicId, _context: &CombatItemContext, _current: i32, _kept: &mut i32) -> bool { false }
pub(crate) fn recharge(_id: RelicId, _context: &CombatItemContext, _character: Character, _unspent: i32, _energy: &mut i32) -> bool { false }
pub(crate) fn master_energy(_id: RelicId, _context: &CombatItemContext, _elite_or_boss: bool, _bonus: &mut i32) -> bool { false }
pub(crate) fn play_allowed(_id: RelicId, _context: &CombatItemContext, _card: PhaseCard, _rule: &mut PlayRule) -> bool { false }
pub(crate) fn repeat_play(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _extra: &mut u32) -> bool { false }
pub(crate) fn would_exhaust(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _exhaust: &mut bool) -> bool { false }
pub(crate) fn power_attempt(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _power: &str, _amount: i32, _allowed: &mut bool) -> bool { false }
pub(crate) fn would_die(_id: RelicId, _context: &mut CombatItemContext, _max_hp: i32, _revive: &mut Option<i32>) -> bool { false }
pub(crate) fn heal_amount(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn public_view(_id: RelicId, _context: &CombatItemContext, _rules: &mut PublicViewRules) -> bool { false }
pub(crate) fn unblocked_outgoing_damage(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn player_vulnerable_factor(_id: RelicId, _context: &CombatItemContext, _factor: &mut f32) -> bool { false }

fn scale_price(price: u32, percent: u32) -> u32 {
    ((u64::from(price) * u64::from(percent) + 50) / 100).min(u64::from(u32::MAX)) as u32
}

fn is_curse(id: crate::content::CardId) -> bool {
    crate::content::Catalog::standard().card(id).is_some_and(|definition| definition.card_type == CardType::Curse)
}

fn reward_has_card(room: super::super::run_laws::RewardRoom) -> bool {
    use super::super::run_laws::RewardRoom;
    matches!(room, RewardRoom::Elite | RewardRoom::Boss
        | RewardRoom::Ordinary { enemy_escaped: false })
}

fn upgrade_random_kind(state: &mut RunState, relic: RelicId, card_type: CardType) -> Result<(), RunError> {
    let mut staged = state.clone();
    let mut eligible = staged.deck.iter().filter(|card| {
        crate::content::Catalog::standard().card(card.id).is_some_and(|definition| definition.card_type == card_type)
            && crate::combat::cards::ironclad::rules::can_upgrade(card.id, card.upgrades)
    }).map(|card| card.instance).collect::<Vec<_>>();
    // A unique semantic key keeps these equip draws independent of card reward rows.
    // M26CardOffer is the registered shared card RNG lane in this base revision.
    let floor = u64::from(staged.floor);
    let relic_key = relic.as_str().bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    for i in (1..eligible.len()).rev() {
        let key = [0x4D27, floor, u64::from(staged.act), relic_key, card_type as u64, i as u64];
        let selected = staged.rng.bounded_scoped_checked(RunRngScope::M26CardOffer, &key, (i + 1) as u64)?;
        eligible.swap(i, selected);
    }
    for instance in eligible.into_iter().take(2) {
        crate::run::mutations::upgrade_card(&mut staged, instance)?;
    }
    *state = staged;
    Ok(())
}

/// Run event work of this lane's relics.
pub(crate) fn run_event(id: RelicId, state: &mut RunState, event: &mut RunItemEvent) -> Result<bool, RunError> {
    if !RELICS.contains(&id) || !complete_run(state) || id == RelicId::Circlet { return Ok(false); }
    match (id, event) {
        (RelicId::AncientTeaSet, RunItemEvent::RoomEntered { kind: crate::run::RoomViewKind::Campfire }) => {
            state.relic_counters.entry(id).or_insert(1);
        }
        (RelicId::CeramicFish, RunItemEvent::CardObtained { .. }) => {
            crate::run::mutations::gain_gold(state, 9)?;
        }
        (RelicId::MawBank, RunItemEvent::FloorEntered { .. }) if !state.relic_counters.contains_key(&id) => {
            crate::run::mutations::gain_gold(state, 12)?;
        }
        (RelicId::MawBank, RunItemEvent::ShopPurchase) => { state.relic_counters.insert(id, 1); }
        (RelicId::MealTicket, RunItemEvent::ShopEntered) => crate::run::mutations::heal(state, 15)?,
        (RelicId::RegalPillow, RunItemEvent::RestHeal { amount }) => *amount = amount.saturating_add(15),
        (RelicId::EternalFeather, RunItemEvent::RoomEntered { kind: crate::run::RoomViewKind::Campfire }) => {
            let amount = (state.deck.len() / 5).saturating_mul(3).min(u32::MAX as usize) as u32;
            crate::run::mutations::heal(state, amount)?;
        }
        (RelicId::SsserpentHead, RunItemEvent::RoomEntered { kind: crate::run::RoomViewKind::Event(_) }) => {
            crate::run::mutations::gain_gold(state, 50)?;
        }
        (RelicId::Omamori, RunItemEvent::CardObtaining { id: card, negated, .. }) if is_curse(*card) && !*negated => {
            let charges = state.relic_counters.get(&id).copied().unwrap_or(0);
            if charges > 0 {
                *negated = true;
                state.relic_counters.insert(id, charges - 1);
            }
        }
        (RelicId::FrozenEgg | RelicId::MoltenEgg | RelicId::ToxicEgg,
            RunItemEvent::CardObtaining { id: card, upgrades, .. }) if *upgrades == 0 => {
                let kind = crate::content::Catalog::standard().card(*card).map(|definition| definition.card_type);
                if kind == Some(egg_card_type(id)) { *upgrades = 1; }
            }
        (RelicId::DarkstonePeriapt, RunItemEvent::CardObtained { id: card, .. }) if is_curse(*card) => {
            crate::run::mutations::increase_max_hp(state, 6)?;
        }
        (RelicId::Ectoplasm, RunItemEvent::GoldGaining { amount }) if *amount > 0 => *amount = 0,
        (RelicId::BloodyIdol, RunItemEvent::GoldGained { amount }) if *amount > 0 => {
            crate::run::mutations::heal(state, 5)?;
        }
        // Source applies The Courier's pass, then Membership Card's, whatever the acquisition
        // order (economy research :172); the Courier arm runs both when both are owned.
        (RelicId::TheCourier, RunItemEvent::ShopPrice { price }) => {
            *price = scale_price(*price, 80);
            if state.relics.contains(&RelicId::MembershipCard) { *price = scale_price(*price, 50); }
        }
        (RelicId::MembershipCard, RunItemEvent::ShopPrice { price })
            if !state.relics.contains(&RelicId::TheCourier) => *price = scale_price(*price, 50),
        (RelicId::SmilingMask, RunItemEvent::RemovalPrice { price }) => *price = 50,
        // Source applyDiscount recomputes removal from its base, so Membership Card's pass
        // supersedes The Courier's at shop entry (economy research §3.2: base 75 -> 38).
        (RelicId::TheCourier, RunItemEvent::RemovalPrice { price })
            if !state.relics.contains(&RelicId::SmilingMask) && !state.relics.contains(&RelicId::MembershipCard) =>
            *price = scale_price(*price, 80),
        (RelicId::MembershipCard, RunItemEvent::RemovalPrice { price })
            if !state.relics.contains(&RelicId::SmilingMask) => *price = scale_price(*price, 50),
        (RelicId::DreamCatcher, RunItemEvent::Rested { card_reward }) => *card_reward = true,
        (RelicId::CoffeeDripper, RunItemEvent::CampfireOptions { options }) => {
            options.retain(|choice| !matches!(choice, crate::run::RoomChoiceKind::Rest { .. }));
        }
        (RelicId::FusionHammer, RunItemEvent::CampfireOptions { options }) => {
            options.retain(|choice| *choice != crate::run::RoomChoiceKind::Smith);
        }
        (RelicId::Girya, RunItemEvent::CampfireOptions { options })
            if state.relic_counters.get(&id).copied().unwrap_or(0) < 3 => {
                if !options.contains(&crate::run::RoomChoiceKind::Lift) { options.push(crate::run::RoomChoiceKind::Lift); }
            }
        (RelicId::PeacePipe, RunItemEvent::CampfireOptions { options }) => {
            if !options.contains(&crate::run::RoomChoiceKind::Toke) { options.push(crate::run::RoomChoiceKind::Toke); }
        }
        (RelicId::Shovel, RunItemEvent::CampfireOptions { options }) => {
            if !options.contains(&crate::run::RoomChoiceKind::Dig) { options.push(crate::run::RoomChoiceKind::Dig); }
        }
        // Every combat gold reward except a TreasureRoom's (economy research §2.3 "Ordinary gold reward aggregation").
        (RelicId::GoldenIdol, RunItemEvent::RewardsBuilding { room, gold_percent, .. })
            if *room != super::super::run_laws::RewardRoom::Ordinary { enemy_escaped: true } => {
            *gold_percent = gold_percent.saturating_add(25);
        }
        (RelicId::BlackStar, RunItemEvent::RewardsBuilding { room: super::super::run_laws::RewardRoom::Elite, extra_elite_relics, .. }) => {
            *extra_elite_relics = extra_elite_relics.saturating_add(1);
        }
        (RelicId::BustedCrown, RunItemEvent::RewardsBuilding { room, card_choices, .. }) if reward_has_card(*room) => {
            *card_choices = card_choices.saturating_sub(2).max(1);
        }
        (RelicId::QuestionCard, RunItemEvent::RewardsBuilding { room, card_choices, .. }) if reward_has_card(*room) => {
            *card_choices = card_choices.saturating_add(1);
        }
        (RelicId::PrayerWheel, RunItemEvent::RewardsBuilding { room: super::super::run_laws::RewardRoom::Ordinary { enemy_escaped: false }, extra_card_rewards, .. }) => {
            *extra_card_rewards = extra_card_rewards.saturating_add(1);
        }
        (RelicId::SingingBowl, RunItemEvent::RewardsBuilding { room, singing_bowl, .. }) if reward_has_card(*room) => *singing_bowl = true,
        (RelicId::WhiteBeastStatue, RunItemEvent::RewardsBuilding { room: super::super::run_laws::RewardRoom::Ordinary { enemy_escaped: false } | super::super::run_laws::RewardRoom::Elite, include_potion, .. }) => *include_potion = true,
        _ => (),
    }
    Ok(true)
}

/// Obtain-time (equip) work of this lane's relics.
pub(crate) fn equip(id: RelicId, state: &mut RunState, _work: &mut EquipWork) -> Result<bool, RunError> {
    if !RELICS.contains(&id) || !complete_run(state) || id == RelicId::Circlet { return Ok(false); }
    match id {
        RelicId::Omamori => { state.relic_counters.entry(id).or_insert(2); }
        RelicId::Matryoshka => { state.relic_counters.entry(id).or_insert(2); }
        RelicId::WingBoots => { state.relic_counters.entry(id).or_insert(3); }
        RelicId::Girya | RelicId::TinyChest => { state.relic_counters.entry(id).or_insert(0); }
        RelicId::OldCoin => crate::run::mutations::gain_gold(state, 300)?,
        // Source increaseMaxHp(7, false), then heal(maxHealth) through the one heal law (design §9.1):
        // Mark of the Bloom keeps the maximum-HP gain but prevents the heal.
        RelicId::LeesWaffle => {
            state.max_hp = state.max_hp.checked_add(7).filter(|max_hp| *max_hp <= 999)
                .ok_or(RunError::Unsupported("maximum HP overflow"))?;
            crate::run::mutations::heal(state, state.max_hp)?;
        }
        // Source FrozenEgg2/MoltenEgg2/ToxicEgg2.onEquip apply the obtain rule to every card of every
        // card reward already on the reward screen (PM source check 2026-11-02, m4).
        RelicId::FrozenEgg | RelicId::MoltenEgg | RelicId::ToxicEgg => {
            let wanted = egg_card_type(id);
            let catalog = crate::content::Catalog::standard();
            if let Some(captured) = state.room_loop.as_mut().and_then(|room| room.active_rewards.as_mut()) {
                for choice in &mut captured.cards.choices {
                    if let crate::run::RoomChoiceKind::Card { id: card, upgrades } = &mut choice.kind {
                        if *upgrades == 0 && catalog.card(*card).map(|definition| definition.card_type) == Some(wanted) {
                            *upgrades = 1;
                        }
                    }
                }
            }
        }
        RelicId::WarPaint => upgrade_random_kind(state, id, CardType::Skill)?,
        RelicId::Whetstone => upgrade_random_kind(state, id, CardType::Attack)?,
        _ => (),
    }
    Ok(true)
}

fn egg_card_type(id: RelicId) -> CardType {
    match id { RelicId::FrozenEgg => CardType::Power, RelicId::MoltenEgg => CardType::Attack, _ => CardType::Skill }
}

/// Completion of a selection this lane's relic opened.
pub(crate) fn selection_done(_id: RelicId, _state: &mut RunState, _chosen: &[u64]) -> Result<bool, RunError> { Ok(false) }

// Run entry points for rooms M28 completes (items/run_laws.rs, design §11.3).
use super::super::run_laws::{CardRewardShape, ChestRules, MapPathRules, RewardRoom, UnknownRoomRules};
pub(crate) fn unknown_room(id: RelicId, state: &mut RunState, rules: &mut UnknownRoomRules) -> bool {
    if !RELICS.contains(&id) || !complete_run(state) { return false; }
    match id {
        RelicId::JuzuBracelet => rules.no_monsters = true,
        // Catalog TINY_CHEST: each qualifying room roll increments the counter; the fourth resets
        // it to 0 and overrides the result to Treasure.
        RelicId::TinyChest => {
            let counter = state.relic_counters.get(&id).copied().unwrap_or(0) + 1;
            if counter >= 4 {
                state.relic_counters.insert(id, 0);
                rules.force_treasure = true;
            } else {
                state.relic_counters.insert(id, counter);
            }
        }
        _ => return false,
    }
    true
}
pub(crate) fn map_path(id: RelicId, state: &RunState, rules: &mut MapPathRules) -> bool {
    if id != RelicId::WingBoots || !complete_run(state) { return false; }
    rules.off_path_moves = state.relic_counters.get(&id).copied().unwrap_or(0).clamp(0, 3) as u32;
    true
}
pub(crate) fn card_reward_shape(id: RelicId, state: &RunState, room: RewardRoom, shape: &mut CardRewardShape) -> bool {
    if !RELICS.contains(&id) || !complete_run(state) { return false; }
    match id {
        RelicId::QuestionCard if reward_has_card(room) => shape.choices = shape.choices.saturating_add(1),
        RelicId::BustedCrown if reward_has_card(room) => shape.choices = shape.choices.saturating_sub(2).max(1),
        RelicId::PrayerWheel if matches!(room, RewardRoom::Ordinary { enemy_escaped: false }) => shape.rewards = shape.rewards.saturating_add(1),
        RelicId::NlothsGift if reward_has_card(room) => shape.rare_factor = shape.rare_factor.saturating_mul(3),
        _ => return false,
    }
    true
}
pub(crate) fn chest_contents(id: RelicId, state: &mut RunState, boss: bool, rules: &mut ChestRules) -> Result<bool, RunError> {
    if !RELICS.contains(&id) || !complete_run(state) { return Ok(false); }
    match id {
        RelicId::CursedKey if !boss => rules.curse = true,
        // Catalog MATRYOSHKA: while a charge remains, a non-Boss chest gains one extra relic,
        // Common 75% / Uncommon 25% (relic RNG), and spends the charge.
        RelicId::Matryoshka if !boss && state.relic_counters.get(&id).copied().unwrap_or(0) > 0 => {
            let remaining = state.relic_counters.get(&id).copied().unwrap_or(0);
            let key = [0x4D27_C4E5, u64::from(state.act), u64::from(state.floor), remaining as u64];
            let tier = if state.rng.bounded_scoped_checked(RunRngScope::M27RelicTier, &key, 100)? < 75 {
                crate::content::RelicTier::Common
            } else { crate::content::RelicTier::Uncommon };
            let relic = super::super::relic_pools::take(state, tier, super::super::relic_pools::Selector::Front,
                super::super::lifecycle::RelicSource::Chest, &key)?;
            state.relic_counters.insert(id, remaining - 1);
            rules.extra_relic = Some(relic);
        }
        // PM source check 2026-11-02 (m7): the after-open callback of the next non-Boss chest removes
        // the first relic reward and a Sapphire Key linked to it; the single use is then spent.
        RelicId::NlothsHungryFace if !boss && !state.relic_counters.contains_key(&id) => {
            rules.remove_first_relic = true;
            state.relic_counters.insert(id, 1);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
pub(crate) fn score_adjustment(id: RelicId, state: &RunState, score: &mut i32) -> bool {
    if id != RelicId::SpiritPoop || !complete_run(state) { return false; }
    *score = score.saturating_sub(1);
    true
}
