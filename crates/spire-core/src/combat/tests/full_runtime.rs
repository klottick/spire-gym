use super::super::*;
use super::super::rng::Domain;
use super::super::rng::StreamState;
use super::super::model::OwnedCard;
use super::support::base_scenario;

// Compatibility sentinel: downstream legacy consumers intentionally exhaust
// the original three-variant public enum.
fn legacy_card_kind_index(kind: CardKind) -> u8 {
    match kind { CardKind::Strike => 0, CardKind::Defend => 1, CardKind::Bash => 2 }
}

fn empty_piles() -> Piles { Piles { hand: vec![], draw: vec![], discard: vec![], exhaust: vec![] } }
fn full_cards(scenario: &mut Scenario) -> Vec<LiveCard> {
    let cards = live_deck(scenario);
    scenario.piles = empty_piles();
    cards
}

#[test]
fn two_enemies_are_not_one_terminal() {
    assert_eq!(legacy_card_kind_index(CardKind::Bash), 2);
    let mut scenario = base_scenario();
    // Full entry starts with an owned shuffled draw pile rather than the legacy
    // supplied-ready hand. Keep deterministic fixture identities and values.
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.player.hp = 40;
    let mut second = scenario.enemies[0].clone();
    second.instance_id = EnemyHandle::parse("enemy-2").unwrap();
    second.enemy_id = EnemyKind::JawWorm;
    second.hp = 6;
    second.max_hp = 44;
    second.intent_id = MoveId::Chomp;
    second.move_history = vec![MoveId::Chomp];
    scenario.enemies[0].hp = 8;
    scenario.enemies.push(second);
    // Legacy Scenario is intentionally still a supplied-ready single-enemy API.
    assert!(matches!(Combat::new(&scenario), Err(InputError::Invariant(_))));

    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let cards = full_cards(&mut scenario);
    assert!(super::super::engine::validate_scenario_mode(&scenario, true).is_ok(), "{:?}", super::super::engine::validate_scenario_mode(&scenario, true));
    let mut combat = Combat::enter(CombatEntry { setup: full_setup(&scenario), opening_draw: 3, cards }).unwrap();
    let ready = combat.public_full().unwrap();
    assert_eq!(ready.hand.len(), 3);
    assert_eq!(ready.draw_count, 0);
    assert_eq!(ready.enemies.len(), 2);
    assert_eq!(ready.hand.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        vec!["defend", "bash", "strike"]);

    let bash = CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) };
    combat.step_full(&bash).unwrap();
    assert_eq!(combat.state.player.energy, 1, "Bash pays exactly two energy");
    assert_eq!(combat.public_full().unwrap().discard[0].instance, CardInstanceId(3));
    let after_first = combat.public_full().unwrap();
    assert_eq!(after_first.enemies[0].hp, 0);
    assert_eq!(after_first.enemies[1].hp, 6);
    assert_eq!(after_first.outcome, None, "one survivor prevents victory");
    let after_first_state = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, after_first_state, "dead target is rejected atomically");

    let hp_before = after_first.player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.fault_record().is_none(), "{:?}", combat.fault_record());
    let after_other_acts = combat.public_full().unwrap();
    assert_eq!(after_other_acts.player.hp, hp_before - 11);
    assert_eq!(after_other_acts.enemies[1].hp, 6);
    assert_eq!(after_other_acts.hand.len(), 3, "discard recycle and draw preserve all three owners");
    assert!(after_other_acts.discard.is_empty());
    let mut returned: Vec<_> = after_other_acts.hand.iter().map(|card| card.instance.0).collect();
    returned.sort_unstable();
    assert_eq!(returned, vec![1, 2, 3]);

    // The surviving enemy is target 2; killing it alone completes combat.
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(2)) }).unwrap();
    assert_eq!(combat.state.player.energy, 2, "Strike pays exactly one energy after turn reset");
    assert_eq!(combat.public_full().unwrap().outcome, Some(Outcome::Victory));
}

#[test]
fn full_card_damage_and_power_follow_selected_actor() {
    let mut scenario = base_scenario();
    scenario.piles.draw.append(&mut scenario.piles.hand);
    let second = EnemySetup { instance_id: EnemyHandle::parse("enemy-2").unwrap(),
        enemy_id: EnemyKind::JawWorm, hp: 44, max_hp: 44, block: 0, powers: vec![],
        intent_id: MoveId::Chomp, move_history: vec![MoveId::Chomp] };
    scenario.enemies.push(second);
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let cards = full_cards(&mut scenario);
    let mut combat = Combat::enter(CombatEntry { setup: full_setup(&scenario), opening_draw: 3, cards }).unwrap();
    combat.state.enemies[1].powers.push(PowerEntry { id: PowerId::Vulnerable,
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    combat.state.enemies[1].block = 2;
    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(2)) }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 48);
    assert!(view.enemies[0].powers.is_empty());
    assert_eq!(view.enemies[1].hp, 34); // 8 × 3/2 damage, then 2 target Block absorbed.
    assert_eq!(view.enemies[1].block, 0);
    assert_eq!(view.enemies[1].powers, vec![FullPowerView { id: FullPowerId("vulnerable"), amount: 3 }]);
}

#[test]
fn guard_is_fault_not_loss() {
    let mut combat = Combat::new(&base_scenario()).unwrap();
    combat.state.resolved_effects = 99_999;
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::Decision });
    super::super::engine::dispatch_one(&mut combat.state);
    assert!(combat.fault_record().is_some());
    assert_eq!(combat.state.outcome, None);
    assert_eq!(combat.state.player.hp, 40);
}

#[test]
fn legacy_entry_does_not_draw() {
    let mut scenario = base_scenario();
    scenario.piles.draw.push(CardInstance { instance_id: CardHandle::parse("4").unwrap(),
        card_id: CardKind::Strike, upgraded: false });
    let combat = Combat::new(&scenario).unwrap();
    assert_eq!(combat.state.piles.hand.len(), 3);
    assert_eq!(combat.state.piles.draw.len(), 1);
    assert!(combat.state.hook_events.is_empty(), "supplied-ready legacy entry has no combat-start hook");
}

#[test]
fn full_entry_is_seeded_and_draws_after_shuffle() {
    let mut scenario = base_scenario();
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let cards = full_cards(&mut scenario);
    let entry = CombatEntry { setup: full_setup(&scenario), cards, opening_draw: 2 };
    let a = Combat::enter(entry.clone()).unwrap();
    let b = Combat::enter(entry).unwrap();
    let av = a.public_full().unwrap();
    let bv = b.public_full().unwrap();
    assert_eq!(av.hand, bv.hand);
    assert_eq!(av.hand.len(), 2);
    assert_eq!(av.draw_count, 1);
    assert_eq!(av.hand.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), vec!["defend", "bash"]);
}

#[test]
fn full_public_projection_is_equivalent_across_hidden_order_and_rng_state() {
    let a = entered_combat(3, 3, |_| {});
    let mut b = a.clone();
    b.state.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["different-hidden-stream"]);
    b.state.piles.draw.reverse();
    assert_eq!(a.public_full().unwrap(), b.public_full().unwrap());
}

#[test]
fn dead_or_unknown_full_target_is_rejected_without_mutation() {
    let mut scenario = base_scenario();
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let cards = full_cards(&mut scenario);
    let mut combat = Combat::enter(CombatEntry { setup: full_setup(&scenario), opening_draw: 3, cards }).unwrap();
    let before = combat.state.clone();
    let result = combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(99)) });
    assert_eq!(result, Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, before);
}

#[test]
fn payment_free_x_cost_and_temporary_cost_share_one_commit() {
    let mut free = entered_combat(3, 3, |card| card.local.free_play = true);
    free.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(free.state.player.energy, 3);
    assert!(!free.state.piles.discard.iter().find(|c| c.handle().as_str() == "3").unwrap().full().unwrap().local.free_play,
        "free-play permission is consumed exactly once");
    assert_eq!(free.state.piles.discard.iter().filter(|c| c.handle().as_str() == "3").count(), 1);

    let mut x = entered_combat(3, 3, |card| card.local.x_cost = true);
    x.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(x.state.player.energy, 0);
    assert_eq!(x.state.piles.discard.iter().filter(|c| c.handle().as_str() == "3").count(), 1);

    let mut changed = entered_combat(3, 3, |card| card.local.temporary_cost = 1);
    changed.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(changed.state.player.energy, 0);
}

#[test]
fn unpayable_card_changes_no_state_and_unknown_handler_faults() {
    let mut poor = entered_combat(1, 1, |_| {});
    let before = poor.state.clone();
    assert_eq!(poor.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(poor.state, before);

    let mut unsupported = entered_combat(3, 3, |card| card.id = crate::content::CardId::Unknown("unwired_runtime_fixture"));
    let before_energy = unsupported.state.player.energy;
    let before_invalid = unsupported.state.clone();
    assert_eq!(unsupported.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(99)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(unsupported.state, before_invalid, "invalid target is rejected before fault capture");
    let action = CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) };
    let result = unsupported.step_full(&action);
    let crate::run::RunError::Faulted(first_packet) = result.clone().unwrap_err() else {
        panic!("combat faults must retain the native fault packet");
    };
    let crate::run::RunFaultKind::Combat(payload) = &first_packet.kind else {
        panic!("combat packet must not be flattened to a string boundary");
    };
    assert_eq!(payload.record.kind, FaultKind::Unsupported);
    assert_eq!(payload.record.message, "unhandled full-run card unwired_runtime_fixture");
    assert_eq!(payload.record.resolved_effects, 0);
    assert_eq!(payload.current, None, "handler fault occurs before effect construction");
    assert_eq!(payload.attempted_action, Some(action));
    assert!(payload.pending.is_empty());
    assert_eq!(unsupported.state.player.energy, before_energy);
    assert_eq!(unsupported.state.in_play, None);
    let first_fault = unsupported.fault_record().unwrap();
    assert!(matches!(first_fault.kind, FaultKind::Unsupported));
    let interrupted = unsupported.state.current.clone();
    assert_eq!(unsupported.step_full(&CombatAction::EndTurn), result);
    assert_eq!(unsupported.public_full(), Err(crate::run::RunError::Faulted(first_packet.clone())));
    assert_eq!(unsupported.exit(), Err(crate::run::RunError::Faulted(first_packet)));
    assert_eq!(unsupported.fault_record(), Some(first_fault));
    assert_eq!(unsupported.state.current, interrupted, "faulted work is not retried or replaced");
}

#[test]
fn unsupported_handler_preflight_rejects_selection_and_terminal_without_mutation() {
    let unsupported_id = crate::content::CardId::Unknown("unwired_runtime_fixture");
    let mut selecting = entered_combat(3, 3, |card| card.id = unsupported_id);
    let unsupported_instance = selecting.state.piles.hand.iter().find(|c|
        c.full().is_some_and(|live| live.id == unsupported_id)).unwrap().handle();
    let unsupported_instance = unsupported_instance.as_str().parse::<u64>().unwrap();
    let candidate = selecting.state.piles.hand[0].handle();
    selecting.state.pending_selection = Some(super::super::model::PendingSelection {
        operation:SelectionOperation::Move(SelectionDisposition::Discard),
        parent: super::super::selection::SelectionParent::Card(CardHandle::parse("99").unwrap()),
        source_zone:super::super::selection::SelectionSourceZone::Hand,candidates: vec![candidate], selected: vec![],
        minimum:1,maximum:1, disposition: super::super::selection::SelectionDisposition::Discard,
        empty: super::super::selection::EmptySelection::Continue,
    });
    selecting.state.phase = Phase::Selection;
    let before_selection = selecting.state.clone();
    assert_eq!(selecting.step_full(&CombatAction::PlayCard { instance: unsupported_instance, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(selecting.state, before_selection, "wrong-phase request preserves full state including RNG and budget");

    let unsupported_id = crate::content::CardId::Unknown("unwired_runtime_fixture");
    let mut terminal = entered_combat(3, 3, |card| card.id = unsupported_id);
    let unsupported_instance = terminal.state.piles.hand.iter().find(|c|
        c.full().is_some_and(|live| live.id == unsupported_id)).unwrap().handle().as_str().parse().unwrap();
    terminal.state.phase = Phase::Terminal;
    terminal.state.outcome = Some(Outcome::Victory);
    let before_terminal = terminal.state.clone();
    assert_eq!(terminal.step_full(&CombatAction::PlayCard { instance: unsupported_instance, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(terminal.state, before_terminal, "terminal request preserves full state including RNG and budget");
}

#[test]
fn full_fault_packet_retains_current_and_queued_effects_across_all_boundaries() {
    let mut combat = entered_combat(3, 3, |_| {});
    let source = Source::Card(CardHandle::parse("3").unwrap());
    let target = Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap());
    let current = Effect { source: source.clone(), target: Some(target.clone()),
        kind: EffectKind::Damage(DamagePayload::Fixed(4)) };
    let queued = Effect { source: source.clone(), target: None, kind: EffectKind::FinishCard };
    combat.state.pending.extend([current.clone(), queued.clone()]);
    combat.state.resolved_effects = 99_999;
    super::super::engine::dispatch_one(&mut combat.state);
    let first = combat.public_full().unwrap_err();
    let crate::run::RunError::Faulted(packet) = &first else { panic!("fault packet expected") };
    let crate::run::RunFaultKind::Combat(native) = &packet.kind else { panic!("native combat cause expected") };
    assert_eq!(native.record.kind, FaultKind::EffectGuard);
    assert_eq!(native.record.resolved_effects, 100_000);
    assert_eq!(native.current, Some(current));
    assert_eq!(native.pending.iter().cloned().collect::<Vec<_>>(), vec![queued]);
    assert_eq!(native.attempted_action, None);
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Err(first.clone()));
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: 3 }), Err(first.clone()));
    assert_eq!(combat.public_full(), Err(first.clone()));
    assert_eq!(combat.exit(), Err(first));
}

#[test]
fn queued_turn_ended_hook_applies_real_ritual_work_through_resolver() {
    let mut combat = entered_combat(3, 3, |_| {});
    let enemy = combat.state.enemies[0].instance_id.clone();
    combat.state.enemies[0].powers.push(PowerEntry { id: PowerId::Ritual,
        state: PowerState { amount: 3, skip_first: false, monster_source_grace: false } });
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Hook(Hook::TurnEnded) });
    let transition = super::super::engine::resolve(&mut combat.state);
    assert_eq!(transition.status, TransitionStatus::Decision);
    assert_eq!(combat.state.enemies[0].powers.iter().find(|p| p.id == PowerId::Strength)
        .map(|p| p.state.amount), Some(3), "hook must enqueue ordinary power work");
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, 9,
        "the same resolver must refresh the next attack from hook-applied Strength");
    assert_eq!(combat.state.hook_events.last().map(|event| event.hook), Some(Hook::TurnEnded));
    assert_eq!(combat.state.enemies[0].instance_id, enemy);
}

#[test]
fn seeded_full_cultist_ritual_hook_skips_once_then_strengthens_next_attack() {
    let mut combat = entered_combat(3, 3, |_| {});
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    let ritual = combat.state.enemies[0].full_powers.iter().find(|p| p.id == FullPowerId("ritual")).unwrap();
    assert_eq!((ritual.state.amount, ritual.state.skip_first), (3, false));
    assert!(!combat.state.enemies[0].full_powers.iter().any(|p| p.id == FullPowerId("strength")),
        "TurnEnded hook honors Ritual's first-turn skip: {:?}", combat.state.enemies[0].full_powers);
    let ended_before = combat.state.hook_events.iter().filter(|event| event.hook == Hook::TurnEnded).count();
    assert_eq!(ended_before, 1);

    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(combat.state.player.hp, 34, "second turn executes published six-damage attack before hook growth");
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|p| p.id == FullPowerId("strength")).unwrap().state.amount, 3);
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, 9);
    assert_eq!(combat.state.enemies[0].intent_damage, Some(9));
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::TurnEnded).count(), 2);
    assert!(combat.state.pending.is_empty(), "hook power work precedes next-player decision construction");
    assert_eq!(combat.state.phase, Phase::Decision);
}

#[test]
fn full_power_damage_refresh_agrees_with_entry_validation() {
    let cases = [
        (vec![], vec![FullPowerEntry { id: FullPowerId("strength"),
            state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } }], 8),
        (vec![FullPowerEntry { id: FullPowerId("vulnerable"),
            state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } }], vec![], 9),
        (vec![FullPowerEntry { id: FullPowerId("vulnerable"),
            state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } }],
            vec![FullPowerEntry { id: FullPowerId("strength"),
                state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } }], 12),
    ];
    for (player_powers, enemy_powers, expected) in cases {
        let mut scenario = base_scenario();
        scenario.piles.draw.append(&mut scenario.piles.hand);
        scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
        scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
        let cards = full_cards(&mut scenario);
        let mut setup = full_setup(&scenario);
        setup.player_powers = player_powers;
        setup.enemies[0].intent = FullIntentId::Attack;
        setup.enemies[0].powers = enemy_powers;
        let combat = Combat::enter(CombatEntry { setup, opening_draw: 3, cards })
            .unwrap_or_else(|error| panic!("entry rejected refreshed full-power damage (expected {expected}): {error:?}"));
        let before = combat.state.clone();
        assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, expected);
        assert_eq!(combat.public_full().unwrap().enemies[0].intent_damage, Some(expected));
        assert_eq!(combat.state, before, "public projection is read-only");
    }
}

#[test]
fn posthook_ready_state_validates_and_public_read_is_read_only() {
    let mut combat = entered_combat(3, 3, |_| {});
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, 9);
    assert_eq!(super::super::engine::validate_runtime(&combat.state), Ok(()));
    let before = combat.state.clone();
    assert_eq!(combat.public_full().unwrap().enemies[0].intent_damage, Some(9));
    assert_eq!(combat.state, before, "public read does not repair or mutate damage entries");
}

#[test]
fn full_damage_invariant_still_rejects_corrupted_refreshed_output() {
    let mut combat = entered_combat(3, 3, |_| {});
    combat.state.enemies[0].full_damage_entries.get_mut(&FullIntentId::Attack).unwrap().output += 1;
    assert!(super::super::engine::validate_runtime(&combat.state).is_err(),
        "damage refresh invariant must reject stale full damage output");
}

#[test]
fn full_native_actor_identity_intent_powers_and_ai_owners_are_typed() {
    let scenario = base_scenario();
    let cards = vec![permanent_live_card(crate::content::CardId::Strike, CardInstanceId(701), 0)];
    let enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "native-a"]);
    let setup = FullCombatSetup {
        enemy_damage_bases: Default::default(),
        character: crate::content::Character::Ironclad,
        card_rng: full_stream(Domain::RunCombatCardRandom, vec!["run-combat-1"]),
        player: scenario.player.clone(),
        player_powers: vec![FullPowerEntry { id: FullPowerId("intangible"), state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } }],
        shuffle_rng: full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]),
        enemies: vec![FullEnemySetup { initial_memory:None,
            instance_id: EnemyHandle::parse("native-enemy-a").unwrap(),
            enemy_id: crate::content::EnemyId::Unknown("spire_guardian"), ascension: 0, profile: super::super::enemies::EnemyProfile::Standard,
            hp: 30, max_hp: 30, block: 0,
            intent: FullIntentId::Unknown("unwired_charge"), powers: vec![], ai_rng: enemy_ai,
        }],
    };
    let mut combat = Combat::enter(CombatEntry { setup, opening_draw: 1, cards }).unwrap();
    let public = combat.public_full().unwrap();
    assert_eq!(public.enemies[0].enemy_id, crate::content::EnemyId::Unknown("spire_guardian"));
    assert_eq!(public.enemies[0].intent, FullIntentId::Unknown("unwired_charge"));
    assert_eq!(public.player.powers, vec![FullPowerView { id: FullPowerId("intangible"), amount: 2 }]);

    let before = combat.state.clone();
    assert!(matches!(combat.step_full(&CombatAction::EndTurn), Err(crate::run::RunError::Faulted(_))));
    assert_eq!(combat.fault_record().unwrap().kind, FaultKind::Unsupported);
    assert_eq!(combat.state.enemies[0].enemy_id, ActorEnemyId::Full(crate::content::EnemyId::Unknown("spire_guardian")));
    assert_eq!(combat.state.enemies[0].intent_id, ActorIntentId::Full(FullIntentId::Unknown("unwired_charge")));
    assert_ne!(combat.state, before, "accepted unsupported native move retains fault provenance");
    assert_eq!(combat.state.player_full_powers[0].id, FullPowerId("intangible"));
}

#[test]
fn full_enemy_ai_cursors_belong_to_their_actors_not_turn_interleaving() {
    let mut scenario = base_scenario();
    scenario.encounter_id = EnemyKind::JawWorm;
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.enemies[0].enemy_id = EnemyKind::JawWorm;
    scenario.enemies[0].hp = 40;
    scenario.enemies[0].max_hp = 40;
    scenario.enemies[0].intent_id = MoveId::Chomp;
    scenario.enemies[0].move_history = vec![MoveId::Chomp];
    let mut second = scenario.enemies[0].clone();
    second.instance_id = EnemyHandle::parse("enemy-2").unwrap();
    scenario.enemies.push(second);
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let cards = full_cards(&mut scenario);
    let mut first_setup = full_setup(&scenario);
    let second_setup = first_setup.clone();
    let first_id = first_setup.enemies[0].ai_rng.id().clone();
    first_setup.enemies[0].ai_rng = StreamState::new(first_id, [99; 32], 0).unwrap();
    let mut left = Combat::enter(CombatEntry { setup: first_setup, opening_draw: 3, cards: cards.clone() }).unwrap();
    let mut right = Combat::enter(CombatEntry { setup: second_setup, opening_draw: 3, cards }).unwrap();
    assert_eq!(left.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(right.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(left.state.enemies[1].selected_move, right.state.enemies[1].selected_move);
    assert_eq!(left.state.enemies[1].full_ai_rng, right.state.enemies[1].full_ai_rng);
    assert_eq!(left.state.enemies[1].full_ai_rng.as_ref().unwrap().word_index(), 2,
        "the second Jaw Worm consumes its own bucket and fallback coin, not a shared cursor");
    assert_eq!(left.state.enemies[0].full_ai_rng.as_ref().unwrap().word_index(), 1,
        "the first actor must own its cursor in the same typed per-actor slot");
    assert_eq!(left.state.rng.enemy_instance_id, left.state.enemies[0].instance_id,
        "legacy-shaped RNG metadata is retained but is not the full actor cursor owner");
}

#[test]
fn full_entry_rejects_duplicate_or_cross_combat_enemy_ai_stream_owners() {
    let mut scenario = base_scenario();
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.enemies[0].enemy_id = EnemyKind::JawWorm;
    scenario.enemies[0].hp = 40;
    scenario.enemies[0].max_hp = 40;
    scenario.enemies[0].intent_id = MoveId::Chomp;
    scenario.enemies[0].move_history = vec![MoveId::Chomp];
    let mut second = scenario.enemies[0].clone();
    second.instance_id = EnemyHandle::parse("enemy-2").unwrap();
    scenario.enemies.push(second);
    let cards = full_cards(&mut scenario);
    let mut setup = full_setup(&scenario);
    setup.enemies[1].ai_rng = setup.enemies[0].ai_rng.clone();
    assert_eq!(Combat::enter(CombatEntry { setup: setup.clone(), opening_draw: 2, cards: cards.clone() }),
        Err(crate::run::RunError::InvalidAction), "one stream cannot be owned by two actors");

    setup.enemies[1].ai_rng = full_stream(Domain::RunCombatEnemyAi, vec!["other-run", "enemy-scope-2"]);
    assert_eq!(Combat::enter(CombatEntry { setup, opening_draw: 2, cards }),
        Err(crate::run::RunError::InvalidAction), "actor AI stream must belong to this combat root");
}

#[test]
fn typed_full_power_operations_keep_nonlegacy_identity_in_shared_actor_storage() {
    let mut combat = entered_combat(3, 3, |_| {});
    let actor = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    let enqueue = |combat: &mut Combat, kind| {
        combat.state.pending.push_back(Effect { source: Source::Actor(actor.clone()),
            target: Some(actor.clone()), kind });
        assert_eq!(super::super::engine::resolve(&mut combat.state).status, TransitionStatus::Decision);
    };
    let ward = FullPowerId("nonlegacy_ward");
    enqueue(&mut combat, EffectKind::ApplyFullPower { id: ward, amount: 4 });
    assert_eq!(combat.state.enemies[0].full_powers[0].id, ward);
    assert_eq!(combat.public_full().unwrap().enemies[0].powers[0].id, ward);
    enqueue(&mut combat, EffectKind::ReduceFullPower { id: ward, amount: 1 });
    assert_eq!(combat.state.enemies[0].full_powers[0].state.amount, 3);
    enqueue(&mut combat, EffectKind::ReduceFullPower { id: ward, amount: 3 });
    assert!(combat.state.enemies[0].full_powers.is_empty(), "zero reduction removes through queued native work");
    assert!(!combat.state.enemies[0].powers.iter().any(|p| p.id == PowerId::Strength),
        "a new key is not renamed into a legacy power");
}

#[test]
fn public_full_hook_boundary_enqueues_typed_source_target_and_resolves_real_work() {
    let mut combat = entered_combat(3, 3, |_| {});
    combat.state.enemies[0].full_powers.push(FullPowerEntry { id: FullPowerId("ritual"),
        state: PowerState { amount: 3, skip_first: false, monster_source_grace: false } });
    let event = HookEvent { hook: Hook::TurnEnded, source: Source::Actor(Actor::Player), target: Some(Actor::Player) };
    assert_eq!(combat.enqueue_hook(event.clone()), Ok(()));
    assert_eq!(combat.state.hook_events.last(), Some(&event));
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|p| p.id == FullPowerId("strength")).unwrap().state.amount, 3);
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, 9);
}

#[test]
fn upgrades_origin_and_exhaust_finish_follow_card_identity() {
    let mut combat = entered_combat(3, 3, |card| {
        if card.instance.0 == 3 { card.upgrades = 2; card.local.exhaust_on_finish = true; }
    });
    let live = combat.state.piles.hand.iter().find(|c| c.handle().as_str() == "3").unwrap().full().unwrap();
    assert_eq!(live.upgrades, 2);
    assert_eq!(live.origin, CombatCardRef::Permanent(CardInstanceId(3)));
    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 38, "upgrade count enables upgraded Bash damage");
    assert_eq!(combat.public_full().unwrap().enemies[0].powers, vec![FullPowerView { id: FullPowerId("vulnerable"), amount: 3 }]);
    assert!(!combat.state.piles.discard.iter().any(|c| c.handle().as_str() == "3"));
    assert_eq!(combat.state.piles.exhaust.iter().filter(|c| c.handle().as_str() == "3").count(), 1);
    assert_eq!(combat.state.piles.exhaust.iter().find(|c| c.handle().as_str() == "3").unwrap().full().unwrap().upgrades, 2);
}

#[test]
fn duplicate_full_card_instance_is_rejected_at_entry() {
    let mut scenario = base_scenario();
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let mut cards = full_cards(&mut scenario);
    cards[1].instance = cards[0].instance;
    assert_eq!(Combat::enter(CombatEntry { setup: full_setup(&scenario), opening_draw: 3, cards }),
        Err(crate::run::RunError::InvalidAction));
}

#[test]
fn full_entry_owns_generic_card_ids_without_legacy_pile_rows() {
    let mut scenario = base_scenario();
    scenario.piles = empty_piles();
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let mut unwired = permanent_live_card(crate::content::CardId::Unknown("unwired_runtime_fixture"), CardInstanceId(9001), 3);
    unwired.origin = CombatCardRef::CombatLocal(77);
    unwired.local.temporary_cost = -1;
    unwired.local.exhaust_on_finish = true;
    let cards = vec![unwired];
    let mut combat = Combat::enter(CombatEntry { setup: full_setup(&scenario), opening_draw: 1, cards }).unwrap();
    let ready = combat.public_full().unwrap();
    assert_eq!(ready.hand, vec![LiveCardView { instance: CardInstanceId(9001),
        id: crate::content::CardId::Unknown("unwired_runtime_fixture"),
        origin: CombatCardRef::CombatLocal(77), upgrades: 3, effective_cost:None,values:None }]);
    assert_eq!(combat.state.piles.draw.len(), 0);
    let owned = &combat.state.piles.hand[0];
    assert!(matches!(owned, OwnedCard::Full(_)), "full ID has no legacy pile row");
    let live = owned.full().unwrap();
    assert_eq!(live.id, crate::content::CardId::Unknown("unwired_runtime_fixture"));
    assert_eq!(live.upgrades, 3);
    assert_eq!(live.origin, CombatCardRef::CombatLocal(77));
    assert_eq!(live.local.temporary_cost, -1);
    assert!(combat.try_snapshot_v2().is_err());
    let full_snapshot = combat.snapshot();
    assert_eq!(full_snapshot.schema_version, 3);
    assert_eq!(Combat::restore(&full_snapshot), Err(RestoreError::Compatibility));
    let mut forged_legacy = full_snapshot.clone();
    forged_legacy.schema_version = 2;
    assert_eq!(Combat::restore(&forged_legacy), Err(RestoreError::Compatibility));
    let action = CombatAction::PlayCard { instance: 9001, target: Some(TargetId(1)) };
    let crate::run::RunError::Faulted(packet) = combat.step_full(&action).unwrap_err() else {
        panic!("accepted unsupported card must produce typed retained combat fault");
    };
    let crate::run::RunFaultKind::Combat(native) = packet.kind else { panic!("native combat fault expected") };
    assert_eq!(native.record.kind, FaultKind::Unsupported);
    assert_eq!(native.record.message, "unhandled full-run card unwired_runtime_fixture");
    assert_eq!(native.record.resolved_effects, 0);
    assert_eq!(native.attempted_action, Some(action));
    assert_eq!(native.current, None);
    assert!(native.pending.is_empty());
    assert_eq!(combat.state.player.energy, 3);
    assert!(matches!(combat.state.piles.hand[0], OwnedCard::Full(_)), "fault preserves exact owner");
}

pub(super) fn full_stream(domain: Domain, scope: Vec<&str>) -> super::super::rng::StreamState {
    super::super::rng::make_stream(&[8; 32], super::super::rng::Partition::Test, "full-entry",
        &super::super::rng::StreamId::new(domain, scope.into_iter().map(str::to_owned).collect()).unwrap()).unwrap()
}

fn live_deck(scenario: &Scenario) -> Vec<LiveCard> {
    scenario.piles.draw.iter().map(|card| {
        let instance = CardInstanceId(card.instance_id.as_str().parse().unwrap());
        let id = match card.card_id { CardKind::Strike => crate::content::CardId::Strike,
            CardKind::Defend => crate::content::CardId::Defend, CardKind::Bash => crate::content::CardId::Bash };
        permanent_live_card(id, instance, u32::from(card.upgraded))
    }).collect()
}

pub(super) fn entered_combat(max_energy: i32, energy: i32, alter_bash: impl FnOnce(&mut LiveCard)) -> Combat {
    let mut scenario = base_scenario();
    scenario.piles.draw.append(&mut scenario.piles.hand);
    scenario.player.max_energy = max_energy;
    scenario.player.energy = energy;
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["run-combat-1"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-1"]);
    let mut cards = full_cards(&mut scenario);
    alter_bash(cards.iter_mut().find(|c| c.instance.0 == 3).unwrap());
    Combat::enter(CombatEntry { setup: full_setup(&scenario), opening_draw: 3, cards }).unwrap()
}

fn full_setup(scenario: &Scenario) -> FullCombatSetup {
    FullCombatSetup { enemy_damage_bases: Default::default(),
        character: crate::content::Character::Ironclad,
        card_rng: full_stream(Domain::RunCombatCardRandom, vec!["run-combat-1"]),
        player: scenario.player.clone(), player_powers: scenario.player.powers.iter().map(|p|
        FullPowerEntry { id: FullPowerId(p.id.as_str()), state: p.state.clone() }).collect(),
        shuffle_rng: scenario.rng.shuffle.clone(), enemies: scenario.enemies.iter().enumerate().map(|(i,e)| {
            let ai_rng = if i == 0 { scenario.rng.enemy_ai.clone() } else {
                full_stream(Domain::RunCombatEnemyAi, vec!["run-combat-1", if i == 1 { "enemy-scope-2" } else { "enemy-scope-3" }]) };
            FullEnemySetup { initial_memory:None, instance_id: e.instance_id.clone(), ascension: 0, profile: super::super::enemies::EnemyProfile::Standard,
                enemy_id: match e.enemy_id { EnemyKind::Cultist => crate::content::EnemyId::Cultist,
                    EnemyKind::JawWorm => crate::content::EnemyId::JawWorm },
                hp: e.hp, max_hp: e.max_hp, block: e.block,
                intent: FullIntentId::from_name(e.intent_id.as_str()),
                powers: e.powers.iter().map(|p| FullPowerEntry { id: FullPowerId(p.id.as_str()), state: p.state.clone() }).collect(),
                ai_rng }
         }).collect() }
}


#[test]
fn starter_entry_fault_retains_committed_ring_draw_and_context_without_retry() {
    let mut scenario = base_scenario();
    scenario.player.hp = 40;
    scenario.rng.shuffle = full_stream(Domain::RunCombatShuffle, vec!["entry-fault"]);
    scenario.rng.enemy_ai = full_stream(Domain::RunCombatEnemyAi, vec!["entry-fault", "enemy-scope-1"]);
    let cards = (1..=10).map(|id| {
        let instance = CardInstanceId(if id == 10 { u64::MAX } else { id });
        LiveCard { instance, id: crate::content::CardId::Strike,
            origin: CombatCardRef::Permanent(instance), upgrades: 0,
            local: super::super::CombatCardData::default(),copy_family:instance.0,copy_provenance:super::super::card_state::CopyProvenance::Original }
    }).collect();
    let mut setup = full_setup(&scenario);
    setup.card_rng = full_stream(Domain::RunCombatCardRandom, vec!["entry-fault"]);
    let entry = CombatEntry { setup, opening_draw: 5, cards };
    let mut owner = Some(crate::items::CombatItemContext::new(
        vec![crate::content::RelicId::RingOfTheSnake, crate::content::RelicId::PureWater], vec![]));
    let failure = Combat::enter_with_item_context(entry, &mut owner).unwrap_err();
    assert!(owner.is_none());
    let CombatEntryFailure::Faulted { combat, fault } = failure else { panic!("expected entry fault") };
    let mut combat = *combat;
    assert_eq!(combat.state.piles.hand.len(), 7, "Ring draw committed before later item fault");
    assert_eq!(combat.state.item_context.as_ref().unwrap().relics(),
        &[crate::content::RelicId::RingOfTheSnake, crate::content::RelicId::PureWater]);
    assert_eq!(combat.state.fault.as_ref().unwrap(), match &fault.kind {
        crate::run::RunFaultKind::Combat(native) => &native.record,
        _ => panic!("expected retained combat RunFault"),
    });
    let before = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Err(crate::run::RunError::Faulted(fault)));
    assert_eq!(super::super::snapshot::compare(&before, &combat.snapshot()), None);
}

#[test]
fn native_fruit_effect_commits_max_hp_then_queues_ordinary_heal() {
    let mut combat = entered_combat(3, 3, |_| {});
    let (phase, observed_hp, observed_max_hp, live_targets) = combat.native_potion_use_state().unwrap();
    assert_eq!(phase, Phase::Decision);
    assert_eq!(observed_hp, combat.state.player.hp);
    assert_eq!(observed_max_hp, combat.state.player.max_hp);
    assert_eq!(live_targets, vec![EnemyHandle::parse("enemy-1").unwrap()]);
    let before_hp = combat.state.player.hp;
    let before_max = combat.state.player.max_hp;
    combat.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::IncreaseMaxHpThenHeal(5) }], None).unwrap();
    assert_eq!(combat.state.player.max_hp, before_max + 5);
    assert_eq!(combat.state.player.hp, (before_hp + 5).min(before_max + 5));
    assert!(combat.state.pending.is_empty());
    assert!(combat.state.current.is_none());
}
