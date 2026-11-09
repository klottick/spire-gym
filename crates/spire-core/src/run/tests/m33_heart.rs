//! M33 A20 public Standard Heart path from a short Neow start for every character (plan item 7).
//! Supplied frontier, disclosed per design §6.8: 400/400 HP and all three keys.
use super::super::*;
use crate::content::{Character, EncounterId};
use super::m33_double_boss::{kind, order, proceed};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn standard_on_map(character: Character, seed: u8) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension: 20 }, profile,
        RunRng::new([seed; 32], crate::combat::rng::Partition::Test, 0), RoomScenario::Standard).unwrap();
    let choice = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(choice)).unwrap();
    run
}

fn drive_checked(run: &mut Run, mut stop: impl FnMut(&Run) -> bool) {
    for _ in 0..64 {
        if stop(run) { return; }
        super::m28_world::drive(run, |candidate| stop(candidate) || candidate.state.combat.is_some());
        if stop(run) { return; }
        if run.state.combat.is_some() {
            assert_eq!(run.state.combat.as_ref().unwrap().ascension(), 20);
            super::m28_world::win(run);
            assert!(run.state.fault.is_none(), "{:?}", run.state.fault);
        }
    }
    panic!("checked route exceeded its step budget");
}

fn boss_combat(run: &Run, act: u8) -> bool {
    run.state.act == act && run.state.combat.is_some()
        && run.state.active_combat.as_ref().is_some_and(|active| matches!(active.origin, CombatOrigin::MapRoom { kind: MapNodeKind::Boss, .. }))
}

#[test]
fn m33_a20_heart_path_all_characters() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = standard_on_map(character, 0xB0 + index as u8);
        run.state.hp = 400;
        run.state.max_hp = 400;
        run.state.keys = [true; 3];

        drive_checked(&mut run, |candidate| boss_combat(candidate, 2));
        let act2 = run.state.active_combat.as_ref().unwrap().encounter;
        assert!(matches!(act2, EncounterId::Champ | EncounterId::Collector | EncounterId::BronzeAutomaton));
        assert_eq!(run.state.combat.as_ref().unwrap().ascension(), 20);
        super::m28_world::win(&mut run);

        drive_checked(&mut run, |candidate| boss_combat(candidate, 3));
        let act3_order = order(&run).expect("A20 all-seen Act 3 stores the two-boss order");
        assert_eq!(act3_order.len(), 2);
        assert_ne!(act3_order[0], act3_order[1]);
        assert_eq!(run.state.map.as_ref().unwrap().boss_icon, Some(act3_order[0]));
        assert_eq!(run.state.floor, 50);
        let first = run.state.active_combat.as_ref().unwrap().encounter;
        assert_eq!(first, act3_order[0]);
        assert_eq!(run.state.combat.as_ref().unwrap().ascension(), 20);
        super::m28_world::win(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3BossComplete);
        let room = rooms::public(&run).unwrap();
        assert_eq!(room.choices.iter().map(|row| row.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Proceed]);
        let act3_frame = run.state.room_loop.as_ref().unwrap();
        assert!(act3_frame.active_rewards.is_none() && act3_frame.reward_parent.is_none() && act3_frame.boss.is_none());
        proceed(&mut run);
        assert_eq!(run.state.floor, 51);
        assert_eq!(run.state.map.as_ref().unwrap().current, Some(super::m33_double_boss::SECOND));
        assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, act3_order[1]);
        assert_eq!(run.state.combat.as_ref().unwrap().ascension(), 20);
        super::m28_world::win(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3BossComplete);
        proceed(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3Ending);

        drive_checked(&mut run, |candidate| candidate.state.act == 4 && candidate.state.active_combat.as_ref().is_some_and(|active| active.encounter == EncounterId::ShieldAndSpear));
        assert_eq!(run.state.floor, 54);
        assert_eq!(run.state.combat.as_ref().unwrap().ascension(), 20);
        super::m28_world::win(&mut run);
        drive_checked(&mut run, |candidate| candidate.state.act == 4 && candidate.state.active_combat.as_ref().is_some_and(|active| active.encounter == EncounterId::CorruptHeart));
        assert_eq!(run.state.floor, 55);
        assert_eq!(run.state.combat.as_ref().unwrap().ascension(), 20);
        super::m28_world::win(&mut run);
        assert_eq!(kind(&run), RoomViewKind::HeartVictory);
        assert_eq!(run.state.floor, 55);
        let heart_proceed = rooms::public(&run).unwrap().choices.iter().find(|row| row.kind == RoomChoiceKind::Proceed).unwrap().id;
        run.step(&GameAction::Choose(heart_proceed)).unwrap();
        assert_eq!(run.state.terminal, Some(RunTerminal::HeartVictory));
        assert_eq!(run.state.room, "TrueVictoryRoom");
        assert_eq!(run.state.floor, 56);
        assert_eq!(run.state.history.rooms.iter().filter(|entry| entry.act == 4).map(|entry| entry.floor).collect::<Vec<_>>(), vec![52, 53, 54, 55, 56]);
        assert_eq!(run.state.combat_progress.settled, run.state.combat_progress.entered);
        assert!(run.state.fault.is_none(), "{character:?}: {:?}", run.state.fault);
    }
}
