//! M27 plan Task M5, lane Main, key m27_potion_owner.
//! Complete-item use, discard, and held-potency checks through the single Run owner.

use super::m27_support;
use crate::combat::{CombatAction, TargetId};
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::items::CombatItemContext;
use crate::run::{Run, RunRng};
use std::collections::BTreeMap;

fn enemy_hp(run: &Run) -> i32 {
    run.state().combat.as_ref().unwrap().public_full().unwrap().enemies[0].hp as i32
}

#[test]
fn m27_potion_owner_use_law_all_characters() {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    for character in characters {
        for (bark, expected) in [(false, 20), (true, 40)] {
            let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
            let mut run = m27_support::complete_run(character, relics,
                vec![Some(PotionId::FirePotion), None, None], BTreeMap::new(), EncounterId::JawWorm);
            let before = enemy_hp(&run);
            run.choose_combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) })
                .unwrap_or_else(|error| panic!("Fire Potion {character:?} Bark={bark}: {error:?}"));
            assert_eq!(before - enemy_hp(&run), expected, "Fire Potion {character:?} Bark={bark}");
        }
    }

    for character in [Character::Ironclad, Character::Watcher] {
        for potion in [PotionId::DistilledChaos, PotionId::LiquidMemories] {
            let mut run = m27_support::complete_run(character, vec![],
                vec![Some(potion), None, None], BTreeMap::new(), EncounterId::JawWorm);
            run.choose_combat(CombatAction::UsePotion { slot: 0, target: None })
                .unwrap_or_else(|error| panic!("{potion:?} for {character:?}: {error:?}"));
            assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().potions()[0], None,
                "{potion:?} is consumed for {character:?}");
        }
        let mut smoke = m27_support::complete_run(character, vec![],
            vec![Some(PotionId::SmokeBomb), None, None], BTreeMap::new(), EncounterId::JawWorm);
        smoke.choose_combat(CombatAction::UsePotion { slot: 0, target: None })
            .unwrap_or_else(|error| panic!("Smoke Bomb for {character:?}: {error:?}"));
        assert_eq!(smoke.state().last_combat.as_ref().unwrap().outcome, crate::combat::Outcome::Escaped);
    }
}

#[test]
fn m27_potion_owner_discard_combat() {
    let mut run = m27_support::complete_run(Character::Ironclad, vec![RelicId::ToyOrnithopter],
        vec![Some(PotionId::BlockPotion), None, None], BTreeMap::new(), EncounterId::JawWorm);
    run.state_mut_for_test().hp = 50;
    let combat = run.state_mut_for_test().combat.as_mut().unwrap();
    crate::combat::tests::m27_state::set_player_hp(combat, 50);
    let discard = CombatAction::DiscardPotion { slot: 0 };
    assert!(combat.legal_actions_full().unwrap().contains(&discard), "occupied complete slot is discardable");
    run.choose_combat(discard).expect("complete DiscardPotion action");
    assert_eq!(run.state().hp, 50, "discard has no Toy Ornithopter PotionUsed heal");
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().potions()[0], None);

    let mut frozen = frozen_m15_combat();
    let before = frozen.snapshot();
    assert!(!frozen.legal_actions_full().unwrap().iter().any(|action|
        matches!(action, CombatAction::DiscardPotion { .. })), "frozen M15 action mask has no discard verb");
    assert!(frozen.step_full(&CombatAction::DiscardPotion { slot: 0 }).is_err());
    assert_eq!(frozen.snapshot(), before, "frozen action rejection is observationally inert");
}

#[test]
fn m27_potion_owner_potency_survives_bark_loss_in_combat() {
    let mut run = m27_support::complete_run(Character::Ironclad, vec![RelicId::SacredBark],
        vec![Some(PotionId::BlockPotion), None, None], BTreeMap::new(), EncounterId::JawWorm);
    let combat = run.state_mut_for_test().combat.as_mut().unwrap();
    let mut items = combat.item_context().unwrap().clone();
    assert_eq!(items.potion_potency(0), Some(24), "Bark doubles the held Block Potion at combat entry");
    items.relics.retain(|id| *id != RelicId::SacredBark);
    crate::combat::tests::m27_state::set_item_context(combat, Some(items));
    run.choose_combat(CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(run.state().combat.as_ref().unwrap().public_full().unwrap().player.block, 24,
        "losing Bark does not reduce the already-held potion potency");
}

fn frozen_m15_combat() -> crate::combat::Combat {
    use crate::combat::{self, CardInstanceId, PlayerState};
    let entry = combat::enemies::setup(EncounterId::JawWorm, 1, 0,
        &mut RunRng::new([0xA6; 32], combat::rng::Partition::Test, 0),
        combat::enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3,
                draw_per_turn: 1, powers: vec![] },
            player_powers: vec![], character: Character::Ironclad,
            cards: vec![combat::permanent_live_card(CardId::Strike, CardInstanceId(1), 0)],
            opening_draw: 1, encounter_visit: 0,
        }).unwrap();
    let context = CombatItemContext::new(vec![], vec![Some(PotionId::BlockPotion)]).with_m15_profile();
    combat::Combat::enter_with_item_context(entry, &mut Some(context)).unwrap()
}
