//! M33 run rules (plan items 1, 2): the Standard cap at A20 and G1, the source's Act 3 unseen-boss
//! priority. Literals come from the M33 design (§6.1, §6.6) and lane B §6 (`TheBeyond.initializeBoss`:
//! CROW, DONUT, WIZARD), worked by hand, never from the code under test.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{Character, EncounterId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn standard(character: Character, ascension: u8, full: bool) -> Result<Run, RunError> { standard_seeded(character, ascension, full, 0x33) }
fn standard_seeded(character: Character, ascension: u8, full: bool, seed: u8) -> Result<Run, RunError> {
    Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(seed), RoomScenario::Standard)
}

#[test]
fn m33_standard_creation_a0_to_a20_all_characters() {
    for character in CHARACTERS {
        for ascension in 0..=20 {
            for full in [false, true] {
                let run = standard(character, ascension, full).unwrap_or_else(|e| panic!("{character:?} A{ascension}: {e:?}"));
                assert_eq!(run.decision().unwrap().content_profile, Some(RoomScenario::Standard));
                assert_eq!(run.state.ascension, ascension);
                assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Neow, "{character:?} A{ascension}");
            }
        }
    }
}

#[test]
fn m33_standard_rejects_a21() {
    for character in CHARACTERS {
        assert!(standard(character, 20, false).is_ok(), "{character:?} A20 (t-1)");
        assert_eq!(standard(character, 21, false).err(), Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A21");
        assert_eq!(rooms::standard::build(RunConfig { character, ascension: 21 }, profile(false), rng(1)).err(),
            Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A21");
    }
}

/// The source's unseen priority per act (lane B §6): Exordium GUARDIAN, GHOST, SLIME; City CHAMP,
/// AUTOMATON, COLLECTOR; Beyond CROW, DONUT, WIZARD.
const PRIORITY: [[(&str, EncounterId); 3]; 3] = [
    [("Guardian", EncounterId::Guardian), ("Hexaghost", EncounterId::Hexaghost), ("SlimeBoss", EncounterId::SlimeBoss)],
    [("Champ", EncounterId::Champ), ("Automaton", EncounterId::BronzeAutomaton), ("Collector", EncounterId::Collector)],
    [("AwakenedOne", EncounterId::AwakenedOne), ("DonuAndDeca", EncounterId::DonuAndDeca), ("TimeEater", EncounterId::TimeEater)],
];

fn standard_state(ascension: u8, act: u8, seen: Vec<&'static str>) -> RunState {
    let mut run = standard(Character::Ironclad, ascension, false).unwrap();
    run.state.act = act;
    run.state.seen_bosses[(act - 1) as usize] = seen;
    run.state
}

/// G1 (lane B FINDING 2): with only Awakened One seen, the source picks Donu and Deca, at every
/// ascension; the Standard selector picked Time Eater (domain order).
#[test]
fn m33_g1_act3_unseen_priority() {
    for ascension in [0, 19, 20] {
        let mut state = standard_state(ascension, 3, vec!["AwakenedOne"]);
        assert_eq!(world::act_boss(&mut state).unwrap(), EncounterId::DonuAndDeca, "A{ascension}");
    }
}

/// Eight seen subsets × Acts 1–3 against the hand-worked source priority. Act 3 {Awakened One} is the
/// G1 row; every other unseen row is preservation; all-seen rows assert membership only (native draw).
#[test]
fn m33_boss_unseen_priority_matrix() {
    for act in 1..=3u8 {
        let priority = PRIORITY[(act - 1) as usize];
        for mask in 0..8u8 {
            let seen: Vec<&'static str> = (0..3).filter(|bit| mask & (1 << bit) != 0).map(|bit| priority[bit].0).collect();
            let mut state = standard_state(0, act, seen.clone());
            let boss = world::act_boss(&mut state).unwrap();
            match priority.iter().find(|(name, _)| !seen.contains(name)) {
                Some((_, expected)) => assert_eq!(boss, *expected, "act {act} seen {seen:?}"),
                None => assert!(priority.iter().any(|(_, id)| *id == boss), "act {act} all seen: {boss:?}"),
            }
        }
    }
}

/// A20 changes the Act 3 route only. Start resources and Act 1 behavior remain the A19 row.
/// Start HP/max HP literals from M33 lane B §2: Ironclad 68/75, Silent 59/66, Defect 64/71,
/// Watcher 61/68. Ascender's Bane is compared as part of the complete deck.
#[test]
fn m33_a20_run_rules_equal_a19() {
    use crate::combat::{self, CardInstanceId, PlayerState};
    use crate::combat::enemies::SetupContext;
    use crate::content::CardId;

    let expected_hp = [
        (Character::Ironclad, 68, 75), (Character::Silent, 59, 66),
        (Character::Defect, 64, 71), (Character::Watcher, 61, 68),
    ];
    for (character, hp, max_hp) in expected_hp {
        for seed in [3u8, 17, 44, 91] {
            for full in [false, true] {
                let (mut a19, mut a20) = (standard_seeded(character, 19, full, seed).unwrap(), standard_seeded(character, 20, full, seed).unwrap());
                let kinds = |run: &Run| rooms::public(run).unwrap().choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>();
                assert_eq!(kinds(&a19), kinds(&a20), "{character:?} seed {seed} full {full}: Neow screen");
                assert_eq!((a19.state.hp, a19.state.max_hp), (hp, max_hp), "{character:?} A19 start HP, seed {seed}, full {full}");
                assert_eq!((a20.state.hp, a20.state.max_hp), (hp, max_hp), "{character:?} A20 start HP, seed {seed}, full {full}");
                assert_eq!(a19.state.gold, a20.state.gold, "{character:?} seed {seed} full {full}: gold");
                assert_eq!(a19.state.deck, a20.state.deck, "{character:?} seed {seed} full {full}: master deck");
                assert!(a19.state.deck.iter().any(|card| card.id == CardId::AscendersBane));
                assert_eq!(a19.state.relics, a20.state.relics);
                assert_eq!(a19.state.potions, a20.state.potions);
                let choice_kind = kinds(&a19).get(1).cloned().expect("Neow continue/choice row");
                assert_eq!(kinds(&a20).get(1), Some(&choice_kind), "same Neow selection row");
                for run in [&mut a19, &mut a20] {
                    let row = rooms::public(run).unwrap().choices.into_iter().find(|row| row.kind == choice_kind).unwrap();
                    run.step(&GameAction::Choose(row.id)).unwrap();
                }
                // A Neow reward may open a child screen (card, relic or selection): take the same first legal
                // action on both Runs until the map, with equal decisions on the way.
                for _ in 0..8 {
                    if a19.state.pending.is_none() && rooms::public(&a19).unwrap().kind == RoomViewKind::Map { break }
                    let (d19, d20) = (a19.decision().unwrap(), a20.decision().unwrap());
                    let actions = |d: &PublicDecision| d.candidates.iter().map(|c| c.action.clone()).collect::<Vec<_>>();
                    assert_eq!(actions(&d19), actions(&d20), "{character:?} seed {seed} full {full}: Neow child decision");
                    let action = d19.candidates[0].action.clone();
                    a19.step(&action).unwrap();
                    a20.step(&action).unwrap();
                }
                assert_eq!(rooms::public(&a20).unwrap().kind, RoomViewKind::Map, "{character:?} seed {seed} full {full}");
                let map_fields = |run: &Run| {
                    let map = run.state.map.as_ref().unwrap();
                    (map.boss_icon, map.nodes.iter().map(|node| (node.id, node.x, node.y, node.floor, node.kind,
                        node.edges.clone(), node.encounter, node.visited, node.revealed)).collect::<Vec<_>>())
                };
                assert_eq!(map_fields(&a19), map_fields(&a20), "{character:?} seed {seed} full {full}: first map");
                super::m28_world::drive(&mut a19, |run| run.state.combat.is_some());
                super::m28_world::drive(&mut a20, |run| run.state.combat.is_some());
                let c19 = a19.state.combat.as_ref().unwrap();
                let c20 = a20.state.combat.as_ref().unwrap();
                let origin = |run: &Run| run.state.active_combat.as_ref().unwrap().origin.clone();
                assert_eq!(origin(&a19), origin(&a20), "same first map move: {character:?} seed {seed} full {full}");
                assert_eq!(a19.state.active_combat.as_ref().unwrap().encounter, a20.state.active_combat.as_ref().unwrap().encounter);
                assert_eq!(c19.public_full().unwrap(), c20.public_full().unwrap(), "first combat public setup and intents");

                // Act 4 setup owner comparison: identical player and deck inputs, RNG seed, and setup visit.
                let cards = a19.state.deck.iter().map(|card| combat::permanent_live_card(card.id, CardInstanceId(card.instance), card.upgrades)).collect::<Vec<_>>();
                for encounter in [EncounterId::ShieldAndSpear, EncounterId::CorruptHeart] {
                    let setup = |ascension| {
                        let mut rng = RunRng::new([seed; 32], Partition::Test, 0);
                        let context = SetupContext { player: PlayerState { hp: 400, max_hp: 400, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
                            player_powers: vec![], character, cards: cards.clone(), opening_draw: 5, encounter_visit: 7 };
                        combat::enemies::setup(encounter, 4, ascension, &mut rng, context).unwrap()
                    };
                    let x = setup(19); let y = setup(20);
                    let public = |entry: &combat::CombatEntry| entry.setup.enemies.iter().map(|enemy|
                        (enemy.enemy_id, enemy.hp, enemy.max_hp, enemy.intent.clone(), enemy.powers.clone())).collect::<Vec<_>>();
                    assert_eq!(public(&x), public(&y), "{character:?} {encounter:?}: enemy ids, HP and public intents");
                }
            }
        }
    }
}
