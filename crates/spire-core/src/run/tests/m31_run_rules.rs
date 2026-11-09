//! M31 run rules (plan items 1, 5–7): the Standard cap at A15, A13 boss and Mind Bloom gold, A14 max
//! HP through the public route and the A15 event tests. Literals come from the source check
//! (`m31-source-check-v1.md` §C–§F) and are worked by hand, never from the code under test.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::Character;

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn standard(character: Character, ascension: u8, full: bool) -> Result<Run, RunError> {
    Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(0x31), RoomScenario::Standard)
}

#[test]
fn m31_standard_creation_a0_to_a15_all_characters() {
    for character in CHARACTERS {
        for ascension in 0..=15 {
            for full in [false, true] {
                let run = standard(character, ascension, full).unwrap_or_else(|e| panic!("{character:?} A{ascension}: {e:?}"));
                assert_eq!(run.decision().unwrap().content_profile, Some(RoomScenario::Standard));
                assert_eq!(run.state.ascension, ascension);
                assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Neow, "{character:?} A{ascension}");
            }
        }
    }
}

/// Formerly `m31_standard_rejects_a16_and_above` (M32 plan item 1): the cap is A19 since M32, so A15–A17
/// are admitted and the rejected levels are 20 and 21, with the A0–A19 message. Amended at M33 (plan item 1):
/// the cap is A20, so the rejected level is 21, with the A0–A20 message.
#[test]
fn m31_standard_rejects_above_cap() {
    for character in CHARACTERS {
        for ascension in [15, 16, 17] { assert!(standard(character, ascension, false).is_ok(), "{character:?} A{ascension}"); }
        // Amended at M33 (plan item 1): the cap is A20, so A21 is the rejected level.
        for ascension in [21] {
            assert_eq!(standard(character, ascension, false).err(),
                Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A{ascension}");
            assert_eq!(rooms::standard::build(RunConfig { character, ascension }, profile(false), rng(1)).err(),
                Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A{ascension}");
        }
    }
}

// ---- A13 boss gold (design §6.6; source check §D) ------------------------------------------------

fn rooms_view(run: &Run) -> RoomView { rooms::public(run).unwrap() }
fn gold_rows(run: &Run) -> Vec<u32> {
    rooms_view(run).choices.into_iter().filter_map(|row| match row.kind { RoomChoiceKind::Gold(amount) => Some(amount), _ => None }).collect()
}
/// An Act 1 Standard boss won at `ascension` on `seed`; the boss reward's gold rows.
fn boss_gold(character: Character, ascension: u8, seed: u8, idol: bool) -> Vec<u32> {
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile(false), rng(seed), RoomScenario::Standard).unwrap();
    let max_hp = rooms_view(&run).choices.iter().find(|c| matches!(c.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
    run.step(&GameAction::Choose(max_hp)).unwrap();
    (run.state.max_hp, run.state.hp) = (999, 999);
    if idol { run.state.relics.push(crate::content::RelicId::GoldenIdol); }
    super::m31_potion_slots::enter_first(&mut run, MapNodeKind::Boss);
    super::m28_world::win(&mut run);
    gold_rows(&run)
}
/// `MathUtils.round(binary32(tmp × .75f))` for tmp = 95..=105, worked by hand (design §6.6 table):
/// 71.25→71, 72→72, 72.75→73, 73.5→74, 74.25→74, 75→75, 75.75→76, 76.5→77, 77.25→77, 78→78, 78.75→79.
const A13_BOSS: [u32; 11] = [71, 72, 73, 74, 74, 75, 76, 77, 77, 78, 79];
/// Golden Idol's +25% on the reduced base, `MathUtils.round(t × .25f)` worked by hand for t = 71..=79.
fn idol_bonus(t: u32) -> u32 { match t { 71 | 72 | 73 => 18, 74 | 75 | 76 | 77 => 19, 78 | 79 => 20, _ => unreachable!() } }

#[test]
fn m31_a13_boss_gold_table() {
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..=255_u8 {
        let a12 = boss_gold(Character::Ironclad, 12, seed, false);
        let a13 = boss_gold(Character::Ironclad, 13, seed, false);
        assert_eq!(a12.len(), 1, "seed {seed}");
        assert!((95..=105).contains(&a12[0]), "seed {seed}: A12 boss row {} is the unscaled 100 ± 5", a12[0]);
        assert_eq!(a13, vec![A13_BOSS[(a12[0] - 95) as usize]], "seed {seed}: A13 row for draw {}", a12[0]);
        seen.insert(a12[0]);
        if seen.len() == 11 { break; }
    }
    assert_eq!(seen.len(), 11, "every boss draw 95..=105 was seen");
}

#[test]
fn m31_a13_boss_gold_golden_idol() {
    for seed in 0..12_u8 {
        let base = boss_gold(Character::Silent, 13, seed, false)[0];
        assert_eq!(boss_gold(Character::Silent, 13, seed, true), vec![base + idol_bonus(base)], "seed {seed}: Idol on the reduced base");
    }
}
