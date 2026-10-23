use super::support::base_scenario;
use super::super::*;
use super::super::rng::{Domain, StreamId, StreamState};
use super::super::snapshot::compare;

fn reject(change: impl FnOnce(&mut Scenario)) {
    let mut scenario = base_scenario();
    change(&mut scenario);
    let baseline = scenario.clone();
    assert!(Combat::new(&scenario).is_err(), "accepted {scenario:?}");
    assert_eq!(scenario, baseline);
}

#[test]
fn supplied_entry_derives_without_sampling() {
    let scenario = base_scenario();
    let combat = Combat::new(&scenario).unwrap();
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.turn, 1);
    assert_eq!(combat.state.resolved_effects, 0);
    assert_eq!(combat.snapshot().schema_version, 2);
    assert_eq!(combat.state.enemies[0].move_history, vec![MoveId::Incantation]);
    assert!(combat.state.pending.is_empty() && combat.state.current.is_none());
    assert_eq!(combat.state.rng, scenario.rng);
    assert_eq!(combat.state.enemies[0].damage_entries[&MoveId::Attack], DamageEntry { base: 6, output: 6 });
    assert_eq!(combat.state.enemies[0].intent_damage, None);
    let mut jaw = scenario;
    jaw.encounter_id = EnemyKind::JawWorm;
    let enemy = &mut jaw.enemies[0];
    enemy.enemy_id = EnemyKind::JawWorm;
    enemy.hp = 40; enemy.max_hp = 40;
    enemy.intent_id = MoveId::Chomp; enemy.move_history = vec![MoveId::Chomp];
    let combat = Combat::new(&jaw).unwrap();
    assert_eq!(combat.state.enemies[0].damage_entries[&MoveId::Chomp], DamageEntry { base: 11, output: 11 });
    assert_eq!(combat.state.enemies[0].damage_entries[&MoveId::Thrash], DamageEntry { base: 7, output: 7 });
    assert_eq!(combat.state.enemies[0].intent_damage, Some(11));
}

#[test]
fn entry_bounds_and_stream_owner() {
    for n in [0, 1000] {
        reject(|s| s.player.max_hp = n);
        reject(|s| { s.player.max_energy = n; s.player.energy = n; });
    }
    for n in [-1, 0, 51] { reject(|s| s.player.hp = n); }
    for n in [0, 11] { reject(|s| s.player.draw_per_turn = n); }
    reject(|s| s.player.energy = 2);
    reject(|s| s.player.block = 1);
    reject(|s| s.player.powers.push(power(PowerId::Strength, 1, false, false)));
    reject(|s| s.piles.hand.clear());
    reject(|s| s.piles.draw = (4..=1000).map(|n| CardInstance {
        instance_id: CardHandle::parse(&n.to_string()).unwrap(), card_id: CardKind::Strike, upgraded: false }).collect());
    reject(|s| s.piles.hand = (1..=11).map(|n| CardInstance {
        instance_id: CardHandle::parse(&n.to_string()).unwrap(), card_id: CardKind::Strike, upgraded: false }).collect());
    reject(|s| s.piles.draw.push(s.piles.hand[0].clone()));
    reject(|s| s.piles.discard.push(s.piles.hand.remove(0)));
    reject(|s| s.piles.exhaust.push(s.piles.hand.remove(0)));
    reject(|s| s.enemies.clear());
    reject(|s| s.enemies.push(s.enemies[0].clone()));
    reject(|s| s.encounter_id = EnemyKind::JawWorm);
    reject(|s| s.enemies[0].intent_id = MoveId::Attack);
    reject(|s| s.enemies[0].move_history.push(MoveId::Incantation));
    reject(|s| s.enemies[0].move_history.clear());
    reject(|s| s.enemies[0].hp -= 1);
    for n in [47, 55] { reject(|s| { s.enemies[0].hp = n; s.enemies[0].max_hp = n; }); }
    reject(|s| s.enemies[0].block = 1);
    reject(|s| s.enemies[0].powers.push(power(PowerId::Ritual, 3, true, false)));
    reject(|s| s.rng.enemy_instance_id = EnemyHandle::parse("other").unwrap());
    reject(|s| s.rng.contract_id = "wrong".into());
    reject(|s| s.rng.shuffle = s.rng.enemy_ai.clone());
    reject(|s| s.rng.enemy_ai = stream(Domain::CombatEnemyHp, &["combat-plan-1", "enemy-scope-1"]));
    reject(|s| s.rng.enemy_ai = stream(Domain::CombatEnemyAi, &["other", "enemy-scope-1"]));
    // Public owner is intentionally unrelated to the private scope suffix.
    let mut valid = base_scenario();
    valid.player.max_hp = 999; valid.player.hp = 999;
    valid.player.max_energy = 999; valid.player.energy = 999; valid.player.draw_per_turn = 10;
    valid.piles.draw = (4..=999).map(|n| CardInstance {
        instance_id: CardHandle::parse(&n.to_string()).unwrap(), card_id: CardKind::Defend, upgraded: true }).collect();
    assert!(Combat::new(&valid).is_ok());
    valid.player.max_hp = 1; valid.player.hp = 1; valid.player.max_energy = 1; valid.player.energy = 1;
    valid.player.draw_per_turn = 1; valid.piles.hand.truncate(1); valid.piles.draw.clear();
    assert!(Combat::new(&valid).is_ok());
}

fn stream(domain: Domain, scope: &[&str]) -> StreamState {
    StreamState::new(StreamId::new(domain, scope.iter().map(|s| s.to_string()).collect()).unwrap(), [7; 32], 13).unwrap()
}

fn power(id: PowerId, amount: i32, skip_first: bool, monster_source_grace: bool) -> PowerEntry {
    PowerEntry { id, state: PowerState { amount, skip_first, monster_source_grace } }
}

#[test]
fn opaque_handles_are_lossless() {
    for text in ["-1", "0", "1267650600228229401496703205376"] {
        let handle = CardHandle::parse(text).unwrap();
        assert_eq!(handle.as_str(), text);
        assert_eq!(CardHandle::parse(handle.as_str()).unwrap(), handle);
    }
    for text in ["01", "-0", "+1", "", "x", "1.0", " 1", "١", "--1", "-"] {
        assert!(CardHandle::parse(text).is_err(), "{text}");
    }
    for text in ["", "player"] { assert!(EnemyHandle::parse(text).is_err()); }
    assert_eq!(EnemyHandle::parse("public enemy").unwrap().as_str(), "public enemy");
}

#[test]
fn wide_signed_decimal_legacy_handle_survives_entry_step_and_snapshot() {
    let negative = "-1267650600228229401496703205376";
    let positive = "1267650600228229401496703205376";
    let mut scenario = base_scenario();
    scenario.piles.hand[0].instance_id = CardHandle::parse(negative).unwrap();
    scenario.piles.hand[1].instance_id = CardHandle::parse(positive).unwrap();
    let mut combat = Combat::new(&scenario).unwrap();
    assert_eq!(combat.state.piles.hand[0].legacy().unwrap().instance_id.as_str(), negative);
    assert_eq!(combat.state.piles.hand[1].legacy().unwrap().instance_id.as_str(), positive);
    combat.step(&Action::PlayCard {
        card_instance_id: CardHandle::parse(negative).unwrap(),
        target_id: Some(scenario.enemies[0].instance_id.clone()),
    }).unwrap();
    combat.step(&Action::PlayCard {
        card_instance_id: CardHandle::parse(positive).unwrap(), target_id: None,
    }).unwrap();
    assert_eq!(combat.state.piles.discard[0].handle().as_str(), negative);
    assert_eq!(combat.state.piles.discard[1].handle().as_str(), positive);
    let restored = Combat::restore(&combat.snapshot()).unwrap();
    assert_eq!(restored.state.piles.discard[0].handle().as_str(), negative);
    assert_eq!(restored.state.piles.discard[1].handle().as_str(), positive);
}

#[test]
fn entry_and_restore_are_detached() {
    let mut scenario = base_scenario();
    let mut live = Combat::new(&scenario).unwrap();
    let captured = live.snapshot();
    let baseline = captured.clone();
    let mut restored = Combat::restore(&captured).unwrap();
    scenario.player.hp = 1; scenario.piles.hand.clear(); scenario.enemies[0].move_history.clear();
    assert_eq!(live.snapshot(), baseline);
    live.state.player.hp = 2; live.state.piles.hand.swap(0, 1); live.state.enemies[0].move_history.push(ActorIntentId::Legacy(MoveId::Attack));
    assert_eq!(restored.snapshot(), baseline);
    restored.state.player.hp = 3; restored.state.piles.draw.push(restored.state.piles.hand.remove(0));
    assert_eq!(captured, baseline);
    assert_eq!(scenario.player.hp, 1);
    assert_eq!(live.state.player.hp, 2);
}

#[test]
fn snapshot_compatibility_precedes_fault() {
    let mut snap = Combat::new(&base_scenario()).unwrap().snapshot();
    snap.state.fault = Some(FaultRecord { kind: FaultKind::Invariant, message: "diagnostic".into(), resolved_effects: 0 });
    assert_eq!(Combat::restore(&snap).unwrap_err(), RestoreError::Faulted);
    for which in 0..5 {
        let mut bad = snap.clone();
        match which { 0 => bad.schema_version = 1, 1 => bad.content_id = "wrong".into(),
            2 => bad.rng_contract_id = "wrong".into(), 3 => bad.state.content_id = "wrong".into(),
            _ => bad.state.rng.contract_id = "wrong".into() }
        assert_eq!(Combat::restore(&bad).unwrap_err(), RestoreError::Compatibility);
    }
    // Snapshot creation is an unvalidated diagnostic copy, including malformed fields.
    let mut live = Combat::new(&base_scenario()).unwrap();
    live.state.turn = 0; live.state.current = Some(effect(EffectKind::Draw(-1)));
    assert_eq!(live.snapshot().state, live.state);
}

fn effect(kind: EffectKind) -> Effect { Effect { source: Source::None, target: None, kind } }

#[test]
fn busy_snapshot_preserves_all_fields() {
    let mut live = Combat::new(&base_scenario()).unwrap();
    let s = &mut live.state;
    s.phase = Phase::PlayerAction; s.turn = 17; s.resolved_effects = 99999;
    s.in_play = Some(s.piles.hand.remove(0));
    s.player.powers.push(power(PowerId::Vulnerable, i32::MAX, false, false));
    s.enemies[0].powers.push(power(PowerId::Ritual, 3, true, false));
    s.enemies[0].powers.push(power(PowerId::Vulnerable, 2, false, true));
    s.enemies[0].selected_move = ActorIntentId::Legacy(MoveId::Attack);
    s.enemies[0].move_history.push(ActorIntentId::Legacy(MoveId::Attack));
    s.public.revealed_order.push(s.piles.hand[0].handle());
    s.rng.shuffle = stream(Domain::CombatShuffle, &["combat-plan-1"]);
    s.rng.enemy_ai = StreamState::new(StreamId::new(Domain::CombatEnemyAi,
        vec!["combat-plan-1".into(), "enemy-scope-1".into()]).unwrap(), [29; 32], 31).unwrap();
    s.pending.push_back(Effect { source: Source::Card(s.in_play.as_ref().unwrap().handle()),
        target: Some(Actor::Enemy(s.enemies[0].instance_id.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(6)) });
    s.pending.push_back(Effect { source: Source::Card(s.in_play.as_ref().unwrap().handle()),
        target: None, kind: EffectKind::FinishCard });
    s.pending.push_back(effect(EffectKind::Decision));
    let snapshot = live.snapshot();
    let baseline = snapshot.clone();
    let restored = Combat::restore(&snapshot).unwrap();
    assert_eq!(restored.snapshot(), baseline);
    assert!(compare(&snapshot, &restored.snapshot()).is_none());
    assert_eq!(restored.state.rng.shuffle.key(), &[7; 32]);
    assert_eq!(restored.state.rng.shuffle.word_index(), 13);
    assert_eq!(restored.state.rng.enemy_ai.key(), &[29; 32]);
    assert_eq!(restored.state.rng.enemy_ai.word_index(), 31);
    live.state.pending.clear(); live.state.player.powers.clear();
    assert_eq!(snapshot, baseline);
}

#[test]
fn runtime_bounds_ownership_and_lifecycle() {
    let good = Combat::new(&base_scenario()).unwrap().snapshot();
    let invalid: Vec<Box<dyn Fn(&mut super::super::model::State)>> = vec![
        Box::new(|s| s.turn = 0), Box::new(|s| s.turn = i32::MAX as u32 + 1),
        Box::new(|s| s.resolved_effects = 100000),
        Box::new(|s| s.player.block = 1000), Box::new(|s| s.player.energy = 1000),
        Box::new(|s| s.player.hp = 0), Box::new(|s| s.enemies[0].hp = 0),
        Box::new(|s| s.card_ids_issued.clear()),
        Box::new(|s| s.piles.draw.push(s.piles.hand[0].clone())),
        Box::new(|s| s.public.revealed_order.push(CardHandle::parse("999").unwrap())),
        Box::new(|s| s.enemies[0].selected_move = ActorIntentId::Legacy(MoveId::Chomp)),
        Box::new(|s| s.enemies[0].move_history.clear()),
        Box::new(|s| s.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().output = 1516),
        Box::new(|s| s.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().base = 7),
        Box::new(|s| s.enemies[0].intent_damage = Some(6)),
        Box::new(|s| s.player.powers.push(power(PowerId::Ritual, 3, true, false))),
        Box::new(|s| s.player.powers.push(power(PowerId::Strength, 1000, false, false))),
        Box::new(|s| s.player.powers.push(power(PowerId::Vulnerable, -1, false, false))),
        Box::new(|s| s.player.powers.push(power(PowerId::Strength, 1, true, false))),
        Box::new(|s| s.player.powers.extend(vec![power(PowerId::Strength, 1, false, false); 2])),
        Box::new(|s| s.pending.push_back(effect(EffectKind::Decision))),
        Box::new(|s| s.current = Some(effect(EffectKind::Decision))),
        Box::new(|s| s.in_play = Some(s.piles.hand.remove(0))),
        Box::new(|s| s.outcome = Some(Outcome::Victory)),
        Box::new(|s| s.hp_before_healing = Some(40)),
        Box::new(|s| s.hp_after_cleanup = Some(40)),
        Box::new(|s| s.phase = Phase::EnemyTurn),
    ];
    for (i, change) in invalid.into_iter().enumerate() {
        let mut bad = good.clone(); change(&mut bad.state);
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))), "case {i}");
    }
}

#[test]
fn busy_payload_validation_and_zero_hp_terminal_work() {
    let mut busy = Combat::new(&base_scenario()).unwrap().snapshot();
    busy.state.phase = Phase::EnemyTurn;
    busy.state.player.hp = 0;
    busy.state.pending.push_back(effect(EffectKind::Terminal(Outcome::Death)));
    assert!(Combat::restore(&busy).is_ok());
    busy.state.player.hp = 40; busy.state.enemies[0].hp = 0;
    busy.state.pending[0] = effect(EffectKind::Terminal(Outcome::Victory));
    assert!(Combat::restore(&busy).is_ok());
    let invalid = [EffectKind::Draw(-1), EffectKind::Draw(11), EffectKind::GainBlock(1008),
        EffectKind::Damage(DamagePayload::Fixed(1516)), EffectKind::Damage(DamagePayload::Fixed(-1)),
        EffectKind::ApplyPower { id: PowerId::Strength, amount: -1 },
        EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 0 }];
    for kind in invalid {
        let mut bad = busy.clone(); bad.state.pending.push_back(effect(kind));
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))));
    }
    for which in 0..4 {
        let mut bad = busy.clone();
        match which {
            0 => bad.state.current = Some(effect(EffectKind::Decision)),
            1 => bad.state.pending.push_back(Effect { source: Source::Card(CardHandle::parse("999").unwrap()), target: None, kind: EffectKind::Decision }),
            2 => bad.state.pending.push_back(Effect { source: Source::Actor(Actor::Enemy(EnemyHandle::parse("other").unwrap())), target: None, kind: EffectKind::Decision }),
            _ => bad.state.pending.push_back(Effect { source: Source::None, target: Some(Actor::Enemy(EnemyHandle::parse("other").unwrap())), kind: EffectKind::GainBlock(1) }),
        }
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))));
    }
}

#[test]
fn terminal_restore_requires_completed_consistent_cleanup() {
    let mut terminal = Combat::new(&base_scenario()).unwrap().snapshot();
    terminal.state.phase = Phase::Terminal; terminal.state.outcome = Some(Outcome::Victory);
    terminal.state.enemies[0].hp = 0; terminal.state.player.hp = 46;
    terminal.state.hp_before_healing = Some(40); terminal.state.hp_after_cleanup = Some(46);
    assert_eq!(Combat::restore(&terminal).unwrap().snapshot(), terminal);
    for which in 0..7 {
        let mut bad = terminal.clone();
        match which { 0 => bad.state.outcome = None, 1 => bad.state.enemies[0].hp = 1,
            2 => bad.state.hp_before_healing = Some(41), 3 => bad.state.hp_after_cleanup = Some(45),
            4 => bad.state.pending.push_back(effect(EffectKind::Decision)),
            5 => bad.state.in_play = Some(bad.state.piles.hand[0].clone()),
            _ => bad.state.player.hp = 0 }
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))), "case {which}");
    }
    // A lethal card retains its unique in-play owner after terminal work is pruned.
    let mut lethal_owner = terminal.clone();
    lethal_owner.state.in_play = Some(lethal_owner.state.piles.hand.remove(0));
    assert_eq!(Combat::restore(&lethal_owner).unwrap().snapshot(), lethal_owner);
    terminal.state.outcome = Some(Outcome::Death); terminal.state.enemies[0].hp = 48;
    terminal.state.player.hp = 0; terminal.state.hp_before_healing = Some(0); terminal.state.hp_after_cleanup = Some(0);
    assert!(Combat::restore(&terminal).is_ok());
    terminal.state.hp_before_healing = Some(1);
    assert!(Combat::restore(&terminal).is_err());
}

#[test]
fn first_component_diagnostics_are_detached() {
    let mut left = Combat::new(&base_scenario()).unwrap().snapshot();
    let mut right = left.clone();
    right.state.turn = 2; right.state.player.hp = 1;
    let difference = compare(&left, &right).unwrap();
    assert_eq!(difference.component, "turn");
    assert_eq!(difference.left, "1"); assert_eq!(difference.right, "2");
    left.state.turn = 3; right.state.turn = 4;
    assert_eq!(difference.left, "1"); assert_eq!(difference.right, "2");
    right.content_id = "other".into(); right.schema_version = 1; right.rng_contract_id = "other".into();
    assert_eq!(compare(&left, &right).unwrap().component, "content_id");
    right.content_id = left.content_id.clone();
    assert_eq!(compare(&left, &right).unwrap().component, "schema_version");
    right.schema_version = 2;
    assert_eq!(compare(&left, &right).unwrap().component, "rng_contract_id");
    right.rng_contract_id = left.rng_contract_id.clone(); right.state.content_id = "embedded".into();
    assert_eq!(compare(&left, &right).unwrap().component, "state.content_id");
}

#[test]
fn every_state_component_participates_in_comparison() {
    let left = Combat::new(&base_scenario()).unwrap().snapshot();
    macro_rules! check {
        ($field:ident, $change:expr) => {{
            let mut right = left.clone();
            ($change)(&mut right.state);
            assert_eq!(compare(&left, &right).unwrap().component, stringify!($field));
        }};
    }
    check!(phase, |s: &mut super::super::model::State| s.phase = Phase::EnemyTurn);
    check!(turn, |s: &mut super::super::model::State| s.turn = 2);
    check!(player, |s: &mut super::super::model::State| s.player.hp = 1);
    check!(enemies, |s: &mut super::super::model::State| s.enemies[0].hp = 1);
    check!(piles, |s: &mut super::super::model::State| s.piles.hand.swap(0, 1));
    check!(card_ids_issued, |s: &mut super::super::model::State| s.card_ids_issued.clear());
    check!(rng, |s: &mut super::super::model::State| s.rng.enemy_ai = stream(Domain::CombatEnemyAi, &["combat-plan-1", "enemy-scope-1"]));
    check!(pending, |s: &mut super::super::model::State| s.pending.push_back(effect(EffectKind::Draw(1))));
    check!(current, |s: &mut super::super::model::State| s.current = Some(effect(EffectKind::Draw(1))));
    check!(in_play, |s: &mut super::super::model::State| s.in_play = Some(s.piles.hand[0].clone()));
    check!(resolved_effects, |s: &mut super::super::model::State| s.resolved_effects = 2);
    check!(public, |s: &mut super::super::model::State| s.public.revealed_order.push(s.piles.hand[0].handle()));
    check!(outcome, |s: &mut super::super::model::State| s.outcome = Some(Outcome::Victory));
    check!(fault, |s: &mut super::super::model::State| s.fault = Some(FaultRecord { kind: FaultKind::Invariant, message: "first".into(), resolved_effects: 0 }));
    check!(hp_before_healing, |s: &mut super::super::model::State| s.hp_before_healing = Some(40));
    check!(hp_after_cleanup, |s: &mut super::super::model::State| s.hp_after_cleanup = Some(46));
}

#[test]
fn busy_effect_targets_and_power_owners_are_validated() {
    let mut busy = Combat::new(&base_scenario()).unwrap().snapshot();
    busy.state.phase = Phase::RoundEnd;
    for kind in [EffectKind::Damage(DamagePayload::Fixed(6)), EffectKind::GainBlock(5),
        EffectKind::ApplyPower { id: PowerId::Vulnerable, amount: 2 },
        EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 },
        EffectKind::RemovePower(PowerId::Vulnerable)] {
        let mut bad = busy.clone(); bad.state.pending.push_back(effect(kind));
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))), "targetless {:?}", bad.state.pending);
    }
    for kind in [EffectKind::RemovePower(PowerId::Ritual),
        EffectKind::ApplyPower { id: PowerId::Ritual, amount: 3 }] {
        let mut bad = busy.clone();
        bad.state.pending.push_back(Effect { source: Source::None, target: Some(Actor::Player), kind });
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))));
    }
    // Every typed payload is retained in order at a safe point, not normalized away.
    busy.state.phase = Phase::PlayerAction;
    busy.state.in_play = Some(busy.state.piles.hand.remove(0));
    let enemy = Actor::Enemy(busy.state.enemies[0].instance_id.clone());
    let card = Source::Card(busy.state.in_play.as_ref().unwrap().handle());
    let mut work = vec![
        Effect { source: card.clone(), target: Some(enemy.clone()), kind: EffectKind::Damage(DamagePayload::Fixed(1515)) },
        Effect { source: Source::Actor(enemy.clone()), target: Some(Actor::Player), kind: EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Attack)) },
        Effect { source: Source::None, target: Some(Actor::Player), kind: EffectKind::GainBlock(8) },
        Effect { source: Source::Actor(enemy.clone()), target: Some(enemy.clone()), kind: EffectKind::ApplyPower { id: PowerId::Ritual, amount: 3 } },
        Effect { source: Source::None, target: Some(Actor::Player), kind: EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 } },
        Effect { source: Source::None, target: Some(enemy), kind: EffectKind::RemovePower(PowerId::Vulnerable) },
    ];
    work.push(Effect { source: card, target: None, kind: EffectKind::FinishCard });
    work.extend([EffectKind::Decision, EffectKind::Draw(10),
        EffectKind::Recycle, EffectKind::DiscardHand, EffectKind::EndPlayerTurn,
        EffectKind::MonsterStart, EffectKind::EnemyMove, EffectKind::RollMove, EffectKind::EndRound]
        .into_iter().map(effect));
    busy.state.pending.extend(work);
    assert_eq!(Combat::restore(&busy).unwrap().snapshot(), busy);
    for phase in [Phase::EnemyTurn, Phase::RoundEnd, Phase::PlayerStart] {
        let mut safe = busy.clone(); safe.state.phase = phase;
        safe.state.piles.discard.push(safe.state.in_play.take().unwrap());
        safe.state.pending.retain(|w| w.kind != EffectKind::FinishCard);
        assert_eq!(Combat::restore(&safe).unwrap().snapshot(), safe);
    }
}

#[test]
fn lethal_busy_snapshots_need_matching_terminal_work() {
    let mut busy = Combat::new(&base_scenario()).unwrap().snapshot();
    busy.state.phase = Phase::EnemyTurn; busy.state.player.hp = 0;
    busy.state.pending.push_back(effect(EffectKind::Decision));
    assert!(Combat::restore(&busy).is_err());
    busy.state.pending[0] = effect(EffectKind::Terminal(Outcome::Victory));
    assert!(Combat::restore(&busy).is_err());
    busy.state.pending[0] = effect(EffectKind::Terminal(Outcome::Death));
    assert!(Combat::restore(&busy).is_ok());
    busy.state.enemies[0].hp = 0;
    assert!(Combat::restore(&busy).is_err());
}

#[test]
fn runtime_numeric_edges_and_stream_sentinel() {
    let good = Combat::new(&base_scenario()).unwrap().snapshot();
    let changes: Vec<Box<dyn Fn(&mut super::super::model::State)>> = vec![
        Box::new(|s| s.player.max_hp = 1000), Box::new(|s| s.player.hp = 51),
        Box::new(|s| s.player.hp = -1), Box::new(|s| s.player.block = -1),
        Box::new(|s| s.player.energy = -1), Box::new(|s| s.player.energy = 4),
        Box::new(|s| s.player.max_energy = 0), Box::new(|s| s.player.max_energy = 1000),
        Box::new(|s| s.player.draw_per_turn = 0), Box::new(|s| s.player.draw_per_turn = 11),
        Box::new(|s| s.enemies[0].hp = -1), Box::new(|s| s.enemies[0].hp = 49),
        Box::new(|s| s.enemies[0].max_hp = 55), Box::new(|s| s.enemies[0].block = -1),
        Box::new(|s| s.enemies[0].block = 1000), Box::new(|s| s.enemies[0].damage_entries.clear()),
        Box::new(|s| s.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().output = -1),
        Box::new(|s| s.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().output = 7),
        Box::new(|s| s.enemies[0].powers.push(power(PowerId::Ritual, 3, true, true))),
        Box::new(|s| s.enemies[0].move_history.extend([ActorIntentId::Legacy(MoveId::Incantation)])),
        Box::new(|s| s.player.powers.push(power(PowerId::Vulnerable, 0, false, false))),
        Box::new(|s| s.resolved_effects = 100001),
    ];
    for (i, change) in changes.into_iter().enumerate() {
        let mut bad = good.clone(); change(&mut bad.state);
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))), "case {i}");
    }
    let mut busy = good;
    busy.state.phase = Phase::PlayerStart; busy.state.turn = i32::MAX as u32;
    busy.state.player.max_hp = 999; busy.state.player.hp = 999; busy.state.player.block = 999;
    busy.state.player.max_energy = 999; busy.state.player.energy = 999;
    busy.state.player.powers.push(power(PowerId::Vulnerable, i32::MAX, false, true));
    busy.state.enemies[0].powers.push(power(PowerId::Strength, 999, false, false));
    busy.state.enemies[0].powers.push(power(PowerId::Ritual, 3, false, false));
    busy.state.enemies[0].damage_entries.get_mut(&MoveId::Attack).unwrap().output = 1507;
    busy.state.pending.push_back(effect(EffectKind::Draw(0)));
    busy.state.rng.shuffle = StreamState::new(busy.state.rng.shuffle.id().clone(), [0; 32], 1_u128 << 64).unwrap();
    assert_eq!(Combat::restore(&busy).unwrap().snapshot(), busy);
    for kind in [EffectKind::GainBlock(-1), EffectKind::GainBlock(1008),
        EffectKind::Damage(DamagePayload::Fixed(-1)), EffectKind::Damage(DamagePayload::Fixed(1516)),
        EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Chomp)),
        EffectKind::ApplyPower { id: PowerId::Strength, amount: 1003 },
        EffectKind::ReducePower { id: PowerId::Vulnerable, amount: -1 }] {
        let mut bad = busy.clone();
        bad.state.pending.push_back(Effect { source: Source::None, target: Some(Actor::Player), kind });
        assert!(Combat::restore(&bad).is_err());
    }
}

#[test]
fn entry_enemy_range_endpoints_and_empty_hand() {
    for (kind, first, lower, upper) in [(EnemyKind::Cultist, MoveId::Incantation, 48, 54),
        (EnemyKind::JawWorm, MoveId::Chomp, 40, 44)] {
        for hp in [lower - 1, lower, upper, upper + 1] {
            let mut scenario = base_scenario();
            scenario.encounter_id = kind;
            let e = &mut scenario.enemies[0];
            e.enemy_id = kind; e.hp = hp; e.max_hp = hp;
            e.intent_id = first; e.move_history = vec![first];
            scenario.piles.draw.append(&mut scenario.piles.hand);
            assert_eq!(Combat::new(&scenario).is_ok(), hp == lower || hp == upper);
        }
    }
}

#[test]
fn busy_continuations_cannot_hide_incomplete_ownership_or_intent() {
    let mut busy = Combat::new(&base_scenario()).unwrap().snapshot();
    busy.state.phase = Phase::PlayerAction;
    busy.state.pending.push_back(effect(EffectKind::Decision));
    for which in 0..5 {
        let mut bad = busy.clone();
        match which {
            0 => bad.state.in_play = Some(bad.state.piles.hand.remove(0)),
            1 => bad.state.pending.push_back(effect(EffectKind::FinishCard)),
            2 => bad.state.pending.push_back(effect(EffectKind::Terminal(Outcome::Victory))),
            3 => bad.state.enemies[0].intent_damage = Some(6),
            _ => {
                bad.state.enemies[0].intent_id = ActorIntentId::Legacy(MoveId::Attack);
                bad.state.enemies[0].selected_move = ActorIntentId::Legacy(MoveId::Attack);
                bad.state.enemies[0].move_history.push(ActorIntentId::Legacy(MoveId::Attack));
                bad.state.enemies[0].intent_damage = None;
            }
        }
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(_))), "case {which}");
    }
    busy.state.in_play = Some(busy.state.piles.hand.remove(0));
    busy.state.pending.push_front(Effect {
        source: Source::Card(busy.state.in_play.as_ref().unwrap().handle()),
        target: None, kind: EffectKind::FinishCard });
    assert!(Combat::restore(&busy).is_ok());
}

fn finish_checkpoint(source: Source) -> super::super::snapshot::Snapshot {
    let mut busy = Combat::new(&base_scenario()).unwrap().snapshot();
    busy.state.phase = Phase::PlayerAction;
    busy.state.in_play = Some(busy.state.piles.hand.remove(0));
    busy.state.pending.push_back(Effect { source, target: None, kind: EffectKind::FinishCard });
    busy.state.pending.push_back(effect(EffectKind::Decision));
    busy
}

#[test]
fn finish_card_restore_rejects_absent_source() {
    let bad = finish_checkpoint(Source::None);
    assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(InputError::Invariant(_)))));
}

#[test]
fn finish_card_restore_rejects_actor_source() {
    for actor in [Actor::Player, Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())] {
        let bad = finish_checkpoint(Source::Actor(actor));
        assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(InputError::Invariant(_)))));
    }
}

#[test]
fn finish_card_restore_rejects_other_issued_card_source() {
    let bad = finish_checkpoint(Source::Card(CardHandle::parse("2").unwrap()));
    assert!(bad.state.card_ids_issued.contains(&CardHandle::parse("2").unwrap()));
    assert!(matches!(Combat::restore(&bad), Err(RestoreError::Invalid(InputError::Invariant(_)))));
}

#[test]
fn matching_finish_card_resumes_all_card_rows_to_real_step_outcomes() {
    use super::super::engine;
    for (id, upgraded, cost, damage, block, vulnerable, dispatches) in [
        ("1", false, 1, 6, 0, 0, 3), ("1", true, 1, 9, 0, 0, 3),
        ("2", false, 1, 0, 5, 0, 3), ("2", true, 1, 0, 8, 0, 3),
        ("3", false, 2, 8, 0, 2, 4), ("3", true, 2, 10, 0, 3, 4),
    ] {
        let mut scenario = base_scenario();
        let handle = CardHandle::parse(id).unwrap();
        scenario.piles.hand.iter_mut().find(|c| c.instance_id == handle).unwrap().upgraded = upgraded;
        let mut expected = Combat::new(&scenario).unwrap();
        let target_id = if block > 0 { None } else { Some(scenario.enemies[0].instance_id.clone()) };
        assert_eq!(expected.step(&Action::PlayCard { card_instance_id: handle.clone(), target_id }).unwrap().status,
            TransitionStatus::Decision);
        // Independently authored card schedule, then real dispatches and restores.
        let mut live = Combat::new(&scenario).unwrap();
        live.state.phase = Phase::PlayerAction;
        let index = live.state.piles.hand.iter().position(|c| c.handle() == handle).unwrap();
        live.state.in_play = Some(live.state.piles.hand.remove(index));
        live.state.player.energy -= cost;
        let source = Source::Card(handle.clone());
        let enemy = Actor::Enemy(live.state.enemies[0].instance_id.clone());
        live.state.pending.push_back(Effect { source: source.clone(),
            target: Some(if block > 0 { Actor::Player } else { enemy.clone() }),
            kind: if block > 0 { EffectKind::GainBlock(block) } else { EffectKind::Damage(DamagePayload::Fixed(damage)) } });
        if vulnerable > 0 {
            live.state.pending.push_back(Effect { source: source.clone(), target: Some(enemy),
                kind: EffectKind::ApplyPower { id: PowerId::Vulnerable, amount: vulnerable } });
        }
        live.state.pending.push_back(Effect { source, target: None, kind: EffectKind::FinishCard });
        live.state.pending.push_back(effect(EffectKind::Decision));
        while !live.state.pending.is_empty() {
            let captured = live.snapshot();
            let baseline = captured.clone();
            let mut restored = Combat::restore(&captured).unwrap();
            assert_eq!(engine::resolve(&mut restored.state).status, TransitionStatus::Decision);
            assert_eq!(restored.snapshot(), expected.snapshot());
            assert_eq!(captured, baseline);
            engine::dispatch_one(&mut live.state);
        }
        assert_eq!(live.snapshot(), expected.snapshot());
        assert_eq!(live.state.enemies[0].hp, 48 - damage);
        assert_eq!(live.state.player.block, block);
        assert_eq!(live.state.player.energy, 3 - cost);
        assert_eq!(live.state.resolved_effects, dispatches);
        assert!(live.state.in_play.is_none());
        assert_eq!(live.state.piles.discard[0].handle(), handle);
        assert_eq!(live.state.fault, None);
    }
}
