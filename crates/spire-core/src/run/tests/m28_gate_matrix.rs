//! M28f2 gate audit coverage: character-sensitive event producers, N'loth sampling, and Act 2/3 laws.
use super::super::*;
use crate::{
    combat::rng::Partition,
    content::{CardColor, CardId, CardRarity, Character, EventId, PotionId, RelicId, RelicTier},
    items::{potion_generation, relic_pools},
    run::rooms::standard::{rewards, treasure, shop::ShopSupply},
};

const CHARACTERS: [Character; 4] = [
    Character::Ironclad,
    Character::Silent,
    Character::Defect,
    Character::Watcher,
];

const SHARED_POTION_POOL: [PotionId; 30] = [
    PotionId::FirePotion, PotionId::FruitJuice, PotionId::BlockPotion, PotionId::EnergyPotion,
    PotionId::StrengthPotion, PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion,
    PotionId::DexterityPotion, PotionId::AncientPotion, PotionId::SmokeBomb, PotionId::AttackPotion,
    PotionId::SkillPotion, PotionId::PowerPotion, PotionId::ColorlessPotion, PotionId::FlexPotion,
    PotionId::SpeedPotion, PotionId::RegenPotion, PotionId::LiquidBronze, PotionId::EssenceOfSteel,
    PotionId::CultistPotion, PotionId::BlessingOfTheForge, PotionId::SneckoOil, PotionId::FairyPotion,
    PotionId::ExplosivePotion, PotionId::GamblersBrew, PotionId::LiquidMemories,
    PotionId::DuplicationPotion, PotionId::DistilledChaos, PotionId::EntropicBrew,
];

fn class_potions(character: Character) -> [PotionId; 3] {
    match character {
        Character::Ironclad => [PotionId::BloodPotion, PotionId::Elixir, PotionId::HeartOfIron],
        Character::Silent => [PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar],
        Character::Defect => [PotionId::FocusPotion, PotionId::PotionOfCapacity, PotionId::EssenceOfDarkness],
        Character::Watcher => [PotionId::BottledMiracle, PotionId::StancePotion, PotionId::Ambrosia],
    }
}

fn character_card_color(character: Character) -> CardColor {
    match character {
        Character::Ironclad => CardColor::Red,
        Character::Silent => CardColor::Green,
        Character::Defect => CardColor::Blue,
        Character::Watcher => CardColor::Purple,
    }
}

fn standard(character: Character, seed: u8) -> Run {
    Run::new_room_scenario(
        RunConfig { character, ascension: 0 },
        CreationProfile::experienced(),
        RunRng::new([seed; 32], Partition::Test, 0),
        RoomScenario::StandardA0,
    ).unwrap()
}

fn assert_character_card(character: Character, item: &str, id: CardId) {
    let definition = crate::content::Catalog::standard().card(id)
        .unwrap_or_else(|| panic!("{item} / {character:?}: {id:?} is a registered card"));
    assert_eq!(definition.color, character_card_color(character),
        "{item} / {character:?}: {id:?} belongs to this character's literal color pool");
    assert!(matches!(definition.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare),
        "{item} / {character:?}: {id:?} belongs to the literal ordinary pool");
}

fn assert_character_relic(character: Character, item: &str, id: RelicId) {
    assert_eq!(crate::content::Catalog::standard().relic_eligible_for(id, character), Some(true),
        "{item} / {character:?}: {id:?} is not another character's class relic");
}

fn assert_character_potion(character: Character, item: &str, id: PotionId) {
    assert!(crate::content::Catalog::standard().potion(id).is_some(), "{item}: {id:?} is registered");
    assert!(SHARED_POTION_POOL.contains(&id) || class_potions(character).contains(&id),
        "{item} / {character:?}: {id:?} belongs to this character's literal 33-potion pool");
    for other in CHARACTERS {
        if other != character {
            assert!(!class_potions(other).contains(&id),
                "{item} / {character:?}: {id:?} never comes from {other:?}'s class potion pool");
        }
    }
}

fn choose_event(run: &mut Run, stage: u8, option: u8) {
    let id = super::m28_event_core::view(run).choices.into_iter().find(|choice| match choice.kind {
        RoomChoiceKind::EventOption { stage: s, option: o }
        | RoomChoiceKind::EventPotion { stage: s, option: o, .. }
        | RoomChoiceKind::EventCard { stage: s, option: o, .. }
        | RoomChoiceKind::EventRelic { stage: s, option: o, .. } => s == stage && o == option,
        _ => false,
    }).unwrap_or_else(|| panic!("no event choice at stage {stage}, option {option}")).id;
    run.step(&GameAction::Choose(id)).unwrap();
}

fn event_at_floor(character: Character, event: EventId, seed: u8, floor: u32) -> Run {
    let (mut run, node) = super::m28_event_core::before_event(character, 1, event, seed);
    run.state.floor = floor - 1;
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = floor;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(super::m28_event_core::view(&run).kind, RoomViewKind::Event(event));
    run
}

fn set_event_stage(run: &mut Run, stage: u8) {
    run.state.room_loop.as_mut().unwrap().standard_event.as_mut().unwrap().stage = stage;
    crate::run::rooms::standard::event_frame::publish(run).unwrap();
}

fn enter_with(character: Character, act: u8, event: EventId, seed: u8, setup: impl FnOnce(&mut Run)) -> Run {
    let (mut run, node) = super::m28_event_core::before_event(character, act, event, seed);
    setup(&mut run);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(super::m28_event_core::view(&run).kind, RoomViewKind::Event(event));
    run
}

fn set_deck(run: &mut Run, ids: &[CardId]) {
    let first = run.state.deck.first().map_or(90_000, |card| card.instance + 100);
    run.state.deck = ids.iter().enumerate().map(|(index, id)| crate::run::CardInstance {
        id: *id, instance: first + index as u64, upgrades: 0, misc: 0,
    }).collect();
    run.state.relic_cards.clear();
}

fn select_instance(run: &mut Run, instance: u64) {
    let candidate = run.state.pending.as_ref().unwrap().selection.candidates.iter()
        .find(|candidate| candidate.card.instance == instance).unwrap();
    run.step(&GameAction::Select(candidate.id)).unwrap();
}

fn wheel_relic_arm(character: Character) -> Run {
    for seed in 0..100_u8 {
        let (mut run, node) = super::m28_event_core::before_event(character, 1, EventId::WheelOfChange, seed);
        let max_hp = super::m28_event_core::base(character);
        run.state.max_hp = max_hp;
        run.state.hp = max_hp;
        run.step(&GameAction::Move(node)).unwrap();
        choose_event(&mut run, 0, 0);
        let result = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.values[2];
        if result == 1 { return run; }
    }
    panic!("Wheel index 1 not reached for {character:?}");
}

fn colosseum(character: Character, seed: u8) -> Run {
    let (mut run, _) = super::m28_event_core::before_event(character, 2, EventId::Colosseum, seed);
    let map = run.state.map.as_mut().unwrap();
    let from = map.nodes.iter().find(|node| node.y == 7 && !node.edges.is_empty()).unwrap().clone();
    let node = from.edges[0];
    map.nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().kind = MapNodeKind::Event;
    map.current = Some(from.id);
    run.state.floor = from.floor;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(super::m28_event_core::view(&run).kind, RoomViewKind::Event(EventId::Colosseum));
    run
}

fn option_row(run: &mut Run, stage: u8, option: u8) {
    choose_event(run, stage, option);
}

fn nloth_offered_relic(run: &Run, option: u8) -> RelicId {
    super::m28_event_core::view(run).choices.into_iter().find_map(|choice| match choice.kind {
        RoomChoiceKind::EventRelic { stage: 0, option: row_option, relic } if row_option == option => relic,
        _ => None,
    }).unwrap_or_else(|| panic!("N'loth option {option} did not show an offered relic"))
}

fn shop_rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_be_bytes());
    RunRng::new(root, Partition::Test, seed)
}

fn diagnostic_shop(character: Character, seed: u64, supply: ShopSupply) -> Run {
    crate::run::rooms::standard::shop::diagnostic_shop(character, supply, shop_rng(seed)).unwrap()
}

fn reenter_shop(template: &Run, seed: u64) -> Run {
    let mut run = template.clone();
    run.state.rng = shop_rng(seed);
    run.state.relic_pools = Some(relic_pools::build(&run.state, &[]).unwrap());
    crate::run::rooms::standard::shop::enter(&mut run).unwrap();
    run
}

fn shop_rows(run: &Run) -> &[RoomChoiceView] {
    let room = run.state.room_loop.as_ref().unwrap();
    assert_eq!(room.screen.kind, RoomViewKind::Shop);
    &room.screen.choices
}

fn shop_card_id(row: &RoomChoiceView) -> CardId {
    match row.kind { RoomChoiceKind::Card { id, .. } => id, ref other => panic!("expected card row, got {other:?}") }
}

fn shop_relic_id(row: &RoomChoiceView) -> RelicId {
    match row.kind { RoomChoiceKind::Relic(id) => id, ref other => panic!("expected relic row, got {other:?}") }
}

fn shop_potion_id(row: &RoomChoiceView) -> PotionId {
    match row.kind { RoomChoiceKind::Potion(id) => id, ref other => panic!("expected potion row, got {other:?}") }
}

fn shop_card_rows(run: &Run) -> Vec<&RoomChoiceView> {
    shop_rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })
        && crate::content::Catalog::standard().card(shop_card_id(row)).is_some_and(|card| card.color != crate::content::CardColor::Colorless)).collect()
}

fn shop_relic_rows(run: &Run) -> Vec<&RoomChoiceView> {
    shop_rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).collect()
}

fn shop_potion_rows(run: &Run) -> Vec<&RoomChoiceView> {
    shop_rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).collect()
}

fn shop_price_range(price: u32, (low, high): (u32, u32)) {
    assert!((low..=high).contains(&price), "price {price} outside {low}..={high}");
}

fn shop_card_bounds(rarity: CardRarity) -> (u32, u32) {
    match rarity {
        CardRarity::Common => (45, 55),
        CardRarity::Uncommon => (67, 82),
        CardRarity::Rare => (135, 165),
        other => panic!("shop card had unsupported rarity {other:?}"),
    }
}

fn shop_relic_bounds(tier: RelicTier) -> (u32, u32) {
    match tier {
        RelicTier::Common | RelicTier::Shop => (143, 158),
        RelicTier::Uncommon => (238, 263),
        RelicTier::Rare => (285, 315),
        other => panic!("shop relic had unsupported tier {other:?}"),
    }
}

fn shop_potion_bounds(rarity: crate::content::PotionRarity) -> (u32, u32) {
    match rarity {
        crate::content::PotionRarity::Common => (48, 52),
        crate::content::PotionRarity::Uncommon => (71, 79),
        crate::content::PotionRarity::Rare => (95, 105),
    }
}

fn economy_pools_without_owned(run: &mut Run) {
    run.state.relic_pools = Some(relic_pools::build(&run.state, &[]).unwrap());
}

fn buy_woman_potions(character: Character, option: u8, amount: usize, price: u32, seed: u8) {
    let mut run = enter_with(character, 1, EventId::WomanInBlue, seed, |run| run.state.gold = 100);
    for (option, expected_price) in [(0, 20), (1, 30), (2, 40)] {
        assert_eq!(super::m28_event_core::row(&run, &RoomChoiceKind::EventOption { stage: 0, option }).price, Some(expected_price));
    }
    choose_event(&mut run, 0, option);
    assert_eq!(run.state.gold, 100 - price, "{character:?}");
    let rows = super::m28_event_core::view(&run);
    assert_eq!(rows.kind, RoomViewKind::Rewards);
    let potions: Vec<_> = rows.choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Potion(id) => Some((choice.id, id)), _ => None,
    }).collect();
    assert_eq!(potions.len(), amount, "{character:?}");
    for (choice, potion) in potions {
        assert_character_potion(character, "Woman in Blue purchase", potion);
        run.step(&GameAction::Choose(choice)).unwrap();
        assert!(run.state.potions.iter().any(|slot| *slot == Some(potion)), "{character:?}: bought potion is obtained");
    }
}

fn newly_obtained_relics(before: &[RelicId], run: &Run) -> Vec<RelicId> {
    run.state.relics.iter().copied().filter(|id| !before.contains(id)).collect()
}

fn assert_reward_card_pool(run: &Run, character: Character, item: &str) {
    let mut probe = run.clone();
    let reward = super::m28_event_core::row(&probe, &RoomChoiceKind::CardReward).id;
    probe.step(&GameAction::Choose(reward)).unwrap();
    let screen = super::m28_event_core::view(&probe);
    assert_eq!(screen.kind, RoomViewKind::CardReward, "{item} opens its EventRoom card offer");
    let cards: Vec<_> = screen.choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Card { id, .. } => Some(id),
        _ => None,
    }).collect();
    assert_eq!(cards.len(), 3, "{item}: three ordinary event-combat cards");
    for id in cards { assert_character_card(character, item, id); }
}

fn assert_combat_reward_pools(run: &Run, character: Character, item: &str, require_potion: bool) {
    let screen = super::m28_event_core::view(run);
    assert_eq!(screen.kind, RoomViewKind::Rewards, "{item} reward screen");
    let relics: Vec<_> = screen.choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Relic(id) => Some(id),
        _ => None,
    }).collect();
    assert!(!relics.is_empty(), "{item}: relic reward is present");
    for id in relics { assert_character_relic(character, item, id); }
    let potions: Vec<_> = screen.choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Potion(id) => Some(id),
        _ => None,
    }).collect();
    if require_potion { assert_eq!(potions.len(), 1, "{item}: one event-combat potion offer"); }
    for id in potions { assert_character_potion(character, item, id); }
    assert_eq!(screen.choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward).count(), 1,
        "{item}: one EventRoom card reward row");
    assert_reward_card_pool(run, character, item);
}

#[test]
fn m28_e0_ordinary_card_reward_pool_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = standard(character, 0x41 + index as u8);
        run.state.card_rarity_offset = Some(100);
        let offers = rewards::ordinary_offer(&mut run.state, rewards::RarityRoom::Ordinary, 3, 1, &[0xCA]).unwrap();
        assert_eq!(offers.len(), 3, "{character:?}");
        for (id, _) in offers { assert_character_card(character, "ordinary card reward", id); }
    }
}

#[test]
fn m28_e0_library_choose_a_card_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 2, EventId::Library, 0x51 + index as u8);
        let before_len = run.state.deck.len();
        choose_event(&mut run, 0, 0);
        let ids = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.card_ids.clone();
        assert_eq!(ids.len(), 20, "{character:?}");
        assert_eq!(ids.iter().collect::<std::collections::BTreeSet<_>>().len(), 20, "{character:?}: distinct offers");
        for id in &ids { assert_character_card(character, "The Library", *id); }
        let selected = 11_u8;
        let expected = ids[usize::from(selected)];
        let choice = super::m28_event_core::view(&run).choices.into_iter().find(|choice| matches!(choice.kind,
            RoomChoiceKind::EventCard { stage: 1, option, card: Some(id) } if option == selected && id == expected)).unwrap();
        run.step(&GameAction::Choose(choice.id)).unwrap();
        assert_eq!(run.state.deck.len(), before_len + 1, "{character:?}");
        assert!(run.state.deck.iter().any(|card| card.id == expected), "{character:?}: selected card is obtained");
        assert_character_card(character, "The Library obtained card", expected);
    }
}

#[test]
fn m28_e0_big_fish_box_random_tier_relic_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 1, EventId::BigFish, 0x61 + index as u8);
        let before = run.state.relics.clone();
        choose_event(&mut run, 0, 2);
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: Box obtains one relic");
        assert_character_relic(character, "Big Fish Box", gained[0]);
        assert_eq!(run.state.deck.iter().filter(|card| card.id == CardId::Regret).count(), 1, "{character:?}");
    }
}

#[test]
fn m28_e0_dead_adventurer_search_relic_token_all_characters() {
    for character in CHARACTERS {
        let mut found = false;
        for seed in 0..=u8::MAX {
            let mut run = event_at_floor(character, EventId::DeadAdventurer, seed, 7);
            for stage in 0..3_u8 {
                let before = run.state.relics.clone();
                choose_event(&mut run, stage, 0);
                let gained = newly_obtained_relics(&before, &run);
                if !gained.is_empty() {
                    assert_eq!(gained.len(), 1, "{character:?}: one relic token");
                    assert_character_relic(character, "Dead Adventurer search RELIC token", gained[0]);
                    found = true;
                    break;
                }
                if run.state.active_combat.is_some() || super::m28_event_core::view(&run).kind != RoomViewKind::Event(EventId::DeadAdventurer) {
                    break;
                }
            }
            if found { break; }
        }
        assert!(found, "{character:?}: fixed seed range reaches the safe RELIC token");
    }
}

#[test]
fn m28_e0_wheel_of_change_relic_candidates_all_characters() {
    for character in CHARACTERS {
        let mut run = wheel_relic_arm(character);
        choose_event(&mut run, 1, 1);
        let relics: Vec<_> = super::m28_event_core::view(&run).choices.iter().filter_map(|choice| match choice.kind {
            RoomChoiceKind::Relic(id) => Some(id), _ => None,
        }).collect();
        assert_eq!(relics.len(), 1, "{character:?}: random-tier reward");
        assert_character_relic(character, "Wheel of Change relic reward", relics[0]);
    }
}

#[test]
fn m28_e0_scrap_ooze_reach_relic_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let seed = 0x71 + index as u8;
        let mut run = super::m28_event_core::at_event(character, 1, EventId::ScrapOoze, seed);
        set_event_stage(&mut run, 8);
        run.state.rng = super::m28_event_core::rng(seed);
        let before = run.state.relics.clone();
        choose_event(&mut run, 8, 0);
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: reach eight is guaranteed success");
        assert_character_relic(character, "Scrap Ooze Reach", gained[0]);
    }
}

#[test]
fn m28_e0_addict_pay_relic_candidates_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 2, EventId::Addict, 0x81 + index as u8);
        run.state.gold = 85;
        let before = run.state.relics.clone();
        choose_event(&mut run, 0, 0);
        assert_eq!(run.state.gold, 0, "{character:?}: Pay spends 85");
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: Pay obtains one relic");
        assert_character_relic(character, "The Addict Pay", gained[0]);
    }
}

#[test]
fn m28_e0_addict_steal_relic_candidates_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 2, EventId::Addict, 0x91 + index as u8);
        let before = run.state.relics.clone();
        choose_event(&mut run, 0, 1);
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: Steal obtains one relic");
        assert_character_relic(character, "The Addict Steal", gained[0]);
    }
}

#[test]
fn m28_e0_mausoleum_open_random_tier_relic_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 2, EventId::Mausoleum, 0xA1 + index as u8);
        let before = run.state.relics.clone();
        choose_event(&mut run, 0, 0);
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: Open obtains one random-tier relic");
        assert_character_relic(character, "The Mausoleum Open", gained[0]);
    }
}

#[test]
fn m28_e0_we_meet_again_give_potion_relic_candidate_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = enter_with(character, 2, EventId::WeMeetAgain, 0xB1 + index as u8, |run| {
            run.state.potions = vec![None, Some(PotionId::FirePotion), None];
        });
        let before = run.state.relics.clone();
        choose_event(&mut run, 0, 0);
        assert_eq!(run.state.potions, vec![None, None, None], "{character:?}: gift potion is removed");
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: potion gift obtains one relic");
        assert_character_relic(character, "We Meet Again Give Potion", gained[0]);
    }
}

#[test]
fn m28_e0_we_meet_again_give_gold_relic_candidate_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = enter_with(character, 2, EventId::WeMeetAgain, 0xC1 + index as u8, |run| run.state.gold = 120);
        let before_gold = run.state.gold;
        let amount = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.values[1];
        assert!((50..=120).contains(&amount), "{character:?}: literal G=120 amount range");
        let before = run.state.relics.clone();
        choose_event(&mut run, 0, 1);
        assert_eq!(run.state.gold, before_gold - amount, "{character:?}");
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: gold gift obtains one relic");
        assert_character_relic(character, "We Meet Again Give Gold", gained[0]);
    }
}

#[test]
fn m28_e0_we_meet_again_give_card_relic_candidate_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = enter_with(character, 2, EventId::WeMeetAgain, 0xD1 + index as u8, |run| {
            set_deck(run, &[CardId::IronWave, CardId::StrikeBlue]);
        });
        let before = run.state.relics.clone();
        let given = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.cards[0];
        choose_event(&mut run, 0, 2);
        assert!(!run.state.deck.iter().any(|card| card.instance == given), "{character:?}: selected gift card is removed");
        let gained = newly_obtained_relics(&before, &run);
        assert_eq!(gained.len(), 1, "{character:?}: card gift obtains one relic");
        assert_character_relic(character, "We Meet Again Give Card", gained[0]);
    }
}

#[test]
fn m28_e0_transmogrifier_transform_pool_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 1, EventId::Transmogrifier, 0xE1 + index as u8);
        let before_instances: std::collections::BTreeSet<_> = run.state.deck.iter().map(|card| card.instance).collect();
        choose_event(&mut run, 0, 0);
        let instance = run.state.pending.as_ref().unwrap().selection.candidates[0].card.instance;
        select_instance(&mut run, instance);
        let transformed: Vec<_> = run.state.deck.iter().filter(|card| !before_instances.contains(&card.instance)).map(|card| card.id).collect();
        assert_eq!(transformed.len(), 1, "{character:?}: one transform result");
        assert_character_card(character, "Transmogrifier", transformed[0]);
    }
}

#[test]
fn m28_e0_drug_dealer_transform_two_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = enter_with(character, 2, EventId::DrugDealer, 0xF1 + index as u8, |_| {});
        let before_instances: std::collections::BTreeSet<_> = run.state.deck.iter().map(|card| card.instance).collect();
        choose_event(&mut run, 0, 1);
        let candidates = run.state.pending.as_ref().unwrap().selection.candidates.clone();
        assert!(candidates.len() >= 2, "{character:?}: at least two selectable cards");
        for instance in [candidates[0].card.instance, candidates[1].card.instance] {
            select_instance(&mut run, instance);
        }
        let transformed: Vec<_> = run.state.deck.iter().filter(|card| !before_instances.contains(&card.instance)).map(|card| card.id).collect();
        assert_eq!(transformed.len(), 2, "{character:?}: two transform results");
        for id in transformed { assert_character_card(character, "Drug Dealer Transform Two", id); }
    }
}

#[test]
fn m28_e0_woman_in_blue_buy_two_potions_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        buy_woman_potions(character, 1, 2, 30, 0x21 + index as u8);
    }
}

#[test]
fn m28_e0_woman_in_blue_buy_three_potions_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        buy_woman_potions(character, 2, 3, 40, 0x31 + index as u8);
    }
}

#[test]
fn m28_e0_knowing_skull_potion_pool_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = super::m28_event_core::at_event(character, 2, EventId::KnowingSkull, 0x41 + index as u8);
        run.state.potions = vec![None; 3];
        let before = run.state.potions.clone();
        choose_event(&mut run, 0, 0);
        let gained: Vec<_> = run.state.potions.iter().flatten().copied().filter(|id| !before.contains(&Some(*id))).collect();
        assert_eq!(gained.len(), 1, "{character:?}: Pick Me Up obtains one potion into a free slot");
        assert_character_potion(character, "Knowing Skull Pick Me Up", gained[0]);
    }
}

#[test]
fn m28_e0_dead_adventurer_ambush_rewards_all_characters() {
    for character in CHARACTERS {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = event_at_floor(character, EventId::DeadAdventurer, seed, 7);
            choose_event(&mut run, 0, 0);
            if run.state.active_combat.is_none() { continue; }
            super::m28_world::win(&mut run);
            if super::m28_event_core::view(&run).choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Potion(_))) {
                found = Some(run);
                break;
            }
        }
        let run = found.unwrap_or_else(|| panic!("{character:?}: no ambush seed produced its event-combat potion row"));
        assert_combat_reward_pools(&run, character, "Dead Adventurer ambush", true);
    }
}

#[test]
fn m28_e0_mushrooms_fight_rewards_all_characters() {
    for character in CHARACTERS {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = event_at_floor(character, EventId::Mushrooms, seed, 7);
            run.state.relics.push(RelicId::OddMushroom);
            choose_event(&mut run, 0, 0);
            super::m28_world::win(&mut run);
            if super::m28_event_core::view(&run).choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Potion(_))) {
                found = Some(run);
                break;
            }
        }
        let run = found.unwrap_or_else(|| panic!("{character:?}: no Fight seed produced its event-combat potion row"));
        assert_combat_reward_pools(&run, character, "Mushrooms Fight", true);
    }
}

#[test]
fn m28_e0_colosseum_taskmaster_nob_rewards_all_characters() {
    for character in CHARACTERS {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = colosseum(character, seed);
            option_row(&mut run, 0, 0);
            super::m28_world::win(&mut run);
            option_row(&mut run, 1, 1);
            super::m28_world::win(&mut run);
            if super::m28_event_core::view(&run).choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Potion(_))) {
                found = Some(run);
                break;
            }
        }
        let run = found.unwrap_or_else(|| panic!("{character:?}: no Taskmaster/Nob seed produced its event-combat potion row"));
        assert_combat_reward_pools(&run, character, "Colosseum Taskmaster/Nob", true);
        let tiers: Vec<_> = super::m28_event_core::view(&run).choices.iter().filter_map(|choice| match choice.kind {
            RoomChoiceKind::Relic(id) => crate::content::Catalog::standard().relic_tier(id), _ => None,
        }).collect();
        assert_eq!(tiers, vec![RelicTier::Rare, RelicTier::Uncommon], "{character:?}: Taskmaster/Nob relic order");
    }
}

#[test]
fn m28_e0_masked_bandits_fight_rewards_all_characters() {
    for character in CHARACTERS {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = super::m28_event_core::at_event(character, 2, EventId::MaskedBandits, seed);
            option_row(&mut run, 0, 1);
            super::m28_world::win(&mut run);
            if super::m28_event_core::view(&run).choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Potion(_))) {
                found = Some(run);
                break;
            }
        }
        let run = found.unwrap_or_else(|| panic!("{character:?}: no Fight seed produced its event-combat potion row"));
        assert_combat_reward_pools(&run, character, "Masked Bandits Fight", true);
        assert!(super::m28_event_core::view(&run).choices.iter().any(|choice| choice.kind == RoomChoiceKind::Relic(RelicId::RedMask)),
            "{character:?}: Fight keeps the fixed Red Mask reward");
    }
}

#[test]
fn m28_e0_mind_bloom_act1_boss_rewards_all_characters() {
    for character in CHARACTERS {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = super::m28_event_core::at_event(character, 3, EventId::MindBloom, seed);
            choose_event(&mut run, 0, 0);
            super::m28_world::win(&mut run);
            if super::m28_event_core::view(&run).choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Potion(_))) {
                found = Some(run);
                break;
            }
        }
        let run = found.unwrap_or_else(|| panic!("{character:?}: no I am War seed produced its event-combat potion row"));
        assert_combat_reward_pools(&run, character, "Mind Bloom I am War", true);
        let screen = super::m28_event_core::view(&run);
        assert!(screen.choices.iter().any(|choice| choice.kind == RoomChoiceKind::Gold(50)), "{character:?}: fixed 50 gold row");
        assert!(screen.choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Relic(id)
            if crate::content::Catalog::standard().relic_tier(id) == Some(RelicTier::Rare))), "{character:?}: Rare relic row");
    }
}

#[test]
fn m28_e0_mysterious_sphere_fight_rewards_all_characters() {
    for character in CHARACTERS {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = super::m28_event_core::at_event(character, 3, EventId::MysteriousSphere, seed);
            choose_event(&mut run, 0, 0);
            super::m28_world::win(&mut run);
            if super::m28_event_core::view(&run).choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Potion(_))) {
                found = Some(run);
                break;
            }
        }
        let run = found.unwrap_or_else(|| panic!("{character:?}: no Fight seed produced its event-combat potion row"));
        assert_combat_reward_pools(&run, character, "Mysterious Sphere Fight", true);
        let screen = super::m28_event_core::view(&run);
        let gold: Vec<_> = screen.choices.iter().filter_map(|choice| match choice.kind {
            RoomChoiceKind::Gold(amount) => Some(amount), _ => None,
        }).collect();
        assert_eq!(gold.len(), 1);
        assert!((45..=55).contains(&gold[0]), "{character:?}: literal event-gold range");
    }
}

#[test]
fn m28_e13_nloth_offers_uniform_without_replacement() {
    const OWNED: [RelicId; 4] = [RelicId::Anchor, RelicId::HornCleat, RelicId::Mango, RelicId::PaperPhrog];
    let mut frequencies = [0_u32; 4];
    let mut pairs = std::collections::BTreeSet::new();
    const DRAWS: u64 = 400;
    for seed in 0..DRAWS {
        let mut root = [0_u8; 32];
        root[..8].copy_from_slice(&seed.to_be_bytes());
        let rng = RunRng::new(root, Partition::Test, seed);
        let (mut run, node) = crate::run::rooms::standard::diagnostic_event_run(Character::Silent, 2, EventId::Nloth, rng).unwrap();
        run.state.relics = OWNED.to_vec();
        run.step(&GameAction::Move(node)).unwrap();
        let offers = [
            nloth_offered_relic(&run, 0),
            nloth_offered_relic(&run, 1),
        ];
        assert_ne!(offers[0], offers[1], "seed {seed}: without replacement");
        let first = OWNED.iter().position(|id| *id == offers[0]).unwrap_or_else(|| panic!("seed {seed}: unexpected offer {:?}", offers[0]));
        let second = OWNED.iter().position(|id| *id == offers[1]).unwrap_or_else(|| panic!("seed {seed}: unexpected offer {:?}", offers[1]));
        frequencies[first] += 1;
        frequencies[second] += 1;
        pairs.insert((first.min(second), first.max(second)));
    }
    assert_eq!(pairs, [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)].into_iter().collect(),
        "all six unordered pairs appear over 400 construction seeds");
    for (index, count) in frequencies.into_iter().enumerate() {
        assert!((120..=280).contains(&count), "owned relic {:?} appears in {count}/400 draws (uniform expectation 200)", OWNED[index]);
    }
}

#[test]
fn m28_f4_shop_act_independent() {
    use crate::content::{CardColor, CardType, Catalog};
    for act in [2_u8, 3] {
        let mut template = diagnostic_shop(Character::Ironclad, 28, ShopSupply { gold: 1_000, deck: None, relics: vec![], potions: None, purge_cost: 75 });
        template.state.act = act;
        template.state.map.as_mut().unwrap().act = act;
        for seed in 0..96_u64 {
            let run = reenter_shop(&template, 10_000 + seed);
            let all = shop_rows(&run);
            assert_eq!(all.len(), 15, "Act {act}, seed {seed}: fixed shop shape");
            let cards = shop_card_rows(&run);
            assert_eq!(cards.len(), 5, "Act {act}, seed {seed}");
            assert_eq!(cards.iter().map(|row| Catalog::standard().card(shop_card_id(row)).unwrap().card_type).collect::<Vec<_>>(),
                [CardType::Attack, CardType::Attack, CardType::Skill, CardType::Skill, CardType::Power], "Act {act}, seed {seed}");
            let mut sale_count = 0;
            for row in &cards {
                let definition = Catalog::standard().card(shop_card_id(row)).unwrap();
                assert_eq!(definition.color, CardColor::Red, "Act {act}: character card pool");
                let (low, high) = shop_card_bounds(definition.rarity);
                let price = row.price.unwrap();
                if price < low { sale_count += 1; shop_price_range(price, (low / 2, high / 2)); }
                else { shop_price_range(price, (low, high)); }
            }
            assert_eq!(sale_count, 1, "Act {act}, seed {seed}: one colored sale slot");
            for (row, (rarity, bounds)) in all[5..7].iter().zip([
                (CardRarity::Uncommon, (81, 99)), (CardRarity::Rare, (162, 198)),
            ]) {
                let definition = Catalog::standard().card(shop_card_id(row)).unwrap();
                assert_eq!((definition.color, definition.rarity), (CardColor::Colorless, rarity), "Act {act}");
                shop_price_range(row.price.unwrap(), bounds);
            }
            let relics = shop_relic_rows(&run);
            assert_eq!(relics.len(), 3, "Act {act}, seed {seed}");
            for row in &relics[..2] {
                let tier = Catalog::standard().relic_tier(shop_relic_id(row)).unwrap();
                shop_price_range(row.price.unwrap(), shop_relic_bounds(tier));
            }
            assert_eq!(Catalog::standard().relic_tier(shop_relic_id(relics[2])), Some(RelicTier::Shop));
            shop_price_range(relics[2].price.unwrap(), (143, 158));
            let potions = shop_potion_rows(&run);
            assert_eq!(potions.len(), 3, "Act {act}, seed {seed}");
            for row in potions {
                let id = shop_potion_id(row);
                assert!(potion_generation::candidates(Character::Ironclad, false).unwrap().contains(&id));
                shop_price_range(row.price.unwrap(), shop_potion_bounds(Catalog::standard().potion_rarity(id).unwrap()));
            }
            assert_eq!((all[13].kind.clone(), all[13].price), (RoomChoiceKind::Purge, Some(75)), "Act {act}");
            assert_eq!((all[14].kind.clone(), all[14].price), (RoomChoiceKind::Leave, None), "Act {act}");
        }
    }
}

#[test]
fn m28_f4_chest_act_independent() {
    // Review m4: the size and tier tables take no act input, so independence is shown at the
    // room-level opening: from one state, a chest opened in Act 2 or 3 gives the same rows and the
    // same resulting state (RNG, pools, deck, gold) as in Act 1, over 64 opening keys.
    let mut base = standard(Character::Ironclad, 0x62);
    economy_pools_without_owned(&mut base);
    let open = |act: u8, key: u64| {
        let mut state = base.state.clone();
        state.act = act;
        let rows = treasure::open_rows(&mut state, &[0xF1, key]).unwrap();
        state.act = 1;
        (rows, state)
    };
    let mut shapes = std::collections::BTreeSet::new();
    for key in 0..64 {
        let act1 = open(1, key);
        assert!(act1.0.iter().any(|row| matches!(row, RoomChoiceKind::Relic(_))), "key {key}: one relic chest row");
        assert!(act1.0.iter().any(|row| *row == RoomChoiceKind::Sapphire), "key {key}: sapphire link remains present");
        shapes.insert(act1.0.iter().any(|row| matches!(row, RoomChoiceKind::Gold(_))));
        for act in [2_u8, 3] {
            assert!(open(act, key) == act1, "Act {act}, key {key}: chest rows or state differ from Act 1");
        }
    }
    assert_eq!(shapes.len(), 2, "the 64 keys cover chests with and without gold");
    for (roll, expected) in [
        (0, treasure::ChestSize::Small), (49, treasure::ChestSize::Small),
        (50, treasure::ChestSize::Medium), (82, treasure::ChestSize::Medium),
        (83, treasure::ChestSize::Large), (99, treasure::ChestSize::Large),
    ] {
        assert_eq!(treasure::chest_size(roll), expected, "size roll {roll}");
    }
    for (size, roll, tier, gold) in [
        (treasure::ChestSize::Small, 0, RelicTier::Common, true),
        (treasure::ChestSize::Small, 49, RelicTier::Common, true),
        (treasure::ChestSize::Small, 50, RelicTier::Common, false),
        (treasure::ChestSize::Small, 74, RelicTier::Common, false),
        (treasure::ChestSize::Small, 75, RelicTier::Uncommon, false),
        (treasure::ChestSize::Small, 99, RelicTier::Uncommon, false),
        (treasure::ChestSize::Medium, 0, RelicTier::Common, true),
        (treasure::ChestSize::Medium, 34, RelicTier::Common, true),
        (treasure::ChestSize::Medium, 35, RelicTier::Uncommon, false),
        (treasure::ChestSize::Medium, 84, RelicTier::Uncommon, false),
        (treasure::ChestSize::Medium, 85, RelicTier::Rare, false),
        (treasure::ChestSize::Medium, 99, RelicTier::Rare, false),
        (treasure::ChestSize::Large, 0, RelicTier::Uncommon, true),
        (treasure::ChestSize::Large, 49, RelicTier::Uncommon, true),
        (treasure::ChestSize::Large, 50, RelicTier::Uncommon, false),
        (treasure::ChestSize::Large, 74, RelicTier::Uncommon, false),
        (treasure::ChestSize::Large, 75, RelicTier::Rare, false),
        (treasure::ChestSize::Large, 99, RelicTier::Rare, false),
    ] {
        assert_eq!(treasure::chest_tier_gold(size, roll), (tier, gold), "{size:?} roll {roll}");
    }
}
