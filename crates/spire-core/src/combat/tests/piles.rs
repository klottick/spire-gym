use super::super::*;
use super::super::model::OwnedCard;
use super::super::{engine, model::State};
use super::super::rng::StreamState;
use super::support::base_scenario;
use std::collections::BTreeSet;

fn work(kind: EffectKind) -> Effect { Effect { source: Source::None, target: None, kind } }
fn ids(cards: &[OwnedCard]) -> Vec<String> {
    cards.iter().map(|c| c.handle().as_str().to_owned()).collect()
}
fn combat(hand: &[i32], draw: &[i32], discard: &[i32]) -> Combat {
    let card = |id: &i32| CardInstance { instance_id: CardHandle::parse(&id.to_string()).unwrap(),
        card_id: CardKind::Strike, upgraded: false };
    let mut scenario = base_scenario();
    scenario.piles.hand = hand.iter().map(card).collect();
    scenario.piles.draw = draw.iter().chain(discard).map(card).collect();
    let mut c = Combat::new(&scenario).unwrap();
    c.state.piles.discard = c.state.piles.draw.split_off(draw.len());
    c.state.phase = Phase::PlayerAction;
    c
}
fn owners(state: &State) {
    let all: Vec<_> = state.piles.hand.iter().chain(&state.piles.draw).chain(&state.piles.discard)
        .chain(&state.piles.exhaust).chain(state.in_play.iter()).map(OwnedCard::handle).collect();
    let unique: BTreeSet<_> = all.iter().cloned().collect();
    assert_eq!(all.len(), unique.len());
    assert_eq!(unique, state.card_ids_issued);
}
fn dispatch(c: &mut Combat) {
    owners(&c.state);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault, None);
    assert!(c.state.current.is_none());
    owners(&c.state);
}
fn finish(c: &mut Combat) {
    while !c.state.pending.is_empty() { dispatch(c); }
}

#[test]
fn draw_uses_end_and_preserves_owners() {
    let mut c = combat(&[4], &[1, 2, 3], &[]);
    c.state.in_play = c.state.piles.hand.pop();
    c.state.pending.push_back(work(EffectKind::Draw(3)));
    c.state.pending.push_back(Effect { source: Source::Card(CardHandle::parse("4").unwrap()),
        target: None, kind: EffectKind::FinishCard });
    dispatch(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["3", "2", "1"]);
    assert!(c.state.piles.draw.is_empty());
    assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(), "4");
    assert_eq!(c.state.public.revealed_order.iter().map(CardHandle::as_str).collect::<Vec<_>>(), ["3", "2", "1"]);
    dispatch(&mut c);
    assert_eq!(ids(&c.state.piles.discard), ["4"]);
}

#[test]
fn discard_hand_visits_reverse_hand_order() {
    let mut c = combat(&[1, 2, 3], &[4], &[5]);
    c.state.pending.push_back(work(EffectKind::DiscardHand));
    dispatch(&mut c);
    assert!(c.state.piles.hand.is_empty());
    assert_eq!(ids(&c.state.piles.discard), ["5", "3", "2", "1"]);
    assert_eq!(ids(&c.state.piles.draw), ["4"]);
    assert!(c.state.public.revealed_order.is_empty());
}

#[test]
fn split_draw_existing_recycle_residual() {
    let mut c = combat(&[], &[4, 5], &[1, 2, 3]);
    let mut draw = work(EffectKind::Draw(5));
    draw.source = Source::Actor(Actor::Player);
    draw.target = Some(Actor::Player);
    c.state.pending.extend([draw.clone(), work(EffectKind::Decision)]);
    dispatch(&mut c);
    assert!(c.state.piles.hand.is_empty());
    assert_eq!(ids(&c.state.piles.draw), ["4", "5"]);
    assert_eq!(c.state.pending.iter().map(|e| e.kind.clone()).collect::<Vec<_>>(),
        [EffectKind::Draw(2), EffectKind::Recycle, EffectKind::Draw(3), EffectKind::Decision]);
    for e in c.state.pending.iter().take(3) {
        assert_eq!(e.source, draw.source);
        assert_eq!(e.target, draw.target);
    }
    dispatch(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["5", "4"]);
    assert_eq!(ids(&c.state.piles.discard), ["1", "2", "3"]);
    assert_eq!(c.state.rng.shuffle.word_index(), 0);
    dispatch(&mut c);
    assert_eq!(ids(&c.state.piles.draw), ["2", "3", "1"]);
    assert_eq!(c.state.rng.shuffle.word_index(), 2);
    finish(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["5", "4", "1", "3", "2"]);
    assert_eq!(c.state.phase, Phase::Decision);
}

#[test]
fn draw_rechecks_sizes_and_hand_cap() {
    let mut c = combat(&[1, 2, 3, 4, 5, 6, 7, 8, 9], &[10, 11], &[12]);
    c.state.pending.extend([work(EffectKind::Draw(10)), work(EffectKind::Draw(10))]);
    finish(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["1", "2", "3", "4", "5", "6", "7", "8", "9", "11"]);
    assert_eq!(ids(&c.state.piles.draw), ["10"]);
    assert_eq!(ids(&c.state.piles.discard), ["12"]);
    assert_eq!(c.state.rng.shuffle.word_index(), 0);

    let mut c = combat(&[1], &[], &[]);
    c.state.pending.extend([work(EffectKind::Draw(10)), work(EffectKind::Draw(0))]);
    finish(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["1"]);
    assert_eq!(c.state.resolved_effects, 2);

    let mut c = combat(&[], &[1], &[2]);
    c.state.pending.push_back(work(EffectKind::Draw(10)));
    finish(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["1", "2"]);
    assert_eq!(c.state.rng.shuffle.word_index(), 0);

    let mut c = combat(&[1, 2], &[3], &[]);
    c.state.pending.extend([work(EffectKind::DiscardHand), work(EffectKind::Draw(3))]);
    finish(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["3", "2", "1"]);
    assert_eq!(c.state.rng.shuffle.word_index(), 1);
}

#[test]
fn fixed_recycle_draw_order() {
    let mut c = combat(&[], &[], &[1, 2, 3]);
    let ai = c.state.rng.enemy_ai.clone();
    c.state.pending.extend([work(EffectKind::Recycle), work(EffectKind::Draw(3))]);
    dispatch(&mut c);
    assert_eq!(ids(&c.state.piles.draw), ["2", "3", "1"]);
    assert!(c.state.piles.discard.is_empty());
    assert_eq!(c.state.rng.shuffle.word_index(), 2);
    finish(&mut c);
    assert_eq!(ids(&c.state.piles.hand), ["1", "3", "2"]);
    assert_eq!(c.state.rng.enemy_ai, ai);
    // Public draw history is chronological, including repeated later draws.
    c.state.pending.extend([work(EffectKind::DiscardHand), work(EffectKind::Draw(1))]);
    finish(&mut c);
    assert_eq!(&c.state.public.revealed_order[..3], &[
        CardHandle::parse("1").unwrap(), CardHandle::parse("3").unwrap(), CardHandle::parse("2").unwrap()]);
    assert_eq!(c.state.public.revealed_order.len(), 4);
}

#[test]
fn empty_and_singleton_recycle_consume_no_words() {
    for discard in [vec![], vec![2]] {
        let mut c = combat(&[1], &[3], &discard);
        let rng = c.state.rng.clone();
        c.state.pending.push_back(work(EffectKind::Recycle));
        dispatch(&mut c);
        assert_eq!(ids(&c.state.piles.draw), if discard.is_empty() { vec!["3"] } else { vec!["3", "2"] });
        assert!(c.state.piles.discard.is_empty());
        assert_eq!(c.state.rng, rng);
    }
}

#[test]
fn recycle_exhaustion_preserves_piles() {
    let mut c = combat(&[5], &[4], &[1, 2, 3]);
    c.state.rng.shuffle = StreamState::new(c.state.rng.shuffle.id().clone(),
        *c.state.rng.shuffle.key(), (1u128 << 64) - 1).unwrap();
    let before = c.state.piles.clone();
    let ai = c.state.rng.enemy_ai.clone();
    let effect = work(EffectKind::Recycle);
    c.state.pending.extend([effect.clone(), work(EffectKind::Draw(2))]);
    owners(&c.state);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.piles, before);
    owners(&c.state);
    assert_eq!(c.state.rng.shuffle.word_index(), 1u128 << 64);
    assert_eq!(c.state.rng.enemy_ai, ai);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
    assert_eq!(c.state.fault.as_ref().unwrap().resolved_effects, 1);
    assert_eq!(c.state.current, Some(effect));
    assert_eq!(c.state.pending, [work(EffectKind::Draw(2))]);
    assert_eq!(c.state.outcome, None);
    assert_eq!(c.state.player.hp, 40);
    let faulted = c.snapshot();
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Fault);
    assert_eq!(c.snapshot(), faulted);
    assert_eq!(Combat::restore(&faulted), Err(RestoreError::Faulted));
}

#[test]
fn split_draw_safe_points_replay_complete_work_and_knowledge() {
    let mut c = combat(&[6], &[4, 5], &[1, 2, 3]);
    c.state.pending.extend([work(EffectKind::Draw(5)), work(EffectKind::Decision)]);
    while !c.state.pending.is_empty() {
        let captured = c.snapshot();
        let baseline = captured.clone();
        let mut resumed = Combat::restore(&captured).unwrap();
        dispatch(&mut c);
        dispatch(&mut resumed);
        assert_eq!(snapshot::compare(&c.snapshot(), &resumed.snapshot()), None);
        assert_eq!(captured, baseline);
    }
    assert_eq!(ids(&c.state.piles.hand), ["6", "5", "4", "1", "3", "2"]);
    assert_eq!(c.state.resolved_effects, 5);
    assert_eq!(c.state.phase, Phase::Decision);
}

#[test]
fn malformed_pile_work_rejects_before_mutation() {
    for kind in [EffectKind::Draw(-1), EffectKind::Draw(11)] {
        let mut c = combat(&[], &[1], &[2]);
        let before = c.snapshot();
        assert!(matches!(super::super::piles::apply_effect(&mut c.state, &work(kind)),
            Err(EffectError::Invalid(InputError::Invariant(_)))));
        assert_eq!(before, c.snapshot());
    }
    let mut c = combat(&[], &[1], &[2]);
    let before = c.snapshot();
    assert!(matches!(super::super::piles::apply_effect(&mut c.state, &work(EffectKind::EndPlayerTurn)),
        Err(EffectError::Invalid(InputError::Unsupported(_)))));
    assert_eq!(before, c.snapshot());
    let mut bad = work(EffectKind::Recycle);
    bad.source = Source::Card(CardHandle::parse("999").unwrap());
    assert!(matches!(super::super::piles::apply_effect(&mut c.state, &bad),
        Err(EffectError::Invalid(InputError::Invariant(_)))));
    assert_eq!(before, c.snapshot());
}
