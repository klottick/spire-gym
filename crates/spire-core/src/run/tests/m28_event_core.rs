//! M28c event core (chunk plan §4 M28c; design §5.1–§5.7): identities and catalog, pools,
//! eligibility, selection law, frame, executor, nested selections and event combat.
use super::super::*;
use crate::content::{Catalog, EventId, EventPool};

/// Design §4 table and §5.2 eligibility acts: (id, acts, pool). Literals, not computed.
const CATALOG: [(EventId, &[u8], EventPool); 52] = {
    use EventId::*; use EventPool::*;
    [
        (Neow, &[0], Start),
        (BigFish, &[1], Ordinary), (DeadAdventurer, &[1], Ordinary), (GoldenIdol, &[1], Ordinary), (Mushrooms, &[1], Ordinary),
        (LivingWall, &[1], Ordinary), (ScrapOoze, &[1], Ordinary), (ShiningLight, &[1], Ordinary), (Cleric, &[1], Ordinary),
        (Ssssserpent, &[1], Ordinary), (WingStatue, &[1], Ordinary), (WorldOfGoop, &[1], Ordinary),
        (Addict, &[2], Ordinary), (AncientWriting, &[2], Ordinary), (Beggar, &[2], Ordinary), (Colosseum, &[2], Ordinary),
        (CursedTome, &[2], Ordinary), (DrugDealer, &[2], Ordinary), (ForgottenAltar, &[2], Ordinary), (Ghosts, &[2], Ordinary),
        (MaskedBandits, &[2], Ordinary), (Nest, &[2], Ordinary), (Library, &[2], Ordinary), (Mausoleum, &[2], Ordinary), (Vampires, &[2], Ordinary),
        (Falling, &[3], Ordinary), (MindBloom, &[3], Ordinary), (MoaiHead, &[3], Ordinary), (MysteriousSphere, &[3], Ordinary),
        (SensoryStone, &[3], Ordinary), (TombOfLordRedMask, &[3], Ordinary), (WindingHalls, &[3], Ordinary),
        (GoldenShrine, &[1, 2, 3], Shrine), (MatchAndKeep, &[1, 2, 3], Shrine), (Purifier, &[1, 2, 3], Shrine),
        (Transmogrifier, &[1, 2, 3], Shrine), (UpgradeShrine, &[1, 2, 3], Shrine), (WheelOfChange, &[1, 2, 3], Shrine),
        (AccursedBlacksmith, &[1, 2, 3], OneTime), (BonfireSpirits, &[1, 2, 3], OneTime), (FaceTrader, &[1, 2], OneTime),
        (FountainOfCleansing, &[1, 2, 3], OneTime), (Lab, &[1, 2, 3], OneTime), (NoteForYourself, &[1, 2, 3], OneTime),
        (WeMeetAgain, &[1, 2, 3], OneTime), (WomanInBlue, &[1, 2, 3], OneTime), (Joust, &[2], OneTime), (Designer, &[2, 3], OneTime),
        (Duplicator, &[2, 3], OneTime), (KnowingSkull, &[2], OneTime), (Nloth, &[2], OneTime), (SecretPortal, &[3], OneTime),
    ]
};

#[test]
fn m28_event_identities_and_catalog() {
    let catalog = Catalog::standard();
    assert_eq!(catalog.events().len(), 52);
    for (id, acts, pool) in CATALOG {
        let row = catalog.event(id).unwrap_or_else(|| panic!("{id:?} missing"));
        let mask = acts.iter().fold(0u8, |m, a| m | 1 << a);
        assert_eq!((row.acts, row.pool), (mask, pool), "{id:?}");
    }
    // The accepted seven keep their wire names. M28a wired the full Neow blessing, so no row stays
    // definition-only (design §5.1).
    let names = [(EventId::Neow, "neow"), (EventId::UpgradeShrine, "upgrade_shrine"), (EventId::Purifier, "purifier"),
        (EventId::AncientWriting, "ancient_writing"), (EventId::Joust, "joust"), (EventId::Falling, "falling"),
        (EventId::TombOfLordRedMask, "tomb_of_lord_red_mask")];
    for (id, name) in names { assert_eq!(id.as_str(), name); }
    assert!(catalog.events().iter().all(|row| !row.definition_only));
    let mut seen = std::collections::BTreeSet::new();
    assert!(catalog.events().iter().all(|row| seen.insert(row.id.as_str())), "wire names are unique");
}

use super::m28_world::on_map;
use crate::content::{CardId, Character, EncounterId, RelicId};
use rooms::standard::event_pool::{self, EventPools, Selected};

/// Design §4 table: each act's ordinary list, the six shrines and the 14 one-time rows.
const ACT1: [EventId; 11] = { use EventId::*; [BigFish, DeadAdventurer, GoldenIdol, Mushrooms, LivingWall, ScrapOoze, ShiningLight, Cleric, Ssssserpent, WingStatue, WorldOfGoop] };
const ACT2: [EventId; 13] = { use EventId::*; [Addict, AncientWriting, Beggar, Colosseum, CursedTome, DrugDealer, ForgottenAltar, Ghosts, MaskedBandits, Nest, Library, Mausoleum, Vampires] };
const ACT3: [EventId; 7] = { use EventId::*; [Falling, MindBloom, MoaiHead, MysteriousSphere, SensoryStone, TombOfLordRedMask, WindingHalls] };
const SHRINES: [EventId; 6] = { use EventId::*; [GoldenShrine, MatchAndKeep, Purifier, Transmogrifier, UpgradeShrine, WheelOfChange] };
const ONE_TIME: [EventId; 14] = { use EventId::*; [AccursedBlacksmith, BonfireSpirits, FaceTrader, FountainOfCleansing, Lab, NoteForYourself, WeMeetAgain, WomanInBlue, Joust, Designer, Duplicator, KnowingSkull, Nloth, SecretPortal] };

fn sorted(list: &[EventId]) -> Vec<&'static str> { let mut v: Vec<_> = list.iter().map(|e| e.as_str()).collect(); v.sort(); v }

#[test]
fn m28_event_pool_per_act() {
    let mut run = on_map(Character::Silent, 3);
    let pools = run.state.event_pools.clone().expect("standard Run builds its event pools");
    assert_eq!((sorted(&pools.ordinary), sorted(&pools.shrines), sorted(&pools.one_time)), (sorted(&ACT1), sorted(&SHRINES), sorted(&ONE_TIME)));
    assert!(pools.seen.is_empty());
    // Consume a shrine, a one-time row and an ordinary row, then start Acts 2 and 3.
    let p = run.state.event_pools.as_mut().unwrap();
    p.shrines.retain(|e| *e != EventId::GoldenShrine); p.one_time.retain(|e| *e != EventId::Lab); p.ordinary.clear(); p.seen.push(EventId::Lab);
    for (act, expected) in [(2u8, &ACT2[..]), (3, &ACT3[..])] {
        run.state.act = act;
        rooms::standard::on_act_start(&mut run);
        let p = run.state.event_pools.as_ref().unwrap();
        assert_eq!(sorted(&p.ordinary), sorted(expected), "act {act} ordinary");
        assert_eq!(sorted(&p.shrines), sorted(&SHRINES), "act {act}: shrines rebuilt");
        assert_eq!(p.one_time.len(), 13, "act {act}: one-time rows never replenished");
        assert!(!p.one_time.contains(&EventId::Lab));
        assert_eq!(p.seen, vec![EventId::Lab]);
    }
}

/// A standard Act 1 state for eligibility probes, standing on a node of row `y`.
fn probe(act: u8) -> (RunState, NodeId) {
    let run = on_map(Character::Ironclad, 5);
    let mut state = run.state.clone();
    state.act = act;
    let node = state.map.as_ref().unwrap().nodes[0].id;
    (state, node)
}
fn node_at_row(state: &RunState, y: u8) -> NodeId { state.map.as_ref().unwrap().nodes.iter().find(|n| n.y == y).unwrap().id }

#[test]
fn m28_event_eligibility_boundaries() {
    use EventId::*;
    let check = |state: &RunState, node: NodeId, id: EventId, expected: bool, why: &str| assert_eq!(event_pool::eligible(state, id, node), expected, "{id:?}: {why}");
    // Floor > 6 (event-neow research :218-222).
    let (mut s, n) = probe(1);
    for id in [DeadAdventurer, Mushrooms] { s.floor = 6; check(&s, n, id, false, "floor 6"); s.floor = 7; check(&s, n, id, true, "floor 7"); }
    // Gold thresholds: Cleric 35, Woman in Blue 50 (any act); Joust 50 (Act 2); Designer 75 (Acts 2–3); Beggar 75 (Act 2, E10).
    for (id, act, gold) in [(Cleric, 1u8, 35u32), (WomanInBlue, 1, 50), (WomanInBlue, 3, 50), (Joust, 2, 50), (Designer, 2, 75), (Designer, 3, 75), (Beggar, 2, 75)] {
        let (mut s, n) = probe(act);
        s.gold = gold - 1; check(&s, n, id, false, "below gold");
        s.gold = gold; check(&s, n, id, true, "at gold");
    }
    // Act masks.
    for (id, yes, no) in [(Joust, 2u8, 3u8), (Designer, 2, 1), (Duplicator, 3, 1), (FaceTrader, 2, 3), (FaceTrader, 1, 3), (KnowingSkull, 2, 3), (Nloth, 2, 1),
        (SecretPortal, 3, 2), (BigFish, 1, 2), (Addict, 2, 1), (MindBloom, 3, 2), (NoteForYourself, 1, 0), (NoteForYourself, 3, 0)] {
        let (mut s, n) = probe(yes); s.gold = 999; s.relics = vec![RelicId::BurningBlood, RelicId::Anchor];
        check(&s, n, id, true, "eligible act");
        s.act = no; check(&s, n, id, false, "other act");
    }
    // Note For Yourself: A0–A14 (UNLOCK-001).
    let (mut s, n) = probe(2);
    s.ascension = 14; check(&s, n, NoteForYourself, true, "A14"); s.ascension = 15; check(&s, n, NoteForYourself, false, "A15");
    // Knowing Skull HP > 12; N'loth at least 2 relics (W, The City page).
    let (mut s, n) = probe(2);
    s.hp = 12; check(&s, n, KnowingSkull, false, "hp 12"); s.hp = 13; check(&s, n, KnowingSkull, true, "hp 13");
    s.relics = vec![RelicId::BurningBlood]; check(&s, n, Nloth, false, "one relic");
    s.relics.push(RelicId::Anchor); check(&s, n, Nloth, true, "two relics");
    // Moai Head: HP <= 50% of max HP, or Golden Idol owned.
    let (mut s, n) = probe(3);
    s.max_hp = 80; s.hp = 40; check(&s, n, MoaiHead, true, "hp 40/80");
    s.hp = 41; check(&s, n, MoaiHead, false, "hp 41/80");
    s.relics.push(RelicId::GoldenIdol); check(&s, n, MoaiHead, true, "Golden Idol");
    // Colosseum (E10): node row y > 15 / 2, so row 7 no, row 8 yes.
    let (s, _) = probe(2);
    check(&s, node_at_row(&s, 7), Colosseum, false, "row 7"); check(&s, node_at_row(&s, 8), Colosseum, true, "row 8");
    // Fountain of Cleansing: isCursed ignores Ascender's Bane, Curse of the Bell and Necronomicurse (E9).
    let (mut s, n) = probe(1);
    let mut curse = s.deck[0].clone();
    for id in [CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse] { curse.id = id; s.deck.push(curse.clone()); }
    check(&s, n, FountainOfCleansing, false, "unremovable curses only");
    curse.id = CardId::Regret; s.deck.push(curse); check(&s, n, FountainOfCleansing, true, "Regret");
}

fn pools(ordinary: &[EventId], shrines: &[EventId], one_time: &[EventId]) -> EventPools {
    EventPools { ordinary: ordinary.to_vec(), shrines: shrines.to_vec(), one_time: one_time.to_vec(), seen: vec![] }
}

#[test]
fn m28_event_selection_law() {
    use EventId::*;
    let (mut s, n) = probe(1);
    s.floor = 3; s.gold = 10; // Dead Adventurer (floor > 6), Cleric (35 gold) and Woman in Blue (50 gold) ineligible.
    let mut lens = vec![];
    // Ordinary branch: uniform over the eligible list, in list order; the pick leaves its list.
    let mut p = pools(&[BigFish, DeadAdventurer, GoldenIdol], &[GoldenShrine], &[Lab]);
    let got = event_pool::select_with(&mut p, &s, n, false, |len| { lens.push(len); Ok(1) }).unwrap();
    assert_eq!((got, lens.clone()), (Selected::Event(GoldenIdol), vec![2]));
    assert_eq!((p.ordinary.clone(), p.seen.clone()), (vec![BigFish, DeadAdventurer], vec![GoldenIdol]));
    // Shrine branch: remaining shrines, then eligible one-time rows.
    lens.clear();
    let mut p = pools(&[BigFish], &[GoldenShrine, Purifier], &[WomanInBlue, Lab]);
    let got = event_pool::select_with(&mut p, &s, n, true, |len| { lens.push(len); Ok(2) }).unwrap();
    assert_eq!((got, lens.clone(), p.one_time.clone(), p.shrines.len()), (Selected::Event(Lab), vec![3], vec![WomanInBlue], 2));
    // Ordinary eligible-empty (raw non-empty) and raw-empty both delegate to the shrine branch.
    for ordinary in [&[DeadAdventurer, Cleric][..], &[][..]] {
        let mut p = pools(ordinary, &[UpgradeShrine], &[]);
        assert_eq!(event_pool::select_with(&mut p, &s, n, false, |_| Ok(0)).unwrap(), Selected::Event(UpgradeShrine));
    }
    // E6a: a raw-non-empty but eligible-empty shrine branch ends in the monster fallback; nothing consumed.
    let mut p = pools(&[BigFish], &[], &[WomanInBlue]);
    assert_eq!(event_pool::select_with(&mut p, &s, n, true, |_| panic!("no pick")).unwrap(), Selected::Monster);
    assert_eq!(p, pools(&[BigFish], &[], &[WomanInBlue]));
    // Both raw shrine lists empty, not delegated: the ordinary pick.
    let mut p = pools(&[BigFish, Cleric], &[], &[]);
    assert_eq!(event_pool::select_with(&mut p, &s, n, true, |len| { assert_eq!(len, 1); Ok(0) }).unwrap(), Selected::Event(BigFish));
    // ... with an eligible-empty ordinary list: fallback.
    let mut p = pools(&[Cleric], &[], &[]);
    assert_eq!(event_pool::select_with(&mut p, &s, n, true, |_| panic!("no pick")).unwrap(), Selected::Monster);
    // E6c: a delegated ordinary branch meeting empty shrine lists never loops back.
    let mut p = pools(&[Cleric], &[], &[]);
    assert_eq!(event_pool::select_with(&mut p, &s, n, false, |_| panic!("no pick")).unwrap(), Selected::Monster);
    // E6b: everything empty.
    let mut p = pools(&[], &[], &[]);
    assert_eq!(event_pool::select_with(&mut p, &s, n, true, |_| panic!("no pick")).unwrap(), Selected::Monster);
}

#[test]
fn m28_event_selection_draws() {
    // The branch draw is binary32 `< 0.25` on M28EventBranch; the pick is M28EventPick (design §8).
    let run = on_map(Character::Defect, 9);
    let (mut shrine, mut ordinary) = (0, 0);
    for floor in 1..=200u32 {
        let mut s = run.state.clone(); s.floor = floor;
        let node = s.map.as_ref().unwrap().nodes[0].id;
        match event_pool::select(&mut s, node).unwrap() {
            Selected::Event(id) if SHRINES.contains(&id) || ONE_TIME.contains(&id) => shrine += 1,
            Selected::Event(_) => ordinary += 1,
            Selected::Monster => panic!("Act 1 lists are full"),
        }
        assert_eq!(s.event_pools.as_ref().unwrap().seen.len(), 1);
    }
    assert!((30..=70).contains(&shrine), "about a quarter of 200 draws take the shrine branch: {shrine}");
    assert_eq!(shrine + ordinary, 200);
}

// ---- Frame, executor, nested selections and event combat (core review-focus witnesses) ----

use crate::combat::{rng::Partition, CombatAction, Outcome as CombatOutcome};
use rooms::standard::event_exec::{Outcome, PotionChoice};
use rooms::standard::event_frame::{self, DONE};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
/// Base max HP at A0 (research-c/character-data.md:95): Ironclad 80, Silent 70, Defect 75, Watcher 72.
pub(super) const BASE_MAX_HP: [(Character, u32); 4] = [(Character::Ironclad, 80), (Character::Silent, 70), (Character::Defect, 75), (Character::Watcher, 72)];

pub(super) fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
/// A standard Run on `act`'s map, one move from `event` (diagnostics builder, item 16).
pub(super) fn before_event(character: Character, act: u8, event: EventId, seed: u8) -> (Run, NodeId) {
    rooms::standard::diagnostic_event_run(character, act, event, rng(seed)).unwrap()
}
pub(super) fn at_event(character: Character, act: u8, event: EventId, seed: u8) -> Run {
    let (mut run, node) = before_event(character, act, event, seed);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    run
}
pub(super) fn view(run: &Run) -> RoomView { rooms::public(run).unwrap() }
pub(super) fn row(run: &Run, kind: &RoomChoiceKind) -> RoomChoiceView { view(run).choices.into_iter().find(|c| &c.kind == kind).unwrap_or_else(|| panic!("no row {kind:?}")) }
pub(super) fn option(run: &mut Run, option: u8) -> Result<RunTransition, RunError> {
    let id = row(run, &RoomChoiceKind::EventOption { stage: 0, option }).id;
    run.step(&GameAction::Choose(id))
}
pub(super) fn leave(run: &mut Run) {
    assert_eq!(view(run).choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Leave], "result stage");
    let id = row(run, &RoomChoiceKind::Leave).id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert_eq!(view(run).kind, RoomViewKind::Map);
}
pub(super) fn count(run: &Run, id: CardId) -> usize { run.state.deck.iter().filter(|c| c.id == id).count() }
pub(super) fn base(character: Character) -> u32 { BASE_MAX_HP.iter().find(|(c, _)| *c == character).unwrap().1 }

#[test]
fn m28_big_fish_branches() {
    // Banana: Heal(M / 3) captured at construction, per character (26, 23, 25, 24).
    for (character, heal) in [(Character::Ironclad, 26), (Character::Silent, 23), (Character::Defect, 25), (Character::Watcher, 24)] {
        let (mut run, node) = before_event(character, 1, EventId::BigFish, 1);
        run.state.max_hp = base(character); run.state.hp = 1;
        run.step(&GameAction::Move(node)).unwrap();
        run.state.max_hp += 30; // captured: a later max-HP change does not recompute the heal
        option(&mut run, 0).unwrap();
        assert_eq!(run.state.hp, 1 + heal, "{character:?}");
        leave(&mut run);
    }
    // Donut: +5 max HP (and the shared owner's 5 HP heal).
    let mut run = at_event(Character::Ironclad, 1, EventId::BigFish, 2);
    run.state.hp = 10; let max = run.state.max_hp;
    option(&mut run, 1).unwrap();
    assert_eq!((run.state.max_hp, run.state.hp), (max + 5, 15));
    leave(&mut run);
    // Box: one random-tier event relic and Regret.
    let mut run = at_event(Character::Silent, 1, EventId::BigFish, 3);
    let relics = run.state.relics.clone();
    option(&mut run, 2).unwrap();
    assert_eq!(run.state.relics.len(), relics.len() + 1);
    assert!(!relics.contains(run.state.relics.last().unwrap()));
    assert_eq!(count(&run, CardId::Regret), 1);
}

#[test]
fn m28_golden_shrine_branches() {
    let mut run = at_event(Character::Defect, 2, EventId::GoldenShrine, 4);
    let gold = run.state.gold;
    option(&mut run, 0).unwrap();
    assert_eq!(run.state.gold, gold + 100);
    leave(&mut run);
    let mut run = at_event(Character::Defect, 3, EventId::GoldenShrine, 5);
    let gold = run.state.gold;
    option(&mut run, 1).unwrap();
    assert_eq!((run.state.gold, count(&run, CardId::Regret)), (gold + 275, 1));
    leave(&mut run);
    let mut run = at_event(Character::Defect, 1, EventId::GoldenShrine, 6);
    let before = run.state.clone();
    option(&mut run, 2).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map, "Leave closes the room at once");
    assert_eq!((run.state.gold, &run.state.deck), (before.gold, &before.deck));
}

#[test]
fn m28_addict_branches() {
    // Offer Gold: price 85 on the row; a random event relic, then the payment.
    let (mut run, node) = before_event(Character::Watcher, 2, EventId::Addict, 7);
    run.state.gold = 85;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).price, Some(85));
    let relics = run.state.relics.len();
    option(&mut run, 0).unwrap();
    assert_eq!((run.state.gold, run.state.relics.len()), (0, relics + 1));
    leave(&mut run);
    // Rob: a random event relic and Shame.
    let mut run = at_event(Character::Watcher, 2, EventId::Addict, 8);
    let relics = run.state.relics.len();
    option(&mut run, 1).unwrap();
    assert_eq!((run.state.relics.len(), count(&run, CardId::Shame)), (relics + 1, 1));
    leave(&mut run);
}

#[test]
fn m28_mind_bloom_branches() {
    // I am Awake: every upgradeable card upgrades once; Mark of the Bloom.
    let mut run = at_event(Character::Ironclad, 3, EventId::MindBloom, 9);
    option(&mut run, 1).unwrap();
    assert!(run.state.deck.iter().all(|c| c.upgrades == 1), "{:?}", run.state.deck);
    assert!(run.state.relics.contains(&RelicId::MarkOfBloom));
    leave(&mut run);
    // Floor 40 (40 % 50 <= 40): I am Rich, 999 gold and two Normality.
    let (mut run, node) = before_event(Character::Silent, 3, EventId::MindBloom, 10);
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == node).unwrap().floor = 40;
    run.state.floor = 39; // entry adds one (PM ruling m28f Q2)
    run.step(&GameAction::Move(node)).unwrap();
    let gold = run.state.gold;
    option(&mut run, 2).unwrap();
    assert_eq!((run.state.gold, count(&run, CardId::Normality), count(&run, CardId::Doubt)), (gold + 999, 2, 0));
    // Floor 41: I am Healthy, heal max HP (captured) and Doubt, per character.
    for character in CHARACTERS {
        let (mut run, node) = before_event(character, 3, EventId::MindBloom, 11);
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == node).unwrap().floor = 41;
        run.state.floor = 40; // entry adds one (PM ruling m28f Q2)
        run.state.max_hp = base(character); run.state.hp = 1;
        run.step(&GameAction::Move(node)).unwrap();
        let gold = run.state.gold;
        option(&mut run, 2).unwrap();
        assert_eq!((run.state.hp, run.state.gold, count(&run, CardId::Doubt), count(&run, CardId::Normality)), (base(character), gold, 1, 0), "{character:?}");
    }
}

/// Review focus 5 (core): Mind Bloom's I am War fights its drawn Act 1 boss as an event combat and
/// returns to the EventRoom reward screen (50 gold, a rare relic, the card offer); a death in it
/// settles the Run once.
#[test]
fn m28_event_combat_returns_to_frame() {
    for (seed, character) in [(12, Character::Ironclad), (13, Character::Watcher)] {
        let mut run = at_event(character, 3, EventId::MindBloom, seed);
        let gold = run.state.gold;
        option(&mut run, 0).unwrap();
        let active = run.state.active_combat.as_ref().unwrap();
        assert!(rooms::standard::events::MIND_BLOOM_BOSSES.contains(&active.encounter));
        assert!(matches!(active.origin, CombatOrigin::Event { event: EventId::MindBloom, stage: 0, .. }));
        super::m28_world::win(&mut run);
        let rows = view(&run);
        assert_eq!(rows.kind, RoomViewKind::Rewards);
        let kinds: Vec<_> = rows.choices.iter().map(|c| c.kind.clone()).collect();
        assert_eq!(kinds[0], RoomChoiceKind::Gold(50), "no combat gold; the event's 50");
        let RoomChoiceKind::Relic(relic) = kinds[1] else { panic!("rare relic row: {kinds:?}") };
        assert_eq!(Catalog::standard().relic_tier(relic), Some(crate::content::RelicTier::Rare));
        assert_eq!(kinds.iter().filter(|k| **k == RoomChoiceKind::CardReward).count(), 1, "one EventRoom card offer");
        assert!(!kinds.iter().any(|k| matches!(k, RoomChoiceKind::Emerald)));
        let id = row(&run, &RoomChoiceKind::Gold(50)).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(run.state.gold, gold + 50);
        let id = row(&run, &RoomChoiceKind::Proceed).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
    // A death in the event combat: one terminal, committed once.
    let mut run = at_event(Character::Defect, 3, EventId::MindBloom, 14);
    run.state.hp = 1;
    option(&mut run, 0).unwrap();
    let mut transition = None;
    for _ in 0..20 {
        let step = run.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
        if run.state.combat.is_none() { transition = Some(step); break }
        if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome == Some(CombatOutcome::Death) { transition = Some(run.settle_combat().unwrap()); break }
    }
    let transition = transition.expect("the boss kills a 1 HP player");
    assert_eq!((transition.terminal, run.state.terminal), (Some(RunTerminal::Death), Some(RunTerminal::Death)));
    assert_eq!(run.state.history.events.iter().filter(|e| matches!(e, CommittedEvent::Ended(RunTerminal::Death))).count(), 1);
    assert!(run.step(&GameAction::Combat(CombatAction::EndTurn)).is_err());
}

#[test]
fn m28_event_combat_origin_entry() {
    // No event frame: the Event origin is refused by the standard gate.
    let (mut run, node) = before_event(Character::Ironclad, 3, EventId::MindBloom, 15);
    let origin = CombatOrigin::Event { node, event: EventId::MindBloom, stage: 0 };
    assert_eq!(run.enter_owned(EncounterId::Guardian, origin), Err(RunError::Unsupported("encounter outside authored M3 profile")));
    // A frame without a started combat, or another encounter, is refused too.
    run.step(&GameAction::Move(node)).unwrap();
    let boss = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.encounter.unwrap();
    let before = run.clone();
    assert!(run.enter_owned(boss, origin).is_err());
    assert!(run == before);
    option(&mut run, 0).unwrap();
    assert_eq!(run.state.active_combat.as_ref().unwrap().origin, origin);
    let other = rooms::standard::events::MIND_BLOOM_BOSSES.into_iter().find(|e| *e != boss).unwrap();
    let mut probe = run.clone(); probe.state.combat = None; probe.state.active_combat = None;
    assert!(probe.enter_owned(other, origin).is_err(), "only the frame's encounter");
}

/// Review focus 1 (core): rejected choices leave the whole Run unchanged.
#[test]
fn m28_rejected_room_choices_are_atomic() {
    // The Addict's Offer Gold below 85 gold.
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::Addict, 16);
    run.state.gold = 84;
    run.step(&GameAction::Move(node)).unwrap();
    let pay = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 });
    assert_eq!(pay.status, RoomChoiceStatus::Disabled);
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(pay.id)), Err(RunError::InvalidAction));
    assert!(run == before);
    // Joust's bet below 50 gold.
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::Joust, 17);
    run.state.gold = 60;
    run.step(&GameAction::Move(node)).unwrap();
    run.state.gold = 49;
    let bet = row(&run, &RoomChoiceKind::JoustOwner);
    let before = run.clone();
    assert!(run.step(&GameAction::Choose(bet.id)).is_err());
    assert!(run == before);
    // Upgrade Shrine with no upgradeable card.
    let mut run = at_event(Character::Ironclad, 1, EventId::UpgradeShrine, 18);
    for card in &mut run.state.deck { card.upgrades = 1; }
    let upgrade = row(&run, &RoomChoiceKind::ShrineUpgrade);
    assert_eq!(upgrade.status, RoomChoiceStatus::Disabled);
    let before = run.clone();
    assert!(run.step(&GameAction::Choose(upgrade.id)).is_err());
    assert!(run == before);
    // Executor lists: ObtainPotion meeting a full belt; a second outcome failing its precondition.
    let mut run = at_event(Character::Silent, 1, EventId::GoldenShrine, 19);
    for slot in &mut run.state.potions { *slot = Some(crate::content::PotionId::FirePotion); }
    let before = run.clone();
    assert_eq!(event_frame::choose_constructed(&mut run, vec![Outcome::GainGold(5), Outcome::ObtainPotion(PotionChoice::Random), Outcome::NextStage(DONE)]), Err(RunError::InvalidAction));
    assert!(run == before);
    let gold = run.state.gold;
    assert_eq!(event_frame::choose_constructed(&mut run, vec![Outcome::GainGold(10), Outcome::LoseGold(gold + 11), Outcome::NextStage(DONE)]), Err(RunError::InvalidAction));
    assert!(run == before, "nothing from the first outcome applies");
    assert_eq!(event_frame::choose_constructed(&mut run, vec![Outcome::Leave, Outcome::GainGold(1)]), Err(RunError::InvalidAction), "frame control is last");
    assert!(run == before);
}

/// Review focus 2 (core): a selected event leaves its list even when skipped; one-time events never
/// return; shrines are rebuilt per act.
#[test]
fn m28_event_history_consumes_on_selection() {
    let (mut run, node) = before_event(Character::Watcher, 1, EventId::GoldenShrine, 20);
    run.state.event_pools.as_mut().unwrap().one_time = vec![EventId::Joust]; // Act 2 only: ineligible here
    run.step(&GameAction::Move(node)).unwrap();
    option(&mut run, 2).unwrap(); // Leave without taking anything
    let pools = run.state.event_pools.clone().unwrap();
    assert_eq!((pools.shrines, pools.one_time.clone(), pools.seen.clone()), (vec![], vec![EventId::Joust], vec![EventId::GoldenShrine]));
    // Act 2: shrines rebuilt with Golden Shrine back; Joust is selected and consumed.
    run.state.act = 2; rooms::standard::on_act_start(&mut run);
    let pools = run.state.event_pools.as_mut().unwrap();
    assert!(pools.shrines.contains(&EventId::GoldenShrine) && pools.shrines.len() == 6);
    pools.ordinary.clear(); pools.shrines.clear();
    let mut state = run.state.clone(); state.gold = 50;
    assert_eq!(event_pool::select(&mut state, node).unwrap(), Selected::Event(EventId::Joust));
    let pools = state.event_pools.as_ref().unwrap();
    assert_eq!((pools.one_time.len(), pools.seen.clone()), (0, vec![EventId::GoldenShrine, EventId::Joust]));
    let mut later = run.clone(); later.state = state; later.state.act = 3; rooms::standard::on_act_start(&mut later);
    assert!(!later.state.event_pools.as_ref().unwrap().one_time.contains(&EventId::Joust), "one-time events never return");
}

/// Review focus 3 (core): nested selections return to their exact parent.
#[test]
fn m28_nested_selection_returns_to_parent() {
    // Purifier and Upgrade Shrine through their accepted handlers, reached from StandardA0.
    for (event, kind) in [(EventId::Purifier, RoomChoiceKind::ShrineRemove), (EventId::UpgradeShrine, RoomChoiceKind::ShrineUpgrade)] {
        let mut run = at_event(Character::Silent, 1, event, 21);
        let id = row(&run, &kind).id;
        run.step(&GameAction::Choose(id)).unwrap();
        let selection = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Select(s) => Some(s), _ => None }).unwrap();
        let deck = run.state.deck.clone();
        run.step(&GameAction::Select(selection)).unwrap();
        assert_ne!(run.state.deck, deck);
        assert_eq!(view(&run).kind, RoomViewKind::Map, "{event:?} returns to the map");
    }
    // An executor Select (remove one, then transform two) returns to the frame's next stage.
    let mut run = at_event(Character::Defect, 1, EventId::GoldenShrine, 22);
    let size = run.state.deck.len();
    event_frame::choose_constructed(&mut run, vec![
        Outcome::Select { op: EventSelectOp::Remove, count: 1, skippable: false },
        Outcome::Select { op: EventSelectOp::Transform, count: 2, skippable: false },
        Outcome::GainGold(3), Outcome::NextStage(DONE)]).unwrap();
    let select = |run: &mut Run| { let s = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Select(s) => Some(s), _ => None }).unwrap(); run.step(&GameAction::Select(s)).unwrap(); };
    assert!(!run.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Cancel), "committed selection: no Cancel");
    let gold = run.state.gold;
    select(&mut run);
    assert_eq!(run.state.deck.len(), size - 1);
    let deck = run.state.deck.clone();
    select(&mut run); assert!(run.state.pending.is_some(), "second card of the transform-two");
    select(&mut run);
    assert_eq!(run.state.deck.len(), size - 1);
    assert_eq!(run.state.deck.iter().filter(|c| !deck.contains(c)).count(), 2, "two cards transformed");
    assert_eq!(run.state.gold, gold + 3, "the rest of the list ran after the selections");
    leave(&mut run);
    // A skippable selection exposes Cancel; cancelling continues the list.
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 23);
    let gold = run.state.gold;
    event_frame::choose_constructed(&mut run, vec![Outcome::Select { op: EventSelectOp::Upgrade, count: 1, skippable: true }, Outcome::GainGold(7), Outcome::NextStage(DONE)]).unwrap();
    run.step(&GameAction::Cancel).unwrap();
    assert_eq!((run.state.gold, run.state.pending.is_none()), (gold + 7, true));
    leave(&mut run);
}

/// Design §5.5: the six accepted events are selected from the standard pools and run on their
/// accepted handlers and screens.
#[test]
fn m28_accepted_events_routed_from_standard() {
    let kinds = |run: &Run| view(run).choices.into_iter().map(|c| c.kind).collect::<Vec<_>>();
    let run = at_event(Character::Ironclad, 1, EventId::UpgradeShrine, 24);
    assert_eq!(kinds(&run), vec![RoomChoiceKind::ShrineUpgrade, RoomChoiceKind::Leave]);
    let run = at_event(Character::Silent, 2, EventId::Purifier, 25);
    assert_eq!(kinds(&run), vec![RoomChoiceKind::ShrineRemove, RoomChoiceKind::Leave]);
    let run = at_event(Character::Defect, 2, EventId::AncientWriting, 26);
    assert_eq!(kinds(&run), vec![RoomChoiceKind::AncientElegance, RoomChoiceKind::AncientSimplicity]);
    let (mut run, node) = before_event(Character::Watcher, 2, EventId::Joust, 27);
    run.state.gold = 50; run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(kinds(&run), vec![RoomChoiceKind::JoustMurderer, RoomChoiceKind::JoustOwner]);
    let run = at_event(Character::Ironclad, 3, EventId::Falling, 28);
    assert_eq!(kinds(&run).len(), 3);
    assert!(run.state.room_loop.as_ref().unwrap().act3_event.is_some());
    let mut run = at_event(Character::Ironclad, 3, EventId::TombOfLordRedMask, 29);
    let gold = run.state.gold;
    assert_eq!(kinds(&run), vec![RoomChoiceKind::TombOfferGold, RoomChoiceKind::Leave]);
    let id = row(&run, &RoomChoiceKind::TombOfferGold).id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert!(gold > 0 && run.state.gold == 0 && run.state.relics.contains(&RelicId::RedMask));
}

/// E6 through map entry: an event room whose lists are empty becomes a monster room of the act.
#[test]
fn m28_event_room_monster_fallback() {
    let (mut run, node) = before_event(Character::Silent, 1, EventId::GoldenShrine, 30);
    run.state.event_pools = Some(EventPools { ordinary: vec![], shrines: vec![], one_time: vec![], seen: vec![] });
    let regular = run.state.map.as_ref().unwrap().regular_encounter_history.len();
    run.step(&GameAction::Move(node)).unwrap();
    let map = run.state.map.as_ref().unwrap();
    let entered = map.nodes.iter().find(|n| n.id == node).unwrap();
    assert_eq!((entered.hidden_kind, map.regular_encounter_history.len()), (Some(MapNodeKind::Monster), regular + 1));
    assert_eq!(run.state.active_combat.as_ref().unwrap().origin, CombatOrigin::MapRoom { node, kind: MapNodeKind::Monster });
    assert!(run.state.event_pools.as_ref().unwrap().seen.is_empty(), "no event consumed");
    super::m28_world::win(&mut run);
    assert!(view(&run).choices.iter().any(|c| matches!(c.kind, RoomChoiceKind::Gold(g) if (10..=20).contains(&g))), "ordinary monster rewards");
}

/// Item 15, closed by M28f: every standard event has a body (Neow and the accepted six keep their
/// own handlers), so no move returns a body interim; the last two (The Designer, N'loth) now construct.
#[test]
fn m28_unlanded_event_body_interim() {
    let accepted = [EventId::Neow, EventId::UpgradeShrine, EventId::Purifier, EventId::AncientWriting, EventId::Joust, EventId::Falling, EventId::TombOfLordRedMask];
    for row in crate::content::Catalog::standard().events() {
        if accepted.contains(&row.id) { continue }
        assert!(rooms::standard::events::body(row.id).is_some(), "{:?} has a body", row.id);
    }
    for (event, act) in [(EventId::Designer, 2), (EventId::Nloth, 2)] {
        let (mut run, node) = before_event(Character::Ironclad, act, event, 31);
        run.state.gold = 100; run.state.relics.push(RelicId::Anchor);
        assert_eq!(run.step(&GameAction::Move(node)).map(|_| ()), Ok(()), "{event:?}");
        assert!(matches!(view(&run).kind, RoomViewKind::Event(e) if e == event), "{event:?}");
    }
}

// ---- Global fixes G10–G12 (accepted events; reconciliation) ----

/// G10: Purifier and Ancient Writing's Elegance offer only purgeable cards (no unremovable curse,
/// no bottled card), like every shop purge.
#[test]
fn m28_g10_room_removals_offer_purgeable_cards() {
    for (event, act, kind) in [(EventId::Purifier, 1, RoomChoiceKind::ShrineRemove), (EventId::AncientWriting, 2, RoomChoiceKind::AncientElegance)] {
        let mut run = at_event(Character::Ironclad, act, event, 32);
        let mut bane = run.state.deck[0].clone(); bane.id = CardId::AscendersBane; bane.instance = 9_001; run.state.deck.push(bane);
        let bottled = run.state.deck[1].instance;
        run.state.relics.push(RelicId::BottledFlame); run.state.relic_cards.insert(RelicId::BottledFlame, bottled);
        let id = row(&run, &kind).id;
        run.step(&GameAction::Choose(id)).unwrap();
        let offered: Vec<u64> = run.state.pending.as_ref().unwrap().selection.candidates.iter().map(|c| c.card.instance).collect();
        assert!(!offered.contains(&9_001) && !offered.contains(&bottled), "{event:?}: {offered:?}");
        assert_eq!(offered.len(), run.state.deck.len() - 2);
    }
}

/// G11: Falling never samples a card bottled by a relic.
#[test]
fn m28_g11_falling_skips_bottled_cards() {
    let (mut run, node) = before_event(Character::Ironclad, 3, EventId::Falling, 33);
    // Bash is the only Attack besides the Strikes; bottle Bash and remove the Strikes.
    run.state.deck.retain(|c| c.id != CardId::Strike);
    let bash = run.state.deck.iter().find(|c| c.id == CardId::Bash).unwrap().instance;
    run.state.relics.push(RelicId::BottledFlame); run.state.relic_cards.insert(RelicId::BottledFlame, bash);
    run.step(&GameAction::Move(node)).unwrap();
    let attack = view(&run).choices.into_iter().find(|c| matches!(c.kind, RoomChoiceKind::FallingLose { card_type: crate::content::CardType::Attack, .. })).unwrap();
    assert_eq!((attack.kind, attack.status), (RoomChoiceKind::FallingLose { card_type: crate::content::CardType::Attack, instance: None }, RoomChoiceStatus::Disabled));
}

/// G12: Ancient Writing's Simplicity upgrades every card tagged starter Strike or Defend (G1 tags),
/// for every character (Ironclad 5+4, Silent 5+5, Defect 4+4, Watcher 4+4).
#[test]
fn m28_g12_simplicity_reads_starter_tags() {
    for (character, strikes, defends) in [(Character::Ironclad, (CardId::Strike, 5), (CardId::Defend, 4)), (Character::Silent, (CardId::StrikeGreen, 5), (CardId::DefendGreen, 5)),
        (Character::Defect, (CardId::StrikeBlue, 4), (CardId::DefendBlue, 4)), (Character::Watcher, (CardId::StrikePurple, 4), (CardId::DefendPurple, 4))] {
        let mut run = at_event(character, 2, EventId::AncientWriting, 34);
        let id = row(&run, &RoomChoiceKind::AncientSimplicity).id;
        run.step(&GameAction::Choose(id)).unwrap();
        let upgraded = |id: CardId| run.state.deck.iter().filter(|c| c.id == id && c.upgrades == 1).count();
        assert_eq!((upgraded(strikes.0), upgraded(defends.0)), (strikes.1, defends.1), "{character:?}");
        assert_eq!(run.state.deck.iter().filter(|c| c.upgrades == 1).count(), strikes.1 + defends.1, "{character:?}: nothing else");
    }
}

/// Review focus 4 (M28c owners): no frozen scenario builds event pools or a standard event frame,
/// or draws an M28 event scope, including a captured profile's own event room.
#[test]
fn m28_frozen_profiles_never_reach_standard_owners_c() {
    use RoomScenario::*;
    let frozen = [
        (M4Act1Prefix, Character::Ironclad), (M5Act1Captured, Character::Ironclad), (M6Act2Prefix, Character::Ironclad),
        (M7Act2Captured, Character::Ironclad), (M8Act3Prefix, Character::Ironclad), (M9IroncladAct3, Character::Ironclad),
        (M10IroncladHeart, Character::Ironclad), (M12SilentAct1, Character::Silent), (M13SilentAct2, Character::Silent),
        (M14SilentAct3, Character::Silent), (M15SilentHeart, Character::Silent), (M17DefectAct1, Character::Defect),
        (M18DefectAct2, Character::Defect), (M19DefectAct3, Character::Defect), (M20DefectHeart, Character::Defect),
        (M22WatcherAct1, Character::Watcher), (M23WatcherAct2, Character::Watcher), (M24WatcherAct3, Character::Watcher),
        (M25WatcherHeart, Character::Watcher),
    ];
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    for (scenario, character) in frozen {
        let run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile.clone(), rng(3), scenario).unwrap();
        assert_eq!(run.state.event_pools, None, "{scenario:?}");
        assert!(run.state.room_loop.as_ref().unwrap().standard_event.is_none(), "{scenario:?}");
    }
    // The captured M5 profile's first question room is its accepted Upgrade Shrine.
    let mut captured = Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 0 }, profile, rng(3), M5Act1Captured).unwrap();
    let lament = view(&captured).choices.iter().find(|row| row.kind == RoomChoiceKind::NeowLament).unwrap().id;
    captured.step(&GameAction::Choose(lament)).unwrap();
    let target = captured.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).unwrap();
    captured.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = MapNodeKind::Question;
    captured.step(&GameAction::Move(target)).unwrap();
    assert_eq!(view(&captured).kind, RoomViewKind::Event(EventId::UpgradeShrine));
    assert!(captured.state.event_pools.is_none() && captured.state.room_loop.as_ref().unwrap().standard_event.is_none());
    assert!(!format!("{:?}", captured.state.rng).contains("M28Event"), "no M28 event scope drawn");
}

/// M28f owners: Secret Portal's boss entry (`EnterBoss`) is standard-only; every frozen scenario's
/// Run rejects it unchanged, its map and floor included.
#[test]
fn m28_frozen_profiles_never_reach_standard_owners_f() {
    use RoomScenario::*;
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    for (scenario, character) in [(M9IroncladAct3, Character::Ironclad), (M14SilentAct3, Character::Silent), (M19DefectAct3, Character::Defect), (M24WatcherAct3, Character::Watcher)] {
        let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile.clone(), rng(3), scenario).unwrap();
        let before = run.clone();
        assert_eq!(rooms::standard::map::enter_portal_boss(&mut run).err(), Some(RunError::InvalidAction), "{scenario:?}");
        assert!(run == before, "{scenario:?}");
    }
}

/// Design §5.1 and §9: the reference record `data/events/events.json` names the same 52 events,
/// act sets and pools as the catalog (the record is a reference, never read at runtime).
#[test]
fn m28_event_reference_record_matches_catalog() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/events/events.json");
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let records = record["records"].as_array().unwrap();
    assert_eq!(records.len(), 52);
    let mut ids = std::collections::BTreeSet::new();
    for entry in records {
        let id = entry["id"].as_str().unwrap();
        assert!(ids.insert(id), "{id} listed once");
        let row = Catalog::standard().events().iter().find(|row| row.id.as_str() == id).unwrap_or_else(|| panic!("{id} not in the catalog"));
        let acts = entry["acts"].as_array().unwrap().iter().fold(0u8, |m, a| m | 1 << a.as_u64().unwrap());
        let pool = match entry["pool"].as_str().unwrap() { "start" => EventPool::Start, "ordinary" => EventPool::Ordinary, "shrine" => EventPool::Shrine, "one_time" => EventPool::OneTime, other => panic!("{other}") };
        assert_eq!((acts, pool), (row.acts, row.pool), "{id}");
    }
}

/// G13 (PM source check m28c Q3): a boss fight is one with a BOSS-type monster, whatever the room.
/// Mind Bloom's I am War (an Act 1 boss in an EventRoom): Pantograph heals 25 and Slaver's Collar
/// adds its energy; Smoke Bomb cannot be used.
#[test]
fn m28_g13_boss_fight_by_enemy_type() {
    let mut run = at_event(Character::Ironclad, 3, EventId::MindBloom, 35);
    run.state.relics.extend([RelicId::Pantograph, RelicId::SlaversCollar]);
    run.state.hp = 30;
    run.state.potions[0] = Some(crate::content::PotionId::SmokeBomb);
    option(&mut run, 0).unwrap();
    let view = run.state.combat.as_ref().unwrap().public_full().unwrap();
    assert_eq!(view.player.hp, 55, "Pantograph heals 25 at a BOSS-type fight");
    assert_eq!(view.player.energy, 4, "Slaver's Collar: +1 energy");
    assert!(!run.decision().unwrap().candidates.iter().any(|c| matches!(c.action, GameAction::Combat(CombatAction::UsePotion { slot: 0, .. }))), "no Smoke Bomb escape from a boss");
}

/// F2 (PM source check m28c Q4): out-of-combat HP loss runs the player-damage law: Tungsten Rod,
/// then Fairy in a Bottle, then an unused Lizard Tail; Mark of the Bloom blocks both; else death.
#[test]
fn m28_out_of_combat_hp_loss_law() {
    let setup = |seed: u8| { let mut run = at_event(Character::Silent, 1, EventId::GoldenShrine, seed); run.state.max_hp = 70; run.state.hp = 10; run };
    let lose = |run: &mut Run, n: u32| event_frame::choose_constructed(run, vec![Outcome::LoseHp(n), Outcome::NextStage(DONE)]);
    // Tungsten Rod takes 1.
    let mut run = setup(36); run.state.relics.push(RelicId::TungstenRod);
    lose(&mut run, 4).unwrap(); assert_eq!(run.state.hp, 7);
    // Fairy in a Bottle first: 30% of 70 = 21, the potion is spent; Lizard Tail stays unused.
    let mut run = setup(37); run.state.relics.push(RelicId::LizardTail);
    assert!(crate::items::obtain_run_potion(&mut run.state, crate::content::PotionId::FairyPotion).unwrap());
    lose(&mut run, 10).unwrap();
    assert_eq!((run.state.hp, run.state.potions.iter().flatten().count(), run.state.relic_counters.get(&RelicId::LizardTail).copied().unwrap_or(0)), (21, 0, 0));
    // Lizard Tail: half of 70 = 35, then spent.
    let mut run = setup(38); run.state.relics.push(RelicId::LizardTail);
    lose(&mut run, 12).unwrap();
    assert_eq!((run.state.hp, run.state.relic_counters.get(&RelicId::LizardTail).copied()), (35, Some(1)));
    // Mark of the Bloom blocks both: death settles the Run once.
    let mut run = setup(39); run.state.relics.extend([RelicId::LizardTail, RelicId::MarkOfBloom]);
    assert!(crate::items::obtain_run_potion(&mut run.state, crate::content::PotionId::FairyPotion).unwrap());
    let transition = lose(&mut run, 10).unwrap();
    assert_eq!((transition.terminal, run.state.terminal, run.state.hp), (Some(RunTerminal::Death), Some(RunTerminal::Death), 0));
    // No rescue: death.
    let mut run = setup(40);
    assert_eq!(lose(&mut run, 10).unwrap().terminal, Some(RunTerminal::Death));    // Torii (PM correction): owned non-HP_LOSS damage of 2–5 becomes 1, before Tungsten Rod; event
    // damage (no owner) and HP_LOSS skip it.
    use crate::run::mutations::{lose_hp, OutOfCombatLoss};
    let mut run = setup(41); run.state.relics.push(RelicId::Torii);
    for (amount, kind, hp) in [(5, OutOfCombatLoss::Damage { owned: true }, 9), (6, OutOfCombatLoss::Damage { owned: true }, 4),
        (1, OutOfCombatLoss::Damage { owned: true }, 9), (5, OutOfCombatLoss::Damage { owned: false }, 5), (5, OutOfCombatLoss::HpLoss, 5)] {
        let mut state = run.state.clone();
        assert!(lose_hp(&mut state, amount, kind));
        assert_eq!(state.hp, hp, "{amount} {kind:?}");
    }
    run.state.relics.push(RelicId::TungstenRod);
    let mut state = run.state.clone();
    assert!(lose_hp(&mut state, 4, OutOfCombatLoss::Damage { owned: true }));
    assert_eq!(state.hp, 10, "Torii to 1, then Tungsten Rod to 0");
    event_frame::choose_constructed(&mut run, vec![Outcome::Damage(4), Outcome::NextStage(DONE)]).unwrap();
    assert_eq!(run.state.hp, 7, "event damage has no owner: Tungsten Rod only");
}

/// Review I1 (global): every card-reward modifier applies to an event combat's card reward (source:
/// `changeNumberOfCardsInReward`, `changeRareCardRewardChance`, the bowl button): Mind Bloom's
/// I am War with Question Card shows 4 cards, Busted Crown 1, Singing Bowl its row; N'loth's Gift
/// triples the rare chance; Prayer Wheel stays MonsterRoom-only.
#[test]
fn m28_event_combat_card_reward_modifiers() {
    for (relic, cards, bowl) in [(None, 3, false), (Some(RelicId::QuestionCard), 4, false), (Some(RelicId::BustedCrown), 1, false), (Some(RelicId::SingingBowl), 3, true), (Some(RelicId::PrayerWheel), 3, false)] {
        let mut run = at_event(Character::Ironclad, 3, EventId::MindBloom, 42);
        if let Some(relic) = relic { run.state.relics.push(relic); }
        option(&mut run, 0).unwrap();
        super::m28_world::win(&mut run);
        assert_eq!(view(&run).choices.iter().filter(|c| c.kind == RoomChoiceKind::CardReward).count(), 1, "{relic:?}: one offer");
        let id = row(&run, &RoomChoiceKind::CardReward).id;
        run.step(&GameAction::Choose(id)).unwrap();
        let rows = view(&run).choices;
        assert_eq!(rows.iter().filter(|c| matches!(c.kind, RoomChoiceKind::Card { .. })).count(), cards, "{relic:?}");
        assert_eq!(rows.iter().any(|c| c.kind == RoomChoiceKind::SingingBowl), bowl, "{relic:?}");
    }
    let mut state = at_event(Character::Silent, 3, EventId::MindBloom, 43).state;
    state.relics.push(RelicId::NlothsGift);
    assert_eq!(crate::items::run_laws::card_reward_shape(&state, crate::items::run_laws::RewardRoom::Event).rare_factor, 3);
}

/// Review M3: the executor's preconditions are simulated in order. Remove one then transform two
/// needs three purgeable cards; with two the list is rejected before anything applies. Ectoplasm
/// blocks a gain that a later payment would need.
#[test]
fn m28_executor_check_is_cumulative() {
    let list = || vec![Outcome::Select { op: EventSelectOp::Remove, count: 1, skippable: false },
        Outcome::Select { op: EventSelectOp::Transform, count: 2, skippable: false }, Outcome::NextStage(DONE)];
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 44);
    let mut bane = run.state.deck[0].clone(); bane.id = CardId::AscendersBane; bane.instance = 9_002;
    run.state.deck.truncate(2); run.state.deck.push(bane);
    let before = run.clone();
    assert_eq!(event_frame::choose_constructed(&mut run, list()), Err(RunError::InvalidAction), "two purgeable cards");
    assert!(run == before);
    let mut extra = run.state.deck[0].clone(); extra.instance = 9_003; run.state.deck.push(extra);
    event_frame::choose_constructed(&mut run, list()).unwrap();
    assert!(run.state.pending.is_some(), "three purgeable cards: the removal opens");
    // Ectoplasm: the 10 gold is never gained, so the payment fails its precondition.
    let mut run = at_event(Character::Silent, 1, EventId::GoldenShrine, 45);
    run.state.relics.push(RelicId::Ectoplasm);
    let gold = run.state.gold;
    let before = run.clone();
    assert_eq!(event_frame::choose_constructed(&mut run, vec![Outcome::GainGold(10), Outcome::LoseGold(gold + 5), Outcome::NextStage(DONE)]), Err(RunError::InvalidAction));
    assert!(run == before);
}

/// Review M1 (PM ruling Q2): a purchase selects the relic, pays, then grants it; below the price it
/// is rejected atomically.
#[test]
fn m28_executor_buy_relic() {
    use rooms::standard::event_exec::RelicChoice;
    let mut run = at_event(Character::Defect, 1, EventId::GoldenShrine, 46);
    run.state.gold = 84;
    let before = run.clone();
    let buy = || vec![Outcome::BuyRelic { choice: RelicChoice::Specific(RelicId::Anchor), gold: 85 }, Outcome::NextStage(DONE)];
    assert_eq!(event_frame::choose_constructed(&mut run, buy()), Err(RunError::InvalidAction));
    assert!(run == before);
    run.state.gold = 85;
    event_frame::choose_constructed(&mut run, buy()).unwrap();
    assert_eq!((run.state.gold, run.state.relics.contains(&RelicId::Anchor)), (0, true));
}

/// Review M4 (PM ruling): with no purgeable card, Elegance stays offered, skips the grid, removes
/// nothing and shows its leave screen (`BackToBasics`); Purifier stays offered and completes with no
/// removal (SOFTLOCK-001: the original strands the player on an empty grid). No fault.
#[test]
fn m28_room_removal_without_purgeable_cards() {
    for (event, act, kind) in [(EventId::Purifier, 1, RoomChoiceKind::ShrineRemove), (EventId::AncientWriting, 2, RoomChoiceKind::AncientElegance)] {
        let mut run = at_event(Character::Watcher, act, event, 47);
        run.state.deck.truncate(1); run.state.deck[0].id = CardId::Necronomicurse;
        let removal = row(&run, &kind);
        assert_eq!(removal.status, RoomChoiceStatus::Offered, "{event:?}");
        let deck = run.state.deck.clone();
        run.step(&GameAction::Choose(removal.id)).unwrap();
        assert!(run.state.deck == deck && run.state.fault.is_none() && run.state.pending.is_none(), "{event:?}");
        if event == EventId::AncientWriting { leave(&mut run) } else { assert_eq!(view(&run).kind, RoomViewKind::Map) }
    }
}

/// Review M2 (PM ruling): a Smoke Bomb escape from a rewards-disabled event fight reopens the event
/// at its post-fight continuation.
#[test]
fn m28_event_combat_escape_without_rewards() {
    let mut run = at_event(Character::Ironclad, 3, EventId::GoldenShrine, 48);
    run.state.potions[0] = Some(crate::content::PotionId::SmokeBomb);
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::OrbWalker, rewards: None, next_stage: 0, elite: false }]).unwrap();
    run.choose_combat(CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert!(run.state.combat.is_none() && run.state.fault.is_none());
    let rows = view(&run);
    assert_eq!(rows.kind, RoomViewKind::Event(EventId::GoldenShrine));
    assert!(rows.choices.iter().all(|c| matches!(c.kind, RoomChoiceKind::EventOption { stage: 0, .. }) && c.status == RoomChoiceStatus::Offered), "the event reopens: {rows:?}");
    option(&mut run, 2).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

/// Review M2 (PM ruling, as M27's m6): a Smoke Bomb escape from an event fight with a reward screen
/// still builds the EventRoom rewards and rolls the potion (RNG and offset consumed), then shows
/// Proceed only, to the map.
#[test]
fn m28_event_combat_escape_with_rewards() {
    use rooms::standard::event_exec::RewardSpec;
    let mut run = at_event(Character::Silent, 3, EventId::GoldenShrine, 49);
    run.state.potions[0] = Some(crate::content::PotionId::SmokeBomb);
    let (offset, gold) = (run.state.potion_offset, run.state.gold);
    let spec = RewardSpec { gold: 20, gold_range: None, relics: vec![rooms::standard::event_exec::RelicChoice::Tier(crate::content::RelicTier::Common)] };
    // M28e (PM check m28e/source-check-open-b-v1 item 12): the hidden relic row leaves the pool at fight entry.
    let pools = run.state.relic_pools.clone();
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::OrbWalker, rewards: Some(spec), next_stage: DONE, elite: false }]).unwrap();
    assert_ne!(run.state.relic_pools, pools, "the hidden relic row was built");
    let started = run.state.relic_pools.clone();
    run.choose_combat(CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    let rows = view(&run);
    assert_eq!((rows.kind, rows.choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>()), (RoomViewKind::Rewards, vec![RoomChoiceKind::Proceed]));
    assert!((run.state.potion_offset - offset).abs() == 10, "the potion roll ran: {offset} -> {}", run.state.potion_offset);
    assert_eq!(run.state.relic_pools, started, "the escape takes nothing more");
    let id = row(&run, &RoomChoiceKind::Proceed).id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert_eq!((view(&run).kind, run.state.gold), (RoomViewKind::Map, gold), "nothing hidden is granted");
}

// ---- M28d seams (plan-check m28d X1–X7, N1, N2, F1, F2) ----

/// N1 (M28c review v2): buying a specific relic already owned is rejected by the preconditions,
/// atomically, as obtaining it is; before M28d it passed `check` and faulted the Run.
#[test]
fn m28_executor_buy_owned_specific_relic_rejected() {
    use rooms::standard::event_exec::RelicChoice;
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 60);
    let before = run.clone();
    let list = vec![Outcome::BuyRelic { choice: RelicChoice::Specific(RelicId::BurningBlood), gold: 10 }, Outcome::NextStage(DONE)];
    assert_eq!(event_frame::choose_constructed(&mut run, list), Err(RunError::InvalidAction));
    assert!(run == before && run.state.fault.is_none());
}

/// N2 (M28c review v2): the cumulative check is exact. An earlier selection takes from a later one
/// only the cards it is forced to share; deterministic card outcomes change the deck first.
#[test]
fn m28_executor_check_is_exact() {
    // A 10-card deck with exactly one upgradeable card: remove one (of the other nine), then upgrade.
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 61);
    for card in run.state.deck.iter_mut() { if card.id != CardId::Bash { card.upgrades = 1; } }
    assert_eq!(rooms::standard::event_exec::select_candidates(&run.state, EventSelectOp::Upgrade).len(), 1);
    let list = || vec![Outcome::Select { op: EventSelectOp::Remove, count: 1, skippable: false },
        Outcome::Select { op: EventSelectOp::Upgrade, count: 1, skippable: false }, Outcome::NextStage(DONE)];
    assert_eq!(rooms::standard::event_exec::check(&run.state, &list()), Ok(()), "the reviewer probe is now allowed");
    // Forced overlap: a two-card deck whose one upgradeable card must be removed with the other.
    let bash = run.state.deck.iter().find(|c| c.id == CardId::Bash).unwrap().clone();
    run.state.deck.truncate(1); run.state.deck.push(bash);
    let remove_two = vec![Outcome::Select { op: EventSelectOp::Remove, count: 2, skippable: false },
        Outcome::Select { op: EventSelectOp::Upgrade, count: 1, skippable: false }, Outcome::NextStage(DONE)];
    assert_eq!(rooms::standard::event_exec::check(&run.state, &remove_two), Err(RunError::InvalidAction));
    // An upgrade takes nothing from a later removal.
    let upgrade_then_remove = vec![Outcome::Select { op: EventSelectOp::Upgrade, count: 1, skippable: false },
        Outcome::Select { op: EventSelectOp::Remove, count: 2, skippable: false }, Outcome::NextStage(DONE)];
    assert_eq!(rooms::standard::event_exec::check(&run.state, &upgrade_then_remove), Ok(()));
    // A deterministic obtain before a selection adds its candidate (Note For Yourself's shape):
    // with no purgeable card, obtain-then-choose is allowed; an Omamori-negated curse adds nothing.
    let mut run = at_event(Character::Silent, 1, EventId::GoldenShrine, 62);
    run.state.deck.truncate(1); run.state.deck[0].id = CardId::Necronomicurse;
    let obtain = |id| vec![Outcome::ObtainCard { id, upgrades: 0 }, Outcome::Select { op: EventSelectOp::Choose, count: 1, skippable: false }, Outcome::NextStage(DONE)];
    assert_eq!(rooms::standard::event_exec::check(&run.state, &obtain(CardId::Neutralize)), Ok(()));
    assert_eq!(rooms::standard::event_exec::check(&run.state, &[Outcome::Select { op: EventSelectOp::Choose, count: 1, skippable: false }, Outcome::NextStage(DONE)]), Err(RunError::InvalidAction));
    let curse = vec![Outcome::ObtainCurse(CardId::Regret), Outcome::Select { op: EventSelectOp::Remove, count: 1, skippable: false }, Outcome::NextStage(DONE)];
    assert_eq!(rooms::standard::event_exec::check(&run.state, &curse), Ok(()));
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, 2);
    assert_eq!(rooms::standard::event_exec::check(&run.state, &curse), Err(RunError::InvalidAction), "Omamori negates the curse");
    // A removal of a card an earlier outcome removed is rejected.
    let instance = run.state.deck[0].instance;
    assert_eq!(rooms::standard::event_exec::check(&run.state, &[Outcome::ObtainCard { id: CardId::Neutralize, upgrades: 0 }, Outcome::RemoveSpecific(instance), Outcome::RemoveSpecific(instance), Outcome::Select { op: EventSelectOp::Choose, count: 1, skippable: false }, Outcome::NextStage(DONE)]), Err(RunError::InvalidAction));
}

/// X1: `Roll` draws at its point of the list and continues with the first arm whose bound exceeds
/// the draw; every arm is checked before anything applies, and a Roll must end its list.
#[test]
fn m28_executor_roll_arms() {
    let roll = |a: u32, b: u32| vec![Outcome::Roll { bound: 2, arms: vec![(1, vec![Outcome::GainGold(a), Outcome::NextStage(DONE)]), (2, vec![Outcome::GainGold(b), Outcome::NextStage(DONE)])] }];
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..24u8 {
        let mut run = at_event(Character::Defect, 1, EventId::GoldenShrine, seed);
        let gold = run.state.gold;
        event_frame::choose_constructed(&mut run, roll(1, 2)).unwrap();
        seen.insert(run.state.gold - gold);
        leave(&mut run);
    }
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![1, 2], "both arms are reached");
    let mut run = at_event(Character::Defect, 1, EventId::GoldenShrine, 63);
    let failing = vec![Outcome::Roll { bound: 2, arms: vec![(1, vec![Outcome::NextStage(DONE)]), (2, vec![Outcome::LoseGold(run.state.gold + 1), Outcome::NextStage(DONE)])] }];
    let before = run.clone();
    assert_eq!(event_frame::choose_constructed(&mut run, failing), Err(RunError::InvalidAction), "an arm failing its precondition rejects the option");
    assert!(run == before);
    for bad in [vec![Outcome::Roll { bound: 2, arms: vec![(1, vec![Outcome::NextStage(DONE)])] }],
        vec![Outcome::Roll { bound: 1, arms: vec![(1, vec![Outcome::NextStage(DONE)])] }, Outcome::GainGold(1)]] {
        assert_eq!(rooms::standard::event_exec::check(&run.state, &bad), Err(RunError::InvalidAction), "{bad:?}");
    }
}

/// X2: `SetValue` writes the frame's captured values (per-event progress).
#[test]
fn m28_executor_set_value() {
    let mut run = at_event(Character::Watcher, 1, EventId::GoldenShrine, 64);
    event_frame::choose_constructed(&mut run, vec![Outcome::SetValue { slot: 3, value: 7 }, Outcome::NextStage(0)]).unwrap();
    let frame = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap();
    assert_eq!((frame.captured.values.clone(), frame.stage), (vec![0, 0, 0, 7], 0));
}

/// X4: an event's non-combat reward screen (Lab, Wheel of Change): the relic rows, `potions` flat
/// class potions, then Proceed; no card offer. Claims use the shared screen; Proceed reaches the map.
#[test]
fn m28_event_rewards_screen() {
    use rooms::standard::event_exec::RelicChoice;
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = at_event(character, 1, EventId::GoldenShrine, 65);
        event_frame::choose_constructed(&mut run, vec![Outcome::Rewards { potions: 3, relics: vec![RelicChoice::RandomTier] }]).unwrap();
        let screen = view(&run);
        assert_eq!(screen.kind, RoomViewKind::Rewards);
        let kinds: Vec<_> = screen.choices.iter().map(|c| c.kind.clone()).collect();
        assert!(matches!(kinds[0], RoomChoiceKind::Relic(_)) && kinds[1..4].iter().all(|k| matches!(k, RoomChoiceKind::Potion(_))) && kinds[4] == RoomChoiceKind::Proceed && kinds.len() == 5, "{character:?}: {kinds:?}");
        let pool = crate::items::potion_generation::candidates(character, false).unwrap();
        assert!(kinds[1..4].iter().all(|k| matches!(k, RoomChoiceKind::Potion(p) if pool.contains(p))), "{character:?}: class potions");
        let RoomChoiceKind::Potion(potion) = kinds[1] else { unreachable!() };
        let id = screen.choices[1].id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert!(run.state.potions.contains(&Some(potion)));
        let id = row(&run, &RoomChoiceKind::Proceed).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

/// X5: an event fight's listed gold range is drawn when it starts and merged with its fixed gold
/// into one row (`addGoldToRewards`); listed relics are fixed or rolled reward offers; `elite`
/// sets the source's `eliteTrigger` for the fight only.
#[test]
fn m28_event_combat_rewards_and_elite_trigger() {
    use rooms::standard::event_exec::{RelicChoice, RewardSpec};
    let mut golds = std::collections::BTreeSet::new();
    for seed in 0..12u8 {
        let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 70 + seed);
        let spec = RewardSpec { gold: 30, gold_range: Some((25, 35)), relics: vec![RelicChoice::Specific(RelicId::OddMushroom), RelicChoice::RandomTier] };
        event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::LagavulinEvent, rewards: Some(spec), next_stage: DONE, elite: seed % 2 == 0 }]).unwrap();
        let elite = run.state.combat.as_ref().unwrap().item_context().unwrap().battle.elite_trigger;
        assert_eq!(elite, seed % 2 == 0, "eliteTrigger follows the event");
        super::m28_world::win(&mut run);
        let rows: Vec<_> = view(&run).choices.into_iter().map(|c| c.kind).collect();
        let gold: Vec<u32> = rows.iter().filter_map(|k| match k { RoomChoiceKind::Gold(n) => Some(*n), _ => None }).collect();
        assert_eq!(gold.len(), 1, "one merged gold row: {rows:?}");
        assert!((55..=65).contains(&gold[0]), "{gold:?}");
        golds.insert(gold[0]);
        assert!(rows.contains(&RoomChoiceKind::Relic(RelicId::OddMushroom)));
        assert_eq!(rows.iter().filter(|k| matches!(k, RoomChoiceKind::Relic(_))).count(), 2, "{rows:?}");
        assert!(!rows.contains(&RoomChoiceKind::Emerald));
    }
    assert!(golds.len() > 1, "the range is drawn: {golds:?}");
}

/// F1/F2 (design §5.8): `ThreeFungiBeasts` is three independent Fungi Beasts in slot order;
/// `LagavulinEvent` is Dead Adventurer's awake Lagavulin (no Block 8 or Metallicize 8; Siphon Soul
/// first), both admitted only as event fights of a standard Run.
#[test]
fn m28_formation_fights() {
    use crate::content::EnemyId;
    let mut run = at_event(Character::Silent, 1, EventId::GoldenShrine, 80);
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::ThreeFungiBeasts, rewards: None, next_stage: DONE, elite: false }]).unwrap();
    let enemies = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies;
    assert_eq!(enemies.len(), 3);
    assert!(enemies.iter().all(|e| e.enemy_id == EnemyId::FungiBeast));
    assert_eq!(enemies.iter().map(|e| e.logical_slot).collect::<Vec<_>>(), vec![0, 1, 2]);
    assert!(enemies.iter().all(|e| (22..=28).contains(&e.hp) && e.powers.iter().any(|p| p.id.0 == "spore_cloud")), "{enemies:?}");
    let ids: std::collections::BTreeSet<_> = enemies.iter().map(|e| e.instance_id.clone()).collect();
    assert_eq!(ids.len(), 3, "separate actors");
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 81);
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::LagavulinEvent, rewards: None, next_stage: DONE, elite: true }]).unwrap();
    let enemies = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies;
    assert_eq!(enemies.len(), 1);
    let lagavulin = &enemies[0];
    assert!((109..=111).contains(&lagavulin.hp));
    assert_eq!((lagavulin.block, lagavulin.intent.as_str()), (0, "siphon"));
    assert!(!lagavulin.powers.iter().any(|p| p.id.0 == "metallicize"), "{:?}", lagavulin.powers);
    // The ordinary Lagavulin still sleeps behind Block 8 and Metallicize 8 (the map elite).
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 82);
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::Lagavulin, rewards: None, next_stage: DONE, elite: true }]).unwrap();
    let enemies = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies;
    assert_eq!((enemies[0].block, enemies[0].intent.as_str()), (8, "sleep"));
    assert!(enemies[0].powers.iter().any(|p| p.id.0 == "metallicize" && p.amount == 8), "{:?}", enemies[0].powers);
}

/// Chunk plan §3, extended by M28d: no frozen profile reaches the new event formations, from a map
/// room, an event origin or training-free entry; each is rejected with the Run unchanged.
#[test]
fn m28_frozen_profiles_never_reach_standard_owners_d() {
    use RoomScenario::*;
    let frozen = [
        (M5Act1Captured, Character::Ironclad), (M10IroncladHeart, Character::Ironclad), (M12SilentAct1, Character::Silent),
        (M17DefectAct1, Character::Defect), (M22WatcherAct1, Character::Watcher), (M25WatcherHeart, Character::Watcher),
    ];
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    for (scenario, character) in frozen {
        let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile.clone(), rng(4), scenario).unwrap();
        let lament = view(&run).choices.iter().find(|row| row.kind == RoomChoiceKind::NeowLament).map(|row| row.id);
        if let Some(id) = lament { run.step(&GameAction::Choose(id)).unwrap(); }
        let node = run.state.map.as_ref().and_then(|map| map.nodes.first()).map_or(NodeId(0), |n| n.id);
        for encounter in [EncounterId::ThreeFungiBeasts, EncounterId::LagavulinEvent] {
            for origin in [CombatOrigin::MapRoom { node, kind: MapNodeKind::Monster }, CombatOrigin::Event { node, event: EventId::DeadAdventurer, stage: 0 }] {
                let before = run.clone();
                assert_eq!(run.enter_owned(encounter, origin), Err(RunError::Unsupported("encounter outside authored M3 profile")), "{scenario:?} {encounter:?} {origin:?}");
                assert!(run == before, "{scenario:?} {encounter:?}");
            }
        }
    }
}

/// N2 (M28c review v2): the precondition check is exact for every landed body. It simulates
/// deterministic card outcomes and the player's choices exactly; only a random deck outcome before a
/// selection would fall back to the worst case. No landed body, at any stage, option or Roll arm, on
/// any character, has that shape.
#[test]
fn m28_event_body_shapes_check_exactly() {
    fn walk(list: &[Outcome], random: bool, path: &str) {
        let mut random = random;
        for outcome in list {
            match outcome {
                Outcome::TransformRandom(_) | Outcome::UpgradeRandom(_) | Outcome::UpgradeAll => random = true,
                Outcome::Select { .. } => assert!(!random, "{path}: a selection after a random deck outcome"),
                Outcome::Roll { arms, .. } => for (_, arm) in arms { walk(arm, random, path) },
                _ => {}
            }
        }
    }
    let mut landed = 0;
    for event in crate::content::Catalog::standard().events().iter().map(|row| row.id) {
        let Some(body) = rooms::standard::events::body(event) else { continue };
        landed += 1;
        let row = crate::content::Catalog::standard().event(event).unwrap();
        let act = (1..=3).find(|act| row.acts & (1 << act) != 0).unwrap();
        for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
            let (mut run, node) = before_event(character, act, event, 90);
            run.state.floor = run.state.floor.max(7); run.state.gold = 300;
            // Entry adds one (PM ruling m28f Q2): the node sits one floor above the Run.
            let entered = run.state.floor + 1;
            run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == node).unwrap().floor = entered;
            let mut curse = run.state.deck[0].clone(); curse.id = CardId::Regret; curse.instance = 9_500; run.state.deck.push(curse);
            if run.step(&GameAction::Move(node)).is_err() { continue }
            let captured = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().map(|f| f.captured.clone()).unwrap_or_default();
            for stage in (0..=16).chain([DONE]) {
                for option in 0..=12 {
                    let Ok(list) = (body.choose)(&run.state, &captured, stage, option) else { continue };
                    walk(&list, false, &format!("{event:?} {character:?} stage {stage} option {option}"));
                }
                if let Some(hook) = body.selected {
                    for card in run.state.deck.clone() {
                        if let Ok(list) = hook(&run.state, &captured, stage, std::slice::from_ref(&card)) {
                            assert!(!list.iter().any(|o| matches!(o, Outcome::Select { .. } | Outcome::Roll { .. })), "{event:?}: a selection hook opens no selection");
                        }
                    }
                }
            }
        }
    }
    assert!(landed >= 4, "{landed} landed bodies");
}

/// Global fix (lane B2 finding): Note For Yourself's take obtains the stored card directly, which
/// bypasses Omamori's obtain-effect constructor but not the other obtain hooks (event-neow research
/// §6 "Note direct obtain"). Before M28d a stored curse with an Omamori charge was negated and the
/// direct-obtain owner returned an error.
#[test]
fn m28_note_direct_obtain_bypasses_omamori() {
    let mut run = at_event(Character::Silent, 1, EventId::GoldenShrine, 66);
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, 2);
    let size = run.state.deck.len();
    mutations::obtain_stored_card(&mut run.state, CardId::Regret, 0).unwrap();
    assert_eq!((run.state.deck.len(), count(&run, CardId::Regret)), (size + 1, 1));
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&2), "no charge spent");
    // An ordinary obtain of the same curse is still negated and spends a charge.
    assert_eq!(mutations::obtain_card_or_negated(&mut run.state, CardId::Regret, 0).unwrap(), None);
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&1));
    // The egg obtain hook still applies to a direct obtain.
    run.state.relics.push(RelicId::MoltenEgg);
    let instance = mutations::obtain_stored_card(&mut run.state, CardId::IronWave, 0).unwrap();
    assert_eq!(run.state.deck.iter().find(|c| c.instance == instance).unwrap().upgrades, 1);
}
