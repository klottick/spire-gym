use super::super::*;
use crate::{
    combat::{self, rng::Partition, CardInstanceId, CombatAction},
    content::{CardId, Character, EncounterId},
    items::CombatItemContext,
    run::RunRng,
};

fn pool_test_items(character: Character) -> CombatItemContext {
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    match character {
        Character::Ironclad => items.with_complete_cards(),
        Character::Silent => items.with_m15_profile().with_complete_cards(),
        Character::Defect => items.with_m20_profile().with_complete_cards(),
        Character::Watcher => items.with_m25_profile().with_complete_cards(),
    }
}

fn pool_test_combat(character: Character, card: CardId, seed: u8) -> Combat {
    let live = combat::complete_live_card(card, CardInstanceId(1), 0)
        .unwrap_or_else(|error| panic!("{character:?} {card:?}: {error:?}"));
    let entry = combat::enemies::setup(
        EncounterId::Cultist,
        1,
        0,
        &mut RunRng::new([seed; 32], Partition::Test, 0),
        combat::enemies::SetupContext {
            player: PlayerState {
                hp: 80, max_hp: 80, block: 0, energy: 9, max_energy: 9,
                draw_per_turn: 5, powers: vec![],
            },
            player_powers: vec![],
            character,
            cards: vec![live],
            opening_draw: 1,
            encounter_visit: 0,
        },
    ).expect("complete single-card Cultist setup");
    let mut items = Some(pool_test_items(character));
    Combat::enter_with_item_context(entry, &mut items)
        .unwrap_or_else(|error| panic!("{character:?} {card:?} entry: {error:?}"))
}

fn combat_local_cards(combat: &Combat) -> Vec<LiveCard> {
    use crate::combat::model::OwnedCard;
    combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full)
        .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_)))
        .cloned()
        .collect()
}

#[test]
fn m26_pools_infernal_blade_player_pool() {
    use crate::content::{
        pools::{cards, CardPool, GenerationPool},
        CardColor, CardType, Catalog,
    };

    let characters = [
        (Character::Ironclad, CardColor::Red),
        (Character::Silent, CardColor::Green),
        (Character::Defect, CardColor::Blue),
        (Character::Watcher, CardColor::Purple),
    ];
    for (character, color) in characters {
        let expected = cards(CardPool::Combat { character, pool: GenerationPool::Attack })
            .unwrap_or_else(|error| panic!("{character:?} Attack pool: {error:?}"));
        for seed in 1..=200u8 {
            let mut combat = pool_test_combat(character, CardId::InfernalBlade, seed);
            let parent = combat.public_full().unwrap().hand.into_iter()
                .find(|card| card.id == CardId::InfernalBlade)
                .expect("opening hand contains Infernal Blade").instance.0;
            combat.step_full(&CombatAction::PlayCard { instance: parent, target: None })
                .unwrap_or_else(|error| panic!("{character:?} seed {seed} Infernal Blade: {error:?}"));
            let generated = combat.public_full().unwrap().hand.into_iter()
                .find(|card| card.instance.0 != parent)
                .unwrap_or_else(|| panic!("{character:?} seed {seed} has a generated card in hand"));
            assert!(expected.contains(&generated.id), "{character:?} seed {seed}: {:?}", generated.id);
            let row = Catalog::standard().card(generated.id).expect("generated catalog row");
            assert_eq!(row.color, color, "{character:?} seed {seed}");
            assert_eq!(row.card_type, CardType::Attack, "{character:?} seed {seed}");
        }
    }

    let expected = cards(CardPool::Combat {
        character: Character::Ironclad, pool: GenerationPool::Attack,
    }).unwrap();
    assert_eq!(crate::run::rooms::m10_content::infernal_candidates(), expected.as_slice(),
        "the frozen M10 helper remains the shared Ironclad Attack pool order");
}

#[test]
fn m26_pools_generators_all_characters() {
    use crate::content::pools::{cards, CardPool, GenerationPool};

    for character in [Character::Ironclad, Character::Watcher] {
        for (index, (generator, expected_pool)) in [
            (CardId::WhiteNoise, GenerationPool::Power),
            (CardId::Distraction, GenerationPool::Skill),
            (CardId::Discovery, GenerationPool::Any),
            (CardId::Chrysalis, GenerationPool::Skill),
            (CardId::Metamorphosis, GenerationPool::Attack),
        ].into_iter().enumerate() {
            let mut combat = pool_test_combat(character, generator, 30 + index as u8);
            combat.step_full(&CombatAction::PlayCard { instance: 1, target: None })
                .unwrap_or_else(|error| panic!("{character:?} {generator:?}: {error:?}"));
            if generator == CardId::Discovery {
                combat.step_full(&CombatAction::ChooseCardOffer { index: 0 })
                    .unwrap_or_else(|error| panic!("{character:?} Discovery first offer: {error:?}"));
            }
            let generated = combat_local_cards(&combat);
            assert!(!generated.is_empty(), "{character:?} {generator:?} creates cards");
            let expected = cards(CardPool::Combat { character, pool: expected_pool })
                .unwrap_or_else(|error| panic!("{character:?} {expected_pool:?}: {error:?}"));
            for card in generated {
                assert!(expected.contains(&card.id),
                    "{character:?} {generator:?} generated {:?} outside {expected_pool:?}", card.id);
            }
        }
    }

    for character in [Character::Ironclad, Character::Silent] {
        for (power, pool) in [
            (CardId::HelloWorld, GenerationPool::Common),
            (CardId::CreativeAI, GenerationPool::Power),
        ] {
            let mut combat = pool_test_combat(character, power, 60);
            combat.step_full(&CombatAction::PlayCard { instance: 1, target: None })
                .unwrap_or_else(|error| panic!("{character:?} {power:?}: {error:?}"));
            combat.step_full(&CombatAction::EndTurn)
                .unwrap_or_else(|error| panic!("{character:?} {power:?} next turn: {error:?}"));
            let generated = combat_local_cards(&combat);
            assert_eq!(generated.len(), 1, "{character:?} {power:?} creates one card at turn start");
            let expected = cards(CardPool::Combat { character, pool })
                .unwrap_or_else(|error| panic!("{character:?} {pool:?}: {error:?}"));
            assert!(expected.contains(&generated[0].id),
                "{character:?} {power:?} generated {:?} outside {pool:?}", generated[0].id);
        }
    }
}

// M26 disclosed shared fix (found by `m26_pools_infernal_blade_player_pool`): Clash's base
// cost came from its payload, which rejects while a non-Attack is in hand, so a plain
// (frozen-M10-shaped) Infernal Blade faulted whenever it rolled Clash beside a Skill.
#[test]
fn m26_infernal_blade_clash_beside_a_skill() {
    let mut clashes = 0;
    for seed in 1..=120u8 {
        let blade = combat::complete_live_card(CardId::InfernalBlade, CardInstanceId(1), 0).unwrap();
        let defend = crate::combat::card_state::permanent(CardId::Defend, CardInstanceId(2), 0);
        let entry = combat::enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([seed; 32], Partition::Test, 0),
            combat::enemies::SetupContext {
                player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 9, max_energy: 9, draw_per_turn: 5, powers: vec![] },
                player_powers: vec![], character: Character::Ironclad, cards: vec![blade, defend], opening_draw: 2, encounter_visit: 0,
            }).unwrap();
        let mut items = Some(CombatItemContext::new(vec![], vec![None; 3]));
        let mut combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
        combat.step_full(&CombatAction::PlayCard { instance: 1, target: None })
            .unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        let made = combat.public_full().unwrap().hand.into_iter().find(|card| card.instance.0 > 2).expect("generated attack");
        if made.id == CardId::Clash {
            clashes += 1;
            assert_eq!(made.effective_cost, Some(0), "seed {seed}: Clash shows its printed cost while unplayable");
        }
    }
    assert!(clashes >= 1, "the seed range covers a Clash roll (seeds 38, 78, 85, 87, 111)");
}

// M26 disclosed shared fix: `card_state::generated` builds a Watcher kind through its own
// M25 initializer, the same fields every M25 constructor produces.
#[test]
fn m26_generated_watcher_kinds_match_their_m25_constructor() {
    for row in crate::content::Catalog::standard().cards() {
        if row.id == CardId::Miracle || !crate::content::watcher::supports_m25_generated(row.id) { continue }
        for upgrades in 0..=1 {
            let generated = crate::combat::card_state::generated(row.id, CardInstanceId(7), upgrades)
                .unwrap_or_else(|| panic!("{:?}+{upgrades} generated constructor", row.id));
            let reference = combat::m25_watcher_live_card(row.id, CardInstanceId(7), upgrades).unwrap();
            assert_eq!(generated.local, reference.local, "{:?}+{upgrades} combat-local fields", row.id);
        }
    }
}
