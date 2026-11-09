//! M30 run rules (plan items 1, 4–8): the Standard cap at A10, the A6 start-HP law, the A10
//! Ascender's Bane at master-deck index 0, and the global Toke / shop purge / raw-remove fixes.
//! Literals come from the source check (`m30-source-check-v1.md` §B, §D, §E), never from the code
//! under test.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::Character;

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn standard(character: Character, ascension: u8, full: bool) -> Result<Run, RunError> {
    Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(0x30), RoomScenario::Standard)
}

#[test]
fn m30_standard_creation_a0_to_a10_all_characters() {
    for character in CHARACTERS {
        for ascension in 0..=10 {
            for full in [false, true] {
                let run = standard(character, ascension, full).unwrap_or_else(|e| panic!("{character:?} A{ascension}: {e:?}"));
                assert_eq!(run.decision().unwrap().content_profile, Some(RoomScenario::Standard));
                assert_eq!(run.state.ascension, ascension);
                assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Neow, "{character:?} A{ascension}");
            }
        }
    }
}

/// Formerly `m30_standard_rejects_a11_and_above` (M31 plan item 1(c)): the cap is A15 since M31, so
/// A10 and A11 are admitted and the rejected levels are 16, 20 and 21, with the A0–A15 message. Amended
/// at M32 (plan item 1): the cap is A19, so the rejected levels are 20 and 21, with the A0–A19 message.
/// Amended at M33 (plan item 1): the cap is A20, so the rejected level is 21, with the A0–A20 message.
#[test]
fn m30_standard_rejects_above_cap() {
    for character in CHARACTERS {
        assert!(standard(character, 10, false).is_ok(), "{character:?} A10");
        assert!(standard(character, 11, false).is_ok(), "{character:?} A11");
        // Amended at M33 (plan item 1): the cap is A20, so A21 is the rejected level.
        for ascension in [21] {
            assert_eq!(standard(character, ascension, false).err(),
                Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A{ascension}");
            assert_eq!(rooms::standard::build(RunConfig { character, ascension }, profile(false), rng(1)).err(),
                Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A{ascension}");
        }
    }
}

// ---- fixtures -------------------------------------------------------------------------------

use crate::content::{CardId, RelicId};

/// A Standard A10 Run (public route, short Neow) past Neow's max-HP blessing, on its Act 1 map.
fn a10_on_map(character: Character) -> Run {
    let mut run = standard(character, 10, false).unwrap();
    let max_hp = rooms::public(&run).unwrap().choices.iter().find(|c| matches!(c.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
    run.step(&GameAction::Choose(max_hp)).unwrap();
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
    run
}
/// Replace the master deck (fresh instances, no hooks) and bottle `bottled` (a deck index) in
/// Bottled Flame. Relics are appended without equip hooks, as `diagnostic_shop` does.
fn supply(run: &mut Run, deck: &[CardId], bottled: Option<usize>, relics: &[RelicId]) {
    run.state.deck.clear();
    for id in deck {
        let instance = run.state.next_instance; run.state.next_instance += 1;
        run.state.deck.push(CardInstance { misc: 0, id: *id, instance, upgrades: 0 });
    }
    let mut relics = relics.to_vec();
    if let Some(index) = bottled {
        relics.push(RelicId::BottledFlame);
        run.state.relic_cards.insert(RelicId::BottledFlame, run.state.deck[index].instance);
    }
    for relic in relics {
        if let Some(pools) = run.state.relic_pools.as_mut() { crate::items::relic_pools::remove(pools, relic); }
        run.state.relics.push(relic);
    }
}
/// Turn the first reachable Act 1 node into `kind` and enter it through the public Move.
fn enter_first(run: &mut Run, kind: MapNodeKind) {
    let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = kind;
    run.step(&GameAction::Move(target)).unwrap();
}
fn row(run: &Run, kind: RoomChoiceKind) -> rooms::RoomChoiceView {
    rooms::public(run).unwrap().choices.into_iter().find(|c| c.kind == kind).unwrap_or_else(|| panic!("no {kind:?} row"))
}
fn instance_of(run: &Run, id: CardId) -> u64 { run.state.deck.iter().find(|c| c.id == id).unwrap().instance }
/// Bane, Necronomicurse, Curse of the Bell and a bottled Strike: nothing is purgeable.
const UNPURGEABLE: [CardId; 4] = [CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike];

// ---- G1: Peace Pipe Toke (design §6.5; PeacePipe.java:53, CampfireTokeEffect.java:61-71) ------

#[test]
fn m30_toke_excludes_unpurgeable_and_bottled() {
    for character in CHARACTERS {
        let mut run = a10_on_map(character);
        supply(&mut run, &[CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike, CardId::Defend], Some(3), &[RelicId::PeacePipe]);
        enter_first(&mut run, MapNodeKind::Rest);
        let toke = row(&run, RoomChoiceKind::Toke);
        assert_eq!(toke.status, RoomChoiceStatus::Offered, "{character:?}");
        run.step(&GameAction::Choose(toke.id)).unwrap();
        let offered: Vec<CardId> = run.decision().unwrap().candidates.iter().filter_map(|c| match c.action { GameAction::Select(_) => c.object.as_ref().map(|o| o.id), _ => None }).collect();
        assert_eq!(offered, vec![CardId::Defend], "{character:?}: only the unbottled purgeable card");
    }
}

#[test]
fn m30_toke_disabled_without_purgeable() {
    for character in CHARACTERS {
        let mut run = a10_on_map(character);
        supply(&mut run, &UNPURGEABLE, Some(3), &[RelicId::PeacePipe]);
        enter_first(&mut run, MapNodeKind::Rest);
        let toke = row(&run, RoomChoiceKind::Toke);
        assert_eq!(toke.status, RoomChoiceStatus::Disabled, "{character:?}");
        assert!(!run.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Choose(toke.id)), "{character:?}: not advertised");
        let before = run.clone();
        assert_eq!(run.step(&GameAction::Choose(toke.id)).err(), Some(RunError::InvalidAction), "{character:?}");
        assert!(run == before, "{character:?}: rejected atomically");
        // t-1: one purgeable card re-enables it (status is derived, row ids unchanged).
        let mut one = before.clone();
        let instance = one.state.next_instance; one.state.next_instance += 1;
        one.state.deck.push(CardInstance { misc: 0, id: CardId::Defend, instance, upgrades: 0 });
        assert_eq!(row(&one, RoomChoiceKind::Toke), rooms::RoomChoiceView { status: RoomChoiceStatus::Offered, ..toke.clone() }, "{character:?}");
    }
}

#[test]
fn m30_toke_cancel_returns_to_campfire() {
    for character in CHARACTERS {
        let mut run = a10_on_map(character);
        supply(&mut run, &[CardId::AscendersBane, CardId::Strike, CardId::Defend], None, &[RelicId::PeacePipe]);
        enter_first(&mut run, MapNodeKind::Rest);
        let deck = run.state.deck.clone();
        let toke = row(&run, RoomChoiceKind::Toke);
        run.step(&GameAction::Choose(toke.id)).unwrap();
        assert!(run.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Cancel), "{character:?}: Toke offers Cancel (CancelButton.java:98-108)");
        run.step(&GameAction::Cancel).unwrap();
        assert!(run.state.pending.is_none(), "{character:?}");
        assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Campfire, "{character:?}");
        assert_eq!(row(&run, RoomChoiceKind::Toke).status, RoomChoiceStatus::Offered, "{character:?}: Toke stays usable");
        assert_eq!(run.state.deck, deck, "{character:?}: nothing removed");
        // The campfire is still usable: Toke again, then a removal completes it.
        run.step(&GameAction::Choose(toke.id)).unwrap();
        let pick = run.decision().unwrap().candidates.iter().find(|c| matches!(c.action, GameAction::Select(_)) && c.object.as_ref().is_some_and(|o| o.id == CardId::Defend)).unwrap().action.clone();
        run.step(&pick).unwrap();
        assert!(!run.state.deck.iter().any(|c| c.id == CardId::Defend), "{character:?}");
    }
}

#[test]
fn m30_campfire_only_disabled_toke_completes_at_entry() {
    for character in CHARACTERS {
        let mut run = a10_on_map(character);
        supply(&mut run, &UNPURGEABLE, Some(3), &[RelicId::PeacePipe, RelicId::CoffeeDripper, RelicId::FusionHammer]);
        run.state.keys[0] = true;
        enter_first(&mut run, MapNodeKind::Rest);
        assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map, "{character:?}: CampfireUI.java:131-139 completes");
        assert!(!run.decision().unwrap().candidates.is_empty(), "{character:?}: not stuck");
        // t-1: with one purgeable card the Toke row keeps the campfire open.
        let mut one = a10_on_map(character);
        supply(&mut one, &[CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike, CardId::Defend], Some(3), &[RelicId::PeacePipe, RelicId::CoffeeDripper, RelicId::FusionHammer]);
        one.state.keys[0] = true;
        enter_first(&mut one, MapNodeKind::Rest);
        assert_eq!(rooms::public(&one).unwrap().kind, RoomViewKind::Campfire, "{character:?}");
        assert_eq!(row(&one, RoomChoiceKind::Toke).status, RoomChoiceStatus::Offered, "{character:?}");
    }
}

// ---- G2: empty shop purge (design §6.6, ruling D1/EVENT-001; ShopScreen.java:256,1007-1015) ----

fn a10_shop(character: Character, deck: &[CardId], bottled: Option<usize>) -> Run {
    let mut run = a10_on_map(character);
    supply(&mut run, deck, bottled, &[]);
    run.state.gold = 999;
    enter_first(&mut run, MapNodeKind::Shop);
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Shop);
    run
}

#[test]
fn m30_shop_purge_disabled_without_purgeable() {
    for character in CHARACTERS {
        let mut run = a10_shop(character, &UNPURGEABLE, Some(3));
        let purge = row(&run, RoomChoiceKind::Purge);
        assert_eq!(purge.status, RoomChoiceStatus::Disabled, "{character:?}");
        assert!(!run.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Choose(purge.id)), "{character:?}");
        let before = run.clone();
        assert_eq!(run.step(&GameAction::Choose(purge.id)).err(), Some(RunError::InvalidAction), "{character:?}");
        assert!(run == before && run.state.fault.is_none(), "{character:?}: no charge, no fault");
        // t-1: one unbottled purgeable card leaves the row Offered.
        let one = a10_shop(character, &[CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike, CardId::Defend], Some(3));
        assert_eq!(row(&one, RoomChoiceKind::Purge).status, RoomChoiceStatus::Offered, "{character:?}");
    }
}

#[test]
fn m30_shop_purge_reenabled_after_buy() {
    for character in CHARACTERS {
        let mut run = a10_shop(character, &UNPURGEABLE, Some(3));
        let purge = row(&run, RoomChoiceKind::Purge);
        assert_eq!(purge.status, RoomChoiceStatus::Disabled, "{character:?}");
        let card = rooms::public(&run).unwrap().choices.into_iter().find(|c| matches!(c.kind, RoomChoiceKind::Card { .. }) && c.status == RoomChoiceStatus::Offered).unwrap();
        let RoomChoiceKind::Card { id: bought, .. } = card.kind else { unreachable!() };
        run.step(&GameAction::Choose(card.id)).unwrap();
        let after = row(&run, RoomChoiceKind::Purge);
        assert_eq!((after.id, after.status), (purge.id, RoomChoiceStatus::Offered), "{character:?}: same row, re-enabled");
        run.step(&GameAction::Choose(purge.id)).unwrap();
        let offered: Vec<CardId> = run.decision().unwrap().candidates.iter().filter_map(|c| match c.action { GameAction::Select(_) => c.object.as_ref().map(|o| o.id), _ => None }).collect();
        assert_eq!(offered, vec![bought], "{character:?}");
    }
}

#[test]
fn m30_shop_purge_no_fault() {
    for character in CHARACTERS {
        let run = a10_shop(character, &UNPURGEABLE, Some(3));
        for candidate in run.decision().unwrap().candidates {
            let mut probe = run.clone();
            let result = probe.step(&candidate.action);
            assert!(result.is_ok(), "{character:?}: advertised {:?} rejected: {result:?}", candidate.action);
            assert!(probe.state.fault.is_none(), "{character:?}: {:?} faulted", candidate.action);
        }
    }
}

// ---- G3: raw remove goes through the removal owner (design §6.7) ------------------------------

fn generic(character: Character, ascension: u8) -> Run {
    Run::new(RunConfig { character, ascension }, CreationProfile::experienced(), rng(0x31)).unwrap()
}

#[test]
fn m30_raw_remove_refuses_unremovable_curses() {
    for character in CHARACTERS {
        let mut run = generic(character, 10);
        for id in [CardId::Necronomicurse, CardId::CurseOfTheBell] {
            let instance = run.state.next_instance; run.state.next_instance += 1;
            run.state.deck.push(CardInstance { misc: 0, id, instance, upgrades: 0 });
        }
        for id in [CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell] {
            let before = run.clone();
            assert_eq!(run.remove_card(instance_of(&run, id)).err(), Some(RunError::InvalidAction), "{character:?} {id:?}");
            assert!(run == before, "{character:?} {id:?}: deck unchanged");
        }
        // t-1: an ordinary card is still removed.
        let strike = run.state.deck.iter().find(|c| c.id != CardId::AscendersBane && c.id != CardId::Necronomicurse && c.id != CardId::CurseOfTheBell).unwrap().clone();
        assert_eq!(run.remove_card(strike.instance), Ok(strike), "{character:?}");
    }
}

#[test]
fn m30_raw_remove_uses_owner_side_effects() {
    for character in CHARACTERS {
        let mut run = generic(character, 0);
        let bottled = run.state.deck[0].instance;
        run.state.relics.push(RelicId::BottledFlame);
        run.state.relic_cards.insert(RelicId::BottledFlame, bottled);
        run.remove_card(bottled).unwrap();
        assert!(run.state.relic_cards.is_empty(), "{character:?}: the bottle link goes with its card");
        let parasite = run.state.next_instance; run.state.next_instance += 1;
        run.state.deck.push(CardInstance { misc: 0, id: CardId::Parasite, instance: parasite, upgrades: 0 });
        let (hp, max_hp) = (run.state.hp, run.state.max_hp);
        let _ = crate::items::dispatch::take_run_event_log();
        run.remove_card(parasite).unwrap();
        assert_eq!(run.state.max_hp, max_hp - 3, "{character:?}: Parasite costs 3 max HP");
        assert!(run.state.hp <= hp && run.state.hp <= run.state.max_hp, "{character:?}");
        assert!(crate::items::dispatch::take_run_event_log().contains(&"CardRemoved"), "{character:?}: CardRemoved dispatched");
    }
}

// ---- E2: A6 start HP (source check §B; MathUtils.round(max * 0.9f)) ------------------------------

#[test]
fn m30_a6_start_hp_all_characters() {
    // (character, A0–A5, A6–A13 current HP, A14+ (current, max)) — source check §B literals.
    let table = [(Character::Ironclad, 80, 72, (68, 75)), (Character::Silent, 70, 63, (59, 66)),
        (Character::Defect, 75, 68, (64, 71)), (Character::Watcher, 72, 65, (61, 68))];
    for (character, max_hp, a6, a14) in table {
        for full in [false, true] {
            let at = |a| { let r = standard(character, a, full).unwrap(); (r.state.hp, r.state.max_hp) };
            assert_eq!(at(5), (max_hp, max_hp), "{character:?} A5 (t-1)");
            assert_eq!(at(6), (a6, max_hp), "{character:?} A6");
            assert_eq!(at(10), (a6, max_hp), "{character:?} A10");
        }
        let at = |a| { let r = generic(character, a); (r.state.hp, r.state.max_hp) };
        assert_eq!(at(13), (a6, max_hp), "{character:?} A13 (generic Run::new)");
        assert_eq!(at(14), a14, "{character:?} A14 (generic Run::new)");
    }
}

// ---- E3: the Bane is the first master-deck card (source check §D) ------------------------------

#[test]
fn m30_bane_insertion_dispatches_no_item_event() {
    for character in CHARACTERS {
        let _ = crate::items::dispatch::take_run_event_log();
        let run = generic(character, 10);
        assert_eq!(crate::items::dispatch::take_run_event_log(), Vec::<&str>::new(), "{character:?}");
        assert_eq!(run.state.deck.iter().filter(|c| c.id == CardId::AscendersBane).count(), 1, "{character:?}");
    }
}

#[test]
fn m30_bane_first_before_neow() {
    for character in CHARACTERS {
        for full in [false, true] {
            let _ = crate::items::dispatch::take_run_event_log();
            let a9 = standard(character, 9, full).unwrap();
            let a9_log = crate::items::dispatch::take_run_event_log();
            assert!(!a9.state.deck.iter().any(|c| c.id == CardId::AscendersBane), "{character:?} A9 (t-1)");
            let starters: Vec<CardId> = a9.state.deck.iter().map(|c| c.id).collect();
            let a10 = standard(character, 10, full).unwrap();
            // Direct insertion dispatches no item event: the A10 log equals A9's (whose only obtain
            // events are Standard's stored-note probe on a scratch state).
            assert_eq!(crate::items::dispatch::take_run_event_log(), a9_log, "{character:?}: no obtain hooks for the Bane");
            assert_eq!(rooms::public(&a10).unwrap().kind, RoomViewKind::Neow, "{character:?}: before the first Neow choice");
            assert_eq!(a10.state.deck[0].id, CardId::AscendersBane, "{character:?} full {full}");
            assert_eq!(a10.state.deck[1..].iter().map(|c| c.id).collect::<Vec<_>>(), starters, "{character:?}: starters follow");
            assert!(a10.state.history.events.is_empty(), "{character:?}: no committed obtain event");
        }
    }
}

#[test]
fn m30_bane_position_feeds_opening_shuffle() {
    // Combat entry copies the master deck in order before its seeded shuffle, so the same seed
    // applies the same permutation to the deck's positions: moving the Bane moves the opening. This
    // holds for the Watcher too since G4 (PM ruling W1): before it, the Standard Watcher shuffled a
    // canonical (id, upgrades, instance) order and the Bane's position had no effect.
    for character in CHARACTERS {
        let mut first = a10_on_map(character);
        assert_eq!(first.state.deck[0].id, CardId::AscendersBane);
        let mut last = first.clone();
        let bane = last.state.deck.remove(0); last.state.deck.push(bane);
        let positions = |run: &Run| run.state.deck.iter().map(|c| c.instance).collect::<Vec<_>>();
        let (first_deck, last_deck) = (positions(&first), positions(&last));
        let target = first.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
        first.step(&GameAction::Move(target)).unwrap();
        last.step(&GameAction::Move(target)).unwrap();
        // Master-deck cards only (the Watcher's Pure Water adds a generated Miracle).
        let order = |run: &Run| run.decision().unwrap().combat.unwrap().hand.iter().map(|c| c.instance.0)
            .filter(|i| run.state.deck.iter().any(|c| c.instance == *i)).collect::<Vec<_>>();
        let (a, b) = (order(&first), order(&last));
        assert!(a.len() >= 5, "{character:?}: opening hand");
        let index_a: Vec<usize> = a.iter().map(|i| first_deck.iter().position(|x| x == i).unwrap()).collect();
        let index_b: Vec<usize> = b.iter().map(|i| last_deck.iter().position(|x| x == i).unwrap()).collect();
        assert_eq!(index_a, index_b, "{character:?}: the opening hand takes the same deck positions");
        assert_ne!(a, b, "{character:?}: the Bane's position changes the opening order");
    }
}

// ---- Item 9(a): Neow's HP rewards and drawbacks read the A6 HP --------------------------------

/// As `m28_neow::force_offer`: force the persisted candidate and its visible row.
fn choose_neow(run: &mut Run, category: u8, reward: NeowReward, drawback: Option<NeowDrawback>) {
    let candidate = run.candidates.iter_mut().find(|c| c.neow_offer.as_ref().is_some_and(|o| o.category == category)).unwrap();
    candidate.neow_offer = Some(NeowOffer { category, reward, drawback });
    let id = candidate.id;
    run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|r| r.id == id).unwrap().kind =
        RoomChoiceKind::NeowBlessing { category, reward, drawback };
    run.step(&GameAction::Choose(id)).unwrap_or_else(|e| panic!("{reward:?}/{drawback:?}: {e:?}"));
}

#[test]
fn m30_neow_hp_rewards_and_drawbacks_at_a6() {
    // Hand-worked from the §B start HP H (72/63/68/65 of M 80/70/75/72) and b = floor(M * 0.1f)
    // (8/7/7/7): MaxHp (H+b, M+b), MaxHpTwice (H+2b, M+2b), MaxHpLoss (min(H, M-b), M-b),
    // CurrentHpLoss (H - 3*floor(H/10), M).
    let table = [
        (Character::Ironclad, (80, 88), (88, 96), (72, 72), (51, 80)),
        (Character::Silent, (70, 77), (77, 84), (63, 63), (45, 70)),
        (Character::Defect, (75, 82), (82, 89), (68, 68), (50, 75)),
        (Character::Watcher, (72, 79), (79, 86), (65, 65), (47, 72)),
    ];
    for ascension in [6, 10] {
        for (character, max_hp, twice, loss, hp_loss) in table {
            let hp = |run: &Run| (run.state.hp, run.state.max_hp);
            let mut run = standard(character, ascension, true).unwrap();
            choose_neow(&mut run, 1, NeowReward::MaxHp, None);
            assert_eq!(hp(&run), max_hp, "{character:?} A{ascension} MaxHp");
            let mut run = standard(character, ascension, true).unwrap();
            choose_neow(&mut run, 2, NeowReward::MaxHpTwice, Some(NeowDrawback::NoGold));
            assert_eq!(hp(&run), twice, "{character:?} A{ascension} MaxHpTwice");
            let mut run = standard(character, ascension, true).unwrap();
            choose_neow(&mut run, 2, NeowReward::Gold250, Some(NeowDrawback::MaxHpLoss));
            assert_eq!(hp(&run), loss, "{character:?} A{ascension} MaxHpLoss");
            let mut run = standard(character, ascension, true).unwrap();
            choose_neow(&mut run, 2, NeowReward::Gold250, Some(NeowDrawback::CurrentHpLoss));
            assert_eq!(hp(&run), hp_loss, "{character:?} A{ascension} CurrentHpLoss");
        }
    }
    // The short blessing heals its amount from the A6 HP too.
    for (character, max_hp, ..) in table {
        let run = a10_on_map(character);
        assert_eq!((run.state.hp, run.state.max_hp), max_hp, "{character:?} short MaxHp");
    }
}

// ---- Item 9(d): an A10 combat settles with the master-deck Bane in place ----------------------

#[test]
fn m30_a10_combat_settlement_keeps_bane() {
    for character in CHARACTERS {
        let mut run = a10_on_map(character);
        let deck = run.state.deck.clone();
        let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = MapNodeKind::Monster;
        run.step(&GameAction::Move(target)).unwrap();
        assert!(run.state.combat.is_some(), "{character:?}: in combat");
        // End turns until the Bane has been drawn and exhausted, then win through the shared settlement.
        let mut exhausted = false;
        for _ in 0..6 {
            let view = run.decision().unwrap().combat.unwrap();
            if view.exhaust.iter().any(|c| c.id == CardId::AscendersBane) { exhausted = true; break; }
            if view.outcome.is_some() { break; }
            run.step(&GameAction::Combat(crate::combat::CombatAction::EndTurn)).unwrap();
        }
        assert!(exhausted, "{character:?}: the combat copy of the Bane was exhausted (Ethereal)");
        super::m28_world::win(&mut run);
        assert!(run.state.combat.is_none(), "{character:?}: settled");
        assert_eq!(run.state.deck[0].id, CardId::AscendersBane, "{character:?}: master-deck Bane at index 0");
        assert_eq!(run.state.deck, deck, "{character:?}: master deck unchanged by the combat");
    }
}

// ---- G4 (PM ruling W1): a Standard combat shuffles the master deck in its own order -------------

#[test]
fn m30_g4_standard_combat_shuffles_master_order_at_a0() {
    // Reversing the master deck must reverse which deck positions the opening hand takes: the
    // same seed permutes positions, whatever the cards. Before G4 the Standard Watcher sorted the
    // pile by (id, upgrades, instance) first, so its hand ignored the master order.
    for character in CHARACTERS {
        let mut run = standard(character, 0, false).unwrap();
        let max_hp = rooms::public(&run).unwrap().choices.iter().find(|c| matches!(c.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
        run.step(&GameAction::Choose(max_hp)).unwrap();
        let mut reversed = run.clone();
        reversed.state.deck.reverse();
        let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
        let positions = |r: &mut Run| {
            let deck: Vec<u64> = r.state.deck.iter().map(|c| c.instance).collect();
            r.step(&GameAction::Move(target)).unwrap();
            r.decision().unwrap().combat.unwrap().hand.iter().filter_map(|c| deck.iter().position(|i| *i == c.instance.0)).collect::<Vec<_>>()
        };
        assert_eq!(positions(&mut run), positions(&mut reversed), "{character:?}: the opening hand takes the same deck positions");
    }
}

// ---- G5: Red Skull's latch after the victory heal (RedSkull.java onBloodied/onVictory) ------------

#[test]
fn m30_red_skull_latch_cleared_after_settlement_heal() {
    // Found by the M30 A8/A9 supplied traversal (Ironclad/5, floor 30). Burning Blood's victory heal
    // resolved after Red Skull's settlement reset and re-armed the latch while still bloodied; the
    // stale `RedSkull: 1` counter then failed the next combat's inventory check. In the source a heal
    // only calls onNotBloodied (on crossing above half, AbstractCreature.java:441) and onBloodied
    // needs the COMBAT phase, so the latch stays cleared after onVictory.
    let mut run = a10_on_map(Character::Ironclad);
    run.state.relics.push(RelicId::RedSkull);
    run.state.hp = 20; // 20 * 2 <= 88: bloodied before, and still bloodied after Burning Blood's +6.
    enter_first(&mut run, MapNodeKind::Monster);
    super::m28_world::win(&mut run);
    assert!(run.state.combat.is_none());
    assert_eq!(run.state.hp, 26, "Burning Blood healed 6");
    assert_eq!(run.state.relic_counters.get(&RelicId::RedSkull), None, "no Red Skull latch outside combat");
    // The next combat enters.
    let rewards_done = |r: &Run| rooms::public(r).unwrap().kind == RoomViewKind::Map;
    for _ in 0..8 {
        if rewards_done(&run) { break }
        let leave = rooms::public(&run).unwrap().choices.into_iter().find(|c| c.status == RoomChoiceStatus::Offered
            && matches!(c.kind, RoomChoiceKind::Proceed | RoomChoiceKind::Leave | RoomChoiceKind::Skip)).unwrap().id;
        run.step(&GameAction::Choose(leave)).unwrap();
    }
    enter_first(&mut run, MapNodeKind::Monster);
    assert!(run.state.combat.is_some(), "the next combat enters");
}
