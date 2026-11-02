//! M27 plan Task J1, lane J, key m27_relic_selections.
//! Run equip/selection relics through the real claim paths (reward and shop rows → obtain_relic →
//! after_obtain → the equip driver): the bottles, Astrolabe, Dolly's Mirror, Orrery, Calling Bell,
//! Pandora's Box, Tiny House, Cauldron and Necronomicon's pickup curse.
//! Spec-sheet literals: main-v1/task0/spec-sheets-v2/j.md (M27 evidence root).
use crate::combat::{CombatAction, TargetId};
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_support::{complete_run, next_complete_combat};
use crate::run::rooms::{RoomChoiceKind, RoomChoiceStatus, RoomChoiceView, RoomLoop, RoomView, RoomViewKind};
use crate::run::{CardInstance, ChoiceId, GameAction, MapNode, MapNodeKind, Run, RunMap, RoomScenario};
use std::collections::BTreeMap;

/// A complete Ironclad Run whose fixture combat is settled by three Fire Potions; the authored
/// deck (instances 100…) is then installed in RunState.
fn settled(deck: &[(CardId, u32, i32)]) -> Run {
    let mut run = complete_run(Character::Ironclad, vec![], vec![Some(PotionId::FirePotion); 3],
        BTreeMap::new(), EncounterId::Cultist);
    for slot in 0..3 {
        run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) })
            .unwrap_or_else(|error| panic!("fixture Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none() && run.state().active_combat.is_none());
    run.state.potions = vec![None; 3];
    run.state.deck = deck.iter().enumerate().map(|(index, (id, upgrades, misc))| CardInstance {
        instance: 100 + index as u64, id: *id, upgrades: *upgrades, misc: *misc }).collect();
    run.state.next_instance = 100 + deck.len() as u64;
    run
}

fn cards(ids: &[CardId]) -> Vec<(CardId, u32, i32)> { ids.iter().map(|id| (*id, 0, 0)).collect() }

/// Install a one-node room whose screen offers `relic` (as a reward row, or a shop row priced 150).
fn offer(run: &mut Run, relic: RelicId, shop: bool) -> ChoiceId {
    let node = crate::run::NodeId(1);
    run.state.act = 1;
    run.state.floor = 1;
    run.state.map = Some(RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![MapNode {
            id: node, floor: 1, label: "lane J equip fixture", edges: vec![], x: 0, y: 0,
            kind: if shop { MapNodeKind::Shop } else { MapNodeKind::Monster }, hidden_kind: None, visited: true,
            revealed: true, encounter: None, burning: false,
        }],
        ..RunMap::default()
    });
    let (kind, price, leave) = if shop { (RoomViewKind::Shop, Some(150), RoomChoiceKind::Leave) }
        else { (RoomViewKind::Rewards, None, RoomChoiceKind::Proceed) };
    if shop { run.state.gold = 150; }
    let choices = vec![
        RoomChoiceView { id: ChoiceId(100), kind: RoomChoiceKind::Relic(relic), price, status: RoomChoiceStatus::Offered },
        RoomChoiceView { id: ChoiceId(101), kind: leave, price: None, status: RoomChoiceStatus::Offered },
    ];
    run.next_choice = 102;
    let scenario = run.state.room_loop.as_ref().map_or(RoomScenario::M5Act1Captured, |room| room.scenario);
    run.state.room_loop = Some(RoomLoop {
        scenario, act1: None, act2: None, act3: None, act4: None, m10_elite_relic_cursor: 0,
        screen: RoomView { kind, choices }, purge_cost: 75, reward_parent: None, active_rewards: None,
        boss: None, transition: None, act2_boss_order: None, act3_boss_order: None,
        joust: None, joust_visits: 0, act3_event: None, act3_end: None, standard_event: None,
    });
    ChoiceId(100)
}

/// Claim `relic` through the reward owner.
fn claim(run: &mut Run, relic: RelicId) {
    let id = offer(run, relic, false);
    run.step(&GameAction::Choose(id)).unwrap_or_else(|error| panic!("claim {relic:?}: {error:?}"));
    assert!(run.state.relics.contains(&relic));
}

fn screen(run: &Run) -> RoomView { run.state.room_loop.as_ref().unwrap().screen.clone() }

fn kinds(run: &Run) -> Vec<RoomChoiceKind> { screen(run).choices.into_iter().map(|row| row.kind).collect() }

fn row(run: &Run, wanted: impl Fn(&RoomChoiceKind) -> bool) -> ChoiceId {
    screen(run).choices.into_iter().find(|row| wanted(&row.kind) && row.status == RoomChoiceStatus::Offered)
        .unwrap_or_else(|| panic!("no offered row in {:?}", kinds(run))).id
}

/// Select the pending candidate whose captured card has `instance`.
fn select(run: &mut Run, instance: u64) {
    let token = run.state.pending.as_ref().expect("open equip selection").selection.candidates.iter()
        .find(|candidate| candidate.card.instance == instance).unwrap_or_else(|| panic!("{instance} is no candidate")).id;
    run.step(&GameAction::Select(token)).unwrap_or_else(|error| panic!("select {instance}: {error:?}"));
}

fn candidates(run: &Run) -> Vec<u64> {
    run.state.pending.as_ref().expect("open equip selection").selection.candidates.iter()
        .map(|candidate| candidate.card.instance).collect()
}

fn instances(run: &Run) -> Vec<u64> { run.state.deck.iter().map(|card| card.instance).collect() }

/// Leave the fixture room and enter the next complete combat through the shared Run owner.
fn enter_combat(run: &mut Run) {
    run.state.room_loop = None;
    run.state.map = None;
    next_complete_combat(run, EncounterId::Cultist).unwrap_or_else(|error| panic!("enter combat: {error:?}"));
}

fn opening_hand(run: &Run) -> Vec<crate::combat::LiveCardView> {
    run.state().combat.as_ref().expect("live combat").public_full().unwrap().hand
}

/// The bottle's literal: the selected instance (basic included) is linked once, dealt first in the
/// opening draw, and is not Innate.
fn bottle_case(relic: RelicId, deck: &[CardId], eligible: &[usize], selected: usize) {
    let mut run = settled(&cards(deck));
    claim(&mut run, relic);
    let eligible: Vec<u64> = eligible.iter().map(|index| 100 + *index as u64).collect();
    assert_eq!(candidates(&run), eligible, "every purgeable card of the type, basics included");
    let chosen = 100 + selected as u64;
    select(&mut run, chosen);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.relic_cards, BTreeMap::from([(relic, chosen)]), "one link to the selected instance");
    enter_combat(&mut run);
    let hand = opening_hand(&run);
    assert_eq!(hand.len(), 5);
    assert_eq!(hand[0].instance.0, chosen, "the bottled instance is dealt first (initializeDeck top placement)");
    assert_eq!(hand.iter().filter(|card| card.instance.0 == chosen).count(), 1);
    // Powers publish no card values; where values are public the Innate flag stays false.
    assert!(hand[0].values.as_ref().is_none_or(|values| !values.innate), "bottling is not the Innate keyword");
}

#[test]
fn m27_bottled_flame_selected_attack_innate() {
    let mut deck = vec![CardId::Anger, CardId::Strike];
    deck.extend([CardId::Defend; 14]);
    bottle_case(RelicId::BottledFlame, &deck, &[0, 1], 1);
}

#[test]
fn m27_bottled_lightning_selected_skill_innate() {
    let mut deck = vec![CardId::ShrugItOff, CardId::Defend];
    deck.extend([CardId::Strike; 14]);
    bottle_case(RelicId::BottledLightning, &deck, &[0, 1], 1);
}

#[test]
fn m27_bottled_tornado_selected_power_innate() {
    let mut deck = vec![CardId::Inflame, CardId::Metallicize];
    deck.extend([CardId::Strike; 14]);
    bottle_case(RelicId::BottledTornado, &deck, &[0, 1], 0);
}

/// A transformed or removed linked card leaves a stale link: no card gets the bottle's placement,
/// and the replacement instance is not bottled.
fn bottle_boundary(relic: RelicId, linked: CardId, filler: CardId) {
    for transform in [true, false] {
        let mut deck = vec![linked, linked];
        deck.extend([filler; 14]);
        let mut run = settled(&cards(&deck));
        claim(&mut run, relic);
        select(&mut run, 100);
        assert_eq!(run.state.relic_cards.get(&relic), Some(&100));
        let replacement = if transform {
            Some(crate::run::mutations::transform_card(&mut run.state, 100, false, &[27, 1]).expect("transform linked card"))
        } else {
            crate::run::mutations::remove_card(&mut run.state, 100).expect("remove linked card");
            None
        };
        assert!(!instances(&run).contains(&100));
        if let Some(created) = replacement {
            assert_ne!(run.state.relic_cards.get(&relic), Some(&created), "the new instance is not bottled");
        }
        enter_combat(&mut run);
        let context = run.state().combat.as_ref().unwrap().item_context().unwrap();
        let hand = opening_hand(&run);
        assert_eq!(hand.len(), 5, "a stale link places 0 extra cards");
        assert!(hand.iter().all(|card| !crate::items::relic_effects::structural::bottled(context, card.instance.0)));
        // 16 cards (transform keeps the count; removal leaves 15) minus the 5-card opening hand.
        assert_eq!(run.state().combat.as_ref().unwrap().public_full().unwrap().draw_count, if transform { 11 } else { 10 });
    }
}

#[test]
fn m27_bottled_flame_boundary() { bottle_boundary(RelicId::BottledFlame, CardId::Anger, CardId::Defend); }

#[test]
fn m27_bottled_lightning_boundary() { bottle_boundary(RelicId::BottledLightning, CardId::ShrugItOff, CardId::Strike); }

#[test]
fn m27_bottled_tornado_boundary() { bottle_boundary(RelicId::BottledTornado, CardId::Inflame, CardId::Strike); }

#[test]
fn m27_astrolabe_three_transform_upgrade() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Strike, CardId::Defend, CardId::Defend, CardId::Bash,
        CardId::Anger]));
    claim(&mut run, RelicId::Astrolabe);
    assert_eq!(candidates(&run), vec![100, 101, 102, 103, 104, 105]);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 3);
    for instance in [100, 102, 104] { select(&mut run, instance); }
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.deck.len(), 6);
    for original in [100, 102, 104] { assert!(!instances(&run).contains(&original), "original {original} removed"); }
    let kept: Vec<_> = run.state.deck.iter().filter(|card| [101, 103, 105].contains(&card.instance)).cloned().collect();
    assert_eq!(kept.iter().map(|card| (card.id, card.upgrades)).collect::<Vec<_>>(),
        vec![(CardId::Strike, 0), (CardId::Defend, 0), (CardId::Anger, 0)], "unchosen cards untouched");
    let created: Vec<_> = run.state.deck.iter().filter(|card| card.instance >= 106).collect();
    assert_eq!(created.len(), 3, "each chosen card transformed once");
    assert!(created.iter().all(|card| card.upgrades == 1), "each result upgraded once: {created:?}");
}

#[test]
fn m27_astrolabe_boundary() {
    // Three purgeable cards and an unpurgeable curse: no selection; all three are transformed.
    let mut run = settled(&cards(&[CardId::Strike, CardId::CurseOfTheBell, CardId::Defend, CardId::Bash]));
    claim(&mut run, RelicId::Astrolabe);
    assert!(run.state.pending.is_none(), "three or fewer eligible cards open no selection");
    assert_eq!(instances(&run)[0], 101, "the curse is kept");
    assert_eq!(run.state.deck.len(), 4);
    assert!(run.state.deck[1..].iter().all(|card| card.instance >= 104 && card.upgrades == 1),
        "originals are not upgraded in place: {:?}", run.state.deck);
    // An empty purgeable group opens nothing and changes nothing.
    let mut empty = settled(&cards(&[CardId::CurseOfTheBell, CardId::AscendersBane]));
    let before = empty.state.deck.clone();
    claim(&mut empty, RelicId::Astrolabe);
    assert!(empty.state.pending.is_none());
    assert_eq!(empty.state.deck, before);
}

#[test]
fn m27_dollys_mirror_duplicate_card_instance() {
    let mut run = settled(&[(CardId::Strike, 0, 0), (CardId::RitualDagger, 1, 7), (CardId::Defend, 0, 0)]);
    claim(&mut run, RelicId::DollysMirror);
    assert_eq!(candidates(&run), vec![100, 101, 102], "master-deck one-card selection");
    select(&mut run, 101);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.deck.len(), 4);
    let copy = run.state.deck.last().unwrap();
    assert_eq!((copy.id, copy.upgrades, copy.misc), (CardId::RitualDagger, 1, 7));
    assert_eq!(copy.instance, 103, "a new physical instance");
    assert_eq!(run.state.deck[1], CardInstance { instance: 101, id: CardId::RitualDagger, upgrades: 1, misc: 7 });
}

#[test]
fn m27_dollys_mirror_boundary() {
    let mut run = settled(&[(CardId::Strike, 0, 0), (CardId::RitualDagger, 1, 7), (CardId::Defend, 0, 0)]);
    run.state.relics.push(RelicId::BottledFlame);
    run.state.relic_cards.insert(RelicId::BottledFlame, 101);
    claim(&mut run, RelicId::DollysMirror);
    select(&mut run, 101);
    let copy = run.state.deck.last().unwrap().clone();
    assert_eq!((copy.upgrades, copy.misc), (1, 7));
    assert_eq!(run.state.relic_cards, BTreeMap::from([(RelicId::BottledFlame, 101)]), "the bottle flag is not copied");
}

#[test]
fn m27_orrery_five_card_rewards() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend, CardId::Bash]));
    let id = offer(&mut run, RelicId::Orrery, true);
    run.step(&GameAction::Choose(id)).expect("buy Orrery");
    assert_eq!(kinds(&run), [vec![RoomChoiceKind::CardReward; 5], vec![RoomChoiceKind::Proceed]].concat());
    assert_eq!(run.state.deck.len(), 3, "no card is auto-chosen");
    for _ in 0..5 {
        let parent = row(&run, |kind| *kind == RoomChoiceKind::CardReward);
        run.step(&GameAction::Choose(parent)).expect("open card reward");
        assert_eq!(screen(&run).choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).count(), 3);
        let take = row(&run, |kind| matches!(kind, RoomChoiceKind::Card { .. }));
        run.step(&GameAction::Choose(take)).expect("take one card");
    }
    assert_eq!(run.state.deck.len(), 8, "five separate rewards, one card each");
    run.step(&GameAction::Choose(row(&run, |kind| *kind == RoomChoiceKind::Proceed))).expect("proceed");
    assert_eq!(screen(&run).kind, RoomViewKind::Shop);
}

#[test]
fn m27_orrery_boundary() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend, CardId::Bash]));
    claim(&mut run, RelicId::Orrery);
    assert_eq!(kinds(&run).iter().filter(|kind| **kind == RoomChoiceKind::CardReward).count(), 5);
    let mut children = vec![];
    for _ in 0..5 {
        run.step(&GameAction::Choose(row(&run, |kind| *kind == RoomChoiceKind::CardReward))).expect("open");
        children.push(screen(&run));
        run.step(&GameAction::Choose(row(&run, |kind| *kind == RoomChoiceKind::Skip))).expect("skip");
    }
    for (index, child) in children.iter().enumerate() {
        assert!(children[..index].iter().all(|other| other.choices.iter().all(|theirs|
            child.choices.iter().all(|mine| mine.id != theirs.id))), "each offer row is generated individually");
    }
    assert_eq!(run.state.deck.len(), 3, "skipping all five rewards adds nothing");
}

#[test]
fn m27_calling_bell_curse_three_tiers() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend]));
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![RelicId::Anchor], vec![RelicId::BottledFlame, RelicId::BagOfPreparation], vec![RelicId::DeadBranch], vec![], vec![]));
    claim(&mut run, RelicId::CallingBell);
    assert_eq!(run.state.deck.iter().filter(|card| card.id == CardId::CurseOfTheBell).count(), 1);
    assert_eq!(kinds(&run), vec![RoomChoiceKind::Relic(RelicId::Anchor), RoomChoiceKind::Relic(RelicId::BagOfPreparation),
        RoomChoiceKind::Relic(RelicId::DeadBranch), RoomChoiceKind::Proceed], "Common, Uncommon, Rare; no bottle from the Bell");
    for relic in [RelicId::Anchor, RelicId::BagOfPreparation, RelicId::DeadBranch] {
        run.step(&GameAction::Choose(row(&run, |kind| *kind == RoomChoiceKind::Relic(relic)))).expect("claim Bell relic");
        assert!(run.state.relics.contains(&relic));
    }
}

#[test]
fn m27_calling_bell_boundary() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend]));
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![RelicId::Anchor], vec![RelicId::BagOfPreparation], vec![RelicId::DeadBranch], vec![], vec![]));
    claim(&mut run, RelicId::CallingBell);
    // The curse is resolved before the three relic offers are opened (offers still unclaimed).
    assert_eq!(run.state.deck.last().map(|card| card.id), Some(CardId::CurseOfTheBell));
    assert_eq!(run.state.relics, vec![RelicId::CallingBell]);
    assert_eq!(screen(&run).choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))
        && row.status == RoomChoiceStatus::Offered).count(), 3);
    assert!(!kinds(&run).contains(&RoomChoiceKind::CardReward), "the generated card row is removed");
}

#[test]
fn m27_pandoras_box_starter_card_transform() {
    let deck = [CardId::Strike, CardId::Strike, CardId::Defend, CardId::Defend, CardId::PerfectedStrike, CardId::Bash];
    let mut run = settled(&cards(&deck));
    claim(&mut run, RelicId::PandorasBox);
    assert!(run.state.pending.is_none(), "no chooser");
    assert_eq!(run.state.deck.len(), 6);
    for removed in 100..104 { assert!(!instances(&run).contains(&removed), "starter {removed} transformed"); }
    assert!(run.state.deck.iter().any(|card| card.instance == 104 && card.id == CardId::PerfectedStrike));
    assert!(run.state.deck.iter().any(|card| card.instance == 105 && card.id == CardId::Bash));
    let created: Vec<_> = run.state.deck.iter().filter(|card| card.instance >= 106).collect();
    assert_eq!(created.len(), 4);
    assert!(created.iter().all(|card| !matches!(card.id, CardId::Strike | CardId::Defend)), "{created:?}");
}

#[test]
fn m27_pandoras_box_boundary() {
    let deck = [CardId::TwinStrike, CardId::PerfectedStrike, CardId::Strike, CardId::Defend];
    let mut run = settled(&cards(&deck));
    claim(&mut run, RelicId::PandorasBox);
    assert_eq!(run.state.deck[..2].iter().map(|card| (card.instance, card.id)).collect::<Vec<_>>(),
        vec![(100, CardId::TwinStrike), (101, CardId::PerfectedStrike)], "nonstarter Strikes untouched");
    assert!(!instances(&run).contains(&102) && !instances(&run).contains(&103));
    // No starter card: nothing changes.
    let mut none = settled(&cards(&[CardId::Bash, CardId::Anger]));
    let before = none.state.deck.clone();
    claim(&mut none, RelicId::PandorasBox);
    assert_eq!(none.state.deck, before);
}

#[test]
fn m27_tiny_house_equip_reward_bundle() {
    let mut run = settled(&[(CardId::Strike, 1, 0), (CardId::Defend, 1, 0), (CardId::Bash, 0, 0)]);
    run.state.hp = 40;
    run.state.max_hp = 80;
    let gold = run.state.gold;
    claim(&mut run, RelicId::TinyHouse);
    assert_eq!((run.state.hp, run.state.max_hp), (45, 85), "max HP +5 through increaseMaxHp");
    assert_eq!(run.state.deck.iter().map(|card| card.upgrades).collect::<Vec<_>>(), vec![1, 1, 1], "the one eligible card upgraded");
    let rows = kinds(&run);
    assert_eq!(rows.len(), 4, "{rows:?}");
    assert_eq!(rows[0], RoomChoiceKind::CardReward);
    assert!(matches!(rows[1], RoomChoiceKind::Potion(_)));
    assert_eq!(rows[2..], [RoomChoiceKind::Gold(50), RoomChoiceKind::Proceed]);
    run.step(&GameAction::Choose(row(&run, |kind| matches!(kind, RoomChoiceKind::Gold(_))))).expect("claim gold");
    assert_eq!(run.state.gold, gold + 50);
    run.step(&GameAction::Choose(row(&run, |kind| matches!(kind, RoomChoiceKind::Potion(_))))).expect("claim potion");
    assert_eq!(run.state.potions.iter().flatten().count(), 1);
}

#[test]
fn m27_tiny_house_boundary() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend]));
    let (gold, deck_len) = (run.state.gold, run.state.deck.len());
    claim(&mut run, RelicId::TinyHouse);
    assert_eq!(run.state.max_hp, 85, "equip itself grants +5 max HP");
    assert_eq!(run.state.gold, gold, "the 50 gold is a reward row, not an immediate gain (source law)");
    assert_eq!(run.state.potions.iter().flatten().count(), 0, "potion offer pending until claimed");
    assert_eq!(run.state.deck.len(), deck_len, "card reward pending until claimed");
    assert_eq!(run.state.deck.iter().filter(|card| card.upgrades == 1).count(), 1, "one random upgrade");
}

#[test]
fn m27_cauldron_five_potion_offers() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend]));
    run.state.relics.push(RelicId::PotionBelt);
    run.state.potions = vec![None; 5];
    claim(&mut run, RelicId::Cauldron);
    let rows = kinds(&run);
    assert_eq!(rows.len(), 6, "{rows:?}");
    assert!(rows[..5].iter().all(|kind| matches!(kind, RoomChoiceKind::Potion(_))));
    assert_eq!(rows[5], RoomChoiceKind::Proceed, "no automatically generated card row");
    let pool = crate::items::potion_generation::candidates(Character::Ironclad, false).unwrap();
    assert!(rows[..5].iter().all(|kind| matches!(kind, RoomChoiceKind::Potion(id) if pool.contains(id))));
    for _ in 0..5 {
        run.step(&GameAction::Choose(row(&run, |kind| matches!(kind, RoomChoiceKind::Potion(_))))).expect("claim potion");
    }
    assert_eq!(run.state.potions.iter().flatten().count(), 5);
}

#[test]
fn m27_cauldron_boundary() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend]));
    claim(&mut run, RelicId::Cauldron);
    assert_eq!(kinds(&run).iter().filter(|kind| matches!(kind, RoomChoiceKind::Potion(_))).count(), 5, "five offers");
    for _ in 0..3 {
        run.step(&GameAction::Choose(row(&run, |kind| matches!(kind, RoomChoiceKind::Potion(_))))).expect("claim potion");
    }
    let fourth = row(&run, |kind| matches!(kind, RoomChoiceKind::Potion(_)));
    assert!(run.step(&GameAction::Choose(fourth)).is_err(), "a full belt keeps the offer");
    assert_eq!(run.state.potions.iter().flatten().count(), 3);
    assert_eq!(screen(&run).choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Potion(_))
        && row.status == RoomChoiceStatus::Offered).count(), 2);
}

#[test]
fn m27_necronomicon_pickup_adds_necronomicurse() {
    let mut run = settled(&cards(&[CardId::Strike, CardId::Defend]));
    claim(&mut run, RelicId::Necronomicon);
    assert_eq!(run.state.deck.iter().map(|card| card.id).collect::<Vec<_>>(),
        vec![CardId::Strike, CardId::Defend, CardId::Necronomicurse]);
}
