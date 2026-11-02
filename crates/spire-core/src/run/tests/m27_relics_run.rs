//! M27 plan Task R3, lane R3, key m27_relics_run.
//! Run-scoped relic assertions use the complete-admission owners and fixed source literals.

use super::super::*;
use crate::{
    combat::{CombatAction, FullPowerId, TargetId},
    content::{CardId, CardType, Character, EncounterId, PotionId, RelicId, RelicTier},
    items::{self, dispatch::RunItemEvent, run_laws::{self, RewardRoom}},
    run::CardInstance,
};
use std::collections::BTreeMap;

fn complete(relics: Vec<RelicId>, counters: BTreeMap<RelicId, i32>) -> Run {
    items::tests::m27_support::complete_run(
        Character::Ironclad, relics, vec![Some(PotionId::FirePotion); 3], counters, EncounterId::Cultist,
    )
}

fn settled(relics: Vec<RelicId>, counters: BTreeMap<RelicId, i32>) -> Run {
    let mut run = complete(relics, counters);
    for slot in 0..3 {
        if run.state.combat.is_none() { break; }
        run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) })
            .unwrap_or_else(|error| panic!("settle complete Run with Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state.combat.is_none() && run.state.active_combat.is_none());
    run
}

fn fire(run: &mut Run, event: RunItemEvent) -> RunItemEvent {
    let mut event = event;
    items::dispatch::run_event(&mut run.state, &mut event)
        .unwrap_or_else(|error| panic!("dispatch {event:?}: {error:?}"));
    event
}

fn rewards(room: RewardRoom) -> RunItemEvent {
    RunItemEvent::RewardsBuilding { room, card_choices: 3, extra_card_rewards: 0,
        gold_percent: 0, extra_elite_relics: 0, include_potion: false, singing_bowl: false }
}

fn card_count(run: &Run, id: CardId) -> usize { run.state.deck.iter().filter(|card| card.id == id).count() }

fn test_ancient_tea_set() {
    let mut run = settled(vec![RelicId::AncientTeaSet], BTreeMap::new());
    fire(&mut run, RunItemEvent::RoomEntered { kind: RoomViewKind::Campfire });
    assert_eq!(run.state.relic_counters.get(&RelicId::AncientTeaSet), Some(&1), "Rest Site entry arms one bonus");
    items::tests::m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    let combat = run.state.combat.as_ref().unwrap();
    assert_eq!(combat.public_full().unwrap().player.energy, 5, "first combat after Rest starts with 2 extra Energy");
    assert_eq!(combat.item_context().unwrap().counter(RelicId::AncientTeaSet), 0, "the first turn consumes the arm");
}

#[test]
fn m27_ancient_tea_set_rest_arms_energy() { test_ancient_tea_set(); }
#[test]
fn m27_ancient_tea_set_boundary() { test_ancient_tea_set(); }
#[test]
fn m27_ancient_tea_set_two_combat_counter() { test_ancient_tea_set(); }

fn test_ceramic_fish() {
    let mut run = settled(vec![RelicId::CeramicFish], BTreeMap::new());
    let before = run.state.gold;
    let _ = fire(&mut run, rewards(RewardRoom::Ordinary { enemy_escaped: false }));
    assert_eq!(run.state.gold, before, "building a reward without claiming a card grants no gold");
    crate::run::mutations::obtain_card(&mut run.state, CardId::Anger, 0).unwrap();
    assert_eq!(run.state.gold, before + 9, "an actual card obtain grants exactly 9 gold");
}
#[test]
fn m27_ceramic_fish_card_obtain_gold() { test_ceramic_fish(); }
#[test]
fn m27_ceramic_fish_boundary() { test_ceramic_fish(); }

#[cfg(feature = "m27-diagnostics")]
fn dream_catcher_campfire() -> Run {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    let supply = CompleteItemSupply { relics: vec![RelicId::DreamCatcher], potions: vec![None; 3], counters: BTreeMap::new() };
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
    let node = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == 51).unwrap().id;
    run.step(&GameAction::Move(node)).unwrap();
    run
}
#[cfg(feature = "m27-diagnostics")]
fn test_dream_catcher() {
    let mut run = dream_catcher_campfire();
    let rest = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| matches!(row.kind, RoomChoiceKind::Rest { .. })).unwrap().id;
    run.step(&GameAction::Choose(rest)).unwrap();
    // Main-4 amendment: the source opens the combat reward screen holding one card reward item.
    let screen = run.decision().unwrap().room.unwrap();
    assert_eq!(screen.kind, RoomViewKind::Rewards, "choosing Rest opens Dream Catcher's reward screen");
    let rows = screen.choices.iter().filter(|row| row.kind == RoomChoiceKind::CardReward).map(|row| row.id).collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "Dream Catcher adds exactly one card reward");
    run.step(&GameAction::Choose(rows[0])).unwrap();
    let cards = run.decision().unwrap().room.unwrap();
    assert_eq!(cards.kind, RoomViewKind::CardReward);
    assert_eq!(cards.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).count(), 3,
        "Dream Catcher uses the ordinary three-choice card reward");
}
#[cfg(feature = "m27-diagnostics")]
fn test_dream_catcher_boundary() {
    let mut run = dream_catcher_campfire();
    let smith = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.kind == RoomChoiceKind::Smith).unwrap().id;
    run.step(&GameAction::Choose(smith)).unwrap();
    assert!(run.state.pending.as_ref().is_some_and(|selection| selection.selection.required == 1),
        "Smith opens its card selection");
    assert_eq!(run.decision().unwrap().room.unwrap().kind, RoomViewKind::Campfire,
        "Smith does not open a Dream Catcher card reward");
}
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_dream_catcher_rest_card_reward() { test_dream_catcher(); }
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_dream_catcher_boundary() { test_dream_catcher_boundary(); }

fn test_juzu_bracelet() {
    let mut run = settled(vec![RelicId::JuzuBracelet], BTreeMap::new());
    let rules = run_laws::unknown_room(&mut run.state);
    assert!(rules.no_monsters, "Juzu removes combat encounters from Unknown rooms");
    assert!(run.state.relics.contains(&RelicId::JuzuBracelet), "the Unknown-room rule does not consume Juzu");
    items::tests::m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert!(run.state.combat.is_some(), "direct authored-combat entry still works while Juzu is owned");
}
#[test]
fn m27_juzu_bracelet_monster_to_event() { test_juzu_bracelet(); }
#[test]
fn m27_juzu_bracelet_boundary() { test_juzu_bracelet(); }

fn test_maw_bank() {
    let mut run = settled(vec![RelicId::MawBank], BTreeMap::new());
    fire(&mut run, RunItemEvent::FloorEntered { floor: 1 });
    assert_eq!(run.state.gold, 12, "each active floor grants 12 gold");
    fire(&mut run, RunItemEvent::ShopPurchase);
    assert_eq!(run.state.relic_counters.get(&RelicId::MawBank), Some(&1));
    fire(&mut run, RunItemEvent::FloorEntered { floor: 2 });
    assert_eq!(run.state.gold, 12, "a paid shop purchase disables later floor gold");
}
#[test]
fn m27_maw_bank_floor_gold_shop_disable() { test_maw_bank(); }
#[test]
fn m27_maw_bank_boundary() { test_maw_bank(); }
#[test]
fn m27_maw_bank_two_combat_counter() { test_maw_bank(); }

fn test_meal_ticket() {
    let mut run = settled(vec![RelicId::MealTicket], BTreeMap::new());
    run.state.hp = 40;
    fire(&mut run, RunItemEvent::ShopEntered);
    assert_eq!(run.state.hp, 55, "Shop entry heals 15 without a purchase");
    fire(&mut run, RunItemEvent::RoomEntered { kind: RoomViewKind::Map });
    assert_eq!(run.state.hp, 55, "non-Shop entry heals 0");
}
#[test]
fn m27_meal_ticket_shop_entry_heal() { test_meal_ticket(); }
#[test]
fn m27_meal_ticket_boundary() { test_meal_ticket(); }

fn test_omamori() {
    let mut run = settled(vec![RelicId::Omamori], BTreeMap::from([(RelicId::Omamori, 2)]));
    for expected in [1, 0] {
        let event = fire(&mut run, RunItemEvent::CardObtaining { id: CardId::Clumsy, upgrades: 0, negated: false });
        let RunItemEvent::CardObtaining { negated, .. } = event else { unreachable!() };
        assert!(negated, "Omamori negates a named Curse obtain");
        assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&expected));
    }
    let third = fire(&mut run, RunItemEvent::CardObtaining { id: CardId::Clumsy, upgrades: 0, negated: false });
    let RunItemEvent::CardObtaining { negated, .. } = third else { unreachable!() };
    assert!(!negated, "the third Curse is no longer blocked");
}
#[test]
fn m27_omamori_two_curse_blocks() { test_omamori(); }
#[test]
fn m27_omamori_negates_through_the_card_owner() {
    // Main-4 (negation-aware obtain owner): a negated Curse is not created, and an equip that obtains a curse
    // (Necronomicon's Necronomicurse) spends a charge instead of failing.
    let mut run = settled(vec![RelicId::Omamori], BTreeMap::from([(RelicId::Omamori, 2)]));
    let deck = run.state.deck.len();
    assert_eq!(crate::run::mutations::obtain_card_or_negated(&mut run.state, CardId::Clumsy, 0), Ok(None));
    assert_eq!(run.state.deck.len(), deck, "the negated Curse is not created");
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&1));
    crate::items::lifecycle::obtain_relic(&mut run.state, RelicId::Necronomicon,
        crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert!(!run.state.deck.iter().any(|card| card.id == CardId::Necronomicurse), "Necronomicurse is negated");
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&0));
    assert!(run.state.relics.contains(&RelicId::Necronomicon), "the relic is still obtained");
    let created = crate::run::mutations::obtain_card_or_negated(&mut run.state, CardId::Clumsy, 0).unwrap();
    assert!(created.is_some_and(|instance| run.state.deck.iter().any(|card| card.instance == instance && card.id == CardId::Clumsy)),
        "with no charge left the Curse is obtained");
}
#[test]
fn m27_omamori_boundary() { test_omamori(); }
#[test]
fn m27_omamori_two_combat_counter() { test_omamori(); }

fn test_regal_pillow() {
    let mut run = settled(vec![RelicId::RegalPillow], BTreeMap::new());
    run.state.hp = 40;
    let event = fire(&mut run, RunItemEvent::RestHeal { amount: 24 });
    assert_eq!(event, RunItemEvent::RestHeal { amount: 39 }, "Rest healing adds exactly 15");
    crate::run::mutations::heal(&mut run.state, 39).unwrap();
    assert_eq!(run.state.hp, 79, "Rest healing 24+15 raises 40 HP to 79");
    run.state.hp = 40;
    crate::run::mutations::heal(&mut run.state, 10).unwrap();
    assert_eq!(run.state.hp, 50, "ordinary healing receives no Pillow bonus");
}
#[test]
fn m27_regal_pillow_rest_heal_plus_fifteen() { test_regal_pillow(); }
#[test]
fn m27_regal_pillow_boundary() { test_regal_pillow(); }

fn test_smiling_mask() {
    let mut run = settled(vec![RelicId::SmilingMask], BTreeMap::new());
    assert_eq!(fire(&mut run, RunItemEvent::RemovalPrice { price: 75 }), RunItemEvent::RemovalPrice { price: 50 });
    assert_eq!(fire(&mut run, RunItemEvent::RemovalPrice { price: 100 }), RunItemEvent::RemovalPrice { price: 50 });
}
#[test]
fn m27_smiling_mask_removal_price_fifty() { test_smiling_mask(); }
#[test]
fn m27_smiling_mask_boundary() { test_smiling_mask(); }

fn test_tiny_chest() {
    let mut run = settled(vec![RelicId::TinyChest], BTreeMap::from([(RelicId::TinyChest, 0)]));
    for roll in 1..=3 {
        assert!(!run_laws::unknown_room(&mut run.state).force_treasure, "roll {roll} retains its result");
        assert_eq!(run.state.relic_counters.get(&RelicId::TinyChest), Some(&roll), "each Unknown roll counts");
    }
    assert!(run_laws::unknown_room(&mut run.state).force_treasure, "the fourth Unknown roll forces Treasure");
    assert_eq!(run.state.relic_counters.get(&RelicId::TinyChest), Some(&0), "the fourth roll resets the counter");
    for _ in 0..3 { assert!(!run_laws::unknown_room(&mut run.state).force_treasure); }
    assert!(run_laws::unknown_room(&mut run.state).force_treasure, "the eighth Unknown roll forces Treasure again");
}
#[test]
fn m27_tiny_chest_fourth_unknown_treasure() { test_tiny_chest(); }
#[test]
fn m27_tiny_chest_boundary() { test_tiny_chest(); }
#[test]
fn m27_tiny_chest_two_combat_counter() { test_tiny_chest(); }

fn equip(relic: RelicId, deck: Vec<CardInstance>) -> Run {
    let mut run = settled(vec![], BTreeMap::new());
    run.state.deck = deck;
    items::lifecycle::obtain_relic(&mut run.state, relic, items::lifecycle::RelicSource::Reward)
        .unwrap_or_else(|error| panic!("equip {relic:?}: {error:?}"));
    run
}

fn test_war_paint() {
    let deck = [CardId::Defend, CardId::ShrugItOff, CardId::Defend]
        .into_iter().enumerate().map(|(index, id)| CardInstance { id, instance: index as u64 + 1, upgrades: 0, misc: 0 }).collect();
    let run = equip(RelicId::WarPaint, deck);
    assert_eq!(run.state.deck.iter().filter(|card| card.upgrades > 0).count(), 2,
        "War Paint upgrades exactly two eligible Skills, including matching instances");
}
#[test]
fn m27_war_paint_two_skill_upgrades() { test_war_paint(); }
#[test]
fn m27_war_paint_boundary() { test_war_paint(); }

fn test_whetstone() {
    let deck = [CardId::Strike, CardId::IronWave, CardId::Bash]
        .into_iter().enumerate().map(|(index, id)| CardInstance { id, instance: index as u64 + 1, upgrades: 0, misc: 0 }).collect();
    let run = equip(RelicId::Whetstone, deck);
    assert_eq!(run.state.deck.iter().filter(|card| card.upgrades > 0).count(), 2,
        "Whetstone upgrades exactly two eligible Attacks");
}
#[test]
fn m27_whetstone_two_attack_upgrades() { test_whetstone(); }
#[test]
fn m27_whetstone_boundary() { test_whetstone(); }

fn test_courier() {
    let mut run = settled(vec![RelicId::MembershipCard, RelicId::TheCourier], BTreeMap::new());
    assert_eq!(fire(&mut run, RunItemEvent::ShopPrice { price: 100 }), RunItemEvent::ShopPrice { price: 40 },
        "Membership Card and Courier discount in separately rounded steps");
    assert_eq!(fire(&mut run, RunItemEvent::RemovalPrice { price: 75 }), RunItemEvent::RemovalPrice { price: 38 },
        "first removal base 75 rounds to 38");
    // Main-4 amendment (economy research §3.2): the 40 of base 100 is the same-shop display after a paid purge, when
    // removal is already disabled; the next shop entry recomputes from base, so Membership Card leaves 50.
    fire(&mut run, RunItemEvent::ShopPurchase);
    assert_eq!(fire(&mut run, RunItemEvent::RemovalPrice { price: 100 }), RunItemEvent::RemovalPrice { price: 50 },
        "the next shop's removal base 100 with both relics is 50");
    let mut courier = settled(vec![RelicId::TheCourier], BTreeMap::new());
    assert_eq!(fire(&mut courier, RunItemEvent::RemovalPrice { price: 75 }), RunItemEvent::RemovalPrice { price: 60 },
        "The Courier alone takes 20% off removal");
}
#[test]
fn m27_the_courier_shop_price_twenty_off() { test_courier(); }
#[test]
fn m27_the_courier_boundary() { test_courier(); }
#[test]
fn m27_membership_card_shop_price_half() { test_courier(); }
#[test]
fn m27_membership_card_boundary() { test_courier(); }

fn test_darkstone_periapt() {
    let mut run = settled(vec![RelicId::DarkstonePeriapt], BTreeMap::new());
    run.state.hp = 40;
    run.state.max_hp = 80;
    let before = (run.state.hp, run.state.max_hp);
    let _ = fire(&mut run, rewards(RewardRoom::Ordinary { enemy_escaped: false }));
    assert_eq!((run.state.hp, run.state.max_hp), before, "displaying a Curse offer changes no HP");
    crate::run::mutations::obtain_card(&mut run.state, CardId::Clumsy, 0).unwrap();
    assert_eq!((run.state.max_hp, run.state.hp), (86, 46), "an obtained Curse raises max/current HP by 6");
}
#[test]
fn m27_darkstone_periapt_curse_max_hp_six() { test_darkstone_periapt(); }
#[test]
fn m27_darkstone_periapt_boundary() { test_darkstone_periapt(); }

fn test_eternal_feather() {
    let mut run = settled(vec![RelicId::EternalFeather], BTreeMap::new());
    run.state.hp = 40;
    run.state.deck = (0..25).map(|index| CardInstance { id: CardId::Strike, instance: index + 1, upgrades: 0, misc: 0 }).collect();
    fire(&mut run, RunItemEvent::RoomEntered { kind: RoomViewKind::Campfire });
    assert_eq!(run.state.hp, 55, "25 deck cards request 3×floor(25/5) = 15 healing");
    run.state.hp = 40;
    run.state.deck.truncate(4);
    fire(&mut run, RunItemEvent::RoomEntered { kind: RoomViewKind::Campfire });
    assert_eq!(run.state.hp, 40, "four cards request zero healing");
}
#[test]
fn m27_eternal_feather_rest_room_heal() { test_eternal_feather(); }
#[test]
fn m27_eternal_feather_boundary() { test_eternal_feather(); }

fn test_egg(relic: RelicId, id: CardId, card_type: CardType) {
    let mut run = settled(vec![relic], BTreeMap::new());
    let before = card_count(&run, id);
    crate::run::mutations::obtain_card(&mut run.state, id, 0).unwrap();
    let obtained = run.state.deck.iter().find(|card| card.id == id && card.upgrades == 1).unwrap();
    assert_eq!(obtained.upgrades, 1, "the newly obtained eligible {card_type:?} is upgraded once");
    assert_eq!(card_count(&run, id), before + 1);
    let unrelated = match card_type {
        CardType::Attack => CardId::Inflame,
        CardType::Power | CardType::Skill => CardId::Strike,
        CardType::Status | CardType::Curse => unreachable!(),
    };
    let irrelevant = fire(&mut run, RunItemEvent::CardObtaining { id: unrelated, upgrades: 0, negated: false });
    let RunItemEvent::CardObtaining { upgrades, .. } = irrelevant else { unreachable!() };
    assert_eq!(upgrades, 0, "an unrelated, unclaimed temporary card is unchanged");
}

#[test]
fn m27_frozen_egg_power_obtain_upgrade() { test_egg(RelicId::FrozenEgg, CardId::Inflame, CardType::Power); }
#[test]
fn m27_frozen_egg_boundary() { test_egg(RelicId::FrozenEgg, CardId::Inflame, CardType::Power); }
#[test]
fn m27_molten_egg_attack_obtain_upgrade() { test_egg(RelicId::MoltenEgg, CardId::IronWave, CardType::Attack); }
#[test]
fn m27_molten_egg_boundary() { test_egg(RelicId::MoltenEgg, CardId::IronWave, CardType::Attack); }
#[test]
fn m27_toxic_egg_skill_obtain_upgrade() { test_egg(RelicId::ToxicEgg, CardId::ShrugItOff, CardType::Skill); }
#[test]
fn m27_toxic_egg_boundary() { test_egg(RelicId::ToxicEgg, CardId::ShrugItOff, CardType::Skill); }

fn test_matryoshka() {
    let mut run = settled(vec![RelicId::Matryoshka], BTreeMap::from([(RelicId::Matryoshka, 2)]));
    run.state.relic_pools = Some(items::relic_pools::explicit_for_test(
        vec![RelicId::Anchor, RelicId::Vajra], vec![RelicId::Kunai, RelicId::OrnamentalFan], vec![RelicId::DeadBranch], vec![], vec![]));
    let boss = run_laws::chest_contents(&mut run.state, true).unwrap();
    assert_eq!(boss.extra_relic, None, "a boss chest gains no extra relic");
    assert_eq!(run.state.relic_counters.get(&RelicId::Matryoshka), Some(&2), "a boss chest spends no charge");
    let mut extras = Vec::new();
    for charge in [1, 0] {
        let rules = run_laws::chest_contents(&mut run.state, false).unwrap();
        let extra = rules.extra_relic.expect("a charged non-boss chest gains one extra relic");
        let front = [run_laws_front(&extras, RelicId::Anchor, RelicId::Vajra), run_laws_front(&extras, RelicId::Kunai, RelicId::OrnamentalFan)];
        assert!(front.contains(&extra), "the front Common or Uncommon relic, got {extra:?}");
        assert_eq!(run.state.relic_counters.get(&RelicId::Matryoshka), Some(&charge), "each extra relic spends a charge");
        extras.push(extra);
    }
    let pools = run.state.relic_pools.as_ref().unwrap();
    assert!(extras.iter().all(|id| !pools.common.contains(id) && !pools.uncommon.contains(id)), "extra relics leave the pools");
    assert_eq!(pools.rare, vec![RelicId::DeadBranch], "the extra relic is never Rare");
    assert_eq!(run_laws::chest_contents(&mut run.state, false).unwrap().extra_relic, None, "no charge, no extra relic");
    let mut rows = vec![RoomChoiceKind::Gold(25), RoomChoiceKind::Relic(RelicId::Lantern), RoomChoiceKind::Sapphire];
    run_laws::apply_chest_rules(&run_laws::ChestRules { extra_relic: Some(RelicId::Anchor), ..Default::default() }, &mut rows);
    assert_eq!(rows[0], RoomChoiceKind::Relic(RelicId::Anchor), "onChestOpen adds the extra relic before the chest rows");
}
fn run_laws_front(taken: &[RelicId], first: RelicId, second: RelicId) -> RelicId {
    if taken.contains(&first) { second } else { first }
}
#[test]
fn m27_matryoshka_nonboss_chest_charge() { test_matryoshka(); }
#[test]
fn m27_matryoshka_boundary() { test_matryoshka(); }
#[test]
fn m27_matryoshka_two_combat_counter() { test_matryoshka(); }

fn test_question_card() {
    let run = settled(vec![RelicId::QuestionCard], BTreeMap::new());
    let shape = run_laws::card_reward_shape(&run.state, RewardRoom::Ordinary { enemy_escaped: false });
    assert_eq!(shape.choices, 4, "Question Card adds one choice to a 3-choice reward");
    // M28c review I1: an EventRoom combat's card reward is a card RewardItem too (amended; original
    // bytes in the M28c review-fix evidence).
    let mut event_room = settled(vec![RelicId::QuestionCard], BTreeMap::new());
    let event = fire(&mut event_room, rewards(RewardRoom::Event));
    let RunItemEvent::RewardsBuilding { card_choices, .. } = event else { unreachable!() };
    assert_eq!(card_choices, 4, "an EventRoom card reward also gains the choice");
}
#[test]
fn m27_question_card_reward_choice_plus_one() { test_question_card(); }
#[test]
fn m27_question_card_boundary() { test_question_card(); }

fn test_singing_bowl() {
    let mut run = settled(vec![RelicId::SingingBowl], BTreeMap::new());
    let mut event = fire(&mut run, rewards(RewardRoom::Ordinary { enemy_escaped: false }));
    let RunItemEvent::RewardsBuilding { singing_bowl, .. } = &event else { unreachable!() };
    assert!(*singing_bowl, "ordinary card reward appends the Singing Bowl choice");
    event = rewards(RewardRoom::Event);
    let RunItemEvent::RewardsBuilding { singing_bowl, .. } = fire(&mut run, event) else { unreachable!() };
    assert!(singing_bowl, "an EventRoom card reward also gains the Bowl option (M28c review I1)");

    let mut bowl = singing_bowl_run();
    bowl.step(&GameAction::Choose(ChoiceId(2))).unwrap();
    assert_eq!((bowl.state.hp, bowl.state.max_hp), (42, 82), "choosing Bowl adds and heals 2 HP");
    let mut skip = singing_bowl_run();
    skip.step(&GameAction::Choose(ChoiceId(3))).unwrap();
    assert_eq!((skip.state.hp, skip.state.max_hp), (40, 80), "plain Skip changes HP by 0");
}
fn singing_bowl_run() -> Run {
    let mut run = settled(vec![RelicId::SingingBowl], BTreeMap::new());
    run.state.hp = 40;
    let parent = RoomView { kind: RoomViewKind::Rewards, choices: vec![RoomChoiceView {
        id: ChoiceId(1), kind: RoomChoiceKind::CardReward, price: None, status: RoomChoiceStatus::Disabled,
    }] };
    let screen = RoomView { kind: RoomViewKind::CardReward, choices: vec![
        RoomChoiceView { id: ChoiceId(2), kind: RoomChoiceKind::SingingBowl, price: None, status: RoomChoiceStatus::Offered },
        RoomChoiceView { id: ChoiceId(3), kind: RoomChoiceKind::Skip, price: None, status: RoomChoiceStatus::Offered },
    ] };
    run.state.room_loop = Some(crate::run::rooms::RoomLoop {
        scenario: RoomScenario::M4Act1Prefix, act1: None, act2: None, act3: None, act4: None,
        m10_elite_relic_cursor: 0, screen, purge_cost: 75, reward_parent: Some(parent), active_rewards: None,
        boss: None, transition: None, act2_boss_order: None, act3_boss_order: None, joust: None,
        joust_visits: 0, act3_event: None, act3_end: None, standard_event: None,
    });
    run
}
#[test]
fn m27_singing_bowl_bowl_max_hp_two() { test_singing_bowl(); }
#[test]
fn m27_singing_bowl_boundary() { test_singing_bowl(); }

fn test_white_beast() {
    let mut run = settled(vec![RelicId::WhiteBeastStatue], BTreeMap::new());
    let event = fire(&mut run, rewards(RewardRoom::Ordinary { enemy_escaped: false }));
    let RunItemEvent::RewardsBuilding { include_potion, .. } = event else { unreachable!() };
    assert!(include_potion, "White Beast Statue makes the potion reward present");
}
#[test]
fn m27_white_beast_statue_potion_chance_override() { test_white_beast(); }
#[test]
fn m27_white_beast_statue_boundary() { test_white_beast(); }

fn test_girya() {
    let mut run = settled(vec![RelicId::Girya], BTreeMap::from([(RelicId::Girya, 3)]));
    let choices = vec![RoomChoiceKind::Rest { heal: 24 }, RoomChoiceKind::Smith];
    let event = fire(&mut run, RunItemEvent::CampfireOptions { options: choices.clone() });
    let RunItemEvent::CampfireOptions { options } = &event else { unreachable!() };
    assert!(!options.contains(&RoomChoiceKind::Lift), "Girya offers no fourth Lift after three uses");
    items::tests::m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    let combat = run.state.combat.as_ref().unwrap();
    assert_eq!(combat.public_full().unwrap().player.powers.iter().find(|power| power.id == FullPowerId("strength")).map(|power| power.amount), Some(3));
    let mut check = settled(vec![RelicId::Girya], BTreeMap::from([(RelicId::Girya, 2)]));
    let event = fire(&mut check, RunItemEvent::CampfireOptions { options: choices });
    let RunItemEvent::CampfireOptions { options } = event else { unreachable!() };
    assert!(options.contains(&RoomChoiceKind::Lift), "Girya offers the third Lift at counter two");
    assert_eq!(check.state.relic_counters.get(&RelicId::Girya), Some(&2), "presenting Lift does not count until selected");
}
#[test]
fn m27_girya_three_lifts_strength() { test_girya(); }
#[test]
fn m27_girya_boundary() { test_girya(); }
#[test]
fn m27_girya_two_combat_counter() { test_girya(); }

fn test_old_coin() {
    let run = equip(RelicId::OldCoin, vec![CardInstance { id: CardId::Strike, instance: 1, upgrades: 0, misc: 0 }]);
    assert_eq!(run.state.gold, 300, "Old Coin requests exactly +300 gold at equip");
}
#[test]
fn m27_old_coin_equip_gold_300() { test_old_coin(); }
#[test]
fn m27_old_coin_boundary() { test_old_coin(); }

#[cfg(feature = "m27-diagnostics")]
fn test_peace_pipe() {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    let supply = CompleteItemSupply { relics: vec![RelicId::PeacePipe], potions: vec![None; 3], counters: BTreeMap::new() };
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
    let node = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == 51).unwrap().id;
    run.step(&GameAction::Move(node)).unwrap();
    let screen = run.decision().unwrap().room.unwrap();
    let toke = screen.choices.iter().find(|row| row.kind == RoomChoiceKind::Toke).unwrap().id;
    let old_len = run.state.deck.len();
    run.step(&GameAction::Choose(toke)).unwrap();
    let selection = run.state.pending.as_ref().unwrap().selection.candidates[0].id;
    run.step(&GameAction::Select(selection)).unwrap();
    assert_eq!(run.state.deck.len(), old_len - 1, "Toke removes exactly one selected card");
}
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_peace_pipe_toke_remove_card() { test_peace_pipe(); }
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_peace_pipe_boundary() { test_peace_pipe(); }

fn test_prayer_wheel() {
    let run = settled(vec![RelicId::PrayerWheel], BTreeMap::new());
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Ordinary { enemy_escaped: false }).rewards, 2);
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Elite).rewards, 1);
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Boss).rewards, 1);
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Event).rewards, 1);
}
#[test]
fn m27_prayer_wheel_monster_second_card_row() { test_prayer_wheel(); }
#[test]
fn m27_prayer_wheel_boundary() { test_prayer_wheel(); }

#[cfg(feature = "m27-diagnostics")]
fn test_shovel() {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    let supply = CompleteItemSupply { relics: vec![RelicId::Shovel], potions: vec![None; 3], counters: BTreeMap::new() };
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
    let node = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == 51).unwrap().id;
    run.step(&GameAction::Move(node)).unwrap();
    let dig = run.decision().unwrap().room.unwrap().choices.iter().find(|row| row.kind == RoomChoiceKind::Dig).unwrap().id;
    run.step(&GameAction::Choose(dig)).unwrap();
    let screen = run.decision().unwrap().room.unwrap();
    assert_eq!(screen.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).count(), 1,
        "Dig offers exactly one relic from the rolled tier's front selector");
}
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_shovel_dig_front_relic() { test_shovel(); }
#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_shovel_boundary() { test_shovel(); }

fn test_wing_boots() {
    let mut run = settled(vec![RelicId::WingBoots], BTreeMap::from([(RelicId::WingBoots, 3)]));
    let rules = run_laws::map_path(&run.state);
    assert_eq!(rules.off_path_moves, 3, "Wing Boots permits exactly three same-row off-path moves");
    run.state.relic_counters.insert(RelicId::WingBoots, 0);
    assert_eq!(run_laws::map_path(&run.state).off_path_moves, 0, "a fourth move is unavailable at zero charges");
}
#[test]
fn m27_wing_boots_three_offpath_moves() { test_wing_boots(); }
#[test]
fn m27_wing_boots_boundary() { test_wing_boots(); }
#[test]
fn m27_wing_boots_two_combat_counter() { test_wing_boots(); }

fn test_black_star() {
    let mut run = settled(vec![RelicId::BlackStar], BTreeMap::new());
    let RunItemEvent::RewardsBuilding { extra_elite_relics, .. } = fire(&mut run, rewards(RewardRoom::Elite)) else { unreachable!() };
    assert_eq!(extra_elite_relics, 1, "actual MonsterRoomElite adds a second relic row");
    let RunItemEvent::RewardsBuilding { extra_elite_relics, .. } = fire(&mut run, rewards(RewardRoom::Event)) else { unreachable!() };
    assert_eq!(extra_elite_relics, 0, "event-authored elite has no extra row");
}
#[test]
fn m27_black_star_elite_second_relic() { test_black_star(); }
#[test]
fn m27_black_star_boundary() { test_black_star(); }

fn test_busted_crown() {
    let mut run = settled(vec![RelicId::BustedCrown], BTreeMap::new());
    let RunItemEvent::RewardsBuilding { card_choices, .. } = fire(&mut run, rewards(RewardRoom::Ordinary { enemy_escaped: false })) else { unreachable!() };
    assert_eq!(card_choices, 1, "baseline 3 choices subtract exactly two");
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Ordinary { enemy_escaped: false }).choices, 1);
    let together = settled(vec![RelicId::BustedCrown, RelicId::QuestionCard], BTreeMap::new());
    assert_eq!(run_laws::card_reward_shape(&together.state, RewardRoom::Ordinary { enemy_escaped: false }).choices, 2,
        "Question Card and Crown combine to two choices");
}
#[test]
fn m27_busted_crown_one_reward_choice() { test_busted_crown(); }
#[test]
fn m27_busted_crown_boundary() { test_busted_crown(); }

fn test_coffee_dripper() {
    let run = settled(vec![RelicId::CoffeeDripper], BTreeMap::new());
    let mut options = vec![RoomChoiceKind::Rest { heal: 24 }, RoomChoiceKind::Smith];
    let mut run = run;
    let RunItemEvent::CampfireOptions { options: filtered } = fire(&mut run, RunItemEvent::CampfireOptions { options: options.clone() }) else { unreachable!() };
    assert!(!filtered.iter().any(|choice| matches!(choice, RoomChoiceKind::Rest { .. })));
    assert!(filtered.contains(&RoomChoiceKind::Smith), "other campfire options remain available");
    let combat = complete(vec![RelicId::CoffeeDripper], BTreeMap::new());
    assert_eq!(combat.state.combat.as_ref().unwrap().public_full().unwrap().player.max_energy, 4);
    options.push(RoomChoiceKind::Lift);
}
#[test]
fn m27_coffee_dripper_no_rest_plus_energy() { test_coffee_dripper(); }
#[test]
fn m27_coffee_dripper_boundary() { test_coffee_dripper(); }

fn test_cursed_key() {
    let mut run = settled(vec![RelicId::CursedKey], BTreeMap::new());
    let normal = run_laws::chest_contents(&mut run.state, false).unwrap();
    assert!(normal.curse, "nonboss chest adds one Curse offer");
    let boss = run_laws::chest_contents(&mut run.state, true).unwrap();
    assert!(!boss.curse, "boss chest adds no Curse offer");
}
#[test]
fn m27_cursed_key_nonboss_chest_curse() { test_cursed_key(); }
#[test]
fn m27_cursed_key_boundary() { test_cursed_key(); }

fn test_ectoplasm() {
    let mut run = settled(vec![RelicId::Ectoplasm], BTreeMap::new());
    crate::run::mutations::gain_gold(&mut run.state, 10).unwrap();
    assert_eq!(run.state.gold, 0, "Ectoplasm blocks direct positive gainGold");
    assert_eq!(fire(&mut run, RunItemEvent::GoldGaining { amount: 10 }), RunItemEvent::GoldGaining { amount: 0 });
}
#[test]
fn m27_ectoplasm_gold_gain_blocked() { test_ectoplasm(); }
#[test]
fn m27_ectoplasm_boundary() { test_ectoplasm(); }

fn test_fusion_hammer() {
    let mut run = settled(vec![RelicId::FusionHammer], BTreeMap::new());
    let RunItemEvent::CampfireOptions { options } = fire(&mut run, RunItemEvent::CampfireOptions {
        options: vec![RoomChoiceKind::Rest { heal: 24 }, RoomChoiceKind::Smith],
    }) else { unreachable!() };
    assert!(!options.contains(&RoomChoiceKind::Smith), "Fusion Hammer removes only Smith");
    assert!(options.contains(&RoomChoiceKind::Rest { heal: 24 }), "Rest remains available");
    let combat = complete(vec![RelicId::FusionHammer], BTreeMap::new());
    assert_eq!(combat.state.combat.as_ref().unwrap().public_full().unwrap().player.max_energy, 4);
}
#[test]
fn m27_fusion_hammer_no_smith_plus_energy() { test_fusion_hammer(); }
#[test]
fn m27_fusion_hammer_boundary() { test_fusion_hammer(); }

fn test_lee_waffle() {
    let mut run = settled(vec![], BTreeMap::new());
    run.state.hp = 40;
    run.state.max_hp = 80;
    items::lifecycle::obtain_relic(&mut run.state, RelicId::LeesWaffle, items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!((run.state.max_hp, run.state.hp), (87, 87), "Lee's Waffle raises max HP by 7, then full-heals");
}
#[test]
fn m27_lees_waffle_max_hp_full_heal() { test_lee_waffle(); }
#[test]
fn m27_lees_waffle_boundary() { test_lee_waffle(); }

fn test_bloody_idol() {
    let mut run = settled(vec![RelicId::BloodyIdol], BTreeMap::new());
    run.state.hp = 40;
    fire(&mut run, RunItemEvent::GoldGained { amount: 0 });
    assert_eq!(run.state.hp, 40, "zero gained gold heals 0");
    fire(&mut run, RunItemEvent::GoldGained { amount: 10 });
    assert_eq!(run.state.hp, 45, "positive gained gold heals 5");
}
#[test]
fn m27_bloody_idol_gold_gain_heal_five() { test_bloody_idol(); }
#[test]
fn m27_bloody_idol_boundary() { test_bloody_idol(); }

fn test_circlet() {
    let mut run = settled(vec![RelicId::Circlet], BTreeMap::from([(RelicId::Circlet, 1)]));
    run.state.relic_pools = Some(items::relic_pools::explicit_for_test(vec![], vec![], vec![], vec![], vec![]));
    let rare = items::relic_pools::take(&mut run.state, RelicTier::Rare, items::relic_pools::Selector::Front,
        items::lifecycle::RelicSource::Reward, &[27]).unwrap();
    assert_eq!(rare, RelicId::Circlet);
    items::lifecycle::obtain_relic(&mut run.state, rare, items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(run.state.relic_counters.get(&RelicId::Circlet), Some(&2), "duplicate Circlet increments the owned count");
    // PM F7 (Main-4 integration-5): exhausted Boss returns the fallback Red Circlet.
    let boss = items::relic_pools::take(&mut run.state, RelicTier::Boss, items::relic_pools::Selector::Front,
        items::lifecycle::RelicSource::Reward, &[28]).unwrap();
    assert_eq!(boss, RelicId::RedCirclet);
}
#[test]
fn m27_circlet_rare_pool_fallback() { test_circlet(); }
#[test]
fn m27_circlet_boundary() { test_circlet(); }
#[test]
fn m27_circlet_two_combat_counter() { test_circlet(); }

fn test_golden_idol() {
    let mut run = settled(vec![RelicId::GoldenIdol], BTreeMap::new());
    let RunItemEvent::RewardsBuilding { gold_percent, .. } = fire(&mut run, rewards(RewardRoom::Ordinary { enemy_escaped: false })) else { unreachable!() };
    assert_eq!(gold_percent, 25, "ordinary combat gold receives a 25% bonus");
    // Economy research §2.3: the bonus applies to every room's gold row except TreasureRoom's.
    for room in [RewardRoom::Elite, RewardRoom::Boss, RewardRoom::Event] {
        let RunItemEvent::RewardsBuilding { gold_percent, .. } = fire(&mut run, rewards(room)) else { unreachable!() };
        assert_eq!(gold_percent, 25, "{room:?} combat gold receives a 25% bonus");
    }
    let before = run.state.gold;
    crate::run::mutations::gain_gold(&mut run.state, 6).unwrap();
    assert_eq!(run.state.gold, before + 6, "direct gold gain receives no stored-reward bonus");
    assert_eq!((6_u32 * 25 + 50) / 100, 2, "catalog base 6 rounds to a +2 bonus and total 8");
}
#[test]
fn m27_golden_idol_stored_gold_bonus() { test_golden_idol(); }
#[test]
fn m27_golden_idol_boundary() { test_golden_idol(); }

fn test_nloths_gift() {
    let run = settled(vec![RelicId::NlothsGift], BTreeMap::new());
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Ordinary { enemy_escaped: false }).rare_factor, 3,
        "eligible base Rare chance 5% is tripled to 15%");
    assert_eq!(run_laws::card_reward_shape(&run.state, RewardRoom::Event).rare_factor, 3,
        "an EventRoom card reward's Rare chance is tripled too (M28c review I1)");
}
#[test]
fn m27_nloths_gift_rare_chance_tripled() { test_nloths_gift(); }
#[test]
fn m27_nloths_gift_boundary() { test_nloths_gift(); }

fn test_nloths_hungry_face() {
    let mut run = settled(vec![RelicId::NlothsHungryFace], BTreeMap::new());
    assert!(!run_laws::chest_contents(&mut run.state, true).unwrap().remove_first_relic, "boss chest is unaffected");
    assert!(!run.state.relic_counters.contains_key(&RelicId::NlothsHungryFace), "a boss chest does not spend the use");
    let rules = run_laws::chest_contents(&mut run.state, false).unwrap();
    assert!(rules.remove_first_relic, "the next nonboss chest loses its first relic");
    assert_eq!(run.state.relic_counters.get(&RelicId::NlothsHungryFace), Some(&1), "the single use is spent");
    let mut rows = vec![RoomChoiceKind::Gold(30), RoomChoiceKind::Relic(RelicId::Anchor), RoomChoiceKind::Sapphire, RoomChoiceKind::Proceed];
    run_laws::apply_chest_rules(&rules, &mut rows);
    assert_eq!(rows, vec![RoomChoiceKind::Gold(30), RoomChoiceKind::Proceed], "relic and its linked Sapphire Key go; gold stays");
    let mut nested = vec![RoomChoiceKind::Gold(30), RoomChoiceKind::Relic(RelicId::Lantern), RoomChoiceKind::Sapphire];
    run_laws::apply_chest_rules(&run_laws::ChestRules { extra_relic: Some(RelicId::Anchor), ..rules }, &mut nested);
    assert_eq!(nested, vec![RoomChoiceKind::Gold(30), RoomChoiceKind::Relic(RelicId::Lantern), RoomChoiceKind::Sapphire],
        "with Matryoshka the chest still contains one relic and keeps its key");
    assert!(!run_laws::chest_contents(&mut run.state, false).unwrap().remove_first_relic, "following chest retains its contents");
}
#[test]
fn m27_nloths_hungry_face_next_chest_relic_removal() { test_nloths_hungry_face(); }
#[test]
fn m27_nloths_hungry_face_boundary() { test_nloths_hungry_face(); }
#[test]
fn m27_nloths_hungry_face_two_combat_counter() { test_nloths_hungry_face(); }

fn test_spirit_poop() {
    let run = settled(vec![RelicId::SpiritPoop], BTreeMap::new());
    let before = format!("{:?}", run.state);
    assert_eq!(run_laws::score_adjustment(&run.state), -1, "full-run score adjustment is −1");
    assert_eq!(format!("{:?}", run.state), before, "score evaluation changes no combat or reward state");
}
#[test]
fn m27_spirit_poop_score_minus_one() { test_spirit_poop(); }
#[test]
fn m27_spirit_poop_boundary() { test_spirit_poop(); }

fn test_ssserpent_head() {
    let mut run = settled(vec![RelicId::SsserpentHead], BTreeMap::new());
    fire(&mut run, RunItemEvent::RoomEntered { kind: RoomViewKind::Event(crate::content::EventId::Joust) });
    assert_eq!(run.state.gold, 50, "actual EventRoom entry gains 50 gold");
    fire(&mut run, RunItemEvent::RoomEntered { kind: RoomViewKind::Map });
    assert_eq!(run.state.gold, 50, "Unknown resolving to a non-Event gains 0");
}
#[test]
fn m27_ssserpent_head_event_room_gold_fifty() { test_ssserpent_head(); }
#[test]
fn m27_ssserpent_head_boundary() { test_ssserpent_head(); }

#[cfg(feature = "m27-diagnostics")]
fn m27_review_move_to_floor(run: &mut Run, floor: u32) {
    let target = run.state.map.as_ref().unwrap().nodes.iter()
        .find(|node| node.floor == floor).unwrap().id;
    run.step(&GameAction::Move(target)).unwrap();
}

#[cfg(feature = "m27-diagnostics")]
fn m27_review_choose_room_kind(run: &mut Run, predicate: impl Fn(&RoomChoiceKind) -> bool) {
    let choice = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind)).unwrap().id;
    run.step(&GameAction::Choose(choice)).unwrap();
}

#[cfg(feature = "m27-diagnostics")]
fn m27_review_act4_elite_reward_run(seed_byte: u8) -> Run {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    let supply = CompleteItemSupply {
        relics: vec![],
        potions: vec![None; 3],
        counters: BTreeMap::new(),
    };
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
    run.state.rng = crate::run::RunRng::new([seed_byte; 32], crate::combat::rng::Partition::Test, 0);
    m27_review_move_to_floor(&mut run, 51);
    m27_review_choose_room_kind(&mut run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    m27_review_move_to_floor(&mut run, 52);
    m27_review_choose_room_kind(&mut run, |kind| *kind == RoomChoiceKind::Leave);
    m27_review_move_to_floor(&mut run, 53);
    let count = run.decision().unwrap().combat.unwrap().enemies.len();
    for slot in 0..count { super::m9_boss::hit(&mut run, slot, 10_000); }
    run.settle_combat().unwrap();
    assert_eq!(run.decision().unwrap().room.unwrap().kind, RoomViewKind::Rewards);
    assert!(run.state.room_loop.as_ref().unwrap().active_rewards.is_some(),
        "the complete elite reward screen captures its generated card row");
    run
}

#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_review_i1_maw_bank_shop_purge_spends_gold_and_disables() {
    use crate::run::diagnostics::{self, CompleteItemSupply};
    let supply = CompleteItemSupply {
        relics: vec![RelicId::MawBank],
        potions: vec![None; 3],
        counters: BTreeMap::new(),
    };
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply).unwrap();
    m27_review_move_to_floor(&mut run, 51);
    m27_review_choose_room_kind(&mut run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    m27_review_move_to_floor(&mut run, 52);
    let before = run.state.gold;
    m27_review_choose_room_kind(&mut run, |kind| *kind == RoomChoiceKind::Purge);
    let select = run.decision().unwrap().candidates.into_iter()
        .find(|candidate| matches!(candidate.action, GameAction::Select(_))).unwrap().action;
    run.step(&select).unwrap();
    let after_purge = run.state.gold;
    assert!(after_purge < before, "the purge spent gold at the shop");
    assert_eq!(run.state.relic_counters.get(&RelicId::MawBank), Some(&1),
        "spending gold on the shop purge disables Maw Bank");
    m27_review_choose_room_kind(&mut run, |kind| *kind == RoomChoiceKind::Leave);
    m27_review_move_to_floor(&mut run, 53);
    assert_eq!(run.state.gold, after_purge,
        "catalog: Maw Bank no longer works once any gold is spent at a shop");
}

#[cfg(feature = "m27-diagnostics")]
#[test]
fn m27_review_m4_reward_screen_eggs_upgrade_matching_generated_cards() {
    use crate::items::lifecycle::RelicSource;
    let cases = [
        (0, RelicId::FrozenEgg,
            vec![(CardId::Warcry, 0), (CardId::FeelNoPain, 0), (CardId::DarkEmbrace, 0)],
            vec![(CardId::Warcry, 0), (CardId::FeelNoPain, 1), (CardId::DarkEmbrace, 1)]),
        (2, RelicId::MoltenEgg,
            vec![(CardId::PerfectedStrike, 0), (CardId::Combust, 0), (CardId::FeelNoPain, 0)],
            vec![(CardId::PerfectedStrike, 1), (CardId::Combust, 0), (CardId::FeelNoPain, 0)]),
        (0, RelicId::ToxicEgg,
            vec![(CardId::Warcry, 0), (CardId::FeelNoPain, 0), (CardId::DarkEmbrace, 0)],
            vec![(CardId::Warcry, 1), (CardId::FeelNoPain, 0), (CardId::DarkEmbrace, 0)]),
    ];
    for (seed, egg, before, expected) in cases {
        let mut run = m27_review_act4_elite_reward_run(seed);
        let cards = |run: &Run| run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap()
            .cards.choices.iter().filter_map(|choice| match choice.kind {
                RoomChoiceKind::Card { id, upgrades } => Some((id, upgrades)),
                _ => None,
            }).collect::<Vec<_>>();
        assert_eq!(cards(&run), before, "fixed generated reward cards for seed {seed}");
        crate::items::obtain_relic(&mut run.state, egg, RelicSource::Reward).unwrap();
        assert_eq!(cards(&run), expected,
            "obtaining {egg:?} upgrades every matching unupgraded card in active_rewards");
    }
}
