use super::super::*;
use super::super::{engine, model};
use super::super::rng::{Domain, StreamId, StreamState};
use super::support::base_scenario;

fn play(combat: &Combat, id: &str) -> Action {
    Action::PlayCard { card_instance_id: CardHandle::parse(id).unwrap(),
        target_id: Some(combat.state.enemies[0].instance_id.clone()) }
}

fn assert_matches(combat: &Combat) {
    let p = combat.public_projection().unwrap();
    let v = combat.public_view().unwrap();
    assert_eq!(p.turn, v.turn);
    assert_eq!(p.player, v.player);
    assert_eq!(p.enemies, v.enemies);
    assert_eq!(p.draw_per_turn, 5);
    assert_eq!(p.draw_count, combat.public_view().unwrap().draw_count);
    assert_eq!(p.outcome, v.outcome);
    assert_eq!(p.hp_before_healing, v.hp_before_healing);
    assert_eq!(p.hp_after_cleanup, v.hp_after_cleanup);
    for (borrowed, detached, storage) in [
        (p.hand, &v.hand, &combat.state.piles.hand),
        (p.discard, &v.discard, &combat.state.piles.discard),
        (p.exhaust, &v.exhaust, &combat.state.piles.exhaust),
    ] {
        assert_eq!(borrowed.as_ptr(), storage.as_ptr());
        assert_eq!(borrowed.len(), storage.len());
        assert_eq!(borrowed.iter().map(|c| CardView { instance_id: c.instance_id.clone(),
            card_id: c.card_id, upgraded: c.upgraded }).collect::<Vec<_>>(), *detached);
    }
    assert!(combat.fault_record().is_none());
}

fn ids(cards: &[CardInstance]) -> Vec<&str> {
    cards.iter().map(|c| c.instance_id.as_str()).collect()
}

#[test]
fn projection_matches_public_values_without_history() {
    let mut combat = Combat::new(&base_scenario()).unwrap();
    assert_matches(&combat);
    let p = combat.public_projection().unwrap();
    assert_eq!(ids(p.hand), ["1", "2", "3"]);
    assert!(p.discard.is_empty() && p.exhaust.is_empty());
    assert_eq!((p.turn, p.player.hp, p.draw_count), (1, 40, 0));
    assert_eq!((p.hp_before_healing, p.hp_after_cleanup), (None, None));

    combat.step(&play(&combat, "3")).unwrap();
    assert_matches(&combat);
    let p = combat.public_projection().unwrap();
    assert_eq!(ids(p.hand), ["1", "2"]);
    assert_eq!(ids(p.discard), ["3"]);
    assert_eq!((p.enemies[0].hp, p.player.energy), (40, 1));
    assert_eq!(p.enemies[0].powers, [(PowerId::Vulnerable, 2)]);

    combat.step(&Action::EndTurn).unwrap();
    assert_matches(&combat);
    let p = combat.public_projection().unwrap();
    assert_eq!((p.turn, p.player.hp, p.player.energy, p.draw_count), (2, 40, 3, 0));
    let mut hand = ids(p.hand);
    hand.sort();
    assert_eq!(hand, ["1", "2", "3"]);
    assert!(p.discard.is_empty() && p.exhaust.is_empty());

    // Nonempty visible piles and a terminal in-play card must remain untouched.
    let mut victory = Combat::new(&base_scenario()).unwrap();
    victory.state.piles.exhaust.push(victory.state.piles.hand.remove(1));
    victory.state.piles.discard.push(victory.state.piles.hand.remove(1));
    victory.state.enemies[0].hp = 1;
    victory.step(&play(&victory, "1")).unwrap();
    assert_matches(&victory);
    let p = victory.public_projection().unwrap();
    assert_eq!(p.outcome, Some(Outcome::Victory));
    assert_eq!((p.hp_before_healing, p.hp_after_cleanup), (Some(40), Some(46)));
    assert!(p.hand.is_empty());
    assert_eq!(ids(p.discard), ["3"]);
    assert_eq!(ids(p.exhaust), ["2"]);
    assert_eq!(victory.state.in_play.as_ref().unwrap().instance_id.as_str(), "1");

    let mut death = Combat::new(&base_scenario()).unwrap();
    death.state.phase = Phase::EnemyTurn;
    death.state.pending.push_back(Effect { source: Source::None,
        target: Some(Actor::Player), kind: EffectKind::Damage(DamagePayload::Fixed(40)) });
    assert_eq!(engine::resolve(&mut death.state).status, TransitionStatus::Terminal);
    assert_matches(&death);
    let p = death.public_projection().unwrap();
    assert_eq!(p.outcome, Some(Outcome::Death));
    assert_eq!((p.hp_before_healing, p.hp_after_cleanup), (Some(0), Some(0)));
    assert_eq!(ids(p.hand), ["1", "2", "3"]);
}

#[test]
fn projection_hides_private_changes_and_does_not_copy_reveals() {
    let mut scenario = base_scenario();
    scenario.piles.draw = scenario.piles.hand.split_off(1);
    let mut a = Combat::new(&scenario).unwrap();
    a.state.player.powers.push(PowerEntry { id: PowerId::Strength,
        state: PowerState { amount: 3, skip_first: false, monster_source_grace: false } });
    a.state.enemies[0].powers = vec![
        PowerEntry { id: PowerId::Vulnerable, state: PowerState {
            amount: 2, skip_first: false, monster_source_grace: false } },
        PowerEntry { id: PowerId::Ritual, state: PowerState {
            amount: 3, skip_first: true, monster_source_grace: false } },
    ];
    let mut b = a.clone();
    b.state.piles.draw.reverse();
    b.state.rng.shuffle = StreamState::new(StreamId::new(Domain::CombatShuffle,
        vec!["different".into()]).unwrap(), [21; 32], 18).unwrap();
    b.state.rng.enemy_ai = StreamState::new(StreamId::new(Domain::CombatEnemyAi,
        vec!["different".into(), "private".into()]).unwrap(), [22; 32], 33).unwrap();
    b.state.enemies[0].selected_move = MoveId::Attack;
    b.state.enemies[0].move_history.push(MoveId::Attack);
    b.state.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().output = 9;
    b.state.player.powers[0].state.skip_first = true;
    b.state.player.powers[0].state.monster_source_grace = true;
    b.state.enemies[0].powers[0].state.monster_source_grace = true;
    b.state.enemies[0].powers[1].state.skip_first = false;
    b.state.public.revealed_order = vec![CardHandle::parse("2").unwrap(); 100_000];
    assert_ne!(a.public_view(), b.public_view());
    let before = b.snapshot();
    assert_eq!(a.public_projection(), b.public_projection());
    assert_eq!(b.public_projection().unwrap().enemies[0].powers,
        [(PowerId::Ritual, 3), (PowerId::Vulnerable, 2)]);
    assert_eq!(b.snapshot(), before);
    assert!(b.fault_record().is_none());

    // Every readiness condition has the same rejection as the detached API.
    for case in 0..4 {
        let mut busy = a.clone();
        let effect = Effect { source: Source::None, target: None, kind: EffectKind::Decision };
        match case {
            0 => busy.state.phase = Phase::PlayerAction,
            1 => busy.state.current = Some(effect),
            2 => busy.state.pending.push_back(effect),
            _ => busy.state.in_play = Some(busy.state.piles.hand.remove(0)),
        }
        assert_eq!(busy.public_view().unwrap_err(), AccessError::NotReady);
        assert_eq!(busy.public_projection().unwrap_err(), AccessError::NotReady);
        assert!(busy.fault_record().is_none());
    }

    b.state.resolved_effects = 17;
    model::mark_fault(&mut b.state, FaultKind::Invariant, "first fault".into());
    b.state.resolved_effects = 29;
    model::mark_fault(&mut b.state, FaultKind::EffectGuard, "later fault".into());
    b.state.phase = Phase::EnemyTurn;
    assert_eq!(b.public_view().unwrap_err(), AccessError::Faulted);
    assert_eq!(b.public_projection().unwrap_err(), AccessError::Faulted);
    let expected = FaultRecord { kind: FaultKind::Invariant,
        message: "first fault".into(), resolved_effects: 17 };
    assert_eq!(b.fault_record(), Some(expected.clone()));
    let mut detached = b.fault_record().unwrap();
    detached.message.clear();
    detached.kind = FaultKind::Unsupported;
    detached.resolved_effects = 0;
    assert_eq!(b.fault_record(), Some(expected));
}
