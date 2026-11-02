//! Standard Neow (design §6.5): the short pair unchanged and the full four-category blessing.
use super::*;
use crate::{
    content::{pools::{self, CardPool}, CardRarity, PotionId, RelicTier},
    items::{self, lifecycle::{RelicObtained, RelicSource}},
    run::{NeowDrawback, NeowReward, NeowSelectOp, RunRngScope, RoomSelectionOperation},
};

const NEOW_SOURCE_TAG: u64 = 0x4E45_4F57; // "NEOW"; isolates payloads on their owning scope.
const RARITY_GRID: u64 = 1 << 24;
const UNCOMMON_GRID_CUTOFF: u64 = 5_536_482;

/// One choice row per Neow candidate, in candidate order.
pub(in crate::run) fn screen(run: &Run) -> Result<Vec<RoomChoiceView>, RunError> {
    run.candidates.iter().map(|candidate| {
        let offer = candidate.neow_offer.as_ref().ok_or(RunError::InvalidAction)?;
        let kind = if run.state.neow_eligible {
            RoomChoiceKind::NeowBlessing { category: offer.category, reward: offer.reward, drawback: offer.drawback }
        } else {
            match offer.reward {
                NeowReward::Lament => RoomChoiceKind::NeowLament,
                _ => RoomChoiceKind::NeowMaxHp { amount: (run.state.max_hp as f32 * 0.1_f32) as u32 },
            }
        };
        Ok(RoomChoiceView { id: candidate.id, kind, price: None, status: RoomChoiceStatus::Offered })
    }).collect()
}

/// Neow owns its initial screen and the child screens that remain before the first map move.
/// Each action works on a clone, so a failed obtain or continuation leaves the live Run untouched.
pub(in crate::run) fn choose(run: &mut Run, id: ChoiceId) -> Result<RunTransition, RunError> {
    let screen = crate::run::rooms::public(run)?;
    let row = screen.choices.iter().find(|row| row.id == id && row.status == RoomChoiceStatus::Offered)
        .cloned().ok_or(RunError::InvalidAction)?;
    if screen.kind == RoomViewKind::Neow {
        return choose_initial(run, id, row.kind);
    }
    if run.state.selected_neow.is_none() { return Err(RunError::InvalidAction); }
    match (screen.kind, row.kind) {
        (RoomViewKind::CardReward, RoomChoiceKind::Card { id: card, upgrades }) => {
            let mut staged = run.clone();
            mutations::obtain_card_or_negated(&mut staged.state, card, upgrades)?;
            finish(&mut staged, id)?;
            *run = staged;
            Ok(empty_transition())
        }
        (RoomViewKind::CardReward, RoomChoiceKind::Leave) => {
            let mut staged = run.clone();
            finish(&mut staged, id)?;
            *run = staged;
            Ok(empty_transition())
        }
        (RoomViewKind::Rewards, RoomChoiceKind::Potion(potion)) => {
            let mut staged = run.clone();
            if !items::obtain_run_potion(&mut staged.state, potion)? { return Err(RunError::InvalidAction); }
            set_row_status(&mut staged, id, RoomChoiceStatus::Claimed)?;
            *run = staged;
            Ok(empty_transition())
        }
        (RoomViewKind::Rewards, RoomChoiceKind::Leave) => {
            let mut staged = run.clone();
            finish(&mut staged, id)?;
            *run = staged;
            Ok(empty_transition())
        }
        _ => Err(RunError::InvalidAction),
    }
}

fn choose_initial(run: &mut Run, id: ChoiceId, kind: RoomChoiceKind) -> Result<RunTransition, RunError> {
    let mut staged = run.clone();
    let (amount, is_short) = match kind {
        RoomChoiceKind::NeowMaxHp { amount } => (Some(amount), true),
        RoomChoiceKind::NeowLament => (None, true),
        _ => (None, false),
    };
    if is_short {
        let mut obtained = None;
        if let Some(amount) = amount {
            // The short blessing keeps its established operation and draw behavior.
            mutations::increase_max_hp(&mut staged.state, amount)?;
        } else {
            obtained = Some((RelicId::NeowsLament, items::obtain_relic(&mut staged.state, RelicId::NeowsLament, RelicSource::Neow)?));
        }
        finish(&mut staged, id)?;
        let transition = if let Some((relic, obtained)) = obtained {
            crate::run::rooms::after_obtain(&mut staged, id, relic, obtained)?
        } else { empty_transition() };
        *run = staged;
        return Ok(transition);
    }

    let offer = staged.candidates.iter().find(|candidate| candidate.id == id)
        .and_then(|candidate| candidate.neow_offer.clone()).ok_or(RunError::InvalidAction)?;
    if kind != (RoomChoiceKind::NeowBlessing { category: offer.category, reward: offer.reward, drawback: offer.drawback })
        || !staged.state.neow_eligible {
        return Err(RunError::InvalidAction);
    }

    // No action can change M between Neow construction and choice; this is the captured b at activation.
    let b = (staged.state.max_hp as f32 * 0.1_f32) as u32;
    staged.state.selected_neow = Some(offer.clone()); // Commit point, retained through child screens.
    staged.candidates.clear();

    match offer.drawback {
        Some(NeowDrawback::MaxHpLoss) => mutations::decrease_max_hp(&mut staged.state, b),
        Some(NeowDrawback::NoGold) => staged.state.gold = 0,
        Some(NeowDrawback::CurrentHpLoss) => {
            let loss = 3 * (staged.state.hp / 10);
            // A0 begins at H >= 70, so this source HP_LOSS request cannot kill the player.
            staged.state.hp = staged.state.hp.saturating_sub(loss);
        }
        Some(NeowDrawback::RandomCurse) | None => (),
    }

    let mut obtained_relic: Option<(RelicId, RelicObtained)> = None;
    let opens_child = match offer.reward {
        NeowReward::ThreeCards => {
            let cards = draw_card_row(&mut staged.state, CardRow::Ordinary)?;
            open_card_child(&mut staged, cards)?;
            true
        }
        NeowReward::ThreeRareCards => {
            let cards = draw_card_row(&mut staged.state, CardRow::OrdinaryRare)?;
            open_card_child(&mut staged, cards)?;
            true
        }
        NeowReward::ThreeUncommonColorless => {
            let cards = draw_card_row(&mut staged.state, CardRow::UncommonColorless)?;
            open_card_child(&mut staged, cards)?;
            true
        }
        NeowReward::RareColorless => {
            let cards = draw_card_row(&mut staged.state, CardRow::RareColorless)?;
            open_card_child(&mut staged, cards)?;
            true
        }
        NeowReward::RandomRare => {
            let card = draw_single_rare(&mut staged.state)?;
            mutations::obtain_card_or_negated(&mut staged.state, card, 0)?;
            false
        }
        NeowReward::RemoveOne | NeowReward::RemoveTwo | NeowReward::UpgradeOne
        | NeowReward::TransformOne | NeowReward::TransformTwo => {
            let (op, count) = match offer.reward {
                NeowReward::RemoveOne => (NeowSelectOp::Remove, 1),
                NeowReward::RemoveTwo => (NeowSelectOp::Remove, 2),
                NeowReward::UpgradeOne => (NeowSelectOp::Upgrade, 1),
                NeowReward::TransformOne => (NeowSelectOp::Transform, 1),
                NeowReward::TransformTwo => (NeowSelectOp::Transform, 2),
                _ => unreachable!(),
            };
            crate::run::rooms::selection::open(&mut staged, id, RoomSelectionOperation::NeowSelect { op, count })?;
            true
        }
        NeowReward::ThreePotions => {
            let potions = draw_potions(&mut staged.state)?;
            // The ordinary reward screen generates its automatic card row and Neow removes it; the
            // generation's rarity-offset writes stay (event-neow research :113, :139).
            let _discarded = super::rewards::ordinary_offer(&mut staged.state, super::rewards::RarityRoom::Ordinary, 3, 1, &neow_key(0xCA, 0))?;
            open_potion_child(&mut staged, potions)?;
            true
        }
        NeowReward::CommonRelic | NeowReward::RareRelic => {
            let tier = if offer.reward == NeowReward::CommonRelic { RelicTier::Common } else { RelicTier::Rare };
            let key = neow_key(0, 0);
            let relic = crate::items::relic_pools::take(&mut staged.state, tier,
                crate::items::relic_pools::Selector::Front, RelicSource::Neow, &key)?;
            let obtained = items::lifecycle::obtain_relic(&mut staged.state, relic, RelicSource::Neow)?;
            obtained_relic = Some((relic, obtained));
            false
        }
        NeowReward::BossSwap => {
            let key = neow_key(0, 0);
            obtained_relic = Some(items::lifecycle::neow_boss_swap_obtained(&mut staged.state, &key)?);
            false
        }
        NeowReward::MaxHp => { mutations::increase_max_hp(&mut staged.state, b)?; false }
        NeowReward::MaxHpTwice => {
            let twice_b = b.checked_mul(2).ok_or(RunError::InvalidAction)?;
            mutations::increase_max_hp(&mut staged.state, twice_b)?;
            false
        }
        NeowReward::Lament => {
            obtained_relic = Some((RelicId::NeowsLament,
                items::lifecycle::obtain_relic(&mut staged.state, RelicId::NeowsLament, RelicSource::Neow)?));
            false
        }
        NeowReward::Gold => { mutations::gain_gold(&mut staged.state, 100)?; false }
        NeowReward::Gold250 => { mutations::gain_gold(&mut staged.state, 250)?; false }
    };

    // The selector snapshot is already committed (or the reward child is open) before this obtain.
    if offer.drawback == Some(NeowDrawback::RandomCurse) {
        let curse = mutations::draw_card(&mut staged.state, CardPool::Curse, &neow_key(0, 0))?;
        mutations::obtain_card_or_negated(&mut staged.state, curse, 0)?;
    }

    if opens_child {
        *run = staged;
        return Ok(empty_transition());
    }
    finish(&mut staged, id)?;
    let transition = if let Some((relic, obtained)) = obtained_relic {
        crate::run::rooms::after_obtain(&mut staged, id, relic, obtained)?
    } else { empty_transition() };
    *run = staged;
    Ok(transition)
}

/// The committed Neow selector finished (`rooms::selection::select_neow`): apply `op` to the
/// selected cards in selection order, then continue the blessing (design §5.6, §6.5).
pub(in crate::run) fn selection_done(run: &mut Run, choice: ChoiceId, op: NeowSelectOp,
    selected: Vec<CardInstance>) -> Result<RunTransition, RunError> {
    let offer = run.state.selected_neow.as_ref().ok_or(RunError::InvalidAction)?;
    let (expected_op, expected_count) = match offer.reward {
        NeowReward::RemoveOne => (NeowSelectOp::Remove, 1),
        NeowReward::RemoveTwo => (NeowSelectOp::Remove, 2),
        NeowReward::UpgradeOne => (NeowSelectOp::Upgrade, 1),
        NeowReward::TransformOne => (NeowSelectOp::Transform, 1),
        NeowReward::TransformTwo => (NeowSelectOp::Transform, 2),
        _ => return Err(RunError::InvalidAction),
    };
    if op != expected_op || selected.len() != expected_count || selected.iter().any(|card| !run.state.deck.contains(card)) {
        return Err(RunError::InvalidAction);
    }
    let mut staged = run.clone();
    match op {
        NeowSelectOp::Remove => {
            for card in &selected { mutations::remove_card(&mut staged.state, card.instance)?; }
        }
        NeowSelectOp::Upgrade => {
            if selected.len() != 1 { return Err(RunError::InvalidAction); }
            mutations::upgrade_card(&mut staged.state, selected[0].instance)?;
        }
        NeowSelectOp::Transform => {
            // N7: transform-two's remove+generate per input in selection order is D-equivalent to
            // removing both first, since no supported removal hook changes later transform support.
            for (slot, card) in selected.iter().enumerate() {
                mutations::transform_card_or_negated(&mut staged.state, card.instance, false,
                    &neow_key(slot as u64, 0))?;
            }
        }
    }
    finish(&mut staged, choice)?;
    *run = staged;
    Ok(empty_transition())
}

/// Close Neow: preserve the committed offer, generate Act 1 and expose the map.
pub(in crate::run) fn finish(run: &mut Run, id: ChoiceId) -> Result<(), RunError> {
    if let Some(offer) = run.candidates.iter().find(|candidate| candidate.id == id)
        .and_then(|candidate| candidate.neow_offer.clone()) {
        run.state.selected_neow = Some(offer);
    }
    super::start_act(run)?;
    run.state.room = "Map";
    run.candidates.clear();
    run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen = RoomView {
        kind: RoomViewKind::Map, choices: vec![],
    };
    Ok(())
}

#[derive(Clone, Copy)]
enum CardRow { Ordinary, OrdinaryRare, UncommonColorless, RareColorless }

/// Three-card payloads use M28EventBody keys; rarity attempts precede identity attempts.
fn draw_card_row(state: &mut RunState, row: CardRow) -> Result<Vec<crate::content::CardId>, RunError> {
    let mut cards = Vec::with_capacity(3);
    for slot in 0..3u64 {
        let rarity_roll = state.rng.bounded_scoped_checked(RunRngScope::M28EventBody,
            &neow_key(slot, 0), RARITY_GRID)?;
        let pool = match row {
            CardRow::Ordinary => {
                let rarity = if rarity_roll < UNCOMMON_GRID_CUTOFF as usize { CardRarity::Uncommon } else { CardRarity::Common };
                pools::cards(CardPool::Ordinary { character: state.character, rarity: Some(rarity), card_type: None })
            }
            CardRow::OrdinaryRare => {
                // The forced-rare row retains its per-slot rarity proposal draw, then overrides it.
                let _forced_rare = rarity_roll;
                pools::cards(CardPool::Ordinary { character: state.character, rarity: Some(CardRarity::Rare), card_type: None })
            }
            CardRow::UncommonColorless => {
                // A common proposal is promoted to uncommon; rare is outside this offer.
                let _promoted = rarity_roll;
                pools::cards(CardPool::Colourless { rarity: Some(CardRarity::Uncommon) })
            }
            CardRow::RareColorless => {
                let _forced_rare = rarity_roll;
                pools::cards(CardPool::Colourless { rarity: Some(CardRarity::Rare) })
            }
        }.map_err(|_| RunError::Unsupported("card pool is incomplete"))?;

        let mut attempt = 1u64;
        loop {
            let index = state.rng.bounded_scoped_checked(RunRngScope::M28EventBody,
                &neow_key(slot, attempt), pool.len() as u64)?;
            let card = pool[index];
            if !cards.contains(&card) {
                cards.push(card);
                break;
            }
            attempt = attempt.checked_add(1).ok_or(RunError::Unsupported("Neow card retry exhausted"))?;
        }
    }
    Ok(cards)
}

fn draw_single_rare(state: &mut RunState) -> Result<crate::content::CardId, RunError> {
    let pool = pools::cards(CardPool::Ordinary { character: state.character, rarity: Some(CardRarity::Rare), card_type: None })
        .map_err(|_| RunError::Unsupported("card pool is incomplete"))?;
    let index = state.rng.bounded_scoped_checked(RunRngScope::M28EventBody, &neow_key(0, 0), pool.len() as u64)?;
    Ok(pool[index])
}

fn draw_potions(state: &mut RunState) -> Result<Vec<PotionId>, RunError> {
    (0..3u64).map(|slot| {
        crate::items::potion_generation::flat(state.character, |bound| {
            state.rng.bounded_scoped_checked(RunRngScope::M27PotionOffer, &neow_key(slot, 0), bound)
        })
    }).collect()
}

fn open_card_child(run: &mut Run, cards: Vec<crate::content::CardId>) -> Result<(), RunError> {
    let rows = cards.into_iter().map(|id| (RoomChoiceKind::Card { id, upgrades: 0 }, None))
        .chain(std::iter::once((RoomChoiceKind::Leave, None))).collect();
    open_child(run, RoomViewKind::CardReward, rows)
}

fn open_potion_child(run: &mut Run, potions: Vec<PotionId>) -> Result<(), RunError> {
    // Interim M28a limit: the discarded ordinary card row and its rarity-offset writes land with M28b.
    let rows = potions.into_iter().map(|id| (RoomChoiceKind::Potion(id), None))
        .chain(std::iter::once((RoomChoiceKind::Leave, None))).collect();
    open_child(run, RoomViewKind::Rewards, rows)
}

fn open_child(run: &mut Run, kind: RoomViewKind, rows: Vec<(RoomChoiceKind, Option<u32>)>) -> Result<(), RunError> {
    let mut next = run.next_choice;
    let screen = crate::run::rooms::view(&mut next, kind, rows)?;
    let room = run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.screen = screen;
    room.reward_parent = None;
    room.active_rewards = None;
    run.next_choice = next;
    run.candidates.clear();
    Ok(())
}

fn set_row_status(run: &mut Run, id: ChoiceId, status: RoomChoiceStatus) -> Result<(), RunError> {
    run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.screen.choices.iter_mut()
        .find(|row| row.id == id).ok_or(RunError::InvalidAction)?.status = status;
    Ok(())
}

fn neow_key(slot: u64, attempt: u64) -> [u64; 7] {
    [1, 0, u64::MAX, 0, NEOW_SOURCE_TAG, slot, attempt]
}

fn empty_transition() -> RunTransition { RunTransition { reward: 0, terminal: None, committed_events: vec![] } }
