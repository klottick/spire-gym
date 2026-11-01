//! M25 Watcher generation, Wish choices, Study and Omega callbacks.
use super::*;
use crate::content::CardId;
use crate::run::RunError;

type Payload = (i32, Vec<(Option<PayloadTarget>, EffectKind)>);

fn no_target(target: Option<u64>) -> Result<(), RunError> {
    if target.is_none() { Ok(()) } else { Err(RunError::InvalidAction) }
}

fn create_shuffled_action(state: &State, id: CardId, count: u32) -> Result<EffectKind, InputError> {
    if !state.item_context.as_ref().is_some_and(|items| items.m25_profile())
        || !crate::content::watcher::supports_m25_generated(id)
        || count == 0 || count > i32::MAX as u32 {
        return Err(InputError::Invariant("M25 shuffled child route kind/count/capability".into()));
    }
    let mut template = super::super::super::card_state::generated(id, CardInstanceId(0), 0)
        .ok_or_else(|| InputError::Unsupported("M25 shuffled child constructor unavailable".into()))?;
    super::m25::initialize_m25_profile(&mut template)?;
    Ok(EffectKind::Watcher(WatcherEffect::CreateShuffled { template, count, next: 0 }))
}

fn create_shuffled(id: CardId, count: u32, state: &State) -> Result<(Option<PayloadTarget>, EffectKind), RunError> {
    let kind = create_shuffled_action(state, id, count).map_err(|_| RunError::InvalidAction)?;
    Ok((Some(PayloadTarget::Player), kind))
}

fn hit(state: &State, target: u64, base: i32) -> Result<(Option<PayloadTarget>, EffectKind), RunError> {
    Ok((Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(
        DamagePayload::Fixed(super::super::modified_mutable_damage(state, target, base)?))))
}

pub(in crate::combat) fn payload(state: &State, card: &LiveCard, target: Option<u64>) -> Result<Option<Payload>, RunError> {
    use CardId::*;
    if !matches!(card.id, Evaluate | Pray | Study | ReachHeaven | Alpha | ConjureBlade | Wish
        | Insight | ThroughViolence | Beta | Omega | Expunger) {
        return Ok(None);
    }
    if !state.item_context.as_ref().is_some_and(|items| items.m25_profile()) {
        return Err(RunError::InvalidAction);
    }
    let cost = super::m25::m25_cost(card.id, card.upgrades > 0).ok_or(RunError::InvalidAction)?;
    let up = card.upgrades > 0;
    let no_target_card = !matches!(card.id, ReachHeaven | ThroughViolence | Expunger);
    if no_target_card { no_target(target)?; }
    let effects = match card.id {
        Evaluate => vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(super::super::modified_block(state, card.local.base_block.ok_or(RunError::InvalidAction)?)?)),
            create_shuffled(Insight, 1, state)?,
        ],
        Pray => vec![
            (Some(PayloadTarget::Player), EffectKind::Watcher(WatcherEffect::GainMantra { amount: if up { 4 } else { 3 } })),
            create_shuffled(Insight, 1, state)?,
        ],
        Study => vec![(Some(PayloadTarget::Player), EffectKind::ApplyFullPower { id: FullPowerId("study"), amount: 1 })],
        ReachHeaven => {
            let selected = target.ok_or(RunError::InvalidAction)?;
            vec![hit(state, selected, card.local.base_damage.ok_or(RunError::InvalidAction)?)?, create_shuffled(ThroughViolence, 1, state)?]
        }
        Alpha => vec![create_shuffled(Beta, 1, state)?],
        Beta => vec![create_shuffled(Omega, 1, state)?],
        ConjureBlade => {
            let mut hits = super::super::captured_x(state, card)?;
            if state.item_context.as_ref().is_some_and(|items| items.owns_relic(crate::content::RelicId::ChemicalX)) {
                hits = hits.checked_add(2).ok_or(RunError::InvalidAction)?;
            }
            if up { hits = hits.checked_add(1).ok_or(RunError::InvalidAction)?; }
            let (payload_target, mut create) = create_shuffled(Expunger, 1, state)?;
            let EffectKind::Watcher(WatcherEffect::CreateShuffled { template, .. }) = &mut create else { unreachable!() };
            template.misc = hits;
            vec![(payload_target, create)]
        }
        Wish => vec![(Some(PayloadTarget::Player), EffectKind::CardOffer {
            cards: crate::content::watcher::M25_WATCHER_CHOICES.to_vec(), optional: false, copies: 1, zero_cost: false,
        })],
        Insight => vec![(Some(PayloadTarget::Player), EffectKind::Draw(card.local.magic_number.ok_or(RunError::InvalidAction)?))],
        ThroughViolence => {
            let selected = target.ok_or(RunError::InvalidAction)?;
            vec![hit(state, selected, card.local.base_damage.ok_or(RunError::InvalidAction)?)?]
        }
        Omega => vec![(Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
            id: FullPowerId("omega"), amount: card.local.magic_number.ok_or(RunError::InvalidAction)?,
        })],
        Expunger => {
            let selected = target.ok_or(RunError::InvalidAction)?;
            let count = usize::try_from(card.misc).map_err(|_| RunError::InvalidAction)?;
            (0..count).map(|_| hit(state, selected, card.local.base_damage.ok_or(RunError::InvalidAction)?))
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => unreachable!(),
    };
    Ok(Some((cost, effects)))
}

pub(super) fn hook(state: &State, hook: Hook, target: Option<&Actor>, work: &mut Vec<Effect>) -> Result<(), EffectError> {
    if hook != Hook::PlayerEndTurn || target != Some(&Actor::Player)
        || !state.item_context.as_ref().is_some_and(|items| items.m25_profile()) {
        return Ok(());
    }
    for power in &state.player_full_powers {
        match power.id.0 {
            "study" if power.state.amount > 0 => {
                let count = u32::try_from(power.state.amount).map_err(|_| InputError::Invariant("Study stack count negative".into()))?;
                work.push(Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                    kind: create_shuffled_action(state, CardId::Insight, count)? });
            }
            "omega" if power.state.amount > 0 => {
                for enemy in state.enemies.iter().filter(|enemy| enemy.life == ActorLife::Active && enemy.hp > 0) {
                    work.push(Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(enemy.instance_id.clone())),
                        kind: EffectKind::Damage(DamagePayload::Thorns(power.state.amount)) });
                }
            }
            _ => (),
        }
    }
    Ok(())
}

pub(super) fn apply_wish_option(state: &mut State, effect: &Effect, option: CardId) -> Result<(), EffectError> {
    let wish = state.in_play.as_ref().filter(|owned| match &effect.source {
        Source::Card(owner) => owned.handle() == *owner,
        _ => false,
    }).and_then(OwnedCard::full).filter(|card| card.id == CardId::Wish)
        .ok_or_else(|| InputError::Invariant("Wish option lost its physical parent".into()))?;
    let upgraded = wish.upgrades > 0;
    match option {
        CardId::BecomeAlmighty => state.pending.push_front(Effect { source: effect.source.clone(), target: Some(Actor::Player),
            kind: EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: if upgraded { 4 } else { 3 } } }),
        CardId::FameAndFortune => super::super::super::enemies::act1::thieves::gain_gold(state, if upgraded { 30 } else { 25 })?,
        CardId::LiveForever => state.pending.push_front(Effect { source: effect.source.clone(), target: Some(Actor::Player),
            kind: EffectKind::ApplyFullPower { id: FullPowerId("plated_armor"), amount: if upgraded { 8 } else { 6 } } }),
        _ => return Err(InputError::Invariant("Wish selected an unknown option".into()).into()),
    }
    Ok(())
}
