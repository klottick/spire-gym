//! M28a Neow lane tests (chunk plan §4 items 10-15).
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{CardColor, CardRarity, Character, RelicId};
use crate::content::pools::CardPool;
use std::collections::{BTreeMap, BTreeSet};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
const HP_LITERALS: [(Character, u32, u32, u32, u32); 4] = [
    (Character::Ironclad, 80, 8, 16, 24),
    (Character::Silent, 70, 7, 14, 21),
    (Character::Defect, 75, 7, 14, 21),
    (Character::Watcher, 72, 7, 14, 21),
];

fn rng(seed: u64) -> RunRng {
    let mut root = [0u8; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(root, Partition::Test, 0)
}

fn profile() -> CreationProfile {
    CreationProfile { neow_eligible: true, ..CreationProfile::experienced() }
}

fn standard(character: Character, seed: u64) -> Run {
    Run::new_room_scenario(
        RunConfig { character, ascension: 0 }, profile(), rng(seed), RoomScenario::StandardA0,
    ).unwrap()
}

fn room(run: &Run) -> RoomView {
    run.decision().unwrap().room.unwrap()
}

fn row_id(run: &Run, predicate: impl Fn(&RoomChoiceKind) -> bool) -> ChoiceId {
    room(run).choices.into_iter().find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind))
        .unwrap_or_else(|| panic!("missing Neow row in {:?}", room(run))).id
}

/// Force both the persisted candidate and visible row, as required for deterministic payload tests.
fn force_offer(run: &mut Run, category: u8, reward: NeowReward, drawback: Option<NeowDrawback>) -> ChoiceId {
    let candidate = run.candidates.iter_mut().find(|candidate| {
        candidate.neow_offer.as_ref().is_some_and(|offer| offer.category == category)
    }).expect("category candidate");
    let offer = NeowOffer { category, reward, drawback };
    candidate.neow_offer = Some(offer.clone());
    let id = candidate.id;
    let choice = run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|row| row.id == id).unwrap();
    choice.kind = RoomChoiceKind::NeowBlessing { category, reward, drawback };
    id
}

fn choose_offer(run: &mut Run, category: u8, reward: NeowReward, drawback: Option<NeowDrawback>) {
    let id = force_offer(run, category, reward, drawback);
    run.step(&GameAction::Choose(id)).unwrap_or_else(|error| panic!("choose {category}/{reward:?}: {error:?}"));
}

fn cards(run: &Run) -> Vec<crate::run::CardInstance> { run.state.deck.clone() }

fn selected_action(run: &Run, index: usize) -> GameAction {
    run.decision().unwrap().candidates.into_iter().filter(|candidate| matches!(candidate.action, GameAction::Select(_)))
        .nth(index).unwrap().action
}

fn card_metadata(id: crate::content::CardId) -> &'static crate::content::CardDefinition {
    crate::content::Catalog::standard().card(id).unwrap()
}

fn check_six_sigma(count: u32, n: u32, p: f64, label: &str) {
    let expected = n as f64 * p;
    let sigma = (n as f64 * p * (1.0 - p)).sqrt();
    assert!((count as f64 - expected).abs() <= 6.0 * sigma + 2.0,
        "{label}: count={count}, expected={expected:.2}, sigma={sigma:.2}");
}

#[test]
fn m28_neow_full_legal_pairs() {
    use NeowReward as R;
    use NeowDrawback as D;
    for character in CHARACTERS {
        let mut category0 = BTreeMap::<u8, u32>::new();
        let mut category1 = BTreeMap::<u8, u32>::new();
        let mut drawbacks = BTreeMap::<u8, u32>::new();
        let mut pairs = BTreeMap::<(u8, u8), u32>::new();
        for seed in 0..2_000 {
            let run = standard(character, seed);
            let offers = run.candidates.iter().filter_map(|candidate| candidate.neow_offer.as_ref()).collect::<Vec<_>>();
            assert_eq!(offers.iter().map(|offer| offer.category).collect::<Vec<_>>(), [0, 1, 2, 3]);
            let first = offers[0];
            assert!(matches!(first.reward, R::ThreeCards | R::RandomRare | R::RemoveOne | R::UpgradeOne | R::TransformOne | R::ThreeUncommonColorless));
            *category0.entry(first.reward as u8).or_default() += 1;
            let second = offers[1];
            assert!(matches!(second.reward, R::ThreePotions | R::CommonRelic | R::MaxHp | R::Lament | R::Gold));
            *category1.entry(second.reward as u8).or_default() += 1;
            let third = offers[2];
            let drawback = third.drawback.unwrap();
            *drawbacks.entry(drawback as u8).or_default() += 1;
            let legal = match (drawback, third.reward) {
                (D::MaxHpLoss, R::MaxHpTwice) | (D::NoGold, R::Gold250) | (D::RandomCurse, R::RemoveTwo) => false,
                (D::CurrentHpLoss, R::RareColorless | R::RareRelic | R::ThreeRareCards | R::TransformTwo | R::RemoveTwo | R::Gold250 | R::MaxHpTwice) => true,
                (D::MaxHpLoss | D::NoGold | D::RandomCurse,
                    R::RareColorless | R::RareRelic | R::ThreeRareCards | R::TransformTwo | R::RemoveTwo | R::Gold250 | R::MaxHpTwice) => true,
                pair => panic!("illegal category-2 pair: {pair:?}"),
            };
            assert!(legal);
            *pairs.entry((drawback as u8, third.reward as u8)).or_default() += 1;
            assert_eq!(offers[3].reward, R::BossSwap);
            assert_eq!(offers[3].drawback, None);
        }
        for reward in [R::ThreeCards, R::RandomRare, R::RemoveOne, R::UpgradeOne, R::TransformOne, R::ThreeUncommonColorless] {
            check_six_sigma(category0.get(&(reward as u8)).copied().unwrap_or(0), 2_000, 1.0 / 6.0, &format!("{character:?} category 0 {reward:?}"));
        }
        for reward in [R::ThreePotions, R::CommonRelic, R::MaxHp, R::Lament, R::Gold] {
            check_six_sigma(category1.get(&(reward as u8)).copied().unwrap_or(0), 2_000, 1.0 / 5.0, &format!("{character:?} category 1 {reward:?}"));
        }
        for drawback in [D::MaxHpLoss, D::NoGold, D::RandomCurse, D::CurrentHpLoss] {
            check_six_sigma(drawbacks.get(&(drawback as u8)).copied().unwrap_or(0), 2_000, 1.0 / 4.0, &format!("{character:?} {drawback:?}"));
            let rewards: &[R] = match drawback {
                D::MaxHpLoss => &[R::RareColorless, R::RareRelic, R::ThreeRareCards, R::TransformTwo, R::RemoveTwo, R::Gold250],
                D::NoGold => &[R::RareColorless, R::RareRelic, R::ThreeRareCards, R::TransformTwo, R::RemoveTwo, R::MaxHpTwice],
                D::RandomCurse => &[R::RareColorless, R::RareRelic, R::ThreeRareCards, R::TransformTwo, R::Gold250, R::MaxHpTwice],
                D::CurrentHpLoss => &[R::RareColorless, R::RareRelic, R::ThreeRareCards, R::TransformTwo, R::RemoveTwo, R::Gold250, R::MaxHpTwice],
            };
            for reward in rewards {
                let p = if drawback == D::CurrentHpLoss { 1.0 / 28.0 } else { 1.0 / 24.0 };
                check_six_sigma(pairs.get(&(drawback as u8, *reward as u8)).copied().unwrap_or(0), 2_000, p,
                    &format!("{character:?} {drawback:?}/{reward:?}"));
            }
        }

        // Binding A0 literals from character-branches-a.md; these are intentionally not derived
        // by the implementation under test.
        let (_, max_hp, b, twice_b, hp_loss) = HP_LITERALS.iter().find(|row| row.0 == character).copied().unwrap();
        let mut run = standard(character, 90_000);
        choose_offer(&mut run, 1, R::MaxHp, None);
        assert_eq!((run.state.hp, run.state.max_hp), (max_hp + b, max_hp + b), "{character:?} +b heals exactly once");
        let mut run = standard(character, 90_001);
        choose_offer(&mut run, 2, R::MaxHpTwice, Some(D::NoGold));
        assert_eq!((run.state.hp, run.state.max_hp, run.state.gold), (max_hp + twice_b, max_hp + twice_b, 0), "{character:?} +2b/no gold");
        let mut run = standard(character, 90_002);
        choose_offer(&mut run, 2, R::RareColorless, Some(D::MaxHpLoss));
        assert_eq!((run.state.hp, run.state.max_hp), (max_hp - b, max_hp - b), "{character:?} max-HP loss");
        let leave = row_id(&run, |kind| *kind == RoomChoiceKind::Leave);
        run.step(&GameAction::Choose(leave)).unwrap();
        let mut run = standard(character, 90_003);
        choose_offer(&mut run, 2, R::Gold250, Some(D::CurrentHpLoss));
        assert_eq!((run.state.hp, run.state.max_hp, run.state.gold), (max_hp - hp_loss, max_hp, 349), "{character:?} HP loss / +250 gold");
        let mut run = standard(character, 90_004);
        choose_offer(&mut run, 1, R::Gold, None);
        assert_eq!(run.state.gold, 199, "{character:?} +100 gold");
    }
}

#[test]
fn m28_neow_three_cards_uncommon_mass() {
    use NeowReward as R;

    let mut uncommon = 0_u32;
    let mut total = 0_u32;
    const RUNS_PER_CHARACTER: u64 = 1_500;
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        for seed in 0..RUNS_PER_CHARACTER {
            let mut run = standard(character, character_index as u64 * 100_000 + seed);
            choose_offer(&mut run, 0, R::ThreeCards, None);
            let offered = room(&run).choices.into_iter().filter_map(|row| match row.kind {
                RoomChoiceKind::Card { id, .. } => Some(id),
                _ => None,
            }).collect::<Vec<_>>();
            assert_eq!(offered.len(), 3, "{character:?} seed {seed}");
            for id in offered {
                total += 1;
                match card_metadata(id).rarity {
                    CardRarity::Uncommon => uncommon += 1,
                    CardRarity::Common => {},
                    other => panic!("ThreeCards offered unexpected {other:?}: {id:?}"),
                }
            }
        }
    }

    // c-a.md N5: each of the three slots is Uncommon with mass 5,536,482/2^24; Rare is impossible.
    let p_uncommon = 5_536_482.0_f64 / 16_777_216.0_f64;
    assert_eq!(total, (RUNS_PER_CHARACTER * CHARACTERS.len() as u64 * 3) as u32);
    check_six_sigma(uncommon, total, p_uncommon, "Neow ThreeCards uncommon slot mass across four characters");
}

#[test]
fn m28_neow_children_skippable_or_committed() {
    use NeowReward as R;
    use NeowDrawback as D;
    for (character, _, _, _, _) in HP_LITERALS {
        let class_rare = crate::content::pools::cards(CardPool::Ordinary { character, rarity: Some(CardRarity::Rare), card_type: None }).unwrap();
        let uncommon_colorless = crate::content::pools::cards(CardPool::Colourless { rarity: Some(CardRarity::Uncommon) }).unwrap();
        let rare_colorless = crate::content::pools::cards(CardPool::Colourless { rarity: Some(CardRarity::Rare) }).unwrap();
        for (category, reward, drawback, expected_pool, expected_rarity, expected_color) in [
            (0, R::ThreeCards, None, None, None, None),
            (2, R::ThreeRareCards, Some(D::NoGold), Some(class_rare.clone()), Some(CardRarity::Rare), None),
            (0, R::ThreeUncommonColorless, None, Some(uncommon_colorless.clone()), Some(CardRarity::Uncommon), Some(CardColor::Colorless)),
            (2, R::RareColorless, Some(D::MaxHpLoss), Some(rare_colorless.clone()), Some(CardRarity::Rare), Some(CardColor::Colorless)),
        ] {
            let mut run = standard(character, 91_000 + reward as u64);
            let before = cards(&run);
            choose_offer(&mut run, category, reward, drawback);
            assert_eq!(room(&run).kind, RoomViewKind::CardReward, "{character:?} {reward:?}");
            let view = room(&run);
            let offered = view.choices.iter().filter_map(|row| if let RoomChoiceKind::Card { id, upgrades } = row.kind {
                assert_eq!(upgrades, 0); Some(id)
            } else { None }).collect::<Vec<_>>();
            assert_eq!(offered.len(), 3, "{character:?} {reward:?}");
            assert_eq!(offered.iter().copied().collect::<BTreeSet<_>>().len(), 3, "distinct card IDs");
            assert!(view.choices.iter().any(|row| row.kind == RoomChoiceKind::Leave), "Leave row");
            if reward == R::ThreeCards {
                assert!(offered.iter().all(|id| matches!(card_metadata(*id).rarity, CardRarity::Common | CardRarity::Uncommon)));
            } else {
                let pool = expected_pool.as_ref().unwrap();
                assert!(offered.iter().all(|id| pool.contains(id)), "{character:?} {reward:?} pool");
                assert!(offered.iter().all(|id| card_metadata(*id).rarity == expected_rarity.unwrap()));
                if let Some(color) = expected_color { assert!(offered.iter().all(|id| card_metadata(*id).color == color)); }
            }
            let leave = row_id(&run, |kind| *kind == RoomChoiceKind::Leave);
            run.step(&GameAction::Choose(leave)).unwrap();
            assert_eq!(run.state.deck, before, "skipping {reward:?} obtains nothing");
            assert_eq!(room(&run).kind, RoomViewKind::Map);
        }

        let mut rare = standard(character, 91_100);
        let deck_len = rare.state.deck.len();
        choose_offer(&mut rare, 0, R::RandomRare, None);
        assert_eq!(rare.state.deck.len(), deck_len + 1, "RandomRare is obtained immediately");
        assert_eq!(card_metadata(rare.state.deck.last().unwrap().id).rarity, CardRarity::Rare);

        let mut claimed = standard(character, 91_101);
        let deck_len = claimed.state.deck.len();
        choose_offer(&mut claimed, 0, R::ThreeCards, None);
        let first_card = row_id(&claimed, |kind| matches!(kind, RoomChoiceKind::Card { .. }));
        claimed.step(&GameAction::Choose(first_card)).unwrap();
        assert_eq!(claimed.state.deck.len(), deck_len + 1, "claiming a card obtains it");
        assert_eq!(room(&claimed).kind, RoomViewKind::Map);

        let mut remove = standard(character, 91_200);
        let deck_len = remove.state.deck.len();
        choose_offer(&mut remove, 2, R::RemoveTwo, Some(D::MaxHpLoss));
        assert_eq!(remove.state.pending.as_ref().unwrap().selection.required, 2);
        assert!(!remove.decision().unwrap().candidates.iter().any(|candidate| candidate.action == GameAction::Cancel));
        remove.step(&selected_action(&remove, 0)).unwrap();
        assert_eq!(remove.decision().unwrap().selection.unwrap().selected_count, 1);
        assert!(!remove.decision().unwrap().candidates.iter().any(|candidate| candidate.action == GameAction::Cancel));
        remove.step(&selected_action(&remove, 0)).unwrap();
        assert_eq!(remove.state.deck.len(), deck_len - 2);
        assert_eq!(remove.state.last_selection.as_ref().unwrap().len(), 2);
        assert_eq!(room(&remove).kind, RoomViewKind::Map);

        let mut upgrade = standard(character, 91_300);
        choose_offer(&mut upgrade, 0, R::UpgradeOne, None);
        let captured = upgrade.state.pending.as_ref().unwrap().selection.candidates[0].card.clone();
        upgrade.step(&selected_action(&upgrade, 0)).unwrap();
        assert_eq!(upgrade.state.deck.iter().find(|card| card.instance == captured.instance).unwrap().upgrades, captured.upgrades + 1);
        assert_eq!(room(&upgrade).kind, RoomViewKind::Map);

        let mut transform = standard(character, 91_400);
        choose_offer(&mut transform, 0, R::TransformOne, None);
        let captured = transform.state.pending.as_ref().unwrap().selection.candidates[0].card.clone();
        transform.step(&selected_action(&transform, 0)).unwrap();
        assert!(!transform.state.deck.iter().any(|card| card.instance == captured.instance), "transform removes input");
        assert_eq!(room(&transform).kind, RoomViewKind::Map);

        let mut potions = standard(character, 91_500);
        choose_offer(&mut potions, 1, R::ThreePotions, None);
        let view = room(&potions);
        assert_eq!(view.kind, RoomViewKind::Rewards);
        let potion_rows = view.choices.iter().filter_map(|row| match row.kind {
            RoomChoiceKind::Potion(potion) => Some((row.id, potion)), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(potion_rows.len(), 3);
        assert!(view.choices.iter().any(|row| row.kind == RoomChoiceKind::Leave));
        let pool = crate::items::potion_generation::candidates(character, false).unwrap();
        assert!(potion_rows.iter().all(|(_, potion)| pool.contains(potion)), "{character:?} legal potion pool");
        let potion_before = potions.state.potions.clone();
        potions.step(&GameAction::Choose(potion_rows[0].0)).unwrap();
        let slot = potion_before.iter().position(Option::is_none).unwrap();
        assert_eq!(potions.state.potions[slot], Some(potion_rows[0].1));
        assert_eq!(room(&potions).kind, RoomViewKind::Rewards, "potion claims stay on screen");
        assert_eq!(room(&potions).choices.iter().find(|row| row.id == potion_rows[0].0).unwrap().status, RoomChoiceStatus::Claimed);
        let leave = row_id(&potions, |kind| *kind == RoomChoiceKind::Leave);
        potions.step(&GameAction::Choose(leave)).unwrap();
        assert_eq!(room(&potions).kind, RoomViewKind::Map);

        let mut full = standard(character, 91_501);
        choose_offer(&mut full, 1, R::ThreePotions, None);
        let potion_id = row_id(&full, |kind| matches!(kind, RoomChoiceKind::Potion(_)));
        full.state.potions = vec![Some(crate::content::PotionId::BlockPotion); 3];
        let before = full.clone();
        assert_eq!(full.step(&GameAction::Choose(potion_id)), Err(RunError::InvalidAction));
        assert_eq!(full, before, "full belt rejects atomically");
    }
}

#[test]
fn m28_neow_boss_swap_equip_offer() {
    for (character, _, _, _, _) in HP_LITERALS {
        let starter = crate::content::Catalog::standard().starter_relic(character).unwrap();
        let mut basic = standard(character, 92_000);
        basic.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(vec![], vec![], vec![], vec![], vec![RelicId::SneckoEye]));
        let offer_id = force_offer(&mut basic, 3, NeowReward::BossSwap, None);
        basic.step(&GameAction::Choose(offer_id)).unwrap();
        assert!(!basic.state.relics.contains(&starter), "{character:?} starter removed");
        assert!(basic.state.relics.contains(&RelicId::SneckoEye), "{character:?} one Boss relic obtained");
        assert_eq!(room(&basic).kind, RoomViewKind::Map);

        let mut astrolabe = standard(character, 92_001);
        let input_instances = astrolabe.state.deck.iter().map(|card| card.instance).collect::<BTreeSet<_>>();
        astrolabe.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(vec![], vec![], vec![], vec![], vec![RelicId::Astrolabe]));
        let astrolabe_id = force_offer(&mut astrolabe, 3, NeowReward::BossSwap, None);
        astrolabe.step(&GameAction::Choose(astrolabe_id)).unwrap();
        assert!(astrolabe.state.map.is_some(), "Map generated before Astrolabe continuation");
        assert_eq!(room(&astrolabe).kind, RoomViewKind::Map);
        assert_eq!(astrolabe.state.pending.as_ref().unwrap().selection.required, 3);
        let astrolabe_inputs = astrolabe.state.pending.as_ref().unwrap().selection.candidates[..3].iter()
            .map(|candidate| candidate.card.instance).collect::<Vec<_>>();
        for _ in 0..3 { astrolabe.step(&selected_action(&astrolabe, 0)).unwrap(); }
        assert!(astrolabe_inputs.iter().all(|instance| !astrolabe.state.deck.iter().any(|card| card.instance == *instance)));
        assert!(input_instances.len() > astrolabe_inputs.len());
        assert_eq!(room(&astrolabe).kind, RoomViewKind::Map, "Astrolabe completes over the map");

        let mut bell = standard(character, 92_002);
        bell.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
            vec![RelicId::Anchor], vec![RelicId::BagOfPreparation], vec![RelicId::DeadBranch], vec![], vec![RelicId::CallingBell],
        ));
        choose_offer(&mut bell, 3, NeowReward::BossSwap, None);
        assert_eq!(room(&bell).kind, RoomViewKind::Rewards, "Calling Bell offers open over the map");
        assert!(bell.state.map.is_some());
        assert!(bell.state.deck.iter().any(|card| card.id == crate::content::CardId::CurseOfTheBell));
    }
}

#[test]
fn m28_nested_selection_returns_neow() {
    for character in CHARACTERS {
        let mut run = standard(character, 93_000);
        let before = run.state.deck.clone();
        choose_offer(&mut run, 2, NeowReward::TransformTwo, Some(NeowDrawback::NoGold));
        assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 2);
        let selected = run.state.pending.as_ref().unwrap().selection.candidates[..2].iter().map(|candidate| candidate.card.clone()).collect::<Vec<_>>();
        run.step(&selected_action(&run, 0)).unwrap();
        assert_eq!(run.decision().unwrap().selection.unwrap().selected_count, 1);
        run.step(&selected_action(&run, 0)).unwrap();
        assert_eq!(run.state.deck.len(), before.len(), "two transforms replace two cards");
        assert!(selected.iter().all(|old| !run.state.deck.iter().any(|card| card.instance == old.instance)), "inputs removed");
        assert!(run.state.deck.iter().all(|card| card.instance != 0));
        assert_eq!(room(&run).kind, RoomViewKind::Map);
        assert_eq!(run.state.room, "Map");
    }
}

#[test]
fn m28_neow_random_curse_after_reward() {
    for character in CHARACTERS {
        let mut run = standard(character, 94_000);
        choose_offer(&mut run, 2, NeowReward::TransformTwo, Some(NeowDrawback::RandomCurse));
        let curses = run.state.deck.iter().filter(|card| card_metadata(card.id).rarity == CardRarity::Curse).count();
        assert_eq!(curses, 1, "one deferred curse after the selector opened");
        let pending = run.state.pending.as_ref().unwrap();
        assert_eq!(pending.selection.required, 2);
        assert!(pending.selection.candidates.iter().all(|candidate| card_metadata(candidate.card.id).rarity != CardRarity::Curse),
            "selector captured before curse and contains no curse candidate");
    }
}
