use super::super::*;
use crate::content::{CardId, Character, EncounterId, EnemyId};

fn entry(encounter: EncounterId, card: CardId) -> CombatEntry {
    enemies::setup(encounter, 2, 0, &mut crate::run::RunRng::new([31; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3,
                draw_per_turn: 1, powers: vec![] },
            player_powers: vec![], character: Character::Ironclad,
            cards: vec![permanent_live_card(card, CardInstanceId(1), 0)], opening_draw: 1,
            encounter_visit: 0,
        }).unwrap()
}

fn set(combat: &mut Combat, name: &'static str) {
    let intent = ActorIntentId::Full(FullIntentId::from_name(name));
    let actor = &mut combat.state.enemies[0];
    actor.intent_id = intent.clone();
    actor.selected_move = intent.clone();
    actor.move_history = vec![intent];
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

fn amount(combat: &Combat, slot: usize, id: &str) -> i32 {
    combat.state.enemies[slot].full_powers.iter()
        .find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn stream(cursor: u128) -> rng::StreamState {
    let id = rng::StreamId::new(rng::Domain::RunCombatEnemyAi,
        vec!["ordinary-closure-test".into(), "actor-ai".into()]).unwrap();
    rng::StreamState::new(id, [31; 32], cursor).unwrap()
}

fn assert_fixture_ticket(stream: &rng::StreamState, bound: u128, expected: u64) -> rng::StreamState {
    let (ticket, next) = rng::uniform_below(stream, bound).unwrap();
    assert_eq!(ticket, expected, "pinned RNG fixture cursor {} / bound {bound}", stream.word_index());
    next
}

fn native_damage(combat: &mut Combat, target: usize, source: Source, payload: DamagePayload) {
    let owner = combat.state.enemies[target].instance_id.clone();
    combat.enqueue_native_effects(vec![Effect {
        source, target: Some(Actor::Enemy(owner)), kind: EffectKind::Damage(payload),
    }], None).unwrap();
}

#[test]
fn ordinary_closure_openers_match_their_distinct_native_ai_laws() {
    let initial = EnemyMemory::for_enemy(EnemyId::ShelledParasite).unwrap();
    for (cursor, ticket, expected) in [
        (0, 0, "shelled_parasite_double_strike"),
        (6, 1, "shelled_parasite_life_suck"),
    ] {
        let stream = stream(cursor);
        let after = assert_fixture_ticket(&stream, 2, ticket);
        let selected = enemies::select_move(EnemyId::ShelledParasite, &[], &initial, &stream).unwrap();
        assert_eq!(selected.intent.as_str(), expected);
        assert_eq!(selected.stream, after);
        assert_eq!(selected.memory, EnemyMemory::ShelledParasite { first: false });
    }

    let snake = EnemyMemory::for_enemy(EnemyId::SnakePlant).unwrap();
    for (cursor, ticket, expected) in [
        (21, 64, "snake_plant_chomp_chomp"),
        (30, 65, "snake_plant_enfeebling_spores"),
    ] {
        let stream = stream(cursor);
        let after = assert_fixture_ticket(&stream, 100, ticket);
        let selected = enemies::select_move(EnemyId::SnakePlant, &[], &snake, &stream).unwrap();
        assert_eq!(selected.intent.as_str(), expected);
        assert_eq!(selected.stream, after, "Snake Plant uses one bounded roll");
    }
}

#[test]
fn shelled_parasite_redirects_exact_a0_buckets_and_uses_the_fell_reroll_range() {
    let memory = EnemyMemory::ShelledParasite { first: false };
    let double = FullIntentId::from_name("shelled_parasite_double_strike");
    let life_suck = FullIntentId::from_name("shelled_parasite_life_suck");
    let fell = FullIntentId::from_name("shelled_parasite_fell");
    for (cursor, ticket, history, expected) in [
        (272, 19, vec![double], "shelled_parasite_fell"),
        (107, 20, vec![double], "shelled_parasite_double_strike"),
        (27, 59, vec![double], "shelled_parasite_double_strike"),
        (18, 60, vec![double], "shelled_parasite_life_suck"),
        (49, 99, vec![life_suck], "shelled_parasite_life_suck"),
        (107, 20, vec![double, double], "shelled_parasite_life_suck"),
        (18, 60, vec![life_suck, life_suck], "shelled_parasite_double_strike"),
    ] {
        let stream = stream(cursor);
        let after = assert_fixture_ticket(&stream, 100, ticket);
        let selected = enemies::select_move(EnemyId::ShelledParasite, &history, &memory, &stream).unwrap();
        assert_eq!(selected.intent.as_str(), expected, "ticket {ticket}, history {history:?}");
        assert_eq!(selected.stream, after);
    }

    let fell_history = [double, fell];
    for (cursor, secondary, expected) in [
        (11_136, 39, "shelled_parasite_double_strike"),
        (255, 40, "shelled_parasite_life_suck"),
    ] {
        let stream = stream(cursor);
        let after_primary = assert_fixture_ticket(&stream, 100, 0);
        let after_reroll = assert_fixture_ticket(&after_primary, 80, secondary);
        let selected = enemies::select_move(EnemyId::ShelledParasite, &fell_history, &memory, &stream).unwrap();
        assert_eq!(selected.intent.as_str(), expected, "secondary ticket {secondary}");
        assert_eq!(selected.stream, after_reroll, "a rejected Fell proposal consumes the 20..=99 reroll");
    }
}

#[test]
fn snake_plant_repeat_rules_forbid_third_chomp_and_consecutive_spores() {
    let memory = EnemyMemory::for_enemy(EnemyId::SnakePlant).unwrap();
    let chomp = FullIntentId::from_name("snake_plant_chomp_chomp");
    let spores = FullIntentId::from_name("snake_plant_enfeebling_spores");
    for (cursor, ticket, history, expected) in [
        (12, 0, vec![chomp, chomp], "snake_plant_enfeebling_spores"),
        (49, 99, vec![chomp, chomp], "snake_plant_enfeebling_spores"),
        (49, 99, vec![spores], "snake_plant_chomp_chomp"),
        (21, 64, vec![spores], "snake_plant_chomp_chomp"),
        (21, 64, vec![chomp], "snake_plant_chomp_chomp"),
        (30, 65, vec![chomp], "snake_plant_enfeebling_spores"),
    ] {
        let stream = stream(cursor);
        assert_fixture_ticket(&stream, 100, ticket);
        let selected = enemies::select_move(EnemyId::SnakePlant, &history, &memory, &stream).unwrap();
        assert_eq!(selected.intent.as_str(), expected, "ticket {ticket}, history {history:?}");
    }
}

#[test]
fn shelled_parasite_setup_and_real_double_strike_and_life_suck_turns_resolve() {
    let mut combat = Combat::enter(entry(EncounterId::ShelledParasite, CardId::Strike)).unwrap();
    assert!((68..=72).contains(&combat.state.enemies[0].max_hp));
    assert_eq!((amount(&combat, 0, "plated_armor"), combat.state.enemies[0].block), (14, 14));
    combat.state.enemies[0].block = 0;
    set(&mut combat, "shelled_parasite_double_strike");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player.hp, 68, "Double Strike resolves two base-six hits");
    assert_eq!(combat.state.enemies[0].block, 14,
        "enemy Plated Armor grants its current amount during pre-end-turn-card work");

    let mut combat = Combat::enter(entry(EncounterId::ShelledParasite, CardId::Strike)).unwrap();
    let max = combat.state.enemies[0].max_hp;
    combat.state.enemies[0].hp = max - 4;
    combat.state.player.block = 4;
    set(&mut combat, "shelled_parasite_life_suck");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((combat.state.player.hp, combat.state.enemies[0].hp), (74, max),
        "Life Suck heals from its post-damage loss amount, up to max HP");
}

#[test]
fn shelled_life_suck_uses_clamped_post_block_damage_and_queues_heal_at_front() {
    let mut combat = Combat::enter(entry(EncounterId::ShelledParasite, CardId::Strike)).unwrap();
    set(&mut combat, "shelled_parasite_life_suck");
    let owner = combat.state.enemies[0].instance_id.clone();
    combat.state.pending.push_back(Effect {
        source: Source::None, target: Some(Actor::Player), kind: EffectKind::GainBlock(1),
    });
    let result = crate::combat::model::DamageResult {
        source: Source::Actor(Actor::Enemy(owner.clone())), target: Actor::Player,
        normal: true, hp_loss: false, thorns: false, attempted: 10, blocked: 4,
        hp_lost: 3, hp_after: 0,
    };
    let life_suck = DamagePayload::FullEnemyMove(FullIntentId::from_name("shelled_parasite_life_suck"));
    enemies::act2::ordinary_closure::vampire_damage_complete(&mut combat.state, &result, &life_suck);
    assert!(matches!(combat.state.pending.front().map(|effect| &effect.kind),
        Some(EffectKind::Heal(3))),
        "Life Suck uses min(post-Block 6, pre-hit HP 3), not uncapped damage or overkill");

    combat.state.pending.clear();
    let blocked = crate::combat::model::DamageResult {
        source: Source::Actor(Actor::Enemy(owner)), target: Actor::Player,
        normal: true, hp_loss: false, thorns: false, attempted: 10, blocked: 10,
        hp_lost: 0, hp_after: 80,
    };
    enemies::act2::ordinary_closure::vampire_damage_complete(&mut combat.state, &blocked, &life_suck);
    assert!(combat.state.pending.is_empty(), "fully blocked Life Suck has lastDamageTaken zero");
}

#[test]
fn shelled_plated_armor_end_turn_captures_the_remaining_amount_after_thorns() {
    let mut combat = Combat::enter(entry(EncounterId::ShelledParasite, CardId::Strike)).unwrap();
    combat.state.enemies[0].block = 0;
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Fixed(1));
    assert_eq!(amount(&combat, 0, "plated_armor"), 13);
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Thorns(1));
    assert_eq!(amount(&combat, 0, "plated_armor"), 13,
        "THORNS loss does not decrement Plated Armor");
    set(&mut combat, "shelled_parasite_stunned");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.enemies[0].block, 13,
        "the pre-end-turn-card callback uses the remaining amount when it captures work");
}

#[test]
fn shelled_plated_armor_loss_uses_qualifying_hp_loss_and_break_forces_stun() {
    let mut combat = Combat::enter(entry(EncounterId::ShelledParasite, CardId::Strike)).unwrap();
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.enemies[0].block = 0;
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Fixed(1));
    assert_eq!(amount(&combat, 0, "plated_armor"), 13);

    combat.state.enemies[0].block = 1;
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Fixed(1));
    assert_eq!(amount(&combat, 0, "plated_armor"), 13, "fully blocked damage is not HP lost");
    combat.state.enemies[0].block = 0;
    native_damage(&mut combat, 0, Source::None, DamagePayload::Fixed(1));
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::HpLoss(1));
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Thorns(1));
    native_damage(&mut combat, 0, Source::Actor(owner.clone()), DamagePayload::Fixed(1));
    assert_eq!(amount(&combat, 0, "plated_armor"), 13,
        "null-source, HP_LOSS, THORNS, and self-source damage do not reduce Plated Armor");

    combat.state.enemies[0].block = 0;
    for _ in 0..13 {
        native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Fixed(1));
    }
    assert_eq!(amount(&combat, 0, "plated_armor"), 0);
    assert_eq!(combat.state.enemies[0].intent_id.as_str(), "shelled_parasite_stunned",
        "removing the last Plated Armor stack publishes the forced Stun intent");
    let hp = combat.state.player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player.hp, hp, "the forced Stunned turn has no damage payload");
    let history = &combat.state.enemies[0].move_history;
    assert_eq!(history.iter().rev().nth(1).unwrap().as_str(), "shelled_parasite_fell",
        "the actual Stun turn records Fell before its later RollMove selection");
}

#[test]
fn shelled_stun_turn_records_fell_before_roll_move_and_rejects_fell_again() {
    let mut combat = Combat::enter(entry(EncounterId::ShelledParasite, CardId::Strike)).unwrap();
    set(&mut combat, "shelled_parasite_stunned");
    combat.state.enemies[0].full_memory = Some(EnemyMemory::ShelledParasite { first: false });
    let owner = combat.state.enemies[0].instance_id.clone();
    let work = enemies::act2::ordinary_closure::take_turn(&mut combat.state, &owner).unwrap();
    assert!(work.is_empty(), "Stunned itself has no damage payload");
    assert_eq!(combat.state.enemies[0].intent_id.as_str(), "shelled_parasite_fell");
    assert_eq!(combat.state.enemies[0].move_history.last().unwrap().as_str(), "shelled_parasite_fell",
        "takeTurn sets Fell before the scheduler performs the later RollMove");

    let history = combat.state.enemies[0].move_history.iter().map(|intent| match intent {
        ActorIntentId::Full(intent) => *intent,
        ActorIntentId::Legacy(_) => panic!("full Shelled Parasite history must stay typed"),
    }).collect::<Vec<_>>();
    let stream = stream(11_136);
    let after_primary = assert_fixture_ticket(&stream, 100, 0);
    let after_reroll = assert_fixture_ticket(&after_primary, 80, 39);
    let selected = enemies::act2::ordinary_closure::select_move(
        EnemyId::ShelledParasite, &history, &EnemyMemory::ShelledParasite { first: false }, &stream,
    ).unwrap();
    assert_eq!(selected.intent.as_str(), "shelled_parasite_double_strike",
        "the next 0..19 proposal is rejected because Fell was just recorded");
    assert_eq!(selected.stream, after_reroll,
        "the rejected Fell consumes the primary roll and 20..=99 retry before the shared RollMove completes");
}

#[test]
fn snake_plant_setup_spores_turn_and_malleable_multihit_response_resolve() {
    let mut combat = Combat::enter(entry(EncounterId::SnakePlant, CardId::Strike)).unwrap();
    assert!((75..=79).contains(&combat.state.enemies[0].max_hp));
    assert_eq!(amount(&combat, 0, "malleable"), 3);
    set(&mut combat, "snake_plant_enfeebling_spores");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player_full_powers.iter().find(|power| power.id.0 == "frail").unwrap().state.amount, 2);
    assert_eq!(combat.state.player_full_powers.iter().find(|power| power.id.0 == "weak").unwrap().state.amount, 2);

    let mut combat = Combat::enter(entry(EncounterId::SnakePlant, CardId::Strike)).unwrap();
    set(&mut combat, "snake_plant_chomp_chomp");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player.hp, 59, "Chomp Chomp resolves three base-seven hits");

    let mut combat = Combat::enter(entry(EncounterId::SnakePlant, CardId::TwinStrike)).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();
    assert_eq!((combat.state.enemies[0].max_hp - combat.state.enemies[0].hp,
        combat.state.enemies[0].block, amount(&combat, 0, "malleable")), (10, 7, 5),
        "two qualifying Twin Strike hits queue Block3 then Block4 before the Malleable amount reaches 5");
    set(&mut combat, "snake_plant_enfeebling_spores");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(amount(&combat, 0, "malleable"), 3, "Snake Plant resets Malleable to its constructor base");
}

#[test]
fn snake_malleable_excludes_blocked_hp_loss_thorns_and_lethal_hits() {
    let mut combat = Combat::enter(entry(EncounterId::SnakePlant, CardId::Strike)).unwrap();
    combat.state.enemies[0].block = 2;
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Fixed(1));
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::HpLoss(1));
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Thorns(1));
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Orb(1));
    native_damage(&mut combat, 0, Source::None, DamagePayload::Fixed(1));
    assert_eq!((amount(&combat, 0, "malleable"), combat.state.enemies[0].block), (3, 0));

    let mut combat = Combat::enter(entry(EncounterId::SnakePlant, CardId::Strike)).unwrap();
    combat.state.enemies[0].hp = 1;
    native_damage(&mut combat, 0, Source::Actor(Actor::Player), DamagePayload::Fixed(5));
    assert_eq!(amount(&combat, 0, "malleable"), 3, "lethal damage does not trigger Malleable");
    assert_eq!(combat.state.enemies[0].hp, 0);
}
