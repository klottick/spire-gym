//! M27 plan Task M3, lane Main, key m27_inert.
//! Empty handoff scaffold for inert and unhandled item checks owned by Main.

use crate::combat::{Combat, CombatAction, CombatEntry, CardInstanceId};
use crate::content::{CardId, Character, RelicId};
use crate::items::CombatItemContext;

fn own_colour_cards(character: Character) -> [CardId; 5] {
    use CardId::*;
    match character {
        Character::Ironclad => [Flex, PommelStrike, Inflame, Armaments, ShrugItOff],
        Character::Silent => [Backflip, DeadlyPoison, Footwork, Acrobatics, BladeDance],
        Character::Defect => [BallLightning, Coolheaded, Defragment, ChargeBattery, Loop],
        Character::Watcher => [FlurryOfBlows, Worship, Protect, Foresight, MentalFortress],
    }
}

fn heart_items(character: Character) -> Vec<RelicId> {
    use RelicId::*;
    match character {
        Character::Ironclad => vec![BurningBlood, Vajra, BagOfPreparation, BloodVial],
        Character::Silent => vec![RingOfTheSnake, NinjaScroll, SneckoSkull, Tingsha],
        Character::Defect => vec![CrackedCore, DataDisk, GoldPlatedCables, EmotionChip],
        Character::Watcher => vec![PureWater, Damaru, GoldenEye, TeardropLocket],
    }
}

fn inert_combat(character: Character, relics: Vec<RelicId>, complete_items: bool) -> Combat {
    let mut entry: CombatEntry = super::full_entry(character, 5, 16);
    let catalog = crate::content::Catalog::standard();
    let mut cards = catalog.starter_deck(character).unwrap().to_vec();
    cards.extend(own_colour_cards(character));
    entry.cards = cards.into_iter().enumerate().map(|(index, id)|
        crate::combat::complete_live_card(id, CardInstanceId(index as u64 + 1), 0)
            .unwrap_or_else(|error| panic!("{character:?} {id:?} live card: {error:?}")))
        .collect();
    let context = crate::run::complete_heart_context(
        CombatItemContext::new(relics, vec![None; 3]), character,
    );
    let context = if complete_items { context.with_complete_items() } else { context };
    let mut owner = Some(context);
    Combat::enter_with_item_context(entry, &mut owner)
        .unwrap_or_else(|error| panic!("{character:?} complete Heart entry: {error:?}"))
}

fn normalized_state_debug(combat: &Combat, complete_items: bool, character: Character) -> String {
    let state = format!("{:?}", combat);
    assert_eq!(state.matches("complete_cards: true").count(), 1, "{character:?} card capability is present once");
    assert_eq!(state.matches("complete_items: true").count(), usize::from(complete_items),
        "{character:?} item capability matches the fixture");
    state.replace(", complete_cards: true", "").replace(", complete_items: true", "")
}

#[test]
fn m27_capability_overlay_inert() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let relics = heart_items(character);
        let mut profile = inert_combat(character, relics.clone(), false);
        let mut complete = inert_combat(character, relics, true);
        for step in 0..400 {
            assert_eq!(profile.public_full().unwrap(), complete.public_full().unwrap(),
                "{character:?} public_full at action {step}");
            assert_eq!(normalized_state_debug(&profile, false, character),
                normalized_state_debug(&complete, true, character),
                "{character:?} State Debug at action {step}");
            let profile_actions = profile.legal_actions_full().unwrap();
            let complete_actions = complete.legal_actions_full().unwrap();
            assert_eq!(profile_actions, complete_actions, "{character:?} legal actions at action {step}");
            let Some(action) = profile_actions.iter().find(|action| matches!(action, CombatAction::ConfirmSelection))
                .or_else(|| profile_actions.iter().find(|action| !matches!(action,
                    CombatAction::EndTurn | CombatAction::DeselectCard { .. })))
                .or_else(|| profile_actions.first()).cloned() else { break };
            profile.step_full(&action).unwrap_or_else(|error| panic!("{character:?} profile {action:?}: {error:?}"));
            complete.step_full(&action).unwrap_or_else(|error| panic!("{character:?} complete {action:?}: {error:?}"));
        }
    }
}
