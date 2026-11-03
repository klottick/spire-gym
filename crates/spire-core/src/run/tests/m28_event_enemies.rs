//! M28e event-owned bandit combat and Colosseum formations.
use super::super::*;
use super::m28_event_core::{at_event, rng, view};
use crate::combat::{CombatAction, CombatPublic, FullPowerView};
use crate::content::{Character, EncounterId, EnemyId, EventId};
use crate::run::{CreationProfile, MapNodeKind, RoomChoiceKind, RoomScenario, Run, RunConfig, RunError};
use rooms::standard::event_exec::Outcome;
use rooms::standard::event_frame::{self, DONE};

fn fight(encounter: EncounterId, seed: u8) -> Run {
    let mut run = at_event(Character::Ironclad, 2, EventId::GoldenShrine, seed);
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat {
        encounter, rewards: None, next_stage: DONE, elite: false,
    }]).unwrap();
    run
}

fn long_fight(encounter: EncounterId, seed: u8) -> Run {
    let mut run = at_event(Character::Ironclad, 2, EventId::GoldenShrine, seed);
    run.state.max_hp = 500;
    run.state.hp = 500;
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat {
        encounter, rewards: None, next_stage: DONE, elite: false,
    }]).unwrap();
    run
}

fn combat(run: &Run) -> CombatPublic {
    run.state.combat.as_ref().expect("event combat is active").public_full().unwrap()
}

fn power(powers: &[FullPowerView], id: &str) -> i32 {
    powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.amount)
}

fn end_turn(run: &mut Run) {
    run.choose_combat(CombatAction::EndTurn).unwrap();
}

#[test]
fn m28_event_enemies_hp_and_openers() {
    for seed in 0..8 {
        let run = fight(EncounterId::MaskedBandits, seed);
        let enemies = combat(&run).enemies;
        assert_eq!(enemies.len(), 3);
        assert_eq!((enemies[0].enemy_id, enemies[0].intent.as_str()), (EnemyId::BanditPointy, "pointy_special"));
        assert_eq!((enemies[1].enemy_id, enemies[1].intent.as_str()), (EnemyId::BanditLeader, "romeo_mock"));
        assert_eq!((enemies[2].enemy_id, enemies[2].intent.as_str()), (EnemyId::BanditBear, "bear_hug"));
        assert_eq!(enemies[0].hp, 30, "Pointy A0 HP is fixed");
        assert!((35..=39).contains(&enemies[1].hp), "Romeo: {:?}", enemies[1]);
        assert!((38..=42).contains(&enemies[2].hp), "Bear: {:?}", enemies[2]);
        assert!(enemies.iter().all(|enemy| enemy.block == 0 && enemy.powers.is_empty()), "{enemies:?}");
    }
}

#[test]
fn m28_event_enemies_move_effects() {
    let mut run = long_fight(EncounterId::MaskedBandits, 91);
    let opening = combat(&run);
    assert_eq!(opening.enemies[0].intent_damage, Some(5), "Pointy's intent damage is per hit");
    let hp = opening.player.hp;
    end_turn(&mut run);
    let first = combat(&run);
    assert_eq!(hp - first.player.hp, 10, "Pointy lands exactly two hits of 5; Mock and Hug do no damage");
    assert_eq!(power(&first.player.powers, "weak"), 0);
    assert_eq!(power(&first.player.powers, "dexterity"), -2, "Bear Hug applies Dexterity −2");
    assert_eq!(first.enemies[2].block, 0);

    let hp = first.player.hp;
    end_turn(&mut run);
    let second = combat(&run);
    assert_eq!(hp - second.player.hp, 29, "Pointy 10 + Agonizing Slash 10 + Lunge 9");
    assert_eq!(power(&second.player.powers, "weak"), 2);
    assert_eq!(second.enemies[2].block, 9, "Lunge grants Bear 9 Block");

    let hp = second.player.hp;
    end_turn(&mut run);
    let third = combat(&run);
    assert_eq!(hp - third.player.hp, 43, "Pointy 10 + Cross Slash 15 + Maul 18");
}

#[test]
fn m28_event_enemies_patterns() {
    let mut run = long_fight(EncounterId::MaskedBandits, 92);
    let expected = [
        ("pointy_special", "romeo_mock", "bear_hug"),
        ("pointy_special", "romeo_agonizing_slash", "bear_lunge"),
        ("pointy_special", "romeo_cross_slash", "bear_maul"),
        ("pointy_special", "romeo_agonizing_slash", "bear_lunge"),
        ("pointy_special", "romeo_cross_slash", "bear_maul"),
        ("pointy_special", "romeo_agonizing_slash", "bear_lunge"),
    ];
    for (turn, expected) in expected.into_iter().enumerate() {
        let enemies = combat(&run).enemies;
        assert_eq!(
            [enemies[0].intent.as_str(), enemies[1].intent.as_str(), enemies[2].intent.as_str()],
            [expected.0, expected.1, expected.2],
            "turn {turn}",
        );
        if turn < 5 { end_turn(&mut run); }
    }
}

#[test]
fn m28_event_formations_colosseum_and_bandits() {
    let cases = [
        (EncounterId::ColosseumSlavers, vec![EnemyId::SlaverBlue, EnemyId::SlaverRed]),
        (EncounterId::ColosseumNobs, vec![EnemyId::Taskmaster, EnemyId::GremlinNob]),
        (EncounterId::MaskedBandits, vec![EnemyId::BanditPointy, EnemyId::BanditLeader, EnemyId::BanditBear]),
    ];
    for (index, (encounter, expected)) in cases.into_iter().enumerate() {
        let enemies = combat(&fight(encounter, 101 + index as u8)).enemies;
        assert_eq!(enemies.iter().map(|enemy| enemy.enemy_id).collect::<Vec<_>>(), expected);
        assert_eq!(
            enemies.iter().map(|enemy| enemy.logical_slot).collect::<Vec<_>>(),
            (0..expected.len() as u8).collect::<Vec<_>>(),
        );
        let ids: std::collections::BTreeSet<_> = enemies.iter().map(|enemy| enemy.instance_id.clone()).collect();
        assert_eq!(ids.len(), enemies.len(), "each formation slot has a distinct actor id");
        if encounter == EncounterId::ColosseumNobs {
            assert!((54..=60).contains(&enemies[0].hp), "Taskmaster A0 HP: {:?}", enemies[0]);
            assert!((82..=86).contains(&enemies[1].hp), "Gremlin Nob A0 HP: {:?}", enemies[1]);
        }
    }
}

/// M29d: every species landed, so the witness moves above Masked Bandits' encounter ceiling (16,
/// Romeo's A17 AI is M32): A17 is the typed `Unsupported` (name kept, plan m4).
#[test]
fn m28_event_enemies_a0_only() {
    let mut run = at_event(Character::Ironclad, 2, EventId::GoldenShrine, 93);
    run.state.ascension = 17;
    let before = run.clone();
    let result = event_frame::choose_constructed(&mut run, vec![Outcome::Combat {
        encounter: EncounterId::MaskedBandits, rewards: None, next_stage: DONE, elite: false,
    }]);
    assert!(matches!(result, Err(RunError::Unsupported(_))), "{result:?}");
    assert_eq!(run, before, "unsupported A17 setup leaves the event Run unchanged");
}

#[test]
fn m28_frozen_profiles_never_reach_standard_owners_e() {
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
        for encounter in [EncounterId::ColosseumSlavers, EncounterId::ColosseumNobs, EncounterId::MaskedBandits] {
            for origin in [
                CombatOrigin::MapRoom { node, kind: MapNodeKind::Monster },
                CombatOrigin::Event { node, event: EventId::GoldenShrine, stage: 0 },
            ] {
                let before = run.clone();
                assert_eq!(
                    run.enter_owned(encounter, origin),
                    Err(RunError::Unsupported("encounter outside authored M3 profile")),
                    "{scenario:?} {encounter:?} {origin:?}",
                );
                assert_eq!(run, before, "{scenario:?} {encounter:?}");
            }
        }
    }
}
