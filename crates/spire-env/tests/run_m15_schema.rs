use serde_json::Value;
use spire_core::{
    combat::{
        CardCostKind, CardInstanceId, CardPublicValues, CharacterCombatView, CombatAction,
        CombatCardOfferView, CombatCardRef, CombatInventoryView, CombatPublic, EncounterActorView,
        EncounterFacingView, EncounterSide, FullEnemyView, FullIntentId, FullPlayerView,
        FullPowerId, FullPowerView, LiveCardView, PlayerFacing, SelectionDisposition,
        SelectionOperation, SelectionParent, SelectionSourceZone, SelectionView, Stance,
    },
    content::{CardId, Character, EncounterId, EnemyId, RelicId},
    run::{
        Candidate, ChoiceId, CombatProgress, CombatRecord, GameAction, PublicDecision,
        PublicObject, PublicPlayer, RoomScenario, RoomView, RoomViewKind,
    },
};
use spire_env::{
    encode_run_m14, encode_run_m15, m10_run_observation_spec, m14_run_observation_spec,
    m15_run_observation_spec, RunCapacities, RunSessionError, RUN_OBSERVATION_M14_SCHEMA_ID,
    RUN_OBSERVATION_M15_SCHEMA_ID,
};

const COLORLESS: &[CardId] = &[
    CardId::Apotheosis,
    CardId::Blind,
    CardId::Chrysalis,
    CardId::DarkShackles,
    CardId::DeepBreath,
    CardId::Discovery,
    CardId::DramaticEntrance,
    CardId::Enlightenment,
    CardId::Finesse,
    CardId::FlashOfSteel,
    CardId::Forethought,
    CardId::GoodInstincts,
    CardId::HandOfGreed,
    CardId::Impatience,
    CardId::JackOfAllTrades,
    CardId::Madness,
    CardId::Magnetism,
    CardId::MasterOfStrategy,
    CardId::Mayhem,
    CardId::Metamorphosis,
    CardId::MindBlast,
    CardId::Panacea,
    CardId::Panache,
    CardId::PanicButton,
    CardId::Purity,
    CardId::SadisticNature,
    CardId::SecretTechnique,
    CardId::SecretWeapon,
    CardId::SwiftStrike,
    CardId::TheBomb,
    CardId::ThinkingAhead,
    CardId::Transmutation,
    CardId::Trip,
    CardId::Violence,
];

const GENERATED_AND_STATUS: &[CardId] = &[
    CardId::Apotheosis,
    CardId::Blind,
    CardId::Chrysalis,
    CardId::DarkShackles,
    CardId::DeepBreath,
    CardId::Discovery,
    CardId::DramaticEntrance,
    CardId::Enlightenment,
    CardId::Finesse,
    CardId::FlashOfSteel,
    CardId::Forethought,
    CardId::GoodInstincts,
    CardId::HandOfGreed,
    CardId::Impatience,
    CardId::JackOfAllTrades,
    CardId::Madness,
    CardId::Magnetism,
    CardId::MasterOfStrategy,
    CardId::Mayhem,
    CardId::Metamorphosis,
    CardId::MindBlast,
    CardId::Panacea,
    CardId::Panache,
    CardId::PanicButton,
    CardId::Purity,
    CardId::SadisticNature,
    CardId::SecretTechnique,
    CardId::SecretWeapon,
    CardId::SwiftStrike,
    CardId::TheBomb,
    CardId::ThinkingAhead,
    CardId::Transmutation,
    CardId::Trip,
    CardId::Violence,
    CardId::Shiv,
    CardId::Dazed,
    CardId::Burn,
    CardId::Slimed,
    CardId::Wound,
    CardId::Void,
];

fn vocab<'a>(descriptor: &'a Value, key: &str) -> &'a [Value] {
    descriptor["vocabularies"][key]
        .as_array()
        .unwrap_or_else(|| panic!("missing {key} vocabulary"))
}

fn vocabulary_row(descriptor: &Value, key: &str, name: &str) -> i64 {
    vocab(descriptor, key)
        .iter()
        .position(|value| value == name)
        .unwrap_or_else(|| panic!("missing {key} name {name}")) as i64
}

fn tensor<'a>(descriptor: &'a Value, key: &str) -> &'a Value {
    descriptor["tensors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tensor| tensor["key"] == key)
        .unwrap_or_else(|| panic!("missing tensor {key}"))
}

fn fixture_decision() -> PublicDecision {
    PublicDecision {
        content_profile: Some(RoomScenario::M15SilentHeart),
        room: None,
        reward_total: 0,
        act: 4,
        floor: 54,
        player: PublicPlayer {
            relic_counters: vec![],
            character: Character::Silent,
            hp: 70,
            max_hp: 70,
            gold: 0,
            relics: vec![],
            potions: vec![None; 3],
            keys: [true; 3],
        },
        combat: None,
        combat_progress: CombatProgress::default(),
        last_combat: None,
        objects: vec![],
        candidates: vec![],
        offers: vec![],
        selection: None,
        terminal: None,
    }
}

fn combat() -> CombatPublic {
    CombatPublic {
        played_powers: vec![],
        playing_cards: vec![],
        stasis: vec![],
        content_id: "m15-schema-fixture".into(),
        turn: 1,
        player: FullPlayerView {
            hp: 70,
            max_hp: 70,
            block: 0,
            energy: 3,
            max_energy: 3,
            powers: vec![],
        },
        enemies: vec![],
        character: CharacterCombatView {
            stance: Stance::Neutral,
            orbs: vec![],
            focus: 0,
        },
        hand: vec![],
        draw_count: 0,
        discard: vec![],
        exhaust: vec![],
        outcome: None,
        hp_before_healing: None,
        hp_after_cleanup: None,
        targets: vec![],
        selection: None,
        known_draw: vec![],
        encounter_facing: None,
        inventory: Some(CombatInventoryView {
            relic_counters: vec![],
            relics: vec![],
            potions: vec![None; 3],
        }),
        card_offer: None,
    }
}

fn live_card(id: CardId, instance: u64, values: Option<CardPublicValues>) -> LiveCardView {
    LiveCardView {
        instance: CardInstanceId(instance),
        id,
        origin: CombatCardRef::CombatLocal(instance),
        upgrades: 0,
        effective_cost: Some(1),
        values,
    }
}

fn enemy(id: EnemyId, instance: usize, intent: &'static str) -> FullEnemyView {
    FullEnemyView {
        instance_id: spire_core::combat::EnemyHandle::parse(&format!("enemy-{instance}"))
            .unwrap(),
        enemy_id: id,
        hp: 100,
        max_hp: 100,
        block: 0,
        powers: vec![],
        intent: FullIntentId::from_name(intent),
        intent_damage: Some(10),
        replaced: false,
        escaped: false,
        half_dead: false,
        logical_slot: instance as u8,
    }
}

fn candidate(id: u64, action: GameAction) -> Candidate {
    Candidate {
        id: ChoiceId(id),
        action,
        label: "schema fixture",
        neow_offer: None,
        object: None,
    }
}

#[test]
fn m15_descriptor_contains_the_complete_finite_silent_and_act4_vocabulary() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();
    assert_eq!(descriptor["schema_id"], RUN_OBSERVATION_M15_SCHEMA_ID);
    assert_eq!(
        descriptor["schema_id"],
        "spire-gym-m15-silent-heart-observation-v1"
    );
    assert_eq!(descriptor["profile_id"], "M15SilentHeart");
    assert_eq!(
        vocab(&descriptor, "outcome"),
        ["ongoing", "victory", "death", "escaped"]
            .map(|name| Value::String(name.into()))
            .as_slice()
    );

    let card_vocab = vocab(&descriptor, "card");
    assert_eq!(card_vocab.len(), 117, "padding + 76 permanent + 34 colorless + 6 generated/status");
    let entitlements = &descriptor["semantics"]["entitlements"];
    let permanent = entitlements["permanent_cards"].as_array().unwrap();
    assert_eq!(permanent.len(), 76, "75 ordinary green cards plus permanent Parasite");
    assert_eq!(
        permanent.iter().map(|value| value.as_str().unwrap()).collect::<Vec<_>>(),
        spire_core::run::M15_SILENT_HEART_CARDS
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
    );
    assert!(permanent.iter().any(|id| id == "parasite"));
    let generated = entitlements["generated_and_status_cards"]
        .as_array()
        .unwrap();
    assert_eq!(generated.len(), 40, "34 colorless kinds plus Shiv and five status kinds");
    assert_eq!(
        generated.iter().map(|value| value.as_str().unwrap()).collect::<Vec<_>>(),
        GENERATED_AND_STATUS
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
    );
    for id in spire_core::run::M15_SILENT_HEART_CARDS
        .iter()
        .copied()
        .chain(COLORLESS.iter().copied())
        .chain([CardId::Shiv, CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Wound, CardId::Void])
    {
        assert!(card_vocab.iter().any(|value| value == id.as_str()), "card {id:?}");
    }
    let mut unique_cards = card_vocab
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    unique_cards.sort_unstable();
    unique_cards.dedup();
    assert_eq!(unique_cards.len(), card_vocab.len(), "card rows are unique");

    assert_eq!(vocab(&descriptor, "relic").len(), 29);
    assert_eq!(
        vocab(&descriptor, "relic")[1..]
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>(),
        spire_core::run::M15_SILENT_HEART_RELICS
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(vocab(&descriptor, "potion").len(), 34);
    assert_eq!(
        vocab(&descriptor, "potion")[1..]
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>(),
        spire_core::run::M15_SILENT_HEART_POTIONS
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
    );

    let m14 = m14_run_observation_spec(&capacities).unwrap().descriptor();
    for category in ["card", "enemy", "encounter", "event", "intent", "power"] {
        for name in vocab(&m14, category).iter().skip(1) {
            assert!(
                vocab(&descriptor, category).contains(name),
                "M15 must retain M14 {category} {name}"
            );
        }
    }
    assert_eq!(
        descriptor["semantics"]["enemy_species"]["cumulative_through_act4"]
            .as_array()
            .unwrap()
            .len(),
        57
    );
    for name in ["spire_shield", "spire_spear", "corrupt_heart"] {
        assert!(vocab(&descriptor, "enemy").iter().any(|value| value == name));
    }
    assert_eq!(
        descriptor["semantics"]["encounters"]["playable"]
            .as_array()
            .unwrap()
            .len(),
        42
    );
    assert_eq!(
        descriptor["semantics"]["encounters"]["act4"],
        serde_json::json!(["shield_and_spear", "corrupt_heart"])
    );

    let m10 = m10_run_observation_spec(&capacities).unwrap().descriptor();
    for category in ["intent", "power"] {
        for name in vocab(&m10, category)
            .iter()
            .filter(|value| value.as_str().is_some_and(|name| name.starts_with("heart_")
                || name.starts_with("shield_") || name.starts_with("spear_")
                || matches!(name, "beat_of_death" | "invincible" | "surrounded" | "back_attack")))
        {
            assert!(vocab(&descriptor, category).contains(name), "M15 Act4 {category} {name}");
        }
    }
    for room in ["act4_ready", "heart_victory", "true_victory"] {
        assert!(vocab(&descriptor, "room").iter().any(|value| value == room));
    }
    for name in ["lose_strength", "lose_dexterity", "plated_armor", "duplication"] {
        assert!(vocab(&descriptor, "power").iter().any(|value| value == name));
    }
    assert!(vocab(&descriptor, "terminal").iter().any(|value| value == "heart_victory"));
    assert_eq!(
        vocab(&descriptor, "profile"),
        [Value::String("padding".into()), Value::String("M15SilentHeart".into())]
    );

    let operations = vocab(&descriptor, "selection_operation");
    for name in ["nightmare_capture", "retain", "setup", "bottomdeck", "retrieve_memories"] {
        assert!(operations.iter().any(|value| value == name), "selection operation {name}");
    }
    assert_eq!(
        vocab(&descriptor, "verb")[11..],
        ["confirm_selection", "choose_card_offer", "skip_card_offer"]
            .into_iter()
            .map(|name| Value::String(name.into()))
            .collect::<Vec<_>>()
    );
    assert!(!descriptor.to_string().contains("draw_queue"));
    assert!(!descriptor.to_string().contains("private_template"));
    assert!(!descriptor.to_string().contains("future_offer"));
    assert!(!descriptor.to_string().contains("rng_cursor"));
}

#[test]
fn m15_potion_generated_powers_and_escape_outcomes_resolve_to_named_rows() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();

    let power_cases = [
        ("lose_strength", -2),
        ("lose_dexterity", -2),
        ("plated_armor", 6),
        ("duplication", 1),
    ];
    let mut active = fixture_decision();
    let mut public = combat();
    public.player.powers = power_cases
        .iter()
        .map(|(name, amount)| FullPowerView {
            id: FullPowerId(name),
            amount: *amount,
        })
        .collect();
    active.combat = Some(public);
    let observation = encode_run_m15(&active, None, &capacities).unwrap();
    let powers = observation.group("powers").unwrap();
    let columns = powers.shape[1];
    for (row_index, (name, amount)) in power_cases.iter().enumerate() {
        assert_eq!(powers.values[row_index * columns], 1, "player owns {name}");
        assert_eq!(powers.values[row_index * columns + 2], vocabulary_row(&descriptor, "power", name));
        assert_eq!(powers.values[row_index * columns + 3], i64::from(*amount));
        assert_eq!(powers.known[row_index * columns..row_index * columns + columns], [1; 4]);
    }

    let mut active_escape = fixture_decision();
    let mut active_combat = combat();
    active_combat.outcome = Some(spire_core::combat::Outcome::Escaped);
    let mut escaped_enemy = enemy(EnemyId::Cultist, 0, "incantation");
    escaped_enemy.escaped = true;
    active_combat.enemies.push(escaped_enemy);
    active_escape.combat = Some(active_combat);
    let active_observation = encode_run_m15(&active_escape, None, &capacities).unwrap();
    let escaped_column = tensor(&descriptor, "enemies")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|field| field == "escaped")
        .unwrap();
    assert_eq!(active_observation.group("enemies").unwrap().values[escaped_column], 1);
    assert_eq!(active_observation.group("enemies").unwrap().known[escaped_column], 1);

    let mut settled_escape = fixture_decision();
    settled_escape.last_combat = Some(CombatRecord {
        visit: 1,
        encounter: EncounterId::Cultist,
        outcome: spire_core::combat::Outcome::Escaped,
    });
    let settled_observation = encode_run_m15(&settled_escape, None, &capacities).unwrap();
    let outcome_column = tensor(&descriptor, "context")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|field| field == "last_outcome")
        .unwrap();
    assert_eq!(settled_observation.group("context").unwrap().values[outcome_column], 3);
    assert_eq!(settled_observation.group("context").unwrap().known[outcome_column], 1);
}

#[test]
fn m15_groups_keep_m10_heart_state_and_bound_public_card_offer_rows() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();
    let groups = vocab(&descriptor, "row_group");
    assert_eq!(groups[11], "combat_card_offers");
    let verbs = vocab(&descriptor, "verb");
    assert_eq!(verbs[11], "confirm_selection");
    assert_eq!(verbs[12], "choose_card_offer");
    assert_eq!(verbs[13], "skip_card_offer");

    let offers = tensor(&descriptor, "combat_card_offers");
    assert_eq!(offers["shape"], serde_json::json!([capacities.offers, 2]));
    assert_eq!(offers["fields"], serde_json::json!(["card_kind", "offer_index"]));
    assert_eq!(offers["reference_columns"], serde_json::json!([]));
    assert_eq!(
        offers["columns"][0]["high"],
        (vocab(&descriptor, "card").len() - 1) as u64
    );
    assert_eq!(offers["columns"][1]["low"], 0);
    assert_eq!(offers["columns"][1]["high"], capacities.offers - 1);
    assert_eq!(tensor(&descriptor, "combat_card_offers_known")["shape"],
        serde_json::json!([capacities.offers, 2]));
    assert_eq!(tensor(&descriptor, "combat_card_offers_occupied")["shape"],
        serde_json::json!([capacities.offers]));

    let state = tensor(&descriptor, "combat_state");
    assert_eq!(
        state["fields"],
        serde_json::json!(["facing", "surrounded", "heart_damage_remaining", "heart_cap"])
    );
    assert_eq!(state["columns"][2]["low"], 0);
    assert_eq!(state["columns"][2]["high"], 300);
    assert_eq!(state["columns"][3]["high"], 300);

    let cards = tensor(&descriptor, "combat_cards");
    for field in ["base_damage", "base_block", "magic_number", "damage_bonus"] {
        let column = cards["fields"]
            .as_array()
            .unwrap()
            .iter()
            .position(|value| value == field)
            .unwrap();
        assert_eq!(cards["columns"][column]["low"], i32::MIN);
        assert_eq!(cards["columns"][column]["high"], i32::MAX);
    }
}

#[test]
fn m15_public_offer_rows_actions_masks_and_signed_values_are_encoded_explicitly() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();
    let mut decision = fixture_decision();
    let mut combat = combat();
    combat.hand.push(live_card(
        CardId::WraithForm,
        7,
        Some(CardPublicValues {
            base_damage: None,
            base_block: None,
            magic_number: Some(2),
            damage_bonus: Some(-3),
            cost_kind: CardCostKind::Fixed,
            free_play: false,
            exhaust_on_finish: false,
            ethereal: false,
            innate: false,
            temporary_cost: -1,
        }),
    ));
    combat.player.powers.push(FullPowerView {
        id: FullPowerId("wraith_form"),
        amount: -1,
    });
    combat.card_offer = Some(CombatCardOfferView {
        cards: vec![CardId::Blind, CardId::Apotheosis, CardId::HandOfGreed],
        optional: true,
    });
    decision.combat = Some(combat);
    decision.candidates = vec![
        candidate(
            1,
            GameAction::Combat(CombatAction::ChooseCardOffer { index: 1 }),
        ),
        candidate(2, GameAction::Combat(CombatAction::SkipCardOffer)),
    ];

    let observation = encode_run_m15(&decision, None, &capacities).unwrap();
    let offer_rows = observation.group("combat_card_offers").unwrap();
    let card_rows = [CardId::Blind, CardId::Apotheosis, CardId::HandOfGreed];
    for (index, id) in card_rows.into_iter().enumerate() {
        assert_eq!(offer_rows.values[index * 2], vocabulary_row(&descriptor, "card", id.as_str()));
        assert_eq!(offer_rows.values[index * 2 + 1], index as i64);
        assert_eq!(offer_rows.known[index * 2..index * 2 + 2], [1, 1]);
        assert_eq!(offer_rows.occupied.as_ref().unwrap()[index], 1);
    }
    assert!(offer_rows.values[6..].iter().all(|value| *value == 0));
    assert!(offer_rows.known[6..].iter().all(|value| *value == 0));
    assert!(offer_rows.occupied.as_ref().unwrap()[3..].iter().all(|value| *value == 0));

    let actions = observation.group("actions").unwrap();
    let columns = actions.shape[1];
    assert_eq!(actions.values[0], vocabulary_row(&descriptor, "verb", "choose_card_offer"));
    assert_eq!(actions.values[1], vocabulary_row(&descriptor, "row_group", "combat_card_offers"));
    assert_eq!(actions.values[2], 1, "the selected index is the public offer row");
    assert_eq!(actions.values[columns], vocabulary_row(&descriptor, "verb", "skip_card_offer"));
    assert_eq!(observation.action_mask[..2], [1, 1]);
    assert!(observation.action_mask[2..].iter().all(|value| *value == 0));

    let cards = observation.group("combat_cards").unwrap();
    let damage_bonus = tensor(&descriptor, "combat_cards")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|field| field == "damage_bonus")
        .unwrap();
    assert_eq!(cards.values[damage_bonus], -3);
    assert_eq!(cards.known[damage_bonus], 1);
    let temporary_cost = tensor(&descriptor, "combat_cards")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|field| field == "temporary_cost")
        .unwrap();
    assert_eq!(cards.values[temporary_cost], -1);
    assert_eq!(cards.known[temporary_cost], 1);
    let powers = observation.group("powers").unwrap();
    let power_amount = tensor(&descriptor, "powers")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|field| field == "amount")
        .unwrap();
    assert_eq!(powers.values[power_amount], -1);
    assert_eq!(powers.known[power_amount], 1);
}

#[test]
fn m15_fresh_discovery_offer_uses_shared_combat_and_selects_its_public_index() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();
    let mut rng = spire_core::run::RunRng::new(
        [73; 32],
        spire_core::combat::rng::Partition::Test,
        0,
    );
    let entry = spire_core::combat::enemies::setup(
        EncounterId::Cultist,
        1,
        0,
        &mut rng,
        spire_core::combat::enemies::SetupContext {
            player: spire_core::combat::PlayerState {
                hp: 70,
                max_hp: 70,
                block: 0,
                energy: 3,
                max_energy: 3,
                draw_per_turn: 5,
                powers: vec![],
            },
            player_powers: vec![],
            character: Character::Silent,
            cards: vec![spire_core::combat::permanent_live_card(
                CardId::Discovery,
                CardInstanceId(1),
                0,
            )],
            opening_draw: 1,
            encounter_visit: 0,
        },
    )
    .unwrap();
    let mut inventory = Some(spire_core::items::CombatItemContext::new(
        vec![],
        vec![None; 3],
    ));
    let mut fight = spire_core::combat::Combat::enter_with_item_context(entry, &mut inventory)
        .unwrap();
    assert!(fight
        .legal_actions_full()
        .unwrap()
        .contains(&CombatAction::PlayCard {
            instance: 1,
            target: None,
        }));

    fight
        .step_full(&CombatAction::PlayCard {
            instance: 1,
            target: None,
        })
        .unwrap();
    let public = fight.public_full().unwrap();
    let offer = public.card_offer.clone().expect("Discovery publishes its current offer");
    assert_eq!(offer.cards.len(), 3);
    assert!(!offer.optional, "Discovery's offer is mandatory");
    let legal = fight.legal_actions_full().unwrap();
    for index in 0..3 {
        assert!(legal.contains(&CombatAction::ChooseCardOffer { index }));
    }
    assert!(!legal.contains(&CombatAction::SkipCardOffer));

    let mut decision = fixture_decision();
    decision.objects=vec![PublicObject{instance:1,id:CardId::Discovery,upgrades:0}];
    decision.combat = Some(public);
    decision.candidates = legal
        .iter()
        .enumerate()
        .map(|(index, action)| {
            candidate(
                index as u64 + 1,
                GameAction::Combat(action.clone()),
            )
        })
        .collect();
    let observation = encode_run_m15(&decision, None, &capacities).unwrap();
    let offer_rows = observation.group("combat_card_offers").unwrap();
    for (index, id) in offer.cards.iter().enumerate() {
        assert_eq!(offer_rows.values[index * 2], vocabulary_row(&descriptor, "card", id.as_str()));
        assert_eq!(offer_rows.values[index * 2 + 1], index as i64);
    }
    let actions = observation.group("actions").unwrap();
    let action_columns = actions.shape[1];
    for (row, action) in legal.iter().enumerate() {
        let CombatAction::ChooseCardOffer { index } = action else {
            panic!("mandatory Discovery offer has only indexed choices")
        };
        assert_eq!(actions.values[row * action_columns], vocabulary_row(&descriptor, "verb", "choose_card_offer"));
        assert_eq!(actions.values[row * action_columns + 1], vocabulary_row(&descriptor, "row_group", "combat_card_offers"));
        assert_eq!(actions.values[row * action_columns + 2], *index as i64);
    }

    let chosen = CombatAction::ChooseCardOffer { index: 1 };
    let generated = offer.cards[1];
    fight.step_full(&chosen).unwrap();
    let resumed = fight.public_full().unwrap();
    assert!(resumed.card_offer.is_none(), "the selected offer is consumed once");
    assert!(resumed.hand.iter().any(|card| {
        card.id == generated && matches!(card.origin, CombatCardRef::CombatLocal(_))
    }));
}

#[test]
fn m15_selection_operation_and_act4_public_combat_values_keep_their_named_domains() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();
    let operation_cases = [
        (SelectionOperation::Move(SelectionDisposition::Hand), "move_hand"),
        (SelectionOperation::Move(SelectionDisposition::Discard), "move_discard"),
        (SelectionOperation::Move(SelectionDisposition::Exhaust), "move_exhaust"),
        (SelectionOperation::Move(SelectionDisposition::DrawTop), "move_draw_top"),
        (SelectionOperation::Upgrade, "upgrade"),
        (SelectionOperation::Topdeck, "topdeck"),
        (SelectionOperation::ReturnFromExhaust, "return_from_exhaust"),
        (SelectionOperation::CopyToHand { copies: 1 }, "copy_to_hand_1"),
        (SelectionOperation::CopyToHand { copies: 2 }, "copy_to_hand_2"),
        (SelectionOperation::CopyToHand { copies: 3 }, "copy_to_hand_3"),
        (SelectionOperation::NightmareCapture, "nightmare_capture"),
        (SelectionOperation::Retain, "retain"),
        (SelectionOperation::Setup, "setup"),
        (SelectionOperation::Bottomdeck, "bottomdeck"),
        (SelectionOperation::RetrieveMemories, "retrieve_memories"),
    ];
    for (operation, name) in operation_cases {
        let card = live_card(CardId::Setup, 99, None);
        let mut view = combat();
        view.selection = Some(SelectionView {
            parent: SelectionParent::Card(
                spire_core::combat::CardHandle::parse("99").unwrap(),
            ),
            operation,
            candidates: vec![card],
            required: 1,
            selected: vec![],
            disposition: SelectionDisposition::Hand,
            source_zone: SelectionSourceZone::Hand,
            minimum: 0,
            maximum: 1,
        });
        let mut decision = fixture_decision();
        decision.combat = Some(view);
        let observation = encode_run_m15(&decision, None, &capacities).unwrap();
        let counts = observation.group("selection_counts").unwrap();
        assert_eq!(counts.values[5], vocabulary_row(&descriptor, "selection_operation", name));
        assert_eq!(counts.known[5], 1);
    }

    let mut shield_spear = combat();
    let left = spire_core::combat::EnemyHandle::parse("shield").unwrap();
    let right = spire_core::combat::EnemyHandle::parse("spear").unwrap();
    let mut shield = enemy(EnemyId::SpireShield, 0, "shield_bash");
    shield.instance_id = left.clone();
    let mut spear = enemy(EnemyId::SpireSpear, 1, "spear_skewer");
    spear.instance_id = right.clone();
    shield_spear.enemies = vec![shield, spear];
    shield_spear.player.powers.push(FullPowerView {
        id: FullPowerId("surrounded"),
        amount: 1,
    });
    shield_spear.encounter_facing = Some(EncounterFacingView {
        player_facing: PlayerFacing::Left,
        surrounded: true,
        enemies: vec![
            EncounterActorView {
                instance_id: left,
                side: EncounterSide::Left,
                back_attack: true,
            },
            EncounterActorView {
                instance_id: right,
                side: EncounterSide::Right,
                back_attack: false,
            },
        ],
    });
    let mut decision = fixture_decision();
    decision.combat = Some(shield_spear);
    let observation = encode_run_m15(&decision, None, &capacities).unwrap();
    let state = observation.group("combat_state").unwrap();
    assert_eq!(state.values[..2], [1, 1]);
    assert_eq!(state.known[..2], [1, 1]);
    assert_eq!(state.known[2..], [0, 0]);
    let enemies = observation.group("enemies").unwrap();
    let side_column = tensor(&descriptor, "enemies")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .position(|field| field == "side")
        .unwrap();
    let enemy_columns = enemies.shape[1];
    assert_eq!(enemies.values[side_column], 1);
    assert_eq!(enemies.values[enemy_columns + side_column], 2);

    let mut heart = combat();
    let mut heart_enemy = enemy(EnemyId::CorruptHeart, 0, "heart_echo");
    heart_enemy.powers.push(FullPowerView {
        id: FullPowerId("invincible"),
        amount: 217,
    });
    heart.enemies = vec![heart_enemy];
    let mut decision = fixture_decision();
    decision.combat = Some(heart);
    let observation = encode_run_m15(&decision, None, &capacities).unwrap();
    let state = observation.group("combat_state").unwrap();
    assert_eq!(state.values[2..], [217, 300]);
    assert_eq!(state.known[2..], [1, 1]);
}

#[test]
fn m15_rejects_padding_as_a_public_power_or_intent_name() {
    let capacities = RunCapacities::m10_default();

    let mut power_decision = fixture_decision();
    let mut power_combat = combat();
    power_combat.player.powers.push(FullPowerView {
        id: FullPowerId("padding"),
        amount: 1,
    });
    power_decision.combat = Some(power_combat);
    assert!(matches!(
        encode_run_m15(&power_decision, None, &capacities),
        Err(RunSessionError::Input(_))
    ));

    let mut intent_decision = fixture_decision();
    let mut intent_combat = combat();
    intent_combat
        .enemies
        .push(enemy(EnemyId::Cultist, 0, "padding"));
    intent_decision.combat = Some(intent_combat);
    assert!(matches!(
        encode_run_m15(&intent_decision, None, &capacities),
        Err(RunSessionError::Input(_))
    ));
}

#[test]
fn m15_capacity_unknown_and_old_profile_boundaries_reject_new_offer_state() {
    let capacities = RunCapacities::m10_default();
    let mut offer_decision = fixture_decision();
    let mut offer_combat = combat();
    offer_combat.card_offer = Some(CombatCardOfferView {
        cards: vec![CardId::Blind, CardId::Trip, CardId::HandOfGreed],
        optional: true,
    });
    offer_decision.combat = Some(offer_combat);
    offer_decision.candidates = vec![candidate(
        1,
        GameAction::Combat(CombatAction::ChooseCardOffer { index: 3 }),
    )];
    assert!(matches!(
        encode_run_m15(&offer_decision, None, &capacities),
        Err(RunSessionError::Input(_))
    ));

    let mut short = capacities.clone();
    short.offers = 2;
    let mut valid_offer = offer_decision.clone();
    valid_offer.candidates[0].action =
        GameAction::Combat(CombatAction::ChooseCardOffer { index: 1 });
    assert!(matches!(
        encode_run_m15(&valid_offer, None, &short),
        Err(RunSessionError::Capacity {
            group: "combat_card_offers",
            required: 3,
            capacity: 2
        })
    ));

    let mut unknown = fixture_decision();
    unknown.objects.push(PublicObject {
        instance: 1,
        id: CardId::Unknown("m15_unlisted_card"),
        upgrades: 0,
    });
    assert!(matches!(
        encode_run_m15(&unknown, None, &capacities),
        Err(RunSessionError::Input(_))
    ));

    let mut m14_decision = fixture_decision();
    m14_decision.content_profile = Some(RoomScenario::M14SilentAct3);
    m14_decision.act=3;
    m14_decision.floor=50;
    m14_decision.combat = offer_decision.combat;
    assert_eq!(
        encode_run_m14(&m14_decision, None, &capacities),
        Err(RunSessionError::UnsupportedBoundary(
            "M15 generated choices require M15 native profile"
        ))
    );
    assert_eq!(
        m14_run_observation_spec(&capacities).unwrap().descriptor()["schema_id"],
        RUN_OBSERVATION_M14_SCHEMA_ID
    );
}

#[test]
fn m15_room_event_choice_encounters_and_public_content_ids_resolve_to_descriptor_names() {
    let capacities = RunCapacities::m10_default();
    let descriptor = m15_run_observation_spec(&capacities).unwrap().descriptor();

    let events = [
        (spire_core::content::EventId::Neow, "neow"),
        (spire_core::content::EventId::UpgradeShrine, "upgrade_shrine"),
        (spire_core::content::EventId::Purifier, "purifier"),
        (spire_core::content::EventId::AncientWriting, "ancient_writing"),
        (spire_core::content::EventId::Joust, "joust"),
        (spire_core::content::EventId::Falling, "falling"),
        (spire_core::content::EventId::TombOfLordRedMask, "tomb_of_lord_red_mask"),
    ];
    for (event, name) in events {
        let mut decision = fixture_decision();
        decision.room = Some(RoomView {
            kind: RoomViewKind::Event(event),
            choices: vec![],
        });
        let observation = encode_run_m15(&decision, None, &capacities).unwrap();
        let context = observation.group("context").unwrap();
        let event_column = tensor(&descriptor, "context")["fields"]
            .as_array()
            .unwrap()
            .iter()
            .position(|field| field == "event_kind")
            .unwrap();
        assert_eq!(context.values[event_column], vocabulary_row(&descriptor, "event", name));
    }

    let encounters = [
        EncounterId::ShieldAndSpear,
        EncounterId::CorruptHeart,
    ];
    for encounter_id in encounters {
        let mut decision = fixture_decision();
        decision.last_combat = Some(CombatRecord {
            visit: 1,
            encounter: encounter_id,
            outcome: spire_core::combat::Outcome::Victory,
        });
        let observation = encode_run_m15(&decision, None, &capacities).unwrap();
        let context = observation.group("context").unwrap();
        let column = tensor(&descriptor, "context")["fields"]
            .as_array()
            .unwrap()
            .iter()
            .position(|field| field == "last_encounter")
            .unwrap();
        assert_eq!(
            context.values[column],
            vocabulary_row(&descriptor, "encounter", encounter_id.as_str())
        );
    }

    let relic = RelicId::TheSpecimen;
    let mut decision = fixture_decision();
    decision.player.relics = vec![relic];
    let observation = encode_run_m15(&decision, None, &capacities).unwrap();
    assert_eq!(
        observation.group("relics").unwrap().values[0],
        vocabulary_row(&descriptor, "relic", relic.as_str())
    );
    for potion in spire_core::run::M15_SILENT_HEART_POTIONS {
        let mut decision = fixture_decision();
        decision.player.potions = vec![Some(*potion)];
        let observation = encode_run_m15(&decision, None, &capacities).unwrap();
        assert_eq!(
            observation.group("potions").unwrap().values[0],
            vocabulary_row(&descriptor, "potion", potion.as_str()),
            "potion {potion:?}"
        );
    }
}

#[test]
fn m15_public_panache_countdown_and_independent_bomb_timers_keep_rows_and_capacity() {
    let capacities=RunCapacities::m10_default();
    let descriptor=m15_run_observation_spec(&capacities).unwrap().descriptor();
    let mut decision=fixture_decision();
    let mut public=combat();
    let entries=[("panache",10),("panache_countdown",4),("the_bomb_plus",2),("the_bomb",3),("the_bomb",1)];
    public.player.powers=entries.iter().map(|(name,amount)|FullPowerView{id:FullPowerId(name),amount:*amount}).collect();
    decision.combat=Some(public);
    let observation=encode_run_m15(&decision,None,&capacities).unwrap();
    let powers=observation.group("powers").unwrap();
    for (index,(name,amount)) in entries.iter().enumerate() {
        assert_eq!(&powers.values[index*4..index*4+4],&[1,0,vocabulary_row(&descriptor,"power",name),i64::from(*amount)]);
        assert_eq!(&powers.known[index*4..index*4+4],&[1;4]);
        assert_eq!(powers.occupied.as_ref().unwrap()[index],1);
    }
    assert!(powers.occupied.as_ref().unwrap()[5..].iter().all(|value|*value==0));
    let mut small=capacities.clone();small.powers=4;
    assert_eq!(encode_run_m15(&decision,None,&small),Err(RunSessionError::Capacity{group:"powers",required:5,capacity:4}));
    let countdowns=&descriptor["semantics"]["power_countdowns"];
    assert_eq!(countdowns["panache_countdown"]["minimum"],1);
    assert_eq!(countdowns["panache_countdown"]["maximum"],5);
    assert_eq!(countdowns["the_bomb"]["damage"],40);
    assert_eq!(countdowns["the_bomb_plus"]["damage"],50);
    assert_eq!(countdowns["the_bomb"]["maximum"],3);
    assert_eq!(countdowns["the_bomb_plus"]["maximum"],3);
    assert_eq!(countdowns["bomb_rows"],"one row per independent timer, in existing creation/callback order; repeated power IDs do not aggregate or sort by damage");
    decision.content_profile=Some(RoomScenario::M14SilentAct3);decision.act=3;decision.floor=50;
    for name in ["panache_countdown","the_bomb","the_bomb_plus"] {
        decision.combat.as_mut().unwrap().player.powers=vec![FullPowerView{id:FullPowerId(name),amount:1}];
        assert!(matches!(encode_run_m14(&decision,None,&capacities),Err(RunSessionError::Input(_))));
    }
}


#[test]
fn m15_review_known_draw_rows_share_combat_card_capacity() {
    let mut decision=fixture_decision();let mut view=combat();view.draw_count=1;
    let card=live_card(CardId::Deflect,1,None);
    view.known_draw=vec![spire_core::combat::KnownDrawCardView{card:card.clone(),from_top:true,offset:0},spire_core::combat::KnownDrawCardView{card,from_top:false,offset:0}];
    decision.combat=Some(view);let mut caps=RunCapacities::m10_default();caps.combat_cards=1;
    assert_eq!(encode_run_m15(&decision,None,&caps),Err(RunSessionError::Capacity{group:"known_draw",required:2,capacity:1}),"two entitled aliases use the combat_cards limit while only one physical card is emitted");
    caps.combat_cards=2;assert!(encode_run_m15(&decision,None,&caps).is_ok());
}
