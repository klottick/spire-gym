//! A28 M5 reward-owner potion, appearance, and Prismatic Shard coverage.
use super::super::*;
use crate::{
    combat::{Actor, DamagePayload, Effect, EffectKind, Outcome, Source},
    content::{CardColor, CardId, CardRarity, Character, EncounterId, PotionId, RelicId},
    items::{dispatch::RunItemEvent, lifecycle::RewardSpec, run_laws::RewardRoom},
    run::{diagnostics::{self, CompleteItemSupply, M25Diagnostic}, rooms},
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

fn enter_elite(run: &mut Run) {
    move_to_floor(run, 51);
    choose_kind(run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    move_to_floor(run, 52);
    choose_kind(run, |kind| *kind == RoomChoiceKind::Leave);
    move_to_floor(run, 53);
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, EncounterId::ShieldAndSpear);
}

fn defeat_current_combat(run: &mut Run) {
    let ids = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.into_iter()
        .filter(|enemy| enemy.hp > 0).map(|enemy| enemy.instance_id).collect::<Vec<_>>();
    for id in ids {
        run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
            source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(id)),
            kind: EffectKind::Damage(DamagePayload::Fixed(10_000)),
        }], None).unwrap();
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(Outcome::Victory));
}

fn complete_at_elite(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>, seed: u8) -> Run {
    let mut run = diagnostics::complete_act4_run(character, supply(relics, potions)).unwrap();
    enter_elite(&mut run);
    run.state.rng = RunRng::new([seed; 32], crate::combat::rng::Partition::Test, 0);
    defeat_current_combat(&mut run);
    run.settle_combat().unwrap();
    run
}

fn potion_rows(run: &Run) -> Vec<(ChoiceId, PotionId, RoomChoiceStatus)> {
    run.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|row| {
        if let RoomChoiceKind::Potion(id) = row.kind { Some((row.id, id, row.status)) } else { None }
    }).collect()
}

fn reward_cards(run: &mut Run) -> Vec<CardId> {
    choose_kind(run, |kind| *kind == RoomChoiceKind::CardReward);
    run.decision().unwrap().room.unwrap().choices.into_iter().filter_map(|row| {
        if let RoomChoiceKind::Card { id, .. } = row.kind { Some(id) } else { None }
    }).collect()
}

fn character_color(character: Character) -> CardColor {
    match character {
        Character::Ironclad => CardColor::Red,
        Character::Silent => CardColor::Green,
        Character::Defect => CardColor::Blue,
        Character::Watcher => CardColor::Purple,
    }
}

#[test]
fn m27_potion_owner_obtain_consumers_reward() {
    let sozu_before = vec![None; 3];
    let mut sozu = complete_at_elite(Character::Ironclad, vec![RelicId::Sozu], sozu_before.clone(), 132);
    let (sozu_row, _, _) = potion_rows(&sozu).into_iter().next().unwrap();
    sozu.step(&GameAction::Choose(sozu_row)).unwrap();
    assert_eq!(potion_rows(&sozu).iter().find(|(id, _, _)| *id == sozu_row).unwrap().2, RoomChoiceStatus::Claimed);
    assert_eq!(sozu.state.potions, sozu_before, "Sozu removes the claimed reward without obtaining it");

    let full = vec![Some(PotionId::FirePotion), Some(PotionId::StrengthPotion), Some(PotionId::BlockPotion)];
    let mut full_belt = complete_at_elite(Character::Ironclad, vec![], full.clone(), 132);
    let (full_row, _, _) = potion_rows(&full_belt).into_iter().next().unwrap();
    assert_eq!(full_belt.step(&GameAction::Choose(full_row)), Err(RunError::InvalidAction));
    assert_eq!(full_belt.state.potions, full, "a full belt refuses without replacing any slot");
    assert_eq!(full_belt.state.room_loop.as_ref().unwrap().screen.choices.iter()
        .find(|row| row.id == full_row).unwrap().status, RoomChoiceStatus::Offered,
        "a refused full-belt claim leaves the stored row Offered");

    let mut normal = complete_at_elite(Character::Ironclad, vec![], vec![None; 3], 132);
    let (normal_row, potion, status) = potion_rows(&normal).into_iter().next().unwrap();
    assert_eq!((potion, status), (PotionId::BlockPotion, RoomChoiceStatus::Offered));
    normal.step(&GameAction::Choose(normal_row)).unwrap();
    assert_eq!(normal.state.potions, vec![Some(PotionId::BlockPotion), None, None]);
    assert_eq!(normal.state.potion_potencies[0], Some((PotionId::BlockPotion, 12)));

    let mut bark = complete_at_elite(Character::Ironclad, vec![RelicId::SacredBark], vec![None; 3], 132);
    let (bark_row, bark_potion, _) = potion_rows(&bark).into_iter().next().unwrap();
    assert_eq!(bark_potion, PotionId::BlockPotion);
    bark.step(&GameAction::Choose(bark_row)).unwrap();
    assert_eq!(bark.state.potion_potencies[0], Some((PotionId::BlockPotion, 24)));
}

#[test]
fn m27_act4_potion_producers_under_complete_elite() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let success = complete_at_elite(character, vec![], vec![None; 3], 1);
        assert_eq!(success.state.potion_offset, -10, "{character:?}: success subtracts ten from the zero offset");
        let successful_rows = potion_rows(&success);
        assert_eq!(successful_rows.len(), 1, "{character:?}: one potion offer on success");
        assert!(crate::items::potion_generation::candidates(character, false).unwrap().contains(&successful_rows[0].1),
            "{character:?}: offer belongs to its 33-potion class pool");

        let failure = complete_at_elite(character, vec![], vec![None; 3], 0);
        assert_eq!(failure.state.potion_offset, 10, "{character:?}: failure adds ten to the zero offset");
        assert!(potion_rows(&failure).is_empty(), "{character:?}: a failed appearance publishes no potion row");

        let statue = complete_at_elite(character, vec![RelicId::WhiteBeastStatue], vec![None; 3], 0);
        assert_eq!(statue.state.potion_offset, -10, "{character:?}: White Beast Statue forces success");
        assert_eq!(potion_rows(&statue).len(), 1, "{character:?}: Statue yields a row when fewer than four rows precede it");
    }

    let mut crowded = diagnostics::complete_act4_run(Character::Ironclad,
        supply(vec![RelicId::WhiteBeastStatue], vec![None; 3])).unwrap();
    enter_elite(&mut crowded);
    crowded.state.rng = RunRng::new([1; 32], crate::combat::rng::Partition::Test, 0);
    defeat_current_combat(&mut crowded);
    let active = crowded.state.active_combat.as_ref().unwrap();
    let record = CombatRecord { visit: active.visit, encounter: active.encounter, outcome: Outcome::Victory };
    // Economy research §5.2: the four-row check precedes the auto-added card reward, so only the
    // Gold and Relic rows count. Gold + 2 Relics is three rows; Gold + 3 Relics is four.
    let building = |extra_elite_relics| RunItemEvent::RewardsBuilding { room: RewardRoom::Elite, card_choices: 3,
        extra_card_rewards: 0, gold_percent: 0, extra_elite_relics, include_potion: true, singing_bowl: false };
    let three = rooms::rewards::prepare_combat_with_building(&crowded, &record, building(1)).unwrap().unwrap();
    assert_eq!(three.screen.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).count(), 1,
        "Gold and two Relic rows are three existing rows; the card reward does not count");
    let preview = rooms::rewards::prepare_combat_with_building(&crowded, &record, building(2)).unwrap().unwrap();
    assert!(!preview.screen.choices.iter().any(|row| matches!(row.kind, RoomChoiceKind::Potion(_))),
        "four existing rows suppress appearance even with White Beast Statue");
    assert_eq!(crowded.state.potion_offset, 0, "a discarded preview does not commit the staged offset");

    let mut frozen = diagnostics::m25_diagnostic(M25Diagnostic::Act4Room { floor: 53 }).unwrap();
    defeat_current_combat(&mut frozen);
    frozen.settle_combat().unwrap();
    assert_eq!(potion_rows(&frozen).into_iter().map(|(_, potion, _)| potion).collect::<Vec<_>>(),
        vec![PotionId::EnergyPotion], "frozen M25 Act 4 keeps its accepted fixed potion row");
    assert_eq!(frozen.state.potion_offset, 0, "frozen profile does not enter the Complete appearance law");
}

#[test]
fn m27_prismatic_shard_any_colour_card_reward() {
    let any_colour = crate::content::pools::cards(crate::content::pools::CardPool::AnyColour { rarity: None }).unwrap();
    let seeds = [0, 1, 7, 18, 29, 41, 53, 89];
    let mut saw_off_colour = [false; 4];
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    for (character_index, character) in characters.into_iter().enumerate() {
        for seed in seeds {
            let mut run = complete_at_elite(character, vec![RelicId::PrismaticShard], vec![None; 3], seed);
            let cards = reward_cards(&mut run);
            assert_eq!(cards.len(), 3, "{character:?} elite reward has three cards");
            assert_eq!(cards.iter().map(|id| crate::content::Catalog::standard().card(*id).unwrap().rarity).collect::<Vec<_>>(),
                vec![CardRarity::Common, CardRarity::Uncommon, CardRarity::Uncommon]);
            for card in cards {
                assert!(any_colour.contains(&card), "{character:?} Shard reward card {card:?} is in M26's any-colour pool");
                let color = crate::content::Catalog::standard().card(card).unwrap().color;
                saw_off_colour[character_index] |= color != character_color(character);
            }
        }

        let mut own_colour = complete_at_elite(character, vec![], vec![None; 3], 18);
        let own_cards = reward_cards(&mut own_colour);
        assert_eq!(own_cards.len(), 3);
        assert!(own_cards.iter().all(|id| crate::content::Catalog::standard().card(*id).unwrap().color == character_color(character)),
            "{character:?} without Shard keeps its accepted colour sampler");
    }
    assert!(saw_off_colour.into_iter().all(|found| found), "each character sees an off-colour card across the keyed samples");

    let mut equip = diagnostics::complete_act4_run(Character::Ironclad, supply(vec![RelicId::PrismaticShard], vec![None; 3])).unwrap();
    move_to_floor(&mut equip, 51);
    let campfire_choice = equip.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| matches!(row.kind, RoomChoiceKind::Rest { .. })).unwrap().id;
    rooms::rewards::open_equip_offers(&mut equip, campfire_choice, RelicId::PrismaticShard,
        vec![RewardSpec { card_rewards: 1, ..RewardSpec::default() }], false).unwrap();
    let equip_cards = reward_cards(&mut equip);
    assert_eq!(equip_cards.len(), 3);
    assert_eq!(equip_cards.iter().map(|id| crate::content::Catalog::standard().card(*id).unwrap().rarity).collect::<Vec<_>>(),
        vec![CardRarity::Common, CardRarity::Uncommon, CardRarity::Uncommon]);
    assert!(equip_cards.iter().all(|id| any_colour.contains(id)), "A25 equip card children use M26's any-colour pool with Shard");
}
