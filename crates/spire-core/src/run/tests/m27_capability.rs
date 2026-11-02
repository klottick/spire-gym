//! M27 Run item admission (plan Task M1, design §5.3).
use super::super::*;
use crate::content::{Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_capability::{inventory, CHARACTERS};

fn rng() -> RunRng { RunRng::new([7; 32], crate::combat::rng::Partition::Test, 0) }
fn supplied(relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> AuthoredRunStart {
    let deck = vec![CardInstance { instance: 1, id: crate::content::CardId::Strike, upgrades: 0, misc: 0 },
        CardInstance { instance: 2, id: crate::content::CardId::Defend, upgrades: 0, misc: 0 }];
    AuthoredRunStart { hp: 80, max_hp: 80, gold: 99, deck, relics, potions, relic_counters: Default::default() }
}

#[test]
fn m27_capability_run_admission() {
    let both = RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete).unwrap();
    for character in CHARACTERS {
        let (relics, potions) = inventory(character);
        let mut run = Run::prepare_complete_admission_combat(CompleteCombatStart::Supplied(supplied(relics.clone(), potions)),
            character, both, EncounterId::Cultist, rng()).unwrap_or_else(|e| panic!("{character:?}: {e:?}"));
        assert_eq!(run.state.item_admission, ItemAdmission::Complete);
        assert_eq!(run.state.relics, relics, "{character:?}: supplied order is acquisition order");
        assert!(format!("{:?}", run.state).ends_with(", card_admission: Complete, item_admission: Complete }"));
        run.enter_training_combat(EncounterId::Cultist).unwrap_or_else(|e| panic!("{character:?}: {e:?}"));
        let items = run.state.combat.as_ref().unwrap().item_context().unwrap();
        assert!(items.complete_cards() && items.complete_items(), "{character:?}");
    }
    // A complete-card Run keeps profile items: no flag, unchanged text.
    let mut run = Run::prepare_complete_combat(CompleteCombatStart::Supplied(supplied(vec![], vec![None; 3])),
        Character::Ironclad, EncounterId::Cultist, rng()).unwrap();
    assert_eq!(run.state.item_admission, ItemAdmission::Profile);
    assert!(format!("{:?}", run.state).ends_with(", card_admission: Complete }"));
    run.enter_training_combat(EncounterId::Cultist).unwrap();
    assert!(!run.state.combat.as_ref().unwrap().item_context().unwrap().complete_items());
    // Complete items with profile cards fail before any Run state exists.
    assert!(matches!(RunAdmission::new(CardAdmission::Profile, ItemAdmission::Complete), Err(RunError::Unsupported(_))));
    // The supplied start goes through the complete validator: a Silent start holding Burning Blood fails.
    let silent = CompleteCombatStart::Supplied(supplied(vec![RelicId::RingOfTheSnake, RelicId::BurningBlood], vec![None; 3]));
    assert_eq!(silent.normalized_for(Character::Silent, ItemAdmission::Complete), Err(RunError::InvalidAction));
    assert!(Run::prepare_complete_admission_combat(CompleteCombatStart::Supplied(supplied(vec![RelicId::RingOfTheSnake, RelicId::BurningBlood], vec![None; 3])),
        Character::Silent, both, EncounterId::Cultist, rng()).is_err());
}

/// Design §7.5: persistent counters and bottle links move into combat and back once;
/// combat-local state never reaches the Run; a second live owner is rejected as before.
#[test]
fn m27_counters_and_links_move_once() {
    let both = RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete).unwrap();
    let mut start = supplied(vec![RelicId::BurningBlood, RelicId::PenNib, RelicId::BottledFlame], vec![None; 3]);
    start.relic_counters.insert(RelicId::PenNib, 4);
    let mut run = Run::prepare_complete_admission_combat(CompleteCombatStart::Supplied(start), Character::Ironclad, both, EncounterId::Cultist, rng()).unwrap();
    run.state.relic_cards.insert(RelicId::BottledFlame, 1);
    let mut items = crate::items::CombatItemContext::take_from_run(&mut run.state);
    assert!(run.state.relics.is_empty() && run.state.relic_counters.is_empty() && run.state.relic_cards.is_empty());
    assert_eq!((items.counter(RelicId::PenNib), items.relic_card(RelicId::BottledFlame)), (4, Some(1)));
    items.set_local(RelicId::PenNib, 7);
    assert_eq!(items.local(RelicId::PenNib), 7);
    assert!(format!("{items:?}").contains("local: {PenNib: 7}"));
    let second = items.clone();
    items.return_to_run(&mut run.state).unwrap();
    assert_eq!(run.state.relic_counters.get(&RelicId::PenNib), Some(&4));
    assert_eq!(run.state.relic_cards.get(&RelicId::BottledFlame), Some(&1));
    assert!(!format!("{:?}", run.state).contains("local"), "combat-local state is dropped at settlement");
    assert_eq!(second.return_to_run(&mut run.state), Err(RunError::InvalidAction), "a second live owner is rejected");
    assert!(format!("{:?}", run.state).ends_with(", item_admission: Complete, relic_cards: {BottledFlame: 1} }"));
}
