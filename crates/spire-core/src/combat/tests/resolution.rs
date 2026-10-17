use super::super::*;
use super::super::{effects, engine, model};
use super::super::rng::{RngError, StreamState};
use super::support::base_scenario;

fn combat() -> Combat { Combat::new(&base_scenario()).unwrap() }
fn enemy(c: &Combat) -> Actor { Actor::Enemy(c.state.enemies[0].instance_id.clone()) }
fn work(target: Option<Actor>, kind: EffectKind) -> Effect {
    Effect { source: Source::None, target, kind }
}
fn queue(c: &mut Combat, items: Vec<Effect>) {
    c.state.phase = Phase::EnemyTurn;
    c.state.pending.extend(items);
}
fn power(target: Actor, id: PowerId, amount: i32) -> Effect {
    work(Some(target), EffectKind::ApplyPower { id, amount })
}
fn end() -> Effect { work(None, EffectKind::Decision) }

#[test]
fn prepend_keeps_current_owner() {
    let mut c = combat();
    let owner = work(Some(Actor::Player), EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 2 });
    c.state.player.powers.push(PowerEntry { id: PowerId::Vulnerable,
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: true } });
    queue(&mut c, vec![owner, end()]);
    engine::dispatch_one(&mut c.state);
    assert!(c.state.current.is_none());
    assert_eq!(c.state.pending.front().unwrap().kind, EffectKind::RemovePower(PowerId::Vulnerable));
    assert_eq!(c.state.pending.back().unwrap().kind, EffectKind::Decision);
    engine::dispatch_one(&mut c.state);
    assert!(c.state.player.powers.is_empty());
    assert_eq!(c.state.resolved_effects, 2);
}

#[test]
fn damage_reference_refresh() {
    let mut c = combat();
    c.state.enemies[0].intent_id = MoveId::Attack;
    c.state.enemies[0].selected_move = MoveId::Attack;
    c.state.enemies[0].move_history.push(MoveId::Attack);
    let e = enemy(&c);
    let mut attack = work(Some(Actor::Player), EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Attack)));
    attack.source = Source::Actor(e.clone());
    queue(&mut c, vec![power(e.clone(), PowerId::Strength, 3), attack, power(e.clone(), PowerId::Vulnerable, 2),
        work(Some(e), EffectKind::Damage(DamagePayload::Fixed(8))), end()]);
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision);
    assert_eq!(c.state.player.hp, 31);
    assert_eq!(c.state.enemies[0].hp, 40);
    assert_eq!(c.state.enemies[0].damage_entries[&MoveId::Attack].output, 9);
    assert_eq!(c.state.enemies[0].intent_damage, Some(9));
}

#[test]
fn power_flags_stack_without_reset() {
    let mut c = combat();
    let e = enemy(&c);
    let mut vulnerable = power(Actor::Player, PowerId::Vulnerable, 2);
    vulnerable.source = Source::Actor(e.clone());
    queue(&mut c, vec![power(e.clone(), PowerId::Ritual, 3), vulnerable,
        power(Actor::Player, PowerId::Vulnerable, 3), power(e.clone(), PowerId::Strength, 3), end()]);
    engine::resolve(&mut c.state);
    assert_eq!(c.state.player.powers[0].state, PowerState { amount: 5, skip_first: false, monster_source_grace: true });
    assert_eq!(c.state.enemies[0].powers.iter().map(|p| p.id).collect::<Vec<_>>(), vec![PowerId::Ritual, PowerId::Strength]);
    assert!(c.state.enemies[0].powers[0].state.skip_first);
    c.state.enemies[0].powers[1].state.amount = 998;
    queue(&mut c, vec![power(e, PowerId::Strength, 3), end()]);
    engine::resolve(&mut c.state);
    assert_eq!(c.state.enemies[0].powers[1].state.amount, 999);
}

#[test]
fn full_reduction_queues_removal_first() {
    let mut c = combat();
    queue(&mut c, vec![power(Actor::Player, PowerId::Vulnerable, 2),
        work(Some(Actor::Player), EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 2 }), end()]);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.enemies[0].damage_entries[&MoveId::Attack].output, 9);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.player.powers[0].state.amount, 0);
    assert_eq!(c.state.enemies[0].damage_entries[&MoveId::Attack].output, 6);
    assert!(Combat::restore(&c.snapshot()).is_ok());
    let mut missing_removal = c.snapshot();
    missing_removal.state.pending.pop_front();
    assert!(matches!(Combat::restore(&missing_removal), Err(RestoreError::Invalid(_))));
    assert_eq!(c.state.pending.front().unwrap().kind, EffectKind::RemovePower(PowerId::Vulnerable));
    engine::resolve(&mut c.state);
    assert!(c.state.player.powers.is_empty());
}

#[test]
fn dispatch_99999_finishes_100000_faults() {
    for total in [99999, 100000] {
        let mut c = combat();
        c.state.phase = Phase::EnemyTurn;
        c.state.pending.extend((0..total-1).map(|_| work(Some(Actor::Player), EffectKind::GainBlock(0))));
        c.state.pending.push_back(end());
        let transition = engine::resolve(&mut c.state);
        assert_eq!(c.state.resolved_effects, total);
        assert_eq!(c.state.player.hp, 40);
        assert_eq!(c.state.player.block, 0);
        assert_eq!(c.state.outcome, None);
        if total == 99999 {
            assert_eq!(transition.status, TransitionStatus::Decision);
            assert!(c.state.current.is_none());
        } else {
            assert_eq!(transition.status, TransitionStatus::Fault);
            assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::EffectGuard);
            assert_eq!(c.state.current, Some(end()));
            assert_eq!(c.state.phase, Phase::EnemyTurn);
            assert_eq!(Combat::restore(&c.snapshot()), Err(RestoreError::Faulted));
        }
    }
}

#[test]
fn resume_does_not_reset_budget() {
    let mut c = combat();
    queue(&mut c, vec![work(Some(Actor::Player), EffectKind::GainBlock(5)), end()]);
    c.state.resolved_effects = 99998;
    engine::dispatch_one(&mut c.state);
    let mut resumed = Combat::restore(&c.snapshot()).unwrap();
    assert_eq!(engine::resolve(&mut resumed.state).status, TransitionStatus::Fault);
    assert_eq!(resumed.state.resolved_effects, 100000);
    assert_eq!(resumed.state.player.block, 5);
    assert_eq!(resumed.state.current, Some(end()));
}

#[test]
fn first_fault_wins() {
    let mut c = combat();
    // Every effect now has an owner; Chomp remains incompatible with Cultist.
    queue(&mut c, vec![work(Some(Actor::Player), EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Chomp))), end()]);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Unsupported);
    let original = c.snapshot();
    model::mark_fault(&mut c.state, FaultKind::Invariant, "later".into());
    engine::dispatch_one(&mut c.state);
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Fault);
    assert_eq!(c.snapshot(), original);
}

fn progressed(stream: &StreamState, index: u128) -> StreamState {
    StreamState::new(stream.id().clone(), *stream.key(), index).unwrap()
}

#[test]
fn exhaustion_installs_only_owned_progress() {
    for ai in [false, true] {
        let mut c = combat();
        let before = c.state.rng.clone();
        let owner = if ai { &before.enemy_ai } else { &before.shuffle };
        let latest = progressed(owner, 1u128 << 64);
        engine::retain_error(&mut c.state, model::EffectError::Random(RngError::Exhausted { state: latest.clone() }));
        assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
        assert_eq!(if ai { &c.state.rng.enemy_ai } else { &c.state.rng.shuffle }, &latest);
        assert_eq!(if ai { &c.state.rng.shuffle } else { &c.state.rng.enemy_ai }, if ai { &before.shuffle } else { &before.enemy_ai });
    }
    for invalid in 0..3 {
        let mut c = combat();
        c.state.rng.enemy_ai = progressed(&c.state.rng.enemy_ai, 12);
        let before = c.state.rng.clone();
        let stream = &before.enemy_ai;
        let wrong = match invalid {
            0 => StreamState::new(stream.id().clone(), [17; 32], 13).unwrap(),
            1 => progressed(stream, 11),
            _ => StreamState::new(super::super::rng::StreamId::new(super::super::rng::Domain::CombatEnemyAi,
                vec!["other-combat".into(), "other-owner".into()]).unwrap(), *stream.key(), 13).unwrap(),
        };
        engine::retain_error(&mut c.state, model::EffectError::Random(RngError::Exhausted { state: wrong }));
        assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
        assert_eq!(c.state.rng, before);
    }
}

#[test]
fn unsupported_and_invariant_faults_preserve_work() {
    for (kind, target, expected) in [
        (EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Chomp)), Some(Actor::Player), FaultKind::Unsupported),
        (EffectKind::GainBlock(5), Some(Actor::Enemy(EnemyHandle::parse("wrong").unwrap())), FaultKind::Invariant),
        (EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Thrash)), Some(Actor::Player), FaultKind::Unsupported),
        (EffectKind::Draw(-1), None, FaultKind::Invariant),
        (EffectKind::Draw(11), None, FaultKind::Invariant),
    ] {
        let mut c = combat();
        let effect = work(target, kind);
        queue(&mut c, vec![effect.clone(), end()]);
        engine::resolve(&mut c.state);
        assert_eq!(c.state.fault.as_ref().unwrap().kind, expected);
        assert_eq!(c.state.current, Some(effect));
        assert_eq!(c.state.pending.iter().cloned().collect::<Vec<_>>(), vec![end()]);
        assert_eq!(c.state.player.hp, 40);
        assert_eq!(c.state.resolved_effects, 1);
    }
}

#[test]
fn all_safe_point_fields_replay() {
    let mut c = combat();
    c.state.rng.shuffle = StreamState::new(c.state.rng.shuffle.id().clone(), [23; 32], 7).unwrap();
    c.state.rng.enemy_ai = StreamState::new(c.state.rng.enemy_ai.id().clone(), [61; 32], 19).unwrap();
    c.state.public.revealed_order.push(CardHandle::parse("2").unwrap());
    let e = enemy(&c);
    queue(&mut c, vec![power(e.clone(), PowerId::Ritual, 3), power(e.clone(), PowerId::Strength, 3),
        power(Actor::Player, PowerId::Vulnerable, 2), work(Some(e), EffectKind::Damage(DamagePayload::Fixed(8))),
        work(Some(Actor::Player), EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 2 }), end()]);
    while !c.state.pending.is_empty() {
        let captured = c.snapshot();
        let baseline = captured.clone();
        let mut restored = Combat::restore(&captured).unwrap();
        engine::dispatch_one(&mut c.state);
        engine::dispatch_one(&mut restored.state);
        assert_eq!(snapshot::compare(&c.snapshot(), &restored.snapshot()), None);
        assert_eq!(captured, baseline);
        let mut finish = Combat::restore(&captured).unwrap();
        assert_eq!(engine::resolve(&mut finish.state).status, TransitionStatus::Decision);
        let mut remaining = Combat::restore(&c.snapshot()).unwrap();
        assert_eq!(engine::resolve(&mut remaining.state).status, TransitionStatus::Decision);
        assert_eq!(snapshot::compare(&finish.snapshot(), &remaining.snapshot()), None);
        assert_eq!(finish.state.rng.shuffle.key(), &[23; 32]);
        assert_eq!(finish.state.rng.enemy_ai.word_index(), 19);
    }
}

#[test]
fn lethal_damage_prepends_terminal_owner() {
    let mut c = combat();
    let e = enemy(&c);
    queue(&mut c, vec![work(Some(e), EffectKind::Damage(DamagePayload::Fixed(48))), end()]);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.enemies[0].hp, 0);
    assert_eq!(c.state.pending.front().unwrap().kind, EffectKind::Terminal(Outcome::Victory));
    assert!(c.state.current.is_none());
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Terminal);
    assert_eq!(c.state.fault, None);
    assert_eq!(c.state.outcome, Some(Outcome::Victory));
    assert_eq!((c.state.hp_before_healing, c.state.hp_after_cleanup), (Some(40), Some(46)));
    assert!(c.state.pending.is_empty() && c.state.current.is_none());
}

#[test]
fn direct_effect_signature_is_checked() {
    let mut c = combat();
    assert!(matches!(effects::apply_effect(&mut c.state, &work(None, EffectKind::GainBlock(1))),
        Err(model::EffectError::Invalid(InputError::Invariant(_)))));
}

#[test]
fn ritual_stacking_and_zero_removal_safe_points() {
    let mut c = combat();
    let e = enemy(&c);
    queue(&mut c, vec![power(e.clone(), PowerId::Ritual, 3), power(e.clone(), PowerId::Ritual, 3),
        work(Some(e.clone()), EffectKind::ReducePower { id: PowerId::Ritual, amount: 3 }),
        work(Some(e), EffectKind::ReducePower { id: PowerId::Ritual, amount: 3 }), end()]);
    for _ in 0..5 {
        engine::dispatch_one(&mut c.state);
        assert!(Combat::restore(&c.snapshot()).is_ok());
        if c.state.resolved_effects == 1 { c.state.enemies[0].powers[0].state.skip_first = false; }
        if c.state.resolved_effects == 2 {
            assert_eq!(c.state.enemies[0].powers[0].state.amount, 6);
            assert!(!c.state.enemies[0].powers[0].state.skip_first);
        }
    }
    assert!(c.state.enemies[0].powers.is_empty());
}

#[test]
fn current_owner_retained_even_after_payload_prepends_then_faults() {
    let mut c = combat();
    c.state.player.powers.push(PowerEntry { id: PowerId::Vulnerable,
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    // A corrupt damage entry causes refresh to fail after the reduction enqueues removal.
    c.state.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().base = 12;
    let owner = work(Some(Actor::Player), EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 2 });
    queue(&mut c, vec![owner.clone(), end()]);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.current, Some(owner));
    assert_eq!(c.state.player.powers[0].state.amount, 0);
    assert_eq!(c.state.pending.front().unwrap().kind, EffectKind::RemovePower(PowerId::Vulnerable));
    let faulted = c.snapshot();
    engine::resolve(&mut c.state);
    assert_eq!(faulted, c.snapshot());
    assert_eq!(Combat::restore(&faulted), Err(RestoreError::Faulted));
}

#[test]
fn interrupted_owner_never_retries_and_error_mapping_is_exact() {
    let mut c = combat();
    c.state.current = Some(work(Some(Actor::Player), EffectKind::GainBlock(5)));
    queue(&mut c, vec![end()]);
    assert!(matches!(Combat::restore(&c.snapshot()), Err(RestoreError::Invalid(_))));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.resolved_effects, 0);
    assert_eq!(c.state.player.block, 0);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
    for error in [EffectError::Random(RngError::Invalid("bad".into())),
        EffectError::Invalid(InputError::Invariant("bad".into()))] {
        let mut c = combat();
        engine::retain_error(&mut c.state, error);
        assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
        let baseline = c.snapshot();
        let latest = progressed(&c.state.rng.shuffle, 1u128 << 64);
        engine::retain_error(&mut c.state, EffectError::Random(RngError::Exhausted { state: latest }));
        assert_eq!(baseline, c.snapshot());
    }
}

#[test]
fn checked_power_overflow_retains_original_owner() {
    let mut c = combat();
    c.state.player.powers.push(PowerEntry { id: PowerId::Vulnerable,
        state: PowerState { amount: i32::MAX, skip_first: false, monster_source_grace: false } });
    let owner = power(Actor::Player, PowerId::Vulnerable, 1);
    queue(&mut c, vec![owner.clone(), end()]);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.current, Some(owner));
    assert_eq!(c.state.player.powers[0].state.amount, i32::MAX);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
}

#[test]
fn native_exhaustion_preserves_latest_rejection_cursor() {
    use super::super::rng;
    for ai in [false, true] {
        let mut c = combat();
        let stream = if ai { &mut c.state.rng.enemy_ai } else { &mut c.state.rng.shuffle };
        *stream = progressed(stream, (1u128 << 64) - 1);
        // Independent standard-library HMAC derivation pins this final word.
        // For bound 2^63+1 it is rejected, exhausting the next sampling attempt.
        *stream = StreamState::new(stream.id().clone(), [0; 32], stream.word_index()).unwrap();
        let (word, _) = rng::draw_u64(stream).unwrap();
        assert_eq!(word, 11913346827027590439);
        let error = rng::uniform_below(stream, (1u128 << 63) + 1).unwrap_err();
        let other = if ai { c.state.rng.shuffle.clone() } else { c.state.rng.enemy_ai.clone() };
        engine::retain_error(&mut c.state, error.into());
        assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
        assert_eq!(if ai { &c.state.rng.shuffle } else { &c.state.rng.enemy_ai }, &other);
        assert_eq!(if ai { c.state.rng.enemy_ai.word_index() } else { c.state.rng.shuffle.word_index() }, 1u128 << 64);
    }
}
