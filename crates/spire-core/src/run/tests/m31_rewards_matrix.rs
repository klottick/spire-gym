//! M31 rewards and equip-offer matrix (plan items 3–7; design §§6.4–6.6). Source draws below are
//! hand-run from the cited source laws and the bound semantic keys, never read from the engine's
//! rarity or upgrade helper.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{CardId, CardRarity, CardType, Catalog, Character, RelicId};
use crate::run::rooms::standard::rewards as standard_rewards;

const SHOP: (i64, i64) = (9, 37);
const ORDINARY: (i64, i64) = (3, 37);

fn rng(seed: u64) -> RunRng { RunRng::new([seed as u8; 32], Partition::Test, seed) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn room(run: &Run) -> RoomView { rooms::public(run).unwrap() }
fn room_kind(run: &Run) -> RoomViewKind { room(run).kind }

fn standard(character: Character, ascension: u8, full: bool, seed: u64) -> Run {
    Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(seed), RoomScenario::Standard)
        .unwrap_or_else(|error| panic!("{character:?} A{ascension}: {error:?}"))
}

/// A public Standard Run past the short Neow and standing on its Act 1 map.
fn on_map(character: Character, ascension: u8, seed: u64) -> Run {
    let mut run = standard(character, ascension, false, seed);
    let choice = room(&run).choices.into_iter().find(|row| matches!(row.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
    run.step(&GameAction::Choose(choice)).unwrap();
    assert_eq!(room_kind(&run), RoomViewKind::Map);
    run
}

/// Drive a Standard Run through public room actions until the requested act's map.
fn on_act_map(character: Character, ascension: u8, seed: u64, act: u8) -> Run {
    let mut run = on_map(character, ascension, seed);
    run.state.hp = 300;
    run.state.max_hp = 300;
    run.state.gold = 999;
    if act > 1 {
        super::m28_world::drive(&mut run, |candidate| candidate.state.act == act && room_kind(candidate) == RoomViewKind::Map);
    }
    assert_eq!((run.state.act, room_kind(&run)), (act, RoomViewKind::Map));
    run
}

fn own(run: &mut Run, relic: RelicId) {
    if !run.state.relics.contains(&relic) {
        if let Some(pools) = run.state.relic_pools.as_mut() { crate::items::relic_pools::remove(pools, relic); }
        run.state.relics.push(relic);
    }
}

fn enter_first(run: &mut Run, kind: MapNodeKind) {
    let target = run.decision().unwrap().candidates.iter().find_map(|candidate| match candidate.action {
        GameAction::Move(id) => Some(id), _ => None,
    }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node| node.id == target).unwrap().kind = kind;
    run.step(&GameAction::Move(target)).unwrap();
}

fn shop_with(character: Character, ascension: u8, seed: u64, relic: RelicId, owned: &[RelicId]) -> (Run, ChoiceId) {
    let mut run = on_act_map(character, ascension, seed, 1);
    for relic in owned { own(&mut run, *relic); }
    run.state.gold = 999;
    enter_first(&mut run, MapNodeKind::Shop);
    let row = run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut()
        .find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
    row.kind = RoomChoiceKind::Relic(relic);
    let id = row.id;
    (run, id)
}

fn equip_key(run: &Run, choice: ChoiceId, row: u64) -> Vec<u64> {
    let node = run.state.map.as_ref().and_then(|map| map.current).unwrap_or(NodeId(u64::MAX));
    vec![u64::from(run.state.act), u64::from(run.state.floor), node.0, choice.0, row, 0xA25]
}

/// Source rarity law, with the current offset and N'loth multiplier applied by hand.
fn hand_rarity(draw: u64, offset: i32, thresholds: (i64, i64), rare_factor: u32) -> CardRarity {
    let roll = draw as i64 + i64::from(offset);
    let rare = thresholds.0 * i64::from(rare_factor);
    if roll < rare { CardRarity::Rare }
    else if roll < rare + thresholds.1 { CardRarity::Uncommon }
    else { CardRarity::Common }
}

fn hand_row(rng: &mut RunRng, key: &[u64], choices: usize, offset: &mut i32,
    thresholds: (i64, i64), rare_factor: u32) -> Vec<CardRarity> {
    (0..choices as u64).map(|slot| {
        let mut rarity_key = key.to_vec();
        rarity_key.extend([0x5241, slot]);
        let draw = rng.bounded_scoped_checked(RunRngScope::M28CardRarity, &rarity_key, 100).unwrap() as u64;
        let rarity = hand_rarity(draw, *offset, thresholds, rare_factor);
        match rarity {
            CardRarity::Rare => *offset = 5,
            CardRarity::Common => *offset = (*offset - 1).max(-40),
            _ => (),
        }
        rarity
    }).collect()
}

fn rarity_rows(run: &Run) -> Vec<Vec<CardRarity>> {
    captured_rows(run).iter().map(|row| row.iter().filter_map(|kind| match kind {
        RoomChoiceKind::Card { id, .. } => Some(Catalog::standard().card(*id).unwrap().rarity),
        _ => None,
    }).collect()).collect()
}

/// Captured children of an active offer, grouped at Skip boundaries.
fn captured_rows(run: &Run) -> Vec<Vec<RoomChoiceKind>> {
    let cards = &run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap().cards;
    let mut rows = vec![vec![]];
    for choice in &cards.choices {
        if choice.kind == RoomChoiceKind::Skip { rows.push(vec![]); }
        else { rows.last_mut().unwrap().push(choice.kind.clone()); }
    }
    rows.pop();
    rows
}

fn card_at(run: &Run, row: usize, slot: usize) -> (CardId, u32) {
    match captured_rows(run)[row][slot] {
        RoomChoiceKind::Card { id, upgrades } => (id, upgrades),
        ref other => panic!("expected card at row {row}, slot {slot}; saw {other:?}"),
    }
}

fn upgradeable(id: CardId) -> bool {
    crate::combat::cards::ironclad::rules::can_upgrade(id, 0)
        && crate::content::complete::supports_permanent(id)
}

fn raw_upgrade(seed: u64, key: &[u64], slot: u64) -> u32 {
    let mut draw_key = key.to_vec();
    draw_key.extend([0x0F, slot]);
    rng(seed).bounded_scoped_checked(RunRngScope::M28CardRarity, &draw_key, 1_u64 << 24).unwrap() as u32
}

fn roll24(raw: u32) -> f32 { raw as f32 / 16_777_216.0_f32 }

fn click(run: &mut Run, id: ChoiceId) { run.step(&GameAction::Choose(id)).unwrap(); }

/// Act 2 boss chest with a deterministic requested Boss-tier relic at the front of the pool.
fn act2_boss_choice(character: Character, ascension: u8, seed: u64, wanted: RelicId, owned: &[RelicId]) -> (Run, ChoiceId) {
    let mut run = on_act_map(character, ascension, seed, 1);
    // The public route reaches the real Act 2 boss chest. This HP fixture survives traversal and leaves room for Tiny House's +5.
    run.state.hp = 300;
    run.state.max_hp = 300;
    super::m28_world::drive(&mut run, |candidate| candidate.state.act == 2 && room_kind(candidate) == RoomViewKind::BossTreasure);
    assert_eq!((run.state.act, room_kind(&run)), (2, RoomViewKind::BossTreasure));
    for relic in owned { own(&mut run, *relic); }
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![], vec![], vec![], vec![], vec![wanted, RelicId::SneckoEye, RelicId::BlackStar, RelicId::Sozu],
    ));
    let open = room(&run).choices.iter().find(|row| row.kind == RoomChoiceKind::OpenChest).unwrap().id;
    click(&mut run, open);
    let choice = room(&run).choices.iter().find(|row| row.kind == RoomChoiceKind::Relic(wanted)).unwrap().id;
    (run, choice)
}

fn force_neow_boss_swap(run: &mut Run) -> ChoiceId {
    let candidate = run.candidates.iter_mut().find(|candidate| {
        candidate.neow_offer.as_ref().is_some_and(|offer| offer.category == 3)
    }).expect("full Neow category 3");
    let offer = NeowOffer { category: 3, reward: NeowReward::BossSwap, drawback: None };
    candidate.neow_offer = Some(offer);
    let id = candidate.id;
    run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|row| row.id == id).unwrap().kind =
        RoomChoiceKind::NeowBlessing { category: 3, reward: NeowReward::BossSwap, drawback: None };
    id
}

fn neow_tiny_house(character: Character, seed: u64) -> (Run, ChoiceId) {
    let mut run = standard(character, 0, true, seed);
    own(&mut run, RelicId::NlothsGift);
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![], vec![], vec![], vec![], vec![RelicId::TinyHouse, RelicId::SneckoEye, RelicId::BlackStar],
    ));
    let choice = force_neow_boss_swap(&mut run);
    (run, choice)
}

fn act2_boss_reward(character: Character, ascension: u8, seed: u64) -> Run {
    let mut run = on_map(character, ascension, seed);
    run.state.hp = 999;
    run.state.max_hp = 999;
    run.state.gold = 999;
    super::m28_world::drive(&mut run, |candidate| {
        candidate.state.act == 2 && room_kind(candidate) == RoomViewKind::Rewards
            && candidate.state.map.as_ref().and_then(|map| map.current).is_some_and(|node| {
                candidate.state.map.as_ref().unwrap().nodes.iter().any(|row| row.id == node && row.kind == MapNodeKind::Boss)
            })
    });
    run
}

fn gold_rows(run: &Run) -> Vec<u32> {
    room(run).choices.into_iter().filter_map(|row| match row.kind {
        RoomChoiceKind::Gold(amount) => Some(amount), _ => None,
    }).collect()
}

fn combat_key(run: &Run, offer: u64) -> Vec<u64> {
    let active = run.state.active_combat.as_ref().expect("active combat");
    let crate::run::CombatOrigin::MapRoom { node, kind } = active.origin else { panic!("map combat") };
    vec![u64::from(run.state.act), u64::from(run.state.floor), node.0, active.visit, kind as u64, 0xCA, offer]
}

fn start_combat_reward(character: Character, ascension: u8, seed: u64, act: u8) -> Run {
    let mut run = on_act_map(character, ascension, seed, act);
    enter_first(&mut run, MapNodeKind::Monster);
    assert!(run.state.combat.is_some());
    run
}

fn settle_combat_reward(mut run: Run, seed: u64) -> Run {
    run.state.rng = rng(seed);
    super::m28_world::win(&mut run);
    run
}

/// Search seeded rows for a non-Rare, upgradeable card with an upgrade draw in [low, high).
fn find_equip_boundary(base: &Run, choice: ChoiceId, offer_row: usize, thresholds: (i64, i64), rare_factor: u32,
    offset: i32, low: f32, high: f32) -> (u64, usize) {
    let key = equip_key(base, choice, offer_row as u64);
    for seed in 0..8192_u64 {
        let mut expected_offset = offset;
        let expected = hand_row(&mut rng(seed), &key, 3, &mut expected_offset, thresholds, rare_factor);
        let candidate_slots: Vec<_> = expected.iter().enumerate().filter_map(|(slot, rarity)| {
            if *rarity == CardRarity::Rare { return None; }
            let roll = roll24(raw_upgrade(seed, &key, slot as u64));
            (roll >= low && roll < high).then_some(slot)
        }).collect();
        if candidate_slots.is_empty() { continue; }
        let mut probe = base.clone();
        probe.state.rng = rng(seed);
        click(&mut probe, choice);
        for slot in candidate_slots {
            if upgradeable(card_at(&probe, offer_row, slot).0) { return (seed, slot); }
        }
    }
    panic!("no seeded equip card hit the requested non-Rare upgrade band");
}

// Source: AbstractRoom.java:201–239; AbstractDungeon.java:1511–1565; equip-offers research §§1–3.
// Grade: Run
#[test]
fn m31_equip_tiny_house_law() {
    let (mut neow, neow_choice) = neow_tiny_house(Character::Ironclad, 0x30);
    let neow_key = equip_key(&neow, neow_choice, 0);
    let mut neow_offset = neow.state.card_rarity_offset.unwrap();
    let neow_expected = hand_row(&mut neow.state.rng.clone(), &neow_key, 3, &mut neow_offset, ORDINARY, 3);
    click(&mut neow, neow_choice);
    assert_eq!(room_kind(&neow), RoomViewKind::Rewards, "Neow's boss swap opens Tiny House's equip row over the map");
    assert_eq!(rarity_rows(&neow), vec![neow_expected], "Neow room uses 3/37 with N'loth's Gift ×3");
    assert!(captured_rows(&neow).iter().flatten().all(|kind| !matches!(kind, RoomChoiceKind::Card { upgrades: 1, .. })),
        "Act 1 has no automatic upgrade chance");

    let (mut a11, a11_choice) = act2_boss_choice(Character::Ironclad, 11, 0x31, RelicId::TinyHouse, &[RelicId::NlothsGift]);
    let (mut a12, a12_choice) = act2_boss_choice(Character::Ironclad, 12, 0x31, RelicId::TinyHouse, &[RelicId::NlothsGift]);
    assert_eq!((a11.state.act, a11.state.floor, a11.state.map.as_ref().unwrap().current, a11_choice),
        (a12.state.act, a12.state.floor, a12.state.map.as_ref().unwrap().current, a12_choice), "same Act 2 chest key");
    let start_offset = a11.state.card_rarity_offset.unwrap();
    assert_eq!(a12.state.card_rarity_offset, Some(start_offset));
    let (seed, slot) = find_equip_boundary(&a11, a11_choice, 0, ORDINARY, 3, start_offset, 0.125, 0.25);
    let key11 = equip_key(&a11, a11_choice, 0);
    let mut expected_offset = start_offset;
    let expected11 = hand_row(&mut rng(seed), &key11, 3, &mut expected_offset, ORDINARY, 3);
    a11.state.rng = rng(seed);
    a12.state.rng = rng(seed);
    click(&mut a11, a11_choice);
    click(&mut a12, a12_choice);
    assert_eq!(rarity_rows(&a11), vec![expected11.clone()], "boss chest uses 3/37 and N'loth's Gift ×3");
    assert_eq!(rarity_rows(&a12), vec![expected11], "same keyed row at A12");
    let (a11_id, a11_upgrades) = card_at(&a11, 0, slot);
    let (a12_id, a12_upgrades) = card_at(&a12, 0, slot);
    assert_eq!(a11_id, a12_id, "same seed/key selects the same card");
    assert!(upgradeable(a11_id), "searched card is upgradeable");
    assert!((0.125..0.25).contains(&roll24(raw_upgrade(seed, &key11, slot as u64))), "searched 24-bit draw lies between A11/A12");
    assert_eq!(a11_upgrades, 1, "Act 2 A11 chance is .25");
    assert_eq!(a12_upgrades, 0, "Act 2 A12 chance is .125");
    for run in [&a11, &a12] {
        for row in captured_rows(run) {
            for kind in row {
                if let RoomChoiceKind::Card { id, upgrades } = kind {
                    if Catalog::standard().card(id).unwrap().rarity == CardRarity::Rare { assert_eq!(upgrades, 0, "Rare cards never upgrade"); }
                }
            }
        }
    }
}

// Source: AbstractDungeon.java:1553–1565; Orrery.java:23–29; Egg onPreviewObtainCard hooks, equip-offers research §1.5.
// Grade: Run
#[test]
fn m31_equip_egg_preview() {
    let eggs = [RelicId::MoltenEgg, RelicId::ToxicEgg, RelicId::FrozenEgg];
    let (template, choice) = shop_with(Character::Ironclad, 0, 0x32, RelicId::Orrery, &eggs);
    let mut found_seed = None;
    for seed in 0..512_u64 {
        let mut probe = template.clone();
        probe.state.rng = rng(seed);
        click(&mut probe, choice);
        let mut seen = std::collections::BTreeSet::new();
        for row in captured_rows(&probe) {
            for kind in row {
                let RoomChoiceKind::Card { id, upgrades } = kind else { continue };
                let definition = Catalog::standard().card(id).unwrap();
                let egg = match definition.card_type {
                    CardType::Attack => Some(RelicId::MoltenEgg),
                    CardType::Skill => Some(RelicId::ToxicEgg),
                    CardType::Power => Some(RelicId::FrozenEgg),
                    _ => None,
                };
                if let Some(egg) = egg.filter(|egg| eggs.contains(egg) && upgradeable(id)) {
                    assert_eq!(upgrades, 1, "matching {egg:?} previews {id:?} as one upgrade");
                    seen.insert(egg);
                } else {
                    assert_eq!(upgrades, 0, "nonmatching or non-upgradeable card {id:?} stays at zero");
                }
            }
        }
        if seen.len() == eggs.len() { found_seed = Some(seed); break; }
    }
    assert!(found_seed.is_some(), "Orrery's five rows expose an upgradeable Attack, Skill, and Power for the three Eggs");
}

// Source: m31-equip-offers-v1.md §3; task0-bind.md §(c), keys for equip `0xA25` and combat `0xCA`.
// Grade: Run
#[test]
fn m31_equip_keys_disjoint() {
    let (template, original_choice) = shop_with(Character::Ironclad, 0, 0x33, RelicId::Orrery, &[]);
    let node = template.state.map.as_ref().unwrap().current.unwrap();
    let combat_key = vec![u64::from(template.state.act), u64::from(template.state.floor), node.0, 0,
        MapNodeKind::Monster as u64, 0xCA, 0];
    let offset = template.state.card_rarity_offset.unwrap();
    let mut base = template.state.clone();
    let combat_before = standard_rewards::ordinary_offer(&mut base, standard_rewards::RarityRoom::Ordinary, 3, 1, &combat_key).unwrap();

    for new_choice in [original_choice, ChoiceId(original_choice.0 + 0x10000)] {
        let mut equip_run = template.clone();
        if new_choice != original_choice {
            let row = equip_run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut()
                .find(|row| row.id == original_choice).unwrap();
            row.id = new_choice;
        }
        equip_run.state.rng = rng(0x33);
        click(&mut equip_run, new_choice);
        assert_eq!(captured_rows(&equip_run).len(), 5, "the equip operation generated Orrery's five rows");

        // Rebuild a combat reward from the same node and unchanged offset using its bound 0xCA key.
        // Equip generation has already consumed its own 0xA25-scoped draws in this RunRng.
        let mut after_equip = equip_run.state.clone();
        after_equip.card_rarity_offset = Some(offset);
        let combat_after = standard_rewards::ordinary_offer(&mut after_equip, standard_rewards::RarityRoom::Ordinary, 3, 1, &combat_key).unwrap();
        assert_eq!(combat_after, combat_before, "equip choice/key inputs cannot alter combat rows at node {node:?}");
    }
}

// Source: CallingBell.java:43–51; AbstractDungeon.java:1511–1539; NlothsGift.java:22–29; equip-offers research §§1–3.
// Grade: Run
#[test]
fn m31_hidden_row_calling_bell() {
    for question_card in [false, true] {
        let mut owned = vec![RelicId::NlothsGift];
        if question_card { owned.push(RelicId::QuestionCard); }
        let (mut run, choice) = act2_boss_choice(Character::Ironclad, 0, 0x34, RelicId::CallingBell, &owned);
        let start_offset = run.state.card_rarity_offset.unwrap();
        let key = {
            let node = run.state.map.as_ref().unwrap().current.unwrap();
            vec![u64::from(run.state.act), u64::from(run.state.floor), node.0, choice.0, 0, 0xA26]
        };
        let choices = if question_card { 4 } else { 3 };
        let mut offset = start_offset;
        let mut expected_rng = run.state.rng.clone();
        let _hidden = hand_row(&mut expected_rng, &key, choices, &mut offset, ORDINARY, 3);
        click(&mut run, choice);
        assert!(!room(&run).choices.iter().any(|row| row.kind == RoomChoiceKind::CardReward), "Calling Bell shows no card reward");
        assert_eq!(run.state.card_rarity_offset, Some(offset), "hidden 0xA26 row advances the source offset; Question Card={question_card}");
    }
}

// Source: ShopScreen.java:649–651; ShopRoom.java:46–47, 62–64. The engine restock key is
// `shop_node_key(run, row_index, COMPLETE_COURIER_CARD_SOURCE_TAG)` in run/rooms/shop.rs:446–449:
// `[act, floor, node, row_index, 6]`; at the first restock `row_index=7` (five colored and two colorless rows).
// Grade: Run
#[test]
fn m31_shop_courier_restock_shop_thresholds() {
    // Main, after mutation pass v1 (M23): sweep shop seeds until the restock draw separates 9/37 from 3/37.
    for seed in 0x35..=0xFF_u64 {
    let (mut run, _) = shop_with(Character::Ironclad, 0, seed, RelicId::Orrery, &[RelicId::TheCourier]);
    let screen = room(&run);
    let purchased = screen.choices.iter().find(|row| matches!(row.kind, RoomChoiceKind::Card { id, .. }
        if Catalog::standard().card(id).unwrap().card_type == CardType::Attack)).unwrap().clone();
    let node = run.state.map.as_ref().unwrap().current.unwrap();
    let row_index = screen.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).count() as u64;
    assert_eq!(row_index, 7);
    let key = [u64::from(run.state.act), u64::from(run.state.floor), node.0, row_index, 6];
    let start_offset = run.state.card_rarity_offset.unwrap();
    let draw = run.state.rng.clone().bounded_scoped_checked(RunRngScope::M28ShopStock, &key, 100).unwrap() as u64;
    let expected = hand_rarity(draw, start_offset, SHOP, 1);
    if expected == hand_rarity(draw, start_offset, (3, 37), 1) { continue; }
    let index = screen.choices.iter().position(|row| row.id == purchased.id).unwrap();
    click(&mut run, purchased.id);
    // spire-0kr: the restock replaces the bought row in place and keeps the first restock key (row index 7).
    let RoomChoiceKind::Card { id: restocked, .. } = room(&run).choices[index].kind else { panic!("card restock in place") };
    assert_eq!(Catalog::standard().card(restocked).unwrap().card_type, CardType::Attack, "Courier keeps the purchased type");
    assert_eq!(Catalog::standard().card(restocked).unwrap().rarity, expected, "restock uses the ShopRoom 9/37 threshold");
    assert_eq!(run.state.card_rarity_offset, Some(start_offset), "Courier reads the offset without changing it");
    return;
    }
    panic!("no shop seed separated the restock thresholds");
}

// Source: TheCity.java:94; TheBeyond.java:89; TheEnding.java:173; AbstractDungeon.java:1553–1559.
// Grade: Combat
#[test]
fn m31_a12_reward_upgrade_boundary() {
    for act in [2_u8, 3] {
        let a11 = start_combat_reward(Character::Ironclad, 11, 0x36, act);
        let a12 = start_combat_reward(Character::Ironclad, 12, 0x36, act);
        let key11 = combat_key(&a11, 0);
        let key12 = combat_key(&a12, 0);
        assert_eq!(key11, key12, "A11/A12 use the same Act {act} combat row key");
        assert_eq!(a11.state.card_rarity_offset, a12.state.card_rarity_offset);
        let offset = a11.state.card_rarity_offset.unwrap();
        let low = if act == 2 { 0.125 } else { 0.25 };
        let high = if act == 2 { 0.25 } else { 0.5 };
        let mut found = None;
        for seed in 0..8192_u64 {
            let mut expected_offset = offset;
            let rarities = hand_row(&mut rng(seed), &key11, 3, &mut expected_offset, ORDINARY, 1);
            let slots: Vec<_> = rarities.iter().enumerate().filter_map(|(slot, rarity)| {
                if *rarity == CardRarity::Rare { return None; }
                let roll = roll24(raw_upgrade(seed, &key11, slot as u64));
                (roll >= low && roll < high).then_some(slot)
            }).collect();
            if slots.is_empty() { continue; }
            let probe = settle_combat_reward(a11.clone(), seed);
            if let Some(slot) = slots.into_iter().find(|slot| upgradeable(card_at(&probe, 0, *slot).0)) {
                found = Some((seed, slot, rarities));
                break;
            }
        }
        let (seed, slot, expected_rarities) = found.unwrap_or_else(|| panic!("Act {act}: no boundary seed found"));
        let a11 = settle_combat_reward(a11, seed);
        let a12 = settle_combat_reward(a12, seed);
        assert_eq!(rarity_rows(&a11), vec![expected_rarities.clone()], "Act {act} source rarity row");
        assert_eq!(rarity_rows(&a12), vec![expected_rarities], "Act {act} same keyed rarity row");
        let (id11, upgrades11) = card_at(&a11, 0, slot);
        let (id12, upgrades12) = card_at(&a12, 0, slot);
        assert_eq!(id11, id12);
        assert!((low..high).contains(&roll24(raw_upgrade(seed, &key11, slot as u64))));
        assert_eq!(upgrades11, 1, "A11 Act {act} chance is {high}");
        assert_eq!(upgrades12, 0, "A12 Act {act} chance is {low}");
        for run in [&a11, &a12] {
            for row in captured_rows(run) {
                for kind in row {
                    if let RoomChoiceKind::Card { id, upgrades } = kind {
                        if Catalog::standard().card(id).unwrap().rarity == CardRarity::Rare {
                            assert_eq!(upgrades, 0, "Act {act}: Rare cards never upgrade");
                        }
                    }
                }
            }
        }
    }
}

// Source: TheBeyond.java:89–91; AbstractDungeon.java:1553–1565; Orrery.java:23–29.
// Grade: Run
#[test]
fn m31_a12_equip_offer_upgrade() {
    // Enter the same Standard Act 3 shop through public act transitions.
    let mut a11 = on_act_map(Character::Ironclad, 11, 0x37, 3);
    let mut a12 = on_act_map(Character::Ironclad, 12, 0x37, 3);
    a11.state.gold = 999; a12.state.gold = 999;
    enter_first(&mut a11, MapNodeKind::Shop);
    enter_first(&mut a12, MapNodeKind::Shop);
    let choice11 = {
        let row = a11.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
        row.kind = RoomChoiceKind::Relic(RelicId::Orrery); row.id
    };
    let choice12 = {
        let row = a12.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
        row.kind = RoomChoiceKind::Relic(RelicId::Orrery); row.id
    };
    assert_eq!(equip_key(&a11, choice11, 0), equip_key(&a12, choice12, 0), "same Act 3 equip key");
    let offset = a11.state.card_rarity_offset.unwrap();
    assert_eq!(a12.state.card_rarity_offset, Some(offset));
    let (seed, slot) = find_equip_boundary(&a11, choice11, 0, SHOP, 1, offset, 0.25, 0.5);
    let key = equip_key(&a11, choice11, 0);
    a11.state.rng = rng(seed); a12.state.rng = rng(seed);
    click(&mut a11, choice11); click(&mut a12, choice12);
    let (id11, upgrades11) = card_at(&a11, 0, slot);
    let (id12, upgrades12) = card_at(&a12, 0, slot);
    assert_eq!(id11, id12);
    assert!(upgradeable(id11));
    assert!((0.25..0.5).contains(&roll24(raw_upgrade(seed, &key, slot as u64))));
    assert_eq!(upgrades11, 1, "Act 3 A11 chance is .5");
    assert_eq!(upgrades12, 0, "Act 3 A12 chance is .25");
}

// Source: AbstractRoom.java:339–350; source check §D; design §6.6.
// Grade: Combat
#[test]
fn m31_a13_boss_gold_act2() {
    let a12 = act2_boss_reward(Character::Ironclad, 12, 0x38);
    let a13 = act2_boss_reward(Character::Ironclad, 13, 0x38);
    assert_eq!((a12.state.act, a12.state.floor), (2, a13.state.floor));
    let base = gold_rows(&a12);
    let reduced = gold_rows(&a13);
    assert_eq!(base.len(), 1);
    assert_eq!(reduced.len(), 1);
    assert!((95..=105).contains(&base[0]), "Act 2 A12 boss base is 100 ± 5");
    const REDUCED: [u32; 11] = [71, 72, 73, 74, 74, 75, 76, 77, 77, 78, 79];
    assert_eq!(reduced[0], REDUCED[(base[0] - 95) as usize], "same-seed A13 boss row is round(binary32(base × .75))");
}

// Source: AbstractRoom.java:339–350; source check §D says the A13 branch is boss-only.
// Grade: Combat
#[test]
fn m31_a13_elite_and_normal_gold_unchanged() {
    for kind in [MapNodeKind::Monster, MapNodeKind::Elite] {
        let mut a12 = on_act_map(Character::Ironclad, 12, 0x39, 2);
        let mut a13 = on_act_map(Character::Ironclad, 13, 0x39, 2);
        enter_first(&mut a12, kind);
        enter_first(&mut a13, kind);
        let a12 = settle_combat_reward(a12, 0x39);
        let a13 = settle_combat_reward(a13, 0x39);
        assert_eq!(gold_rows(&a12), gold_rows(&a13), "same-seed Act 2 {kind:?} gold is unchanged at A13");
        let gold = gold_rows(&a12);
        assert_eq!(gold.len(), 1);
        if kind == MapNodeKind::Monster { assert!((10..=20).contains(&gold[0])); }
        else { assert!((25..=35).contains(&gold[0])); }
    }
}
