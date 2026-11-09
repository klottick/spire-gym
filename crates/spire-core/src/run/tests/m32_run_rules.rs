//! M32 run rules (plan items 1 and 9): the Standard cap at A19 and the A17–A19 Standard route.
//! Literals come from the M32 design (§6.1) and the lane files, worked by hand, never from the code
//! under test.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{Character, EncounterId, EnemyId, EventId};
use crate::run::GameAction;
use rooms::standard::event_exec::Outcome as EventOutcome;
use rooms::standard::event_frame::{self, DONE};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn standard(character: Character, ascension: u8, full: bool) -> Result<Run, RunError> {
    Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(0x32), RoomScenario::Standard)
}

#[test]
fn m32_standard_creation_a0_to_a19_all_characters() {
    for character in CHARACTERS {
        for ascension in 0..=19 {
            for full in [false, true] {
                let run = standard(character, ascension, full).unwrap_or_else(|e| panic!("{character:?} A{ascension}: {e:?}"));
                assert_eq!(run.decision().unwrap().content_profile, Some(RoomScenario::Standard));
                assert_eq!(run.state.ascension, ascension);
                assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Neow, "{character:?} A{ascension}");
            }
        }
    }
}

/// Formerly `m32_standard_rejects_a20_and_above` (M33 plan item 1): the cap is A20 since M33, so the
/// rejected level is 21, with the A0–A20 message.
#[test]
fn m32_standard_rejects_above_cap() {
    for character in CHARACTERS {
        assert!(standard(character, 19, false).is_ok(), "{character:?} A19 (t-1)");
        for ascension in [21] {
            assert_eq!(standard(character, ascension, false).err(),
                Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A{ascension}");
            assert_eq!(rooms::standard::build(RunConfig { character, ascension }, profile(false), rng(1)).err(),
                Some(RunError::Unsupported("Standard supports A0–A20")), "{character:?} A{ascension}");
        }
    }
}

fn standard_on_map(character: Character, ascension: u8, seed: u8) -> Run {
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile(false), rng(seed), RoomScenario::Standard)
        .unwrap_or_else(|error| panic!("{character:?} A{ascension}: {error:?}"));
    let neow = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(neow)).unwrap();
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
    run
}

fn m32_event_run(character: Character, ascension: u8, act: u8, event: EventId, seed: u8) -> Run {
    let (mut run, node) = super::m28_event_core::before_event(character, act, event, seed);
    run.state.ascension = ascension;
    run.state.potions = vec![None; crate::run::potion_base_slots(ascension)];
    if event == EventId::DeadAdventurer {
        run.state.floor = 6;
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = 7;
    }
    run.step(&GameAction::Move(node)).unwrap_or_else(|error| panic!("{event:?} A{ascension}: {error:?}"));
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Event(event));
    run
}

fn m32_assert_run_fight(run: &Run, encounter: EncounterId, ascension: u8, context: &str) {
    assert!(run.state.fault.is_none(), "{context}: {:?}", run.state.fault);
    let combat = run.state.combat.as_ref().unwrap_or_else(|| panic!("{context}: combat is not active"));
    assert_eq!(combat.ascension(), ascension, "{context}: combat ascension");
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, encounter, "{context}: encounter");
    let public = combat.public_full().unwrap_or_else(|error| panic!("{context}: public view: {error:?}"));
    assert!(!public.enemies.is_empty(), "{context}: public enemies");
    assert!(public.enemies.iter().all(|enemy| !enemy.intent.as_str().is_empty()), "{context}: public intents");
}

fn m32_settle_run_fight(run: &mut Run, context: &str) {
    super::m28_world::win(run);
    assert!(run.state.fault.is_none(), "{context}: {:?}", run.state.fault);
    assert!(run.state.combat.is_none() && run.state.active_combat.is_none(), "{context}: fight settled");
    assert_eq!(run.state.last_combat.as_ref().map(|record| record.outcome), Some(crate::combat::Outcome::Victory), "{context}: victory record");
}

#[test]
fn m32_a17_first_act1_combat_public_behaviour() {
    // Seed 18 reaches Acid Slime S as the first Act 1 combat and opens with Tackle at A0. Its A17-only
    // row forces Lick without an AI draw, making the public A17 assertion distinguish the threshold.
    let seed = 18;
    let mut a0 = standard_on_map(Character::Ironclad, 0, seed);
    super::m28_world::drive(&mut a0, |candidate| candidate.state.combat.is_some());
    let a0_public = a0.state.combat.as_ref().unwrap().public_full().unwrap();
    let a0_slime = a0_public.enemies.iter().find(|enemy| enemy.enemy_id == EnemyId::AcidSlimeS).unwrap();
    assert_eq!(a0_slime.intent.as_str(), "acid_tackle", "A0 first combat seed {seed}");

    let mut a17 = standard_on_map(Character::Ironclad, 17, seed);
    super::m28_world::drive(&mut a17, |candidate| candidate.state.combat.is_some());
    let combat = a17.state.combat.as_ref().unwrap();
    let public = combat.public_full().unwrap();
    let slime = public.enemies.iter().find(|enemy| enemy.enemy_id == EnemyId::AcidSlimeS).unwrap();
    assert_eq!(slime.intent.as_str(), "acid_lick", "A17 first combat seed {seed}: Acid Slime S opening row");
    assert_eq!(combat.ascension(), 17);
}

#[test]
fn m32_event_fights_a17_to_a19() {
    for ascension in 17..=19u8 {
        // Open the real Colosseum event; its two combat stages enter and settle through their event route.
        let (mut colosseum, _) = super::m28_event_core::before_event(Character::Ironclad, 2, EventId::Colosseum, 0x61 + ascension);
        let (from, node) = {
            let map = colosseum.state.map.as_mut().unwrap();
            let from = map.nodes.iter().find(|node| node.y == 7 && !node.edges.is_empty()).unwrap().clone();
            let node = from.edges[0];
            map.nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().kind = MapNodeKind::Event;
            map.current = Some(from.id);
            (from, node)
        };
        colosseum.state.floor = from.floor;
        colosseum.state.ascension = ascension;
        colosseum.state.potions = vec![None; crate::run::potion_base_slots(ascension)];
        colosseum.step(&GameAction::Move(node)).unwrap_or_else(|error| panic!("Colosseum A{ascension}: {error:?}"));
        assert_eq!(rooms::public(&colosseum).unwrap().kind, RoomViewKind::Event(EventId::Colosseum));
        super::m28_events_act2::option_row(&mut colosseum, 0, 0);
        m32_assert_run_fight(&colosseum, EncounterId::ColosseumSlavers, ascension, "Colosseum Slavers");
        m32_settle_run_fight(&mut colosseum, "Colosseum Slavers");
        assert_eq!(rooms::public(&colosseum).unwrap().kind, RoomViewKind::Event(EventId::Colosseum), "A{ascension}: event reopens");
        super::m28_events_act2::option_row(&mut colosseum, 1, 1);
        m32_assert_run_fight(&colosseum, EncounterId::ColosseumNobs, ascension, "Colosseum Nobs");
        m32_settle_run_fight(&mut colosseum, "Colosseum Nobs");

        // Constructed outcomes exercise each Dead Adventurer elite and the Masked Bandits event fight.
        for (event, act, encounter, elite) in [
            (EventId::MaskedBandits, 2, EncounterId::MaskedBandits, false),
            (EventId::DeadAdventurer, 1, EncounterId::ThreeSentries, true),
            (EventId::DeadAdventurer, 1, EncounterId::GremlinNob, true),
            (EventId::DeadAdventurer, 1, EncounterId::LagavulinEvent, true),
        ] {
            let mut run = m32_event_run(Character::Silent, ascension, act, event, 0x70 + ascension);
            event_frame::choose_constructed(&mut run, vec![EventOutcome::Combat {
                encounter, rewards: None, next_stage: DONE, elite,
            }]).unwrap_or_else(|error| panic!("{event:?} {encounter:?} A{ascension}: {error:?}"));
            m32_assert_run_fight(&run, encounter, ascension, "constructed event fight");
            m32_settle_run_fight(&mut run, "constructed event fight");
        }

        // Mind Bloom's constructed Act 1 boss is Slime Boss so the event route also crosses its split threshold.
        let mut mind_bloom = m32_event_run(Character::Defect, ascension, 3, EventId::MindBloom, 0x80 + ascension);
        event_frame::choose_constructed(&mut mind_bloom, vec![EventOutcome::Combat {
            encounter: EncounterId::SlimeBoss, rewards: None, next_stage: DONE, elite: false,
        }]).unwrap_or_else(|error| panic!("Mind Bloom A{ascension}: {error:?}"));
        m32_assert_run_fight(&mind_bloom, EncounterId::SlimeBoss, ascension, "Mind Bloom Slime Boss");
        let before_split = mind_bloom.state.combat.as_ref().unwrap().public_full().unwrap();
        let slot = before_split.enemies.iter().position(|enemy| enemy.enemy_id == EnemyId::SlimeBoss).unwrap();
        let damage = before_split.enemies[slot].hp - (before_split.enemies[slot].max_hp / 2 - 1);
        super::m9_boss::hit(&mut mind_bloom, slot, damage);
        mind_bloom.choose_combat(crate::combat::CombatAction::EndTurn).unwrap();
        let split = mind_bloom.state.combat.as_ref().unwrap().public_full().unwrap();
        assert!(split.enemies.iter().any(|enemy| enemy.enemy_id != EnemyId::SlimeBoss), "A{ascension}: Mind Bloom Slime Boss split actors");
        m32_settle_run_fight(&mut mind_bloom, "Mind Bloom Slime Boss");
    }
}

#[test]
fn m32_bosses_and_act4_a17_to_a19() {
    for character in CHARACTERS {
        for ascension in 17..=19u8 {
            // Supplied frontier: 400/400 HP and all three keys, matching M29's constructed Act 4 route.
            // The route's public combats still enter and settle through Standard; lethal damage is supplied by win().
            let mut run = standard_on_map(character, ascension, 0x90 + ascension + character as u8);
            run.state.max_hp = 400;
            run.state.hp = 400;
            run.state.keys = [true; 3];

            super::m28_world::drive(&mut run, |candidate| candidate.state.act == 2
                && candidate.state.combat.is_some()
                && candidate.state.active_combat.as_ref().is_some_and(|active| matches!(active.origin, CombatOrigin::MapRoom { kind: MapNodeKind::Boss, .. })));
            let act2_boss = run.state.active_combat.as_ref().expect("Act 2 boss entered").encounter;
            assert!(matches!(act2_boss, EncounterId::Champ | EncounterId::Collector | EncounterId::BronzeAutomaton), "{character:?} A{ascension}: {act2_boss:?}");
            m32_assert_run_fight(&run, act2_boss, ascension, "Act 2 boss");
            m32_settle_run_fight(&mut run, "Act 2 boss");

            super::m28_world::drive(&mut run, |candidate| candidate.state.act == 3
                && candidate.state.combat.is_some()
                && candidate.state.active_combat.as_ref().is_some_and(|active| matches!(active.origin, CombatOrigin::MapRoom { kind: MapNodeKind::Boss, .. })));
            let act3_boss = run.state.active_combat.as_ref().expect("Act 3 boss entered").encounter;
            assert!(matches!(act3_boss, EncounterId::AwakenedOne | EncounterId::TimeEater | EncounterId::DonuAndDeca), "{character:?} A{ascension}: {act3_boss:?}");
            m32_assert_run_fight(&run, act3_boss, ascension, "Act 3 boss");
            m32_settle_run_fight(&mut run, "Act 3 boss");

            super::m28_world::drive(&mut run, |candidate| candidate.state.act == 4
                && candidate.state.combat.is_some()
                && candidate.state.active_combat.as_ref().is_some_and(|active| active.encounter == EncounterId::ShieldAndSpear));
            m32_assert_run_fight(&run, EncounterId::ShieldAndSpear, ascension, "Shield and Spear");
            m32_settle_run_fight(&mut run, "Shield and Spear");

            super::m28_world::drive(&mut run, |candidate| candidate.state.act == 4
                && candidate.state.combat.is_some()
                && candidate.state.active_combat.as_ref().is_some_and(|active| active.encounter == EncounterId::CorruptHeart));
            m32_assert_run_fight(&run, EncounterId::CorruptHeart, ascension, "Heart");
            m32_settle_run_fight(&mut run, "Heart");
            assert_eq!(run.state.combat_progress.settled, run.state.combat_progress.entered, "{character:?} A{ascension}: every entered combat settled");
        }
    }
}
