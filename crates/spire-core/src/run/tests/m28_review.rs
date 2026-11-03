//! M28b review-v1 regressions (findings m1, m2 and m4; the reviewer's probes
//! `m28b-review-v1/probe/m28b_review_probe.rs`, adopted). Constructed states; the upgrade-draw
//! law is the PM source check m28b/source-check-act4-upgrade-v1.md, F3b.
use super::super::*;
use super::m28_world::on_map;
use crate::combat::rng::Partition;
use crate::content::{CardRarity, CardType, Catalog, Character, RelicId};
use crate::run::rooms::standard::rewards::{self, ordinary_offer, RarityRoom};
use crate::run::rooms::standard::shop::{diagnostic_shop, ShopSupply};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
const EXHAUSTED: RunError = RunError::Unsupported("run scoped RNG exhausted");

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }

/// Saturate the upgrade-roll ticket of every slot, so any draw on it errors.
fn saturate_upgrade_tickets(run: &mut Run, key: &[u64]) {
    for slot in 0..3_u64 {
        let mut ticket = key.to_vec(); ticket.extend([0x0F, slot]);
        run.state.rng.set_counter_for_test(RunRngScope::M28CardRarity, &ticket, u64::MAX);
    }
}

/// A standard Run in `act` with forced Common rewards (offset 100), optionally saturated.
fn commons(act: u8, relics: &[RelicId], key: &[u64], saturated: bool) -> Run {
    let mut run = on_map(Character::Ironclad, 7);
    run.state.act = act; run.state.card_rarity_offset = Some(100);
    run.state.relics.extend_from_slice(relics);
    if saturated { saturate_upgrade_tickets(&mut run, key) }
    run
}

/// m1: every Common and Uncommon reward card consumes its upgrade draw, even at chance 0 (Act 1);
/// a Rare draws nothing.
#[test]
fn m28_review_upgrade_roll_draws_for_every_non_rare() {
    let key = [0xAB_u64];
    for act in 1..=4 {
        let mut run = commons(act, &[], &key, true);
        assert_eq!(ordinary_offer(&mut run.state, RarityRoom::Ordinary, 3, 1, &key).err(), Some(EXHAUSTED),
            "Act {act}: a non-Rare draws its upgrade roll");
        let mut run = commons(act, &[], &key, false);
        let offers = ordinary_offer(&mut run.state, RarityRoom::Ordinary, 3, 1, &key).unwrap();
        assert!(offers.iter().all(|(id, _)| Catalog::standard().card(*id).unwrap().rarity == CardRarity::Common), "Act {act}");
        if act == 1 { assert!(offers.iter().all(|(_, upgrades)| *upgrades == 0), "Act 1: chance 0 never upgrades") }
    }
    for act in [1, 3, 4] {
        let mut boss = on_map(Character::Ironclad, 7);
        boss.state.act = act;
        saturate_upgrade_tickets(&mut boss, &key);
        let offers = ordinary_offer(&mut boss.state, RarityRoom::Boss, 3, 1, &key).unwrap();
        assert!(offers.iter().all(|(_, upgrades)| *upgrades == 0), "Act {act}: boss Rares are never auto-upgraded");
    }
    // A Rare in an ordinary offer (offset 0, source rarity multiplier) also draws nothing.
    let mut rare = on_map(Character::Ironclad, 7);
    rare.state.act = 3; rare.state.card_rarity_offset = Some(0);
    saturate_upgrade_tickets(&mut rare, &key);
    let offers = ordinary_offer(&mut rare.state, RarityRoom::Ordinary, 3, 100, &key).unwrap();
    assert!(offers.iter().all(|(id, upgrades)| Catalog::standard().card(*id).unwrap().rarity == CardRarity::Rare && *upgrades == 0));
}

/// m2: the eggs apply after the automatic roll. They never skip the draw, and they preview
/// Rares (which skip the roll) and non-upgraded Commons alike.
#[test]
fn m28_review_eggs_apply_after_the_upgrade_roll() {
    let key = [0xAC_u64];
    let eggs = [RelicId::MoltenEgg, RelicId::ToxicEgg, RelicId::FrozenEgg];
    for act in [1, 2] {
        let mut run = commons(act, &eggs, &key, true);
        assert_eq!(ordinary_offer(&mut run.state, RarityRoom::Ordinary, 3, 1, &key).err(), Some(EXHAUSTED),
            "Act {act}: owning every egg does not skip the upgrade draw");
    }
    let mut act1 = commons(1, &eggs, &key, false);
    let offers = ordinary_offer(&mut act1.state, RarityRoom::Ordinary, 3, 1, &key).unwrap();
    for (id, upgrades) in offers {
        assert_eq!(upgrades, rewards::preview_upgrades(&act1.state, id, 0), "Act 1 {id:?}: no auto upgrade, egg preview");
        assert_eq!(upgrades, 1, "Act 1 {id:?}: every typed Common is previewed by its egg");
    }
    let mut boss = on_map(Character::Ironclad, 7);
    boss.state.act = 3; boss.state.relics.push(RelicId::MoltenEgg);
    saturate_upgrade_tickets(&mut boss, &key);
    let offers = ordinary_offer(&mut boss.state, RarityRoom::Boss, 3, 1, &key).unwrap();
    for (id, upgrades) in offers {
        let attack = Catalog::standard().card(id).unwrap().card_type == CardType::Attack;
        assert_eq!(upgrades, rewards::preview_upgrades(&boss.state, id, 0), "boss {id:?}");
        if !attack { assert_eq!(upgrades, 0, "boss {id:?}: Molten Egg previews Attacks only") }
    }
}

/// m3: an Event node and a question room that rolls an event (Tiny Chest counter and ramps
/// present) whose selected event is rejected up front return the typed error and leave the whole
/// Run unchanged, choice tokens and event pools included. M28f lands the last body, so the witness
/// is the remaining typed rejection: the Colosseum above A0 (no enemy facts; review-v1 M2 of M28e),
/// fixed by a one-event pool.
#[test]
fn m28_review_event_interim_leaves_the_whole_run() {
    // The diagnostic Run on a row-7 node whose row-8 child is the event room (E10: Colosseum needs y >= 8).
    fn row8(character: Character, seed: u8) -> (Run, crate::run::NodeId) {
        let (mut run, _) = super::m28_event_core::before_event(character, 2, crate::content::EventId::Colosseum, seed);
        let map = run.state.map.as_mut().unwrap();
        let from = map.nodes.iter().find(|n| n.y == 7 && !n.edges.is_empty()).unwrap().clone();
        let node = from.edges[0];
        map.nodes.iter_mut().find(|n| n.id == node).unwrap().kind = MapNodeKind::Event;
        map.current = Some(from.id);
        run.state.floor = from.floor;
        (run, node)
    }
    const INTERIM: &str = "event combat enemy setup unavailable at this ascension";
    let pools = || Some(rooms::standard::event_pool::EventPools { ordinary: vec![crate::content::EventId::Colosseum], shrines: vec![], one_time: vec![], seen: vec![] });
    for character in CHARACTERS {
        let (mut run, target) = row8(character, 9);
        run.state.event_pools = pools();
        run.state.ascension = 15;
        let before = run.clone();
        assert_eq!(run.step(&GameAction::Move(target)), Err(RunError::Unsupported(INTERIM)));
        assert!(run == before, "{character:?}: whole Run unchanged");
        let (mut question, target) = row8(character, 10);
        question.state.event_pools = pools();
        question.state.ascension = 15;
        question.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = MapNodeKind::Question;
        question.state.map.as_mut().unwrap().question_ramps = [0; 3];
        question.state.relics.push(RelicId::TinyChest); question.state.relic_counters.insert(RelicId::TinyChest, 1);
        let before = question.clone();
        assert_eq!(question.step(&GameAction::Move(target)), Err(RunError::Unsupported(INTERIM)));
        assert!(question == before, "{character:?}: question→event leaves the whole Run unchanged");
    }
}

fn shop_supply(relics: Vec<RelicId>) -> ShopSupply {
    ShopSupply { gold: 1_000, deck: None, relics, potions: Some(vec![None; 3]), purge_cost: 75 }
}

/// m2: the Courier's card restock row is previewed by the matching egg.
#[test]
fn m28_review_courier_card_restock_previews_eggs() {
    for character in CHARACTERS {
        let mut run = diagnostic_shop(character, shop_supply(vec![RelicId::TheCourier, RelicId::MoltenEgg]), rng(0x61)).unwrap();
        let attack = rooms::public(&run).unwrap().choices.into_iter().find(|row| matches!(row.kind,
            RoomChoiceKind::Card { id, .. } if Catalog::standard().card(id).unwrap().card_type == CardType::Attack)).unwrap();
        let before = rooms::public(&run).unwrap().choices.len();
        run.step(&GameAction::Choose(attack.id)).unwrap();
        let rows = rooms::public(&run).unwrap().choices;
        assert_eq!(rows.len(), before + 1, "{character:?}: Courier appends a card row");
        let RoomChoiceKind::Card { id, upgrades } = rows.last().unwrap().kind else { panic!("{character:?}: card restock row") };
        assert_eq!(Catalog::standard().card(id).unwrap().card_type, CardType::Attack, "{character:?}");
        assert_eq!(upgrades, 1, "{character:?} {id:?}: Molten Egg previews the restocked Attack");
    }
}

/// m4: G9 disables Sozu potion rows, so `Run::step` rejects them at the candidate filter. The
/// `shop::choose` Sozu guard stays covered by a direct call, atomically.
#[test]
fn m28_review_shop_choose_keeps_the_sozu_guard() {
    for character in CHARACTERS {
        let mut run = diagnostic_shop(character, shop_supply(vec![RelicId::Sozu]), rng(0x62)).unwrap();
        let potion = rooms::public(&run).unwrap().choices.into_iter().find(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
        assert_eq!(potion.status, RoomChoiceStatus::Disabled, "{character:?}");
        let before = run.clone();
        assert_eq!(rooms::shop::choose(&mut run, potion.id).err(), Some(RunError::InvalidAction), "{character:?}");
        assert!(run == before, "{character:?}: the guard refuses before payment or stock change");
    }
}

/// The M27 complete-items Act 4 sites of the shared G6, G8 and G9 fixes.
#[cfg(feature = "m27-diagnostics")]
mod m27_act4 {
    use super::*;
    use crate::run::diagnostics::{self, CompleteItemSupply};
    use std::collections::BTreeMap;

    fn supply(relics: Vec<RelicId>) -> CompleteItemSupply { CompleteItemSupply { relics, potions: vec![None; 3], counters: BTreeMap::new() } }
    fn move_to_floor(run: &mut Run, floor: u32) -> Result<RunTransition, RunError> {
        let target = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == floor).unwrap().id;
        run.step(&GameAction::Move(target))
    }
    fn rest_then_shop(run: &mut Run) {
        move_to_floor(run, 51).unwrap();
        let rest = rooms::public(run).unwrap().choices.iter().find(|c| matches!(c.kind, RoomChoiceKind::Rest { .. })).unwrap().id;
        run.step(&GameAction::Choose(rest)).unwrap();
        move_to_floor(run, 52).unwrap();
        assert_eq!(rooms::public(run).unwrap().kind, RoomViewKind::Shop);
    }

    /// m2/G8 at `act4.rs` (the fuzz finding's site): an unusable floor-51 campfire completes.
    #[test]
    fn m28_review_g8_m27_act4_campfire() {
        for character in CHARACTERS {
            let mut run = diagnostics::complete_act4_run(character, supply(vec![RelicId::CoffeeDripper, RelicId::FusionHammer])).unwrap();
            move_to_floor(&mut run, 51).unwrap();
            assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map, "{character:?}: unusable Act 4 campfire completes");
            assert!(run.decision().unwrap().candidates.iter().any(|c| matches!(c.action, GameAction::Move(_))), "{character:?}");
            move_to_floor(&mut run, 52).unwrap();
            assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Shop, "{character:?}: route continues");
            let mut usable = diagnostics::complete_act4_run(character, supply(vec![RelicId::FusionHammer])).unwrap();
            move_to_floor(&mut usable, 51).unwrap();
            assert_eq!(rooms::public(&usable).unwrap().kind, RoomViewKind::Campfire, "{character:?}: a usable campfire stays");
        }
    }

    /// m2/G6 on M27's complete Act 4 shop: Molten Egg previews Attack rows only.
    #[test]
    fn m28_review_g6_m27_act4_shop() {
        for character in CHARACTERS {
            let mut run = diagnostics::complete_act4_run(character, supply(vec![RelicId::MoltenEgg])).unwrap();
            rest_then_shop(&mut run);
            let mut attacks = 0;
            for row in rooms::public(&run).unwrap().choices {
                let RoomChoiceKind::Card { id, upgrades } = row.kind else { continue };
                if Catalog::standard().card(id).unwrap().card_type == CardType::Attack {
                    attacks += 1; assert_eq!(upgrades, 1, "{character:?} {id:?}");
                } else { assert_eq!(upgrades, 0, "{character:?} {id:?}") }
            }
            assert!(attacks > 0, "{character:?}: the Act 4 shop has Attack rows");
        }
    }

    /// m4: the M27 Sozu shop assertion now meets the G9 candidate filter; the direct call still
    /// reaches the `shop::choose` guard on M27's Act 4 shop.
    #[test]
    fn m28_review_m27_act4_sozu_guard() {
        for character in CHARACTERS {
            let mut run = diagnostics::complete_act4_run(character, supply(vec![RelicId::Sozu])).unwrap();
            rest_then_shop(&mut run);
            let potion = rooms::public(&run).unwrap().choices.into_iter().find(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
            assert_eq!(potion.status, RoomChoiceStatus::Disabled, "{character:?}");
            let before = run.clone();
            assert_eq!(run.step(&GameAction::Choose(potion.id)), Err(RunError::InvalidAction), "{character:?}: candidate filter");
            assert!(run == before, "{character:?}");
            assert_eq!(rooms::shop::choose(&mut run, potion.id).err(), Some(RunError::InvalidAction), "{character:?}: guard");
            assert!(run == before, "{character:?}: the guard refuses before payment or stock change");
        }
    }
}
