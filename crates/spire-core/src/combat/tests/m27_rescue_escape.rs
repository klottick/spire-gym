//! M27 plan Task M5, lane Main, key m27_rescue_escape.
//! Complete-item combat tests for rescue order, Smoke Bomb restrictions and escape settlement.

use super::super::{Actor, CombatAction, DamagePayload, Effect, EffectKind, Source};
use crate::content::{Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_support;
use crate::run::{Run, RunError};
use std::collections::BTreeMap;

fn lethal(run: &mut Run) {
    let combat = run.state_mut_for_test().combat.as_mut().expect("live complete combat");
    super::m27_state::set_player_hp(combat, 1);
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player),
        target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::HpLoss(1)),
    }], None).expect("accepted lethal damage");
}

fn hp(run: &Run) -> i32 {
    run.state().combat.as_ref().expect("live combat").public_full().unwrap().player.hp as i32
}

#[test]
fn m27_rescue_order_fairy_then_lizard_tail_once() {
    let mut run = m27_support::complete_run(Character::Ironclad,
        vec![RelicId::LizardTail, RelicId::ToyOrnithopter],
        vec![Some(PotionId::FairyPotion), Some(PotionId::FairyPotion), None],
        BTreeMap::new(), EncounterId::JawWorm);

    lethal(&mut run);
    assert_eq!(hp(&run), 24, "first Fairy rescues for 30% of 80 max HP");
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().potions(),
        &[None, Some(PotionId::FairyPotion), None]);

    lethal(&mut run);
    assert_eq!(hp(&run), 24, "second Fairy rescues before Lizard Tail");
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().potions(), &[None; 3]);

    lethal(&mut run);
    assert_eq!(hp(&run), 40, "unused Lizard Tail restores 50% of 80 max HP");
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::LizardTail), 1);

    lethal(&mut run);
    assert_eq!(run.state().combat.as_ref().unwrap().public_full().unwrap().outcome,
        Some(crate::combat::Outcome::Death), "Lizard Tail is consumed once");

    let mut bark = m27_support::complete_run(Character::Ironclad, vec![RelicId::LizardTail, RelicId::SacredBark],
        vec![Some(PotionId::FairyPotion), None, None], BTreeMap::new(), EncounterId::JawWorm);
    lethal(&mut bark);
    assert_eq!(hp(&bark), 48, "Sacred Bark doubles held Fairy rescue potency");

    let mut bloom = m27_support::complete_run(Character::Ironclad,
        vec![RelicId::LizardTail, RelicId::MarkOfBloom],
        vec![Some(PotionId::FairyPotion), None, None], BTreeMap::new(), EncounterId::JawWorm);
    lethal(&mut bloom);
    assert_eq!(bloom.state().combat.as_ref().unwrap().public_full().unwrap().outcome,
        Some(crate::combat::Outcome::Death), "Mark of the Bloom blocks both rescue laws");
}

#[test]
fn m27_smoke_bomb_rules_and_settlement() {
    let bosses = [EncounterId::SlimeBoss, EncounterId::Guardian, EncounterId::Hexaghost,
        EncounterId::Champ, EncounterId::Collector, EncounterId::BronzeAutomaton,
        EncounterId::AwakenedOne, EncounterId::TimeEater, EncounterId::DonuAndDeca,
        EncounterId::CorruptHeart];
    for encounter in bosses {
        let mut run = m27_support::complete_run(Character::Ironclad, vec![],
            vec![Some(PotionId::SmokeBomb), None, None], BTreeMap::new(), encounter);
        let combat = run.state().combat.as_ref().unwrap();
        let before = combat.snapshot();
        assert!(matches!(run.choose_combat(CombatAction::UsePotion { slot: 0, target: None }),
            Err(RunError::InvalidAction)), "Smoke Bomb is blocked in boss encounter {encounter:?}");
        assert_eq!(run.state().combat.as_ref().unwrap().snapshot(), before,
            "rejected boss escape leaves the combat unchanged: {encounter:?}");
    }

    let mut back_attack = m27_support::complete_run(Character::Ironclad, vec![],
        vec![Some(PotionId::SmokeBomb), None, None], BTreeMap::new(), EncounterId::ShieldAndSpear);
    let frozen = back_attack.state().combat.as_ref().unwrap().snapshot();
    assert!(matches!(back_attack.choose_combat(CombatAction::UsePotion { slot: 0, target: None }), Err(RunError::InvalidAction)));
    assert_eq!(back_attack.state().combat.as_ref().unwrap().snapshot(), frozen,
        "Act 4 Shield and Spear is an Elite blocked by Back Attack");

    let mut counters = BTreeMap::new();
    counters.insert(RelicId::PenNib, 9);
    let mut run = m27_support::complete_run(Character::Ironclad,
        vec![RelicId::BurningBlood, RelicId::PenNib, RelicId::Kunai],
        vec![Some(PotionId::SmokeBomb), None, None], counters, EncounterId::JawWorm);
    {
        let combat = run.state_mut_for_test().combat.as_mut().unwrap();
        super::m27_state::set_player_hp(combat, 40);
        let mut items = combat.item_context().unwrap().clone();
        items.set_local(RelicId::Kunai, 2);
        super::m27_state::set_item_context(combat, Some(items));
    }
    let transition = run.choose_combat(CombatAction::UsePotion { slot: 0, target: None })
        .expect("Smoke Bomb escapes an ordinary Jaw Worm encounter");
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome),
        Some(crate::combat::Outcome::Escaped));
    assert_eq!(run.state().hp, 40, "Burning Blood does not heal on Escaped (G9/F3)");
    assert_eq!(transition.reward, 0);
    assert_eq!(transition.committed_events.len(), 1, "escape commits only CombatFinished, with no reward rows");
    assert!(run.state().combat.is_none(), "escape settles the training room");
    assert_eq!(run.state().combat_progress.entered, run.state().combat_progress.settled);
    assert_eq!(run.state().potions, vec![None; 3], "the consumed slot returns once");
    assert_eq!(run.state().relic_counters.get(&RelicId::PenNib), Some(&9), "persistent counter returns");
    m27_support::next_complete_combat(&mut run, EncounterId::JawWorm).unwrap();
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().local(RelicId::Kunai), 0,
        "combat-local Kunai state is dropped at settlement");
}

#[test]
fn m27_g9_escape_skips_victory_relics() {
    for relics in [vec![RelicId::BurningBlood], vec![RelicId::BlackBlood],
        vec![RelicId::MeatOnTheBone], vec![RelicId::FaceOfCleric]] {
        let label = format!("{relics:?}");
        let mut run = m27_support::complete_run(Character::Ironclad, relics,
            vec![Some(PotionId::SmokeBomb), None, None], BTreeMap::new(), EncounterId::JawWorm);
        super::m27_state::set_player_hp(run.state_mut_for_test().combat.as_mut().unwrap(), 40);
        run.choose_combat(CombatAction::UsePotion { slot: 0, target: None }).unwrap();
        assert_eq!(run.state().hp, 40, "{label} does not run its Victory effect on Escaped");
        assert_eq!(run.state().max_hp, 80, "Face of Cleric does not grant Max HP on Escaped");
    }
}
