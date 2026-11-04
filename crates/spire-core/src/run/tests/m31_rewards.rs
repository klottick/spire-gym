//! M31 card rewards (plan items 3–4; design §6.4–§6.5). G1: on the Standard Run, the equip card rows
//! (Orrery, Tiny House, Dream Catcher) and the Cauldron / Calling Bell hidden rows follow the ordinary
//! `getRewardCards` law on the current room's thresholds (equip-offers research §1–§4). A12: the
//! upgrade chance halves in Acts 2–4 (source check §C). Expected rarities are a hand-run of the keyed
//! rarity draws against the source thresholds (ShopRoom 9/37, RestRoom 3/37, AbstractRoom 3/37),
//! never the classifier under test.
use super::super::*;
use super::m31_potion_slots::{enter_first, on_map};
use crate::content::{CardRarity, Catalog, Character, RelicId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

/// Source thresholds (rare, uncommon width) by room (equip-offers §1.2).
const SHOP: (i64, i64) = (9, 37);
const REST: (i64, i64) = (3, 37);

fn hand_rarity(n: u64, offset: i32, (rare, uncommon): (i64, i64)) -> CardRarity {
    let roll = n as i64 + i64::from(offset);
    if roll < rare { CardRarity::Rare } else if roll < rare + uncommon { CardRarity::Uncommon } else { CardRarity::Common }
}
/// One row's rarities from the pre-offer RNG, writing the offset as the source does (Rare → 5,
/// Common → max(−40, b−1), Uncommon unchanged).
fn hand_row(rng: &mut RunRng, key: &[u64], choices: usize, offset: &mut i32, room: (i64, i64)) -> Vec<CardRarity> {
    (0..choices as u64).map(|slot| {
        let n = rng.bounded_scoped_checked(RunRngScope::M28CardRarity, &[key, &[0x5241, slot]].concat(), 100).unwrap() as u64;
        let rarity = hand_rarity(n, *offset, room);
        match rarity { CardRarity::Rare => *offset = 5, CardRarity::Common => *offset = (*offset - 1).max(-40), _ => () }
        rarity
    }).collect()
}
fn screen(run: &Run) -> RoomView { run.state.room_loop.as_ref().unwrap().screen.clone() }
/// The captured card rows of the open offer, each without its trailing Skip / Singing Bowl.
fn captured_rows(run: &Run) -> Vec<Vec<RoomChoiceKind>> {
    let cards = &run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap().cards;
    let mut rows = vec![vec![]];
    for choice in &cards.choices {
        if choice.kind == RoomChoiceKind::Skip { rows.push(vec![]) } else { rows.last_mut().unwrap().push(choice.kind.clone()) }
    }
    rows.pop();
    rows
}
fn rarities(row: &[RoomChoiceKind]) -> Vec<CardRarity> {
    row.iter().filter_map(|kind| match kind { RoomChoiceKind::Card { id, .. } => Some(Catalog::standard().card(*id).unwrap().rarity), _ => None }).collect()
}
fn own(run: &mut Run, relic: RelicId) {
    if let Some(pools) = run.state.relic_pools.as_mut() { crate::items::relic_pools::remove(pools, relic); }
    run.state.relics.push(relic);
}
/// A Standard Run in a shop whose first relic row is `relic`; returns the row id.
fn shop_with(character: Character, ascension: u8, relic: RelicId, owned: &[RelicId]) -> (Run, ChoiceId) {
    let mut run = on_map(character, ascension);
    for relic in owned { own(&mut run, *relic); }
    run.state.gold = 999;
    enter_first(&mut run, MapNodeKind::Shop);
    let screen = &mut run.state.room_loop.as_mut().unwrap().screen;
    let row = screen.choices.iter_mut().find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
    row.kind = RoomChoiceKind::Relic(relic);
    let id = row.id;
    (run, id)
}
fn equip_key(run: &Run, choice: ChoiceId, row: u64, tag: u64) -> Vec<u64> {
    let node = run.state.map.as_ref().unwrap().current.unwrap();
    vec![u64::from(run.state.act), u64::from(run.state.floor), node.0, choice.0, row, tag]
}

#[test]
fn m31_a12_upgrade_chance_table() {
    use rooms::standard::rewards::upgrade_chance;
    // Source check §C (binary32 dungeon fields): Exordium 0; City .25 / .125; Beyond and Ending .5 / .25.
    for (act, a11, a12) in [(1_u8, 0.0_f32, 0.0_f32), (2, 0.25, 0.125), (3, 0.5, 0.25), (4, 0.5, 0.25)] {
        for ascension in [0_u8, 11] { assert_eq!(upgrade_chance(act, ascension).to_bits(), a11.to_bits(), "act {act} A{ascension}"); }
        for ascension in [12_u8, 15, 20] { assert_eq!(upgrade_chance(act, ascension).to_bits(), a12.to_bits(), "act {act} A{ascension}"); }
    }
}

/// The ordinary-room thresholds, the wrong answer a mis-routed shop or rest row would give.
const ORDINARY: (i64, i64) = (3, 37);
/// RestRoom with the relic hooks wrongly on and N'loth's Gift owned (3 × 3).
const REST_WITH_NLOTH: (i64, i64) = (9, 37);
fn seeded(run: &mut Run, seed: u8) { run.state.rng = RunRng::new([seed; 32], crate::combat::rng::Partition::Test, u64::from(seed)); }

#[test]
fn m31_equip_orrery_shop_law() {
    // Mutation pass v1 (M10): seeds are swept so some draw falls between the shop and ordinary bands.
    let mut discriminating = 0;
    for character in CHARACTERS {
        for (seed, (owned, choices)) in [(vec![], 3_usize), (vec![RelicId::QuestionCard], 4), (vec![RelicId::BustedCrown], 1),
            (vec![RelicId::PrayerWheel], 3), (vec![RelicId::NlothsGift], 3)].into_iter().enumerate() {
            let (mut run, id) = shop_with(character, 0, RelicId::Orrery, &owned);
            seeded(&mut run, seed as u8 + 8 * character as u8);
            run.state.card_rarity_offset = Some(-20);
            let (mut rng, mut offset) = (run.state.rng.clone(), -20);
            let expected: Vec<Vec<CardRarity>> = (0..5).map(|row| hand_row(&mut rng, &equip_key(&run, id, row, 0xA25), choices, &mut offset, SHOP)).collect();
            let (mut rng, mut wrong) = (run.state.rng.clone(), -20);
            let ordinary: Vec<Vec<CardRarity>> = (0..5).map(|row| hand_row(&mut rng, &equip_key(&run, id, row, 0xA25), choices, &mut wrong, ORDINARY)).collect();
            discriminating += u32::from(ordinary != expected);
            run.step(&GameAction::Choose(id)).unwrap();
            assert_eq!(screen(&run).choices.iter().filter(|c| c.kind == RoomChoiceKind::CardReward).count(), 5,
                "{character:?} {owned:?}: five rows, Prayer Wheel adds none");
            let rows = captured_rows(&run);
            assert_eq!(rows.iter().map(|row| rarities(row)).collect::<Vec<_>>(), expected,
                "{character:?} {owned:?}: shop 9/37 (hooks off: N'loth's Gift does not apply)");
            assert_eq!(run.state.card_rarity_offset, Some(offset), "{character:?} {owned:?}: the shared offset moved");
        }
    }
    assert!(discriminating > 0, "some case separates ShopRoom 9/37 from 3/37");
}

#[test]
fn m31_hidden_row_moves_offset() {
    for character in CHARACTERS {
        for (owned, choices) in [(vec![], 3_usize), (vec![RelicId::QuestionCard], 4)] {
            let (mut run, id) = shop_with(character, 0, RelicId::Cauldron, &owned);
            run.state.card_rarity_offset = Some(-20);
            let (mut rng, mut offset) = (run.state.rng.clone(), -20);
            hand_row(&mut rng, &equip_key(&run, id, 0, 0xA26), choices, &mut offset, SHOP);
            run.step(&GameAction::Choose(id)).unwrap();
            assert!(captured_rows(&run).is_empty(), "{character:?}: Cauldron offers no card row");
            assert!(!screen(&run).choices.iter().any(|c| c.kind == RoomChoiceKind::CardReward));
            assert_eq!(run.state.card_rarity_offset, Some(offset), "{character:?} {owned:?}: the hidden row moved the offset");
        }
    }
}

#[test]
fn m31_equip_dream_catcher_law() {
    // Mutation pass v1 (M11): seeds are swept so some draw separates hooks off (3) from N'loth's ×3 (9).
    let mut discriminating = 0;
    for character in CHARACTERS { for seed in 0..6_u8 {
        let mut run = on_map(character, 0);
        for relic in [RelicId::DreamCatcher, RelicId::SingingBowl, RelicId::NlothsGift] { own(&mut run, relic); }
        run.state.hp = 1;
        run.state.card_rarity_offset = Some(-30);
        enter_first(&mut run, MapNodeKind::Rest);
        seeded(&mut run, seed + 8 * character as u8);
        let rest = screen(&run).choices.iter().find(|c| matches!(c.kind, RoomChoiceKind::Rest { .. })).unwrap().id;
        let (mut rng, mut offset) = (run.state.rng.clone(), -30);
        let expected = hand_row(&mut rng, &equip_key(&run, rest, 0, 0xA25), 3, &mut offset, REST);
        let (mut rng, mut wrong) = (run.state.rng.clone(), -30);
        discriminating += u32::from(hand_row(&mut rng, &equip_key(&run, rest, 0, 0xA25), 3, &mut wrong, REST_WITH_NLOTH) != expected);
        run.step(&GameAction::Choose(rest)).unwrap();
        let cards = &run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap().cards;
        assert!(cards.choices.iter().any(|c| c.kind == RoomChoiceKind::SingingBowl), "{character:?}: Singing Bowl offered");
        assert_eq!(captured_rows(&run).iter().map(|row| rarities(row)).collect::<Vec<_>>(), vec![expected],
            "{character:?}: rest 3/37, hooks off");
        assert_eq!(run.state.card_rarity_offset, Some(offset));
    } }
    assert!(discriminating > 0, "some case separates hooks off from N'loth's Gift");
}

/// Review I1: a non-Standard Complete Run has no live offset and keeps the legacy fixed C/U/U row
/// with no upgrades (the M27 `settled` + shop `offer` fixture, copied).
#[test]
fn m31_legacy_equip_row_on_diagnostics() {
    use crate::combat::{CombatAction, TargetId};
    use crate::content::{EncounterId, PotionId};
    let mut run = crate::items::tests::m27_support::complete_run(Character::Ironclad, vec![], vec![Some(PotionId::FirePotion); 3],
        std::collections::BTreeMap::new(), EncounterId::Cultist);
    for slot in 0..3 { run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) }).unwrap(); }
    assert!(run.state.combat.is_none());
    assert_eq!(run.state.card_rarity_offset, None);
    let node = NodeId(1);
    (run.state.act, run.state.floor, run.state.gold) = (1, 1, 150);
    run.state.map = Some(RunMap { act: 1, current: Some(node), nodes: vec![MapNode { id: node, floor: 1, label: "m31 legacy equip fixture",
        edges: vec![], x: 0, y: 0, kind: MapNodeKind::Shop, hidden_kind: None, visited: true, revealed: true, encounter: None, burning: false }],
        ..RunMap::default() });
    let choices = vec![
        RoomChoiceView { id: ChoiceId(100), kind: RoomChoiceKind::Relic(RelicId::Orrery), price: Some(150), status: RoomChoiceStatus::Offered },
        RoomChoiceView { id: ChoiceId(101), kind: RoomChoiceKind::Leave, price: None, status: RoomChoiceStatus::Offered },
    ];
    run.next_choice = 102;
    let scenario = run.state.room_loop.as_ref().map_or(RoomScenario::M5Act1Captured, |room| room.scenario);
    run.state.room_loop = Some(rooms::RoomLoop { scenario, act1: None, act2: None, act3: None, act4: None, m10_elite_relic_cursor: 0,
        screen: RoomView { kind: RoomViewKind::Shop, choices }, purge_cost: 75, reward_parent: None, active_rewards: None,
        boss: None, transition: None, act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0, act3_event: None,
        act3_end: None, standard_event: None });
    run.step(&GameAction::Choose(ChoiceId(100))).unwrap();
    let rows = captured_rows(&run);
    assert_eq!(rows.len(), 5);
    for row in &rows {
        assert_eq!(rarities(row), [CardRarity::Common, CardRarity::Uncommon, CardRarity::Uncommon]);
        assert!(row.iter().all(|kind| matches!(kind, RoomChoiceKind::Card { upgrades: 0, .. })));
    }
    assert_eq!(run.state.card_rarity_offset, None);
}

/// G3 (PM ruling m31/pm-ruling-g3-v1.md): the Merchant rolls its coloured cards with `rollRarity()` in
/// the ShopRoom, 9/37 without the relic hooks (ShopRoom.java:46–47, :62–64). Slots 0, 2 and 4 (the
/// first Attack, first Skill and the Power, which have no same-type redraw) are checked against a
/// hand-run of their first stock draw; a Common Power falls through to Uncommon (Merchant).
#[test]
fn m31_shop_stock_shop_thresholds() {
    let mut rares = 0;
    for character in CHARACTERS {
        for seed in 0..40_u64 {
            let mut run = on_map(character, 0);
            run.state.rng = RunRng::new([seed as u8; 32], crate::combat::rng::Partition::Test, seed);
            run.state.card_rarity_offset = Some(5);
            let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
            let node = run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap();
            node.kind = MapNodeKind::Shop;
            let floor = node.floor;
            let mut rng = run.state.rng.clone();
            run.step(&GameAction::Move(target)).unwrap();
            let rows = screen(&run).choices.into_iter().filter_map(|c| match c.kind { RoomChoiceKind::Card { id, .. } => Some(id), _ => None }).collect::<Vec<_>>();
            for slot in [0_u64, 2, 4] {
                let key = [1, u64::from(floor), target.0, 0, 0, slot, 0];
                let n = rng.bounded_scoped_checked(RunRngScope::M28ShopStock, &key, 100).unwrap() as u64;
                let mut expected = hand_rarity(n, 5, SHOP);
                if slot == 4 && expected == CardRarity::Common { expected = CardRarity::Uncommon; }
                rares += u32::from(expected == CardRarity::Rare);
                assert_eq!(Catalog::standard().card(rows[slot as usize]).unwrap().rarity, expected, "{character:?} seed {seed} slot {slot} (n {n})");
            }
            assert_eq!(run.state.card_rarity_offset, Some(5), "the shop reads the offset without writing it");
        }
    }
    assert!(rares > 0, "the 9-point rare band was exercised");
}
