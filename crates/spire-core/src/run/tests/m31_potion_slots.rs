//! M31 A11 potion slots and G2 (plan item 2; design §6.2–§6.3). Source: `m31-source-check-v1.md` §B —
//! base slots 3, or 2 from A11, made at player construction before Neow; Potion Belt adds two to
//! the live count; losing it keeps the slots and their potions usable. Driven through the public
//! Standard route (`Run::new_room_scenario`) unless a row says otherwise.
use super::super::*;
use crate::combat::{rng::Partition, CombatAction};
use crate::content::{Character, PotionId, RelicId};
use crate::items::lifecycle::RelicSource;

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn standard(character: Character, ascension: u8, full: bool) -> Run {
    Run::new_room_scenario(RunConfig { character, ascension }, CreationProfile { neow_eligible: full, ..CreationProfile::experienced() },
        rng(0x31), RoomScenario::Standard).unwrap_or_else(|e| panic!("{character:?} A{ascension}: {e:?}"))
}
/// A Standard Run at `ascension` (short Neow) past Neow's max-HP blessing, on its Act 1 map.
pub(super) fn on_map(character: Character, ascension: u8) -> Run {
    let mut run = standard(character, ascension, false);
    let max_hp = rooms::public(&run).unwrap().choices.iter().find(|c| matches!(c.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
    run.step(&GameAction::Choose(max_hp)).unwrap();
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
    run
}
/// Turn the first reachable Act 1 node into `kind` and enter it through the public Move.
pub(super) fn enter_first(run: &mut Run, kind: MapNodeKind) {
    let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = kind;
    run.step(&GameAction::Move(target)).unwrap();
}
fn combat_potions(run: &Run) -> Vec<Option<PotionId>> {
    run.state.combat.as_ref().unwrap().item_context().unwrap().potions().to_vec()
}
fn use_potion(run: &mut Run, slot: usize) -> Result<RunTransition, RunError> {
    run.step(&GameAction::Combat(CombatAction::UsePotion { slot, target: None }))
}

#[test]
fn m31_a11_two_slots_before_neow() {
    for character in CHARACTERS {
        for full in [false, true] {
            assert_eq!(standard(character, 10, full).state.potions, vec![None; 3], "{character:?} A10 (t-1)");
            let run = standard(character, 11, full);
            assert_eq!(run.state.potions, vec![None; 2], "{character:?} A11");
            assert_eq!(run.decision().unwrap().player.potions.len(), 2, "{character:?} A11 public view");
        }
    }
}

#[test]
fn m31_a11_first_combat_enters() {
    for character in CHARACTERS {
        let mut run = on_map(character, 11);
        enter_first(&mut run, MapNodeKind::Monster);
        assert!(run.state.combat.is_some(), "{character:?}: the A11 Standard combat entered");
        assert_eq!(combat_potions(&run).len(), 2, "{character:?}: two slots in combat");
        super::m28_world::win(&mut run);
        assert_eq!(run.state.potions.len(), 2, "{character:?}: two slots after settlement");
    }
}

#[test]
fn m31_a11_potion_belt_four_slots() {
    for character in CHARACTERS {
        let mut run = on_map(character, 11);
        crate::items::obtain_relic(&mut run.state, RelicId::PotionBelt, RelicSource::Reward).unwrap();
        assert_eq!(run.state.potions, vec![None; 4], "{character:?}: Belt appends two empty slots to two");
        run.state.potions[2] = Some(PotionId::EnergyPotion);
        run.state.potions[3] = Some(PotionId::EnergyPotion);
        enter_first(&mut run, MapNodeKind::Monster);
        assert_eq!(combat_potions(&run).len(), 4, "{character:?}: four slots at combat entry");
        let legal = run.decision().unwrap().candidates.iter().filter_map(|c| match c.action {
            GameAction::Combat(CombatAction::UsePotion { slot, .. }) => Some(slot), _ => None }).collect::<Vec<_>>();
        assert!(legal.contains(&2) && legal.contains(&3), "{character:?}: slots 2 and 3 advertised, saw {legal:?}");
        assert!(use_potion(&mut run.clone(), 4).is_err(), "{character:?}: slot 4 does not exist");
        use_potion(&mut run, 3).unwrap();
        assert_eq!(combat_potions(&run)[3], None, "{character:?}: slot 3 used");
        super::m28_world::win(&mut run);
        assert_eq!(run.state.potions, vec![None, None, Some(PotionId::EnergyPotion), None], "{character:?}: settlement keeps four");
    }
}

#[test]
fn m31_a0_potion_belt_unchanged() {
    for character in CHARACTERS {
        let mut run = on_map(character, 0);
        crate::items::obtain_relic(&mut run.state, RelicId::PotionBelt, RelicSource::Reward).unwrap();
        assert_eq!(run.state.potions, vec![None; 5], "{character:?}: A0 Belt is five slots");
        run.state.potions[4] = Some(PotionId::EnergyPotion);
        enter_first(&mut run, MapNodeKind::Monster);
        assert_eq!(combat_potions(&run).len(), 5);
        use_potion(&mut run, 4).unwrap();
        assert!(use_potion(&mut run.clone(), 5).is_err(), "{character:?}: slot 5 does not exist");
    }
}

/// G2 (design §6.3; source-check §B: no Belt unequip hook, TopPanel/PotionPopUp read the live slots):
/// after the Belt is lost, a potion in a kept slot is usable in the next combat, at A0 and at A11.
#[test]
fn m31_lost_belt_kept_slot_usable() {
    for character in CHARACTERS {
        for (ascension, kept) in [(0_u8, 4_usize), (11, 3)] {
            let mut run = on_map(character, ascension);
            crate::items::obtain_relic(&mut run.state, RelicId::PotionBelt, RelicSource::Reward).unwrap();
            run.state.potions[kept] = Some(PotionId::EnergyPotion);
            crate::items::lose_relic(&mut run.state, RelicId::PotionBelt).unwrap();
            assert_eq!(run.state.potions.len(), kept + 1, "{character:?} A{ascension}: slots kept");
            enter_first(&mut run, MapNodeKind::Monster);
            assert!(run.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Combat(CombatAction::UsePotion { slot: kept, target: None })),
                "{character:?} A{ascension}: kept-slot potion advertised");
            use_potion(&mut run, kept).unwrap_or_else(|e| panic!("{character:?} A{ascension}: kept slot {kept}: {e:?}"));
            assert_eq!(combat_potions(&run)[kept], None);
        }
    }
}

/// A11 Belt then N'loth-style loss: four slots kept (`complete && len == base + 2`), slot-3 potion usable.
#[test]
fn m31_a11_belt_lost_keeps_four() {
    for character in CHARACTERS {
        let mut run = on_map(character, 11);
        crate::items::obtain_relic(&mut run.state, RelicId::PotionBelt, RelicSource::Reward).unwrap();
        run.state.potions[3] = Some(PotionId::EnergyPotion);
        crate::items::lose_relic(&mut run.state, RelicId::PotionBelt).unwrap();
        assert_eq!(run.state.potions.len(), 4, "{character:?}");
        enter_first(&mut run, MapNodeKind::Monster);
        assert_eq!(combat_potions(&run).len(), 4, "{character:?}: entry accepts the kept four");
        use_potion(&mut run, 3).unwrap();
        super::m28_world::win(&mut run);
        assert_eq!(run.state.potions.len(), 4, "{character:?}: settlement keeps four");
    }
}
