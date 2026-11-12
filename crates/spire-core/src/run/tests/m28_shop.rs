//! M28a shop lane tests (chunk plan §4 items 16-20).
use super::super::*;
use crate::{
    combat::rng::Partition,
    content::{CardColor, CardId, CardRarity, CardType, Character, Catalog, PotionId, RelicId, RelicTier},
    items::{potion_generation, relic_pools},
    run::rooms::{self, standard::shop::ShopSupply},
};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_be_bytes());
    RunRng::new(root, Partition::Test, seed)
}

fn supply(gold: u32, relics: Vec<RelicId>, potions: Option<Vec<Option<PotionId>>>) -> ShopSupply {
    ShopSupply { gold, deck: None, relics, potions, purge_cost: 75 }
}

fn shop(character: Character, seed: u64, items: ShopSupply) -> Run {
    rooms::standard::shop::diagnostic_shop(character, items, rng(seed)).unwrap()
}

fn reenter(template: &Run, seed: u64, offset: Option<i32>) -> Run {
    let mut run = template.clone();
    run.state.rng = rng(seed);
    if let Some(offset) = offset { run.state.card_rarity_offset = Some(offset); }
    run.state.relic_pools = Some(relic_pools::build(&run.state, &[]).unwrap());
    rooms::standard::shop::enter(&mut run).unwrap();
    run
}

fn rows(run: &Run) -> &[RoomChoiceView] {
    let room = run.state.room_loop.as_ref().unwrap();
    assert_eq!(room.screen.kind, RoomViewKind::Shop);
    &room.screen.choices
}

fn card_id(row: &RoomChoiceView) -> CardId {
    match row.kind { RoomChoiceKind::Card { id, .. } => id, ref other => panic!("expected card row, got {other:?}") }
}

fn potion_id(row: &RoomChoiceView) -> PotionId {
    match row.kind { RoomChoiceKind::Potion(id) => id, ref other => panic!("expected potion row, got {other:?}") }
}

fn relic_id(row: &RoomChoiceView) -> RelicId {
    match row.kind { RoomChoiceKind::Relic(id) => id, ref other => panic!("expected relic row, got {other:?}") }
}

fn card_color(character: Character) -> CardColor {
    match character {
        Character::Ironclad => CardColor::Red,
        Character::Silent => CardColor::Green,
        Character::Defect => CardColor::Blue,
        Character::Watcher => CardColor::Purple,
    }
}

fn price_range(price: u32, (low, high): (u32, u32)) {
    assert!((low..=high).contains(&price), "price {price} outside {low}..={high}");
}

fn card_bounds(rarity: CardRarity) -> (u32, u32) {
    match rarity {
        CardRarity::Common => (45, 55),
        CardRarity::Uncommon => (67, 82),
        CardRarity::Rare => (135, 165),
        other => panic!("shop card had unsupported rarity {other:?}"),
    }
}

fn relic_bounds(tier: RelicTier) -> (u32, u32) {
    match tier {
        RelicTier::Common | RelicTier::Shop => (143, 158),
        RelicTier::Uncommon => (238, 263),
        RelicTier::Rare => (285, 315),
        other => panic!("shop relic had unsupported tier {other:?}"),
    }
}

fn potion_bounds(rarity: crate::content::PotionRarity) -> (u32, u32) {
    match rarity {
        crate::content::PotionRarity::Common => (48, 52),
        crate::content::PotionRarity::Uncommon => (71, 79),
        crate::content::PotionRarity::Rare => (95, 105),
    }
}

fn card_rows(run: &Run) -> Vec<&RoomChoiceView> {
    rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })
        && Catalog::standard().card(card_id(row)).is_some_and(|definition| definition.color != CardColor::Colorless)).collect()
}

fn all_card_rows(run: &Run) -> Vec<&RoomChoiceView> {
    rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).collect()
}

fn potion_rows(run: &Run) -> Vec<&RoomChoiceView> {
    rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).collect()
}

fn relic_rows(run: &Run) -> Vec<&RoomChoiceView> {
    rows(run).iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).collect()
}

/// spire-0kr: buys a row with The Courier owned and returns its restock, which replaces the bought row in place (same index,
/// fresh choice id, same row count).
pub(super) fn courier_buy(run: &mut Run, bought: ChoiceId) -> RoomChoiceView {
    let screen = |run: &Run| run.state.room_loop.as_ref().unwrap().screen.choices.clone();
    let before = screen(run);
    let index = before.iter().position(|row| row.id == bought).unwrap();
    run.step(&GameAction::Choose(bought)).unwrap();
    let after = screen(run);
    assert_eq!(after.len(), before.len(), "the Courier restock replaces the bought row");
    assert!(after[index].id != bought && after[index].status == RoomChoiceStatus::Offered, "restock at the bought index");
    after[index].clone()
}

fn choose_row(run: &mut Run, predicate: impl Fn(&RoomChoiceKind) -> bool) {
    let id = rows(run).iter().find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind))
        .unwrap_or_else(|| panic!("offered shop row not found: {:?}", rows(run)))
        .id;
    run.step(&GameAction::Choose(id)).unwrap();
}

fn rarity(id: CardId) -> CardRarity {
    Catalog::standard().card(id).unwrap().rarity
}

#[test]
fn m28_shop_stock_shape_and_prices() {
    // Static price endpoints from c-a.md S5/S6, checked with the shared binary32/grid helpers.
    for (base, low, high, expected_low, expected_high) in [
        (50.0_f32, 0.9_f32, 1.1_f32, 45, 55),
        (75.0_f32, 0.9_f32, 1.1_f32, 67, 82),
        (150.0_f32, 0.9_f32, 1.1_f32, 135, 165),
    ] {
        for (k, expected) in [(0, expected_low), ((1 << 24) - 1, expected_high)] {
            assert_eq!((base * rooms::shop::jitter(low, high, k)) as u32, expected);
        }
    }
    for (base, low, high, expected_low, expected_high) in [
        (75, 0.9_f32, 1.1_f32, 81, 99),
        (150, 0.9_f32, 1.1_f32, 162, 198),
    ] {
        for (k, expected) in [(0, expected_low), ((1 << 24) - 1, expected_high)] {
            let raw = base as f32 * rooms::shop::jitter(low, high, k) * 1.2_f32;
            assert_eq!(raw as u32, expected);
        }
    }
    for (base, expected_low, expected_high) in [(150, 143, 158), (250, 238, 263), (300, 285, 315), (50, 48, 52), (75, 71, 79), (100, 95, 105)] {
        assert_eq!(rooms::shop::item_sticker(base, 0), expected_low);
        assert_eq!(rooms::shop::item_sticker(base, (1 << 24) - 1), expected_high);
    }

    for character in CHARACTERS {
        let template = shop(character, 28, supply(1_000, vec![], None));
        assert_eq!(rows(&template).len(), 15, "{character:?}");
        for seed in 0..96 {
            let run = if seed == 0 { template.clone() } else { reenter(&template, seed, None) };
            let all = rows(&run);
            assert_eq!(all.len(), 15, "{character:?}, seed {seed}");
            let cards = card_rows(&run);
            assert_eq!(cards.len(), 5);
            assert_eq!(cards.iter().map(|row| Catalog::standard().card(card_id(row)).unwrap().card_type).collect::<Vec<_>>(),
                [CardType::Attack, CardType::Attack, CardType::Skill, CardType::Skill, CardType::Power]);
            let mut sale_count = 0;
            for row in &cards {
                let id = card_id(row);
                let definition = Catalog::standard().card(id).unwrap();
                assert_eq!(definition.color, card_color(character), "{character:?}: {id:?}");
                let (low, high) = card_bounds(definition.rarity);
                let price = row.price.unwrap();
                if price < low {
                    sale_count += 1;
                    price_range(price, (low / 2, high / 2));
                } else {
                    price_range(price, (low, high));
                }
            }
            assert_eq!(sale_count, 1, "one colored sale slot; {character:?}, seed {seed}");

            let colorless = &all[5..7];
            assert_eq!(colorless.len(), 2);
            for (row, (expected_rarity, (low, high))) in colorless.iter().zip([
                (CardRarity::Uncommon, (81, 99)),
                (CardRarity::Rare, (162, 198)),
            ]) {
                let definition = Catalog::standard().card(card_id(row)).unwrap();
                assert_eq!(definition.color, CardColor::Colorless);
                assert_eq!(definition.rarity, expected_rarity);
                price_range(row.price.unwrap(), (low, high));
            }

            let relics = relic_rows(&run);
            assert_eq!(relics.len(), 3);
            assert!(relics.iter().all(|row| matches!(row.status, RoomChoiceStatus::Offered)));
            for row in &relics[..2] {
                let tier = Catalog::standard().relic_tier(relic_id(row)).unwrap();
                assert!(matches!(tier, RelicTier::Common | RelicTier::Uncommon | RelicTier::Rare));
                price_range(row.price.unwrap(), relic_bounds(tier));
            }
            assert_eq!(Catalog::standard().relic_tier(relic_id(relics[2])), Some(RelicTier::Shop));
            price_range(relics[2].price.unwrap(), (143, 158));

            assert_eq!(potion_generation::candidates(character, false).unwrap().len(), 33);
            let potions = potion_rows(&run);
            assert_eq!(potions.len(), 3);
            for row in potions {
                let potion = potion_id(row);
                assert!(potion_generation::candidates(character, false).unwrap().contains(&potion));
                let potion_rarity = Catalog::standard().potion_rarity(potion).unwrap();
                price_range(row.price.unwrap(), potion_bounds(potion_rarity));
            }
            assert_eq!(all[13].kind, RoomChoiceKind::Purge);
            assert_eq!(all[13].price, Some(75));
            assert_eq!(all[14].kind, RoomChoiceKind::Leave);
            assert_eq!(all[14].price, None);
        }

        // A full belt and Sozu still leave all three potion offers in the stock.
        let class_potion = potion_generation::candidates(character, false).unwrap()[0];
        let full = shop(character, 29, supply(1_000, vec![RelicId::Sozu], Some(vec![Some(class_potion); 3])));
        assert_eq!(potion_rows(&full).len(), 3, "{character:?}");
    }
}

#[test]
fn m28_shop_sale_slot_uniform() {
    for character in CHARACTERS {
        let template = shop(character, 30, supply(1_000, vec![], None));
        let mut hits = [0_u32; 5];
        const SAMPLES: u64 = 250;
        for seed in 0..SAMPLES {
            let run = reenter(&template, 10_000 + seed, None);
            let cards = card_rows(&run);
            let sales: Vec<_> = cards.iter().enumerate().filter(|(_, row)| {
                let definition = Catalog::standard().card(card_id(row)).unwrap();
                row.price.unwrap() < card_bounds(definition.rarity).0
            }).map(|(slot, _)| slot).collect();
            assert_eq!(sales.len(), 1, "{character:?}, seed {seed}");
            hits[sales[0]] += 1;
            for row in &rows(&run)[5..7] {
                let definition = Catalog::standard().card(card_id(row)).unwrap();
                assert!(row.price.unwrap() >= card_bounds(definition.rarity).0, "colorless row was discounted");
            }
        }
        for (slot, count) in hits.into_iter().enumerate() {
            // Six-sigma binomial band for 250 independent 1/5 proposals is approximately 12..88.
            assert!((12..=88).contains(&count), "{character:?} sale slot {slot} hit {count}/250");
        }
    }
}

#[test]
fn m28_shop_cards_read_offset_without_mutation() {
    // S2's boundaries and counts for the 0..100 proposal support.
    let ordinary = |n, offset| rooms::standard::rewards::classify(rooms::standard::rewards::RarityRoom::Ordinary, n, offset, 1);
    assert_eq!((0..100).filter(|n| ordinary(*n, 5) == CardRarity::Rare).count(), 0);
    assert_eq!((0..100).filter(|n| ordinary(*n, 5) == CardRarity::Uncommon).count(), 35);
    assert_eq!((0..100).filter(|n| ordinary(*n, -40) == CardRarity::Rare).count(), 43);
    assert_eq!((0..100).filter(|n| ordinary(*n, -40) == CardRarity::Uncommon).count(), 37);

    for character in CHARACTERS {
        let template = shop(character, 31, supply(1_000, vec![], None));
        let (mut common_noncommon, mut shifted_noncommon, mut shifted_rare) = (0, 0, 0);
        const SAMPLES: u64 = 160;
        for seed in 0..SAMPLES {
            let usual = reenter(&template, 20_000 + seed, Some(5));
            assert_eq!(usual.state.card_rarity_offset, Some(5), "entry wrote offset for {character:?}");
            for row in card_rows(&usual) {
                let rarity = rarity(card_id(row));
                common_noncommon += u32::from(rarity != CardRarity::Common);
            }
            let shifted = reenter(&template, 30_000 + seed, Some(-40));
            assert_eq!(shifted.state.card_rarity_offset, Some(-40), "entry wrote forced offset for {character:?}");
            for row in card_rows(&shifted) {
                let rarity = rarity(card_id(row));
                shifted_noncommon += u32::from(rarity != CardRarity::Common);
                shifted_rare += u32::from(rarity == CardRarity::Rare);
            }
        }
        assert!(shifted_noncommon > common_noncommon + 100, "{character:?}: offset shift should raise rare/uncommon offers ({common_noncommon} -> {shifted_noncommon})");
        assert!(shifted_rare > 250, "{character:?}: offset -40 should produce the 43/100 rare band, saw {shifted_rare}");

        // Courier card replacement reads (but never writes) the live rarity offset.
        let mut courier = shop(character, 32, supply(1_000, vec![RelicId::TheCourier], None));
        assert_eq!(courier.state.card_rarity_offset, Some(5));
        choose_row(&mut courier, |kind| matches!(kind, RoomChoiceKind::Card { .. }));
        assert_eq!(courier.state.card_rarity_offset, Some(5), "Courier replacement wrote offset for {character:?}");
    }
}

#[test]
fn m28_shop_second_attack_skill_redraws() {
    for character in CHARACTERS {
        let template = shop(character, 33, supply(1_000, vec![], None));
        for seed in 0..128 {
            let run = if seed == 0 { template.clone() } else { reenter(&template, 40_000 + seed, None) };
            let cards = card_rows(&run);
            assert_eq!(cards.len(), 5);
            assert_ne!(card_id(cards[0]), card_id(cards[1]), "duplicate Attacks for {character:?}, seed {seed}");
            assert_ne!(card_id(cards[2]), card_id(cards[3]), "duplicate Skills for {character:?}, seed {seed}");
            for row in cards {
                let definition = Catalog::standard().card(card_id(row)).unwrap();
                assert!(!(definition.card_type == CardType::Power && definition.rarity == CardRarity::Common), "common Power offered for {character:?}");
            }
        }
    }
}

#[test]
fn m28_shop_removal_price_precedence() {
    let cases: Vec<(Vec<RelicId>, u32)> = vec![
        (vec![], 75),
        (vec![RelicId::TheCourier], 60),
        (vec![RelicId::MembershipCard], 38),
        (vec![RelicId::TheCourier, RelicId::MembershipCard], 38),
        (vec![RelicId::MembershipCard, RelicId::TheCourier], 38),
        (vec![RelicId::SmilingMask], 50),
        (vec![RelicId::SmilingMask, RelicId::TheCourier], 50),
        (vec![RelicId::TheCourier, RelicId::SmilingMask], 50),
        (vec![RelicId::SmilingMask, RelicId::MembershipCard], 50),
        (vec![RelicId::MembershipCard, RelicId::SmilingMask], 50),
        (vec![RelicId::SmilingMask, RelicId::TheCourier, RelicId::MembershipCard], 50),
    ];
    for character in CHARACTERS {
        for (relics, expected) in &cases {
            let run = shop(character, 34, supply(1_000, relics.clone(), None));
            let purge = rows(&run).iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
            assert_eq!(purge.price, Some(*expected), "{character:?}, {relics:?}");
        }

        // The selected callback spends the displayed price, raises the base once and claims the row.
        let mut paid = shop(character, 35, supply(1_000, vec![RelicId::TheCourier, RelicId::MembershipCard], None));
        let purge = rows(&paid).iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().id;
        let gold = paid.state.gold;
        paid.step(&GameAction::Choose(purge)).unwrap();
        let select = paid.decision().unwrap().candidates.into_iter().find_map(|candidate| match candidate.action {
            GameAction::Select(id) => Some(id), _ => None,
        }).unwrap();
        paid.step(&GameAction::Select(select)).unwrap();
        let room = paid.state.room_loop.as_ref().unwrap();
        assert_eq!(paid.state.gold, gold - 38);
        assert_eq!(room.purge_cost, 100);
        let claimed = room.screen.choices.iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
        assert_eq!(claimed.status, RoomChoiceStatus::Claimed);
        // Research :180-183: the next-price branch shows G(F(F(100 × .8) × .5)) = 40 (display only).
        assert_eq!(claimed.price, Some(40));

        // Non-removable curses and a bottled card do not enter the standard ShopPurge selector.
        let deck = vec![(CardId::Strike, 0), (CardId::Necronomicurse, 0), (CardId::CurseOfTheBell, 0), (CardId::AscendersBane, 0), (CardId::Defend, 0)];
        let mut candidates = shop(character, 36, ShopSupply { gold: 1_000, deck: Some(deck), relics: vec![], potions: None, purge_cost: 75 });
        let bottled_instance = candidates.state.deck[0].instance;
        candidates.state.relic_cards.insert(RelicId::BottledFlame, bottled_instance);
        let purge = rows(&candidates).iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().id;
        candidates.step(&GameAction::Choose(purge)).unwrap();
        let eligible = candidates.decision().unwrap().selection.unwrap().eligible;
        assert_eq!(eligible.iter().map(|card| card.id).collect::<Vec<_>>(), [CardId::Defend]);
    }
}

#[test]
fn m28_shop_courier_replacement_prices() {
    for character in CHARACTERS {
        let template = shop(character, 37, supply(1_000, vec![RelicId::TheCourier], None));

        let mut card_run = template.clone();
        let card_row = rows(&card_run).iter().find(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).unwrap().clone();
        let bought = card_id(&card_row);
        let bought_type = Catalog::standard().card(bought).unwrap().card_type;
        let replacement = courier_buy(&mut card_run, card_row.id);
        let replacement_id = card_id(&replacement);
        let replacement_definition = Catalog::standard().card(replacement_id).unwrap();
        assert_eq!(replacement_definition.card_type, bought_type, "{character:?} replacement keeps type");
        assert_eq!(replacement_definition.color, card_color(character), "{character:?} colored replacement stays colored");
        assert!(replacement.price.unwrap() > 0);

        let mut potion_run = template.clone();
        let potion_row = potion_rows(&potion_run)[0].clone();
        let replacement = courier_buy(&mut potion_run, potion_row.id);
        let replacement_potion = potion_id(&replacement);
        assert!(potion_generation::candidates(character, false).unwrap().contains(&replacement_potion));
        assert!(replacement.price.unwrap() > 0);

        // Pick an offered relic without an equip selector, then confirm the Merchant replacement takes its row.
        let mut relic_run = None;
        for seed in 0..64 {
            let candidate = reenter(&template, 50_000 + seed, None);
            let selectable = relic_rows(&candidate).into_iter().find(|row| {
                let id = relic_id(row);
                !matches!(id, RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado
                    | RelicId::TheCourier | RelicId::MembershipCard | RelicId::SmilingMask)
            });
            if let Some(row) = selectable {
                let selected_id = row.id;
                relic_run = Some((candidate, selected_id));
                break;
            }
        }
        let (mut relic_run, chosen) = relic_run.expect("a non-selection, non-discount Merchant offer in 64 seeds");
        let old_relics = relic_rows(&relic_run).into_iter().map(|row| relic_id(row)).collect::<Vec<_>>();
        let bought_relic = relic_rows(&relic_run).into_iter().find(|row| row.id == chosen).map(|row| relic_id(row)).unwrap();
        let replacement_relic = relic_id(&courier_buy(&mut relic_run, chosen));
        assert!(!old_relics.contains(&replacement_relic), "replacement consumes an unoffered pool relic");
        assert!(!matches!(replacement_relic, RelicId::OldCoin | RelicId::SmilingMask | RelicId::MawBank | RelicId::TheCourier));
        assert_ne!(replacement_relic, bought_relic);

        // S8 literal: initial sequential rounding gives 61; replacement multiplies/truncates once to 60.
        let first_pass = rooms::shop::libgdx_round(151.0_f32 * 0.8_f32);
        let sequential = rooms::shop::libgdx_round(first_pass as f32 * 0.5_f32);
        let replacement_price = (151.0_f32 * 0.8_f32 * 0.5_f32) as u32;
        assert_eq!(sequential, 61);
        assert_eq!(replacement_price, 60);
    }
}

#[test]
fn m28_shop_courier_card_discount_one_final_truncation() {
    use crate::items::dispatch::{run_event, RunItemEvent};

    let run = shop(Character::Ironclad, 151, supply(1_000,
        vec![RelicId::TheCourier, RelicId::MembershipCard], None));
    // c-a.md S8: binary32 151×.8×.5 = 60.4, then one final T gives 60.
    assert_eq!(rooms::shop::courier_card_discount(&run.state, 151.0_f32), 60);

    // Shop entry uses two G passes: G(151×.8)=121, then G(121×.5)=61.
    let mut sequential_run = run.clone();
    let mut event = RunItemEvent::ShopPrice { price: 151 };
    run_event(&mut sequential_run.state, &mut event).unwrap();
    assert_eq!(event, RunItemEvent::ShopPrice { price: 61 });

    let courier = shop(Character::Ironclad, 152, supply(1_000, vec![RelicId::TheCourier], None));
    // 151×.8 in binary32 is 120.8; one T gives 120.
    assert_eq!(rooms::shop::courier_card_discount(&courier.state, 151.0_f32), 120);

    let membership = shop(Character::Ironclad, 153, supply(1_000, vec![RelicId::MembershipCard], None));
    // 151×.5 in binary32 is 75.5; one T gives 75.
    assert_eq!(rooms::shop::courier_card_discount(&membership.state, 151.0_f32), 75);
}

fn courier_card_price_bounds(rarity: CardRarity, colorless: bool, membership: bool) -> (u32, u32) {
    // c-a.md S8: floor(card_bounds×.8) for Courier, floor(card_bounds×.4) for Courier+Membership.
    // Colorless first multiplies the endpoints by 1.2: Uncommon 80..98, Rare 162..198.
    match (rarity, colorless, membership) {
        (CardRarity::Common, false, false) => (36, 44),
        (CardRarity::Uncommon, false, false) => (54, 66),
        (CardRarity::Rare, false, false) => (108, 132),
        (CardRarity::Uncommon, true, false) => (64, 79),
        (CardRarity::Rare, true, false) => (129, 158),
        (CardRarity::Common, false, true) => (18, 22),
        (CardRarity::Uncommon, false, true) => (27, 33),
        (CardRarity::Rare, false, true) => (54, 66),
        (CardRarity::Uncommon, true, true) => (32, 39),
        (CardRarity::Rare, true, true) => (64, 79),
        other => panic!("unsupported Courier replacement rarity/color: {other:?}"),
    }
}

fn courier_relic_price_bounds(tier: RelicTier, membership: bool) -> (u32, u32) {
    // c-a.md S6/S8: for both relics use G(G(lo×.8)×.5)..G(G(hi×.8)×.5).
    // The raw endpoint ranges are 143..158, 238..263, and 285..315.
    match (tier, membership) {
        (RelicTier::Common | RelicTier::Shop, false) => (114, 126), // G(143×.8)..G(158×.8)
        (RelicTier::Uncommon, false) => (190, 210),                 // G(238×.8)..G(263×.8)
        (RelicTier::Rare, false) => (228, 252),                     // G(285×.8)..G(315×.8)
        (RelicTier::Common | RelicTier::Shop, true) => (57, 63),    // 114→57 .. 126→63
        (RelicTier::Uncommon, true) => (95, 105),                   // 190→95 .. 210→105
        (RelicTier::Rare, true) => (114, 126),                      // 228→114 .. 252→126
        other => panic!("unsupported Courier replacement relic tier: {other:?}"),
    }
}

fn courier_potion_price_bounds(rarity: crate::content::PotionRarity, membership: bool) -> (u32, u32) {
    // c-a.md S6/S8: raw jitter endpoints are Common 48..52, Uncommon 71..79, Rare 95..105.
    match (rarity, membership) {
        (crate::content::PotionRarity::Common, false) => (38, 42), // G(48×.8)..G(52×.8)
        (crate::content::PotionRarity::Uncommon, false) => (57, 63),
        (crate::content::PotionRarity::Rare, false) => (76, 84),
        (crate::content::PotionRarity::Common, true) => (19, 21), // G(38×.5)..G(42×.5)
        (crate::content::PotionRarity::Uncommon, true) => (29, 32),
        (crate::content::PotionRarity::Rare, true) => (38, 42),
    }
}

#[test]
fn m28_shop_courier_replacement_price_bands() {
    for character in CHARACTERS {
        for (owned, membership) in [
            (vec![RelicId::TheCourier], false),
            (vec![RelicId::TheCourier, RelicId::MembershipCard], true),
        ] {
            let template = shop(character, 154, supply(1_000, owned, None));

            let mut colored = template.clone();
            let bought = card_rows(&colored)[0].clone();
            let bought_definition = Catalog::standard().card(card_id(&bought)).unwrap();
            let replacement = courier_buy(&mut colored, bought.id);
            let definition = Catalog::standard().card(card_id(&replacement)).unwrap();
            assert_eq!(definition.color, card_color(character), "{character:?} colored replacement");
            assert_eq!(definition.card_type, bought_definition.card_type, "{character:?} colored type");
            price_range(replacement.price.unwrap(), courier_card_price_bounds(definition.rarity, false, membership));

            let mut colorless = template.clone();
            let bought = all_card_rows(&colorless).into_iter().find(|row| {
                Catalog::standard().card(card_id(row)).unwrap().color == CardColor::Colorless
            }).unwrap().clone();
            let replacement = courier_buy(&mut colorless, bought.id);
            let definition = Catalog::standard().card(card_id(&replacement)).unwrap();
            assert_eq!(definition.color, CardColor::Colorless, "{character:?} colorless replacement");
            price_range(replacement.price.unwrap(), courier_card_price_bounds(definition.rarity, true, membership));

            let mut potion = template.clone();
            let bought = potion_rows(&potion)[0].clone();
            let replacement = courier_buy(&mut potion, bought.id);
            let rarity = Catalog::standard().potion_rarity(potion_id(&replacement)).unwrap();
            price_range(replacement.price.unwrap(), courier_potion_price_bounds(rarity, membership));

            // Keep the purchase on a non-selector, non-discount relic as in S8's restock path.
            let mut selected = None;
            for seed in 0..64 {
                let candidate = reenter(&template, 50_000 + seed, None);
                let chosen = relic_rows(&candidate).into_iter().find(|row| {
                    let id = relic_id(row);
                    !matches!(id, RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado
                        | RelicId::TheCourier | RelicId::MembershipCard | RelicId::SmilingMask)
                }).map(|row| row.id);
                if let Some(chosen) = chosen {
                    selected = Some((candidate, chosen));
                    break;
                }
            }
            let (mut relic, chosen) = selected.expect("a non-selector, non-discount Merchant offer in 64 seeds");
            let replacement = courier_buy(&mut relic, chosen);
            let tier = Catalog::standard().relic_tier(relic_id(&replacement)).unwrap();
            price_range(replacement.price.unwrap(), courier_relic_price_bounds(tier, membership));
        }
    }
}

#[test]
fn m28_shop_merchant_relics_take_from_back() {
    let template = shop(Character::Ironclad, 155, supply(1_000, vec![], None));
    let mut run = template.clone();
    run.state.rng = rng(156);
    run.state.relic_pools = Some(relic_pools::explicit_for_test(
        vec![RelicId::Anchor, RelicId::BloodVial],
        vec![RelicId::Kunai, RelicId::OrnamentalFan],
        vec![RelicId::DeadBranch, RelicId::IncenseBurner],
        vec![RelicId::MembershipCard],
        vec![],
    ));
    rooms::standard::shop::enter(&mut run).unwrap();

    // c-a.md S4/R1: the Merchant rows take the current pool Back. If both rolls share a tier,
    // the second back removal is its former front; [front, back] therefore yields [back, front].
    let mut seen = [0_usize; 3];
    for row in relic_rows(&run).into_iter().take(2) {
        let tier = Catalog::standard().relic_tier(relic_id(row)).unwrap();
        let tier_index = match tier {
            RelicTier::Common => 0,
            RelicTier::Uncommon => 1,
            RelicTier::Rare => 2,
            other => panic!("unexpected Merchant tier: {other:?}"),
        };
        let position = seen[tier_index];
        let expected = match (tier, position) {
            (RelicTier::Common, 0) => RelicId::BloodVial,
            (RelicTier::Common, 1) => RelicId::Anchor,
            (RelicTier::Uncommon, 0) => RelicId::OrnamentalFan,
            (RelicTier::Uncommon, 1) => RelicId::Kunai,
            (RelicTier::Rare, 0) => RelicId::IncenseBurner,
            (RelicTier::Rare, 1) => RelicId::DeadBranch,
            other => panic!("unexpected Merchant tier occurrence: {other:?}"),
        };
        if position == 0 {
            let front = match tier {
                RelicTier::Common => RelicId::Anchor,
                RelicTier::Uncommon => RelicId::Kunai,
                RelicTier::Rare => RelicId::DeadBranch,
                other => panic!("unexpected Merchant tier: {other:?}"),
            };
            assert_ne!(relic_id(row), front, "the first {tier:?} draw must not take the front entry");
        }
        assert_eq!(relic_id(row), expected, "Merchant {tier:?} draw {position}");
        seen[tier_index] += 1;
    }
    assert_eq!(relic_rows(&run).len(), 3);
    assert_eq!(Catalog::standard().relic_tier(relic_id(relic_rows(&run)[2])), Some(RelicTier::Shop));
}

#[test]
fn m28_shop_courier_relic_skips_excluded() {
    let template = shop(Character::Ironclad, 157, supply(2_000, vec![RelicId::TheCourier], None));
    let excluded = [RelicId::OldCoin, RelicId::SmilingMask, RelicId::MawBank, RelicId::TheCourier];
    let mut selected = None;
    for seed in 0..64 {
        let candidate = reenter(&template, 60_000 + seed, None);
        let chosen = relic_rows(&candidate).into_iter().find(|row| {
            let id = relic_id(row);
            !matches!(id, RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado
                | RelicId::TheCourier | RelicId::MembershipCard | RelicId::SmilingMask)
                && ![RelicId::Anchor, RelicId::Kunai, RelicId::DeadBranch].contains(&id)
                && !excluded.contains(&id)
        }).map(|row| row.id);
        if let Some(chosen) = chosen {
            selected = Some((candidate, chosen));
            break;
        }
    }
    let (mut run, chosen) = selected.expect("a safe initial Merchant relic in 64 seeds");
    run.state.relic_pools = Some(relic_pools::explicit_for_test(
        vec![RelicId::Anchor, RelicId::OldCoin, RelicId::SmilingMask, RelicId::MawBank, RelicId::TheCourier],
        vec![RelicId::Kunai, RelicId::OldCoin, RelicId::SmilingMask, RelicId::MawBank, RelicId::TheCourier],
        vec![RelicId::DeadBranch, RelicId::OldCoin, RelicId::SmilingMask, RelicId::MawBank, RelicId::TheCourier],
        vec![], vec![],
    ));
    let replacement = relic_id(&courier_buy(&mut run, chosen));
    let expected = match Catalog::standard().relic_tier(replacement).unwrap() {
        RelicTier::Common => RelicId::Anchor,
        RelicTier::Uncommon => RelicId::Kunai,
        RelicTier::Rare => RelicId::DeadBranch,
        other => panic!("Courier replacement must come from a Merchant tier, got {other:?}"),
    };
    assert_eq!(replacement, expected, "the four tail exclusions are skipped to the legal front relic");
}

#[test]
fn m28_shop_courier_relic_restock_takes_from_back() {
    let template = shop(Character::Ironclad, 158, supply(2_000, vec![RelicId::TheCourier], None));
    let pool_relics = [RelicId::Anchor, RelicId::BloodVial, RelicId::Kunai,
        RelicId::OrnamentalFan, RelicId::DeadBranch, RelicId::IncenseBurner];
    let mut selected = None;
    for seed in 0..64 {
        let candidate = reenter(&template, 61_000 + seed, None);
        let chosen = relic_rows(&candidate).into_iter().find(|row| {
            let id = relic_id(row);
            !matches!(id, RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado
                | RelicId::TheCourier | RelicId::MembershipCard | RelicId::SmilingMask)
                && !pool_relics.contains(&id)
        }).map(|row| row.id);
        if let Some(chosen) = chosen {
            selected = Some((candidate, chosen));
            break;
        }
    }
    let (mut run, chosen) = selected.expect("a safe initial Merchant relic in 64 seeds");
    run.state.relic_pools = Some(relic_pools::explicit_for_test(
        vec![RelicId::Anchor, RelicId::BloodVial],
        vec![RelicId::Kunai, RelicId::OrnamentalFan],
        vec![RelicId::DeadBranch, RelicId::IncenseBurner],
        vec![], vec![],
    ));
    let replacement = relic_id(&courier_buy(&mut run, chosen));
    let tier = Catalog::standard().relic_tier(replacement).unwrap();
    let pools = run.state.relic_pools.as_ref().unwrap();
    let (front, back, remaining) = match tier {
        RelicTier::Common => (RelicId::Anchor, RelicId::BloodVial, &pools.common),
        RelicTier::Uncommon => (RelicId::Kunai, RelicId::OrnamentalFan, &pools.uncommon),
        RelicTier::Rare => (RelicId::DeadBranch, RelicId::IncenseBurner, &pools.rare),
        other => panic!("Courier replacement must come from a Merchant tier, got {other:?}"),
    };
    assert_eq!(replacement, back, "Courier restock takes the {tier:?} pool's back entry");
    assert_eq!(remaining.as_slice(), &[front], "the front remains and the back leaves the {tier:?} pool");
}

#[test]
fn m28_shop_discount_pass_order() {
    fn discount(price: u32, factor: f32) -> u32 { rooms::shop::libgdx_round(price as f32 * factor) }
    for character in CHARACTERS {
        let base = shop(character, 38, supply(1_000, vec![], None));
        for order in [[RelicId::TheCourier, RelicId::MembershipCard], [RelicId::MembershipCard, RelicId::TheCourier]] {
            let mut run = base.clone();
            run.state.relics.extend(order);
            let raw_rows = rows(&run).iter().map(|row| (row.kind.clone(), row.price)).collect();
            let discounted = rooms::shop::standard_screen(&mut run, raw_rows).unwrap();
            for (raw, actual) in rows(&base).iter().zip(discounted.choices.iter()) {
                if matches!(raw.kind, RoomChoiceKind::Card { .. } | RoomChoiceKind::Potion(_) | RoomChoiceKind::Relic(_)) {
                    let expected = discount(discount(raw.price.unwrap(), 0.8_f32), 0.5_f32);
                    assert_eq!(actual.price, Some(expected), "{character:?}, {order:?}, {:?}", raw.kind);
                }
            }
        }
    }
}

#[test]
fn m28_rejected_shop_choices_are_atomic() {
    for character in CHARACTERS {
        let mut poor = shop(character, 39, supply(1_000, vec![], None));
        poor.state.gold = 0;
        let card = rows(&poor).iter().find(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).unwrap().id;
        let before = poor.clone();
        assert!(poor.step(&GameAction::Choose(card)).is_err());
        assert!(poor == before, "{character:?} insufficient-gold purchase");

        let mut sozu = shop(character, 40, supply(1_000, vec![RelicId::Sozu], None));
        let potion = potion_rows(&sozu)[0].id;
        let before = sozu.clone();
        assert!(sozu.step(&GameAction::Choose(potion)).is_err());
        assert!(sozu == before, "{character:?} Sozu purchase");

        let class_potion = potion_generation::candidates(character, false).unwrap()[0];
        let mut full = shop(character, 41, supply(1_000, vec![], Some(vec![Some(class_potion); 3])));
        let potion = potion_rows(&full)[0].id;
        let before = full.clone();
        assert!(full.step(&GameAction::Choose(potion)).is_err());
        assert!(full == before, "{character:?} full-belt purchase");
    }
}

#[test]
fn m28_nested_selection_returns_shop_purge() {
    for character in CHARACTERS {
        let mut run = shop(character, 42, supply(1_000, vec![], None));
        let before_screen = rooms::public(&run).unwrap();
        let before_deck = run.state.deck.clone();
        let before_gold = run.state.gold;
        let before_rng = run.state.rng.clone();
        let purge = rows(&run).iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().id;
        run.step(&GameAction::Choose(purge)).unwrap();
        run.step(&GameAction::Cancel).unwrap();
        assert_eq!(rooms::public(&run).unwrap(), before_screen, "{character:?} returns to the exact shop screen");
        assert_eq!(run.state.gold, before_gold);
        assert_eq!(run.state.deck, before_deck);
        assert_eq!(run.state.rng, before_rng);
        let purge = rows(&run).iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
        assert_eq!(purge.status, RoomChoiceStatus::Offered);
    }
}

/// spire-0kr: with The Courier a bought card, relic or potion is replaced in place by its restock, as in the original's 13
/// fixed item slots (`ShopScreen` card list index, `StoreRelic`/`StorePotion` wrappers). The shop keeps its 15 rows, no
/// bought row stays `Claimed`, and the restock takes a fresh choice id at the bought row's index. Restock identities,
/// prices and RNG keys are unchanged: the pins below are the rows the append engine (main `4880d2d2`) added for the same
/// purchases, chosen by choice id.
#[test]
fn x0kr_courier_restocks_each_bought_slot_in_place() {
    type Pin = (u64, u64, &'static str, u32, u32);
    // (bought id, restock id, restock kind, restock price, gold after) for the 12 purchases of `plan`.
    const IRONCLAD: [Pin; 12] = [
        (2, 17, "Card { id: Headbutt, upgrades: 0 }", 18, 99_991), (17, 18, "Card { id: SwordBoomerang, upgrades: 0 }", 19, 99_973),
        (12, 19, "Potion(FirePotion)", 21, 99_952), (19, 20, "Potion(BloodPotion)", 20, 99_931),
        (9, 21, "Relic(TheBoot)", 57, 99_871), (21, 22, "Relic(OrnamentalFan)", 102, 99_814),
        (3, 23, "Card { id: Hemokinesis, upgrades: 0 }", 27, 99_794), (23, 24, "Card { id: BodySlam, upgrades: 0 }", 21, 99_767),
        (24, 25, "Card { id: Hemokinesis, upgrades: 0 }", 28, 99_746), (13, 26, "Potion(EnergyPotion)", 20, 99_726),
        (10, 27, "Relic(Shuriken)", 103, 99_665), (4, 28, "Card { id: SeeingRed, upgrades: 0 }", 28, 99_634),
    ];
    const WATCHER: [Pin; 12] = [
        (2, 17, "Card { id: BowlingBash, upgrades: 0 }", 20, 99_991), (17, 18, "Card { id: JustLucky, upgrades: 0 }", 21, 99_971),
        (12, 19, "Potion(EntropicBrew)", 40, 99_952), (19, 20, "Potion(SwiftPotion)", 21, 99_912),
        (9, 21, "Relic(TungstenRod)", 120, 99_811), (21, 22, "Relic(Damaru)", 60, 99_691),
        (3, 23, "Card { id: Conclude, upgrades: 0 }", 28, 99_671), (23, 24, "Card { id: SandsOfTime, upgrades: 0 }", 27, 99_643),
        (24, 25, "Card { id: Consecrate, upgrades: 0 }", 21, 99_616), (13, 26, "Potion(StrengthPotion)", 20, 99_596),
        (10, 27, "Relic(MummifiedHand)", 99, 99_538), (4, 28, "Card { id: Protect, upgrades: 0 }", 20, 99_520),
    ];
    let card = |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Card { .. });
    let potion = |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Potion(_));
    let relic = |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Relic(_));
    // (row kind, newest id): oldest-id rows are opening stock, newest-id rows are earlier restocks.
    let plan: [(&dyn Fn(&RoomChoiceKind) -> bool, bool); 12] = [(&card, false), (&card, true), (&potion, false), (&potion, true),
        (&relic, false), (&relic, true), (&card, false), (&card, true), (&card, true), (&potion, false), (&relic, false), (&card, false)];
    for (character, seed, pins) in [(Character::Ironclad, 41, IRONCLAD), (Character::Watcher, 7, WATCHER)] {
        let mut run = shop(character, seed, supply(100_000, vec![RelicId::TheCourier, RelicId::MembershipCard], Some(vec![None; 5])));
        assert_eq!(rows(&run).len(), 15, "5 class cards, 2 colorless, 3 relics, 3 potions, purge and leave");
        for ((predicate, newest), (bought, restock, kind, price, gold)) in plan.iter().zip(pins) {
            let offered = rows(&run).iter().filter(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind)).map(|row| row.id);
            let id = if *newest { offered.max() } else { offered.min() }.unwrap();
            assert_eq!(id.0, bought, "{character:?}");
            let index = rows(&run).iter().position(|row| row.id == id).unwrap();
            let before = rows(&run).to_vec();
            run.step(&GameAction::Choose(id)).unwrap();
            let after = rows(&run);
            assert_eq!(after.len(), 15, "{character:?}: the restock replaces the bought row; the shop never grows");
            assert_eq!((after[index].id.0, format!("{:?}", after[index].kind).as_str(), after[index].price, after[index].status),
                (restock, kind, Some(price), RoomChoiceStatus::Offered), "{character:?} bought {bought}");
            assert!(after.iter().zip(&before).enumerate().filter(|(i, _)| *i != index).all(|(_, (a, b))| a.id == b.id && a.kind == b.kind),
                "{character:?}: every other row keeps its place");
            assert!(after.iter().all(|row| row.status == RoomChoiceStatus::Offered), "{character:?}: no bought row stays claimed");
            assert_eq!(run.state.gold, gold);
        }
    }
}

/// spire-0kr keeps every other shop path: without The Courier a bought card, relic or potion row stays `Claimed` and
/// nothing is restocked (the original removes it; no offered stock remains either way); with The Courier the purge is a
/// service, not an item slot, so its row is claimed and never restocked.
#[test]
fn x0kr_without_the_courier_bought_rows_stay_claimed_and_the_purge_is_never_restocked() {
    for (relics, restocks) in [(vec![], false), (vec![RelicId::TheCourier], true)] {
        let mut run = shop(Character::Ironclad, 41, supply(100_000, relics, Some(vec![None; 5])));
        let opening = rows(&run).to_vec();
        for predicate in [|kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Card { .. }),
            |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Relic(_)),
            |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Potion(_))] {
            let id = rows(&run).iter().find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind)
                && !matches!(row.kind, RoomChoiceKind::Relic(RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado))).unwrap().id;
            let index = rows(&run).iter().position(|row| row.id == id).unwrap();
            run.step(&GameAction::Choose(id)).unwrap();
            assert_eq!(rows(&run).len(), 15);
            let row = &rows(&run)[index];
            if restocks { assert!(row.id != id && row.status == RoomChoiceStatus::Offered) } else { assert_eq!((row.id, row.status), (id, RoomChoiceStatus::Claimed)) }
        }
        let purge = rows(&run).iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().id;
        run.step(&GameAction::Choose(purge)).unwrap();
        let card = run.state.pending.as_ref().unwrap().selection.candidates[0].id;
        run.step(&GameAction::Select(card)).unwrap();
        assert_eq!(rows(&run).len(), 15);
        assert_eq!(rows(&run).iter().find(|row| row.id == purge).unwrap().status, RoomChoiceStatus::Claimed, "the purge is claimed, not restocked");
        assert_eq!(rows(&run).iter().filter(|row| row.status == RoomChoiceStatus::Claimed).count(), if restocks { 1 } else { 4 });
        assert!(rows(&run).iter().zip(&opening).all(|(row, open)| restocks || row.id == open.id), "no row moves or appears");
    }
}

/// spire-0kr (PM ruling, Circlet): with the Rare pool exhausted the Merchant relic draw falls back to Circlet, as the
/// original does, and the shop prices it from its Special tier base 400 (original `AbstractRelic.getPrice`), then the usual
/// jitter and discounts. Shop entry and a Courier relic restock used to fail with `InvalidAction` there.
#[test]
fn x0kr_exhausted_relic_pools_offer_circlet_at_its_special_price() {
    let empty = || Some(relic_pools::explicit_for_test(vec![], vec![], vec![], vec![], vec![]));
    for (owned, (low, high)) in [(vec![], (380, 420)), (vec![RelicId::TheCourier], (304, 336))] {
        // Shop entry: every relic slot (two Merchant tiers and the Shop tier) promotes to Circlet.
        let mut run = shop(Character::Ironclad, 155, supply(100_000, owned.clone(), Some(vec![None; 5])));
        run.state.relic_pools = empty();
        rooms::standard::shop::enter(&mut run).unwrap();
        let relics = relic_rows(&run);
        assert_eq!(relics.len(), 3);
        for row in relics {
            assert_eq!(relic_id(row), RelicId::Circlet);
            assert!((low..=high).contains(&row.price.unwrap()), "{owned:?}: Circlet price {:?}", row.price);
        }
    }
    // A Courier relic restock from exhausted pools is Circlet too; buying it stacks the Circlet counter.
    let mut run = shop(Character::Ironclad, 155, supply(100_000, vec![RelicId::TheCourier], Some(vec![None; 5])));
    run.state.relic_pools = empty();
    let bought = relic_rows(&run).into_iter().find(|row| !matches!(relic_id(row),
        RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado)).unwrap().id;
    let restock = courier_buy(&mut run, bought);
    assert_eq!(restock.kind, RoomChoiceKind::Relic(RelicId::Circlet));
    assert!((304..=336).contains(&restock.price.unwrap()), "{:?}", restock.price);
    let again = courier_buy(&mut run, restock.id);
    assert_eq!(again.kind, RoomChoiceKind::Relic(RelicId::Circlet));
    courier_buy(&mut run, again.id);
    assert_eq!(run.state.relics.iter().filter(|relic| **relic == RelicId::Circlet).count(), 1);
    assert_eq!(run.state.relic_counters.get(&RelicId::Circlet), Some(&2));
}
