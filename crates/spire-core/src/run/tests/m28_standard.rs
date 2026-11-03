//! M28a seams (chunk plan §4 M28a items 1–7): standard A0 creation, M28 RNG scopes, the max-HP
//! loss owner, starter tags (G1) and frozen isolation. Literals from design §4, §6.5, §7, §8.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{CardId, CardTag, Character};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn profile(full: bool) -> CreationProfile { let mut p = CreationProfile::experienced(); p.neow_eligible = full; p }
fn standard(character: Character, full: bool) -> Run {
    Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile(full), rng(0x28), RoomScenario::Standard).unwrap()
}

#[test]
fn m28_standard_creation_all_characters() {
    // Base max HP (run/engine.rs:13-17) and b = T.1(M) (design §6.5): 80/8, 70/7, 75/7, 72/7.
    for (character, max_hp, b) in [(Character::Ironclad, 80, 8), (Character::Silent, 70, 7), (Character::Defect, 75, 7), (Character::Watcher, 72, 7)] {
        for full in [false, true] {
            let run = standard(character, full);
            assert!(rooms::is_standard(&run.state));
            assert_eq!(run.state.card_admission, CardAdmission::Complete);
            assert_eq!(run.state.item_admission, ItemAdmission::Complete);
            assert!(run.state.relic_pools.is_some());
            assert_eq!(run.state.card_rarity_offset, Some(5));
            assert_eq!((run.state.hp, run.state.max_hp, run.state.gold), (max_hp, max_hp, 99));
            let view = rooms::public(&run).unwrap();
            assert_eq!(view.kind, RoomViewKind::Neow);
            let kinds: Vec<_> = view.choices.iter().map(|c| c.kind.clone()).collect();
            if full {
                let categories: Vec<_> = kinds.iter().map(|k| match k { RoomChoiceKind::NeowBlessing { category, .. } => *category, _ => panic!("{k:?}") }).collect();
                assert_eq!(categories, [0, 1, 2, 3], "{character:?}");
                assert!(matches!(kinds[3], RoomChoiceKind::NeowBlessing { reward: NeowReward::BossSwap, drawback: None, .. }));
                assert!(matches!(kinds[2], RoomChoiceKind::NeowBlessing { drawback: Some(_), .. }));
            } else {
                assert_eq!(kinds, [RoomChoiceKind::NeowLament, RoomChoiceKind::NeowMaxHp { amount: b }], "{character:?}");
            }
        }
        // Amended (M29a1, plan erratum v3 n2): A0 only in a default build until M29f; A6+ never.
        // Amended at the M29 close (n2): A1 is public in every build.
        assert!(Run::new_room_scenario(RunConfig { character, ascension: 1 }, profile(false), rng(1), RoomScenario::Standard).is_ok());
        assert_eq!(Run::new_room_scenario(RunConfig { character, ascension: 6 }, profile(false), rng(1), RoomScenario::Standard).err(),
            Some(RunError::Unsupported("Standard supports A0–A5")));
    }
}

#[test]
fn m28_standard_short_neow_opens_generated_act1_map() {
    for character in CHARACTERS {
        let mut run = standard(character, false);
        let id = rooms::public(&run).unwrap().choices[1].id;
        run.step(&GameAction::Choose(id)).unwrap();
        let max_hp = match character { Character::Ironclad => 88, Character::Silent => 77, Character::Defect => 82, Character::Watcher => 79 };
        assert_eq!((run.state.hp, run.state.max_hp), (max_hp, max_hp));
        let map = run.state.map.as_ref().unwrap();
        assert_eq!(map.act, 1);
        assert!(matches!(map.boss_icon, Some(crate::content::EncounterId::Guardian | crate::content::EncounterId::Hexaghost | crate::content::EncounterId::SlimeBoss)));
        assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
        // M28b: the first floor is a monster room entered through the standard owner.
        let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).unwrap();
        run.step(&GameAction::Move(target)).unwrap();
        assert!(run.state.combat.is_some(), "{character:?} floor 1 is a combat");
    }
    // Lament under complete admission keeps the M27 relic owner.
    let mut run = standard(Character::Silent, false);
    let id = rooms::public(&run).unwrap().choices[0].id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert!(run.state.relics.contains(&crate::content::RelicId::NeowsLament));
}

#[test]
fn m28_rng_scopes_appended() {
    let scopes = [
        ("run.m28.event.branch", RunRngScope::M28EventBranch), ("run.m28.event.pick", RunRngScope::M28EventPick),
        ("run.m28.event.body", RunRngScope::M28EventBody), ("run.m28.card.rarity", RunRngScope::M28CardRarity),
        ("run.m28.shop.stock", RunRngScope::M28ShopStock), ("run.m28.shop.price", RunRngScope::M28ShopPrice),
        ("run.m28.chest", RunRngScope::M28Chest),
    ];
    let mut previous = RunRngScope::M27PotionOffer;
    for (label, scope) in scopes {
        assert_eq!(RunRngScope::parse(label), Ok(scope));
        assert!(previous < scope, "{scope:?} follows {previous:?}");
        previous = scope;
    }
}

#[test]
fn m28_decrease_max_hp_owner() {
    for (character, max_hp) in [(Character::Ironclad, 80), (Character::Silent, 70), (Character::Defect, 75), (Character::Watcher, 72)] {
        let mut run = standard(character, false);
        assert_eq!(run.state.max_hp, max_hp);
        mutations::decrease_max_hp(&mut run.state, 7);
        assert_eq!((run.state.hp, run.state.max_hp), (max_hp - 7, max_hp - 7));
        run.state.hp = 10;
        mutations::decrease_max_hp(&mut run.state, 3);
        assert_eq!((run.state.hp, run.state.max_hp), (10, max_hp - 10));
        mutations::decrease_max_hp(&mut run.state, 999);
        assert_eq!((run.state.hp, run.state.max_hp), (1, 1));
    }
}

#[test]
fn m28_g1_starter_tags_all_characters() {
    // Disclosed global correction G1: every character's basic Strike and Defend carries the
    // source STARTER_STRIKE / STARTER_DEFEND tag; other cards do not.
    use crate::content::card_tags;
    for strike in [CardId::Strike, CardId::StrikeGreen, CardId::StrikeBlue, CardId::StrikePurple] {
        assert!(card_tags(strike).contains(&CardTag::StarterStrike), "{strike:?}");
        assert!(card_tags(strike).contains(&CardTag::Strike), "{strike:?}");
    }
    for defend in [CardId::Defend, CardId::DefendGreen, CardId::DefendBlue, CardId::DefendPurple] {
        assert!(card_tags(defend).contains(&CardTag::StarterDefend), "{defend:?}");
    }
    for other in [CardId::Bash, CardId::Neutralize, CardId::Survivor, CardId::Zap, CardId::Dualcast, CardId::Eruption,
        CardId::Vigilance, CardId::PerfectedStrike, CardId::TwinStrike] {
        assert!(!card_tags(other).iter().any(|t| matches!(t, CardTag::StarterStrike | CardTag::StarterDefend)), "{other:?}");
    }
    // Every starter deck: exactly its Strikes and Defends are starter-tagged.
    for character in CHARACTERS {
        let deck = crate::content::Catalog::standard().starter_deck(character).unwrap();
        let tagged = deck.iter().filter(|id| card_tags(**id).iter().any(|t| matches!(t, CardTag::StarterStrike | CardTag::StarterDefend))).count();
        // Ironclad 5+4, Silent 5+5, Defect 4+4, Watcher 4+4 (starter decks, content/cards.rs).
        let expected = match character { Character::Ironclad => 9, Character::Silent => 10, Character::Defect => 8, Character::Watcher => 8 };
        assert_eq!(tagged, expected, "{character:?}");
    }
}

#[test]
fn m28_frozen_profiles_never_reach_standard_owners() {
    use RoomScenario::*;
    let frozen = [(M4Act1Prefix, Character::Ironclad), (M5Act1Captured, Character::Ironclad), (M6Act2Prefix, Character::Ironclad),
        (M7Act2Captured, Character::Ironclad), (M8Act3Prefix, Character::Ironclad), (M9IroncladAct3, Character::Ironclad),
        (M10IroncladHeart, Character::Ironclad), (M12SilentAct1, Character::Silent), (M13SilentAct2, Character::Silent),
        (M14SilentAct3, Character::Silent), (M15SilentHeart, Character::Silent), (M17DefectAct1, Character::Defect),
        (M18DefectAct2, Character::Defect), (M19DefectAct3, Character::Defect), (M20DefectHeart, Character::Defect),
        (M22WatcherAct1, Character::Watcher), (M23WatcherAct2, Character::Watcher), (M24WatcherAct3, Character::Watcher),
        (M25WatcherHeart, Character::Watcher)];
    for (scenario, character) in frozen {
        let run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile(false), rng(3), scenario).unwrap();
        assert!(!rooms::is_standard(&run.state), "{scenario:?}");
        assert_eq!(run.state.card_rarity_offset, None, "{scenario:?}");
        assert!(!format!("{:?}", run.state).contains("card_rarity_offset"), "{scenario:?}");
        assert!(rooms::public(&run).unwrap().choices.iter().all(|c| !matches!(c.kind, RoomChoiceKind::NeowBlessing { .. })), "{scenario:?}");
        // M28a a-owners: no pre-map selection node, no standard shop screen.
        assert_eq!(rooms::selection_node(&run.state), Err(RunError::InvalidAction), "{scenario:?}");
        assert_eq!(rooms::shop::standard_screen(&mut run.clone(), vec![]).err(), Some(RunError::InvalidAction), "{scenario:?}");
        // The frozen scenarios still reject the full blessing.
        assert!(Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile(true), rng(3), scenario).is_err(), "{scenario:?}");
    }
}

/// Global fix (M28a, disclosed): a Complete shop purge is priced from its displayed row, so a
/// discounted removal (Smiling Mask 50, Membership 38, Courier 60) is purchasable. Before the fix
/// the select compared the displayed price with the run-wide base 75 and rejected it.
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m28_discounted_complete_purge_is_purchasable() {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    use crate::content::RelicId;
    for (relic, price) in [(RelicId::SmilingMask, 50), (RelicId::MembershipCard, 38), (RelicId::TheCourier, 60)] {
        let supply = CompleteItemSupply { relics: vec![relic], potions: vec![None; 3], counters: std::collections::BTreeMap::new() };
        let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
        for floor in [51, 52] {
            if floor == 52 {
                let rest = run.decision().unwrap().room.unwrap().choices.into_iter()
                    .find(|row| row.status == RoomChoiceStatus::Offered && matches!(row.kind, RoomChoiceKind::Rest { .. })).unwrap().id;
                run.step(&GameAction::Choose(rest)).unwrap();
            }
            let target = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == floor).unwrap().id;
            run.step(&GameAction::Move(target)).unwrap();
        }
        let purge = run.decision().unwrap().room.unwrap().choices.into_iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
        assert_eq!(purge.price, Some(price), "{relic:?} removal price");
        let gold = run.state.gold;
        run.step(&GameAction::Choose(purge.id)).unwrap();
        let select = run.decision().unwrap().candidates.into_iter()
            .find(|candidate| matches!(candidate.action, GameAction::Select(_))).unwrap().action;
        run.step(&select).unwrap();
        assert_eq!(run.state.gold, gold - price, "{relic:?} spends the displayed price");
        assert_eq!(run.state.room_loop.as_ref().unwrap().purge_cost, 100, "the run-wide base still rises by 25");
    }
}

/// Global fix G5 (M28a, disclosed): every shop purge, not only the standard shop's, excludes the
/// unremovable curses and bottled cards (economy research :161), and a purge that cannot remove its
/// card spends nothing. Before the fix the M27 Complete Act 4 shop offered Necronomicurse and a
/// bottled card, and selecting the curse spent gold and raised the base before failing.
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m28_g5_every_shop_purge_excludes_unremovable_and_bottled() {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    use crate::content::RelicId;
    let supply = CompleteItemSupply { relics: vec![], potions: vec![None; 3], counters: std::collections::BTreeMap::new() };
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
    for floor in [51, 52] {
        if floor == 52 {
            let rest = run.decision().unwrap().room.unwrap().choices.into_iter()
                .find(|row| row.status == RoomChoiceStatus::Offered && matches!(row.kind, RoomChoiceKind::Rest { .. })).unwrap().id;
            run.step(&GameAction::Choose(rest)).unwrap();
        }
        let target = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == floor).unwrap().id;
        run.step(&GameAction::Move(target)).unwrap();
    }
    let curse = run.state.next_instance;
    run.state.deck.push(CardInstance { id: CardId::Necronomicurse, instance: curse, upgrades: 0, misc: 0 });
    run.state.next_instance += 1;
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Bash).unwrap().instance;
    run.state.relics.push(RelicId::BottledFlame);
    run.state.relic_cards.insert(RelicId::BottledFlame, bottled);
    let purge = run.decision().unwrap().room.unwrap().choices.into_iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
    run.step(&GameAction::Choose(purge.id)).unwrap();
    let offered = run.state.pending.as_ref().unwrap().selection.candidates.iter().map(|candidate| candidate.card.instance).collect::<Vec<_>>();
    assert!(!offered.contains(&curse) && !offered.contains(&bottled), "purge candidates {offered:?}");
    assert_eq!(offered.len(), run.state.deck.len() - 2);
    // A candidate that cannot be removed is rejected before any gold is spent.
    let forged = SelectionId(u64::MAX - 1);
    run.state.pending.as_mut().unwrap().selection.candidates.push(SelectionCandidate { id: forged, card: run.state.deck.iter().find(|card| card.instance == curse).unwrap().clone() });
    run.publish_selection_candidates().unwrap();
    assert!(run.decision().unwrap().candidates.iter().any(|candidate| candidate.action == GameAction::Select(forged)));
    let (gold, base) = (run.state.gold, run.state.room_loop.as_ref().unwrap().purge_cost);
    assert_eq!(run.step(&GameAction::Select(forged)), Err(RunError::InvalidAction));
    assert_eq!((run.state.gold, run.state.room_loop.as_ref().unwrap().purge_cost), (gold, base), "nothing spent");
    assert!(run.state.deck.iter().any(|card| card.instance == curse));
}

/// Global fix (M28a, disclosed): at shop entry The Courier's ×.8 pass precedes Membership Card's
/// ×.5 pass whatever the acquisition order (economy research :172). Literal: 53 → G(F(53×.8)) = 42
/// → G(F(42×.5)) = 21; the reversed order gives 27 → 22.
#[test]
fn m28_shop_price_courier_pass_precedes_membership() {
    use crate::content::RelicId;
    use crate::items::dispatch::{run_event, RunItemEvent};
    for relics in [[RelicId::TheCourier, RelicId::MembershipCard], [RelicId::MembershipCard, RelicId::TheCourier]] {
        let mut run = standard(Character::Ironclad, false);
        run.state.relics.extend(relics);
        let mut event = RunItemEvent::ShopPrice { price: 53 };
        run_event(&mut run.state, &mut event).unwrap();
        assert_eq!(event, RunItemEvent::ShopPrice { price: 21 }, "{relics:?}");
    }
}
