use spire_core::{
    combat::{self, Combat, CombatAction, SelectionOperation, TargetId},
    content::{CardId, Character, EncounterId, RelicId},
    items::CombatItemContext,
    run::{Candidate, CreationProfile, GameAction, PublicDecision, PublicObject, Run, RunConfig, RunRng, RoomScenario},
};
use spire_env::{encode_run_m12, m12_run_observation_spec, RunCapacities, RunObservation};

fn decision_with_silent_combat(cards: &[CardId]) -> (PublicDecision, Combat) {
    decision_with_silent_combat_at(cards, EncounterId::Cultist, 3, 70)
}

fn decision_with_silent_combat_energy(cards: &[CardId], energy: i32) -> (PublicDecision, Combat) {
    decision_with_silent_combat_at(cards, EncounterId::Cultist, energy, 70)
}

fn decision_with_silent_combat_at(cards: &[CardId], encounter: EncounterId, energy: i32, hp: i32) -> (PublicDecision, Combat) {
    decision_with_silent_combat_at_upgraded(cards, encounter, energy, hp, &[])
}

fn decision_with_silent_combat_at_upgraded(
    cards: &[CardId], encounter: EncounterId, energy: i32, hp: i32, upgrades: &[u32],
) -> (PublicDecision, Combat) {
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    let base = Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        creation,
        RunRng::new([203; 32], combat::rng::Partition::Test, 0),
        RoomScenario::M12SilentAct1,
    ).unwrap().decision().unwrap();
    let live_cards = cards.iter().enumerate().map(|(index, id)|
        combat::permanent_live_card(*id, combat::CardInstanceId(index as u64 + 1), upgrades.get(index).copied().unwrap_or(0))).collect::<Vec<_>>();
    let setup = combat::enemies::setup(
        encounter,
        1,
        0,
        &mut RunRng::new([207; 32], combat::rng::Partition::Test, 0),
        combat::enemies::SetupContext {
            player: combat::PlayerState { hp, max_hp: hp, block: 0, energy, max_energy: energy, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Silent,
            cards: live_cards,
            opening_draw: cards.len(),
            encounter_visit: 0,
        },
    ).unwrap();
    let mut items = Some(CombatItemContext::new(vec![RelicId::RingOfTheSnake], vec![None, None, None]));
    let native = Combat::enter_with_item_context(setup, &mut items).unwrap();
    let mut decision = base;
    decision.objects = cards.iter().enumerate().map(|(index, id)| PublicObject { misc:0,
        instance: index as u64 + 1, id: *id, upgrades: upgrades.get(index).copied().unwrap_or(0),
    }).collect();
    decision.player.hp = u32::try_from(hp).unwrap();
    decision.player.max_hp = u32::try_from(hp).unwrap();
    decision.combat = Some(native.public_full().unwrap());
    decision.room = None;
    decision.offers.clear();
    decision.selection = None;
    (decision, native)
}

fn publish_combat(decision: &mut PublicDecision, combat: &Combat) -> RunObservation {
    let public = combat.public_full().unwrap();
    let candidates = combat.legal_actions_full().unwrap().into_iter().enumerate().map(|(index, action)| Candidate {
        id: spire_core::run::ChoiceId(900 + index as u64),
        action: GameAction::Combat(action),
        label: "actual shared-handler action",
        neow_offer: None,
        object: None,
    }).collect::<Vec<_>>();
    decision.combat = Some(public);
    decision.candidates = candidates;
    encode_run_m12(decision, None, &RunCapacities::m10_default()).unwrap()
}

fn group<'a>(observation: &'a RunObservation, key: &str) -> &'a spire_env::RunFeatureGroup {
    observation.group(key).unwrap()
}

fn operation_id(name: &str) -> i64 {
    m12_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor()["vocabularies"]["selection_operation"]
        .as_array().unwrap().iter().position(|value| value == name).unwrap() as i64
}

fn selected_card_row(observation: &RunObservation, instance: u64) -> usize {
    let rows = group(observation, "combat_cards");
    rows.values.chunks(18).position(|row| row[0] >= 0 && row[5] >= 0 && row[3] <= 1
        && row[5] == (instance as i64 - 1)).expect("permanent row references selected card")
}

#[test]
fn m12_actual_silent_card_values_encode_base_and_upgraded_public_fields() {
    for id in spire_core::run::M12_SILENT_ACT1_CARDS {
        for upgrades in [0, 1] {
            let (mut decision, combat) = decision_with_silent_combat_at_upgraded(
                &[*id], EncounterId::Cultist, 3, 70, &[upgrades]);
            let public = combat.public_full().unwrap();
            let card = public.hand.iter().find(|card| card.instance.0 == 1)
                .expect("the actual supplied permanent card remains in hand");
            let values = card.values.as_ref().expect("the finite M12 card has owned public values");
            let observation = publish_combat(&mut decision, &combat);
            let row = selected_card_row(&observation, 1);
            let cards = group(&observation, "combat_cards");
            let offset = row * 18;
            for (column, value) in [
                (8, values.base_damage), (9, values.base_block),
                (10, values.magic_number), (11, values.damage_bonus),
            ] {
                assert_eq!(cards.known[offset + column], i8::from(value.is_some()), "{id:?} +{upgrades} column {column} knownness");
                if let Some(value) = value {
                    assert_eq!(cards.values[offset + column], i64::from(value), "{id:?} +{upgrades} column {column}");
                }
            }
            let expected_cost_kind = match values.cost_kind {
                combat::CardCostKind::Fixed => 1,
                combat::CardCostKind::X => 2,
                combat::CardCostKind::Unplayable => 3,
            };
            assert_eq!(cards.values[offset + 4], card.effective_cost.map(i64::from).unwrap_or(0));
            assert_eq!(cards.known[offset + 4], i8::from(card.effective_cost.is_some()));
            assert_eq!(cards.values[offset + 12], expected_cost_kind);
            assert_eq!(&cards.known[offset + 12..offset + 18], &[1; 6]);
            assert_eq!(&cards.values[offset + 13..offset + 18], &[
                i64::from(values.free_play), i64::from(values.exhaust_on_finish),
                i64::from(values.ethereal), i64::from(values.innate), i64::from(values.temporary_cost),
            ]);
        }
    }
}

#[test]
fn m12_act1_enemy_status_producers_encode_generated_cards() {
    let cases = [
        (EncounterId::ThreeSentries, CardId::Dazed),
        (EncounterId::Hexaghost, CardId::Burn),
        (EncounterId::SlimeBoss, CardId::Slimed),
    ];
    for (encounter, status) in cases {
        let cards = [CardId::DefendGreen; 5];
        let (mut decision, mut combat) = decision_with_silent_combat_at(&cards, encounter, 3, 500);
        let mut found = false;
        for _ in 0..8 {
            let public = combat.public_full().unwrap();
            found = public.hand.iter().chain(&public.discard).chain(&public.exhaust).any(|card| card.id == status);
            if found { break; }
            assert!(combat.legal_actions_full().unwrap().contains(&CombatAction::EndTurn), "{encounter:?} is at a legal end-turn boundary");
            combat.step_full(&CombatAction::EndTurn).unwrap();
        }
        assert!(found, "the actual {encounter:?} producer creates {status:?} within bounded end turns");
        let snapshot = publish_combat(&mut decision, &combat);
        let vocabulary = m12_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor()["vocabularies"]["card"].clone();
        let card_id = vocabulary.as_array().unwrap().iter().position(|value| value == status.as_str()).unwrap() as i64;
        assert!(group(&snapshot, "combat_cards").values.chunks(18).any(|row| row[2] == card_id),
            "actual local {status:?} card identity has a finite M12 combat-card row");
    }
}

#[test]
fn m12_actual_discard_retain_and_nightmare_snapshots_encode() {
    for reaction in [CardId::Reflex, CardId::Tactician] {
        let cards = [CardId::DaggerThrow, reaction, CardId::StrikeGreen, CardId::DefendGreen, CardId::Neutralize];
        let (mut decision, mut combat) = decision_with_silent_combat(&cards);
        let before = combat.public_full().unwrap();
        let dagger = before.hand.iter().find(|card| card.id == CardId::DaggerThrow).unwrap().instance.0;
        let reaction_id = before.hand.iter().find(|card| card.id == reaction).unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance: dagger, target: Some(TargetId(1)) }).unwrap();
        let pending = combat.public_full().unwrap();
        let selection = pending.selection.as_ref().expect("actual Dagger Throw opens its shared discard selector");
        assert_eq!(selection.operation, SelectionOperation::Move(combat::SelectionDisposition::Discard));
        assert!(selection.candidates.iter().any(|card| card.instance.0 == reaction_id));
        let snapshot = publish_combat(&mut decision, &combat);
        assert_eq!(group(&snapshot, "selection_counts").values[5], operation_id("move_discard"));
        let selection_rows = group(&snapshot, "selection");
        let selected_row = selection_rows.values.chunks(4).find(|row| row[1] == selected_card_row(&snapshot, reaction_id) as i64)
            .expect("discard choice references the exact hand card row");
        assert_eq!(&selected_row[0..3], &[4, selected_card_row(&snapshot, reaction_id) as i64, 1]);
        assert_eq!(selected_row[3], 0, "unselected discard candidates remain marked unselected");

        combat.step_full(&CombatAction::SelectCard { instance: reaction_id }).unwrap();
        let after = combat.public_full().unwrap();
        assert!(after.selection.is_none(), "Dagger Throw commits the exact selected discard");
        if reaction == CardId::Reflex {
            assert!(after.hand.iter().any(|card| card.instance.0 == reaction_id && card.id == reaction),
                "with the supplied empty draw pile, Reflex's actual Draw 2 reshuffles its selected discard back into hand");
            assert!(after.hand.iter().any(|card| card.instance.0 == dagger && card.id == CardId::DaggerThrow),
                "the same real reshuffle draws the just-finished Dagger Throw from discard");
            assert!(!after.discard.iter().any(|card| card.instance.0 == reaction_id || card.instance.0 == dagger));
        } else {
            assert!(after.discard.iter().any(|card| card.instance.0 == reaction_id && card.id == reaction),
                "the actual Tactician reaction remains in discard after its energy trigger");
        }
        if reaction == CardId::Tactician {
            assert_eq!(after.player.energy, 3, "the selected Tactician reaction restores the Dagger Throw payment");
        } else {
            assert_eq!(after.player.energy, 2);
        }
        let after_snapshot = publish_combat(&mut decision, &combat);
        if reaction == CardId::Reflex {
            let cards = group(&after_snapshot, "combat_cards").values.chunks(18).collect::<Vec<_>>();
            assert!(cards.iter().any(|row| row[0] == 1 && row[5] == (reaction_id as i64 - 1)),
                "the exact selected Reflex is encoded in its actual post-reshuffle hand zone");
            assert!(cards.iter().any(|row| row[0] == 1 && row[5] == (dagger as i64 - 1)),
                "the actual Dagger Throw reshuffle draw also appears in the post-action observation");
        }
    }

    let power_cards = [CardId::NoxiousFumes, CardId::Accuracy, CardId::DeadlyPoison, CardId::BladeDance, CardId::DefendGreen];
    let (mut decision, mut combat) = decision_with_silent_combat_energy(&power_cards, 5);
    for (id, target) in [
        (CardId::NoxiousFumes, None),
        (CardId::Accuracy, None),
        (CardId::DeadlyPoison, Some(TargetId(1))),
        (CardId::BladeDance, None),
    ] {
        let instance = combat.public_full().unwrap().hand.iter().find(|card| card.id == id).unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target }).unwrap();
        publish_combat(&mut decision, &combat);
    }
    let powered = combat.public_full().unwrap();
    assert!(powered.player.powers.iter().any(|power| power.id.0 == "noxious_fumes" && power.amount == 2));
    assert!(powered.player.powers.iter().any(|power| power.id.0 == "accuracy" && power.amount == 4));
    assert!(powered.enemies[0].powers.iter().any(|power| power.id.0 == "poison" && power.amount == 5));
    let shivs = powered.hand.iter().filter(|card| card.id == CardId::Shiv).collect::<Vec<_>>();
    assert_eq!(shivs.len(), 3, "actual base Blade Dance generates three finite Shiv card rows");
    assert!(shivs.iter().all(|card| card.values.as_ref().is_some_and(|values|
        values.base_damage == Some(8) && values.exhaust_on_finish)),
        "actual Accuracy leaves each generated Shiv at base damage 8 and the Shiv exhaust flag remains public");
    let powers_snapshot = publish_combat(&mut decision, &combat);
    let power_vocab = m12_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor()["vocabularies"]["power"].clone();
    let power_id = |name: &str| power_vocab.as_array().unwrap().iter().position(|value| value == name).unwrap() as i64;
    let powers = group(&powers_snapshot, "powers");
    assert!(powers.values.chunks(4).any(|row| row[2] == power_id("noxious_fumes") && row[3] == 2));
    assert!(powers.values.chunks(4).any(|row| row[2] == power_id("accuracy") && row[3] == 4));
    assert!(powers.values.chunks(4).any(|row| row[2] == power_id("poison") && row[3] == 5));
    let card_vocab = m12_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor()["vocabularies"]["card"].clone();
    let shiv_id = card_vocab.as_array().unwrap().iter().position(|value| value == "shiv").unwrap() as i64;
    let cards_snapshot = group(&powers_snapshot, "combat_cards");
    assert_eq!(cards_snapshot.values.chunks(18).filter(|row| row[0] == 1 && row[2] == shiv_id && row[8] == 8 && row[14] == 1).count(), 3,
        "the full M12 schema encodes each actual generated Shiv's Accuracy damage and exhaust flag");

    let retain_cards = [CardId::WellLaidPlans, CardId::StrikeGreen, CardId::DefendGreen, CardId::Neutralize, CardId::Survivor];
    let (mut decision, mut combat) = decision_with_silent_combat(&retain_cards);
    let wlp = combat.public_full().unwrap().hand.iter().find(|card| card.id == CardId::WellLaidPlans).unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: wlp, target: None }).unwrap();
    let power_state = publish_combat(&mut decision, &combat);
    let power_vocab = m12_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor()["vocabularies"]["power"].clone();
    let retain_power = power_vocab.as_array().unwrap().iter().position(|value| value == "retain_cards").unwrap() as i64;
    assert!(group(&power_state, "powers").values.chunks(4).any(|row| row[2] == retain_power && row[3] == 1),
        "actual Well-Laid Plans publishes its finite Silent power");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let retain = combat.public_full().unwrap().selection.expect("actual Well-Laid Plans opens retain on end turn");
    assert_eq!(retain.operation, SelectionOperation::Retain);
    assert_eq!((retain.minimum, retain.maximum), (0, 1));
    let empty_snapshot = publish_combat(&mut decision, &combat);
    assert_eq!(group(&empty_snapshot, "selection_counts").values[4], 1, "Confirm0 is a real legal action");
    assert_eq!(group(&empty_snapshot, "selection_counts").values[5], operation_id("retain"));
    let chosen = retain.candidates[0].instance.0;
    combat.step_full(&CombatAction::SelectCard { instance: chosen }).unwrap();
    let selected_snapshot = publish_combat(&mut decision, &combat);
    let counts = group(&selected_snapshot, "selection_counts");
    assert_eq!(counts.values[1], 1);
    assert_eq!(counts.values[5], operation_id("retain"));
    let row = selected_card_row(&selected_snapshot, chosen);
    let card_width = 18;
    assert_eq!(group(&selected_snapshot, "combat_cards").values[row * card_width + 7], 1,
        "retain selection and combat card row agree on selected identity");
    assert!(group(&selected_snapshot, "selection").values.chunks(4).any(|selection| selection[1] == row as i64 && selection[3] == 1));
    combat.step_full(&CombatAction::ConfirmSelection).unwrap();
    publish_combat(&mut decision, &combat);

    let nightmare_cards = [CardId::Nightmare, CardId::StrikeGreen, CardId::DefendGreen, CardId::Neutralize, CardId::Survivor];
    let (mut decision, mut combat) = decision_with_silent_combat(&nightmare_cards);
    let initial = combat.public_full().unwrap();
    let nightmare = initial.hand.iter().find(|card| card.id == CardId::Nightmare).unwrap().instance.0;
    let target = initial.hand.iter().find(|card| card.id == CardId::StrikeGreen).unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: nightmare, target: None }).unwrap();
    let capture = combat.public_full().unwrap().selection.expect("actual Nightmare opens its capture selector");
    assert_eq!(capture.operation, SelectionOperation::NightmareCapture);
    let capture_snapshot = publish_combat(&mut decision, &combat);
    assert_eq!(group(&capture_snapshot, "selection_counts").values[5], operation_id("nightmare_capture"));
    let card_vocab = m12_run_observation_spec(&RunCapacities::m10_default()).unwrap().descriptor()["vocabularies"]["card"].clone();
    let nightmare_card_id = card_vocab.as_array().unwrap().iter().position(|value| value == "nightmare").unwrap() as i64;
    assert!(group(&capture_snapshot, "combat_cards").values.chunks(18).any(|row| row[0] == 9 && row[2] == nightmare_card_id),
        "the actual playing Nightmare has its finite card ID and dedicated playing zone");

    combat.step_full(&CombatAction::SelectCard { instance: target }).unwrap();
    publish_combat(&mut decision, &combat);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let next_turn = combat.public_full().unwrap();
    assert_eq!(next_turn.turn, 2);
    let copies = next_turn.hand.iter().filter(|card| card.id == CardId::StrikeGreen
        && matches!(card.origin, combat::CombatCardRef::CombatLocal(_))).count();
    assert_eq!(copies, 3, "actual Nightmare delivers three generated Silent card copies on the next turn");
    let copied_snapshot = publish_combat(&mut decision, &combat);
    assert!(group(&copied_snapshot, "combat_cards").values.chunks(18)
        .filter(|row| row[0] == 1 && row[2] == 1).count() >= 3,
        "generated copies are encoded as ordinary finite card rows");
}
