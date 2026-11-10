//! M34s-a a4: the Gym66 Escaped law (design §4.4): `spire-gym-combat-outcome-reward-v1` gives 0 to an escape, at the
//! step reward and the M21 replay validator (one shared law), and no Gym66 catalog can reach an escape.
use spire_core::{combat::Outcome, content::PotionId, run::{M21_WATCHER_POTIONS, TRAINING_POTIONS}};

#[test]
fn gym66_escaped_law_is_zero_and_unreachable() {
    let law = crate::combat_training::outcome_reward;
    assert_eq!([law(Outcome::Victory), law(Outcome::Death), law(Outcome::Escaped)], [1, -1, 0]);
    // Smoke Bomb is the only source of `Outcome::Escaped` (thief escapes end as Victory); no Gym66 catalog admits it.
    assert!(!TRAINING_POTIONS.contains(&PotionId::SmokeBomb) && !M21_WATCHER_POTIONS.contains(&PotionId::SmokeBomb));
    for support in [crate::combat_training_support(), crate::combat_training_support_v2()] {
        assert!(support["potions"].as_array().unwrap().iter().all(|p| p != "smoke_bomb"));
    }
}
