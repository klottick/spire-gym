#![cfg(feature = "m27-diagnostics")]
//! M27 plan Task X2, lane X, key m27_supplied.
//! Four literal supplied Act 4 paths through campfire, shop, elite rewards, and the Heart.

use super::super::*;
use crate::{
    combat::{CombatAction, Outcome},
    content::{CardType, Character, PotionId, RelicId},
    run::{diagnostics::{self, CompleteItemSupply}, GameAction, RunTerminal},
};
use std::collections::BTreeMap;

// Phase A's shared combat set (sets.md) and its Appendix A trigger citations:
// design Appendix A:1045-1046,1049-1053,1065,1067,1070,1078,1124,1126,1128-1129,
// 1143,1147-1148,1196,1207,1209,1216.
const COMBAT_PACKAGE: &[RelicId] = &[
    RelicId::Akabeko, RelicId::Anchor, RelicId::BagOfMarbles, RelicId::BagOfPreparation,
    RelicId::BronzeScales, RelicId::TheBoot, RelicId::OddlySmoothStone, RelicId::Orichalcum,
    RelicId::Vajra, RelicId::PreservedInsect, RelicId::SlingOfCourage, RelicId::DuVuDoll,
    RelicId::FossilizedHelix, RelicId::ThreadAndNeedle, RelicId::GremlinVisage,
    RelicId::RedMask, RelicId::MutagenicStrength, RelicId::TungstenRod, RelicId::Ginger,
    RelicId::Turnip,
];

// Literal 30 shared potion identities plus the three class identities in design §4.3
// (design:198-209) and rarity rows in Appendix B (design:1227-1268).
const COMMON_POTIONS: &[PotionId] = &[
    PotionId::FirePotion, PotionId::BlockPotion, PotionId::EnergyPotion,
    PotionId::StrengthPotion, PotionId::SwiftPotion, PotionId::WeakPotion,
    PotionId::FearPotion, PotionId::DexterityPotion, PotionId::AttackPotion,
    PotionId::SkillPotion, PotionId::PowerPotion, PotionId::ColorlessPotion,
    PotionId::FlexPotion, PotionId::SpeedPotion, PotionId::BlessingOfTheForge,
    PotionId::ExplosivePotion, PotionId::BloodPotion, PotionId::PoisonPotion,
    PotionId::FocusPotion, PotionId::BottledMiracle,
];
const UNCOMMON_POTIONS: &[PotionId] = &[
    PotionId::AncientPotion, PotionId::RegenPotion, PotionId::LiquidBronze,
    PotionId::EssenceOfSteel, PotionId::GamblersBrew, PotionId::LiquidMemories,
    PotionId::DuplicationPotion, PotionId::DistilledChaos, PotionId::Elixir,
    PotionId::CunningPotion, PotionId::PotionOfCapacity, PotionId::StancePotion,
];
const RARE_POTIONS: &[PotionId] = &[
    PotionId::FruitJuice, PotionId::SmokeBomb, PotionId::CultistPotion,
    PotionId::SneckoOil, PotionId::FairyPotion, PotionId::EntropicBrew,
    PotionId::HeartOfIron, PotionId::GhostInAJar, PotionId::EssenceOfDarkness,
    PotionId::Ambrosia,
];

// The 16 shared Shop-tier rows and one class row per character are the accepted literal
// list in sets.md; design catalog tier source is crates/spire-core/src/content/items.rs:47-78,165-179.
const SHOP_SHARED: &[RelicId] = &[
    RelicId::ChemicalX, RelicId::TheAbacus, RelicId::Cauldron,
    RelicId::ClockworkSouvenir, RelicId::DollysMirror, RelicId::FrozenEye,
    RelicId::HandDrill, RelicId::MedicalKit, RelicId::MembershipCard,
    RelicId::OrangePellets, RelicId::Orrery, RelicId::PrismaticShard,
    RelicId::SlingOfCourage, RelicId::StrangeSpoon, RelicId::Toolbox,
    RelicId::LeesWaffle,
];

#[derive(Clone, Copy, Debug)]
struct Case {
    name: &'static str,
    character: Character,
}

#[derive(Default)]
struct Checks {
    failures: Vec<String>,
    blockers: Vec<String>,
}

#[derive(Clone, Default)]
struct RescueStep {
    hp_after: u32,
    max_hp_after: u32,
    potions_before: Vec<Option<PotionId>>,
    potions_after: Vec<Option<PotionId>>,
    tail_before: i32,
    tail_after: i32,
    action_was_potion_use: bool,
}

#[derive(Default)]
struct RescueTrace {
    fairy: Option<RescueStep>,
    lizard_tail: Option<RescueStep>,
    order: Vec<&'static str>,
}

fn note_defect_rescue(
    trace: &mut RescueTrace,
    potions_before: Vec<Option<PotionId>>,
    tail_before: i32,
    action: &GameAction,
    run: &Run,
) {
    let potions_after = run.state.combat.as_ref().and_then(|combat| combat.item_context())
        .map(|items| items.potions().to_vec())
        .unwrap_or_else(|| run.state.potions.clone());
    let tail_after = run.state.combat.as_ref().and_then(|combat| combat.item_context())
        .map(|items| items.counter(RelicId::LizardTail))
        .or_else(|| run.state.relic_counters.get(&RelicId::LizardTail).copied())
        .unwrap_or(0);
    let (hp_after, max_hp_after) = run.state.combat.as_ref().and_then(|combat| combat.public_full().ok())
        .map(|public| (public.player.hp.max(0) as u32, public.player.max_hp as u32))
        .unwrap_or((run.state.hp, run.state.max_hp));
    let action_was_potion_use = matches!(action, GameAction::Combat(CombatAction::UsePotion { .. }));
    let step = RescueStep {
        hp_after,
        max_hp_after,
        potions_before: potions_before.clone(),
        potions_after: potions_after.clone(),
        tail_before,
        tail_after,
        action_was_potion_use,
    };
    let fairy_before = potions_before.iter().any(|potion| *potion == Some(PotionId::FairyPotion));
    let fairy_after = potions_after.iter().any(|potion| *potion == Some(PotionId::FairyPotion));
    if trace.fairy.is_none() && fairy_before && !fairy_after {
        trace.fairy = Some(step.clone());
        trace.order.push("FairyPotion");
    }
    if trace.lizard_tail.is_none() && tail_before == 0 && tail_after == 1 {
        trace.lizard_tail = Some(step);
        trace.order.push("LizardTail");
    }
}

fn expected_elite_run_counters(character: Character, relics: &[RelicId]) -> BTreeMap<RelicId, i32> {
    let mut counters = match character {
        Character::Ironclad => BTreeMap::from([(RelicId::NeowsLament, -2)]),
        Character::Silent => BTreeMap::from([(RelicId::NeowsLament, -2), (RelicId::MawBank, 1)]),
        Character::Defect | Character::Watcher => BTreeMap::from([(RelicId::NeowsLament, -2)]),
    };
    if relics.contains(&RelicId::HappyFlower) {
        // The elite reward is acquired after its combat; its equip counter starts at 0.
        counters.insert(RelicId::HappyFlower, 0);
    }
    counters
}

fn expected_heart_run_counters(character: Character, relics: &[RelicId]) -> BTreeMap<RelicId, i32> {
    let mut counters = match character {
        Character::Ironclad => BTreeMap::from([(RelicId::NeowsLament, -2)]),
        Character::Silent => BTreeMap::from([(RelicId::NeowsLament, -2), (RelicId::MawBank, 1)]),
        Character::Defect => BTreeMap::from([(RelicId::NeowsLament, -2), (RelicId::LizardTail, 1)]),
        Character::Watcher => BTreeMap::from([(RelicId::NeowsLament, -2)]),
    };
    if relics.contains(&RelicId::HappyFlower) {
        counters.insert(RelicId::HappyFlower, 2);
    }
    counters
}

fn check_defect_rescues(checks: &mut Checks, test: &str, trace: &RescueTrace) {
    let source = "Main-8 C4; Phase B ruling 2 C4; design §9.3:712-720; combat/tests/m27_rescue_escape.rs:25-58";
    expect_true(checks, test, "first lethal step restores positive HP with Fairy and leaves Lizard Tail unused",
        trace.fairy.as_ref().is_some_and(|step| step.hp_after > 0 && step.tail_before == 0 && step.tail_after == 0),
        "combat/effects.rs:373-388; items/mod.rs:125-135; items/relic_effects/accepted_law.rs:41-47", source);
    expect_true(checks, test, "later lethal step uses Lizard Tail and leaves positive HP",
        trace.lizard_tail.as_ref().is_some_and(|step| step.hp_after > 0 && step.tail_after == 1),
        "combat/effects.rs:378-388; items/relic_effects/accepted_law.rs:41-47", source);
    expect(checks, test, "Defect rescue order is Fairy Potion then Lizard Tail",
        trace.order.clone(), vec!["FairyPotion", "LizardTail"], "combat/effects.rs:373-388",
        source);
    if let Some(step) = &trace.fairy {
        let max_hp_limit = step.max_hp_after * 60 / 100;
        let mut expected_potions = step.potions_before.clone();
        if let Some(slot) = expected_potions.iter().position(|potion| *potion == Some(PotionId::FairyPotion)) {
            expected_potions[slot] = None;
        }
        expect_true(checks, test, "Fairy post-step HP is at most 60 percent of current max HP",
            step.hp_after <= max_hp_limit, "combat/effects.rs:373-388; items/mod.rs:125-135",
            "Phase B ruling 2 C4; design §9.3:712-720; combat/tests/m27_rescue_escape.rs:48-52");
        expect(checks, test, "Fairy rescue empties only its potion slot",
            step.potions_after.clone(), expected_potions, "items/mod.rs:125-135",
            "Phase B ruling 2 C4; design §9.3:712-720");
        expect_true(checks, test, "Fairy rescue does not invoke manual potion-use hooks",
            !step.action_was_potion_use, "items/mod.rs:125-135; combat/effects.rs:373-388",
            "Phase B ruling 2 C4; combat/tests/m15_silent_items.rs:484-495");
    }
    if let Some(step) = &trace.lizard_tail {
        expect_true(checks, test, "Lizard Tail post-step HP is at most half of current max HP",
            step.hp_after <= step.max_hp_after / 2, "items/relic_effects/accepted_law.rs:41-47",
            "Phase B ruling 2 C4; design §9.3:712-720; combat/tests/m27_rescue_escape.rs:48-52");
        expect(checks, test, "Lizard Tail step leaves potion slots unchanged",
            step.potions_after.clone(), step.potions_before.clone(), "items/relic_effects/accepted_law.rs:41-47",
            "Phase B ruling 2 C4; design §9.3:712-720");
        expect_true(checks, test, "Lizard Tail rescue does not invoke manual potion-use hooks",
            !step.action_was_potion_use, "items/relic_effects/accepted_law.rs:41-47",
            "Phase B ruling 2 C4; combat/tests/m15_silent_items.rs:484-495");
    }
}

fn expect<T: std::fmt::Debug + PartialEq>(
    checks: &mut Checks,
    test: &str,
    assertion: &str,
    actual: T,
    expected: T,
    owner: &str,
    source: &str,
) {
    if actual != expected {
        checks.failures.push(format!(
            "RED test={test} assertion={assertion}; actual={actual:?}; expected={expected:?}; suspected owner={owner}; source={source}"
        ));
    }
}

fn expect_true(
    checks: &mut Checks,
    test: &str,
    assertion: &str,
    actual: bool,
    owner: &str,
    source: &str,
) {
    expect(checks, test, assertion, actual, true, owner, source);
}

fn finish_checks(test: &str, checks: Checks) {
    assert!(checks.failures.is_empty() && checks.blockers.is_empty(),
        "{test} accumulated assertions:\n{}\nBLOCKERS:\n{}",
        checks.failures.join("\n"), checks.blockers.join("\n"));
}

fn class_shop_relic(character: Character) -> RelicId {
    match character {
        Character::Ironclad => RelicId::Brimstone,
        Character::Silent => RelicId::TwistedFunnel,
        Character::Defect => RelicId::RunicCapacitor,
        Character::Watcher => RelicId::Melange,
    }
}

fn shop_literal_members(character: Character) -> Vec<RelicId> {
    SHOP_SHARED.iter().copied().chain([class_shop_relic(character)]).collect()
}

fn literal_potion_price(potion: PotionId, character: Character) -> Option<u32> {
    if COMMON_POTIONS.contains(&potion)
        && (!matches!(potion, PotionId::BloodPotion | PotionId::Elixir)
            || character == Character::Ironclad)
        && (potion != PotionId::PoisonPotion && potion != PotionId::CunningPotion
            || character == Character::Silent)
        && (potion != PotionId::FocusPotion && potion != PotionId::PotionOfCapacity
            || character == Character::Defect)
        && (potion != PotionId::BottledMiracle && potion != PotionId::StancePotion
            || character == Character::Watcher)
        && (potion != PotionId::HeartOfIron || character == Character::Ironclad)
        && (potion != PotionId::GhostInAJar || character == Character::Silent)
        && (potion != PotionId::EssenceOfDarkness || character == Character::Defect)
        && (potion != PotionId::Ambrosia || character == Character::Watcher)
    {
        return Some(50);
    }
    if UNCOMMON_POTIONS.contains(&potion)
        && (potion != PotionId::Elixir || character == Character::Ironclad)
        && (potion != PotionId::CunningPotion || character == Character::Silent)
        && (potion != PotionId::PotionOfCapacity || character == Character::Defect)
        && (potion != PotionId::StancePotion || character == Character::Watcher)
    {
        return Some(75);
    }
    if RARE_POTIONS.contains(&potion)
        && (potion != PotionId::HeartOfIron || character == Character::Ironclad)
        && (potion != PotionId::GhostInAJar || character == Character::Silent)
        && (potion != PotionId::EssenceOfDarkness || character == Character::Defect)
        && (potion != PotionId::Ambrosia || character == Character::Watcher)
    {
        return Some(100);
    }
    None
}

fn literal_character_potions(character: Character) -> Vec<PotionId> {
    COMMON_POTIONS.iter().chain(UNCOMMON_POTIONS).chain(RARE_POTIONS)
        .copied().filter(|potion| literal_potion_price(*potion, character).is_some()).collect()
}

fn supplied_relics(character: Character) -> Vec<RelicId> {
    let mut relics = COMBAT_PACKAGE.to_vec();
    let additions: &[RelicId] = match character {
        Character::Ironclad => &[
            RelicId::BlackBlood, RelicId::AncientTeaSet, RelicId::DreamCatcher,
            RelicId::Girya, RelicId::RegalPillow, RelicId::EternalFeather,
            RelicId::PeacePipe, RelicId::Shovel, RelicId::MeatOnTheBone,
            RelicId::FaceOfCleric, RelicId::MagicFlower, RelicId::GoldenIdol,
            RelicId::BloodyIdol, RelicId::CeramicFish, RelicId::MoltenEgg,
            RelicId::CursedKey, RelicId::MarkOfPain, RelicId::NeowsLament,
            RelicId::Pantograph, RelicId::WhiteBeastStatue,
        ],
        Character::Silent => &[
            RelicId::RingOfTheSerpent, RelicId::CoffeeDripper, RelicId::MawBank,
            RelicId::MealTicket, RelicId::MembershipCard, RelicId::TheCourier,
            RelicId::SmilingMask, RelicId::BlackStar, RelicId::QuestionCard,
            RelicId::FrozenEgg, RelicId::WhiteBeastStatue, RelicId::ToyOrnithopter,
            RelicId::VelvetChoker, RelicId::NeowsLament, // Phase B ruling 2 C9; Appendix A:1211.
        ],
        Character::Defect => &[
            RelicId::FrozenCore, RelicId::FusionHammer, RelicId::PhilosophersStone,
            RelicId::PrismaticShard, RelicId::SingingBowl, RelicId::QuestionCard,
            RelicId::WhiteBeastStatue, RelicId::LizardTail, RelicId::SacredBark,
            RelicId::NeowsLament, // Phase B ruling 2 C9; Appendix A:1211.
        ],
        Character::Watcher => &[
            RelicId::HolyWater, RelicId::BustedCrown, RelicId::Ectoplasm,
            RelicId::RunicDome, RelicId::SneckoEye, RelicId::MarkOfBloom,
            RelicId::SlaversCollar, RelicId::WhiteBeastStatue, RelicId::ToxicEgg,
            RelicId::NeowsLament, // Phase B ruling 2 C9; Appendix A:1211.
        ],
    };
    relics.extend_from_slice(additions);
    relics
}

fn complete_supply(character: Character) -> CompleteItemSupply {
    let potions = match character {
        Character::Ironclad | Character::Watcher => vec![Some(PotionId::FirePotion), None, None],
        Character::Silent => vec![Some(PotionId::FirePotion), Some(PotionId::SwiftPotion), None],
        Character::Defect => vec![Some(PotionId::FirePotion), Some(PotionId::FairyPotion), None],
    };
    // Main-8 C9 supplies Neow's Lament at 1 in every set so all four reach the Heart;
    // this is the accepted first-combat boundary (design §7.5:571; m10_content.rs:327-374 pattern).
    let counters = BTreeMap::from([(RelicId::NeowsLament, 1)]);
    CompleteItemSupply { relics: supplied_relics(character), potions, counters }
}

fn move_to_floor(run: &mut Run, floor: u32) {
    let node = run.state.map.as_ref().unwrap().nodes.iter()
        .find(|node| node.floor == floor).unwrap().id;
    run.step(&GameAction::Move(node)).unwrap_or_else(|error| panic!(
        "route move to floor {floor}: {error:?}; current floor={} room={} screen={:?} map_current={:?} reward_parent={} active_rewards={} fault={:?}",
        run.state.floor, run.state.room, run.state.room_loop.as_ref().map(|room| room.screen.kind),
        run.state.map.as_ref().and_then(|map| map.current),
        run.state.room_loop.as_ref().is_some_and(|room| room.reward_parent.is_some()),
        run.state.room_loop.as_ref().is_some_and(|room| room.active_rewards.is_some()), run.state.fault));
}

fn choose_kind(run: &mut Run, predicate: impl Fn(&RoomChoiceKind) -> bool) -> RoomChoiceView {
    let row = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind))
        .unwrap_or_else(|| panic!("no offered room row matching the scripted choice"));
    run.step(&GameAction::Choose(row.id)).unwrap_or_else(|error| panic!("legal choice {:?}: {error:?}", row.kind));
    row
}

fn first_selection_action(run: &Run) -> GameAction {
    run.decision().unwrap().candidates.into_iter()
        .find(|candidate| matches!(candidate.action, GameAction::Select(_)))
        .unwrap_or_else(|| panic!("the open selection exposes a legal Select action")).action
}

fn resolve_equip_continuation(run: &mut Run) {
    for _ in 0..32 {
        let decision = run.decision().unwrap_or_else(|error| panic!("equip continuation decision: {error:?}"));
        if decision.selection.is_some() {
            let action = first_selection_action(run);
            run.step(&action).unwrap_or_else(|error| panic!("legal first equip selection: {error:?}"));
            continue;
        }
        let Some(room) = decision.room else { break };
        if room.kind == RoomViewKind::CardReward {
            let skip = room.choices.iter().find(|row|
                row.kind == RoomChoiceKind::Skip && row.status == RoomChoiceStatus::Offered)
                .unwrap_or_else(|| panic!("equip card offer has legal Skip"));
            run.step(&GameAction::Choose(skip.id)).unwrap();
            continue;
        }
        if room.kind == RoomViewKind::Rewards {
            let proceed = room.choices.iter().find(|row|
                row.kind == RoomChoiceKind::Proceed && row.status == RoomChoiceStatus::Offered)
                .unwrap_or_else(|| panic!("equip offer screen has legal Proceed"));
            run.step(&GameAction::Choose(proceed.id)).unwrap();
            continue;
        }
        break;
    }
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen.kind, RoomViewKind::Shop,
        "the equip continuation returns to the floor-52 Shop");
}

fn pool_contains(run: &Run, id: RelicId) -> bool {
    let pools = run.state.relic_pools.as_ref().unwrap();
    pools.common.contains(&id) || pools.uncommon.contains(&id) || pools.rare.contains(&id)
        || pools.shop.contains(&id) || pools.boss.contains(&id)
}

fn shop_discounted_price(character: Character, base: u32) -> u32 {
    if character == Character::Silent {
        // sets.md §Potion supply and shop expectations: Courier ×0.8 then Membership ×0.5,
        // with each step rounded separately; Appendix A:1085,1191.
        let courier = (base * 80 + 50) / 100;
        (courier * 50 + 50) / 100
    } else {
        base
    }
}

fn assert_shop_relic_equip(
    checks: &mut Checks,
    test: &str,
    relic: RelicId,
    hp_before: u32,
    max_before: u32,
    deck_before: &[crate::run::CardInstance],
    potions_before: &[Option<PotionId>],
    run: &Run,
) {
    let hp_delta = run.state.hp.saturating_sub(hp_before);
    let max_delta = run.state.max_hp.saturating_sub(max_before);
    match relic {
        RelicId::LeesWaffle => {
            // Appendix A:1200, max HP +7 then full heal through increaseMaxHp/heal law (design §7.1:480-490).
            expect(checks, test, "Lee's Waffle equip max HP", max_delta, 7, "items/relic_effects/structural.rs", "design Appendix A:1200; design §7.1:480-490");
            // Main-10 I3: the full heal goes through the one heal law, so an owned Mark of the Bloom prevents it
            // (LEES_WAFFLE notes); was an unconditional heal to the new maximum.
            let expected_hp = if run.state.relics.contains(&RelicId::MarkOfBloom) { hp_before } else { run.state.max_hp };
            expect(checks, test, "Lee's Waffle equip heals to new maximum through the heal law", run.state.hp, expected_hp, "items/relic_effects/run.rs", "design §9.1; LEES_WAFFLE notes");
        }
        RelicId::DollysMirror => {
            // Appendix A:1059, design §7.1:484-495: copy one selected permanent card.
            expect(checks, test, "Dolly's Mirror duplicates one selected card", run.state.deck.len(), deck_before.len() + 1, "items/relic_effects/structural.rs", "design §7.1:484-495; Appendix A equipment row");
        }
        RelicId::Cauldron | RelicId::Orrery => {
            // These equip-only relics open offers; this path proceeds past those offers without claiming them.
            expect(checks, test, "equip-only shop offer leaves supplied potion slots unchanged", run.state.potions.clone(),
                potions_before.to_vec(), "run/rooms/rewards.rs:156-165,211-276", "design §7.1:488-498; Main-8 C8");
        }
        _ => {
            expect(checks, test, "non-equip shop relic leaves HP unchanged", hp_delta, 0, "items/lifecycle.rs:66-72", "design Appendix A trigger/effect rows; Main-8 C8");
            expect(checks, test, "non-equip shop relic leaves max HP unchanged", max_delta, 0, "items/lifecycle.rs:66-72", "design Appendix A trigger/effect rows; Main-8 C8");
            expect(checks, test, "non-equip shop relic leaves deck unchanged", run.state.deck.clone(), deck_before.to_vec(), "items/lifecycle.rs:66-72", "design Appendix A trigger/effect rows; Main-8 C8");
        }
    }
}

fn shop_and_campfire(case: Case, run: &mut Run, checks: &mut Checks, expected_relics: &mut Vec<RelicId>) -> Option<(u32, u32)> {
    let test = case.name;
    let source_fixture = "Main-8 C6; run/diagnostics/m27.rs:11-50";
    expect(checks, test, "supplied fixture begins at 30/80 before its first post-construction Run::step",
        (run.state.hp, run.state.max_hp), (30, 80), "run/diagnostics/m27.rs:11-50", source_fixture);
    move_to_floor(run, 51);

    let (entered_hp, entered_gold) = match case.character {
        Character::Ironclad => (36, 999),
        Character::Silent => (30, 1011), // Maw Bank's literal +12 floor-51 gain.
        Character::Defect => (30, 999),
        Character::Watcher => (30, 999),
    };
    // Eternal Feather heals 3 × floor(10 deck cards / 5) on Rest-room entry; the supplied
    // starter deck is 10 cards (run/diagnostics/m27.rs:14-20); design Appendix A:1088.
    expect(checks, test, "floor-51 room-entry HP", run.state.hp, entered_hp, "items/relic_effects/run.rs:131-134", "Main-8 C6; design Appendix A:1088");
    // Main-8 C7 expects Maw Bank +12 on each of floors 51 and 52; Act4 route currently only
    // commits the ledger event and RoomEntered (run/rooms/act4.rs:132-153).
    expect(checks, test, "floor-51 gold after Maw Bank floor entry", run.state.gold, entered_gold,
        "run/rooms/act4.rs:132-153", "Main-8 C7; design Appendix A:1062; run/tests/m27_relics_run.rs:131-138");
    if case.character == Character::Ironclad {
        expect(checks, test, "Ancient Tea Set arms on floor-51 campfire entry",
            run.state.relic_counters.get(&RelicId::AncientTeaSet).copied().unwrap_or(0), 1,
            "items/relic_effects/run.rs:119-121", "design Appendix A:1047; run/tests/m27_relics_run.rs:44-52");
    }

    let campfire = run.decision().unwrap().room.unwrap();
    expect(checks, test, "floor 51 is a Campfire", campfire.kind, RoomViewKind::Campfire,
        "run/rooms/act4.rs:133-153", "design §15.4:931-944");
    let rest_offered = campfire.choices.iter().any(|row| row.kind == RoomChoiceKind::Rest { heal: 24 }
        && row.status == RoomChoiceStatus::Offered);
    let smith_offered = campfire.choices.iter().any(|row| row.kind == RoomChoiceKind::Smith
        && row.status == RoomChoiceStatus::Offered);
    match case.character {
        Character::Silent => {
            expect(checks, test, "Coffee Dripper removes Rest", rest_offered, false,
                "items/relic_effects/run.rs", "design Appendix A:1156; design §9.2:701-710");
            expect_true(checks, test, "Coffee Dripper leaves Smith available", smith_offered,
                "items/relic_effects/run.rs", "design Appendix A:1156; sets.md Set 2 path");
            choose_kind(run, |kind| *kind == RoomChoiceKind::Smith);
            let before = run.state.deck.clone();
            let action = first_selection_action(run);
            run.step(&action).unwrap_or_else(|error| panic!("legal floor-51 Smith selection: {error:?}"));
            expect(checks, test, "Smith upgrades one selected permanent card",
                run.state.deck.iter().filter(|card| card.upgrades > 0).count(),
                before.iter().filter(|card| card.upgrades > 0).count() + 1,
                "run/rooms/rest.rs:18-22", "sets.md Set 2 path; floor-51 Smith is the scripted action");
        }
        _ => {
            expect_true(checks, test, "scripted Rest remains offered", rest_offered,
                "run/rooms/rest.rs:4-16", "Main-8 C6; sets.md floor-51 scripts");
            if case.character == Character::Defect {
                expect(checks, test, "Fusion Hammer removes Smith", smith_offered, false,
                    "items/relic_effects/run.rs", "design Appendix A:1161; design §9.2:701-710");
            }
            if case.character == Character::Watcher {
                expect_true(checks, test, "Mark of the Bloom does not remove Rest", rest_offered,
                    "items/relic_effects/run.rs", "design Appendix A:1208; Main-8 C4");
            }
            choose_kind(run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
            if case.character == Character::Ironclad {
                let child = run.decision().unwrap().room.unwrap();
                expect(checks, test, "Dream Catcher opens its one campfire card reward",
                    child.kind, RoomViewKind::Rewards, "run/rooms/rest.rs:33-41", "design Appendix A:1058; sets.md Set 1 path");
                expect(checks, test, "Dream Catcher offers one CardReward row",
                    child.choices.iter().filter(|row| row.kind == RoomChoiceKind::CardReward).count(), 1,
                    "run/rooms/rest.rs:33-41", "design Appendix A:1058; sets.md Set 1 path");
                choose_kind(run, |kind| *kind == RoomChoiceKind::CardReward);
                expect_true(checks, test, "Dream Catcher card child offers Skip",
                    run.decision().unwrap().room.unwrap().choices.iter().any(|row| row.kind == RoomChoiceKind::Skip),
                    "run/rooms/rest.rs:33-41", "design Appendix A:1058; sets.md Set 1 path");
                choose_kind(run, |kind| *kind == RoomChoiceKind::Skip);
                choose_kind(run, |kind| *kind == RoomChoiceKind::Proceed);
            }
        }
    }

    let after_campfire_hp = match case.character {
        Character::Ironclad => 75, // HP 30 + Feather 6 + Rest (24 + Pillow 15), capped at 80.
        Character::Silent => 30,  // Smith, and Coffee Dripper removes Rest.
        Character::Defect => 54,  // Rest heals 30% of 80 = 24.
        Character::Watcher => 30, // Mark of the Bloom blocks the Rest heal.
    };
    expect(checks, test, "floor-51 campfire action HP", run.state.hp, after_campfire_hp,
        "run/rooms/rest.rs:18-43; items/relic_effects/run.rs:130-134", "Main-8 C6; design §9.1:690-699");
    expect(checks, test, "floor-51 Campfire has no unexpected gold gain", run.state.gold, entered_gold,
        "run/rooms/rest.rs:18-43", "Main-8 C7; accepted Rest script");
    if case.character == Character::Ironclad {
        expect(checks, test, "Girya remains unused after choosing Rest",
            run.state.relic_counters.get(&RelicId::Girya).copied().unwrap_or(0), 0,
            "run/rooms/rest.rs:45-49", "sets.md Set 1 counter row; run/tests/m27_relics_run.rs:435-448");
    }
    eprintln!("X2 TRACE {} floor51 entered hp={entered_hp} gold={} after-campfire hp={} gold={} counters={:?}",
        case.name, run.state.gold, run.state.hp, run.state.gold, run.state.relic_counters);

    move_to_floor(run, 52);
    let shop_before = run.decision().unwrap().room.unwrap();
    expect(checks, test, "floor 52 is a Shop", shop_before.kind, RoomViewKind::Shop,
        "run/rooms/act4.rs:133-153", "design §15.4:931-944");
    // Main-8 C7's two +12 Maw Bank floor entries should produce 1023 from 999 (every room owner
    // dispatches FloorEntered; X2 run 1 found the Act 4 owner missing it).
    let expected_shop_entry_gold = if case.character == Character::Silent { 1023 } else { 999 };
    expect(checks, test, "floor-52 gold on Shop entry", run.state.gold, expected_shop_entry_gold,
        "run/rooms/act4.rs:132-153", "Main-8 C7; design Appendix A:1062; run/tests/m27_relics_run.rs:131-138");
    let expected_shop_hp = match case.character {
        Character::Silent => 45, // Meal Ticket +15, capped at 80.
        _ => after_campfire_hp,
    };
    expect(checks, test, "floor-52 Shop entry HP", run.state.hp, expected_shop_hp,
        "items/relic_effects/run.rs:129", "Main-8 C6; design §9.1:690-699; Appendix A:1063");

    let literal_shop = shop_literal_members(case.character);
    let relic_row = shop_before.choices.iter().find(|row| row.status == RoomChoiceStatus::Offered
        && matches!(row.kind, RoomChoiceKind::Relic(_))).cloned()
        .unwrap_or_else(|| panic!("floor-52 shop offers its one relic row"));
    let RoomChoiceKind::Relic(shop_relic) = relic_row.kind else { unreachable!() };
    expect_true(checks, test, "shop relic belongs to literal Shop-tier union",
        literal_shop.contains(&shop_relic), "run/rooms/shop.rs:106-118", "sets.md Shop-tier literal list; design §11.4:799-804");
    expect(checks, test, "shop relic base/discounted price", relic_row.price.unwrap_or(0),
        shop_discounted_price(case.character, 150), "run/rooms/shop.rs:106-151", "design §11.4:801-804; Main-8 C7");
    expect_true(checks, test, "shop relic is removed from its pool when offered",
        !pool_contains(run, shop_relic), "items/relic_pools.rs:67-68; run/rooms/shop.rs:109-118", "design §7.4:526-547; design §11.4:801");
    let potion_row = shop_before.choices.iter().find(|row| row.status == RoomChoiceStatus::Offered
        && matches!(row.kind, RoomChoiceKind::Potion(_))).cloned()
        .unwrap_or_else(|| panic!("floor-52 shop offers its named initial potion row"));
    let RoomChoiceKind::Potion(shop_potion) = potion_row.kind else { unreachable!() };
    let base_potion_price = literal_potion_price(shop_potion, case.character);
    expect_true(checks, test, "shop potion belongs to literal 33-potion character union",
        base_potion_price.is_some(), "run/rooms/shop.rs:98-104", "design §4.3:198-209; design Appendix B:1227-1268");
    let base_potion_price = base_potion_price.unwrap_or(0);
    expect(checks, test, "shop potion price follows literal rarity and discount rules",
        potion_row.price.unwrap_or(0), shop_discounted_price(case.character, base_potion_price),
        "run/rooms/shop.rs:92-104,119-151", "design §11.4:802-804; Appendix B:1227-1268");

    let hp_before_shop_relic = run.state.hp;
    let max_before_shop_relic = run.state.max_hp;
    let deck_before_shop_relic = run.state.deck.clone();
    let potions_before_shop_relic = run.state.potions.clone();
    run.step(&GameAction::Choose(relic_row.id)).unwrap_or_else(|error| panic!("legal Shop relic row: {error:?}"));
    expected_relics.push(shop_relic);
    resolve_equip_continuation(run);
    expect_true(checks, test, "purchased Shop relic is owned once",
        run.state.relics.iter().filter(|&&id| id == shop_relic).count() == 1,
        "run/rooms/shop.rs:160-175; items/lifecycle.rs:49-72", "design §7.1:475-498; design §11.4:801-804");
    expect_true(checks, test, "consumed Shop relic is not offered again",
        !run.decision().unwrap().room.unwrap().choices.iter().any(|row|
            row.kind == RoomChoiceKind::Relic(shop_relic) && row.status == RoomChoiceStatus::Offered),
        "run/rooms/shop.rs:160-175", "design §7.4:532-535; design §11.4:801");
    assert_shop_relic_equip(checks, test, shop_relic, hp_before_shop_relic,
        max_before_shop_relic, &deck_before_shop_relic, &potions_before_shop_relic, run);

    let expected_relic_price = shop_discounted_price(case.character, 150);
    let expected_potion_price = shop_discounted_price(case.character, base_potion_price);
    let potion_row_now = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.id == potion_row.id)
        .unwrap_or_else(|| panic!("the named initial potion row remains in the Shop"));
    expect(checks, test, "named initial potion row retains its expected price after relic purchase",
        potion_row_now.price.unwrap_or(0), expected_potion_price,
        "run/rooms/shop.rs:231-247", "design §11.4:803-804");
    run.step(&GameAction::Choose(potion_row.id)).unwrap_or_else(|error| panic!("legal initial Shop potion row: {error:?}"));
    let potion_slot = run.state.potions.iter().position(|slot| *slot == Some(shop_potion));
    let expected_slot = match case.character { Character::Ironclad | Character::Watcher => 1, _ => 2 };
    expect(checks, test, "Shop potion occupies first empty slot", potion_slot, Some(expected_slot),
        "items/mod.rs:250-280; run/rooms/shop.rs:178-195", "Main-8 C5; design §8.2:610-622");
    let expected_gold_after_shop = if case.character == Character::Silent {
        1023_u32.saturating_sub(expected_relic_price).saturating_sub(expected_potion_price)
    } else {
        999_u32.saturating_sub(expected_relic_price).saturating_sub(expected_potion_price)
    };
    expect(checks, test, "gold after the two named Shop purchases", run.state.gold,
        expected_gold_after_shop, "run/rooms/shop.rs:160-195", "Main-8 C7; design §11.4:801-804");
    if case.character == Character::Silent {
        expect(checks, test, "Maw Bank disables after first Shop purchase",
            run.state.relic_counters.get(&RelicId::MawBank).copied().unwrap_or(0), 1,
            "items/relic_effects/run.rs:125-129", "Main-8 C7-C8; run/tests/m27_relics_run.rs:131-138");
    }
    let expected_slots = match case.character {
        Character::Ironclad | Character::Watcher => vec![Some(PotionId::FirePotion), Some(shop_potion), None],
        Character::Silent => vec![Some(PotionId::FirePotion), Some(PotionId::SwiftPotion), Some(shop_potion)],
        Character::Defect => vec![Some(PotionId::FirePotion), Some(PotionId::FairyPotion), Some(shop_potion)],
    };
    expect(checks, test, "potion slots after Shop purchase", run.state.potions.clone(), expected_slots,
        "items/mod.rs:250-280; run/rooms/shop.rs:178-195", "Main-8 C5; design §8.2:610-622");
    choose_kind(run, |kind| *kind == RoomChoiceKind::Leave);
    eprintln!("X2 TRACE {} floor52 hp={} gold={} shop_relic={shop_relic:?} relic_price={expected_relic_price} shop_potion={shop_potion:?} potion_price={expected_potion_price} slots={:?} counters={:?}",
        case.name, run.state.hp, run.state.gold, run.state.potions, run.state.relic_counters);
    Some((base_potion_price, expected_potion_price))
}

fn combat_policy_step(run: &mut Run) -> Option<GameAction> {
    let decision = run.decision().unwrap_or_else(|error| panic!("combat decision: {error:?}"));
    if let Some(candidate) = decision.candidates.iter().find(|candidate|
        matches!(candidate.action, GameAction::Combat(CombatAction::PlayCard { .. })))
    {
        return Some(candidate.action.clone());
    }
    // Resolve a mandatory pending card selection before applying the turn policy.
    // It is a legal Run::step continuation, not a choice to discard a potion.
    if let Some(candidate) = decision.candidates.iter().find(|candidate|
        matches!(candidate.action, GameAction::Combat(CombatAction::SelectCard { .. }))) {
        return Some(candidate.action.clone());
    }
    if let Some(candidate) = decision.candidates.iter().find(|candidate|
        candidate.action == GameAction::Combat(CombatAction::ConfirmSelection)) {
        return Some(candidate.action.clone());
    }
    decision.candidates.iter().find(|candidate|
        candidate.action == GameAction::Combat(CombatAction::EndTurn))
        .map(|candidate| candidate.action.clone())
}

fn combat_inventory(run: &Run) -> &crate::items::CombatItemContext {
    run.state.combat.as_ref().unwrap().item_context().unwrap()
}

fn combat_enemies(run: &Run) -> Vec<(i32, i32)> {
    run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter()
        .filter(|enemy| !enemy.replaced && !enemy.escaped)
        .map(|enemy| (enemy.hp, enemy.max_hp)).collect()
}

fn assert_elite_equip(
    checks: &mut Checks,
    test: &str,
    relic: RelicId,
    hp_before: u32,
    max_before: u32,
    deck_before: &[crate::run::CardInstance],
    potion_slots_before: &[Option<PotionId>],
    run: &Run,
) {
    let hp_delta = run.state.hp.saturating_sub(hp_before);
    let max_delta = run.state.max_hp.saturating_sub(max_before);
    match relic {
        RelicId::Strawberry => expect(checks, test, "Strawberry equip max HP", max_delta, 7,
            "items/relic_effects/structural.rs", "design Appendix A:1075; design §7.1:480-490"),
        RelicId::Pear => expect(checks, test, "Pear equip max HP", max_delta, 10,
            "items/relic_effects/structural.rs", "design Appendix A:1106; design §7.1:480-490"),
        RelicId::Mango => expect(checks, test, "Mango equip max HP", max_delta, 14,
            "items/relic_effects/structural.rs", "design Appendix A:1130; design §7.1:480-490"),
        RelicId::PotionBelt => {
            expect(checks, test, "Potion Belt appends two empty slots", run.state.potions.len(), potion_slots_before.len() + 2,
                "items/relic_effects/structural.rs", "design §8.1:601-608; design §7.1:480-490; Main-8 C8");
        }
        RelicId::DollysMirror => expect(checks, test, "Dolly's Mirror duplicates a selected card",
            run.state.deck.len(), deck_before.len() + 1, "items/relic_effects/structural.rs", "design §7.1:484-495; Main-8 C8"),
        RelicId::Cauldron | RelicId::Orrery | RelicId::TinyHouse => {
            expect_true(checks, test, "equip offer returned through legal Proceed without taking an extra potion",
                run.state.potions.len() >= potion_slots_before.len(), "run/rooms/rewards.rs:156-276", "design §7.1:488-498; Main-8 C8");
        }
        RelicId::WarPaint | RelicId::Whetstone => {
            let increased = run.state.deck.iter().zip(deck_before).filter(|(after, before)| after.upgrades > before.upgrades).count();
            expect_true(checks, test, "upgrade relic changes no more than two eligible cards", increased <= 2,
                "items/relic_effects/structural.rs", "design §7.1:484-490; Main-8 C8");
            if relic == RelicId::WarPaint {
                expect_true(checks, test, "War Paint upgrades only Skills", run.state.deck.iter().zip(deck_before)
                    .filter(|(after, before)| after.upgrades > before.upgrades)
                    .all(|(card, _)| crate::content::Catalog::standard().card(card.id).unwrap().card_type == CardType::Skill),
                    "items/relic_effects/structural.rs", "design §7.1:484-490; Main-8 C8");
            } else {
                expect_true(checks, test, "Whetstone upgrades only Attacks", run.state.deck.iter().zip(deck_before)
                    .filter(|(after, before)| after.upgrades > before.upgrades)
                    .all(|(card, _)| crate::content::Catalog::standard().card(card.id).unwrap().card_type == CardType::Attack),
                    "items/relic_effects/structural.rs", "design §7.1:484-490; Main-8 C8");
            }
        }
        RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado => {
            expect_true(checks, test, "bottle equip links an eligible permanent card", run.state.relic_cards.contains_key(&relic),
                "items/relic_effects/structural.rs", "design §7.1:484-495; Main-8 C8");
        }
        _ => {
            expect(checks, test, "elite relic with no equip effect leaves max HP unchanged", max_delta, 0,
                "items/lifecycle.rs:66-72", "design Appendix A; Main-8 C8");
            expect(checks, test, "elite relic with no equip effect leaves permanent deck unchanged",
                run.state.deck.clone(), deck_before.to_vec(), "items/lifecycle.rs:66-72", "design Appendix A; Main-8 C8");
            expect(checks, test, "elite relic with no equip effect leaves current HP unchanged", hp_delta, 0,
                "items/lifecycle.rs:66-72", "design Appendix A; Main-8 C8");
        }
    }
    let expected_max_gain = match relic {
        RelicId::Strawberry => 7,
        RelicId::Pear => 10,
        RelicId::Mango => 14,
        _ => 0,
    };
    if expected_max_gain > 0 {
        // §7.1/§9.1: max-HP equip gains heal by that amount through the normal capped heal law;
        // Mark of the Bloom blocks the heal but does not undo the maximum-HP increase.
        let expected_heal = if run.state.relics.contains(&RelicId::MarkOfBloom) { 0 } else { expected_max_gain };
        expect(checks, test, "max-HP relic equip heals by its literal increase through the heal law",
            hp_delta, expected_heal, "items/relic_effects/structural.rs; items/laws.rs",
            "design §7.1:480-490; design §9.1:692-699; Main-8 C8");
    }
}

fn claim_card_row(case: Case, run: &mut Run, checks: &mut Checks) {
    let test = case.name;
    let parent = run.decision().unwrap().room.unwrap();
    let rows = parent.choices.iter().filter(|row| row.kind == RoomChoiceKind::CardReward
        && row.status == RoomChoiceStatus::Offered).collect::<Vec<_>>();
    expect(checks, test, "elite has exactly one card-reward row", rows.len(), 1,
        "run/rooms/rewards.rs:316-321,429-446", "Main-8 C2; design §11.4:808");
    let Some(card_row) = rows.first() else { return };
    let row_id = card_row.id;
    run.step(&GameAction::Choose(row_id)).unwrap_or_else(|error| panic!("legal elite CardReward row: {error:?}"));
    let child = run.decision().unwrap().room.unwrap();
    let card_choices = child.choices.iter().filter_map(|row| match row.kind {
        RoomChoiceKind::Card { id, upgrades } => Some((id, upgrades)), _ => None,
    }).collect::<Vec<_>>();
    let expected_count = match case.character {
        Character::Ironclad => 3,
        Character::Silent | Character::Defect => 4,
        Character::Watcher => 1,
    };
    expect(checks, test, "elite card choice count follows Question Card/Busted Crown literal",
        card_choices.len(), expected_count, "run/rooms/rewards.rs:355-375,440-489",
        "Main-8 C2; design Appendix A:1107,1154; design §11.4:808");

    if case.character == Character::Defect {
        // C3: with one card row, take the Singing Bowl +2 Max HP option; increaseMaxHp heals by +2.
        expect_true(checks, test, "Defect elite card child offers Singing Bowl",
            child.choices.iter().any(|row| row.kind == RoomChoiceKind::SingingBowl),
            "run/rooms/rewards.rs:485-489,545-548", "Main-8 C3; design Appendix A:1110");
        let hp_before = run.state.hp;
        let max_before = run.state.max_hp;
        choose_kind(run, |kind| *kind == RoomChoiceKind::SingingBowl);
        expect(checks, test, "Singing Bowl increases Max HP by 2", run.state.max_hp - max_before, 2,
            "run/rooms/rewards.rs:545-548", "Main-8 C3; design Appendix A:1110; design §9.1:698-699");
        expect(checks, test, "Singing Bowl heal is 2 through increaseMaxHp", run.state.hp.saturating_sub(hp_before), 2,
            "run/mutations.rs:148-159; run/rooms/rewards.rs:545-548", "Main-8 C3; design §9.1:698-699");
        return;
    }

    let egg = match case.character {
        Character::Ironclad => Some((RelicId::MoltenEgg, CardType::Attack)),
        Character::Silent => Some((RelicId::FrozenEgg, CardType::Power)),
        Character::Watcher => Some((RelicId::ToxicEgg, CardType::Skill)),
        Character::Defect => None,
    };
    let chosen = if let Some((_, target_type)) = egg {
        card_choices.iter().find(|(id, _)| crate::content::Catalog::standard().card(*id)
            .is_some_and(|definition| definition.card_type == target_type)).copied()
            .or_else(|| card_choices.first().copied())
    } else { card_choices.first().copied() };
    let Some((card_id, offered_upgrades)) = chosen else {
        checks.failures.push(format!("RED test={test} assertion=elite card row has a legal card choice; actual=none; expected=one or more; suspected owner=run/rooms/rewards.rs:439-489; source=Main-8 C2-C3; design §11.4:808"));
        return;
    };
    let before_len = run.state.deck.len();
    let gold_before_obtain = run.state.gold;
    let hp_before_obtain = run.state.hp;
    run.step(&GameAction::Choose(child.choices.iter().find(|row|
        row.kind == RoomChoiceKind::Card { id: card_id, upgrades: offered_upgrades }
            && row.status == RoomChoiceStatus::Offered).unwrap().id))
        .unwrap_or_else(|error| panic!("legal elite card choice {card_id:?}: {error:?}"));
    expect(checks, test, "claimed elite card is added once", run.state.deck.len(), before_len + 1,
        "run/mutations.rs:105-145; run/rooms/rewards.rs:540-544", "design §11.4:808; Main-8 C3");
    if case.character == Character::Ironclad {
        // Ceramic Fish pays 9 gold on actual obtain; Bloody Idol heals 5 on each actual gold gain.
        expect(checks, test, "Ceramic Fish grants 9 gold on obtained elite card", run.state.gold.saturating_sub(gold_before_obtain), 9,
            "items/relic_effects/run.rs:122-124; run/mutations.rs:184-210", "Main-8 C3,C7; design Appendix A:1055,1201");
        expect(checks, test, "Bloody Idol heals on Ceramic Fish gold gain subject to cap",
            run.state.hp.saturating_sub(hp_before_obtain), 5.min(run.state.max_hp.saturating_sub(hp_before_obtain)),
            "items/relic_effects/run.rs:122-124", "Main-8 C7; design Appendix A:1201; design §9.1:690-699");
    }
    if let Some((egg, target_type)) = egg {
        let obtained = run.state.deck.iter().find(|card| card.id == card_id
            && !crate::content::Catalog::standard().card(card.id).unwrap().definition_only
            && card.instance >= 11).unwrap_or_else(|| run.state.deck.iter().rev().find(|card| card.id == card_id).unwrap());
        let matches_egg = crate::content::Catalog::standard().card(card_id)
            .is_some_and(|definition| definition.card_type == target_type);
        let expected_upgrades = if matches_egg { offered_upgrades + 1 } else { offered_upgrades };
        expect(checks, test, "egg upgrades the obtained card iff its type matches",
            obtained.upgrades, expected_upgrades, "run/mutations.rs:105-145; items/relic_effects/run.rs",
            match egg { RelicId::MoltenEgg => "Main-8 C3; design Appendix A:1099",
                RelicId::FrozenEgg => "Main-8 C3; design Appendix A:1089",
                _ => "Main-8 C3; design Appendix A:1115" });
    }
}

fn check_elite_rows_and_claim(case: Case, run: &mut Run, checks: &mut Checks, expected_relics: &mut Vec<RelicId>) {
    let test = case.name;
    let room = run.decision().unwrap().room.unwrap();
    let gold_rows = room.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Gold(_))).cloned().collect::<Vec<_>>();
    expect(checks, test, "elite reward has one Gold row", gold_rows.len(), 1,
        "run/rooms/rewards.rs:316-321", "design §11.4:807; Main-8 C7");
    if let Some(gold_row) = gold_rows.first() {
        let RoomChoiceKind::Gold(amount) = gold_row.kind else { unreachable!() };
        let (lower, upper) = if case.character == Character::Ironclad { (31, 44) } else { (25, 35) };
        if !(lower..=upper).contains(&amount) {
            checks.failures.push(format!(
                "RED test={test} assertion=elite Gold row amount is in its literal range; actual={amount}; expected={lower}..={upper}; suspected owner=run/rooms/rewards.rs:312-320; items/relic_effects/rewards.rs; source=Main-8 C7; design §11.4:807; Appendix A:1206"));
        }
        let gold_before = run.state.gold;
        let hp_before = run.state.hp;
        run.step(&GameAction::Choose(gold_row.id)).unwrap_or_else(|error| panic!("legal elite Gold claim: {error:?}"));
        let gold_delta = run.state.gold.saturating_sub(gold_before);
        let expected_delta = if case.character == Character::Watcher { 0 } else { amount };
        expect(checks, test, "elite Gold claim delta follows Golden Idol/Ectoplasm", gold_delta, expected_delta,
            "run/rooms/rewards.rs:514-518; run/mutations.rs:184-210", "Main-8 C7; design §9.2:701-710; Appendix A:1206,1208");
        if case.character == Character::Ironclad {
            expect(checks, test, "Bloody Idol heals on elite Gold gain, subject to the HP cap",
                run.state.hp.saturating_sub(hp_before), 5.min(run.state.max_hp.saturating_sub(hp_before)),
                "items/relic_effects/run.rs", "Main-8 C7; design Appendix A:1201; design §9.1:690-699");
        }
        eprintln!("X2 TRACE {} floor53 gold_row={amount} gold_before={gold_before} gold_after={} hp_before={hp_before} hp_after={}",
            case.name, run.state.gold, run.state.hp);
    }

    // The one elite card row is claimed before relic rows so egg obtain and Ceramic Fish are real.
    claim_card_row(case, run, checks);
    let relic_rows = run.decision().unwrap().room.unwrap().choices.into_iter()
        .filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_)) && row.status == RoomChoiceStatus::Offered)
        .collect::<Vec<_>>();
    let expected_relic_count = if case.character == Character::Silent { 2 } else { 1 };
    expect(checks, test, "Black Star controls elite relic row count", relic_rows.len(), expected_relic_count,
        "run/rooms/rewards.rs:324-414", "Main-8 C4; design Appendix A:1153; design §11.4:805");
    for (index, relic_row) in relic_rows.into_iter().enumerate() {
        let RoomChoiceKind::Relic(relic) = relic_row.kind else { unreachable!() };
        if case.character == Character::Silent && index == 1 {
            expect_true(checks, test, "Black Star second relic excludes campfire relics",
                ![RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel].contains(&relic),
                "run/rooms/rewards.rs:378-390", "design §11.4:805; design Appendix A:1153");
        }
        expect_true(checks, test, "elite relic was not already owned", !run.state.relics.contains(&relic),
            "run/rooms/rewards.rs:324-331; items/lifecycle.rs:55-72", "design §7.4:532-547; design §11.4:805");
        let hp_before = run.state.hp;
        let max_before = run.state.max_hp;
        let deck_before = run.state.deck.clone();
        let potions_before = run.state.potions.clone();
        run.step(&GameAction::Choose(relic_row.id)).unwrap_or_else(|error| panic!("legal elite relic row {relic:?}: {error:?}"));
        expected_relics.push(relic);
        assert_elite_equip(checks, test, relic, hp_before, max_before, &deck_before, &potions_before, run);
        expect_true(checks, test, "claimed elite relic is removed from every pool tier",
            !pool_contains(run, relic), "items/lifecycle.rs:66-72; items/relic_pools.rs:67-76",
            "design §7.4:526-547; design §11.4:805");
        eprintln!("X2 TRACE {} floor53 elite_relic[{}]={relic:?} hp={} max={} gold={} potions={:?}",
            case.name, index + 1, run.state.hp, run.state.max_hp, run.state.gold, run.state.potions);
    }

    let room = run.decision().unwrap().room.unwrap();
    let potion_rows = room.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))).cloned().collect::<Vec<_>>();
    // Main-8 C1: roll-time rows are Gold + (one or two) Relics; the CardReward row is auto-added
    // afterward (X2 found rewards.rs counting the CardReward row; fixed at the shared owner).
    expect(checks, test, "White Beast elite potion row is present", potion_rows.len(), 1,
        "run/rooms/rewards.rs:416-427", "Main-8 C1; docs/research/2026-10-19-act1-content-economy-potions.md:333-340");
    expect(checks, test, "White Beast elite potion offset is -10 after success", run.state.potion_offset, -10,
        "run/rooms/rewards.rs:416-427", "Main-8 C1; design §8.5:671-678; docs/research/2026-10-19-act1-content-economy-potions.md:334-340");
    if let Some(potion_row) = potion_rows.first() {
        let RoomChoiceKind::Potion(potion) = potion_row.kind else { unreachable!() };
        expect_true(checks, test, "elite potion belongs to literal 33-potion character union",
            literal_potion_price(potion, case.character).is_some(), "items/potion_generation.rs", "design §4.3:198-209; Appendix B:1227-1268");
        if matches!(case.character, Character::Silent | Character::Defect) {
            let expected_slots = match case.character {
                Character::Silent => vec![Some(PotionId::FirePotion), Some(PotionId::SwiftPotion), Some(PotionId::EnergyPotion)],
                Character::Defect => vec![Some(PotionId::FirePotion), Some(PotionId::FairyPotion), Some(PotionId::EnergyPotion)],
                _ => unreachable!(),
            };
            expect(checks, test, "full belt leaves elite potion slots unchanged", run.state.potions.clone(), expected_slots,
                "items/mod.rs:250-280; run/rooms/rewards.rs:514-528", "Main-8 C5; design §8.2:612-620");
            expect_true(checks, test, "full belt prevents a legal elite potion claim action",
                !run.decision().unwrap().candidates.iter().any(|candidate| candidate.action == GameAction::Choose(potion_row.id)),
                "run/rooms/rewards.rs:514-528; rooms::public", "Main-8 C5; design §8.2:612-620");
            expect_true(checks, test, "declined elite potion remains in stored rewards",
                run.state.room_loop.as_ref().unwrap().screen.choices.iter().any(|row|
                    row.kind == RoomChoiceKind::Potion(potion) && row.status == RoomChoiceStatus::Offered),
                "run/rooms/rewards.rs:514-528", "Main-8 C5; design §8.2:612-620");
        } else {
            let before_slots = run.state.potions.clone();
            let row = run.decision().unwrap().room.unwrap().choices.into_iter()
                .find(|row| row.id == potion_row.id).unwrap();
            run.step(&GameAction::Choose(row.id)).unwrap_or_else(|error| panic!("legal elite potion claim: {error:?}"));
            let mut expected_slots = before_slots;
            let slot = expected_slots.iter().position(Option::is_none).unwrap();
            expected_slots[slot] = Some(potion);
            expect(checks, test, "elite potion claim fills first empty belt slot", run.state.potions.clone(), expected_slots,
                "items/mod.rs:250-280; run/rooms/rewards.rs:514-528", "Main-8 C5; design §8.2:612-620");
        }
    }
    if case.character == Character::Silent {
        expect(checks, test, "Set 2 slots remain literal after elite potion suppression",
            run.state.potions.clone(),
            vec![Some(PotionId::FirePotion), Some(PotionId::SwiftPotion), Some(PotionId::EnergyPotion)],
            "items/mod.rs:250-280; run/rooms/rewards.rs:416-427", "Main-8 C1,C5; design §8.2:612-620");
    }
    let expected_proceed = run.decision().unwrap().room.unwrap().choices.iter().any(|row|
        row.kind == RoomChoiceKind::Proceed && row.status == RoomChoiceStatus::Offered);
    expect_true(checks, test, "elite reward Proceed remains legal", expected_proceed,
        "run/rooms/rewards.rs:429-430,551-557", "Main-8 C5; design §15.4:931-944");
    choose_kind(run, |kind| *kind == RoomChoiceKind::Proceed);
    expect(checks, test, "elite settlement returns the relic inventory once", run.state.relics.clone(), expected_relics.clone(),
        "run/combat.rs:210-282", "design §11.1:751-755; design §15.4:935-944");
    expect(checks, test, "elite settlement count equals entered count",
        run.state.combat_progress.settled, run.state.combat_progress.entered,
        "run/combat.rs:210-282", "design §11.1:751-755; design §15.4:935-944");
    if case.character == Character::Ironclad {
        expect(checks, test, "Neow's Lament is exhausted after the elite", run.state.relic_counters.get(&RelicId::NeowsLament).copied(), Some(-2),
            "items/relic_effects/combat.rs", "Main-8 C8; design §7.5:571; m10_content.rs:327-374 pattern");
        expect(checks, test, "Ancient Tea Set first-turn arm is consumed after the elite", run.state.relic_counters.get(&RelicId::AncientTeaSet).copied().unwrap_or(0), 0,
            "items/relic_effects/combat.rs", "Main-8 C8; design Appendix A:1047; run/tests/m27_relics_run.rs:44-52");
    }
    if case.character == Character::Silent {
        expect(checks, test, "Maw Bank remains disabled after elite floor entry", run.state.relic_counters.get(&RelicId::MawBank).copied(), Some(1),
            "items/relic_effects/run.rs:125-129", "Main-8 C7-C8; design §7.5:578");
    }
    expect(checks, test, "elite settlement returns literal Run counters once",
        run.state.relic_counters.clone(), expected_elite_run_counters(case.character, expected_relics),
        "run/combat.rs:210-282", "Main-8 C8-C9; sets.md counter table; design §7.5; items/relic_effects/turns.rs:264-268");
}

fn play_elite(case: Case, run: &mut Run, checks: &mut Checks, expected_relics: &[RelicId], rescue: &mut RescueTrace) -> bool {
    move_to_floor(run, 53);
    let test = case.name;
    expect(checks, test, "floor-53 starts the Shield and Spear Elite",
        run.state.active_combat.as_ref().map(|active| active.encounter), Some(crate::content::EncounterId::ShieldAndSpear),
        "run/rooms/act4.rs:128-158", "design §15.4:931-944");
    expect(checks, test, "elite combat receives the complete supplied relic inventory",
        combat_inventory(run).relics(), expected_relics, "run/combat.rs:349-388", "design §7.3:509-517; §11.1:751-755");
    let elite_hp = combat_enemies(run);
    expect_true(checks, test, "Neow's Lament sets every initial elite enemy to 1 HP",
        !elite_hp.is_empty() && elite_hp.iter().all(|(current, _)| *current == 1),
        "items/relic_effects/combat.rs", "Main-8 C8-C9; Appendix A:1211; design §7.5:571");
    expect(checks, test, "Neow's Lament counter moves from 1 to exhausted -2 at elite entry",
        combat_inventory(run).counter(RelicId::NeowsLament), -2, "items/relic_effects/combat.rs",
        "Main-8 C8-C9; Appendix A:1211; m10_content.rs:327-374 pattern");
    if case.character == Character::Ironclad {
        expect(checks, test, "Ancient Tea Set gives +2 elite first-turn energy",
            run.state.combat.as_ref().unwrap().public_full().unwrap().player.energy, 7,
            "items/relic_effects/combat.rs", "Main-8 C8; design Appendix A:1047,1157,1165");
    }
    let mut first_turn_checked = false;
    let mut steps = 0;
    while run.state.combat.is_some() && run.state.terminal.is_none() && steps < 1000 {
        let potions_before = combat_inventory(run).potions().to_vec();
        let tail_before = combat_inventory(run).counter(RelicId::LizardTail);
        let Some(action) = combat_policy_step(run) else {
            let offered = run.decision().map(|decision| decision.candidates.iter()
                .map(|candidate| candidate.action.clone()).collect::<Vec<_>>()).unwrap_or_default();
            checks.blockers.push(format!(
                "test={test} floor=53 has no legal PlayCard or EndTurn under the approved combat policy; offered actions={offered:?}; production owner=Run::decision candidate generation"));
            eprintln!("X2 BLOCKER {test} floor53 offered actions={offered:?}");
            return false;
        };
        run.step(&action).unwrap_or_else(|error| panic!("legal elite policy action {action:?}: {error:?}"));
        if case.character == Character::Defect {
            note_defect_rescue(rescue, potions_before, tail_before, &action, run);
        }
        if case.character == Character::Ironclad && !first_turn_checked {
            first_turn_checked = true;
            let counter = run.state.combat.as_ref().and_then(|combat| combat.item_context())
                .map(|items| items.counter(RelicId::AncientTeaSet))
                .or_else(|| run.state.relic_counters.get(&RelicId::AncientTeaSet).copied());
            expect(checks, test, "Ancient Tea Set clears after first elite turn", counter, Some(0),
                "items/relic_effects/combat.rs", "Main-8 C8; design Appendix A:1047; run/tests/m27_relics_run.rs:44-52");
        }
        steps += 1;
    }
    expect_true(checks, test, "elite policy reaches a terminal combat outcome within 1000 legal actions",
        run.state.combat.is_none() || run.state.terminal.is_some(), "run/combat.rs:193-207", "design §15.4:931-944");
    if case.character == Character::Defect {
        expect_true(checks, test, "1-HP elite ends without a Fairy or Lizard Tail rescue",
            rescue.fairy.is_none() && rescue.lizard_tail.is_none(), "items/mod.rs:125-135; items/relic_effects/accepted_law.rs:41-47",
            "Phase B ruling 2 C4; Appendix A:1211; design §15.4:931-944");
        expect(checks, test, "Fairy remains in slot 1 after the 1-HP elite",
            run.state.potions.get(1).copied().flatten(), Some(PotionId::FairyPotion), "items/mod.rs:125-135",
            "Phase B ruling 2 C4-C5; design §8.2:612-620");
        expect(checks, test, "Lizard Tail remains unused after the 1-HP elite",
            run.state.relic_counters.get(&RelicId::LizardTail).copied().unwrap_or(0), 0,
            "items/relic_effects/accepted_law.rs:41-47", "Phase B ruling 2 C4; design §15.4:931-944");
        expect_true(checks, test, "Set 3 does not own Toy Ornithopter",
            !run.state.relics.contains(&RelicId::ToyOrnithopter), "items/relic_effects/run.rs:67-73",
            "sets.md Set 3; Phase B ruling 2 C4");
    }
    if run.state.terminal.is_some() {
        // C9 explicitly allows an early Death at the floor-53 elite and requires disclosure.
        expect(checks, test, "early floor-53 terminal is Death without fault", run.state.terminal, Some(RunTerminal::Death),
            "run/combat.rs:210-282", "Main-8 C9; design §15.4:937-944");
        expect(checks, test, "floor-53 Death has no retained fault", run.state.fault.is_none(), true,
            "run/combat.rs:210-282", "Main-8 C9; design §15.4:937-944");
        expect(checks, test, "floor-53 Death returns the supplied relic inventory once", run.state.relics.as_slice(), expected_relics,
            "run/combat.rs:253-282", "Main-8 C9; design §11.1:751-755");
        expect(checks, test, "floor-53 Death settles once", run.state.combat_progress.settled, run.state.combat_progress.entered,
            "run/combat.rs:210-282", "Main-8 C9; design §11.1:751-755");
        expect(checks, test, "floor-53 Death returns literal Run counters once",
            run.state.relic_counters.clone(), expected_elite_run_counters(case.character, expected_relics),
            "run/combat.rs:210-282", "Main-8 C8-C9; sets.md counter table; design §7.5");
        eprintln!("X2 TRACE {} floor53 terminal={:?} hp={} gold={} slots={:?} counters={:?}",
            case.name, run.state.terminal, run.state.hp, run.state.gold, run.state.potions, run.state.relic_counters);
        return false;
    }
    expect(checks, test, "elite is won for reward settlement", run.state.last_combat.as_ref().map(|record| record.outcome), Some(Outcome::Victory),
        "run/combat.rs:210-282", "design §15.4:931-944");
    expect(checks, test, "elite settlement leaves the Run unfaulted", run.state.fault.is_none(), true,
        "run/combat.rs:210-282", "design §15.4:937-944");
    eprintln!("X2 TRACE {} elite-combat steps={steps} outcome={:?} hp={} gold={} max={} slots={:?} counters={:?}",
        case.name, run.state.last_combat.as_ref().map(|record| record.outcome), run.state.hp, run.state.gold, run.state.max_hp, run.state.potions, run.state.relic_counters);
    true
}

fn play_heart(case: Case, run: &mut Run, checks: &mut Checks, expected_relics: &mut Vec<RelicId>, rescue: &mut RescueTrace) {
    let test = case.name;
    let gold_before_entry = run.state.gold;
    let potion_slots_before_heart = run.state.potions.clone();
    move_to_floor(run, 54);
    let heart_hp = combat_enemies(run);
    expect(checks, test, "floor-54 starts the Corrupt Heart", run.state.active_combat.as_ref().map(|active| active.encounter),
        Some(crate::content::EncounterId::CorruptHeart), "run/rooms/act4.rs:128-158", "design §15.4:931-944");
    expect_true(checks, test, "Heart setup begins at full encounter HP",
        !heart_hp.is_empty() && heart_hp.iter().all(|(current, maximum)| current == maximum),
        "run/rooms/act4.rs:128-158; crates/spire-core/src/combat/enemies/act4/corrupt_heart.rs",
        "Main-8 C8; design §15.4:937-944");
    expect(checks, test, "Maw Bank produces no gold after the floor-52 purchase", run.state.gold, gold_before_entry,
        "items/relic_effects/run.rs:125-129; run/rooms/act4.rs:132-153", "Main-8 C7; design Appendix A:1062");
    expect(checks, test, "Heart combat receives returned relics", combat_inventory(run).relics(), expected_relics.as_slice(),
        "run/combat.rs:349-388", "design §11.1:751-755; §15.4:935-944");

    expect(checks, test, "Neow's Lament counter stays exhausted in Heart", combat_inventory(run).counter(RelicId::NeowsLament), -2,
        "items/relic_effects/combat.rs", "Main-8 C8-C9; design §7.5:571; Appendix A:1211");
    if case.character == Character::Ironclad {
        expect(checks, test, "Ancient Tea Set grants no Heart energy after the elite turn consumed it",
            run.state.combat.as_ref().unwrap().public_full().unwrap().player.energy, 5,
            "items/relic_effects/combat.rs", "Main-8 C8; design Appendix A:1047,1157,1165");
    }

    let initial_enemy_hp = combat_enemies(run).first().copied().unwrap_or((0, 0)).0;
    let player_hp_before_fire = run.state.combat.as_ref().unwrap().public_full().unwrap().player.hp;
    let player_max_hp_before_fire = run.state.combat.as_ref().unwrap().public_full().unwrap().player.max_hp;
    let fire_action = run.decision().unwrap().candidates.into_iter()
        .find(|candidate| matches!(candidate.action, GameAction::Combat(CombatAction::UsePotion { slot: 0, .. })))
        .map(|candidate| candidate.action)
        .unwrap_or_else(|| panic!("slot-0 Fire Potion use is legal on Heart turn 1"));
    run.step(&fire_action).unwrap_or_else(|error| panic!("legal scripted Heart Fire Potion: {error:?}"));
    let target_hp = combat_enemies(run).first().copied().unwrap_or((0, 0)).0;
    let damage = initial_enemy_hp - target_hp;
    let expected_damage = if case.character == Character::Defect { 40 } else { 20 };
    expect(checks, test, "Heart turn-1 Fire Potion damage uses Sacred Bark only for Defect", damage, expected_damage,
        "items/potions.rs; combat/enemies", "Main-8 C4; design Appendix B:1227; design §8.4:650-660");
    expect(checks, test, "Heart turn-1 Fire Potion empties slot zero", combat_inventory(run).potions()[0], None,
        "items/mod.rs:633-650", "Main-8 C5-C9; design §8.3:624-640");
    if case.character == Character::Silent {
        // Toy Ornithopter requests +5 from actual potion use; it is not doubled by Bark.
        let player_hp_after_fire = run.state.combat.as_ref().unwrap().public_full().unwrap().player.hp;
        expect(checks, test, "Toy Ornithopter heals 5 on Fire Potion use, subject to cap",
            player_hp_after_fire - player_hp_before_fire,
            5.min(player_max_hp_before_fire - player_hp_before_fire),
            "items/potions.rs", "Main-8 C7; design Appendix A:1077; design §8.3:626-629");
    }

    let mut steps = 0;
    let mut heart_end_turns = 0;
    while run.state.combat.is_some() && run.state.terminal.is_none() && steps < 2000 {
        let potions_before = combat_inventory(run).potions().to_vec();
        let tail_before = combat_inventory(run).counter(RelicId::LizardTail);
        let Some(action) = combat_policy_step(run) else {
            let offered = run.decision().map(|decision| decision.candidates.iter()
                .map(|candidate| candidate.action.clone()).collect::<Vec<_>>()).unwrap_or_default();
            checks.blockers.push(format!(
                "test={test} floor=54 has no legal PlayCard or EndTurn under the approved combat policy; offered actions={offered:?}; production owner=Run::decision candidate generation"));
            eprintln!("X2 BLOCKER {test} floor54 offered actions={offered:?}");
            return;
        };
        run.step(&action).unwrap_or_else(|error| panic!("legal Heart policy action {action:?}: {error:?}"));
        if matches!(action, GameAction::Combat(CombatAction::EndTurn)) {
            heart_end_turns += 1;
        }
        if case.character == Character::Defect {
            note_defect_rescue(rescue, potions_before, tail_before, &action, run);
        }
        steps += 1;
    }
    expect_true(checks, test, "Heart policy reaches a terminal combat outcome within 2000 legal actions",
        run.state.combat.is_none() || run.state.terminal.is_some(), "run/combat.rs:193-207", "design §15.4:931-944");
    if expected_relics.contains(&RelicId::HappyFlower) {
        // Its equip is after Elite; Heart starts at counter 1, then seven nonlethal
        // EndTurn actions advance it to 2 before the eighth EndTurn ends in Death.
        expect(checks, test, "Happy Flower Heart policy reaches the literal eight-EndTurn boundary",
            heart_end_turns, 8, "run/diagnostics/m27.rs:51-88; combat_policy_step in this file",
            "sets.md Phase B action contract; items/relic_effects/turns.rs:66-75; items/tests/m27_relics_turns.rs:191-200");
    }
    if run.state.terminal.is_none() && run.state.combat.is_none()
        && run.state.room_loop.as_ref().is_some_and(|room| room.screen.kind == RoomViewKind::HeartVictory)
    {
        choose_kind(run, |kind| *kind == RoomChoiceKind::Proceed);
    }
    expect_true(checks, test, "Heart ends in Victory or Death and is terminal",
        matches!(run.state.terminal, Some(RunTerminal::HeartVictory | RunTerminal::Death)),
        "run/rooms/act4.rs:224-255; run/combat.rs:278-282", "Main-8 C9; design §15.4:937-944");
    expect(checks, test, "Heart settlement has no fault", run.state.fault.is_none(), true,
        "run/combat.rs:210-282", "Main-8 C9; design §15.4:937-944");
    expect(checks, test, "Heart settlement returns the relic inventory once", run.state.relics.clone(), expected_relics.clone(),
        "run/combat.rs:210-282", "design §11.1:751-755; Main-8 C9");
    expect(checks, test, "Heart settlement count equals combat entries",
        run.state.combat_progress.settled, run.state.combat_progress.entered,
        "run/combat.rs:210-282", "design §11.1:751-755; Main-8 C9");
    expect(checks, test, "Heart settlement returns literal Run counters once",
        run.state.relic_counters.clone(), expected_heart_run_counters(case.character, expected_relics),
        "run/combat.rs:210-282", "Main-8 C8-C9; sets.md counter table; design §7.5; items/relic_effects/turns.rs:66-75");

    let mut expected_potions = potion_slots_before_heart;
    expected_potions[0] = None; // C5 scripted Fire Potion use.
    if case.character == Character::Defect { expected_potions[1] = None; } // C4 Fairy rescue empties its slot.
    expect(checks, test, "potion slots after scripted use and any rescue", run.state.potions.clone(), expected_potions,
        "run/combat.rs:253-282; items/mod.rs:633-650", "Main-8 C5-C9; design §8.2:612-620; §9.3:712-720");
    if case.character == Character::Defect {
        expect(checks, test, "Fairy rescue consumes its slot before Lizard Tail rescue",
            run.state.potions.get(1).copied().flatten(), None, "items/rescue.rs", "Main-8 C4; design §9.3:712-720");
        expect(checks, test, "Lizard Tail was used on the later lethal Heart hit",
            run.state.relic_counters.get(&RelicId::LizardTail).copied(), Some(1),
            "items/relic_effects/run.rs", "Main-8 C4; design §9.3:714-720; design §7.5:578");
    }
    if case.character == Character::Watcher {
        expect(checks, test, "Mark of the Bloom prevents Heart healing and rescue", run.state.hp, 0,
            "items/laws.rs; items/relic_effects/run.rs", "Main-8 C4; design §9.1:692-699; §9.3:714-720");
    }
    eprintln!("X2 TRACE {} floor54 Heart steps={steps} end_turns={heart_end_turns} terminal={:?} hp={}/{} gold={} slots={:?} counters={:?}",
        case.name, run.state.terminal, run.state.hp, run.state.max_hp, run.state.gold, run.state.potions, run.state.relic_counters);
}

fn run_case(case: Case) {
    let test = match case.character {
        Character::Ironclad => "m27_supplied_act4_ironclad",
        Character::Silent => "m27_supplied_act4_silent",
        Character::Defect => "m27_supplied_act4_defect",
        Character::Watcher => "m27_supplied_act4_watcher",
    };
    let mut checks = Checks::default();
    let mut rescue = RescueTrace::default();
    let supply = complete_supply(case.character);
    expect(&mut checks, test, "literal character potion union has 33 identities",
        literal_character_potions(case.character).len(), 33, "items/potion_generation.rs",
        "design §4.3-4.4:198-218; Appendix B:1227-1268");
    expect(&mut checks, test, "each supplied set has at least 20 shared combat relics",
        COMBAT_PACKAGE.len(), 20, "crates/spire-core/src/run/tests/m27_supplied.rs:18-31", "sets.md Shared combat package");
    let mut run = diagnostics::complete_act4_run(case.character, supply)
        .unwrap_or_else(|error| panic!("construct complete Act 4 supplied {} Run: {error:?}", case.name));
    // Main-8 C6 explicitly makes HP=30 fixture provenance, after the constructor and before this test's first Run::step.
    run.state.hp = 30;
    let mut expected_relics = supplied_relics(case.character);
    expect(&mut checks, test, "complete Act 4 diagnostic enters floor 50 for the intended character",
        (run.state.character, run.state.act, run.state.floor), (case.character, 4, 50),
        "run/diagnostics/m27.rs:51-88", "design §15.4:931-944");
    expect(&mut checks, test, "supplied complete relic inventory matches literal Phase A set",
        run.state.relics.clone(), expected_relics.clone(), "run/diagnostics/m27.rs:31-50", "sets.md per-character relic lists; Main-8 C8");
    expect(&mut checks, test, "supplied potion inventory matches C5 literal slots",
        run.state.potions.clone(), complete_supply(case.character).potions,
        "run/diagnostics/m27.rs:31-50", "Main-8 C5; sets.md Potion supply");
    expect(&mut checks, test, "Neow's Lament starts with the supplied counter of 1",
        run.state.relic_counters.get(&RelicId::NeowsLament).copied(), Some(1),
        "run/diagnostics/m27.rs:31-50", "Phase B ruling 2 C9; design §7.5:571; Appendix A:1211");

    shop_and_campfire(case, &mut run, &mut checks, &mut expected_relics);
    if play_elite(case, &mut run, &mut checks, &expected_relics, &mut rescue) {
        check_elite_rows_and_claim(case, &mut run, &mut checks, &mut expected_relics);
        play_heart(case, &mut run, &mut checks, &mut expected_relics, &mut rescue);
    }
    if case.character == Character::Defect {
        check_defect_rescues(&mut checks, test, &rescue);
        eprintln!("X2 TRACE Defect rescue post-step HP fairy={:?} lizard_tail={:?} order={:?}",
            rescue.fairy.as_ref().map(|step| (step.hp_after, step.max_hp_after)),
            rescue.lizard_tail.as_ref().map(|step| (step.hp_after, step.max_hp_after)), rescue.order);
    }
    finish_checks(test, checks);
}

#[test]
fn m27_supplied_act4_ironclad() {
    run_case(Case { name: "Ironclad", character: Character::Ironclad });
}

#[test]
fn m27_supplied_act4_silent() {
    run_case(Case { name: "Silent", character: Character::Silent });
}

#[test]
fn m27_supplied_act4_defect() {
    run_case(Case { name: "Defect", character: Character::Defect });
}

#[test]
fn m27_supplied_act4_watcher() {
    run_case(Case { name: "Watcher", character: Character::Watcher });
}
