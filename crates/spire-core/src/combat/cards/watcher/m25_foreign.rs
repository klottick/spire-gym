//! M25 Foreign Influence over the versioned card RNG and shared offer/delivery path.
use super::*;
use crate::content::{CardColor, CardId, CardRarity, CardType};
use crate::run::RunError;

type Payload = (i32, Vec<(Option<PayloadTarget>, EffectKind)>);
type Tiers = [Vec<CardId>; 3];

fn tiers(support: &[CardId]) -> Result<Tiers, InputError> {
    let mut result: Tiers = [Vec::new(), Vec::new(), Vec::new()];
    let mut seen = std::collections::BTreeSet::new();
    for id in support {
        if !seen.insert(*id) {
            return Err(InputError::Invariant("Foreign Influence support contains a duplicate kind".into()));
        }
        let row = crate::content::Catalog::standard().card(*id)
            .ok_or_else(|| InputError::Invariant("Foreign Influence support kind lacks a catalog row".into()))?;
        if row.card_type != CardType::Attack || crate::content::is_healing(*id)
            || !matches!(row.color, CardColor::Red | CardColor::Green | CardColor::Blue | CardColor::Purple | CardColor::Colorless) {
            return Err(InputError::Invariant("Foreign Influence support has a forbidden catalog row".into()));
        }
        let tier = match row.rarity {
            CardRarity::Common => 0,
            CardRarity::Uncommon => 1,
            CardRarity::Rare => 2,
            _ => return Err(InputError::Invariant("Foreign Influence support has a non-ordinary rarity".into())),
        };
        result[tier].push(*id);
    }
    if result.iter().map(Vec::len).collect::<Vec<_>>() != [43, 48, 15] {
        return Err(InputError::Unsupported("Foreign Influence requires complete 43/48/15 rarity support".into()));
    }
    Ok(result)
}

fn rarity_tier(roll: u64) -> Result<usize, InputError> {
    match roll {
        0..=54 => Ok(0),
        55..=84 => Ok(1),
        85..=99 => Ok(2),
        _ => Err(InputError::Invariant("Foreign Influence rarity roll outside 0..=99".into())),
    }
}

fn proposals(stream: &crate::combat::rng::StreamState, tiers: &Tiers) -> Result<(Vec<CardId>, crate::combat::rng::StreamState), InputError> {
    let mut cursor = stream.clone();
    let mut cards = Vec::with_capacity(3);
    while cards.len() < 3 {
        let (roll, after_rarity) = crate::combat::rng::uniform_below(&cursor, 100)
            .map_err(|_| InputError::Unsupported("Foreign Influence card RNG exhausted".into()))?;
        cursor = after_rarity;
        let tier = rarity_tier(roll)?;
        let support = &tiers[tier];
        let (index, after_kind) = crate::combat::rng::uniform_below(&cursor, support.len() as u128)
            .map_err(|_| InputError::Unsupported("Foreign Influence card RNG exhausted".into()))?;
        cursor = after_kind;
        let id = support[index as usize];
        // A duplicate retries the whole proposal: the next loop starts with a new
        // rarity roll, then chooses from that tier again.
        if !cards.contains(&id) {
            cards.push(id);
        }
    }
    Ok((cards, cursor))
}

fn propose_from_state(state: &mut State) -> Result<Vec<CardId>, EffectError> {
    let tiers = tiers(crate::content::watcher::foreign_influence_support())?;
    let stream = state.card_rng.as_ref()
        .ok_or_else(|| InputError::Invariant("Foreign Influence card-random stream absent".into()))?;
    let (cards, next) = proposals(stream, &tiers)?;
    state.card_rng = Some(next);
    Ok(cards)
}

pub(in crate::combat) fn proposals_with_support(
    stream: &crate::combat::rng::StreamState,
    support: &[CardId],
) -> Result<(Vec<CardId>, crate::combat::rng::StreamState), InputError> {
    proposals(stream, &tiers(support)?)
}

pub(in crate::combat) fn rarity_tier_for_test(roll: u64) -> Result<usize, InputError> {
    rarity_tier(roll)
}

pub(in crate::combat) fn validate_payload_with_support(
    state: &State,
    card: &LiveCard,
    target: Option<u64>,
    support: &[CardId],
) -> Result<(), RunError> {
    if card.id != CardId::ForeignInfluence { return Err(RunError::InvalidAction) }
    if target.is_some() || !state.full_runtime || !cards_active(state)
        || !state.item_context.as_ref().is_some_and(|items| items.m25_cards())
        || !card.local.m21 {
        return Err(RunError::InvalidAction);
    }
    tiers(support).map(|_| ()).map_err(|_| RunError::InvalidAction)
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>) -> Result<Option<Payload>, RunError> {
    if card.id != CardId::ForeignInfluence { return Ok(None) }
    validate_payload_with_support(state, card, target, crate::content::watcher::foreign_influence_support())?;
    let cost = super::m25::m25_cost(card.id, card.upgrades > 0).ok_or(RunError::InvalidAction)?;
    Ok(Some((cost, vec![(Some(PayloadTarget::Player), EffectKind::Watcher(WatcherEffect::OpenForeignInfluence))])))
}

pub(super) fn open(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    let Source::Card(owner) = &effect.source else {
        return Err(InputError::Invariant("Foreign Influence offer lacks its card source".into()).into());
    };
    let parent = state.in_play.as_ref().filter(|owned| owned.handle() == *owner)
        .and_then(OwnedCard::full)
        .filter(|card| card.id == CardId::ForeignInfluence)
        .ok_or_else(|| InputError::Invariant("Foreign Influence card left play before proposal".into()))?;
    let zero_cost = parent.upgrades > 0;
    let cards = propose_from_state(state)?;
    state.pending.push_front(Effect {
        source: effect.source.clone(),
        target: Some(Actor::Player),
        kind: EffectKind::CardOffer { cards, optional: true, copies: 1, zero_cost },
    });
    Ok(())
}

pub(in crate::combat) fn output_template(state: &State, id: CardId, upgrades: u32) -> Result<LiveCard, InputError> {
    if upgrades > 1 || !crate::content::watcher::foreign_influence_support().contains(&id)
        || !state.item_context.as_ref().is_some_and(|items| items.m25_cards()) {
        return Err(InputError::Unsupported("Foreign Influence output outside M25 admission".into()));
    }
    let mut template = crate::combat::card_state::generated_with_history(id, CardInstanceId(0), upgrades,
        crate::combat::card_state::CardConstructionHistory::from_state(state))
        .unwrap_or_else(|| crate::combat::card_state::permanent(id, CardInstanceId(0), upgrades));
    template.origin = CombatCardRef::CombatLocal(0);
    template.copy_provenance = crate::combat::card_state::CopyProvenance::Generated;
    super::initialize_m25_profile(&mut template)?;
    Ok(template)
}
