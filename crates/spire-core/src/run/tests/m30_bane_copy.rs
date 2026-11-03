//! M30 Standard A10 Ascender's Bane copy and obtain-hook coverage.
use super::super::*;
use crate::combat::{CardInstanceId, rng::Partition};
use crate::content::{CardId, Character, EventId, RelicId};
use crate::run::rooms::{RoomChoiceKind, RoomChoiceStatus, RoomChoiceView, RoomLoop, RoomView, RoomViewKind};
use crate::run::{ChoiceId, CreationProfile, GameAction, MapNode, MapNodeKind, Run, RunConfig, RunMap, RunRng, RoomScenario};

fn standard_a10(seed: u8) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 10 },
        profile,
        RunRng::new([seed; 32], Partition::Test, 0),
        RoomScenario::Standard,
    ).unwrap_or_else(|error| panic!("Standard A10 creation: {error:?}"))
}

fn bane_instance(run: &Run) -> u64 {
    run.state.deck.iter().find(|card| card.id == CardId::AscendersBane)
        .unwrap_or_else(|| panic!("Standard A10 deck has no Bane: {:?}", run.state.deck))
        .instance
}

/// Offer one relic through the Run reward choice so equip and selection work use their normal owners.
fn claim_relic(run: &mut Run, relic: RelicId) {
    let node = crate::run::NodeId(1);
    run.state.act = 1;
    run.state.floor = 1;
    run.state.map = Some(RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![MapNode {
            id: node, floor: 1, label: "M30 Bane copy fixture", edges: vec![], x: 0, y: 0,
            kind: MapNodeKind::Monster, hidden_kind: None, visited: true, revealed: true,
            encounter: None, burning: false,
        }],
        ..RunMap::default()
    });
    run.next_choice = 102;
    run.state.room_loop = Some(RoomLoop {
        // Match the isolated one-node reward fixture used by the existing M27 relic-selection tests.
        scenario: RoomScenario::M5Act1Captured, act1: None, act2: None, act3: None, act4: None,
        m10_elite_relic_cursor: 0,
        screen: RoomView { kind: RoomViewKind::Rewards, choices: vec![
            RoomChoiceView { id: ChoiceId(100), kind: RoomChoiceKind::Relic(relic), price: None,
                status: RoomChoiceStatus::Offered },
            RoomChoiceView { id: ChoiceId(101), kind: RoomChoiceKind::Proceed, price: None,
                status: RoomChoiceStatus::Offered },
        ] },
        purge_cost: 75, reward_parent: None, active_rewards: None, boss: None, transition: None,
        act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0,
        act3_event: None, act3_end: None, standard_event: None,
    });
    run.step(&GameAction::Choose(ChoiceId(100)))
        .unwrap_or_else(|error| panic!("claim {relic:?}: {error:?}"));
    assert!(run.state.relics.contains(&relic), "Run reward path owns {relic:?}");
}

fn dolly_select_bane(run: &mut Run) -> u64 {
    let source = bane_instance(run);
    claim_relic(run, RelicId::DollysMirror);
    let selection = run.state.pending.as_ref().expect("Dolly's Mirror opens a card selection");
    assert!(selection.selection.candidates.iter().any(|candidate|
        candidate.card.instance == source && candidate.card.id == CardId::AscendersBane),
        "Dolly's Mirror offers the Bane found by card ID");
    let token = selection.selection.candidates.iter().find(|candidate|
        candidate.card.instance == source && candidate.card.id == CardId::AscendersBane)
        .expect("Bane candidate").id;
    run.step(&GameAction::Select(token)).unwrap_or_else(|error| panic!("select Dolly Bane: {error:?}"));
    source
}

fn duplicator_run(seed: u8) -> Run {
    let mut run = standard_a10(seed);
    let skip = rooms::public(&run).expect("Neow screen").choices.get(1)
        .unwrap_or_else(|| panic!("short Neow has a continue row" )).id;
    run.step(&GameAction::Choose(skip)).expect("continue to the Standard map");

    run.state.act = 2;
    rooms::standard::start_act(&mut run).expect("generate Act 2 Standard map");
    let offset = 17;
    for node in &mut run.state.map.as_mut().expect("Act 2 map").nodes { node.floor += offset; }
    run.state.floor = offset;
    run.state.event_pools = Some(rooms::standard::event_pool::EventPools {
        ordinary: vec![], shrines: vec![], one_time: vec![EventId::Duplicator], seen: vec![],
    });
    let node = run.state.map.as_mut().expect("Act 2 map").nodes.iter_mut().find(|node| node.y == 0)
        .unwrap_or_else(|| panic!("Act 2 map has a first-row node"));
    node.kind = MapNodeKind::Event;
    let node_id = node.id;
    run.step(&GameAction::Move(node_id)).expect("enter the Standard Duplicator event");
    assert_eq!(rooms::public(&run).expect("Duplicator screen").kind, RoomViewKind::Event(EventId::Duplicator));
    run
}

fn duplicate_bane_at_duplicator(run: &mut Run) -> u64 {
    let source = bane_instance(run);
    let option = rooms::public(run).expect("Duplicator screen").choices.iter()
        .find(|choice| choice.kind == RoomChoiceKind::EventOption { stage: 0, option: 0 }
            && choice.status == RoomChoiceStatus::Offered)
        .unwrap_or_else(|| panic!("Duplicator offers its copy option")).id;
    run.step(&GameAction::Choose(option)).expect("open Duplicator's card selection");
    let pending = run.state.pending.as_ref().expect("Duplicator opens a card selection");
    let candidate = pending.selection.candidates.iter().find(|candidate|
        candidate.card.id == CardId::AscendersBane && candidate.card.instance == source)
        .unwrap_or_else(|| panic!("Duplicator offers Bane instance {source}"));
    run.step(&GameAction::Select(candidate.id)).unwrap_or_else(|error| panic!("select event Bane: {error:?}"));
    source
}

fn copied_bane_instance(run: &Run, original: u64) -> u64 {
    run.state.deck.iter().find(|card| card.id == CardId::AscendersBane && card.instance != original)
        .unwrap_or_else(|| panic!("no copied Bane in deck: {:?}", run.state.deck)).instance
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DollysMirror.java:44–56; m30-source-check-v1.md §E (purgeable and public remove paths), §C (Ethereal); verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:961–967.
#[test]
fn m30_bane_copy_dolly_appends_unremovable_ethereal() {
    let mut run = standard_a10(0x31);
    let original = bane_instance(&run);
    let before_len = run.state.deck.len();
    let selected = dolly_select_bane(&mut run);
    assert_eq!(selected, original);
    assert_eq!(run.state.deck.len(), before_len + 1, "Dolly appends one physical copy");
    let copied = copied_bane_instance(&run, original);
    assert_ne!(copied, original);
    assert_eq!(run.state.deck.last().map(|card| (card.id, card.instance)),
        Some((CardId::AscendersBane, copied)), "the new copy is appended to the raw master deck");

    let live = crate::combat::complete_live_card(CardId::AscendersBane, CardInstanceId(copied), 0)
        .expect("catalog completes a permanent Bane");
    assert!(live.local.ethereal, "the copied live card keeps Bane's Ethereal flag");

    run.state.room_loop = None;
    let removal = run.remove_card(copied);
    assert!(matches!(removal.as_ref(), Err(crate::run::RunError::InvalidAction)),
        "the shared public remove path rejects the copied NonremovableCurse; observed {removal:?}");
    assert!(run.state.deck.iter().any(|card| card.instance == copied && card.id == CardId::AscendersBane),
        "rejected removal leaves the copy in the master deck");
}

// Source: verification/2026-11-03-m28f-v1/decompiled/Duplicator/com/megacrit/cardcrawl/events/shrines/Duplicator.java:90–100; m30-source-check-v1.md §E (Duplicator uses the raw master deck).
#[test]
fn m30_bane_copy_duplicator_appends() {
    let mut run = duplicator_run(0x32);
    let original = bane_instance(&run);
    let before_len = run.state.deck.len();
    let selected = duplicate_bane_at_duplicator(&mut run);
    assert_eq!(selected, original);
    assert_eq!(run.state.deck.len(), before_len + 1, "Duplicator obtains one Bane copy");
    let copied = copied_bane_instance(&run, original);
    assert_eq!(run.state.deck.last().map(|card| (card.id, card.instance)),
        Some((CardId::AscendersBane, copied)), "the event appends the new Bane");
}

// Source: m30-source-check-v1.md §G; verification/2026-11-03-m28f-v1/decompiled/FastCardObtainEffect/com/megacrit/cardcrawl/vfx/FastCardObtainEffect.java:26–30, 40–48.
#[test]
fn m30_bane_copy_omamori_absorbs() {
    let mut run = standard_a10(0x33);
    claim_relic(&mut run, RelicId::Omamori);
    claim_relic(&mut run, RelicId::DarkstonePeriapt);
    claim_relic(&mut run, RelicId::CeramicFish);
    let charges = *run.state.relic_counters.get(&RelicId::Omamori).expect("Omamori starts charged");
    assert!(charges > 0);
    let before_count = run.state.deck.iter().filter(|card| card.id == CardId::AscendersBane).count();
    let before_hp = run.state.max_hp;
    let before_gold = run.state.gold;

    dolly_select_bane(&mut run);

    assert_eq!(run.state.deck.iter().filter(|card| card.id == CardId::AscendersBane).count(), before_count,
        "Omamori absorbs the copied Bane");
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&(charges - 1)), "one Omamori charge is spent");
    assert_eq!(run.state.max_hp, before_hp, "negated Bane does not trigger Darkstone Periapt");
    assert_eq!(run.state.gold, before_gold, "negated Bane does not trigger Ceramic Fish");
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DarkstonePeriapt.java:32–40; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/CeramicFish.java:28–39.
#[test]
fn m30_bane_copy_darkstone_and_fish_react() {
    let mut run = standard_a10(0x34);
    claim_relic(&mut run, RelicId::DarkstonePeriapt);
    claim_relic(&mut run, RelicId::CeramicFish);
    let before_hp = run.state.max_hp;
    let before_gold = run.state.gold;
    let before_len = run.state.deck.len();
    let original = dolly_select_bane(&mut run);

    assert_eq!(run.state.max_hp, before_hp + 6, "Darkstone Periapt gains 6 max HP for the copied Curse");
    assert_eq!(run.state.gold, before_gold + 9, "Ceramic Fish gains 9 gold for the obtained copy");
    assert_eq!(run.state.deck.len(), before_len + 1, "one Bane copy is added");
    let copied = copied_bane_instance(&run, original);
    assert_eq!(run.state.deck.last().map(|card| (card.id, card.instance)),
        Some((CardId::AscendersBane, copied)), "Dolly appends the Bane copy after obtain hooks");
}
