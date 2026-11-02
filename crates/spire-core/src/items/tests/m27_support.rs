//! M27 plan Task M2, lane Main, key m27_support.
//! Shared complete-item Run fixtures use a documented six-card starter-like deck and HP 80/80.

use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::run::{AuthoredRunStart, CardAdmission, CompleteCombatStart, ItemAdmission, Run,
    RunAdmission, RunError, RunRng};
use std::collections::BTreeMap;

fn starter_like_deck(character: Character) -> Vec<crate::run::CardInstance> {
    let (strike, defend, signature) = match character {
        Character::Ironclad => (CardId::Strike, CardId::Defend, CardId::Bash),
        Character::Silent => (CardId::StrikeGreen, CardId::DefendGreen, CardId::Neutralize),
        Character::Defect => (CardId::StrikeBlue, CardId::DefendBlue, CardId::Zap),
        Character::Watcher => (CardId::StrikePurple, CardId::DefendPurple, CardId::Eruption),
    };
    [strike, strike, strike, defend, defend, signature]
        .into_iter()
        .enumerate()
        .map(|(index, id)| crate::run::CardInstance {
            instance: index as u64 + 1,
            id,
            upgrades: 0,
            misc: 0,
        })
        .collect()
}

/// Build and enter a complete-admission A0 training combat through the shared Run owner.
pub(crate) fn complete_run(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>,
    counters: BTreeMap<RelicId, i32>, encounter: EncounterId) -> Run {
    let start = AuthoredRunStart {
        hp: 80,
        max_hp: 80,
        gold: 0,
        deck: starter_like_deck(character),
        relics,
        potions,
        relic_counters: counters,
    };
    let admission = RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete)
        .expect("the complete card/item admission pair is valid");
    let rng = RunRng::new([0xA6; 32], crate::combat::rng::Partition::Test, 0);
    let mut run = Run::prepare_complete_admission_combat(
        CompleteCombatStart::Supplied(start), character, admission, encounter, rng,
    ).unwrap_or_else(|error| panic!("complete Run admission: {error:?}"));
    run.enter_training_combat(encounter)
        .unwrap_or_else(|error| panic!("complete training combat entry: {error:?}"));
    run
}

/// Training action paths settle finished outcomes before returning; re-enter through the same
/// validated RunState with the authored-combat entry accepted by this training Run.
pub(crate) fn next_complete_combat(run: &mut Run, encounter: EncounterId) -> Result<(), RunError> {
    if run.state().combat.is_some() || run.state().active_combat.is_some() || run.state().pending.is_some()
        || run.state().combat_progress.entered == 0
        || run.state().combat_progress.settled != run.state().combat_progress.entered
        || run.state().last_combat.as_ref().is_none_or(|record|
            !matches!(record.outcome, crate::combat::Outcome::Victory | crate::combat::Outcome::Death
                | crate::combat::Outcome::Escaped))
    {
        return Err(RunError::InvalidAction);
    }
    run.enter_combat(encounter)?;
    Ok(())
}

#[test]
fn m27_support_pen_nib_counter_survives_settled_combat_without_an_attack() {
    let mut counters = BTreeMap::new();
    counters.insert(RelicId::PenNib, 4);
    let mut run = complete_run(Character::Ironclad, vec![RelicId::PenNib],
        vec![Some(PotionId::FirePotion); 3], counters, EncounterId::Cultist);

    // Three Fire Potions defeat the Cultist; no Attack card is played in the first combat.
    for slot in 0..3 {
        run.choose_combat(crate::combat::CombatAction::UsePotion {
            slot,
            target: Some(crate::combat::TargetId(1)),
        }).unwrap_or_else(|error| panic!("Fire Potion slot {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "the training Run path settled the victorious combat");
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome),
        Some(crate::combat::Outcome::Victory));
    assert_eq!(run.state().relic_counters.get(&RelicId::PenNib), Some(&4));

    next_complete_combat(&mut run, EncounterId::JawWorm).unwrap();
    assert_eq!(run.state().combat_progress.entered, 2);
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::PenNib), 4,
        "a settlement and second complete combat preserve Pen Nib's counter");
}

#[test]
fn m27_support_refuses_next_combat_while_current_combat_is_live() {
    let mut run = complete_run(Character::Ironclad, vec![RelicId::PenNib], vec![None; 3],
        BTreeMap::from([(RelicId::PenNib, 4)]), EncounterId::Cultist);
    let before = run.state().combat.as_ref().unwrap().snapshot();
    assert_eq!(next_complete_combat(&mut run, EncounterId::JawWorm), Err(RunError::InvalidAction));
    assert_eq!(run.state().combat.as_ref().unwrap().snapshot(), before);
    assert_eq!(run.state().combat_progress.entered, 1);
    assert_eq!(run.state().combat_progress.settled, 0);
}
