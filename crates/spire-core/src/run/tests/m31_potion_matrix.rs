//! M31 A11 potion-capacity matrix (plan item 2, design §6.3–§7).
//! Literals and source locators below are from m31-source-check-v1.md §B.
use super::super::*;
use super::m31_potion_slots::{enter_first, on_map};
use crate::combat::{Actor, CombatAction, DamagePayload, Effect, EffectKind, Source};
use crate::content::{CardId, Character, EventId, PotionId, RelicId};
use crate::items::lifecycle::RelicSource;

fn room(run: &Run) -> RoomView {
    run.decision().unwrap().room.unwrap()
}

fn potion_rows(run: &Run) -> Vec<(ChoiceId, PotionId, RoomChoiceStatus)> {
    room(run).choices.into_iter().filter_map(|row| match row.kind {
        RoomChoiceKind::Potion(potion) => Some((row.id, potion, row.status)),
        _ => None,
    }).collect()
}

fn own_plain_relic(run: &mut Run, relic: RelicId) {
    if let Some(pools) = run.state.relic_pools.as_mut() {
        crate::items::relic_pools::remove(pools, relic);
    }
    run.state.relics.push(relic);
}

fn a11_belt_run(character: Character, potions: [Option<PotionId>; 4]) -> Run {
    let mut run = on_map(character, 11);
    crate::items::obtain_relic(&mut run.state, RelicId::PotionBelt, RelicSource::Reward).unwrap();
    assert_eq!(run.state.potions.len(), 4, "A11 Potion Belt gives the Standard Run four slots");
    run.state.potions = potions.to_vec();
    run
}

fn combat_potions(run: &Run) -> Vec<Option<PotionId>> {
    run.state.combat.as_ref().unwrap().item_context().unwrap().potions().to_vec()
}

fn obtain_in_combat(run: &mut Run, id: PotionId) {
    run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source: Source::Potion { slot: 0, id: PotionId::EntropicBrew },
        target: None,
        kind: EffectKind::ObtainPotion { id },
    }], None).unwrap();
}

fn set_only_alchemize_in_deck(run: &mut Run) {
    let instance = run.state.next_instance;
    run.state.deck = vec![CardInstance { misc: 0, instance, id: CardId::Alchemize, upgrades: 0 }];
    run.state.next_instance = instance + 1;
}

fn play_alchemize(run: &mut Run) {
    let decision = run.decision().unwrap();
    let instance = decision.combat.as_ref().unwrap().hand.iter()
        .find(|card| card.id == CardId::Alchemize)
        .unwrap_or_else(|| panic!("Alchemize is in the opening hand: {:?}", decision.combat.as_ref().unwrap().hand))
        .instance.0;
    let action = GameAction::Combat(CombatAction::PlayCard { instance, target: None });
    assert!(decision.candidates.iter().any(|candidate| candidate.action == action), "Alchemize is playable");
    run.step(&action).unwrap();
}

fn use_combat_potion(run: &mut Run, slot: usize) {
    let action = GameAction::Combat(CombatAction::UsePotion { slot, target: None });
    assert!(run.decision().unwrap().candidates.iter().any(|candidate| candidate.action == action),
        "potion slot {slot} is usable");
    run.step(&action).unwrap();
}

fn assert_combat_brew_samples(run: &mut Run, count: usize) {
    use_combat_potion(run, 0);
    assert_eq!(combat_potions(run).iter().flatten().count(), count,
        "Entropic Brew fills one proposal into each available slot after its own slot is consumed");
}

fn expected_outside_brew_rng(run: &Run, count: usize) -> RunRng {
    let mut expected = run.state.clone();
    let character = expected.character;
    let semantic = [u64::from(expected.floor), 0];
    crate::items::potion_generation::sample_many_for_run(&mut expected, character, false, count, &semantic).unwrap();
    expected.rng
}

fn assert_outside_brew_samples(run: &mut Run, count: usize) {
    let expected = expected_outside_brew_rng(run, count);
    crate::items::use_run_potion(&mut run.state, 0, false, false).unwrap();
    assert_eq!(run.state.rng, expected, "outside Entropic Brew samples {count} proposals");
    assert_eq!(run.state.potions.iter().flatten().count(), count,
        "one proposal fills each live slot after the Brew is consumed");
}

fn enter_lab(run: &mut Run) {
    let pools = run.state.event_pools.as_mut().unwrap();
    pools.ordinary.clear();
    pools.shrines.clear();
    pools.one_time = vec![EventId::Lab];
    pools.seen.clear();
    enter_first(run, MapNodeKind::Event);
    assert_eq!(room(run).kind, RoomViewKind::Event(EventId::Lab), "the public Standard move enters Lab");
}

fn take_lab_potions(run: &mut Run) {
    let id = room(run).choices.into_iter()
        .find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option: 0 })
        .expect("Lab take-potions choice").id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert_eq!(room(run).kind, RoomViewKind::Rewards);
}

fn enter_shop(run: &mut Run) {
    enter_first(run, MapNodeKind::Shop);
    assert_eq!(room(run).kind, RoomViewKind::Shop);
}

fn lethal_combat_hp_loss(run: &mut Run) {
    run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source: Source::None,
        target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::HpLoss(999)),
    }], None).unwrap();
}

fn assert_fairy_rescued(run: &Run, slot: usize) {
    let combat = run.state.combat.as_ref().expect("Fairy leaves the combat alive");
    assert!(combat.public_full().unwrap().player.hp > 0, "Fairy prevents death");
    assert_eq!(combat.item_context().unwrap().potions()[slot], None, "the rescuing Fairy is consumed from its slot");
}

#[test]
// Source: m31-source-check-v1.md §B, AbstractPlayer.java:235,292–307; PotionBelt.java:24–29;
// AbstractPlayer.java:2135–2155; EntropicBrew.java:48–75; Alchemize.java:23–28.
// Grade: Combat
fn m31_a11_belt_generated_potions_fill_four() {
    let mut run = a11_belt_run(Character::Silent, [Some(PotionId::EntropicBrew), Some(PotionId::FirePotion), None, None]);
    set_only_alchemize_in_deck(&mut run);
    enter_first(&mut run, MapNodeKind::Monster);
    assert_eq!(combat_potions(&run).len(), 4);

    obtain_in_combat(&mut run, PotionId::EnergyPotion);
    assert_eq!(combat_potions(&run)[2], Some(PotionId::EnergyPotion), "in-combat obtain fills slot 2");
    play_alchemize(&mut run);
    assert!(combat_potions(&run)[3].is_some(), "Alchemize fills slot 3");
    let full = combat_potions(&run);
    obtain_in_combat(&mut run, PotionId::SwiftPotion);
    assert_eq!(combat_potions(&run), full, "the fifth generated potion is lost at capacity four");

    let mut in_combat = a11_belt_run(Character::Ironclad, [Some(PotionId::EntropicBrew), None, None, None]);
    enter_first(&mut in_combat, MapNodeKind::Monster);
    assert_combat_brew_samples(&mut in_combat, 4);

    let mut out_of_combat = a11_belt_run(Character::Ironclad, [Some(PotionId::EntropicBrew), None, None, None]);
    assert_outside_brew_samples(&mut out_of_combat, 4);
}

#[test]
// Source: m31-source-check-v1.md §B, RewardItem.java:299–316; StorePotion.java:74–99;
// Lab.java:50–64; AbstractPlayer.java:2135–2155.
// Grade: Run
fn m31_a11_belt_rows_disabled_only_when_four_full() {
    for (filled, expected) in [
        ([Some(PotionId::BlockPotion), Some(PotionId::FirePotion), Some(PotionId::SwiftPotion), None], RoomChoiceStatus::Offered),
        ([Some(PotionId::BlockPotion), Some(PotionId::FirePotion), Some(PotionId::SwiftPotion), Some(PotionId::StrengthPotion)], RoomChoiceStatus::Disabled),
    ] {
        let mut reward = a11_belt_run(Character::Ironclad, filled);
        own_plain_relic(&mut reward, RelicId::WhiteBeastStatue);
        enter_first(&mut reward, MapNodeKind::Monster);
        super::m28_world::win(&mut reward);
        let rewards = potion_rows(&reward);
        assert_eq!(rewards.len(), 1, "White Beast Statue guarantees the normal combat potion row");
        assert_eq!(rewards[0].2, expected, "combat reward status with {filled:?}");

        let mut shop = a11_belt_run(Character::Ironclad, filled);
        enter_shop(&mut shop);
        let shop_rows = potion_rows(&shop);
        assert!(!shop_rows.is_empty(), "Standard shop has potion rows");
        assert!(shop_rows.iter().all(|(_, _, status)| *status == expected), "shop statuses with {filled:?}");

        let mut lab = a11_belt_run(Character::Ironclad, filled);
        enter_lab(&mut lab);
        take_lab_potions(&mut lab);
        let lab_rows = potion_rows(&lab);
        assert_eq!(lab_rows.len(), 3, "A11 Lab offers three rows before A15");
        assert!(lab_rows.iter().all(|(_, _, status)| *status == expected), "Lab statuses with {filled:?}");
    }
}

#[test]
// Source: m31-source-check-v1.md §B, AbstractPlayer.java:2135–2155 and PotionBelt.java:24–29;
// Fairy's carried-slot rescue is the shared M27 potion lifecycle.
// Grade: Combat
fn m31_a11_belt_fairy_rescue_from_slot_three() {
    let mut run = a11_belt_run(Character::Ironclad, [None, None, None, Some(PotionId::FairyPotion)]);
    run.state.hp = 1;
    enter_first(&mut run, MapNodeKind::Monster);
    assert_eq!(combat_potions(&run), [None, None, None, Some(PotionId::FairyPotion)]);
    lethal_combat_hp_loss(&mut run);
    assert_fairy_rescued(&run, 3);
}

#[test]
// Source: m31-source-check-v1.md §B, AbstractPlayer.java:2074–2105 and 2135–2155;
// Fairy's carried-slot rescue is the shared M27 potion lifecycle.
// Grade: Combat
fn m31_lost_belt_fairy_rescue_kept_slot() {
    for (ascension, slot) in [(0_u8, 4_usize), (11, 3)] {
        let mut run = on_map(Character::Ironclad, ascension);
        crate::items::obtain_relic(&mut run.state, RelicId::PotionBelt, RelicSource::Reward).unwrap();
        run.state.potions[slot] = Some(PotionId::FairyPotion);
        crate::items::lose_relic(&mut run.state, RelicId::PotionBelt).unwrap();
        assert_eq!(run.state.potions.len(), slot + 1, "A{ascension} loss retains the added slots");
        assert!(!run.state.relics.contains(&RelicId::PotionBelt));
        run.state.hp = 1;
        enter_first(&mut run, MapNodeKind::Monster);
        lethal_combat_hp_loss(&mut run);
        assert_fairy_rescued(&run, slot);
    }
}

#[test]
// Source: m31-source-check-v1.md §B, RewardItem.java:299–316; StorePotion.java:74–99;
// AbstractPlayer.java:2135–2155.
// Grade: Run
fn m31_a11_sozu_potion_rows() {
    let mut reward = on_map(Character::Ironclad, 11);
    own_plain_relic(&mut reward, RelicId::Sozu);
    own_plain_relic(&mut reward, RelicId::WhiteBeastStatue);
    enter_first(&mut reward, MapNodeKind::Monster);
    super::m28_world::win(&mut reward);
    let rows = potion_rows(&reward);
    assert_eq!(rows.len(), 1, "the guaranteed potion reward is present under Sozu");
    assert_eq!(rows[0].2, RoomChoiceStatus::Offered);
    let before = reward.state.potions.clone();
    reward.step(&GameAction::Choose(rows[0].0)).unwrap();
    assert_eq!(reward.state.potions, before, "Sozu consumes the claimed reward without inserting it");
    assert_eq!(potion_rows(&reward).iter().find(|row| row.0 == rows[0].0).unwrap().2, RoomChoiceStatus::Claimed);

    let mut shop = on_map(Character::Ironclad, 11);
    own_plain_relic(&mut shop, RelicId::Sozu);
    enter_shop(&mut shop);
    let rows = potion_rows(&shop);
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|(_, _, status)| *status == RoomChoiceStatus::Disabled), "Sozu disables every shop potion row");
}

fn full_neow_a11() -> Run {
    let profile = CreationProfile { neow_eligible: true, ..CreationProfile::experienced() };
    Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 11 },
        profile,
        RunRng::new([0x61; 32], crate::combat::rng::Partition::Test, 0),
        RoomScenario::Standard,
    ).unwrap()
}

fn choose_three_potions_neow(run: &mut Run) {
    let candidate = run.candidates.iter_mut().find(|candidate| {
        candidate.neow_offer.as_ref().is_some_and(|offer| offer.category == 1)
    }).expect("full Neow category 1 candidate");
    let id = candidate.id;
    candidate.neow_offer = Some(NeowOffer { category: 1, reward: NeowReward::ThreePotions, drawback: None });
    let row = run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|row| row.id == id).unwrap();
    row.kind = RoomChoiceKind::NeowBlessing { category: 1, reward: NeowReward::ThreePotions, drawback: None };
    run.step(&GameAction::Choose(id)).unwrap();
}

#[test]
// Source: m31-source-check-v1.md §B, NeowReward.java:210–244,288–300;
// AbstractPlayer.java:2135–2155.
// Grade: Run
fn m31_a11_neow_potion_bundle_two_slots() {
    let mut run = full_neow_a11();
    assert_eq!(run.state.potions, [None, None], "the A11 base inventory exists before Neow");
    choose_three_potions_neow(&mut run);
    let rows = potion_rows(&run);
    assert_eq!(rows.len(), 3, "Three Small Potions creates three reward rows");
    assert!(rows.iter().all(|(_, _, status)| *status == RoomChoiceStatus::Offered));

    for _ in 0..2 {
        let id = potion_rows(&run).into_iter().find(|(_, _, status)| *status == RoomChoiceStatus::Offered).unwrap().0;
        run.step(&GameAction::Choose(id)).unwrap();
    }
    assert_eq!(run.state.potions.iter().flatten().count(), 2, "two bundle rows fill the A11 slots");
    let rows = potion_rows(&run);
    assert_eq!(rows.iter().filter(|(_, _, status)| *status == RoomChoiceStatus::Claimed).count(), 2);
    assert_eq!(rows.iter().filter(|(_, _, status)| *status == RoomChoiceStatus::Disabled).count(), 1,
        "the remaining bundle row is disabled when both slots are full");
}

#[test]
// Source: m31-source-check-v1.md §B, StorePotion.java:74–99; AbstractPlayer.java:2135–2155.
// Grade: Run
fn m31_a11_shop_potion_disabled_when_full() {
    for (potions, expected) in [
        ([Some(PotionId::BlockPotion), Some(PotionId::FirePotion)], RoomChoiceStatus::Disabled),
        ([Some(PotionId::BlockPotion), None], RoomChoiceStatus::Offered),
    ] {
        let mut run = on_map(Character::Ironclad, 11);
        run.state.potions = potions.to_vec();
        enter_shop(&mut run);
        let rows = potion_rows(&run);
        assert!(!rows.is_empty(), "Standard shop has potion rows");
        assert!(rows.iter().all(|(_, _, status)| *status == expected), "A11 shop with {potions:?}");
    }
}

#[test]
// Source: m31-source-check-v1.md §B, EntropicBrew.java:48–75;
// AbstractPlayer.java:235,292–307 (A10/A11 base counts).
// Grade: Combat
fn m31_a11_entropic_brew_samples_two() {
    for (ascension, count) in [(10_u8, 3_usize), (11, 2)] {
        let mut outside = on_map(Character::Ironclad, ascension);
        outside.state.potions[0] = Some(PotionId::EntropicBrew);
        assert_outside_brew_samples(&mut outside, count);

        let mut in_combat = on_map(Character::Ironclad, ascension);
        in_combat.state.potions[0] = Some(PotionId::EntropicBrew);
        enter_first(&mut in_combat, MapNodeKind::Monster);
        assert_combat_brew_samples(&mut in_combat, count);
    }
}

#[test]
// Source: m31-source-check-v1.md §B, Alchemize.java:23–28;
// AbstractPlayer.java:2135–2155.
// Grade: Combat
fn m31_a11_alchemize_full_loses_potion() {
    let mut run = on_map(Character::Silent, 11);
    run.state.potions = vec![Some(PotionId::BlockPotion), Some(PotionId::FirePotion)];
    set_only_alchemize_in_deck(&mut run);
    enter_first(&mut run, MapNodeKind::Monster);
    let before = combat_potions(&run);
    assert_eq!(before.len(), 2);
    play_alchemize(&mut run);
    assert_eq!(combat_potions(&run), before, "Alchemize's generated potion is lost when both A11 slots are full");
    assert!(run.state.combat.as_ref().unwrap().fault_record().is_none(), "full capacity is not a combat fault");
}

fn lab_reward_screen(ascension: u8) -> Run {
    let mut run = on_map(Character::Ironclad, ascension);
    enter_lab(&mut run);
    take_lab_potions(&mut run);
    run
}

#[test]
// Source: m31-source-check-v1.md §B, Lab.java:50–64; RewardItem.java:299–316;
// AbstractPlayer.java:235,292–307.
// Grade: Run
fn m31_a15_lab_two_rows_two_slots() {
    for (ascension, expected_rows) in [(14_u8, 3_usize), (15, 2)] {
        let mut run = lab_reward_screen(ascension);
        assert_eq!(run.state.potions.len(), 2, "A{ascension} keeps the A11 base capacity");
        assert_eq!(potion_rows(&run).len(), expected_rows, "A{ascension} Lab row count");
        assert!(potion_rows(&run).iter().all(|(_, _, status)| *status == RoomChoiceStatus::Offered));

        for _ in 0..2 {
            let id = potion_rows(&run).into_iter().find(|(_, _, status)| *status == RoomChoiceStatus::Offered).unwrap().0;
            run.step(&GameAction::Choose(id)).unwrap();
        }
        assert_eq!(run.state.potions.iter().flatten().count(), 2, "both empty A11 slots can be claimed at A{ascension}");
        let rows = potion_rows(&run);
        assert_eq!(rows.iter().filter(|(_, _, status)| *status == RoomChoiceStatus::Claimed).count(), 2);
        assert_eq!(rows.iter().filter(|(_, _, status)| *status == RoomChoiceStatus::Disabled).count(), expected_rows - 2,
            "A{ascension}: excess Lab rows are disabled only after both slots fill");
    }
}
