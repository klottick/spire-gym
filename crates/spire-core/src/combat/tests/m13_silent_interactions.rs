use super::super::*;
use crate::{
    combat::model::OwnedCard,
    content::{CardId, Character, EncounterId, EnemyId, RelicId},
    items::CombatItemContext,
};

fn live_cards(cards: &[(CardId, u32)]) -> Vec<LiveCard> {
    cards.iter().enumerate()
        .map(|(index, (id, upgrades))| permanent_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades))
        .collect()
}

fn actual_act2_entry(
    encounter: EncounterId,
    cards: Vec<LiveCard>,
    opening_draw: usize,
    energy: i32,
    draw_per_turn: i32,
) -> CombatEntry {
    let mut rng = crate::run::RunRng::new([41; 32], rng::Partition::Test, 0);
    enemies::setup(encounter, 2, 0, &mut rng, enemies::SetupContext {
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
        cards,
        opening_draw,
        encounter_visit: 0,
    }).expect("actual A0 Act2 encounter setup")
}

fn actual_act2_combat(
    encounter: EncounterId,
    cards: &[(CardId, u32)],
    opening_draw: usize,
    energy: i32,
    draw_per_turn: i32,
) -> Combat {
    Combat::enter(actual_act2_entry(
        encounter, live_cards(cards), opening_draw, energy, draw_per_turn,
    )).expect("actual Act2 encounter enters through the shared combat owner")
}

fn power_amount(combat: &Combat, slot: usize, id: &'static str) -> Option<i32> {
    combat.state.enemies[slot].full_powers.iter()
        .find(|power| power.id == FullPowerId(id))
        .map(|power| power.state.amount)
}

fn hand_instance(combat: &Combat, id: CardId) -> u64 {
    combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == id)
        .expect("requested card is in hand")
        .instance.0
}

fn hand_card_by_instance(combat: &Combat, instance: u64) -> Option<&LiveCard> {
    combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == instance)
}

fn play(combat: &mut Combat, instance: u64, target: Option<TargetId>) {
    combat.step_full(&CombatAction::PlayCard { instance, target })
        .expect("selected card resolves through the shared combat continuation");
}

fn select(combat: &mut Combat, instance: u64) {
    combat.step_full(&CombatAction::SelectCard { instance })
        .expect("the exact selected hand instance is accepted");
}

fn discard_count(combat: &Combat, instance: u64) -> usize {
    combat.state.piles.discard.iter().filter(|card| card.handle().as_str() == instance.to_string()).count()
}

fn discarded_hook_count(combat: &Combat, instance: u64) -> usize {
    let source = Source::Card(card_state::handle(CardInstanceId(instance)));
    combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardDiscarded && event.source == source)
        .count()
}

fn append_draw_cards(combat: &mut Combat, cards: &[CardId], first_instance: u64) {
    for (offset, id) in cards.iter().copied().enumerate() {
        let card = permanent_live_card(id, CardInstanceId(first_instance + offset as u64), 0);
        assert!(combat.state.card_ids_issued.insert(card_state::handle(card.instance)),
            "the diagnostic draw card has a unique issued identity");
        combat.state.piles.draw.push(OwnedCard::Full(card));
    }
}

fn slavers_turn_with_real_wound() -> Combat {
    let cards = [
        (CardId::Survivor, 0),
        (CardId::DaggerThrow, 0),
        (CardId::Reflex, 0),
        (CardId::Tactician, 0),
        (CardId::DefendGreen, 0),
        (CardId::StrikeGreen, 0),
    ];
    let mut combat = actual_act2_combat(EncounterId::Slavers, &cards, cards.len(), 3, 10);
    let taskmaster = combat.state.enemies.iter()
        .find(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::Taskmaster))
        .expect("actual Slavers setup contains Taskmaster");
    assert_eq!(taskmaster.intent_id.as_str(), "taskmaster_whip",
        "Taskmaster's actual selected first turn creates its Wound");
    combat.step_full(&CombatAction::EndTurn)
        .expect("actual Slavers take their setup-selected first turn");
    assert_eq!(combat.state.turn, 2);
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Wound).count(), 1,
        "the Taskmaster-created Wound is drawn by the ordinary next-turn draw");
    combat
}

// Removing either the actual setup Artifact charge or the shared poison/relic
// application path must break this assertion for each captured Act2 Artifact enemy.
#[test]
fn m13_poison_and_snecko_skull_resolve_against_act2_artifact() {
    for encounter in [EncounterId::SphericGuardian, EncounterId::BronzeAutomaton] {
        let deck = vec![(CardId::DeadlyPoison, 0); 4];
        let entry = actual_act2_entry(encounter, live_cards(&deck), deck.len(), 4, 5);
        let mut items = Some(CombatItemContext::new(vec![RelicId::SneckoSkull], vec![]));
        let mut combat = Combat::enter_with_item_context(entry, &mut items)
            .expect("actual setup and Snecko Skull enter the shared item owner");
        assert!(items.is_none(), "the active combat takes custody of the relic context");
        assert_eq!(power_amount(&combat, 0, "artifact"), Some(3),
            "the selected actual Act2 setup supplies Artifact 3");

        for (instance, remaining) in [(1, Some(2)), (2, Some(1)), (3, None)] {
            play(&mut combat, instance, Some(TargetId(1)));
            assert_eq!(power_amount(&combat, 0, "artifact"), remaining,
                "each blocked Poison application consumes one real Artifact charge");
            assert_eq!(power_amount(&combat, 0, "poison"), None,
                "Artifact blocks each of the first three Poison applications");
        }

        play(&mut combat, 4, Some(TargetId(1)));
        assert_eq!(power_amount(&combat, 0, "artifact"), None);
        assert_eq!(power_amount(&combat, 0, "poison"), Some(6),
            "the first unblocked base Deadly Poison applies five plus Snecko Skull's one");
    }
}

// A missing Accuracy modifier, wrong Shiv producer, or Flight charged once per
// card instead of once per hit changes the observed HP/Flight sequence.
#[test]
fn m13_accuracy_shivs_count_actual_byrd_flight_hits() {
    let mut combat = actual_act2_combat(
        EncounterId::ThreeByrds,
        &[(CardId::Accuracy, 0), (CardId::BladeDance, 0), (CardId::CloakAndDagger, 0)],
        3,
        3,
        5,
    );
    assert_eq!(combat.state.enemies.len(), 3);
    for enemy in &combat.state.enemies {
        assert_eq!(enemy.enemy_id, ActorEnemyId::Full(EnemyId::Byrd));
        assert_eq!(enemy.full_powers.iter().find(|power| power.id == FullPowerId("flight"))
            .map(|power| power.state.amount), Some(3),
            "all three Byrds carry Flight from the real Act2 setup");
    }
    let first_hp = combat.state.enemies[0].hp;
    let second_hp = combat.state.enemies[1].hp;

    play(&mut combat, 1, None);
    assert_eq!(combat.state.player_full_powers.iter()
        .find(|power| power.id == FullPowerId("accuracy")).map(|power| power.state.amount), Some(4));
    play(&mut combat, 2, None);
    let mut blade_dance_shivs = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::Shiv)
        .map(|card| card.instance.0).collect::<Vec<_>>();
    blade_dance_shivs.sort_unstable();
    assert_eq!(blade_dance_shivs.len(), 3,
        "Blade Dance produces three separate unupgraded Shivs");
    for (instance, flight_after) in blade_dance_shivs.into_iter().zip([2, 1, 0]) {
        play(&mut combat, instance, Some(TargetId(1)));
        assert_eq!(combat.state.enemies[0].hp, first_hp - 4 *
            (3 - flight_after), "eight pre-mitigation damage costs a flying Byrd four HP");
        assert_eq!(power_amount(&combat, 0, "flight"), (flight_after > 0).then_some(flight_after));
    }

    play(&mut combat, 3, None);
    let cloak_shiv = hand_instance(&combat, CardId::Shiv);
    play(&mut combat, cloak_shiv, Some(TargetId(2)));
    assert_eq!(combat.state.enemies[1].hp, second_hp - 4);
    assert_eq!(power_amount(&combat, 1, "flight"), Some(2),
        "Cloak and Dagger's Accuracy-buffed Shiv also removes exactly one Flight");
    let accuracy_loss = second_hp - combat.state.enemies[1].hp;
    assert!(combat.permanent_gains().is_empty(),
        "generated Shivs are temporary combat instances");

    let mut unbuffed = actual_act2_combat(
        EncounterId::ThreeByrds,
        &[(CardId::CloakAndDagger, 0)],
        1,
        3,
        5,
    );
    let unbuffed_hp = unbuffed.state.enemies[0].hp;
    play(&mut unbuffed, 1, None);
    let unbuffed_shiv = hand_instance(&unbuffed, CardId::Shiv);
    play(&mut unbuffed, unbuffed_shiv, Some(TargetId(1)));
    assert_eq!(unbuffed.state.enemies[0].hp, unbuffed_hp - 2);
    assert_eq!(power_amount(&unbuffed, 0, "flight"), Some(2));
    let unbuffed_loss = unbuffed_hp - unbuffed.state.enemies[0].hp;
    assert_eq!(accuracy_loss - unbuffed_loss, 2,
        "base Accuracy adds two HP loss over an unbuffed Shiv while Flight remains positive");
}

// Wounds must come from the actual Taskmaster turn; moving one must not run the
// Reflex/Tactician listeners, while discarding each listener itself runs once.
#[test]
fn m13_discard_reflex_and_tactician_resolve_actual_act2_wounds() {
    let mut survivor_combat = slavers_turn_with_real_wound();
    let wound = hand_instance(&survivor_combat, CardId::Wound);
    let survivor = hand_instance(&survivor_combat, CardId::Survivor);
    assert!(hand_card_by_instance(&survivor_combat, hand_instance(&survivor_combat, CardId::Reflex)).is_some());
    assert!(hand_card_by_instance(&survivor_combat, hand_instance(&survivor_combat, CardId::Tactician)).is_some());
    let draw_events_before = survivor_combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardDrawn).count();
    play(&mut survivor_combat, survivor, None);
    assert_eq!(survivor_combat.state.player.energy, 2,
        "Survivor pays its one-energy cost before the discard selector");
    assert!(survivor_combat.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(wound))));
    select(&mut survivor_combat, wound);
    assert_eq!(discard_count(&survivor_combat, wound), 1);
    assert!(hand_card_by_instance(&survivor_combat, wound).is_none());
    assert_eq!(survivor_combat.state.player.energy, 2,
        "discarding the actual Wound grants no Tactician energy");
    assert_eq!(survivor_combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardDrawn).count(), draw_events_before,
        "discarding the actual Wound draws no Reflex cards");

    let mut dagger_combat = slavers_turn_with_real_wound();
    let wound = hand_instance(&dagger_combat, CardId::Wound);
    let dagger_throw = hand_instance(&dagger_combat, CardId::DaggerThrow);
    play(&mut dagger_combat, dagger_throw, Some(TargetId(1)));
    assert_eq!(dagger_combat.state.player.energy, 2,
        "Dagger Throw pays its one-energy cost before its discard selector");
    assert!(dagger_combat.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(wound))));
    select(&mut dagger_combat, wound);
    assert_eq!(discard_count(&dagger_combat, wound), 1);
    assert!(hand_card_by_instance(&dagger_combat, wound).is_none());

    let mut reflex_combat = actual_act2_combat(
        EncounterId::Slavers,
        &[(CardId::DaggerThrow, 0), (CardId::Reflex, 0), (CardId::DefendGreen, 0)],
        3,
        3,
        5,
    );
    append_draw_cards(&mut reflex_combat, &[CardId::StrikeGreen; 3], 100);
    let dagger_throw = hand_instance(&reflex_combat, CardId::DaggerThrow);
    let reflex = hand_instance(&reflex_combat, CardId::Reflex);
    let draw_events_before_parent = reflex_combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardDrawn).count();
    play(&mut reflex_combat, dagger_throw, Some(TargetId(1)));
    assert_eq!(reflex_combat.state.player.energy, 2);
    let draw_events_after_parent = reflex_combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardDrawn).count();
    assert_eq!(draw_events_after_parent - draw_events_before_parent, 1,
        "Dagger Throw draws its one card before opening the selection");
    select(&mut reflex_combat, reflex);
    assert_eq!(reflex_combat.state.player.energy, 2);
    assert_eq!(discard_count(&reflex_combat, reflex), 1);
    assert_eq!(discarded_hook_count(&reflex_combat, reflex), 1);
    assert_eq!(reflex_combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardDrawn).count() - draw_events_after_parent, 2,
        "base Reflex itself draws two cards");
    assert!(hand_card_by_instance(&reflex_combat, reflex).is_none(),
        "the selected Reflex stays in discard when two real draw-pile cards are available");

    for (upgrades, expected_gain) in [(0, 1), (1, 2)] {
        let mut tactician_combat = actual_act2_combat(
            EncounterId::Slavers,
            &[(CardId::DaggerThrow, 0), (CardId::Tactician, upgrades), (CardId::DefendGreen, 0)],
            3,
            3,
            5,
        );
        let dagger_throw = hand_instance(&tactician_combat, CardId::DaggerThrow);
        let tactician = hand_instance(&tactician_combat, CardId::Tactician);
        play(&mut tactician_combat, dagger_throw, Some(TargetId(1)));
        assert_eq!(tactician_combat.state.player.energy, 2);
        select(&mut tactician_combat, tactician);
        assert_eq!(tactician_combat.state.player.energy, 2 + expected_gain,
            "Tactician's own base/upgraded discard grants the captured energy exactly once");
        assert_eq!(discard_count(&tactician_combat, tactician), 1);
        assert_eq!(discarded_hook_count(&tactician_combat, tactician), 1);
        assert!(hand_card_by_instance(&tactician_combat, tactician).is_none());
    }
}

// Replacing actual Act2 setup, retain, or Nightmare custody should break the
// retained-original and three-distinct-copy ownership assertions.
#[test]
fn m13_retain_and_nightmare_copy_keep_actual_act2_instance_ownership() {
    let nightmare = crate::content::Catalog::standard().cards().iter()
        .find(|definition| definition.id.as_str() == "nightmare")
        .expect("Nightmare is in the supported card catalog").id;
    let mut combat = actual_act2_combat(
        EncounterId::SphericGuardian,
        &[(CardId::WellLaidPlans, 0), (nightmare, 0), (CardId::DefendGreen, 0)],
        3,
        4,
        5,
    );
    let well_laid_plans = hand_instance(&combat, CardId::WellLaidPlans);
    let nightmare_instance = hand_instance(&combat, nightmare);
    let retained = hand_instance(&combat, CardId::DefendGreen);

    play(&mut combat, well_laid_plans, None);
    assert_eq!(combat.state.player_full_powers.iter()
        .find(|power| power.id == FullPowerId("retain_cards")).map(|power| power.state.amount), Some(1));
    play(&mut combat, nightmare_instance, None);
    assert!(combat.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(retained))));
    select(&mut combat, retained);
    assert_eq!(combat.state.player.energy, 0);
    assert!(combat.permanent_gains().is_empty(),
        "capturing a card for Nightmare is not a permanent deck acquisition");

    combat.step_full(&CombatAction::EndTurn)
        .expect("Well-Laid Plans opens its actual end-turn retain selector");
    assert_eq!(combat.state.pending_selection.as_ref().unwrap().maximum, 1);
    assert!(combat.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(retained))));
    select(&mut combat, retained);
    combat.step_full(&CombatAction::ConfirmSelection)
        .expect("the exact selected original card is retained while the shared turn continues");

    assert_eq!(combat.state.turn, 2);
    let in_hand = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == CardId::DefendGreen).collect::<Vec<_>>();
    assert_eq!(in_hand.len(), 4,
        "Well-Laid Plans keeps the original while Nightmare delivers three copies next turn");
    assert!(in_hand.iter().any(|card| card.instance.0 == retained),
        "the retained card is the exact original combat instance");
    let copies = in_hand.iter().filter(|card| card.instance.0 != retained)
        .map(|card| card.instance.0).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(copies.len(), 3,
        "Nightmare copies have three separate temporary instance identities");
    assert!(combat.permanent_gains().is_empty(),
        "the delivered copies never enter the permanent deck gain journal");
}
