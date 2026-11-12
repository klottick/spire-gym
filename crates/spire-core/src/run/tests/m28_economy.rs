//! M28b Lane C reward and treasure contracts; expected values are source literals.
use super::super::*;
use crate::{
    combat::rng::Partition,
    content::{CardColor, CardId, CardRarity, Character, Catalog, RelicId, RelicTier},
    items::relic_pools::{self, RelicPools},
    run::rooms::standard::{rewards, treasure},
};

fn standard(seed: u8) -> Run {
    Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        CreationProfile::experienced(),
        RunRng::new([seed; 32], Partition::Test, 0),
        RoomScenario::Standard,
    ).unwrap()
}

fn reseed(run: &mut Run, seed: u8) {
    run.state.rng = RunRng::new([seed; 32], Partition::Test, 0);
}

fn card_rarity(id: CardId) -> CardRarity {
    Catalog::standard().card(id).unwrap().rarity
}

fn card_color(id: CardId) -> CardColor {
    Catalog::standard().card(id).unwrap().color
}

fn pools_without_owned(run: &mut Run) {
    run.state.relic_pools = Some(relic_pools::build(&run.state, &[]).unwrap());
}

#[test]
fn m28_card_reward_rarity_offset() {
    let mut common = standard(1);
    common.state.card_rarity_offset = Some(100);
    let offers = rewards::ordinary_offer(&mut common.state, rewards::RarityRoom::Ordinary, 3, 1, &[0xCA]).unwrap();
    assert!(offers.iter().all(|(id, _)| card_rarity(*id) == CardRarity::Common));
    assert_eq!(common.state.card_rarity_offset, Some(97), "three common slots decrement once each");

    let mut rare = standard(2);
    rare.state.card_rarity_offset = Some(-200);
    let offers = rewards::ordinary_offer(&mut rare.state, rewards::RarityRoom::Ordinary, 1, 1, &[0xCB]).unwrap();
    assert_eq!(card_rarity(offers[0].0), CardRarity::Rare);
    assert_eq!(rare.state.card_rarity_offset, Some(5), "a rare slot resets the offset");

    let mut boss = standard(3);
    boss.state.card_rarity_offset = Some(100);
    let offers = rewards::ordinary_offer(&mut boss.state, rewards::RarityRoom::Boss, 3, 1, &[0xCC]).unwrap();
    assert!(offers.iter().all(|(id, _)| card_rarity(*id) == CardRarity::Rare));
    assert_eq!(boss.state.card_rarity_offset, Some(5), "every boss slot resets the offset");

    // Across independent offers, rows are distinct and the offset follows one write per slot,
    // regardless of any identity retry at that slot's already-selected rarity.
    for seed in 0..32 {
        let mut run = standard(seed);
        run.state.card_rarity_offset = Some(-40);
        let offers = rewards::ordinary_offer(&mut run.state, rewards::RarityRoom::Ordinary, 3, 1, &[0xCD]).unwrap();
        let mut expected = -40;
        for (id, _) in &offers {
            expected = match card_rarity(*id) {
                CardRarity::Rare => 5,
                CardRarity::Common => (expected - 1).max(-40),
                CardRarity::Uncommon => expected,
                other => panic!("ordinary reward had unsupported rarity {other:?}"),
            };
        }
        let ids: Vec<_> = offers.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids.iter().copied().collect::<std::collections::BTreeSet<_>>().len(), ids.len());
        assert_eq!(run.state.card_rarity_offset, Some(expected));
    }

    // Prismatic Shard selects from AnyColour, including another character's ordinary cards.
    let mut found_other_colour = false;
    for seed in 0..64 {
        let mut run = standard(seed);
        run.state.relics.push(RelicId::PrismaticShard);
        run.state.card_rarity_offset = Some(100);
        let offers = rewards::ordinary_offer(&mut run.state, rewards::RarityRoom::Ordinary, 3, 1, &[0xCE]).unwrap();
        found_other_colour |= offers.iter().any(|(id, _)| !matches!(card_color(*id), CardColor::Red | CardColor::Colorless));
    }
    assert!(found_other_colour, "Shard should expose another colour over seeded offers");
}

#[test]
fn m28_card_reward_upgrade_chance_by_act() {
    // Amended at M31 (plan item 4): `upgrade_chance` takes the ascension; these are the A0 values.
    assert_eq!(rewards::upgrade_chance(1, 0).to_bits(), 0.0_f32.to_bits());
    assert_eq!(rewards::upgrade_chance(2, 0).to_bits(), 0.25_f32.to_bits());
    assert_eq!(rewards::upgrade_chance(3, 0).to_bits(), 0.5_f32.to_bits());
    assert_eq!(rewards::upgrade_chance(4, 0).to_bits(), 0.5_f32.to_bits());
    assert_eq!(rewards::upgrade_chance(5, 0).to_bits(), 0.0_f32.to_bits());

    let mut act1 = standard(4);
    act1.state.act = 1;
    act1.state.card_rarity_offset = Some(100);
    let offers = rewards::ordinary_offer(&mut act1.state, rewards::RarityRoom::Ordinary, 3, 1, &[0xD1]).unwrap();
    assert!(offers.iter().all(|(_, upgrades)| *upgrades == 0), "Act 1 has no automatic upgrades without Eggs");

    let mut upgraded_nonrare = false;
    for seed in 0..32 {
        let mut run = standard(seed);
        run.state.act = 3;
        run.state.card_rarity_offset = Some(100);
        let offers = rewards::ordinary_offer(&mut run.state, rewards::RarityRoom::Ordinary, 3, 1, &[0xD2]).unwrap();
        upgraded_nonrare |= offers.iter().any(|(id, upgrades)| card_rarity(*id) != CardRarity::Rare && *upgrades == 1);
        assert!(offers.iter().all(|(id, upgrades)| card_rarity(*id) != CardRarity::Rare || *upgrades == 0));
    }
    assert!(upgraded_nonrare, "Act 3's literal 0.5 chance should upgrade a non-Rare card in the seeded sample");

    // Force a Rare result with the source rarity multiplier; F3b (source-confirmed) keeps it at 0.
    let mut rare = standard(5);
    rare.state.act = 3;
    rare.state.card_rarity_offset = Some(0);
    let offers = rewards::ordinary_offer(&mut rare.state, rewards::RarityRoom::Ordinary, 3, 100, &[0xD3]).unwrap();
    assert!(offers.iter().all(|(id, upgrades)| card_rarity(*id) == CardRarity::Rare && *upgrades == 0));
}

#[test]
fn m28_card_reward_egg_preview() {
    let mut run = standard(6);
    run.state.relics.push(RelicId::MoltenEgg);
    assert_eq!(rewards::preview_upgrades(&run.state, CardId::Bash, 0), 1);
    assert_eq!(rewards::preview_upgrades(&run.state, CardId::Bash, 1), 1);
    assert_eq!(rewards::preview_upgrades(&run.state, CardId::ShrugItOff, 0), 0, "Molten Egg matches Attacks only");
}

#[test]
fn m28_colourless_offer_rare_threshold() {
    let mut found_rare = false;
    let mut found_uncommon = false;
    for seed in 0..64 {
        let mut run = standard(seed);
        run.state.card_rarity_offset = Some(123);
        let mut expected_rng = run.state.rng.clone();
        let draw_key = [0xE1, 0xC0, 0];
        let u = expected_rng.bounded_scoped_checked(RunRngScope::M28CardRarity, &draw_key, 1_u64 << 24).unwrap() as u32;
        let expected_rare = (u as f32 / 16_777_216.0_f32) < 0.3_f32;
        let offers = rewards::colourless_offer(&mut run.state, 1, &[0xE1]).unwrap();
        assert_eq!(card_color(offers[0].0), CardColor::Colorless);
        assert_eq!(card_rarity(offers[0].0) == CardRarity::Rare, expected_rare, "strict binary32 threshold for seed {seed}");
        match card_rarity(offers[0].0) {
            CardRarity::Rare => {
                found_rare = true;
                assert_eq!(run.state.card_rarity_offset, Some(5));
            }
            CardRarity::Uncommon => {
                found_uncommon = true;
                assert_eq!(run.state.card_rarity_offset, Some(123));
            }
            other => panic!("colourless reward had unsupported rarity {other:?}"),
        }
    }
    assert!(found_rare && found_uncommon, "seeded offers exercise both sides of the strict binary32 threshold");
}

#[test]
fn m28_chest_size_and_tier_tables() {
    for (roll, expected) in [
        (0, treasure::ChestSize::Small), (49, treasure::ChestSize::Small),
        (50, treasure::ChestSize::Medium), (82, treasure::ChestSize::Medium),
        (83, treasure::ChestSize::Large), (99, treasure::ChestSize::Large),
    ] {
        assert_eq!(treasure::chest_size(roll), expected);
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

#[test]
fn m28_chest_gold_endpoints() {
    assert_eq!(treasure::chest_gold(25, 3_355_440), 24);
    for (nominal, low, high) in [(25, 23, 28), (50, 45, 55), (75, 68, 83)] {
        assert_eq!(treasure::chest_gold(nominal, 0), low);
        assert_eq!(treasure::chest_gold(nominal, (1 << 24) - 1), high);
    }
}

#[test]
fn m28_chest_sapphire_link() {
    let mut open_with_gold = None;
    for seed in 0..64 {
        let mut run = standard(seed);
        reseed(&mut run, seed);
        let rows = treasure::open_rows(&mut run.state, &[0xF1]).unwrap();
        if rows.iter().any(|row| matches!(row, RoomChoiceKind::Gold(_))) {
            open_with_gold = Some((run, rows));
            break;
        }
    }
    let (_, rows) = open_with_gold.expect("seeded ordinary chest with gold");
    assert!(matches!(rows.as_slice(), [RoomChoiceKind::Gold(_), RoomChoiceKind::Relic(_), RoomChoiceKind::Sapphire]));

    let mut keyed = standard(7);
    keyed.state.keys[1] = true;
    let rows = treasure::open_rows(&mut keyed.state, &[0xF2]).unwrap();
    assert!(rows.iter().any(|row| matches!(row, RoomChoiceKind::Relic(_))));
    assert!(!rows.iter().any(|row| matches!(row, RoomChoiceKind::Sapphire)));

    let mut cursed_key = standard(9);
    cursed_key.state.relics.push(RelicId::CursedKey);
    pools_without_owned(&mut cursed_key);
    let deck_len = cursed_key.state.deck.len();
    treasure::open_rows(&mut cursed_key.state, &[0xF5]).unwrap();
    assert_eq!(cursed_key.state.deck.len(), deck_len + 1, "ordinary Cursed Key schedules one permanent curse obtain");
    assert_eq!(card_rarity(cursed_key.state.deck.last().unwrap().id), CardRarity::Curse);

    let mut hungry_gold = None;
    for seed in 0..64 {
        let mut run = standard(seed);
        run.state.relics.push(RelicId::NlothsHungryFace);
        pools_without_owned(&mut run);
        reseed(&mut run, seed);
        let rows = treasure::open_rows(&mut run.state, &[0xF3]).unwrap();
        if rows.iter().any(|row| matches!(row, RoomChoiceKind::Gold(_))) {
            hungry_gold = Some((run, rows));
            break;
        }
    }
    let (run, rows) = hungry_gold.expect("seeded Hungry Face chest with gold");
    assert!(matches!(rows.as_slice(), [RoomChoiceKind::Gold(_)]), "Hungry Face removes the base relic and linked Sapphire, leaving gold");
    assert_eq!(run.state.relic_counters.get(&RelicId::NlothsHungryFace), Some(&1));

    let mut matryoshka_extra_first = false;
    for seed in 0..64 {
        let mut run = standard(seed);
        run.state.relics.push(RelicId::Matryoshka);
        run.state.relic_counters.insert(RelicId::Matryoshka, 2);
        pools_without_owned(&mut run);
        reseed(&mut run, seed);
        let rows = treasure::open_rows(&mut run.state, &[0xF4]).unwrap();
        if matches!(rows.first(), Some(RoomChoiceKind::Relic(_))) && rows.iter().filter(|row| matches!(row, RoomChoiceKind::Relic(_))).count() == 2 {
            matryoshka_extra_first = true;
            assert_eq!(rows.iter().filter(|row| matches!(row, RoomChoiceKind::Sapphire)).count(), 1);
            assert_eq!(run.state.relic_counters.get(&RelicId::Matryoshka), Some(&1));
            break;
        }
    }
    assert!(matryoshka_extra_first, "Matryoshka inserts its extra relic before the base chest rows");
}

#[test]
fn m28_boss_chest_three_offers() {
    let mut run = standard(8);
    let first = [RelicId::CoffeeDripper, RelicId::SneckoEye, RelicId::RunicPyramid];
    let tail = [RelicId::CursedKey, RelicId::VelvetChoker];
    run.state.relic_pools = Some(RelicPools { boss: [first.as_slice(), tail.as_slice()].concat(), ..RelicPools::default() });
    run.state.relics.push(RelicId::CursedKey);
    let deck_before = run.state.deck.clone();
    let offers = treasure::boss_offers(&mut run.state, &[0xF5]).unwrap();
    assert_eq!(offers, first);
    assert_eq!(run.state.relic_pools.as_ref().unwrap().boss, tail);
    assert_eq!(offers.iter().copied().collect::<std::collections::BTreeSet<_>>().len(), 3);
    assert_eq!(run.state.deck, deck_before, "boss opening does not schedule Cursed Key's ordinary chest curse");
}

/// spire-0kr (PM ruling): with every relic pool exhausted a second time an ordinary chest offers Circlet again (the Rare
/// fallback), even when Circlet is owned; the shared reward chooser then stacks it (see
/// `m28_events_a15::x0kr_a_duplicate_circlet_reward_stacks`).
#[test]
fn x0kr_an_exhausted_chest_offers_circlet_while_circlet_is_owned() {
    let mut run = standard(11);
    run.state.relics.push(RelicId::Circlet);
    run.state.relic_counters.insert(RelicId::Circlet, 1);
    run.state.relic_pools = Some(RelicPools::default());
    let rows = treasure::open_rows(&mut run.state, &[0xF7]).unwrap();
    assert!(rows.contains(&RoomChoiceKind::Relic(RelicId::Circlet)), "{rows:?}");
}
