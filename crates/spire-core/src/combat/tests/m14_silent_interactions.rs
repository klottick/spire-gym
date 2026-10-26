use super::super::*;
use crate::{
    combat::model::OwnedCard,
    content::{CardId, Character, EncounterId, EnemyId, RelicId},
    items::CombatItemContext,
    run::{CreationProfile, Run, RunConfig, RunRng},
};
use super::m5_elites::hit;

fn live_cards(cards: &[(CardId, u32)]) -> Vec<LiveCard> {
    cards.iter().enumerate()
        .map(|(index, (id, upgrades))| {
            permanent_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades)
        })
        .collect()
}

fn actual_act3_entry(
    encounter: EncounterId,
    cards: &[(CardId, u32)],
    opening_draw: usize,
    energy: i32,
    draw_per_turn: i32,
) -> CombatEntry {
    let mut rng = RunRng::new([41; 32], rng::Partition::Test, 0);
    enemies::setup(encounter, 3, 0, &mut rng, enemies::SetupContext {
        player: PlayerState {
            hp: 80,
            max_hp: 80,
            block: 0,
            energy,
            max_energy: energy,
            draw_per_turn,
            powers: vec![],
        },
        player_powers: vec![],
        character: Character::Silent,
        cards: live_cards(cards),
        opening_draw,
        encounter_visit: 0,
    }).expect("actual A0 Act3 encounter setup")
}

fn actual_act3_combat(
    encounter: EncounterId,
    cards: &[(CardId, u32)],
    opening_draw: usize,
    energy: i32,
    draw_per_turn: i32,
) -> Combat {
    Combat::enter(actual_act3_entry(encounter, cards, opening_draw, energy, draw_per_turn))
        .expect("actual Act3 setup enters the shared combat owner")
}

fn combat_with_relics(
    encounter: EncounterId,
    cards: &[(CardId, u32)],
    opening_draw: usize,
    energy: i32,
    draw_per_turn: i32,
    relics: &[RelicId],
) -> Combat {
    let entry = actual_act3_entry(encounter, cards, opening_draw, energy, draw_per_turn);
    let mut run = Run::new(
        RunConfig { character: Character::Silent, ascension: 0 },
        CreationProfile::experienced(),
        RunRng::new([43; 32], rng::Partition::Test, 0),
    ).expect("Silent run owns the acquired item context");
    for relic in relics {
        crate::items::acquire_relic(run.state_mut_for_test(), *relic)
            .expect("the diagnostic relic is acquired through the shared item owner");
    }
    let mut items = Some(CombatItemContext::take_from_run(run.state_mut_for_test()));
    let combat = Combat::enter_with_item_context(entry, &mut items)
        .expect("actual Act3 setup and acquired relics enter together");
    assert!(items.is_none(), "the combat takes custody of its actual item context");
    combat
}

fn enemy_power(combat: &Combat, slot: usize, id: &'static str) -> i32 {
    combat.state.enemies[slot].full_powers.iter()
        .find(|power| power.id == FullPowerId(id))
        .map_or(0, |power| power.state.amount)
}

fn player_power(combat: &Combat, id: &'static str) -> i32 {
    combat.state.player_full_powers.iter()
        .find(|power| power.id == FullPowerId(id))
        .map_or(0, |power| power.state.amount)
}

fn hand_instance(combat: &Combat, id: CardId) -> u64 {
    combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == id)
        .unwrap_or_else(|| panic!("expected {id:?} in hand"))
        .instance.0
}

fn move_to_hand(combat: &mut Combat, instance: u64) {
    if combat.state.piles.hand.iter().any(|card| card.handle().as_str() == instance.to_string()) {
        return;
    }
    if let Some(index) = combat.state.piles.draw.iter()
        .position(|card| card.handle().as_str() == instance.to_string()) {
        combat.state.piles.hand.push(combat.state.piles.draw.remove(index));
    } else if let Some(index) = combat.state.piles.discard.iter()
        .position(|card| card.handle().as_str() == instance.to_string()) {
        combat.state.piles.hand.push(combat.state.piles.discard.remove(index));
    } else {
        panic!("card instance {instance} is in neither draw nor discard");
    }
}

fn play(combat: &mut Combat, instance: u64, target: Option<TargetId>) {
    combat.step_full(&CombatAction::PlayCard { instance, target })
        .expect("the selected actual Silent card resolves through the shared continuation");
}

fn select(combat: &mut Combat, instance: u64) {
    combat.step_full(&CombatAction::SelectCard { instance })
        .expect("the exact selected hand instance is accepted");
}

fn apply_enemy_power(combat: &mut Combat, slot: usize, id: &'static str, amount: i32) {
    let owner = Actor::Enemy(combat.state.enemies[slot].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player),
        target: Some(owner),
        kind: EffectKind::ApplyFullPower { id: FullPowerId(id), amount },
    }], None).expect("native shared resolver accepts the diagnostic power application");
}

fn set_move(combat: &mut Combat, slot: usize, name: &'static str) {
    let selected = ActorIntentId::Full(FullIntentId::from_name(name));
    let enemy = &mut combat.state.enemies[slot];
    enemy.intent_id = selected.clone();
    enemy.selected_move = selected.clone();
    enemy.move_history.push(selected);
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

fn enemy_move(combat: &mut Combat, slot: usize) {
    let owner = Actor::Enemy(combat.state.enemies[slot].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect {
        source: Source::None,
        target: Some(owner),
        kind: EffectKind::EnemyMove,
    }], None).expect("actual setup actor resolves its native selected move");
}

fn roll_move(combat: &mut Combat, slot: usize) {
    let owner = Actor::Enemy(combat.state.enemies[slot].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect {
        source: Source::None,
        target: Some(owner),
        kind: EffectKind::RollMove,
    }], None).expect("actual setup actor rolls through the native selector");
}

fn count_cards(combat: &Combat, id: CardId) -> usize {
    combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full)
        .filter(|card| card.id == id)
        .count()
}

fn source_hook_count(combat: &Combat, hook: Hook, instance: u64) -> usize {
    let source = Source::Card(card_state::handle(CardInstanceId(instance)));
    combat.state.hook_events.iter()
        .filter(|event| event.hook == hook && event.source == source)
        .count()
}

#[test]
fn m14_donu_deca_poison_artifact_and_mask() {
    let mut plain = combat_with_relics(
        EncounterId::DonuAndDeca,
        &[(CardId::DeadlyPoison, 0), (CardId::DefendGreen, 0)],
        2,
        5,
        5,
        &[RelicId::SneckoSkull],
    );
    assert_eq!(plain.state.enemies.iter().map(|enemy| enemy.enemy_id.clone()).collect::<Vec<_>>(),
        [ActorEnemyId::Full(EnemyId::Deca), ActorEnemyId::Full(EnemyId::Donu)]);
    assert_eq!((enemy_power(&plain, 0, "artifact"), enemy_power(&plain, 1, "artifact")), (2, 2),
        "the actual A0 pair each starts with Artifact 2");
    for slot in 0..2 {
        for remaining in [1, 0] {
            apply_enemy_power(&mut plain, slot, "poison", 1);
            assert_eq!(enemy_power(&plain, slot, "artifact"), remaining,
                "each blocked Poison application spends exactly one partner-owned charge");
            assert_eq!(enemy_power(&plain, slot, "poison"), 0,
                "Artifact prevents Poison from accumulating on either actual partner");
        }
    }
    let deadly_poison = hand_instance(&plain, CardId::DeadlyPoison);
    play(&mut plain, deadly_poison, Some(TargetId(1)));
    assert_eq!(enemy_power(&plain, 0, "poison"), 6,
        "the first unblocked Deadly Poison applies base five plus acquired Snecko Skull one");
    hit(&mut plain, 0, 10_000);
    assert_eq!(plain.state.outcome, None, "one partner's death does not settle while the other is live");
    enemy_move(&mut plain, 1);
    assert_eq!(enemy_power(&plain, 1, "strength"), 3,
        "the surviving Donu still completes its own live-partner action");
    assert_eq!(enemy_power(&plain, 0, "strength"), 0,
        "a settled Deca corpse does not receive Donu's partner work");
    assert_eq!(plain.state.outcome, None);
    hit(&mut plain, 1, 10_000);
    assert_eq!(plain.state.outcome, Some(Outcome::Victory));
    assert_eq!(plain.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 1,
        "both actual partners produce exactly one final settlement");

    let mut masked = combat_with_relics(
        EncounterId::DonuAndDeca,
        &[(CardId::DefendGreen, 0)],
        1,
        3,
        1,
        &[RelicId::RedMask],
    );
    assert!(masked.item_context().unwrap().owns_relic(RelicId::RedMask));
    for slot in 0..2 {
        assert_eq!(enemy_power(&masked, slot, "artifact"), 1,
            "actual Red Mask startup consumes one of each partner's initial Artifact charges");
        assert_eq!(enemy_power(&masked, slot, "weak"), 0,
            "Artifact blocks the startup Weak without removing the remaining charge");
        apply_enemy_power(&mut masked, slot, "poison", 1);
        assert_eq!(enemy_power(&masked, slot, "artifact"), 0);
        assert_eq!(enemy_power(&masked, slot, "poison"), 0,
            "a later Poison application consumes the remaining partner-owned Artifact");
    }
}

#[test]
fn m14_awakened_silent_powers_poison_void_and_rebirth() {
    let mut combat = actual_act3_combat(
        EncounterId::AwakenedOne,
        &[
            (CardId::NoxiousFumes, 0),
            (CardId::Accuracy, 0),
            (CardId::WellLaidPlans, 0),
            (CardId::DeadlyPoison, 0),
            (CardId::DefendGreen, 0),
        ],
        5,
        8,
        5,
    );
    assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.enemy_id.clone()).collect::<Vec<_>>(),
        [ActorEnemyId::Full(EnemyId::Cultist), ActorEnemyId::Full(EnemyId::Cultist),
            ActorEnemyId::Full(EnemyId::AwakenedOne)]);
    let boss = combat.state.enemies[2].instance_id.clone();

    let noxious_fumes = hand_instance(&combat, CardId::NoxiousFumes);
    play(&mut combat, noxious_fumes, None);
    assert_eq!((player_power(&combat, "noxious_fumes"), enemy_power(&combat, 2, "curiosity"),
        enemy_power(&combat, 2, "strength")), (2, 1, 1),
        "Curiosity stays at its actual A0 amount and adds Strength after Noxious Fumes");
    let accuracy = hand_instance(&combat, CardId::Accuracy);
    play(&mut combat, accuracy, None);
    assert_eq!((player_power(&combat, "accuracy"), enemy_power(&combat, 2, "curiosity"),
        enemy_power(&combat, 2, "strength")), (4, 1, 2),
        "Curiosity stays at its actual A0 amount and adds Strength after Accuracy");
    let plans = hand_instance(&combat, CardId::WellLaidPlans);
    play(&mut combat, plans, None);
    assert_eq!((player_power(&combat, "retain_cards"), enemy_power(&combat, 2, "curiosity"),
        enemy_power(&combat, 2, "strength")), (1, 1, 3),
        "Curiosity stays at its actual A0 amount and adds Strength after Well-Laid Plans");

    let poison = hand_instance(&combat, CardId::DeadlyPoison);
    play(&mut combat, poison, Some(TargetId(3)));
    assert_eq!(enemy_power(&combat, 2, "poison"), 5,
        "the actual Silent Deadly Poison card leaves a live Poison stack before collapse");
    hit(&mut combat, 2, 300);
    assert_eq!((combat.state.enemies[2].hp, combat.state.enemies[2].life, combat.state.outcome),
        (0, ActorLife::HalfDead, None), "the first phase is not a Victory or final settlement");
    assert_eq!(combat.state.enemies[2].instance_id, boss, "rebirth retains the same native actor owner");
    assert_eq!((enemy_power(&combat, 2, "poison"), enemy_power(&combat, 2, "curiosity"),
        enemy_power(&combat, 2, "strength")), (0, 0, 3),
        "collapse clears Poison and Curiosity while preserving positive Strength");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 0);

    combat.step_full(&CombatAction::EndTurn).expect("the shared end-turn pipeline advances phase two");
    if combat.state.phase == Phase::Selection {
        let retained = hand_instance(&combat, CardId::DefendGreen);
        select(&mut combat, retained);
        combat.step_full(&CombatAction::ConfirmSelection)
            .expect("Well-Laid Plans completes its actual retain choice before the enemy turn");
    }
    assert_eq!((combat.state.enemies[2].hp, combat.state.enemies[2].instance_id.clone(),
        combat.state.enemies[2].life), (300, boss, ActorLife::Active));
    assert_eq!((enemy_power(&combat, 2, "curiosity"), enemy_power(&combat, 2, "strength")), (0, 3));
    hit(&mut combat, 2, 300);
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert!(combat.state.enemies[..2].iter().all(|enemy| enemy.life == ActorLife::Escaped),
        "surviving actual Cultists use the shared final-escape owner");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 1);

    let mut sludge = actual_act3_combat(
        EncounterId::AwakenedOne,
        &[(CardId::DefendGreen, 0)],
        1,
        3,
        1,
    );
    set_move(&mut sludge, 2, "awa_sludge");
    enemy_move(&mut sludge, 2);
    assert_eq!(sludge.state.piles.draw.first().and_then(OwnedCard::full).map(|card| card.id),
        Some(CardId::Void), "Sludge creates Void at BottomDraw; there is no Sludge CardId");
    assert!(sludge.permanent_gains().is_empty(), "generated Void is not a permanent deck gain");
    sludge.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player),
        target: Some(Actor::Player),
        kind: EffectKind::Draw(1),
    }], None).expect("the shared actual Draw owner resolves the generated Void");
    assert_eq!(sludge.state.player.energy, 2, "drawing Void loses one available energy through its draw hook");
    assert!(sludge.state.piles.hand.iter().filter_map(OwnedCard::full).any(|card| card.id == CardId::Void));
}

fn time_combat(cards: &[(CardId, u32)]) -> Combat {
    actual_act3_combat(EncounterId::TimeEater, cards, 0, 10, 8)
}

fn play_deflects(combat: &mut Combat, instances: std::ops::RangeInclusive<u64>) {
    for instance in instances {
        move_to_hand(combat, instance);
        play(combat, instance, None);
    }
}

#[test]
fn m14_time_warp_finishes_shiv_discard_nightmare_and_retain() {
    let mut shiv_cards = vec![(CardId::Deflect, 0); 10];
    shiv_cards.push((CardId::BladeDance, 0));
    let mut shiv = time_combat(&shiv_cards);
    play_deflects(&mut shiv, 1..=10);
    move_to_hand(&mut shiv, 11);
    play(&mut shiv, 11, None);
    assert_eq!(enemy_power(&shiv, 0, "time_warp"), 11,
        "eleven actual Silent card plays precede the generated Shiv");
    let shiv_instance = shiv.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == CardId::Shiv).expect("actual Blade Dance generated a Shiv").instance.0;
    play(&mut shiv, shiv_instance, Some(TargetId(1)));
    assert_eq!((enemy_power(&shiv, 0, "time_warp"), enemy_power(&shiv, 0, "strength"), shiv.state.turn),
        (0, 2, 2), "the actual twelfth Shiv resolves before one ordinary forced end turn");
    assert_eq!(shiv.state.hook_events.iter().filter(|event| event.hook == Hook::TurnEnded).count(), 1);
    assert!(shiv.state.piles.exhaust.iter().any(|card| card.handle().as_str() == shiv_instance.to_string()),
        "the generated Shiv keeps its ordinary Exhaust disposition after the barrier");

    for (listener, upgrades, expected_gain) in [
        (CardId::Reflex, 0, 0),
        (CardId::Tactician, 0, 1),
        (CardId::Tactician, 1, 2),
    ] {
        let mut cards = vec![(CardId::Deflect, 0); 11];
        cards.extend([(CardId::DaggerThrow, 0), (listener, upgrades),
            (CardId::DefendGreen, 0), (CardId::DefendGreen, 0), (CardId::DefendGreen, 0)]);
        let mut combat = time_combat(&cards);
        play_deflects(&mut combat, 1..=11);
        let dagger_throw = 12;
        let listener_instance = 13;
        move_to_hand(&mut combat, dagger_throw);
        move_to_hand(&mut combat, listener_instance);
        play(&mut combat, dagger_throw, Some(TargetId(1)));
        assert_eq!(combat.state.phase, Phase::Selection,
            "Dagger Throw remains the current twelfth-card owner while its discard choice is pending");
        assert_eq!(enemy_power(&combat, 0, "time_warp"), 11,
            "Time Warp does not start before the actual selected discard finishes");
        let selected = combat.step_full_observed(&CombatAction::SelectCard { instance: listener_instance });
        selected.result.expect("the actual Reflex or Tactician instance is selected");
        assert_eq!((enemy_power(&combat, 0, "time_warp"), enemy_power(&combat, 0, "strength"), combat.state.turn),
            (0, 2, 2));
        assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::TurnEnded).count(), 1,
            "the forced turn starts only after the owned listener completes");
        assert_eq!(source_hook_count(&combat, Hook::CardDiscarded, listener_instance), 1);
        if listener == CardId::Reflex {
            assert_eq!(source_hook_count(&combat, Hook::CardDrawn, listener_instance), 2,
                "the actual selected base Reflex itself draws two cards");
        } else {
            assert!(selected.effects.iter().any(|event| event.source == Source::Card(
                card_state::handle(CardInstanceId(listener_instance)))
                && matches!(event.kind, ResolvedEffectKind::Energy { before, after }
                    if after - before == expected_gain)),
                "the selected base/upgraded Tactician itself grants its exact energy payload");
        }
    }

    let mut nightmare_cards = vec![(CardId::Deflect, 0); 10];
    nightmare_cards.extend([
        (CardId::WellLaidPlans, 0),
        (CardId::Nightmare, 0),
        (CardId::StrikeGreen, 0),
    ]);
    let mut nightmare = time_combat(&nightmare_cards);
    play_deflects(&mut nightmare, 1..=10);
    move_to_hand(&mut nightmare, 11);
    play(&mut nightmare, 11, None);
    assert_eq!(player_power(&nightmare, "retain_cards"), 1);
    assert_eq!(enemy_power(&nightmare, 0, "time_warp"), 11);
    move_to_hand(&mut nightmare, 12);
    move_to_hand(&mut nightmare, 13);
    play(&mut nightmare, 12, None);
    assert_eq!(nightmare.state.phase, Phase::Selection);
    assert_eq!(enemy_power(&nightmare, 0, "time_warp"), 11,
        "Nightmare's selected copy target remains inside card twelve's continuation");
    select(&mut nightmare, 13);
    assert_eq!(nightmare.state.phase, Phase::Selection,
        "Well-Laid Plans opens its actual retain choice only after Nightmare's selection completes");
    assert_eq!((enemy_power(&nightmare, 0, "time_warp"), enemy_power(&nightmare, 0, "strength")), (0, 2),
        "Time Warp resets and grants Strength before its own end-turn retain selector");
    assert!(nightmare.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(13))));
    select(&mut nightmare, 13);
    nightmare.step_full(&CombatAction::ConfirmSelection)
        .expect("the exact original card is retained through the shared forced-turn pipeline");
    assert_eq!((enemy_power(&nightmare, 0, "time_warp"), enemy_power(&nightmare, 0, "strength"), nightmare.state.turn),
        (0, 2, 2));
    assert_eq!(nightmare.state.hook_events.iter().filter(|event| event.hook == Hook::TurnEnded).count(), 1);
    let strikes = nightmare.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::StrikeGreen).collect::<Vec<_>>();
    assert_eq!(strikes.len(), 4, "Well-Laid Plans retains the original while Nightmare delivers three copies");
    assert!(strikes.iter().any(|card| card.instance.0 == 13
        && card.origin == CombatCardRef::Permanent(CardInstanceId(13))));
    let copy_ids = strikes.iter().filter(|card| card.instance.0 != 13)
        .map(|card| card.instance).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(copy_ids.len(), 3, "the delayed Nightmare children have distinct temporary identities");
    assert!(nightmare.state.piles.exhaust.iter().any(|card| card.handle().as_str() == "12"));
    assert!(nightmare.permanent_gains().is_empty(), "Nightmare copy delivery is not a permanent deck gain");

    let mut haste = actual_act3_combat(
        EncounterId::TimeEater,
        &[(CardId::Deflect, 0)],
        1,
        3,
        1,
    );
    hit(&mut haste, 0, 228);
    roll_move(&mut haste, 0);
    assert_ne!(haste.state.enemies[0].intent_id.as_str(), "time_haste",
        "Time Eater does not select Haste at exactly half HP");
    hit(&mut haste, 0, 1);
    apply_enemy_power(&mut haste, 0, "poison", 3);
    roll_move(&mut haste, 0);
    assert_eq!(haste.state.enemies[0].intent_id.as_str(), "time_haste",
        "the actual selector admits Haste after crossing strictly below half HP");
    enemy_move(&mut haste, 0);
    assert_eq!(enemy_power(&haste, 0, "poison"), 0, "Haste clears the existing Poison stack once");
    assert!(matches!(haste.state.enemies[0].full_memory,
        Some(EnemyMemory::TimeEater { haste_used: true })));
    apply_enemy_power(&mut haste, 0, "poison", 2);
    roll_move(&mut haste, 0);
    assert_ne!(haste.state.enemies[0].intent_id.as_str(), "time_haste",
        "a second low-HP selection cannot repeat the consumed Haste cleanse");
}

fn reptomancer_with_silent_cards(tactician_upgrades: u32) -> Combat {
    let mut combat = actual_act3_combat(
        EncounterId::Reptomancer,
        &[
            (CardId::Survivor, 0),
            (CardId::Reflex, 0),
            (CardId::Tactician, tactician_upgrades),
            (CardId::DefendGreen, 0),
            (CardId::StrikeGreen, 0),
            (CardId::DefendGreen, 0),
        ],
        0,
        8,
        5,
    );
    for instance in 1..=4 {
        move_to_hand(&mut combat, instance);
    }
    assert_eq!(combat.state.enemies.iter().map(|enemy| enemy.enemy_id.clone()).collect::<Vec<_>>(),
        [ActorEnemyId::Full(EnemyId::Dagger), ActorEnemyId::Full(EnemyId::Reptomancer),
            ActorEnemyId::Full(EnemyId::Dagger)]);
    assert_eq!(combat.state.enemies[1].intent_id.as_str(), "repto_spawn");
    enemy_move(&mut combat, 1);
    assert_eq!(combat.state.enemies.len(), 4, "the actual first Reptomancer move summons a Dagger");
    assert_eq!(combat.state.enemies[3].enemy_id, ActorEnemyId::Full(EnemyId::Dagger));
    for slot in [0, 2] {
        assert_eq!(combat.state.enemies[slot].intent_id.as_str(), "dagger_wound");
        enemy_move(&mut combat, slot);
    }
    assert_eq!(count_cards(&combat, CardId::Wound), 2,
        "both original actual Daggers create one Wound in their real first move");
    combat
}

#[test]
fn m14_elites_silent_shiv_poison_and_generated_statuses() {
    let mut nemesis = actual_act3_combat(
        EncounterId::Nemesis,
        &[(CardId::DeadlyPoison, 0), (CardId::BladeDance, 0), (CardId::DefendGreen, 0)],
        3,
        6,
        3,
    );
    assert_eq!(nemesis.state.enemies[0].enemy_id, ActorEnemyId::Full(EnemyId::Nemesis));
    set_move(&mut nemesis, 0, "nemesis_burn");
    enemy_move(&mut nemesis, 0);
    assert_eq!(enemy_power(&nemesis, 0, "intangible"), 1,
        "the actual Nemesis action installs its once-per-turn Intangible");
    assert_eq!(count_cards(&nemesis, CardId::Burn), 3,
        "the same actual setup move creates its three real Burn statuses");
    let deadly_poison = hand_instance(&nemesis, CardId::DeadlyPoison);
    play(&mut nemesis, deadly_poison, Some(TargetId(1)));
    assert_eq!(enemy_power(&nemesis, 0, "poison"), 5,
        "Intangible does not block the Poison application itself");
    let blade_dance = hand_instance(&nemesis, CardId::BladeDance);
    play(&mut nemesis, blade_dance, None);
    let shivs = nemesis.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Shiv).map(|card| card.instance.0).collect::<Vec<_>>();
    assert_eq!(shivs.len(), 3);
    for shiv in shivs {
        let before = nemesis.state.enemies[0].hp;
        play(&mut nemesis, shiv, Some(TargetId(1)));
        assert_eq!(nemesis.state.enemies[0].hp, before - 1,
            "each positive actual Shiv hit is capped at one while Nemesis Intangible is active");
    }
    let before_poison_tick = nemesis.state.enemies[0].hp;
    nemesis.step_full(&CombatAction::EndTurn).expect("the ordinary turn owner executes the actual Poison tick");
    assert_eq!(nemesis.state.enemies[0].hp, before_poison_tick - 1,
        "a Poison tick is also capped at one while Intangible is active");
    assert_eq!(enemy_power(&nemesis, 0, "poison"), 4,
        "the capped Poison tick still decrements the captured stack once");

    let mut giant_head = actual_act3_combat(
        EncounterId::GiantHead,
        &[
            (CardId::StrikeGreen, 0),
            (CardId::DeadlyPoison, 0),
            (CardId::BladeDance, 0),
            (CardId::StrikeGreen, 0),
        ],
        4,
        8,
        4,
    );
    assert_eq!(giant_head.state.enemies[0].enemy_id, ActorEnemyId::Full(EnemyId::GiantHead));
    assert_eq!(enemy_power(&giant_head, 0, "slow"), 0);
    let hp_before = giant_head.state.enemies[0].hp;
    play(&mut giant_head, 1, Some(TargetId(1)));
    let baseline_hit = hp_before - giant_head.state.enemies[0].hp;
    assert_eq!(enemy_power(&giant_head, 0, "slow"), 1,
        "the actual first normal card increments Slow after its own payload");
    play(&mut giant_head, 2, Some(TargetId(1)));
    assert_eq!(enemy_power(&giant_head, 0, "poison"), 5);
    play(&mut giant_head, 3, None);
    assert_eq!(enemy_power(&giant_head, 0, "slow"), 3,
        "Blade Dance itself counts as a card play after its payload");
    let shivs = giant_head.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Shiv).map(|card| card.instance.0).collect::<Vec<_>>();
    for (index, shiv) in shivs.into_iter().enumerate() {
        let before = giant_head.state.enemies[0].hp;
        play(&mut giant_head, shiv, Some(TargetId(1)));
        assert_eq!(enemy_power(&giant_head, 0, "slow"), 4 + index as i32,
            "each generated Shiv increments Giant Head Slow once after its payload");
        assert!(giant_head.state.enemies[0].hp < before);
    }
    let hp_before_slowed_strike = giant_head.state.enemies[0].hp;
    play(&mut giant_head, 4, Some(TargetId(1)));
    let slowed_hit = hp_before_slowed_strike - giant_head.state.enemies[0].hp;
    assert!(slowed_hit > baseline_hit,
        "Giant Head Slow multiplies later NORMAL card damage, including after generated-card plays");
    let before_poison_tick = giant_head.state.enemies[0].hp;
    giant_head.step_full(&CombatAction::EndTurn).expect("Giant Head Poison ticks through the normal turn hook");
    assert_eq!(giant_head.state.enemies[0].hp, before_poison_tick - 5,
        "Slow does not scale Poison HP loss");
    assert_eq!(enemy_power(&giant_head, 0, "poison"), 4);

    let mut orb = actual_act3_combat(
        EncounterId::OrbWalker, &[(CardId::DefendGreen, 0)], 1, 3, 1,
    );
    set_move(&mut orb, 0, "orb_laser");
    orb.step_full(&CombatAction::EndTurn).expect("actual Orb Walker executes its selected laser");
    assert_eq!(count_cards(&orb, CardId::Burn), 2,
        "the actual Orb Walker laser creates one shuffled-draw and one discard Burn");
    assert!(orb.state.piles.all(orb.state.in_play.as_ref()).filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Burn)
        .all(|card| matches!(card.origin, CombatCardRef::CombatLocal(_))));

    let shape_entry = actual_act3_entry(
        EncounterId::FourShapes, &[(CardId::DefendGreen, 0)], 1, 3, 1,
    );
    let repulsor_slot = shape_entry.setup.enemies.iter()
        .position(|enemy| enemy.enemy_id == EnemyId::Repulsor)
        .expect("the fixed actual A0 Four Shapes setup includes a captured Repulsor");
    let mut shapes = Combat::enter(shape_entry).expect("the actual Four Shapes lineup enters");
    set_move(&mut shapes, repulsor_slot, "repulsor_daze");
    enemy_move(&mut shapes, repulsor_slot);
    assert_eq!(count_cards(&shapes, CardId::Dazed), 2,
        "the actual Repulsor move creates two generated Dazed cards");
    assert!(shapes.state.piles.all(shapes.state.in_play.as_ref()).filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Dazed)
        .all(|card| matches!(card.origin, CombatCardRef::CombatLocal(_))));

    let mut pair = actual_act3_combat(
        EncounterId::DonuAndDeca, &[(CardId::DefendGreen, 0)], 1, 3, 1,
    );
    assert_eq!(pair.state.enemies[0].intent_id.as_str(), "deca_beam");
    enemy_move(&mut pair, 0);
    assert_eq!(count_cards(&pair, CardId::Dazed), 2,
        "the actual Deca beam creates two generated Dazed cards in its discard destination");

    let mut wound = reptomancer_with_silent_cards(0);
    let wound_instance = wound.state.piles.discard.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == CardId::Wound).expect("actual Daggers generated a Wound").instance.0;
    move_to_hand(&mut wound, wound_instance);
    let survivor = hand_instance(&wound, CardId::Survivor);
    let reflex = hand_instance(&wound, CardId::Reflex);
    let tactician = hand_instance(&wound, CardId::Tactician);
    play(&mut wound, survivor, None);
    select(&mut wound, wound_instance);
    assert_eq!(wound.state.piles.discard.iter()
        .filter(|card| card.handle().as_str() == wound_instance.to_string()).count(), 1,
        "Survivor moves the exact generated Wound once");
    assert_eq!(count_cards(&wound, CardId::Wound), 2,
        "the selected Wound returns to discard without creating another instance");
    assert_eq!(wound.state.player.energy, 7,
        "discarding the actual Wound grants no Tactician energy");
    assert!(wound.state.piles.hand.iter().any(|card| card.handle().as_str() == reflex.to_string()));
    assert!(wound.state.piles.hand.iter().any(|card| card.handle().as_str() == tactician.to_string()));
    assert_eq!(source_hook_count(&wound, Hook::CardDrawn, reflex), 0,
        "discarding Wound does not impersonate a Reflex listener");
    assert_eq!(source_hook_count(&wound, Hook::CardDiscarded, tactician), 0,
        "discarding Wound does not impersonate a Tactician listener");

    for (listener, upgrades, expected_gain) in [
        (CardId::Reflex, 0, None),
        (CardId::Tactician, 0, Some(1)),
        (CardId::Tactician, 1, Some(2)),
    ] {
        let mut combat = reptomancer_with_silent_cards(upgrades);
        let listener_instance = if listener == CardId::Reflex { 2 } else { 3 };
        play(&mut combat, 1, None);
        let observed = combat.step_full_observed(&CombatAction::SelectCard { instance: listener_instance });
        observed.result.expect("the actual selected discard listener resolves");
        assert_eq!(source_hook_count(&combat, Hook::CardDiscarded, listener_instance), 1);
        if listener == CardId::Reflex {
            assert_eq!(source_hook_count(&combat, Hook::CardDrawn, listener_instance), 2,
                "selecting the actual base Reflex itself draws two cards");
        } else {
            assert!(observed.effects.iter().any(|event| event.source == Source::Card(
                card_state::handle(CardInstanceId(listener_instance)))
                && matches!(event.kind, ResolvedEffectKind::Energy { before, after }
                    if Some(after - before) == expected_gain)),
                "the selected actual base/upgraded Tactician itself grants one or two energy");
        }
    }
}

#[test]
fn m14_red_mask_startup_and_temporary_parasite_copy_keep_ownership() {
    let mut masked_repto = combat_with_relics(
        EncounterId::Reptomancer,
        &[(CardId::DefendGreen, 0)],
        1,
        3,
        1,
        &[RelicId::RedMask],
    );
    assert_eq!(masked_repto.state.enemies.iter().map(|enemy| enemy.enemy_id.clone()).collect::<Vec<_>>(),
        [ActorEnemyId::Full(EnemyId::Dagger), ActorEnemyId::Full(EnemyId::Reptomancer),
            ActorEnemyId::Full(EnemyId::Dagger)]);
    for slot in 0..masked_repto.state.enemies.len() {
        assert_eq!(enemy_power(&masked_repto, slot, "weak"), 1,
            "actual Red Mask startup applies Weak to every initial actor");
    }
    assert_eq!(masked_repto.state.enemies[1].intent_id.as_str(), "repto_spawn");
    enemy_move(&mut masked_repto, 1);
    let summoned_slot = masked_repto.state.enemies.len() - 1;
    assert_eq!(masked_repto.state.enemies[summoned_slot].enemy_id, ActorEnemyId::Full(EnemyId::Dagger));
    assert_eq!(enemy_power(&masked_repto, summoned_slot, "weak"), 0,
        "Red Mask's actual opening effect does not apply again to a later Dagger summon");
    assert!(masked_repto.state.enemies[..3].iter().enumerate()
        .all(|(slot, _)| enemy_power(&masked_repto, slot, "weak") == 1));

    let mut parasite = actual_act3_combat(
        EncounterId::TimeEater,
        &[(CardId::Nightmare, 0), (CardId::Parasite, 0), (CardId::DefendGreen, 0)],
        3,
        5,
        3,
    );
    let max_hp = parasite.state.player.max_hp;
    let nightmare = hand_instance(&parasite, CardId::Nightmare);
    let original = hand_instance(&parasite, CardId::Parasite);
    assert_eq!(parasite.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == original).unwrap().origin,
        CombatCardRef::Permanent(CardInstanceId(original)));
    play(&mut parasite, nightmare, None);
    select(&mut parasite, original);
    assert!(parasite.permanent_gains().is_empty(), "Nightmare captures no permanent Parasite gain");
    parasite.step_full(&CombatAction::EndTurn)
        .expect("the shared next-player-start owner delivers temporary copies");
    let copies = parasite.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Parasite && card.instance.0 != original)
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 3, "Nightmare delivers three distinct temporary Parasite copies");
    assert!(copies.iter().all(|card| matches!(card.origin, CombatCardRef::CombatLocal(_))));
    assert_eq!(parasite.state.player.max_hp, max_hp,
        "temporary copied Curse does not invoke permanent Parasite purge or maximum-HP loss");
    assert!(parasite.permanent_gains().is_empty(), "temporary copies never enter the permanent-gain journal");
    let copy_ids = copies.iter().map(|card| card.instance.0).collect::<Vec<_>>();
    parasite.step_full(&CombatAction::EndTurn)
        .expect("unplayable temporary Parasites take ordinary discard disposition");
    assert!(copy_ids.iter().all(|instance|
        source_hook_count(&parasite, Hook::CardDiscarded, *instance) == 1),
        "each temporary copy takes its ordinary discard disposition exactly once");
    assert!(parasite.state.piles.all(parasite.state.in_play.as_ref()).filter_map(OwnedCard::full)
        .any(|card| card.instance.0 == original
            && card.origin == CombatCardRef::Permanent(CardInstanceId(original))),
        "the original Parasite retains its permanent identity after ordinary discard/draw work");
    assert_eq!(parasite.state.player.max_hp, max_hp);
    assert!(parasite.permanent_gains().is_empty());
}
