//! Native Looter/Mugger actor regressions. Expected values are source-qualified
//! A0 literals; the tests exercise the shared effect queue and damage resolver.
use super::super::*;
use super::super::enemies::{self, EnemyMemory};
use super::super::enemies::act1::thieves::{self as thief_actor, StolenGoldReceipt, ThiefMemory};
use super::super::rng::{self, Domain, StreamState};
use crate::content::{CardId, Character, EnemyId};

fn stream(domain: Domain, scope: Vec<String>) -> StreamState {
    let id = rng::StreamId::new(domain, scope).unwrap();
    rng::make_stream(&[0x51; 32], rng::Partition::Test, "thief-actor-tests", &id).unwrap()
}

fn full_intent(id: EnemyId, suffix: &'static str) -> FullIntentId {
    FullIntentId::from_name(match id {
        EnemyId::Looter => match suffix {
            "mug" => "looter_mug", "lunge" => "looter_lunge",
            "smoke_bomb" => "looter_smoke_bomb", "escape" => "looter_escape",
            _ => unreachable!("known Looter suffix"),
        },
        EnemyId::Mugger => match suffix {
            "mug" => "mugger_mug", "lunge" => "mugger_lunge",
            "smoke_bomb" => "mugger_smoke_bomb", "escape" => "mugger_escape",
            _ => unreachable!("known Mugger suffix"),
        },
        _ => unreachable!("thief species"),
    })
}

fn encounter(species: &[EnemyId], gold: u32) -> Combat {
    let run_scope = vec!["thieves-fixture".to_owned()];
    let enemies = species.iter().enumerate().map(|(slot, id)| {
        let owner = match id { EnemyId::Looter => "looter-1", EnemyId::Mugger => "mugger-1", _ => unreachable!() };
        let hp = if *id == EnemyId::Looter { 44 } else { 48 };
        FullEnemySetup {
            initial_memory: Some(EnemyMemory::Thief(ThiefMemory::default())),
            instance_id: EnemyHandle::parse(owner).unwrap(),
            enemy_id: *id,
            ascension: 0,
            profile: enemies::EnemyProfile::Standard,
            hp,
            max_hp: hp,
            block: 0,
            intent: full_intent(*id, "mug"),
            powers: vec![],
            ai_rng: stream(Domain::RunCombatEnemyAi,
                vec![run_scope[0].clone(), format!("{owner}-{slot}")]),
        }
    }).collect();
    let setup = FullCombatSetup {
        player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3,
            max_energy: 3, draw_per_turn: 1, powers: vec![] },
        player_powers: vec![],
        character: Character::Ironclad,
        card_rng: stream(Domain::RunCombatCardRandom, run_scope.clone()),
        shuffle_rng: stream(Domain::RunCombatShuffle, run_scope),
        enemies,
        enemy_damage_bases: Default::default(),
    };
    let mut combat = Combat::enter(CombatEntry {
        setup,
        opening_draw: 1,
        cards: vec![permanent_live_card(CardId::Strike, CardInstanceId(1), 0)],
    }).unwrap();
    thief_actor::initialize_gold(&mut combat.state, gold).unwrap();
    combat
}

fn enemy_move(combat: &mut Combat, slot: usize) {
    let owner = combat.state.enemies[slot].instance_id.clone();
    let actor = Actor::Enemy(owner);
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(actor.clone()),
        target: Some(actor),
        kind: EffectKind::EnemyMove,
    }], None).unwrap();
}

fn hit(combat: &mut Combat, slot: usize, amount: i32) {
    let owner = combat.state.enemies[slot].instance_id.clone();
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(owner)),
        kind: EffectKind::Damage(DamagePayload::Fixed(amount)),
    }], None).unwrap();
}

fn memory(combat: &Combat, slot: usize) -> &ThiefMemory {
    match combat.state.enemies[slot].full_memory.as_ref().unwrap() {
        EnemyMemory::Thief(memory) => memory,
        other => panic!("expected Thief memory, got {other:?}"),
    }
}

fn force_move(combat: &mut Combat, slot: usize, intent: FullIntentId, history: &[FullIntentId]) {
    let enemy = &mut combat.state.enemies[slot];
    enemy.intent_id = ActorIntentId::Full(intent);
    enemy.selected_move = ActorIntentId::Full(intent);
    enemy.move_history = history.iter().copied().map(ActorIntentId::Full).collect();
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

// Removing the independent current-gold cap or sharing a ledger across actors
// must fail this real queued-turn test, even with enough Block to absorb damage.
#[test]
fn theft_caps_against_current_gold_per_actor_even_through_full_block() {
    let mut combat = encounter(&[EnemyId::Looter, EnemyId::Mugger], 19);
    combat.state.player.block = 100;
    let starting_hp = combat.state.player.hp;

    enemy_move(&mut combat, 0);
    assert_eq!(thief_actor::current_gold(&combat.state), Some(4));
    assert_eq!(memory(&combat, 0).stolen_gold, 15);
    assert_eq!(memory(&combat, 1).stolen_gold, 0);
    assert_eq!(combat.state.player.hp, starting_hp, "Block absorbs Mug damage, not theft");

    enemy_move(&mut combat, 1);
    assert_eq!(thief_actor::current_gold(&combat.state), Some(0));
    assert_eq!(memory(&combat, 0).stolen_gold, 15);
    assert_eq!(memory(&combat, 1).stolen_gold, 4);
    assert_eq!(combat.state.player.hp, starting_hp, "the second actor has an independent ledger");
}

// Theft at zero gold must still resolve the authored attack through shared HP damage.
#[test]
fn empty_wallet_skips_only_gold_debit_not_mug_damage() {
    let mut combat = encounter(&[EnemyId::Looter], 0);
    let starting_hp = combat.state.player.hp;
    enemy_move(&mut combat, 0);
    assert_eq!(thief_actor::current_gold(&combat.state), Some(0));
    assert_eq!(memory(&combat, 0).stolen_gold, 0);
    assert_eq!(combat.state.player.hp, starting_hp - 10);
}

// Death earns a receipt from the actor ledger; it does not refund combat gold.
#[test]
fn killed_thief_receipt_uses_ledger_without_immediate_refund() {
    let mut combat = encounter(&[EnemyId::Looter], 15);
    let owner = combat.state.enemies[0].instance_id.clone();
    enemy_move(&mut combat, 0);
    assert_eq!(thief_actor::current_gold(&combat.state), Some(0));
    hit(&mut combat, 0, 500);

    assert_eq!(thief_actor::stolen_gold_rewards(&combat.state), vec![StolenGoldReceipt {
        owner: owner.clone(), amount: 15,
    }]);
    assert_eq!(thief_actor::current_gold(&combat.state), Some(0));
    assert_eq!(combat.state.enemies[0].life, ActorLife::Active);
    assert_eq!(combat.state.enemies[0].hp, 0);
}

// Real enemy turns commit the second-Mug branch before Thorns kills each actor;
// the unused branch latch must not prevent its ledger receipt or rewind AI RNG.
#[test]
fn second_mug_thorns_deaths_keep_receipts_gold_and_committed_ai_frontiers() {
    let mut combat = encounter(&[EnemyId::Looter, EnemyId::Mugger], 100);
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player),
        target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("thorns"), amount: 25 },
    }], None).unwrap();

    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.fault.is_none());
    assert_eq!(thief_actor::current_gold(&combat.state), Some(70));
    assert_eq!((combat.state.enemies[0].hp, combat.state.enemies[1].hp), (19, 23));
    assert_eq!((memory(&combat, 0).stolen_gold, memory(&combat, 1).stolen_gold), (15, 15));

    let ai_before = combat.state.enemies.iter().map(|enemy|
        enemy.full_ai_rng.as_ref().unwrap().clone()).collect::<Vec<_>>();
    combat.step_full(&CombatAction::EndTurn).unwrap();

    assert!(combat.state.fault.is_none());
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert_eq!(thief_actor::current_gold(&combat.state), Some(40),
        "death receipts are claimable and do not refund stolen combat gold");
    assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), vec![0, 0]);
    assert_eq!(thief_actor::stolen_gold_rewards(&combat.state).len(), 2);
    for (slot, owner, before) in [
        (0, EnemyId::Looter, &ai_before[0]),
        (1, EnemyId::Mugger, &ai_before[1]),
    ] {
        assert_eq!(memory(&combat, slot).stolen_gold, 30);
        assert!(memory(&combat, slot).reward_recorded);
        assert_eq!(memory(&combat, slot).pending_next, None,
            "the unused branch latch clears with the committed death receipt");
        let after = combat.state.enemies[slot].full_ai_rng.as_ref().unwrap();
        assert_eq!(after.word_index(), before.word_index() + 1,
            "second-Mug branch draw remains committed after death");
        assert!(thief_actor::stolen_gold_rewards(&combat.state).contains(&StolenGoldReceipt {
            owner: combat.state.enemies[slot].instance_id.clone(), amount: 30,
        }), "{owner:?} earns its own ledger receipt");
    }
}

// Escape uses the shared actor lifecycle. It is not a Death hook or a stolen-gold receipt.
#[test]
fn escape_marks_mugged_then_shared_all_escaped_settlement_runs_once() {
    let mut combat = encounter(&[EnemyId::Looter, EnemyId::Mugger], 30);
    for slot in 0..2 {
        let id = combat.state.enemies[slot].enemy_id.as_str();
        let species = if id == "looter" { EnemyId::Looter } else { EnemyId::Mugger };
        let history = [full_intent(species, "mug"), full_intent(species, "mug"),
            full_intent(species, "smoke_bomb"), full_intent(species, "escape")];
        force_move(&mut combat, slot, full_intent(species, "escape"), &history);
    }
    let deaths_before = combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death).count();

    enemy_move(&mut combat, 0);
    assert_eq!(combat.state.enemies[0].life, ActorLife::Escaped);
    assert!(!thief_actor::all_escaped(&combat.state));
    assert_eq!(combat.state.outcome, None);
    assert_eq!(thief_actor::stolen_gold_rewards(&combat.state), Vec::<StolenGoldReceipt>::new());

    enemy_move(&mut combat, 1);
    assert!(thief_actor::all_escaped(&combat.state));
    assert!(thief_actor::mugged(&combat.state));
    assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.life).collect::<Vec<_>>(),
        vec![ActorLife::Escaped, ActorLife::Escaped]);
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert_eq!(thief_actor::stolen_gold_rewards(&combat.state), Vec::<StolenGoldReceipt>::new());
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death).count(), deaths_before);
}

// At the resolver guard, accepted ledger and debit remain committed while the
// still-current shared HP damage is retained as the first unexecuted checkpoint.
#[test]
fn effect_guard_after_debit_retains_gold_ledger_and_unapplied_hp_damage() {
    let mut combat = encounter(&[EnemyId::Looter], 19);
    let hp = combat.state.player.hp;
    combat.state.resolved_effects = 99_996;
    let owner = combat.state.enemies[0].instance_id.clone();
    let actor = Actor::Enemy(owner);
    combat.state.pending.push_front(Effect {
        source: Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove,
    });
    assert_eq!(engine::resolve(&mut combat.state).status,TransitionStatus::Fault);

    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::EffectGuard);
    assert_eq!(combat.state.current.as_ref().unwrap().kind,
        EffectKind::Damage(DamagePayload::FullEnemyMove(full_intent(EnemyId::Looter, "mug"))));
    assert_eq!((thief_actor::current_gold(&combat.state), memory(&combat, 0).stolen_gold), (Some(4), 15));
    assert_eq!(memory(&combat, 0).pending_theft, None);
    assert_eq!(combat.state.player.hp, hp, "the retained current damage has not run");
}

// A foreign producer cannot commit a thief ledger with a player-owned source.
#[test]
fn ledger_effect_requires_matching_thief_source_and_player_target() {
    let combat = encounter(&[EnemyId::Looter], 19);
    let owner = combat.state.enemies[0].instance_id.clone();
    let before_gold = thief_actor::current_gold(&combat.state);
    let before_ledger = memory(&combat, 0).stolen_gold;
    let intent = full_intent(EnemyId::Looter, "mug");
    let result = engine::validate_effect(&Effect {
        source: Source::Actor(Actor::Player),
        target: Some(Actor::Player),
        kind: EffectKind::Thief(thief_actor::ThiefEffect::Ledger { owner, cap: 15, intent }),
    }, &combat.state);
    assert!(result.is_err());
    assert_eq!(thief_actor::current_gold(&combat.state), before_gold);
    assert_eq!(memory(&combat, 0).stolen_gold, before_ledger);
}

#[test]
fn runtime_validation_rejects_foreign_thief_current_and_historical_moves() {
    let mut current = encounter(&[EnemyId::Looter], 0);
    current.state.enemies[0].intent_id = ActorIntentId::Full(FullIntentId::from_name("unknown_move"));
    assert!(thief_actor::validate_state(&current.state).is_err());

    let mut history = encounter(&[EnemyId::Looter], 0);
    history.state.enemies[0].move_history[0] = ActorIntentId::Full(full_intent(EnemyId::Mugger, "mug"));
    assert!(thief_actor::validate_state(&history.state).is_err());

    let mut selected = encounter(&[EnemyId::Looter], 0);
    selected.state.enemies[0].selected_move = ActorIntentId::Full(full_intent(EnemyId::Mugger, "mug"));
    assert!(thief_actor::validate_state(&selected.state).is_err());
}

// The source rolls after the second Mug turn, before the ledger action. The
// later selector consumes that committed branch without another AI draw.
#[test]
fn looter_and_mugger_commit_two_mug_branch_in_take_turn_once() {
    for id in [EnemyId::Looter, EnemyId::Mugger] {
        let stream = stream(Domain::RunCombatEnemyAi,
            vec!["selector-fixture".into(), id.as_str().into()]);
        let mug = full_intent(id, "mug");
        let lunge = full_intent(id, "lunge");
        let smoke = full_intent(id, "smoke_bomb");
        let escape = full_intent(id, "escape");
        let history = [mug, mug];
        let opening_memory = EnemyMemory::Thief(ThiefMemory::default());
        let first = thief_actor::select_move(id, &[], &opening_memory, &stream).unwrap();
        assert_eq!(first.intent, mug);
        assert_eq!(first.stream, stream, "opening Mug consumes no AI draw");
        let second = thief_actor::select_move(id, &[mug], &opening_memory, &stream).unwrap();
        assert_eq!(second.intent, mug);
        assert_eq!(second.stream, stream, "second Mug is forced");
        assert!(thief_actor::select_move(id, &history, &opening_memory, &stream).is_err(),
            "RollMove cannot invent the branch before the second Mug takeTurn");
        let mut saw_smoke = false;
        let mut saw_lunge = false;
        for cursor in 0..256 {
            let branch_stream = StreamState::new(stream.id().clone(), *stream.key(), cursor).unwrap();
            let (smoke_branch, after_coin) = rng::bernoulli(&branch_stream, 1, 2).unwrap();
            let expected = if smoke_branch { smoke } else { lunge };
            let mut combat = encounter(&[id], 30);
            force_move(&mut combat, 0, mug, &history);
            combat.state.enemies[0].full_ai_rng = Some(branch_stream);

            let owner = combat.state.enemies[0].instance_id.clone();
            let work = thief_actor::take_turn(&mut combat.state, &owner).unwrap();
            assert!(matches!(work.first().map(|effect| &effect.kind),
                Some(EffectKind::Thief(thief_actor::ThiefEffect::Ledger { .. }))));
            assert!(matches!(work.get(1).map(|effect| &effect.kind),
                Some(EffectKind::Thief(thief_actor::ThiefEffect::TheftDamage { .. }))));
            assert_eq!(memory(&combat, 0).pending_next, Some(expected),
                "takeTurn owns its choice before returning ledger work");
            assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref(), Some(&after_coin));

            let choice = thief_actor::select_move(id, &history,
                &EnemyMemory::Thief(memory(&combat, 0).clone()),
                combat.state.enemies[0].full_ai_rng.as_ref().unwrap()).unwrap();
            assert_eq!(choice.intent, expected);
            assert_eq!(choice.stream, after_coin, "RollMove must not redraw the branch");
            let EnemyMemory::Thief(selected_memory) = choice.memory else { panic!("thief memory"); };
            assert_eq!(selected_memory.pending_next, None, "selection consumes the committed latch");
            saw_smoke |= smoke_branch;
            saw_lunge |= !smoke_branch;

            let after_lunge = thief_actor::select_move(id, &[mug, mug, lunge],
                &EnemyMemory::Thief(ThiefMemory::default()), &after_coin).unwrap();
            assert_eq!((after_lunge.intent, after_lunge.stream), (smoke, after_coin.clone()));
            let after_smoke = thief_actor::select_move(id, &[mug, mug, smoke],
                &EnemyMemory::Thief(ThiefMemory::default()), &after_coin).unwrap();
            assert_eq!((after_smoke.intent, after_smoke.stream), (escape, after_coin));
        }
        assert!(saw_smoke && saw_lunge, "both fair branches must be represented by the fixture streams");
    }
}

// A fault at the second-Mug ledger retains the already-owned branch and draw.
// Selection from that checkpoint uses the same stream frontier without redrawing.
#[test]
fn second_mug_branch_is_committed_before_faulting_ledger_and_is_not_redrawn() {
    let mut combat = encounter(&[EnemyId::Looter], 30);
    enemy_move(&mut combat, 0);
    let owner = combat.state.enemies[0].instance_id.clone();
    let history = [full_intent(EnemyId::Looter, "mug"), full_intent(EnemyId::Looter, "mug")];
    assert_eq!(combat.state.enemies[0].move_history,
        history.map(ActorIntentId::Full).to_vec());
    let before = combat.state.enemies[0].full_ai_rng.as_ref().unwrap().clone();
    let (smoke_branch, after_coin) = rng::bernoulli(&before, 1, 2).unwrap();
    let branch = if smoke_branch { full_intent(EnemyId::Looter, "smoke_bomb") }
        else { full_intent(EnemyId::Looter, "lunge") };
    let gold_before = thief_actor::current_gold(&combat.state);
    let ledger_before = memory(&combat, 0).stolen_gold;

    combat.state.resolved_effects = 99_998;
    combat.state.pending.push_front(Effect {
        source:Source::Actor(Actor::Enemy(owner.clone())),target:Some(Actor::Enemy(owner)),kind:EffectKind::EnemyMove,
    });
    assert_eq!(engine::resolve(&mut combat.state).status,TransitionStatus::Fault);

    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::EffectGuard);
    assert!(matches!(combat.state.current.as_ref().map(|effect| &effect.kind),
        Some(EffectKind::Thief(thief_actor::ThiefEffect::Ledger { .. }))));
    assert_eq!(memory(&combat, 0).pending_next, Some(branch));
    assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref(), Some(&after_coin));
    assert_eq!(thief_actor::current_gold(&combat.state), gold_before);
    assert_eq!(memory(&combat, 0).stolen_gold, ledger_before,
        "the fault precedes the second ledger checkpoint");

    let selection = thief_actor::select_move(EnemyId::Looter, &history,
        &EnemyMemory::Thief(memory(&combat, 0).clone()),
        combat.state.enemies[0].full_ai_rng.as_ref().unwrap()).unwrap();
    assert_eq!(selection.intent, branch);
    assert_eq!(selection.stream, after_coin, "the committed branch consumes no second draw");
}
