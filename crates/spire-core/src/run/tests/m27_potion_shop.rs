//! M27 complete-admission shop potion producers and Run potion actions.
use super::super::*;
use crate::{
    combat::{Effect, EffectKind, Source},
    content::{Catalog, Character, CardRarity, PotionId, PotionRarity, RelicId},
    run::diagnostics::{self, CompleteItemSupply, M25Diagnostic},
};
use std::collections::BTreeMap;

fn supply(relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> CompleteItemSupply {
    CompleteItemSupply { relics, potions, counters: BTreeMap::new() }
}

fn move_to_floor(run: &mut Run, floor: u32) {
    let target = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == floor).unwrap().id;
    run.step(&GameAction::Move(target)).unwrap();
}

fn choose_kind(run: &mut Run, predicate: impl Fn(&RoomChoiceKind) -> bool) {
    let id = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind))
        .unwrap_or_else(|| panic!("offered room choice matching predicate")).id;
    run.step(&GameAction::Choose(id)).unwrap();
}

fn enter_shop(run: &mut Run) {
    move_to_floor(run, 51);
    choose_kind(run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    move_to_floor(run, 52);
    assert_eq!(run.decision().unwrap().room.unwrap().kind, RoomViewKind::Shop);
}

fn potion_price(id: PotionId) -> u32 {
    match Catalog::standard().potion_rarity(id).unwrap() {
        PotionRarity::Common => 50,
        PotionRarity::Uncommon => 75,
        PotionRarity::Rare => 100,
    }
}

#[test]
fn m27_potion_owner_obtain_consumers_shop() {
    let mut sozu = diagnostics::complete_act4_run(Character::Ironclad,
        supply(vec![RelicId::Sozu], vec![None, None, None])).unwrap();
    enter_shop(&mut sozu);
    let potion = sozu.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
    let before = sozu.clone();
    assert_eq!(sozu.step(&GameAction::Choose(potion.id)), Err(RunError::InvalidAction),
        "Sozu refuses before payment or stock change");
    assert_eq!(sozu, before);

    let mut full = diagnostics::complete_act4_run(Character::Ironclad,
        supply(vec![], vec![Some(PotionId::FirePotion), Some(PotionId::BlockPotion), Some(PotionId::StrengthPotion)])).unwrap();
    enter_shop(&mut full);
    let potion = full.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
    let before = full.clone();
    assert_eq!(full.step(&GameAction::Choose(potion.id)), Err(RunError::InvalidAction),
        "a full belt refuses before payment or stock change");
    assert_eq!(full, before);

    let mut normal = diagnostics::complete_act4_run(Character::Ironclad, supply(vec![], vec![None; 3])).unwrap();
    enter_shop(&mut normal);
    let potion = normal.state.room_loop.as_mut().unwrap().screen.choices.iter_mut()
        .find(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
    potion.kind = RoomChoiceKind::Potion(PotionId::BlockPotion);
    potion.price = Some(50);
    let potion_id = potion.id;
    normal.step(&GameAction::Choose(potion_id)).unwrap();
    assert_eq!(normal.state.potions[0], Some(PotionId::BlockPotion));
    assert_eq!(normal.state.potion_potencies[0], Some((PotionId::BlockPotion, 12)),
        "shop obtain initializes ordinary Block Potion potency");

    let mut combat_sozu = diagnostics::complete_act4_run(Character::Ironclad,
        supply(vec![RelicId::Sozu], vec![Some(PotionId::EntropicBrew), None, None])).unwrap();
    enter_shop(&mut combat_sozu);
    choose_kind(&mut combat_sozu, |kind| *kind == RoomChoiceKind::Leave);
    move_to_floor(&mut combat_sozu, 53);
    let before = combat_sozu.state.potions.clone();
    combat_sozu.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source: Source::Potion { slot: 0, id: PotionId::EntropicBrew }, target: None,
        kind: EffectKind::ObtainPotion { id: PotionId::BlockPotion },
    }], None).unwrap();
    assert_eq!(combat_sozu.state.potions, before, "combat ObtainPotion still checks Sozu");
}

#[test]
fn m27_act4_potion_producers_under_complete_shop() {
    for (character, expected_rows) in [
        (Character::Ironclad, 3), (Character::Silent, 1),
        (Character::Defect, 1), (Character::Watcher, 1),
    ] {
        let mut run = diagnostics::complete_act4_run(character, supply(vec![], vec![None; 3])).unwrap();
        enter_shop(&mut run);
        let rows = run.decision().unwrap().room.unwrap().choices.into_iter()
            .filter_map(|row| match row.kind {
                RoomChoiceKind::Potion(id) => Some((id, row.price)), _ => None,
            }).collect::<Vec<_>>();
        assert_eq!(rows.len(), expected_rows, "{character:?} keeps its accepted floor-52 potion row count");
        let eligible = crate::items::potion_generation::candidates(character, false).unwrap();
        for (id, price) in rows {
            assert!(eligible.contains(&id), "{character:?} shop offers only its 33-potion class pool");
            assert_eq!(price, Some(potion_price(id)), "shop price is the rarity base after ShopPrice");
        }
    }

    let frozen = diagnostics::m25_diagnostic(M25Diagnostic::Act4Room { floor: 52 }).unwrap();
    let frozen_rows = frozen.decision().unwrap().room.unwrap().choices.into_iter()
        .filter_map(|row| if let RoomChoiceKind::Potion(id) = row.kind { Some((id, row.price)) } else { None })
        .collect::<Vec<_>>();
    assert_eq!(frozen_rows, vec![(PotionId::StancePotion, Some(50))],
        "frozen M25 keeps its accepted single potion row");
}

#[test]
fn m27_courier_potion_and_card_replacement() {
    let mut run = diagnostics::complete_act4_run(Character::Ironclad,
        supply(vec![RelicId::TheCourier], vec![None; 3])).unwrap();
    enter_shop(&mut run);
    // spire-0kr: The Courier restocks the bought row in place (fresh id, same index, same row count).
    let before = run.decision().unwrap().room.unwrap().choices;
    let potion_index = before.iter()
        .position(|row| row.status == RoomChoiceStatus::Offered && matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
    run.step(&GameAction::Choose(before[potion_index].id)).unwrap();
    let room = run.decision().unwrap().room.unwrap();
    assert_eq!(room.choices.len(), before.len(), "The Courier replaces a purchased potion row");
    let replacement = &room.choices[potion_index];
    assert_ne!(replacement.id, before[potion_index].id);
    let RoomChoiceKind::Potion(id) = replacement.kind else { unreachable!() };
    let potion_replacement_price = replacement.price;
    let expected_replacement_price = Some(match Catalog::standard().potion_rarity(id).unwrap() {
        PotionRarity::Common => 40, PotionRarity::Uncommon => 60, PotionRarity::Rare => 80,
    });

    let card_index = room.choices.iter().position(|row| row.status == RoomChoiceStatus::Offered
        && matches!(row.kind, RoomChoiceKind::Card { .. })).unwrap();
    let card = room.choices[card_index].clone();
    let RoomChoiceKind::Card { id: purchased, .. } = card.kind else { unreachable!() };
    let purchased_definition = Catalog::standard().card(purchased).unwrap();
    run.step(&GameAction::Choose(card.id)).unwrap();
    let after = run.decision().unwrap().room.unwrap();
    assert_eq!(after.choices.len(), room.choices.len(), "The Courier replaces a purchased card row");
    let replacement = &after.choices[card_index];
    assert_ne!(replacement.id, card.id);
    let RoomChoiceKind::Card { id: replacement_id, .. } = replacement.kind else { unreachable!() };
    let replacement_definition = Catalog::standard().card(replacement_id).unwrap();
    assert_eq!(replacement_definition.color, purchased_definition.color, "replacement uses the purchased row's accepted color sampler");
    assert_eq!(replacement_definition.card_type, purchased_definition.card_type, "replacement keeps the purchased card type");
    let card_replacement_price = replacement.price.unwrap();
    match replacement_definition.rarity {
        CardRarity::Common => assert!((36..=44).contains(&card_replacement_price), "common card replacement applies 0.9–1.1, Courier, then one truncation"),
        CardRarity::Uncommon => assert!((54..=66).contains(&card_replacement_price), "uncommon card replacement applies 0.9–1.1, Courier, then one truncation"),
        CardRarity::Rare => assert!((108..=132).contains(&card_replacement_price), "rare card replacement applies 0.9–1.1, Courier, then one truncation"),
        rarity => panic!("merchant replacement has non-ordinary rarity {rarity:?}"),
    }
    assert_eq!(potion_replacement_price, expected_replacement_price,
        "potion replacement uses the rarity base through the Courier ShopPrice event");
}

#[test]
fn m27_potion_owner_run_discard_we_meet_again() {
    let mut run = diagnostics::complete_act4_run(Character::Ironclad,
        supply(vec![], vec![Some(PotionId::BlockPotion), None, None])).unwrap();
    enter_shop(&mut run);
    run.state.potion_potencies = vec![Some((PotionId::BlockPotion, 24)), None, None];
    run.step(&GameAction::DiscardPotion { slot: 0 }).unwrap();
    assert_eq!(run.state.potions[0], None, "Run discard empties the selected belt slot");
    assert!(run.state.potion_potencies.is_empty(), "an empty belt clears stored potion potency");

    let potion = run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut()
        .find(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).unwrap();
    potion.kind = RoomChoiceKind::Potion(PotionId::BlockPotion);
    potion.price = Some(50);
    let potion_id = potion.id;
    run.step(&GameAction::Choose(potion_id)).unwrap();
    assert_eq!(run.state.potion_potencies[0], Some((PotionId::BlockPotion, 12)),
        "a new Block Potion in the discarded slot gets fresh potency without Sacred Bark");
    run.state.potions[1] = Some(PotionId::BloodPotion);
    run.state.potion_potencies[1] = Some((PotionId::BloodPotion, 20));
    assert!(crate::items::can_use_run_potion(&run.state, 1, false, false),
        "Blood Potion is otherwise usable outside combat");

    let frozen = diagnostics::m25_diagnostic(M25Diagnostic::Act4Room { floor: 52 }).unwrap();
    assert!(!format!("{:?}", frozen.state).contains("we_meet_again"),
        "false We Meet Again stays omitted from frozen RunState Debug");

    run.state.we_meet_again = true;
    assert!(!crate::items::can_use_run_potion(&run.state, 1, false, true));
    let decision = run.decision().unwrap();
    assert!(!decision.candidates.iter().any(|candidate|matches!(candidate.action,
        GameAction::UsePotion { .. } | GameAction::DiscardPotion { .. })),
        "We Meet Again offers neither Run potion action");
    let before = run.clone();
    assert_eq!(run.step(&GameAction::UsePotion { slot: 1, target: None }), Err(RunError::InvalidAction));
    assert_eq!(run, before);
    assert_eq!(run.step(&GameAction::DiscardPotion { slot: 1 }), Err(RunError::InvalidAction));
    assert_eq!(run, before);
}
