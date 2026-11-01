//! M26 Task M1b character-sensitive law tests, authored against the H1 snapshot.
use super::super::*;
use crate::{
    content::{CardId, Character, EncounterId, PotionId},
    items::CombatItemContext,
    run::{AuthoredRunStart, CompleteCombatStart, Run, RunRng},
};

fn base_context(character: Character, complete: bool) -> CombatItemContext {
    let context = CombatItemContext::new(vec![], vec![None; 3]);
    let context = match character {
        Character::Ironclad => context,
        Character::Silent => context.with_m15_profile(),
        Character::Defect => context.with_m20_profile(),
        Character::Watcher => context.with_m25_profile(),
    };
    if complete { context.with_complete_cards() } else { context }
}

fn entry(
    character: Character,
    cards: &[(CardId, u32)],
    max_energy: i32,
    opening_draw: usize,
    draw_per_turn: i32,
) -> CombatEntry {
    let mut run_rng = RunRng::new([41; 32], rng::Partition::Test, 0);
    let mut entry = enemies::setup(EncounterId::JawWorm, 1, 0, &mut run_rng, enemies::SetupContext {
        player: PlayerState {
            hp: 80,
            max_hp: 80,
            block: 0,
            energy: max_energy,
            max_energy,
            draw_per_turn: draw_per_turn.max(1),
            powers: vec![],
        },
        player_powers: vec![],
        character,
        cards: vec![permanent_live_card(CardId::Defend, CardInstanceId(99), 0)],
        opening_draw,
        encounter_visit: 0,
    }).unwrap_or_else(|error| panic!("Jaw Worm fixture setup: {error:?}"));
    entry.cards = cards.iter().enumerate().map(|(index, (id, upgrades))| {
        complete_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades)
            .unwrap_or_else(|error| panic!("complete card {id:?}+{upgrades}: {error:?}"))
    }).collect();
    entry.opening_draw = opening_draw;
    entry
}

fn combat(
    character: Character,
    cards: &[(CardId, u32)],
    max_energy: i32,
    opening_draw: usize,
    draw_per_turn: i32,
    complete: bool,
) -> Combat {
    let mut context = Some(base_context(character, complete));
    let mut combat = Combat::enter_with_item_context(
        entry(character, cards, max_energy, opening_draw, draw_per_turn),
        &mut context,
    ).unwrap_or_else(|error| panic!("{character:?} combat entry: {error:?}"));
    combat.state.player.draw_per_turn = draw_per_turn;
    combat
}

fn play(combat: &mut Combat, instance: u64, target: Option<u64>) {
    let action = CombatAction::PlayCard { instance, target: target.map(TargetId) };
    combat.step_full(&action).unwrap_or_else(|error| panic!("{action:?}: {error:?}"));
    assert!(combat.fault_record().is_none(), "unexpected fault: {:?}", combat.fault_record());
}

fn end_turn(combat: &mut Combat) {
    combat.step_full(&CombatAction::EndTurn)
        .unwrap_or_else(|error| panic!("EndTurn: {error:?}"));
    assert!(combat.fault_record().is_none(), "unexpected fault: {:?}", combat.fault_record());
}

fn add_power(combat: &mut Combat, id: &'static str, amount: i32) {
    combat.state.player_full_powers.push(FullPowerEntry {
        id: FullPowerId(id),
        state: PowerState { amount, skip_first: false, monster_source_grace: false },
    });
}

fn power_amount(combat: &Combat, id: &'static str) -> Option<i32> {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id)
        .map(|power| power.state.amount)
}

fn add_draw(combat: &mut Combat, id: CardId, instance: u64) {
    let card = complete_live_card(id, CardInstanceId(instance), 0).unwrap();
    let handle = card_state::handle(card.instance);
    assert!(combat.state.card_ids_issued.insert(handle), "duplicate fixture instance {instance}");
    combat.state.piles.draw.push(model::OwnedCard::Full(card));
}

fn count_card(combat: &Combat, id: CardId) -> usize {
    combat.state.piles.hand.iter().chain(&combat.state.piles.draw)
        .chain(&combat.state.piles.discard).chain(&combat.state.piles.exhaust)
        .chain(combat.state.in_play.iter())
        .filter_map(model::OwnedCard::full).filter(|card| card.id == id).count()
}

fn normalize_overlay_debug(value: &str) -> String {
    value.replace(", complete_cards: true", "")
}

fn own_script_debug(character: Character, id: CardId, target: Option<u64>, turns: usize) -> Vec<String> {
    let mut combat = combat(character, &[(id, 0)], 3, 1, 0, false);
    let mut result = vec![format!("{:?}", combat.public_full().unwrap())];
    play(&mut combat, 1, target);
    result.push(format!("{:?}", combat.public_full().unwrap()));
    for _ in 0..turns {
        end_turn(&mut combat);
        result.push(format!("{:?}", combat.public_full().unwrap()));
        while combat.state.phase == Phase::Selection {
            combat.step_full(&CombatAction::ConfirmSelection)
                .unwrap_or_else(|error| panic!("confirm own {id:?} selection: {error:?}"));
            result.push(format!("{:?}", combat.public_full().unwrap()));
        }
    }
    result
}

fn assert_own_overlay_inert(character: Character, id: CardId, target: Option<u64>, turns: usize) {
    let base = own_script_debug(character, id, target, turns);
    let mut overlay = own_script_debug_with_overlay(character, id, target, turns);
    assert_eq!(base.len(), overlay.len(), "{character:?} {id:?} step count");
    for (step, (before, after)) in base.iter().zip(overlay.iter_mut()).enumerate() {
        let actual = normalize_overlay_debug(after);
        assert_eq!(before, &actual, "{character:?} {id:?} public bytes at step {step}");
    }
}

fn own_script_debug_with_overlay(character: Character, id: CardId, target: Option<u64>, turns: usize) -> Vec<String> {
    let mut context = Some(base_context(character, true));
    let mut combat = Combat::enter_with_item_context(entry(character, &[(id, 0)], 3, 1, 0), &mut context)
        .unwrap_or_else(|error| panic!("{character:?} overlay entry: {error:?}"));
    combat.state.player.draw_per_turn = 0;
    let mut result = vec![format!("{:?}", combat.public_full().unwrap())];
    play(&mut combat, 1, target);
    result.push(format!("{:?}", combat.public_full().unwrap()));
    for _ in 0..turns {
        end_turn(&mut combat);
        result.push(format!("{:?}", combat.public_full().unwrap()));
        while combat.state.phase == Phase::Selection {
            combat.step_full(&CombatAction::ConfirmSelection)
                .unwrap_or_else(|error| panic!("confirm overlay {id:?} selection: {error:?}"));
            result.push(format!("{:?}", combat.public_full().unwrap()));
        }
    }
    result
}

#[test]
fn m26_offcolour_stance_mantra_scry() {
    let mut ironclad = combat(Character::Ironclad, &[
        (CardId::Eruption, 0), (CardId::Strike, 0),
        (CardId::Vigilance, 0), (CardId::Eruption, 0),
    ], 10, 4, 0, true);
    ironclad.state.enemies[0].hp = 500;
    ironclad.state.enemies[0].max_hp = 500;
    ironclad.state.enemies[0].block = 0;
    add_power(&mut ironclad, "mental_fortress", 4);
    play(&mut ironclad, 1, Some(1));
    assert_eq!((ironclad.state.character.stance, ironclad.state.player.block), (Stance::Wrath, 4));
    let before = ironclad.state.enemies[0].hp;
    play(&mut ironclad, 2, Some(1));
    assert_eq!(before - ironclad.state.enemies[0].hp, 12, "Wrath doubles Strike");
    play(&mut ironclad, 3, None);
    assert_eq!((ironclad.state.character.stance, ironclad.state.player.block), (Stance::Calm, 16));
    play(&mut ironclad, 4, Some(1));
    assert_eq!(ironclad.state.player.energy, 5, "leaving Calm adds two energy");
    assert_eq!((ironclad.state.character.stance, ironclad.state.player.block), (Stance::Wrath, 20));

    let mut silent = combat(Character::Silent, &[
        (CardId::Worship, 0), (CardId::Worship, 0), (CardId::Strike, 0),
    ], 10, 3, 0, true);
    silent.state.enemies[0].hp = 500;
    silent.state.enemies[0].max_hp = 500;
    play(&mut silent, 1, None);
    play(&mut silent, 2, None);
    assert_eq!((silent.state.character.mantra_gained, silent.state.character.stance, silent.state.player.energy),
        (10, Stance::Divinity, 9));
    let before = silent.state.enemies[0].hp;
    play(&mut silent, 3, Some(1));
    assert_eq!(before - silent.state.enemies[0].hp, 18, "Divinity triples Strike");

    let mut defect = combat(Character::Defect, &[
        (CardId::ThirdEye, 0), (CardId::Defend, 0), (CardId::Defend, 0),
        (CardId::Defend, 0), (CardId::Defend, 0), (CardId::Defend, 0),
    ], 10, 6, 0, true);
    let mut third_eye = None;
    let mut draw = Vec::new();
    for card in std::mem::take(&mut defect.state.piles.hand) {
        if card.full().is_some_and(|live| live.instance == CardInstanceId(1)) {
            third_eye = Some(card);
        } else {
            draw.push(card);
        }
    }
    defect.state.piles.hand.push(third_eye.expect("Third Eye in hand"));
    defect.state.piles.draw = draw;
    play(&mut defect, 1, None);
    let selection = defect.public_full().unwrap().selection.expect("Third Eye opens Scry");
    assert_eq!((selection.minimum, selection.maximum, selection.candidates.len()), (0, 3, 3));
    assert_eq!(selection.parent, SelectionParent::Card(card_state::handle(CardInstanceId(1))));
}

#[test]
fn m26_offcolour_turn_structure() {
    let mut divinity = combat(Character::Ironclad, &[(CardId::Strike, 0)], 3, 1, 0, true);
    divinity.state.character.stance = Stance::Divinity;
    end_turn(&mut divinity);
    assert_eq!(divinity.state.character.stance, Stance::Neutral, "Divinity ends at the next Ironclad turn start");

    let mut deva = combat(Character::Ironclad, &[(CardId::DevaForm, 0)], 3, 1, 0, true);
    play(&mut deva, 1, None);
    end_turn(&mut deva);
    assert_eq!((deva.state.turn, deva.state.player.energy), (2, 4));
    end_turn(&mut deva);
    assert_eq!((deva.state.turn, deva.state.player.energy), (3, 5));

    let mut collect = combat(Character::Ironclad, &[(CardId::Collect, 0)], 3, 1, 0, true);
    collect.state.player.energy = 2;
    play(&mut collect, 1, None);
    assert_eq!(power_amount(&collect, "collect"), Some(2), "X is two");
    end_turn(&mut collect);
    assert_eq!(count_card(&collect, CardId::Miracle), 1);
    assert!(collect.state.piles.hand.iter().filter_map(model::OwnedCard::full)
        .filter(|card| card.id == CardId::Miracle).all(|card| card.upgrades == 1));
    end_turn(&mut collect);
    assert_eq!(count_card(&collect, CardId::Miracle), 2);
    assert!(collect.state.piles.all(collect.state.in_play.as_ref()).filter_map(model::OwnedCard::full)
        .filter(|card| card.id == CardId::Miracle).all(|card| card.upgrades == 1));

    let mut simmering = combat(Character::Ironclad, &[(CardId::SimmeringFury, 0)], 3, 1, 0, true);
    add_draw(&mut simmering, CardId::Strike, 40);
    add_draw(&mut simmering, CardId::Defend, 41);
    play(&mut simmering, 1, None);
    end_turn(&mut simmering);
    assert_eq!(simmering.state.character.stance, Stance::Wrath);
    let mut drawn = simmering.state.piles.hand.iter().filter_map(model::OwnedCard::full)
        .map(|card| card.id).collect::<Vec<_>>();
    drawn.sort_by_key(|id| id.as_str());
    assert_eq!(drawn, vec![CardId::Defend, CardId::Strike]);
    for id in [CardId::DevaForm, CardId::Collect, CardId::SimmeringFury] {
        assert_own_overlay_inert(Character::Watcher, id, None, 2);
    }
}

#[test]
fn m26_offcolour_orbs_zero_slots() {
    // Disclosed decision F12 (seam map H0 §7): at 0 slots channel/evoke do nothing, Consume still gives
    // Focus with slots staying 0, and orb-count readers (Barrage, Thunder Strike) read 0.
    for character in [Character::Ironclad, Character::Silent, Character::Watcher] {
        let mut combat = combat(character, &[
            (CardId::Zap, 0), (CardId::ColdSnap, 0), (CardId::Dualcast, 0),
            (CardId::BallLightning, 0), (CardId::Defragment, 0), (CardId::Barrage, 0),
            (CardId::ThunderStrike, 0), (CardId::Consume, 0),
        ], 20, 8, 0, true);
        combat.state.enemies[0].hp = 500;
        combat.state.enemies[0].max_hp = 500;
        let lightning = combat.state.defect.lightning_channeled;
        play(&mut combat, 1, None);
        assert!(combat.state.character.orb_slots.is_empty(), "{character:?} Zap at zero slots");
        assert_eq!(combat.state.defect.lightning_channeled, lightning);

        let before = combat.state.enemies[0].hp;
        play(&mut combat, 2, Some(1));
        assert_eq!(before - combat.state.enemies[0].hp, 6, "{character:?} Cold Snap damage");
        assert!(combat.state.character.orb_slots.is_empty());

        play(&mut combat, 3, None);
        assert!(combat.state.character.orb_slots.is_empty(), "Dualcast evokes nothing");
        let before = combat.state.enemies[0].hp;
        play(&mut combat, 4, Some(1));
        assert_eq!(before - combat.state.enemies[0].hp, 7, "Ball Lightning still hits");
        assert!(combat.state.character.orb_slots.is_empty());

        play(&mut combat, 5, None);
        assert_eq!(combat.state.character.focus, 1, "Defragment gives one Focus");
        let before = combat.state.enemies[0].hp;
        play(&mut combat, 6, Some(1));
        assert_eq!(combat.state.enemies[0].hp, before, "Barrage has zero orb hits");
        let before = combat.state.enemies[0].hp;
        play(&mut combat, 7, None);
        assert_eq!(combat.state.enemies[0].hp, before, "Thunder Strike has zero Lightning hits");
        play(&mut combat, 8, None);
        assert_eq!(combat.state.character.focus, 3, "Consume gives two more Focus");
        assert!(combat.state.character.orb_slots.is_empty());
        assert_eq!(combat.state.defect.lightning_channeled, lightning);
        assert!(combat.state.pending.is_empty(), "{character:?} has no queued orb effect after zero-slot plays");

        let ended = combat.step_full_observed(&CombatAction::EndTurn);
        ended.result.unwrap_or_else(|error| panic!("{character:?} zero-slot EndTurn: {error:?}"));
        assert!(ended.effects.iter().all(|effect| !matches!(effect.kind,
            ResolvedEffectKind::Damage { category: DamageCategory::Orb, .. })),
            "{character:?} zero slots enqueue no start/end orb damage");
        assert!(combat.state.pending.is_empty());
        assert!(combat.state.character.orb_slots.is_empty());
    }
}

#[test]
fn m26_offcolour_orb_passives_after_capacitor() {
    let mut combat = combat(Character::Ironclad, &[
        (CardId::Capacitor, 0), (CardId::Zap, 0),
        (CardId::Coolheaded, 0), (CardId::Loop, 0),
    ], 3, 4, 2, true);
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat.state.enemies[0].block = 0;
    play(&mut combat, 1, None);
    assert_eq!(combat.state.character.orb_slots.len(), 2);
    play(&mut combat, 2, None);
    assert_eq!(combat.state.character.orb_slots[0].unwrap().kind, OrbKind::Lightning);
    for instance in [3, 4] {
        let text = instance.to_string();
        let index = combat.state.piles.hand.iter().position(|card| card.handle().as_str() == text)
            .expect("Coolheaded and Loop remain in hand");
        let card = combat.state.piles.hand.remove(index);
        combat.state.piles.draw.push(card);
    }

    let before = combat.state.enemies[0].hp;
    let first_end = combat.step_full_observed(&CombatAction::EndTurn);
    first_end.result.unwrap_or_else(|error| panic!("first EndTurn: {error:?}"));
    let first_orb_hits = first_end.effects.iter().filter_map(|effect| match effect.kind {
        ResolvedEffectKind::Damage { incoming, category: DamageCategory::Orb, .. } => Some(incoming),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(first_orb_hits, vec![3]);
    assert_eq!(before - combat.state.enemies[0].hp, 3, "first Lightning passive");
    let orb_position = first_end.effects.iter().position(|effect| matches!(effect.kind,
        ResolvedEffectKind::Damage { category: DamageCategory::Orb, .. })).unwrap();
    let enemy_position = first_end.effects.iter().position(|effect| matches!(effect.kind,
        ResolvedEffectKind::EnemyMove { .. })).unwrap();
    assert!(orb_position < enemy_position, "F15 orb work resolves before the enemy decision");
    assert!(combat.state.pending.is_empty());

    play(&mut combat, 3, None);
    play(&mut combat, 4, None);
    assert_eq!(combat.state.character.orb_slots[1].unwrap().kind, OrbKind::Frost);
    let before = combat.state.enemies[0].hp;
    let second_end = combat.step_full_observed(&CombatAction::EndTurn);
    second_end.result.unwrap_or_else(|error| panic!("second EndTurn: {error:?}"));
    let second_orb_hits = second_end.effects.iter().filter_map(|effect| match effect.kind {
        ResolvedEffectKind::Damage { incoming, category: DamageCategory::Orb, .. } => Some(incoming),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(second_orb_hits, vec![3, 3], "turn-end Lightning and turn-start Loop passives");
    // Jaw Worm may Bellow (Block) between the two hits, so count Block absorbed plus HP lost.
    let _ = before;
    let dealt: i32 = second_end.effects.iter().filter_map(|effect| match effect.kind {
        ResolvedEffectKind::Damage { absorbed_block, hp_lost, category: DamageCategory::Orb, .. } => Some(absorbed_block + hp_lost),
        _ => None,
    }).sum();
    assert_eq!(dealt, 6, "second Lightning and Loop each hit once");
    assert!(second_end.effects.iter().any(|effect| matches!(effect.kind,
        ResolvedEffectKind::Block { before, after } if after - before == 2)), "Frost passive gains two Block");
    let second_orb_positions = second_end.effects.iter().enumerate().filter_map(|(index, effect)| matches!(effect.kind,
        ResolvedEffectKind::Damage { category: DamageCategory::Orb, .. }).then_some(index)).collect::<Vec<_>>();
    let enemy_position = second_end.effects.iter().position(|effect| matches!(effect.kind,
        ResolvedEffectKind::EnemyMove { .. })).unwrap();
    assert!(second_orb_positions[0] < enemy_position && enemy_position < second_orb_positions[1],
        "F15 places the Loop start passive after the intervening enemy phase");
    assert!(combat.state.pending.is_empty());
}

#[test]
fn m26_offcolour_turn_powers() {
    let mut echo = combat(Character::Ironclad, &[(CardId::EchoForm, 0)], 3, 1, 1, true);
    add_draw(&mut echo, CardId::Strike, 3);
    add_draw(&mut echo, CardId::Strike, 2);
    echo.state.enemies[0].hp = 500;
    echo.state.enemies[0].max_hp = 500;
    echo.state.enemies[0].block = 0;
    play(&mut echo, 1, None);
    assert!(echo.state.character.orb_slots.is_empty());
    end_turn(&mut echo);
    echo.state.enemies[0].block = 0;
    let before = echo.state.enemies[0].hp;
    play(&mut echo, 2, Some(1));
    assert_eq!(before - echo.state.enemies[0].hp, 12, "Echo Form doubles turn-two Strike");
    end_turn(&mut echo);
    echo.state.enemies[0].block = 0;
    let before = echo.state.enemies[0].hp;
    play(&mut echo, 3, Some(1));
    assert_eq!(before - echo.state.enemies[0].hp, 12, "Echo Form doubles turn-three Strike");

    let mut biased = combat(Character::Ironclad, &[(CardId::BiasedCognition, 0)], 3, 1, 0, true);
    play(&mut biased, 1, None);
    assert_eq!(biased.state.character.focus, 4);
    end_turn(&mut biased);
    assert_eq!((biased.state.turn, biased.state.character.focus), (2, 3));
    end_turn(&mut biased);
    assert_eq!((biased.state.turn, biased.state.character.focus), (3, 2));

    for character in [Character::Ironclad, Character::Silent, Character::Watcher] {
        let mut battery = combat(character, &[(CardId::ChargeBattery, 0)], 3, 1, 0, true);
        play(&mut battery, 1, None);
        end_turn(&mut battery);
        assert_eq!((battery.state.turn, battery.state.player.energy), (2, 4), "{character:?} Charge Battery once");
    }

    for character in [Character::Defect, Character::Watcher] {
        let mut outmaneuver = combat(character, &[(CardId::Outmaneuver, 0)], 3, 1, 0, true);
        play(&mut outmaneuver, 1, None);
        end_turn(&mut outmaneuver);
        assert_eq!((outmaneuver.state.turn, outmaneuver.state.player.energy), (2, 5), "{character:?} Outmaneuver once");
    }

    let mut flying_knee = combat(Character::Watcher, &[(CardId::FlyingKnee, 0)], 3, 1, 0, true);
    play(&mut flying_knee, 1, Some(1));
    end_turn(&mut flying_knee);
    assert_eq!((flying_knee.state.turn, flying_knee.state.player.energy), (2, 4), "M25 Flying Knee recharge");

    let mut watcher_powers = combat(Character::Ironclad, &[
        (CardId::Foresight, 0), (CardId::BattleHymn, 0), (CardId::Devotion, 0),
    ], 3, 3, 0, true);
    add_draw(&mut watcher_powers, CardId::Strike, 50);
    add_draw(&mut watcher_powers, CardId::Defend, 51);
    add_draw(&mut watcher_powers, CardId::Strike, 52);
    play(&mut watcher_powers, 1, None);
    play(&mut watcher_powers, 2, None);
    play(&mut watcher_powers, 3, None);
    end_turn(&mut watcher_powers);
    let selection = watcher_powers.public_full().unwrap().selection.expect("Ironclad Foresight opens Scry");
    assert_eq!((selection.minimum, selection.maximum, selection.candidates.len()), (0, 3, 3));
    assert_eq!(selection.parent, SelectionParent::Power { turn: 2, id: FullPowerId("foresight") });
    watcher_powers.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(count_card(&watcher_powers, CardId::Smite), 1);
    assert_eq!(watcher_powers.state.character.mantra_gained, 2);
    end_turn(&mut watcher_powers);
    assert!(watcher_powers.public_full().unwrap().selection.is_some(), "Foresight repeats next turn");
    watcher_powers.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(count_card(&watcher_powers, CardId::Smite), 2);
    assert_eq!(watcher_powers.state.character.mantra_gained, 4);

    let mut capacity = combat(Character::Ironclad, &[
        (CardId::Capacitor, 0), (CardId::Outmaneuver, 0),
    ], 3, 2, 0, true);
    play(&mut capacity, 1, None);
    assert_eq!(capacity.state.character.orb_slots.len(), 2);
    play(&mut capacity, 2, None);
    end_turn(&mut capacity);
    assert_eq!((capacity.state.turn, capacity.state.player.energy), (2, 5),
        "orb slots do not run Defect recharge or apply Outmaneuver twice");

    for (character, id, target) in [
        (Character::Defect, CardId::EchoForm, None),
        (Character::Defect, CardId::BiasedCognition, None),
        (Character::Defect, CardId::ChargeBattery, None),
        (Character::Defect, CardId::Capacitor, None),
        (Character::Defect, CardId::Loop, None),
        (Character::Silent, CardId::Outmaneuver, None),
        (Character::Silent, CardId::FlyingKnee, Some(1)),
        (Character::Watcher, CardId::Foresight, None),
        (Character::Watcher, CardId::BattleHymn, None),
        (Character::Watcher, CardId::Devotion, None),
        (Character::Watcher, CardId::Fasting, None),
    ] {
        assert_own_overlay_inert(character, id, target, 2);
    }
}

#[test]
fn m26_offcolour_fasting_energy_down() {
    for character in [Character::Ironclad, Character::Defect, Character::Watcher] {
        let mut combat = combat(character, &[(CardId::Fasting, 0)], 3, 1, 0, true);
        play(&mut combat, 1, None);
        assert_eq!(power_amount(&combat, "strength"), Some(3));
        assert_eq!(power_amount(&combat, "dexterity"), Some(3));
        assert_eq!(power_amount(&combat, "energy_down"), Some(1));
        end_turn(&mut combat);
        assert_eq!((combat.state.turn, combat.state.player.max_energy, combat.state.player.energy), (2, 3, 2),
            "{character:?} Fasting reduces next turn energy once");
        end_turn(&mut combat);
        assert_eq!((combat.state.turn, combat.state.player.energy), (3, 2),
            "{character:?} energy_down persists");
    }
}

#[test]
fn m26_offcolour_retain_discard() {
    let mut retained = combat(Character::Ironclad, &[
        (CardId::Protect, 0), (CardId::Strike, 0), (CardId::Strike, 0),
    ], 3, 3, 0, true);
    end_turn(&mut retained);
    assert_eq!(retained.state.piles.hand.iter().filter_map(model::OwnedCard::full)
        .map(|card| card.id).collect::<Vec<_>>(), vec![CardId::Protect]);
    assert_eq!(retained.state.piles.discard.iter().filter_map(model::OwnedCard::full)
        .map(|card| card.id).collect::<Vec<_>>(), vec![CardId::Strike, CardId::Strike]);

    let run = |overlay: bool| {
        let mut combat = combat(Character::Ironclad, &[(CardId::Strike, 0), (CardId::Strike, 0)], 3, 2, 0, overlay);
        let mut debug = vec![format!("{:?}", combat.public_full().unwrap())];
        end_turn(&mut combat);
        debug.push(format!("{:?}", combat.public_full().unwrap()));
        assert!(combat.state.piles.hand.is_empty());
        assert_eq!(combat.state.piles.discard.iter().filter_map(model::OwnedCard::full)
            .filter(|card| card.id == CardId::Strike).count(), 2);
        debug
    };
    let before = run(false);
    let after = run(true);
    assert_eq!(before.len(), after.len());
    for (step, (base, overlay)) in before.iter().zip(after.iter()).enumerate() {
        assert_eq!(base, &normalize_overlay_debug(overlay), "no-Retain discard bytes at step {step}");
    }
}

const SHARED_LIMITED_POTIONS: [PotionId; 29] = [
    PotionId::BlockPotion, PotionId::DexterityPotion, PotionId::EnergyPotion,
    PotionId::ExplosivePotion, PotionId::FirePotion, PotionId::StrengthPotion,
    PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion,
    PotionId::AttackPotion, PotionId::SkillPotion, PotionId::PowerPotion,
    PotionId::ColorlessPotion, PotionId::FlexPotion, PotionId::SpeedPotion,
    PotionId::BlessingOfTheForge, PotionId::RegenPotion, PotionId::AncientPotion,
    PotionId::LiquidBronze, PotionId::GamblersBrew, PotionId::EssenceOfSteel,
    PotionId::DuplicationPotion, PotionId::DistilledChaos, PotionId::LiquidMemories,
    PotionId::CultistPotion, PotionId::SneckoOil, PotionId::FairyPotion,
    PotionId::SmokeBomb, PotionId::EntropicBrew,
];

// The accepted Silent list has 30 shared potions after removing PoisonPotion,
// CunningPotion, and GhostInAJar; limited generation then removes FruitJuice.
fn limited_class_pool(character: Character) -> Vec<PotionId> {
    let class = match character {
        Character::Ironclad => [PotionId::BloodPotion, PotionId::Elixir, PotionId::HeartOfIron],
        Character::Silent => [PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar],
        Character::Defect => [PotionId::FocusPotion, PotionId::PotionOfCapacity, PotionId::EssenceOfDarkness],
        Character::Watcher => [PotionId::BottledMiracle, PotionId::StancePotion, PotionId::Ambrosia],
    };
    SHARED_LIMITED_POTIONS.into_iter().chain(class).collect()
}

fn alchemize(character: Character, seed: u8, potions: Vec<Option<PotionId>>) -> Combat {
    let mut entry = entry(character, &[(CardId::Alchemize, 0)], 3, 1, 0);
    let stream = &entry.setup.card_rng;
    entry.setup.card_rng = rng::StreamState::new(stream.id().clone(), [seed; 32], 0).unwrap();
    let mut context = Some(match character {
        Character::Ironclad => CombatItemContext::new(vec![], potions),
        Character::Silent => CombatItemContext::new(vec![], potions).with_m15_profile(),
        Character::Defect => CombatItemContext::new(vec![], potions).with_m20_profile(),
        Character::Watcher => CombatItemContext::new(vec![], potions).with_m25_profile(),
    }.with_complete_cards());
    let mut combat = Combat::enter_with_item_context(entry, &mut context)
        .unwrap_or_else(|error| panic!("{character:?} Alchemize entry: {error:?}"));
    combat.state.player.draw_per_turn = 0;
    combat
}

#[test]
fn m26_offcolour_alchemize_all_characters() {
    // H1 capture is filled with the eight deterministic Silent results below.
    let silent_h1 = (0u8..8).map(|seed| {
        let mut combat = alchemize(Character::Silent, seed, vec![None; 3]);
        play(&mut combat, 1, None);
        combat.item_context().unwrap().potions()[0].unwrap()
    }).collect::<Vec<_>>();
    // Captured by running the H1 code (pre-M26 Alchemize), fixed keys 0 through 7.
    assert_eq!(silent_h1.as_slice(), &[
        PotionId::FlexPotion, PotionId::SpeedPotion, PotionId::LiquidMemories,
        PotionId::PoisonPotion, PotionId::AttackPotion, PotionId::CultistPotion,
        PotionId::WeakPotion, PotionId::FearPotion,
    ], "Silent Alchemize preserves the H1 pre-M26 seed results");

    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let pool = limited_class_pool(character);
        let mut seen = Vec::new();
        for seed in 0u8..200 {
            let mut combat = alchemize(character, seed,
                vec![Some(PotionId::StrengthPotion), None, Some(PotionId::BlockPotion)]);
            play(&mut combat, 1, None);
            let potion = combat.item_context().unwrap().potions()[1]
                .unwrap_or_else(|| panic!("{character:?} seed {seed} gained no potion"));
            assert!(pool.contains(&potion), "{character:?} seed {seed} yielded off-pool {potion:?}");
            if !seen.contains(&potion) { seen.push(potion); }
        }
        seen.sort_by_key(|id| id.as_str());
        let mut expected = pool;
        expected.sort_by_key(|id| id.as_str());
        assert_eq!(seen, expected, "{character:?} limited pool coverage over 200 seeds");

        let belt = vec![Some(PotionId::BlockPotion); 3];
        let mut full = alchemize(character, 71, belt.clone());
        play(&mut full, 1, None);
        assert_eq!(full.item_context().unwrap().potions(), belt.as_slice(), "{character:?} full belt stays unchanged");
        assert!(full.fault_record().is_none(), "{character:?} full belt does not fault");
    }

    let mut defect = alchemize(Character::Defect, 137,
        vec![Some(PotionId::StrengthPotion), None, Some(PotionId::BlockPotion)]);
    let stream = defect.state.card_rng.clone().unwrap();
    let expected = crate::items::potion_generation::sample(Character::Defect, true, &stream).unwrap().0;
    play(&mut defect, 1, None);
    assert_eq!(defect.item_context().unwrap().potions()[1], Some(expected),
        "Defect Alchemize agrees with the class-pool sampler on the same stream");

    let resources = AuthoredRunStart {
        hp: 80,
        max_hp: 80,
        gold: 0,
        deck: vec![
            crate::run::CardInstance { misc: 0, instance: 1, id: CardId::Alchemize, upgrades: 0 },
            crate::run::CardInstance { misc: 0, instance: 2, id: CardId::Strike, upgrades: 0 },
        ],
        relics: vec![],
        potions: vec![None; 3],
        relic_counters: Default::default(),
    };
    let mut run = Run::prepare_complete_combat(
        CompleteCombatStart::Supplied(resources), Character::Ironclad, EncounterId::JawWorm,
        RunRng::new([99; 32], rng::Partition::Test, 0),
    ).unwrap();
    run.enter_training_combat(EncounterId::JawWorm).unwrap();
    run.state_mut_for_test().combat.as_mut().unwrap().state.enemies[0].hp = 6;
    run.choose_combat(CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    run.choose_combat(CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(run.state().last_combat.as_ref().unwrap().outcome, Outcome::Victory);
    let settled = run.state().potions.iter().flatten().copied().collect::<Vec<_>>();
    assert_eq!(settled.len(), 1, "victory returns the Alchemize potion to the Run belt");
    assert!(limited_class_pool(Character::Ironclad).contains(&settled[0]));
}

fn deus_result(character: Character) -> (usize, bool) {
    let mut combat = combat(character, &[(CardId::DeusExMachina, 0)], 3, 0, 1, true);
    end_turn(&mut combat);
    let miracles = count_card(&combat, CardId::Miracle);
    let exhausted = combat.state.piles.exhaust.iter().filter_map(model::OwnedCard::full)
        .any(|card| card.id == CardId::DeusExMachina && card.instance == CardInstanceId(1));
    (miracles, exhausted)
}

#[test]
fn m26_offcolour_deus_ex_machina_draw() {
    let ironclad = deus_result(Character::Ironclad);
    let watcher = deus_result(Character::Watcher);
    assert_eq!(watcher, (2, true), "Watcher draw adds two Miracles and exhausts Deus");
    assert_eq!(ironclad, watcher, "Ironclad off-colour draw matches Watcher");
}
