use super::super::*;
use super::super::{engine, model, piles};
use super::super::rng::{Domain, StreamId, StreamState};
use super::support::base_scenario;

fn combat() -> Combat { Combat::new(&base_scenario()).unwrap() }
fn play(c: &Combat, id: &str, targeted: bool) -> Action {
    Action::PlayCard { card_instance_id: CardHandle::parse(id).unwrap(),
        target_id: targeted.then(|| c.state.enemies[0].instance_id.clone()) }
}
fn work(target: Option<Actor>, kind: EffectKind) -> Effect {
    Effect { source: Source::None, target, kind }
}

#[test]
fn bash_then_strike_order() {
    let mut s = base_scenario();
    s.piles.hand[0].card_id = CardKind::Bash;
    s.piles.hand[1].card_id = CardKind::Strike;
    let mut c = Combat::new(&s).unwrap();
    let rng = c.state.rng.clone();
    assert_eq!(c.step(&play(&c, "1", true)).unwrap().status, TransitionStatus::Decision);
    assert_eq!((c.state.enemies[0].hp, c.state.player.energy), (40, 1));
    assert_eq!(c.state.enemies[0].powers[0].state.amount, 2);
    assert_eq!(c.step(&play(&c, "2", true)).unwrap().status, TransitionStatus::Decision);
    assert_eq!((c.state.enemies[0].hp, c.state.player.energy), (31, 0));
    assert_eq!(c.state.piles.discard.iter().map(|c| c.instance_id.as_str()).collect::<Vec<_>>(), ["1", "2"]);
    assert!(c.state.in_play.is_none());
    assert_eq!(c.state.rng, rng);
    assert!(Combat::restore(&c.snapshot()).is_ok());
}

#[test]
fn upgrades_and_existing_vulnerable() {
    for (id, target, hp, block, energy, vulnerable) in [
        ("1", true, 39, 0, 2, 0), ("2", false, 48, 8, 2, 0), ("3", true, 38, 0, 1, 3),
    ] {
        let mut s = base_scenario();
        for card in &mut s.piles.hand { card.upgraded = true; }
        let mut c = Combat::new(&s).unwrap();
        c.step(&play(&c, id, target)).unwrap();
        assert_eq!((c.state.enemies[0].hp, c.state.player.block, c.state.player.energy), (hp, block, energy));
        assert_eq!(c.state.enemies[0].powers.first().map_or(0, |p| p.state.amount), vulnerable);
    }
    let mut c = combat();
    c.state.enemies[0].powers.push(PowerEntry { id: PowerId::Vulnerable,
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: true } });
    c.state.player.powers.push(PowerEntry { id: PowerId::Strength,
        state: PowerState { amount: 3, skip_first: false, monster_source_grace: false } });
    c.step(&play(&c, "3", true)).unwrap();
    assert_eq!(c.state.enemies[0].hp, 32);
    assert_eq!(c.state.enemies[0].powers[0].state.amount, 4);
    assert!(c.state.enemies[0].powers[0].state.monster_source_grace);
}

#[test]
fn late_invalid_action_is_atomic() {
    let mut c = combat();
    c.step(&play(&c, "3", true)).unwrap();
    let actions = [play(&c, "3", true), play(&c, "999", true), play(&c, "1", false),
        play(&c, "2", true), Action::PlayCard { card_instance_id: CardHandle::parse("1").unwrap(),
            target_id: Some(EnemyHandle::parse("wrong").unwrap()) }];
    for action in actions {
        let before = c.snapshot();
        let view = c.public_view().unwrap();
        assert_eq!(c.step(&action), Err(StepError::InvalidAction));
        assert_eq!(c.snapshot(), before);
        assert_eq!(c.public_view().unwrap(), view);
    }
    c.step(&play(&c, "1", true)).unwrap();
    let before = c.snapshot();
    assert_eq!(c.step(&play(&c, "2", false)), Err(StepError::InvalidAction));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn terminal_busy_and_faulted_access_are_distinct() {
    let mut c = combat();
    c.state.phase = Phase::PlayerAction;
    c.state.pending.push_back(work(None, EffectKind::Decision));
    let before = c.snapshot();
    assert_eq!(c.step(&Action::EndTurn), Err(StepError::NotReady));
    assert_eq!(c.public_view(), Err(AccessError::NotReady));
    assert_eq!(c.legal_actions(), Err(AccessError::NotReady));
    assert_eq!(c.snapshot(), before);
    model::mark_fault(&mut c.state, FaultKind::Invariant, "test".into());
    let before = c.snapshot();
    assert_eq!(c.step(&Action::EndTurn), Err(StepError::Faulted));
    assert_eq!(c.public_view(), Err(AccessError::Faulted));
    assert_eq!(c.legal_actions(), Err(AccessError::Faulted));
    assert_eq!(c.snapshot(), before);
    let mut c = combat();
    c.state.enemies[0].hp = 1;
    c.step(&play(&c, "1", true)).unwrap();
    let before = c.snapshot();
    assert_eq!(c.step(&Action::EndTurn), Err(StepError::Terminal));
    assert_eq!(c.legal_actions().unwrap(), []);
    assert_eq!(c.snapshot(), before);
}

#[test]
fn public_hidden_state_noninterference() {
    let mut s = base_scenario();
    s.piles.draw = s.piles.hand.split_off(1);
    let a = Combat::new(&s).unwrap();
    let mut b = Combat::new(&s).unwrap();
    b.state.piles.draw.reverse();
    b.state.rng.shuffle = StreamState::new(StreamId::new(Domain::CombatShuffle,
        vec!["different".into()]).unwrap(), [21; 32], 18).unwrap();
    b.state.rng.enemy_ai = StreamState::new(StreamId::new(Domain::CombatEnemyAi,
        vec!["different".into(), "private".into()]).unwrap(), [22; 32], 33).unwrap();
    b.state.enemies[0].move_history.push(MoveId::Attack);
    b.state.enemies[0].selected_move = MoveId::Attack;
    b.state.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().output = 9;
    assert_eq!(a.public_view(), b.public_view());
    assert_eq!(a.legal_actions(), b.legal_actions());
    let before = b.snapshot();
    b.public_view().unwrap(); b.legal_actions().unwrap();
    assert_eq!(b.snapshot(), before);
}

#[test]
fn views_and_actions_are_owned_and_ordered() {
    let mut c = combat();
    let baseline = c.public_view().unwrap();
    let captured = baseline.clone();
    let actions = c.legal_actions().unwrap();
    assert_eq!(actions, vec![play(&c, "1", true), play(&c, "2", false), play(&c, "3", true), Action::EndTurn]);
    let actions_baseline = actions.clone();
    let mut altered = c.public_view().unwrap();
    altered.hand.clear(); altered.enemies.clear(); altered.player.powers.push((PowerId::Strength, 9));
    altered.content_id.clear(); altered.revealed_order.clear(); altered.discard.clear(); altered.exhaust.clear();
    let mut altered_actions = c.legal_actions().unwrap(); altered_actions.clear();
    assert_eq!(c.public_view().unwrap(), baseline);
    c.step(&actions[0]).unwrap();
    assert_eq!(baseline, captured);
    assert_eq!(actions, actions_baseline);
    assert_ne!(c.public_view().unwrap(), baseline);
}

#[test]
fn burning_blood_terminal_checkpoints() {
    for (hp, healed) in [(40, 46), (49, 50)] {
        let mut c = combat();
        c.state.player.hp = hp;
        c.state.player.block = 7;
        c.state.enemies[0].hp = 1;
        let hand = c.state.piles.hand[1..].to_vec();
        let rng = c.state.rng.clone();
        assert_eq!(c.step(&play(&c, "1", true)).unwrap().status, TransitionStatus::Terminal);
        assert_eq!((c.state.hp_before_healing, c.state.hp_after_cleanup), (Some(hp), Some(healed)));
        assert_eq!(c.state.outcome, Some(Outcome::Victory));
        assert_eq!(c.state.piles.hand, hand);
        assert_eq!(c.state.player.block, 7);
        assert_eq!(c.state.player.energy, 2);
        assert_eq!(c.state.in_play.as_ref().unwrap().instance_id.as_str(), "1");
        assert!(c.state.piles.discard.is_empty());
        assert!(c.state.pending.is_empty() && c.state.current.is_none());
        assert_eq!(c.state.rng, rng);
        assert_eq!(c.state.resolved_effects, 2);
        assert_eq!(c.public_view().unwrap().hp_after_cleanup, Some(healed));
        assert!(c.legal_actions().unwrap().is_empty());
        assert_eq!(Combat::restore(&c.snapshot()).unwrap().snapshot(), c.snapshot());
    }
}

#[test]
fn death_never_heals_or_rolls() {
    let mut c = combat();
    c.state.rng.enemy_ai = StreamState::new(c.state.rng.enemy_ai.id().clone(), [0; 32], 1u128 << 64).unwrap();
    let rng = c.state.rng.clone();
    c.state.phase = Phase::EnemyTurn;
    c.state.pending.extend([work(Some(Actor::Player), EffectKind::Damage(DamagePayload::Fixed(40))),
        work(None, EffectKind::RollMove), work(None, EffectKind::Draw(1))]);
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Terminal);
    assert_eq!((c.state.hp_before_healing, c.state.hp_after_cleanup), (Some(0), Some(0)));
    assert_eq!(c.state.outcome, Some(Outcome::Death));
    assert_eq!(c.state.rng, rng);
    assert_eq!(c.state.resolved_effects, 2);
    assert!(c.state.fault.is_none());
    assert!(Combat::restore(&c.snapshot()).is_ok());
}

#[test]
fn end_turn_schedules_and_resets_only_accepted_budget() {
    let mut c = combat(); c.state.resolved_effects = 99999;
    let before = c.snapshot();
    assert_eq!(c.step(&play(&c, "999", false)), Err(StepError::InvalidAction));
    assert_eq!(c.snapshot(), before);
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Decision);
    assert_eq!(c.state.resolved_effects, 11); // The accepted action resets the old budget.
    assert!(c.state.current.is_none() && c.state.pending.is_empty());
    assert_eq!(c.state.fault, None);
    assert_eq!((c.state.turn, c.state.player.hp), (2, 40));
    let owned = c.state.piles.hand.iter().chain(&c.state.piles.draw)
        .chain(&c.state.piles.discard).chain(&c.state.piles.exhaust)
        .map(|card| card.instance_id.clone()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(owned, before.state.card_ids_issued);
    assert!(c.state.in_play.is_none());
}

#[test]
fn discard_membership_is_immediate_and_checked() {
    let mut c = combat();
    let id = CardHandle::parse("2").unwrap();
    piles::discard_card(&mut c.state, &id, true).unwrap();
    assert_eq!(c.state.piles.discard[0].instance_id, id);
    assert_eq!(c.state.piles.hand.len(), 2);
    let before = c.snapshot();
    assert!(piles::discard_card(&mut c.state, &id, true).is_err());
    assert_eq!(c.snapshot(), before);
}

#[test]
fn base_defend_caps_block_and_affordable_choices_keep_hand_order() {
    let mut c = combat();
    c.state.player.block = 997;
    c.step(&play(&c, "2", false)).unwrap();
    assert_eq!(c.state.player.block, 999);
    c.step(&play(&c, "1", true)).unwrap();
    assert_eq!(c.state.player.energy, 1);
    assert_eq!(c.legal_actions().unwrap(), vec![Action::EndTurn]);
    let before = c.snapshot();
    assert_eq!(c.step(&play(&c, "3", true)), Err(StepError::InvalidAction));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn public_power_sorting_hides_lifecycle_flags() {
    let mut c = combat();
    c.state.enemies[0].powers = vec![
        PowerEntry { id: PowerId::Vulnerable, state: PowerState {
            amount: 2, skip_first: false, monster_source_grace: false } },
        PowerEntry { id: PowerId::Strength, state: PowerState {
            amount: 3, skip_first: false, monster_source_grace: false } },
        PowerEntry { id: PowerId::Ritual, state: PowerState {
            amount: 3, skip_first: true, monster_source_grace: false } },
    ];
    let before = c.snapshot();
    let view = c.public_view().unwrap();
    assert_eq!(view.enemies[0].powers,
        vec![(PowerId::Ritual, 3), (PowerId::Strength, 3), (PowerId::Vulnerable, 2)]);
    assert_eq!(c.snapshot(), before);
    c.state.enemies[0].powers[0].state.monster_source_grace = true;
    c.state.enemies[0].powers[2].state.skip_first = false;
    assert_eq!(c.public_view().unwrap(), view);
}

#[test]
fn lethal_bash_prunes_vulnerable_and_keeps_all_other_piles() {
    let mut s = base_scenario();
    s.piles.draw.push(CardInstance { instance_id: CardHandle::parse("4").unwrap(),
        card_id: CardKind::Defend, upgraded: true });
    let mut c = Combat::new(&s).unwrap();
    c.step(&play(&c, "2", false)).unwrap();
    c.state.piles.exhaust.push(c.state.piles.draw.pop().unwrap());
    c.state.enemies[0].hp = 8;
    let before = c.snapshot();
    assert_eq!(c.step(&play(&c, "3", true)).unwrap().status, TransitionStatus::Terminal);
    assert!(c.state.enemies[0].powers.is_empty());
    assert_eq!(c.state.piles.discard, before.state.piles.discard);
    assert_eq!(c.state.piles.exhaust, before.state.piles.exhaust);
    assert_eq!(c.state.piles.draw, before.state.piles.draw);
    assert_eq!(c.state.player.block, 5);
    assert_eq!(c.state.in_play.as_ref().unwrap().instance_id.as_str(), "3");
    assert!(Combat::restore(&c.snapshot()).is_ok());
}

#[test]
fn malformed_ready_checkpoint_and_finish_owner_do_not_silently_advance() {
    let mut c = combat();
    c.state.pending.push_back(work(None, EffectKind::Decision));
    let before = c.snapshot();
    assert_eq!(c.step(&Action::EndTurn), Err(StepError::NotReady));
    assert_eq!(c.snapshot(), before);
    c.state.pending.clear();
    c.state.phase = Phase::PlayerAction;
    c.state.in_play = Some(c.state.piles.hand.remove(0));
    c.state.pending.push_back(Effect { source: Source::Card(CardHandle::parse("2").unwrap()),
        target: None, kind: EffectKind::FinishCard });
    let owner = c.state.in_play.clone();
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Fault);
    assert_eq!(c.state.in_play, owner);
    assert!(c.state.piles.discard.is_empty());
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
}
