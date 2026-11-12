//! M32 A16 shop (plan item 8; design §6.6; lane A §2, `m32-source-nonmonster-v1.md`). Source
//! (`ShopScreen` constructor): after the stock is priced (cards truncated, one coloured card halved),
//! `ascensionLevel >= 16` runs `applyDiscount(1.1f, false)` over every card, relic and potion row,
//! then Courier `0.8f`, then Membership `0.5f`, each a separate `MathUtils.round`; purge is not
//! touched and Courier restocks are priced without the A16 pass. Expected prices are worked here
//! from the A15 Run of the same seed and actions (the Standard Run has no other A16 rule, so both
//! Runs are identical up to the first shop), never read from the code under test.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{Character, RelicId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

/// libGDX `MathUtils.round(float)`: `(int)((double)value + 16384.5) - 16384`.
fn gdx_round(value: f32) -> u32 { ((f64::from(value) + 16384.5) as i64 - 16384) as u32 }
/// One `applyDiscount(m)` pass on a stored integer price.
fn pass(price: u32, multiplier: f32) -> u32 { gdx_round(price as f32 * multiplier) }

fn rng(seed: u64) -> RunRng { RunRng::new([seed as u8; 32], Partition::Test, seed) }
fn room(run: &Run) -> RoomView { rooms::public(run).unwrap() }
fn click(run: &mut Run, id: ChoiceId) { run.step(&GameAction::Choose(id)).unwrap(); }

/// A public Standard Run past the short Neow, owning `owned`, standing in a Shop entered from its
/// first map node.
fn shop(character: Character, ascension: u8, seed: u64, owned: &[RelicId]) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile, rng(seed), RoomScenario::Standard)
        .unwrap_or_else(|error| panic!("{character:?} A{ascension}: {error:?}"));
    let choice = room(&run).choices.into_iter().find(|row| matches!(row.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
    run.step(&GameAction::Choose(choice)).unwrap();
    for relic in owned {
        if let Some(pools) = run.state.relic_pools.as_mut() { crate::items::relic_pools::remove(pools, *relic); }
        run.state.relics.push(*relic);
    }
    run.state.gold = 999;
    let target = run.decision().unwrap().candidates.iter().find_map(|candidate| match candidate.action {
        GameAction::Move(id) => Some(id), _ => None,
    }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node| node.id == target).unwrap().kind = MapNodeKind::Shop;
    run.step(&GameAction::Move(target)).unwrap();
    assert_eq!(room(&run).kind, RoomViewKind::Shop, "{character:?} A{ascension}");
    run
}

fn goods(kind: &RoomChoiceKind) -> bool {
    matches!(kind, RoomChoiceKind::Card { .. } | RoomChoiceKind::Relic(_) | RoomChoiceKind::Potion(_))
}

fn priced(run: &Run) -> Vec<(RoomChoiceKind, Option<u32>)> {
    room(run).choices.into_iter().map(|row| (row.kind, row.price)).collect()
}

/// Oracle (a): same seed and actions at A15 and A16; every goods row is `round(p15 × 1.1f)`, purge
/// equal; A17 and A19 equal A16.
#[test]
fn m32_a16_shop_prices_vs_a15() {
    for character in CHARACTERS {
        for seed in [3_u64, 0x32, 0x77, 0xB1] {
            let a15 = priced(&shop(character, 15, seed, &[]));
            let a16 = priced(&shop(character, 16, seed, &[]));
            assert_eq!(a15.len(), a16.len(), "{character:?} seed {seed}");
            let mut marked = 0;
            for ((kind15, p15), (kind16, p16)) in a15.iter().zip(&a16) {
                assert_eq!(kind15, kind16, "{character:?} seed {seed}: the stock is identical");
                if goods(kind15) {
                    assert_eq!(*p16, Some(pass(p15.unwrap(), 1.1)), "{character:?} seed {seed} {kind15:?}");
                    marked += 1;
                } else {
                    assert_eq!(p15, p16, "{character:?} seed {seed} {kind15:?}: purge and leave unchanged");
                }
            }
            assert_eq!(marked, 13, "5 coloured + 2 colourless cards, 3 relics, 3 potions");
            for ascension in [17, 19] {
                assert_eq!(priced(&shop(character, ascension, seed, &[])), a16, "{character:?} seed {seed} A{ascension}");
            }
        }
    }
}

/// Oracle (b): the pass order against the sale card, Courier, Membership and both (lane A §2 worked
/// examples at jitter 1.0, then the composed order on live stock).
#[test]
fn m32_a16_shop_discount_literals() {
    // Lane A §2 literals: Common coloured 50 → 55; on sale 50 → 25 → 28, Courier 22, Membership 14,
    // both 11; colourless Uncommon 90 → 99 → 79 / 50 / 40; relic 150 → 165 → 132 / 83 / 66; potion
    // 50 → 55 → 44 / 28 / 22.
    assert_eq!(pass(50, 1.1), 55);
    for (stocked, a16, courier, membership, both) in [(25, 28, 22, 14, 11), (90, 99, 79, 50, 40), (150, 165, 132, 83, 66), (50, 55, 44, 28, 22)] {
        assert_eq!(pass(stocked, 1.1), a16);
        assert_eq!(pass(a16, 0.8), courier);
        assert_eq!(pass(a16, 0.5), membership);
        assert_eq!(pass(pass(a16, 0.8), 0.5), both);
    }
    // Live stock: the A15 no-relic price p15 (sale already applied) becomes the composed A16 price.
    for character in CHARACTERS {
        let seed = 0x55;
        let base = priced(&shop(character, 15, seed, &[]));
        for (owned, law) in [
            (vec![RelicId::TheCourier], [0.8_f32].as_slice()),
            (vec![RelicId::MembershipCard], [0.5_f32].as_slice()),
            (vec![RelicId::TheCourier, RelicId::MembershipCard], [0.8_f32, 0.5].as_slice()),
            (vec![RelicId::SmilingMask], [].as_slice()),
        ] {
            let a15 = priced(&shop(character, 15, seed, &owned));
            let a16 = priced(&shop(character, 16, seed, &owned));
            for (((kind, p0), (_, p15)), (_, p16)) in base.iter().zip(&a15).zip(&a16) {
                if !goods(kind) {
                    assert_eq!(p15, p16, "{character:?} {owned:?} {kind:?}: purge as at A15");
                    continue;
                }
                let expected = law.iter().fold(pass(p0.unwrap(), 1.1), |price, m| pass(price, *m));
                assert_eq!(*p16, Some(expected), "{character:?} {owned:?} {kind:?}");
            }
        }
    }
}

/// Oracle (c): a Courier restock in an A16 shop is priced by the A15 law (no 1.1 pass), for a card,
/// a relic and a potion.
#[test]
fn m32_a16_courier_restock_no_markup() {
    for character in CHARACTERS {
        let seed = 0x41;
        let mut runs = [15_u8, 16].map(|ascension| shop(character, ascension, seed, &[RelicId::TheCourier]));
        for pick in [
            |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Card { .. }),
            |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Relic(_)),
            |kind: &RoomChoiceKind| matches!(kind, RoomChoiceKind::Potion(_)),
        ] {
            // spire-0kr: the restock replaces the bought row in place.
            let [a15, a16] = runs.each_mut().map(|run| {
                run.state.gold = 999;
                run.state.potions.iter_mut().for_each(|slot| *slot = None);
                let index = room(run).choices.iter().position(|row| pick(&row.kind) && row.status == RoomChoiceStatus::Offered).unwrap();
                let bought = room(run).choices[index].id;
                click(run, bought);
                let restock = room(run).choices[index].clone();
                assert_ne!(restock.id, bought, "{character:?}: restock in place");
                (restock.kind, restock.price)
            });
            assert_eq!(a15.0, a16.0, "{character:?}: same restock");
            assert_eq!(a15.1, a16.1, "{character:?}: restock priced as at A15");
        }
    }
}

/// Oracle (d): Membership Card bought inside an A16 shop halves the A16-inflated rows (the engine's
/// in-shop Courier reprice is unreachable: The Courier is never stocked; review m5).
#[test]
fn m32_a16_membership_bought_in_shop() {
    for character in CHARACTERS {
        let seed = 0x23;
        // The same seed and actions give the same choice ids, so rows are matched by id.
        let base = room(&shop(character, 15, seed, &[])).choices;
        let after = [15_u8, 16].map(|ascension| {
            let mut run = shop(character, ascension, seed, &[]);
            let row = run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut()
                .find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
            row.kind = RoomChoiceKind::Relic(RelicId::MembershipCard);
            let id = row.id;
            if let Some(pools) = run.state.relic_pools.as_mut() { crate::items::relic_pools::remove(pools, RelicId::MembershipCard); }
            click(&mut run, id);
            assert!(run.state.relics.contains(&RelicId::MembershipCard));
            (id, room(&run).choices)
        });
        let [(bought, after15), (_, after16)] = &after;
        let mut halved = 0;
        for row in base.iter().filter(|row| row.id != *bought) {
            let Some(row16) = after16.iter().find(|other| other.id == row.id) else { continue };
            let row15 = after15.iter().find(|other| other.id == row.id).unwrap();
            if goods(&row.kind) {
                assert_eq!(row16.price, Some(pass(pass(row.price.unwrap(), 1.1), 0.5)), "{character:?} {:?}", row.kind);
                halved += 1;
            } else {
                assert_eq!(row16.price, row15.price, "{character:?} {:?}: purge as at A15", row.kind);
            }
        }
        assert_eq!(halved, 12, "{character:?}: every other goods row is halved");
    }
}

/// t − 1: A15 shop prices equal A14 (no shop rule below A16) and differ from A16.
#[test]
fn m32_a15_shop_unchanged() {
    for character in CHARACTERS {
        for seed in [3_u64, 0x32] {
            let a14 = priced(&shop(character, 14, seed, &[]));
            let a15 = priced(&shop(character, 15, seed, &[]));
            assert_eq!(a14, a15, "{character:?} seed {seed}");
            assert_ne!(a15, priced(&shop(character, 16, seed, &[])), "{character:?} seed {seed}");
        }
    }
}
