//! M28b room/reward integration contracts through the standard Run owners.
use super::super::*;
use super::m28_world::{drive, on_map, win};
use crate::{
    combat::{CombatAction, rng::Partition},
    content::{CardColor, CardRarity, CardType, Character, Catalog, EncounterId, PotionId, RelicId},
    run::{rooms::{self, standard::{map as standard_map, shop::{self, ShopSupply}}}, NodeId, RoomScenario},
};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_be_bytes());
    RunRng::new(root, Partition::Test, seed)
}

fn view(run: &Run) -> RoomView { rooms::public(run).unwrap() }

fn first_move(run: &Run) -> NodeId {
    run.decision().unwrap().candidates.iter().find_map(|candidate| match &candidate.action {
        GameAction::Move(node) => Some(*node), _ => None,
    }).expect("legal map move")
}

fn enter_as(run: &mut Run, kind: MapNodeKind) {
    let node = first_move(run);
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry| entry.id == node).unwrap().kind = kind;
    run.step(&GameAction::Move(node)).unwrap();
}

fn choose_kind(run: &mut Run, predicate: impl Fn(&RoomChoiceKind) -> bool) {
    let id = view(run).choices.into_iter().find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind))
        .unwrap_or_else(|| panic!("offered room choice not found: {:?}", view(run))).id;
    run.step(&GameAction::Choose(id)).unwrap();
}

fn gold_rows(run: &Run) -> Vec<u32> {
    view(run).choices.into_iter().filter_map(|row| match row.kind {
        RoomChoiceKind::Gold(amount) => Some(amount), _ => None,
    }).collect()
}

fn reward_cards(run: &mut Run) -> Vec<(crate::content::CardId, u32)> {
    choose_kind(run, |kind| *kind == RoomChoiceKind::CardReward);
    assert_eq!(view(run).kind, RoomViewKind::CardReward);
    view(run).choices.into_iter().filter_map(|row| match row.kind {
        RoomChoiceKind::Card { id, upgrades } => Some((id, upgrades)), _ => None,
    }).collect()
}

fn set_survivable(run: &mut Run) { run.state.max_hp = 999; run.state.hp = 999; }

fn refresh_relic_pools(run: &mut Run) {
    run.state.relic_pools = Some(crate::items::relic_pools::build(&run.state, &[]).unwrap());
}

fn standard_profile(neow_eligible: bool) -> CreationProfile {
    CreationProfile { neow_eligible, ..CreationProfile::experienced() }
}

#[test]
fn m28_gold_reward_ranges() {
    // docs/research/2026-10-19-act1-content-economy-potions.md:125-132; planning design §6.1.
    // Reachable node kinds are constructed to sample each reward owner.
    for seed in 0..24_u8 {
        for (kind, range) in [
            (MapNodeKind::Monster, 10..=20),
            (MapNodeKind::Elite, 25..=35),
            (MapNodeKind::Boss, 95..=105),
        ] {
            let mut base_run = on_map(Character::Ironclad, seed);
            set_survivable(&mut base_run);
            enter_as(&mut base_run, kind);
            win(&mut base_run);
            let base = gold_rows(&base_run);
            assert_eq!(base.len(), 1, "{kind:?}, seed {seed}: ordinary non-theft row");
            assert!(range.contains(&base[0]), "{kind:?}, seed {seed}: {} outside range", base[0]);

            let mut idol_run = on_map(Character::Ironclad, seed);
            set_survivable(&mut idol_run);
            idol_run.state.relics.push(RelicId::GoldenIdol);
            enter_as(&mut idol_run, kind);
            win(&mut idol_run);
            let with_idol = gold_rows(&idol_run);
            assert_eq!(with_idol.len(), 1, "{kind:?}, seed {seed}");
            let allowed = range.clone().map(|amount| amount + (amount as f32 * 0.25_f32).round() as u32)
                .collect::<std::collections::BTreeSet<_>>();
            assert!(allowed.contains(&with_idol[0]), "{kind:?}, seed {seed}: Golden Idol row {} not in {allowed:?}", with_idol[0]);
        }
    }

    // A real Looter combat earns a separate stolen-gold row, without merging it into ordinary gold.
    let mut thief = on_map(Character::Ironclad, 0xD5);
    set_survivable(&mut thief);
    thief.state.gold = 99;
    let node = first_move(&thief);
    thief.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry| entry.id == node).unwrap().kind = MapNodeKind::Monster;
    crate::run::world::enter_standard(&mut thief.state, node).unwrap();
    thief.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry| entry.id == node).unwrap().encounter = Some(EncounterId::Looter);
    thief.enter_owned(EncounterId::Looter, CombatOrigin::MapRoom { node, kind: MapNodeKind::Monster }).unwrap();
    for _ in 0..6 {
        if thief.state.combat.as_ref().unwrap().current_combat_gold().unwrap() < 99 { break; }
        thief.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
    }
    assert!(thief.state.combat.as_ref().unwrap().current_combat_gold().unwrap() < 99, "Looter stole gold before its escape");
    win(&mut thief);
    let stolen_rows = gold_rows(&thief);
    assert_eq!(stolen_rows.len(), 2, "ordinary and STOLEN_GOLD remain separate rows: {stolen_rows:?}");
    assert!((10..=20).contains(&stolen_rows[0]));
    assert_eq!(stolen_rows[1], 15, "the A0 Looter receipt is its actual stolen amount");
}

#[test]
fn m28_potion_reward_calls_appearance() {
    // docs/research/2026-10-19-act1-content-economy-potions.md:332-343; planning design §6.1.
    // Constructed route/relic fixtures; the four-row pre-check is intentionally not synthesized here.
    for character in CHARACTERS {
        let mut saw_offer = false;
        let mut saw_absence = false;
        for seed in 0..64_u8 {
            let mut run = on_map(character, seed);
            set_survivable(&mut run);
            enter_as(&mut run, MapNodeKind::Monster);
            let before = run.state.potion_offset;
            win(&mut run);
            let has_potion = view(&run).choices.iter().any(|row| matches!(row.kind, RoomChoiceKind::Potion(_)));
            let expected = if has_potion { before - 10 } else { before + 10 };
            assert_eq!(run.state.potion_offset, expected, "{character:?}, seed {seed}, has_potion={has_potion}");
            saw_offer |= has_potion;
            saw_absence |= !has_potion;
        }
        assert!(saw_offer && saw_absence, "{character:?}: sample includes success and failure");

        let mut statue = on_map(character, 0xB1);
        set_survivable(&mut statue);
        statue.state.relics.push(RelicId::WhiteBeastStatue);
        enter_as(&mut statue, MapNodeKind::Monster);
        win(&mut statue);
        assert_eq!(view(&statue).choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).count(), 1,
            "{character:?}: Statue forces a potion row when fewer than four rows precede it");
        assert_eq!(statue.state.potion_offset, -10, "{character:?}: Statue success advances the offset");
    }
    // A standard Monster reward cannot be built with four preexisting room rows using the current
    // source seam; the four-row suppression clause is covered by the M27 constructed appearance tests.
}

#[test]
fn m28_card_reward_exclusions() {
    // docs/research/2026-10-19-act1-content-economy-potions.md:103-110, :132; planning design §6.1.
    // Constructed room nodes and Prayer Wheel inventory; run all characters where card pools affect rows.
    for character in CHARACTERS {
        let mut treasure = on_map(character, 0x21);
        enter_as(&mut treasure, MapNodeKind::Treasure);
        assert!(!view(&treasure).choices.iter().any(|row| row.kind == RoomChoiceKind::CardReward));
        choose_kind(&mut treasure, |kind| *kind == RoomChoiceKind::OpenChest);
        assert!(!view(&treasure).choices.iter().any(|row| row.kind == RoomChoiceKind::CardReward));

        let mut rest = on_map(character, 0x22);
        enter_as(&mut rest, MapNodeKind::Rest);
        assert_eq!(view(&rest).kind, RoomViewKind::Campfire);
        assert!(!view(&rest).choices.iter().any(|row| row.kind == RoomChoiceKind::CardReward));

        let mut boss = on_map(character, 0x23);
        set_survivable(&mut boss);
        enter_as(&mut boss, MapNodeKind::Boss);
        win(&mut boss);
        let cards = reward_cards(&mut boss);
        assert_eq!(cards.len(), 3, "{character:?} boss offer length");
        assert!(cards.iter().all(|(id, _)| Catalog::standard().card(*id).unwrap().rarity == CardRarity::Rare), "{character:?}");

        let mut monster = on_map(character, 0x24);
        monster.state.relics.push(RelicId::PrayerWheel);
        refresh_relic_pools(&mut monster);
        enter_as(&mut monster, MapNodeKind::Monster);
        win(&mut monster);
        assert_eq!(view(&monster).choices.iter().filter(|row| row.kind == RoomChoiceKind::CardReward).count(), 2, "{character:?} ordinary MonsterRoom");

        let mut elite = on_map(character, 0x25);
        elite.state.relics.push(RelicId::PrayerWheel);
        refresh_relic_pools(&mut elite);
        set_survivable(&mut elite);
        enter_as(&mut elite, MapNodeKind::Elite);
        win(&mut elite);
        assert_eq!(view(&elite).choices.iter().filter(|row| row.kind == RoomChoiceKind::CardReward).count(), 1, "{character:?} Elite");
    }
}

#[test]
fn m28_smoke_bomb_potion_roll_and_offset() {
    // docs/research/2026-10-19-act1-content-economy-potions.md:134, :332-343. Constructed potion inventory on an ordinary monster.
    let mut run = on_map(Character::Ironclad, 0x26);
    set_survivable(&mut run);
    run.state.potions[0] = Some(PotionId::SmokeBomb);
    enter_as(&mut run, MapNodeKind::Monster);
    let card_offset = run.state.card_rarity_offset;
    let action = run.decision().unwrap().candidates.into_iter().find_map(|candidate| match candidate.action {
        GameAction::Combat(CombatAction::UsePotion { slot: 0, .. }) => Some(candidate.action), _ => None,
    }).expect("Smoke Bomb potion action in combat candidates");
    run.step(&action).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Rewards);
    assert_eq!(view(&run).choices.iter().map(|row| row.kind.clone()).collect::<Vec<_>>(), [RoomChoiceKind::Proceed]);
    assert!(matches!(run.state.potion_offset, -10 | 10), "escape applies exactly one appearance offset step");
    assert_ne!(run.state.potion_offset, 0, "escape does not leave the offset unchanged or reset it");
    assert_eq!(run.state.card_rarity_offset, card_offset, "escaped reward creates no card setup");
}

#[test]
fn m28_campfire_option_order() {
    // Planning design §6.6; docs/research/2026-10-19-act1-content-economy-potions.md:241-256.
    // Constructed relic/key fixtures; standard options preserve owned M27 option order.
    for character in CHARACTERS {
        let mut run = on_map(character, 0x31);
        run.state.relics.extend([RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel]);
        enter_as(&mut run, MapNodeKind::Rest);
        let choices = view(&run).choices.iter().map(|row| row.kind.clone()).collect::<Vec<_>>();
        assert!(matches!(choices.as_slice(), [RoomChoiceKind::Rest { .. }, RoomChoiceKind::Smith, RoomChoiceKind::Lift,
            RoomChoiceKind::Toke, RoomChoiceKind::Dig, RoomChoiceKind::Recall]), "{character:?}: {choices:?}");

        run.state.keys[0] = true;
        let mut ruby = on_map(character, 0x32);
        ruby.state.relics.extend([RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel]);
        ruby.state.keys[0] = true;
        enter_as(&mut ruby, MapNodeKind::Rest);
        assert!(!view(&ruby).choices.iter().any(|row| row.kind == RoomChoiceKind::Recall), "{character:?}: Ruby suppresses Recall");

        let mut dripper = on_map(character, 0x33);
        dripper.state.relics.push(RelicId::CoffeeDripper);
        enter_as(&mut dripper, MapNodeKind::Rest);
        assert!(!view(&dripper).choices.iter().any(|row| matches!(row.kind, RoomChoiceKind::Rest { .. })), "{character:?}: Coffee Dripper");

        let mut hammer = on_map(character, 0x34);
        hammer.state.relics.push(RelicId::FusionHammer);
        enter_as(&mut hammer, MapNodeKind::Rest);
        assert!(!view(&hammer).choices.iter().any(|row| row.kind == RoomChoiceKind::Smith), "{character:?}: Fusion Hammer");
    }
}

#[test]
fn m28_nested_selection_returns_neow_shop_smith() {
    // Planning design §6.6; docs/research/2026-10-19-act1-content-economy-potions.md:241-256.
    // Constructed reachable campfire; Cancel returns to the same screen and Smith returns to Map after upgrade.
    let mut run = on_map(Character::Ironclad, 0x35);
    enter_as(&mut run, MapNodeKind::Rest);
    let campfire = view(&run);
    let deck = run.state.deck.clone();
    choose_kind(&mut run, |kind| *kind == RoomChoiceKind::Smith);
    assert!(run.state.pending.is_some());
    run.step(&GameAction::Cancel).unwrap();
    assert_eq!(view(&run), campfire);
    assert_eq!(run.state.deck, deck);

    choose_kind(&mut run, |kind| *kind == RoomChoiceKind::Smith);
    let selection = run.decision().unwrap().candidates.into_iter().find_map(|candidate| match candidate.action {
        GameAction::Select(id) => Some(id), _ => None,
    }).expect("Smith selection candidate");
    let before = run.state.deck.clone();
    run.step(&GameAction::Select(selection)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!(run.state.deck.iter().filter(|card| card.upgrades == 1).count(),
        before.iter().filter(|card| card.upgrades == 1).count() + 1);
}

#[test]
fn m28_question_room_overrides() {
    // docs/research/2026-10-19-act1-content-event-neow.md:190-199; planning design §6.2.
    // Constructed legal nodes and ramps.
    let mut tiny = on_map(Character::Ironclad, 0x41);
    let node = first_move(&tiny);
    tiny.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry| entry.id == node).unwrap().kind = MapNodeKind::Question;
    tiny.state.relics.push(RelicId::TinyChest);
    tiny.state.relic_counters.insert(RelicId::TinyChest, 3);
    tiny.state.map.as_mut().unwrap().question_ramps = [0; 3];
    tiny.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&tiny).kind, RoomViewKind::Treasure, "the fourth Tiny Chest roll forces Treasure");
    assert_eq!(tiny.state.relic_counters.get(&RelicId::TinyChest), Some(&0));
    let ramps = tiny.state.map.as_ref().unwrap().question_ramps;
    assert_eq!(ramps, [0.1_f32.to_bits(), 0.03_f32.to_bits(), 0x3CA3D70A], "forced Treasure resets its ramp and advances the others");

    let mut juzu = on_map(Character::Ironclad, 0x42);
    let node = first_move(&juzu);
    juzu.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry| entry.id == node).unwrap().kind = MapNodeKind::Question;
    juzu.state.relics.push(RelicId::JuzuBracelet);
    juzu.state.map.as_mut().unwrap().question_ramps = [1.0_f32.to_bits(), 0, 0];
    let before = juzu.clone();
    assert_eq!(juzu.step(&GameAction::Move(node)), Err(RunError::Unsupported(standard_map::EVENT_ROOMS_INTERIM)));
    assert!(juzu == before, "Juzu converts Monster to the interim Event without committing a combat");

    let mut after_shop = on_map(Character::Ironclad, 0x43);
    let node = first_move(&after_shop);
    after_shop.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry| entry.id == node).unwrap().kind = MapNodeKind::Question;
    after_shop.state.room = "Shop";
    after_shop.state.map.as_mut().unwrap().question_ramps = [0, 1.0_f32.to_bits(), 0];
    let before = after_shop.clone();
    assert_eq!(after_shop.step(&GameAction::Move(node)), Err(RunError::Unsupported(standard_map::EVENT_ROOMS_INTERIM)));
    assert!(after_shop == before, "a post-shop question cannot resolve to Shop");
}

#[test]
fn m28_rejected_shop_and_chest_choices_are_atomic() {
    // docs/research/2026-10-19-act1-content-economy-potions.md:161, :173-184, :190-198, :275-291; design §§6.3–6.4.
    // Constructed Treasure/Shop choices and a zero-gold shop state.
    let mut chest = on_map(Character::Ironclad, 0x51);
    enter_as(&mut chest, MapNodeKind::Treasure);
    choose_kind(&mut chest, |kind| *kind == RoomChoiceKind::OpenChest);
    let relic = view(&chest).choices.iter().find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap().id;
    let sapphire = view(&chest).choices.iter().find(|row| row.kind == RoomChoiceKind::Sapphire).unwrap().id;
    chest.step(&GameAction::Choose(relic)).unwrap();
    let before = chest.clone();
    assert_eq!(chest.step(&GameAction::Choose(sapphire)), Err(RunError::InvalidAction));
    assert!(chest == before, "rejected linked Sapphire leaves the whole Run unchanged");

    let mut poor = on_map(Character::Ironclad, 0x52);
    enter_as(&mut poor, MapNodeKind::Shop);
    poor.state.gold = 0;
    let card = view(&poor).choices.iter().find(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).unwrap().id;
    let before = poor.clone();
    assert_eq!(poor.step(&GameAction::Choose(card)), Err(RunError::InvalidAction));
    assert!(poor == before, "an unaffordable shop row is atomic");

    let mut removal = on_map(Character::Ironclad, 0x53);
    enter_as(&mut removal, MapNodeKind::Shop);
    let purge = view(&removal).choices.iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().id;
    removal.step(&GameAction::Choose(purge)).unwrap();
    let selection = removal.decision().unwrap().candidates.into_iter().find_map(|candidate| match candidate.action {
        GameAction::Select(id) => Some(id), _ => None,
    }).expect("purge selection candidate");
    removal.step(&GameAction::Select(selection)).unwrap();
    let before = removal.clone();
    assert_eq!(removal.step(&GameAction::Choose(purge)), Err(RunError::InvalidAction));
    assert!(removal == before, "a second removal in one shop is rejected atomically");
}

#[test]
fn m28_act4_standard_rooms() {
    // Planning design §6.1, §6.3, §6.6-§6.7; docs/research/2026-10-19-act1-content-economy-potions.md:103-110, :125-132, :155-161.
    // Constructed survivable/key/relic state; Act 4 floor 51–53 producers include the campfire and card pools.
    for character in CHARACTERS {
        let mut run = on_map(character, 0x61);
        set_survivable(&mut run);
        drive(&mut run, |current| current.state.act == 3);
        run.state.relics.extend([RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel]);
        refresh_relic_pools(&mut run);
        run.state.keys = [true; 3];
        drive(&mut run, |current| current.state.act == 4);
        drive(&mut run, |current| current.state.floor == 51 && view(current).kind == RoomViewKind::Campfire);
        let campfire = view(&run).choices.into_iter().map(|row| row.kind).collect::<Vec<_>>();
        assert!(matches!(campfire.as_slice(), [RoomChoiceKind::Rest { .. }, RoomChoiceKind::Smith, RoomChoiceKind::Lift,
            RoomChoiceKind::Toke, RoomChoiceKind::Dig]), "{character:?}: floor 51 rows {campfire:?}");

        drive(&mut run, |current| current.state.floor == 52 && view(current).kind == RoomViewKind::Shop);
        let shop_rows = view(&run).choices;
        assert_eq!(shop_rows.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).count(), 7, "{character:?}");
        assert_eq!(shop_rows.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).count(), 3, "{character:?}");
        assert_eq!(shop_rows.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).count(), 3, "{character:?}");
        assert!(shop_rows.iter().any(|row| row.kind == RoomChoiceKind::Purge));
        assert!(shop_rows.iter().any(|row| row.kind == RoomChoiceKind::Leave));

        drive(&mut run, |current| current.state.floor == 53 && view(current).kind == RoomViewKind::Rewards);
        let rewards = view(&run);
        let gold = gold_rows(&run);
        assert_eq!(gold.len(), 1, "{character:?}: no stolen-gold combat in floor 53");
        assert!((25..=35).contains(&gold[0]), "{character:?}: elite gold before bonuses");
        assert!(rewards.choices.iter().any(|row| matches!(row.kind, RoomChoiceKind::Relic(_))), "{character:?}: elite relic");
        let cards = reward_cards(&mut run);
        assert_eq!(cards.len(), 3, "{character:?}: elite card offer");
        let color = match character {
            Character::Ironclad => CardColor::Red, Character::Silent => CardColor::Green,
            Character::Defect => CardColor::Blue, Character::Watcher => CardColor::Purple,
        };
        assert!(cards.iter().all(|(id, _)| Catalog::standard().card(*id).unwrap().color == color), "{character:?}: cards belong to the class pool");
    }
}

#[test]
fn m28_frozen_profiles_never_reach_standard_owners_b() {
    // crates/spire-core/src/run/tests/m28_standard.rs:127-146; planning design §7.
    // The scenarios mirror the accepted frozen list.
    use RoomScenario::*;
    let frozen = [
        (M4Act1Prefix, Character::Ironclad), (M5Act1Captured, Character::Ironclad), (M6Act2Prefix, Character::Ironclad),
        (M7Act2Captured, Character::Ironclad), (M8Act3Prefix, Character::Ironclad), (M9IroncladAct3, Character::Ironclad),
        (M10IroncladHeart, Character::Ironclad), (M12SilentAct1, Character::Silent), (M13SilentAct2, Character::Silent),
        (M14SilentAct3, Character::Silent), (M15SilentHeart, Character::Silent), (M17DefectAct1, Character::Defect),
        (M18DefectAct2, Character::Defect), (M19DefectAct3, Character::Defect), (M20DefectHeart, Character::Defect),
        (M22WatcherAct1, Character::Watcher), (M23WatcherAct2, Character::Watcher), (M24WatcherAct3, Character::Watcher),
        (M25WatcherHeart, Character::Watcher),
    ];
    for (scenario, character) in frozen {
        let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, standard_profile(false), rng(3), scenario).unwrap();
        assert_eq!(standard_map::enter(&mut run, NodeId(u64::MAX)), Err(RunError::InvalidAction), "{scenario:?}");
        assert_eq!(run.state.card_rarity_offset, None, "{scenario:?}");
    }

    let mut captured = Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 0 },
        standard_profile(false), rng(3), M5Act1Captured).unwrap();
    let lament = view(&captured).choices.iter().find(|row| row.kind == RoomChoiceKind::NeowLament).unwrap().id;
    captured.step(&GameAction::Choose(lament)).unwrap();
    let move_action = captured.decision().unwrap().candidates.iter().find(|candidate| matches!(candidate.action, GameAction::Move(_)))
        .map(|candidate| candidate.action.clone()).expect("captured M5 map move");
    captured.step(&move_action).unwrap();
    assert_eq!(captured.state.card_rarity_offset, None, "a captured map move never creates the standard card offset");
}

#[test]
fn m28_neow_three_potions_discards_card_row() {
    // docs/research/2026-10-19-act1-content-event-neow.md:74; docs/research/2026-10-19-act1-content-economy-potions.md:103-107; planning design §6.5.
    let mut found_offset_change = false;
    let mut tested_offers = 0;
    for seed in 0..5_000_u64 {
        if tested_offers == 64 { break; }
        let character = CHARACTERS[(seed as usize) % CHARACTERS.len()];
        let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, standard_profile(true), rng(seed), RoomScenario::StandardA0).unwrap();
        let Some(offer) = run.candidates.iter().find(|candidate| candidate.neow_offer.as_ref().is_some_and(|offer|
            offer.category == 1 && offer.reward == NeowReward::ThreePotions)).map(|candidate| candidate.id) else { continue };
        assert_eq!(run.state.card_rarity_offset, Some(5));
        run.step(&GameAction::Choose(offer)).unwrap();
        let offset = run.state.card_rarity_offset.unwrap();
        assert!((2..=5).contains(&offset), "{character:?}, seed {seed}: three ordinary slots leave b in 2..=5, got {offset}");
        found_offset_change |= offset != 5;
        assert!(!view(&run).choices.iter().any(|row| row.kind == RoomChoiceKind::CardReward
            || matches!(row.kind, RoomChoiceKind::Card { .. })), "the discarded offer is not shown with potions");
        tested_offers += 1;
    }
    assert_eq!(tested_offers, 64, "sampled sixty-four category 1 ThreePotions offers");
    assert!(found_offset_change, "at least one seed shows the three-slot offset is not guaranteed to remain 5");
}

#[test]
fn m28_shop_card_rows_preview_eggs() {
    // docs/research/2026-10-19-act1-content-economy-potions.md:108, :155-161, :190-197; planning design §6.1/§6.3.
    // Constructed ShopSupply and Toxic Egg row; use the standard diagnostic shop builder.
    for character in CHARACTERS {
        let molten = shop::diagnostic_shop(character, ShopSupply {
            gold: 1_000, deck: None, relics: vec![RelicId::MoltenEgg], potions: Some(vec![None; 3]), purge_cost: 75,
        }, rng(0x701)).unwrap();
        let card_rows = view(&molten).choices.into_iter().filter_map(|row| match row.kind {
            RoomChoiceKind::Card { id, upgrades } => Some((id, upgrades)), _ => None,
        }).collect::<Vec<_>>();
        let attack_rows = card_rows.iter().filter(|(id, _)| Catalog::standard().card(*id).unwrap().card_type == CardType::Attack).collect::<Vec<_>>();
        assert!(!attack_rows.is_empty(), "{character:?}: standard shop has attack rows");
        assert!(attack_rows.iter().all(|(_, upgrades)| *upgrades == 1), "{character:?}: Molten Egg previews attack upgrades");

        let mut toxic = shop::diagnostic_shop(character, ShopSupply {
            gold: 1_000, deck: None, relics: vec![RelicId::MoltenEgg], potions: Some(vec![None; 3]), purge_cost: 75,
        }, rng(0x702)).unwrap();
        // Pin one generated relic slot to Toxic Egg so the purchase path is deterministic.
        let toxic_id = {
            let toxic_row = toxic.state.room_loop.as_mut().unwrap().screen.choices.iter_mut()
                .find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
            toxic_row.kind = RoomChoiceKind::Relic(RelicId::ToxicEgg);
            toxic_row.price = Some(1);
            toxic_row.id
        };
        toxic.step(&GameAction::Choose(toxic_id)).unwrap();
        let rows = view(&toxic).choices;
        let mut attacks = 0;
        let mut skills = 0;
        for row in rows {
            let RoomChoiceKind::Card { id, upgrades } = row.kind else { continue };
            match Catalog::standard().card(id).unwrap().card_type {
                CardType::Attack => { attacks += 1; assert_eq!(upgrades, 1, "{character:?}: Molten Egg remains previewed"); }
                CardType::Skill => { skills += 1; assert_eq!(upgrades, 1, "{character:?}: Toxic Egg re-previews shop rows"); }
                _ => assert_eq!(upgrades, 0, "{character:?}: eggs affect only matching card types"),
            }
        }
        assert!(attacks > 0 && skills > 0, "{character:?}: shop includes both typed card rows");
    }
}
