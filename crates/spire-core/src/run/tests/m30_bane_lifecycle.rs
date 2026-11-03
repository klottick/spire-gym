//! M30 A10 Ascender's Bane lifecycle matrix. Source literals are from the M30 source check §E–G.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{pools::{self, CardPool}, CardId, CardType, Catalog, Character, EventId, EventPool, RelicId};
use crate::run::{CardInstance, ChoiceId, GameAction, MapNodeKind, NeowReward, RoomScenario, Run, RunRng};
use crate::run::rooms::{RoomChoiceKind, RoomChoiceStatus, RoomView, RoomViewKind};

const BANE: CardId = CardId::AscendersBane;
const CURSES: [CardId; 3] = [CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn a10(seed: u8, full_neow: bool) -> Run {
    let profile = CreationProfile { neow_eligible: full_neow, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 10 }, profile, rng(seed), RoomScenario::Standard)
        .unwrap_or_else(|error| panic!("A10 Standard: {error:?}"))
}
fn short_neow_to_map(run: &mut Run) {
    let view = rooms::public(run).unwrap();
    assert_eq!(view.kind, RoomViewKind::Neow);
    let choice = view.choices.get(1).expect("short Neow second choice").id;
    run.step(&GameAction::Choose(choice)).unwrap();
    assert_eq!(rooms::public(run).unwrap().kind, RoomViewKind::Map);
}
fn map_run(seed: u8) -> Run {
    let mut run = a10(seed, false);
    assert!(run.state.deck.iter().any(|card| card.id == BANE), "A10 constructor supplies Bane");
    short_neow_to_map(&mut run);
    run
}
fn bane_instance(run: &Run) -> u64 {
    run.state.deck.iter().find(|card| card.id == BANE).expect("constructor Bane").instance
}
fn assert_bane(run: &Run, instance: u64) {
    assert!(run.state.deck.iter().any(|card| card.id == BANE && card.instance == instance), "A10 Bane remains in master deck");
}
/// Keep the constructor-created Bane and replace the other cards with the requested test fixture.
fn set_deck(run: &mut Run, ids: &[CardId]) {
    let bane = run.state.deck.iter().find(|card| card.id == BANE).expect("constructor Bane").clone();
    let mut deck = vec![bane];
    let mut next = run.state.next_instance.max(deck[0].instance + 1);
    for id in ids.iter().copied().filter(|id| *id != BANE) {
        deck.push(CardInstance { id, instance: next, upgrades: 0, misc: 0 });
        next += 1;
    }
    run.state.deck = deck;
    run.state.next_instance = next;
    run.state.relic_cards.clear();
}
fn add_bottle(run: &mut Run, relic: RelicId, instance: u64) {
    if !run.state.relics.contains(&relic) { run.state.relics.push(relic); }
    run.state.relic_cards.insert(relic, instance);
}
fn room(run: &Run) -> RoomView { rooms::public(run).unwrap() }
fn row_id(run: &Run, kind: &RoomChoiceKind) -> ChoiceId {
    room(run).choices.into_iter().find(|row| &row.kind == kind && row.status == RoomChoiceStatus::Offered)
        .unwrap_or_else(|| panic!("missing offered row {kind:?}: {:?}", room(run).choices)).id
}
fn choose_kind(run: &mut Run, kind: &RoomChoiceKind) {
    let id = row_id(run, kind);
    run.step(&GameAction::Choose(id)).unwrap_or_else(|error| panic!("choose {kind:?}: {error:?}"));
}
fn choose_option(run: &mut Run, stage: u8, option: u8) {
    choose_kind(run, &RoomChoiceKind::EventOption { stage, option });
}
fn select_instance(run: &mut Run, instance: u64) {
    let token = run.state.pending.as_ref().expect("selection open").selection.candidates.iter()
        .find(|candidate| candidate.card.instance == instance)
        .unwrap_or_else(|| panic!("instance {instance} not offered: {:?}", run.state.pending.as_ref().unwrap().selection.candidates))
        .id;
    run.step(&GameAction::Select(token)).unwrap_or_else(|error| panic!("select {instance}: {error:?}"));
}
fn selection_cards(run: &Run) -> Vec<(CardId, u64)> {
    run.state.pending.as_ref().expect("selection open").selection.candidates.iter()
        .map(|candidate| (candidate.card.id, candidate.card.instance)).collect()
}
fn assert_selection_ids(run: &Run, expected: &[CardId]) {
    let mut got = selection_cards(run).into_iter().map(|(id, _)| id).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    got.sort(); expected.sort();
    assert_eq!(got, expected, "candidate card identities");
}
fn resolve_one(run: &mut Run, instance: u64) { select_instance(run, instance); }
fn first_candidate(run: &Run) -> u64 { selection_cards(run).first().expect("candidate").1 }
fn resolve_first(run: &mut Run) { let instance = first_candidate(run); resolve_one(run, instance); }
fn enter_world_room(run: &mut Run, kind: MapNodeKind) {
    let map = run.state.map.as_mut().expect("standard map");
    let node = map.nodes.iter_mut().find(|node| node.y == 0).expect("first map row");
    node.kind = kind; node.hidden_kind = None;
    let id = node.id;
    run.step(&GameAction::Move(id)).unwrap_or_else(|error| panic!("enter {kind:?}: {error:?}"));
}
fn at_event_with(event: EventId, act: u8, seed: u8, prepare: impl FnOnce(&mut Run)) -> Run {
    let mut run = map_run(seed);
    if act > 1 {
        run.state.act = act;
        rooms::standard::start_act(&mut run).unwrap();
        let offset = [0, 0, 17, 34][usize::from(act)];
        for node in &mut run.state.map.as_mut().unwrap().nodes { node.floor += offset; }
        run.state.floor = offset;
    }
    prepare(&mut run);
    run.state.gold = 500;
    let pool = Catalog::standard().event(event).expect("event catalog row").pool;
    let singleton = vec![event];
    run.state.event_pools = Some(match pool {
        EventPool::Ordinary => rooms::standard::event_pool::EventPools { ordinary: singleton, shrines: vec![], one_time: vec![], seen: vec![] },
        EventPool::Shrine => rooms::standard::event_pool::EventPools { ordinary: vec![], shrines: singleton, one_time: vec![], seen: vec![] },
        EventPool::OneTime => rooms::standard::event_pool::EventPools { ordinary: vec![], shrines: vec![], one_time: singleton, seen: vec![] },
        EventPool::Start => panic!("Neow is not a map event"),
    });
    let map = run.state.map.as_mut().unwrap();
    let node = map.nodes.iter_mut().find(|node| node.y == 0).expect("first map row");
    node.kind = MapNodeKind::Event; node.hidden_kind = None;
    let id = node.id;
    run.step(&GameAction::Move(id)).unwrap_or_else(|error| panic!("enter {event:?}: {error:?}"));
    assert_eq!(room(&run).kind, RoomViewKind::Event(event));
    run
}
fn at_event(event: EventId, act: u8, seed: u8) -> Run { at_event_with(event, act, seed, |_| {}) }
fn neow_offer(reward: NeowReward) -> (Run, ChoiceId) {
    for seed in 0..=u8::MAX {
        let run = a10(seed, true);
        if let Some(choice) = room(&run).choices.iter().find(|choice|
            matches!(choice.kind, RoomChoiceKind::NeowBlessing { reward: offered, .. } if offered == reward)) {
            return (run, choice.id);
        }
    }
    panic!("no A10 Neow offer for {reward:?} in full seed sweep")
}
fn designer_at_option(option: u8, cards: &[CardId]) -> Run {
    for seed in 0..=u8::MAX {
        let run = at_event_with(EventId::Designer, 2, seed, |run| set_deck(run, cards));
        if room(&run).choices.iter().any(|row|
            row.kind == RoomChoiceKind::EventOption { stage: 0, option } && row.status == RoomChoiceStatus::Offered) {
            return run;
        }
    }
    panic!("Designer did not expose option {option}")
}
fn open_neow_selection(reward: NeowReward, ids: &[CardId], bottle: Option<(RelicId, usize)>) -> (Run, u64) {
    let (mut run, choice) = neow_offer(reward);
    set_deck(&mut run, ids);
    if let Some((relic, deck_index)) = bottle {
        let instance = run.state.deck[deck_index].instance;
        add_bottle(&mut run, relic, instance);
    }
    let bane = bane_instance(&run);
    run.step(&GameAction::Choose(choice)).unwrap();
    (run, bane)
}
fn event_select(run: &mut Run, option: u8, instance: u64) {
    choose_option(run, 0, option);
    select_instance(run, instance);
}
fn relic_offer(run: &mut Run, relic: RelicId) {
    let node = run.state.map.as_mut().and_then(|map| {
        let node = map.nodes.iter_mut().find(|node| node.y == 0)?;
        node.visited = true; node.id.into()
    }).unwrap_or(crate::run::NodeId(0));
    if let Some(map) = run.state.map.as_mut() { map.current = Some(node); }
    let id = ChoiceId(run.next_choice);
    run.next_choice += 2;
    let active = run.state.room_loop.as_mut().expect("Standard room loop");
    active.screen = RoomView { kind: RoomViewKind::Rewards, choices: vec![
        crate::run::rooms::RoomChoiceView { id, kind: RoomChoiceKind::Relic(relic), price: None, status: RoomChoiceStatus::Offered },
        crate::run::rooms::RoomChoiceView { id: ChoiceId(id.0 + 1), kind: RoomChoiceKind::Proceed, price: None, status: RoomChoiceStatus::Offered },
    ] };
    run.step(&GameAction::Choose(id)).unwrap_or_else(|error| panic!("obtain {relic:?}: {error:?}"));
}
fn event_leave_if_present(run: &mut Run) {
    if room(run).choices.iter().any(|choice| choice.kind == RoomChoiceKind::Leave && choice.status == RoomChoiceStatus::Offered) {
        choose_kind(run, &RoomChoiceKind::Leave);
    }
}

// Source: verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:961–967
// Grade: Table
#[test]
fn m30_bane_matrix_get_purgeable() {
    let mut run = map_run(1);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike, CardId::Defend]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    // The owner every caller uses (M30 E7), not the content table behind it (final review m6).
    let ids = run.state.deck.iter().filter(|card| crate::run::mutations::removable(card.id))
        .map(|card| card.id).collect::<Vec<_>>();
    assert_eq!(ids, vec![CardId::Strike, CardId::Defend], "the raw purgeable set includes bottles and excludes the three special curses");
    assert!(ids.contains(&CardId::Strike), "bottle removal is a caller-specific filter");
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:944–955
// Grade: Table
#[test]
fn m30_bane_matrix_get_upgradable() {
    let mut run = map_run(95);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike, CardId::Defend]);
    let bane = bane_instance(&run);
    let ids = run.state.deck.iter().filter(|card| crate::run::mutations::can_upgrade(&run.state, card))
        .map(|card| card.id).collect::<Vec<_>>();
    assert_eq!(ids, vec![CardId::Strike, CardId::Defend], "upgradeable pool excludes all three special curses");
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:270
// The source opens Neow's grid on the unfiltered getPurgeableCards; the engine uses `purgeable`
// (removable and unbottled). They are equivalent because no bottle can exist before the first Neow
// choice, so this fixture has no bottle (final review m2; design §6.4).
// Grade: Run
#[test]
fn m30_bane_matrix_neow_remove_one() {
    let (mut run, bane) = open_neow_selection(NeowReward::RemoveOne, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell], None);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 1);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend]);
    resolve_first(&mut run);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:274
// Grade: Run
#[test]
fn m30_bane_matrix_neow_remove_two() {
    let (mut run, bane) = open_neow_selection(NeowReward::RemoveTwo, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Necronomicurse, CardId::CurseOfTheBell], None);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 2);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    let candidates = selection_cards(&run);
    resolve_one(&mut run, candidates[0].1);
    resolve_one(&mut run, candidates[1].1);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:303
// The source opens Neow's grid on the unfiltered getPurgeableCards; the engine uses `purgeable`
// (removable and unbottled). They are equivalent because no bottle can exist before the first Neow
// choice, so this fixture has no bottle (final review m2; design §6.4).
// Grade: Run
#[test]
fn m30_bane_matrix_neow_transform_one() {
    let (mut run, bane) = open_neow_selection(NeowReward::TransformOne, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell], None);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 1);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend]);
    resolve_first(&mut run);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:307
// Grade: Run
#[test]
fn m30_bane_matrix_neow_transform_two() {
    let (mut run, bane) = open_neow_selection(NeowReward::TransformTwo, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Necronomicurse, CardId::CurseOfTheBell], None);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 2);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    let candidates = selection_cards(&run);
    resolve_one(&mut run, candidates[0].1);
    resolve_one(&mut run, candidates[1].1);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:320
// Grade: Run
#[test]
fn m30_bane_matrix_neow_upgrade() {
    let (mut run, bane) = open_neow_selection(NeowReward::UpgradeOne, &[CardId::Strike, CardId::Defend, CardId::AscendersBane, CardId::Necronomicurse, CardId::CurseOfTheBell], None);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 1);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend]);
    resolve_first(&mut run);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:274
// Grade: Run
#[test]
fn m30_bane_matrix_neow_short_remove_two() {
    let (mut run, bane) = open_neow_selection(NeowReward::RemoveTwo, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell], None);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 2, "fixed two-card request is not clamped");
    assert_selection_ids(&run, &[CardId::Strike]);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/shop/ShopScreen.java:256, 328–337, 1007
// Grade: Run
#[test]
fn m30_bane_matrix_shop_purge() {
    let mut run = map_run(2);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    enter_world_room(&mut run, MapNodeKind::Shop);
    let purge = room(&run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
    assert_eq!(purge.status, RoomChoiceStatus::Offered);
    run.step(&GameAction::Choose(purge.id)).unwrap();
    assert_selection_ids(&run, &[CardId::Defend]);
    resolve_first(&mut run);
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/shop/ShopScreen.java:256, 328–337, 1007
// Grade: Run
#[test]
fn m30_bane_matrix_shop_purge_empty() {
    let mut run = map_run(3);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    enter_world_room(&mut run, MapNodeKind::Shop);
    let purge = room(&run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap();
    let observed = if purge.status == RoomChoiceStatus::Offered { run.step(&GameAction::Choose(purge.id)) } else { Err(RunError::InvalidAction) };
    let fault_free = run.decision().is_ok();
    assert!(purge.status == RoomChoiceStatus::Disabled && fault_free,
        "M30 ruling D1: no purgeable card disables Purge and does not fault; status={:?}, step={observed:?}", purge.status);
    let bane = bane_instance(&run);
    assert_bane(&run, bane);
    assert!(run.decision().is_ok(), "empty shop purge must not fault the Run");
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/PeacePipe.java:53
// Grade: Run
#[test]
fn m30_bane_matrix_peace_pipe_candidates() {
    let mut run = map_run(4);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    run.state.relics.push(RelicId::PeacePipe);
    enter_world_room(&mut run, MapNodeKind::Rest);
    let toke = room(&run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::Toke).unwrap();
    assert_eq!(toke.status, RoomChoiceStatus::Offered);
    run.step(&GameAction::Choose(toke.id)).unwrap();
    assert_selection_ids(&run, &[CardId::Defend]);
    resolve_first(&mut run);
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/PeacePipe.java:53
// Grade: Run
#[test]
fn m30_bane_matrix_peace_pipe_empty_status() {
    let mut run = map_run(5);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    run.state.relics.push(RelicId::PeacePipe);
    enter_world_room(&mut run, MapNodeKind::Rest);
    let toke = room(&run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::Toke).unwrap();
    assert_eq!(toke.status, RoomChoiceStatus::Disabled, "no unbottled purgeable card disables Toke");
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/vfx/campfire/CampfireTokeEffect.java:61–71; CancelButton.java:98–108
// Grade: Run
#[test]
fn m30_bane_matrix_peace_pipe_cancel() {
    let mut run = map_run(6);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    run.state.relics.push(RelicId::PeacePipe);
    enter_world_room(&mut run, MapNodeKind::Rest);
    let toke = row_id(&run, &RoomChoiceKind::Toke);
    let before = run.state.deck.clone();
    run.step(&GameAction::Choose(toke)).unwrap();
    let has_cancel = run.decision().unwrap().candidates.iter().any(|candidate| candidate.action == GameAction::Cancel);
    let cancel = run.step(&GameAction::Cancel);
    assert!(has_cancel && cancel.is_ok(), "Toke picker publishes Cancel and takes it: candidate={has_cancel}, result={cancel:?}");
    assert!(run.state.pending.is_none());
    assert_eq!(room(&run).choices.iter().find(|row| row.kind == RoomChoiceKind::Toke).unwrap().status, RoomChoiceStatus::Offered);
    assert_eq!(run.state.deck, before);
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/EmptyCage.java:49–60
// Grade: Run
#[test]
fn m30_bane_matrix_empty_cage() {
    let mut run = map_run(7);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::EmptyCage);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    assert!(selection_cards(&run).iter().any(|(_, instance)| *instance == bottled), "Empty Cage does not apply bottle filter");
    let candidates = selection_cards(&run);
    resolve_one(&mut run, candidates[0].1);
    resolve_one(&mut run, candidates[1].1);
    assert_bane(&run, bane);

    let mut empty = map_run(70);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    relic_offer(&mut empty, RelicId::EmptyCage);
    assert!(empty.state.pending.is_none(), "empty eligible group is a no-op");
    assert_eq!(empty.state.deck, before);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Astrolabe.java:46–62
// Grade: Run
#[test]
fn m30_bane_matrix_astrolabe() {
    let mut run = map_run(8);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Anger, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::Astrolabe);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Anger]);
    assert!(selection_cards(&run).iter().any(|(_, instance)| *instance == bottled), "Astrolabe does not apply bottle filter");
    let candidates = selection_cards(&run);
    for (_, instance) in candidates.iter().take(3) { select_instance(&mut run, *instance); }
    assert_bane(&run, bane);

    let mut empty = map_run(71);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    relic_offer(&mut empty, RelicId::Astrolabe);
    assert!(empty.state.pending.is_none(), "empty eligible group opens no selection");
    assert_eq!(empty.state.deck, before);
    assert_bane(&empty, empty_bane);
}

fn bottle_case(relic: RelicId, cards: &[CardId], want: &[CardId], seed: u8) {
    let mut run = map_run(seed);
    set_deck(&mut run, cards);
    let bane = bane_instance(&run);
    relic_offer(&mut run, relic);
    assert_selection_ids(&run, want);
    let chosen = first_candidate(&run);
    resolve_one(&mut run, chosen);
    assert_bane(&run, bane);
}
fn bottle_empty_case(relic: RelicId, seed: u8) {
    let mut run = map_run(seed);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bane = bane_instance(&run);
    relic_offer(&mut run, relic);
    assert!(run.state.pending.is_none(), "no type-eligible card opens no bottle selection");
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BottledFlame.java:52, 60
// Grade: Run
#[test]
fn m30_bane_matrix_bottled_flame() {
    bottle_case(RelicId::BottledFlame, &[CardId::Strike, CardId::Bash, CardId::Anger, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell], &[CardId::Strike, CardId::Bash, CardId::Anger], 9);
    bottle_empty_case(RelicId::BottledFlame, 72);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BottledLightning.java:52, 60
// Grade: Run
#[test]
fn m30_bane_matrix_bottled_lightning() {
    bottle_case(RelicId::BottledLightning, &[CardId::Defend, CardId::ShrugItOff, CardId::TrueGrit, CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell], &[CardId::Defend, CardId::ShrugItOff, CardId::TrueGrit], 10);
    bottle_empty_case(RelicId::BottledLightning, 73);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BottledTornado.java:53, 61
// Grade: Run
#[test]
fn m30_bane_matrix_bottled_tornado() {
    bottle_case(RelicId::BottledTornado, &[CardId::Inflame, CardId::Metallicize, CardId::DemonForm, CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell], &[CardId::Inflame, CardId::Metallicize, CardId::DemonForm], 11);
    bottle_empty_case(RelicId::BottledTornado, 74);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/city/BackToBasics.java:78–80
// Grade: Run
#[test]
fn m30_bane_matrix_ancient_writing_elegance() {
    let mut run = at_event(EventId::AncientWriting, 2, 12);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let strike = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    choose_kind(&mut run, &RoomChoiceKind::AncientElegance);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend]);
    resolve_one(&mut run, strike);
    assert_bane(&run, bane_instance(&run));

    let mut empty = at_event_with(EventId::AncientWriting, 2, 75, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]));
    let bottled = empty.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut empty, RelicId::BottledFlame, bottled);
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    choose_kind(&mut empty, &RoomChoiceKind::AncientElegance);
    assert!(empty.state.pending.is_none(), "empty purge group skips the branch");
    assert_eq!(empty.state.deck, before);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/city/BackToBasics.java:98–105
// Grade: Run
#[test]
fn m30_bane_matrix_ancient_writing_simplicity() {
    let mut run = at_event(EventId::AncientWriting, 2, 13);
    let bane = bane_instance(&run);
    choose_kind(&mut run, &RoomChoiceKind::AncientSimplicity);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);

    let mut empty = at_event_with(EventId::AncientWriting, 2, 76, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let empty_bane = bane_instance(&empty);
    choose_kind(&mut empty, &RoomChoiceKind::AncientSimplicity);
    assert_bane(&empty, empty_bane);
    assert_eq!(empty.state.deck.iter().find(|card| card.instance == empty_bane).unwrap().upgrades, 0);
}

// Source: verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/events/city/Beggar.java:88; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_beggar_softlock() {
    let mut run = at_event(EventId::Beggar, 2, 14);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let bane = bane_instance(&run);
    let gold = run.state.gold;
    let before = run.state.deck.clone();
    choose_option(&mut run, 0, 0);
    assert_eq!(run.state.gold, gold - 75);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.deck, before);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/events/city/DrugDealer.java:54, 139–144
// Grade: Run
#[test]
fn m30_bane_matrix_drug_dealer() {
    let short = at_event_with(EventId::DrugDealer, 2, 15, |run|
        set_deck(run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let row = room(&short).choices.into_iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option: 1 }).unwrap();
    assert_eq!(row.status, RoomChoiceStatus::Disabled, "transform two disabled with fewer than two purgeables");
    let mut run = at_event_with(EventId::DrugDealer, 2, 16, |run|
        set_deck(run, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 1);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required, 2);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    let selected = selection_cards(&run);
    select_instance(&mut run, selected[0].1);
    select_instance(&mut run, selected[1].1);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/exordium/Cleric.java:85–88
// Grade: Run
#[test]
fn m30_bane_matrix_cleric() {
    let mut run = at_event(EventId::Cleric, 1, 16);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let gold = run.state.gold;
    choose_option(&mut run, 0, 1);
    assert_eq!(run.state.gold, gold, "empty unbottled purge group does not charge");
    assert!(run.state.pending.is_none());
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/GoldenWing.java:124; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_wing_statue_softlock() {
    let mut run = at_event(EventId::WingStatue, 1, 17);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let bane = bane_instance(&run);
    let hp = run.state.hp;
    choose_option(&mut run, 0, 0);
    assert_eq!(run.state.hp, hp - 7);
    assert!(run.state.pending.is_none());
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/LivingWall.java:106–119
// Grade: Run
#[test]
fn m30_bane_matrix_living_wall_forget() {
    let mut run = at_event(EventId::LivingWall, 1, 18);
    set_deck(&mut run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let strike = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert_selection_ids(&run, &[CardId::Strike]);
    select_instance(&mut run, strike);
    assert_bane(&run, bane);

    let mut empty = at_event_with(EventId::LivingWall, 1, 77, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]));
    let bottled = empty.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut empty, RelicId::BottledFlame, bottled);
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    choose_option(&mut empty, 0, 0);
    assert!(empty.state.pending.is_none(), "Forget skips an empty unbottled purge group");
    assert_eq!(empty.state.deck, before);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/LivingWall.java:106–119
// Grade: Run
#[test]
fn m30_bane_matrix_living_wall_change() {
    let mut run = at_event(EventId::LivingWall, 1, 19);
    set_deck(&mut run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let strike = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 1);
    assert_selection_ids(&run, &[CardId::Strike]);
    select_instance(&mut run, strike);
    assert_bane(&run, bane);

    let mut empty = at_event_with(EventId::LivingWall, 1, 78, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]));
    let bottled = empty.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut empty, RelicId::BottledFlame, bottled);
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    choose_option(&mut empty, 0, 1);
    assert!(empty.state.pending.is_none(), "Change skips an empty unbottled purge group");
    assert_eq!(empty.state.deck, before);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/LivingWall.java:54–57, 117–119
// Grade: Run
#[test]
fn m30_bane_matrix_living_wall_grow() {
    let mut run = at_event(EventId::LivingWall, 1, 20);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 2);
    assert!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades == 0);
    assert_bane(&run, bane);

    let empty = at_event_with(EventId::LivingWall, 1, 79, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let empty_view = room(&empty);
    let grow = empty_view.choices.iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option: 2 }).unwrap();
    assert_eq!(grow.status, RoomChoiceStatus::Disabled, "Grow has no upgradeable card");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Bonfire.java:111–114
// Grade: Run
#[test]
fn m30_bane_matrix_bonfire_spirits() {
    let mut run = at_event(EventId::BonfireSpirits, 1, 21);
    set_deck(&mut run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let strike = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert_selection_ids(&run, &[CardId::Strike]);
    select_instance(&mut run, strike);
    assert_bane(&run, bane);

    let mut empty = at_event_with(EventId::BonfireSpirits, 1, 80, |run| {
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
        let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
        add_bottle(run, RelicId::BottledFlame, bottled);
    });
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    choose_option(&mut empty, 0, 0);
    assert!(empty.state.pending.is_none(), "empty purge group skips the picker");
    assert_eq!(empty.state.deck, before);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:174–176, 195, 240–253
// Grade: Run
#[test]
fn m30_bane_matrix_designer_adjust_chosen() {
    let mut run = designer_at_option(0, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert_selection_ids(&run, &[CardId::Strike, CardId::Defend]);
    resolve_first(&mut run);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:174–176, 195, 240–253
// Grade: Run
#[test]
fn m30_bane_matrix_designer_adjust_random() {
    let mut run = designer_at_option(1, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 1);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_designer_clean_remove_softlock() {
    let mut run = designer_at_option(2, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    let gold = run.state.gold;
    let before = run.state.deck.clone();
    choose_option(&mut run, 0, 2);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.gold, gold - 60);
    assert_eq!(run.state.deck, before);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_designer_clean_transform_softlock() {
    let mut run = designer_at_option(3, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    let gold = run.state.gold;
    let before = run.state.deck.clone();
    choose_option(&mut run, 0, 3);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.gold, gold - 60);
    assert_eq!(run.state.deck, before, "short fixed-two grid completes with no mutation");
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_designer_full_service_softlock() {
    let mut run = designer_at_option(4, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    let gold = run.state.gold;
    let before = run.state.deck.clone();
    choose_option(&mut run, 0, 4);
    assert_eq!(run.state.gold, gold - 90);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.deck, before, "no card is removed or upgraded from the short grid");
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217
// Grade: Run
#[test]
fn m30_bane_matrix_designer_punch() {
    let mut run = designer_at_option(5, &[]);
    let bane = bane_instance(&run);
    let hp = run.state.hp;
    choose_option(&mut run, 0, 5);
    assert_eq!(run.state.hp, hp - 3);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_designer_a10_short_clean_transform() {
    let mut run = designer_at_option(3, &[CardId::Strike]);
    let bane = bane_instance(&run);
    let gold = run.state.gold;
    let clean_transform = room(&run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option: 3 }).unwrap();
    assert_eq!(clean_transform.status, RoomChoiceStatus::Offered, "raw unbottled count includes Bane");
    let grid = crate::run::rooms::standard::event_exec::select_candidates(&run.state, crate::run::EventSelectOp::Transform);
    assert_eq!(grid.iter().map(|card| card.id).collect::<Vec<_>>(), vec![CardId::Strike], "two-card menu has a one-card filtered grid");
    choose_option(&mut run, 0, 3);
    assert_eq!(run.state.gold, gold - 60);
    assert_eq!(run.state.deck.len(), 2, "one-card grid cannot satisfy two-card request");
    assert!(run.state.pending.is_none());
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/GremlinWheelGame.java:309–312
// Grade: Run
#[test]
fn m30_bane_matrix_wheel_of_change() {
    let mut found = None;
    for seed in 0..=u8::MAX {
        let mut run = at_event_with(EventId::WheelOfChange, 1, seed, |run| {
            set_deck(run, &[CardId::Strike, CardId::Defend, CardId::Bash, CardId::Anger, CardId::ShrugItOff,
                CardId::Necronomicurse, CardId::CurseOfTheBell]);
            let strike = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
            add_bottle(run, RelicId::BottledFlame, strike);
        });
        choose_option(&mut run, 0, 0);
        if room(&run).choices.iter().any(|row| row.kind == RoomChoiceKind::EventOption { stage: 1, option: 4 }) {
            found = Some((seed, run)); break;
        }
    }
    let (seed, mut run) = found.expect("seed sweep reaches the removal arm");
    let bane = bane_instance(&run);
    choose_option(&mut run, 1, 4);
    assert_selection_ids(&run, &[CardId::Defend, CardId::Bash, CardId::Anger, CardId::ShrugItOff]);
    resolve_first(&mut run);
    assert_bane(&run, bane);
    let mut empty = at_event_with(EventId::WheelOfChange, 1, seed, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]));
    let bottled = empty.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut empty, RelicId::BottledFlame, bottled);
    let before = empty.state.deck.clone();
    choose_option(&mut empty, 0, 0);
    choose_option(&mut empty, 1, 4);
    assert_eq!(empty.state.deck, before, "empty remove arm skips selection");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/NoteForYourself.java:71–73
// Grade: Run
#[test]
fn m30_bane_matrix_note_for_yourself() {
    let mut run = at_event(EventId::NoteForYourself, 1, 29);
    set_deck(&mut run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let strike = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert_selection_ids(&run, &[CardId::Strike, CardId::IronWave]);
    select_instance(&mut run, strike);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/PurificationShrine.java:68; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_purifier_softlock() {
    let mut run = at_event(EventId::Purifier, 1, 30);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let bane = bane_instance(&run);
    let before = run.state.deck.clone();
    choose_kind(&mut run, &RoomChoiceKind::ShrineRemove);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.deck, before);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Transmogrifier.java:72; docs/intentional-deviations.md SOFTLOCK-001
// Grade: Run
// SOFTLOCK-001
#[test]
fn m30_bane_matrix_transmogrifier_softlock() {
    let mut run = at_event(EventId::Transmogrifier, 1, 31);
    set_deck(&mut run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Strike]);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Strike).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    let bane = bane_instance(&run);
    let before = run.state.deck.clone();
    choose_option(&mut run, 0, 0);
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.deck, before);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DollysMirror.java:44–56
// Grade: Run
#[test]
fn m30_bane_matrix_dollys_mirror() {
    let mut run = map_run(32);
    set_deck(&mut run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::DollysMirror);
    assert!(selection_cards(&run).iter().any(|(id, _)| *id == BANE), "raw deck selector includes Bane");
    let copy_bane = selection_cards(&run).iter().find(|(id, _)| *id == BANE).unwrap().1;
    select_instance(&mut run, copy_bane);
    assert_eq!(run.state.deck.iter().filter(|card| card.id == BANE).count(), 2);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-02-m28f-v1/decompiled/Duplicator/com/megacrit/cardcrawl/events/shrines/Duplicator.java:90–100
// Grade: Run
#[test]
fn m30_bane_matrix_duplicator() {
    let mut run = at_event(EventId::Duplicator, 2, 33);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert!(selection_cards(&run).iter().any(|(id, _)| *id == BANE), "raw deck selector includes Bane");
    select_instance(&mut run, bane);
    assert_eq!(run.state.deck.iter().filter(|card| card.id == BANE).count(), 2);
    assert_bane(&run, bane);
}

// Source: verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/events/shrines/WeMeetAgain.java:59–69, 102–107
// Grade: Run
#[test]
fn m30_bane_matrix_we_meet_again() {
    let mut run = at_event_with(EventId::WeMeetAgain, 2, 34, |run|
        set_deck(run, &[CardId::IronWave, CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let bane = bane_instance(&run);
    let card = room(&run).choices.into_iter().find_map(|choice| match choice.kind {
        RoomChoiceKind::EventCard { card: Some(card), .. } => Some(card), _ => None,
    }).expect("We Meet Again selected card");
    assert_ne!(card, BANE);
    let row = room(&run).choices.into_iter().find(|choice|
        matches!(choice.kind, RoomChoiceKind::EventCard { stage: 0, option: 2, card: Some(_) })).unwrap();
    run.step(&GameAction::Choose(row.id)).unwrap();
    assert_bane(&run, bane);

    let empty = at_event_with(EventId::WeMeetAgain, 2, 81, |run|
        set_deck(run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let empty_view = room(&empty);
    let card_row = empty_view.choices.iter().find(|choice| matches!(choice.kind,
        RoomChoiceKind::EventCard { stage: 0, option: 2, .. })).unwrap();
    assert_eq!(card_row.status, RoomChoiceStatus::Disabled, "no nonbasic non-curse card disables the offer");
    assert!(matches!(card_row.kind, RoomChoiceKind::EventCard { card: None, .. }));
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/beyond/Falling.java:62–73, 117–132; verification/2026-11-03-m28f-v1/decompiled/CardHelper/com/megacrit/cardcrawl/helpers/CardHelper.java:92–107
// Grade: Run
#[test]
fn m30_bane_matrix_falling() {
    let mut run = at_event_with(EventId::Falling, 3, 35, |run| {
        set_deck(run, &[CardId::Strike, CardId::Bash, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
        let bash = run.state.deck.iter().find(|card| card.id == CardId::Bash).unwrap().instance;
        add_bottle(run, RelicId::BottledFlame, bash);
    });
    let bane = bane_instance(&run);
    let view = room(&run);
    let attack = view.choices.iter().find(|row| matches!(row.kind, RoomChoiceKind::FallingLose { card_type: CardType::Attack, .. })).expect("unbottled attack option");
    assert_eq!(attack.status, RoomChoiceStatus::Offered);
    run.step(&GameAction::Choose(attack.id)).unwrap();
    assert_bane(&run, bane);

    let empty = at_event_with(EventId::Falling, 3, 82, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell]));
    assert!(room(&empty).choices.iter().all(|row| !matches!(row.kind, RoomChoiceKind::FallingLose { .. })),
        "categories absent from the deck are not offered");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/events/city/Vampires.java:98–110
// Grade: Run
#[test]
fn m30_bane_matrix_vampires() {
    let mut run = at_event(EventId::Vampires, 2, 36);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert!(!run.state.deck.iter().any(|card| card.id == CardId::Strike), "tagged starter Strike is removed");
    assert_bane(&run, bane);

    let mut no_strike = at_event_with(EventId::Vampires, 2, 83, |run|
        set_deck(run, &[CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let no_strike_bane = bane_instance(&no_strike);
    choose_option(&mut no_strike, 0, 0);
    assert!(!no_strike.state.deck.iter().any(|card| card.id == CardId::Strike));
    assert_bane(&no_strike, no_strike_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/PandorasBox.java:62–80
// Grade: Run
#[test]
fn m30_bane_matrix_pandoras_box() {
    let mut run = map_run(37);
    set_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::PandorasBox);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().id, BANE);

    let mut empty = map_run(84);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let empty_bane = bane_instance(&empty);
    relic_offer(&mut empty, RelicId::PandorasBox);
    assert_eq!(empty.state.deck.len(), 3, "no starter tags means no transformation");
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/FountainOfCurseRemoval.java:57–65
// Grade: Run
#[test]
fn m30_bane_matrix_fountain_of_cleansing() {
    let mut run = at_event_with(EventId::FountainOfCleansing, 1, 38, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Regret, CardId::Doubt]));
    let bane = bane_instance(&run);
    let bottled = run.state.deck.iter().find(|card| card.id == CardId::Regret).unwrap().instance;
    add_bottle(&mut run, RelicId::BottledFlame, bottled);
    choose_option(&mut run, 0, 0);
    assert!(run.state.deck.iter().any(|card| card.id == CardId::Regret), "bottled curse remains");
    assert!(!run.state.deck.iter().any(|card| card.id == CardId::Doubt), "unbottled removable curse is removed");
    assert_bane(&run, bane);

    let mut empty = at_event_with(EventId::FountainOfCleansing, 1, 85, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell, CardId::Regret]));
    let bottled = empty.state.deck.iter().find(|card| card.id == CardId::Regret).unwrap().instance;
    add_bottle(&mut empty, RelicId::BottledFlame, bottled);
    let before = empty.state.deck.clone();
    let empty_bane = bane_instance(&empty);
    choose_option(&mut empty, 0, 0);
    assert_eq!(empty.state.deck, before, "protected and bottled curses remain");
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/rooms/CampfireUI.java:115–117; CampfireSmithEffect.java:77
// Grade: Run
#[test]
fn m30_bane_matrix_smith() {
    let mut run = map_run(39);
    set_deck(&mut run, &[CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    enter_world_room(&mut run, MapNodeKind::Rest);
    choose_kind(&mut run, &RoomChoiceKind::Smith);
    assert_selection_ids(&run, &[CardId::Strike]);
    resolve_first(&mut run);
    assert_bane(&run, bane);

    let mut empty = map_run(86);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    enter_world_room(&mut empty, MapNodeKind::Rest);
    let empty_view = room(&empty);
    let smith = empty_view.choices.iter().find(|row| row.kind == RoomChoiceKind::Smith).unwrap();
    assert_eq!(smith.status, RoomChoiceStatus::Disabled, "no upgradeable card disables Smith");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/AccursedBlacksmith.java:51–56, 86
// Grade: Run
#[test]
fn m30_bane_matrix_accursed_blacksmith() {
    let mut run = at_event(EventId::AccursedBlacksmith, 1, 40);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert!(!selection_cards(&run).iter().any(|(id, _)| *id == BANE));
    resolve_first(&mut run);
    assert_bane(&run, bane);

    let empty = at_event_with(EventId::AccursedBlacksmith, 1, 87, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let empty_view = room(&empty);
    let forge = empty_view.choices.iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option: 0 }).unwrap();
    assert_eq!(forge.status, RoomChoiceStatus::Disabled, "no upgradeable card disables Forge");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/UpgradeShrine.java:42–49, 77
// Grade: Run
#[test]
fn m30_bane_matrix_upgrade_shrine() {
    let mut run = at_event(EventId::UpgradeShrine, 1, 41);
    let bane = bane_instance(&run);
    choose_kind(&mut run, &RoomChoiceKind::ShrineUpgrade);
    assert!(!selection_cards(&run).iter().any(|(id, _)| *id == BANE));
    resolve_first(&mut run);
    assert_bane(&run, bane);

    let empty = at_event_with(EventId::UpgradeShrine, 1, 88, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let empty_view = room(&empty);
    let upgrade = empty_view.choices.iter().find(|row| row.kind == RoomChoiceKind::ShrineUpgrade).unwrap();
    assert_eq!(upgrade.status, RoomChoiceStatus::Disabled, "no upgradeable card disables the shrine choice");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/TinyHouse.java:40–62
// Grade: Run
#[test]
fn m30_bane_matrix_tiny_house() {
    let mut run = map_run(42);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::TinyHouse);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);

    let mut empty = map_run(89);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let empty_bane = bane_instance(&empty);
    relic_offer(&mut empty, RelicId::TinyHouse);
    assert_eq!(empty.state.deck.iter().find(|card| card.instance == empty_bane).unwrap().upgrades, 0);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/WarPaint.java:38–60
// Grade: Run
#[test]
fn m30_bane_matrix_war_paint() {
    let mut run = map_run(43);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::WarPaint);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);

    let mut empty = map_run(90);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let empty_bane = bane_instance(&empty);
    relic_offer(&mut empty, RelicId::WarPaint);
    assert_eq!(empty.state.deck.iter().find(|card| card.instance == empty_bane).unwrap().upgrades, 0);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Whetstone.java:38–60
// Grade: Run
#[test]
fn m30_bane_matrix_whetstone() {
    let mut run = map_run(44);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::Whetstone);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);

    let mut empty = map_run(91);
    set_deck(&mut empty, &[CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let empty_bane = bane_instance(&empty);
    relic_offer(&mut empty, RelicId::Whetstone);
    assert_eq!(empty.state.deck.iter().find(|card| card.instance == empty_bane).unwrap().upgrades, 0);
    assert_bane(&empty, empty_bane);
}

// Source: verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/events/beyond/MindBloom.java:80–86
// Grade: Run
#[test]
fn m30_bane_matrix_mind_bloom() {
    let mut run = at_event(EventId::MindBloom, 3, 45);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 1);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);
}

// Source: verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/exordium/ShiningLight.java:61–66, 104–110
// Grade: Run
#[test]
fn m30_bane_matrix_shining_light() {
    let mut run = at_event(EventId::ShiningLight, 1, 46);
    let bane = bane_instance(&run);
    choose_option(&mut run, 0, 0);
    assert_bane(&run, bane);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bane).unwrap().upgrades, 0);

    let empty = at_event_with(EventId::ShiningLight, 1, 92, |run|
        set_deck(run, &[CardId::Necronomicurse, CardId::CurseOfTheBell]));
    let empty_view = room(&empty);
    let upgrade = empty_view.choices.iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option: 0 }).unwrap();
    assert_eq!(upgrade.status, RoomChoiceStatus::Disabled, "no upgradeable card disables Shining Light");
    assert_bane(&empty, bane_instance(&empty));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Necronomicon.java:61–68
// Grade: Run
#[test]
fn m30_bane_matrix_necronomicon_unequip() {
    let mut run = map_run(47);
    set_deck(&mut run, &[CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    relic_offer(&mut run, RelicId::Necronomicon);
    assert!(run.state.deck.iter().any(|card| card.id == CardId::Necronomicurse));
    crate::items::lifecycle::lose_relic(&mut run.state, RelicId::Necronomicon).unwrap();
    assert!(!run.state.deck.iter().any(|card| card.id == CardId::Necronomicurse));
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m28f-v1/decompiled/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java:309–310; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1105–1108
// Grade: Run
#[test]
fn m30_bane_matrix_match_and_keep() {
    let mut run = at_event(EventId::MatchAndKeep, 1, 48);
    let bane = bane_instance(&run);
    assert!(room(&run).choices.iter().all(|choice| !matches!(choice.kind, RoomChoiceKind::EventCard { card: Some(BANE), .. })));
    for _ in 0..5 {
        let first = room(&run).choices.into_iter().find(|choice|
            choice.status == RoomChoiceStatus::Offered && matches!(choice.kind, RoomChoiceKind::EventCard { card: None, .. }))
            .expect("face-down board card");
        let first_position = match first.kind { RoomChoiceKind::EventCard { option, .. } => option, _ => unreachable!() };
        run.step(&GameAction::Choose(first.id)).unwrap();
        let after_first = room(&run);
        let first_card = after_first.choices.iter().find_map(|choice| match choice.kind {
            RoomChoiceKind::EventCard { option, card: Some(card), .. } if option == first_position => Some(card), _ => None,
        }).expect("first card is revealed");
        if let Some(match_row) = after_first.choices.iter().find(|choice| choice.status == RoomChoiceStatus::Offered
            && matches!(choice.kind, RoomChoiceKind::EventCard { option, card: Some(card), .. } if option != first_position && card == first_card)) {
            run.step(&GameAction::Choose(match_row.id)).unwrap();
        } else {
            let second = after_first.choices.into_iter().find(|choice| choice.status == RoomChoiceStatus::Offered
                && matches!(choice.kind, RoomChoiceKind::EventCard { card: None, .. })).expect("second board card");
            run.step(&GameAction::Choose(second.id)).unwrap();
        }
    }
    assert!(run.state.pending.is_none());
    assert_bane(&run, bane);
}

// Source: verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/helpers/CardLibrary.java:1398–1415; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1192–1198
// Grade: Table
#[test]
fn m30_bane_matrix_random_curse_pool() {
    let run = map_run(49);
    let pool = pools::cards(CardPool::Curse).unwrap();
    assert!(!pool.contains(&CardId::AscendersBane));
    assert!(!pool.contains(&CardId::Necronomicurse));
    assert!(!pool.contains(&CardId::CurseOfTheBell));
    assert_bane(&run, bane_instance(&run));
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/actions/watcher/LessonLearnedAction.java:52–59
// Grade: Table
#[test]
fn m30_bane_matrix_lesson_learned() {
    let mut run = map_run(93);
    set_deck(&mut run, &[CardId::LessonLearned, CardId::Strike, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    let candidates = run.state.deck.iter().filter(|card| crate::run::mutations::can_upgrade(&run.state, card))
        .map(|card| card.id).collect::<Vec<_>>();
    assert_eq!(candidates, vec![CardId::LessonLearned, CardId::Strike], "only upgradeable master-deck cards are in the Lesson Learned pool");
    assert_bane(&run, bane);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/actions/unique/RitualDaggerAction.java:46–57
// Grade: Table (static). This row does not drive a dagger kill: the source's master-deck update scans
// for the dagger's own UUID, so the Bane can never be its target. It only pins the static facts that
// rule the Bane out (a Curse, not the Attack the dagger is). The master-deck misc update is driven by
// the M26 supplied play in `m26_supplied.rs` (Ritual Dagger's Fatal +3) (final review m6).
#[test]
fn m30_bane_matrix_ritual_dagger_static() {
    let mut run = map_run(94);
    set_deck(&mut run, &[CardId::RitualDagger, CardId::Necronomicurse, CardId::CurseOfTheBell]);
    let bane = bane_instance(&run);
    let catalog = Catalog::standard();
    assert_eq!(catalog.card(BANE).unwrap().card_type, CardType::Curse);
    assert_eq!(catalog.card(CardId::RitualDagger).unwrap().card_type, CardType::Attack);
    assert!(run.state.deck.iter().any(|card| card.id == CardId::RitualDagger));
    assert_bane(&run, bane);
}

// Source: m30-engine-audit-v1 §3; assigned checkout crates/spire-core/src/run/engine.rs:172–176
// Grade: Run
#[test]
fn m30_bane_matrix_raw_remove() {
    let mut run = a10(50, false);
    let bane = bane_instance(&run);
    let before = run.state.deck.clone();
    run.state.room_loop = None;
    let removed = run.remove_card(bane);
    assert!(removed == Err(RunError::InvalidAction) && run.state.deck == before,
        "raw A10 Bane removal is rejected atomically: result={removed:?}");
}
