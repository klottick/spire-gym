//! M27 plan Task M5, lane Main, key m27_energy_law.
//! Expected energy, Block, and Strength values are literals from Review Focus 4,
//! the M27 approved design §12 G1, and Task M5.

use crate::combat::{Actor, CombatAction, Effect, EffectKind, FullPowerId, Source};
use crate::content::{CardId, Character, EncounterId, RelicId};
use crate::items::{laws, CombatItemContext};
use crate::run::{AuthoredRunStart, CardAdmission, CardInstance, CompleteCombatStart, ItemAdmission, Run, RunAdmission, RunRng};
use super::m27_support;
use std::collections::BTreeMap;

fn complete(character: Character, relics: Vec<RelicId>, encounter: EncounterId) -> Run {
    m27_support::complete_run(character, relics, vec![None; 3], BTreeMap::new(), encounter)
}

fn view(run: &Run) -> crate::combat::CombatPublic {
    run.state().combat.as_ref().expect("live complete combat").public_full().unwrap()
}

fn energy(run: &Run) -> (i32, i32) {
    let player = view(run).player;
    (player.energy, player.max_energy)
}

fn spend_defend(run: &mut Run, character: Character) {
    let id = match character {
        Character::Ironclad => CardId::Defend,
        Character::Silent => CardId::DefendGreen,
        Character::Defect => CardId::DefendBlue,
        Character::Watcher => CardId::DefendPurple,
    };
    let instance = view(run).hand.iter().find(|card| card.id == id)
        .unwrap_or_else(|| panic!("starter Defend {id:?} is absent from {:?}", view(run).hand))
        .instance.0;
    run.choose_combat(CombatAction::PlayCard { instance, target: None })
        .unwrap_or_else(|error| panic!("play {id:?}: {error:?}"));
}

fn complete_items(relics: Vec<RelicId>) -> CombatItemContext {
    CombatItemContext::new(relics, vec![]).with_complete_cards().with_complete_items()
}

fn watcher_fasting_run() -> Run {
    let deck=(1..=5).map(|instance|CardInstance{instance,id:CardId::Fasting,upgrades:1,misc:0}).collect();
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:0,deck,relics:vec![RelicId::IceCream,RelicId::Sozu],
        potions:vec![None;3],relic_counters:BTreeMap::new()};
    let admission=RunAdmission::new(CardAdmission::Complete,ItemAdmission::Complete).unwrap();
    let rng=RunRng::new([0xA6;32],crate::combat::rng::Partition::Test,0);
    let mut run=Run::prepare_complete_admission_combat(CompleteCombatStart::Supplied(start),
        Character::Watcher,admission,EncounterId::Cultist,rng).unwrap();
    run.enter_training_combat(EncounterId::Cultist).unwrap();
    run
}

#[test]
fn m27_g1_sozu_energy_was_missing() {
    let run = complete(Character::Ironclad, vec![RelicId::Sozu], EncounterId::Cultist);
    assert_eq!(energy(&run), (4, 4), "Sozu grants +1 master energy under the global G1 correction");
}

#[test]
fn m27_energy_law_all_characters() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let sozu = complete(character, vec![RelicId::Sozu], EncounterId::Cultist);
        assert_eq!(energy(&sozu), (4, 4), "Sozu / {character:?}");

        let paired = complete(character, vec![RelicId::Sozu, RelicId::BustedCrown], EncounterId::Cultist);
        assert_eq!(energy(&paired), (5, 5), "Sozu + Busted Crown / {character:?}");

        let dome = complete(character, vec![RelicId::RunicDome], EncounterId::Cultist);
        assert_eq!(energy(&dome), (4, 4), "Runic Dome / {character:?}");
        let choker = complete(character, vec![RelicId::VelvetChoker], EncounterId::Cultist);
        assert_eq!(energy(&choker), (4, 4), "Velvet Choker / {character:?}");

        let mut ice_cream = complete(character, vec![RelicId::IceCream], EncounterId::Lagavulin);
        spend_defend(&mut ice_cream, character);
        assert_eq!(energy(&ice_cream), (2, 3), "one starter Defend leaves 2 energy / {character:?}");
        ice_cream.choose_combat(CombatAction::EndTurn).unwrap();
        assert_eq!(energy(&ice_cream), (5, 3), "Ice Cream carries 2 into the next 3-energy turn / {character:?}");
    }

    let mut cap = complete(Character::Ironclad, vec![RelicId::IceCream], EncounterId::Lagavulin);
    crate::combat::tests::m27_state::set_player_energy(
        cap.state_mut_for_test().combat.as_mut().expect("live complete combat"), 999);
    cap.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(energy(&cap), (999, 999), "Ice Cream recharge respects the native 999-energy cap");

    let mut fasting = watcher_fasting_run();
    let instance = view(&fasting).hand.iter().find(|card| card.id == CardId::Fasting)
        .expect("upgraded Fasting is in the opening hand").instance.0;
    fasting.choose_combat(CombatAction::PlayCard { instance, target: None }).unwrap();
    assert_eq!(energy(&fasting), (2, 4), "upgraded Fasting costs 2 of 4 energy");
    fasting.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(energy(&fasting), (5, 4), "Watcher Fasting subtracts Energy Down after Ice Cream carries 2");
}

#[test]
fn m27_master_energy_all_boss_relics() {
    let accepted = CombatItemContext::new(vec![RelicId::Sozu], vec![]);
    assert_eq!(laws::master_energy(&accepted, false), 1, "Sozu's G1 correction is global");
    let accepted_mark = CombatItemContext::new(vec![RelicId::MarkOfPain], vec![]);
    assert_eq!(laws::master_energy(&accepted_mark, false), 1, "Mark of Pain keeps its accepted +1");

    let collar = CombatItemContext::new(vec![RelicId::SlaversCollar], vec![]);
    assert_eq!(laws::master_energy(&collar, false), 0, "Slaver's Collar is inactive outside elite/boss combat");
    assert_eq!(laws::master_energy(&collar, true), 1, "Slaver's Collar adds +1 in elite/boss combat");

    for relic in [RelicId::BustedCrown, RelicId::CoffeeDripper, RelicId::CursedKey,
        RelicId::Ectoplasm, RelicId::FusionHammer, RelicId::PhilosophersStone,
        RelicId::RunicDome, RelicId::VelvetChoker] {
        let complete = complete_items(vec![relic]);
        assert_eq!(laws::master_energy(&complete, false), 1, "{relic:?} adds +1 in complete admission");
        let frozen = CombatItemContext::new(vec![relic], vec![]);
        assert_eq!(laws::master_energy(&frozen, false), 0, "{relic:?} stays inert in frozen admission");
    }
}

fn next_turn_block(block: i32, barricade: bool) -> i32 {
    let mut run = complete(Character::Ironclad, vec![RelicId::Calipers], EncounterId::Lagavulin);
    crate::combat::tests::m27_state::set_player_block(
        run.state_mut_for_test().combat.as_mut().expect("live complete combat"), block);
    if barricade {
        run.state_mut_for_test().combat.as_mut().expect("live complete combat")
            .enqueue_native_effects(vec![Effect {
                source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("barricade"), amount: 1 },
            }], None).unwrap();
    }
    run.choose_combat(CombatAction::EndTurn).unwrap();
    view(&run).player.block
}

#[test]
fn m27_calipers_keeps_fifteen_less() {
    assert_eq!(next_turn_block(20, false), 5, "Calipers preserves current Block minus 15");
    assert_eq!(next_turn_block(10, false), 0, "Calipers floors preserved Block at 0");
    assert_eq!(next_turn_block(20, true), 20, "Barricade keeps all 20 Block through Calipers");
}

#[test]
fn m27_philosophers_stone_strength_and_energy() {
    let mut run = complete(Character::Ironclad, vec![RelicId::PhilosophersStone], EncounterId::Collector);
    assert_eq!(energy(&run), (4, 4), "Philosopher's Stone adds +1 master energy");
    let initial = view(&run);
    assert!(!initial.enemies.is_empty());

    run.choose_combat(CombatAction::EndTurn).unwrap();
    let after_spawn = view(&run);
    let spawned = after_spawn.enemies.iter().filter(|enemy| enemy.enemy_id == crate::content::EnemyId::TorchHead).collect::<Vec<_>>();
    assert_eq!(spawned.len(), 2, "Collector's opening move spawns two Torch Heads");
    for enemy in spawned {
        let strength = enemy.powers.iter().find(|power| power.id.0 == "strength").map_or(0, |power| power.amount);
        assert_eq!(strength, 1, "Philosopher's Stone gives spawned {:?} +1 Strength", enemy.enemy_id);
    }
    for enemy in &initial.enemies {
        let strength = enemy.powers.iter().find(|power| power.id.0 == "strength").map_or(0, |power| power.amount);
        assert_eq!(strength, 1, "Philosopher's Stone gives initial {:?} +1 Strength", enemy.enemy_id);
    }
}
