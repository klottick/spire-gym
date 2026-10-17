use super::super::*;
use super::super::{effects, engine};
use super::super::rng::StreamState;
use super::support::base_scenario;

fn combat() -> Combat { Combat::new(&base_scenario()).unwrap() }
fn work(kind: EffectKind) -> Effect { Effect { source: Source::None, target: None, kind } }
fn queue(c: &mut Combat, kind: EffectKind) {
    c.state.phase = Phase::EnemyTurn;
    c.state.pending.push_back(work(kind));
}
fn jaw() -> Combat {
    let mut s = base_scenario();
    s.encounter_id = EnemyKind::JawWorm;
    let e = &mut s.enemies[0];
    e.enemy_id = EnemyKind::JawWorm; e.hp = 40; e.max_hp = 40;
    e.intent_id = MoveId::Chomp; e.move_history = vec![MoveId::Chomp];
    let mut c = Combat::new(&s).unwrap();
    // Independently derived HMAC word 4050467930481678147: bucket 47, no fallback.
    c.state.rng.enemy_ai = StreamState::new(c.state.rng.enemy_ai.id().clone(), [1; 32], 0).unwrap();
    c
}
fn power(id: PowerId, amount: i32, skip: bool, grace: bool) -> PowerEntry {
    PowerEntry { id, state: PowerState { amount, skip_first: skip, monster_source_grace: grace } }
}
fn kinds(c: &Combat) -> Vec<EffectKind> { c.state.pending.iter().map(|w| w.kind.clone()).collect() }
fn dispatch(c: &mut Combat) {
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault, None);
    assert!(c.state.current.is_none());
    let owners = c.state.piles.hand.iter().chain(&c.state.piles.draw)
        .chain(&c.state.piles.discard).chain(&c.state.piles.exhaust)
        .chain(c.state.in_play.iter()).map(|card| card.instance_id.clone()).collect::<Vec<_>>();
    let unique = owners.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(owners.len(), unique.len());
    assert_eq!(unique, c.state.card_ids_issued);
}
fn select_published(c: &mut Combat, id: MoveId) {
    let e = &mut c.state.enemies[0];
    e.selected_move = id; e.intent_id = id; e.move_history.push(id);
    effects::refresh_enemy_values(&mut c.state).unwrap();
}

#[test]
fn cultist_ritual_first_skip_and_growth() {
    let mut c = combat();
    let ai = c.state.rng.enemy_ai.clone();
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Decision);
    assert_eq!((c.state.player.hp, c.state.turn, c.state.resolved_effects), (40, 2, 11));
    assert_eq!(c.state.enemies[0].powers, vec![power(PowerId::Ritual, 3, false, false)]);
    assert_eq!(c.state.enemies[0].intent_damage, Some(6));
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Decision);
    assert_eq!((c.state.player.hp, c.state.turn, c.state.resolved_effects), (34, 3, 12));
    assert_eq!(c.state.enemies[0].powers[1], power(PowerId::Strength, 3, false, false));
    assert_eq!(c.state.enemies[0].intent_damage, Some(9));
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Decision);
    assert_eq!((c.state.player.hp, c.state.enemies[0].intent_damage), (25, Some(12)));
    assert_eq!(c.state.rng.enemy_ai, ai);
}

#[test]
fn published_move_runs() {
    let mut c = jaw();
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Decision);
    assert_eq!(c.state.player.hp, 29); // Chomp, not the newly selected Thrash.
    assert_eq!(c.state.enemies[0].intent_id, MoveId::Thrash);
    assert_eq!(c.state.enemies[0].move_history, vec![MoveId::Chomp, MoveId::Thrash]);
    assert_eq!(c.state.enemies[0].block, 0);
    assert_eq!(c.state.rng.enemy_ai.word_index(), 1);
}

#[test]
fn jaw_move_payload_order() {
    for id in [MoveId::Thrash, MoveId::Bellow] {
        let mut c = jaw(); select_published(&mut c, id);
        c.state.enemies[0].block = 19;
        queue(&mut c, EffectKind::MonsterStart);
        dispatch(&mut c);
        assert_eq!(c.state.enemies[0].block, 0);
        assert_eq!(kinds(&c), vec![EffectKind::EnemyMove]);
        dispatch(&mut c);
        let payload = if id == MoveId::Thrash {
            vec![EffectKind::Damage(DamagePayload::EnemyMove(id)), EffectKind::GainBlock(5)]
        } else { vec![EffectKind::ApplyPower { id: PowerId::Strength, amount: 3 }, EffectKind::GainBlock(6)] };
        assert_eq!(kinds(&c), [payload, vec![EffectKind::RollMove, EffectKind::EndRound]].concat());
        assert_eq!(c.state.pending[2], Effect {
            source: Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone())),
            target: None, kind: EffectKind::RollMove });
        dispatch(&mut c);
        assert_eq!(c.state.enemies[0].block, 0);
        if id == MoveId::Thrash { assert_eq!(c.state.player.hp, 33); }
        else {
            assert_eq!(c.state.enemies[0].powers, vec![power(PowerId::Strength, 3, false, false)]);
            assert_eq!(c.state.enemies[0].damage_entries[&MoveId::Chomp].output, 14);
        }
        dispatch(&mut c);
        assert_eq!(c.state.enemies[0].block, if id == MoveId::Thrash { 5 } else { 6 });
    }
}

#[test]
fn next_player_construction_before_ritual_payload() {
    let mut s = base_scenario(); s.piles.draw = s.piles.hand.split_off(1); s.player.draw_per_turn = 2;
    let mut c = Combat::new(&s).unwrap();
    c.state.player.energy = 0; c.state.player.block = 17;
    c.state.enemies[0].powers.push(power(PowerId::Ritual, 3, false, false));
    c.state.enemies[0].selected_move = MoveId::Attack;
    c.state.enemies[0].move_history.push(MoveId::Attack);
    queue(&mut c, EffectKind::EndRound);
    dispatch(&mut c);
    assert_eq!((c.state.phase, c.state.turn, c.state.player.energy, c.state.player.block), (Phase::PlayerStart, 2, 3, 0));
    assert_eq!(c.state.enemies[0].intent_id, MoveId::Attack);
    assert_eq!(c.state.enemies[0].intent_damage, Some(6));
    assert_eq!(c.state.enemies[0].powers.len(), 1);
    assert_eq!(c.state.piles.hand.len(), 1);
    assert_eq!(kinds(&c), vec![EffectKind::ApplyPower { id: PowerId::Strength, amount: 3 }, EffectKind::Draw(2), EffectKind::Decision]);
    assert_eq!(c.state.pending[1], Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Draw(2) });
    assert_eq!(c.public_view(), Err(AccessError::NotReady));
    let captured = c.snapshot(); let mut restored = Combat::restore(&captured).unwrap();
    dispatch(&mut c);
    assert_eq!(c.state.enemies[0].intent_damage, Some(9));
    assert_eq!(c.state.piles.hand.len(), 1);
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision);
    assert_eq!(engine::resolve(&mut restored.state).status, TransitionStatus::Decision);
    assert_eq!(c.snapshot(), restored.snapshot());
    assert_eq!(c.state.piles.hand.iter().map(|c| c.instance_id.as_str()).collect::<Vec<_>>(), ["1", "3", "2"]);
    assert_eq!(c.state.public.revealed_order.iter().map(|id| id.as_str()).collect::<Vec<_>>(), ["3", "2"]);
}

#[test]
fn vulnerable_expiry_preserves_grace() {
    let mut c = combat();
    let enemy = Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let apply = |target: Actor, amount| Effect { source: Source::Actor(enemy.clone()), target: Some(target),
        kind: EffectKind::ApplyPower { id: PowerId::Vulnerable, amount } };
    effects::apply_effect(&mut c.state, &apply(Actor::Player, 1)).unwrap();
    effects::apply_effect(&mut c.state, &apply(Actor::Player, 1)).unwrap();
    c.state.enemies[0].powers.push(power(PowerId::Vulnerable, 1, false, false));
    queue(&mut c, EffectKind::EndRound);
    dispatch(&mut c);
    assert_eq!(c.state.player.powers, vec![power(PowerId::Vulnerable, 2, false, false)]);
    assert_eq!(kinds(&c), vec![EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 }, EffectKind::Draw(5), EffectKind::Decision]);
    dispatch(&mut c); // Enemy reaches zero; removal is a separate safe point.
    assert!(Combat::restore(&c.snapshot()).is_ok());
    assert_eq!(c.state.enemies[0].powers[0].state.amount, 0);
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision);
    assert!(c.state.enemies[0].powers.is_empty());
    effects::apply_effect(&mut c.state, &apply(Actor::Player, 1)).unwrap();
    assert_eq!(c.state.player.powers[0].state, power(PowerId::Vulnerable, 3, false, false).state);
    queue(&mut c, EffectKind::EndRound); dispatch(&mut c);
    assert_eq!(c.state.pending[0].target, Some(Actor::Player));
    assert_eq!(c.state.pending[0].kind, EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 });
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision);
    assert_eq!(c.state.player.powers[0].state.amount, 2);
}

#[test]
fn roll_installs_then_appends_once() {
    let mut c = jaw();
    queue(&mut c, EffectKind::RollMove);
    c.state.pending.push_back(work(EffectKind::EndRound));
    dispatch(&mut c);
    assert_eq!(c.state.enemies[0].selected_move, MoveId::Thrash);
    assert_eq!(c.state.enemies[0].intent_id, MoveId::Chomp);
    assert_eq!(c.state.enemies[0].intent_damage, Some(11));
    assert_eq!(c.state.enemies[0].move_history, [MoveId::Chomp, MoveId::Thrash]);
    assert_eq!(c.state.rng.enemy_ai.word_index(), 1);
    assert!(Combat::restore(&c.snapshot()).is_ok());
    dispatch(&mut c);
    assert_eq!(c.state.enemies[0].intent_id, MoveId::Thrash);
    assert_eq!(c.state.enemies[0].intent_damage, Some(7));
    assert_eq!(c.state.enemies[0].move_history, [MoveId::Chomp, MoveId::Thrash]);
    assert_eq!(c.state.rng.enemy_ai.word_index(), 1);
}

#[test]
fn turn_surface_limit_is_explicit() {
    let mut c = combat(); c.state.turn = i32::MAX as u32;
    c.state.player.energy = 0; c.state.player.block = 5;
    queue(&mut c, EffectKind::EndRound);
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::Unsupported);
    assert!(c.state.fault.as_ref().unwrap().message.contains("turn"));
    assert_eq!(c.state.turn, i32::MAX as u32);
    assert_eq!((c.state.player.energy, c.state.player.block), (0, 5));
    assert_eq!(c.state.current, Some(work(EffectKind::EndRound)));
    assert_eq!(c.state.resolved_effects, 1);
}

#[test]
fn resume_each_turn_safe_point() {
    let mut c = combat();
    for (round, count, hp, intent) in [(1, 11, 40, 6), (2, 12, 34, 9)] {
        c.state.phase = Phase::PlayerAction; c.state.resolved_effects = 0;
        c.state.pending.push_back(work(EffectKind::EndPlayerTurn));
        let mut uninterrupted = Combat::restore(&c.snapshot()).unwrap();
        assert_eq!(engine::resolve(&mut uninterrupted.state).status, TransitionStatus::Decision);
        let mut points = 0;
        while !c.state.pending.is_empty() {
            let captured = c.snapshot(); let baseline = captured.clone();
            let mut restored = Combat::restore(&captured).unwrap();
            dispatch(&mut c); dispatch(&mut restored);
            assert_eq!(snapshot::compare(&c.snapshot(), &restored.snapshot()), None);
            assert_eq!(captured, baseline);
            let mut continuation = Combat::restore(&c.snapshot()).unwrap();
            assert_eq!(engine::resolve(&mut continuation.state).status, TransitionStatus::Decision);
            assert_eq!(continuation.snapshot(), uninterrupted.snapshot());
            c = restored; points += 1;
        }
        assert_eq!(points, count);
        assert_eq!((c.state.turn, c.state.player.hp, c.state.enemies[0].intent_damage), (round + 1, hp, Some(intent)));
        assert_eq!(c.state.resolved_effects, count);
        assert_eq!(c.public_view(), uninterrupted.public_view());
        assert_eq!(c.legal_actions(), uninterrupted.legal_actions());
    }
}

#[test]
fn fallback_exhaustion_preserves_prior_turn_damage() {
    let mut c = jaw();
    // Independently pinned final word 10295784580418336610 gives bucket 10.
    // Chomp restriction requires its coin, but no next word exists.
    c.state.rng.enemy_ai = StreamState::new(c.state.rng.enemy_ai.id().clone(), [3; 32], (1u128 << 64) - 1).unwrap();
    let shuffle = c.state.rng.shuffle.clone();
    let piles = c.state.piles.clone();
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Fault);
    assert_eq!(c.state.player.hp, 29);
    assert_eq!(c.state.turn, 1);
    assert_eq!(c.state.resolved_effects, 6);
    assert_eq!(c.state.current, Some(Effect {
        source: Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone())),
        target: None, kind: EffectKind::RollMove }));
    assert_eq!(kinds(&c), [EffectKind::EndRound]);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
    assert_eq!(c.state.rng.enemy_ai.word_index(), 1u128 << 64);
    assert_eq!(c.state.rng.shuffle, shuffle);
    assert_eq!(c.state.enemies[0].move_history, [MoveId::Chomp]);
    assert_eq!(c.state.enemies[0].selected_move, MoveId::Chomp);
    assert_eq!((c.state.enemies[0].intent_id, c.state.enemies[0].intent_damage), (MoveId::Chomp, Some(11)));
    assert!(c.state.piles.hand.is_empty());
    assert_eq!(c.state.piles.discard, piles.hand.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(c.state.outcome, None);
    let captured = c.snapshot();
    assert_eq!(c.step(&Action::EndTurn), Err(StepError::Faulted));
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Fault);
    assert_eq!(c.snapshot(), captured);
    assert_eq!(Combat::restore(&captured), Err(RestoreError::Faulted));
}

#[test]
fn forced_fallback_error_uses_selection_installation_boundary() {
    use super::super::{rules, turns};
    use super::super::rng::RngError;
    struct ExhaustCoin { latest: StreamState, coins: usize }
    impl rules::SelectionDraws for ExhaustCoin {
        fn bucket(&mut self) -> Result<u8, RngError> { Ok(0) }
        fn coin(&mut self, probability: rules::Ratio) -> Result<bool, RngError> {
            assert_eq!(probability, rules::Ratio { numerator: 9, denominator: 16 });
            self.coins += 1;
            Err(RngError::Exhausted { state: self.latest.clone() })
        }
    }
    let mut c = jaw();
    queue(&mut c, EffectKind::MonsterStart);
    dispatch(&mut c); dispatch(&mut c); dispatch(&mut c); // Published Chomp already damaged.
    assert_eq!(c.state.player.hp, 29);
    let mut draws = ExhaustCoin { latest: StreamState::new(c.state.rng.enemy_ai.id().clone(),
        *c.state.rng.enemy_ai.key(), 1u128 << 64).unwrap(), coins: 0 };
    // Test-only interruption at the selection call: same selector and result owner,
    // then the same resolver error installation; no product RNG substitution.
    c.state.current = c.state.pending.pop_front(); c.state.resolved_effects += 1;
    let result = rules::select(EnemyKind::JawWorm, &c.state.enemies[0].move_history, &mut draws)
        .map(|id| (id, draws.latest.clone()));
    let error = turns::install_selection(&mut c.state, result).unwrap_err();
    engine::retain_error(&mut c.state, error);
    assert_eq!(draws.coins, 1);
    assert_eq!(c.state.current, Some(Effect {
        source: Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone())),
        target: None, kind: EffectKind::RollMove }));
    assert_eq!(c.state.resolved_effects, 4);
    assert_eq!(c.state.player.hp, 29);
    assert_eq!(c.state.enemies[0].move_history, [MoveId::Chomp]);
    assert_eq!(c.state.enemies[0].intent_id, MoveId::Chomp);
    assert_eq!(c.state.rng.enemy_ai, draws.latest);
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
    assert_eq!(kinds(&c), [EffectKind::EndRound]);
}

#[test]
fn end_player_prepends_and_round_hooks_append_in_insertion_order() {
    let mut c = combat();
    queue(&mut c, EffectKind::EndPlayerTurn);
    let tail = Effect { source: Source::None, target: Some(Actor::Player), kind: EffectKind::GainBlock(0) };
    c.state.pending.push_back(tail.clone());
    dispatch(&mut c);
    assert_eq!(kinds(&c), [EffectKind::DiscardHand, EffectKind::MonsterStart, EffectKind::GainBlock(0)]);
    assert_eq!(c.state.pending.back(), Some(&tail));
    dispatch(&mut c);
    assert_eq!(c.state.piles.discard.iter().map(|c| c.instance_id.as_str()).collect::<Vec<_>>(), ["3", "2", "1"]);
    c.state.pending.clear();
    c.state.player.powers.push(power(PowerId::Vulnerable, 2, false, false));
    c.state.enemies[0].powers = vec![power(PowerId::Vulnerable, 2, false, false), power(PowerId::Ritual, 3, false, false)];
    queue(&mut c, EffectKind::EndRound); c.state.pending.push_back(tail.clone());
    dispatch(&mut c);
    assert_eq!(c.state.pending.front(), Some(&tail));
    assert_eq!(kinds(&c), [EffectKind::GainBlock(0),
        EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 },
        EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 },
        EffectKind::ApplyPower { id: PowerId::Strength, amount: 3 }, EffectKind::Draw(5), EffectKind::Decision]);
    assert_eq!(c.state.pending[1].target, Some(Actor::Player));
    assert_eq!(c.state.pending[2].target, Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())));
    assert_eq!(c.state.pending[3].target, c.state.pending[2].target);
    assert!(Combat::restore(&c.snapshot()).is_ok());
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision);
}

#[test]
fn queued_enemy_attack_retains_live_damage_reference() {
    let mut c = combat(); select_published(&mut c, MoveId::Attack);
    queue(&mut c, EffectKind::EnemyMove); dispatch(&mut c);
    let attack = c.state.pending.front().unwrap().clone();
    assert_eq!(attack.kind, EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Attack)));
    let enemy = Actor::Enemy(c.state.enemies[0].instance_id.clone());
    effects::apply_effect(&mut c.state, &Effect { source: Source::Actor(enemy.clone()), target: Some(enemy),
        kind: EffectKind::ApplyPower { id: PowerId::Strength, amount: 3 } }).unwrap();
    dispatch(&mut c);
    assert_eq!(c.state.player.hp, 31); // Not enqueue-time 6.
    assert_eq!(c.state.enemies[0].intent_damage, Some(9));
}

#[test]
fn lethal_published_move_prunes_selection_and_next_turn() {
    let mut c = jaw(); c.state.player.hp = 11;
    c.state.rng.enemy_ai = StreamState::new(c.state.rng.enemy_ai.id().clone(), [3; 32], 1u128 << 64).unwrap();
    let rng = c.state.rng.clone();
    assert_eq!(c.step(&Action::EndTurn).unwrap().status, TransitionStatus::Terminal);
    assert_eq!(c.state.resolved_effects, 6);
    assert_eq!(c.state.turn, 1);
    assert_eq!(c.state.rng, rng);
    assert_eq!(c.state.enemies[0].move_history, [MoveId::Chomp]);
    assert_eq!((c.state.hp_before_healing, c.state.hp_after_cleanup), (Some(0), Some(0)));
    assert_eq!(c.state.outcome, Some(Outcome::Death));
    assert_eq!(Combat::restore(&c.snapshot()).unwrap().snapshot(), c.snapshot());
}

#[test]
fn resume_jaw_payloads_and_round_expiry_safe_points() {
    for id in [MoveId::Chomp, MoveId::Thrash, MoveId::Bellow] {
        let mut c = jaw();
        if id != MoveId::Chomp { select_published(&mut c, id); }
        c.state.player.powers.push(power(PowerId::Vulnerable, 1, false, false));
        c.state.enemies[0].powers.push(power(PowerId::Vulnerable, 1, false, false));
        effects::refresh_enemy_values(&mut c.state).unwrap();
        queue(&mut c, EffectKind::EndPlayerTurn);
        let mut expected = Combat::restore(&c.snapshot()).unwrap();
        assert_eq!(engine::resolve(&mut expected.state).status, TransitionStatus::Decision);
        while !c.state.pending.is_empty() {
            let captured = c.snapshot(); let baseline = captured.clone();
            let mut restored = Combat::restore(&captured).unwrap();
            dispatch(&mut c); dispatch(&mut restored);
            assert_eq!(snapshot::compare(&c.snapshot(), &restored.snapshot()), None);
            assert_eq!(captured, baseline);
            let mut finish = Combat::restore(&c.snapshot()).unwrap();
            assert_eq!(engine::resolve(&mut finish.state).status, TransitionStatus::Decision);
            assert_eq!(finish.snapshot(), expected.snapshot());
            c = restored;
        }
        assert!(c.state.player.powers.is_empty());
        assert!(!c.state.enemies[0].powers.iter().any(|p| p.id == PowerId::Vulnerable));
        assert_eq!(c.public_view(), expected.public_view());
        assert_eq!(c.legal_actions(), expected.legal_actions());
    }
}

#[test]
fn growing_round_caps_strength_and_preserves_hook_order() {
    let mut c = combat();
    select_published(&mut c, MoveId::Attack);
    c.state.enemies[0].powers = vec![power(PowerId::Strength, 998, false, false),
        power(PowerId::Ritual, 3, false, false)];
    effects::refresh_enemy_values(&mut c.state).unwrap();
    queue(&mut c, EffectKind::EndRound); dispatch(&mut c);
    assert_eq!(c.state.enemies[0].intent_damage, Some(1004));
    dispatch(&mut c);
    assert_eq!(c.state.enemies[0].intent_damage, Some(1005));
    assert_eq!(c.state.enemies[0].powers, vec![power(PowerId::Strength, 999, false, false),
        power(PowerId::Ritual, 3, false, false)]);
    assert!(Combat::restore(&c.snapshot()).is_ok());
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision);
}

#[test]
fn turn_owner_checks_direct_payload_and_identity_before_mutation() {
    use super::super::turns;
    let mut c = combat();
    for effect in [work(EffectKind::Draw(1)), Effect { source: Source::Actor(Actor::Enemy(EnemyHandle::parse("foreign").unwrap())),
        target: None, kind: EffectKind::EndRound }] {
        let before = c.snapshot();
        assert!(matches!(turns::apply_effect(&mut c.state, &effect), Err(EffectError::Invalid(InputError::Invariant(_)))));
        assert_eq!(c.snapshot(), before);
    }
}

#[test]
fn next_player_draw_provenance_survives_split_replay_and_recycle_fault() {
    let mut c = combat();
    c.state.rng.shuffle = StreamState::new(c.state.rng.shuffle.id().clone(),
        *c.state.rng.shuffle.key(), (1u128 << 64) - 1).unwrap();
    queue(&mut c, EffectKind::EndPlayerTurn);
    let mut saw_draw = false;
    let mut saw_split = false;
    while c.state.fault.is_none() {
        if let Some(front) = c.state.pending.front() {
            if matches!(front.kind, EffectKind::Draw(_) | EffectKind::Recycle) {
                assert_eq!(front.source, Source::Actor(Actor::Player));
                assert_eq!(front.target, Some(Actor::Player));
                saw_draw |= matches!(front.kind, EffectKind::Draw(_));
                saw_split |= matches!(front.kind, EffectKind::Recycle);
            }
        }
        let captured = c.snapshot();
        let baseline = captured.clone();
        let mut restored = Combat::restore(&captured).unwrap();
        engine::dispatch_one(&mut c.state);
        engine::dispatch_one(&mut restored.state);
        assert_eq!(c.snapshot(), restored.snapshot());
        assert_eq!(captured, baseline);
    }
    assert!(saw_draw && saw_split);
    assert_eq!(c.state.current, Some(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Recycle }));
    assert_eq!(c.state.pending[0], Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Draw(3) });
    assert_eq!(c.state.pending[1], work(EffectKind::Decision));
    assert_eq!((c.state.turn, c.state.player.hp, c.state.resolved_effects), (2, 40, 9));
    assert_eq!(c.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
    let faulted = c.snapshot();
    assert_eq!(c.step(&Action::EndTurn), Err(StepError::Faulted));
    assert_eq!(c.snapshot(), faulted);
}
