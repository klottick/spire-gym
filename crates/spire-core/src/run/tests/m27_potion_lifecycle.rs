//! M27 plan Task P1, lane P, key m27_potion_lifecycle.
//! Run-owned use, obtain, discard, and equip-offer potion lifecycle checks.

use super::super::*;
use crate::{
    combat::CombatAction,
    content::{Character, EncounterId, PotionId, RelicId},
    items::{self, tests::m27_support},
    run::rooms::{RoomChoiceKind, RoomChoiceStatus, RoomChoiceView, RoomLoop, RoomScenario,
        RoomView, RoomViewKind},
};
use std::collections::BTreeMap;

fn fixture(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Run {
    m27_support::complete_run(character, relics, potions, BTreeMap::new(), EncounterId::JawWorm)
}

fn escape_fixture(run: &mut Run) {
    run.choose_combat(CombatAction::UsePotion { slot: 0, target: None })
        .expect("Smoke Bomb leaves the ordinary Jaw Worm combat");
    assert!(run.state().combat.is_none() && run.state().active_combat.is_none());
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome), Some(crate::combat::Outcome::Escaped));
}

fn outside_potion(character: Character, relics: Vec<RelicId>, potion: PotionId) -> Run {
    let mut run = fixture(character, relics, vec![Some(PotionId::SmokeBomb), Some(potion), None]);
    escape_fixture(&mut run);
    run
}

#[test]
fn m27_potion_lifecycle_blood_potion_outside_combat() {
    for (bark, expected_hp) in [(false, 56), (true, 72)] {
        let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
        let mut run = outside_potion(Character::Ironclad, relics, PotionId::BloodPotion);
        run.state.hp = 40;
        run.state.max_hp = 80;
        items::use_run_potion(&mut run.state, 1, false, false).unwrap();
        assert_eq!((run.state.hp, run.state.max_hp), (expected_hp, 80),
            "Blood Potion heals 20% / 40% of max HP, with no minimum-one clamp");
        assert_eq!(run.state.potions[1], None);
    }
}

#[test]
fn m27_potion_lifecycle_fruit_juice_outside_combat_all_characters() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        for (bark, expected) in [(false, (45, 85)), (true, (50, 90))] {
            let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
            let mut run = outside_potion(character, relics, PotionId::FruitJuice);
            run.state.hp = 40;
            run.state.max_hp = 80;
            items::use_run_potion(&mut run.state, 1, false, false)
                .unwrap_or_else(|error| panic!("Fruit Juice {character:?}, Bark={bark}: {error:?}"));
            assert_eq!((run.state.hp, run.state.max_hp), expected,
                "Fruit Juice first increases maximum HP, then heals by the same amount");
            assert_eq!(run.state.potions[1], None);
        }
    }
}

#[test]
fn m27_potion_lifecycle_entropic_brew_unrestricted_and_sozu_first() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = outside_potion(character, vec![], PotionId::EntropicBrew);
        items::use_run_potion(&mut run.state, 1, false, false)
            .unwrap_or_else(|error| panic!("outside Entropic Brew {character:?}: {error:?}"));
        assert_ne!(run.state.potions[1], Some(PotionId::EntropicBrew),
            "the empty slot is available for one of the Brew's unrestricted proposals for {character:?}");
        assert_eq!(run.state.potions.iter().flatten().count(), 3,
            "unrestricted outside Brew fills each remaining empty belt slot for {character:?}");
        let pool = items::potion_generation::candidates(character, false).unwrap();
        assert!(run.state.potions.iter().flatten().all(|potion| pool.contains(potion)),
            "outside proposals use the full rarity-first class pool for {character:?}");

        let mut sozu = outside_potion(character, vec![RelicId::Sozu], PotionId::EntropicBrew);
        let rng = sozu.state.rng.clone();
        items::use_run_potion(&mut sozu.state, 1, false, false)
            .unwrap_or_else(|error| panic!("Sozu outside Entropic Brew {character:?}: {error:?}"));
        assert_eq!(sozu.state.potions, [None, None, None], "Sozu consumes Brew without obtaining for {character:?}");
        assert_eq!(sozu.state.rng, rng, "Sozu is checked before any rarity or proposal draw for {character:?}");
    }
}

#[test]
fn m27_potion_lifecycle_obtains_first_empty_slot() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = fixture(character, vec![RelicId::PotionBelt], vec![
            Some(PotionId::SmokeBomb), Some(PotionId::BlockPotion), Some(PotionId::StrengthPotion), None, None,
        ]);
        escape_fixture(&mut run);
        // Refill the escaped Smoke Bomb slot, leaving slot 3 as the first empty one.
        assert!(items::obtain_run_potion(&mut run.state, PotionId::EnergyPotion).unwrap());
        assert_eq!(run.state.potions, [Some(PotionId::EnergyPotion), Some(PotionId::BlockPotion),
            Some(PotionId::StrengthPotion), None, None]);
        assert!(items::obtain_run_potion(&mut run.state, PotionId::FruitJuice).unwrap());
        assert_eq!(run.state.potions[3], Some(PotionId::FruitJuice), "first empty belt slot wins for {character:?}");
        assert_eq!(run.state.potions[4], None);
        assert_eq!(run.state.potion_potencies[3], Some((PotionId::FruitJuice, 5)));
    }
}

fn install_discard_room(run: &mut Run) {
    let node = NodeId(1);
    run.state.act = 1;
    run.state.floor = 1;
    run.state.map = Some(RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![MapNode { id: node, floor: 1, label: "M27 potion discard fixture", edges: vec![],
            x: 0, y: 0, kind: MapNodeKind::Shop, hidden_kind: None, visited: true, revealed: true,
            encounter: None, burning: false }],
        ..RunMap::default()
    });
    let scenario = run.state.room_loop.as_ref().map_or(RoomScenario::M5Act1Captured, |room| room.scenario);
    run.state.room_loop = Some(RoomLoop {
        scenario, act1: None, act2: None, act3: None, act4: None, m10_elite_relic_cursor: 0,
        screen: RoomView { kind: RoomViewKind::Shop, choices: vec![RoomChoiceView {
            id: ChoiceId(91), kind: RoomChoiceKind::Leave, price: None, status: RoomChoiceStatus::Offered,
        }] },
        purge_cost: 75, reward_parent: None, active_rewards: None, boss: None, transition: None,
        act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0,
        act3_event: None, act3_end: None,
    });
}

#[test]
fn m27_potion_lifecycle_discard_clears_combat_and_run_potency() {
    let mut in_combat = fixture(Character::Ironclad, vec![], vec![Some(PotionId::BlockPotion), None, None]);
    let combat = in_combat.state().combat.as_ref().unwrap();
    assert_eq!(combat.item_context().unwrap().potion_potency(0), Some(12));
    in_combat.choose_combat(CombatAction::DiscardPotion { slot: 0 }).unwrap();
    let combat = in_combat.state().combat.as_ref().unwrap();
    assert_eq!(combat.item_context().unwrap().potions()[0], None);
    assert_eq!(combat.item_context().unwrap().potion_potency(0), None,
        "combat discard clears the held slot's potency");

    let mut outside = fixture(Character::Ironclad, vec![], vec![
        Some(PotionId::SmokeBomb), Some(PotionId::BlockPotion), None,
    ]);
    escape_fixture(&mut outside);
    assert_eq!(outside.state.potion_potencies[1], Some((PotionId::BlockPotion, 12)));
    install_discard_room(&mut outside);
    outside.step(&GameAction::DiscardPotion { slot: 1 }).unwrap();
    assert_eq!(outside.state.potions[1], None);
    assert!(outside.state.potion_potencies.is_empty(), "Run discard clears potency when the belt is empty");

    outside.state.room_loop = None;
    outside.state.map = None;
    m27_support::next_complete_combat(&mut outside, EncounterId::JawWorm)
        .expect("the shared support re-enters a completed Run combat");
    assert!(outside.state.combat.as_ref().unwrap().item_context().unwrap().potions().iter().all(Option::is_none));
}

fn install_relic_reward(run: &mut Run, relic: RelicId) -> ChoiceId {
    let node = NodeId(1);
    run.state.act = 1;
    run.state.floor = 1;
    run.state.map = Some(RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![MapNode { id: node, floor: 1, label: "M27 equip potion fixture", edges: vec![],
            x: 0, y: 0, kind: MapNodeKind::Monster, hidden_kind: None, visited: true, revealed: true,
            encounter: None, burning: false }],
        ..RunMap::default()
    });
    let relic_choice = ChoiceId(100);
    run.state.room_loop = Some(RoomLoop {
        scenario: RoomScenario::M5Act1Captured, act1: None, act2: None, act3: None, act4: None,
        m10_elite_relic_cursor: 0,
        screen: RoomView { kind: RoomViewKind::Rewards, choices: vec![
            RoomChoiceView { id: relic_choice, kind: RoomChoiceKind::Relic(relic), price: None,
                status: RoomChoiceStatus::Offered },
            RoomChoiceView { id: ChoiceId(101), kind: RoomChoiceKind::Proceed, price: None,
                status: RoomChoiceStatus::Offered },
        ] },
        purge_cost: 75, reward_parent: None, active_rewards: None, boss: None, transition: None,
        act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0,
        act3_event: None, act3_end: None,
    });
    run.next_choice = 102;
    relic_choice
}

fn claim_potion_rows_from_relic(run: &mut Run, relic: RelicId, sozu: bool) {
    let relic_choice = install_relic_reward(run, relic);
    run.step(&GameAction::Choose(relic_choice))
        .unwrap_or_else(|error| panic!("claim {relic:?} through reward owner: {error:?}"));
    assert!(run.state.relics.contains(&relic));
    let potion_rows = run.state.room_loop.as_ref().unwrap().screen.choices.iter()
        .filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_)))
        .map(|row| row.id).collect::<Vec<_>>();
    assert!(!potion_rows.is_empty(), "{relic:?} opens its potion offers through the reward owner");
    let before = run.state.potions.clone();
    if sozu {
        for id in potion_rows {
            run.step(&GameAction::Choose(id)).unwrap_or_else(|error|
                panic!("Sozu claim from {relic:?} offer: {error:?}"));
            assert_eq!(run.state.potions, before, "Sozu removes the offer without obtaining its potion");
            assert_eq!(run.state.room_loop.as_ref().unwrap().screen.choices.iter()
                .find(|row| row.id == id).unwrap().status, RoomChoiceStatus::Claimed);
        }
    } else {
        let id = potion_rows[0];
        assert_eq!(run.step(&GameAction::Choose(id)), Err(RunError::InvalidAction),
            "a full belt refuses the potion offer");
        assert_eq!(run.state.potions, before);
        assert_eq!(run.state.room_loop.as_ref().unwrap().screen.choices.iter()
            .find(|row| row.id == id).unwrap().status, RoomChoiceStatus::Offered,
            "the declined full-belt row remains Offered in stored rewards");
    }
}

#[test]
fn m27_potion_lifecycle_cauldron_and_tiny_house_reward_owner_sozu_and_full_belt() {
    for relic in [RelicId::Cauldron, RelicId::TinyHouse] {
        let mut sozu = fixture(Character::Ironclad, vec![RelicId::Sozu],
            vec![Some(PotionId::SmokeBomb), None, None]);
        escape_fixture(&mut sozu);
        claim_potion_rows_from_relic(&mut sozu, relic, true);

        let mut full = fixture(Character::Ironclad, vec![RelicId::PotionBelt], vec![
            Some(PotionId::SmokeBomb), Some(PotionId::BlockPotion), Some(PotionId::EnergyPotion),
            Some(PotionId::StrengthPotion), Some(PotionId::FruitJuice),
        ]);
        escape_fixture(&mut full);
        assert!(items::obtain_run_potion(&mut full.state, PotionId::FirePotion).unwrap());
        assert_eq!(full.state.potions.iter().filter(|slot| slot.is_some()).count(), 5);
        claim_potion_rows_from_relic(&mut full, relic, false);
    }
}

#[test]
fn m27_potion_lifecycle_we_meet_again_uses_run_fact() {
    for (character, potion) in [
        (Character::Ironclad, PotionId::BloodPotion),
        (Character::Ironclad, PotionId::FruitJuice),
        (Character::Silent, PotionId::FruitJuice),
        (Character::Defect, PotionId::FruitJuice),
        (Character::Watcher, PotionId::FruitJuice),
        (Character::Ironclad, PotionId::EntropicBrew),
        (Character::Silent, PotionId::EntropicBrew),
        (Character::Defect, PotionId::EntropicBrew),
        (Character::Watcher, PotionId::EntropicBrew),
    ] {
        let mut run = outside_potion(character, vec![], potion);
        assert!(items::can_use_run_potion(&run.state, 1, false, false),
            "{potion:?} is otherwise usable outside combat for {character:?}");
        run.state.we_meet_again = true;
        assert!(!items::can_use_run_potion(&run.state, 1, false, run.state.we_meet_again),
            "the RunState We Meet Again fact removes the use action for {character:?}");
        let before = run.state.clone();
        assert_eq!(items::use_run_potion(&mut run.state, 1, false, true), Err(RunError::InvalidAction));
        assert_eq!(run.state, before, "rejected We Meet Again use is inert for {character:?}");
    }
}

#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_review_m5_outside_entropic_brew_draws_per_live_slot() {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    let mut full = diagnostics::complete_act4_run(Character::Ironclad, CompleteItemSupply {
        relics: vec![],
        potions: vec![Some(PotionId::EntropicBrew), Some(PotionId::BlockPotion), Some(PotionId::StrengthPotion)],
        counters: BTreeMap::new(),
    }).unwrap();
    let mut sparse = diagnostics::complete_act4_run(Character::Ironclad, CompleteItemSupply {
        relics: vec![],
        potions: vec![Some(PotionId::EntropicBrew), None, None],
        counters: BTreeMap::new(),
    }).unwrap();
    assert_eq!(full.state.floor, sparse.state.floor, "both complete Runs start at the same floor");
    full.state.floor = 51;
    sparse.state.floor = 51;
    assert_eq!(full.state.rng, sparse.state.rng, "the complete Runs use the same seed and initial RNG state");
    items::use_run_potion(&mut full.state, 0, false, false).unwrap();
    items::use_run_potion(&mut sparse.state, 0, false, false).unwrap();
    assert_eq!(full.state.rng, sparse.state.rng,
        "outside Entropic Brew draws one proposal for each of three live slots, including full slots");
}
