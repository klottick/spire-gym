//! Colorless choice, upgrade, and mutable-cost card rules.
use super::*;
use crate::combat::{
    command::CardDestination,
    model::OwnedCard,
    selection::{EmptySelection, SelectionDisposition, SelectionOperation, SelectionParent, SelectionSourceZone},
};
use crate::content::{CardId, CardType, Catalog};

pub(super) fn starting_cost(id: CardId, upgrades: u32) -> Option<i32> {
    Some(match id {
        CardId::Apotheosis => if upgrades > 0 { 1 } else { 2 },
        CardId::Madness => if upgrades > 0 { 0 } else { 1 },
        CardId::Enlightenment | CardId::Forethought | CardId::Purity
        | CardId::SecretTechnique | CardId::SecretWeapon | CardId::ThinkingAhead | CardId::Violence => 0,
        _ => return None,
    })
}

pub(super) fn requires_draw_history(id: CardId) -> bool {
    matches!(id, CardId::Apotheosis | CardId::Enlightenment | CardId::Forethought
        | CardId::Madness | CardId::Purity | CardId::SecretTechnique | CardId::SecretWeapon
        | CardId::ThinkingAhead | CardId::Violence)
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use CardId::*;
    if !matches!(card.id, Apotheosis | Enlightenment | Forethought | Madness | Purity
        | SecretTechnique | SecretWeapon | ThinkingAhead | Violence) {
        return Ok(None);
    }
    if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
    let up = card.upgrades > 0;
    let mut effects = Vec::new();
    match card.id {
        Apotheosis => effects.push((None, EffectKind::Colorless(ColorlessEffect::ApotheosisUpgrade))),
        Enlightenment => effects.push((None, EffectKind::Colorless(ColorlessEffect::EnlightenmentResolve))),
        Forethought => effects.push((None, EffectKind::Colorless(ColorlessEffect::ForethoughtSelect))),
        Madness => effects.push((None, EffectKind::Colorless(ColorlessEffect::MadnessResolve))),
        Purity => effects.push((None, EffectKind::Colorless(ColorlessEffect::PuritySelect))),
        SecretTechnique | SecretWeapon => {
            let card_type = if card.id == SecretTechnique { CardType::Skill } else { CardType::Attack };
            if draw_candidates(state, card_type).is_empty() {
                return Err(crate::run::RunError::InvalidAction);
            }
            effects.push((None, EffectKind::Colorless(ColorlessEffect::SecretRetrieve { card_type })));
        }
        ThinkingAhead => {
            effects.push((None, EffectKind::Draw(2)));
            // `use` observes the playing card in hand before it enters limbo.
            if !state.piles.hand.is_empty() {
                effects.push((None, EffectKind::Colorless(ColorlessEffect::ThinkingAheadSelect)));
            }
        }
        Violence => effects.push((None, EffectKind::Colorless(ColorlessEffect::ViolenceMove { count: if up { 4 } else { 3 } }))),
        _ => unreachable!(),
    }
    Ok(Some((starting_cost(card.id, card.upgrades).expect("choice card has a starting cost"), effects)))
}

pub(super) fn public_values(card: &LiveCard) -> Option<CardPublicValues> {
    use CardId::*;
    let magic = match card.id {
        Purity => Some(if card.upgrades > 0 { 5 } else { 3 }),
        ThinkingAhead => Some(2),
        Violence => Some(if card.upgrades > 0 { 4 } else { 3 }),
        Apotheosis | Enlightenment | Forethought | Madness | SecretTechnique | SecretWeapon => None,
        _ => return None,
    };
    Some(CardPublicValues {
        base_damage: None,
        base_block: None,
        magic_number: magic,
        damage_bonus: None,
        cost_kind: CardCostKind::Fixed,
        free_play: card.local.free_play,
        exhaust_on_finish: card.local.exhaust_on_finish,
        ethereal: card.local.ethereal,
        innate: card.local.innate,
        temporary_cost: card.local.temporary_cost,
    })
}

pub(super) fn validate_work(state: &State, effect: &Effect, work: &ColorlessEffect) -> Result<(), InputError> {
    let Source::Card(source) = &effect.source else {
        return Err(InputError::Invariant("Colorless choice work has no card owner".into()));
    };
    let card = state.in_play.as_ref().filter(|owned| owned.handle() == *source)
        .and_then(OwnedCard::full)
        .ok_or_else(|| InputError::Invariant("Colorless choice continuation parent absent".into()))?;
    let valid = match work {
        ColorlessEffect::ApotheosisUpgrade => card.id == CardId::Apotheosis,
        ColorlessEffect::EnlightenmentResolve => card.id == CardId::Enlightenment,
        ColorlessEffect::ForethoughtSelect => card.id == CardId::Forethought,
        ColorlessEffect::MadnessResolve => card.id == CardId::Madness,
        ColorlessEffect::PuritySelect => card.id == CardId::Purity,
        ColorlessEffect::SecretRetrieve { card_type } => {
            matches!((card.id, *card_type), (CardId::SecretTechnique, CardType::Skill)
                | (CardId::SecretWeapon, CardType::Attack))
        }
        ColorlessEffect::ThinkingAheadSelect => card.id == CardId::ThinkingAhead,
        ColorlessEffect::ViolenceMove { count } => card.id == CardId::Violence
            && *count == if card.upgrades > 0 { 4 } else { 3 },
        _ => false,
    };
    if !valid || effect.target.is_some() {
        return Err(InputError::Invariant("Colorless choice continuation owner/kind".into()));
    }
    Ok(())
}

pub(super) fn apply(state: &mut State, effect: &Effect, work: &ColorlessEffect) -> Result<(), EffectError> {
    match work {
        ColorlessEffect::ApotheosisUpgrade => upgrade_all_piles(state),
        ColorlessEffect::EnlightenmentResolve => enlightenment(state),
        ColorlessEffect::MadnessResolve => madness(state),
        ColorlessEffect::PuritySelect => select_exhaust(state, effect),
        ColorlessEffect::ForethoughtSelect => select_bottomdeck(state, effect),
        ColorlessEffect::SecretRetrieve { card_type } => retrieve_secret(state, effect, *card_type),
        ColorlessEffect::ThinkingAheadSelect => select_topdeck(state, effect),
        ColorlessEffect::ViolenceMove { count } => move_random_attacks(state, *count),
        _ => Err(InputError::Invariant(
            "Impatience continuation reached Colorless choices dispatcher".into()).into()),
    }
}

fn upgrade_all_piles(state: &mut State) -> Result<(), EffectError> {
    for owned in &mut state.piles.hand { upgrade_if_possible(owned)?; }
    for owned in &mut state.piles.draw { upgrade_if_possible(owned)?; }
    for owned in &mut state.piles.discard { upgrade_if_possible(owned)?; }
    for owned in &mut state.piles.exhaust { upgrade_if_possible(owned)?; }
    Ok(())
}

fn upgrade_if_possible(owned: &mut OwnedCard) -> Result<(), EffectError> {
    let Some(card) = owned.full_mut() else { return Ok(()); };
    if !super::super::ironclad::rules::can_upgrade(card.id, card.upgrades) { return Ok(()); }
    super::super::ironclad::rules::upgrade(card).map_err(|error|
        InputError::Unsupported(format!("Apotheosis combat upgrade failed: {error:?}")))?;
    Ok(())
}

fn mutable_combat_cost(state: &State, card: &LiveCard) -> Option<i32> {
    if card.local.x_cost { return None; }
    card.local.mutable_combat_cost.or(card.local.randomized_cost).or_else(|| super::super::base_cost(state, card))
}

fn current_turn_cost(state: &State, card: &LiveCard) -> Result<Option<i32>, InputError> {
    if card.local.x_cost { return Ok(None); }
    if let Some(cost) = card.local.turn_cost { return Ok(Some(cost)); }
    let base = super::super::base_cost(state, card).or(card.local.mutable_combat_cost);
    let Some(base) = base else { return Ok(None); };
    Ok(Some(crate::combat::card_state::combat_price(card, base)?))
}

fn enlightenment(state: &mut State) -> Result<(), EffectError> {
    let up = state.in_play.as_ref().and_then(OwnedCard::full)
        .is_some_and(|card| card.id == CardId::Enlightenment && card.upgrades > 0);
    let mut costs = Vec::with_capacity(state.piles.hand.len());
    for owned in &state.piles.hand {
        if let Some(card) = owned.full() {
            costs.push((owned.handle(), mutable_combat_cost(state, card), current_turn_cost(state, card)?));
        }
    }
    for (handle, combat, turn) in costs {
        let Some(card) = state.piles.hand.iter_mut().find(|owned| owned.handle() == handle)
            .and_then(OwnedCard::full_mut) else {
                return Err(InputError::Invariant("Enlightenment hand card left its captured group".into()).into());
            };
        if turn.is_some_and(|cost| cost > 1) { card.local.turn_cost = Some(1); }
        if up && combat.is_some_and(|cost| cost > 1) { card.local.mutable_combat_cost = Some(1); }
    }
    Ok(())
}

fn madness(state: &mut State) -> Result<(), EffectError> {
    let mut positive_turn = Vec::new();
    let mut positive_combat = Vec::new();
    for owned in &state.piles.hand {
        let Some(card) = owned.full() else { continue; };
        if current_turn_cost(state, card)?.is_some_and(|cost| cost > 0) {
            positive_turn.push(owned.handle());
        }
        if mutable_combat_cost(state, card).is_some_and(|cost| cost > 0) {
            positive_combat.push(owned.handle());
        }
    }
    let candidates = if positive_turn.is_empty() { positive_combat } else { positive_turn };
    if candidates.is_empty() { return Ok(()); }
    let stream = state.card_rng.as_ref()
        .ok_or_else(|| InputError::Invariant("Madness card-random stream absent".into()))?;
    let (index, next) = super::super::super::rng::uniform_below(stream, candidates.len() as u128)?;
    state.card_rng = Some(next);
    let handle = &candidates[index as usize];
    let card = state.piles.hand.iter_mut().find(|owned| owned.handle() == *handle)
        .and_then(OwnedCard::full_mut)
        .ok_or_else(|| InputError::Invariant("Madness selected hand card disappeared".into()))?;
    card.local.mutable_combat_cost = Some(0);
    card.local.turn_cost = Some(0);
    Ok(())
}

fn select_exhaust(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    let Source::Card(parent) = &effect.source else { unreachable!("validated Colorless card parent") };
    let candidates = state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    if candidates.is_empty() { return Ok(()); }
    let maximum = state.in_play.as_ref().and_then(OwnedCard::full).map_or(3, |card| if card.upgrades > 0 { 5 } else { 3 });
    let maximum = maximum.min(candidates.len());
    crate::combat::effects::begin_selection_range(state, SelectionParent::Card(parent.clone()),
        SelectionSourceZone::Hand, candidates, 0, maximum, SelectionDisposition::Exhaust, EmptySelection::Continue)?;
    Ok(())
}

fn select_bottomdeck(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    let Source::Card(parent) = &effect.source else { unreachable!("validated Colorless card parent") };
    let upgraded = state.in_play.as_ref().and_then(OwnedCard::full)
        .is_some_and(|card| card.id == CardId::Forethought && card.upgrades > 0);
    let candidates = state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    if candidates.is_empty() { return Ok(()); }
    if !upgraded && candidates.len() == 1 {
        move_hand_to_bottom(state, &candidates[0])?;
        return Ok(());
    }
    let (minimum, maximum) = if upgraded { (0, candidates.len()) } else { (1, 1) };
    crate::combat::effects::begin_selection_range(state, SelectionParent::Card(parent.clone()),
        SelectionSourceZone::Hand, candidates, minimum, maximum, SelectionDisposition::DrawTop, EmptySelection::Continue)?;
    state.pending_selection.as_mut().expect("Forethought installed a nonempty selection").operation = SelectionOperation::Bottomdeck;
    Ok(())
}

fn move_hand_to_bottom(state: &mut State, handle: &crate::combat::CardHandle) -> Result<(), EffectError> {
    let mut owned = crate::combat::piles::take_hand_card(state, handle)
        .ok_or_else(|| InputError::Invariant("Forethought hand card disappeared".into()))?;
    if let Some(card) = owned.full_mut() { super::super::apply_forethought_free_play(state, card); }
    crate::combat::piles::bottomdeck_owned_card(state, owned, handle.clone());
    crate::combat::piles::refresh_hand_values(state);
    Ok(())
}

fn draw_candidates(state: &State, card_type: CardType) -> Vec<crate::combat::CardHandle> {
    state.piles.draw.iter().filter_map(|owned| {
        let card = owned.full()?;
        Catalog::standard().card(card.id).filter(|definition| definition.card_type == card_type)
            .map(|_| owned.handle())
    }).collect()
}

fn retrieve_secret(state: &mut State, effect: &Effect, card_type: CardType) -> Result<(), EffectError> {
    let Source::Card(parent) = &effect.source else { unreachable!("validated Colorless card parent") };
    let candidates = draw_candidates(state, card_type);
    if candidates.is_empty() { return Ok(()); }
    let destination = if state.piles.hand.len() >= 10 { SelectionDisposition::Discard } else { SelectionDisposition::Hand };
    if candidates.len() == 1 {
        let destination = if destination == SelectionDisposition::Hand { CardDestination::Hand } else { CardDestination::Discard };
        crate::combat::piles::move_draw_card(state, &candidates[0], destination)?;
        return Ok(());
    }
    crate::combat::effects::begin_selection(state, parent.clone(), candidates, 1, destination, EmptySelection::Continue)?;
    Ok(())
}

fn select_topdeck(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    let Source::Card(parent) = &effect.source else { unreachable!("validated Colorless card parent") };
    let candidates = state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(()),
        [only] => {
            let card = crate::combat::piles::take_hand_card(state, only)
                .ok_or_else(|| InputError::Invariant("Thinking Ahead hand card disappeared".into()))?;
            crate::combat::piles::topdeck_owned_card(state, card, only.clone());
            crate::combat::piles::refresh_hand_values(state);
            Ok(())
        }
        _ => {
            crate::combat::effects::begin_selection(state, parent.clone(), candidates, 1,
                SelectionDisposition::DrawTop, EmptySelection::Continue)?;
            state.pending_selection.as_mut().expect("Thinking Ahead installed a nonempty selection").operation = SelectionOperation::Topdeck;
            Ok(())
        }
    }
}

fn move_random_attacks(state: &mut State, count: u8) -> Result<(), EffectError> {
    for _ in 0..count {
        let candidates = draw_candidates(state, CardType::Attack);
        if candidates.is_empty() { break; }
        let stream = state.card_rng.as_ref()
            .ok_or_else(|| InputError::Invariant("Violence card-random stream absent".into()))?;
        let (index, next) = super::super::super::rng::uniform_below(stream, candidates.len() as u128)?;
        state.card_rng = Some(next);
        let destination = if state.piles.hand.len() >= 10 { CardDestination::Discard } else { CardDestination::Hand };
        crate::combat::piles::move_draw_card(state, &candidates[index as usize], destination)?;
    }
    Ok(())
}
