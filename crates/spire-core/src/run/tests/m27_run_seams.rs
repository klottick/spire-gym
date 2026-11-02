//! M27 Run-room seams: complete-admission event owners and frozen M25 behavior.
use super::super::*;
use crate::{
    combat::rng::Partition,
    content::{Character, EncounterId, RelicId},
    items::dispatch::{take_combat_settled_outcomes, take_run_event_log},
    run::diagnostics::{m23_diagnostic, m25_diagnostic, M23Diagnostic, M23DiagnosticBoss, M25Diagnostic},
};

fn m25_act4_floor(floor: u32) -> Run {
    m25_diagnostic(M25Diagnostic::Act4Room { floor }).unwrap()
}
fn act4_entry() -> Run {
    let (mut run, node) = super::m9_boss::source(EncounterId::TimeEater);
    run.state.room_loop.as_mut().unwrap().scenario = RoomScenario::M10IroncladHeart;
    let start = AuthoredRunStart {
        hp: run.state.hp,
        max_hp: run.state.max_hp,
        gold: run.state.gold,
        deck: run.state.deck.clone(),
        relics: run.state.relics.clone(),
        potions: run.state.potions.clone(),
        relic_counters: run.state.relic_counters.clone(),
    };
    let _ = RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete).unwrap();
    let CompleteCombatStart::Supplied(resources) = CompleteCombatStart::Supplied(start)
        .normalized_for(Character::Ironclad, ItemAdmission::Complete).unwrap();
    run.state.hp = resources.hp;
    run.state.max_hp = resources.max_hp;
    run.state.gold = resources.gold;
    run.state.deck = resources.deck;
    run.state.relics = resources.relics;
    run.state.potions = resources.potions;
    run.state.relic_counters = resources.relic_counters;
    run.state.next_instance = run.state.deck.iter().map(|card| card.instance).max().unwrap() + 1;
    run.state.card_admission = CardAdmission::Complete;
    run.state.item_admission = ItemAdmission::Complete;
    for key in [Key::Ruby, Key::Sapphire, Key::Emerald] {
        let event = crate::run::mutations::acquire_key(&mut run.state, key).unwrap();
        crate::run::reward::commit(&mut run.state, &[event]).unwrap();
    }
    run.step(&GameAction::Move(node)).unwrap();
    hit(&mut run, 0, 456);
    run.settle_combat().unwrap();
    choose(&mut run, |kind| *kind == RoomChoiceKind::Proceed);
    for stage in 1..=3 {
        choose(&mut run, |kind| *kind == RoomChoiceKind::Act3Continue { stage });
    }
    choose(&mut run, |kind| *kind == RoomChoiceKind::Act3Continue { stage: 4 });
    let _ = take_run_event_log();
    let _ = take_combat_settled_outcomes();
    run
}
fn choice_id(run: &Run, pred: impl Fn(&RoomChoiceKind) -> bool) -> ChoiceId {
    run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|choice| choice.status == RoomChoiceStatus::Offered && pred(&choice.kind))
        .unwrap().id
}
fn choose(run: &mut Run, pred: impl Fn(&RoomChoiceKind) -> bool) {
    let id = choice_id(run, pred);
    run.step(&GameAction::Choose(id)).unwrap();
}
fn hit(run: &mut Run, slot: usize, amount: i32) {
    let combat = run.state.combat.as_mut().unwrap();
    let target = crate::combat::Actor::Enemy(combat.public_full().unwrap().enemies[slot].instance_id.clone());
    let result = combat.enqueue_native_effects(vec![crate::combat::Effect {
        source: crate::combat::Source::Actor(crate::combat::Actor::Player),
        target: Some(target),
        kind: crate::combat::EffectKind::Damage(crate::combat::DamagePayload::Fixed(amount)),
    }], None);
    if let Err(error) = result {
        panic!("complete M10 fixture lethal damage: {error:?}; state={:?}", combat.snapshot());
    }
}
fn move_floor(run: &mut Run, floor: u32) {
    let node = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == floor).unwrap().id;
    run.step(&GameAction::Move(node)).unwrap();
}
#[test]
fn m27_run_seams_inert_frozen() {
    let campfire = m25_act4_floor(51);
    let campfire_before = format!("{:?}", campfire.state);
    let campfire_view = campfire.decision().unwrap().room.unwrap();
    assert_eq!(campfire_view.choices.iter().map(|choice| (choice.kind.clone(), choice.status)).collect::<Vec<_>>(), vec![
        (RoomChoiceKind::Rest { heal: 21 }, RoomChoiceStatus::Offered),
        (RoomChoiceKind::Smith, RoomChoiceStatus::Offered),
        (RoomChoiceKind::Recall, RoomChoiceStatus::Disabled),
    ]);
    assert_eq!(format!("{:?}", campfire.state), campfire_before);

    let shop = m25_act4_floor(52);
    let shop_before = format!("{:?}", shop.state);
    let shop_view = shop.decision().unwrap().room.unwrap();
    assert_eq!(shop_view.choices.iter().map(|choice| choice.price).collect::<Vec<_>>(), vec![Some(50), Some(75), Some(150), Some(150), Some(50), Some(shop.state.room_loop.as_ref().unwrap().purge_cost), None]);
    assert!(matches!(shop_view.choices[3].kind, RoomChoiceKind::Relic(RelicId::Melange)));
    assert_eq!(shop_view.choices.last().unwrap().kind, RoomChoiceKind::Leave);
    assert_eq!(format!("{:?}", shop.state), shop_before);

    let mut rewards = m25_act4_floor(53);
    let active = rewards.state.active_combat.as_ref().unwrap();
    let record = CombatRecord { visit: active.visit, encounter: active.encounter, outcome: crate::combat::Outcome::Victory };
    let rewards_before = format!("{:?}", rewards.state);
    let prepared = rooms::rewards::prepare_combat(&rewards, &record).unwrap().unwrap();
    assert_eq!(format!("{:?}", rewards.state), rewards_before);
    rooms::rewards::commit_combat(&mut rewards, prepared);
    // Inspect the committed reward screen directly: this synthetic fixture leaves
    // the active combat installed, while Run::decision correctly publishes combat.
    // Accepted M25 coverage pins this frozen row order at m25_watcher_heart.rs:442-452.
    let kinds = rewards.state.room_loop.as_ref().unwrap().screen.choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>();
    assert!(matches!(kinds.as_slice(), [RoomChoiceKind::Gold(_), RoomChoiceKind::CardReward, RoomChoiceKind::Potion(_), RoomChoiceKind::Relic(_), RoomChoiceKind::Proceed]));
}

#[test]
fn m27_run_seams_reached() {
    let _ = take_run_event_log();
    let _ = take_combat_settled_outcomes();
    let mut reached = Vec::new();

    // The generic world.rs constructor also dispatches FloorEntered, then deliberately retains its
    // documented unsupported-room fault after the entry event has committed.
    let mut world_run = act4_entry();
    let first = world_run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == 51).unwrap().id;
    assert!(matches!(crate::run::world::enter(&mut world_run.state, first), Err(RunError::Faulted(_))));
    reached.extend(take_run_event_log());

    // Start at the accepted, complete-admission M10 Act4 entry and traverse the real owners.
    let mut run = act4_entry();
    move_floor(&mut run, 51);
    reached.extend(take_run_event_log());
    choose(&mut run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    reached.extend(take_run_event_log());
    move_floor(&mut run, 52);
    reached.extend(take_run_event_log());
    choose(&mut run, |kind| matches!(kind, RoomChoiceKind::Card { .. }));
    reached.extend(take_run_event_log());
    crate::run::mutations::gain_gold(&mut run.state, 3).unwrap();
    let gained = run.state.deck.last().unwrap().instance;
    crate::run::mutations::remove_card(&mut run.state, gained).unwrap();
    reached.extend(take_run_event_log());

    choose(&mut run, |kind| *kind == RoomChoiceKind::Leave);
    move_floor(&mut run, 53);
    let enemy_count = run.decision().unwrap().combat.unwrap().enemies.len();
    for slot in (0..enemy_count).rev() { hit(&mut run, slot, 30_012); }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(crate::combat::Outcome::Victory));
    run.settle_combat().unwrap();
    let settlement = take_run_event_log();
    assert_eq!(settlement.iter().filter(|event| **event == "CombatSettled").count(), 1);
    assert_eq!(take_combat_settled_outcomes(), vec![crate::combat::Outcome::Victory]);
    reached.extend(settlement);

    // The existing ordinary and boss chest owners each publish their base contents before
    // the shared ChestOpened dispatch.
    let mut ordinary = act4_entry();
    let view = rooms::view(&mut ordinary.next_choice, RoomViewKind::Treasure, vec![
        (RoomChoiceKind::OpenChest, None), (RoomChoiceKind::Proceed, None),
    ]).unwrap();
    ordinary.state.room_loop.as_mut().unwrap().screen = view;
    choose(&mut ordinary, |kind| *kind == RoomChoiceKind::OpenChest);
    reached.extend(take_run_event_log());

    let mut boss_entry=m23_diagnostic(M23Diagnostic::BossRewards{boss:M23DiagnosticBoss::Collector}).unwrap();
    boss_entry.state.item_admission=ItemAdmission::Complete;
    choose(&mut boss_entry,|kind|*kind==RoomChoiceKind::CardReward);
    choose(&mut boss_entry,|kind|*kind==RoomChoiceKind::Skip);
    choose(&mut boss_entry,|kind|*kind==RoomChoiceKind::Proceed);
    assert_eq!(boss_entry.decision().unwrap().room.unwrap().kind,RoomViewKind::BossTreasure);
    reached.extend(take_run_event_log());

    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = vec!["Hexaghost", "SlimeBoss"];
    let mut boss = Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 }, profile,
        RunRng::new([7; 32], Partition::Test, 0), RoomScenario::M5Act1Captured,
    ).unwrap();
    boss.state.item_admission = ItemAdmission::Complete;
    boss.state.floor = 17;
    boss.state.room = "BossTreasure";
    boss.state.room_loop.as_mut().unwrap().screen = rooms::view(&mut boss.next_choice, RoomViewKind::BossTreasure, vec![
        (RoomChoiceKind::OpenChest, None), (RoomChoiceKind::Proceed, None),
    ]).unwrap();
    choose(&mut boss, |kind| *kind == RoomChoiceKind::OpenChest);
    reached.extend(take_run_event_log());

    for event in [
        "FloorEntered", "RoomEntered", "CampfireOptions", "RestHeal", "Rested", "ShopEntered", "ShopPrice",
        "RemovalPrice", "ShopPurchase", "RewardsBuilding", "ChestOpened", "CardObtaining",
        "CardObtained", "CardRemoved", "GoldGaining", "GoldGained", "CombatSettled",
    ] {
        assert!(reached.contains(&event), "missing {event}: {reached:?}");
    }
    assert_eq!(reached.iter().filter(|event| **event == "ChestOpened").count(), 2);
    // Main-3 (A21): every room owner now dispatches RoomEntered — Act 4 Rest, Shop, Elite and
    // BossTreasure on this route (was 1 while only BossTreasure was wired).
    assert_eq!(reached.iter().filter(|event| **event == "RoomEntered").count(), 4);
    // Main-8 (X2): each real room owner also dispatches FloorEntered (Maw Bank's onEnterRoom), so
    // FloorEntered = the four room entries plus the faulting generic world.rs entry above.
    assert_eq!(reached.iter().filter(|event| **event == "FloorEntered").count(), 5);
    assert_eq!(reached.iter().filter(|event| **event == "CombatSettled").count(), 1);
}

#[test]
fn m27_extra_card_rewards_are_distinct() {
    let _ = take_run_event_log();
    let mut run = m25_act4_floor(53);
    run.state.item_admission = ItemAdmission::Complete;
    let active = run.state.active_combat.as_ref().unwrap();
    let record = CombatRecord { visit: active.visit, encounter: active.encounter, outcome: crate::combat::Outcome::Victory };
    let building = crate::items::dispatch::RunItemEvent::RewardsBuilding {
        room: crate::items::run_laws::RewardRoom::Elite,
        card_choices: 3,
        extra_card_rewards: 1,
        gold_percent: 0,
        extra_elite_relics: 0,
        include_potion: true,
        singing_bowl: false,
    };
    let prepared = rooms::rewards::prepare_combat_with_building(&run, &record, building).unwrap().unwrap();
    rooms::rewards::commit_combat(&mut run, prepared);
    run.state.combat = None;
    run.state.active_combat = None;

    assert_eq!(run.decision().unwrap().room.unwrap().choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward).count(), 2);
    choose(&mut run, |kind| *kind == RoomChoiceKind::CardReward);
    let first_child = run.decision().unwrap().room.unwrap();
    choose(&mut run, |kind| matches!(kind, RoomChoiceKind::Card { .. }));
    let parent = run.decision().unwrap().room.unwrap();
    assert_eq!(parent.choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward && choice.status == RoomChoiceStatus::Claimed).count(), 1);
    assert_eq!(parent.choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward && choice.status == RoomChoiceStatus::Offered).count(), 1);

    choose(&mut run, |kind| *kind == RoomChoiceKind::CardReward);
    let second_child = run.decision().unwrap().room.unwrap();
    assert_ne!(first_child, second_child, "each card reward parent owns distinct child choice IDs");
}

fn run_profile() -> CreationProfile {
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = vec!["Hexaghost", "SlimeBoss"];
    profile.seen_bosses[1] = vec!["Automaton", "Collector"];
    profile.seen_bosses[2] = vec!["AwakenedOne", "DonuAndDeca"];
    profile
}

fn enter_connected_kind(run: &mut Run, kind: MapNodeKind) -> NodeId {
    let map = run.state.map.as_mut().unwrap();
    let target = map.nodes.iter().find(|node| node.kind == kind && !node.visited
        && map.nodes.iter().any(|previous| previous.edges.contains(&node.id))).unwrap().id;
    let previous = map.nodes.iter_mut().find(|node| node.edges.contains(&target)).unwrap();
    previous.visited = true;
    run.state.floor = previous.floor;
    map.current = Some(previous.id);
    target
}

fn assert_one_room_entered(run: &mut Run, target: NodeId) {
    let _ = take_run_event_log();
    run.step(&GameAction::Move(target)).unwrap();
    let events = take_run_event_log();
    assert_eq!(events.iter().filter(|event| **event == "RoomEntered").count(), 1, "{events:?}");
}

fn profile_act4_entry() -> Run {
    let (mut run, node) = super::m9_boss::source(EncounterId::TimeEater);
    run.state.room_loop.as_mut().unwrap().scenario = RoomScenario::M10IroncladHeart;
    run.state.act4_enabled = true;
    for key in [Key::Ruby, Key::Sapphire, Key::Emerald] {
        let event = crate::run::mutations::acquire_key(&mut run.state, key).unwrap();
        crate::run::reward::commit(&mut run.state, &[event]).unwrap();
    }
    run.step(&GameAction::Move(node)).unwrap();
    super::m9_boss::hit(&mut run, 0, 456);
    run.settle_combat().unwrap();
    choose(&mut run, |kind| *kind == RoomChoiceKind::Proceed);
    for stage in 1..=4 {
        choose(&mut run, |kind| *kind == RoomChoiceKind::Act3Continue { stage });
    }
    assert_eq!(run.state.item_admission, ItemAdmission::Profile);
    run
}

#[test]
fn m27_room_seams_frozen_inert() {
    let mut run = profile_act4_entry();
    let profile_debug = format!("{:?}", run.state);
    assert!(!profile_debug.contains("item_admission"));

    let _ = take_run_event_log();
    move_floor(&mut run, 51);
    assert!(!take_run_event_log().contains(&"RoomEntered"));
    choose(&mut run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    move_floor(&mut run, 52);
    assert!(!take_run_event_log().contains(&"RoomEntered"));
    choose(&mut run, |kind| *kind == RoomChoiceKind::Leave);
    move_floor(&mut run, 53);
    assert!(!take_run_event_log().contains(&"RoomEntered"));

    let enemy_count = run.decision().unwrap().combat.unwrap().enemies.len();
    for slot in (0..enemy_count).rev() { hit(&mut run, slot, 30_012); }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(crate::combat::Outcome::Victory));
    let active = run.state.active_combat.as_ref().unwrap();
    let record = CombatRecord { visit: active.visit, encounter: active.encounter, outcome: crate::combat::Outcome::Victory };
    let before_prepare = format!("{:?}", run.state);
    let _prepared = rooms::rewards::prepare_combat(&run, &record).unwrap().unwrap();
    assert_eq!(format!("{:?}", run.state), before_prepare, "Profile reward preparation is observational");
    run.settle_combat().unwrap();
    let captured = run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap();
    assert_eq!(captured.return_to, rooms::rewards::RewardReturn::Map);
    assert_eq!(captured.cards.choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![
        RoomChoiceKind::Card { id: crate::content::CardId::TrueGrit, upgrades: 0 },
        RoomChoiceKind::Card { id: crate::content::CardId::RecklessCharge, upgrades: 0 },
        RoomChoiceKind::Card { id: crate::content::CardId::FireBreathing, upgrades: 0 },
        RoomChoiceKind::Skip,
    ]);
    let captured_debug = format!("{:?}", captured);
    assert_eq!(captured_debug,
        "CapturedRewards { cards: RoomView { kind: CardReward, choices: [RoomChoiceView { id: ChoiceId(54), kind: Card { id: TrueGrit, upgrades: 0 }, price: None, status: Offered }, RoomChoiceView { id: ChoiceId(55), kind: Card { id: RecklessCharge, upgrades: 0 }, price: None, status: Offered }, RoomChoiceView { id: ChoiceId(56), kind: Card { id: FireBreathing, upgrades: 0 }, price: None, status: Offered }, RoomChoiceView { id: ChoiceId(57), kind: Skip, price: None, status: Offered }] }, return_to: Map }");
    assert!(captured_debug.contains("TrueGrit") && captured_debug.contains("RecklessCharge") && captured_debug.contains("FireBreathing"));
    assert!(!captured_debug.contains("extra"));
    assert!(!take_run_event_log().iter().any(|event| matches!(*event, "RoomEntered" | "RewardsBuilding" | "CombatSettled")));
}

#[test]
fn m27_room_entered_every_owner() {
    let mut profile = Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 }, run_profile(),
        RunRng::new([7; 32], Partition::Test, 0), RoomScenario::M4Act1Prefix,
    ).unwrap();
    choose(&mut profile, |kind| matches!(kind, RoomChoiceKind::NeowMaxHp { .. }));
    profile.state.item_admission = ItemAdmission::Complete;
    let target = enter_connected_kind(&mut profile, MapNodeKind::Question);
    assert_one_room_entered(&mut profile, target);
    assert!(matches!(profile.decision().unwrap().room.unwrap().kind, RoomViewKind::Event(_)));

    let mut act1 = Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 }, run_profile(),
        RunRng::new([7; 32], Partition::Test, 0), RoomScenario::M5Act1Captured,
    ).unwrap();
    choose(&mut act1, |kind| *kind == RoomChoiceKind::NeowLament);
    act1.state.item_admission = ItemAdmission::Complete;
    let target = enter_connected_kind(&mut act1, MapNodeKind::Question);
    assert_one_room_entered(&mut act1, target);
    assert!(matches!(act1.decision().unwrap().room.unwrap().kind, RoomViewKind::Event(_)));

    let mut act2 = Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 }, run_profile(),
        RunRng::new([7; 32], Partition::Test, 0), RoomScenario::M13SilentAct2,
    ).unwrap();
    choose(&mut act2, |kind| *kind == RoomChoiceKind::NeowLament);
    let mut rng = act2.state.rng.clone();
    let mut map = super::super::map::generate(2, &mut rng, super::super::map::MapEligibility::from_run(&act2.state)).unwrap();
    for node in &mut map.nodes { node.floor += 17; }
    map.boss_icon = Some(EncounterId::Champ);
    act2.state.act = 2;
    act2.state.floor = 17;
    act2.state.map = Some(map);
    act2.state.rng = rng;
    let room = act2.state.room_loop.as_mut().unwrap();
    room.act1 = None;
    room.act2 = Some(Default::default());
    room.act2_boss_order = Some(vec![EncounterId::Champ]);
    room.screen = RoomView { kind: RoomViewKind::Map, choices: vec![] };
    act2.state.item_admission = ItemAdmission::Complete;
    let target = enter_connected_kind(&mut act2, MapNodeKind::Shop);
    assert_one_room_entered(&mut act2, target);
    assert_eq!(act2.decision().unwrap().room.unwrap().kind, RoomViewKind::Shop);

    let mut act3 = Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 }, run_profile(),
        RunRng::new([7; 32], Partition::Test, 0), RoomScenario::M14SilentAct3,
    ).unwrap();
    choose(&mut act3, |kind| *kind == RoomChoiceKind::NeowLament);
    let mut rng = act3.state.rng.clone();
    let mut map = super::super::map::generate(3, &mut rng, super::super::map::MapEligibility::from_run(&act3.state)).unwrap();
    for node in &mut map.nodes { node.floor += 34; }
    map.boss_icon = Some(EncounterId::TimeEater);
    act3.state.act = 3;
    act3.state.floor = 34;
    act3.state.map = Some(map);
    act3.state.rng = rng;
    let room = act3.state.room_loop.as_mut().unwrap();
    room.act1 = None;
    room.act2 = None;
    room.act3 = Some(Default::default());
    room.act3_boss_order = Some(vec![EncounterId::TimeEater]);
    room.screen = RoomView { kind: RoomViewKind::Map, choices: vec![] };
    act3.state.item_admission = ItemAdmission::Complete;
    let target = enter_connected_kind(&mut act3, MapNodeKind::Question);
    assert_one_room_entered(&mut act3, target);
    assert!(matches!(act3.decision().unwrap().room.unwrap().kind, RoomViewKind::Event(_)));

    let mut act4 = act4_entry();
    let _ = take_run_event_log();
    move_floor(&mut act4, 51);
    let events = take_run_event_log();
    assert_eq!(events.iter().filter(|event| **event == "RoomEntered").count(), 1, "rest: {events:?}");
    choose(&mut act4, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    let _ = take_run_event_log();
    move_floor(&mut act4, 52);
    let events = take_run_event_log();
    assert_eq!(events.iter().filter(|event| **event == "RoomEntered").count(), 1, "shop: {events:?}");
    choose(&mut act4, |kind| *kind == RoomChoiceKind::Leave);
    let _ = take_run_event_log();
    move_floor(&mut act4, 53);
    let events = take_run_event_log();
    assert_eq!(events.iter().filter(|event| **event == "RoomEntered").count(), 1, "elite: {events:?}");

    let mut boss = m23_diagnostic(M23Diagnostic::BossRewards { boss: M23DiagnosticBoss::Collector }).unwrap();
    boss.state.item_admission = ItemAdmission::Complete;
    choose(&mut boss, |kind| *kind == RoomChoiceKind::CardReward);
    choose(&mut boss, |kind| *kind == RoomChoiceKind::Skip);
    let _ = take_run_event_log();
    choose(&mut boss, |kind| *kind == RoomChoiceKind::Proceed);
    let events = take_run_event_log();
    assert_eq!(events.iter().filter(|event| **event == "RoomEntered").count(), 1, "boss treasure: {events:?}");
}
